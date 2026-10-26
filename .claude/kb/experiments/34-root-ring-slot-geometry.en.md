## E34 Root Ring Slot Geometry — Arithmetic + Real Device Halves Run (2026-09-01 / 2026-09-02): **Assertion 1 exposure holds, but fault unit on real array is smaller; Assertion 2 blocked by settled cross-region placement**

[中文](34-根环槽几何.md) · **English** · [日本語](34-ルートリングスロットジオメトリ.ja.md)

**What it asks**: The **relationship between root ring slot width and device physical mapping unit (`io_min`)**—
when slot width is taken as `physical_block_size`, whether multiple slots pack into a single `io_min` unit.

⚠️ **Division of labor with E41 (Root Ring Slot Geometry 2×4×256)**: E41 (Root Ring Slot Geometry 2×4×256) asks whether a specific parameter combination suffices under crash and block reuse constraints;
E34 (Root Ring Slot Geometry) asks according to which quantity slot width should be chosen, a question of format-level design.

### Verified Premises (Verbatim, Sources Verifiable Individually)

| Source | Verbatim |
|---|---|
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Settled | Atomic width of self-authenticating units "**equals runtime probed `physical_block_size`, hardcoding forbidden**" |
| D2 (RAID Stripe Policy) Settled Item 16 | "**Do not issue writes smaller than device physical mapping unit**. On Linux this quantity is `io_min`, not `physical_block_size`" |
| D2 (RAID Stripe Policy) Settled Item 17 | "**Do not allow two objects with differing lifecycles to share the same physical mapping unit**" |
| Linux Stable ABI (`fs-refs/linux-6.17/Documentation/ABI/stable/sysfs-block` audited 2026-08-29) | `minimum_io_size` "For RAID arrays it is often the **stripe chunk size**"; `physical_block_size` is "the smallest unit a physical storage device can write **atomically**" |
| D22 (Synthesizing Unit Atomicity) Settled Item 2 (2026-08-31) | "**R = 3 regions; regions placed with prime stride** (region r lands at `r × P × chunk`, P prime and `P > devs`), **slot sequence rotates across regions**; **slot width = probed `physical_block_size` at mount**; no reserved slots" |
| E53 (Can Root Ring Mount After Losing an Entire Disk) Real Device Test | Wiping disks one by one across 4 disks, **consistently mounts, worst-case rollback 1 generation** |

**Local Machine Probing (2026-08-29)**: `/sys/block/nvme0n1/queue/physical_block_size = 512`,
and `io_min == physical_block_size == 512`.

### ⚠️ Original Premise Expired: Tested Candidate Geometry Replaced Entirely Two Days After Document Creation

When E34 (Root Ring Slot Geometry) was created, it asked about candidate form "2 regions × 4 slots × 256 bytes = 2048 bytes".
**D22 (Synthesizing Unit Atomicity) Settled Item 2 replaced it completely on 2026-08-31** (verbatim: "candidate form 2 regions × 4 slots × 256 bytes replaced entirely")—
slot width 256 was rejected by E41 (Root Ring Slot Geometry 2×4×256), and R=4 was rejected by E47 (Root Ring Failure Domain Losses).

⇒ **Update premise, preserve criteria.** Criteria of E34 (Root Ring Slot Geometry) were originally written **parameterized** by
`(physical_block_size, io_min, slot_width, region_stride)`, with zero words bound to 256
⇒ run on **settled** geometry as-is, leaving an extra cell running the replaced candidate as control.

### Criteria (Hardcoded Before Running, Unmodified After)

- **What is measured**: Given `(physical_block_size, io_min, slot_width, region_stride)`,
  measure "slots destroyed by a single write" and "generation of latest root still readable after power loss".
- **Criterion**: If any parameter combination causes **a single write to destroy ≥ 2 slots** ⇒ Assertion 1 holds;
  if any combination causes **a single write to destroy all slots** ⇒ Assertion 2 holds.
- ⚠️ **Must carry absolute assertions**: "Slots destroyed by a single write" directly calculated from `(slot_width, unit_width, start_offset)`,
  not read back from tested code.

### Failure Clauses (Hardcoded Before Running)

| Clause | Judgment |
|---|---|
| **Positive control**: Slot width set ≥ `io_min` and each slot exclusively occupies one `io_min` unit, both metrics must drop to 1 | **Passed** (512 / 4096 / 65536 all three tiers `both_are_one=1`) |
| If across all realistic parameter combinations a single write destroys only 1 slot, assertion is invalid, **record truthfully** | **Not triggered** (18 of 96 cells destroy ≥ 2) |

### Measured Results

#### Assertion 1: **Holds**, and Holds Only for Settled Design Choice

