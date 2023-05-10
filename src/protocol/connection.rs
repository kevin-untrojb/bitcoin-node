use crate::config;
use crate::errores::NodoBitcoinError;
use crate::messages::version::VersionMessage;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use chrono::Utc;


pub fn connect() -> Result<(), NodoBitcoinError>{
    let addresses = get_address();

    for address in addresses.iter() {
        println!("Address: {:?}", address);

        let socket: TcpStream = match TcpStream::connect(address){
            Ok(socket) => socket,
            Err(_) => return Err(NodoBitcoinError::NoSePudoConectar)
        };

        handshake(socket, *address)?;
    }

    Ok(())
}

fn handshake(mut socket: TcpStream, address: SocketAddr) -> Result<(), NodoBitcoinError>{
    let timestamp = Utc::now().timestamp() as u64;

    let version = VersionMessage::new(70015, 0, timestamp, 0, address.ip().to_string(), address.port(), 0, "181.165.131.147".to_string(), 18333, 0, 0, "".to_string(), 0, true);
    let mensaje = version.serialize()?;
    socket.write_all(&mensaje).unwrap();

    println!("{} bytes sent version", mensaje.len());

    let mut num_buffer = [0u8; 1024];

    let lectura = socket.read(&mut num_buffer);

    println!("{} bytes read version", lectura.unwrap());

    Ok(())
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