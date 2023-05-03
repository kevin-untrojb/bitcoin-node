mod blockchain {
    pub mod block;
    pub mod transaction;
    pub mod header;
}

use blockchain::block::SerializedBlock;
use blockchain::transaction::Transaction;
use blockchain::header::BlockHeader;
fn main() {
    println!("Hello, world!");
}
