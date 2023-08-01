use crate::{
    blockchain::blockheader::BlockHeader, common::utils_bytes::parse_varint,
    errores::NodoBitcoinError,
};

use super::{getheaders::GetHeadersMessage, messages_header::make_header};

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

pub fn make_headers_msg(get_headers: GetHeadersMessage) -> Result<Vec<u8>, NodoBitcoinError> {
    // aca hay q agarrar el hash header del mensaje get headers y buscarlo en el archivo para devolver 2 mil headers desde ese header
    let headers: Vec<BlockHeader> = Vec::new();
    let header_buscado = get_headers.start_block_hash;
    let mut payload: Vec<u8> = Vec::new();
    let mut msg = Vec::new();

    for header in headers {
        let header_bytes = header.serialize()?;
        payload.extend(header_bytes);
    }

    let header_msg = make_header("headers".to_string(), &payload)?;

    msg.extend_from_slice(&header_msg);
    msg.extend_from_slice(&payload);

    Ok(msg)
}
