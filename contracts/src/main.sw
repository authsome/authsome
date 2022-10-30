predicate;

use std::b512::B512;
use std::constants::ZERO_B256;
use std::ecr::ec_recover_address;
use std::inputs::input_predicate_data;

fn compose(w0: u64, w1: u64, w2: u64, w3: u64) -> b256 {
    let res: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm(w0: w0, w1: w1, w2: w2, w3: w3, res: res) {
        sw res w0 i0;
        sw res w1 i1;
        sw res w2 i2;
        sw res w3 i3;
        res: b256
    }
}


fn get_input_index() -> u8 {
    asm(txin) {
        gm txin i3;
        txin: u8
    }
}

fn get_output_index(txin: u8) -> u8 {
    asm(txin: txin, txout) {
        gtf txout txin i259;
        txout: u8
    }
}

fn get_tx_id_memory_address(txin: u8) -> u64 {
    asm(txin: txin, addr) {
        gtf addr txin i258;
        addr: u64
    }
}

fn get_word_at_memory_address_offset_0(addr: u64) -> u64 {
    asm(addr: addr, word) {
        lw word addr i0;
        word: u64
    }
}

fn get_word_at_memory_address_offset_1(addr: u64) -> u64 {
    asm(addr: addr, word) {
        lw word addr i1;
        word: u64
    }
}

fn get_word_at_memory_address_offset_2(addr: u64) -> u64 {
    asm(addr: addr, word) {
        lw word addr i2;
        word: u64
    }
}

fn get_word_at_memory_address_offset_3(addr: u64) -> u64 {
    asm(addr: addr, word) {
        lw word addr i3;
        word: u64
    }
}

fn get_tx_id_at_address(addr: u64) -> b256 {
    let w0 = get_word_at_memory_address_offset_0(addr);
    let w1 = get_word_at_memory_address_offset_1(addr);
    let w2 = get_word_at_memory_address_offset_2(addr);
    let w3 = get_word_at_memory_address_offset_3(addr);
    let tx_id = compose(w0, w1, w2, w3);
    return tx_id;
}

fn extract_public_key_and_match(signature: B512, expected_public_key: b256) -> u64 {
    let txin = get_input_index();
    let txout = get_output_index(txin);
    let addr = get_tx_id_memory_address(txin);
    let tx_id = get_tx_id_at_address(addr);
    if let Result::Ok(pub_key_sig) = ec_recover_address(signature, ZERO_B256)
    {
        if pub_key_sig.value == expected_public_key {
            return 1;
        }
    }
    return 0;       
}

fn main() -> bool {
    let signatures: [B512; 3] = input_predicate_data(0);

    let public_keys = [
       0xec54e0a8f1c0d9d530fd2e8d673c86904c052901de5331637feb825efba56e3f,
       0xec54e0a8f1c0d9d530fd2e8d673c86904c052901de5331637feb825efba56e3f,
       0xec54e0a8f1c0d9d530fd2e8d673c86904c052901de5331637feb825efba56e3f,
    ];

    let mut matched_keys = 0;

    matched_keys = extract_public_key_and_match(signatures[0], public_keys[0]);
    matched_keys = matched_keys + extract_public_key_and_match(signatures[1], public_keys[1]);
    matched_keys = matched_keys + extract_public_key_and_match(signatures[2], public_keys[2]);

    return matched_keys > 1;
}