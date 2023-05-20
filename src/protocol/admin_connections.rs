use std::{net::TcpStream, collections::HashMap};

use crate::errores::NodoBitcoinError;

pub struct Connection{
    pub tcp: TcpStream,
    free: bool
}

pub struct AdminConnections{
    connections: HashMap<i32, Connection>
}

impl AdminConnections {
    pub fn new() -> AdminConnections{
        AdminConnections { connections: HashMap::new() }
    }

    pub fn add(&mut self, tcp: TcpStream, id: i32) -> Result<(), NodoBitcoinError> {
        let _ = &(self.connections).insert(id, Connection {tcp, free: true});
        Ok(())
    }

    pub fn find_free_connection(admin_connections: &AdminConnections) -> Result<(&mut Connection, i32), NodoBitcoinError>{
        match &(admin_connections.connections).into_iter().find(| (_id, connection) | connection.free == true){
            Some((id, mut connection)) => {
                connection.free = false;
                Ok((&mut connection, *id))
            },
            None => return Err(NodoBitcoinError::NoSeEncuentraConexionLibre),
        }
    }

    pub fn change_connection(admin_connections: &AdminConnections, old_connection_id: i32) -> Result<(&mut Connection, i32), NodoBitcoinError>{
        let free_connection = Self::find_free_connection(admin_connections);
        match (admin_connections.connections).get(&old_connection_id) {
            Some(res) => res.free = false,
            None => todo!(),
        };
        free_connection
    }
}