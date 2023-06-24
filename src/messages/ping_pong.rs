use crate::errores::NodoBitcoinError;

use super::messages_header::make_header;

/// Crea un mensaje pong que solamente tiene el header, sin payload
pub fn make_pong(bytes: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let msg = make_header("pong".to_string(), &bytes.to_vec())?;
    Ok(msg)
}
