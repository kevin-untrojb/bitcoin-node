use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::{Mutex, MutexGuard},
};

use crate::errores::NodoBitcoinError;

/// Representa el item de configuración
struct ConfigItem {
    _key: String,
    _value: String,
}

const COMMENT_CHAR: char = '#';
const KEY_VALUE_SEPARATOR: char = '=';

static HASH_CONFIG: Mutex<Vec<ConfigItem>> = Mutex::new(vec![]);

/// Brinda acceso multithread al vector de items de configuración
///
/// # Errores
///
/// Si otro usuario de este mutex entró en panic mientras mantenía lockeado el mutex, entonces
/// esta llamada devolverá un error una vez que se obtenga el mutex.
fn access_config() -> Result<MutexGuard<'static, Vec<ConfigItem>>, NodoBitcoinError> {
    if let Ok(retorno) = HASH_CONFIG.lock() {
        return Ok(retorno);
    }
    Err(NodoBitcoinError::ConfigLock)
}

/// Inicializa el modulo de configuración
/// Recibe la ruta del archivo de configuración
///
/// # Errores
///
/// Si no puede leer el archivo
pub fn init_config(filename: String) -> Result<(), NodoBitcoinError> {
    if let Ok(file) = File::open(filename) {
        let buf = BufReader::new(file);
        let lineas = parsear_archivo(buf);
        let items = parsear_lineas(lineas);

        if let Ok(mut config) = access_config() {
            for item in items {
                config.push(item);
            }
        };
        return Ok(());
    }
    Err(NodoBitcoinError::NoExisteArchivo)
}

/// Parsea las lineas en ConfigItem
///
/// Ignora las lineas que no tengan el correspondiente KEY_VALUE_SEPARATOR
///     las que comiencen con un COMMENT_CHAR
fn parsear_lineas(lineas: Vec<String>) -> Vec<ConfigItem> {
    let mut items: Vec<ConfigItem> = vec![];
    for linea in lineas {
        if !linea.starts_with(COMMENT_CHAR) {
            if let Some((key, value)) = linea.split_once(KEY_VALUE_SEPARATOR) {
                let item = ConfigItem {
                    _key: key.to_string(),
                    _value: value.to_string(),
                };
                items.push(item);
            }
        }
    }
    items
}

fn parsear_archivo(buf: BufReader<File>) -> Vec<String> {
    let buf_lineas = buf.lines();
    let lineas: Vec<String> = buf_lineas
        .map(|l| {
            if let Ok(linea) = l {
                linea
            } else {
                "".to_string()
            }
        })
        .collect();
    lineas
}

/// Devuelve el valor correspondiente a la clave
///
/// # Errores
///
/// Devuelve error en caso que no exista la clave solicitada
pub fn get_valor(key: String) -> Result<String, NodoBitcoinError> {
    let config = access_config()?;
    for item in config.iter() {
        if item._key == key {
            let valor_clonado = item._value.clone();
            return Ok(valor_clonado);
        }
    }
    Err(NodoBitcoinError::NoExisteClave)
}

#[test]
fn test_archivo_config() {
    let files_folder = "src/test_files/".to_string();

    let filename = format!("{}{}", files_folder, "test_1.conf".to_string());
    let config_result = init_config(filename);
    assert!(config_result.is_ok());

    let valor_url_result = get_valor("URL".to_string());
    assert!(valor_url_result.is_ok());

    let valor_url = valor_url_result.unwrap();
    assert_eq!(valor_url, "www.github.com");

    let valor_nombre_grupo_result = get_valor("NOMBRE_GRUPO".to_string());
    assert!(valor_nombre_grupo_result.is_ok());

    let valor_nombre_grupo = valor_nombre_grupo_result.unwrap();
    assert_eq!(valor_nombre_grupo, "Rustybandidos Test");

    let valor_comentado_result = get_valor("COMENTADO".to_string());
    assert!(valor_comentado_result.is_err());
    assert_eq!(valor_comentado_result, Err(NodoBitcoinError::NoExisteClave));

    let valor_invalido_result = get_valor("FORMATO_INVALIDO".to_string());
    assert!(valor_invalido_result.is_err());
    assert_eq!(valor_invalido_result, Err(NodoBitcoinError::NoExisteClave));
}

#[test]
fn test_archivo_inexistente() {
    let files_folder = "src/test_files/".to_string();

    let filename = format!("{}{}", files_folder, "no_existe.conf".to_string());
    let config_result = init_config(filename);
    assert!(config_result.is_err());
    assert_eq!(config_result, Err(NodoBitcoinError::NoExisteArchivo));
}
