# heimdall — Planning

> **Purpose:** Project motivation, goals/non-goals, roadmap, reasoning,
> alternatives considered, and decision rationale. This is the document
> where *why* questions are answered. *What* and *how* live in
> `architecture.md`; *current state* lives in `context.md`.

## 1. Motivation

The security of every widely deployed blockchain rests on classical
public-key cryptography — ECDSA, EdDSA, and BLS — whose security collapses
against a sufficiently large fault-tolerant quantum computer running Shor's
algorithm. NIST completed its post-quantum cryptography standardization in
August 2024 with the publication of FIPS 203, 204, and 205, but the
engineering work of actually migrating blockchain systems to these new
primitives has barely begun.

Blockchain systems occupy an unusually difficult position in this
transition. Unlike TLS, where a key rotation is a software upgrade that
propagates across a fleet of servers within weeks, blockchains are
append-only ledgers whose entire history must remain verifiable
indefinitely. A signature produced today may need to verify correctly
thirty years from now, against a chain state that has been reorganized,
re-synced, and re-archived dozens of times. The literature has many
proposals for PQ-secure blockchain primitives but very few production-grade
implementations, and quantitative measurements of integration overhead are
scattered and difficult to compare across projects. This project exists
to fill that gap.

## 2. Goals

The project's primary goals are:

1. **Build a minimal, instrumented blockchain baseline** in Rust that is
   small enough to reason about end-to-end but realistic enough to surface
   real engineering problems when PQ authentication is added.
2. **Incrementally introduce classical signature authentication** (likely
   Ed25519) over transactions, with a pluggable `SignatureScheme` trait so
   that PQ schemes can be dropped in later without rewriting the chain.
3. **Integrate ML-DSA (FIPS 204)** using a vetted existing implementation —
   not a from-scratch reimplementation of the primitive.
4. **Measure the impact** of PQ signatures on block size, validation
   throughput, and storage growth at small but realistic scales.
5. **Surface novel engineering observations** that the existing literature
   does not capture, and translate them into candidate specifications or
   research questions.
6. **Document every decision** with rationale, alternatives, and
   consequences so the project is defensible to a thesis committee.

## 3. Non-Goals

The following are explicitly out of scope, at least for the initial
milestones:

- **Inventing a new cryptographic primitive.** The project uses
  standardized schemes (SHA-256, Ed25519, ML-DSA) and treats their
  internals as black boxes. The research contribution, if any, is at the
  systems-integration layer.
- **Implementing ML-DSA from scratch.** The project integrates an
  existing vetted implementation via a Rust binding, not a from-scratch
  implementation of the lattice primitive.
- **Production-grade consensus.** Proof-of-work, proof-of-stake, BFT
  leader election, and similar are out of scope. The chain is local and
  single-writer for the foreseeable future.
- **P2P networking.** No transaction gossip, no peer discovery, no block
  propagation. The chain is constructed programmatically in code and tests.
- **Smart contracts.** No scripting layer.
- **A production wallet UI.** A CLI demo and unit tests are sufficient.
- **Production database.** Blocks live in memory in `Vec<Block>` for now.
  Persistence is a future concern that will be addressed only if research
  requires it.
- **Side-channel resistance.** The reference implementations of Ed25519
  and ML-DSA handle their own side-channel concerns. The project does not
  reimplement them.
- **Hash-based PQ signatures (SLH-DSA / SPHINCS+).** Considered as a
  conservative fallback for cold-wallet use cases, but not the primary
  integration target for transaction authentication, because of the
  multi-KB signature overhead.

## 4. Roadmap

The roadmap is incremental. Each milestone produces a verifiable artifact
and feeds observations into `research/research-log.md`.

### Milestone M1 — Week 1 baseline (current)

**Status:** code written, awaiting `cargo test` verification in the
user's environment.

- `Transaction`, `Block`, `Blockchain` data structures.
- SHA-256 block hashing with explicit canonical byte input.
- Genesis block with all-zero `previous_hash` placeholder.
- Chain validation enforcing both invariants (links + content integrity).
- Tampering tests for transaction mutation, hash substitution, broken
  links, and genesis tampering.
