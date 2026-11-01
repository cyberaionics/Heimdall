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
            return Err("chain is empty — genesis block missing".to_string());
        }
        let genesis = &self.chain[0];
        if genesis.hash != genesis.calculate_hash() {
            return Err(format!("genesis block hash mismatch: stored={}, computed={}", genesis.hash, genesis.calculate_hash()));
        }
        for i in 1..self.chain.len() {
            let prev = &self.chain[i-1];
            let curr = &self.chain[i];
            if curr.previous_hash != prev.hash {
                return Err(format!("broken link at index {}: previous_hash={}, but block[{}].hash={}", curr.index, curr.previous_hash, prev.index, prev.hash));
            }
            if curr.hash != curr.calculate_hash() {
                return Err(format!("hash mismatch at index {}: stored={}, computed={}", curr.index, curr.hash, curr.calculate_hash()));
            }
        }
        for (tx_idx, tx) in curr.transactions.iter().enumerate(){
            if !tx.is_signed() {
                return Err(format!("unsigned transaction at block {} tx {}", curr.index, tx_idx));
            }
            if let Err(reason) = tx.verify() {
                return Err(format!("invalid signature at block {} tx {}: {}", curr.index, tx_idx, reason));
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
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}



// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block
    use crate::transaction::Transaction;
    use crate::wallet::Wallet;
    
    fn alice_wallet() -> Wallet {
        Wallet::from_seed([0x01; 32])
    }
    fn bob_wallet() -> Wallet {
        Wallet::from_seed([0x02; 32])
    }
    fn carol_wallet() -> Wallet {
        Wallet::from_seed([0x03; 32])
    }

    fn sample_txs() -> Vec<Transaction> {
        let mut tx1 = Transaction::new("Alice".to_string(), "Bob".to_string, 50);
        alice_wallet().sign_transaction(&mut tx1);
        let mut tx2 = Transaction::new("Bob".to_string(), "Carol".to_string(), 25);
        bob_wallet().sign_transaction(&mut tx2);
        vec![tx1, tx2];
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
        assert!(err.contains("hash mismatch at index 1"), "in-place tamper must be caught by the hash layer, got: {}", err);
    }
    #[test]
    fn tampered_block_hash_is_detected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        blkc.chain[1].hash = "deadbeef".repeat(8);
        let result = blkc.is_valid();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("hash mismatch at index 1"));
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
    #[test]
    fn signed_chain_validates_cleanly() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        blkc.add_block(sample_txs());
        assert!(blkc.is_valid().is_ok(), "a chain of correctly signed transactions must validate");
    }
    #[test]
    fn unsigned_transaction_is_rejected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        let prev = blkc.last_block().clone();
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        let mut block = Block::new(prev.index + 1, 1_700_000_001, vec![tx], prev.hash.clone());
        block.recompute_hash();
        blkc.chain.push(block);
        let result = blkc.is_valid();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsigned transaction"), "an unsigned transaction must be rejected");
    }
    #[test]
    fn tampered_signature_is_detected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        let mut chars = Vec<char> = blkc.chain[1].transactions[0].signature.chars().collect();
        chars[0] = if chars[0] == '0' {'1'} else {'0'};
        blkc.chain[1].transactions[0].signature = chars.into_iter().collect();
        blkc.chain[1].recompute_hash();
        let result = blkc.is_valid();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid signature"), "a forged signature must be detected after rehash, got: {}", err);
    }
    #[test]
    fn tampered_transaction_content_is_detected_by_signature() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        blkc.chain[1].transactions[0].amount = 999;
        blkc.chain[1].recompute_hash();
        let result = blkc.is_valid();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid signature"), "content changed after signing must fail the signature check");
    }
    #[test]
    fn wrong_public_key_is_detected() {
        let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
        blkc.add_block(sample_txs());
        blkc.chain[1].transaction[0].public_key = carol_wallet().public_key_hex();
        blkc.chain[1].recompute_hash();
        let result = blkc.is_valid();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid signature"), "a signature must not verify under a different public key");
    }
    #[test]
    fn wallet_deterministic_from_seed() {
        assert_eq!(alice_wallet().public_key_hex(), alice_wallet().public_key_hex(), "identical seeds must yield identical public keys");
        assert_ne!(alice_wallet().public_key_hex(), bob_wallet().public_key_hex(), "different seeds must yield different public keys");
    }
    #[test]
    #[ignore = "timing measurement; run with --ignored --nocapture (and --release for realistic numbers)"]
    fn measure_signature_verification_cost() {
        use std::time::Instant;
        let n_txs: usize = 1000;
        let mut txs = Vec::with_capacity(n_txs);
        for i in 0..n_txs {
            let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), i as u64);
            wallet.sign_transaction(&mut tx)
        }
    }
}