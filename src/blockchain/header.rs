use std::io::Write;

use crate::errores::NodoBitcoinError;

const HEADER_SIZE: usize = 80;

/// A struct representing a Bitcoin Header
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/block_chain.html
///
/// # Fields
///
/// * `id` - The unique identifier of the transaction.
/// * `version` - The version number of the transaction.
/// * `previous_block_hash` - The hash of the previous block in the chain.
/// * `merkle_root_hash` - The Merkle root hash of the transactions in the block.
/// * `time` - The Unix timestamp of the block's creation.
/// * `n_bits` - The compressed target difficulty of the block in compact format.
/// * `nonce` - A random number used in the mining process to try and find a valid block hash.
#[derive(Debug, PartialEq)]
pub struct BlockHeader {
    id: usize,
    version: i32,
    previous_block_hash: [u8; 32],
    merkle_root_hash: [u8; 32],
    time: u32,
    n_bits: u32,
    nonce: u32,
}

impl BlockHeader {
    pub fn _serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();

        bytes
            .write_all(&(self.version).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&self.previous_block_hash)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&self.merkle_root_hash)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.time).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.n_bits).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.nonce).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<BlockHeader, NodoBitcoinError> {
        if block_bytes.len() != HEADER_SIZE {
            return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
        }
        
        let id = 1;
        let mut offset = 0;

        let version = i32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes).unwrap(),
        );
        offset += 4;

        let mut previous_block_hash = [0u8; 32];
        previous_block_hash.copy_from_slice(&block_bytes[offset..offset + 32]);
        offset += 32;

        let mut merkle_root_hash = [0u8; 32];
        merkle_root_hash.copy_from_slice(&block_bytes[offset..offset + 32]);
        offset += 32;

        let time = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes1).unwrap(),
        );
        offset += 4;

        let n_bits = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes2).unwrap(),
        );
        offset += 4;

        let nonce = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes3).unwrap(),
        );

        Ok(BlockHeader {
            id,
            version,
            previous_block_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
        })
    }
}

fn _bytes_to_string(bytes: &[u8]) -> Result<String, NodoBitcoinError> {
    if let Ok(string) = String::from_utf8(bytes.to_vec()) {
        return Ok(string);
    }
    Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_serialize() {
//         let block_header = BlockHeader {
//             id: 0,
//             version: 1,
//             previous_block_hash: String::from("12345678901234567890123456789012"),
//             merkle_root_hash: String::from("12345678901234567890123456789012"),
//             time: 123456789,
//             n_bits: 123456789,
//             nonce: 123456789,
//         };

//         let result_serialized = block_header._serialize();
//         assert!(result_serialized.is_ok());

//         let serialized = result_serialized.unwrap();

//         assert_eq!(serialized.len(), 80);
//         assert_eq!(serialized[0..4], [1, 0, 0, 0]);
//         assert_eq!(
//             &serialized[4..36],
//             "12345678901234567890123456789012".as_bytes()
//         );
//         assert_eq!(
//             &serialized[36..68],
//             "12345678901234567890123456789012".as_bytes()
//         );
//         assert_eq!(serialized[68..72], [21, 205, 91, 7]);
//         assert_eq!(serialized[72..76], [21, 205, 91, 7]);
//         assert_eq!(serialized[76..80], [21, 205, 91, 7]);
//     }

//     #[test]
//     fn test_deserialize() {
//         let block_bytes = [
//             //version
//             1, 0, 0, 0, // previous block
//             49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
//             51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //merkle root
//             49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
//             51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //time
//             21, 205, 91, 7, // n bites
//             21, 205, 91, 7, //nonce
//             21, 205, 91, 7,
//         ];

//         let result_block_header = BlockHeader::deserialize(&block_bytes);
//         assert!(result_block_header.is_ok());

//         let block_header = result_block_header.unwrap();

//         assert_eq!(block_header.version, 1);
//         assert_eq!(
//             block_header.previous_block_hash,
//             String::from("12345678901234567890123456789012")
//         );
//         assert_eq!(
//             block_header.merkle_root_hash,
//             String::from("12345678901234567890123456789012")
//         );
//         assert_eq!(block_header.time, 123456789);
//         assert_eq!(block_header.n_bits, 123456789);
//         assert_eq!(block_header.nonce, 123456789);
//     }
// }
