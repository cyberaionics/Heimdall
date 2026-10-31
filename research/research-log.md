# Research Log

> **Purpose:** Chronological record of observations, experiments,
> measurements, failures, and potential research questions.
>
> **Rules:**
>
> - Use the observation template below. Do not invent values.
> - Clearly label every entry as **Fact** (supported by source or
>   reproducible implementation), **Decision** (explicit project
>   choice), **Hypothesis** (proposition requiring evidence),
>   **Observation** (something actually measured), or **Open
>   question**.
> - Mark estimates and simulated results as such. Do NOT report them
>   as actual measurements.
> - Update an entry's status when new evidence arrives. Do not delete
>   old entries; supersede them with a dated note.

---

## Observation Template

```markdown
## Observation #NNN

- Date: YYYY-MM-DD
- Component:
- Experiment:
- Baseline:
- Observation:
- Quantitative impact:
- Why it matters:
- Current workaround:
- Limitation:
- Open question:
- Potential research direction:
- Follow-up experiment:
- Status: Fact | Decision | Hypothesis | Observation | Open question
```

---

## Seeded Example Entries

> ⚠️ The entries below are EXAMPLES intended to show how to fill in
> the template. They are **NOT real observations** — they describe
> hypothetical results that have not been measured. Replace them with
> real entries as soon as actual measurements are taken.

## Observation #001 (EXAMPLE — not a real measurement)

- Date: 2026-10-28
- Component: `Block::calculate_hash`
- Experiment: Hash a single block with N transactions, varying N from
  1 to 1,000, and measure wall-clock time per hash.
- Baseline: SHA-256 on a 4 KB input (rough proxy for a single
  transaction).
- Observation: *(placeholder — to be filled in once `cargo bench`
  is set up in M6)*. Expected approximate range: 1-50 microseconds
  per hash for blocks under 100 transactions, scaling roughly linearly
  with transaction count.
- Quantitative impact: TBD.
- Why it matters: The per-block hash cost is the floor of block
  validation cost. Any PQ signature verification cost will be added
  on top of this. Without a baseline, the marginal cost of PQ
  signatures cannot be attributed correctly.
- Current workaround: None needed for Week 1.
- Limitation: Microbenchmarks on the developer's machine are not
  reproducible across hardware. The M6 benchmark report will pin
  hardware and report variance.
- Open question: Does the `sha2` crate's SIMD acceleration engage on
  the developer's CPU? If yes, the baseline is faster than a naive
  SHA-256 implementation would suggest.
- Potential research direction: Compare `sha2` (Rust) against
  OpenSSL's SHA-256 against a hand-rolled SHA-256 to quantify the
  impact of implementation quality on the baseline.
- Follow-up experiment: Run `cargo bench` with `criterion` and
  record p50, p99, and variance. Repeat on at least two machines.
- Status: **Hypothesis** (the "expected approximate range" line is a
  hypothesis, not a measured result).

## Observation #002 (EXAMPLE — not a real measurement)

- Date: 2026-10-28
- Component: `Blockchain::is_valid`
- Experiment: Build a chain of 1,000 blocks, each with 10
  transactions. Run `is_valid()` end-to-end. Measure wall-clock time.
- Baseline: Same chain with empty transactions (only the block header
  is hashed).
- Observation: *(placeholder — to be filled in M6)*. Expected: the
  full pass should take low milliseconds on commodity hardware
  without signature verification.
- Quantitative impact: TBD. The interesting number is the *delta*
  between this baseline and the same chain with Ed25519 verification
  (M2) and then ML-DSA verification (M5).
- Why it matters: This is the reference point against which all
  signature-scheme comparisons will be made. Without it, "ML-DSA is
  N× slower than Ed25519" is a meaningless claim.
- Current workaround: None.
- Limitation: In-memory chain. A real node loading the chain from
  disk would also pay I/O cost; this baseline excludes that.
- Open question: Should the baseline include the I/O cost of loading
  blocks from a hypothetical disk format? Probably not for M6; we
  want to isolate the validation cost.
- Potential research direction: Eventually, measure disk-loaded
  validation and decompose into hashing vs. signature vs. I/O.
- Follow-up experiment: Add a `load_from_disk` variant in M6 and
  measure both.
- Status: **Hypothesis** (the "low milliseconds" line is a
  hypothesis, not a measured result).

## Observation #003 (EXAMPLE — not a real measurement)

- Date: 2026-10-28
- Component: `Transaction::canonical_bytes` (placeholder) vs. the
  planned strict binary encoding (architecture.md §5.3).
- Experiment: Hash 1,000,000 transactions with each encoding. Compare
  throughput.
- Baseline: The current `format!("{}|{}|{}", ...)` placeholder.
- Observation: *(placeholder)*. Expected: the binary encoding should
  be modestly faster (perhaps 1.5-3×) because it avoids UTF-8
  validation and string formatting overhead.
- Quantitative impact: TBD.
- Why it matters: The signed-digest input is invoked once per
  transaction at sign time and once at verify time. At blockchain
  scale, even a 2× difference in digest computation can dominate the
  signing/verification cost for small signature schemes.
- Current workaround: Use the placeholder for Week 1.
- Limitation: The placeholder is unsafe (delimiter collisions); the
  comparison must be done with realistic transaction data that
  exercises edge cases (empty fields, very long fields, non-ASCII
  content).
- Open question: Does the binary encoding introduce alignment
  penalties on common CPUs? Big-endian length prefixes are portable
  but may be slower than native-endian on little-endian hardware.
- Potential research direction: Compare big-endian, little-endian,
  and length-prefixed-LEB128 encodings for the digest input.
- Follow-up experiment: Implement all three, benchmark, and pick the
  fastest with portability constraints documented.
- Status: **Hypothesis** (the "1.5-3× faster" line is a hypothesis).

---

## Real Observations

*(None yet. The first real observation should be filed once
`cargo test` is run in the user's environment and the test output
is examined.)*

---

## Append-Only Log

When adding a new observation, do NOT edit existing entries. Instead,
append a new entry with the next sequential number. If a previous
observation needs to be revised, add a new entry that references the
old one (e.g., "Supersedes Observation #002 — actual measurement
disagrees with the hypothesis by 4×") and update the old entry's
Status line to point to the superseding entry.
