# Lighter RS Examples

This directory contains practical examples demonstrating how to use the Lighter RS SDK.

## Available Examples

### Basic Examples (Offline Mode)

These examples demonstrate transaction signing without actually submitting to the API:

1. **create_order.rs** - Basic limit order creation and signing
   ```bash
   cargo run --example create_order
   ```

2. **transfer_funds.rs** - Transferring funds between accounts
   ```bash
   cargo run --example transfer_funds
   ```

3. **pool_operations.rs** - Pool creation, minting, and burning shares
   ```bash
   cargo run --example pool_operations
   ```

4. **advanced_orders.rs** - Cancel, modify, and grouped orders
   ```bash
   cargo run --example advanced_orders
   ```

### Testnet Examples (Live Trading)

#### testnet_trading.rs - Complete testnet integration example

This example demonstrates real trading on Lighter testnet including:
- Creating limit orders
- Creating market orders
- Creating stop-loss orders
- Updating leverage
- Canceling orders

#### Setup

1. **Get Testnet Credentials**:
   - Visit [Lighter Testnet](https://testnet.lighter.xyz)
   - Create an account and get your API credentials
   - Note your account index and API key

2. **Set Environment Variables**:
   ```bash
   export LIGHTER_API_KEY="0xYourPrivateKeyHere"
   export LIGHTER_ACCOUNT_INDEX="12345"
   export LIGHTER_API_KEY_INDEX="0"
   ```

3. **Run the Example**:
   ```bash
   cargo run --example testnet_trading
   ```

#### WebSocket Examples (Real-time Data)

**websocket_orderbook.rs** - Real-time order book monitoring
```bash
cargo run --example websocket_orderbook
```

**websocket_account.rs** - Real-time account monitoring
```bash
export LIGHTER_ACCOUNT_INDEX="12345"
cargo run --example websocket_account
```

**websocket_combined.rs** - Combined order book and account monitoring
```bash
export LIGHTER_ACCOUNT_INDEX="12345"
cargo run --example websocket_combined
```

**trading_bot_simple.rs** - Simple trading bot combining WebSocket + API
```bash
export LIGHTER_API_KEY="0xYourKey"
export LIGHTER_ACCOUNT_INDEX="12345"
cargo run --example trading_bot_simple
```

#### Expected Output

```
╔═══════════════════════════════════════════════════╗
║   Lighter RS - Testnet Trading Example           ║
╚═══════════════════════════════════════════════════╝

Configuration:
  API Endpoint: https://api-testnet.lighter.xyz
  Chain ID: 300
  Account Index: 12345
  API Key Index: 0

✓ Connected to Lighter Testnet

═══ Example 1: Creating Limit Order ═══
Order Parameters:
  Market Index: 0
  Side: BUY
  Amount: 1000000
  Price: 100000000
  Order Type: LIMIT

Signing transaction...
✓ Transaction signed
  Nonce used: 42
  Transaction Hash: 0x...

Submitting to testnet...
✓ Transaction successful!
  Tx Hash: 0x...

[Additional examples continue...]
```

## Example Features Demonstrated

### Transaction Types
- ✅ Limit Orders
- ✅ Market Orders
- ✅ Stop Loss Orders
- ✅ Take Profit Orders
- ✅ Order Cancellation
- ✅ Leverage Updates
- ✅ Fund Transfers
- ✅ Pool Operations

### SDK Features
- ✅ Automatic nonce management (fetched from API)
- ✅ Transaction signing with Poseidon cryptography
- ✅ API submission with error handling
- ✅ Helper methods for common operations
- ✅ Type-safe transaction construction
- ✅ Comprehensive validation

## API Endpoints

### Testnet
- **URL**: `https://api-testnet.lighter.xyz`
- **Chain ID**: 300
- **Explorer**: https://testnet.lighter.xyz

### Mainnet
- **URL**: `https://api.lighter.xyz`
- **Chain ID**: 304
- **Explorer**: https://lighter.xyz

## Important Notes

1. **Private Keys**: Never commit your private keys to version control. Always use environment variables or secure key management.

2. **Testnet vs Mainnet**:
   - Testnet is for testing and development
   - Always test thoroughly on testnet before using mainnet
   - Mainnet involves real funds

3. **Nonce Management**:
   - The SDK automatically fetches nonces from the API when HTTPClient is configured
   - For offline mode (no API), you must provide nonces manually in TransactOpts

4. **Error Handling**:
   - All examples use proper Rust error handling with `Result<T>`
   - Check transaction responses for success codes
   - Handle network errors gracefully

## Troubleshooting

### "nonce was not provided and HTTPClient is not available"
- Solution: Either provide an API URL when creating TxClient, or manually specify nonce in TransactOpts

### "Failed to get nonce" or API errors
- Check your internet connection
- Verify the API endpoint is correct
- Ensure your account exists on the network
- Check that your API key is valid

### Compilation errors
- Ensure you're using Rust 2021 edition or later
- Run `cargo update` to get latest dependencies
- Check that all dependencies are properly installed

## Next Steps

After running the examples:

1. **Explore the API**: Check [Lighter API Docs](https://apidocs.lighter.xyz)
2. **Build your app**: Use these examples as templates
3. **Join the community**: [Lighter Discord](https://discord.gg/lighter)
4. **Read the docs**: See the main README.md for detailed documentation

## Contributing

Found a bug or have a suggestion? Please open an issue or submit a PR!
