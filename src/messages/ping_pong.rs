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

pub fn make_ping(nonce: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut msg = Vec::new();
    let header = make_header("ping".to_string(), &nonce.to_vec())?;
    msg.extend_from_slice(&header);
    msg.extend_from_slice(nonce);
    Ok(msg)
}