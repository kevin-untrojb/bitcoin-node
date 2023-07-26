use std::{
    io::{BufRead, BufReader, Read},
    net::TcpListener,
};

use crate::{config, errores::NodoBitcoinError};

pub fn init_listener() -> Result<(), NodoBitcoinError> {
    let port = match config::get_valor("PORT".to_owned()) {
        Ok(res) => res,
        Err(_) => "18333".to_owned(),
    };

    let address = "127.0.0.1:".to_owned() + &port;
    server_run(&address).unwrap();
    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    // accept devuelve una tupla (TcpStream, std::net::SocketAddr)
    let (mut client_stream, socket_addr) = listener.accept()?;
    println!("La socket addr del client: {:?}", socket_addr);
    // let mut client_stream : TcpStream = connection.0;
    // TcpStream implementa el trait Read, así que podemos trabajar como si fuera un archivo
    handle_client(&mut client_stream)?;
    Ok(())
}

//
fn handle_client(stream: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    while let Some(Ok(line)) = lines.next() {
        println!("Recibido: {:?}", line);
    }
    Ok(())
}
