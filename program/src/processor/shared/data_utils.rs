extern crate alloc;

use alloc::vec::Vec;

// Serializes an array of bytes to Vector representation by prepending array length.
pub fn to_serialized_vec(data: &[u8]) -> Vec<u8> {
    [(data.len() as u32).to_le_bytes().as_slice(), data].concat()
}
