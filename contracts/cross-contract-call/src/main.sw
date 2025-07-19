contract;

use std::{
    asset::transfer,
    auth::msg_sender,
    call_frames::{
        msg_asset_id,
    },
    context::{
        balance_of,
        msg_amount,
    },
    hash::Hash,
    logging::log,
    storage::storage_api::{
        read,
        write,
    },
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
    #[payable]
    #[storage(read, write)]
    fn cross_contract_deposit(user: Identity);

    /// Get the vault's balance of the accepted token.
    #[storage(read)]
    fn get_vault_balance() -> u64;
}

configurable {
    /// The admin of this contract.
    ADMIN: Identity = Identity::Address(Address::zero()),
}

abi CrossContractCall {
    #[payable]  
    fn deposit(token_vault_contract_id: ContractId, user: Identity);
}

impl CrossContractCall for Contract {
    #[payable]
    fn deposit(token_vault_contract_id: ContractId, user: Identity) {
        let msg_caller = msg_sender().unwrap();
        let amount = msg_amount();
        let asset_id = msg_asset_id();
        // restrict who can call this function
        require(msg_caller == ADMIN, "Only admin can deposit");

        let token_vault_contract = abi(TokenVault, token_vault_contract_id.into());

        // Call the cross_contract_deposit function on the token-vault contract
        token_vault_contract
            .cross_contract_deposit {
                gas: 1000000,
                coins: amount,
                asset_id: asset_id.into(),
            }(user);
    }
}
