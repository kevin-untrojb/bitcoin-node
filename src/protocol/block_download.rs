use std::{
    cmp,
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    blockchain::{
        block::SerializedBlock,
        blockheader::BlockHeader,
        file::{_header_count, _leer_headers, _leer_todos_headers},
    },
    common::utils_timestamp::{_timestamp_to_datetime, obtener_timestamp_dia},
    config,
    errores::NodoBitcoinError,
    messages::{
        getdata::GetDataMessage, headers::deserealize_desde_archivo, messages_header::check_header,
    },
};

use super::admin_connections::AdminConnections;

const DEFAULT_THREADS: usize = 10;

pub fn get_cantidad_threads(len_headers: usize) -> usize {
    let config_threades = config::get_valor("CANTIDAD_THREADS".to_string());
    let n_threads_str = match config_threades {
        Ok(n_threads_str) => n_threads_str,
        Err(_) => return cmp::min(DEFAULT_THREADS, len_headers),
    };
    let n_threads = n_threads_str.parse::<usize>();
    match n_threads {
        Ok(n_threads) => cmp::min(n_threads, len_headers),
        Err(_) => cmp::min(DEFAULT_THREADS, len_headers),
    }
}

fn headers_por_threads(headers_filtrados: Vec<BlockHeader>) -> HashMap<usize, Vec<BlockHeader>> {
    let n_threads = get_cantidad_threads(headers_filtrados.len());
    let n_blockheaders_thread = (headers_filtrados.len() as f64 / n_threads as f64).ceil() as usize;
    let mut headers_por_threads = HashMap::new();
    for i in 0..n_threads {
        let start: usize = i * n_blockheaders_thread;
        let end: usize = cmp::min(start + n_blockheaders_thread, headers_filtrados.len());
        headers_por_threads.insert(i, headers_filtrados[start..end].to_vec());
    }
    headers_por_threads
}

pub fn get_blocks(mut admin_connections: AdminConnections) -> Result<(), NodoBitcoinError> {
    let mut headers = _leer_headers(0)?;
    let header_count = _header_count()?;
    let total_vueltas = header_count / 2000;

    //let total_headers = _header_count()?;

    //loop {
    //headers = _leer_header_desde_archivo(header_ix)?;
    let blockheaders = deserealize_desde_archivo(headers)?;
    let fecha_inicial = config::get_valor("DIA_INICIAL".to_string())?;
    let timestamp_ini = obtener_timestamp_dia(fecha_inicial);
    let last_header = blockheaders[blockheaders.len() - 1];
    let datetime = _timestamp_to_datetime(last_header.time.into());
    let letn_blockheaders = blockheaders.len();
    let headers_filtrados: Vec<_> = blockheaders
        .into_iter()
        .filter(|header| header.time >= timestamp_ini)
        .collect();

    println!(
            "Descarga de headers. Total obtenidos: {:?}, bloques a descargar: {:?}. Ultimo timestamp: {:?}",
            letn_blockheaders,
            headers_filtrados.len(),
            datetime.format("%Y-%m-%d %H:%M").to_string()
        );

    let n_threads = get_cantidad_threads(headers_filtrados.len());
    let headers_por_threads = headers_por_threads(headers_filtrados);

    let blocks = Arc::new(Mutex::new(vec![]));
    let mut threads = vec![];

    let admin_connections_mutex = Arc::new(Mutex::new(admin_connections.clone()));

    for i in 0..n_threads {
        let block_headers_thread = headers_por_threads[&i].clone();
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
                let hash_header = match header.hash() {
                    Ok(hash_header) => hash_header,
                    Err(_) => continue,
                };
                let get_data = GetDataMessage::new(1, hash_header);
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
                                        match admin.change_connection(thread_id_connection) {
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
                                            match admin.change_connection(thread_id_connection) {
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

    Ok(())
}
