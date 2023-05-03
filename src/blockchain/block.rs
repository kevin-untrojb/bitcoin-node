use super::header;
use super::transaction;
use super::block;
use transaction::Transaction;
use crate::BlockHeader;
/// A struct representing a Bitcoin Serialized Block
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/block_chain.html#serialized-blocks
///
/// # Fields
///
/// * `id` - The unique identifier of the transaction.
/// * `header` - The header of the block, which contains metadata such as the block's version, hash, and timestamp.
/// * `txns` - The transactions included in the block, represented as a vector of `Transaction` structs.
pub struct SerializedBlock {
    id:usize,
    header: BlockHeader,
    txns: Vec<Transaction>,
}
