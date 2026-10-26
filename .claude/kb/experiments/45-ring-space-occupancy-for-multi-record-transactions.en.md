## E45 Ring Space Occupancy for Multi-Record Transactions — ⚠️ Run but **Conclusions Invalidated** (2026-08-30 Attack Round): Alignment Rule Stricter than I-8.2 (Record Header Does Not Cross Atomic Unit), Multiple Was Artifact of That Rule

[中文](45-事务跨多条记录的环占用.md) · **English** · [日本語](45-トランザクションの複数レコード跨ぎによるリング占有.ja.md)

**It retests D23 (Journal Role & Format) settled item 7**, because the justification for that item was overturned,
and **the direction of overturning was a mutual transposition**: previously written "one record per transaction inflates the ring", whereas actually "spanning multiple records inflates the ring".

⚠️ **A mutual transposition of this magnitude requires three independent verifications.** A single arithmetic recheck is insufficient——
if the step overturning conclusion from side A to side B was itself mistaken, flipping it back merely errs in a different direction.
E45 (Ring Space Occupancy for Multi-Record Transactions) assigns an **isolated path sharing no code** to each of the three steps under `.claude/singlefs-ai-sop/rules/evidence-discipline.md`.

### Verified Premises (Investigated 2026-08-30)

| Source | Verbatim |
|---|---|
| D23 (Journal Role & Format) settled item 4 settled | "Record header **lands entirely within an atomic unit**", header 84 bytes |
| I-8.2 (Record Header Does Not Cross Atomic Unit) | Restrains only the **record header**, **no clause prohibits a large record** |
| I-8.1 (Ring Geometry Sufficiently Large) | "Ring size >= F * **any transaction's** worst-case journal occupancy", F >= 2 |
| E23 (Journal Geometry) alignment cost table | pbs=512: 1 named item 140 B -> **512 B** (3.7x); 10 items 644 B -> **1024 B** (1.6x); 100 items 5684 B -> **6144 B** (1.1x) |

### What to Test

**For the same transaction, packed into how many records, how many bytes it occupies in the ring.** The independent variable is "how many named items packed per record".

⚠️ **Costs on both sides must be measured, not solely the overturned side**:

| Arm | Where Its Cost Should Land |
|---|---|
| **One record per transaction** | Records grow larger ⇒ **more named items lost when a record is torn**. This cell was only asserted prior to E45 (Ring Space Occupancy for Multi-Record Transactions), never measured |
| **Spanning multiple records** | Each record rounds up independently to an atomic unit ⇒ **ring occupancy amplified**, further amplifying the ring via I-8.1 (Ring Geometry Sufficiently Large) |

### Path and Conclusion Registry

Format per `.claude/rules/format-evolution.md` "Path and Conclusion Registry on Experiment Pages".

| # | Path | How Calculated | Source Code Location | Shared Items Read |
|---|---|---|---|---|
| 1 | **Byte-level placement** | Actually placing records sequentially into a byte-addressable ring, advancing cursor, measuring how far cursor progressed | `scripts/experiment-apparatus/e45_span_ring_cost.rs:51` `fn laid_out_bytes` | `BASE_RECORD_HEADER_BYTES` (84), `NAMED_ITEM_BYTES` (56), **entire record rounded up to `atomic_unit_bytes`** (internal to `Ring`) |
| 2 | **Closed-form arithmetic** | `record_count * ceil((header + items_per_record * item_width) / unit) * unit`, written independently, not calling Path 1 | `scripts/experiment-apparatus/e45_span_ring_cost.rs:63` `fn closed_form_bytes` | Same three as Path 1 (Byte-level placement): `BASE_RECORD_HEADER_BYTES`, `NAMED_ITEM_BYTES`, **`.div_ceil(atomic_unit_bytes)` rounding up entire record** |
| 3 | **Matching published table of E23 (Journal Geometry)** | Asserting this model reproduces the three cells 140 -> 512, 644 -> 1024, 5684 -> 6144 | `scripts/experiment-apparatus/e45_span_ring_cost.rs:78` `fn e23_unpadded_record_bytes`, `:79` `fn e23_padded_record_bytes` | Same three as Path 1 (Byte-level placement) |

