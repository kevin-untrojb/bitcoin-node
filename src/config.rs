use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    sync::{Mutex, MutexGuard},
};

use crate::{errores::NodoBitcoinError, parse_args};

/// Representa el item de configuración
struct ConfigItem {
    key: String,
    value: String,
}

const COMMENT_CHAR: char = '#';
const KEY_VALUE_SEPARATOR: char = '=';

static HASHMAP_CONFIG: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// Brinda acceso multithread al vector de items de configuración
///
/// # Errores
///
/// Si otro usuario de este mutex entró en panic mientras mantenía lockeado el mutex, entonces
/// esta llamada devolverá un error una vez que se obtenga el mutex.
fn access_hashmap() -> Result<MutexGuard<'static, Option<HashMap<String, String>>>, NodoBitcoinError>
{
    if let Ok(retorno) = HASHMAP_CONFIG.lock() {
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
        return from_reader(file);
    }
    Err(NodoBitcoinError::NoExisteArchivo)
}

/// Parsea el archivo de configuración
/// Recibe un reader del archivo de configuración
fn from_reader<T: Read>(file: T) -> Result<(), NodoBitcoinError> {
    let buf = BufReader::new(file);
    let lineas = parsear_archivo(buf);
    let items = parsear_lineas(lineas);
    if let Ok(mut config_hashmap) = access_hashmap() {
        *config_hashmap = Some(crear_hashmap(items));
    };
    Ok(())
}

/// Crea un hashmap a partir de los items de configuración
/// Recibe un vector de items de configuración
/// Devuelve un hashmap con los items de configuración
///    donde la clave es el key del item
///   y el valor es el value del item
/// Si hay items con la misma clave, se queda con la última aparición
fn crear_hashmap(items: Vec<ConfigItem>) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for item in items {
        map.insert(item.key, item.value);
    }
    map
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
                    key: key.to_string(),
                    value: value.to_string(),
                };
                items.push(item);
            }
        }
    }
    items
}

fn parsear_archivo<T: Read>(buf: BufReader<T>) -> Vec<String> {
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
    let config = access_hashmap()?;
    if let Some(hashmap_config) = config.as_ref() {
        if let Some(valor) = hashmap_config.get(&key) {
            return Ok(valor.clone());
        }
    }
    Err(NodoBitcoinError::NoExisteClave)
}

pub fn inicializar(args: Vec<String>) -> Result<(), NodoBitcoinError> {
    let filename = parse_args::parse_args(args)?;
    init_config(filename)?;
    Ok(())
}

#[test]
fn test_all() {
    test_archivo_config();
    test_archivo_inexistente();
    test_config_con_valores_comentados();
    test_config_con_valores_validos();
}

#[test]
fn test_archivo_config() {
    let files_folder = "src/test_files/".to_string();

    let filename = format!(
        "{}{}",
        files_folder,
        "config_file_con_todos_los_formatos.conf".to_string()
    );
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
fn test_config_con_valores_validos() {
    let contenido = "URL=www.github.com\n\
                        NOMBRE_GRUPO=Rustybandidos Test\n"
        .as_bytes();

    let leer_config = from_reader(contenido);
    assert!(leer_config.is_ok());

    let valor_url_result = get_valor("URL".to_string());
    assert!(valor_url_result.is_ok());

    let valor_url = valor_url_result.unwrap();
    assert_eq!(valor_url, "www.github.com");

    let valor_nombre_grupo_result = get_valor("NOMBRE_GRUPO".to_string());
    assert!(valor_nombre_grupo_result.is_ok());

    let valor_nombre_grupo = valor_nombre_grupo_result.unwrap();
    assert_eq!(valor_nombre_grupo, "Rustybandidos Test");
}

#[test]
fn test_config_con_valores_comentados() {
    let contenido = "#VALOR_COMENTADO=Este valor no se ve\n\
                        NOMBRE_GRUPO=Rustybandidos\n\
                        VALOR_NO_COMENTADO=Valor visible\n"
        .as_bytes();

    let leer_config = from_reader(contenido);
    assert!(leer_config.is_ok());

    let valor_comentado_result = get_valor("VALOR_COMENTADO".to_string());
    assert!(valor_comentado_result.is_err());
    assert_eq!(valor_comentado_result, Err(NodoBitcoinError::NoExisteClave));

    let valor_visible_result = get_valor("VALOR_NO_COMENTADO".to_string());
    assert!(valor_visible_result.is_ok());

    let valor_visible = valor_visible_result.unwrap();
    assert_eq!(valor_visible, "Valor visible");

    let valor_nombre_grupo_result = get_valor("NOMBRE_GRUPO".to_string());
    assert!(valor_nombre_grupo_result.is_ok());

    let valor_nombre_grupo = valor_nombre_grupo_result.unwrap();
    assert_eq!(valor_nombre_grupo, "Rustybandidos");
}

#[test]
fn test_archivo_inexistente() {
    let files_folder = "src/test_files/".to_string();

    let filename = format!("{}{}", files_folder, "no_existe.conf".to_string());
    let config_result = init_config(filename);
    assert!(config_result.is_err());
    assert_eq!(config_result, Err(NodoBitcoinError::NoExisteArchivo));
}
