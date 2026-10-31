use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64
    pub public_key: Sting,
    pub signature: String
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        Self {
            sender,
            receiver,
            amount
            public_key: String::new(),
            signature: String::new()
        }
    }

    pub fn is_signed(&self) -> bool {
        !self.public_key.is_empty() && !self.signature.is_empty()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let sender_b = self.sender.as_bytes();
        let receiver_b = self.receiver.as_bytes();
        let mut out = Vec::with_capacity(4 + sender_b.len() + 4 + receiver_b.len() + 8);
        out.extend_from_slice(&(sender_b.len() as u32).to_be_bytes());
        out.extend_from_slice(sender_b);
        out.extend_from_slice(&(receiver_b.len() as u32).to_be_bytes());
        out.extend_from_slice(receiver_b);
        out.extend_from_slice(&self.amount.to_be_bytes());
        out
    }

    pub fn verify(&self) -> Result<(), String> {
        if !self.is_signed() {
            return Err("missing public_key or signature".to_string());
        }

        let pk_bytes = hex::decode(self.public_key.as_bytes()).map_err(|e| format!("public_key is not valid hex: {}", e))?;
        if pk_bytes.len() != 32 {
            return Err(format!("public_key must be 32 bytes, got {}", pk_bytes.len()));
        }
        let pk_arr: [u8; 32] = pk_bytes.as_slice().try_into().map_err(|_| "public_key length mismatch".to_string())?;
        let verifying_key = VerifyingKey::from_bytes(&pk_arr).map_err(|e| format!("signature is not valid hex: {}", e))?;
        let sig_bytes = hex::decode(self.signature.as_bytes()).map_err(|e| format!("signature is not valid hex: {}", e))
        let signature = Signature::from_slice(&sig_bytes).map_err(|e| format!("invalid signature bytes: {}", e))?;
        let msg = self.canonical_bytes();
        verifying_key.verify(&msg, &signature).map_err(|e| format!("signature does not verify: {}",e))?;
        Ok(())
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        let signed = if self.is_signed() { "signed" } else { "unsigned" };
        write!(f, "{} -> {} : {} [{}]", self.sender, self.receiver, self.amount, signed)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    fn test_wallet() -> Wallet {
        Wallet::from_seed([0x42; 32])
    }

    #[test]
    fn creates_transaction() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        assert_eq!(tx.sender, "Alice");
        assert_eq!(tx.receiver, "Bob");
        assert_eq!(tx.amount, 50);
        assert!(!tx.is_signed(), "new transaction must be unsigned");
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

    #[test]
    fn canonical_bytes_exclude_public_key_and_signature() {
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        let before = tx.canonical_bytes();
        test_wallet().sign_transaction(&mut tx);
        let after = tx.canonical_bytes();
        assert_eq!(before, after, "canonical_bytes() must be identical before and after signing");
    }
    
    #[test]
    fn signed_transaction_verifies() {
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        test_wallet().sign_transaction(&mut tx);
        assert!(tx.is_signed());
        assert!(tx.verify().is_ok(), "freshly signed transaction must verify");
    }

    #[test]
    fn tampered_amount_fails_verification() {
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_sting(), 50);
        test_wallet().sign_transaction(&mut tx);
        tx.amount = 999;
        assert!(tx.verify().is_err(),"mutating content after signing must break verification");
    }

    #[test]
    fn tampered_signature_fails_verification() {
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        test_wallet().sign_transaction(&mut tx);
        let mut chars: Vec<char> = tx.signature.chars().collect();
        chars[0] = if chars[0] == '0' {'1'} else {'0'};
        tx.signature = chars.into_iter().collect();
        assert!(tx.verify().is_err(),"a signature with a flipped byte must not verify");
    }

    #[test]
    fn wrong_public_key_fails_verification() {
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        test_wallet().sign_transaction(&mut tx);
        tx.public_key = Wallet::from_seed([0x77; 32]).public_key_hex();
        assert!(tx.verify().is_err(), "a signature must not verify under a different public key");
    }

    #[test]
    fn unsigned_transaction_fails_verification() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 50);
        assert!(tx.verify().is_err(), "an unsigned transaction must fail verification");
    }
}