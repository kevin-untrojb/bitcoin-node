use super::messages_header::make_header;
use crate::{common::utils_bytes::parse_varint, errores::NodoBitcoinError};

const MSG_BLOCK: u32 = 2;
const MSG_TX: u32 = 1;

pub struct Inventory {
    inv_type: u32,
    hash: Vec<u8>,
}

pub struct GetDataMessage {
    count: u8,
    inventory: Vec<Inventory>,
}

impl GetDataMessage {
    pub fn new(count: u8, hash: [u8; 32]) -> GetDataMessage {
        let inventory = Inventory {
            inv_type: MSG_BLOCK,
            hash: hash.to_vec(),
        };

        GetDataMessage {
            count,
            inventory: vec![inventory],
        }
    }

    pub fn new_for_tx(inv_msg: &Vec<u8>) -> Result<GetDataMessage, NodoBitcoinError> {
        let mut inventory = Vec::new();
        let (size_bytes, count) = parse_varint(&inv_msg);

        for i in 0..count {
            let offset = (i * 36) + size_bytes;
            let inv_type = u32::from_le_bytes([
                inv_msg[offset],
                inv_msg[offset + 1],
                inv_msg[offset + 2],
                inv_msg[offset + 3],
            ]);

            if inv_type != MSG_TX {
                return Err(NodoBitcoinError::NoEsTransaccion);
            }

            inventory.push(Inventory {
                inv_type,
                hash: inv_msg[offset + 4..].to_vec(),
            });
        }

        Ok(GetDataMessage {
            count: count as u8,
            inventory: inventory,
        })
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
