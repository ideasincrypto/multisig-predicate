predicate;

use std::result::*;
use std::address::Address;
use std::b512::B512;
use std::ecr::{EcRecoverError,ec_recover_address};
use std::logging::log;
use std::revert::*;

// Return pointer to 32-byte hash region 
pub fn tx_hash() -> b256 {
    asm() {
        zero: b256
    }
}

pub fn get_signature(index: u8) -> B512 {
  
    asm(r1: index, r2, r3) {
        xws r2 r1;
        addi r3 r2 i8;
        r3: B512
    }
}

fn main() -> bool {
    let mut result = false;
    
    let raw_address:b256 = 0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1;

    let owner_address: Address = ~Address::from(raw_address);
    let msg_hash: b256 = tx_hash();
    let sig = get_signature(0);    
   
    let result_address: Result<Address, EcRecoverError> = ec_recover_address(sig, msg_hash);
    if let Result::Ok(address) = result_address {
       
        if (owner_address == address) {
            result = true
        }
    } else {
        revert(0);
    }
    result
}
