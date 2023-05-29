use crate::{
    blockchain::{blockheader::BlockHeader, file::_leer_ultimo_header},
    common::utils_timestamp::_timestamp_to_datetime,
    config,
    errores::NodoBitcoinError,
    messages::{
        getheaders::GetHeadersMessage, headers::deserealize, messages_header::check_header,
    },
};

use super::connection::connect;

pub const GENESIS_BLOCK: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79, 0xba,
    0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f, 0x49, 0x43,
];

pub fn _version() -> Result<u32, NodoBitcoinError> {
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig),
    };
    Ok(version)
}

pub fn _get_last_header_hash() -> Result<[u8; 32], NodoBitcoinError> {
    let block_header = _get_last_header()?;
    let hash = block_header.hash()?;
    Ok(hash)
}

pub fn _get_last_header() -> Result<BlockHeader, NodoBitcoinError> {
    let bytes = _leer_ultimo_header()?;
    let block_header = BlockHeader::deserialize(&bytes)?;
    Ok(block_header)
}

pub fn _get_all_headers() -> Result<(), NodoBitcoinError> {
    let mut admin_connections = connect()?;
    let version = _version()?;
    let start_block = GENESIS_BLOCK;
    let get_headers = GetHeadersMessage::new(version, 1, start_block, [0; 32]);
    let mut get_headers_message = get_headers.serialize()?;

    let (connection, _id) = admin_connections.find_free_connection()?;
    connection.write_message(&get_headers_message)?;

    loop {
        let mut buffer = [0u8; 24];
        let bytes_read_option = connection.read_message(&mut buffer)?;
        match bytes_read_option {
            Some(read_bytes) => {
                if read_bytes == 0 {
                    let (connection, _id) = match admin_connections.find_free_connection() {
                        Ok(res) => res,
                        Err(NodoBitcoinError::NoSeEncuentraConexionLibre) => {
                            admin_connections = connect()?; // actualizo la lista de conexiones
                            admin_connections.find_free_connection()?
                        }
                        Err(_) => continue,
                    };
                    connection.write_message(&get_headers_message)?;
                    continue;
                }
            }
            None => continue,
        }
        let valid_command: bool;
        let (_command, headers) = match check_header(&buffer) {
            Ok((command, payload_len)) => {
                let mut headers = vec![0u8; payload_len];
                connection.read_exact_message(&mut headers)?;
                valid_command = command == "headers";
                if valid_command && payload_len == 1 {
                    break; // llegué al final de los headers
                }
                (command, headers)
            }
            Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                let (connection, _id) = admin_connections.find_free_connection()?;
                connection.write_message(&get_headers_message)?;
                continue;
            }
            Err(_) => continue,
        };

        if valid_command {
            get_headers_message = process_blockheaders(headers, version)?;
            connection.write_message(&get_headers_message)?;
        }
    }
    Ok(())
}

fn process_blockheaders(headers: Vec<u8>, version: u32) -> Result<Vec<u8>, NodoBitcoinError> {
    let blockheaders = deserealize(headers)?;
    let last_header = blockheaders[blockheaders.len() - 1];
    let datetime = _timestamp_to_datetime(last_header.time.into());
    println!(
        "Ultimo timestamp: {:?}",
        datetime.format("%Y-%m-%d %H:%M").to_string()
    );
    let hash = last_header.hash()?;
    let get_headers = GetHeadersMessage::new(version, 1, hash, [0; 32]);
    let get_headers_message = get_headers.serialize()?;
    Ok(get_headers_message)
}
