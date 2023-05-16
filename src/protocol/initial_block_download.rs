use crate::blockchain::header::BlockHeader;
use crate::blockchain::node::Node;
use crate::errores::NodoBitcoinError;
use crate::messages::getheaders::GetHeadersMessage;
use crate::messages::header::check_header;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn get_headers(
    mut connections: Vec<TcpStream>,
    mut node: Node,
) -> Result<(), NodoBitcoinError> {
    let start_block = [
        0x00, 0x00, 0x00, 0x00, 0x09, 0x33, 0xea, 0x01, 0xad, 0x0e, 0xe9, 0x84, 0x20, 0x97, 0x79,
        0xba, 0xae, 0xc3, 0xce, 0xd9, 0x0f, 0xa3, 0xf4, 0x08, 0x71, 0x95, 0x26, 0xf8, 0xd7, 0x7f,
        0x49, 0x43,
    ];
    let _get_headers = GetHeadersMessage::new(70015, 1, start_block, [0; 32]);
    let message = GetHeadersMessage::serialize(&_get_headers)?;

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
            let (command, mut headers) = match check_header(&buffer) {
                Ok((command, payload_len)) => {
                    let mut headers = vec![0u8; payload_len];
                    if connection.read_exact(&mut headers).is_err() {
                        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
                    }
                    (command, headers)
                },
                Err(_) => continue,
            };
            println!("{}", command);
            if command == "headers" {
                // thread descargar datos
                
                let (size_bytes, num_headers) = parse_varint(&headers);
                headers = headers[size_bytes..].to_vec();

                for i in 0..num_headers {
                    let start: usize = i * 80;
                    let end: usize = start + 80;

                    let block_header = BlockHeader::deserialize(&headers[start..end])?;
                    node.add_header(block_header);
                }
                break; // descarga màs headers
            }
            println!("{:?} bytes read", bytes_read);
        }
    }
    Ok(())
}

fn parse_varint(bytes: &[u8]) -> (usize, usize) {
    let prefix = bytes[0];
    match prefix {
        0xfd => (3, u16::from_le_bytes([bytes[1], bytes[2]]) as usize),
        0xfe => (
            5,
            u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize,
        ),
        0xff => (
            9,
            u64::from_le_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            ]) as usize,
        ),
        _ => (1, u64::from(prefix) as usize),
    }
}
