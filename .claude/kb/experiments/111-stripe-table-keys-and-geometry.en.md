## E111 Stripe Table Keys and Geometry — Run (2026-09-06, 9 unit tests / 13 mutants caught): **Among the three keys, only "ordered by placement" answers the load-bearing query; tree ordered by placement has height 3 and occupies 3087 ppm, 6.98x of allocation record tree, and entry count is independent of `w`**

[中文](111-条带表的key与它的几何.md) · **English** · [日本語](111-ストライプ表のキーとその幾何.ja.md)

**This answers** D2 (RAID Stripe Policy) settled item 14 (what key to choose for the logical tree of stripe table).
D2 (RAID Stripe Policy) settled item 12 settled the **carrier** (residing in code 3 packed records, using container index, containers constantly mirrored),
**without specifying how this logical tree is sorted**; whereas its three consumers all ask the exact same question:
"**Given a placement, which stripe does it belong to?**" — allocation acceptance (C127 (pinned state has no placement in allocation records)),
scrub acceptance (C128 (sweep admission cannot decide either branch of the parity cell)),
and issuance criterion for grant area A (D9 (Encryption) settled item 9 conjunction ⑦).
⚠️ **Container index cannot answer this question**: its key is `(birth tree ID, packed record type, container number, container birth generation)`,
**with no placement across the four segments**; container numbers per C103 (container index has no placement) are allocated monotonically as "max number + 1",
having **no functional relationship** with which placements its records cover.

### Three Arms

| Arm | What Key Is Chosen | How "Find Stripe for Placement" Is Answered |
|---|---|---|
| `slot` | Placement (device identity 4 + 16 KiB slot number 6), same coordinate system as allocation record tree in D3 (Space Allocation) settled item 3 | **Single point lookup** |
| `stripe` | Stripe number (stripe birth generation 8 + stripe sequence 4) | **Cannot answer** — requires scanning entire table |
| `none` | Unsorted, indexed only by container identity (**status when run on 2026-09-06**) | **Cannot answer** — requires scanning all containers |

### Measurement Basis (Locked Before Run)

1. Geometry all points to settled items: unit 32768 (D4 (Checksum Location) settled item 1), node 16384 (D8 (Core Index Structure) settled item 2),
   container header 107 and member record 56 (D18 (Information Carried in a Block) settled item 11 / nailed in E106 (Carrier for Stripe Member Table)),
   placement key segment width 4 + 6 (D19 (Block Pointer Structure & Width Budget) settled item 4, same coordinate system as D3 (Space Allocation) settled item 7).
2. **Same ruler as E97 (Entry Encoding of Accounting and Allocation Records)**: 16 TiB / 90% fill, entry count and tree height calculated per same algorithm,
   facilitating direct comparison with allocation record tree (483 183 820 entries, tree height 4, 442 ppm).
3. **One record per cell, including parity cells** (D2 (RAID Stripe Policy) settled item 12) ⇒ entry count calculated by **cell count**, not unit count.
   ⚠️ **The 90% budget is spent jointly by data cells and parity cells**: a stripe of `w` columns occupies `w` cells, 1 of which is parity
   ⇒ total cell count constantly equals the 483 183 820 cells carved out by this budget, `w` only modifies how much within this budget is user data.
   **Never treat 483 183 820 as data cell count and add parity on top** — that would place 135% cells on a 16 TiB disk at `w = 3`
   (same error as E106 (Carrier for Stripe Member Table) round 1 126.3%, repeated once in E111 (Stripe Table Keys and Geometry) round 1,
   see "Historical Versions"; blocked by `cells_never_overfill_the_pool`).
4. `w` sweeps 2 / 3 / 4. At `w = 2` all-mirrored, zero stripe records written ⇒ entry count constantly 0, serving as negative control.

### Results (2026-09-06, artifact `e111-stripe-table-key-2026-09-06.out`, 14 lines, `emitted=14`)

