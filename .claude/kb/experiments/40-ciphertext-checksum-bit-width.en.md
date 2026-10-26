## E40 Ciphertext Checksum Bit Width —— Run (2026-08-30, N=5 rounds byte-identical): 32-bit holds under original derivation input set, margin is 3.37×

[中文](40-密文校验和位宽.md) · **English** · [日本語](40-暗号文チェックサムのビット幅.ja.md)

**What it asks**: D19 (Block Pointer Structure & Width Budget) settled item 2 "how many bits should ciphertext checksum take".

⚠️ **The existing derivation does not count as evidence**: D19 (Block Pointer Structure & Width Budget) body self-states
"**this is a one-legged derivation, not reproduced via verification path, and the corruption rate number is an assumption, not measurement**"—
assuming 10 TB pool / weekly scrub / 10 years ≈ 1.3×10¹² block reads, corruption rate 1e-9/block read,
requiring undetected error expectation < 1e-6, obtaining n > 30.3 bits. **Two out of three inputs were fabricated.**

### What to Test, Criteria, Failure Clauses

- **What to test** (two segments, neither requiring file system):
  1. **Arithmetic segment**: Express "undetected expectation" as an explicit function of `block reads × corruption rate × 2⁻ⁿ`, perform sensitivity sweep on `(pool capacity, scrub period, service lifespan, corruption rate)`, providing **how many times each input must individually err to push conclusion from 32-bit to 64-bit**.
  2. **Collision segment**: Feed candidate checksums (CRC32C / CRC64 / xxh3-64 / truncated SHA-256) with **the kinds of corruption encountered in this project**—single-bit flip, single-sector block zeroing, single-sector misdirected write, full-block replacement with another legal block—measure undetected rate, comparing against theoretical value 2⁻ⁿ.
- **Criteria**:
  - If sensitivity sweep shows 32-bit satisfies < 1e-6 across **all realistic input ranges**, take 32-bit; D19 (Block Pointer Structure & Width Budget) derivation holds.
  - If realistic inputs exist where 32-bit is insufficient, provide minimum bit width to make it hold.
  - Candidate algorithms whose measured collisions deviate from 2⁻ⁿ by **more than one order of magnitude** are **eliminated**—their real strength is not nominal strength.
- **Positive control**: Feed a **deliberately weak** checksum (such as 8-bit sum check); undetected rate must be significantly non-zero and close to 2⁻⁸—otherwise "32-bit undetected is 0" cannot distinguish whether strength is sufficient or corruption was simply never injected.
- **Failure clauses**: If positive control detects no difference, **implementation is faulty, invalidate entire round**; if sensitivity sweep shows conclusion is dominated entirely by fabricated corruption rate, **faithfully record "this number cannot be settled, requires field data"**, never cherry-picking attractive inputs to force 32-bit.
- ⚠️ **Must include absolute value assertions**: Injected corruption count is directly computed by construction, and "undetected rate" denominator must equal actual injection count, not read back from tested code.

### ⚠️ The Half It Cannot Answer

Corruption rate 1e-9/block read **has no local host source**. What E40 (Ciphertext Checksum Bit Width) can do is "given corruption rate, what width should be taken", **cannot** answer "what the corruption rate is". ⇒ Conclusion must be formulated conditionally, and when cited to support D19 (Block Pointer Structure & Width Budget) settlement, that condition must be cited together.

### Prerequisites

None. Pure CPU experiment.

### Host Testing (2026-08-30)

#### Arithmetic Segment: 32-bit Suffices, but **Margin Is Only 3.37×**

| Quantity | Value |
|---|---|
| Block reads (10 TB / 4 KiB × 10 years ÷ weekly scrub) | **1.273×10¹²** (Reproduces order of 1.3×10¹² stated in D19 (Block Pointer Structure & Width Budget) original text) |
| Corrupted reads (× 1e-9) | **1273** |
| Minimum bit width satisfying undetected expectation < 1e-6 | **31** (D19 (Block Pointer Structure & Width Budget) original text "n > 30.3", rounding matches) |
| 32-bit undetected expectation | **2.96×10⁻⁷** |
| **32-bit margin = target ÷ actual** | **3.37×** |

**All five inputs enter undetected expectation linearly** (pool capacity, corruption rate, service lifespan directly proportional; scrub period, block size inversely proportional)
⇒ **If any single one errs by 3.37×, conclusion is pushed from 31 bits to 33 bits**. Verified item by item: modifying any one produces identical shift.

Corruption rate swept independently (the one D19 (Block Pointer Structure & Width Budget) conceded was "assumed not measured"):

| Corruption Rate | Minimum Bit Width |
|---|---|
| 1e-9 (Value taken by D19 (Block Pointer Structure & Width Budget)) | **31** |
| 1e-8 (10× error) | **34** |
| 1e-7 | **37** |
| 1e-6 (1000× error) | **41** |
| 1e-5 | **44** |

⇒ **Conclusion of 32 bits cannot withstand one order of magnitude of input error.**

#### Collision Segment: Prove Law "Undetected Rate = 2⁻ⁿ" First, Then Extrapolate

⚠️ **32-bit undetected rate cannot be measured within feasible trial counts** (~4×10⁹ trials needed to wait for one undetected fault).
Hence testing focuses on measurable range **truncated to 8 / 12 / 16 bits**:

| Bit Width | Measured Undetected (ppm, four corruption classes) | Theoretical 2⁻ⁿ (ppm) |
|---|---|---|
| 8 | 3212 / 3960 / 4076 / 3908 | 3906 |
| 12 | 0 / 244 / 260 / 232 | 244 |
| 16 | 0 / 8 / 12 / 12 | 15.3 |
| 32 (CRC32C full width) | All 0 (250,000 injections) | 0.00023 |
| 64 (CRC64 full width) | All 0 (250,000 injections) | 5.4×10⁻¹⁴ |
| 32 (Truncated SHA-256) | All 0 (25,000 injections) | 0.00023 |

