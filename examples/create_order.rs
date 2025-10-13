//! Example: Creating and signing a limit order
//!
//! This example demonstrates how to create a limit order transaction using the Lighter SDK.
//!
//! Run with: cargo run --example create_order

use lighter_rs::client::TxClient;
use lighter_rs::constants::*;
use lighter_rs::types::{CreateOrderTxReq, TxInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lighter RS: Create Order Example ===\n");

    // Initialize the transaction client
    // Note: In production, use environment variables for sensitive data
    let tx_client = TxClient::new(
        "", // Empty string disables API calls (offline mode)
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        12345, // Your account index
        0,     // API key index
        1,     // Chain ID
    )?;

    println!("✓ Transaction client initialized");
    println!("  Account Index: {}", tx_client.account_index());
    println!("  API Key Index: {}\n", tx_client.api_key_index());

    // Create a buy limit order
    let order_req = CreateOrderTxReq {
        market_index: 0,
        client_order_index: 1,
        base_amount: 1000000, // 1 unit (6 decimals)
        price: 100000000,     // Price with proper decimals
        is_ask: 0,            // 0 = buy, 1 = sell
        order_type: ORDER_TYPE_LIMIT,
        time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
        reduce_only: 0,
        trigger_price: 0,
        order_expiry: 0,
    };

    println!("Order Details:");
    println!("  Market Index: {}", order_req.market_index);
    println!(
        "  Side: {}",
        if order_req.is_ask == 0 { "BUY" } else { "SELL" }
    );
    println!("  Amount: {}", order_req.base_amount);
    println!("  Price: {}", order_req.price);
    println!("  Order Type: LIMIT");
    println!();

    // Create transaction options with manual nonce (since we're in offline mode)
    use lighter_rs::types::TransactOpts;
    let opts = TransactOpts {
        from_account_index: Some(tx_client.account_index()),
        api_key_index: Some(tx_client.api_key_index()),
        expired_at: 1000000000,
        nonce: Some(1),
        dry_run: false,
    };

    // Sign the transaction
    let tx = tx_client.create_order(&order_req, Some(opts)).await?;

    println!("✓ Transaction signed successfully");
    println!("  Transaction Type: {}", tx.get_tx_type());
    println!("  Account Index: {}", tx.account_index);
    println!("  Nonce: {}", tx.nonce);
    if let Some(hash) = tx.get_tx_hash() {
        println!("  Transaction Hash: {}", hash);
    }

    println!("\n✓ Example completed successfully!");

    Ok(())
}
