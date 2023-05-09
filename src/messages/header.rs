use std::io::Write;

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

fn string_to_bytes(s: String, fixed_size: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    match bytes.len() < fixed_size {
        true => bytes.resize(fixed_size, 0),
        false => bytes.truncate(fixed_size),
    }
    bytes
}

pub fn make_header(testnet: bool, command: String, payload: &Vec<u8>) -> Vec<u8>{
    let mut result = Vec::new();
    let magic;

    if testnet {
        magic = [0x0b, 0x11, 0x09, 0x07];
    }else{
        magic = [0x00, 0x00, 0x00, 0x00];
    }

    let payload_size = payload.len() as u32;
    let hash = sha256d::Hash::hash(&payload);
    let checksum = &hash[..4];

    result.write_all(&magic).unwrap();
    result.write_all(&string_to_bytes(command, 12)).unwrap();
    result.write_all(&payload_size.to_le_bytes()).unwrap();
    result.write_all(checksum).unwrap();

    result

}