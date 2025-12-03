//! WebSocket client for real-time Lighter Protocol data streams
//!
//! This module provides WebSocket connectivity to subscribe to:
//! - Order book updates
//! - Account updates
//! - Real-time trading data
//!
//! # Order Book Management
//!
//! The order book is maintained as a sorted structure with:
//! - Asks sorted ascending by price (lowest first)
//! - Bids sorted descending by price (highest first)
//!
//! Lighter provides full order book depth (all available price levels).
//! Updates are applied incrementally, with size "0" indicating level removal.

use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::errors::{LighterError, Result};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessageType {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "subscribed/order_book")]
    SubscribedOrderBook,
    #[serde(rename = "update/order_book")]
    UpdateOrderBook,
    #[serde(rename = "subscribed/account_all")]
    SubscribedAccount,
    #[serde(rename = "update/account_all")]
    UpdateAccount,
}

/// Subscription request message
#[derive(Debug, Clone, Serialize)]
struct SubscribeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    channel: String,
}

/// Price level in order book (for external API compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriceLevel {
    pub price: String,
    pub size: String,
}

impl PriceLevel {
    /// Create a new price level
    pub fn new(price: impl Into<String>, size: impl Into<String>) -> Self {
        Self {
            price: price.into(),
            size: size.into(),
        }
    }

    /// Get price as Decimal
    pub fn price_decimal(&self) -> Option<Decimal> {
        Decimal::from_str(&self.price).ok()
    }

    /// Get size as Decimal
    pub fn size_decimal(&self) -> Option<Decimal> {
        Decimal::from_str(&self.size).ok()
    }
}

/// Wrapper for Decimal that implements ordering for BTreeMap keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrderedDecimal(Decimal);

impl PartialOrd for OrderedDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

/// Managed order book with efficient sorted storage
///
/// Lighter provides full order book depth - all available price levels.
/// This struct maintains the order book state with:
/// - Asks sorted ascending by price (best ask = lowest price first)
/// - Bids sorted descending by price (best bid = highest price first)
/// - Sequence tracking via offset/nonce for consistency validation
#[derive(Debug, Clone)]
pub struct ManagedOrderBook {
    /// Ask levels keyed by price (sorted ascending)
    asks: BTreeMap<OrderedDecimal, Decimal>,
    /// Bid levels keyed by price (sorted ascending, but we iterate in reverse)
    bids: BTreeMap<OrderedDecimal, Decimal>,
    /// Current offset (sequence number)
    pub offset: u64,
    /// Current nonce
    pub nonce: u64,
    /// Last update timestamp (milliseconds)
    pub timestamp: u64,
    /// Market ID
    pub market_id: String,
}

impl ManagedOrderBook {
    /// Create a new empty managed order book
    pub fn new(market_id: impl Into<String>) -> Self {
        Self {
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            offset: 0,
            nonce: 0,
            timestamp: 0,
            market_id: market_id.into(),
        }
    }