- Documentation: `context.md`, `planning.md`, `architecture.md`,
  `research/research-log.md`, `README.md`, `literature_review.pdf`.

**Definition of done:** `cargo test` passes all unit tests; `cargo run`
demonstrates a small chain with tamper detection.

### Milestone M2 — Wallet and classical signatures

- Define a `Wallet` struct that holds a keypair.
- Adopt Ed25519 (`ed25519-dalek` crate) as the classical signature scheme.
- Add `public_key` and `signature` fields to `Transaction`.
- Define the transaction digest as a strict, length-prefixed binary
  encoding (replacing the current `canonical_bytes()` placeholder).
- Add signature verification to chain validation.

**Definition of done:** a transaction with an invalid signature is
rejected by `is_valid()`; the validation cost of signature verification
is measured and recorded in `research/research-log.md`.

### Milestone M3 — Mempool and block construction

- Introduce a `Mempool` struct holding validated-but-unconfirmed
  transactions.
- Implement block construction from mempool: select transactions, build a
  block, compute the hash, append to the chain.
- Add a simple block-size-limit policy (e.g., 1 MB or 10 KB depending on
  signature scheme).

**Definition of done:** a mempool with 100 transactions can produce a
valid block; the block construction latency is measured.

### Milestone M4 — Pluggable SignatureScheme trait

- Define `trait SignatureScheme { type PubKey; type Sig; fn sign(...);
  fn verify(...); }`.
- Refactor `Transaction` to be generic over `SignatureScheme` (or use
  an enum / boxed trait object).
- Implement `Ed25519Scheme` and a `DummyScheme` (deterministic SHA-256
  signature for tests).

**Definition of done:** the same chain code can validate transactions
signed by either scheme via the trait abstraction.

### Milestone M5 — ML-DSA integration

- Add `pqcrypto-dilithium` (or equivalent vetted binding) as a dependency.
- Implement `MlDsaScheme` for the trait.
- Choose a parameter set (likely ML-DSA-65 for transactions, ML-DSA-87
  for consensus-critical signatures if/when those exist).
- Run the chain end-to-end with ML-DSA signed transactions.

**Definition of done:** a chain with ML-DSA-signed transactions validates
correctly; signature size, public key size, signing latency, and
verification latency are measured and recorded.

### Milestone M6 — Controlled benchmarking

- Add `criterion` benchmarks for: block hashing, single-signature
  verification, batched verification, full chain validation at varying
  block counts and signature schemes.
- Compare classical (Ed25519), PQ (ML-DSA), and (optionally) hybrid
  configurations.
- Produce plots via Python (matplotlib) for inclusion in a future
  research article draft.

**Definition of done:** a benchmark report exists in
`research/benchmark-report.md` with reproducible numbers.

### Milestone M7 — Research article draft

- Identify the strongest novel observations from M5 and M6.
- Compare against the literature reviewed in `literature_review.pdf`.
- Draft a research article.

## 5. Decision Rationale

### Decision D1: Rust as the implementation language

**Decision:** Rust.

**Rationale:** Rust provides memory safety without garbage collection,
which is critical for cryptographic code; it has mature crates for the
primitives we need (`sha2`, `ed25519-dalek`, `pqcrypto`); its trait system
supports the pluggable `SignatureScheme` abstraction cleanly; and it has
first-class benchmarking via `criterion`.

**Alternatives considered:**
- *C++* — equally capable, but manual memory management is a liability
  for cryptographic code and slows the research loop.
- *Go* — simpler, but its trait system (interfaces) is weaker than
  Rust's for the pluggable-signature abstraction.
- *Python* — too slow for realistic validation throughput measurement.
- *Haskell* — interesting for formal verification but less ergonomic
  for the systems-integration work that this project actually is.

**Consequences:** The user must be comfortable with Rust's borrow
checker. The professor's stated methodology (build, observe, measure) is
well-served by Rust's emphasis on explicitness.

### Decision D2: SHA-256 for the Week 1 block hash

**Decision:** SHA-256 via the `sha2` crate.

