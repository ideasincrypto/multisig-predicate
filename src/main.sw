predicate;

use std::result::*;
use std::address::Address;
use std::b512::B512;
use std::logging::log;
use std::revert::*;
use std::tx::*;

// Return pointer to 32-byte hash region 
pub fn tx_hash() -> b256 {
    asm() {
        zero: b256
    }
}

fn get_signature(index: u64) -> B512 {
  
    asm(r1: index, r2, r3) {
        xws r2 r1;
        addi r3 r2 i8;
        r3: B512
    }
}

fn ec_recover_address(signature: B512, msg_hash: b256) -> b256 {

    asm(sig: signature.bytes, hash: msg_hash, addr_buffer, pub_key_buffer, hash_len: 64) {
        move addr_buffer sp; // Buffer for address.
        cfei i32;
        move pub_key_buffer sp; // Temporary buffer for recovered key.
        cfei i64;
        ecr pub_key_buffer sig hash; // Recover public_key from sig & hash.
        s256 addr_buffer pub_key_buffer hash_len; // Hash 64 bytes to the addr_buffer.
        cfsi i64; // Free temporary key buffer.
        addr_buffer: b256
    }
}

// 2 out of 3 multi-signature predicate
fn main() -> bool {

    let raw_addresses = [0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1, 
        0x54944e5b8189827e470e5a8bacfc6c3667397dc4e1eef7ef3519d16d6d6c6610,
        0x577e424ee53a16e6a85291feabc8443862495f74ac39a706d2dd0b9fc16955eb];
    let mut zero_address_mask = false;
    let mut one_address_mask = false;
    let mut second_address_mask = false;
    let sig_threshold = 2;

    let msg_hash: b256 = tx_hash();
    let witnesses_count = tx_witnesses_count();
    let mut valid_counter = 0;

    let sig0 = get_signature(0); 

    if (raw_addresses[0] == ec_recover_address(sig0, msg_hash)) {
            zero_address_mask = true;
            valid_counter = valid_counter + 1;
    } else if (raw_addresses[1] == ec_recover_address(sig0, msg_hash)) {
            one_address_mask = true;
            valid_counter = valid_counter + 1;
    } else if (raw_addresses[2] == ec_recover_address(sig0, msg_hash)) {
            second_address_mask = true;
            valid_counter = valid_counter + 1;
    }
    
    if (witnesses_count >= 2) {
        let sig1 = get_signature(1); 
        if ((raw_addresses[0] == ec_recover_address(sig1, msg_hash)) && !zero_address_mask) {
            zero_address_mask = true;
            valid_counter = valid_counter + 1;
        } else if ((raw_addresses[1] == ec_recover_address(sig1, msg_hash)) && !one_address_mask) {
            one_address_mask = true;
            valid_counter = valid_counter + 1;
        } else if ((raw_addresses[2] == ec_recover_address(sig1, msg_hash)) && !second_address_mask) {
            one_address_mask = true;
            valid_counter = valid_counter + 1;
        }
    }

    if (witnesses_count >= 3) {
        let sig2 = get_signature(2); 
        if ((raw_addresses[0] == ec_recover_address(sig2, msg_hash)) && !zero_address_mask) {
            valid_counter = valid_counter + 1;
        } else if ((raw_addresses[1] == ec_recover_address(sig2, msg_hash)) && !one_address_mask) {
            valid_counter = valid_counter + 1;
        }  else if ((raw_addresses[2] == ec_recover_address(sig2, msg_hash)) && !second_address_mask) {
            valid_counter = valid_counter + 1;
        }
    }

    valid_counter >= sig_threshold
}
