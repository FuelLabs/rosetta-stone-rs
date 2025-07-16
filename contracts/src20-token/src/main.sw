contract;

use src20::{SetDecimalsEvent, SetNameEvent, SetSymbolEvent, SRC20, TotalSupplyEvent};
use src3::SRC3;
use std::{
    asset::{
        burn,
        mint_to,
        transfer
    },
    auth::msg_sender,
    call_frames::msg_asset_id,
    constants::DEFAULT_SUB_ID,
    context::{
        balance_of,
        msg_amount,
    },
    logging::log,
    string::String,
};

configurable {
    /// The decimals of the asset minted by this contract.
    DECIMALS: u8 = 9u8,
    /// The name of the asset minted by this contract.
    NAME: str[7] = __to_str_array("MyAsset"),
    /// The symbol of the asset minted by this contract.
    SYMBOL: str[5] = __to_str_array("MYTKN"),
    /// The initial supply of the asset minted by this contract.
    INITIAL_SUPPLY: u64 = 1_000_000_000,
    /// The admin of the contract (can mint/burn tokens).
    ADMIN: Identity = Identity::Address(Address::zero()),
}

storage {
    /// The total supply of the asset minted by this contract.
    total_supply: u64 = 0,
}

// Event structs for logging
pub struct MintEvent {
    pub recipient: Identity,
    pub amount: u64,
    pub asset_id: AssetId,
}

pub struct BurnEvent {
    pub amount: u64,
    pub asset_id: AssetId,
}

pub struct TransferEvent {
    pub from: Identity,
    pub to: Identity,
    pub amount: u64,
    pub asset_id: AssetId,
}

// SRC3 extends SRC20, so this must be included
impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        1
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == AssetId::default() {
            Some(storage.total_supply.read())
        } else {
            None
        }
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(NAME)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(SYMBOL)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == AssetId::default() {
            Some(DECIMALS)
        } else {
            None
        }
    }
}

abi EmitSRC20Events {
    fn emit_src20_events();
}

impl EmitSRC20Events for Contract {
    fn emit_src20_events() {
        // Metadata that is stored as a configurable should only be emitted once.
        let asset = AssetId::default();
        let sender = msg_sender().unwrap();
        let name = Some(String::from_ascii_str(from_str_array(NAME)));
        let symbol = Some(String::from_ascii_str(from_str_array(SYMBOL)));

        SetNameEvent::new(asset, name, sender).log();
        SetSymbolEvent::new(asset, symbol, sender).log();
        SetDecimalsEvent::new(asset, DECIMALS, sender).log();
    }
}

impl SRC3 for Contract {
    /// Mints assets to a given identity.
    #[storage(read, write)]
    fn mint(recipient: Identity, sub_id: Option<SubId>, amount: u64) {
        // Only the admin can mint assets.
        require(
            msg_sender()
                .unwrap() == ADMIN,
            "Unauthorized: Only admin can mint",
        );
        require(
            sub_id
                .is_some() && sub_id
                .unwrap() == DEFAULT_SUB_ID,
            "Incorrect Sub Id",
        );

        // Increment total supply of the asset and mint to the recipient.
        let new_supply = amount + storage.total_supply.read();
        storage.total_supply.write(new_supply);

        mint_to(recipient, DEFAULT_SUB_ID, amount);

        // Log mint event
        log(MintEvent {
            recipient,
            amount,
            asset_id: AssetId::default(),
        });

        TotalSupplyEvent::new(AssetId::default(), new_supply, msg_sender().unwrap())
            .log();
    }

    //   Burns coins of the given asset.
    #[payable]
    #[storage(read, write)]
    fn burn(sub_id: SubId, amount: u64) {
        require(sub_id == DEFAULT_SUB_ID, "Incorrect Sub Id");
        require(msg_amount() >= amount, "Incorrect amount provided");
        require(
            msg_asset_id() == AssetId::default(),
            "Incorrect asset provided",
        );

        // Decrement total supply of the asset and burn.
        let new_supply = storage.total_supply.read() - amount;
        storage.total_supply.write(new_supply);

        burn(DEFAULT_SUB_ID, amount);

        // Log burn event
        log(BurnEvent {
            amount,
            asset_id: AssetId::default(),
        });

        TotalSupplyEvent::new(AssetId::default(), new_supply, msg_sender().unwrap())
            .log();
    }
}

// Additional ABI for cross-contract interaction demonstrations
abi TokenInteraction {
    #[storage(read)]
    fn get_balance() -> u64;

    #[storage(read, write)]
    fn transfer_to_contract(contract_id: ContractId, amount: u64);

    #[storage(read)]
    fn get_contract_balance(contract_id: ContractId) -> u64;

    #[storage(read)]
    fn get_asset_id() -> AssetId;
}

impl TokenInteraction for Contract {
    /// Get balance of tokens for a given Identity.
    #[storage(read)]
    fn get_balance() -> u64 {
        balance_of(ContractId::this(), AssetId::default())
    }

    /// Transfer tokens to another contract.
    #[storage(read, write)]
    fn transfer_to_contract(contract_id: ContractId, amount: u64) {
        let contract_identity = Identity::ContractId(contract_id);
        transfer(contract_identity, AssetId::default(), amount);

        // Log transfer event
        log(TransferEvent {
            from: msg_sender().unwrap(),
            to: contract_identity,
            amount,
            asset_id: AssetId::default(),
        });
    }

    /// Get balance of tokens held by a contract.
    #[storage(read)]
    fn get_contract_balance(contract_id: ContractId) -> u64 {
        let addr = ContractId::from(contract_id);
        balance_of(addr, AssetId::default())
    }

    /// Get the AssetId of this contract's native token.
    #[storage(read)]
    fn get_asset_id() -> AssetId {
        AssetId::default()
    }
}
