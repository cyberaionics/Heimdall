# heimdall — Architecture

> **Purpose:** System components, data models, interfaces, invariants,
> and interactions. This is the *what* and *how* document. *Why* lives
> in `planning.md`; *current state* lives in `context.md`.

## 1. Overview

`heimdall` is a single-process, single-writer, in-memory blockchain.
There is no networking, no consensus, no persistence, and no signature
scheme in the Week 1 baseline. The system is structured as a library
(`src/transaction.rs`, `src/block.rs`, `src/blockchain.rs`) plus a small
demo binary (`src/main.rs`).

The architecture is intentionally layered so that future milestones can
substitute or extend individual layers without rewriting the chain. The
three layers, from innermost to outermost, are:

1. **Data layer** — `Transaction`, `Block`. Pure data structures with
   serialization and hashing methods.
2. **Chain layer** — `Blockchain`. Owns the ordered list of blocks and
   the validation logic.
3. **Application layer** — `main.rs` (and, in future milestones, a
   mempool, a wallet, and a signature abstraction).

Each layer depends only on the layer below it.

## 2. Components

### 2.1 `Transaction` (`src/transaction.rs`)

A minimal value-transfer record.

```rust
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
}
```

Methods:

- `Transaction::new(sender, receiver, amount) -> Self`
- `Transaction::canonical_bytes(&self) -> Vec<u8>` — placeholder for the
  future signed-digest input.
- `impl Display for Transaction` — human-readable form
  `sender -> receiver : amount`.

Derives: `Debug, Clone, Serialize, Deserialize, PartialEq, Eq`.

### 2.2 `Block` (`src/block.rs`)

A container of transactions linked to the previous block via its hash.

```rust
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}
```

Methods:

- `Block::new(index, timestamp, transactions, previous_hash) -> Self`
  (hash left empty; caller must invoke `recompute_hash`).
- `Block::genesis(timestamp) -> Self` — constructs the genesis block
  with `index = 0`, empty transactions, and the all-zero placeholder
  `previous_hash`.
- `Block::canonical_hash_input(&self) -> Vec<u8>` — the deterministic
  byte input to the hash function. Excludes `self.hash`.
- `Block::calculate_hash(&self) -> String` — SHA-256 hex digest of
  `canonical_hash_input()`.
- `Block::recompute_hash(&mut self)` — recomputes and stores `self.hash`.
- `impl Display for Block` — multi-line, human-readable.

Derives: `Debug, Clone, Serialize, Deserialize, PartialEq`.

### 2.3 `Blockchain` (`src/blockchain.rs`)

The chain itself.

```rust
pub struct Blockchain {
    pub chain: Vec<Block>,
}
```

Methods:

- `Blockchain::new() -> Self` — creates a chain with only the genesis
  block, timestamp from the system clock.
- `Blockchain::new_with_genesis_timestamp(ts) -> Self` — same, with an
  explicit genesis timestamp for deterministic tests.
- `Blockchain::last_block(&self) -> &Block` — panics if the chain is
  empty (which can only happen if the constructor was bypassed).
- `Blockchain::add_block(&mut self, transactions: Vec<Transaction>)` —
  appends a new block linked to the current last block.
- `Blockchain::is_valid(&self) -> Result<(), String>` — runs the full
  validation pass; returns `Ok(())` or `Err(reason)`.
- `impl Display for Blockchain` — multi-line, human-readable.

Derives: `Debug, Clone`. `Default` is implemented via `Blockchain::new()`.

### 2.4 `main.rs`

A small demo binary. Constructs a chain with two blocks, prints the
chain, validates it, then demonstrates tamper detection by mutating a
transaction in-place and re-validating.

## 3. Data Models

### 3.1 Transaction

| Field | Type | Notes |
|-------|------|-------|
| `sender` | `String` | Free-form. Will become an address type (digest of a public key) in M2. |
| `receiver` | `String` | Free-form. Will become an address type in M2. |
| `amount` | `u64` | No currency semantics. Will require overflow/underflow rules in M2. |

