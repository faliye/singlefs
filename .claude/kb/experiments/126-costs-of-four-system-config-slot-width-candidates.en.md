## E126 Costs of Four System Configuration Slot Width Candidates —— Run (2026-09-09, pure arithmetic, 9 unit tests / 10 mutations caught)

[中文](126-系统配置槽宽四条候选的代价.md) · **English** · [日本語](126-システム構成スロット幅4候補のコスト.ja.md)

**What it answers is the cell blocked by C232 (system configuration slot width: class rule gave answer, field table says unset)**: which candidate to choose for system configuration slot width.
E126 (Costs of Four System Configuration Slot Width Candidates) **only gives costs, not judgments** —— judgments must undergo tripartite review (`.claude/rules/three-way-inference.md`).

### Why This Cell Was Empty (Confirmed on the Spot Before Run)

| Fact | Verbatim Provenance |
|---|---|
| Field table of new-pool file creation marks it as unset, pointing back to itself | In system configuration slot width row of [decisions/18-块里携带什么信息.md](../decisions/18-块里携带什么信息.md): `\| 18 \| system configuration slot \| slot width \| D22 (Synthesizing Unit Atomicity) settled item 9 \| own \| placeholder, unset \| not applicable \| without \| as above \|` |
| Self-proving unit list of D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) excludes system configuration slot | Line 69: "Self-proving units (root slot, journal record header) \| equals `physical_block_size` probed at runtime, hardcoding forbidden." |
| E115 (Completeness and Byte Budget of System Configuration Field Table) itself marks those two tiers as analogies | File header comment in `scripts/experiment-apparatus/e115_system_configuration_completeness.rs`: "⚠️ Its self-proving unit list verbatim excludes system configuration slot —— E100 (Three-Segment Geometry of System Configuration) applying these two tiers to system configuration was an analogical extension, not clause coverage." |
| The only place recording it was a paraphrase in an owed check | C212 (fixed-structure writes smaller than io_min): "system configuration slot width follows root slot", while C212 (fixed-structure writes smaller than io_min) in the same cell verbatim states "three candidates mutually differ, pre-supposing which path to take is forbidden" |
| D22 (Synthesizing Unit Atomicity) settled item 8 only settled replica count and slot count lower bound | D22 (Synthesizing Unit Atomicity) settled item 8: "system configuration places one copy per disk... structured as slot rotation, at least 2 slots per disk." Not a single word on how wide. |

### Four Candidates (Hardcoded Before Run, Never Modified After)

| Code | What Slot Width Takes | What It Aims to Solve |
|---|---|---|
| **A** | `= physical_block_size` (probed value, follows root slot) | Implicit assumption of current status quo; shares shape with D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) self-proving units |
| **B** | `= max(physical_block_size, io_min)` (probed value) | Hard requirement 1 of D2 (RAID Stripe Policy) "do not issue writes smaller than `io_min`", i.e., Candidate ① of C212 (fixed-structure writes smaller than io_min) applied to system configuration |
| **C** | `= a format constant` (sweeping 512 / 4096 tiers) | Decoupled from runtime probing, no longer "a set" on heterogeneous pools |
| **D** | `= minimum unit size in pool` (16384 in v1, D18 (Information Carried in a Block) settled item 9) | Makes system configuration slot **a unit itself** ⇒ automatically acquires those five mechanisms per `.claude/rules/fs-design.md` "On-disk space not being a unit receives zero core-layer coverage" |

### Criteria (Hardcoded Before Run, Never Modified After)

