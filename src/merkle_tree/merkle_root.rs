use crate::{common::uint256::Uint256, errores::NodoBitcoinError};

use super::merkle_node::MerkleNode;

pub struct MerkleRoot {
    pub root: Option<Box<MerkleNode>>,
}

impl MerkleRoot {
    fn _new(transactions_ids: &Vec<Uint256>) -> Result<MerkleRoot, NodoBitcoinError> {
        let mut root = None;
        if !transactions_ids.is_empty() {
            let mut ids = transactions_ids.clone();
            ids.sort();
            let node = Self::_build_merkle_tree(&ids)?;
            root = Some(Box::new(node));
        }
        Ok(MerkleRoot { root })
    }

    // https://developer.bitcoin.org/reference/block_chain.html#merkle-trees
    fn _build_merkle_tree(ordered_txids: &[Uint256]) -> Result<MerkleNode, NodoBitcoinError> {
        let mut nodes: Vec<MerkleNode> = ordered_txids
            .iter()
            .map(|id| MerkleNode {
                left: None,
                right: None,
                hash: id._to_bytes().to_vec(),
            })
            .collect();

        if nodes.len() == 1 {
            return nodes.first().cloned().ok_or(NodoBitcoinError::_NoChildren);
        }

        while nodes.len() > 1 {
            let mut new_level = Vec::new();
            for i in (0..nodes.len()).step_by(2) {
                let left_node = nodes[i].clone();
                let right_node = if i + 1 < nodes.len() {
                    nodes[i + 1].clone()
                } else {
                    nodes[i].clone()
                };
                let new_node = MerkleNode::from_nodes(Some(left_node), Some(right_node))?;
                new_level.push(new_node);
            }
            nodes = new_level;
        }
        nodes.first().cloned().ok_or(NodoBitcoinError::_NoChildren)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::uint256::Uint256, errores::NodoBitcoinError, merkle_tree::merkle_root::MerkleRoot,
    };
    use bitcoin_hashes::{sha256d, Hash};

    // Create test cases for the MerkleRoot struct
    #[test]
    fn test_merkle_root() {
        let txids = vec![
            Uint256::_from_u32(1),
            Uint256::_from_u32(2),
            Uint256::_from_u32(3),
            Uint256::_from_u32(4),
            Uint256::_from_u32(5),
            Uint256::_from_u32(6),
            Uint256::_from_u32(7),
            Uint256::_from_u32(8),
        ];
        let merkle_root_result = MerkleRoot::_new(&txids);
        assert!(merkle_root_result.is_ok());

        let merkle_root = merkle_root_result.unwrap();
        assert!(merkle_root.root.is_some());
    }

    #[test]
    fn test_merkle_root_one() {
        let txids = vec![Uint256::_from_u32(1)];
        let merkle_root_result = MerkleRoot::_new(&txids);
        assert!(merkle_root_result.is_ok());

        let merkle_root = merkle_root_result.unwrap();
        assert!(merkle_root.root.is_some());

        let root = merkle_root.root.unwrap();

        let calculate_hash = txids[0]._to_bytes().to_vec();
        assert!(root.hash == calculate_hash);
    }

    #[test]
    fn test_merkle_root_empty() {
        let txids = vec![];
        let merkle_root_result = MerkleRoot::_new(&txids);
        assert!(merkle_root_result.is_ok());

        let merkle_root = merkle_root_result.unwrap();
        assert!(merkle_root.root.is_none());
    }

    #[test]
    fn test_build_merkle_root_no_children() {
        let txids = vec![];
        let merkle_root_result = MerkleRoot::_build_merkle_tree(&txids);
        assert!(merkle_root_result.is_err());

        let merkle_root_error = merkle_root_result.unwrap_err();
        assert_eq!(merkle_root_error, NodoBitcoinError::_NoChildren);
    }

    #[test]
    fn test_build_merkle_root_one_child() {
        let txids = vec![Uint256::_from_u32(1)];
        let result_merkle_root = MerkleRoot::_build_merkle_tree(&txids);
        assert!(result_merkle_root.is_ok());

        let merkle_root = result_merkle_root.unwrap();
        assert!(merkle_root.left.is_none());
        assert!(merkle_root.right.is_none());
    }

    #[test]
    fn test_build_merkle_root_two_children() {
        let txids = vec![Uint256::_from_u32(1), Uint256::_from_u32(2)];
        let result_merkle_root = MerkleRoot::_build_merkle_tree(&txids);
        assert!(result_merkle_root.is_ok());

        let merkle_root = result_merkle_root.unwrap();
        assert!(merkle_root.left.is_some());
        assert!(merkle_root.right.is_some());

        let hash = merkle_root.hash;

        let left = txids[0]._to_bytes().to_vec();
        let right = txids[1]._to_bytes().to_vec();
        let concat_hashes = [left, right].concat();
        let calculate_hash = sha256d::Hash::hash(&concat_hashes);
        let calculate_hash_vector = calculate_hash.as_byte_array().clone().to_vec();

        assert_eq!(hash, calculate_hash_vector);
    }