| Slot Width Choice | Cells Destroying ≥ 2 Slots in a Single Write |
|---|---|
| **D22 (Synthesizing Unit Atomicity) Settled Item 2: Slot width = `physical_block_size`** | **18 / 48** |
| E34 (Root Ring Slot Geometry) Proposal: Slot width = `max(physical_block_size, io_min)` | **0 / 48** |

On arrays (`io_min` = 64 KiB, `physical_block_size` = 512), **a single `io_min` unit covers an entire region**:

| Slots per Region S | 1 | 4 | 8 | 16 |
|---|---|---|---|---|
| Slots destroyed by a single write | 1 | 4 | 8 | **16 (Entire region)** |

⇒ **Unmeasurable on local machine** (`io_min == physical_block_size == 512` ⇒ constant 1),
which was the cell blocking E34 (Root Ring Slot Geometry); arithmetically it holds deterministically on array geometry.

#### Assertion 2: **Does Not Hold**—Blocked by Settled Cross-Region Prime Stride Placement

| Placement Rule | Cells Destroying All Slots in a Single Write |
|---|---|
| **D22 (Synthesizing Unit Atomicity) Settled Item 2: Prime stride (Region stride = `P × chunk`)** | **0 / 48** |
| Replaced candidate: Contiguous across entire ring | **7 / 48** |
| Candidate itself (2 regions × 4 slots × 256 B = 2048 B, `io_min` = 64 KiB) | **Destroys all 4 slots** |

⇒ **Assertion 2 holds only for the replaced contiguous form.**
Under prime stride, distance between adjacent regions is ≥ `3 × chunk`, so a single `io_min` unit cannot cover two regions
⇒ **The placement rule of D22 (Synthesizing Unit Atomicity) Settled Item 2 already blocks Assertion 2**,
even though originally settled for "mounting after losing a disk", not for this.

#### Consequences of Assertion 1 Holding: Restricted to 1-Generation Rollback by Cross-Region Rotation

Slot sequence rotates across regions (region = `txg mod R`) ⇒ wiping an entire region wipes only the batch where `txg ≡ r (mod 3)`,
so **latest readable root rolls back at worst 1 generation** (wiping three regions individually yields 1 in all three cells).
⇒ Cell-by-cell identical to "worst-case rollback 1 generation" in E53 (Can Root Ring Mount After Losing an Entire Disk) real device measurements.

### Real Device Measurements (2026-09-02, md/raid5 Real Array, N=5 Rounds × 2 Invocations = 10 Rounds Total, Judgments Cell-by-cell Identical)

The arithmetic half counts only "how many slots land in the same `io_min` unit", **unable to answer "how many a single fault actually destroys"**.
This section measures "how many a single fault actually destroys" on a **real parity array**.

#### Geometric Premise (Artifact `name=v1_geometry`): `io_min` = 64 KiB Measured on Repository Real Device

| Metric | Local nvme0n1 | md/raid5 (4 Legs, Chunk 64 KiB) |
|---|---|---|
| `minimum_io_size` | 512 | **65536** |
| `physical_block_size` | 512 | **512** |
| `optimal_io_size` | — | 196608 |

⇒ Local machine readings substantiate Linux Stable ABI's "For RAID arrays it is often the **stripe chunk size**":
**`io_min` and `physical_block_size` differ by 128× on this array, while D22 (Synthesizing Unit Atomicity) Settled Item 2 tracks the latter.**

#### Two Controls (Artifacts `scen=v2_writehole_hits` / `scen=v3_no_tear_control`): Mechanism Reproduced, Not Broken by Harness

| Control | Construction | Judgment (10 Rounds) |
|---|---|---|
| **Positive control** | Inject tear into slot A (data leg modified, parity unchanged), then degrade victim leg | **Victim position judged corrupted 10/10** |
| **Negative control** | Do **not** inject tear, degrade that leg similarly | **Victim position intact 10/10** |

⇒ Corruption caught by positive control is caused by tear, not harness. **Without negative control, the two cannot be distinguished.**

#### Both Isolations (Artifacts `scen=v4_region*_survives` / `scen=v5_intra_slot_survives`) Hold

| Scenario | Construction | Judgment (10 Rounds) |
|---|---|---|
| **Cross-region isolation** | Tear injected into region 0, degrade leg of region 1 / region 2 | Both regions **intact 10/10** |
| **Intra-region neighboring slot isolation** | Tear injected into slot A (sector offset 0), degrade same leg | Neighboring slot B in same chunk (sector offset 1) **intact 10/10** |

#### ⚠️ Intra-region Neighboring Slot Cell Narrows Arithmetic Conclusion

