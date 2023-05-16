use super::{block::SerializedBlock, blockheader::BlockHeader};

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

    pub fn get_last_header(&self) -> &BlockHeader{
        self.headers.last().unwrap()
    }
}
