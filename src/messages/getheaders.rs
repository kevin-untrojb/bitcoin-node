use bitcoin_hashes::{sha256d, Hash};

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
            end_block_hash: end_block
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut payload = Vec::new();
        let mut msg = Vec::new();
        let start_block = [0x00,0x00,0x00,0x00,0x09,0x33,0xea,0x01,0xad,0x0e,0xe9,0x84,0x20,0x97,0x79,0xba,0xae,0xc3,0xce,0xd9,0x0f,0xa3,0xf4,0x08,0x71,0x95,0x26,0xf8,0xd7,0x7f,0x49,0x43];
        //let start_block = "a35bdoca2f4a88c4eda6d2132378a5758dfcd6a43712000000000000000000";
        //println!("{:?}, {:?}, {:?}", self.version, self.num_hashes, start_block);

            let genesis_block_hash = [
        0xf9, 0x3f, 0xb5, 0x9b, 0xa9, 0x7e, 0xe3, 0x16,
        0x2b, 0x7d, 0xf2, 0x6b, 0xa5, 0x35, 0x9f, 0x4b,
        0x1d, 0x4c, 0x49, 0x4e, 0x5e, 0x95, 0x72, 0x2e,
        0xe3, 0x76, 0x66, 0x6a, 0xa2, 0x1b, 0x3b, 0xc1,
    ];
            payload.extend_from_slice(&self.version.to_le_bytes());
            payload.extend_from_slice(&self.num_hashes.to_le_bytes());
            payload.extend_from_slice(&start_block);
            payload.extend_from_slice(&self.end_block_hash);
        
        /*payload.extend_from_slice(&self.version.to_le_bytes());
        payload.extend_from_slice(&self.num_hashes.to_le_bytes());
        payload.extend_from_slice(&start_block);
        payload.extend_from_slice(&self.end_block_hash);*/

        /*
        payload.extend_from_slice(&[127, 17, 1, 0, 1, 163, 91, 208, 202, 47, 74, 136, 196, 237, 166, 210, 19, 35, 120, 165, 117, 141, 252, 214, 164, 55, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec());
        */

        println!("{:?}", payload);

        let header = make_header(true, "getheaders".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);

        Ok(msg)
    }
}
