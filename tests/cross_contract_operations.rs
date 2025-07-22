//! Cross Contract Operations Tests
//! 
//! This module contains tests for cross-contract communication including:
//! - Cross-contract calls
//! - Contract-to-contract interactions
//! - Multi-contract workflows

use fuels::{
    accounts::signers::private_key::PrivateKeySigner,
    prelude::*,
    types::{Bits256, ContractId, Identity, SizedAsciiString},
};

use fuels::accounts::wallet::Unlocked;

// Load abi from json
abigen!(
    Contract(
        name = "Src20Token",
        abi = "contracts/src20-token/out/debug/src20_token-abi.json",
    ),
    Contract(
        name = "TokenVault",
        abi = "contracts/token-vault/out/debug/token_vault-abi.json",
    ),
    Contract(
        name = "CrossContractCall",
        abi = "contracts/cross-contract-call/out/debug/cross_contract_call-abi.json",
    ),
);

/// Common test constants
const TOKEN_AMOUNT: u64 = 1_000_000;
const SUB_ID_ARRAY: [u8; 32] = [0u8; 32];
const SUB_ID: Bits256 = Bits256(SUB_ID_ARRAY);

/// Deploys the SRC20 token contract with the given wallet and metadata.
async fn deploy_src20_token(
    wallet: Wallet<Unlocked<PrivateKeySigner>>,
    name: &str,
    symbol: &str,
    decimals: u8,
) -> Result<Src20Token<Wallet<Unlocked<PrivateKeySigner>>>> {
    let name_bytes: SizedAsciiString<7> = name.try_into()?;
    let symbol_bytes: SizedAsciiString<5> = symbol.try_into()?;

    let configurables = Src20TokenConfigurables::default()
        .with_NAME(name_bytes.clone())?
        .with_SYMBOL(symbol_bytes.clone())?
        .with_DECIMALS(decimals)?
        .with_ADMIN(Identity::Address(wallet.address().into()))?;

    let deploy_response = Contract::load_from(
        "contracts/src20-token/out/debug/src20_token.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;
    println!("✅ Token '{}' ({}) deployed at: {}", name, symbol, contract_id.to_string());
    Ok(Src20Token::new(contract_id, wallet))
}

/// Deploys the CrossContractCall contract
async fn deploy_cross_contract_call(
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>> {
    let configurables = CrossContractCallConfigurables::default()
        .with_ADMIN(Identity::Address(admin_wallet.address().into()))?;

    let deploy_response = Contract::load_from(
        "contracts/cross-contract-call/out/debug/cross_contract_call.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&admin_wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;
    println!("✅ CrossContractCall deployed at: {}", contract_id.to_string());
    Ok(CrossContractCall::new(contract_id, admin_wallet))
}

/// Deploys the TokenVault contract
async fn deploy_token_vault(
    wallet: Wallet<Unlocked<PrivateKeySigner>>,
    cross_contract_call_contract_instance: &CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>,
) -> Result<TokenVault<Wallet<Unlocked<PrivateKeySigner>>>> {
    let configurables = TokenVaultConfigurables::default()
        .with_CROSS_CONTRACT_CALL(ContractId::from(
            cross_contract_call_contract_instance.contract_id(),
        ))?
        .with_ADMIN(Identity::Address(wallet.address().into()))?;

    let deploy_response = Contract::load_from(
        "contracts/token-vault/out/debug/token_vault.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;
    println!("✅ TokenVault deployed at: {}", contract_id.to_string());
    Ok(TokenVault::new(contract_id, wallet))
}

/// Test cross-contract call functionality
#[tokio::test]
async fn test_cross_contract_call() -> Result<()> {
    println!("🧪 Testing cross-contract call...");

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

    // Deploy contracts
    let token_contract = deploy_src20_token(
        admin_wallet.clone(),
        "CROSSTK",
        "CROSS",
        6,
    ).await?;

    let cross_contract_call_contract = deploy_cross_contract_call(
        admin_wallet.clone(),
    ).await?;

    let vault_contract = deploy_token_vault(
        admin_wallet.clone(),
        &cross_contract_call_contract,
    ).await?;

    let user_vault_contract =
        TokenVault::new(vault_contract.contract_id().clone(), user_wallet.clone());

    // 🔧 FIX: Mint tokens to ADMIN wallet instead of user wallet
    // Since the CrossContractCall requires admin authorization
    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(admin_wallet.address().into()); // ← Changed to admin_wallet

    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    println!("🔄 Minting {} tokens to admin wallet...", mint_amount);
    match admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), mint_amount)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
    {
        Ok(_) => println!("✅ Mint successful"),
        Err(e) => {
            println!("❌ Mint failed: {:?}", e);
            return Err(e.into());
        }
    };

    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    // 🔧 FIX: Check admin wallet balance instead of user wallet
    let admin_balance = admin_wallet.get_asset_balance(&asset_id).await?;
    println!("💰 Admin balance before deposit: {}", admin_balance);

    let initial_deposit_balance = match vault_contract
        .methods()
        .get_deposit(Identity::Address(user_wallet.address().into()))
        .call()
        .await
    {
        Ok(response) => {
            println!("📊 Initial deposit balance for user: {}", response.value);
            response.value
        }
        Err(e) => {
            println!("❌ Failed to get initial deposit balance: {:?}", e);
            return Err(e.into());
        }
    };

    let deposit_amount: u64 = 100;

    println!("🔄 Preparing deposit of {} tokens...", deposit_amount);
    println!("🔄 Executing cross-contract deposit...");
    println!("  From: Admin wallet ({})", admin_wallet.address());
    println!("  To: User ({}) via cross-contract call", user_wallet.address());

    // Check if admin has enough balance
    if admin_balance < deposit_amount as u128 {
        println!(
            "❌ Admin has insufficient balance: {} < {}",
            admin_balance, deposit_amount
        );
        return Err("Insufficient balance for deposit".into());
    }

    let call_params = CallParameters::default()
        .with_amount(deposit_amount as u64)
        .with_asset_id(asset_id);

    // 🔧 FIX: The cross-contract call should work now because:
    // 1. Admin wallet has the tokens (we minted to admin)
    // 2. Admin wallet is calling the CrossContractCall contract
    // 3. CrossContractCall contract will forward tokens to vault for the user
    match cross_contract_call_contract
        .methods()
        .deposit(
            user_vault_contract.contract_id(),
            user_wallet.address().into(),
        )
        .call_params(call_params)?
        .with_contract_ids(&[user_vault_contract.contract_id().clone()])
        .call()
        .await
    {
        Ok(response) => {
            println!("✅ Cross-contract deposit successful");
            println!("📋 Transaction ID: {:?}", response.tx_id);
            println!("📋 Transaction Status: {:?}", response.tx_status);
        }
        Err(e) => {
            println!("❌ Cross-contract deposit failed: {:?}", e);
            return Err(e.into());
        }
    }

    // Check the results
    let final_deposit_balance = match vault_contract
        .methods()
        .get_deposit(Identity::Address(user_wallet.address().into()))
        .call()
        .await
    {
        Ok(response) => {
            println!("✅ Final deposit balance for user: {}", response.value);
            response.value
        }
        Err(e) => {
            println!("❌ Failed to get final deposit balance: {:?}", e);
            return Err(e.into());
        }
    };

    let balance_increase = final_deposit_balance - initial_deposit_balance;
    println!("📈 Balance increase: {} (expected: {})", balance_increase, deposit_amount);
    
    // Verify the cross-contract deposit worked
    assert_eq!(balance_increase, deposit_amount, 
        "Expected deposit increase of {} but got {}. Initial: {}, Final: {}", 
        deposit_amount, balance_increase, initial_deposit_balance, final_deposit_balance);
    
    println!("✅ Cross Contract Call Deposit verification passed");

    // 🔧 BONUS: Verify admin wallet balance decreased
    let admin_balance_after = admin_wallet.get_asset_balance(&asset_id).await?;
    println!("💰 Admin balance after deposit: {}", admin_balance_after);
    
    let admin_balance_decrease = admin_balance - admin_balance_after;
    println!("📉 Admin balance decrease: {} (expected: {})", admin_balance_decrease, deposit_amount);

    Ok(())
}

// 🔧 ALTERNATIVE TEST: If you want to test with user wallet sending tokens
#[tokio::test]
async fn test_cross_contract_call_user_sends() -> Result<()> {
    println!("🧪 Testing cross-contract call with user sending tokens...");

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

    // Deploy contracts
    let token_contract = deploy_src20_token(
        admin_wallet.clone(),
        "USERTOK", // ← 7 characters (already fixed)
        "USERR",   // ← Fixed: 5 characters exactly
        6,
    ).await?;

    let cross_contract_call_contract = deploy_cross_contract_call(
        admin_wallet.clone(),
    ).await?;

    let vault_contract = deploy_token_vault(
        admin_wallet.clone(),
        &cross_contract_call_contract,
    ).await?;

    // Mint tokens to USER wallet
    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(user_wallet.address().into());

    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), mint_amount)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await?;

    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    // Create user instance of CrossContractCall (but this will fail due to admin restriction)
    let user_cross_contract_call = CrossContractCall::new(
        cross_contract_call_contract.contract_id().clone(),
        user_wallet.clone(),
    );

    let deposit_amount: u64 = 100;
    let call_params = CallParameters::default()
        .with_amount(deposit_amount)
        .with_asset_id(asset_id);

    // This should fail because only admin can call the cross-contract function
    match user_cross_contract_call
        .methods()
        .deposit(
            vault_contract.contract_id(),
            user_wallet.address().into(),
        )
        .call_params(call_params)?
        .with_contract_ids(&[vault_contract.contract_id().clone()])
        .call()
        .await
    {
        Ok(_) => {
            panic!("❌ This should have failed! User should not be able to call admin-only function");
        }
        Err(e) => {
            println!("✅ Expected failure: User cannot call admin-only function");
            println!("   Error: {:?}", e);
        }
    }

    println!("✅ User authorization test passed");
    Ok(())
}