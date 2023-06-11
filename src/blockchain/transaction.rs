use crate::common::utils_bytes;
use crate::common::{base58::p2pkh_script_serialized, uint256::Uint256};
use crate::errores::NodoBitcoinError;
use bitcoin_hashes::{sha256d, Hash};
use std::{collections::HashMap, io::Write, vec};

use super::block::SerializedBlock;

/// A struct representing a Bitcoin transaction
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/transactions.html
///
/// # Fields
///
/// * version - The version number of the transaction.
/// * input - The vector of input transactions for the transaction.
/// * output - The vector of output transactions for the transaction.
/// * lock_time - The lock time for the transaction.
#[warn(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Transaction {
    pub version: u32,
    pub input: Vec<TxIn>,
    pub output: Vec<TxOut>,
    pub lock_time: u32,
    pub tx_in_count: usize,
    pub tx_out_count: usize,
}

impl Transaction {
    #[warn(dead_code)]
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.version).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let tx_in_count_prefix = utils_bytes::from_amount_bytes_to_prefix(self.tx_in_count);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(tx_in_count_prefix, self.input.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        for tx_in in &self.input {
            bytes
                .write_all(&tx_in.serialize()?)
                .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        }

        let tx_out_count_prefix = utils_bytes::from_amount_bytes_to_prefix(self.tx_out_count);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(tx_out_count_prefix, self.output.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        for tx_out in &self.output {
            bytes
                .write_all(&tx_out.serialize()?)
                .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        }
        bytes
            .write_all(&self.lock_time.to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }
    pub fn deserialize(block_bytes: &[u8]) -> Result<Transaction, NodoBitcoinError> {
        let mut offset = 0;
        let version = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 4;
        let (tx_in_count, tx_in_amount) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += tx_in_count;

        let mut input = Vec::new();
        for _v in 0..tx_in_amount {
            let tx_in = TxIn::deserialize(&block_bytes[offset..])?;
            offset += tx_in.size();
            input.push(tx_in);
        }

        let (tx_out_count, tx_out_amount) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += tx_out_count;

        let mut output = Vec::new();
        for _v in 0..tx_out_amount {
            let tx_out = TxOut::deserialize(&block_bytes[offset..])?;
            offset += tx_out.size();
            output.push(tx_out);
        }

        let lock_time = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        //offset += 4;
        Ok(Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count,
            tx_out_count,
        })
    }

    pub fn size(&self) -> usize {
        let input_size = self.input.iter().map(|tx_in| tx_in.size()).sum::<usize>();
        let output_size = self
            .output
            .iter()
            .map(|tx_out| tx_out.size())
            .sum::<usize>();
        8 + input_size + output_size + self.tx_in_count + self.tx_out_count
    }

    pub fn txid(&self) -> Result<Uint256, NodoBitcoinError> {
        let bytes = self.serialize()?;
        let hash = sha256d::Hash::hash(&bytes);
        let u256 = Uint256::_from_be_bytes(*hash.as_byte_array());
        Ok(u256)
    }

    pub fn _get_tx_from_file(txid: Uint256) -> Result<Transaction, NodoBitcoinError> {
        let blocks = SerializedBlock::read_blocks_from_file()?;
        let mut txs = HashMap::new();
        for block in blocks {
            for tx in block.txns {
                txs.insert(tx.txid()?, tx);
            }
        }
        let ret = txs.get(&txid);
        match ret {
            Some(tx) => Ok(tx.clone()),
            None => Err(NodoBitcoinError::NoExisteClave),
        }
    }

    pub fn new(
        version: u32,
        input: Vec<TxIn>,
        output: Vec<TxOut>,
        lock_time: u32,
    ) -> Result<Transaction, NodoBitcoinError> {
        let tx_in_count = input.len();
        let tx_out_count = output.len();
        Ok(Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count,
            tx_out_count,
        })
    }
}

/// A struct representing an input transaction for a Bitcoin transaction
///
/// # Fields
///
/// * previous_output - The outpoint from the previous transaction that this input is spending.
/// * script_bytes - The number of bytes in the signature script.
/// * signature_script - The signature script for the input.
/// * sequence - The sequence number for the input.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TxIn {
    pub previous_output: Outpoint,
    pub script_bytes: usize,
    pub signature_script: Vec<u8>,
    pub sequence: u32,
    pub script_bytes_amount: usize,
}

