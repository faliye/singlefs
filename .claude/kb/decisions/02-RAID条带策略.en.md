## D2 RAID Stripe Policy —— Semi-settled (1 item unsettled)

[中文](02-RAID条带策略.md) · **English** · [日本語](02-RAID条带策略.ja.md)

D2 (RAID Stripe Policy) governs the striping aspect: how wide a stripe is, which disks it uses, how a unit is placed into a stripe, where the membership table lives, how partial release and degradation are handled, what path changes to device sets follow, and the granularity of writes issued to devices along with slot spacing for fixed structures. Out of scope: fields and widths of position entries themselves belong to D19 (Block Pointer Structure & Width Budget) settled item 1 / settled item 4; slot geometry of root slots and system configuration slots, and address space of attribution formulas belong to D22 (Unit Atomicity Synthesis) settled item 2 and settled item 16; media branches and the "device → layout" table belong to D12 (Target Media); which slot a write lands in belongs to D3 (Space Allocation) settled item 8; admission inequalities belong to D28 (Mount-Time Commitments) settled item 1.

### Settled Items

**D2 (RAID Stripe Policy) settled items use a single numbering sequence**, continuous with the "Unsettled Items" section, never reordered.

| # | Item | Settlement |
|---|---|---|
| 1 | **Granularity of "variable width"** | Variable per write: on format side, physical locations enumerated individually with device identities, without referencing chunk mapping tables; allocation side rejects fixed-within-group **Status: Settled.** **Progress: Implemented.** |
| 2 | **Whether disks must be equal-sized** | Disks need not be equal-sized, strictly within sub-problem of addressing correctness; pool members and disk geometries validated at mount, refusing mount if safe combination cannot be judged **Status: Settled.** **Progress: Implemented.** |
| 3 | **How geometry is re-evaluated after device set changes** | Follows explicit "geometry re-evaluation" transaction published atomically, volume remains mounted, forced remount forbidden **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Shape of geometry re-evaluation transaction** | Refuse disk addition if geometry unqualified, switch geometry only after draining, run a background resumable rebalance after disk addition, migration unit is whole object **Status: Settled.** **Progress: Unimplemented.** |
| 5 | **What to do if root ring failure domain is invalidated after device set changes** | Root ring is not relocated; rebalance moves user data without touching root ring, accepts degradation when disks drop below devs < R **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **How many columns allocator uses per write** | `w = clamp(batched write volume + 1, 2, 4)` then clamped by currently writable device count, bucketed by slot width; upper bound 4 is constant declared in system configuration **Status: Settled.** **Progress: Implemented.** |
| 7 | **Which devs is taken in root ring attribution formula** | Attribution is no longer computed, mkfs explicitly stores device identity per region; version 1 hardcodes three regions as 0 / 1 / 0, not an mkfs parameter **Status: Settled.** **Progress: Implemented.** |
| 8 | **Which devices participate (how subset is chosen)** | Select a group from the g emptiest disks upon object creation, thereafter choosing w emptiest disks strictly within group; reselect group when disk fills, group not stored on disk **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **How many disks run in version 1** | First runnable target runs on 2 disks; single disk follows independent track, settled when that track opens **Status: Settled.** **Progress: Implemented.** |
| 10 | **Whether unit spans w columns or lands entirely in one column** | Lands entirely in one column, neither split nor spanning; position entry carries one per replica **Status: Settled.** **Progress: Implemented.** |
| 11 | **Stripe lifecycle (how partial release is handled)** | Partial stripes pinned, background lazy reallocation; member slots cannot be reused prior to repair, w = 2 stripes reclaimed entirely upon single member death **Status: Settled.** **Progress: Unimplemented.** |
| 12 | **Parity column location and discovery mechanism** | Membership table fixed 56 bytes lives in Code 3 packed records, identified by container index, container chain strictly takes `w = 2` mirror **Status: Settled.** **Progress: Unimplemented.** |
| 13 | **Whether writes allowed or read-only during degradation** | Read-only: writable mount requires exclusive open of majority of devices, transitions to read-only if running disks fall below w lower bound or below majority **Status: Settled.** **Progress: Unimplemented.** |
| 14 | **What key the logical stripe table tree takes** | Takes placement (device identity 4 + 16 KiB slot number 6), sharing identical coordinate system with allocation record tree in D3 (Space Allocation) settled item 3 **Status: Settled.** **Progress: Unimplemented.** |
| 15 | **General criterion for "objects with different lifecycles"** | Death trigger event strictly identical commit counts as same lifecycle, sharing permitted; settled item 17 scope narrowed to fixed structures within io_min **Status: Settled.** **Progress: No code object.** |
| 16 | **Do not issue writes smaller than `io_min`** | Issuing smaller writes into device physical mapping unit induces device-internal RMW; this quantity is `io_min`, not `physical_block_size` **Status: Settled.** **Progress: Implemented.** |
| 17 | **Do not allow two objects with different lifecycles to share same physical mapping unit** | Criterion provided by settled item 15; scope shields only fixed structures within io_min, 16 KiB nodes and 32 KiB units excluded **Status: Settled.** **Progress: Implemented.** |
| 18 | **Whether w_max and g in system configuration write 4 or 2 in version 1** | Both bytes write 4, g adds runtime clamping; version 1 disk addition acceptance refuses device set changes reaching `w ≥ 3`, devs capped at 2 **Status: Settled.** **Progress: Implemented.** |
| 19 | **Slot spacing of root slots and system configuration slots** | Slot spacing = 4096 rounded up to integer multiple of io_min probed at mkfs; io_min enters mount comparison list, version 1 sets only gate (a) **Status: Settled.** **Progress: Implemented.** |
| 20 | **Whether stripe table and container index register day-1 in tree table** | Do not register; allocate new tree ID and kind code when `w ≥ 3` opens **Status: Settled.** **Progress: Implemented.** |
| 22 | **Full-stripe write, never read-modify-write** | Variable stripe width, every write is full-stripe write; write hole is not repaired, designed not to exist **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Granularity of "Variable Width"

**Settlement**: Variable per write. On format side, **physical locations are enumerated individually with device identities** (D19 (Block Pointer Structure & Width Budget) settled item 1), without introducing the indirection layer of "logical address + chunk mapping table"; allocation side is likewise variable per write, rejecting "fixed within group".

**Scope**: Settles granularity and "not introducing a second authoritative structure", without settling how many columns per write (settled item 6), nor which disks participate (settled item 8). Basis is not performance: this project never measured performance of stripe widths (the sole stripe-related experiment in `.claude/kb/experiments.md`, E55 (Stripe Mapping Arithmetic on Real Devices), tested correctness of mapping arithmetic), and `.claude/rules/fs-design.md` explicitly forbids "this is faster" from criteria. The sole format benefit of fixed-within-group (pointers need not carry dev, can use chunk mapping table) was independently claimed by D19 (Block Pointer Structure & Width Budget) settled item 1, whose three bases (paying removal cost in D1 (Data Mobility / Reverse Index), pointer without dev needing bootstrap structure like `sys_chunk_array`, low cost of wide pointers) never cite D2 (RAID Stripe Policy) granularity.

**Basis**:

