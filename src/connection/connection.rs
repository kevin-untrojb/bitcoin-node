use bitcoin_hashes::sha256;
use bitcoin_hashes::Hash;

use crate::config;
use std::io::Read;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs};
use std::result;
use std::{
    net::TcpStream,
    time::{SystemTime, UNIX_EPOCH},
};

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
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(); // hora actual del remitente
            let addr_recv_services: u64 = 0;
            let addr_recv_ip = address.ip().to_string();
            let addr_recv_ip_bytes = string_to_bytes(&addr_recv_ip, 16);

            let addr_recv_port = address.port();

            let addr_trans_services: u64 = 0;
            let addr_trans_ip = "181.165.131.147".to_string();
            let addr_trans_ip_bytes = string_to_bytes(&addr_trans_ip, 16);

            // let addr_trans_ip = [
            //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
            //     0x00, 0x00,
            // ]; // "127.0.0.1".to_string(); //181.165.131.147
            let addr_trans_port: u16 = 18333;

            let nonce: u64 = 0; // número aleatorio generado por el remitente

            //let user_agent = "/my-node:1.0.0/".to_string(); // software utilizado por el remitente
            let user_agent_bytes: u8 = 0;
            let start_height: u32 = 0; // altura del bloque del remitente
            let relay = true; // el remitente desea recibir transacciones adicionales después de recibir el mensaje "inv"

            payload
                .write_all(&((version as u32).to_le_bytes()))
                .unwrap();
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
            let command = [
                0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];
            let payload_len = payload.len() as u32;
            let hash = sha256::Hash::hash(&payload);
            let checksum = &hash[..4];

            cabecera.write_all(&(net_magic_testnet)).unwrap();
            cabecera.write_all(&(command)).unwrap();
            cabecera.write_all(&(payload_len).to_le_bytes()).unwrap();
            cabecera.write_all(&(checksum)).unwrap();

            let mut mensaje = Vec::new();
            mensaje.write_all(&(&cabecera)).unwrap();
            mensaje.write_all(&(&payload)).unwrap();

            //let result = socket.write(&mensaje).unwrap();
            socket.write_all(&mensaje).unwrap();

            println!("{} bytes sent", mensaje.len());

            let mut num_buffer = [0u8; 1024];

            let lectura = socket.read(&mut num_buffer);

            println!("{} bytes read", lectura.unwrap());
        }
    }
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
    if bytes.len() < fixed_size {
        bytes.resize(fixed_size, 0);
    } else if bytes.len() > fixed_size {
        bytes.truncate(fixed_size);
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
