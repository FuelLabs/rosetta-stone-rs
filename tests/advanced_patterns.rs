// Advanced Patterns Tests
// 
// This module contains tests for advanced blockchain patterns including:
// - Block manipulation
// - Gas optimization
// - Custom transaction policies
// - Performance benchmarks

use fuels::{
    accounts::signers::private_key::PrivateKeySigner,
    prelude::*,
    types::{AssetId, Bits256, ContractId, Identity, SizedAsciiString},
};

use fuels::accounts::wallet::Unlocked;

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

// Test advanced blockchain patterns
#[tokio::test]
async fn test_advanced_patterns() -> Result<()> {
    println!("🧪 Testing advanced patterns...");

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

    // Deploy contracts
    let token_contract = deploy_src20_token(
        admin_wallet.clone(),
        "ADVTOKE",
        "ADVOK",
        9,
    ).await?;

    let cross_contract_call_contract = deploy_cross_contract_call(
        admin_wallet.clone(),
    ).await?;

    let _vault_contract = deploy_token_vault(
        admin_wallet.clone(),
        cross_contract_call_contract.clone(),
    ).await?;

    // Test block manipulation
    let provider = admin_wallet.try_provider()?;
    let initial_height = provider.latest_block_height().await?;

    // Produce blocks
    provider.produce_blocks(5, None).await?;
    let new_height = provider.latest_block_height().await?;

    assert_eq!(new_height, initial_height + 5);
    println!("✅ Block manipulation test passed");

    // Test gas optimization
    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    let recipient = Identity::Address(admin_wallet.address().into());

    // Check wallet balance before transaction
    let base_balance = admin_wallet.get_asset_balance(&AssetId::BASE).await?;
    println!("base_balance: {:?}", base_balance);

    // Estimate gas cost
    let estimated_cost = admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), TOKEN_AMOUNT)
        .estimate_transaction_cost(None, None)
        .await?;

    // Ensure we have enough base assets
    if base_balance < estimated_cost.total_fee as u128 {
        println!("❌ Insufficient base assets for transaction");
        return Err("Insufficient base assets".into());
    }

    println!("⛽ Estimated gas cost: {:?}", estimated_cost);
    // Test with custom transaction policies
    let custom_policies = TxPolicies::default()
        .with_script_gas_limit(estimated_cost.total_gas * 2)
        .with_max_fee(estimated_cost.total_fee * 2);

    let txn_with_custom_policies = match admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), TOKEN_AMOUNT)
        .with_tx_policies(custom_policies)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
    {
        Ok(txn) => txn,
        Err(e) => {
            println!("❌ Mint with custom policies failed: {:?}", e);
            return Err(e.into());
        }
    };

    println!(
        "Mint transaction: {:?}",
        txn_with_custom_policies.decode_logs().results[0]
    );

    let txn_with_custom_policies_logs = txn_with_custom_policies.decode_logs();
    assert!(
        !txn_with_custom_policies_logs.results.is_empty(),
        "Should have mint logs"
    );

    let balances = admin_wallet.get_balances().await?;
    println!("balances: {:?}", balances);

    println!("✅ Advanced patterns test passed successfully");

    Ok(())
}

// Test comprehensive logging functionality
#[tokio::test]
async fn test_comprehensive_logging() -> Result<()> {
    println!("🧪 Testing comprehensive logging...");

    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(1), Some(1), Some(1_000_000)),
        None,
        None,
    )
    .await?;
    let wallet = wallets[0].clone();
    let token_contract = deploy_src20_token(wallet.clone(), "MYTOKEN", "TOKEN", 9).await?;

    // Test various operations with logging
    let recipient = Identity::Address(wallet.address().into());

    // Mint operation
    let mint_response = token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), 10000)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await?;

    // Decode different log types
    let all_logs = mint_response.decode_logs();
    println!("📝 Total logs: {}", all_logs.results.len());

    let asset_id = token_contract.methods().get_asset_id().call().await?.value;

    // Test burn operation
    let burn_amount = 5000;
    let call_params = CallParameters::default()
        .with_amount(burn_amount)
        .with_asset_id(asset_id);

    let burn_response = token_contract
        .methods()
        .burn(SUB_ID, burn_amount)
        .call_params(call_params)?
        .call()
        .await?;

    let burn_logs = burn_response.decode_logs();
    println!("🔥 Burn logs: {}", burn_logs.results.len());

    println!("✅ Comprehensive logging test passed");
    Ok(())
}

// Test performance benchmarks
#[tokio::test]
async fn test_performance_benchmarks() -> Result<()> {
    println!("🧪 Testing performance benchmarks...");

    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(1), Some(1), Some(1_000_000)),
        None,
        None,
    )
    .await?;
    let wallet = wallets[0].clone();
    let token_contract = deploy_src20_token(wallet.clone(), "MYTOKEN", "TOKEN", 9).await?;

    let admin_token_contract = token_contract.with_account(wallet.clone());
    // Benchmark batch operations
    let batch_size = 10;
    let start_time = std::time::Instant::now();

    for i in 0..batch_size {
        let recipient = Identity::Address(wallet.address().into());
        admin_token_contract
            .methods()
            .mint(recipient, Some(SUB_ID), 1000 * (i + 1) as u64)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await?;
    }

    let elapsed = start_time.elapsed();
    println!("⏱️  Batch of {} operations took: {:?}", batch_size, elapsed);

    // Verify final state
    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;
    let final_balance = wallet.get_asset_balance(&asset_id).await?;
    let expected_total: u64 = (1..=batch_size).sum::<u64>() * 1000;
    assert_eq!(final_balance, expected_total as u128);

    println!("✅ Performance benchmarks test passed");
    Ok(())
} 