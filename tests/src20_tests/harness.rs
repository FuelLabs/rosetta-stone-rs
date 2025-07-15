use fuels::{prelude::*, types::ContractId};

// Load abi from json
abigen!(Contract(
    name = "Src20Token",
    abi = "contracts/src20-token/out/debug/src20_token-abi.json"
));

async fn get_contract_instance() -> (Src20Token<WalletUnlocked>, ContractId) {
    // This helper launches a local node and provides 10 test wallets linked to it.
    let num_wallets = 5;
    let coins_per_wallet = 4;
    let amount_per_coin = 100;
    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );
    // Launches a local node and provides test wallets as specified by the config.
    let mut wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();

    let wallet = wallets.pop().unwrap();

    let id = Contract::load_from(
        ".ontracts/src20-token/out/debug/src20_token.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = Src20Token::new(id.clone(), wallet);

    (instance, id.into())
}
