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

configurable {
    /// The contract ID of the token this vault accepts.
    TOKEN_CONTRACT: ContractId = ContractId::zero(),
    /// The admin of the vault.
    ADMIN: Identity = Identity::Address(Address::zero()),
}

storage {
    /// Mapping of user deposits.
    deposits: StorageMap<Identity, u64> = StorageMap {},
    /// Total amount deposited in the vault.
    total_deposits: u64 = 0,
}

// Event structs
pub struct DepositEvent {
    pub user: Identity,
    pub amount: u64,
    pub asset_id: AssetId,
}

pub struct WithdrawEvent {
    pub user: Identity,
    pub amount: u64,
    pub asset_id: AssetId,
}

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

impl TokenVault for Contract {
    /// Deposit tokens into the vault.
    #[payable]
    #[storage(read, write)]
    fn deposit() {
        let amount = msg_amount();
        let asset_id = msg_asset_id();
        let sender = msg_sender().unwrap();
        
        // For this example, accept any asset, but in production you'd check:
        // require(asset_id == expected_asset_id, "Wrong asset type");
        
        // Update user's deposit balance
        let current_deposit = storage.deposits.get(sender).try_read().unwrap_or(0);
        storage.deposits.insert(sender, current_deposit + amount);
        
        // Update total deposits
        let new_total = storage.total_deposits.read() + amount;
        storage.total_deposits.write(new_total);
        
        // Log deposit event
        log(DepositEvent {
            user: sender,
            amount,
            asset_id,
        });
    }
    
    /// Withdraw tokens from the vault.
    #[storage(read, write)]
    fn withdraw(amount: u64) {
        let sender = msg_sender().unwrap();
        let current_deposit = storage.deposits.get(sender).try_read().unwrap_or(0);
        
        require(current_deposit >= amount, "Insufficient balance");
        
        // Update user's deposit balance
        storage.deposits.insert(sender, current_deposit - amount);
        
        // Update total deposits
        let new_total = storage.total_deposits.read() - amount;
        storage.total_deposits.write(new_total);
        
        // Transfer tokens back to user
        transfer(sender, msg_asset_id(), amount);
        
        // Log withdrawal event
        log(WithdrawEvent {
            user: sender,
            amount,
            asset_id: msg_asset_id(),
        });
    }
    
    /// Get the deposit amount for a user.
    #[storage(read)]
    fn get_deposit(user: Identity) -> u64 {
        storage.deposits.get(user).try_read().unwrap_or(0)
    }
    
    /// Get total deposits in the vault.
    #[storage(read)]
    fn get_total_deposits() -> u64 {
        storage.total_deposits.read()
    }
    
    /// Cross-contract transfer demonstration.
    #[storage(read, write)]
    fn cross_contract_deposit(user: Identity, amount: u64) {
        require(msg_sender().unwrap() == ADMIN, "Only admin can cross-contract deposit");
        
        // This would typically involve calling another contract
        // For demonstration, we'll just update the deposit
        let current_deposit = storage.deposits.get(user).try_read().unwrap_or(0);
        storage.deposits.insert(user, current_deposit + amount);
        
        let new_total = storage.total_deposits.read() + amount;
        storage.total_deposits.write(new_total);
        
        log("Cross-contract deposit completed");
    }
    
    /// Get the vault's balance of the accepted token.
    #[storage(read)]
    fn get_vault_balance() -> u64 {
        balance_of(ContractId::this(), msg_asset_id())
    }
}