use bitcoin_hashes::{sha256d, Hash};

use crate::blockchain::block::SerializedBlock;
use crate::blockchain::blockheader::BlockHeader;
use crate::blockchain::node::Node;
use crate::config;
use crate::errores::NodoBitcoinError;
use crate::messages::getdata::{GetDataMessage, Inventory};
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::deserealize;
use crate::messages::messages_header::check_header;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::{println, thread, vec};

pub fn get_headers(connections: Vec<TcpStream>, node: &mut Node) -> Result<(), NodoBitcoinError> {
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
    let mut threads = vec![];

    for mut connection in connections {
        if connection.write(&get_headers_message).is_err() {
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }

        let mut buffer = [0u8; 24];

        loop {
            match connection.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        println!("0 bytes read");
                        break;
                    }
                }
                Err(_) => continue,
            }
            let (command, headers) = match check_header(&buffer) {
                Ok((command, payload_len)) => {
                    let mut headers = vec![0u8; payload_len];
                    if connection.read_exact(&mut headers).is_err() {
                        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                    }
                    (command, headers)
                }
                Err(_) => continue,
            };

            if command == "headers" {
                let blockheaders = deserealize(headers)?;
                
                let n_threads:usize = match (config::get_valor("CANTIDAD_THREADS".to_string())?).parse::<usize>() {
                    Ok(res) => res,
                    Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig)
                };
                let n_blockheaders_thread = blockheaders.len() as usize/n_threads;
                let blocks = Arc::new(Mutex::new(vec![]));

                for i in 0..n_threads {
                    let start: usize = i * n_blockheaders_thread;
                    let end:usize = start + n_blockheaders_thread;
                    let mut block_headers_thread = vec![];
                    block_headers_thread.clone_from_slice(&blockheaders[start..end]);

                    let shared_blocks = blocks.clone();

                    let mut thread_connection = match connection.try_clone() {
                        Ok(res) => res,
                        Err(_) => continue,
                    };
                    let mut thread_buffer = buffer.clone();
            
                    threads.push(thread::spawn(move || {

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
        
                            if thread_connection.write(&get_data_message).is_err() {
                                // throw/catch error
                            }
        
                            match thread_connection.read(&mut thread_buffer) {
                                Ok(bytes_read) => {
                                    if bytes_read == 0 {
                                        println!("0 bytes read");
                                        break;
                                    }
                                    println!("{} bytes read getData", thread_buffer.len());
                                }
                                Err(_) => continue,
                            }
        
                            let (command, response_get_data) = match check_header(&thread_buffer) {
                                Ok((command, payload_len)) => {
                                    let mut response_get_data = vec![0u8; payload_len];
                                    if thread_connection.read_exact(&mut response_get_data).is_err() {
                                        // throw/catch error
                                    }
                                    (command, response_get_data)
                                }
                                Err(_) => continue,
                            };
        
                            println!("{:?}", command);
        
                            if command == "block"{
                                let mut cloned = shared_blocks.lock().unwrap();
                                cloned.push(SerializedBlock::new(header));
                                drop(cloned);
                                // deserealize response_get_data
                                /*
                                    lock al vector clonado
                                    agregar bloque al vector clonado
                                    drop vector clonado
                                 */
                            }
                        }
                    }));
                }

                let get_headers = GetHeadersMessage::new(
                    version,
                    1,
                    *sha256d::Hash::hash(&node.get_last_header().serialize()?).as_byte_array(),
                    [0; 32],
                );
                get_headers_message = GetHeadersMessage::serialize(&get_headers)?;

                if connection.write(&get_headers_message).is_err() {
                    return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                }
            }
        }
    }

    //Join threads
    Ok(())
}
