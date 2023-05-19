use super::messages_header::make_header;
use crate::{errores::NodoBitcoinError};

const MSG_BLOCK: u32 = 2;

pub struct Inventory {
    inv_type: u32,
    hash: [u8; 32],
}

pub struct GetDataMessage {
    count: u8,
    inventory: Vec<Inventory>,
}

impl GetDataMessage {
    pub fn new(count: u8, hash: [u8; 32]) -> GetDataMessage {
        let inventory = Inventory {
            inv_type: MSG_BLOCK,
            hash,
        };

        GetDataMessage {
            count,
            inventory: vec![inventory],
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut payload = Vec::new();
        let mut msg = Vec::new();

        payload.extend_from_slice(&(self.count).to_le_bytes());
        for inventory in &self.inventory {
            payload.extend_from_slice(&inventory.inv_type.to_le_bytes());
            payload.extend_from_slice(&inventory.hash);
        }

        let header = make_header("getdata".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);
        Ok(msg)
    }
}
