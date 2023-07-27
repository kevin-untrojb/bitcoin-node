use std::{
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
    thread::{self, sleep},
    time::Duration,
};

use chrono::Utc;

use crate::{
    config,
    errores::NodoBitcoinError,
    log::{log_error_message, log_info_message, LogMessages},
    messages::{
        messages_header::{check_header, make_header},
        version::VersionMessage,
    },
};

pub fn init_server(logger: Sender<LogMessages>) -> Result<(), NodoBitcoinError> {
    let port = config::get_valor("PORT".to_owned())?;

    let address = "127.0.0.1:".to_owned() + &port;
    server_run(&address, logger).unwrap();
    Ok(())
}

fn server_run(address: &str, logger: Sender<LogMessages>) -> Result<(), NodoBitcoinError> {
    let listener = match TcpListener::bind(address) {
        Ok(res) => res,
        Err(_) => {
            log_error_message(logger.clone(), "Error al bindear el socket".to_string());
            return Err(NodoBitcoinError::NoSePudoConectar);
        }
    };

    if listener.local_addr().is_err() {
        log_error_message(
            logger.clone(),
            "Error al obtener la dirección local".to_string(),
        );
        return Err(NodoBitcoinError::NoSePudoConectar);
    }

    log_info_message(
        logger.clone(),
        format!("Escuchando en: {:?}", listener.local_addr().unwrap()),
    );

    loop {
        match listener.accept() {
            Ok((mut stream, socket_addr)) => {
                log_info_message(
                    logger.clone(),
                    format!("Conexión establecida: {:?}", socket_addr),
                );
                let logger_cloned = logger.clone();
                thread::spawn(move || {
                    handle_message(&mut stream, logger_cloned);
                });
            }
            Err(_) => {
                log_error_message(logger.clone(), "Error al aceptar la conexión".to_string());
                continue;
            }
        }
    }
    Ok(())
}

fn shakehand(stream: &mut TcpStream) -> Result<(), NodoBitcoinError> {
    let mut header = [0u8; 24];
    if stream.read_exact(&mut header).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let (command, payload_len) = check_header(&header)?;

    if command != "version" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let mut payload = vec![0u8; payload_len];
    if stream.read_exact(&mut payload).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    // chequear que la version sea como la nuestra para mandar el version nuestro, sino abortar
    // despues hago el deserealize del version para esto

    let timestamp = Utc::now().timestamp() as u64;
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        // sacamos del config la version??
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::ErrorEnHandshake),
    };

    let version_message = VersionMessage::new(version, timestamp, stream.peer_addr().unwrap()); //LIMPIAR EL UNWRAP
    let mensaje = version_message.serialize()?;
    if stream.write_all(&mensaje).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let mut verack_resp = vec![0u8; 24];
    if stream.read_exact(&mut verack_resp).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytesVerackMessage);
    }

    let (command, _payload_len) = check_header(&verack_resp)?;

    if command != "verack" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let verack_msg = make_header("verack".to_string(), &Vec::new())?;
    if stream.write_all(&verack_msg).is_err() {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    Ok(())
}

fn handle_message(stream: &mut TcpStream, logger: Sender<LogMessages>) {
    // handshake al revés
    // read version
    // write version
    // read verack
    // write verack

    // si es ok el handshake, se hace el loop de recibir mensajes
    // si no, se cierra el socket
    // loop
    // si el read sale por timeout, enviamos un ping, si no responde pong se cierra el socket
    // si el read no lee bytes, se cierra el socket
    // si el read lee bytes, se hace el parseo del mensaje

    // los mensajes que podemos recibir son:
    // - ping
    // - pong
    // - getheaders
    // - getdata

    // los mensajes que podemos enviar son:
    // - ping
    // - pong
    // - headers
    // - block

    match shakehand(stream) {
        Ok(()) => {
            // salio bien el handshake, ponerse a escuchar
            thread_connection(stream, logger.clone());
        }
        Err(_) => return,
    };

    // prueba inicial
    // loop {
    //     let mut buffer_read = [0 as u8; 100];
    //     let leidos = stream.read(&mut buffer_read).unwrap();
    //     if leidos == 0 {
    //         break;
    //     }
    //     println!("Recibido: {:?}", buffer_read);
    // }
}

fn thread_connection(stream: &mut TcpStream, logger: Sender<LogMessages>) {
    loop {
        let mut buffer = [0 as u8; 100];
        let len_bytes = match stream.read(&mut buffer) {
            Ok(res) => res,
            Err(error) => {
                if error.kind() == ErrorKind::WouldBlock {
                    // Error de Timeout, TODO: enviamos un ping
                    continue;
                } else {
                    log_error_message(
                        logger.clone(),
                        "Error al leer una solicitud del cliente".to_string(),
                    );
                    return;
                }
            }
        };
        if len_bytes == 0 {
            log_error_message(
                logger.clone(),
                "Se cierra la conexión al cliente porque se leyó 0 bytes".to_string(),
            );
            return;
        }
        let buffer = &buffer[..len_bytes];
        // verifico los tipos de mensajes aceptados
        let (command, _) = match check_header(&buffer) {
            Ok((command, payload_len)) => (command, payload_len),
            Err(_) => {
                log_error_message(
                    logger.clone(),
                    "Error al parsear el header del cliente".to_string(),
                );
                continue;
            }
        };
        log_info_message(logger.clone(), format!("Command recibido: {:?}", command));
        if command == "ping" {
            log_info_message(logger.clone(), "Ping recibido".to_string());
        }
        if command == "getheaders" {
            log_info_message(logger.clone(), "getheaders recibido".to_string());
        }
        if command == "getdata" {
            log_info_message(logger.clone(), "getdata recibido".to_string());
        }
    }
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

            sleep(Duration::from_millis(5000));
            socket.write(line.as_bytes())?;
            // El reader le quita el salto de linea, así que se lo mando aparte
            socket.write("\n".as_bytes())?;
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::log::create_logger_actor;

    use super::*;

    #[test]
    fn test_run_server() {
        let logger = create_logger_actor(config::get_valor("LOG_FILE".to_string()));
        init_server(logger);
    }

    #[test]
    fn test_run_client() {
        init_client("Hola, espero que esto llegue".to_string());
    }
}
