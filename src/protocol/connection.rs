use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;
use chrono::Utc;

use crate::config;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// A struct representing a Version Message
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/p2p_networking.html#version
///
/// # Fields

pub struct VersionMessage {
    // version: i32,
    // services: u64,
    // timestamp: u64,
    // addr_recv_services: u64,
    // addr_recv_ip: String,
    // addr_recv_port: u64,
    // addr_trans_services: u64,
    // addr_trans_ip: String,
    // addr_trans_port: u64,
    // nonce: u64,
    // user_agent_bytes: i32,
    // user_agent: String,
    // start_height: i32,
    // relay: bool,
}

impl VersionMessage {
    pub fn connect() {
        let addresses = get_address();

        for address in addresses.iter() {
            println!("Address: {:?}", address);

            let result_socket = TcpStream::connect(address);
            if result_socket.is_err() {
                println!("Error: {:?}", result_socket.err());
                continue;
            }

            let mut socket = result_socket.unwrap();

            let mut payload = Vec::new();
            let version: u32 = 70015; // versión del protocolo utilizada por el remitente
            let services: u64 = 0; // los servicios que ofrece el remitente (0 = ningún servicio)

            let timestamp = Utc::now().timestamp();

            // let timestamp = SystemTime::now()
            //     .duration_since(UNIX_EPOCH)
            //     .unwrap()
            //     .as_secs(); // hora actual del remitente.

            let addr_recv_services: u64 = 1;
            //let addr_recv_ip = address.ip().to_string();
            let mut addr_recv_ip_16_bytes = [0; 16];
            let _addr_recv_ip_bytes = match address.ip() {
                IpAddr::V4(ip) => addr_recv_ip_16_bytes[..4].copy_from_slice(&ip.octets().to_vec()),
                IpAddr::V6(ip) => addr_recv_ip_16_bytes.copy_from_slice(&ip.octets().to_vec()),
            };

            let addr_recv_ip_bytes = addr_recv_ip_16_bytes; //string_to_bytes(&addr_recv_ip, 16);

            let addr_recv_port = address.port();

            let addr_trans_services: u64 = 0;
            let addr_trans_ip_v4 = Ipv4Addr::new(181, 165, 131, 147);
            let mut addr_trans_ip_16_bytes = [0; 16];
            addr_trans_ip_16_bytes[..4].copy_from_slice(&addr_trans_ip_v4.octets().to_vec());
            //let addr_trans_ip = "181.165.131.147".to_string();
            let addr_trans_ip_bytes = addr_trans_ip_16_bytes; //string_to_bytes(&addr_trans_ip, 16);

            // let addr_trans_ip = [
            //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
            //     0x00, 0x00,
            // ]; // "127.0.0.1".to_string(); //181.165.131.147
            let addr_trans_port = 18333 as u16;

            let nonce: u64 = 0; // número aleatorio generado por el remitente

            //let user_agent = "/my-node:1.0.0/".to_string(); // software utilizado por el remitente
            let user_agent_bytes: u8 = 0;
            let start_height: u32 = 0; // altura del bloque del remitente
            let relay = true; // el remitente desea recibir transacciones adicionales después de recibir el mensaje "inv"

            payload.write_all(&((version).to_le_bytes())).unwrap();
            payload.write_all(&(services).to_le_bytes()).unwrap();
            payload.write_all(&(timestamp).to_le_bytes()).unwrap();

            payload
                .write_all(&(addr_recv_services).to_le_bytes())
                .unwrap();
            payload.write_all(&(addr_recv_ip_bytes)).unwrap();
            payload.write_all(&(addr_recv_port).to_be_bytes()).unwrap();

            payload
                .write_all(&(addr_trans_services).to_le_bytes())
                .unwrap();
            payload.write_all(&(addr_trans_ip_bytes)).unwrap();
            payload.write_all(&(addr_trans_port).to_be_bytes()).unwrap();

            payload.write_all(&(nonce).to_le_bytes()).unwrap();
            //payload.write_all(&(user_agent).as_bytes()).unwrap();
            payload
                .write_all(&(user_agent_bytes).to_le_bytes())
                .unwrap();
            payload.write_all(&(start_height).to_le_bytes()).unwrap();
            payload.write_all(&(relay as u8).to_le_bytes()).unwrap();

            // Cabecera del mensaje
            let mut cabecera = Vec::new();
            let net_magic_testnet = [0x0b, 0x11, 0x09, 0x07];
            let command = string_to_bytes("version", 12);
            let payload_len = payload.len() as u32;
            let hash = sha256d::Hash::hash(&payload);
            let checksum = string_to_bytes(&hash.to_string(), 4);

            cabecera.write_all(&(net_magic_testnet)).unwrap();
            cabecera.write_all(&(command)).unwrap();
            cabecera.write_all(&(payload_len).to_le_bytes()).unwrap();
            cabecera.write_all(&checksum).unwrap();

            let mut mensaje = Vec::new();
            mensaje.write_all(&cabecera).unwrap();
            mensaje.write_all(&payload).unwrap();

            //let result = socket.write(&mensaje).unwrap();
            socket.write_all(&mensaje).unwrap();

            println!("{} bytes sent version", mensaje.len());

            // let message_payload = read_message(&mut socket);
            // println!("Message payload: {:?}", message_payload);

            let mut num_buffer = [0u8; 1024];

            let lectura = socket.read(&mut num_buffer);

            println!("{} bytes read version", lectura.unwrap());

            // Cabecera del verack
            let mut verack_message = Vec::new();
            let net_magic_testnet_verack = [0x0b, 0x11, 0x09, 0x07];
            let command_verack = string_to_bytes("verack", 12);
            let payload_len_verack: u32 = 0;
            let payload_verack: Vec<u8> = Vec::new();
            let hash_verack = sha256d::Hash::hash(&payload_verack);

            let checksum_verack = string_to_bytes(&hash_verack.to_string(), 4);

            verack_message
                .write_all(&(net_magic_testnet_verack))
                .unwrap();
            verack_message.write_all(&(command_verack)).unwrap();
            verack_message
                .write_all(&(payload_len_verack).to_be_bytes())
                .unwrap();
            verack_message.write_all(&checksum_verack).unwrap();

            //let result = socket.write(&mensaje).unwrap();
            socket.write_all(&verack_message).unwrap();

            println!("{} bytes sent verack", verack_message.len());

            let mut num_buffer_verack = [0u8; 24];

            let lectura_verack = socket.read_exact(&mut num_buffer_verack);

            if lectura_verack.is_err() {
                println!("Error al leer el verack");
            } else {
                println!("{:?} bytes read verack", num_buffer_verack);
            }
        }
    }
}

