use bitcoin_hashes::{sha256d, Hash};

use crate::{common::uint256::Uint256, errores::NodoBitcoinError};

use super::blockheader::BlockHeader;

pub fn pow_validation(header: &BlockHeader) -> Result<bool, NodoBitcoinError> {
    let target = _calculate_target(header);
    let header_bytes = header.serialize()?;
    _is_valid_pow(&header_bytes, target)
}

fn _calculate_target(blockheader: &BlockHeader) -> Uint256 {
    let n_bits_le = blockheader.n_bits.to_le_bytes();
    let n_bits = u32::from_be_bytes(n_bits_le);

    let bite_array = n_bits.to_le_bytes();
    let exp = bite_array[0] as u32;

    let coeff_bytes = n_bits & 0xFFFFFF00;
    let coeff_bytes_le = coeff_bytes.to_le_bytes();
    let coeff_u32 = u32::from_be_bytes(coeff_bytes_le);

    let coeff_256 = Uint256::_from_u32(coeff_u32);
    let value256 = Uint256::_from_u32(256);

    let potencia = value256._pow(exp - 3);
    coeff_256 * potencia
}

// fn _calculate_dificulty(bits: u32) -> Uint256 {
//     let target = _calculate_target_oreilly(bits);
//     let difficulty = (65535 as f64) * 256f64.powf(26 as f64) / target;
//     difficulty
// }

fn _calculate_hash(data: &[u8]) -> [u8; 32] {
    let hash = sha256d::Hash::hash(data);
    *hash.as_byte_array()
}

fn _calculate_proof(data: &[u8]) -> Uint256 {
    let hash = _calculate_hash(data);
    Uint256::_from_le_bytes(hash)
}

fn _is_valid_pow(data: &[u8], target_difficulty: Uint256) -> Result<bool, NodoBitcoinError> {
    let proof_of_work = _calculate_proof(data);
    Ok(proof_of_work < target_difficulty)
}

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::{
            blockheader::BlockHeader,
            proof_of_work::{
                _calculate_hash, _calculate_proof, _calculate_target, _is_valid_pow,
                pow_validation,
            },
        },
        common::uint256::Uint256,
    };

    fn bytes_target_oreilly() -> [u8; 32] {
        [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x3C, 0xE9, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    fn bytes_hash_oreilly() -> [u8; 32] {
        [
            0x23, 0x75, 0x04, 0x4d, 0x64, 0x6a, 0xd7, 0x35, 0x94, 0xdd, 0x0b, 0x37, 0xb1, 0x13,
            0xbe, 0xcd, 0xb0, 0x39, 0x64, 0x58, 0x4c, 0x9e, 0x7e, 0x00, 0x0, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    fn bytes_block_oreilly() -> [u8; 80] {
        [
            0x02, 0x00, 0x00, 0x20, 0x8e, 0xc3, 0x94, 0x28, 0xb1, 0x73, 0x23, 0xfa, 0x0d, 0xde,
            0xc8, 0xe8, 0x87, 0xb4, 0xa7, 0xc5, 0x3b, 0x8c, 0x0a, 0x0a, 0x22, 0x0c, 0xfd, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5b, 0x07, 0x50, 0xfc, 0xe0, 0xa8,
            0x89, 0x50, 0x2d, 0x40, 0x50, 0x8d, 0x39, 0x57, 0x68, 0x21, 0x15, 0x5e, 0x9c, 0x9e,
            0x3f, 0x5c, 0x31, 0x57, 0xf9, 0x61, 0xdb, 0x38, 0xfd, 0x8b, 0x25, 0xbe, 0x1e, 0x77,
            0xa7, 0x59, 0xe9, 0x3c, 0x01, 0x18, 0xa4, 0xff, 0xd7, 0x1d,
        ]
    }

    fn blockheader_test_oreilly() -> BlockHeader {
        blockheader_for_test(bytes_block_oreilly())
    }

    fn blockheader_for_test(block_header_bytes: [u8; 80]) -> BlockHeader {
        let lock_header_result = BlockHeader::deserialize(&block_header_bytes);
        assert!(lock_header_result.is_ok());
        lock_header_result.unwrap()
    }

    #[test]
    fn test_pow_ok() {
        // pruebo con el BlokHeader del bloque génesis
        let block_header = blockheader_test_oreilly();

        let validation_result = pow_validation(&block_header);
        assert!(validation_result.is_ok());

        let is_valid = validation_result.unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_pow_no_ok_change_nonce() {
        // pruebo con el BlokHeader del bloque génesis
        let mut block_header = blockheader_test_oreilly();
        // le cambio el nonce para que no pase la pow
        block_header.nonce = 0xFFFFFFFF;
        let validation_result = pow_validation(&block_header);
        assert!(validation_result.is_ok());

        let is_valid = validation_result.unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_pow_no_ok_change_n_bits() {
        // pruebo con el BlokHeader del bloque génesis
        let mut block_header = blockheader_test_oreilly();
        // le cambio el n_bits para que no pase la pow
        block_header.n_bits = 0xFFFFFFFF;
        let validation_result = pow_validation(&block_header);
        assert!(validation_result.is_ok());

        let is_valid = validation_result.unwrap();
        assert!(!is_valid);
    }

    // #[test]
    // fn test_calculate_dificulty() {
    //     let n_bits: u32 = 0xe93c0118;
    //     let dificulty_ok: f64 = 888171856257.3206;
    //     let difficulty = _calculate_dificulty(n_bits);
    //     assert_eq!(difficulty, dificulty_ok);
    // }

    #[test]
    fn test_calculate_proof() {
        let header_bytes_array = bytes_block_oreilly();
        let proof = _calculate_proof(&header_bytes_array);
        let proof_ok = Uint256::_from_le_bytes(bytes_hash_oreilly()); // la proof es el hash del bloque en little endian
        assert_eq!(proof, proof_ok);
    }

    #[test]
    fn test_calculate_target() {
        let block_header = blockheader_test_oreilly();
        let target_bl = _calculate_target(&block_header);
        let target_ok = Uint256::_from_bytes(bytes_target_oreilly());
        assert_eq!(target_bl, target_ok);
    }

    #[test]
    fn test_calculate_hash() {
        let block_header = blockheader_test_oreilly();
        let header_bytes_result = block_header.serialize();
        assert!(header_bytes_result.is_ok());

        let header_bytes_vec = header_bytes_result.unwrap();
        let header_bytes_array: &[u8] = header_bytes_vec.as_slice();

        let hash = _calculate_hash(&header_bytes_array);

        let hash_ok = bytes_hash_oreilly();
        assert_eq!(hash, hash_ok);
    }

    #[test]
    fn test_is_valid_pow() {
        let target = Uint256::_from_bytes(bytes_target_oreilly());
        let header = bytes_block_oreilly();

        let is_valid_pow_result = _is_valid_pow(&header, target);
        assert!(is_valid_pow_result.is_ok());

        let is_valid_pow = is_valid_pow_result.unwrap();
        assert!(is_valid_pow);
    }

    #[test]
    fn test_is_invalid_pow() {
        let target = Uint256::_from_bytes(bytes_target_oreilly());
        let header = bytes_block_oreilly();

        let is_valid_pow_result = _is_valid_pow(&header, target);
        assert!(is_valid_pow_result.is_ok());

        let is_valid_pow = is_valid_pow_result.unwrap();
        assert!(is_valid_pow);
    }
}