Cross-path comparison assertion pinned in unit test `all_three_paths_agree_on_every_cell` of `scripts/experiment-apparatus/e45_span_ring_cost.rs` (`assert_eq!(laid_out_bytes(...), closed_form_bytes(...))`).

⚠️ **This table reveals precisely why the conclusions of this experiment are invalidated, visible cell by cell**: The shared items column is completely identical across all three paths, among which "entire record rounded up to `atomic_unit_bytes`" was an unsettled rule in the repository, far stricter than I-8.2 (Record Header Does Not Cross Atomic Unit)——which only restrains the record header. Reading it together and erring together, all 10 cells remained identical. **Prior to filling that column, these three paths appeared to share no code.**

### Criteria

- **All three paths must match cell by cell**. Any inequality ⇒ entire round invalidated, fix model before discussing conclusions.
- **Claim**: For a 12-item transaction on a 4096 unit, ring occupancy of "split into one per item" >= **10x** that of "packed into one record".
- **Reverse**: Torn write loss for "one record per transaction" (named items lost when one record is destroyed) must **grow linearly with items per record**——
  this is the cost on its own side; failing to measure it implies the model is not modeling it.

### Positive Controls (Two, Neither May Be Omitted)

1. **Set atomic unit to 1 byte** (canceling round-up) ⇒ extra occupancy of splitting into multiple records must be **exactly 0**.
   Otherwise the model measures something other than "alignment waste", **invalidating entire round**.
2. **Verification paths must prove ability to go red**: Injecting a known deviation into Path 1 (e.g., omitting one round-up),
   Paths 2 and 3 must immediately judge inequality. Failing to catch ⇒ those two are decorative, invalidating all reported "three paths agree".

### Failure Clauses

- If occupancy of "split into one per item" is **not** significantly greater than "packed into one", the overturning on 2026-08-30 was itself flawed,
  **faithfully record and flip back the justification for D23 (Journal Role & Format) settled item 7**.
- If the three paths disagree, **draw no conclusions**.
- ⚠️ **Must carry absolute value assertions**: Byte count for each cell must be independently calculated from `(header, item_count, item_width, unit)`,
  never read back from tested code.

### ⚠️ Conclusions Invalidated: Entire Multiple Produced by an Undefined Alignment Rule

**The model implemented "record header lands entirely within an atomic unit" as "entire record begins at unit boundary".**
I-8.2 (Record Header Does Not Cross Atomic Unit) verbatim requires only "record header lands entirely within an atomic unit"——allowing starting offset `o` satisfying `o mod unit <= unit - 97`,
**not `o mod unit == 0`**. Placing twelve 1-item records back-to-back under literal rules (153 bytes each, 1836 bytes total):
**all 12 headers land entirely within the same 4096 unit, not a single one violating I-8.2 (Record Header Does Not Cross Atomic Unit)** ⇒ occupancy is 4096 bytes.

| 12-Item Transaction, 4096 Unit | Occupancy | Relative to "One Record Per Transaction" |
|---|---|---|
| One record per transaction | 4096 B | 1x |
| Split into 12 records, **under strict rule of E45 (Ring Space Occupancy for Multi-Record Transactions)** | 49152 B | 12x |
| Split into 12 records, **under literal I-8.2 (Record Header Does Not Cross Atomic Unit)** | **4096 B** | **1.00x** |

⇒ **That 12x collapsed from "12x" to "1.00x". The claim of E45 (Ring Space Occupancy for Multi-Record Transactions) does not hold.**

### ⚠️ The Claim of "Three Paths Sharing No Code" Also Does Not Hold

