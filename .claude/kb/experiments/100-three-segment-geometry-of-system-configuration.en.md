## E100 Three-Segment Geometry of System Configuration — Run (2026-09-03, Pure Arithmetic, 9 Unit Tests / 6 Mutations All Caught): **On 512-Byte Slots Inlining Ceiling of Device Table Varies with Entry Width (24-Byte Entry 6 Units / 40-Byte 3 Units / 64-Byte 2 Units); Moving Out via Indirect Layer Constant 420 Bytes, Zero Torn States; Three-Segment Axis Cannot Accommodate Runtime Watermark, Requiring Fourth Category**

[中文](100-系统配置的三段几何.md) · **English** · [日本語](100-システム構成の3段幾何.ja.md)

**What it answers is D22 (Synthesizing Unit Atomicity) settled item 9 (field table of system configuration).** This item judged "bytes altering new-pool file creation: **yes**" — the first batch of bytes written by mkfs is precisely this.

### Cited Clauses (Copied Verbatim into Source Code Comments)

| Clause | The Sentence Used |
|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 8 | System configuration **one copy per disk**, updates follow **≥2 slot rotation** |
| D22 (Synthesizing Unit Atomicity) settled item 21 | In-place overwritten structures must carry **whole-unit checksum** and **actually checked generation number** |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | Atomic width takes `physical_block_size` probed at runtime ⇒ **one slot is precisely one sector** |
| D12 (Target Media) settled item 3 | Device-level geometric quantities live in **device descriptor table** inside system configuration; **heterogeneous pools must be supported** (one volume concurrently holding multiple sets of layouts) |
| D12 (Target Media) settled item 4 | Device identity mismatch ⇒ **refuse mount** |
| D23 (Journal Role & Format) | "**journal geometry enters system configuration** (ring size, max record bytes, in-flight record upper bound), **must not be code constants**" |
| D16 (Publish Semantics) settled item 5 | "`T_dirty` is tunable value, not format constant: that field in system configuration is format, what number placed inside field is not" |
| D15 (Format Freeze Policy) settled item 2 | Tree table day-1 implemented as **open list** (as of 2026-09-13 title dropped "system configuration", tree table lives in indirect layer of root record) |
| E72 (Mount-Time Recalculation of Device Descriptor Table) | Bytes in table exactly `device_count × entry_width`; entry width never settled in repo, calculated once under each of three tiers 24 / 40 / 64 |
| E87 (Placement of Fixed Structures) | System configuration single copy dropping disk 0 makes entire pool unmountable; one copy per disk + journal mirror makes all 8 cells mountable |

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Fitting or not fitting must be calculated as absolute values**: Sum of bytes across three segments vs slot width (512 / 4096 two tiers, derived from probing `physical_block_size`). **Report "fits / overflows how many bytes" cell by cell, no merely stating "might be insufficient".**
2. **Device table is the only thing here growing with scale**: Scan device count 1 / 2 / 8 / 64 / 256 × entry width 24 / 40 / 64, **counting the first cell overflowing slot**. This is an absolute value, not a trend.
3. **Way-out arm**: Move device table out of slot (system configuration stores only one pointer to device table unit, form identical to tree table unit indirect layer of D22 (Synthesizing Unit Atomicity) settled item 7), three-segment byte count must become **constant independent of device count**, and constantly fit across both slot width tiers.
4. **Atomicity criterion**: In `.claude/rules/fs-design.md`, "can wrong usage turn into refusal to mount" manifests here as **one slot must be exactly one sector**. If three segments exceed one sector, slot rotation is not atomic ⇒ **model must count "how many readable but inconsistent states exist across torn points for multi-sector slot"**, absolute values.
5. **Bootstrap ordering**: Fields in segment 1 must be decipherable **before reading device table**. Model evaluates by "parser reads only first N bytes", N increasing tier by tier, reporting which byte must be reached to "judge fsid / judge dev id / judge feature bits".

### Failure Clauses (Hardcoded Before Run)

