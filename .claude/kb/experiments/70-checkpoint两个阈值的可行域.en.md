## E70 Feasible Region for Two Checkpoint Thresholds — Run (2026-08-31, Pure Arithmetic; Expanded to 5 Unit Tests on 2026-09-03, All 6 Mutations Caught): **Practical Constraint on `T_dirty` Is Dirty Page Memory, Not Ring Geometry**

[中文](70-checkpoint两个阈值的可行域.md) · **English** · [日本語](70-checkpoint两个阈值的可行域.ja.md)

D16 (Publish Semantics) settled item 2 settled the trigger condition `(now − last ≥ T_time) ∨ (dirty_bytes ≥ T_dirty)`,
D16 (Publish Semantics) settled item 5 asked what values to choose for those two numbers, and described two couplings. **What E70 (Feasible Region for Two Checkpoint Thresholds) verified was precisely whether those two statements held true.**

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. `T_dirty` upper bound = `(ring_size/F − record_header) × (node_size / entry_bytes)`.
   **If larger than dirty page memory a machine could plausibly possess (taking 1 TiB as absurdity threshold), judge that "ring geometry is not the constraint on T_dirty".**
2. `T_time` lower bound: Given amortized budget `B` (blocks/operation), solve inversely from two-point interpolation of E16 (Journal Role: WAL vs Intent Log)
   how many operations a single checkpoint must cover at minimum, then convert to milliseconds per fsync rate from E44 (Sequence Bit-width Local Measurement and Cost).
3. The two bounds must not depend on each other; if solved `T_time` in turn depends on `T_dirty`, judge "model cycle", invalidating entire round.

### Failure Clauses (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| **Positive control**: Doubling `F` ⇒ upper bound halved | **Passed** (17179829248 → 8589894656, difference exactly header bytes deducted once more) |
| **Negative control**: entry bytes = node size ⇒ amplification factor 1, upper bound collapses to `ring_size/F − header` | **Passed** (33554354 == 33554354) |
| E16 (Journal Role: WAL vs Intent Log) contains only two points; interpolation beyond [100, 1000] marked as extrapolation | Marked |

### Measurement

#### Upper Bound on `T_dirty`: Amplification Factor Is `node/entry = 512`

A dirty node is **16 KiB data**, but in the journal consumes merely **a single 32-byte position entry** (D23 (Journal Role and Format): records carry pointer layer target state, no data movement)
⇒ **every 1 byte accommodated by the ring permits 512 bytes of dirty data**.

| Ring Size | F=2 Upper Bound | F=3 Upper Bound |
|---|---|---|
| 16 MiB | 4.0 GiB | 2.7 GiB |
| 64 MiB | **16 GiB** | 10.7 GiB |
| 256 MiB | 64 GiB | 42.7 GiB |
| 1 GiB | 256 GiB | 170 GiB |

**Criterion 1 not triggered** (all lower than 1 TiB absurdity threshold, `absurd=0`).
⚠️ **Yet it provides a directional answer**: a common ring such as 64 MiB already permits 16 GiB of dirty data,
**over an order of magnitude larger than dirty page memory an actual machine possesses** ⇒ **what practically constrains `T_dirty` is memory and crash loss window, not ring geometry**.
The statement in body of D16 (Publish Semantics) settled item 5 that "upper bound ties to ring geometry" **is correct in direction but bears no structural load** — it is a far-off ceiling.

#### Lower Bound on `T_time`: Inversely Solved by Amortized Budget, Varies by Two Orders of Magnitude Across Workloads

| Workload | Budget (blocks/op) | Operations One Checkpoint Must Cover | `T_time` Lower Bound (at 2785 fsync/sec) |
|---|---|---|---|
| multistream | 0.2 | 262 ops | **94 ms** |
| multistream | 0.1 | 547 ops | **197 ms** |
| multistream | 0.05 | 1142 ops (**extrapolated**) | 410 ms |
| seq | 0.2 / 0.1 / 0.05 | **Already satisfied at ckpt=100** | **No lower bound** |

⇒ **Lower bound on `T_time` is entirely governed by the worst workload**: multistream workloads require 100–200 ms scale, while sequential workloads require none at all.
D25 (Target Workload Priority) settled that the primary workload is seq (single fsync carries 8 leaves, falling on 1 shared spine)
⇒ **Governing by primary workload yields "no lower bound needed", while governing by worst workload yields 200 ms**. Between these two choices, **E70 (Feasible Region for Two Checkpoint Thresholds) does not choose on its behalf.**

### What It Cannot Answer

1. Pure arithmetic model: no transaction layer, no journal implementation, 0 file operations.
2. **"Each dirty node occupies exactly one entry in journal" is an assumption** — the question from C37 (Undefined Mapping Between Transaction and Record)
   whether "a transaction produces exactly one record" has been repaid (a transaction may span multiple records, I-8.8 (Transactions in Prefix Are Not Split) judged, see repaid debts in owed check table); this model still calculates one entry per dirty node. Entry width 32 B is also input, not measurement.
3. Interpolation relies solely on two points from E16 (Journal Role: WAL vs Intent Log); `budget=0.05` tier is an extrapolation.
4. **Does not model crash loss window** — that is one of the actual constraints on `T_dirty`; E70 (Feasible Region for Two Checkpoint Thresholds) only proved ring geometry is not.

### Prerequisites

None. Pure arithmetic.

### Measurement Basis and Re-run

- Code `scripts/experiments/e70_ckpt_thresholds.rs`
  (`cargo run --release --bin e70-ckpt-thresholds`),
  raw output `e70-ckpt-thresholds-2026-08-31.out` (18 lines, ending with line `emitted=18`).
- Mutation table `scripts/mutation/tables/e70_ckpt_thresholds.tsv` (2026-09-03, all 6 caught).
  Supplemented unit tests pinned small ring underflow guardrails — when the ring cannot fit a record header, upper bound must be 0; before 2026-09-03 no test traversed that branch.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D16 (Publish Semantics) settled item 5 | Support | 2026-10-11 unaffected: knowledge rot governance updated only obsolete status sentences; conclusions and artifact unchanged |
| D16 (Publish Semantics) settled item 2 | Material prep | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |
| D23 (Journal Role and Format) settled item 20 | Does not affect | 2026-10-11 unaffected: knowledge rot governance updated only obsolete status sentences; conclusions and artifact unchanged |
| D25 (Target Workload Priority) settled item 1 | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |

## Historical Versions

Historical entries for E70 (Feasible Region for Two Checkpoint Thresholds) are concentrated in [experiments-history.md](../experiments-history.md).
