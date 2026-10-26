**出处 `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:1-6`（整段抄，未转述）**

```markdown
## D22 (Synthesizing Unit Atomicity) — Semi-settled (One Unsettled Item)

[中文](22-单元原子性怎么合成.md) · **English** · [日本語](22-ユニット原子性の合成手法.ja.md)

D22 (Synthesizing Unit Atomicity) governs how unit on-disk atomicity is synthesized: the demarcation line (whether a parent pointer carrying its checksum exists upon persistence before publication) divides units into two classes: "COW + D4 (Checksum Placement) suffices" versus "must self-authenticate"; the self-authenticating class (root, system configuration, journal) requires whole-unit checksums, actively verified generation numbers, and slot rotation; root ring parameters, field tables for root records and system configuration, placement of fixed structures, zone write pointer not serving as authoritative recovery ceiling and excluded from O2 (independent parser + checker), endianness and registry locations for on-disk fixed-length fields. What it does not govern: persistence order of publication and block reuse bounds are in D16 (Publish Semantics); journal record format, ring geometry, and replay prefix are in D23 (Journal Role & Format); unit header and unit class codes are in D18 (Information Carried in a Block); pointer fields and widths are in D19 (Block Pointer Structure and Width Budget); write granularity, fixed structure slot spacing, and criteria for "differing lifecycles" are in D2 (RAID Stripe Policy) Settled Items 15 / 16 / 17 / 19.

```

**出处 `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:7-511`（整段抄，未转述）**

```markdown
### Settled Items

**D22 (Synthesizing Unit Atomicity) items have a single numbering scheme**, continuously numbered with the "Unsettled Items" section, without reordering.

| # | Item | Ruling |
|---|---|---|
| 1 | **Whether journal is a circular ring** | Fixed-length ring, authoritative record in D23 (Journal Role & Format) Settled Item 2 **Status: Settled.** **Progress: Implemented.** |
| 2 | **Root ring parameters** | R = 3, regions placed with prime stride, slot sequence rotates across regions, S resides in system configuration, root slot width = probed `physical_block_size`, system configuration slot width = constant 4096, no reserved slots **Status: Settled.** **Progress: Implemented.** |
| 3 | **Whether accounting structures use COW** | Adopts COW, attached under parent pointer **Status: Settled.** **Progress: Implemented.** |
| 4 | **Whether I-7.4 ("blocks freed in this transaction not reused") exists** | Exists: blocks freed within entire journal replay window forbidden from reuse **Status: Settled.** **Progress: Implemented.** |
| 5 | **Expanding scope of D2 (RAID Stripe Policy)** | Completed: two write granularity hard requirements in D2 (RAID Stripe Policy) Settled Items 16 / 17 **Status: Settled.** **Progress: No code object.** |
| 7 | **Field table of root records** | Always routes via indirection layer: fixed header + one tree table unit pointer; tree roots never flattened into root slot; 17 fields total 466 bytes **Status: Settled.** **Progress: Unimplemented.** |
| 8 | **Placement of fixed structures** | System configuration one copy per disk with slot rotation and sequence generation written in two rounds; journal ring mirrored across two disks; mkfs zeros [0, root ring base) on each disk first and passes a barrier, then seeds Generation 0 root into all root ring regions and zeros regions completely **Status: Settled.** **Progress: Implemented.** |
| 9 | **Field table of system configuration** | Takes "system configuration field table" from `layout/01-first-txn.md`: 5 segments, 46 rows, 489 bytes, residing in 4096-byte system configuration slot **Status: Settled.** **Progress: Implemented.** |
| 26 | **Three-tier mutability classification for system configuration** | Divided into three tiers by cost of alteration: immutable system configuration 389 (rebuild file system), mutable system configuration 4 (rebuild index without losing data), runtime system configuration 36 (free, reconfigurable per mount and written back); system runtime observables 60 excluded from the three tiers **Status: Settled.** **Progress: Unimplemented.** |
| 10 | **Whether zone write pointer can serve as authoritative recovery ceiling** | Cannot **Status: Settled.** **Progress: No code object.** |
| 11 | **Whether zone write pointer enters O2 (independent parser + checker) judgment inputs** | Cannot: O2 (independent parser + checker) domain is a single image; zone write pointer does not reside in image **Status: Settled.** **Progress: No code object.** |
| 12 | **Whether system configuration requires its own tree table unit pointer** | Does not require; tree table authority resides in root record **Status: Settled.** **Progress: Implemented.** |
| 13 | **Endianness and enum encoding conventions for on-disk fixed-length fields** | Uniformly little-endian; enum values each have assigned registry location; bitmap bit n resides at bit (n mod 8) of byte (n div 8) **Status: Settled.** **Progress: Implemented.** |
| 14 | **Scope of I-7.6 judgment when devs < R** | Region device identity covers min(R, devs) distinct values, no disk holds more than ⌈R / devs⌉ regions; mount not refused, fault tolerance degrades with disk count **Status: Settled.** **Progress: Implemented.** |
| 15 | **System config quad-linkages: indirect dir pointer 59 vs 83, device table writing, devs, initial watermarks** | No indirect directory unit pointer, first version omits device table, devs occupies 4 bytes, compaction three watermarks constant 0 **Status: Settled.** **Progress: Implemented.** |
| 16 | **System config slot rotation rules; root ring formula address space and intra-region slots** | System config 2 slots per disk rotating by generation mod 2 (mkfs initializes both slots to 1); root ring base is 16 KiB slot index; intra-region slot = (txg div R) mod S; disk attribution 0 / 1 / 0 **Status: Settled.** **Progress: Implemented.** |
| 17 | **Completion of registry locations for Item 13** | Journal record types, metric tags, algorithm types, three flag classes and bitmap bit orders each have registry locations **Status: Settled.** **Progress: Implemented.** |
| 18 | **Demarcation line** | Units published only after parent pointer with checksum persisted on disk: COW + D4 (Checksum Placement) suffices; otherwise must self-authenticate **Status: Settled.** **Progress: No code object.** |
| 19 | **Ordinary units with parent pointers: COW + D4 (Checksum Placement) suffices without device atomicity assumptions** | Any torn byte subset mismatches checksum in parent **Status: Settled.** **Progress: No code object.** |
| 20 | **Larger tear granularity degrades issue into lost writes rather than worsening it** | All-stale torn writes caught by parent pointer checksum, missed by in-block self-contained checksum **Status: Settled.** **Progress: No code object.** |
| 21 | **In-place overwrite structures require both unit checksum and verified generation** | Checksum verifies integrity, generation verifies current generation; per-sector stamping rejected **Status: Settled.** **Progress: Implemented.** |
| 22 | **Demarcation line cannot move upward: root ring rollback candidates become invalid** | Units with parent pointers not converted to stable-address in-place overwrite with self-authentication **Status: Settled.** **Progress: No code object.** |
| 23 | **Journal cannot be covered by COW: holds for fixed rings, belongs to self-authenticating category** | Generation number must be in format and actively verified by replay path **Status: Settled.** **Progress: Implemented.** |
| 24 | **Root belongs to same category, and both layout classes require identical mechanisms** | Root self-authenticates: whole-unit checksum + generation + slot rotation; verify all candidates first, select newest by generation, break ties by earliest root ring read order **Status: Settled.** **Progress: Implemented.** |
| 25 | **Root ring depth is upper bound on block reuse latency, not identical values** | Root ring only guarantees "if current root corrupted, previous generation still readable"; block reuse bound computed at runtime, cannot be format constant **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Journal Takes Fixed-Length Ring

**Ruling**: Journal adopts a fixed-length ring, authoritative record in D23 (Journal Role & Format) Settled Item 2. Decisive rationale: chained log replaces Deadlock 3 criterion from "worst-case operation ≤ ring size / F" (mkfs computable) with "worst-case operation ≤ current free space", which fluctuates mid-operation, violating `.claude/rules/fs-design.md` Hard Requirement 5.

**Scope**: Ring does not expand with files: when single-transaction budget exceeded, solution is runtime forced transaction splitting (D23 (Journal Role & Format) Deadlock 3 fallback, I-8.1 (Ring geometry sufficiently large)), not allowing ring to grow with files. Resolution and rationale for rejecting chained log belong to Settled Item 23 and D23 (Journal Role & Format) Settled Item 2.

**Evidence**:

- No experiment: Settled Item 1 serves as pointer; trade-off between ring and chain and empirical evidence reside in D23 (Journal Role & Format) Settled Item 2.

**Owed checks**: None.

#### Settled Item 2: Root Ring Parameters

**Ruling**:

| Quantity | Ruling |
|---|---|
| **Region count R** | **3** (`R = F + 1`, F = 2) |
| **Region location** | Region r placed at `r × P × chunk`, **P prime and `P > devs`**; address space, base address, and version 1 values in Settled Item 16 |
| **Slot order** | **Rotates across regions** (region = `txg mod R`); intra-region slot = `(txg div R) mod S` (Settled Item 16) |
| **Slots per region S** | Resides in system configuration. **Lower bound k_tol + 2 = 4** (D16 (Publish Semantics) Settled Item 1: **slots per region S** must accommodate 4 states plus handling k_tol root slot write failures — measured directly on S, not R × S; term "ring depth" suffered issue in C131 (Dimensions of stack depth and protection window), avoided here); **upper bound computed at mount from blocks held by I-7.4 (Blocks freed in recent K generations not reused)**; values between 4..16 touch no boundaries. ⚠️ **Specific value is not format decision** — format commits to "S is system configuration field + interval verified at mount", value chosen by version 1 implementation according to D25 (Target Workload Priority) workload characteristics |
| **Root slot width (bytes per root record)** | **Equals probed `physical_block_size` at mount**, exact equality suffices. Exceeding it buys no additional safety, only wasting bytes. mkfs allocates per pool `max(physical_block_size)`, mount verifies per-device comparison, **refusing mount** if unmet — isomorphic to D21 (Boundary Between Authoritative and Derived States) "unit geometry evaluated once at mount" |
| **System configuration slot width** | **Format constant 4096** (registered name `SYSTEM_CONFIGURATION_SLOT_BYTES`), sharing precedent with D23 (Journal Role & Format) Settled Item 12 record size 4 KiB; resides in D15 (Format Freeze Policy) Layer 1, altering it requires incompat bit. D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) statement "slot width = physical_block_size" refers to tear determination resolution, not structure size |
| **Slot spacing** | Slot spacing for root slots and system configuration slots = 4096 rounded up to integer multiple of io_min probed at mkfs (D2 (RAID Stripe Policy) Settled Item 19); conversion from root slot index to offset in Settled Item 16 |
| **Alignment** | Starting offset of each slot is integer multiple of judgment width, settled as I-7.5 (Root slots aligned to judgment width) |
| **Reserved slots** | **None reserved**. ⚠️ **Reopening condition**: Reopened if multi-host protection introduced |

**Scope**:

- **Applicable condition: Disk holding the region is still present. Must cite this condition alongside conclusion.** Region locations are established facts on disk at mkfs, **as is on-disk attribution** — D2 (RAID Stripe Policy) Settled Item 7 verbatim: "Attribution no longer computed as `(r × P) mod devs`, device identities written down per-region at mkfs" ⇒ **Adding / replacing disks never alters attribution of existing regions.** ⇒ **R = 3 fault tolerance commitment still degrades, but mechanism is not 'regions collide', but 'disk holding region is gone'.** This slot is an instance of D2 (RAID Stripe Policy) Settled Item 3 (explicit geometry re-evaluation transaction), falling under D2 (RAID Stripe Policy) Settled Item 5 (root ring not relocated).
- **`P` retains sole purpose of "region placement"**: Constraint "P prime and P > devs" originally justified by "`gcd(P, devs) = 1` ⇒ multiplying P is bijection mod devs ⇒ regions land on distinct disks"; rationale invalidated with attribution formula; whether constraint itself retained unwritten across repository (C217 (Rationale for P greater than device count removed while constraint remains)).
- When root slot width takes `physical_block_size`, array `io_min` is far larger: after spacing takes integer multiples of io_min, single io_min unit no longer packs two root slots (D2 (RAID Stripe Policy) Settled Item 17 treats distinct slots in same region as differing-lifecycle objects under D2 (RAID Stripe Policy) Settled Item 15 criterion); each root slot write remains smaller than io_min, verbatim violating D2 (RAID Stripe Policy) Settled Item 16, accepted knowingly for version 1 (C212 (Fixed structure writes smaller than io_min)). Internal SSD RMW across entire mapping unit un-injected, only RAID5 write holes measured on real devices.
- System configuration slot width four candidates evaluated in E126 (Cost of Four Candidates for System Configuration Slot Width), whose Arm B measures probed max(pbs, io_min), distinct from ruling's Arm B (recorded slot spacing). System configuration slot width separated from root slot width: root slot width remains probed `physical_block_size`, version 1 local `nvme0n1` currently probes 512, D2 (RAID Stripe Policy) Settled Item 9 runs 2 disks in version 1 ⇒ homogeneous 512 pool slot is 512.
- Root ring degraded semantics written down, measured on real devices, addressing [pitfalls.md](../pitfalls.md) Item 9 (degraded mount writes single replica worsening corruption, root cause being lack of degraded behavior design).
- **Reader verifies root slot width and fixed structure slot spacing during system configuration selection**: Fixed structure slot spacing ∈ [4096, 1 MiB − 4096] (upper bound ensures Slot 1 entirely precedes root ring base, `root_ring::region_start(0)` minus system configuration slot width 4096); root slot width (`physical_block_size`) ∈ [root record width 466 bytes (Settled Item 7), slot spacing]; out-of-bounds refuses mount for whole pool, identical handling to slots per region S violation (audit A3a: `crates/singlefs-core/src/recovery.rs` `system_configuration_values_this_reader_accepts`, `SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds`); pool-level checker reports I-7.13 (System configuration pool-level fields within reader acceptance range) violation, all other invariants reported not applicable (audit A3-checker-2: `crates/singlefs-checker/src/image.rs` `judge_system_configuration_values_the_reader_accepts`).
- Root ring only guarantees "if current root corrupted, previous generation still readable", not "can rollback K generations" (Settled Item 25); zoned parameters inapplicable (Unsettled Item 6).

**Evidence**:

- F = 2: Product judgment, E46 (Root Ring Region Spacing and Failure Domains) / E47 (Root Ring Failure Domain Loss) / E48 (Feasible Points of Root Ring Geometry) / E50 (Upper and Lower Bounds on Root Ring Slots) all self-declare experiments cannot answer; user ruling, original wording in change history.
- E46 (Root Ring Region Spacing and Failure Domains): "Head and tail" placement on common stripe geometries lands on identical disk ⇒ regions must be chosen by failure domain, region count = tolerable failure domains + 1.
- E47 (Root Ring Failure Domain Loss): Head and tail placement fails to mount upon losing single disk across most device counts; contiguous slot order intra-region incurs 8× worse rollback than cross-region rotation ⇒ slot order takes cross-region rotation.
- E48 (Feasible Points of Root Ring Geometry): Grid search across placement rules × slot sequences reveals only "prime stride + cross-region rotation" combination is feasible.
- E50 (Upper and Lower Bounds on Root Ring Slots): Measured S lower bound and upper bound lie outside 4..16, S is not a quantity requiring fine-tuning.
- E53 (Root Ring Mountability After Losing Whole Disk): Physically wiped per-disk on VM real block devices, mounted successfully all 4 times, worst-case rollback 1 generation.
- E55 (Stripe Mapping Arithmetic on Real Devices): Prime stride on real `dm-stripe` landed across mutually distinct disks for all 6 groups.
- E41 (Root Ring Slot Geometry 2×4×256): Slot width smaller than judgment width corrupts multiple slots on single tear; half-slot offset doubles at-risk slots ⇒ slot width takes judgment width, starting offset aligned to it; candidate "2 regions × 4 slots × 256 bytes" replaced accordingly.
- E34 (Root Ring Slot Geometry): Prime stride ensures one io_min unit cannot cover two regions; wiping entire region incurs worst-case rollback 1 generation; real md/raid5 tear pollutes only its row, exposure surface ≠ loss surface.
- System configuration slot width: Settled via user final dialog after tripartite evaluation `research/prompts/d22-slotwidth-r1-main-verification.md` (forward leg judged recorded slot spacing, reverse leg judged constant 4096, local leg unstable across two samplings), original wording in change history.
- Reserved slots: Sole active reference has valid slots unequal to physical slots — ZFS `ub_txg % (VDEV_UBERBLOCK_COUNT − m)`, where `m` occupied by multi-host protection (MMP); known difference: SingleFS has no multi-host semantics (external fact, unverified in this project).

**Owed checks**: C212 (Fixed structure writes smaller than io_min); C217 (Rationale for P greater than device count removed while constraint remains).

#### Settled Item 3: Accounting Structures Adopt COW

**Ruling**: Accounting structures adopt COW, positioned under parent pointers. Path "run with in-place first, switch to COW later" does not exist: paths diverge at mkfs.

**Scope**: Only governs update morphology of accounting structures. In-place wins on write amplification axis (E38 (Accounting Structures: COW vs In-Place), re-evaluated if D25 (Target Workload Priority) granularity alters), direction determined by factors other than write volume; time required for full scan unmeasured, pool size is configuration parameter.

**Evidence**:

- Settled Item 22: Accounting is authoritative state (D21 (Boundary Between Authoritative and Derived States)), residing under root; in-place overwrite similarly renders rollback candidates of old roots invalid — holds independent of mkfs reservation modeling.
- E38 (Accounting Structures: COW vs In-Place): COW arm mkfs reservation identically 0, tears cannot destroy sole copy; in-place arm requires reserving fixed locations per accounting tree geometry.
- User ruling (unit self-containment ⇒ parent pointer path requires no extra positioning conventions, D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment), D18 (Information Carried in a Block) Settled Item 3), original wording in change history.

**Owed checks**: C95 (Unchecked settled items with retracted evidence): Recovery cost column in E38 (Accounting Structures: COW vs In-Place) retracted; in-place arm modeled with static full reservation judged strawman; Settled Item 3 supported by Settled Item 22.

#### Settled Item 4: Rule "Blocks Freed in This Transaction Not Reused" Exists

**Ruling**: Rule "blocks freed in this transaction not reused" exists. Corresponds to D16 (Publish Semantics) Settled Item 15: "All releases generated within checkpoint C forbidden from entering allocatable set prior to C publication". ⚠️ **Granularity stricter than checkpoint**: Second rationale in D16 (Publish Semantics) rule stated in terms of **journal position** — block freed at position p, reallocated and written at q > p; replaying prefix `p < r < q` yields tree pointing to address whose on-disk bytes already contain q's content. ⇒ True constraint is **entire journal replay window**, not checkpoint boundary. Failure mode in its absence: **detectable (parent pointer checksum mismatch), unrecoverable.**

**Scope**: btrfs achieves same behavior via pin (external fact, unverified in this project). Pinning targets different blocks than root ring pinning rule, division of labor detailed in Settled Item 25.

**Evidence**:

- No experiment: Logical deduction from D16 (Publish Semantics) Settled Item 15 via journal positions, no measurable quantities.

**Owed checks**: None.

#### Settled Item 5: Scope of D2 (RAID Stripe Policy) Expanded, Completed

**Ruling**: D2 (RAID Stripe Policy) Settled Items 16 and 17 both incorporated: issuing no writes smaller than device physical mapping unit (in Linux represented by `io_min`, not `physical_block_size`), namely D2 (RAID Stripe Policy) Settled Item 16; forbidding two objects of differing lifecycles from sharing identical physical mapping unit, namely D2 (RAID Stripe Policy) Settled Item 17.

**Scope**: Settled Item 5 records completion of registration; scope and criteria for both hard requirements belong to D2 (RAID Stripe Policy): "differing lifecycles" evaluated per D2 (RAID Stripe Policy) Settled Item 15, fixed structure slot spacing per D2 (RAID Stripe Policy) Settled Item 19. Fixed structure writes smaller than io_min conflict with D2 (RAID Stripe Policy) Settled Item 16, knowingly accepted for version 1 (C212 (Fixed structure writes smaller than io_min)).

**Evidence**:

- No experiment: Registration task incorporating two hard requirements into D2 (RAID Stripe Policy), no measurable quantities.

**Owed checks**: C212 (Fixed structure writes smaller than io_min).

#### Settled Item 7: Field Table of Root Records

**Ruling**: **Root record always routes via indirection layer: fixed header + one pointer to "tree table unit", tree roots uniformly reside in tree table unit, never flattened into root slot.** **No branching based on "flattened when few trees, indirect when many trees"**: Branching variable is tree count, which changes during mount (creating writable head adds tree, D6 (Snapshot Implementation Model) Settled Item 1) — violating `.claude/rules/fs-design.md` Hard Requirement 5: "branch variables forbidden from changing mid-operation". Uniform single form.

| Field | Width | Source |
|---|---|---|
| magic | 4 | — |
| fsid | 16 | Identical measurement basis as I-1.4 (Block header fsid consistency) |
| flags | 4 | Bit 0 = unmount flag: set to 1 in root written during clean unmount sequence (D16 (Publish Semantics) Settled Item 1 "clean unmount"), 0 in other roots; pool-level checker branches I-7.9 (Rollback floor F no higher than raise F ceiling) based on it; remaining bits constant 0 in version 1, non-zero rejected (D22 (Synthesizing Unit Atomicity) Settled Item 17). Bit n resides at bit (n mod 8) of byte (n div 8) (Settled Item 13) |
| **Instance generation** | **4** | D16 (Publish Semantics) Settled Item 6 Rider 1: **identical counter** as journal instance generation in D23 (Journal Role & Format) Settled Item 9; tie-breaker during checkpoint_txg ties favors higher value (when reading the root ring each root's slot is checked against the placement formula (Settled Item 16), with generation-0 roots recognized separately by the slot mkfs seeds them in; one txg has only one placement slot, so ties remain only among the byte-identical generation-0 roots, and the first in root ring read order is taken (Settled Item 24); the formula check is not yet implemented, owed in [checks-owed.md](../checks-owed.md) C743 (rebuilt allocator recording new class does not verify root occupies slot)) |
| **checkpoint_txg** | 8 | D16 (Publish Semantics) Settled Item 6: publish count, rotation key, accounting generation, and replay watermark are identical number — **no separate "publish generation" field** |
| Tree table unit pointer | **86** | Pointer header 50 + position entry **14** × 2 replicas + instance generation 4 + birth sequence 4 (D19 (Block Pointer Structure and Width Budget) Settled Items 7 / 8; position entry = device ID 4 + 16 KiB slot index 6 + ciphertext checksum 4, D19 (Block Pointer Structure and Width Budget) Settled Item 4); bootstrap exemption, position entry is authoritative |
| **Tree ID watermark** | **8** | D8 (Core Index Structure) Settled Item 8 ②: `max(that field across all root records in root ring, highest tree ID issued in this checkpoint + 1)`, rewritten with root record upon each publish |
| **Rollback floor F** | **8** | D16 (Publish Semantics) Settled Item 1: reallocatable boundary max(F_effective, oldest valid root in ring); stationary normally, raised when acceptance insufficient, raised to current txg upon clean unmount; system configuration records separate copy (Settled Item 9), effective value post-recovery takes maximum readable across both locations (D16 (Publish Semantics) Settled Item 1 "effective") |
| Self-authenticating checksum | 32 | Identical width as E23 (Journal Geometry) `header_csum` |
| **Instance table unit pointer** | **86** | C113 (Current version determination for multi-version units during scan rebuild lacks input) Ruling P2: Instance table unit (D18 (Information Carried in a Block) Settled Item 11 packed record type 4) excluded from container index, directly held by root record; identical structure to tree table unit pointer |
| **Central mapping tree root pointer** | **86** | D19 (Block Pointer Structure and Width Budget) Settled Item 11: Central mapping tree root resides in root record, excluded from tree table, isomorphic to tree table unit pointer (bootstrap exemption); mkfs and warm-up write all zeros |
| **Allocation record tree root pointer** | **86** | D16 (Publish Semantics) Settled Item 9: Points to allocation record nodes written during publish that wrote rows on 0-entry tree table version; writes non-zero only on 0-entry tree table version with row writes, mkfs Generation 0 and versions with files write all zeros; isomorphic to tree table unit pointer |
| **Reclamation threshold t** | **8** | D16 (Publish Semantics) Settled Item 1, "Pins are also persisted" in the "Root slot read error this time" row: the reclamation threshold at the time this root is written, taken as max(the F this root carries, the reclamation threshold when the allocator recorded this root), so that the threshold pinned by a root slot that read bad or read back all zeros is persisted with the root instead of living only in this process's memory; a zero-unit publish copies the previous version's value, and mkfs generation 0 writes 0; judged by pool-level checker I-7.18 (the root record's reclamation threshold equals the currently computed threshold) |
| **Pinned slot p** | **1** | D16 (Publish Semantics) Settled Item 1, "Pins are also persisted" in the "Root slot read error this time" row: among the root ring slots still pinned when this version's accounting is built, the one with the smallest pin value, where pin value = 3·slot + region, i.e. the oldest txg that slot could have held (less than the ring length 3S); writes 255 when no slot is pinned (`ROOT_RECORD_NO_RING_SLOT_PINNED`); a pinned slot must be released when this process writes over it, and if that slot already holds a root written by this instance while p still points at it, pool-level checker I-7.19 (a pinned slot does not hold a root written by this instance) judges red |
| **Algorithm type** | **1** | D9 (Encryption) Settled Item 12 Item 1 (scope expanded to self-authenticating units): 0 = unencrypted; registry in D9 (Encryption), D22 (Synthesizing Unit Atomicity) Settled Item 17 |
| **nonce** | **12** | D9 (Encryption) Settled Item 12 Item 1; identical measurement basis as 12-byte unit header (D18 (Information Carried in a Block) Settled Item 14) |
| **MAC** | **16** | D9 (Encryption) Settled Item 8 identical measurement basis; root record is self-authenticating structure, covered by own MAC when encryption enabled |

Row order reflects on-disk layout, offsets accumulated sequentially: magic 0 / fsid 4 / flags 20 / instance generation 24 / checkpoint_txg 28 / tree table unit pointer 36 / tree ID watermark 122 / rollback floor F 130 / self-authenticating checksum 138 / instance table unit pointer 170 / central mapping tree root pointer 256 / allocation record tree root pointer 342 / reclamation threshold t 428 / pinned slot p 436 / algorithm type 437 / nonce 438 / MAC 450.

<!-- format-const: ROOT_RECORD_BYTES = 466 stale=根记录 371 字节|根记录合计 371|14 个字段合计 371 字节|合计 **371 字节**，512 槽内余量 141|3 × 371|合计 **457 字节**|根记录宽 457|15 个字段合计 457|457 / 512 槽|3 × 457|ROOT_RECORD_BYTES=457|ROOT_RECORD_BYTES = 457|`ROOT_RECORD_BYTES` 是 457|根记录（457 字节|根槽偏移 428..457|15 个字段合计|512 槽内余量 55|根记录合计 457|根记录 **457 字节**|根记录 [0, 457)|[457, 槽宽) -->
Total **466 bytes**, 46 bytes margin within 512 slot — slot width probed at runtime, margin reserved for future fields.

**Tree table unit**: 16 KiB metadata unit, storing **81 trees/layer**, nesting second layer beyond that (81² = 6561 trees); entry width takes **200 bytes** settled in D8 (Core Index Structure) Settled Item 8 (tree ID 8 + entry length 2 + tree kind 2 + flags 2 + root pointer 86 + `previous_snapshot_txg` 8 + birth txg 8 + head ID 8 + reserved 76), header width takes tree table unit's own Code 2 header **131** (key = tree ID, key width 8 ⇒ 115 + 16, D18 (Information Carried in a Block) Settled Items 16 / 18): ⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81**. ⚠️ **Denominator has sole reading**: 16384 minus Code 2 header with reserved space, subtracting nothing else (C157 (Tree table capacity computed with differing entry widths in two places) demanded this unified measurement basis; D8 (Core Index Structure) Settled Item 8 identical verbatim). Tree table unit header fields follow index node identity segment in D18 (Information Carried in a Block) Settled Item 7; constitutes part of authoritative state traversal (I-7.2 (Latest root fully traversable) "traverse all authoritative state" traverses via it).

**Scope**: Only governs root record and referenced tree table unit; system configuration carries no tree table unit pointer (Settled Item 12). ⚠️ **Replay watermark reading** (D23 (Journal Role & Format) Settled Item 14): Recovery only applies records where `(instance generation, checkpoint_txg)` strictly exceeds root; tail serves only as scan start optimization. Cost of indirection layer is one extra tree table unit COW per publish, knowingly accepted; nesting second layer when trees exceed one layer adds extra COWs per publish unmeasured in absolute values (C148 (Wider tree table entries compress indirection threshold to two layers)).

**Evidence**:

- E79 (Root Record Capacity): Root slot width = `physical_block_size` × tree table day-1 open (D15 (Format Freeze Policy) Settled Item 2) × one tree per head (D6 (Snapshot Implementation Model) Settled Item 1) combined: flat layout cannot accommodate settled components of this design ⇒ indirection layer structurally unavoidable.
- Trailing 29 bytes (algorithm type + nonce + MAC): Root record and journal record headers are self-authenticating structures, lacking parent pointers carrying MAC and nonce ⇒ omitting day-1 reservations forces either plaintext exemptions (leaking `checkpoint_txg`, tree table pointer, mapping root pointer, tree ID watermark) or layout changes upon enabling encryption; D9 (Encryption) Settled Item 3 principle requires reservation bits across all volumes, non-repurposable.
- User ruling (indirection layer and field table, tree ID watermark, instance table unit pointer, rollback floor F, central mapping tree root pointer, three 86-byte pointers, trailing 29 bytes), original wording in change history; widening three pointers and trailing 29 bytes bypassed tripartite evaluation, refutable.
- User ruling 2026-09-23: Added 86-byte allocation record tree root pointer (C512 (Where replaced instance table is recorded on 0-entry tree table version lacks clause)), total 371 → 457, original wording in change history.
- User ruling 2026-10-20: the root record gains the 8-byte reclamation threshold t and the 1-byte pinned slot p (C773 (reclamation threshold pins for bad and all-zero root slots only in memory); D16 (Publish Semantics) Settled Item 1, "Pins are also persisted" in the "Root slot read error this time" row), total 457 → 466, original wording in change history. Placing t at offset 428 and p at 436 and moving algorithm type, nonce and MAC back was decided by the main agent, following Y4.2 of the root ring round 2 forward leg (`research/prompts/m3-close-rootring-r2-sonnet-output.md` section "Y4.2 fields"); not attacked, zero rounds; no experiment: what was added is the width and offset of two fields, no experiment ID established.
- User ruling 2026-09-26: flags bit 0 assigned as unmount flag (I-7.9 (Rollback floor F no higher than raise F ceiling) adopts (b)+(c) combined), original wording in change history; no experiment: tripartite prototype `research/prompts/m2-rollback-forward-r3-opus-model/` measured discriminatory power of three revisions (Round 3 verdict K6: (a) loses "acceptance over-raised" category entirely; (b) loses "acceptance over-raised carrying unmount flag" cell; (c) pool-level image unjudged), no experiment ID established; combination proposed by main agent, survived zero attack rounds.
- Tie in the instance generation row: root ring tripartite round three verdict `research/prompts/m3-close-rootring-r3-main-verification.md` Section 2 cell C743 (rebuilt allocator recording new class does not verify root occupies slot) (two roots with the same txg coexisting conflicts with the placement formula of Settled Item 16; what changed is this row's tie parenthesis; the generation-0 exception was added after round one hit the bare formula, round two attacked it without a hit, and the defense held in round three).

**Owed checks**: C88 (Timeline discrimination for root ring unimplemented); C148 (Wider tree table entries compress indirection threshold to two layers); C308 (Field offsets for self-authenticating units unsettled); C743 (rebuilt allocator recording new class does not verify root occupies slot): the placement formula check when reading the root ring. (C78 (Root record lacks field table) settled 2026-10-11, merged into C66 (Format-level blanks lack item numbers); C157 (Tree table capacity computed with differing entry widths in two places) settled 2026-10-11, merged into C148 (Wider tree table entries compress indirection threshold to two layers), see checks-owed settled table).

#### Settled Item 8: Placement of Fixed Structures

**Ruling**: Three rules, tailored for 2-disk version 1 (D2 (RAID Stripe Policy) Settled Item 9):

1. **System configuration stored one copy per disk.** Updated per Settled Item 21 (in-place structures require whole-unit checksum and actively verified generation number), implemented as slot rotation, at least 2 slots per disk; slot rules in Settled Item 16, fields in Settled Item 9.
2. **Journal ring mirrored across two disks.** Cost is writing every record twice in steady state. **Not an added overhead** — D2 (RAID Stripe Policy) Settled Item 6 already requires all writes w ≥ 2; this ruling merely clarifies "all writes" covers journal records.
3. **mkfs must seed Generation 0 root into all root ring regions**; before writing any metadata, zero [0, root ring base) entirely on each disk (version 1 [0, 1 MiB), covering slot 0 and all 2,033 candidate positions for slot 1; remaking mkfs on used disks leaves no stale pool slots), zero all three root ring regions entirely (including 3 × (S−1) unseeded slots), and zero journal ring entirely: all issued by mkfs via block device zeroing, called once per disk for [0, root ring base), root ring three regions, and journal ring (region attribution defines which disk root is seeded, not which disk is cleared), each whole-region clear recorded as single step in recording stream (D17 (Implementation Layering and Third-Party Pipelines) Settled Item 2 Rule 1 Type 1), 5 steps per disk, 10 steps across two disks, segment sequence and steps defined in [layout/01-first-txn.md](../layout/01-first-txn.md) VIII "mkfs Root Seeding"; after the [0, root ring base) of every disk is cleared, both disks each pass one barrier (invalidation) before anything else is written; no barriers between the other clears and unit writes. fsid is generated by mkfs's external entry point with the first 8 bytes taken from a random source; the inner entry point may supply fsid, for tests, harnesses and crash-point replay. Once seeded, state "freshly mkfs-ed, zero publications" mirrors steady state, root rolling back at most region depth generations.

**Scope**: System configuration update protocol (generation number, checksum width in slot rotation) follows unit header field table (D18 (Information Carried in a Block) Settled Item 7) measurement basis, residing in Settled Items 9 and 16; whether fixed structures can rotate in-place on zoned media belongs to D22 (Synthesizing Unit Atomicity) Unsettled Item 6, unanswered. Fault tolerance commitment covers single failures: one corrupted system configuration slot per disk covered, and the newest slot on both disks staying bad after sequence generation is also covered (user settled 2026-10-19, original wording in change history): sequence generation is written in two rounds, each round one write per disk followed by one pool barrier, both slots on every disk carry the new number from sequence generation, and when all-or-nothing judges failure each disk is rolled back only once (`write_the_witnessed_acquisition`, `roll_back_acquisition` in `crates/singlefs-core/src/transaction.rs`); dual-failure pattern where each disk loses one system configuration slot while latest root is temporarily unreadable excluded from commitment (user settled 2026-09-28 not covered); before raising F, pool-level checker shows zero red invariants under that pattern (shielded by writable mount pre-quarantine, commitment still excludes it), registered as two test cases in [checks-owed.md](../checks-owed.md) settled table row C583 (Dual-failure pattern: each disk loses one system config slot plus latest root temporarily unreadable). Writable mount system configuration witness read operates per-disk (S-distinguish: one slot torn, other self-authenticated, distinct from device unreadability); unresolvable ⟺ slot device reports read error, and no disk has both slots self-authenticated (R2, user settled, survived zero attack rounds, original wording in change history). Unresolvable layouts have one offline way out (C596 (system configuration layout with unjudgeable witness left only with read-only mount)): the administrator asserts a version; only if that version's root or the last record of that publish is readable and is it, and the maximum `birth_txg` of this pool's units in each disk's unit area is not greater than it, are the non-self-authenticating system configuration slots rewritten, with the instance generation written taken as the maximum over each disk's self-authenticated slots, root ring, journal ring and unit write order, without adding one; otherwise not a byte is written (user settled 2026-10-19, `birth_txg` gate added 2026-10-20, original wording in change history; `rewrite_system_configuration_slots_for_the_asserted_version` in `crates/singlefs-core/src/system_configuration_repair.rs`). When both slots of a disk are void (not even one self-authenticated system configuration of this pool), the sequence generation step rewrites its two slots, requiring all of the following and refusing if any fails: the disk holds all units of the selected version (list non-empty, each placement read once, bytes identical to the verified copy); no root ring or journal ring slot of the disk has a device read error; these slots hold no root or record newer than the selected version (by root selection order); the disk's unit area holds no unit whose write-order instance is greater than the selected version's instance; the root slot half of the selected version is not missing (same judgment as the missing-root half of the lagging branch). A failed rewrite rolls back once per disk as in sequence generation (user settled 2026-10-03 that this step refuses, opened the exception 2026-10-19, added the unit area scan and ring slot read errors 2026-10-20, original wording in change history; `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration` in `crates/singlefs-core/src/mount.rs`; the implementation only lets disks whose two slots are both readable and both fail self-authentication through the gate, and still refuses disks with a device read error in a slot). Lost write bounds categorized by class: pure pattern reading single value pool-wide incurs 120 lost write cells; mixed pattern where one disk reports read error and other reads all zeros incurs 0 lost write cells (cells on C331 (Root selection inversion overrides confirmed write) grid, measured on replica), lost write cells falling entirely within dual-failure pattern. When the two disks are the same pool, a crash before the invalidation is durable: the old pool reads back the last confirmed version; the outcome of writable mount follows the current rewrite clause. A crash after the invalidation is durable and before the last barrier: recovery has only two outcomes—whole-pool refusal (`NoValidSystemConfiguration`), or reading back mkfs's generation-0 empty pool (root at instance 0, txg 0, no files); in the latter, system configuration redundancy may be missing, and writable mount of the disk lacking the new slot is handled by the current rewrite clause. A refusal because parameters the caller asserts itself (for example the old pool's fsid) disagree with the system configuration selected on disk (`CallerParametersDisagreeWithTheSelectedSystemConfiguration`) is the caller's assertion disagreeing with the disk, not an outcome of the on-disk state. When devices honor barriers, after the invalidation is durable no outcome may read back any version of the old pool, nor may a new system configuration coexist with a root of the old pool; barriers reaching only one disk belong to C6 (Wrong Block Layer Semantic Assumptions Written), and starting points where the two disks come from two pools are outside scope.

**Evidence**:

- Dual-failure pattern uncovered: User settled 2026-09-28, original wording in change history; two test cases in [checks-owed.md](../checks-owed.md) settled table row C583 (Dual-failure pattern: each disk loses one system config slot plus latest root temporarily unreadable).
- E87 (Placement of Fixed Structures): Exhaustive evaluation of 8 failure combinations on 2 disks — single copy of system configuration loses mountability pool-wide upon losing that disk (data, root, journal intact is useless); one copy per disk + mirrored journal achieves 8 mountable cells, zero replay window loss; omitting Generation 0 seeds leaves zero rollback roots upon losing any disk post-mkfs before publishing.
- Whole-segment zeroing: Without it, remaking mkfs on disk previously holding singlefs allows stale root records in slots 1..S−1 to pass self-authenticating checksums with larger `checkpoint_txg` ⇒ first mount selects root from previous file system while I-7.1 (Root slot valid set non-empty) and I-7.3 (Ring health) stay green (deduced, unmeasured).
- Zeroing [0, root ring base): Tripartite verdict `research/prompts/m3-c651-r1-main-verification.md` Section 3 "Handling old pool system configuration slots left on reused disks" and user ruling (post-zeroing, remaking mkfs with differing slot spacing on used disks followed by tearing slot 0 on both disks no longer selects old pool residue in slot 1: measured by attacker on replica harness, confirmed by main agent re-run, enforced in repository by `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader`); `research/prompts/m3-formal-r6-main-verification.md` Section 2 mid-mkfs crash cell (zeroing upheld, resolving post-first-barrier cell). Implemented in `crates/singlefs-core/src/make_filesystem.rs` first statement of per-disk zeroing loop, test case `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader`.
- User ruling, original wording in change history.
- System config witness read per-disk, unresolvable adopts R2: Tripartite verdict `research/prompts/m3-ring-r2-main-verification.md` Section 2 Line 9 (per-disk read upheld) and `research/prompts/m3-ring-r3-main-verification.md` Section 2 Lines 7, 8 (per-disk read + pooled mixed pattern incurs 240 lost writes, bounds categorized by class; R2 zero rounds), Section 4 end "R2 Verification" paragraph (R2 measured on C331 (Root selection inversion overrides confirmed write) grid: pure pattern 120, mixed pattern 0, mixed pattern rejects 116 extra cells; 8 grid runs verified by transaction writer on clean replica, byte-for-byte identical with defense logs, grid omitted from repository). User ruling: adopted R2 (original wording in change history); refusing sequence generation when any disk lacks self-authenticated system configuration settled 2026-10-03, rewrite exception opened 2026-10-19 (original wording in change history). No experiment: numbers reside on tripartite replica and transaction writer replica, no experiment ID established.
- mkfs two-pass write order and mid-crash clauses: tripartite verdicts `research/prompts/m3-close-c656-r1-main-verification.md` Section 2 V2 (Option B upheld, conditional on the two-pass write order), `research/prompts/m3-close-c656-r2-main-verification.md` Section 2 U2, `research/prompts/m3-close-c656-r3-main-verification.md` Section 2 S2, S3 (synthesized form and two-pass write order upheld, the two mount clauses deleted, clause text taken from the "end of part three" draft in defense report `research/prompts/m3-close-c656-r3-sonnet-output.md`, zero-round wording); user ruling 2026-10-20 "accept, write back per the rewrite draft", original wording in change history. No experiment: numbers measured on tripartite replicas (segment sequence `2+12+1+1+1+4`, 27 steps, 31 195 states with tearing), no experiment ID established.
- fsid generated by the external entry point with the first 8 bytes from a random source: the same round three verdict Section 2 S1 generation cell (boundary at bytes 8 / 9, measured by both legs) and Section 3; user ruling 2026-10-20 "generate externally, keep the inner entry point", original wording in change history.
- Sequence generation written in two rounds, each disk rolled back only once: tripartite verdict `research/prompts/m3-close-sysconf-r3-main-verification.md` Section 2 U1 (attacker missed the two-round sequence generation write in rounds two and three with 0 number collisions; once-per-disk rollback attacked for one round without a hit; measured on replicas); user ruling 2026-10-19 "cover it, write sequence generation in two rounds", maintained 2026-10-20, original wording in change history.
- Rewrite exception in the sequence generation step: tripartite verdicts `research/prompts/m3-close-sysconf-r2-main-verification.md` Section 4, `research/prompts/m3-close-sysconf-r3-main-verification.md` Section 2 U4 and Section 3 (the form where a cloned disk wrote only units was hit, derived; the two added fixes, unit area scan and ring slot read errors, attacked zero rounds); user ruling 2026-10-19 "open the exception, four-part gate", two fixes adopted 2026-10-20, original wording in change history.
- Offline way out for unresolvable layouts: the same round three verdict Section 2 U2 (without the `birth_txg` gate, asserting the previous version loses confirmed writes, measured by attacker) and Section 3; user ruling 2026-10-19 "allow, additionally scan unit write order", 2026-10-20 "add birth_txg gate to the offline way out" (zero rounds), original wording in change history.

**Owed checks**: C79 (Placement of system configuration and journal ring unsettled): Journal ring mirrored across two disks implemented as forceable branch (one copy per disk and whole-disk drop already forceable, test cases in C79 (Placement of system configuration and journal ring unsettled) row).

#### Settled Item 9: Field Table of System Configuration

**Ruling**: Field table = "system configuration field table" from [layout/01-first-txn.md](../layout/01-first-txn.md) Section "I. Planted by mkfs", 46 rows, total **489 bytes**, residing in 4096-byte system configuration slot (Settled Item 2), margin 3607 bytes. Fields re-segmented by **mutability**:

| New Segment Name | Sources | Bytes |
|---|---|---|
| **Immutable system configuration** | Bootstrap header: magic 4, format version 2, feature bits 96, fsid 16, writer identity 20, checksum algorithm ID 1, local device ID 4, devs 4; encryption reservations 114; geometry excluding node size 128 | 147 + 114 + 128 = 389 |
| **Mutable system configuration** | Geometry: node size 4 | 4 |
| **Runtime system configuration** | Tunables: T_time 4, T_dirty 8, compaction low/high/stop watermarks 8×3 | 36 |
| **System runtime observables** (not config) | Slot generation 8, whole-slot checksum 32, journal tail 8, instance generation 4, rollback floor F 8 | 60 |

- Content permitted to exceed 512 (probed `physical_block_size`). Exceeding it allows writes to land partially, detected by whole-slot checksum (covering 4096 bytes including padding), slot selection rules (valid checksum and highest generation, Settled Item 16) automatically rolling back to alternate slot. **Detectable, recoverable.**
- Cost knowingly accepted: Crash-point replay must enumerate one additional tear dimension; torn mount retrieves prior generation configuration.
- Hard ceiling remains slot width 4096 (`SYSTEM_CONFIGURATION_SLOT_BYTES`, format constant, D15 (Format Freeze Policy) Layer 1). Exceeding it requires incompat bit.
- ⚠️ Current value 489 does not exceed 512; this rule represents **permission**, not **current state**. When fields added exceed it, tear dimension and assertions paid concurrently.

- Encryption reservations: KDF ID takes 4 bytes (not 2), master key slot inlined into system configuration slot (takes 80-byte minimal usable size, avoiding indirect directory units), encryption type retains 1-byte declaration.
- Version 1 omits indirect directory unit pointer and device descriptor table, `devs` occupies 4 bytes, compaction three watermarks constant 0 (Settled Item 15).
- Bootstrap header contains writer identity 20 (implementation ID 16-byte ASCII zero-padded + version 4-byte u32) and checksum algorithm ID 1 (0 invalid, 1 CRC-32C); geometry segment contains mkfs `physical_block_size` 4 and extension point declared value N 4 (widths in bytes). Readers encountering a system configuration format version other than 1 refuse to mount the whole pool, not degrading to this slot being unselectable, consistent with the C663 (two readers classify system configuration slot out-of-bounds and unaccepted values differently) sentence in Settled Item 16 sentence 1 (core: `system_configuration_values_this_reader_accepts` in `crates/singlefs-core/src/recovery.rs` reports `FormatVersionNotRecognized`, and `mountable_slot_or_refusal` returns `RecoveryFailure::SystemConfigurationValueRefused`); encountering a unit header format version other than 1, they reject that unit. Neither continues parsing by the current field table (the system configuration half user settled 2026-10-21 "change the clause to whole-pool refusal", original wording in change history; the unit header half main agent settled 2026-09-27; pool-level checker enforcement: unit header format version under I-2.4 (Header checksum coverage), system configuration format version under I-7.13 (System configuration pool-level fields within reader acceptance range), audit A3-checker-2).

**Scope**: Changes new-pool file creation bytes: **Yes** — every system configuration byte written by mkfs, sequence generation, warm-up, and publish determined by this table. Field encodings (endianness, enum values, bitmap bit orders) determined by Settled Items 13 / 17, feature bit allocation by D15 (Format Freeze Policy) Settled Item 4; slot rotation rules in Settled Item 16. Master key slot 80-byte internal composition (salt, wrap format) unsettled, determined when D9 (Encryption) enables encryption. Argument in blocker ①/④ second clause (pool-level scalar vs per-device, incompat bit escape) does not block bytes (C181 (Pool heterogeneity criteria missed stripe bitwise XOR axis), C183 (One incompat bit per layout while disks show zero byte differences)). E115 (Completeness and Byte Budget of System Configuration Field Table) and E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) four-cell figures counted tree table unit pointer and separate mapping root pointer into slot, superseded after E147 (System Configuration Field Table Recomputed per New-Pool File Creation Byte Table). Content is **pool-level**, duplicated per disk, both slots sharing content; sole disk-level field is "local device ID 4", not per-disk distinct values. Accommodates 60 bytes of **system runtime observables** (slot generation 8 + whole-slot checksum 32 + journal tail 8 + instance generation 4 + rollback floor F 8), which are not configuration: values alter with publish or raising F, never user-configurable; whether 12 bytes of journal tail and instance generation belong here assigned to C245 (Write frequency of system configuration slot, contradictory clauses).

**Evidence**:

- E147 (System Configuration Field Table Recomputed per New-Pool File Creation Byte Table): Recomputed row-by-row against byte table; encryption reservations taking widest variants across three cells fit within single sector — choices are encoding trade-offs, not budget trade-offs.
- Writer identity, checksum algorithm ID, mkfs `physical_block_size`, extension point declared value N required by settled clauses or invariants: I-1.5 (System configuration records pipeline identity) and D17 (Implementation Layering and Third-Party Pipelines) "debts to repay" Item 3; I-2.2 (Checksum algorithm consistent across pool) and C309 (Algorithm for 32-byte checksum unsettled); D13 (Verification Route) Settled Item 5 and D12 (Target Media) Settled Item 4 `physical_block_size` comparison; D21 (Boundary Between Authoritative and Derived States) Settled Item 8. First two items have "un-addable post-hoc" property: writer identity is fact at mkfs, checksum algorithm ID distinguishes legacy images upon introducing second algorithm.
- User ruling (encryption reservations, four fields), original wording in change history; four fields unvetted by tripartite evaluation, refutable.
- User ruling 2026-09-26: System configuration added rollback floor F 8 bytes (C419 (How F remains non-receding once effective) adopts SysPre, D16 (Publish Semantics) Settled Item 1 "effective"), deleted rollback witness table (D23 (Journal Role & Format) Settled Item 14), original wording in change history; no experiment: tripartite prototype `research/prompts/m2-rollback-forward-r3-opus-model/` measured SysPre on frozen replica (Round 3 verdict K4), no experiment ID established; F placed at field table tail starting offset 481 as specification appendix to system runtime observables, survived zero attack rounds.
- User ruling 2026-10-21: system configuration format version other than 1 refuses the whole pool (option "change the clause to whole-pool refusal"), original wording in change history; the cause was that the clause originally said to reject that slot, which did not match the implementation in `crates/singlefs-core/src/recovery.rs` and Settled Item 16 sentence 1 (`records/2026-10-18-里程碑三收尾与近期欠账.md` line 560, the milestone three wrap-up record).

**Owed checks**: C178 (Mandatory system configuration field table copied manually); C180 (Device identity comparison lacks second independent source); C181 (Pool heterogeneity criteria missed stripe bitwise XOR axis); C183 (One incompat bit per layout while disks show zero byte differences); C218 (Device table unit code assignment unsettled); C308 (Field offsets for self-authenticating units unsettled).

#### Settled Item 26: Three-Tier Mutability Classification for System Configuration

**Ruling**: System configuration divided into three tiers by "cost of alteration", each field assigned to exactly one tier:

| Tier | Segment | Cost of Alteration | Who Can Alter, When |
|---|---|---|---|
| Immutable system configuration | 389 bytes | Rebuild file system | Set at mkfs: fsid generated from a random source by mkfs's external entry point, first 8 bytes random, the inner entry point may supply fsid; journal ring size, unit size, placement granularity, devs supplied by user as mkfs parameters; immutable thereafter |
| Mutable system configuration | 4 bytes (node size) | **Rebuild index, without losing data** (single full run whole-disk I/O) | Configured at mkfs; altering requires index rebuild path, absent in version 1 |
| Runtime system configuration | 36 bytes | None | Configured with defaults at mkfs; **reconfigurable per mount, written back to disk** |
| (System runtime observables 60 bytes excluded: never user-configurable, maintained by file system) | — | — | — |

**Scope**: Governs "cost of alteration" axis, not state category (authoritative / derived in D21 (Boundary Between Authoritative and Derived States) Settled Item 9). System runtime observables 60 bytes are not configuration, excluded from three tiers: four categories differ across two axes — user-configurability and mutation timing; first three change only when altered externally, system runtime observables mutate on every publish. ⚠️ Omission of word "immutable" in name is deliberate: user named it "system runtime immutable variables" on 2026-09-20, yet values mutate continuously (slot generation increments per write, journal tail advances per publish); per `code-discipline.md` "one semantic concept one name repository-wide", applying "immutable" to mutating state risks caching as constants.

**Evidence**:

- No experiment: Three tiers categorize existing fields by cost of alteration, classification itself lacks measurable quantities; cost of each tier provided by respective clauses — node size tier governed by D8 (Core Index Structure) Settled Item 2 verbatim: "altering it = rebuild index, not rebuild file system"; remaining fields lack per-item evidence, owed under C426 (Per-field mutability of geometry segment lacks per-item evidence).
- User ruling 2026-09-20, original wording in change history.
- fsid generated from a random source by the external entry point, the inner entry point may supply it: tripartite verdict `research/prompts/m3-close-c656-r3-main-verification.md` Section 2 S1 generation cell and Section 3; user ruling 2026-10-20 "generate externally, keep the inner entry point", original wording in change history.

**Owed checks**: C422 (Mount option channel nonexistent); C423 (Whether writeback constitutes publish); C424 (Effective configuration invisible); C426 (Per-field mutability of geometry segment lacks per-item evidence).

#### Settled Item 10: Zone Write Pointer Cannot Serve as Authoritative Recovery Ceiling

**Ruling**: **No necessary algebraic relationship exists between zone write pointer (WP) and committed points recognized by SingleFS**, forbidding recovery judgments from treating WP as authoritative ceiling "commit point advanced at most here".

**Scope**: Settled Item 10 only excludes usage "WP = recovery authoritative ceiling", **without asserting WP has no role in recovery** — "serving only as addressing hint" remains open in Unsettled Item 6. **Does not resolve real crash semantics**: `zone_full` models static terminal state; mid-write crashes still require crash-point replay harness. ⚠️ **Cite this scope whenever citing Settled Item 10** — citing title alone implies "WP unrelated to recovery", which Settled Item 10 does not state. Settled Items 10 and 11 are exclusionary clauses, committing zero on-disk bytes.

**Evidence**:

- E125 (What is Read Above Sequential Zone Write Pointer): Device parameter `zone_full=1` produces state where "SingleFS committed bytes equal 0 while zone write pointer is at zone end".
- External documentation (unverified in this project): Local `~/code/fs-refs/linux-6.17/Documentation/filesystems/zonefs.rst` lines 236–242 verbatim: "the partial failure of a multi-BIO large write operation will cause the zone write pointer to advance partially, even though the entire write operation will be reported as failed to the user" — occurs on normal error paths, without awaiting crash.
- Tripartite: Unsettled Item 6 Round 1 three legs unanimously rejected arm "authoritative recovery ceiling" (`d22-item6-local-output.md`, `d22-item6-forward-sonnet-output.md`, `d22-item6-reverse-opus-output.md`), Round 2 deepened evidence.

**Owed checks**: None.

#### Settled Item 11: Zone Write Pointer Does Not Enter O2 (Independent Parser + Checker) Judgment Inputs

**Ruling**: D13 (Verification Route) ruling "O2 (independent parser + checker) evaluates **one** image" states verbatim (**full sentence**):

> **Ruling: Domain of O2 (independent parser + checker) is a single image** — a unary predicate `P(image) → holds/fails`,
> judging only set membership, never judging identity. **Any check requiring a second input (another image, or a record stream) does not belong to it.**

Zone write pointer is out-of-band state maintained by device, **not in image**; feeding it into checker provides second input outside domain — which final sentence strictly forbids. ⚠️ **Must be cited in full**: truncating at comma to "unary predicate" loses load-bearing clause "Any check requiring a second input... does not belong to it".

**Scope**: Settled Item 11 only excludes "WP entering O2 (independent parser + checker)", **without excluding runtime / mount-time cross-checks** — those represent distinct verification subjects, unassessed in repository.

**Evidence**:

- No experiment: Type-theoretic proof (unary predicate cannot accept second input), experiments neither add nor subtract validity.
- Tripartite: Unsettled Item 6 Round 1 three legs unanimously rejected checker form of arm "enters checker for cross-validation" (output files identical to Settled Item 10).

**Owed checks**: None.

#### Settled Item 12: System Configuration Does Not Carry Tree Table Unit Pointer

**Ruling**: **Authority of tree table resides in root record** (Settled Item 7 verbatim: "**Root record always routes via indirection layer: fixed header + one pointer to 'tree table unit', tree roots uniformly reside in tree table unit, never flattened into root slot.**"), **system configuration does not carry a second pointer to tree table unit**.

**Scope**: Settled Item 12 only states system configuration carries no **pointer to tree table unit**. It does **not** alter morphology of tree table unit itself, does not answer "which fields system configuration contains" (Settled Item 9), and does **not** resolve C212 (Fixed structure writes smaller than io_min). Associated with three unbacked debts, reflecting open areas rather than errors in Settled Item 12: C246 (Tree table unit lacks freeze layer assignment), C247 (Replica counts for tree table unit and instance table unit un-enumerated), C248 (Bootstrap exemption for tree table unit unwritten). Whether moving tree table length into ciphertext reduced or increased plaintext leakage remains unmeasured.

**Evidence**: Two independent counter-proofs, either sufficient alone to reject requirement:

- **Publish order contains no system configuration step**: D16 (Publish Semantics) Settled Item 7 persistence order is closed sequence; system configuration slot write occurs **after** publish sequence, entering crash-point replay enumeration without adding ordering points; adding tree table pointer mutating per generation forces fourth ordering point, reopening user ruling.
- **Rollback semantics**: System configuration does not rollback with root, whereas tree table pointer mutates with root ⇒ tree table pointer in system configuration would point to alternate timeline post-rollback.
- Collateral evidence (confirmed by main agent): Tree table unit itself is ciphertext (D9 (Encryption) verbatim: "All encrypted except system configuration"); providing plaintext pointer to non-key side yields unreadable entries, while D9 (Encryption) Settled Item 5 settled "scrub only", sealing path for "future non-key read-only mount".
- Isomorphic precedent: Allocation between system configuration versus root record decided previously for tree ID watermark (decision change history 2026-09-06 (Item 36)), assigned to root record.
- No experiment: Structural argument derived from publish sequence and rollback semantics, no measurable quantities; tripartite: reverse leg tasked with challenging "rejection", failing to construct viable path across 10 paths in two independent samplings (`system_configuration-treetable-reverse` leg report, `system_configuration-treetable-reverse-opus-s2-output.md`), satisfying `.claude/rules/three-way-inference.md` requirements.

**Owed checks**: C246 (Tree table unit lacks freeze layer assignment); C247 (Replica counts for tree table unit and instance table unit un-enumerated); C248 (Bootstrap exemption for tree table unit unwritten).

#### Settled Item 13: On-Disk Fixed-Length Fields Little-Endian, Registry Locations Assigned, Bitmap Bit Order Unified

**Ruling**: All on-disk fixed-length multi-byte fields (system configuration, root record, tree table entry, unit header, journal record header, position entry, key / value types) uniformly little-endian. Enum values avoid centralized master table, retaining distinct registry locations: unit class codes in D18 (Information Carried in a Block) Settled Item 11 unit class registry, tree kind codes in D8 (Core Index Structure) Settled Item 9, feature bit indices in D15 (Format Freeze Policy) Settled Item 4, encryption type values settled when D9 (Encryption) enables encryption, livelist event types in D6 (Snapshot Implementation Model) Settled Item 2 (ALLOC / FREE). Bitmap bit order: feature bits and remaining bitmaps store bit n at bit (n mod 8) of byte (n div 8) (Settled Item 17).

**Scope**: Big-endian rejected: E142 (Dry Run of New-Pool File Creation) harness and `crates/singlefs-core/src/bytes.rs` (`to_le_bytes`) write little-endian; switching to big-endian requires rewriting all serialization and artifacts. Remaining registry locations in Settled Item 17.

**Evidence**:

- No experiment: Endianness and registry locations are conventions, no measurable quantities; little-endian chosen because existing harness and implementation write little-endian.
- User ruling, original wording in change history.

**Owed checks**: None.

#### Settled Item 14: Scope of I-7.6 (Root Ring Regions on Mutually Distinct Disks) When devs < R

**Ruling**: I-7.6 (Root ring regions on mutually distinct disks) revised: Device identities of R regions cover min(R, devs) distinct values, with no disk carrying more than ⌈R / devs⌉ regions; when devs < R, mount not refused, fault tolerance commitment degrades with disk count (2 disks R = 3: losing Disk 1 incurs worst-case 1 generation, losing Disk 0 incurs worst-case 2 generations). R = 3 unchanged.

**Scope**: Bytes unchanged, modifies only invariant and checker; I-7.6 (Root ring regions on mutually distinct disks) evaluated by pool-level checker.

**Evidence**:

- E87 (Placement of Fixed Structures): On 2 disks R = 3, losing Disk 1 incurs worst-case 1 generation rollback, losing Disk 0 incurs worst-case 2 generations — on 2 disks R = 3 one disk inevitably carries two regions.
- User ruling (F = 2 in Settled Item 2; decided per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) Section 9), original wording in change history; unvetted by tripartite evaluation, main agent recommendation and user choices refutable.

**Owed checks**: None.

#### Settled Item 15: Quad-Linkages of System Configuration Field Table

**Ruling**: Version 1 omits indirect directory unit pointer (explicit blank, not reserved space); device descriptor table omitted in version 1, D12 (Target Media) Settled Item 5 clause revised to "first line has two homogeneous disks, omitted in version 1; must write prior to adding second layout", clarifying D12 (Target Media) Settled Item 4 capacity / layout bits / physical_block_size comparisons degrade in version 1 to pairwise disk comparisons; `devs` 4 bytes retained; compaction three watermarks 24 bytes constant 0 = built-in defaults. Settled Item 9 field table updated: removes 59, adds "unit area start slot" 8 (D3 (Space Allocation) Settled Item 10), "mkfs io_min" 4, and "fixed structure slot spacing" 4 (D2 (RAID Stripe Policy) Settled Item 19).

**Scope**: Only settles these four cells; remaining system configuration fields in Settled Item 9.

**Evidence**:

- E147 (System Configuration Field Table Recomputed per New-Pool File Creation Byte Table): Variant omitting indirect directory pointer fits within single sector.
- Pointer width 59 bytes lacked source; computed as 83 under D19 (Block Pointer Structure and Width Budget) Settled Items 7 / 8 pointer widths.
- User ruling (decided per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) Section 9), original wording in change history; unvetted by tripartite evaluation, refutable.

**Owed checks**: C218 (Device table unit code assignment unsettled); C180 (Device identity comparison lacks second independent source).

#### Settled Item 16: System Config Slot Rotation Rules; Root Ring Formula Address Space and Intra-Region Slots

**Ruling**: System configuration slots: Exactly 2 slots per disk, slot i intra-device offset = i × fixed structure slot spacing (D2 (RAID Stripe Policy) Settled Item 19); slot generation starts at 1 (both slots written at mkfs have generation 1), increments +1 per write; next written slot = generation mod 2; slot selection takes valid checksum with highest generation (evaluated per disk: sequence generation, post-publish rotation, and rollback on sequence generation failure all write highest self-authenticated generation on disk + 1). Slot 1 read offset determined in sequence: if local Disk Slot 0 usable (self-authenticated, geometry fields within current reader acceptance range), takes self-declared slot spacing; otherwise takes self-declared spacing of first usable Slot 0 in pool order; if pool has zero usable Slot 0, reader reads one slot at every format-permissible slot spacing (≥ 4096, multiple of 512, not exceeding root ring base minus 4096), accepting only slots self-authenticated whose self-declared spacing equals offset; if multiple tiers accepted, takes highest generation, tie-breaking by smaller offset; encountering self-authenticated slot with out-of-bounds pool fields rejects whole pool (Rule 5 "out-of-bounds rejects"), unparsed incompat bits ineligible without verifying offset. Reading across tiers checks unaccepted values first (including I-8.1 (Ring geometry sufficiently large) three journal ring geometry fields), rejecting whole pool immediately if present; otherwise verifies self-declared spacing matches offset. Root ring: `r × P × chunk` is intra-device offset, base address = system configuration "root ring base" field (version 1: 1 MiB), region r start = base address + r × P × chunk; intra-region slot = `(txg div R) mod S`, stateless, determined solely by txg and (R, S), promoted to Settled Item 2 parameter table. Five fixed rules:

1. **System configuration "root ring base" field is 16 KiB slot index**, sharing coordinate system with D3 (Space Allocation) Settled Item 7 placement slot indices and D19 (Block Pointer Structure and Width Budget) Settled Item 4 position entry physical offsets; version 1 writes **64** (= 1 MiB ÷ 16384). Base address (intra-device byte offset) = field × 16384; region r start = base address + r × P × chunk. ⚠️ Field stores 64, not 1048576. Readers parsing slots inspect this field and journal ring base field (offset 325), rejecting whole pool if unequal to version 1 constants (64, 1024); the root ring region count R (the 1 byte at offset 361) is read likewise, and a value other than 3 refuses the whole pool; a per-region slot count S out of bounds refuses the whole pool per Settled Item 2; when a slot has unrecognized incompat bits and its format version is also not 1, the whole pool is refused by format version first, not treated as unselectable (C663 (two readers classify system configuration slot out-of-bounds and unaccepted values differently), user settled 2026-10-20, original wording in change history; core: `RootRingRegionCountNotTheFirstVersionValue` and `mountable_slot_or_refusal` in `crates/singlefs-core/src/recovery.rs`; checker: `value_refused_by_this_reader` in `crates/singlefs-checker/src/image.rs`); spacing upper bound evaluated using same slot's root ring base; checker I-7.13 (System configuration pool-level fields within reader acceptance range) shares identical table with core (user settled, tripartite code verdict Round 2 `research/prompts/m2-closeout-code-r2-main-verification.md`). Implemented: core `crates/singlefs-core/src/system_configuration.rs` `parse_slot` reads offsets 325, 371 into `SystemImmutableConfiguration` fields `journal_ring_start_slot`, `root_ring_base_slot`; `crates/singlefs-core/src/recovery.rs` `system_configuration_values_this_reader_accepts` rejects mismatches via `SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot` / `JournalRingStartNotTheFirstVersionSlot`, slot spacing bound `largest_fixed_structure_slot_spacing_under_the_root_ring_base` (root ring base × 16384 − 4096); checker `crates/singlefs-checker/src/image.rs` `geometry_of` evaluates same table. Test cases: `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` `a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write`; `crates/singlefs-harness/tests/checker_known_bad_images.rs` `each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`; joint verification in `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs`.
2. **Root slot index j intra-device offset = region start + j × fixed structure slot spacing** (slot spacing = max(4096, mkfs io_min), D2 (RAID Stripe Policy) Settled Item 19). Setting spacing 512 complies with Settled Item 2 literal text, but byte table requires root of new-pool file creation to land at 1 MiB + 4096; this rule fixes index-to-offset conversion.
3. **P = 3, chunk = 1 MiB are version 1 mkfs constants**; mkfs verifies `base + (R−1) × P × chunk + S × spacing ≤ device available bytes`, refusing mkfs on overflow. P = 3 is smallest prime satisfying "P prime and P > devs" on 2 disks; chunk = 1 MiB matches base tier ⇒ three region starts land at 1 / 4 / 7 MiB, matching [layout/01-first-txn.md](../layout/01-first-txn.md) byte table. ⚠️ Constraint "P > devs" lacks justification (rationale tied to `gcd(P, devs) = 1` bijection, invalidated by D2 (RAID Stripe Policy) Settled Item 7 per-region written attribution), debt in C217 (Rationale for P greater than device count removed while constraint remains); taking 3 independent of constraint.
4. **Root ring three regions have fixed device attribution 0 / 1 / 0 in version 1** (Region 0 → Disk 0, Region 1 → Disk 1, Region 2 → Disk 0), written per-region by mkfs into system configuration 12-byte "root ring per-region device identity" (D2 (RAID Stripe Policy) Settled Item 7). ⚠️ **Not an mkfs parameter**: devs = 2, R = 3 under D22 (Synthesizing Unit Atomicity) Settled Item 14 pigeonhole bound (covers min(R, devs) = 2 distinct values, no disk carries > ⌈3/2⌉ = 2 regions), leaving only "two regions on one disk, one on other", making 0 / 1 / 0 sole canonical form.
5. **Warm-up empty publishes = 2, new-pool file creation txg = 3 are format constants**, not runtime predicates (`format-const` WARM_UP_EMPTY_PUBLISHES = 2, NEW_POOL_FILE_CREATION_TXG = 3, registered in [layout/01-first-txn.md](../layout/01-first-txn.md) VIII). D16 (Publish Semantics) Settled Item 8 requires "advancing until root of this instance covers both disks", and under 0 / 1 / 0 attribution, txg 1 lands on Region 1 (Disk 1), txg 2 lands on Region 2 (Disk 0), covering both disks in exactly two steps. `NEW_POOL_FILE_CREATION_TXG` applies solely to mkfs sequence (mkfs → sequence generation → warm-up twice → new-pool file creation within single process), not initial file versions on arbitrary pools: pools reopening post-mkfs writing first file take current root + 1. Reader verifies slot spacing: ≥ 4096, Slot 1 entirely precedes self-declared root ring base; `physical_block_size` ∈ [root record width 466, slot spacing]. Out-of-bounds rejected: checker reports I-7.13 (System configuration pool-level fields within reader acceptance range) on out-of-bounds slot, other invariants not applicable, refusing mount pool-wide (user settled 2026-09-27).

**Scope**: Replacing region formula with "mkfs writes start per region" deferred to tripartite before zoned line opens. Reading Slot 1 via version 1 constant 4096 when pool lacks usable Slot 0 (user settled 2026-09-14 "Option A") overturned by user 2026-10-02 to tier-by-tier scan, original wording in change history; tie-breaker "equal generation takes smaller offset" applies only to media corruption artifacts following mkfs zeroing of [0, root ring base) (Settled Item 8 Rule 3). Pool-level checker, recovery, and GPU judge parse per identical specification ([layout/01-first-txn.md](../layout/01-first-txn.md) Section "Location of system configuration slot 0 and slot 1").

**Evidence**:

- E142 (Dry Run of New-Pool File Creation): Writes all fixed structure bytes of new-pool file creation (system configuration slot 0 / 4096, root ring 1 / 4 / 7 MiB, new-pool file creation root landing on Region 0 Slot 1), byte-for-byte identical across 21 regions against `crates/` implementation.
- E87 (Placement of Fixed Structures): On 2 disks R = 3 one disk inevitably carries two regions — 0 / 1 / 0 is canonical half.
- Per-disk evaluation reading: User ruling after three rounds of tripartite evaluation on C322 (How barrier is placed during sequence generation lacks clause) (pool-wide max + 1 causes lagging disk to skip 2, falling back to chosen slot), verdict in `research/prompts/c322-r3-main-verification.md`.
- User ruling (decided per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) Section 9; Rules 4 / 5 settled in subsequent dialogs), original wording in change history; unvetted by tripartite evaluation, refutable.
- User ruling 2026-09-23: `NEW_POOL_FILE_CREATION_TXG` applies solely to mkfs sequence (`research/prompts/m2-refusals-presumed-r1-main-verification.md` Section 3), original wording in change history.
- User ruling 2026-10-01: "mkfs initializes both slots to generation 1" incorporated, evaluated jointly with I-7.16 (Relationship between system configuration slot generation and slot index) (`research/prompts/m3-inv-b-r1-main-verification.md` Section 4 Item 1), original wording in change history.
- Tier-by-tier reading when pool lacks usable Slot 0, tie-breaking, and two exit paths: Tripartite verdict `research/prompts/m3-c651-r1-main-verification.md` Section 2 and Section 4 Items 1, 2; `research/prompts/m3-formal-r6-main-verification.md` Section 2; test case `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`.
- Tier-by-tier evaluating unaccepted values before verifying offset: Code verdict `research/prompts/m3-formal-code-r2-main-verification.md` Section 2 Line "K3-a" and `research/prompts/m3-formal-code-r3-main-verification.md` Section 2 Line "K3".
- User ruling 2026-10-02: Re-incorporated tier-by-tier reading with initial steps and tie-breaking; switched to tier-by-tier when pool lacks usable Slot 0, overturning 2026-09-14 "Option A", original wording in change history.
- Root ring region count R read with whole-pool refusal, per-region slot count S out of bounds refusing the whole pool, and format version first when unrecognized incompat bits and a format version other than 1 coexist: user ruling 2026-10-20 (C663 (two readers classify system configuration slot out-of-bounds and unaccepted values differently)), original wording in change history.

**Owed checks**: C217 (Rationale for P greater than device count removed while constraint remains).

#### Settled Item 17: Completion of Registry Locations for Item 13

**Ruling**: Registry for journal record types in D23 (Journal Role & Format) Settled Item 1 (0 invalid, 1 normal record); metric tags in D5 (Snapshot / Space Accounting Mechanism) Settled Item 10; algorithm types in D9 (Encryption) (0 unencrypted, 1 ChaCha20-Poly1305, 2 AES-256-GCM); tree table entry / unit header flags clauses state "version 1 constant 0, non-zero rejected, bit index table empty", root record flags bit 0 is unmount flag, remaining bits constant 0 (Settled Item 7); bitmap bit order (bit n at bit (n mod 8) of byte (n div 8)) incorporated into Settled Item 13.

**Scope**: Only registers where values reside; values written in new-pool file creation remain unchanged.

**Evidence**:

- No experiment: Registry task, no second option; non-zero codes for algorithm types assigned in order specified by D9 (Encryption) Settled Item 3: "both algorithms must be implemented".
- User ruling (decided per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) Section 9), original wording in change history; unvetted by tripartite evaluation, refutable.

**Owed checks**: None.

#### Settled Item 18: Demarcation Line

**Ruling**: **Demarcation line: Whether a parent pointer carrying its checksum exists upon persistence before unit publication.** Yes ⇒ COW + D4 (Checksum Placement) suffices, tearing harmless (Settled Item 19, Settled Item 20); No ⇒ Must self-authenticate, requiring both properties simultaneously (Settled Item 21).

**Scope**: Categorizes units. Units with parent pointers are ordinary units; units without are root, system configuration, and journal (Settled Items 23, 24). Demarcation line cannot move upward (Settled Item 22).

**Evidence**:

- No experiment: Demarcation line represents structural argument categorizing by "whether parent pointer with checksum exists"; torn states non-reproducible locally (Settled Item 19).
- Tripartite evaluation (main agent ruling) lacks leg records: prompts and local leg outputs not found under `research/prompts/`; per `.claude/rules/three-way-inference.md` "absence of local leg must be reported explicitly" ⇒ claim "all three legs searched for counterexamples finding none" demoted to **negative result without leg records**, requiring fresh execution before supporting new conclusions.

**Owed checks**: None.

#### Settled Item 19: Ordinary Units with Parent Pointers: COW + D4 (Checksum Placement) Suffices Without Device Atomicity Assumptions

**Ruling**: Mechanism: Checksum resides in **another unit** (D4 (Checksum Placement)), and parent published only after child persists (D16 (Publish Semantics)). Any byte subset of child block being stale / fresh / zero / corrupt mismatches checksum in parent. ⇒ **Tear granularity being 512, 4 KiB, or whole block has zero impact on this proof.** **This is the substantive advantage of D4 (Checksum Placement) over "in-block self-contained checksum" — not saving bytes, but decoupling detection capability from device assumptions.**

**Scope**: ⚠️ **D22 (Synthesizing Unit Atomicity) Settled Item 19 answers whether tears are detected, not whether rollback candidates exist post-crash.** It proves "arbitrary byte subset of child block mismatches checksum in parent", independent of device atomicity; whether **prior version remains on disk is a separate clause** — D22 (Synthesizing Unit Atomicity) Settled Item 4 (rule "blocks freed in this transaction not reused" I-7.4 (Blocks freed in recent K generations not reused) exists) governs that half, whose failure mode is verbatim: "**detectable (parent pointer checksum mismatch), unrecoverable.**" ⇒ **Must cite this caveat alongside D22 (Synthesizing Unit Atomicity) Settled Item 19**: citing title alone "COW + D4 (Checksum Placement) suffices" discards rollback candidate prerequisites, repeating excerpt errors documented in `.claude/rules/three-way-inference.md`.

**Evidence**:

- No experiment: Torn states non-reproducible locally — `dm-flakey` provides only `drop_writes` / `error_writes` / `corrupt_bio_byte`, unable to model partial-block writes; crash-point replay assimilates torn states into "unpersisted", un-enumerating torn subsets (D13 (Verification Route) Settled Item 4).
- Tripartite evaluation lacks leg records (Settled Item 18).
- Production comparison (verified verbatim, unverified in this project): btrfs `struct btrfs_key_ptr { btrfs_disk_key key; __le64 blockptr; __le64 generation; }` lacks checksum, csum inside node header; self-referential checksum cannot distinguish "block belongs to current generation" from "valid block of some generation", forcing btrfs to maintain transid differential; ZFS `blkptr_t` has `blk_cksum` (256 bits), ordinary blocks lacking secondary gate ⇒ **Difference hinges on "where checksum resides"; D4 (Checksum Placement) buys capabilities of both btrfs gates at once.**

**Owed checks**: C16 (Pipeline declared failure model exception): Tear injection cannot rely on dm-flakey, requiring write truncation in crash-point replay harness.

#### Settled Item 20: Larger Tear Granularity Degrades Issue into Lost Writes Rather Than Worsening It

**Ruling**:

| Tear Outcome | Parent Pointer Checksum (D4 (Checksum Placement)) | In-Block Self-Contained Checksum |
|---|---|---|
| All-fresh | Passes (correct) | Passes (correct) |
| **Mixed** | Caught | Caught |
| **All-stale** | **Caught** (parent holds new checksum) | **Missed** — lost write, sealed by D18 (Information Carried in a Block) |

**Scope**: Detection comparison only. Direct corollary of D18 (Information Carried in a Block): larger tear granularity degrades into lost writes, and lost writes cannot be caught by block content.

**Evidence**:

- No experiment: Truth table derived from checksum location; torn states non-reproducible locally (Settled Item 19).

**Owed checks**: C16 (Pipeline declared failure model exception).

#### Settled Item 21: In-Place Overwrite Structures Require Both Whole-Unit Checksum and Actively Verified Generation Number

**Ruling**: Checksum alone is insufficient. A **completely unwritten** record reads **valid, complete legacy record** from prior cycle at same location — checksum passes, yet content is stale. ⇒ Checksum answers "is this unit intact", generation number answers "does it belong to this generation", **two questions, two fields.** SingleFS rejects XFS per-sector stamping (Axis ②).

**Scope**:

<!-- doc-registries:not-field-frequency refers to the rejected XFS per-sector stamping, not a field of this format -->
- ⚠️ **"Whether to adopt XFS per-sector stamping" spans two orthogonal axes, not single question**: Axis ① is sequence **identity** (cycle count vs transaction txg; XFS LSN is `(cycle:32, block:32)`, cycle packing sequence and offset into 64 bits), related to sequence bit-width; Axis ② is sequence **granularity** (per-record vs per-512-bytes), **unrelated** to bit-width. **"Replacing cycle with txg" touches only Axis ①, leaving Axis ② unaltered.**
- Rationale for rejecting Axis ②: "2⁻³² is sufficiently small, and switching to 64-bit checksum cheaply compresses it to 2⁻⁶⁴", **not "having txg eliminates need for cycle"** — latter is category error. Comparison set incomplete: stamping buys alignment freedom (resolution at sector level allows multiple records in single atomic unit to detect tearing), alignment cost measured in E23 (Journal Geometry). **Not reopened immediately**, but two prerequisites required before treating rejection as fully proven: ① Target workload fsync granularity — D25 (Target Workload Priority) coarse granularity favorable to rejecting Axis ②, prerequisite satisfied; ② Crash-point replay measuring whether tears land on sector multiples (assumption "device atomic width 512 ⇒ sectors independently fresh/stale" unmeasured).
- ⚠️ **Methodological note**: Deductions on tear patterns **non-falsifiable locally**, categorized as reasoning, not conclusions; falsification requires crash-point replay harness modeling torn states. ⚠️ **Scope covers tear patterns, excluding Unsettled Item 6** (write pointer leading commit point is not tear).

**Evidence**:

- No experiment: "Unwritten record reads valid old record from previous cycle" is structural fact of in-place overwrite; torn states non-reproducible locally (Settled Item 19).
- Trade-offs of Axis ②: Device atomic width 512 ⇒ sectors independently fresh/stale post-crash, stale sectors carry cycle stamp from previous cycle ⇒ deterministic detection; whole-record checksum only ⇒ miss probability ≈ 2⁻ⁿ (n = checksum width). Cost is stamp inside sector: shadow copies (`h_cycle_data[]`, header grows with record length) or reserving 8 bytes per 512 bytes (1.6% waste, complicates variable-length record offsets). XFS self-declares tear detection belongs to CRC; cycle used for binary searching head — policy, not structural limitation (external fact, unverified in this project).

**Owed checks**: C16 (Pipeline declared failure model exception): Tear injection.

#### Settled Item 22: Demarcation Line Cannot Move Upward: Root Ring Rollback Candidates Become Invalid

**Ruling**: **Converting units with parent pointers to "stable address + in-place overwrite + self-authentication" destroys root ring immediately.** I-7.4 (Blocks freed in recent K generations not reused) requires physical extents referenced by every root in rollback candidate set remain un-reallocated and un-cleared; otherwise blocks referenced by older roots are overwritten, rendering them **false candidates** (Settled Item 25). **In-place overwrite means "those blocks already overwritten"** ⇒ All roots in ring except newest become false candidates. ⇒ Maintaining old roots as genuine candidates requires retaining legacy replicas **up to block reuse bound** (`max(F_effective, oldest valid root in ring)`, D16 (Publish Semantics) Settled Item 1). In-place scheme pays COW delayed reuse space cost **plus** copy writes, accounting **holding identically at zero snapshots** — cost of root ring, not snapshot feature.

**Scope**: **Holds against strongest form of in-place scheme** (metadata self-authenticates per Settled Item 21, addresses allocated once immutably, parent pointers mutate only on split-merge), independent of retaining D4 (Checksum Placement). ⚠️ **In-place scheme wins on one axis, cannot be ignored**: Wins on write amplification axis (E38 (Accounting Structures: COW vs In-Place) in-place arm does not rewrite ancestors); eliminated by root ring, not write volume. Five arguments refuted in same round cannot reach strongest form, forbidden from re-use: in-place must rewrite entire Merkle spine; data side requires COW creating dual mechanisms; crash point explosion; D9 (Encryption) nonce reuse is hard barrier; D5 (Snapshot / Space Accounting Mechanism) birth criterion invalidated (refutations in decision change history).

**Evidence**:

- No experiment: Structural argument derived from I-7.4 (Blocks freed in recent K generations not reused) and block reuse bound, no measurable quantities.
- Tripartite evaluation (main agent ruling), repository preserves only local leg output `cow-necessity-local-en-output.md`.

**Owed checks**: None.

#### Settled Item 23: Journal Cannot Be Covered by COW: Holds for Fixed Rings, Belongs to Self-Authenticating Category

**Ruling**: **Holds for fixed rings**: To COW a journal record requires pointer referencing it; that pointer requires commit; commit is the task of journal. Circular. Non-circular journal requiring root write per record **nullifies entire journal benefit (amortizing root writes)**. ⇒ **Journal belongs to Settled Item 21 category**: Requires whole-unit checksum + actively verified generation number. ⚠️ Imposes hard requirement on journal prefix rules: **Generation number must reside in format, and must be actively verified by replay path.**

**Scope**:

- Chained log has alternative resolution: ZFS ZIL is not fixed ring; trailer in each block carries `blkptr` pointing to next block on chain, chain head stored in `zil_header_t.zh_log`, circularity broken by "pointer residing in prior record" (external fact, unverified in this project); SingleFS adopts fixed ring (Settled Item 1).
- Generation number is `jsn`: D23 (Journal Role & Format) verbatim assigns generation number to `jsn` ("Generation number: globally monotonic number at record granularity, not D5 (Snapshot / Space Accounting Mechanism) txg"), excluding `checkpoint_txg` (shared across window, unable to order); D23 (Journal Role & Format) Settled Item 14 record watermark uses lower bound, not fulfilling requirement. Does not reside in D16 (Publish Semantics) Settled Item 3 (journal freeze layer).
- Fulfillment status: `jsn` = instance generation 32 bits + counter 48 bits in format (D23 (Journal Role & Format) Settled Item 9); replay path `crates/singlefs-core/src/recovery.rs` `replay_journal` stops upon missing counter within instance or crossing instance boundaries (D23 (Journal Role & Format) Settled Item 14 note); checker enforcement under I-8.3 (Replay prefix strictly contiguous) unimplemented.

**Evidence**:

- No experiment: Circularity on fixed ring is definitional, no measurable quantities.
- Tripartite evaluation lacks leg records (Settled Item 18); fulfillment status verified during tripartite review (origin of C199 (Instance generation increment contradicts jsn stop on missing counter)).

**Owed checks**: C199 (Instance generation increment contradicts jsn stop on missing counter): Contradiction resolved, two crash-point replay cells and I-8.3 (Replay prefix strictly contiguous) checker remain.

#### Settled Item 24: Root Belongs to Same Category, and Both Layout Classes Require Identical Mechanisms

**Ruling**: **Root lacks parent, discarding all COW protections at once** — checksum in parent and atomic publish via parent write are both lost. ⇒ Root must self-authenticate: **whole-unit checksum (proves integrity) + generation number (proves current generation) + slot rotation (avoids overwriting sole replica).** "In-place overwriting root pointer" class **cannot overwrite sole replica in practice** — single tear destroys sole witness; it must rotate within fixed slot set with generation numbers; zoned class roots migrate, requiring discovery procedure for "where latest root resides". ⇒ **Difference lies in "how candidates are enumerated", not "candidate structure".** **Read order is mandatory requirement, not implementation detail**: Must **verify all candidates individually first, then select newest among valid ones by generation**; forbidden from "selecting newest by generation first, then verifying" — latter fails outright upon latest slot tearing instead of falling back to prior generation. When candidate key (`checkpoint_txg`, instance generation) ties, take earliest in root ring read order; pool-level checker root selection follows identical reading.

**Scope**: Overturns reading in D12 (Target Media) phrasing ("in-place overwriting root pointer" class); narrows gap in D16 (Publish Semantics): missing single mechanism, not two. Discovery procedure for zoned class un-designed, representing debt in D22 (Synthesizing Unit Atomicity) Unsettled Item 6. Recovery root selection in `crates/singlefs-core/src/recovery.rs` evaluates self-authenticated slots across all three regions taking largest `(checkpoint_txg, instance generation)`, breaking ties by earliest in root ring read order (`choose_root_slot` and `newest_root_among` share identical logic). Pool-level checker root selection (`crates/singlefs-checker/src/walk.rs`) follows identical logic: compares sequentially, swapping only when strictly greater, breaking ties by earliest in root ring read order.

**Evidence**:

- Root lacking parent is structural fact, read order is logical deduction.
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound): Read order "verify all candidates first, select newest among valid ones" switched to taking first valid candidate (halting scan once valid slot encountered) turns red within legal histories K, K9, KW bounds (Re-run 3 Section 3).
- Production reference (unverified in this project): ZFS "scan all labels all slots, select newest by txg" follows this exact sequence.
- Tie-breaking taking earlier slot: Code verdict `research/prompts/m3-formal-code-r1-main-verification.md` Section 2 ("Which slot taken when keys tie": core took earlier, `newest_root_among` took later, contradicting its docs; zero ties in legal histories) and "Fixes Applied" section; pool-level checker identical reading: Code verdict Round 2 `research/prompts/m3-formal-code-r2-main-verification.md` Section 2 Line "checker root selection tie-break takes later" and Section 3 Item 2.
- Tripartite evaluation lacks leg records (Settled Item 18).

**Owed checks**: C88 (Timeline discrimination for root ring unimplemented).

#### Settled Item 25: Root Ring Depth Is Upper Bound on Block Reuse Latency, Not Identical Values

**Ruling**: Retaining K generations of roots ⇒ **space freed within recent K transactions forbidden from reuse**; otherwise blocks referenced by older roots are overwritten, rendering them **false rollback candidates**. ⇒ **Do not read root ring as "can rollback K generations".** It buys only "if current root corrupted, previous generation root readable"; whether previous root is **usable** depends on block reuse latency — separate matter, separate cost. **Conflating the two creates illusion that rings are free.** ⚠️ **Block reuse bound cannot be format constant** (I-7.4 (Blocks freed in recent K generations not reused) citing D22 (Synthesizing Unit Atomicity)): Computed dynamically as `max(F_effective, oldest valid root in ring)` (D16 (Publish Semantics) Settled Item 1); F raised under space pressure, oldest valid root advances with publish. Implementing "root ring depth = block reuse latency" as format constant welds purely operational availability parameter, offering zero benefits. **Root ring only guarantees "if current root corrupted, previous generation still readable"**; checkpoint-scale block reuse constraints borne independently by D16 (Publish Semantics) Settled Item 15.

**Scope**:

- K is no longer runtime parameter: I-7.4 (Blocks freed in recent K generations not reused) verbatim: "Former rule 'K is runtime policy, lower bound ≥ 2' retracted"; "K generations" refers to rollback candidate set. Cost of buying extra generations under legacy K measurement basis analyzed in E123 (Trade-off: K vs Rollback Depth Costs); superseded after block reuse bound switched to dynamic calculation.
- D22 (Synthesizing Unit Atomicity) statement "one cycle duration ≥ worst-case block reuse latency demand" impossible at checkpoint scale (cycle is millisecond-scale); settled under consumer-costing reading: recovery root selection rollback requires exactly 2 generations, cycle = R × S ≥ 2 holds identically. Sole consumer costed by wall clock is traversal leap (reading parent pointer to child block step); if leap duration exceeds protection window (rollback generations ÷ publish rate), protection ceases (C214 (Window margin for traversal leaps unguarded)).
- Two block pinning rules target distinct blocks: Root ring rule pins blocks referenced by recent roots; D16 (Publish Semantics) Settled Item 15 + Settled Item 4 pin blocks freed within entire journal replay window. **Load-bearing metric is phase count, not multiple**: Extra escape hatch pinned by root ring exists for only `root_ring(K) − 1` phases within single checkpoint cycle, independent of checkpoint interval; ⚠️ **forbidden from claiming "250× difference"** (cross-phase comparison whose multiple scales linearly with checkpoint interval).

**Evidence**:

- E33 (Two Block Pinning Rules): Two pinning rules target distinct blocks; shrinking root ring depth frees space only during initial phases following checkpoint completion — legacy rationale "K pins sole escape hatch from ENOSPC deadlock" refuted; Deadlock 2 resolved via reserved pool.
- E50 (Upper and Lower Bounds on Root Ring Slots): Cycle duration is millisecond-scale; reading "cycle ≥ demand" at checkpoint scale impossible across any reasonable slot count.
- E92 (Worst-Case Block Reuse Latency Demand): Nine consumers priced across three categories: recovery root selection rollback exactly 2 generations, administrator rollback requires full ring depth, only traversal leap priced by wall clock.
- Block reuse bound: User ruling on D16 (Publish Semantics) Settled Item 1, original wording in change history.
- Evidence grade: Data represents observations; deduction "legacy rationale must be rewritten" represents inference, completed on local leg, two cloud legs owed.

**Owed checks**: C214 (Window margin for traversal leaps unguarded).

```

**出处 `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:512-546`（整段抄，未转述）**

```markdown
### Unsettled Items

| # | Item | Status |
|---|---|---|
| 6 | **Zoned write pointers as witnesses outside filesystem checksum jurisdiction** | Maintained by device, SingleFS cannot compute or verify; deferred to zoned line opening alongside discovery procedure for "where latest root resides". Modifies format: Modifies (2026-09-21, evidence: Question B addresses "whether fixed structures on zoned rotate in-place", text verbatim states "opening line costs at least +1 structure") **Status: Unsettled.** **Progress: No code object.** |

Unsettled Item 6 modifies new-pool file creation bytes: No (2026-09-01, evidence: Version 1 runs non-zoned layout, non-zoned lacks zone write pointer object. ⚠️ Unjudgeable if version 1 switches to zoned).

#### Unsettled Item 6: Zoned Write Pointers as Witnesses Outside Filesystem Checksum Jurisdiction

**Question**: Zone write pointer (WP) maintained by device; SingleFS cannot compute or verify it. Post-crash write pointer may lead committed points recognized by SingleFS. D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) load-bearing surface definition fails to cover it. Split into two questions: A, status of WP in SingleFS judgment; B, whether fixed structures (system configuration slot rotation, root ring slot rotation) on zoned rotate in-place.

**Current status**: Deferred to zoned layout line opening: Version 1 runs non-zoned layout (D12 (Target Media) Settled Item 5 "standard SSD" requires no ZNS); this object does not exist in version 1; avoids premature tripartite evaluation and avoids pre-settling "addressing hint only, not witness"; judged alongside discovery procedure for "where latest root resides" and D12 (Target Media) media tiering (user orientation, original wording in change history). Remains unsettled not from lack of evidence, but because prerequisite is an owed debt: Settled Item 2 represents **non-zoned parameters**, relying on overwriting identical slot addresses, whereas zoned root discovery procedure was never designed (Settled Item 24 leaves note: "requires a discovery procedure") ⇒ On non-zoned WP does not exist, while on zoned "slot" object does not yet exist; every arm must answer "where latest root resides" first. Known linkages upon line opening: D17 (Implementation Layering and Third-Party Pipelines) Settled Item 1 adopts single failure model ⇒ zoned line forbidden from pushing zone write pointers out via pipeline declaration, requiring core layer synthesis; D16 (Publish Semantics) Settled Item 8 warm-up imposes requirement on discovery procedure: "at most R empty publishes occupying slots, fsync returns only after roots cover two disks"; line opening costs at least +1 structural equivalence class (D17 (Implementation Layering and Third-Party Pipelines) Settled Item 2).

**Excluded arms**:

| Question | Arm | Status |
|---|---|---|
| A | Authoritative recovery ceiling | Excluded, promoted to Settled Item 10 |
| A | Enters checker for cross-validation | Checker form excluded, promoted to Settled Item 11; runtime / mount-time form unassessed |
| A | Addressing hint only (where to write next, where to start scan, excluded from newest / published / integrity judgments) | **Unwritable as clause**: "Where to start scan" in append-only zone equals "which are candidates", and root record field table (Settled Item 7) lacks row recording "which index in this zone"; requires either using WP to delimit candidate set (WP enters root selection, contradicting premise) or scanning entire zone (reading unwritten bytes above WP); boundary between "device-maintained state" and "device-probed geometry" undefined, while root slot width straddles it (C219 (Boundary between device-maintained state and device-probed geometry undefined)) |
| A | Candidate set defined directly as slots passing whole-unit self-authenticating checksum | **Excluded**: Conflates filter with enumerator — I-7.1 (Root slot valid set non-empty) domain "all root slot locations" and predicate "self-authenticating checksum passes" written separately, arm merely rewrote predicate; on zoned "slot" lacks referent; on non-zoned candidate set is R × S known locations, where WP does not exist |
| B | Zone-pair alternation + append write + zone reset | **Refuted by pure arithmetic**: `P` is odd prime ⇒ `v₂(P × chunk) = v₂(chunk)`, and `zone_size` is power of 2 ⇒ `zone_size` divides region spacing ⟺ `zone_size ≤ chunk`; root ring region starts fail to land on zone boundaries, regions share zones with other data, zone reset impossible. Survival requires altering settled clause (aligning start downward, requiring `zone_size ≤ chunk`, or abandoning zone reset) |
| B | Fixed structures uniformly land in conventional zones, devices lacking conventional zones rejected at mkfs | Contradicts Settled Item 24 "zoned class roots migrate"; requires rewriting that clause first; conventional zones absent in NVMe ZNS namespaces (kernel tree `include/linux/nvme.h` registers only `NVME_ZONE_TYPE_SEQWRITE_REQ`), shrinking applicability to zoned devices with conventional zones |
| B | Branch by zone type (land in conventional zone if present, else alternate zone pairs) | **Excluded**: Branch predicate lacks subject (pool-level branching variable alters in-flight, violating `.claude/rules/fs-design.md` Hard Requirement 5); actual criterion is whether conventional zone covers `{0, P·chunk, 2P·chunk}`; slots per region S is pool-level scalar in system configuration, unable to express dual forms, leaving "cycle = R × S" undefined |
| A / B | Await zoned line opening | Current status |

**Existing observations**:

- E125 (What is Read Above Sequential Zone Write Pointer) (`null_blk`, no physical zoned disk locally, simulated): Sequential zone WP readable above pointer (all `0xff`), `zone_full=1` models WP leading commit point; unwritten sequential zone reads `0xff`, conventional zone reads `0x00`; **never-written sequential zone and written zone post-`BLKRESETZONE` share identical bytes, write pointer, and zone state** ⇒ rule "all `0xff` ⇒ unwritten" distinguishing unwritten from corrupted lacks usable input on single device, unresolved by consulting WP.
- `0xff` not returned by device, but hardcoded line in `null_blk` driver (`memset(p + valid_len, 0xff, len);`); NVMe ZNS return value on unwritten blocks selected by `DLFEAT`, controller can error directly when bit in `NSFEAT` set, while Linux `drivers/scsi/sd_zbc.c` for host-managed devices executes `if (!sdkp->urswrz) return -ENODEV`; `zonefs.rst` lines 330–332: read-only zone write pointers invalid per ZBC / ZAC standards — when root reading is most critical (media failing), WP is precisely when values may be unavailable (external fact, unverified in this project).
- Tripartite: Unsettled Item 6 three rounds (Round 1 materials `_d22-item6-background.md`, Round 2 `d22-item6r2-*`, Round 3 `d22-item6r3-*`; Round 2 local leg three samplings invalidated).

**Owed checks**: C219 (Boundary between device-maintained state and device-probed geometry undefined); C230 (Post-reset state indistinguishable from never-written state while invariants all green); C17 (Device geometry treated as global parameter): stripe width × `zone_size` indivisibility cell must substitute `P`.

```

**出处 `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:547-558`（整段抄，未转述）**

```markdown
### Interactions with Other Decisions

| Decision | Interaction Point |
|---|---|
| D4 (Checksum Placement) | D22 (Synthesizing Unit Atomicity) provides rationale for D4 (Checksum Placement): **decoupling detection capability from device assumptions**. Production comparisons: btrfs (no checksum in parent pointer ⇒ forced transid addition) vs ZFS (checksum in parent pointer ⇒ no second gate) |
| D12 (Target Media) | Overturns reading of phrasing "in-place overwriting root pointer"; both layout classes require identical root mechanism (Settled Item 24) |
| D16 (Publish Semantics) | Gap in root redundancy mechanism narrowed to single mechanism; journal format gains hard requirement (generation number must be actively verified, Settled Item 23) |
| D18 (Information Carried in a Block) | D22 (Synthesizing Unit Atomicity) Settled Item 20 is direct corollary of D18 (Information Carried in a Block): larger tear granularity degrades into lost writes, and lost writes cannot be caught by block content |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | D22 (Synthesizing Unit Atomicity) provides answers to Corollary 3 Questions 1 and 2 of D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment); Question 3 (detection upon violation) lands on crash-point replay |
| D21 (Boundary Between Authoritative and Derived States) | Accounting is authoritative state lacking replicas; recovery path post-tearing is D22 (Synthesizing Unit Atomicity) Settled Item 3 ("Whether accounting structures use COW"; Settled Item 2 is root ring parameters) |
| [checks-owed.md](../checks-owed.md) C16 (Pipeline declared failure model exception) | Tear injection cannot rely on dm-flakey, requiring execution in crash-point replay harness |

```

**出处 `.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md:559-562`（整段抄，未转述）**

```markdown
## Version History

D22 (Synthesizing Unit Atomicity) historical entries are consolidated in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:1-6`（整段抄，未转述）**

```markdown
## D22 (ユニット原子性の合成手法) — 半確定 (1項目未確定)

[中文](22-单元原子性怎么合成.md) · [English](22-synthesizing-unit-atomicity.en.md) · **日本語**

D22 (ユニット原子性の合成手法) はユニットのディスク永続化における原子性をいかに合成するかを規定する：境界線（永続化後にそのチェックサムを担持する親ポインタが存在するか否か）によってユニットを「COW + D4 (チェックサム位置) で十分」と「自己証明が必須」の2種類に分類する；自己証明クラス（ルート、システム構成、journal）に求められるユニット全体のチェックサム、実際に検証される世代番号、およびスロットローテーション；ルートリングのパラメータ、ルートレコードおよびシステム構成のフィールド表、固定構造の配置先、ゾーン書き込みポインタをリカバリ判定の権威上限とせず O2 (独立パーサー + checker) にも含めないこと、ディスク上の固定長フィールドのエンディアンと登録表の位置。管轄外：パブリッシュの永続化順序とブロック再利用境界は D16 (パブリッシュセマンティクス)；journal のレコード形式、リング幾何構造、および再生プレフィックスは D23 (ジャーナルの役割と形式)；ユニットヘッダとユニットタイプコードは D18 (ブロックに含める情報)；ポインタのフィールドと幅は D19 (ブロックポインタ構造と幅予算)；書き込み粒度、固定構造スロット間隔、「ライフサイクルの相違」の判定基準は D2 (RAID ストライプ方針) 確定項目 15 / 16 / 17 / 19。

```

**出处 `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:7-511`（整段抄，未转述）**

```markdown
### 確定項目

**D22 (ユニット原子性の合成手法) の項目番号体系は単一であり**、「未確定項目」の節と通し番号とし、番号の振り直しは行わない。

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **journal は環状（リング）か** | 固定長リング、権威記録は D23 (ジャーナルの役割と形式) 確定項目 2 **状態：確定。** **進捗：実装済み。** |
| 2 | **ルートリングのパラメータ** | R = 3、領域は素数ステップで配置、スロット順序は領域間ローテーション、S はシステム構成内に保持、ルートスロット幅 = プローブされた `physical_block_size`、システム構成スロット幅 = 定数 4096、予約スロットなし **状態：確定。** **進捗：実装済み。** |
| 3 | **会計構造は COW か** | COW を採用、親ポインタ配下に配置 **状態：確定。** **進捗：実装済み。** |
| 4 | **「本トランザクション内で解放されたブロックは再利用禁止」I-7.4 (直近 K 代ブロック非再利用) は存在するか** | 存在：journal 再生ウィンドウ全体で解放されたブロックは再利用禁止 **状態：確定。** **進捗：実装済み。** |
| 5 | **D2 (RAID ストライプ方針) の計測条件拡張** | 完了済み：書き込み粒度に関する2つの厳格要件を D2 (RAID ストライプ方針) 確定項目 16 / 17 に記載 **状態：確定。** **進捗：対象なし。** |
| 7 | **ルートレコードのフィールド表** | 恒常的に間接層を経由：固定ヘッダ + 1本のツリーテーブルユニットポインタ、ツリールートはルートスロットに平坦展開しない；17フィールド計 466 バイト **状態：確定。** **進捗：未実装。** |
| 8 | **固定構造の配置先** | システム構成は各ディスク1部・スロットローテーション・番号取得は 2 ラウンド書き込み；journal リングは2ディスクでミラーリング；mkfs はまず各ディスクの [0, ルートリング基底アドレス) をゼロクリアしてバリアを通過させ、第0世代ルートを全ルートリング領域に植え付け、領域全体をゼロクリア **状態：確定。** **進捗：実装済み。** |
| 9 | **システム構成のフィールド表** | `layout/01-first-txn.md`「システム構成フィールド表」を採用：5段 46 行・489 バイト、4096 バイトのシステム構成スロットに格納 **状態：確定。** **進捗：実装済み。** |
| 26 | **システム構成の3段階可変性** | 変更時のコストに応じて3段階に分類：システム不変構成 389（ファイルシステム再構築）、システム可変構成 4（データ損失なしのインデックス再構築）、システム実行構成 36（コストなし、毎マウント時に再設定・書き戻し可能）；システム実行量 60 は3段階の対象外 **状態：確定。** **進捗：未実装。** |
| 10 | **ゾーン書き込みポインタをリカバリ判定の権威上限とできるか** | 不可 **状態：確定。** **進捗：対象なし。** |
| 11 | **ゾーン書き込みポインタを O2 (独立パーサー + checker) の判定入力に含められるか** | 不可：O2 (独立パーサー + checker) の定義域は単一イメージであり、ゾーン書き込みポインタはイメージ内に存在しない **状態：確定。** **進捗：対象なし。** |
| 12 | **システム構成に独自のツリーテーブルユニットポインタが必要か** | 不要、ツリーテーブルの権威はルートレコードに存在 **状態：確定。** **進捗：実装済み。** |
| 13 | **ディスク上の固定長フィールドのエンディアンと列挙エンコード規約** | すべてリトルエンディアン；列挙値は各登録表に配置；ビットマップのビット n は第 n div 8 バイトの第 n mod 8 下位ビットに格納 **状態：確定。** **進捗：実装済み。** |
| 14 | **devs < R のとき I-7.6 (ルートリング領域が互いに異なるディスクに配置) は何を判定するか** | 領域デバイス識別子は min(R, devs) 個の異なる値を網羅し、各ディスクあたり ⌈R / devs⌉ 個以下の領域；マウントは拒否せず、フォールトトレランスはディスク数に応じて縮退 **状態：確定。** **進捗：実装済み。** |
| 15 | **システム構成フィールド表の4項目連動：間接ディレクトリユニットポインタ 59 か 83 か、初期版でデバイス表を書き込むか、devs、整理3水域の初期値** | 間接ディレクトリユニットポインタは残さない、初期版でデバイス表は書き込まない、devs に 4 バイト確保、整理3水域は恒常的に 0 **状態：確定。** **進捗：実装済み。** |
| 16 | **システム構成スロットローテーション規則；ルートリング領域計算式のアドレス空間と領域内スロット位置** | システム構成は各ディスク2スロットで世代番号 mod 2 に従いローテーション（mkfs 時は両スロットとも 1）；ルートリングの起点は 16 KiB スロット番号；領域内スロット位置 = (txg div R) mod S；所属 0 / 1 / 0 **状態：確定。** **進捗：実装済み。** |
| 17 | **確定項目 13 の登録表一覧の補完** | journal レコードタイプ、統計量ラベル、アルゴリズムタイプ、3種類の flags、ビットマップビット順序にそれぞれ登録表を設定 **状態：確定。** **進捗：実装済み。** |
| 18 | **境界線** | ディスク永続化後にそのチェックサムを担持する親ポインタが存在してからパブリッシュされるユニットは、COW + D4 (チェックサム位置) で十分；存在しないものは自己証明が必須 **状態：確定。** **進捗：対象なし。** |
| 19 | **親ポインタを持つ通常ユニットは COW + D4 (チェックサム位置) で十分であり、いかなるデバイス原子性仮定にも依存しない** | 断裂書き込みで生じる任意のバイト部分集合は親のチェックサムと不一致となる **状態：確定。** **進捗：対象なし。** |
| 20 | **断裂粒度の拡大は問題を悪化させるのではなく、消失書き込みへと退化させる** | 全て旧データの断裂は親ポインタのチェックサムで捕捉可能、ブロック内自己完結チェックサムでは捕捉不能 **状態：確定。** **進捗：対象なし。** |
| 21 | **インプレース上書き構造には「ユニット全体のチェックサム + 実際に検証される世代番号」が必須であり、両者不可欠** | チェックサムは完全性を検証、世代番号は現世代であるかを検証；per-sector のスタンプは不採用 **状態：確定。** **進捗：実装済み。** |
| 22 | **境界線を上位層に引き上げることは不可：ルートリングのロールバック候補が無効化される** | 親ポインタを持つユニットをアドレス固定・インプレース上書き・自己証明に変更しない **状態：確定。** **進捗：対象なし。** |
| 23 | **journal を COW でカバーすることは不可能：固定長リングで成立、自己証明クラスに属する** | 世代番号がフォーマット内に存在し、かつ再生経路で実際に検証されなければならない **状態：確定。** **進捗：実装済み。** |
| 24 | **ルートも同一クラスに属し、2種類のレイアウトは同一のメカニズムを必要とする** | ルートの自己証明：ユニット全体のチェックサム + 世代番号 + スロットローテーション；全候補を個別に検証した後に世代番号で最新を選択、同キー時はルートリング読み出し順序の先頭を選択 **状態：確定。** **進捗：実装済み。** |
| 25 | **ルートリング深度はブロック再利用遅延の上限であり、両者は同一の値ではない** | ルートリングは「当世代のルートが破損しても前世代を読み出せる」ことのみを保証；ブロック再利用境界はその場で計算し、フォーマット定数としない **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：journal は固定長リングを採用

**確定内容**：journal は固定長リングを採用、権威記録は D23 (ジャーナルの役割と形式) 確定項目 2。決定的な理由は、チェーン方式ではデッドロック 3 の判定基準が「最悪操作 ≤ リングサイズ / F」（mkfs 時に計算可能）から「最悪操作 ≤ 現在の空き領域」に置き換わり、後者が操作の途中で変動するため、`.claude/rules/fs-design.md` の厳格要件 5 に抵触することである。

**射程**：リングはファイルサイズに伴って拡大しない：単一トランザクション予算を超える操作が発生した場合の出口は、実行時に強制的にトランザクションを分割することであり（D23 (ジャーナルの役割と形式) デッドロック 3 のフォールバック、I-8.1 (リングジオメトリが十分大きい)）、リングをファイルサイズに合わせて伸長させることではない。チェーン方式ログの出口とそれを採用しない理由は、確定項目 23 および D23 (ジャーナルの役割と形式) 確定項目 2 に属する。

**根拠**：

- 実験なし：確定項目 1 はポインタに過ぎず、リングとチェーンの取捨選択およびその実測根拠はすべて D23 (ジャーナルの役割と形式) 確定項目 2 に記載。

**未済の検査**：なし。

#### 確定項目 2：ルートリングのパラメータ

**確定内容**：

| 量 | 確定内容 |
|---|---|
| **領域数 R** | **3**（`R = F + 1`、F = 2） |
| **領域位置** | 領域 r は `r × P × chunk` に配置、**P は素数かつ `P > devs`**；アドレス空間、基底アドレスおよび初期版の値は確定項目 16 参照 |
| **スロット順序** | **領域間ローテーション**（領域 = `txg mod R`）；領域内スロット位置 = `(txg div R) mod S`（確定項目 16） |
| **各領域のスロット数 S** | システム構成内に保持。**下限 k_tol + 2 = 4**（D16 (パブリッシュセマンティクス) 確定項目 1：**各領域のスロット数 S** は 4 つの状態と k_tol 回のルートスロット書き込み失敗を伴う処理を収容できなければならない——その項目は直接 S 上で計測されたものであり、R × S 上ではない；「リング深度」という語は C131 (スタック深さと保護ウィンドウのディメンション) で問題が生じたため、ここでは用いない）、**上限は I-7.4 (直近 K 代ブロック非再利用) が拘束するブロック数としてマウント時に算出**；4..16 の間の値はどの境界にも触れない。⚠️ **具体的な値はフォーマット決定ではない**——フォーマットが約束するのは「S はシステム構成フィールド + マウント時に区間を判定」であり、値の決定は初期版実装が D25 (対象ワークロードの優先順位) の負荷形態に応じて決定 |
| **ルートスロット幅（ルートレコード1件のバイト数）** | **マウント時にプローブされた `physical_block_size` に等しい**、等号で十分。これより大きくしても安全性は向上せず、バイト消費が増加するだけである。mkfs はプール内の `max(physical_block_size)` に基づいて設定し、マウント時にデバイスごとに再計算して突合、未達時は**マウントを拒否**——形態は D21 (権威態と派生態の境界) の「ユニット幾何構造はマウント時に1回評価」と同型 |
| **システム構成スロット幅** | **フォーマット定数 4096**（登録名 `SYSTEM_CONFIGURATION_SLOT_BYTES`）、D23 (ジャーナルの役割と形式) 確定項目 12 のレコードサイズ 4 KiB と同一の先例；D15 (フォーマット凍結ポリシー) 第 1 層に属し、変更には incompat ビットを要する。D20 (荷重受け面: ユニットの原子性と自己完結性) の「スロット幅 = physical_block_size」は断裂判定の分解能であり、構造体サイズではない |
| **スロット間隔** | ルートスロットおよびシステム構成スロットのスロット間隔 = 4096 を mkfs 時にプローブされた io_min の整数倍に切り上げた値（D2 (RAID ストライプ方針) 確定項目 19）；ルートスロット番号からオフセットへの変換は確定項目 16 参照 |
| **アライメント** | 各スロットの開始オフセットは判定幅の整数倍とし、I-7.5 (ルートスロットが判定幅に整列) として着地 |
| **予約スロット** | **確保しない**。⚠️ **再審理条件**：マルチホスト保護を導入する際、D22 (ユニット原子性の合成手法) 確定項目 2 の予約スロット枠を再審理する |

**射程**：

- **適用条件：領域が存在するディスクが現存していること。この条項は結論と併せて引用されなければならない。** 領域位置は mkfs 時にディスク上に割り当てられた既成事実であり、**配置先の所属も同様である**——D2 (RAID ストライプ方針) 確定項目 7 に「所属は `(r × P) mod devs` による計算を廃止し、mkfs 時に領域ごとにデバイス識別子を書き込む」と明記 ⇒ **ディスクの追加や交換によって既存の領域の所属が変更されることはない。** ⇒ **R = 3 のフォールトトレランス保証は依然として縮退するが、そのメカニズムは「領域の衝突」ではなく、「領域が存在するディスクの喪失」である。** この枠は D2 (RAID ストライプ方針) 確定項目 3（明示的な幾何構造再評価トランザクションを経由）の具体例であり、D2 (RAID ストライプ方針) 確定項目 5（ルートリングは再配置しない）に属する。
- **`P` は「領域位置」という唯一の用途にのみ残る**：「`P` は素数かつ `P > devs`」という制約の本来の理由は「`gcd(P, devs) = 1` ⇒ P の乗算はモジュロ `devs` の全単射 ⇒ 領域は対ごとに異なるディスクに配置される」であったが、その理由は所属の計算式とともに破棄された；制約自体を維持すべきか否かはリポジトリ内に一切記述がない（C217 (Pがデバイス数超の制約理由消滅と制約残存)）。
- ルートスロット幅を `physical_block_size` とする場合、アレイ上の `io_min` はこれより遥かに大きい：スロット間隔を io_min の整数倍とした後、単一の io_min ユニット内に2つのルートスロットが混在することはなくなる（D2 (RAID ストライプ方針) 確定項目 17 は D2 (RAID ストライプ方針) 確定項目 15 の判定基準に従い、同一領域内の異なるスロットをライフサイクルの異なるオブジェクトとみなす）；毎回のルートスロット書き込みは依然として io_min 未満であり、D2 (RAID ストライプ方針) 確定項目 16 に文面通り違反するが、初期版ではこれを認識した上で許容する（C212 (固定構造の書き込みがio_min未満)）。マッピングユニット全体に対する SSD 内部の RMW に対する注入試験は行われておらず、実デバイス上では RAID5 の write hole のみを計測している。
- システム構成スロット幅の4つの候補のコストは E126 (システム構成スロット幅の4つの候補のコスト) で計測済みであり、その「乙」アームはプローブ値 max(pbs, io_min) を計測したものであり、裁定における乙（記録されたスロット間隔）とは同一の量ではない。システム構成スロット幅の枠はルートスロット幅から分離されている：ルートスロット幅は依然としてプローブされた `physical_block_size` であり、初期版のローカルマシン `nvme0n1` の現在値は 512、D2 (RAID ストライプ方針) 確定項目 9 の初期版は2台のディスクで稼働 ⇒ 同種 512 プール上のスロットは 512 となる。
- ルートリングの縮退セマンティクスは明文化され、実デバイス上で計測されており、[pitfalls.md](../pitfalls.md) 第9項（縮退マウントにより単一コピー書き込みが発生し事態が悪化する問題、根本原因は縮退動作の設計欠落）に対する回答となっている。
- **読者がシステム構成を選択する際、ルートスロット幅と固定構造スロット間隔を判定する**：固定構造スロット間隔 ∈ [4096, 1 MiB − 4096]（上限はスロット 1 全体がルートリング基底アドレスの前に収まることであり、`root_ring::region_start(0)` からシステム構成スロット幅 4096 を引いた値）；ルートスロット幅（`physical_block_size`）∈ [ルートレコード幅 466 バイト (確定項目 7), スロット間隔]；境界逸脱時はプール全体のマウントを拒否し、領域スロット数 S の逸脱時と同一の処置を行う（実審査 A3a に反映：`crates/singlefs-core/src/recovery.rs` の `system_configuration_values_this_reader_accepts`、`SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds`）；プールレベル checker は I-7.13 (システム構成のプールレベルフィールドが読者の許容範囲内にある) 違反を報告し、当該プールの他の不変条件はすべて適用対象外と報告する（実審査 A3-checker-2 に反映：`crates/singlefs-checker/src/image.rs` の `judge_system_configuration_values_the_reader_accepts`）。
- ルートリングは「当世代のルートが破損しても前世代を読み出せる」ことのみを担い、「K世代ロールバック可能」を意味しない（確定項目 25）；zoned 上ではこのパラメータ群は適用されない（未確定項目 6）。

**根拠**：

- F = 2：プロダクト判断であり、E46 (領域デバイス識別子が min(R, devs) を満たす算術制約) / E47 (ルートリング失敗ドメインの損失) / E48 (ルートリング幾何構造の実現可能点) / E50 (ルートリングスロット数の上下限) の4箇所すべてで実験では回答できないと自認；ユーザー確定、原文は変更履歴に記録。
- E46 (領域デバイス識別子が min(R, devs) を満たす算術制約)：「ディスク先頭と末尾」の配置は一般的なストライプ幾何構造上で同一ディスクに落ちる ⇒ 領域は障害ドメインに基づいて選択されるべきであり、領域数 = 許容すべき障害ドメイン数 + 1。
- E47 (ルートリング失敗ドメインの損失)：ディスク先頭末尾配置では大半のデバイス数構成でディスク1台消失時にマウント不能となり、領域内連続スロット順序の最悪ロールバック幅は領域間ローテーションの8倍に達する ⇒ スロット順序は領域間ローテーションを採用。
- E48 (ルートリング幾何構造の実現可能点)：配置規則 × スロット順序を網羅探索した結果、実現可能点は「素数ステップ + 領域間ローテーション」の組み合わせのみであった。
- E50 (ルートリングスロット数の上下限)：計測された S の下限と上限はいずれも 4..16 の外側にあり、S は精密計算を要する量ではない。
- E53 (ディスク1台まるごと消失後にルートリングがマウントできるか)：仮想マシンの実ブロックデバイス上でディスクごとに実際にゼロ消去を実施し、4回すべてでマウント可能、最悪ロールバックは1世代であった。
- E55 (ストライプマッピング算術の実デバイス検証)：実 `dm-stripe` 上で素数ステップの6グループすべてが互いに異なるディスクに配置された。
- E41 (ルートリングスロット幾何 2×4×256)：スロット幅が判定幅未満の場合、1回の断裂で複数スロットが破壊され、半スロットオフセット時にリスクスロットが倍増する ⇒ スロット幅は判定幅とし、開始オフセットはこれにアライメントする；「2領域 × 4スロット × 256バイト」の候補はこのため差し替えられた。
- E34 (ルートリングスロット幾何)：素数ステップ下では単一の io_min ユニットが2つの領域を覆うことはなく、領域全体を消去しても最悪ロールバックは1世代；実 md/raid5 上で断裂はその行のみを汚染し、暴露面 ≠ 損失面。
- システム構成スロット幅：サードパーティ `research/prompts/d22-slotwidth-r1-main-verification.md`（順方向検証アームは記録のスロット間隔を支持、逆方向検証アームは定数 4096 を支持、ローカルアームは2回のサンプリングで不安定）の後、ユーザーが最終ポップアップで確定、原文は変更履歴に記録。
- 予約スロット：現役の唯一の参照例において有効スロット数は物理スロット数と等しくない——ZFS の `ub_txg % (VDEV_UBERBLOCK_COUNT − m)` において、`m` はマルチホスト保護（MMP）が占有している；本プロジェクトとの既知の相違点：本プロジェクトにはマルチホストセマンティクスが存在しない（外部事実、本プロジェクトでは未検証）。

**未済の検査**：C212 (固定構造の書き込みがio_min未満)；C217 (Pがデバイス数超の制約理由消滅と制約残存)。

#### 確定項目 3：会計構造は COW を採用

**確定内容**：会計構造は COW を採用し、親ポインタ配下に配置する。「まずインプレースで稼働させ、後に COW に移行する」という方針は存在しない：両方式は mkfs の段階で分岐する。

**射程**：会計構造の更新形態のみを規定する。書き込み増幅の軸においてはインプレースが優位であるが（E38 (会計構造の COW かインプレースか)、D25 (対象ワークロードの優先順位) の粒度変更時に再実行が必要）、方針を決定づけたのは書き込み量ではない；「ディスク全走査に要する時間」は未計測であり、プールサイズは構成項目である。

**根拠**：

- 確定項目 22：会計は権威態（D21 (権威態と派生態の境界)）であり、ルート配下に存在するため、インプレース上書きは旧ルートのロールバック候補を同様に無効化する——これは mkfs の予約モデルに依存しない。
- E38 (会計構造の COW かインプレースか)：COW アームでは mkfs 予約領域は常に 0 であり、断裂によって唯一のコピーが破壊されることはない；インプレースアームでは会計ツリーの幾何構造に応じた固定位置の予約が必要。
- ユーザー確定（ユニットの自己完結性 ⇒ 親ポインタ方式は追加の位置決め規約を要しない、D20 (荷重受け面: ユニットの原子性と自己完結性)、D18 (ブロックに含める情報) 確定項目 3）、原文は変更履歴に記録。

**未済の検査**：C95 (根拠が撤回された確定項目への未確認)：E38 (会計構造の COW かインプレースか) のリカバリコストの列は撤回され、インプレースアームの静的全量 (フル実行) 予約モデルは藁人形論法と判定されたため、確定項目 3 は確定項目 22 を支えとしている。

#### 確定項目 4：「本トランザクション内で解放されたブロックは再利用禁止」が存在

**確定内容**：「本トランザクション内で解放されたブロックは再利用禁止」は存在する。これは D16 (パブリッシュセマンティクス) 確定項目 15「checkpoint C 内で生じた一切の解放は、C がパブリッシュされるまで割り当て可能集合に進入してはならない」そのものである。⚠️ **ただし制約粒度は checkpoint より厳格である**：D16 (パブリッシュセマンティクス) の規則の第2の理由は **journal 位置**に基づいて記述されている——ブロックが位置 p で解放され、q > p で再割り当て・書き込みされた場合、`p < r < q` のプレフィックスを再生すると当該アドレスを指すツリーが得られるが、ディスク上のバイトはすでに q の内容となっている。⇒ 真の制約は checkpoint 境界ではなく、**journal 再生ウィンドウ全体**である。これを欠いた場合の障害モード：**検出可能（親ポインタのチェックサム不一致）、リカバリ不能。**

**射程**：btrfs は pin を用いて同一の動作を実現している（外部事実、本プロジェクトでは未検証）。これはルートリングのブロック固定ルールとは異なるブロック群を拘束しており、両者の分担は確定項目 25 参照。

**根拠**：

- 実験なし：D16 (パブリッシュセマンティクス) 確定項目 15 から journal 位置に基づいて演繹された論理的結論であり、定量的な量は存在しない。

**未済の検査**：なし。

#### 確定項目 5：D2 (RAID ストライプ方針) の計測条件拡張、完了済み

**確定内容**：D2 (RAID ストライプ方針) 確定項目 16 および確定項目 17 が反映された：デバイスの物理マッピングユニット未満の書き込みを発行しない（Linux 上での当該量は `physical_block_size` ではなく `io_min` である）、すなわち D2 (RAID ストライプ方針) 確定項目 16；ライフサイクルの異なる2つのオブジェクトに同一の物理マッピングユニットを共有させない、すなわち D2 (RAID ストライプ方針) 確定項目 17。

**射程**：確定項目 5 は登録の完了のみを記録する；2つの厳格要件の射程と判定基準は D2 (RAID ストライプ方針) に属する：「ライフサイクルの相違」は D2 (RAID ストライプ方針) 確定項目 15 に基づいて判定し、固定構造のスロット間隔は D2 (RAID ストライプ方針) 確定項目 19 に属する。固定構造の書き込みが io_min 未満であることは D2 (RAID ストライプ方針) 確定項目 16 と抵触するが、初期版ではこれを認識した上で許容する（C212 (固定構造の書き込みがio_min未満)）。

**根拠**：

- 実験なし：本項目は2つの厳格要件を D2 (RAID ストライプ方針) に反映する登録作業であり、定量的な量は存在しない。

**未済の検査**：C212 (固定構造の書き込みがio_min未満)。

#### 確定項目 7：ルートレコードのフィールド表

**確定内容**：**ルートレコードは恒常的に間接層を経由する：固定ヘッダ +「ツリーテーブルユニット」を指すポインタ1本であり、ツリールートはすべてツリーテーブルユニットに格納し、ルートスロットには平坦展開しない。** **「ツリーが少ないときは平坦展開、多いときは間接層」という分岐は設けない**：分岐変数がツリー数となり、ツリー数はマウント期間中に変化するため（書き込み可能ヘッドを1つ作成するたびにツリーが1本増加、D6 (スナップショット実現モデル) 確定項目 1）——`.claude/rules/fs-design.md` の厳格要件 5「分岐変数は操作の途中で変化してはならない」に抵触する。恒常的に単一の形態を維持する。

| フィールド | 幅 | 出典 |
|---|---|---|
| magic | 4 | — |
| fsid | 16 | I-1.4 (ブロックヘッダ fsid 一致) と同一計測条件 |
| flags | 4 | ビット 0 = アンマウントマーク：正常なアンマウントシーケンスで書き込まれたルートは 1 に設定（D16 (パブリッシュセマンティクス) 確定項目 1「正常アンマウント」）、その他のルートは 0、プールレベル checker はこれに基づき I-7.9 (ロールバック下限 F は F 引き上げ上限以下) を分岐；その他のビットは初期版では常に 0、非 0 は拒否（D22 (ユニット原子性の合成手法) 確定項目 17）。ビット n は第 n div 8 バイトの第 n mod 8 下位ビットに配置（確定項目 13） |
| **インスタンスコード** | **4** | D16 (パブリッシュセマンティクス) 確定項目 6 付則 1：D23 (ジャーナルの役割と形式) 確定項目 9 の journal インスタンス番号と**同一のカウンタ**；ルート選択において checkpoint_txg が同点の場合に値が大きい側を勝者とする（ルートリングを読む際は各ルートのスロットを配置先公式（確定項目 16）で照合し、第 0 世代ルートは mkfs がルートを置くスロットで別に認める；1 つの txg には配置先スロットが 1 つしかないので、同点は同一バイトの第 0 世代ルート同士にしか残らず、ルートリングの読み出し順で最初のものを取る（確定項目 24）；公式照合はまだ実装されておらず、[checks-owed.md](../checks-owed.md) C743 (再構築アロケータが新クラス記録時にルートが当該スロットに存在するか検証しない) に未済） |
| **checkpoint_txg** | 8 | D16 (パブリッシュセマンティクス) 確定項目 6：パブリッシュカウント、ローテーションキー、会計世代、再生水位は同一の値——**別途「パブリッシュコード」フィールドは設けない** |
| ツリーテーブルユニットポインタ | **86** | ポインタヘッダ 50 + 位置エントリ **14** × 2 コピー + インスタンスコード 4 + 誕生シーケンス番号 4（D19 (ブロックポインタ構造と幅予算) 確定項目 7 / 8；位置エントリ = デバイス ID 4 + 16 KiB スロット番号 6 + 暗号文チェックサム 4、D19 (ブロックポインタ構造と幅予算) 確定項目 4）；ブートストラップ免除、位置エントリが権威 |
| **ツリー ID 水位** | **8** | D8 (コアインデックス構造) 確定項目 8 ②：`max(ルートリング内の全ルートレコードの当該フィールド, 今回の checkpoint で発行された最大ツリー ID + 1)`、パブリッシュごとにルートレコードとともに書き直す |
| **ロールバック下限 F** | **8** | D16 (パブリッシュセマンティクス) 確定項目 1：再割り当て可能な境界 max(F_有効, リング内最古の有効ルート)；通常は固定、受入容量不足時に引き上げ、正常アンマウント時に現行版の txg まで引き上げ；システム構成にも別途記録（確定項目 9）、リカバリ後の有効値は2箇所から読み出された最大値を取る（D16 (パブリッシュセマンティクス) 確定項目 1「有効」） |
| 自己証明チェックサム | 32 | E23 (ジャーナル幾何) の `header_csum` と同一幅 |
| **インスタンス表ユニットポインタ** | **86** | C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定裁定 P2：インスタンス表ユニット（D18 (ブロックに含める情報) 確定項目 11 パッキングレコードタイプ 4）はコンテナインデックスに入れず、ルートレコードが直接保持；構造はツリーテーブルユニットポインタと同型 |
| **中央マッピングツリールートポインタ** | **86** | D19 (ブロックポインタ構造と幅予算) 確定項目 11：中央マッピングツリーのルートはルートレコードに格納しツリーテーブルには入れない、ツリーテーブルユニットポインタと同型（ブートストラップ免除）；mkfs およびウォームアップ時はオールゼロを書き込む |
| **割り当てレコードツリールートポインタ** | **86** | D16 (パブリッシュセマンティクス) 確定項目 9：ツリーテーブル 0 件のバージョンでレコード書き込みを行ったパブリッシュで書き込まれた割り当てレコードノード群；ツリーテーブル 0 件でレコード書き込みを経たバージョンのみ非ゼロを書き込み、mkfs の第0世代およびファイルを含むバージョンではオールゼロを書き込む；ツリーテーブルユニットポインタと同型 |
| **回収閾値 t** | **8** | D16 (パブリッシュセマンティクス) 確定項目 1「ルートスロットの今回の読み出し破損」行の「固定もディスクに永続化する」：このルートを書く時の回収閾値で、max(このルート自身が持つ F, アロケータがこのルートを記録した時の回収閾値) を取る。読み損じたか全ゼロで読み戻されたルートスロットが固定した閾値を、このプロセスのメモリ内だけに置かず、ルートとともにディスクに永続化する；ゼロユニットのパブリッシュは前の版の値をそのまま写し、mkfs の第 0 世代は 0 を書く；プールレベル checker の I-7.18 (ルートレコードの回収閾値は現に算出した閾値に等しい) が判定する |
| **固定スロット p** | **1** | D16 (パブリッシュセマンティクス) 確定項目 1「ルートスロットの今回の読み出し破損」行の「固定もディスクに永続化する」：この版の会計を作る時点でまだ固定されているルートリングスロットのうち、固定の値が最小のもの。固定の値 = 3·スロット + 領域で、そのスロットに住んでいた可能性のある最古の txg にあたる（リング長 3S 未満）；固定されたスロットがなければ 255 を書く（`ROOT_RECORD_NO_RING_SLOT_PINNED`）；固定されたスロットはこのプロセスがそこを書き潰す時に必ず解除し、そのスロットに本インスタンスが書いたルートが既に住んでいるのに p がまだそれを指していれば、プールレベル checker の I-7.19 (固定されたスロットには本インスタンスが書いたルートが住まない) が赤と判定する |
| **アルゴリズムタイプ** | **1** | D9 (暗号化) 確定項目 12 第 1 項（自己証明ユニットへ射程拡張）：0 = 暗号化なし；登録表は D9 (暗号化)、D22 (ユニット原子性の合成手法) 確定項目 17 |
| **nonce** | **12** | D9 (暗号化) 確定項目 12 第 1 項；ユニットヘッダの 12 バイトと同一計測条件（D18 (ブロックに含める情報) 確定項目 14） |
| **MAC** | **16** | D9 (暗号化) 確定項目 8 と同一計測条件；ルートレコードは自己証明構造であり、暗号化有効時は自身の MAC で保護される |

行順がそのままディスク上の順序であり、オフセットは行ごとに加算される：magic 0 / fsid 4 / flags 20 / インスタンスコード 24 / checkpoint_txg 28 / ツリーテーブルユニットポインタ 36 / ツリー ID 水位 122 / ロールバック下限 F 130 / 自己証明チェックサム 138 / インスタンス表ユニットポインタ 170 / 中央マッピングツリールートポインタ 256 / 割り当てレコードツリールートポインタ 342 / 回収閾値 t 428 / 固定スロット p 436 / アルゴリズムタイプ 437 / nonce 438 / MAC 450。

<!-- format-const: ROOT_RECORD_BYTES = 466 stale=根记录 371 字节|根记录合计 371|14 个字段合计 371 字节|合计 **371 字节**，512 槽内余量 141|3 × 371|合计 **457 字节**|根记录宽 457|15 个字段合计 457|457 / 512 槽|3 × 457|ROOT_RECORD_BYTES=457|ROOT_RECORD_BYTES = 457|`ROOT_RECORD_BYTES` 是 457|根记录（457 字节|根槽偏移 428..457|15 个字段合计|512 槽内余量 55|根记录合计 457|根记录 **457 字节**|根记录 [0, 457)|[457, 槽宽) -->
合計 **466 バイト**、512 スロット内の余白は 46——スロット幅は実行時プローブ値であり、余白は将来のフィールドのために残されている。

**ツリーテーブルユニット**：16 KiB のメタデータユニットであり、**81 ツリー/層** を収容、超過時はさらに1層ネストする（81² = 6561 ツリー）；エントリ幅は D8 (コアインデックス構造) 確定項目 8 が定めた **200 バイト**（ツリー ID 8 + エントリ長 2 + ツリーの種類 2 + flags 2 + ルートポインタ 86 + `previous_snapshot_txg` 8 + 誕生 txg 8 + ヘッド ID 8 + 予約 76）、ヘッダ幅はツリーテーブルユニット自身のタイプ 2 ヘッダ **131**（key = ツリー ID、key 幅 8 ⇒ 115 + 16、D18 (ブロックに含める情報) 確定項目 16 / 確定項目 18）：⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81**。⚠️ **分母には単一の解釈のみが存在する**：16384 から予約ビットを含むタイプ 2 ヘッダを引いた値であり、他の数値を引いてはならない（C157 (ツリーテーブル容量が2箇所で異なるエントリ幅で計算されている) が求めた統一計測条件そのものである；D8 (コアインデックス構造) 確定項目 8 と文面通り同一）。ツリーテーブルユニットのヘッダフィールドは D18 (ブロックに含める情報) 確定項目 7 のインデックスノードクラス識別セクションに従う；これは権威態走査の一部である（I-7.2 (最新ルートの完全走査可能性) の「権威態全体を走査する」はこれを経由して下位に到達する）。

**射程**：ルートレコードおよびそれが指すツリーテーブルユニットのみを管轄する；システム構成はツリーテーブルユニットポインタを持たない（確定項目 12）。⚠️ **再生水位の解釈**（D23 (ジャーナルの役割と形式) 確定項目 14）：リカバリは `(インスタンスコード, checkpoint_txg)` がルートより厳格に大きいレコードのみを適用する；tail は走査開始点の最適化に過ぎない。間接層のコストとして毎パブリッシュ時にツリーテーブルユニットの COW が1回余分に発生するが、これを認識した上で許容する；ツリー数が1層を超えて2層にネストした後に毎パブリッシュで余分に COW されるユニット数は絶対値として算出されていない（C148 (ツリーテーブルエントリ肥大化による間接層2段化)）。

**根拠**：

- E79 (ルートレコードの容量)：ルートスロット幅 = `physical_block_size` × ツリーテーブルの初期開放（D15 (フォーマット凍結ポリシー) 確定項目 2）× ヘッドごとの1ツリー（D6 (スナップショット実現モデル) 確定項目 1）の3条件が合わさり、平坦展開ではこの設計自身が確定した要素を収容しきれない ⇒ 間接層構造は構造上不可避である。
- 末尾 29 バイト（アルゴリズムタイプ + nonce + MAC）：ルートレコードと journal レコードヘッダは自己証明構造であり、親ポインタが MAC と nonce を担持することはない ⇒ 初日から予約枠を確保しなければ、暗号化を有効化した際に全体を平文免除とするか（`checkpoint_txg`、ツリーテーブルポインタ、マッピングルートポインタ、ツリー ID 水位の漏洩）、レイアウト変更を伴うことになる；D9 (暗号化) 確定項目 3 の原則は、暗号化のために確保されたビットは全ボリュームで確保し、他用途に流用しないことである。
- ユーザー確定（間接層とフィールド表、ツリー ID 水位、インスタンス表ユニットポインタ、ロールバック下限 F、中央マッピングツリールートポインタ、3本のポインタ 86、末尾 29 バイト）、原文は変更履歴に記録；3本のポインタの幅拡大と末尾 29 バイトはサードパーティ検証を経ておらず、反証可能である。
- ユーザー確定 2026-09-23：割り当てレコードツリールートポインタ 86 バイトを追加（C512 (树表 0 条的一版上被换下的实例表记在哪没有条款)）、合計 371 → 457、原文は変更履歴に記録。
- ユーザー確定 2026-10-20：ルートレコードに 8 バイトの回収閾値 t と 1 バイトの固定スロット p を追加（C773 (不良および全ゼロルートスロットの回収閾値固定がメモリ内のみ)；D16 (パブリッシュセマンティクス) 確定項目 1「ルートスロットの今回の読み出し破損」行の「固定もディスクに永続化する」）、合計 457 → 466、原文は変更履歴に記録。t をオフセット 428、p を 436 に置き、アルゴリズムタイプ・nonce・MAC を後ろへずらしたのはメイン agent の決定で、ルートリング第2ラウンドの順推論レグ Y4.2（`research/prompts/m3-close-rootring-r2-sonnet-output.md`「Y4.2 フィールド」節）に従う；攻撃を受けておらず、審査ラウンドは0回；実験なし：補ったのは二つのフィールドの幅とオフセットで、実験番号は未付与。
- ユーザー確定 2026-09-26：flags ビット 0 をアンマウントマークとする（I-7.9 (ロールバック下限 F は F 引き上げ上限以下) は (b)+(c) を併用）、原文は変更履歴に記録；実験なし：サードパーティプロトタイプ `research/prompts/m2-rollback-forward-r3-opus-model/` で3つの改訂方針の弁別力を計測（第3ラウンド裁定 K6：(a) は「受入枠の引き上げ過ぎ」を完全に落とすため除外；(b) は「受入枠を引き上げ過ぎたにもかかわらずアンマウントマークが付与されている」ケースを落とす；(c) はプールレベルイメージで判定しない）、実験番号は未付与；併用はメイン agent が導出したものであり、審査ラウンドは0回。
- インスタンスコードの行の同点：ルートリングサードパーティ第3ラウンド裁定 `research/prompts/m3-close-rootring-r3-main-verification.md` 第2節 C743 (再構築アロケータが新クラス記録時にルートが当該スロットに存在するか検証しない) のセル（同じ txg のルート 2 本の併存は確定項目 16 の配置先公式と衝突し、変えたのはこの行の同点の括弧；第 0 世代の例外は第1ラウンドが裸の公式を突いた後に補ったもので、第2ラウンドは攻撃したがヒットせず、第3ラウンドで弁護側が成立）。

**未済の検査**：C88 (ルートリングのタイムライン判別未実装)；C148 (ツリーテーブルエントリ肥大化による間接層2段化)；C308 (自己検証ユニットのフィールドオフセットが未定)；C743 (再構築アロケータが新クラス記録時にルートが当該スロットに存在するか検証しない)：ルートリング読み出し時の配置先公式照合。（C78 (ルートレコードにフィールド表がない) は 2026-10-11 に完済され C66 (フォーマットレベルの空白に細分化項目番号がない) に統合；C157 (ツリーテーブル容量が2箇所で異なるエントリ幅で計算されている) は 2026-10-11 に完済され C148 (ツリーテーブルエントリ肥大化による間接層2段化) に統合、未済の検査一覧の完済表を参照）。

#### 確定項目 8：固定構造の配置先

**確定内容**：3箇条、いずれも2ディスク構成の初期版（D2 (RAID ストライプ方針) 確定項目 9）を対象とする：

1. **システム構成は各ディスクに1部ずつ配置する。** その更新方法は確定項目 21 に従い（インプレース上書き構造にはユニット全体のチェックサムと、実際に検査される世代番号が必要）、スロットローテーションとして構成し、各ディスクに少なくとも2つのスロットを設ける；スロットの規則は確定項目 16、フィールドは確定項目 9 参照。
2. **journal リングは2台のディスク間でミラーリングする。** コストとして定常状態で各レコードが2回書き込まれる。**これは新たなオーバーヘッドではない**——D2 (RAID ストライプ方針) 確定項目 6 はもともとあらゆる書き込みに w≥2 を要求しており、本決定はその「あらゆる書き込み」を journal レコードに対しても明文化したに過ぎない。
3. **mkfs は第0世代ルートを全ルートリング領域に植え付けなければならない**；任意のメタデータを書き込む前に、まず各ディスクの [0, ルートリング基底アドレス) を全区間ゼロクリアし（初期版 [0, 1 MiB)、システム構成スロット 0 とスロット 1 を段階探索する全 2,033 段階を完全に覆い、使用済みディスク上で mkfs を再実行した際に旧プールのシステム構成スロットを残さない）、3つのルートリング領域を全区間ゼロクリアし（ルートを植えていない 3 × (S−1) 個のスロットを含む）、journal リングも全区間ゼロクリアする：3者とも mkfs 自身がブロックデバイスのゼロ書き込み操作を介して発行し、各ディスク上で [0, ルートリング基底アドレス)、ルートリングの3領域、journal リングに対してそれぞれ1回呼び出す（領域の所属はルートがどのディスクに植えられるかを定めるのみで、どのディスクをクリアするかを定めるものではない）、記録ストリームにはそれぞれ全区間ゼロクリアとして1ステップ記録する（D17 (実装階層とサードパーティパイプライン) 確定項目 2 第 1 条の第1種）、各ディスク5ステップ、2台で計10ステップであり、区間シーケンスとステップ数は [layout/01-first-txn.md](../layout/01-first-txn.md) 八「mkfs のルート植え付け」の行に準拠する；各ディスクの [0, ルートリング基底アドレス) をゼロクリアした後、2 台のディスクはそれぞれバリアを 1 回通し（無効化）、それから初めて他を書く；その他のゼロクリアとユニット書き込みの間にはバリアを挟まない。fsid は mkfs の外部向け入口が生成し、先頭 8 バイトは乱数源から取る；内部入口は fsid を与えてよく、テスト、治具、クラッシュポイント再生に用いる。植え付け完了後、「mkfs 直後で一度もパブリッシュしていない」状態は定常状態と同様となり、ルートは最大でも領域深度の世代数までしかロールバックしない。

**射程**：システム構成自身の更新プロトコル（スロットローテーションの世代番号、チェックサム幅）はユニットヘッダフィールド表（D18 (ブロックに含める情報) 確定項目 7）の計測条件に従い、確定項目 9 および確定項目 16 に属する；zoned メディア上でこれらの固定構造がインプレースにローテーション可能であるか否かは D22 (ユニット原子性の合成手法) 未確定項目 6 に属し、未解決である。フォールトトレランス保証は単一障害を対象とする：各ディスク1スロットのシステム構成障害は保証の範囲内であり、番号取得後に両ディスクの最新スロットが壊れ続けることも保証の範囲内である（ユーザーが 2026-10-19 に確定、原文は変更履歴に記録）：番号取得は 2 ラウンドで書き、各ラウンドでディスクごとに 1 回書いた後にプールバリアを 1 回通し、各ディスクの 2 スロットとも番号取得時の新番号を持ち、全か無かで失敗と判定したときは各ディスク 1 回だけ巻き戻す（`crates/singlefs-core/src/transaction.rs` の `write_the_witnessed_acquisition`、`roll_back_acquisition`）；各ディスクでそれぞれ1スロットのシステム構成が破損し、同時に最新ルートが一時的に読み出せない二重障害形態は保証の対象外である（ユーザーが 2026-09-28 に対象外と裁定）、その形態では F を引き上げるまでプールレベル checker は1件も赤を出さない（書き込み可能マウントの事前隔離によって保護されているが、保証は依然としてこれをカバーしない）、2件のテストケースを登録とし、着地点は [checks-owed.md](../checks-owed.md) 完済表の C583 (二重障害形態：各ディスク 1 スロットのシステム構成不良＋最新ルートが一時的に読めない) の行にある。書き込み可能マウントがシステム構成を読み出す際の証人はディスク単位で読み出す（S-識別：1スロットが断裂し、もう1スロットが自己証明を通過した状態を、デバイスの読み出しエラーと分離する）、判定不能 ⟺ いずれかのスロットでデバイスが読み出しエラーを報告し、かつ両スロットとも自己証明を通過したディスクが1台も存在しない（R2、ユーザー確定、審査ラウンド0回、原文は変更履歴に記録）。判定不能なディスク面にはオフラインの出口が一つある（C596 (証拠判定不能なシステム構成ディスク面が読み取り専用マウントのみ残る)）：管理者が 1 つの版を断言し、その版のルートまたはそのパブリッシュの末尾レコードが読み出せてそれであり、かつ各ディスクのユニット領域にある本プールのユニットの `birth_txg` の最大値がそれ以下である場合に限り、自己証明を通らないシステム構成スロットだけを書き直し、書くインスタンスコードは各ディスクの自己証明スロット、ルートリング、journal リング、ユニット書き込み順序の最大値を取り 1 を加えない；そうでなければ 1 バイトも書かない（ユーザー 2026-10-19 確定、2026-10-20 に `birth_txg` ゲートを追加、原文は変更履歴に記録；`crates/singlefs-core/src/system_configuration_repair.rs` の `rewrite_system_configuration_slots_for_the_asserted_version`）。あるディスクの 2 スロットがともに無効（本プールの自己証明を通過したシステム構成が 1 部もない）の場合、番号取得ステップでその 2 スロットを補って書くが、次のすべてが成り立つ必要があり、1 つでも成り立たなければ拒否する：当該ディスクが選択された版の全ユニットを持つ（リストが非空、各配置先を 1 回読み、バイト列が検証済みのものと同一）；当該ディスクのルートリングスロットと journal リングスロットにデバイスが読み出しエラーを報告したスロットがない；これらのスロットに選択された版より新しいルートやレコードがない（ルート選択の順序で）；当該ディスクのユニット領域に書き込み順序のインスタンスが選択された版のインスタンスより大きいユニットがない；選択された版のルートスロット側が欠けていない（遅延分岐のルート欠落側と同じ判定）。補い書きに失敗した場合は番号取得と同様に各ディスク 1 回巻き戻す（ユーザーが 2026-10-03 にこのステップを拒否と確定、2026-10-19 に例外を開き、2026-10-20 にユニット領域スキャンとリングスロット読み出しエラーの 2 点を追加、原文は変更履歴に記録；`crates/singlefs-core/src/mount.rs` の `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`、実装は 2 スロットとも読み出せていずれも自己証明を通らないディスクだけをゲートに通し、スロットにデバイス読み出しエラーがあるものは従来通り拒否）。消失書き込みの境界はクラス別に記述する：プール全体で単一の値を読み出す純粋形態では消失書き込み 120 ケース、一方のディスクが読み出しエラーを報告し他方がオールゼロを読み出す混合形態では消失書き込み 0 ケース（C331 (ルート選択逆転が確認済み書き込みを上書き) グリッド上のケース数、レプリカ上で計測）、消失書き込みケースはいずれも各ディスクでそれぞれ1スロットのシステム構成が破損し同時に最新ルートが一時的に読み出せない二重障害形態に属する。2 台のディスクが同じプールである場合、無効化が永続化される前のクラッシュ：旧プールは最後に確認された版を読み戻す；書き込み可能マウントの結末は現行の補い書き条項に従う。無効化が永続化された後、最後のバリアの前のクラッシュ：リカバリの結末は 2 種類のみ——プール全体の拒否（`NoValidSystemConfiguration`）、または mkfs の第 0 世代の空プール（ルートはインスタンス 0、txg 0、ファイルなし）を読み戻す；後者ではシステム構成の冗長性が欠けてよく、新スロットを欠くディスクの書き込み可能マウントは現行の補い書き条項に従って処置する。呼び出し側が自ら主張するパラメータ（例えば旧プールの fsid）でマウントし、ディスク上で選ばれたシステム構成と合わずに拒否される（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`）のは、呼び出し側の主張とディスクの不一致であり、ディスク面の結末には数えない。デバイスがバリアを守る場合、無効化が永続化された後はいかなる結末も旧プールのいかなる版も読み戻してはならず、新しいシステム構成と旧プールのルートが併存してもならない；バリアが 1 台のディスクにしか届かないものは C6 (ブロック層セマンティクス仮定の誤り) に属し、2 台のディスクが 2 つのプールに由来する起点は適用範囲外である。

**根拠**：

- 二重障害形態の非カバー：ユーザー 2026-09-28 確定、原文は変更履歴に記録；その形態の2件のテストケースは [checks-owed.md](../checks-owed.md) 完済表 C583 (二重障害形態：各ディスク 1 スロットのシステム構成不良＋最新ルートが一時的に読めない) の行にある。
- E87 (固定構造の配置)：2ディスクにおける8通りの障害組み合わせの網羅検証——システム構成を1部のみ配置した場合、そのディスクが失われるとプール全体がマウント不能となる（データ、ルート、journal が健在であっても無効）、各ディスク1部 + journal ミラーリングにより8ケースすべてでマウント可能となり、再生ウィンドウでの損失はゼロ；第0世代ルートを満杯に植え付けない場合、「mkfs 直後、パブリッシュ0回、いずれかのディスク消失」においてロールバック可能なルートが1つも存在しない。
- 全区間ゼロクリア：これを行わない場合、以前 singlefs をインストールしていたディスク上で mkfs を再実行した際、旧ファイルシステムがスロット 1..S−1 に残したルートレコードの自己証明チェックサムが通過してしまい、さらに `checkpoint_txg` がより大きくなる ⇒ 初回マウント時に前ファイルシステムのルートが選択されてしまい、I-7.1 (ルートスロット有効集合が非空) および I-7.3 (リング健全性) が全緑となる（推論であり未計測）。
- [0, ルートリング基底アドレス) 全区間ゼロクリア：サードパーティ裁定 `research/prompts/m3-c651-r1-main-verification.md` 第3節「使用済みディスク上で mkfs を再実行した際に残る旧プールシステム構成スロットの処理」の行およびユーザー確定（クリア後、使用済みディスクでスロット間隔を変更して mkfs を再実行し両ディスクのスロット 0 を断裂させても旧プールの残留スロット 1 が選択されることはなくなる：攻撃側がレプリカ装置上で計測しメイン agent が再実行して確認、マージ側では `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader` テストケースで判定）；`research/prompts/m3-formal-r6-main-verification.md` 第2節 mkfs 途中クラッシュの枠（クリアの文言が成立し、mkfs の第1バリア以降の枠を解消）。実装は `crates/singlefs-core/src/make_filesystem.rs` の各ディスククリアループの先頭行、テストケースは `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` の `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader`。
- ユーザー確定、原文は変更履歴に記録。
- システム構成の証人読み出しはディスク単位、判定不能時は R2 を採用：サードパーティ裁定 `research/prompts/m3-ring-r2-main-verification.md` 第2節第9行（ディスク単位読み出しが成立）および `research/prompts/m3-ring-r3-main-verification.md` 第2節第7・8行（ディスク単位読み出しに加え各ディスク統合の混合形態で消失書き込み 240、境界はクラス別に記述；R2 は0ラウンド）、第4節末「R2 検証」段（R2 は C331 (ルート選択逆転が確認済み書き込みを上書き) グリッド上で計測：純粋形態 120、混合形態 0、混合形態で拒否が 116 ケース増加；8回のグリッドはトランザクション担当者が新レプリカ上で再確認し弁護側ログとキーごと行ごとにバイトレベルで一致、グリッドはリポジトリ外）。ユーザー確定：R2 を採用に変更（原文は変更履歴に記録）；あるディスクで自己証明されたシステム構成が1部も存在しない場合は番号取得ステップも拒否、ユーザー 2026-10-03 確定、2026-10-19 に補い書きの例外を開く（原文は変更履歴に記録）。実験なし：数値はサードパーティレプリカおよび担当者のレプリカ上にあり、実験番号は未付与。
- mkfs の 2 パス書き込み順序と途中クラッシュの条文：サードパーティ裁定 `research/prompts/m3-close-c656-r1-main-verification.md` 第2節 V2（乙が成立、条件は 2 パス書き込み順序）、`research/prompts/m3-close-c656-r2-main-verification.md` 第2節 U2、`research/prompts/m3-close-c656-r3-main-verification.md` 第2節 S2、S3（合成形態と 2 パス書き込み順序が成立、マウントの 2 つの分句は削除、条文は弁護側報告 `research/prompts/m3-close-c656-r3-sonnet-output.md`「三の末」の稿を採用、攻撃ゼロ回の文言）；ユーザー確定 2026-10-20「認める、書き直し稿のとおり書き戻す」（原文は変更履歴に記録）。実験なし：数値はサードパーティレプリカ上で計測（区間シーケンス `2+12+1+1+1+4`、27 ステップ、断裂込み 31 195 状態）、実験番号は未付与。
- fsid は外部向け入口が生成し、先頭 8 バイトは乱数源から取る：同じ第3ラウンド裁定の第2節 S1 生成のセル（境界は第 8 / 9 バイト、両レグで計測）および第3節；ユーザー確定 2026-10-20「外部で生成、内部入口は残す」（原文は変更履歴に記録）。
- 番号取得を 2 ラウンドで書き、各ディスク 1 回だけ巻き戻す：サードパーティ裁定 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第2節 U1（2 ラウンド書き込みは第2・第3ラウンドとも攻撃側がヒットせず番号衝突 0、各ディスク 1 回の巻き戻しは 1 ラウンド攻撃されヒットせず、レプリカで計測）；ユーザー確定 2026-10-19「カバーする、番号取得は 2 ラウンドで書く」、2026-10-20 維持（原文は変更履歴に記録）。
- 番号取得ステップの補い書き例外：サードパーティ裁定 `research/prompts/m3-close-sysconf-r2-main-verification.md` 第4節、`research/prompts/m3-close-sysconf-r3-main-verification.md` 第2節 U4 および第3節（クローンディスクがユニットだけを書いた形が突かれた、推論；補ったユニット領域スキャンとリングスロット読み出しエラーの 2 点は攻撃ゼロ回）；ユーザー確定 2026-10-19「例外を開く、4 点のゲート」、2026-10-20 に 2 点の補い方を採用（原文は変更履歴に記録）。
- 判定不能なディスク面のオフラインの出口：同じ第3ラウンド裁定の第2節 U2（`birth_txg` ゲートなしでは前の版を断言すると確認済みの書き込みを失う、攻撃側が計測）および第3節；ユーザー確定 2026-10-19「許可、さらにユニット書き込み順序をスキャン」、2026-10-20「オフラインの出口に birth_txg ゲートを追加」（攻撃ゼロ回、原文は変更履歴に記録）。

**未済の検査**：C79 (システム構成と journal リングの配置が未確定)：journal リングの2ディスクミラーリングの分岐を強制到達可能な分岐とする（システム構成各ディスク1部、ディスク全体喪失の2分岐は強制到達可能、テストケースは C79 (システム構成と journal リングの配置が未確定) 参照）。

#### 確定項目 9：システム構成のフィールド表

**確定内容**：フィールド表 = [layout/01-first-txn.md](../layout/01-first-txn.md)「一、mkfs が植え付けたもの」の節にある「システム構成フィールド表」、46 行、合計 **489 バイト**であり、4096 バイトのシステム構成スロット（確定項目 2）に格納され、スロット内余白は 3607。フィールドは**可変性**に応じて再区分される：

| 新セクション名 | 出典セクション | バイト数 |
|---|---|---|
| **システム不変構成** | ブートストラップヘッダ内の magic 4、フォーマットバージョン 2、feature bits 96、fsid 16、書き込み者識別子 20、チェックサムアルゴリズム識別子 1、自ディスクデバイス番号 4、devs 4；暗号化予約全 114；幾何構造内のノードサイズを除く全フィールド | 147 + 114 + 128 = 389 |
| **システム可変構成** | 幾何構造内のノードサイズ 4 | 4 |
| **システム実行構成** | 調整可能値すべて：T_time 4、T_dirty 8、整理低/高/停止の3水域 8×3 | 36 |
| **システム実行量**（構成ではない） | スロット世代番号 8、全スロットチェックサム 32、journal tail 8、インスタンスコード 4、ロールバック下限 F 8 | 60 |

- 内容は 512（プローブされた `physical_block_size`）を超えることが許容される。超えた場合、1回の書き込みが半分のみ完了する可能性があるが、全スロットチェックサム（パディングを含むスロット全体 4096 をカバー）によって検出され、スロット選択規則（チェックサム通過かつ世代番号最大、確定項目 16）により自動的にもう一方のスロットへフォールバックする。**検出可能、リカバリ可能。**
- コストは認識の上で許容：クラッシュポイント再生において断裂の次元を1つ追加列挙する必要がある；断裂が発生したマウントでは前世代の構成が取得される。
- 厳格な上限は依然としてスロット幅 4096（`SYSTEM_CONFIGURATION_SLOT_BYTES`、フォーマット定数、D15 (フォーマット凍結ポリシー) 第 1 層）である。これを超えるには incompat ビットを要する。
- ⚠️ 現在値 489 は 512 を超えておらず、この条項は**許容**であり**すでに超えている**ことではない。実際にフィールドを追加して超えた際には、クラッシュポイント再生の当該次元および関連アサーションを同時に支払う。

- 暗号化予約の3項目：KDF 識別子は 4 バイト（2 ではない）、マスターキースロットはシステム構成スロット内にインライン化（間接ディレクトリユニットを用いず、最小利用可能値 80 バイトを確保）、暗号化タイプ宣言に 1 バイト確保。
- 初期版では間接ディレクトリユニットポインタを残さず、デバイス記述子テーブルを書き込まず、`devs` に 4 バイトを確保し、整理3水域は恒常的に 0 とする（確定項目 15）。
- ブートストラップヘッダには書き込み者識別子 20（実装識別子 16 バイト ASCII ゼロパディング + バージョン 4 バイト u32）とチェックサムアルゴリズム識別子 1（0 無効、1 CRC-32C）が存在；幾何構造セクションには mkfs 時の `physical_block_size` 4 と拡張点宣言値 N 4（いずれも幅はバイト単位）が存在。読者はシステム構成のフォーマットバージョンが 1 でない場合、プール全体のマウントを拒絶し、このスロットを選択不可とする扱いに退化させない。これは確定項目 16 第 1 文の C663 (システム構成スロット範囲外と不認領値で二読者の分類が異なる) の文と一致する（core は `crates/singlefs-core/src/recovery.rs` の `system_configuration_values_this_reader_accepts` が `FormatVersionNotRecognized` を報告し、`mountable_slot_or_refusal` が `RecoveryFailure::SystemConfigurationValueRefused` を返す）；ユニットヘッダのフォーマットバージョンが 1 でない場合は当該ユニットを拒絶する。いずれも現行フィールド表に従ってパースを続行しない（システム構成側はユーザー 2026-10-21 確定「条文をプール全体拒絶に改める」、原文は変更履歴に記録；ユニットヘッダ側はメイン agent 2026-09-27 確定；プールレベル checker の判定は反映済み：ユニットヘッダのフォーマットバージョンは I-2.4 (ヘッダチェックサムカバー範囲) に属し、システム構成のフォーマットバージョンは I-7.13 (システム構成のプールレベルフィールドが読者の許容範囲内にある) に属する、実審査 A3-checker-2）。

**射程**：新規プールでのファイル作成のバイト列を変更するか：**変更する**——mkfs、番号取得、ウォームアップ、パブリッシュによって書き出される各システム構成のバイトは本テーブルによって決定される。フィールドのエンコード（エンディアン、列挙値、ビットマップビット順序）は確定項目 13 / 17 により決定され、feature bit のビット割り当ては D15 (フォーマット凍結ポリシー) 確定項目 4 により決定される；スロットのローテーション規則は確定項目 16 参照。マスターキースロット 80 バイトの内部構成（ソルト、ラッピング形式）は未定であり、D9 (暗号化) の暗号化導入時に決定する。障害要件 ①/④ の後半（プールレベル単一値 vs デバイス単位、incompat ビットの出口）の論証はバイト列をブロックしない（C181 (プール内異種性判定基準がストライプビット単位 XOR 軸を欠落)、C183 (每套布局一个 incompat 位而盘上差不出字节)）。E115 (システム構成フィールド表の完全性とバイト予算) および E124 (D18 確定項目 14 によるシステム構成フィールド表の再計算) の4項目の数値はツリーテーブルユニットポインタと単独占有のマッピングルートポインタをスロット内に含めて計算しており、E147 (システム構成フィールド表の4項目連動) 以降は引用しない。内容は**プールレベル**であり、各ディスクに1部ずつレプリカを保持し、両スロットは同一内容となる；唯一のディスクレベルフィールドは「自ディスクデバイス番号 4」であり、ディスクごとに異なる構成を設定できるわけではない。本スロットはさらに 60 バイトの**システム実行量**（スロット世代番号 8 + 全スロットチェックサム 32 + journal tail 8 + インスタンスコード 4 + ロールバック下限 F 8）を格納するが、これらは構成ではない：値はパブリッシュまたは F の引き上げに伴って変化し、ユーザーが設定することはできない；このうち journal tail とインスタンスコードの 12 バイトをここに置くべきか否かは C245 (ルートレコードの間接層が2段間接で打ち切られる) に属する。

**根拠**：

- E147 (システム構成フィールド表の4項目連動)：バイト表に基づいて行ごとに再計算し、暗号化予約の3項目それぞれで最広のバリアントを選択しても、すべて単一セクター内に収まる——3項目はエンコードの選択であり、予算の選択ではない。
- 書き込み者識別子、チェックサムアルゴリズム識別子、mkfs 時の `physical_block_size`、拡張点宣言値 N の4項目は、それぞれ確定済みの条項または不変条件によってシステム構成に配置することが明記されている：I-1.5 (システム構成記パイプライン同一性) と D17 (実装階層とサードパーティパイプライン)「未済の負債」第3項；I-2.2 (プール全体でチェックサムアルゴリズムが一致) と C309 (32 バイトチェックサムのアルゴリズム未定)；D13 (検証ルート) 確定項目 5 と D12 (対象メディア) 確定項目 4 の `physical_block_size` 突合；D21 (権威態と派生態の境界) 確定項目 8。前2項目には「事後に追加できない」性質がある：書き込み者識別子は mkfs 時の事実であり、チェックサムアルゴリズム識別子は第2のアルゴリズムが存在した際に旧イメージを識別するために不可欠となる。
- ユーザー確定（暗号化予約の3項目、4フィールド）、原文は変更履歴に記録；4フィールドはサードパーティ検証を経ておらず、反証可能である。
- ユーザー確定 2026-09-26：システム構成にロールバック下限 F 8 バイトを追加（C419 (F 生效之后靠什么保持不回落) は SysPre を採用、D16 (パブリッシュセマンティクス) 確定項目 1「有効」）、ロールバック目撃表を削除（D23 (ジャーナルの役割と形式) 確定項目 14）、原文は変更履歴に記録；実験なし：サードパーティプロトタイプ `research/prompts/m2-rollback-forward-r3-opus-model/` で凍結レプリカ上で SysPre を計測（第3ラウンド裁定 K4）、実験番号は未付与；F をフィールド表末尾のオフセット 481 以降に配置することは本仕様が「システム実行量の末尾に追加」に従って決定したものであり、審査ラウンドは0回。
- ユーザー確定 2026-10-21：システム構成のフォーマットバージョンが 1 でなければプール全体を拒絶（選択肢「条文をプール全体拒絶に改める」）、原文は変更履歴に記録；発端は条文が元々そのスロットを拒絶すると書いており、`crates/singlefs-core/src/recovery.rs` の実装および確定項目 16 第 1 文と合わなかったこと（マイルストーン三の締めくくり記録 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 560 行）。

**未済の検査**：C178 (書き込みポインタを権威リカバリ上限とできない理由)；C180 (ルートスロット幅と物理ブロックサイズの不一致)；C181 (プール内異種性判定基準がストライプビット単位 XOR 軸を欠落)；C183 (每套布局一个 incompat 位而盘上差不出字节)；C218 (ルートリングスロット番号とシステム構成スロット番号のアドレス空間)；C308 (自己検証ユニットのフィールドオフセットが未定)。

#### 確定項目 26：システム構成の3段階可変性

**確定内容**：システム構成は「変更時に何を支払うか」に基づいて3段階に分類され、各フィールドはいずれか1つの段階に排他的に属する：

| 段階 | セクション | 変更のコスト | 誰が変更可能か、いつ可能か |
|---|---|---|---|
| システム不変構成 | 389 バイト | ファイルシステム再構築 | mkfs 時に設定：fsid は mkfs の外部向け入口が乱数源から生成し先頭 8 バイトは乱数、内部入口は fsid を与えてよい；journal リング長、ユニットサイズ、配置粒度、devs はユーザーが mkfs パラメータとして与える；以降変更不可 |
| システム可変構成 | 4 バイト（ノードサイズ） | **インデックス再構築、データ損失なし**（フルディスク I/O 1回分） | mkfs 時に設定；変更にはインデックス再構築経路を経由する必要があるが、初期版にその経路は存在しない |
| システム実行構成 | 36 バイト | なし | mkfs 時にデフォルト値を設定；**毎マウント時に再設定可能であり、ディスクへ書き戻す** |
| （システム実行量 60 バイトは3段階に含まれない：ユーザーは設定不可、ファイルシステム自身が保守） | — | — | — |

**射程**：管轄するのは「変更時に何を支払うか」という軸であり、状態の区分ではない（権威態 / 派生態は D21 (権威態と派生態の境界) 確定項目 9）。システム実行量の 60 バイトは構成ではなく、これら3段階には属さない：4分類は2本の軸でのみ相違する——ユーザーが変更可能か否か、値がいつ変化するか；前3者の値は変更者が存在するときにのみ変化するが、システム実行量は毎パブリッシュ時に変動する。⚠️ 名称に「不変」の語を含めないことは意図的である：ユーザーは 2026-09-20 に一時「システム実行不変量」と命名したが、その値はまさに常に変化しており（スロット世代番号は書き込みごとに +1、journal tail は毎パブリッシュで増加）、`code-discipline.md` の「単一のセマンティック概念には全リポジトリで単一の名称を用いる」規則に従い、同一の「不変」が2箇所で異なる事象を指すと定数としてキャッシュされる危険がある。

**根拠**：

- 実験なし：3段階は「変更時に何を支払うか」に基づいて既存のフィールドにラベルを付与したものであり、分類自体に定量的な量は存在しない；各段階のコストはそれぞれの条項から導出される——ノードサイズの段階は D8 (コアインデックス構造) 確定項目 2 の射程に「変更 = インデックス再構築でありファイルシステム再構築ではない」と明記、その他のフィールドには個別の根拠がなく、それこそが C426 (システム構成の3段階可変性が列挙登録表に未反映) の未済の検査である。
- ユーザー確定 2026-09-20、原文は変更履歴に記録。
- fsid は外部向け入口が乱数源から生成し、内部入口は与えてよい：サードパーティ裁定 `research/prompts/m3-close-c656-r3-main-verification.md` 第2節 S1 生成のセルおよび第3節；ユーザー確定 2026-10-20「外部で生成、内部入口は残す」（原文は変更履歴に記録）。

**未済の検査**：C422 (ツリーテーブルポインタが相対論理アドレスか絶対ディスクオフセットか)；C423 (ツリーテーブルが0エントリのときルートスロットポインタに何を埋めるか)；C424 (ツリーテーブルユニットの初期容量上限と拡張時断裂)；C426 (システム構成の3段階可変性が列挙登録表に未反映)。

#### 確定項目 10：ゾーン書き込みポインタをリカバリ判定の権威上限とできない

**確定内容**：**ゾーン書き込みポインタ（WP）と本プロジェクトが認めるコミット点との間には必然的な代数関係が存在しない**。したがってリカバリ判定において WP を「コミット点は最大でもここまでしか進んでいない」という権威上限とみなすことは禁じられる。

**射程**：確定項目 10 は「WP = リカバリの権威上限」という単一の用法のみを排除し、**WP がリカバリにおいて一切の位置を占めないことを断定するものではない**——「アドレッシングのヒントとしてのみ機能」という項目は未確定項目 6 において依然として未決定である。また**実際のクラッシュセマンティクスに回答するものでもない**：`zone_full` が作成するのは静的な最終状態であり、書き込みの途中でのクラッシュは依然としてクラッシュポイント再生 harness を必要とする。⚠️ **確定項目 10 を引用する際は、この射程も併せて引用しなければならない**——タイトル前半のみを引用すると「WP はリカバリと無関係である」と読めてしまうが、それは確定項目 10 が述べたことではない。確定項目 10 と確定項目 11 はいずれも排除型の条項であり、いかなるディスク上のバイトも保証しない。

**根拠**：

- E125 (シーケンシャルゾーンの書き込みポインタ上方で何が読めるか)：`zone_full=1` という単一のデバイスパラメータによって「本プロジェクトが認めるコミットバイト数は 0 であるが、ゾーン書き込みポインタはすでにゾーン末尾に達している」状態が作成される。
- 外部ドキュメント（本プロジェクトでは未検証）：ローカル `~/code/fs-refs/linux-6.17/Documentation/filesystems/zonefs.rst` 236–242 行に「the partial failure of a multi-BIO large write operation will cause the zone write pointer to advance partially, even though the entire write operation will be reported as failed to the user」と明記——通常の異常経路で発生し得る事象であり、クラッシュを待つまでもない。
- サードパーティ：未確定項目 6 第1ラウンドの3アームすべてが「リカバリの権威上限とする」アームを一致して除外判定（`d22-item6-local-output.md`、`d22-item6-forward-sonnet-output.md`、`d22-item6-reverse-opus-output.md`）、第2ラウンドで根拠を補強。

**未済の検査**：なし。

#### 確定項目 11：ゾーン書き込みポインタは O2 (独立パーサー + checker) の判定入力に入らない

**確定内容**：D13 (検証ルート)「O2 (独立パーサー + checker) は**単一の**イメージを判定する」という決定事項（**全文**）：

> **確定内容：O2 (独立パーサー + checker) の定義域は単一のイメージである**——単項述語 `P(イメージ) → 成立/不成立` であり、
> 集合の要素判定のみを行い、アイデンティティは決して判定しない。**第2の入力（別のイメージ、または一連の記録ストリーム）を必要とするいかなる検査もこれに属さない。**

ゾーン書き込みポインタはデバイスが保守する帯域外状態であり、**イメージ内には存在しない**；これを checker に接続することは、O2 (独立パーサー + checker) に定義域外の第2入力を与えることに等しい——そして末尾の一文こそがこのステップを完全に遮断する根幹である。⚠️ **引用時は全文を引用しなければならない**：読点で区切り「単項述語」までしか引用しない場合、「第2の入力を必要とする……これに属さない」という荷重部分が脱落してしまう。

**射程**：確定項目 11 は「WP が O2 (独立パーサー + checker) に入ること」のみを排除し、**実行時 / マウント時の相互検査を排除するものではない**——それらは異なる検査主体であり、リポジトリ内では未評価である。

**根拠**：

- 実験なし：これは型の論証（単項述語は第2入力を受け入れられない）であり、実験によって点数を増減させることはできない。
- サードパーティ：未確定項目 6 第1ラウンドの3アームすべてが「checker に入れてクロスバリデーションを行う」アームの checker 形態を一致して除外判定（出力ファイルは確定項目 10 と同一）。

**未済の検査**：なし。

#### 確定項目 12：システム構成は独自のツリーテーブルユニットポインタを持たない

**確定内容**：**ツリーテーブルの権威はルートレコードに存在する**（確定項目 7「**ルートレコードは恒常的に間接層を経由する：固定ヘッダ +「ツリーテーブルユニット」を指すポインタ1本であり、ツリールートはすべてツリーテーブルユニットに格納し、ルートスロットには平坦展開しない。**」）、**システム構成はツリーテーブルユニットを指す第2のポインタを持たない**。

**射程**：確定項目 12 はシステム構成が**ツリーテーブルユニットを指すポインタ**を持たないことのみを述べる。ツリーテーブルユニット自身の形態を変更するものではなく、「システム構成にどのフィールドを含めるべきか」（確定項目 9）に回答するものでもなく、C212 (固定構造の書き込みがio_min未満) を解決するものでもない。これに連動する3項目の空席の検査は、いずれも「確定項目 12 が誤っている」のではなく、「確定項目 12 のもとでこれらの枠が空白となった」ことを意味する：C246 (树表单元没有冻结层归属)、C247 (ルートスロットフィールド表の幅算術)、C248 (ツリーテーブルユニットのブートストラップ免除未記載)。ツリーテーブル長を暗号文内に移動した後に、平文の漏洩面が増加したか減少したかは未計測である。

**根拠**：互いに独立した2つの逆方向の根拠があり、いずれか1つのみでも「システム構成に持たせる」案を否定するのに十分である——

- **パブリッシュ順序にシステム構成のステップが存在しない**：D16 (パブリッシュセマンティクス) 確定項目 7 の永続化順序は完結したシーケンスであり、システム構成スロットの書き込みはパブリッシュシーケンス**完了後**の最終ステップであり、クラッシュポイント再生の列挙には入るが順序ポイントは追加しない；システム構成に世代ごとに変化するツリーテーブルポインタを与えると、その順序に第4の順序ポイントを追加することになり、再審理を要するユーザー確定事項となる。
- **ロールバックセマンティクス**：システム構成はルートとともにロールバックしないが、ツリーテーブルポインタはルートに従う量である ⇒ システム構成内に存在するツリーテーブルポインタは、ロールバック後に別のタイムラインのユニットを指すことになる。
- 傍証（メイン agent の実調査により確認）：ツリーテーブルユニット自身も暗号文である（D9 (暗号化)「システム構成を除きすべて暗号化」）、鍵を持たない側に平文ポインタを与えてもエントリを読み出すことはできず、D9 (暗号化) 確定項目 5 は「scrub のみを実施」を採用しており、同節にこの方針が「将来の無鍵読み取り専用マウントのサポート」を完全に排除することが明記されている。
- 同型の先例：「この量をシステム構成に置くかルートレコードに置くか」は本プロジェクトで一度判定されており（ツリー ID 水位、決定変更履歴 2026-09-06（その三十六））、ルートレコードに配置すると裁定された。
- 実験なし：パブリッシュ順序とロールバックセマンティクスから導かれた構造論証であり、定量的な量は存在しない；サードパーティ：逆方向検証アームは「不要」という傾向を覆すよう明確に求められたが、2回の独立サンプリングによる10経路のいずれでも反証を構成できなかった（`system_configuration-treetable-reverse ラウンドのアーム報告（成果物は git 内）`、`system_configuration-treetable-reverse-opus-s2-output.md`）、`.claude/rules/three-way-inference.md` の「反証なし」に対する2回サンプリング要求を満たしている。

**未済の検査**：C246 (树表单元没有冻结层归属)；C247 (ルートスロットフィールド表の幅算術)；C248 (ツリーテーブルユニットのブートストラップ免除未記載)。

#### 確定項目 13：ディスク上の固定長フィールドは一律リトルエンディアン、列挙値は各登録表に配置、ビットマップビット順序を統一

**確定内容**：ディスク上のすべての固定長マルチバイトフィールド（システム構成、ルートレコード、ツリーテーブルエントリ、ユニットヘッダ、journal レコードヘッダ、位置エントリ、各種 key / value）は一律リトルエンディアンとする。列挙値は単一の総合表に集約せず、各自の登録表に配置する：ユニットタイプコードは D18 (ブロックに含める情報) 確定項目 11 のユニットクラス登録表、ツリーの種類コードは D8 (コアインデックス構造) 確定項目 9、feature bit のビット番号は D15 (フォーマット凍結ポリシー) 確定項目 4、暗号化タイプの値は D9 (暗号化) の暗号化導入時に決定、livelist イベントタイプの値は D6 (スナップショット実現モデル) 確定項目 2（ALLOC / FREE）。ビットマップのビット順序：feature bit およびその他のビットマップのビット n は、第 n div 8 バイトの第 n mod 8 下位ビットに格納する（確定項目 17）。

**射程**：ビッグエンディアンは採用しない：E142 (新規プールでのファイル作成のドライラン) 装置および `crates/singlefs-core/src/bytes.rs`（`to_le_bytes`）はいずれもリトルエンディアンで記述されており、ビッグエンディアンへの変更はすべての put および成果物の書き直しを伴う。その他の登録表は確定項目 17 参照。

**根拠**：

- 実験なし：エンディアンと登録表は取り決めであり、定量的な量は存在しない；リトルエンディアンの採用は既存の装置および実装がいずれもリトルエンディアンで書かれているためである。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 14：devs < R のとき I-7.6 (ルートリング領域が互いに異なるディスクに配置) は何を判定するか

**確定内容**：I-7.6 (ルートリング領域が互いに異なるディスクに配置) を以下のように改定する：R 個の領域のデバイス識別子は min(R, devs) 個の異なる値を網羅し、かついずれのディスクも ⌈R / devs⌉ 個を超える領域を担持しない；devs < R のときマウントを拒絶せず、フォールトトレランス保証はディスク数に応じて縮退する（2台で R = 3 の場合：ディスク 1 喪失時は最悪 1 世代、ディスク 0 喪失時は最悪 2 世代）。R = 3 は維持する。

**射程**：バイト列は変更せず、不変条件と checker のみを変更する；I-7.6 (ルートリング領域が互いに異なるディスクに配置) はプールレベル checker が判定する。

**根拠**：

- E87 (固定構造の配置)：2台で R = 3 の場合、ディスク 1 喪失時の最悪ロールバックは 1 世代、ディスク 0 喪失時は最悪 2 世代——2台で R = 3 では必ず1台が2つの領域を担持することになる。
- ユーザー確定（F = 2 は確定項目 2 参照；[records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に沿って個別に決定）、原文は変更履歴に記録；サードパーティ論証を経ておらず、メイン agent の推薦およびユーザーの選択はいずれも反証可能である。

**未済の検査**：なし。

#### 確定項目 15：システム構成フィールド表の4項目連動

**確定内容**：初期版では間接ディレクトリユニットポインタを残さない（明示的な空欄であり予約枠ではない）、デバイス記述子テーブルは初期版では書き込まない、D12 (対象メディア) 確定項目 5 の文言を「第1のラインは2ディスク同種構成であり初期版では書き込まない；第2のレイアウトを追加する前に書き込み必須」に改定し、D12 (対象メディア) 確定項目 4 の容量 / レイアウトビット / physical_block_size の3項目の突合が初期版では各ディスク間の相互比較に退化することを明記；`devs` に 4 バイトを確保；整理3水域の 24 バイトは恒常的に 0 = 組み込みデフォルトとする。確定項目 9 のフィールド表はこれに伴い変更：59 を削除し、「ユニット領域開始スロット番号」8（D3 (領域割り当て) 確定項目 10）、「mkfs 時の io_min」4、および「固定構造スロット間隔」4（D2 (RAID ストライプ方針) 確定項目 19）を追加する。

**射程**：これら4項目のみを規定する；システム構成のその他のフィールドは確定項目 9 参照。

**根拠**：

- E147 (システム構成フィールド表の4項目連動)：間接ディレクトリポインタを残さないバリアントは単一セクター内に収まる。
- 当該ポインタの 59 バイトは出典がなく、D19 (ブロックポインタ構造と幅予算) 確定項目 7 / 8 のポインタ幅に基づく計算では 83 を要する。
- ユーザー確定（[records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に沿って個別に決定）、原文は変更履歴に記録；サードパーティ論証を経ておらず、メイン agent の推薦およびユーザーの選択はいずれも反証可能である。

**未済の検査**：C218 (ルートリングスロット番号とシステム構成スロット番号のアドレス空間)；C180 (ルートスロット幅と物理ブロックサイズの不一致)。

#### 確定項目 16：システム構成スロットローテーション規則；ルートリング領域計算式のアドレス空間と領域内スロット位置

**確定内容**：システム構成スロット：各ディスク恒常的に2スロット、スロット i のデバイス内オフセット = i × 固定構造スロット間隔（D2 (RAID ストライプ方針) 確定項目 19）、スロット世代番号は 1 から開始（mkfs で書き込まれた両スロットの世代番号はいずれも 1）、書き込みごとに +1、次回書き込みスロット = 世代番号 mod 2、スロット選択はチェックサム通過かつ世代番号最大のものを選択（ディスクごとに計算：番号取得、パブリッシュ末尾のローテーション、番号取得の全か無かの失敗時のロールバックの3種のシステム構成書き込みはいずれも当該ディスク上で自己証明を通過したスロットの最大世代番号 + 1 を書き込む）。スロット 1 を読み出す際のオフセットは以下の順序で取得する：自ディスクのスロット 0 が利用可能（自己証明を通過し、幾何構造フィールドが当該バージョンの読者の許容範囲内にある）な場合はそれが自述するスロット間隔を取得；それ以外の場合はプール内でデバイス順で最初の利用可能なスロット 0 が自述するスロット間隔を取得；プール全体で利用可能なシステム構成スロット 0 が存在しない場合、読者はフォーマットで許容される各スロット間隔ごとに1スロットずつ読み出し（4096 以上、512 の整数倍、同一読者が受け入れるルートリング起点マイナス 4096 を超えない）、自己証明を通過し、かつ自述する固定構造スロット間隔が所在オフセットに等しいスロットのみを受け入れる、複数の段階で受け入れ可能な場合は世代番号が最大のスロットを選択、世代番号が同一の場合はオフセットが小さい方を選択；段階探索中に自己証明を通過したもののプールレベルフィールドが範囲外であるスロットを読み出した場合はプール全体を拒否（第5文「逸脱即拒絶」）、incompat ビットが未認識のスロットは選択不可とし、オフセットの検証は行わない。段階探索中に自己証明を通過したスロットを読み出した場合、まずそれが当該バージョンの読者が受け入れない値を含んでいるか判定（I-8.1 (リングジオメトリが十分大きい) の3つの journal リング幾何構造フィールドを含む）、含んでいる場合はプール全体を拒否しオフセットの検証は行わない；含んでいない場合に自述スロット間隔が所在オフセットと等しいかを検証する。ルートリング：`r × P × chunk` はデバイス内オフセットであり、基底アドレス = システム構成の「ルートリング起点」フィールド（初期版 1 MiB）、領域 r の起点 = 基底アドレス + r × P × chunk；領域内スロット位置 = `(txg div R) mod S`、ステートレスであり、txg と (R, S) のみによって決定され、確定項目 2 のパラメータ表に昇格。以下の5文を厳格に固定する：

1. **システム構成の「ルートリング起点」フィールドは 16 KiB スロット番号である**、D3 (領域割り当て) 確定項目 7 の配置スロット番号、D19 (ブロックポインタ構造と幅予算) 確定項目 4 の位置エントリの物理オフセットと同一の座標系；初期版では **64**（= 1 MiB ÷ 16384）を書き込む。基底アドレス（デバイス内バイトオフセット）= 当該フィールド × 16384；領域 r の起点 = 基底アドレス + r × P × chunk。⚠️ フィールドに書き込まれるのは 64 であり、1048576 ではない。読者がスロットを解釈する際、このフィールドと journal リング起点フィールド（オフセット 325）を読み出し、初期版定数（64、1024）と不一致の場合は一律プール全体を拒絶する；ルートリング領域数 R（オフセット 361 の 1 バイト）も同様に読み出し、3 でなければプール全体を拒絶、領域ごとのスロット数 S が範囲外なら確定項目 2 に従いプール全体を拒絶；あるスロットの incompat ビットが未知でかつフォーマットバージョンも 1 でない場合は、先にフォーマットバージョンでプール全体を拒絶し、選択不可とは扱わない（C663 (システム構成スロット範囲外と不認領値で二読者の分類が異なる)、ユーザー 2026-10-20 確定、原文は変更履歴に記録；core は `crates/singlefs-core/src/recovery.rs` の `RootRingRegionCountNotTheFirstVersionValue` と `mountable_slot_or_refusal`、checker は `crates/singlefs-checker/src/image.rs` の `value_refused_by_this_reader`）；固定構造スロット間隔の上限の判定には同一スロットが自述するルートリング起点を用い、checker の I-7.13 (システム構成のプールレベルフィールドが読者の許容範囲内にある) と core は同一の表を受け入れる（ユーザー確定、発端と根拠はコードサードパーティ第2ラウンド裁定 `research/prompts/m2-closeout-code-r2-main-verification.md`「core と checker におけるシステム構成幾何構造フィールドの解釈」の枠）。実装済み：core の `crates/singlefs-core/src/system_configuration.rs` `parse_slot` はオフセット 325、371 を `SystemImmutableConfiguration` の `journal_ring_start_slot`、`root_ring_base_slot` に読み込む；`crates/singlefs-core/src/recovery.rs` の `system_configuration_values_this_reader_accepts` はシステム構成選択時に判定し、不一致時は `SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot` / `JournalRingStartNotTheFirstVersionSlot` を報告、スロット間隔の上限は `largest_fixed_structure_slot_spacing_under_the_root_ring_base`（ルートリング起点 × 16384 − 4096；段階探索でスロット 1 を探す箇所では読み出し可能なスロットがまだ存在しないため、依然として初期版定数を使用）；checker の `crates/singlefs-checker/src/image.rs` `geometry_of` は同一の表を判定（325、371 は初期版定数と等しく、417 = リング長から現計算された起点かつ 64 スロット区画境界上にある）。テストケース：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` の `a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write`（攻撃側5枠の読み取り専用および書き込み可能マウントの双方が拒否、ディスク上は不変）；`crates/singlefs-harness/tests/checker_known_bad_images.rs` の `each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`；同一イメージ上で双方を判定するのは `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs`（5枠各1件：core の両マウントがプール全体を拒絶、ディスク上は不変、checker は I-7.13 (システム構成のプールレベルフィールドが読者の許容範囲内にある) を報告し拒否したメンバーを名指し；ルートリング起点に 0 を書き込む枠では双方が報告するメンバーが異なる——core は先にスロット間隔上限を判定しスロット間隔逸脱を報告、checker は先に2つの起点を判定しルートリング起点を報告）（`m2-impl-r2-fixes-b-implementer-report.md` 第3節第2行、`m2-impl-r2-fixes-a-implementer-report.md`「結論」第1行）。
2. **ルートスロット番号 j のデバイス内オフセット = 領域起点 + j × 固定構造スロット間隔**（スロット間隔 = max(4096, mkfs 時にプローブされた io_min)、D2 (RAID ストライプ方針) 確定項目 19）。確定項目 2 の文面通りにスロット間隔 512 を取ることも規約上は合致するが、バイト表は新規プールでのファイル作成のルートが 1 MiB + 4096 に配置されることを要求しており、この一文は番号からオフセットへの変換を厳密に固定する。
3. **P = 3、chunk = 1 MiB は初期版の mkfs 定数である**；mkfs は `基底アドレス + (R−1) × P × chunk + S × スロット間隔 ≤ デバイス利用可能バイト数` を検証し、範囲外の場合は mkfs を拒否しなければならない。3 を採用するのは初期版の2ディスク構成において「P は素数かつ P > devs」を満たす最小の素数であるためであり、chunk に 1 MiB を採用するのは基底アドレスと同スケールとするためである ⇒ 3つの領域の起点は 1 / 4 / 7 MiB となり、[layout/01-first-txn.md](../layout/01-first-txn.md) のバイト表と一致する。⚠️ **「P > devs」という制約にはもはや理由が存在しない**（その唯一の理由は `gcd(P, devs) = 1` による全単射であったが、D2 (RAID ストライプ方針) 確定項目 7 が所属を mkfs 時に領域ごとに書き込むデバイス識別子に変更したことにより破棄された）、負債は C217 (Pがデバイス数超の制約理由消滅と制約残存) にある；3 の選択は当該制約に依存しない。
4. **ルートリングの3領域のデバイス所属は初期版では 0 / 1 / 0 とハードコードする**（領域 0 → ディスク 0、領域 1 → ディスク 1、領域 2 → ディスク 0）、mkfs は領域ごとにデバイス識別子をシステム構成の「ルートリング領域別デバイス識別子」12 バイトに書き込む（D2 (RAID ストライプ方針) 確定項目 7）。⚠️ **これは mkfs パラメータではない**：初期版 devs = 2、R = 3 において、D22 (ユニット原子性の合成手法) 確定項目 14 の鳩の巣下限（min(R, devs) = 2 個の異なる値を網羅し、いずれのディスクも ⌈3/2⌉ = 2 個を超える領域を背負わない）に従うと、「2領域が一方のディスク、1領域が他方のディスク」という単一の形状のみが存在し、領域番号を固定した後は 0 / 1 / 0 が唯一の標準的記述となる。これを mkfs パラメータとした場合に得られる自由度は「どちらのディスクが2領域を背負うか」という1点のみであり、2ディスク同種構成においては観測可能な差異がなく、マウント時に検証しディスク追加時に再計算すべきパラメータが1つ増えるという代償を払うことになる。
5. **ウォームアップの空パブリッシュ 2 回、新規プールでのファイル作成 txg 3 はフォーマット定数である**、実行時述語ではない（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、NEW_POOL_FILE_CREATION_TXG = 3、[layout/01-first-txn.md](../layout/01-first-txn.md) 八に登録）。D16 (パブリッシュセマンティクス) 確定項目 8 は「自インスタンスのルートが2台のディスクをカバーするまで推進する」ことを要求しており、0 / 1 / 0 の所属のもとでは txg 1 は領域 1（ディスク 1）、txg 2 は領域 2（ディスク 0）に配置され、2回の空パブリッシュで正確に2台のディスクがカバーされる ⇒ 回数は所属によって一意に決定され、定数として記述することで実行時述語とするよりも取り得る値が1つしかない分岐を1つ削減できる。⚠️ 所属を変更した場合、これら2つの定数は再計算を要する（ディスク追加時またはストライプ表導入時）。`NEW_POOL_FILE_CREATION_TXG` は mkfs シーケンス（mkfs → 番号取得 → ウォームアップ2回 → 新規プールでのファイル作成、同一プロセス内の一連の処理）のみを管轄し、任意のプールにおける最初のファイルバージョンを管轄するものではない：mkfs のみを実施したプールを再開した後に最初のファイルバージョンを書き込む場合、txg は現行ルート + 1 を採用する。読者による固定構造スロット間隔の判定：≥ 4096、かつスロット 1 全体が同一スロットが自述するルートリング起点より前に位置すること；`physical_block_size` ∈ [ルートレコード幅 466, スロット間隔]。逸脱即拒絶：checker は範囲外のスロットに対して I-7.13 (システム構成のプールレベルフィールドが読者の許容範囲内にある) を報告し、当該プールの他の不変条件は適用対象外と報告する、実装はプール全体のマウントを拒絶する（ユーザー 2026-09-27 確定「プール全体拒絶」）。

**射程**：領域計算式を「mkfs 時に領域ごとの起点を書き込む」方式に変更することは、zoned ライン着工前に改めてサードパーティ検証を行う。プール全体で利用可能なスロット 0 が存在しない場合に初期版定数 4096 に従ってスロット 1 を読み出すという解釈（ユーザー 2026-09-14 確定の「甲」）はユーザーにより 2026-10-02 に破棄され、段階探索に変更された、原文は変更履歴に記録；「世代番号が同一の場合はオフセットが小さい方を選択」という規定は、mkfs による [0, ルートリング基底アドレス) のゼロクリア（確定項目 8 第3条）以降、メディア破損によって生じる残留物に対してのみ対象が存在する。プールレベル checker、リカバリ、および GPU 判定器の2実装は同一の規定に従って読み出す（[layout/01-first-txn.md](../layout/01-first-txn.md) 一「システム構成スロット 0 およびスロット 1 の位置」の段落）。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：この選定方針に従って新規プールでのファイル作成におけるすべての固定構造バイトを書き出し（システム構成スロット 0 / 4096、ルートリング 1 / 4 / 7 MiB、新規プールでのファイル作成のルートは領域 0 スロット 1 に配置）、`crates/` の実装と21領域にわたって突合した結果完全に一致（ルートレコード、journal レコード、システム構成をバイト単位で完全突合）。
- E87 (固定構造の配置)：2ディスク上で R = 3 では必ず1台が2つの領域を担持する——0 / 1 / 0 が唯一の標準的記述であることの根拠の半分。
- ディスクごとの計算方式：C322 (インスタンス番号取得ステップのバリア配置条項なし) 3ラウンドのサードパーティ検証後のユーザー確定（プール全体の最大値 + 1 を取ると、遅延ディスクが一度に 2 ジャンプし、自身が選択したスロットに戻ってしまう問題が生じる）、裁定は `research/prompts/c322-r3-main-verification.md`。
- ユーザー確定（[records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に沿って個別に決定；第4・5文は事後のポップアップ確定）、原文は変更履歴に記録；ディスクごとの計算の一文を除きサードパーティ論証を経ておらず、反証可能である。
- ユーザー確定 2026-09-23：`NEW_POOL_FILE_CREATION_TXG` は mkfs シーケンスのみを管轄する（`research/prompts/m2-refusals-presumed-r1-main-verification.md` 第3節で射程を判定した枠）、原文は変更履歴に記録。
- ユーザー確定 2026-10-01：「mkfs 時の両スロット世代番号はいずれも 1」を本項目に明記、I-7.16 (システム構成スロット世代番号とスロット番号の関係) も併せて判定（`research/prompts/m3-inv-b-r1-main-verification.md` 第4節第1条）、原文は変更履歴に記録。
- プール全体で利用可能なスロット 0 が存在しない場合の段階探索、同点時の選定方法および2つの出口：サードパーティ裁定 `research/prompts/m3-c651-r1-main-verification.md` 第2節（定数読み出しとプール全体拒絶の両案は除外、段階探索読み出し案および同点時選定案が成立）および第4節第1・2条、`research/prompts/m3-formal-r6-main-verification.md` 第2節条項文の2つの出口の枠；テストケース `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`（リカバリとプールレベル checker が同一イメージ上で段階探索を判定、自述スロット間隔がオフセットと不一致のものは不採用、最大世代番号を選択、同番号は小オフセットを選択、逸脱時はプール全体拒否、未認識の incompat は選択不可）。
- 段階探索で先に非許容値を判定し、その後にオフセットを検証：コードラウンド裁定 `research/prompts/m3-formal-code-r2-main-verification.md` 第2節「K3-a」行（改訂前の checker は先にオフセットを検証したため、脱落したスロットに対して I-8.1 (リングジオメトリが十分大きい) を判定せず実装と乖離；第13バッチで実装と同順序に変更）および `research/prompts/m3-formal-code-r3-main-verification.md` 第2節「K3」行（弁護側 Z5：条項にこの一文の文言が欠落）。
- ユーザー確定 2026-10-02：段階探索を前2ステップおよび同点選定法とともに書き戻し；プール全体で利用可能なスロット 0 が存在しない場合は段階探索に変更、2026-09-14 の最終ポップアップ確定の「甲」（最小スロット間隔 4096 でスロット 1 を読み出す）を破棄、原文は変更履歴に記録。
- ルートリング領域数 R を読み出してプール全体拒絶、領域ごとのスロット数 S が範囲外ならプール全体拒絶、incompat 未知とフォーマットバージョン非 1 が同時にある場合はフォーマットバージョンを先に：ユーザー確定 2026-10-20（C663 (システム構成スロット範囲外と不認領値で二読者の分類が異なる)）、原文は変更履歴に記録。

**未済の検査**：C217 (Pがデバイス数超の制約理由消滅と制約残存)。

#### 確定項目 17：確定項目 13 の登録表一覧の補完

**確定内容**：journal レコードタイプの登録表は D23 (ジャーナルの役割と形式) 確定項目 1（0 無効、1 通常レコード）；統計量ラベルは D5 (スナップショット / 空間会計メカニズム) 確定項目 10；アルゴリズムタイプは D9 (暗号化)（0 暗号化なし、1 ChaCha20-Poly1305、2 AES-256-GCM）；ツリーテーブルエントリ / ユニットヘッダの flags の各条項には「初期版では常に 0、非 0 は拒否、ビット番号表は空」と明記、ルートレコードの flags ビット 0 はアンマウントマーク、その他のビットも同様に常に 0（確定項目 7）；ビットマップのビット順序（ビット n は第 n div 8 バイトの第 n mod 8 下位ビットに格納）を確定項目 13 に反映。

**射程**：取り得る値の配置先を登録するのみ；新規プールでのファイル作成で書き出される値は不変。

**根拠**：

- 実験なし：登録作業であり、代替の選択肢は存在しない；アルゴリズムタイプの2つの非ゼロコードは D9 (暗号化) 確定項目 3「両アルゴリズムとも実装必須」の順序に従って付与。
- ユーザー確定（[records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に沿って個別に決定）、原文は変更履歴に記録；サードパーティ論証を経ておらず、反証可能である。

**未済の検査**：なし。

#### 確定項目 18：境界線

**確定内容**：**境界線：ユニットがディスクに永続化された後、そのチェックサムを担持する親ポインタが存在してからパブリッシュされるか否か。** YES ⇒ COW + D4 (チェックサム位置) で十分であり、断裂書き込みの害はない（確定項目 19、確定項目 20）；NO ⇒ 自己証明が必須であり、自己証明には2つの要素が同時に存在しなければならない（確定項目 21）。

**射程**：分類の対象はユニットである。親ポインタを持つ側は通常ユニット；持たない側はルート、システム構成、journal である（確定項目 23、確定項目 24）。境界線を上位層へ移動させることはしない（確定項目 22）。

**根拠**：

- 実験なし：境界線は「チェックサム付き親ポインタの有無」に基づく構造論証である；断裂形態はローカルマシン上で作成不能（確定項目 19）。
- サードパーティ論証（メイン agent 裁定）にアーム状況記録がない：`research/prompts/` 配下に対応するプロンプトとローカルアーム出力が存在せず、`.claude/rules/three-way-inference.md` は「ローカルアーム欠席時は明示的に報告しなければならない」と明記 ⇒「3アームすべてが反例を探索し見出せなかった」という記述は**アーム状況未記録の陰性結果**に降格され、新たな結論を支持するために利用する場合は事前に追試を実行しなければならない。

**未済の検査**：なし。

#### 確定項目 19：親ポインタを持つ通常ユニットは COW + D4 (チェックサム位置) で十分であり、いかなるデバイス原子性仮定にも依存しない

**確定内容**：メカニズム：チェックサムは**別のユニット内**に保持され（D4 (チェックサム位置)）、親は子が永続化された後に初めてパブリッシュされる（D16 (パブリッシュセマンティクス)）。子ブロックの任意のバイト部分集合が旧 / 新 / ゼロ / ゴミのいずれであっても、親のチェックサムと不一致となる。⇒ **断裂粒度が 512、4 KiB、またはブロック全体であるかは、この論証に一切影響を与えない。** **これこそが「ブロック内自己完結チェックサム」に対する D4 (チェックサム位置) の実質的な優位性である——バイト数の節約ではなく、検出能力をデバイスの仮定から完全に切り離したことにある。**

**射程**：⚠️ **D22 (ユニット原子性の合成手法) 確定項目 19 が回答するのは断裂が検出可能か否かであり、クラッシュ後にフォールバック可能な対象が存在するか否かではない。** 本項目が証明するのは「子ブロックの任意のバイト部分集合が親のチェックサムと不一致となる」ことであり、このステップはデバイスの原子性に依存しない；一方、**旧バージョンがディスク上に現存しているか否かは別の条項である**——D22 (ユニット原子性の合成手法) 確定項目 4（「本トランザクション内で解放されたブロックは再利用禁止」I-7.4 (直近 K 代ブロック非再利用) は存在するか）が管轄するのはまさにその後半であり、これを欠いた場合の障害モードは文面通り「**検出可能（親ポインタのチェックサム不一致）、リカバリ不能**」である。⇒ **D22 (ユニット原子性の合成手法) 確定項目 19 を引用する際は、この文言も併せて引用しなければならない**：タイトル前半の「COW + D4 (チェックサム位置) で十分」のみを引用すると、ロールバック候補に関する後半が完全に脱落してしまい、それは `.claude/rules/three-way-inference.md`「kb 内の項目を引用する際は行全体を転記する」に記録されている引用事故そのものとなる。

**根拠**：

- 実験なし：断裂形態はローカルマシン上で作成不能——`dm-flakey` は `drop_writes` / `error_writes` / `corrupt_bio_byte` のみを提供し、「前半は永続化され後半は未永続化」という状態を作成できない；クラッシュポイント再生は断裂状態を「未永続化」に統合し、断裂の部分集合を列挙しない（D13 (検証ルート) 確定項目 4）。
- サードパーティ論証にアーム状況記録がない（確定項目 18）。
- 現役の参照例（文面通り確認、本プロジェクトでは未検証）：btrfs の `struct btrfs_key_ptr { btrfs_disk_key key; __le64 blockptr; __le64 generation; }` はチェックサムを持たず、csum はノードヘッダ内に存在するため、自己参照チェックサムでは「このブロックが現世代のものか」「ある世代の正当なブロックか」を区別できず、btrfs は別途 transid による突き合わせを余儀なくされている；ZFS の `blkptr_t` は `blk_cksum`（256ビット）を持ち、通常ブロックに第2の関門は存在しない ⇒ **両者の相違はまさに「チェックサムをどこに置くか」という単一の変数に集約される；D4 (チェックサム位置) は1つの仕組みで btrfs の2重の関門に匹敵する能力を獲得している。**

**未済の検査**：C16 (パイプラインが障害モデル例外を宣言)：断裂注入は dm-flakey に依存できず、クラッシュポイント再生 harness 内で書き込みリクエストを切り詰めることで作成しなければならない。

#### 確定項目 20：断裂粒度の拡大は問題を悪化させるのではなく、消失書き込みへと退化させる

**確定内容**：

| 断裂結果 | 親ポインタチェックサム (D4 (チェックサム位置)) | ブロック内自己完結チェックサム |
|---|---|---|
| 全て新規 | 通過（正常） | 通過（正常） |
| **混合** | 捕捉可能 | 捕捉可能 |
| **全て旧データ** | **捕捉可能**（親には新チェックサムが存在） | **捕捉不能**——これは消失書き込みであり、D18 (ブロックに含める情報) で対処不能と判定済み |

**射程**：検出能力の比較のみを対象とする。これは D18 (ブロックに含める情報) の直接の系である：断裂粒度が拡大すると消失書き込みに退化し、消失書き込みはブロック内に何を含めようとも捕捉できない。

**根拠**：

- 実験なし：3行はチェックサムの配置場所から導出された真理値表であり、断裂状態はローカルマシン上で作成不能（確定項目 19）。

**未済の検査**：C16 (パイプラインが障害モデル例外を宣言)。

#### 確定項目 21：インプレース上書き構造には「ユニット全体のチェックサム + 実際に検証される世代番号」が必須であり、両者不可欠

**確定内容**：チェックサムのみでは不十分である。**完全にディスクに書き込まれなかった**レコードを読み出した場合、同一位置に残存する前周の**完全で正当な旧レコード**が取得される——チェックサムは通過するが、それは陳腐化している。⇒ チェックサムは「このユニットが完全か否か」に答え、世代番号は「それが現世代のものか否か」に答える、**2つの問いに2つのフィールドで対応する。** 本プロジェクトでは XFS のような per-sector スタンプ（軸②）は採用しない。

**射程**：

<!-- doc-registries:not-field-frequency 否決した XFS のセクタ単位スタンプの話であり、このフォーマットのフィールドではない -->
- ⚠️ **「XFS のような per-sector スタンプが必要か」は直交する2本の軸であり、単一の問題ではない**：軸① はシーケンス番号の**同一性**（周回数 cycle かトランザクション番号 txg か；XFS の LSN は `(cycle:32, block:32)` であり、周回数はシーケンス番号と位置を同一の 64 ビットに圧縮した産物）、これはシーケンス番号のビット幅に関連する；軸② はシーケンス番号の**粒度**（レコードごとに1つか、512 バイトごとに1つか）、これはシーケンス番号のビット幅とは**無関係**である。**「txg で cycle を置き換える」ことは軸①のみを変更し、軸②は一切変更しない。**
- 軸②を採用しない理由は「2⁻³² は十分に小さく、64 ビットチェックサムに変更すればより低コストで 2⁻⁶⁴ に抑制できる」ためであり、**「txg があるから cycle は不要」ではない**——後者はカテゴリー錯誤である。この否決理由の比較対象集合は不完全である：スタンプはアライメント免除の利点ももたらす（判定分解能がセクターである場合、複数レコードが同一の原子ユニット内に混在しても断裂を判定可能）、アライメントのコストは E23 (ジャーナル幾何) で計測済みである。**その場での再審理は行わない**が、以下の2つの前提が満たされるまで「軸②不採用」が十分に論証されたとみなしてはならない：① 対象負荷の fsync 粒度——D25 (対象ワークロードの優先順位) は粗粒度を採用済みであり、「軸②不採用」に有利な方向であり、本条件は満たされている；② クラッシュポイント再生の実測により断裂境界が実際にセクター倍数に落ちるか否かの検証（「デバイスの原子幅 512 ⇒ 各セクターが独立して新/旧となる」はこの点に依存するが、未実測である）。
- ⚠️ **方法論的注記**：断裂形態に関する推論は**ローカルマシン上では反証不可能**であり、当該領域全体は結論ではなく推論として記録される；反証にはクラッシュポイント再生 harness が断裂状態を作成できるようになるのを待つ必要がある。⚠️ **射程は断裂形態であり、未確定項目 6 は含まない**（書き込みポインタがコミット点に先行することは断裂ではない）。

**根拠**：

- 実験なし：「永続化されなかったレコードから前周の正当な旧レコードが読み出される」ことはインプレース上書き構造における構造的事実であり、断裂形態はローカルマシン上で作成不能（確定項目 19）。
- 軸② の損益：デバイスの原子幅 512 ⇒ クラッシュ後、1レコードの各セクターは独立して新/旧となり、陳腐なセクターには必ず前周のスタンプが付与されている ⇒ 確実に検出可能；ユニット全体のチェックサムのみの場合 ⇒ 検出漏れ確率 ≈ 2⁻ⁿ（n = チェックサムビット幅）。代償としてスタンプがセクター内部に存在することになる：XFS のようなシャドウバックアップ（`h_cycle_data[]`、レコード長に伴いヘッダが伸長）を行うか、512 バイトごとに 8 バイトを確保する（1.6% の浪費、可変長レコードのオフセット計算が全面的に複雑化）。XFS 自身の記述によれば断裂検出は CRC が担っており、XFS における cycle の実際の用途は二分探索による head の特定である——それは XFS のポリシーであり、構造上の制約ではない（外部事実、本プロジェクトでは未検証）。

**未済の検査**：C16 (パイプラインが障害モデル例外を宣言)：断裂注入。

#### 確定項目 22：境界線を上位層に引き上げることは不可：ルートリングのロールバック候補が無効化される

**確定内容**：**親ポインタを持つユニットをも「アドレス固定 + インプレース上書き + 自己証明」に変更すると、ルートリングが即座に無効化される。** I-7.4 (直近 K 代ブロック非再利用) は、ロールバック候補集合内のすべてのルートが参照するブロックの物理範囲が、他のオブジェクトへ再割り当てされておらず、消去もされていないことを要求する。さもなければリング内の旧ルートが指すブロックはすでに上書きされており、ロールバック候補として無効（偽）となる（確定項目 25）。**インプレース上書きとはまさに「それらのブロックがすでに上書きされている」ことそのものである** ⇒ リング内の最新世代を除くすべてのルートが偽の候補と化す。⇒ リング内の旧ルートを正当な候補として維持するには、旧バージョンのレプリカを保持しなければならず、**しかもブロック再利用の境界まで保持し続けなければならない**（`max(F_有効, リング内最古の有効ルート)`、D16 (パブリッシュセマンティクス) 確定項目 1）。その結果、インプレース方式は COW の遅延再利用空間コスト**に加えて**コピー書き込みのコストを支払うことになり、このコスト計算は**スナップショットがゼロであっても同様に成立する**——これはスナップショット機能のコストではなく、ルートリングのコストである。

**射程**：**この条項はインプレース方式の最強の形態に対して成立する**（メタデータの自己証明は確定項目 21 に従い、アドレスは初回の割り当てから不変、親ポインタは分割・マージ時にのみ変更）、したがって D4 (チェックサム位置) を維持するか否かに依存しない。⚠️ **インプレース方式が1つの軸において確実に優位であることは否定できず、存在しないものとしてはならない**：書き込み増幅の軸においてはインプレースが優位である（E38 (会計構造の COW かインプレースか) のインプレースアームは祖先を再書き込みしない）、それを退けたのはルートリングであり、書き込み量ではない。同一ラウンドで退けられた5つの理由は最強形態に適用できず、再利用してはならない：インプレースでも Merkle 脊柱全体を再書き込みする必要がある；データ側は依然として COW を要するため2重メカニズムとなる；クラッシュポイント集合の爆発；D9 (暗号化) の nonce 再利用という厳格な壁；D5 (スナップショット / 空間会計メカニズム) の birth 判定が直接無効化される（それぞれ適用できない理由は決定変更履歴の元の表を参照）。

**根拠**：

- 実験なし：I-7.4 (直近 K 代ブロック非再利用) とブロック再利用境界から導かれた構造論証であり、定量的な量は存在しない。
- サードパーティ論証（メイン agent 裁定）、リポジトリ内にはローカルアームの出力 `cow-necessity-local-en-output.md` のみが残されている。

**未済の検査**：なし。

#### 確定項目 23：journal を COW でカバーすることは不可能：固定長リングで成立、自己証明クラスに属する

**確定内容**：**固定長リングにおいて成立する**：1件の journal レコードを COW するには、それを指すポインタが必要である；そのポインタをコミットする必要がある；そしてコミットこそがまさに journal が実行すべきことである。循環が生じる。また非環状 journal において毎レコードにルートの書き込みを要する場合、**journal のすべての利益（ルート書き込みの償却）がゼロとなる**。⇒ **journal は確定項目 21 のクラスに属する**：ユニット全体のチェックサム + 実際に検証される世代番号が必要である。⚠️ これにより journal のプレフィックス規則に厳格要件が1つ追加される：**世代番号がフォーマット内に存在し、かつ再生経路において実際に検証されなければならない。**

**射程**：

- チェーン方式ログには別の出口が存在する：ZFS の ZIL は固定長リングではなく、各ブロックのトレイラーにチェーン上の次ブロックを指す `blkptr` を保持し、チェーン先頭は `zil_header_t.zh_log` に存在するため、「ポインタが前レコード内に存在する」ことによって循環が解消されている（外部事実、本プロジェクトでは未検証）；本プロジェクトでは固定長リングを採用する（確定項目 1）。
- 世代番号は `jsn` である：D23 (ジャーナルの役割と形式)「世代番号：レコード粒度の大域単調番号であり、D5 (スナップショット / 空间会計メカニズム) の txg ではない」は文面通り世代番号を `jsn` に割り当て、`checkpoint_txg` を排除している（同一ウィンドウ内で同一の txg を共有するため、先後関係を識別できない）；D23 (ジャーナルの役割と形式) 確定項目 14 のレコード水位は下限を用いており、実現とはみなされない。着地点も D16 (パブリッシュセマンティクス) 確定項目 3（journal の凍結層）ではない。
- どこまで実現されているか：`jsn` = インスタンスコード 32 ビット + カウンタ 48 ビットがフォーマット内に存在（D23 (ジャーナルの役割と形式) 確定項目 9）；再生経路 `crates/singlefs-core/src/recovery.rs` の `replay_journal` はインスタンス内カウンタの番号欠落で即座に停止し、インスタンス境界を跨ぐと停止する（D23 (ジャーナルの役割と形式) 確定項目 14 の注）；checker 側が担う I-8.3 (再生プレフィックスが厳密に連続) は未実装。

**根拠**：

- 実験なし：固定長リングにおける循環は定義上のものであり、定量的な量は存在しない。
- サードパーティ論証にアーム状況記録がない（確定項目 18）；世代番号の厳格要件がどこまで実現されているかは、1回のサードパーティ再確認（C199 (インスタンス番号インクリメントとjsn断号即停止が矛盾) の出典）で実調査された。

**未済の検査**：C199 (インスタンス番号インクリメントとjsn断号即停止が矛盾)：コードインクリメントと欠落即停止の矛盾は収束済みであり、残るはクラッシュポイント再生の2枠と I-8.3 (再生プレフィックスが厳密に連続) の checker である。

#### 確定項目 24：ルートも同一クラスに属し、2種類のレイアウトは同一のメカニズムを必要とする

**確定内容**：**ルートには親が存在しないため、COW のすべての保護を一度に失う**——親の中に存在するチェックサム、親の書き込みであるアトミックなパブリッシュ、その双方が失われる。⇒ ルートは自己証明が必須である：**ユニット全体のチェックサム（完全性の自己証明） + 世代番号（現世代であることの自己証明） + スロットローテーション（唯一のコピーを上書きしない）。**「ルートポインタをインプレース上書き可能」なクラスであっても**実際には唯一のコピーを上書きすることはできない**——1回の断裂によって唯一の目撃者が抹消されるため、固定スロット群の間でローテーションし、かつ世代番号を保持しなければならない；zoned クラスではルートがドリフトするため、「最新のルートがどこにあるか」を発見するプロシージャが必要となる。⇒ **両クラスの相違は「候補をいかに列挙するか」のみにあり、「候補がどのような形態であるか」にはない。** **読み出し順序は要件であり、実装の詳細ではない**：必ず**全候補を個別に検証した後に、正当なものの中から世代番号に従って最新を選択**しなければならない；「先に世代番号で最新を選定し、その後に検証する」ことは禁止される——後者は最新スロットが断裂した際に前世代にフォールバックできず即座に失敗する。ルート選択キー (`checkpoint_txg`, インスタンスコード) が完全に一致するスロットが複数存在する場合、ルートリングの読み出し順序において先頭にあるものを選択する；プールレベル checker のルート選択も同一の解釈に従う。

**射程**：D12 (対象メディア) の一文の解釈（「ルートポインタをインプレース上書き可能」なクラス）を覆す；D16 (パブリッシュセマンティクス) のギャップを1つに集約する：不足しているのは単一のメカニズムであり、2種類ではない。zoned クラスの発見プロシージャは一度も設計されておらず、D22 (ユニット原子性の合成手法) 未確定項目 6 に棚上げされた負債である。`crates/singlefs-core/src/recovery.rs` のルート選択は、3領域の全スロットのうち自己証明を通過し、`(checkpoint_txg, インスタンスコード)` が最大のものであり、キーが同一の場合はルートリング読み出し順序で先頭のものを選択する（ルート選択コア `choose_root_slot` と `newest_root_among` は同一の選択ロジック）。プールレベル checker のルート選択（`crates/singlefs-checker/src/walk.rs`）もすでに同一の選択ロジックである：順次比較し、厳格に大きい場合にのみ更新、キー同一時はルートリング読み出し順序の先頭を選択。

**根拠**：

- ルートに親が存在しないことは構造的事実であり、読み出し順序は論理的帰結である。
- E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)：「全候補を個別に検証した後に正当なものの中から最新を選択」という読み出し順序に対し、最初の正当なものを選択する（自己証明を通過したスロットを1つ選択したらそれ以上変更しない）バリアントに変更した場合、正当な履歴 K、K9、KW の3ファミリの境界内で赤となる（3回目の再実行第3セクション）。
- 現役の参照例（本プロジェクトでは未検証）：ZFS の「全ラベル全スロットを全走査した後に txg で最新を選択」はまさにこの順序である。
- キー完全一致時に先頭を選択：コードラウンド裁定 `research/prompts/m3-formal-code-r1-main-verification.md` 第2節「キー完全一致のスロットでどちらを選択するか」の枠（コアは先頭を選択、`newest_root_among` は当時末尾を選択しており自身のドキュメントと矛盾；正当な履歴においてキーが完全一致する2スロットは生成されず）および「改訂反映」節（`newest_root_among` をコアと同一の選択法に改訂）；プールレベル checker も同一の解釈：コードラウンド第2ラウンド裁定 `research/prompts/m3-formal-code-r2-main-verification.md` 第2節弁護側「checker のルート選択同点時に末尾を選択」の行（同型別アームに的中、`walk.rs` のアームが末尾を選択していた）および第3節第2条。
- サードパーティ論証にアーム状況記録がない（確定項目 18）。

**未済の検査**：C88 (ルートリングのタイムライン判別未実装)。

#### 確定項目 25：ルートリング深度はブロック再利用遅延の上限であり、両者は同一の値ではない

**確定内容**：K 世代のルートを保持する ⇒ **直近 K トランザクション内で解放された空間はすべて再利用禁止**、さもなければリング内の旧ルートが指すブロックはすでに上書きされており、**それらはロールバック候補として無効（偽）となる**。⇒ **ルートリングを「K 世代ロールバック可能」と読んではならない。** それがもたらすのは「現世代のルートが破損しても前世代のルートを読み出せる」ことのみである；前世代のルートが**利用可能か否か**はブロック再利用遅延に依存し、それは別の事象であり別のコストである。**両者を混同するとリングが無償であるかのような錯覚に陥る。** ⚠️ **ブロック再利用境界をフォーマット定数としてはならない**（I-7.4 (直近 K 代ブロック非再利用) は D22 (ユニット原子性の合成手法) のこの一文を引用）：それは `max(F_有効, リング内最古の有効ルート)` からその場で計算され（D16 (パブリッシュセマンティクス) 確定項目 1）、F はディスク逼迫時に引き上げられ、リング内最古の有効ルートはパブリッシュに伴って前進する。「ルートリング深度 = ブロック再利用遅延」は容易に「深度はフォーマットフィールドであるため、K もフォーマットフィールドである」と実装されがちであるが、フォーマット定数によって単なる可用性パラメータを固定化することに何の利益もない。**ルートリングは「現世代のルートが破損しても前世代を読み出せる」ことのみを担い**、checkpoint スケールのブロック再利用制約は D16 (パブリッシュセマンティクス) 確定項目 15 が単独で担う。

**射程**：

- K は実行時パラメータではなくなった：I-7.4 (直近 K 代ブロック非再利用) に「旧規定『K は実行時ポリシー、下限 ≥ 2』を破棄」と明記され、「K 世代」とはロールバック候補集合を指す。旧 K 計測条件における「1世代追加ごとにどれだけ余分に拘束されるか」のスケールは E123 (Kとロールバック深度の二者択一のコスト) にあるが、ブロック再利用境界が動的計算に変更された後、それは現行の算出法ではない。
- D22 (ユニット原子性の合成手法) 自身の「1周の所要時間 ≥ 最悪のブロック再利用遅延要件」という文言は、checkpoint スケールで解釈すると達成不能である（1周はミリ秒スケール）；コンシューマ課金の観点からはすでに清算済みである：リカバリのルート選択ロールバックが必要とするのは正確に2世代であり、1周 = R × S ≥ 2 は恒常的に成立する。唯一壁時計時間で課金されるのは走査ジャンプ（親ポインタの読み出しから子ブロックの読み出しまでのステップ）であり、1回のジャンプの所要時間が保護ウィンドウ（ロールバック可能世代数 ÷ パブリッシュレート）を超過すると保護の対象外となる（C214 (走査スキップのウィンドウ余白が未保護)）。
- 2つのブロック固定ルールが拘束するのは同一のブロック群ではない：ルートリングのルールは直近数世代のルートが参照するブロックを拘束し、D16 (パブリッシュセマンティクス) 確定項目 15 + 確定項目 4 は journal 再生ウィンドウ全体で解放されたブロックを拘束する。**荷重を支えるのは位相数であり倍率ではない**：ルートリングによって余分に拘束される脱出経路は、1つの checkpoint 周期において `ルートリング(K) − 1` 個の位相においてのみ存在し、checkpoint 間隔とは無関係である；⚠️ **「250倍の差」といった表現を用いてはならない**（それは位相間比較であり、倍率は checkpoint 間隔に伴って線形に変化する）。

**根拠**：

- E33 (2つのブロック固定ルール)：2つのブロック固定ルールが拘束するのは同一のブロック群ではなく、ルートリング深度を縮小しても checkpoint 完了直後の最初の数位相でのみ空間が解放される——「K が拘束しているものこそが ENOSPC デッドロックの唯一の脱出経路である」という旧根拠は完全に覆された；デッドロック 2 の解決策は予約プールである。
- E50 (ルートリングスロット数の上下限)：1周はミリ秒スケールであり、checkpoint スケールで「1周 ≥ 要件」と解釈するといかなる合理的なスロット数でも達成不能。
- E92 (最悪ブロック再利用遅延要件)：9つのコンシューマを3クラスに分けて課金、リカバリのルート選択ロールバックは正確に2世代、管理者ロールバックはルートリング深度全体を要し、走査ジャンプのみが壁時計時間で課金される。
- ブロック再利用境界：D16 (パブリッシュセマンティクス) 確定項目 1 におけるユーザー確定、原文は変更履歴に記録。
- 証拠レベル：数値は観測値；「したがって旧根拠は書き改められるべきである」は推論であり、ローカルアームのみで実施され、2つのクラウドアームが未済となっている。

**未済の検査**：C214 (走査スキップのウィンドウ余白が未保護)。

```

**出处 `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:512-546`（整段抄，未转述）**

```markdown
### 未確定項目

| # | 項目 | 状態 |
|---|---|---|
| 6 | **zoned のゾーン書き込みポインタは本プロジェクトのチェックサム管轄外の目撃者である** | デバイスが保守し、本プロジェクトでは計算も検証も不能；zoned ラインの着工時に「最新のルートがどこにあるか」の発見プロシージャとともに決定 フォーマット変更の有無：変更する（2026-09-21、根拠：乙の問いは「zoned 上で固定構造（システム構成スロットローテーション、ルートリングスロットローテーション）がインプレースにローテーション可能か否か」、本文に「ライン着工コストは最低でも +1 構造」と明記） **状態：未定。** **進捗：対象なし。** |

未確定項目 6 が新規プールでのファイル作成のバイト列を変更するか：変更しない（2026-09-01、根拠：初期版は非 zoned レイアウトで稼働し、非 zoned 上にはゾーン書き込みポインタというオブジェクトが存在しない。⚠️ 初期版が zoned 稼働に変更された場合は判定不能）。

#### 未確定項目 6：zoned のゾーン書き込みポインタは本プロジェクトのチェックサム管轄外の目撃者である

**問題**：ゾーン書き込みポインタ（WP）はデバイスによって保守され、本プロジェクトでは計算も検証もできない。クラッシュ後、書き込みポインタは本プロジェクトが認識するコミット点より先行している可能性がある。D20 (荷重受け面: ユニットの原子性と自己完結性) の荷重受け面の定義はこれをカバーできない。2つの問いに分解される：甲、WP は本プロジェクトの判定においてどのような地位を占めるか；乙、zoned 上で固定構造（システム構成スロットローテーション、ルートリングスロットローテーション）はインプレースにローテーション可能か。

**現状**：zoned レイアウトライン着工時まで棚上げ：初期版は非 zoned レイアウトで稼働し（D12 (対象メディア) 確定項目 5「標準 SSD」は ZNS を要求しない）、初期版においてこのオブジェクトは存在しない；サードパーティ検証を先走って実施せず、「アドレッシングのヒントとしてのみ機能させ、目撃者とはしない」とあらかじめ固定することもしない、「最新のルートがどこにあるか」の発見プロシージャおよび D12 (対象メディア) のメディア階層化とともにその時点で判定する（ユーザー方針決定、原文は変更履歴に記録）。これが確定できない理由は証拠不足ではなく、その前提が未済の負債であるためである：確定項目 2 は一連の**非 zoned パラメータ**であり、すべて同一スロットアドレスへの反復的な上書きを前提として構築されているが、zoned におけるルート発見プロシージャは一度も設計されていない（確定項目 24 は「一連の発見プロシージャが必要」とのみ注記を残している）⇒ 非 zoned 上では WP というオブジェクトが存在せず、zoned 上では「スロット」というオブジェクトがまだ存在しないため、各アームはまず「最新のルートがどこにあるか」に回答しなければならない。着工時に判明している連動事項：D17 (実装階層とサードパーティパイプライン) 確定項目 1 は単一障害モデルを採用 ⇒ zoned ラインはパイプライン宣言によってゾーン書き込みポインタを押し出すことは禁じられ、コア層の合成を経由しなければならない；D16 (パブリッシュセマンティクス) 確定項目 8 のウォームアップは発見プロシージャに対し「最大でも R 回の空パブリッシュでスロットを占有し、ルートが2台のディスクをカバーして初めて fsync から復帰する」という制約を課す；ライン着工コストは最低でも +1 の構造同値クラスを伴う（D17 (実装階層とサードパーティパイプライン) 確定項目 2）。

**除外されたアーム**：

| 問い | アーム | 現状 |
|---|---|---|
| 甲 | リカバリの権威上限とする | 除外、確定項目 10 に昇格 |
| 甲 | checker に含めてクロスバリデーションを行う | checker の形態は除外、確定項目 11 に昇格；実行時 / マウント時の形態は未評価 |
| 甲 | アドレッシングのヒントとしてのみ機能（次回どこに書き込むか、どこから走査を開始するか、最新判定 / パブリッシュ済み判定 / 完全性判定には含めない） | **条項として記述不能**：追記型のゾーンにおいて「どこから走査を開始するか」は「どれが候補であるか」と等価であり、ルートレコードのフィールド表（確定項目 7）には「これがゾーンの第何番目か」を記録する行が存在しない ⇒ WP を用いて候補集合を画定するか（WP がルート選択に入り前文と矛盾）、ゾーン全体を走査するか（本プロジェクトが書き込んでいない WP より先のバイトを読み出す必要がある）のいずれかとなる；「デバイスが保守する状態」と「デバイスがプローブした幾何構造量」の境界がリポジトリ内で未定義であり、ルートスロット幅はまさにその両者に跨っている（C219 (デバイス保守状態とデバイス検出幾何量に境界がない)） |
| 甲 | 候補集合をユニット全体の自己証明チェックサムを通過したスロットと直接定義 | **除外**：フィルタを列挙子と混同している——I-7.1 (ルートスロット有効集合が非空) の論域「全ルートスロット位置」と述語「自己証明チェックサム通過」はもともと分離して記述されており、本アームは述語を書き換えたに過ぎない；zoned 上で「スロット」には指示対象が存在しない；非 zoned 上の候補集合は R × S 個の既知位置であり、それはまさに WP が存在しない側の世界である |
| 乙 | ゾーンペア交互 + 追記書き込み + ゾーンリセット | **純粋な算術により破綻**：`P` は奇素数 ⇒ `v₂(P × chunk) = v₂(chunk)`、一方 `zone_size` は 2 の冪 ⇒ `zone_size` が領域間隔を整除 ⟺ `zone_size ≤ chunk`；ルートリング領域の起点がゾーン境界に一致せず、領域が他のデータとゾーンを共有するため、ゾーンリセットは恒久的に実行不可能となる。成立させるには確定済み条項の改定を要する（起点を下方にアライメント、`zone_size ≤ chunk` を要求、またはゾーンリセットの放棄） |
| 乙 | 固定構造は一律に通常ゾーンに配置し、通常ゾーンを持たないデバイスは mkfs を拒否 | 確定項目 24 の「zoned クラスではルートがドリフトする」と矛盾し、採用するにはまず当該記述を書き戻す必要がある；NVMe ZNS 名前空間には通常ゾーンが存在しない（ローカルカーネルツリー `include/linux/nvme.h` は `NVME_ZONE_TYPE_SEQWRITE_REQ` のみを登録）、適用範囲が通常ゾーンを持つ zoned デバイスに縮小する |
| 乙 | ゾーンタイプによる分岐（通常ゾーンがあれば通常ゾーン、なければゾーンペア交互） | **除外**：分岐述語に主語が存在しない（プールレベルの解釈における分岐変数は書き込み飛行中に変化し、`.claude/rules/fs-design.md` 厳格要件 5 に抵触）；真の判定基準は通常ゾーンが `{0, P·chunk, 2P·chunk}` の3点をカバーしているか否かである；領域スロット数 S はシステム構成内のプールレベル単一値であり、2種類の形態を表現できず、「1周 = R × S」が定義を失う |
| 甲 / 乙 | zoned ライン着工待ち | 現状 |

**既存の観測**：

- E125 (シーケンシャルゾーンの書き込みポインタ上方で何が読めるか)（`null_blk`、ローカルマシンに実 zoned ディスクはなく、数値はすべてシミュレーション）：シーケンシャルゾーンの WP より上方でバイトが読み出し可能（全 `0xff`）、`zone_full=1` によって WP がコミット点に先行する状態を再現可能；未書き込みのシーケンシャルゾーンからは `0xff`、通常ゾーンからは `0x00` が読み出される；**一度も書き込まれていないシーケンシャルゾーンと、書き込み後に `BLKRESETZONE` されたゾーンは、バイト、書き込みポインタ、ゾーン状態の3者が完全に一致する** ⇒「全 `0xff` ⇒ 未書き込み」という、未書き込みと破損を分離する規則は単一デバイス上では利用可能な入力を持たず、WP の取得に置き換えても同様に分離不能である。
- その `0xff` はデバイスが返したものではなく、`null_blk` ドライバ内のハードコード行（`memset(p + valid_len, 0xff, len);`）によるものである；NVMe ZNS における未書き込みブロックの読み出し戻り値は `DLFEAT` によって選択され、`NSFEAT` のあるビットがセットされている場合コントローラは直接エラーを返すことができ、Linux の `drivers/scsi/sd_zbc.c` は host-managed デバイスに対して `if (!sdkp->urswrz) return -ENODEV` を直接実行する；`zonefs.rst` 330–332 行：読み取り専用ゾーンの書き込みポインタは ZBC / ZAC 規格上無効である——ルートの読み出しが最も必要な局面（メディアが故障し始めた時）こそが、WP が値を持たない可能性がある局面である（外部事実、本プロジェクトでは未検証）。
- サードパーティ：未確定項目 6 は3ラウンド（第1ラウンド資料 `_d22-item6-background.md`、第2ラウンド `d22-item6r2-*`、第3ラウンド `d22-item6r3-*`；第2ラウンドのローカルアーム3回のサンプリングはいずれも無効化）。

**未済の検査**：C219 (デバイス保守状態とデバイス検出幾何量に境界がない)；C230 (ゾーンリセット後の状態と未書き込み状態が区別できず不変条件が全緑)；C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)：ストライプ幅 × `zone_size` の割り切れない枠に `P` を代入する必要がある。

```

**出处 `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:547-558`（整段抄，未转述）**

```markdown
### 他の決定との連鎖

| 決定 | 連鎖項目 |
|---|---|
| D4 (チェックサム位置) | D22 (ユニット原子性の合成手法) は D4 (チェックサム位置) に対して根拠を1つ提供した：**検出能力をデバイスの仮定から切り離すこと**。現役の比較対象は btrfs（親ポインタにチェックサムなし ⇒ transid の追加を余儀なくされる）と ZFS（親ポインタにチェックサムあり ⇒ 第2の関門なし） |
| D12 (対象メディア) | 「ルートポインタをインプレース上書き可能」という文言の解釈を覆す；2種類のレイアウトは同一のルートメカニズムを必要とする（確定項目 24） |
| D16 (パブリッシュセマンティクス) | ルート自身の冗長形態というギャップが単一のメカニズムに集約された；journal のフォーマットに厳格要件が1つ追加された（世代番号は実際に検証されなければならない、確定項目 23） |
| D18 (ブロックに含める情報) | D22 (ユニット原子性の合成手法) 確定項目 20 は D18 (ブロックに含める情報) の直接の系である：断裂粒度が拡大すると消失書き込みに退化し、消失書き込みはブロック内に何を含めようとも捕捉できない |
| D20 (荷重受け面: ユニットの原子性と自己完结性) | D22 (ユニット原子性の合成手法) は D20 (荷重受け面: ユニットの原子性と自己完結性) 系 3 の第1問および第2問に対する回答である；第3問（違反時にどう検出するか）はクラッシュポイント再生に着地する |
| D21 (権威態と派生態の境界) | 会計は権威態でありレプリカを持たないため、断裂後のリカバリ経路は D22 (ユニット原子性の合成手法) 確定項目 3 である（「会計構造は COW か」；確定項目 2 はルートリングパラメータ） |
| [checks-owed.md](../checks-owed.md) C16 (パイプラインが障害モデル例外を宣言) | 断裂注入は dm-flakey に依存できず、クラッシュポイント再生 harness 内で実行しなければならない |

```

**出处 `.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md:559-562`（整段抄，未转述）**

```markdown
## 過去の版

D22 (ユニット原子性の合成手法) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:1-6`（整段抄，未转述）**

```markdown
## D22 单元原子性怎么合成 —— 半定（一项未定）

**中文** · [English](22-synthesizing-unit-atomicity.en.md) · [日本語](22-ユニット原子性の合成手法.ja.md)

D22（单元原子性怎么合成） 管单元落盘的原子性怎么合成：分界线（落盘之后有没有带它校验和的父指针）把单元分成「COW + D4（校验和位置） 就够」与「必须自证」两类；自证那一类（根、系统配置、journal）要的整单元校验和、被实际检查的世代号与槽轮换；根环的参数、根记录与系统配置的字段表、固定结构的放置、区写指针不作恢复的权威上界也不进 O2（独立解析器 + checker）、盘上定长字段的字节序与登记位。不管的：发布的持久顺序与块复用的界在 D16（发布语义）；journal 的记录格式、环几何与重放前缀在 D23（journal 的角色与格式）；单元头与单元类型码在 D18（块里携带什么信息）；指针的字段与宽度在 D19（块指针的结构与宽度预算）；写粒度、固定结构槽距与「生命周期不同」的判据在 D2（RAID 条带策略） 已定项 15 / 16 / 17 / 19。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:39-50`（整段抄，未转述）**

```markdown
#### 已定项 1：journal 取定长环

**定案**：journal 取定长环，权威记录在 D23（journal 的角色与格式） 已定项 2。决定性理由是链式把死锁 3 的判据从「最坏操作 ≤ 环大小 / F」（mkfs 可算）换成「最坏操作 ≤ 当前空闲」，而后者在操作中途会变，撞上 `.claude/rules/fs-design.md` 硬要求 5。

**射程**：环不随文件变大：一次操作超出单事务预算时的出路是运行时强制拆事务（D23（journal 的角色与格式） 死锁 3 的兜底，I-8.1（环几何够大）），不是让环跟着文件长。链式日志的出路与为什么不取，归已定项 23 与 D23（journal 的角色与格式） 已定项 2。

**依据**：

- 无实验：已定项 1 只是指针，环与链的取舍和它的实测依据都在 D23（journal 的角色与格式） 已定项 2。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:51-92`（整段抄，未转述）**

```markdown
#### 已定项 2：根环的参数

**定案**：

| 量 | 定案 |
|---|---|
| **区域数 R** | **3**（`R = F + 1`，F = 2） |
| **区域位置** | 区域 r 落在 `r × P × chunk`，**P 素数且 `P > devs`**；地址空间、基址与第一版取值见已定项 16 |
| **槽序** | **跨区轮转**（区域 = `txg mod R`）；区域内槽位 = `(txg div R) mod S`（已定项 16） |
| **每区槽数 S** | 住系统配置。**下界 k_tol + 2 = 4**（D16（发布语义） 已定项 1：**每区槽数 S** 要装得下 4 个状态加一次带 k_tol 次根槽写失败的处置——那一项是直接在 S 上量的，不是在 R × S 上；「环深」这个词在 C131（栈深与保护窗口的量纲） 出过一次事，这里不用它），**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。⚠️ **具体取多少不是格式决策**——格式承诺的是「S 是系统配置字段 + 挂载时判区间」，取值由第一版实现按 D25（目标负载优先级） 的负载形态定 |
| **根槽宽（每条根记录多少字节）** | **等于挂载时探测的 `physical_block_size`**，等号就够。大于它不多买安全性，只多花字节。mkfs 按池内 `max(physical_block_size)` 划，挂载时逐设备复算比对，未达到**拒绝挂载**——形态与 D21（权威态与派生态的分界）「单元几何挂载时求值一次」同型 |
| **系统配置槽宽** | **格式常量 4096**（登记名 `SYSTEM_CONFIGURATION_SLOT_BYTES`），与 D23（journal 的角色与格式） 已定项 12 记录尺寸 4 KiB 同一先例；落在 D15（格式冻结政策） 第 1 层，改它要 incompat 位。D20（承重面：单元的原子性与自包含） 那句「槽宽 = physical_block_size」是撕裂判定的分辨率，不是结构尺寸 |
| **槽距** | 根槽与系统配置槽的槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍（D2（RAID 条带策略） 已定项 19）；根槽序号到偏移的换算见已定项 16 |
| **对齐** | 每个槽的起始偏移是判定宽度的整数倍，落成 I-7.5（根槽按判定宽度对齐） |
| **保留槽** | **不留**。⚠️ **重开条件**：引入多主机保护时，D22（单元原子性怎么合成） 已定项 2 的保留槽这一格重开 |

**射程**：

- **适用条件：区域所在的那块盘还在。这一条必须连同结论一起引。** 区域位置是 mkfs 时划在盘上的既成事实，**落盘归属也是**——D2（RAID 条带策略） 已定项 7 逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」⇒ **加盘 / 换盘不再改变任何既有区域的归属。** ⇒ **R = 3 的容错承诺仍会退化，但机制不是「区域撞到一起」，是「区域所在的那块盘没了」。** 这一格是 D2（RAID 条带策略） 已定项 3（走显式几何再求值事务）的一个实例，落在 D2（RAID 条带策略） 已定项 5（根环不重放置）。
- **`P` 只剩「区域位置」这一个用途**：「`P` 素数且 `P > devs`」这个约束原本的理由是「`gcd(P, devs) = 1` ⇒ 乘 P 是模 `devs` 的双射 ⇒ 区域两两落在不同盘上」，那条理由随归属公式一起作废；约束本身还要不要，仓里一处都没写过（C217（P 大于设备数这条约束的理由没了而约束还在））。
- 根槽宽取 `physical_block_size` 时，阵列上 `io_min` 比它大得多：槽距取 io_min 的整数倍之后一个 io_min 单元里不再挤进两个根槽（D2（RAID 条带策略） 已定项 17 按 D2（RAID 条带策略） 已定项 15 的判据把同一区域里的不同槽算作生命周期不同的对象）；每次根槽写仍然小于 io_min、逐字违反 D2（RAID 条带策略） 已定项 16，第一版知情接受（C212（固定结构的写小于 io_min））。SSD 内部对整个映射单元的 RMW 没注入过，真设备上只量过 RAID5 写洞。
- 系统配置槽宽四条候选的代价 E126（系统配置槽宽四条候选的代价） 量过，它的「乙」臂量的是探测值 max(pbs, io_min)，与判决里的乙（记录的槽距）不是同一个量。系统配置槽宽那一格与根槽宽分开：根槽宽仍是探测到的 `physical_block_size`，第一版本机 `nvme0n1` 现查是 512、D2（RAID 条带策略） 已定项 9 第一版跑 2 块盘 ⇒ 同构 512 池上槽就是 512。
- 根环的降级语义写下来了、在真设备上量过，答的是 [pitfalls.md](../pitfalls.md) 第 9 条（degraded 挂载会写出单副本、越救越糟，根因是降级行为没有设计）。
- **读者择系统配置时判根槽宽与固定结构槽距**：固定结构槽距 ∈ [4096, 1 MiB − 4096]（上界是槽 1 整槽落在根环基址之前，`root_ring::region_start(0)` 减系统配置槽宽 4096）；根槽宽（`physical_block_size`）∈ [根记录宽 466 字节（已定项 7）, 槽距]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts`，`SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds`）；池级 checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、该池其余不变量一律报不适用（实审 A3-checker-2 落地：`crates/singlefs-checker/src/image.rs` 的 `judge_system_configuration_values_the_reader_accepts`）。
- 根环只承担「本代根写坏了还能读到上一代」，不是「可以回滚 K 代」（已定项 25）；zoned 上这套参数不适用（未定项 6）。

**依据**：

- F = 2：产品判断，E46（根环区域间距与失败域） / E47（根环失败域的损失） / E48（根环几何的可行点） / E50（根环槽数的上下界） 四处都自陈实验答不了它；用户定案，原话在变更史。
- E46（根环区域间距与失败域）：「盘首盘尾」在常见条带几何上落同一块盘 ⇒ 区域要按失败域选，区域数 = 要容忍的失败域数 + 1。
- E47（根环失败域的损失）：盘首盘尾放置在多数设备数上丢一块盘就挂不上，区内连续槽序的最坏回退比跨区轮转大 8 倍 ⇒ 槽序取跨区轮转。
- E48（根环几何的可行点）：放置规则 × 槽序搜遍，可行点只有「素数步长 + 跨区轮转」这一条组合。
- E50（根环槽数的上下界）：它量出的 S 下界与上界都在 4..16 之外，S 不是一个要精算的量。
- E53（丢一整块盘之后根环还挂不挂得上）：虚机真块设备上逐块真抹一次，四次都挂得上、最坏回退 1 代。
- E55（条带映射的算术在真设备上对不对）：真 `dm-stripe` 上素数步长六组全部落在互不相同的盘上。
- E41（根环槽几何 2×4×256）：槽宽小于判定宽度时一次撕裂打掉多个槽、偏移半个槽时风险槽翻倍 ⇒ 槽宽取判定宽度、起始偏移按它对齐；「2 区 × 4 槽 × 256 字节」那个候选因此被换掉。
- E34（根环槽几何）：素数步长下一个 io_min 单元盖不住两个区域、抹掉一整个区域最坏回退 1 代；真 md/raid5 上撕裂只污染它那一行，暴露面 ≠ 损失面。
- 系统配置槽宽：三方 `research/prompts/d22-slotwidth-r1-main-verification.md`（正推腿判记录的槽距、反推腿判常量 4096、本地腿两次抽样不稳定）之后用户收尾弹窗定案，原话在变更史。
- 保留槽：唯一的现役参照里有效槽数不等于物理槽数——ZFS 的 `ub_txg % (VDEV_UBERBLOCK_COUNT − m)`，那个 `m` 是多主机保护（MMP）占掉的；与本工程的一处已知差异：本工程没有多主机语义（外部事实，未在本项目验证）。

**欠**：C212（固定结构的写小于 io_min）；C217（P 大于设备数这条约束的理由没了而约束还在）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:93-106`（整段抄，未转述）**

```markdown
#### 已定项 3：记账结构取 COW

**定案**：记账结构取 COW，挂父指针下。「先按原地跑起来、以后再改 COW」这条路不存在：两条路在 mkfs 那一步就分叉。

**射程**：只定记账结构的更新形态。写放大这根轴上原地是赢的（E38（记账结构 COW 还是原地），D25（目标负载优先级） 的粒度一改要重跑），决定方向的不是写量；「全盘扫描要多久」没量，池大小是配置项。

**依据**：

- 已定项 22：记账是权威态（D21（权威态与派生态的分界））、住在根下面，原地覆写同样让旧根的回退候选变假——这条不依赖 mkfs 预留怎么建模。
- E38（记账结构 COW 还是原地）：COW 臂 mkfs 预留恒为 0、撕裂打不掉唯一一份；原地臂要按记账树几何预留固定位置。
- 用户定案（单元自包含 ⇒ 父指针那条路不需要额外的定位约定，D20（承重面：单元的原子性与自包含）、D18（块里携带什么信息） 已定项 3），原话在变更史。

**欠**：C95（依据被撤回的已定项无人回看）：E38（记账结构 COW 还是原地） 的恢复代价那一栏已撤回、原地臂按静态全量预留建模被判为稻草人，已定项 3 靠已定项 22 撑着。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:107-118`（整段抄，未转述）**

```markdown
#### 已定项 4：「本事务内释放的块不得重用」存在

**定案**：「本事务内释放的块不得重用」存在。它就是 D16（发布语义） 已定项 15「checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合」。⚠️ **但约束粒度比 checkpoint 更严**：D16（发布语义）那条规则的第二条理由是按 **journal 位置**说的——块在位置 p 被释放、q > p 被重新分配写入，重放到 `p < r < q` 的前缀就会得到一棵指向该地址的树，而盘上字节已是 q 的内容。⇒ 真正的约束是**整个 journal 重放窗口**，不是 checkpoint 边界。缺了它的失败模式：**可检测（父指针校验和失配），不可恢复。**

**射程**：btrfs 用 pin 实现同一件事（外部事实，未在本项目验证）。它与根环那条钉块规则扣的不是同一批块，两条的分工见已定项 25。

**依据**：

- 无实验：它是 D16（发布语义） 已定项 15 按 journal 位置推出的逻辑推论，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:119-130`（整段抄，未转述）**

```markdown
#### 已定项 5：D2（RAID 条带策略） 的口径要扩，已完成

**定案**：D2（RAID 条带策略） 已定项 16 与已定项 17 都写进去了：不发出小于设备物理映射单元的写（在 Linux 上那个量是 `io_min`，不是 `physical_block_size`），即 D2（RAID 条带策略） 已定项 16；不让两个生命周期不同的对象共享同一个物理映射单元，即 D2（RAID 条带策略） 已定项 17。

**射程**：已定项 5 只记登记完成；两条硬要求的射程与判据归 D2（RAID 条带策略）：「生命周期不同」按 D2（RAID 条带策略） 已定项 15 判，固定结构槽距归 D2（RAID 条带策略） 已定项 19。固定结构的写小于 io_min 与 D2（RAID 条带策略） 已定项 16 打架，第一版知情接受（C212（固定结构的写小于 io_min））。

**依据**：

- 无实验：这一项是把两条硬要求写进 D2（RAID 条带策略） 的登记工，没有可量的量。

**欠**：C212（固定结构的写小于 io_min）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:131-175`（整段抄，未转述）**

```markdown
#### 已定项 7：根记录的字段表

**定案**：**根记录恒走间接层：固定头 + 一条指向「树表单元」的指针，树根一律住树表单元，不平铺进根槽。** **不做「树少时平铺、树多时间接」的分支**：分支变量是树数，而树数在挂载期间会变（建一个可写头就加一棵树，D6（快照实现模型）已定项 1）——撞 `.claude/rules/fs-design.md` 硬要求 5「分支变量不许在操作中途改变」。恒一种形态。

| 字段 | 宽 | 出处 |
|---|---|---|
| magic | 4 | — |
| fsid | 16 | I-1.4（块头 fsid 一致）同口径 |
| flags | 4 | 位 0 = 卸载记号：正常卸载那一串写的根置 1（D16（发布语义） 已定项 1「正常卸载」），其余根写 0，池级 checker 按它给 I-7.9（回退下界 F 不高于抬 F 的上限） 分支；其余位第一版恒 0、非 0 拒收（D22（单元原子性怎么合成） 已定项 17）。位 n 住第 n div 8 个字节的第 n mod 8 低位（已定项 13） |
| **实例代号** | **4** | D16（发布语义）已定项 6 骑手 1：与 D23（journal 的角色与格式）已定项 9 的 journal 实例代号**同一个计数**；择新在 checkpoint_txg 平局时按它高者赢（读根环时每条根照落点公式（已定项 16）核槽位，第 0 代根照 mkfs 种根的槽另认，一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根、取读根环次序最前的（已定项 24）；公式核还没实现，欠在 [checks-owed.md](../checks-owed.md) C743（重建分配器记新类时不核根住不住这一槽）） |
| **checkpoint_txg** | 8 | D16（发布语义）已定项 6：发布计数、轮转键、记账代、重放水位是同一个数——**不另设「发布代号」字段** |
| 树表单元指针 | **86** | 指针头部 50 + 位置条目 **14** × 2 副本 + 实例代号 4 + 出生序号 4（D19（块指针的结构与宽度预算） 已定项 7 / 8；位置条目 = 设备 ID 4 + 16 KiB 槽号 6 + 密文校验和 4，D19（块指针的结构与宽度预算） 已定项 4）；自举豁免，位置条目是权威 |
| **树 ID 水位** | **8** | D8（核心索引结构） 已定项 8 ②：`max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，每次发布随根记录重写 |
| **回退下界 F** | **8** | D16（发布语义） 已定项 1：可再分配的界 max(F_生效, 环里最旧有效根)；平时不动，准入不够时抬，正常卸载时抬到现行那一版的 txg；系统配置另记一份（已定项 9），恢复后生效值取两处读得出的最大值（D16（发布语义） 已定项 1「生效」） |
| 自证校验和 | 32 | 与 E23（journal 几何）的 `header_csum` 同宽 |
| **实例表单元指针** | **86** | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2：实例表单元（D18（块里携带什么信息） 已定项 11 打包记录类型 4）不进容器索引，由根记录直接持有；结构与树表单元指针同型 |
| **中央映射树根指针** | **86** | D19（块指针的结构与宽度预算） 已定项 11：中央映射树的根住根记录、不进树表，与树表单元指针同形（自举豁免）；mkfs 与暖机写全 0 |
| **分配记录树根指针** | **86** | D16（发布语义） 已定项 9：树表 0 条的一版上写行那次发布写的那一片分配记录节点；只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0；与树表单元指针同型 |
| **回收门槛 t** | **8** | D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」：写这条根时的回收门槛，取 max(这条根自己带的 F, 分配器记这条根时的回收门槛)，让读坏或读回全零的根槽钉住的门槛随根落盘，不只在这个进程的内存里；零单元发布照抄上一版的值，mkfs 的第 0 代写 0；池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 判 |
| **钉槽 p** | **1** | D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」：建这一版的账时还钉着的根环槽里钉值最小的那个，钉值 = 3·槽 + 区域，就是那一槽可能住过的最旧 txg（小于环长 3S）；没有钉着的槽写 255（`ROOT_RECORD_NO_RING_SLOT_PINNED`）；钉着的槽在这个进程写掉它时必解，那一槽已住着本实例写的根而 p 还指着它，池级 checker 的 I-7.19（钉着的槽不住本实例写的根） 判红 |
| **算法类型** | **1** | D9（加密） 已定项 12 第 1 项（射程扩到自证单元）：0 = 未加密；登记表在 D9（加密），D22（单元原子性怎么合成） 已定项 17 |
| **nonce** | **12** | D9（加密） 已定项 12 第 1 项；与单元头那 12 字节同口径（D18（块里携带什么信息） 已定项 14） |
| **MAC** | **16** | D9（加密） 已定项 8 同口径；根记录是自证结构，加密开启时由它自己的 MAC 罩住 |

行序即盘上顺序，偏移按行累加：magic 0 / fsid 4 / flags 20 / 实例代号 24 / checkpoint_txg 28 / 树表单元指针 36 / 树 ID 水位 122 / 回退下界 F 130 / 自证校验和 138 / 实例表单元指针 170 / 中央映射树根指针 256 / 分配记录树根指针 342 / 回收门槛 t 428 / 钉槽 p 436 / 算法类型 437 / nonce 438 / MAC 450。

<!-- format-const: ROOT_RECORD_BYTES = 466 stale=根记录 371 字节|根记录合计 371|14 个字段合计 371 字节|合计 **371 字节**，512 槽内余量 141|3 × 371|合计 **457 字节**|根记录宽 457|15 个字段合计 457|457 / 512 槽|3 × 457|ROOT_RECORD_BYTES=457|ROOT_RECORD_BYTES = 457|`ROOT_RECORD_BYTES` 是 457|根记录（457 字节|根槽偏移 428..457|15 个字段合计|512 槽内余量 55|根记录合计 457|根记录 **457 字节**|根记录 [0, 457)|[457, 槽宽) -->
合计 **466 字节**，512 槽内余量 46——槽宽是运行时探测值，余量给将来字段。

**树表单元**：16 KiB 元数据单元，装 **81 棵/层**，超过再嵌一层（81² = 6561 棵）；条目宽取 D8（核心索引结构） 已定项 8 定的 **200 字节**（树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8 + 头 ID 8 + 预留 76），头宽取树表单元自己那个码 2 头 **131**（key = 树 ID、key 宽 8 ⇒ 115 + 16，D18（块里携带什么信息） 已定项 16 / 已定项 18）：⌊(16384 − 131) / 200⌋ = ⌊16253 / 200⌋ = **81**。⚠️ **分母只有一种读法**：16384 减含预留位的码 2 头，不减别的数（C157（树表容量在两处按不同条目宽算） 要的正是这个统一口径；D8（核心索引结构） 已定项 8 那一处逐字相同）。树表单元的头字段按 D18（块里携带什么信息）已定项 7 的索引节点类身份段走；它是权威态走读的一部分（I-7.2（最新根可完整遍历）的「走完全部权威态」经它下去）。

**射程**：只管根记录与它指向的树表单元；系统配置不带树表单元指针（已定项 12）。⚠️ **重放水位的读法**（D23（journal 的角色与格式）已定项 14）：恢复只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录；tail 只是扫描起点的优化。间接层的代价是每次发布多 COW 一个树表单元，知情接受；树多过一层嵌第二层之后每次发布多 COW 几个单元没算成绝对值（C148（树表条目变宽把间接层门槛压到二层））。

**依据**：

- E79（根记录的容量）：根槽宽 = `physical_block_size` × 树表 day-1 开放（D15（格式冻结政策） 已定项 2）× 每头一棵树（D6（快照实现模型） 已定项 1）三条合起来，平铺装不下这个设计自己已定的东西 ⇒ 间接层结构上躲不开。
- 末尾 29 字节（算法类型 + nonce + MAC）：根记录与 journal 记录头是自证结构，没有父指针带 MAC 与 nonce ⇒ 不 day-1 留位，开加密那天它们要么整条走明文豁免（泄漏 `checkpoint_txg`、树表指针、映射根指针、树 ID 水位），要么就是一次布局变更；D9（加密） 已定项 3 的原则是为加密留的位所有卷都留、不挪作他用。
- 用户定案（间接层与字段表、树 ID 水位、实例表单元指针、回退下界 F、中央映射树根指针、三条指针 86、末尾 29 字节），原话在变更史；三条指针变宽与末尾 29 字节没有走三方论证，可推翻。
- 用户定案 2026-09-23：加分配记录树根指针 86 字节（C512（树表 0 条的一版上被换下的实例表记在哪没有条款）），合计 371 → 457，原话在变更史。
- 用户定案 2026-10-20：根记录加回收门槛 t 8 字节与钉槽 p 1 字节（C773（读坏与全零根槽的回收门槛钉只在内存里）；D16（发布语义） 已定项 1「根槽这一次读坏」那一行「钉也落盘」），合计 457 → 466，原话在变更史。t 在偏移 428、p 在 436、算法类型 nonce MAC 往后挪是主 agent 定的，照根环第二轮正推腿 Y4.2（`research/prompts/m3-close-rootring-r2-sonnet-output.md`「Y4.2 字段」），没被攻过，零轮；无实验：补的是两个字段的宽度与偏移，没立实验号。
- 用户定案 2026-09-26：flags 位 0 当卸载记号（I-7.9（回退下界 F 不高于抬 F 的上限） 取 (b)+(c) 合用），原话在变更史；无实验：三方原型 `research/prompts/m2-rollback-forward-r3-opus-model/` 量了三条改法的判别力（第三轮判决 K6：(a) 丢「准入抬过头」整类出局；(b) 丢「准入抬过头却带了卸载记号」那一格；(c) 池级镜像不判），没立实验号；合用是主 agent 推的，被攻过零轮。
- 实例代号那一行的平局：根环三方第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C743（重建分配器记新类时不核根住不住这一槽） 那一格（同 txg 两条根并存与已定项 16 的落点公式冲突，改的是这一行的平局括号；第 0 代例外是第一轮打中裸公式之后补的，第二轮攻过没打中，第三轮辩方站住）。

**欠**：C88（根环的时间线判别未实现）；C148（树表条目变宽把间接层门槛压到二层）；C308（自证单元的字段偏移没定）；C743（重建分配器记新类时不核根住不住这一槽）：读根环时的落点公式核。（C78（根记录没有字段表） 2026-10-11 已还清，并入 C66（格式级空白没有分项号）；C157（树表容量在两处按不同条目宽算） 2026-10-11 已还清，并入 C148（树表条目变宽把间接层门槛压到二层），见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:176-201`（整段抄，未转述）**

```markdown
#### 已定项 8：固定结构的放置

**定案**：三条，都是针对 2 盘的第一版（D2（RAID 条带策略）已定项 9）：

1. **系统配置每盘放一份。** 怎么更新它按已定项 21 办（原地覆写的结构要有整单元校验和，还要有真的被检查的世代号），做成槽轮换，每盘至少 2 个槽；槽的规则见已定项 16，字段见已定项 9。
2. **journal 环在两块盘上互为镜像。** 代价是稳态下每条记录写两遍。**这不是新增的开销**——D2（RAID 条带策略）已定项 6 本来就要求任何写都得 w≥2，这条定案只是把「任何写」明确到 journal 记录上。
3. **mkfs 要把第 0 代根种进全部根环区域**；写任何元数据之前，先把每块盘的 [0, 根环基址) 整段写 0（第一版 [0, 1 MiB)，恰盖住系统配置槽 0 与逐档找槽 1 的全部 2 033 档，用过的盘上重做 mkfs 不留旧池的系统配置槽），并把三个根环区域整段写 0（含未种根的那 3 × (S−1) 个槽），journal 环也整段写 0：三者都由 mkfs 自己经块设备的写零动作发出，每块盘上 [0, 根环基址)、根环三个区域与 journal 环各一次调用（区域归属只定根种在哪块盘，不定清哪块盘），录制流里各记一步整段清零（D17（实现分层与第三方管道） 已定项 2 第 1 条的第一种），每块盘 5 步、两块盘共 10 步，段序列与步数以 [layout/01-first-txn.md](../layout/01-first-txn.md) 八「mkfs 种根」那一行为准；每块盘的 [0, 根环基址) 清零之后两块盘各过一道屏障（作废），之后才写别的；其余清零与单元写之间不加屏障。fsid 由 mkfs 的对外入口生成，前 8 字节取自随机源；内层入口可给 fsid，给测试、装置与崩溃点重放用。种满之后「刚 mkfs 完、一次都还没发布」这一格跟稳态一样，根最多退回区域深度那么多代。

**射程**：系统配置自己的更新协议（槽轮换的代号、校验和多宽）跟着单元头字段表（D18（块里携带什么信息）已定项 7）的口径走，落在已定项 9 与已定项 16；zoned 介质上这些固定结构能不能原地轮转，属于 D22（单元原子性怎么合成） 未定项 6，还没答。容错承诺是单故障：每盘一槽系统配置坏在承诺里，取号之后两块盘最新那一槽持续坏也在承诺里（用户 2026-10-19 定，原话在变更史）：取号写两轮，每轮逐盘一写、轮后一道池屏障，每块盘两槽都带取号时的新号，全或无判失败时每块盘只回卷一次（`crates/singlefs-core/src/transaction.rs` 的 `write_the_witnessed_acquisition`、`roll_back_acquisition`）；每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩），那一形抬 F 之前池级 checker 一条都不红（可写挂载的预隔离护住了，承诺仍不罩它），两条用例当登记，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） 那一行。可写挂载读系统配置的见证按盘读（S-辨：一槽撕裂、另一槽自证过，与设备读不出分开），判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）。判不出的盘面有一条离线出路（C596（见证判不出的系统配置盘面只剩只读挂载））：管理员断言一个版本，那一版的根或那次发布的末条记录读得出且是它、各盘单元区里本池单元的 `birth_txg` 最大值不大于它，才只重写不自证的系统配置槽，写的实例代号取各盘自证槽、根环、journal 环与单元写序里的最大值、不加一；否则一个字节不写（用户 2026-10-19 定、2026-10-20 加 `birth_txg` 闸，原话在变更史；`crates/singlefs-core/src/system_configuration_repair.rs` 的 `rewrite_system_configuration_slots_for_the_asserted_version`）。某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒：这块盘带所选那一版的全部单元（清单非空、每个落点读一次、字节与验得过的那一份相同）；这块盘的根环槽与 journal 环槽没有一槽设备报读错；这些槽里没有比所选那一版新的根或记录（按择根的序）；这块盘单元区里没有写序实例大于所选那一版实例的单元；所选那一版的根槽那一半不缺（与落后支缺根那一半同一判）。补写失败照取号每块盘回卷一次（用户 2026-10-03 定这一步拒，2026-10-19 开例外，2026-10-20 加单元区扫描与环槽读错两样，原话在变更史；`crates/singlefs-core/src/mount.rs` 的 `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`，实现只拿两槽都读得出、都自证不过的盘过闸，有一槽设备报读错的照旧拒）。丢写的界按类写：全池读回一个值的纯形丢写 120 格，一盘报读错、另一盘读回全 0 的混合形丢写 0 格（C331（择根倒挂压过已确认的写） 网格上的格数，副本上量的），丢写格都落在每盘各坏一槽系统配置、同时最新根一时读不出的双故障形里。两块盘是同一个池时，作废持久之前崩溃：旧池读回最后确认的一版；可写挂载的结局按现行补写条款。作废持久之后、最后一道屏障之前崩溃：恢复只有两种结局——整池拒（`NoValidSystemConfiguration`），或读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）；后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置。调用方拿自己断言的参数（例如旧池的 fsid）去挂、与盘上择到的系统配置不符而拒（`CallerParametersDisagreeWithTheSelectedSystemConfiguration`），是调用方的断言与盘不符，不算盘面的结局。设备守屏障时，作废持久之后任何结局不得读回旧池的任何一版，也不得新系统配置与旧池的根并存；屏障只到一块盘归 C6（块层语义假设写错），两块盘来自两个池的起点不在射程里。

**依据**：

- 双故障形不罩：用户 2026-09-28 定案，原话在变更史；那一形的两条用例在 [checks-owed.md](../checks-owed.md) 已还清表 C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） 那一行。
- E87（固定结构的放置）：2 盘 8 种失效组合穷举——系统配置只放一份时掉了那块盘全池挂不上（数据、根、journal 健在也没用），每盘一份 + journal 镜像 8 格全可挂、重放窗口零丢失；不种满第 0 代根时「刚 mkfs、零次发布、掉任一盘」连一个能退回去的根都没有。
- 整段写 0：不写的话，在一块装过上一份 singlefs 的盘上重 mkfs，旧文件系统留在槽 1..S−1 里的根记录自证校验和照样通过、`checkpoint_txg` 还更大 ⇒ 第一次挂载择到的是上一个文件系统的根，而 I-7.1（根槽有效集非空） 与 I-7.3（环健康性） 全绿（推的，没量）。
- [0, 根环基址) 整段写 0：三方判决 `research/prompts/m3-c651-r1-main-verification.md` 第三节「用过的盘上重做 mkfs 留下的旧池系统配置槽怎么处置」那一行与用户定案（清零之后，用过的盘换槽距重做 mkfs 再撕两盘槽 0 不再择到旧池残留的槽 1：攻方在副本装置上量过、主 agent 复跑坐实，入库这一侧由 `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader` 那条用例判）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 mkfs 中途崩溃那一格（清零那一句站住，收掉的是 mkfs 第一道屏障之后那一格）。实现在 `crates/singlefs-core/src/make_filesystem.rs` 每盘清零循环的第一句，用例 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 的 `remaking_the_filesystem_with_another_io_min_leaves_no_slot_one_of_the_earlier_pool_for_either_reader`。
- 用户定案，原话在变更史。
- 系统配置见证读按盘读、判不出取 R2：三方判决 `research/prompts/m3-ring-r2-main-verification.md` 第二节第 9 行（按盘读站得住）与 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 7、8 行（按盘读加每盘合起来混合形丢写 240，界按类写；R2 零轮）、第四节末「R2 验证」段（R2 在 C331（择根倒挂压过已确认的写） 网格上量过：纯形 120、混合形 0，混合形多拒 116 格；八趟网格由事务员在新副本上复核，与辩方日志逐键逐行字节相同，网格不在仓里）。用户定案：改取 R2（原话在变更史）；某块盘一份自证过的系统配置都没有时取号那一步也拒，用户 2026-10-03 定、2026-10-19 开补写例外（原话在变更史）。无实验：数都在三方副本与事务员的副本上，没立实验号。
- mkfs 两趟写序与中途崩溃的条文：三方判决 `research/prompts/m3-close-c656-r1-main-verification.md` 第二节 V2（乙站住，条件是两趟写序）、`research/prompts/m3-close-c656-r2-main-verification.md` 第二节 U2、`research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S2、S3（合成形态与两趟写序站住，挂载那两个分句删掉，条文取辩方报告 `research/prompts/m3-close-c656-r3-sonnet-output.md`「三之末」那一稿，零轮措辞）；用户定案 2026-10-20「认，照改写稿写回」，原话在变更史。无实验：数在三方副本上量（段序列 `2+12+1+1+1+4`、27 步、带撕裂 31 195 态），没立实验号。
- fsid 由对外入口生成、前 8 字节取自随机源：同一份第三轮判决第二节 S1 生成那一格（边界在第 8 / 9 字节，两腿量过）与第三节；用户定案 2026-10-20「对外生成，留内层入口」，原话在变更史。
- 取号写两轮、每块盘只回卷一次：三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U1（取号写两轮第二、三两轮攻方没打中、撞号 0，每块盘只回卷一次攻过一轮没打中，副本量过）；用户定案 2026-10-19「罩，取号写两轮」、2026-10-20 维持，原话在变更史。
- 取号那一步的补写例外：三方判决 `research/prompts/m3-close-sysconf-r2-main-verification.md` 第四节、`research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U4 与第三节（克隆盘只写了单元那一形打中，推的；补的单元区扫描与环槽读错两样被攻过零轮）；用户定案 2026-10-19「开例外，四件闸」、2026-10-20 采两样补法，原话在变更史。
- 判不出盘面的离线出路：同一份第三轮判决第二节 U2（不带 `birth_txg` 闸时断言前一版丢已确认的写，攻方量过）与第三节；用户定案 2026-10-19「许，另加扫单元写序」、2026-10-20「离线出路加 birth_txg 闸」（零轮），原话在变更史。

**欠**：C79（系统配置与 journal 环的放置没人定）：journal 环两盘镜像那一支做成能强制进入的分支（系统配置每盘一份、整盘掉了两支已能强制进入，用例见 C79（系统配置与 journal 环的放置没人定） 行）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:202-234`（整段抄，未转述）**

```markdown
#### 已定项 9：系统配置的字段表

**定案**：字段表 = [layout/01-first-txn.md](../layout/01-first-txn.md)「一、mkfs 已经种下的」那一节里的「系统配置字段表」，46 行，合计 **489 字节**，住 4096 字节的系统配置槽（已定项 2），槽内余 3607。字段按**可改性**重新分段：

| 新段名 | 从哪几段来 | 字节 |
|---|---|---|
| **系统不可变配置** | 自举头里的 magic 4、格式版本 2、feature bits 96、fsid 16、写入者身份 20、校验和算法标识 1、本盘设备号 4、devs 4；加密预留全部 114；几何里除节点大小之外的全部 | 147 + 114 + 128 = 389 |
| **系统可变配置** | 几何里的节点大小 4 | 4 |
| **系统运行配置** | 可调值全部：T_time 4、T_dirty 8、整理低/高/停三水位 8×3 | 36 |
| **系统运行量**（不是配置） | 槽世代号 8、整槽校验和 32、journal tail 8、实例代号 4、回退下界 F 8 | 60 |

- 内容允许跨过 512（探测到的 `physical_block_size`）。跨过之后一次写可能落一半，由整槽校验和（覆盖整槽 4096 含补齐）判出，择槽规则（校验和过且世代号最大，已定项 16）自动回退到另一个槽。**可检测、可恢复。**
- 代价知情接受：崩溃点重放要多枚举一个撕裂维度；撕裂那次挂载拿到的是上一代的配置。
- 硬顶仍是槽宽 4096（`SYSTEM_CONFIGURATION_SLOT_BYTES`，格式常量、D15（格式冻结政策） 第 1 层）。超过它要 incompat 位。
- ⚠️ 现值 489 没跨 512，这条是**允许**不是**已经跨**。真加字段跨过那天，崩溃点重放那个维度与相关断言一起付。

- 加密预留三格：KDF 标识取 4 字节（不取 2）、主密钥槽内联进系统配置槽（取 80 字节最小可用值，不走间接目录单元）、加密类型留 1 字节声明。
- 第一版不留间接目录单元指针、不写设备描述符表，`devs` 留 4 字节，整理三水位恒 0（已定项 15）。
- 自举头有写入者身份 20（实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32）与校验和算法标识 1（0 无效、1 CRC-32C）；几何段有 mkfs 时的 `physical_block_size` 4 与扩展点声明值 N 4（宽度都按字节计）。读者遇到系统配置的格式版本不是 1，整池拒绝挂载，不退化成这一槽不可择，与已定项 16 第 1 句 C663（系统配置槽越界与不收之值两读者分类不同） 那一句一致（core 是 `crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 报 `FormatVersionNotRecognized`，`mountable_slot_or_refusal` 交回 `RecoveryFailure::SystemConfigurationValueRefused`）；遇到单元头的格式版本不是 1，拒收那一个单元。两样都不按现行的字段表往下解（系统配置那一半用户 2026-10-21 定「条文改成整池拒」，原话在变更史；单元头那一半主 agent 2026-09-27 定；池级 checker 判定已落地：单元头格式版本归 I-2.4（头校验和覆盖范围），系统配置格式版本归 I-7.13（系统配置池级字段在读者收的范围里），实审 A3-checker-2）。


**射程**：改新池新建文件的字节：**是**——mkfs、取号、暖机与发布写出的每个系统配置字节都由这张表定。字段的编码（字节序、枚举取值、位图位序）由已定项 13 / 17 定，feature bit 位分配由 D15（格式冻结政策） 已定项 4 定；槽的轮换规则见已定项 16。主密钥槽 80 字节的内部构成（盐、包裹格式）没定，进 D9（加密） 开加密时定。挡路条 ①/④ 后半句（池级单值 vs 每设备、incompat 位那条出路）的论证不挡字节（C181（池内异构判据漏了条带逐位异或这个轴）、C183（每套布局一个 incompat 位而盘上差不出字节））。E115（系统配置字段表的完备性与字节预算） 与 E124（系统配置字段表按 D18 已定项 14 重算） 的四格数把树表单元指针与单占的映射根指针都算进了槽里，E147（系统配置字段表按新池新建文件字节表重算） 之后不再引。内容是**池级**的，每盘存一份副本，两槽同内容；唯一的盘级字段是「本盘设备号 4」，不是每块盘能配不同的值。它还装着 60 字节**系统运行量**（槽世代号 8 + 整槽校验和 32 + journal tail 8 + 实例代号 4 + 回退下界 F 8），那不是配置：值随发布或抬 F 在变，用户从来不能设；其中 journal tail 与实例代号那 12 字节该不该住这儿归 C245（系统配置槽的写频率，两条条款说反话）。

**依据**：

- E147（系统配置字段表按新池新建文件字节表重算）：按字节表逐行重算，加密预留三格各取一个变体、三样都取最宽，全在一个扇区内——三格是编码取舍，不是预算取舍。
- 写入者身份、校验和算法标识、mkfs 时的 `physical_block_size`、扩展点声明值 N 四项各被一条已定条款或不变量逐字点名要求住系统配置：I-1.5（系统配置记管道身份） 与 D17（实现分层与第三方管道）「要还的债」第 3 项；I-2.2（校验和算法全盘一致） 与 C309（32 字节校验和的算法没定）；D13（验证路线） 已定项 5 与 D12（目标介质） 已定项 4 的 `physical_block_size` 比对；D21（权威态与派生态的分界） 已定项 8。前两项还有「事后加不进去」的性质：写入者身份是 mkfs 那一刻的事实，校验和算法标识一旦有第二种算法就要靠它分辨旧镜像。
- 用户定案（加密预留三格、四个字段），原话在变更史；四个字段没有走三方论证，可推翻。
- 用户定案 2026-09-26：系统配置加回退下界 F 8 字节（C419（F 生效之后靠什么保持不回落） 取 SysPre，D16（发布语义） 已定项 1「生效」），删掉回退见证表（D23（journal 的角色与格式） 已定项 14），原话在变更史；无实验：三方原型 `research/prompts/m2-rollback-forward-r3-opus-model/` 在冻结副本上量过 SysPre（第三轮判决 K4），没立实验号；F 放在字段表末尾、偏移 481 起是本规格按「追加在系统运行量末尾」取的，被攻过零轮。
- 用户定案 2026-10-21：系统配置格式版本不是 1 整池拒（选项「条文改成整池拒」），原话在变更史；起因是条文原写拒收那一槽，与 `crates/singlefs-core/src/recovery.rs` 的实现和已定项 16 第 1 句对不上（`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 560 行）。

**欠**：C178（系统配置必需字段表靠手抄）；C180（设备身份比对没有第二个独立来源）；C181（池内异构判据漏了条带逐位异或这个轴）；C183（每套布局一个 incompat 位而盘上差不出字节）；C218（设备表单元取哪个码没定）；C308（自证单元的字段偏移没定）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:235-255`（整段抄，未转述）**

```markdown
#### 已定项 26：系统配置的三档可改性

**定案**：系统配置按「改了要付什么」分三档，每个字段归且只归一档：

| 档 | 段 | 改的代价 | 谁能改、什么时候 |
|---|---|---|---|
| 系统不可变配置 | 389 字节 | 重建文件系统 | mkfs 时设：fsid 由 mkfs 的对外入口从随机源生成、前 8 字节随机，内层入口可给 fsid；journal 环长、单元大小、落点粒度、devs 由用户作 mkfs 参数给；之后不可改 |
| 系统可变配置 | 4 字节（节点大小） | **重建索引，不丢数据**（一次满量级全盘 I/O） | mkfs 时设；改它要走重建索引那条路，第一版没有这条路 |
| 系统运行配置 | 36 字节 | 无 | mkfs 时设默认值；**每次挂载可重设，而且要回写盘上** |
| （系统运行量 60 字节不在三档里：用户从来不能设，文件系统自己维护） | — | — | — |

**射程**：管的是「改了要付什么」这根轴，不是态别（权威态 / 派生态在 D21（权威态与派生态的分界） 已定项 9）。系统运行量那 60 字节不是配置，不归这三档：四类只差在两根轴上——用户能不能改、值什么时候变；前三类值只在有人改它时才变，系统运行量每次发布都在动。⚠️ 名字里没有「不可变」三个字是有意的：用户 2026-09-20 一度取名「系统运行不可变量」，而它的值恰恰一直在变（槽世代号每写一次 +1、journal tail 每次发布都涨），按 `code-discipline.md`「一个语义概念全仓一个名字」那条，同一个「不可变」在两处指两件事会让人把它当常量缓存。

**依据**：

- 无实验：三档是按「改了要付什么」给字段表里已有的字段贴标签，分类本身没有可量的量；每一档的代价由各自的条款给——节点大小那一档是 D8（核心索引结构） 已定项 2 射程逐字「改它 = 重建索引，不是重建文件系统」，其余字段没有逐条依据，那正是 C426（几何段逐字段的可改性没有逐条依据） 欠的。
- 用户定案 2026-09-20，原话在变更史。
- fsid 由对外入口从随机源生成、内层入口可给：三方判决 `research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S1 生成那一格与第三节；用户定案 2026-10-20「对外生成，留内层入口」，原话在变更史。

**欠**：C422（挂载选项通道不存在）；C423（回写算不算一次发布）；C424（生效配置不可见）；C426（几何段逐字段的可改性没有逐条依据）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:256-269`（整段抄，未转述）**

```markdown
#### 已定项 10：区写指针不得作恢复判定的权威上界

**定案**：**区写指针（WP）与本工程认定的提交点之间没有任何必然的代数关系**，因此恢复判定不许把 WP 当成「提交点最多推进到这里」的权威上界。

**射程**：已定项 10 只排除「WP = 恢复权威上界」这一种用法，**不断言 WP 在恢复里毫无位置**——「只做寻址提示」那一条仍在未定项 6 里开着。也**不回答真实崩溃语义**：`zone_full` 造的是一个静态终态，崩在写到一半仍要崩溃点重放 harness。⚠️ **引用已定项 10 时连这条射程一起引**——只引标题那半句会读成「WP 与恢复无关」，而那不是已定项 10 说的。已定项 10 与已定项 11 都是排除式条款，不承诺任何盘上字节。

**依据**：

- E125（顺序区写指针之上读到什么）：`zone_full=1` 一个设备参数就造出「本工程认定的提交字节数为 0，而区写指针已在区末尾」。
- 外部文档（未在本项目验证）：本机 `~/code/fs-refs/linux-6.17/Documentation/filesystems/zonefs.rst` 第 236–242 行逐字「the partial failure of a multi-BIO large write operation will cause the zone write pointer to advance partially, even though the entire write operation will be reported as failed to the user」——正常错误路径上就会发生，不必等崩溃。
- 三方：未定项 6 第一轮三条腿一致判「作恢复的权威上界」臂出局（`d22-item6-local-output.md`、`d22-item6-forward-sonnet-output.md`、`d22-item6-reverse-opus-output.md`），第二轮把证据加厚。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:270-287`（整段抄，未转述）**

```markdown
#### 已定项 11：区写指针不进 O2（独立解析器 + checker） 的判定输入

**定案**：D13（验证路线）「O2（独立解析器 + checker） 判的是**一个**镜像」那条定案逐字（**整句**）：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

区写指针是设备维护的带外状态，**不在镜像里**；把它接进 checker 等于给 O2（独立解析器 + checker） 一个定义域之外的第二输入——而最后那句正是把这一步封死的那半句。⚠️ **它必须整句引**：切在逗号处、只抄到「一元谓词」为止，丢掉的恰恰是「任何需要第二个输入……都不属于它」这句承重的。

**射程**：已定项 11 只排除「WP 进 O2（独立解析器 + checker）」，**不排除运行时 / 挂载时的交叉检查**——那是不同的检查主体，仓里没有评估过。

**依据**：

- 无实验：这是一条类型论证（一元谓词收不进第二个输入），实验既加不了分也减不了分。
- 三方：未定项 6 第一轮三条腿一致判「进 checker 作交叉校验」臂的 checker 形态出局（输出文件同已定项 10）。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:288-303`（整段抄，未转述）**

```markdown
#### 已定项 12：系统配置不带自己的树表单元指针

**定案**：**树表的权威在根记录**（已定项 7 逐字「**根记录恒走间接层：固定头 + 一条指向「树表单元」的指针，树根一律住树表单元，不平铺进根槽。**」），**系统配置不带第二条指向树表单元的指针**。

**射程**：已定项 12 只说系统配置不带**指向树表单元的指针**。它**不**改树表单元本身的形态、不答「系统配置该有哪些字段」（那是已定项 9）、也**不**解 C212（固定结构的写小于 io_min）。它连着三格空账，都不是「已定项 12 错了」，是「已定项 12 之下这几格空了」：C246（树表单元没有冻结层归属）、C247（树表单元与实例表单元的副本数从没枚举过）、C248（树表单元的自举豁免没写过）。树表表长挪进密文之后，那一条明文泄漏是减了还是增了，没有量过。

**依据**：两条互相独立的反向证据，任一条单独就足以否掉「系统配置要带」——

- **发布顺序里没有系统配置这一步**：D16（发布语义） 已定项 7 的持久顺序是一条封闭的序列，系统配置槽写是发布序列**之后**的最后一步、进崩溃点重放枚举但不加序点；给系统配置一条随每代变的树表指针，就得给那条顺序加第四个序点，而那是一条要重开的用户定案。
- **回退语义**：系统配置不随根回退，而树表指针是随根走的量 ⇒ 一条住系统配置的树表指针在回退之后指向的是另一条时间线的单元。
- 旁证（主 agent 现查坐实）：树表单元本身也是密文（D9（加密） 逐字「除系统配置外全部加密」），给无密钥侧一条明文指针读不出任何条目，而 D9（加密） 已定项 5 取「只做 scrub」、同节写明这条路把「将来支持无密钥只读挂载」焊死。
- 同形先例：「这个量住系统配置还是住根记录」本工程判过一次（树 ID 水位，决策变更史 2026-09-06（其三十六）），判给了根记录。
- 无实验：这是一条按发布顺序与回退语义推出的结构论证，没有可量的量；三方：反推腿被明确要求去顶「不要」这个倾向，两次独立抽样十条路径都构造不出（`system_configuration-treetable-reverse 那一轮的腿报告（产物在 git 里）`、`system_configuration-treetable-reverse-opus-s2-output.md`），满足 `.claude/rules/three-way-inference.md` 对「没打中」要抽两次样的要求。

**欠**：C246（树表单元没有冻结层归属）；C247（树表单元与实例表单元的副本数从没枚举过）；C248（树表单元的自举豁免没写过）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:304-316`（整段抄，未转述）**

```markdown
#### 已定项 13：盘上定长字段一律小端，枚举取值各有登记位，位图位序统一

**定案**：盘上所有定长多字节字段（系统配置、根记录、树表条目、单元头、journal 记录头、位置条目、各类 key / value）一律小端。枚举取值不集中一张总表，各有自己的登记位：单元类型码在 D18（块里携带什么信息） 已定项 11 的单元类登记表，树的种类码在 D8（核心索引结构） 已定项 9，feature bit 位号在 D15（格式冻结政策） 已定项 4，加密类型的取值随 D9（加密） 开加密时定，livelist 事件类型的取值在 D6（快照实现模型） 已定项 2（ALLOC / FREE）。位图位序：feature bit 与其余位图的位 n 住第 n div 8 个字节的第 n mod 8 低位（已定项 17）。

**射程**：大端不取：E142（新池新建文件的干跑） 装置与 `crates/singlefs-core/src/bytes.rs`（`to_le_bytes`）都按小端写，改成大端要改写全部 put 与产物。其余登记位见已定项 17。

**依据**：

- 无实验：字节序与登记位是约定，没有可量的量；取小端是因为已有的装置与实现都按小端写。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:317-329`（整段抄，未转述）**

```markdown
#### 已定项 14：devs < R 时 I-7.6（根环区域落在互不相同的盘上） 判什么

**定案**：I-7.6（根环区域落在互不相同的盘上） 改成：R 个区域的设备身份取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域；devs < R 时不拒绝挂载，容错承诺按盘数降级（2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。R = 3 不动。

**射程**：字节不动，只改不变量与 checker；I-7.6（根环区域落在互不相同的盘上） 由池级 checker 判。

**依据**：

- E87（固定结构的放置）：2 盘 R = 3 上掉盘 1 最坏回退 1 代、掉盘 0 最坏 2 代——两盘上 R = 3 必有一盘背两个区域。
- 用户定案（F = 2 见已定项 2；按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定），原话在变更史；没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:330-343`（整段抄，未转述）**

```markdown
#### 已定项 15：系统配置字段表的四格连带

**定案**：第一版不留间接目录单元指针（显式留白，不是留位），设备描述符表第一版不写，D12（目标介质） 已定项 5 那句改成「第一条线两盘同构、第一版不写；加第二套布局前必须写」，并写明 D12（目标介质） 已定项 4 的容量 / 布局位 / physical_block_size 三条比对在第一版退化成各盘互比；`devs` 4 字节留；整理三水位 24 字节恒 0 = 内置默认。已定项 9 的字段表随之：去掉 59、加「单元区起始槽号」8（D3（空间分配） 已定项 10）、「mkfs 时的 io_min」4 与「固定结构槽距」4（D2（RAID 条带策略） 已定项 19）。

**射程**：只定这四格；系统配置其余字段见已定项 9。

**依据**：

- E147（系统配置字段表按新池新建文件字节表重算）：不留间接目录指针的变体在一个扇区内。
- 那条指针 59 字节没有出处，按 D19（块指针的结构与宽度预算） 已定项 7 / 8 的指针宽算要 83。
- 用户定案（按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定），原话在变更史；没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**欠**：C218（设备表单元取哪个码没定）；C180（设备身份比对没有第二个独立来源）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:344-370`（整段抄，未转述）**

```markdown
#### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位

**定案**：系统配置槽：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距（D2（RAID 条带策略） 已定项 19），槽世代号从 1 起（mkfs 写下的两槽世代号都是 1）、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1）。读槽 1 的偏移按这个次序取：本盘槽 0 可用（自证过、几何字段都在这一版读者收的范围里）时取它自述的槽距；否则取池里按设备次序第一块可用槽 0 自述的槽距；全池没有可用的系统配置槽 0 时，读者在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过同一读者收的根环起点减 4096），只收自证过、且自述的固定结构槽距等于所在偏移的槽，几档都收得到时取世代号最大的，世代号相同取偏移小的；逐档读到自证过而池级字段越界的槽整池拒（第 5 句「越界即拒」），incompat 位不认得的槽不可择、不核偏移。逐档读到自证过的槽先判它带不带这一版读者不收之值（含 I-8.1（环几何够大） 三个 journal 环几何字段），带的整池拒、不再核偏移；不带的再核自述槽距等于所在偏移。根环：`r × P × chunk` 是设备内偏移，基址 = 系统配置「根环起点」字段（第一版 1 MiB），区域 r 的起点 = 基址 + r × P × chunk；区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定，升格进已定项 2 的参数表。五句写死：

1. **系统配置「根环起点」字段是 16 KiB 槽号**，与 D3（空间分配） 已定项 7 的落点槽号、D19（块指针的结构与宽度预算） 已定项 4 的位置条目物理偏移同一个坐标系；第一版写 **64**（= 1 MiB ÷ 16384）。基址（设备内字节偏移）= 该字段 × 16384；区域 r 的起点 = 基址 + r × P × chunk。⚠️ 字段里写的是 64，不是 1048576。读者解槽时读这个字段与 journal 环起点字段（偏移 325），与第一版常量（64、1024）不等一律整池拒；根环区域数 R（偏移 361 那 1 字节）同样读，不等于 3 整池拒，每区槽数 S 越界照已定项 2 整池拒；一槽 incompat 位不认得而格式版本又不是 1 时先按格式版本整池拒，不当不可择（C663（系统配置槽越界与不收之值两读者分类不同），用户 2026-10-20 定，原话在变更史；core 是 `crates/singlefs-core/src/recovery.rs` 的 `RootRingRegionCountNotTheFirstVersionValue` 与 `mountable_slot_or_refusal`，checker 是 `crates/singlefs-checker/src/image.rs` 的 `value_refused_by_this_reader`）；判固定结构槽距的上界用同一槽自述的根环起点，checker 的 I-7.13（系统配置池级字段在读者收的范围里） 与 core 收同一张表（用户定，起因与依据在代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`「core 与 checker 对系统配置几何字段的读法」那一格）。已实现：core 的 `crates/singlefs-core/src/system_configuration.rs` `parse_slot` 按偏移 325、371 读进 `SystemImmutableConfiguration` 的 `journal_ring_start_slot`、`root_ring_base_slot`；`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 择系统配置时判，不等报 `SystemConfigurationValueOutsideWhatThisReaderAccepts::RootRingBaseNotTheFirstVersionSlot` / `JournalRingStartNotTheFirstVersionSlot`，槽距上界取 `largest_fixed_structure_slot_spacing_under_the_root_ring_base`（根环起点 × 16384 − 4096；逐档找槽 1 那一处还没有哪一槽可读，仍用第一版常量）；checker 的 `crates/singlefs-checker/src/image.rs` `geometry_of` 判同一张表（325、371 等于第一版常量，417 = 环长现算的起点且在 64 槽段边界上）。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write`（攻方五格只读与可写都拒、盘上不变）；`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`；两边在同一份镜像上一起判的是 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs`（五格各一条：core 两种挂载都整池拒、盘上不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 并点名它拒的成员；根环起点写 0 那一格两边报的成员不同——core 先判槽距上界、报槽距越界，checker 先判两个起点、报根环起点）（`m2-impl-r2-fixes-b-implementer-report.md` 第三节第 2 行、`m2-impl-r2-fixes-a-implementer-report.md`「结论」第 1 行）。
2. **根槽序号 j 的设备内偏移 = 区域起点 + j × 固定结构槽距**（槽距 = max(4096, mkfs 时探测的 io_min)，D2（RAID 条带策略） 已定项 19）。按已定项 2 的字面取槽距 512 也合规，而字节表要新池新建文件的根落在 1 MiB + 4096，这一句把序号到偏移的换算钉死。
3. **P = 3、chunk = 1 MiB 是第一版的 mkfs 常量**；mkfs 要校验 `基址 + (R−1) × P × chunk + S × 槽距 ≤ 设备可用字节`，越界拒绝 mkfs。取 3 是第一版两盘下满足「P 素数且 P > devs」的最小素数，chunk 取 1 MiB 与基址同档 ⇒ 三个区域起点落在 1 / 4 / 7 MiB，与 [layout/01-first-txn.md](../layout/01-first-txn.md) 的字节表一致。⚠️ **「P > devs」那条约束已经没有理由**（它唯一的理由是 `gcd(P, devs) = 1` 的双射，已随 D2（RAID 条带策略） 已定项 7 把归属改成 mkfs 逐区域写下的设备身份而作废），账在 C217（P 大于设备数这条约束的理由没了而约束还在）；取 3 不依赖那条约束。
4. **根环三个区域的设备归属第一版写死 0 / 1 / 0**（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域把设备身份写进系统配置的「根环逐区域设备身份」12 字节（D2（RAID 条带策略） 已定项 7）。⚠️ **它不是 mkfs 参数**：第一版 devs = 2、R = 3，按 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界（取满 min(R, devs) = 2 个不同值、没有一块盘背超过 ⌈3/2⌉ = 2 个区域），只有「两个区域在一块盘、一个在另一块」这一种形状，区域编号一定之后 0 / 1 / 0 是唯一的规范写法。把它做成 mkfs 参数，买到的是「哪块盘背两个」这一格自由度，而那一格在两盘同构下没有可观测差别，代价是多一个要在挂载时校验、要在加盘时重算的参数。
5. **暖机空发布 2 次、新池新建文件 txg 3 是格式常量**，不是运行时谓词（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、NEW_POOL_FILE_CREATION_TXG = 3，登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 八）。D16（发布语义） 已定项 8 要求「推到本实例的根覆盖两块盘为止」，而在 0 / 1 / 0 的归属下 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定，写成常量比写成运行时谓词少一条只有一个取值的分支。⚠️ 归属改了这两个常量要重算（加盘或开条带表时）。`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（mkfs → 取号 → 暖机两次 → 新池新建文件，同一个进程里那一串），不管任何池的第一个文件版本：只做过 mkfs 的池重开之后再写第一个文件版本，txg 取现行根 + 1。读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前；`physical_block_size` ∈ [根记录宽 466, 槽距]。越界即拒：checker 对越界槽报 I-7.13（系统配置池级字段在读者收的范围里）、该池其余不变量报不适用，实现整池拒挂载（用户 2026-09-27 定「整池拒」）。

**射程**：把区域公式换成「mkfs 逐区域写起点」那条留到 zoned 开线前再走三方。全池没有可用槽 0 时按第一版常量 4096 读槽 1 的那一读（用户 2026-09-14 定的「甲」）由用户 2026-10-02 推翻、改成逐档，原话在变更史；「世代号相同取偏移小」那一句在 mkfs 清零 [0, 根环基址)（已定项 8 第 3 条）之后只在介质损坏造出的残留上有对象。池级 checker、恢复与 GPU 判器的两份按同一句读（[layout/01-first-txn.md](../layout/01-first-txn.md) 一「系统配置槽 0 与槽 1 的位置」那一段）。

**依据**：

- E142（新池新建文件的干跑）：按这套取法写出新池新建文件的全部固定结构字节（系统配置槽 0 / 4096、根环 1 / 4 / 7 MiB、新池新建文件的根落区域 0 槽 1），与 `crates/` 实装按 21 个区域比对全等（根记录、journal 记录与系统配置逐字节全比）。
- E87（固定结构的放置）：两盘上 R = 3 必有一盘背两个区域——0 / 1 / 0 是唯一规范写法的那一半。
- 逐盘计那一读法：C322（取号那一步的屏障怎么放没有条款） 三轮三方之后的用户定案（全池最大 + 1 会让滞后盘一次跳 2、落回它择到的那一槽），判决在 `research/prompts/c322-r3-main-verification.md`。
- 用户定案（按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定；第 4 / 5 句是之后的弹窗定案），原话在变更史；除逐盘计那一句，都没有走三方论证，可推翻。
- 用户定案 2026-09-23：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 那条流（`research/prompts/m2-refusals-presumed-r1-main-verification.md` 第三节判它射程的那一格），原话在变更史。
- 用户定案 2026-10-01：「mkfs 两槽世代号都是 1」写进这一项，I-7.16（系统配置槽世代号与槽号的关系） 一并判上（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节第 1 条），原话在变更史。
- 全池没有可用槽 0 时逐档、平局取法与两个出口：三方判决 `research/prompts/m3-c651-r1-main-verification.md` 第二节（常量与整池拒两读出局、逐档读法站住、平局取法站住）与第四节第 1、2 条，`research/prompts/m3-formal-r6-main-verification.md` 第二节条款句两个出口那一格；用例 `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`（恢复与池级 checker 在同一份镜像上判逐档、自述槽距不等于偏移不收、取世代号最大、同号取偏移小、越界整池拒、incompat 不认得择不到）。
- 逐档先判不收之值、再核偏移：代码轮判决 `research/prompts/m3-formal-code-r2-main-verification.md` 第二节「K3-a」一行（checker 改前先核偏移，丢掉的槽 I-8.1（环几何够大） 不判，与实现分叉；第 13 批改成与实现同序）与 `research/prompts/m3-formal-code-r3-main-verification.md` 第二节「K3」一行（辩方 Z5：条款缺这一句的字面）。
- 用户定案 2026-10-02：逐档连同前两步与平局取法写回；全池没有可用槽 0 时改成逐档，推翻 2026-09-14 收尾弹窗定的「甲」（按最小槽距 4096 读槽 1），原话在变更史。
- 根环区域数 R 读并整池拒、每区槽数 S 越界整池拒、incompat 不认得与格式版本不是 1 同在时格式版本先：用户定案 2026-10-20（C663（系统配置槽越界与不收之值两读者分类不同）），原话在变更史。

**欠**：C217（P 大于设备数这条约束的理由没了而约束还在）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:371-383`（整段抄，未转述）**

```markdown
#### 已定项 17：已定项 13 的登记位清单补全

**定案**：journal 记录类型的登记表在 D23（journal 的角色与格式） 已定项 1（0 无效、1 普通记录）；统计量标签在 D5（快照 / 空间记账机制） 已定项 10；算法类型在 D9（加密）（0 未加密、1 ChaCha20-Poly1305、2 AES-256-GCM）；树表条目 / 单元头的 flags 各自条款写「第一版恒 0、非 0 拒收、位号表空」，根记录的 flags 位 0 是卸载记号、其余位同样恒 0（已定项 7）；位图位序（位 n 住第 n div 8 个字节的第 n mod 8 低位）抄进已定项 13。

**射程**：只登记取值住哪；新池新建文件写的值不变。

**依据**：

- 无实验：登记工，没有第二种选法；算法类型的两个非零码照 D9（加密） 已定项 3「两种算法都必须实现」的顺序给。
- 用户定案（按 [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第九节的选项逐条定），原话在变更史；没有走三方论证，可推翻。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:384-396`（整段抄，未转述）**

```markdown
#### 已定项 18：分界线

**定案**：**分界线：一个单元落盘之后，是否有一个带它校验和的父指针才被发布。** 是 ⇒ COW + D4（校验和位置） 已经够，撕裂无危害（已定项 19、已定项 20）；否 ⇒ 必须自证，且自证要两样同时在场（已定项 21）。

**射程**：分的是单元。有父指针那一侧是普通单元；没有的那一侧是根、系统配置与 journal（已定项 23、已定项 24）。分界线不往上挪（已定项 22）。

**依据**：

- 无实验：分界线是按「有没有带校验和的父指针」分类的结构论证；撕裂形态本机造不出（已定项 19）。
- 三方论证（主 agent 裁决）没有腿况记录：`research/prompts/` 下查不到对应的提示与本地腿输出，而 `.claude/rules/three-way-inference.md` 明写「本地腿缺席时必须显式报告」⇒ 「三条腿都去找反例、都没找到」这句降级为**未记录腿况的阴性结果**，要拿它支撑新结论先补跑。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:397-410`（整段抄，未转述）**

```markdown
#### 已定项 19：有父指针的普通单元，COW + D4（校验和位置） 足够，且不依赖任何设备原子性假设

**定案**：机制：校验和住在**另一个单元**里（D4（校验和位置）），且父在子持久之后才发布（D16（发布语义））。子块的任意字节子集是旧 / 新 / 零 / 垃圾，都与父里的校验和失配。⇒ **撕裂粒度是 512、4 KiB 还是整块，对这个论证毫无影响。** **这正是 D4（校验和位置） 相对「块内自带校验和」的实质优势——不是省字节，是把检测能力从设备假设上摘了下来。**

**射程**：⚠️ **D22（单元原子性怎么合成） 已定项 19 答的是撕裂检不检得出来，不是崩溃之后还有没有东西可以退回去。** 它证的是「子块的任意字节子集与父里的校验和失配」，这一步不依赖设备原子性；而**旧版本还在不在盘上是另一条条款**——D22（单元原子性怎么合成） 已定项 4（「本事务内释放的块不得重用」I-7.4（近 K 代块未被复用）存在）管的正是那一半，缺了它的失败模式逐字是「**可检测（父指针校验和失配），不可恢复。**」⇒ **引用 D22（单元原子性怎么合成） 已定项 19 时连这句一起引**：只引标题那半句「COW + D4（校验和位置） 足够」，会把回退候选那一半整条丢掉，而那正是 `.claude/rules/three-way-inference.md`「引 kb 里的条目要整行抄」记着的那种摘句事故。

**依据**：

- 无实验：撕裂形态本机造不出——`dm-flakey` 只有 `drop_writes` / `error_writes` / `corrupt_bio_byte`，造不出「前半落盘后半没落」；崩溃点重放把撕裂态并进「没持久」、不枚举撕裂子集（D13（验证路线） 已定项 4）。
- 三方论证没有腿况记录（已定项 18）。
- 现役对照（逐字核实，未在本项目验证）：btrfs 的 `struct btrfs_key_ptr { btrfs_disk_key key; __le64 blockptr; __le64 generation; }` 没有校验和、csum 在节点头内，一个自指的校验和分不清「这个块是本代的」与「这是某一代的合法块」，所以 btrfs 被迫另造 transid 对拍；ZFS 的 `blkptr_t` 有 `blk_cksum`（256 位），普通块没有第二道闸 ⇒ **两家的差别正好落在「校验和放哪」这一个变量上；D4（校验和位置） 一次买到 btrfs 两道闸的能力。**

**欠**：C16（管道声明失败模型例外）：撕裂注入不能靠 dm-flakey，只能在崩溃点重放 harness 里截断写请求造。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:411-428`（整段抄，未转述）**

```markdown
#### 已定项 20：撕裂粒度变大不是让问题变严重，是让它退化成丢失写

**定案**：

| 撕裂结果 | 父指针校验和（D4（校验和位置）） | 块内自带校验和 |
|---|---|---|
| 全新 | 通过（正确） | 通过（正确） |
| **混合** | 抓住 | 抓住 |
| **全旧** | **抓住**（父里是新校验和） | **抓不到**——这是丢失写，D18（块里携带什么信息） 已判死 |

**射程**：只比检测。它是 D18（块里携带什么信息） 的直接推论：撕裂粒度变大就退化成丢失写，而丢失写块里带什么都抓不到。

**依据**：

- 无实验：三行是按校验和住哪推出的真值表，撕裂态本机造不出（已定项 19）。

**欠**：C16（管道声明失败模型例外）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:429-446`（整段抄，未转述）**

```markdown
#### 已定项 21：原地覆写的结构必须「整单元校验和 + 被实际检查的世代号」，两样缺一不可

**定案**：只有校验和不够。一条**完全没落盘**的记录，读到的是同一位置上一圈遗留的**完整合法旧记录**——校验和通过，而它是陈旧的。⇒ 校验和答「这个单元完不完整」，世代号答「它是不是本代的」，**两个问题，两个字段。** 本工程不做 XFS 那种 per-sector 盖章（轴②）。

**射程**：

<!-- doc-registries:not-field-frequency 说的是否决掉的 XFS 按扇区盖章，不是这个格式的字段 -->
- ⚠️ **「要不要 XFS 那种 per-sector 盖章」是两个正交的轴，不是一个问题**：轴① 是序号的**身份**（圈数 cycle 还是事务号 txg；XFS 的 LSN 是 `(cycle:32, block:32)`，圈数是把序号与位置压进同一个 64 位的产物），与序号位宽有关；轴② 是序号的**粒度**（每条记录一份，还是每 512 字节一份），与序号位宽**无关**。**「用 txg 取代 cycle」只动轴①，轴②原封不动。**
- 不做轴②的理由是「2⁻³² 够小，且换 64 位校验和能更便宜地压到 2⁻⁶⁴」，**不是「有 txg 了所以不需要 cycle」**——后者是范畴错误。这条否决理由的比较集不完整：盖章还买到免对齐（判定分辨率是扇区时，多条记录挤同一个原子单元照样判得了撕裂），对齐代价 E23（journal 几何） 量过。**不当场重开**，但两条前置补齐之前不许把「不做轴②」当成已经论证充分的：① 目标负载的 fsync 粒度——D25（目标负载优先级） 已定取粗粒度，方向对「不做轴②」有利，这一条已满足；② 崩溃点重放实测撕裂边界是不是真落在扇区倍数上（「设备原子宽 512 ⇒ 各扇区独立新/旧」依赖这一点，从未被实测）。
- ⚠️ **方法论标注**：撕裂形态的推理**在本机无法证伪**，那一整块记为推理，不是结论；证伪要等崩溃点重放 harness 造得出撕裂态。⚠️ **射程是撕裂形态，不含未定项 6**（写指针领先于提交点不是撕裂）。

**依据**：

- 无实验：「没落盘的记录读回上一圈的合法旧记录」是原地覆写的结构事实，撕裂形态本机造不出（已定项 19）。
- 轴② 的得失：设备原子宽 512 ⇒ 崩溃后一条记录的各扇区是独立的新/旧，陈旧扇区必带上一圈的章 ⇒ 确定检出；只有整条校验和 ⇒ 漏检概率 ≈ 2⁻ⁿ（n = 校验和位宽）。代价是章住在扇区内部：要么做 XFS 那种影子备份（`h_cycle_data[]`，header 随记录长度增长），要么每 512 字节留 8 字节（浪费 1.6%，变长记录的偏移计算全部变复杂）。XFS 自述撕裂检测归 CRC，cycle 在 XFS 里的实际用途是二分找 head——那是它的政策，不是结构上的不能（外部事实，未在本项目验证）。

**欠**：C16（管道声明失败模型例外）：撕裂注入。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:447-459`（整段抄，未转述）**

```markdown
#### 已定项 22：分界线不能往上挪：根环的回退候选会变假

**定案**：**把有父指针的单元也改成「地址稳定 + 原地覆写 + 自证」，会当场打掉根环。** I-7.4（近 K 代块未被复用） 要求回退候选集里每一个根所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头，否则环里旧根指向的块已被覆盖、作为回退候选是假的（已定项 25）。**原地覆写正是「那些块已经被覆盖」** ⇒ 环里除最新那一代以外的根全部变成假候选。⇒ 要让环里的旧根仍是真候选，就得保留旧版副本，**而且要保留到块复用的界为止**（`max(F_生效, 环里最旧有效根)`，D16（发布语义） 已定项 1）。于是原地方案付的是 COW 的延迟重用空间代价**加上**拷贝写，而且这笔账**在零快照时同样成立**——它不是快照功能的代价，是根环的代价。

**射程**：**这一条对原地方案的最强形态成立**（元数据自证走已定项 21、地址一次分配永不变、父指针只在分裂合并时改），因此不依赖 D4（校验和位置） 是否保留。⚠️ **原地方案在一根轴上确实赢，不许当它不存在**：写放大这根轴上原地是赢的（E38（记账结构 COW 还是原地） 的原地臂不重写祖先），杀掉它的是根环，不是写量。同一轮里被打掉的五条理由够不着最强形态、不许再用：原地也要重写整条 Merkle 脊柱；数据侧仍要 COW 所以是两套机制；崩溃点集合爆炸；D9（加密） 的 nonce 重用是硬墙；D5（快照 / 空间记账机制） 的 birth 判据直接失效（各自为什么够不着，原表在决策变更史）。

**依据**：

- 无实验：这是一条按 I-7.4（近 K 代块未被复用） 与块复用的界推出的结构论证，没有可量的量。
- 三方论证（主 agent 裁决），仓里只留了本地腿的输出 `cow-necessity-local-en-output.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:460-476`（整段抄，未转述）**

```markdown
#### 已定项 23：journal 不可能被 COW 覆盖：对定长环成立，落在自证那一类

**定案**：**对定长环成立**：要 COW 一条 journal record，就得有指针指向它；那个指针要提交；而提交正是 journal 要办的事。循环。且非环形 journal 若每条 record 都要一次根写，**journal 的全部收益（摊薄根写）就归零**。⇒ **journal 落在已定项 21 那一类**：它需要整单元校验和 + 被实际检查的世代号。⚠️ 由此对 journal 的前缀规则多一条硬要求：**世代号必须在格式里，且必须被重放路径实际检查。**

**射程**：

- 链式日志另有出路：ZFS 的 ZIL 不是定长环，每块的 trailer 里带一个 `blkptr` 指向链上下一块，链头存在 `zil_header_t.zh_log`，循环因此被「指针住在前一条记录里」打破（外部事实，未在本项目验证）；本工程取定长环（已定项 1）。
- 世代号是 `jsn`：D23（journal 的角色与格式）「世代号：记录粒度的全局单调号，不是 D5（快照 / 空间记账机制） 的 txg」逐字把世代号指给 `jsn`、并排除了 `checkpoint_txg`（同一窗口内共享同一个 txg，分不出先后）；D23（journal 的角色与格式） 已定项 14 的记录水位用的是下界，不构成兑现。落点也不是 D16（发布语义） 已定项 3（journal 的冻结层）。
- 兑现到哪：`jsn` = 实例代号 32 位 + 计数器 48 位在格式里（D23（journal 的角色与格式） 已定项 9）；重放路径 `crates/singlefs-core/src/recovery.rs` 的 `replay_journal` 按实例内计数器断号即止、跨实例边界即停（D23（journal 的角色与格式） 已定项 14 的注）；checker 侧承担它的 I-8.3（重放前缀严格连续） 未实现。

**依据**：

- 无实验：定长环上的循环是定义上的，没有可量的量。
- 三方论证没有腿况记录（已定项 18）；世代号那条硬要求兑现到哪，是一轮三方复核（C199（实例代号递增与 jsn 断号即止互相矛盾） 的出处）现查的。

**欠**：C199（实例代号递增与 jsn 断号即止互相矛盾）：代号递增与断号即止的矛盾已收口，剩崩溃点重放两格与 I-8.3（重放前缀严格连续） 的 checker。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:477-492`（整段抄，未转述）**

```markdown
#### 已定项 24：根落在同一类，且两类布局需要的是同一种机制

**定案**：**根没有父，所以它把 COW 的全部防护一次性丢光**——校验和住在父里、原子发布就是父的那一次写，两样都没了。⇒ 根必须自证：**整单元校验和（自证完整） + 代号（自证是本代） + 槽轮换（不覆写唯一副本）。** 「能原地覆盖根指针」那一类**实际上也不能覆写唯一副本**——一次撕裂就抹掉唯一见证者，它只能在固定槽组里轮换并带代号；zoned 那一类根会漂移，需要一套「最新的根在哪」的发现程序。⇒ **两类的差别只在「怎么枚举候选」，不在「候选长什么样」。** **读取次序是要求，不是实现细节**：必须**先逐个验证全部候选、再在有效者中按代号择新**；不许「先按代号挑最新、再验证」——后者在最新槽撕裂时直接失败，而不是回落到上一代。择新键 (`checkpoint_txg`, 实例代号) 完全相同的不止一条时，取读根环次序里最前的那条；池级 checker 的择根照同一读法。

**射程**：它推翻 D12（目标介质） 一处措辞的读法（「能原地覆盖根指针」那一类）；对 D16（发布语义） 的缺口是一次收窄：缺的是一种机制，不是两种。zoned 那一类的发现程序从没设计过，是 D22（单元原子性怎么合成） 未定项 6 悬着的那张欠条。`crates/singlefs-core/src/recovery.rs` 的择根是三个区域全部槽里自证过、`(checkpoint_txg, 实例代号)` 最大的那一条，键相同取读根环次序里最前的（择根核 `choose_root_slot` 与 `newest_root_among` 同一个取法）。池级 checker 的择根（`crates/singlefs-checker/src/walk.rs`）已是同一个取法：逐个比、只在严格更大时才换，键相同取读根环次序里最前的一条。

**依据**：

- 根没有父是结构事实，读取次序是逻辑推论。
- E165（恢复重放有界穷举与变异杀伤界）：「先逐个验证全部候选、再在有效者中择新」这一读取次序，改成取第一个有效者（择了一个自证过的槽就不再换）的变体在合法历史 K、K9、KW 三族界内红（第 3 次重跑第三段）。
- 现役参照（未在本项目验证）：ZFS 的「全扫所有 label 所有槽再按 txg 择新」正是这个次序。
- 键完全相同取前一个：代码轮判决 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节「键完全相同的槽取哪个」一格（核取前一个，`newest_root_among` 当时取后一个、与它的文档自相矛盾；合法历史里没造出键完全相同的两条）与「改法落地」一节（`newest_root_among` 改成与核同一择法）；池级 checker 同一读法：代码轮第二轮判决 `research/prompts/m3-formal-code-r2-main-verification.md` 第二节辩方「checker 择根平局取后一个」一行（打中同形别臂，`walk.rs` 那一臂取后一个）与第三节第 2 条。
- 三方论证没有腿况记录（已定项 18）。

**欠**：C88（根环的时间线判别未实现）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:493-512`（整段抄，未转述）**

```markdown
#### 已定项 25：根环深度是块重用延迟的上界，两者不是同一个数

**定案**：保留 K 代根 ⇒ **最近 K 个事务里释放的空间都不许重用**，否则环里那些旧根指向的块已经被覆盖，**它们作为回退候选是假的**。⇒ **不要把根环读成「可以回滚 K 代」。** 它买到的只是「本代根写坏了还能读到上一代根」；上一代根**能不能用**取决于块重用延迟，那是另一件事、另一笔成本。**把两者混为一谈会让人以为环是免费的。** ⚠️ **块复用的界不能是格式常量**（I-7.4（近 K 代块未被复用） 引 D22（单元原子性怎么合成） 这一句）：它由 `max(F_生效, 环里最旧有效根)` 现算（D16（发布语义） 已定项 1），F 随盘紧抬、环里最旧有效根随发布走。「根环深度 = 块重用延迟」很容易被实现成「深度是格式字段，所以 K 是格式字段」，而格式常量把一个纯可用性参数焊死，没有任何好处。**根环只承担「本代根写坏了还能读到上一代」**，checkpoint 尺度的块重用约束由 D16（发布语义） 已定项 15 单独承担。

**射程**：

- K 不再是运行时参数：I-7.4（近 K 代块未被复用） 逐字「原「K 是运行时策略、下限 ≥ 2」作废」，「K 代」就指回退候选集。按旧 K 口径「每多买一代要多扣多少」的量级在 E123（K 与回退深度的二选一各要付什么），块复用的界改成现算之后它不是现行的取法。
- D22（单元原子性怎么合成） 自己那句「一圈的时长 ≥ 最坏的块重用延迟需求」按 checkpoint 尺度读做不到（一圈是毫秒级）；按消费者计价读已经结清：恢复择根回退要的是恰 2 代，一圈 = R × S ≥ 2 恒成立。唯一按墙钟计价的是遍历跳（读父指针到读子块那一步），一跳耗时超过保护窗口（可退的代数 ÷ 发布率）就不再被罩住（C214（遍历跳的窗口余量没人守））。
- 两条钉块规则扣的不是同一批：根环那条扣最近几代根引用的块，D16（发布语义） 已定项 15 + 已定项 4 扣整个 journal 重放窗口内释放的块。**能承重的是相位数，不是倍数**：根环多扣出来的那条逃生口在一个 checkpoint 周期里只存在 `根环(K) − 1` 个相位，与 checkpoint 间隔无关；⚠️ **不许用「相差 250 倍」那种说法**（那是跨相位比较，倍数还随 checkpoint 间隔线性变）。

**依据**：

- E33（两条钉块规则）：两条钉块规则扣的不是同一批，调小根环深度只在 checkpoint 刚完成后的头几个相位腾得出空间——「K 扣住的正是 ENOSPC 死锁的唯一逃生口」那条旧依据被打空；死锁 2 的破法是保留池。
- E50（根环槽数的上下界）：一圈是毫秒级，按 checkpoint 尺度读「一圈 ≥ 需求」任何合理槽数都做不到。
- E92（最坏块重用延迟需求）：九个消费者分三类计价，恢复择根回退恰 2 代、管理员回退要整个根环深度、只有遍历跳按墙钟计价。
- 块复用的界：D16（发布语义） 已定项 1 的用户定案，原话在变更史。
- 证据等级：数是观测；「所以那条旧依据要改写」是推论，只走了本地腿，两条云端腿欠着。

**欠**：C214（遍历跳的窗口余量没人守）。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:521-547`（整段抄，未转述）**

```markdown
#### 未定项 6：zoned 的区写指针是一个不受本工程校验和管辖的见证者

**问题**：区写指针（WP）由设备维护，本工程既算不出也验不了。崩溃后写指针可能领先于我们认定的提交点。D20（承重面：单元的原子性与自包含） 承重面的表述覆盖不到它。拆成两问：甲，WP 在本工程的判定里占什么地位；乙，zoned 上固定结构（系统配置槽轮换、根环槽轮换）能不能原地轮转。

**现状**：挂到 zoned 布局线开线：第一版跑非 zoned 布局（D12（目标介质） 已定项 5「普通 SSD」不要求 ZNS），这个对象在第一版里不存在；不提前走三方，也不先定「只做寻址提示、不当见证者」，到时连同「最新的根在哪」的发现程序与 D12（目标介质） 的介质分层一起判（用户定向，原话在变更史）。它定不下来，原因不是证据不够，是它的前置是一张欠条：已定项 2 是一套**非 zoned 参数**，全部建立在同一个槽地址被反复覆写上，而 zoned 的根发现程序从没设计过（已定项 24 只留了一句「需要一套发现程序」）⇒ 非 zoned 上 WP 这个对象不存在，zoned 上「槽」这个对象还不存在，每一条臂都要先回答「最新的根在哪」。开线时已知的连带：D17（实现分层与第三方管道） 已定项 1 取单一失败模型 ⇒ zoned 线不许靠管道声明把区写指针推出去、只能走核心层合成；D16（发布语义） 已定项 8 的暖机给发现程序加了「至多 R 次空发布占槽位、根覆盖两块盘才放 fsync 返回」；开线代价至少 +1 个结构等价类（D17（实现分层与第三方管道） 已定项 2）。

**已排除的臂**：

| 问 | 臂 | 现状 |
|---|---|---|
| 甲 | 作恢复的权威上界 | 出局，升格为已定项 10 |
| 甲 | 进 checker 作交叉校验 | checker 那个形态出局，升格为已定项 11；运行时 / 挂载时的形态没评估 |
| 甲 | 只做寻址提示（下次往哪写、从哪开始扫，不进择新 / 已发布 / 完整性判定） | **写不成条款**：「从哪开始扫」在追加式的区里就等于「哪些是候选」，而根记录的字段表（已定项 7）里没有一行记「这是本区第几条」⇒ 要么用 WP 界定候选集（WP 就进了择新，前后半句自相矛盾），要么扫整个区（要读 WP 之上那片不是本工程写下的字节）；「设备维护的状态」与「设备探测的几何量」怎么分仓里没有定义，而根槽宽正好横跨它（C219（设备维护的状态与设备探测的几何量没有分界）） |
| 甲 | 候选集直接定义成整单元自证校验和验得过的槽 | **出局**：把过滤器当成了枚举器——I-7.1（根槽有效集非空） 的论域「全部根槽位」与谓词「自证校验和通过」本来就分两段写着，它只重写了谓词；zoned 上「槽」没有指称物；非 zoned 上候选集是 R × S 个已知位置，那正是 WP 不存在的一侧 |
| 乙 | 区对交替 + 追加写 + 区重置 | **被一条纯算术打掉**：`P` 是奇素数 ⇒ `v₂(P × chunk) = v₂(chunk)`，而 `zone_size` 是 2 的幂 ⇒ `zone_size` 整除区域间距 ⟺ `zone_size ≤ chunk`；根环区域的起点落不到区边界上，区域与别的数据共享区，区重置永远做不成。要活下来得动一条已定条款（起点向下对齐、要求 `zone_size ≤ chunk`、或放弃区重置） |
| 乙 | 固定结构一律落常规区，无常规区的设备拒绝 mkfs | 与已定项 24「zoned 那一类根会漂移」说反话，要走它得先回写那一句；NVMe ZNS 命名空间里不存在常规区（本机内核树 `include/linux/nvme.h` 只登记 `NVME_ZONE_TYPE_SEQWRITE_REQ`），适用面缩到有常规区的 zoned 设备 |
| 乙 | 按区类型分支（有常规区落常规区，没有走区对交替） | **出局**：分支谓词没有主语（池级读法的分支变量在写飞行中就变，撞 `.claude/rules/fs-design.md` 硬要求 5）；真实判据是常规区覆不覆盖 `{0, P·chunk, 2P·chunk}` 三个点；每区槽数 S 是系统配置里的池级单值，表达不了两种形态，「一圈 = R × S」失去定义 |
| 甲 / 乙 | 等 zoned 线开线 | 现状 |

**已有的观测**：

- E125（顺序区写指针之上读到什么）（`null_blk`，本机没有真的 zoned 盘，数都是模拟出来的）：顺序区 WP 之上读得到字节（全 `0xff`），`zone_full=1` 造得出 WP 领先于提交点；未写过的顺序区读到 `0xff`、常规区读到 `0x00`；**从没写过的顺序区与写过再 `BLKRESETZONE` 的区，字节、写指针、区状态三样逐格相同** ⇒ 「全 `0xff` ⇒ 未写」这条把未写与损坏分开的规则在单台设备上没有可用的输入，换成取 WP 也一样分不开。
- 那个 `0xff` 不是设备返回的，是 `null_blk` 驱动里一行硬编码（`memset(p + valid_len, 0xff, len);`）；NVMe ZNS 读未写块的返回值由 `DLFEAT` 选，`NSFEAT` 有一位打开时控制器可以直接报错，而 Linux 的 `drivers/scsi/sd_zbc.c` 对 host-managed 设备 `if (!sdkp->urswrz)` 直接 `return -ENODEV`；`zonefs.rst` 第 330–332 行：只读区的写指针按 ZBC / ZAC 标准无效——最需要读根的时候（介质开始坏），恰恰是 WP 可能没有值的时候（外部事实，未在本项目验证）。
- 三方：未定项 6 三轮（第一轮材料 `_d22-item6-background.md`，第二轮 `d22-item6r2-*`，第三轮 `d22-item6r3-*`；第二轮本地腿三次抽样都作废）。

**欠**：C219（设备维护的状态与设备探测的几何量没有分界）；C230（区重置后的态与从没写过的态分不开而不变量全绿）；C17（设备几何当全局参数）：条带宽度 × `zone_size` 不整除那一格要把 `P` 代进去。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:548-559`（整段抄，未转述）**

```markdown
### 与其他决策的连锁

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） 校验和位置 | D22（单元原子性怎么合成）给了 D4（校验和位置） 一条理由：**把检测能力从设备假设上摘下来**。现役对照是 btrfs（父指针无校验和 ⇒ 被迫加 transid）与 ZFS（父指针有校验和 ⇒ 无第二道闸） |
| D12（目标介质） 目标介质 | 推翻「能原地覆盖根指针」这个措辞的读法；两类布局需要的是同一种根机制（已定项 24） |
| D16（发布语义） 发布语义 | 根自身的冗余形态这个缺口被收窄成一种机制；journal 的格式多一条硬要求（世代号必须被实际检查，已定项 23） |
| D18（块里携带什么信息） 自包含的物理上限 | D22（单元原子性怎么合成） 已定项 20 是 D18（块里携带什么信息） 的直接推论：撕裂粒度变大就退化成丢失写，而丢失写块里带什么都抓不到 |
| D20（承重面：单元的原子性与自包含） 承重面 | D22（单元原子性怎么合成）是 D20（承重面：单元的原子性与自包含） 推论三第一问与第二问的答案；第三问（违反时怎么发现）落在崩溃点重放上 |
| D21（权威态与派生态的分界） 权威态/派生态 | 记账是权威态且无副本，撕裂后的恢复路径是 D22（单元原子性怎么合成）已定项 3（「记账结构是不是 COW」；已定项 2 是根环参数） |
| [checks-owed.md](../checks-owed.md) C16（管道声明失败模型例外） | 撕裂注入不能靠 dm-flakey，只能在崩溃点重放 harness 里做 |

```

**出处 `crates/singlefs-checker/invariants/02-根环.md:1-38`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态已定，已实现（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根） 形态 2026-10-20 定，随 C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批实现）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](../../../.claude/kb/decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](../../../.claude/kb/decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 与崩溃点重放也按它取回退候选集下界（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image`，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清），带 F 的根全坏时不在合法状态上误红这一条与别的按回退候选集判的几条。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是判定宽度的整数倍，判定宽度 = max(声明的 `physical_block_size`, 声明的 `io_min`)（`io_min` 为 0 视为没声明）；mkfs 在任何写之前先拒不按这个宽度对齐的几何，非常规几何由 mkfs 拒、不再是合法镜像。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 已实现（池级 checker `judge_root_slots_aligned_to_the_decision_width`（`crates/singlefs-checker/src/image.rs`）逐槽判：判定宽度取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的（异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条），是声明值、注明「声明值，未探测」，探测比对仍欠 C388（checker 的判定宽度不自己探测）；mkfs 在任何写之前拒不按这个宽度对齐的几何（`crates/singlefs-core/src/make_filesystem.rs` 的 `first_slot_not_aligned_to_the_decision_width`）；异构池一份与逐槽判的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行） ⚠️ **射程注（D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；崩溃点重放全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，崩溃点重放全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 ⚠️ **判法补一句（已实现：`walk::check_pool_image` 末尾在取 max 那一格之后再判最新那条根自己的水位）**：另判最新那条根的水位不低于全池最大树 ID。只取根环里全部根的 max 判不到把最新那条根的水位压低的镜像：攻方量到 checker 49 条 0 违例，而 core 的 `recover` 报 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 失败；core 重发树 11..18 的状态 checker 也 0 违例。用攻方那两份坏镜像钉住（只压低最新那条根的水位；建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去、再把环里每条根的水位压到 11，只红它），被攻过零轮（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 9、10 行） |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小、又落在锚在镜像最新根上的那一个一圈窗口里的最新那条根的 txg（只取这一个窗口，`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings`；窗口里一条都没有时不判，同不带记号那一支的「之前的根」；盲带：F 越过洞里一条有效根时这一支看不见，欠账见 C717（回退下界上限的单窗口盲带）；正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」（两支都取）取锚在镜像最新有效根的 txg 上的那一个一圈窗口里 txg 比它小的那些（窗口 = txg > 锚 − 3S，3S 是环长；用户 2026-10-04 定「乙，两支都取两个窗口」，第 10 批起锚在被判那条根上的那个窗口被这一个罩住、不再另取，主 agent 认的读法、被攻过零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 10 批 core 簇交回那一条），窗口里没有比它小的根时那条根不判；只取锚在被判那条根上的窗口时，洞里留着的上一圈的旧根会顶上来、在合法镜像上判红（随机历史 230 段红 21、150 段红 9，并上锚在镜像最新有效根上的窗口之后各剩 1 段，红在带卸载记号那一支，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批 9-6 交回那一条）；取全环时攻方五批随机历史里每批 27–46 段红（判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第五节）；上限照 D16（发布语义） 已定项 1「抬 F 的上限」那一行，窗口里挂载那一刻读不出或自证不过、之后这个进程没写过的槽当作可能是一条非空有效根、F 不越过它，这些槽 checker 看不见，checker 只按镜像上读得出的根判（推的：读故障不留在盘上）。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归崩溃点重放录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归崩溃点重放与 C556（checker 与崩溃点重放不读系统配置里的 F）（已还清：checker 与崩溃点重放的记录核对器已按系统配置里的 F 取下界），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（两支的「之前的根」只取锚在镜像最新有效根上的那一个一圈窗口）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；崩溃点重放 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二）。**一圈窗口**（用户 2026-10-04 定两个窗口，被攻过零轮；第 10 批起只留锚在镜像最新有效根上的那一个）：harness 用例在 `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs`（带卸载记号的根在洞上再挂之后不许红，用例 `an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole`；两个窗口里仍照红，用例 `an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant`；准入那一支的几形同文件），`crates/mutations.tsv` 里以「第9批改钉」打头、点名带卸载记号那一支的 2 行，与以「第9批」打头、点名「一圈窗口」的 4 行；改钉之后随机历史 230 段、150 段两批 0 段红（改钉簇实现员量的，输出在 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`）；GPU 判器的扫描内核两支同取锚在镜像最新有效根上的那一个窗口（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `in_the_one_revolution_window`），对拍用例与 8 行变异在 checker 档、没跑 |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；崩溃点重放每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，崩溃点重放没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数，偏移 361）与第一版常量 3（`ROOT_RING_REGIONS`）不等、S（每区槽数，偏移 362）不在 [4, 16] 里同样归这一条、整池不作保：S 照 D22（单元原子性怎么合成） 已定项 2「越界整池拒绝挂载，与每区槽数 S 越界同一个处置」（core 报 `RecoveryFailure::RootRingSlotsPerRegionOutOfRange`），R 用户 2026-10-20 定 C663（系统配置槽越界与不收之值两读者分类不同）「读并整池拒」；两样此前归「这一槽不可择」，2026-10-20 起不再是。格式版本不认得与 incompat 位不认得同在一槽时先判格式版本（用户 2026-10-20 定 C663（系统配置槽越界与不收之值两读者分类不同）「格式版本先」）：这一槽归这一条、整池不作保，不归 incompat 不认得那一支的「这一槽不可择」 | 已实现（池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md`「用户定第二轮改法：四入口再核两样、core 读 325/371（2026-09-27）」那一行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）。R、S 与次序（C663（系统配置槽越界与不收之值两读者分类不同））：checker 在 `geometry_of` 判 R、S，R 是 0、1 或 2、大于 3 各报一个成员（`RegionCountZero`、`RegionCountBelowTheFirstVersionConstant`、`RegionCountPastTheRegionDeviceFields`），S 报 `RootRingSlotsPerRegionOutsideTheFormatInterval`，四个成员都归 `value_refused_by_this_reader`；次序在 `crates/singlefs-checker/src/lib.rs` 的 `check_system_configuration_slot`（先格式版本、后 incompat 位），GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `checker_read_system_configuration_slot` 同序同判。坏镜像在 `crates/singlefs-checker/src/image.rs` 的库单测：`a_slot_declaring_seventeen_slots_per_region_refuses_the_whole_pool`、`a_slot_declaring_two_regions_refuses_the_whole_pool`、`a_slot_declaring_zero_regions_refuses_the_whole_pool`、`a_slot_declaring_four_regions_refuses_the_whole_pool`、`a_slot_with_an_unrecognized_format_version_and_an_unknown_incompat_bit_refuses_the_whole_pool_by_its_format_version`（盘 1 槽 0 改一处、重封，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余报不适用），阳性对照 `a_pool_holding_only_first_version_values_reads_every_slot_as_usable_and_keeps_header_checksum_coverage`；GPU 一侧对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 的 `the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_seventeen_slots_per_region` 与 `the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_two_regions`|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`m2-impl-r2-fixes-a-implementer-report.md`「每条新测试的证红」那张表变异 6、7、8 那两行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告「受影响的层 0 流与崩溃枚举用例」一节「I-7.14（系统配置本盘设备号等于盘在池里的身份）：判绿 4155 次、不适用 0 次」那一句）；崩溃点重放快档与全量没跑）|
| I-7.15 | 各盘所选系统配置槽设备数相同 | 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同（D22（单元原子性怎么合成） 已定项 9 的字段）。不要求等于盘表：降级镜像（掉了一块盘）里 checker 手里的盘数小于设备数是合法的。形态抄 I-1.4（块头 fsid 一致） 的逐盘循环；core 的 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上，两边同表。 | 已实现（池级 checker `walk::check_pool_image` 逐盘把各盘所选槽的设备数同第一块盘所选槽的比；坏镜像是盘 1 两个槽的设备数改成 3 并重封，只红它，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C466（几块盘的系统配置设备数对不上时 checker 判不出）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7 行） |
| I-7.16 | 系统配置槽世代号与槽号的关系 | 每块盘上，任一可择的（自证过、池级字段在读者收的范围里）系统配置槽 i：世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时世代号相差不超过 1，相等时只许都是 1（mkfs 写下的两槽世代号都是 1，D22（单元原子性怎么合成） 已定项 16）。带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里） 整池拒，不进这一条。 | 已实现（池级 checker `judge_system_configuration_slot_generations_follow_the_rotation`（`crates/singlefs-checker/src/image.rs`）：奇偶一格、差值一格，差值那一格取「两槽都是 mkfs 写的 1，或相差恰为 1」；差值与奇偶各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 7 行与第四节用户定案第 1 条） |
| I-7.17 | 根槽位置等于 txg 算出的位置 | 根环里任一自证过的根所在的槽位 (区域 r, 槽 s)：`checkpoint_txg` ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成） 已定项 8）时只判 s == 0，不判 r。公式是 D22（单元原子性怎么合成） 已定项 16 的定案。根环补洞判决要的「槽位与 txg 对得上」就是这一条（`research/prompts/m3-ring-r1-main-verification.md` 第二节第 6 行）。⚠️ **这一条红的镜像上，checker 其余不变量不作保**：合法写者（发布路径与 mkfs 种第 0 代根）造不出错位的根；checker 照旧把错位那条根当有效根，按有效根算的判定（I-3.1（已分配统计对得上） 的候选集、回收门槛与环里最旧有效根）会把它算进去，结论不作数（根环三方第一轮正推 X5 取甲、第二轮 Y5.2，判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第四节）。core 一侧：C743（重建分配器记新类时不核根住不住这一槽） 定案（用户 2026-10-20「所有读根环处都收」），读根环处照同一道落点公式核、读回不住这一槽的根按自证不过处置，第 0 代根照 mkfs 种根的槽另认，与这一条同一例外（根环三方第一轮判决 `research/prompts/m3-close-rootring-r1-main-verification.md` 第二节 X1）；core 那一半随根环实现批，今天没写。 | 已实现（池级 checker `judge_root_records_live_in_the_slot_their_txg_names`（`crates/singlefs-checker/src/image.rs`）：根环里每条自证过的根逐条判，第 0 代只判槽 0；公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs`）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 8 行） |
| I-7.18 | 根记录的回收门槛等于现算的门槛 | 根记录带回收门槛 t（8 字节）与钉槽 p（1 字节，取建账时还钉着的槽里钉值最小的那个）；t 在根记录偏移 428（8 字节小端）、p 在 436，紧跟分配记录树根指针、在算法类型之前，根记录宽 466（D22（单元原子性怎么合成） 已定项 7；journal 新根段末尾同一对字段，段内偏移 364、372，C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批定）。写者取 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)（回收门槛见 D16（发布语义） 已定项 1「可再分配」那一行）；checker 取这条根自己带的 F 现算回收门槛，要求等于 t。**只判建账的发布**（那次发布建了这条根自己的记账树）：零单元发布与无文件版本上的复制根，t、p 抄自上一版，环转满之后求等会误红，不判；最新根下面没有记账树时报不适用。坏实现里「t 恒取 F」与「p 写成没有钉、其实还钉着槽」这一条判得出（后者现算的门槛不带钉、比 t 高，求等红；第三轮辩方 Z4.4 推的，没造坏镜像），判不出的两类见 I-7.19（钉着的槽不住本实例写的根）。出处：根环三方第二轮判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y4 那一行、第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773（读坏与全零根槽的回收门槛钉只在内存里） 那一行与第三节用户定案（用户 2026-10-20 认 t 的取法与只判建账的发布） | 已实现（池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root`：「建账的发布」按走最新根那一遍记下的记账树根指针出生 txg 等于这条根的 txg 认，现算式与丁-defer 门槛共用 `deferred_slots_reclaim_threshold`，钉值当一条有效根并进去；最新根的实例表链没走到末片时报不适用（被抛弃的根滤不全，实现员的读法，交主 agent 认）；GPU 判器走读内核 `crash_judge_checker.wgsl` 同名函数同判；坏镜像与回归在 `crates/singlefs-harness/tests/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs`） |
| I-7.19 | 钉着的槽不住本实例写的根 | 根记录的钉槽 p 指着的那一槽（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：读坏或读回全零的根槽把回收门槛钉在它可能住过的最旧 txg），镜像上不得住着这条根同一实例写的根——钉在这个进程写掉那一槽时必解，那一槽已住着本实例的根而 p 还指着它，就是写掉了没解钉。只判建账的发布，最新根下面没有记账树时报不适用，口径同 I-7.18（根记录的回收门槛等于现算的门槛）。两样故障叠在一起（读坏钉着的槽，这个进程对它的写报失败而实际落了盘）时这一条会误红，超出单故障，记在 C773（读坏与全零根槽的回收门槛钉只在内存里） 的射程里。**判不出的两类坏实现**：钉着的槽重读读出有效根而写者没解钉、凭空钉一个读得出的槽——读故障不留在盘上，镜像上与合法的钉逐字节相同；用户 2026-10-20 定不记成已认代价、要做成判得出，账在 C863（钉槽的两类坏实现镜像上判不出）。出处同 I-7.18（根记录的回收门槛等于现算的门槛） | 已实现（池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root`，与 I-7.18 同一处；p 不是 255 又不小于环长 3S 时判红（还原不成一个槽，根环三方第二轮正推 Y4.3 拟稿那一句，实现员照收，交主 agent 认）；GPU 判器走读内核同判；坏镜像在 `crates/singlefs-harness/tests/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs`） |

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](../../../.claude/kb/decisions.md) D22（单元原子性怎么合成） 已定项 2。


```

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:1-6`（整段抄，未转述）**

```markdown
## D15 (Format Freeze Policy) — Settled

[中文](15-格式冻结政策.md) · **English** · [日本語](15-フォーマット凍結ポリシー.ja.md)

D15 (Format Freeze Policy) governs two questions: **when it is permissible to state "format v1 frozen"**, and **what channel newly added items follow after freezing**. It is the common prerequisite for D11 (Index Nodes Message Buffers) / D12 (Target Media) / D14 (Dual-Track File Sizes and Lifetime) — the current value of the cost "format branch permanent" is zero, but infinite after freezing, and the true cost of those decisions is defined by it. Out of scope: what value each individual field should settle on belongs to its respective decision; before the first external user emerges, the format is soft, and that policy resides in `.claude/rules/format-evolution.md`; the semantics of each bit in `feature-bits.md` belongs to the decision claiming that bit, while D15 (Format Freeze Policy) only governs how bit numbers are allocated and whether they can be reclaimed.

```

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:7-235`（整段抄，未转述）**

````markdown
### Settled Items

Items in D15 (Format Freeze Policy) have only a single set of numbers without reordering.

| # | Item | Settlement |
|---|---|---|
| 1 | **Specific handling of item 7 in pre-freeze checklist** | Item 7 does not write a list of decisions, but is written as a single predicate — each unsettled item across the repository declares whether it "touches the format"; if a freeze request intersects with it, judge red; default padding width is 0 **Status: Settled.** **Progress: No code object.** |
| 2 | **Whether the tree table is an open list** | Yes, made an open list from day-1 **Status: Settled.** **Progress: Implemented.** |
| 3 | **Source of `spec_hash`** | No switching. Invariably computed against `format-spec/<component>.toml`; that toml file is a complete projection of the kb field table, including row ordering and non-scalar on-disk semantic rules **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Bit allocation registry for feature bits** | Registry resides in settled item 4; incompat bit 1 = first pure-SSD layout line (the version with F and unmount mark), set upon mkfs, bit 0 retired; compat_ro and compat have zero bits allocated **Status: Settled.** **Progress: Implemented.** |
| 5 | **What the action of freezing constitutes** | Freezing = committing a file `.claude/kb/format-frozen/<component>.toml`; humans only decide whether to initiate it, while the machine decides whether the request holds **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Checklist that must be completed before freeze** | Seven prerequisites, re-judged upon each freeze; values for each item are taken fresh from the settlement line of the corresponding item, no manual copying allowed **Status: Settled.** **Progress: No code object.** |
| 7 | **How many layers freezing is split into, and what lies outside the four layers** | The four layers have strict dependency ordering (system configuration and block header, pointer field layout, key encoding, index node internal layout), plus three independent freeze components **Status: Settled.** **Progress: No code object.** |
| 8 | **What freedoms are lost after freezing** | Width, existence, and meaning of frozen fields can no longer change, only additions permitted; checker judgments on frozen portions also become compatibility contracts **Status: Settled.** **Progress: No code object.** |
| 9 | **Post-freeze change judgment table** | Nine categories of changes judged for compat / compat_ro / incompat and whether to bump, with the criterion being what old readers will do **Status: Settled.** **Progress: No code object.** |
| 10 | **Bit count, accounting, and allocation rights of feature bits** | Three bitmaps of 256 bits each stored in system configuration; allocation right is "first merged, first claimed", bit numbers strictly monotonic; once used, bits may never be reclaimed **Status: Settled.** **Progress: Implemented.** |
| 11 | **Two gates that can go red paired with freezing** | Format freeze contract (`spec_hash` comparison) and backward compatibility regression (golden images); image bodies do not enter git, checklist is append-only **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Target of Item 7 is Unsettled Items, Not a Decision List

**Settlement**: Three rules.

1. **The target is every still unsettled item across the whole repository, not a decision list.** Each unsettled item must carry a verdict statement — **touches the format or not**: whether its answer will modify any on-disk bytes or alter the **interpretation measurement basis** of existing bytes (the measurement basis aspect reflects the pattern of C97 (changing field semantics without altering width produces no red gate anywhere in repository): width does not change by a single byte, but semantics change). Three possible values: **touches / does not touch / boundary unsettled (treated as touches)**; **unwritten is treated as touches**.
2. **Judgment**: When a freeze request is initiated, any item judged as "touches" or not yet judged triggers a red gate. To freeze in batches, the request must **list each item by name and explicitly accept the risk** (pattern following settled item 7 "risks of batch freezing must be explicitly accepted"); **omitting any item triggers a red gate**.
3. **Default padding width is 0**, with the burden of proof on the party proposing non-zero width; it must pass the format branching criterion in `.claude/rules/fs-design.md` (verbatim: "whether a format branch is worthwhile depends on whether it can turn 'used incorrectly' into 'fails to mount'").

Framing determines interpretation (`.claude/singlefs-ai-sop/skills/decide/SKILL.md` hard requirement 7: phrasing of tentative items must have only one reading): instead of "pinning values across six cells item by item" (content of each cell lives in its respective decision; duplicating it creates dual records of the same fact), we adopt "defining the judgment rules for item 7"; item 7 is **re-judged upon each freeze**, because the checklist is a pre-commit check triggered by commits and runs with every freeze request.

**Scope**: Default padding width 0 governs **reserving placeholders for features settled as not done**; it does not govern **general reserved areas** within structures — general reserved areas are governed by the first row of settled item 9 judgment table (new field in reserved area / TLV tail => compat without bump) and C102 (inode header lacks reserved space making field additions non-compat), which have different criteria; conflating the two would deduce "reserve nothing anywhere", whereas C102 (inode header lacks reserved space making field additions non-compat) is repaying an owed check in the opposite direction. What elevates to a rule is solely the procedure of "default 0 + inverted burden of proof"; any entry reaching the branch "v1 does not include this feature" must pass the format branching criterion again on its own. Predicates are not partitioned by layer: exiting freeze is judged by structure, not by layer (scope of settled item 7), and membership is incomplete — the three independent freeze components in settled item 7 do not belong to any of the four layers => layer-partitioned predicates are undefined on them; partitioning by the binary predicate "touches format or not" avoids both defects and assumes no independent adjudicability between layers. The cell for D26 (Background Compaction and Placement Reclamation) settled item 4 remains owned by D26 (Background Compaction and Placement Reclamation) itself; D15 (Format Freeze Policy) has no authority to answer on its behalf. Judgment inputs are always fetched fresh and never cached here: `grep -c '未定\*\*' .claude/kb/decisions.md` counts how many unsettled items remain across the repository (item list line tails are currently formatted as "—— **未定** · 未实现", no longer ending with "未定**"), and `bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` judges whether that list is a faithful projection of the body.

**Evidence**: No experiment: what is settled is the target and default values of gate judgment rules, with no quantifiable metric. Supporting it are two facts — list patterns leak entries (D22 (Synthesizing Unit Atomicity) settled item 9 is the **field table of system configuration**, and system configuration is Layer 1; D19 (Block Pointer Structure & Width Budget) settled item 6 is the **field table of central mapping entries**, residing in Layer 2 — neither was on that six-decision list, so the list had zero say when freezing those two layers), and relying on a "format-level" tag fails to catch anything (registry location of IDs is in the title line of each decision body, not on the index page; `grep -rn 格式级 .claude/gate.d/ .claude/scripts/` yields zero hits => zero objects, zero implementation). Settled by main agent based on three-way verification; an invalid argument is recorded in decisions history to avoid reusing it.

**Owed checks**: C62 (settlement values of items branch in kb); C102 (inode header lacks reserved space making field additions non-compat).

#### Settled Item 2: Tree Table Made an Open List from Day-1

**Settlement**: The tree table is made an open list from day-1. Evidence lies in the judgment table of settled item 9: under an open list, "adding a keyspace" means old readers merely **do not see the new tree** (compat grade, graceful degradation); hardcoding tree counts means old readers **cannot read the tree table** (incompat, immediate breakage). **Open lists strictly dominate**, with the only cost being an extra table length field plus a variable-length table section in the tree table unit — identical to the rationale in settled item 10 "reserves should lean generous". => Adding keyspaces is an explicitly stated expansion objective of this project, not an accident, so that row in the judgment table is pinned as **compat**.

**Scope**: What is settled is "tree table is an open list", regardless of where tree tables reside — tree tables reside in the root record indirection layer (D22 (Synthesizing Unit Atomicity) settled item 7 verbatim "tree roots invariably reside in tree table units, never flattened into root slots", settled item 12 settles "system configuration does not carry its own tree table unit pointer"; D21 (Boundary Between Authoritative and Derived States) groups root record + tree table unit + instance table unit together into "root"). It **must be incorporated when Layer 1 freezes**, and cannot wait until adding a tree reveals it was omitted; freeze layer membership for the tree table unit itself remains owed.

**Evidence**:

- E142 (new-pool file creation dry run): The tree table unit written during new-pool file creation declares entries by count (0 entries in mkfs version, 11 entries in transaction version, representing values before adding retired trees; 12 entries in transaction version after adding retired trees), table length is data rather than compile-time constant — the open list form holds on written bytes.
- User settlement, verbatim text in decisions history.

**Owed checks**: C246 (tree table unit lacks freeze layer membership).

#### Settled Item 3: `spec_hash` Invariably Computed from Complete Projection of kb Field Tables, Never Switched to Rust Structs

**Settlement**: `spec_hash` is invariably computed against `format-spec/<component>.toml`. That toml file is a **complete projection of the kb field table** — containing **row ordering** (on-disk sequence) and **non-scalar on-disk semantic rules** (merge rules for component 2 in settled item 7 are encoded as data), not merely the collection of registered scalar constants. What is added once code exists is **not changing the hash source**, but two gates: (1) **equality check** between spec and code (comparing both width and ordering); (2) golden image regression (settled item 11 rule 2).

Four reasons:

1. **Directionality**. D13 (Verification Route) settled item 5 verbatim: "**checker and implementation share only one artifact: a constants module generated from kb field tables**, with the generator reading from kb and both sides only consuming, no manual edits permitted by anyone" => authority resides in kb, while Rust structs are **downstream consumers**; computing hashes against structs anchors the fingerprint of a permanent contract on a derivative artifact.
2. **Hash is a change detector, not an equality check between two artifacts.** Computing hashes on structs cannot catch "structs inconsistent with kb"; it alarms only when structs **change**; what genuinely catches that class of error is an equality check, which requires the spec side to exist => switching across not only gains nothing extra, but tears down the opposite side required for comparison.
3. **Language binding**. D17 (Implementation Layering & Third-Party Pipeline) settled item 3 verbatim: "Verdicts must be independently verifiable by third parties, otherwise 'conformance' degrades into 'we claim you conform'". Anchoring fingerprints on Rust source text prevents third-party pipelines from recomputing them.
4. **Feasibility**. Spec files for the three components can be drafted immediately, allowing hash mechanisms to take effect long before the first freeze; whereas the freeze moment for component 1 in settled item 7 is the first image writing a Type 2 container => switching would be forced into the identical commit as the first real freeze, with zero burn-in period.

**Scope**: Projecting only scalars into the spec is insufficient; row ordering must be included, and equality checks must compare sequence — the unit header in [layout/01-first-txn.md](../layout/01-first-txn.md) lists six adjacent fields of width 8 in row order (the 5-tuple's tree ID / object ID / object birth txg / anchor offset, plus birth generation and fsid); transposing any two during serialization leaves all scalar constants and `DATA_UNIT_HEADER_BYTES` unchanged while on-disk bytes become completely misaligned, falling into the "changing field semantics" row of settled item 9 judgment table. The boundary in D13 (Verification Route) settled item 5 stating "generator emits only scalar values" governs shared artifacts between checker and implementation, whereas `format-spec/<component>.toml` is a **gate artifact** with zero runtime consumers => outside that boundary's scope, and a field table with row ordering contains neither branching nor arithmetic. **Explicit statement of what is uncaught**: hashes do not establish binding between spec and actually written bytes — if the first golden image was itself written by faulty code, regression will treat that faulty version as baseline; what stops that is checker as an independent parser and the conformance suite judge, not `spec_hash`. Specs can only be written manually, with no existing mechanism projecting kb field tables to toml. The **magnitude of risk** in batch freezing where "early frozen layers omit reserved fields" cannot be quantified; it depends on yet non-existent concrete layouts in D8 (Core Index Structure) / D11 (Index Nodes Message Buffers), so only the stop-loss rule in settled item 7 can be provided, while residual risk remains unquantifiable.

**Evidence**:

- E142 (new-pool file creation dry run): Six adjacent fields of width 8 in the unit header are arranged sequentially in written bytes; swapping any two while preserving all scalars is constructible — projecting only scalars in spec cannot safeguard field ordering.
- Three-way verification: the reverse deduction leg eliminated the branch "spec projects only scalars" according to pre-execution failed clauses; settlement adopts the tightened form (spec must include row ordering), which demands one extra guarantee and subtracts nothing. Settled by main agent.

**Owed checks**: C227 (golden image baseline written by its own code version); C226 (spec manually copied without generation from field tables); C119 (freeze components lack spec files).

#### Settled Item 4: Bit Allocation Registry for Feature Bits

**Settlement**: Three bitmaps of 256 bits each (three bitmaps of settled item 10); the sole registry for bit numbers is the table in this section. "Refuse mount on unrecognized bits" must apply even to the default layout, so the first layout line claims a bit from mkfs onward — avoiding the alternative "first line claims no bit" (which would allow readers unfamiliar with SSD layout to mount).

| bitmap | bit | meaning | provenance |
|---|---|---|---|
| incompat | 0 | First pure-SSD layout line | D12 (Target Media) settled item 5; D12 (Target Media) settled item 1 "one incompat bit per layout"; retired: the version with rollback witness and system configuration lacking F, readers reject as unrecognized bit (user 2026-09-26) |
| incompat | 1 | First pure-SSD layout line (system configuration carries rollback floor F, root record carries unmount mark, lacks rollback witness and rollback row) | D12 (Target Media) settled item 5; user 2026-09-26 settled "switch incompat bit, cannot mount" |
| incompat | 2 | inode record carries truncation fields (T and N_w starting at offset 120; in a pool without bit 2, offsets 120–135 must be all zero); set from mkfs onward, and a writable mount raises an old pool without bit 2 from the number-taking write onward | User settled 2026-10-17 to open a new bit, judgment `research/prompts/m4-trunc-r2-main-verification.md` Section 6; judgment for the registration and the raise, `research/prompts/m4-validate-code-r1-main-verification.md` Section 2 row 'truncation fields and bit 2' and Section 3 row 1 |
| incompat | 3..255 | Unallocated; append one bit for each new layout line | D12 (Target Media) settled item 1 |
| compat_ro | 0..255 | Unallocated | — |
| compat | 0..255 | Unallocated | — |

**Scope**: Governs only how bit numbers are split and the assignment of the first bit; semantic definitions of each bit belong to the decision claiming that bit. Bytes written in new-pool file creation: the 96 bytes of feature bits in system configuration change from all zeros to incompat bit 1 set to 1; bitmap is little-endian, bit 1 is the 2nd bit counting from the least significant bit of the first byte (D22 (Synthesizing Unit Atomicity) settled item 13). This item did not go through three-way verification; both main agent recommendation and user choice can be overturned.

**Evidence**:

- E142 (new-pool file creation dry run): The 96 bytes of feature bits in system configuration are written per this item, closing its blank G12, and [layout/01-first-txn.md](../layout/01-first-txn.md) updated synchronously.
- User settlement, verbatim text in decisions history; pre-settlement cell text also in decisions history, including its same-day entry "alters bytes: no" — that statement has been overturned by this settlement.
- User settlement 2026-09-26: bit 0 retired, layout identity switched to bit 1, old images (with rollback witness, system configuration without F) cannot mount (three-way verdict `research/prompts/m2-rollback-forward-r3-main-verification.md` section 2 K7: "reject writable mount if witness table non-empty" eliminated; read-only recovery of 13 corruption depths had 4 falling into rolled-back versions; altering system configuration format version number cannot block it as current readers do not verify version numbers), verbatim in decisions history; no experiment: old readers failing to mount on new bit measured on prototype, new readers rejecting old bit is derived; "altering system configuration format version number cannot block it as current readers do not verify version numbers" is obsolete: starting 2026-09-27, readers refuse to mount the whole pool on any system configuration format version other than 1 (settled 2026-10-21, D22 (Synthesizing Unit Atomicity) settled item 9).
- User settlement 2026-10-17: open a new incompat bit 2 (inode record carries truncation fields), three-way judgment `research/prompts/m4-trunc-r2-main-verification.md` Section 6; judgment for writing back the bit 2 registration and the in-place raise by a writable mount, `research/prompts/m4-validate-code-r1-main-verification.md` Section 2 row 'truncation fields and bit 2' and Section 3 row 1; verbatim in decisions history.

**Owed checks**: C183 (one incompat bit per layout while on-disk bytes show no difference).

#### Settled Item 5: Freezing is a Commit Arbitrated by Gates, Not a Verbal Statement

**Settlement**: The **action** is committing `.claude/kb/format-frozen/<component>.toml` to the repository (containing `version`, `frozen_at_commit`, `spec_hash`). **Who judges**: not a human "feeling ready" — once the gate detects that the current diff touches this directory, it switches automatically to a stricter suite of pre-checks (checklist in settled item 6), permitting the commit to pass only if all are satisfied. **Humans only decide "whether to initiate a freeze now", while machines decide "whether this freeze request holds"**, isomorphic with `.claude/singlefs-ai-sop/rules/show-me-test.md` "do not differentiate by source, differentiate only by evidence".

**Scope**: That stage does not run ordinarily (running only when the diff touches that directory), so attaching it never slows daily commits; **it is destined to trigger 0 times for long periods, which is not a flaw but correct behavior** — sharing the same honesty as gates explicitly listing unimplemented stages. ⚠️ It is commit-triggered, whereas the freeze moment for component 1 in settled item 7 is "the first image writing a Type 2 container", which is an execution rather than a commit => pre-freeze checklist has no execution path for component 1, and the gate will never ignite for it.

**Evidence**: No experiment: settles gate trigger timing and arbitration ownership, with no quantifiable metric; format aligns with existing "project local stage" mounting points in shared `gate.sh`. Settled by main agent.

**Owed checks**: C225 (component freeze moment bypasses gate triggers).

#### Settled Item 6: Checklist That Must Be Completed Before Freeze

**Settlement**: Seven prerequisites, judged item by item upon freeze request initiation, re-judged upon each freeze.

| # | Item | What happens if freeze proceeds without it |
|---|---|---|
| 1 | For the component to freeze, its field table in kb has no row marked "unsettled", "envisioned", "semi-settled", or "placeholder" | Freezing an evolving design; subsequent answers to open items will be retroactively claimed as part of v1. ⚠️ This and item 7 are two distinct rulers: item 1 measures completeness of **already written tables**, while item 7 measures whether **unwritten items** touch it |
| 2 | I-1 (block self-describing) and I-2 (checksum) category checkers are implemented and pass green | "Checker is specification" — without checkers, "what v1 is" exists only in documents, and subsequent modifications cannot be mechanically adjudicated |
| 3 | Four reserved fields of D9 (Encryption) settled item 12 can be asserted in checkers as "present with width matching reservation" | Omitting even one forces incompat patching post-freeze, and existing old images permanently lack this field |
| 4 | I-3.1 (allocated accounting matches) is implemented and passed N>=5 rounds all green | The sole verification method for accounting correctness is this comparison; without it, whether frozen ledger fields are correctly designed remains completely untested |
| 5 | Model differential and crash-point replay each pass at least one round, **running against the exact format fingerprint about to be frozen** | Otherwise v1 describes a format never touched by automated verification |
| 6 | C4 (project rule checklists inconsistent) and C8 (gate scope cannot be evaluated) in [checks-owed.md](../checks-owed.md) are implemented | Post-freeze rule divergence or gates slowing down without measurement leads to incidents in higher-risk phases |
| 7 | Every still unsettled item across repository carries the verdict "touches format or not", and none judges "touches" or remains unjudged — unless the freeze request explicitly enumerates them and accepts the risk | This chain mutually defines itself; silently freezing with "unsettled" items equates to pretending decisions are already made. Judgment rules in settled item 1 |

⚠️ **Reviewing every item in the checklist requires taking values fresh from the settlement line of corresponding items; manual copying is forbidden.** Freezing is irreversible, while copying an erroneous number looks nowhere awkward in prose => **open the settlement line of that item verbatim during review**; machine-checked forms and prerequisites are tracked in C62 (settlement values of items branch in kb); until that check is implemented, this step relies on humans.

**Executable forms of items 1 and 7**: Item 1 scans the field table of the component to freeze — the table in [layout/01-first-txn.md](../layout/01-first-txn.md) listing fields sequentially by on-disk order is its ready form; scanning its **"width" and "status" columns**, encountering **any** of `未定` / `预想` / `半定` / `占位` triggers a red gate. Item 7 scans the "touches format or not" verdict in unsettled items across the repository. It splits into two halves: the **presence check** is the second ruler of the blocking-verdict cell in `.claude/gate.d/doc-decisions.sh`, sharing parsing, authoritative lists, independent reporting, and distinct remedies with the first ruler (modifying new-pool file creation bytes), running regularly; the **rejection check** (judging "touches" or unwritten triggers red gate unless listed explicitly with accepted risk) happens upon freeze request initiation, whose prerequisite is C225 (component freeze moment bypasses gate triggers) — freeze requests have no execution path for component 1. Neither item **creates a duplicate checklist** — `.claude/singlefs-ai-sop/rules/kb-discipline.md`: only one authoritative record permitted per fact.

**Scope**: Wordlist for item 1 is written into clauses without leaving script authors to guess — terms used in byte tables today are "envisioned" and "semi-settled" (the extent tree node row verbatim "envisioned 162 = 86 + 2 x 24 + 28", status column "semi-settled"; whether it carries key ranges is unsettled, 162 with, 114 without, a 48-byte difference shifting the entry start offset), precisely the evolving design item 1 intends to intercept; a check scanning only "unsettled" would pass it, allowing a Layer 4 freeze request to retroactively affirm a 48-byte unresolved measurement basis into v1. The checklist validates whether prerequisites are complete, not whether the content of any item is correct.

**Evidence**:

- E142 (new-pool file creation dry run): The field table item 1 scans is the very table written and checked row by row; the words "envisioned" and "semi-settled" in "width" and "status" columns possess concrete objects.
- Settled by main agent (narrowing of item 1 vocabulary in 2026 detailed in decisions history); judgment rules for item 7 reside in settled item 1.

**Owed checks**: C62 (settlement values of items branch in kb); C8 (gate scope cannot be evaluated) (the other half of checklist item 6, C4 (project rule checklists inconsistent), has been repaid).

#### Settled Item 7: Freezing Split into Four Layers, Plus Three Independent Freeze Components Outside Layers

**Settlement**: The four layers have strict dependency ordering.

```
Layer 1: System configuration + block header (self address / generation / tree ID / fsid / magic)
    ↓
Layer 2: Pointer field layout (D4 checksum + D9 MAC / nonce / ciphertext checksum)
    ↓
Layer 3: Key encoding for each keyspace (D8 locality_id / inode / offset)
    ↓
Layer 4: Index node internal layout (node size rules, bset format, D11 message buffers)
```

Rationale for dependency direction: Layer 1 is the physical premise that "metadata blocks must be self-describing", upon which all subsequent layers rest; Layer 2 is pinned by D9 (Encryption) MAC / nonce / ciphertext checksum being "impossible to insert after the fact", and must precede tree structures relying on it; once key encodings in Layer 3 are settled, organizing these keys within nodes has a foundation for discussion.

**Risks of batch freezing must be explicitly accepted**: If an early frozen layer omits fields, subsequent work must patch via incompat, equivalent to burning design margins prematurely. There is only one stop-loss rule: **Layers 1 and 2 must include all four D9 (Encryption) reserved fields prior to freeze**, as they happen to be fields known to be needed eventually, which cannot be deferred to Layers 3 or 4.

**Independent freeze components outside the four layers**: Three classes of permanent bytes do not belong to any of the four layers; each forms an independent component, with `spec_hash` in C9 (covertly modifying format after freeze) recorded per component, keeping the four layers untouched:

- 1. **Payload record format for packed record units**, registered by packed record type: Type 2's 140-byte inode record table (D8 (Core Index Structure) settled item 6), freeze moment being the first image writing a Type 2 container; Type 1 (width 40, D18 (Information Carried in a Block) settled item 24) is mutually independent, not frozen together with Type 2.
- 2. **Statistics registry** (tag value + dimension + merge rules, D5 (Snapshot and Space Accounting Mechanism) settled item 3 / 4) — merge rules are on-disk semantics; two readers applying different rules to the same bytes will compute divergent values; C71 (silently ignoring unrecognized statistics tags) shields only unrecognized tags, not recognizing tags while applying wrong rules.
- 3. **journal record format** (record header, named items, payload checksum coverage, transaction boundary): pre-freeze requirements no earlier than Layer 2 and placement granularity; freeze moment is the commit submitting its toml file; new record formats follow "new record type + system configuration log-incompat bit", full text in D16 (Publish Semantics) settled item 3.

**Scope**: The four-layer diagram is not rearranged: D21 (Boundary Between Authoritative and Derived States) settled item 10 changed status classification to per-structure judgments; in Layer 4, Code 2 nodes of the accounting tree are authoritative state while internal nodes of other trees are derived state, so no layer is pure; thus the unit of exiting freeze is a structure rather than a layer (placement of each structure, its state category, and whether it exits freeze are registered in `.claude/kb/freeze-layer-membership.md`, checked by gate doc-decisions freeze-layer-membership cell); spec files for the three components do not exist; freeze moment for component 1 is an execution rather than a commit, so the gate in settled item 5 does not ignite for it. No clauses yet exist for which layer tree table units and instance table units belong to.

**Evidence**: No experiment: layering and dependency ordering are derived from sorting "which elements cannot be retrofitted later", with no quantifiable metric; target of the stop-loss rule (four reserved fields of D9 (Encryption)) resides in D9 (Encryption) settled item 12. Settled by main agent and user, verbatim in decisions history.

**Owed checks**: C119 (freeze components lack spec files); C225 (component freeze moment bypasses gate triggers); C246 (tree table unit lacks freeze layer membership).

#### Settled Item 8: What Freedoms Are Lost After Freezing

**Settlement**:

| Freedoms during soft format phase | What they become after freezing |
|---|---|
| Tearing down and rebuilding structures | Width / existence / meaning of frozen fields can no longer change; additions only, retired fields follow "retain read support, cease writing" |
| Correcting text directly upon finding flawed designs without compatibility | Can still be overturned at decision record level, but **on-disk byte compatibility transitions from zero cost to non-zero** |
| mkfs automatic selection rules can switch algorithms anytime | Selection rules for frozen components are also pinned; changing algorithms causes default discrepancies between old and new images, requiring a new format version number |
| Modifying formats directly upon disappointing experiment outcomes | New policies must be introduced via new feature bits, retaining interpretability of old values |
| Checker freely modified alongside formats | **For frozen portions, checker judgments become de facto compatibility contracts** — silently altering checker interpretations of frozen fields is as perilous as silently altering formats |

**Scope**: Only states which actions cease to be valid post-freeze, without adjudicating specific change grades — which belongs to settled item 9. Before the first external user emerges, format remains soft, and none of this table takes effect prior to that point (`.claude/rules/format-evolution.md`).

**Evidence**: No experiment: itemized expansion of the constraint "external permanent commitments cannot be altered", with no quantifiable metric. The final row (checker judgments become contracts) shares provenance with "checker is specification" in D13 (Verification Route). Settled by main agent.

**Owed checks**: None.

#### Settled Item 9: Post-Freeze Change Judgment Table

**Settlement**:

| Change type | Old reader behavior | Grade | Bump | Rationale |
|---|---|---|---|---|
| Adding fields in reserved space / TLV tail | Ignored, continues normal operation | compat | No | Pure addition without semantic dependency; safety net of "ignoring unfamiliar fields" suffices |
| Adding fields whose omission causes old readers to miscompute **another existing field** | **Miscomputes, potentially corrupting data** | incompat | Yes | Once semantics couple, safety nets fail |
| Expanding existing field width | Parsed with old width, out-of-bounds read / misalignment | incompat | Yes | Width is part of layout; offset calculations in old code break completely |
| Changing existing field meaning | Interpreted with old meaning, producing erroneous behavior **without error** | incompat, with **old position abandoned without reuse** | Yes | Reusing positions leaves old and new readers unable to distinguish, the most dangerous class. Two examples (D18 (Information Carried in a Block) settled item 11): adding a unit class (registry codes 4–15) alters the interpretation measurement basis of class identity segments; giving meaning to the constant-0 flag bit at common prefix offset 7 (old readers reject non-zero bits) |
| Adding record types (new key types) | Read-only traversals may misjudge; compaction / write may delete misunderstood records | compat_ro | Yes | **"Read-only safety" and "writable safety" are independent guarantees** that must be judged separately. ⚠️ Packed record types inside packed record units **do not follow this row** (D18 (Information Carried in a Block) settled item 11): skipping a tombstone or inode record alters read answers, defaulting to incompat; only types proven at registration that "skipping changes no read answer" demote to compat_ro, and compat_ro readers must not execute scan-rebuilds |
| Adding a keyspace | If tree table is open list day-1: simply does not see new tree; if tree count hardcoded: fails to read tree table | **compat** (settled item 2 makes tree table an open list day-1). Reserving open tree table -> compat; omitted -> incompat | Yes | **"Whether tree table is expandable" must be settled at freeze**, cannot wait until adding trees reveals omission |
| Changing sorting order | Binary search / traversal results incorrect **without error** | incompat | Yes | Sort order is index identity; reordering log-structured nodes directly relies on its invariance |
| Changing checksum / MAC algorithm | D9 (Encryption) algorithm type field reserved -> unfamiliar types fail explicitly | Reserved -> compat_ro; unreserved -> incompat | Yes | Item 4 of D9 (Encryption) settled item 12 is precisely the channel prepared for this change |
| Retiring a field | Unaffected, old readers continue interpreting old way | compat, with **parsing code preserved permanently** | No | Any field committed to disk must remain interpretable as long as a single old image exists |

**Scope**: Judges "what old readers will do", not "whether this change ought to be made". The first row (additions in reserved space are compat) holds only where real reserved space exists; inode headers carry only 2 bytes of reserved space, accommodating few fields.

**Evidence**:

- E59 (can buffered messages be recomputed solely from units): Skipping a tombstone record causes deleted keys to fully resurrect, resurrection count precisely matching deletion count — the quantified basis for "packed record types do not follow compat_ro row, defaulting to incompat".
- Settled by main agent, row-by-row rationales documented in table.

**Owed checks**: C102 (inode header lacks reserved space making field additions non-compat); C97 (changing field semantics without altering width produces no red gate anywhere in repository).

#### Settled Item 10: Bit Count, Accounting, and Allocation Rights of Feature Bits

**Settlement**:

- **Bit count**: Three bitmaps of **256 bits** each (`u64[4]`), stored in system configuration. Bits are far cheaper than field widths (3x32 bytes negligible for system configuration), **reserves should lean generous**.
- **Accounting**: `.claude/kb/feature-bits.md` lists `bit | category | name | introduced version | introduced commit | status | one-line semantics`; the sole registry for bit numbers is the table in settled item 4.
- **Allocation right is "first merged, first claimed", not an approval hierarchy**. Gate enforcement: bit numbers strictly monotonic, no gap-skipping reservations, no code referencing non-existent bit numbers — mechanical arbitration.
- **Once used, bits may never be reclaimed.** Rationale is **stricter** than non-recycling of numbers in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md): numbers are retrieval keys whose reuse misleads humans or models; **feature bits are physical bits that existed on disk**; reallocating them to a new feature means an old image mounted on new code misinterprets the bit as "new feature X enabled" — **the file system would perform real incorrect actions** (e.g., treating non-existent encryption characteristics as present and searching for non-existent keys). Retired bits are marked "meaning permanently locked to semantics of last use prior to retirement", and never assigned to new features.

**Scope**: Governs how bits are divided, recorded, and whether they can be reclaimed; does not govern semantics of individual bits (belonging to decisions claiming them).

**Evidence**: No experiment: bit counts and non-reclamation are deduced from the physical fact that "old images will be read by new code", with no quantifiable metric; 256-bit space overhead is negligible for system configuration, so this cell does not optimize for frugality. Settled by main agent.

**Owed checks**: C183 (one incompat bit per layout while on-disk bytes show no difference).

#### Settled Item 11: Two Gates That Can Go Red Paired with Freezing

**Settlement**:

1. **Format freeze contract**: Record `spec_hash` for each frozen component; gate recomputes and compares; divergence requires current diff to **simultaneously** satisfy "`feature-bits.md` has new rows or `version` is explicitly altered" and "corresponding entry in `decisions.md` modified in same commit". Missing either triggers a red gate.
2. **Backward compatibility regression (golden images)**: Whenever a component version changes, generate and archive an image using **code of that era**; gate reads all historical images using **current code**, executes checker, and compares expected content digests. Image bodies **do not enter git** (`.claude/singlefs-ai-sop/rules/command-safety.md`: test images must not reside in repository); repository commits only the manifest `.claude/kb/golden-images.toml`; images are placed in `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}`. The manifest is **append-only** — deleting regression for any version equates to silently abandoning compatibility commitments, requiring formal decision records.

**Scope**: Neither is yet implemented; numbers are in checks-owed table. Rule 2 compares only "whether today's code understands historical images", not "whether the original image was written correctly"; Rule 1 anchors on declarations rather than bytes; neither shields against "the initial golden image itself being produced by erroneous code".

**Evidence**: No experiment: settles what two gate checks should judge; discrimination self-proof awaits implementation; failure modes they guard against (covertly modifying formats, quietly abandoning compatibility commitments) are tracked in C9 (covertly modifying format after freeze) and C227 (golden image baseline written by its own code version). Settled by main agent.

**Owed checks**: C9 (covertly modifying format after freeze); C227 (golden image baseline written by its own code version).

````

**出处 `.claude/kb/decisions/15-format-freeze-policy.en.md:236-239`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D15 (Format Freeze Policy) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:1-6`（整段抄，未转述）**

```markdown
## D15 (フォーマット凍結ポリシー) — 確定

[中文](15-格式冻结政策.md) · [English](15-format-freeze-policy.en.md) · **日本語**

D15 (フォーマット凍結ポリシー) は二つの問題を所管する：**いつ「format v1 frozen」と宣言できるか**、および**凍結後に新要素を追加する際にどの流路を経由するか**。本決定は D11 (インデックスノード内のメッセージバッファ要否) / D12 (対象メディア) / D14 (大小ファイル・永続/一時デュアルトラック) に共通する前提である——「フォーマット分岐が恒久化する」というコストの現在価値はゼロであるが、凍結後は無限大となるため、これら各決定の真の代償は本決定により定義される。対象外とする事項：各フィールド自体に定めるべき具体的な値は各決定の所管に属する；最初の外部ユーザーが現れるまでフォーマットは流動的であり、その運用方針は `.claude/rules/format-evolution.md` にある；`feature-bits.md` における各ビットのセマンティクスは当該ビットを要求する決定に属し、D15 (フォーマット凍結ポリシー) はビット番号の割り振り方針と回収可否のみを所管する。

```

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:7-235`（整段抄，未转述）**

````markdown
### 確定項目

D15 (フォーマット凍結ポリシー) の項目番号体系は単一であり、番号の振り直しは行わない。

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **凍結前チェックリスト第 7 項の具体的処置** | 第 7 項は決定リストを列挙せず、単一の判定式とする——リポジトリ内の未確定項目ごとに「フォーマット変更要否」を自ら宣言し、凍結要求がそれと交差すれば赤判定；デフォルトパディング幅は 0 **状態：確定。** **進捗：対象なし。** |
| 2 | **ツリーテーブルは開放リストか否か** | 開放リストとする、day-1 から開放リストとして実装 **状態：確定。** **進捗：実装済み。** |
| 3 | **`spec_hash` の算出元** | 切り替えない。常に `format-spec/<コンポーネント>.toml` に対して算出し、当該 toml は行順および非スカラーのディスク上セマンティクス規則を含む kb フィールド表の完全射影とする **状態：確定。** **進捗：未実装。** |
| 4 | **feature bit のビット割り当て登録表** | 登録表は確定項目 4 に配置；incompat ビット 1 = 最初の純 SSD レイアウト系統（F およびアンマウント記号を含む版）、mkfs 時点から設定、ビット 0 は廃止；compat_ro、compat は 1 ビットも割り当てない **状態：確定。** **進捗：実装済み。** |
| 5 | **凍結という動作の定義** | 凍結 = リポジトリに `.claude/kb/format-frozen/<コンポーネント>.toml` をコミットすること；人間は開始するか否かのみを決定し、機械が要求の成立可否を判定する **状態：確定。** **進捗：未実装。** |
| 6 | **凍結前に完了必須のチェックリスト** | 7 項目の前提条件とし、凍結ごとに再判定；各項目の値は対応する小項目の確定行からその都度取得し、手作業の転記を禁止する **状態：確定。** **進捗：対象なし。** |
| 7 | **凍結階層の分割と4層外の独立要素** | 4層には厳格な依存関係（システム構成とブロックヘッダ、ポインタフィールド配置、key 符号化、インデックスノード内部レイアウト）が存在し、別途3つの独立凍結コンポーネントを設ける **状態：確定。** **進捗：対象なし。** |
| 8 | **凍結後に失われる自由** | 凍結済みフィールドの幅・存在・意味は変更不可となり追加のみ許容；凍結箇所に対する checker の判定も互換性規約となる **状態：確定。** **進捗：対象なし。** |
| 9 | **凍結後の変更判定表** | 9 分類の変更それぞれについて旧リーダーの挙動を基準に compat / compat_ro / incompat および bump の要否を判定 **状態：確定。** **進捗：対象なし。** |
| 10 | **feature bit のビット数・会計・割り当て権** | 3 つのビットマップ（各 256 ビット）をシステム構成に格納；割り当て権は「先行マージ優先」、ビット番号は厳格に単調増加；一度使用したビットは再利用を禁止 **状態：確定。** **進捗：実装済み。** |
| 11 | **凍結に伴い整備すべき赤判定検査** | フォーマット凍結規約（`spec_hash` 照合）および後方互換性リグレッション（ゴールデンイメージ）；イメージ本体は git に含めず、リストは追加のみ許容 **状態：確定。** **進捗：未実装。** |

#### 確定項目 1：第 7 項の判定対象は未確定項目であり決定リストではない

**確定内容**：3 条。

1. **対象はリポジトリ内の未だ未確定であるすべての項目であり、決定リストではない。** 未確定項目ごとに判定文言——**フォーマット変更要否**を付与する：その結論がディスク上のバイト列を変更するか、あるいは既存バイト列の**解釈計測条件**を変更するか否か（計測条件の側面は C97 (改字段含义而不改宽度，全仓没有一处会红) の形態を指す：幅は 1 バイトも変わらないが意味が変化する場合）。取り得る値は 3 つ：**変更あり / 変更なし / 境界未確定 (変更ありとみなす)**；**未記載は変更ありとみなす**。
2. **判定**：凍結要求の発行時、「変更あり」または未判定の項目があれば赤判定とする。段階的凍結を行う場合は、要求内で**全項目を個別に列挙してリスクを明示的に受容**しなければならず（確定項目 7 の「段階的凍結のリスクは明示的に受容されなければならない」を踏襲）、**列挙漏れがあれば赤判定**とする。
3. **パディング幅のデフォルトは 0** とし、非ゼロの幅を確保しようとする側に立証責任を課し、`.claude/rules/fs-design.md` のフォーマット分岐判定基準（原文ママ：「フォーマット分岐に価値があるかは、『誤用』を『マウント不可』に変えられるか否かによる」）を通過させなければならない。

問いの立て方が読み方を規定する (`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 必須要件 7：未確定項目の問いは一意に読めなければならない)：「6 つのスロットの値を個別に固定する」方式（各スロットの内容は各決定内に存在するため、複製は同一事実の二重記録となる）は採らず、「第 7 項の判定規則を定める」方式を採る；チェックリストはコミット契機の前置検査であり凍結要求ごとに実行されるため、第 7 項は**凍結ごとに再判定**される。

**射程**：デフォルトパディング幅 0 の規定が所管するのは**実装しないことが確定した機能のためのプレースホルダ確保**であり、構造体内の**汎用予約領域**は所管しない——汎用予約領域は確定項目 9 判定表の第 1 行（予約領域 / TLV 末尾へのフィールド追加 ⇒ compat かつ bump なし）および C102 (索引节点头没有预留区，加字段不是 compat) が所管し、判定基準が異なる；両者を混同すると「どこにも予約を残さない」という結論に至るが、C102 (索引节点头没有预留区，加字段不是 compat) は現在その逆方向の未済の検査を清算中である。規則として昇格するのは「デフォルト 0 + 立証責任の転換」という手続きのみであり、「v1 にはこの機能を含めない」という分岐に達した項目は、各自でフォーマット分岐基準を再度通過する必要がある。述語は層ごとに分割しない：凍結からの除外は構造単位で判定され、層単位ではないためであり（確定項目 7 の射程）、かつ構成要素が揃っていない——確定項目 7 における 3 つの独立凍結コンポーネントは 4 層のいずれにも属さない ⇒ 層単位の述語はそれらに対して定義できない；「フォーマット変更要否」という二値述語で分割することでこれら双方の破綻を回避でき、層間の独立判定可能性を前提としない。D26 (バックグラウンド整理と配置回収) 確定項目 4 のセルは従来どおり D26 (バックグラウンド整理と配置回収) 自らが保持し、D15 (フォーマット凍結ポリシー) が代行して回答する権限はない。判定入力は常に現行状態から取得し、ここに複製を保持しない：`grep -c '未定\*\*' .claude/kb/decisions.md` によりリポジトリ内の未確定項目残数を数え（項目リスト行末は現在「—— **未定** · 未实现」の形式であり「未定**」で終わらない）、`bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` で当該リストが本文の忠実な射影であることを判定する。

**根拠**：実験なし：ゲート判定規則の対象とデフォルト値を定めるものであり、定量可能な数値は存在しない。これを支えるのは 2 つの事実である——リスト形式には漏れが生じ得ること（D22 (ユニット原子性の合成方法) 確定項目 9 は**システム構成のフィールド表**であり、システム構成は第 1 層である；D19 (ブロックポインタの構造と幅予算) 確定項目 6 は**中央マッピングエントリのフィールド表**であり、第 2 層に位置する——いずれも 6 決定のリストには含まれず、これら 2 層を凍結する際にリストは一切関与できない）、および「フォーマットレベル」タグに依存した網羅は不可能であること（番号の登録場所は各決定本文のタイトル行であり索引ページではないため、`grep -rn 格式级 .claude/gate.d/ .claude/scripts/` はヒット 0 件 ⇒ 対象 0、実装 0）。メイン agent による確定、三方検証に基づく；成立しなかった論拠は再利用防止のため変更履歴に記録している。

**未済の検査**：C62 (分项定案值在 kb 里分叉)；C102 (索引节点头没有预留区，加字段不是 compat)。

#### 確定項目 2：ツリーテーブルを day-1 から開放リストとする

**確定内容**：ツリーテーブルは day-1 から開放リストとして構成する。根拠は確定項目 9 の判定表にある：開放リスト下では「新規 keyspace の追加」は旧リーダーにとって単に**新規ツリーが見えない**だけであり（compat グレード、グレースフルデグラデーション）；ツリー数をハードコードすると**ツリーテーブルを読み出せない**状態となる（incompat、即時破損）。**開放リストが厳格に優位**であり、その代償はツリーテーブルユニット内のテーブル長フィールドと可変長テーブル部分の追加にとどまる——これは確定項目 10 の「予約は広めに確保する」と同一の論理である。⇒ keyspace の追加は本プロジェクトにおいて明記された拡張目標であり偶発的事象ではないため、判定表の当該行のグレードは **compat** として固定される。

**射程**：定めるのは「ツリーテーブルが開放リストであること」であり、ツリーテーブルの配置先とは無関係である——ツリーテーブルはルートレコードの間接参照層に位置する（D22 (ユニット原子性の合成方法) 確定項目 7 原文ママ「ツリーのルートは常にツリーテーブルユニットに格納し、ルートスロットにフラット展開しない」、確定項目 12「システム構成は自前のツリーテーブルユニットポインタを持たない」；D21 (権威態と派生態の境界) はルートレコード + ツリーテーブルユニット + インスタンス表ユニットを一括して「ルート」に分類）。これは**第 1 層の凍結時に組み込まれなければならず**、ツリーを追加する段になって未確保であることが判明する事態は許容されない；なおツリーテーブルユニット自体の凍結層帰属は依然として未済である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：新規プールでのファイル作成で書き出されるツリーテーブルユニットはエントリ数で宣言され（mkfs 版は 0 エントリ、トランザクション版は 11 エントリであり、退役ツリー追加前の現行値；退役ツリー追加後のトランザクション版は 12 エントリ）、テーブル長はコンパイル時定数ではなくデータである——開放リストの形態は書き出されたバイト列において成立している。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：C246 (树表单元没有冻结层归属)。

#### 確定項目 3：`spec_hash` は kb フィールド表の完全射影から常に算出し、Rust 構造体へ切り替えない

**確定内容**：`spec_hash` は常に `format-spec/<コンポーネント>.toml` に対して算出する。当該 toml は **kb フィールド表の完全射影**であり——**行順**（ディスク上の順序）および**非スカラーのディスク上セマンティクス規則**（確定項目 7 のコンポーネント ② のマージ規則をデータ形式で記述）を含み、登録済みのスカラー定数の単なる集合ではない。コード実装後に新たに追加されるのは**ハッシュ元の変更ではなく**、2 つの検査である：① spec とコード間の**同等性検査**（幅と順序の双方を比較）；② ゴールデンイメージリグレッション（確定項目 11 第 2 条）。

4 つの理由：

1. **方向性**。D13 (検証ルート) 確定項目 5 原文ママ：「**checker と実装の間で共有されるのは唯一つ：kb のフィールド表から生成された定数モジュールのみであり**、ジェネレータが kb から読み取り双方は消費のみ行い、何人による手作業の変更も禁止する」⇒ 権威は kb にあり、Rust 構造体は**下流の消費者**である；構造体に対してハッシュを算出することは、恒久的な規約の指紋を派生品に固定することに他ならない。
2. **ハッシュは変更検知器であり、二つの成果物間の同等性検査ではない。** 構造体に対してハッシュを算出しても「構造体と kb の不一致」は捕捉できず、構造体が**変更された**際に警告を発するのみである；その種の誤りを真に捕捉するのは同等性検査であり、同等性検査は spec 側の存在を前提とする ⇒ 切り替えを行っても付加価値が得られないばかりか、照合すべき対向側を解体してしまう。
3. **言語バインディング**。D17 (実装階層とサードパーティパイプライン) 確定項目 3 原文ママ：「判定は第三者が独自に再検証可能でなければならず、さもなければ『適合』は『当方が適合と主張している』状態に劣化する」。指紋を Rust ソースコードのテキストに固定すると、サードパーティパイプラインによる再計算が不可能となる。
4. **実現性**。3 つのコンポーネントの spec ファイルは直ちに記述可能であり、ハッシュ機構を最初の凍結より遥か以前から有効化できる；一方で確定項目 7 のコンポーネント ① の凍結契機はタイプ 2 コンテナを書き出す最初のイメージである ⇒ 切り替え作業が最初の実凍結と同一コミットに強制され、試行運用期間がゼロとなる。

**射程**：spec がスカラーのみを射影するのでは不十分であり、行順を含めなければならず、同等性検査も順序を比較しなければならない——[layout/01-first-txn.md](../layout/01-first-txn.md) のユニットヘッダには幅 8 の隣接フィールドが 6 つ行順に並んでおり（5つ組のツリー ID / オブジェクト ID / オブジェクト誕生代 / アンカーオフセット、ならびに誕生代番号、fsid）、シリアライズ時に任意の 2 つを入れ替えても全スカラーおよび `DATA_UNIT_HEADER_BYTES` は一切不変のままディスク上のバイト列が完全に狂い、確定項目 9 判定表の「既存フィールドの意味変更」の行に該当することになる。D13 (検証ルート) 確定項目 5 の「ジェネレータはスカラー値のみを出力する」という境界規定は checker と実装の間の共有物を所管しており、`format-spec/<コンポーネント>.toml` は**ゲート成果物**であり実行時の消費者は存在しない ⇒ 当該境界の射程外であり、行順を持つフィールド表には分岐も算術も存在しない。**検知不能な事項の明記**：spec と実際に書き出されたバイト列との間において、ハッシュは直接の拘束力を持たない——最初のゴールデンイメージ自体が誤ったコードで出力された場合、リグレッションは誤った版を基準として扱うことになる；これを阻止するのは独立したパーサである checker および適合性スイートの判定プログラムであり、`spec_hash` ではない。spec は手書きするほかなく、kb フィールド表から toml へ射影する自動機構は存在しない。段階的凍結における「早期凍結層でのフィールド確保漏れ」の**リスク規模**を定量化することはできず、未だ存在しない D8 (コアインデックス構造) / D11 (インデックスノード内のメッセージバッファ要否) の具体的レイアウトに依存するため、確定項目 7 の損切り規則を提示できるのみであり、残留リスクは計測不能である。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：ユニットヘッダにおける幅 8 の隣接する 6 フィールドは書き出されたバイト列上で行ごとに整列しており、任意の 2 つを入れ替えても全スカラーを不変に保つ状態は構成可能である——spec がスカラーのみを射影する形態ではフィールド順序を担保できない。
- 三方論証：事前定義された失敗条項に基づき、逆推論レグが「spec はスカラーのみを射影する」という方針を除外した。確定内容は厳格化された形態（spec は行順を含まなければならない）を採用しており、新形態は旧形態よりも追加要件を課すのみで要件を一切削減していない。メイン agent による確定。

**未済の検査**：C227 (黄金镜像的基准是它自己那份代码写的)；C226 (spec 是手抄的，没有从字段表生成)；C119 (冻结组件没有 spec 文件)。

#### 確定項目 4：feature bit のビット割り当て登録表

**確定内容**：3 つのビットマップ（各 256 ビット、確定項目 10 の 3 種のビットマップ）；ビット番号の唯一の登録場所は本節の登録表とする。「認識不能なビットでのマウント拒絶」はデフォルトレイアウトに対しても機能させる必要があるため、第 1 の系統が独自に 1 ビットを占有し mkfs 時点から設定する——「第 1 系統はビットを消費しない」方式は採らない（さもなければ SSD レイアウトを認識しないリーダーでもマウント可能となってしまうため）。

| bitmap | bit | 意味 | 出典 |
|---|---|---|---|
| incompat | 0 | 最初の純 SSD レイアウト系統 | D12 (対象メディア) 確定項目 5；D12 (対象メディア) 確定項目 1「レイアウトごとに 1 つの incompat ビット」；廃止：ロールバック証人を含みシステム構成に F を欠く版であり、リーダーは一律に未認識ビットとして拒絶する（ユーザー 2026-09-26） |
| incompat | 1 | 最初の純 SSD レイアウト系統（システム構成にロールバック下限 F を含み、ルートレコードにアンマウント記号を含み、ロールバック証人およびロールバック行を持たない） | D12 (対象メディア) 確定項目 5；ユーザー 2026-09-26 決定「incompat ビットを変更し、マウント不可とする」 |
| incompat | 2 | inode レコードが切り詰めフィールドを持つ（オフセット 120 から T と N_w；ビット 2 を持たないプールではオフセット 120–135 はすべてゼロでなければならない）；mkfs 時からセットされ、書き込み可能マウントはビット 2 を持たない旧プールを番号取得の書き込みからビット 2 付きに引き上げる | ユーザー 2026-10-17 決定、新たに 1 ビットを開く、判定 `research/prompts/m4-trunc-r2-main-verification.md` 第 6 節；登録と引き上げの書き戻し判定 `research/prompts/m4-validate-code-r1-main-verification.md` 第 2 節の「切り詰めフィールドとビット 2」の行、第 3 節の 1 行目 |
| incompat | 3..255 | 未割り当て；レイアウト系統が新設されるごとに追加 | D12 (対象メディア) 確定項目 1 |
| compat_ro | 0..255 | 未割り当て | — |
| compat | 0..255 | 未割り当て | — |

**射程**：ビット番号の分割方法と最初のビットの帰属のみを定め、各ビットのセマンティクスは当該ビットを要求する決定に属する。新規プールでのファイル作成におけるバイト列：システム構成の feature bits 領域（96 バイト）は全ゼロから incompat ビット 1 のセット（1）へと変化する；ビットマップはリトルエンディアンであり、ビット 1 は第 1 バイトの最下位から数えて第 2 ビットである（D22 (ユニット原子性の合成方法) 確定項目 13）。本項目は三方論証を経ておらず、メイン agent の推奨およびユーザーの選択はいずれも覆され得る。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：システム構成の feature bits 領域（96 バイト）が本項目に従って書き出され、空白領域 G12 が解消され、[layout/01-first-txn.md](../layout/01-first-txn.md) が同期更新された。
- ユーザーによる確定、発言原文は変更履歴にある；確定前の当該セルの原文も変更履歴にあり、当時の記述「バイト変更：否」を含んでいたが、その記述は本項目の確定により覆された。
- ユーザー確定 2026-09-26：ビット 0 は廃止されレイアウト識別はビット 1 に切り替わり、旧イメージ（ロールバック証人を含みシステム構成に F を欠く）はマウント不可となる（三方判定 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第 2 節 K7：「証人テーブルが空でなければ書き込み可能マウントを拒絶する」方針は除外、読み出し専用リカバリの破損深度 13 パターンのうち 4 パターンがロールバックされた版に該当；システム構成フォーマットバージョン番号の変更では阻止できず、現行リーダーはバージョン番号を検証しないため）、原文は変更履歴にある；実験なし：新ビットに対する旧リーダーのマウント不可はプロトタイプで計測済み、旧ビットに対する新リーダーの拒絶は論理的導出；「システム構成フォーマットバージョン番号の変更では阻止できず、現行リーダーはバージョン番号を検証しない」との記述は陳腐化：2026-09-27 以降、リーダーはシステム構成フォーマットバージョンが 1 でない場合プール全体のマウントを拒否する（2026-10-21 決定、D22 (ユニット原子性の合成方法) 確定項目 9）。
- ユーザー確定 2026-10-17：新たに incompat ビット 2（inode レコードが切り詰めフィールドを持つ）を開く、三方判定 `research/prompts/m4-trunc-r2-main-verification.md` 第 6 節；ビット 2 の登録と書き込み可能マウントによる引き上げの書き戻し判定 `research/prompts/m4-validate-code-r1-main-verification.md` 第 2 節の「切り詰めフィールドとビット 2」の行、第 3 節の 1 行目；原文は変更履歴にある。

**未済の検査**：C183 (每套布局一个 incompat 位而盘上差不出字节)。

#### 確定項目 5：凍結はゲート仲裁を受けるコミットであり言葉の表明ではない

**確定内容**：**アクション**はリポジトリに `.claude/kb/format-frozen/<コンポーネント>.toml`（内容：`version`、`frozen_at_commit`、`spec_hash`）をコミットすることである。**誰が判定するか**：人間が「準備ができたと感じた」から発効するのではない——ゲートは今回の差分が当該ディレクトリに触れたことを検知すると、より厳格な前置検査群（確定項目 6 のチェックリスト）へと自動的に切り替わり、すべてを満たした場合にのみ当該コミットの通過を許可する。**「今凍結を開始するか否か」のみを人間が決定し、「今回の凍結要求が成立するか否か」は機械が決定する**のであり、`.claude/singlefs-ai-sop/rules/show-me-test.md` の「出所によって区別せず、根拠によってのみ区別する」と同型である。

**射程**：当該ステージは通常時には実行されず（差分が当該ディレクトリに触れた時のみ実行）、日常のコミットを遅延させない；**長期間にわたりトリガー回数が 0 回となる宿命にあるが、これは欠陥ではなく正しい状態である**——ゲートが未実装ステージを明示的に列挙することと同種の誠実さである。⚠️ コミット契機であるのに対し、確定項目 7 のコンポーネント ① の凍結契機は「タイプ 2 コンテナを書き出す最初のイメージ」であり、これは実行であってコミットではない ⇒ 凍結前チェックリストはコンポーネント ① に対して実行経路を持たず、ゲートがそれに対して発火することはない。

**根拠**：実験なし：ゲートトリガーの契機と仲裁権の帰属を定めるものであり、定量可能な数値は存在しない；共有 `gate.sh` に既存の「プロジェクトローカルステージ」マウントポイントと整合させた。メイン agent による確定。

**未済の検査**：C225 (组件的冻结时刻绕开门禁触发)。

#### 確定項目 6：凍結前に完了必須のチェックリスト

**確定内容**：7 項目の前提条件であり、凍結要求の発行時に項目ごとに判定し、凍結ごとに再判定する。

| # | 項目 | 実施せずに凍結した場合の帰結 |
|---|---|---|
| 1 | 今回凍結対象となるコンポーネントのフィールド表において、kb 内のいかなる行にも「未定」「構想」「半定」「プレースホルダ」の記載がないこと | 変更途中の設計を凍結することになり、以降未確定項目の回答が補填されるたびに v1 の一部として追認されてしまう。⚠️ 本項目と第 7 項は異なる 2 つの尺度である：第 1 項は**既に書き出された表**の充足度を測り、第 7 項は**未だ書かれていない項目群**がそれに抵触するかを測る |
| 2 | I-1 (ブロック自己記述)、I-2 (チェックサム) 系（ブロック自己記述、チェックサム）の checker が実装済みかつ緑通過していること | 「checker こそが仕様」——checker がなければ「v1 とは何か」は文書内にのみ存在することになり、後続の変更を機械的に判定できなくなる |
| 3 | D9 (暗号化) 確定項目 12 の 4 つの予約フィールドについて、checker 上で「存在し幅が予約値に合致している」とアサート可能であること | 1 つでも確保を怠ると凍結後に incompat でパッチを当てるほかなく、既存の旧イメージは恒久的に当該フィールドを欠損する |
| 4 | I-3.1 (割り当て済み統計の合致) が実装済みであり、N>=5 回のフル実行ですべて緑通過していること | 会計の正当性を検証する唯一の手段がこの対比である；通過していなければ、凍結された台帳フィールドの設計が正しかったか否か一切検証されていないことになる |
| 5 | モデル突き合わせおよびクラッシュポイント再生がそれぞれ最低 1 回 PASS しており、**かつ実行対象が凍結直前のフォーマット指紋そのものであること** | さもなければ v1 が記述するフォーマットはいかなる自動検証も経ていないことになる |
| 6 | [checks-owed.md](../checks-owed.md) の C4 (项目规则清单不一致) および C8 (门禁范围判不出来) が清算済みであること | 凍結後に規則の乖離やゲートの遅延が放置されれば、よりリスクの高いフェーズにおいて障害へと直結する |
| 7 | リポジトリ内の未だ未確定であるすべての項目に「フォーマット変更要否」の判定が付与されており、「変更あり」または未判定の項目が 1 つも存在しないこと——今回の凍結要求において全件を個別に列挙しリスクを明示的に受容した場合を除く | この連鎖は相互に定義されており、「未確定」を残したまま暗黙裏に凍結することは既に決定したと詐称することと同義である。判定規則は確定項目 1 を参照 |

⚠️ **チェックリスト内の各項目の再確認において、値は常に対応項目の確定行からその都度取得しなければならず、手作業の転記を禁止する。** 凍結は不可逆であり、数値を誤記してもテキスト上いかなる違和感も生じないためである ⇒ **再確認時は当該項目の確定行を逐字確認すること**；機械検査の形態と前提条件は C62 (分项定案值在 kb 里分叉) に記録されており、それが実装されるまでは本工程は人手に依存する。

**第 1 項および第 7 項の実行可能形態**：第 1 項は今回凍結対象のフィールド表を走査する——[layout/01-first-txn.md](../layout/01-first-txn.md) のディスク上順序で行ごとに列挙された表がその既定形態であり、その**「幅」列および「状態」列**を走査し、`未定` / `预想` / `半定` / `占位` の**いずれか**が現れれば直ちに赤判定とする。第 7 項はリポジトリ内の未確定項目における「フォーマット変更要否」判定を走査する。これは 2 つに分割される：**記載の有無**についてはゲートステージ `.claude/gate.d/doc-decisions.sh` の blocking-verdict セルの第 2 の尺度であり、第 1 の尺度（新規プールでのファイル作成バイトの変更有無）と同一の解析・同一の権威リストを共有し、それぞれ数値を報告し個別の解決策を提示し常時稼働する；**赤判定**の側面（「変更あり」または未記載なら赤判定、全件列挙の上リスク明示受容の場合を除く）は凍結要求発行時の処理であり、前提は C225 (组件的冻结时刻绕开门禁触发) である——凍结要求という契機はコンポーネント ① に対して実行経路を持たない。いずれの項目も**第 2 のリストを別途作成しない**——`.claude/singlefs-ai-sop/rules/kb-discipline.md`：同一事実に対する権威ある記録は 1 箇所のみ許容される。

**射程**：第 1 項の用語集は条文内に規定し、スクリプト作成者の推測に委ねない——バイト表で現行使用されている語は「**預想**」および「**半定**」である（extent ツリーノードの行は原文ママ「**預想 162** = 86 + 2 × 24 + 28」、状態列「半定」、key 区間を含めるか否かは未定であり、含めれば 162、含めなければ 114 であり 48 バイトの差が生じエントリ領域の開始位置もずれる）、それこそが第 1 項が阻止すべき「変更途中の設計」に他ならず、「未定」のみを走査する方式では誤って通過させてしまい、第 4 層の凍結要求が 48 バイトの未確定計測条件を v1 として追認してしまうことになる。チェックリストが判定するのは前提条件の充足有無であり、いかなる項目の内容の当否も判定しない。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：第 1 項が走査すべきフィールド表とは、それが行ごとに出力し行ごとに検証した表そのものであり、「幅」列および「状態」列における「預想」「半定」という 2 語は実在の対象を有している。
- メイン agent による確定（第 1 項の語彙が 2026 年に絞り込まれた経緯は変更履歴にある）；第 7 項の判定規則は確定項目 1 にある。

**未済の検査**：C62 (分项定案值在 kb 里分叉)；C8 (门禁范围判不出来)（チェックリスト第 6 項のもう半分である C4 (项目规则清单不一致) は清算済み）。

#### 確定項目 7：凍結は 4 層に分割し、4 層外に独立した 3 つの凍結コンポーネントを設ける

**確定内容**：4 層には厳格な依存順序が存在する。

```
第 1 層：システム構成 + ブロックヘッダ（自アドレス / generation / ツリー ID / fsid / magic）
    ↓
第 2 層：ポインタフィールドレイアウト（D4 チェックサム + D9 の MAC / nonce / 暗号文チェックサム）
    ↓
第 3 層：各 keyspace の key 符号化（D8 の locality_id / inode / offset）
    ↓
第 4 層：インデックスノード内部レイアウト（ノードサイズ規則、bset 形式、D11 のメッセージバッファ）
```

依存順序の理由：第 1 層は「メタデータブロックは自己記述でなければならない」という物理的前提であり、後続のすべての層はその上に構築される；第 2 層は D9 (暗号化) の MAC / nonce / 暗号文チェックサムが「事後追加不能」であることにより固定され、これに依存するツリー構造に先行しなければならない；第 3 層の key 符号化が定まって初めて、ノード内部でこれら key をどう編制するかの議論基盤が成立する。

**段階的凍結のリスクは明示的に受容されなければならない**：早期に凍結された層でフィールド確保を怠った場合、後続は incompat によるパッチ適用を余儀なくされ、全体設計時に確保すべきであった余力を前倒しで喪失することに等しい。損切り規則は唯一つである：**第 1 層および第 2 層の凍結前に D9 (暗号化) の全 4 つの予約フィールドを内包していなければならない**。これらは「いずれ必要となることが明白な」フィールドであり、第 3 層・第 4 層での補填を期待することはできないためである。

**4 層外の独立凍結コンポーネント**：4 層のいずれにも属さない恒久的なバイト列が 3 つ存在し、それぞれを独立したコンポーネントとして立脚させ、C9 (冻结后偷改格式) の `spec_hash` はコンポーネント単位で記録し、4 層側には影響を与えない：

- ① **パック記録ユニットのペイロードレコード形式**、パック記録種別ごとに登録：種別 2 の 140 バイト inode レコード表（D8 (コアインデックス構造) 確定項目 6）、凍結契機は種別 2 コンテナを書き出す最初のイメージ；種別 1 のもの（幅 40、D18 (ブロックに含める情報) 確定項目 24）は相互に独立しており、種別 2 と一括凍結は行わない。
- ② **統計量登録表**（タグ値 + 次元 + マージ規則、D5 (スナップショット・領域会計メカニズム) 確定項目 3 / 4）——マージ規則はディスク上セマンティクスであり、2 つのリーダーが同一のバイト列に対し異なる規則を適用すれば異なる値を算出してしまう；C71 (不认识的统计量标签静默忽略) は「未認識タグ」のみを保護し、「タグは認識するが誤った規則を適用する」事態は防げない。
- ③ **journal 記録形式**（レコードヘッダ、指定項目、ペイロードチェックサムの適用範囲、トランザクション境界）：凍結前提条件は第 2 層および配置先粒度セルより先行せず、凍結契機は当該 toml を提出するコミットとし、新規レコード形式は「新規レコード種別 + システム構成 log-incompat ビット」を採用する。全文は D16 (パブリッシュセマンティクス) 確定項目 3 にある。

**射程**：4 層の図は再編しない：D21 (権威態と派生態の境界) 確定項目 10 は状態区分を構造単位の判定へと変更した；第 4 層において会計ツリーのコード 2 ノードは権威態、他のツリーの内部ノードは派生態であり、純粋な層は存在しないため、凍結除外の単位は構造体であり層ではない（各構造体がどの層に属し、状態区分が何であり、凍結から除外されるか否かは `.claude/kb/freeze-layer-membership.md` に登録され、ゲート doc-decisions の freeze-layer-membership セルが判定する）；3 つのコンポーネントの spec ファイルは存在しない；コンポーネント ① の凍結契機は実行でありコミットではないため、確定項目 5 の遮断ゲートはそれに対して発火しない。ツリーテーブルユニットおよびインスタンス表ユニットがどの層に属するかは未だ条項が存在しない。

**根拠**：実験なし：層の分割と依存順序は「後から追加できない要素はどれか」から導出した順序付けであり、定量可能な数値は存在しない；損切り規則の対象（D9 (暗号化) の 4 つの予約フィールド）は D9 (暗号化) 確定項目 12 にある。メイン agent およびユーザーによる確定、原文は変更履歴にある。

**未済の検査**：C119 (冻结组件没有 spec 文件)；C225 (组件的冻结时刻绕开门禁触发)；C246 (树表单元没有冻结层归属)。

#### 確定項目 8：凍結後に失われる自由

**確定内容**：

| 流動フォーマット期の自由 | 凍結後に何へと変化するか |
|---|---|
| 構造を「解体して再構築」 | 凍結済みフィールドの幅 / 存在 / 意味は変更不可；追加のみ許容され、廃止フィールドは「読み出しサポート維持、書き込み停止」とする |
| 設計の誤りを発見時に互換性を残さず本文を直接修正 | 決定記録レベルでは依然として覆すことが可能であるが、**ディスク上バイト列レベルでは互換性コストがゼロから非ゼロへと転換する** |
| mkfs の自動選択規則でアルゴリズムを随時変更 | 凍結済みコンポーネントの選択規則自体も固定される；アルゴリズム変更により新旧イメージのデフォルト値が乖離するため、新規フォーマットバージョン番号の下で表現されなければならない |
| 実験結果が芳しくない場合にフォーマットを直接改訂 | 新規 feature bit を通じて新規ポリシーを導入し、旧値の解釈能力を保持しなければならない |
| フォーマットに合わせて checker を自由に変更 | **凍結部分について、checker の判定は事実上の互換性規約となる**——凍結フィールドに対する checker の解釈を密かに改変することは、フォーマットを密かに改変することと同等に危険である |

**射程**：凍結後にいかなる行為が非正当となるかを述べるにとどまり、個別の具体的変更がいずれのグレードに属するかは判定しない——それは確定項目 9 の所管である。最初の外部ユーザーが現れるまでフォーマットは流動的であり、本表はそれ以前には一切効力を持たない (`.claude/rules/format-evolution.md`)。

**根拠**：実験なし：「対外的な恒久公約は変更できない」という制約を項目ごとに展開したものであり、定量可能な数値は存在しない。最終行の規定（checker の判定も規約となる）は D13 (検証ルート) の「checker こそが仕様」と同源である。メイン agent による確定。

**未済の検査**：なし。

#### 確定項目 9：凍結後の変更判定表

**確定内容**：

| 変更分類 | 旧リーダーはどう振る舞うか | グレード | bump | 理由 |
|---|---|---|---|---|
| 予約領域 / TLV 末尾へのフィールド追加 | 無視し、正常動作を継続 | compat | 否 | セマンティクス依存のない純粋な追加であり、「未知のフィールドを無視する」セーフティネットで十分であるため |
| フィールド追加であり、その欠落により旧リーダーが**他の既存フィールド**を誤計算する場合 | **誤計算し、データを破損する恐れがある** | incompat | 是 | セマンティクスが結合した場合、セーフティネットは破綻するため |
| 既存フィールド幅の拡張 | 旧幅で解析し、範囲外読み出し / 位置ずれ | incompat | 是 | 幅はレイアウトの一部であり、旧コードのオフセット計算が完全に狂うため |
| 既存フィールドの意味の変更 | 旧解釈で処理し、**エラーを出さずに**不正な挙動を発生 | incompat、かつ**旧位置を廃棄し再利用禁止** | 是 | 位置の再利用は新旧リーダーの識別を不能とし、最も危険な分類であるため。実例 2 件（D18 (ブロックに含める情報) 確定項目 11）：ユニット種別の追加（登録表コード 4–15）は種別識別セグメントの解釈計測条件を変更する；共通プレフィックスオフセット 7 の常に 0 である flag ビットへの意味付与（旧リーダーは非ゼロビットを一律拒絶） |
| レコード種別の追加（新規 key 種別） | 読み出し専用トラバースが誤判定の恐れ；整理 / 書き込みが理解不能なレコードを誤削除する恐れ | compat_ro | 是 | **「読み出し専用の安全性」と「書き込み可能の安全性」は独立した二重の保証**であり、個別に判定しなければならないため。⚠️ パック記録ユニット内のパック記録種別は**本行に従わない**（D18 (ブロックに含める情報) 確定項目 11）：墓石系または inode 系のレコードのスキップは読み出し専用の回答を変化させるためデフォルトは incompat とする；登録時に「スキップしても読み出し専用の回答が一切不変である」と証明された種別のみ compat_ro へ降格し、compat_ro リーダーはスキャン再構築の実行を禁止する |
| keyspace の追加 | ツリーテーブルが day-1 から開放リストの場合：新規ツリーが見えないのみ；ツリー数がハードコードの場合：ツリーテーブルを読み出せない | **compat**（確定項目 2 によりツリーテーブルは day-1 から開放リスト化）。開放ツリーテーブルを確保済み → compat；未確保 → incompat | 是 | **「ツリーテーブルが拡張可能か否か」は凍結時に決定されなければならず**、ツリー追加時に未確保であることが判明する事態は許容されないため |
| ソート順序の変更 | 二分探索 / トラバース結果の誤り、**エラーなし** | incompat | 是 | ソート順序はインデックスの本質であり、ログ構造ノードの並べ替えは順序の不変性に直結するため |
| チェックサム / MAC アルゴリズムの変更 | D9 (暗号化) のアルゴリズム種別フィールドが確保済み → 未認識の種別に遭遇時は明示的に失敗 | 確保済み → compat_ro；未確保 → incompat | 是 | D9 (暗号化) 確定項目 12 第 4 項はこの種の変更のために用意された流路であるため |
| フィールドの廃止 | 影響を受けず、旧リーダーは旧方式で解釈を継続 | compat、かつ**パース用コードを恒久保持** | 否 | 一度ディスクに書き込まれたフィールドは、旧イメージが 1 つでも存在する限り恒久的に解釈可能でなければならないため |

**射程**：「旧リーダーがどう振る舞うか」を判定するものであり、「その変更を行うべきか否か」は判定しない。第 1 行の規定（予約領域への追加は compat）は真に予約領域が存在する場合にのみ成立し、インデックスノードヘッダには 2 バイトの予約領域しかなく、格納可能なフィールドは極めて限られる。

**根拠**：

- E59 (缓冲消息能不能只从单元重算)：墓石系レコードを 1 つスキップすると削除済み key がすべて復活し、復活数は削除回数に正確に一致する——「パック記録種別は compat_ro の行に従わずデフォルトは incompat である」という文言の定量的根拠。
- メイン agent による確定、行ごとの理由は表内に記載。

**未済の検査**：C102 (索引节点头没有预留区，加字段不是 compat)；C97 (改字段含义而不改宽度，全仓没有一处会红)。

#### 確定項目 10：feature bit のビット数・会計・割り当て権

**確定内容**：

- **ビット数**：3 つのビットマップ（各 **256 ビット**、`u64[4]`）をシステム構成に格納。ビットはフィールド幅より遥かに低コストであり（3×32 バイトはシステム構成にとって無視可能）、**予約は広めに確保する**。
- **会計**：`.claude/kb/feature-bits.md` に `ビット番号 | カテゴリ | 名称 | 導入バージョン | 導入コミット | 状態 | セマンティクス一言` を列挙；ビット番号の唯一の登録場所は確定項目 4 の表とする。
- **割り当て権は「先行マージ優先」とし、承認制としない**。ゲートによる強制：ビット番号は厳格に単調増加し、番号の飛び越し予約を禁止し、表に存在しないビット番号を参照するコードを禁止——機械的仲裁を行う。
- **一度使用したビットは再利用を禁止する。** 根拠は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) の番号非回収規定より**さらに硬い**：番号は検索キーであり、回収による誤導は「文書を読む人間またはモデル」にとどまる；**feature bit はディスク上に物理的に実在したビットであり**、回収して新機能に再割り当てした場合、旧イメージを新コードでマウントした際に当該ビットが「新機能 X が有効」と誤読され——**ファイルシステムが実際に誤作動を起こす**（例：存在しない暗号化特徴を存在すると誤認し、存在しない鍵を探索する）。廃止されたビットは「廃止直前の最終使用セマンティクスに恒久固定」と表記し、新機能への付与を禁止する。

**射程**：ビットの分割・記録・回収可否を所管し、各ビットのセマンティクス（要求した決定に属する）は所管しない。

**根拠**：実験なし：ビット数および再利用禁止は「旧イメージが新コードにより読み出される」という物理的事実から導出したものであり、定量可能な数値は存在しない；256 ビットの空間コストはシステム構成にとって無視可能であり、本セルは節約競争を行わない。メイン agent による確定。

**未済の検査**：C183 (每套布局一个 incompat 位而盘上差不出字节)。

#### 確定項目 11：凍結に伴い整備すべき赤判定検査

**確定内容**：

1. **フォーマット凍結規約**：凍結済みコンポーネントごとに `spec_hash` を記録；ゲートは再計算して照合し、不一致時は今回の差分が「`feature-bits.md` に新規行が存在するか、または `version` が明示的に変更されている」ことと「`decisions.md` の対応条目が同一コミット内で変更されている」ことの**双方**を満たすことを要求する。いずれか一方でも欠落すれば赤判定とする。
2. **後方互換性リグレッション（ゴールデンイメージ）**：あるコンポーネントのバージョン変更ごとに**当時のコード**を用いてイメージを生成・アーカイブし、ゲートは**現行コード**を用いて全履歴イメージを読み出し、checker を実行し、期待されるコンテンツダイジェストを照合する。イメージ本体は **git に含めない** (`.claude/singlefs-ai-sop/rules/command-safety.md`：テストイメージのリポジトリ内格納を禁止)；リポジトリにはマニフェスト `.claude/kb/golden-images.toml` のみをコミットする；イメージは `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}` に配置する。マニフェストは**追加のみ許容**される——特定バージョンのリグレッション削除はその互換性公約を密かに放棄することに等しく、正式な決定記録を経なければならない。

**射程**：双方が未実装であり、番号は未済の検査表にある。第 2 項は「現行コードが過去のイメージを理解できるか」のみを比較し、「当時のイメージが正しく書き出されていたか」は比較しない；第 1 項は宣言に固定されバイト列には固定されない；双方は「最初のゴールデンイメージ自体が誤ったコードで出力された」事態を防ぐことはできない。

**根拠**：実験なし：2 つのゲート検査がいかなる判定を行うべきかを定めるものであり、判別力の自己証明は実装を待つ必要がある；これらが防ぐべき障害形態（フォーマットの不正改変、互換性公約の密かな放棄）は C9 (冻结后偷改格式) および C227 (黄金镜像的基准是它自己那份代码写的) にそれぞれ 1 条存在する。メイン agent による確定。

**未済の検査**：C9 (冻结后偷改格式)；C227 (黄金镜像的基准是它自己那份代码写的)。

````

**出处 `.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md:236-239`（整段抄，未转述）**

```markdown
## 履歴バージョン

D15 (フォーマット凍結ポリシー) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:1-6`（整段抄，未转述）**

```markdown
## D15 格式冻结政策 —— 已定

**中文** · [English](15-format-freeze-policy.en.md) · [日本語](15-フォーマット凍結ポリシー.ja.md)

D15（格式冻结政策） 管两个问题：**什么时候允许说「format v1 frozen」**，以及**冻结之后新增东西走什么通道**。它是 D11（索引节点要不要留消息缓冲区） / D12（目标介质） / D14（双轨（大小文件 / 持久临时）） 的共同前置——「格式分支永久」这条成本的当期值为零、冻结后为无穷，那几条决策的真实代价由它定义。不管的：每一条字段本身该定什么值归各自那条决策；第一个外部用户出现之前格式是软的，那条政策在 `.claude/rules/format-evolution.md`；`feature-bits.md` 里每一位的语义归申请那一位的决策，D15（格式冻结政策） 只管位号怎么分、能不能回收。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:25-40`（整段抄，未转述）**

```markdown
#### 已定项 1：第 7 项的对象是未定分项，不是决策名单

**定案**：三条。

1. **对象是全仓每一条仍未定的分项，不是一份决策名单。** 每条未定分项要带一句判定——**动不动格式**：它的答案会不会改动任何盘上字节，或改动已有字节的**解释口径**（口径这半是 C97（改字段含义而不改宽度，全仓没有一处会红） 的形态：宽度一个字节不变而含义变了）。三个取值：**动 / 不动 / 界不定（视同动）**；**没写过视同动**。
2. **判定**：冻结请求发起时，凡判「动」或没判过的，判红。要分批冻结就在请求里**逐条列名并显式接受风险**（形态照已定项 7 那条「分批冻结的风险必须显式接受」），**列漏一条判红**。
3. **留白宽度默认 0**，举证责任在要留非零宽度的一方，必须过 `.claude/rules/fs-design.md` 的格式分支判据（逐字：「一条格式分支值不值得，看它能不能把『用错了』变成『挂不上』」）。

问法先定读法（`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 硬要求 7：待定项的问法必须只有一种读法）：不取「逐条钉死六格的取值」（每一格的内容住在各自那条决策里，复制一份就是同一事实两处记录），取「定第 7 项的判定规则」；第 7 项是**逐次冻结重判**，因为清单是 commit 触发的前置检查，每一次冻结请求都跑一遍。

**射程**：留白宽度默认 0 这一条管的是**给一个已定不做的特性占位**，不管结构里的**通用预留区**——通用预留区由已定项 9 判定表第一行（新增字段在预留区 / TLV 尾部 ⇒ compat 不 bump）与 C102（索引节点头没有预留区，加字段不是 compat） 管，判据不同；把两者混成一条会推出「哪儿都别留」，而 C102（索引节点头没有预留区，加字段不是 compat） 正在往相反方向还账。升成规则的只有「默认 0 + 举证责任倒置」这套程序，任何一条走到「v1 不含此特性」那一支的条目，都要自己再过一遍格式分支判据。谓词不按层切：退出冻结按结构判、不按层判（已定项 7 的射程），且成员不齐——已定项 7 里那三个独立冻结组件不属于四层里的任何一层 ⇒ 按层切的谓词在它上面没有定义；按「动不动格式」这个二值谓词切，这两条毛病都碰不到，而且不假设层与层之间可以独立判定。D26（后台整理与放置回收） 已定项 4 那一格照旧由 D26（后台整理与放置回收） 自己拥有，D15（格式冻结政策） 无权替它答。判定输入一律现取，不在这里存副本：`grep -c '未定\*\*' .claude/kb/decisions.md` 数全仓还有几条未定分项（分项清单行末现为「—— **未定** · 未实现」这一形，不再以「未定**」收尾），`bash .claude/gate.d/doc-decisions.sh --check decision-items-sync` 判那张清单是不是正文的忠实投影。

**依据**：无实验：定的是门禁判定规则的对象与默认值，没有可量的量。撑它的是两条事实——名单形态漏得到（D22（单元原子性怎么合成） 已定项 9 是**系统配置的字段表**，而系统配置就是第 1 层；D19（块指针的结构与宽度预算）已定项 6 是**中央映射条目的字段表**，落在第 2 层——两条都不在那份六条名单里，冻这两层时名单一个字都说不上话），以及靠一个「格式级」标签去兜是兜不住的（编号的登记位在各决策正文的标题行、不在索引页，`grep -rn 格式级 .claude/gate.d/ .claude/scripts/` 零命中 ⇒ 零对象、零实现）。主 agent 定案，按三方论证；一条不成立的论据写在变更史里，免得再用。

**欠**：C62（分项定案值在 kb 里分叉）；C102（索引节点头没有预留区，加字段不是 compat）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:41-53`（整段抄，未转述）**

```markdown
#### 已定项 2：树表 day-1 做成开放列表

**定案**：树表 day-1 就做成开放列表。依据在已定项 9 那张判定表里：开放列表下「新增一棵 keyspace」对旧读者只是**看不到新树**（档次 compat，优雅降级）；树数量硬编码则是**读不出树表**（incompat，直接坏）。**开放列表严格占优**，代价只是树表单元里多一个表长字段加一段可变长表——与已定项 10「预留要往宽了留」是同一条理由。⇒ 加 keyspace 是本工程明写的扩展目标，不是意外，判定表里那一行的档次因此钉死为 **compat**。

**射程**：定的是「树表是开放列表」，与树表住哪无关——树表住根记录的间接层（D22（单元原子性怎么合成） 已定项 7 逐字「树根一律住树表单元，不平铺进根槽」、已定项 12 定「系统配置不带自己的树表单元指针」；D21（权威态与派生态的分界） 把根记录 + 树表单元 + 实例表单元一并划进「根」）。它**必须在第 1 层冻结时就做进去**，不能等到要加树才发现没留；而树表单元自己的冻结层归属仍欠。

**依据**：

- E142（新池新建文件的干跑）：新池新建文件写出的树表单元按条数声明（mkfs 那版 0 条、事务那版 11 条，是退役树加进之前的现值；退役树加进之后事务那版 12 条），表长是数据不是编译期常数——开放列表这个形态在写出来的字节上成立。
- 用户定案，原话在变更史。

**欠**：C246（树表单元没有冻结层归属）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:54-73`（整段抄，未转述）**

```markdown
#### 已定项 3：`spec_hash` 恒对 kb 字段表的完整投影求，不切到 Rust 结构体

**定案**：`spec_hash` 恒对 `format-spec/<组件>.toml` 求。那份 toml 是 **kb 字段表的完整投影**——含**行序**（盘上顺序）与**非标量的盘上语义规则**（已定项 7 里组件 ② 的合并规则以数据形式写进去），不是已登记的那批标量常量的集合。有代码之后新增的**不是换哈希源**，是两道检查：① spec 与代码之间的**相等性检查**（宽度与顺序都比）；② 黄金镜像回归（已定项 11 第 2 条）。

四条理由：

1. **方向**。D13（验证路线） 已定项 5 逐字：「**checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块**，生成器从 kb 读、两边都只消费，任何人不许手改」⇒ 权威在 kb，Rust 结构体是**下游消费者**；对结构体求哈希，是把永久契约的指纹锚在派生物上。
2. **哈希是变更探测器，不是两份工件的相等性检查。** 对结构体求哈希抓不到「结构体与 kb 不一致」，它只在结构体**变化**时报警；真正抓那一类错的是相等性检查，而相等性检查要求 spec 侧存在 ⇒ 切过去不但没多买到东西，还把要比对的另一侧拆掉了。
3. **语言绑定**。D17（实现分层与第三方管道） 已定项 3 逐字：「判定必须能被第三方自己复核，否则『合规』就退化成『我们说你合规』」。指纹锚在 Rust 源码文本上，第三方管道复算不了它。
4. **落地**。三个组件的 spec 文件写得出来，哈希机制可以远早于第一次冻结开始生效；而已定项 7 里组件 ① 的冻结时刻是第一个写出类型 2 容器的镜像 ⇒ 切换会被逼到与第一次真实冻结同一个提交里完成，零试跑期。

**射程**：spec 只投影标量是不够的，必须含行序，相等性检查也要比顺序——[layout/01-first-txn.md](../layout/01-first-txn.md) 的单元头逐行列着六个宽度都是 8 的相邻字段（五元组的树 ID / 对象 ID / 对象出生代 / 锚点偏移，以及诞生代号、fsid），把序列化时任意两个对调，所有标量与 `DATA_UNIT_HEADER_BYTES` 一个都不变、盘上字节全错位，落进已定项 9 判定表「改变已有字段含义」那一行。D13（验证路线） 已定项 5 那条「生成器只发射标量值」的边界管的是 checker 与实现之间的共享物，而 `format-spec/<组件>.toml` 是**门禁工件**、运行期一个消费者都没有 ⇒ 不在那条边界的射程里，而且一张带行序的字段表既没有分支也没有算术。**抓不到的明说**：spec 与实际写出的字节之间，哈希不构成绑定——第一份黄金镜像若本身就是错代码写出来的，回归会把错的那份当基准；堵它的是 checker 这个独立解析器与一致性套件的判定程序，不是 `spec_hash`。spec 只能手写，没有任何从 kb 字段表投影到 toml 的机制。分批冻结「早冻层漏留字段」的**风险大小**给不出量化形式，它依赖尚不存在的 D8（核心索引结构） / D11（索引节点要不要留消息缓冲区） 具体布局，只能给出已定项 7 那条止损规则，残余风险无法量。

**依据**：

- E142（新池新建文件的干跑）：单元头那六个宽度都是 8 的相邻字段在写出来的字节里逐行排着，对调任意两个而全部标量不变这件事是可构造的——spec 只投影标量罩不住字段顺序。
- 三方论证：反推腿按跑前写死的失败条款判「spec 只投影标量」那条臂出局，定案取的是收严后的形态（spec 必须含行序），新形态比旧形态多要求一样东西、没少要求任何东西。主 agent 定案。

**欠**：C227（黄金镜像的基准是它自己那份代码写的）；C226（spec 是手抄的，没有从字段表生成）；C119（冻结组件没有 spec 文件）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:74-97`（整段抄，未转述）**

```markdown
#### 已定项 4：feature bit 的位分配登记表

**定案**：三个 bitmap 各 256 位（已定项 10 的三张位图），位号的唯一登记位是这一节的登记表。「不认识不许挂」要对默认布局也生效，所以第一条线自己占一位、mkfs 起就置上——不取「第一条线不占位」那条（那样不认识 SSD 布局的读者照样能挂）。

| bitmap | 位 | 含义 | 出处 |
|---|---|---|---|
| incompat | 0 | 第一条纯 SSD 布局线 | D12（目标介质） 已定项 5；D12（目标介质） 已定项 1「每套布局一个 incompat 位」；退役：带回退见证、系统配置不带 F 的那一版，读者一律按不认识的位拒（用户 2026-09-26） |
| incompat | 1 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行） | D12（目标介质） 已定项 5；用户 2026-09-26 定「换 incompat 位，挂不上」 |
| incompat | 2 | inode 记录带截断字段（偏移 120 起 T 与 N_w；不带位 2 的池偏移 120–135 必须全零）；mkfs 起就置上，可写挂载把不带位 2 的旧池在取号那一写起升上 | 用户 2026-10-17 定新开一位，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节；登记与升位的写回判决 `research/prompts/m4-validate-code-r1-main-verification.md` 第二节「截断字段与位 2」那一行、第三节第一行 |
| incompat | 3..255 | 未分配；每开一条布局线追加一位 | D12（目标介质） 已定项 1 |
| compat_ro | 0..255 | 未分配 | — |
| compat | 0..255 | 未分配 | — |

**射程**：只定位号怎么分与第一位的归属，每一位的语义归申请那一位的决策。新池新建文件的字节：系统配置 feature bits 那 96 字节从全 0 变成 incompat 位 1 置 1，位图小端、位 1 是第一个字节从最低位数起的第 2 位（D22（单元原子性怎么合成） 已定项 13）。这一项没有走三方论证，主 agent 的推荐与用户的选择都可推翻。

**依据**：

- E142（新池新建文件的干跑）：系统配置 feature bits 那 96 字节按这一项写出来，它的空白 G12 随之收口，[layout/01-first-txn.md](../layout/01-first-txn.md) 同步改。
- 用户定案，原话在变更史；定案之前那一格的原文也在变更史，含它当天写的「改字节：否」——那句已被这一项的定案推翻。
- 用户定案 2026-09-26：位 0 退役、布局身份换成位 1，旧镜像（带回退见证、系统配置不带 F）挂不上（三方判决 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第二节 K7：「见证表非空就拒可写挂载」出局，只读恢复 13 个改坏深度里 4 个落到被回退掉的那一版；改系统配置格式版本号挡不住，今天的读者不核版本号），原话在变更史；无实验：旧读者对新位挂不上在原型上量过，新读者对旧位拒是推的；「改系统配置格式版本号挡不住，今天的读者不核版本号」这一句已过时：2026-09-27 起读者遇系统配置格式版本不是 1 整池拒绝挂载（2026-10-21 定，D22（单元原子性怎么合成） 已定项 9）。
- 用户定案 2026-10-17：新开 incompat 位 2（inode 记录带截断字段），三方判决 `research/prompts/m4-trunc-r2-main-verification.md` 第六节；位 2 登记与可写挂载升位的写回判决 `research/prompts/m4-validate-code-r1-main-verification.md` 第二节「截断字段与位 2」那一行、第三节第一行；原话在变更史。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:98-107`（整段抄，未转述）**

```markdown
#### 已定项 5：冻结是一次受门禁仲裁的提交，不是一句话

**定案**：**动作**是向仓库提交 `.claude/kb/format-frozen/<组件>.toml`（内容含 `version`、`frozen_at_commit`、`spec_hash`）。**谁判**：不是人「觉得可以了」就生效——门禁一旦发现本次 diff 触碰了这个目录，自动切换到一套更严的前置检查（清单在已定项 6），全部满足才允许该 commit 通过。**人只决定「现在要不要发起冻结」，机器决定「这次冻结请求成不成立」**，与 `.claude/singlefs-ai-sop/rules/show-me-test.md`「不按来源区分，只按证据区分」同构。

**射程**：那个阶段平时不跑（只在本次 diff 触碰这个目录时才跑），因此挂上去也不拖慢日常提交；**它注定常年触发 0 次，这不是缺陷，是正确状态**——与门禁显式列出未实现阶段是同一种诚实。⚠️ 它是 commit 触发的，而已定项 7 里组件 ① 的冻结时刻是「第一个写出类型 2 容器的镜像」，那是一次运行、不是一次提交 ⇒ 冻结前清单对组件 ① 没有执行路径，门禁永远不会为它点火。

**依据**：无实验：定的是门禁触发时机与仲裁归属，没有可量的量；形态与共享 `gate.sh` 已有的「项目本地阶段」挂载点对齐。主 agent 定案。

**欠**：C225（组件的冻结时刻绕开门禁触发）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:108-134`（整段抄，未转述）**

```markdown
#### 已定项 6：冻结前必须完成的清单

**定案**：七项前置，冻结请求发起时逐项判，逐次冻结重判。

| # | 项 | 不做它冻结会怎样 |
|---|---|---|
| 1 | 本次要冻的组件，它的字段表在 kb 里没有一行写着「未定」「预想」「半定」「占位」 | 冻结的是一个还在动的设计；此后每次补答一个待定项都会被追认成 v1 的一部分。⚠️ 它与第 7 项是两把尺子：第 1 项量**已经写下来的那张表**齐不齐，第 7 项量**还没写下来的那些分项**动不动它 |
| 2 | I-1（块自描述）、I-2（校验和） 类（块自描述、校验和）checker 已实现且跑绿 | 「checker 即规范」——没有 checker，「v1 是什么」只存在于文档里，后续改动无法被机械判定 |
| 3 | D9（加密） 已定项 12 的四项预留字段在 checker 里可断言「存在且宽度符合预留」 | 漏留一项，冻结后只能靠 incompat 打补丁，且已有旧镜像永远缺这个字段 |
| 4 | I-3.1（已分配统计对得上） 已实现且跑过 N≥5 轮全绿 | 记账正确性唯一的验证手段就是这条对照；没跑过，冻结的账本字段是不是设计对了完全没被检验 |
| 5 | 模型对拍与崩溃点重放 各至少跑过一轮 PASS，**且跑的正是即将冻结的那份格式指纹** | 否则 v1 描述的是一份从未被任何自动化验证碰过的格式 |
| 6 | [checks-owed.md](../checks-owed.md) 的 C4（项目规则清单不一致） 与 C8（门禁范围判不出来） 已实现 | 冻结后规则脱节、门禁越跑越慢却没人量化，会在风险更高的阶段变成事故 |
| 7 | 全仓每一条仍未定的分项都带着「动不动格式」那句判定，且没有一条判「动」或没判过——除非本次冻结请求把它们逐条列名并显式接受风险 | 这条链互相定义，沉默地带着「待定」冻结等同于假装它已经决定了。判定规则见已定项 1 |

⚠️ **清单里每一项的复核，取值一律从对应分项的定案行现取，不许手抄。** 冻结是不可逆的，而抄错一个数在文本上没有任何地方看起来别扭 ⇒ **复核时逐字打开那条分项的定案行**；机检形态与它的前置记在 C62（分项定案值在 kb 里分叉），在那条实现之前这一步靠人。

**第 1 项与第 7 项的可执行形态**：第 1 项扫本次要冻组件的字段表——[layout/01-first-txn.md](../layout/01-first-txn.md) 那张按盘上顺序逐行列字段的表就是它的现成形态，扫它的**「宽度」列与「状态」列**，出现 `未定` / `预想` / `半定` / `占位` **任一**即判红。第 7 项扫全仓未定分项里那句「动不动格式」的判定。它拆成两半：**写没写**那一半是门禁阶段 `.claude/gate.d/doc-decisions.sh` 的 blocking-verdict 格的第二把尺，与那一道的第一把尺（改不改新池新建文件的字节）共用同一次解析、同一份权威清单，各报各的数、各带各的出路，常跑；**判红**那一半（凡判「动」或没写过即判红，除非逐条列名并显式接受风险）是冻结请求发起时的事，前置是 C225（组件的冻结时刻绕开门禁触发）——冻结请求这个触发对组件 ① 没有执行路径。两项都**不另建第二份清单**——`.claude/singlefs-ai-sop/rules/kb-discipline.md`：同一事实只许一处权威记录。

**射程**：第 1 项的词表写进条款、不留给写脚本的人猜——字节表今天用的词是「**预想**」与「**半定**」（extent 树节点那一行逐字「**预想 162** = 86 + 2 × 24 + 28」、状态列「半定」，带不带 key 区间还没定，带 162、不带 114，差 48 字节，条目区起点跟着挪），那正是第 1 项要拦的「还在动的设计」，而按只扫「未定」的形态它一定放行，第 4 层的冻结请求就能把一个 48 字节的未决口径追认成 v1。清单判的是前置齐不齐，判不了任何一条分项的内容对不对。

**依据**：

- E142（新池新建文件的干跑）：第 1 项要扫的那张字段表就是它逐行写出来又逐行核过的那张，「宽度」列与「状态」列上「预想」「半定」这两个词有真实对象。
- 主 agent 定案（第 1 项词表 2026 年那次收窄的经过在变更史），第 7 项的判定规则在已定项 1。

**欠**：C62（分项定案值在 kb 里分叉）；C8（门禁范围判不出来）（清单第 6 项的另一半 C4（项目规则清单不一致） 已还清）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:135-164`（整段抄，未转述）**

````markdown
#### 已定项 7：冻结分四层，四层之外另有三个独立冻结组件

**定案**：四层有严格依赖顺序。

```
第 1 层：系统配置 + 块头（自身地址 / generation / 树 ID / fsid / magic）
    ↓
第 2 层：指针字段布局（D4 校验和 + D9 的 MAC / nonce / 密文校验和）
    ↓
第 3 层：各 keyspace 的 key 编码（D8 的 locality_id / inode / offset）
    ↓
第 4 层：索引节点内部布局（节点大小规则、bset 形态、D11 的消息缓冲区）
```

依赖方向的理由：第 1 层是「元数据块必须自描述」的物理前提，后面所有层都建在它之上；第 2 层被 D9（加密） 的 MAC / nonce / 密文校验和「事后加不进去」钉死，必须先于依赖它的树结构；第 3 层的 key 编码定了，节点内部怎么组织这些 key 才有讨论基础。

**分批冻结的风险必须显式接受**：早冻的层若漏留字段，后面只能走 incompat 打补丁，等价于提前烧掉一次本该整体设计时留的余量。止损规则只有一条：**第 1、2 层冻结前必须已包含 D9（加密） 全部四项预留**，因为它们恰好是「已知迟早要用」的字段，不能指望第 3、4 层再补。

**四层之外的独立冻结组件**：有三样永久字节不属于四层里的任何一层，各立成一个组件，C9（冻结后偷改格式） 的 `spec_hash` 按组件记，四层不动——

- ① **打包记录单元的载荷记录格式**，按打包记录类型登记：类型 2 的 140 字节 inode 记录表（D8（核心索引结构） 已定项 6），冻结时刻是第一个写出类型 2 容器的镜像；类型 1 的（宽 40，D18（块里携带什么信息） 已定项 24）各自独立，不因类型 2 一起冻结。
- ② **统计量登记表**（标签值 + 维度 + 合并规则，D5（快照 / 空间记账机制） 已定项 3 / 4）——合并规则是盘上语义，两个读者对同一批字节用不同规则会算出不同的值，C71（不认识的统计量标签静默忽略） 只罩「不认识的标签」，罩不到「认识标签但用错规则」。
- ③ **journal 记录格式**（记录头、点名项、载荷校验和的覆盖范围、事务边界）：冻结前置不早于第 2 层与落点粒度那一格，冻结时刻是提交它那份 toml 的 commit，新记录格式走「新记录类型 + 系统配置 log-incompat 位」，全文在 D16（发布语义） 已定项 3。

**射程**：四层图不重排：D21（权威态与派生态的分界） 已定项 10 把态别改成逐结构判，第 4 层里记账树的码 2 节点是权威态、别的树的内部节点是派生态，没有一层是纯的，所以退出冻结的单位是一个结构、不是一层（每个结构落哪一层、态别是什么、退不退出冻结，登记在 `.claude/kb/freeze-layer-membership.md`，门禁 doc-decisions 的 freeze-layer-membership 格判）；三个组件的 spec 文件都不存在；组件 ① 的冻结时刻是一次运行不是一次提交，已定项 5 那道闸对它不点火。树表单元与实例表单元落哪一层还没有条款。

**依据**：无实验：分层与依赖顺序是从「哪一样事后加不进去」推出来的排序，没有可量的量；止损规则那一条的对象（D9（加密） 四项预留）在 D9（加密） 已定项 12。主 agent 与用户定案，原话在变更史。

**欠**：C119（冻结组件没有 spec 文件）；C225（组件的冻结时刻绕开门禁触发）；C246（树表单元没有冻结层归属）。

````

**出处 `.claude/kb/decisions/15-格式冻结政策.md:165-182`（整段抄，未转述）**

```markdown
#### 已定项 8：冻结之后失去哪些自由

**定案**：

| 软格式期的自由 | 冻结后变成什么 |
|---|---|
| 结构「拆了重做」 | 已冻结字段的宽度 / 存在性 / 含义不能再改；只能新增，退役字段走「保留读支持、停止写」 |
| 发现设计错了直接改正文不留兼容 | 决策记录层面仍可推翻，但**盘上字节层面从 0 兼容成本变成非零** |
| mkfs 自动选取规则可随时换算法 | 已冻结组件的选取规则本身也被钉死；换算法会让新旧镜像默认值不同，必须体现在新格式版本号下 |
| 实验结果不理想就直接改格式重定 | 只能通过新 feature bit 引入新策略，并保留旧值的解释能力 |
| checker 可以跟着格式自由改 | **对已冻结部分，checker 的判断变成事实上的兼容性契约**——悄悄改 checker 对冻结字段的解释，和悄悄改格式一样危险 |

**射程**：只说冻结之后哪些动作不再合法，不说某一次具体改动该判哪一档——那归已定项 9。第一个外部用户出现之前格式是软的，这张表在那之前一条都不生效（`.claude/rules/format-evolution.md`）。

**依据**：无实验：它是「对外的永久承诺改不掉」这条约束的逐条展开，没有可量的量。最后一行那条（checker 的判断也成契约）与 D13（验证路线） 的「checker 即规范」同源。主 agent 定案。

**欠**：无。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:183-207`（整段抄，未转述）**

```markdown
#### 已定项 9：冻结后的改动判定表

**定案**：

| 改动类型 | 旧读者会怎样 | 哪档 | bump | 理由 |
|---|---|---|---|---|
| 新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否 | 没有语义依赖的纯加法，「忽略陌生字段」这条安全网够用 |
| 新增字段，缺了会让旧读者对**另一个已有字段**算错 | **算错，可能损坏数据** | incompat | 是 | 语义一旦耦合，安全网失效 |
| 扩大已有字段宽度 | 按旧宽度解析，越界读 / 错位 | incompat | 是 | 宽度是布局的一部分，旧代码的偏移计算全错 |
| 改变已有字段含义 | 按旧含义解读，**不报错**地产生错误行为 | incompat，且**弃用旧位置不复用** | 是 | 位置复用会让新旧读者无法区分，是最危险的一类。两个实例（D18（块里携带什么信息） 已定项 11）：新增一个单元类（登记表码 4–15）改的是类身份段的解析口径；给共同前缀偏移 7 里恒 0 的 flag 位赋义（旧读者对非 0 位一律拒收） |
| 新增记录类型（新 key 类型） | 只读遍历可能误判；整理 / 写可能误删看不懂的记录 | compat_ro | 是 | **「只读安全」与「可写安全」是两条独立保证**，必须分开判。⚠️ 打包记录单元内的打包记录类型**不走这一行**（D18（块里携带什么信息） 已定项 11）：跳过一个墓碑类或 inode 类记录会改变只读答案，默认档是 incompat；只有登记时能证明「跳过它不改变任何只读答案」的类型才降到 compat_ro，且 compat_ro 读者不许跑扫描重建 |
| 新增一棵 keyspace | 若树表 day-1 就是开放列表：只是看不到新树；若树数量硬编码：读不出树表 | **compat**（已定项 2 定树表 day-1 做成开放列表）。留了开放树表 → compat；没留 → incompat | 是 | **「树表是否可扩展」必须在冻结时决定**，不能等要加树才发现没留 |
| 改变排序规则 | 二分查找 / 遍历结果错，**不报错** | incompat | 是 | 排序规则是索引本体，日志结构节点的重排直接依赖它不变 |
| 改变校验和 / MAC 算法 | D9（加密） 的算法类型字段已留 → 遇到不认识的类型只能显式失败 | 已留 → compat_ro；未留 → incompat | 是 | D9（加密） 已定项 12 第 4 项正是为这类改动准备的通道 |
| 退役一个字段 | 不受影响，旧读者继续按旧方式解释 | compat，且**永久保留解析代码** | 否 | 落过盘的字段，只要还有一份旧镜像存在就必须一直能被解释 |

**射程**：判的是「旧读者会怎样」，不判「这个改动该不该做」。第一行那条（预留区里的新增是 compat）只在真有预留区时成立，索引节点头只有 2 字节预留，装得下的字段很少。

**依据**：

- E59（缓冲消息能不能只从单元重算）：跳过一个墓碑类记录会让已删的 key 全数复活、复活数恰等于删除次数——「打包记录类型不走 compat_ro 那一行、默认档是 incompat」这半句的量。
- 主 agent 定案，逐行的理由写在表里。

**欠**：C102（索引节点头没有预留区，加字段不是 compat）；C97（改字段含义而不改宽度，全仓没有一处会红）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:208-222`（整段抄，未转述）**

```markdown
#### 已定项 10：feature bit 的位数、记账与分配权

**定案**：

- **位数**：三个 bitmap 各 **256 位**（`u64[4]`），存系统配置。位比字段宽度便宜得多（3×32 字节对系统配置可忽略），**预留要往宽了留**。
- **记账**：`.claude/kb/feature-bits.md` 列 `位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话`；位号的唯一登记位是已定项 4 那张表。
- **分配权是「谁先合并谁先占」，不是审批制**。门禁强制：位号严格递增、不许跳号占位、不许有代码引用了表里不存在的位号——机械仲裁。
- **位一旦用过不许回收。** 理由比 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 编号不回收**更硬**：编号是检索键，回收误导的是「读文档的人或模型」；**feature bit 是磁盘上真实存在过的物理位**，回收后重新分配给新特性，一个旧镜像挂到新代码上这一位会被误读成「新特性 X 已启用」——**文件系统会真的做错事**（例如把不存在的加密特征当存在，去找不存在的密钥）。退役的位标为「含义永久锁定为退役前最后一次使用的语义」，不许赋予新特性。

**射程**：管位怎么分、怎么记、能不能回收，不管每一位的语义（归申请那一位的决策）。

**依据**：无实验：位数与不回收都是从「旧镜像会被新代码读到」这条物理事实推出来的，没有可量的量；256 位的空间代价对系统配置可忽略，这一格不比谁省。主 agent 定案。

**欠**：C183（每套布局一个 incompat 位而盘上差不出字节）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:223-235`（整段抄，未转述）**

```markdown
#### 已定项 11：冻结要配的两条会红的检查

**定案**：

1. **格式冻结契约**：每个已冻结组件记 `spec_hash`；门禁重算并比对，不同则要求本次 diff **同时**满足「`feature-bits.md` 有新增行或 `version` 被显式改动」与「`decisions.md` 对应条目在同一 commit 里被改动」。缺一判红。
2. **向后兼容回归（黄金镜像）**：每次某组件版本变化时用**当时的代码**生成一份镜像并归档，门禁用**当前代码**去读全部历史镜像、跑 checker、比对期望内容摘要。镜像本体**不进 git**（`.claude/singlefs-ai-sop/rules/command-safety.md`：测试镜像不许放仓库内），仓库只提交清单 `.claude/kb/golden-images.toml`；镜像放 `${SINGLEFS_GOLDEN_DIR:-$HOME/.cache/singlefs-golden}`。清单**只增不删**——删掉某版本的回归等于悄悄放弃对它的兼容承诺，必须走决策记录。

**射程**：两条都还没实现，编号在欠账表里。第 2 条只比「今天的代码读不读得懂历史镜像」，不比「当初那份镜像写对没有」；第 1 条锚在声明上、不锚在字节上，两条都罩不到「第一份黄金镜像本身就是错代码写出来的」。

**依据**：无实验：定的是两道门禁检查该判什么，判别力自证要等它们有实现；它们要罩的失败形态（偷改格式、悄悄放弃兼容承诺）在 C9（冻结后偷改格式） 与 C227（黄金镜像的基准是它自己那份代码写的） 里各有一条。主 agent 定案。

**欠**：C9（冻结后偷改格式）；C227（黄金镜像的基准是它自己那份代码写的）。

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:1-10`（整段抄，未转述）**

```markdown
## E50 Upper and Lower Bounds on Root Ring Slot Count — Run (2026-08-30, N=5 Rounds Byte-Identical, 8 Unit Tests / 6 Mutation Rules Caught): Lower Bound S=1 Is Sufficient, Upper Bound Far Beyond 256 Slots; **One Lap Is Millisecond-Scale**

[中文](50-根环槽数的上下界.md) · **English** · [日本語](50-ルートリングスロット数の上下限.ja.md)

**This is the final piece of D22 (Synthesizing Unit Atomicity) settled item 2.** E48 (Feasible Points for Root Ring Geometry) narrowed down that item to two numbers:
**R = F + 1** (F is a product judgment that experiments cannot answer) and **S (slots per region)**. E50 (Upper and Lower Bounds on Root Ring Slot Count) measures S.

D22 (Synthesizing Unit Atomicity) wrote only one requirement for S: **"duration of one lap ≥ worst-case block reuse delay requirement"**,
and that statement was never calculated. E50 (Upper and Lower Bounds on Root Ring Slot Count) calculates both the **lower bound** and **upper bound** of S.

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:11-20`（整段抄，未转述）**

```markdown
### What to Measure

Total number of slots `N = R × S`. Every fsync issues one root (D23 (Journal Role & Format) settled item 1 axis 1 settled) ⇒ three quantities:

| Quantity | Algorithm |
|---|---|
| **One lap duration** | `N ÷ fsync rate`. fsync rate takes two tiers: **2785/sec** (E44 (Host Testing and Cost of Sequence Number Bit Width) local measurement, consumer NVMe without PLP) and **10⁵/sec** (enterprise drive with power loss protection, **hypothetical, not measured**)|
| **Blocks pinned by I-7.4 (Blocks in Recent K Generations Not Reused)** | `N × blocks written per fsync`. The latter takes two tiers: **coarse-grained** (D25 (Target Workload Priority) settled: 8 leaves + 1 shared spine) and **dispersed** (8 leaves each with own spine) as a control |
| **Generations of rollback remaining after losing one failure domain** | Reuses conclusion of E48 (Feasible Points for Root Ring Geometry) (worst-case rollback 1 generation under cross-region round-robin + prime step size); **E50 (Upper and Lower Bounds on Root Ring Slot Count) does not recalculate**, only uses it to set lower bound |

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:21-29`（整段抄，未转述）**

```markdown
### Criteria (Hardcoded Before Run)

| # | Criterion |
|---|---|
| 1 | **Lower bound**: `N ≥ 2` (D22 (Synthesizing Unit Atomicity) settled K lower limit 2), and after losing one failure domain **at least 1 usable slot remains** |
| 2 | **Upper bound**: The N where blocks pinned by I-7.4 (Blocks in Recent K Generations Not Reused) **begins to matter**. The "matters" threshold was **hardcoded before run as 1000 blocks** (≈ 16 MiB under 16 KiB units)|
| 3 | **One lap duration** given in absolute values per tier, compared against "worst-case block reuse delay requirement"—⚠️ **that requirement quantity is absent in this repo**, so E50 (Upper and Lower Bounds on Root Ring Slot Count) can only provide one lap duration, **and explicitly notes "cannot compare"** |
| 4 | Two tiers of fsync rate, two tiers of blocks written per fsync **given separately**, forbidden to combine into one sentence |

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:30-38`（整段抄，未转述）**

```markdown
### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: When `N` doubles, pinned blocks must also double (linear) | Non-linear ⇒ model not computing by N, **entire round aborted** |
| **Negative control**: Blocks written per fsync set to 0 ⇒ pinned blocks constantly 0 | Non-zero ⇒ **entire round aborted** |
| Two tiers of fsync rate yield identical one lap duration | ⇒ This dimension is dead, **entire round aborted** |
| **Pinned blocks never exceed 1000 across all scanned N** | **Faithfully record "upper bound does not exist in scanned range"**, forbidden to lower threshold to invent an upper bound |

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:39-51`（整段抄，未转述）**

```markdown
### The Half It Cannot Answer

1. **"Worst-case block reuse delay requirement" was filled by E92 (Worst-Case Block Reuse Delay Requirements) (2026-09-08)**:
   It is not a single number—among nine consumers, the requirement priced by generations is **exactly 2 generations** (root selection rollback),
   while only one is priced by wall-clock (traversal jump), shielded by a 718-microsecond window under Case A and K=2, with a 7.18× margin
   ⇒ **The one lap duration requirement is satisfied, independent of fsync rate**.
   When E50 (Upper and Lower Bounds on Root Ring Slot Count) ran on 2026-08-30, E92 (Worst-Case Block Reuse Delay Requirements) did not yet exist;
   the original phrasing in this cell at the time was: **"worst-case block reuse delay requirement" does not exist in this repo**. What it asks for is "how long after publish a root can no longer be a rollback candidate",
   and that depends on crash recovery policy and journal replay window (the defer window of D16 (Publish Semantics) settled item 1 remains unsettled).
   ⇒ **E50 (Upper and Lower Bounds on Root Ring Slot Count) gives one lap duration; cannot compare.**
2. **Blocks written per fsync** takes the coarse-grained form of D25 (Target Workload Priority), **which is a decision-specified workload model, not empirical measurement**.
3. **The 10⁵/sec tier is an assumption** (local drive lacks PLP, cannot be measured).

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:52-55`（整段抄，未转述）**

```markdown
### Prerequisites

None. Pure arithmetic.

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:56-62`（整段抄，未转述）**

```markdown
### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e50_ring_slots.rs`
  (`cargo run --release --bin e50_ring_slots`),
  raw output `e50-ring-slots-2026-08-30.out`.
- Mutation table `scripts/mutation/tables/e50_ring_slots.tsv`.

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:63-114`（整段抄，未转述）**

```markdown
### Actual Measurement (2026-08-30, N=5 Rounds Byte-Identical)

#### Quantity 1: One Lap Duration Is **Millisecond-Scale**, Not Second-Scale

| Total Slots N | One Lap (Host 2785 fsync/sec)| One Lap (Enterprise Drive 10⁵/sec, Hypothetical)|
|---|---|---|
| 2 | 0.718 ms | 20 µs |
| **8** (Candidate 2×4)| **2.87 ms** | **80 µs** |
| 64 | 22.98 ms | 0.64 ms |
| 1024 | 367.7 ms | 10.2 ms |

⇒ **This number directly rewrites the reading of D22 (Synthesizing Unit Atomicity)'s phrase "duration of one lap ≥ worst-case block reuse delay requirement"**:
If "requirement" means "can roll back to previous checkpoint" (ZFS default on 5-second scale),
host requires **≈14,000 slots**, enterprise drive requires **≈500,000 slots**—**impossible under any reasonable slot count.**
⇒ **Root ring is not used for rolling back to checkpoints**. It buys only "reading previous generation if current generation root is corrupted",
while journal replay window is independently covered by D16 (Publish Semantics) settled item 15—
**E33 (Two Pinned Block Rules) already verified empirically long ago that these two pinning rules do not pin the same batch.**

#### Quantity 2: Upper Bound Is Far Beyond Any Reasonable Slot Count

Blocks pinned by I-7.4 (Blocks in Recent K Generations Not Reused) = N × blocks written per fsync:

| N | Coarse-grained (13 blocks/fsync, D25 (Target Workload Priority) settled form)| Dispersed (34 blocks/fsync, control)|
|---|---|---|
| 8 | **104** | 272 |
| 64 | 832 | 2 176 |
| **First N reaching the 1000-block threshold** | **256** | **32** |

⇒ Under coarse-grained, **64 slots pin only 832 blocks** (≈ 13 MiB under 16 KiB units); reaching the pre-fixed 1000-block threshold requires **256 slots**.
⚠️ **Under dispersed workload, 32 slots already reaches it**—**upper bound is sensitive to D25 (Target Workload Priority) workload model**; modifying D25 (Target Workload Priority) requires recalculation.

#### Quantity 3: Lower Bound — **S = 1 Is Already Satisfied As Long As R ≥ 2**

| Geometry | Total Slots | Surviving After 1 Failure Domain Lost | Satisfies "N ≥ 2 and at least 1 remains" |
|---|---|---|---|
| 2 regions × 1 slot | 2 | **1** | **Yes** (barely enough)|
| 2 regions × 2 slots | 4 | 2 | Yes |
| 2 regions × 4 slots | 8 | 4 | Yes |
| 4 regions × 2 slots | 8 | 6 | Yes |
| **1 region × any** | —— | **0** | **No** |

(Surviving count calculated by feasible points of E48 (Feasible Points for Root Ring Geometry): cross-region round-robin + regions on distinct disks ⇒ losing one region = losing S slots.)

#### Controls

| Clause | Verdict |
|---|---|
| **Positive control**: N doubles ⇒ pinned blocks double | **Passed** (8 slots 104 → 16 slots 208)|
| **Negative control**: Write 0 blocks each time ⇒ pinned constantly 0 | **Passed** |
| Two tiers of fsync rate must yield different one lap durations | **Passed** (ratio 35.91)|
| Never exceeding 1000 blocks across all N ⇒ record "upper bound absent" | **Did not occur**: reached at coarse-grained 256 slots, dispersed 32 slots |

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:115-123`（整段抄，未转述）**

```markdown
### Conclusions

1. **Lower bound of S is 1** (as long as R ≥ 2); **upper bound is 256 slots under coarse-grained workload**, 32 slots under dispersed workload.
   ⇒ **Choosing S anywhere in 1..16 touches neither edge**, it is not a quantity requiring fine calculation.
2. **One lap duration is millisecond-scale** ⇒ **For the requirement "duration of one lap ≥ worst-case block reuse delay requirement",
   if "requirement" is understood as checkpoint scale, no reasonable slot count can satisfy it.**
   The proper reading: root ring only undertakes "rolling back recent generations"; checkpoint-scale block reuse constraint is borne by D16 (Publish Semantics) settled item 15.
3. ⚠️ **E50 (Upper and Lower Bounds on Root Ring Slot Count) cannot provide "worst-case block reuse delay requirement"** (absent in this repo); can only give one lap duration and explicitly say "cannot compare".

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:124-134`（整段抄，未转述）**

```markdown
### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e50_ring_slots.rs`
  (`cargo run --release --bin e50_ring_slots`),
  raw output `e50-ring-slots-2026-08-30.out`.
  Mutation table `scripts/mutation/tables/e50_ring_slots.tsv` (6 rules, all caught).
- **Blocks written per fsync takes the coarse-grained form settled in D25 (Target Workload Priority) (8 leaves + 1 shared spine of depth 3 + 1 root + 1 record = 13 blocks),
  which is a decision-specified workload model, not empirical measurement.** Dispersed tier (34 blocks) is a control.
- **The 10⁵ fsync/sec tier is hypothetical** (local drive lacks PLP, cannot be measured); host 2785/sec taken from E44 (Host Testing and Cost of Sequence Number Bit Width).
- **"Matters" threshold was fixed to 1000 blocks** before run; changing the line shifts the upper bound, but relative relationship between tiers remains unchanged.

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:135-145`（整段抄，未转述）**

```markdown
### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 2 | Supports | 2026-10-11 unaffected: phrasing only (time words, location refs, self-references); conclusion, numbers, artifacts unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 25 | Supports | 2026-10-11 unaffected: phrasing only (time words, location refs, self-references); conclusion, numbers, artifacts unchanged |
| D16 (Publish Semantics) settled item 6 | Supports | 2026-10-11 unaffected: phrasing only (time words, location refs, self-references); conclusion, numbers, artifacts unchanged |
| D16 (Publish Semantics) settled item 15 | Does not affect | 2026-10-11 unaffected: phrasing only (time words, location refs, self-references); conclusion, numbers, artifacts unchanged |
| D23 (Journal Role & Format) | Does not affect | 2026-10-11 unaffected: phrasing only (time words, location refs, self-references); conclusion, numbers, artifacts unchanged |
| D25 (Target Workload Priority) | Does not affect | 2026-10-11 unaffected: phrasing only (time words, location refs, self-references); conclusion, numbers, artifacts unchanged |

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:146-153`（整段抄，未转述）**

```markdown
## Historical Versions

### 2026-08-30

- Run completed. Not a single criterion modified, all 6 mutations caught.
- Initiated. Basis: E48 (Feasible Points for Root Ring Geometry) narrowed down D22 (Synthesizing Unit Atomicity) settled item 2 to R and S,
  where R is product judgment and S is computable; whereas D22 (Synthesizing Unit Atomicity) wrote only "duration of one lap ≥ worst-case block reuse delay requirement" for S, never computed.

```

**出处 `.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md:148-153`（整段抄，未转述）**

```markdown
### 2026-08-30

- Run completed. Not a single criterion modified, all 6 mutations caught.
- Initiated. Basis: E48 (Feasible Points for Root Ring Geometry) narrowed down D22 (Synthesizing Unit Atomicity) settled item 2 to R and S,
  where R is product judgment and S is computable; whereas D22 (Synthesizing Unit Atomicity) wrote only "duration of one lap ≥ worst-case block reuse delay requirement" for S, never computed.

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:1-10`（整段抄，未转述）**

```markdown
## E50 ルートリングスロット数の上下限 —— 実行済み (2026-08-30、N=5 ラウンドバイト一致、単体テスト 8 件 / 変異 6 件全捕捉)：下限は S=1 で十分、上限は 256 スロットの遥か彼方；**1 周はミリ秒オーダー**

[中文](50-根环槽数的上下界.md) · [English](50-upper-and-lower-bounds-on-root-ring-slot-count.en.md) · **日本語**

**これは D22 (ユニット原子性の合成手法) 確定項目 2 の最後のピースである。** E48 (ルートリング幾何の実行可能点) は同項目を 2 つの数値に絞り込んだ：
**R = F + 1** (F は製品判断であり実験では答えられない) と **S (領域ごとのスロット数)** である。E50 (ルートリングスロット数の上下限) は S を測定する。

D22 (ユニット原子性の合成手法) は S について 1 つの要求しか記述していなかった：**「1 周の所要時間 ≥ 最悪ブロック再利用遅延要件」**、
そしてその記述は一度も計算されたことがなかった。E50 (ルートリングスロット数の上下限) は S の**下限**と**上限**をそれぞれ計算する。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:11-20`（整段抄，未转述）**

```markdown
### 何を測定するか

総スロット数 `N = R × S`。毎回の fsync で 1 回ルートをパブリッシュする (D23 (ジャーナルの役割と形式) 確定項目 1 軸 1 決定済み) ⇒ 3 つの量：

| 測定量 | アルゴリズム |
|---|---|
| **1 周の所要時間** | `N ÷ fsync レート`。fsync レートは 2 段階を採用：**2785/秒** (E44 (シーケンス番号ビット幅の実機測定とコスト) ローカル実測、PLP なし民生用 NVMe) と **10⁵/秒** (電力喪失保護付きエンタープライズドライブ、**仮定であり実測ではない**) |
| **I-7.4 (直近K世代のブロック未再利用) が拘束するブロック数** | `N × 毎回の fsync で書き込むブロック数`。後者は 2 段階を採用：**粗粒度** (D25 (対象ワークロードの優先順位) 確定済み：8 リーフ + 1 本の共有スパイン) と対照としての**分散** (8 リーフがそれぞれ独自のスパインを持つ) |
| **1 つの障害ドメイン喪失後に何世代のロールバックが残るか** | E48 (ルートリング幾何の実行可能点) の結論を再利用 (リージョン間ラウンドロビン + 素数ステップ下で最悪 1 世代ロールバック)、**E50 (ルートリングスロット数の上下限) は再計算せず**、下限の決定にのみ使用 |

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:21-29`（整段抄，未转述）**

```markdown
### 判定基準 (実行前ハードコード)

| # | 判定基準 |
|---|---|
| 1 | **下限**：`N ≥ 2` (D22 (ユニット原子性の合成手法) 確定済み K 下限 2)、かつ 1 つの障害ドメイン喪失後に**少なくとも 1 つの利用可能なスロットが残る** |
| 2 | **上限**：I-7.4 (直近K世代のブロック未再利用) が拘束するブロック数が**問題となり始める** N。「問題となる」ラインは**実行前に 1000 ブロックとハードコード** (16 KiB ユニット下で ≈ 16 MiB) |
| 3 | **1 周の所要時間**は段階ごとに絶対値を提示し、「最悪ブロック再利用遅延要件」と比較する——⚠️ **その要求量は本リポジトリに存在しない**ため、E50 (ルートリングスロット数の上下限) は 1 周の所要時間のみを提示でき、**明示的に「比較不能」と記録する** |
| 4 | 2 段階の fsync レート、2 段階の書き込みブロック数は**個別に**提示し、1 文に統合してはならない |

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:30-38`（整段抄，未转述）**

```markdown
### どのような場合を失敗とするか (実行前ハードコード、実行後変更不可)

| 条項 | トリガー時の判定方法 |
|---|---|
| **陽性対照**：`N` が倍増した場合、拘束ブロック数も倍増しなければならない (線形) | 非線形 ⇒ モデルが N に基づいて計算していない、**ラウンド全体無効** |
| **陰性対照**：毎回の fsync 書き込みブロック数を 0 に設定 ⇒ 拘束ブロック数は恒常的に 0 | 非 0 ⇒ **ラウンド全体無効** |
| 2 段階の fsync レートが同一の 1 周所要時間を提示 | ⇒ この次元が機能していない、**ラウンド全体無効** |
| **スキャンしたすべての N において拘束ブロック数が 1000 を超えない** | **「スキャン範囲内に上限が存在しない」と忠実に記録する**、上限をでっち上げるためにラインを引き下げてはならない |

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:39-51`（整段抄，未转述）**

```markdown
### 本実験で答えられない半分

1. **「最悪ブロック再利用遅延要件」という量は E92 (最悪ブロック再利用遅延要件) によって補完された (2026-09-08)**：
   それは単一の数値ではない——9 つの消費者の中で世代数で評価される要求は**厳密に 2 世代** (ルート選択ロールバック) であり、
   壁時計で評価されるものは 1 つのみ (トラバーサルジャンプ) であり、ケース A かつ K=2 の下で 718 マイクロ秒のウィンドウに包含され、マージンは 7.18 倍である
   ⇒ **1 周の所要時間の要求は満たされており、fsync レートとは無関係である**。
   E50 (ルートリングスロット数の上下限) が実行された 2026-08-30 当時、E92 (最悪ブロック再利用遅延要件) はまだ存在していなかった。
   当時のこのセルの原文は次の通りであった：**「最悪ブロック再利用遅延要件」という量は本リポジトリに存在しない**。求められているのは「パブリッシュされたルートが、どの程度経過すればロールバック候補と見なされなくなるか」であり、
   それはクラッシュ回復戦略と journal 再生ウィンドウに依存する (D16 (パブリッシュセマンティクス) 確定項目 1 の defer ウィンドウは依然未確定)。
   ⇒ **E50 (ルートリングスロット数の上下限) は 1 周の所要時間を提示するが、比較はできない。**
2. **毎回の fsync で何ブロック書き込むか**は D25 (対象ワークロードの優先順位) の粗粒度形態を採用しており、**決定によって定められたワークロードモデルであって実測ではない**。
3. **10⁵/秒の段階は仮定である** (ローカルドライブに PLP がなく、測定不能)。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:52-55`（整段抄，未转述）**

```markdown
### 前提条件

なし。純粋な算術。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:56-62`（整段抄，未转述）**

```markdown
### 計測条件と再実行

- コード `scripts/experiment-apparatus/e50_ring_slots.rs`
  (`cargo run --release --bin e50_ring_slots`)、
  生出力 `e50-ring-slots-2026-08-30.out`。
- ミューテーション表 `scripts/mutation/tables/e50_ring_slots.tsv`。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:63-114`（整段抄，未转述）**

```markdown
### 実測 (2026-08-30、N=5 ラウンドバイト一致)

#### 測定量 1：1 周の所要時間は**ミリ秒オーダー**であり、秒オーダーではない

| 総スロット数 N | 1 周 (ローカル 2785 fsync/秒) | 1 周 (エンタープライズドライブ 10⁵/秒、仮定) |
|---|---|---|
| 2 | 0.718 ms | 20 µs |
| **8** (候補 2×4) | **2.87 ms** | **80 µs** |
| 64 | 22.98 ms | 0.64 ms |
| 1024 | 367.7 ms | 10.2 ms |

⇒ **この数値は D22 (ユニット原子性の合成手法) の「1 周の所要時間 ≥ 最悪ブロック再利用遅延要件」の読み方を直接書き換える**：
もし「要件」が「前回のチェックポイントまでロールバックできること」 (ZFS デフォルトの 5 秒オーダー) であるならば、
ローカルマシンでは **≈14,000 スロット**、エンタープライズドライブでは **≈50 万スロット**が必要となる——**いかなる合理的なスロット数でも実現不可能である。**
⇒ **ルートリングはチェックポイントへのロールバックのために使われるものではない**。それが担保するのは「現世代のルートの書き込みに失敗しても前世代を読み取れること」のみであり、
journal 再生ウィンドウ側は D16 (パブリッシュセマンティクス) 確定項目 15 により個別にカバーされている——
**E33 (2つのブロック固定規則) は、これら 2 つの固定ルールが同一のバッチを拘束するものではないことをとっくに実測済みである。**

#### 測定量 2：上限はいかなる合理的なスロット数の遥か彼方にある

I-7.4 (直近K世代のブロック未再利用) が拘束するブロック数 = N × 毎回の fsync で書き込むブロック数：

| N | 粗粒度 (13 ブロック/fsync、D25 (対象ワークロードの優先順位) 確定形態) | 分散 (34 ブロック/fsync、対照) |
|---|---|---|
| 8 | **104** | 272 |
| 64 | 832 | 2 176 |
| **1000 ブロックのラインに到達する最初の N** | **256** | **32** |

⇒ 粗粒度下では **64 スロットでも 832 ブロックしか拘束しない** (16 KiB ユニット下で約 13 MiB)；実行前に定めた 1000 ブロックのラインに達するには **256 スロット**を要する。
⚠️ **分散ワークロード下では 32 スロットで到達する**——**上限は D25 (対象ワークロードの優先順位) のワークロードモデルに敏感である**ため、D25 (対象ワークロードの優先順位) を変更した場合は再計算が必要。

#### 測定量 3：下限 —— **R ≥ 2 であれば S = 1 で既に満たされる**

| 幾何構造 | 総スロット数 | 1 障害ドメイン喪失後の生存数 | 「N ≥ 2 かつ最低 1 個残る」を満たすか |
|---|---|---|---|
| 2 領域 × 1 スロット | 2 | **1** | **はい** (ちょうど足りる) |
| 2 領域 × 2 スロット | 4 | 2 | はい |
| 2 領域 × 4 スロット | 8 | 4 | はい |
| 4 領域 × 2 スロット | 8 | 6 | はい |
| **1 領域 × 任意** | —— | **0** | **いいえ** |

(生存数は E48 (ルートリング幾何の実行可能点) の実行可能点に基づき計算：領域間ラウンドロビン + 領域が互いに異なるディスク上に配置 ⇒ 1 領域の喪失 = S スロットの喪失。)

#### 対照

| 条項 | 判定 |
|---|---|
| **陽性対照**：N 倍増 ⇒ 拘束ブロック数倍増 | **合格** (8 スロット 104 → 16 スロット 208) |
| **陰性対照**：毎回 0 ブロック書き込み ⇒ 拘束は恒常的に 0 | **合格** |
| 2 段階の fsync レートで異なる 1 周所要時間を提示 | **合格** (比率 35.91) |
| すべての N で 1000 ブロックを超えない ⇒ 「上限なし」と記録 | **発生せず**：粗粒度 256 スロット、分散 32 スロットで到達 |

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:115-123`（整段抄，未转述）**

```markdown
### 結論

1. **S の下限は 1** (R ≥ 2 である限り)；**上限は粗粒度ワークロード下で 256 スロット**、分散ワークロード下で 32 スロットである。
   ⇒ **S は 1..16 の区間のどこを取ってもいかなる境界にも触れない**ため、精密な計算を要する量ではない。
2. **1 周の所要時間はミリ秒オーダー** ⇒ **「1 周の所要時間 ≥ 最悪ブロック再利用遅延要件」という要求は、
   もし「要件」をチェックポイント尺度と解釈するならば、いかなる合理的なスロット数でも満たすことはできない。**
   正しい解釈は：ルートリングは「直近数世代のロールバック」のみを担い、チェックポイント尺度のブロック再利用制約は D16 (パブリッシュセマンティクス) 確定項目 15 が担う。
3. ⚠️ **E50 (ルートリングスロット数の上下限) は「最悪ブロック再利用遅延要件」という量を提示できず** (本リポジトリに存在しない)、1 周の所要時間を提示して明示的に「比較不能」と述べることしかできない。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:124-134`（整段抄，未转述）**

```markdown
### 計測条件と既知の限界

- コード `scripts/experiment-apparatus/e50_ring_slots.rs`
  (`cargo run --release --bin e50_ring_slots`)、
  生出力 `e50-ring-slots-2026-08-30.out`。
  ミューテーション表 `scripts/mutation/tables/e50_ring_slots.tsv` (6 件、全捕捉)。
- **毎回の fsync で何ブロック書き込むかは D25 (対象ワークロードの優先順位) 確定済みの粗粒度形態 (8 リーフ + 深さ 3 の共有スパイン 1 本 + 1 ルート + 1 レコード = 13 ブロック) を採用しており、
  決定によって定められたワークロードモデルであって実測ではない。** 分散の段階 (34 ブロック) は対照である。
- **10⁵ fsync/秒 の段階は仮定である** (ローカルドライブに PLP がなく測定不能)；ローカルの 2785/秒 は E44 (シーケンス番号ビット幅の実機測定とコスト) から取得。
- **「問題となる」ラインは 1000 ブロックと定めた** (実行前設定)；ラインを変更すれば上限も連動して変動するが、2 段階間の相対関係は不変。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:135-145`（整段抄，未转述）**

```markdown
### 影響を受ける決定

| 決定項目 | 関係 | 見直し |
|---|---|---|
| D22 (ユニット原子性の合成手法) 確定項目 2 | 支持 | 2026-10-11 影響なし：表現の改訂のみ (時間表現、位置指示、自称表現)、結論、数値、成果物は不変 |
| D22 (ユニット原子性の合成手法) 確定項目 25 | 支持 | 2026-10-11 影響なし：表現の改訂のみ (時間表現、位置指示、自称表现)、結論、数値、成果物は不変 |
| D16 (パブリッシュセマンティクス) 確定項目 6 | 支持 | 2026-10-11 影響なし：表現の改訂のみ (時間表現、位置指示、自称表現)、結論、数値、成果物は不変 |
| D16 (パブリッシュセマンティクス) 確定項目 15 | 影響なし | 2026-10-11 影響なし：表現の改訂のみ (時間表現、位置指示、自称表現)、結論、数値、成果物は不変 |
| D23 (ジャーナルの役割と形式) | 影響なし | 2026-10-11 影響なし：表現の改訂のみ (時間表現、位置指示、自称表現)、結論、数値、成果物は不変 |
| D25 (対象ワークロードの優先順位) | 影響なし | 2026-10-11 影響なし：表現の改訂のみ (時間表現、位置指示、自称表現)、結論、数値、成果物は不変 |

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:146-153`（整段抄，未转述）**

```markdown
## 履歴バージョン

### 2026-08-30

- 実行完了。判定基準は 1 つも変更せず、変異 6 件を全捕捉。
- 起草。根拠：E48 (ルートリング幾何の実行可能点) が D22 (ユニット原子性の合成手法) 確定項目 2 を R と S の 2 つの数値に絞り込み、
  R は製品判断、S は計算可能；一方 D22 (ユニット原子性の合成手法) は S について「1 周の所要時間 ≥ 最悪ブロック再利用遅延要件」と記述したのみで、一度も計算されていなかった。

```

**出处 `.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md:148-153`（整段抄，未转述）**

```markdown
### 2026-08-30

- 実行完了。判定基準は 1 つも変更せず、変異 6 件を全捕捉。
- 起草。根拠：E48 (ルートリング幾何の実行可能点) が D22 (ユニット原子性の合成手法) 確定項目 2 を R と S の 2 つの数値に絞り込み、
  R は製品判断、S は計算可能；一方 D22 (ユニット原子性の合成手法) は S について「1 周の所要時間 ≥ 最悪ブロック再利用遅延要件」と記述したのみで、一度も計算されていなかった。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:1-10`（整段抄，未转述）**

```markdown
## E50 根环槽数的上下界 —— 已跑（2026-08-30，N=5 轮字节一致，8 单测 / 6 条变异全抓）：下界 S=1 就够，上界远在 256 槽之外；**一圈是毫秒级**

**中文** · [English](50-upper-and-lower-bounds-on-root-ring-slot-count.en.md) · [日本語](50-ルートリングスロット数の上下限.ja.md)

**它是 D22（单元原子性怎么合成） 已定项 2 的最后一块。** E48（根环几何的可行点） 把该项收窄成两个数：
**R = F + 1**（F 是产品判断，实验答不了）与 **S（每区槽数）**。E50（根环槽数的上下界）测 S。

D22（单元原子性怎么合成） 对 S 只写过一句要求：**「一圈的时长 ≥ 最坏的块重用延迟需求」**，
而那句话从没被算过。E50（根环槽数的上下界）把 S 的**下界**与**上界**各算一遍。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:11-20`（整段抄，未转述）**

```markdown
### 要测什么

槽总数 `N = R × S`。每次 fsync 发一次根（D23（journal 的角色与格式） 已定项 1 轴一已定）⇒ 三个量：

| 量 | 算法 |
|---|---|
| **一圈时长** | `N ÷ fsync 率`。fsync 率取两档：**2785/秒**（E44（序号位宽的本机实测与代价） 本机实测，消费级 NVMe 无 PLP）与 **10⁵/秒**（带掉电保护的企业盘，**假设不是实测**）|
| **I-7.4（近 K 代块未被复用） 扣住的块数** | `N × 每次 fsync 写的块数`。后者取两档：**粗粒度**（D25（目标负载优先级） 已定：8 叶 + 1 条共享脊柱）与**散开**（8 叶各自一条脊柱）作对照 |
| **丢一个失败域之后还剩几代可回退** | 复用 E48（根环几何的可行点） 的结论（跨区轮转 + 素数步长下最坏回退 1 代），**E50（根环槽数的上下界）不重算**，只用它定下界 |

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:21-29`（整段抄，未转述）**

```markdown
### 判据（跑前写死）

| # | 判据 |
|---|---|
| 1 | **下界**：`N ≥ 2`（D22（单元原子性怎么合成） 已定 K 下限 2），且丢一个失败域后**至少还剩 1 个可用槽** |
| 2 | **上界**：I-7.4（近 K 代块未被复用） 扣住的块数**开始要紧**的那个 N。「要紧」的线**跑前定死为 1000 块**（16 KiB 单元下 ≈ 16 MiB）|
| 3 | **一圈时长**逐档给出绝对值，并与「最坏块重用延迟需求」比——⚠️ **那个需求量本仓没有**，所以E50（根环槽数的上下界）只能给出一圈时长，**并显式记「比不了」** |
| 4 | 两档 fsync 率、两档每次写块数**分别**给，不许合成一句 |

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:30-38`（整段抄，未转述）**

```markdown
### 怎样才算失败（跑前写死，跑完不许改）

| 条款 | 触发时怎么判 |
|---|---|
| **阳性对照**：`N` 翻倍，扣住的块数必须跟着翻倍（线性） | 不线性 ⇒ 模型没在按 N 算，**整轮作废** |
| **阴性对照**：每次 fsync 写块数取 0 ⇒ 扣住的块数恒 0 | 非 0 ⇒ **整轮作废** |
| 两档 fsync 率给出同一个一圈时长 | ⇒ 这一维是死的，**整轮作废** |
| **在扫描到的全部 N 上扣住的块数都没超过 1000** | **如实记录「上界在扫描范围内不存在」**，不许把线往下调凑一个上界 |

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:39-51`（整段抄，未转述）**

```markdown
### 它答不了的那一半

1. **「最坏块重用延迟需求」这个量由 E92（最坏块重用延迟需求） 补上了（2026-09-08）**：
   它不是一个数——九个消费者里代数计价的那一类需求是**恰 2 代**（择根回退），
   而按墙钟计价的只有一个（遍历跳），在甲与 K=2 下被 718 微秒的窗口罩住，余量 7.18 倍
   ⇒ **一圈时长这条要求满足，且与 fsync 率无关**。
   E50（根环槽数的上下界） 2026-08-30 跑的那一天 E92（最坏块重用延迟需求） 还不存在，
   当时这一格的原话是：**「最坏块重用延迟需求」这个量本仓没有**。它要的是「一个已发布的根，多久之后不再可能被当作回退候选」，
   而那取决于崩溃恢复策略与 journal 重放窗口（D16（发布语义） 已定项 1 的 defer 窗口仍未定）。
   ⇒ **E50（根环槽数的上下界）给出一圈时长，比不了。**
2. **每次 fsync 写多少块**取 D25（目标负载优先级） 的粗粒度形态，**是决策定的负载模型，不是实测**。
3. **10⁵/秒那一档是假设**（本机盘无 PLP，测不到）。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:52-55`（整段抄，未转述）**

```markdown
### 前置

无。纯算术。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:56-62`（整段抄，未转述）**

```markdown
### 口径与复跑

- 代码 `scripts/experiment-apparatus/e50_ring_slots.rs`
  （`cargo run --release --bin e50_ring_slots`），
  原始输出 `e50-ring-slots-2026-08-30.out`。
- 变异表 `scripts/mutation/tables/e50_ring_slots.tsv`。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:63-114`（整段抄，未转述）**

```markdown
### 实测（2026-08-30，N=5 轮字节一致）

#### 量一：一圈时长是**毫秒级**，不是秒级

| 槽总数 N | 一圈（本机 2785 fsync/秒）| 一圈（企业盘 10⁵/秒，假设）|
|---|---|---|
| 2 | 0.718 ms | 20 µs |
| **8**（候选 2×4）| **2.87 ms** | **80 µs** |
| 64 | 22.98 ms | 0.64 ms |
| 1024 | 367.7 ms | 10.2 ms |

⇒ **这条数直接改写了 D22（单元原子性怎么合成） 那句「一圈的时长 ≥ 最坏的块重用延迟需求」的读法**：
若「需求」是「能回退到上一个 checkpoint」（ZFS 默认 5 秒量级），
本机要 **≈14 000 槽**、企业盘要 **≈50 万槽**——**任何合理的槽数都做不到。**
⇒ **根环不是用来回退到 checkpoint 的**。它买的只是「本代根写坏了还能读到上一代」，
而 journal 重放窗口那一侧由 D16（发布语义） 已定项 15 单独覆盖——
**E33（两条钉块规则） 早就实测过这两条钉块规则扣的不是同一批。**

#### 量二：上界远在任何合理槽数之外

I-7.4（近 K 代块未被复用） 扣住的块数 = N × 每次 fsync 写的块数：

| N | 粗粒度（13 块/fsync，D25（目标负载优先级） 已定形态）| 散开（34 块/fsync，对照）|
|---|---|---|
| 8 | **104** | 272 |
| 64 | 832 | 2 176 |
| **首个够到 1000 块这条线的 N** | **256** | **32** |

⇒ 粗粒度下 **64 槽只扣 832 块**（16 KiB 单元下约 13 MiB）；要够到跑前定死的 1000 块那条线得到 **256 槽**。
⚠️ **散开负载下 32 槽就够到了**——**上界对 D25（目标负载优先级） 的负载模型敏感**，D25（目标负载优先级） 一改要重算。

#### 量三：下界 —— **S = 1 就已经满足，只要 R ≥ 2**

| 几何 | 槽总数 | 丢一个失败域后幸存 | 满足「N ≥ 2 且至少剩 1 个」 |
|---|---|---|---|
| 2 区 × 1 槽 | 2 | **1** | **是**（恰好够）|
| 2 区 × 2 槽 | 4 | 2 | 是 |
| 2 区 × 4 槽 | 8 | 4 | 是 |
| 4 区 × 2 槽 | 8 | 6 | 是 |
| **1 区 × 任意** | —— | **0** | **否** |

（幸存数按 E48（根环几何的可行点） 的可行点算：跨区轮转 + 区域落在互不相同的盘上 ⇒ 丢一个区域 = 丢 S 个槽。）

#### 对照

| 条款 | 判定 |
|---|---|
| **阳性对照**：N 翻倍 ⇒ 扣住的块数翻倍 | **通过**（8 槽 104 → 16 槽 208）|
| **阴性对照**：每次写 0 块 ⇒ 扣住恒 0 | **通过** |
| 两档 fsync 率必须给出不同的一圈时长 | **通过**（比值 35.91）|
| 全部 N 上都没超过 1000 块 ⇒ 记「上界不存在」 | **没有发生**：粗粒度 256 槽、散开 32 槽够到了 |

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:115-123`（整段抄，未转述）**

```markdown
### 结论

1. **S 的下界是 1**（只要 R ≥ 2）；**上界在粗粒度负载下是 256 槽**，散开负载下 32 槽。
   ⇒ **S 在 1..16 这个区间里怎么取都不碰任何一条边**，它不是一个要精算的量。
2. **一圈时长是毫秒级** ⇒ **「一圈的时长 ≥ 最坏的块重用延迟需求」这句要求，
   若把「需求」理解成 checkpoint 尺度，任何合理槽数都满足不了。**
   正确的读法是：根环只承担「回退最近几代」，checkpoint 尺度的块重用约束由 D16（发布语义） 已定项 15 承担。
3. ⚠️ **E50（根环槽数的上下界）给不出「最坏块重用延迟需求」这个量**（本仓没有），只能给出一圈时长并显式说「比不了」。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:124-134`（整段抄，未转述）**

```markdown
### 口径与已知局限

- 代码 `scripts/experiment-apparatus/e50_ring_slots.rs`
  （`cargo run --release --bin e50_ring_slots`），
  原始输出 `e50-ring-slots-2026-08-30.out`。
  变异表 `scripts/mutation/tables/e50_ring_slots.tsv`（6 条，全抓）。
- **每次 fsync 写多少块取 D25（目标负载优先级） 已定的粗粒度形态（8 叶 + 1 条共享脊柱深 3 + 1 根 + 1 记录 = 13 块）,
  那是决策定的负载模型，不是实测。** 散开档（34 块）是对照。
- **10⁵ fsync/秒 那一档是假设**（本机盘无 PLP，测不到）；本机 2785/秒 取自 E44（序号位宽的本机实测与代价）。
- **「要紧」的线定死为 1000 块**，跑前定的；换一条线上界跟着变，但两档之间的相对关系不变。

```

**出处 `.claude/kb/experiments/50-根环槽数的上下界.md:135-145`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 2 | 支撑 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D22（单元原子性怎么合成） 已定项 25 | 支撑 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D16（发布语义） 已定项 6 | 支撑 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D16（发布语义） 已定项 15 | 不影响 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D23（journal 的角色与格式） | 不影响 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |
| D25（目标负载优先级） | 不影响 | 2026-10-21 不受影响：只加了三语切换行；此前 2026-10-11 不受影响：只改措辞（时间词、位置指代与自称），结论、数与产物不变 |

```

**出处 `.claude/kb/decisions/16-publish-semantics.en.md:1-6`（整段抄，未转述）**

```markdown
## D16 (Publish Semantics) — Settled

[中文](16-发布语义.md) · **English** · [日本語](16-パブリッシュセマンティクス.ja.md)

D16 (Publish Semantics) governs the publish slot: when a new root is published, in what persistence ordering a publish lands on disk, which number the publication counter is, when this batch of releases returns to the allocatable set, how the first root of a new instance first covers two disks, whether an empty publish writes units, and what the journal record format counts as in the D15 (Format Freeze Policy) freeze layering. Out of scope: journal record fields, ring geometry, and replay prefixes are in D23 (Journal Role & Format); root ring parameters, root record, and system configuration field tables are in D22 (Synthesizing Unit Atomicity); snapshot accounting model and lifetime boundary tables are in D5 (Snapshot and Space Accounting Mechanism) and D6 (Snapshot Implementation Model); acceptance inequalities and mount commitment bounds are in D3 (Space Allocation) and D28 (Mount-Time Commitment Bounds); memory ordering within a publish event is in `litmus/commit-publish.litmus`, which nails down write block content -> `smp_wmb` -> write generation, making no mention of frequency.

```

**出处 `.claude/kb/decisions/16-publish-semantics.en.md:7-382`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **What size the defer window takes** | Takes the tightened form of the per-disk rollback floor, counted by state count: candidate set retains the 4 most recent rollback-accessible states (deduplicated, bounded by what root ring holds), allocatable <=> released /\ release_txg <= max(F_effective, oldest valid root in ring); F recorded in system configuration **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Checkpoint trigger predicate** | Earlier of time and dirty data volume: `trigger <=> (now - last_checkpoint_time >= T_time) \/ (dirty_bytes >= T_dirty)` **Status: Settled.** **Progress: Implemented.** |
| 3 | **Which layer journal records fall into within D15 (Format Freeze Policy) freeze layering** | Does not enter four layers, established as third independent freeze component outside four layers; new record formats use record type + system configuration log-incompat bit **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **The second replay implementation in record checker** | A publish is applied atomically or not at all; second implementation stops at syntactic layer (prefix verdict + root selection + continuation), serving as fourth judge in collusion tests **Status: Settled.** **Progress: Implemented.** |
| 5 | **Values for `T_time` and `T_dirty`, loss window measurement basis** | `T_time` = 5 s, out-of-bounds yields warning only; `T_dirty` = 2 GiB, clamped to min(field, ring_len/F); crash loses at most T_time+N_switch*(T_retry+T_redo) **Status: Settled.** **Progress: Implemented.** |
| 6 | **Publication counter** | Each publish increments checkpoint_txg by 1, fsync-triggered publish is also a publish; root slot rotation key, accounting generation, and recovery replay watermarks share identical counter **Status: Settled.** **Progress: Unimplemented.** |
| 7 | **Persistence ordering of publish** | Units / nodes -> barrier -> journal records -> barrier -> root slot (FUA) -> system configuration slot -> barrier; item-by-item verification of named units required before replay application **Status: Settled.** **Progress: Implemented.** |
| 8 | **Whether a new instance must first let its root cover two disks** | Adopts Option E: fsync does not return until root covers two disks, pushing empty publishes at most R = 3 times **Status: Settled.** **Progress: Implemented.** |
| 9 | **Whether an empty publish writes units** | Writes whenever accounting tree exists: empty publish is also a publish, rewriting accounting rows and four associated items per D5 (Snapshot and Space Accounting Mechanism) settled item 2 **Status: Settled.** **Progress: Implemented.** |
| 10 | **Publish granularity** | Batches dirty nodes (or reaches timeout) before publishing a root, rather than publishing once per transaction; publish frequency governed solely by D16 (Publish Semantics) **Status: Settled.** **Progress: Implemented.** |
| 11 | **Whether journal can be bypassed to publish roots directly** | Cannot: all structures and operations enter the identical journal, sharing checkpoint cadence; fsync merely triggers an early publish **Status: Settled.** **Progress: Implemented.** |
| 12 | **Legitimacy semantics of intermediate states** | D8 (Core Index Structure) "intermediate states must each be valid file system states" rewritten to "root + arbitrary journal prefix, valid after replay" **Status: Settled.** **Progress: Implemented.** |
| 13 | **Ownership of this journal replay** | Owned by record checker, not O2 (independent parser + checker); this is the second replay implementation, the most expensive item in D16 (Publish Semantics) **Status: Settled.** **Progress: Implemented.** |
| 14 | **Placement of snapshot publish points** | Aligned with checkpoint boundary, pinned after all data mutations in this checkpoint; dual snapshots in same window disambiguated by `(txg, snapshot_id)` **Status: Settled.** **Progress: Unimplemented.** |
| 15 | **When releases generated in checkpoint C enter the allocatable set** | Forbidden from entering before C is published; two reasons, the second formulated by journal position, stricter than checkpoint boundary **Status: Settled.** **Progress: Implemented.** |
| 16 | **Amortization window boundary between checkpoint and message buffers** | Both buy the identical property, differing only in whether window resides in memory or disk; control group in E7 (offline index harness) pinned to "with checkpoint, without message buffers" **Status: Settled.** **Progress: No code object.** |
| 17 | **What callers may assume after fsync errors** | After error, this publish data may or may not be present; forbidden from assuming old memory state can be reused for retry; caller reads active root to judge outcome **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Defer Window Takes Tightened Form of Per-Disk Rollback Floor, Counted by State Count

**Settlement**: Rollback candidate set retains the 4 most recent distinct rollback-accessible states — counting only roots that changed user-visible state, excluding empty publish roots generated by pushing empty or raising F; identical states counted only once (deduplication judge detailed in "How 'non-empty' is recognized on disk"). **Root ring capacity is the boundary of this statement**: when two states alternate across more than one full turn of the root ring, roots in the ring consist entirely of these two states, retaining only those present in the ring, as no mechanism can produce states absent from the ring. **Window unit is publishes, not seconds**: defer window protects "whether roots in the ring remain usable"; none of the surveyed consumers price by seconds. Format takes Arm G2S of E139 (tightened form of per-disk rollback floor); after reclamation threshold rolls back to oldest valid root in ring, device candidate floor carries F in system configuration (E139 (tightened form of per-disk rollback floor) 2nd rerun; RGall family and window arm in harness carry it, Arm G2S itself does not, RGall is G2S plus clause-form candidate floor, pinned by unit test K15), while post-recovery effective value still follows device old formula (E139 (tightened form of per-disk rollback floor) item-by-item conformance table "effective" row, difference 3), this phrase still not matching device word-for-word, owed under C660 (settlement phrasing of per-disk rollback floor does not match harness):

| Item | Settlement |
|---|---|
| Allocatable | Released /\ release_txg <= max(F_effective, oldest valid root in ring) |
| Oldest valid root in ring | Minimum checkpoint_txg among roots across all on-disk root slots that self-verify as legal and remain valid per instance table; failed-write slots count by old content, in-flight non-persisted publishes excluded. **Six locations sharing this identical value** (settled by user 2026-10-04, attacked for one round): allocator reclamation threshold (including raising F pass), checker candidate filtering, checker I-3.1 (allocated accounting matches) Option D defer threshold (max(F_effective, oldest valid root in ring)), shadow accounting candidate filtering, rollback candidate set, ceiling for raising F (including "non-empty" check pass); all six no longer filter roots by one-turn window, valid roots outside window (old roots lingering in slots from crashes after records but before root slots) also enter candidate set and depress threshold. **Slots unreadable during mount**: N = txg of newest valid root readable in ring (mount allocator reconstruction takes this, not first_txg - 1; in-process root recording takes that root); unreadable slots in txg > N - 3S (one turn, 3S is ring length) correspond only to one or two txgs: threshold computes by minimum, root ring table records maximum (zero rounds, measured only on attacker model), unreadable slots during mount in raising F pass likewise merged into oldest valid root per this rule; one-turn window quantity remains used only here, bounding the txg range unreadable slots could inhabit. When in-process root recording encounters this entry becoming the minimum, rereads that slot first: if readable, self-verified, and valid per instance table, computes by read root; if readable but failing self-verification, treated identically to still unreadable, reclamation threshold pinned to oldest txg it could have inhabited ("Root slot read error this time" row); if readable, self-verified, but abandoned per instance table, treated as having no root (user 2026-10-03 settled reread first; 2026-10-04 settled taking recorded max txg on self-verify failure, later same day settled pinning read-bad slots in reclamation threshold to oldest possible txg, 2026-10-05 settled on pinning oldest txg, zero rounds); slots rereading valid roots count into "slots this process knows to hold a root" ("Root slot read error this time" row). |
| Rollback floor F | Root record carries 8 bytes (D22 (Synthesizing Unit Atomicity) settled item 7, 242 -> 250), system configuration carries another 8 bytes (D22 (Synthesizing Unit Atomicity) settled item 9). Ordinarily untouched, window spans entire root ring; raised when acceptance space is insufficient, raised to active version's txg on normal unmount |
| Ceiling for raising F | **Acceptance raising F**: min(newest persisted valid root on each disk, 4th newest non-empty persisted valid root); takes oldest valid root when non-empty valid roots fewer than 4; "valid root" and "non-empty" evaluated across all roots in ring; slots unreadable or failing self-verification at mount and unwritten by this process since are treated as potentially non-empty valid roots, F not exceeding them (user settled 2026-10-04, zero rounds: initially merged only unreadable slots, redetermined after code round 3 to merge both unreadable and self-verify-failed slots; sharing logic with merging unreadable slots into reclamation threshold in "Oldest valid root in ring" row). Bad slots merged into ceiling; if blocked, reports ENOSPC in this mount without pushing empty publishes; all-zero slots not merged into ceiling, F may exceed valid roots residing in all-zero slots (cost acknowledged by user). Single disposal target = min(release_txg of this release, 4th newest non-empty root). **Unmount raising F**: raised to active version's txg, without independent ceiling evaluation (equivalent to ceiling taking newest persisted valid root, not taking minimum across disks) |
| Root slot read error this time | Evaluating acceptance raising F ceiling, judging rollback candidates while mounted (whether target exists in root ring), and reading root ring when calculating F_effective: slots unreadable or failing self-verification at mount and unwritten by this process since are treated when computing acceptance raising F ceiling as potentially non-empty valid roots, F not exceeding them ("Ceiling for raising F" row, user 2026-10-04 settled merging only unreadable slots, redetermined after code round 3 to merge both unreadable and self-verify-failed, zero rounds), and treated as having no root when judging rollback candidates and computing F_effective; slots readable at mount, written by this process with returned FUA, or rereading valid root during in-process root recording (these three classes constitute "slots this process knows to hold a root"; slots self-verifying on only one of the two allocator reconstruction passes also included, raising F pass handled per "slots self-verifying on only one pass" clause), if unreadable or failing self-verification this time, reread once; if still bad, reject this floor raise or rollback, reporting distinct error members without guessing presence or absence (raising F half settled by user 2026-09-26, derived, attacked zero rounds; rollback half settled by main agent 2026-09-28, attacked one round: code three-way round 3 verdict `research/prompts/m2-closeout-code-r3-main-verification.md` section 1 judged choosing for unwritten clauses, rollback expansion reject-wrong or accept-wrong cell missed (single sampling)). Writable mount root selection handled per C554 (shadow accounting missed when crash recovery abandoned roots temporarily unreadable) Option B configuration continuation (D23 (Journal Role & Format) settled item 14: if system configuration witnessed a publish newer than selected version, reread reading stage once; if still bad, reject writable mount); read-only mount root selection and how "oldest valid root in ring" handles this unjudged, see C563 (reading one bad slot in root ring treated as no root, other callers unjudged); **Reading the root ring to judge candidates for falling back to the witnessed version** (the fall-back of D23 (Journal Role & Format) settled item 14, "selected root read failure and unwitnessed by system configuration regresses to witnessed version": recovery, read-only mount and writable mount share one logic; the judgement asks once, then again after the trigger and rescan, and each ask reads the root ring once): a slot that is unreadable or fails self-verification is always reread once; if the reread yields a self-verified root, that root is used as read; if the reread is still bad, the slot counts as no root, and the check "do not fall back if falling back would skip a publish whose last record is readable but whose root is not" turns this into no fall-back, so all three report the selected version as unreadable and never pick another version (user ruling, exact words in the changelog; implementation awaits the next crates batch); **Bad slots and all-zero read slots**: slots read-bad at mount (unreadable, or readable non-zero but failing self-verification) and all-zero read slots have reclamation threshold pinned in three places (at mount including m-unobtainable branch, in-process, raising F pass) to oldest txg they could have inhabited, unpinned only when this process overwrites that slot or in-process recording rereads valid or abandoned roots, remaining pinned on rereading all-zero; bad slots merged into acceptance raising F ceiling, all-zero not merged ("Ceiling for raising F" row). Pins are also persisted: the root record carries the reclamation threshold t and the pinned slot p used by this publish, and the writer takes t = max(the F this root carries, the reclamation threshold when the allocator recorded this root); a pinned slot must not hold a root written by this instance (synthesized form of C773 (reclamation threshold pins for bad and all-zero root slots only in memory), user settled 2026-10-20, attacked zero rounds; changes format, implemented in the implementation batch after the system configuration landing patch merges). The two kinds of bad implementation this form cannot judge (a pinned slot rereads a valid root while the writer did not unpin; pinning a readable slot out of nothing) are not accepted costs, see C863 (two kinds of bad implementations of the pinned slot not judgeable on images). **Units newly allocated by roots un-witnessed by system configuration**: protected by pre-isolation during writable mount (D23 (Journal Role & Format) settled item 14: isolated placements named by unapplied journal records above watermark, unpinned only after this process overwrites the root ring slot corresponding to that record's txg and that publish finishes acquiring placements), not unpinned by in-process reread opportunities for all-zero slots ("Root slot read error this time" row) (user settled 2026-10-07, attacked zero rounds). **Writable mount allocator reconstruction root ring read pass**: if any disk yields zero non-abandoned self-verified valid roots (grouped by disk based on region of read slot), reread entire ring once, invoking hooks before reread; after reread, takes better of two passes per slot (self-verified root on either pass, taking reread on both self-verified > bad non-zero > all-zero), root ring table, reclamation threshold, and shadow accounting all using merged result; if merged passes yield zero non-abandoned self-verified valid roots, reject writable mount prior to taking numbers, reporting `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread` (separate from read-stage `Recovery(NoValidRoot)`). **Slots self-verified on only one of the two allocator reconstruction passes** (merged across passes as self-verified root, other pass unreadable, self-verify-failed, or all-zero; implemented in `crates/singlefs-core/src/allocator.rs` as `RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`): when computing acceptance raising F ceiling, raising F sequence (acceptance and unmount) recomputing reclamation threshold and shadow accounting, rereads once if read-bad; if still bad, does not reject, calculating by root read at mount and recorded in root ring table: valid roots enter ceiling and candidate set per self, abandoned roots enter shadow accounting per recorded placements without entering ceiling (implementation attacked zero rounds); when judging candidate and computing F_effective during mount, counts as "slot this process knows to hold a root", reread still bad rejects as usual ("non-reject" arm measured); reclassified once this process overwrites that slot, leaving no pin across raising F: root topping read-bad slot in raising F pass counts only in that raising F. **Abandoned roots in raising F sequence (acceptance and unmount)**: regardless of slot class, shadow accounting computes invariably per placements calculated at mount and recorded in root ring table, no longer rereading accounting from disk, not rejecting on unreadable, not following D23 (Journal Role & Format) settled item 14 credential exemption (C694 (normal unmount rejected when abandoned root accounting unreadable) Option B); "crash then writable mount" path rejects per B without change (user settled 2026-10-07, implemented in root ring line batch 10z, measured only on attacker model, attacked zero rounds); reading root ring in same sequence does not reject if abandoned root slot read-bad and reread still bad, entering shadow accounting per recorded placements without entering ceiling, whereas valid root slots reject if reread still bad per "slots this process knows to hold a root" (user settled 2026-10-08 "abandoned root slots do not reject"); when root ring table lacks recorded placements for that abandoned root (root judged abandoned only after writable mount writes row, recorded as valid root at mount) while Circle C (D23 (Journal Role & Format) settled item 14 "pre-isolation during writable mount") records same root in that slot (recording valid root with identical txg), calculates per placements recorded in Circle C without reading accounting, not rejecting if unreadable, reading disk as usual if Circle C also lacks it (user settled 2026-10-08 "use placements recorded in Circle C"); when that root (R) has unreadable accounting but complete records at writable mount, Circle C slot merges placements still allocated in active accounting at that moment (Option C, user settled 2026-10-08 "change to Option C"; attacked one round, hit two forms, patched to Option C merging intermediate and rollback revival), plus placements recorded in Circle C slots for publishes between selected version and R (Option C merging intermediate, user settled 2026-10-08 "Option C continues patching holes", attacked one round); when lineage piece named by R's record contains a lineage record with rollback_txg equal to R's txg, R is an administrator rollback, rollback target R_old is root read from root ring, valid per instance table, with txg equal to that record's T_old, R slot merges placements still allocated in R_old accounting but released in active accounting (rollback revival, same settlement, attacked one round); if rollback unrecognized and packed units named by R's record have unreadable or undecodable items, or rollback recognized but R_old unread from ring or R_old accounting unreadable, R slot merges all released placements in active accounting (broad protection, user settled 2026-10-08 "broad protection tri when unrecognized", attacked one round); if named packed units all readable and decodable with genuinely no lineage record, follows Option C as-is (implemented in `crates/singlefs-core/src/mount.rs` as `placements_in_the_accounts_of_newer_readable_roots`, `with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`, `placements_a_rollback_publish_revived_released_in_the_current_account`); these unpin at identical timing as other Circle C placements (when this process overwrites R slot). When acceptance floor raise stops at ceiling and Circle C isolates slots at this moment (judged across whole circle, indistinguishable between broad protection and other Circle C parts), publish reports `UserChangeRefused::NoSpaceAfterRaisingTheFloor`, its `wide_protection_at_the_ceiling` carrying R slot, R txg, empty publish count needed to overwrite R slot, blocked slots per disk, and two remedies (normal unmount then writable mount; invoke push ring entry), without automatic push ring (`crates/singlefs-core/src/mounted_session.rs` `StuckBehindTheWideProtection`); push ring entry `MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states` issues constant-F empty publishes only on explicit user invocation until R slot is overwritten, confirming rollback candidates drop below 4 states settled in D16 (Publish Semantics) settled item 1 (user settled 2026-10-08 "explicit push ring command plus error carrying remedies"; error with remedies and push ring entry attacked zero rounds); "abandoned root slots do not reject" and "use placements recorded in Circle C" measured only on three-way leg copies and models, attacked zero rounds |
| Effective | F additionally recorded in system configuration of each disk ("raising F sequence" row specifies write first, raise after); post-recovery effective value F_effective = max(maximum F carried by newest persisted valid roots across surviving disks, maximum F carried by roots on abandoned timelines, maximum readable F in system configuration); readable F in system configuration takes all self-verified slots (checksum passes and fsid matches this pool) across two slots on each surviving disk, sharing formula with number allocation (D18 (Information Carried in a Block) settled item 11) |
| Rollback candidate set | Judged still valid per active version instance table /\ txg >= F_effective (mounted rollback in D23 (Journal Role & Format) settled item 14 takes identical set) |
| Raising F sequence | Shared between acceptance and unmount. Rehearses entire empty publish sequence on allocator copy before touching disk: if placements unobtainable at any step, aborts without publishing once, restoring allocator to pre-raising state (`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`); **after rehearsal succeeds and before first publish, writes new F into system configuration of each disk and passes a barrier**, publishing first root carrying new F only thereafter; **if write-first step fails on any disk, sequence publishes zero roots, returning error; new F already written to other disks remains without rollback, F already used by reclamation does not fall back** (settled by main agent 2026-09-26, derived, attacked zero rounds); if power is lost before the new F is effective on every disk, the next mount may read back a lower value (when the write-first step landed in only one system configuration slot on one disk and that slot stays unreadable throughout the next writable mount, the effective F on disk goes 5 → 0); reclamation has not used it yet, so this is not a violation (user settled 2026-10-20 "narrow the scope, name the test case", zero-round wording; test case `crates/singlefs-harness/tests/root_ring_reads_that_fail_are_not_taken_as_no_root.rs` `a_floor_raise_cut_after_its_first_system_configuration_slot_falls_back_to_zero_without_reusing_units_when_that_slot_stays_unreadable_through_the_writable_mount`); empty publishes pushed until every disk carries a root with new F; slots reclaimed by this sequence held back before taking effect (every disk carrying new F root), released thereafter (normal unmount exception detailed in "Normal unmount" row); if write errors occur mid-flight during real publication, reports completed publish count (`MountError::RaiseFloorSequencePublishFailed`), handling of held slots in that cell lacks clauses, see C546 (held slots unreturned when raising F rejected) |
| Normal unmount | Pushes sequence of empty publishes raising F to active version's txg ("raising F sequence" follows acceptance path, ceiling in "Ceiling for raising F" row); reclaimed slots in this sequence are immediately usable only for the segment whose release_txg is no higher than the minimum of "newest valid root txg on each disk" (counting rehearsal and real publish), remaining held until effective identically to acceptance sequence (recovery root selection ignores F, selecting newest root on surviving disk upon losing one disk; units referencing it must not be overwritten by this sequence); if any disk yields zero readable valid roots, this segment is empty and entirely held until effective, unreadable or self-verify-failed slots during this pass treated as no root; subsequent writable mount after mid-unmount crash (system configuration carrying new F, new F roots not filling every disk) likewise releases only one segment: allocator reconstruction immediately releases only slots with release_txg <= max(min(F_effective, min newest valid root txg on each disk), oldest valid root in ring), releasing only to oldest valid root in ring if any disk yields zero valid roots (unreadable slots at mount merged into this minimum by min possible txg, sharing reading with reclamation threshold), remainder held until this mount warm-up lands roots on every disk (user settled 2026-10-04, zero rounds); roots written in this sequence carry unmount mark in root record flags (D22 (Synthesizing Unit Atomicity) settled item 7); pushes 2 or 3 empty publishes under initial geometry (R = 3, region assignment 0 / 1 / 0) (3 times when active txg mod 3 == 1, 2 times when mod 3 == 0 or 2); no clean close marker added, next mount follows recovery as usual (D23 (Journal Role & Format) settled item 14); when active version tree table has 0 entries (mkfs followed only by warm-up, before first file creation), writes zero bytes, leaves F unchanged, pushes no empty publishes, unmount succeeding as usual (`crates/singlefs-core/src/mount.rs` `Unmounted::NothingWrittenOnAVersionWithoutFile`) |
| Acceptance | Allocatable = min(re-allocatable + live metadata - reserve pool, `df`); when acceptance space insufficient or placements unobtainable despite clearance, pushes empty publishes raising F before re-evaluating (not pushed before row-writing publish: first publish of new instance, metadata uses switch reserve, user data redo in same publish evaluates acceptance normally, returning ENOSPC if insufficient); **when writable mount acceptance insufficient, row-writing publish proceeds using switch reserve, pushing raising F empty publishes after row writing before re-evaluating**; single acceptance pushes at most B = 4 + 2 k_tol publishes (k_tol = 2 => 8), reporting ENOSPC only after exhausting B; if exhausted at mount, mount still succeeds as writable, subsequent publishes evaluated per publish acceptance, reporting ENOSPC if insufficient, normal unmount proceeding as usual (exception to D2 (RAID Stripe Policy) settled item 13 cell) (user settled 2026-09-26, attacked zero rounds); deletion publishes continued during mount rejected by placement space likewise retry after pushing floor raises, succeeding as writable mount if still unaccommodated (D8 (Core Index Structure) settled item 16 rule 6, user settled 2026-10-03) |
| `df` | Unused + released + live metadata - reserve pool - push empty worst residual - lag volume; reserve pool = 2 * 5 + (B - 1) * c_max = 10 + 7 c_max, residual = 5 + (B - 2) * c_max = 5 + 6 c_max; lag volume = release_txg > max(4th newest non-empty valid root, oldest valid root in ring) and released blocks not freed by empty publishes |
| Ring depth lower bound | Slot count per region S >= k_tol + 2 = 4 (D22 (Synthesizing Unit Atomicity) settled item 2 S lower bound raised accordingly) |

**How "non-empty" is recognized on disk**: A valid root is non-empty <=> root pointers of user-visible trees in its tree table (inode tree, extent tree, and all snapshot trees: Code 14 snapshot inode root, Code 15 snapshot extent root, D8 (Core Index Structure) settled item 9) differ from those in the tree table of its preceding valid root (sorted by txg; valid = valid per instance table /\ txg >= current F); publishes creating snapshots count as non-empty. Comparison targets entries of these types in tree tables, not placements of tree table units themselves — every publish (including empty publishes) rewrites tree table units (`crates/singlefs-core/src/transaction.rs`), so comparing placements would treat all empty publishes as non-empty. **Deduplication when counting "Nth newest non-empty valid root" and "4 distinct states"**: Counting from newest to oldest, a non-empty root is counted only when its (inode tree root pointer, extent tree root pointer, all snapshot tree root pointers) differs from every newer root already counted — new roots issued by administrator rollback differ from their immediate predecessor root, counting as non-empty per "How 'non-empty' is recognized on disk", but share state with an earlier root; without deduplication, identical states would be counted twice (Round 2 X7: after bouncing rollbacks, only 2 distinct states remained by content). Reverse direction merely holds F until the 4th state's root departs the root ring.

**Modifies new-pool file creation bytes: Yes**: Root record adds F 8 bytes; in new-pool file creation, F is always 0 (fewer than 4 non-empty valid roots at mkfs and first publish, ceiling takes oldest valid root txg 0). System configuration adds F 8 bytes, written as 0 in new-pool file creation (D22 (Synthesizing Unit Atomicity) settled item 9).

**Associated modifications**: Root record in D22 (Synthesizing Unit Atomicity) settled item 7 adds F (242 -> 250), flags claims unmount bit, system configuration in settled item 9 adds F 8 bytes, settled item 2 S lower bound; rollback candidate set in D23 (Journal Role & Format) settled item 14 adds "txg >= F_effective"; D3 (Space Allocation) settled item 9 item 2 specifies bounds; K in I-7.4 (blocks within K generations not reused) expressed via F; I-7.9 (rollback floor F not higher than ceiling for raising F) splits into two branches by unmount mark.

**Scope**: Settles calculation of defer window and rollback floor, not ring geometry, nor c_max measurement basis (c_max is declared worst-case empty publish overhead, evaluated fresh per current structure per D28 (Mount-Time Commitment Bounds) settled item 4). Eleven known corner cases:

- **"Effective" takes SysPre**: When F lives only in root records and root slots are not mirrored (settled item 7), corruption of roots carrying new F causes F_effective to regress, earlier roots re-entering candidate set while blocks they referenced were legitimately reused — I-7.4 (blocks within K generations not reused) fails on this history (`research/prompts/m2-step45-code-r2-main-verification.md` section 4, `research/prompts/m2-step6-checker-r1-main-verification.md` section 4); root cause is F living only in root records, independent of which rule effective value takes (Round 3 K4: when roots with F all corrupted, Round 3 rules, MAX, MAX+HOLD candidates broke 12 / 140 / 12, SysPre 0). Costs: system configuration adds 8 bytes (touches format); each floor raise (acceptance and unmount) adds one system configuration write per disk plus barrier; checker and crash-point replay must read F in system configuration — checker then judged by F on roots, falsely flagging I-7.4 (blocks within K generations not reused) and four others when roots with F all corrupted (Round 3 Y7, derived); record checkers in checker and crash-point replay now read F in system configuration, C556 (checker and crash-point replay unreading F in system configuration) repaid. "Write first, raise after" proposed by Round 3 attacker, attacked zero rounds, settled knowingly by user 2026-09-26. F carried by roots on abandoned timelines counts into F_effective (user settled 2026-09-28 "count in", attacked one round, verdict `research/prompts/abandoned-floor-r1-main-verification.md` M2: when abandoned root carries higher F, omitting it causes I-7.12 (system configuration F not lower than F on same-disk roots) to go red, rollback reaching below that F while abandoned timeline already reclaimed slots when raising F; implemented in `crates/singlefs-core/src/recovery.rs` `effective_rollback_floor_of_the_roots_read`, checker computes counterpart in `crates/singlefs-checker/src/walk.rs`); after C331 (inverted root selection overriding confirmed writes) fails to reproduce upon changing to R2 (D22 (Synthesizing Unit Atomicity) settled item 8 scope), writable mount rejects only when slotted devices report read errors and no disk has both slots self-verified; disks with one system configuration slot temporarily unreadable while another disk has both slots self-verified are no longer rejected; whether disks with abandoned roots carrying higher F exist only on constructed images has not been remeasured (derived). Judged alongside F measurement basis fork in `research/prompts/alloc-basis-r3-main-verification.md`, remaining open.
- When F regresses, generation 0 root re-enters valid roots, its tree table units legitimately reused, subsequent floor raises rejected until root ring overwrites it (`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` section 1 Z1): premise is F regression; under SysPre, no longer manifests on cell where roots with F all corrupt (derived, unconstructed in Round 3). When continuous publishes in single mount exceed root ring capacity without intra-mount reclamation, checker falsely flags I-3.1 (allocated accounting matches) on legal states (measured in `crates/` copies in E153 (costs of accounting forms and holes in ring)), remaining open, assigned to milestone "overwrite, release, rollback and reuse" supplement 2 closure table row 2.
- **Costs of normal unmount**: Each unmount adds 2 or 3 empty publishes (unpushed on 0-entry tree tables, per settlement phrasing), initial geometry adding 8 units, 21 write calls across two disks, 279,040 bytes per unmount (attacker t10, t5a in Round 1, measured on copy); post-unmount F = active version txg, leaving active version as sole rollback candidate. User-acknowledged cost when choosing was "shutdown crash fallback retains only newest root" (derived from clauses); in Round 2 unmounting then remounting with newest j roots corrupted across 75 rows, recovery read correctly and writable mounts all succeeded; this phenomenon did not appear: roots below F remain in ring, their records replaying state to the end of this sequence (Round 2 verdict G4).
- **Three user-acknowledged costs**: Deleted space returns to allocatable set only after 3 subsequent publishes altering user-visible state; until then `df` does not count it free, writing per `df` hits genuine ENOSPC; `df` underreporting = reserve pool + residual = 15 + 13 c_max (80 / 145 blocks at c_max = 5 / 10) plus lag volume, while lag volume = 4 * release volume of recent four windows, **having no upper bound** (averaging ~34 blocks, max 44 / 52 when deleting one 8-block object per window; 1044 blocks when deleting 32 objects per window), deleting s bytes yields 0 bytes increase in `df`; c_max computed fresh, initial pool scale 9 blocks => reserve pool 73 blocks.
- **Minimum 4 retained states fulfilled only when root slot write failures <= k_tol in single disposal**: S = 4, c0 = 5, 3-failure cell violates 24 times, counting as literal failure, outside k_tol scope; under full enumeration, S = 4, c0 = 1 also violates 160 times (similarly exceeding tolerance).
- **Cost belongs to option itself, not defect of this family**: Retained states pin all blocks alive at that moment and dying subsequently; choosing 4 newest means colliding on every deletion — lag volume and "waiting 3 states after deletion" both originate here.
- **Cost of reclamation threshold taking oldest valid root in ring**: Old roots lingering in ring from crashes after records but before root slots depart only after being overwritten across one turn, each hole depressing reclamation threshold by an extra turn (forward leg port S = 16 averaging 893 blocks vs window threshold 583 blocks); units held back scale linearly without upper bound when one disk persistently fails root writes (attacker model measurement, holding up to 50 / 80 / 140 / 260 blocks under 15 / 30 / 60 / 120 persistent write failures); mounting after 13 consecutive crashes after records but before root slots (S = 4) allocated 166 blocks, window threshold in same segment 26 blocks (attacker copy measurement). Numbers measured across legs under differing measurement basis, not repository harness numbers; settled knowingly by user 2026-10-04.
- **Unanswered by E139 (tightened form of per-disk rollback floor)**: Counting model, pricing by publishes not wall clock; models only root-level references without journal record replay; instance switch mount reserve deducted by D28 (Mount-Time Commitment Bounds) settled item 3; capacity 4000 blocks, two disks; k_tol fixed at 2; lag volume measurement basis constructed for state-count interpretation by registry author; reserve pool scan minimum falls at scan lower edge, genuine requirement unmeasured; ring depth lower bound taken from probe. Predecessors E135 (dynamic rollback floor) and E138 (space requirements for per-disk rollback floor and pushing empty) superseded by E139 (tightened form of per-disk rollback floor), no longer used as evidence.
- **I-7.9 (rollback floor F not higher than ceiling for raising F) recomputed ceiling outside six locations**: Recomputed ceiling in pool-level checker and GPU judge takes only roots in one-turn window anchored on newest root of image (`crates/singlefs-checker/src/walk.rs` `judge_rollback_floor_raises_against_their_ceilings` doc comment explains rationale: recomputing on later images causes old roots lingering in holes to lower ceiling, falsely flagging legal images), six locations rolling back to "oldest valid root in ring" excluding this site.
- **Allocator reconstruction root ring read pass groups by slot to disk, merging two passes per slot**: Clauses follow user settlement ("Root slot read error this time" row), implemented in batch 10h.
- **Snapshot tree roots merged into "non-empty" and deduplication**: Clauses follow user settlement ("How 'non-empty' is recognized on disk"), implemented in milestone 4 batch 3丙2: core `crates/singlefs-core/src/mount.rs` `user_visible_trees_changed` and checker `crates/singlefs-checker/src/walk.rs` modified synchronously; prior to merge, `user_visible_trees_changed` compared only inode and extent trees.

**Evidence**:

- E139 (tightened form of per-disk rollback floor): Arm G2S yielded zero false candidates, zero unrecoverable states across all fault worlds (crash in activation interval without losing disk, number-skipping before floor raise, 1..3 root slot write failures during push-empty, bad slots, overhead rising to c_max); false ENOSPC was 0 across all cells per respective `df`, publish count per acceptance measured max 7 <= 8, avoiding deadlocks under pre-run reserve pool — settlement adopts this arm (pre-registered `e139-preregistration.md`, artifact `e139-tightened-floor-2026-09-12.out`).
- Four rounds of three-way verification, round 4 verdict `research/prompts/d16-item1-r4-main-verification.md`; round 3 `research/prompts/d16-item1-r3-main-verification.md`.
- User three settlements (candidate set can continually shrink, retain minimum 4, "4" counts distinct rollback-accessible states, selecting 4 most recent among three interpretations of "which 4"), verbatim in decisions history; recognizing "non-empty" and "not pushing before row-writing publish" both settled by user, verbatim in decisions history.
- User settlement 2026-09-25: sequence of raising F changed to full sequence rehearsal, overturning 2026-09-24 "maintain successive publishes" (verbatim in decisions history); implemented in 実26 (`m2-e26-implementer-report.md`).
- Effective takes SysPre, unmount raises F to active version txg (B1), deduplicating "4 distinct states" and root ring capacity boundary, candidates evaluated per active version instance table: no experiment, three-way prototypes (`research/prompts/m2-rollback-forward-r1-opus-model/`, `m2-rollback-forward-r2-opus-model/`, `m2-rollback-forward-r3-opus-model/`) measured on frozen copies without experiment numbers, prototype numbers not repository harness numbers; three-round verdicts `research/prompts/m2-rollback-forward-r1-main-verification.md`, `m2-rollback-forward-r2-main-verification.md`, `m2-rollback-forward-r3-main-verification.md` (surviving in section 3, delivered to user in section 4 of round 3). SysPre "write first, raise after" attacked zero rounds. System configuration readable F taking all self-verified slots across two slots on each disk, publishing zero roots on write-first step failure without rolling back already-written F settled by main agent 2026-09-26, derived, attacked zero rounds.
- User settlements 2026-09-25 (raise F on normal unmount, taking B1) and 2026-09-26 (C419 (how F stays non-regressing after taking effect) takes SysPre, overturning 2026-09-25 settlement "effective value takes max F across disks"; "4 distinct states" plus root ring capacity boundary), verbatim in decisions history.
- "Root slot read error this time" row: raising F half settled by user, verbatim in decisions history, constructed counterexample in `records/2026-09-24-里程碑二收尾调度.md` section 3 "実四丙 hand-back" row; rollback half settled by main agent 2026-09-28 (same schedule record line 309), details in decisions history. Implementation merged: `crates/singlefs-core/src/recovery.rs` `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` rereads once when slots known to hold a root are unreadable or fail self-verification, reporting `RootRingSlotKnownToHoldARootStillBadAfterOneReread` (with `first_reading` / `reread`) if still bad; computing ceiling pass in `crates/singlefs-core/src/mount.rs` `roots_of_the_ring_for_the_ceiling` (reporting `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread` if still bad), raising F and unmount recomputing shadow accounting and reclamation threshold reporting `MountError::NewerStateStillUnreadableAfterOneReread` (`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`), administrator rollback `rollback_candidate` candidate evaluation and F_effective computation (`recovery.rs` `effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`) reporting `RollbackError::CandidateJudgementStillUnreadableAfterOneReread` before any write. Test cases: `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write`, `a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised`, `rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`; `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` and `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write` (rollback rejected and disk unchanged when roots written by this process corrupted) (`m2-impl-r2-fixes-b-implementer-report.md` section 3 line 3; post-merge verification 1 narrowed to unreadable only, restored to full sentence by fix B, see `m2-impl-merge-verify-1-implementer-report.md` section 3 subsection 1).
- Pushing on placement rejection in "Acceptance" row, pushing after row-writing at mount: user settlement, verbatim in decisions history; safety design round 3 verdict `research/prompts/m2-safety-r3-main-verification.md` (unobtainable placements in session C283 (reporting ENOSPC without pushing publish first upon acceptance failure) ignored, small disks deadlocked upon crash-remount).
- Reclamation threshold and five other locations rolling back to oldest valid root in ring: verdict `research/prompts/m3-reclaim-window-r1-main-verification.md` section 2 rows W1, W3 and section 6 (window threshold eliminated on target form: mounting after losing one disk lands on older root on surviving disk, reading corrupted data, full counting model enumeration; changing only allocator threshold or checker reopens I-3.1 (allocated accounting matches) false reds, measured on copy; six locations rolling back together yielded zero false candidates, zero unrecoverable states across independent leg measurements, attacked one round); user settlement 2026-10-04 (Q1 RGall, Q2 modify next round), verbatim in decisions history. E139 (tightened form of per-disk rollback floor) 1st rerun: old threshold control arm G2SW_ringgate evaluated rows zero false candidates, zero unrecoverable states; G2SW with window had non-zero rows; part caused by device candidate floor omitting F in system configuration (verdict row W2), numbers not used as evidence, governed by experiment page; 2nd rerun post-device F addition completed 2026-10-05: candidate floor carries F in system configuration, RGall (reclamation threshold and five other locations rolling back to whole ring) yielded 0 false candidates, 0 unrecoverable, 0 false ENOSPC across 13 fault worlds in 1st rerun and `disk_stuck`; numbers governed by experiment page, recorded in closure owed. No experiment: verdict numbers measured on copies and models of three-way legs, without experiment numbers, not repository harness numbers. Three-way verdicts taking one-turn window (`research/prompts/m3-ring-r2-main-verification.md`, `research/prompts/m3-ring-r3-main-verification.md`) and user settlement 2026-10-02, details and verbatim text in decisions history.
- Ceiling for raising F merging unreadable and self-verify-failed slots at mount: code round 3 verdict `research/prompts/m3-root-ring-code-r3-main-verification.md` section 2 row K2 RS3 (slot reading all-zero from mount, readable but self-verify-failed, omitted from root ring table; acceptance raising F ceiling computed as 22, exceeding intact non-empty valid root 21 on disk, pool-level checker flagging I-7.9 (rollback floor F not higher than ceiling for raising F); device error case stopped at 21, 0 red; measured on copy, main agent rerun matched) and section 6 row Q3; user settlement 2026-10-04 (merging both unreadable and self-verify-failed), verbatim in decisions history. No experiment: numbers measured on attacker copy, without experiment numbers.
- Raising F pass merging unreadable slots at mount, in-process recording rereading first, raising F ceiling taking only window roots (this half obsoleted by reclamation threshold rolling back to oldest valid root in ring): code round verdict `research/prompts/m3-root-ring-code-r1-main-verification.md` section 2 rows K2-e, K2-b, K2-b2 (attacker prototype measured, main agent rerun 115 outputs byte-identical) and section 3 after Table 2 user settlement section (e2, B2).
- In-process reread readable but self-verify-failed treated identically to still unreadable (2026-10-04 RS1 initially took recorded max txg, 2026-10-05 user settled pinning to oldest possible txg: recording max under RGall reclaimed units referenced by mkfs txg 0 valid root, `records/2026-09-30-里程碑三开工第一轮.md` batch 10 core cluster hand-back entry), slots rereading valid roots counted into slots known to hold a root, raising F ceiling treating unreadable slots at mount as potentially non-empty valid roots: code round 2 verdict `research/prompts/m3-root-ring-code-r2-main-verification.md` section 2 rows 2, 6 (valid root slot read-errored at mount, reread bad again, implementation treated as no root, reclamation threshold exceeding valid root on disk, attacker model 12 parameter sets 12 red, fix A 12 sets 0 red; raising F exceeding unreadable valid root case, attacker probe F = 22 vs F = 21 when readable, derived) and section 4 rows RS1, RS3 (fix A cost is extra accounting on unwritten slot plus mount read error, safe direction, 2 sets; ceiling counting unreadable slots cost is F raising slightly less on read faults, derived); user settlement in section 6 of same verdict, verbatim in decisions history. No experiment: numbers measured on attacker model, without experiment numbers.
- Normal unmount sequence releasing only one segment: design round verdict `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md` (unmount rejected in rehearsal after continuation exhausted, Option c1 held, zero rounds); code round 3 verdict `research/prompts/m3-unlink-code-r3-main-verification.md` section 2 n1 rows (Option c1 overwriting units referenced by selected old root when single disk survives, two samplings hit consistently, confirmed by main agent rerun; recovery root selection ignores F) and fork list `m3-unlink-code-r3-forks.md`. User settlement 2026-10-03: "Option c1 enters next batch", single-disk query "Option A GA (recommended)", verbatim in decisions history. No experiment: numbers measured on copies of three-way legs, without experiment numbers.
- Disk lacking valid roots holding entire segment, unreadable slots treated as no root, subsequent writable mount after mid-unmount crash releasing only one segment: code round verdict `research/prompts/m3-batch14-code-r1-main-verification.md` section 2 (n1 "unreadable slots, holes in root ring" row is conservative reading adopted in code; "case R" row hit by attacker, confirmed by main agent rerun, `research/prompts/m3-batch14-code-r1-main-checks/`) and fork list `m3-batch14-code-r1-forks.md`. User settlement 2026-10-04 selected "Option A remount also releases only one segment (recommended)", verbatim in decisions history. No experiment: numbers measured on copies of three-way legs, without experiment numbers.
- Disk lacking valid roots releasing only to oldest valid root on remount (written as oldest valid root in one-turn window in verdict, obsoleted alongside reclamation threshold rollback), immediately released formula carrying oldest valid root term: code round verdict `research/prompts/m3-batch16-code-r1-main-verification.md` section 2 (n1 "entire root ring segment read-bad on disk during remount" row hit by attacker, confirmed by main agent rerun, `research/prompts/m3-batch16-code-r1-main-checks/`; window term reported as omitted clause by forward leg) and fork list `m3-batch16-code-r1-forks.md`. User settlement 2026-10-04 selected "Option A release only to oldest valid root in window (recommended)", verbatim in decisions history. No experiment: numbers measured on copies of three-way legs, without experiment numbers.
- Normal unmount writing zero bytes and leaving F unchanged on 0-entry tree table: user settlement 2026-10-04 (selected Option B "do not push, clauses follow implementation", verbatim in decisions history), codifying main agent 2026-09-26 settlement in implementation (`crates/singlefs-core/src/mount.rs` `Unmounted::NothingWrittenOnAVersionWithoutFile` comment, derived, attacked zero rounds) into clauses; code round verdict `research/prompts/m3-formal-code-b16-r1-main-verification.md` section 2 K1 and section 5 row 1 (literal push-empty reading measured on attacker copy: model pushed 2, differential reported PublishCount). No experiment: numbers measured on copies of three-way legs, without experiment numbers.
- Pinning oldest txg, all-zero slots, whole ring reread once before rejection during reconstruction, merging two passes per slot in "Root slot read error this time" row: user settlements 2026-10-04 (pinning read-bad slots in reclamation threshold to oldest possible txg) and 2026-10-05 (all-zero slots pinned in reclamation threshold per self-verify failure, excluded only from acceptance raising F ceiling; reconstruction pass rereads once before rejection; Q3 new rejection cause; Q4 adopt rr2; merge two passes per slot), verbatim in decisions history; verdicts `research/prompts/m3-batch10-code-r1-main-verification.md` sections 2, 5, 6, `research/prompts/m3-batch10-code-r2-main-verification.md` sections 2, 5, 6. E173 (three treatments for ceiling blocked on read-bad slots) does not support this cell. No experiment: numbers measured on copies and models of three-way legs, without experiment numbers.
- Slots self-verified on only one of the two allocator reconstruction passes in "Root slot read error this time" row: verdict `research/prompts/m3-batch10-code-r3-main-verification.md` section 2 K1 "whether merged classification is identical to read-bad at mount" row (slots self-verified on first pass and bad continuously since reread recorded as slots known to hold a root, raising F rereading still bad rejects this raise, writable mount failing after row-writing warm-up) and section 6 S1; verdict `research/prompts/m3-batch10i-code-r1-main-verification.md` section 2 K1 (when pinned per read-bad at mount, abandoned half drops out of shadow accounting, I-7.4 (blocks within K generations not reused) flags red, acceptance ceiling blocked on txg it could inhabit; "when unpinned" cell), K2 (differential treatment of same slot between rollback and raising F) and section 6 t1, t2, fork list `m3-batch10i-code-r1-forks.md`; user settlement 2026-10-06 (separate category; computed per root read at mount and recorded in root ring table; rollback rejects as usual, next round measures non-reject arm first), verbatim in decisions history. No experiment: numbers measured on copies and attacker prototypes of three-way legs, without experiment numbers.
- Slots self-verified on only one pass in "Root slot read error this time" row rejecting if reread still bad during candidate evaluation and F_effective computation: verdict `research/prompts/m3-batch10o-code-r1-main-verification.md` section 2 K1-3 cell ("reject" and "non-reject" arms measured on copy across legs, numbers in that cell) and section 5, fork list `m3-batch10o-code-r1-forks.md` row T2; user settlement 2026-10-06 (reject as usual), verbatim in decisions history. No experiment: numbers measured on copies of three-way legs, without experiment numbers.
- Bad slots merged into ceiling, all-zero excluded in "Ceiling for raising F" row: arm comparison table of E173 (three treatments for ceiling blocked on read-bad slots) (under current rule preventing F from exceeding non-empty roots in bad slots, pushing empty rejected 4 / 15,552 fewer positions than it (clause reading), while pushing empty has positions where placements unobtainable; non-merging allowed F to exceed 180 positions; all-zero non-merging under clause reading allowed F to exceed 90 / 7,794 positions, all-zero merging rejected 18 additional positions, numbers governed by experiment page); user settlement 2026-10-05 (Q1 follow current, Q2 follow current non-merging), verbatim in decisions history.
- Recomputed ceiling of I-7.9 (rollback floor F not higher than ceiling for raising F) in pool-level checker and GPU judge taking one-turn window: verdict `research/prompts/m3-batch10-code-r1-main-verification.md` section 2 K3 and section 5 (forward conflict rows 1, 2).
- Units newly allocated by roots un-witnessed by system configuration protected by writable mount pre-isolation and not unpinned by in-process rereads: verdict `research/prompts/m3-c694-r1-main-verification.md` section 2 "K1 which half each clause governs" row, `research/prompts/m3-c694-r2-main-verification.md` section 2 "K2 Circle C unpin timing" row and section 3 item 1; user settlement 2026-10-07 (C694 (normal unmount rejected when abandoned root accounting unreadable) "execute both C and B"), verbatim in decisions history. No experiment: numbers measured on copies and models of three-way legs, without experiment numbers.
- Abandoned root slots read-bad in raising F sequence not rejecting and using placements recorded in Circle C when unrecorded in table: verdict `research/prompts/m3-batch10z-b-code-r1-main-verification.md` section 2 K1-1 (H1, attacker: rejected 90 / 90 histories per implementation at verdict, non-reject arm 0 / 90, both arms I-7.4 (blocks within K generations not reused), I-4.8 (root checksum self-consistent within K generations) 0 red), K2-2 (H2, per implementation at verdict forward 19 / 19, attacker 285 / 285 rejected, Circle C arm 0 rejected, 0 red) and section 3; Circle C slot merging placements still allocated at mount: verdict `research/prompts/m3-batch10z-d-code-r1-main-verification.md` section 2 K2-2 two rows (attacker: when root shares released units with selected version, borrowing Circle C arm alone yielded k=3 53 / 150, k=5 68 / 150 histories I-7.4 (blocks within K generations not reused) red; merging that segment yielded 0 red, 0 rejected, main agent rerun on frozen copies verbatim identical) and section 3; user settlement 2026-10-08, verbatim in decisions history. No experiment: numbers measured on copies and models of three-way legs, rerun by main agent on frozen copy, without experiment numbers.
- Option C merging intermediate, rollback revival, broad protection (tri), error carrying remedies when broad protection blocks acceptance, and push ring entry in "Root slot read error this time" row: verdict `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` section 2 K2-1 two rows (Option C hit one attacker and one forward form) and section 4; `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` section 2 K1-1, K2-2 (both branches held), K2-3 (unrecognized rollback form) and section 5; `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` section 2 K1-1, K1-2 (broad protection missed) and K1-3 (broad protection scales with release history, write rejection prevents overwriting R slot, isolating until process end); `research/prompts/m3-batch10z-no-code-r1-main-verification.md` section 2 K1-1, K1-3 (fix removing double deduction missed); investigation report `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md` (two tiers of acceptance blocked by broad protection and push ring costs); user settlement 2026-10-08 ("Option C continues patching holes", "broad protection tri when unrecognized", "explicit push ring command plus error carrying remedies"), verbatim in decisions history. No experiment: numbers measured on copies and models of three-way legs, rerun by main agent, without experiment numbers.
- Abandoned roots in raising F sequence following recorded placements without disk rereading and not rejecting: verdict `research/prompts/m3-batch10y-code-r1-main-verification.md` section 2 K4-2 cell (attacker measured: after mount, abandoned root accounting read-bad, rereading from disk per B rejected unmount across 84 / 84 histories; following recorded placements arm unmounted across 84 / 84 histories, post-unmount I-7.4 (blocks within K generations not reused) 0) and section 3; user settlement 2026-10-07 (raising F follows recorded placements), verbatim in decisions history. No experiment: numbers measured on copies and models of attacker leg, without experiment numbers.
- Snapshot tree root pointers merged into "non-empty" and deduplication: user settlement 2026-10-14 (publish creating snapshot "counts as non-empty"), verbatim in decisions history; formats for Code 14 / 15 entries in verdict `research/prompts/m4-snapshot-r1-main-verification.md` section 3 S1. No experiment: settles which tree root pointers constitute "state", with no quantifiable metric.
- The pin on the "root slot reads bad this time" row is also persisted (root record carries t and p, how t is taken, a pinned slot must not hold a root written by this instance): root ring three-party verdict `research/prompts/m3-close-rootring-r2-main-verification.md` section 2 cell judging C773 (reclamation threshold pins for bad and all-zero root slots only in memory) and `research/prompts/m3-close-rootring-r3-main-verification.md` section 2 cell for the synthesized form of C773 (reclamation threshold pins for bad and all-zero root slots only in memory) (form upheld, with a zero-round fix); user settlement 2026-10-20 ("drop the condition, do it in milestone three", "accept (1)(2), not as accepted costs"), verbatim in decisions history. No experiment: neither turning green nor false red has a committed image; both legs marked as derived.
- The fall-back candidate judgement of the "root slot reads bad this time" row reads the root ring with every bad slot reread once: user ruling (popup, exact words in the changelog); the investigation into what one missing root does to each of the four callers is in the C563 (reading one bad slot in root ring treated as no root, other callers unjudged) row, and only this place has a consequence; failing test case `crates/singlefs-harness/tests/fall_back_candidate_root_slot_reading_bad_once_does_not_change_where_recovery_and_both_mounts_land.rs`.
- The "raise-F sequence" row: F already used by reclamation does not fall back, and power loss before the new F is effective on every disk may read back a lower value: investigation (`records/2026-10-18-里程碑三收尾与近期欠账.md` line 382: effective value 5 → 0 under stacked faults, 5 was not used by reclamation before the fallback, no data consequence) and the regression test case named in that row; user settlement 2026-10-20 "narrow the scope, name the test case", verbatim in decisions history. No experiment: numbers measured on the investigation tests and the regression test case, no experiment number assigned.

**Owed checks**: C314 (units referenced by abandoned roots reusable after crash recovery falls back to old root) (unrelated to defer window size, root cause in shadow accounting of D23 (Journal Role & Format) settled item 14); C329 (no check against pushing empty publish before row-writing publish); C563 (reading one bad slot in root ring treated as no root, other callers unjudged); C660 (settlement phrasing of per-disk rollback floor does not match harness); C746 (pins on all-zero slots unpinned identically when in-process reread decodes abandoned root); C863 (two kinds of bad implementations of the pinned slot not judgeable on images).

#### Settled Item 2: Checkpoint Trigger Predicate Takes Earlier of Time and Dirty Data Volume

**Settlement**: `trigger <=> (now - last_checkpoint_time >= T_time) \/ (dirty_bytes >= T_dirty)`. Both thresholds are **declared values in system configuration**, evaluated once at mount (isomorphic with D21 (Boundary Between Authoritative and Derived States) "unit geometry evaluated once at mount").

**Scope**: Settles when triggering occurs, not the threshold values themselves (settled item 5), nor how transactions are segmented post-trigger (likewise settled item 5). One known corner case: `.claude/rules/fs-design.md` hard requirement 5 branch variables are those two thresholds, constant during mount; `now` and dirty byte count are compared quantities that change, not branch variables.

**Evidence**:

- No experiment: The `\/` form is mutually bounded by two failure modes, with no quantifiable metric — relying solely on dirty volume allows trailing writes on idle systems to remain uncommitted indefinitely, causing unbounded crash loss windows; relying solely on time allows write bursts to bloat journal consumption beyond ring geometry, whereas I-8.1 (ring geometry sufficiently large) requires bounded worst-case journal consumption per transaction. That `\/` formula constitutes the entirety, leaving zero degrees of freedom beyond magic numbers, satisfying `.claude/rules/fs-design.md` hard requirement 1 "criteria explicit, computable, written down". Quantifiable parameters are the values and feasible regions of the two thresholds, addressed in settled item 5.
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 3: Journal Record Format is Third Independent Freeze Component Outside Four Layers

**Settlement**: Journal records do not enter the four-layer chain of D15 (Format Freeze Policy). Its record format — record header (including settled pending transaction number + commit mark, reverse chain, and payload checksum three increments), named items, payload checksum coverage, and transaction boundaries — forms the third independent freeze component outside the four layers; journal geometry fields residing in system configuration (record size, ring size, in-flight record bound, tail slot) are system configuration fields, frozen alongside Layer 1. New record formats adopt "new record type + system configuration log-incompat bit": set when records of this format reside in the ring, cleared when the ring is clean, refusing mount on unrecognized bits; the only fields permanently required to be readable are the header fields used to judge "do not apply this record" (magic, type, self-described length, `jsn`, `checkpoint_txg`, header checksum).

**Freeze prerequisites** (no earlier than every external definition embedded within it): Layer 2 (position entry items within named items); placement granularity cell (owed disposition in D3 (Space Allocation) settled item 7); unit registry in D18 (Information Carried in a Block) settled item 11 (unit type tags in named items); tree table unit where tree IDs reside — lacking freeze layer membership, so this prerequisite remains pending. **Freeze moment is the commit submitting `format-frozen/journal-record.toml`**, forbidden from being phrased as "the first image writing records".

**Scope**: Answers only for journal. Freeze layer membership for tree table units, instance table units, and root records remains under C246 (tree table unit lacks freeze layer membership). Three known corner cases:

- **Setting and clearing bits are runtime actions**, outside the commit timeline, requiring an independent runtime gate (persistently setting the bit before writing the first new-format record, refusing writes if absent), landing alongside the component spec.
- **Not yet ready to freeze**, blocked by five items: named items 56 bytes only settles total width, internal composition consisting only of envisioned table in [layout/01-first-txn.md](../layout/01-first-txn.md) "VI. Journal", without room for encryption fields (governed by D23 (Journal Role & Format) settled item 1 item D "item size determined by record type"); registered format constant for record header is 78, becoming 95 after landing three increments; spec file does not exist; log-incompat bit lacks bit number and entry in D15 (Format Freeze Policy) feature bit accounting table; pre-freeze checklist item 1 measures width while item 7 measures unsettled items, both blind to `jsn` measurement basis conflicts, so neither flags red under current checklist.
- **"When the ring is clean and bit can be cleared" must follow this project's replay floor** (watermark in D23 (Journal Role & Format) settled item 14), not copied from existing implementations.

**Evidence**:

- No experiment: Freeze layer membership is format policy; what can be audited is dependency ordering between layers, with no quantifiable metric; byte accounting for record header width resides in C94 (registered format constants conflict with subsequent settlements).
- Three-way verification (background `_d16-items134-background.md`, artifacts in same directory `d16-items134-*`: `d16-items134-sonnet-output.md`, `d16-items134-opus-output.md`, `d16-items134-local-output-s1.md` and `-s2.md`): Forward leg judged "independent component" as sole answer self-consistent with D15 (Format Freeze Policy) settled item 1; reverse leg won evolution cell after adding log-incompat bit, identifying two prerequisites; local leg counterexamples across two samplings misallocated fields to wrong layers, rejected after main agent audit. Verdict adopted independent component + log-incompat bit, tightening freeze prerequisites per reverse leg.
- Existing practice reference: XFS features `XFS_SB_FEAT_INCOMPAT_LOG_ALL`, set in primary superblock and cleared when log lacks protected entries (definitions in local tree `fs/xfs/libxfs/xfs_format.h` `XFS_SB_FEAT_INCOMPAT_LOG_ALL`, `fs/xfs/xfs_mount.c` `xfs_add_incompat_log_feature` and `xfs_clear_incompat_log_features`), **unverified in this project**; known difference: XFS log replays logical-physical hybrid entries, whereas records in this project carry only pointer-layer target states.
- User settlement, verbatim in decisions history.

**Owed checks**: C119 (freeze components lack spec files) (journal record format is third, short name still says "two"), C246 (tree table unit lacks freeze layer membership), C199 (instance number monotonicity conflicts with jsn break-and-stop) (must be repaid before freeze), C225 (component freeze moment bypasses gate triggers).

#### Settled Item 4: Publish Applied Atomically or Not at All, Second Replay Implementation Stops at Syntactic Layer

**Settlement**: In the binary choice of "what applying a record does on the pointer layer", adopts "a publish is applied atomically or not at all". The second replay implementation in record checker therefore stops at the syntactic layer: prefix verdict + root selection + continuation, omitting tree semantics and ancestor regeneration. Relationship with D13 (Verification Route) follows four tightened points:

| # | What is settled | Evidence |
|---|---|---|
| 1 | **Sharing**: Shares only D13 (Verification Route) settled item 5 generated constants with implementation; **default non-sharing** with checker, sharing requiring separate justification; no recovery code shared with RefFS | D13 (Verification Route) conflict 1 compromise governs between checker traversal and scan directions, not authorizing sharing between two judges (both cloud legs agreed) |
| 2 | **Input split into three parts**: Crash-state image (inspected by root selection and prefix verdict), record stream (journal records in crash-state image ring), post-recovery image from implementation (comparison target) | Recovery itself modifies disk (instance switch writes row, reissues in-flight checkpoint, D23 (Journal Role & Format) settled item 14); "post-crash image" in D13 (Verification Route) settlement constitutes two distinct images under harness sequence of recover first, check after |
| 3 | **Missing value**: "In-flight record bound" required by prefix rule 1 has no value in this repository (I-8.3 (replay prefix strictly contiguous); D23 (Journal Role & Format) settled item 1 item C requires it in system configuration) | That quantity has zero hits in repository |
| 4 | **Collusion tests**: Record checker serves as fourth judge; each judge must possess faulty implementations caught exclusively by it that harm visible state or structure. "Removing barrier before root slots" is unqualified — its caught omission harms only record checker itself; candidate pending: original candidate truncation of rollback row W deleted alongside mounted forward publish in administrator rollback (D23 (Journal Role & Format) settled item 14), Round 2 measuring truncation unreachable in Round 2 code. O3 (independent spec executor) is step 6 in [verification-build.md](../verification-build.md) landing sequence | Reverse leg 4-D |

**Scope**: Settles what second replay does and omits, **not its required engineering effort** — without code, that figure has no verifiable answer, possessing only model equivalents in research as lower-bound proxies (`e42_transaction_records.rs` 240 lines, `e77_publish_order.rs` 506 lines, `e78_replay_start.rs` 556 lines, reverse leg `wc -l`), while fourth item (computing expected state) lacks even one line of model. Remainder of pointer layer (which pointer to swap on partial application, who COWs ancestors, who allocates placements) remains under C284 (action of applying record on pointer layer undefined); D8 (Core Index Structure) settled item 13 (intermediate state folding for identical keys within single publish) relies on this branch: application lands only on publish boundaries, preventing intermediate states within single publish from being replayed.

**Evidence**:

- No experiment: What second implementation must implement independently vs share is verification route division of labor, with no quantifiable metric; engineering effort lacks verifiable answers without code, having only line count lower-bound proxies.
- User settlement narrowing question post three-way verification (asking "what second implementation must implement independently vs share" plus "its position in collusion tests"), verbatim in decisions history.

**Owed checks**: C284 (action of applying record on pointer layer undefined).

#### Settled Item 5: Values for `T_time` and `T_dirty`, Loss Window Measurement Basis

**Settlement**: `T_time` defaults to 5 s, taking upper bound of feasible interval [200 ms, 5 s] — default loss window is 5 seconds; smaller values merely diminish amortization without buying extra safety, as the 5 seconds itself bounds data loss. **`T_time` is user-configurable without restriction**: mount-time value overrides and writes back to on-disk field; exceeding [200 ms, 5 s] produces warnings without rejection (user 2026-09-20). `T_dirty` = 2 GiB (initial value, tuned during coding); independent of loss window — under `\/` rule, time branch guarantees window, while `T_dirty` governs **volume**, serving as dirty memory budget. **`T_dirty` is tunable, not format constant**: field in system configuration is format, value inside is not. **Clamped again at mount**: effective `T_dirty` = min(system configuration field, ring_length / F); ring length is mkfs parameter (default 768 MiB, constrained to ring <= device capacity / 4), so "whether ring accommodates `F *` worst-case consumption" is guaranteed by depressing dirty budget at mount rather than mkfs lower bounds; under default ring, effective value = min(2 GiB, 256 MiB) = **256 MiB**, system configuration field remaining 2 GiB untouched.

**Loss window measurement basis**: Taking number = moment transaction assigned to a checkpoint; checkpoint slicing closes membership only — post-trigger transactions are excluded, assigned transactions write respective units, publish phase waits for all units to land (any COW checkpoint waits for slowest I/O, ZFS txg sync waiting for slowest vdev, unverified in this project). Transaction slicing discipline is one transaction per unit. For number allocation, write requests slice by unit into transactions taking numbers individually; a transaction writes at most one data unit of user data; **transactions sliced from same request take numbers in ascending order of unit keys** (commit path records "inversions between number order and key order within same request", non-zero flagging red). Unit write retry budget is wall-clock T_retry (named tunable parameter, not format constant; exhaustion triggers instance switch or read-only transition per failure nature, D23 (Journal Role & Format) settled item 14 — simultaneously loss window increment, mount-time switch predicate, and time baseline for consecutive switch counting). **Thus crash loss is at most T_time + N_switch * (T_retry + T_redo), not exactly 5 seconds** — single mount permits N_switch switches, each waiting out T_retry before redoing in-flight checkpoint (T_redo), while checkpoint open during redo cannot publish, accumulating writes. Here `T_time` is effective value for current mount, not 5-second constant; with tunable `T_time`, user 2026-08 settlement "crash loses at most 5 seconds of writes" demotes to **commitment under default values**, not global guarantee. **"Time of last checkpoint" takes moment of reissue on reissuance** (taking original time leaves trigger continuously satisfied during switch while in-flight depth is only two, idling trigger); in-flight checkpoint depth = open + publishing (two); concurrent fsync return times delayed by peer transactions in same checkpoint is inherent cost of group commit. This is executable form of settled item 14 "snapshot publish points pinned after all data mutations in this checkpoint".

**Scope**: Settles values of two thresholds and maximum crash loss, not ring geometry itself, nor whether checkpoint open during redo stalls writers (unaddressed gap noted in D28 (Mount-Time Commitment Bounds) settled item 3, unwritten in D16 (Publish Semantics)). Seven known corner cases:

- **`T_time` lower bound applies only to non-fsync publishes**: Derivation presumes multi-operation amortization per window; under fsync-heavy workloads, window is driven by fsync itself. Quote this qualification when citing lower bound; upper bound 5 s (user-settled loss window) unaffected.
- **"Rounding up to 200 ms" and "budget taking 0.1" are selections, not calculations**: 0.2 gives 94 ms, 0.05 gives 410 ms (extrapolated); lower bound shifts linearly with budget, requiring recalculation if budget changes.
- **Two derived quantities from `T_dirty` calculated per settled record format**: Under D23 (Journal Role & Format) settled item 12 4 KiB fixed records (header 95 B, 56 B/item, 71 items/record), full window under same `T_dirty` is 1847 records, consuming **7388 KiB**; ring lower bound in I-8.1 (ring geometry sufficiently large) is **14,776 KiB** at F = 2, **22,164 KiB** at F = 3. Settlement values of two thresholds unchanged, altering only measurement basis of these two derived values.
- **Record accounting under one transaction per unit conflicts with this derived value**: Evaluated across six load points, record count per fsync equals data unit count (seq batch 1 is 8 records, batch 10 is 80 records); journal permanently consumes 12.5% of user data; full window under `T_dirty` is 65,536 records, 256 MiB, 4x the envisioned 64 MiB ring in byte table. Both measurement bases juxtaposed, resolution pending user decision.
- **Continuous write bandwidth unmeasured on local hardware** (E58 (end-to-end costs of checksum granularities) 2045 MiB/s was sequential checksum throughput, not write bandwidth, forbidden from conversion), so "bytes writable in 5 seconds" cannot be computed.
- **`T_dirty` and `T_time` follow distinct rules**: `T_dirty` clamps — effective value = min(field, ring_len / F); `T_time` warns — exceeding produces warnings without rejection. Two fields in same paragraph follow distinct rules, not interpreted identically.
- **`T_time` default recommended by drive type in later phases** (user 2026-09-20): Requires empirical measurements and probed inputs across drive batches, landing along D12 (Target Media) line (initial layout line pure SSD). Recorded as reopening condition for `T_time` default, deferred to later phases.

**Evidence**:

- E70 (feasible region of two checkpoint thresholds): Feasible region of two thresholds — `T_dirty` upper bound binds ring geometry without bearing load (64 MiB ring permits 16 GiB dirty data, actual constraints being dirty memory and crash loss window); `T_time` lower bound derived from amortization budget against worst workload multistream rather than primary seq, taking amortization budget `B = 0.1` blocks/op => 547 operations => 197 ms, rounded up to 200 ms; seq workload requires zero lower bound.
- E44 (hardware measurement and cost of sequence number width): Local hardware 2785 fsync/sec, rate used to invert amortization budget into `T_time` lower bound.
- E91 (ring accounting and acceptance accounting under settled format): Recomputing journal consumption and ring lower bounds under full window of `T_dirty` using settled record format revealed old 32 B/item basis undercounted by 1.80x.
- User three settlements (two thresholds, loss window measurement basis closure, clamping again at mount), verbatim in decisions history.

**Owed checks**: C310 (transaction slicing discipline conflicts with record count basis), C425 (T_time smaller than single publish duration causes trigger idling).

#### Settled Item 6: Publication Counter, Four Equal Signs Written as One

**Settlement**: Each publish increments checkpoint_txg by one — fsync-triggered publish is also a publish, with no exceptions for "minor publishes omitting numbering"; root slot rotation key (txg in "region = txg mod R" of D22 (Synthesizing Unit Atomicity) settled item 2), accounting generation (D5 (Snapshot and Space Accounting Mechanism) settled item 2), and recovery replay watermark (D23 (Journal Role & Format) settled item 14) all share this identical counter. No separate "publish generation" field — identical to checkpoint_txg. **Four equal signs**: fsync = early publish, region = txg mod R, one number per txg, accounting generation = checkpoint number; all four read identical number, any divergent implementation colliding root slots or corrupting accounting.

**Four rider clauses** (adopted same day as settlement, not optional):

1. **Root records must carry 32-bit instance generation** (identical counter to journal instance generation in D23 (Journal Role & Format) settled item 9); tie-breaking on identical checkpoint_txg favors higher instance generation. When reading the root ring, each root's slot is checked against the placement formula (D22 (Synthesizing Unit Atomicity) settled item 16), with generation-0 roots recognized separately by the slot mkfs seeds them in; one txg has only one placement slot, so ties remain only among the byte-identical generation-0 roots, and the first in root ring read order is taken (D22 (Synthesizing Unit Atomicity) settled item 24); the formula check is not yet implemented, owed in C743 (rebuilt allocator recording new class does not verify root occupies slot). Field placed in D22 (Synthesizing Unit Atomicity) settled item 7.
2. **Accounting key "generation" segment width >= 48 bits**: At local 2785 publishes/sec, 32 bits wrap in 17.8 days. Width arithmetic self-declared as assumption, requiring recalculation.
3. **`T_time` lower bound derivation (197 ms) applies only to non-fsync publishes** — derivation presumes multi-operation amortization per window; under fsync-heavy workloads, window is driven by fsync itself. Quote this qualification when citing settled item 5 lower bound; upper bound 5 s (user-settled loss window) unaffected.
4. **Cross-publish merging of accounting entries no longer viable** (generation entered key, "last write wins" cannot merge distinct generations) => rewriting entire active generation rows each publish is actual paid cost, not upper bound. Acknowledge cost faithfully.

**Scope**: Settles counter incrementation and sharing across four sites, not root ring geometry (D22 (Synthesizing Unit Atomicity) settled item 2), nor instance generation increment timing (D23 (Journal Role & Format) settled item 9). One known corner case: adversarial testing on settlement day ran across two legs — local leg report subsequently flagged red by narrowed text corruption checks (`proposed proposed` repetitive words), invalidated under `.claude/rules/three-way-inference.md` "round invalidated after gate flags red", **forbidden from citation as three-way consensus**; settlement itself not overturned, as cloud legs' independent verdicts stand.

**Evidence**:

- E48 (feasible points of root ring geometry): Rotation key named txg and increments per publish, 9 feasible points all constructed upon this interpretation.
- E50 (upper and lower bounds of root ring slot counts): Source code verbatim "one turn duration (seconds) = total slots / fsync rate. One root published per fsync" — repository root ring arithmetic true only under "each publish increments checkpoint_txg by 1"; without it, empirical measurements retroactively invalidated.
- E53 (whether root ring mounts after losing entire disk): Timeline of losing disk and rolling back + disk recovery is precisely what rider clause 1 (root records carrying instance generation) disambiguates.
- E44 (hardware measurement and cost of sequence number width): Local 2785 publishes/sec wraps 32 bits in 17.8 days — rider clause 2 ">= 48 bits" originates here.
- Raw outputs of adversarial testing (user required attack before settlement) `c85-publication-counter-forward leg report (artifact in git)`, `c85-publication-counter-reverse leg report (artifact in git)`, `c85-publication-counter-local.out`; user settlement, verbatim in decisions history.
- The tie in rider clause 1 (reading the root ring checks slots against the placement formula, generation-0 roots recognized separately, ties remain only among byte-identical generation-0 roots): root ring three-party round three verdict `research/prompts/m3-close-rootring-r3-main-verification.md` section 2 cell C743 (rebuilt allocator recording new class does not verify root occupies slot) (the instance generation row in D22 (Synthesizing Unit Atomicity) settled item 7 shares its source with rider clause 1 and changes with it; the generation-0 exception was added after round one hit the bare formula, round two attacked it without a hit, and the defense held in round three).

**Owed checks**: C88 (timeline differentiation in root ring unimplemented) (checker side of rider clause 1, reachable flaw found in reverse leg round; merged from C85 (publication counter four equal signs unpinned), clauses consolidated, check that flags red remains owed); C743 (rebuilt allocator recording new class does not verify root occupies slot): the placement formula check when reading the root ring.

#### Settled Item 7: Persistence Ordering of Publish

**Settlement**: Persistence ordering within a single publish is invariably COW units / nodes -> barrier -> journal records -> barrier -> root slot (FUA) -> system configuration slot -> barrier; fsync returns only after system configuration rotation persists. Recovery replay must verify checksum of named units individually before applying any record. **System configuration slot updates after root slot persists, once per checkpoint**, serving as final step of publish sequence and entering crash-point replay enumeration, making segment sequence of a publish unique. **Raising F sequence (acceptance and unmount) adds step prior to first publish**: writes new F into system configuration of each disk and passes barrier (settled item 1 "raising F sequence"); this system configuration write belongs to no single publish, internal publish ordering remaining unchanged. **fsync return condition adds clause**: fsync does not return until new instance roots cover two disks (mechanism and count in settled item 8); administrator rollback is forward publish during mount (D23 (Journal Role & Format) settled item 14), confirmed to administrator only after root persists, sharing return condition with fsync; implementation pending dispatch after code audit A3c (in `transaction.rs`), cost registered in separate micro-experiment.

**Scope**: Settles sequence within publish and fsync return timing, not physical barrier costs on hardware, nor root slot redundancy (D22 (Synthesizing Unit Atomicity) settled item 2). Three known corner cases:

- **Scope of "fsync returns only after system configuration rotation persists"**: After return, root of this generation is covered solely by that root slot (root slots unmirrored), witnessed by persisted system configuration. Within same instance, retreating one generation caught up via journal replay; **first root of each new instance (writable mount, recovery, switch) is single point of failure before next publish** — if slot unreadable or disk drops, recovery retreats to previous instance root, replay halting at instance boundary per strict prefix (D23 (Journal Role & Format) settled item 14 note 1), losing transactions confirmed by fsync in this mount, a single fault sufficing. Whether new instance must write roots to two disks before confirmation settled in settled item 8.
- **Barrier measurement basis**: Two FLUSH + root slot FUA + barrier after system configuration rotation = four ordering points per publish, two more than D25 (Target Workload Priority) derivation — extra two are cost of second barrier and barrier prior to publish return, acknowledged knowingly (latter settled by user 2026-09-27, triggered by test case C577 (newest root unwitnessed by system configuration unshielded by Option B) flagging red earlier before floor raise: publish path returned after root slot FUA and rotating system configuration without barrier).
- **Second barrier buys record stream integrity**, not data integrity: data integrity requires exactly one barrier (before root slot); omitting second barrier leaves 7 classes of states with "roots present but records absent", creating holes in inputs to record checker and reverse chain. User settled "required", applying both barriers.

**Evidence**:

- E77 (persistence ordering of publishes): Crash subset enumeration — omitting barrier before root slot yielded 504 violations out of 1024 crash states; removing "verify named units before applying replay" caused 63 silent graftings in b_rs arm, proving load-bearing status.
- User settlement, verbatim in decisions history; "barrier before publish return" (after system configuration rotation, increasing ordering points from three to four) settled by user 2026-09-27, origins, prompt facts, and forks in `c554-fix-forks.md` row 4, implementation merged (`crates/singlefs-core/src/transaction.rs` `persist_the_root_then_rotate_the_system_configuration`, implementer report C577 (newest root unwitnessed by system configuration unshielded by Option B) `m2-impl-c577-barrier-implementer-report.md`), cost registered in separate micro-experiment.

**Owed checks**: C26 (barrier count unmeasured on hardware) (barriers per fsync on hardware unmeasured). Two self-verifying test cases in crash-point replay harness (removing barrier before root slot must flag red, disabling replay verification must flag red) repaid alongside C76 (publish timeline lacking authoritative placement).

#### Settled Item 8: Warm-Up Takes Option E, New Instance Root First Covers Two Disks

**Settlement**: In a new instance, before roots written (FUA returned) by this instance cover two disks, no fsync is permitted to return; achieved by pushing empty publishes successively, up to R = 3 times in initial geometry. **Initial version pays exactly 2 times, fixed as format constant rather than runtime predicate**: initial device assignment of root ring's three regions hardcoded 0 / 1 / 0 (D2 (RAID Stripe Policy) settled item 7), so txg 1 lands in region 1 (disk 1) and txg 2 in region 2 (disk 0), two publishes covering two disks => count uniquely determined by assignment. Registered as `format-const` WARM_UP_EMPTY_PUBLISHES = 2 and NEW_POOL_FILE_CREATION_TXG = 3 ([layout/01-first-txn.md](../layout/01-first-txn.md) VIII), clauses in D22 (Synthesizing Unit Atomicity) settled item 16 sentence 5. **Subsequent mounts and mid-mount crash remounts**: count calculated dynamically — row-writing publish txg takes lower bound in D23 (Journal Role & Format) settled item 14 note 3 (max(all root record txgs in ring, all self-verified record checkpoint_txgs in ring) + 1), incrementing by one per empty publish, checking disk placement per root ring, pushing until roots cover every disk, up to R times; pre-numbering acceptance and post-numbering publishes share identical schedule (`crates/singlefs-core/src/mount.rs` `warm_up_publish_txgs`).

**Scope**: Settles required actions before new instance confirmation, not instance switch criteria (D23 (Journal Role & Format) settled item 14), nor unit counts written by empty publishes (settled item 9). Four known corner cases:

- **Clause half of prerequisite C314 (units referenced by abandoned roots reusable after crash recovery falls back to old root) satisfied** (user adopted shadow accounting, clauses incorporated into D23 (Journal Role & Format) settled item 14 and I-7.4 (blocks within K generations not reused)), warm-up active; **audit half remains owed**: recorded streams of multiple mounts exist (crash-point replay overwrite fixed script stream), sentence-by-sentence audit lacks dedicated assertion in cell 1, leaving C314 (units referenced by abandoned roots reusable after crash recovery falls back to old root) on owed table.
- **Modifying assignments requires recalculating format constants** (when adding disks or opening stripe tables); "up to R = 3 times" remains geometric upper bound, not initial version count. Initial version paying 2 times unverified across three-way legs, subject to change.
- **Costs**: Modifies fsync return condition in settled item 7 (extra condition before new instance confirmation); checkpoint_txg of new-pool file creation changes 1 -> 3, adding two empty records and two root writes; switch reserve in D28 (Mount-Time Commitment Bounds) settled item 3 adds "up to R = 3 empty publishes per switch", evaluated per fresh c_max in D28 (Mount-Time Commitment Bounds) settled item 4 (pool scale 9 blocks => 27 blocks); empty publishes occupy root ring slots, evicting older non-empty roots earlier.
- **Alternative arms**: Option A (no patch) lost on single fault under criterion 2; Option D (replay across instance boundaries) eliminated.
- **Wasted pushes**: When row-writing txg = 2 (mod 3), first empty publish lands on disk 0 already covered, wasting one push — device copy measured extra 147,968 bytes written, 2 extra barriers, 1 extra FUA, 1 extra root ring slot consumed per push; mount without writing lands in this tier, 24-slot root ring cycling completely in 8 mounts (unrepeated on repository harness). Direct consequence of row-writing txg taking lower bound, accepted; fix "shifting row-writing txg of normal mount to push exactly once" attacked zero rounds, unadopted.

**Evidence**:

- E150 (reuse of units referenced by abandoned roots upon rollback): Without warm-up, 1 fault lands on abandoned root; with warm-up, requires 4 faults; shadow accounting and tombstones both yielded zero violations, user adopting shadow accounting.
- E148 (commit fixed point recomputed per two record trees): Empty publish at pool scale and commit block clustering consumes 9 blocks, switch reserve half computed accordingly.
- E142 (new-pool file creation dry run): Applying checkpoint_txg 1 -> 3 to new-pool file creation bytes produced zero violations across 262,162 states.
- Two rounds of three-way verification `research/prompts/c199-r2-main-verification.md`; user settlement, verbatim in decisions history.
- Subsequent mount clause and wasted pushes: Three-way round 1 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K2 (pushing until covered generated zero deficiencies across subsequent mounts, rollbacks, and mid-mount crash remounts); user settled 2026-09-23 following implementation and accepting wasted pushes, verbatim in decisions history.

**Owed checks**: C314 (units referenced by abandoned roots reusable after crash recovery falls back to old root) (audit half).

#### Settled Item 9: Empty Publishes Also Write Units

**Settlement**: Writes whenever accounting tree exists — empty publish is also a publish, rewriting accounting rows per D5 (Snapshot and Space Accounting Mechanism) settled item 2 (each root accounting tree stores only rows for that generation, rewriting active generation rows each publish), including accounting tree nodes, allocation records, mapping entries, and tree table units. Warm-up during first writable mount has 0 tree table entries => zero units; subsequent empty publishes consume c_max blocks evaluated fresh per D28 (Mount-Time Commitment Bounds) settled item 4. **Row-writing publish on version with 0 tree table entries writes two units**: instance table and one allocation record node, allocation record tree root pointer living in root record (D22 (Synthesizing Unit Atomicity) settled item 7); superseded instance table and allocation record node marked released, reconstructing accounting from this node on reopen. Root record entry is non-zero only on row-writing version with 0 tree table entries, mkfs generation 0 and file-bearing versions writing all zeros, zero-unit publishes copying predecessor version.

**Scope**: Settles whether empty publishes write units and which items, not c_max calculation (D28 (Mount-Time Commitment Bounds) settled item 4 evaluates fresh per current structure), nor empty publish timing (raising F in settled item 1, warm-up in settled item 8). Allocation record node written by row-writing publish writes tree ID 0 in unit header (implementation choice: tree ID watermark was mkfs 65536, writing allocation record tree 65538 immediately flags I-7.8 (root record tree ID watermark not lower than pool max tree ID) red), lacking clauses; whether it enters central mapping likewise lacks clauses — that version lacks mapping tree, omitted from D19 (Block Pointer Structure & Width Budget) settled item 8 bootstrap exemptions.

**Evidence**:

- E148 (commit fixed point recomputed per two record trees): Recomputing commit fixed point per two record trees yielded pool scale 9 blocks, new-pool file creation scale 4 blocks, providing basis for "subsequent empty publishes consume fresh c_max blocks".
- E142 (new-pool file creation dry run): Two empty publishes during first writable mount warm-up written byte-by-byte, yielding zero units when tree table has 0 entries.
- User settlement 2026-09-23 (row-writing publish on version with 0 tree table entries writes instance table and allocation record node, allocation record tree root pointer living in root record), verbatim in decisions history; implementation and test cases cited in [checks-owed.md](../checks-owed.md) repaid table row C512 (where superseded instance table on 0-entry tree table version is recorded lacks clauses).
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 10: Publish Granularity Batches Writes, Not Once Per Transaction

**Settlement**: Batches dirty nodes (or reaches timeout) before publishing a root, rather than publishing once per transaction.

**Scope**: Settles frequency, not ordering within publish event — governed by `litmus/commit-publish.litmus`, which nails down write block content -> `smp_wmb` -> write generation, making no mention of frequency. Two known corner cases:

- **Net cost distribution**: Format side virtually zero (journal required regardless), verification side substantial — intermediate state semantics rewrite (settled item 12), second replay implementation (settled items 4 and 13), plus coarsened D13 (Verification Route) "Merkle root chain differential": one generation root packs mutations from K transactions, degrading localization precision from "which transaction" to "which checkpoint", with D13 (Verification Route) specifying localization cost as genuine cost of full run tier.
- **Publish semantics govern traversal counts**: Per-transaction root publish => K batches traverse K times; checkpoint => K batches in same window traverse roughly once, each batch leaving small sequential journal record => metadata write amplification for batched large deletions drops from `K * M * H` to `checkpoint_count * M * H`.

**Evidence**:

- No experiment: Settlement rests on two independent rationales, lacking quantifiable metrics at the time. (1) Naturally compatible with D5 (Snapshot and Space Accounting Mechanism) ZFS birth txg model — ZFS txg is inherently a checkpoint (`zfs_txg_timeout` default 5 seconds, or accumulating dirty volume), batching operations and updating uberblock once at txg close; adopting checkpoint eliminates birth txg numbering granularity issues, matching deadlist comparison granularity without recalculation (unverified in this project). (2) **Root is ultimate witness of lost writes**: detecting "whether this on-disk block is the expected version" requires external witness, unobtainable within block itself (legal old and new versions isomorphic at block level, fully argued in D18 (Information Carried in a Block)); witness must reside in distinct failure domain — if parent fails to write in same event, requires grandparent, recursing to root => checkpoint-published root is termination of this recursion, its persistence requiring higher reliability than individual blocks (ZFS uses 4 labels * 128 KiB uberblock rings, writing even and odd labels in two phases; project root redundancy governed by D22 (Synthesizing Unit Atomicity) settled item 2). Quantifiable costs (feasible region of two thresholds) in settled item 5.
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 11: No Operation Bypasses Journal to Publish Roots Directly

**Settlement**: All structures and operations enter the identical journal, sharing checkpoint cadence. Effect of fsync is "triggering an early publish", **not establishing an alternate log format or recovery logic**. Criterion: whether journal is mandatory path for all commits or bypass for select paths — former represents unified mechanism with two-tier persistence, conforming to design; routing only fsync (or only metadata, only small writes) through journal while others publish roots directly constitutes "dedicated log tree for fsync" prohibited by `.claude/rules/fs-design.md`.

**Scope**: Applies to operations post-mount; mkfs seed writing generation 0 root bypasses journal (one of three root slot writers in D13 (Verification Route) settled item 1, mkfs writing zero records), outside scope. One known corner case: D23 (Journal Role & Format) settled item 1 axis 1 Option A relies on this rule, which must be addressed if reopened.

**Evidence**:

- No experiment: Embodiment of `.claude/rules/fs-design.md` "single transaction layer shared by all structures" in publish slot, judging mechanism shape (mandatory path vs bypass), not quantifiable metric; unpinned design degrades into secondary transaction mechanisms for specific paths. Write volume per persistence is separate question in D23 (Journal Role & Format) settled item 1.
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 12: Intermediate State Legitimacy Semantics Rewritten to "Root + Arbitrary Journal Prefix"

**Settlement**: D8 (Core Index Structure) "intermediate states must each be valid file system states" rewritten to "**root + arbitrary journal prefix, valid after replay**".

**Scope**: Settles how legitimacy is judged, not how prefix is truncated — prefix judgment criteria detailed in five rules of D23 (Journal Role & Format) settled item 14. One known corner case: block reuse constraint is stricter than checkpoint boundary, formulated by journal position (settled item 15 and D22 (Synthesizing Unit Atomicity) settled item 4).

**Evidence**:

- E42 (records per transaction): 1400 of 1601 prefixes truncated by prefix (87.4%); arm with one record per transaction yielded 0 illegal prefixes — direct manifestation of "arbitrary prefix valid after replay"; multi-record arm without boundary fields yielded illegal prefixes exactly equal to truncated count, dropping to 0 with boundary fields. Semantics computable on model, predicting breach conditions.
- User settlement, verbatim in decisions history.

**Owed checks**: C42 (lingering records masquerading as valid prefix) (records left in ring from previous timeline replayed as valid prefix by subsequent recovery).

#### Settled Item 13: Journal Replay Owned by Record Checker, Not O2 (Independent Parser + Checker)

**Settlement**: D13 (Verification Route) settled O2 (independent parser + checker) as unary predicate evaluating single images, so journal replay belongs to "record checker". This represents the second replay implementation, where crash semantics are most prone to **shared misconceptions** — precisely what "three-oracle collusion tests" in D13 (Verification Route) guard against. **This is the most expensive item in D16 (Publish Semantics).**

**Scope**: Settles ownership; tasks and shared elements governed by settled item 4. One known corner case: choosing Option A on axis 1 of D23 (Journal Role & Format) yields "empty redo", eliminating bulk of cost, leaving prefix verdict + root selection + continuation. ⚠️ **"Empty redo" holds only for publishes whose root has persisted**: E42 (records per transaction) measured 1400 of 1601 prefixes truncated (87.4%), truncation ratio independent of boundary fields — shifting whether these 1400 are legal or illegal (1400 illegal without boundary fields, 0 with) => **non-empty recovery replay is normal condition, not edge case.**

**Evidence**:

- No experiment: Which judge implements replay is verification route division of labor, lacking quantifiable metrics; cost magnitude and independent requirements governed by settled item 4.
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 14: Snapshot Publish Points Aligned with Checkpoint Boundary, Pinned After All Data Mutations

**Settlement**: Snapshot publish points must align with checkpoint boundaries, pinned strictly "after all data mutations in this checkpoint", forbidden within checkpoint interiors. Creating two snapshots within same checkpoint is permitted: they capture identical state, deadlist between them being empty; checker disambiguates "previous snapshot" using `(txg, snapshot_id)`.

**Scope**: Settles snapshot point placement, not snapshot accounting model (D5 (Snapshot and Space Accounting Mechanism) and D6 (Snapshot Implementation Model)). One known corner case: executable form is loss window measurement basis in settled item 5 — publish phase waits for all units to land, snapshot point landing thereafter.

**Evidence**:

- No experiment: Mechanistic rationale lacking quantifiable metric — resolution of `birth` reaches only checkpoint level; **blocks cannot express whether birth preceded or followed snapshot point within same checkpoint**; unpinned points cause `birth == S.txg` and `death == S.txg` in D5 (Snapshot and Space Accounting Mechanism) boundary table to lose deterministic answers, one causing **data loss** and the other **space leaks**.
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 15: Releases in Checkpoint C Forbidden from Entering Allocatable Set Before C Publishes

**Settlement**: All releases generated in checkpoint C are forbidden from entering the allocatable set before C is published. Two rationales, the second unique to D16 (Publish Semantics) and stricter:

1. **Rollback**: Crash may return to C-1 root, where blocks born before C and dying in C remain alive.
2. **Settled item 12 rewrites D8 (Core Index Structure) intermediate semantics to "root + arbitrary journal prefix, valid after replay"**: If block released at journal position p is reallocated and written with new content at position q > p, replaying prefix `p < r < q` yields tree pointing to that address **while on-disk bytes already reflect q content** (physical writes unconstrained by journal position). **This is an inevitable defect, not theoretical risk.**

**Scope**: Settles lower bound for returning to allocatable set, not window size (settled item 1). Three known corner cases:

- **Second rationale formulated by journal position** => true constraint is entire journal replay window, not checkpoint boundary; D22 (Synthesizing Unit Atomicity) settled item 4 closes in that form, failure mode without it being "detectable (parent pointer checksum mismatch), unrecoverable".
- **Window capacity correlates directly with ENOSPC** (D3 (Space Allocation): releasing space cannot allocate space), while batching increases probability of create-and-delete closing within same window => **defer window resides on hot path**.
- **External reference**: ZFS features isomorphic conservative mechanism, `include/sys/txg.h` defining `#define TXG_DEFER_SIZE 2 /* Number of txgs worth of frees we defer adding to in-core spacemaps */`, **unverified in this project**; ⚠️ why that value is 2 lacks primary explanation; project defer window must derive from own replay semantics, **copying forbidden**.

**Evidence**:

- E82 (on-the-fly synthesis of acceptance): Arm treating intra-window releases as available consumed defer-held blocks across 20 cells — phenomenon of silently bypassing this rule is quantifiable, acceptance form incorporated into D28 (Mount-Time Commitment Bounds) settled item 2.
- User settlement, verbatim in decisions history.

**Owed checks**: None (block reuse check handled by I-7.4 (blocks within K generations not reused), owed checks listed in settled item 1).

#### Settled Item 16: Checkpoint and Message Buffers Buy Identical Property, E7 (Offline Index Harness) Control Group Pinned

**Settlement**: Checkpoint and D11 (Index Nodes Message Buffers) message buffers purchase the identical property, differing only in **where amortization window resides**: checkpoint window resides in **memory** outside format; message buffer window resides on **disk** within format. => **D11 (Index Nodes Message Buffers) marginal benefit = cross-checkpoint amortization minus intra-checkpoint amortization**. **Control group in E7 (offline index harness) must be "with checkpoint, without message buffers".**

**Scope**: Settles control group configuration, not D11 (Index Nodes Message Buffers) choice — governed by D11 (Index Nodes Message Buffers) itself. One known corner case: BetrFS figures never decoupled the two (featuring 4 MiB message buffers **and** 5-second checkpoints), **unverified in this project**.

**Evidence**:

- No experiment: Rule governing control group setup, not experiment conclusion; literature lacks control groups with write buffer frontends, providing zero figures for marginal benefits of message buffers over project direction, project currently lacking citable quantities.
- User settlement, verbatim in decisions history.

**Owed checks**: None.

#### Settled Item 17: What Callers May Assume After fsync Errors

**Settlement**:

- After fsync errors, data from this publish **may or may not appear** — callers forbidden from interpreting "error" as "publish failed".
- **Forbidden from assuming identical memory state can be reused for retry.** (`close(2)` NOTES section states "Retrying the close() after a failure return is the wrong thing to do"; project adopts identical semantics.)
- To determine outcome, caller reads active root directly: `choose_system_configuration` and `choose_root` in `crates/singlefs-core/src/recovery.rs` evaluate against open device handle; criterion is "whether this checkpoint_txg is active root", without remounting. `transaction.rs` lacks invocation path, missing glue logic.
- Does not settle fsync return timing, which belongs to settled item 7.

**Scope**:

- **Applies only to "transition to read-only" and "redo fails again" cells.** Cells in D23 (Journal Role & Format) settled item 14 failure table judging "switch" deliver zero errors to caller (instance switch reissues in-flight checkpoint), this rule remaining silent for them.
- Governs caller assumptions after error, not failure routing (D23 (Journal Role & Format) settled item 14) or persistence ordering (settled item 7).

**Evidence**:

- No experiment: Commitment on external semantics, lacking quantifiable metric. Supported by `research/prompts/c381-r3-main-verification.md` section 1 K1 — on H1 (step 6 system configuration rotation fails, prior five steps succeed), "error equals failure" semantics **served as root cause of C381 (allocator rolls back after publish failure despite root landing on disk)**: led callers to assume failure and continue publishing from stale state.
- `research/prompts/c381-r3-main-verification.md` section 3 (Option A held, origins of two restrictions: POSIX clue in K2, scope bound in K4-e).
- User settlement 2026-09-21, verbatim in decisions history.
- ⚠️ **Explicit statement in evidence**: Latter two sentences of settlement (prohibitive clause, scope restriction) **attacked zero rounds** (`.claude/rules/three-way-inference.md`: attacker leg self-proposed restrictions count as unattacked), accepted knowingly by user 2026-09-21.

**Owed checks**: C381 (allocator rolls back after publish failure despite root landing on disk) (remains open pending fix landing).

```

**出处 `.claude/kb/decisions/16-publish-semantics.en.md:383-386`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D16 (Publish Semantics) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:1-6`（整段抄，未转述）**

```markdown
## D16 (パブリッシュセマンティクス) — 確定

[中文](16-发布语义.md) · [English](16-publish-semantics.en.md) · **日本語**

D16 (パブリッシュセマンティクス) はパブリッシュのスロットを所管する：新規ルートをいつ発行するか、1 回のパブリッシュがディスク上にどのような順序で永続化されるか、パブリッシュカウンタはいずれの数値であるか、今回の一連の解放がいつ割り当て可能集合へと復帰するか、新インスタンスの最初のルートがどのようにしてまず 2 台のディスクを網羅するか、空パブリッシュがユニットを書き出すか否か、journal 記録形式が D15 (フォーマット凍結ポリシー) の凍結階層において何に位置づけられるか。対象外とする事項：journal のレコードフィールド、リング幾何構造、および再生プレフィックスは D23 (ジャーナルの役割と形式) にある；ルートリングのパラメータ、ルートレコード、およびシステム構成のフィールド表は D22 (ユニット原子性の合成方法) にある；スナップショットの会計モデルおよび生死境界表は D5 (スナップショット・領域会計メカニズム) および D6 (スナップショット実現モデル) にある；受入不等式およびマウント時コミットメント量は D3 (領域割り当て) および D28 (マウント期コミットメント量) にある；1 回のパブリッシュイベント内部のメモリ順序は `litmus/commit-publish.litmus` にあり、これはブロック内容の書き込み → `smp_wmb` → generation の書き込みを規定するもので、頻度については一切言及しない。

```

**出处 `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:7-382`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **defer ウィンドウの大きさ** | ディスク単位のロールバック下限の厳格化形態を採用し、状態数で計測：候補集合は直近 4 個の復帰可能状態を保持（重複排除、ルートリングの収容限界を境界とする）、再割り当て可能 ⟺ 解放済み ∧ 解放代 ≤ max(F_有効, リング内最古有効ルート)；F をシステム構成に記録 **状態：確定。** **進捗：未実装。** |
| 2 | **checkpoint のトリガー判定条件** | 時間およびダーティデータ量のいずれか早い方：`トリガー ⟺ (now − 前回 checkpoint 時刻 ≥ T_time) ∨ (ダーティバイト数 ≥ T_dirty)` **状態：確定。** **進捗：実装済み。** |
| 3 | **journal は D15 (フォーマット凍結ポリシー) の凍結階層の第何層か** | 4 層には含めず、4 層外の第 3 の独立凍結コンポーネントとして立脚；新規レコード形式はレコード種別 + システム構成 log-incompat ビットを採用 **状態：確定。** **進捗：未実装。** |
| 4 | **レコード照合器における第 2 の replay 実装** | 1 回のパブリッシュを一括適用または一括非適用とする；第 2 の実装は統語論層（プレフィックス判定 + ルート選択 + 継続実行）にとどめ、共謀テストにおける第 4 の判定器とする **状態：確定。** **進捗：実装済み。** |
| 5 | **`T_time` と `T_dirty` の設定値、消失ウィンドウの計測条件** | `T_time` = 5 s、逸脱時は警告のみ；`T_dirty` = 2 GiB、min(フィールド値, リング長/F) にクランプ；クラッシュ時の最大消失量は T_time+N_switch×(T_retry+T_redo) **状態：確定。** **進捗：実装済み。** |
| 6 | **パブリッシュカウンタ** | パブリッシュごとに checkpoint_txg を 1 加算、fsync 起因のパブリッシュもパブリッシュである；ルートスロットのローテーションキー、会計の世代、リカバリ再生水位は同一のカウンタ **状態：確定。** **進捗：未実装。** |
| 7 | **パブリッシュの永続順序** | ユニット / ノード → バリア → journal レコード → バリア → ルートスロット (FUA) → システム構成スロット → バリア；再生適用前に指定ユニットの個別検証を必須とする **状態：確定。** **進捗：実装済み。** |
| 8 | **新インスタンスは自前のルートを 2 台のディスクに先行配置すべきか** | 戊を採用：ルートが 2 台のディスクを網羅するまで fsync の復帰を抑止、空パブリッシュを最大 R = 3 回連続実行 **状態：確定。** **進捗：実装済み。** |
| 9 | **空パブリッシュはユニットを書き出すか** | 会計ツリーが存在する場合は書き出す：空パブリッシュもパブリッシュであり、D5 (スナップショット・領域会計メカニズム) 確定項目 2 に従い会計行および連動する 4 要素を書き直す **状態：確定。** **進捗：実装済み。** |
| 10 | **パブリッシュ粒度** | ダーティノードを一定数バッチ化（または時間到達）した段階でルートを発行し、トランザクションごとの発行は行わない；パブリッシュ頻度は D16 (パブリッシュセマンティクス) のみが規定 **状態：確定。** **進捗：実装済み。** |
| 11 | **journal を迂回して直接ルートを発行できるか** | 不能：すべての構造・すべての操作が同一の journal を経由し同一の checkpoint リズムを共有する、fsync は単にパブリッシュを前倒しでトリガーするのみ **状態：確定。** **進捗：実装済み。** |
| 12 | **中間状態の正当性セマンティクス** | D8 (コアインデックス構造) の「中間状態はすべて正当なファイルシステム状態でなければならない」を「ルート + journal の任意プレフィックス、replay 後に正当」に改定 **状態：確定。** **進捗：実装済み。** |
| 13 | **journal replay の帰属** | レコード照合器に帰属し、O2 (独立パーサ + checker) には帰属しない；これは replay の第 2 の実装であり、D16 (パブリッシュセマンティクス) において最も高コストな項目である **状態：確定。** **進捗：実装済み。** |
| 14 | **スナップショット発行点の発行位置** | checkpoint 境界に整列し、当該 checkpoint の全データ変更の直後に固定；同一ウィンドウ内の 2 つのスナップショットは `(txg, スナップショット id)` でタイブレーク **状態：確定。** **進捗：未実装。** |
| 15 | **checkpoint C で発生した解放が割り当て可能集合に入る契機** | C が発行されるまで侵入を禁止；2 つの理由があり、第 2 の理由は journal 位置に基づく厳格な制約である **状態：確定。** **進捗：実装済み。** |
| 16 | **checkpoint とメッセージバッファの償却ウィンドウ境界** | 双方は同一の効用を購入しており、相違はウィンドウがメモリ上かディスク上かのみ；E7 (オフラインインデックス harness) の対照群は「checkpoint あり、メッセージバッファなし」に固定 **状態：確定。** **進捗：対象なし。** |
| 17 | **fsync エラー発生後に呼び出し側が前提とできる事項** | エラー発生後、今回のパブリッシュデータは出現している場合も出現していない場合もある、旧メモリ状態を再利用してのリトライは前提不可；現行ルートの読み出しにより成否を判定 **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：defer ウィンドウはディスク単位のロールバック下限の厳格化形態を採用し、状態数として読み取る

**確定内容**：ロールバック候補集合は復帰可能な直近 4 個の異なる状態を保持する——ユーザー可視状態を変更したルートのみを計数し、空プッシュおよび F 引き上げによる空パブリッシュルートは含めない；同一状態は 1 回のみ計数する（重複排除判定は「ディスク上で『非空』をいかに識別するか」の段落を参照）。**ルートリングの容量がこの規定の境界である**：2 つの状態がルートリング 1 周を超えて交互に出現する場合、リング内のルートはこれら 2 状態のみで占有され、保持されるのはリング内に実在するもののみとなる。いかなる機構もリング外の状態を提供することはできない。**ウィンドウの単位はパブリッシュであり秒ではない**：defer ウィンドウが保護するのは「リング内のルートが依然として使用可能か否か」であり、調査対象となった消費者に秒単位で価格付けを行うものは皆無であった。形態は E139 (按盘回退下界的收严形态) のアーム G2S を採用；回収閾値がリング内最古有効ルートへ後退した後、実験装置の候補下限はシステム構成内の F を内包する（E139 (按盘回退下界的收严形态) 第 2 回再実行；装置内の RGall 系統およびウィンドウアームは内包し、アーム G2S 自体は内包しない、RGall は G2S に条文形式の候補下限を加えたものであり単体テスト K15 が固定）、リカバリ後の有効値は引き続き装置の旧式に従って取得する（E139 (按盘回退下界的收严形态) 逐条適合表「有効」行、相違点 ③）。この文言は依然として装置の実行結果と完全には一致しておらず、取捨選択は C660 (按盘回退下界的定案那一句与装置对不上) に未済として計上されている：

| 項目 | 確定内容 |
|---|---|
| 再割り当て可能 | 解放済み ∧ 解放代 ≤ max(F_有効, リング内最古有効ルート) |
| リング内最古有効ルート | ディスク上の全ルートスロットにおいて自己証明に合格し、インスタンステーブルの判定により依然として有効であるルートの最小 checkpoint_txg；書き込み失敗のスロットは旧内容として扱い、インフライトで未永続化のパブリッシュは含めない。**同一値をとる 6 箇所**（ユーザー 2026-10-04 確定、攻撃検証済み）：アロケータの回収閾値（F 引き上げ時を含む）、checker の候補フィルタリング、checker の I-3.1 (割り当て済み統計の合致) 丁-defer 閾値（max(F_有効, リング内最古有効ルート)）、シャドウ会計の候補フィルタリング、ロールバック候補集合、F 引き上げの上限（「非空」判定時を含む）；6 箇所はいずれも 1 周ウィンドウによるルート除外を行わず、ウィンドウ外の有効ルート（レコード後・ルートスロット前のクラッシュによりスロットに残存した旧ルート）も候補集合に含まれ、閾値を抑止する。**マウント時に読み出せないスロット**：N = リング内で読み出せた最新有効ルートの txg（マウント時のアロケータ再構築時はこれを採用し、first_txg − 1 は不採用；プロセス内のルート記録時は当該ルートを採用）；読み出せないスロットは txg > N − 3S（1 周、3S はリング長）において 1 つまたは 2 つの txg のみに対応する：閾値は最小値で計算し、ルートリングテーブルには最大値を記録（ゼロ回攻撃、攻撃者モデルでのみ計測）、F 引き上げ時のルートリングテーブルのマウント時読み出し不能項目も本規定に従い最古有効ルートに合流する；1 周ウィンドウという量はここでのみ使用され、読み出し不能スロットが存在し得る txg の範囲を画定する。プロセス内でルートを記録する際、ルートリングテーブルの当該項目が最小となった場合は、まず当該スロットを再読み出しする：読み出し可能かつ自己証明合格、インスタンステーブル上依然有効であれば、読み出したルートに従って計算；読み出し可能だが自己証明不合格の場合は、依然読み出し不能と同様に処置し、回収閾値はそれが存在し得た最古 txg に固定する（「ルートスロットの今回の読み出し破損」行）；読み出し可能かつ自己証明合格であるがインスタンステーブルにより破棄されたものは、ルートなしとして扱う（ユーザー 2026-10-03 確定、まず再読み出し；2026-10-04 自己証明不合格セルを記録された最大 txg と規定、同日後に読み出し破損スロットを回収閾値において存在し得た最古 txg に固定と再決定、2026-10-05 最古 txg 固定を確定、ゼロ回攻撃）；再読み出しで有効ルートが得られたスロットは、「本プロセスがルートの存在を把握しているスロット」に算入する（「ルートスロットの今回の読み出し破損」行）。 |
| ロールバック下限 F | ルートレコードに 8 バイト（D22 (ユニット原子性の合成方法) 確定項目 7、242 → 250）、システム構成に別途 8 バイト記録（D22 (ユニット原子性の合成方法) 確定項目 9）。通常時は変更せず、ウィンドウはルートリング全体となる；受入容量不足時に引き上げ、正常アンマウント時に現行版の txg へ引き上げる |
| F 引き上げの上限 | **受入時の F 引き上げ**：min(各ディスク上の最新の永続有効ルート, 第 4 位に新しい非空永続有効ルート)；非空有効ルートが 4 個未満の場合は最古の有効ルートを採用；「有効ルート」「非空」はリング内の全ルートを対象に評価；マウント時点で読み出し不能または自己証明不合格であり、以降本プロセスが書き込みを行っていないスロットは、非空有効ルートである可能性を考慮し、F はこれを超過しない（ユーザー 2026-10-04 確定、ゼロ回攻撃：当初読み出し不能スロットのみ合流と規定したが、コード第 3 ラウンド後に読み出し不能および自己証明不合格の双方を合流と再決定；「リング内最古有効ルート」行で読み出し不能スロットを回収閾値に合流させたのと同一の思想）。破損スロットは上限に合流し、行き詰まった場合は今回のマウントで ENOSPC を報告し空パブリッシュは行わない；全ゼロが読み出されたスロットは上限に合流せず、F は全ゼロスロット内の有効ルートを超過し得る（ユーザーが認容したコスト）。1 回の処置目標 = min(今回の解放の解放世代, 第 4 位に新しい非空ルート)。**アンマウント時の F 引き上げ**：現行版の txg まで引き上げ、上限判定は別途行わない（上限として最新の永続有効ルートを採用し、ディスク間の最小値をとらないことと同等） |
| ルートスロットの今回の読み出し破損 | 受入時 F 引き上げの上限算出、マウント中の管理者ロールバック候補判定（ルートリング内にターゲットが存在するか否か）、および F_有効 算出時のルートリング読み出し：マウント時点で読み出し不能または自己証明不合格であり、本プロセスが以降書き込みを行っていないスロットは、受入時 F 引き上げ上限算出時は非空有効ルートである可能性を考慮して F はこれを超過させず（「F 引き上げの上限」行、ユーザー 2026-10-04 確定、ゼロ回攻撃）、ロールバック候補判定および F_有効 算出時はルートなしとして扱う；マウント時点で読み出し可能、本プロセスが書き込み済みかつ FUA が復帰済み、あるいはプロセス内ルート記録時の再読み出しで有効ルートが得られたスロット（これら 3 種は「本プロセスがルートの存在を把握しているスロット」；アロケータ再構築の 2 パス中 1 パスのみ自己証明合格したスロットもこれに含まれ、F 引き上げパスは別途「2 パス中 1 パスのみ合格したスロット」の規定に従う）について、今回読み出し不能または自己証明不合格となった場合は 1 回再読み出しを行い、依然破損している場合は今回の F 引き上げまたは今回のロールバックを拒絶し、固有のエラーメンバーを報告し、ルートの有無を推測しない（F 引き上げ側はユーザー 2026-09-26 確定、導出、攻撃ゼロ回；ロールバック側はメイン agent 2026-09-28 確定、1 回攻撃検証済み：コード三方第 3 ラウンド判定 `research/prompts/m2-closeout-code-r3-main-verification.md` 第 1 節で未記載条文の選択を指摘、ロールバック拡張の拒絶誤り・許可誤りセルは非ヒット（単一サンプリング））。書き込み可能マウントのルート選択は C554 (崩溃恢复抛弃的根暂时读不出时影子账算不到) 乙-構成継続処置に従う（D23 (ジャーナルの役割と形式) 確定項目 14：システム構成が選択された版より新しいパブリッシュを目撃していた場合、読み出しフェーズを 1 回再読み出しし、依然として真であれば書き込み可能を拒絶）；読み出し専用マウントのルート選択、「リング内最古有効ルート」の扱いについては本セルでは未判定、C563 (读根环一槽读坏就当没有根，别的调用方没判) 参照；**証人された版へ戻る候補の判定でルートリングを読む**（D23 (ジャーナルの役割と形式) 確定項目 14 の戻り「選択されたルートの走査が失敗し、システム構成がそれを証人していないとき、証人された版へ戻る」：リカバリ・読み取り専用マウント・書き込み可能マウントの三箇所が同じ論理を使う；判定は一度行い、トリガ後の再走査のあとにもう一度行い、それぞれルートリングを一周読む）：読めない、または自己証明に通らないスロットは必ず一度だけ再読する；再読で自己証明に通るルートが読めたらそれで計算し、再読でも壊れていればルートなしとして扱い、「戻ると最終レコードは読めるがルートが読めないパブリッシュを飛び越すなら戻らない」という関門で戻らない扱いになり、三箇所とも選択された版が読めないと報告し、別の版は選ばない（ユーザー決定、原文は変更履歴；実装は次の crates バッチ待ち）；**読み出し破損スロットと全ゼロ読み出しスロット**：マウント時点の破損スロット（読み出し不能、または非ゼロだが自己証明不合格）および全ゼロ読み出しスロットは、回収閾値の 3 箇所（マウント時 m 取得不能分岐含む、プロセス内、F 引き上げパス）において存在し得た最古 txg に固定し、本プロセスが当該スロットを上書きするか、またはプロセス内記録時の再読み出しで有効ルートまたは破棄されたルートを読み出した時に初めて解除され、再読み出しで全ゼロの場合は固定を維持；破損スロットは受入 F 引き上げ上限に合流し、全ゼロスロットは合流しない（「F 引き上げの上限」行）。固定もディスクに永続化する：ルートレコードはこのパブリッシュで用いた回収閾値 t と固定スロット p を持ち、ライターは t = max(このルート自身が持つ F, アロケータがこのルートを記録した時の回収閾値) を取る；固定されたスロットに本インスタンスが書いたルートが存在してはならない（C773 (不良および全ゼロルートスロットの回収閾値固定がメモリ内のみ) の合成形態、ユーザー 2026-10-20 確定、攻撃ゼロ回；フォーマットを変更し、実装はシステム構成着地パッチのマージ後の実装バッチで行う）。この形態で判定できない二種の不良実装（固定されたスロットの再読み出しで有効ルートが読めたのにライターが固定を解除しない、根拠なく読み出し可能なスロットを固定する）は承認済みコストとはしない、C863 (固定スロットの二種の不良実装がイメージ上で判定できない) 参照。**システム構成が目撃していないルートにより新規割り当てされたユニット**：書き込み可能マウント時の事前隔離により保護され（D23 (ジャーナルの役割と形式) 確定項目 14：水位より上であり未適用の全 journal レコードが指定する配置先を隔離し、本プロセスが当該レコードの所属 txg に対応するルートリングスロットを上書きし、そのパブリッシュが配置先を取得し終えた後に初めて解除）、全ゼロスロットに対するプロセス内再読み出し契機では解除されない（ユーザー 2026-10-07 確定、攻撃ゼロ回）。**書き込み可能マウント時のアロケータ再構築におけるルートリング読み出しパス**：1 台のディスクで破棄されていない自己証明有効ルートが 1 本も読み出せなかった場合（読み出したスロットの所属領域によりディスクを判別）、リング全体を 1 回再読み出しし、再読み出し前にフックを呼び出す；再読み出し後、スロットごとに 2 パス中より良好な結果を採用（いずれかのパスで自己証明合格のルート、双方合格の場合は再読み出し側を採用 ＞ 非ゼロ破損 ＞ 全ゼロ）、ルートリングテーブル、回収閾値、シャドウ会計はいずれも統合結果を使用；2 パスを統合しても破棄されていない自己証明有効ルートが 1 本も存在しない場合、番号取得前に書き込み可能マウントを拒絶し、`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread` を報告する（読み出しフェーズでルートを選択できない `Recovery(NoValidRoot)` とは分離して報告）。**アロケータ再構築の 2 パス中 1 パスのみ自己証明合格したスロット**（2 パスをスロットごとに統合すると自己証明合格ルートであり、他方のパスでは読み出し不能、自己証明不合格、または全ゼロ；実装は `crates/singlefs-core/src/allocator.rs` の `RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）：受入時 F 引き上げ上限算出、F 引き上げシーケンス（受入およびアンマウント）での回収閾値とシャドウ会計の再計算時、今回の読み出しで破損した場合は 1 回再読み出しを行い、依然破損していても拒絶せず、マウント時に読み出されルートリングテーブルに記録されたルートに従って計算：有効ルートはそれ自体に従い上限および候補集合に入り、破棄ルートは記録された配置先に従いシャドウ会計に入り上限には入らない（本処置の実装は攻撃ゼロ回）；マウント中の管理者ロールバック候補判定および F_有効 算出時は「本プロセスがルートの存在を把握しているスロット」として扱い、破損かつ再読み出しも破損の場合は従来どおり拒絶（「拒絶しない」アームは計測済み）；本プロセスが当該スロットを上書きした時点で分類が変更され、F 引き上げを跨ぐ固定は残さない：F 引き上げパスの破損時に補填されたルートは当該 F 引き上げ内でのみ有効。**F 引き上げシーケンス（受入およびアンマウント）における破棄ルート**：いかなる分類のスロットに存在しようとも、マウント時に算出されルートリングテーブルに記録された配置先に従ってシャドウ会計を一律に計算し、ディスクから会計を再読み出しせず、読み出し不能でも拒絶せず、D23 (ジャーナルの役割と形式) 確定項目 14 のクレデンシャル免除には従わない（C694 (被抛弃的根的账读不出时正常卸载被拒) の B）；「クラッシュ後の再書き込み可能マウント」パスは B に従い拒絶し、変更なし（ユーザー 2026-10-07 確定、ルートリングライン第 10z バッチで実装、攻撃者モデルでのみ計測、攻撃ゼロ回）；同一シーケンスのルートリング読み出し時、破棄ルートのスロットが破損し再読み出しも破損した場合は拒絶せず、ルートリングテーブルに記録された配置先に従いシャドウ会計に入り上限には入らず、有効ルートのスロットは「本プロセスがルートの存在を把握しているスロット」の規定に従い破損かつ再読み出し破損で拒絶（ユーザー 2026-10-08 確定「破棄ルートのスロットは拒絶しない」）；ルートリングテーブルに当該破棄ルートの配置先が記録されておらず（書き込み可能マウントが自ら行を書き込んだ後に破棄ルートと判定され、マウント時点では有効ルートとして記録されていた場合）、かつ C の輪（D23 (ジャーナルの役割と形式) 確定項目 14「書き込み可能マウント時の事前隔離」）が当該スロットに同一ルートを記録している場合（当該スロットに有効ルートが記録され txg が一致）、C の輪に記録された配置先に従って計算し、会計を読み出さず、読み出し不能でも拒絶せず、C にも記録がない場合は従来どおりディスクを読み出す（ユーザー 2026-10-08 確定「C の輪に記録された配置先を使用」）；当該ルート (R) が書き込み可能マウント時点で会計が読み出せずレコードが揃っている場合、C の当該スロットはさらにその時点で現行会計において依然割り当てられている配置先を合流（丙、ユーザー 2026-10-08 確定「丙に変更」；1 回攻撃を受け 2 形態ヒット、丙に中間およびロールバック復活を合流させる形に補正）、さらに選択された版と R の間の各パブリッシュが C の該当スロットに記録した配置先を合流（丙に中間合流、ユーザー 2026-10-08 確定「丙で穴埋め継続」、1 回攻撃検証）；R のレコードが指定する系統片の中に rollback_txg が R の txg に等しい系統レコードが存在する場合、R は管理者ロールバックであり、ロールバックターゲット R_old はルートリングから読み出されインスタンステーブル上有効かつ txg が当該レコードの T_old に等しいルートであり、R のスロットはさらに R_old 会計で割り当て済みかつ現行会計で解放済みの配置先を合流（ロールバック復活、同一確定、1 回攻撃検証）；ロールバックと認識されず R のレコードが指定するパックユニットに読み出し不能またはデコード不能なものが存在する場合、あるいはロールバックと認識されたがルートリングから R_old が読み出せないか R_old 会計が読み出せない場合、R のスロットは現行会計の全解放済み配置先を合流（広域保護、ユーザー 2026-10-08 確定「認識不能時は広域保護 tri」、1 回攻撃検証）；指定されたパックユニットがすべて読み出し可能・デコード可能であり、かつ当該系統レコードが真に存在しない場合は、丙の現行どおり（実装は `crates/singlefs-core/src/mount.rs` の `placements_in_the_accounts_of_newer_readable_roots`、`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`、`placements_a_rollback_publish_revived_released_in_the_current_account`）；これらは C の輪の他の配置先と同一の契機で解除される（本プロセスが R のスロットを上書きした時点）。受入による F 引き上げが上限で停止し、C の輪がこの時点で依然としてスロットを隔離している場合（輪全体で判定し、広域保護と C の他の部分を区別できない）、パブリッシュは `UserChangeRefused::NoSpaceAfterRaisingTheFloor` を報告し、その `wide_protection_at_the_ceiling` には R のスロット、R の txg、R スロットを上書きするために必要な空パブリッシュ回数、ディスクごとの遮断スロット数、および 2 つの解決策（正常アンマウント後の再書き込み可能マウント；リングプッシュエントリの呼び出し）を含み、自動的なリングプッシュは行わない（`crates/singlefs-core/src/mounted_session.rs` の `StuckBehindTheWideProtection`）；リングプッシュエントリ `MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states` はユーザーの明示的呼び出し時にのみ F 不変の空パブリッシュを発行し、R スロットが上書きされるまで継続し、呼び出しによりロールバック候補集合が D16 (パブリッシュセマンティクス) 確定項目 1 の確定内容「ロールバック候補集合は直近 4 個の復帰可能状態を保持」の 4 個を下回ることを確認する（ユーザー 2026-10-08 確定「解決策付きエラーと明示的リングプッシュコマンド」；解決策付きエラーとプッシュエントリは攻撃ゼロ回）；「破棄ルートのスロットは拒絶しない」および「C の輪に記録された配置先を使用」の 2 条は三方レグの複製およびモデルでのみ計測、攻撃ゼロ回 |
| 有効化 | F は各ディスクのシステム構成に別途記録（「F 引き上げシーケンス」行に先書き後引き上げを明記）；リカバリ後の有効値 F_有効 = max(各生存ディスクの最新永続有効ルートが保持する F の最大値, 破棄されたタイムライン上のルートが保持する F の最大値, システム構成から読み出せる F の最大値)；システム構成から読み出せる値は各生存ディスクの 2 スロット中自己証明合格（チェックサム合格かつ fsid 一致）の全スロットを対象とし、番号取得式と同一の抽出方法とする（D18 (ブロックに含める情報) 確定項目 11） |
| ロールバック候補集合 | 現行版のインスタンステーブルの判定により依然有効 ∧ txg ≥ F_有効（D23 (ジャーナルの役割と形式) 確定項目 14 のマウント中ロールバックも同一集合を採用） |
| F 引き上げシーケンス | 受入とアンマウントで共用。ディスク操作前にアロケータの複製上で空パブリッシュシーケンス全体をリハーサル：1 回でも配置先が取得できなければ 1 回も発行せず、アロケータを F 引き上げ前の状態にそのまま復帰（`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）；**リハーサル合格後、最初のパブリッシュの前に、まず新 F を各ディスクのシステム構成に書き込みバリアを通過させる**。その後に初めて新 F を保持する最初のルートを発行する；**先行書き込み段階でいずれか 1 台のディスクが失敗した場合、本シーケンスのルートは 1 本も発行せずエラーを返す；他ディスクのシステム構成に書き込み済みの新 F は保持されロールバックせず、回収に使われた F は後退しない**（メイン agent 2026-09-26 確定、推論、攻撃ゼロ回）；新 F が全ディスクで有効になる前に電源断した場合、次回のマウントはより低い値を読み戻しうる（先行書き込みが 1 台のディスクのシステム構成 1 スロットにしか落ちず、そのスロットが次回の書き込み可能マウントを通じて読み出せないままの場合、ディスク上の F 有効値は 5 → 0）、回収はまだそれを使っていないので違反とはしない（ユーザー 2026-10-20 確定「適用範囲を狭め、テストケースを名指しする」、攻撃ゼロ回の文言；テストケース `crates/singlefs-harness/tests/root_ring_reads_that_fail_are_not_taken_as_no_root.rs` の `a_floor_raise_cut_after_its_first_system_configuration_slot_falls_back_to_zero_without_reusing_units_when_that_slot_stays_unreadable_through_the_writable_mount`）；空パブリッシュは各ディスクに新 F を保持するルートが 1 本以上配置されるまで継続；本シーケンスにより回収されたスロットは有効化（全ディスクに新 F ルートが配置完了）まで保留され割り当て不可、有効化後に開放（正常アンマウントは例外、下記「正常アンマウント」行参照）；実発行のディスク書き込み途中でエラーが発生した場合、ディスク書き込み完了回数を返却（`MountError::RaiseFloorSequencePublishFailed`）、保留スロットの解放方針は未規定、C546 (抬 F 被拒时扣住的槽不退回) 参照 |
| 正常アンマウント | 一連の空パブリッシュを実行し F を現行版の txg へ引き上げる（「F 引き上げシーケンス」は受入の手順に従い、上限は「F 引き上げの上限」行参照）；本シーケンスで回収されたスロットのうち、解放代が「各ディスク上の最新有効ルートの txg」の最小値以下である区間のみ即時利用可能（リハーサルと実発行の双方で算定）、残余は受入シーケンスと同様に有効化まで保留（リカバリ時のルート選択は F を参照せず、1 ディスク喪失時は残存ディスク上の最新ルートを選択するため、これを参照するユニットが上書きされてはならない）；1 ディスクでも有効ルートが読み出せない場合は本区間は空となり全区間が有効化まで保留、今回読み出し不能・自己証明不合格のスロットはルートなしとして扱う；アンマウント途中でクラッシュ（システム構成は新 F を保持するが新 F ルートが全ディスクに行き渡っていない）した後の次回書き込み可能マウントも同様に一区間のみ開放：アロケータ再構築時に即時利用可能なのは解放代が max(min(F_有効, 「各ディスク上の最新有効ルートの txg」の最小値), リング内最古有効ルート) 以下のスロットにとどまり、特定ディスクで有効ルートが 1 本も読み出せない場合はリング内最古有効ルートまでにとどまる（マウント時読み出し不能スロットは存在し得る最小 txg でこの最小値に合流、回収閾値と同一値）、残余は今回のマウントのウォームアップが全ディスクにルートを配置するまで保留（ユーザー 2026-10-04 確定、ゼロ回攻撃）；本シーケンスが書き込むルートはルートレコードの flags にアンマウントマークを付与（D22 (ユニット原子性の合成方法) 確定項目 7）；第 1 版の幾何構造（R = 3、領域帰属 0 / 1 / 0）下では空パブリッシュを 2 回または 3 回実行（現行 txg を 3 で割った余りが 1 の時は 3 回、0 または 2 の時は 2 回）；クリーンクローズフラグは付与せず、次回マウントは従来どおりリカバリを実行（D23 (ジャーナルの役割と形式) 確定項目 14）；現行版のツリーテーブルが 0 件（mkfs 後にウォームアップのみ行い、初ファイル作成前）の場合は 1 バイトも書き込まず、F も変更せず、空パブリッシュも行わず、アンマウントは正常終了（`crates/singlefs-core/src/mount.rs` の `Unmounted::NothingWrittenOnAVersionWithoutFile`） |
| 受入 | 割り当て可能 = min(再割り当て可能 + 生存メタデータ − 予約プール, `df`)；受入容量不足、または受入通過にもかかわらず配置先が取得できない場合は、まず空パブリッシュを実行して F を引き上げてから再判定（行書き込みパブリッシュの前には実行しない：新インスタンスの初回パブリッシュであり、メタデータはスイッチ予約を使用、同パブリッシュ内のユーザーデータ再実行は通常受入を評価し不足なら ENOSPC）；**書き込み可能マウント時に受入容量が不足する場合、行書き込みパブリッシュはスイッチ予約で実行し、行書き込み完了後に F 引き上げ空パブリッシュを実行して再判定**；1 回の受入において最大 B = 4 + 2 k_tol 回のパブリッシュ（k_tol = 2 ⇒ 8）、上限まで実行しても不足の場合に初めて ENOSPC を報告；マウント箇所で上限まで実行しても不足の場合もマウントは書き込み可能として完了し、以降のパブリッシュは通常どおり判定され不足なら ENOSPC、正常アンマウントは通常どおり実行（D2 (RAID 条带策略) 確定項目 13 セルの例外）（ユーザー 2026-09-26 確定、攻撃ゼロ回）；マウント継続実行の削除系パブリッシュが配置先判定で空間不足により拒絶された場合も同様に F 引き上げを実行して再試行、上限でも収容不能ならマウントは書き込み可能として完了（D8 (コアインデックス構造) 確定項目 16 第 6 条、ユーザー 2026-10-03 確定） |
| `df` | 未使用 + 解放済み + 生存メタデータ − 予約プール − 空プッシュ最悪残存量 − 滞留量；予約プール = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max、残存量 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞留量 = 解放代 > max(第 4 位に新しい非空有効ルート, リング内最古有効ルート) であり空パブリッシュで解放されていない解放済みブロック |
| リング深度下限 | 領域ごとのスロット数 S ≥ k_tol + 2 = 4（D22 (ユニット原子性の合成方法) 確定項目 2 の S 下限も連動して引き上げ） |

**ディスク上で「非空」をいかに識別するか**：有効ルートが非空である ⟺ 当該ルートのツリーテーブル内のユーザー可視ツリー（inode ツリー、extent ツリー、および全スナップショットツリー：コード 14 スナップショット inode ルート、コード 15 スナップショット extent ルート、D8 (コアインデックス構造) 確定項目 9）のルートポインタが、直前の有効ルート（txg 順；有効 = インスタンステーブル上有効 ∧ txg ≥ 現行 F）のツリーテーブル内のものと異なる；スナップショット作成パブリッシュは非空と判定する。比較対象はツリーテーブル内のこれらエントリであり、ツリーテーブルユニット自体の配置先ではない——空パブリッシュを含むすべてのパブリッシュにおいてツリーテーブルユニットは書き直されるため（`crates/singlefs-core/src/transaction.rs`）、配置先で比較すると全空パブリッシュが非空と誤判定される。**「第 N 位に新しい非空有効ルート」および「4 つの異なる状態」の計数における重複排除**：最新から過去へ遡り、ある非空ルートの (inode ツリーポインタ, extent ツリーポインタ, 全スナップショットツリーポインタ) が、それより新しく計数済みのすべてのルートと異なる場合にのみ計数する——管理者ロールバックが発行した新ルートは直前ルートと異なり非空判定となるが、内容はより過去のルートと同一であり、重複排除を行わないと同一状態が 2 回計数されてしまう（第 2 ラウンド X7：往復ロールバック後に内容ベースで 2 状態しか残存しない現象）。逆方向については、第 4 の状態のルートがルートリングから退出するまで F の進行を停止させるのみである。

**新規プールでのファイル作成バイトの変更：あり**：ルートレコードに F 8 バイトを追加；新規プールでのファイル作成において F は常に 0（mkfs および初回パブリッシュ時は非空有効ルートが 4 個未満であり、上限は最古有効ルート txg 0 となる）。システム構成に F 8 バイトを追加、新規プールでのファイル作成では 0 を書き込み（D22 (ユニット原子性の合成方法) 確定項目 9）。

**連動改定**：D22 (ユニット原子性の合成方法) 確定項目 7 のルートレコードに F 追加（242 → 250）、flags にアンマウントマーク用ビット確保、確定項目 9 のシステム構成に F 8 バイト追加、確定項目 2 の S 下限引き上げ；D23 (ジャーナルの役割と形式) 確定項目 14 のロールバック候補集合に「txg ≥ F_有効」を追加；D3 (領域割り当て) 確定項目 9 第 2 条に境界を明記；I-7.4 (直近 K 代ブロック非再利用) の K を F による表現に改定；I-7.9 (ロールバック下限 F は F 引き上げ上限以下) をアンマウントマークの有無で 2 分岐化。

**射程**：defer ウィンドウおよびロールバック下限の算出方法を規定するものであり、リング幾何構造や c_max の計測条件は規定しない（c_max は公約された最悪空パブリッシュオーバーヘッドであり、D28 (マウント期コミットメント量) 確定項目 4 に従い当時の構造に基づき随時算出）。判明している 11 点のコーナーケース：

- **「有効化」に SysPre を採用**：F がルートレコードのみに格納されルートスロットがミラーリングされない場合（確定項目 7）、新 F を保持するルートが破損すると F_有効 が後退し、より過去のルートがロールバック候補集合に復帰し、それらが参照していたブロックが正当に再利用済みとなる——I-7.4 (直近 K 代ブロック非再利用) がこの履歴上で破綻する（`research/prompts/m2-step45-code-r2-main-verification.md` 第 4 節、`research/prompts/m2-step6-checker-r1-main-verification.md` 第 4 節）；根本原因は F がルートレコードのみに存在することにあり、有効化規則の選択とは無関係である（第 3 ラウンド K4：F 保持ルート全損セルにおいて、第 3 ラウンド規則、MAX、MAX+HOLD 候補の破損が 12 / 140 / 12 に対し SysPre は 0）。コスト：システム構成が 8 バイト増加（フォーマット変更）；F 引き上げ（受入およびアンマウント）ごとに全ディスクでシステム構成書き込みが 1 回増加しバリアが 1 回追加；checker およびクラッシュポイント再生がシステム構成内の F を読み出す必要が生じる——当時の checker はルート上の F で判定していたため、F 保持ルート全損時に I-7.4 (直近 K 代ブロック非再利用) 等 5 項目で誤って赤判定を出していた（第 3 ラウンド Y7、推論）；checker およびクラッシュポイント再生のレコード照合器はシステム構成の F を読み出し済みであり、C556 (checker 与崩溃点重放不读系统配置里的 F) は清算済み。「先書き後引き上げ」は第 3 ラウンド攻撃側が提案し攻撃ゼロ回、ユーザーが 2026-09-26 に合意・確定。破棄されたタイムライン上のルートが保持する F も F_有効 に算入（ユーザー 2026-09-28 確定「算入」、1 回攻撃検証済み、判定 `research/prompts/abandoned-floor-r1-main-verification.md` M2：破棄ルートが他より高い F を持つ場合、算入しないと I-7.12 (システム構成 F は同ディスクルート上 F 以上) が赤となり、ロールバックがその F 未満に達し得るが、破棄タイムラインの F 引き上げ時に既にスロットが回収されているため；実装 `crates/singlefs-core/src/recovery.rs` の `effective_rollback_floor_of_the_roots_read`、checker も `crates/singlefs-checker/src/walk.rs` で別途算出）；C331 (择根倒挂压过已确认的写) が R2 採用後に再現不能となった後（D22 (ユニット原子性の合成方法) 確定項目 8 射程）、書き込み可能マウントはスロットデバイスが読み出しエラーを報告し、かつ両スロットとも自己証明済みのディスクが 1 台も存在しない場合にのみ拒絶され、1 スロットのシステム構成が一時的に読み出せず他ディスクの両スロットが自己証明済みである盤面は拒絶されなくなった；破棄ルートが高い F を保持する盤面が人為的イメージ以外に生じるかは未再計測（推論）。`research/prompts/alloc-basis-r3-main-verification.md` の F 計測条件分岐とともに判定された件は依然オープン。
- F 後退後に第 0 代ルートが有効ルートに復帰し、そのツリーテーブルユニットが正当に再利用され、以降の全 F 引き上げがルートリングによる上書きまで拒絶される事態（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第 1 節 Z1）：前提は F の後退であり、SysPre 採用後は F 保持ルート全損セルにおいて発生しなくなった（推論、第 3 ラウンドでは未構成）。1 回のマウント内でルートリング容量を超える連続パブリッシュが発生した場合にマウント内回収を行わず、checker が正当な状態において I-3.1 (割り当て済み統計の合致) を赤判定する問題（E153 (账本形态与环上有洞的代价) で `crates/` 複製上で計測）、未解決、マイルストーン「上書き・解放・ロールバック・再利用」増補 2 収束表第 ② 行に帰属。
- **正常アンマウントのコスト**：アンマウントごとに空パブリッシュが 2 回または 3 回増加（ツリーテーブル 0 件版では不実行、確定文言参照）、第 1 版の幾何構造では 1 回あたり 8 ユニット、2 ディスク合計で 21 回の書き込み呼び出し、279,040 バイト（第 1 ラウンド攻撃側 t10、t5a、複製上で計測）；アンマウント後は F = 現行版の txg となり、ロールバック候補集合は現行版のみとなる。ユーザー選択時の合意コストは「停止時のクラッシュ安全ネットが最新ルート 1 本のみとなる」（条文からの推論）；第 2 ラウンドにおいてアンマウント後に再マウントし最新 j 本のルートが破損した 75 行でリカバリは全件正しく読み出し書き込み可能マウントも全件成功、本現象は生じず：F 未満のルートもリング内に残存しており、それらのレコードにより本シーケンス末尾まで状態を再生可能であるため（第 2 ラウンド判定 G4）。
- **ユーザーが合意・受容した 3 つのコスト**：削除された領域はユーザー可視状態を変更するパブリッシュがさらに 3 回発生して初めて割り当て可能集合に復帰し、それまで `df` は空き領域として計上せず、`df` に従った書き込みは真の ENOSPC となる；`df` の過少報告 = 予約プール + 残存量 = 15 + 13 c_max（c_max = 5 / 10 の時 80 / 145 ブロック）に滞留量を加算したものであり、滞留量 = 直近 4 ウィンドウの解放量の 4 倍、**上限なし**（1 ウィンドウで 8 ブロックのオブジェクトを 1 個削除時平均約 34 ブロック、最大 44 / 52、32 個削除時 1044 ブロック）、s バイト削除しても `df` は 1 バイトも増加しない；c_max は実測値に基づき、第 1 版プール規模 9 ブロック ⇒ 予約プール 73 ブロック。
- **最小 4 状態の保持は 1 回の処置におけるルートスロット書き込み失敗 ≤ k_tol 回の場合にのみ成立**：S = 4、c₀ = 5、3 回失敗セルにおいて 24 回違反し、字面どおり 1 回の敗北と記録、k_tol の射程外；全数探索下では S = 4、c₀ = 1 でも 160 回違反（同様に許容範囲超過セル）。
- **コストは選択肢自体の性質であり、本系統の欠陥ではない**：保持された状態はその時点で生存し後に死亡した全ブロックを固定するため、直近 4 個を選択すれば削除のたびに衝突する——滞留量および「削除後 3 状態待機」はいずれもここに起因する。
- **回収閾値をリング内最古有効ルートとするコスト**：レコード後・ルートスロット前のクラッシュによりリングに残存した旧ルートは 1 周上書きされるまで退去せず、ホールごとに回収閾値が 1 周分余分に抑止される（順方向レグポートで S = 16 において平均 893 ブロック vs ウィンドウ閾値 583 ブロック）；1 ディスクでルート書き込み不能が継続した場合、保留ユニットはパブリッシュに伴い線形増加し上限なし（攻撃側モデルで計測、15 / 30 / 60 / 120 回の継続失敗で最大 50 / 80 / 140 / 260 保留）；レコード後・ルートスロット前で 13 回連続クラッシュ後にマウントした場合（S = 4）、割り当て済み 166 ブロック、ウィンドウ閾値同一区間で 26 ブロック（攻撃側複製で計測）。数値は両レグが個別の計測条件で計測したものであり、リポジトリ内装置の数値ではない；ユーザーが 2026-10-04 に合意・確定。
- **E139 (按盘回退下界的收严形态) が回答不能な事項**：計数モデルであり、パブリッシュ単位で計数し壁時計では計数しない；ルートレベルの参照のみモデル化し journal レコード再生は未モデル化；インスタンス切り替え時のマウント時予約は D28 (マウント期コミットメント量) 確定項目 3 が控除；容量 4000 ブロック、ディスク 2 台；k_tol は 2 に固定；滞留量の計測条件は登録者が状態数解釈のために構成したもの；予約プールスキャンの最小値はスキャン下限に位置し真の必要量は未計測；リング深度下限はプローブの数値。先行する 2 ラウンド E135 (動的ロールバック下限) および E138 (按盘回退下界与推空的空间要求) の結論は E139 (按盘回退下界的收严形态) により置換され、証拠としての使用を終了。
- **I-7.9 (ロールバック下限 F は F 引き上げ上限以下) の再計算上限は 6 箇所の対象外**：プールレベル checker および GPU 判定器の再計算上限は引き続きイメージ最新ルートをアンカーとする 1 周ウィンドウ内のルートのみを対象とする（`crates/singlefs-checker/src/walk.rs` の `judge_rollback_floor_raises_against_their_ceilings` ドキュメントコメントに理由を記載：後続イメージでの再計算時、ホールに残存した旧ルートが上限を低く算出し、正当イメージで誤って赤判定となるため）、6 箇所が「リング内最古有効ルート」へ後退した措置に本箇所は含まれない。
- **アロケータ再構築におけるルートリング読み出しパスはスロット単位でディスクに帰属させ、スロットごとに 2 パスを統合**：条文はユーザー確定に従い記述（「ルートスロットの今回の読み出し破損」行）、実装は第 10h バッチ。
- **スナップショットツリーのルートを「非空」および重複排除に合流**：条文はユーザー確定に従い記述（「ディスク上で『非空』をいかに識別するか」段落）、マイルストーン 4 第 3 バッチ丙 2 で実装：core `crates/singlefs-core/src/mount.rs` の `user_visible_trees_changed` と checker `crates/singlefs-checker/src/walk.rs` を同期改定；マージ前は `user_visible_trees_changed` は inode ツリーと extent ツリーのみを比較していた。

**根拠**：

- E139 (按盘回退下界的收严形态)：アーム G2S は全障害環境（活性化区間内クラッシュでディスク喪失なし、番号スキップ後 F 引き上げ、空プッシュ中 1..3 回のルートスロット書き込み失敗、不良スロット、オーバーヘッドの c_max への上昇）において偽候補ゼロ、復旧不能ゼロを達成、各セルが自前の `df` に従い偽性 ENOSPC は 0、1 回の受入でのパブリッシュ数は実測最大 7 ≤ 8 であり、事前定義予約プール下でデッドロックなし——本アームを確定内容として採用（事前登録 `e139-preregistration.md`、成果物 `e139-tightened-floor-2026-09-12.out`）。
- 4 ラウンドにわたる三方検証、第 4 ラウンド判定 `research/prompts/d16-item1-r4-main-verification.md`；第 3 ラウンド `research/prompts/d16-item1-r3-main-verification.md`。
- ユーザーによる 3 回の確定（候補集合は継続縮小可能、最小 4 個を保持、「4」が数えるのは復帰可能な異なる状態、3 種の「いずれの 4 個か」から直近 4 個を選択）、発言原文は変更履歴にある；「非空」の判定方法および「行書き込みパブリッシュ前の空プッシュ抑止」もユーザー確定、発言原文は変更履歴にある。
- ユーザー確定 2026-09-25：F 引き上げシーケンスをシーケンス全体のリハーサルに変更、2026-09-24 の「逐次パブリッシュ維持」を撤回（発言原文は変更履歴にある）；実装は実 26 (`m2-e26-implementer-report.md`)。
- 有効化に SysPre 採用、アンマウント時 F を現行版 txg へ引き上げ (B1)、「4 つの異なる状態」の重複排除およびルートリング容量境界、現行版インスタンステーブルによる候補判定：実験なし、三方プロトタイプ（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）において凍結複製上で計測、実験番号は未発行、プロトタイプ上の数値はリポジトリ内装置の数値ではない；3 ラウンド判定 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`（第 3 ラウンド第 3 節で成立、第 4 節でユーザーへ提示）。SysPre の「先書き後引き上げ」は攻撃ゼロ回。システム構成から読み出せる F を各ディスク 2 スロット中全自己証明合格スロットとし、先行書き込み失敗時にルートを一切発行せず書き込み済み F をロールバックしない 2 点はメイン agent 2026-09-26 決定、推論、攻撃ゼロ回。
- ユーザー確定 2026-09-25（正常アンマウント時の F 引き上げ、B1 採用）および 2026-09-26（C419 (F 生效之后靠什么保持不回落) に SysPre を採用し 2026-09-25 の「有効値を各ディスク F の最大値とする」を撤回；「4 つの異なる状態」にルートリング容量境界を追加）、発言原文は変更履歴にある。
- 「ルートスロットの今回の読み出し破損」行：F 引き上げ側はユーザー確定、発言原文は変更履歴にあり、構成された反例は `records/2026-09-24-里程碑二收尾调度.md` 第 3 節「実四丙 hand-back」行にある；ロールバック側はメイン agent 2026-09-28 決定（同スケジュール記録 309 行目）、経緯は変更履歴にある。実装はマージ済み：`crates/singlefs-core/src/recovery.rs` の `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` はルート存在把握スロットの「読み出し不能または自己証明不合格」に対して 1 回再読み出しを行い、依然破損なら `RootRingSlotKnownToHoldARootStillBadAfterOneReread`（`first_reading` / `reread` 付き）を報告；上限算出パスは `crates/singlefs-core/src/mount.rs` の `roots_of_the_ring_for_the_ceiling`（依然破損なら `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）、F 引き上げ・アンマウント時のシャドウ会計と回収閾値再計算パスは `MountError::NewerStateStillUnreadableAfterOneReread`（`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`）、管理者ロールバックの `rollback_candidate` 候補判定と F_有効 算出（`recovery.rs` の `effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）は `RollbackError::CandidateJudgementStillUnreadableAfterOneReread` を書き込み前に報告。テストケース：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` の `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write`、`a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised`、`rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` の `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` および `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` の `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write`（自プロセス書き込みルート改ざん後のロールバック拒絶、ディスク不変）（`m2-impl-r2-fixes-b-implementer-report.md` 第 3 節 3 行目；マージ後検証 1 で読み出し不能のみに縮小後、修正法 B で全文復元、`m2-impl-merge-verify-1-implementer-report.md` 第 3 節第 1 小節参照）。
- 「受入」行の配置先拒絶時プッシュ、マウント時行書き込み後プッシュ：ユーザー確定、発言原文は変更履歴にある；安全設計ラウンド第 3 ラウンド判定 `research/prompts/m2-safety-r3-main-verification.md`（セッション内配置先取得不能 C283 (准入失败时不先推发布就报 ENOSPC) を放置しクラッシュ再マウントで小容量ディスクがデッドロック）。
- 回収閾値および同一ウィンドウ採用の他 5 箇所のリング内最古有効ルートへの後退：判定 `research/prompts/m3-reclaim-window-r1-main-verification.md` 第 2 節 W1、W3 各行および第 6 節（ウィンドウ閾値が目標形態で破綻：1 ディスク喪失後マウントで残存ディスクのより旧いルートにフォールバックし破損データを読み出し、計数モデル全数探索；アロケータ閾値のみ、または checker のみの改定では I-3.1 (割り当て済み統計の合致) 誤赤が再発、複製で計測；6 箇所一斉後退により両レグ独立計測で偽候補ゼロ、復旧不能ゼロ、1 回攻撃検証済み）；ユーザー確定 2026-10-04（Q1 RGall、Q2 次ラウンド改定）、発言原文は変更履歴にある。E139 (按盘回退下界的收严形态) 第 1 回再実行：旧閾値対照アーム G2SW_ringgate 判定行は偽候補ゼロ、復旧不能ゼロ、ウィンドウ追加の G2SW は非ゼロ行あり；一部は装置の候補下限がシステム構成内の F を含んでいなかったことに起因（判定 W2 行）、数値は証拠とせず実験ページを正とし、装置への F 補填後の第 2 回再実行が 2026-10-05 完了：候補下限にシステム構成内 F を含め、RGall（回収閾値と他 5 箇所をリング全体へ後退）は第 1 回再実行の 13 分類環境および `disk_stuck` において偽候補、復旧不能、判定セルの偽性 ENOSPC すべて 0、数値は実験ページを正とする；収束未済に記録。実験なし：判定内の数値は三方レグの複製およびモデル上で計測、実験番号未発行、リポジトリ内装置の数値ではない。1 周ウィンドウ採用の三方判定（`research/prompts/m3-ring-r2-main-verification.md`、`research/prompts/m3-ring-r3-main-verification.md`）およびユーザー 2026-10-02 確定、経緯と原文は変更履歴にある。
- F 引き上げ上限へのマウント時読み出し不能・自己証明不合格スロットの合流：コードラウンド第 3 ラウンド判定 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第 2 節 K2 の RS3 行（マウント時から全ゼロ読み出し、読み出せるが自己証明不合格のスロットをルートリングテーブルに記録せず；受入 F 引き上げ上限が 22 と算出され、ディスク上の完全な非空有効ルート 21 を超過、プールレベル checker が I-7.9 (ロールバック下限 F は F 引き上げ上限以下) 赤判定；デバイスエラー時は 21 で停止、赤判定 0；複製で計測、メイン agent 再実行で一致）および第 6 節 Q3 行；ユーザー確定 2026-10-04（読み出し不能と自己証明不合格の双方を合流）、発言原文は変更履歴にある。実験なし：数値は攻撃側複製で計測、実験番号未発行。
- F 引き上げパスでのマウント時読み出し不能スロット合流、プロセス内記録前の再読み出し、F 引き上げ上限のウィンドウ内ルート限定（この半分は回収閾値の後退に伴い破棄）：コードラウンド判定 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第 2 節 K2-e、K2-b、K2-b2 の 3 行（攻撃側プロトタイプで計測、メイン agent 再実行 115 件の出力がバイト単位で完全一致）および第 3 節表 2 後のユーザー確定段落（e2、B2）。
- プロセス内再読み出しで自己証明不合格を依然読み出し不能と同一処置（2026-10-04 RS1 で当初記録された最大 txg 採用と規定、2026-10-05 ユーザーが最古 txg 固定を確定：最大記録下では RGall において mkfs txg 0 有効ルート参照ユニットを回収してしまうため、`records/2026-09-30-里程碑三开工第一轮.md` 第 10 バッチ core クラスタ手戻り項目）、再読み出しで有効ルートが得られたスロットをルート存在把握スロットに算入、F 引き上げ上限でマウント時読み出し不能スロットを非空有効ルートの可能性ありと扱う：コードラウンド第 2 ラウンド判定 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第 2 節 2、6 行目（有効ルートのスロットがマウント時に読み出しエラー、再読み出しでも破損、実装がルートなしと扱ったため回収閾値がディスク上有効ルートを超過、攻撃側モデル 12 パラメータセット中 12 セット赤判定、修正法 A で 12 セット 0 赤；F 引き上げが読み出し不能有効ルートを超過する形態で攻撃側プローブ F = 22 vs 読み出し可能時 F = 21、推論）および第 4 節 RS1、RS3 行（修正法 A のコストは未書き込みスロット + マウント読み出しエラー時の会計増加、安全側、2 セット；上限への読み出し不能スロット算入のコストは読み出し障害時の F 引き上げ幅減少、推論）；ユーザー確定は同判定第 6 節参照、原文は変更履歴にある。実験なし：数値は攻撃側モデルで計測、実験番号未発行。
- 正常アンマウントシーケンスで回収されたスロットの限定的開放：設計ラウンド判定 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`（継続実行上限到達後のアンマウントがリハーサルで拒絶、丙 1 成立、ゼロ回攻撃）；コードラウンド第 3 ラウンド判定 `research/prompts/m3-unlink-code-r3-main-verification.md` 第 2 節 n1 各行（丙 1 が単一ディスク残存時に選択された旧ルート参照ユニットを上書き、2 回サンプリングで同方向ヒット、メイン agent 再実行で確認；リカバリのルート選択は F を参照しない）および分岐表 `m3-unlink-code-r3-forks.md`。ユーザー確定 2026-10-03：「丙 1 を次バッチへ投入」、単一ディスクの設問「甲 GA（推奨）」、原文は変更履歴にある。実験なし：数値は三方レグ複製上で計測、実験番号未発行。
- 1 ディスクに有効ルートが存在しない場合の全区間保留、読み出し不能スロットをルートなしとして扱う、アンマウント途中クラッシュ後の再書き込み可能マウントでの限定開放：コードラウンド判定 `research/prompts/m3-batch14-code-r1-main-verification.md` 第 2 節（n1「ルートリング内の読み出し不能スロット、ホール」行はコードが採用した保守的解釈；「形態 R」行は攻撃側ヒット、メイン agent 再実行で確認、`research/prompts/m3-batch14-code-r1-main-checks/`）および分岐表 `m3-batch14-code-r1-forks.md`。ユーザー確定 2026-10-04 「甲 再マウント時も一区間のみ開放（推奨）」を選択、原文は変更履歴にある。実験なし：数値は三方レグ複製上で計測、実験番号未発行。
- 再マウント時に特定ディスクで有効ルートが 1 本も読み出せない場合に最古有効ルートまでのみ開放（当該判定では 1 周ウィンドウ内最古有効ルートと記述、回収閾値後退に伴い破棄）、即時開放式に最古有効ルート項を追加：コードラウンド判定 `research/prompts/m3-batch16-code-r1-main-verification.md` 第 2 節（n1「再マウント中に特定ディスクのルートリング全体が読み出し破損」行は攻撃側ヒット、メイン agent 再実行で確認、`research/prompts/m3-batch16-code-r1-main-checks/`；ウィンドウ項は順方向レグ指摘の条文記載漏れ）および分岐表 `m3-batch16-code-r1-forks.md`。ユーザー確定 2026-10-04 「甲 ウィンドウ内最古有効ルートまでのみ開放（推奨）」を選択、原文は変更履歴にある。実験なし：数値は三方レグ複製上で計測、実験番号未発行。
- ツリーテーブル 0 件版での正常アンマウントにおけるゼロバイト書き込み・F 不変：ユーザー確定 2026-10-04（乙「プッシュせず、条文は実装に従い記述」を選択、原文は変更履歴にある）、実装内のメイン agent 2026-09-26 確定（`crates/singlefs-core/src/mount.rs` の `Unmounted::NothingWrittenOnAVersionWithoutFile` コメント、推論、攻撃ゼロ回）を条文に明記；コードラウンド判定 `research/prompts/m3-formal-code-b16-r1-main-verification.md` 第 2 節 K1 および第 5 節 1 行目（字面どおり空パブリッシュ実行の解釈は攻撃側複製で計測：モデルが 2 回プッシュ、突き合わせが PublishCount を報告）。実験なし：数値は三方レグ複製上で計測、実験番号未発行。
- 「ルートスロットの今回の読み出し破損」行の最古 txg 固定、全ゼロスロット、再構築時全リング再読み出し後拒絶、スロットごと 2 パス統合：ユーザー確定 2026-10-04（読み出し破損スロットを回収閾値において最古可能 txg に固定）および 2026-10-05（全ゼロスロットは自己証明不合格に従い回収閾値で固定、受入 F 引き上げ上限のみ合流除外；再構築パスは 1 回再読み出し後拒絶；Q3 拒絶理由新設；Q4 rr2 採用；スロットごと 2 パス統合）、原文は変更履歴にある；判定 `research/prompts/m3-batch10-code-r1-main-verification.md` 第 2、5、6 節、`research/prompts/m3-batch10-code-r2-main-verification.md` 第 2、5、6 節。E173 (上限卡在读坏槽的三种处置) は本セルを支持しない。実験なし：数値は三方レグ複製およびモデル上で計測、実験番号未発行。
- 「ルートスロットの今回の読み出し破損」行のアロケータ再構築 2 パス中 1 パスのみ自己証明合格スロット：判定 `research/prompts/m3-batch10-code-r3-main-verification.md` 第 2 節 K1「統合分類はマウント時破損と同一分類か」行（第 1 パス合格・再読み出し以降破損のスロットをルート存在把握スロットと記録し、F 引き上げ再読み出し破損で引き上げ拒絶、書き込み可能マウントが行書き込みウォームアップ後に失敗）および第 6 節 S1；判定 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第 2 節 K1（マウント時破損に従い固定時、破棄側がシャドウ会計から脱落、I-7.4 (直近 K 代ブロック非再利用) 赤判定、受入上限がその txg で停止；「解除契機」セル）、K2（ロールバックと F 引き上げで同一スロットの処置相違）および第 6 節 t1、t2、分岐表 `m3-batch10i-code-r1-forks.md`；ユーザー確定 2026-10-06（別分類新設；マウント時読み出しルートリングテーブル記録ルートに従い計算；ロールバックは従来どおり拒絶、次ラウンドで非拒絶アームを先行計測）、原文は変更履歴にある。実験なし：数値は三方レグ複製および攻撃側プロトタイプ上で計測、実験番号未発行。
- 「ルートスロットの今回の読み出し破損」行の 1 パスのみ合格スロットがロールバック候補判定および F_有効 算出時に再読み出し破損で拒絶：判定 `research/prompts/m3-batch10o-code-r1-main-verification.md` 第 2 節 K1-3 セル（「拒絶」と「非拒絶」の両アームを両レグ複製上で計測、数値は当該セル）および第 5 節、分岐表 `m3-batch10o-code-r1-forks.md` T2 行；ユーザー確定 2026-10-06（従来どおり拒絶）、原文は変更履歴にある。実験なし：数値は三方レグ複製上で計測、実験番号未発行。
- 「F 引き上げの上限」行の破損スロット合流・全ゼロ非合流：E173 (上限卡在读坏槽的三种处置) アーム比較表（現行の破損スロット内非空ルート超過防止下で、空プッシュはこれより 4 / 15,552 箇所少なく拒絶（条文解釈）、空パブリッシュに配置先取得不能箇所が存在；非合流は 180 箇所で F 超過；全ゼロ非合流は条文解釈下で F が 90 / 7,794 箇所超過、全ゼロ合流は 18 箇所多く拒絶、数値は実験ページを正とする）；ユーザー確定 2026-10-05（Q1 現行どおり、Q2 現行非合流どおり）、原文は変更履歴にある。
- プールレベル checker および GPU 判定器の I-7.9 (ロールバック下限 F は F 引き上げ上限以下) 再計算上限が 1 周ウィンドウを維持：判定 `research/prompts/m3-batch10-code-r1-main-verification.md` 第 2 節 K3 および第 5 節（順方向衝突並列第 1、2 行）。
- 「ルートスロットの今回の読み出し破損」行のシステム構成未目撃ルートによる新規割り当てユニットを書き込み可能マウント時事前隔離で保護・プロセス内再読み出しで非解除：判定 `research/prompts/m3-c694-r1-main-verification.md` 第 2 節「K1 2 つの条文の各分担」行、`research/prompts/m3-c694-r2-main-verification.md` 第 2 節「K2 C の解除契機」行および第 3 節第 1 条；ユーザー確定 2026-10-07（C694 (被抛弃的根的账读不出时正常卸载被拒)「C と B の双方を実施」）、原文は変更履歴にある。実験なし：数値は三方レグ複製およびモデル上で計測、実験番号未発行。
- 「ルートスロットの今回の読み出し破損」行の F 引き上げシーケンスで破棄ルートスロット破損時に非拒絶・未記録時に C の輪記録配置先を使用：判定 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第 2 節 K1-1（H1、攻撃側：判定時実装で 90 / 90 履歴拒絶、非拒絶アーム 0 / 90、両アーム I-7.4 (直近 K 代ブロック非再利用)、I-4.8 (直近 K 代ルートチェックサム自己整合性) 0 赤）、K2-2（H2、判定時実装で順方向 19 / 19、攻撃側 285 / 285 拒絶、C アーム 0 拒絶、0 赤）および第 3 節；C スロットにマウント時依然割り当て済み配置先を合流：判定 `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第 2 節 K2-2 の 2 行（攻撃側：選択版と解放予定ユニットを共有する際 C アーム単独で k=3 53 / 150、k=5 68 / 150 履歴 I-7.4 (直近 K 代ブロック非再利用) 赤判定、合流アームで 0 赤、0 拒絶、メイン agent 凍結複製再実行で完全一致）および第 3 節；ユーザー確定 2026-10-08、原文は変更履歴にある。実験なし：数値は三方レグ複製およびモデル上で計測、メイン agent 凍結複製再実行、実験番号未発行。
- 「ルートスロットの今回の読み出し破損」行の丙中間合流、ロールバック復活、広域保護 (tri)、広域保護による受入停止時解決策付きエラーとプッシュエントリ：判定 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第 2 節 K2-1 の 2 行（丙が攻撃側 1 形態および順方向 1 形態にヒット）および第 4 節；`research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第 2 節 K1-1、K2-2（両分岐成立）、K2-3（ロールバック未認識形態）および第 5 節；`research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第 2 節 K1-1、K1-2（広域保護非ヒット）および K1-3（広域保護が解放履歴に伴い増大、書き込み拒絶で R スロット上書き不能、プロセス終了まで隔離継続）；`research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第 2 節 K1-1、K1-3（二重控除排除修正が非ヒット）；調査報告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md`（広域保護による受入停止 2 層構造とリングプッシュのコスト）；ユーザー確定 2026-10-08（「丙で穴埋め継続」「認識不能時は広域保護 tri」「解決策付きエラーと明示的リングプッシュコマンド」）、原文は変更履歴にある。実験なし：数値は三方レグ複製およびモデル上で計測、メイン agent 再実行、実験番号未発行。
- 「ルートスロットの今回の読み出し破損」行の F 引き上げシーケンス破棄ルートが記録配置先に従いディスク再読み出しせず破損時非拒絶：判定 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第 2 節 K4-2 セル（攻撃側計測：マウント後破棄ルート会計破損時 B に従いディスク再読み出しで 84 / 84 履歴アンマウント拒絶；記録配置先アームで 84 / 84 履歴アンマウント成功、アンマウント後 I-7.4 (直近 K 代ブロック非再利用) 0）および第 3 節；ユーザー確定 2026-10-07（F 引き上げは記録配置先に従う）、原文は変更履歴にある。実験なし：数値は攻撃側レグ複製およびモデル上で計測、実験番号未発行。
- 「非空」および重複排除に全スナップショットツリールートポインタを合流：ユーザー確定 2026-10-14（スナップショット作成パブリッシュを「非空と判定」）、原文は変更履歴にある；コード 14 / 15 エントリ形式は判定 `research/prompts/m4-snapshot-r1-main-verification.md` 第 3 節 S1 参照。実験なし：「状態」がいずれのツリーのルートポインタで構成されるかを定めるものであり、定量可能な数値は存在しない。
- 「ルートスロットの今回の読み出し破損」行の固定もディスクに永続化する（ルートレコードが t と p を持つ、t の取り方、固定されたスロットに本インスタンスが書いたルートが存在してはならない）：ルートリング三方判定 `research/prompts/m3-close-rootring-r2-main-verification.md` 第 2 節で C773 (不良および全ゼロルートスロットの回収閾値固定がメモリ内のみ) を判定したセルと `research/prompts/m3-close-rootring-r3-main-verification.md` 第 2 節 C773 (不良および全ゼロルートスロットの回収閾値固定がメモリ内のみ) の合成形態のセル（形態は維持、攻撃ゼロ回の補い方付き）；ユーザー確定 2026-10-20（「条件を外し、マイルストーン三で行う」「(1)(2) は認めるが、承認済みコストとはしない」）、原文は変更履歴にある。実験なし：緑化・誤赤ともに登録済みイメージがなく、両レグとも推論と標記。
- 「ルートスロットの今回の読み出し破損」行の、証人された版へ戻る候補の判定でルートリングを読むとき壊れたスロットを一律一度再読する件：ユーザー決定（ポップアップ、原文は変更履歴）；四箇所の呼び出し元でルートが一つ欠けたときの影響の調査は C563 (读根环一槽读坏就当没有根，别的调用方没判) の行にあり、影響があるのはこの一箇所だけ；赤になるテストケース `crates/singlefs-harness/tests/fall_back_candidate_root_slot_reading_bad_once_does_not_change_where_recovery_and_both_mounts_land.rs`。
- 「F 引き上げシーケンス」行で回収に使われた F は後退しない、新 F が全ディスクで有効になる前に電源断するとより低い値を読み戻しうる：調査（`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 382 行：重畳故障下で有効値 5 → 0、後退前に 5 は回収に使われておらず、データへの影響なし）とその行が名指しした回帰テストケース；ユーザー確定 2026-10-20「適用範囲を狭め、テストケースを名指しする」、原文は変更履歴にある。実験なし：数値は調査用テストと回帰テストケースで計測、実験番号なし。

**未済の検査**：C314 (崩溃恢复落到旧根后可以复用被抛弃的根引用的单元)（defer ウィンドウの大きさとは無関係、根本原因は D23 (ジャーナルの役割と形式) 確定項目 14 のシャドウ会計にある）；C329 (写行那次发布之前推抬 F 的空发布没有检查)；C563 (读根环一槽读坏就当没有根，别的调用方没判)；C660 (按盘回退下界的定案那一句与装置对不上)；C746 (进程内重读读出被抛弃的根时全零槽的钉同样解开)；C863 (固定スロットの二種の不良実装がイメージ上で判定できない)。

#### 確定項目 2：checkpoint のトリガー判定条件は時間とダーティデータ量のいずれか早い方とする

**確定内容**：`トリガー ⟺ (now − 前回 checkpoint 時刻 ≥ T_time) ∨ (ダーティバイト数 ≥ T_dirty)`。双方の閾値は**システム構成の宣言値**であり、マウント時に 1 回評価される（D21 (権威態と派生態の境界) の「ユニット幾何構造はマウント時に 1 回評価」と同型）。

**射程**：トリガー契機を定めるものであり、両閾値の具体値（確定項目 5）やトリガー後のトランザクション分割方針（同確定項目 5）は定めない。判明しているコーナーケース：`.claude/rules/fs-design.md` 必須要件 5 が規定する分岐変数はこれら 2 つの閾値であり、マウント期間中恒常である；`now` およびダーティバイト数は比較対象の変動量であり分岐変数ではない。

**根拠**：

- 実験なし：`∨` の形態は 2 つの障害モードが相互に規定し合った結果であり、定量可能な数値は存在しない——ダーティデータ量のみに依存すると、アイドル状態のシステムで最後の書き込み群が無期限にフラッシュされずクラッシュ損失ウィンドウが無限大となる；時間のみに依存すると、突発的な書き込みバーストにより 1 checkpoint の journal 消費がリング幾何構造を超過し、I-8.1 (リング幾何構造の十分性) が要求する「トランザクションごとの最悪 journal 消費」の有界性が破綻する。この `∨` の論理式こそがすべてであり、マジックナンバー以外の自由度は存在せず、`.claude/rules/fs-design.md` 必須要件 1「判定基準は明示的、計算可能、記述済みであること」を充足する。定量可能な数値は両閾値の設定値およびその実行可能領域であり、確定項目 5 にある。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 3：journal 記録形式は 4 層外の第 3 の独立凍結コンポーネントである

**確定内容**：journal は D15 (フォーマット凍結ポリシー) の 4 層連鎖には含めない。そのレコード形式——レコードヘッダ（確定済み保留トランザクション番号 + コミットフラグ、逆方向チェーン、ペイロードチェックサムの 3 増分を含む）、指定項目、ペイロードチェックサムの適用範囲、トランザクション境界——は 4 層外の第 3 の独立凍結コンポーネントとして立脚する；システム構成に格納される journal 幾何フィールド（レコードサイズ、リングサイズ、インフライトレコード上限、tail スロット）はシステム構成フィールドであり、第 1 層とともに凍結される。新規レコード形式は「新規レコード種別 + システム構成 log-incompat ビット」を採用する：リング内に当該形式のレコードが存在する時はビットをセットし、リングがクリーンとなった時にクリアし、認識不能なビットが存在する場合はマウントを拒絶する；恒久的に読み出し可能でなければならないのは「当該レコードを非適用と判定する」ために必要なヘッダフィールド群（magic、種別、自己記述長、`jsn`、`checkpoint_txg`、ヘッダチェックサム）のみである。

**凍結前提条件**（内包する各外部定義より先行してはならない）：第 2 層（指定項目内の位置エントリ）；配置先粒度セル（D3 (領域割り当て) 確定項目 7 の未済処置）；D18 (ブロックに含める情報) 確定項目 11 のユニット種別登録表（指定項目内のユニット種別タグ）；ツリー ID が属するツリーテーブルユニット——これには未だ凍結層帰属が定まっていないため、本前提条件は保留中である。**凍結契機は `format-frozen/journal-record.toml` を提出するコミットとし**、「レコードを書き出す最初のイメージ」等の記述を禁止する。

**射程**：journal のみに回答する。ツリーテーブルユニット、インスタンス表ユニット、およびルートレコードの凍結層帰属は依然として C246 (树表单元没有冻结层归属) のセルにある。判明している 3 点のコーナーケース：

- **セットおよびクリアは実行時アクションであり**、コミットのタイムラインには従わないため、別途実行時のゲート（新規形式レコードの初回書き込み前に永続的にビットをセットし、ビット欠落時は書き込みを拒絶）を整備し、コンポーネントの spec とともに配備する必要がある。
- **未だ凍結不能であり**、5 つの障壁が存在する：指定項目 56 バイトは総幅のみ確定しており内部構成は [layout/01-first-txn.md](../layout/01-first-txn.md)「VI. journal」の構想表にとどまり、暗号化フィールドの余白も未確保である（D23 (ジャーナルの役割と形式) 確定項目 1 項目 D「項目のサイズはレコード種別により決定」がこれを管轄）；レコードヘッダの登録フォーマット定数は 78 であり 3 増分の配備後は 95 となる；spec ファイルが存在しない；log-incompat ビットはビット番号が未定であり D15 (フォーマット凍結ポリシー) の feature bit 会計表にも未登録；凍結前チェックリスト第 1 項は幅を測り第 7 項は未確定項目を測るため、双方は `jsn` 解釈計測条件の衝突という未済を検知できず、現行チェックリスト下では本コンポーネント凍結時にいずれも赤判定とならない。
- **「いつリングがクリーンとなりクリア可能となるか」は本プロジェクトのリプレイ下限に従い決定しなければならず**（D23 (ジャーナルの役割と形式) 確定項目 14 の水位）、現行の他実装をそのまま模倣してはならない。

**根拠**：

- 実験なし：凍結層帰属はフォーマットポリシーであり、検証可能なのは層間の依存順序であって、定量可能な数値は存在しない；レコードヘッダ幅のバイト会計は C94 (登记的格式常量与后来的定案对不上) にある。
- 三方論証（資料 `_d16-items134-background.md`、成果物は同一ディレクトリの `d16-items134-*`：`d16-items134-sonnet-output.md`、`d16-items134-opus-output.md`、`d16-items134-local-output-s1.md` および `-s2.md`）：順方向レグは「独立コンポーネント」が D15 (フォーマット凍結ポリシー) 確定項目 1 と整合する唯一の回答であると判定し、逆方向レグが log-incompat ビットを補完した後に進化セルで勝利し、2 つの前提条件を特定した；ローカルレグの 2 回のサンプリングによる反例はフィールドの配置層を誤認しており、メイン agent の監査を経て不採用となった。判定は独立コンポーネント + log-incompat ビットを採用し、凍結前提条件は逆方向レグに従い厳格化した。
- 現行実装の参照：XFS には `XFS_SB_FEAT_INCOMPAT_LOG_ALL` が存在し、メインシステム構成でセットされログ内に保護対象エントリが存在しない時にクリアされる（ローカルツリー `fs/xfs/libxfs/xfs_format.h` の `XFS_SB_FEAT_INCOMPAT_LOG_ALL` 定義、`fs/xfs/xfs_mount.c` の `xfs_add_incompat_log_feature` および `xfs_clear_incompat_log_features`）、**本プロジェクトでは未検証**；判明している相違点：XFS のログは論理・物理の混合エントリを再実行するが、本プロジェクトのレコードはポインタ層の目標状態のみを担持する。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：C119 (冻结组件没有 spec 文件)（journal 記録形式は 3 番目であるが、略称行は「2 つ」と記載されたまま）、C246 (树表单元没有冻结层归属)、C199 (实例代号递增与 jsn 断号即止互相矛盾)（凍結前に清算必須）、C225 (组件的冻结时刻绕开门禁触发)。

#### 確定項目 4：1 回のパブリッシュを一括適用または一括非適用とし、第 2 の replay 実装は統語論層にとどめる

**確定内容**：「1 レコードの適用がポインタ層において何をもたらすか」という二者択一において、「1 回のパブリッシュを一括適用または一括非適用」を採用する。したがってレコード照合器における第 2 の replay 実装は統語論層にとどまる：プレフィックス判定 + ルート選択 + 継続実行であり、ツリーのセマンティクスは実装せず祖先の再生成も行わない。D13 (検証ルート) との関係は厳格化された 4 点に従う：

| # | 確定内容 | 根拠 |
|---|---|---|
| ① | **共有**：実装との間では D13 (検証ルート) 確定項目 5 の生成定数のみを共有；checker とは**原則として非共有**であり、共有には個別論証を要する；RefFS とリカバリコードを共有しない | D13 (検証ルート) 衝突 1 の妥協案は checker 自体のトラバースとスキャンの 2 方向間を律するものであり、2 つの判定器間の共有を許諾するものではない（クラウド 2 レグ一致） |
| ② | **入力パラメータを 3 つに分割**：クラッシュ状態イメージ（ルート選択とプレフィックス判定が検査）、レコードストリーム（クラッシュ状態イメージリング内の journal レコード）、実装リカバリ後のイメージ（照合対象） | リカバリ自体がディスクを変更するため（インスタンス切り替えによる行書き込み、インフライト checkpoint の再発行、D23 (ジャーナルの役割と形式) 確定項目 14）、D13 (検証ルート) 確定内容に記載の「クラッシュ後イメージ」は harness のリカバリ先行・照合後続の順序において 2 つのイメージとなる |
| ③ | **数値の欠損**：プレフィックス第 1 条が要求する「インフライトレコード数上限」は本リポジトリ内に値が存在しない（I-8.3 (再生プレフィックスの厳格連続性)；D23 (ジャーナルの役割と形式) 確定項目 1 項目 C はシステム構成への格納を要求） | 当該数値は本リポジトリ内でヒット 0 件である |
| ④ | **共謀テスト**：レコード照合器は第 4 の判定器である；各判定器はそれのみが検知可能であり可視状態や構造に有害である不良実装を検知対象として持たなければならない。「ルートスロット前のバリアの除去」は不適格——それによる欠落はレコード照合器自身のみを毀損するため；候補保留：従前の候補であったロールバック行 W の切り捨ては、管理者ロールバックのマウント中順方向パブリッシュへの変更に伴い削除（D23 (ジャーナルの役割と形式) 確定項目 14）、第 2 ラウンドの計測で切り捨ては当時のコードにおいて到達不能であることが判明。O3 (独立規約実行器) は [verification-build.md](../verification-build.md) 実装順序のステップ 6 である | 逆方向レグ 4-D |

**射程**：第 2 の replay が何を行い何を行わないかを定めるものであり、**必要とされる工数は規定しない**——コードが存在しない段階では検証可能な回答は存在せず、research 内の 3 つの作業のモデル等価物が下限代理として存在するのみである（`e42_transaction_records.rs` 240 行、`e77_publish_order.rs` 506 行、`e78_replay_start.rs` 556 行、逆方向レグ `wc -l`）、第 4 の要素（期待状態の算出）はモデルが 1 行も存在しない。ポインタ層の残余部分（部分適用許容時のポインタ切り替え先、祖先の COW 実行主体、配置先の割り当て主体）は依然として C284 (施加一条记录在指针层上做什么没有定义) に帰属する；D8 (コアインデックス構造) 確定項目 13（同一パブリッシュ内での同 key 中間状態の縮約）は本方針に依存する：適用がパブリッシュ境界においてのみ発生することで、同一パブリッシュ内の中間状態が replay される事態を防止する。

**根拠**：

- 実験なし：第 2 の実装が何を独自実装し何を共有許容とするかは検証ルートの分担であり、定量可能な数値は存在しない；「必要工数」もコード皆無の段階では検証可能な回答はなく、行数の下限代理が存在するのみである。
- 三方論証後のユーザー確定により問いを厳格化（「第 2 の実装が何を独自実装し何を共有許容とするか」および「共謀テストにおける立ち位置」を問う）、発言原文は変更履歴にある。

**未済の検査**：C284 (施加一条记录在指针层上做什么没有定义)。

#### 確定項目 5：`T_time` と `T_dirty` の設定値、消失ウィンドウの計測条件

**確定内容**：`T_time` のデフォルト値は 5 s とし、実行可能区間 [200 ms, 5 s] の上限を採用——デフォルトの損失ウィンドウは 5 秒であり、これより小さく設定しても償却効果が減少するのみで新たな安全性を購入できず、損失を阻止しているのは 5 秒という時間そのものである。**`T_time` はユーザー設定可能とし、制限を設けない**：マウント時に渡された値はディスク上のフィールドを上書きしライトバックされる；[200 ms, 5 s] を超過しても拒絶せず警告のみを出力（ユーザー 2026-09-20）。`T_dirty` = 2 GiB（初期値、コーディング時に再調整）；損失ウィンドウによって決定されるのではない——`∨` 規則下では時間側が既にウィンドウを保証しており、`T_dirty` は**量**を管理し、ダーティページメモリ予算そのものである。**`T_dirty` は調整可能値でありフォーマット定数ではない**：システム構成内のフィールドはフォーマットであるが、フィールドに格納される数値はそうではない。**マウント時に再度クランプされる**：有効 `T_dirty` = min(システム構成内フィールド値, リング長 ÷ F)；リング長は mkfs パラメータ（デフォルト 768 MiB、リング ≤ デバイス容量 ÷ 4 に制約）であるため、「リングが `F ×` 最悪消費量を収容できるか」は mkfs 時の下限によって保証するのではなく、マウント時にダーティバイト予算を抑制する方針に変更；デフォルトリング下での有効値 = min(2 GiB, 256 MiB) = **256 MiB**、システム構成内の `T_dirty` は 2 GiB のまま変更しない。

**消失ウィンドウの計測条件**：番号取得 = トランザクションが特定 checkpoint に割り当てられた瞬間；checkpoint の分割はメンバー資格を閉じるのみ——トリガー後は新規トランザクションの割り当てを停止し、割り当て済みトランザクションは各自のユニットを書き終え、パブリッシュフェーズは全ユニットのディスク永続化を待機する（いかなる COW checkpoint も最も遅い I/O を待機しなければならず、ZFS の txg sync も最も遅い vdev を待機する、本プロジェクトでは未検証）。トランザクション分割規律は 1 トランザクション 1 ユニットとする。番号取得において、トランザクション分割規律は書き込み要求をユニット単位で複数トランザクションに分割して個別に番号を取得させ、1 トランザクションは最大 1 ユニットのユーザーデータを書き込み、**同一要求から分割された複数トランザクションはそのユニットの key 昇順で番号を取得する**（コミットパスにおいて「同一要求内で番号順と key 順が不一致となった回数」を記録し、非ゼロなら赤判定）。ユニット書き込みのリトライ予算は壁時計 T_retry（名前付き調整可能パラメータでありフォーマット定数ではない；枯渇時は失敗の性質に応じてインスタンス切り替えまたは読み出し専用移行、D23 (ジャーナルの役割と形式) 確定項目 14——これは損失ウィンドウの加算項、マウント内インスタンス切り替えの判定条件、および連続切り替えカウントの時間基準を兼ねる）。**これによりクラッシュ時の最大消失量は T_time + N_switch × (T_retry + T_redo) であり、正確に 5 秒ではない**——1 回のマウントで N_switch 回のインスタンス切り替えが許容され、毎回 T_retry を待機した後にインフライト checkpoint を再実行 (T_redo) する必要があり、再実行中に開放されている checkpoint はパブリッシュできず蓄積を続ける。ここでの `T_time` は今回のマウントで有効な値であり、5 秒という定数ではない；`T_time` 調整可能化に伴い、2026-08 のユーザー確定「クラッシュ時の最大消失量は 5 秒の書き込み」は**デフォルト値下での公約**へと降格し、グローバルな公約ではなくなった。**「前回 checkpoint 時刻」は再発行時は再発行の瞬間を採用する**（元の時刻を採用すると切り替え期間中にトリガー条件が成立し続け、インフライト深度が 2 のみであるためトリガーが空転する）；インフライト checkpoint 深度 = 開放中 + パブリッシュ中の 2 個；並行 fsync の復帰時刻が同一 checkpoint の他トランザクションに牽引されるのはグループコミット固有の代償である。これは確定項目 14 の「スナップショット発行点は当該 checkpoint の全データ変更の直後に固定」の実行可能形態である。

**射程**：両閾値の数値およびクラッシュ最大損失量を規定するものであり、リング幾何構造自体や、再実行中に開放された checkpoint がライターを閉塞させるか否かは規定しない（D28 (マウント期コミットメント量) 確定項目 3 が指摘した計測条件の空白、D16 (パブリッシュセマンティクス) には未記載）。判明している 7 点のコーナーケース：

- **`T_time` の下限は非 fsync 起因のパブリッシュにのみ適用される**：導出は 1 ウィンドウ複数操作による 1 回パブリッシュの共有を前提としており、fsync 集中負荷下ではウィンドウは fsync 自体によりトリガーされる。本下限を引用する際は本但し書きを併記すること；上限 5 s（ユーザー確定の損失ウィンドウ）は影響を受けない。
- **「200 ms への切り上げ」および「予算 0.1 の採用」はいずれも選択値であり計算値ではない**：0.2 では 94 ms、0.05 では 410 ms（外挿）となり、下限は予算に伴い線形移動するため、予算改定時は再計算を要する。
- **`T_dirty` から導出される 2 つの派生値は確定レコード形式に従い算出**：D23 (ジャーナルの役割と形式) 確定項目 12 の 4 KiB 固定長レコード（ヘッダ 95 B、56 B/項目、71 項目/レコード）下において、同一 `T_dirty` の満杯ウィンドウは 1847 レコードであり、消費は **7388 KiB**；I-8.1 (リング幾何構造の十分性) のリング下限は F = 2 で **14,776 KiB**、F = 3 で **22,164 KiB**。両閾値の確定値は不変であり、変化したのはこれら 2 つの派生値の計測条件のみである。
- **1 トランザクション 1 ユニット下のレコード会計と本派生値の衝突**：6 つの負荷点で算出すると、fsync ごとのレコード数はデータユニット数に等しく（seq バッチ 1 は 8 件、バッチ 10 は 80 件）、journal は常にユーザーデータの 12.5% を占有し、`T_dirty` 満杯ウィンドウは 65,536 レコード、256 MiB となり、バイト表が想定した 64 MiB リングの 4 倍に達する。2 つの計測条件が並立しており、改定方針はユーザー決定に委ねられる。
- **ローカルハードウェアにおける持続書き込み帯域は未実測**（E58 (チェックサム粒度のエンドツーエンドコスト) の 2045 MiB/s はチェックサム実験における順序計測条件であり、書き込み帯域ではないため換算禁止）、したがって「5 秒間に何バイト書き込み可能か」は算出不能である。
- **`T_dirty` と `T_time` は異なる規則に従う**：`T_dirty` はクランプに従う——有効値 = min(フィールド値, リング長 ÷ F)、変更なし；`T_time` は警告に従う——逸脱しても拒絶せず警告のみ。同一段落内の 2 フィールドが異なる規則に従うため、同一規則として解釈してはならない。
- **`T_time` デフォルト値は後期にディスク種別に応じた推奨値を提示**（ユーザー 2026-09-20）：ディスク群の実測およびプローブ入力が必要であり、D12 (対象メディア) の路線（第 1 のレイアウト系統は純 SSD）に位置づけられる。`T_time` デフォルト値の再検討条件として記録し、後期作業に留保。

**根拠**：

- E70 (checkpoint の二つの閾値の実現可能領域)：両閾値の実現可能領域——`T_dirty` の上限はリング幾何構造に結合するが荷重は受けず（64 MiB リングで 16 GiB のダーティデータを許容、実際に制約するのはダーティメモリとクラッシュ損失ウィンドウ）、`T_time` の下限は償却予算から逆算され、主要負荷 seq ではなく最悪負荷 multistream に基づき決定、償却予算 `B = 0.1` ブロック/操作 ⇒ 逆算 547 回操作 ⇒ 197 ms、切り上げて 200 ms、seq 負荷では下限は一切不要。
- E44 (シーケンス番号ビット幅のハードウェア実測と代償)：ローカル 2785 fsync/秒、償却予算を `T_time` 下限へと逆算した際のレート。
- E91 (確定フォーマット下でのリング会計と受入会計)：確定レコード形式に従い `T_dirty` 満杯ウィンドウの journal 消費とリング下限を再計算、旧 32 B/エントリの計測条件は 1.80 倍過小評価していた。
- ユーザーによる 3 回の確定（両閾値、損失ウィンドウ計測条件の収束、マウント時再クランプ）、発言原文は変更履歴にある。

**未済の検査**：C310 (事务切分纪律与记录数口径打架)、C425 (T_time 小于一次发布耗时时触发空转)。

#### 確定項目 6：パブリッシュカウンタ、4 つの等号を 1 つに統合

**確定内容**：パブリッシュごとに checkpoint_txg を 1 加算する——fsync 起因のパブリッシュもパブリッシュであり、「軽微なパブリッシュには番号を付与しない」等の例外は設けない；ルートスロットのローテーションキー（D22 (ユニット原子性の合成方法) 確定項目 2「領域 = txg mod R」における txg）、会計の世代（D5 (スナップショット・領域会計メカニズム) 確定項目 2）、リカバリ再生の水位（D23 (ジャーナルの役割と形式) 確定項目 14）はすべて同一のカウンタである。別途「パブリッシュ代番号」フィールドは設けない——checkpoint_txg と同一物であるためである。**4 つの等号**：fsync = 前倒しパブリッシュ、領域 = txg mod R、1 txg につき 1 番号、会計世代 = checkpoint 番号；4 箇所が同一の数値を参照しており、いずれかの実装が参照を誤ればルートスロットの衝突や会計の誤読を引き起こす。

**4 つの付帯条項**（確定と同一日に採択、選択不可）：

1. **ルートレコードは 32 ビットのインスタンス代番号を必須保持とする**（D23 (ジャーナルの役割と形式) 確定項目 9 の journal インスタンス代番号と同一のカウンタ）；checkpoint_txg が同値の場合、インスタンス代番号が大きい方を勝者とする。ルートリングを読む際は各ルートのスロットを配置先公式（D22 (ユニット原子性の合成方法) 確定項目 16）で照合し、第 0 世代ルートは mkfs がルートを置くスロットで別に認める；1 つの txg には配置先スロットが 1 つしかないので、同値は同一バイトの第 0 世代ルート同士にしか残らず、ルートリングの読み出し順で最初のものを取る（D22 (ユニット原子性の合成方法) 確定項目 24）；公式照合はまだ実装されておらず、C743 (再構築アロケータが新クラス記録時にルートが当該スロットに存在するか検証しない) に未済として記録。フィールド配置先は D22 (ユニット原子性の合成方法) 確定項目 7。
2. **会計 key の「世代」セグメント幅 ≥ 48 ビット**：ローカル 2785 パブリッシュ/秒において 32 ビットは 17.8 日で周回。会計 key の幅算術は仮定であると自己申告しており、これに基づき再計算を要する。
3. **`T_time` 下限の導出（197 ms）は非 fsync 起因のパブリッシュにのみ適用される**——導出は 1 ウィンドウ複数操作による 1 回パブリッシュの共有を前提としており、fsync 集中負荷下ではウィンドウは fsync 自体によりトリガーされる。確定項目 5 の下限を引用する際は本但し書きを併記すること；上限 5 s（ユーザー確定の損失ウィンドウ）は影響を受けない。
4. **会計エントリのパブリッシュ間合流は実現不能**（世代が key に含まれるため、「後着優先」では異なる世代の 2 行を合流できない）⇒ パブリッシュごとに当該世代の行をすべて書き出すのが実コストであり、上限ではない。コストとして忠実に計上する。

**射程**：本カウンタの加算方法および 4 箇所での共有方法を定めるものであり、ルートリング幾何構造自体（D22 (ユニット原子性の合成方法) 確定項目 2）や journal インスタンス代番号のインクリメント契機（D23 (ジャーナルの役割と形式) 確定項目 9）は規定しない。判明しているコーナーケース：本確定当日の対抗テストは実質的に 2 レグで実行された——ローカルレグの報告書は後に厳格化された単語破損検査で赤判定（`proposed proposed`、実単語自己反復分類）となり、`.claude/rules/three-way-inference.md`「ゲート赤判定後のラウンドは無効」に基づき、**「三者合意」として引用することを禁止する**；確定内容自体はこれにより覆ることはなく、両クラウドレグ各自の判定成果物は有効である。

**根拠**：

- E48 (ルートリング幾何構造の実現可能点)：ローテーションキーは txg と呼ばれパブリッシュごとに単調増加し、9 つの実現可能点すべてがこの解釈に基づき構築されている。
- E50 (ルートリングスロット数の上下限)：ソースコード原文ママ「1 周時間（秒）= 総スロット数 ÷ fsync レート。fsync ごとに 1 回ルートを発行」——リポジトリ内のルートリング算術は「パブリッシュごとに checkpoint_txg を 1 加算」の等式下でのみ真であり、これを固定しなければ実測値は遡及的に無効化される。
- E53 (ディスク 1 台喪失後のルートリングマウント可否)：ディスク喪失ロールバック + ディスク再認識の履歴こそが、付帯条項 1（ルートレコードへのインスタンス代番号保持）が識別すべき対象である。
- E44 (シーケンス番号ビット幅のハードウェア実測と代償)：ローカル 2785 パブリッシュ/秒において 32 ビットは 17.8 日で周回——付帯条項 2 の「≥ 48 ビット」はこのレートに由来する。
- 対抗テスト（ユーザー要求に基づき攻撃先行で確定）の元出力 `c85-publication-counter-forward レグ報告書 (成果物は git 内)`、`c85-publication-counter-reverse レグ報告書 (成果物は git 内)`、`c85-publication-counter-local.out`；ユーザー確定、発言原文は変更履歴にある。
- 付帯条項 1 の同値（ルートリング読み出しでスロットを配置先公式で照合、第 0 世代ルートは別に認める、同値は同一バイトの第 0 世代ルート同士にしか残らない）：ルートリング三方第 3 ラウンド判定 `research/prompts/m3-close-rootring-r3-main-verification.md` 第 2 節 C743 (再構築アロケータが新クラス記録時にルートが当該スロットに存在するか検証しない) のセル（D22 (ユニット原子性の合成方法) 確定項目 7 のインスタンス代番号の行は付帯条項 1 と同源で、一緒に変更；第 0 世代の例外は第 1 ラウンドが裸の公式を突いた後に補ったもので、第 2 ラウンドは攻撃したがヒットせず、第 3 ラウンドで防御側が維持）。

**未済の検査**：C88 (ルートリングのタイムライン判別未実装)（付帯条項 1 の checker 側、逆方向レグで発見された到達可能な不備；C85 (发布计数器的四个等号没写死) から統合、条文は 1 つにまとめられたが、赤判定を出すべき検査は未済のまま）；C743 (再構築アロケータが新クラス記録時にルートが当該スロットに存在するか検証しない)：ルートリング読み出し時の配置先公式照合。

#### 確定項目 7：パブリッシュの永続順序

**確定内容**：1 回のパブリッシュの永続順序は常に COW ユニット / ノード → バリア → journal レコード → バリア → ルートスロット (FUA) → システム構成スロット → バリア；fsync はシステム構成ローテーションの永続化を待機して初めて復帰する。リカバリ再生はいかなるレコードの適用前にも、指定ユニットのチェックサムを個別に検証しなければならない。**システム構成スロットはルートスロット永続化後に更新され、checkpoint ごとに 1 回実行**、これはパブリッシュシーケンスの最終ステップでありクラッシュポイント再生の列挙対象となるため、パブリッシュのセグメントシーケンスは一意に定まる。**F 引き上げシーケンス（受入およびアンマウント）は初回パブリッシュ前に 1 ステップ追加される**：まず新 F を各ディスクのシステム構成に書き込みバリアを通過させる（確定項目 1「F 引き上げシーケンス」）；このシステム構成書き込みはいかなる単一パブリッシュにも属さず、パブリッシュ内部の順序は従来どおり。**fsync 復帰条件に 1 句追加**：新インスタンスにおいて本インスタンスが書き込んだルートが 2 台のディスクを網羅するまで、fsync は復帰しない（手順および回数は確定項目 8）；管理者ロールバックはマウント中の一連の順方向パブリッシュであり（D23 (ジャーナルの役割と形式) 確定項目 14）、そのルートが永続化した後に初めて管理者へ確認を返し、fsync と同一の復帰条件を共有する；実装はディスパッチ保留中、コード監査 A3c の後に配置（同一 `transaction.rs` 内）、コストは別途小実験で登録・計測。

**射程**：1 回のパブリッシュ内部のステップ順序および fsync 復帰契機を定めるものであり、ハードウェア上のバリア実コストやルートスロット自体の冗長形態（D22 (ユニット原子性の合成方法) 確定項目 2）は規定しない。判明している 3 点のコーナーケース：

- **「fsync はシステム構成ローテーション永続化を待機して復帰」の射程**：復帰後、当該世代のルートは単一のルートスロット（ルートスロットは非ミラー）によってのみ保護され、それを目撃したシステム構成はディスク永続化済みとなる。同一インスタンス内での 1 世代後退は journal 再生により追従可能；**新インスタンス（書き込み可能マウント、リカバリ、切り替え）の初回ルートは次回パブリッシュまで単一障害点となる**——当該スロットが読み出し不能となるかディスクが喪失した場合、リカバリは前インスタンスのルートへ後退し、再生は厳格プレフィックスに基づきインスタンス境界で停止し（D23 (ジャーナルの役割と形式) 確定項目 14 注 1）、今回のマウントで fsync 復帰済みのトランザクションが消失し、単一障害で十分破損する。新インスタンスが確認前にまず 2 ディスクへルートを書き込むべきか否かは確定項目 8 にある。
- **バリアの計測条件**：2 回の FLUSH + ルートスロット FUA + システム構成ローテーション後のバリア 1 回 = パブリッシュごとに 4 つの順序点であり、D25 (目標負荷優先順位) の導出による 2 点より 2 点多い——増加した 2 点は第 2 のバリアおよび「パブリッシュ復帰前のバリア追加」のコストであり合意済み（後者はユーザー 2026-09-27 確定、起因は C577 (系统配置没见证到的最新根，乙罩不到) のテストケースが F 引き上げ前に早期赤判定となったこと：パブリッシュパスがルートスロット FUA 後にシステム構成をローテーションして復帰しバリアを欠いていたため）。
- **第 2 のバリアが購入するのはレコードストリームの完全性**であり、データの完全性ではない：データ完全性に必須のバリアはちょうど 1 回（ルートスロット前）；第 2 のバリアを省略すると「ルートは存在するがレコードが欠落」という状態が 7 分類生じ、レコード照合器および逆方向チェーンの入力にホールが生じる。ユーザーは「必要」と確定し、双方を配備。

**根拠**：

- E77 (パブリッシュの永続順序)：クラッシュサブセット全数探索——ルートスロット前のバリアを欠落させると 1024 のクラッシュ状態中 504 状態で違反；「再生適用前の指定ユニット検証」を省略すると b_rs アームで 63 箇所のサイレントな誤結合が発生、ゆえに荷重を受ける重要ステップである。
- ユーザーによる確定、発言原文は変更履歴にある；「パブリッシュ復帰前のバリア追加」（システム構成ローテーション後のバリア、順序点を 3 から 4 へ増加）はユーザー 2026-09-27 確定、起因、プロンプト事実、および分岐は `c554-fix-forks.md` 4 行目にあり、実装はマージ済み（`crates/singlefs-core/src/transaction.rs` の `persist_the_root_then_rotate_the_system_configuration`、実装報告 C577 (系统配置没见证到的最新根，乙罩不到) `m2-impl-c577-barrier-implementer-report.md`）、コストは別途小実験で登録・計測。

**未済の検査**：C26 (バリア数が未計測)（実機での fsync ごとのバリア数は依然未計測）。クラッシュポイント再生 harness の 2 つの自己証明テストケース（ルートスロット前バリア除去で赤必須、再生検証無効化で赤必須）は C76 (发布时序没有权威落点) に伴い清算済み。

#### 確定項目 8：ウォームアップは戊を採用、新インスタンスのルートを 2 台のディスクに先行配置

**確定内容**：新インスタンスにおいて本インスタンスが書き込み（FUA 復帰）を完了したルートが 2 台のディスクを網羅するまで、いかなる fsync の復帰も許可しない；手法としては空パブリッシュを連続実行し、第 1 版の幾何構造では最大 R = 3 回とする。**第 1 版の実コストは 2 回であり、これは実行時述語ではなくフォーマット定数である**：ルートリングの 3 領域のデバイス帰属は第 1 版において 0 / 1 / 0 とハードコードされており（D2 (RAID 条带策略) 確定項目 7）、したがって txg 1 は領域 1（ディスク 1）、txg 2 は領域 2（ディスク 0）に配置され、2 回でちょうど 2 台のディスクを網羅する ⇒ 回数は帰属により一意に定まる。`format-const` WARM_UP_EMPTY_PUBLISHES = 2 および NEW_POOL_FILE_CREATION_TXG = 3（[layout/01-first-txn.md](../layout/01-first-txn.md) VIII）として登録、条文は D22 (ユニット原子性の合成方法) 確定項目 16 第 5 文。**後続マウントおよびマウント途中クラッシュ後の再マウント**：回数は随時算出——行書き込みパブリッシュの txg は D23 (ジャーナルの役割と形式) 確定項目 14 注 3 の下限値そのもの（max(リング内全ルートレコード txg, リング内全自己証明通過レコード checkpoint_txg) + 1）を採用し、以降空パブリッシュごとに 1 加算し、ルートリング配置先に基づきディスクを判定し、本インスタンスのルートが全ディスクを網羅するまでプッシュ、最大 R 回；番号取得前の受入と取得後のパブリッシュプッシュは同一スケジュールを使用（`crates/singlefs-core/src/mount.rs` の `warm_up_publish_txgs`）。

**射程**：新インスタンス確認前のアクションを規定するものであり、インスタンス切り替え自体の判定条件（D23 (ジャーナルの役割と形式) 確定項目 14）や空パブリッシュが書き出すユニット数（確定項目 9）は規定しない。判明している 4 点のコーナーケース：

- **前提 C314 (崩溃恢复落到旧根后可以复用被抛弃的根引用的单元) の条文側は充足済み**（ユーザーがシャドウ会計を採用、条文は D23 (ジャーナルの役割と形式) 確定項目 14 および I-7.4 (直近 K 代ブロック非再利用) に記述）、ウォームアップは有効化済み；**検査側は未済**：複数回マウントの記録ストリームは整備済み（クラッシュポイント再生上書き固定スクリプトストリーム）、逐文監査においてスロット 1 の専用アサーションを欠いており、C314 (崩溃恢复落到旧根后可以复用被抛弃的根引用的单元) は依然として未清算表に残存。
- **帰属変更時はこれら 2 つのフォーマット定数を再計算**（ディスク追加またはストライプテーブル開放時）；「最大 R = 3 回」は幾何学的な上限であり、第 1 版で到達する回数ではない。第 1 版の実コスト 2 回は三方論証を経ておらず、変更の余地がある。
- **コスト**：確定項目 7 の fsync 復帰条件を改定（新インスタンス確認前に条件を 1 つ追加）；新規プールでのファイル作成の checkpoint_txg が 1 から 3 へ変更、空レコード 2 本とルート書き込み 2 回が増加；D28 (マウント期コミットメント量) 確定項目 3 のスイッチ予約に「切り替えごとに最大 R = 3 回の空パブリッシュ」を追加、D28 (マウント期コミットメント量) 確定項目 4 で算出された c_max に従い取得（プール規模 9 ブロック ⇒ 27 ブロック）；空パブリッシュがルートリングスロットを占有し、旧い非空ルートを早期にリング外へ追出す。
- **他アーム**：甲（補填なし）は判定基準 2 で 1 障害により破損；丁（インスタンス境界を跨ぐ再生）は除外。
- **無駄プッシュ**：行書き込み txg ≡ 2 (mod 3) の時、初回の空パブリッシュは既に網羅済みのディスク 0 に配置され無駄プッシュが 1 回発生——装置複製上で毎回 147,968 バイト余分に書き込まれ、バリア 2 回、FUA 1 回、ルートリングスロット 1 個が余分に消費されることを計測、再オープンのみで書き込みを行わない場合はマウントごとに本階層に該当し、24 スロットのルートリングは 8 回のマウントで 1 周完結（リポジトリ内装置では未再試行）。行書き込み txg が下限値をそのまま採用したことの直接の帰結であり、受容する；「通常マウントの行書き込み txg を 1 回プッシュで済む値へ後退させる」修正案は攻撃ゼロ回であり不採用。

**根拠**：

- E150 (ロールバックによる破棄ルート参照ユニットの再利用)：ウォームアップなしでは 1 障害で破棄ルートにフォールバック、ウォームアップ後は 4 障害を要する；シャドウ会計と墓石の双方が違反ゼロ、ユーザーはシャドウ会計を選択。
- E148 (コミット固定点の二つの記録ツリーによる再計算)：空パブリッシュはプール規模・コミットブロッククラスタリング時 9 ブロック、スイッチ予約側はこれに基づき算定。
- E142 (新規プールでのファイル作成のドライラン)：checkpoint_txg 1 → 3 の変更を新規プールでのファイル作成バイト列に適用後、262,162 状態で違反ゼロ。
- 2 ラウンドの三方論証 `research/prompts/c199-r2-main-verification.md`；ユーザー確定、発言原文は変更履歴にある。
- 後続マウントの規定および無駄プッシュ：三方第 1 ラウンド `research/prompts/m2-presumed-clauses-r1-main-verification.md` K2（網羅までのプッシュは後続マウント、ロールバック、マウント途中クラッシュ再マウントの 3 種の起点において不足を構成不能）；ユーザーが 2026-09-23 に当時の実装に従い記述し無駄プッシュを受容すると決定、発言原文は変更履歴にある。

**未済の検査**：C314 (崩溃恢复落到旧根后可以复用被抛弃的根引用的单元)（検査側）。

#### 確定項目 9：空パブリッシュもユニットを書き出す

**確定内容**：会計ツリーが存在する場合は書き出す——空パブリッシュもパブリッシュであり、D5 (スナップショット・領域会計メカニズム) 確定項目 2（各ルートの会計ツリーはその世代の行のみを格納し、パブリッシュごとに当該世代の行をすべて書き直す）に従い会計行、連動する会計ツリーノード、割り当てレコード、マッピングエントリ、およびツリーテーブルユニットを書き直す。初回書き込み可能マウントのウォームアップ時はツリーテーブル 0 件 ⇒ ゼロユニット；以降の空パブリッシュは D28 (マウント期コミットメント量) 確定項目 4 で算出された c_max ブロックを消費。**ツリーテーブル 0 件版で行書き込みを行うパブリッシュは 2 ユニットを書き出す**：インスタンス表と 1 セグメントの割り当てレコードノードであり、割り当てレコードツリーのルートポインタはルートレコードに格納（D22 (ユニット原子性の合成方法) 確定項目 7）；置換された前版のインスタンス表と前セグメントの割り当てレコードノードは解放済みとして記録され、再オープン時はこのノードから当該版の会計を再構築する。ルートレコードの当該項目はツリーテーブル 0 件で行書き込みを行った版のみ非ゼロを書き、mkfs の第 0 代およびファイルを含む版は全ゼロを書き、ゼロユニットパブリッシュは前版をそのままコピーする。

**射程**：空パブリッシュがユニットを書き出すか否か、および何を書くかを定めるものであり、c_max の算出方法（D28 (マウント期コミットメント量) 確定項目 4 に従い当時の構造に基づき随時算出）や空パブリッシュのプッシュ契機（F 引き上げは確定項目 1、ウォームアップは確定項目 8）は規定しない。行書き込みパブリッシュが書き出す割り当てレコードノードはユニットヘッダのツリー ID に 0 を書き込む（実装の選択：当該版のツリー ID 水位は mkfs の 65536 であり、割り当てレコードツリーの 65538 を書くと I-7.8 (ルートレコードツリー ID 水位は全プール最大ツリー ID 以上) が直ちに赤となるため）、条文は未規定；中央マッピングに入るか否かも条文未規定——当該版にはマッピングツリーが存在せず、D19 (ブロックポインタの構造と幅予算) 確定項目 8 のブートストラップ免除 3 分類に該当しないため。

**根拠**：

- E148 (コミット固定点の二つの記録ツリーによる再計算)：空パブリッシュの固定点を 2 つの記録ツリーにより再計算するとプール規模 9 ブロック、新規プールでのファイル作成規模 4 ブロックであり、「以降の空パブリッシュは実測 c_max ブロック」の数値はここに由来する。
- E142 (新規プールでのファイル作成のドライラン)：初回書き込み可能マウントのウォームアップにおける 2 回の空パブリッシュをバイト単位で書き出し、ツリーテーブル 0 件時にゼロユニットとなることを確認。
- ユーザー確定 2026-09-23（ツリーテーブル 0 件版で行書き込みパブリッシュはインスタンス表と割り当てレコードノードの 2 ユニットを書き出し、割り当てレコードツリーのルートポインタはルートレコードに格納）、発言原文は変更履歴にある；実装およびテストケース指定は [checks-owed.md](../checks-owed.md) 清算済み表 C512 (树表 0 条的一版上被换下的实例表记在哪没有条款) 行にある。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 10：パブリッシュ粒度はバッチ化を採用し、トランザクションごとの発行は行わない

**確定内容**：ダーティノードを一定数バッチ化（または時間到達）した段階でルートを発行し、トランザクションごとの発行は行わない。

**射程**：頻度を定めるものであり、パブリッシュイベント内部の順序は規定しない——それは `litmus/commit-publish.litmus` にあり、ブロック内容の書き込み → `smp_wmb` → generation の書き込みを規定するもので、頻度には一切言及しない。判明している 2 点のコーナーケース：

- **正味コストの分布**：フォーマット側はほぼゼロ（journal は元より必須であるため）、検証側は多大——中間状態セマンティクスの改定（確定項目 12）、第 2 の replay 実装（確定項目 4 および 13）、さらに D13 (検証ルート) の「Merkle ルートチェーン差分」粒度の粗大化：1 世代のルートに K 個のトランザクションの変更が詰め込まれ、特定精度が「どのトランザクションか」から「どの checkpoint か」へと劣化し、D13 (検証ルート) はフル実行層の真のコストは特定コストであると明記している。
- **パブリッシュセマンティクスが進捗回数を決定**：トランザクションごとのルート発行 ⇒ K バッチで K 回実行；checkpoint ⇒ 同一ウィンドウ内の K バッチで概ね 1 回実行、各バッチは 1 本の journal レコード（小容量、シーケンシャル書き込み）を残すのみ ⇒ バッチ大規模削除セルにおけるメタデータ書き込み増幅は `K × M × H` から `checkpoint 数 × M × H` へと減少。

**根拠**：

- 実験なし：確定内容は相互に独立した 2 つの推論に基づき、当時定量可能な数値は存在しなかった。① D5 (スナップショット・領域会計メカニズム) で確定済みの ZFS birth txg モデルと自然に整合する——ZFS の txg 自体が checkpoint であり（`zfs_txg_timeout` デフォルト 5 秒、または一定のダーティデータ量蓄積）、1 txg で大量操作をバッチ実行し txg 終了時に uberblock を 1 回更新；checkpoint 採用に伴い birth txg の番号粒度の問題は自然解消し、1 txg 1 番号となり、deadlist の比較粒度は checkpoint 粒度となり再計算不要（本プロジェクトでは未検証）。② **ルートは書き込み消失の最終的目撃者である**：「ディスク上のこのブロックが想定されるバージョンであるか」の検知には外部の目撃者が必要であり、ブロック自体では不可能（正当な旧バージョンと正当な新バージョンはブロック内レベルで同型、D18 (ブロックに含める情報) に完全な論証あり）、そして目撃者は異なる障害ドメインに存在しなければならない——親ノードが同一イベントで書き込み失敗した場合は祖父ノードを要し、ルートへと再帰する ⇒ checkpoint が発行するルートはこの再帰の終点であり、その永続化はいかなる単一ブロックよりも高信頼でなければならない（ZFS では 4 つの label × 各 128 KiB の uberblock リングを用い偶数・奇数 label を 2 フェーズで書き込む；本プロジェクトのルート自体の冗長形態は D22 (ユニット原子性の合成方法) 確定項目 2 が規定）。頻度の定量的コスト（両閾値の実現可能領域）は確定項目 5 にある。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 11：いかなる操作も journal を迂回して直接ルートを発行することはできない

**確定内容**：すべての構造・すべての操作が同一の journal を経由し、同一の checkpoint リズムを共有する。fsync の効果は「パブリッシュを前倒しでトリガーする」ことであり、**別のログ形式や別系統のリカバリロジックを開設することではない**。判定基準：journal がすべてのコミットの必須経路であるか、あるいは特定パスのバイパスであるか——前者は統一機構かつ永続化の 2 段階化であり正当；fsync のみ（あるいはメタデータのみ、小容量書き込みのみ）が journal を経由し他が直接ルートを発行する形態は、`.claude/rules/fs-design.md` が指摘する「fsync のために専用のログツリーを開設する」に該当する。

**射程**：マウント後の操作を対象とする；mkfs シードによる第 0 代ルートの書き込みは journal を経由しない（D13 (検証ルート) 確定項目 1 が列挙するルートスロットの 3 種の書き込み主体の一つであり、mkfs はレコードを 1 件も書き込まない）、本規定の射程外。判明しているコーナーケース：D23 (ジャーナルの役割と形式) 確定項目 1 の第 1 軸 Option A は本条項に立脚しており、それを再開する際の論拠に本条項が含まれる。

**根拠**：

- 実験なし：`.claude/rules/fs-design.md`「単一のトランザクション層、全構造で共用」のパブリッシュスロットにおける具現化であり、機構の形態（必須経路かバイパスか）を判定するものであって、定量可能な数値ではない；厳格に固定しなければ「特定パスのための第 2 トランザクション機構」へと形骸化する。永続化ごとの書き込み量は別の問題であり、D23 (ジャーナルの役割と形式) 確定項目 1 に帰属。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 12：中間状態の正当性セマンティクスを「ルート + journal の任意プレフィックス」に改定

**確定内容**：D8 (コアインデックス構造) の「中間状態はすべて正当なファイルシステム状態でなければならない」を「**ルート + journal の任意プレフィックス、replay 後に正当**」へと改定する。

**射程**：正当性の判定方法を定めるものであり、プレフィックス自体の切り出し方法は規定しない——プレフィックス判定の完全な計測条件は D23 (ジャーナルの役割と形式) 確定項目 14 の 5 条であり、一つも欠かすことはできない。判明しているコーナーケース：ブロック再利用側の真の制約は checkpoint 境界よりも厳格であり、journal 位置に基づき規定される（確定項目 15 および D22 (ユニット原子性の合成方法) 確定項目 4）。

**根拠**：

- E42 (1 トランザクションあたりのレコード数)：1601 個のプレフィックス中 1400 個がプレフィックスによって分断（87.4%）、1 トランザクション 1 レコードのアームでは不正プレフィックスは常に 0——これは「任意プレフィックス replay 後に正当」の直接の具現化である；複数レコードを跨ぎ境界フィールドが存在しない場合は不正プレフィックスは分断されたプレフィックス数に正確に一致し、境界フィールド追加後は 0 となる。このセマンティクスはモデル上で計数可能であり、いつ破綻するかも算出可能である。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：C42 (残留レコードの正当プレフィックス偽装)（前タイムラインがリング内に残したレコードが次回復旧時に正当プレフィックスとして再生される問題）。

#### 確定項目 13：本 journal replay はレコード照合器に帰属し、O2 (独立パーサ + checker) には帰属しない

**確定内容**：D13 (検証ルート) は O2 (独立パーサ + checker) を単項述語として規定し単一イメージのみを判定するため、本 journal replay は「レコード照合器」に帰属する。これは replay の第 2 の実装であり、replay こそがクラッシュセマンティクスにおいて**共通の誤解**を生じさせやすい箇所である——D13 (検証ルート) の「三 oracle 共謀テスト」が防止すべきはまさにこれである。**これは D16 (パブリッシュセマンティクス) において最も高コストな項目である。**

**射程**：帰属先を規定するものであり、何を実行し何を共有許容とするかは確定項目 4 にある。判明しているコーナーケース：D23 (ジャーナルの役割と形式) の第 1 軸で Option A を採択した後は「redo は空」となり、最大コストの主要部分は解消され、プレフィックス判定 + ルート選択 + 継続実行のみが残る。⚠️ **「redo は空」は「ルートが永続化済み」のパブリッシュに対してのみ成立する**：E42 (1 トランザクションあたりのレコード数) の実測では 1601 プレフィックス中 1400 プレフィックスが分断（87.4%）されており、分断される割合は境界フィールドの有無とは無関係である——変化するのはこれら 1400 個が不正か正当かである（境界フィールドなしアームは 1400 不正、境界フィールドありアームは 0）⇒ **リカバリ期間中の非空再生は通常状態でありコーナーケースではない。**

**根拠**：

- 実験なし：いずれの判定器が replay を実装するかは検証ルートの役割分担であり、定量可能な数値は存在しない；第 2 の実装のコスト規模および必須の独立実装内容は確定項目 4 にある。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 14：スナップショット発行点は checkpoint 境界に整列し全データ変更の直後に固定

**確定内容**：スナップショット発行点は checkpoint 境界に整列しなければならず、かつその位置は「本 checkpoint の全データ変更の直後」に固定され、checkpoint の中途に出現してはならない。同一 checkpoint 内で 2 つのスナップショットを作成することは許容される：同一状態を捕捉し、その間の deadlist は空となる；ただし checker 内の「直前のスナップショット」は `(txg, スナップショット id)` でタイブレークを行う。

**射程**：スナップショット発行点の配置位置を規定するものであり、スナップショット自体の会計モデル（D5 (スナップショット・領域会計メカニズム) および D6 (スナップショット実現モデル)）は規定しない。判明しているコーナーケース：本規則の実行可能形態は確定項目 5 の損失ウィンドウ計測条件にある——パブリッシュフェーズは全ユニットのディスク永続化を待機し、スナップショット発行点はその直後に配置される。

**根拠**：

- 実験なし：機構的な理由によるものであり、定量可能な数値は存在しない——`birth` の解像度は checkpoint 単位にとどまり、**ブロック自身は「自分が本 checkpoint 内のスナップショット発行点の前と後のいずれで誕生したか」を表現できない**；これを厳格に固定しなければ、D5 (スナップショット・領域会計メカニズム) の境界表において `birth == S.txg` および `death == S.txg` の 2 行が同時に確定的回答を失い、一方は**データ損失**、他方は**空間リーク**を引き起こす。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 15：checkpoint C で発生した解放は、C が発行されるまで割り当て可能集合に復帰しない

**確定内容**：checkpoint C 内で発生したあらゆる解放は、C が発行されるまで割り当て可能集合への復帰を禁止する。理由は 2 つあり、第 2 の理由は D16 (パブリッシュセマンティクス) 固有かつより厳格な制約である：

1. **ロールバック**：クラッシュ後に C−1 のルートへ後退する可能性があり、C 以前に誕生し C で死亡したブロックはその時点で依然として生存しているため。
2. **確定項目 12 が D8 (コアインデックス構造) の中間状態セマンティクスを「ルート + journal の任意プレフィックス、replay 後に正当」に改定したため**：あるブロックが journal 位置 p で解放され、位置 q > p で再割り当てされ新内容が書き込まれた場合、`p < r < q` のプレフィックスを再生すると当該アドレスを指すツリーが得られるが、**ディスク上のバイト列は既に q の内容となっている**（物理書き込みは journal 位置に拘束されないため）。**これは理論上のリスクではなく必然的な誤りである。**

**射程**：「いつ割り当て可能集合へ復帰できるか」という下限を定めるものであり、ウィンドウの大きさは規定しない（確定項目 1）。判明している 3 点のコーナーケース：

- **第 2 の理由は journal 位置に基づき規定されている** ⇒ 真の制約は journal 再生ウィンドウ全体であり、checkpoint 境界ではない；D22 (ユニット原子性の合成方法) 確定項目 4 はその形態に収束しており、これを欠いた場合の障害モードは「検知可能（親ポインタチェックサム不一致）、復旧不能」となる。
- **ウィンドウ容量は ENOSPC に直結する**（D3 (領域割り当て)：領域解放という操作自体が領域確保を要求してはならない）、そしてバッチ化により「作成して即削除」が同一ウィンドウ内で完結する確率が大幅に上昇する ⇒ **defer ウィンドウはホットパス上に位置する**。
- **外部の手がかり**：ZFS に同型の機構が存在し、より保守的である、`include/sys/txg.h` の `#define TXG_DEFER_SIZE 2 /* Number of txgs worth of frees we defer adding to in-core spacemaps */`、**本プロジェクトでは未検証**；⚠️ その 2 がなぜ 2 であるかの一次資料による説明は見当たらず、本プロジェクトの defer ウィンドウは自前の再生セマンティクスに基づき演繹しなければならず、**模倣を禁止する**。

**根拠**：

- E82 (受入のインフライト合成)：ウィンドウ内で発生した解放を利用可能と記録したアームでは、20 セルにおいて defer が保留すべきブロックを取り込んでしまった——本規則が暗黙に迂回された際の現象は計数可能であり、受入側の形態は D28 (マウント期コミットメント量) 確定項目 2 に反映された。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし（ブロック再利用側の赤判定検査は I-7.4 (直近 K 代ブロック非再利用) が担持、未済は確定項目 1 に列挙された各項目）。

#### 確定項目 16：checkpoint とメッセージバッファは同一の効用を購入しており、E7 (オフラインインデックス harness) の対照群を固定

**確定内容**：checkpoint と D11 (インデックスノード内のメッセージバッファ要否) のメッセージバッファは同一の効用を購入しており、相違は**償却ウィンドウがどこに存在するか**のみである：checkpoint のウィンドウは**メモリ**に存在しフォーマット外；メッセージバッファのウィンドウは**ディスク**に存在しフォーマット内。⇒ **D11 (インデックスノード内のメッセージバッファ要否) の限界便益 = 「checkpoint 跨ぎの償却」引く「checkpoint 内の償却」**。**E7 (オフラインインデックス harness) の対照群は必ず「checkpoint あり、メッセージバッファなし」でなければならない。**

**射程**：対照群の設定方法を規定するものであり、D11 (インデックスノード内のメッセージバッファ要否) がいずれを選択するかは規定しない——それは D11 (インデックスノード内のメッセージバッファ要否) 自身が決定する。判明しているコーナーケース：BetrFS の数値においてこれら 2 要素は分離されたことがない（4 MiB ノードのメッセージバッファ**と** 5 秒の checkpoint を併有）、**本プロジェクトでは未検証**。

**根拠**：

- 実験なし：これは「対照群をいかに設定するか」という規定であり実験の結論ではない；先行文献において write buffer フロントエンドを伴う対照群は存在せず、本プロジェクトの現行方針に対するメッセージバッファの限界便益について、既存文献は数値を一切提示しておらず、本プロジェクトにも未だ引用可能な定量的数値は存在しない。
- ユーザーによる確定、発言原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 17：fsync エラー発生後、呼び出し側は何を前提とできるか

**確定内容**：

- fsync エラー発生後、今回のパブリッシュデータは**出現している可能性も出現していない可能性もある**——呼び出し側は「エラー発生」を「今回は不成立」と解釈してはならない。
- **エラー発生後、同一のメモリ状態をそのまま再利用してリトライ可能であると前提してはならない。**（`close(2)` の NOTES 節には「Retrying the close() after a failure return is the wrong thing to do」と明記されている；本プロジェクトも同一のセマンティクスを採用する。）
- 実際に成立したか否かを把握するには、呼び出し側自らが現行ルートを読み出す：`crates/singlefs-core/src/recovery.rs` の `choose_system_configuration`、`choose_root` はオープン中のデバイスハンドルに対して判定可能であり、判定基準は「今回の checkpoint_txg が現行ルートであるか否か」であり、アンマウント・再マウントは不要である。`transaction.rs` にはこれらを呼び出すパスが存在せず、接着コードが欠落している。
- 本条項は fsync の復帰契機を規定するものではなく、それは確定項目 7 にある。

**射程**：

- **「読み出し専用移行」および「再実行も失敗」の各セルでのみ有効。** `D23` (ジャーナルの役割と形式) 確定項目 14 の障害表が「切り替え」と判定するセルでは、呼び出し側はエラーを一切受信せず（インスタンス切り替えによりインフライト checkpoint が再発行されるため）、本条項はそれらのセルには関与しない。
- 「エラー発生後に呼び出し側が何を前提とできるか」を管理するものであり、障害自体の分岐（それは `D23` (ジャーナルの役割と形式) 確定項目 14）や永続順序（確定項目 7）は管理しない。

**根拠**：

- 実験なし：本条項は対外セマンティクスの公約であり、定量可能な数値は存在しない。これを支えるのは `research/prompts/c381-r3-main-verification.md` 第 1 節 K1 セルである——H1（第 6 ステップのシステム構成スロットローテーション失敗、先行 5 ステップ全成功）において、「エラー即不成立」というセマンティクス**自体が `C381` (根已落盘之后发布失败，分配器仍退回) の原因の一つであった**：呼び出し側に不成立と誤認させ、旧状態のままパブリッシュを継続させてしまうためである。
- `research/prompts/c381-r3-main-verification.md` 第 3 節（候補甲が成立、2 つの厳格化の出所：K2 の POSIX 手がかり、K4-e の射程限定）。
- ユーザー確定 2026-09-21、発言原文は変更履歴にある。
- ⚠️ **根拠に明記を要する事項**：確定内容の後半 2 文（禁止条項、射程限定）は**攻撃ゼロ回**（`.claude/rules/three-way-inference.md`：攻撃側レグ自らが提案した厳格化は攻撃を受けていないものとみなす）、ユーザーが 2026-09-21 に合意・受容。

**未済の検査**：`C381` (根已落盘之后发布失败，分配器仍退回)（修正の配備完了まで本セルはオープンのまま）。

```

**出处 `.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md:383-386`（整段抄，未转述）**

```markdown
## 履歴バージョン

D16 (パブリッシュセマンティクス) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/16-发布语义.md:1-6`（整段抄，未转述）**

```markdown
## D16 发布语义 —— 已定

**中文** · [English](16-publish-semantics.en.md) · [日本語](16-パブリッシュセマンティクス.ja.md)

D16（发布语义） 管发布这一格：一个新根什么时候发、一次发布按什么顺序落盘、发布计数器是哪一个数、这一批释放什么时候回可分配集合、新实例的第一个根怎么先覆盖两块盘、一次空发布写不写单元、journal 记录格式在 D15（格式冻结政策） 的冻结分层里算什么。不管的：journal 的记录字段、环几何与重放前缀在 D23（journal 的角色与格式）；根环参数、根记录与系统配置的字段表在 D22（单元原子性怎么合成）；快照的记账模型与生死边界表在 D5（快照 / 空间记账机制） 与 D6（快照实现模型）；准入不等式与挂载期承诺量在 D3（空间分配） 与 D28（挂载期承诺量）；一次发布事件内部的内存序在 `litmus/commit-publish.litmus`，那份钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。

```

**出处 `.claude/kb/decisions/16-发布语义.md:29-101`（整段抄，未转述）**

```markdown
#### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读

**定案**：回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算；同一个状态只数一次（去重的判法在「「非空」从盘上怎么认」那一段）。**根环容量是这句的边界**：两个状态来回交替超过根环一圈时，环里的根全是这两个状态，保留的就只有环里有的那几个，任何机制都给不出环里没有的状态。**窗口的单位是发布，不是秒**：defer 窗口保护的是「环里的根还能不能用」，普查过的消费者没有一个按秒计价。形态取 E139（按盘回退下界的收严形态） 的臂 G2S；回收门槛退回环里最旧有效根之后，装置的候选下界已带系统配置里的 F（E139（按盘回退下界的收严形态） 第 2 次重跑；装置里 RGall 一族与窗口臂带，臂 G2S 本身不带，RGall 是 G2S 加条款形候选下界，单测 K15 钉着），恢复之后的生效值仍照装置旧式取（E139（按盘回退下界的收严形态） 逐条相符表「生效」一行，差异 ③），这一句仍不与装置里跑的一字不差，取舍欠在 C660（按盘回退下界的定案那一句与装置对不上）：

| 项 | 定案 |
|---|---|
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算。**同取这个值的六处**（用户 2026-10-04 定，被攻过一轮）：分配器的回收门槛（含抬 F 那一遍）、checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 环里最旧有效根)）、影子账的候选过滤、回退候选集、抬 F 的上限（含判「非空」那一遍）；六处都不再按一圈窗口滤根，窗口之外的有效根（崩在记录之后、根槽之前那一形留在槽里的旧根）也在候选集里、也压着门槛。**挂载时读不出的槽**：N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1；进程内每记一条根时取这条根）；读不出的槽在 txg > N − 3S（一圈，3S 是环长）里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过），抬 F 那一遍里根环表挂载时读不出的那几项同样按这一句并进最旧有效根；一圈窗口这个量只剩这一处在用，给读不出的槽划出它能住的 txg 的范围。进程内记根时，根环表里这一项成了最小的那个，先重读那一槽：读得出、自证过、按实例表仍然有效的，照读出来的根算；读得出而自证不过的，与仍读不出同一处置，回收门槛照旧钉在它可能住过的最旧 txg（「根槽这一次读坏」那一行）；读得出、自证过而按实例表被抛弃的，当没有根（用户 2026-10-03 定先重读；2026-10-04 先定自证不过那一格取按记的最大 txg 算、同日后来定读坏的槽在回收门槛里钉在可能住过的最旧 txg，2026-10-05 定以钉最旧 txg 为准，零轮）；重读读出有效根的那一槽，算进「这个进程知道住着根的槽」（「根槽这一次读坏」那一行）。 |
| 回退下界 F | 根记录带 8 字节（D22（单元原子性怎么合成） 已定项 7，242 → 250），系统配置另记一份 8 字节（D22（单元原子性怎么合成） 已定项 9）。平时不动，窗口就是整个根环；准入不够时抬，正常卸载时抬到现行那一版的 txg |
| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」在环里全部的根上取；挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮：先定只并读不出的槽，代码轮第三轮之后重定为读不出或自证不过两类都并；与「环里最旧有效根」那一行把读不出的槽并进回收门槛同一个思路）。读坏槽并进上限、卡住就在这次挂载里报 ENOSPC、不推空发布；读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |
| 根槽这一次读坏 | 算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它（「抬 F 的上限」那一行，用户 2026-10-04 先定只并读不出的槽、代码轮第三轮之后重定为读不出或自证不过都并，零轮），判回退候选与算 F_生效 时当没有根；挂载那一刻读得出、这个进程写过且 FUA 返回过、或进程内记根时重读读出了有效根的槽（这三类是「这个进程知道住着根的槽」；重建分配器两遍里只有一遍自证过的槽也在其内，抬 F 那一遍另照「重建分配器两遍里只有一遍自证过的槽」那一句处置），这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜（抬 F 那一半用户 2026-09-26 定，推的，被攻过零轮；回退那一半主 agent 2026-09-28 定，被攻过一轮：代码三方第三轮判决 `research/prompts/m2-closeout-code-r3-main-verification.md` 第一节判替没写的条款做了选择、回退扩写拒错或放错那一格没打中（一次抽样））。可写挂载的择根按 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续处置（D23（journal 的角色与格式） 已定项 14：系统配置见证过比所选那一版新的发布就把读阶段重读一次，仍判真拒可写）；只读挂载的择根、「环里最旧有效根」怎么处置这一格没判，见 C563（读根环一槽读坏就当没有根，别的调用方没判）；**退回见证到的那一版判候选读根环**（D23（journal 的角色与格式） 已定项 14 的「所选根走读失败、系统配置没见证它时退回见证到的那一版」，恢复、只读挂载、可写挂载三处同一套逻辑；判退时先问一次、触发之后重扫再问一次，各读一遍根环）：读不出或自证不过的槽一律重读一次，重读读出自证过的根照它算，重读仍坏当没有根，由「退过去会越过一次末条记录读得出、根读不出的发布就不退」那一关挡成不退，三处照报所选那一版读不出，不择别的版本（用户定案，原话在变更史；实现待下一批 crates）；**读坏的槽与读回全零的槽**：挂载那一刻读坏（读不出，或读得出不是全零而自证不过）与读回全零的槽，回收门槛三处（挂载时含 m 取不到那一支、进程内、抬 F 那一遍）钉在它可能住过的最旧 txg，这个进程写掉那一槽、或进程内记根时重读读出有效根或被抛弃的根才解开，重读读回全零照旧钉着；读坏的并进准入抬 F 的上限，读回全零的不并（「抬 F 的上限」那一行）。钉也落盘：根记录带这次发布用的回收门槛 t 与钉槽 p，写者取 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)；钉着的槽不得住着本实例写的根（C773（读坏与全零根槽的回收门槛钉只在内存里） 的合成形态，用户 2026-10-20 定，被攻过零轮；动格式，实现在系统配置落地补丁合入之后的实现批）。这一形判不出的两类坏实现（钉着的槽重读读出有效根而写者没解钉、凭空钉一个读得出的槽）不算已认代价，见 C863（钉槽的两类坏实现镜像上判不出）。**系统配置没见证到的那条根新分配的单元**：由可写挂载时的预隔离护着（D23（journal 的角色与格式） 已定项 14：水位之上、没被施加的全部 journal 记录点名的落点隔离，只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开），不随「根槽这一次读坏」那一行给读回全零的槽的进程内重读时机解开（用户 2026-10-07 定，被攻过零轮）。**可写挂载重建分配器读根环那一遍**：有一块盘一条不被抛弃的自证有效根都没读出（按读出的那一槽所在区域归盘）就整环重读一次，重读之前调钩子；重读之后逐槽取两遍里较好的那一份（任一遍自证过的根、两遍都自证过取重读那一条 ＞ 读坏不是全零 ＞ 全零），根环表、回收门槛与影子账都用并出来的那一份；两遍并起来一条不被抛弃的自证有效根都没有，在取号之前拒可写挂载，报 `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（与读阶段择不到根的 `Recovery(NoValidRoot)` 分开报）。**重建分配器两遍里只有一遍自证过的槽**（两遍按槽并起来是一条自证过的根、另一遍那一槽读不出、自证不过或读回全零；实现是 `crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）：算准入抬 F 的上限、抬 F 那一串（准入与卸载）重算回收门槛与影子账时，这一遍读它读坏就重读一次，仍坏不拒，照挂载时读出、记在根环表里的那条根算：有效根照它本身进上限与候选集，被抛弃根照它记着的落点进影子账、不进上限（这一处置的实现被攻过零轮）；挂着时管理员回退判候选与算 F_生效 时它算「这个进程知道住着根的槽」，读坏重读仍坏照旧拒（「不拒」那一臂量过）；这个进程写掉那一槽就换类，不留跨抬 F 的钉：抬 F 那一遍读坏时顶上的那条根只算在那一次抬 F 里。**抬 F 那一串（准入与卸载）里的被抛弃根**：不论住在哪一类槽，一律照挂载时算出、记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒，不走 D23（journal 的角色与格式） 已定项 14 的凭据放行（C694（被抛弃的根的账读不出时正常卸载被拒） 的 B）；「崩了再可写挂载」那一道照 B 拒，不变（用户 2026-10-07 定，实现在根环线第 10z 批，只在攻方模型上量过，被攻过零轮）；同一串读根环时被抛弃根的根槽读坏、重读仍坏不拒，照根环表里记着的落点进影子账、不进上限，有效根的槽照「这个进程知道住着根的槽」那一句读坏重读仍坏就拒（用户 2026-10-08 定「被抛弃根的槽不拒」）；根环表里没记着那条被抛弃根的落点（可写挂载自己写行之后才判成被抛弃的根，挂载那一刻记成有效根）而 C 那一圈（D23（journal 的角色与格式） 已定项 14「可写挂载时的预隔离」）在那一槽记着同一条根（那一槽记的是有效根且 txg 相同）时，照 C 那一圈记着的落点算、不读账、读不出不拒，C 也没记着的照旧读盘（用户 2026-10-08 定「用 C 那一圈记着的落点」）；那条根（R）在可写挂载那一刻账读不出而记录齐时，C 那一槽另并上那一刻现行账里仍分配着的落点（丙，用户 2026-10-08 定「改成丙」；被攻过一轮、打中两形，补成丙并中间与回退复活），再并上所选那一版与 R 之间各次发布在 C 里那几槽记着的落点（丙并中间，用户 2026-10-08 定「丙继续补洞」，被攻过一轮）；R 的记录点名的谱系片里有一条 rollback_txg 等于 R 的 txg 的谱系记录时，R 是一次管理员回退，回退目标 R_old 是根环里读出、按实例表有效、txg 等于那条记录 T_old 的根，R 那一槽另并上 R_old 账里仍分配、现行账里已释放的落点（回退复活，同一定案，被攻过一轮）；没认出回退而 R 的记录点名的打包单元有读不出或解不开的，或认出回退而根环里没读出 R_old、R_old 的账读不出，R 那一槽并上现行账里全部已释放的落点（宽护，用户 2026-10-08 定「认不出时宽护 tri」，被攻过一轮）；点名的打包单元全读得出、解得开而确实没有这条谱系记录的，照丙原样（实现 `crates/singlefs-core/src/mount.rs` 的 `placements_in_the_accounts_of_newer_readable_roots`、`with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots`、`placements_a_rollback_publish_revived_released_in_the_current_account`）；这几份与 C 那一圈别的落点同一个时机解开（这个进程写掉 R 那一槽时）。准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时（按整圈判，分不出宽护与 C 里别的几份），发布报 `UserChangeRefused::NoSpaceAfterRaisingTheFloor`，它的 `wide_protection_at_the_ceiling` 带 R 那一槽、R 的 txg、写掉 R 那一槽要几次空发布、每块盘挡着的槽数与两条出路（正常卸载之后再可写挂载；调推环入口），不自动推环（`crates/singlefs-core/src/mounted_session.rs` 的 `StuckBehindTheWideProtection`）；推环入口 `MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states` 只在用户显式调用时发 F 不变的空发布，直到 R 那一槽被写掉，调用即确认回退候选集掉到少于 D16（发布语义） 已定项 1 定案「回退候选集保留最近 4 个可退到的不同状态」的 4 个（用户 2026-10-08 定「显式推环命令加带出路的错」；带出路的错与推环入口被攻过零轮）；「被抛弃根的槽不拒」与「用 C 那一圈记着的落点」两条只在三方腿的副本与模型上量过，被攻过零轮 |
| 生效 | F 另记进每块盘的系统配置（「抬 F 那一串」那一行写明先写后抬）；恢复后生效值 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)；系统配置里读得出的取每块幸存盘两槽里全部自证过（校验和过且 fsid 与本池相同）的槽，与取号式子同一取法（D18（块里携带什么信息） 已定项 11） |
| 回退候选集 | 按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的挂着时回退取同一个集合） |
| 抬 F 那一串 | 准入与卸载共用。动盘之前在分配器的拷贝上把这一串空发布整串预演：任一次取不到落点，就一次都不发、分配器原样换回抬 F 之前那一份（`MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`）；**预演过了、第一次发布之前，先把新 F 写进每块盘的系统配置、过一道屏障**，之后才发第一条带新 F 的根；**先写那一步任何一块盘失败，这一串的根一条都不发、返回错误；已写进别的盘系统配置的新 F 留着、不回卷，回收用过的 F 不回落**（主 agent 2026-09-26 定，推的，被攻过零轮）；新 F 还没在每块盘上生效时断电，下一次挂载可以读回较低的值（先写只落了一块盘的一槽系统配置、那一槽在下一次可写挂载里一直读不出时，盘上 F 生效值 5 → 0），回收还没用过它，不算违例（用户 2026-10-20 定「收窄射程，点名用例」，零轮措辞；用例 `crates/singlefs-harness/tests/root_ring_reads_that_fail_are_not_taken_as_no_root.rs` 的 `a_floor_raise_cut_after_its_first_system_configuration_slot_falls_back_to_zero_without_reusing_units_when_that_slot_stays_unreadable_through_the_writable_mount`）；空发布推到每块盘上都有一条带新 F 的根为止；这一串回收出来的槽在生效（每块盘上都有带新 F 的根）之前扣住、不发出去，生效之后放开（正常卸载那一格例外，见「正常卸载」那一行）；真发时落盘途中写错，交回已落盘几次（`MountError::RaiseFloorSequencePublishFailed`），那一格里扣住的槽怎么放没有条款，见 C546（抬 F 被拒时扣住的槽不退回） |
| 正常卸载 | 推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串回收出来的槽，只有释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用（预演与真发都算上），其余与准入那一串一样扣到生效（恢复择根不看 F，丢一块盘时择的是剩下那块盘上最新的根，引用它的单元不能被这一串写掉）；有一块盘一条有效根都读不出时这一截为空、整截扣到生效，这一遍读不出、自证不过的槽当没有根；卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载同样只放开一截：重建分配器时当场可用的只到释放代不高于 max(min(F_生效, 「每块盘上最新有效根的 txg」最小者), 环里最旧有效根) 的槽，某块盘一条有效根都读不出时只到环里最旧有效根（挂载时读不出的槽按它能住的最小 txg 并进这个最小值，与回收门槛同一个读数），其余扣到这次挂载的暖机在每块盘上都落了根（用户 2026-10-04 定，零轮）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14）；现行那一版树表 0 条（mkfs 之后只暖机、还没写过第一个文件）时一个字节都不写、F 不动、不推空发布，卸载照常做成（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile`） |
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC）；**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**；一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC；挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）（用户 2026-09-26 定，被攻过零轮）；挂载续做的删除类发布被落点那一判以空间拒时同样推抬 F 再试，推满仍放不下挂载照样做成可写（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定） |
| `df` | 没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量；保留池 = 2 × 5 + (B − 1) × c_max = 10 + 7 c_max，残留 = 5 + (B − 2) × c_max = 5 + 6 c_max；滞后量 = 释放代 > max(第 4 新的非空有效根, 环里最旧有效根) 且不是空发布放掉的已释放块 |
| 环深下限 | 每区槽数 S ≥ k_tol + 2 = 4（D22（单元原子性怎么合成） 已定项 2 的 S 下界随之抬） |

**「非空」从盘上怎么认**：一条有效根算非空 ⟺ 它的树表里用户可见的树（inode 树、extent 树与全部快照树：码 14 快照 inode 根、码 15 快照 extent 根，D8（核心索引结构） 已定项 9）的根指针，与它前一条有效根（按 txg 排；有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F）树表里的不同；建快照那次发布算非空。比的是树表里这几类条目、不是树表单元自己的落点——每一次发布（连空发布）都重写树表单元（`crates/singlefs-core/src/transaction.rs`），按落点比会把空发布全算成非空。**数「第 N 新的非空有效根」与「4 个不同状态」时去重**：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针, 全部快照树的根指针) 与每一条比它新、已计入的根都不同时才计入——管理员回退发出的新根与它前一条有效根不同、按「「非空」从盘上怎么认」那一判法算非空，而它的状态与更早的某一条相同，不去重就会把同一个状态数两次（第二轮 X7：来回回退之后按内容只剩 2 个不同状态）。反方向只让 F 停住，直到第 4 个状态的根离开根环。

**改新池新建文件的字节：是**：根记录加 F 8 字节；新池新建文件里 F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）。系统配置加 F 8 字节，新池新建文件里写 0（D22（单元原子性怎么合成） 已定项 9）。

**连带改的**：D22（单元原子性怎么合成） 已定项 7 的根记录加 F（242 → 250）、flags 占一位卸载记号，已定项 9 的系统配置加 F 8 字节，已定项 2 的 S 下界；D23（journal 的角色与格式） 已定项 14 的回退候选集加「txg ≥ F_生效」；D3（空间分配） 已定项 9 第 2 条写明界；I-7.4（近 K 代块未被复用） 的 K 改由 F 表达；I-7.9（回退下界 F 不高于抬 F 的上限） 按卸载记号分两支。

**射程**：定的是 defer 窗口与回退下界怎么算，不定环几何、不定 c_max 的口径（c_max 是声明的最坏空发布开销，按 D28（挂载期承诺量） 已定项 4 按当时结构现算）。已知边角十一样：

- **「生效」取 SysPre**：F 只住根记录、根槽不镜像（已定项 7）时，带新 F 的根坏掉 F_生效 就回落，更早的根重新进回退候选集、而它们引用的块已被合法复用——I-7.4（近 K 代块未被复用） 在这段历史上不成立（`research/prompts/m2-step45-code-r2-main-verification.md` 第四节、`research/prompts/m2-step6-checker-r1-main-verification.md` 第四节）；病根是 F 只住根记录，与生效取哪条规则无关（第三轮 K4：带 F 的根全坏那一格，第三轮时的规则、MAX、MAX+HOLD 候选坏 12 / 140 / 12，SysPre 0）。代价：系统配置多 8 字节（动格式）；每次抬 F（准入与卸载）每块盘多一次系统配置写、加一道屏障；checker 与崩溃点重放要跟着读系统配置里的 F——当时的 checker 按根上带的 F 判，带 F 的根全坏时会误红 I-7.4（近 K 代块未被复用） 等五条（第三轮 Y7，推的）；checker 与崩溃点重放的记录核对器已读系统配置里的 F，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清。「先写后抬」是第三轮攻方提的，被攻过零轮，用户 2026-09-26 知情定。被抛弃时间线上的根带的 F 算进 F_生效（用户 2026-09-28 定「算进」，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M2：被抛弃根带着比别处高的 F 时，不算的话 I-7.12（系统配置 F 不低于同盘根上的 F） 红、回退退得到那个 F 之下，而被抛弃时间线抬 F 时已按它回收过槽；实现 `crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor_of_the_roots_read`，checker 另算一份 `crates/singlefs-checker/src/walk.rs`）；C331（择根倒挂压过已确认的写） 判不出改取 R2 之后（D22（单元原子性怎么合成） 已定项 8 射程），可写挂载只在有槽设备报读错、且没有一块盘两槽都自证过时才拒，一槽系统配置一时读不出而另一块盘两槽都自证过的盘面不再被拒；被抛弃根带着更高 F 的盘面是不是仍只在构造的镜像上有，没有重新量过（推的）。与 `research/prompts/alloc-basis-r3-main-verification.md` 的 F 口径岔路一起判的那一件仍开着。
- F 回落之后第 0 代根重新进有效根、它的树表单元已被合法复用、此后每一次抬 F 都被拒到根环把它盖掉为止（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md` 第一节 Z1）：前提是 F 回落，取 SysPre 之后在带 F 的根全坏那一格上不再出现（推的，第三轮没造那一形）。一次挂载里连续发布超过根环容量时挂载内不回收，checker 在合法状态上判 I-3.1（已分配统计对得上） 红（E153（账本形态与环上有洞的代价） 在 `crates/` 副本上量到），仍开着，归里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ② 行。
- **正常卸载的代价**：每次卸载多 2 或 3 次空发布（树表 0 条的一版上不推，见定案那一句），第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节（第一轮攻方 t10、t5a，副本上量的）；卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版。用户选时知情的代价是「停机时崩溃兜底只剩最新一条根」（按条款推的）；第二轮在卸载之后再挂、最新 j 条根坏的 75 行上恢复全读对、可写挂载全挂上，这一条没出现：F 之下的根还在环里，它们的记录能把状态重放回这一串的末尾（第二轮判决 G4）。
- **用户知情接受的代价三条**：删掉的空间要再发生 3 次改变用户可见状态的发布才回可分配集合，在那之前 `df` 不把它算空闲、按 `df` 写它是真 ENOSPC；`df` 少报 = 保留池 + 残留 = 15 + 13 c_max（c_max = 5 / 10 时 80 / 145 块）再加滞后量，而滞后量 = 4 × 最近四个窗口的释放量、**没有上界**（一窗删一个 8 块对象时平均约 34 块、最大 44 / 52，一窗删 32 个时 1044 块），删掉 s 字节 `df` 一个字节都不涨；c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块。
- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。
- **代价是选项本身的，不是这一族的缺陷**：一个被保留的状态钉住的是在它那一刻活着、后来才死的全部块，选最近 4 个就是每次删除都撞上——滞后量与「删了要等 3 次状态」都从这里来。
- **回收门槛取环里最旧有效根的代价**：崩在记录之后、根槽之前那一形留在环里的旧根要等被覆盖一圈才离开，每个洞回收门槛多压一圈（正推腿端口上 S = 16 平均 893 块对窗口门槛 583 块）；一块盘持续写不进根时扣住的单元随发布线性涨、没有上界（攻方模型上量，持续写失败 15 / 30 / 60 / 120 次时最多扣住 50 / 80 / 140 / 260）；崩在记录之后、根槽之前连崩 13 次之后挂载（S = 4），已分配 166 块、窗口门槛同一段 26 块（攻方副本上量）。数两腿各量一种口径，不是入库装置上的数；用户 2026-10-04 知情定。
- **E139（按盘回退下界的收严形态） 答不了的**：计数模型，按发布计不按墙钟；只建根一级的引用、没建 journal 记录重放；实例切换的挂载时预留由 D28（挂载期承诺量） 已定项 3 扣；容量 4000 块、两块盘；k_tol 固定 2；滞后量口径是写登记的人替状态数读法构造的；保留池扫描的最小值落在扫描下沿、真正要多少没量到；环深下限是探针的数。两轮前身 E135（动态回退下界） 与 E138（按盘回退下界与推空的空间要求） 的结论已被 E139（按盘回退下界的收严形态） 取代，不再当依据用。
- **I-7.9（回退下界 F 不高于抬 F 的上限） 的重算上限不在六处之内**：池级 checker 与 GPU 判器的重算上限仍只取锚在镜像最新根上的一圈窗口里的根（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings` 文档注释写了理由：后来的镜像上重算，洞里留下的旧根会把上限算低、在合法镜像上误红），六处退回「环里最旧有效根」不含这一处。
- **重建分配器读根环那一遍按槽归盘、逐槽并两遍**：条文照用户定案写（「根槽这一次读坏」那一行），实现在第 10h 批。
- **快照树根并进「非空」与去重**：条文照用户定案写（「「非空」从盘上怎么认」那一段），实现在里程碑四第 3 批丙 2：core `crates/singlefs-core/src/mount.rs` 的 `user_visible_trees_changed` 与 checker `crates/singlefs-checker/src/walk.rs` 那一份一起改；合入之前 `user_visible_trees_changed` 只比 inode 树与 extent 树。

**依据**：

- E139（按盘回退下界的收严形态）：臂 G2S 在全部故障世界（活化区间内崩溃不丢盘、先跳号再抬 F、推空期间 1..3 次根槽写失败、坏槽、开销升到 c_max）零假候选、零不可恢复，各格按自己的 `df` 假性 ENOSPC 为 0，一次准入的发布数实测最大 7 ≤ 8，跑前保留池下不卡死——定案取的就是这条臂（跑前登记 `e139-preregistration.md`，产物 `e139-tightened-floor-2026-09-12.out`）。
- 四轮三方论证，第四轮判决 `research/prompts/d16-item1-r4-main-verification.md`；第三轮 `research/prompts/d16-item1-r3-main-verification.md`。
- 用户三次定案（候选集可以一直缩、最少保留 4 个、「4」数的是可退到的不同状态、三种「哪 4 个」里选最近 4 个），原话在变更史；「非空」怎么认与「写行那次发布之前不推」同为用户定案，原话在变更史。
- 用户定案 2026-09-25：抬 F 那一串改成整串预演，推翻 2026-09-24「维持逐次发布」（原话在变更史）；实现是实二六（`m2-e26-implementer-report.md`）。
- 生效取 SysPre、卸载抬 F 到现行那一版的 txg（B1）、「4 个不同状态」去重与根环容量边界、候选按现行那一版的实例表判：无实验，三方原型（`research/prompts/m2-rollback-forward-r1-opus-model/`、`m2-rollback-forward-r2-opus-model/`、`m2-rollback-forward-r3-opus-model/`）在冻结副本上量过，没立实验号，原型上的数不是入库装置上的数；三轮判决 `research/prompts/m2-rollback-forward-r1-main-verification.md`、`m2-rollback-forward-r2-main-verification.md`、`m2-rollback-forward-r3-main-verification.md`（第三轮第三节站住的、第四节交用户的）。SysPre 的「先写后抬」被攻过零轮。系统配置里读得出的 F 取每块盘两槽里全部自证过的槽、先写那一步失败时一条根都不发且已写进的 F 不回卷，这两处是主 agent 2026-09-26 定的，推的，被攻过零轮。
- 用户定案 2026-09-25（正常卸载时抬 F、取 B1）与 2026-09-26（C419（F 生效之后靠什么保持不回落） 取 SysPre，推翻 2026-09-25 定的「生效值取各盘 F 的最大值」；「4 个不同状态」加根环容量边界），原话在变更史。
- 「根槽这一次读坏」那一行：抬 F 那一半是用户定案，原话在变更史，构造的反例在 `records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行；回退那一半是主 agent 2026-09-28 定（同一份调度记录第 309 行），经过在变更史。实现已合入：`crates/singlefs-core/src/recovery.rs` 的 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 对这个进程知道住着根的槽「读不出或自证不过」都重读一次，仍坏报 `RootRingSlotKnownToHoldARootStillBadAfterOneReread`（带 `first_reading` / `reread`）；算上限那一遍是 `crates/singlefs-core/src/mount.rs` 的 `roots_of_the_ring_for_the_ceiling`（仍坏报 `MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），抬 F 与卸载重算影子账与回收门槛那一遍仍坏报 `MountError::NewerStateStillUnreadableAfterOneReread`（`StillUnreadableAfterOneReread::RootRingSlotKnownToHoldARoot`），管理员回退的 `rollback_candidate` 判候选与算 F_生效（`recovery.rs` 的 `effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once`）仍坏报 `RollbackError::CandidateJudgementStillUnreadableAfterOneReread`、在任何写之前。用例：`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的 `raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write`、`a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised`、`rolling_back_while_the_targets_root_ring_slot_stays_unreadable_is_refused_before_any_write`；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 的 `rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write` 与 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 的 `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write`（这个进程写过的根被改坏之后回退拒、盘上不变）（`m2-impl-r2-fixes-b-implementer-report.md` 第三节第 3 行；合入后验证一收窄成只管读不出、之后由改法 B 还原成全句，见 `m2-impl-merge-verify-1-implementer-report.md` 第三节第 1 小节）。
- 「准入」那一行的落点被拒也推、挂载处写行之后推：用户定案，原话在变更史；安全设计轮第三轮判决 `research/prompts/m2-safety-r3-main-verification.md`（会话里落点取不到 C283（准入失败时不先推发布就报 ENOSPC） 不管、只崩了再挂就把小盘挂死）。
- 回收门槛与同取窗口的另外五处退回环里最旧有效根：判决 `research/prompts/m3-reclaim-window-r1-main-verification.md` 第二节 W1、W3 各行与第六节（窗口门槛在目标形态上出局：丢一块盘之后挂载落到幸存盘上更旧的根，读出来是坏数据，计数模型全枚举；只改分配器门槛或只改 checker 都重开 I-3.1（已分配统计对得上） 误红，副本量过；六处一起退，两腿独立量到零假候选、零不可恢复，被攻过一轮）；用户定案 2026-10-04（Q1 RGall、Q2 下一轮就改），原话在变更史。E139（按盘回退下界的收严形态） 第 1 次重跑：旧门槛对照臂 G2SW_ringgate 判的行零假候选、零不可恢复，加窗口的 G2SW 有非零行；其中一部分是装置的候选下界没带系统配置里的 F 造出来的（判决 W2 行），数不当依据，以实验页为准，装置补 F 之后的第 2 次重跑 2026-10-05 已跑完：候选下界带系统配置里的 F，RGall（回收门槛与另外五处退回整环）在第 1 次重跑那 13 类世界与 `disk_stuck` 上假候选、不可恢复、判的格假性 ENOSPC 都是 0，数以实验页为准；收口欠里记着。无实验：判决里的数在三方腿的副本与模型上量，没立实验号，不是入库装置上的数。取一圈窗口的三方判决（`research/prompts/m3-ring-r2-main-verification.md`、`research/prompts/m3-ring-r3-main-verification.md`）与用户 2026-10-02 的定案，经过与原话在变更史。
- 抬 F 的上限并进挂载那一刻读不出或自证不过两类槽：代码轮第三轮判决 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第二节 K2 的 RS3 行（一槽从挂载起读回全零、读得出而自证不过，根环表不记它；准入抬 F 的上限算成 22，越过盘上完好的非空有效根 21，池级 checker 判 I-7.9（回退下界 F 不高于抬 F 的上限） 红；设备报错那一形停在 21、0 红；副本量过，主 agent 复跑对上）与第六节 Q3 行；用户定案 2026-10-04（读不出与自证不过两类都并），原话在变更史。无实验：数在攻方副本上量，没立实验号。
- 抬 F 那一遍并进挂载时读不出的槽、进程内记根先重读、抬 F 的上限只取窗口里的根（这一半随回收门槛退回环里最旧有效根而作废）：代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 K2-e、K2-b、K2-b2 三行（攻方原型量过，主 agent 复跑 115 份输出逐字节相同）与第三节表二之后用户定案那一段（e2、B2）。
- 进程内重读读得出而自证不过与仍读不出同一处置（2026-10-04 RS1 先定按记的最大 txg 算，2026-10-05 用户定以钉在可能住过的最旧 txg 为准：记最大在 RGall 下回收过 mkfs txg 0 那条有效根引用的单元，`records/2026-09-30-里程碑三开工第一轮.md` 第 10 批 core 簇交回那一条）、重读读出有效根的槽算进这个进程知道住着根的槽、抬 F 的上限把挂载那一刻读不出的槽当作可能是非空有效根：代码轮第二轮判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第二节第 2、6 行（有效根所在的槽挂载那一刻读错、重读又读坏，实现当没有根，回收门槛越过盘上有效根，攻方模型上 12 组参数 12 组红，改法 A 12 组 0 红；抬 F 越过读不出的有效根那一形，攻方探针上 F = 22、读得出时 F = 21，推的）与第四节 RS1、RS3 两行（改法 A 的代价是从没写过的槽加挂载读错时记账多，安全方向，2 组；上限算进读不出的槽的代价是读故障时 F 抬得少一点，推的）；用户定案见同一份判决第六节，原话在变更史。无实验：数在攻方模型上量，没立实验号。
- 正常卸载那一串回收的槽只放开一截：设计轮判决 `research/prompts/m3-unmount-after-short-continuation-r1-main-verification.md`（续做推满之后卸载在预演里被拒，丙1 站住、零轮）；代码轮第三轮判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第二节 n1 各行（丙1 在单盘幸存时写掉被择中的旧根引用的单元，两次抽样同向打中，主 agent 复跑坐实；恢复择根不看 F）与岔路单 `m3-unlink-code-r3-forks.md`。用户定案 2026-10-03：「丙 1 进下一批」、单盘那一问「甲 GA（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 有一块盘没有有效根时整截扣、读不出的槽当没有根、卸载半途崩溃之后的下一次可写挂载只放开一截：代码轮判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节（n1「根环里读不出的槽、有洞」那一行是代码取的保守读法；「形 R」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch14-code-r1-main-checks/`）与岔路单 `m3-batch14-code-r1-forks.md`。用户定案 2026-10-04 选「甲 再挂也只放开一截（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 再挂时某块盘一条有效根都读不出只放到最旧有效根（该判决写作一圈窗口里最旧有效根，随回收门槛退回环里最旧有效根）、当场放开的式子带上最旧有效根那一项：代码轮判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节（n1「再挂期间某块盘根环整段读坏」那一行攻方打中、主 agent 复跑坐实，`research/prompts/m3-batch16-code-r1-main-checks/`；窗口那一项是正推腿报的条款漏写）与岔路单 `m3-batch16-code-r1-forks.md`。用户定案 2026-10-04 选「甲 只放到窗口里最旧有效根（推荐）」，原话在变更史。无实验：数在三方腿的拷贝上量，没立实验号。
- 树表 0 条那一版上正常卸载一个字节都不写、F 不动：用户定案 2026-10-04（选乙「不推，条款照实现写」，原话在变更史），把实现里主 agent 2026-09-26 那条定案（`crates/singlefs-core/src/mount.rs` 的 `Unmounted::NothingWrittenOnAVersionWithoutFile` 的注释，推的，被攻过零轮）写进条款；代码轮判决 `research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K1 与第五节第 1 行（照字面推空发布那一读法在攻方副本上量过：模型推 2 条、对拍报 PublishCount）。无实验：数在三方腿的副本上量，没立实验号。
- 「根槽这一次读坏」那一行的钉最旧 txg、全零槽、重建时整环重读一次再拒、按槽并两遍：用户定案 2026-10-04（读坏的槽在回收门槛里钉在可能住过的最旧 txg）与 2026-10-05（全零槽在回收门槛里照自证不过钉住、只在准入抬 F 的上限里不并；重建那一遍重读一次再拒；Q3 另起拒因；Q4 采用 rr2；按槽并两遍），原话在变更史；判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二、五、六节、`research/prompts/m3-batch10-code-r2-main-verification.md` 第二、五、六节。E173（上限卡在读坏槽的三种处置） 不撑这一格。无实验：数在三方腿的副本与模型上量，没立实验号。
- 「根槽这一次读坏」那一行的重建分配器两遍里只有一遍自证过的槽：判决 `research/prompts/m3-batch10-code-r3-main-verification.md` 第二节 K1「并出来的分类与「挂载那一刻读坏」是不是同一类」那一行（第一遍自证、重读起一直读坏的槽记成这个进程知道住着根的槽，抬 F 重读它仍坏就拒这一次抬，可写挂载在写行暖机之后失败）与第六节 S1；判决 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第二节 K1（照挂载那一刻读坏钉住时，被抛弃那一半掉出影子账、I-7.4（近 K 代块未被复用） 红，准入上限卡在它能住的 txg；「什么时候解开」那一格）、K2（回退与抬 F 对同一槽处置不同）与第六节 t1、t2，岔路单 `m3-batch10i-code-r1-forks.md`；用户定案 2026-10-06（另立一类；照挂载时读出、记在根环表里的那条根算；回退照旧拒、下一轮先量「不拒」那一臂），原话在变更史。无实验：数在三方腿的副本与攻方原型上量，没立实验号。
- 「根槽这一次读坏」那一行的重建分配器两遍里只有一遍自证过的槽在回退判候选与算 F_生效 时读坏重读仍坏照拒：判决 `research/prompts/m3-batch10o-code-r1-main-verification.md` 第二节 K1-3 那一格（「拒」与「不拒」两臂，两腿在副本上各量一遍，数在那一格）与第五节，岔路单 `m3-batch10o-code-r1-forks.md` T2 那一行；用户定案 2026-10-06（照旧拒），原话在变更史。无实验：数在三方腿的副本上量，没立实验号。
- 「抬 F 的上限」那一行读坏槽并进上限、读回全零不并：E173（上限卡在读坏槽的三种处置） 的分臂表（照今天不让 F 越过读坏槽里的非空根，推空比它只少拒 4 / 15552 个位置（条款读法），而推空发布有取不到落点的位置；不并让 F 越过 180 个位置；全零不并在条款读法下 F 越过 90 / 7794 个位置、全零并多拒 18 个位置，数以实验页为准）；用户定案 2026-10-05（Q1 照今天、Q2 照今天不并），原话在变更史。
- 池级 checker 与 GPU 判器的 I-7.9（回退下界 F 不高于抬 F 的上限） 重算上限仍取一圈窗口：判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节 K3 与第五节（正推冲突并排第 1、2 行）。
- 「根槽这一次读坏」那一行的系统配置没见证到的那条根新分配的单元由可写挂载时的预隔离护着、解开不随进程内重读：判决 `research/prompts/m3-c694-r1-main-verification.md` 第二节「K1 两条条款各管哪一半」那一行，`research/prompts/m3-c694-r2-main-verification.md` 第二节「K2 C 的解开时机」那一行与第三节第 1 条；用户定案 2026-10-07（C694（被抛弃的根的账读不出时正常卸载被拒）「C 与 B 都做」），原话在变更史。无实验：数在三方腿的副本与模型上量，没立实验号。
- 「根槽这一次读坏」那一行的抬 F 那一串被抛弃根的根槽读坏不拒、表里没记着时用 C 那一圈记着的落点：判决 `research/prompts/m3-batch10z-b-code-r1-main-verification.md` 第二节 K1-1（H1，攻方：照判决时的实现 90 / 90 段历史拒，不拒那一臂 0 / 90，两臂 I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽） 0 红）、K2-2（H2，照判决时的实现正推 19 / 19、攻方 285 / 285 拒，C 那一臂 0 拒、0 红）与第三节；C 那一槽另并挂载那一刻仍分配着的落点：判决 `research/prompts/m3-batch10z-d-code-r1-main-verification.md` 第二节 K2-2 两行（攻方：那条根与所选那一版共用会被释放的单元时只借 C 那一臂 k=3 53 / 150、k=5 68 / 150 段 I-7.4（近 K 代块未被复用） 红，并上那一截的一臂 0 红、0 拒，主 agent 在冻结副本拷贝上复跑逐字相同）与第三节；用户定案 2026-10-08，原话在变更史。无实验：数在三方腿的副本与模型上量、主 agent 在冻结副本拷贝上重跑，没立实验号。
- 「根槽这一次读坏」那一行的丙并中间、回退复活、宽护（tri）、宽护卡住准入时带出路的错与推环入口：判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 两行（丙打中攻方一形与正推一形）与第四节；`research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第二节 K1-1、K2-2（两支站住）、K2-3（认不出回退那一形）与第五节；`research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第二节 K1-1、K1-2（宽护没打中）与 K1-3（宽护随释放历史涨，写被拒就写不掉 R 那一槽、隔离到进程结束）；`research/prompts/m3-batch10z-no-code-r1-main-verification.md` 第二节 K1-1、K1-3（去掉双扣的改法没打中）；调查报告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md`（准入被宽护卡住的两层与推环的代价）；用户定案 2026-10-08（「丙继续补洞」「认不出时宽护 tri」「显式推环命令加带出路的错」），原话在变更史。无实验：数在三方腿的副本与模型上量、主 agent 复跑，没立实验号。
- 「根槽这一次读坏」那一行的抬 F 那一串被抛弃根照记着的落点、不从盘重读、读不出不拒：判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第二节 K4-2 那一格（攻方量过：挂上之后被抛弃根的账读坏，照 B 从盘重读，84 / 84 段卸载被拒；照记着的落点那一臂 84 / 84 段卸载做成、卸载后 I-7.4（近 K 代块未被复用） 0）与第三节；用户定案 2026-10-07（抬 F 照记着的落点），原话在变更史。无实验：数在攻方腿的副本与模型上量，没立实验号。
- 「非空」与去重并进全部快照树的根指针：用户定案 2026-10-14（建快照那次发布「算非空」），原话在变更史；码 14 / 15 两类条目的形态见判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1。无实验：定的是「状态」由哪几棵树的根指针组成，没有可量的量。
- 「根槽这一次读坏」那一行的钉也落盘（根记录带 t 与 p、t 的取法、钉着的槽不得住着本实例写的根）：根环三方判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节判 C773（读坏与全零根槽的回收门槛钉只在内存里） 那一格与 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773（读坏与全零根槽的回收门槛钉只在内存里） 合成形态那一格（形态站住、带零轮补法）；用户定案 2026-10-20（「去掉条件，里程碑三里做」「认 (1)(2)，不认已认代价」），原话在变更史。无实验：转绿、误红都没有入库镜像，两腿都标推的。
- 「根槽这一次读坏」那一行的退回见证到的那一版判候选读根环一律重读一次：用户定案（弹窗，原话在变更史）；四处调用方少一条根各有什么后果的调查在 C563（读根环一槽读坏就当没有根，别的调用方没判） 那一行，有后果的只有这一处；会红用例 `crates/singlefs-harness/tests/fall_back_candidate_root_slot_reading_bad_once_does_not_change_where_recovery_and_both_mounts_land.rs`。
- 「抬 F 那一串」那一行回收用过的 F 不回落、新 F 没在每块盘上生效时断电可以读回较低的值：调查（`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 382 行：叠加故障下生效值 5 → 0，回落之前没有回收用过 5，无数据后果）与那一行点名的回归用例；用户定案 2026-10-20「收窄射程，点名用例」，原话在变更史。无实验：数在调查用测试与回归用例上量，没立实验号。

**欠**：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（与 defer 窗口取多大无关，病根在 D23（journal 的角色与格式） 已定项 14 的影子账）；C329（写行那次发布之前推抬 F 的空发布没有检查）；C563（读根环一槽读坏就当没有根，别的调用方没判）；C660（按盘回退下界的定案那一句与装置对不上）；C746（进程内重读读出被抛弃的根时全零槽的钉同样解开）；C863（钉槽的两类坏实现镜像上判不出）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:102-114`（整段抄，未转述）**

```markdown
#### 已定项 2：checkpoint 的触发判据取时间与脏数据量，两者取先到

**定案**：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)`。两个阈值是**系统配置声明值**，挂载时求值一次（形态与 D21（权威态与派生态的分界）「单元几何挂载时求值一次」同型）。

**射程**：定的是什么时候触发，不定两个阈值取多少（已定项 5），也不定触发之后怎么切分事务（同样在已定项 5）。已知边角一样：`.claude/rules/fs-design.md` 硬要求 5 说的分支变量是那两个阈值、它们在挂载期间恒定，`now` 与脏字节数是被比较的量、会变，不是分支变量。

**依据**：

- 无实验：`∨` 这个形态是两条失败模式互相堵出来的，没有可量的量——只按脏数据量，空闲系统上最后一批写可以无限期不落、崩溃丢窗口无界；只按时间，突发写会把一个 checkpoint 的 journal 占用撑过环几何，而 I-8.1（环几何够大） 要的是「任一事务的最坏 journal 占用」有界。那个 `∨` 式子就是全部，没有魔数以外的自由度，`.claude/rules/fs-design.md` 硬要求 1「判据显式、可计算、写下来」因此满足。可量的量是两个阈值的取值与它们的可行域，在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:115-135`（整段抄，未转述）**

```markdown
#### 已定项 3：journal 记录格式是四层之外的第三个独立冻结组件

**定案**：journal 不进 D15（格式冻结政策） 的四层链。它的记录格式——记录头（含已定待落地的事务号 + 提交标记、反向链、载荷校验和三笔增量）、点名项、载荷校验和的覆盖范围、事务边界——立成四层之外的第三个独立冻结组件；住系统配置的 journal 几何字段（记录尺寸、环大小、在飞记录数上限、tail 槽）是系统配置字段，随第 1 层冻。新记录格式走「新记录类型 + 系统配置 log-incompat 位」：环里有这种格式的记录时置位、环干净时清掉，不认识就拒绝挂载；永久要读得懂的只有判定「这条记录不施加」要用的那几个头字段（magic、类型、自述长度、`jsn`、`checkpoint_txg`、头校验和）。

**冻结前置**（不早于它嵌进来的每一个外部定义）：第 2 层（点名项里的位置条目）；落点粒度那一格（D3（空间分配） 已定项 7 欠着的处置）；D18（块里携带什么信息） 已定项 11 的单元类登记表（点名项里的单元类型标签）；树 ID 所在的树表单元——它还没有冻结层归属，所以这一条前置还挂着。**冻结时刻是提交 `format-frozen/journal-record.toml` 的那个 commit**，不许写成「第一个写出记录的镜像」。

**射程**：只答 journal。树表单元、实例表单元与根记录的冻结层归属仍是 C246（树表单元没有冻结层归属） 那一格。三样已知边角：

- **置位与清位是运行期动作**，不走提交这条时间线，要另配一道运行期的闸（写第一条新格式记录之前先持久置位，缺位即拒绝写），随组件的 spec 一起落。
- **还冻不了**，挡着五样：点名项 56 字节只定了总宽，构成只有 [layout/01-first-txn.md](../layout/01-first-txn.md)「六、journal」的预想表，也没给加密字段留位（D23（journal 的角色与格式） 已定项 1 的 D 条「项的大小由记录类型决定」管着这件事）；记录头登记的格式常量是 78，三笔增量落地后是 95；spec 文件不存在；log-incompat 位还没有位号，也没进 D15（格式冻结政策） 那张 feature bit 记账表；冻结前清单第 1 项量宽度、第 7 项量未定分项，两项都看不见 `jsn` 解释口径打架那笔欠账，按现行的清单冻这个组件时两项都不红。
- **「什么时候算环干净、可以清位」要按本工程的重放下界定**（D23（journal 的角色与格式） 已定项 14 的水位），不照搬现役实现。

**依据**：

- 无实验：冻结层归属是格式政策，能核的是各层之间的依赖顺序，没有可量的量；记录头宽度那笔字节账在 C94（登记的格式常量与后来的定案对不上）。
- 三方论证（材料 `_d16-items134-background.md`，产物同目录 `d16-items134-*`：`d16-items134-sonnet-output.md`、`d16-items134-opus-output.md`、`d16-items134-local-output-s1.md` 与 `-s2.md`）：正推腿判「独立组件」是唯一与 D15（格式冻结政策） 已定项 1 自洽的答案，反推腿补上 log-incompat 位之后演进那一格赢，并打中两处前置；本地腿两次抽样的反例把字段挂错了层，主 agent 核后不采纳。判决取独立组件 + log-incompat 位，冻结前置按反推腿收严。
- 现役参照：XFS 有 `XFS_SB_FEAT_INCOMPAT_LOG_ALL`，在主系统配置里置位、日志里没有受保护的条目时清掉（本机树 `fs/xfs/libxfs/xfs_format.h` 的 `XFS_SB_FEAT_INCOMPAT_LOG_ALL` 定义、`fs/xfs/xfs_mount.c` 的 `xfs_add_incompat_log_feature` 与 `xfs_clear_incompat_log_features`），**未在本项目验证**；一处已知差异：XFS 的日志要重做逻辑与物理混合的条目，本工程的记录只承载指针层目标态。
- 用户定案，原话在变更史。

**欠**：C119（冻结组件没有 spec 文件）（journal 记录格式是第三个，那一行的简称还写「两个」）、C246（树表单元没有冻结层归属）、C199（实例代号递增与 jsn 断号即止互相矛盾）（冻它之前要先还）、C225（组件的冻结时刻绕开门禁触发）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:136-155`（整段抄，未转述）**

```markdown
#### 已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层

**定案**：「施加一条记录在指针层上做什么」那个二选一取「一次发布整体施加或整体不施加」。记录核对器那第二份 replay 实现因此停在句法层：前缀判定 + 择根 + 续做，不实现树语义、不重新生成祖先。与 D13（验证路线） 的关系照收窄后的四样：

| # | 定了什么 | 依据 |
|---|---|---|
| ① | **共享**：与实现只共享 D13（验证路线） 已定项 5 那份生成常量；与 checker **默认不共享**，要共享得另行论证；不与 RefFS 共享恢复代码 | D13（验证路线） 冲突 1 的折中管的是 checker 自己遍历、扫描两个方向之间，不授权两个判决器之间共享（两条云端腿一致） |
| ② | **入参拆成三份**：崩溃态镜像（择根与前缀判定看它）、记录流（崩溃态镜像环里的 journal 记录）、实现恢复后的镜像（比对的对象） | 恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14），D13（验证路线） 定案写的「崩溃后镜像」在 harness 先恢复、后核对的顺序下是两份 |
| ③ | **缺一个数**：前缀第一条要的「在飞记录数上限」本仓没有值（I-8.3（重放前缀严格连续）；D23（journal 的角色与格式） 已定项 1 的 C 条要它进系统配置） | 那个数在本仓零命中 |
| ④ | **合谋测试**：记录核对器是第四个判决器；每个判决器都要有一个只有它抓得住、而且对可见状态或结构有害的坏实现。「摘掉根槽前的屏障」不够格——它抓到的缺失只伤记录核对器自己；候选待找：原来的候选回退行 W 截断随管理员回退改成挂着时的向前发布一起删掉（D23（journal 的角色与格式） 已定项 14），第二轮量到截断在第二轮时的代码上本来就走不到。O3（独立规约执行器） 是 [verification-build.md](../verification-build.md) 落地顺序的第 6 步 | 反推腿 4-D |

**射程**：定的是第二份 replay 做什么、不做什么，**不定它要多少人工**——零代码时那个数没有可核的答案，只有 research 里三样工作的模型等价物当下界替身（`e42_transaction_records.rs` 240 行、`e77_publish_order.rs` 506 行、`e78_replay_start.rs` 556 行，反推腿 `wc -l`），第四样（把期望态算出来）一行模型都没有。指针层其余部分（允许部分施加时换哪个指针、祖先谁来 COW、落点谁分配）仍归 C284（施加一条记录在指针层上做什么没有定义）；D8（核心索引结构） 已定项 13（同一次发布里同 key 的中间态折叠）靠这一支：施加只落在发布边界上，同一次发布里的中间态才不会被 replay 出来。

**依据**：

- 无实验：第二份实现必须独立实现什么、允许共享什么，是验证路线的分工，没有可量的量；「要多少人工」在零代码时也没有可核的答案，只有行数下界替身。
- 三方论证之后的用户定案收窄问法（问的是「那第二份实现必须独立实现什么、允许共享什么」加「它在合谋测试里站什么位置」），原话在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:156-180`（整段抄，未转述）**

```markdown
#### 已定项 5：`T_time` 与 `T_dirty` 取多少，丢失窗口的口径

**定案**：`T_time` 默认值 5 s，取可行区间 [200 ms, 5 s] 的上端——默认丢失窗口是 5 秒，取更小只减少摊销、不多买任何东西，拦住丢失的是那 5 秒本身。**`T_time` 用户可设，不限制**：挂载时传的值覆盖盘上的字段并回写；超出 [200 ms, 5 s] 不拒绝，只给警告（用户 2026-09-20）。`T_dirty` = 2 GiB（初值，写代码时再调）；它不由丢失窗口定——`∨` 规则下时间那一支已经保证了窗口，`T_dirty` 管的是**量**，它就是脏页内存预算本身。**`T_dirty` 是可调值，不是格式常量**：系统配置里那个字段是格式，字段里放什么数不是。**挂载时还要再夹一次**：有效 `T_dirty` = min(系统配置里那个字段, 环长 ÷ F)；环长是 mkfs 参数（默认 768 MiB、约束环 ≤ 设备容量 ÷ 4），所以「环装不装得下 `F ×` 最坏占用」不由一条 mkfs 时的下界保证，改成挂载时把脏字节预算压下去；默认环下有效值 = min(2 GiB, 256 MiB) = **256 MiB**，系统配置里 `T_dirty` 仍写 2 GiB、字段不动。

**丢失窗口的口径**：取号 = 事务被分配进某个 checkpoint 那一刻，checkpoint 的切分只关闭成员资格——触发之后不再分配事务进它，已分配的事务把各自的单元写完，发布阶段等全部单元落盘（任何 COW checkpoint 都要等最慢的那次 I/O，ZFS 的 txg sync 同样等最慢的 vdev，未在本项目验证）。事务切分纪律是一事务一单元。落到取号上，事务切分纪律把一次写请求按单元切成若干事务各自取号，一个事务最多写一个单元的用户数据，**同一次请求切出的若干事务按其单元的 key 升序取号**（提交路径要记「同一请求内取号序与 key 序不一致的次数」、非 0 判红）。单元写的重试预算是墙钟 T_retry（具名可调参数，不是格式常量；用尽即按失败的性质走实例切换或转只读，D23（journal 的角色与格式） 已定项 14——它同时是丢失窗口的加项、挂载内实例切换的判据、以及连续切换计数的时间基准）。**由此崩溃最多丢的是 T_time + N_switch × (T_retry + T_redo)，不是恰好 5 秒**——一次挂载允许 N_switch 次实例切换，每次都要熬完 T_retry 再把在飞 checkpoint 重做一遍（T_redo），而重做期间开放的那个 checkpoint 发布不了、只能一直攒。这里的 `T_time` 是本次挂载生效的那个值，不是 5 秒这个常数；`T_time` 可调之后，2026-08 用户定案的「崩溃最多丢 5 秒的写」降级成**默认值下的承诺**，不是全局承诺。**「上次 checkpoint 的时刻」在重发时取重发那一刻**（取原来那一刻的话，切换期间触发条件一直成立而在飞深度只有两个，触发只能空转）；在飞 checkpoint 深度 = 开放 + 发布中两个；并发 fsync 的返回时刻被同 checkpoint 的其他事务拖住是组提交固有的代价。这是已定项 14 那条「快照发布点钉在这个 checkpoint 全部数据变更之后」的可执行形态。

**射程**：定的是两个阈值取什么数与崩溃最多丢多少，不定环几何本身、也不定重做期间开放的那个 checkpoint 会不会堵住写者（D28（挂载期承诺量） 已定项 3 点名的那一格口径空白，D16（发布语义） 没写）。七样已知边角：

- **`T_time` 的下界只适用于非 fsync 触发的发布**：那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用这条下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
- **「向上取整到 200 ms」与「预算取 0.1」都是选的，不是算出来的**：0.2 给 94 ms、0.05 给 410 ms（外推），下界随预算线性挪，改预算就要重算。
- **由 `T_dirty` 推出的两个派生数按已定记录格式算**：D23（journal 的角色与格式） 已定项 12 的 4 KiB 定长记录（头 95 B、每项 56 B、71 项/条）下，同一个 `T_dirty` 满窗是 1847 条记录、占用 **7388 KiB**，I-8.1（环几何够大） 的环下界 F = 2 时 **14776 KiB**、F = 3 时 **22164 KiB**。两个阈值的定案值不变，变的只是这两个派生数的口径。
- **一事务一单元下的记录账与这个派生数打架**：按六个负载点算，每次 fsync 的记录数等于数据单元数（seq 批 1 是 8 条、批 10 是 80 条），journal 恒占用户数据 12.5%，`T_dirty` 满窗 65 536 条记录、256 MiB，是字节表预想的 64 MiB 环的 4 倍。两条口径原样并列，改法要用户定。
- **本机持续写带宽没有实测**（E58（校验和粒度的端到端代价） 那个 2045 MiB/s 是校验和实验里的顺序口径，不是写带宽，不许拿来换算），所以「5 秒能写多少字节」算不出来。
- **`T_dirty` 与 `T_time` 是两种规则**：`T_dirty` 走夹取——有效值 = min(字段, 环长 ÷ F)，不动；`T_time` 走警告——越界不拒绝，只给警告。同一段里两个字段两种规则，不按同一规则读。
- **`T_time` 默认值后期按盘给推荐**（用户 2026-09-20）：要先有一批盘的实测与探测输入，落在 D12（目标介质） 那条线上（第一条布局线是纯 SSD）。记成 `T_time` 默认值的重开条件，留到后期做。

**依据**：

- E70（checkpoint 两个阈值的可行域）：两个阈值的可行域——`T_dirty` 的上界绑环几何但不承重（64 MiB 环就允许 16 GiB 脏数据，实际卡住它的是脏页内存与崩溃丢失窗口），`T_time` 的下界由摊销预算反解，按最差负载 multistream 定而不按主负载 seq 定，取摊销预算 `B = 0.1` 块/操作 ⇒ 反解 547 次操作 ⇒ 197 ms、向上取整记 200 ms，seq 负载一点下界都不需要。
- E44（序号位宽的本机实测与代价）：本机 2785 fsync/秒，是把摊销预算反解成 `T_time` 下界的那个速率。
- E91（已定格式下的环账与准入账）：按已定记录格式重算 `T_dirty` 满窗的 journal 占用与环下界，旧的 32 B/条目口径少算 1.80 倍。
- 用户定案三次（两个阈值、丢失窗口口径的收口、挂载时再夹一次），原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）、C425（T_time 小于一次发布耗时时触发空转）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:181-204`（整段抄，未转述）**

```markdown
#### 已定项 6：发布计数器，四个等号写成一条

**定案**：每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外；根槽的轮转键（D22（单元原子性怎么合成） 已定项 2 的「区域 = txg mod R」里那个 txg）、记账的代（D5（快照 / 空间记账机制） 已定项 2）、恢复重放的水位（D23（journal 的角色与格式） 已定项 14）都是这同一个计数。不另设「发布代号」字段——它与 checkpoint_txg 是同一个东西。**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号，四处读的是同一个数，哪个实现读岔一处就会撞根槽或读错账。

**四条骑手条款**（与定案同日采纳，不是可选项）：

1. **根记录必须带 32 位实例代号**（与 D23（journal 的角色与格式） 已定项 9 的 journal 实例代号是同一个计数），择新在 checkpoint_txg 平局时按实例代号高者赢。读根环时每条根照落点公式（D22（单元原子性怎么合成） 已定项 16）核槽位，第 0 代根照 mkfs 种根的槽另认，一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根、取读根环次序最前的（D22（单元原子性怎么合成） 已定项 24）；公式核还没实现，欠在 C743（重建分配器记新类时不核根住不住这一槽）。字段落点在 D22（单元原子性怎么合成） 已定项 7。
2. **记账 key 的「代」段宽度 ≥ 48 位**：按本机 2785 发布/秒，32 位 17.8 天回绕。记账 key 的宽度算术自陈是假设，要按此重算。
3. **`T_time` 下界的推导（197 ms）只适用于非 fsync 触发的发布**——那步推导以「一窗多次操作共享一次发布」为前提，fsync 密集负载下窗口由 fsync 自身触发。引用已定项 5 的下界时连这句一起引；上界 5 s（用户定的丢失窗口）不受影响。
4. **记账条目的跨发布合并不再可行**（代进了 key，「后者胜」并不掉不同代的两条）⇒ 每次发布把这一代的行全写一遍就是实付值，不是上界。代价，照实记。

**射程**：定的是这个计数怎么加、四处怎么共用一个数，不定根环几何本身（D22（单元原子性怎么合成） 已定项 2）、也不定 journal 实例代号什么时候递增（D23（journal 的角色与格式） 已定项 9）。已知边角一样：这条定案当天的对抗测试实际跑在两条腿上——本地腿那一份后来被收窄后的字词损坏检查判红（`proposed proposed`，实词自复读那一类），按 `.claude/rules/three-way-inference.md`「闸判红之后那一轮作废」，**不许再拿它当「三方一致」引用**；定案本身不因此推翻，两条云端腿各自的判定性产出仍在。

**依据**：

- E48（根环几何的可行点）：轮转键就叫 txg 且逐次发布递增，9 个可行点全部建在这个读法上。
- E50（根环槽数的上下界）：源码逐字「一圈时长（秒）= 槽总数 ÷ fsync 率。每次 fsync 发一次根」——已入库的根环算术只在「每次发布 checkpoint_txg 加一」这条等式下为真，不钉它那批实测追溯性作废。
- E53（丢一整块盘之后根环还挂不挂得上）：丢盘回退 + 盘失而复得的那条历史，就是骑手条款 1（根记录带实例代号）要分辨的东西。
- E44（序号位宽的本机实测与代价）：本机 2785 发布/秒，32 位 17.8 天回绕——骑手条款 2 的「≥ 48 位」是从这个速率来的。
- 对抗测试（用户要求先攻再定）的原始输出 `c85-publication-counter-forward 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-reverse 那一轮的腿报告（产物在 git 里）`、`c85-publication-counter-local.out`；用户定案，原话在变更史。
- 骑手条款 1 的平局（读根环照落点公式核槽位、第 0 代根另认，平局只剩第 0 代同字节的根）：根环三方第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C743（重建分配器记新类时不核根住不住这一槽） 那一格（D22（单元原子性怎么合成） 已定项 7 实例代号那一行与骑手条款 1 同源、一起改；第 0 代例外是第一轮打中裸公式之后补的，第二轮攻过没打中，第三轮辩方站住）。

**欠**：C88（根环的时间线判别未实现）（骑手条款 1 的 checker 侧，反推腿那一轮找出来的可达破绽；并入自 C85（发布计数器的四个等号没写死），条款已写成一条，会红的检查仍欠）；C743（重建分配器记新类时不核根住不住这一槽）：读根环时的落点公式核。

```

**出处 `.claude/kb/decisions/16-发布语义.md:205-221`（整段抄，未转述）**

```markdown
#### 已定项 7：发布的持久顺序

**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进崩溃点重放枚举，一次发布的段序列因此唯一。**抬 F 那一串（准入与卸载）在它的第一次发布之前多一步**：先把新 F 写进每块盘的系统配置、过一道屏障（已定项 1「抬 F 那一串」）；这一次系统配置写不属于任何一次发布，每次发布内部的顺序照旧。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回（做法与次数在已定项 8）；管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），它的根持久之后才向管理员确认，与 fsync 同一个返回条件；实现待派，排在实审 A3c 之后（同在 `transaction.rs`），代价另登记一个小实验量。

**射程**：定的是一次发布内部这几步的先后与 fsync 什么时候返回，不定屏障的真机代价、也不定根槽本身的冗余形态（D22（单元原子性怎么合成） 已定项 2）。三样已知边角：

- **「fsync 等系统配置轮换持久之后才返回」的射程**：返回之后这一代的根只由那一个根槽罩着（根槽不镜像），见证它的系统配置已落盘。同一实例里退一代，靠 journal 重放追得上；**每个新实例（每次可写挂载、恢复、切换）的第一个根在下一次发布之前是单点**——那个槽读不出或它所在的盘掉了，恢复退到上一个实例的根，重放按严格前缀停在实例边界（D23（journal 的角色与格式） 已定项 14 的注 1），这次挂载里 fsync 已返回的事务丢，一个故障就够。要不要让新实例先把根写到两块盘上再确认，是已定项 8。
- **屏障口径**：两道 FLUSH + 根槽 FUA + 系统配置轮换之后一道屏障 = 每次发布四个序点，比 D25（目标负载优先级） 推导的两个多两个——多的两个是第二道屏障与「发布返回前加屏障」的价钱，知情接受（后者用户 2026-09-27 定，起因是 C577（系统配置没见证到的最新根，乙罩不到） 那条用例在抬 F 之前更早红：发布路径根槽 FUA 之后轮换系统配置就返回，没有屏障）。
- **第二道屏障买的是记录流完整性**，不是数据完整性：数据完整性的必要屏障恰一道（根槽之前）；不上第二道有 7 类「根在案而记录缺席」的状态，记录核对器与反向链的输入有洞。用户定案「需要」，两道都上。

**依据**：

- E77（发布的持久顺序）：崩溃子集穷举——不上根槽之前那道屏障，1024 个崩溃状态里 504 个违例；拿掉「重放施加前验证点名单元」这一步，b_rs 臂 63 处静默嫁接，所以它是承重步骤。
- 用户定案，原话在变更史；「发布返回前加屏障」（系统配置轮换之后那一道，序点从三个变四个）是用户 2026-09-27 定的，起因、弹窗事实与岔路在 `c554-fix-forks.md` 第 4 行，实现已合入（`crates/singlefs-core/src/transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`，实 C577（系统配置没见证到的最新根，乙罩不到） 报告 `m2-impl-c577-barrier-implementer-report.md`），代价仍另登记小实验量。

**欠**：C26（屏障数从没量过）（真机每次 fsync 的屏障数仍没量）。崩溃点重放 harness 的两条自证用例（摘掉根槽前的屏障必须红、关掉重放验证必须红）随 C76（发布时序没有权威落点） 已还清。

```

**出处 `.claude/kb/decisions/16-发布语义.md:222-243`（整段抄，未转述）**

```markdown
#### 已定项 8：暖机取戊，新实例的根先覆盖两块盘

**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回；做法是连推空发布，第一版几何至多 R = 3 次。**第一版实付 2 次，而且它是格式常量不是运行时谓词**：根环三个区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7），于是 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定。登记成 `format-const` WARM_UP_EMPTY_PUBLISHES = 2 与 NEW_POOL_FILE_CREATION_TXG = 3（[layout/01-first-txn.md](../layout/01-first-txn.md) 八），条款在 D22（单元原子性怎么合成） 已定项 16 第 5 句。**后续挂载与挂载中途崩了再挂**：次数现算——写行那次发布的 txg 取 D23（journal 的角色与格式） 已定项 14 注 3 那个下界本身（max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1），之后每次空发布加一，按根环落点看落哪块盘，推到本实例的根覆盖每块盘为止，至多 R 次；取号之前的准入与取号之后推发布用同一份计划（`crates/singlefs-core/src/mount.rs` 的 `warm_up_publish_txgs`）。

**射程**：定的是新实例确认之前要做什么，不定实例切换本身的判据（D23（journal 的角色与格式） 已定项 14）、不定空发布写出多少单元（已定项 9）。四样已知边角：

- **前置 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 的条款那一半已满足**（用户取影子账，条款写进 D23（journal 的角色与格式） 已定项 14 与 I-7.4（近 K 代块未被复用）），暖机已生效；**检查那一半仍欠**：多次挂载的录制流已有（崩溃点重放覆盖写固定脚本流），逐句核仍差格 1 的专门断言，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 因此仍在未还表。
- **归属改了那两个格式常量要重算**（加盘或开条带表时）；「至多 R = 3 次」那句仍是几何上界，不是第一版走得到的次数。第一版实付 2 次这一档没有走三方论证，可推翻。
- **代价**：改已定项 7 的 fsync 返回条件（新实例确认前多一个条件）；新池新建文件的 checkpoint_txg 从 1 变 3，多两条空记录与两次根写；D28（挂载期承诺量） 已定项 3 的切换预留加「每次切换至多 R = 3 次空发布」，按 D28（挂载期承诺量） 已定项 4 现算的 c_max 取（池规模 9 块 ⇒ 27 块）；空发布占根环槽位，把旧的非空根更早挤出环。
- **另两臂**：甲（不补）判据 2 一个故障就丢；丁（重放跨实例边界）出局。
- **白推**：写行 txg ≡ 2 (mod 3) 时第一次空发布落在已经覆盖的盘 0 上，白推一次——副本装置上量到每次多写 147968 字节、多 2 道屏障、多 1 次 FUA、多占根环一槽，只重开不写时每次挂载都落在这一档，24 槽的根环 8 次挂载就整圈转完（入库装置上没重做）。这是写行 txg 取下界本身的直接后果，认下；「普通挂载的写行 txg 往后挪到只推 1 次的值」那条改法被攻过零轮，没采纳。

**依据**：

- E150（回退复用被抛弃的根引用的单元）：无暖机时 1 个故障就落到被抛弃的根上，暖机之后要 4 个；影子账与墓碑两种修法都零违例，用户取影子账。
- E148（提交固定点按两棵记录树重算）：空发布在池规模、提交块聚簇时 9 块，切换预留那一半按它算。
- E142（新池新建文件的干跑）：checkpoint_txg 1 → 3 的改动落到新池新建文件的字节上之后，262162 个状态零违例。
- 两轮三方论证 `research/prompts/c199-r2-main-verification.md`；用户定案，原话在变更史。
- 后续挂载那一句与白推：三方第一轮 `research/prompts/m2-presumed-clauses-r1-main-verification.md` K2（推到覆盖为止在后续挂载、回退、挂载中途崩了再挂三种起点上都构造不出不够），用户 2026-09-23 定照当时的实现写、认白推，原话在变更史。

**欠**：C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（检查那一半）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:244-258`（整段抄，未转述）**

```markdown
#### 已定项 9：一次空发布也写单元

**定案**：记账树存在时就写——空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2（每条根的记账树只装这条根那一代的行，每次发布把这一代的行全写一遍）重写记账行，连带记账树节点、分配记录、映射条目与树表单元。第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。**树表 0 条的一版上写行的那次发布写两个单元**：实例表与一片分配记录节点，分配记录树根指针住根记录（D22（单元原子性怎么合成） 已定项 7）；被换下的上一版实例表与上一片分配记录节点记成已释放，重开时从这片节点重建这一版的账。根记录那一项只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0，零单元发布照抄上一版。

**射程**：定的是空发布写不写单元、写哪几样，不定 c_max 怎么算（D28（挂载期承诺量） 已定项 4 按当时结构现算）、也不定空发布什么时候推（推抬 F 在已定项 1，暖机在已定项 8）。写行那次发布写的那片分配记录节点，单元头树 ID 写 0（实现的取法：那一版的树 ID 水位是 mkfs 的 65536，写分配记录树的 65538 会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 当场红），没有条款；它进不进中央映射也没有条款——那一版没有映射树，D19（块指针的结构与宽度预算） 已定项 8 的自举豁免三类里没有它。

**依据**：

- E148（提交固定点按两棵记录树重算）：空发布的固定点按两棵记录树重算是池规模 9 块、新池新建文件规模 4 块，「以后的空发布按现算的 c_max 块」那句的数出自这里。
- E142（新池新建文件的干跑）：第一次可写挂载的暖机那两次空发布逐字节写出来，树表 0 条时零单元。
- 用户定案 2026-09-23（写行那次发布在树表 0 条的一版上写实例表与分配记录节点两个单元、分配记录树根指针住根记录），原话在变更史；实现与用例点名在 [checks-owed.md](../checks-owed.md) 已还清表 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 那一行。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:259-274`（整段抄，未转述）**

```markdown
#### 已定项 10：发布粒度取攒批，不是每个事务发一次

**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。

**射程**：定的是频率，不定一次发布事件内部的顺序——那在 `litmus/commit-publish.litmus`，它钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。两样已知边角：

- **净代价分布**：格式侧几乎为零（journal 本来就要有），验证侧一大笔——中间态语义改写（已定项 12）、第二份 replay 实现（已定项 4 与 13），再加 D13（验证路线） 的「Merkle 根链差分」粒度变粗：一代根里塞了 K 个事务的改动，定位精度从「哪个事务」降到「哪个 checkpoint」，而 D13（验证路线） 已写明全量档的真代价是定位成本。
- **发布语义决定「走几次」**：每事务发根 → K 批走 K 次；checkpoint → 同一窗口内的 K 批只走约一次，各批只留一条 journal 记录（小、顺序写）⇒ 分批大删除那一格的元数据写放大从 `K × M × H` 降为 `checkpoint 数 × M × H`。

**依据**：

- 无实验：定案靠两条互相独立的推理，当时没有可量的量。① 它与 D5（快照 / 空间记账机制） 已定的 ZFS birth txg 模型天然相容——ZFS 的 txg 本身就是 checkpoint（`zfs_txg_timeout` 默认 5 秒，或攒够一定脏数据量），一个 txg 批量大量操作、txg 结束才更新一次 uberblock；采纳 checkpoint 之后 birth txg 的号粒度问题自动消失，一个 txg 一个号，deadlist 的比较粒度就是 checkpoint 粒度、不必重算（未在本项目验证）。② **根是丢失写的最终见证者**：检测「盘上这块是不是它应该是的那个版本」需要一个外部见证者，块自身办不到（合法的旧版本与合法的新版本在块内层面同构，D18（块里携带什么信息） 有完整论证），而见证者必须在不同的失败域上——父节点若在同一次事件里也没写成就要祖父，递归到根 ⇒ checkpoint 发布的那个根是这条递归的终点，它的持久化必须比任何单个块更可靠（ZFS 的形态是 4 个 label × 每个 128 KiB 的 uberblock 环、且偶数与奇数 label 分两阶段写；本工程根自身的冗余形态由 D22（单元原子性怎么合成） 已定项 2 定）。频率的可量代价（两个阈值的可行域）在已定项 5。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:275-287`（整段抄，未转述）**

```markdown
#### 已定项 11：没有任何操作可以绕过 journal 直接发根

**定案**：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏。fsync 的效果是「提前触发一次发布」，**不是另开一条日志格式、另一套恢复逻辑**。判据是：journal 是所有提交的必经之路，还是某些路径的旁路——前者是一套机制、持久化分两级，合规；只有 fsync（或只有元数据、只有小写）走 journal、别的直接发根，就是 `.claude/rules/fs-design.md` 点名的「为 fsync 单开一套日志树」。

**射程**：说的是挂载之后的操作；mkfs 种子写第 0 代根不经 journal（D13（验证路线） 已定项 1 自己列的根槽三种写者之一，mkfs 一条记录都不写），不在这句的射程里。已知边角一样：D23（journal 的角色与格式） 已定项 1 的轴一取甲压在这一条上，重开它要回答的依据里有这一条。

**依据**：

- 无实验：它是 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」在发布这一格的形态，判的是机制的形状（必经之路还是旁路），不是一个能量出来的数；不写死就会退化成「为某个路径开第二套事务机制」。每次持久化的写量是另一个问题，归 D23（journal 的角色与格式） 已定项 1。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:288-300`（整段抄，未转述）**

```markdown
#### 已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」

**定案**：D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写为「**根 + journal 任意前缀，replay 之后合法**」。

**射程**：定的是合法性怎么判，不定前缀本身怎么截——前缀判定的完整口径是 D23（journal 的角色与格式） 已定项 14 的五条，缺一不可。已知边角一样：块复用那一侧的真正约束比 checkpoint 边界更严，按 journal 位置说（已定项 15 与 D22（单元原子性怎么合成） 已定项 4）。

**依据**：

- E42（一事务几条记录）：1601 个前缀里 1400 个被前缀切开（87.4%），一事务一条那一臂非法前缀恒 0——那是「任意前缀 replay 之后合法」的直接形式；跨多条而无边界字段时非法前缀恰等于被切开的前缀数，加边界字段之后 0。这条语义在模型上是可数的，而且数得出它什么时候破。
- 用户定案，原话在变更史。

**欠**：C42（残留记录冒充合法前缀）（上一条时间线留在环里的记录会被下一次恢复当作合法前缀重放）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:301-313`（整段抄，未转述）**

```markdown
#### 已定项 13：这份 journal replay 归记录核对器，不归 O2（独立解析器 + checker）

**定案**：D13（验证路线） 已定 O2（独立解析器 + checker） 是一元谓词、只判单个镜像，所以这份 journal replay 归「记录核对器」。那是第二次实现 replay，而 replay 正是崩溃语义最容易出**共同误解**的地方——D13（验证路线） 的「三 oracle 合谋测试」要防的就是这个。**这是 D16（发布语义） 最贵的一笔。**

**射程**：定的是归谁，做什么与允许共享什么在已定项 4。已知边角一样：D23（journal 的角色与格式） 轴一定甲之后「redo 为空」，最贵那一笔的大头随之消失，只剩前缀判定 + 择根 + 续做。⚠️ **「redo 为空」只对「根已持久」的发布成立**：E42（一事务几条记录） 实测 1601 个前缀里 1400 个被前缀切开（87.4%），被切开的比例与边界字段在不在无关——变的是这 1400 个非法还是合法（无边界字段臂 1400 非法，有边界字段臂 0）⇒ **恢复期重放非空是常态，不是边角。**

**依据**：

- 无实验：哪个判决器实现 replay 是验证路线的分工，没有可量的量；第二份实现的成本量级与它必须独立实现什么在已定项 4。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:314-326`（整段抄，未转述）**

```markdown
#### 已定项 14：快照发布点与 checkpoint 边界对齐，钉在全部数据变更之后

**定案**：快照发布点必须与 checkpoint 边界对齐，且位置钉死在「本 checkpoint 全部数据变更之后」，快照不许出现在 checkpoint 中间。同一 checkpoint 内建两个快照是允许的：它们捕获同一状态、其间 deadlist 为空；但 checker 里的「上一个快照」要用 `(txg, 快照 id)` 打破并列。

**射程**：定的是快照点落在哪，不定快照本身的记账模型（D5（快照 / 空间记账机制） 与 D6（快照实现模型））。已知边角一样：这条规则的可执行形态是已定项 5 那段丢失窗口口径——发布阶段等全部单元落盘，快照点就落在那之后。

**依据**：

- 无实验：理由是机制性的，没有可量的量——`birth` 的分辨率只到 checkpoint，**块无法表达「我生在本 checkpoint 里快照点之前还是之后」**；不钉死它，D5（快照 / 空间记账机制） 那张边界表里 `birth == S.txg` 与 `death == S.txg` 两行会同时失去确定答案，而它们一个是**数据丢失**、一个是**空间泄漏**。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:327-346`（整段抄，未转述）**

```markdown
#### 已定项 15：checkpoint C 中产生的释放，C 发布之前不进可分配集合

**定案**：checkpoint C 中产生的一切释放，在 C 被发布之前不得进入可分配集合。两个理由，第二个是 D16（发布语义） 独有且更严的：

1. **回滚**：崩溃后可能回到 C−1 的根，而生于 C 之前、死于 C 的块在那时仍然活着。
2. **已定项 12 把 D8（核心索引结构） 的中间态语义改写成「根 + journal 任意前缀，replay 后合法」**：若一个块在 journal 位置 p 被释放、位置 q > p 被重新分配并写入新内容，那么重放到 `p < r < q` 的前缀会得到一棵指向该地址的树，**而盘上的字节已经是 q 的内容了**（物理写不受 journal 位置约束）。**这不是理论风险，是必然的错。**

**射程**：定的是「什么时候可以回可分配集合」这条下界，不定窗口取多大（已定项 1）。三样已知边角：

- **第二条理由是按 journal 位置说的** ⇒ 真正的约束是整个 journal 重放窗口，不是 checkpoint 边界；D22（单元原子性怎么合成） 已定项 4 收在那一形，缺了它的失败模式是「可检测（父指针校验和失配），不可恢复」。
- **窗口容量与 ENOSPC 直接相关**（D3（空间分配）：释放空间这个操作本身不许申请空间），而攒批让「建了又删」在同一窗口内闭合的概率大幅上升 ⇒ **defer 窗口落在热路径上**。
- **外部线索**：ZFS 有同型机制且更保守，`include/sys/txg.h` 的 `#define TXG_DEFER_SIZE 2 /* Number of txgs worth of frees we defer adding to in-core spacemaps */`，**未在本项目验证**；⚠️ 那个 2 为什么是 2，没查到一手解释，本工程的 defer 窗口要按自己的重放语义推，**不许照抄**。

**依据**：

- E82（准入的在飞合成）：把窗口内产生的释放记成可用的那条臂，20 格吃进 defer 该扣住的块——这条规则被静默绕过时的现象是可数的，准入侧的形态因此写进了 D28（挂载期承诺量） 已定项 2。
- 用户定案，原话在变更史。

**欠**：无（块复用那一侧的会红检查由 I-7.4（近 K 代块未被复用） 承担，欠账在已定项 1 列的那几笔）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:347-359`（整段抄，未转述）**

```markdown
#### 已定项 16：checkpoint 与消息缓冲买同一样东西，E7（离线索引 harness） 的对照组钉死

**定案**：checkpoint 与 D11（索引节点要不要留消息缓冲区） 的消息缓冲区买的是同一样东西，差别只在**摊销窗口住在哪**：checkpoint 的窗口住**内存**、不进格式；消息缓冲的窗口住**盘**、必须进格式。⇒ **D11（索引节点要不要留消息缓冲区） 的边际收益 = 「跨 checkpoint 的摊销」减去「checkpoint 内摊销」**。**E7（离线索引 harness） 的对照组必须是「有 checkpoint、无消息缓冲」。**

**射程**：定的是那个对照组怎么设，不定 D11（索引节点要不要留消息缓冲区） 取哪一边——那由 D11（索引节点要不要留消息缓冲区） 自己定。已知边角一样：BetrFS 那组数字里这两块从没被拆开过（它同时有 4 MiB 节点的消息缓冲**和** 5 秒 checkpoint），**未在本项目验证**。

**依据**：

- 无实验：这是「对照组怎么设」的规定，不是实验结论；文献里没有一个对照组带 write buffer 前端，消息缓冲相对本工程现方向的边际收益，现有文献一个数字都没给，本项目也还没有可引的量。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/16-发布语义.md:360-382`（整段抄，未转述）**

```markdown
#### 已定项 17：fsync 报错之后，调用方能假设什么

**定案**：

- fsync 报错之后，这次发布的数据**可以出现也可以不出现**——调用方不许把「报错」读成「这次没成立」。
- **报错之后不许假设可以照旧复用同一份内存态重试。**（`close(2)` 的 NOTES 段明写「Retrying the close() after a failure return is the wrong thing to do」；本工程取同一条语义。）
- 要知道到底成没成立，调用方自己去读现行根：`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`、`choose_root` 对着仍打开的设备句柄就判得出，判据是「这次的 checkpoint_txg 是不是现行根」，不用卸载重挂。`transaction.rs` 没有调它们的路径，缺的是一段胶水。
- 这一条不定 fsync 什么时候返回，那是已定项 7。

**射程**：

- **只在「转只读」与「重做又失败」那几格上生效。** `D23`（journal 的角色与格式） 已定项 14 失败表判「切换」的那几格，调用方根本收不到错误（实例切换会把在飞 checkpoint 重发掉），这一条对那几格无话可说。
- 管的是「报错之后调用方能假设什么」，不管失败本身怎么分流（那是 `D23`（journal 的角色与格式） 已定项 14），也不管持久顺序（已定项 7）。

**依据**：

- 无实验：这一条是对外语义的承诺，没有可量的量。撑它的是 `research/prompts/c381-r3-main-verification.md` 第一节 K1 那一格——H1（第六步系统配置槽轮换失败、前五步全成）上，「报错即没成立」这条语义**本身就是 `C381`（根已落盘之后发布失败，分配器仍退回） 的成因之一**：它让调用方以为没成立、拿旧状态接着发。
- `research/prompts/c381-r3-main-verification.md` 第三节（候选甲站住，两样收严的来处：K2 的 POSIX 线索、K4-e 的射程限定）。
- 用户定案 2026-09-21，原话在变更史。
- ⚠️ **依据里要写明**：定案的后两句（禁止性条款、射程限定）**被攻过零轮**（`.claude/rules/three-way-inference.md`：攻方腿自己提的收严算没被攻过），用户 2026-09-21 知情接受。

**欠**：`C381`（根已落盘之后发布失败，分配器仍退回）（改法落地之前这一格还开着）。

```

**出处 `.claude/kb/decisions/02-raid-stripe-policy.en.md:1-6`（整段抄，未转述）**

```markdown
## D2 RAID Stripe Policy —— Semi-settled (1 item unsettled)

[中文](02-RAID条带策略.md) · **English** · [日本語](02-RAID-ストライプ方針.ja.md)

D2 (RAID Stripe Policy) governs the striping aspect: how wide a stripe is, which disks it uses, how a unit is placed into a stripe, where the membership table lives, how partial release and degradation are handled, what path changes to device sets follow, and the granularity of writes issued to devices along with slot spacing for fixed structures. Out of scope: fields and widths of position entries themselves belong to D19 (Block Pointer Structure & Width Budget) settled item 1 / settled item 4; slot geometry of root slots and system configuration slots, and address space of attribution formulas belong to D22 (Unit Atomicity Synthesis) settled item 2 and settled item 16; media branches and the "device → layout" table belong to D12 (Target Media); which slot a write lands in belongs to D3 (Space Allocation) settled item 8; admission inequalities belong to D28 (Mount-Time Commitments) settled item 1.

```

**出处 `.claude/kb/decisions/02-raid-stripe-policy.en.md:7-352`（整段抄，未转述）**

```markdown
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

```

**出处 `.claude/kb/decisions/02-raid-stripe-policy.en.md:353-364`（整段抄，未转述）**

```markdown
### Unsettled Items

| # | Item | Status |
|---|---|---|
| 21 | **Parity slot count** | How many parity slots in a stripe has no clauses; version 1 2-disk strictly `w = 2` ⇒ geometrically strictly 1 slot, but that is forced by geometry. Modifies new-pool file creation bytes: No (2026-09-20, basis: version 1 2-disk mirror implemented as `parity = 1`, no alternative value writes different bytes; what requires settling is slot count when `w ≥ 3` opens, where branches first appear) Modifies format: Yes (2026-09-21, basis: scope states verbatim "upper bound resets with parity"—`w_max` byte in system configuration varies with this item's answer; parity=2 yields 7 under equal weight arithmetic, while settled item 18 currently takes 4) **Status: Unsettled.** **Progress: Unimplemented.** |

#### Unsettled Item 21: Parity Slot Count

**Topic**: How many parity slots in a stripe (`parity`). Version 1 2-disk strictly `w = 2`, geometrically forced to 1 slot; how many slots when `w ≥ 3` opens has zero clauses repo-wide. **Path for version 1**: Unsettled, implemented as `parity = 1` (2-disk mirror, `w = 2`). **Modifies new-pool file creation bytes: No**—version 1 has 2 disks, `w` strictly 2; `w_max` / `g` bytes in system configuration uniquely determined by settled item 18; this item alters input when resetting those bytes in future, not modifying bytes written by new-pool file creation.

**Scope**: Revisit status of settled item 18 before settling: `w_max` = 4 derivation had dimensional error (corrected equidimensional arithmetic yields 6 under equal weight), equal weighting lacks source in kb, making 4 a conservative choice accepted with awareness, not pinned by that arithmetic; under parity=2, dual disk failure loses no data, risk axis shifting to triple failure, yielding 7 under same arithmetic ⇒ upper bound resets with parity. **Cascade with D9 (Encryption)**: Parity computed over ciphertext; multiple replicas write identical ciphertext N times—encrypting N times yields N distinct byte streams. **For reference (unadopted)**: bcachefs erasure-codes whole buckets in background: foreground writes normal multi-replica, background aggregates sufficient candidate buckets then recomputes parity once, atomically updating all pointers; parity computed once over immutable data, likewise eliminating write hole; cost is buckets cannot be reused as long as live data remains in stripe; if D8 (Core Index Structure) adopts bucket model in future, D2 (RAID Stripe Policy) should re-evaluate.

```

**出处 `.claude/kb/decisions/02-raid-stripe-policy.en.md:365-368`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D2 (RAID Stripe Policy) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/02-RAID-ストライプ方針.ja.md:1-6`（整段抄，未转述）**

```markdown
## D2 RAID ストライプ方針 —— 半確定（1 項目未確定）

[中文](02-RAID条带策略.md) · [English](02-raid-stripe-policy.en.md) · **日本語**

D2 (RAID ストライプ方針) はストライプの枠を管轄する：ストライプの幅、使用するディスク、ユニットをストライプへどう配置するか、メンバー表の格納場所、部分解放およびデグレード時の処置、デバイス集合変更時の遷移経路、ならびにデバイスへ発行する書き込みの粒度と固定構造のスロット間隔。管轄外：位置エントリ自体のフィールドと幅は D19 (ブロックポインタの構造と幅予算) 確定項目 1 / 確定項目 4；ルートスロットおよびシステム構成スロットの幾何、帰属計算式のアドレス空間は D22 (ユニット原子性の合成方法) 確定項目 2 および確定項目 16；メディア分岐および「デバイス → レイアウト」表は D12 (目標メディア)；書き込みがどのスロットへ配置されるかは D3 (領域割り当て) 確定項目 8；受入不等式は D28 (マウント時コミットメント量) 確定項目 1。

```

**出处 `.claude/kb/decisions/02-RAID-ストライプ方針.ja.md:7-352`（整段抄，未转述）**

```markdown
### 確定項目

**D2 (RAID ストライプ方針) の確定項目は単一の番号体系を使用し**、「未確定項目」節と連番であり、番号の振り直しは行わない。

| # | 項目 | 決定事項 |
|---|---|---|
| 1 | **「可変幅」の粒度** | 書き込みごとの可変：フォーマット側で物理位置を個別に列挙し各々デバイス識別子を保持、chunk マッピングテーブルは導入しない；割り当て側もグループ内固定を採用しない **状態: 確定。** **進捗: 実装済み。** |
| 2 | **各ディスクの容量均一性の要否** | 均一である必要はなく、アドレッシングの正当性という部分問題に限定；プールメンバーと各ディスクの幾何はマウント時に検証し、安全な組み合わせと判定できなければマウントを拒否 **状態: 確定。** **進捗: 実装済み。** |
| 3 | **デバイス集合変更後の幾何の再評価方法** | 明示的な「幾何再評価」トランザクションによりアトミックにパブリッシュ、ボリュームはマウントを維持し強制再マウントは禁止 **状態: 確定。** **進捗: 未実装。** |
| 4 | **幾何再評価トランザクションの形態** | 幾何不合格時はディスク追加を拒否、ドレイン後に幾何を切り替え、ディスク追加後はバックグラウンドの再開可能なリバランスを実行、移行単位はオブジェクト全体 **状態: 確定。** **進捗: 未実装。** |
| 5 | **デバイス集合変更後にルートリングの障害ドメインが無効化された場合の対処** | ルートリングは再配置しない；リバランスはユーザーデータを移動しルートリングには触れず、devs < R への低下時は縮退を受容 **状態: 確定。** **進捗: 未実装。** |
| 6 | **アロケータが書き込みごとに使用する列数** | `w = clamp(バッチ後書き込み量 + 1, 2, 4)` の後さらに当時書き込み可能なデバイス数でクランプ、スロット幅ごとにバケット分割；上限 4 はシステム構成で宣言された定数 **状態: 確定。** **進捗: 実装済み。** |
| 7 | **ルートリング帰属計算式における devs の決定** | 帰属は計算せず、mkfs 時に領域ごとにデバイス識別子を明示保存；第 1 版の 3 領域は 0 / 1 / 0 に固定、mkfs パラメータではない **状態: 確定。** **進捗: 実装済み。** |
| 8 | **どのデバイスが参加するか（サブセットの選定方法）** | オブジェクト作成時に最も空いている g 台からグループを選定、以後はグループ内で最も空いている w 台のみを選定；グループ内のディスク満杯時に再選定、グループはディスクに保存しない **状態: 確定。** **進捗: 未実装。** |
| 9 | **第 1 版で稼働させるディスク台数** | 最初の実行可能ターゲットは 2 台で稼働；単一ディスクは独立した開発ラインとし、その開通時に決定 **状態: 確定。** **進捗: 実装済み。** |
| 10 | **ユニットが w 列に跨がるか単一列に収まるか** | 単一列に完全に収まり、分割も跨がりもしない；位置エントリはレプリカごとに 1 つ **状態: 確定。** **進捗: 実装済み。** |
| 11 | **ストライプのライフサイクル（部分解放時の対処）** | 部分ストライプはピン留め固定しバックグラウンドで遅延再配置；修復完了までメンバー枠は再利用禁止、w = 2 のストライプはメンバー 1 つの破棄で全体を回収 **状態: 確定。** **進捗: 未実装。** |
| 12 | **パリティ列の位置と検出メカニズム** | メンバー表は固定長 56 バイトでコード 3 パッキングレコードに格納、コンテナインデックスで指名、コンテナチェーン全体は常に `w = 2` ミラー **状態: 確定。** **進捗: 未実装。** |
| 13 | **デグレード期間中の書き込み許可または読み取り専用** | 読み取り専用：書き込み可能マウントはプール内の過半数の排他オープンを要求、稼働中に w 下限未満または過半数未満に低下した時点で読み取り専用へ移行 **状態: 確定。** **進捗: 未実装。** |
| 14 | **ストライプテーブル論理ツリーのキー選定** | 配置先（デバイス識別子 4 + 16 KiB スロット番号 6）を採用、D3 (領域割り当て) 確定項目 3 の割り当てレコードツリーと同一座標系 **状態: 確定。** **進捗: 未実装。** |
| 15 | **「ライフサイクルの異なるオブジェクト」の一般的判定基準** | 破棄トリガーイベントが常に同一コミットであれば同一ライフサイクルとし共有を許可；確定項目 17 の適用範囲を io_min 内の固定構造へ狭小化 **状態: 確定。** **進捗: 対象なし。** |
| 16 | **`io_min` 未満の書き込みを発行しない** | デバイスの物理マッピングユニットへより小さな書き込みを発行すると内部 RMW を誘発；この量は `physical_block_size` ではなく `io_min` **状態: 確定。** **進捗: 実装済み。** |
| 17 | **ライフサイクルの異なる 2 つのオブジェクトによる同一物理マッピングユニット共有の禁止** | 判定基準は確定項目 15 に従う；適用範囲は io_min 内の固定構造のみをカバーし、16 KiB ノードと 32 KiB ユニットは対象外 **状態: 確定。** **進捗: 実装済み。** |
| 18 | **システム構成における w_max と g の第 1 版値（4 または 2）** | 双方のバイトとも 4 を書き込み、g にランタイムクランプを追加；第 1 版のディスク追加受入は `w ≥ 3` に到達可能なデバイス集合変更を拒絶、devs 上限は 2 **状態: 確定。** **進捗: 実装済み。** |
| 19 | **ルートスロットおよびシステム構成スロットのスロット間隔** | スロット間隔 = 4096 を mkfs 検出の io_min の整数倍へ切り上げ；io_min はマウント時比較リストに入り、第 1 版は 2 つのゲートのうち (a) のみ設定 **状態: 確定。** **進捗: 実装済み。** |
| 20 | **ストライプテーブルとコンテナインデックスの day-1 ツリーテーブル登録要否** | 登録しない；`w ≥ 3` 開通時に新規ツリー ID と新規種類コードを取得 **状態: 確定。** **進捗: 実装済み。** |
| 22 | **全幅ストライプ書き込み、リード・モディファイ・ライトの完全排除** | ストライプ幅は可変、毎回の書き込みは全幅書き込み；write hole は修復されるのではなく存在しないよう設計 **状態: 確定。** **進捗: 実装済み。** |

#### 確定項目 1: 「可変幅」の粒度

**決定事項**: 書き込みごとの可変。フォーマット側では**物理位置を個別に列挙し各々デバイス識別子を保持**し（D19 (ブロックポインタの構造と幅予算) 確定項目 1）、「論理アドレス + chunk マッピングテーブル」という間接層を導入しない；割り当て側も同様に書き込みごとの可変とし、「グループ内固定」を採用しない。

**適用範囲**: 粒度と「第 2 の権威構造を持ち込まない」ことを決定し、書き込みごとに何列使うか（確定項目 6）やどのディスクが参加するか（確定項目 8）は決定しない。根拠は性能ではない：本プロジェクトはストライプ幅の性能を測定したことがなく（`.claude/kb/experiments.md` でストライプに関係する唯一の E55 (ストライプマッピング算術の実デバイス検証) はマッピング算術の正当性検証）、`.claude/rules/fs-design.md` も判定基準に「この方が高速」と書くことを禁じている。グループ内固定の唯一のフォーマット上の利益（ポインタに dev を含める必要がなく chunk マッピングテーブルを使用可能）は D19 (ブロックポインタの構造と幅予算) 確定項目 1 が独立して主張済みであり、D19 (ブロックポインタの構造と幅予算) の 3 つの根拠（D1 (データ移動性 / 逆インデックス) がデバイス削除コストを支払い済み／dev なしは `sys_chunk_array` のようなブートストラップ構造が必要／幅広ポインタの低コスト）はいずれも D2 (RAID ストライプ方針) の粒度を引用していない。

**根拠**:

- 三者検証（背景資料 `_d2-item1-background.md`、`d2-item1-local-counterexample.md`、ローカル脚生出力 `d2-item1-local.out`）：フォーマット側に有効な反対意見なし；逆推脚が [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) および [checks-owed.md](../checks-owed.md) を調査したところ「同一グループ内の幅一致」に依存する項目は存在せず、btrfs が `num_stripes` / `type` / `stripe_len` を chunk（`include/uapi/linux/btrfs_tree.h` の `struct btrfs_chunk`）に持たせている唯一の利点はフィールド削減であり、`.claude/rules/fs-design.md`「誰がより節約できるかの比較は判定基準にならない」に基づき考慮しない；順推脚は chunk マッピングテーブルが**第 2 の権威構造**であり、ポインタと不一致になり得、割り当てと同一トランザクションでアトミック更新が必要であると指摘し、checker の判定可能性とクラッシュポイント再生の有界性の 2 点において個別列挙の方が優位であるとし、同時にグループ内固定が「当時書き込み可能なデバイス集合」という変動入力の読み取り点を書き込みごとからグループ開設時に圧縮できる利点も挙げ、それは確定項目 6 に収容；ローカル脚の 2 つの反例は「論理オフセットから stride 算術で物理位置を逆算する」という誤った前提に基づいており、不採用。
- ユーザー決定事項（割り当て側も書き込みごとの可変を採用）、原文は変更履歴にある。
- 実験なし: ストライプ幅の粒度には定量可能な量がなく、「chunk マッピングテーブルは第 2 の権威構造である」というメカニズム論証が承重している。

**未済**: なし。

#### 確定項目 2: 各ディスクの容量均一性は不要

**決定事項**: プール内の各ディスクは容量が均一である必要はない。プールメンバー集合と各ディスクの幾何は**マウント時に**検証し、安全な組み合わせと判定できなければ**マウントを拒否**する。

**適用範囲**: 「アドレッシングの正当性」という部分問題においてのみ成立する。全幅書き込みにおいて小容量ディスクが先に満杯となった後の「当時書き込み可能なデバイス集合」の選定方法やストライプの縮退方法は確定項目 2 ではカバーされず、それら 2 つの入力は確定項目 6 および確定項目 8 に帰属し、確定項目 2 で上書きしてはならない。

**根拠**:

- 2 つの確定条項の連言：D12 (目標メディア) 確定項目 3 の判定基準原文通り「**ある幾何量がプール内で異種混在可能かどうかは、それを使用するアドレス算術が既にデバイス識別子を保持しているかによる**」とし、D19 (ブロックポインタの構造と幅予算) 確定項目 1 は「位置エントリはデバイス識別子を**保持する**」と決定済み ⇒ 物理位置は個別に明示列挙され、幾何計算式から位置を逆算する余地はない。
- 外部の事実（2026-08-27 Linux 6.17 調査、本プロジェクト内では未検証）：md/raid0 の 5 年間にわたるサイレントデータ破損の引き金となった連鎖はまさに「デバイス識別子を持たず、計算式で逆算した」ことにあった。
- 実験なし: 本項目は 2 つの確定条項の連言であり、本プロジェクト内に定量可能な量はない。

**未済**: なし。

#### 确定項目 3: デバイス集合変更は明示的な「幾何再評価」トランザクションをたどる

**決定事項**: ディスクの追加 / 交換 / 脱落の後、マウント時に算出した幾何は**明示的なトランザクション**によって再評価されアトミックにパブリッシュされ、**ボリュームはマウントを維持**する；「強制再マウント」は採用しない。

**適用範囲**: 方向性を決定し、そのトランザクションの具体的な形態は確定項目 4 にある；排除された第 3 の結末（ランタイム幾何変更ウィンドウの発生、`.claude/rules/fs-design.md`「分岐変数は操作の途中で変更してはならない」に抵触）は確定項目 4 b の排他セマンティクスにより閉じられ、確定項目 3 には依存しない。ディスク脱落の枠は[pitfalls.md](../pitfalls.md) 第 9 条（degraded マウントが単一コピーの chunk を書き出し状況を悪化させる）と同領域であるが、全幅書き込み + 位置エントリの個別列挙および各々のデバイス識別子保持により、縮退期間中に書き込まれたストライプはディスク上で個別に自己証明され、「暗黙の進行」を排除するのみで「許可すべきか」は解決しない——アトミック性は冗長性を回復しないため、縮退期間中に書き込みを許可するか読み取り専用とするかは確定項目 13 が決定する。

**根拠**:

- ユーザー決定事項（強制再マウントはユーザーにとって極めて不親切であり、その解決策は採用しない）、原文は変更履歴にある；`.claude/singlefs-ai-sop/rules/evidence-discipline.md` に従いメカニズム演繹として記録、三者検証は経ていない。
- 実験なし: 方向性の決定にとどまり、定量可能な量はない。

**未済**: なし。

#### 確定項目 4: 幾何再評価トランザクションの 3 つの形態

**決定事項**: 3 項目——

| # | 決定事項 | 展開 |
|---|---|---|
| a | **幾何が不合格であれば拒絶、プール全体の再評価は行わない** | ディスク追加 / 交換の受入判定は D21 (権威態と派生態の分界) のマウント判定と**同型**：「受け入れ可能 / 不可能」を返し、「少し遅くなる」は返さない。⚠️ **安全に組み合わせ可能かという幾何量（`io_min`、原子性幅など）をチェックし、容量は制限しない**——確定項目 2 で各ディスクの不均一性を許容しており、より大きなディスクの追加は必ず収容しなければならない。⇒ D21 (権威態と派生態の分界)「ランタイムは検査のみ行い計算しない」はそのまま維持 |
| b | **ドレイン完了後に幾何を切り替え** | 再評価トランザクション：ゲート閉鎖（新規書き込みキューイング）→ ドレイン（飛行中の書き込みの全着弾を待機）→ 新幾何のアトミックパブリッシュ → ゲート開放。⇒ システム全体でいかなる瞬間も**幾何は常に 1 つのみ**であり、**いかなる操作も切り替えポイントを跨がない**。代償は有界な一時停止であり、アンマウントではない |
| c | **ディスク追加後にバックグラウンドの再開可能・中断可能なリバランスを実行** | `.claude/rules/fs-design.md` 第 3 枠（バックグラウンド、非決定経路、**戒律なし**）に属する ⇒ フォアグラウンドを停止せず、確定項目 3「業務を中断しない」と矛盾しない。無界の重労働 ⇒ 「書き込み意図 → バッチ分割 → 再開可能」に従う。**移行単位は「オブジェクト全体の新規グループへの再配置」であり、単一ストライプ単位ではない**（根拠は確定項目 8） |

**適用範囲**: b は C55 (ドレイン完了前の幾何切り替え) を「阻止すべき」から「構造的に不可能」へと変え、残るのは「ドレインが本当に完了したか」という判定可能な検査のみとなる；`.claude/rules/fs-design.md`「分岐変数は操作の途中で変更してはならない」が禁じるウィンドウは b に至って初めて真に閉じられる。b の一時停止には上限が必要であるが上限は存在しない：ドレインが待つ対象は「飛行中の書き込み」であり、その量は I-8.1 (ルートリング幾何が十分大きい) の mkfs 計算によって提示され、双方は未接続でありトランザクション層を待つ。c と D3 (領域割り当て) 確定項目 14 の常駐整理は**別個のタスク**である：整理は断片化を扱い、リバランスはディスク間の均等化を扱う。c を引用する際はこの一文を併せて引用すること：座礁容量は E60 (ディスク追加後にデータ移動しないコスト) において判定基準 5 に属し、判定基準 5 は報告のみを要求して閾値を設けていない ⇒ c の決定方向は閾値のない観測から提示されたものであり、「どの程度で受容不能か」は本プロジェクトで一度も決定されていない；座礁を閾値付き判定基準として確立する前に、E60 (ディスク追加後にデータ移動しないコスト) が判定基準に基づきリバランス必須と判定したと述べることは許されない。

**根拠**:

- E60 (ディスク追加後にデータ移動しないコスト): データを移動しない場合、新ディスクは「他ディスクの残余空き領域の合計」しか吸収できず、これは割り当て戦略に無関係な算術上限である；4 台を 90% まで満たした後に同容量のディスクを追加した場合、最良の非移動戦略であっても新ディスクの 60% が座礁し、旧ディスクが 95% 充填時は 80% が座礁する ⇒ コストの次元は「遅い」ではなく追加ディスクの大半が無駄になることであり、それゆえ c のリバランスを実行する必要がある。
- ユーザー決定事項（a / b / c のすべて）、原文は変更履歴にある。

**未済**: C2 (無界操作の単一トランザクション化): c の無界重労働はこれによって担保される；C55 (ドレイン完了前の幾何切り替え): b の後は「ドレインが本当に完了したか」の検査のみが残る。

#### 確定項目 5: ルートリングは再配置しない

**決定事項**: ディスク追加 / 交換の後に**ルートリング領域は再配置しない**；確定項目 4 c のリバランスはユーザーデータを移動し、ルートリングには触れない。

**適用範囲**: コストは 2 方向で異なり、同一ではない。ディスク追加（devs 増加）は同一ディスク衝突を引き起こさない（P は素数かつ任意の合理的なデバイス数より大きい ⇒ `gcd(P, devs) = 1` ⇒ P の乗算は法 devs の全単射であり、`R ≤ devs` であるかぎり帰属は互いに異なる）。真の問題は「現行 devs で計算すると別のディスクを指すが、そのディスク上に該当データが存在しない」ことであり、この枠は確定項目 7 によって解消された——帰属を mkfs 時に書き込まれたデバイス識別子へ変更したことで、ディスク追加によって既存領域の帰属が変更されることはなくなり、衝突は mkfs 時の `R > devs` という鳩の巣原理のみとなり、その時点で mkfs は判定して拒絶可能である。**残る縮退はただ 1 種類である**: 領域が存在するディスクが失われること——これは衝突ではなく喪失である。ディスク脱落 / より少ないディスクへの交換（devs が `< R` へ低下）は鳩の巣原理により必ず衝突し、配置規則とは無関係であり、E47 (ルートリング失敗ドメインの損失) / E48 (ルートリング幾何の実現可能点) の「任意の 1 台喪失でも常にマウント可能、最悪 1 世代ロールバック」という保証はここで失効する。領域のバイト列は mkfs 時にディスク上に固定された既成事実である。

**根拠**:

- E62 (ルートリング帰属：ID保存 vs 計算式): 識別子保存アームは 5 ラウンドにわたりディスク誤指名がすべて 0、計算式アームは 8 ステップのデバイス集合変更で 2 / 4 / 6 / 17 / 19 回誤指名、追加読み取り 0、バイトコスト 4 バイト ⇒ ディスク追加方向のコストは確定項目 7 で解消され、再配置しないことに伴う残余はディスク脱落の枠のみ；陰性対照において mkfs 時点で `R > devs` のときは双方のアームとも衝突し、識別子保存は鳩の巣原理を解決できず、障害ドメインの半分は確定項目 5 が引き受ける。
- ユーザー決定事項（ディスク追加 / 交換後にルートリング領域を再配置しない）、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 6: 書き込みごとに使用する列数

**決定事項**: `w = clamp(バッチ後の書き込み量 + 1, 2, 4)` とし、さらに「当時書き込み可能なデバイス数」でクランプする。**「バッチ後の書き込み量」はスロット幅ごとにバケット分割して個別に計算する**（確定項目 10 根拠 2：1 つのストライプの全スロットは必ず同一幅でなければならない）——1 回のパブリッシュ内で 16 KiB と 32 KiB の 2 つのバケットが各自の `w` を計算する。**上限 4 はシステム構成で宣言された定数**であり、デバイス数には依存しない；`w ≥ 2` はハードな下限である（冗長度ゼロのストライプの発行は禁止）。

**適用範囲**: `w` と確定項目 8 のグループサイズ `g` は別個の量であり、常に `w ≤ g` を満たす；双方の初期値が 4 であるのは偶然であり、一方を変更しても他方は自動変更されない。第 1 版では `w` は常に 2 であり確定項目 18 のディスク追加受入により固定され、`clamp` の上限 4 に第 1 版で触れることはない——これは**宣言**であり、第 1 版で到達可能な値ではない。承重する根拠は `.claude/rules/fs-design.md` のハード要件 5（分岐変数は操作の途中で変更してはならない：上限を `f(デバイス数)` と書くと飛行中にデバイス数が変動し、ディスク脱落が確定項目 4 b のドレインバリアをバイパスする）およびハード要件 2（各分岐はテストにより強制進入可能でなければならない：定数化により値の集合が `{2,3,4}` に閉じる）、限界算術ではない。明記必須の 3 つの未知事項：① 「変曲点 4」という限界算術には**次元の誤り**がある——リスク項はストライプごと、コスト項は容量ごとに計測され、同一次元の尺度は期待データ損失率 `(w−1)/C(D,2)` であり、**二次関数ではなく線形**であり、同一等加重で D=8 のとき上限は **6** ⇒ 4 はより保守的な側にあり、その算術によって固定されたものではない；② 等加重（空間 1 パーセントポイント ↔ データ損失 1 パーセントポイント）には kb 内に出典がなく、λ=1 で 6、λ=2 で 3、λ=0.5 で 5；③ 推導全体は**厳密に `parity = 1` のもとでのみ成立**し、parity=2 では 2 台同時破損でもデータ損失はなくリスク軸は 3 台破損へ移行し、同一算術は 7 を与える ⇒ 上限は parity に応じて再設定され、parity の枠は未確定項目 21 である。

**根拠**:

- E63 (アロケータの書き込み列数): 小サイズ書き込み比率 90% のとき「書き込み量適合」は「常に全幅」に比べ 61% の物理スロットを節約 ⇒ 判定基準は書き込み量を含まなければならない；8 台バッチ処理下で上限 4 と上限撤廃（=8）の 2 台同時破損時の期待損失は 10.7% vs 25.0%、上限撤廃は 13% の領域節約と引き換えに 2.3 倍の期待損失を招き、ストライプ当たりヒット率 1.000 ⇒ 無傷なファイルが 1 つもなくなり、破損モードは「部分破損」から拡散型へと悪化。
- 三者検証：順推脚は狭小化版を受け入れ限界算術の次元誤りを捕捉、逆推脚は「D2 (RAID ストライプ方針) 原文は 3 要素の一括提示を要求しており、1 つの数値のみの提示は決定とみなせない」と判定 ⇒ 今回は判定基準と上限のみを決定し参加サブセットは確定項目 8 へ分離、ローカル脚はプール全体での計算を主張したがその小プール反例は次元修正後に弱体化（D=4 で 3→4 は引き分け）、生出力 `d2-item6-local.out`。
- ユーザー決定事項、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 7: ルートリング帰属は領域ごとに明示的にデバイス識別子を保存

**決定事項**: 帰属は `(r × P) mod devs` による計算を廃止し、mkfs 時に領域ごとにデバイス識別子を書き込む。第 1 版の 3 領域の帰属は **0 / 1 / 0** に固定（領域 0 → ディスク 0、領域 1 → ディスク 1、領域 2 → ディスク 0）、mkfs が領域ごとにシステム構成の 12 バイト（領域ごとに 4 バイトのデバイス識別子）に書き込む；**これは mkfs パラメータではない**。

**適用範囲**: 確定項目 1、D19 (ブロックポインタの構造と幅予算) 確定項目 1 と同型——本プロジェクトが「計算式による位置逆算」を「個別明示列挙」に置換した 3 度目の事例。獲得したものは「誤ったディスクを指さないこと」であり、「障害ドメインの独立」ではない：ディスクが `devs < R` まで減少した後は障害ドメインの保証は同様に失効し、その半分は確定項目 5 が引き受ける。フィールドがシステム構成のどの区画に位置し、領域起点がどう計算されるかは D22 (ユニット原子性の合成方法) 確定項目 16；ウォームアップ空パブリッシュの回数はこの帰属によって一意に決定され（txg 1 は領域 1 = ディスク 1、txg 2 は領域 2 = ディスク 0、2 回で 2 台を網羅）、新規プールでのファイル作成の txg 3 とともにフォーマット定数となる（D22 (ユニット原子性の合成方法) 確定項目 16 第 5 文参照）。**新規プールでのファイル作成のバイト列を変更するか：いいえ**——システム構成のその 12 バイトには元から 0 / 1 / 0 が書き込まれており、0 / 1 / 0 の文言は条項を補完するものである。

**根拠**:

- E62 (ルートリング帰属：ID保存 vs 計算式): 8 ステップのデバイス集合変更で計算式アームは誤指名 2 / 4 / 6 / 17 / 19 回、識別子保存アームは 5 ラウンド全 0、追加読み取り 0（識別子はシステム構成と同一アドレスにあり、システム構成は元から読み取り必須）、バイトコスト 4 バイト（4 領域 × 1 バイト）。
- E87 (固定構造の配置): 「2 台ディスク上で R=3 は必ず 1 台が 2 領域を背負う」——D22 (ユニット原子性の合成方法) 確定項目 14 の鳩の巣下限と合致し、領域番号決定後は 0 / 1 / 0 が唯一の規範的表記；mkfs パラメータとしても「どちらが 2 領域を背負うか」を買うに過ぎず、同種 2 台下で観測可能な差異はない。
- ユーザー決定事項 2 回（帰属を識別子保存へ変更、第 1 版の 3 領域を 0 / 1 / 0 固定）、後者は三者検証を経ておらず覆され得る、原文は変更履歴にある。

**未済**: C182 (ルートリング配置帰属に関する確定二項目の相反)、C217 (Pがデバイス数超の制約理由消滅と制約残存)。

#### 確定項目 8: どのデバイスが参加するか

**決定事項**: オブジェクト作成時に、その時点で最も空いている `g` 台のディスクからグループを選定；以後は当該オブジェクトの各ストライプはグループ内で最も空いている `w` 台のみを選定（`w` は確定項目 6 から供給）；グループ内で満杯のディスクが出た場合はグループを再選定する。`g` はシステム構成で宣言された定数であり初期値 4。**グループはディスクに保存しない**——当該オブジェクトの位置エントリのデバイス識別子の**和集合**から導出され、ディスク上のバイト追加はゼロ。確定項目 4 c のリバランスは「オブジェクト」を移行単位とし、グループ全体を再配置し、ストライプ単位の移動は禁止する。

**適用範囲**: `g` と `w` は別個の量であり常に `w ≤ g` を満たす、双方の初期値が 4 であるのは偶然；無傷なオブジェクトの比率は `g` のみに従い（`1 − C(g,2)/C(D,2)`）、空間オーバーヘッドは `w` のみに従う（`1/(w−1)`）。「オブジェクト → どのディスクか」を別途保存することは縮小版の chunk マッピングテーブルであり、確定項目 1 で判定済みの第 2 の権威構造であり、D18 (ブロックに含める情報) 確定項目 3 の禁止リストも物理配置先 / デバイス ID がブロックヘッダに入ることを明確に禁止している ⇒ グループはアロケータの派生状態である；配置先自体は中央マッピングを正とし、位置エントリはヒントであり、中央マッピングこそが参照解決と解放判定の唯一の入口である（D19 (ブロックポインタの構造と幅予算) 確定項目 5）；読み取り時はその配置先を直接試行可能であるが陳腐化している可能性がある。明記必須の 2 つの未知事項：① **単一オブジェクトの並行度上限 = `g`**、1 オブジェクトが 4 台のディスクに固定される ⇒ 並行度はデバイス数ではなく 4 であり、この次元は未計測であり、D25 (目標ワークロード優先度) の主負荷がシーケンシャル大ファイルである以上、計測前に 78.6% を純利益として引用することは禁止；② 計測したのは `g = w = 4` の 1 点のみであり、`g=5` で 64.3%、`g=6` で 46.4% となり、`g` 変更時は再計測が必要。

**根拠**:

- E67 (ストライプが参加するデバイスの選定): 8 台、幅 4、512 オブジェクト、2 台同時破損の全 28 ペアを網羅検証、本項の規則は無傷オブジェクト比率 78.6%（＝`1 − C(4,2)/C(8,2)`、単体テストで固定）かつディスク間極差 0 を達成、対してストライプごとのラウンドロビンは 14.3%、ストライプごと最空選択は 57.1%（均一）/ 77.2%（偏り）；オブジェクト番号によるグループ固定も 78.6% であるが偏り時のディスク間極差は 88.7%（早期 ENOSPC）；6 つのアームの期待損失率は同一（`3/28`、幅のみで決定）⇒ **選定法が決定するのは損失の分布形態であり、損失量ではない**。第 4 ラウンドで 9 台目を追加しリバランス、オブジェクト全体移動は 83.3%（＝`1 − C(4,2)/C(9,2)` の理論値、欠落ゼロ）、ストライプ単位移動は 75.0% へ低下 ⇒ 「オブジェクトを移行単位とする」条項の根拠。
- 三者検証：順推脚は狭小化版を受け入れ（グループ非永続化、オブジェクト単位のリバランス、`w<g` 時にグループ内最空 `w` 台を選択）`g` と `w` が 2 つのノブであると指摘；逆推脚はリバランスと「以後固定」の構造的衝突の追加テストを要求し補完（E67 (ストライプが参加するデバイスの選定) 第 4 ラウンド）；ローカル脚の反例「各ディスク容量不均一 ⇒ 4 台の小ディスクに固定される可能性」は容量異種ラウンドで再現せず（残余領域選定とストライプごと最空選定が一致、1148 vs 1148）、同ラウンド出力に破損語が含まれ瑕疵付き 1 票として記録、生出力 `d2-item8-local.out`。
- ユーザー決定事項、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 9: 最初の実行可能ターゲットは 2 台のディスクで稼働

**決定事項**: 第 1 版では単一ディスクで稼働させず、最初の実行可能ターゲットは 2 台のディスクで稼働；単一ディスクは独立したラインとし、その開通時に決定する。

**適用範囲**: 裁定された排他性——**確定項目 6 の `w ≥ 2`（冗長度ゼロのストライプ発行禁止）と「当時書き込み可能なデバイス数でクランプ」が単一デバイス上で排他**となり、1 台のディスクでは前者が 2 列を要求し後者が 1 列しか供給できない。これにより D18 (ブロックに含める情報) 確定項目 4（単一ディスク自己回復の形態）をブロッキング集合から除外：その項目は「単一ディスクレイアウト」ラインの開通に伴い開始され、それまでは 1 バイトも書き出せない；⚠️ そのラインの開通時にはまず挙証責任を果たす必要がある——**いかなる現役実装も「任意地点の前方・後方チェーン回復」の道をとっておらず**、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` に従い挙証責任はこちら側にあるが挙証は一切なされていない。実行可能：`.claude/kb/vm-harness.md` の QEMU 環境で 2 台のディスクを合成すれば十分であり実ハードウェアは不要。

**根拠**:

- D2 (RAID ストライプ方針) は可変幅全幅ストライプに ≥2 台を要求と決定済み ⇒ 単一ディスクは元より第 1 版のレイアウトに含まれない；強引に回避するには単一ディスク用に別レイアウトを開設せねばならず（D12 (目標メディア) で各レイアウトは incompat ビットを 1 つ占有と決定済み）、`.claude/rules/fs-design.md` 原文通り「判定基準が書けないときは、まず 1 つだけ作り、かつ**遅くとも正しい方を先に作る**」。
- ユーザー決定事項、原文は変更履歴にある。
- 実験なし: 本項目は単一デバイス上での 2 つの確定条項の排他性に対する裁定であり、定量可能な量はない。

**未済**: なし。

#### 確定項目 10: 1 ユニットは単一列に完全に収まる

**決定事項**: 1 つの 32768 バイトのユニットは切断されず、複数列に跨がらない；位置エントリはレプリカごとに 1 つとする。

**適用範囲**: 根拠は 1 つの算術と 2 つの確定条項。**スロット = ユニット**（D4 (チェックサム配置位置) 確定項目 2 / 3 が最小書き込みを整ユニットに固定）、ユニットには 2 段階が存在（D18 (ブロックに含める情報) 確定項目 9：メタデータ 16384、データ 32768）⇒ スロットにも 2 段階が存在；**1 つのストライプの全スロットは必ず同一幅でなければならず**、ビットごとの XOR が定義される前提である ⇒ アロケータはスロット幅ごとにバケット分割してバッチ処理し、確定項目 6 の「バッチ後の書き込み量」はバケットごとに計算され、パブリッシュ全体の総量ではない。跨がり解釈は `w = 4` で直ちに破綻：`32768 / 3` は整数ではなく `io_min` の倍数でもなく、確定項目 16 に抵触。D19 (ブロックポインタの構造と幅予算) 根拠 3 の「ヘッダ + 4 + 2 ストライプ 6 列」で計算されたポインタ占有率は跨がり解釈下で計算されたものであり、その計算式は不成立であるが、論証していたのは「固定幅ポインタのコストは受容可能」であり、**結論は影響を受けない**——本項の計測条件で再計算するとポインタはより薄く按分される（位置エントリが w 個からレプリカあたり 1 個に減少）。確定項目 10 と確定項目 6 の連言は `w ≥ 3` で初めて生じる穴（ストライプメンバー解放後に残存メンバーの再構築入力が失われる）を残すが、E80 (ストライプの部分解放) で測定され確定項目 11 により収束；`w = 2` の対照群は全ゼロ ⇒ 第 1 版の全テストはこれに対して盲目である。

**根拠**:

- ユーザー決定事項、原文は変更履歴にある。
- 実験なし: 跨がり解釈は算術的に即時破綻（`32768 / 3` が整数でない）、承重するのは 1 つの算術と 2 つの確定条項であり、定量可能な量はない。

**未済**: C81 (ストライプ部分解放の規則なし)。

#### 確定項目 11: ストライプのライフサイクル——部分ストライプ固定、バックグラウンド遅延再配置

**決定事項**: 3 つの規則——

1. **w ≥ 3 ストライプのメンバーが解放された後、当該スロットとストライプ全体（パリティを含む）は修復完了まで再利用を禁止する**。ピン留めされたスロットは「回収不能バイト数」統計量（D5 (スナップショット・領域台帳機構) 確定項目 4 第 3 項、既成の配置先）に計上——固定量は自然に観測可能となり、`.claude/rules/fs-design.md` ハード要件 4 を充足。
2. **修復 = バックグラウンドで残存者を一括して新規ストライプへ再配置し、旧ストライプ全体を回収**、D26 (バックグラウンド整理と配置回収) 脚 1（常駐整理）のキューと予算で稼働。修復キューは**派生状態**である（割り当てレコードとストライプメンバーから再構築可能であり、消失しても遅延するのみ）。
3. **w = 2 のストライプはメンバー 1 つの破棄で全体を回収**（データ 1 + レプリカ 1、ユニット間の結合なし）⇒ 第 1 版（2 台恒常 `w = 2`）は自動的に充足、コードパスの追加ゼロ。

**適用範囲**: 「修復」という語は確定項目 11 では 1 つの分類のみを管轄——**ストライプメンバーの解放**。ディスク脱落後に喪失スロットを再構築することは別問題であり形態は未定（まさに確定項目 12 のメンバー表を要する）、両者を混同してはならない：規則 2 の「一括して**1 本の**新規ストライプへ再配置」は解放の分類においてのみ恒常的に成立（解放後に k 個の残存者があるとき `w = clamp(k + 1, 2, 4)`、幅 `w` のストライプはちょうど k 個のデータスロットを収容、`k ≤ 3`）、デグレード環境下では文面通り実行不可能——`w` は当時書き込み可能なデバイス数でクランプされ（確定項目 6）、4 台から 1 台脱落後は `w ≤ 3`、新規ストライプは 2 つのデータスロットしか持てず、3 つの再配置ユニットを収容できない ⇒ ディスク脱落再構築の分類は個別に決定せねばならず、配置先は確定項目 12 とその後継である。コストと既知・未知の事項、引用時は併せて引用：余分な書き込み +47% は均一ランダム再書き込み負荷下での規模感の提示であり予測ではない；修復予算の下限は未走査（予算 2 でモデル負荷に辛うじて追従）；「残存者をフォアグラウンド書き込みのバッチに合流」というより低コストな形態は実装へ委ねられ、E80 (ストライプの部分解放) ではモデル化されていない。

**根拠**:

- E80 (ストライプの部分解放)（2 ラウンド、5 シード × 2 負荷）：4 つの処置の中で「ピン留め固定 + バックグラウンド遅延再配置」のみが損失ゼロ、有界な固定（ラウンドあたり予算 2 でラウンド末尾恒常 0、負荷停止ドレインでゼロ復帰）、即時修復より約 37% 低コスト（余分な書き込み +47% vs +75%）を同時に達成；放置して再利用すると単一ディスク障害で平均 9.8–13.0 個の生存ユニットを喪失しかつ障害前エラー報告ゼロ、完全破棄まで回収しないと固定生存データが 33–46% 無界累積、メンバー破棄時即時修復は修復債務が自己更新（約 4000 回の修復イベント）。
- ユーザー決定事項、原文は変更履歴にある。

**未済**: C81 (ストライプ部分解放の規則なし): イメージ側で「任意の生存ユニットのストライプメンバーはバイトが原型を留めているか、さもなくばストライプ全体が完全に破棄されているか」を判定；C127 (固定状態が割り当てレコード内で配置先なし)。

#### 確定項目 12: メンバー表はコード 3 パッキングレコード、コンテナインデックス経由、コンテナは常時ミラー

**決定事項**: ストライプメンバー表のレコード（固定長 56 バイト、パリティを含め 1 スロット 1 レコード）は**コード 3 パッキングレコードユニット**に格納、単一の**コンテナインデックス**によってコンテナを指名；**コンテナチェーン全体は常に `w = 2` ミラー**とする。状態区分は**権威態**：D21 (権威態と派生態の分界) 自身の判定基準「失われればデータ損失 / 失われても単に遅延」に従い、本表の喪失は再構築を不可能としアロケータの固定判定を不能にする ⇒ ユニットのペイロード内に格納されねばならず、ユニットペイロードへの格納は権威態リストの第 1 項にとどまり拡張を行わない。**新規プールでのファイル作成のバイト列を変更するか：いいえ**（第 1 版 2 台恒常 `w = 2`、パリティは第 2 レプリカであり、レプリカごとの位置エントリが検出を網羅済み ⇒ 新規プールでのファイル作成でストライプレコードは書き出さない）。

**適用範囲**: 媒体を決定し、順序は決定しない——ストライプテーブル論理ツリーのキー順序は確定項目 14 に単独列挙。**なぜコンテナは常にミラーか**: ミラーなしでは定常状態における単一ディスク障害時の未解決スロット数は恒常的に 0 であるが、**プール作成直後の枠において恒常的に 1**——ストライプテーブルに 1 つしかコンテナがないとき、それは自身が記述するストライプ内に配置され、コンテナの存在するディスクが破損するといかなる順序でも解決できず、5 シードすべてで発生し、構造的であって確率的ではない；ミラー化はこのブートストラップ依存を構造的に遮断する。本方針の採用理由は空間でも書き込みバイト数でもない：空間の次元は `.claude/rules/fs-design.md`「誰がより節約できるかの比較は判定基準にならない」により判定基準に入らず、書き込みバイト数の次元は互角（主負荷の差は 4.7%、混合負荷でバイト比 0.7816 で甲が 22% 節約するがラウンド数では甲が有利なのは 65.8% のパブリッシュのみ）⇒ 採用理由は「単一ディスク障害後に障害ディスク上のスロットが再構築可能か」に確定的回答を与えることであり、`w` 恒常 2 全ミラーの道は `w ≥ 3` を完全に放棄してしまう。

コスト内訳：

| 次元 | 数値 | 出典 |
|---|---|---|
| 定常状態空間 | **3087 ppm、`w = 3` と `w = 4` で同値**（16 TiB / 90% 占有、ミラー込み；非ミラー 1543）——エントリ数は 90% 予算で決定され、`w` は予算内のデータとパリティの比率を変更するのみ | E111 (ストライプテーブルのキーと幾何)、E106 (ストライプメンバー表の媒体) アーム甲ミラーの 54 315 515 904 バイトとビット単位で一致 |
| パブリッシュごと書き込みバイト数 ÷「`w` 恒常 2 全ミラー」 | 主負荷**期待値 0.9531**（4.7% 節約）；コンテナ満杯直前枠 1.050 | E110 (連続パブリッシュ下でのストライプテーブルの期待書き込み増幅) |
| ストライプテーブル分の占有 | **163 840 バイト / パブリッシュ、葉数に無関係**（コンテナ 32768 + コンテナインデックス 3 層 × 16384、チェーン全体ミラー） | E107 (主ワークロードに対するストライプテーブルの書き込み増幅) |
| 3 段階書き込み順序の遅延 | ack セマンティクスの差はノイズ内；durable セマンティクス **1.625×** | E107 (主ワークロードに対するストライプテーブルの書き込み増幅) 乙アーム |
| 単一ディスク障害時の未解決スロット数 | **構造的に恒常 0** | E106 (ストライプメンバー表の媒体) アーム甲ミラー |

**根拠**:

- E106 (ストライプメンバー表の媒体): メンバー表がコード 3 パッキングレコードに格納される場合、定常状態の未解決スロット数は恒常的に 0 であるが、プール作成直後の枠では恒常的に 1 であり 5 シードすべてで発生 ⇒ コンテナはミラー必須；3 アームの定常空間も同一成果物内。
- E107 (主ワークロードに対するストライプテーブルの書き込み増幅): メンバー表維持による毎パブリッシュの追加書き込みバイト数と 3 段階順序の遅延、これら 2 点は E106 (ストライプメンバー表の媒体) では未回答と明記。
- E110 (連続パブリッシュ下でのストライプテーブルの期待書き込み増幅): E107 (主ワークロードに対するストライプテーブルの書き込み増幅) のエッジケースを解消——コンテナ占有スロット数をワークロードの自己進化に委ねた後、主負荷の累積比率は安定し、コンテナ満杯直前の端点はごく一部のパブリッシュにとどまる。
- E111 (ストライプテーブルのキーと幾何): 配置先順のエントリ数は `w` に依存せず、定常空間は E106 (ストライプメンバー表の媒体) アーム甲ミラーとビット単位で一致 ⇒ コスト行が収束。
- ユーザー決定事項、原文は変更履歴にある。

**未済**: `w ≥ 3` 開通前に 4 項目を清算必須——C103 (コンテナインデックスの配置先なし)（コンテナインデックスツリーの格納場所、キーエンコード、自身の更新コスト、専有 MAC の再構築時再計算方法）；**ブートストラップ**（コンテナインデックス自身のノードを誰が指名するか、**新規の予約ツリー ID 登録**によって閉じる必要があるが、確定項目 20 は day-1 登録を行わず開通時に新規番号取得と決定済み）；C127 (固定状態が割り当てレコード内で配置先なし)；C128 (清掃受入がパリティスロットの双方で判定不能)。加えて C303 (D2境界③の根拠がD9確定7撤回に伴い失効) および C155 (実験間共通量の相互アサーションなし)。

#### 確定項目 13: デグレード期間中は読み取り専用

**決定事項**: デグレード期間中は**読み取り専用**とする。判定基準は書き込み可能マウント受入の連言として策定し、稼働時のゲートを配備：

| タイミング | 判定基準 |
|---|---|
| マウント受入 | 書き込み可能デバイス数 ≥ w 下限 ∧ **プール内過半数のデバイスの排他オープン成功**（過半数は閾値；新インスタンス番号は排他オープンに成功した**全**デバイスに書き込み、過半数のみではない）∧ インスタンス表可読 ∧ インスタンス切り替え予約の確保可能（計算式は D28 (マウント時コミットメント量) 確定項目 3）；いずれか不成立で読み取り専用マウント；例外：空間条項（インスタンス切り替え予約および D28 (マウント時コミットメント量) 確定項目 1 受入）が番号取得前に不足する場合、D16 (パブリッシュセマンティクス) 確定項目 1「受入」行に従い行を先行書き込み、F を引き上げる空パブリッシュを推進し再判定、満杯まで推進しても不足する場合はマウントを書き込み可能として成立させ、以後のパブリッシュはパブリッシュ受入で判定し不足時に ENOSPC を報告、正常アンマウントは通常通り（ユーザー 2026-09-26 決定、攻撃ラウンド 0） |
| 稼働時 | 書き込み可能デバイス数が w 下限未満、**または排他オープン成功デバイス数が過半数未満に低下**した時点で読み取り専用へ移行。**「書き込み可能デバイス数」の計測条件**：デバイスが書き込み不能であるとは、その固定配置先へのプローブ書き込みが 1 回失敗することと同値（D23 (ジャーナルの役割とフォーマット) 確定項目 14 の 2 系統弁別子と同一動作）——失敗回数や失敗継続時間では判定しない、障害発生の瞬間に判定不能であるため |

**デバイス集合変更時の再判定**: 確定項目 3 の「幾何再評価」トランザクションに 1 条追加——デバイス集合変更後に過半数および番号取得の書き込み集合を再判定し、不成立時は読み取り専用へ移行。

**適用範囲**: 「デグレード期間中に書き込み可能でマウントできるか、マウント後にいつ読み取り専用へ移行するか」を決定し、「デグレード期間中に書き込まれたストライプをどう補修するか」は決定しない——確定項目 11 の部分ストライプ固定とバックグラウンド遅延再配置に属する。**なぜ「書き込み可能デバイス数が足りていれば十分」ではなく過半数なのか**: C113 (スキャン再構築時の複数版ユニット現行版判定の入力欠落) の決定は同一瞬間に書き込み可能マウントが 1 つのみであることを要求しており（2 つの並行書き込み可能マウントは同一のインスタンス番号を算出し、パブリッシュ済み述語の 3 系統のいずれも区別できない）、ブロックデバイスの排他オープンは**デバイス単位**の排他であるため、**プール単位**の相互排他へと引き上げるには任意の 2 つの書き込み可能マウントのデバイス集合が必ず交差せねばならず、過半数は最も緩やかな条件である；ディスク数 ≤ 3 のときは w 下限を満たす任意の 2 集合は元より交差するが、ディスク数 ≥ 4 では互いに素な 2 元部分集合が存在し、過半数が唯一成立するゲートとなる。代償：4 台プールで書き込み可能が 2 台に低下すると書き込み可能マウントできず、マウント後に 2 台へ低下しても読み取り専用へ移行、旧判定基準（書き込み可能デバイス数 ≥ w 下限のみ参照）では双方書き込み可能であった——これは可用性を犠牲にして「2 つのタイムラインが同一インスタンス番号を共有しない」ことを買い取っており、他所の括弧書きで「確定項目 9 と同一志向」と見なすことは許されない。確定項目 9 は第 1 版の稼働ディスク数に関するものであり、デグレード時の書き込み可否とは無関係である。再判定を行わない場合、オンラインでのディスク追加はプールの基数を引き上げマウントの保持する排他集合は不変であるため、本マウントで一度もオープンされていない健全ディスクは永遠に新インスタンス番号を取得できず、「番号取得を逃したディスク」としてディスク全体が無効化され再同期を余儀なくされる。

**根拠**:

- E141 (切替予約のマウント受入自己証明): マウント受入における「インスタンス切り替え予約の確保可能」という連言が、1 台のみ異なる 2 つのプールにおいて書き込み可能と読み取り専用を弁別でき、予約を 0 に設定すると双方が書き込み可能となる ⇒ 本連言は弁別力を持ち形骸ではない。
- C113 (スキャン再構築時の複数版ユニット現行版判定の入力欠落) 決定（第 7、第 8 ラウンド逆推脚 5.1 および 7.2 調査で確定）およびユーザー決定事項に伴い、原文は変更履歴にある。
- 空間条項の例外：ユーザー決定事項、原文は変更履歴にある；実装とテストケースは清算済み表内の C565 (マウント時押し切り不足の収拾策未定) 行参照。

**未済**: C833 (書き込み可能マウントの排他オープン欠落)。

#### 確定項目 14: ストライプテーブルは配置先順、割り当てレコードツリーと同一座標系

**決定事項**: キー = **配置先**（デバイス識別子 4 バイト + 16 KiB スロット番号 6 バイト）、D3 (領域割り当て) 確定項目 3 の割り当てレコードツリーと**同一の座標系**（D3 (領域割り当て) 確定項目 3 根拠 2 原文通り「各自が勝手に決定してはならない」）。**新規プールでのファイル作成のバイト列を変更するか：いいえ**（第 1 版 2 台恒常 `w = 2`、ストライプレコードは 1 行も書き込まない ⇒ 本ツリーは新規プールでのファイル作成において空であり、キーに何を選定してもバイト列は出力されない；`w ≥ 3` の最初のストライプ発行前に存在必須）。

**適用範囲**: 本項目は計測によるものではなく、3 つの候補キーの定義から直接導出されたものである——ストライプテーブルの 3 つの消費者は**同一の問い**「与えられた配置先はどのストライプに属するか」を問う：割り当て受入（C127 (固定状態が割り当てレコード内で配置先なし)）は候補配置先がピン留めスロットかを判定、清掃受入（C128 (清掃受入がパリティスロットの双方で判定不能)）はヘッダのないパリティスロットを清掃すべきかを判定、譲渡区 A の発行判定は固定スロットおよびその所属ストライプの全スロットを排除 ⇒ ストライプ番号順やコンテナ識別子のみのインデックスではこの問いに回答できない；コンテナインデックスはなおさら回答できず、キーは `(誕生ツリー ID, パッキングレコードタイプ, コンテナ番号, コンテナ誕生世代)` であり配置先を含まず、コンテナ番号は C103 (コンテナインデックスの配置先なし) に従い「最大番号 + 1」の単調割り当てであり、格納レコードがどの配置先を網羅するかと関数関係を持たない。空間の次元は判定基準に入らず（`.claude/rules/fs-design.md`「誰がより節約できるかの比較は判定基準にならない」）、支払う代償の認識のために提示されたにとどまる；判定基準に入るのは「その問いに回答できるかどうか」である。依然未済：コンテナインデックス自身の格納場所（C103 (コンテナインデックスの配置先なし)）——本ツリーは配置先順であり葉はコード 3 コンテナであり、コンテナの物理ポインタはコンテナインデックスが保持するが、コンテナインデックス自身の配置先はリポジトリ全体で未定義；本ツリー自身の更新コスト（パブリッシュごとの書き換えレコード数、COW ノード数）は E111 (ストライプテーブルのキーと幾何) では未回答、E107 (主ワークロードに対するストライプテーブルの書き込み増幅) および C103 (コンテナインデックスの配置先なし) の範囲に属する。

**根拠**:

- E111 (ストライプテーブルのキーと幾何): 配置先順のとき `w = 2` はレコード書き出しゼロ（0 ppm）、`w = 3` と `w = 4` はともに 483 183 820 レコード、ツリー高 3、占有 3087 ppm、同尺度下で割り当てレコードツリーは 442 ppm ⇒ 本ツリーは 6.98 倍；エントリ数は `w` に無関係であり 90% 予算から切り出されるスロット数と同数。
- ユーザー決定事項、原文は変更履歴にある。

**未済**: C103 (コンテナインデックスの配置先なし)、C127 (固定状態が割り当てレコード内で配置先なし)、C128 (清掃受入がパリティスロットの双方で判定不能)。

#### 確定項目 15: 「ライフサイクルの異なるオブジェクト」の一般的判定基準

**決定事項**: 同一の物理マッピングユニットを占有する 2 つのオブジェクト X、Y について、それぞれの**破棄トリガーイベント**を特定する——「バイトが静止を維持せねばならない」から「上書き / 回収可能」へと移行させる特定のコミット。**フォーマットが X と Y の破棄トリガーイベントが常に同一コミットであることを保証できるならば同一ライフサイクルとし、共有を許可する；双方の破棄イベントが互いに独立して乖離可能なコミットであることを許容するならばライフサイクルは異なるとみなし、共有を禁止する。**判定基準は「生」ではなく「死」に固定される。同時に確定項目 17 の適用範囲を io_min 内の固定構造（ルートスロット、システム構成スロット、墓碑の類）へ狭小化し、16 KiB ノードと 32 KiB ユニットは適用外とし、**アロケータのアラインメント変更はゼロ**とする。**新規プールでのファイル作成のバイト列を変更するか：はい**——その裁定は D22 (ユニット原子性の合成方法) 確定項目 2 のスロット幾何を直接決定し、ルートリング領域は mkfs 時にディスクに刻まれた既成事実であり変更不能である。

**適用範囲**: **「書き込み時点」で判定してはならない**——同世代の墓碑レコードは元より逐次追記されるものであり、書き込み時点で判定すると「ライフサイクルが異なる」と判定され、D18 (ブロックに含める情報) 確定項目 10 のユーザー決定事項と即座に矛盾する。既知の 2 つの消費者はともに検証をクリア：墓碑——同世代レコードの破棄イベントはいずれも「当該世代 deadlist の一括退役」という単一の判定 ⇒ 同一ライフサイクル、D18 (ブロックに含める情報) 確定項目 10 と合致；ルートスロット——D22 (ユニット原子性の合成方法) 確定項目 2 のスロット順序は領域跨ぎローテーション（領域 = `txg mod R`）⇒ 1 回の txg で 1 スロットのみ書き込み ⇒ 異なるスロットの次回上書きは 2 つの独立した将来コミット ⇒ 「異なる」と判定、しかし「異なる ⇒ スロット幅変更必須 ⇒ D20 (承重面：ユニットの原子性と自己完結性) に衝突」という演繹は第 1 歩で破綻：スロット幅とスロット開始オフセットは D22 (ユニット原子性の合成方法) 確定項目 2 自体が区別する 2 行であり、開始オフセットを `io_min`（I-7.5 (ルートスロットの判定幅アラインメント) の粒度パラメータ）へ引き上げれば確定項目 17 を満たし、スロット幅は依然 `physical_block_size` に等しく、D20 (承重面：ユニットの原子性と自己完結性) 推論 3 の規則には触れない。再走査により判明した消費者は 2 つにとどまらない（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「新たな判定基準を策定した場合、直ちに既存項目を再走査する」）：インデックスノード（16 KiB）とデータユニット（32 KiB）は `io_min` = 64 KiB のアレイ上で本判定基準により「異なる」と判定され、COW は免除を構成せず（ブロック再利用により隣接スロットが任意の後刻に再書き込みされる）、ジャーナルレコードの枠はルートスロットより強固（2 台ミラーのみ）；[invariants/02-根环.md](../../../crates/singlefs-checker/invariants/02-根环.md) を調査したところ、`io_min` は I-7.5 (ルートスロットの判定幅アラインメント) の 1 箇所のみに出現（判定幅は max(`physical_block_size`, `io_min`)、`io_min` が `physical_block_size` より大きい場合は `io_min` で判定と注記）、I-7.5 (ルートスロットの判定幅アラインメント) は実装済み（プールレベル checker がスロットごとにルートスロット開始オフセットを判定）、「物理マッピングユニット」は依然ヒットゼロ ⇒ checker は確定項目 17 についてルートスロットのみ判定し、インデックスノード、データユニット、ジャーナルレコードは判定せず、確定項目 16 も判定しない。別の方針（文面通りアロケータのアラインメントを変更）は採用しない；配置方針は D3 (領域割り当て) 確定項目 8 にある（ユーザーデータはクラスタセグメントに入らず選定デバイス内の最小空きスロット番号を選択、アラインメントは考慮しない）。確定項目 15 と C212 (固定構造の書き込みがio_min未満) は別個の事象：後者は確定項目 16 に圧力をかけており、確定項目 16 には未定義語が 1 つも存在しないため、それは 2 つの確定条項の衝突であり判定基準の欠落ではない。

**根拠**:

- 三者検証 2026-09-10 ラウンド：判定基準は順推脚が提示、メイン agent が各消費者（墓碑とルートスロットの 2 箇所）に代入して検証、資料およびローカル脚出力は `_d2-item15-background.md`、`d2-item15 ラウンドの脚報告（git 内の成果物）`。
- 外部の事実（2026-09-08 調査、本プロジェクト内では未検証）：ZFS のルートスロット幅は `MIN(MAX(vdev_ashift, UBERBLOCK_SHIFT), MAX_UBERBLOCK_SHIFT)` 原文通りであり、デバイスアラインメント粒度に従い上限 8 KiB、使用量は `io_min` ではなく `ashift`、[prior-art.md](../prior-art.md)「8.1 ZFS：ルートスロット幅はデバイス粒度に伴い引き上げられるが上限 8 KiB であり、幅引き上げはスロット数削減であって占有面積拡大ではない」参照；既知の相違として ZFS のリングは 128 KiB 固定、本プロジェクトの領域サイズはフォーマット定数ではない。
- ユーザー決定事項、原文は変更履歴にある。
- 実験なし: 判定基準自体は定義からの演繹、2 つの消費者は代入による検証であり、定量可能な量はない。

**未済**: C254 (物理マッピングユニットの利用先多数とchecker網羅ゼロ)。

#### 確定項目 16: `io_min` 未満の書き込みを発行しない

**決定事項**: **`io_min` 未満の書き込みを発行しない。** Linux 上においてこの量は `physical_block_size` ではなく `io_min` である；**両者を 1 つの文に統合することは禁止する**。

| 量 | カーネル内での算出 | 本質 |
|---|---|---|
| `io_min` | `lim->io_min = phys_bs`、`phys_bs = bs * (1 + npwg)`（NPWG = Namespace Preferred Write Granularity） | **デバイス内部 RMW を誘発しない最小書き込み幅** |
| `physical_block_size` | `lim->physical_block_size = min(phys_bs, atomic_bs)`（`atomic_bs` は NAWUPF より、未宣言時は論理ブロック同等） | **デバイスが保証する停電アトミック幅** |

**適用範囲**: メカニズムとして**デバイスの物理マッピングユニットに対してより小さな書き込みを発行すると、デバイス内部でリード・モディファイ・ライトが発生する**。計測条件：2026-08-28 `drivers/nvme/host/core.c` の `nvme_update_disk_info` における `physical_block_size` 算出箇所を調査、Linux 7.2.0、`min` 直上のコメントは原子性のみを言及（「limit the physical block size to the value of the Atomic Write Unit **Power Fail** parameter」）⇒ `physical_block_size ≤ io_min`、`io_min` で書き込んでも依然 NPWG を下回り RMW を誘発する可能性がある。**本条項の変更には三者検証が必要**、E129 (io_min未満の書き込みが実デバイスで引き起こす問題) は観測結果の提示にとどまる；3 つの確定条項と衝突（ルートスロット幅、ジャーナルレコード幅、システム構成スロット幅がいずれも `io_min` 基準で決定されていない）、衝突と 3 つの候補は C212 (固定構造の書き込みがio_min未満) に記録、未選択；確定項目 19 のマウントゲートで第 1 版が (a) のみ設定するのは**認識の上での受容**である。

**根拠**:

- E129 (io_min未満の書き込みが実デバイスで引き起こす問題): ① 位置確定型の断裂書き込みインジェクタで断裂を生成可能（`dm-linear` 正常区間 + `dm-error` エラー区間、境界跨ぎ書き込みでエラー、プレフィックス着弾、サフィックス未着弾、5/5 ラウンド）⇒ 「ローカル環境では故障注入で反証不能」という全称命題は不成立、`dm-flakey` が断裂を生成できないという半文は依然正しい（バイト破損は生成するが断裂は生成しない）；② 「デバイス内部 RMW の誘発」を実測——`dm-thin`（ブロック 64 KiB、`io_min` 65536 かつ `physical_block_size` 512、非 RAID）でブロック共有時の 512 バイト書き込みによりデータデバイスへ 129 セクタが書き込まれ、非共有時は 1 セクタ。
- ユーザー決定事項および D22 (ユニット原子性の合成方法) 逆推 C が暴露した箇所（「我々はリード・モディファイ・ライトを行わない」は「我々はデバイスにリード・モディファイ・ライトを誘発させない」をカバーしない）、原文は変更履歴にある。

**未済**: C82 (io_min 不在挂载比对清单)、C212 (固定構造の書き込みがio_min未満)、C254 (物理マッピングユニットの利用先多数とchecker網羅ゼロ)。

#### 確定項目 17: ライフサイクルの異なる 2 つのオブジェクトによる同一物理マッピングユニット共有の禁止

**決定事項**: **ライフサイクルの異なる 2 つのオブジェクトが同一の物理マッピングユニットを共有してはならない。**「ライフサイクルが異なる」は確定項目 15 の判定基準に従い判定（破棄トリガーイベントが常に同一コミット ⇒ 同一ライフサイクル）。

**適用範囲**: **io_min 内の固定構造のみをカバー**（ルートスロット、システム構成スロット、墓碑の類）；16 KiB ノードおよび 32 KiB ユニットは適用外とし、アロケータのアラインメント変更はゼロ。理由文には実測の裏付けが一切ない：実測した 2 つのメカニズムの隣接領域はいずれも無傷——E34 (ルートリングスロット幾何) 実デバイス側の記述通り「同一 chunk 内の隣接スロット B（セクタ位置 1）**10/10 無傷**」、故障ユニットの記述通り「『同一 stripe 行 × 同一セクタ位置』の脚跨ぎであり、**`io_min` ユニット全体ではない**」；E129 (io_min未満の書き込みが実デバイスで引き起こす問題) が `dm-thin` の RMW を精密に引き裂いた際、同一マッピングユニット内の隣接領域は 5/5 ラウンドでバイト単位で完全に無傷、書き込み側はエラーを受信（`dm-thin` のブロック内 COW は**トランザクショナル**：新ブロックを割り当てコピー完了後にマッピング切り替え）⇒ **「停電が同一マッピングユニット内の永続化済み隣接データを破壊し得る」という主張は実デバイス上でメカニズムごとに成立または不成立であり、全称命題ではない**；一度も注入されたことのないメカニズム——SSD FTL 内部でのマッピングユニット全体に対する RMW のみが支えとして残り、E34 (ルートリングスロット幾何) に記録済み、E129 (io_min未満の書き込みが実デバイスで引き起こす問題) では未解決。

**根拠**:

- E129 (io_min未満の書き込みが実デバイスで引き起こす問題): 非 RAID の `io_min > physical_block_size` 実デバイスは実在し小サイズ書き込みが実際にデバイス側増幅を誘発、メカニズムの前提は成立；同一ラウンドで精密に引き裂いた際に隣接領域はバイト単位で無傷であり、理由文の全称命題はメカニズムごとの判定へと狭小化された。
- ユーザー決定事項および D22 (ユニット原子性の合成方法) 逆推 C が暴露した箇所、原文は変更履歴にある；適用範囲狭小化の文言は確定項目 15 のユーザー決定事項である。

**未済**: C254 (物理マッピングユニットの利用先多数とchecker網羅ゼロ)、C212 (固定構造の書き込みがio_min未満)。

#### 確定項目 18: システム構成における w_max と g の設定値は 4 / 4

**決定事項**: システム構成の `w_max` と `g` の両バイトとも 4 を書き込む（決定原文の数値）；`g` にランタイムクランプ `g ← min(g, 当時書き込み可能デバイス数)` を追加し、第 1 版 2 台構成時のグループはその 2 台とする。**第 1 版のディスク追加受入は `w ≥ 3` に到達可能なデバイス集合変更を拒絶し、ストライプテーブルが開通するまで `devs` の上限を 2 とする。****新規プールでのファイル作成のバイト列を変更するか：いいえ**——システム構成のその 2 バイトは変わらず 4 / 4 を書き込み、ディスク追加受入は稼働時のゲートであって、新規プールでのファイル作成は 2 台のみであり一度もトリガーされない。

**適用範囲**: `w_max` = 4 はシステム構成内の**宣言**であり、第 1 版のディスク上にはストライプテーブルが存在しない——`w ≥ 3` のストライプが書き出された後に当時どのディスクを使用したかやどのスロットがパリティかを記録する構造が存在せず（位置エントリが列挙するのは本データコピーの配置先であり、本ストライプのメンバー集合ではない）、その経路のリカバリセマンティクスは一切記述されていない；第 1 版 2 台構成では `w` は `clamp(…, 2, 4)` と書き込み可能デバイス数により定数 2 へクランプされその経路をたどらないが、3 台目を追加すると `clamp` は 3 を出力し得る ⇒ `w_max` を 2 に変更するのではなく（決定済みのバイトを変更し将来開通時に戻す必要があるため）、ディスク追加のステップで遮断する。**`w_max` に 4 を書き込むことは認識の上での保守的な選択であり、算術によって固定されたものではない**：数値を決定した限界算術には次元の誤りがあり（同次元修正後は同一等加重で 6）、等加重自体 kb 内に出典がなく、推導全体は `parity = 1` のもとでのみ成立；パリティの枠は未確定項目 21 であり、パリティ変更に伴い本バイトも再設定を要し、変更には D15 (フォーマット凍結ポリシー) の凍結階層に従う必要がある。

**根拠**:

- E63 (アロケータの書き込み列数): 8 台構成において上限 4 と上限撤廃での期待データ損失 10.7% vs 25.0% は 4 を選択する保守的根拠となる；4 を変曲点と主張した限界算術には次元の誤りがあり、4 はそれによって固定されたものではない。
- ユーザー決定事項（4 / 4 はコード記述前総合レビュー当日に決定、ディスク追加受入条項は翌日のモーダル決定、いずれも三者検証を経ておらず覆され得る）、原文は変更履歴にある。

**未済**: なし。

#### 確定項目 19: ルートスロットおよびシステム構成スロットのスロット間隔

**決定事項**: スロット間隔 = **4096 を mkfs 検出の `io_min` の整数倍へ切り上げた値**（`io_min` が 2 の累乗のときは `max(4096, io_min)`）；システム構成幾何区画に「mkfs 時の `io_min`」と「固定構造スロット間隔」の 2 つの 4 バイトフィールドを追加；`io_min` は D12 (目標メディア) 確定項目 4 のマウント時比較リストに入る。マウントゲートは**並列する 2 条件**であり単一ではない：**(a) 実測 `io_min` ≤ 記録スロット間隔**、不成立時は書き込み可能マウント拒否——確定項目 17（固定構造の同一マッピングユニット共有）を管轄；**(b) 実測 `io_min` ≤ 固定構造の書き込み幅**——確定項目 16（`io_min` 未満の書き込み禁止）を管轄。**第 1 版は (a) のみ設定し、(b) は設定しない。**

**適用範囲**: `io_min` = 4096、`physical_block_size` = 512 の通常の NVMe は (a) を通過するが（4096 ≤ 4096）、ルートスロットおよびシステム構成スロットの書き込みは依然 512 バイト < `io_min` ⇒ 確定項目 16 に文面通り違反し、これは**認識の上での受容**である。現状と 3 つの候補は C212 (固定構造の書き込みがio_min未満) に記録され未選択。`max` ではなく整数倍を採用した理由：`io_min` = 3072 のような非 2 の累乗の幾何において 2 つのスロットが同一マッピングユニットを共有してしまう（三者検証第 1 ラウンド逆推脚が構築）、整数倍とすることでスロット間隔以下の任意のスロット幅で共有が発生しない。インデックスからオフセットへの換算、ルートリングベースアドレス、領域起点は D22 (ユニット原子性の合成方法) 確定項目 16 第 2 文にある。`crates/` はシステム構成の該当 2 フィールドとスロット間隔計算式を実装（`crates/singlefs-core/src/system_configuration.rs` の `SystemImmutableSizes::slot_spacing_for`、単体テストで「io_min = 3072 時スロット間隔 6144」を固定）、マウント時に実測 `io_min` と記録スロット間隔を照合するステップは未実装（呼び出し側の mkfs パラメータとディスク記録の一致照合のみ存在）。`io_min` が過大で固定構造同士が重複するデバイスは mkfs で拒絶し非サポート（ユーザー決定事項としてサポート拒否、原文は変更履歴にある）：S = 8 ではスロット間隔 384 KiB を許容し 512 KiB 以上を拒絶（ルートリング領域 0 と領域 1 が重複）、`io_min` = 1 MiB ではシステム構成スロット 1 とルートリング領域 0 がともに 1 MiB から開始し S の値にかかわらず拒絶；根拠は三者判定 `research/prompts/m3-admission-r1-main-verification.md` 第三節 C568 (固定構造厳格化後にio_min 1 MiBディスクでボリューム作成不可) 行、未済の検査 C568 (固定構造厳格化後にio_min 1 MiBディスクでボリューム作成不可)。

**根拠**:

- E129 (io_min未満の書き込みが実デバイスで引き起こす問題): 実デバイス上に `io_min` が `physical_block_size` を大幅に超過するブロックデバイスが実在し（`dm-thin` ブロック 64 KiB、`io_min` 65536、`physical_block_size` 512）、そのようなデバイスでは小サイズ書き込みがデバイス側増幅を誘発 ⇒ スロット間隔は検出された `io_min` で切り上げる必要があり、`physical_block_size` のみで区切ると 2 スロットが同一マッピングユニットに入る。
- 三者検証（システム構成スロット幅ラウンド）第 1 ラウンド逆推脚が `io_min` = 3072 の幾何で 2 スロットが同一マッピングユニットを共有するケースを構築、資料と出力は `system_configuration-slot-width-reverse-opus.md`、`system_configuration-slot-width-reverse ラウンドの脚報告（git 内の成果物）`。
- ユーザー決定事項（コード記述前総合レビューおよび翌日のクロージングモーダル決定）、原文は変更履歴にある。

**未済**: C82 (io_min 不在挂载比对清单)、C212 (固定構造の書き込みがio_min未満)、C414 (マウント時のio_minと記録スロット間隔の不照合)。

#### 確定項目 20: ストライプテーブルとコンテナインデックスは day-1 ツリーテーブル登録を行わない

**決定事項**: ストライプテーブルとコンテナインデックスは第 1 版ではツリーテーブルに登録**しない**；`w ≥ 3` 開通時に新規ツリー ID と新規種類コードを取得する。確定項目 12 の「新規の予約ツリー ID 登録が必要」という記述は「開通時に新規番号を取得」と読み替える。

**適用範囲**: 登録のタイミングを決定し、媒体（確定項目 12）やキー（確定項目 14）は決定しない。第 1 版 2 台構成は常に `w = 2` であり、ストライプテーブルとコンテナインデックスはディスク上に 1 行もレコードを書き出さず、登録の有無にかかわらずバイト列は出力されない。

**根拠**:

- ユーザー決定事項（コード記述前総合レビュー）、原文は変更履歴にある。
- 実験なし: 第 1 版においてこれら 2 つのツリーはディスク上で空であり、登録の有無はバイト列を出力せず、定量可能な量はない。

**未済**: C103 (コンテナインデックスの配置先なし)。

#### 確定項目 22: 全幅ストライプ書き込み、リード・モディファイ・ライトの完全排除

**決定事項**: **ストライプ幅は可変、毎回の書き込みは全幅書き込みであり、リード・モディファイ・ライトは絶対に行わない。** write hole は修復されるのではなく、存在しないよう設計されている。代償：空間効率のわずかな低下、小サイズ書き込みにおける浪費。受容。

**適用範囲**: 書き込み経路におけるストライプの発行方法を管轄し、幅の決定は確定項目 6、参加ディスクは確定項目 8、ユニットの配置は確定項目 10。デバイス側への拡張は確定項目 16 および確定項目 17（デバイス内部でのリード・モディファイ・ライトを誘発しない）。外部の事実（本プロジェクト内では未検証）：ZFS RAID-Z がこの手法の実現可能性を実証済み。

**根拠**:

- ユーザー決定事項（D2 (RAID ストライプ方針) 立項時）、原文は変更履歴にある。
- 実験なし: write hole が存在しないよう設計されていることはフォーマット層のメカニズム演繹であり本プロジェクト内に定量可能な量はない；ZFS RAID-Z は外部の事実であり、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` に従い根拠には含めない。

**未済**: なし。

```

**出处 `.claude/kb/decisions/02-RAID-ストライプ方針.ja.md:353-364`（整段抄，未转述）**

```markdown
### 未確定項目

| # | 項目 | 状態 |
|---|---|---|
| 21 | **パリティスロット数** | ストライプ内にパリティスロットをいくつ配置するかの条項が存在しない；第 1 版 2 台構成は常に `w = 2` ⇒ 幾何学的に 1 スロットのみとなり得るが幾何が強いた結果にすぎない。新規プールでのファイル作成のバイト列を変更するか：いいえ（2026-09-20、根拠：第 1 版の 2 台ミラーは `parity = 1` で実装され異なるバイト列を出力する別値は存在しない；決定すべきは `w ≥ 3` 開通時に何スロット配置するかであり、その時点で初めて分岐が生じる） フォーマット変更を伴うか：はい（2026-09-21、根拠：適用範囲の原文通り「上限は parity に応じて再設定」——システム構成内の `w_max` バイトの値は本項の回答に応じて変化し、parity=2 時は同一等加重算術で 7 となり、確定項目 18 は現在 4 を採用） **状態: 未確定。** **進捗: 未実装。** |

#### 未確定項目 21: パリティスロット数

**議題**: 1 つのストライプ内にパリティスロットをいくつ配置するか（`parity`）。第 1 版 2 台構成は常に `w = 2` であり幾何学的に 1 スロットしかあり得ない；`w ≥ 3` 開通時に何スロット配置するかはリポジトリ全体に条項が存在しない。**第 1 版の進行方針**: 決定せず、`parity = 1` として実装（2 台ミラー、`w = 2`）。**新規プールでのファイル作成のバイト列を変更するか：いいえ**——第 1 版は 2 台、`w` は常に 2 であり、システム構成内の `w_max` / `g` の 2 バイトは確定項目 18 により一意に決定される；本項目が変更するのは将来それら 2 バイトを再設定する際の入力であり、新規プールでのファイル作成で書き出される値は変更しない。

**適用範囲**: 本項を決定する前に確定項目 18 の現状を確認すること——`w_max` = 4 の推導には次元の誤りがあり（同次元修正後は同一等加重で 6）、等加重自体 kb 内に出典がなく、4 は認識の上での保守的な選択であって算術によって固定されたものではない；parity=2 時は 2 台同時破損でもデータ損失はなくリスク軸は 3 台破損へ移行し、同一算術は 7 を与える ⇒ 上限は parity に応じて再設定される。**D9 (暗号化) との連動**: パリティは暗号文に対して計算され、複数レプリカは同一の暗号文を N 回書き出す——N 回暗号化すると N 種類の異なるバイト列が得られてしまう。**参考（不採用）**: bcachefs はバックグラウンドで bucket 全体に対してイレースコーディングを実施し、フォアグラウンドは通常の複数レプリカ書き込み、バックグラウンドで十分な候補 bucket を蓄積した後にパリティを 1 回再計算して全ポインタをアトミック更新、パリティは不変データに対して 1 回のみ計算され同様に write hole は生じないが、ストライプ内に生存データが残るかぎり該当 bucket を再利用できない代償を伴う；将来 D8 (コアインデックス構造) が bucket モデルを採用する場合、D2 (RAID ストライプ方針) は再評価を行うべきである。

```

**出处 `.claude/kb/decisions/02-RAID-ストライプ方針.ja.md:365-368`（整段抄，未转述）**

```markdown
## 履歴

D2 (RAID ストライプ方針) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:1-6`（整段抄，未转述）**

```markdown
## D2 RAID 条带策略 —— 半定（一项未定）

**中文** · [English](02-raid-stripe-policy.en.md) · [日本語](02-RAID-ストライプ方針.ja.md)

D2（RAID 条带策略） 管条带这一格：一条条带有多宽、用哪几块盘、一个单元怎么摆进条带、成员表住哪、部分释放与降级怎么处置、设备集合变化走什么路径，以及向设备发写的粒度与固定结构的槽距。不管的：位置条目本身的字段与宽度在 D19（块指针的结构与宽度预算） 已定项 1 / 已定项 4；根槽与系统配置槽的槽几何、归属公式的地址空间在 D22（单元原子性怎么合成） 已定项 2 与已定项 16；介质分支与「设备 → 布局」表在 D12（目标介质）；一次写落哪个槽在 D3（空间分配） 已定项 8；准入不等式在 D28（挂载期承诺量） 已定项 1。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:35-48`（整段抄，未转述）**

```markdown
#### 已定项 1：「宽度可变」的粒度

**定案**：逐次可变。格式侧**物理位置逐个列出、各带设备身份**（D19（块指针的结构与宽度预算） 已定项 1），不引入「逻辑地址 + chunk 映射表」这层间接；分配侧同样逐次可变，不采纳「组内固定」。

**射程**：定的是粒度与「不引第二个权威结构」，不定每次写用几列（已定项 6），也不定参与哪几块盘（已定项 8）。依据不是性能：本工程从未测过条带宽度的性能（`.claude/kb/experiments.md` 里唯一沾条带的 E55（条带映射的算术在真设备上对不对） 测的是映射算术对不对），`.claude/rules/fs-design.md` 也明令判据里不许出现「这样更快」。组内固定唯一的格式收益（指针不必带 dev、可走 chunk 映射表）已被 D19（块指针的结构与宽度预算） 已定项 1 独立拿走，而 D19（块指针的结构与宽度预算） 的三条依据（D1（数据可移动性 / 反向索引） 已付移除设备的代价／不带 dev 要 `sys_chunk_array` 那种自举结构／宽指针代价小）一条都不引 D2（RAID 条带策略） 的粒度。

**依据**：

- 三方论证（材料 `_d2-item1-background.md`、`d2-item1-local-counterexample.md`，本地腿原始输出 `d2-item1-local.out`）：格式侧无有效反对；反推腿现查 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 与 [checks-owed.md](../checks-owed.md) 里没有任何一条依赖「同组内宽度一致」，btrfs 把 `num_stripes` / `type` / `stripe_len` 挂在 chunk 上（`include/uapi/linux/btrfs_tree.h` 的 `struct btrfs_chunk`）唯一买到的是省字段，按 `.claude/rules/fs-design.md`「比谁省不构成任何判据」不计入；正推腿指出 chunk 映射表是**第二个权威结构**，可与指针不符、要与分配同事务原子更新，checker 可判定性与崩溃点重放有界性两项上逐个列出反而占优，同时指出组内固定还买到第二样——把「当时可写的设备集合」这个易变输入的读取点从每次写压到「开新组」一点，那一格收进已定项 6；本地腿的两条反例建立在「逻辑偏移靠 stride 算术推算物理位置」这个错前提上，不采纳。
- 用户定案（分配侧也取逐次可变），原话在变更史。
- 无实验：条带宽度的粒度没有可量的量，承重的是「chunk 映射表是第二个权威结构」这条机制论证。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:49-62`（整段抄，未转述）**

```markdown
#### 已定项 2：各盘不必等大

**定案**：池内各盘不必等大。池成员集合与各盘几何在**挂载时**校验，判不出安全组合就**拒绝挂载**。

**射程**：只在「寻址正确性」这个子问题上成立。全条带写在小盘先写满之后「当时可写的设备集合」怎么选、条带怎么退化，不由已定项 2 覆盖，那两个输入归已定项 6 与已定项 8，不许拿已定项 2 去覆盖它。

**依据**：

- 两条已定条款的合取：D12（目标介质） 已定项 3 的判据逐字「**一个几何量能不能在池内异构，看用到它的地址算术是否已经携带设备身份**」，而 D19（块指针的结构与宽度预算） 已定项 1 已定「位置条目**带**设备身份」⇒ 物理位置逐个显式列出，不再有靠几何公式推算位置的余地。
- 外部事实（2026-08-27 现查 Linux 6.17，未在本项目验证）：md/raid0 那次跨 5 年的静默损坏，触发链条正是「不带设备身份、靠公式推算」。
- 无实验：这一项是两条已定条款的合取，本工程没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:63-75`（整段抄，未转述）**

```markdown
#### 已定项 3：设备集合变化走显式「几何再求值」事务

**定案**：加盘 / 换盘 / 掉盘之后，挂载时算的那份几何由一条**显式事务**重新求值并原子发布，**卷保持挂载**；不采纳「强制重挂」。

**射程**：定的是方向，那条事务长什么样在已定项 4；被排除的第三种结局（一个运行时几何变更窗口，撞 `.claude/rules/fs-design.md`「分支变量不许在操作中途改变」）靠已定项 4 b 的互斥语义关上，不靠已定项 3。掉盘那一格与 [pitfalls.md](../pitfalls.md) 第 9 条（degraded 挂载会写出单副本 chunk，越救越糟）同域，而全条带写 + 位置条目逐个列出且各带设备身份让降级期间写下的条带盘上逐条自证，吃掉的是「静默」不是「要不要允许」——原子性不恢复冗余度，降级期间允许写还是只读由已定项 13 定。

**依据**：

- 用户定案（强制重挂对用户非常不友善，不采纳那条出路），原话在变更史；按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 记为机制推导，未三方论证。
- 无实验：只定方向，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:76-94`（整段抄，未转述）**

```markdown
#### 已定项 4：几何再求值事务的三样形态

**定案**：三样——

| # | 定案 | 展开 |
|---|---|---|
| a | **几何不合格就拒绝，不做全池再求值** | 加盘 / 换盘的准入判定与 D21（权威态与派生态的分界） 的挂载判定**同构**：返回「加得进来 / 加不进来」，不返回「慢一点」。⚠️ **卡的是几何量能不能安全组合**（`io_min`、原子宽度这一类），**不卡容量**——已定项 2 已定各盘不必等大，加一块更大的盘必须收得下。⇒ D21（权威态与派生态的分界）「运行时只查不算」原样不动 |
| b | **排空之后再换几何** | 再求值事务：关闸（新写排队）→ 排空（等已在飞的写全部落盘）→ 原子发布新几何 → 开闸。⇒ 全系统任何时刻**只有一份几何**，且**没有任何操作跨过切换点**。代价是一次有界的停顿，不是卸载 |
| c | **加盘后跑一次后台、可续做、可中断的重平衡** | 落在 `.claude/rules/fs-design.md` 第三格（后台、非决策路径，**无戒律**）⇒ 前台不停，与已定项 3「不中断业务」不冲突。无界重活 ⇒ 走「写意图 → 分批 → 可续做」。**搬迁单位是「整个对象重落一组」，不是单条条带**（依据在已定项 8） |

**射程**：b 把 C55（排空没排干净就换几何） 从「要拦」变成「结构上不可能」，剩下的是「排空真的排干净了吗」这一条可判定的检查；`.claude/rules/fs-design.md`「分支变量不许在操作中途改变」禁的那个窗口到 b 才真正关上。b 的停顿要有上界，而上界没有：排空要等的是「已在飞的写」，那个量由 I-8.1（环几何够大） 的 mkfs 计算给出，两者没有对接过，等事务层。c 与 D3（空间分配） 已定项 14 那条常驻整理是**两件活**：整理管碎片，重平衡管盘间均衡。引用 c 时要连这一句一起引：搁浅容量在 E60（加盘之后不搬数据的代价） 里归判据 5，而判据 5 只要求报出、没设阈值 ⇒ c 的定案方向由一个未设阈值的观测给出，「多大算不可接受」本工程一次都没定过；把搁浅立成带阈值的判据之前，不许说 E60（加盘之后不搬数据的代价） 按判据判出要重平衡。

**依据**：

- E60（加盘之后不搬数据的代价）：不搬数据时新盘只吸收得了「其余盘剩余空闲之和」，这是与分配策略无关的算术上界；4 盘填到 90% 再加一块同样大的盘，最好的不搬策略让新盘的 60% 搁浅，旧盘 95% 填充时搁浅 80% ⇒ 代价的量纲不是「慢」，是加的那块盘大部分买不到，c 那条重平衡因此要跑。
- 用户定案（a / b / c 三样都是），原话在变更史。

**欠**：C2（无界操作做成单事务）：c 的无界重活由它兜底；C55（排空没排干净就换几何）：b 之后只剩「排空真的排干净了吗」这一条检查。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:95-107`（整段抄，未转述）**

```markdown
#### 已定项 5：根环不重放置

**定案**：加盘 / 换盘之后**不重放置根环区域**；已定项 4 c 的重平衡搬用户数据，不动根环。

**射程**：代价两个方向不同，而且不是同一个。加盘（devs 变大）不会撞同盘（P 素数且大于任何合理设备数 ⇒ `gcd(P, devs) = 1` ⇒ 乘 P 是模 devs 的双射，只要 `R ≤ devs` 归属就两两不同），真正的问题是「按当前 devs 算会指到另一块盘，而那块盘上没有那份数据」，这一格已由已定项 7 消掉——归属改成 mkfs 时写下的设备身份之后，加盘不再改变任何既有区域的归属，撞同盘只剩 mkfs 那一刻 `R > devs` 的鸽笼，而那一刻 mkfs 判得出来、可以拒绝。**剩下的退化只有一种**：区域所在的那块盘没了——那是丢失不是撞。掉盘 / 换成更少的盘（devs 掉到 `< R`）必撞，鸽笼原理，与放置规则无关，E47（根环失败域的损失） / E48（根环几何的可行点） 那套「丢任一盘恒挂得上、最坏回退 1 代」在这里失效。区域的字节是 mkfs 时划死在盘上的既成事实。

**依据**：

- E62（根环归属：存身份 vs 算公式）：存身份臂指错盘五轮全部为 0、公式臂 8 步设备集合变化里 2 / 4 / 6 / 17 / 19 次指错盘，额外读 0、字节代价 4 字节 ⇒ 加盘方向那笔代价由已定项 7 消掉，不重放置剩下的只有掉盘那一格；阴性对照里 mkfs 那一刻 `R > devs` 时两条臂都撞，存身份解决不了鸽笼，失败域那一半由已定项 5 承担。
- 用户定案（加盘 / 换盘之后不重放置根环区域），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:108-121`（整段抄，未转述）**

```markdown
#### 已定项 6：每次写用几列

**定案**：`w = clamp(攒批后的写入量 + 1, 2, 4)`，再夹「当时可写的设备数」。**「攒批后的写入量」按格宽分桶各算各的**（已定项 10 依据 2：一条条带的全部格必须等宽）——一次发布里 16 KiB 与 32 KiB 两桶各自算出自己的 `w`。**上界 4 是系统配置声明的常量**，与设备数无关；`w ≥ 2` 是硬下界（零冗余的条带不许发出）。

**射程**：`w` 与已定项 8 的组大小 `g` 是两个量，恒有 `w ≤ g`；两者初值都是 4 是巧合，改一个不自动改另一个。第一版 `w` 恒 2 并被已定项 18 的加盘准入钉住，`clamp` 的上界 4 在第一版永远碰不到——它是**声明**，不是第一版走得到的取值。承重依据是 `.claude/rules/fs-design.md` 的硬要求 5（分支变量不许在操作中途改变：上界若写成 `f(设备数)`，设备数在写飞行中就变，掉盘不走已定项 4 b 的排空屏障）与硬要求 2（每条分支必须能被测试强制进入：常量把取值集封闭成 `{2,3,4}`，三条分支各自可达），不是边际算术。三处必须写明的不知道：① 「拐点 4」那套边际算术有**量纲错误**，风险项按每条条带、成本项按每份容量，同量纲的度量是期望丢数据比例 `(w−1)/C(D,2)`，**线性不是二次**，同一等权给 D=8 时上界 **6** ⇒ 4 是更保守的那一侧，不由那套算术钉住；② 等权（1 个百分点空间 ↔ 1 个百分点丢数据）在 kb 里没有出处，λ=1 给 6、λ=2 给 3、λ=0.5 给 5；③ 整套推导**只在 `parity = 1` 下成立**，parity=2 时双盘同坏不丢数据、风险轴要换成三盘同坏，同一算术给 7 ⇒ 上界随 parity 重定，而 parity 几格是未定项 21。

**依据**：

- E63（分配器每次写用几列）：小写占比 90% 时「贴合写入量」比「恒取全宽」省 61% 物理格 ⇒ 判据必须含写入量；8 盘攒批下上界取 4 与取消上界（＝8）在双盘同坏时期望丢掉 10.7% 对 25.0%，取消上界只省 13% 空间却换 2.3 倍的期望丢数据，而且每条带命中率 1.000 ⇒ 没有一个文件是完整的，失败模式从「部分受损」变成弥散。
- 三方论证：正推腿接受收窄版并抓到那套边际算术的量纲错误，反推腿判「D2（RAID 条带策略） 原文写了三样要一起给，只给一个数不算定案」⇒ 本次只定判据与上界、参与子集拆去已定项 8，本地腿主张按池算、它的小池反例在量纲修正后弱化（D=4 时 3→4 打平），原始输出 `d2-item6-local.out`。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:122-135`（整段抄，未转述）**

```markdown
#### 已定项 7：根环归属逐区域显式存设备身份

**定案**：归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来。第一版三个区域的归属写死 **0 / 1 / 0**（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域写进系统配置那 12 字节（每区域 4 字节设备身份）；**它不是 mkfs 参数**。

**射程**：与已定项 1、D19（块指针的结构与宽度预算） 已定项 1 同型——本工程第三次把「靠公式推算位置」换成「逐个显式列出」。买到的是「不指错盘」，不是「失败域独立」：掉盘掉到 `devs < R` 之后失败域保证照样失效，那一半由已定项 5 承担。字段落在系统配置哪一段、区域起点怎么算在 D22（单元原子性怎么合成） 已定项 16；暖机空发布的次数由这个归属唯一确定（txg 1 落区域 1 = 盘 1、txg 2 落区域 2 = 盘 0，两次覆盖两块盘），所以它与新池新建文件的 txg 3 一起是格式常量，见 D22（单元原子性怎么合成） 已定项 16 第 5 句。**改新池新建文件的字节：否**——系统配置那 12 字节本来就写 0 / 1 / 0，0 / 1 / 0 这一句补的是条款。

**依据**：

- E62（根环归属：存身份 vs 算公式）：8 步设备集合变化里公式臂指错盘 2 / 4 / 6 / 17 / 19 次、存身份臂五轮全 0，额外读 0（身份与系统配置同址，系统配置本来就要读），字节代价 4 字节（4 区域 × 1 字节）。
- E87（固定结构的放置）：「两盘上 R=3 必有一盘背两个区域」——配 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界，区域编号一定之后 0 / 1 / 0 是唯一的规范写法；做成 mkfs 参数只买到「哪块盘背两个」这一格，而两盘同构下它没有可观测差别。
- 用户定案两次（归属改成存身份、第一版三个区域写死 0 / 1 / 0），后一次没有走三方论证、可推翻，原话在变更史。

**欠**：C182（两条已定项对根环落盘归属说反话）、C217（P 大于设备数这条约束的理由没了而约束还在）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:136-149`（整段抄，未转述）**

```markdown
#### 已定项 8：参与哪些设备

**定案**：建对象时按当时最空的 `g` 块盘选一组；此后该对象的每条条带只在组内取最空的 `w` 块（`w` 由已定项 6 给）；组内有盘满了就重选一组。`g` 是系统配置声明的常量，初值 4。**组不落盘**——它由该对象位置条目的设备身份**并集**推出，零新增字节。已定项 4 c 的重平衡以「对象」为搬迁单位、整组重落，不许逐条带搬。

**射程**：`g` 与 `w` 是两个量，恒有 `w ≤ g`，两者初值都是 4 是巧合；完好比例只跟 `g` 走（`1 − C(g,2)/C(D,2)`），空间开销只跟 `w` 走（`1/(w−1)`）。另存一份「对象 → 哪几块盘」就是微缩的 chunk 映射表，即已定项 1 判过的第二个权威结构，而 D18（块里携带什么信息） 已定项 3 的禁放清单也明令物理落点 / 设备 ID 不许进块头 ⇒ 组是分配器的派生态；落点本身以中央映射为准，位置条目是提示、中央映射才是解引用与释放判定的唯一入口（D19（块指针的结构与宽度预算） 已定项 5），读到它可以直接去那个落点试，但它可能过期。两处必须写明的不知道：① **单对象并行度上限 = `g`**，一个对象钉在 4 块盘上 ⇒ 它的并行度是 4 不是设备数，这一维没量过，而 D25（目标负载优先级） 的主负载是顺序大文件，量到之前不许把 78.6% 当净收益引用；② 只测了 `g = w = 4` 这一个点，`g=5` 给 64.3%、`g=6` 给 46.4%，`g` 变了要重测。

**依据**：

- E67（一条条带参与哪几块盘）：8 盘、宽度 4、512 对象、两盘同坏穷举全部 28 对，本项定的规则给完好对象比例 78.6%（＝`1 − C(4,2)/C(8,2)`，单测钉死）且盘间极差 0，而每条带轮转只剩 14.3%、每条带取最空 57.1%（均匀）/ 77.2%（倾斜）、按对象号绑组虽然也是 78.6% 但倾斜轮盘间极差 88.7%（会提前 ENOSPC）；六条臂的期望丢数据比例逐格相等（`3/28`，只由宽度定）⇒ **选法决定的是损失怎么分布，不是丢多少**。第四轮加第 9 块盘并重平衡，整对象搬给 83.3%（＝`1 − C(4,2)/C(9,2)` 的理论值，一格没丢），逐条带搬掉到 75.0% ⇒ 这是「以对象为搬迁单位」那一条的依据。
- 三方论证：正推腿接受收窄版（组不落盘、重平衡以对象为单位、`w<g` 时取组内最空的 `w` 块）并指出 `g` 与 `w` 是两个旋钮；反推腿要求补测重平衡与「此后固定」的结构性对撞，已补（E67（一条条带参与哪几块盘） 第四轮）；本地腿的反例「各盘不必等大 ⇒ 可能钉在四块小盘上」在异构容量轮里没有兑现（按剩余空间选组与逐条带选最空逐格相同，1148 对 1148），该轮输出含拼接损坏词、按带瑕疵的一票记，原始输出 `d2-item8-local.out`。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:150-163`（整段抄，未转述）**

```markdown
#### 已定项 9：第一个可运行目标跑 2 块盘

**定案**：第一版不跑单盘，第一个可运行目标跑 2 块盘；单盘走单独一条线，那条线开线时再定。

**射程**：裁掉的互斥是——**已定项 6 的 `w ≥ 2`（零冗余的条带不许发出）与「再夹当时可写的设备数」在单设备上互斥**，一块盘时前者要 2 列、后者只给得出 1 列。它顺带把 D18（块里携带什么信息） 已定项 4（单盘自恢复的形态）移出阻塞集：那一项随「单盘布局」那条线一起开，开线之前写不出任何字节；⚠️ 那条线开线时要先还它的举证——**没有任何现役实现走「任意点前后链式恢复」这条路**，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 举证责任压在这一侧，而举证一点没做。做得到：`.claude/kb/vm-harness.md` 那套 QEMU 现场造两块盘就够，不需要真硬件。

**依据**：

- D2（RAID 条带策略） 已定变宽全条带需 ≥2 盘 ⇒ 单盘本来就不在第一版的布局里；硬要绕就得为单盘另开一套布局（D12（目标介质） 已定每套布局各占一个 incompat 位），而 `.claude/rules/fs-design.md` 逐字「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」。
- 用户定案，原话在变更史。
- 无实验：这一项是两条已定条款在单设备上互斥的裁决，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:164-176`（整段抄，未转述）**

```markdown
#### 已定项 10：一个单元整个落在一列上

**定案**：一个 32768 字节的单元不切开、不横跨多列；位置条目每个副本一条。

**射程**：依据是一条算术加两条已定条款。**格 = 单元**（D4（校验和位置） 已定项 2 / 3 把最小写钉成一个整单元），而单元有两档（D18（块里携带什么信息） 已定项 9：元数据 16384、数据 32768）⇒ 格也有两档；**一条条带的全部格必须等宽**，逐位异或才有定义 ⇒ 分配器按格宽分桶攒批，已定项 6 那个「攒批后的写入量」按每桶算，不是每次发布的总量。横跨读法在 `w = 4` 上当场破：`32768 / 3` 不是整数，也不是 `io_min` 的倍数，撞已定项 16。D19（块指针的结构与宽度预算） 依据三那段按「头部 + 4 + 2 条带六列」算的指针占盘比例是在横跨读法下算的，那个算法不成立，但它论证的是「定宽指针的代价可接受」，**结论不受影响**——按这一项的口径重算只会让指针摊得更薄（位置条目从 w 条降到 1 条 / 副本）。已定项 10 与已定项 6 的合取留了一个 `w ≥ 3` 才存在的洞（条带成员被释放之后幸存成员的重建输入没了），E80（条带的部分释放） 量过、由已定项 11 收口；`w = 2` 对照全零 ⇒ 第一版的全部测试对它全盲。

**依据**：

- 用户定案，原话在变更史。
- 无实验：横跨读法在算术上当场破（`32768 / 3` 不是整数），承重的是一条算术加两条已定条款，没有可量的量。

**欠**：C81（条带部分释放没有规则）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:177-193`（整段抄，未转述）**

```markdown
#### 已定项 11：条带的生命周期——部分条带钉住、后台惰性重落

**定案**：三条规则——

1. **w ≥ 3 条带的成员被释放后，该格与整条带（含 parity）在修复完成前不得复用**。被钉住的格计入「不可回收字节」统计量（D5（快照 / 空间记账机制） 已定项 4 第 3 项，现成落点）——钉住量因此天然可观测，满足 `.claude/rules/fs-design.md` 硬要求 4。
2. **修复 = 后台把幸存者整批重落一条新条带，旧条带整条回收**，挂 D26（后台整理与放置回收） 腿一（常驻整理）的队列与预算跑。修复队列是**派生态**（可从分配记录与条带成员重建，丢了只是慢）。
3. **w = 2 的条带成员一死即整条回收**（1 数据 + 1 副本，无跨单元耦合）⇒ 第一版（2 盘恒 `w = 2`）自动满足，零新增代码路径。

**射程**：「修复」这个词在已定项 11 里只管一类——**条带的成员被释放**。掉盘之后重建一个丢失的格是另一件事，形态未定（它要的正是已定项 12 那张成员表），两者不能合读：规则 2 那句「整批重落**一条**新条带」只在释放这一类上恒成立（释放剩下 k 个幸存者时 `w = clamp(k + 1, 2, 4)`，一条 `w` 宽条带恰好装 k 个数据格，`k ≤ 3`），而降级场景下按字面不可执行——`w` 还要夹当时可写设备数（已定项 6），4 盘掉 1 块之后 `w ≤ 3`，一条新条带只装 2 个数据格，3 个待重落的单元装不下 ⇒ 掉盘重建那一类要单独定，落点是已定项 12 与它的后继。代价与已知未知，引用时连着引：额外写 +47% 是均匀随机重写负载下的量级指示、不是预测；修复预算的下限没扫（预算 2 恰好跟得上模型负载）；「幸存者并进前台写的攒批」这个更省的形态留给实现，E80（条带的部分释放） 没建模。

**依据**：

- E80（条带的部分释放）（两轮，5 种子 × 2 负载）：四种处置里只有「钉住 + 后台惰性重落」同时做到丢失恒 0、钉住有界（预算 2 条/轮下轮末恒 0、停负载排空归零）、比立刻修便宜约 37%（额外写 +47% 对 +75%）；放任复用时单盘失效平均丢 9.8–13.0 个活单元且盘坏前零报错，死透才回收钉住活数据 33–46% 无界累积，成员一死立刻修的修复债自我更新（约 4000 次修复事件）。
- 用户定案，原话在变更史。

**欠**：C81（条带部分释放没有规则）：镜像侧判「任一活单元的条带成员要么字节原始、要么整条带死透」；C127（钉住状态在分配记录里没有落点）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:194-219`（整段抄，未转述）**

```markdown
#### 已定项 12：成员表住码 3 打包记录，走容器索引，容器恒走镜像

**定案**：条带成员表的记录（定长 56 字节，一格一条、含 parity 格）住**码 3 打包记录单元**，由一棵**容器索引**指认容器；**容器整条链恒走 `w = 2` 镜像**。态别是**权威态**：按 D21（权威态与派生态的分界） 自己的判据「丢了就丢数据 / 丢了只是慢」，这张表丢了会让重建做不成、让分配器判不出钉住 ⇒ 必须住在单元的载荷里，而住在单元载荷里意味着它仍是权威态清单的第一项，不扩项。**改新池新建文件的字节：否**（第一版 2 盘恒 `w = 2`，parity 即第二副本，位置条目每副本一条已覆盖发现 ⇒ 新池新建文件不写条带记录）。

**射程**：定的是载体，不是排序——条带表这棵逻辑树按什么 key 排单列在已定项 14。**为什么容器恒走镜像**：不镜像时稳态下一次单盘失效的不可解格数恒 0，但**池刚建好那一格恒 1**——条带表只有一个容器时它落在自己描述的那条条带里，容器所在盘一坏就无论按什么顺序都解不开，五个种子全中，是结构性的不是概率性的；镜像把这条自举边结构性地关掉。取这条的理由不是空间也不是写字节：空间那一维按 `.claude/rules/fs-design.md`「比谁省不构成任何判据」不进判据，写字节那一维是平的（主负载只差 4.7%，混合负载按字节 0.7816、甲省 22%，但按轮数甲只在 65.8% 的发布上更省）⇒ 取它的理由是它让「一次单盘失效之后坏盘上的格还能不能重建」有确定答案，而 `w` 恒 2 全镜像那条路直接放弃 `w ≥ 3`。

代价，逐条：

| 维 | 数 | 出处 |
|---|---|---|
| 稳态空间 | **3087 ppm，`w = 3` 与 `w = 4` 同值**（16 TiB / 90% 占用、含镜像；不镜像 1543）——条目数由那笔 90% 预算定，`w` 只改预算里数据与 parity 的配比 | E111（条带表的 key 与它的几何），与 E106（条带成员表的载体） 臂甲镜的 54 315 515 904 字节逐位相同 |
| 每次发布写字节 ÷「`w` 恒 2 全镜像」 | 主负载**期望 0.9531**（省 4.7%）；容器将满那一格 1.050 | E110（条带表在连续发布下的期望写放大） |
| 条带表那一份 | **163 840 字节 / 次发布，与叶数无关**（容器 32768 + 容器索引 3 层 × 16384，整条链镜像） | E107（条带表对主负载的写放大） |
| 三段写序的延迟 | ack 语义差在噪声内；durable 语义 **1.625×** | E107（条带表对主负载的写放大） 乙臂 |
| 单盘失效不可解格数 | **结构性恒 0** | E106（条带成员表的载体） 臂甲镜 |

**依据**：

- E106（条带成员表的载体）：成员表住码 3 打包记录时稳态不可解格数恒 0、而池刚建好那一格恒 1 且五个种子全中 ⇒ 容器必须镜像；三条臂的稳态空间也在同一份产物里。
- E107（条带表对主负载的写放大）：每次发布因维护成员表多写的字节与三段写序的延迟，这两笔 E106（条带成员表的载体） 明确不答。
- E110（条带表在连续发布下的期望写放大）：把 E107（条带表对主负载的写放大） 那条刀刃解掉——容器占用槽位数交给负载自己演化之后，主负载的累计比值稳住，容器将满那个端点只占很小一部分发布。
- E111（条带表的 key 与它的几何）：按落点排的条目数与 `w` 无关，稳态空间与 E106（条带成员表的载体） 臂甲镜逐位相同 ⇒ 代价那一行收口。
- 用户定案，原话在变更史。

**欠**：`w ≥ 3` 开线之前必须还清四笔——C103（容器索引无落点）（容器索引这棵树住哪、key 编码、它自己的更新代价、它独占的 MAC 重建时怎么重算）；**自举**（容器索引自己的节点由谁指认，要**新登记一个保留树 ID** 才关得掉，而已定项 20 已定不 day-1 注册、开线时取新号）；C127（钉住状态在分配记录里没有落点）；C128（清扫准入对 parity 格两支都判不出）。另有 C303（D2 边界 ③ 的依据随 D9 已定项 7 撤回而失效）与 C155（跨实验同量无交叉断言）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:220-240`（整段抄，未转述）**

```markdown
#### 已定项 13：降级期间只读

**定案**：降级期间**只读**。判据写成可写挂载准入的一条合取，并配一条运行期的闸：

| 时刻 | 判据 |
|---|---|
| 挂载准入 | 可写设备数 ≥ w 的下限 ∧ **独占打开池中过半的设备**（过半是门槛；新实例代号要写进独占打开成功的**全部**设备，不只是过半那几块）∧ 实例表可读 ∧ 实例切换的预留拿得到（那个量的式子在 D28（挂载期承诺量） 已定项 3）；任一不成立即只读挂载；例外：空间那一条（实例切换的预留与 D28（挂载期承诺量） 已定项 1 的准入）在取号之前不够时，照 D16（发布语义） 已定项 1「准入」那一行先写行、再推抬 F 的空发布、再判，推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（用户 2026-09-26 定，被攻过零轮） |
| 运行期 | 可写设备数掉到 w 的下限以下、**或独占打开成功的设备数掉到过半以下**即转只读。**「可写设备数」的口径**：一台设备算不可写，当且仅当对它的固定落点做一次探针写失败（与 D23（journal 的角色与格式） 已定项 14 的两支判别子同一个动作）——不按失败次数、不按失败时长算，那两样在故障发生那一刻都判不出来 |

**设备集合一变就复判**：已定项 3 的「几何再求值」事务里加一条——设备集合变化之后重新判过半与取号的写入集合，不成立即转只读。

**射程**：定的是「降级期间能不能挂成可写、以及挂上之后什么时候转只读」，不定「降级期间已经写下去的条带怎么补」——那是已定项 11 的部分条带钉住与后台惰性重落。**为什么是过半而不是「可写设备数够就行」**：C113（扫描重建时多版单元的现行版本判定无输入） 定案要求同一时刻只有一个可写挂载（两个并发可写挂载会算出同一个实例代号，已发布谓词的三支一支都分不出它们），而块设备独占打开是**每设备**的排它，要把它抬成**池级**互斥就得让任意两个可写挂载的设备集合必相交，过半是最松的那个条件；盘数 ≤ 3 时任意两个够 w 下限的集合本来就相交，盘数 ≥ 4 之后两个互不相交的 2 元子集就存在了，那时过半是唯一还成立的那道闸。代价：4 盘池只剩 2 块可写时不能挂成可写，挂上之后掉到 2 块也转只读，按旧判据（只看可写设备数 ≥ w 的下限）两种情形都是可写的——这是拿可用性换「两条时间线不会共用一个实例代号」，不许挂在别处的括号里当成「与已定项 9 同一取向」，已定项 9 讲的是第一版跑几块盘，与降级期间可写不可写无关。不复判的话，在线加盘会把池的基数抬高而挂载持有的独占集合不变，从未被这次挂载打开过的健康盘就永远拿不到新实例代号，按「错过取号的盘」那条要整盘作废重同步。

**依据**：

- E141（切换预留的挂载准入自证）：挂载准入里「实例切换的预留拿得到」这条合取，在只差 1 块的两个池上分得出可写与只读，预留置 0 时两个都可写 ⇒ 这条合取有判别力，不是摆设。
- 随 C113（扫描重建时多版单元的现行版本判定无输入） 定案（那一轮的第七、第八轮反推腿 5.1 与 7.2 现查坐实）与用户定案，原话在变更史。
- 空间那一条的例外：用户定案，原话在变更史；实五实现与用例见 C565（挂载处推满仍不够怎么收尾没定） 在已还清表里那一行。

**欠**：C833（可写挂载没有独占打开）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:241-253`（整段抄，未转述）**

```markdown
#### 已定项 14：条带表按落点排，与分配记录树同一个坐标系

**定案**：key = **落点**（设备身份 4 字节 + 16 KiB 槽号 6 字节），与 D3（空间分配） 已定项 3 的分配记录树**同一个坐标系**（D3（空间分配） 已定项 3 依据第 2 条逐字「不许各定各的」）。**改新池新建文件的字节：否**（第一版 2 盘恒 `w = 2`，一条条带记录都不写 ⇒ 这棵树在新池新建文件里是空的，key 取什么都写不出字节；`w ≥ 3` 的第一条条带发出之前必须有它）。

**射程**：这一条不是量出来的，是三条候选 key 的定义直接给的——条带表的三个消费者问的是**同一句话**「给一个落点，它属于哪条条带」：分配准入（C127（钉住状态在分配记录里没有落点））判候选落点是不是被钉住的格、清扫准入（C128（清扫准入对 parity 格两支都判不出）判一个没有头的 parity 格该不该清、让渡区 A 的签发判据排除钉住的格与它所属条带的全部格 ⇒ 按条带号排、或者只按容器身份索引都答不出这句话；容器索引更答不了，它的 key 是 `(出生树 ID, 打包记录类型, 容器号, 容器出生代)`，四段里没有落点，容器号按 C103（容器索引无落点） 是「最大号 + 1」单调分配，与它装的记录覆盖哪些落点无函数关系。空间那一维不进判据（`.claude/rules/fs-design.md`「比谁省不构成任何判据」），摆出来只是让人知道付的是什么；进判据的是「答不答得出那句话」。仍欠：容器索引本身住哪（C103（容器索引无落点））——这棵树按落点排，而它的叶是码 3 容器，容器的物理指针由容器索引持有，容器索引自己的落点全仓无定义；这棵树自己的更新代价（每次发布改写多少条、COW 多少节点）E111（条带表的 key 与它的几何） 不答，那一格在 E107（条带表对主负载的写放大） 与 C103（容器索引无落点） 的射程里。

**依据**：

- E111（条带表的 key 与它的几何）：按落点排时 `w = 2` 一条记录都不写（0 ppm）、`w = 3` 与 `w = 4` 都是 483 183 820 条、树高 3、占 3087 ppm，同尺下分配记录树是 442 ppm ⇒ 这棵树是它的 6.98 倍；条目数与 `w` 无关，那笔 90% 预算切出多少格就有多少条记录。
- 用户定案，原话在变更史。

**欠**：C103（容器索引无落点）、C127（钉住状态在分配记录里没有落点）、C128（清扫准入对 parity 格两支都判不出）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:254-268`（整段抄，未转述）**

```markdown
#### 已定项 15：「生命周期不同的对象」的通用判据

**定案**：对占用同一物理映射单元的两个对象 X、Y，各自找出它的**死亡触发事件**——让它从「字节必须保持不动」变为「可以被覆盖 / 回收」的那一次具体提交。**若格式能保证 X 与 Y 的死亡触发事件恒为同一次提交，则同一生命周期，允许共享；只要格式允许两者的死亡事件是两个可以互相错开的独立提交，就是生命周期不同，不许共享。**判据钉的是「死」不是「生」。同时把已定项 17 的射程收窄到 io_min 内的固定结构（根槽、系统配置槽、墓碑一类），16 KiB 节点与 32 KiB 单元不在射程里，**分配器对齐零改动**。**改新池新建文件的字节：是**——它的裁决直接决定 D22（单元原子性怎么合成） 已定项 2 的槽几何，而根环区域是 mkfs 划在盘上的既成事实、改不了。

**射程**：**不能按「写入时刻」判**——同代际墓碑记录本来就是逐条 append 的，按写入时刻会判成「生命周期不同」，当场与 D18（块里携带什么信息） 已定项 10 的用户定案矛盾。两个已知消费方都验得过：墓碑——同代际记录的死亡事件都是「该代际 deadlist 整体退休」这一次判定 ⇒ 同生命周期，与 D18（块里携带什么信息） 已定项 10 吻合；根槽——D22（单元原子性怎么合成） 已定项 2 的槽序是跨区轮转（区域 = `txg mod R`）⇒ 一次 txg 只写一个槽 ⇒ 不同槽的下一次被覆盖是两个独立的未来提交 ⇒ 判「算」，而「算 ⇒ 必须改槽宽 ⇒ 撞 D20（承重面：单元的原子性与自包含）」这条推理断在第一步：槽宽与槽的起始偏移是 D22（单元原子性怎么合成） 已定项 2 自己分开的两行，把起始偏移抬到 `io_min`（I-7.5（根槽按判定宽度对齐） 的粒度参数）即满足已定项 17，槽宽仍等于 `physical_block_size`，不碰 D20（承重面：单元的原子性与自包含） 推论三那条类规则。回扫打出的消费方远不止两个（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「新立一条判据，当场拿它回扫已有的条目」）：索引节点（16 KiB）与数据单元（32 KiB）在 `io_min` = 64 KiB 的阵列上按这条判据都判「算」，而 COW 不构成豁免（块重用让邻格在任意晚的时刻被重写），journal 记录那一格比根槽更硬（只有两盘镜像）；现查 [invariants/02-根环.md](../../../crates/singlefs-checker/invariants/02-根环.md)：`io_min` 只出现在 I-7.5（根槽按判定宽度对齐） 一条里（判定宽度取 max(`physical_block_size`, `io_min`)，射程注写 `io_min` 大于 `physical_block_size` 时按 `io_min` 判），I-7.5（根槽按判定宽度对齐） 已实现（池级 checker 逐槽判根槽起始偏移），「物理映射单元」仍零命中 ⇒ checker 对已定项 17 只判根槽那一格，索引节点、数据单元与 journal 记录那几格不判，已定项 16 也不判。另一条出路（按字面改分配器对齐）不取；落点政策在 D3（空间分配） 已定项 8（用户数据不进聚簇段、取已选设备内槽号最小的空槽，它不管对齐）。已定项 15 与 C212（固定结构的写小于 io_min） 不是一件事：那一条压在已定项 16 上，而已定项 16 一个未定义的词都没有，所以那是两条已定条款打架，不是判据缺失。

**依据**：

- 三方论证 2026-09-10 那一轮：判据由正推腿给出，主 agent 逐个消费方代入核过（墓碑与根槽两处），材料与本地腿输出在 `_d2-item15-background.md`、`d2-item15 那一轮的腿报告（产物在 git 里）`。
- 外部事实（2026-09-08 现查，未在本项目验证）：ZFS 的根环槽宽逐字取 `MIN(MAX(vdev_ashift, UBERBLOCK_SHIFT), MAX_UBERBLOCK_SHIFT)`，跟设备对齐粒度走但封顶 8 KiB，用的量是 `ashift` 不是 `io_min`，见 [prior-art.md](../prior-art.md)「8.1 ZFS：根环槽宽确实随设备粒度抬，但封顶 8 KiB，而且抬槽宽是减槽数不是多占地」；一处已知差异是 ZFS 的环固定 128 KiB、本工程的区域大小不是格式常量。
- 用户定案，原话在变更史。
- 无实验：判据本身是定义推导，两个消费方靠代入核，没有可量的量。

**欠**：C254（物理映射单元的消费方远不止两个，checker 零覆盖）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:269-286`（整段抄，未转述）**

```markdown
#### 已定项 16：不发出小于 `io_min` 的写

**定案**：**不发出小于 `io_min` 的写。** 在 Linux 上这个量是 `io_min`，不是 `physical_block_size`；**两个量不许合成一句话**。

| 量 | 内核里怎么来的 | 它是什么 |
|---|---|---|
| `io_min` | `lim->io_min = phys_bs`，`phys_bs = bs * (1 + npwg)`（NPWG = Namespace Preferred Write Granularity） | **不诱发设备 RMW 的最小写宽度** |
| `physical_block_size` | `lim->physical_block_size = min(phys_bs, atomic_bs)`（`atomic_bs` 由 NAWUPF 定，未声明则等于逻辑块） | **设备承诺的掉电原子宽度** |

**射程**：机制是**向一个设备物理映射单元里发一次更小的写，设备内部会做 read-modify-write**。口径：2026-08-28 现查 `drivers/nvme/host/core.c` 的 `nvme_update_disk_info` 里算 `physical_block_size` 那一段，Linux 7.2.0，那个 `min` 上方的注释只讲原子性（「limit the physical block size to the value of the Atomic Write Unit **Power Fail** parameter」）⇒ `physical_block_size ≤ io_min`，按 `io_min` 发写照样可能低于 NPWG 而诱发 RMW。**要动这一条得走三方论证**，E129（小于 io_min 的写在真设备上怎么出事） 只提供观测；它与三条已定条款打架（根槽宽、journal 记录宽、系统配置槽宽都不是按 `io_min` 定的），冲突与三条候选记在 C212（固定结构的写小于 io_min），一条都还没选，而已定项 19 的挂载闸第一版只设 (a) 是**知情接受**。

**依据**：

- E129（小于 io_min 的写在真设备上怎么出事）：① 位置确定的撕裂注入器造得出（`dm-linear` 好段 + `dm-error` 错段，跨边界写报错、前缀落盘、后缀不落，5/5 轮）⇒ 「本机无法用故障注入证伪」那句全称不成立，`dm-flakey` 造不出撕裂这半句仍对（它造字节损坏，不造撕裂）；② 「诱发设备做 RMW」量到了——`dm-thin`（块 64 KiB、`io_min` 65536 而 `physical_block_size` 512，不是 RAID）在块共享时一次 512 字节的写让数据设备写 129 扇区，块独占时 1 扇区。
- 用户定案与 D22（单元原子性怎么合成） 反推 C 暴露的那一处（「我们不做 read-modify-write」覆盖不到「我们不诱发设备做 read-modify-write」），原话在变更史。

**欠**：C82（io_min 不在挂载比对清单）、C212（固定结构的写小于 io_min）、C254（物理映射单元的消费方远不止两个，checker 零覆盖）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:287-299`（整段抄，未转述）**

```markdown
#### 已定项 17：不让两个生命周期不同的对象共享同一个物理映射单元

**定案**：**不让两个生命周期不同的对象共享同一个物理映射单元。**「生命周期不同」按已定项 15 的判据判（死亡触发事件恒为同一次提交 ⇒ 同生命周期）。

**射程**：**只罩 io_min 内的固定结构**（根槽、系统配置槽、墓碑一类）；16 KiB 节点与 32 KiB 单元不在射程里，分配器对齐零改动。理由句一条实测支撑都没有：两条已量机制的邻居都完好——E34（根环槽几何） 真设备那一半逐字「同 chunk 内邻槽 B（扇区位 1）**10/10 完好**」，故障单元逐字「是「同 stripe 行 × 同扇区位」跨腿，**不是整个 `io_min` 单元**」，而 E129（小于 io_min 的写在真设备上怎么出事） 把 `dm-thin` 那条 RMW 精确撕开时同一映射单元里的邻居 5/5 轮逐字节完好、写方收到错误（`dm-thin` 的块内 COW 是**事务性**的：先分配新块、拷完才切映射）⇒ **「掉电可能损坏同一映射单元里已经持久的邻居数据」在真设备上是逐机制成立或不成立的，不是全称**；只剩一条从没注进去过的机制撑着——SSD FTL 内部对整个映射单元的 RMW，E34（根环槽几何） 已记、E129（小于 io_min 的写在真设备上怎么出事） 没解决。

**依据**：

- E129（小于 io_min 的写在真设备上怎么出事）：非 RAID 的 `io_min > physical_block_size` 真设备存在且一次小写确实诱发设备侧多写，机制前提成立；同一轮把它精确撕开时邻居逐字节完好，理由句的全称被收窄成逐机制判。
- 用户定案与 D22（单元原子性怎么合成） 反推 C 暴露的那一处，原话在变更史；射程收窄那一句是已定项 15 的用户定案。

**欠**：C254（物理映射单元的消费方远不止两个，checker 零覆盖）、C212（固定结构的写小于 io_min）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:300-312`（整段抄，未转述）**

```markdown
#### 已定项 18：系统配置里 w_max 与 g 写 4 / 4

**定案**：系统配置的 `w_max` 与 `g` 两个字节都写 4（决策原文的值）；`g` 加一条运行时夹取 `g ← min(g, 当时可写设备数)`，第一版 2 盘时组就是那两块盘。**第一版加盘准入拒绝让 `w ≥ 3` 可达的设备集合变化，`devs` 上限 2，直到条带表开线。****改新池新建文件的字节：否**——系统配置那两个字节仍写 4 / 4，加盘准入是运行期的一道闸，新池新建文件只有两块盘、一次都不触发。

**射程**：`w_max` = 4 是系统配置里的一个**声明**，而第一版盘上没有条带表——`w ≥ 3` 的条带写出去之后没有任何结构记得它当时用了哪几块盘、哪一格是校验（位置条目逐个列出的是这一份数据的落点，不是这条条带的成员集合），而那条路径的恢复语义一个字都没写；第一版 2 盘时 `w` 被 `clamp(…, 2, 4)` 再夹可写设备数压成恒 2，走不到那条路径，一旦加第三块盘 `clamp` 就可能给出 3 ⇒ 不是把 `w_max` 改成 2（那要改一个已经定过的字节、且将来开线时又要改回去），而是在加盘那一步拦住。**`w_max` 写 4 是知情接受的保守取值，不由算术钉住**：给它定值的那套边际算术有量纲错误（同量纲改正后同一等权给 6），等权在 kb 里没有出处，而且整套推导只在 `parity = 1` 下成立；parity 几格是未定项 21，parity 改了这个字节要重定，而改它要走 D15（格式冻结政策） 的冻结层次。

**依据**：

- E63（分配器每次写用几列）：上界 4 与取消上界在 8 盘上的期望丢数据 10.7% 对 25.0%，是取 4 这一侧的保守依据；那套把 4 说成拐点的边际算术有量纲错误，4 不由它钉住。
- 用户定案（4 / 4 是写代码前的总审核当日定的，加盘准入那一条是次日弹窗定案，都没有走三方论证、可推翻），原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:313-326`（整段抄，未转述）**

```markdown
#### 已定项 19：根槽与系统配置槽的槽距

**定案**：槽距 = **4096 向上取整到 mkfs 时探测的 `io_min` 的整数倍**（`io_min` 是 2 的幂时就是 `max(4096, io_min)`）；系统配置几何段加两个 4 字节字段「mkfs 时的 `io_min`」与「固定结构槽距」；`io_min` 进 D12（目标介质） 已定项 4 的挂载比对清单。挂载闸是**两条并列**，不是一条：**(a) 实际 `io_min` ≤ 记录的槽距**，不满足拒绝可写挂载——它管的是已定项 17（固定结构共享同一个映射单元）；**(b) 实际 `io_min` ≤ 固定结构的写宽**——它管的是已定项 16（不发出小于 `io_min` 的写）。**第一版只设 (a)，不设 (b)。**

**射程**：`io_min` = 4096、`physical_block_size` = 512 的普通 NVMe 过得了 (a)（4096 ≤ 4096），而每一次根槽写与系统配置槽写仍然是 512 字节 < `io_min` ⇒ 逐字违反已定项 16，这是**知情接受**，现状与三条候选记在 C212（固定结构的写小于 io_min），一条都还没选。取整数倍而不是取 `max` 的理由：`io_min` = 3072 这种非 2 的幂的几何上，两槽会共享一个映射单元（三方第一轮反推腿构造出来的），槽距取整数倍之后任何 ≤ 槽距的槽宽都不共享。序号到偏移的换算、根环基址与区域起点在 D22（单元原子性怎么合成） 已定项 16 第 2 句。`crates/` 写出了系统配置里那两个字段与槽距算式（`crates/singlefs-core/src/system_configuration.rs` 的 `SystemImmutableSizes::slot_spacing_for`，单测钉住「io_min = 3072 时槽距 6144」），挂载时拿实际探测的 `io_min` 与记录的槽距比对那一步还没实现（只有调用方 mkfs 参数与盘上记录的相等比对）。`io_min` 大到固定结构互相重叠的设备 mkfs 拒、不支持（用户定案不接受，原话在变更史）：S = 8 时槽距 384 KiB 收、512 KiB 起拒（根环区域 0 与区域 1 重叠），`io_min` = 1 MiB 时系统配置槽 1 与根环区域 0 同从 1 MiB 起、S 取多少都拒；依据是三方判决 `research/prompts/m3-admission-r1-main-verification.md` 第三节 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷） 那一行，欠账 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）。

**依据**：

- E129（小于 io_min 的写在真设备上怎么出事）：真设备上确实存在 `io_min` 远大于 `physical_block_size` 的块设备（`dm-thin` 块 64 KiB、`io_min` 65536 而 `physical_block_size` 512），且在那种设备上一次小写会诱发设备侧多写 ⇒ 槽距必须按探测到的 `io_min` 取整，只按 `physical_block_size` 划会让两槽落进同一个映射单元。
- 三方论证（系统配置槽宽那一轮）第一轮反推腿在 `io_min` = 3072 的几何上构造出两槽共享一个映射单元，材料与输出在 `system_configuration-slot-width-reverse-opus.md`、`system_configuration-slot-width-reverse 那一轮的腿报告（产物在 git 里）`。
- 用户定案（写代码前的总审核加次日收尾弹窗），原话在变更史。

**欠**：C82（io_min 不在挂载比对清单）、C212（固定结构的写小于 io_min）、C414（挂载时不比对 io_min 与记录的槽距）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:327-339`（整段抄，未转述）**

```markdown
#### 已定项 20：条带表与容器索引不 day-1 注册进树表

**定案**：条带表与容器索引**不**在第一版注册进树表；`w ≥ 3` 开线时取新树 ID 与新种类码。已定项 12 那句「要新登记一个保留树 ID」改读成「开线时取新号」。

**射程**：定的是注册时机，不是载体（载体在已定项 12）也不是 key（在已定项 14）。第一版 2 盘恒 `w = 2`，条带表与容器索引在盘上一条记录都不写，注册与否写不出任何字节。

**依据**：

- 用户定案（写代码前的总审核），原话在变更史。
- 无实验：第一版这两棵树在盘上是空的，注册与否写不出字节，没有可量的量。

**欠**：C103（容器索引无落点）。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:340-352`（整段抄，未转述）**

```markdown
#### 已定项 22：全条带写，永不 read-modify-write

**定案**：**条带宽度可变，每次写都是全条带写，永不 read-modify-write。** write hole 不是被修好的，是被设计成不存在。代价：空间效率略降，小写有浪费。接受。

**射程**：这一条管写路径怎么发条带，宽度怎么定在已定项 6、参与哪几块盘在已定项 8、单元怎么摆在已定项 10。它扩到设备侧就是已定项 16 与已定项 17（不诱发设备内部做 read-modify-write）。外部事实（未在本项目验证）：ZFS RAID-Z 已验证这条路可行。

**依据**：

- 用户定案（D2（RAID 条带策略） 立项那一次），原话在变更史。
- 无实验：write hole 被设计成不存在是格式层的机制推导，本工程没有可量的量；ZFS RAID-Z 是外部事实，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 不进依据。

**欠**：无。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:359-364`（整段抄，未转述）**

```markdown
#### 未定项 21：parity 几格

**题面**：一条条带里放几格校验（`parity`）。第一版 2 盘恒 `w = 2`，几何上只能是 1 格；`w ≥ 3` 开线时放几格，全仓没有条款。**第一版怎么走**：不定，按 `parity = 1` 实现（两盘镜像，`w = 2`）。**改新池新建文件的字节：否**——第一版两盘、`w` 恒 2，系统配置里 `w_max` / `g` 那两个字节由已定项 18 唯一确定；这一项改的是将来重定那两个字节时的输入，不改新池新建文件写出的值。

**射程**：要定它之前先看已定项 18 那一格的现状——`w_max` = 4 的推导有量纲错误（同量纲改正后同一等权给 6）、等权本身在 kb 里没有出处，所以 4 是知情接受的保守取值，不由那套算术钉住；parity=2 时双盘同坏不丢数据，风险轴要换成三盘同坏，同一算术给 7 ⇒ 上界随 parity 重定。**与 D9（加密） 的连锁**：parity 对密文算，多副本是同一份密文写 N 遍——加密 N 次会得到 N 份不同的字节。**备查（未采纳）**：bcachefs 在后台对整个 bucket 做纠删码，前台正常多副本写、后台攒够候选 bucket 再算一次 parity 然后原子更新所有指针，parity 只对不可变数据算一次，同样不存在 write hole，代价是 stripe 里只要还有活数据其中的 bucket 就不能复用；将来 D8（核心索引结构） 若采用 bucket 模型，D2（RAID 条带策略） 应重新权衡。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:8-38`（整段抄，未转述）**

```markdown

**D22（单元原子性怎么合成）的分项只有一套编号**，与「未定项」一节连号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **journal 是不是环形** | 定长环，权威记录在 D23（journal 的角色与格式） 已定项 2 **状态：已定。** **进度：已实现。** |
| 2 | **根环的参数** | R = 3、区域按素数步长放置、槽序跨区轮转、S 住系统配置、根槽宽 = 探测的 `physical_block_size`、系统配置槽宽 = 常量 4096、不留保留槽 **状态：已定。** **进度：已实现。** |
| 3 | **记账结构是不是 COW** | 取 COW，挂父指针下 **状态：已定。** **进度：已实现。** |
| 4 | **「本事务内释放的块不得重用」I-7.4（近 K 代块未被复用）是否存在** | 存在：整个 journal 重放窗口内释放的块不得重用 **状态：已定。** **进度：已实现。** |
| 5 | **D2（RAID 条带策略） 的口径要扩** | 已完成：两条写粒度硬要求在 D2（RAID 条带策略） 已定项 16 / 17 **状态：已定。** **进度：无对象。** |
| 7 | **根记录的字段表** | 恒走间接层：固定头 + 一条树表单元指针，树根不平铺进根槽；17 个字段合计 466 字节 **状态：已定。** **进度：未实现。** |
| 8 | **固定结构的放置** | 系统配置每盘一份、槽轮换、取号写两轮；journal 环两盘镜像；mkfs 先把每盘 [0, 根环基址) 写 0、过屏障，再把第 0 代根种进全部根环区域并把区域整段写 0 **状态：已定。** **进度：已实现。** |
| 9 | **系统配置的字段表** | 取 `layout/01-first-txn.md`「系统配置字段表」那一张：五段 46 行、489 字节，住 4096 字节的系统配置槽 **状态：已定。** **进度：已实现。** |
| 26 | **系统配置的三档可改性** | 按改了要付什么分三档：系统不可变配置 389（重建文件系统）、系统可变配置 4（重建索引不丢数据）、系统运行配置 36（无代价，每次挂载可重设并回写）；系统运行量 60 不在三档里 **状态：已定。** **进度：未实现。** |
| 10 | **区写指针能不能作恢复判定的权威上界** | 不能 **状态：已定。** **进度：无对象。** |
| 11 | **区写指针能不能进 O2（独立解析器 + checker） 的判定输入** | 不能：O2（独立解析器 + checker） 的定义域是单个镜像，区写指针不在镜像里 **状态：已定。** **进度：无对象。** |
| 12 | **系统配置要不要一条自己的树表单元指针** | 不要，树表的权威在根记录 **状态：已定。** **进度：已实现。** |
| 13 | **盘上定长字段的字节序与枚举编码约定** | 一律小端；枚举取值各有登记位；位图的位 n 住第 n div 8 个字节的第 n mod 8 低位 **状态：已定。** **进度：已实现。** |
| 14 | **devs < R 时 I-7.6（根环区域落在互不相同的盘上） 判什么** | 区域设备身份取满 min(R, devs) 个不同值、每盘不超过 ⌈R / devs⌉ 个区域；不拒绝挂载，容错按盘数降级 **状态：已定。** **进度：已实现。** |
| 15 | **系统配置字段表的四格连带：间接目录单元指针 59 还是 83、第一版写不写设备表、devs、整理三水位初值** | 不留间接目录单元指针、第一版不写设备表、devs 留 4 字节、整理三水位恒 0 **状态：已定。** **进度：已实现。** |
| 16 | **系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位** | 系统配置每盘 2 槽按世代号 mod 2 轮换（mkfs 两槽都是 1）；根环起点是 16 KiB 槽号；区域内槽位 = (txg div R) mod S；归属 0 / 1 / 0 **状态：已定。** **进度：已实现。** |
| 17 | **已定项 13 的登记位清单补全** | journal 记录类型、统计量标签、算法类型、三类 flags 与位图位序各有登记位 **状态：已定。** **进度：已实现。** |
| 18 | **分界线** | 落盘之后有带它校验和的父指针才发布的单元，COW + D4（校验和位置） 就够；没有的必须自证 **状态：已定。** **进度：无对象。** |
| 19 | **有父指针的普通单元，COW + D4（校验和位置） 足够，且不依赖任何设备原子性假设** | 撕裂出来的任意字节子集都与父里的校验和失配 **状态：已定。** **进度：无对象。** |
| 20 | **撕裂粒度变大不是让问题变严重，是让它退化成丢失写** | 全旧的撕裂父指针校验和抓得住，块内自带校验和抓不到 **状态：已定。** **进度：无对象。** |
| 21 | **原地覆写的结构必须「整单元校验和 + 被实际检查的世代号」，两样缺一不可** | 校验和答完不完整、世代号答是不是本代；不做 per-sector 盖章 **状态：已定。** **进度：已实现。** |
| 22 | **分界线不能往上挪：根环的回退候选会变假** | 有父指针的单元不改成地址稳定、原地覆写再自证 **状态：已定。** **进度：无对象。** |
| 23 | **journal 不可能被 COW 覆盖：对定长环成立，落在自证那一类** | 世代号必须在格式里，且必须被重放路径实际检查 **状态：已定。** **进度：已实现。** |
| 24 | **根落在同一类，且两类布局需要的是同一种机制** | 根自证：整单元校验和 + 代号 + 槽轮换；先逐个验证全部候选、再按代号择新，同键取读根环次序最前的 **状态：已定。** **进度：已实现。** |
| 25 | **根环深度是块重用延迟的上界，两者不是同一个数** | 根环只保「本代根写坏了还能读到上一代」；块复用的界现算、不能是格式常量 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:514-520`（整段抄，未转述）**

```markdown

| # | 分项 | 状态 |
|---|---|---|
| 6 | **zoned 的区写指针是一个不受本工程校验和管辖的见证者** | 设备维护，本工程算不出也验不了；挂到 zoned 线开线，连同「最新的根在哪」的发现程序一起定 动不动格式：动（2026-09-21，依据：乙问是「zoned 上固定结构（系统配置槽轮换、根环槽轮换）能不能原地轮转」，正文逐字「开线代价至少 +1 个结构」） **状态：未定。** **进度：无对象。** |

未定项 6 改新池新建文件的字节：否（2026-09-01，依据：第一版跑非 zoned 布局，非 zoned 上没有区写指针这个对象。⚠️ 第一版若改跑 zoned 就判不了）。

```

**出处 `.claude/kb/decisions/15-格式冻结政策.md:8-24`（整段抄，未转述）**

```markdown

D15（格式冻结政策）的分项只有一套编号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **冻结前清单第 7 项的具体处置** | 第 7 项不写决策名单，写成一句判定——全仓每条未定分项自己声明「动不动格式」，冻结请求与它相交即判红；留白宽度默认 0 **状态：已定。** **进度：无对象。** |
| 2 | **树表是不是开放列表** | 是，day-1 就做成开放列表 **状态：已定。** **进度：已实现。** |
| 3 | **`spec_hash` 的来源** | 不切换。恒对 `format-spec/<组件>.toml` 求，那份 toml 是 kb 字段表的完整投影，含行序与非标量的盘上语义规则 **状态：已定。** **进度：未实现。** |
| 4 | **feature bit 的位分配登记表** | 登记表住已定项 4；incompat 位 1 = 第一条纯 SSD 布局线（带 F 与卸载记号那一版），mkfs 起就置上，位 0 退役；compat_ro、compat 一位都没分 **状态：已定。** **进度：已实现。** |
| 5 | **冻结这个动作是什么** | 冻结 = 提交一份 `.claude/kb/format-frozen/<组件>.toml`，人只决定发不发起，机器决定这次请求成不成立 **状态：已定。** **进度：未实现。** |
| 6 | **冻结前必须完成的清单** | 七项前置，逐次冻结重判；每一项的取值从对应分项的定案行现取，不许手抄 **状态：已定。** **进度：无对象。** |
| 7 | **冻结分几层，四层之外还有什么** | 四层有严格依赖顺序（系统配置与块头、指针字段布局、key 编码、索引节点内部布局），另有三个独立冻结组件 **状态：已定。** **进度：无对象。** |
| 8 | **冻结之后失去哪些自由** | 已冻结字段的宽度、存在性、含义不能再改，只能新增；对已冻结部分 checker 的判断也成了兼容性契约 **状态：已定。** **进度：无对象。** |
| 9 | **冻结后的改动判定表** | 九类改动各自判 compat / compat_ro / incompat 与 bump 与否，判据是旧读者会怎样 **状态：已定。** **进度：无对象。** |
| 10 | **feature bit 的位数、记账与分配权** | 三个 bitmap 各 256 位存系统配置；分配权是「谁先合并谁先占」，位号严格递增；位一旦用过不许回收 **状态：已定。** **进度：已实现。** |
| 11 | **冻结要配的两条会红的检查** | 格式冻结契约（`spec_hash` 比对）与向后兼容回归（黄金镜像），镜像本体不进 git、清单只增不删 **状态：已定。** **进度：未实现。** |

```

**出处 `.claude/kb/decisions/16-发布语义.md:8-28`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **defer 窗口取多大** | 按盘回退下界的收严形态，按状态数：候选集留最近 4 个可退状态（去重、以根环装得下为界），可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；F 记进系统配置 **状态：已定。** **进度：未实现。** |
| 2 | **checkpoint 的触发判据** | 时间与脏数据量两者取先到：`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty)` **状态：已定。** **进度：已实现。** |
| 3 | **journal 在 D15（格式冻结政策） 的冻结分层里记第几层** | 不进四层，立成四层之外的第三个独立冻结组件；新记录格式走记录类型 + 系统配置 log-incompat 位 **状态：已定。** **进度：未实现。** |
| 4 | **记录核对器那第二份 replay 实现** | 一次发布整体施加或整体不施加；第二份实现停在句法层（前缀判定 + 择根 + 续做），在合谋测试里是第四个判决器 **状态：已定。** **进度：已实现。** |
| 5 | **`T_time` 与 `T_dirty` 取多少，丢失窗口的口径** | `T_time` = 5 s、越界只警告；`T_dirty` = 2 GiB、夹成 min(字段, 环长/F)；崩溃最多丢 T_time+N_switch×(T_retry+T_redo) **状态：已定。** **进度：已实现。** |
| 6 | **发布计数器** | 每次发布 checkpoint_txg 加一，fsync 触发的也是发布；根槽轮转键、记账的代、重放水位是同一个计数 **状态：已定。** **进度：未实现。** |
| 7 | **发布的持久顺序** | 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；重放施加前必须逐项验证点名单元 **状态：已定。** **进度：已实现。** |
| 8 | **新实例要不要先让本实例的根落到两块盘上** | 取戊：根覆盖两块盘之前不让 fsync 返回，连推空发布至多 R = 3 次 **状态：已定。** **进度：已实现。** |
| 9 | **一次空发布写不写单元** | 记账树存在时就写：空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2 重写记账行与连带的四样 **状态：已定。** **进度：已实现。** |
| 10 | **发布粒度** | 攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次；发布频率只由 D16（发布语义） 定 **状态：已定。** **进度：已实现。** |
| 11 | **能不能绕过 journal 直接发根** | 不能：所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏，fsync 只是提前触发一次发布 **状态：已定。** **进度：已实现。** |
| 12 | **中间状态的合法性语义** | D8（核心索引结构）「中间状态必须每一个都是合法的文件系统状态」改写成「根 + journal 任意前缀，replay 之后合法」 **状态：已定。** **进度：已实现。** |
| 13 | **这份 journal replay 归谁** | 归记录核对器，不归 O2（独立解析器 + checker）；它是第二次实现 replay，是 D16（发布语义） 最贵的一笔 **状态：已定。** **进度：已实现。** |
| 14 | **快照发布点的位置** | 与 checkpoint 边界对齐，钉在这个 checkpoint 全部数据变更之后；同窗两个快照用 `(txg, 快照 id)` 打破并列 **状态：已定。** **进度：未实现。** |
| 15 | **checkpoint C 中产生的释放什么时候进可分配集合** | C 被发布之前不得进入；两条理由，第二条按 journal 位置说，比 checkpoint 边界更严 **状态：已定。** **进度：已实现。** |
| 16 | **checkpoint 与消息缓冲的摊销窗口分界** | 两者买同一样东西，差别只在窗口住内存还是住盘；E7（离线索引 harness） 的对照组钉成「有 checkpoint、无消息缓冲」 **状态：已定。** **进度：无对象。** |
| 17 | **fsync 报错之后调用方能假设什么** | 报错之后这次发布可能已成也可能未成，不许假设可复用旧内存态重试；成没成由调用方自己读现行根判定 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:8-34`（整段抄，未转述）**

```markdown

**D2（RAID 条带策略）的分项只有一套编号**，与「未定项」一节连号，不重排。

| # | 分项 | 定案 |
|---|---|---|
| 1 | **「宽度可变」的粒度** | 逐次可变：格式侧物理位置逐个列出、各带设备身份，不引 chunk 映射表；分配侧不采纳组内固定 **状态：已定。** **进度：已实现。** |
| 2 | **各盘要不要等大** | 不必等大，只在寻址正确性这个子问题上；池成员与各盘几何挂载时校验，判不出安全组合就拒绝挂载 **状态：已定。** **进度：已实现。** |
| 3 | **设备集合变化之后几何怎么再求值** | 走显式「几何再求值」事务并原子发布，卷保持挂载，不许强制重挂 **状态：已定。** **进度：未实现。** |
| 4 | **几何再求值事务的形态** | 几何不合格拒绝加盘、排空后再换几何、加盘后跑一次后台可续做的重平衡，搬迁单位是整个对象 **状态：已定。** **进度：未实现。** |
| 5 | **根环失败域在设备集合变化之后失效怎么办** | 根环不重放置；重平衡搬用户数据、不动根环，掉盘掉到 devs < R 时接受退化 **状态：已定。** **进度：未实现。** |
| 6 | **分配器每次写用几列** | `w = clamp(攒批后写入量 + 1, 2, 4)` 再夹当时可写设备数，按格宽分桶；上界 4 是系统配置声明的常量 **状态：已定。** **进度：已实现。** |
| 7 | **根环归属公式里的 devs 取哪一个** | 归属不再算，mkfs 逐区域显式存设备身份；第一版三个区域写死 0 / 1 / 0，不是 mkfs 参数 **状态：已定。** **进度：已实现。** |
| 8 | **参与哪些设备（子集怎么选）** | 建对象时按最空的 g 块选一组，此后只在组内取最空的 w 块；组内有盘满了就重选，组不落盘 **状态：已定。** **进度：未实现。** |
| 9 | **第一版跑几块盘** | 第一个可运行目标跑 2 块盘；单盘走单独一条线，那条线开线时再定 **状态：已定。** **进度：已实现。** |
| 10 | **一个单元横跨 w 列还是整个落一列** | 整个落在一列上，不切开、不横跨；位置条目每个副本一条 **状态：已定。** **进度：已实现。** |
| 11 | **条带的生命周期（部分释放怎么办）** | 部分条带钉住、后台惰性重落；修复前成员格不得复用，w = 2 的条带成员一死即整条回收 **状态：已定。** **进度：未实现。** |
| 12 | **parity 列的位置与发现机制** | 成员表定长 56 字节住码 3 打包记录，由容器索引指认，容器整条链恒走 `w = 2` 镜像 **状态：已定。** **进度：未实现。** |
| 13 | **降级期间允许写还是只读** | 只读：可写挂载要独占打开过半设备，运行期掉到 w 下限以下或过半以下即转只读 **状态：已定。** **进度：未实现。** |
| 14 | **条带表这棵逻辑树的 key 取什么** | 取落点（设备身份 4 + 16 KiB 槽号 6），与 D3（空间分配） 已定项 3 的分配记录树同一个坐标系 **状态：已定。** **进度：未实现。** |
| 15 | **「生命周期不同的对象」的通用判据** | 死亡触发事件恒为同一次提交就算同生命周期、允许共享；已定项 17 的射程收窄到 io_min 内的固定结构 **状态：已定。** **进度：无对象。** |
| 16 | **不发出小于 `io_min` 的写** | 向一个设备物理映射单元里发更小的写会诱发设备内 RMW；这个量是 `io_min`，不是 `physical_block_size` **状态：已定。** **进度：已实现。** |
| 17 | **不让两个生命周期不同的对象共享同一个物理映射单元** | 判据由已定项 15 给；射程只罩 io_min 内的固定结构，16 KiB 节点与 32 KiB 单元不在里面 **状态：已定。** **进度：已实现。** |
| 18 | **系统配置里 w_max 与 g 第一版写 4 还是 2** | 两个字节都写 4，g 加运行时夹取；第一版加盘准入拒绝让 `w ≥ 3` 可达的设备集合变化，devs 上限 2 **状态：已定。** **进度：已实现。** |
| 19 | **根槽与系统配置槽的槽距** | 槽距 = 4096 向上取整到 mkfs 探测的 io_min 的整数倍；io_min 进挂载比对清单，第一版只设两条闸里的 (a) **状态：已定。** **进度：已实现。** |
| 20 | **条带表与容器索引要不要 day-1 注册进树表** | 不注册；`w ≥ 3` 开线时取新树 ID 与新种类码 **状态：已定。** **进度：已实现。** |
| 22 | **全条带写，永不 read-modify-write** | 条带宽度可变，每次写都是全条带写；write hole 不被修复，被设计成不存在 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:354-358`（整段抄，未转述）**

```markdown

| # | 分项 | 状态 |
|---|---|---|
| 21 | **parity 几格** | 一条条带里放几格校验没有条款；第一版 2 盘恒 `w = 2` ⇒ 只能是 1 格，但那是几何逼出来的。改新池新建文件的字节：否（2026-09-20，依据：第一版两盘镜像按 `parity = 1` 实现，没有第二种取值写得出不同的字节；要定的是 `w ≥ 3` 开线时放几格，那时才有分支） 动不动格式：动（2026-09-21，依据：射程逐字「上界随 parity 重定」——系统配置里 `w_max` 那个字节的取值随本项的答案变，parity=2 时同一等权算术给 7，而已定项 18 现取 4） **状态：未定。** **进度：未实现。** |

```
