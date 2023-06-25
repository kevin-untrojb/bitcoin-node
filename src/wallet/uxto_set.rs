use crate::blockchain::block::SerializedBlock;
use crate::blockchain::transaction::{Transaction, TxIn, TxOut};
use crate::common::uint256::Uint256;
use crate::common::utils_file::{read_decoded_string_offset, save_encoded_len_bytes};
use crate::errores::NodoBitcoinError;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::{fmt, mem};

use super::user::Account;

#[derive(Debug, Clone)]
pub struct Utxo {
    pub tx_id: Uint256,
    pub output_index: u32,
    pub tx_out: TxOut,
    pub pk_script: Vec<u8>,
    pub tx: Transaction,
}

impl fmt::Display for Utxo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "tx_id: {:?}\namount: {:?}\naccount: {:?}",
            self.tx_id, self.tx_out.value, self.pk_script
        )
    }
}

impl Utxo {
    // serializar en una cadena de bytes el utxo
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut serialized = Vec::new();
        serialized.extend(self.tx_id.get_bytes());
        serialized.extend(self.output_index.to_ne_bytes());

        let tx_out_bytes = self.tx_out.serialize()?;
        serialized.extend(tx_out_bytes.len().to_ne_bytes());
        serialized.extend(tx_out_bytes);

        serialized.extend(self.pk_script.clone().len().to_ne_bytes());
        serialized.extend(self.pk_script.clone());

