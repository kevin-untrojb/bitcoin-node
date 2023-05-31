use std::cmp::Ordering;

use super::{blockheader::BlockHeader, transaction};
use crate::common::utils_bytes;
use crate::errores::NodoBitcoinError;
use crate::merkle_tree::merkle_root::_MerkleRoot;
use transaction::Transaction;

/// A struct representing a Bitcoin Serialized Block
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/block_chain.html#serialized-blocks
///
/// # Fields
///
/// * `header` - The header of the block, which contains metadata such as the block's version, hash, and timestamp.
/// * `txns` - The transactions included in the block, represented as a vector of `Transaction` structs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SerializedBlock {
    pub header: BlockHeader,
    pub txns: Vec<Transaction>,
    pub block_bytes: Vec<u8>,
}

impl SerializedBlock {
    pub fn deserialize(block_bytes: &[u8]) -> Result<SerializedBlock, NodoBitcoinError> {
        let mut offset = 0;
        if block_bytes.len() < 80 {
            return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
        }
        let header = BlockHeader::deserialize(&block_bytes[offset..offset + 80])?;
        offset += 80;
        let (size_bytes, txn_count) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += size_bytes;

        let mut txns = Vec::new();
        for _ in 0..txn_count {
            let trn = Transaction::deserialize(&block_bytes[offset..])?;
            offset += trn.size();
            txns.push(trn);
        }
        Ok(SerializedBlock {
            header,
            txns,
            block_bytes: block_bytes.to_vec(),
        })
    }

    pub fn _is_valid_merkle(&self) -> bool {
        let current_merkle = self.header.merkle_root_hash;
        let local_merkle = match _MerkleRoot::_from_block(self) {
            Ok(calculated_merkle) => calculated_merkle,
            Err(_) => return false,
        };
        let binding = local_merkle._root_hash();
        let local_merkle_hash = binding.as_slice();
        current_merkle == local_merkle_hash
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
}
