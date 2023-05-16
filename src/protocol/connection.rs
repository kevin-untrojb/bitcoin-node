use crate::config;
use crate::errores::NodoBitcoinError;
use crate::messages::messages_header::check_header;
use crate::messages::messages_header::make_header;
use crate::messages::version::VersionMessage;
use chrono::Utc;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

pub fn connect() -> Result<Vec<TcpStream>, NodoBitcoinError> {
    let addresses = get_address();
    let mut connections = Vec::new();

    for address in addresses.iter() {
        println!("Address: {:?}", address);
        match TcpStream::connect_timeout(address, Duration::from_secs(10)) {
            Ok(socket) => {
                let connection: TcpStream = handshake(socket, *address)?;
                connections.push(connection);
            },
            Err(_) => continue,
        };
    }
    Ok(connections)
}

fn handshake(mut socket: TcpStream, address: SocketAddr) -> Result<TcpStream, NodoBitcoinError> {
    let timestamp = Utc::now().timestamp() as u64;

    let version = VersionMessage::new(
        70015,
        0,
        timestamp,
        0,
        address.ip().to_string(),
        address.port(),
        0,
        "181.165.131.147".to_string(),
        18333,
        0,
        0,
        "".to_string(),
        0,
        true,
    );
    let mensaje = version.serialize()?;
    if socket.write_all(&mensaje).is_err() {
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }

    let mut header = [0u8; 24];
    if socket.read_exact(&mut header).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }

    let (command, payload_len) = check_header(&header)?;

    if command != "version" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let mut payload = vec![0u8; payload_len];
    if socket.read_exact(&mut payload).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }

    let mut verack_resp = vec![0u8; 24];
    if socket.read_exact(&mut verack_resp).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
    }

    let (command, _payload_len) = check_header(&verack_resp)?;

    if command != "verack" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let verack_msg = make_header("verack".to_string(), &Vec::new())?;
    if socket.write_all(&verack_msg).is_err() {
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }

    Ok(socket)
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

fn _get_local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok()?.ip().into()
}