| Path | What It Actually Is |
|---|---|
| 1 Byte-level placement / 2 Closed-form arithmetic | **Two ways of writing the same specification**. Closed form is merely the algebraic form of the loop; both share the same constants and **same flawed alignment rule** ⇒ they make this mistake together, matching cell by cell |
| 3 Matching published table of E23 (Journal Geometry) | **Not cross-binary reproduction**: 140 / 644 / 5684 are **not** in `e23-journal-geom-2026-08-29.out` (the sole 140 in that artifact is `forced=140`), they were **hand-calculated tables in the text** of E23 (Journal Geometry), copied into E45 (Ring Space Occupancy for Multi-Record Transactions) as three literals. Furthermore, it only anchors the two constants `BASE_RECORD_HEADER_BYTES` and `NAMED_ITEM_BYTES`, offering **zero discriminative power over the multiple under test** |

⇒ **The three paths agreed because all three shared the same false premise.**
`.claude/singlefs-ai-sop/rules/evidence-discipline.md`: Such agreement is not evidence.
**The discipline that "mutual transposition requires three verifications" was sound; the flaw was that the three verifications were not genuinely independent.**

### ⚠️ Four Other Points Identified in Attack Round

1. **The arm tested was not the arm decided.** The "spanning multiple records" arm in D23 (Journal Role & Format) settled item 7 **did not prescribe how many items per record**;
   E45 (Ring Space Occupancy for Multi-Record Transactions) took `items_per_record = 1` (one per item) as representative, which was a strawman. Real implementations pack as full as possible
   ——when one record packs 71 items under 4096 units, both arms have **completely identical byte counts**.
2. **Spanning multiple records is a superset of one record per transaction.** Formats allowing multiple records permit implementations to issue strictly one record
   ⇒ Rejecting a superset based on its worst usage treats "you could use it wrong" as "forbidden to exist".
3. **The causal link of "inflating the ring" misses the ring entirely.** E23 (Journal Geometry) settled that under pbs=4096, rings >= 4 MiB avoid forced checkpoints,
   whereas the I-8.1 (Ring Geometry Sufficiently Large) floor calculated by E45 (Ring Space Occupancy for Multi-Record Transactions) is 8 KiB / 96 KiB——**both two orders of magnitude smaller than 4 MiB**.
   What binds the ring is checkpoint geometry, not single-transaction floors.
4. **Two dead assertions and a constant function**: `assert_eq!(laid_out_bytes(1,1,u), laid_out_bytes(1,1,u))` has identical expressions on both sides,
   never going red; `transactions_lost_when_torn` returns literal 1, whose test merely asserts that a constant function is constant.
   ⇒ The conclusion that "no reverse cost was found" was **coded and self-tested**, not verified.

### ⚠️ A Physical Fact Untouched by E45 (Ring Space Occupancy for Multi-Record Transactions) Yet Decisive in Direction

`header_csum` (32 bytes) **covers only the record header** (E23 (Journal Geometry) field table verbatim: "premise distinguishing header landed from tail unlanded"),
while the record **payload carries no defined independent checksum**.
⇒ **For a large record spanning multiple atomic units, payload integrity cannot be judged.**
Both E42 (Number of Records per Transaction) and E45 (Ring Space Occupancy for Multi-Record Transactions) enumerated torn writes treating "record" as an indivisible atomic unit,
an assumption with **physical basis only when the entire record lands within a single atomic unit**.
⇒ **The premise of the inference "one record per transaction eliminates tail-dropping logic" was never proven.**

### Original Empirical Measurements (Measurement Basis Invalidated, Retained for Archive)

All numbers below were computed under the **unsettled strict rule** that "every record starts at a unit boundary", **forbidden from further citation**.

### Original Measurements (2026-08-30, N=5 Rounds Byte-for-Byte Identical; All 7 Mutations Caught, 0 Blind Spots)

**All three paths agreed cell by cell** (all 10 cells `paths_agree=true`), **Verification B reproduced the three published cells of E23 (Journal Geometry)**
(140 -> 512, 644 -> 1024, 5684 -> 6144), **both positive controls passed**. ⇒ Conclusions could be drawn.