**Law holds** (measurements across 8 / 12 / 16 tiers match order of 2⁻ⁿ) ⇒ 32 / 64 bit rows are **extrapolations, not measurements**.

#### Positive Control Holds in Both Directions

| Arm | Byte Swap (Same Batch of Corruptions) |
|---|---|
| 8-bit sum check (deliberately weak arm) | **248951 / 248951 all undetected**—swapping two bytes leaves sum unchanged |
| CRC32C full width | **0 undetected** |

⇒ Measurement possesses discriminative power: one arm misses all, one arm catches all, corruptions indeed injected.

#### A Discovery Unique to E40 (Ciphertext Checksum Bit Width): **Truncated CRC Is Not "Shorter CRC"**

| Corruption Class | CRC32C Full Width | CRC32C Truncated to 8-bit |
|---|---|---|
| Single-bit flip | **0 undetected** (Structural property of CRC: single bit guaranteed caught) | **3212 ppm undetected** |

⇒ **Truncation completely sheds structural properties of CRC**, degrading it into an 8-bit random hash.
⚠️ This directly impacts D19 (Block Pointer Structure & Width Budget) value selection:
If ciphertext checksum is settled as "CRC32C truncated to n bits", when n < 32 not only does probability worsen,
but **single-bit flip downgrades from "guaranteed caught" to "probabilistically caught"**. Settled 4 bytes in D9 (Encryption) happens to be full width of CRC32C, avoiding truncation.

⚠️ **8-bit sum check unexpectedly had 0 undetected for single-bit flip** (flipping one bit necessarily alters byte sum),
yet had 100% undetected for byte swap—**"which algorithm is stronger" is not a scalar**, requiring evaluation by corruption class.

### Conclusion and the Half It Cannot Answer

**Under D19 (Block Pointer Structure & Width Budget)'s own input set, 32-bit holds** (minimum 31 bits),
**across the three tested algorithms, none on collision side were eliminated** (deviations from 2⁻ⁿ across 8 / 12 / 16 tiers all within one order of magnitude).

⚠️ **Among four candidates listed at inception, xxh3-64 was never implemented and never run** (pointed out in 2026-08-30 round 2 review).
⇒ It was **neither verified nor eliminated**, forbidden to be included in "none were eliminated".
Tested algorithms were CRC32C (full width and 8 / 12 / 16 bit truncations), CRC64-ECMA, truncated SHA-256, plus positive control 8-bit sum check.

⚠️ **However, fragility of that derivation has been quantified**: margin is **3.37×**,
and one of its three inputs (corruption rate 1e-9) was conceded as assumption in D19 (Block Pointer Structure & Width Budget).
**E40 (Ciphertext Checksum Bit Width) cannot answer "what the corruption rate is"**—that requires field data, no local host source.
⇒ Conclusion must be stated conditionally: **Given 1e-9, 32-bit suffices; if corruption rate reaches 1e-8, 34-bit is required.**

**2026-08-30 User settled based on this: take 32 bits** (D19 (Block Pointer Structure & Width Budget) settled item 2 settled).
E40 (Ciphertext Checksum Bit Width) thus provided a **testable reopening criterion**: once field corruption rate exceeds **3.37e-9**, that settlement reopens immediately.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e40_checksum_width.rs`
  (`cargo run --release --bin e40_checksum_width`, ~11 seconds), artifact
  `e40-csum-width-2026-08-30.out` (48 lines, closing line `emitted=48`).
  Rerun passes E40 (Ciphertext Checksum Bit Width) number to `scripts/experiment-apparatus/replay.sh`.
- **10 unit tests all green; 9 mutation items all caught** (`scripts/mutation/tables/e40_checksum_width.tsv`).
  ⚠️ **Do not write "zero blind spots"** (noted in 2026-08-30 round 2 review): mutations only cover **functions with written assertions**;
  it proves "code with written assertions was tested by mutations", **not "every piece of code covered by conclusion was tested"**.
- ⚠️ **Only one positive control pair was made** (8-bit sum check vs CRC32C):
  CRC64 and truncated SHA-256 **were not individually proven to have discriminative power**.
  `.claude/singlefs-ai-sop/rules/test-discipline.md` "positive control must run for every tested arm" was not fully satisfied.
- **Three algorithm implementations pinned with public test vectors**: `crc32c("123456789") = 0xE3069283`,
  `cyclic_redundancy_check_64("123456789") = 0x995DC9BBDF1939FA`, SHA-256 empty string and first 8 bytes of `"abc"`.
  ⚠️ **Cannot be omitted**: if algorithm implementation is wrong, undetected rate shifts systematically, and four arms **shift together**, undetectable by mutual comparison.
- **Denominator is actual injection count**, directly counted by construction; **trials that did not alter bytes are excluded**
  (byte swap hitting identical bytes, full block replacement hitting identical block)—measured denominator for byte swap class was 248951, not 250000.
- **Block size set to 512 bytes** (one sector); corruption classes also constructed at sector scale. Changing to 4 KiB does not alter conclusion direction but was unrun.
- **Deterministic PRNG** (xorshift64\*, pure integer), 5 seeds, invariant across machines.
- ⚠️ **Does not test attacker model**: D19 (Block Pointer Structure & Width Budget) threat model is verbatim "media corruption not attacker",
  and E40 (Ciphertext Checksum Bit Width) models accordingly. Resisting attackers requires MAC (D9 (Encryption) settled at 128 bits), outside this field.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 11 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-29
- Documented. Source: D19 (Block Pointer Structure & Width Budget) settled item 2, user requested "run an empirical test".