| `w` | Data Cells | Parity Cells | Entry Count | Tree Height Ordered by Placement | This Tree Occupies |
|---|---|---|---|---|---|
| 2 | 0 | 0 | **0** | — | **0 ppm** |
| 3 | 322 122 547 | 161 061 273 | **483 183 820** | **3** | **3087 ppm** |
| 4 | 362 387 865 | 120 795 955 | **483 183 820** | **3** | **3087 ppm** |

⇒ **Entry count is independent of `w`**: however many cells the 90% budget carves out, that many records exist; `w` only alters ratio of data to parity among these cells.
This became visible only after changing measurement basis; round 1 read `w` as an independent variable of entry count.

**The load-bearing conclusion was not measured, but directly given by the definitions of the three keys**: Arm `slot` answers that query,
the other two cannot. ⇒ The criterion for this cell **does not lie in space** (`.claude/rules/fs-design.md` "comparing who saves more does not constitute any criterion"),
but in "whether there exists a key that allows those three consumers to ask the question."

**The space dimension still needs presentation to check affordability**: Tree ordered by placement occupies **3087 ppm ≈ 0.31% capacity** (same value at `w = 3` and `w = 4`),
which is **6.98x** of allocation record tree (442 ppm) under same ruler.
Fanouts of the two keys do differ (placement key 10 bytes ⇒ 906, stripe key 12 bytes ⇒ 815),
but at this entry count **tree heights are both 3**, so fanout does not alter this ppm; difference lies entirely in **entry count**:
allocation record has one per data unit, stripe table has one per cell, including parity cells.

⚠️ **This byte count is bit-for-bit identical to Arm A-mirrored of E106 (Carrier for Stripe Member Table)** (`54 315 515 904` bytes, 3087.5 ppm) —
two experiments using mutually distinct algorithms calculating the same quantity arriving at the exact same number. This cross-validation appeared only after changing measurement basis:
under old basis E111 (Stripe Table Keys and Geometry) reported 4631 and E106 (Carrier for Stripe Member Table) reported 3087.5, differing by 1.5x with **no one checking**.

⚠️ **When filing that day it was written "it incidentally corrected a measurement basis in E106 (Carrier for Stripe Member Table)", which was in reverse and has been retracted**:
The 483 183 820 of E106 (Carrier for Stripe Member Table) was always **total occupied placement count** (including data cells, parity cells, and containers),
not data cell count — its own round 1 made the mistake of "treating as data cell count then adding parity" and fixed it same day,
retaining `locs <= occupied_slots()` assertion that turns red.
What was wrong was E111 (Stripe Table Keys and Geometry) round 1, repeating the exact error E106 (Carrier for Stripe Member Table) had fixed.
⇒ **The corresponding line in D2 (RAID Stripe Policy) settled item 12 was revised accordingly** (4631 → 3087, matching E106 (Carrier for Stripe Member Table)).

**Controls**: Negative control (`w = 2` entry count constantly 0 across all three arms) pass;
same-ruler cross-validation (**total cell count** 483 183 820 matches recorded entry count of E97 (Entry Encoding of Accounting and Allocation Records) **bit-for-bit**) pass;
positive control (changing key width must change fanout: 906 vs 815) pass;
**boundary gate** (`cells_never_overfill_the_pool`: cells × 32768 placed across three `w` tiers must not exceed the 90% budget,
with self-proven discriminative power — must fail the 724 775 730 of round 1) pass.

**Mutations**: `scripts/mutation/tables/e111_stripe_table_key.tsv` **13 items, all 13 caught**, baseline green before and after run,
log `e111-mutate-2026-09-06.log`. Among them **M13 (parity added on top of budget) is that exact round 1 error** —
turning an encountered pitfall into a test that turns red rather than a warning comment.
⚠️ **This is a deterministic model**: evidence strength comes from mutations and assertions nailing absolute values, not round count.
**Manual calculation (expanded step-by-step)**: entries 483 183 820 ÷ 583 entries per container ⇒ `583 × 828 788 = 483 183 404 < 483 183 820`
⇒ ceil **828 789** containers; containers constantly mirrored ⇒ `828 789 × 32768 × 2 = 54 315 515 904` bytes
⇒ `÷ 16 TiB × 10⁶` = **3087 ppm**.

