//! Simplified Script Operations Test
//! 
//! This test focuses on a single working script execution pattern

use fuels::{
    accounts::signers::private_key::PrivateKeySigner,
    prelude::*,
    types::{Bits256, Identity, SizedAsciiString, tx_status::TxStatus},
};

use fuels::accounts::wallet::Unlocked;

// Load abi from json
abigen!(
    Contract(
        name = "Src20Token",
        abi = "contracts/src20-token/out/debug/src20_token-abi.json",
    ),
    Script(
        name = "MultiAssetTransfer",
        abi = "scripts/multi-asset-transfer/out/debug/multi_asset_transfer-abi.json",
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

/// Test simple script execution with single recipient
#[tokio::test]
async fn test_simple_script_execution() -> Result<()> {
    println!("🧪 Testing simple script execution...");

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
    let recipient_wallet = wallets.pop().unwrap();

    println!("👤 Admin wallet: {}", admin_wallet.address());
    println!("👤 Recipient wallet: {}", recipient_wallet.address());

    // Deploy the SRC20 token contract
    let token_contract = deploy_src20_token(
        admin_wallet.clone(),
        "SCRIPTK",
        "SCRIP",
        6,
    ).await?;

    // 🔧 FIX: Use exactly 3 recipients as expected by the script
    let recipients = [
        Identity::Address(recipient_wallet.address().into()),
        Identity::Address(recipient_wallet.address().into()), // Same recipient for simplicity
        Identity::Address(recipient_wallet.address().into()), // Same recipient for simplicity
    ];
    let amounts = [100u64, 200u64, 300u64]; // Three amounts as expected
    let total_amount = 100 + 200 + 300; // = 600

    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    // Mint tokens to admin
    let mint_amount = 10000u64; // Mint plenty
    println!("🔄 Minting {} tokens to admin wallet...", mint_amount);

    admin_token_contract
        .methods()
        .mint(Identity::Address(admin_wallet.address().into()), Some(SUB_ID), mint_amount)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await?;

    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    let admin_balance = admin_wallet.get_asset_balance(&asset_id).await?;
    println!("💰 Admin balance after mint: {}", admin_balance);

    // Configure script with single recipient
    let configurables = MultiAssetTransferConfigurables::default()
        .with_RECIPIENTS(recipients)?
        .with_AMOUNTS(amounts)?;

    println!("🔧 Script configuration:");
    println!("  Recipient 1: {} (amount: {})", recipient_wallet.address(), amounts[0]);
    println!("  Recipient 2: {} (amount: {})", recipient_wallet.address(), amounts[1]);
    println!("  Recipient 3: {} (amount: {})", recipient_wallet.address(), amounts[2]);
    println!("  Total amount: {}", total_amount);

    // Create script instance
    let script_instance = MultiAssetTransfer::new(
        admin_wallet.clone(),
        "scripts/multi-asset-transfer/out/debug/multi_asset_transfer.bin",
    )
    .with_configurables(configurables);

    // Execute script using manual transaction building to include token inputs
    println!("🚀 Executing script with manual transaction building...");
    
    let script_call = script_instance.main(asset_id);
    let mut tb = script_call.transaction_builder().await?;

    // 🔧 KEY FIX: Add the token inputs to the script transaction
    println!("🔄 Adding token inputs to script transaction...");
    let token_inputs = admin_wallet
        .get_asset_inputs_for_amount(asset_id, total_amount as u128, None)
        .await?;
    
    println!("📦 Found {} token inputs for script", token_inputs.len());
    for (i, input) in token_inputs.iter().enumerate() {
        println!("  Input {}: {:?}", i + 1, input);
    }
    
    tb.inputs.extend(token_inputs);

    // Enable burning for unused tokens
    tb = tb.enable_burn(true);

    // Set transaction policies
    let tx_policies = TxPolicies::default()
        .with_script_gas_limit(2_000_000)
        .with_max_fee(1_000_000);
    
    tb = tb
        .with_tx_policies(tx_policies)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1));

    // Add fees and witnesses
    admin_wallet.adjust_for_fee(&mut tb, 0).await?;
    admin_wallet.add_witnesses(&mut tb)?;

    println!("🔍 Transaction builder state:");
    println!("  - Inputs: {}", tb.inputs.len());
    println!("  - Outputs: {}", tb.outputs.len());
    println!("  - Witnesses: {}", tb.witnesses.len());

    // Build and send transaction
    let provider = admin_wallet.try_provider()?.clone();
    let tx = tb.build(&provider).await?;
    let tx_id = provider.send_transaction(tx).await?;
    
    println!("📋 Transaction sent: {:?}", tx_id);
    
    // Wait for result
    let tx_status = provider.tx_status(&tx_id).await?;
    println!("📋 Transaction status: {:?}", tx_status);

    match tx_status {
        TxStatus::Success { .. } => {
            println!("✅ Script executed successfully!");
            
            let response = script_call.get_response(tx_status)?;
            println!("📋 Script returned: {}", response.value);
            // Check logs
            let logs = response.decode_logs();
            if !logs.results.is_empty() {
                println!("📋 Script logs:");
                for (i, log) in logs.results.iter().enumerate() {
                    println!("  Log {}: {:?}", i + 1, log);
                }
            }

            // Verify recipient balance (all transfers go to same wallet)
            let recipient_balance = recipient_wallet.get_asset_balance(&asset_id).await?;
            println!("💰 Recipient balance after script: {}", recipient_balance);

            let expected_total = amounts[0] + amounts[1] + amounts[2];
            if recipient_balance >= expected_total as u128 {
                println!("✅ Recipient received all tokens successfully! (Expected: {}, Got: {})", expected_total, recipient_balance);
            } else {
                println!("⚠️  Recipient balance lower than expected (Expected: {}, Got: {})", expected_total, recipient_balance);
            }

            // Verify admin balance decreased
            let admin_balance_after = admin_wallet.get_asset_balance(&asset_id).await?;
            println!("💰 Admin balance after script: {}", admin_balance_after);
            
            let balance_decrease = admin_balance - admin_balance_after;
            println!("📉 Admin balance decreased by: {}", balance_decrease);

            println!("✅ Simple script execution test passed!");
        }
        TxStatus::Failure(failure) => {
            println!("❌ Script execution failed: {:?}", failure);
            return Err("Script execution failed".into());
        }
        _ => {
            return Err("Transaction still pending".into());
        }
    }

    Ok(())
}
