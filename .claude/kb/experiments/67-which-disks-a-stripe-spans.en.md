## E67 Which Disks a Stripe Spans — Run (2026-08-31, N=5 Seeds + Skew Round, 9 Unit Tests / 3 of 4 Mutations Caught, 1 Invalid): **Select One Group by Least Full at Object Creation, Fixed Thereafter** Best Across Both Rounds

[中文](67-一条条带参与哪几块盘.md) · **English** · [日本語](67-1つのストライプはどのディスク群にまたがるか.ja.md)

D2 (RAID Stripe Policy) settled item 6 settled how many columns to use (upper bound 4), but did not settle which disks to choose. What E67 (Which Disks a Stripe Spans) measured was not "how much is lost" —
**the expected proportion of lost data is fixed by width** (`(w−1)/C(D,2)`, independent of selection algorithm; E67 (Which Disks a Stripe Spans) empirically showed all four arms cell-by-cell equal) —
but **how the loss is distributed**: concentrated on a few objects, or dispersed across every object.

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. Difference in "intact object proportion after two simultaneous disk failures" between arms ≥ 10 percentage points ⇒ this dimension decides the selection; < 10pp ⇒ decided instead by balance degree.
2. **Conservation**: The expected proportion of lost data across arms must be mutually equal (difference ≤ 1%). Not equal ⇒ entire round invalidated.
3. Arms where the inter-disk usage range exceeds 5% of single-disk usage are **eliminated** (causes premature ENOSPC; E60 (Cost of Adding Disks Without Data Migration) already measured stranding cost).

### What Constitutes Failure (Hardcoded Before Run)

| Clause | Verdict |
|---|---|
| **Positive control, run for every arm**: `w = device count` (full width) ⇒ intact proportion must **all be 0** | **Passed**, all five arms 0 |
| **Negative control**: width 2, single-stripe object ⇒ intact proportion = `1 − C(2,2)/C(8,2)` = 27/28 | **Passed** |
| Failure pairs **exhaustively** evaluate all `C(8,2)=28` pairs, no sampling | No sampling error |

### Measurement (8 Disks, Width 4, 512 Objects; Four Rounds of Workload)

| Arm | Uniform (8 stripes per object) | Skewed (1 in 16 objects occupies 64 stripes) |
|---|---|---|
| | Intact ratio / Inter-disk range | Intact ratio / Inter-disk range ratio |
| A: Bind group by object ID | **78.6%** / 0 | 78.6% / **88.7%** ← Eliminated by criterion 3 |
| B: Round-robin per stripe | 14.3% / 0 | 74.6% / **31.9%** ← Eliminated |
| C: Pick least full 4 disks per stripe | 57.1% / 0 | 77.2% / 0 |
| D: Random per stripe | 14.1% / 4.5% | 73.7% / **6.4%** ← Eliminated |
| **E: Select group by least full at object creation, fixed thereafter** | **78.6%** / **0** | **78.6%** / **0** |

| F: Select group at object creation, **reselect when a disk in group fills** | 78.6% / 0 | **78.4%** / **0** |

Expected proportion of lost data: **all six arms 107142 ppm** (= 3/28 = `(w−1)/C(D,2)`) ⇒ Criterion 2 conservation holds.

⚠️ **Cannot compare intact proportion across rounds**: in skewed rounds most objects have only 1 stripe, inherently easier to remain intact. **Compare only within the same round.**

#### Round 3 (Added 2026-08-31): Under Heterogeneous Capacity, Does "Group Full While Pool Has Space" Occur?

The verbatim counterexample from local leg: disks need not be equal size (D2 (RAID Stripe Policy) settled item 2) ⇒ group binding might pin objects to 4 small disks,
unable to write while space remains elsewhere. Added one round with heterogeneous capacities (4 disks of 3000 cells + 4 disks of 900 cells, total capacity 15600 < demand 16384):

| Arm | Number of stripes falling on full disks (lower is better) |
|---|---|
| A: Bind group by object ID | 2012 |
| B: Round-robin | 2009 |
| D: Random | **2277** |
| C: Pick least full per stripe | **1148** |
| E: Select group at object creation | **1148** |
| F: Select group + reselect when disk in group fills | **1148** |

⇒ **Counterexample did not materialize**: Arm E selects groups based on remaining space; once small disks skew towards full they are no longer selected ⇒ **cell-by-cell identical** to picking least full per stripe.
What stumbled was the binding approach like "bind group by object ID" (Arm A) that ignores remaining space.
⚠️ All six arms had 4 disks exceed capacity — total demand already exceeded total capacity; **this round can only be compared horizontally, cannot be read as absolute values**.

#### Round 4 (Added 2026-08-31): After Adding Disks + Rebalancing of Settled Item 4 c, How Much Intact Proportion Remains?

