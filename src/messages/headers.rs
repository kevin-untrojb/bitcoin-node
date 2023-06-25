use crate::{
    blockchain::blockheader::BlockHeader, common::utils_bytes::parse_varint,
    errores::NodoBitcoinError,
};

/// Deserealiza el vector de bytes de headers recibidos
/// Devuelve un vector de BlockHeaders deserealizados
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
        block_headers.push(block_header);
    }

    Ok(block_headers)
}
