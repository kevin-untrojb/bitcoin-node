use crate::blockchain::file_manager::get_headers_from_file;
use crate::blockchain::file_manager::FileMessages;
use crate::{
    blockchain::blockheader::BlockHeader,
    common::utils_bytes::{self, parse_varint},
    errores::NodoBitcoinError,
};
use std::sync::mpsc::Sender;

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

pub fn make_headers_msg(
    file_manager_sender: Sender<FileMessages>,
    get_headers: GetHeadersMessage,
) -> Result<Vec<u8>, NodoBitcoinError> {
    let header_buscado = get_headers.start_block_hash;
    let mut payload: Vec<u8> = Vec::new();
    let mut msg = Vec::new();
    let mut header_deserelized: Vec<BlockHeader> = Vec::new();

    let headers: Vec<u8> = get_headers_from_file(file_manager_sender, header_buscado)?;

    let cantidad_headers = headers.len() / 80;

    for i in 0..cantidad_headers {
        let serialized_block = match BlockHeader::deserialize(&headers[i * 80..(i * 80) + 80]) {
            Ok(block) => block,
            Err(err) => return Err(err),
        };
        header_deserelized.push(serialized_block);
    }

    if cantidad_headers > 0 {
        let prefix = utils_bytes::from_amount_bytes_to_prefix(3);
        let count = utils_bytes::build_varint_bytes(prefix, cantidad_headers)?;

        payload.extend(count);

        for header in header_deserelized {
            let header_bytes = header.serialize()?;
            payload.extend(header_bytes);
            payload.push(0);
        }

        payload.pop();
    } else {
        payload.push(1_u8);
    }

    let header_msg = make_header("headers".to_string(), &payload)?;

    msg.extend_from_slice(&header_msg);
    msg.extend_from_slice(&payload);

    Ok(msg)
}
