use fuels::{prelude::*, types::ContractId, types::SizedAsciiString, types::Identity};

// Load abi from json
abigen!(Contract(
    name = "Src20Token",
    abi = "contracts/src20-token/out/debug/src20_token-abi.json"
));

const TOKEN_AMOUNT: u64 = 1_000_000;

#[tokio::test]
async fn get_contract_instance() {
    // This helper launches a local node and provides 10 test wallets linked to it.
    let num_wallets = 5;
    let coins_per_wallet = 4;
    let amount_per_coin = 1_000_000_000;
    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );
    // Launches a local node and provides test wallets as specified by the config.
    let mut wallets = launch_custom_provider_and_get_wallets(config, None, None).await.unwrap();

    let admin_wallet = wallets.pop().unwrap();
    let user1_wallet = wallets.pop().unwrap();
    let user2_wallet = wallets.pop().unwrap();
    let user3_wallet = wallets.pop().unwrap();

    let src20_token_instance = deploy_src20_token(
        &admin_wallet,
        "MYTOKEN", 
        "TOKEN",
        9,
    ).await.unwrap();
    
    let ethereum_token_contract_id = src20_token_instance.contract_id();

    let instance = Src20Token::new(ethereum_token_contract_id, user1_wallet);

    // Use the instance and contract_id here if needed, or just drop them.
    // For now, do nothing to satisfy the () return type.
}

async fn deploy_src20_token(
    wallet: &WalletUnlocked,
    name: &str,        // 🆕 Pass as string
    symbol: &str,      // 🆕 Pass as string  
    decimals: u8,
) -> Result<Src20Token<WalletUnlocked>> {
    
    // 🔄 Convert strings to byte arrays
    let name_bytes: SizedAsciiString<7> = name.try_into()?;
    // Max 12 bytes
    let symbol_bytes: SizedAsciiString<5> = symbol.try_into()?;

    let configurables = Src20TokenConfigurables::default()
        .with_NAME(name_bytes.clone())?      // 🎯 Use converted bytes
        .with_SYMBOL(symbol_bytes.clone())?  // 🎯 Use converted bytes
        .with_DECIMALS(decimals)?
        .with_ADMIN(Identity::Address(wallet.address().into()))?; // 🆕 Set admin to wallet address

    let contract_id = Contract::load_from(
        "contracts/src20-token/out/debug/src20_token.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(wallet, TxPolicies::default())
    .await?;

    println!("✅ Token '{}' ({}) deployed at: {}", name, symbol, contract_id.hash());
    Result::Ok(Src20Token::new(contract_id.clone(), wallet.clone()))
}

// [[bin]]
// name = "deploy"
// path = "examples/deploy.rs"

// [[bin]]
// name = "interact"
// path = "examples/interact.rs"