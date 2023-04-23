use crate::errores::NodoBitcoinError;

/// Parsea los argumentos del main
pub fn parse_args(args: Vec<String>) -> Result<String, NodoBitcoinError> {
    if args.len() < 2 {
        // Acá debería devolver el error pero por ahora le devolvemos un archivo hardcodeado para que sea más facil de probar
        // y no tener que estar cargando por parametro el archivo de conf
        //Err(NodoBitcoinError::NoArgument)
        Ok("src/nodo.conf".to_string())
    } else {
        Ok(args[1].to_string())
    }
}
