use super::proof_of_work;
use crate::errores::NodoBitcoinError;
use bitcoin_hashes::{sha256d, Hash};
use std::{fmt, io::Write};

const HEADER_SIZE: usize = 80;

/// A struct representing a Bitcoin Header
/// ### Bitcoin Core References
/// <https://developer.bitcoin.org/reference/block_chain.html>
///
/// # Fields
///
/// * `version` - The version number of the transaction.
/// * `previous_block_hash` - The hash of the previous block in the chain.
/// * `merkle_root_hash` - The Merkle root hash of the transactions in the block.
/// * `time` - The Unix timestamp of the block's creation.
/// * `n_bits` - The compressed target difficulty of the block in compact format.
/// * `nonce` - A random number used in the mining process to try and find a valid block hash.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_block_hash: [u8; 32],
    pub merkle_root_hash: [u8; 32],
    pub time: u32,
    pub n_bits: u32,
    pub nonce: u32,
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f, "BlockHeader:\nversion: {:?}\nprevious_block_hash: {:?}\nmerkle_root_hash: {:?}\ntime: {:?}",
            self.version,
            self.previous_block_hash,
            self.merkle_root_hash,
            self.time
        )
    }
}

impl BlockHeader {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
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

        let mut offset = 0;

        let version = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
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
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 4;

        let n_bits = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 4;

        let nonce = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );

        Ok(BlockHeader {
            version,
            previous_block_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
        })
    }

    pub fn hash(&self) -> Result<[u8; 32], NodoBitcoinError> {
        let serialized = self.serialize()?;
        let hash = sha256d::Hash::hash(&serialized);
        Ok(*hash.as_byte_array())
    }

    pub fn _is_valid_pow(&self) -> bool {
        proof_of_work::pow_validation(self).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, mem};

    use crate::{blockchain::file::_leer_todos_headers, config};

    use super::*;

    #[test]
    fn test_serialize() {
        let block_header = BlockHeader {
            version: 1,
            previous_block_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5,
            ],
            merkle_root_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5,
            ],
            time: 123456789,
            n_bits: 123456789,
            nonce: 123456789,
        };

        let result_serialized = block_header.serialize();
        assert!(result_serialized.is_ok());

        let serialized = result_serialized.unwrap();

        assert_eq!(serialized.len(), 80);
        assert_eq!(serialized[0..4], [1, 0, 0, 0]);
        assert_eq!(
            &serialized[4..36],
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5
            ]
        );
        assert_eq!(
            &serialized[36..68],
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5
            ]
        );
        assert_eq!(serialized[68..72], [21, 205, 91, 7]);
        assert_eq!(serialized[72..76], [21, 205, 91, 7]);
        assert_eq!(serialized[76..80], [21, 205, 91, 7]);
    }

    #[test]
    fn test_deserialize() {
        let block_bytes = [
            //version
            1, 0, 0, 0, // previous block
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //merkle root
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //time
            21, 205, 91, 7, // n bites
            21, 205, 91, 7, //nonce
            21, 205, 91, 7,
        ];

        let result_block_header = BlockHeader::deserialize(&block_bytes);
        assert!(result_block_header.is_ok());

        let block_header = result_block_header.unwrap();

        assert_eq!(block_header.version, 1);
        assert_eq!(
            block_header.previous_block_hash,
            [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ]
        );
        assert_eq!(
            block_header.merkle_root_hash,
            [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ]
        );
        assert_eq!(block_header.time, 123456789);
        assert_eq!(block_header.n_bits, 123456789);
        assert_eq!(block_header.nonce, 123456789);
    }

    #[test]
    fn hash_headers() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        _ = config::inicializar(args);
        println!(
            "Empieza a las {}",
            chrono::offset::Local::now().format("%F %T")
        );
        // cargar todos los headers
        let headers_bytes = _leer_todos_headers();
        assert!(headers_bytes.is_ok());
        println!(
            "Leo todos los bytes a las {}",
            chrono::offset::Local::now().format("%F %T")
        );
        let headers_bytes = headers_bytes.unwrap();
        let total_bytes = headers_bytes.len();

        let mut headers = vec![];

        let mut offset = 0;

        let mut hash_map = HashMap::new();

        while offset < total_bytes {
            let bytes = &headers_bytes[offset..offset + 80];
            let header = BlockHeader::deserialize(bytes).unwrap();
            let hash = header.hash().unwrap();
            headers.push(header);
            hash_map.insert(hash, header);
            offset += 80;
        }
        println!(
            "Hash map terminado {}",
            chrono::offset::Local::now().format("%F %T")
        );
    }
}