**Rerun** (criterion `exact`, should be byte-for-byte identical):

```bash
bash scripts/experiment-apparatus/replay.sh E111
```

### What It Cannot Answer

- **Does not answer update cost of this tree itself**: how many entries rewritten per publish, how many nodes COWed, which belongs to C103 (container index has no placement) and
  E107 (Stripe Table Write Amplification on Primary Workload); E111 (Stripe Table Keys and Geometry) only computes steady-state geometry.
- **Does not answer wall clock**, nor real latency of that point lookup on allocation path.
- **Does not answer how container index is constructed** — Arm `slot` assumes this logical tree sorts by placement while container index still performs "container identity → physical pointer",
  whereas **where container index itself lives lacks placement repo-wide** (C103 (container index has no placement)).
- **Calculated only at 16 TiB / 90% point**, not a swept curve; `w` sweeps only 2 / 3 / 4.
- Segment width of Arm `stripe` key (birth generation 8 + sequence 4) is **modeling choice**, unprescribed by any clause in repo.

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D2 (RAID Stripe Policy) settled item 14 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 12 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-06 (Part 2): Measurement Basis Reverted to "90% is a Shared Budget", Entry Count 724 775 730 → 483 183 820

- **Before revision**: `cells()` treated `16 TiB × 90% ÷ 32768 = 483 183 820` as **data cell count**, adding
  `parity = data cells / (w−1)` on top, yielding 724 775 730 cells of 32 KiB at `w = 3`.
  **That was 135% of a 16 TiB disk** — at filing no assertion watched this dimension.
- **After revision**: That 90% budget is spent **jointly** by data cells and parity cells; a stripe of `w` columns occupies `w` cells, 1 of which is parity
  ⇒ total cell count (= entry count) constantly 483 183 820, independent of `w`; `w` only modifies ratio of data to parity within it.
- **Collateral conclusion changes**: Entry count 724 775 730 / 644 245 094 → **483 183 820 (same across both tiers)**;
  ppm 4631 / 4116 → **3087 (same across both tiers)**; multiple over allocation record tree 10.5 → **6.98**;
  Arm `stripe` tree height 4 → **3** (both keys share same height at this entry count, fanout difference no longer changes height).
- **Load-bearing conclusion unaltered by a single word**: Among three keys only `slot` answers "given a placement which stripe does it belong to",
  which was never measured, but directly given by definitions of the three keys.
- **Discovery process**: Checking why E106 (Carrier for Stripe Member Table) and E111 (Stripe Table Keys and Geometry) reported two numbers differing by 1.5x for same quantity.
  Both text and source code of E106 (Carrier for Stripe Member Table) clearly stated 483 183 820 was **total occupied placement count**,
  and its own round 1 had made same mistake (126.3%), fixing it same day and retaining `locs <= occupied_slots()` assertion that turns red.
  ⇒ Statement written at filing "E111 (Stripe Table Keys and Geometry) incidentally corrected a measurement basis in E106 (Carrier for Stripe Member Table)" was in reverse and retracted.
- **This pitfall now has a test that turns red**: Newly added unit test `cells_never_overfill_the_pool` (all three `w` tiers must not exceed 90% budget,
  with self-proven discriminative power — must fail round 1's 724 775 730) and mutant M13 (parity added on top of budget).
  Unit tests 8 → **9**, mutations 12 → **13**, all 13 caught.
- ⚠️ **Cross-assertion across both experiments on same quantity remains owed**: Discovered through human cross-checking,
  no automated check compared that byte count between E106 (Carrier for Stripe Member Table) and E111 (Stripe Table Keys and Geometry).
  Recorded in [checks-owed.md](../checks-owed.md) C155 (same quantity across experiments has no cross-assertion).

