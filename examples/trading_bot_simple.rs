//! Example: Simple Trading Bot with WebSocket Monitoring
//!
//! This example demonstrates a complete trading bot that:
//! 1. Monitors order book via WebSocket
//! 2. Monitors account state via WebSocket
//! 3. Places orders based on market conditions
//!
//! This is a simple example for educational purposes.
//! DO NOT use in production without proper risk management!
//!
//! Prerequisites - .env file:
//! - Set LIGHTER_API_KEY
//! - Set LIGHTER_ACCOUNT_INDEX
//! - Set LIGHTER_API_KEY_INDEX
//!
//! Run with: cargo run --example trading_bot_simple

use lighter_rs::client::TxClient;
use lighter_rs::ws_client::{ManagedOrderBook, WsClient};
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Lighter RS - Simple Trading Bot Example        ║");
    println!("║   WARNING: Educational purposes only!             ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    // Load .env file
    dotenv::dotenv().ok();

    // Load configuration
    let api_key =
        env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY environment variable not set");

    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    let market_index = 0u8; // Trading on market 0 -> ETH
    let url = "https://mainnet.zklighter.elliot.ai";
    let url_ws = "mainnet.zklighter.elliot.ai";

    println!("Bot Configuration:");
    println!("  Account: {}", account_index);
    println!("  Market: {}", market_index);
    println!("  Mode: Demo (Educational)\n");

    // Create trading client
    let tx_client = Arc::new(TxClient::new(
        url,
        &api_key,
        account_index,
        api_key_index, // api_key_index
        304,           // 300 Testnet; 304 Mainnet
    )?);

    // Flag to track if we've placed an order
    let order_placed = Arc::new(AtomicBool::new(false));

    println!("✓ Trading client initialized\n");

    // Create WebSocket client
    let ws_client = WsClient::builder()
        .host(url_ws)
        .order_books(vec![market_index as u32])
        .accounts(vec![account_index])
        .build()?;

    println!("✓ WebSocket client created");
    println!("  Monitoring market {}", market_index);
    println!("  Monitoring account {}\n", account_index);

    // Clone for callbacks
    let tx_client_clone = tx_client.clone();
    let order_placed_clone = order_placed.clone();

    // Order book callback - Simple trading logic
    let on_order_book_update = move |market_id: String, order_book: &ManagedOrderBook| {
        println!("📊 Order Book Update - Market {}", market_id);

        if let (Some(best_ask), Some(best_bid)) =
            (order_book.best_ask(), order_book.best_bid())
        {
            println!("  Best Ask: {} @ {}", best_ask.size, best_ask.price);
            println!("  Best Bid: {} @ {}", best_bid.size, best_bid.price);

            // Get spread using new helper methods
            let ask_f64 = best_ask.price.parse::<f64>().unwrap_or(0.0);
            let bid_f64 = best_bid.price.parse::<f64>().unwrap_or(0.0);
            let spread_bps = order_book.spread_bps().and_then(|s| s.to_f64()).unwrap_or(0.0);

            println!("  Spread: {:.2} bps", spread_bps);

            // Simple trading logic: Place order if spread > 10 bps
            if spread_bps > 10.0 && !order_placed_clone.load(Ordering::Relaxed) {
                println!("\n  🎯 Spread > 10 bps detected! Placing order...");

                let tx_client = tx_client_clone.clone();
                let order_placed = order_placed_clone.clone();
                let mid_price = ((ask_f64 + bid_f64) / 2.0) as u32;

                // Spawn task to place order (non-blocking)
                tokio::spawn(async move {
                    match tx_client
                        .create_market_order(
                            market_index,
                            chrono::Utc::now().timestamp_millis(),
                            100_000, // Small size for demo
                            mid_price,
                            0,     // BUY
                            false, // not reduce-only
                            None,
                        )
                        .await
                    {
                        Ok(order) => {
                            println!("  ✓ Order created and signed");
                            match tx_client.send_transaction(&order).await {
                                Ok(response) => {
                                    if response.code == 200 {
                                        println!("  ✓ Order submitted successfully!");
                                        if let Some(hash) = response.tx_hash {
                                            println!("    Tx Hash: {}", hash);
                                        }
                                        order_placed.store(true, Ordering::Relaxed);
                                    } else {
                                        println!("  ✗ Order failed: {:?}", response.message);
                                    }
                                }
                                Err(e) => println!("  ✗ Submit error: {}", e),
                            }
                        }
                        Err(e) => println!("  ✗ Order creation error: {}", e),
                    }
                });
            }
        }
        println!();
    };

    // Account callback - Monitor our positions
    let on_account_update = move |account_id: String, account_data: Value| {
        println!("👤 Account Update - ID: {}", account_id);

        if let Some(obj) = account_data.as_object() {
            if let Some(balance) = obj.get("usdc_balance") {
                println!("  Balance: {} USDC", balance);
            }

            if let Some(orders) = obj.get("orders").and_then(|o| o.as_array()) {
                println!("  Active Orders: {}", orders.len());
            }

            if let Some(positions) = obj.get("positions").and_then(|p| p.as_array()) {
                if !positions.is_empty() {
                    println!("  Open Positions: {}", positions.len());
                }
            }

            if let Some(pnl) = obj.get("unrealized_pnl") {
                println!("  Unrealized PnL: {}", pnl);
            }
        }
        println!();
    };

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Trading Bot Started                             ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!("\nStrategy: Place order when spread > 10 bps");
    println!("Press Ctrl+C to stop\n");
    println!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    ws_client
        .run(on_order_book_update, on_account_update)
        .await?;

    Ok(())
}