### 3.2 Block

| Field | Type | Notes |
|-------|------|-------|
| `index` | `u64` | 0 for genesis; `prev.index + 1` for subsequent blocks. |
| `timestamp` | `i64` | Unix epoch seconds at block creation. |
| `transactions` | `Vec<Transaction>` | Empty for genesis. |
| `previous_hash` | `String` | 64-char hex SHA-256 digest. Genesis uses the all-zero placeholder. |
| `hash` | `String` | 64-char hex SHA-256 digest of `canonical_hash_input()`. Excluded from its own input. |

### 3.3 Blockchain

| Field | Type | Notes |
|-------|------|-------|
| `chain` | `Vec<Block>` | `chain[0]` is genesis; `chain[i].index == i` is an invariant maintained by `add_block`. |

## 4. Intentional Omissions for Week 1

The following are deliberately absent from the Week 1 baseline and are
documented here so that their absence is not mistaken for a bug:

- **No public keys.** `Transaction.sender` and `Transaction.receiver`
  are free-form `String`s, not addresses.
- **No signatures.** Transactions are not authenticated. Anyone can
  construct a transaction with any sender name.
- **No nonce or transaction timestamp.** A transaction is fully
  identified by its (sender, receiver, amount) tuple, which means two
  identical transactions are indistinguishable. This is acceptable for
  Week 1 because there is no mempool and no double-spend concern; it
  will change in M2.
- **No transaction hash.** A `Transaction` does not carry its own hash.
  The transaction's identity is implicit in its position within its
  containing block. This will change in M2 when transactions need to
  be referenced individually (e.g., as UTXO inputs).
- **No mempool.** `Blockchain::add_block` accepts a `Vec<Transaction>`
  directly; there is no mempool data structure.
- **No persistence.** The chain lives in `Vec<Block>` in process memory.
- **No networking.** Single-process, single-writer.
- **No consensus.** The writer is trusted to append blocks honestly;
  `is_valid()` detects after-the-fact tampering but does not prevent
  it.
- **No block-size limit.** A block may contain any number of
  transactions. This will change in M3 when a mempool and block-size
  policy are introduced.

## 5. Hash Procedure

### 5.1 Block hash

The block hash is computed as follows:

```
canonical_input = format!(
    "{}|{}|{}|{}",
    self.index,
    self.timestamp,
    serde_json::to_string(&self.transactions),
    self.previous_hash,
)
hash = SHA-256(canonical_input) as hex string
```

The `self.hash` field is **excluded** from `canonical_input`. This is
mandatory: including the hash field in its own input would create a
fixpoint equation that has no efficient solution.

### 5.2 Why this is not yet a production canonical format

The format above is deterministic for a fixed struct definition, but it
has three properties that disqualify it as a protocol-grade canonical
encoding:

1. **Field-order dependence.** `serde_json::to_string` emits struct
   fields in declaration order. If the `Transaction` struct's field
   order ever changes, every existing hash becomes invalid. A
   production format must be order-independent or must commit to a
   fixed order via a versioned schema.
2. **Delimiter fragility.** The `|` delimiter is unsafe if a `sender`
   or `receiver` ever contains `|`. Currently they are free-form
   strings, so this is a real risk.
3. **JSON-specific quirks.** JSON's escaping rules (for Unicode,
   control characters, etc.) are well-defined but add complexity that
   is irrelevant to hashing.

### 5.3 Planned migration to a strict binary encoding (Milestone M2)

When signatures are added (M2), the transaction digest will be
redefined as:

```
digest_input = length_prefix(sender_bytes) ||
               length_prefix(receiver_bytes) ||
               length_prefix(amount_bytes_be) ||
               length_prefix(nonce_bytes) ||
               length_prefix(timestamp_bytes)
```

Where `length_prefix` is a 4-byte big-endian length, and `_bytes_be`
denotes big-endian encoding. This format is:

- Order-independent within a field (lengths are explicit).
- Robust to arbitrary byte content in `sender` / `receiver`.
- Trivially extensible: new fields append without breaking old hashes
  if a version byte is included.

