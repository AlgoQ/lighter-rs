//! Simplified Market Maker with Volatility Score
//!
//! This market maker:
//! 1. Uses vol_score to dynamically adjust spread based on volatility
//! 2. Places orders on BOTH sides (bid and ask)
//! 3. Logs when position changes
//!
//! Prerequisites - .env file:
//! - Set LIGHTER_API_KEY
//! - Set LIGHTER_ACCOUNT_INDEX
//! - Set LIGHTER_API_KEY_INDEX
//!
//! Run with: cargo run --example mm_2511

use lighter_rs::client::TxClient;
use lighter_rs::ws_client::{OrderBook, WsClient};
use serde_json::Value;
use std::collections::VecDeque;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the market maker
#[derive(Clone)]
struct MmConfig {
    market_index: u8,
    base_spread_bps: f64,     // Base spread in basis points
    order_size: i64,          // Size per order
    vol_lookback: usize,      // Number of samples for volatility calculation
    max_spread_multiplier: f64, // Max spread = base_spread * multiplier when vol is high
}

impl Default for MmConfig {
    fn default() -> Self {
        Self {
            market_index: 0,
            base_spread_bps: 5.0,
            order_size: 100_000, // 0.001 ETH in base units
            vol_lookback: 20,
            max_spread_multiplier: 3.0,
        }
    }
}

/// Market maker state
struct MmState {
    mid_prices: VecDeque<f64>,
    last_position: f64,
    orders_active: bool,
}

impl MmState {
    fn new(lookback: usize) -> Self {
        Self {
            mid_prices: VecDeque::with_capacity(lookback),
            last_position: 0.0,
            orders_active: false,
        }
    }

    /// Calculate volatility score (0.0 to 1.0)
    /// Higher score = higher volatility
    fn vol_score(&self) -> f64 {
        if self.mid_prices.len() < 2 {
            return 0.0;
        }

        // Calculate returns
        let returns: Vec<f64> = self.mid_prices
            .iter()
            .zip(self.mid_prices.iter().skip(1))
            .map(|(prev, curr)| (curr - prev) / prev)
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        // Calculate standard deviation of returns
        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        // Normalize to 0-1 range (assuming max reasonable vol is 1% per tick)
        (std_dev * 100.0).min(1.0)
    }

    /// Add a new mid price observation
    fn update_mid_price(&mut self, mid_price: f64, lookback: usize) {
        self.mid_prices.push_back(mid_price);
        while self.mid_prices.len() > lookback {
            self.mid_prices.pop_front();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("  MM Vol Score - Two-Sided Market Maker");
    println!("  WARNING: Educational purposes only!");
    println!("========================================\n");

    dotenv::dotenv().ok();

    // Load configuration from environment
    let api_key =
        env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY environment variable not set");
    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");
    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")
        .expect("LIGHTER_API_KEY_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_API_KEY_INDEX must be a valid number");

    let config = MmConfig::default();
    let url = "https://mainnet.zklighter.elliot.ai";
    let url_ws = "mainnet.zklighter.elliot.ai";

    println!("Configuration:");
    println!("  Account: {}", account_index);
    println!("  Market: {}", config.market_index);
    println!("  Base Spread: {} bps", config.base_spread_bps);
    println!("  Order Size: {}", config.order_size);
    println!("  Vol Lookback: {} samples", config.vol_lookback);
    println!("  Max Spread Multiplier: {}x\n", config.max_spread_multiplier);

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        url,
        &api_key,
        account_index,
        api_key_index,
        304, // Mainnet chain ID
    )?);

    // Market maker state
    let mm_state = Arc::new(RwLock::new(MmState::new(config.vol_lookback)));

    // Flag to prevent concurrent order placement
    let placing_orders = Arc::new(AtomicBool::new(false));

    println!("[OK] Trading client initialized\n");

    // Create WebSocket client
    let ws_client = WsClient::builder()
        .host(url_ws)
        .order_books(vec![config.market_index as u32])
        .accounts(vec![account_index])
        .build()?;

    println!("[OK] WebSocket client created");
    println!("  Monitoring market {}", config.market_index);
    println!("  Monitoring account {}\n", account_index);

    // Clone for callbacks
    let tx_client_clone = tx_client.clone();
    let mm_state_clone = mm_state.clone();
    let placing_orders_clone = placing_orders.clone();
    let config_clone = config.clone();

