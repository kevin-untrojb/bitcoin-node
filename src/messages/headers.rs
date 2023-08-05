use crate::{
    blockchain::{
        blockheader::BlockHeader,
        file::{buscar_header, leer_primeros_2mil_headers},
    },
    common::utils_bytes::{self, parse_varint},
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
    let header_buscado = get_headers.start_block_hash;
    let mut payload: Vec<u8> = Vec::new();
    let mut msg = Vec::new();
    let mut header_deserelized: Vec<BlockHeader> = Vec::new();

    let headers = buscar_header(header_buscado)?;
    let cantidad_headers = headers.len() / 80;
    //headers.extend(leer_primeros_2mil_headers().unwrap());
    for i in 0..cantidad_headers {
        header_deserelized.push(BlockHeader::deserialize(&headers[i * 80..(i * 80) + 80]).unwrap());
    }

    let prefix = utils_bytes::from_amount_bytes_to_prefix(3);
    let count = utils_bytes::build_varint_bytes(prefix, 2000)?;

    payload.extend(count);

    for header in header_deserelized {
        let header_bytes = header.serialize()?;
        payload.extend(header_bytes);
        payload.push(0);
    }

    let header_msg = make_header("headers".to_string(), &payload)?;

    msg.extend_from_slice(&header_msg);
    msg.extend_from_slice(&payload);

    Ok(msg)
}
