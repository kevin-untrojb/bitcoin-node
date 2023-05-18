use super::{block::SerializedBlock, blockheader::BlockHeader};

#[derive(Clone)]
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

    pub fn add_headers(&mut self, headers: &Vec<BlockHeader>) {
        let _ = &(self.headers).extend_from_slice(headers);
    }

    pub fn add_block(&mut self, block: SerializedBlock) {
        let _ = &(self.blocks).push(block);
    }

    pub fn get_last_header(&self) -> &BlockHeader {
        self.headers.last().unwrap()
    }

    pub fn get_headers(&self) -> Vec<BlockHeader> {
        self.headers.clone()
    }
}
