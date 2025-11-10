//! # Lighter RS
//!
//! A comprehensive Rust SDK for the Lighter Protocol trading application blockchain.
//!
//! This library provides all necessary functionality for trading on Lighter, including:
//! - Cryptographic signing and key management
//! - Transaction construction and validation
//! - HTTP client for API interactions
//! - Type-safe transaction types for all supported operations
//!
//! ## Modules
//!
//! - `constants`: Core constants and limits used throughout the protocol
//! - `signer`: Cryptographic key management and signing functionality
//! - `types`: Transaction types and request builders
//! - `client`: HTTP client for API interactions
//! - `errors`: Error types and handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use lighter_rs::client::TxClient;
//! use lighter_rs::types::CreateOrderTxReq;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a transaction client
//! let tx_client = TxClient::new(
//!     "https://api.lighter.xyz",
//!     "your_api_key_hex",
//!     12345,  // account_index
//!     0,      // api_key_index
//!     1,      // chain_id
//! )?;
//!
//! // Create and submit an order
//! // let order = CreateOrderTxReq { ... };
//! // let result = tx_client.create_order(&order, None).await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod constants;
pub mod errors;
pub mod types;
pub mod utils;
pub mod ws_client;

// Re-export commonly used types
pub use client::TxResponse;
pub use constants::*;
pub use errors::{LighterError, Result};
pub use types::{TransactOpts, TxInfo};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
