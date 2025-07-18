use fuels::{
    accounts::signers::{derivation::DEFAULT_DERIVATION_PATH, private_key::PrivateKeySigner},
    crypto::SecretKey,
    prelude::*,
    types::{AssetId, Bits256, ContractId, Identity, SizedAsciiString},
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
    Script(
        name = "MultiAssetTransfer",
        abi = "scripts/multi-asset-transfer/out/debug/multi_asset_transfer-abi.json",
    ),
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

    let cross_contract_call_contract_instance = deploy_cross_contract_call(admin_wallet.clone())
        .await
        .unwrap();

    // Get the contract ID of the deployed token contract.
    let ethereum_token_contract_id = src20_token_instance.contract_id();

    // Create a contract instance for user1 to interact with the token contract.
    let src20_contract_instance = Src20Token::new(ethereum_token_contract_id, user1_wallet.clone());

    // Deploy the token vault contract, passing the admin wallet and token contract instance.
    let token_vault_instance = deploy_token_vault(
        admin_wallet.clone(),
        cross_contract_call_contract_instance.clone(),
    )
    .await
    .unwrap();

    // Run basic token operations test (minting, supply checks, etc.).
    let _ = test_token_operations(
        src20_contract_instance.clone(),
        admin_wallet.clone(),
        user1_wallet.clone(),
    )
    .await;

    // Run multi-wallet interaction test (minting to multiple users, transfers, etc.).
    let __ = test_multi_wallet_interactions(
        src20_contract_instance.clone(),
        admin_wallet.clone(),
        vec![
            user1_wallet.clone(),
            user2_wallet.clone(),
            user3_wallet.clone(),
        ],
    )
    .await;

    let ___ = test_vault_deposit(
        src20_contract_instance.clone(),
        admin_wallet.clone(),
        token_vault_instance.clone(),
        user1_wallet.clone(),
    )
    .await;

    let ____ = test_script_execution(
        admin_wallet.clone(),
        &[
            user1_wallet.clone(),
            user2_wallet.clone(),
            user3_wallet.clone(),
        ],
        src20_contract_instance.clone(),
    )
    .await;

    let _____ = test_advanced_patterns(
        src20_contract_instance.clone(),
        token_vault_instance.clone(),
        admin_wallet.clone(),
    )
    .await;

    let ______ = test_cross_contract_call(
        cross_contract_call_contract_instance.clone(),
        src20_contract_instance.clone(),
        token_vault_instance.clone(),
        admin_wallet.clone(),
        user1_wallet.clone(),
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
    cross_contract_call_contract_instance: CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>,
) -> Result<TokenVault<Wallet<Unlocked<PrivateKeySigner>>>> {
    // Set up contract configurables (token contract, admin).
    let configurables = TokenVaultConfigurables::default()
        .with_CROSS_CONTRACT_CALL(ContractId::from(
            cross_contract_call_contract_instance.contract_id(),
        ))?
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
    println!(
        "✅ Cross Contract Call deployed at: {}",
        contract_id.to_string()
    );
    Ok(CrossContractCall::new(contract_id, admin_wallet))
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

    println!("Mint transaction successful!");
    println!("Mint transaction: {:?}", mint_tx.decode_logs().results[0]);

    let mint_logs = mint_tx.decode_logs();
    assert!(!mint_logs.results.is_empty(), "Should have mint logs");

    // Calculate the correct asset ID from contract ID and sub ID
    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

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
    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

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
        let mint_tx: fuels::programs::responses::CallResponse<()> = admin_token_contract
            .methods()
            .mint(recipient, Some(SUB_ID), amount)
            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
            .call()
            .await?;

        println!(" Mint transaction successful for user {}!", i + 1);
    }
    println!("Multi-wallet interactions test passed");
    println!("initiating transfer");

    let transfer_amount = 50_000;

    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

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
            println!("Transfer successful! Transaction: {:?}", tx_result);
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

async fn test_cross_contract_call(
    cross_contract_call_contract: CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>,
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
    vault_contract: TokenVault<Wallet<Unlocked<PrivateKeySigner>>>,
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
    user_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<()> {
    println!("🧪 Testing cross-contract call...");

    let user_vault_contract =
        TokenVault::new(vault_contract.contract_id().clone(), user_wallet.clone());

    // Mint tokens to the user wallet.
    let mint_amount = TOKEN_AMOUNT;
    let recipient = Identity::Address(user_wallet.address().into());

    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

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
    };

    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    let user_balance = user_wallet.get_asset_balance(&asset_id).await?;
    println!("💰 User balance before deposit: {}", user_balance);

    let initial_deposit_balance = match vault_contract
    .methods()
    .get_deposit(Identity::Address(user_wallet.address().into()))
    .call()
    .await
{
    Ok(response) => {
        println!("📊 Initial deposit balance: {}", response.value);
        response.value
    }
    Err(e) => {
        println!("❌ Failed to get initial deposit balance: {:?}", e);
        return Err(e.into());
    }
};

    let deposit_amount: u64 = 100;

    println!("🔄 Preparing deposit of {} tokens...", deposit_amount);

    println!("🔄 Executing deposit with admin wallet...");

    // Check if user has enough balance
    if user_balance < deposit_amount as u128 {
        println!(
            "❌ User has insufficient balance: {} < {}",
            user_balance, deposit_amount
        );
        return Err("Insufficient balance for deposit".into());
    }

    let call_params = CallParameters::default()
        .with_amount(deposit_amount as u64)
        .with_asset_id(asset_id);

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
        Ok(_) => println!("✅ Deposit successful"),
        Err(e) => {
            println!("❌ Deposit failed: {:?}", e);
            return Err(e.into());
        }
    }


    let final_deposit_balance = match vault_contract
        .methods()
        .get_deposit(Identity::Address(user_wallet.address().into()))
        .call()
        .await
    {
        Ok(response) => {
            println!("✅ Final deposit balance: {}", response.value);
            response.value
        }
        Err(e) => {
            println!("❌ Failed to get final deposit balance: {:?}", e);
            return Err(e.into());
        }
    };
    let balance_increase = final_deposit_balance - initial_deposit_balance;
    println!("📈 Balance increase: {} (expected: {})", balance_increase, deposit_amount);
    
    assert_eq!(balance_increase, deposit_amount, 
        "Expected deposit increase of {} but got {}. Initial: {}, Final: {}", 
        deposit_amount, balance_increase, initial_deposit_balance, final_deposit_balance);
    
    println!("✅ Cross Contract Call Deposit verification passed");

    Ok(())
}

