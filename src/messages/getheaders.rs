use crate::{errores::NodoBitcoinError, common::utils_bytes::parse_varint};

use super::messages_header::make_header;

/// Representa un mensaje GetHeaders del protocolo Bitcoin
///
/// # Campos
/// * version: versión del protocolo Bitcoin, la misma enviada en el mensaje Version
/// * num_hashes: el número de hashes de headers que se proveen en el mensaje
/// * start_block_hash: uno o mas hashes de headers
/// * stop_block_hash: el hash del header del último heeader que está siendo pedido
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

    pub fn deserealize(bytes: &[u8]) -> Result<GetHeadersMessage, NodoBitcoinError> {
        let mut offset = 0;
        let version = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 4;

        let (size_bytes, num_hashes) = parse_varint(bytes);

        offset += size_bytes;

        let start_block_hash = &bytes[offset..offset + 32];
        offset += 32;

        let end_block_hash = &bytes[offset..offset + 32];

        let msg = GetHeadersMessage {
            version,
            num_hashes: num_hashes as u8,
            start_block_hash: start_block_hash.try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
            end_block_hash: end_block_hash.try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?
        };

        Ok(msg)
    }
}
