## E47 Root Ring Failure Domain Losses — Run (2026-08-30, N=5 rounds byte-identical, 11 unit tests / 8 mutations all caught): Candidate geometry fails to mount in 4 of 6 device count tiers; slot ordering determines order of magnitude of loss

[中文](47-根环失败域的损失.md) · **English** · [日本語](47-ルートリング障害ドメインの損失.ja.md)

**What it supplements is the final dimension of D22 (Synthesizing Unit Atomicity) settled item 2.** E46 (Root Ring Region Spacing and Failure Domains) falsified
the assumption that "disk head and disk tail failure domains are independent" (7 of 18 geometric cells on the same disk), but it **only answered "whether regions die together",
without answering "what is lost if they die together"**—and the number stalling D22 (Synthesizing Unit Atomicity) settled item 2
("how many failure domains must be tolerated") precisely depends on the magnitude of loss.

### What to Test

**What remains of the root ring after one failure domain fails, and how many generations the latest usable root lags behind.**

The test object is a purely geometric + purely arithmetic model with three inputs:

| Input | Values | Provenance |
|---|---|---|
| Root ring geometry | Region count R ∈ {1,2,3,4} × slots per region S ∈ {2,4,8} | D22 (Synthesizing Unit Atomicity) candidate form is 2 × 4 |
| Array geometry | chunk ∈ {64 KiB, 512 KiB, 4 MiB} × device count ∈ {1,2,3,4,6,8} | Same measurement basis as E46 (Root Ring Region Spacing and Failure Domains), reusing the same disk placement arithmetic |
| **Slot ordering** (core independent variable of E47 (Root Ring Failure Domain Losses)) | (1) **Intra-region contiguous**: `slot = txg mod (R×S)`, region = `slot / S`; (2) **Cross-region rotation**: region = `txg mod R`, intra-region slot = `(txg / R) mod S` | **D22 (Synthesizing Unit Atomicity) never settled this dimension**, both are valid |

**⚠️ The slot ordering dimension is the direct justification for setting up E47 (Root Ring Failure Domain Losses)**: The loss of the two slot orderings under "losing one region" **is not a minor difference, but a difference in order of magnitude**—
intra-region contiguous loses **S consecutive generations**, whereas cross-region rotation loses **one generation per R generations**.
This dimension was never touched by any decision or experiment prior to E47 (Root Ring Failure Domain Losses).

**Three measured quantities** (one row per geometric cell × each slot ordering × each device failure):

| Quantity | Definition |
|---|---|
| `survivors` | Number of slots still readable after failure |
| `rollback` | Latest generation − newest generation among surviving slots. Recorded as `NA` if none survive, **not 0** |
| `usable` | Whether `rollback ≤ K − 1` holds. K is root ring depth (during scanning, taken per D22 (Synthesizing Unit Atomicity) as "K is runtime policy, lower bound ≥ 2"; this claim was retired on 2026-09-13 along with I-7.4 (Blocks in Recent K Generations Not Reused), with block reuse bound changed to `max(F_effective, oldest valid root in ring)`), scanning K ∈ {2,4,8} |

Basis for `usable`: I-7.4 (Blocks in Recent K Generations Not Reused) only guarantees blocks referenced by roots of the **most recent K generations** are not reused
⇒ When rollback exceeds K−1 generations, that root **still exists on disk, but the blocks it points to may have been reallocated and overwritten**—
**it is a rollback candidate that appears valid but is actually spurious**. ⇒ **Mountable ≠ usable; both must be measured separately.**

### Acceptance Criteria

