use crate::blockchain::block::SerializedBlock;
use crate::blockchain::node::Node;
use crate::common::utils_timestamp::obtener_timestamp_dia;
use crate::config;
use crate::errores::NodoBitcoinError;
use crate::messages::getdata::GetDataMessage;
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::deserealize;
use crate::messages::messages_header::check_header;
use bitcoin_hashes::{sha256d, Hash};
use std::sync::{Arc, Mutex};
use std::{cmp, println, thread, vec};

use super::admin_connections::AdminConnections;

pub fn genesis_block() -> [u8; 32] {
    let start_block: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
        0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
        0x49, 0x43,
    ];
    start_block
}
pub fn get_headers(
    mut admin_connections: AdminConnections,
    _node: &mut Node,
) -> Result<(), NodoBitcoinError> {
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig),
    };
    let start_block = genesis_block();
    let get_headers = GetHeadersMessage::new(version, 1, start_block, [0; 32]);
    let mut get_headers_message = get_headers.serialize()?;

    let (connection, id) = admin_connections.find_free_connection()?;
    connection.write_message(&get_headers_message)?;

    loop {
        let mut buffer = [0u8; 24];
        let bytes_read_option = connection.read_message(&mut buffer)?;
        match bytes_read_option {
            Some(read_bytes) => {
                if read_bytes == 0 {
                    break;
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
                (command, headers)
            }
            Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                let (connection, _id) = admin_connections.change_connection(id)?;
                connection.write_message(&get_headers_message)?;
                continue;
            }
            Err(_) => continue,
        };

        if valid_command {
            let blockheaders = deserealize(headers)?;
            let fecha_inicial = config::get_valor("DIA_INICIAL".to_string())?;
            let timestamp_ini = obtener_timestamp_dia(fecha_inicial);
            let last_header = blockheaders[blockheaders.len() - 1];
            let headers_filtrados: Vec<_> = blockheaders
                .into_iter()
                .filter(|header| header.time >= timestamp_ini)
                .collect();

            let n_threads_max: usize =
                match (config::get_valor("CANTIDAD_THREADS".to_string())?).parse::<usize>() {
                    Ok(res) => res,
                    Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig),
                };
            let n_threads = cmp::min(n_threads_max, headers_filtrados.len());
            let n_blockheaders_thread =
                (headers_filtrados.len() as f64 / n_threads as f64).ceil() as usize;
            let blocks = Arc::new(Mutex::new(vec![]));
            let mut threads = vec![];

            let admin_connections_mutex = Arc::new(Mutex::new(admin_connections.clone()));
            println!(
                "Descarga de headers. Total obtenidos: {:?}",
                headers_filtrados.len()
            );

            for i in 0..n_threads {
                let start: usize = i * n_blockheaders_thread;
                let end: usize = start + n_blockheaders_thread;
                let block_headers_thread =
                    headers_filtrados[start..cmp::min(end, headers_filtrados.len())].to_vec();
                let shared_blocks = blocks.clone();
                let admin_connections_mutex_thread = admin_connections_mutex.clone();
                threads.push(thread::spawn(move || {
                    let (mut cloned_connection, mut thread_id_connection) =
                        match admin_connections_mutex_thread.lock() {
                            Ok(mut admin) => {
                                let (thread_connection, thread_id_connection) =
                                    match admin.find_free_connection() {
                                        Ok((connection, id)) => (connection, id),
                                        Err(_) => return,
                                    };
                                drop(admin);
                                (thread_connection, thread_id_connection)
                            }
                            Err(_) => return,
                        };

                    for header in block_headers_thread {
                        let hash_header = match header.serialize() {
                            Ok(serialized_header) => serialized_header,
                            Err(_) => continue,
                        };

                        let get_data = GetDataMessage::new(
                            1,
                            *sha256d::Hash::hash(&hash_header).as_byte_array(),
                        );
                        let get_data_message = match get_data.serialize() {
                            Ok(res) => res,
                            Err(_) => continue,
                        };
                        if cloned_connection.write_message(&get_data_message).is_err() {
                            return;
                        }
                        loop {
                            let mut change_connection: bool = false;
                            let mut thread_buffer = [0u8; 24];

                            let thread_bytes_read_result =
                                cloned_connection.read_message(&mut thread_buffer);
                            match thread_bytes_read_result {
                                Ok(thread_bytes_read_option) => match thread_bytes_read_option {
                                    Some(read_bytes) => {
                                        if read_bytes == 0 {
                                            change_connection = true;
                                        }
                                    }
                                    None => continue,
                                },
                                Err(_) => return,
                            }
                            if change_connection {
                                (cloned_connection, thread_id_connection) =
                                    match admin_connections_mutex_thread.lock() {
                                        Ok(mut admin) => {
                                            let (thread_connection, thread_id_connection) =
                                                match admin.change_connection(thread_id_connection)
                                                {
                                                    Ok((connection, id)) => (connection, id),
                                                    Err(_) => continue,
                                                };
                                            drop(admin);
                                            (thread_connection.clone(), thread_id_connection)
                                        }
                                        Err(_) => return,
                                    };
                                if cloned_connection.write_message(&get_data_message).is_err() {
                                    return;
                                }
                                continue;
                            }

                            let valid_command: bool;
                            let (_command, response_get_data) = match check_header(&thread_buffer) {
                                Ok((command, payload_len)) => {
                                    let mut response_get_data = vec![0u8; payload_len];

                                    if cloned_connection
                                        .read_exact_message(&mut response_get_data)
                                        .is_err()
                                    {
                                        return;
                                    }
                                    valid_command = command == "block";
                                    (command, response_get_data)
                                }
                                Err(NodoBitcoinError::MagicNumberIncorrecto) => {
                                    (cloned_connection, thread_id_connection) =
                                        match admin_connections_mutex_thread.lock() {
                                            Ok(mut admin) => {
                                                let (thread_connection, thread_id_connection) =
                                                    match admin
                                                        .change_connection(thread_id_connection)
                                                    {
                                                        Ok((connection, id)) => (connection, id),
                                                        Err(_) => continue,
                                                    };
                                                drop(admin);
                                                (thread_connection.clone(), thread_id_connection)
                                            }
                                            Err(_) => return,
                                        };
                                    if cloned_connection.write_message(&get_data_message).is_err() {
                                        return;
                                    }
                                    continue;
                                }
                                Err(_) => {
                                    continue;
                                }
                            };

                            if valid_command {
                                let mut cloned = shared_blocks.lock().unwrap();
                                cloned.push(SerializedBlock::deserialize(&response_get_data));
                                println!("Bloque #{} descargado", cloned.len());
                                drop(cloned);
                                break;
                            }
                        }
                    }
                }));
            }

            for thread in threads {
                let _ = thread.join();
            }

            let get_headers = GetHeadersMessage::new(
                version,
                1,
                *sha256d::Hash::hash(&last_header.serialize()?).as_byte_array(),
                [0; 32],
            );
            get_headers_message = get_headers.serialize()?;
            connection.write_message(&get_headers_message)?;
        }
    }

    Ok(())
}
