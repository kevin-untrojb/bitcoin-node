use crate::errores::NodoBitcoinError;

use super::header::make_header;

struct InventoryVector {
    inv_type: u32,
    hash: [u8; 32],
}

pub struct GetDataMessage {
    count: u8,
    inventory: Vec<InventoryVector>
}

impl GetDataMessage {
    pub fn new(inventory: Vec<InventoryVector>) -> GetDataMessage {
        GetDataMessage {
            count: inventory.len() as u8,
            inventory
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

        let header = make_header(true, "getdata".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);

    }
}