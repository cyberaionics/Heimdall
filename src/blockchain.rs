use crate::block::Block;
use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Block::genesis(current_unix_seconds());
        Self {
            chain: vec![genesis]
        }
    }

    pub fn new_with_genesis_timestamp(timestamp: i64) -> Self {
        let genesis = Block::genesis(timestamp);
        Self {
            chain: vec![genesis]
        }
    }

    pub fn last_block(&self) -> &Block {
        self.chain.last().expect("chain always contains at least the genesis block")
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let prev = self.last_block();
        let index = prev.index + 1;
        let previous_hash = prev.hash.clone();
        let mut block = Block::new(index, current_unix_seconds(), transactions, previous_hash);
        block.recompute_hash();
        self.chain.push(block);
    }

    pub fn is_valid(&self) -> Result<(), String> {
        if self.chain.is_empty() {
            return Err("Chain is empty - genesis block missing".to_string());
        }
        let genesis = &self.chain[0];
        if genesis.hash != genesis.calculate_hash() {
            return Err(format!("genesis block hash mismatch: Stored={}, computed={}", genesis.hash, genesis.calculate_hash()));
        }
        for i in 1..self.chain.len() {
            let prev = &self.chain[i-1];
            let curr = &self.chain[i];
            if curr.previous_hash != prev.hash {
                return Err(format!("Broken link at Index {}: Previous Hash = {}, but block[{}].hash = {}", curr.index, curr.previous_hash, prev.index, prev.hash));
            }
            if curr.hash != curr.calculate_hash() {
                return Err(format!("hash mismatch at index {}: stored = {}, computed={}", curr.index, curr.hash, curr.calculate_hash()));
            }
        }
        Ok(())
    } 
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

fn current_unix_seconds() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

impl std::fmt::Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Blockchain ({} blocks):", self.chain.len())?;
        for block in &self.chain {
            let _ = write!(f, "{}", block);
        }
        Ok(())
    }
}



// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    fn sample_txs() -> Vec<Transaction> {
        vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 50), Transaction::new("Bob".to_string(), "Carol".to_string(), 25)]
    }
    #[test]
    fn new_chain_has_genesis_only() {
        let blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        assert_eq!(blkc.chain.len(), 1);
        assert_eq!(blkc.chain[0].index, 0);
        assert!(blkc.is_valid().is_ok());
    }
    #[test]
    fn add_block_appends_a_valid_block() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        assert_eq!(blkc.chain.len(), 2);
        assert_eq!(blkc.chain[1].index, 1);
        assert!(blkc.is_valid().is_ok());
    }
    #[test]
    fn adding_multiple_blocks_keeps_chain_valid() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        for _ in 0..5 {
            blkc.add_block(sample_txs());
        }
        assert_eq!(blkc.chain.len(), 6);
        assert!(blkc.is_valid().is_ok());
    }
    #[test]
    fn tampered_transaction_is_detected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        blkc.chain[1].transactions[0].amount = 999;
        let result = blkc.is_valid();
        assert!(result.is_err(), "tampered transaction must be detected by validation");
        let err = result.unwrap_err();
        assert!(err.contains("Hash Mismatch at index 1"), "error message must point at the tampered block, got: {}", err);
    }
    #[test]
    fn tampered_block_hash_is_detected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        let result = blkc.is_valid();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Hash mismatch at index 1"));
    }
    #[test]
    fn broken_link_is_detected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        blkc.add_block(sample_txs());
        blkc.chain[2].previous_hash = "0".repeat(64);
        let result = blkc.is_valid();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("broken link at index 2"), "broken chain link must be detected, got: {}", err);
    }
    #[test]
    fn tampered_genesis_is_detected() {
        let blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        let mut tampered = blkc.clone();
        tampered.chain[0].timestamp = 1;
        let result = tampered.is_valid();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("genesis block hash mismatch"));
    }
}