impl TxIn {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.previous_output._serialize()?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let script_bytes_prefix =
            utils_bytes::from_amount_bytes_to_prefix(self.script_bytes_amount);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(script_bytes_prefix, self.script_bytes)?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&self.signature_script)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.sequence).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<TxIn, NodoBitcoinError> {
        let mut offset = 0;

        let previous_output = Outpoint::deserialize(&block_bytes[offset..offset + 36])?;
        offset += 36;

        let (script_bytes_amount, script_bytes) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += script_bytes_amount;

        let mut signature_script = vec![0u8; script_bytes];
        signature_script.copy_from_slice(&block_bytes[offset..offset + script_bytes]);
        offset += script_bytes;

        let sequence = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        //offset += 4;

        Ok(TxIn {
            previous_output,
            script_bytes,
            signature_script,
            sequence,
            script_bytes_amount,
        })
    }
    pub fn size(&self) -> usize {
        40 + self.script_bytes_amount + self.signature_script.len()
    }
    pub fn new(hash: Uint256, index: usize) -> TxIn {
        let previous_output = Outpoint::new(hash, index);
        TxIn {
            previous_output,
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0xffffffff,
            script_bytes_amount: 0,
        }
    }
}

/// A struct representing an outpoint from a previous transaction
///
/// # Fields
///
/// * hash - The transaction hash of the previous transaction.
/// * index - The index of the output in the previous transaction.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Outpoint {
    pub hash: [u8; 32],
    pub index: u32,
}
impl Outpoint {
    pub fn _serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&self.hash)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&(self.index).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }

    pub fn deserialize(block_bytes: &[u8]) -> Result<Outpoint, NodoBitcoinError> {
        let mut offset = 0;

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&block_bytes[offset..offset + 32]);
        offset += 32;

        let index = u32::from_le_bytes(
            block_bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );

        Ok(Outpoint { hash, index })
    }

    pub fn new(hash: Uint256, index: usize) -> Outpoint {
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&hash.get_bytes());
        Outpoint {
            hash: hash_bytes,
            index: index as u32,
        }
    }
}

/// A struct representing an output transaction for a Bitcoin transaction
///
/// # Fields
///
/// * value - The value of the output in satoshis.
/// * pk_script - The public key script for the output.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TxOut {
    pub value: u64,
    pub pk_len: usize,
    pub pk_script: Vec<u8>,
    pub pk_len_bytes: usize,
}