    /// Apply a snapshot (initial order book state)
    pub fn apply_snapshot(&mut self, snapshot: &OrderBookSnapshot) -> Result<()> {
        self.asks.clear();
        self.bids.clear();

        // Parse and insert asks
        for level in &snapshot.asks {
            let price = Decimal::from_str(&level.price).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid ask price '{}': {}", level.price, e))
            })?;
            let size = Decimal::from_str(&level.size).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid ask size '{}': {}", level.size, e))
            })?;
            if size > Decimal::ZERO {
                self.asks.insert(OrderedDecimal(price), size);
            }
        }

        // Parse and insert bids
        for level in &snapshot.bids {
            let price = Decimal::from_str(&level.price).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid bid price '{}': {}", level.price, e))
            })?;
            let size = Decimal::from_str(&level.size).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid bid size '{}': {}", level.size, e))
            })?;
            if size > Decimal::ZERO {
                self.bids.insert(OrderedDecimal(price), size);
            }
        }

        self.offset = snapshot.offset;
        self.nonce = snapshot.nonce;

        Ok(())
    }

    /// Apply an incremental update
    pub fn apply_update(&mut self, update: &OrderBookUpdate) -> Result<()> {
        // Validate sequence (offset should be monotonically increasing)
        if update.offset <= self.offset && self.offset > 0 {
            // Skip stale updates
            return Ok(());
        }

        // Apply ask updates
        for level in &update.asks {
            let price = Decimal::from_str(&level.price).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid ask price '{}': {}", level.price, e))
            })?;
            let size = Decimal::from_str(&level.size).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid ask size '{}': {}", level.size, e))
            })?;

            if size == Decimal::ZERO {
                self.asks.remove(&OrderedDecimal(price));
            } else {
                self.asks.insert(OrderedDecimal(price), size);
            }
        }

        // Apply bid updates
        for level in &update.bids {
            let price = Decimal::from_str(&level.price).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid bid price '{}': {}", level.price, e))
            })?;
            let size = Decimal::from_str(&level.size).map_err(|e| {
                LighterError::InvalidResponse(format!("Invalid bid size '{}': {}", level.size, e))
            })?;

            if size == Decimal::ZERO {
                self.bids.remove(&OrderedDecimal(price));
            } else {
                self.bids.insert(OrderedDecimal(price), size);
            }
        }

        self.offset = update.offset;
        self.nonce = update.nonce;

        Ok(())
    }

    /// Get the best ask (lowest price)
    pub fn best_ask(&self) -> Option<PriceLevel> {
        self.asks.iter().next().map(|(price, size)| PriceLevel {
            price: price.0.to_string(),
            size: size.to_string(),
        })
    }

    /// Get the best bid (highest price)
    pub fn best_bid(&self) -> Option<PriceLevel> {
        self.bids.iter().next_back().map(|(price, size)| PriceLevel {
            price: price.0.to_string(),
            size: size.to_string(),
        })
    }

    /// Get the spread (best ask - best bid)
    pub fn spread(&self) -> Option<Decimal> {
        match (self.asks.iter().next(), self.bids.iter().next_back()) {
            (Some((ask_price, _)), Some((bid_price, _))) => Some(ask_price.0 - bid_price.0),
            _ => None,
        }
    }

    /// Get spread in basis points
    pub fn spread_bps(&self) -> Option<Decimal> {
        match (self.asks.iter().next(), self.bids.iter().next_back()) {
            (Some((ask_price, _)), Some((bid_price, _))) if bid_price.0 > Decimal::ZERO => {
                let spread = ask_price.0 - bid_price.0;
                Some(spread / bid_price.0 * Decimal::from(10000))
            }
            _ => None,
        }
    }

    /// Get mid price
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.asks.iter().next(), self.bids.iter().next_back()) {
            (Some((ask_price, _)), Some((bid_price, _))) => {
                Some((ask_price.0 + bid_price.0) / Decimal::from(2))
            }
            _ => None,
        }
    }

    /// Get top N ask levels (sorted by price ascending - best first)
    pub fn top_asks(&self, n: usize) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .take(n)
            .map(|(price, size)| PriceLevel {
                price: price.0.to_string(),
                size: size.to_string(),
            })
            .collect()
    }

    /// Get top N bid levels (sorted by price descending - best first)
    pub fn top_bids(&self, n: usize) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .take(n)
            .map(|(price, size)| PriceLevel {
                price: price.0.to_string(),
                size: size.to_string(),
            })
            .collect()
    }

    /// Get all ask levels (sorted by price ascending)
    pub fn all_asks(&self) -> Vec<PriceLevel> {
        self.asks
            .iter()
            .map(|(price, size)| PriceLevel {
                price: price.0.to_string(),
                size: size.to_string(),
            })
            .collect()
    }

    /// Get all bid levels (sorted by price descending)
    pub fn all_bids(&self) -> Vec<PriceLevel> {
        self.bids
            .iter()
            .rev()
            .map(|(price, size)| PriceLevel {
                price: price.0.to_string(),
                size: size.to_string(),
            })
            .collect()
    }

    /// Get total number of ask levels
    pub fn ask_depth(&self) -> usize {
        self.asks.len()
    }

    /// Get total number of bid levels
    pub fn bid_depth(&self) -> usize {
        self.bids.len()
    }

    /// Get total ask volume
    pub fn total_ask_volume(&self) -> Decimal {
        self.asks.values().sum()
    }

    /// Get total bid volume
    pub fn total_bid_volume(&self) -> Decimal {
        self.bids.values().sum()
    }

    /// Get ask volume up to a price level
    pub fn ask_volume_to_price(&self, price: Decimal) -> Decimal {
        self.asks
            .iter()
            .take_while(|(p, _)| p.0 <= price)
            .map(|(_, size)| size)
            .sum()
    }

    /// Get bid volume down to a price level
    pub fn bid_volume_to_price(&self, price: Decimal) -> Decimal {
        self.bids
            .iter()
            .rev()
            .take_while(|(p, _)| p.0 >= price)
            .map(|(_, size)| size)
            .sum()
    }

    /// Convert to legacy OrderBook format for backward compatibility
    pub fn to_order_book(&self) -> OrderBook {
        OrderBook {
            asks: self.top_asks(self.ask_depth()),
            bids: self.top_bids(self.bid_depth()),
        }
    }
}

