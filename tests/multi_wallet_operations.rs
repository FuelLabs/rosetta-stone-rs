// Multi Wallet Operations Tests
// 
// This module contains tests for multi-wallet interactions including:
// - Minting to multiple users
// - Token transfers between wallets
// - Multi-wallet balance management
// - Complex wallet interactions

use fuels::{
    accounts::signers::private_key::PrivateKeySigner,
    prelude::*,
    types::{Bits256, Identity, SizedAsciiString},
};

use fuels::accounts::wallet::Unlocked;

// Load abi from json
abigen!(
    Contract(
        name = "Src20Token",
        abi = "contracts/src20-token/out/debug/src20_token-abi.json",
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

// Test minting tokens to multiple users and transferring between them
#[tokio::test]
async fn test_multi_wallet_interactions() -> Result<()> {
    println!("🧪 Testing multi-wallet interactions...");

    // Set up test wallets
    let num_wallets = 5;
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
    let user_wallets = wallets;

    // Deploy the SRC20 token contract
    let token_contract = deploy_src20_token(
        admin_wallet.clone(),
        "MULTITK",
        "MULTK",
        9,
    ).await?;

    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    // Mint tokens to all the users
    for (i, user_wallet) in user_wallets.iter().enumerate() {
        let amount = TOKEN_AMOUNT + (i as u64 * 1000);
        let recipient = Identity::Address(user_wallet.address().into());

        println!(
            "🔁 Minting {} tokens to user {}: {:?}",
            amount,
            i + 1,
            recipient
        );

        // Mint tokens to the user wallet
        let _mint_tx: fuels::programs::responses::CallResponse<()> = admin_token_contract
            .methods()
            .mint(recipient, Some(SUB_ID), amount)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await?;

        println!("✅ Mint transaction successful for user {}!", i + 1);
    }

    println!("✅ Multi-wallet minting completed");

    // Get the asset ID for transfers
    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    // Verify balances before transfer
    println!("🔍 Checking balances before transfer...");
    for (i, user_wallet) in user_wallets.iter().enumerate() {
        let balance = user_wallet.get_asset_balance(&asset_id).await?;
        println!("User {} balance: {}", i + 1, balance);
    }

    // Perform the transfer
    let transfer_amount = 50_000;

    println!("🔄 About to transfer {} tokens", transfer_amount);
    println!("From: {} (User 1)", user_wallets[0].address());
    println!("To: {} (User 2)", user_wallets[1].address());
    println!("Asset ID: {:?}", asset_id);

    // Get initial balances
    let sender_initial_balance = user_wallets[0].get_asset_balance(&asset_id).await?;
    let recipient_initial_balance = user_wallets[1].get_asset_balance(&asset_id).await?;

    println!("📊 Initial balances:");
    println!("  Sender: {}", sender_initial_balance);
    println!("  Recipient: {}", recipient_initial_balance);

    // Verify sender has enough tokens
    if sender_initial_balance < transfer_amount as u128 {
        panic!(
            "❌ Sender has insufficient balance: {} < {}",
            sender_initial_balance, transfer_amount
        );
    }

    // Transfer tokens txn from user1 to user2
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
            println!("✅ Transfer successful! Transaction ID: {:?}", tx_result.tx_id);
        }
        Err(e) => {
            println!("❌ Transfer failed: {:?}", e);
            return Err(e.into());
        }
    }

    println!("🔄 Checking balances after transfer...");

    // Query balances after transfer
    let sender_final_balance = user_wallets[0].get_asset_balance(&asset_id).await?;
    let recipient_final_balance = user_wallets[1].get_asset_balance(&asset_id).await?;

    println!("📊 Final balances:");
    println!("  Sender: {} (was {})", sender_final_balance, sender_initial_balance);
    println!("  Recipient: {} (was {})", recipient_final_balance, recipient_initial_balance);

    let expected_sender_balance = sender_initial_balance - transfer_amount as u128;
    let expected_recipient_balance = recipient_initial_balance + transfer_amount as u128;

    println!("🔄 Running assertions...");
    println!("  Expected sender balance: {}", expected_sender_balance);
    println!("  Expected recipient balance: {}", expected_recipient_balance);

    // Assert balances are as expected after transfer
    assert_eq!(
        sender_final_balance, 
        expected_sender_balance,
        "Sender balance mismatch: expected {}, got {}",
        expected_sender_balance,
        sender_final_balance
    );
    
    assert_eq!(
        recipient_final_balance, 
        expected_recipient_balance,
        "Recipient balance mismatch: expected {}, got {}",
        expected_recipient_balance,
        recipient_final_balance
    );

    println!("✅ Multi-wallet interactions test completed successfully!");

    Ok(())
}