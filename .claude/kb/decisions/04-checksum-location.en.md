## D4 Checksum Location —— Settled

[中文](04-校验和位置.md) · **English** · [日本語](04-チェックサム配置位置.ja.md)

D4 (Checksum Location) governs where the checksum lives and what coverage it provides: checksums are inlined into the parent pointer pointing to the block (parent node stores child node checksum), turning the entire tree into a self-verifying Merkle tree capable of detecting correctly formed but stale blocks read from disk, without establishing an independent csum tree like btrfs. When encryption is enabled, this Merkle field stores the MAC (D9 (Encryption) settled item 2); checksum granularity is MAC granularity, and encryption introduces no separate tree layer; what it takes when encryption is disabled is in settled item 8. Pointers contain two integrity fields, not one: the Merkle field serves the keyed side, while the keyless ciphertext checksum (D9 (Encryption) settled item 1, D19 (Block Pointer Structure & Width Budget) settled item 2, I-6.8 (ciphertext checksum matches)) serves the unkeyed side. Out of scope: widths and offsets of pointer segments belong to the offset table in D19 (Block Pointer Structure & Width Budget) settled item 11 (byte table in [layout/01-first-txn.md](../layout/01-first-txn.md) Section III), with total pointer width not acting as a constraint; node size belongs to D8 (Core Index Structure) settled item 2 / settled item 4. Stale blocks lack a check here, owed in C7 (no check for stale blocks); accounting for node size conclusion rerun at final pointer width is in C386 (the node size conclusion owes a rerun at the final pointer width, unnumbered).

### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Tension between checksum granularity and random small reads** | Data extent checksum / MAC covers 32 KiB; random small reads pay approximately 10% more as a result, accepted with awareness **Status: Settled.** **Progress: Implemented.** |
| 2 | **How extents smaller than a unit are calculated** | Padded to 32 KiB: units are strictly one invariant size, checksum / MAC covers the whole unit after padding **Status: Settled.** **Progress: Implemented.** |
| 3 | **How writes not filling an entire unit are written** | Read-modify-write: read back old unit to verify MAC, recompute whole-unit MAC after edit, write whole unit to new location; no exceptions for initial writes **Status: Settled.** **Progress: Unimplemented.** Whole-content writes and COW exist; partial-write RMW at arbitrary offsets has no entry point. |
| 4 | **What to do when read-modify-write cannot read back old unit** | That write fails and returns an error, zero-fill reconstruction strictly forbidden; padding bytes are always zero and included in checksum **Status: Settled.** **Progress: Unimplemented.** Zero padding and its checksum are implemented; read-failure rejection for partial writes still needs an entry point and fault-injection validation. |
| 5 | **Whether 32 KiB refers to with-header or net payload** | With header: unit is strictly 32,768 bytes; self-describing header, declared length, payload, and padding all reside within these 32,768 bytes **Status: Settled.** **Progress: Implemented.** |
| 6 | **Where extent declared length lives, how wide** | Lives in unit self-describing header, 2 bytes wide **Status: Settled.** **Progress: Implemented.** |
| 7 | **Relationship between logical extent and unit** | One logical extent strictly occupies one unit on disk (32,768 bytes), never spanning multiple units; declared length variable, ≤ 32,768; splitting reserved, disabled in version 1 **Status: Settled.** **Progress: Implemented.** |
| 8 | **What Merkle field takes when encryption disabled; 5% reopening criterion baseline for settled item 1** | The 4-byte CRC32C in the position entry is the Merkle checksum, covering all bytes of pointed unit; device-side multiplier for random small reads ≥ 1.136 reopens settled item 1 **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Tension Between Checksum Granularity and Random Small Reads

<!-- format-const: DATA_UNIT_BYTES = 32768 -->

**Settlement**: Data extent checksum / MAC covers 32 KiB. Inlining **couples** checksum granularity with extent granularity; random small reads within large extents are forced to read and verify an entire unit—this tension does not disappear, but is accepted. Numerically, the settlement is equivalent to "accepting paying approximately 10% on random small reads", not equivalent to "32 KiB is faster on this axis"; citing it as the latter is forbidden.

