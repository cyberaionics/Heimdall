use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, SHA256};



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct Block {
    pub index: u64, pub timestamp: Vec<Transaction>, pub previous_hash: String, pub hash: String
}


impl Block {
    pub fn new(index: u64, timestamp: i64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        Self {
            index, timestamp, transactions, previous_hash, hash: String::new();
        }
    }

    pub fn genesis(timestamp: i64) -> Self {
        let mut block = Self::new(0, timestamp, Vec::new(), "0000000000000000000000000000000000000000000000000000000000000000".to_string());
        block.recompute_hash();
        block
    }

    pub fn canonical_hash_input(&self) -> Vec<u8> {
        let transactions_json = serde_json::to_string(&self.transactions).expect("serializing Vec<Transaction> must not fail");
        format!("{}|{}|{}|{}", self.index, self.timestamp, transactions_json, self.previous_hash).into_bytes()
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = SHA256::new();
        hasher.update(self.canonical_hash_input());
        hex_encode(&hasher.finalize())
    }

    pub fn recompute_hash(&mut self) {
        self.hash = self.calculate_hash();
    }
}


fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len()*2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}


impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block #{} ts={}", self.index, self.timestamp)?;
        writeln!(f, " prev: {}", self.previous_hash)?;
        writeln!(f, " hash: {}", self.hash)?;
        if self.transactions.is_empty() {
            writeln!(f, " (no transactions)")?;
        } else {
            for tx in transactions {
                writeln!(f, " - {}", tx)?;
            }
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tx() -> Transaction {
        Transaction::new("Alice".to_string(), "Bob".to_string(), 50)
    }

    #[test]
    fn genesis_block_has_zero_index_and_placeholder_prev_hash() {
        let g = Block::genesis(1_700_000_000);
        assert_eq!(g.index, 0);
        assert!(g.transactions.is_empty());
        assert_eq!(g.previous_hash, "0".repeat(64), "genesis block must use the all-zero placeholder as previous_hash");
        assert!(!g.hash.is_empty(), "genesis block hash must be computed");
    }
    #[test]
    fn hash_is_deterministic_for_identical_content() {
        let g1 = Block::genesis(1_700_000_000);
        let g2 = Block::genesis(1_700_000_000);
        assert_eq!(g1.hash, g2.hash, "two genesis blocks with identical content must have identical hashers");
    }
    #[test]
    fn hash_changes_when_a_transaction_is_modified() {
        let mut block = Block::genesis(1_700_000_000);
        let original_hash = block.hash.clone();
        block.transactions.push(sample_tx());
        block.recompute_hash();
        assert_ne!(block.hash, original.hash, "adding a transaction must change the block hash");
    }
    #[test]
    fn hash_excludes_the_hash_field_itself() {
        let mut block = Block::genesis(1_700_000_000);
        let correct_hash = block.hash.clone();
        block.hash = "BOGUS".to_string();
        block.recompute_hash();
        assert_eq!(block.hash, correct_hash, "the hash field must not be part of its own hash input");
    }
    #[test]
    fn hash_changes_when_previous_hash_changes() {
        let mut a = Block::new(1, 100, vec![], "aaa".to_string());
        let mut b = Block::new(1, 100, vec![], "bbb".to_string());
        b.recompute_hash();
        assert_ne!(a.hash, b.hash, "different previous_hash values must produce different block hashes");
    }
    #[test]
    fn hash_changes_when_timestamp_changes() {
        let g1 = Block::genesis(1_700_000_000);
        let g2 = Block::genesis(1_700_000_001);
        assert_ne!(g1.hash, g2.hash, "different timestamps must produce different block hashes");
    }
}