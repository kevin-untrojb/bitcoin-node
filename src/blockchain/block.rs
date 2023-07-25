use std::cmp::Ordering;
use std::fmt;
use std::io::Write;

use super::file::{_leer_algunos_blocks, _leer_primer_block, leer_todos_blocks};
use super::{blockheader::BlockHeader, transaction};
use crate::common::utils_bytes;
use crate::errores::NodoBitcoinError;
use crate::merkle_tree::merkle_root::MerkleRoot;
use transaction::Transaction;

/// A struct representing a Bitcoin Serialized Block
/// ### Bitcoin Core References
/// <https://developer.bitcoin.org/reference/block_chain.html#serialized-blocks>
///
/// # Fields
///
/// * `header` - The header of the block, which contains metadata such as the block's version, hash, and timestamp.
/// * `txns` - The transactions included in the block, represented as a vector of `Transaction` structs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SerializedBlock {
    pub header: BlockHeader,
    pub txns: Vec<Transaction>,
    pub txn_amount: usize,
}

impl fmt::Display for SerializedBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SerializedBlock[header: {:?}, txns: {:?}]",
            self.header, self.txns
        )
    }
}

impl SerializedBlock {
    pub fn deserialize(block_bytes: &[u8]) -> Result<SerializedBlock, NodoBitcoinError> {
        let mut offset = 0;
        if block_bytes.len() < 80 {
            return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
        }
        let header = BlockHeader::deserialize(&block_bytes[offset..offset + 80])?;
        offset += 80;
        let (txn_amount, txn_count) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += txn_amount;

        let mut txns = Vec::new();
        for _ in 0..txn_count {
            let trn = Transaction::deserialize(&block_bytes[offset..])?;
            offset += trn.size();
            txns.push(trn);
        }
        Ok(SerializedBlock {
            header,
            txns,
            txn_amount,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();

        let bytes_header = self.header.serialize()?;
        bytes
            .write_all(bytes_header.as_slice())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let tx_count_prefix = utils_bytes::from_amount_bytes_to_prefix(self.txn_amount);
        bytes
            .write_all(&(utils_bytes::build_varint_bytes(tx_count_prefix, self.txns.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let bytes_txns_array = self.txns.iter().map(|txn| txn.serialize());
        for bytes_txn in bytes_txns_array {
            bytes
                .write_all(bytes_txn?.as_slice())
                .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        }
        Ok(bytes)
    }

    pub fn _tx_proof_of_inclusion(&self, tx: &Transaction) -> bool {
        let tree = match self.local_merkle_tree() {
            Ok(tree) => tree,
            Err(_) => return false,
        };
        tree._proof_of_inclusion(tx)
    }

    pub fn local_merkle_tree(&self) -> Result<MerkleRoot, NodoBitcoinError> {
        let local_merkle = match MerkleRoot::from_block(self) {
            Ok(calculated_merkle) => calculated_merkle,
            Err(_) => return Err(NodoBitcoinError::NoSePuedeArmarElArbol),
        };
        Ok(local_merkle)
    }

    pub fn is_valid_merkle(&self) -> bool {
        let current_merkle = self.header.merkle_root_hash;
        let local_merkle = match self.local_merkle_tree() {
            Ok(calculated_merkle) => calculated_merkle,
            Err(_) => return false,
        };
        let binding = local_merkle.root_hash();
        let local_merkle_hash = binding.as_slice();
        current_merkle == local_merkle_hash
    }

    pub fn _read_first_block_from_file() -> Result<SerializedBlock, NodoBitcoinError> {
        let block_bytes = _leer_primer_block()?;
        SerializedBlock::deserialize(&block_bytes)
    }

    pub fn _read_n_blocks_from_file(
        cantidad: u32,
    ) -> Result<Vec<SerializedBlock>, NodoBitcoinError> {
        let block_bytes = _leer_algunos_blocks(cantidad)?;
        let mut serialized_blocks = vec![];
        for block in &block_bytes {
            let serialized_block = SerializedBlock::deserialize(block)?;
            serialized_blocks.push(serialized_block);
        }
        Ok(serialized_blocks)
    }
    // TODO: terminar de migrar esta función
    pub fn read_blocks_from_file() -> Result<Vec<SerializedBlock>, NodoBitcoinError> {
        let block_bytes = leer_todos_blocks()?;
        let mut serialized_blocks = vec![];
        for block in &block_bytes {
            let serialized_block = SerializedBlock::deserialize(block)?;
            serialized_blocks.push(serialized_block);
        }
        Ok(serialized_blocks)
    }

    pub fn contains_block(blocks: Vec<SerializedBlock>, block: SerializedBlock) -> bool {
        // verificar si el block se encuentra en blocks
        let mut exist = false;
        for b in blocks {
            if b.header.hash() == block.header.hash() {
                exist = true;
                break;
            }
        }
        exist
    }
}

impl PartialOrd for SerializedBlock {
    fn partial_cmp(&self, other: &SerializedBlock) -> Option<Ordering> {
        let self_timestamp = self.header.time;
        let other_timestamp = other.header.time;
        match self_timestamp > other_timestamp {
            true => return Some(Ordering::Greater),
            false => match self_timestamp < other_timestamp {
                true => return Some(Ordering::Less),
                false => (),
            },
        }
        Some(Ordering::Equal)
    }
}

impl Ord for SerializedBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ordering) => ordering,
            None => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_deserialize() {
        let block_bytes: Vec<u8> = vec![
            // header
            1, 0, 0, 0, //version
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, // previous block
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
            50, //merkle root
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //merkle root
            21, 205, 91, 7, //time /
            21, 205, 91, 7, // n bites
            21, 205, 91, 7, //nonce
            // cantidad transactions
            1, // transaction
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, // trout pk
            172, 0, 0, 0, 0,
        ]; // lock time

        let header = BlockHeader {
            version: 1,
            previous_block_hash: [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            ],
            merkle_root_hash: [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            ],
            time: 123456789,
            n_bits: 123456789,
            nonce: 123456789,
        };

        let version = 1;
        let input = vec![transaction::TxIn {
            previous_output: transaction::Outpoint {
                hash: [0; 32],
                index: 4294967295,
            },
            script_bytes: 31,
            script_bytes_amount: 1,
            signature_script: vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
            ],
            sequence: 4294967295,
        }];
        let output = vec![transaction::TxOut {
            value: 5000000000,
            pk_len: 35,
            pk_len_bytes: 1,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        }];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count: 1,
            tx_out_count: 1,
        };

        let result = SerializedBlock::deserialize(&block_bytes);

        assert!(result.is_ok());

        let serialized_block = result.unwrap();

        assert_eq!(serialized_block.header, header);
        assert_eq!(serialized_block.txns.len(), 1);
        assert_eq!(serialized_block.txns[0], transaction);
    }
    fn get_bloque_bytes() -> Vec<u8> {
        let bytes: Vec<u8> = vec![
            1, 0, 0, 0, 32, 120, 42, 0, 82, 85, 182, 87, 105, 110, 160, 87, 213, 185, 143, 52, 222,
            252, 247, 81, 150, 246, 79, 110, 234, 200, 2, 108, 0, 0, 0, 0, 65, 186, 90, 252, 83,
            42, 174, 3, 21, 27, 138, 168, 123, 101, 225, 89, 79, 151, 80, 74, 118, 142, 1, 12, 152,
            192, 173, 215, 146, 22, 36, 113, 134, 231, 73, 77, 255, 255, 0, 29, 5, 141, 194, 182,
            1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 14, 4, 134, 231, 73, 77, 1, 81, 6, 47,
            80, 50, 83, 72, 47, 255, 255, 255, 255, 1, 0, 242, 5, 42, 1, 0, 0, 0, 35, 33, 3, 246,
            217, 255, 76, 18, 149, 148, 69, 202, 85, 73, 200, 17, 104, 59, 249, 200, 142, 99, 123,
            34, 45, 210, 224, 49, 17, 84, 196, 200, 92, 244, 35, 172, 0, 0, 0, 0,
        ];
        bytes
    }

    fn get_tx_bytes() -> Vec<u8> {
        let bytes: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
        ];
        bytes
    }