**Scope**: Governs only the data side, granularity tied to extents, not nodes. It and node size are two distinct numbers, permitted to be unequal: the 16 KiB in D8 (Core Index Structure) settled item 2 governs only index nodes; a node is itself a unit, whose checksum granularity follows the node ⇒ 16 KiB on metadata side, 32 KiB on data side. Reopening conditions: device-side multiplier for random small reads ≥ 1.136 (settled item 8; judged by device-side multiplier, not total multiplier including AES); when media changes, target workload shifts to hotspot working sets, or workload weights in D25 (Target Workload Priority) change, underlying premises change, requiring rerun of E58 (End-to-End Cost of Checksum Granularity) to re-evaluate.

**Basis**:

- E58 (End-to-End Cost of Checksum Granularity): Cost of 32 KiB relative to 16 KiB falls on random small reads, mostly on the device side, while gains on sequential side come solely from AES chunking, zero on device side—tension genuinely exists, paying approximately 10%.
- E6 (Encryption Algorithm Selection): Under warm buffer, large units buy no throughput on CPU side; this holds only for warm buffer; the random small read path buffer is permanently cold, and what throughput can be bought is less than device cost on that same path, direction unchanged.
- User settlement, date and process in history.

**Owed**: C238 (derived number "53" after position entry changed from 11 to 14 un-back-scanned): Critical random small read proportion by byte measurement basis recalculated at current pointer width (pointing to code 1 is 88 bytes, D19 (Block Pointer Structure & Width Budget) settled item 11); C357 (two reopening criteria on same axis never synthesized): Intersection between reopening criterion and settled item 5 lies on the same axis, lacking composition rules.

#### Settled Item 2: How Extents Smaller Than a Unit Are Calculated

**Settlement**: Units are strictly 32 KiB. Short extents (small files, tail blocks, truncated extents) are padded to whole units, with checksum / MAC covering the entire unit after padding ⇒ no "variable-length unit" branch exists in format.

**Scope**: Cost is space amplification, falling entirely on small files, entering ENOSPC acceptance via D5 (Snapshot / Space Accounting Mechanism) accounting. On the write path after padding, packing multiple small files into one unit is not performed, a statement holding only under single-operation measurement basis (C168 (E66 pack arm covers only the worst implementation)); outside the write path, background compaction moves small objects into shared containers, belonging to D27 (Small-Data Packing Container), with writes still padded per settled item 2; inline threshold belongs to D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 (settled at 0).

**Basis**:

- E65 (Write-Side Cost of Checksum Granularity): The `pad` arm identity yields padding space amplification, falling solely on small files, proportional to unit size.
- E66 (How Small Files Land After Padding): On-write-path packing after padding wins nothing in I/O, buying only space; inline points win reads but write is an order of magnitude more expensive—padding did not overturn the independent extent placement method.
- User settlement, date and process in history.

**Owed**: C168 (E66 pack arm covers only the worst implementation): Add batching and background compaction arms to packing, rerun on identical device and report absolute values.

#### Settled Item 3: How Writes Not Filling an Entire Unit Are Written

**Settlement**: First read back the entire old unit and verify MAC, recompute MAC for whole unit after edit, then write whole unit to new location (COW, never overwriting in place). Initial writes have no exception: all writes not filling an entire unit go through RMW, opening no second publish path for "initial writes" (aligned with `.claude/rules/fs-design.md` "one transaction layer, shared by all structures"); initial writes without readback would be much faster; this cost is accepted with awareness, buying a single write path.

**Scope**:

- Newly allocated units have no old unit to read: "taking RMW" here reads the residue of the new slot assigned to this publish once, discards the bytes and takes the old content as zeros; a read error on the residue is counted and does not refuse the write (user settlement 2026-10-26); residue bytes never enter the new unit (the residue is often an old unit of the same file). The number of read-backs of one partial write = the number of touched units whose payload is not fully covered by the write, and a unit that has an old unit reads that old unit (verdict `research/prompts/m4-partial-write-r1-main-verification.md` Section 2, cell "first write"). C385 (two settled items contradict each other on whether the first write needs a read-back) was settled by the user on 2026-10-26 as "the first write also reads back", and the gains column of D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 changes accordingly.
- **Implementation boundary**: `publish_overwrite_of_an_inode` and `publish_sequential_write_of_an_inode` in `crates/singlefs-core/src/transaction.rs` accept whole contents and replace from the beginning; `build_data_unit` in `crates/singlefs-core/src/unit.rs` constructs a unit in a fresh buffer. There is no arbitrary-offset partial-write entry point; the specialized read-modify-write of a truncated tail does not provide general partial writes. The unencrypted path computes CRC, not MAC.
- When old units cannot be read back or MAC fails verification, even writes "wishing to modify only 4 KiB" cannot proceed (recomputing whole-unit MAC requires remaining plaintext); what to do belongs to settled item 4.