Arithmetic exposure was "one `io_min` unit covers entire region ⇒ 16 slots".
**On a real array, those 16 slots do not fail together**: parity in md/raid5 is computed row-by-row by **sector rows**,
so a tear only pollutes the row it resides on ⇒ **the fault unit is "same stripe row × same sector position" across legs, not the entire `io_min` unit.**

⇒ **Exposure ≠ Loss.** The 16 in arithmetic is an **upper bound**; on real devices this class of fault cannot realize it.
**This directly eases the rationale for changing slot width** (see tension in D22 (Synthesizing Unit Atomicity) Settled Item 2).

#### Measurement Basis: Demonstrates RAID5 Write Hole, Not SSD Internal RMW

- Tear bypasses md to modify data leg sectors, producing **on-disk state "data landed, parity did not"**,
  **not actual power loss**—loop devices cannot simulate power cuts. **States equivalent, paths not equivalent.**
- Hence it proves the **RAID5 write hole** mechanism. The other mechanism of Assertion 1—
  **SSD internal RMW over entire mapping unit**—**cannot be injected on loop devices, remaining owed**.
  Fault units differ (RAID5 write hole by sector row, SSD internal RMW by entire mapping unit); **they cannot substitute for each other**.

#### Two Harness Flaws Caught by Controls, Not Conjectured

1. **First version injected without stopping array**, so md stripe cache held pre-tear data ⇒ **positive control failed to judge corruption**.
   Per failure clauses, entire round should be voided. Fixed to "stop array → inject → degrade and reassemble with one fewer leg → drop cache and re-read".
2. **First version failed wrong leg in cross-region cell**: parity block of single marker row on all-zero disk is **verbatim copy** of marker,
   so `grep` mistook parity leg for data leg. Added companion marker on same row and position to make parity XOR garbage,
   requiring `leg_of` to **hit exactly one leg**, aborting round if more.
3. **Intra-region neighboring slot aborted 5 rounds consecutively**: data marker `B_1_9` was **substring** of companion marker `CB_1_9`,
   causing `grep` to misjudge two legs. Added delimiters on both sides of each marker and strictly used `grep -F`.
   ⚠️ All three were "controls/gates blocking false conclusions", **not post-hoc rationalizations**.

### ⚠️ Remaining Decision Problem: E34 (Root Ring Slot Geometry) Does Not Decide It

**Assertion 1 holds ⇒ Settled slot width allows one `io_min` unit to cover entire region on arrays**,
while D2 (RAID Stripe Policy) Settled Item 17 verbatim states "do not allow two objects with differing lifecycles to share the same physical mapping unit".
**Whether different slots in the same region count as "objects with differing lifecycles" has only one generational instantiation in repository, lacking a general criterion**
(D18 (Information Carried in a Block) Settled Item 10 tombstones grouped by generation + E84 (Pinned Volume of Packed Tombstones) verbatim
"**the lifecycle of a tombstone record is the deadlist (snapshot generation) it belongs to**")—2026-09-08 three-way argument rewritten per audit,
criterion gap assigned to D2 (RAID Stripe Policy) Settled Item 15. ⚠️ Settled on 2026-09-13: criterion "death trigger event constantly the same commit ⇒ same lifecycle"; different slots in same region count under it, root slots falling within narrowed scope of Settled Item 17.
⚠️ **Same round struck a harder requirement**: What decides this is D2 (RAID Stripe Policy) Settled Item 16
("**Do not issue writes smaller than `io_min`**, without an undefined word)—when root slot width is `physical_block_size`,
root slot writes on devices with larger `io_min` are themselves non-compliant. E34 (Root Ring Slot Geometry) premise table copied both hard requirements side-by-side,
yet entire argument cited only Requirement 2. Assigned to [checks-owed.md](../checks-owed.md) C212 (fixed-structure writes smaller than io_min).

Whether to change D22 (Synthesizing Unit Atomicity) Settled Item 2 slot width from `physical_block_size` to `max(physical_block_size, io_min)`
is an **inference**, requiring a three-way argument per `.claude/rules/three-way-inference.md`
⇒ **two cloud subagents remain owed; E34 (Root Ring Slot Geometry) provides arithmetic, not ruling.**
⚠️ Cost side was also unmeasured: raising slot width from 512 to 64 KiB is **128×** root ring footprint,
while D22 (Synthesizing Unit Atomicity) Settled Item 2 verbatim reasoned "exceeding it buys no additional safety, only wasting bytes"—
**that sentence was written under premise lacking the `io_min` dimension.**

### What It Cannot Answer

1. **Pure geometric arithmetic, touches no device**, 0 file operations. Computes "how many slots land in same `io_min` unit",
   **not** "whether power cut truly destroys that unit via internal RMW"—
   "single write destroys N slots" here means **exposure N, not observed loss**.
2. **Real device half ran (2026-09-02)**, but covered only one mechanism:
   **RAID5 write hole** measured on real md/raid5 (with positive/negative controls);
   **SSD internal RMW over entire mapping unit** cannot be injected on loop devices, **remaining owed**.
   ⚠️ Tear constructs on-disk state "data landed, parity did not", **not real power loss**—
   states equivalent, paths not equivalent. True power loss requires power cut hardware or QEMU-level write sequence truncation.
3. **Two cloud subagents remain owed** (`.claude/rules/three-way-inference.md` requires three).
   ⚠️ Existing local subagent artifact `e34-rootring-local.out` was queried **against replaced candidate geometry**
   ⇒ per "background materials fed to debate must be pre-audited", **it cannot serve as vote for 2026-09-01**; rerun requires updated materials.
4. **Counterexample raised by local subagent judged invalid**: argued "devices with `io_min` exactly 256 bytes do not violate",
   whereas lower bound of `io_min` is logical block size, and local logical block is 512 ⇒ such devices unobtainable on Linux.
5. **Local subagent claim judged erroneous**: argued D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) setting self-authenticating unit width to
   `physical_block_size` was wrong and should use `io_min`. The two govern different matters (premise table quoted both ABI definitions verbatim);
   D2 (RAID Stripe Policy) explicitly warned "do not merge the two into one sentence again", and local subagent did just that
   ⇒ **D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) unchanged**; what E34 (Root Ring Slot Geometry) proposes is a **write-side** requirement, not a replacement.

