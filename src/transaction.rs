use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

impl Transaction {
    pub fn get_id(&self) -> &[u8] {
        self.id.as_slice()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TXInput {
    txid: Vec<u8>,
    vout: i64,
    signature: Vec<u8>,
    pub_key: Vec<u8>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct TXOutput {
    value: i32,
    pub_key_hash: Vec<u8>,
}
