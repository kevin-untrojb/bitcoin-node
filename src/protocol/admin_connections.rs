use std::{net::TcpStream, collections::HashMap, sync::{Mutex, Arc}};

use crate::errores::NodoBitcoinError;
#[derive(Clone)]
pub struct Connection{
    pub id: i32,
    pub tcp: Arc<Mutex<TcpStream>>,
    free: bool
}
#[derive(Clone)]
pub struct AdminConnections{
    connections: HashMap<i32, Connection>
}

impl AdminConnections {
    pub fn new() -> AdminConnections{
        AdminConnections { connections: HashMap::new() }
    }

    pub fn add(&mut self, tcp: TcpStream, id: i32) -> Result<(), NodoBitcoinError> {
        let _ = &(self.connections).insert(id, Connection {id, tcp: Arc::new(Mutex::new(tcp)), free: true});
        Ok(())
    }

    pub fn find_free_connection(&mut self) -> Result<(Connection, i32), NodoBitcoinError>{
        match self.connections.iter_mut().find(| (id, connection) | connection.free == true){
            Some((id, mut connection)) => {
                connection.free = false;
                Ok((connection.clone(), *id))
            },
            None => return Err(NodoBitcoinError::NoSeEncuentraConexionLibre),
        }
    }

    pub fn change_connection(&mut self, old_connection_id: i32) -> Result<(Connection, i32), NodoBitcoinError>{
        let free_connection = self.find_free_connection();
        match self.connections.get_mut(&old_connection_id) {
            Some(mut res) => (*res).free = false,
            None => todo!(),
        };
        free_connection
    }
}