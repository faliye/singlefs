## E60 (Cost of Adding Disks Without Data Migration) — Run (2026-08-31, N=5 Seeds, 10 Unit Tests / 5 Mutations Caught): Cost of Not Migrating Is Not Slowness, But Inability to Purchase 60% of New Disk

[中文](60-加盘之后不搬数据的代价.md) · **English** · [日本語](60-ディスク追加後にデータ移動を行わないコスト.ja.md)

D2 (RAID Stripe Policy) settled item 4 c (user settlement) settled "do not migrate data; new data lands preferentially on new disks",
while the user simultaneously requested "measure performance; if performance is poor, migration is acceptable".

### What to Measure

| # | Quantity |
|---|---|
| 1 | **How much user data can still be written** after adding disks (evaluated across three allocation strategies) |
| 2 | **Physical cells / data cells** (parity overhead) |
| 3 | **Stranded capacity**: remaining free cells when stripes of width ≥ 2 can no longer be assembled |
| 4 | **One-time migration volume** to rebalance—what is saved by not migrating; omitting it counts only one side of the ledger |

### Criteria (Hardcoded Pre-Run, Unmodified Post-Run)

| # | Criterion |
|---|---|
| 1 | Three arms compared by "physical cells consumed to write identical volume of user data" |
| 2 | Arm A **≥ 10% higher** than Arm C ⇒ judge "cost of not migrating is significant", D2 (RAID Stripe Policy) settled item 4 c must be reopened |
| 3 | Higher by **< 10%** ⇒ judge "cost not significant, maintain no migration". **Written down pre-run: reversed outcome accepted** |
| 4 | Arm C migration volume must be reported alongside |
| 5 | Stranded capacity of each arm must be reported |

### What Constitutes Failure (Hardcoded Pre-Run)

| Clause | Verdict Upon Triggering |
|---|---|
| **Conservation**: sum of used cell deltas across devices ≠ physical cells in accounting | **Entire round scrapped** |
| **Positive control, run for every arm**: new disk capacity 0 (equivalent to not adding disk) ⇒ all four arms collapse to 4-disk pool figures, Arm C migration volume 0 | Any arm failing ⇒ **entire round scrapped** |
| **Negative control**: existing disks also empty upon adding disk (no imbalance) ⇒ Arm C migration volume 0 | Non-zero ⇒ **entire round scrapped** |
| 5 seeds yield inconsistent conclusion directions | Report "unstable", render no conclusion |

### Model

4 legacy disks of 1000 cells each, filled to 90% before disk addition; new disk empty with 1000 cells. A stripe occupies one cell on each participating disk,
with 1 cell parity and `w−1` cells data out of `w` cells (D2 (RAID Stripe Policy) primary conclusion: full stripe writes, never read-modify-write).
Write payload randomly 1..8 cells per write (5 seeds). Writes continue until stripes with `w ≥ 2` can no longer be assembled.

| Arm | Strategy |
|---|---|
| Arm A | **No migration + favor new disk**: narrow stripes (width 2 or 3) land on disks with most remaining free space |
| Arm B | **No migration + unbiased**: each stripe utilizes all disks that still have free space |
| Arm C | **Full width after rebalancing**: rebalance via water-filling, then write per Arm B |

### Measurements (2026-08-31, N=5 Seeds)

#### Result 1: An Analytical Upper Bound Independent of Allocation Strategy

**Without migrating data, the new disk can absorb at most "the sum of remaining free space on all other disks" cells.**
Mechanism: a redundant stripe requires at least two disks to each contribute one cell ⇒ every cell written to the new disk requires another disk to contribute a cell concurrently.

4 disks of 1000 cells each, filled to 90% ⇒ remaining free space is 400 cells ⇒ **600 out of 1000 cells on the new disk are permanently unusable (60%)**.
The fuller the existing disks, the worse this ratio: 80% stranded at 95% fill, 96% stranded at 99% fill.

⚠️ **Arm A is not a strawman**: narrow stripe w=2 **hits this analytical upper bound exactly** (new disk absorbs 400 = 4 × 100 cells, not one cell more or less)
⇒ it represents the **optimal strategy without migration**, rather than an arbitrarily chosen poor variant.

#### Result 2: Three Arms Cell by Cell

| Arm | User Data Still Writable After Disk Addition (Cells) | Physical / Data | Average Width | Stranded (All on New Disk) | One-time Migration |
|---|---|---|---|---|---|
| Arm A Narrow Stripe w=2 | **400** (identical across all 5 seeds) | **2.000** | 2.0 | **600** | 0 |
| Arm A Narrow Stripe w=3 | 358–362 | 1.657–1.676 | 3.0 | **800** | 0 |
| Arm B Full Width Unbiased | 294–302 | 1.656–1.701 | 5.0 | **900** | 0 |
| Arm C Full Width After Rebalancing | **819–837** | 1.673–1.709 | 5.0 | **0** | **720** |

