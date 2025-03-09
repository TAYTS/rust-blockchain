use std::{collections::HashMap, sync::RwLock};

use data_encoding::HEXLOWER;

use crate::transaction::Transaction;

pub struct MemoryPool {
    inner: RwLock<HashMap<String, Transaction>>,
}

impl MemoryPool {
    pub fn new() -> Self {
        MemoryPool {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn contains(&self, txid_hex: &str) -> bool {
        self.inner.read().unwrap().contains_key(txid_hex)
    }

    pub fn add(&self, tx: Transaction) {
        let txid_hex = HEXLOWER.encode(tx.get_id());
        self.inner.write().unwrap().insert(txid_hex, tx);
    }

    pub fn get(&self, txid_hex: &str) -> Option<Transaction> {
        self.inner.read().unwrap().get(txid_hex).cloned()
    }

    pub fn remove(&self, txid_hex: &str) {
        self.inner.write().unwrap().remove(txid_hex);
    }

    pub fn get_all(&self) -> Vec<Transaction> {
        let inner = self.inner.read().unwrap();
        let mut output = Vec::with_capacity(inner.len());
        for tx in inner.values() {
            output.push(tx.clone());
        }
        output
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

pub struct BlockInTransit {
    inner: RwLock<Vec<Vec<u8>>>,
}

impl BlockInTransit {
    pub fn new() -> BlockInTransit {
        BlockInTransit {
            inner: RwLock::new(Vec::new()),
        }
    }

    pub fn add_blocks(&self, blocks: Vec<Vec<u8>>) {
        self.inner.write().unwrap().extend(blocks);
    }

    pub fn first(&self) -> Option<Vec<u8>> {
        self.inner.read().unwrap().first().cloned()
    }

    pub fn remove(&self, block_hash: &[u8]) {
        let mut inner = self.inner.write().unwrap();
        if let Some(index) = inner.iter().position(|x| x.eq(block_hash)) {
            inner.remove(index);
        }
    }

    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}
