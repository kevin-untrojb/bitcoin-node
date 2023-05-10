use std::io::Write;

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

use crate::errores::NodoBitcoinError;

fn string_to_bytes(s: String, fixed_size: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    match bytes.len() < fixed_size {
        true => bytes.resize(fixed_size, 0),
        false => bytes.truncate(fixed_size),
    }
    bytes
}

pub fn make_header(testnet: bool, command: String, payload: &Vec<u8>) -> Result<Vec<u8>, NodoBitcoinError>{
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

    result.write_all(&magic).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    result.write_all(&string_to_bytes(command, 12)).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    result.write_all(&payload_size.to_le_bytes()).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    result.write_all(checksum).map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

    Ok(result)
}