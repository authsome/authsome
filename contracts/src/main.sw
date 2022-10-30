predicate;

use std::b512::B512;
use std::constants::ZERO_B256;
use std::ecr::ec_recover_address;
use std::inputs::input_predicate_data;

use std::hash::sha256;

fn compose(w0: u64, w1: u64, w2: u64, w3: u64) -> b256 {
    let addr: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm(w0: w0, w1: w1, w2: w2, w3: w3, addr: addr) {
        sw addr w0 i0;
        sw addr w1 i1;
        sw addr w2 i2;
        sw addr w3 i3;
        addr: b256
    }
}

fn get_input_index() -> u8 {
    let txin: u8 = 0;
    asm(txin) {
        gm txin i3;
        txin: u8
    }
}

fn get_output_index(txin: u8) -> u8 {
    let txout: u8 = 0;
    asm(txin: txin, txout) {
        gtf txout txin i259;
        txout: u8
    }
}

fn get_output_txid(txin: u8) -> b256 {
    let addr = get_txid_address(txin);
    let w0 = get_word_at_address_offset_0(addr);
    let w1 = get_word_at_address_offset_1(addr);
    let w2 = get_word_at_address_offset_2(addr);
    let w3 = get_word_at_address_offset_3(addr);
    let txid = compose(w0, w1, w2, w3);
    return txid;
}

fn get_txid_address(txin: u8) -> u64 {
    let addr: u64 = 0;
    asm(txin: txin, addr) {
        gtf addr txin i258;
        addr: u64
    }
}

fn get_word_at_address_offset_0(addr: u64) -> u64 {
    let word: u64 = 0;
    asm(addr: addr, word) {
        lw word addr i0;
        word: u64
    }
}

fn get_word_at_address_offset_1(addr: u64) -> u64 {
    let word: u64 = 0;
    asm(addr: addr, word) {
        lw word addr i1;
        word: u64
    }
}

fn get_word_at_address_offset_2(addr: u64) -> u64 {
    let word: u64 = 0;
    asm(addr: addr, word) {
        lw word addr i2;
        word: u64
    }
}

fn get_word_at_address_offset_3(addr: u64) -> u64 {
    let word: u64 = 0;
    asm(addr: addr, word) {
        lw word addr i3;
        word: u64
    }
}

fn extract_public_key_and_match(signature: B512, expected_public_key: b256) -> u64 {

    let txin = get_input_index();
    let txout = get_output_index(txin);
    let txid = get_output_txid(txin);
    let msg_hash = sha256(txid); // we just work with one output for now

    if let Result::Ok(pub_key_sig) = ec_recover_address(signature, msg_hash)
    {
        if pub_key_sig.value == expected_public_key {
            return 1;
        }
    }
    0
}

fn main() -> bool {
    let signatures: [B512; 3] = input_predicate_data(0);

    let public_keys = [
        0xd58573593432a30a800f97ad32f877425c223a9e427ab557aab5d5bb89156db0,
        0x14df7c7e4e662db31fe2763b1734a3d680e7b743516319a49baaa22b2032a857,
        0x3ff494fb136978c3125844625dad6baf6e87cdb1328c8a51f35bda5afe72425c,
    ];

    let mut matched_keys = 0;

    matched_keys = extract_public_key_and_match(signatures[0], public_keys[0]);
    matched_keys = matched_keys + extract_public_key_and_match(signatures[1], public_keys[1]);
    matched_keys = matched_keys + extract_public_key_and_match(signatures[2], public_keys[2]);

    return matched_keys > 1;
}