#### Claim Stood: Spanning Multiple Records Inflates Ring, Worst-Case 12x

A 12-item transaction (D25 (Target Workload Priority) coarse granularity: 8 leaves + 4 ancestors):

| Items Per Record | Record Count | 512 Unit Occupancy | 4096 Unit Occupancy | Minimum Ring Under I-8.1 (Ring Geometry Sufficiently Large) (F=2, 4096 Unit) |
|---|---|---|---|---|
| **12 (one record per txn)** | 1 | **1024 B** | **4096 B** | **8 KiB** |
| 8 | 2 | 1536 B | 8192 B | 16 KiB |
| 4 | 3 | 1536 B | 12288 B | 24 KiB |
| 2 | 6 | 3072 B | 24576 B | 48 KiB |
| **1 (one per item)** | 12 | **6144 B (6x)** | **49152 B (12x)** | **96 KiB** |

⇒ **The direction of overturning on 2026-08-30 was correct**: the finer the split, the larger the ring occupancy, worst-case 12x.

#### Size Scan: Conclusion is "Never Worse", Not "Always 12x"

⚠️ **First version ran only a single size of 12 items**, insufficient to support "worst-case 12x".
In the attack round on 2026-08-30, the local leg pointed out "both paths might be identical on small transactions", prompting a size scan (4096 unit):

| Transaction Item Count | Packed Into One Record | Split Into One Per Item | Multiple |
|---|---|---|---|
| **1** | 4096 B | 4096 B | **1.0x (completely identical)** |
| 4 | 4096 B | 16384 B | 4.0x |
| **12 (target workload)** | 4096 B | 49152 B | **12.0x** |
| 64 | 4096 B | 262144 B | 64x |
| 1024 | 61440 B | 4194304 B | 68x |
| 62 000 | 3.31 MiB | 242 MiB | **73x** |

⇒ **The multiple is >= 1 across all sizes and monotonically non-decreasing with size** ⇒ **"one record per transaction" is never worse**.
⚠️ **The 1-item cell is identical on both sides, wasting 96% on both**——that is
the cost of the settled axis "one record per fsync", **not the cost of this choice**, and must not be attributed to either side.

#### ⚠️ Reverse Cost **Sought But Not Found**

Intuition suggests "larger records lose more upon torn write". **Measured the wrong thing**: Under D23 (Journal Role & Format) settled item 7
"discard uncommitted tails", incomplete transactions spanning multiple records are **discarded entirely** ⇒ **both sides lose exactly one transaction**,
and transactions are inherently atomic, making discarding an uncommitted transaction **correct behavior, not a cost**.
`transactions_lost_when_torn` is constantly 1 across all five tiers, pinned by an assertion to prevent treating "losing more" as a reverse cost in the future.

⇒ **No quantifiable cost has yet been found on the side of "one record per transaction".**
⚠️ **However, the attack round's local leg identified two real costs belonging to "one record per transaction" not modeled in E45 (Ring Space Occupancy for Multi-Record Transactions)**:

| Cost | Form | Status |
|---|---|---|
| **Ring wraparound** | If a large record cannot cross ring tail, it requires contiguous space; when remaining space is split into two fragments by ring tail, it cannot write even if total space suffices | **Not modeled**. E45 (Ring Space Occupancy for Multi-Record Transactions) uses an infinite cursor without wraparound |
| **Must buffer entire record first** | Packed into one ⇒ entire record must form in memory before write initiates; for 62,000 items that is **3.31 MiB** per record. Splitting allows streaming | **Not modeled**. E45 (Ring Space Occupancy for Multi-Record Transactions) counts bytes only, touching neither memory nor issue timing |

⚠️ **Remember point 2 in particular**: It is the sole cost whose **direction is unambiguously disadvantageous** to "one record per transaction",
while E45 (Ring Space Occupancy for Multi-Record Transactions)'s conclusion (never worse) holds only under the single dimension of **byte occupancy**.
⇒ **Never interpret "never worse" as "suffers no disadvantage on any side".**

