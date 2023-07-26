use std::{
    io::{stdin, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
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

pub fn init_client(mensaje: String) -> Result<(), NodoBitcoinError> {
    let port = match config::get_valor("PORT".to_owned()) {
        Ok(res) => res,
        Err(_) => "18333".to_owned(),
    };

    let address = "127.0.0.1:".to_owned() + &port;
    let mut mensaje_a_enviar = mensaje.as_bytes();
    client_run(&address, &mut mensaje_a_enviar).unwrap();
    Ok(())
}

fn server_run(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    // accept devuelve una tupla (TcpStream, std::net::SocketAddr)
    println!("Escuchando en: {:?}", listener.local_addr()?);
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

/// Client run recibe una dirección y cualquier cosa "legible"
/// Esto nos da la libertad de pasarle stdin, un archivo, incluso otro socket
fn client_run(address: &str, stream: &mut dyn Read) -> std::io::Result<()> {
    // Vamos a usar un BufReader para comodidad de leer lineas
    // Notar que como el stream es de tipo `Read`, podemos leer de a bytes.
    // BufReader nos provee una capa de abstracción extra para manejarnos con strings
    let reader = BufReader::new(stream);
    // Intentamos conectar el socket a un puerto abierto
    let mut socket = TcpStream::connect(address)?;
    // BufReader nos permite leer lineas de texto
    for line in reader.lines() {
        // lines nos devuelve un iterador de Result(string), agarramos el string adentro
        if let Ok(line) = line {
            println!("Enviando: {:?}", line);
            // TcpStream implementa Write
            socket.write(line.as_bytes())?;
            // El reader le quita el salto de linea, así que se lo mando aparte
            socket.write("\n".as_bytes())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_server() {
        init_listener();
    }

    #[test]
    fn test_run_client() {
        init_client("Hola, espero que esto llegue".to_string());
    }
}
