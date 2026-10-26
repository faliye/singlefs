## E63 How Many Columns Allocator Writes per Invocation — Run (2026-08-31, N=5 Seeds, 7 Unit Tests / 4 Mutations Caught): **Space Demands Wide, Safety Demands Narrow; Criterion Input Is Post-Batching Write Volume**

[中文](63-分配器每次写用几列.md) · **English** · [日本語](63-アロケータは書き込み毎に何列を使用するか.ja.md)

Among the three items required by D2 (RAID Stripe Policy) settled item 6, E63 (How Many Columns Allocator Writes per Invocation) measures the first two: what the criterion input is, and whether width needs an upper bound.

### Three Axes

| Axis | Mechanism |
|---|---|
| Space | A stripe of width `w` carries `w−1` cells of data and 1 cell of parity ⇒ overhead when fully packed is `1/(w−1)`, **the larger w is, the more space is saved**; but small writes cannot fill it, when payload=1 it costs `w` cells to obtain 1 cell of data, **the larger w is, the greater the penalty** |
| Safety | Choosing any two disks failing simultaneously, probability that a stripe of width `w` is struck on two cells is `C(w,2)/C(D,2)` (parity=1 ⇒ striking two cells loses data) ⇒ **the larger w is, the more dangerous** |
| Batching | When multiple small writes are merged into a single stripe, the loss along the space axis disappears ⇒ criterion input should be the post-batching volume. E17 (Write Buffer Merge Limit) has already measured that this project can batch effectively |

### Criteria (Hardcoded Before Run, Unchanged After)

1. When the proportion of small writes is high, "sizing width to fit write volume" saves ≥ 10% physical cells compared to "always taking full width" ⇒ **criterion must include write volume**; saving < 10% ⇒ verdict "write volume does not need to enter criterion". **The reverse outcome is accepted.**
2. Double-disk strike probability must increase monotonically with `w` ⇒ **criterion must place an upper bound on width**; non-monotonic ⇒ safety axis measured wrongly, entire round aborted.
3. Batching arm compresses within 1.05× of "exact fill" ⇒ criterion input takes **post-batching** volume.

### What Constitutes Failure (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| **Positive control, run for every arm**: payload equals exactly `w−1` of that arm ⇒ physical/data must equal `w/(w−1)` | **Passed** (8-wide 1.142857, 3-wide 1.500000, cell-by-cell equal across arms) |
| **Negative control**: payload constantly 1, `w=2` ⇒ physical/data exactly 2.000 | **Passed** |
| 5 seeds have consistent directions | **Consistent**, rankings across four small-write proportions completely identical |

⚠️ **Positive control in the first version was written incorrectly**: fed `DEVICE_COUNT−1` to all arms, whereas `w−1` for `fixed3` is 2 ⇒ what it judged red was the control, not the arm. Implementation was fixed, not criterion.

### Actual Measurement (2026-08-31, N=5 Seeds, 8 Disks, 20 000 Writes, Batch Size 16)

Small write = 1 cell, large write = 8..15 cells. `hit` is the probability that a stripe is hit on two cells when any two disks fail simultaneously.

| Small-Write Proportion | Always Take Full Width | Sizing to Fit Write Volume | Fixed 3-Wide | Batch First Then Full Width |
|---|---|---|---|---|
| 0% | 1.475–1.478 / hit 1.000 | 1.475–1.478 / 1.000 | 1.564–1.567 / **0.107** | **1.161–1.162** / 1.000 |
| 25% | 1.657–1.666 / 1.000 | 1.490–1.496 / 0.867 | 1.605–1.606 / **0.107** | **1.167** / 1.000 |
| 50% | 1.991–2.007 / 1.000 | 1.518–1.522 / 0.688 | 1.678–1.682 / **0.107** | **1.177–1.179** / 1.000 |
| 90% | **4.278–4.402** / 1.000 | 1.702–1.716 / 0.215 | 2.181–2.209 / **0.107** | **1.258–1.271** / 1.000 |

Double-disk strike probability (8 disks, independent arithmetic): w=2 → 0.0357, w=4 → 0.214, w=6 → 0.536, w=8 → **1.000**.

### Item-by-Item Verdict for Criteria

| # | Verdict |
|---|---|
| 1 | **Holds**: When small-write proportion is 90%, fitting arm 1.71 vs full-width arm 4.34, **saves 61%** ⇒ **criterion must include write volume** |
| 2 | **Holds**: Strike rate `C(w,2)/C(D,2)` strictly monotonic ⇒ **width must have an upper bound, cannot "use however many disks exist"** |
| 3 | **Partially holds**: Ratio of batching arm to "exact fill" (1.142857) is 1.016 / 1.021 / 1.031 (0/25/50%), **90% tier is 1.10–1.11, exceeding 1.05** ⇒ batch size 16 insufficient under extreme small-write workload. Truthfully recorded, criterion unchanged |

### Three Measured Tensions

1. **Batching saves the most space, but pushes width to maximum ⇒ safety axis is worst** (hit=1.000: any two disks failing simultaneously is guaranteed to strike).
2. **Fixed 3-wide is safest** (hit constantly 0.107, 1/9 of full width); space penalty at 90% small-write proportion is 1.73× that of batching arm.
3. **Fitting write volume is the only one self-adaptive to workload**: automatically narrows when small writes abound (hit drops from 1.000 to 0.215), space still saves 61% over full width.

### The Other Half It Cannot Answer

1. **Counting model**: No file system, no block device, 0 file operations.
2. **Does not model "which devices participate"** (subset selection of D2 (RAID Stripe Policy) settled item 6 c), only models how many columns to use.
3. **Does not model latency cost of batching**, nor does it model conflict between batching and fsync—batch size 16 was configured, not measured.
4. `parity=1` is an assumption: this project has never settled how many cells parity takes (D16 (Publish Semantics) body acknowledges redundancy format of root is a gap).

### Prerequisites

None. Pure counting model.

### Measurement Basis and Rerun

- Code `scripts/experiment-apparatus/e63_width_rule.rs` (`cargo run --release --bin e63-width-rule`), raw output `e63-width-rule-2026-08-31.out` (93 lines, closing line `emitted=93`).
- Mutation table `scripts/mutation/tables/e63_width_rule.tsv` (4 items, all caught).
  ⚠️ Two of them (not deducting parity, batch size changed to 1) **were not caught in round 1**, turned red only after adding golden standard and "batching actually batches" tests.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 6 | Supports | 2026-09-20 Unaffected: at 90% small writes, sizing to fit write volume saves 61% physical cells, and double-disk strike rate monotonically increases with w, serving as evidence for criterion containing write volume and providing upper bound |
| D2 (RAID Stripe Policy) settled item 18 | Supports | 2026-09-20 Unaffected: upper bound 4 vs eliminating upper bound on 8 disks expects data loss 10.7% vs 25.0%, serving as conservative evidence for system configuration w_max taking 4 |
| D16 (Publish Semantics) | Does not affect | 2026-09-20 Unaffected: E63 (How Many Columns Allocator Writes per Invocation) only cites "how many cells parity takes has never been settled" as assumption, judging no clauses of D16 (Publish Semantics) |

## Historical Versions

Historical entries for E63 (How Many Columns Allocator Writes per Invocation) are collected in [experiments-history.md](../experiments-history.md).
