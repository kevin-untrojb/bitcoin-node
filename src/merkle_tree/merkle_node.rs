use crate::errores::NodoBitcoinError;
use bitcoin_hashes::{sha256d, Hash};

#[derive(Clone)]
pub struct MerkleNode {
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub hash: Vec<u8>,
}

impl MerkleNode {
    pub fn to_hash(
        left_node: Option<MerkleNode>,
        right_node: Option<MerkleNode>,
    ) -> Result<MerkleNode, NodoBitcoinError> {
        if left_node.is_none() && right_node.is_none() {
            return Err(NodoBitcoinError::NoChildren);
        }
        let hash = MerkleNode::hash_bitcoin(left_node.clone(), right_node.clone());
        let left = left_node.map(Box::new);
        let right = right_node.map(Box::new);
        let node = MerkleNode { left, right, hash };
        Ok(node)
    }

    fn hash_bitcoin(left: Option<MerkleNode>, right: Option<MerkleNode>) -> Vec<u8> {
        let one_none = left.is_none() || right.is_none();
        let left_hash = match left {
            Some(left) => left.hash,
            None => vec![],
        };
        let right_hash = match right {
            Some(right) => right.hash,
            None => vec![],
        };
        let leaps_hash = [left_hash, right_hash].concat();
        if one_none {
            return leaps_hash;
        }
        let node_hash = sha256d::Hash::hash(&leaps_hash);
        node_hash.as_byte_array().clone().to_vec()
    }
}