    // Order book callback - Core MM logic
    let on_order_book_update = move |market_id: String, order_book: OrderBook| {
        let tx_client = tx_client_clone.clone();
        let mm_state = mm_state_clone.clone();
        let placing_orders = placing_orders_clone.clone();
        let config = config_clone.clone();

        tokio::spawn(async move {
            // Get best bid and ask
            let (best_bid, best_ask) = match (order_book.bids.first(), order_book.asks.first()) {
                (Some(bid), Some(ask)) => (bid, ask),
                _ => return,
            };

            // Parse prices
            let bid_price: f64 = match best_bid.price.parse() {
                Ok(p) => p,
                Err(_) => return,
            };
            let ask_price: f64 = match best_ask.price.parse() {
                Ok(p) => p,
                Err(_) => return,
            };

            let mid_price = (bid_price + ask_price) / 2.0;

            // Update state and calculate vol_score
            let vol_score = {
                let mut state = mm_state.write().await;
                state.update_mid_price(mid_price, config.vol_lookback);
                state.vol_score()
            };

            // Calculate spread based on volatility
            // Higher vol_score -> wider spread
            let spread_multiplier = 1.0 + (vol_score * (config.max_spread_multiplier - 1.0));
            let spread_bps = config.base_spread_bps * spread_multiplier;
            let half_spread = mid_price * (spread_bps / 10000.0) / 2.0;

            // Calculate our quote prices
            let our_bid = mid_price - half_spread;
            let our_ask = mid_price + half_spread;

            println!("Market {} | Mid: {:.2} | Vol Score: {:.3} | Spread: {:.2} bps",
                     market_id, mid_price, vol_score, spread_bps);
            println!("  -> Bid: {:.2} | Ask: {:.2}", our_bid, our_ask);

            // Place orders if not already placing
            if placing_orders.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                let market_index = config.market_index;
                let order_size = config.order_size;
                let timestamp = chrono::Utc::now().timestamp_millis();

                // Place BID order (is_ask = 0)
                let bid_result = tx_client
                    .create_limit_order(
                        market_index,
                        timestamp,
                        order_size,
                        our_bid as u32,
                        0,     // BUY (bid)
                        false, // not reduce-only
                        None,
                    )
                    .await;

                match bid_result {
                    Ok(bid_order) => {
                        match tx_client.send_transaction(&bid_order).await {
                            Ok(resp) if resp.code == 200 => {
                                println!("  [BID] Placed @ {:.2}", our_bid);
                            }
                            Ok(resp) => {
                                println!("  [BID] Failed: {:?}", resp.message);
                            }
                            Err(e) => println!("  [BID] Error: {}", e),
                        }
                    }
                    Err(e) => println!("  [BID] Create error: {}", e),
                }

                // Place ASK order (is_ask = 1)
                let ask_result = tx_client
                    .create_limit_order(
                        market_index,
                        timestamp + 1, // Different client_order_index
                        order_size,
                        our_ask as u32,
                        1,     // SELL (ask)
                        false, // not reduce-only
                        None,
                    )
                    .await;

                match ask_result {
                    Ok(ask_order) => {
                        match tx_client.send_transaction(&ask_order).await {
                            Ok(resp) if resp.code == 200 => {
                                println!("  [ASK] Placed @ {:.2}", our_ask);
                            }
                            Ok(resp) => {
                                println!("  [ASK] Failed: {:?}", resp.message);
                            }
                            Err(e) => println!("  [ASK] Error: {}", e),
                        }
                    }
                    Err(e) => println!("  [ASK] Create error: {}", e),
                }

                placing_orders.store(false, Ordering::SeqCst);
            }

            println!();
        });
    };

    // Account callback - Monitor position changes
    let mm_state_account = mm_state.clone();
    let on_account_update = move |account_id: String, account_data: Value| {
        let mm_state = mm_state_account.clone();

        tokio::spawn(async move {
            if let Some(obj) = account_data.as_object() {
                // Extract position from account data
                let current_position = if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                    positions.iter()
                        .filter_map(|pos| {
                            pos.get("size")
                                .and_then(|s| s.as_str())
                                .and_then(|s| s.parse::<f64>().ok())
                        })
                        .sum::<f64>()
                } else {
                    0.0
                };

                // Check for position change
                let mut state = mm_state.write().await;
                if (current_position - state.last_position).abs() > 0.0001 {
                    println!("========================================");
                    println!("  POSITION CHANGE DETECTED!");
                    println!("  Account: {}", account_id);
                    println!("  Previous: {:.6}", state.last_position);
                    println!("  Current:  {:.6}", current_position);
                    println!("  Delta:    {:.6}", current_position - state.last_position);
                    println!("========================================\n");

                    state.last_position = current_position;
                }

                // Log other account info
                if let Some(balance) = obj.get("usdc_balance") {
                    println!("Account {} | Balance: {} USDC | Position: {:.6}",
                             account_id, balance, current_position);
                }

                if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                    if !orders.is_empty() {
                        println!("  Active orders: {}", orders.len());
                    }
                }
            }
        });
    };

    println!("========================================");
    println!("  Market Maker Started");
    println!("  Strategy: Vol-adjusted two-sided quotes");
    println!("  Press Ctrl+C to stop");
    println!("========================================\n");

    // Run the WebSocket client
    ws_client
        .run(on_order_book_update, on_account_update)
        .await?;

    Ok(())
}