| Clause | Disposition When Triggered |
|---|---|
| **Positive control**: Keep device table in slot, device count takes 64; 512-byte slot must judge overflow; after moving out must judge non-overflow | Not holding ⇒ slot width dimension not in model, **whole round voided** |
| **Negative control**: Device count 1, entry width 24, 4096 slot, must judge fitting | Judging overflow ⇒ model already errs on most lenient cell, whole round voided |
| **Reverse acceptance clause**: If way-out arm of criterion 3 still cannot fit across both slot width tiers | Conclusion is "slot rotation form of D22 (Synthesizing Unit Atomicity) settled item 8 must reopen", written faithfully |
| Entry width never settled in repo ⇒ calculate once under all three tiers | Three tiers yield differing directions ⇒ write as "dependent on entry width", establishing it as new blank |

### Actual Measurement (2026-09-03, Pure Arithmetic, N=5 Rounds Byte-Identical, 9 Unit Tests / 6 Mutations All Caught)

**Measurement basis and re-run**: Code `scripts/experiment-apparatus/e100_system_configuration_slot.rs`, artifact `research/results/e100-system-configuration-slot-2026-09-03.out` (127 lines, ending `emitted=127`), mutation table `scripts/mutation/tables/e100_system_configuration_slot.tsv`, hooked into `scripts/experiment-apparatus/replay.sh`.

#### Criterion 1: Absolute Byte Counts of Segments

| Segment | Bytes | Largest Item Within |
|---|---|---|
| 1 Bootstrap header | **162** | **feature bit 96** (D15 (Format Freeze Policy) verbatim: three bitmaps 256 bits each) |
| 1 D9 (Encryption) reservation | 24 | MAC 16 + nonce watermark 8 (D9 (Encryption) settled item 8 + D15 (Format Freeze Policy) stop-loss rule) |
| 2 Geometry (excluding device table) | 127 | Tree table unit pointer 59 |
| 3 Tunables | 36 | — |
| **4 Runtime watermark** | 12 | journal tail 8 + instance generation 4 |

Fixed overhead totals **361 bytes**. ⚠️ **Category 4 is newly added this time** — three-segment axis (who can modify it) cannot accommodate fields like tail slot "modified once per checkpoint", which is neither never-modified, nor mkfs read-only, nor human-adjusted tunable.

#### Criteria 2 + 3 + 4: Device Table Overflows in Slot, Constantly Fits When Moved Out

512-byte slot, 24-byte device table entry, including D9 (Encryption) reservation:

| Device Table | 1 Unit | 2 Units | 8 Units | 64 Units | 256 Units |
|---|---|---|---|---|---|
| **Kept in slot** | 385 ✓ | 409 ✓ | **553 ✗ overflow 41** | 1897 ✗ overflow 1385 | 6505 ✗ overflow 5993 |
| **Via indirect layer** | **420 ✓** | 420 ✓ | 420 ✓ | 420 ✓ | 420 ✓ |

=> **On 512-byte slots, device table inlining ceiling varies with entry width**: 24-byte entry **6 units**, 40-byte **3 units**, 64-byte **2 units** (`361 + device_count × entry_width ≤ 512`, pinned in unit test `criterion2_inline_device_table_overflows_a_512_byte_slot_early`).
⚠️ **Scan set in E100 (Three-Segment Geometry of System Configuration) criterion 2 contains only {1, 2, 8, 64, 256}, 3–7 units unrun** ⇒ ceiling cannot be read from table, which only shows "2 units fit, 8 units overflow". **Ceiling was computed, not scanned.**

**Slots spanning sectors are not atomic** (D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Corollary 3: tear judgment width = `physical_block_size`): spanning n sectors gives `2ⁿ − 2` readable but inconsistent states — 8 units **2 states**, 64 units **14 states**, 256 units **8190 states**. Via indirect layer, constantly **0**.

⚠️ **This is the identical shape root records already stepped into and rejected**: C78 (Root record lacks field table) (merged into C66 (format-level gaps have no item number) on 2026-10-11, original text in git history) verbatim recorded E79 (Capacity of Root Record) finding "512-byte slot fits only 6 trees, 7 trees overflows 43 bytes ⇒ **indirect layer structurally unavoidable**".

