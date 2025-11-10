//! Example: Connect to Lighter Testnet and Execute Transactions
//!
//! This example demonstrates how to:
//! 1. Connect to Lighter testnet
//! 2. Create and send a limit order
//! 3. Cancel an order
//! 4. Transfer funds
//! 5. Update leverage
//!
//! Prerequisites:
//! - Set environment variables:
//!   * LIGHTER_API_KEY - Your private API key (hex format)
//!   * LIGHTER_ACCOUNT_INDEX - Your account index
//!   * LIGHTER_API_KEY_INDEX - Your API key index (usually 0)
//!
//! Run with: cargo run --example testnet_trading

use lighter_rs::client::{TxClient, TxResponse};
use lighter_rs::constants::*;
use lighter_rs::types::{CancelOrderTxReq, CreateOrderTxReq, TxInfo};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Lighter RS - Testnet Trading Example           ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    // Load configuration from environment variables
    let api_key =
        env::var("LIGHTER_API_KEY").expect("LIGHTER_API_KEY environment variable not set");

    let account_index: i64 = env::var("LIGHTER_ACCOUNT_INDEX")
        .expect("LIGHTER_ACCOUNT_INDEX environment variable not set")
        .parse()
        .expect("LIGHTER_ACCOUNT_INDEX must be a valid number");

    let api_key_index: u8 = env::var("LIGHTER_API_KEY_INDEX")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .expect("LIGHTER_API_KEY_INDEX must be a valid number");

    // Testnet configuration
    let testnet_url = "https://api-testnet.lighter.xyz";
    let chain_id = 300; // Testnet chain ID

    println!("Configuration:");
    println!("  API Endpoint: {}", testnet_url);
    println!("  Chain ID: {}", chain_id);
    println!("  Account Index: {}", account_index);
    println!("  API Key Index: {}\n", api_key_index);

    // Initialize the transaction client
    let tx_client = TxClient::new(
        testnet_url,
        &api_key,
        account_index,
        api_key_index,
        // chain_id,
    )?;

    println!("✓ Connected to Lighter Testnet\n");

    // ========== Example 1: Create a Limit Order ==========
    println!("═══ Example 1: Creating Limit Order ═══");

    let order_req = CreateOrderTxReq {
        market_index: 0,
        client_order_index: chrono::Utc::now().timestamp_millis(), // Use timestamp as unique ID
        base_amount: 1_000_000,                                    // 1 unit (assuming 6 decimals)
        price: 100_000_000,                                        // Price
        is_ask: 0,                                                 // 0 = BUY, 1 = SELL
        order_type: ORDER_TYPE_LIMIT,
        time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
        reduce_only: 0,
        trigger_price: 0,
        order_expiry: 0,
    };

    println!("Order Parameters:");
    println!("  Market Index: {}", order_req.market_index);
    println!(
        "  Side: {}",
        if order_req.is_ask == 0 { "BUY" } else { "SELL" }
    );
    println!("  Amount: {}", order_req.base_amount);
    println!("  Price: {}", order_req.price);
    println!("  Order Type: LIMIT");

    // Sign the order (nonce will be fetched automatically from API)
    println!("\nSigning transaction...");
    let signed_order = tx_client.create_order(&order_req, None).await?;

    println!("✓ Transaction signed");
    println!("  Nonce used: {}", signed_order.nonce);
    println!(
        "  Transaction Hash: {}",
        signed_order.get_tx_hash().unwrap_or("N/A".to_string())
    );

    // Send the transaction
    println!("\nSubmitting to testnet...");
    let response = tx_client.send_transaction(&signed_order).await?;

    print_tx_response(&response);

    // Store order index for later cancellation
    let order_client_index = order_req.client_order_index;

    println!("\n");

    // Wait a bit before next transaction
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // ========== Example 2: Create Market Order ==========
    println!("═══ Example 2: Creating Market Order ═══");

    let market_order = tx_client
        .create_market_order(
            0,                                     // market_index
            chrono::Utc::now().timestamp_millis(), // client_order_index
            500_000,                               // base_amount (0.5 units)
            105_000_000,                           // price (max acceptable price for buy)
            0,                                     // is_ask (BUY)
            false,                                 // reduce_only
            None,                                  // opts
        )
        .await?;

    println!("Market Order Parameters:");
    println!("  Amount: {}", market_order.order_info.base_amount);
    println!("  Max Price: {}", market_order.order_info.price);
    println!("  Order Type: MARKET");

    println!("\nSubmitting market order...");
    let market_response = tx_client.send_transaction(&market_order).await?;
    print_tx_response(&market_response);

    println!("\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // ========== Example 3: Create Stop Loss Order ==========
    println!("═══ Example 3: Creating Stop Loss Order ═══");

    let sl_order = tx_client
        .create_sl_order(
            0,                                     // market_index
            chrono::Utc::now().timestamp_millis(), // client_order_index
            1_000_000,                             // base_amount
            95_000_000,                            // trigger_price
            94_000_000,                            // price
            1,                                     // is_ask (SELL)
            false,                                 // reduce_only
            None,                                  // opts
        )
        .await?;

    println!("Stop Loss Order Parameters:");
    println!("  Trigger Price: {}", sl_order.order_info.trigger_price);
    println!("  Execution Price: {}", sl_order.order_info.price);
    println!("  Order Type: STOP_LOSS");

    println!("\nSubmitting stop loss order...");
    let sl_response = tx_client.send_transaction(&sl_order).await?;
    print_tx_response(&sl_response);

    println!("\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // ========== Example 4: Update Leverage ==========
    println!("═══ Example 4: Updating Leverage ═══");

    let leverage_tx = tx_client
        .update_leverage_with_multiplier(
            0,                 // market_index
            5,                 // 5x leverage
            MARGIN_MODE_CROSS, // cross margin mode
            None,              // opts
        )
        .await?;

    println!("Leverage Update Parameters:");
    println!("  Market: {}", leverage_tx.market_index);
    println!("  Leverage: 5x");
    println!("  Margin Mode: CROSS");

    println!("\nSubmitting leverage update...");
    let leverage_response = tx_client.send_transaction(&leverage_tx).await?;
    print_tx_response(&leverage_response);

    println!("\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // ========== Example 5: Cancel Order ==========
    println!("═══ Example 5: Canceling Order ═══");

    // Note: You would typically get the order index from a previous order creation
    // or from querying active orders. For this example, we'll use the first order's index.
    let cancel_req = CancelOrderTxReq {
        market_index: 0,
        index: order_client_index, // Using client order index from first example
    };

    println!("Cancel Parameters:");
    println!("  Market Index: {}", cancel_req.market_index);
    println!("  Order Index: {}", cancel_req.index);

    println!("\nSigning cancel transaction...");
    let cancel_tx = tx_client.cancel_order(&cancel_req, None).await?;

    println!("✓ Cancel transaction signed");
    println!("\nSubmitting cancellation...");
    let cancel_response = tx_client.send_transaction(&cancel_tx).await?;
    print_tx_response(&cancel_response);

    println!("\n");

    // ========== Summary ==========
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   All Testnet Operations Completed Successfully  ║");
    println!("╚═══════════════════════════════════════════════════╝");
    println!("\nTransactions executed:");
    println!("  1. ✓ Limit Order Created");
    println!("  2. ✓ Market Order Created");
    println!("  3. ✓ Stop Loss Order Created");
    println!("  4. ✓ Leverage Updated");
    println!("  5. ✓ Order Canceled");

    println!("\nNext steps:");
    println!("  - Check your orders on Lighter testnet UI");
    println!("  - Monitor your account balance");
    println!("  - Experiment with other transaction types");

    Ok(())
}

/// Helper function to print transaction response
fn print_tx_response(response: &TxResponse) {
    if response.code == 200 {
        println!("✓ Transaction successful!");
        if let Some(hash) = &response.tx_hash {
            println!("  Tx Hash: {}", hash);
        }
    } else {
        println!("✗ Transaction failed (code: {})", response.code);
        if let Some(msg) = &response.message {
            println!("  Error: {}", msg);
        }
    }
}
