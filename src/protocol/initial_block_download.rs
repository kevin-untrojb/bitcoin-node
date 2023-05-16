use bitcoin_hashes::{sha256d, Hash};

use crate::blockchain::node::Node;
use crate::errores::NodoBitcoinError;
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::headers::deserealize;
use crate::messages::messages_header::check_header;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn get_headers(
    connections: Vec<TcpStream>,
    node: &mut Node,
) -> Result<(), NodoBitcoinError> {
    let start_block = [
        0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
        0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
        0x49, 0x43,
    ];
    let get_headers = GetHeadersMessage::new(70015, 1, start_block, [0; 32]);
    let mut message = GetHeadersMessage::serialize(&get_headers)?;

    for mut connection in connections {
        println!("{:?}", connection);

        if connection.write(&message).is_err() {
            return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
        }
        println!("{} bytes sent getHeaders", message.len());

        let mut buffer = [0u8; 24];

        loop {
            let bytes_read = connection.read(&mut buffer).unwrap(); //unwrap() para probar
            if bytes_read == 0 {
                println!("0 bytes read");
                break;
            }
            let (command, headers) = match check_header(&buffer) {
                Ok((command, payload_len)) => {
                    let mut headers = vec![0u8; payload_len];
                    if connection.read_exact(&mut headers).is_err() {
                        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                    }
                    (command, headers)
                },
                Err(_) => continue,
            };

            if command == "headers" {
                deserealize(node, headers)?;

                let get_headers = GetHeadersMessage::new(70015, 1, *sha256d::Hash::hash(&node.get_last_header().serialize()?).as_byte_array(), [0; 32]);
                message = GetHeadersMessage::serialize(&get_headers)?; 

                if connection.write(&message).is_err() {
                    return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                }       
            }                    
        }
    }
    Ok(())
}

