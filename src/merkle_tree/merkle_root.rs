use std::{collections::HashMap, vec};

use crate::{
    blockchain::{block::SerializedBlock, transaction::Transaction},
    common::uint256::Uint256,
    errores::NodoBitcoinError,
};

use super::merkle_node::MerkleNode;

pub struct _MerkleRoot {
    pub root: Option<Box<MerkleNode>>,
    _hashmap: HashMap<[u8; 32], bool>,
}

impl _MerkleRoot {
    pub fn _from_block(block: &SerializedBlock) -> Result<_MerkleRoot, NodoBitcoinError> {
        let txs = &block.txns;
        Self::_from_txs(txs)
    }

    pub fn _from_txs(transactions: &[Transaction]) -> Result<_MerkleRoot, NodoBitcoinError> {
        let transactions_ids = transactions
            .iter()
            .map(|tx| tx._txid())
            .collect::<Result<Vec<Uint256>, NodoBitcoinError>>()?;
        Self::_from_ids(&transactions_ids)
    }

    pub fn _from_ids(transactions_ids: &Vec<Uint256>) -> Result<_MerkleRoot, NodoBitcoinError> {
        let mut root = None;
        let mut hashmap = HashMap::new();
        if !transactions_ids.is_empty() {
            let ids = transactions_ids.clone();

            hashmap = ids
                .iter()
                .map(|id| (id.clone()._to_bytes(), true))
                .collect::<HashMap<[u8; 32], bool>>();

            let node = Self::_build_merkle_tree(&ids)?;
            root = Some(Box::new(node));
        }
        Ok(_MerkleRoot {
            root,
            _hashmap: hashmap,
        })
    }