| # | Criterion | Pre-hardcoded Expectation |
|---|---|---|
| 1 | **Surviving slot count**: `survivors = R×S − S × (regions on failed device)` | Pinned by absolute value assertion, e.g. R=2 / S=4 / chunk=64 KiB / 4 disks: region 0 on disk 0, region 1 (disk tail) on disk 2 ⇒ losing disk 0 leaves **4** |
| 2 | **Worst-case rollback differs between the two slot orderings** | Intra-region contiguous worst-case `rollback = S`; cross-region rotation worst-case `rollback = 1` (when R ≥ 2). **Equality judges that the slot ordering dimension is unimplemented** |
| 3 | **Tolerating F failure domains requires R ≥ F+1, with R regions landing on R mutually distinct devices** | Measure how many cells in the geometry table of E46 (Root Ring Region Spacing and Failure Domains) satisfy "R regions on mutually distinct disks" |
| 4 | **Relationship between `usable` and K** | Intra-region contiguous + S > K inevitably produces cells with `mountable=1 and usable=0`—**that is the pitfall E47 (Root Ring Failure Domain Losses) seeks to find** |

### Failure Conditions (Written Before Run, Immutable Afterward)

| Clause | Verdict when Triggered |
|---|---|
| **Positive control**: When R=1, failure of device hosting this sole region ⇒ `survivors=0`, `mountable=0` | Model yields non-zero ⇒ **not computing by geometry, entire run voided** |
| **Negative control**: Device count = 1 loses everything for any geometry | Yields survivors ⇒ entire run voided |
| **Cross-arm comparison alone does not count**: Both slot orderings yield identical `rollback` across all cells | ⇒ Slot ordering dimension is dead, **entire run voided** |
| **Missing absolute value assertion** | Alongside each relative comparison assertion must stand an absolute value assertion from **independent arithmetic**; missing one prohibits entering conclusion into kb |
| Result does not support "raising region count" | **Record faithfully**, no modifying criteria. E47 (Root Ring Failure Domain Losses) does not presuppose 2 regions are insufficient |

### What It Cannot Answer

1. **Real shape of physical failure domains** (controllers, backplanes, power supplies, firmware batches)—requires real hardware.
   Same limitation shared by E47 (Root Ring Failure Domain Losses) and E46 (Root Ring Region Spacing and Failure Domains): **Can only falsify "geometrically sharing disk", cannot prove "geometrically distinct disks ⇒ physical independence".**
2. **Translating loss into bytes or time**: `rollback` is in **generations**, not data volume. Translation requires the workload model of D25 (Target Workload Priority)
   (how much data a single fsync carries); E47 (Root Ring Failure Domain Losses) does not touch this.
3. **The number "how many failure domains to tolerate" itself**: This is a product judgment; experiments can only present the costs of each value.

### Prerequisites