The `Transaction::canonical_bytes()` method already returns a placeholder
for this future format. M2 will replace its body, not its signature.

### 5.4 Transaction hash (planned, not yet implemented)

When transactions need to be referenced individually (e.g., as UTXO
inputs in M2 or later), each transaction will carry its own hash:

```
tx_hash = SHA-256(canonical_bytes())
```

This is not yet implemented because Week 1 transactions are identified
positionally within their containing block.

## 6. Invariants

The chain enforces the following invariants. Violations are detected by
`Blockchain::is_valid()`.

### 6.1 Genesis invariant

```text
chain[0].index == 0
chain[0].previous_hash == "0".repeat(64)
chain[0].hash == chain[0].calculate_hash()
```

### 6.2 Linkage invariant

For every `i > 0`:

```text
chain[i].previous_hash == chain[i - 1].hash
```

### 6.3 Content integrity invariant

For every `i >= 0`:

```text
chain[i].hash == chain[i].calculate_hash()
```

This catches:
- Transaction tampering (a transaction field was mutated after the
  block was hashed).
- Hash substitution (the stored `hash` was overwritten with a fake
  value).
- Timestamp tampering.
- Index tampering.
- `previous_hash` tampering (also caught by the linkage invariant).

### 6.4 Index monotonicity invariant (implicit)

```text
chain[i].index == i  for all i in 0..chain.len()
```

This is maintained by `add_block` (`let index = prev.index + 1;`) but
is not explicitly checked by `is_valid()` in Week 1. It will be added
in M2 when block reorganization becomes a concern.

## 7. Interfaces

### 7.1 Public API summary

```rust
// src/transaction.rs
pub struct Transaction { pub sender: String, pub receiver: String, pub amount: u64 }
impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self;
    pub fn canonical_bytes(&self) -> Vec<u8>;
}

// src/block.rs
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
}
impl Block {
    pub fn new(index: u64, timestamp: i64, transactions: Vec<Transaction>, previous_hash: String) -> Self;
    pub fn genesis(timestamp: i64) -> Self;
    pub fn canonical_hash_input(&self) -> Vec<u8>;
    pub fn calculate_hash(&self) -> String;
    pub fn recompute_hash(&mut self);
}

// src/blockchain.rs
pub struct Blockchain { pub chain: Vec<Block> }
impl Blockchain {
    pub fn new() -> Self;
    pub fn new_with_genesis_timestamp(timestamp: i64) -> Self;
    pub fn last_block(&self) -> &Block;
    pub fn add_block(&mut self, transactions: Vec<Transaction>);
    pub fn is_valid(&self) -> Result<(), String>;
}
```

### 7.2 Error handling

Week 1 uses `Result<(), String>` for validation errors and `expect()`
for cases that "cannot happen" (e.g., serializing a `Vec<Transaction>`
to JSON). This is acceptable for a teaching baseline. M2 will introduce
a proper `thiserror`-based error type.

## 8. Interactions

### 8.1 Adding a block

1. Caller invokes `bc.add_block(transactions)`.
2. `add_block` reads `last_block()` to get `index` and `previous_hash`.
3. `add_block` constructs a `Block` with `Block::new(...)`.
4. `add_block` invokes `block.recompute_hash()`.
5. `add_block` pushes the block onto `chain`.

### 8.2 Validating the chain

1. Caller invokes `bc.is_valid()`.
2. `is_valid` checks the chain is non-empty.
3. `is_valid` checks the genesis block's hash matches its content.
4. `is_valid` iterates `i` from 1 to `chain.len() - 1`:
   - Checks `chain[i].previous_hash == chain[i - 1].hash`.
   - Checks `chain[i].hash == chain[i].calculate_hash()`.
5. Returns `Ok(())` if all checks pass; otherwise `Err(reason)`.

### 8.3 Tampering (adversarial)

Tampering is detected by `is_valid()` after the fact. The chain does
not prevent tampering — that is the job of signatures, added in M2.
The four tampering modes tested in Week 1:

1. **Transaction tampering** — mutate a `Transaction` field. Detected
   by the content-integrity check.
2. **Hash substitution** — overwrite `Block.hash` with a fake value.
   Detected by the content-integrity check.
3. **Link severing** — overwrite `Block.previous_hash` with a wrong
   value. Detected by the linkage check.
4. **Genesis tampering** — mutate the genesis block's content.
   Detected by the genesis hash check.

## 9. Future Architecture (M2 and beyond)

### 9.1 M2 — Wallets and Ed25519 signatures

- Add `Wallet` struct holding an `ed25519_dalek::Keypair`.
- Add `public_key: PublicKey` and `signature: Signature` fields to
  `Transaction`.
- Replace `Transaction::canonical_bytes()` body with the strict
  binary encoding described in Section 5.3.
- Add `is_valid_signature()` to the validation pass.

### 9.2 M3 — Mempool and block construction

- Add `Mempool` struct holding `Vec<Transaction>` of unconfirmed
  transactions.
- `Mempool::build_block(prev_block, max_size) -> Block` selects
  transactions by fee (or, in Week 1, by FIFO), respecting the block
  size limit.
- `Blockchain::add_block_from_mempool(&mut self, mempool: &mut Mempool)`.

### 9.3 M4 — Pluggable signature abstraction

```rust
pub trait SignatureScheme {
    type PublicKey;
    type Signature;
    fn sign(secret_key: &SecretKey, message: &[u8]) -> Self::Signature;
    fn verify(public_key: &Self::PublicKey, message: &[u8], signature: &Self::Signature) -> bool;
}

// Implementations:
pub struct Ed25519Scheme;
pub struct MlDsaScheme;
pub struct DummyScheme;  // for tests
```

`Transaction` becomes generic over `S: SignatureScheme`, or holds boxed
trait objects. The chain code is unchanged.

### 9.4 M5 — ML-DSA integration

- Add `pqcrypto-dilithium` (or equivalent vetted binding) dependency.
- Implement `MlDsaScheme` for the trait.
- Choose ML-DSA-65 for transactions, ML-DSA-87 for any future
  consensus-critical signatures.
- Run the chain end-to-end with ML-DSA signed transactions.

### 9.5 M6 — Benchmarking

- Add `criterion` benchmarks for hashing, single verification, batched
  verification, and full-chain validation.
- Compare Ed25519Scheme, MlDsaScheme, and (optionally) a hybrid
  `Ed25519AndMlDsaScheme`.
- Output: `research/benchmark-report.md`.

## 10. Open Architectural Questions

These are questions the implementation will eventually have to answer.
They are recorded here so they are not forgotten.

1. **Block header vs. body separation.** Should the block hash cover
   only a small header (index, timestamp, prev_hash, merkle_root) or
   the full transaction list? Bitcoin's choice is the header-only
   approach with a Merkle root. This matters more when signatures
   arrive because the validation cost of recomputing the Merkle root
   scales with transaction count.

2. **Merkle tree inclusion proofs.** Light clients need to verify that
   a specific transaction is in a specific block without downloading
   the whole block. This requires a Merkle (or other accumulator)
   structure over the transactions. Not needed for Week 1 but
   architecturally consequential.

3. **Address derivation.** When `sender` / `receiver` become addresses,
   are they SHA-256 of the public key (Bitcoin-style), Keccak-256
   (Ethereum-style), or something else? The choice affects future
   address-format migration (see `literature_review.pdf` Section 6,
   Gap 4).

4. **Signature aggregation field.** Even though signature aggregation
   is out of scope for the foreseeable future, the block format should
   reserve a field for an aggregate signature so that adding it later
   does not require a hard fork. This is a placeholder; see
   `literature_review.pdf` Section 4.2.

5. **Hybrid signature semantics.** If a hybrid Ed25519+ML-DSA signature
   is supported, is it (a) required for all transactions, (b) optional
   per transaction, or (c) activated by a chain-rule fork height?
   Each choice has different downgrade-resistance properties.