### Prerequisites

Arithmetic half has no prerequisites. Real device half requires `sudo` (password via `.env`, ready 2026-09-02).
The two cloud legs of the three-way argument remain owed; true power loss (rather than constructing disk state) remains owed.

### Measurement Basis and Rerunning

- Code `scripts/experiment-apparatus/e34_ring_iomin.rs`
  (`cargo run --release --bin e34-ring-iomin`),
  raw output `e34-ring-iomin-2026-09-01.out` (89 lines, final line `emitted=89`).
- Sweep space: `physical_block_size` ∈ {512, 4096} × `io_min` ∈ {512, 4096, 65536}
  × slots per region S ∈ {1, 4, 8, 16} × 2 slot width choices × 2 placements, total 96 cells (combinations with `io_min < pbs` skipped).
- **N=5 rounds byte-for-byte identical** (identical md5). **8 unit tests, all 7 mutations caught, 0 blind spots**,
  mutation table `scripts/mutation/tables/e34_ring_iomin.tsv`
  (`bash scripts/mutation/mutate.sh e34-ring-iomin scripts/experiment-apparatus/e34_ring_iomin.rs scripts/mutation/tables/e34_ring_iomin.tsv`).
- **Real device half**: Script `scripts/devices/e34-iomin-probe.sh`
  (`bash scripts/devices/e34-iomin-probe.sh`, requires `sudo`, password via `.env`),
  raw output `e34-iomin-probe-2026-09-02.out` (28 lines, final line `emitted=28`).
  4 loop devices of 64 MiB forming md/raid5, chunk 64 KiB, `--assume-clean`; image placed in `TMPDIR`,
  EXIT cleans up via literal names `mdadm --stop` / `losetup -d`.
  ⚠️ **Rerunning is not byte-by-byte comparison**: marker string contains round number and pid ⇒ outputs differ each run;
  **criterion is verdict pattern across 5 scenarios** (positive control all corrupt, other three all intact), consistent across two full invocations (10 rounds total).
- Local subagent Q&A (**materials expired, see Item 3 of "What It Cannot Answer"**):
  `bash scripts/agents/ask-local.sh e34-rootring-geometry-local.md` (script deleted 2026-10-11, the method for retrieving it is in the “Local LLM (not assigned a leg in three-way argument)” section of `.claude/kb/tooling.md`).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) Settled Item 2 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: wording only; conclusions, numbers, and artifacts unchanged |
| D2 (RAID Stripe Policy) | Material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: wording only; conclusions, numbers, and artifacts unchanged |
| D18 (Information Carried in a Block) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: wording only; conclusions, numbers, and artifacts unchanged |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-17 unaffected: wording, artifact pointers, paths, and numbers synchronized; conclusions, numbers, and artifacts unchanged; previously 2026-10-11 unaffected: wording only; conclusions, numbers, and artifacts unchanged |

## Historical Versions

Historical entries for E34 (Root Ring Slot Geometry) are gathered in [experiments-history.md](../experiments-history.md).
