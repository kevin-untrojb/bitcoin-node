use crate::config;
use std::{net::TcpStream, time::{UNIX_EPOCH, SystemTime}};
use std::io::{Write};


/// A struct representing a Version Message
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/p2p_networking.html#version
///
/// # Fields

pub struct _VersionMessage {
    version: i32,
    services: u64,
    timestamp: u64,
    addr_recv_services: u64,
    addr_recv_ip: String,
    addr_recv_port: u64,
    addr_trans_services: u64,
    addr_trans_ip: String,
    addr_trans_port: u64,
    nonce: u64,
    user_agent_bytes: i32,
    user_agent: String,
    start_height: i32,
    relay: bool,
}

impl _VersionMessage {

    fn connect(){
        let address = config::get_valor("ADDRESS".to_owned()).unwrap();
        let mut socket = TcpStream::connect(address).unwrap();

        let mut payload = Vec::new();
        let version = 70015; // versión del protocolo utilizada por el remitente
        let services: u64 = 0; // los servicios que ofrece el remitente (0 = ningún servicio)
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(); // hora actual del remitente
        let addr_recv_services:u32 = 1;
        let addr_recv_ip = "127.0.0.1".to_string();
        let addr_recv_port:u32 = 8333;
        let addr_trans_services:u32 = 1;
        let addr_trans_ip = "127.0.0.1".to_string();
        let addr_trans_port:u32 = 8333;
        let nonce:u32 = 123456; // número aleatorio generado por el remitente
        let user_agent = "/my-node:1.0.0/".to_string(); // software utilizado por el remitente
        let start_height:u32 = 0; // altura del bloque del remitente
        let relay = true; // el remitente desea recibir transacciones adicionales después de recibir el mensaje "inv"
        
        payload.write_all(&((version as u32).to_le_bytes())).unwrap();
        payload.write_all(&(services).to_le_bytes()).unwrap();
        payload.write_all(&(timestamp).to_le_bytes()).unwrap();
        payload.write_all(&(addr_recv_services).to_le_bytes()).unwrap();
        payload.write_all(&(addr_recv_ip).as_bytes()).unwrap();
        payload.write_all(&(addr_recv_port).to_le_bytes()).unwrap();
        payload.write_all(&(addr_trans_services).to_le_bytes()).unwrap();
        payload.write_all(&(addr_trans_ip).as_bytes()).unwrap();
        payload.write_all(&(addr_trans_port).to_le_bytes()).unwrap();
        payload.write_all(&(nonce).to_le_bytes()).unwrap();
        payload.write_all(&(user_agent).as_bytes()).unwrap();
        payload.write_all(&(start_height).to_le_bytes()).unwrap();
        payload.write_all(&(relay as u8).to_le_bytes()).unwrap();
        
        let result = socket.write(&payload).unwrap();

    }
}
