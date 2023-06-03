use std::collections::HashMap;
use crate::blockchain::transaction::{Transaction, TxIn, TxOut, Outpoint};
use crate::errores::NodoBitcoinError;
use crate::common::uint256::Uint256;

pub struct UTXO {
    pub tx_id: Uint256,
    pub output_index: u32,
    pub amount: u64,
    pub recipient: Vec<u8>,
}

pub struct UTXOSet {
    pub utxos: HashMap<Uint256, Vec<UTXO>>,
}

impl UTXOSet {
    pub fn new() -> Self {
        UTXOSet {
            utxos: HashMap::new(),
        }
    }

    pub fn build_from_transactions(&mut self, transactions: Vec<Transaction>) -> Result<(), NodoBitcoinError> {
        let mut spent_outputs: HashMap<Uint256, Vec<u32>> = HashMap::new();

        for transaction in transactions.iter().rev() {
            for tx_in in &transaction.input {
                let previous_tx_id = Uint256::_from_bytes(tx_in.previous_output.hash);
                let output_index = tx_in.previous_output.index;

                let spent_outputs_for_tx = spent_outputs.entry(previous_tx_id).or_insert(Vec::new());
                spent_outputs_for_tx.push(output_index);
            }
        }

        for transaction in transactions {
            let tx_id = transaction._txid()?;
            for (output_index, tx_out) in transaction.output.iter().enumerate() {
                if spent_outputs.contains_key(&tx_id) && spent_outputs[&tx_id].contains(&(output_index as u32)) {
                    // si la salida de transacción está gastada no es un UXTO
                    continue;
                }

                let utxo = UTXO {
                    tx_id,
                    output_index: output_index as u32,
                    amount: tx_out.value,
                    recipient: tx_out.pk_script.clone(),
                };

                let utxos_for_tx = self.utxos.entry(tx_id).or_insert(Vec::new());
                utxos_for_tx.push(utxo);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_from_transactions() {
        let tx_out1 = TxOut {
            value: 100,
            pk_len: 0,
            pk_script: vec![],
            pk_len_bytes: 0,
        };
        let tx_out2 = TxOut {
            value: 200,
            pk_len: 0,
            pk_script: vec![],
            pk_len_bytes: 0,
        };

        let tx_in1 = TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 0,
            },
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0,
            script_bytes_amount: 0,
        };
        let tx_in2 = TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 0,
            },
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0,
            script_bytes_amount: 0,
        };

        let transaction1 = Transaction {
            version: 1,
            input: vec![tx_in1.clone()],
            output: vec![tx_out1.clone()],
            lock_time: 0,
            tx_in_count: 1,
            tx_out_count: 1,
        };
        let transaction2 = Transaction {
            version: 1,
            input: vec![tx_in2.clone()],
            output: vec![tx_out2.clone()],
            lock_time: 0,
            tx_in_count: 1,
            tx_out_count: 1,
        };
        let transactions = vec![transaction1.clone(), transaction2.clone()];

        let mut utxo_set = UTXOSet::new();
        utxo_set.build_from_transactions(transactions);

        assert_eq!(utxo_set.utxos.len(), 2);
        assert!(utxo_set.utxos.contains_key(&transaction1._txid().unwrap()));
        assert!(utxo_set.utxos.contains_key(&transaction2._txid().unwrap()));

        let utxos_tx1 = utxo_set.utxos.get(&transaction1._txid().unwrap()).unwrap();
        assert_eq!(utxos_tx1.len(), 1);
        assert_eq!(utxos_tx1[0].amount, tx_out1.value);

        let utxos_tx2 = utxo_set.utxos.get(&transaction2._txid().unwrap()).unwrap();
        assert_eq!(utxos_tx2.len(), 1);
        assert_eq!(utxos_tx2[0].amount, tx_out2.value);
    }

    #[test]
    fn test_build_from_transactions_for_spent_outputs() {
        let mut utxo_set = UTXOSet::new();

        let transaction1 = Transaction {
            input: vec![],
            output: vec![TxOut {
                value: 10,
                pk_script: vec![],
                pk_len:0,
                pk_len_bytes:0,
            }],
            lock_time:0,
            tx_in_count:1,
            tx_out_count:1,
            version:1,
        };

        let transaction2 = Transaction {
            input: vec![TxIn {
                previous_output: Outpoint {
                    hash: transaction1._txid().unwrap()._to_bytes(),
                    index: 0,
                },
                signature_script: vec![],
                script_bytes: 0,
                script_bytes_amount:0,
                sequence:0,
            }],
            output: vec![TxOut {
                value: 5,
                pk_script: vec![],
                pk_len:0,
                pk_len_bytes:0,
            }],
            lock_time:0,
            tx_in_count:1,
            tx_out_count:1,
            version:1,
        };

        utxo_set.build_from_transactions(vec![transaction1,transaction2]).unwrap();
        assert_eq!(utxo_set.utxos.len(), 0);
    }
}