1. **Occupancy (Absolute Values)**: Per disk `slot_count × slot_width`, and pool occupancy for initial 2 disks (D2 (RAID Stripe Policy) settled item 9). Slot count sweeps `{2, 4, 8, 16}` (settled item 8 only gives lower bound 2). **Report in bytes, not multiples** —— `.claude/rules/fs-design.md` verbatim states "'Who is more frugal' constitutes zero criteria"; occupancy is used solely to refute "this option is extravagant".
2. **Whether It Fits**: System configuration record bytes taken from four cells of E115 (Completeness and Byte Budget of System Configuration Field Table) / E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) (507 / 566 / 536 / 595) and two cells of synthesis arm (389 / 477), evaluating `fits` under each candidate cell by cell. ⚠️ **These six numbers carry a known error** (C231 (root record fields mixed into system configuration required items): provenance of tree table unit pointer 59 bytes cited settled item for root record) ⇒ **simultaneously report a "minus 59" set**, presenting both sets together, **reporting only one set is forbidden**.
3. **D2 (RAID Stripe Policy) Hard Requirement 1 Compliance**: Device `io_min` sweeps `{512, 4096, 65536}` (512 probed value on local `nvme0n1`; 65536 measured on real md/raid5 by E34 (Root Ring Slot Geometry)), reporting cell by cell whether "a single slot write is ≥ `io_min`". **Report in absolute values, not ratios.**
4. **Tear Exposure Surface**: `slot_width ÷ 512` = spans how many 512 sectors, and reporting `2ⁿ − 2` "readable but inconsistent" states per E115 (Completeness and Byte Budget of System Configuration Field Table) measurement basis. ⚠️ **Measurement basis must carry one sentence**: D22 (Synthesizing Unit Atomicity) settled items 19 and 20 state tears are caught by parent pointer checksums, whereas system configuration has no parent and relies on D22 (Synthesizing Unit Atomicity) settled item 21 self-proving ⇒ this number is **exposure surface, not loss surface** (identical to E34 (Root Ring Slot Geometry) "exposure surface ≠ loss surface").
5. **Whether It Is Single-Valued on Heterogeneous Pools**: A / B depend on probing ⇒ may differ across disks within a pool; C / D are format constants ⇒ constant single value. Report "how many distinct values each of the four candidates takes on a heterogeneous pool mixing `pbs` 512 and 4096". **Absolute values.**

### Failure Clauses (Hardcoded Before Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: In criterion 2, slot width 512 tier must reproduce judgments committed in E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) (507 fits, 566 / 536 / 595 do not fit) | Cannot reproduce ⇒ **apparatus is flawed, entire round invalidated** (cannot distinguish "I computed incorrectly" from "E124 is wrong") |
| **Negative control**: In slot width 16384 tier, all six record byte counts must **all** `fits` | Any non-fitting occurs ⇒ entire round invalidated (indicating record byte measurement basis was misread) |
| **Reverse acceptance clause**: If a candidate holds no advantage across all criteria | Record faithfully that "none of the four holds overall dominance", **picking one as winner is forbidden** —— E126 (Costs of Four System Configuration Slot Width Candidates) only provides costs |
| **No judgments permitted**: | E126 (Costs of Four System Configuration Slot Width Candidates) stops after reporting numbers. "Which to choose" is a deduction requiring three-way argument; **hardcoding this before run prevents treating cost tables directly as settlements** |
| C231 (root record fields mixed into system configuration required items) unpatched | In the two sets of criterion 2, the "minus 59" set is **an estimate not a settlement**, citation in text must carry this qualifier |

### What It Cannot Answer

**Does not answer "what fields system configuration should have"** —— that is D22 (Synthesizing Unit Atomicity) settled item 9; E126 (Costs of Four System Configuration Slot Width Candidates) sweeps record bytes as **input**.
**Does not answer whole C212 (fixed-structure writes smaller than io_min) conflict** —— C212 also governs root slot and journal records; E126 only examines system configuration slot class.
**Does not answer real device behavior** —— `io_min` three tiers in criterion 3 are swept parameters, not live probe; 65536 tier originates from real array in E34 (Root Ring Slot Geometry), citation carries [tooling.md](../tooling.md) clause "the only real block device on this machine is `nvme0n1`... all related numbers are simulated".

### Actual Run Results (2026-09-09)

**Criterion 3 (D2 (RAID Stripe Policy) hard requirement 1 compliance) is the most cleanly separated cell**, artifact row by row:

| Device | A `pbs` | B `max(pbs,io_min)` | C 512 | C 4096 | D 16384 |
|---|---|---|---|---|---|
| `nvme_local` (pbs 512 / io_min 512, local probe) | 1 | 1 | 1 | 1 | 1 |
| `common_4k` (io_min 4096) | 1 | 1 | **0** | 1 | 1 |
| `md_raid5` (pbs 512 / io_min 65536, E34 (Root Ring Slot Geometry) real array) | **0** | 1 | **0** | **0** | **0** |

⇒ **Only B satisfies `honors=1` across all three tiers.**

**Criterion 1 (Occupancy, absolute bytes)**: Most expensive cell is B on real array with 16 slots `pool_bytes=2097152` (2 MiB);
D most expensive `pool_bytes=524288`; A cheapest `pool_bytes=2048`.
⚠️ Per `.claude/rules/fs-design.md` "'Who is more frugal' constitutes zero criteria", this cell **serves solely to refute 'extravagance'** ——
the most expensive among the four is merely 2 MiB, **none expensive enough to constitute an objection**.

