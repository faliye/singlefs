## E106 Carrier for Stripe Member Table — Run (2026-09-06, pure arithmetic model, 15 unit tests / 11 mutants caught): **When member table lives in code 3 packed records (Arm A), unsolvable cell count on single-disk failure in steady state is constantly 0 (8 disks, 4096 units, 5 seeds × 6 container capacity tiers, container occupying 0.17%–50% of placements); however, cell right after pool creation is constantly 1 — when stripe table has only one container, it falls inside the stripe it describes, and once the disk holding the container fails it cannot be solved under any order, all five seeds hit; on this same cell Arm A-mirrored (container mirrored) and Arm B (parity self-proving) are both 0. Cost (measurement basis changed once on that day, see body): Arm A 1544 ppm, Arm B (each unit reserves 128-byte constantly zero region) 2511 ppm, Arm A-mirrored 3088 ppm, all based on 16 TiB / 90% occupancy / unit 32768**

[中文](106-条带成员表的载体.md) · **English** · [日本語](106-ストライプメンバー表の媒体.ja.md)

**This tests one cell in D2 (RAID Stripe Policy) settled item 12 (parity column location and discovery mechanism)**: where the member table resides
determines "whether cells on a failed disk can still be reconstructed after a single-disk failure." The self-admitted vulnerability in Proposal P4 was bootstrapping —
the stripe record container itself resides in a stripe; if lost, the container must be reconstructed first, yet reconstructing the container requires the member table.
**That statement was an estimate; E106 (Carrier for Stripe Member Table) turned it into an actual measurement**, and the measured form was **different** from what that sentence claimed:
not "pairwise cross-referencing", but **self-referencing**, falling on the first stripe of `w ≥ 3` (the history provided in Section 5 of the backward leg;
first built as a world in the model that must report non-zero, then rules modified).

### Cited Items (Copied Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D2 (RAID Stripe Policy) settled items 6 / 10 | `w = clamp(batched write amount + 1, 2, 4)`; an entire unit falls on one column, cell = unit |
| D2 (RAID Stripe Policy) settled item 11 | Repair = background re-places survivors as whole batch onto new stripe; member cells not eligible for reuse before repair |
| D18 (Information Carried in a Block) settled item 11 | Code 3 packed record unit 32768, header 107; second-level registry unassigned from 5 onward |
| D3 (Space Allocation) settled item 7 | Allocation record key = (device identity 4, 16 KiB slot number 6); E97 (Entry Encoding of Accounting and Allocation Records) 16 TiB / 90% measurement basis |

### Four Arms

