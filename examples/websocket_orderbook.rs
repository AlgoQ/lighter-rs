//! Example: Real-time Order Book Updates via WebSocket
//!
//! This example demonstrates how to:
//! 1. Connect to Lighter WebSocket
//! 2. Subscribe to order book updates
//! 3. Handle real-time price updates
//!
//! Run with: cargo run --example websocket_orderbook

use lighter_rs::ws_client::{ManagedOrderBook, WsClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Lighter RS - WebSocket Order Book Example      ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    // Create WebSocket client
    let client = WsClient::builder()
        .host("api-testnet.lighter.xyz")
        .order_books(vec![0, 1]) // Subscribe to markets 0 and 1
        .build()?;

    println!("Connecting to WebSocket...");
    println!("Subscriptions: markets 0, 1\n");

    // Define callback for order book updates
    let on_order_book_update = |market_id: String, order_book: &ManagedOrderBook| {
        println!("═══ Order Book Update: Market {} ═══", market_id);
        println!("  Offset: {} | Timestamp: {}", order_book.offset, order_book.timestamp);

        // Display top 5 asks using new helper methods
        println!("\n  📈 Top 5 Asks (Sell Orders):");
        for (i, level) in order_book.top_asks(5).iter().enumerate() {
            println!(
                "    {}. Price: {:>12} | Size: {:>12}",
                i + 1,
                level.price,
                level.size
            );
        }

        // Display top 5 bids using new helper methods
        println!("\n  📉 Top 5 Bids (Buy Orders):");
        for (i, level) in order_book.top_bids(5).iter().enumerate() {
            println!(
                "    {}. Price: {:>12} | Size: {:>12}",
                i + 1,
                level.price,
                level.size
            );
        }

        // Use new helper methods for spread calculation
        if let Some(spread_bps) = order_book.spread_bps() {
            if let Some(spread) = order_book.spread() {
                println!("\n  💰 Spread: {} ({:.2} bps)", spread, spread_bps);
            }
        }

        // Display order book depth info
        println!("  📊 Depth: {} asks, {} bids", order_book.ask_depth(), order_book.bid_depth());
        println!("  📊 Volume: {} total asks, {} total bids",
            order_book.total_ask_volume(), order_book.total_bid_volume());

        println!("\n{}\n", "─".repeat(50));
    };

    // Placeholder for account updates (not used in this example)
    let on_account_update = |_account_id: String, _account_data: serde_json::Value| {};

    println!("Starting WebSocket stream...");
    println!("Press Ctrl+C to stop\n");
    println!("{}\n", "═".repeat(50));

    // Run the WebSocket client
    client.run(on_order_book_update, on_account_update).await?;

    Ok(())
}
