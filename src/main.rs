mod block;
mod blockchain;
mod transaction;

use blockchain::Blockchain;
use transaction::Transaction;

fn main() {
    println!("=== Baseline Demo ===");
    let mut blkc = Blockchain::new_with_genesis_timestamp(1_700_000_000);
    println!("Genesis block created. Hash: {}...\n", &blkc.last_block().hash[..16]);
    blkc.add_block(vec![Transaction::new("Alice".to_string(), "Bob".to_string(), 50), Transaction::new("Bob".to_string(), "Carol".to_string(), 25)]);
    println!("Added Block #1 (2 Transactions).");

    blkc.add_block(vec![Transaction::new("Carol".to_string(), "Dave".to_string(), 10)]);
    println!("Added Block #2 (2 transactions).");

    println!("{}",blkc);
    match blkc.is_valid() {
        Ok(()) => println!("Chain validation: OK"), Err(e) => println!("Chain validation FAILED: {}", e)
    }

    println!("\n--- Tamper Detection Demo ---");
    let mut tampered = blkc.clone();
    tampered.chain[1].transactions[0].amount = 999;
    match tampered.is_valid(){
        Ok(()) => println!("(unexpected) tampered chain validated OK"), Err(e) => println!("Tampered Chain correctly rejected: {}", e)
    }
}