**Basis**:

- E65 (Write-Side Cost of Checksum Granularity): Measured byte and time costs of writes not filling a unit taking RMW; byte amplification far exceeds time cost—the price settled item 3 accepted with awareness.
- E66 (How Small Files Land After Padding): Initial writes without readback are an order of magnitude faster than reading back whole unit and rewriting; difference is that device round-trip, not bytes—cost paid for "no exceptions for initial writes".
- Verdict `research/prompts/m4-partial-write-r1-main-verification.md` Section 2, cell "first write": the cell-by-cell read-back rule in which a newly allocated unit reads the residue once, discards the bytes and takes the old content as zeros (proposed by the forward leg, attacked zero rounds); user settlement 2026-10-26: C385 (two settled items contradict each other on whether the first write needs a read-back) "the first write also reads back", and a read error on the residue "is counted and does not refuse the write" (original wording in history).
- User settlement, date and process in history.

**Owed**: C57 (Zero-Fill Displaces Original Content): Failure coupling aspect verifiable only via fault injection; C385 (two settled items contradict each other on whether the first write needs a read-back): the reading is settled (first bullet of the scope); cases pinning the read-back count and read sequence are still to be implemented.

#### Settled Item 4: What to Do When Read-Modify-Write Cannot Read Back Old Unit

**Settlement**: When old unit cannot be read back, or read back with MAC verification failure, that partial write fails and returns an error. Zero-filling to synthesize a new unit is strictly forbidden—that turns "lost data segment" into "valid segment of zeros", which on disk is byte-for-byte indistinguishable from real data. Padding bytes are always 0, and checksum covers the whole unit including padding.

**Scope**: Padding bytes being zero is single-image verifiable, an invariant I-2.3 (padding bytes are zero and participate in the checksum) ([invariants/04-校验和.md](../../../crates/singlefs-checker/invariants/04-校验和.md)), judged by pool-level checker (`crates/singlefs-checker/src/walk.rs`); "must fail when readback fails" is not single-image verifiable—a unit reconstructed with zero-fill has self-consistent checksums—verifiable only via fault injection. When encryption is enabled, RMW must first decrypt the old unit before modifying ⇒ unkeyed side cannot perform partial writes; D9 (Encryption) settled item 5 settled as "scrub only", unkeyed side does not move data. Node-side recoverability belongs to D8 (Core Index Structure) settled item 4, not in this cell.

**Basis**: No experiment: "Fail when readback fails" is a correctness rule with no measurable quantity, verifiable only via fault injection checks, not counting experiments. Constraining padding bytes to zero rather than "anything goes" is a discriminative power requirement: without constraint, checker cannot separate "valid padding" from "original text displaced by zero-fill", and I-2.3 (padding bytes are zero and participate in the checksum) would turn green on both. User settlement, date and process in history.

**Owed**: C57 (Zero-Fill Displaces Original Content): Corrupt read of a unit, then issue a sub-unit write to that extent; that write must fail.

#### Settled Item 5: Whether 32 KiB Refers to With-Header or Net Payload

**Settlement**: 32 KiB with header. Units are strictly 32,768 bytes; self-describing header, declared length, payload, and padding all reside within these 32,768 bytes.

**Scope**: Known cost is payload is not a power of two: user data granularity = `32768 − header − extension point declared value N`; file offset → unit requires division rather than shifting, and N varies with line declarations; on read path, over 10% of 4 KiB file pages straddle two units; payload starts at offset h, preventing page cache and O_DIRECT buffers from achieving zero-copy. These two layers intersect with random page read proportions; D25 (Target Workload Priority) has no numerical weights, not reopening settled item 5 based on this.

