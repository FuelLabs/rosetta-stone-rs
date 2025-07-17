contract;

use std::{
    asset::transfer,
    call_frames::{msg_asset_id},
    context::{msg_amount, balance_of},
    storage::storage_api::{read, write},
    logging::log,
    hash::Hash,
     auth::msg_sender,
};

abi TokenVault {
    /// Deposit tokens into the vault.
    #[payable]
    #[storage(read, write)]
    fn deposit();
    
    /// Withdraw tokens from the vault.
    #[storage(read, write)]
    fn withdraw(amount: u64);
    
    /// Get the deposit amount for a user.
    #[storage(read)]
    fn get_deposit(user: Identity) -> u64;
    
    /// Get total deposits in the vault.
    #[storage(read)]
    fn get_total_deposits() -> u64;
    
    /// Cross-contract transfer demonstration.
    #[storage(read, write)]
    fn cross_contract_deposit(user: Identity, amount: u64);
    
    /// Get the vault's balance of the accepted token.
    #[storage(read)]
    fn get_vault_balance() -> u64;
}

configurable {
    /// The admin of this contract.
    ADMIN: Identity = Identity::Address(Address::zero()),
}

abi CrossContractCall {
    fn deposit(token_vault_contract_id: b256, amount: u64);
}

impl CrossContractCall for Contract {
    fn deposit(token_vault_contract_id: b256, amount: u64) {
        let user = msg_sender().unwrap();
        // restrict who can call this function
        require(user == ADMIN, "Only admin can deposit");

        let token_vault_contract = abi(TokenVault, token_vault_contract_id);


        // Call the cross_contract_deposit function on the token-vault contract
        token_vault_contract.cross_contract_deposit(user, amount);
    }
}