None. Pure arithmetic model, runnable locally, requiring neither virtual machines nor real devices.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e47_ring_loss.rs` (`cargo run --release --bin e47_ring_loss`),
  raw output `e47-ring-loss-2026-08-30.out`.
- Mutation table `scripts/mutation/tables/e47_ring_loss.tsv`.
- Placement arithmetic (`device_of`) shares the **identical form** with E46 (Root Ring Region Spacing and Failure Domains) (`(logical_block_address / chunk) % device_count`, taken from Linux md raid0/raid5 data distribution),
  but **E47 (Root Ring Failure Domain Losses) rewrites its own version with separate unit tests**—sharing code between the two would mean E46 (Root Ring Region Spacing and Failure Domains) and E47 (Root Ring Failure Domain Losses) are no longer two independent paths.

### Empirical Measurements (2026-08-30, N=5 rounds byte-identical, 1741 lines, trailing line `emitted=1741`)

#### Result 1: Loss in Candidate Geometry (2 Regions × 4 Slots) Is Not "Rollback of a Few Generations", but "Failure to Mount"

Losing one disk, chunk 64 KiB, per device count (`failed_dev=0`):

| Device Count | Disks Spanned by Two Regions | Surviving Slots | Worst-case Rollback | Mountable | Usable at K=2 |
|---|---|---|---|---|---|
| 1 / 2 / 3 / 6 | **1** | **0** | NA | **No** | No |
| 4 | 2 | 4 | Intra-region contiguous **4** / Cross-region rotation **1** | Yes | **No / Yes** |
| 8 | 2 | 4 | Intra-region contiguous **4** / Cross-region rotation **1** | Yes | **No / Yes** |

⇒ **Fails to mount directly in 4 of 6 device count tiers**, because the two regions land on the identical disk—
consistent cell-by-cell with the same-disk table of E46 (Root Ring Region Spacing and Failure Domains) (E46 (Root Ring Region Spacing and Failure Domains) identified 64 KiB tier on 1/2/3/6 disks as same disk).
⚠️ **E47 (Root Ring Failure Domain Losses) is therefore not about "how large the loss is", but first "whether there is a loss at all"**:
On most device counts, candidate geometry losing one disk leaves zero surviving roots.

#### Result 2: The Dimension of Slot Ordering Determines the Order of Magnitude of Loss, Which D22 (Synthesizing Unit Atomicity) Never Settled

Distribution of worst-case rollback across 1728 cells (only counting mountable cells):

| Worst-case Rollback | 0 | 1 | 2 | 3 | 4 | 6 | 8 | 12 | 16 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|
| **Cross-region rotation** | 489 | 219 | 48 | **3** | — | — | — | — | — | — |
| **Intra-region contiguous** | 489 | — | 73 | — | 89 | 1 | 89 | 1 | 16 | **1** |

**Worst case: cross-region rotation 3, intra-region contiguous 24** (cell with R=4 / S=8 and three regions on same disk).
⇒ **Under the identical geometry and identical failure, the loss between the two slot orderings differs by 8x**;
on the candidate geometry (2×4, 4 disks) it is **4 generations vs 1 generation**, and **at K=2 the former is unusable while the latter is usable**.

⚠️ **"Mountable" does not equal "usable"** (pre-registered criterion 4 of E47 (Root Ring Failure Domain Losses), confirmed empirically):
Among 1728 × 3 = 5184 "cell × K tier" combinations, **625 cells** have `mountable=1 and usable=0`—
the root remains on disk and the checksum is good, yet the blocks it points to have exceeded the protection window of I-7.4 (Blocks in Recent K Generations Not Reused)
⇒ **It is a rollback candidate that appears valid but is actually spurious**.
Broken down by R / S / slot ordering: **Intra-region contiguous is entirely unusable at K=2** (22 cells in each of three R=2 tiers, 35 cells each for R=3, 33 cells each for R=4),
whereas cross-region rotation only appears at R≥3 (7 / 10 cells each).

#### Result 3: Increasing Region Count Makes Landing on Mutually Distinct Disks Harder

Among 18 geometric cells (3 chunk tiers × 6 device count tiers), count of cells where R regions land on R **mutually distinct** disks:

| Region Count R | 1 | 2 | 3 | 4 |
|---|---|---|---|---|
| Cells on distinct disks / 18 | **18** | **11** | **7** | **3** |
| Nominally tolerated failure domains | 0 | 1 | 2 | 3 |

⇒ **"Tolerating F failure domains requires F+1 regions" holds arithmetically, but becomes increasingly difficult to deliver geometrically**:
At R=4 only 3/18 geometric cells actually land across 4 distinct disks, with multiple regions clustered on the same disk in remaining cells,
**nominally tolerating 3 failure domains, but in reality tolerating none**.
⚠️ **This directly supports the direction already documented in E46 (Root Ring Region Spacing and Failure Domains)**:
Region locations must be selected based on "landing in distinct failure domains", **rather than simply elevating region count**.
E47 (Root Ring Failure Domain Losses) provides the quantitative measure: **Simply elevating R drops delivery rate from 11/18 down to 3/18.**

#### Item-by-Item Verdicts for Controls and Failure Clauses

| Clause | Verdict |
|---|---|
| **Positive control**: R=1 and disk hosting it fails ⇒ 0 survivors, rollback recorded as NA | **Pass** (`survivors=0 rollback=NA dead_phases=4`) |
| **Negative control**: Device count 1 ⇒ all lost | **Pass** (`survivors=0`) |
| **The two slot orderings must give different rollbacks** | **Pass**: `orders_differ=1`, worst case 24 vs 3 |
| **Absolute value assertions alongside relative comparisons** | **Pass**: 7 of 11 unit tests pin absolute values (survivors 4 / 6, rollback S / 1 / 0, disk tail LBA 9 999 999 959 040, midpoint 4 999 999 979 520, `regions_needed` four tiers) |
| **Mutation testing** | **8 mutations all caught, zero blind spots** (table in `scripts/mutation/tables/e47_ring_loss.tsv`) |

⚠️ **Record of a pre-registered criterion mismatching first implementation (discovery process, must be retained)**:
The first implementation fixed `latest` at 1 000 000, and `1 000 000 mod 8 == 0` happened to fall at the **beginning** of the region 0 segment,
causing the intra-region contiguous arm to measure a rollback of **1** instead of 4—**it measured that single phase, not the worst phase**.
The criterion specified "worst-case rollback", so **the implementation mismatched the criterion, rather than the criterion being wrong**; the implementation was fixed (added `survey_worst` to take worst across all phases in a ring).
⚠️ **Both arms yield 0 on the "best phase"** (latest generation happens to land on a surviving region, losing zero generations)
⇒ **Sampling a single phase cannot measure the difference between the two arms**. Mutation `M4_只取一个相位` injects this bug back,
and is simultaneously judged Red by 5 tests—**this blind spot is now gated.**

### Conclusions

1. **The primary risk of candidate geometry 2 regions × 4 slots is not rolling back several generations, but failure to mount on most device counts** (survivors is 0 in 4 of 6 tiers).
2. **Slot ordering must be settled, and the direction is clear: cross-region rotation**—it compresses worst-case rollback from 24 to 3, and on candidate geometry from 4 to 1,
   **at zero cost** (same slot count, same bytes, differing only in the formula for choosing which slot to publish).
   ⚠️ But it **only compresses the loss, not resolving the "regions sharing disk" issue**: when sharing disk, both slot orderings yield 0 survivors.
3. **Region count is not the primary variable; the placement rule for region locations is.** Elevating R causes delivery rate to plunge from 11/18 to 3/18.
4. ⚠️ **E47 (Root Ring Failure Domain Losses) cannot answer the number "how many failure domains to tolerate" itself**—it presents the cost of each value; choosing which is a product judgment.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e47_ring_loss.rs` (`cargo run --release --bin e47_ring_loss`),
  raw output `e47-ring-loss-2026-08-30.out` (1741 lines, trailing line `emitted=1741`).
  Mutation table `scripts/mutation/tables/e47_ring_loss.tsv` (8 mutations, all caught).