    #[test]
    fn test_proof_of_inclusion() {
        let bloque_bytes = get_bloque_bytes();
        let tx_bytes = get_tx_bytes();

        let serialized_block_result = SerializedBlock::deserialize(&bloque_bytes);
        assert!(serialized_block_result.is_ok());

        let serialized_block = serialized_block_result.unwrap();

        let tx = &serialized_block.txns[0];
        let proof_of_inclusion_ok = serialized_block._tx_proof_of_inclusion(tx);
        assert!(proof_of_inclusion_ok);

        let tx_not_included_result = transaction::Transaction::deserialize(&tx_bytes);
        assert!(tx_not_included_result.is_ok());

        let tx_not_included = tx_not_included_result.unwrap();

        let proof_of_inclusion_no_ok = serialized_block._tx_proof_of_inclusion(&tx_not_included);
        assert!(!proof_of_inclusion_no_ok);
    }

    #[test]
    fn test_serialize() {
        let bloque_bytes = get_bloque_bytes();
        let serialized_block_result = SerializedBlock::deserialize(&bloque_bytes);
        assert!(serialized_block_result.is_ok());

        let serialized_block = serialized_block_result.unwrap();

        let serialize_result = serialized_block.serialize();
        assert!(serialize_result.is_ok());

        let serialized = serialize_result.unwrap();

        assert_eq!(serialized, bloque_bytes);
    }

    // #[test]
    // fn test_is_valid_merkle_root() {
    //     let args: Vec<String> = vec![];
    //     let init_result = config::inicializar(args);
    //     assert!(init_result.is_ok());

    //     let blocks = SerializedBlock::read_blocks_from_file();
    //     assert!(blocks.is_ok());
    //     let blocks = blocks.unwrap();

    //     let blocks_reverse = blocks
    //         .iter()
    //         .rev()
    //         .collect::<Vec<&SerializedBlock>>()
    //         .clone();

    //     let mut is_valid_merkle_root = true;
    //     for block in blocks_reverse {
    //         is_valid_merkle_root = block.is_valid_merkle();
    //         if !is_valid_merkle_root {
    //             println!(
    //                 "Block mined as {:?} UNIXTIME, is not valid",
    //                 block.header.time
    //             );
    //             //return;
    //         }
    //     }
    // }
}
