// Predicate Operations Tests
// 
// This module contains tests for predicate authorization including:
// - Multi-signature predicates
// - Predicate funding
// - Predicate balance checks
// - Authorization workflows

use fuels::{
    prelude::*,
    types::transaction_builders::ScriptTransactionBuilder,
};

abigen!(Predicate(
    name = "MultiSigPredicate",
    abi = "predicates/multi-sig/out/debug/multi_sig_predicate-abi.json",
));

// Test predicate authorization functionality
#[tokio::test]
async fn test_predicate_authorization() -> Result<()> {
    println!("Testing predicate authorization...");

    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(3), Some(1), Some(1_000_000)),
        None,
        None,
    )
    .await?;

    let provider = wallets[0].provider().clone();

    let signer1 = &wallets[0];
    let signer2 = &wallets[1];
    let signer3 = &wallets[2];

    // Configure predicate
    let signers = [
        signer1.address().into(),
        signer2.address().into(),
        signer3.address().into(),
    ];

    let configurables = MultiSigPredicateConfigurables::default()
        .with_SIGNERS(signers)?
        .with_REQUIRED_SIGNATURES(2)?;

    // Load predicate
    let predicate = Predicate::load_from("predicates/multi-sig/out/debug/multi_sig_predicate.bin")?
        .with_provider(signer1.provider().clone())
        .with_configurables(configurables);

    // Fund predicate
    let fund_amount = 500_000;
    let initial_balance = provider.get_asset_balance(&signer1.address(), &AssetId::default()).await?;
    
    println!("  Initial balances (authorization test):");
    println!("  Signer1 balance: {}", initial_balance);
    println!("  Predicate balance: 0");
    println!("  Funding predicate with {} tokens...", fund_amount);
    
    signer1
        .transfer(
            predicate.address(),
            fund_amount,
            AssetId::default(),
            TxPolicies::default(),
        )
        .await?;

    // Verify predicate balance
    let predicate_balance = predicate.get_asset_balance(&AssetId::default()).await?;
    let final_signer1_balance = provider.get_asset_balance(&signer1.address(), &AssetId::default()).await?;
    
    println!("  After funding predicate:");
    println!("  Signer1 balance: {} (was: {})", final_signer1_balance, initial_balance);
    println!("  Predicate balance: {}", predicate_balance);
    println!("  Transfer fee: {}", initial_balance - final_signer1_balance - fund_amount as u128);
    
    assert_eq!(predicate_balance, fund_amount as u128);

    println!("✅ Predicate authorization test completed");

    Ok(())
}

// Test spending from predicate using 2/3 signatures
#[tokio::test]
async fn test_predicate_spending_2_of_3() -> Result<()> {
    println!("Testing predicate spending with 2/3 signatures...");

    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(3), Some(1), Some(1_000_000)),
        None,
        None,
    )
    .await?;

    let provider = wallets[0].provider().clone();
    let asset_id = AssetId::default();

    let signer1 = &wallets[0];
    let signer2 = &wallets[1];
    let signer3 = &wallets[2];

    // Configure predicate
    let signers = [
        signer1.address().into(),
        signer2.address().into(),
        signer3.address().into(),
    ];

    let configurables = MultiSigPredicateConfigurables::default()
        .with_SIGNERS(signers)?
        .with_REQUIRED_SIGNATURES(2)?;

    // Load predicate
    let predicate = Predicate::load_from("predicates/multi-sig/out/debug/multi_sig_predicate.bin")?
        .with_provider(provider.clone())
        .with_configurables(configurables);

    // Fund predicate
    let fund_amount = 500_000;
    let initial_balance = provider.get_asset_balance(&signer1.address(), &asset_id).await?;
    
    println!("  Initial balances:");
    println!("  Signer1 balance: {}", initial_balance);
    println!("  Predicate balance: 0");
    println!("  Funding predicate with {} tokens...", fund_amount);
    
    signer1
        .transfer(
            predicate.address(),
            fund_amount,
            asset_id,
            TxPolicies::default(),
        )
        .await?;

    // Verify predicate is funded
    let predicate_balance = provider.get_asset_balance(&predicate.address(), &asset_id).await?;
    let signer1_balance_after_funding = provider.get_asset_balance(&signer1.address(), &asset_id).await?;
    
    println!("  After funding predicate:");
    println!("  Signer1 balance: {}", signer1_balance_after_funding);
    println!("  Predicate balance: {}", predicate_balance);
    println!("  Transfer fee: {}", initial_balance - signer1_balance_after_funding - fund_amount as u128);
    
    assert_eq!(predicate_balance, fund_amount as u128);

    // Build transaction to spend from predicate
    let spend_amount = 300_000;
    let gas_amount = 1; // Reserve some for gas
    
    println!("  Before spending from predicate:");
    println!("  Predicate balance: {}", provider.get_asset_balance(&predicate.address(), &asset_id).await?);
    println!("  Signer1 balance: {}", provider.get_asset_balance(&signer1.address(), &asset_id).await?);
    println!("  Spending {} tokens (reserving {} for gas)...", spend_amount, gas_amount);
    
    let input_coin = predicate.get_asset_inputs_for_amount(asset_id, 1, None).await?;
    let output_coin = predicate.get_asset_outputs_for_amount(
        signer1.address().into(), 
        asset_id, 
        (spend_amount - gas_amount) as u64
    );

    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        input_coin,
        output_coin,
        TxPolicies::default(),
    );

    // For predicate spending with multiple signatures, we need to add both signers
    // The predicate will verify the signatures in the witnesses
    println!("🔐 Adding signatures from both signers...");
    signer1.adjust_for_fee(&mut transaction_builder, 0).await?;
    signer1.add_witnesses(&mut transaction_builder)?;
    signer2.adjust_for_fee(&mut transaction_builder, 0).await?;
    signer2.add_witnesses(&mut transaction_builder)?;

    // Build and send transaction
    println!("🚀 Building and sending transaction...");
    let transaction = transaction_builder.build(provider.clone()).await?;
    provider.send_transaction_and_await_commit(transaction).await?;
    println!("✅ Transaction executed successfully!");

    // Verify predicate balance decreased
    let final_predicate_balance = provider.get_asset_balance(&predicate.address(), &asset_id).await?;
    let final_signer1_balance = provider.get_asset_balance(&signer1.address(), &asset_id).await?;
    
    println!("  After spending from predicate:");
    println!("  Predicate balance: {} (was: {})", final_predicate_balance, fund_amount);
    println!("  Signer1 balance: {} (was: {})", final_signer1_balance, signer1_balance_after_funding);
    println!("  Amount spent: {}", spend_amount);
    println!("  Gas used: {}", gas_amount);
    
    assert_eq!(final_predicate_balance, (fund_amount - spend_amount) as u128);
    assert!(final_signer1_balance > initial_balance - fund_amount as u128);

    println!("✅ Predicate spending test completed successfully");

    Ok(())
}

