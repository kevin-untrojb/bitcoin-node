use super::{blockheader::BlockHeader, transaction};
use transaction::Transaction;
use transaction::TxIn;
use transaction::TxOut;
use transaction::Outpoint;
use crate::errores::NodoBitcoinError;

/// A struct representing a Bitcoin Serialized Block
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/block_chain.html#serialized-blocks
///
/// # Fields
///
/// * `header` - The header of the block, which contains metadata such as the block's version, hash, and timestamp.
/// * `txns` - The transactions included in the block, represented as a vector of `Transaction` structs.
#[derive(Clone)]
pub struct SerializedBlock {
    pub header: BlockHeader,
    pub txns: Vec<Transaction>,
}

impl SerializedBlock {
    pub fn deserialize(block_bytes: &[u8]) -> Result<SerializedBlock,NodoBitcoinError> {
        let mut offset = 0;
        let header = BlockHeader::deserialize(&block_bytes[offset..offset + 80])?;
        offset += 80;
        
        let txn_count = u32::from_le_bytes(block_bytes[offset..offset + 4].try_into().map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?);
        offset += 4;
        
        let mut txns = Vec::new();
        for _ in 0..txn_count {
            let trn = Transaction::deserialize(&block_bytes[offset..])?;
            offset += trn.size();
            txns.push(trn);
        }
        Ok(SerializedBlock{
            header,
            txns
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let block_bytes:Vec<u8> = vec![
            // header
            1, 0, 0, 0,//version
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, // previous block
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50,//merkle root
            51, 52, 53, 54, 55, 56, 57, 48, 49, 50, //merkle root
            21, 205, 91, 7, //time /
            21, 205, 91, 7,// n bites
            21, 205, 91, 7, //nonce
            // cantidad transactions
            1,0,0,0,
            // transaction
            1, 0, 0, 0,  // version
            1, 0, 0, 0,  // number_tx_in
            // Datos de input
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // hash
            123, 0, 0, 0,
            4, 0, 0, 0,
            128, 0, 0, 0,
            255, 0, 0, 0,
            // Datos de número de output y output
            1, 0, 0, 0,  // number_tx_out
            // Datos de output
            123, 0, 0, 0, 0, 0, 0, 0, // Valor
            5, 0, 0, 0, //  pk_len
            1, 2, 3, 4, 5, // pk_script
            // Datos de lock_time
            0, 0, 0, 0, 0, 0, 0, 0,  // lock_time
        ];

        let header = BlockHeader {
            version: 1,
            previous_block_hash: [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ],
            merkle_root_hash: [
                49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49,
                50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50
            ],
            time: 123456789,
            n_bits: 123456789,
            nonce: 123456789,
        };

        let version = 1;
        let input = vec![
            TxIn {
                previous_output: Outpoint {
                    hash: [1u8; 32],
                    index: 123,
                },
                script_bytes:4,
                signature_script: vec![128, 0, 0, 0],
                sequence:255,
            }
        ];
        let output = vec![
            TxOut {
                value: 123,
                pk_len:5,
                pk_script: vec![1, 2, 3, 4, 5],
            }
        ];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
        };

        let result = SerializedBlock::deserialize(&block_bytes);

        assert!(result.is_ok());

        let serialized_block = result.unwrap();

        assert_eq!(serialized_block.header,header );
        assert_eq!(serialized_block.txns.len(),1);
        assert_eq!(serialized_block.txns[0],transaction);
    }
}