- Three-way verification (background materials `_d2-item1-background.md`, `d2-item1-local-counterexample.md`, local leg raw output `d2-item1-local.out`): Format side has no valid objections; reverse leg checked [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) and [checks-owed.md](../checks-owed.md) finding nothing relying on "uniform width within group", btrfs attaching `num_stripes` / `type` / `stripe_len` to chunks (`struct btrfs_chunk` in `include/uapi/linux/btrfs_tree.h`) buys solely field savings, discounted per `.claude/rules/fs-design.md` "comparing who saves more constitutes no criterion"; forward leg pointed out chunk mapping table is a **second authoritative structure**, can diverge from pointers, requires atomic update with allocator in same transaction, whereas individual enumeration wins on checker determinism and crash-point replay boundedness, while noting fixed-within-group buys a second benefit—collapsing the reading of mutable "currently writable device set" from per-write to group creation, incorporated into settled item 6; local leg counterexamples rested on erroneous premise that "logical offset deduces physical location via stride arithmetic", rejected.
- User decision (allocation side also takes variable per write), original wording in history.
- No experiment: Stripe width granularity has no measurable quantity; what bears weight is the mechanism argument that "chunk mapping table is a second authoritative structure".

**Owed check**: None.

#### Settled Item 2: Disks Need Not Be Equal-Sized

**Settlement**: Disks in pool need not be equal-sized. Pool membership set and disk geometries are validated at **mount time**, refusing mount if safe combination cannot be determined.

**Scope**: Holds strictly within sub-problem of "addressing correctness". How full-stripe writes select "currently writable device set" after smaller disks fill first, and how stripes degrade, is not covered by settled item 2; those inputs belong to settled item 6 and settled item 8, and settled item 2 must not be used to override them.

**Basis**:

- Conjunction of two settled clauses: D12 (Target Media) settled item 3 criterion states verbatim: "**whether a geometric quantity can be heterogeneous in pool depends on whether address arithmetic using it already carries device identity**", and D19 (Block Pointer Structure & Width Budget) settled item 1 settled "position entry **carries** device identity" ⇒ physical locations are enumerated explicitly, leaving no room for deducing locations via geometric formulas.
- External fact (Linux 6.17 inspected on 2026-08-27, unverified in this project): md/raid0 5-year silent corruption chain was triggered precisely by "omitting device identity, deducing via formulas".
- No experiment: This item is a conjunction of two settled clauses, with no measurable quantity in this project.

**Owed check**: None.

#### Settled Item 3: Device Set Changes Follow Explicit "Geometry Re-Evaluation" Transaction

**Settlement**: After adding / replacing / dropping disks, geometry computed at mount is re-evaluated by an **explicit transaction** and published atomically, with the **volume remaining mounted**; forced remount rejected.

**Scope**: Settles direction, what the transaction looks like is in settled item 4; the eliminated third outcome (runtime geometry modification window, violating `.claude/rules/fs-design.md` "branch variables must not change mid-operation") is closed by mutual exclusion semantics of settled item 4 b, not settled item 3. Dropped disks share domain with [pitfalls.md](../pitfalls.md) item 9 (degraded mount writes single-copy chunk, exacerbating corruption), while full-stripe writes + individually enumerated position entries with device identities make stripes written during degradation self-proving on disk, eliminating "silence" rather than "permission"—atomicity does not restore redundancy; whether writes or read-only are permitted during degradation is settled in settled item 13.

**Basis**:

- User decision (forced remount is extremely unfriendly to users, rejecting that path), original wording in history; recorded as mechanism deduction per `.claude/singlefs-ai-sop/rules/evidence-discipline.md`, without three-way verification.
- No experiment: Settles only direction, with no measurable quantity.

**Owed check**: None.

#### Settled Item 4: Three Shapes of Geometry Re-Evaluation Transaction

**Settlement**: Three components——

| # | Settlement | Elaboration |
|---|---|---|
| a | **Refuse if geometry unqualified, without pool-wide re-evaluation** | Acceptance judgment for adding / replacing disks is **isomorphic** to D21 (Boundary Between Authoritative and Derived States) mount validation: returns "acceptable / unacceptable", never "a bit slower". ⚠️ **Checks whether geometric quantities safely combine** (`io_min`, atomic width, etc.), **not capacity**—settled item 2 settled disks need not be equal-sized; adding larger disks must be accommodated. ⇒ D21 (Boundary Between Authoritative and Derived States) "runtime checks without recalculating" unchanged |
| b | **Switch geometry only after draining** | Re-evaluation transaction: close gate (queue new writes) → drain (wait for all in-flight writes to land) → atomically publish new geometry → open gate. ⇒ Entire system has **only one geometry** at any instant, and **zero operations straddle the switch point**. Cost is a bounded pause, not unmount |
| c | **Run background resumable interruptible rebalance after adding disks** | Resides in `.claude/rules/fs-design.md` third cell (background, non-decision path, **no commandments**) ⇒ foreground unblocked, not conflicting with settled item 3 "no service disruption". Unbounded heavy work ⇒ follows "write intent → batching → resumable". **Migration unit is "whole object relocated to a new group", not individual stripes** (basis in settled item 8) |

**Scope**: b converts C55 (Switching Geometry Before Drain Completes) from "must prevent" into "structurally impossible", leaving solely the check "did drain truly complete"; the window forbidden by `.claude/rules/fs-design.md` "branch variables must not change mid-operation" is closed only by b. Pause in b must be bounded, yet upper bound is absent: drain waits for "in-flight writes", whose quantity is calculated at mkfs per I-8.1 (Root Ring Geometry Sufficient), not yet bridged to transaction layer. c and resident compaction in D3 (Space Allocation) settled item 14 are **two separate tasks**: compaction handles fragmentation, rebalance handles inter-disk balance. When citing c, include this qualification: stranded capacity belongs to criterion 5 in E60 (Cost of Not Migrating Data After Adding Disks), which requires reporting without setting threshold ⇒ settlement direction of c was provided by an unthresholded observation; "how much is unacceptable" was never settled in this project; asserting E60 (Cost of Not Migrating Data After Adding Disks) judged rebalance necessary by criteria is forbidden until stranded capacity is formulated into a thresholded criterion.

**Basis**:

- E60 (Cost of Not Migrating Data After Adding Disks): When not migrating data, new disks absorb only "sum of remaining free space on other disks", an arithmetic upper bound independent of allocation policy; filling 4 disks to 90% then adding an equal-sized disk leaves 60% of new disk stranded under best non-migration policy, 80% stranded at 95% fill on old disks ⇒ cost metric is not "slow", but that added disk is mostly unusable, necessitating rebalance in c.
- User decision (for all three a / b / c), original wording in history.

**Owed check**: C2 (Unbounded Operations Made as Single Transaction): c's unbounded work backed by it; C55 (Switching Geometry Before Drain Completes): leaves solely the check "did drain truly complete" after b.

#### Settled Item 5: Root Ring Is Not Relocated

**Settlement**: After adding / replacing disks, **root ring regions are not relocated**; rebalance in settled item 4 c moves user data without touching root ring.

**Scope**: Costs differ in two directions, and are not identical. Adding disks (larger devs) does not cause disk collisions (P prime and larger than any reasonable device count ⇒ `gcd(P, devs) = 1` ⇒ multiplication by P is bijection modulo devs, attribution mutually distinct as long as `R ≤ devs`); real issue was "calculating under current devs points to another disk where that data does not exist", eliminated by settled item 7—attribution converted to explicit device identity written at mkfs, so adding disks changes attribution of no existing region; collisions reduce solely to pigeonhole when `R > devs` at mkfs, which mkfs can detect and refuse. **Only one degradation remains**: the disk hosting the region is lost—that is loss, not collision. Dropping / replacing with fewer disks (devs drops to `< R`) inevitably collides by pigeonhole principle, regardless of placement rules; E47 (Root Ring Failure Domain Loss) / E48 (Feasible Points of Root Ring Geometry) guarantee of "surviving loss of any disk, worst-case 1 generation rollback" fails here. Region bytes are a fait accompli carved on disk at mkfs.