Two cloud legs pointed to the same spot: disks added after groups established, rebalancing requires migrating data ⇒ whether groups still hold.
**Migration granularity is the sole independent variable.** After 8 disks placed, add 9th disk, migrate until new disk gets 1/9,
then exhaustively evaluate all `C(9,2)=36` pairs of failed disks:

| Migration Granularity | Moved Cells | Intact Ratio |
|---|---|---|
| **Whole object re-landed into a group** | 1824 | **83.3%** = theoretical value of `1 − C(4,2)/C(9,2)`, **not a single cell lost** |
| Stripe-by-stripe migration | 1820 | **75.0%** |

⇒ **Whole-object migration preserves the benefits of group binding intact**, and because disk count increased, intact proportion even rose from 78.6% to 83.3%.
Stripe-by-stripe migration drops 8.3 percentage points — **below the 10pp threshold of criterion 1**, faithfully recorded;
it did not drop further only because this round migrated just 11% of cells; migrating more drops more severely.

⚠️ **First version of stripe-by-stripe arm was modeled incorrectly**: scanned first N stripes sequentially, while stripes clustered by object in array
⇒ equivalent to whole-object migration, both arms gave identical numbers (83.3% vs 83.3%). Separated only after changing to stride dispersion across all objects.

### Criterion-by-Criterion Verdict

| # | Verdict |
|---|---|
| 1 | **Holds in uniform round** (A 78.6 vs C 57.1, diff 21.4pp ⇒ this dimension decides); **fails in skewed round** (E 78.6 vs C 77.2, diff 1.4pp ⇒ decided by balance degree instead). Two rounds differ in conclusion, **faithfully juxtaposed** |
| 2 | **Holds**, five arms cell-by-cell equal |
| 3 | **Eliminated A, B, D**; only C and E achieved range 0 in both rounds |

⇒ **Arm E is the only arm that achieved highest intact ratio in both rounds and range 0 in both rounds.**

### Why Group Binding Wins: Mechanism

When a pair of disks fail together, group-bound objects are either struck as an entire group (multiple stripes lost together for that object), or lose not a single cell.
**Probability of being struck is `C(4,2)/C(8,2) = 6/28`** ⇒ intact proportion is exactly `22/28 = 78.6%` (pinned by unit test).
Changing groups per stripe rolls dice independently per stripe; losing even one of 8 stripes makes the object non-intact
⇒ round-robin arm retains only `4/28 = 14.3%` (the 4 disk pairs at ring distance 4 never land in any window, independent arithmetic, pinned by unit test).

⇒ **Under the same 10.7% data loss, group binding leaves 78.6% of objects completely intact, while round-robin causes 85.7% of objects to miss blocks.**

### What It Cannot Answer

1. **Counting model**: No file system, no block device, 0 file operations.
2. **Does not model single-object throughput**: Group binding pins an object across 4 disks ⇒ upper bound of parallelism for that object is 4, not 8.
   E67 (Which Disks a Stripe Spans) did not measure this dimension, which is the known cost of group binding.
3. Does not model triple disk failure, does not model rebuild window, does not model group reclamation after object deletion.
4. Skew pattern (1/16th of objects occupy 64 stripes) is postulated, not empirically measured distribution of this project.

### Prerequisites

None. Pure counting model.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e67_device_subset.rs`
  (`cargo run --release --bin e67-device-subset`),
  raw output `e67-device-subset-2026-08-31.out` (52 lines, ending with line `emitted=52`).
- Mutation table `scripts/mutation/tables/e67_device_subset.tsv` (4 entries: M2 / M3 / M4 caught, M1 invalid).
  ⚠️ **M1 (Arm E does not fix and reselects every time) replacement failed compilation**, re-run on 2026-09-16 recorded it as compilation failure invalid mutation —
  per `.claude/singlefs-ai-sop/rules/test-discipline.md`, compilation failure counts as neither caught nor blind spot;
  **it owes a replacement mutation that genuinely alters behavior**: no mutation proves that "Arm E reselecting every time" turns red.
  ⚠️ M2 (hit only considers one disk) original text hit two locations before 2026-09-16 (`mutate.sh` exited with code 3, subsequent entries not run),
  re-run after adding one line of context on 2026-09-16 to make unique hit, M2 / M3 / M4 all turned red (C327 (Mutation Table Anchor Rot Lacks Failing Check) repaid that day).
  ⚠️ M4 (counts data loss by `w`, without deducting parity) **was not caught in first round** — conservation test only performs inter-arm comparison;
  four arms wrong together remain equal. Turned red only after adding an absolute value assertion pinning `3/28`.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 8 | Support | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-referential terms), conclusions, figures, and artifact unchanged |

## Historical Versions

Historical entries for E67 (Which Disks a Stripe Spans) are concentrated in [experiments-history.md](../experiments-history.md).
