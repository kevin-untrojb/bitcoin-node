use crate::errores::NodoBitcoinError;

pub fn bytes_to_string(bytes: &[u8]) -> Result<String, NodoBitcoinError> {
    if let Ok(string) = String::from_utf8(bytes.to_vec()) {
        return Ok(string);
    }
    Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
}