**Basis**:

- E62 (Root Ring Attribution: Storing Identity vs Computing Formula): Formula arm pointed to wrong disk 2 / 4 / 6 / 17 / 19 times across 8 device set change steps, identity arm had 0 errors across five rounds, extra reads 0, byte cost 4 bytes ⇒ cost in disk addition direction eliminated by settled item 7, leaving only dropped disk cell for non-relocation; negative control collided on both arms when `R > devs` at mkfs; storing identity cannot resolve pigeonhole, failure domain half borne by settled item 5.
- User decision (not relocating root ring after adding / replacing disks), original wording in history.

**Owed check**: None.

#### Settled Item 6: How Many Columns Allocator Uses Per Write

**Settlement**: `w = clamp(batched write volume + 1, 2, 4)`, then clamped by "currently writable device count". **"Batched write volume" is calculated separately per slot width bucket** (settled item 10 basis 2: all slots in a stripe must have uniform width)—16 KiB and 32 KiB buckets in a single publish independently compute their own `w`. **Upper bound 4 is a constant declared in system configuration**, independent of device count; `w ≥ 2` is a hard lower bound (stripes with zero redundancy forbidden).

**Scope**: `w` and group size `g` in settled item 8 are two distinct quantities, strictly satisfying `w ≤ g`; both initial values being 4 is coincidence, changing one does not automatically change the other. Version 1 `w` is strictly 2 and pinned by disk addition acceptance in settled item 18; upper bound 4 in `clamp` is never reached in version 1—it is a **declaration**, not a reachable value in version 1. Weight-bearing basis is `.claude/rules/fs-design.md` hard requirement 5 (branch variables must not change mid-operation: writing upper bound as `f(device count)` changes while write is in flight, dropped disks bypass drain barrier in settled item 4 b) and hard requirement 2 (every branch must be test-enterable: constants close value set to `{2,3,4}`, all three branches reachable), not marginal arithmetic. Three required unknowns: ① marginal arithmetic for "inflection point 4" has **dimensional error**—risk term measured per stripe, cost term per capacity; equidimensional measure is expected data loss proportion `(w−1)/C(D,2)`, **linear not quadratic**, yielding upper bound **6** for D=8 under equal weight ⇒ 4 is on the more conservative side, not pinned by that arithmetic; ② equal weighting (1 percentage point space ↔ 1 percentage point data loss) has no source in kb; λ=1 gives 6, λ=2 gives 3, λ=0.5 gives 5; ③ entire derivation **holds strictly under `parity = 1`**; under parity=2, dual disk failure loses no data, risk axis shifts to triple failure, yielding 7 under same arithmetic ⇒ upper bound resets with parity, and parity count is unsettled item 21.

**Basis**:

- E63 (Allocator Column Count Per Write): When small writes constitute 90%, "fitting write volume" saves 61% physical slots over "constant full width" ⇒ criterion must incorporate write volume; under 8-disk batching, upper bound 4 vs no upper bound (=8) expected loss on dual failure is 10.7% vs 25.0%; uncapping saves only 13% space in exchange for 2.3× expected data loss, with per-stripe hit rate 1.000 ⇒ zero complete files remain, failure mode shifting from "partially damaged" to diffuse.
- Three-way verification: Forward leg accepted narrowed version and caught dimensional error in marginal arithmetic; reverse leg judged "D2 (RAID Stripe Policy) original text required three items together, providing only one number is not a settlement" ⇒ this item settles only criterion and upper bound, participating subset moved to settled item 8; local leg advocated per-pool calculation, whose small-pool counterexample weakened after dimensional correction (3→4 tied at D=4), raw output `d2-item6-local.out`.
- User decision, original wording in history.

**Owed check**: None.

#### Settled Item 7: Root Ring Attribution Explicitly Stores Device Identity Per Region

**Settlement**: Attribution is no longer computed via `(r × P) mod devs`, but written per region at mkfs. Version 1 hardcodes attribution of three regions as **0 / 1 / 0** (Region 0 → Disk 0, Region 1 → Disk 1, Region 2 → Disk 0), written into the 12 bytes of system configuration by mkfs (4 bytes device identity per region); **it is not an mkfs parameter**.

**Scope**: Isomorphic to settled item 1 and D19 (Block Pointer Structure & Width Budget) settled item 1—this project's third replacement of "formula-based location deduction" with "explicit individual enumeration". Buys "never pointing to wrong disk", not "failure domain independence": failure domain guarantees still fail when disks drop to `devs < R`, that half borne by settled item 5. Where fields land in system configuration and how region starting offsets are computed belong to D22 (Unit Atomicity Synthesis) settled item 16; warm-up empty publish count is uniquely determined by this attribution (txg 1 lands in Region 1 = Disk 1, txg 2 lands in Region 2 = Disk 0, two publishes covering both disks), making it a format constant alongside txg 3 of new-pool file creation, see D22 (Unit Atomicity Synthesis) settled item 16 sentence 5. **Modifies new-pool file creation bytes: No**—those 12 bytes of system configuration already wrote 0 / 1 / 0; 0 / 1 / 0 here supplements clauses.

**Basis**:

- E62 (Root Ring Attribution: Storing Identity vs Computing Formula): Formula arm pointed to wrong disk 2 / 4 / 6 / 17 / 19 times across 8 device set changes, identity arm 0 errors across 5 rounds, extra reads 0 (identity co-located with system configuration, which is already read), byte cost 4 bytes (4 regions × 1 byte).
- E87 (Placement of Fixed Structures): "On two disks R=3 inevitably forces one disk to bear two regions"—matching pigeonhole lower bound in D22 (Unit Atomicity Synthesis) settled item 14, 0 / 1 / 0 is the unique canonical layout once region numbering is fixed; making it an mkfs parameter buys only "which disk bears two", showing no observable difference under isomorphic disks.
- Two user decisions (attribution changed to storing identity, version 1 hardcoding three regions as 0 / 1 / 0), latter without three-way verification, overturnable; original wording in history.

**Owed check**: C182 (Two Settled Items Contradict on Root Ring Placement Attribution), C217 (Rationale for P Exceeding Device Count Vanished While Constraint Remains).

#### Settled Item 8: Which Devices Participate

**Settlement**: When creating an object, select a group from the `g` emptiest disks at that moment; thereafter, each stripe of that object selects the `w` emptiest disks strictly within the group (`w` provided by settled item 6); reselect group when any disk in the group fills. `g` is a constant declared in system configuration, initial value 4. **Group is not stored on disk**—deduced from **union** of device identities in object's position entries, zero added bytes. Rebalance in settled item 4 c takes "object" as migration unit, relocating whole group, never migrating stripe by stripe.

**Scope**: `g` and `w` are two distinct quantities, strictly satisfying `w ≤ g`, both initial values being 4 is coincidence; intact proportion tracks solely `g` (`1 − C(g,2)/C(D,2)`), space overhead tracks solely `w` (`1/(w−1)`). Storing a separate "object → disks" mapping is a miniature chunk mapping table, the second authoritative structure judged in settled item 1, while D18 (Information Carried in a Block) settled item 3 forbidden list prohibits physical placement / device ID in block headers ⇒ group is allocator derived state; placement itself is authoritative in central mapping; position entries are hints, while central mapping is sole entry point for dereferencing and release verdicts (D19 (Block Pointer Structure & Width Budget) settled item 5); reads can try that placement directly, though it may be stale. Two required unknowns: ① **per-object concurrency ceiling = `g`**, an object pinned to 4 disks ⇒ concurrency is 4, not device count, an unmeasured dimension, and citing 78.6% as net gain is forbidden until measured given D25 (Target Workload Priority) primary workload is sequential large files; ② only measured `g = w = 4`, where `g=5` yields 64.3%, `g=6` yields 46.4%; remeasurement required if `g` changes.

