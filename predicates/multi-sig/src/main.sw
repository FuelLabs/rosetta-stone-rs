predicate;

use std::{
    ecr::ec_recover_address,
    b512::B512,
    logging::log,
    hash::{Hash, keccak256},
    tx::{
    tx_witness_data,
    tx_witnesses_count,
    tx_id
},};

configurable {
    /// Required signers for the multi-sig.
    SIGNERS: [Address; 3] = [
        Address::zero(),
        Address::zero(), 
        Address::zero(),
    ],
    /// Number of signatures required.
    REQUIRED_SIGNATURES: u64 = 2,
}

fn main() -> bool {
    let mut valid_signatures = 0;
 
    // Verifiying each potential signature 
    valid_signatures = verify_signature(0);
    valid_signatures = valid_signatures + verify_signature(1);
    valid_signatures = valid_signatures + verify_signature(2);
 
    if valid_signatures >= REQUIRED_SIGNATURES {
        return true;
    }
    return false;
}

fn verify_signature(i: u64) -> u64 {
    // Discard any out of bounds signatures
    if (i >= tx_witnesses_count()) {
        return 0;
    }
 
    let tx_hash = tx_id();
 
    let mut j = 0;
 
    while j < 3 {
        let current_signature = tx_witness_data::<B512>(j).unwrap();
        
        let current_address = ec_recover_address(current_signature, tx_hash).unwrap();
 
        if current_address == SIGNERS[i] {
            return 1;
        }
 
        j += 1;
    }
    return 0;
}