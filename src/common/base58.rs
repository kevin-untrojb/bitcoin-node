use std::vec;

use crate::errores::NodoBitcoinError;

pub fn decode_base58(input: &str) -> Result<Vec<u8>, NodoBitcoinError> {
    let base_58 = bs58::decode(input);
    if let Ok(base_vec) = base_58.into_vec() {
        // quitar el primer byte
        let mut base_vec = base_vec[1..].to_vec();
        // quitar los ultimos 4 bytes
        base_vec.truncate(base_vec.len() - 4);
        return Ok(base_vec);
    } else {
        return Err(NodoBitcoinError::DecodeError);
    }
}

pub fn p2pkh_script_serialized(pubkey_hash: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut script = vec![0x76, 0xa9]; // OP_DUP OP_HASH160
    let length = pubkey_hash.len();
    if length < 75 {
        script.push(length as u8);
    } else if length > 75 && length < 0x100 {
        script.push(76);
        script.push(length as u8);
    } else if length >= 0x100 && length <= 520 {
        script.push(77);
        script.extend_from_slice(&length.to_le_bytes());
    } else {
        return Err(NodoBitcoinError::DecodeError);
    }
    script.extend_from_slice(pubkey_hash);
    script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
    Ok(script)
}
