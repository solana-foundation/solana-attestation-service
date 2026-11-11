extern crate alloc;

use alloc::vec::Vec;
use pinocchio::error::ProgramError;

use crate::require_len;

// Serializes an array of bytes to Vector representation by prepending array length.
pub fn to_serialized_vec(data: &[u8]) -> Vec<u8> {
    [(data.len() as u32).to_le_bytes().as_slice(), data].concat()
}

pub struct LayoutArgs<'a> {
    pub layout: &'a [u8],
    pub field_names_count: u32,
    pub field_names_bytes: &'a [u8],
}

#[inline(always)]
pub fn parse_layout_args(data: &[u8], mut offset: usize) -> Result<LayoutArgs<'_>, ProgramError> {
    require_len!(data, offset + 4);
    let layout_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;

    require_len!(data, offset + layout_len);
    let layout = &data[offset..offset + layout_len];
    offset += layout_len;

    require_len!(data, offset + 4);
    let field_names_count = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    offset += 4;

    let mut byte_len = 0;
    for _ in 0..field_names_count {
        let start = offset + byte_len;
        let end = start + 4;
        require_len!(data, end);

        let name_len = u32::from_le_bytes(data[start..end].try_into().unwrap()) as usize;
        byte_len += 4 + name_len;
    }

    require_len!(data, offset + byte_len);
    let field_names_bytes = &data[offset..offset + byte_len];

    Ok(LayoutArgs { layout, field_names_count, field_names_bytes })
}
