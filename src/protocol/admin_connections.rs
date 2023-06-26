use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::mpsc::Sender,
    sync::{Arc, Mutex},
};

use crate::{
    errores::NodoBitcoinError,
    log::{log_error_message, log_info_message, LogMessages},
};

#[derive(Clone)]
/// Representa una conexión a un nodo de la red y si esa conexión está siendo usada o no
///
/// # Campos
/// * id: identificador
/// * tcp: conexión a un nodo
/// * free: indica si la conexión está siendo usada o no
/// * logger: sender al logger
pub struct Connection {
    pub id: i32,
    pub tcp: Arc<Mutex<TcpStream>>,
    free: bool,
    logger: Option<Sender<LogMessages>>,
}
impl Connection {
    /// Escribe el mensaje recibido en la conexión
    pub fn write_message(&self, message: &[u8]) -> Result<(), NodoBitcoinError> {
        let connection = self.tcp.lock();
        match connection {
            Ok(mut connection) => match connection.write(message) {
                Ok(_) => Ok(()),
                Err(error) => {
                    self.log_error_msg(format!{"No se pudo escribir el mensaje en la connection {}: {}.", self.id, error});
                    Err(NodoBitcoinError::NoSePuedeEscribirLosBytes)
                }
            },
            Err(_) => {
                println!("No se pudo lockear el TcpStream");
                Err(NodoBitcoinError::NoSePuedeEscribirLosBytes)
            }
        }
    }

    /// Lee un mensaje de la conexión en el buffer recibido
    /// Si el error es de tipo WouldBlock, no se considera error ya que es debido al timeout seteado para el read
    /// y se debe seguir el ciclo de lectura
    /// Cualquier otro error indica que la conexión se cayó
    pub fn read_message(&self, buf: &mut [u8]) -> Result<Option<usize>, NodoBitcoinError> {
        let connection = self.tcp.lock();
        match connection {
            Ok(mut connection) => match connection.read(buf) {
                Ok(bytes_read) => Ok(Some(bytes_read)),
                Err(error) => {
                    if error.kind() == ErrorKind::WouldBlock {
                        //println!("No se pudo leer el mensaje");
                        Ok(None)
                    } else {
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
    fn _log_info_msg(&self, log_msg: String) {
        match &self.logger {
            Some(log) => {
                log_info_message(log.clone(), format!("connection:: {}", log_msg));
            }
            None => {}
        }
    }
    fn log_error_msg(&self, log_msg: String) {
        match &self.logger {
            Some(log) => {
                log_error_message(log.clone(), format!("connection:: {}", log_msg));
            }
            None => {}
        }
    }
}

#[derive(Clone)]
/// Administrador de conexiones, tiene todas las conexiones a los nodos de la red
/// Es quien se encarga de dar conexiones libres a quien lo solicite para evitar que se crucen los mensajes
///
/// # Campos
/// * connections: diccionario donde la key es el id de la conexión y el value es un Connection
/// * connectios_for_send_tx: diccionario igual al descripto anteriormente que será utilizado
///     exclusivamente para enviar nuevas transacciones
/// * logger: sender al logger
pub struct AdminConnections {
    connections: HashMap<i32, Connection>,
    connections_for_send_tx: HashMap<i32, Connection>,
    logger: Option<Sender<LogMessages>>,
}

impl Default for AdminConnections {
    fn default() -> Self {
        Self::new(None)
    }
}

impl AdminConnections {
    /// Crea un administrador de conexiones
    pub fn new(logger: Option<Sender<LogMessages>>) -> AdminConnections {
        AdminConnections {
            connections: HashMap::new(),
            connections_for_send_tx: HashMap::new(),
            logger,
        }
    }

    /// Crea un Connection a partir del TcpStream recibido y lo guarda en el administrador
    pub fn add(&mut self, tcp: TcpStream, id: i32) -> Result<(), NodoBitcoinError> {
        let _ = &(self.connections).insert(
            id,
            Connection {
                id,
                tcp: Arc::new(Mutex::new(tcp)),
                free: true,
                logger: self.logger.clone(),
            },
        );
        Ok(())
    }

    /// Crea un Connection a partir del TcpStream recibido y lo guarda en el administrador
    pub fn add_connection_for_send_tx(
        &mut self,
        tcp: TcpStream,
        id: i32,
    ) -> Result<(), NodoBitcoinError> {
        let _ = &(self.connections_for_send_tx).insert(
            id,
            Connection {
                id,
                tcp: Arc::new(Mutex::new(tcp)),
                free: true,
                logger: self.logger.clone(),
            },
        );
        Ok(())
    }

    /// Devuelve las conexiones en un vector
    pub fn get_connections_for_send_tx(&mut self) -> Vec<Connection> {
        let values = self.connections.values().cloned().collect();
        values
    }

    /// Devuelve las conexiones en un vector
    pub fn get_connections(&mut self) -> Vec<Connection> {
        let values: Vec<_> = self.connections.values().cloned().collect();
        let ten_values = values.iter().take(10).cloned().collect();
        ten_values
    }

    /// Encuentra una conexión que no esté ocupada (free = true)
    /// En caso de que se encuentre una, se pone como ocupada y se devuelve esa conexión y su id
    /// Caso contrario, devuelve un error
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

    /// Busca una conexión libre y se pone como libre la conexión correspondiente al id recibido
    /// Se devuelve la conexión libre en caso de ser encontrada.
    pub fn change_connection(
        &mut self,
        old_connection_id: i32,
    ) -> Result<(Connection, i32), NodoBitcoinError> {
        let free_connection = self.find_free_connection();
        match self.connections.get_mut(&old_connection_id) {
            Some(mut res) => res.free = false,
            None => self.log_error_msg(format!(
                "No se encontro la conexion {:?}",
                old_connection_id
            )),
        };
        self.log_error_msg("Cambio de conexion".to_string());
        free_connection
    }

    /// Libera la conexión correspondiente al id recibido
    pub fn free_connection(&mut self, connection_id: i32) -> Result<(), NodoBitcoinError> {
        match self.connections.get_mut(&connection_id) {
            Some(mut res) => res.free = false,
            None => self.log_error_msg("No se encontro la conexion".to_string()),
        };
        Ok(())
    }

    fn log_error_msg(&self, log_msg: String) {
        match &self.logger {
            Some(log) => {
                log_error_message(log.clone(), format!("admin_connection::{}", log_msg));
            }
            None => {}
        }
    }
}