**Basis**:

- E67 (Which Disks Participate in a Stripe): 8 disks, width 4, 512 objects, exhaustively tested all 28 pairs of dual disk failures; rule settled here yields 78.6% intact object proportion (`1 − C(4,2)/C(8,2)`, pinned by unit test) with 0 inter-disk range, whereas per-stripe round-robin leaves 14.3%, per-stripe emptiest gives 57.1% (uniform) / 77.2% (skewed); binding group by object ID also yields 78.6% but skewed rounds have 88.7% inter-disk range (premature ENOSPC); expected data loss proportion across six arms identical (`3/28`, determined solely by width) ⇒ **selection policy determines how losses distribute, not how much is lost**. Fourth round added 9th disk with rebalance; whole-object migration yielded 83.3% (`1 − C(4,2)/C(9,2)` theoretical value, zero loss), whereas per-stripe migration dropped to 75.0% ⇒ basis for "migration unit is object".
- Three-way verification: Forward leg accepted narrowed version (group unpersisted, rebalance per object, `w<g` selects `w` emptiest in group) and identified `g` and `w` as two separate knobs; reverse leg demanded testing rebalance collision with "fixed thereafter", supplemented (E67 (Which Disks Participate in a Stripe) round 4); local leg counterexample "disks not equal-sized ⇒ may pin to 4 small disks" failed to materialize in heterogeneous capacity rounds (group by remaining space matched per-stripe emptiest cell-for-cell, 1148 vs 1148), output recorded as flawed vote due to splicing corruption, raw output `d2-item8-local.out`.
- User decision, original wording in history.

**Owed check**: None.

#### Settled Item 9: First Runnable Target Runs on 2 Disks

**Settlement**: Version 1 does not run single disk; first runnable target runs on 2 disks; single disk follows an independent track, settled when that track opens.

**Scope**: Eliminates conflict: **`w ≥ 2` in settled item 6 (zero redundancy stripes forbidden) conflicts on single device with "clamped by currently writable devices"**, the former requiring 2 columns while the latter provides only 1. Concurrently moves D18 (Information Carried in a Block) settled item 4 (single-disk self-recovery shape) out of blocking set: that item opens alongside "single-disk layout" track, writing zero bytes before then; ⚠️ that track must first provide burden of proof upon opening—**no production implementation takes "arbitrary-point forward-backward chained recovery" path**, burden of proof rests here per `.claude/singlefs-ai-sop/rules/evidence-discipline.md`, where proof has not begun. Achievable: QEMU apparatus in `.claude/kb/vm-harness.md` synthesizing two disks suffices without real hardware.

**Basis**:

- D2 (RAID Stripe Policy) settled variable-width full-stripe requires ≥2 disks ⇒ single disk never belonged to version 1 layout; bypassing requires opening separate layout for single disk (D12 (Target Media) settled each layout occupies one incompat bit), while `.claude/rules/fs-design.md` states verbatim: "when criteria cannot be written, implement only one path first, and **implement slow but correct one first**".
- User decision, original wording in history.
- No experiment: This item is a ruling on mutual exclusion of two settled clauses on single device, with no measurable quantity.

**Owed check**: None.

#### Settled Item 10: A Unit Lands Entirely in One Column

**Settlement**: A 32,768-byte unit is never split, never spanning multiple columns; position entries carry one per replica.

**Scope**: Basis is one arithmetic plus two settled clauses. **Slot = unit** (D4 (Checksum Location) settled item 2 / 3 pins minimum write to whole unit), and units have two sizes (D18 (Information Carried in a Block) settled item 9: metadata 16,384, data 32,768) ⇒ slots have two sizes; **all slots in a stripe must have uniform width**, required for bitwise XOR to be defined ⇒ allocator batches writes bucketed by slot width; "batched write volume" in settled item 6 calculates per bucket, not total publish volume. Spanning interpretation breaks immediately at `w = 4`: `32768 / 3` is not an integer, nor multiple of `io_min`, violating settled item 16. D19 (Block Pointer Structure & Width Budget) basis 3 calculated pointer footprint ratio assuming "header + 4 + 2 six-column stripe" under spanning interpretation; that arithmetic fails, but proved "fixed-width pointer cost acceptable", **conclusion unaffected**—recalculating per this item's measurement basis thins pointers further (position entries drop from w to 1 per replica). Conjunction of settled item 10 and settled item 6 leaves a hole existing only at `w ≥ 3` (reconstruction inputs for surviving members lost after member release), measured in E80 (Stripe Partial Release) and closed by settled item 11; `w = 2` control is all-zero ⇒ all version 1 tests completely blind to it.

**Basis**:

- User decision, original wording in history.
- No experiment: Spanning interpretation breaks mathematically (`32768 / 3` not an integer); what bears weight is one arithmetic plus two settled clauses, with no measurable quantity.

**Owed check**: C81 (No Rules for Stripe Partial Release).

#### Settled Item 11: Stripe Lifecycle—Partial Stripes Pinned, Background Lazy Reallocation

**Settlement**: Three rules——

1. **After member release in w ≥ 3 stripes, that slot and whole stripe (including parity) must not be reused prior to repair**. Pinned slots count toward "unreclaimable bytes" metric (D5 (Snapshot / Space Accounting Mechanism) settled item 4 item 3, ready placement)—pinned volume is naturally observable, satisfying `.claude/rules/fs-design.md` hard requirement 4.
2. **Repair = background batch reallocation of survivors into a new stripe, old stripe reclaimed entirely**, running under D26 (Background Compaction and Placement Reclamation) leg 1 (resident compaction) queue and budget. Repair queue is **derived state** (reconstructible from allocation records and stripe members; loss is merely slow).
3. **w = 2 stripe members reclaimed entirely upon single member death** (1 data + 1 replica, no cross-unit coupling) ⇒ version 1 (2 disks strictly `w = 2`) satisfies automatically, zero new code paths.

**Scope**: "Repair" in settled item 11 governs solely one category—**release of stripe members**. Disk failure rebuild of a lost slot is distinct, shape unsettled (requiring membership table in settled item 12); the two cannot be conflated: rule 2 "reallocating survivors into **one** new stripe" holds strictly for release (releasing down to k survivors yields `w = clamp(k + 1, 2, 4)`, a `w`-wide stripe holding exactly k data slots, `k ≤ 3`), while unexecutable verbatim under degradation—`w` clamped by currently writable devices (settled item 6); after 4-disk drops 1 disk `w ≤ 3`, new stripe holds only 2 data slots, unable to fit 3 reallocating units ⇒ disk rebuild category must be settled separately, landing in settled item 12 and successors. Costs and known unknowns, cited together: +47% extra write indicates order of magnitude under uniform random rewrite workloads, not prediction; lower bound of repair budget unmeasured (budget 2 barely tracks model workload); more economical form "merging survivors into foreground batching" left to implementation, unmodeled in E80 (Stripe Partial Release).

**Basis**:

- E80 (Stripe Partial Release) (two rounds, 5 seeds × 2 workloads): Among four disposals, only "pinning + background lazy reallocation" simultaneously achieves zero loss, bounded pinning (budget 2 per round yields zero at round end, draining to zero upon stopping load), and ~37% cheaper than immediate repair (+47% vs +75% extra writes); unconstrained reuse averages 9.8–13.0 lost live units per failure with zero errors before disk death, deferred reclamation unbinds 33–46% pinned live data, immediate repair regenerates repair debt (~4000 repair events).
- User decision, original wording in history.

