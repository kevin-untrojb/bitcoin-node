use bitcoin_hashes::{sha256d, Hash};
use crate::blockchain::block::{SerializedBlock};
use crate::blockchain::node::Node;
use crate::common::utils_bytes_conversion::obtener_timestamp_dia;
use crate::config;
use crate::errores::NodoBitcoinError;
use crate::messages::getdata::{GetDataMessage};
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::{deserealize};
use crate::messages::messages_header::check_header;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::{println, thread, vec, cmp};

use super::admin_connections::{AdminConnections, self};

pub fn get_headers(mut admin_connections: AdminConnections, node: &mut Node) -> Result<(), NodoBitcoinError> {
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig)
    };
    let start_block = [
        0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
        0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
        0x49, 0x43,
    ];
    let get_headers = GetHeadersMessage::new(version, 1, start_block, [0; 32]);
    let mut get_headers_message = GetHeadersMessage::serialize(&get_headers)?;

    let (connection, id) = admin_connections.find_free_connection()?;

    if connection.tcp.lock().unwrap().write(&get_headers_message).is_err() {
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }

    loop {

        let mut buffer = [0u8; 24];
        match connection.tcp.lock().unwrap().read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    let (connection, id) = admin_connections.change_connection(id)?;
                    break;
                }
            }
            Err(_) => continue,
        }
        let (command, headers) = match check_header(&buffer) {
            Ok((command, payload_len)) => {
                let mut headers = vec![0u8; payload_len];
                if connection.tcp.lock().unwrap().read_exact(&mut headers).is_err() {
                    return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                }
                (command, headers)
            }
            Err(_) => continue,
        };

        if command == "headers" {
            let blockheaders = deserealize(headers)?;
            let fecha_inicial = config::get_valor("DIA_INICIAL".to_string())?;
            let timestamp_ini = obtener_timestamp_dia(fecha_inicial);
            let last_header = blockheaders[blockheaders.len() - 1]; 
            let headers_filtrados: Vec<_> = blockheaders.into_iter().filter(|header| header.time >= timestamp_ini).collect();

            let n_threads_max:usize = match (config::get_valor("CANTIDAD_THREADS".to_string())?).parse::<usize>() {
                Ok(res) => res,
                Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig)
            };
            let n_threads = cmp::min(n_threads_max, headers_filtrados.len());
            let n_blockheaders_thread = (headers_filtrados.len() as f64 / n_threads as f64).ceil() as usize;
            let blocks = Arc::new(Mutex::new(vec![]));
            let mut threads = vec![];
            
            for i in 0..n_threads {

                let start: usize = i * n_blockheaders_thread;
                let end:usize = start + n_blockheaders_thread;
                let block_headers_thread = headers_filtrados[start..cmp::min(end, headers_filtrados.len())].to_vec();

                let shared_blocks = blocks.clone();

                let admin_connections_mutex = Arc::new(Mutex::new(admin_connections.clone()));
                let admin_connections_mutex_thread = admin_connections_mutex.clone();
                
                threads.push(thread::spawn(move || {

                    let mut admin_thread = admin_connections_mutex_thread.lock().unwrap();
                    let (thread_connection, thread_id_connection) = admin_thread.find_free_connection().unwrap();

                    for header in block_headers_thread {
                        let hash_header = match header.serialize() {
                            Ok(serialized_header) => serialized_header,
                            Err(_) => continue,
                        };
    
                        let get_data =
                            GetDataMessage::new(1, *sha256d::Hash::hash(&hash_header).as_byte_array());
                        let get_data_message = match GetDataMessage::serialize(&get_data) {
                            Ok(res) => res,
                            Err(_) => continue,
                        };
    
                        if thread_connection.tcp.lock().unwrap().write(&get_data_message).is_err() {
                            // throw/catch error
                        }

                        loop {
                            let mut thread_buffer= [0u8; 24];
                            match thread_connection.tcp.lock().unwrap().read(&mut thread_buffer) {
                                Ok(bytes_read) => {
                                    if bytes_read == 0 {
                                        println!("0 bytes read");
                                        /*let thread_connection = match admin_connections_mutex_thread.lock(){
                                            Ok(mut admin) => {
                                                thread_connection = admin.change_connection(thread_connection).unwrap();
                                                drop(admin_connections_mutex_thread);
                                                thread_connection
                                            },
                                            Err(_) => return,
                                        };*/
                                        break;
                                    }
                                    println!("{} bytes read getData", thread_buffer.len());
                                }
                                Err(_) => continue,
                            }
        
                            let (command, response_get_data) = match check_header(&thread_buffer) {
                                Ok((command, payload_len)) => {
                                    let mut response_get_data = vec![0u8; payload_len];
                                    if thread_connection.tcp.lock().unwrap().read_exact(&mut response_get_data).is_err() {
                                        // throw/catch error
                                    }
                                    (command, response_get_data)
                                }
                                Err(_) => {
                                    continue;
                                },
                            };
        
                            println!("{:?}", command);
        
                            if command == "block"{
                                let mut cloned = shared_blocks.lock().unwrap();
                                cloned.push(SerializedBlock::deserialize(&response_get_data));
                                println!("cloned: {}", cloned.len());
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
            get_headers_message = GetHeadersMessage::serialize(&get_headers)?;

            if connection.tcp.lock().unwrap().write(&get_headers_message).is_err() {
                return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
            }
        }
    }

    Ok(())
}
