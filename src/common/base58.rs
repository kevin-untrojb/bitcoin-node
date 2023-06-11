use std::vec;

use crate::errores::NodoBitcoinError;

pub fn decode_base58(input: &str) -> Result<Vec<u8>, NodoBitcoinError> {
    let base_58 = bs58::decode(input);
    if let Ok(base_vec) = base_58.into_vec() {
        // quitar el primer byte
        let mut base_vec = base_vec[1..].to_vec();
        // quitar los ultimos 4 bytes
        base_vec.truncate(base_vec.len() - 4);
        Ok(base_vec)
    } else {
        Err(NodoBitcoinError::DecodeError)
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
    } else if (0x100..=520).contains(&length) {
        script.push(77);
        script.extend_from_slice(&length.to_le_bytes());
    } else {
        return Err(NodoBitcoinError::DecodeError);
    }
    script.extend_from_slice(pubkey_hash);
    script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::{decode_base58, p2pkh_script_serialized};

    #[test]
    fn test_decode_base58() {
        let decode_ok: [u8; 20] = [
            0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52, 0xc2, 0x01, 0x8e,
            0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f,
        ];

        let source = "mzx5YhAH9kNHtcN481u6WkjeHjYtVeKVh2";
        let result = decode_base58(source);

        assert_eq!(result.is_ok(), true);

        let decode = result.unwrap();
        let bytes_decoded = decode.as_slice();
        assert_eq!(bytes_decoded, decode_ok.as_ref());
    }

    #[test]
    fn test_decode_base58_no_ok() {
        let decode_ok: [u8; 20] = [
            0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52, 0xc2, 0x01, 0x8e,
            0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6c,
        ];

        let source = "mzx5YhAH9kNHtcN481u6WkjeHjYtVeKVh2";
        let result = decode_base58(source);

        assert_eq!(result.is_ok(), true);

        let decode = result.unwrap();
        let bytes_decoded = decode.as_slice();
        assert_ne!(bytes_decoded, decode_ok.as_ref());
    }

    #[test]
    fn test_p2pkh_script_serialized() {
        let p2pkh_ok: [u8; 25] = [
            0x76, 0xa9, 0x14, 0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52,
            0xc2, 0x01, 0x8e, 0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f, 0x88, 0xac,
        ];
        let source: [u8; 20] = [
            0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52, 0xc2, 0x01, 0x8e,
            0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f,
        ];

        let result = p2pkh_script_serialized(&source);
        assert_eq!(result.is_ok(), true);

        let script = result.unwrap();
        let bytes_script = script.as_slice();
        assert_eq!(bytes_script, p2pkh_ok.as_ref());
    }

    #[test]
    fn test_p2pkh_script_serialized_no_ok() {
        let p2pkh_ok: [u8; 25] = [
            0x76, 0xa9, 0x14, 0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52,
            0xc2, 0x01, 0x8e, 0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f, 0x88, 0xde,
        ];
        let source: [u8; 20] = [
            0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52, 0xc2, 0x01, 0x8e,
            0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f,
        ];

        let result = p2pkh_script_serialized(&source);
        assert_eq!(result.is_ok(), true);

        let script = result.unwrap();
        let bytes_script = script.as_slice();
        assert_ne!(bytes_script, p2pkh_ok.as_ref());
    }
}
