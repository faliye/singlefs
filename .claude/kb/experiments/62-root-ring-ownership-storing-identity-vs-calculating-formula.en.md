## E62 Root Ring Ownership: Storing Identity vs Calculating Formula — Run (2026-08-31, N=5 Seeds, 6 Unit Tests / 4 Mutations Caught): **Storing Identity Yields Constant 0 Wrong-Disk Points, Cost 4 Bytes**

[中文](62-根环归属存身份vs算公式.md) · **English** · [日本語](62-ルートリング帰属-ID保持-vs-計算式算出.ja.md)

Two exits for D2 (RAID Stripe Policy) settled item 7: user chose "explicitly store device identity per zone" and requested measuring performance and safety.

### What to Measure

| # | Metric |
|---|---|
| 1 | **Wrong-disk pointing**: Computed ownership differs from disk where data actually resides, while that disk is still present |
| 2 | **Colocated disks**: Two zones belong to the same device |
| 3 | **Extra reads**: Whether locating root ring requires an additional read |
| 4 | **Byte cost**: Zone count × device identity width |

### Criteria (Hardcoded Before Run, Unchanged After)

1. Wrong-disk pointing for storing identity arm must be constantly 0; **formula arm must be > 0**, otherwise this metric cannot be measured at all.
2. Colocated disks for storing identity arm can only originate from pigeonholing at the moment of mkfs.
3. Extra reads are 0 (identity is colocated with system configuration, and system configuration must be read anyway); > 0 recorded truthfully.
4. Byte cost pinned to absolute value.

### What Constitutes Failure (Hardcoded Before Run)

| Clause | Verdict Upon Trigger |
|---|---|
| **Positive control, run for every arm**: Device set unchanged ⇒ wrong-disk pointing is 0 for both arms | Formula arm non-zero ⇒ model conflated "changed" and "unchanged", **entire round aborted** |
| **Negative control**: At mkfs time R > devs ⇒ **both arms collide** | Storing identity arm is 0 ⇒ model granted it an unearned benefit, **entire round aborted** |
| 5 seeds have inconsistent directions | Report "unstable" |

### Actual Measurement (2026-08-31, N=5 Seeds, 8-Step Device Set Changes per Round)

| Arm | Wrong-Disk Pointing | Colocated Disks | Extra Reads | Byte Cost |
|---|---|---|---|---|
| Formula (ownership calculated by current devs) | **2 / 4 / 6 / 17 / 19** | 0 / 6 / 2 / 2 / 0 | 0 | 0 |
| **Store Identity** (written at mkfs, not recalculated thereafter) | **0 (all five seeds)** | **0** | **0** | **4 bytes** |

- **Cost is 4 bytes**: 4 zones × 1 byte device identity. Extra reads 0—identity is colocated with system configuration, and system configuration must be read anyway.
- **Exhaustively verified one property**: R∈[2,8] × devs∈[1,64] totaling 504 cells; the **28 cells where the formula arm collides are all `devs < R`**, and cells where `devs ≥ R` yet collides are **0 cells** ⇒ disk additions never collide; collisions originate solely from disk drops to `devs < R`.

### Item-by-Item Verdict for Controls

| Clause | Verdict |
|---|---|
| Positive control, run on both arms: device set unchanged | **Passed**, wrong-disk pointing is 0 for both arms |
| Negative control: at mkfs R=4 > devs=3 | **Passed**, both arms collide (1 time each) ⇒ **storing identity cannot solve pigeonhole**, it solves wrong-disk pointing |
| 5 seed directions | **Consistent**: storing identity arm all 0 across five rounds, formula arm all > 0 across five rounds |

All 4 mutations caught: storing identity arm changed to calculate by current devs / wrong-disk pointing does not count lost disks / prime number changed to composite number / byte cost zeroed out.

### The Other Half It Cannot Answer

1. **Counting model**: No system configuration, no real disk, no mount, 0 file operations. "Extra reads 0" is counted under the premise that "identity is stored in system configuration, and system configuration must be read anyway", **not measured I/O**.
2. **Does not cover losses from the dropped-disk dimension**: If the disk where a zone resides is gone, it is simply gone; E62 (Root Ring Ownership: Storing Identity vs Calculating Formula) only judges whether ownership is correct.
3. R and device identity width are inputs, not measurements.

### Prerequisites

None. Pure counting model.

### Measurement Basis and Rerun

- Code `scripts/experiment-apparatus/e62_ring_home.rs` (`cargo run --release --bin e62-ring-home`), raw output `e62-ring-home-2026-08-31.out` (16 lines, closing line `emitted=16`).
- Mutation table `scripts/mutation/tables/e62_ring_home.tsv` (4 items, all caught).

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 7 | Supports | 2026-09-20 Unaffected: storing identity arm wrong-disk pointing is all 0 across five rounds, formula arm 2 / 4 / 6 / 17 / 19 times in 8 steps, extra reads 0, byte cost 4 bytes, serving as evidence for changing ownership to storing device identity per zone; repository-wide renaming on the same date "system configuration" → "system configuration" adjusted wording only, with conclusions, numbers, and artifacts unchanged |
| D2 (RAID Stripe Policy) settled item 5 | Supports | 2026-09-20 Unaffected: the cost in disk addition direction of "calculating by current devs points to another disk" is eliminated by storing identity; without rearrangement, only the disk drop cell remains; in negative control where R > devs at mkfs both arms collide, and that half of the failure domain is still borne by settled item 5; repository-wide renaming on the same date "system configuration" → "system configuration" adjusted wording only, with conclusions, numbers, and artifacts unchanged |

## Historical Versions

Historical entries for E62 (Root Ring Ownership: Storing Identity vs Calculating Formula) are collected in [experiments-history.md](../experiments-history.md).
