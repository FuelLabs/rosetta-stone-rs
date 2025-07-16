use fuels::{
    prelude::*,
    types::AssetId,
};


abigen!(Predicate(
    name = "MultiSigPredicate",
    abi = "predicates/multi-sig/out/debug/multi_sig_predicate-abi.json",
));

#[tokio::test]
async fn test_predicate_authorization() -> Result<()> {
    println!("🧪 Testing predicate authorization...");

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
    assert_eq!(predicate_balance, fund_amount as u128);

    println!("✅ Predicate authorization test completed");

    Ok(())
}
