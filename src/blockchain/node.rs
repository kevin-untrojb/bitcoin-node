use super::{block::SerializedBlock, blockheader::BlockHeader};

#[derive(Clone)]
pub struct Node {
    _headers: Vec<BlockHeader>,
    _blocks: Vec<SerializedBlock>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            _headers: Vec::new(),
            _blocks: Vec::new(),
        }
    }

    pub fn _add_headers(&mut self, headers: &Vec<BlockHeader>) {
        let _ = &(self._headers).extend_from_slice(headers);
    }

    pub fn _add_block(&mut self, block: SerializedBlock) {
        let _ = &(self._blocks).push(block);
    }

    pub fn _get_last_header(&self) -> &BlockHeader {
        self._headers.last().unwrap()
    }

    pub fn _get_headers(&self) -> Vec<BlockHeader> {
        self._headers.clone()
    }
}
