use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    mem,
};

use crate::errores::NodoBitcoinError;

pub struct Account {
    pub secret_key: String,
    pub public_key: String,
    pub wallet_name: String,
}

const ACCOUNT_FILENAME: &str = "accounts.dat";

impl Account {
    pub fn new(secret_key: String, public_key: String, wallet_name: String) -> Account {
        Account {
            secret_key,
            public_key,
            wallet_name,
        }
    }

    pub fn save(&self) -> Result<(), NodoBitcoinError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(ACCOUNT_FILENAME)
            .expect("No se pudo abrir el archivo");

        self.save_attributes(&mut file)
    }

    fn save_attributes(&self, file: &mut dyn Write) -> Result<(), NodoBitcoinError> {
        Account::save_len_bytes(file, self.secret_key.clone())?;
        Account::save_len_bytes(file, self.public_key.clone())?;
        Account::save_len_bytes(file, self.wallet_name.clone())
    }

    fn save_len_bytes(file: &mut dyn Write, data: String) -> Result<(), NodoBitcoinError> {
        let encoded = bs58::encode(data.as_bytes()).into_string();
        let len = encoded.len();
        match file.write_all(&len.to_ne_bytes()) {
            Ok(_) => {}
            Err(_) => {
                return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
            }
        };
        match file.write_all(&encoded.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                return Err(NodoBitcoinError::NoSePuedeEscribirLosBytes);
            }
        };
        Ok(())
    }

    pub fn get_all_accounts() -> Result<Vec<Account>, NodoBitcoinError> {
        let mut file = match File::open(ACCOUNT_FILENAME) {
            Ok(file) => file,
            Err(_) => return Err(NodoBitcoinError::NoExisteArchivo),
        };
        // leer todos los bytes del archivo
        let mut buffer = vec![];
        match file.read_to_end(&mut buffer) {
            Ok(_) => {}
            Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        };
        let accounts = Self::parse_accounts(buffer)?;
        Ok(accounts)
    }

    fn parse_accounts(buffer: Vec<u8>) -> Result<Vec<Account>, NodoBitcoinError> {
        let mut todas = vec![];
        let mut offset = 0;
        let buffer_len = buffer.len() as u64;
        while offset < buffer_len {
            let mut account_bytes = vec![];
            for _ in 0..3 {
                let (value, new_offset) = Self::leer_account(buffer.clone(), offset)?;
                account_bytes.push(value);
                offset = new_offset;
            }
            let mut account = Account::new(
                account_bytes[0].clone(),
                account_bytes[1].clone(),
                account_bytes[2].clone(),
            );
            todas.push(account);
        }
        Ok(todas)
    }

    fn leer_account(buffer: Vec<u8>, offset: u64) -> Result<(String, u64), NodoBitcoinError> {
        let sizeof_usize = mem::size_of::<usize>() as u64;
        let len_bytes: [u8; 8] = match Self::leer_bytes(buffer.clone(), offset, sizeof_usize)?
            .as_slice()
            .try_into()
        {
            Ok(bytes) => bytes,
            Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        };
        let len_account = usize::from_ne_bytes(len_bytes);
        let account_bytes = Self::leer_bytes(buffer, offset + sizeof_usize, len_account as u64)?;
        let account = String::from_utf8(account_bytes);
        if account.is_err() {
            return Err(NodoBitcoinError::NoSePuedeLeerLosBytes);
        }
        let account = account.unwrap();

        // Decodificar el string codificado
        let decoded = match bs58::decode(&account).into_vec() {
            Ok(bytes) => bytes,
            Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        };
        let decoded_string = match String::from_utf8(decoded) {
            Ok(string) => string,
            Err(_) => return Err(NodoBitcoinError::NoSePuedeLeerLosBytes),
        };

        Ok((decoded_string, offset + sizeof_usize + len_account as u64))
    }

    fn leer_bytes(buffer: Vec<u8>, offset: u64, length: u64) -> Result<Vec<u8>, NodoBitcoinError> {
        let mut bytes = vec![0; length as usize];
        for i in 0..length {
            bytes[i as usize] = buffer[(offset + i) as usize];
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::user::Account;

    #[test]
    fn test_one_account_save_read() {
        let mut account = Account::new(
            "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string(),
            "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string(),
            "wallet1".to_string(),
        );

        let mut mock_write = vec![];

        let save = account.save_attributes(&mut mock_write);
        assert!(save.is_ok());

        let read = Account::parse_accounts(mock_write);
        assert!(read.is_ok());
        let accounts = read.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].secret_key, account.secret_key);
        assert_eq!(accounts[0].public_key, account.public_key);
        assert_eq!(accounts[0].wallet_name, account.wallet_name);
    }

    #[test]
    fn test_two_account_save_read() {
        let mut account1 = Account::new(
            "cRJzHMCgDLsvttTH8R8t6LLcZgMDs1WtgwQXxk8bFFk7E2AJp1tw".to_string(),
            "mnJvq7mbGiPNNhUne4FAqq27Q8xZrAsVun".to_string(),
            "wallet2".to_string(),
        );

        let mut mock_write = vec![];

        let save = account1.save_attributes(&mut mock_write);
        assert!(save.is_ok());

        let read = Account::parse_accounts(mock_write.clone());
        assert!(read.is_ok());
        let accounts = read.unwrap();

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].secret_key, account1.secret_key);
        assert_eq!(accounts[0].public_key, account1.public_key);
        assert_eq!(accounts[0].wallet_name, account1.wallet_name);

        let mut account2 = Account::new(
            "cU7dbzeBRgMEZ5BUst2CFydGRm9gt8uQbNoojWPRRuHb2xk5q5h2".to_string(),
            "mtm4vS3WH7pg13pjFEmqGq2TSPDcUN6k7a".to_string(),
            "wallet1".to_string(),
        );

        let save = account2.save_attributes(&mut mock_write);
        assert!(save.is_ok());

        let read = Account::parse_accounts(mock_write);
        assert!(read.is_ok());
        let accounts = read.unwrap();

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].secret_key, account1.secret_key);
        assert_eq!(accounts[0].public_key, account1.public_key);
        assert_eq!(accounts[0].wallet_name, account1.wallet_name);
        assert_eq!(accounts[1].secret_key, account2.secret_key);
        assert_eq!(accounts[1].public_key, account2.public_key);
        assert_eq!(accounts[1].wallet_name, account2.wallet_name);
    }
}