**Owed check**: C81 (No Rules for Stripe Partial Release): image-side checks "stripe members of any live unit either have original bytes or whole stripe dead"; C127 (Pinned State Lacks Placement in Allocation Records).

#### Settled Item 12: Membership Table Lives in Code 3 Packed Records, Via Container Index, Container Always Mirrored

**Settlement**: Stripe membership records (fixed 56 bytes, one per slot including parity) live in **Code 3 packed record units**, identified by a **container index**; **container chain strictly takes `w = 2` mirror**. State class is **authoritative state**: per D21 (Boundary Between Authoritative and Derived States) criterion "loss loses data / loss is merely slow", losing this table prevents rebuilds and leaves allocator unable to judge pinning ⇒ must reside in unit payload, remaining first item on authoritative list without expanding scope. **Modifies new-pool file creation bytes: No** (version 1 2-disk strictly `w = 2`, parity is second replica, position entry per replica covers discovery ⇒ new-pool file creation writes zero stripe records).

**Scope**: Settles carrier, not ordering—key ordering of logical stripe table tree listed separately in settled item 14. **Why container is always mirrored**: without mirroring, steady-state unsolvable slots under single disk failure are zero, but **strictly 1 when pool newly created**—when stripe table has only one container, it lands within the very stripe it describes; disk containing container failing renders it unsolvable under any order, hitting across all 5 seeds, structurally not probabilistically; mirroring structurally closes this bootstrap loop. Reason is neither space nor written bytes: space discounted per `.claude/rules/fs-design.md` "comparing who saves more constitutes no criterion"; written bytes dimension is tied (primary workload differs by 4.7%, mixed workload byte ratio 0.7816 saving 22%, but round-wise arm A cheaper on only 65.8% of publishes) ⇒ chosen because it provides definitive answer to "can slots on failed disk be rebuilt after single disk failure", while constant `w = 2` full mirroring completely abandons `w ≥ 3`.

Costs itemized:

| Dimension | Number | Citation |
|---|---|---|
| Steady-state space | **3087 ppm, identical for `w = 3` and `w = 4`** (16 TiB / 90% utilization, mirrored; 1543 unmirrored)—entry count fixed by 90% budget, `w` shifts ratio of data to parity | E111 (Stripe Table Key and Geometry), bit-for-bit identical to 54,315,515,904 bytes in E106 (Stripe Membership Table Carrier) arm A-mirror |
| Written bytes per publish ÷ "`w` constant 2 full mirror" | Primary workload **expected 0.9531** (saves 4.7%); container nearly full cell 1.050 | E110 (Expected Write Amplification of Stripe Table Under Continuous Publish) |
| Stripe table share | **163,840 bytes / publish, independent of leaf count** (container 32,768 + container index 3 levels × 16,384, whole chain mirrored) | E107 (Write Amplification of Stripe Table on Primary Workload) |
| Latency of three-stage write ordering | ack semantics difference within noise; durable semantics **1.625×** | E107 (Write Amplification of Stripe Table on Primary Workload) arm B |
| Unsolvable slots on single disk failure | **Structurally 0** | E106 (Stripe Membership Table Carrier) arm A-mirror |

**Basis**:

- E106 (Stripe Membership Table Carrier): Steady-state unsolvable slots 0 when membership table lives in Code 3 packed records, but strictly 1 when pool newly created across all 5 seeds ⇒ container must mirror; steady-state space across three arms in same artifact.
- E107 (Write Amplification of Stripe Table on Primary Workload): Extra bytes written and three-stage ordering latency from maintaining membership table, which E106 (Stripe Membership Table Carrier) explicitly left unanswered.
- E110 (Expected Write Amplification of Stripe Table Under Continuous Publish): Resolved knife-edge in E107 (Write Amplification of Stripe Table on Primary Workload)—after container slot utilization evolves with workload, cumulative ratio for primary workload stabilizes, container-full endpoint representing tiny fraction of publishes.
- E111 (Stripe Table Key and Geometry): Entry count ordered by placement independent of `w`, steady-state space bit-for-bit identical to E106 (Stripe Membership Table Carrier) arm A-mirror ⇒ cost row closed.
- User decision, original wording in history.

**Owed check**: Four checks required before opening `w ≥ 3`—C103 (Container Index Lacks Placement) (where container index tree lives, key encoding, update costs, MAC recomputation during rebuild); **bootstrap** (what identifies container index nodes, requiring **new reserved tree ID registered** to close, while settled item 20 settled not registering day-1, acquiring new ID upon opening); C127 (Pinned State Lacks Placement in Allocation Records); C128 (Scavenge Admission Indeterminate for Both Parity Arms). Plus C303 (D2 Boundary ③ Rationale Invalidated by D9 Item 7 Withdrawal) and C155 (Cross-Experiment Shared Quantities Lack Inter-Assertion).

#### Settled Item 13: Read-Only During Degradation

**Settlement**: **Read-only** during degradation. Criterion formulated as conjunction for writable mount acceptance, paired with runtime gate:

| Moment | Criterion |
|---|---|
| Mount acceptance | Writable device count ≥ w lower bound ∧ **exclusive open of majority of disks in pool** (majority is threshold; new instance number written to **all** successfully exclusively opened disks, not merely majority) ∧ instance table readable ∧ instance switch reservation obtainable (formula in D28 (Mount-Time Commitments) settled item 3); any failure results in read-only mount; exception: space criterion (instance switch reservation and D28 (Mount-Time Commitments) settled item 1 admission) insufficient prior to number acquisition, follow D16 (Publish Semantics) settled item 1 "admission" row: write row first, push empty publish raising F, re-judge; if still insufficient after full pushes, mount succeeds as writable, subsequent publishes judged by publish admission reporting ENOSPC on shortfall, normal unmount proceeds as usual (user settled 2026-09-26, zero attack rounds) |
| Runtime | Writable device count drops below w lower bound, **or successfully exclusively opened devices drop below majority**, transitions to read-only. **Measurement basis of "writable device count"**: a device is unwritable if and only if single probe write to its fixed placement fails (identical action to D23 (Role and Format of Journal) settled item 14 two-branch discriminator)—neither failure count nor failure duration used, as neither judgeable at fault instant |

**Re-judge upon device set changes**: Added to "geometry re-evaluation" transaction in settled item 3—re-judge majority and write set for number acquisition after device set changes, transitioning to read-only upon failure.

**Scope**: Settles "whether mountable as writable during degradation, and when transitioning to read-only after mounted", not "how stripes written during degradation are patched"—that belongs to partial stripe pinning and background lazy reallocation in settled item 11. **Why majority rather than "sufficient writable devices"**: C113 (Multi-Version Unit Current Version Verdict Lacks Input During Scan Reconstruction) settlement requires strictly one writable mount at any instant (two concurrent writable mounts produce identical instance numbers, unseparated by any of three branches of published predicate); block device exclusive open is **per-device** mutual exclusion, which elevates to **pool-level** mutual exclusion only by requiring device sets of any two writable mounts to intersect, where majority is the loosest condition; when disks ≤ 3, any two sets satisfying w lower bound already intersect; when disks ≥ 4, disjoint 2-element subsets exist, where majority is the sole remaining gate. Cost: 4-disk pool with 2 writable disks cannot mount as writable, and transitions to read-only if dropping to 2 disks while mounted, whereas both scenarios writable under old criterion (only requiring writable devices ≥ w lower bound)—trading availability for "two timelines never sharing an instance number", forbidden from being parenthetically mischaracterized elsewhere as "same orientation as settled item 9"; settled item 9 addresses how many disks run in version 1, unrelated to writable degradation. Without re-judging, online disk additions elevate pool cardinality while mount's held exclusive set remains unchanged, leaving healthy disks never opened by this mount unable to obtain new instance numbers, invalidated and resynchronized entirely under "disks that missed number acquisition".

