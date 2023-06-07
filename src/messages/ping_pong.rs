use crate::errores::NodoBitcoinError;

use super::messages_header::make_header;

pub fn make_pong(bytes: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let nonce_bytes = &bytes[24..24 + 8].to_vec();
    let msg = make_header("pong".to_string(), nonce_bytes)?;
    Ok(msg)
}