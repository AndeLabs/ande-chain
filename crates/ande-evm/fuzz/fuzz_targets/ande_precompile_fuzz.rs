#![no_main]

use libfuzzer_sys::fuzz_target;
use alloy_primitives::{Address, Bytes, U256};

fuzz_target!(|data: &[u8]| {
    if data.len() < 96 {
        return;
    }

    let from = Address::from_slice(&data[0..20]);
    let to = Address::from_slice(&data[20..40]);
    
    let mut value_bytes = [0u8; 32];
    value_bytes.copy_from_slice(&data[40..72]);
    let value = U256::from_be_bytes(value_bytes);
    
    let mut caller_bytes = [0u8; 20];
    caller_bytes.copy_from_slice(&data[72..92]);
    let caller = Address::from_slice(&caller_bytes);

    let mut input = Vec::new();
    input.extend_from_slice(&[0u8; 12]);
    input.extend_from_slice(from.as_slice());
    input.extend_from_slice(&[0u8; 12]);
    input.extend_from_slice(to.as_slice());
    
    let value_full = value.to_be_bytes_vec();
    input.extend_from_slice(&value_full);

    let _test_data = (from, to, value, caller, input);
});
