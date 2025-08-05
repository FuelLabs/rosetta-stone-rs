# Rosetta Stone RS - Fuel Blockchain Integration Testing

A comprehensive Rust + Sway integration testing template for the Fuel blockchain ecosystem. This project demonstrates real-world patterns for building and testing Fuel applications with a focus on maintainability and beginner-friendly organization.

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
│   ├── token_operations.rs      # Basic token operations
│   ├── vault_operations.rs      # Vault deposits/withdrawals
│   ├── cross_contract_operations.rs # Cross-contract calls
│   ├── multi_wallet_operations.rs # Multi-wallet scenarios
│   ├── predicate_operations.rs  # Predicate authorization
│   ├── advanced_patterns.rs     # Advanced patterns & benchmarks
│   ├── script_operations.rs     # Script execution
│   └── simple_token_test.rs     # Beginner-friendly standalone
└── build.rs                     # Build configuration
```

## 🚀 Getting Started

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

### Navigating the Project
- **contracts/**: Sway smart contracts (SRC20 token, token vault, cross-contract call)
- **scripts/**: Sway scripts (multi-asset transfer)
- **predicates/**: Sway predicates (multi-sig, timelock)
- **tests/**: Rust integration tests, each file is self-contained and tests a specific functionality:
  - `token_operations.rs`: Basic token operations
  - `vault_operations.rs`: Vault deposits/withdrawals
  - `cross_contract_operations.rs`: Cross-contract calls
  - `multi_wallet_operations.rs`: Multi-wallet scenarios
  - `predicate_operations.rs`: Predicate authorization
  - `advanced_patterns.rs`: Advanced patterns & benchmarks
  - `script_operations.rs`: Script execution (currently failing)
  - `simple_token_test.rs`: Standalone, beginner-friendly example
- **examples/**: Usage examples
- **build.rs**: Build configuration

### Running Specific Tests
```bash
cargo test --test token_operations
cargo test --test vault_operations
cargo test --test cross_contract_operations
cargo test --test multi_wallet_operations
cargo test --test predicate_operations
cargo test --test advanced_patterns
cargo test --test script_operations
```

## Troubleshooting
- **Contract deployment failures**: Ensure contracts are built with `forc build`
- **Test timeout**: Use `RUST_LOG=debug cargo test -- --nocapture`
- **Balance issues**: Check wallet funding and token minting in test setup

## Resources
- [Fuel Documentation](https://docs.fuel.network/)
- [Fuel Forum](https://forum.fuel.network/)
- Check test examples and `project_summary.md` for project status 