**Rationale:** SHA-256 is what Bitcoin uses; it is familiar to anyone
reading the code; it is fast enough for Week 1; and it is a safe choice
that does not preclude later switching to SHA-3 or BLAKE3 if research
justifies it.

**Alternatives considered:**
- *SHA-3 (Keccak)* — structurally different from SHA-2 and resistant to
  the same attacks; would be the right choice if we were worried about
  SHA-2 specifically. For a teaching baseline, the familiarity of SHA-256
  wins.
- *BLAKE3* — significantly faster, but the speedup is irrelevant at
  Week 1 scale and the implementation landscape is less mature.

**Consequences:** Block hashes are 32 bytes / 64 hex characters. The
hash function is a single point of failure for chain integrity — if a
break in SHA-2 is ever discovered, the chain would need a hard fork.

### Decision D3: All-zero placeholder for genesis `previous_hash`

**Decision:** Genesis uses `"0".repeat(64)` (64 hex zeros) as
`previous_hash`.

**Rationale:** Makes the genesis block trivially recognizable in test
output and audit logs. Avoids the temptation to "hash nothing" or hash
a constant string, both of which would be valid but less obvious.

**Alternatives considered:**
- *Hash of an empty input* (`e3b0c442...`) — valid SHA-256 output but
  indistinguishable from a real previous block hash in output.
- *Hash of the literal string "genesis"* — also valid but introduces a
  magic string.

**Consequences:** Genesis detection in any future tooling reduces to
checking `previous_hash == "0".repeat(64)`. This will need to be
documented as a chain rule if the project ever publishes a spec.

### Decision D4: `serde_json` as the *initial* serialization format inside the block hash input

**Decision:** Use `serde_json::to_string(&self.transactions)` as the
serialization step inside `Block::canonical_hash_input()`.

**Rationale:** serde_json is deterministic for a fixed struct definition
(fields emitted in declaration order); it is human-readable in test
output; it is already a dependency we declare.

**Alternatives considered:**
- *Length-prefixed binary encoding* — strictly better as a canonical
  format, but heavier to write and read for Week 1. Will be adopted in
  Milestone M2 when signatures require it.
- *Postcard (serde-postcard)* — compact binary serde format. Good
  candidate for M2.
- *Manual byte concatenation* — what `Transaction::canonical_bytes()`
  already does for the future signed-digest input. Less readable than
  JSON in test output.

**Consequences:** The current serialization is NOT a protocol-grade
canonical format. This is documented in `architecture.md` Section 5 and
in `context.md` Section 11. The serialization will be replaced, not
extended, when signatures arrive.

### Decision D5: `i64` for the timestamp

**Decision:** `timestamp: i64` (Unix epoch seconds).

**Rationale:** `i64` permits negative values, which we do not use in
Week 1 but which are useful for synthetic test fixtures that should sort
before any real timestamp. Seconds (rather than milliseconds or
nanoseconds) keep test output readable.

**Alternatives considered:**
- *`u64`* — semantically cleaner (no negative time) but loses the
  ability to use sentinel negative values in tests.
- *`SystemTime`* — heavier, not serializable as-is.
- *ISO 8601 string* — human-readable but parses slowly and breaks
  determinism if formatting varies.

**Consequences:** The chain is limited to one block per second per
writer. If sub-second block production is ever needed, switch to
millisecond precision (`i64` of millis).

### Decision D6: No signatures in Week 1

**Decision:** The Week 1 baseline has no signatures of any kind.

**Rationale:** The professor's methodology is build → observe → measure.
Adding signatures in Week 1 would conflate two layers (chain integrity
vs. transaction authentication) and make it harder to attribute any
observed problem to its actual cause. By building a signature-less
baseline first, we get a clean reference point for measuring the impact
of signatures later.

**Alternatives considered:**
- *Start with Ed25519 immediately* — would save a milestone but loses
  the clean reference point.
- *Use a dummy signature scheme from day one* — would force the
  `SignatureScheme` trait abstraction to be designed before any
  concrete scheme is integrated, which is premature abstraction.