    pub fn _root_hash(&self) -> Vec<u8> {
        match &self.root {
            Some(node) => node.hash.clone(),
            None => vec![0; 32],
        }
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
                let new_node = MerkleNode::_from_nodes(Some(left_node), Some(right_node))?;
                new_level.push(new_node);
            }
            nodes = new_level;
        }
        nodes.first().cloned().ok_or(NodoBitcoinError::_NoChildren)
    }

    pub fn _proof_of_inclusion(&self, tx: &Transaction) -> bool {
        // verificar que el txid de la transaccion este en el arbol
        let txid_result = tx._txid();
        let txid = match txid_result {
            Ok(txid) => txid,
            Err(_) => return false,
        };
        let txid_bytes = txid._to_bytes();
        self._hashmap.contains_key(&txid_bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::uint256::Uint256, errores::NodoBitcoinError, merkle_tree::merkle_root::_MerkleRoot,
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
        let merkle_root_result = _MerkleRoot::_from_ids(&txids);
        assert!(merkle_root_result.is_ok());

        let merkle_root = merkle_root_result.unwrap();
        assert!(merkle_root.root.is_some());
    }

    #[test]
    fn test_merkle_root_one() {
        let txids = vec![Uint256::_from_u32(1)];
        let merkle_root_result = _MerkleRoot::_from_ids(&txids);
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
        let merkle_root_result = _MerkleRoot::_from_ids(&txids);
        assert!(merkle_root_result.is_ok());

        let merkle_root = merkle_root_result.unwrap();
        assert!(merkle_root.root.is_none());
    }

    #[test]
    fn test_build_merkle_root_no_children() {
        let txids = vec![];
        let merkle_root_result = _MerkleRoot::_build_merkle_tree(&txids);
        assert!(merkle_root_result.is_err());

        let merkle_root_error = merkle_root_result.unwrap_err();
        assert_eq!(merkle_root_error, NodoBitcoinError::_NoChildren);
    }

    #[test]
    fn test_build_merkle_root_one_child() {
        let txids = vec![Uint256::_from_u32(1)];
        let result_merkle_root = _MerkleRoot::_build_merkle_tree(&txids);
        assert!(result_merkle_root.is_ok());

        let merkle_root = result_merkle_root.unwrap();
        assert!(merkle_root.left.is_none());
        assert!(merkle_root.right.is_none());
    }

    #[test]
    fn test_build_merkle_root_two_children() {
        let bytes1: [u8; 32] = [
            0xc1, 0x17, 0xea, 0x8e, 0xc8, 0x28, 0x34, 0x2f, 0x4d, 0xfb, 0x0a, 0xd6, 0xbd, 0x14,
            0x0e, 0x03, 0xa5, 0x07, 0x20, 0xec, 0xe4, 0x01, 0x69, 0xee, 0x38, 0xbd, 0xc1, 0x5d,
            0x9e, 0xb6, 0x4c, 0xf5,
        ];
        let bytes2: [u8; 32] = [
            0xc1, 0x31, 0x47, 0x41, 0x64, 0xb4, 0x12, 0xe3, 0x40, 0x66, 0x96, 0xda, 0x1e, 0xe2,
            0x0a, 0xb0, 0xfc, 0x9b, 0xf4, 0x1c, 0x8f, 0x05, 0xfa, 0x8c, 0xee, 0xa7, 0xa0, 0x8d,
            0x67, 0x2d, 0x7c, 0xc5,
        ];

        let merkle_root_bytes: [u8; 32] = [
            0x8b, 0x30, 0xc5, 0xba, 0x10, 0x0f, 0x6f, 0x2e, 0x5a, 0xd1, 0xe2, 0xa7, 0x42, 0xe5,
            0x02, 0x04, 0x91, 0x24, 0x0f, 0x8e, 0xb5, 0x14, 0xfe, 0x97, 0xc7, 0x13, 0xc3, 0x17,
            0x18, 0xad, 0x7e, 0xcd,
        ];

        let txids = vec![Uint256::_from_bytes(bytes1), Uint256::_from_bytes(bytes2)];
        let result_merkle_root = _MerkleRoot::_build_merkle_tree(&txids);
        assert!(result_merkle_root.is_ok());

        let merkle_root = result_merkle_root.unwrap();
        assert!(merkle_root.left.is_some());
        assert!(merkle_root.right.is_some());

        let hash = merkle_root.hash;

        //let left = txids[0]._to_bytes().to_vec();
        //let right = txids[1]._to_bytes().to_vec();
        //let concat_hashes = [left, right].concat();
        //let calculate_hash = sha256d::Hash::hash(&concat_hashes);
        //let calculate_hash_vector = calculate_hash.as_byte_array().clone().to_vec();

        assert_eq!(hash, merkle_root_bytes);
    }

    #[test]
    fn test_build_merkle_root_three_children() {
        let txids = vec![
            Uint256::_from_u32(1),
            Uint256::_from_u32(2),
            Uint256::_from_u32(3),
        ];
        let result_merkle_root = _MerkleRoot::_build_merkle_tree(&txids);
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
        let result_merkle_root = _MerkleRoot::_build_merkle_tree(&txids);
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
    fn test_build_merkle_root_five_children() {
        /*
            Datos del libro de O'Ŕeilly
            hex_hashes = [
                'c117ea8ec828342f4dfb0ad6bd140e03a50720ece40169ee38bdc15d9eb64cf5',
                'c131474164b412e3406696da1ee20ab0fc9bf41c8f05fa8ceea7a08d672d7cc5',
                'f391da6ecfeed1814efae39e7fcb3838ae0b02c02ae7d0a5848a66947c0727b0',
                '3d238a92a94532b946c90e19c49351c763696cff3db400485b813aecb8a13181',
                '10092f2633be5f3ce349bf9ddbde36caa3dd10dfa0ec8106bce23acbff637dae',
            ]
            root = [28cd414af2e36dc7eeea84e189bf09372afbd98bf968543965bc2eb3de9b841c]
        */
        let bytes1: [u8; 32] = [
            0xc1, 0x17, 0xea, 0x8e, 0xc8, 0x28, 0x34, 0x2f, 0x4d, 0xfb, 0x0a, 0xd6, 0xbd, 0x14,
            0x0e, 0x03, 0xa5, 0x07, 0x20, 0xec, 0xe4, 0x01, 0x69, 0xee, 0x38, 0xbd, 0xc1, 0x5d,
            0x9e, 0xb6, 0x4c, 0xf5,
        ];
        let bytes2: [u8; 32] = [
            0xc1, 0x31, 0x47, 0x41, 0x64, 0xb4, 0x12, 0xe3, 0x40, 0x66, 0x96, 0xda, 0x1e, 0xe2,
            0x0a, 0xb0, 0xfc, 0x9b, 0xf4, 0x1c, 0x8f, 0x05, 0xfa, 0x8c, 0xee, 0xa7, 0xa0, 0x8d,
            0x67, 0x2d, 0x7c, 0xc5,
        ];
        let bytes3: [u8; 32] = [
            0xf3, 0x91, 0xda, 0x6e, 0xcf, 0xee, 0xd1, 0x81, 0x4e, 0xfa, 0xe3, 0x9e, 0x7f, 0xcb,
            0x38, 0x38, 0xae, 0x0b, 0x02, 0xc0, 0x2a, 0xe7, 0xd0, 0xa5, 0x84, 0x8a, 0x66, 0x94,
            0x7c, 0x07, 0x27, 0xb0,
        ];

        let bytes4: [u8; 32] = [
            0x3d, 0x23, 0x8a, 0x92, 0xa9, 0x45, 0x32, 0xb9, 0x46, 0xc9, 0x0e, 0x19, 0xc4, 0x93,
            0x51, 0xc7, 0x63, 0x69, 0x6c, 0xff, 0x3d, 0xb4, 0x00, 0x48, 0x5b, 0x81, 0x3a, 0xec,
            0xb8, 0xa1, 0x31, 0x81,
        ];
        let bytes5: [u8; 32] = [
            0x10, 0x09, 0x2f, 0x26, 0x33, 0xbe, 0x5f, 0x3c, 0xe3, 0x49, 0xbf, 0x9d, 0xdb, 0xde,
            0x36, 0xca, 0xa3, 0xdd, 0x10, 0xdf, 0xa0, 0xec, 0x81, 0x06, 0xbc, 0xe2, 0x3a, 0xcb,
            0xff, 0x63, 0x7d, 0xae,
        ];

        let merkle_root_bytes: [u8; 32] = [
            0x28, 0xcd, 0x41, 0x4a, 0xf2, 0xe3, 0x6d, 0xc7, 0xee, 0xea, 0x84, 0xe1, 0x89, 0xbf,
            0x09, 0x37, 0x2a, 0xfb, 0xd9, 0x8b, 0xf9, 0x68, 0x54, 0x39, 0x65, 0xbc, 0x2e, 0xb3,
            0xde, 0x9b, 0x84, 0x1c,
        ];

        let txids = vec![
            Uint256::_from_bytes(bytes1),
            Uint256::_from_bytes(bytes2),
            Uint256::_from_bytes(bytes3),
            Uint256::_from_bytes(bytes4),
            Uint256::_from_bytes(bytes5),
        ];
        let result_merkle_tree = _MerkleRoot::_from_ids(&txids);
        assert!(result_merkle_tree.is_ok());

        let merkle_tree = result_merkle_tree.unwrap();
        assert!(merkle_tree.root.is_some());

        let merkle_node_root = merkle_tree.root.unwrap();
        assert!(merkle_node_root.left.is_some());
        assert!(merkle_node_root.right.is_some());

        let hash = merkle_node_root.hash;

        assert_eq!(hash, merkle_root_bytes);
    }
}
