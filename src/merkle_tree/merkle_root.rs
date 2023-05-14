use crate::errores::NodoBitcoinError;

use super::merkle_node::MerkleNode;

pub struct MerkleRoot {
    pub root: Option<Box<MerkleNode>>,
}

impl MerkleRoot {
    fn _new(transactions_ids: &Vec<usize>) -> Result<MerkleRoot, NodoBitcoinError> {
        let mut root = None;
        if !transactions_ids.is_empty() {
            let mut ids = transactions_ids.clone();
            ids.sort();
            let node = _build_merkle_tree(&ids)?;
            root = Some(Box::new(node));
        }
        Ok(MerkleRoot { root })
    }
}

// https://developer.bitcoin.org/reference/block_chain.html#merkle-trees
fn _build_merkle_tree(ordered_txids: &[usize]) -> Result<MerkleNode, NodoBitcoinError> {
    let mut nodes: Vec<MerkleNode> = ordered_txids
        .iter()
        .map(|id| MerkleNode {
            left: None,
            right: None,
            hash: id.to_ne_bytes().to_vec(),
        })
        .collect();

    if nodes.len() == 1 {
        return nodes.first().cloned().ok_or(NodoBitcoinError::NoChildren);
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
    nodes.first().cloned().ok_or(NodoBitcoinError::NoChildren)
}
