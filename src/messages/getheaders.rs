use crate::errores::NodoBitcoinError;

use super::messages_header::make_header;

pub struct GetHeadersMessage {
    version: u32,
    num_hashes: u8,
    start_block_hash: [u8; 32],
    end_block_hash: [u8; 32],
}

impl GetHeadersMessage {
    /// Crea un mensaje GetHeadersMessage y lo devuelve
    pub fn new(
        version: u32,
        num_hashes: u8,
        start_block: [u8; 32],
        end_block: [u8; 32],
    ) -> GetHeadersMessage {
        GetHeadersMessage {
            version,
            num_hashes,
            start_block_hash: start_block,
            end_block_hash: end_block,
        }
    }

    /// Serializa un mensaje Get Headers y devuelve sus bytes
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut payload = Vec::new();
        let mut msg = Vec::new();
        payload.extend_from_slice(&self.version.to_le_bytes());
        payload.extend_from_slice(&self.num_hashes.to_le_bytes());
        payload.extend_from_slice(&self.start_block_hash);
        payload.extend_from_slice(&self.end_block_hash);

        let header = make_header("getheaders".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);

        Ok(msg)
    }
}