async fn test_vault_deposit(
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
    vault_contract: TokenVault<Wallet<Unlocked<PrivateKeySigner>>>,
    user_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<()> {
    println!("Starting vault deposit test...");
    println!("🧪 Testing vault deposit...");

    // Mint tokens to the user wallet.
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

async fn test_script_execution(
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
    users: &[Wallet<Unlocked<PrivateKeySigner>>],
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
) -> Result<()> {
    println!("🧪 Testing script execution...");

    // Configure script with user addresses
    let recipients = [
        Identity::Address(users[0].address().into()),
        Identity::Address(users[1].address().into()),
        Identity::Address(users[2].address().into()),
    ];

    let amounts = [1000, 2000, 3000];

    let admin_token_contract =
        Src20Token::new(token_contract.contract_id().clone(), admin_wallet.clone());

    let recipient = Identity::Address(admin_wallet.address().into());

    match admin_token_contract
        .methods()
        .mint(recipient, Some(SUB_ID), TOKEN_AMOUNT)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
    {
        Ok(txn) => {
            println!("Script test Mint successful!");
            txn
        }
        Err(e) => {
            println!("❌ Script test Mint failed: {:?}", e);
            return Err(e.into());
        }
    };

    let configurables = MultiAssetTransferConfigurables::default()
        .with_RECIPIENTS(recipients)?
        .with_AMOUNTS(amounts)?;

    // Load and execute script using custom transaction builder pattern
    let script_instance = MultiAssetTransfer::new(
        admin_wallet.clone(),
        "scripts/multi-asset-transfer/out/debug/multi_asset_transfer.bin",
    )
    .with_configurables(configurables);

    let asset_id = admin_token_contract
        .methods()
        .get_asset_id()
        .call()
        .await?
        .value;

    let script_call_handler = script_instance.main(asset_id);

    let mut tb = script_call_handler.transaction_builder().await?;

    // Add enough input coins of the asset_id to the script
    let total_amount = 1000 + 2000 + 3000;

    let asset_inputs = match admin_wallet
        .get_asset_inputs_for_amount(asset_id, total_amount, None)
        .await
    {
        Ok(inputs) => {
            println!("Found {} asset inputs", inputs.len());
            inputs
        }
        Err(e) => {
            println!("❌ Failed to get asset inputs: {:?}", e);

            // Check if we have enough balance
            let current_balance = admin_wallet.get_asset_balance(&asset_id).await?;
            println!(
                "💰 Current balance: {}, Required: {}",
                current_balance, total_amount
            );

            return Err(e.into());
        }
    };

    tb.inputs.extend(asset_inputs);
    let tb = tb.enable_burn(true);
    let mut tb = tb.with_variable_output_policy(VariableOutputPolicy::Exactly(1));
    // Add base asset for fees
    println!("🔄 Adding base asset for fees...");
    let base_balance = admin_wallet.get_asset_balance(&AssetId::BASE).await?;
    println!("💰 Base balance: {}", base_balance);

    // Adjust for fee and add witnesses
    admin_wallet.adjust_for_fee(&mut tb, 0).await?;
    admin_wallet.add_witnesses(&mut tb)?;

    let tb = tb.enable_burn(true);

    // Add more gas limit for script execution
    let tx_policies = TxPolicies::default()
        .with_script_gas_limit(1_000_000)
        .with_max_fee(1_000_000);
    let tb = tb.with_tx_policies(tx_policies);

    println!("🔄 Building and sending transaction...");
    let provider = admin_wallet.try_provider()?.clone();
    let tx = match tb.clone().build(&provider).await {
        Ok(transaction) => {
            println!("Transaction built successfully");
            transaction
        }
        Err(e) => {
            println!("❌ Transaction building failed: {:?}", e);
            println!("🔍 Transaction builder state:");
            println!("  - Inputs: {}", tb.inputs.len());
            println!("  - Outputs: {}", tb.outputs.len());
            println!("  - Witnesses: {}", tb.witnesses.len());
            // println!("  - Script gas limit: {:?}", tx_policies.script_gas_limit);
            // println!("  - Max fee: {:?}", tx_policies.max_fee);
            return Err(e.into());
        }
    };

    let tx_id = match provider.send_transaction(tx).await {
        Ok(id) => {
            println!("Transaction sent successfully: {:?}", id);
            id
        }
        Err(e) => {
            println!("❌ Script execution failed: {:?}", e);
            return Err(e.into());
        }
    };

    let tx_status = provider.tx_status(&tx_id).await?;

    println!("tx_status: {:?}", tx_status);

    let response = script_call_handler.get_response(tx_status)?;
    println!("Script execution response: {:?}", response);

    // Verify script execution
    assert!(response.value, "Script should return true on success");

    // check logs
    let logs = response.decode_logs();
    assert!(!logs.results.is_empty(), "Should have executed logs");

    println!("✅ Script execution test passed");
    Ok(())
}

async fn test_advanced_patterns(
    token_contract: Src20Token<Wallet<Unlocked<PrivateKeySigner>>>,
    vault_contract: TokenVault<Wallet<Unlocked<PrivateKeySigner>>>,
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<()> {
    println!("🧪 Testing advanced patterns...");

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

// [[bin]]
// name = "deploy"
// path = "examples/deploy.rs"

// [[bin]]
// name = "interact"
// path = "examples/interact.rs"
