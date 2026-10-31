use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64
}

impl Transaction (
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        Self {
            sender,
            receiver,
            amount
        }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!("{}|{}|{}", self.sender, self.receiver, self.amount).info_bytes()
    }
)

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f, "{} -> {} : {}", self.sender, self.receiver, self.amount)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_transaction() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        assert_eq!(tx.sender, "Alice");
        assert_eq!(tx.receiver, "Bob");
        assert_eq!(tx.amount, 50);
    }

    #[test]
    fn transaction_equality_is_value_based() {
        let a = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        let b = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        let c = Transaction::new("Alice".to_string(), "Bob".to_string(), 51);

        assert_eq!(a, b, "two transaction with identical fields must be equal");
        assert_ne!(a, c, "differing amount must break equality");
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        let bytes_a = tx.canonical_bytes();
        let bytes_b = tx.canonical_bytes();
        assert_eq!(bytes_a, bytes_b, "canonical_bytes() must be deterministic for the same transaction");
    }

    #[test]
    fn canonical_bytes_differ_when_amount_differs() {
        let a = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        let b = Transaction::new("Alice".to_string(), "Bob".to_string(), 51);
        assert_ne!(a.canonical_bytes(), b.canonical_bytes(), "different amounts must produce different canonical bytes");
    }
}