| Arm | Where Member Table Resides | Bootstrap Dependency |
|---|---|---|
| Arm A | Code 3 packed records (type 5), containers placed in same pool as data | Present: if container lost, must reconstruct container first |
| Arm A-mirrored | Same as Arm A, but containers constantly take `w = 2` mirroring | None: single-disk failure cannot lose entire mirror |
| Arm B | Parity cell's own header (member units reserve constantly zero region) | None: parity self-proving |
| Arm C | Resides nowhere (today's status) | Not applicable, **positive control** |

### Criteria (Locked Before Run, Unchanged After Run)

1. **Positive control**: Unsolvable cell count of Arm C must exactly equal live cell count on failed disk (counted independently). Mismatch indicates metric cannot distinguish differences, **whole round invalidated**.
2. Arm B and Arm A-mirrored constantly 0, structurally guaranteed.
3. Arm A nails absolute values by seed. **Both outcomes accepted**: all 0 ⇒ proposal P4 concern invalid; non-zero ⇒ bootstrap chain is real. Written before run.
4. **Negative control**: No failure ⇒ all four arms 0.
5. Cost calculated on 16 TiB / 90% fill / unit 32768, reporting absolute ppm; unit count must equal 483 183 820 already recorded in E97 (Entry Encoding of Accounting and Allocation Records).
6. Fixed-point iteration must prove it actually moves; otherwise that loop was effectively unexecuted.

### Measurements (2026-09-06, artifact `e106-stripe-member-table-2026-09-12.out`, 65 lines, `emitted=65`; regenerated on 2026-09-12 with code 3 header 107, differs from prior version `e106-stripe-member-table-2026-09-06.out` in results/ only in config lines; mutation rerun `e106-mutate-2026-09-12.log` caught all 11)

| Arm | Unsolvable Cells Across 5 Seeds | Cost (16 TiB / 90%) |
|---|---|---|
| Arm A | **0 / 0 / 0 / 0 / 0** (failed disk loses 682 / 695 / 726 / 680 / 712 cells respectively) | 828 789 containers ⇒ capacity reduced by 828 789 data cells = 27 157 757 952 bytes = **1544 ppm** |
| Arm A-mirrored | 0 (structural: mirror containers cannot be lost) | **3088 ppm** (exactly 2x) |
| Arm B | 0 (structural: parity self-proving, no dependency edges) | 128 × 345 131 300 = 44 176 806 400 bytes = **2511 ppm** |
| Arm C | 682 / 695 / 726 / 680 / 712, **cell-by-cell equals live cell count on failed disk** | 0 |

⚠️ **A note stating "several ppm excludes parity cells" was posted here that day, which was incorrect and has been withdrawn**:
`483 183 820` is not data cell count, but **total occupied placement count** (including data cells, parity cells, and containers);
`828 789 = ⌈483 183 820 / 583⌉` is therefore already "one record per cell, including parity cells" — consistent with the measurement basis set in D2 (RAID Stripe Policy)
settled item 12, needing no recalculation. Posting that note was based on E111 (Stripe Table Keys and Geometry) round 1,
which misread this number as data cell count and added parity on top, yielding 135% of a 16 TiB disk
(recorded in "Historical Versions" (Part 2) of `106-条带成员表的载体.md`).

**Self-referencing cell (`name=first_stripe`, 4 disks, `w = 4`, pool freshly created, stripe table has only one container)**:
Across five seeds `self_referential=true`, `lost=1`, **`unsolvable=1`** — all four records of that stripe live inside one of its own members;
once the disk holding the container fails, P4's statement "as long as one member record in the stripe survives it can be solved" is false.
Arm A-mirrored on same cell is 0 (two copies of mirror on two different disks). ⇒ **Failure of Arm A is not a rare placement, but the very first `w ≥ 3` stripe upon pool creation.**

**Measurement basis for cost changed once on that day (hit by round 2 attack leg 5.5)**: Round 1 reported "bytes occupied by metadata itself",
which took `483 183 820` as **data cell count**, then added parity cells and containers on top ⇒ into a 16 TiB disk claiming 90% fill
placed **126.3%** cells (678 085 686 placements vs 536 870 912 total slots). The assertion nailing absolute values at that time only nailed data cell count,
**leaving the actual quantity fed into the cost formula unguarded** — exactly the shape of "assertions nailing non-load-bearing quantities".
Current measurement basis: **483 183 820 is total occupied placement count (including data cells, parity cells, and containers)**;
under this fixed occupancy, how many data cells each arm can hold, cost = reduction relative to Arm C.
Changing measurement basis also changed ranking: old reading "Arm A 2166 < Arm B 3516 < Arm A-mirrored 4333", self-consistent reading "Arm A 1544 < Arm B 2511 < Arm A-mirrored 3088".

⚠️ **This self-consistent measurement basis was reversely "corrected" once that day by E111 (Stripe Table Keys and Geometry) round 1, which was wrong**:
It misread 483 183 820 back into data cell count, adding parity on top (`w = 3` yields 724 775 730 cells = 135% of 16 TiB disk),
modifying cost line in D2 (RAID Stripe Policy) settled item 12 to 4631 ppm accordingly. After E111 (Stripe Table Keys and Geometry) was fixed,
it reported **54 315 515 904 bytes / 3087 ppm**, matching `cost_bytes` of E106 (Carrier for Stripe Member Table) Arm A-mirrored **bit-for-bit** —
two experiments using mutually distinct algorithms calculating the same quantity arriving at the exact same number.
⚠️ **No automated check does this cross-validation**, discovered only through manual cross-checking: recorded in
[checks-owed.md](../checks-owed.md) C155 (same quantity across experiments has no cross-assertion).

**Adding disk window cell: predicted degradation did not reproduce (recorded truthfully)**. Round 2 attack leg 5.3 predicted Arm A in adding disk window would degrade to positive control Arm C —
rationale being D2 (RAID Stripe Policy) settled item 8 rule "take the emptiest" is deterministic and globally synchronized; new disk is emptiest ⇒ every new stripe chooses it,
containers also pile on it ⇒ containers positively correlated with cells they describe, while the pillar of "steady state constantly 0" is precisely that independence.
Modeled under that rule (`name=add_disk`: 8 disks, remaining disks prefilled with 4096 slots, last disk empty, placement changed to "take the emptiest"),
**Arm A remains 0** (1643 cells lost on new disk), Arm C on same cell loses all 1639. **Reason for non-reproduction nailed as assertion**:
w−1 units in single batch spread across w−1 **distinct columns** ⇒ a single stripe has at most one cell on the same disk;
container and cells it describes therefore still fall on different disks (measured 10 containers fall on [0,7,2,3,7,5,7,0,1,7], only 4 on new disk).
⇒ **This prediction does not hold under the write pattern where "containers and data share the same batch"**; holding requires a different pattern.

**A pure arithmetic observation (without model)**: When container is written in a **single independent publish**,
formula in D2 (RAID Stripe Policy) settled item 6 gives `w = clamp(1 + 1, 2, 4) = 2`,
and parity for w = 2 is byte-for-byte identical second replica (D9 (Encryption)) ⇒ **that write is automatically mirrored**
⇒ Arm A under this write pattern degrades to Arm A-mirrored, bootstrap edge vanishes.
⇒ Difference between Arm A and Arm A-mirrored exists only in write pattern where "containers and data share the same batch".

**Container capacity sensitivity sweep** (Arm A, capacity 583 → 64 → 16 → 8 → 4 → 2, container to placement ratio 0.17% → 1.57% → 6.25% → 12.5% → 25% → 50%):
**Unsolvable count across all 30 cells is 0**. Fixed-point iteration indeed moves — `solved_after_first` non-zero on 12 of 30 cells (max 3),
meaning cells unsolvable in round 1 but solved in subsequent rounds genuinely exist; none are permanently unsolvable.

**Why constantly 0 (mechanism, not luck)**: A lost cell has `w` independent paths to take — each member of its stripe has one record;
as long as container holding any record is readable or reconstructed, member table is obtained. To be unsolvable, `w` paths must simultaneously fall into containers that are "on failed disk and themselves unsolvable".
Sorting into containers by key = (device, slot number) ⇒ records for placements on failed disk cluster together, while **what is needed are records of sibling members on living disks**,
whose keys fall in other container intervals and whose physical placements are independently randomized.

**However "extremely low probability of being unsolvable under random placement" does not equal "unsolvability cannot happen"**:
Hand-crafted deadlock (two stripes, member records of each residing in each other's container, both containers on failed disk) reports **2**,
moving one container to living disk returns to 0; and **self-referencing cell requires no hand-crafting** — built naturally by model's own pool creation order.
⇒ **Solvability of Arm A is probabilistic and constantly false at pool creation; solvability of Arm A-mirrored and Arm B is structural.**
This distinction relies not on numbers from this run, but on shape of dependency graph: Arm A's dependency graph has edges, while the other two do not.

### Mutation Testing: 11 Mutants Caught

"Unsolvable check only looks at round 1", "positive control arm also runs solver", "container fixed point ignores itself", "record capacity calculated by full unit",
"mirror arm also issues parity cell", "member table looks only at its own entry", "cost misses parity placement",
"mirror copies treated as two containers", "self-referencing cell swaps to uncorrupted disk", "occupancy treated as data cell count", "closed form stripe count undercounts one round".
⚠️ Last two were added later: round 1 model treated two copies of Arm A-mirrored as **two** containers, failing to judge "either copy readable",
falsely reporting 2 on self-referencing cell for Arm A-mirrored. Reset to 0 after changing to logical container (one container with multiple copies).

### Assertions Nailing Absolute Values (No Cross-Arm Comparisons)

`unit_count_matches_e97` (483 183 820, 16 TiB = 17 592 186 044 416),
`records_per_container_is_pinned` (583, and 107 + 56 × 583 + 13 = 32768),
`positive_control_unsolvable_count_equals_lost_count` (Arm C equals failed disk live cells per seed and > 0),
`deadlock_is_reachable_by_construction` (handcrafted deadlock reports 2, moving one container returns to 0),
`cost_absolute` (data cells 345 131 300, Arm A capacity reduced by 27 157 757 952 bytes, Arm A-mirrored = 2 × Arm A, three ppm values each nailed),
`occupancy_is_self_consistent` (placements inserted by each arm ≤ slot total 536 870 912, filling occupancy),
`closed_form_matches_loop` (closed form stripe count matches stripe loop per cell across 0–2000 and three large values),
`lone_container_publish_is_mirrored_by_the_width_formula` (clamp(1+1,2,4) = 2),
`add_disk_window_leaves_pooled_container_arm_solvable` (add disk window yields 0, nails "single stripe has at most one cell on same disk"),
`first_stripe_self_reference_is_unsolvable` (self-reference holds across five seeds, unsolvable count == cells on failed disk, Arm A-mirrored on same cell is 0),
`container_fixpoint_accounts_for_itself` (container itself requires one record).

### What It Does Not Answer

Wall clock; placement distribution of real allocator (**uniform random is modeling choice, not measurement**); multiple disks failing together (`w = 3` tolerates only one cell);
write amplification of repair (measured in E80 (Partial Release of Stripes)); **whether Arm A records readable after encryption enabled** —
that was cell hit by forward leg, belonging to criterion issue rather than arithmetic issue, dimension absent from model.
**Placement distribution in steady state cell takes uniform random, `w` only tested at 3 / 4 tiers**;
**three cost numbers hold only under assumption that "all units participate in `w ≥ 3` stripes"**; in round 1 where `w` constantly 2 Arm A constantly 0 bytes, Arm B still pays.

### Rerun

```bash
bash scripts/experiment-apparatus/replay.sh E106
cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e106-stripe-member-table \
  scripts/experiment-apparatus/e106_stripe_member_table.rs mutations/e106_stripe_member_table.tsv
```

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 12 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-06 (Part 2): Measurement Basis Reversely "Corrected" Once by E111 (Stripe Table Keys and Geometry), Clarified; Conclusion Numbers Unaltered

- **Zero numbers altered in main text**: `occupied_slots() = 483 183 820` is **total occupied placement count** (including data cells, parity cells, and containers);
  this self-consistent measurement basis was established after second round attack leg hit that day, guarded in source code by `locs <= occupied_slots()` and
  `locs + 2 >= occupied_slots()` two assertions that turn red.
- **What changed is one note**: E111 (Stripe Table Keys and Geometry) round 1 misread this number back into data cell count, adding parity on top
  (135% disk), writing "incidentally corrected a measurement basis in E106 (Carrier for Stripe Member Table)", which was retracted in E111 (Stripe Table Keys and Geometry);
  this text adds a paragraph recording this intersection and citing C155 (same quantity across experiments has no cross-assertion).