    #[test]
    fn test_build_merkle_root_three_children() {
        let txids = vec![
            Uint256::_from_u32(1),
            Uint256::_from_u32(2),
            Uint256::_from_u32(3),
        ];
        let result_merkle_root = MerkleRoot::_build_merkle_tree(&txids);
        assert!(result_merkle_root.is_ok());

        let merkle_root = result_merkle_root.unwrap();
        assert!(merkle_root.left.is_some());
        assert!(merkle_root.right.is_some());

        let hash = merkle_root.hash;

        let left_one = txids[0]._to_bytes().to_vec();
        let right_one = txids[1]._to_bytes().to_vec();
        let concat_hashes_one = [left_one, right_one].concat();
        let calculate_hash_one = sha256d::Hash::hash(&concat_hashes_one);
        let calculate_hash_vector_one = calculate_hash_one.as_byte_array().clone().to_vec();

        let left_two = txids[2]._to_bytes().to_vec();
        let right_two = txids[2]._to_bytes().to_vec();
        let concat_hashes_two = [left_two, right_two].concat();
        let calculate_hash_two = sha256d::Hash::hash(&concat_hashes_two);
        let calculate_hash_vector_two = calculate_hash_two.as_byte_array().clone().to_vec();

        let concat_hashes_both = [calculate_hash_vector_one, calculate_hash_vector_two].concat();
        let calculate_hash_both = sha256d::Hash::hash(&concat_hashes_both);
        let calculate_hash_vector = calculate_hash_both.as_byte_array().clone().to_vec();

        assert_eq!(hash, calculate_hash_vector);
    }

    #[test]
    fn test_build_merkle_root_four_children() {
        let txids = vec![
            Uint256::_from_u32(1),
            Uint256::_from_u32(2),
            Uint256::_from_u32(3),
            Uint256::_from_u32(4),
        ];
        let result_merkle_root = MerkleRoot::_build_merkle_tree(&txids);
        assert!(result_merkle_root.is_ok());

        let merkle_root = result_merkle_root.unwrap();
        assert!(merkle_root.left.is_some());
        assert!(merkle_root.right.is_some());

        let hash = merkle_root.hash;

        let left_one = txids[0]._to_bytes().to_vec();
        let right_one = txids[1]._to_bytes().to_vec();
        let concat_hashes_one = [left_one, right_one].concat();
        let calculate_hash_one = sha256d::Hash::hash(&concat_hashes_one);
        let calculate_hash_vector_one = calculate_hash_one.as_byte_array().clone().to_vec();

        let left_two = txids[2]._to_bytes().to_vec();
        let right_two = txids[3]._to_bytes().to_vec();
        let concat_hashes_two = [left_two, right_two].concat();
        let calculate_hash_two = sha256d::Hash::hash(&concat_hashes_two);
        let calculate_hash_vector_two = calculate_hash_two.as_byte_array().clone().to_vec();

        let concat_hashes_both = [calculate_hash_vector_one, calculate_hash_vector_two].concat();
        let calculate_hash_both = sha256d::Hash::hash(&concat_hashes_both);
        let calculate_hash_vector = calculate_hash_both.as_byte_array().clone().to_vec();

        assert_eq!(hash, calculate_hash_vector);
    }

    #[test]
    fn test_build_merkle_root_four_children_disordered() {
        let txids = vec![
            Uint256::_from_u32(1),
            Uint256::_from_u32(3),
            Uint256::_from_u32(4),
            Uint256::_from_u32(2),
        ];
        let result_merkle_tree = MerkleRoot::_new(&txids);
        assert!(result_merkle_tree.is_ok());

        let merkle_tree = result_merkle_tree.unwrap();
        assert!(merkle_tree.root.is_some());

        let merkle_node_root = merkle_tree.root.unwrap();
        assert!(merkle_node_root.left.is_some());
        assert!(merkle_node_root.right.is_some());

        let hash = merkle_node_root.hash;

        let left_one = txids[0]._to_bytes().to_vec();
        let right_one = txids[3]._to_bytes().to_vec();
        let concat_hashes_one = [left_one, right_one].concat();
        let calculate_hash_one = sha256d::Hash::hash(&concat_hashes_one);
        let calculate_hash_vector_one = calculate_hash_one.as_byte_array().clone().to_vec();

        let left_two = txids[1]._to_bytes().to_vec();
        let right_two = txids[2]._to_bytes().to_vec();
        let concat_hashes_two = [left_two, right_two].concat();
        let calculate_hash_two = sha256d::Hash::hash(&concat_hashes_two);
        let calculate_hash_vector_two = calculate_hash_two.as_byte_array().clone().to_vec();

        let concat_hashes_both = [calculate_hash_vector_one, calculate_hash_vector_two].concat();
        let calculate_hash_both = sha256d::Hash::hash(&concat_hashes_both);
        let calculate_hash_vector = calculate_hash_both.as_byte_array().clone().to_vec();

        assert_eq!(hash, calculate_hash_vector);
    }
}
