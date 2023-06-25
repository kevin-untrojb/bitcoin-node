use std::{
    collections::HashMap,
    sync::mpsc::{self, channel, Sender},
    io::{Read, Write, ErrorKind},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::{
    errores::NodoBitcoinError,
    log::{log_error_message, LogMessages},
};

#[derive(Clone)]
pub struct Connection {
    pub id: i32,
    pub tcp: Arc<Mutex<TcpStream>>,
    free: bool,
    logger: Option<Sender<LogMessages>>
}
impl Connection {
    pub fn write_message(&self, message: &[u8]) -> Result<(), NodoBitcoinError> {
        let connection = self.tcp.lock();
        match connection {
            Ok(mut connection) => {
                let _ = match connection.write(message) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(error) =>{
                        self.log_error_msg(format!{"no se pudo escribir el mensaje en la connection {}: {}.", self.id, error});
                        return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                    }
                };
            }
            Err(_) => {
                println!("No se pudo lockear el TcpStream");
                Err(NodoBitcoinError::NoSePuedeEscribirLosBytes)
            }
        }
    }

    pub fn read_message(&self, buf: &mut [u8]) -> Result<Option<usize>, NodoBitcoinError> {
        let connection = self.tcp.lock();
        match connection {
            Ok(mut connection) => match connection.read(buf) {
                Ok(bytes_read) => Ok(Some(bytes_read)),
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        println!("No se pudo leer el mensaje");
                        Ok(None)
                    }else{
                        self.log_error_msg(format!{"no se pudo leer el mensaje en la connection {}: {}.", self.id, error});
                        Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
                    }
                }
            },
            Err(_) => {
                println!("No se pudo lockear el TcpStream");
                Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
            }
        }
    }
    pub fn read_exact_message(&self, buf: &mut [u8]) -> Result<(), NodoBitcoinError> {
        let connection = self.tcp.lock();
        match connection {
            Ok(mut connection) => match connection.read_exact(buf) {
                Ok(()) => Ok(()),
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        Ok(())
                    } else {
                        println!("No se pudo leer exact message");
                        self.log_error_msg(format!{"no se pudo leer exact mensaje en la connection {}: {}.", self.id, error});
                        Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
                    }
                }
            },
            Err(_) => {
                println!("No se pudo lockear el TcpStream");
                Err(NodoBitcoinError::NoSePuedeLeerLosBytes)
            }
        }
    }
    fn log_error_msg(&self,log_msg: String){
        match &self.logger {
            Some(log) =>{
                log_error_message(
                    log.clone(),format!("connection:: {}",log_msg),
                );
            }
            None=> {}
        }
    }
}

#[derive(Clone)]
pub struct AdminConnections {
    connections: HashMap<i32, Connection>,
    logger: Option<Sender<LogMessages>>
}

impl Default for AdminConnections {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AdminConnections {
    pub fn new(logger: Option<Sender<LogMessages>>) -> AdminConnections {
        AdminConnections {
            connections: HashMap::new(),
            logger
        }
    }

    pub fn add(&mut self, tcp: TcpStream, id: i32) -> Result<(), NodoBitcoinError> {
        let _ = &(self.connections).insert(
            id,
            Connection {
                id,
                tcp: Arc::new(Mutex::new(tcp)),
                free: true,
                logger: self.logger.clone()
            },
        );
        Ok(())
    }

    pub fn get_connections(&mut self) -> Vec<Connection> {
        let values = self.connections.values().cloned().collect();
        values
    }

    pub fn find_free_connection(&mut self) -> Result<(Connection, i32), NodoBitcoinError> {
        match self
            .connections
            .iter_mut()
            .find(|(_id, connection)| connection.free)
        {
            Some((id, mut connection)) => {
                connection.free = false;
                Ok((connection.clone(), *id))
            }
            None => Err(NodoBitcoinError::NoSeEncuentraConexionLibre),
        }
    }

    pub fn change_connection(
        &mut self,
        old_connection_id: i32,
    ) -> Result<(Connection, i32), NodoBitcoinError> {
        let free_connection = self.find_free_connection();
        match self.connections.get_mut(&old_connection_id) {
            Some(mut res) => res.free = false,
            None => println!("No se encontro la conexion"),
        };
        println!("Cambio de conexion");
        free_connection
    }

    pub fn free_connection(&mut self, connection_id: i32) -> Result<(), NodoBitcoinError> {
        match self.connections.get_mut(&connection_id) {
            Some(mut res) => res.free = false,
            None => println!("No se encontro la conexion"),
        };
        Ok(())
    }

    fn log_error_msg(&self,log_msg: String){
        match &self.logger {
            Some(log) =>{
                log_error_message(
                    log.clone(),format!("admin_connection::{}",log_msg),
                );
            }
            None=> {}
        }
    }
}
