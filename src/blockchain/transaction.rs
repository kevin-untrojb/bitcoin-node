/// A struct representing a Bitcoin transaction
/// ### Bitcoin Core References
/// https://developer.bitcoin.org/reference/transactions.html
///
/// # Fields
///
/// * id - The unique identifier of the transaction.
/// * version - The version number of the transaction.
/// * input - The vector of input transactions for the transaction.
/// * output - The vector of output transactions for the transaction.
/// * lock_time - The lock time for the transaction.
struct Transaction {
    id: usize,
    version: i32,
    input: Vec<TxIn>,
    output: Vec<TxOut>,
    lock_time: u64,
}

/// A struct representing an input transaction for a Bitcoin transaction
///
/// # Fields
///
/// * id - The unique identifier of the input transaction.
/// * previous_output - The outpoint from the previous transaction that this input is spending.
/// * script_bytes - The number of bytes in the signature script.
/// * signature_script - The signature script for the input.
/// * sequence - The sequence number for the input.
struct TxIn {
    id: usize,
    previous_output: Outpoint,
    script_bytes: usize,
    signature_script: String,
    sequence: u32,
}

/// A struct representing an outpoint from a previous transaction
///
/// # Fields
///
/// * id - The unique identifier of the outpoint.
/// * hash - The transaction hash of the previous transaction.
/// * index - The index of the output in the previous transaction.
struct Outpoint {
    id: usize,
    hash: String,
    index: u32,
}

/// A struct representing an output transaction for a Bitcoin transaction
///
/// # Fields
///
/// * id - The unique identifier of the output transaction.
/// * value - The value of the output in satoshis.
/// * pk_script - The public key script for the output.
struct TxOut {
    id: usize,
    value: f64,
    pk_script: String,
}

// todo add traits to handle the functionalities
