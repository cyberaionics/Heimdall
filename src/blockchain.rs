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
            write!(f, "{}", block);
        }
        Ok(())
    }
}