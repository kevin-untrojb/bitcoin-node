use std::net::TcpStream;

use crate::errores::NodoBitcoinError;
pub struct Connection{
    id: i32,
    pub tcp: TcpStream,
    free: bool
}

pub struct AdminConnections{
    connections: Vec<Connection>
}

impl AdminConnections {
    pub fn new() -> AdminConnections{
        AdminConnections { connections: vec![] }
    }

    pub fn add(&mut self, tcp: TcpStream, id: i32) -> Result<(), NodoBitcoinError> {
        let _ = &(self.connections).push(Connection { id, tcp, free: true});
        Ok(())
    }

    pub fn find_free_connection(&mut self) -> Result<&Connection, NodoBitcoinError>{
        match &(self.connections).into_iter().find(| connection | connection.free == true){
            Some(connection) => Ok(connection),
            None => return Err(NodoBitcoinError::NoSeEncuentraConexionLibre),
        }
    }

    pub fn change_connection(&mut self, old_connection: &Connection) -> Result<&Connection, NodoBitcoinError>{
        match &(self.connections).into_iter().find(| connection | connection.id == old_connection.id){
            Some(connection) => {
                connection.free = true;
            },
            None => return Err(NodoBitcoinError::NoSeEncuentraConexionLibre),
        }
        Ok(self.find_free_connection()?)
    }
}