## E48 Feasible Points for Root Ring Geometry — Run (2026-08-30, N=5 rounds byte-identical, 12 unit tests / 8 mutations all caught): Found 9 feasible points, all "prime stride + cross-region rotation"

[中文](48-根环几何的可行点.md) · **English** · [日本語](48-ルートリング幾何の実行可能点.ja.md)

**It proceeds directly from E47 (Root Ring Failure Domain Losses).** E47 (Root Ring Failure Domain Losses) measured that candidate geometry (2 regions × 4 slots, disk head and disk tail)
**fails to mount in 4 of 6 device count tiers**, pointing out that "what needs to be settled is not the region count, but the **selection rule for region locations** plus **slot ordering**".
E48 (Feasible Points for Root Ring Geometry) treats those two dimensions as independent variables to **search for a point satisfying all acceptance criteria**.

⚠️ **This is a new experiment, not a rerun of E47 (Root Ring Failure Domain Losses)**: The question asked has changed
(E47 (Root Ring Failure Domain Losses) asked "how large is the loss", whereas E48 (Feasible Points for Root Ring Geometry) asks "whether there exists a point without this loss"),
and per `.claude/singlefs-ai-sop/rules/test-discipline.md` "changing criteria makes a new experiment", a new ID is assigned.
The artifacts and conclusions of E47 (Root Ring Failure Domain Losses) remain untouched.

### Independent Variables

| Dimension | Values |
|---|---|
| **Placement rule** (new) | (1) `even_span`: region r placed at `r × (capacity − region_bytes)/(R−1)`, aligned down to chunk—**the one used by E47 (Root Ring Failure Domain Losses)**; (2) `prime_stride`: region r placed at `r × P × chunk`, **P = 8191 (prime)**; (3) `adjacent`: region r placed at `r × region_bytes` (contiguous, serving as negative control) |
| Slot ordering | Intra-region contiguous / Cross-region rotation (reusing the two arms from E47 (Root Ring Failure Domain Losses)) |
| Root ring geometry | R ∈ {1,2,3,4} × S ∈ {2,4,8} |
| Array geometry | chunk ∈ {64 KiB, 512 KiB, 4 MiB} × device count ∈ {1,2,3,4,6,8} |

**Mechanism of `prime_stride`**: Placement assignment is `(lba / chunk) % devs`, taking `lba_r = r × P × chunk`
⇒ assignment = `(r × P) mod devs`. **P is prime and larger than any reasonable device count ⇒ `gcd(P, devs) = 1`
⇒ multiplication by P is a bijection modulo devs ⇒ distinct r map to pairwise distinct device assignments**, as long as `R ≤ devs`.
Spacing = `P × chunk − region_bytes`, which on 64 KiB chunk is **536 804 352 bytes** (`8191 × 65536 − 1024`), far exceeding the maximum `io_min` tier (65536).
⚠️ **This number was corrected on 2026-09-09 from 536 870 912**: that was `8192 × 65536 = 2²⁹`, whereas `P = 8191`; E48 (Feasible Points for Root Ring Geometry) unit tests verbatim `assert_eq!(g, 536_804_352)`, and artifacts verbatim state `gap=536804352`, **with the prose differing from its own artifact by 66 560 bytes**. ⚠️ **The erroneous number was not random**: `2²⁹` is a power of 2, divisible by any power-of-two `zone_size`, whereas the true value `8191 × chunk` (because 8191 is an **odd** prime) has v₂ strictly equal to chunk's v₂ ⇒ **cannot divide any power-of-two `zone_size` larger than chunk**—that clerical error happened to conceal this incompatibility; see D22 (Synthesizing Unit Atomicity) unsettled item 6.

### Acceptance Criteria: Definition of "Suitable Point" (Written Before Run)

A `(placement rule, R, S, slot ordering)` combination is called **feasible** if and only if across all scanned `(chunk, devs)` cells:

| # | Condition |
|---|---|
| a | On cells with `devs ≥ R`, `mountable` remains constantly 1 after losing **any** single disk |
| b | On those same cells, `usable` remains constantly 1 at `K = 2` (worst-case rollback ≤ 1) |
| c | Spacing between any two regions ≥ **65536** (maximum `io_min` tier, isolation criterion from E46 (Root Ring Region Spacing and Failure Domains)) |
| d | **No requirement placed on cells with `devs < R`** |

⚠️ **Condition d must be stated explicitly, otherwise the criterion is unsatisfiable**: When region count exceeds device count, the pigeonhole principle guarantees at least two regions share a disk;
**this is not a failure of the placement rule, but a geometric impossibility**. ⇒ E48 (Feasible Points for Root Ring Geometry) concurrently reports "on how many device counts a point is feasible".

### Failure Conditions (Written Before Run, Immutable Afterward)

