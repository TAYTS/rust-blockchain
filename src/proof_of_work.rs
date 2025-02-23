use std::borrow::Borrow;

use data_encoding::HEXLOWER;
use num::{bigint::Sign, BigInt};

use crate::block::Block;

const MAX_NONCE: i64 = i64::MAX;

pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

impl ProofOfWork {
    pub fn new_proof_of_work(block: Block) -> Self {
        Self {
            block,
            target: BigInt::from(0),
        }
    }

    pub fn run(&self) -> (i64, String) {
        let mut nonce = 0;
        let mut hash = Vec::new();
        println!("Mining the block");
        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);
            hash = sha256::digest(data.as_slice()).into();
            let hash_int = BigInt::from_bytes_be(Sign::Plus, hash.as_slice());
            if hash_int.lt(self.target.borrow()) {
                println!("{}", HEXLOWER.encode(hash.as_slice()));
                break;
            } else {
                nonce += 1;
            }
        }
        println!();
        (nonce, HEXLOWER.encode(hash.as_slice()))
    }

    pub fn prepare_data(&self, nonce: i64) -> Vec<u8> {
        Vec::new()
    }
}