**Basis**:

- I-2.3 (padding bytes are zero and participate in the checksum) verbatim: "checksum / MAC covers **entire unit including padding**", settled item 1 verbatim "covers 32 KiB" ⇒ the covered 32,768 is the entirety of the unit; the alternative interpretation (32,768 refers to payload, on-disk unit = 32,768 + header + N then aligned) would make "covers 32 KiB" point to a range smaller than the unit, directly conflicting with I-2.3 (padding bytes are zero and participate in the checksum). Unit size is a constant, independent of header length or N ⇒ allocator, root slot alignment, and `physical_block_size` divisibility all answer to a single number.
- E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header): Two-layer cost of header on read path (cross-unit pages, read-side copies) opposes padding shape between random and sequential; reports only intersection point, constituting no rationale for reopening.
- User settlement, date and process in history.

**Owed**: C357 (two reopening criteria on same axis never synthesized): Intersection point and settled item 1 reopening criterion lie on the same axis.

#### Settled Item 6: Where Extent Declared Length Lives, How Wide

**Settlement**: Extent declared length lives in the unit self-describing header, 2 bytes wide: range 0..32768, 16 bits suffice. 4 bytes rejected—4 bytes was estimated for 128 KiB unit size class, while settled item 1 already settled units at 32,768.

**Scope**: If unit size is modified in future to > 64 KiB, this field must be reopened. When encryption is enabled, it is ciphertext (D18 (Information Carried in a Block) settled item 13, arm G): location and width unchanged, merely removed from plaintext prefix whitelist shared across three classes; settled item 6 requires it to live on unit itself, not requiring plaintext. Code 1 declared length counted from end of header containing reserved bits at 134, upper bound 32,768 − 134 (D18 (Information Carried in a Block) settled item 16); implementation written at offset 8 of common prefix in 2 bytes (`write_common_prefix` and `declared_length` in `crates/singlefs-core/src/unit.rs`).

**Basis**: No experiment: Where it lives is derived from I-2.3 (padding bytes are zero and participate in the checksum); width is domain arithmetic, with no measurable cost. I-2.3 (padding bytes are zero and participate in the checksum) must judge that from declared length to unit end is permanently 0 without an index—at which time index key ranges cannot be read ⇒ declared length can only reside on unit itself; omitting it leaves the criterion unjudgeable on scan rebuild paths, and C57 (Zero-Fill Displaces Original Content) presses precisely on this criterion. User settlement, date and process in history.

**Owed**: None.

#### Settled Item 7: Relationship Between Logical Extent and Unit

**Settlement**: On-disk length of a logical extent is strictly 32,768 bytes; one extent per unit, never spanning multiple units; declared length variable, ≤ 32,768. The two lengths must be written separately; padding starting point of I-2.3 (padding bytes are zero and participate in the checksum) uses declared length:

| Length | Value Range | Consumer |
|---|---|---|
| **Declared length** | ≤ 32768, user data | Padding start for I-2.3 (padding bytes are zero and participate in the checksum); settled item 6 settled where it lives |
| **On-disk length** | **Strictly 32768** | Checksum / MAC (settled item 1); counted per D19 (Block Pointer Structure & Width Budget) "one copy per logical extent" |

Splitting is a reserved feature, disabled in version 1, `extent_off` strictly 0. Bits preserved rather than removed per D19 (Block Pointer Structure & Width Budget) settled item 3 policy of reserving fixed-width bits, not because version 1 uses them; in the future, only compression could resurrect splitting—when packing multiple compressed extents into one unit, `extent_off` would take non-zero values.

**Scope**: The tradeoff of variable vs immutable is not reopened: variable declared length saves not a single byte, on-disk footprint remains 32,768; saving requires changing on-disk length, while settled item 2 states verbatim "no 'variable-length unit' branch in format"; reopening must not argue multiples (`.claude/rules/fs-design.md` "comparing who saves more constitutes no criterion"), but must measure when ENOSPC acceptance hits the wall under small file workloads. Splitting settlement touches no bytes in new-pool file creation (those 2 bytes occupy space whether used or not); what it alters is "whether one MAC copy is duplicated into k copies", strictly one copy in version 1. Compression side reserves bits in version 1, strictly 0: compression algorithm code and compressed length in pointer header (D19 (Block Pointer Structure & Width Budget) settled item 7 / settled item 11), `PointerHead::write_to` writes 0 in `crates/singlefs-core/src/pointer.rs`.