⇒ **Arm C writes 2.05–2.09× more user data than the best no-migration strategy**, at the cost of migrating 720 cells once
(20% of the 3600 cells used on legacy disks).

#### Result 3: Judged by Hardcoded Pre-Run Criteria, Two Arm A Variants Judge Oppositely

| Arm | Physical / Data Relative to Arm C | Criterion 2 / 3 Verdict |
|---|---|---|
| Arm A w=2 | **+18.3%** | Judged "cost significant" ⇒ settled item 4 c must be reopened |
| Arm A w=3 | **−2.0%** | Judged "cost not significant" ⇒ maintain no migration |

⚠️ **Criteria 1–3 focused solely on the parity overhead dimension, whereas the decisive dimension is stranded capacity** (Criterion 5 only required reporting, setting no threshold).
Per `.claude/singlefs-ai-sop/rules/test-discipline.md` "never retroactively modify criteria—modifications constitute a new experiment",
here both are **faithfully juxtaposed**: the criteria deliver a split verdict, while the unconstrained dimension reveals a 2× disparity.
**To establish stranded capacity as a decision criterion, a separate experiment must be founded.**

#### Clause-by-Clause Verdict on Controls

| Clause | Verdict |
|---|---|
| Conservation (physical cells = sum of stripe occupancy) | **Pass**, holds individually across all three arms |
| **Positive control, run for all four arms**: new disk capacity 0 | **Pass**, all four arms collapse to 4-disk pool, Arm C migration volume 0 |
| Negative control: balanced pool ⇒ migration volume 0 | **Pass** |
| 5 seeds yield identical direction | **Consistent**: rankings for stranded capacity and data volume are identical across all 5 seeds |

#### Mutation Testing: 5 Mutations, All Eventually Caught, But One Missed on First Try

| # | Mutation | Result |
|---|---|---|
| M1 | Alter upper bound to entire new disk | Caught |
| M2 | Narrow stripe fails to clamp width | Caught |
| M3 | Stripe counts `w` cells as data (without deducting parity) | Caught |
| M4 | Rebalancing does not clamp capacity | Caught (**genuine bug, turned red on positive control first run**: migrated 200 cells when new disk capacity was 0) |
| M5 | Width lower bound in disk selection relaxed to 1 | **Missed on first try** |

⚠️ **M5 warrants specific documentation**: `no_arm_emits_width_one` verified minimum width after completion,
while `run()` contained a second identical `stripe_devices.len() < 2` gate, which **swallowed the error from disk selection**—
unobservable when both gates are correct, and unobservable when either gate is removed. Caught only after adding a test **pinned directly to the disk selection step**
(`choose_refuses_width_one`).

### The Half It Cannot Answer

1. **This is an allocation-layer count model, not throughput measurement**: zero file systems, zero block devices, zero file operations.
   It provides physical cell consumption, stripe width, and stranded capacity, **not MB/s**. Real throughput must await transaction layer + QEMU.
2. **Does not model reads**: the dimension that "new data clusters on few disks ⇒ lower read parallelism" is reflected only indirectly via average width, without direct measurement.
3. Does not model fragmentation, nor background compaction (D3 (Space Allocation) settled item 14 resident compaction handles fragmentation, not inter-disk balance).
4. **Fill rate is an input, not a measurement**: 90% was configured, not an empirically measured typical value for this project.

### Prerequisites

None. Pure allocation-layer count model.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e60_rebalance.rs`
  (`cargo run --release --bin e60-rebalance`),
  raw output `e60-rebalance-2026-08-31.out` (27 lines, terminating line `emitted=27`).
- Mutation table `scripts/mutation/tables/e60_rebalance.tsv` (5 mutations, all caught).

### Affected Decisions

| Decision Item | Relationship | Retrospective |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 4 | Supports | 2026-09-20 unaffected: without migration new disk absorbs only the sum of remaining free space on other disks; under 4 disks at 90% fill, 60% of new disk is stranded, and at 95% fill 80% is stranded, supporting the background rebalancing direction of settled item 4 c |
| D3 (Space Allocation) | Does not affect | 2026-09-20 unaffected: E60 (Cost of Adding Disks Without Data Migration) explicitly states it does not model fragmentation and resident compaction, leaving all clauses of D3 (Space Allocation) unjudged |

## Historical Versions

Historical entries for E60 (Cost of Adding Disks Without Data Migration) are consolidated in [experiments-history.md](../experiments-history.md).