#### Criterion 5: Bootstrap Ordering

When segment 1 is laid out in declaration order, parser must read to byte **4** to judge magic, byte **6** to judge format version, byte **102** to judge feature bits, byte **118** to judge fsid, and byte **122** to judge local disk dev id.

### What It Cannot Answer

Pure arithmetic: no mkfs, no mount, no real device, 0 file operations.
**Does not answer transaction form of "how device table is modified when adding disks"** — that belongs to crash-point replay; E100 (Three-Segment Geometry of System Configuration) answers only "is it in an atomic slot" and "how many torn reading modes exist when it is not".

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 9 | Provisioning | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: this page provides absolute byte counts and device table inlining ceiling on 512-byte slots; field table cell was later recalculated for larger slots, this page serves only as provisioning) |
| D22 (Synthesizing Unit Atomicity) settled item 15 | Provisioning | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: "first version does not write device table" used this page's cell "device table moved to indirect layer leaves three segments constant 420 bytes, independent of device count") |
| D22 (Synthesizing Unit Atomicity) settled item 8 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: copied "one copy per disk, ≥2 slot rotation" as pre-run fact into source comments) |
| D22 (Synthesizing Unit Atomicity) settled item 21 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: copied "whole-unit checksum + actually checked generation number" as pre-run fact into source comments) |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took "atomic width is runtime-probed `physical_block_size`" as premise for "one slot is one sector") |
| D12 (Target Media) settled item 3 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took "device descriptor table lives in system configuration, heterogeneous pools supported" as input for device table segment) |
| D12 (Target Media) settled item 4 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took "device identity mismatch refuses mount" as pre-run fact) |
| D23 (Journal Role & Format) | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took "journal geometry enters system configuration, not code constants" as input for segment 3) |
| D16 (Publish Semantics) settled item 5 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took "field in system configuration is format, number inside is not" as input for watermark category) |
| D15 (Format Freeze Policy) settled item 2 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-19 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took tree table day-1 as open list as pre-run fact) |
| D9 (Encryption) settled item 8 | Unaffected | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-19: unaffected: only took system configuration's own MAC and nonce watermark as source for 24-byte reservation in segment 1) |

## Historical Versions

### 2026-09-07
- **Corrected one incorrect conclusion number**: Title and criterion 2 conclusion sentence originally wrote verbatim "on 512-byte slot device table inline fits only 2 disks", whereas E100 (Three-Segment Geometry of System Configuration) **own unit test** asserted verbatim `total_bytes(6, 24, true, true) == 505` (fits) and `total_bytes(7, 24, true, true) == 529` (overflows 17) — **the body contradicted its own test**. True values per entry width are 6 / 3 / 2 units. Cause of error was reading scan set of only {1,2,8,64,256} as ceiling: 2 units fit in table, 8 units overflowed, transcribed into "fits only 2 units". The original phrase "64-byte entry tier reaches top **earlier**: 2 units" was the fingerprint of this error — both tiers were 2 on sampled cells, and "earlier" was meaningful only when true values differed, but when writing it the true values had never been computed.
- **Rationale**: Pointed out by backward leg of three-way argument on 2026-09-07; main agent recalculated via `python3 -c "b=361;print([(e,max(d for d in range(1,40) if b+d*e<=512)) for e in (24,40,64)])"` to confirm, matching unit test in source code of E100 (Three-Segment Geometry of System Configuration) word for word.
- ⚠️ **This number had already been inherited downstream**: Narrowing section of D22 (Synthesizing Unit Atomicity) settled item 9 verbatim copied "fits only 2 disks", corrected together on same day.

### 2026-09-03

#### (Part 1)
- Design registration, criteria and failure clauses hardcoded before run. Rationale: D22 (Synthesizing Unit Atomicity) settled item 9 lacked even argumentation materials across repo, while local leg hit "device table grows with device count while slot width does not" and "multi-sector slot is not atomic write" in round 1.

#### (Part 2)
- Run on same day and written back to measurement. Criteria 1..N and failure clauses unchanged by a single word.