// Test that spending fails with insufficient signatures
#[tokio::test]
async fn test_predicate_spending_insufficient_signatures() -> Result<()> {
    println!("Testing predicate spending fails with insufficient signatures...");

    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(3), Some(1), Some(1_000_000)),
        None,
        None,
    )
    .await?;

    let provider = wallets[0].provider().clone();
    let asset_id = AssetId::default();

    let signer1 = &wallets[0];
    let signer2 = &wallets[1];
    let signer3 = &wallets[2];

    // Configure predicate
    let signers = [
        signer1.address().into(),
        signer2.address().into(),
        signer3.address().into(),
    ];

    let configurables = MultiSigPredicateConfigurables::default()
        .with_SIGNERS(signers)?
        .with_REQUIRED_SIGNATURES(2)?;

    // Load predicate
    let predicate = Predicate::load_from("predicates/multi-sig/out/debug/multi_sig_predicate.bin")?
        .with_provider(provider.clone())
        .with_configurables(configurables);

    // Fund predicate
    let fund_amount = 500_000;
    let initial_balance = provider.get_asset_balance(&signer1.address(), &asset_id).await?;
    
    println!("  Initial balances (insufficient signatures test):");
    println!("  Signer1 balance: {}", initial_balance);
    println!("  Predicate balance: 0");
    println!("  Funding predicate with {} tokens...", fund_amount);
    
    signer1
        .transfer(
            predicate.address(),
            fund_amount,
            asset_id,
            TxPolicies::default(),
        )
        .await?;

    // Build transaction to spend from predicate
    let spend_amount = 300_000;
    let gas_amount = 1;
    
    println!("  Before attempting to spend (insufficient signatures):");
    println!("  Predicate balance: {}", provider.get_asset_balance(&predicate.address(), &asset_id).await?);
    println!("  Signer1 balance: {}", provider.get_asset_balance(&signer1.address(), &asset_id).await?);
    println!("  Attempting to spend {} tokens with only 1 signature...", spend_amount);
    
    let input_coin = predicate.get_asset_inputs_for_amount(asset_id, 1, None).await?;
    let output_coin = predicate.get_asset_outputs_for_amount(
        signer1.address().into(), 
        asset_id, 
        (spend_amount - gas_amount) as u64
    );

    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        input_coin,
        output_coin,
        TxPolicies::default(),
    );

    // Add fees and witnesses from only one signer (insufficient for 2/3 requirement)
    println!("  Adding signature from only one signer (insufficient for 2/3 requirement)...");
    signer1.adjust_for_fee(&mut transaction_builder, 0).await?;
    signer1.add_witnesses(&mut transaction_builder)?;

    // Build transaction
    println!("  Building transaction with insufficient signatures...");
    let transaction = transaction_builder.build(provider.clone()).await?;
    
    // Attempt to send transaction - should fail
    println!("❌ Attempting to execute transaction (should fail due to insufficient signatures)...");
    let result = provider.send_transaction_and_await_commit(transaction).await;
    assert!(result.is_err());
    println!("✅ Transaction correctly failed due to insufficient signatures");

    // Verify predicate balance remains unchanged
    let final_predicate_balance = provider.get_asset_balance(&predicate.address(), &asset_id).await?;
    let final_signer1_balance = provider.get_asset_balance(&signer1.address(), &asset_id).await?;
    
    println!("  After failed transaction attempt:");
    println!("  Predicate balance: {} (unchanged)", final_predicate_balance);
    println!("  Signer1 balance: {} (unchanged)", final_signer1_balance);
    
    assert_eq!(final_predicate_balance, fund_amount as u128);

    println!("✅ Predicate insufficient signatures test completed successfully");

    Ok(())
} 