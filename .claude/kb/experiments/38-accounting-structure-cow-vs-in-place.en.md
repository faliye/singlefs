## E38 Accounting Structure: COW vs In-Place — Run (2026-08-29); ⚠️ Recovery Cost Column Withdrawn (2026-08-30)

[中文](38-记账结构COW还是原地.md) · **English** · [日本語](38-記帳構造-COW-かインプレースか.ja.md)

**What it asks**: Directional settlement of D22 (Synthesizing Unit Atomicity) settled item 3 "adopt COW, hanging beneath parent pointer",
with rationale "units are self-contained ⇒ the parent pointer path requires no extra positioning convention"——**this step was an inference, never tested.**

⚠️ **This item forks at the mkfs stage**: COW ⇒ accounting units hang beneath parent pointer, obtaining a new address each time;
In-place ⇒ mkfs must reserve fixed locations on disk in advance. **Running first and changing later is equivalent to redoing mkfs.**

### Verified Premises (Investigated 2026-08-29)

| Source | Verbatim |
|---|---|
| D21 (Boundary Between Authoritative and Derived States) settled | Accounting belongs to **authoritative state** (authoritative state = units + accounting + root) |
| `.claude/rules/fs-design.md` cell 1 | Accounting must be **incrementally maintained at transaction commit time**, post-hoc traversal forbidden |
| D22 (Synthesizing Unit Atomicity) settled item 3 original text | On in-place side: "Accounting is authoritative state without copies; once checksum goes red the sole reconstruction path is full-disk scan ⇒ **single accounting block torn write = single mount-time full-disk scan**" |
| D8 (Core Index Structure) settled item 1 settled | Accounting entries are **idempotent complete values**, merging by "last-write-wins", without incremental Δ |

### What to Test, Criteria, Failure Clause

- **What to test**: Under the identical workload (N transactions, each modifying M statistics), for both arms respectively:
  ① The **extra blocks written** per commit (accounting itself + dirty ancestor path);
  ② The **recovery cost** after a single accounting block torn write (how many blocks must be read to reconstruct correct accounts).
- **Criteria**:
  - **Write amplification**: Extra blocks written per commit in COW arm = accounting leaves + ancestor path to root; in-place arm = accounting block itself (+ slot alternation).
    If COW arm write amplification **does not exceed 2x the in-place arm** under coarse-grained workloads settled in D25 (Target Workload Priority), the directional settlement stands.
  - **Recovery cost**: In-place arm after torn write must be on the order of "full-disk scan"; COW arm must be O(path to root).
    ⚠️ **This criterion is the true basis of directional settlement; write amplification is merely its price.**
- **Positive control**: Replace "new address each time" in COW arm with "overwrite same address in-place",
  write amplification must drop noticeably——otherwise the cost of "COW is more expensive" was never measured by the model, invalidating the entire round.
- **Failure clause**: If write amplification of COW arm exceeds 2x the in-place arm **and** recovery cost shows no order-of-magnitude difference,
  directional settlement does not hold, **faithfully record and return to D22 (Synthesizing Unit Atomicity) settled item 3**.
- ⚠️ **Must carry absolute value assertions**: The block count per commit for both arms must be independently calculated from `(tree_height, fanout, stat_count)`,
  and must not be read back from tested code (`.claude/singlefs-ai-sop/rules/test-discipline.md`).

### Empirical Measurements (2026-08-29, N=5 Rounds Byte-for-Byte Identical; All 8 Mutations Caught, 0 Blind Spots)

**Both criteria satisfied, but the decisive one was not write amplification.**

#### ⚠️ Recovery Cost Column Was Not Established——Model Self-Contradiction (Pointed Out 2026-08-30 by Forward Leg of Three-Way Argument)

**The in-place arm was charged for slot alternation, yet in recovery pretended alternation did not exist.**
In code `reserved_blocks = total_nodes(tree) * 2`, comments verbatim: "To detect torn writes, allocate slot alternation (D22 (Synthesizing Unit Atomicity) settled item 21),
thus doubling reservation"; yet in the same construction `torn_loses_only_copy: 1`, `torn_rebuild_reads: pool_blocks`,
with comments "no copies ⇒ sole reconstruction path is full-disk scan".

**Slot alternation IS the second copy.** D22 (Synthesizing Unit Atomicity) itself writes the role of alternation verbatim at root slot and tail as
"do not overwrite the sole copy". ⇒ **The recovery cost table (350,000x - 89,480,000x) in section "What Does Full-Disk Scan Mean" of E38 (Accounting Structure: COW vs In-Place) does not hold and is withdrawn.**
To reconstruct it, the model must choose between: ① In-place with alternation ⇒ has second copy ⇒ recovery cost must be recalculated;
② In-place without alternation ⇒ halve reservations (not `* 2`) ⇒ mkfs column must also be recalculated.

⚠️ **Direction unaffected**: The mkfs reservation rationale stands independently (COW is constantly 0, in-place requires 32.5 MiB - 128.5 GiB).
However, **the assertion that "decisive rationale is recovery cost" has no basis**, and the settlement of D22 (Synthesizing Unit Atomicity) settled item 3 must rely on reservation to hold.

#### What Does "Full-Disk Scan" Mean: Reading Every Block in Pool at Mount Time

Accounting on in-place side is authoritative state (D21 (Boundary Between Authoritative and Derived States)) and **has no copy**
⇒ A single torn write destroys the sole copy ⇒ Sole reconstruction path is **recomputing accounts from authoritative side**,
and authoritative side is all units ⇒ **reading every block in pool**. On COW side, torn write occurs at freshly written new address,
parent pointer has not switched over yet, and old version remains untouched ⇒ only need to traverse that path along old root (tree height 2-3 blocks).