**Basis**:

- E141 (Mount Acceptance Self-Proof of Switch Reservation): Conjunction "instance switch reservation obtainable" separates writable from read-only across two pools differing by 1 disk; setting reservation to 0 makes both writable ⇒ conjunction has discriminatory power, not ornamental.
- Settled alongside C113 (Multi-Version Unit Current Version Verdict Lacks Input During Scan Reconstruction) (confirmed by reverse leg 5.1 and 7.2 inspection in rounds 7 and 8) and user decision, original wording in history.
- Space exception: user decision, original wording in history; implementation and test cases in C565 (Unsettled How to Close When Mount Full Pushes Insufficient) row in cleared table.

**Owed check**: C833 (Writable Mount Lacks Exclusive Open).

#### Settled Item 14: Stripe Table Ordered by Placement, Sharing Coordinate System with Allocation Record Tree

**Settlement**: key = **placement** (device identity 4 bytes + 16 KiB slot number 6 bytes), **sharing identical coordinate system** with allocation record tree in D3 (Space Allocation) settled item 3 (D3 (Space Allocation) settled item 3 basis 2 verbatim: "never settling independently"). **Modifies new-pool file creation bytes: No** (version 1 2-disk strictly `w = 2`, zero stripe records written ⇒ tree empty in new-pool file creation, writing zero bytes regardless of key; must exist before issuing first `w ≥ 3` stripe).

**Scope**: Deduced directly from definitions of three candidate keys, not measured—three consumers ask **identical question**: "given a placement, which stripe does it belong to": allocation acceptance (C127 (Pinned State Lacks Placement in Allocation Records)) judges whether candidate placement is pinned slot; scavenge admission (C128 (Scavenge Admission Indeterminate for Both Parity Arms)) judges whether headerless parity slot should be cleared; concession zone A issuance criterion excludes pinned slots and all slots of their stripe ⇒ ordering by stripe ID or container identity fails to answer this question; container index cannot answer, its key being `(birth tree ID, packed record type, container number, container birth generation)`, lacking placement, with container numbers monotonically allocated as "max + 1" per C103 (Container Index Lacks Placement), having no functional relationship to placements covered by its records. Space dimension excluded from criteria (`.claude/rules/fs-design.md` "comparing who saves more constitutes no criterion"), presented solely for awareness of cost; what enters criteria is "whether question can be answered". Still owed: where container index lives (C103 (Container Index Lacks Placement))—this tree ordered by placement has Code 3 container leaves, whose physical pointers are held by container index, while container index placement is undefined repo-wide; update costs of this tree (records rewritten and nodes COW per publish) left unanswered by E111 (Stripe Table Key and Geometry), residing in scope of E107 (Write Amplification of Stripe Table on Primary Workload) and C103 (Container Index Lacks Placement).

**Basis**:

- E111 (Stripe Table Key and Geometry): Ordered by placement, `w = 2` writes zero records (0 ppm), `w = 3` and `w = 4` both write 483,183,820 records, tree height 3, occupying 3087 ppm, compared to allocation record tree at 442 ppm ⇒ this tree is 6.98×; entry count independent of `w`, scaling strictly with slots carved from 90% budget.
- User decision, original wording in history.

**Owed check**: C103 (Container Index Lacks Placement), C127 (Pinned State Lacks Placement in Allocation Records), C128 (Scavenge Admission Indeterminate for Both Parity Arms).

#### Settled Item 15: General Criterion for "Objects with Different Lifecycles"

**Settlement**: For two objects X and Y sharing same physical mapping unit, identify their respective **death trigger events**—the specific commit transitioning bytes from "must remain static" to "may be overwritten / reclaimed". **If format guarantees death trigger events of X and Y are strictly identical commit, lifecycles are identical, sharing permitted; whenever format permits death events to occur in independent commits that may diverge, lifecycles differ, sharing forbidden.** Criterion anchors on "death", not "birth". Concurrently narrows settled item 17 scope to fixed structures within io_min (root slots, system configuration slots, tombstones, etc.); 16 KiB nodes and 32 KiB units excluded, **zero changes to allocator alignment**. **Modifies new-pool file creation bytes: Yes**—its ruling directly determines slot geometry of D22 (Unit Atomicity Synthesis) settled item 2, while root ring regions are fait accompli carved on disk at mkfs, unalterable.

