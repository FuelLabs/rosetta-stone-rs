//! Simple Token Test
//! 
//! This is a basic test to verify the project structure and demonstrate
//! how to write tests for beginners.

use fuels::{
    prelude::*,
    types::{Bits256, Identity, SizedAsciiString},
};


// Load abi from json
abigen!(
    Contract(
        name = "Src20Token",
        abi = "contracts/src20-token/out/debug/src20_token-abi.json",
    ),
);

const TOKEN_AMOUNT: u64 = 1_000_000;
const SUB_ID_ARRAY: [u8; 32] = [0u8; 32];
const SUB_ID: Bits256 = Bits256(SUB_ID_ARRAY);

/// Simple test to verify basic token functionality
#[tokio::test]
async fn test_simple_token_operations() -> Result<()> {
    println!("🧪 Testing simple token operations...");

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
    let name_bytes: SizedAsciiString<7> = "MYTOKEN".try_into()?;
    let symbol_bytes: SizedAsciiString<5> = "TOKEN".try_into()?;

    let configurables = Src20TokenConfigurables::default()
        .with_NAME(name_bytes.clone())?
        .with_SYMBOL(symbol_bytes.clone())?
        .with_DECIMALS(9)?
        .with_ADMIN(Identity::Address(admin_wallet.address().into()))?;

    let deploy_response = Contract::load_from(
        "contracts/src20-token/out/debug/src20_token.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&admin_wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;
    println!("✅ Token deployed at: {}", contract_id.to_string());

    // Create contract instances
    let admin_token_contract = Src20Token::new(contract_id, admin_wallet.clone());
    let user_token_contract = Src20Token::new(contract_id, user_wallet.clone());

    // Get the asset ID for this token
    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    println!("📊 Asset ID: {:?}", asset_id);

    // Test 1: Check initial supply
    println!("📊 Checking initial token supply...");
    let initial_supply = user_token_contract
        .methods()
        .total_supply(asset_id)
        .call()
        .await?
        .value;
    
    println!("   Initial supply: {:?}", initial_supply);
    assert_eq!(initial_supply, Some(0), "Initial supply should be 0");

    // Test 2: Mint tokens
    println!("🪙 Minting tokens to admin...");
    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(admin_wallet.address().into());
    
    admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), mint_amount)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await?;

    println!("   Minted {} tokens to admin", mint_amount);

    // Test 3: Check admin balance
    println!("💰 Checking admin balance...");
    let admin_balance = admin_wallet.get_asset_balance(&asset_id).await?;

    println!("   Admin balance: {}", admin_balance);
    assert_eq!(admin_balance, mint_amount as u128, "Admin balance should match minted amount");

    // Test 4: Check total supply after minting
    println!("📊 Checking total supply after minting...");
    let total_supply = user_token_contract
        .methods()
        .total_supply(asset_id)
        .call()
        .await?
        .value;

    println!("   Total supply: {:?}", total_supply);
    assert_eq!(total_supply, Some(mint_amount), "Total supply should match minted amount");

    // Test 5: Check token metadata
    println!("📋 Checking token metadata...");
    let name = user_token_contract
        .methods()
        .name(asset_id)
        .call()
        .await?
        .value;
    let symbol = user_token_contract
        .methods()
        .symbol(asset_id)
        .call()
        .await?
        .value;

    println!("   Token name: {:?}", name);
    println!("   Token symbol: {:?}", symbol);

    assert_eq!(name, Some("MYTOKEN".to_string()), "Token name should match");
    assert_eq!(symbol, Some("TOKEN".to_string()), "Token symbol should match");

    println!("✅ Simple token operations test completed successfully!");
    Ok(())
}

/// Test token minting with different amounts
#[tokio::test]
async fn test_token_minting_scenarios() -> Result<()> {
    println!("🧪 Testing token minting scenarios...");

    // Set up test wallets
    let num_wallets = 2;
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

    // Deploy the SRC20 token contract
    let name_bytes: SizedAsciiString<7> = "TESTTOK".try_into()?;
    let symbol_bytes: SizedAsciiString<5> = "TESTT".try_into()?;

    let configurables = Src20TokenConfigurables::default()
        .with_NAME(name_bytes.clone())?
        .with_SYMBOL(symbol_bytes.clone())?
        .with_DECIMALS(6)?
        .with_ADMIN(Identity::Address(admin_wallet.address().into()))?;

    let deploy_response = Contract::load_from(
        "contracts/src20-token/out/debug/src20_token.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&admin_wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;
    println!("✅ Token deployed at: {}", contract_id.to_string());

    let admin_token_contract = Src20Token::new(contract_id, admin_wallet.clone());

    // Get the asset ID for this token
    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    // Test minting different amounts
    let mint_amounts = vec![1000, 10000, 100000, 1000000];
    
    for amount in mint_amounts {
        println!("🪙 Minting {} tokens...", amount);
        
        let recipient = Identity::Address(admin_wallet.address().into());
        
        admin_token_contract
            .methods()
            .mint(recipient, Some(SUB_ID), amount)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await?;
        
        let balance = admin_wallet.get_asset_balance(&asset_id).await?;
            
        println!("   Admin balance after minting {}: {}", amount, balance);
    }

    println!("✅ Token minting scenarios test completed successfully!");
    Ok(())
} 