| Pool | In-Place Must Read | COW Must Read | Multiple |
|---|---|---|---|
| 4 GiB (1,048,576 blocks) | **4 GiB** | 3 blocks = 12 KiB | **350,000x** |
| 64 GiB | **64 GiB** | 12 KiB | 5,590,000x |
| 1 TiB | **1 TiB** | 12 KiB | **89,480,000x** |

⚠️ **This is block count ratio, not time ratio.** "How long it takes to read 1 TiB" was not measured in E38 (Accounting Structure: COW vs In-Place).

#### mkfs Reservation: In-Place Must Reserve Fixed Positions on Disk for **All Nodes**, Doubled for Slot Alternation

Blocks calculated at 4 KiB (**configuration, not empirical measurement**, all byte figures scale linearly):

| Accounting Tree F/H | Node Count | **Reservation** | Share of 4 GiB Pool | Share of 1 TiB Pool | Required Pool Size for Share <=2% |
|---|---|---|---|---|---|
| 64 / 2 | 4,161 | **32.5 MiB** | 0.79% | 0.00% | 1.6 GiB |
| 64 / 3 | 266,305 | **2.0 GiB** | 50.8% | 0.20% | 101.6 GiB |
| 128 / 3 | 2,113,665 | **16.1 GiB** | 403% | 1.57% | 806.3 GiB |
| 256 / 3 | 16,843,009 | **128.5 GiB** | 3213% | 12.5% | 6.3 TiB |

**Acceptable threshold set at 2% (2026-08-29, user settlement)**, not 1%——metadata overhead in other file systems naturally approaches 1%,
while the stance in `.claude/rules/fs-design.md` is "do not sacrifice self-containment to save space", **1% is insufficiently wasteful**.

⇒ **Interpretation**: Once accounting tree geometry is set large, reservations on in-place side completely consume small pools
(256/3 on 4 GiB pool requires 128.5 GiB, 32x pool capacity——**that combination cannot hold at all on small pools**).
COW side is **constantly 0**.
⇒ **"Run in-place first, switch to COW later" does not exist as a viable path**: it forks at the mkfs step.

#### Write Amplification: COW is 25–37.5% More Expensive Under D25 (Target Workload Priority) Coarse Granularity

| Leaf Spread Pattern | COW / In-Place |
|---|---|
| **Coarse-grained** (D25 (Target Workload Priority) settled: single fsync carries 8 leaves, landing on 1 shared spine) | **1.25–1.375x** |
| Dispersed (same 8 leaves, landing under 8 distinct parents) | 2.125–3.125x |

⚠️ **Both numbers recorded without presupposing which is "real"**: If D25 (Target Workload Priority) changes,
simply rerun `Spread` dimension (`cargo run --release --bin e38_accounting_copy_on_write`, seconds).

**Positive control**: Replacing COW "new address each time" with "in-place overwrite of same address" (without rewriting ancestors),
written volume drops from 110,000 to 80,000 ⇒ model is indeed measuring the COW portion of cost.

**Measurement basis**: 10,000 commits × 8 leaves each; pools 4 GiB / 64 GiB / 1 TiB across four tree geometries × two spread patterns.
Code `scripts/experiment-apparatus/e38_accounting_copy_on_write.rs`
(`cargo run --release --bin e38_accounting_copy_on_write`),
raw output `e38-accounting-cow-2026-08-29.out`,
mutation table `scripts/mutation/tables/e38_accounting_copy_on_write.tsv`.
**13 unit tests, all 8 mutations caught one by one, 0 blind spots.** Absolute value assertions:
touching 1 leaf writes exactly `1 + tree_height`; touching full F leaves under 1 parent writes exactly `F + tree_height`;
total nodes calculated directly from geometry; proportion unit is 1/10000 (`8322 / 1048576 = 79 bp`).

⚠️ **Arithmetic for proportion must be a function pinned by unit tests, not living solely in `main()`**:
When living solely in `main()`, changing 10,000 to 1,000 **will not turn a single test red**——
`M8_ratio_calculated_as_one_thousandth` in E38 (Accounting Structure: COW vs In-Place) mutation table was introduced specifically to guard this rule.

### ⚠️ What E38 (Accounting Structure: COW vs In-Place) Cannot Answer

- **It is a counting model, not a timing model.** "How long reading 1 TiB takes" was not measured.
- **It did not test torn writes directly.** "In-place torn write ⇒ sole copy destroyed" was modeled directly from D21 (Boundary Between Authoritative and Derived States)
  "accounting is authoritative state without copies", **not an observation**.
- **No one settled how large the accounting tree should be.** The "mkfs reservation" table in E38 (Accounting Structure: COW vs In-Place) swept four geometries as independent variables; **which one is correct remains unanswered**.

### Prerequisites

None. Pure counting model, runnable without waiting for other work.

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 3 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 3 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 22 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-29
- Created and completed. ⚠️ First version left proportion arithmetic in `main()`; after mutation testing revealed a blind spot, extracted into function with absolute value assertions. Source: directional settlement of D22 (Synthesizing Unit Atomicity) settled item 3 (user) explicitly requested "test it".
- Conclusion: Directional settlement stands, decisive reasons being recovery cost (350,000x on 4 GiB pool, 89,480,000x on 1 TiB pool)
  and mkfs reservation (COW constantly 0; in-place 32.5 MiB - 128.5 GiB, scaling with accounting tree geometry).
- Acceptable reservation threshold set to **2%** (user settlement), not 1%.
