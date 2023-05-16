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
    user_agent_bytes: u32,
    user_agent: String,
    start_height: u32,
    relay: bool,
}

impl VersionMessage {
    fn string_to_bytes(s: &str, fixed_size: usize) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        match bytes.len() < fixed_size {
            true => bytes.resize(fixed_size, 0),
            false => bytes.truncate(fixed_size),
        }
        bytes
    }

    pub fn new(
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
        user_agent_bytes: u32,
        user_agent: String,
        start_height: u32,
        relay: bool,
    ) -> VersionMessage {
        VersionMessage {
            version: version,
            services: services,
            timestamp: timestamp,
            addr_recv_services: addr_recv_services,
            addr_recv_ip: addr_recv_ip,
            addr_recv_port: addr_recv_port,
            addr_trans_services: addr_trans_services,
            addr_trans_ip: addr_trans_ip,
            addr_trans_port: addr_trans_port,
            nonce: nonce,
            user_agent_bytes: user_agent_bytes,
            user_agent: user_agent,
            start_height: start_height,
            relay: relay,
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
            payload.extend_from_slice(&(self.user_agent).as_bytes());
        }
        payload.extend_from_slice(&(self.start_height).to_le_bytes());
        payload.extend_from_slice(&(self.relay as u8).to_le_bytes());

        let header = make_header("version".to_string(), &payload)?;

        msg.extend_from_slice(&header);
        msg.extend_from_slice(&payload);

        Ok(msg)
    }
}