⚠️ The local leg also suggested "splitting saves space under mixed small transactions", **which does not hold**:
The size scan shows both sides are **completely identical** at 1 item (both one unit), and splitting never saves space.
What it truly pointed toward was a third form——**multiple transactions packed into one record**——which was neither of E45 (Ring Space Occupancy for Multi-Record Transactions)'s two arms nor ever modeled.

#### Positive Controls

| Control | Result |
|---|---|
| **Unit = 1 byte** (canceling round-up) | Extra occupancy of splitting **equals exactly the 11 extra headers** (1067 B), with zero alignment waste ⇒ model indeed measures alignment waste |
| **Injecting deviation into Path 1** (omitting tail alignment) | Path 2 immediately judges inequality (769 vs 4096) ⇒ verification paths are not decorative |

#### ⚠️ A Blind Spot: Alignment Conceals Measurement Basis Discrepancies

Verification B originally asserted only the three cells **after** rounding up. Changing an 84-byte header to 97 bytes,
those three cells remained 512 / 1024 / 6144 after rounding up——**none of the three assertions turned red** (empirically tested in mutation `M3_E24_calibration_mixed_with_txn_fields`).
⇒ **Matching numbers only "after rounding up" is insufficient to prove measurement basis consistency**; one must simultaneously match raw byte counts before rounding up (140 / 644 / 5684).
**This is the third form of the identical failure class as "both sides adopting normative values".**

**Measurement basis**: 12-item transactions, 12 / 8 / 4 / 2 / 1 items per record, 512 and 4096 atomic units;
Header = 84 (E23 (Journal Geometry)) + 9 (txn ID + commit mark) + 4 (reverse chain 32-bit) = 97 bytes; item width 56 bytes.
Code `scripts/experiment-apparatus/e45_span_ring_cost.rs`
(`cargo run --release --bin e45_span_ring_cost`),
raw output `e45-span-ring-cost-2026-08-30.out`,
mutation table `scripts/mutation/tables/e45_span_ring_cost.tsv`.
**9 unit tests, all 7 mutations caught one by one, 0 blind spots.**

### Prerequisites

None. Pure counting model, runnable directly.

### ⚠️ Round 2 Attack Returned Only One Leg, Leaving Two Unmodeled Costs

Under `.claude/rules/three-way-inference.md`, Round 2 attack **lacked two cloud legs** (session ended without return)
⇒ **Forbidden from recording as "tripartite agreement"**. Among four points from local leg (stance: finding counterexamples), two stood,
and **both represent costs on the side of "one record per transaction", neither modeled in E45 (Ring Space Occupancy for Multi-Record Transactions)**:

| Counterexample | Why It Stands | What Observing It Requires |
|---|---|---|
| **Ring wraparound requires contiguous space** | If a large record cannot cross ring tail, it requires contiguous space; when ring tail has insufficient space, writes fail even if total free space suffices | Model ring wraparound, measuring bytes and occurrences "forced to pad/wait due to inability to cross ring tail" |
| **Cannot issue in streaming fashion** | One record per txn ⇒ entire record must accumulate in memory before writing starts; multi-record allows streaming | Measure record bytes of worst-case transaction = peak memory residency at commit time |

⚠️ The other two points did not stand: **"Ring free space fragmentation"** does not apply——journal rings advance sequentially,
and free space is a single contiguous segment (modulo ring tail), free of fragmentation; **"Splitting saves space on small transactions"** is backwards——
at 1 item both arms are **completely identical** (both occupy one atomic unit, both waste 70%),
which is the cost of "one record per fsync", not the cost of this choice.

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 7 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 4 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30
- Created and run. Source: justification for D23 (Journal Role & Format) settled item 7 overturned,
  user requested "retest decision; such wide-ranging mutual transposition requires three verifications".
