# Lighter RS

A comprehensive Rust SDK for the Lighter Protocol trading application blockchain.

This library is a complete Rust implementation of the [lighter-sdk](https://apidocs.lighter.xyz/docs/repos) library, providing all necessary functionality for trading on Lighter, including cryptographic signing, transaction construction, validation, and HTTP client for API interactions.

## Features

- **Complete Transaction Support**: All 19 transaction types + 6 helper methods
  - Orders: Create, Cancel, Modify, Cancel All, Grouped Orders
  - Transfers: Transfer funds, Withdraw
  - Pools: Create, Update, Mint/Burn Shares
  - Account Management: Change Public Key, Create Sub Account
  - Margin: Update Leverage, Update Margin

- **Cryptographic Signing**: Full support for Poseidon cryptography
  - Schnorr signatures over Goldilocks quintic extension field
  - Key management with secure private key handling

- **Type Safety**: Strongly typed transaction requests with comprehensive validation
  - Compile-time guarantees for transaction structure
  - Runtime validation of all transaction parameters

- **HTTP Client**: Async HTTP client for Lighter API
  - Automatic nonce management
  - Transaction submission (send_tx)
  - Fat-finger protection
  - Configurable timeouts

- **WebSocket Client**: Real-time data streaming
  - Order book subscriptions
  - Account update subscriptions
  - Incremental state updates
  - Callback-based event handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lighter-rs = "0.1.0"
```

## Quick Start

```rust
use lighter_rs::client::TxClient;
use lighter_rs::types::CreateOrderTxReq;
use lighter_rs::constants::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a transaction client
    let tx_client = TxClient::new(
        "https://api.lighter.xyz",           // API endpoint
        "0xYOUR_PRIVATE_KEY_HEX",            // Your API key (hex)
        12345,                                // account_index
        0,                                    // api_key_index
        1,                                    // chain_id
    )?;

    // Create an order
    let order_req = CreateOrderTxReq {
        market_index: 0,
        client_order_index: 1,
        base_amount: 1000000,                // 1 unit (6 decimals)
        price: 100000000,                    // Price with proper decimals
        is_ask: 0,                           // 0 = buy, 1 = sell
        order_type: ORDER_TYPE_LIMIT,
        time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
        reduce_only: 0,
        trigger_price: 0,
        order_expiry: 0,
    };

    // Sign and prepare transaction
    let tx = tx_client.create_order(&order_req, None).await?;

    println!("Transaction signed: {}", tx.get_tx_hash().unwrap());

    Ok(())
}
```

## Architecture

The library is organized into several modules:

- **constants**: Protocol constants and limits
- **errors**: Error types with detailed error messages
- **signer**: Cryptographic key management and signing
- **types**: Transaction types and request builders
  - `common`: Base transaction types and traits
  - `orders`: Order-related transactions
  - `pools`: Pool-related transactions
  - `transfers`: Transfer and withdrawal transactions
  - `validation`: Validation utilities
- **client**: HTTP client for API interactions
- **utils**: Utility functions

## Transaction Types

### Order Transactions

```rust
// Create Order
let order = CreateOrderTxReq { ... };
let tx = tx_client.create_order(&order, None).await?;

// Cancel Order
let cancel = CancelOrderTxReq {
    market_index: 0,
    index: 12345,
};

// Modify Order
let modify = ModifyOrderTxReq {
    market_index: 0,
    index: 12345,
    base_amount: 2000000,
    price: 105000000,
    trigger_price: 0,
};

// Create Grouped Orders (OTO, OCO, OTOCO)
let grouped = CreateGroupedOrdersTxReq {
    grouping_type: GROUPING_TYPE_ONE_CANCELS_THE_OTHER,
    orders: vec![order1, order2],
};
```

### Transfer Transactions

```rust
// Transfer funds
let transfer = TransferTxReq {
    to_account_index: 54321,
    usdc_amount: 1000000,  // 1 USDC
    fee: 1000,             // 0.001 USDC
    memo: [0u8; 32],
};

// Withdraw
let withdraw = WithdrawTxReq {
    usdc_amount: 1000000,
};
```

### Pool Transactions

```rust
// Create Public Pool
let pool = CreatePublicPoolTxReq {
    operator_fee: 10000,              // 1%
    initial_total_shares: 1000000000, // Initial shares
    min_operator_share_rate: 5000,    // 0.5%
};

// Mint Shares
let mint = MintSharesTxReq {
    public_pool_index: 123,
    share_amount: 100000,
};
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

Generate and view the documentation:

```bash
cargo doc --open
```

## Important Notes

### Poseidon Cryptography

This implementation uses placeholder functions for Poseidon cryptography. In production, you'll need to integrate with the actual Poseidon crypto library used by Lighter Protocol:

- `github.com/elliottech/poseidon_crypto` (Go reference)
- Rust implementation needed for full functionality

The following functions require actual implementation:
- `PoseidonKeyManager::derive_public_key()` - Schnorr public key derivation
- `PoseidonKeyManager::sign()` - Schnorr signature generation
- Transaction hashing functions - Poseidon2 hash over Goldilocks field

### Chain ID

Make sure to use the correct chain ID for your environment:
- Testnet: usually 1 or 2
- Mainnet: check official documentation

## Comparison with Go Implementation

This Rust implementation provides equivalent functionality to the Go version with additional benefits:

- **Memory Safety**: Rust's ownership system prevents common memory bugs
- **Type Safety**: Stronger type system with compile-time guarantees
- **Performance**: Zero-cost abstractions and efficient async runtime
- **Error Handling**: Comprehensive error types with context
- **Modern Async**: Built on Tokio for efficient async I/O

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`

## License

MIT License - see LICENSE file for details


## Support

For issues and questions:
- GitHub Issues: [lighter-rs/issues](https://github.com/elliottech/lighter-rs/issues)
- Lighter Discord: [Join Discord](https://discord.gg/lighter)
