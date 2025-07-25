// Vault Operations Tests
// 
// This module contains tests for the TokenVault contract operations including:
// - Vault deposits
// - Vault withdrawals
// - Vault balance checks
// - Admin operations

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

// Common test constants
const TOKEN_AMOUNT: u64 = 1_000_000;
const SUB_ID_ARRAY: [u8; 32] = [0u8; 32];
const SUB_ID: Bits256 = Bits256(SUB_ID_ARRAY);

// Deploys the SRC20 token contract with the given wallet and metadata
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

// Deploys the CrossContractCall contract
async fn deploy_cross_contract_call(
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>> {
    let deploy_response = Contract::load_from(
        "contracts/cross-contract-call/out/debug/cross_contract_call.bin",
        LoadConfiguration::default(),
    )?
    .deploy(&admin_wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;
    println!("✅ CrossContractCall deployed at: {}", contract_id.to_string());
    Ok(CrossContractCall::new(contract_id, admin_wallet))
}

// Deploys the TokenVault contract
async fn deploy_token_vault(
    wallet: Wallet<Unlocked<PrivateKeySigner>>,
    cross_contract_call_contract_instance: CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>,
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

// Test vault deposit and withdrawal functionality
#[tokio::test]
async fn test_vault_deposit() -> Result<()> {
    println!("🧪 Testing vault deposit...");

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
        "VAULTOK",
        "VAULT",
        6,
    ).await?;

    let cross_contract_call_contract = deploy_cross_contract_call(
        admin_wallet.clone(),
    ).await?;

    let vault_contract = deploy_token_vault(
        admin_wallet.clone(),
        cross_contract_call_contract.clone(),
    ).await?;

    // Mint tokens to the user wallet
    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(user_wallet.address().into());

    println!("🔄 Creating admin token contract instance...");
    let admin_token_contract = Src20Token::new(token_contract.contract_id().clone(), admin_wallet);

    println!("🔄 Minting {} tokens to user...", mint_amount);
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
    }

    // Check user balance after mint
    let asset_id = match admin_token_contract.methods().get_asset_id().call().await {
        Ok(response) => {
            println!("✅ Got asset ID: {:?}", response.value);
            response.value
        }
        Err(e) => {
            println!("❌ Failed to get asset ID: {:?}", e);
            return Err(e.into());
        }
    };

    let user_balance = user_wallet.get_asset_balance(&asset_id).await?;
    println!("💰 User balance before deposit: {}", user_balance);

    // Deposit tokens into the vault
    let deposit_amount = 100_000;

    println!("🔄 Preparing deposit of {} tokens...", deposit_amount);

    // Check if user has enough balance
    if user_balance < deposit_amount {
        println!(
            "❌ User has insufficient balance: {} < {}",
            user_balance, deposit_amount
        );
        return Err("Insufficient balance for deposit".into());
    }

    let call_params = CallParameters::default()
        .with_amount(deposit_amount as u64)
        .with_asset_id(asset_id);

    println!("🔄 Executing deposit with user wallet...");

    // Use user wallet for deposit, not admin wallet
    let user_vault_contract = vault_contract.clone().with_account(user_wallet.clone());

    match user_vault_contract
        .methods()
        .deposit()
        .call_params(call_params)?
        .call()
        .await
    {
        Ok(_) => println!("✅ Deposit successful"),
        Err(e) => {
            println!("❌ Deposit failed: {:?}", e);
            return Err(e.into());
        }
    }

    // Verify deposit
    println!("🔄 Verifying deposit...");
    let deposit_balance = match vault_contract
        .methods()
        .get_deposit(Identity::Address(user_wallet.address().into()))
        .call()
        .await
    {
        Ok(response) => {
            println!("✅ Got deposit balance: {}", response.value);
            response.value
        }
        Err(e) => {
            println!("❌ Failed to get deposit balance: {:?}", e);
            return Err(e.into());
        }
    };

    assert_eq!(deposit_balance, deposit_amount as u64);
    println!("✅ Deposit verification passed");

    // Test withdrawal
    let withdrawal_amount = 50_000;

    println!("🔄 Preparing withdrawal of {} tokens...", withdrawal_amount);

    let withdraw_call_params = CallParameters::default().with_asset_id(asset_id);

    let vault_contract_for_withdraw = vault_contract.clone().with_account(user_wallet.clone());

    match vault_contract_for_withdraw
        .methods()
        .withdraw(withdrawal_amount)
        .call_params(withdraw_call_params)?
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
    {
        Ok(_) => println!("✅ Withdrawal successful"),
        Err(e) => {
            println!("❌ Withdrawal failed: {:?}", e);
            return Err(e.into());
        }
    }

    // Verify withdrawal
    println!("🔄 Verifying withdrawal...");
    let remaining_deposit = match vault_contract
        .methods()
        .get_deposit(Identity::Address(user_wallet.address().into()))
        .call()
        .await
    {
        Ok(response) => {
            println!("✅ Got remaining deposit balance: {}", response.value);
            response.value
        }
        Err(e) => {
            println!("❌ Failed to get remaining deposit balance: {:?}", e);
            return Err(e.into());
        }
    };

    assert_eq!(
        remaining_deposit,
        deposit_amount as u64 - withdrawal_amount as u64
    );
    println!("✅ Withdrawal verification passed");

    // Check final user balance
    let final_user_balance = user_wallet.get_asset_balance(&asset_id).await?;
    println!("💰 User final balance: {}", final_user_balance);

    println!("✅ Vault deposit test passed");
    Ok(())
} 