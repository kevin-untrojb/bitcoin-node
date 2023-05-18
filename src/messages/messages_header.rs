use std::io::Write;
use std::str::from_utf8;

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;

use crate::errores::NodoBitcoinError;

const MAGIC_NUMBER_TESTNET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];

fn string_to_bytes(s: String, fixed_size: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    match bytes.len() < fixed_size {
        true => bytes.resize(fixed_size, 0),
        false => bytes.truncate(fixed_size),
    }
    bytes
}

pub fn make_header(command: String, payload: &Vec<u8>) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut result = Vec::new();
    let magic = MAGIC_NUMBER_TESTNET; //Obtenerlo de config despues

    let payload_size = payload.len() as u32;
    let hash = sha256d::Hash::hash(&payload);
    let checksum = &hash[..4];

    result
        .write_all(&magic)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    result
        .write_all(&string_to_bytes(command, 12))
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    result
        .write_all(&payload_size.to_le_bytes())
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
    result
        .write_all(checksum)
        .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

    Ok(result)
}

pub fn check_header(header: &[u8]) -> Result<(String, usize), NodoBitcoinError> {
    let mut offset = 0;

    let magic_num = &header[offset..offset + 4];

    if magic_num != MAGIC_NUMBER_TESTNET {
        println!("magic number error");
        return Err(NodoBitcoinError::MagicNumberIncorrecto);
    }

    offset += 4;
    let command = from_utf8(&header[offset..offset + 12])
        .unwrap()
        .trim_end_matches('\0')
        .to_string();

    offset += 12;

    let payload_len = u32::from_le_bytes(
        header[offset..offset + 4]
            .try_into()
            .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
    ) as usize;

    Ok((command, payload_len))
}