| Clause | Verdict when Triggered |
|---|---|
| **Positive control**: When `devs = 1`, `mountable` must be 0 for any rule | Non-zero ⇒ model not computing by geometry, **entire run voided** |
| **Negative control**: `adjacent` rule spacing must be < `io_min` ⇒ condition c must judge No | Judged Yes ⇒ spacing not computed, **entire run voided** |
| **`prime_stride` shares disk on any cell with `devs ≥ R`** | **Record faithfully, no switching P to rerun**—switching is a new experiment |
| **Three placement rules yield identical disk assignments across all cells** | ⇒ Placement rule dimension is dead, **entire run voided** |
| Zero feasible points found | **Record faithfully**. E48 (Feasible Points for Root Ring Geometry) does not presuppose finding one |

### What It Cannot Answer

1. **Physical failure domains**: Same limitation as E46 (Root Ring Region Spacing and Failure Domains) / E47 (Root Ring Failure Domain Losses)—only judges whether regions geometrically land on the same disk,
   without judging common causes such as controllers / backplanes / power supplies.
2. **Behavior after device set changes**: The assignment for `prime_stride` is `(r × P) mod devs`, and **when devs changes the assignment changes**.
   ⚠️ **Adding disks does not collide**: `gcd(P, devs) = 1` ⇒ multiplication by P is bijective ⇒ assignments are pairwise distinct when `R ≤ devs`, and adding disks only increases devs;
   collisions occur when `devs < R` (dropped disk / replaced with fewer disks), which is pigeonhole. The issue when adding disks is that **the assignment points to another disk where no data exists**,
   because **region locations are an on-disk fait accompli laid down at mkfs**.
   ⇒ This cell is an instance of D2 (RAID Stripe Policy) settled item 3 (explicit geometry re-evaluation transactions); D2 (RAID Stripe Policy) settled item 5 settled **no re-placement** ⇒ the guarantee of E48 (Feasible Points for Root Ring Geometry) holds strictly on the mkfs device set.
3. **The reason for P = 8191 is solely "prime and larger than any reasonable device count"**, not an optimization result; E48 (Feasible Points for Root Ring Geometry) does not search P.

### Prerequisites

None. Pure arithmetic model.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e48_ring_placement.rs`
  (`cargo run --release --bin e48_ring_placement`),
  raw output `e48-ring-placement-2026-08-30.out`.
- Mutation table `scripts/mutation/tables/e48_ring_placement.tsv`.
- Placement arithmetic shares form with E46 (Root Ring Region Spacing and Failure Domains) / E47 (Root Ring Failure Domain Losses), **rewritten and unit-tested separately in E48 (Feasible Points for Root Ring Geometry)**.

### Empirical Measurements (2026-08-30, N=5 rounds byte-identical, 131 lines, trailing line `emitted=131`)

#### Feasible Points: 9, All Landing on the Identical Combination

| Placement Rule | Slot Ordering | Feasible (R, S) | Worst-case Rollback | Cells with Pairwise Distinct Assignments |
|---|---|---|---|---|
| **`prime_stride`** | **Cross-region rotation** | **R ∈ {2,3,4} × S ∈ {2,4,8}, all 9 combinations** | **1** | **100%** (15/15 for R=2, 12/12 for R=3, 9/9 for R=4) |
| `prime_stride` | Intra-region contiguous | None | S | 100% (mountable, but **not a single cell usable** at K=2) |
| `even_span` (used by E47 (Root Ring Failure Domain Losses)) | Both tested | **None** | —— | Cannot reach 100% |
| `adjacent` (negative control) | Both tested | None | —— | Gap 0, condition c judged No |

**Disk assignment of `prime_stride`** (chunk 64 KiB, four regions r=0..3):

| Device Count | 1 | 2 | 3 | 4 | 6 | 8 |
|---|---|---|---|---|---|---|
| Assignment | 0,0,0,0 | 0,1,0,1 | 0,1,2,0 | **0,3,2,1** | **0,1,2,3** | **0,7,6,5** |

⇒ **Pairwise distinct when `R ≤ devs`; this is construction, not coincidence**: assignment = `(r × P) mod devs`,
P is prime and larger than any reasonable device count ⇒ `gcd(P, devs) = 1` ⇒ multiplication by P is a bijection modulo `devs`.
Spacing is constantly `P × chunk − region_bytes`, which on 64 KiB chunk is **536 804 352 bytes**,
far exceeding all three `io_min` tiers (512 / 4096 / 65536).

#### Item-by-Item Verdicts for Three Controls

| Clause | Verdict |
|---|---|
| **Positive control**: When `devs = 1`, survivors must be 0 for any rule, any geometry | **Pass** (`all_zero=1`) |
| **Negative control**: Spacing for `adjacent` must be < `io_min` | **Pass** (`gap=0`) |
| **Three placement rules must not yield identical assignments** | **Pass**: On 4 regions / 4 disks, `prime_stride` is 0,3,2,1, `adjacent` is 0,0,0,0, and `even_span` differs from the former |
| **`prime_stride` still on same disk on any `devs ≥ R`** | **Did not occur**. ⚠️ Had it occurred, the failure clause of E48 (Feasible Points for Root Ring Geometry) would be "record faithfully, no switching P to rerun" |

#### ⚠️ Mutation Testing Round 1 Exposed Two Blind Spots, Both Remediated

In Round 1, `M5_feasible_criterion_ignores_usability` and `M7_pigeonhole_criterion_inverted` **failed to turn any test red**—
both criteria resided inside `main()` at the time, **unreachable by any unit test**.
Remediation: Extracted feasibility judgment into pure function `judge()`, and added two absolute value assertions—
(1) Prime stride + intra-region contiguous produces `mount_ok=15 but usable_ok=0` (**judging mountable alone would falsely deem it feasible**);
(2) Cell count with `devs ≥ R` per R is 18 / 15 / 12 / 9 (independent arithmetic: 3 chunk tiers × count of DEVS ≥ R).
After remediation, **all 8 mutations were caught**.
⚠️ **This is a close relative of what C49 (Universal Assertions Only Verified at Sample Points) aims to intercept**: criteria written outside test reach, like universal assertions verified only at samples,
both represent "conclusions without gates".

### Conclusions

1. **Feasible points exist, and only under one combination**: **Regions placed by prime stride + cross-region rotation slot ordering**.
   Across all cells with `R ≤ devs`: losing any single disk **constantly mounts**, **worst-case rollback is only 1 generation**, and K=2 is sufficient.
2. **Two degrees of freedom are decoupled**:
   - **R is determined solely by "how many failure domains to tolerate"** (R = F + 1), no longer constrained by geometric delivery rate—
     in E47 (Root Ring Failure Domain Losses), delivery rate for `even_span` dropped from 11/18 to 3/18 as R increased, whereas `prime_stride` is constantly 100%;
   - **S does not affect loss** (worst-case rollback under cross-region rotation is constantly 1, independent of S) ⇒ **S is determined by "how many generations to retain in one turn"**,
     namely root ring depth K and block reuse delay settled in D22 (Synthesizing Unit Atomicity), independent of failure domains.
3. **The intra-region contiguous slot ordering can be directly eliminated**: It fails to achieve K=2 usability under **all** placement rules.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e48_ring_placement.rs`
  (`cargo run --release --bin e48_ring_placement`),
  raw output `e48-ring-placement-2026-08-30.out` (131 lines, trailing line `emitted=131`).
  Mutation table `scripts/mutation/tables/e48_ring_placement.tsv` (8 mutations, all caught).
