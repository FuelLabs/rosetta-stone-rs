//! Common utilities and constants for integration tests
//! 
//! This module provides shared functionality that can be used across
//! different test modules to avoid code duplication and improve maintainability.

use fuels::{
    accounts::signers::{derivation::DEFAULT_DERIVATION_PATH, private_key::PrivateKeySigner},
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

/// Common test constants
pub const TOKEN_AMOUNT: u64 = 1_000_000;
pub const SUB_ID_ARRAY: [u8; 32] = [0u8; 32];
pub const SUB_ID: Bits256 = Bits256(SUB_ID_ARRAY);

/// Deploys the SRC20 token contract with the given wallet and metadata.
/// Returns a contract instance for further interaction.
pub async fn deploy_src20_token(
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

/// Deploys the CrossContractCall contract
pub async fn deploy_cross_contract_call(
    admin_wallet: Wallet<Unlocked<PrivateKeySigner>>,
) -> Result<CrossContractCall<Wallet<Unlocked<PrivateKeySigner>>>> {
    let deploy_response = Contract::load_from(
        "contracts/cross-contract-call/out/debug/cross_contract_call.bin",
        LoadConfiguration::default(),
    )?
    .deploy(&admin_wallet, TxPolicies::default())
    .await?;

    let contract_id = deploy_response.contract_id;

    println!(
        "✅ CrossContractCall deployed at: {}",
        contract_id.to_string()
    );
    
    Ok(CrossContractCall::new(contract_id, admin_wallet))
}

/// Deploys the TokenVault contract, linking it to the given token contract and admin wallet.
pub async fn deploy_token_vault(
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

    println!(
        "✅ TokenVault deployed at: {}",
        contract_id.to_string()
    );
    
    Ok(TokenVault::new(contract_id, wallet))
} 