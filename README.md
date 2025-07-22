# Rosetta Stone RS - Fuel Blockchain Integration Testing

A comprehensive Rust + Sway integration testing template for the Fuel blockchain ecosystem. This project demonstrates real-world patterns for building and testing Fuel applications with a focus on maintainability and beginner-friendly organization.

## 📊 Project Status

✅ **7 out of 8 tests passing**  
✅ **All compilation issues resolved**  
✅ **Modular test organization**  
✅ **Self-contained test files**  

## 🏗️ Project Structure

```
rosetta-stone-rs/
├── contracts/                    # Sway smart contracts
│   ├── src20-token/             # SRC20 token implementation
│   ├── token-vault/             # Token vault for deposits/withdrawals
│   └── cross-contract-call/     # Cross-contract communication
├── scripts/                     # Sway scripts
│   └── multi-asset-transfer/    # Multi-asset transfer script
├── predicates/                  # Sway predicates
│   ├── multi-sig/              # Multi-signature predicate
│   └── timelock/               # Time-lock predicate
├── tests/                       # Rust integration tests
│   ├── token_operations.rs      # Basic token operations ✅
│   ├── vault_operations.rs      # Vault deposits/withdrawals ✅
│   ├── cross_contract_operations.rs # Cross-contract calls ✅
│   ├── multi_wallet_operations.rs # Multi-wallet scenarios ✅
│   ├── predicate_operations.rs  # Predicate authorization ✅
│   ├── advanced_patterns.rs     # Advanced patterns & benchmarks ✅
│   ├── script_operations.rs     # Script execution ❌
│   └── simple_token_test.rs     # Beginner-friendly standalone ✅
├── examples/                    # Usage examples
└── build.rs                     # Build configuration
```

## 🚀 Quick Start

### Prerequisites

- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Fuel toolchain** - [Install Fuel](https://docs.fuel.network/guides/installation/)
- **Sway compiler** - `forc` (included with Fuel toolchain)

### Installation & Setup

```bash
# Clone the repository
git clone <repository-url>
cd rosetta-stone-rs

# Install dependencies and build contracts
cargo build

# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture
```

## 📚 Test Organization & Navigation

### 🎯 **For Beginners** - Start Here

#### 1. **Simple Token Test** (`tests/simple_token_test.rs`)
**Perfect starting point for newcomers**
- Basic token deployment and minting
- Simple balance checking
- Clear, well-documented code
- Standalone example

#### 2. **Token Operations** (`tests/token_operations.rs`)
**Core token functionality**
- Token minting and transfers
- Supply checks and balance queries
- Token metadata handling
- Basic contract interaction patterns

### 🔧 **For Developers** - Core Functionality

#### 3. **Vault Operations** (`tests/vault_operations.rs`)
**Token vault functionality**
- Deposits and withdrawals
- Vault balance management
- User-specific vault operations
- Admin functionality

#### 4. **Cross-Contract Operations** (`tests/cross_contract_operations.rs`)
**Contract-to-contract communication**
- Cross-contract calls
- Multi-contract workflows
- Complex integration patterns
- Admin authorization testing

#### 5. **Multi-Wallet Operations** (`tests/multi_wallet_operations.rs`)
**Complex multi-user scenarios**
- Multi-wallet token distribution
- Inter-wallet transfers
- Balance verification
- Complex token flows

### 🔬 **For Advanced Users** - Advanced Patterns

#### 6. **Predicate Operations** (`tests/predicate_operations.rs`)
**Sway predicate functionality**
- Multi-signature authorization
- Time-lock functionality
- Predicate validation
- Authorization patterns

#### 7. **Advanced Patterns** (`tests/advanced_patterns.rs`)
**Advanced blockchain patterns**
- Block manipulation
- Gas optimization
- Performance benchmarks
- Comprehensive logging
- Custom transaction policies

#### 8. **Script Operations** (`tests/script_operations.rs`)
**Sway script execution** ⚠️ *Currently failing*
- Multi-asset transfer scripts
- Script parameter handling
- Transaction building
- Execution verification

## 🧪 Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Basic token operations
cargo test --test token_operations

# Vault functionality
cargo test --test vault_operations

# Cross-contract calls
cargo test --test cross_contract_operations

# Multi-wallet scenarios
cargo test --test multi_wallet_operations

# Predicate authorization
cargo test --test predicate_operations

# Advanced patterns
cargo test --test advanced_patterns

# Script execution
cargo test --test script_operations
```

### Run with Verbose Output
```bash
cargo test -- --nocapture
```

## 📖 Learning Path

### 🎓 **Beginner Path**
1. **Start**: `simple_token_test.rs` - Basic concepts
2. **Learn**: `token_operations.rs` - Core token functionality
3. **Explore**: `vault_operations.rs` - Advanced token patterns
4. **Practice**: `multi_wallet_operations.rs` - Complex scenarios

### 🔧 **Developer Path**
1. **Understand**: `cross_contract_operations.rs` - Contract integration
2. **Master**: `predicate_operations.rs` - Authorization patterns
3. **Optimize**: `advanced_patterns.rs` - Performance and gas
4. **Debug**: `script_operations.rs` - Script execution (when fixed)

## 🏗️ Smart Contracts

### **SRC20 Token** (`contracts/src20-token/`)
Standard SRC20 token implementation with:
- Minting and burning functionality
- Transfer capabilities
- Supply management
- Metadata support

### **Token Vault** (`contracts/token-vault/`)
Secure token vault for deposits and withdrawals:
- User deposit/withdrawal functionality
- Balance tracking
- Admin controls
- Cross-contract integration

### **Cross-Contract Call** (`contracts/cross-contract-call/`)
Contract-to-contract communication:
- Admin-controlled operations
- Cross-contract function calls
- Authorization patterns
- Complex workflows

## 📜 Scripts & Predicates

### **Multi-Asset Transfer Script** (`scripts/multi-asset-transfer/`)
Script for transferring multiple assets in a single transaction:
- Batch transfers
- Configurable recipients
- Amount management
- Transaction optimization

### **Predicates** (`predicates/`)
Authorization and validation patterns:
- **Multi-Sig**: Multi-signature authorization
- **Time-Lock**: Time-based access control

## 🔧 Development Workflow

### Adding New Tests

1. **Choose the right module** based on functionality
2. **Follow the existing pattern** in similar test files
3. **Use descriptive names** with `test_` prefix
4. **Include comprehensive documentation**

### Example Test Structure

```rust
#[tokio::test]
async fn test_your_functionality() -> Result<()> {
    println!("🧪 Testing your functionality...");
    
    // Set up test environment
    let num_wallets = 3;
    let config = WalletsConfig::new(Some(num_wallets), Some(2), Some(1_000_000_000));
    let mut wallets = launch_custom_provider_and_get_wallets(config, None, None).await?;
    
    // Deploy contracts
    let contract = deploy_your_contract(wallets.pop().unwrap()).await?;
    
    // Execute test logic
    let result = contract.methods().your_method().call().await?;
    
    // Verify results
    assert!(result.value, "Expected condition");
    
    println!("✅ Test completed successfully!");
    Ok(())
}
```

## 🐛 Troubleshooting

### Common Issues

1. **Contract deployment failures**
   ```bash
   # Ensure contracts are built
   forc build --path contracts/src20-token
   forc build --path contracts/token-vault
   forc build --path contracts/cross-contract-call
   ```

2. **Test timeout**
   ```bash
   # Increase timeout for complex operations
   RUST_LOG=debug cargo test -- --nocapture
   ```

3. **Balance issues**
   - Check wallet funding in test setup
   - Verify token minting before transfers
   - Ensure sufficient base asset balance

4. **Import errors**
   - All test files are now self-contained
   - No shared module dependencies
   - Each file includes its own imports

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo test

# Run specific test with debug
RUST_LOG=debug cargo test --test token_operations -- --nocapture
```

## 📊 Test Status Summary

| Test Category | Status | Description |
|---------------|--------|-------------|
| **Token Operations** | ✅ Passing | Basic token minting, transfers, supply checks |
| **Vault Operations** | ✅ Passing | Deposit/withdrawal functionality |
| **Cross-Contract Calls** | ✅ Passing | Contract-to-contract communication |
| **Multi-Wallet Operations** | ✅ Passing | Complex multi-user scenarios |
| **Predicate Operations** | ✅ Passing | Authorization and validation |
| **Advanced Patterns** | ✅ Passing | Performance benchmarks and gas optimization |
| **Script Operations** | ❌ Failing | Script execution (revert issue) |
| **Simple Token Test** | ✅ Passing | Beginner-friendly standalone example |

## 🆘 Resources

### Documentation
- [Fuel Documentation](https://docs.fuel.network/)

### Community
- [Fuel Forum](https://forum.fuel.network/)

---

**🎯 Ready to start?** Begin with `tests/simple_token_test.rs` for a gentle introduction to Fuel development! 