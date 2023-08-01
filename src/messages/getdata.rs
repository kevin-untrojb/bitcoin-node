use super::messages_header::make_header;
use crate::{common::utils_bytes::parse_varint, errores::NodoBitcoinError};

const MSG_BLOCK: u32 = 2;
const MSG_TX: u32 = 1;

/// Representa un inventario del protocolo Bitcoin
///
/// # Campos
/// * inv_type: el tipo de objeto al que pertenece el hash
/// * hash: hash SHA256(SHA256()) del objeto

pub struct Inventory {
    inv_type: u32,
    hash: Vec<u8>,
}

/// Representa un mensaje GetData del protocolo Bitcoin
///
/// # Campos
/// * count: cantidad de inventarios
/// * inventory: vector de inventarios
pub struct GetDataMessage {
    count: u8,
    inventory: Vec<Inventory>,
}

impl GetDataMessage {
    /// Crea un GetDataMessage donde el inventory es del tipo BLOCK.
    /// O sea que este mensaje servirá para pedir bloques.
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

    /// Crea un GetDataMessage para cuando se recibe una transacción a partir del mensaje inv
    /// Devuelve un struct del mensaje GetDataMessage
    pub fn new_for_tx(inv_msg: &[u8]) -> Result<GetDataMessage, NodoBitcoinError> {
        let mut inventory = Vec::new();
        let (size_bytes, count) = parse_varint(inv_msg);

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
            inventory,
        })
    }

    /// Devuelve los hashes de los inventarios
    pub fn get_hashes(&self) -> Vec<Vec<u8>> {
        let mut hashes = Vec::new();
        for inventory in &self.inventory {
            hashes.push(inventory.hash.clone());
        }
        hashes
    }

    /// Serializa el mensaje GetData y devuelve los bytes del mismo
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

    // deseralizar el getdata, recibiendo como parámetro los bytes del mensaje sin header
    pub fn deserealize(bytes: &[u8]) -> Result<GetDataMessage, NodoBitcoinError> {
        let mut offset = 0;
        let (size_bytes, count) = parse_varint(bytes);
        offset += size_bytes;
        let mut inventory = Vec::new();

        for _ in 0..count {
            let inv_type = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
            );
            offset += 4;
            let hash = bytes[offset..offset + 32].to_vec();
            offset += 32;

            inventory.push(Inventory { inv_type, hash });
        }

        Ok(GetDataMessage {
            count: count as u8,
            inventory,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::{getdata::GetDataMessage, messages_header::check_header};

    #[test]
    fn test_getdata_deserialize() {
        let hash_header: [u8; 32] = [
            0xc1, 0x17, 0xea, 0x8e, 0xc8, 0x28, 0x34, 0x2f, 0x4d, 0xfb, 0x0a, 0xd6, 0xbd, 0x14,
            0x0e, 0x03, 0xa5, 0x07, 0x20, 0xec, 0xe4, 0x01, 0x69, 0xee, 0x38, 0xbd, 0xc1, 0x5d,
            0x9e, 0xb6, 0x4c, 0xf5,
        ];
        let get_data_original = GetDataMessage::new(1, hash_header);
        let get_data_message = get_data_original.serialize();
        assert!(get_data_message.is_ok());

        let get_data_message = get_data_message.unwrap();

        let check_header = check_header(&get_data_message);
        assert!(check_header.is_ok());

        let (command, response_get_data) = check_header.unwrap();
        assert!(command == "getdata");
        assert_eq!(response_get_data, 37);

        let get_data_message_serialized = &get_data_message[24..];

        let deserealized = GetDataMessage::deserealize(&get_data_message_serialized);
        assert!(deserealized.is_ok());

        let deserealized = deserealized.unwrap();

        assert_eq!(deserealized.count, get_data_original.count);
        assert_eq!(
            deserealized.inventory[0].inv_type,
            get_data_original.inventory[0].inv_type
        );
        assert_eq!(
            deserealized.inventory[0].hash,
            get_data_original.inventory[0].hash
        );
    }

    #[test]
    fn test_get_hashes() {
        let hash_header: [u8; 32] = [
            0xc1, 0x17, 0xea, 0x8e, 0xc8, 0x28, 0x34, 0x2f, 0x4d, 0xfb, 0x0a, 0xd6, 0xbd, 0x14,
            0x0e, 0x03, 0xa5, 0x07, 0x20, 0xec, 0xe4, 0x01, 0x69, 0xee, 0x38, 0xbd, 0xc1, 0x5d,
            0x9e, 0xb6, 0x4c, 0xf5,
        ];
        let get_data_original = GetDataMessage::new(1, hash_header);
        let hashes = get_data_original.get_hashes();
        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0], hash_header.to_vec());
    }
}
