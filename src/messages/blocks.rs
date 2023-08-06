use crate::errores::NodoBitcoinError;

use super::messages_header::make_header;

/// Crea un mensaje block que contenga el bloque deserealizado
pub fn make_block(payload: &[u8]) -> Result<Vec<u8>, NodoBitcoinError> {
    let mut msg = Vec::new();
    let header = make_header("block".to_string(), &payload.to_vec())?;
    msg.extend_from_slice(&header);
    msg.extend_from_slice(payload);
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::block::SerializedBlock,
        config,
        messages::{blocks::make_block, messages_header::check_header},
    };

    fn init_config() {
        let args: Vec<String> = vec!["app_name".to_string(), "src/nodo.conf".to_string()];
        _ = config::inicializar(args);
    }

    #[test]
    fn test_make_block() {
        init_config();
        let block = SerializedBlock::read_last_block_from_file();
        assert!(block.is_ok());

        let block = block.unwrap();
        let block_bytes = block.serialize();
        assert!(block_bytes.is_ok());

        let block_bytes = block_bytes.unwrap();
        let block_msg = make_block(&block_bytes);
        assert!(block_msg.is_ok());

        let block_msg = block_msg.unwrap();

        let check_header = check_header(&block_msg);
        assert!(check_header.is_ok());

        let (command, response_get_data) = check_header.unwrap();
        assert!(command == "block");
        assert_eq!(response_get_data, block_bytes.len());

        let block_message_serialized = &block_msg[24..];

        let resultado = SerializedBlock::deserialize(&block_message_serialized);
        assert!(resultado.is_ok());

        let deserealized = resultado.unwrap();
        assert_eq!(deserealized.header.hash(), block.header.hash());
    }
}