/// Raw order book snapshot from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
    #[serde(default)]
    pub offset: u64,
    #[serde(default)]
    pub nonce: u64,
    #[serde(default)]
    pub code: i32,
}

/// Raw order book update from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookUpdate {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
    #[serde(default)]
    pub offset: u64,
    #[serde(default)]
    pub nonce: u64,
    #[serde(default)]
    pub code: i32,
}

/// Order book data structure (legacy format for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderBook {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
}

/// WebSocket client configuration
pub struct WsClientBuilder {
    host: Option<String>,
    path: String,
    order_book_ids: Vec<u32>,
    account_ids: Vec<i64>,
}

impl WsClientBuilder {
    /// Create a new WebSocket client builder
    pub fn new() -> Self {
        Self {
            host: None,
            path: "/stream".to_string(),
            order_book_ids: Vec::new(),
            account_ids: Vec::new(),
        }
    }

    /// Set the WebSocket host (defaults to testnet)
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the WebSocket path (defaults to "/stream")
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Subscribe to order book updates for specific markets
    pub fn order_books(mut self, ids: Vec<u32>) -> Self {
        self.order_book_ids = ids;
        self
    }

    /// Subscribe to account updates for specific accounts
    pub fn accounts(mut self, ids: Vec<i64>) -> Self {
        self.account_ids = ids;
        self
    }

