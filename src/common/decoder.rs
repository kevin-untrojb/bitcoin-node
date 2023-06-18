use std::vec;

use crate::errores::NodoBitcoinError;

use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey};

pub fn point_sec(secret_key_hexa_bytes: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    // Crear una instancia de Secp256k1
    let secp = Secp256k1::new();

    let secret_key =
        SecretKey::from_slice(secret_key_hexa_bytes).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let serializada = public_key.serialize().to_vec();
    Ok(serializada)
}

pub fn signature_der(secret_key_hexa_bytes: &[u8], message: &[u8]) -> Signature {
    let secp = Secp256k1::new();
    let secret_key =
        SecretKey::from_slice(secret_key_hexa_bytes).expect("32 bytes, within curve order");
    //let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let message = Message::from_slice(message).unwrap();

    secp.sign_ecdsa(&message, &secret_key)
}

/// Convert a WIF private key to hexadecimal format
/// Private Key WIF Compressed 52 characters base58, starts with a 'K' or 'L'
/// a
/// Private Key Hexadecimal Format (64 characters [0-9A-F])
pub fn wif_to_hex(wif: &str) -> Result<Vec<u8>, NodoBitcoinError> {
    // Decode the base58-encoded WIF compressed private key
    let decoded = bs58::decode(wif).into_vec().unwrap();

    // Ensure the decoded length is valid
    if decoded.len() < 34 {
        return Err(NodoBitcoinError::DecodeError);
    }

    // Extract the 32-byte private key from the decoded WIF
    let private_key = &decoded[1..33];

    // Convert the private key to hexadecimal format
    let hex = private_key.to_vec();

    Ok(hex)
}

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

pub fn script_serialized(key: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut script = vec![];
    let length = key.len();
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
    script.extend_from_slice(key);
    Ok(script)
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
    use crate::common::decoder::{point_sec, signature_der, wif_to_hex};

    use super::{decode_base58, p2pkh_script_serialized};

    #[test]
    fn test_compress_public_key() {
        let secret_key_bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x84, 0x5f, 0xed,
        ];

        let expected_key = [
            0x03, 0x93, 0x55, 0x81, 0xE5, 0x2C, 0x35, 0x4C, 0xD2, 0xF4, 0x84, 0xFE, 0x8E, 0xD8,
            0x3A, 0xF7, 0xA3, 0x09, 0x70, 0x05, 0xB2, 0xF9, 0xC6, 0x0B, 0xFF, 0x71, 0xD3, 0x5B,
            0xD7, 0x95, 0xF5, 0x4B, 0x67,
        ];

        let sec = point_sec(&secret_key_bytes);
        assert!(sec.is_ok());

        let sec = sec.unwrap();
        assert_eq!(sec, expected_key);
    }

    #[test]
    fn test_wif_to_hex() {
        let wif = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw";
        let hex = wif_to_hex(wif);
        assert!(hex.is_ok());

        let hex = hex.unwrap();
        let hex = hex.as_slice();
        let expected_hex: [u8; 32] = [
            0x6F, 0x44, 0x20, 0x56, 0xE4, 0x0A, 0x68, 0x54, 0xDD, 0xAB, 0x67, 0xAB, 0xBD, 0xE3,
            0xFB, 0x4A, 0x19, 0xAC, 0xE2, 0xC2, 0xC4, 0xF8, 0x44, 0x23, 0x85, 0xED, 0x7F, 0x29,
            0x34, 0x23, 0x95, 0xC6,
        ];

        assert_eq!(hex, expected_hex);
    }

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

    #[test]
    fn test_signature() {
        let message: [u8; 32] = [
            0x27, 0xe0, 0xc5, 0x99, 0x4d, 0xec, 0x78, 0x24, 0xe5, 0x6d, 0xec, 0x6b, 0x2f, 0xcb,
            0x34, 0x2e, 0xb7, 0xcd, 0xb0, 0xd0, 0x95, 0x7c, 0x2f, 0xce, 0x98, 0x82, 0xf7, 0x15,
            0xe8, 0x5d, 0x81, 0xa6,
        ];
        let secret_key_bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x84, 0x5f, 0xed,
        ];
        let sig = signature_der(&secret_key_bytes, &message);

        let expected_der = [
            0x30, 0x44, 0x02, 0x20, 0x7d, 0xb2, 0x40, 0x2a, 0x33, 0x11, 0xa3, 0xb8, 0x45, 0xb0,
            0x38, 0x88, 0x5e, 0x3d, 0xd8, 0x89, 0xc0, 0x81, 0x26, 0xa8, 0x57, 0x0f, 0x26, 0xa8,
            0x44, 0xe3, 0xe4, 0x04, 0x9c, 0x48, 0x2a, 0x11, 0x02, 0x20, 0x10, 0x17, 0x8c, 0xdc,
            0xa4, 0x12, 0x9e, 0xac, 0xbe, 0xab, 0x7c, 0x44, 0x64, 0x8b, 0xf5, 0xac, 0x1f, 0x9c,
            0xac, 0x21, 0x7c, 0xd6, 0x09, 0xd2, 0x16, 0xec, 0x2e, 0xbc, 0x8d, 0x24, 0x2c, 0x0a,
        ];

        let binding = sig.serialize_der().clone();
        let serialized_bytes = binding.as_ref();
        assert_eq!(serialized_bytes, &expected_der);
    }
}
