use crate::errores::NodoBitcoinError;
use bitcoin_hashes::{sha256d, Hash};

#[derive(Clone, Debug)]
pub struct MerkleNode {
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub hash: Vec<u8>,
}

impl MerkleNode {
    pub fn _from_nodes(
        left_node: Option<MerkleNode>,
        right_node: Option<MerkleNode>,
    ) -> Result<MerkleNode, NodoBitcoinError> {
        if left_node.is_none() && right_node.is_none() {
            return Err(NodoBitcoinError::_NoChildren);
        }
        let hash = Self::_hash(left_node.clone(), right_node.clone());
        let left = left_node.map(Box::new);
        let right = right_node.map(Box::new);
        let node = MerkleNode { left, right, hash };
        Ok(node)
    }

    fn _hash(left: Option<MerkleNode>, right: Option<MerkleNode>) -> Vec<u8> {
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

#[test]
fn test_error() {
    let left_node: Option<MerkleNode> = None;
    let right_node: Option<MerkleNode> = None;

    let result_error = MerkleNode::_from_nodes(left_node, right_node);
    assert!(result_error.is_err());
    assert!(matches!(result_error, Err(NodoBitcoinError::_NoChildren)));
}

#[test]
fn test_one_node() {
    let hash_left: Vec<u8> = vec![1, 2];
    let left_node = Some(MerkleNode {
        left: None,
        right: None,
        hash: hash_left.clone(),
    });

    let right_node: Option<MerkleNode> = None;

    let result_one_node = MerkleNode::_from_nodes(left_node, right_node);
    assert!(result_one_node.is_ok());

    let node = result_one_node.unwrap();
    assert_eq!(node.hash, hash_left);
}

#[test]
fn test_two_nodes() {
    let hash_left: Vec<u8> = vec![1, 2];
    let left_node = Some(MerkleNode {
        left: None,
        right: None,
        hash: hash_left.clone(),
    });

    let hash_right: Vec<u8> = vec![1, 2];
    let right_node = Some(MerkleNode {
        left: None,
        right: None,
        hash: hash_right.clone(),
    });

    let result_one_node = MerkleNode::_from_nodes(left_node, right_node);
    assert!(result_one_node.is_ok());

    let node = result_one_node.unwrap();
    let calculation_hash = sha256d::Hash::hash(&[hash_left, hash_right].concat());
    assert_eq!(node.hash, calculation_hash.as_byte_array().clone().to_vec());
}

#[test]
fn test_hash() {
    let hash_left: Vec<u8> = vec![1, 2];
    let left_node = Some(MerkleNode {
        left: None,
        right: None,
        hash: hash_left.clone(),
    });

    let hash_right: Vec<u8> = vec![1, 2];
    let right_node = Some(MerkleNode {
        left: None,
        right: None,
        hash: hash_right.clone(),
    });

    let hash = MerkleNode::_hash(left_node, right_node);

    let calculation_hash = sha256d::Hash::hash(&[hash_left, hash_right].concat());
    assert_eq!(hash, calculation_hash.as_byte_array().clone().to_vec());
}
