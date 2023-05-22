use super::{blockheader::BlockHeader, transaction};
use crate::common::utils_bytes;
use crate::errores::NodoBitcoinError;
use transaction::Transaction;

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
    pub txns: Vec<Transaction>,
}

impl SerializedBlock {
    pub fn deserialize(block_bytes: &[u8]) -> Result<SerializedBlock, NodoBitcoinError> {
        let mut offset = 0;
        let header = BlockHeader::deserialize(&block_bytes[offset..offset + 80])?;
        offset += 80;
        let (size_bytes, txn_count) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += size_bytes;

        let mut txns = Vec::new();
        for _num in 0..txn_count {
            let trn = Transaction::deserialize(&block_bytes[offset..])?;
            offset += trn.size();
            txns.push(trn);
        }
        Ok(SerializedBlock { header, txns })
    }
}