**Consequences:** A Week 1 chain can be tampered with trivially (just
mutate a transaction in-place). This is by design. The tampering tests
verify that the *chain detects* the tamper, not that the tamper is
prevented — that is the job of signatures, added in M2.

### Decision D7: Project name "Heimdall"

**Decision:** The project is named **Heimdall** (display name) /
`heimdall` (crate name, lowercase per Rust convention).

**Rationale:** Heimdall is the Norse god who guards the Bifrost bridge
and sees what is coming. Post-quantum security is, at its core, about
seeing the quantum threat coming and preparing before it arrives. The
name carries a narrative hook that fits a research project: "Heimdall: A
Watchman for Post-Quantum Blockchain Authentication" is a paper title
that writes its own introduction. It is distinctive enough that a
committee member will remember it, pronounceable, and free of obvious
collisions in the PQ-blockchain space (QRL, Algorand, Bitcoin-PQ forks
all use different naming conventions).

**Alternatives considered:**
- *LatticeChain* — clear and descriptive but generic; no narrative.
- *Tessera* — original (a mosaic tile, evoking lattice geometry) and
  the original second choice. Retained as a fallback if Heimdall turns
  out to collide with an existing project.
- *Ledger204* — references FIPS 204 directly; memorable but commits
  the name to a single standard, which limits future scope.
- *DilithiumChain* — names the scheme; locks the project into one
  primitive, which is undesirable if Falcon or SLH-DSA comparisons
  are added later.
- *pq-chain* — what the codebase was originally called. Honest but
  generic; the rename reflects a deliberate choice to give the project
  a defensible identity for a future research article.

**Consequences:**
- Directory, crate name, and all documentation now use `heimdall` /
  `Heimdall` consistently.
- The name does not commit the project to any specific cryptographic
  primitive; ML-DSA is the planned integration target, but the name
  works equally well if Falcon or SLH-DSA comparisons are added.
- Domain and crate-registry availability has not been verified; this
  should be checked before any public release. The name is used
  internally for now.

## 6. Risks and Mitigations

### Risk R1: The user gets stuck on Rust borrow-checker issues

**Likelihood:** High (especially if the user is new to Rust).
**Mitigation:** The code is intentionally minimal and uses only owned
types (`String`, `Vec`) in the data structures; no lifetimes, no
references in struct fields. Tests are also minimal. If a borrow-checker
issue arises, the user should paste the error and I will diagnose.

### Risk R2: The `serde_json` serialization produces surprises

**Likelihood:** Low for Week 1, higher in M2.
**Mitigation:** Documented in `architecture.md` Section 5. The
serialization will be replaced with a strict binary encoding before
signatures are added.

### Risk R3: The `sha2` crate API changes between versions

**Likelihood:** Low (the crate is stable).
**Mitigation:** `Cargo.toml` pins `sha2 = "0.10"`. If the user has an
older or newer version, they should run `cargo update -p sha2`.

### Risk R4: The ML-DSA integration (M5) reveals a fundamental issue

**Likelihood:** Unknown — this is what the research is for.
**Mitigation:** All observations are recorded in
`research/research-log.md` with explicit Fact / Decision / Hypothesis /
Observation labels. If a fundamental issue is found, that *is* the
research contribution.

### Risk R5: The literature review missed a relevant paper

**Likelihood:** Moderate; the field is active.
**Mitigation:** The literature review (`literature_review.pdf`) is
versioned and will be updated as the project progresses. Any paper
discovered later that changes the project's positioning will be added to
the annotated bibliography and cited where relevant.

## 7. Success Criteria

The project is successful if, at the end of Milestone M7, the following
all hold:

1. A reproducible Rust implementation of a blockchain baseline with
   pluggable classical and PQ signature schemes exists and is documented.
2. Quantitative measurements of the impact of PQ signatures on block
   size, validation throughput, and storage are recorded and reproducible.
3. At least one novel engineering observation, supported by both
   implementation evidence and literature comparison, is identified.
4. A research article draft exists that frames the observation as a
   research contribution.

The project is *not* successful if it produces a working blockchain
without any novel observation, or if it produces a research claim that
is not supported by reproducible implementation evidence.
