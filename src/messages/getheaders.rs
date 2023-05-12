use crate::errores::NodoBitcoinError;

use super::header::make_header;

pub struct GetHeadersMessage {
    version: u32,
    num_hashes: u8,
    start_block_hash: [u8; 32],
    end_block_hash: [u8; 32],
}

impl GetHeadersMessage {
    pub fn new(
        version: u32,
        num_hashes: u8,
        start_block: [u8; 32],
        end_block: [u8; 32],
    ) -> GetHeadersMessage {
        GetHeadersMessage {
            version: version,
            num_hashes: num_hashes,
            start_block_hash: start_block,
            end_block_hash: end_block,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut payload = Vec::new();
        let mut msg = Vec::new();

        payload.extend_from_slice(&(self.version).to_le_bytes());
        payload.extend_from_slice(&(self.num_hashes).to_le_bytes());
        payload.extend_from_slice(&(self.start_block_hash));
        payload.extend_from_slice(&(self.end_block_hash));
        /*
        payload.extend_from_slice(&[127, 17, 1, 0, 1, 163, 91, 208, 202, 47, 74, 136, 196, 237, 166, 210, 19, 35, 120, 165, 117, 141, 252, 214, 164, 55, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec());
        payload.extend_from_slice(&[0x12, 0x71, 0x71, 0x00]);
        payload.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        payload.extend_from_slice(&[0x00; 32]);
        payload.extend_from_slice(&[0x00; 32]);
        */

        println!("{:?}", payload.len());

        let header = make_header(true, "getheaders".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);

        Ok(msg)
    }
}
