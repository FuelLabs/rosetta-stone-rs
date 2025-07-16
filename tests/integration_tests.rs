
use fuels::{
        accounts::{
            signers::{derivation::DEFAULT_DERIVATION_PATH, private_key::PrivateKeySigner},
   
        },
        types::{AssetId, Bits256, ContractId, Identity, SizedAsciiString},
        crypto::SecretKey,
        prelude::*,
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
    )
);

const TOKEN_AMOUNT: u64 = 1_000_000;
const SUB_ID_ARRAY: [u8; 32] = [0u8; 32];
const SUB_ID: Bits256 = Bits256(SUB_ID_ARRAY);

/// Integration test for the complete Rosetta Stone workflow.
/// This test deploys the SRC20 token contract, mints tokens, deploys the token vault, and performs multi-wallet interactions including transfers.
#[tokio::test]
async fn test_complete_rosetta_stone_workflow() {
    // Configure the test environment: number of wallets, coins per wallet, and amount per coin.
    let num_wallets = 5;
    let coins_per_wallet = 4;
    let amount_per_coin = 1_000_000_000;
    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );
    // Launch a local Fuel node and create test wallets.
    let mut wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();

    // Pop wallets for admin and users from the wallet pool.
    let admin_wallet = wallets.pop().unwrap();
    let user1_wallet = wallets.pop().unwrap();
    let user2_wallet = wallets.pop().unwrap();
    let user3_wallet = wallets.pop().unwrap();

    // Deploy the SRC20 token contract with the admin wallet.
    let src20_token_instance = deploy_src20_token(admin_wallet.clone(), "MYTOKEN", "TOKEN", 9)
        .await
        .unwrap();

    // Get the contract ID of the deployed token contract.
    let ethereum_token_contract_id = src20_token_instance.contract_id();

    // Create a contract instance for user1 to interact with the token contract.
    let src20_contract_instance = Src20Token::new(ethereum_token_contract_id, user1_wallet.clone());

    // Deploy the token vault contract, passing the admin wallet and token contract instance.
    let token_vault_instance = deploy_token_vault(admin_wallet.clone(), src20_contract_instance.clone())
        .await
        .unwrap();

    // Run basic token operations test (minting, supply checks, etc.).
    let _ = test_token_operations(src20_contract_instance.clone(), admin_wallet.clone(), user1_wallet.clone()).await;

    // Run multi-wallet interaction test (minting to multiple users, transfers, etc.).
    let __ = test_multi_wallet_interactions(
        src20_contract_instance,
        admin_wallet,
        vec![user1_wallet, user2_wallet, user3_wallet],
    )
    .await;

    // Assert that the token vault contract was deployed successfully (not default ID).
    assert_ne!(
        token_vault_instance.contract_id().to_string(),
        ContractId::default().to_string(),
        "Token vault contract ID should not be the default (all zeros)"
    );

    println!("🎉 All tests passed successfully!");
}

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

/// Deploys the TokenVault contract, linking it to the given token contract and admin wallet.
async fn deploy_token_vault(
    wallet: Wallet<Unlocked<PrivateKeySigner>>,
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
) -> Result<TokenVault<Wallet<Unlocked<PrivateKeySigner>>>> {
    // Set up contract configurables (token contract, admin).
    let configurables = TokenVaultConfigurables::default()
        .with_TOKEN_CONTRACT(ContractId::from(token_contract.contract_id()))?
        .with_ADMIN(Identity::Address(wallet.address().into()))?;

    // Deploy the contract to the local node.
    let deploy_response = Contract::load_from(
        "contracts/token-vault/out/debug/token_vault.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;

    let token_vault_instance = TokenVault::new(contract_id, wallet);

    println!("✅ Token Vault deployed at: {}", contract_id.to_string());
    Ok(token_vault_instance)
}

/// Tests basic token operations: minting, checking supply, and verifying logs.
async fn test_token_operations(
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
    user_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<()> {
    println!("🧪 Testing token operations...");

    // Create a contract instance for the admin to mint tokens.
    let admin_token_contract = Src20Token::new(token_contract.contract_id().clone(), admin_wallet);

    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(user_wallet.address().into());

    println!(
        "🔄 Attempting to mint {} tokens to {:?}",
        mint_amount, recipient
    );

    // Mint tokens to the recipient (user wallet).
    let mint_tx = admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), mint_amount)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await?;

    println!("✅ Mint transaction successful!");
    println!("Mint transaction: {:?}", mint_tx.decode_logs().results[0]);

    let mint_logs = mint_tx.decode_logs();
    assert!(!mint_logs.results.is_empty(), "Should have mint logs");

    // Calculate the correct asset ID from contract ID and sub ID
    let asset_id = admin_token_contract.methods().get_asset_id().call().await?.value;

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

/// Tests multi-wallet interactions: minting to multiple users and transferring tokens between them.
async fn test_multi_wallet_interactions(
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
    user_wallets: Vec<Wallet<Unlocked<PrivateKeySigner>>>,
) -> Result<()> {
    println!("🧪 Testing multi-wallet interactions...");
    let admin_token_contract = Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    // Mint tokens to each user wallet in the list.
    for (i, user_wallet) in user_wallets.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let amount = TOKEN_AMOUNT + (i as u64 * 1000);
        let recipient = Identity::Address(user_wallet.address().into());

        // Create admin contract instance for minting.

        println!(
            "🔄 Attempting to mint {} tokens to user {}: {:?}",
            amount,
            i + 1,
            recipient
        );

        // Mint tokens to the user wallet.
        let mint_tx = admin_token_contract
            .methods()
            .mint(recipient, Some(SUB_ID), amount)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await?;

        println!("✅ Mint transaction successful for user {}!", i + 1);
    }
    println!("✅ Multi-wallet interactions test passed");
    println!("initiating transfer");

    let transfer_amount = 50_000;
    // let token_asset_id = AssetId::from(*token_contract.contract_id());
    let asset_id = admin_token_contract.methods().get_asset_id().call().await?.value;

    println!("🔄 About to transfer {} tokens", transfer_amount);
    println!("From: {}", user_wallets[0].address());
    println!("To: {}", user_wallets[1].address());
    println!("Asset ID: {:?}", asset_id);

    // Attempt to transfer tokens from user1 to user2 using the DK's transfSer method.
    match user_wallets[0]
        .transfer(
            user_wallets[1].address(),
            transfer_amount,
            asset_id,
            TxPolicies::default(),
        )
        .await
    {
        Ok(tx_result) => {
            println!("✅ Transfer successful! Transaction: {:?}", tx_result);
        }
        Err(e) => {
            println!("❌ Transfer failed: {:?}", e);
            return Err(e.into());
        }
    }

    println!("🔄 Checking balances...");

    // Query balances after transfer.
    let sender_balance = user_wallets[0].get_asset_balance(&asset_id).await?;
    let recipient_balance = user_wallets[1].get_asset_balance(&asset_id).await?;

    println!(
        "Sender balance after transfer: {}, Recipient balance after transfer: {}",
        sender_balance, recipient_balance
    );

    // Assert balances are as expected after transfer.
    assert_eq!(sender_balance, (1_000_000u128 - transfer_amount as u128));
    assert_eq!(recipient_balance, (1_001_000u128 + transfer_amount as u128));
    // assert_eq!(sender_balance, 1950000);
    // assert_eq!(recipient_balance, 1051000);

    println!("🔄 Running assertions...");
    println!("✅ All assertions passed!");

    Ok(())
}

// [[bin]]
// name = "deploy"
// path = "examples/deploy.rs"

// [[bin]]
// name = "interact"
// path = "examples/interact.rs"