        let tx_bytes = self.tx.serialize()?;
        serialized.extend(tx_bytes.len().to_ne_bytes());
        serialized.extend(tx_bytes);
        Ok(serialized)
    }

    // deseralizar una cadena de bytes en un utxo y devolverlo
    pub fn deserialize(bytes: &[u8]) -> Result<Utxo, NodoBitcoinError> {
        let sizeof_usize: usize = mem::size_of::<usize>();
        let mut offset = 0;
        let tx_id = Uint256::from_be_bytes(
            bytes[offset..offset + 32]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 32;
        let output_index = u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;
        let tx_out_len =
            usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
        offset += sizeof_usize;
        let tx_out = TxOut::deserialize(&bytes[offset..offset + tx_out_len])?;
        offset += tx_out_len;
        let pk_script_len =
            usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
        offset += sizeof_usize;
        let pk_script = bytes[offset..offset + pk_script_len].to_vec();
        offset += pk_script_len;
        let tx_len = usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
        offset += sizeof_usize;
        let tx = Transaction::deserialize(&bytes[offset..offset + tx_len])?;

        Ok(Utxo {
            tx_id,
            output_index,
            tx_out,
            pk_script,
            tx,
        })
    }

    pub fn save(&self, file: &mut dyn Write) -> Result<(), NodoBitcoinError> {
        let binding = self.serialize()?;
        let serialized = binding.as_slice();
        let len_utxo = serialized.len();
        file.write_all(&len_utxo.to_ne_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        file.write_all(serialized)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TxReport {
    pub is_pending: bool,
    pub timestamp: u32,
    pub tx_id: Uint256,
    pub amount: i128,
    is_tx_in: bool,
    index: u32,
}

impl TxReport {
    pub fn new(
        is_pending: bool,
        timestamp: u32,
        tx_id: Uint256,
        amount: i128,
        is_tx_in: bool,
        index: u32,
    ) -> TxReport {
        TxReport {
            is_pending,
            timestamp,
            tx_id,
            amount,
            is_tx_in,
            index,
        }
    }

    // serializar en una cadena de bytes el txreport
    pub fn serialize(&self) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut serialized = Vec::new();
        serialized.extend((self.is_pending as u8).to_ne_bytes());
        serialized.extend(self.timestamp.to_ne_bytes());
        serialized.extend(self.tx_id.get_bytes());
        serialized.extend(self.amount.to_ne_bytes());
        serialized.extend((self.is_tx_in as u8).to_ne_bytes());
        serialized.extend(self.index.to_ne_bytes());
        Ok(serialized)
    }

    // deseralizar una cadena de bytes en un txreport y devolverlo
    pub fn deserialize(bytes: &[u8]) -> Result<TxReport, NodoBitcoinError> {
        let sizeof_u32: usize = mem::size_of::<u32>();
        let sizeof_i128: usize = mem::size_of::<i128>();
        let mut offset = 0;
        let is_pending = bytes[0] != 0;
        offset += 1;
        let timestamp = u32::from_ne_bytes(bytes[offset..offset + sizeof_u32].try_into().unwrap());
        offset += sizeof_u32;
        let tx_id = Uint256::from_be_bytes(
            bytes[offset..offset + 32]
                .try_into()
                .map_err(|_| NodoBitcoinError::NoSePuedeLeerLosBytes)?,
        );
        offset += 32;
        let amount = i128::from_ne_bytes(bytes[offset..offset + sizeof_i128].try_into().unwrap());
        offset += sizeof_i128;
        let is_tx_in = bytes[offset] != 0;
        offset += 1;
        let index = u32::from_ne_bytes(bytes[offset..offset + sizeof_u32].try_into().unwrap());

        Ok(TxReport {
            is_pending,
            timestamp,
            tx_id,
            amount,
            is_tx_in,
            index,
        })
    }

    pub fn save(&self, file: &mut dyn Write) -> Result<(), NodoBitcoinError> {
        let binding = self.serialize()?;
        let serialized = binding.as_slice();
        let len_tx_report = serialized.len();
        file.write_all(&len_tx_report.to_ne_bytes())
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        file.write_all(serialized)
            .map_err(|_| NodoBitcoinError::NoSePuedeEscribirLosBytes)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UTXOSet {
    pub utxos_for_account: HashMap<String, Vec<Utxo>>,
    pub account_for_txid_index: HashMap<(Uint256, u32), String>,
    pub tx_report_by_accounts: HashMap<String, Vec<TxReport>>,
    pub tx_report_pending_by_accounts: HashMap<String, Vec<TxReport>>,
    pub last_timestamp: u32,
}

impl fmt::Display for UTXOSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut utxos = Vec::new();
        for (key, utxos_for_tx) in &self.utxos_for_account {
            let format_string = format!("key: {:?}\nutxos_for_tx: {:?}", key, utxos_for_tx);
            utxos.push(format_string);
        }
        write!(f, "UTXOSet\n{:?}]", utxos)
    }
}

const UTXO_FOR_ACCOUNT_FILENAME: &str = "utxos_for_account.dat";
const ACCOUNT_FOR_TXID_INDEX_FILENAME: &str = "account_for_txid_index.dat";
const TX_REPORT_BY_ACCOUNT_FILENAME: &str = "tx_report_by_accounts.dat";
const TX_REPORT_PENDING_BY_ACCOUNT_FILENAME: &str = "tx_report_pending_by_accounts.dat";

impl Default for UTXOSet {
    fn default() -> Self {
        UTXOSet::new()
    }
}

impl UTXOSet {
    pub fn new() -> Self {
        UTXOSet {
            utxos_for_account: HashMap::new(),
            account_for_txid_index: HashMap::new(),
            tx_report_by_accounts: HashMap::new(),
            tx_report_pending_by_accounts: HashMap::new(),
            last_timestamp: 0,
        }
    }

    // verificar si existe una tx report para un account
    pub fn existe_tx_report_para_account(&self, account: String, new_tx_report: &TxReport) -> bool {
        if !self.tx_report_by_accounts.contains_key(&account) {
            return false;
        }

        let tx_report_by_accounts = match self.tx_report_by_accounts.get(&account) {
            Some(tx_by_account) => tx_by_account,
            None => return false,
        };

        for tx_report in tx_report_by_accounts {
            if tx_report.tx_id == new_tx_report.tx_id
                && tx_report.index == new_tx_report.index
                && tx_report.is_tx_in == new_tx_report.is_tx_in
            {
                return true;
            }
        }
        false
    }

    fn eliminar_tx_report_pending(&self, tx_report_to_delete: TxReport) {
        let tx_id = tx_report_to_delete.tx_id;
        let index = tx_report_to_delete.index;
        let is_tx_in = tx_report_to_delete.is_tx_in;

        let mut tx_report_pending_by_accounts = self.tx_report_pending_by_accounts.clone();
        for (_, tx_reports) in tx_report_pending_by_accounts.iter_mut() {
            tx_reports.retain(|tx_report| {
                !(tx_report.tx_id == tx_id
                    && tx_report.index == index
                    && tx_report.is_tx_in == is_tx_in)
            });
        }
    }

    fn agregar_tx_report_desde_out(
        &mut self,
        current_account: String,
        utxo: Utxo,
        timestamp: u32,
        pending: bool,
    ) -> Option<TxReport> {
        let tx_report = TxReport {
            is_pending: pending,
            timestamp,
            tx_id: utxo.tx_id,
            amount: utxo.tx_out.value as i128,
            is_tx_in: false,
            index: utxo.output_index,
        };

        if self.existe_tx_report_para_account(current_account.clone(), &tx_report) {
            return None;
        }

        let tx_report_by_accounts = self
            .tx_report_by_accounts
            .entry(current_account)
            .or_default();
        tx_report_by_accounts.push(tx_report.clone());
        Some(tx_report)
    }

    fn agregar_tx_report_desde_in(
        &mut self,
        tx_id: Uint256,
        timestamp: u32,
        pending: bool,
        tx_in_index: u32,
        key: (Uint256, u32),
    ) -> Option<TxReport> {
        let (previous_tx_id, output_index) = key;

        let account = self.account_for_txid_index[&key].clone();

        let utxos_for_account = self.utxos_for_account[&account].clone();

        let mut value = 0;
        for utxo in utxos_for_account.iter() {
            if utxo.tx_id == previous_tx_id && utxo.output_index == output_index {
                value = utxo.tx_out.value;
            }
        }

        let tx_report = TxReport {
            is_pending: pending,
            timestamp,
            tx_id,
            amount: -(value as i128),
            is_tx_in: true,
            index: tx_in_index,
        };

        if self.existe_tx_report_para_account(account.clone(), &tx_report) {
            return None;
        }

        let tx_report_by_accounts = self.tx_report_by_accounts.entry(account).or_default();
        tx_report_by_accounts.push(tx_report.clone());
        Some(tx_report)
    }

    // verificar si existe una tx report para un account
    pub fn existe_utxo_para_account(&self, account: String, new_utxo: &Utxo) -> bool {
        if !self.utxos_for_account.contains_key(&account) {
            return false;
        }

        let utxos_for_account = match self.utxos_for_account.get(&account) {
            Some(utxos_for_account) => utxos_for_account,
            None => return false,
        };

        for utxo in utxos_for_account {
            if utxo.tx_id == new_utxo.tx_id && utxo.output_index == new_utxo.output_index {
                return true;
            }
        }
        false
    }

    fn agregar_utxo(
        &mut self,
        current_account: String,
        tx_id: Uint256,
        output_index: u32,
        tx_out: &TxOut,
        tx: &Transaction,
    ) -> Utxo {
        let utxo = Utxo {
            tx_id,
            output_index,
            tx_out: tx_out.clone(),
            pk_script: tx_out.pk_script.clone(),
            tx: tx.clone(),
        };

        if !self.existe_utxo_para_account(current_account.clone(), &utxo) {
            let utxos_for_account = self
                .utxos_for_account
                .entry(current_account.clone())
                .or_insert(Vec::new());
            utxos_for_account.push(utxo.clone());

            self.account_for_txid_index
                .insert((tx_id, output_index), current_account);
        }
        utxo
    }

    fn eliminar_utxo(&mut self, previous_tx_id: Uint256, output_index: u32, key: (Uint256, u32)) {
        let account = self.account_for_txid_index[&key].clone();
        let utxos_for_account = self.utxos_for_account.entry(account).or_default();
        utxos_for_account
            .retain(|utxo| !(utxo.tx_id == previous_tx_id && utxo.output_index == output_index));
        self.account_for_txid_index.remove(&key);
    }

    pub fn validar_output(
        accounts: Vec<Account>,
        tx_out: &TxOut,
    ) -> Result<Account, NodoBitcoinError> {
        for account in accounts.iter() {
            if tx_out.is_user_account_output(account.clone().public_key) {
                return Ok(account.clone());
            }
        }
        Err(NodoBitcoinError::InvalidAccount)
    }

    pub fn update_from_blocks(
        &mut self,
        mut blocks: Vec<SerializedBlock>,
        accounts: Vec<Account>,
    ) -> Result<(), NodoBitcoinError> {
        // filtrar los bloques que ya estan en la base de datos por el timestamp del header
        blocks.retain(|x| x.header.time > self.last_timestamp);
        for block in blocks.iter() {
            let txs = block.txns.clone();
            for tx in txs.iter() {
                let tx_id = tx.txid()?;
                // recorro los output para identificar los que son mios
                for (output_index, tx_out) in tx.output.iter().enumerate() {
                    let current_account = match UTXOSet::validar_output(accounts.clone(), tx_out) {
                        Ok(account) => account,
                        Err(_) => continue,
                    };

                    let utxo = self.agregar_utxo(
                        current_account.public_key.clone(),
                        tx_id,
                        output_index as u32,
                        tx_out,
                        tx,
                    );

                    match self.agregar_tx_report_desde_out(
                        current_account.public_key.clone(),
                        utxo,
                        block.header.time,
                        false,
                    ) {
                        Some(tx_report) => {
                            self.eliminar_tx_report_pending(tx_report.clone());
                        }
                        None => continue,
                    };
                }

                for (tx_in_index, tx_in) in tx.input.iter().enumerate() {
                    let previous_tx_id = Uint256::from_be_bytes(tx_in.previous_output.hash);
                    let output_index = tx_in.previous_output.index;
                    let key = (previous_tx_id, output_index);

                    if self.validar_input(tx_in.clone()).is_ok() {
                        match self.agregar_tx_report_desde_in(
                            tx_id,
                            block.header.time,
                            false,
                            tx_in_index as u32,
                            key,
                        ) {
                            Some(tx_report) => {
                                self.eliminar_tx_report_pending(tx_report.clone());
                            }
                            None => continue,
                        }
                        self.eliminar_utxo(previous_tx_id, output_index, key);
                    }
                }
            }
            self.last_timestamp = block.header.time;
        }
        Ok(())
    }

    pub fn validar_input(&self, tx_in: TxIn) -> Result<String, NodoBitcoinError> {
        let previous_tx_id = Uint256::from_be_bytes(tx_in.previous_output.hash);
        let output_index = tx_in.previous_output.index;
        let key = (previous_tx_id, output_index);
        if self.account_for_txid_index.contains_key(&key) {
            return Ok(self.account_for_txid_index[&key].clone());
        }
        Err(NodoBitcoinError::InvalidAccount)
    }

    pub fn get_available(&self, account: String) -> Result<u64, NodoBitcoinError> {
        let mut balance = 0;
        if let Some(utxos) = self.utxos_for_account.get(&account) {
            for utxo in utxos.iter() {
                balance += utxo.tx_out.value;
            }
        }
        Ok(balance)
    }

    pub fn get_pending(&self, account: String) -> Result<i128, NodoBitcoinError> {
        let mut balance = 0;
        if let Some(tx_reports) = self.tx_report_pending_by_accounts.get(&account) {
            for tx_report in tx_reports.iter() {
                balance += tx_report.amount;
            }
        }
        Ok(balance)
    }

    /*
    pub utxos_for_account: HashMap<String, Vec<Utxo>>,
    pub account_for_txid_index: HashMap<(Uint256, u32), String>,
    pub tx_report_by_accounts: HashMap<String, Vec<TxReport>>,
    pub tx_report_pending_by_accounts: HashMap<String, Vec<TxReport>>,
     */

    pub fn save_utxos_for_account(
        timestamp: u32,
        hashmap: HashMap<String, Vec<Utxo>>,
        file: &mut dyn Write,
    ) -> Result<(), NodoBitcoinError> {
        // guardar el timestamp
        match file.write_all(&timestamp.to_be_bytes()) {
            Ok(_) => {}
            Err(_) => {
                return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
            }
        };
        for (key, values) in hashmap {
            save_encoded_len_bytes(file, key)?;
            let len = values.len();
            match file.write_all(&len.to_ne_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                }
            };
            for utxo in values {
                utxo.save(file)?;
            }
        }
        Ok(())
    }

    pub fn load_utxos_for_account_and_timestamp(
        buffer: Vec<u8>,
    ) -> Result<(u32, HashMap<String, Vec<Utxo>>), NodoBitcoinError> {
        let sizeof_usize: usize = mem::size_of::<usize>();
        let mut hashmap: HashMap<String, Vec<Utxo>> = HashMap::new();
        let mut offset = 0;
        let buffer_len = buffer.len();
        if buffer_len < 4 {
            return Ok((0_u32, hashmap));
        }
        let binding = buffer.clone();
        let bytes = binding.as_slice();
        let timestamp = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;
        while offset < buffer_len {
            let (key, new_offset) = read_decoded_string_offset(buffer.clone(), offset as u64)?;
            offset = new_offset as usize;
            let len_hashmap =
                usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
            offset += sizeof_usize;
            for _ in 0..len_hashmap {
                let len_utxo =
                    usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
                offset += sizeof_usize;
                let utxo_bytes = &bytes[offset..offset + len_utxo];
                let utxo = Utxo::deserialize(utxo_bytes)?;
                hashmap.entry(key.clone()).or_default().push(utxo);
                offset += len_utxo;
            }
        }
        Ok((timestamp, hashmap))
    }

    pub fn save_account_for_txid_index(
        hashmap: HashMap<(Uint256, u32), String>,
        file: &mut dyn Write,
    ) -> Result<(), NodoBitcoinError> {
        for (key, values) in hashmap {
            let key_uint256 = key.0;
            match file.write_all(&key_uint256.get_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                }
            };
            let key_u32 = key.1;
            match file.write_all(&key_u32.to_ne_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                }
            };
            save_encoded_len_bytes(file, values)?;
        }
        Ok(())
    }

    pub fn load_account_for_txid_index(
        buffer: Vec<u8>,
    ) -> Result<HashMap<(Uint256, u32), String>, NodoBitcoinError> {
        let mut hashmap: HashMap<(Uint256, u32), String> = HashMap::new();
        let mut offset = 0;
        let buffer_len = buffer.len();
        if buffer_len < 4 {
            return Ok(hashmap);
        }
        let binding = buffer.clone();
        let bytes = binding.as_slice();
        while offset < buffer_len {
            let tx_id = Uint256::from_be_bytes(bytes[offset..offset + 32].try_into().unwrap());
            offset += 32;
            let output_index = u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap());
            offset += 4;
            let key = (tx_id, output_index);
            let (value, new_offset) = read_decoded_string_offset(buffer.clone(), offset as u64)?;
            offset = new_offset as usize;
            hashmap.insert(key, value);
        }
        Ok(hashmap)
    }

    pub fn save_tx_report_by_accounts(
        hashmap: HashMap<String, Vec<TxReport>>,
        file: &mut dyn Write,
    ) -> Result<(), NodoBitcoinError> {
        for (key, values) in hashmap {
            save_encoded_len_bytes(file, key)?;
            let len = values.len();
            match file.write_all(&len.to_ne_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
                }
            };
            for tx_report in values {
                tx_report.save(file)?;
            }
        }
        Ok(())
    }

    pub fn load_tx_report_by_accounts(
        buffer: Vec<u8>,
    ) -> Result<HashMap<String, Vec<TxReport>>, NodoBitcoinError> {
        let sizeof_usize: usize = mem::size_of::<usize>();
        let mut hashmap: HashMap<String, Vec<TxReport>> = HashMap::new();
        let mut offset = 0;
        let buffer_len = buffer.len();
        if buffer_len < 4 {
            return Ok(hashmap);
        }
        let binding = buffer.clone();
        let bytes = binding.as_slice();
        while offset < buffer_len {
            let (key, new_offset) = read_decoded_string_offset(buffer.clone(), offset as u64)?;
            offset = new_offset as usize;
            let len_hashmap =
                usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
            offset += sizeof_usize;
            for _ in 0..len_hashmap {
                let len_tx_report =
                    usize::from_ne_bytes(bytes[offset..offset + sizeof_usize].try_into().unwrap());
                offset += sizeof_usize;
                let tx_report_bytes = &bytes[offset..offset + len_tx_report];
                let tx_report = TxReport::deserialize(tx_report_bytes)?;
                hashmap.entry(key.clone()).or_default().push(tx_report);
                offset += len_tx_report;
            }
        }
        Ok(hashmap)
    }

    pub fn save(&self) -> Result<(), NodoBitcoinError> {
        let file_utxo_for_account =
            File::create(UTXO_FOR_ACCOUNT_FILENAME).expect("No se pudo crear el archivo");
        let file_account_for_txid =
            File::create(ACCOUNT_FOR_TXID_INDEX_FILENAME).expect("No se pudo crear el archivo");
        let file_tx_report_by_account =
            File::create(TX_REPORT_BY_ACCOUNT_FILENAME).expect("No se pudo crear el archivo");
        let file_tx_report_pending_by_account = File::create(TX_REPORT_PENDING_BY_ACCOUNT_FILENAME)
            .expect("No se pudo crear el archivo");
        Self::save_utxos_for_account(
            self.last_timestamp,
            self.utxos_for_account.clone(),
            &mut &file_utxo_for_account,
        )?;
        Self::save_account_for_txid_index(
            self.account_for_txid_index.clone(),
            &mut &file_account_for_txid,
        )?;
        Self::save_tx_report_by_accounts(
            self.tx_report_by_accounts.clone(),
            &mut &file_tx_report_by_account,
        )?;
        Self::save_tx_report_by_accounts(
            self.tx_report_pending_by_accounts.clone(),
            &mut &file_tx_report_pending_by_account,
        )?;

        Ok(())
    }

    fn load_bytes_from_file(filename: String) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut file_utxo_for_account = match File::open(filename) {
            Ok(file) => file,
            Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
        };
        let mut buffer = vec![];
        match file_utxo_for_account.read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        };
        Ok(buffer)
    }

    pub fn load(&mut self) -> Result<(), NodoBitcoinError> {
        let buffer_utxos_for_account =
            Self::load_bytes_from_file(UTXO_FOR_ACCOUNT_FILENAME.to_string())?;

        let (timestamp, hash_utxos_for_account) =
            Self::load_utxos_for_account_and_timestamp(buffer_utxos_for_account)?;
        self.last_timestamp = timestamp;
        self.utxos_for_account = hash_utxos_for_account;

        let buffer_account_for_txid =
            Self::load_bytes_from_file(ACCOUNT_FOR_TXID_INDEX_FILENAME.to_string())?;
        self.account_for_txid_index = Self::load_account_for_txid_index(buffer_account_for_txid)?;

        let buffer_tx_report_by_account =
            Self::load_bytes_from_file(TX_REPORT_BY_ACCOUNT_FILENAME.to_string())?;
        self.tx_report_by_accounts = Self::load_tx_report_by_accounts(buffer_tx_report_by_account)?;

        let buffer_tx_report_pending_by_account =
            Self::load_bytes_from_file(TX_REPORT_PENDING_BY_ACCOUNT_FILENAME.to_string())?;
        self.tx_report_pending_by_accounts =
            Self::load_tx_report_by_accounts(buffer_tx_report_pending_by_account)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::{
            blockheader::BlockHeader,
            transaction::{Outpoint, TxIn, TxOut},
        },
        common::decoder::{decode_base58, p2pkh_script_serialized},
        log::create_logger_actor,
    };

    use super::*;

    fn get_pk_script_from_account(account: String) -> Vec<u8> {
        let script = match decode_base58(account) {
            Ok(script) => script,
            Err(_) => return vec![],
        };

        match p2pkh_script_serialized(&script) {
            Ok(p2pkh_script) => return p2pkh_script,
            Err(_) => return vec![],
        };
    }

    #[test]
    fn test_build_from_transactions() {
        let private_key = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string();
        let public_key = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
        let account_name = "test".to_string();
        let account = Account::new(private_key, public_key.clone(), account_name);
        let p2pkh_script = get_pk_script_from_account(public_key.clone());

        let tx_out1 = TxOut {
            value: 100,
            pk_len: 0,
            pk_script: p2pkh_script.clone(),
            pk_len_bytes: 1,
        };
        let tx_out2 = TxOut {
            value: 200,
            pk_len: 0,
            pk_script: p2pkh_script,
            pk_len_bytes: 1,
        };

        let tx_in1 = TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 0,
            },
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0,
            script_bytes_amount: 1,
        };
        let tx_in2 = TxIn {
            previous_output: Outpoint {
                hash: [0; 32],
                index: 0,
            },
            script_bytes: 0,
            signature_script: vec![],
            sequence: 0,
            script_bytes_amount: 1,
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

        let block_header = BlockHeader {
            version: 1,
            previous_block_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5,
            ],
            merkle_root_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5,
            ],
            time: 123456789,
            n_bits: 123456789,
            nonce: 123456789,
        };

        let block = SerializedBlock {
            header: block_header,
            txns: transactions,
            txn_amount: 2,
        };

        let mut utxo_set = UTXOSet::new();
        let result = utxo_set.update_from_blocks(vec![block], vec![account]);
        assert!(result.is_ok());

        assert_eq!(utxo_set.utxos_for_account.len(), 1);
        assert!(utxo_set.utxos_for_account.contains_key(&public_key));
        let utxos_for_account = utxo_set.utxos_for_account.get(&public_key).unwrap();
        assert_eq!(utxos_for_account.len(), 2);
        assert!(utxos_for_account[0].tx_id == transaction1.txid().unwrap());
        assert!(utxos_for_account[1].tx_id == transaction2.txid().unwrap());

        let balance = utxo_set.get_available(public_key);
        assert!(balance.is_ok());
        assert_eq!(balance.unwrap(), 300);
    }

    #[test]
    fn test_build_from_transactions_for_spent_outputs() {
        let private_key = "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string();
        let public_key = "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string();
        let account_name = "test".to_string();
        let account = Account::new(private_key, public_key.clone(), account_name);

        let p2pkh_script = get_pk_script_from_account(public_key.clone());

        let mut utxo_set = UTXOSet::new();

        let transaction1 = Transaction {
            input: vec![],
            output: vec![TxOut {
                value: 5,
                pk_script: p2pkh_script,
                pk_len: 0,
                pk_len_bytes: 0,
            }],
            lock_time: 0,
            tx_in_count: 0,
            tx_out_count: 1,
            version: 1,
        };

        let transaction2 = Transaction {
            input: vec![TxIn {
                previous_output: Outpoint {
                    hash: transaction1.txid().unwrap().get_bytes(),
                    index: 0,
                },
                signature_script: vec![],
                script_bytes: 0,
                script_bytes_amount: 0,
                sequence: 0,
            }],
            output: vec![TxOut {
                value: 5,
                pk_script: vec![],
                pk_len: 0,
                pk_len_bytes: 0,
            }],
            lock_time: 0,
            tx_in_count: 1,
            tx_out_count: 1,
            version: 1,
        };

        let block_header = BlockHeader {
            version: 1,
            previous_block_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5,
            ],
            merkle_root_hash: [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1,
                2, 3, 4, 5,
            ],
            time: 123456789,
            n_bits: 123456789,
            nonce: 123456789,
        };

        let block = SerializedBlock {
            header: block_header,
            txns: vec![transaction1.clone(), transaction2.clone()],
            txn_amount: 2,
        };

        utxo_set
            .update_from_blocks(vec![block], vec![account.clone()])
            .unwrap();
        assert_eq!(utxo_set.utxos_for_account[&account.public_key].len(), 0);
    }

    #[test]
    fn test_serialize_deserialize_utxo() {
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

        let tx = Transaction::deserialize(&tx_bytes[..]);
        assert!(tx.is_ok());
        let tx = tx.unwrap();

        let tx_id = tx.txid();
        assert!(tx_id.is_ok());
        let tx_id = tx_id.unwrap();

        let tx_out = tx.output[0].clone();
        let output_index: u32 = 0;

        let utxo = Utxo {
            tx_id,
            output_index,
            tx_out: tx_out.clone(),
            pk_script: tx_out.pk_script.clone(),
            tx,
        };

        let serialized_utxo = utxo.serialize();
        assert!(serialized_utxo.is_ok());

        let deserialized_utxo = Utxo::deserialize(&serialized_utxo.unwrap());
        assert!(deserialized_utxo.is_ok());

        let deserialized_utxo = deserialized_utxo.unwrap();
        assert_eq!(deserialized_utxo.tx_id, utxo.tx_id);
        assert_eq!(deserialized_utxo.output_index, utxo.output_index);
        assert_eq!(deserialized_utxo.tx_out, utxo.tx_out);
        assert_eq!(deserialized_utxo.pk_script, utxo.pk_script);
        assert_eq!(deserialized_utxo.tx, utxo.tx);
    }

    #[test]
    fn test_serialize_and_deserialize_txin_tx_report() {
        // armar un tx_report serelizarlo y deserealizarlo
        let tx_report = TxReport {
            is_pending: false,
            timestamp: 123456789,
            tx_id: Uint256::_from_u64(132456),
            amount: -159,
            is_tx_in: true,
            index: 0,
        };

        let serialized = tx_report.serialize();
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let deserialized = TxReport::deserialize(&serialized);
        assert!(deserialized.is_ok());
        let deserialized = deserialized.unwrap();

        assert_eq!(tx_report.is_pending, deserialized.is_pending);
        assert_eq!(tx_report.timestamp, deserialized.timestamp);
        assert_eq!(tx_report.tx_id, deserialized.tx_id);
        assert_eq!(tx_report.amount, deserialized.amount);
        assert_eq!(tx_report.is_tx_in, deserialized.is_tx_in);
        assert_eq!(tx_report.index, deserialized.index);
    }

    #[test]
    fn test_serialize_and_deserialize_txout_tx_report() {
        // armar un tx_report serelizarlo y deserealizarlo
        let tx_report = TxReport {
            is_pending: false,
            timestamp: 123456789,
            tx_id: Uint256::_from_u64(132456),
            amount: 6545,
            is_tx_in: false,
            index: 1,
        };

        let serialized = tx_report.serialize();
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let deserialized = TxReport::deserialize(&serialized);
        assert!(deserialized.is_ok());
        let deserialized = deserialized.unwrap();

        assert_eq!(tx_report.is_pending, deserialized.is_pending);
        assert_eq!(tx_report.timestamp, deserialized.timestamp);
        assert_eq!(tx_report.tx_id, deserialized.tx_id);
        assert_eq!(tx_report.amount, deserialized.amount);
        assert_eq!(tx_report.is_tx_in, deserialized.is_tx_in);
        assert_eq!(tx_report.index, deserialized.index);
    }

    #[test]
    fn test_save_load_hashmap_utxos() {
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

        let tx = Transaction::deserialize(&tx_bytes[..]);
        assert!(tx.is_ok());
        let tx = tx.unwrap();

        let tx_id = tx.txid();
        assert!(tx_id.is_ok());
        let tx_id = tx_id.unwrap();

        let tx_out = tx.output[0].clone();
        let output_index: u32 = 0;

        let utxo = Utxo {
            tx_id,
            output_index,
            tx_out: tx_out.clone(),
            pk_script: tx_out.pk_script.clone(),
            tx: tx.clone(),
        };

        let utxo2 = utxo.clone();

        let mut hashmap: HashMap<String, Vec<Utxo>> = HashMap::new();
        let account = "cuenta_test".to_string();

        hashmap.entry(account.clone()).or_default().push(utxo);
        hashmap.entry(account.clone()).or_default().push(utxo2);

        let mut file: Vec<u8> = Vec::new();
        let saved = UTXOSet::save_utxos_for_account(123456987, hashmap, &mut file);
        assert!(saved.is_ok());

        let loaded = UTXOSet::load_utxos_for_account_and_timestamp(file);
        assert!(loaded.is_ok());

        let (timestamp, loaded) = loaded.unwrap();
        assert_eq!(timestamp, 123456987);

        assert!(loaded.contains_key(&account));
        let utxos = loaded.get(&account).unwrap();
        assert_eq!(utxos.len(), 2);
        assert_eq!(utxos[0].tx_id, tx_id);
        assert_eq!(utxos[0].output_index, output_index);
        assert_eq!(utxos[0].tx_out, tx_out);
        assert_eq!(utxos[0].pk_script, tx_out.pk_script);
        assert_eq!(utxos[0].tx, tx);
        assert_eq!(utxos[1].tx_id, tx_id);
        assert_eq!(utxos[1].output_index, output_index);
        assert_eq!(utxos[1].tx_out, tx_out);
        assert_eq!(utxos[1].pk_script, tx_out.pk_script);
        assert_eq!(utxos[1].tx, tx);
    }

    #[test]
    fn test_save_load_hashmap_tx_index() {
        // crear 2 pares de claves (Uint256, u32) y guardarlos en un hashmap donde los valores sean String
        // y las claves sean los pares de claves (Uint256, u32)
        // guardar el hashmap en un archivo
        // cargar el hashmap del archivo
        // comprobar que el hashmap cargado es igual al hashmap original

        let key1_u256 = Uint256::_from_u64(123456);
        let key1_u32: u32 = 0;

        let key2_u256 = Uint256::_from_u64(654321);
        let key2_u32: u32 = 1;

        let value1 = "valor1".to_string();
        let value2 = "valor2".to_string();

        let mut hashmap: HashMap<(Uint256, u32), String> = HashMap::new();
        hashmap.insert((key1_u256, key1_u32), value1.clone());
        hashmap.insert((key2_u256, key2_u32), value2.clone());

        let mut file: Vec<u8> = Vec::new();
        let saved = UTXOSet::save_account_for_txid_index(hashmap, &mut file);
        assert!(saved.is_ok());

        let loaded = UTXOSet::load_account_for_txid_index(file);
        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();

        assert!(loaded.contains_key(&(key1_u256, key1_u32)));
        assert!(loaded.contains_key(&(key2_u256, key2_u32)));
        assert_eq!(loaded.get(&(key1_u256, key1_u32)).unwrap(), &value1);
        assert_eq!(loaded.get(&(key2_u256, key2_u32)).unwrap(), &value2);
    }

    #[test]
    fn test_save_load_hashmap_tx_report() {
        // crear un hashmap donde las claves sean String y los valores sean Vec<TxReport>
        // guardar el hashmap en un archivo
        // cargar el hashmap del archivo
        // comprobar que el hashmap cargado es igual al hashmap original

        let key1 = "key1".to_string();
        let key2 = "key2".to_string();

        let tx_report1 = TxReport {
            is_pending: false,
            timestamp: 123456789,
            tx_id: Uint256::_from_u64(132456),
            amount: -159,
            is_tx_in: true,
            index: 0,
        };
        let tx_report2 = TxReport {
            is_pending: false,
            timestamp: 123456789,
            tx_id: Uint256::_from_u64(132456),
            amount: 6545,
            is_tx_in: false,
            index: 1,
        };

        let tx_report3 = tx_report2.clone();

        let mut hashmap: HashMap<String, Vec<TxReport>> = HashMap::new();
        hashmap
            .entry(key1.clone())
            .or_default()
            .push(tx_report1.clone());
        hashmap
            .entry(key2.clone())
            .or_default()
            .push(tx_report2.clone());
        hashmap
            .entry(key2.clone())
            .or_default()
            .push(tx_report3.clone());

        let mut file: Vec<u8> = Vec::new();
        let saved = UTXOSet::save_tx_report_by_accounts(hashmap, &mut file);
        assert!(saved.is_ok());

        let loaded = UTXOSet::load_tx_report_by_accounts(file);
        assert!(loaded.is_ok());

        let loaded = loaded.unwrap();

        assert!(loaded.contains_key(&key1));
        assert!(loaded.contains_key(&key2));
        let tx_reports = loaded.get(&key1).unwrap();
        assert_eq!(tx_reports.len(), 1);
        assert_eq!(tx_reports[0].amount, tx_report1.amount);
        assert_eq!(tx_reports[0].is_pending, tx_report1.is_pending);
        assert_eq!(tx_reports[0].timestamp, tx_report1.timestamp);
        assert_eq!(tx_reports[0].tx_id, tx_report1.tx_id);
        assert_eq!(tx_reports[0].is_tx_in, tx_report1.is_tx_in);
        assert_eq!(tx_reports[0].index, tx_report1.index);

        let tx_reports = loaded.get(&key2).unwrap();
        assert_eq!(tx_reports.len(), 2);
        assert_eq!(tx_reports[0].amount, tx_report2.amount);
        assert_eq!(tx_reports[0].is_pending, tx_report2.is_pending);
        assert_eq!(tx_reports[0].timestamp, tx_report2.timestamp);
        assert_eq!(tx_reports[0].tx_id, tx_report2.tx_id);
        assert_eq!(tx_reports[0].is_tx_in, tx_report2.is_tx_in);
        assert_eq!(tx_reports[0].index, tx_report2.index);
        assert_eq!(tx_reports[1].amount, tx_report3.amount);
        assert_eq!(tx_reports[1].is_pending, tx_report3.is_pending);
        assert_eq!(tx_reports[1].timestamp, tx_report3.timestamp);
        assert_eq!(tx_reports[1].tx_id, tx_report3.tx_id);
        assert_eq!(tx_reports[1].is_tx_in, tx_report3.is_tx_in);
        assert_eq!(tx_reports[1].index, tx_report3.index);
    }
}