- ⚠️ **Assignment of `prime_stride` depends on `devs`, while region locations are an on-disk fait accompli established at mkfs**:
  After adding / replacing disks, `(r × P) mod devs` shifts ⇒ assignment points to another disk; when `devs < R`, pigeonhole causes same-disk collisions.
  **This cell is an instance of D2 (RAID Stripe Policy) settled item 3 (explicit geometry re-evaluation transactions); D2 (RAID Stripe Policy) settled item 5 settled **no re-placement** ⇒ the guarantee of E48 (Feasible Points for Root Ring Geometry) holds strictly on the mkfs device set.**
- ⚠️ **P = 8191 is merely "prime and larger than any reasonable device count", not searched**; E48 (Feasible Points for Root Ring Geometry) does not search P, nor test devs > 8191.
- ⚠️ **Failure domain = one device**, sharing the same limitation with E46 (Root Ring Region Spacing and Failure Domains) / E47 (Root Ring Failure Domain Losses):
  Only judges whether geometrically landing on the same disk, not common causes such as controllers / backplanes / power supplies.
- ⚠️ Criterion b sets `K = 2` (lower bound of root ring depth). When K is larger, intra-region contiguous flips to usable on some cells; **E48 (Feasible Points for Root Ring Geometry) did not sweep K**.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 2 | Supports | 2026-09-19 Unaffected: All 9 feasible points are "prime stride + cross-region rotation", serving as basis for region placement and slot ordering lines |
| D16 (Publish Semantics) settled item 6 | Supports | 2026-09-20 Unaffected: Rotation key in the mechanism is called txg and increments per publish; all 9 feasible points rest on this reading—invalidated retrospectively if "checkpoint_txg increments by 1 per publish" fails |
| D22 (Synthesizing Unit Atomicity) unsettled item 6 | Material | 2026-09-19 Unaffected: P is odd prime, region spacing not divisible by power-of-two zone_size, serving as arithmetic input striking down option B "alternating zone pairs plus zone reset" |
| D2 (RAID Stripe Policy) | No effect | 2026-09-19 Unaffected: Settled item 5 settled root ring no re-placement, E48 (Feasible Points for Root Ring Geometry) guarantee holds strictly at mkfs time |

## Historical Versions

### 2026-08-30

- Run completed, 9 feasible points found. Criteria unchanged; mutation testing round 1 exposed two blind spots (feasibility judgment and pigeonhole criteria both in `main()`),
  extracted to pure function `judge()` and added two absolute value assertions, catching all 8.
- Established. Rationale: E47 (Root Ring Failure Domain Losses) pointed out "what needs to be settled is not region count, but selection rule for region locations plus slot ordering",
  and D22 (Synthesizing Unit Atomicity) settled item 2 required a feasible point before settling.
