use crate::errores::NodoBitcoinError;

use super::messages_header::make_header;

/// Crea un mensaje block que contenga el bloque deserealizado
pub fn make_block(payload: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut msg = Vec::new();
    let header = make_header("block".to_string(), &payload.to_vec())?;
    msg.extend_from_slice(&header);
    msg.extend_from_slice(payload);
    Ok(msg)
}
