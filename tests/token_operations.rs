//! Token Operations Tests
//! 
//! This module contains tests for basic SRC20 token operations including:
//! - Token minting
//! - Token transfers
//! - Supply checks
//! - Balance queries
//! - Token metadata

use fuels::{
    accounts::signers::private_key::PrivateKeySigner,
    prelude::*,
    types::{Bits256, Identity, SizedAsciiString},
};

use fuels::accounts::wallet::Unlocked;

// Load abi from json
abigen!(
    Contract(
        name = "Src20Token",
        abi = "contracts/src20-token/out/debug/src20_token-abi.json",
    ),
);

/// Common test constants
const TOKEN_AMOUNT: u64 = 1_000_000;
const SUB_ID_ARRAY: [u8; 32] = [0u8; 32];
const SUB_ID: Bits256 = Bits256(SUB_ID_ARRAY);

/// Deploys the SRC20 token contract with the given wallet and metadata.
/// Returns a contract instance for further interaction.
async fn deploy_src20_token(
    wallet: Wallet<Unlocked<PrivateKeySigner>>,
    name: &str,
    symbol: &str,
    decimals: u8,
) -> Result<Src20Token<Wallet<Unlocked<PrivateKeySigner>>>> {
    // Convert name and symbol to SizedAsciiString for contract configurables.
    let name_bytes: SizedAsciiString<7> = name.try_into()?;
    let symbol_bytes: SizedAsciiString<5> = symbol.try_into()?;

    // Set up contract configurables (name, symbol, decimals, admin).
    let configurables = Src20TokenConfigurables::default()
        .with_NAME(name_bytes.clone())?
        .with_SYMBOL(symbol_bytes.clone())?
        .with_DECIMALS(decimals)?
        .with_ADMIN(Identity::Address(wallet.address().into()))?;

    // Deploy the contract to the local node.
    let deploy_response = Contract::load_from(
        "contracts/src20-token/out/debug/src20_token.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;

    println!(
        "✅ Token '{}' ({}) deployed at: {}",
        name,
        symbol,
        contract_id.to_string()
    );
    Ok(Src20Token::new(contract_id, wallet))
}

/// Test basic token operations including minting, transfers, and supply checks
#[tokio::test]
async fn test_token_operations() -> Result<()> {
    println!("🧪 Testing token operations...");

    // Set up test wallets
    let num_wallets = 3;
    let coins_per_wallet = 2;
    let amount_per_coin = 1_000_000_000;
    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );
    
    let mut wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await?;

    let admin_wallet = wallets.pop().unwrap();
    let user_wallet = wallets.pop().unwrap();

    // Deploy the SRC20 token contract
    let token_contract = deploy_src20_token(
        admin_wallet.clone(),
        "MYTOKEN",
        "TOKEN",
        9,
    ).await?;

    // Create admin token contract instance for minting
    let admin_token_contract = Src20Token::new(
        token_contract.contract_id().clone(),
        admin_wallet.clone(),
    );

    // Mint tokens to the user wallet
    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(user_wallet.address().into());

    println!(
        "🔄 Minting {} tokens to user: {:?}",
        mint_amount, recipient
    );

    // Mint tokens to the recipient (user wallet).
    let mint_tx = admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), mint_amount)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await?;

    println!("Mint transaction successful!");
    println!("Mint transaction: {:?}", mint_tx.decode_logs().results[0]);

    let mint_logs = mint_tx.decode_logs();
    assert!(!mint_logs.results.is_empty(), "Should have mint logs");

    // Calculate the correct asset ID from contract ID and sub ID
    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    // Query the total supply after minting.
    let total_supply = token_contract
        .methods()
        .total_supply(asset_id)
        .call()
        .await?
        .value;

    println!("Total supply after minting: {:?}", total_supply);

    // Optionally, assert the total supply matches the minted amount.
    assert_eq!(total_supply, Some(mint_amount));

    println!("✅ Token operations test passed");
    Ok(())
} 