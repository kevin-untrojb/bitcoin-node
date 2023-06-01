use crate::{
    blockchain::{blockheader::BlockHeader, file::escribir_archivo},
    common::utils_bytes::parse_varint,
    errores::NodoBitcoinError,
};

pub fn _deserealize(mut headers: Vec<u8>) -> Result<Vec<BlockHeader>, NodoBitcoinError> {
    let (size_bytes, num_headers) = parse_varint(&headers);
    headers = headers[size_bytes..].to_vec();
    let mut block_headers = Vec::new();
    for i in 0..num_headers {
        let mut start: usize = i * 80;
        let mut end: usize = start + 80;
        if i != 0 {
            start += i;
            end += i;
        }

        let block_header = BlockHeader::deserialize(&headers[start..end])?;
        escribir_archivo(&headers[start..end])?;
        block_headers.push(block_header);
    }

    Ok(block_headers)
}

pub fn deserealize_sin_guardar(mut headers: Vec<u8>) -> Result<Vec<BlockHeader>, NodoBitcoinError> {
    let (size_bytes, num_headers) = parse_varint(&headers);
    headers = headers[size_bytes..].to_vec();
    let mut block_headers = Vec::new();
    for i in 0..num_headers {
        let mut start: usize = i * 80;
        let mut end: usize = start + 80;
        if i != 0 {
            start += i;
            end += i;
        }

        let block_header = BlockHeader::deserialize(&headers[start..end])?;
        //escribir_archivo(&headers[start..end])?;
        block_headers.push(block_header);
    }

    Ok(block_headers)
}

pub fn _deserealize_desde_archivo(headers: Vec<u8>) -> Result<Vec<BlockHeader>, NodoBitcoinError> {
    let mut block_headers = Vec::new();
    let num_headers: usize = headers.len() / 80;
    for i in 0..num_headers {
        let mut start = i * 80;
        let mut end = start + 80;
        if i != 0 {
            start += i;
            end += i;
        }

        //println!("start: {}, end: {}", start, end);
        println!("actual: {}, total: {}", i, num_headers);

        let block_header = BlockHeader::deserialize(&headers[start..end])?;
        block_headers.push(block_header);
    }

    Ok(block_headers)
}
