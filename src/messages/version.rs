use std::net::SocketAddr;

use crate::{errores::NodoBitcoinError, messages::messages_header::make_header};
pub struct VersionMessage {
    version: u32,
    services: u64,
    timestamp: u64,
    addr_recv_services: u64,
    addr_recv_ip: String,
    addr_recv_port: u16,
    addr_trans_services: u64,
    addr_trans_ip: String,
    addr_trans_port: u16,
    nonce: u64,
    user_agent_bytes: u8,
    user_agent: String,
    start_height: i32,
    relay: u8,
}

const DEFAULT_SERVICES: u64 = 0;
const DEFAULT_NONCE: u64 = 0;
const DEFAULT_TRANS_PORT: u16 = 18333;
const DEFAULT_TRANS_IP: &str = "192.168.0.66";
const DEFAULT_USER_AGENT_BYTES: u8 = 0;
const DEFAULT_USER_AGENT: &str = "5";
const DEFAULT_START_HEIGHT: i32 = 0;
const DEFAULT_RELAY: u8 = 1;

impl VersionMessage {
    fn string_to_bytes(s: &str, fixed_size: usize) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        match bytes.len() < fixed_size {
            true => bytes.resize(fixed_size, 0),
            false => bytes.truncate(fixed_size),
        }
        bytes
    }
    pub fn new(version: u32, timestamp: u64, addr_recv_socket: SocketAddr) -> VersionMessage {
        let services = DEFAULT_SERVICES;
        let addr_trans_services = DEFAULT_SERVICES;

        let addr_recv_services = DEFAULT_SERVICES;
        let addr_recv_ip = addr_recv_socket.ip().to_string();
        let addr_recv_port = addr_recv_socket.port();

        let addr_trans_ip = DEFAULT_TRANS_IP.to_string();
        let addr_trans_port = DEFAULT_TRANS_PORT;
        let nonce = DEFAULT_NONCE;
        let user_agent_bytes = DEFAULT_USER_AGENT_BYTES;
        let user_agent = DEFAULT_USER_AGENT.to_string();
        let start_height = DEFAULT_START_HEIGHT;
        let relay = DEFAULT_RELAY;

        VersionMessage {
            version,
            services,
            timestamp,
            addr_recv_services,
            addr_recv_ip,
            addr_recv_port,
            addr_trans_services,
            addr_trans_ip,
            addr_trans_port,
            nonce,
            user_agent_bytes,
            user_agent,
            start_height,
            relay,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut payload = Vec::new();
        let mut msg = Vec::new();

        payload.extend_from_slice(&(self.version).to_le_bytes());
        payload.extend_from_slice(&(self.services).to_le_bytes());
        payload.extend_from_slice(&(self.timestamp).to_le_bytes());
        payload.extend_from_slice(&(self.addr_recv_services).to_le_bytes());
        payload.extend_from_slice(&Self::string_to_bytes(&self.addr_recv_ip, 16));
        payload.extend_from_slice(&(self.addr_recv_port).to_be_bytes());
        payload.extend_from_slice(&(self.addr_trans_services).to_le_bytes());
        payload.extend_from_slice(&Self::string_to_bytes(&self.addr_trans_ip, 16));
        payload.extend_from_slice(&(self.addr_trans_port).to_be_bytes());
        payload.extend_from_slice(&(self.nonce).to_le_bytes());
        payload.extend_from_slice(&(self.user_agent_bytes).to_le_bytes());
        if self.user_agent_bytes != 0 {
            payload.extend_from_slice(self.user_agent.as_bytes());
        }
        payload.extend_from_slice(&(self.start_height).to_le_bytes());
        payload.extend_from_slice(&(self.relay).to_le_bytes());

        let header = make_header("version".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);

        Ok(msg)
    }
}