**Criterion 5 (Distinct values taken on heterogeneous pool)**: A, B `distinct_widths=2`; C, D `distinct_widths=1`
⇒ **A and B are not single-valued on heterogeneous pools**, colliding with C47 (Root Slots Not Aligned to the Judged Width) clause "judgment width in heterogeneous pools is a set rather than a single number".

**Criterion 2 (Whether it fits)**: On 512 slot only 507 fits among six cells (minus-59 set 448 / 477 / 507 fit, 536 overflows);
**all six cells fit on 4096 and 16384 slots** ⇒ once slot width is ≥ 4096, byte budget pressure of D22 (Synthesizing Unit Atomicity) settled item 9 vanishes entirely.
⚠️ Those six numbers carry known error C231 (root record fields mixed into system configuration required items); reported together, **the 'minus 59' set is an estimate, not a settlement**.

**Criterion 4 (Tear exposure surface)**: A 512 `sectors=1 tear_states=0`; C 4096 `sectors=8 tear_states=254`;
D `sectors=32`; B on real array `sectors=128` (`2^128-2`, exceeds u64, reported in exponential format).
⚠️ E34 (Root Ring Slot Geometry) verbatim states "**exposure surface ≠ loss surface**" —— system configuration has no parent, relying on self-proving checksums in D22 (Synthesizing Unit Atomicity) settled item 21, with tears caught by whole-slot checksums.

**Pre-run reverse acceptance clause triggered**: **None of the four holds overall dominance across all criteria**, recorded faithfully without picking a winner.

### ⚠️ Three-way Argument Judgment (2026-09-10): Cost Numbers of E126 Are Valid, While Framing Framework Is Annulled

**All three legs present** (local gate judged green). **Backward leg struck 6 points, one of which overturned the premise of E126 formulation.**

**Premise was false**: E126 "Why this cell was empty" section relied on D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) line 69 parenthetical enumeration excluding system configuration slot. Yet **class membership was settled long before elsewhere** ——
`crates/singlefs-checker/invariants/01-块自描述.md` under "I-1 (block self-describing)" paragraph beginning with "> Self-proving units" states verbatim "Self-proving units (journal records / root slots / **system configuration slots**)... explicitly exempt",
and `.claude/kb/decisions/18-块里携带什么信息.md` under "settled item 7: v1 unit header field tables, grouped by unit class" states verbatim "**Self-proving units (journal records / root slots / system configuration slots) do not use this table**".
And the rule assigned by D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) to **this class** is verbatim "**equals `physical_block_size` probed at runtime, hardcoding forbidden.**"

⇒ **By class membership, the answer already existed: Candidate A; and 'hardcoding forbidden' directly eliminates C and D.**
⇒ **All five cost cells reported by E126 (Costs of Four System Configuration Slot Width Candidates) remain completely valid** (criteria hardcoded before run, mutations all caught);
what was annulled was merely the framing of "four candidates filling a blank". C232 (system configuration slot width: class rule gave answer, field table says unset) has been revised accordingly.

**Other five strikes confirmed or marked pending re-review by main agent**:

| Strike | Status |
|---|---|
| B and A **differ only on the `md_raid5` pair**, matching cell by cell on first ten rows; and that cell is precisely where real-device tests in E34 (Root Ring Slot Geometry) measured adjacent slots within same chunk 10/10 intact, "**exposure surface ≠ loss surface**" | Main agent checked artifact, **confirmed true** —— failure mode improvement bought by B measured zero on the sole sampling point separating it |
| Heterogeneous pool sampling point `[(512,512),(4096,4096)]` in criterion 5 has `io_min == pbs` on both machines ⇒ **zero discriminative power for B**; switching to `[(512,512),(512,65536)]`, A is single-valued while B is not | **Confirmed true**, sampling point of criterion 5 in E126 was selected too narrowly |
| `io_min` is a **stack** attribute not disk attribute; local `man mdadm` GROW MODE explicitly states chunk-size can be modified online ⇒ B slot width would alter post-mkfs, while slot geometry is fait accompli on disk | **Pending re-review** (main agent has not run live dm-stripe demonstration) |
| D2 (RAID Stripe Policy) settled item 15 governing this **remained unsettled at the time (settled 2026-09-13, narrowing scope to fixed structures)**, while the same cell had already judged on real md/raid5 over 10 rounds on 2026-09-02 that "**slot width in this cell will not change for now**" | **Confirmed true** —— E126 is pure arithmetic, **overturning a conclusion judged on real devices using it is forbidden** |
| Active reference bounds precisely Candidate B: ZFS `MAX_UBERBLOCK_SHIFT (13)` caps at 8 KiB, using `ashift` rather than `io_min` | **Pending re-review** (no ZFS source tree locally, can only cite repository probe records from 2026-09-08) |

