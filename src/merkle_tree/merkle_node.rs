use crate::errores::NodoBitcoinError;
use bitcoin_hashes::{sha256d, Hash};

#[derive(Clone, Debug)]
pub struct MerkleNode {
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub hash: Vec<u8>,
}

impl MerkleNode {
    pub fn from_nodes(
        left_node: Option<MerkleNode>,
        right_node: Option<MerkleNode>,
    ) -> Result<MerkleNode, NodoBitcoinError> {
        if left_node.is_none() && right_node.is_none() {
            return Err(NodoBitcoinError::NoChildren);
        }
        let hash = Self::hash(left_node.clone(), right_node.clone());
        let left = left_node.map(Box::new);
        let right = right_node.map(Box::new);
        let node = MerkleNode { left, right, hash };
        Ok(node)
    }

    pub fn hash_bytes(&self) -> [u8; 32] {
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&self.hash);
        hash_bytes
    }

    pub fn has_children(&self) -> bool {
        self.left.is_some() || self.right.is_some()
    }

    fn hash(left: Option<MerkleNode>, right: Option<MerkleNode>) -> Vec<u8> {
        let left_hash = match left {
            Some(left) => left.hash,
            None => vec![],
        };
        let right_hash = match right {
            Some(right) => right.hash,
            None => vec![],
        };
        Self::calculate_hash(left_hash, right_hash)
    }

    pub fn calculate_hash(left: Vec<u8>, right: Vec<u8>) -> Vec<u8> {
        let leaps_hash = [left.clone(), right.clone()].concat();
        if left.is_empty() || right.is_empty() {
            return leaps_hash;
        }
        let node_hash = sha256d::Hash::hash(&leaps_hash);
        node_hash.as_byte_array().clone().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use bitcoin_hashes::{sha256d, Hash};

    use crate::{errores::NodoBitcoinError, merkle_tree::merkle_node::MerkleNode};

    #[test]
    fn test_error() {
        let left_node: Option<MerkleNode> = None;
        let right_node: Option<MerkleNode> = None;

        let result_error = MerkleNode::from_nodes(left_node, right_node);
        assert!(result_error.is_err());
        assert!(matches!(result_error, Err(NodoBitcoinError::NoChildren)));
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

        let result_one_node = MerkleNode::from_nodes(left_node, right_node);
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

        let result_one_node = MerkleNode::from_nodes(left_node, right_node);
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

        let hash = MerkleNode::hash(left_node, right_node);

        let calculation_hash = sha256d::Hash::hash(&[hash_left, hash_right].concat());
        assert_eq!(hash, calculation_hash.as_byte_array().clone().to_vec());
    }
}
