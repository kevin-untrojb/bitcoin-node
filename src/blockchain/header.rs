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
struct BlockHeader {
    id:usize,
    version: i32,
    previous_block_hash: String,
    merkle_root_hash: String,
    time: u32,
    n_bits: u32,
    nonce: u32,
}