use crate::common::decoder::{
    _wif_to_hex, decode_base58, point_sec, script_serialized, signature_der,
};
use crate::common::utils_bytes;
use crate::common::{decoder::p2pkh_script_serialized, uint256::Uint256};
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
        let u256 = Uint256::from_be_bytes(*hash.as_byte_array());
        Ok(u256)
    }

    pub fn get_tx_from_file(txid: Uint256) -> Result<Transaction, NodoBitcoinError> {
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
        input: Vec<TxIn>,
        output: Vec<TxOut>,
        lock_time: u32,
    ) -> Result<Transaction, NodoBitcoinError> {
        let version = 1;
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

    fn get_new_tx_in(tx_in_base: TxIn, previous_tx: Transaction) -> TxIn {
        let previous_output = tx_in_base.clone().previous_output;
        let previous_index = previous_output.index;
        let previous_outpts = previous_tx.output;
        let previous_tx_out = &previous_outpts[previous_index as usize];

        let script_bytes = previous_tx_out.pk_len;
        let script_bytes_amount = previous_tx_out.pk_len_bytes;
        let signature_script = previous_tx_out.pk_script.clone();

        let sequence = tx_in_base.sequence;

        TxIn {
            previous_output,
            script_bytes,
            signature_script,
            sequence,
            script_bytes_amount,
        }
    }

    pub fn sig_hash(
        &self,
        index: usize,
        previous_tx: Transaction,
    ) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = Vec::new();
        bytes
            .write_all(&(self.version).to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let tx_in_count_prefix = utils_bytes::from_amount_bytes_to_prefix(self.tx_in_count);
        bytes
            .write_all(&(utils_bytes::_build_varint_bytes(tx_in_count_prefix, self.input.len())?))
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        for (i, tx_in) in self.input.iter().enumerate() {
            if i == index {
                let tx_new = Self::get_new_tx_in(tx_in.clone(), previous_tx.clone());
                bytes
                    .write_all(&tx_new.serialize()?)
                    .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
            } else {
                let tx_new = TxIn {
                    previous_output: tx_in.clone().previous_output,
                    script_bytes: 0,
                    signature_script: vec![],
                    sequence: tx_in.clone().sequence,
                    script_bytes_amount: 0,
                };

                bytes
                    .write_all(&tx_new.serialize()?)
                    .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
            }
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

        // SIGHASH_ALL
        let sighash_all: u32 = 1;
        bytes
            .write_all(&sighash_all.to_le_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;

        let hash = sha256d::Hash::hash(&bytes);
        let ret = *hash.as_byte_array();
        Ok(ret.to_vec())
    }

    pub fn sign_with_hexa_key(
        &mut self,
        input_index: usize,
        private_key_hexa: Vec<u8>,
        previous_tx: Transaction,
    ) -> Result<(), NodoBitcoinError> {
        let sign_hash = self.sig_hash(0, previous_tx)?;
        let signature_der = signature_der(&private_key_hexa, &sign_hash);
        let signature_der_bytes = signature_der.serialize_der().clone().as_ref().to_vec();

        let sighash_all: u8 = 1;
        let sig = [&signature_der_bytes[..], &sighash_all.to_be_bytes()[..]].concat();
        let sec = point_sec(&private_key_hexa)?;

        let script_sig = vec![sig, sec];

        self.input[input_index].sign(script_sig)?;

        Ok(())
    }

    pub fn sign_with_wif_compressed_key(
        &mut self,
        input_index: usize,
        private_key_compresed: &str,
        previous_tx: Transaction,
    ) -> Result<(), NodoBitcoinError> {
        let private_key_hexa = _wif_to_hex(private_key_compresed)?;
        self.sign_with_hexa_key(input_index, private_key_hexa, previous_tx)
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

    pub fn sign(&mut self, script_sig: Vec<Vec<u8>>) -> Result<(), NodoBitcoinError> {
        let mut pk_script = vec![];
        for script in script_sig {
            let partial_script = script_serialized(&script)?;
            pk_script.append(&mut partial_script.clone());
        }
        let pk_len = pk_script.len();
        let pk_len_bytes = utils_bytes::from_amount_bytes_to_prefix(pk_len);

        self.script_bytes_amount = pk_len_bytes as usize;
        self.script_bytes = pk_len;
        self.signature_script = pk_script;
        Ok(())
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

    pub fn new(amount: usize, account: &str) -> Result<TxOut, NodoBitcoinError> {
        let script = decode_base58(account)?;
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

    pub fn is_user_account_output(&self, account: &str) -> Result<bool, NodoBitcoinError> {
        let script = decode_base58(account)?;
        let p2pkh_script = p2pkh_script_serialized(&script)?;
        Ok(self.pk_script == p2pkh_script)
    }
}

#[cfg(test)]
mod tests {
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
        let txout = TxOut::new(change_amount, public_account);
        assert!(txout.is_ok());
        let txout = txout.unwrap();

        let target_amount = 10000000;
        let target_account = "mnrVtF8DWjMu839VW3rBfgYaAfKk8983Xf";
        let tx_out_change = TxOut::new(target_amount, target_account);
        assert!(tx_out_change.is_ok());
        let tx_out_change = tx_out_change.unwrap();

        let tx_obj = Transaction::new(vec![tx_in], vec![txout, tx_out_change], 0);
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

    #[test]
    fn test_sig_hash() {
        let sig_hash_expected: [u8; 32] = [
            0x27, 0xe0, 0xc5, 0x99, 0x4d, 0xec, 0x78, 0x24, 0xe5, 0x6d, 0xec, 0x6b, 0x2f, 0xcb,
            0x34, 0x2e, 0xb7, 0xcd, 0xb0, 0xd0, 0x95, 0x7c, 0x2f, 0xce, 0x98, 0x82, 0xf7, 0x15,
            0xe8, 0x5d, 0x81, 0xa6,
        ];

        let previous_tx = [
            0x01, 0x00, 0x00, 0x00, 0x02, 0x13, 0x7c, 0x53, 0xf0, 0xfb, 0x48, 0xf8, 0x36, 0x66,
            0xfc, 0xfd, 0x2f, 0xe9, 0xf1, 0x2d, 0x13, 0xe9, 0x4e, 0xe1, 0x09, 0xc5, 0xae, 0xab,
            0xbf, 0xa3, 0x2b, 0xb9, 0xe0, 0x25, 0x38, 0xf4, 0xcb, 0x00, 0x00, 0x00, 0x00, 0x6a,
            0x47, 0x30, 0x44, 0x02, 0x20, 0x7e, 0x60, 0x09, 0xad, 0x86, 0x36, 0x7f, 0xc4, 0xb1,
            0x66, 0xbc, 0x80, 0xbf, 0x10, 0xcf, 0x1e, 0x78, 0x83, 0x2a, 0x01, 0xe9, 0xbb, 0x49,
            0x1c, 0x6d, 0x12, 0x6e, 0xe8, 0xaa, 0x43, 0x6c, 0xb5, 0x02, 0x20, 0x0e, 0x29, 0xe6,
            0xdd, 0x77, 0x08, 0xed, 0x41, 0x9c, 0xd5, 0xba, 0x79, 0x89, 0x81, 0xc9, 0x60, 0xf0,
            0xcc, 0x81, 0x1b, 0x24, 0xe8, 0x94, 0xbf, 0xf0, 0x72, 0xfe, 0xa8, 0x07, 0x4a, 0x7c,
            0x4c, 0x01, 0x21, 0x03, 0xbc, 0x9e, 0x73, 0x97, 0xf7, 0x39, 0xc7, 0x0f, 0x42, 0x4a,
            0xa7, 0xdc, 0xce, 0x9d, 0x2e, 0x52, 0x1e, 0xb2, 0x28, 0xb0, 0xcc, 0xba, 0x61, 0x9c,
            0xd6, 0xa0, 0xb9, 0x69, 0x1d, 0xa7, 0x96, 0xa1, 0xff, 0xff, 0xff, 0xff, 0x51, 0x74,
            0x72, 0xe7, 0x7b, 0xc2, 0x9a, 0xe5, 0x9a, 0x91, 0x4f, 0x55, 0x21, 0x1f, 0x05, 0x02,
            0x45, 0x56, 0x81, 0x2a, 0x2d, 0xd7, 0xd8, 0xdf, 0x29, 0x32, 0x65, 0xac, 0xd8, 0x33,
            0x01, 0x59, 0x01, 0x00, 0x00, 0x00, 0x6b, 0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xf4,
            0xbf, 0xdb, 0x0b, 0x31, 0x85, 0xc7, 0x78, 0xcf, 0x28, 0xac, 0xba, 0xf1, 0x15, 0x37,
            0x63, 0x52, 0xf0, 0x91, 0xad, 0x9e, 0x27, 0x22, 0x5e, 0x6f, 0x3f, 0x35, 0x0b, 0x84,
            0x75, 0x79, 0xc7, 0x02, 0x20, 0x0d, 0x69, 0x17, 0x77, 0x73, 0xcd, 0x2b, 0xb9, 0x93,
            0xa8, 0x16, 0xa5, 0xae, 0x08, 0xe7, 0x7a, 0x62, 0x70, 0xcf, 0x46, 0xb3, 0x3f, 0x8f,
            0x79, 0xd4, 0x5b, 0x0c, 0xd1, 0x24, 0x4d, 0x9c, 0x4c, 0x01, 0x21, 0x03, 0x1c, 0x0b,
            0x0b, 0x95, 0xb5, 0x22, 0x80, 0x5e, 0xa9, 0xd0, 0x22, 0x5b, 0x19, 0x46, 0xec, 0xae,
            0xb1, 0x72, 0x7c, 0x0b, 0x36, 0xc7, 0xe3, 0x41, 0x65, 0x76, 0x9f, 0xd8, 0xed, 0x86,
            0x0b, 0xf5, 0xff, 0xff, 0xff, 0xff, 0x02, 0x7a, 0x95, 0x88, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x19, 0x76, 0xa9, 0x14, 0xa8, 0x02, 0xfc, 0x56, 0xc7, 0x04, 0xce, 0x87, 0xc4,
            0x2d, 0x7c, 0x92, 0xeb, 0x75, 0xe7, 0x89, 0x6b, 0xdc, 0x41, 0xae, 0x88, 0xac, 0xa5,
            0x51, 0x5e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xe8, 0x2b, 0xd7,
            0x5c, 0x9c, 0x66, 0x2c, 0x3f, 0x57, 0x00, 0xb3, 0x3f, 0xec, 0x8a, 0x67, 0x6b, 0x6e,
            0x93, 0x91, 0xd5, 0x88, 0xac, 0x00, 0x00, 0x00, 0x00,
        ];

        let previous_tx = Transaction::deserialize(&previous_tx[..]);
        assert!(previous_tx.is_ok());
        let previous_tx = previous_tx.unwrap();

        let tx_bytes = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d,
            0xfe, 0x69, 0xb3, 0xde, 0xf3, 0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f,
            0x71, 0xbf, 0x83, 0x03, 0xc6, 0xa9, 0x89, 0xc7, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x6b,
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xed, 0x81, 0xff, 0x19, 0x2e, 0x75, 0xa3, 0xfd,
            0x23, 0x04, 0x00, 0x4d, 0xca, 0xdb, 0x74, 0x6f, 0xa5, 0xe2, 0x4c, 0x50, 0x31, 0xcc,
            0xfc, 0xf2, 0x13, 0x20, 0xb0, 0x27, 0x74, 0x57, 0xc9, 0x8f, 0x02, 0x20, 0x7a, 0x98,
            0x6d, 0x95, 0x5c, 0x6e, 0x0c, 0xb3, 0x5d, 0x44, 0x6a, 0x89, 0xd3, 0xf5, 0x61, 0x00,
            0xf4, 0xd7, 0xf6, 0x78, 0x01, 0xc3, 0x19, 0x67, 0x74, 0x3a, 0x9c, 0x8e, 0x10, 0x61,
            0x5b, 0xed, 0x01, 0x21, 0x03, 0x49, 0xfc, 0x4e, 0x63, 0x1e, 0x36, 0x24, 0xa5, 0x45,
            0xde, 0x3f, 0x89, 0xf5, 0xd8, 0x68, 0x4c, 0x7b, 0x81, 0x38, 0xbd, 0x94, 0xbd, 0xd5,
            0x31, 0xd2, 0xe2, 0x13, 0xbf, 0x01, 0x6b, 0x27, 0x8a, 0xfe, 0xff, 0xff, 0xff, 0x02,
            0xa1, 0x35, 0xef, 0x01, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xbc, 0x3b,
            0x65, 0x4d, 0xca, 0x7e, 0x56, 0xb0, 0x4d, 0xca, 0x18, 0xf2, 0x56, 0x6c, 0xda, 0xf0,
            0x2e, 0x8d, 0x9a, 0xda, 0x88, 0xac, 0x99, 0xc3, 0x98, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x19, 0x76, 0xa9, 0x14, 0x1c, 0x4b, 0xc7, 0x62, 0xdd, 0x54, 0x23, 0xe3, 0x32, 0x16,
            0x67, 0x02, 0xcb, 0x75, 0xf4, 0x0d, 0xf7, 0x9f, 0xea, 0x12, 0x88, 0xac, 0x19, 0x43,
            0x06, 0x00,
        ];

        let transaction = Transaction::deserialize(&tx_bytes[..]);
        assert!(transaction.is_ok());
        let transaction = transaction.unwrap();

        let sign_hash = transaction.sig_hash(0, previous_tx);
        assert!(sign_hash.is_ok());
        let sign_hash = sign_hash.unwrap();

        assert_eq!(sign_hash.as_slice(), sig_hash_expected);
    }

    #[test]
    fn test_sign_with_hexa() {
        let previous_tx = [
            0x01, 0x00, 0x00, 0x00, 0x02, 0x13, 0x7c, 0x53, 0xf0, 0xfb, 0x48, 0xf8, 0x36, 0x66,
            0xfc, 0xfd, 0x2f, 0xe9, 0xf1, 0x2d, 0x13, 0xe9, 0x4e, 0xe1, 0x09, 0xc5, 0xae, 0xab,
            0xbf, 0xa3, 0x2b, 0xb9, 0xe0, 0x25, 0x38, 0xf4, 0xcb, 0x00, 0x00, 0x00, 0x00, 0x6a,
            0x47, 0x30, 0x44, 0x02, 0x20, 0x7e, 0x60, 0x09, 0xad, 0x86, 0x36, 0x7f, 0xc4, 0xb1,
            0x66, 0xbc, 0x80, 0xbf, 0x10, 0xcf, 0x1e, 0x78, 0x83, 0x2a, 0x01, 0xe9, 0xbb, 0x49,
            0x1c, 0x6d, 0x12, 0x6e, 0xe8, 0xaa, 0x43, 0x6c, 0xb5, 0x02, 0x20, 0x0e, 0x29, 0xe6,
            0xdd, 0x77, 0x08, 0xed, 0x41, 0x9c, 0xd5, 0xba, 0x79, 0x89, 0x81, 0xc9, 0x60, 0xf0,
            0xcc, 0x81, 0x1b, 0x24, 0xe8, 0x94, 0xbf, 0xf0, 0x72, 0xfe, 0xa8, 0x07, 0x4a, 0x7c,
            0x4c, 0x01, 0x21, 0x03, 0xbc, 0x9e, 0x73, 0x97, 0xf7, 0x39, 0xc7, 0x0f, 0x42, 0x4a,
            0xa7, 0xdc, 0xce, 0x9d, 0x2e, 0x52, 0x1e, 0xb2, 0x28, 0xb0, 0xcc, 0xba, 0x61, 0x9c,
            0xd6, 0xa0, 0xb9, 0x69, 0x1d, 0xa7, 0x96, 0xa1, 0xff, 0xff, 0xff, 0xff, 0x51, 0x74,
            0x72, 0xe7, 0x7b, 0xc2, 0x9a, 0xe5, 0x9a, 0x91, 0x4f, 0x55, 0x21, 0x1f, 0x05, 0x02,
            0x45, 0x56, 0x81, 0x2a, 0x2d, 0xd7, 0xd8, 0xdf, 0x29, 0x32, 0x65, 0xac, 0xd8, 0x33,
            0x01, 0x59, 0x01, 0x00, 0x00, 0x00, 0x6b, 0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xf4,
            0xbf, 0xdb, 0x0b, 0x31, 0x85, 0xc7, 0x78, 0xcf, 0x28, 0xac, 0xba, 0xf1, 0x15, 0x37,
            0x63, 0x52, 0xf0, 0x91, 0xad, 0x9e, 0x27, 0x22, 0x5e, 0x6f, 0x3f, 0x35, 0x0b, 0x84,
            0x75, 0x79, 0xc7, 0x02, 0x20, 0x0d, 0x69, 0x17, 0x77, 0x73, 0xcd, 0x2b, 0xb9, 0x93,
            0xa8, 0x16, 0xa5, 0xae, 0x08, 0xe7, 0x7a, 0x62, 0x70, 0xcf, 0x46, 0xb3, 0x3f, 0x8f,
            0x79, 0xd4, 0x5b, 0x0c, 0xd1, 0x24, 0x4d, 0x9c, 0x4c, 0x01, 0x21, 0x03, 0x1c, 0x0b,
            0x0b, 0x95, 0xb5, 0x22, 0x80, 0x5e, 0xa9, 0xd0, 0x22, 0x5b, 0x19, 0x46, 0xec, 0xae,
            0xb1, 0x72, 0x7c, 0x0b, 0x36, 0xc7, 0xe3, 0x41, 0x65, 0x76, 0x9f, 0xd8, 0xed, 0x86,
            0x0b, 0xf5, 0xff, 0xff, 0xff, 0xff, 0x02, 0x7a, 0x95, 0x88, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x19, 0x76, 0xa9, 0x14, 0xa8, 0x02, 0xfc, 0x56, 0xc7, 0x04, 0xce, 0x87, 0xc4,
            0x2d, 0x7c, 0x92, 0xeb, 0x75, 0xe7, 0x89, 0x6b, 0xdc, 0x41, 0xae, 0x88, 0xac, 0xa5,
            0x51, 0x5e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xe8, 0x2b, 0xd7,
            0x5c, 0x9c, 0x66, 0x2c, 0x3f, 0x57, 0x00, 0xb3, 0x3f, 0xec, 0x8a, 0x67, 0x6b, 0x6e,
            0x93, 0x91, 0xd5, 0x88, 0xac, 0x00, 0x00, 0x00, 0x00,
        ];
        let previous_tx = Transaction::deserialize(&previous_tx[..]);
        assert!(previous_tx.is_ok());
        let previous_tx = previous_tx.unwrap();

        let tx_bytes = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d,
            0xfe, 0x69, 0xb3, 0xde, 0xf3, 0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f,
            0x71, 0xbf, 0x83, 0x03, 0xc6, 0xa9, 0x89, 0xc7, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x6b,
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xed, 0x81, 0xff, 0x19, 0x2e, 0x75, 0xa3, 0xfd,
            0x23, 0x04, 0x00, 0x4d, 0xca, 0xdb, 0x74, 0x6f, 0xa5, 0xe2, 0x4c, 0x50, 0x31, 0xcc,
            0xfc, 0xf2, 0x13, 0x20, 0xb0, 0x27, 0x74, 0x57, 0xc9, 0x8f, 0x02, 0x20, 0x7a, 0x98,
            0x6d, 0x95, 0x5c, 0x6e, 0x0c, 0xb3, 0x5d, 0x44, 0x6a, 0x89, 0xd3, 0xf5, 0x61, 0x00,
            0xf4, 0xd7, 0xf6, 0x78, 0x01, 0xc3, 0x19, 0x67, 0x74, 0x3a, 0x9c, 0x8e, 0x10, 0x61,
            0x5b, 0xed, 0x01, 0x21, 0x03, 0x49, 0xfc, 0x4e, 0x63, 0x1e, 0x36, 0x24, 0xa5, 0x45,
            0xde, 0x3f, 0x89, 0xf5, 0xd8, 0x68, 0x4c, 0x7b, 0x81, 0x38, 0xbd, 0x94, 0xbd, 0xd5,
            0x31, 0xd2, 0xe2, 0x13, 0xbf, 0x01, 0x6b, 0x27, 0x8a, 0xfe, 0xff, 0xff, 0xff, 0x02,
            0xa1, 0x35, 0xef, 0x01, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xbc, 0x3b,
            0x65, 0x4d, 0xca, 0x7e, 0x56, 0xb0, 0x4d, 0xca, 0x18, 0xf2, 0x56, 0x6c, 0xda, 0xf0,
            0x2e, 0x8d, 0x9a, 0xda, 0x88, 0xac, 0x99, 0xc3, 0x98, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x19, 0x76, 0xa9, 0x14, 0x1c, 0x4b, 0xc7, 0x62, 0xdd, 0x54, 0x23, 0xe3, 0x32, 0x16,
            0x67, 0x02, 0xcb, 0x75, 0xf4, 0x0d, 0xf7, 0x9f, 0xea, 0x12, 0x88, 0xac, 0x19, 0x43,
            0x06, 0x00,
        ];

        let transaction = Transaction::deserialize(&tx_bytes[..]);
        assert!(transaction.is_ok());
        let mut transaction = transaction.unwrap();

        let private_key_hexa: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x84, 0x5f, 0xed,
        ];

        let input_index = 0;
        let result =
            transaction.sign_with_hexa_key(input_index, private_key_hexa.to_vec(), previous_tx);

        assert!(result.is_ok());

        let tx_in_bytes = transaction.input[0].serialize();
        assert!(tx_in_bytes.is_ok());
        let tx_in_bytes = tx_in_bytes.unwrap();

        let tx_bytes = transaction.serialize();
        assert!(tx_bytes.is_ok());
        let tx_bytes = tx_bytes.unwrap();

        let expected_tx_in_bytes: [u8; 147] = [
            0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d, 0xfe, 0x69, 0xb3, 0xde, 0xf3,
            0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f, 0x71, 0xbf, 0x83, 0x03, 0xc6,
            0xa9, 0x89, 0xc7, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x6a, 0x47, 0x30, 0x44, 0x02, 0x20,
            0x7d, 0xb2, 0x40, 0x2a, 0x33, 0x11, 0xa3, 0xb8, 0x45, 0xb0, 0x38, 0x88, 0x5e, 0x3d,
            0xd8, 0x89, 0xc0, 0x81, 0x26, 0xa8, 0x57, 0x0f, 0x26, 0xa8, 0x44, 0xe3, 0xe4, 0x04,
            0x9c, 0x48, 0x2a, 0x11, 0x02, 0x20, 0x10, 0x17, 0x8c, 0xdc, 0xa4, 0x12, 0x9e, 0xac,
            0xbe, 0xab, 0x7c, 0x44, 0x64, 0x8b, 0xf5, 0xac, 0x1f, 0x9c, 0xac, 0x21, 0x7c, 0xd6,
            0x09, 0xd2, 0x16, 0xec, 0x2e, 0xbc, 0x8d, 0x24, 0x2c, 0x0a, 0x01, 0x21, 0x03, 0x93,
            0x55, 0x81, 0xe5, 0x2c, 0x35, 0x4c, 0xd2, 0xf4, 0x84, 0xfe, 0x8e, 0xd8, 0x3a, 0xf7,
            0xa3, 0x09, 0x70, 0x05, 0xb2, 0xf9, 0xc6, 0x0b, 0xff, 0x71, 0xd3, 0x5b, 0xd7, 0x95,
            0xf5, 0x4b, 0x67, 0xfe, 0xff, 0xff, 0xff,
        ];

        let expected_tx_bytes: [u8; 225] = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x81, 0x3f, 0x79, 0x01, 0x1a, 0xcb, 0x80, 0x92, 0x5d,
            0xfe, 0x69, 0xb3, 0xde, 0xf3, 0x55, 0xfe, 0x91, 0x4b, 0xd1, 0xd9, 0x6a, 0x3f, 0x5f,
            0x71, 0xbf, 0x83, 0x03, 0xc6, 0xa9, 0x89, 0xc7, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x6a,
            0x47, 0x30, 0x44, 0x02, 0x20, 0x7d, 0xb2, 0x40, 0x2a, 0x33, 0x11, 0xa3, 0xb8, 0x45,
            0xb0, 0x38, 0x88, 0x5e, 0x3d, 0xd8, 0x89, 0xc0, 0x81, 0x26, 0xa8, 0x57, 0x0f, 0x26,
            0xa8, 0x44, 0xe3, 0xe4, 0x04, 0x9c, 0x48, 0x2a, 0x11, 0x02, 0x20, 0x10, 0x17, 0x8c,
            0xdc, 0xa4, 0x12, 0x9e, 0xac, 0xbe, 0xab, 0x7c, 0x44, 0x64, 0x8b, 0xf5, 0xac, 0x1f,
            0x9c, 0xac, 0x21, 0x7c, 0xd6, 0x09, 0xd2, 0x16, 0xec, 0x2e, 0xbc, 0x8d, 0x24, 0x2c,
            0x0a, 0x01, 0x21, 0x03, 0x93, 0x55, 0x81, 0xe5, 0x2c, 0x35, 0x4c, 0xd2, 0xf4, 0x84,
            0xfe, 0x8e, 0xd8, 0x3a, 0xf7, 0xa3, 0x09, 0x70, 0x05, 0xb2, 0xf9, 0xc6, 0x0b, 0xff,
            0x71, 0xd3, 0x5b, 0xd7, 0x95, 0xf5, 0x4b, 0x67, 0xfe, 0xff, 0xff, 0xff, 0x02, 0xa1,
            0x35, 0xef, 0x01, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xbc, 0x3b, 0x65,
            0x4d, 0xca, 0x7e, 0x56, 0xb0, 0x4d, 0xca, 0x18, 0xf2, 0x56, 0x6c, 0xda, 0xf0, 0x2e,
            0x8d, 0x9a, 0xda, 0x88, 0xac, 0x99, 0xc3, 0x98, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xa9, 0x14, 0x1c, 0x4b, 0xc7, 0x62, 0xdd, 0x54, 0x23, 0xe3, 0x32, 0x16, 0x67,
            0x02, 0xcb, 0x75, 0xf4, 0x0d, 0xf7, 0x9f, 0xea, 0x12, 0x88, 0xac, 0x19, 0x43, 0x06,
            0x00,
        ];

        assert_eq!(expected_tx_bytes, tx_bytes.as_slice());
        assert_eq!(expected_tx_in_bytes, tx_in_bytes.as_slice());
    }

    #[test]
    fn test_new_tx_signed() {
        let previous_tx = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x98, 0x2e, 0x32, 0xee, 0xc9, 0x30, 0x06, 0x50, 0x1d,
            0x6b, 0x93, 0x3d, 0x7d, 0x0d, 0x47, 0x04, 0x2d, 0x1c, 0x8d, 0x8e, 0x33, 0xdc, 0x39,
            0x4e, 0x2f, 0x90, 0x97, 0xe8, 0x65, 0x28, 0x67, 0xdd, 0x0d, 0x00, 0x00, 0x00, 0x6b,
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xbe, 0x0a, 0x2f, 0x95, 0xc1, 0xad, 0x0f, 0x7f,
            0x00, 0xa4, 0x21, 0x95, 0xc4, 0x51, 0x0e, 0x38, 0x04, 0x1c, 0x0d, 0x09, 0x5a, 0xed,
            0x58, 0x9b, 0xe1, 0x9d, 0x61, 0x73, 0x43, 0x2c, 0xc2, 0x25, 0x02, 0x20, 0x28, 0xc0,
            0xbd, 0xa8, 0x92, 0xda, 0x40, 0xfc, 0x9d, 0x51, 0x4b, 0x57, 0x64, 0xb0, 0x52, 0x34,
            0xf0, 0x45, 0x86, 0x5f, 0xd6, 0xe0, 0xb6, 0xbf, 0x61, 0x9f, 0x26, 0x40, 0x59, 0x71,
            0xf3, 0x77, 0x01, 0x21, 0x03, 0x54, 0x97, 0xd8, 0x52, 0xf4, 0x16, 0xb4, 0x84, 0x4c,
            0xb2, 0x39, 0x68, 0x6d, 0xeb, 0x16, 0xaa, 0xdf, 0x3e, 0x20, 0xa7, 0x60, 0x7f, 0x38,
            0x72, 0xa8, 0xb8, 0x3b, 0x71, 0x31, 0x17, 0x57, 0x35, 0xff, 0xff, 0xff, 0xff, 0x02,
            0x80, 0x84, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0x50, 0x7b,
            0x27, 0x41, 0x1c, 0xcf, 0x7f, 0x16, 0xf1, 0x02, 0x97, 0xde, 0x6c, 0xef, 0x3f, 0x29,
            0x16, 0x23, 0xed, 0xdf, 0x88, 0xac, 0xe0, 0xfd, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x19, 0x76, 0xa9, 0x14, 0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7,
            0x52, 0xc2, 0x01, 0x8e, 0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f, 0x88, 0xac, 0x00, 0x00,
            0x00, 0x00,
        ];
        let previous_tx = Transaction::deserialize(&previous_tx[..]);
        assert!(previous_tx.is_ok());
        let previous_tx = previous_tx.unwrap();

        let prev_index: usize = 1;

        let private_key_hexa: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x84, 0x5f, 0xed,
        ];

        let _tx_id = previous_tx.txid();
        //let _bytes_previous_id = _tx_id._get_le_bytes();

        let target_address = "mwJn1YPMq7y5F8J3LkC5Hxg9PHyZ5K4cFv";
        let target_amount: usize = 1000000;

        let change_address = "mzx5YhAH9kNHtcN481u6WkjeHjYtVeKVh2";
        let change_amount: usize = 900000;

        let mut tx_ins = vec![];
        let previous_tx_id = previous_tx.txid();
        assert!(previous_tx_id.is_ok());
        let previous_tx_id = previous_tx_id.unwrap();

        let tx_in = TxIn::new(previous_tx_id, prev_index);
        tx_ins.push(tx_in);

        let mut tx_outs = vec![];
        let tx_out = TxOut::new(target_amount, target_address);
        assert!(tx_out.is_ok());
        let tx_out = tx_out.unwrap();

        tx_outs.push(tx_out);

        let tx_out_change = TxOut::new(change_amount, change_address);
        assert!(tx_out_change.is_ok());
        let tx_out_change = tx_out_change.unwrap();

        tx_outs.push(tx_out_change);

        let tx_obj = Transaction::new(tx_ins, tx_outs, 0);
        assert!(tx_obj.is_ok());
        let mut tx_obj = tx_obj.unwrap();

        let result = tx_obj.sign_with_hexa_key(0, private_key_hexa.to_vec(), previous_tx);
        assert!(result.is_ok());

        let tx_obj_bytes = tx_obj.serialize();
        assert!(tx_obj_bytes.is_ok());
        let tx_obj_bytes = tx_obj_bytes.unwrap();

        let expected_obj_bytes = [
            0x01, 0x00, 0x00, 0x00, 0x01, 0x1c, 0x5f, 0xb4, 0xa3, 0x5c, 0x40, 0x64, 0x7b, 0xca,
            0xcf, 0xef, 0xfc, 0xb8, 0x68, 0x6f, 0x1e, 0x99, 0x25, 0x77, 0x4c, 0x07, 0xa1, 0xdd,
            0x26, 0xf6, 0x55, 0x1f, 0x67, 0xbc, 0xc4, 0xa1, 0x75, 0x01, 0x00, 0x00, 0x00, 0x6a,
            0x47, 0x30, 0x44, 0x02, 0x20, 0x71, 0x8c, 0x28, 0xfc, 0x34, 0xdf, 0xe6, 0x47, 0xde,
            0xb3, 0xed, 0xaf, 0x5b, 0xdd, 0x69, 0xb6, 0x8a, 0xb0, 0x16, 0x15, 0x0c, 0xc1, 0xe6,
            0x04, 0xd2, 0x90, 0x84, 0x92, 0x34, 0xb0, 0xc4, 0xfc, 0x02, 0x20, 0x7a, 0xb4, 0xef,
            0x43, 0xd4, 0xff, 0xbe, 0x48, 0x2b, 0x30, 0xe6, 0xb2, 0x1a, 0x86, 0x46, 0x01, 0xb7,
            0x7e, 0x01, 0xba, 0x9a, 0xac, 0xa2, 0x5a, 0xb3, 0x04, 0xe8, 0x1b, 0xab, 0x68, 0xe2,
            0xc1, 0x01, 0x21, 0x03, 0x93, 0x55, 0x81, 0xe5, 0x2c, 0x35, 0x4c, 0xd2, 0xf4, 0x84,
            0xfe, 0x8e, 0xd8, 0x3a, 0xf7, 0xa3, 0x09, 0x70, 0x05, 0xb2, 0xf9, 0xc6, 0x0b, 0xff,
            0x71, 0xd3, 0x5b, 0xd7, 0x95, 0xf5, 0x4b, 0x67, 0xff, 0xff, 0xff, 0xff, 0x02, 0x40,
            0x42, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xa9, 0x14, 0xad, 0x34, 0x6f,
            0x8e, 0xb5, 0x7d, 0xee, 0x9a, 0x37, 0x98, 0x17, 0x16, 0xe4, 0x98, 0x12, 0x0a, 0xe8,
            0x0e, 0x44, 0xf7, 0x88, 0xac, 0xa0, 0xbb, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19,
            0x76, 0xa9, 0x14, 0xd5, 0x2a, 0xd7, 0xca, 0x9b, 0x3d, 0x09, 0x6a, 0x38, 0xe7, 0x52,
            0xc2, 0x01, 0x8e, 0x6f, 0xbc, 0x40, 0xcd, 0xf2, 0x6f, 0x88, 0xac, 0x00, 0x00, 0x00,
            0x00,
        ];
        assert_eq!(tx_obj_bytes, expected_obj_bytes);
    }
}
