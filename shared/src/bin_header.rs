//! [`bincode`] rawdogs binary, so I need a magic string to identify when running precompiled IR
//! files.

const MAGIC_STR: &[u8; 5] = b"ELSIE";

#[derive(PartialEq, Eq)]
pub enum FileType {
    Text,
    Binary,
}

pub fn binary_ir(ir: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(MAGIC_STR.len() + ir.len());

    out.extend_from_slice(MAGIC_STR);
    out.extend_from_slice(ir);

    out
}

pub fn detect_ir(ir: &[u8]) -> FileType {
    if &ir[0..5] == MAGIC_STR {
        FileType::Binary
    } else {
        FileType::Text
    }
}

pub fn get_ir(ir: &[u8]) -> &[u8] {
    if detect_ir(ir) == FileType::Text {
        unreachable!("you fucked up")
    } else {
        &ir[MAGIC_STR.len()..]
    }
}