**Counter-deduction leg provided a fifth path (material's 'choice of four' was itself incomplete)**: E126 only built Candidate ① from the three candidates of
C212 (fixed-structure writes smaller than io_min) into an arm, leaving ② and ③ without arms.
Newly proposed **Candidate E: probe at mkfs, write value onto disk** —— simultaneously gaining B's `honors` and C/D's immutability,
while converting inconsistencies into "refusing to mount" (`.claude/rules/fs-design.md` "convert misuse into inability to mount").
⚠️ **Candidate E faces a structural challenge**: root slot guard holds because written value lives in system configuration,
whereas **system configuration slot itself lacks an outer container to store this value**. **Candidate E has not undergone tripartite review, not binding.**

### Measurement Basis and Replay

Code `scripts/experiment-apparatus/e126_system_configuration_slot_width.rs`
(`cargo run --release --bin e126-system-configuration-slot-width`),
raw output `research/results/e126-system-configuration-slot-width-2026-09-09.out` (172 lines).
Mutation table `scripts/mutation/tables/e126_system_configuration_slot_width.tsv`, **all 10 caught**.
**Registered into replay**: line in `scripts/experiment-apparatus/replay.sh` (mode `exact`), byte-for-byte identical upon replay.

⚠️ **Repaired a self-introduced non-compliance prior to registration**: closing line was originally written by hand as `name=count n=171`,
whereas integrity gate 2 in `replay.sh` recognizes `name=done emitted=N`, and **N must include config row and closing row itself**
(my 171 omitted itself, actual 172 lines) ⇒ **registering as-is would immediately judge red**.
Switched to `Emitter::finish()` (already in library), yielding `emitted=172` matching actual line count.
**This point was 'apparatus non-compliance' rather than 'erroneous conclusion'**: all five cost cells remained unchanged, reproduced byte for byte.

⚠️ **Pure arithmetic: running once provides as much information as running five times** (`.claude/singlefs-ai-sop/rules/test-discipline.md`
"Deterministic experiments can only state that running N rounds demonstrates absence of hidden state, not statistical stability") ⇒ run once,
evidence strength derives from mutation testing and pinned absolute value assertions.

⚠️ **Three mutations uncaught in first round, supplemented with sensitive sampling points after triaging per `.claude/rules/mutation-sampling.md`**:
`first_version_disks` was **true blind spot** (constant used only in `main()`, unit tests passed literal 2) ⇒ supplemented assertion pinning it;
`fits_strict` (`<=` → `<`) and `tear_sector_floor` (`div_ceil` → `/`) were both **sampling point insensitivity** ——
none of the six record byte counts equaled any slot width tier, and slot widths produced by all four candidates were exact multiples of 512
⇒ supplemented sensitive points (`fits(512, 512)`, `tear_states(600).0 == 2`), all three caught immediately.

⚠️ **Stepped on overflow before run**: `tear_states` first revision wrote `(1u64 << n) - 2`,
where unit tests only took 512 / 4096 / 16384 (n ≤ 32), panicking immediately upon reaching Candidate B's 65536 (n = 128).
**Same sampling point issue, only compiling clean and blowing up at runtime.**

### Influenced Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 2 | Raw material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 9 | Raw material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 8 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 21 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 16 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 9 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 9 | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | No effect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-09 (Run Completed)
- Run completed, **criteria unchanged by a single word**. Reverse acceptance clause triggered: none of the four holds overall dominance.
  10 mutations all caught (with 3 supplemented per sampling point insensitivity).

### 2026-09-09 (Design)
- Design registered, four candidates, five criteria, and five failure clauses **hardcoded before run**. Zero cells of data existed at this point.
  Basis: on the day C232 (system configuration slot width: class rule gave answer, field table says unset) was opened, user requested "Start system configuration slot width now".