impl TxOut {
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.value).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        let n_bytes_prefix = utils_bytes::from_amount_bytes_to_prefix(self.pk_len_bytes);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(n_bytes_prefix, self.pk_script.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        bytes
            .write_all(&self.pk_script)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(bytes)
    }
    pub fn deserialize(block_bytes: &[u8]) -> Result<TxOut, NodoBitcoinError> {
        let mut offset = 0;

        let value = u64::from_le_bytes(
            block_bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 8;
        let (pk_len_bytes, pk_len) = utils_bytes::parse_varint(&block_bytes[offset..]);
        offset += pk_len_bytes;

        let mut pk_script = vec![0u8; pk_len];
        pk_script.copy_from_slice(&block_bytes[offset..offset + pk_len]);
        Ok(TxOut {
            value,
            pk_len,
            pk_script,
            pk_len_bytes,
        })
    }

    pub fn size(&self) -> usize {
        8 + self.pk_len_bytes + self.pk_script.len()
    }

    pub fn new(amount: usize, script: Vec<u8>) -> Result<TxOut, NodoBitcoinError> {
        let p2pkh_script = p2pkh_script_serialized(&script)?;
        let pk_len = p2pkh_script.len();
        let pk_len_bytes = utils_bytes::from_amount_bytes_to_prefix(pk_len);
        Ok(TxOut {
            value: amount as u64,
            pk_len,
            pk_script: p2pkh_script,
            pk_len_bytes: pk_len_bytes.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::common::base58::decode_base58;

    use super::*;

    #[test]
    fn test_serialize_transaction() {
        let version = 1;
        let input = vec![TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 4294967295,
            },
            script_bytes: 31,
            script_bytes_amount: 1,
            signature_script: vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
            ],
            sequence: 4294967295,
        }];
        let output = vec![TxOut {
            value: 5000000000,
            pk_len: 35,
            pk_len_bytes: 1,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        }];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count: 1,
            tx_out_count: 1,
        };

        let expected_bytes = vec![
            //
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, // trout pk
            172, 0, 0, 0, 0,
        ]; // lock time

        let serialized = transaction.serialize().unwrap();

        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_transaction() {
        let block_bytes: Vec<u8> = vec![
            //
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
            172, // trout pk
            0, 0, 0, 0,
        ]; // lock time

        let transaction = Transaction::deserialize(&block_bytes).unwrap();

        assert_eq!(transaction.version, 1);
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(
            transaction.input[0],
            TxIn {
                previous_output: Outpoint {
                    hash: [0; 32],
                    index: 4294967295,
                },
                script_bytes: 31,
                script_bytes_amount: 1,
                signature_script: vec![
                    4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114,
                    111, 110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47
                ],
                sequence: 4294967295,
            }
        );
        assert_eq!(transaction.output.len(), 1);
        assert_eq!(
            transaction.output[0],
            TxOut {
                value: 5000000000,
                pk_len: 35,
                pk_len_bytes: 1,
                pk_script: vec![
                    33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108,
                    243, 201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
                    172
                ],
            }
        );
        assert_eq!(transaction.lock_time, 0);
    }

    #[test]
    fn test_serialize_and_deserialize_transaction() {
        let bytes = [
            1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 14, 4, 134, 231, 73, 77, 1, 81, 6, 47, 80,
            50, 83, 72, 47, 255, 255, 255, 255, 1, 0, 242, 5, 42, 1, 0, 0, 0, 35, 33, 3, 246, 217,
            255, 76, 18, 149, 148, 69, 202, 85, 73, 200, 17, 104, 59, 249, 200, 142, 99, 123, 34,
            45, 210, 224, 49, 17, 84, 196, 200, 92, 244, 35, 172, 0, 0, 0, 0,
        ];

        let tx = Transaction::deserialize(&bytes);
        assert!(tx.is_ok());

        let tx = tx.unwrap();
        let serialized = tx.serialize();
        assert!(serialized.is_ok());

        let serialized = serialized.unwrap();

        assert_eq!(serialized, bytes);
    }

    #[test]
    fn test_transaction_size() {
        let version = 1;

        let input = vec![TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 4294967295,
            },
            script_bytes: 31,
            script_bytes_amount: 1,
            signature_script: vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
            ],
            sequence: 4294967295,
        }];
        let output = vec![TxOut {
            value: 5000000000,
            pk_len: 35,
            pk_len_bytes: 1,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        }];
        let lock_time = 0;

        let transaction = Transaction {
            version,
            input,
            output,
            lock_time,
            tx_in_count: 1,
            tx_out_count: 1,
        };

        let expected_bytes = vec![
            1, 0, 0, 0, //version
            1, // n-tin
            // trin
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
            1,   // n-trout
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, // trout pk
            172, 0, 0, 0, 0,
        ]; // lock time

        assert_eq!(transaction.size(), expected_bytes.len());
    }

    #[test]
    fn test_serialize_tx_in() {
        let expected_bytes = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
        ];
        let previous_output = Outpoint {
            hash: [0; 32],
            index: 4294967295,
        };
        let script_bytes = 31;
        let signature_script = vec![
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47,
        ];
        let sequence = 4294967295;

        let tx_in = TxIn {
            previous_output,
            script_bytes,
            script_bytes_amount: 1,
            signature_script: signature_script.clone(),
            sequence,
        };

        let serialized = tx_in.serialize().unwrap();

        assert_eq!(serialized.len(), expected_bytes.len());
        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_tx_in() {
        let bytes = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, //hash
            255, 255, 255, 255, //index
            31,  // script bytes
            4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
            110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47, // script
            255, 255, 255, 255, // sequene
        ];

        let tx_in = TxIn::deserialize(&bytes).unwrap();

        assert_eq!(tx_in.previous_output.hash, [0u8; 32]);
        assert_eq!(tx_in.previous_output.index, 4294967295);
        assert_eq!(tx_in.script_bytes, 31);
        assert_eq!(
            tx_in.signature_script,
            vec![
                4, 231, 190, 191, 79, 1, 34, 23, 47, 80, 50, 83, 72, 47, 73, 99, 101, 73, 114, 111,
                110, 45, 81, 67, 45, 109, 105, 110, 101, 114, 47
            ]
        );
        assert_eq!(tx_in.sequence, 4294967295);
    }

    #[test]
    fn test_size_tx_in() {
        let previous_output = Outpoint {
            hash: [0; 32],
            index: 0,
        };
        let script_bytes = 10;
        let signature_script = vec![1, 2, 3, 4, 5];
        let sequence = 100;
        let script_bytes_amount = 1;

        let tx_in = TxIn {
            previous_output,
            script_bytes,
            signature_script: signature_script.clone(),
            sequence,
            script_bytes_amount,
        };

        let expected_size = 40 + script_bytes_amount + signature_script.len();
        let actual_size = tx_in.size();

        assert_eq!(expected_size, actual_size);
    }

    #[test]
    fn test_serialize_outpoint() {
        let outpoint = Outpoint {
            hash: [1u8; 32],
            index: 123,
        };

        let expected_bytes = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, // hash
            123, 0, 0, 0, // index
        ];

        let serialized = outpoint._serialize().unwrap();

        assert_eq!(serialized, expected_bytes);
    }

    #[test]
    fn test_deserialize_outpoint() {
        let bytes = vec![
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, // hash
            255, 255, 255, 255, // index
        ];

        let expected_outpoint = Outpoint {
            hash: [2u8; 32],
            index: 4294967295,
        };

        let deserialized = Outpoint::deserialize(&bytes).unwrap();

        assert_eq!(deserialized, expected_outpoint);
    }

    #[test]
    fn test_serialize_tx_out() {
        let expected_bytes = vec![
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
            172, // trout pk
        ];

        let txout = TxOut {
            value: 5000000000,
            pk_len: 1,
            pk_len_bytes: 35,
            pk_script: vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172,
            ],
        };

        let bytes = txout.serialize().unwrap();

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_deserialize_tx_out() {
        let bytes = vec![
            0, 242, 5, 42, 1, 0, 0, 0,  // value
            35, //pk_len
            33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243, 201,
            188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81,
            172, // trout pk
        ];

        let txout = TxOut::deserialize(&bytes).unwrap();
        assert_eq!(txout.value, 5000000000);
        assert_eq!(
            txout.pk_script,
            vec![
                33, 2, 142, 194, 100, 195, 242, 76, 65, 16, 171, 255, 30, 164, 219, 91, 108, 243,
                201, 188, 210, 174, 108, 157, 164, 77, 116, 205, 122, 47, 28, 107, 84, 81, 172
            ]
        );
    }

    #[test]
    fn test_size() {
        let value = 1000;
        let pk_len = 20;
        let pk_script = vec![1, 2, 3, 4, 5];
        let pk_len_bytes = 1;

        let tx_out = TxOut {
            value,
            pk_len,
            pk_script,
            pk_len_bytes,
        };

        let expected_size = 8 + pk_len_bytes + 5;
        let actual_size = tx_out.size();

        assert_eq!(expected_size, actual_size);
    }

    #[test]
    fn test_create_new_tx() {
        let prev_tx_bytes = [
            0x0d, 0x6f, 0xe5, 0x21, 0x3c, 0x0b, 0x32, 0x91, 0xf2, 0x08, 0xcb, 0xa8, 0xbf, 0xb5,
            0x9b, 0x74, 0x76, 0xdf, 0xfa, 0xcc, 0x4e, 0x5c, 0xb6, 0x6f, 0x6e, 0xb2, 0x0a, 0x08,
            0x08, 0x43, 0xa2, 0x99,
        ];
        let prev_tx = Uint256::from_le_bytes(prev_tx_bytes.clone());
        let prev_index = 13;
        let tx_in = TxIn::new(prev_tx, prev_index);

        let change_amount = 33000000;
        let public_account = "mzx5YhAH9kNHtcN481u6WkjeHjYtVeKVh2";
        let script = decode_base58(public_account);
        assert!(script.is_ok());
        let script = script.unwrap();
        let txout = TxOut::new(change_amount, script);
        assert!(txout.is_ok());
        let txout = txout.unwrap();

        let target_amount = 10000000;
        let target_account = "mnrVtF8DWjMu839VW3rBfgYaAfKk8983Xf";
        let target_h160 = decode_base58(target_account);
        assert!(target_h160.is_ok());
        let target_h160 = target_h160.unwrap();
        let tx_out_change = TxOut::new(target_amount, target_h160);
        assert!(tx_out_change.is_ok());
        let tx_out_change = tx_out_change.unwrap();

        let tx_obj = Transaction::new(1, vec![tx_in], vec![txout, tx_out_change], 0);
        assert!(tx_obj.is_ok());
        let tx_obj = tx_obj.unwrap();

        let serialize = tx_obj.serialize();
        assert!(serialize.is_ok());
        let serialize = serialize.unwrap();

        println!("serialize: {:?}", serialize);

        let bytes_serialized_oreilly = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x99, 0xa2, 0x43, 0x08, 0x08, 0x0a, 0xb2, 0x6e, 0x6f,
            0xb6, 0x5c, 0x4e, 0xcc, 0xfa, 0xdf, 0x76, 0x74, 0x9b, 0xb5, 0xbf, 0xa8, 0xcb, 0x08,
            0xf2, 0x91, 0x32, 0x0b, 0x3c, 0x21, 0xe5, 0x6f, 0x0d, 0x0d, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0x02, 0x40, 0x8a, 0xf7, 0x01, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xa9, 0x14, 0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52,
            0xc2, 0x01, 0x8e, 0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f, 0x88, 0xac, 0x80, 0x96, 0x98,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0x50, 0x7b, 0x27, 0x41, 0x1c,
            0xcf, 0x7f, 0x16, 0xf1, 0x02, 0x97, 0xde, 0x6c, 0xef, 0x3f, 0x29, 0x16, 0x23, 0xed,
            0xdf, 0x88, 0xac, 0x00, 0x00, 0x00, 0x00,
        ];

        let bytes_tx = serialize.as_slice();
        assert_eq!(bytes_tx, bytes_serialized_oreilly);
    }
}
