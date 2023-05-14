use super::{block::SerializedBlock, header::BlockHeader};

pub struct Node {
    headers: Vec<BlockHeader>,
    blocks: Vec<SerializedBlock>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            headers: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn add_header(&mut self, header: BlockHeader) {
        let _ = &(self.headers).push(header);
    }
}
