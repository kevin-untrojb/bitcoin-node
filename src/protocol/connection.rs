use super::admin_connections::AdminConnections;
use crate::config;
use crate::errores::NodoBitcoinError;
use crate::log::{log_error_message, log_info_message, LogMessages};
use crate::messages::messages_header::check_header;
use crate::messages::messages_header::make_header;
use crate::messages::version::VersionMessage;
use chrono::Utc;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::mpsc::Sender;
use std::time::Duration;

/// Recorre lista de direccions e intenta conectarse a cada una de ellas
/// Si la conexión se realizó con éxito, se guarda esa conexión en el administrador de conexiones
pub fn connect(logger: Sender<LogMessages>) -> Result<AdminConnections, NodoBitcoinError> {
    let mut admin_connections = AdminConnections::new(Some(logger.clone()));
    _ = add_connections(&mut admin_connections, logger, false);
    Ok(admin_connections)
}

/// Recorre lista de direccions e intenta conectarse a cada una de ellas
/// Si la conexión se realizó con éxito, se guarda esa conexión en el administrador de conexiones
fn add_connections(
    admin_connection: &mut AdminConnections,
    logger: Sender<LogMessages>,
    add_for_send_tx: bool,
) -> Result<(), NodoBitcoinError> {
    //let mut admin_connections = AdminConnections::new(Some(logger.clone()));
    let addresses = get_address();
    let mut id: i32 = 0;
    for address in addresses.iter() {
        match TcpStream::connect_timeout(address, Duration::from_secs(10)) {
            Ok(socket) => {
                match handshake(socket, *address) {
                    Ok(connection) => {
                        log_info_message(
                            logger.clone(),
                            format!("Conexión establecida: {:?}", address),
                        );
                        if add_for_send_tx {
                            admin_connection.add_connection_for_send_tx(connection, id)?;
                        } else {
                            let duration = connection.set_read_timeout(Some(Duration::new(10, 0)));
                            if duration.is_err() {
                                log_error_message(
                                    logger.clone(),
                                    "Error al setear read timeout.".to_string(),
                                );
                            }
                            admin_connection.add(connection, id)?;
                        }
                        id += 1;
                    }
                    Err(_) => continue,
                };
            }
            Err(_) => continue,
        };
    }

    Ok(())
}

/// Se realiza el handshake con una conexión
/// Si se envían con éxito los mensajes version y verack y también se reciben los mismos con éxito
/// se considera que la conexión ha sido establecida con éxito.
///
/// También se envía un mensaje sendHeaders para establecer de qué forma se quiere recibir los bloques nuevos
fn handshake(mut socket: TcpStream, address: SocketAddr) -> Result<TcpStream, NodoBitcoinError> {
    let timestamp = Utc::now().timestamp() as u64;
    let version = match (config::get_valor("VERSION".to_string())?).parse::<u32>() {
        Ok(res) => res,
        Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerValorDeArchivoConfig),
    };

    let version_message = VersionMessage::new(version, timestamp, address);
    let mensaje = version_message.serialize()?;
    if socket.write_all(&mensaje).is_err() {
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }

    let mut header = [0u8; 24];
    if socket.read_exact(&mut header).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytesHeaderVersionMessage);
    }

    let (command, payload_len) = check_header(&header)?;

    if command != "version" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let mut payload = vec![0u8; payload_len];
    if socket.read_exact(&mut payload).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytesVersionMessage);
    }

    let mut verack_resp = vec![0u8; 24];
    if socket.read_exact(&mut verack_resp).is_err() {
        return Err(NodoBitcoinError::NoSePuedeLeerLosBytesVerackMessage);
    }

    let (command, _payload_len) = check_header(&verack_resp)?;

    if command != "verack" {
        return Err(NodoBitcoinError::ErrorEnHandshake);
    }

    let verack_msg = make_header("verack".to_string(), &Vec::new())?;
    if socket.write_all(&verack_msg).is_err() {
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }

    let sendheaders_msg = make_header("sendheaders".to_string(), &Vec::new())?;
    if socket.write_all(&sendheaders_msg).is_err() {
        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
    }

    Ok(socket)
}

/// Obtiene las distintas direcciones de una semilla DNS
pub fn get_address() -> Vec<SocketAddr> {
    let mut seeds = Vec::new();
    let url = match config::get_valor("ADDRESS".to_owned()) {
        Ok(res) => res,
        Err(_) => return seeds,
    };
    let port = match config::get_valor("PORT".to_owned()) {
        Ok(res) => res,
        Err(_) => "18333".to_owned(),
    };

    let seedhost = format!("{}:{}", url, port);

    if let Ok(lookup) = seedhost.to_socket_addrs() {
        for host in lookup {
            seeds.push(host);
        }
    }
    seeds
}
