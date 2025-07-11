script;

use std::{
    asset::transfer,
    logging::log,
};

configurable {
    /// List of recipients for the multi-asset transfer.
    RECIPIENTS: [Identity; 3] = [
        Identity::Address(Address::zero()),
        Identity::Address(Address::zero()),
        Identity::Address(Address::zero()),
    ],
    /// Amounts to transfer to each recipient.
    AMOUNTS: [u64; 3] = [1000, 2000, 3000],
}

fn main(asset_id: AssetId) -> bool {
    let mut i = 0;
    let mut success = true;
    
    while i < 3 {
        // Transfer to each recipient
        transfer(RECIPIENTS[i], asset_id, AMOUNTS[i]);
        
        // Log the transfer
        log(AMOUNTS[i]);
        log(RECIPIENTS[i]);
        
        i += 1;
    }
    
    log("Multi-asset transfer completed successfully");
    success
}