fn _read_message(stream: &mut TcpStream) -> Vec<u8> {
    let mut header = [0; 24];
    stream.read_exact(&mut header).unwrap();

    let _magic_bytes = &header[..4];
    let _command = String::from_utf8(header[4..16].iter().map(|&b| b).collect()).unwrap();
    let payload_size = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
    let checksum = &header[20..24];

    let mut payload = vec![0; payload_size as usize];
    stream.read_exact(&mut payload).unwrap();

    let calculated_checksum = bitcoin_hashes::sha256d::Hash::hash(&payload)[..4].to_vec();
    assert_eq!(checksum.to_vec(), calculated_checksum, "Invalid checksum");

    payload
}

pub fn get_address() -> Vec<SocketAddr> {
    let mut seeds = Vec::new();
    let url = config::get_valor("ADDRESS".to_owned()).unwrap();
    let port = 18333;

    let seedhost = format!("{}:{}", url, port);

    if let Ok(lookup) = seedhost.to_socket_addrs() {
        for host in lookup {
            //return Ok(host);
            seeds.push(host);
        }
    }
    //Err(NodoBitcoinError::NoSeEncontroURL)
    seeds
}

fn string_to_bytes(s: &str, fixed_size: usize) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    match bytes.len() < fixed_size {
        true => bytes.resize(fixed_size, 0),
        false => bytes.truncate(fixed_size),
    }
    bytes
}

// fn build_version_message() -> Vec<u8> {
//     let mut msg: Vec<u8> = vec![];

//     // Cabecera del mensaje
//     msg.extend(&TESTNET_MAGIC.to_le_bytes()); // magic bytes
//     msg.extend(b"version".iter().take(12)); // comando
//     msg.extend(&28u32.to_le_bytes()); // longitud del payload

//     // Payload
//     msg.extend(&70015u32.to_le_bytes()); // version del protocolo
//     msg.extend(&1u64.to_le_bytes()); // servicios ofrecidos por el nodo
//     msg.extend(&u64::from_be_bytes(
//         hex::decode("0e3f5b8d7f5c9ee9").unwrap(),
//     )); // timestamp
//     msg.extend(&u64::from_be_bytes([0, 0, 0, 0, 0, 0, 0, 0]).to_le_bytes()); // servicios del receptor
//     msg.extend(&[0; 16]); // dirección IP del receptor
//     msg.extend(&[0, 0]); // puerto del receptor
//     msg.extend(&u64::from_be_bytes([0, 0, 0, 0, 0, 0, 0, 0]).to_le_bytes()); // servicios del transmisor
//     msg.extend(&[0; 16]); // dirección IP del transmisor
//     msg.extend(&[0, 0]); // puerto del transmisor
//     msg.extend(&u64::from_be_bytes(
//         hex::decode("7f34c607d8c2a1cd").unwrap(),
//     )); // nonce
//     msg.push(0); // longitud de user agent string
//     msg.push(0); // user agent string (vacío)
//     msg.extend(&0u32.to_le_bytes()); // último bloque conocido
//     msg.push(0); // "relay" flag (falso)

//     // Checksum
//     let checksum = sha256d(&msg);
//     msg.extend(&checksum[..4]);

//     msg
// }

// fn sha256d(data: &[u8]) -> [u8; 32] {
//     let hasher = sha256::Hash::hash(&data);
//     let hasher.to_byte_array();
// }
