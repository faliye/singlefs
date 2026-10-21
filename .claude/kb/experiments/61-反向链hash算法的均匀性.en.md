## E61 (Uniformity of Reverse Chain Hash Algorithm) — Run (2026-08-31), Conclusion Revoked: Three-Way Inference Identified Two Modeling Defects; "Difference Only in Instance ID" Was Model Artifact

[中文](61-反向链hash算法的均匀性.md) · **English** · [日本語](61-反向链hash算法的均匀性.ja.md)

**Answers D23 (Journal Role & Format) settled item 11 (which hash algorithm to select for reverse chain).**

> ⚠️ **Metric 2 conclusion of E61 (Uniformity of Reverse Chain Hash Algorithm) is revoked (pointed out independently by counterfactual and local legs of three-way inference; verified item-by-item by main agent).**
> Metric 1, Metric 3, and all unit tests stand; **revoked is the claim that "under actual layout differences concentrate solely in the 4-byte instance ID", along with the justification derived from it for selecting Arm A.**
> **The corrected experiment was not run and is not planned to be run** (user settlement): margin provided by E51 (Number of Reverse Chain Collision Opportunities) spans three orders of magnitude
> (lifetime probability of at least one occurrence is 8.50×10⁻⁶ against a 1% criterion line) ⇒ both candidates are fully adequate, and function selection is not load-bearing.
> D23 (Journal Role & Format) settled item 11 therefore settled on full 32-bit CRC32C on secondary grounds. Artifacts and criteria in this document remain intact.
>
> | Defect | Confirmed Concrete Realization |
> |---|---|
> | **① No new fields populated in 95 / 99 byte header tiers** | `make_header` only populated bytes 0..54, zeroing the rest; while `variable_bits = (header_length_in_bytes − 32) × 8` set scan range to bytes 0..62 / 0..66 ⇒ **settled item 7 transaction sequence number + commit flag, and settled item 8 reverse chain (bytes 86..95 / 86..99) had zero bits scanned**. Those two tiers were effectively "86-byte header followed by zeroes". Since `crc32c(m‖0ᵏ)` is an affine bijection of `crc32c(m)`, birthday collision counts of 114 / 114 / 114 for Arm A matching cell-by-cell was mathematically enforced, **rendering Criterion 5 devoid of discriminative power for Arm A** |
> | **② Difference set was hardcoded** | `record_length` set to `header_length_in_bytes`, `named_count` constant 7, `nonce` function of `jsn` ⇒ competing records were **constructed to be identical** across these three fields. Yet they represent two **distinct transactions**, with nothing guaranteeing identical `named_count`; I-6.1 (Nonce Uniqueness) additionally mandates pool-wide nonce uniqueness. ⇒ "Difference concentrates solely in instance ID" was the product of these three hardcoded choices |
> | **③ Metric 1 was algebraically impossible to fail for Arm A** | Full rank of CRC-32 on differences constrained to ≤32-bit windows is an **algebraic tautology** (`gcd(x, g) = 1` and `deg d < 32 ⇒ xᵏd mod g ≠ 0`), independent of base, header length, or field layout. 1379 windows represent 1379 instances of the identical theorem, not 1379 independent empirical facts |
> | **④ Metric 3 lacked statistical power** | λ=128, Poisson sd=11.3 ⇒ band [64, 256] in Criterion 4 corresponds to **±5.7σ**, capable only of ruling out "effective bit width loss ≥ 1 full bit". **"Arm B truncation is uniform" was an overstatement**; evidence only supports "Arm B loses less than 1 bit" |
>
> ⚠️ **This is precisely the pattern targeted by `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "forbidding constructing models after deciding conclusion"**,
> and indeed its primary archetype: every step seems reasonable, yet those steps were selected to fit the conclusion.
> Defect ② had all three arms sharing `make_header` ⇒ **inter-arm comparisons appeared completely normal**, exactly
> what `.claude/singlefs-ai-sop/rules/test-discipline.md` "comparing multiple arms mutually fails to detect when all arms err together" was designed to stop.

D23 (Journal Role & Format) settled item 8 settled width at 32 bits, and settled item 10 settled coverage (entire header of prior record,
where 32-byte `header_csum` is excluded from hash input). **Missing was solely which function to compress those bytes into 32 bits.**

⚠️ **Not a matter of "arbitrary selection"**: E51 (Number of Reverse Chain Collision Opportunities) judging "32 bits is sufficient" rested on the premise that
**false acceptance rate = 2⁻ⁿ**, a curve **conditioned on uniform chain values**.
E40 (Ciphertext Checksum Bit Width) demonstrated empirically that **truncated CRC is not simply a shorter CRC** (full-width CRC32C achieves 0 escapes on single-bit flips,
whereas truncating to 8 bits yields 3212 ppm) ⇒ if chain values adopted "wider CRC truncated to 32 bits", that curve does not apply.

### What to Measure

**Where differences lie between two compared record headers is structured, not random.**
Record headers from two timelines are constructively identical across magic / version / algo_type / record_length;
differences concentrate in `jsn` (including 32-bit instance ID), `checkpoint_txg`, `tail_lsn`, and `nonce`.
⇒ What must truly be asked is not "whether collision rate under random inputs is 2⁻³²", but
**"what collision count each arm achieves when differences are constrained within a specific window"**.

Three arms:

| Arm | Representation | Why Included |
|---|---|---|
| Arm A | **Full 32-bit CRC32C** | Identical function to ciphertext checksum settled in D19 (Block Pointer Structure & Width Budget) settled item 2 |
| Arm B | **SHA-256 truncated to 32 bits** | Candidate representing "32-bit truncated cryptographic hash"; **requires proving its truncation is uniform** |
| Arm C | **CRC-64 truncated to low 32 bits** (Positive control) | E40 (Ciphertext Checksum Bit Width) conclusively proved truncated CRC degrades. Failing to measure its degradation ⇒ criteria lack discriminative power |

**Core technique: CRC is a linear mapping over GF(2); hence collision count in a window can be computed exactly without enumeration.**
Constraining differences to a w-bit window ⇒ those differences form a w-dimensional linear subspace;
CRC linearly maps it into GF(2)³². **Collision count is exactly 0 ⟺ rank of this linear mapping = w.**
⇒ Computing images of w basis vectors and finding rank is equivalent to enumerating all 2^w differences.
Applies to both Arm A and Arm C (truncation remains linear); **does not apply to Arm B**—cryptographic hash is non-linear, permitting only sampling.
**"Whether rank verdict can be utilized" is itself a conclusion**, not an implementation detail.

### Criteria (Hardcoded Pre-Run)

| # | Criterion |
|---|---|
| 1 | **Rank Criterion**: for each 32-bit window offset, rank of difference space image must = 32 ⇒ collision count in that window is **exactly 0** |
| 2 | **Positive control must fail**: Arm C (truncated CRC-64) must exhibit rank < 32 in at least one window. **Full rank everywhere ⇒ rank criterion cannot discriminate arms, entire round scrapped** |
| 3 | **Absolute Value Assertions** (preventing shared errors across three arms): `CRC32C("123456789") == 0xE3069283`; first 4 bytes of `SHA-256("abc")` `== 0xBA7816BF`; standard test vector for CRC-64/XZ. Three assertions pinned independently, **forbidden from reading back from code under test** |
| 4 | **Non-linear Arm Sampling**: Arm B sampled via birthday method, ratio of collisions to 2⁻³² expectation falls within [0.5, 2] |
| 5 | **Header Length Sweep**: three tiers of 86 / 95 / 99 bytes (then-active value, plus 9 bytes from D23 (Journal Role & Format) settled item 7, plus 4 bytes for reverse chain). **Conclusion forbidden from shifting with length**—if it changes, formulated as conditional |
| 6 | **Isogenic comparisons must be identical**: two evaluations on identical input must match bit-for-bit, collision opportunities count 0 (same archetype as E51 (Number of Reverse Chain Collision Opportunities) Criterion 1) |

### What Constitutes Failure (Hardcoded Pre-Run, Unmodified Post-Run)

| Clause | Verdict Upon Triggering |
|---|---|
| Positive control (Arm C) achieves full rank everywhere | Criteria lack discriminative power, **entire round scrapped** |
| **Arm A exhibits window with rank < 32** | **Record faithfully.** E61 (Uniformity of Reverse Chain Hash Algorithm) does not presuppose CRC32C is superior—that would indicate it does not outperform cryptographic hash in this use case |
| Arm B sampled collision ratio falls outside [0.5, 2] | Record faithfully, noting "truncation uniformity unproven" |
| Three header length tiers yield conflicting conclusions | Faithfully juxtapose, formulating conclusion conditionally; **forbidden from cherry-picking one tier** |
| Arm A and Arm C yield identical rank spectrum | This dimension is dead, **entire round scrapped** |

### The Half It Cannot Answer

1. **Distribution of instance IDs themselves**. Rank criterion answers "collisions are exactly 0 when differences fall within window",
   unable to answer "whether instance IDs happen to be reused"—that belongs to D23 (Journal Role & Format) settled item 9.
2. **Hardware acceleration cost unmeasured**. CRC32C has SSE4.2 instructions while SHA-256 has SHA-NI, both fast;
   **E61 (Uniformity of Reverse Chain Hash Algorithm) measures zero cycles**—it answers only uniformity.
3. **Whether sharing the identical function causes common-mode failure**: Arm A shares function with ciphertext checksum in D19 (Block Pointer Structure & Width Budget) settled item 2;
   bytes covered and purposes differ, but the risk of "identical function" resides in C48 (Multiple Verification Paths Sharing Same Premise),
   which E61 (Uniformity of Reverse Chain Hash Algorithm) does not answer.

### Prerequisites

None. Pure arithmetic, independent of file systems and QEMU.

### Measurements (2026-08-31)

#### Metric 1: Rank Spectrum—Arm A Achieves Full Rank on Every Window, Arm C on Fewer Than Half

⚠️ **Three header widths taken from 86 / 95 / 99 active during 2026-08-31 run**; active header is **487** (D23 (Journal Role & Format) settled item 4: ten fields 78 + three settled increments 17 + intra-publish sequence 4 + new root segment 364 + fsid 8 + MAC 16). Arm A achieves full rank across all three tiers, while Arm C achieves full rank on fewer than half ⇒ conclusion insensitive to tier; **source and artifacts frozen on legacy tiers, re-run before citing**.

| Arm | Header 86 | Header 95 | Header 99 | Worst Rank | Worst Window Collision Count |
|---|---|---|---|---|---|
| **Arm A Full CRC32C** | **401 / 401** | **473 / 473** | **505 / 505** | **32** | **0** |
| Arm C CRC-64 Truncated to 32 (Positive control) | 193 / 401 (48.1%) | 226 / 473 (47.8%) | 237 / 505 (46.9%) | **28** | **15** |

⇒ **Arm A experiences zero collisions on non-zero differences within any 32-bit window in variable region—this is a calculated 0, not an unobserved 0.**
Full rank is equivalent to enumerating all 2³² differences in that window.

#### Metric 2: Actual Pattern (Two Timelines Differ Only on 4-Byte Instance ID)

| Arm | Header 86 | Header 95 | Header 99 |
|---|---|---|---|
| **Arm A** | Rank 32, Collisions **0** | Rank 32, Collisions **0** | Rank 32, Collisions **0** |
| Arm C | Rank 31, Collisions 1 | Rank 30, Collisions 3 | Rank 31, Collisions 1 |

#### Metric 3: Sampling (Sole Path for Arm B), 2^20 Headers, Expected 128 Collisions

| Arm | Header 86 | Header 95 | Header 99 |
|---|---|---|---|
| Arm A | 114 (Ratio 0.891) | 114 (0.891) | 114 (0.891) |
| **Arm B SHA-256 Truncated to 32** | 148 (**1.156**) | 116 (**0.906**) | 104 (**0.813**) |
| Arm C | 126 (0.984) | 142 (1.109) | 126 (0.984) |

⇒ **Arm B truncation is uniform**, with all three tiers falling inside [0.5, 2] in Criterion 4.

#### Clause-by-Clause Verdict on Criteria and Failure Clauses

| # | Verdict |
|---|---|
| 1 Rank Criterion | **Pass**: Arm A achieves full rank across all 1379 windows without exception |
| 2 Positive Control Must Fail | **Pass**: Arm C worst rank 28, over half of windows lack full rank ⇒ rank criterion discriminative |
| 3 Absolute Value Assertions | **Pass**: `CRC32C("123456789")=0xE3069283`, `CRC-64/XZ=0x995DC9BBDF1939FA`, first 4 bytes of `SHA-256("abc")` `=0xBA7816BF`, pinned independently |
| 4 Non-linear Arm Sampling | **Pass**: 0.813 ~ 1.156 |
| 5 Three Header Length Tiers | **Pass**: conclusion independent of length, formulating conditionally unneeded |
| 6 Isogenic Identity | **Pass**: all three arms true |

### Measurement Basis and Re-run

- Code `scripts/experiments/e61_chain_hash.rs`
  (`cargo run --release --bin e61-chain-hash`).
- Artifact `e61-chain-hash-2026-08-31.out` (24 lines, terminating line `emitted=24`).
- ⚠️ **Three rounds byte-identical, but proves solely "absence of hidden state", not statistical stability**:
  E61 (Uniformity of Reverse Chain Hash Algorithm) is a deterministic model (no I/O, no concurrency, no clock inputs); running identical binary thrice must produce identical results
  ⇒ N=1 and N=3 convey identical information. **Must never impersonate evidence strength of real stress tests.**
- Three hash implementations **written independently**, avoiding reusing E40 (Ciphertext Checksum Bit Width)—
  reuse equates to sharing code between object under test and its correctness evidence. Pinned independently via standard test vectors in Criterion 3.

### Affected Decisions

| Decision Item | Relationship | Retrospective |
|---|---|---|
| D23 (Journal Role & Format) settled item 11 | Supports | 2026-10-04 unaffected: updated header width, new root segment, and named items to current values only, conclusions and artifacts unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-10-04 unaffected: updated header width, new root segment, and named items to current values only, conclusions and artifacts unchanged |

## Historical Versions

Historical entries for E61 (Uniformity of Reverse Chain Hash Algorithm) are consolidated in [experiments-history.md](../experiments-history.md).
