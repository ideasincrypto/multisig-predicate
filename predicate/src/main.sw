predicate;

use std::result::*;
use std::address::Address;
use std::b512::B512;
use std::ecr::{EcRecoverError,ec_recover_address};
use std::logging::log;
use std::revert::*;
use std::tx::*;

// Return pointer to 32-byte hash region 
pub fn tx_hash() -> b256 {
    asm() {
        zero: b256
    }
}

fn get_predicate_data_length() -> u64 {
    asm(r1) {
        subi r1 is i8;
        r1: u64
    }
}

fn get_predicate_length() -> u64 {
    asm(r1) {
        subi r1 is i16;
        r1: u64
    }
}

fn get_predicate_data<T>() -> T {
    // get the predicate length
    let mut predicate_byte_length = get_predicate_length() * 4;

    // if the number of instructions is odd, add 4,
    // since odd instruction numbers are padded to a multiple of 8
    if predicate_byte_length % 8 != 0 {
        predicate_byte_length = predicate_byte_length + 4;
    }

    asm(r1: predicate_byte_length) {
        add r1 is r1; // let predicateData = predicate_byte_length
        r1: T
    }
}

pub fn get_signature(index: u64) -> B512 {
  
    asm(r1: index, r2, r3) {
        xws r2 r1;
        addi r3 r2 i8;
        r3: B512
    }
}

fn main() -> bool {

    let raw_addresses = [0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1, 0x54944e5b8189827e470e5a8bacfc6c3667397dc4e1eef7ef3519d16d6d6c6610];
    let sig_threshold = 1;

    let msg_hash: b256 = tx_hash();
    let witness_indexes: u64 = get_predicate_data();
    let len = tx_witnesses_count();
    let mut valid_counter = 0;
    let sig1 = get_signature(0); 

    let result_address1: Result<Address, EcRecoverError> = ec_recover_address(sig1, msg_hash);
    if let Result::Ok(address) = result_address1 {
       
        if ((~Address::from(raw_addresses[0]) == address) || (~Address::from(raw_addresses[1]) == address)) {
            valid_counter = valid_counter + 1;
        }
    } else {
       //
    }

    if (len > 1) {
        let sig2 = get_signature(1); 

        let result_address2: Result<Address, EcRecoverError> = ec_recover_address(sig2, msg_hash);
        if let Result::Ok(address) = result_address2 {
            if ((~Address::from(raw_addresses[0]) == address) || (~Address::from(raw_addresses[1]) == address)) {
                valid_counter = valid_counter + 1;
            }
        } else {
       //
        }
    }
    valid_counter >= sig_threshold
}
