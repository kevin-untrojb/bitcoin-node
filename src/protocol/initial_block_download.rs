use bitcoin_hashes::{sha256d, Hash};

use crate::blockchain::node::Node;
use crate::errores::NodoBitcoinError;
use crate::messages::getdata::{GetDataMessage, Inventory};
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::deserealize;
use crate::messages::messages_header::check_header;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::{thread, println};

pub fn get_headers(connections: Vec<TcpStream>, node: &mut Node) -> Result<(), NodoBitcoinError> {
    let start_block = [
        0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
        0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
        0x49, 0x43,
    ];
    let get_headers = GetHeadersMessage::new(70015, 1, start_block, [0; 32]);
    let mut get_headers_message = GetHeadersMessage::serialize(&get_headers)?;
    //let mut threads = vec![];

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
                deserealize(node, headers)?;
                /*let thread_node = node.clone();
                let mut thread_connection = match connection.try_clone() {
                    Ok(res) => res,
                    Err(_) => continue,
                };
                let mut thread_buffer = buffer.clone();*/

                //threads.push(thread::spawn(move || {
                    for header in node.get_headers() {
                        let hash_header = match header.serialize() {
                            Ok(serialized_header) => serialized_header,
                            Err(_) => continue,
                        };

                        let get_data = GetDataMessage::new(
                            1,
                            *sha256d::Hash::hash(&hash_header).as_byte_array(),
                        );
                        let get_data_message = match GetDataMessage::serialize(&get_data) {
                            Ok(res) => res,
                            Err(_) => continue,
                        };

                        if connection.write(&get_data_message).is_err() {
                            // throw/catch error
                        }

                        match connection.read(&mut buffer) {
                            Ok(bytes_read) => {
                                if bytes_read == 0 {
                                    println!("0 bytes read");
                                    break;
                                }
                                println!("{} bytes read getData", buffer.len());

                            }
                            Err(_) => continue,
                        }

                        let (command, getData) = match check_header(&buffer) {
                            Ok((command, payload_len)) => {
                                let mut headers = vec![0u8; payload_len];
                                if connection.read_exact(&mut headers).is_err() {
                                    // throw/catch error
                                }
                                (command, headers)
                            }
                            Err(_) => continue,
                        };

                        println!("{:?}", command);
                        // Agregar bloque a nodo
                    }
                //}));

                let get_headers = GetHeadersMessage::new(
                    70015,
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