**Scope**: **Cannot judge by "write moment"**—same-generation tombstone records appended individually, judging by write moment yields "different lifecycles", contradicting user decision in D18 (Information Carried in a Block) settled item 10. Both known consumers verified: tombstones—death event for same-generation records is single verdict "entire generational deadlist retired" ⇒ same lifecycle, matching D18 (Information Carried in a Block) settled item 10; root slots—slot sequence in D22 (Unit Atomicity Synthesis) settled item 2 rotates across regions (region = `txg mod R`) ⇒ single txg writes single slot ⇒ subsequent overwrite of different slots represents independent future commits ⇒ judged "different", yet deduction "different ⇒ must change slot width ⇒ conflicts with D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment)" breaks at first step: slot width and slot start offset are separate lines in D22 (Unit Atomicity Synthesis) settled item 2; elevating start offset to `io_min` (granularity parameter in I-7.5 (Root Slot Aligned by Verdict Width)) satisfies settled item 17 while slot width remains `physical_block_size`, avoiding D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) corollary 3. Rescan revealed far more than two consumers (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` "establish new criterion, rescan existing items immediately"): index nodes (16 KiB) and data units (32 KiB) on arrays with `io_min` = 64 KiB both judged "different", where COW constitutes no exemption (block reuse causes adjacent slots to be rewritten arbitrarily later), and journal records even more rigid than root slots (two-disk mirror only); inspection of [invariants/02-根环.md](../../../crates/singlefs-checker/invariants/02-根环.md) reveals `io_min` appears solely in I-7.5 (Root Slot Aligned by Verdict Width) (verdict width takes max(`physical_block_size`, `io_min`), scope note stating judged by `io_min` when `io_min` > `physical_block_size`), I-7.5 (Root Slot Aligned by Verdict Width) implemented (pool-level checker judges root slot start offsets per slot), "physical mapping unit" still zero hits ⇒ checker judges settled item 17 solely for root slots, omitting index nodes, data units, and journal records, while settled item 16 unjudged. Alternative (literal allocator alignment change) rejected; placement policy in D3 (Space Allocation) settled item 8 (user data avoids clustered segments, lowest free slot in chosen device, ignoring alignment). Settled item 15 and C212 (Fixed Structure Writes Smaller than io_min) are distinct: latter presses on settled item 16, which contains zero undefined words, representing two settled clauses in conflict rather than absent criterion.

**Basis**:

- Three-way verification round 2026-09-10: Criterion provided by forward leg, main agent verified across individual consumers (tombstones and root slots), materials and local leg output in `_d2-item15-background.md` and `d2-item15 leg report in git`.
- External fact (2026-09-08 inspection, unverified in this project): ZFS root slot width takes `MIN(MAX(vdev_ashift, UBERBLOCK_SHIFT), MAX_UBERBLOCK_SHIFT)` verbatim, tracking device alignment capped at 8 KiB, using `ashift` rather than `io_min`, see [prior-art.md](../prior-art.md) "8.1 ZFS: Root slot width tracks device granularity capped at 8 KiB, reducing slot count rather than expanding footprint"; known difference is ZFS ring fixed at 128 KiB, whereas region size in this project is not format constant.
- User decision, original wording in history.
- No experiment: Criterion itself derived from definitions, verified across consumers by substitution, with no measurable quantity.

**Owed check**: C254 (Physical Mapping Unit Consumers Far Exceed Two, Zero Checker Coverage).

#### Settled Item 16: Do Not Issue Writes Smaller Than `io_min`

**Settlement**: **Do not issue writes smaller than `io_min`.** On Linux this quantity is `io_min`, not `physical_block_size`; **the two quantities must never be merged into one sentence**.

| Quantity | Source in Kernel | What It Is |
|---|---|---|
| `io_min` | `lim->io_min = phys_bs`, `phys_bs = bs * (1 + npwg)` (NPWG = Namespace Preferred Write Granularity) | **Minimum write width avoiding device RMW** |
| `physical_block_size` | `lim->physical_block_size = min(phys_bs, atomic_bs)` (`atomic_bs` from NAWUPF, defaults to logical block if undeclared) | **Power-fail atomic width committed by device** |

**Scope**: Mechanism: **issuing smaller writes into device physical mapping unit causes internal device read-modify-write**. Measurement basis: inspected `nvme_update_disk_info` calculating `physical_block_size` in `drivers/nvme/host/core.c` on 2026-08-28, Linux 7.2.0; comment above `min` discusses solely atomicity ("limit the physical block size to the value of the Atomic Write Unit **Power Fail** parameter") ⇒ `physical_block_size ≤ io_min`; issuing writes at `io_min` may still fall below NPWG inducing RMW. **Modifying this requires three-way verification**; E129 (How Sub-io_min Writes Fail on Real Devices) provides solely observations; conflicts with three settled clauses (root slot width, journal record width, system configuration slot width none settled by `io_min`); conflicts and three candidates recorded in C212 (Fixed Structure Writes Smaller than io_min), none chosen; version 1 mount gate in settled item 19 setting solely (a) is **accepted with awareness**.

**Basis**:

- E129 (How Sub-io_min Writes Fail on Real Devices): ① Position-deterministic torn-write injector produces torn writes (`dm-linear` good segment + `dm-error` bad segment, cross-boundary write reports error, prefix lands, suffix lost, 5/5 rounds) ⇒ universal claim "unfalsifiable via fault injection on local host" disproved; half-sentence "dm-flakey cannot produce torn writes" remains valid (produces byte corruption, not torn writes); ② "inducing device RMW" measured—`dm-thin` (64 KiB chunk, `io_min` 65,536, `physical_block_size` 512, non-RAID) under shared chunk issues 129 sectors written to data device for single 512-byte write, 1 sector when unshared.
- User decision and exposure from D22 (Unit Atomicity Synthesis) reverse deduction C ("we do not do read-modify-write" does not cover "we do not induce device read-modify-write"), original wording in history.

**Owed check**: C82 (io_min Excluded from Mount Comparison List), C212 (Fixed Structure Writes Smaller than io_min), C254 (Physical Mapping Unit Consumers Far Exceed Two, Zero Checker Coverage).

#### Settled Item 17: Do Not Allow Two Objects with Different Lifecycles to Share Same Physical Mapping Unit

**Settlement**: **Do not allow two objects with different lifecycles to share same physical mapping unit.** "Different lifecycles" judged per settled item 15 criterion (death trigger events strictly identical commit ⇒ same lifecycle).

**Scope**: **Shields solely fixed structures within io_min** (root slots, system configuration slots, tombstones, etc.); 16 KiB nodes and 32 KiB units excluded, zero allocator alignment changes. Rationale sentences lack empirical backing: neighbors intact across both measured mechanisms—E34 (Root Ring Slot Geometry) real-device half verbatim "adjacent slot B in same chunk (sector 1) **10/10 intact**", fault unit verbatim "crosses legs along 'same stripe row × same sector', **not whole `io_min` unit**"; E129 (How Sub-io_min Writes Fail on Real Devices) precisely tearing `dm-thin` RMW left neighbors in same mapping unit 5/5 rounds byte-for-byte intact, writer receiving error (`dm-thin` intra-chunk COW is **transactional**: allocates new chunk, copies, then switches mapping) ⇒ **"power failure may damage already-persisted neighbor data in same mapping unit" holds or fails per mechanism on real devices, not universal**; supported solely by un-injected mechanism—SSD FTL internal RMW over whole mapping unit, noted in E34 (Root Ring Slot Geometry), unresolved in E129 (How Sub-io_min Writes Fail on Real Devices).

**Basis**:

- E129 (How Sub-io_min Writes Fail on Real Devices): Non-RAID `io_min > physical_block_size` real devices exist and small writes genuinely induce device-side amplification, mechanism premise valid; precisely tearing it in same round left neighbors byte-for-byte intact, universal claim narrowed to per-mechanism evaluation.
- User decision and exposure from D22 (Unit Atomicity Synthesis) reverse deduction C, original wording in history; scope narrowing sentence is user decision in settled item 15.

**Owed check**: C254 (Physical Mapping Unit Consumers Far Exceed Two, Zero Checker Coverage), C212 (Fixed Structure Writes Smaller than io_min).

#### Settled Item 18: System Configuration Writes 4 / 4 for w_max and g

**Settlement**: Both `w_max` and `g` bytes in system configuration write 4 (decision text value); `g` adds runtime clamping `g ← min(g, currently writable devices)`, group being those two disks on version 1 2-disk setup. **Version 1 disk addition acceptance refuses device set changes reaching `w ≥ 3`, `devs` capped at 2 until stripe table opens.****Modifies new-pool file creation bytes: No**—both bytes in system configuration still write 4 / 4; disk addition acceptance is a runtime gate, never triggered in 2-disk new-pool file creation.

**Scope**: `w_max` = 4 is a **declaration** in system configuration, while version 1 disk lacks stripe table—stripes with `w ≥ 3` written out have no structure remembering which disks participated or which slot holds parity (position entries enumerate placement of this data copy, not membership of this stripe), while recovery semantics for that path contain zero text; 2-disk version 1 clamps `w` via `clamp(…, 2, 4)` and writable devices to constant 2, avoiding that path; adding third disk allows `clamp` to yield 3 ⇒ blocked at disk addition step rather than altering `w_max` to 2 (which would modify settled byte and require reverting when opening track). **Writing 4 for `w_max` is conservative choice accepted with awareness, not pinned by arithmetic**: marginal arithmetic setting value had dimensional error (corrected equidimensional arithmetic yields 6 under equal weight), equal weighting lacks source in kb, and entire derivation holds strictly under `parity = 1`; parity count is unsettled item 21; modifying parity alters this byte, requiring D15 (Format Freeze Policy) freeze layers.

**Basis**:

- E63 (Allocator Column Count Per Write): Expected data loss of 10.7% vs 25.0% for upper bound 4 vs unconstrained on 8 disks supports conservative choice of 4; marginal arithmetic claiming 4 as inflection point had dimensional error, not pinning 4.
- User decision (4 / 4 settled day of pre-coding master audit, disk addition acceptance modal decision next day, neither via three-way verification, overturnable), original wording in history.

**Owed check**: None.

#### Settled Item 19: Slot Spacing of Root Slots and System Configuration Slots

**Settlement**: Slot spacing = **4096 rounded up to integer multiple of `io_min` probed at mkfs** (equal to `max(4096, io_min)` when `io_min` is power of two); system configuration geometry section adds two 4-byte fields: "mkfs `io_min`" and "fixed structure slot spacing"; `io_min` enters D12 (Target Media) settled item 4 mount comparison list. Mount gates are **two parallel conditions**, not one: **(a) actual `io_min` ≤ recorded slot spacing**, refusing writable mount upon failure—governs settled item 17 (fixed structures sharing mapping unit); **(b) actual `io_min` ≤ fixed structure write width**—governs settled item 16 (do not issue writes smaller than `io_min`). **Version 1 sets only (a), omitting (b).**

**Scope**: Standard NVMe with `io_min` = 4096, `physical_block_size` = 512 passes (a) (4096 ≤ 4096), while each root slot and system configuration slot write remains 512 bytes < `io_min` ⇒ verbatim violation of settled item 16, **accepted with awareness**; status and three candidates recorded in C212 (Fixed Structure Writes Smaller than io_min), none chosen. Reason for integer multiple rather than `max`: non-power-of-two geometries like `io_min` = 3072 share a mapping unit across two slots (constructed by reverse leg in round 1), whereas integer multiples guarantee any slot width ≤ slot spacing shares no mapping unit. Index-to-offset conversion, root ring base, and region starting points belong to D22 (Unit Atomicity Synthesis) settled item 16 sentence 2. `crates/` implements both fields in system configuration and slot spacing formula (`SystemImmutableSizes::slot_spacing_for` in `crates/singlefs-core/src/system_configuration.rs`, unit test pinning "slot spacing 6144 when io_min = 3072"); comparing probed `io_min` against recorded slot spacing at mount is unimplemented (only caller mkfs parameter equality check against disk records exists). Devices whose `io_min` causes fixed structures to overlap rejected at mkfs, unsupported (user decision rejected support, original wording in history): S = 8 accepts slot spacing 384 KiB, rejects from 512 KiB (root ring region 0 and region 1 overlap); `io_min` = 1 MiB causes system configuration slot 1 and root ring region 0 to both start at 1 MiB, rejected for all S; basis is third section of three-way verdict `research/prompts/m3-admission-r1-main-verification.md` row C568 (Disks with io_min 1 MiB Cannot Create Volumes Under Strict Fixed Structures), owed check C568 (Disks with io_min 1 MiB Cannot Create Volumes Under Strict Fixed Structures).

**Basis**:

- E129 (How Sub-io_min Writes Fail on Real Devices): Real block devices with `io_min` far exceeding `physical_block_size` exist (`dm-thin` chunk 64 KiB, `io_min` 65,536, `physical_block_size` 512), where small writes induce device-side amplification ⇒ slot spacing must round to probed `io_min`; rounding solely to `physical_block_size` places two slots in same mapping unit.
- Three-way verification (system configuration slot width round) round 1 reverse leg constructed two slots sharing mapping unit under `io_min` = 3072 geometry, materials and output in `system_configuration-slot-width-reverse-opus.md` and `system_configuration-slot-width-reverse leg report in git`.
- User decision (pre-coding master audit plus closing modal decision next day), original wording in history.

**Owed check**: C82 (io_min Excluded from Mount Comparison List), C212 (Fixed Structure Writes Smaller than io_min), C414 (Mount Fails to Compare Probed io_min with Recorded Slot Spacing).

#### Settled Item 20: Stripe Table and Container Index Do Not Register Day-1 in Tree Table

**Settlement**: Stripe table and container index **do not** register in tree table in version 1; allocate new tree ID and kind code when `w ≥ 3` opens. Settled item 12 clause "new reserved tree ID must be registered" reinterpreted as "acquire new ID upon opening track".

**Scope**: Settles registration timing, not carrier (settled item 12) nor key (settled item 14). Version 1 2-disk strictly `w = 2`; stripe table and container index write zero records on disk, registering or not producing zero bytes.

**Basis**:

- User decision (pre-coding master audit), original wording in history.
- No experiment: Both trees empty on disk in version 1, registering or not writes zero bytes, with no measurable quantity.

**Owed check**: C103 (Container Index Lacks Placement).

#### Settled Item 22: Full-Stripe Write, Never Read-Modify-Write

**Settlement**: **Stripe width is variable, every write is full-stripe write, never read-modify-write.** Write hole is not repaired, but designed not to exist. Cost: slight reduction in space efficiency, waste on small writes. Accepted.

**Scope**: Governs how write path issues stripes; width settled in settled item 6, participating disks in settled item 8, unit placement in settled item 10. Extends to device side via settled item 16 and settled item 17 (do not induce device internal read-modify-write). External fact (unverified in this project): ZFS RAID-Z validated viability of this path.

**Basis**:

- User decision (initial project establishment of D2 (RAID Stripe Policy)), original wording in history.
- No experiment: Designing write hole out of existence is format-layer mechanism deduction, with no measurable quantity in this project; ZFS RAID-Z is external fact, excluded from basis per `.claude/singlefs-ai-sop/rules/evidence-discipline.md`.

**Owed check**: None.

### Unsettled Items

| # | Item | Status |
|---|---|---|
| 21 | **Parity slot count** | How many parity slots in a stripe has no clauses; version 1 2-disk strictly `w = 2` ⇒ geometrically strictly 1 slot, but that is forced by geometry. Modifies new-pool file creation bytes: No (2026-09-20, basis: version 1 2-disk mirror implemented as `parity = 1`, no alternative value writes different bytes; what requires settling is slot count when `w ≥ 3` opens, where branches first appear) Modifies format: Yes (2026-09-21, basis: scope states verbatim "upper bound resets with parity"—`w_max` byte in system configuration varies with this item's answer; parity=2 yields 7 under equal weight arithmetic, while settled item 18 currently takes 4) **Status: Unsettled.** **Progress: Unimplemented.** |

#### Unsettled Item 21: Parity Slot Count

**Topic**: How many parity slots in a stripe (`parity`). Version 1 2-disk strictly `w = 2`, geometrically forced to 1 slot; how many slots when `w ≥ 3` opens has zero clauses repo-wide. **Path for version 1**: Unsettled, implemented as `parity = 1` (2-disk mirror, `w = 2`). **Modifies new-pool file creation bytes: No**—version 1 has 2 disks, `w` strictly 2; `w_max` / `g` bytes in system configuration uniquely determined by settled item 18; this item alters input when resetting those bytes in future, not modifying bytes written by new-pool file creation.

**Scope**: Revisit status of settled item 18 before settling: `w_max` = 4 derivation had dimensional error (corrected equidimensional arithmetic yields 6 under equal weight), equal weighting lacks source in kb, making 4 a conservative choice accepted with awareness, not pinned by that arithmetic; under parity=2, dual disk failure loses no data, risk axis shifting to triple failure, yielding 7 under same arithmetic ⇒ upper bound resets with parity. **Cascade with D9 (Encryption)**: Parity computed over ciphertext; multiple replicas write identical ciphertext N times—encrypting N times yields N distinct byte streams. **For reference (unadopted)**: bcachefs erasure-codes whole buckets in background: foreground writes normal multi-replica, background aggregates sufficient candidate buckets then recomputes parity once, atomically updating all pointers; parity computed once over immutable data, likewise eliminating write hole; cost is buckets cannot be reused as long as live data remains in stripe; if D8 (Core Index Structure) adopts bucket model in future, D2 (RAID Stripe Policy) should re-evaluate.

## Historical Versions

Historical entries for D2 (RAID Stripe Policy) are centralized in [decisions-history.md](../decisions-history.md).
