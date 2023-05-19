use super::{header::BlockHeader, transaction};
use transaction::_Transaction;
use crate::errores::NodoBitcoinError;

/// A struct representing a Bitcoin Serialized Block
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/block_chain.html#serialized-blocks
///
/// # Fields
///
/// * `header` - The header of the block, which contains metadata such as the block's version, hash, and timestamp.
/// * `txns` - The transactions included in the block, represented as a vector of `Transaction` structs.
#[derive(Clone)]
pub struct SerializedBlock {
    pub header: BlockHeader,
    pub txns: Vec<_Transaction>,
}

impl SerializedBlock {
    pub fn deserialize(block_bytes: &[u8]) -> Result<SerializedBlock,NodoBitcoinError> {
        let mut offset = 0;
        let header = BlockHeader::deserialize(&block_bytes[offset..offset + 80])?;
        offset += 80;
        Ok(SerializedBlock{
            header,
            txns: vec!()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let block_bytes: [u8; 116] = [
            /// header
            1, 0, 0, 0,//version
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, // previous block
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,//merkle root
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //merkle root
            21, 205, 91, 7, //time /
            21, 205, 91, 7,// n bites
            21, 205, 91, 7, //nonce
            /// todo modificar los bytes siguientes conforme esté las transacciones
            1,1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,1,
            1,1,1,1,1,1,1,1,1,
        ];
        let header = BlockHeader {
            version: 1,
            previous_block_hash: [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ],
            merkle_root_hash: [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ],
            time: 123456789,
            n_bits: 123456789,
            nonce: 123456789,
        };

        let result = SerializedBlock::deserialize(&block_bytes);

        assert!(result.is_ok());

        let serialized_block = result.unwrap();

        assert_eq!(serialized_block.header,header )
    }
}