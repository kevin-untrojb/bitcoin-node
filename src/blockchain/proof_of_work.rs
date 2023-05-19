use bitcoin_hashes::{sha256d, Hash};

use crate::errores::NodoBitcoinError;

use super::blockheader::BlockHeader;

fn _pow_validation(header: &BlockHeader) -> Result<bool, NodoBitcoinError> {
    let target_difficulty = _calculate_target(header.n_bits);
    let header_bytes = header.serialize()?;
    _is_valid_pow(&header_bytes, target_difficulty)
}

fn _calculate_target(bits: u32) -> u32 {
    let exp = (bits >> 24) as i32;
    let coeff = bits & 0x00FFFFFF; // Este mapeo ya obtiene los 3 bytes menos significativos, o sea un número de 24 bits en little endian
    let target = (coeff as f64) * 2f64.powf((exp - 3) as f64);
    target as u32
}

fn _calculate_hash(data: &[u8]) -> [u8; 32] {
    let hash = sha256d::Hash::hash(&data);
    hash.as_byte_array().clone()
}

fn _is_valid_pow(data: &[u8], target_difficulty: u32) -> Result<bool, NodoBitcoinError> {
    let input = data.to_vec();
    let hash = _calculate_hash(&input);
    let bytes_from_hash = hash[28..32]
        .try_into()
        .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?;
    let hash_as_u32 = u32::from_le_bytes(bytes_from_hash);
    Ok(hash_as_u32 < target_difficulty)
}

#[cfg(test)]
mod tests {
    use crate::blockchain::{blockheader::BlockHeader, proof_of_work::_pow_validation};

    fn blockheader_genesis() -> [u8; 80] {
        [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b,
            0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3,
            0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a, 0x29, 0xab,
            0x5f, 0x49, 0xff, 0xff, 0x00, 0x1d, 0x1d, 0xac, 0x2b, 0x7c,
        ]
    }

    fn blockheader_for_test() -> BlockHeader {
        let block_header_bytes = blockheader_genesis();
        let lock_header_result = BlockHeader::deserialize(&block_header_bytes);
        assert!(lock_header_result.is_ok());

        lock_header_result.unwrap()
    }

    #[test]
    fn test_pow() {
        // pruebo con el BlokHeader del bloque génesis
        let mut block_header = blockheader_for_test();
        let validation_result = _pow_validation(&block_header);
        assert!(validation_result.is_ok());

        let is_valid = validation_result.unwrap();
        assert!(is_valid);

        // // le cambio el nonce para que tenga un hash válido
        // block_header.nonce = 0xFFFFFFFF;
        // let validation_result = _pow_validation(&block_header);
        // assert!(validation_result.is_ok());

        // let is_valid = validation_result.unwrap();
        // assert!(is_valid);

        // // le cambio el nonce para que tenga un hash inválido
        // block_header.nonce = 967295 + 1; // 2083236893 + 1;
        // let validation_result = _pow_validation(&block_header);
        // assert!(validation_result.is_ok());

        // let is_valid = validation_result.unwrap();
        // assert!(is_valid);

        // le cambio el n_bits para que tenga un hash válido
        block_header.n_bits = 0x00000001;
        let validation_result = _pow_validation(&block_header);
        assert!(validation_result.is_ok());

        let is_valid = validation_result.unwrap();
        assert!(!is_valid);
    }
}