**Basis**:

- Four settled clauses, not the "clamping from both ends" deduction (which was punctured by the local leg of three-way argument, details in history): D19 (Block Pointer Structure & Width Budget) opening: "logical attributes (MAC, nonce, algorithm type, extent offset from origin) in header, one copy per logical extent"—singular; MAC and nonce in D19 (Block Pointer Structure & Width Budget) settled item 3 are fixed-width slots, unable to hold N length-varying copies; D9 (Encryption) settled item 4 "pointer header **stores one copy** of 96-bit nonce for each logical extent"; settled item 1 "checksum granularity **coupled** with extent granularity"—coupled means equal, spanning multiple units uncouples them.
- Corroborating evidence: D19 (Block Pointer Structure & Width Budget) settled item 11 gives extent offset from origin 2 bytes (offset 32), unable to fit 17 bits needed for 128 KiB extents; D18 (Information Carried in a Block) settled item 6 verbatim "unit (extent) level", directly glossing unit with extent in parentheses.
- E58 (End-to-End Cost of Checksum Granularity): Source code amortizes over parent-node pointer payload; dilution ratio cited in settled item 1 calculated precisely on one extent per unit—corroborating evidence.
- E66 (How Small Files Land After Padding): On-write-path sub-unit packing eliminated; splitting has no source on write path.
- Splitting: Settled item 3 mandates writes not filling a unit must read back whole unit, edit, and rewrite to new location; together with one extent occupying one unit, no operation can create "one physical unit referenced by multiple pointers from different offsets"; C72 (nobody answers how extents relate to units) three-way argument round (`research/prompts/c72-extent-split-*.md`) checked reflink, deduplication, RAID rebuild, journal replay, snapshot destruction, restriping, and tombstone packing, finding zero counterexamples.
- User settlement, date and process in history.

**Owed**: C72 (nobody answers how extents relate to units): Gate compares field frequency like "one copy per X", where X must point to a size given by settled clauses (check for `extent_off` strictly 0 in version 1 exists: `judge_pointer_extent_offset_is_zero` in pool-level checker (`crates/singlefs-checker/src/image.rs`) judges every data pointer, core on-disk pointer header readers reject non-zero, belonging to C581 (non-zero extent offset lacks judge)).

#### Settled Item 8: What Merkle Field Takes When Encryption Disabled; 5% Reopening Criterion Baseline for Settled Item 1

**Settlement**: When encryption is disabled, the Merkle checksum in parent pointers is simply the 4-byte CRC32C in the position entry, covering all bytes of pointed unit (32,768 for code 1 / code 3, 16,384 for code 2), one copy per physical placement, opening no separate field; version 1 self-verification strength of whole tree is 32 bits, accepted with awareness. Reopening criterion for settled item 1 is "random small read device-side multiplier ≥ 1.136 (E58 (End-to-End Cost of Checksum Granularity) measured 1.086 plus 5 percentage points reopens)", judged on device-side multiplier.

**Scope**: When encryption is enabled, Merkle field stores MAC (D9 (Encryption) settled item 2), not in this cell. Implementation: write side `location_entries` (`crates/singlefs-core/src/make_filesystem.rs`) computes CRC-32C over whole unit into two position entries; read side `read_unit_via_locations` in `crates/singlefs-core/src/recovery.rs` compares whole-unit CRC-32C against `unit_checksum` in position entries.

**Basis**:

- E58 (End-to-End Cost of Checksum Granularity): Device-side multiplier after stripping verification is reopening baseline.
- Pre-coding master audit (`records/2026-09-13-总审核.md` D4 (Checksum Location) sections) recommended based on "no third checksum slot in pointer header", user settlement, date and process in history.

**Owed**: C357 (two reopening criteria on same axis never synthesized): Reopening criterion and settled item 5 intersection point lack composition rules.

## Historical Versions

Historical entries for D4 (Checksum Location) are centralized in [decisions-history.md](../decisions-history.md).
