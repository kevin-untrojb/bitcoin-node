use crate::{blockchain::{node::Node, blockheader::BlockHeader}, errores::NodoBitcoinError};

pub fn deserealize(node: &mut Node, mut headers: Vec<u8> ) -> Result<(), NodoBitcoinError>{
    let (size_bytes, num_headers) = parse_varint(&headers);
    headers = headers[size_bytes..].to_vec();

    for i in 0..num_headers {
        let mut start: usize = i * 80; 
        let mut end: usize = start + 80;
        if i != 0 {
            start += 1*i;
            end += 1*i;
        }

        let block_header = BlockHeader::deserialize(&headers[start..end])?;
        let _ = node.add_header(block_header);
    }

    Ok(())
}

fn parse_varint(bytes: &[u8]) -> (usize, usize) {
    let prefix = bytes[0];
    match prefix {
        0xfd => (3, u16::from_le_bytes([bytes[1], bytes[2]]) as usize),
        0xfe => (
            5,
            u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize,
        ),
        0xff => (
            9,
            u64::from_le_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            ]) as usize,
        ),
        _ => (1, u64::from(prefix) as usize),
    }
}