    /// Build the WebSocket client
    pub fn build(self) -> Result<WsClient> {
        if self.order_book_ids.is_empty() && self.account_ids.is_empty() {
            return Err(LighterError::ValidationError(
                "At least one subscription (order_book or account) is required".to_string(),
            ));
        }

        let host = self
            .host
            .unwrap_or_else(|| "api-testnet.lighter.xyz".to_string());
        let base_url = format!("wss://{}{}", host, self.path);

        Ok(WsClient {
            base_url,
            order_book_ids: self.order_book_ids,
            account_ids: self.account_ids,
            order_book_states: Arc::new(RwLock::new(HashMap::new())),
            account_states: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl Default for WsClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket client for Lighter Protocol
pub struct WsClient {
    base_url: String,
    order_book_ids: Vec<u32>,
    account_ids: Vec<i64>,
    order_book_states: Arc<RwLock<HashMap<String, ManagedOrderBook>>>,
    account_states: Arc<RwLock<HashMap<String, Value>>>,
}

impl std::fmt::Debug for WsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsClient")
            .field("base_url", &self.base_url)
            .field("order_book_ids", &self.order_book_ids)
            .field("account_ids", &self.account_ids)
            .finish()
    }
}

impl WsClient {
    /// Create a new WebSocket client builder
    pub fn builder() -> WsClientBuilder {
        WsClientBuilder::new()
    }

    /// Run the WebSocket client with callbacks using the new ManagedOrderBook
    ///
    /// # Arguments
    /// * `on_order_book_update` - Callback for order book updates (market_id, managed_order_book)
    /// * `on_account_update` - Callback for account updates (account_id, account_data)
    pub async fn run<F1, F2>(&self, on_order_book_update: F1, on_account_update: F2) -> Result<()>
    where
        F1: Fn(String, &ManagedOrderBook) + Send + Sync + 'static,
        F2: Fn(String, Value) + Send + Sync + 'static,
    {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.base_url).await.map_err(|e| {
            LighterError::InvalidConfiguration(format!("WebSocket connection failed: {}", e))
        })?;

        println!("✓ WebSocket connected to {}", self.base_url);

        let (mut write, mut read) = ws_stream.split();

        // Clone states for message handler
        let order_book_states = self.order_book_states.clone();
        let account_states = self.account_states.clone();
        let order_book_ids = self.order_book_ids.clone();
        let account_ids = self.account_ids.clone();

        // Wrap callbacks in Arc for sharing
        let on_order_book_update = Arc::new(on_order_book_update);
        let on_account_update = Arc::new(on_account_update);

        // Message handling loop
        while let Some(message) = read.next().await {
            let message = message
                .map_err(|e| LighterError::InvalidResponse(format!("WebSocket error: {}", e)))?;

            if let Message::Text(text) = message {
                let parsed: Value = serde_json::from_str(&text)?;
                let msg_type = parsed.get("type").and_then(|t| t.as_str());

                match msg_type {
                    Some("connected") => {
                        println!("✓ WebSocket connection established");
                        // Send subscriptions
                        for market_id in &order_book_ids {
                            let sub_msg = SubscribeMessage {
                                msg_type: "subscribe".to_string(),
                                channel: format!("order_book/{}", market_id),
                            };
                            let json = serde_json::to_string(&sub_msg)?;
                            write.send(Message::Text(json)).await.map_err(|e| {
                                LighterError::InvalidResponse(format!("Send error: {}", e))
                            })?;
                            println!("  → Subscribed to order_book/{}", market_id);
                        }

                        for account_id in &account_ids {
                            let sub_msg = SubscribeMessage {
                                msg_type: "subscribe".to_string(),
                                channel: format!("account_all/{}", account_id),
                            };
                            let json = serde_json::to_string(&sub_msg)?;
                            write.send(Message::Text(json)).await.map_err(|e| {
                                LighterError::InvalidResponse(format!("Send error: {}", e))
                            })?;
                            println!("  → Subscribed to account_all/{}", account_id);
                        }
                    }
                    Some("subscribed/order_book") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let market_id = channel.split(':').nth(1).unwrap_or("unknown");
                            let timestamp = parsed
                                .get("timestamp")
                                .and_then(|t| t.as_u64())
                                .unwrap_or(0);

                            if let Some(order_book_data) = parsed.get("order_book") {
                                let snapshot: OrderBookSnapshot =
                                    serde_json::from_value(order_book_data.clone())?;

                                let mut managed_ob = ManagedOrderBook::new(market_id);
                                managed_ob.timestamp = timestamp;
                                managed_ob.apply_snapshot(&snapshot)?;

                                let mut states = order_book_states.write().await;
                                states.insert(market_id.to_string(), managed_ob);

                                // Call callback with reference to stored state
                                if let Some(ob) = states.get(market_id) {
                                    on_order_book_update(market_id.to_string(), ob);
                                }
                            }
                        }
                    }
                    Some("update/order_book") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let market_id = channel.split(':').nth(1).unwrap_or("unknown");
                            let timestamp = parsed
                                .get("timestamp")
                                .and_then(|t| t.as_u64())
                                .unwrap_or(0);

                            if let Some(order_book_data) = parsed.get("order_book") {
                                let update: OrderBookUpdate =
                                    serde_json::from_value(order_book_data.clone())?;

                                let mut states = order_book_states.write().await;
                                if let Some(existing) = states.get_mut(market_id) {
                                    existing.timestamp = timestamp;
                                    existing.apply_update(&update)?;
                                    on_order_book_update(market_id.to_string(), existing);
                                }
                            }
                        }
                    }
                    Some("subscribed/account_all") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let account_id = channel.split(':').nth(1).unwrap_or("unknown");
                            account_states
                                .write()
                                .await
                                .insert(account_id.to_string(), parsed.clone());
                            on_account_update(account_id.to_string(), parsed);
                        }
                    }
                    Some("update/account_all") => {
                        if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                            let account_id = channel.split(':').nth(1).unwrap_or("unknown");
                            account_states
                                .write()
                                .await
                                .insert(account_id.to_string(), parsed.clone());
                            on_account_update(account_id.to_string(), parsed);
                        }
                    }
                    _ => {
                        // Silently ignore unhandled message types
                    }
                }
            }
        }

        Ok(())
    }

    /// Run the WebSocket client with legacy OrderBook callback (for backward compatibility)
    ///
    /// # Arguments
    /// * `on_order_book_update` - Callback for order book updates (market_id, order_book)
    /// * `on_account_update` - Callback for account updates (account_id, account_data)
    pub async fn run_legacy<F1, F2>(
        &self,
        on_order_book_update: F1,
        on_account_update: F2,
    ) -> Result<()>
    where
        F1: Fn(String, OrderBook) + Send + Sync + 'static,
        F2: Fn(String, Value) + Send + Sync + 'static,
    {
        self.run(
            move |market_id, managed_ob: &ManagedOrderBook| {
                on_order_book_update(market_id, managed_ob.to_order_book());
            },
            on_account_update,
        )
        .await
    }

    /// Get current managed order book state for a market
    pub async fn get_order_book(&self, market_id: &str) -> Option<ManagedOrderBook> {
        self.order_book_states.read().await.get(market_id).cloned()
    }

    /// Get current order book as legacy OrderBook format
    pub async fn get_order_book_legacy(&self, market_id: &str) -> Option<OrderBook> {
        self.order_book_states
            .read()
            .await
            .get(market_id)
            .map(|ob| ob.to_order_book())
    }

    /// Get current account state
    pub async fn get_account(&self, account_id: &str) -> Option<Value> {
        self.account_states.read().await.get(account_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_client_builder() {
        let client = WsClient::builder()
            .order_books(vec![0, 1])
            .accounts(vec![12345])
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_ws_client_builder_no_subscriptions() {
        let client = WsClient::builder().build();

        assert!(client.is_err());
        assert!(matches!(
            client.unwrap_err(),
            LighterError::ValidationError(_)
        ));
    }

    #[test]
    fn test_managed_order_book_snapshot() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![
                PriceLevel::new("100.0", "10.0"),
                PriceLevel::new("101.0", "5.0"),
                PriceLevel::new("99.5", "8.0"), // Out of order - should still work
            ],
            bids: vec![
                PriceLevel::new("98.0", "15.0"),
                PriceLevel::new("97.0", "20.0"),
                PriceLevel::new("99.0", "12.0"), // Out of order - should still work
            ],
            offset: 100,
            nonce: 12345,
            code: 0,
        };

        ob.apply_snapshot(&snapshot).unwrap();

        // Check asks are sorted ascending (best ask = lowest price)
        let asks = ob.top_asks(3);
        assert_eq!(asks.len(), 3);
        assert_eq!(asks[0].price, "99.5"); // Best ask (lowest)
        assert_eq!(asks[1].price, "100.0");
        assert_eq!(asks[2].price, "101.0");

        // Check bids are sorted descending (best bid = highest price)
        let bids = ob.top_bids(3);
        assert_eq!(bids.len(), 3);
        assert_eq!(bids[0].price, "99.0"); // Best bid (highest)
        assert_eq!(bids[1].price, "98.0");
        assert_eq!(bids[2].price, "97.0");

        assert_eq!(ob.offset, 100);
        assert_eq!(ob.nonce, 12345);
    }

    #[test]
    fn test_managed_order_book_update() {
        let mut ob = ManagedOrderBook::new("0");

        // Initial snapshot
        let snapshot = OrderBookSnapshot {
            asks: vec![
                PriceLevel::new("100.0", "10.0"),
                PriceLevel::new("101.0", "5.0"),
            ],
            bids: vec![
                PriceLevel::new("99.0", "15.0"),
                PriceLevel::new("98.0", "20.0"),
            ],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        // Apply update - modify existing, add new, remove one
        let update = OrderBookUpdate {
            asks: vec![
                PriceLevel::new("100.0", "15.0"),  // Update existing
                PriceLevel::new("102.0", "8.0"),   // Add new
                PriceLevel::new("101.0", "0.0000"), // Remove (zero size)
            ],
            bids: vec![
                PriceLevel::new("99.5", "10.0"), // Add new (becomes best bid)
            ],
            offset: 101,
            nonce: 12346,
            code: 0,
        };
        ob.apply_update(&update).unwrap();

        // Verify asks
        let asks = ob.all_asks();
        assert_eq!(asks.len(), 2); // 101.0 was removed
        assert_eq!(asks[0].price, "100.0");
        assert_eq!(asks[0].size, "15.0"); // Updated
        assert_eq!(asks[1].price, "102.0"); // New level

        // Verify bids
        let bids = ob.all_bids();
        assert_eq!(bids.len(), 3);
        assert_eq!(bids[0].price, "99.5"); // New best bid
        assert_eq!(bids[0].size, "10.0");

        assert_eq!(ob.offset, 101);
        assert_eq!(ob.nonce, 12346);
    }

    #[test]
    fn test_managed_order_book_stale_update() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![PriceLevel::new("100.0", "10.0")],
            bids: vec![PriceLevel::new("99.0", "15.0")],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        // Stale update (offset <= current offset) should be ignored
        let stale_update = OrderBookUpdate {
            asks: vec![PriceLevel::new("100.0", "999.0")],
            bids: vec![],
            offset: 100, // Same as current
            nonce: 12344,
            code: 0,
        };
        ob.apply_update(&stale_update).unwrap();

        // Should still have original value
        let asks = ob.all_asks();
        assert_eq!(asks[0].size, "10.0");
    }

    #[test]
    fn test_managed_order_book_best_bid_ask() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![
                PriceLevel::new("100.5", "10.0"),
                PriceLevel::new("100.0", "5.0"), // Best ask
                PriceLevel::new("101.0", "8.0"),
            ],
            bids: vec![
                PriceLevel::new("99.0", "15.0"), // Best bid
                PriceLevel::new("98.0", "20.0"),
            ],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        let best_ask = ob.best_ask().unwrap();
        assert_eq!(best_ask.price, "100.0");
        assert_eq!(best_ask.size, "5.0");

        let best_bid = ob.best_bid().unwrap();
        assert_eq!(best_bid.price, "99.0");
        assert_eq!(best_bid.size, "15.0");
    }

    #[test]
    fn test_managed_order_book_spread() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![PriceLevel::new("100.5", "10.0")],
            bids: vec![PriceLevel::new("99.5", "15.0")],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        let spread = ob.spread().unwrap();
        assert_eq!(spread, Decimal::from_str("1.0").unwrap());

        let mid = ob.mid_price().unwrap();
        assert_eq!(mid, Decimal::from_str("100.0").unwrap());
    }

    #[test]
    fn test_managed_order_book_depth() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![
                PriceLevel::new("100.0", "10.0"),
                PriceLevel::new("101.0", "5.0"),
                PriceLevel::new("102.0", "8.0"),
            ],
            bids: vec![
                PriceLevel::new("99.0", "15.0"),
                PriceLevel::new("98.0", "20.0"),
            ],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        assert_eq!(ob.ask_depth(), 3);
        assert_eq!(ob.bid_depth(), 2);

        // Test total volume
        assert_eq!(ob.total_ask_volume(), Decimal::from_str("23.0").unwrap());
        assert_eq!(ob.total_bid_volume(), Decimal::from_str("35.0").unwrap());
    }

    #[test]
    fn test_managed_order_book_volume_to_price() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![
                PriceLevel::new("100.0", "10.0"),
                PriceLevel::new("101.0", "5.0"),
                PriceLevel::new("102.0", "8.0"),
            ],
            bids: vec![
                PriceLevel::new("99.0", "15.0"),
                PriceLevel::new("98.0", "20.0"),
                PriceLevel::new("97.0", "25.0"),
            ],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        // Ask volume up to price 101.0 (includes 100.0 and 101.0)
        let ask_vol = ob.ask_volume_to_price(Decimal::from_str("101.0").unwrap());
        assert_eq!(ask_vol, Decimal::from_str("15.0").unwrap());

        // Bid volume down to price 98.0 (includes 99.0 and 98.0)
        let bid_vol = ob.bid_volume_to_price(Decimal::from_str("98.0").unwrap());
        assert_eq!(bid_vol, Decimal::from_str("35.0").unwrap());
    }

    #[test]
    fn test_managed_order_book_to_legacy() {
        let mut ob = ManagedOrderBook::new("0");

        let snapshot = OrderBookSnapshot {
            asks: vec![
                PriceLevel::new("101.0", "5.0"),
                PriceLevel::new("100.0", "10.0"), // Should be first after sorting
            ],
            bids: vec![
                PriceLevel::new("98.0", "20.0"),
                PriceLevel::new("99.0", "15.0"), // Should be first after sorting
            ],
            offset: 100,
            nonce: 12345,
            code: 0,
        };
        ob.apply_snapshot(&snapshot).unwrap();

        let legacy = ob.to_order_book();

        // Asks should be sorted ascending
        assert_eq!(legacy.asks[0].price, "100.0");
        assert_eq!(legacy.asks[1].price, "101.0");

        // Bids should be sorted descending
        assert_eq!(legacy.bids[0].price, "99.0");
        assert_eq!(legacy.bids[1].price, "98.0");
    }

    #[test]
    fn test_price_level_new() {
        let level = PriceLevel::new("100.5", "10.25");
        assert_eq!(level.price, "100.5");
        assert_eq!(level.size, "10.25");

        assert_eq!(
            level.price_decimal(),
            Some(Decimal::from_str("100.5").unwrap())
        );
        assert_eq!(
            level.size_decimal(),
            Some(Decimal::from_str("10.25").unwrap())
        );
    }
}