- **Failure domain = one device**. Real failure domains may be larger (controllers, backplanes, power supplies) or smaller (tracks, erase blocks); E47 (Root Ring Failure Domain Losses) does not touch these.
- **Placement arithmetic shares form with E46 (Root Ring Region Spacing and Failure Domains) but is separately written** (`(logical_block_address / chunk) % device_count`, taken from Linux md raid0/raid5 data distribution),
  with separate unit tests; sharing code would negate independence of the two paths.
- **`rollback` is in generations, not bytes**. Translating to data volume requires the workload model of D25 (Target Workload Priority), which E47 (Root Ring Failure Domain Losses) does not do.
- Region positions taken as "region 0 at disk head, remainder distributed by `(capacity − region_bytes) × r / (R − 1)` toward disk tail aligned down to chunk"—
  **this is the placement rule chosen by E47 (Root Ring Failure Domain Losses), not settled by any decision**; changing the placement rule alters the delivery rate table in Result 3.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 2 | Supports | 2026-10-11 Unaffected: wording adjustments only (temporal terms, positional references, self-referential terms); conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | No effect | 2026-10-11 Unaffected: wording adjustments only (temporal terms, positional references, self-referential terms); conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-30

- Run completed, conclusions in text. **Criteria unchanged**; first implementation missed "taking worst over phases", judged Red on the spot by E47 (Root Ring Failure Domain Losses) unit tests, fixing the implementation.
- Established. Rationale: D22 (Synthesizing Unit Atomicity) settled item 2 stalled on "how many failure domains to tolerate",
  which depends on loss magnitude; E46 (Root Ring Region Spacing and Failure Domains) only answered "whether they die together", not "what is lost if dead".
