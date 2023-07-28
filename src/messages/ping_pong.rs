use crate::errores::NodoBitcoinError;

use super::messages_header::make_header;

/// Crea un mensaje pong que solamente tiene el header, sin payload
pub fn make_pong(bytes: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut msg = Vec::new();
    let header = make_header("pong".to_string(), &bytes.to_vec())?;
    msg.extend_from_slice(&header);
    msg.extend_from_slice(bytes);
    Ok(msg)
}

pub fn make_ping(nonce: &[u8; 8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut msg = Vec::new();
    let header = make_header("ping".to_string(), &nonce.to_vec())?;
    msg.extend_from_slice(&header);
    msg.extend_from_slice(nonce);
    Ok(msg)
}

pub fn get_nonce(msg: &[u8]) -> Result<[u8; 8], NodoBitcoinError> {
    let mut nonce = [0u8; 8];
    nonce.copy_from_slice(&msg[24..32]);
    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use crate::{
        common::utils_bytes::ping_nonce,
        messages::ping_pong::{get_nonce, make_ping, make_pong},
    };

    #[test]
    fn test_make_pong() {
        let nonce = ping_nonce();
        let msg = make_pong(&nonce);
        assert!(msg.is_ok());
        let msg = msg.unwrap();
        assert_eq!(msg.len(), 32);

        let binding = "pong".to_string();
        let pong_bytes = binding.as_bytes();
        assert_eq!(msg[4..8], *pong_bytes);
        assert_eq!(msg[24..32], nonce);
    }

    #[test]
    fn test_make_ping() {
        let nonce = ping_nonce();
        let msg = make_ping(&nonce);
        assert!(msg.is_ok());
        let msg = msg.unwrap();
        assert_eq!(msg.len(), 32);

        let binding = "ping".to_string();
        let ping_bytes = binding.as_bytes();
        assert_eq!(msg[4..8], *ping_bytes);
        assert_eq!(msg[24..32], nonce);
    }

    #[test]
    fn test_get_nonce() {
        // crear un vector de 8 bytes con 8 valores aleatorios
        let bytes = ping_nonce();
        let msg = make_ping(&bytes).unwrap();
        let nonce = get_nonce(&msg).unwrap();
        assert_eq!(nonce, bytes);
    }
}
