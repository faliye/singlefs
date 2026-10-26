## D19 (Block Pointer Structure & Width Budget) —— Settled

[中文](19-块指针的结构与宽度预算.md) · **English** · [日本語](19-ブロックポインタ構造と幅予算.ja.md)

D19 (Block Pointer Structure & Width Budget) governs block pointers: how many sections a pointer is divided into, what fields are contained in each section, their widths, and their ordering; who holds the authority between position entries and the central mapping for dereferencing and release judgments; how the central mapping entry key and value are encoded; which segments of the mapping key travel with the pointer, and how birth sequence numbers are allocated. Out of scope: unit header field tables and unit type codes in D18 (Information Carried in a Block); node size and intra-tree entry layout in D8 (Core Index Structure); placement coordinate system and allocation record keys in D3 (Space Allocation) settled items 3 / 7, commit endogenous block bump sequence in D3 (Space Allocation) settled item 10; checksum coverage and the relationship between logical extents and units in D4 (Checksum Location); encryption algorithms, nonce derivation, and origin of expected AAD values in D9 (Encryption); root record and system configuration field tables in D22 (Synthesizing Unit Atomicity) settled items 7 / 9; stripe width granularity in D2 (RAID Stripe Policy) settled item 1.

<!-- doc-lint:registry name-col=2 -->

| Identifier | Short Name |
|---|---|
| N1 | Reread once upon read failure |
| N3 | Mismatched entries remain in allocated |

### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Whether position entries carry device identity** | Yes: each physical position carries a dev id, without introducing a layer of chunk mapping table **Status: Settled.** **Progress: Implemented.** |
| 2 | **Bit-width of ciphertext checksum** | 32 bits; takes complete CRC32C, truncation from a wider CRC is forbidden **Status: Settled.** **Progress: Implemented.** |
| 3 | **Fixed-width or variable-length** | Fixed-width, reserving slots when unencrypted: MAC and nonce constantly occupy slots, kept empty and not repurposed **Status: Settled.** **Progress: Implemented.** |
| 4 | **Width of each section in position entry** | Device ID 4 bytes, physical offset 6 bytes (16 KiB slot number), ciphertext checksum 4 bytes => position entry 14 bytes **Status: Settled.** **Progress: Implemented.** |
| 5 | **Where location authority resides** | Position entry demoted to hint, central mapping is unique entrance for dereference and release verdicts **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Field table of central mapping entries** | Key reuses write sequence key by class (post-tightening): 27 bytes for Code 1, 25 bytes for Code 2 / Code 3; value follows settled items 4 / 5 **Status: Settled.** **Progress: Implemented.** |
| 7 | **Which segments pointer header carries** | Header 50: MAC 16, nonce 12, algorithm type 1, compression algorithm code 1, compressed length 2, extent offset 2, birth tree 8, birth txg 8 **Status: Settled.** **Progress: Implemented.** |
| 8 | **How much birth identity reference side carries, what mapping contains** | All pointers enter mapping, pointers to Code 1 are 88, pointers to Code 2 / Code 3 are 86; mapping tree nodes, tree table units, instance table units bootstrap exempt **Status: Settled.** **Progress: Implemented.** |
| 9 | **How birth sequence numbers are allocated and in what order** | Starts from 0 within tree, Code 2 and Code 3 share one counter, clears upon checkpoint switch, changes number upon rewrite; order matches bump order of commit endogenous blocks **Status: Settled.** **Progress: Implemented.** |
| 10 | **Whether mapping key is fixed-width or variable by class** | Strictly 27 bytes across the board, Code 2 / Code 3 padded with zeroes at tail; central mapping entries strictly 55 bytes across the board **Status: Settled.** **Progress: Implemented.** |
| 11 | **Pointer offset table, position entry order, where mapping tree root resides** | Segments arranged per offset table, position entries in ascending order of device identity; mapping tree root lives in root record, root record totals 466 **Status: Settled.** **Progress: Implemented.** |
| 12 | **Whether Code 2 / Code 3 mapping entries are written day-1, how many times fixed point writes** | Written day-1; an instance writes out each Code 2 node and Code 3 container only once within a checkpoint **Status: Settled.** **Progress: Implemented.** |
| 13 | **How many sections a block pointer is split into** | Split into two sections: logical properties in header, one per logical extent; physical properties in position entries, one per physical placement **Status: Settled.** **Progress: Implemented.** |
| 14 | **Whether position entries can be deduced geometrically from single origin** | Cannot, enumerated individually **Status: Settled.** **Progress: Implemented.** |
| 15 | **Value range of extent offset from origin and version 1 value** | Range [0, 32768), 15 bits sufficient; version 1 constantly 0, bits reserved as specified **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Position Entries Carry Device Identity

**Settlement**: Position entries carry device identity (dev id), one copy per physical position; no indirection layer of "logical address + chunk mapping table" is introduced.

**Scope**: Governs "whether to carry", not width (settled item 4) or ordering (settled item 11). The "per-occurrence variable" property of D2 (RAID Stripe Policy) settled item 1 is the premise for the forward-proving side of this cell, and carrying dev holds under both granularities (it is the more capable side) => granularity settlement only affects whether position entries can be compressed in expression, not whether they are carried. The viability of wide pointers is coupled with node size in D8 (Core Index Structure): for the same 40-byte pointer header, a 4 KiB node pays +98% whereas a 16 KiB node pays only +14%; D8 (Core Index Structure) settled item 2 selects 16 KiB and pins it as a constant. E20 (CPU Cache Cost of Widening Pointers) measured a 40-byte header, while the settlement is 50 (settled items 7 / 11); the extra bytes were measured in E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) showing accounting tree point lookups slow down by 1.8%–4.5%, leaving the premise intact. The largest cost of carrying dev (removing a device requires a full run scan and rewrite of every tree with pointers) is reduced to O(references of that device) by the reverse index settled in D1 (Data Mobility / Reverse Index), a path this project was already committed to taking.

**Basis**:

- E20 (CPU Cache Cost of Widening Pointers): the cost of pointer widening appears only when the working set exceeds L3 cache, which 16 KiB nodes absorb—the direct premise for wide pointers being viable in this project, and the basis side of D8 (Core Index Structure) settled item 2.
- Production implementations (inspected live 2026-08-28, not verified in this project): bcachefs with dev requires a full run scan and rewrite of every tree carrying pointers to remove a device (`bch2_dev_usrdata_drop()`), relying on `BTREE_ID_backpointers` to reduce it to O(references of that device); ZFS welds dev into blkptr, so device removal can only add an immortal layer of `vdev_indirect_mapping` (can only condense, cannot delete); btrfs without dev must carve out a fixed, non-expandable `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]` (2048 bytes, definition of `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` in `include/uapi/linux/btrfs_tree.h` and this field in system configuration structure) in system configuration for bootstrap, and balance still rewrites all references.
- Three-way argument (2026-08-28): forward leg and backward leg reached opposite conclusions, and the fact distinguishing them was the variable granularity of D2 (RAID Stripe Policy); local leg invalidated for the round, leg circumstances and process in changelog.

**Owed**: None.

#### Settled Item 2: Ciphertext Checksum Bit-Width Takes 32 Bits

**Settlement**: Ciphertext checksum in position entries takes 32 bits, exactly the width of full CRC32C, involving no truncation; implementations are forbidden from taking the path of "truncating a wider CRC down to 32 bits".

**Scope**: Conclusion is conditional, and conditions must be cited together when referenced—it holds only under this set of inputs: 10 TB pool, 4 KiB blocks, weekly scrub, 10 years, corruption rate 1e-9 per block read, expected undetected upper bound 1e-6. Reopening condition is decidable: if field corruption rates are obtained exceeding 3.37e-9, this cell reopens on the spot. None of the candidate algorithms were eliminated (deviations were all within one order of magnitude). Media corruption is not an attacker in the threat model; this cell does not judge attack resistance.

**Basis**:

- E40 (Ciphertext Checksum Bit Width): minimum bit-width under this input set is 31 (matching rounding of original derivation "n > 30.3"), margin for 32 bits is 3.37x—each of the five inputs (pool capacity, corruption rate, service life, scrub interval, block size) must be off by this factor for 33 bits to be needed; a truncated CRC is not a shorter CRC—full-width CRC32C having 0 undetected single-bit flips is a structural property, whereas truncation turns it into probabilistic detection.
- Collateral evidence (inspected live, not verified in this project): default checksum of btrfs is 32-bit CRC32C (`BTRFS_CSUM_TYPE_CRC32 = 0`), its 256-bit slot is reserved for optional SHA256 / BLAKE2.
- User settlement, verbatim words and that day's derivation in changelog.

**Owed**: None.

#### Settled Item 3: Fixed-Width, Slots Reserved When Unencrypted

**Settlement**: Pointers take fixed width per worst-case scenario—MAC (128 bits) and nonce (96 bits) in the header constantly occupy slots; unencrypted volumes leave them as empty slots, avoiding omission via type flags; these bits remain occupied across all volumes and are not repurposed for other fields.

**Scope**: What is optional is the AAD computation step (not computed when encryption is off), not the format (`.claude/rules/fs-design.md` "encryption is an optional computation, not an optional format"). The mechanism of opening layout branches per medium cannot govern this cell: incompat bits govern whether readers recognize this layout, while this cell governs how a pointer is encoded within the same layout; adopting fixed width first and adding a variable-length layout later is not expansion, but an extra parser plus checker plus crash-point replay that must be permanently maintained. Leaving MAC 16 bytes + nonce 12 bytes = 28 bytes / logical extent unoccupied when unencrypted represents `28 / 32768 = 0.0854%` under D4 (Checksum Location) settled item 7 measurement basis (logical extent occupies exactly 32768 bytes on disk). Variable-length saves those 28 bytes rather than saving a field (algorithm type field D9 (Encryption) settled item 12 item 4 must exist regardless), while paying the cost that parsing must proceed from the beginning: fixed-width pointers can directly locate the k-th item by offset, whereas variable-length cannot; "whether scanning a partially corrupted node during recovery can skip a bad item and continue reading" is a deduction, not empirically measured. When encryption is off, MAC 16 and nonce 12 are constantly 0, readers encountering non-zero judge the structure containing the pointer corrupted, following I-2.4 (header checksum coverage) reading for unit header 29 bytes (audit A3a landed in two reader sites—multi-layer Code 2 tree internal entries, journal new root segment pointer: `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero` in `crates/singlefs-core/src/pointer.rs`); extent offset 2 bytes is constantly 0 in version 1 (version 1 pointers point to whole units), readers encountering non-zero judge the structure containing the pointer corrupted, regardless of encryption status; checker checks on every traversed pointer, core checks in every reader of on-disk pointer headers, landing in [checks-owed.md](../checks-owed.md) repaid table C581 (non-zero extent offset lacks judge) row.

**Basis**:

- No experiments: switching from fixed to variable length saves space at the expense of "whether an isolated pointer can be resolved directly by offset", which are different dimensions with no quantifiable metric. It is underpinned by two rules in `.claude/rules/fs-design.md`—"never sacrifice self-containment to save space", and the criteria for format branches (testing whether it turns "misuse" into "unmountable", without permitting "this is faster / leaner" in criteria).
- User settlement, verbatim words and reaffirmation on 2026-09-12 in changelog.
- Verdict 2026-10-01 adopted: extent offset non-zero in version 1 judges the structure containing the pointer corrupted (`research/prompts/m3-inv-a-r1-main-verification.md` section 2 line 11).

**Owed**: C210 (Nobody Has Judged Which Kind of Extension the Three 'Second Sets' Are).

#### Settled Item 4: Location Entry Width Takes 14 Bytes

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**Settlement**: Device ID 4 bytes, physical offset 6 bytes (16 KiB slot number), ciphertext checksum 4 bytes => position entry 14 bytes.

| Segment | Width | Why this width |
|---|---|---|
| Device ID | **4** | Upper bound points to no settled clause (D12 (Target Media) does not cap media tiers, repository has no dev id recycling clause => narrow width caps total device identities across pool lifetime), takes wide tier per `.claude/rules/fs-design.md` "never sacrifice self-containment to save space"; precedent is settled item 3 |
| Physical offset | **6**, encoded as **16 KiB slot number** | Index nodes aligned to 16384, size is exactly 16 KiB (D18 (Information Carried in a Block) settled item 9) => encoding as 32 KiB unit number cannot express placements of half the index nodes, while byte offset makes non-16 KiB-aligned origins representable. 6-byte slot number addresses 4 EiB / device, whereas same-width byte offset addresses only 256 TiB |
| Ciphertext checksum | **4** | Settled item 2 |

**Scope**: Physical offset shares identical coordinate system with D3 (Space Allocation) settled item 7 (D3 (Space Allocation) settled item 3 basis 2 verbatim requires identical coordinate system) => settled together, independent divergent definitions forbidden. Tree table unit pointer and root record total downstream counts adjust accordingly, and root record total is not registered here; the sole authoritative location is the field table of D22 (Synthesizing Unit Atomicity) settled item 7. Relative to legacy 11 bytes, each position entry adds 3 bytes, each pointer adds 6 bytes.

**Basis**:

- E97 (Entry Encoding of Accounting and Allocation Records): criterion 7 judges placement encoding by representability—when encoded as 16 KiB slot number, all legitimate origins are representable, whereas half cannot be written under coarser unit numbers.
- User settlement, verbatim words and review of "this figure was consumed without passing through decision process" in changelog.

**Owed**: C238 (derived number "53" after position entry changed from 11 to 14 un-back-scanned).

#### Settled Item 5: Location Authority Is Hybrid, Central Mapping Is Unique Entrance

**Settlement**: Takes hybrid—

| Component | Settlement |
|---|---|
| Position entry (14 bytes in settled item 4) | **Demoted to hint**. Width and segment division unchanged; what changes is its status: reading it allows directly trying that placement, but it may be stale |
| Central mapping | **Unique entrance for dereference and release verdicts**. Relocation modifies only one entry here; "who references me" need not be known |
| State classification | Neither is item 4 on the authoritative state list: mapping can be reconstructed via unit self-describing scan, load-bearing surface remains units per D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) |

Three hard rules taking effect with settlement:

1. **Releases strictly proceed via mapping, never via hints**; after verifying through mapping to that entry, a read-disk verification against the unit checksum in the position entry is required before release. **When mismatch is verified, isolate that slot, publish succeeds normally**: logically released as usual (mapping entry removed), physical slot not returned to free pool, logged into isolation and reported via counter.
2. **Unit header or hint must carry block birth**—already satisfied: data unit class identity segment in D18 (Information Carried in a Block) settled item 7 already contains birth generation 8 bytes.
3. **Runtime observation point required for hint staleness multi-hop rate** (taking wrong branch is a performance cliff, and cliffs do not error, `.claude/rules/fs-design.md` hard requirement 4).

**How mount state reads mapping**: Version 1 mount state reads the central mapping entirely into memory when opening the pool (`central_mapping_entries` in `crates/singlefs-core/src/mounted_read.rs`); subsequent dereferences query this in-memory copy without issuing mapping reads; when central mapping grows to multiple levels, opening the pool reads the entire mapping tree into mount state, dereferences still only query memory without issuing device reads for mapping; write side checks mapping tree capacity acceptance on each publish based on current entry count plus maximum new entries from this publish, without relying on coincidental capacity numbers from other trees. Multi-level mapping tree has no entry count ceiling; structural ceiling is only the 1-byte level in header; calculates shape in planning step before any disk writes, rejecting if level cannot hold (`HeightBeyondTheLevelField`), without establishing separate entry ceiling (implemented in sprint 19, main agent settled). The reading "one dereference with stale hint issues 3 device reads" rides on this rule, measured when root doubled as leaf, remeasured by test cases when multi-level landed (milestone "overwrite, release, rollback and reuse" closeout table line 28). Allocation record tree is likewise read entirely into mount state upon mount; extent tree is not read entirely, traversing by position when opening files (D8 (Core Index Structure) settled item 14, user settled 2026-09-24).

**What to do when read-disk check in Hard Rule 1 fails to read or mismatches**: Before release, verify checksum by reading disk per position entry; if read fails or mismatches, reread once; only if both reads fail or both mismatch is it treated as mismatch (reread mismatch half settled by main agent 2026-09-25: transient bad read must not permanently isolate a unit; implemented in sprint 25). Different classes across the two reads (first fails to read, reread mismatches, or vice versa) treated as both mismatched, reporting the reread outcome (sprint 25 adopted, zero rounds of attack). If any replica mismatches, this unit's allocation records across every disk remain in "allocated", not flipped to released (even if the replica on the other disk matched, maintaining symmetry across disks)—persists across mounts, counted as allocated in acceptance without extra formulas. Position entries pointing to a disk outside the pool, or two entries pointing to the same disk, treated as corrupted mapping entry: rejected before any write, disk unmodified. Rebuilding previous version from disk (`rebuild_version` in `crates/singlefs-core/src/recovery.rs`) copies position entries of unreadable data units without reading contents; mount proceeds normally, erroring only when reading that file, except when unwitnessed (when the selected version is not witnessed by system configuration and its data units are unreadable, writable mount falls back to the witnessed version per D23 (Journal Role & Format) settled item 14 "when the selected root fails its walk and system configuration has not witnessed it, fall back to the witnessed version"; the fallback is judged by the union of C755 (recovery and two mount triggers fail differently on rollback), zero round wording, implemented in the implementation batch).

**On-disk mark for isolation**: the publish in which the writer isolates a unit sets the span segment of that allocation record on both disks to 0x4000 (the second-highest bit; the highest bit 0x8000 remains the released flag), leaves the allocation generation unchanged, and writes it out together with the other changes of that publish. The rollback publish that revives the unit clears the 0x4000 on both disks in the same publish. When an isolated record is released it is rewritten as released and the 0x4000 is cleared with it. The pool-level checker recognizes only the mark: an unreleased record that carries the mark on both disks is exempt from I-3.1 (Allocated Statistics Match), I-3.11 (Allocated Minus Defer Equals Latest Root Traversal) and I-3.15 (Unit Covered by an Unreleased Record Is Pointed to by the Mapping at This Slot), and its copies are not examined. A new criterion, "mark agrees with references", has three sentences: a record carrying the mark must be unreleased (0x4000 and 0x8000 are never set together); the unit covered by a record carrying the mark is not referenced by the latest root; for one unit the mark is present on every disk or on none; if any sentence fails, the verdict is red. An allocated record without the mark and without references is still judged a violation, with no fallback to judging by copies.

**Scope**: Simultaneously resolves the list half of "where multiple referrers live under sharing" in D21 (Boundary Between Authoritative and Derived States) and the three-way position authority choice in D26 (Background Compaction and Placement Reclamation) settled item 5, all other sites linking to settled item 5; originally also closed C34 (three-way clash on plain text mapping layer states), which became void on 2026-09-07 when D9 (Encryption) settled item 5 switched to "scrub only", leaving the other two sites unaffected. Mapping entry field table lives in settled item 6. Three known corners lack numbers: mapping tree key layout must satisfy rider in D8 (Core Index Structure) settled item 7 (deduplication window bounded by `T_dirty`); hint refresh policy only modeled as "refresh upon rewrite"; cache hit dimension has zero measurements, E109 (Runtime Costs of Three Location Authority Arms) explicitly disclaims answering. Three bootstrap exemptions in settled item 8. Read-disk verification handling details in settlement paragraph (three questions of C394 (release judgment does not verify unit checksum in mapping entry position item)); "reread once" rereads only once, without backoff or multiple retries. Isolation borrows the second-highest bit of the span segment: when readers that do not know the mark read 0x4001 and 0x4002, the allocation record tree reader refuses the whole tree (writable mount and recovery fail; a read-only mount does not read the allocation record tree, and E187 (Cost of Persisting Quarantine Marks) did not measure that path), the pool-level checker judges a violation, the GPU judge stops the walk, and the old reading accepts 0 of the 812 legal start slots of a leaf (E187 (Cost of Persisting Quarantine Marks) segment one arithmetic and code reading, artifact `research/results/e187-quarantine-mark-cost-segment-one-2026-10-21.out` lines 240–248).

**Basis**:

- E109 (Runtime Costs of Three Location Authority Arms): three arms measured across dimensions—unit migration cost is linear in referencing trees under pointer authority, independent under central mapping and hybrid; publish pays a fixed overhead independent of leaf count; read pays only 10%–40% extra. Absolute figures must be cited with sampling points: on primary sampling point referencing tree and mapping tree happen to share height, a degenerate point.
- E96 (Consistency of Hybrid Architecture): resolving placement via hint causes misplacement, forcing Hard Rule 1; its `hdr_key` arm references D18 (Information Carried in a Block) settled item 3 rather than settled item 7 field table, so "requires extra fields" format cost is non-existent.
- User settlement 2026-09-23: read disk to verify unit checksum in position entry before release; isolate slot upon mismatch, publish succeeds normally (`research/prompts/m2-placement-falsepositive-r1-main-verification.md` section 3 judging C394 (release judgment does not verify unit checksum in mapping entry position item): none of the three hard rules answered whether to verify), verbatim words in changelog.
- User settlement 2026-09-24: reading mapping entirely into mount state codified into clause per that day's implementation; multi-level mapping changed to whole-tree read by another user settlement the same day (splitting four trees in milestone 2, including mount reading multi-level mapping), landed in sprint 19 (C483 (Reading mapping and tree node rollback in mounted state both lack clauses) ①, settled during parallel line 2 without clause), verbatim words in changelog.
- User settlement 2026-09-24: C394 (release judgment does not verify unit checksum in mapping entry position item) three questions—read disk failure rereads once before treating as mismatch, tolerate unreadable data units during rebuild, allocation records for isolated replica remain allocated (verdict `research/prompts/m2-newq-r1-main-verification.md` section 3 N1 (Reread once upon read failure)–N3 (Mismatched entries remain in allocated), verbatim words in changelog); no experiments: candidate readings came from counter-proving leg prototype on replica (`research/prompts/m2-newq-r1-opus-model/`), not registered as experiment.
- User settlement 2026-09-24: multi-level mapping read entirely into mount state upon mount, write side adds mapping tree capacity admission (verdict `research/prompts/m2-treesplit-r1-main-verification.md` T3).
- User settlement 2026-09-25: when only one replica mismatches both disks retain allocated, position entries pointing outside pool codified as reject per that day's code (sprint 16 follow-up report `m2-writepath-implementer-report.md` section 6 Q2, Q3), verbatim words in changelog; no experiments: neither alters disk format, only settles policy.
- Main agent settlement 2026-09-25: `records/2026-09-24-里程碑二收尾调度.md` section 3 row on "transient bad read causes permanent unit isolation".
- Except when unwitnessed: root ring three-party round three verdict `research/prompts/m3-close-rootring-r3-main-verification.md` section 4, the row in the table "where the ones that entered no cell went" that revisits D19 (Block Pointer Structure & Width Budget) settled item 5 (defense Z5.2: data units on an unwitnessed R are unreadable, and writable mount changes from mounting R as usual to falling back to the previous version); user settlement 2026-10-20 (C755 (recovery and two mount triggers fail differently on rollback) "add all three sentences"), verbatim wording in history. No experiment: numbers measured on the three-party legs' replicas, no experiment number assigned.
- E187 (Cost of Persisting Quarantine Marks): the extra writes of placement A, the bit-flip classification table and the old readers' reading are measured; the fork sheet row of `research/prompts/m4-i315-quarantine-mark-cost-forks.md` is decidable.
- User settlement 2026-10-22: the "intact-on-disk isolation" shape takes placement A with a checker that recognizes only the mark (popup choice "落点甲，checker 只认记号 (Recommended)", original wording in history); the clause wording is the `span_bit` row of the arm table in Section 5 of the pre-run registration `research/prompts/e187-preregistration.md` and its paragraph "丙的 checker" (the checker for the mark-persisting option), variant `mark_only`.

**Owed**: C248 (bootstrap exemption of tree table units unwritten), C253 (contradictory kb claims on 66 048 fixed overhead), C133 (relocation forwarding information has no carrier), C279 (scan rebuild sees both old unit and slot after packing); C417 (hint refresh policy only modeled in one variant), C418 (Zero measurement of cache hit dimension across three location authority arms)—cells that respective basis experiments acknowledged inability to answer; C543 (rebuilding previous version only brings slice 0 of instance table).

#### Settled Item 6: Central Mapping Key Reuses Write Sequence Key by Class (Post-Tightening)

**Settlement**: Key is fixed-width, arranged segment by segment, definition and width matching `class_reuse_tagged` arm of E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) verbatim—

| Class | Key (fixed-width, segment by segment) | Width |
|---|---|---|
| Code 1 data unit | Class tag 1 + birth tree 8 + birth txg 8 + write sequence 10 (instance generation 4 + transaction number 6) | 27 |
| Code 2 / Code 3 | Class tag 1 + birth tree 8 + birth txg 8 + instance generation 4 + birth sequence number 4 | 25 |

Value follows settled items 4 / 5: position entry × w replicas (28 bytes when w = 2). Padding and total entry width belong to settled item 10.

**Four tightenings carried with it** (added during three rounds of debate per "record one loss / tighten only / declare where tightened", must still be respected after settlement):

- Class tag takes code in D18 (Information Carried in a Block) settled item 11 registry table for the named object; objects in D27 (Small-Data Packing Container) slots are constantly Code 1; upon packing and whole-container rewriting, segments of slot key take verbatim values from pre-packing unit header.
- Birth tree takes birth tree in class identity segment (Code 3 offset 43), birth txg takes birth generation (Code 3 offset 73; Code 1 and Code 2 likewise take birth generation), instance generation takes first 4 bytes of write sequence.
- D16 (Publish Semantics) settled item 5 closing sentence "one transaction writes at most one unit of user data" is read as covering all Code 1 units (preallocations, truncated tail units, punch hole ends, reflink copies on both sides count), preferably incorporated into that statement.
- Birth sequence number serves only as one segment of key, not for multi-version selection at fixed points—using it for selection falls into "sequence number for distinguishing stale replicas" rejected in D18 (Information Carried in a Block) settled item 1; D18 (Information Carried in a Block) settled item 1 owes a scope note following write sequence precedent.

**Scope**: Governs which segments comprise key and value and their widths, not padding or total entry width (settled item 10), classes contained (settled item 8), or birth sequence allocation (settled item 9). Lands in D15 (Format Freeze Policy) Layer 2. Uniqueness of Code 1 half relies on deduction that "Code 1 write sequence is already unique version identifier", built on transaction subdivision discipline in D16 (Publish Semantics) settled item 5 tail (paired with D23 (Journal Role & Format) settled item 7 one transaction number per record); that section closed review on 2026-09-13 with C123 (user settlement modified without review), subdivision discipline unchanged, deduction holds; remaining contradictions logged in C310 (Txn Split Discipline Conflicts With Record Count Measurement Basis). Recorded provisionally: chosen based on existing cost and performance numbers, reopenable upon new counter-evidence, not a proven unique solution.

**Basis**:

- E137 (Performance Disparity Across Mapping Key Forms): gap across three arms under fsync, background load, and random reads—key reusing write sequence by class leads in most fsync cells and all random read cells, losing in global random overwrites, file deletion, and single-file snapshot deletion; user selected it after reviewing table. ⚠️ In model, physical placement equals write order, the most favorable scenario for migration and snapshot deletion; shrinkage after aging unmeasured.
- E133 (Format Cost of Mapping Key Candidates): bytes paid in format by each key variant across two rounds of debate, giving pointer width, entry width, and fanout across seven candidates.
- E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms): arm chosen here is its `class_reuse_tagged`, reporting 1-byte class tag and two baselines of D27 (Small-Data Packing Container) slots line by line.
- E136 (Cost Rows for Mapping Key Forks): four missing cost numbers before user settlement—adding 4 bytes to Code 2 header drops zero fanout under three real header widths of D18 (Information Carried in a Block) settled item 14, mapping tree height differs by one layer only in a narrow pool scale range.
- Verdicts of three debate rounds and process in changelog; user settlement, verbatim words in changelog.

**Owed**: C288 (collateral changes from mapped key settlement did not land in other decisions), C289 (unwritten which field code 2 birth tree takes), C163 (identical write sequence across fixed point rounds defeats tie-breaking), C286 (whether remount after read-only takes a new instance generation), C287 (Switch adopts transactions of open checkpoint then crashes), C310 (Txn Split Discipline Conflicts With Record Count Measurement Basis).

#### Settled Item 7: Pointer Header Carries Birth (Tree, TXG), 8 Bytes Each

**Settlement**: In addition to MAC 16 + nonce 12 + algorithm type 1 + compression algorithm code 1 + compressed length 2 + extent offset 2, pointer header carries birth tree 8 and birth txg 8, header totaling **50**; pointer remains fixed-width (settled item 3). Compression 3 bytes includes compression algorithm code 1 byte (registry table in D9 (Encryption), 0 = no compression, version 1 writes 0) plus compressed length 2 bytes (version 1 writes 0, sufficient for [0, 32768) intra-unit length).

**Scope**: Settles which 16 bytes reside in pointer header, not total pointer width (settled by combining settled item 8 and settled item 11 to 88 / 86). Compression 3 bytes provides the format carrier for hard requirement in D9 (Encryption) settled item 13 item 2—requiring compression/encryption order to be explicitly defined in format, where previously no field carried "this ciphertext is compressed, original length X". ⚠️ Main agent recommended omitting (compression is optional computation, and "reserve bits across all volumes" principle governs only encryption); user chose retention via popup settlement, without three-way argument, overturnable. Four consumers read these two segments from pointer: D5 (Snapshot / Space Accounting Mechanism) O(1) judgment reads birth txg; deadlist entries recorded by version identity, birth tree taken from pointer; central mapping key; AAD format requires pointer to reserve slots for these fields, values independently verified (D9 (Encryption) settled item 16 explicitly forbids reading back from pointer). Alters bytes of new-pool file creation: yes—adds 3 bytes to every pointer.

**Basis**:

- E128 (Point Lookup Cost of Pointers with Birth (tree, txg)): accounting tree internal entries widened slower across four rounds, inode tree pair unstable across four rounds and unusable; measures only these 16 bytes, a lower bound on this cell's cost.
- One round of three-way argument, verdict in `research/prompts/d19-item7-r1-main-verification.md`: backward leg noted closing sentence of settled item 8 left ambiguity on "whether living in pointer or referencing entry", both interpretations defensible.
- User settlement ("16 bytes live in pointer"), verbatim words and disqualification of candidates B and C in changelog.

**Owed**: C295 (whether key segment in pointer counts as AAD lookup path), C237 (code 1 / code 2 pointers have no room for the birth (tree, txg)).

#### Settled Item 8: All Pointers Enter Mapping, Reference Side Carries Mapping Key Segments by Class

**Settlement**: Central mapping holds Code 1, Code 2, and Code 3 units; pointers to a unit carry mapping key segments for that class excluding class tag, class tag supplied by lookup path.

| Target Class | Pointer Carries (key segments from settled item 6) | Pointer Width |
|---|---|---|
| Code 1 | Birth tree 8 + birth txg 8 + write sequence 10 | Data pointer **88** |
| Code 2 / Code 3 | Birth tree 8 + birth txg 8 + instance generation 4 + birth sequence number 4 | Node pointer **86** |

**Three classes excluded from mapping (bootstrap exemption)**: Mapping tree's own nodes, tree table units, instance table units. Position entries in their parent pointers (mapping tree root, tree table pointer and instance table pointer in root record) are authoritative, not hints; settled item 5 "unique entrance" and Hard Rule 1 do not apply to these three; all three are pool-level singletons with exactly one referrer. Parent pointers of these three share 86 width with node pointers to Code 2 / Code 3: position entries 14 × 2 are authoritative, instance generation 4 + birth sequence number 4 populated from target unit header without querying mapping; parser determines pointer width by target unit class, not by whether target is an exempt class. For non-exempt units (Code 1 data units and Code 2 tree nodes—extent tree root, inode tree root, inode leaf container), unreadable position hints trigger fallback through mapping; exempt three classes (mapping tree root, tree table, root record chain) do not fall back.

**Scope**: Decides which classes mapping holds and which key segments pointers carry, invariant to pointer width (where birth (tree, txg) 16 bytes living in pointer header was settled by settled item 7). Holding Code 1 only loses on two correctness points: Code 2 / Code 3 blocks likewise enter deadlist / livelist, and D5 (Snapshot / Space Accounting Mechanism) settles deadlist entries recorded by version identity and released via mapping; without mapping they must be recorded by placement, causing release at stale placements after compaction or drive-loss reconstruction; migrating a Code 2 / Code 3 referenced by read-only snapshot requires rewriting snapshot tree ancestors, taking new birth violates reference interval while keeping old birth violates `birth` definition. All pointers entering mapping avoids both losses: deadlist / livelist recorded by mapping key, migration alters only one mapping entry, migrated unit remains byte-for-byte unchanged with birth unchanged. Costs: mapping blocks per unit during fsync and background exceed Code 1-only by 10%–20%+, random read misses also rise; column undercounts two items (insert/delete of derived tree Code 2 in mapping, extent nodes with stale hints reading via mapping). Parent node rewrites populate position hints with mapping placements (C417 (hint refresh policy only modeled in one variant) version 1 takes "refresh upon rewrite"). Checker and GPU judge treat position entries of non-exempt Code 1 / 2 / 3 as hints: traversal attempts hint first, reads mapping placement if both mismatch, records mapping placement in reference set.

**Basis**:

- E137 (Performance Disparity Across Mapping Key Forms): measured cost gaps between two configurations (Code 1 only / all pointers enter mapping) cell by cell, cost row of this cell taken directly from it.
- E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms): pointer width, entry width, and fanout across configurations provided by it; 88 / 86 derived from its two rows plus 3 compression bytes from settled item 7.
- E128 (Point Lookup Cost of Pointers with Birth (tree, txg)): point lookup cost omitted on settlement day and supplemented next day, recorded in cost row of this cell.
- Two rounds of three-way argument, verdicts in `research/prompts/d19-item8-r1-main-verification.md` and `research/prompts/d19-item8-r2-main-verification.md` (first round cloud legs reached opposite conclusions, user requested rerun, stances reversed in second round).
- User settlement, verbatim words and reasons why both remedy paths required modifying settled clauses in changelog.
- Verdict in `m2-lines234-code-r1-main-verification.md` 2026-09-23 on whether tree nodes fall back via mapping: counter-proving leg tested same corruption where data unit read back correct bytes via extra hop, whereas extent tree root made `open_pool_for_read` unmountable for entire pool.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` section 3 G3: write path, reconstruction, and checker refactored per "mapping is unique entrance" (Option B), parent rewrites write mapping placement, checker and GPU judge treat position entries as hints only. Section 7 user settlement "if inappropriate, illogical, or unnatural, refactor", verbatim words in changelog.

**Owed**: C293 (two items undercounted by E137 after all pointers enter mapping), C292 (The Reason Why the Code-2 Identity Segment Has Zero Consumers No Longer Holds), C248 (bootstrap exemption of tree table units unwritten) (merged from C280 (Central Mapping Has No Clause for Which Unit Classes It Contains)).

#### Settled Item 9: How Birth Sequence Numbers Are Allocated and in What Order

**Settlement**: Birth sequence numbers start from 0; increments by 1 for each Code 2 or Code 3 unit written within same tree (both classes share one counter); clears when scope shifts to next checkpoint; same node rewritten second time by fixed point within same checkpoint takes new number. Allocation sequence matches bump sequence of commit endogenous blocks from open cluster segment—ascending by tree ID, within tree leaves before root, key-ascending within same level, central mapping tree penultimate, tree table unit last.

**Scope**: Sole registration site for sequence itself is D3 (Space Allocation) settled item 10, no second copy stored here. Birth sequence number is a segment of central mapping key (settled item 6); key must be determined before unit is written, binding allocation to placement allocation sequence: single cursor in implementation, single sequence checked by checker. Serves only as segment of key, not for multi-version selection at fixed points (fourth tightening of settled item 6). User popup settlement, without three-way argument, overturnable; main agent recommended this tier. Alters bytes of new-pool file creation: yes—placements and birth sequence numbers of eleven commit endogenous blocks (default geometry; 13 blocks when ring is 1024 MiB) in new-pool file creation determined by it, per-block values in D3 (Space Allocation) settled item 10 ⑤ table.

**Basis**:

- No experiments: sequence and initial values have no quantifiable metric, cost of sharing rule between numbering and placement lives on D3 (Space Allocation) settled item 10 side; cell supported by deduction "key must be determined before unit is written" plus tightening arguments (same container receiving birth sequence 0 in two checkpoints buys collision-free runtime mapping) holding only under rewrite number change.
- User settlement, verbatim words in changelog.

**Owed**: C291 (birth sequence number allocation rule exists only in experiment source).

#### Settled Item 10: Mapping Key Strictly 27 Bytes, Entries Strictly 55 Bytes

**Settlement**: Central mapping key is **strictly 27 bytes across the board**—Code 1 (class tag 1, birth tree 8, birth txg 8, write sequence 10) is naturally 27; Code 2 / Code 3 (class tag 1, birth tree 8, birth txg 8, instance generation 4, birth sequence number 4) is 25 bytes, **zero-padded to 27 at tail**. Central mapping entries are thus **strictly 55 bytes across the board** (key 27 + value 28, value = position entry 14 × 2). Key range in mapping tree node headers is likewise stored at 27.

**Scope**: Two related sites return to fixed-width with it—sparse side table entries in D5 (Snapshot / Space Accounting Mechanism) settled item 6 fixed to 31 (mapping key 27 + count 4), livelist entries in D6 (Snapshot Implementation Model) settled item 2 fixed to 36 (head tree ID 8 + mapping key 27 + event type 1). Varying length by class would violate two universal rules in D8 (Core Index Structure) settled item 11: "fixed-width entries arranged contiguously" and "declared length = entry count × entry width"—entry width in Code 2 header has only one value, so two widths require a sentinel, eliminating intra-node fixed-width binary search and forcing sequential scan from start to resolve a mapping node during scan. Cost is 2 extra bytes per Code 2 / Code 3 entry, 10 extra bytes in new-pool file creation. User popup settlement, without three-way argument, overturnable; main agent recommended this tier. Alters bytes of new-pool file creation: yes.

**Basis**:

- E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width): measured single-width arm; "two widths 55 / 53, self-delimited by first key byte" was an unmeasured fourth form in that round, taking it as settlement would cite a number with no artifact.
- User settlement, verbatim words in changelog.

**Owed**: None (C321 (code 2 header entry width is single value while mapping tree entries have two widths by class) repaid 2026-09-14 with user settlement returning mapping key to fixed-width 27, see owed check repaid table).

#### Settled Item 11: Pointer Offset Table, Position Entry Ordering, Where Mapping Tree Root Resides

**Settlement**: Pointer segments arranged from offset 0—

| Segment | Offset | Width |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| Algorithm type | 28 | 1 |
| Compression algorithm code | 29 | 1 |
| Compressed length | 30 | 2 |
| Extent offset from origin | 32 | 2 |
| Birth tree | 34 | 8 |
| Birth txg | 42 | 8 |
| Position entries × 2, ascending by device identity | 50 | 14 × 2 |
| Key tail: Code 1 write sequence 10; Code 2 / Code 3 instance generation 4 + birth sequence number 4 | 78 | 10 / 8 |

=> Pointers to Code 1 are **88**, pointers to Code 2 / Code 3 are **86**. Position entries sorted ascending by device identity, established as I-2.5 (position entries in ascending device identity). **Central mapping tree root lives in root record**: root record carries an 86-byte central mapping tree root pointer, not entering tree table; root record totals **466** bytes. Tree table version 1 has **12** entries.

**Scope**: Settles only ordering—set of segments and widths settled in settled items 3 / 4 / 7 / 8. Compression 3 bytes immediately follows algorithm type, belonging to the group "how this ciphertext was produced"; placed before extent offset to keep four encryption and compression fields contiguous. Sole authoritative location for root record total is field table of D22 (Synthesizing Unit Atomicity) settled item 7; tree table version 1 count follows D5 (Snapshot / Space Accounting Mechanism) settled item 11 adding deadlist. User popup settlement, without three-way argument, overturnable. Alters bytes of new-pool file creation: yes—segment offsets and widths of every pointer, and root record total.

**Basis**:

- No experiments: offset table has no second choice, segments and widths already settled, cell only orders them with no quantifiable metric. Ascending device identity establishes deterministic order for two position entries, enabling invariant.
- User settlement, verbatim words in changelog.

**Owed**: None (C304 (Instance table chain pointer record cannot accommodate 83-wide pointer) repaid 2026-10-11, see owed check repaid table).

#### Settled Item 12: Code 2 / Code 3 Mapping Entries Written Day-1, Fixed-Point Writes Once per Checkpoint

**Settlement**: Maintain day-1 write-out (6 entries in new-pool file creation). Fixed point codified: each Code 2 node and Code 3 container written only once per checkpoint by an instance—one-shot COW at end of publish; version 1 forbids concurrent insertion with COW (implementation permitted by D3 (Space Allocation) settled item 5 not adopted in version 1).

**Scope**: ⚠️ When concurrent insertion with COW is permitted later, D8 (Core Index Structure) settled item 13 must still be respected: same key lands in leaf only once per publish based on final state. Failing checks counted per unit, not per fixed-point round. Cell settles whether to write and write frequency, not key form (settled item 6) or entry width (settled item 10). When settled, repository lacked release and reuse experiments, leaving other path as "revisit once release experiments exist", an uncodified trigger condition.

**Basis**:

- E81 (Fixed Point of Commits): origin of reading "dirty nodes COW only once per publish", and basis for fixed point convergence under this reading; applying it to Code 2 and Code 3 eliminates multi-version same-key loss within same checkpoint.
- User settlement, verbatim words in changelog.

**Owed**: C352 (settlement of writing mapping entries day-1 lacks awakeners for reopening conditions), C163 (identical write sequence across fixed point rounds defeats tie-breaking).

#### Settled Item 13: Block Pointer Split into Two Sections

**Settlement**: **Block pointer is split into two sections: one "header" following the logical extent, and N "position entries" following physical placements.**
Logical properties (MAC, nonce, algorithm type, extent offset from origin) in header, one copy per logical extent;
Physical properties (placement, and ciphertext checksum for unkeyed corruption detection) in position entries, one copy per physical position.

**Scope**: Splitting demotes "total pointer width" from a hard constraint to a budget, rewriting D4 (Checksum Location) into a two-section form. Settles only which class of properties lives in which section, not specific fields (settled items 4 / 7) or order (settled item 11); birth tree and birth txg later added to header (settled item 7) remain logical properties. "Moving out of pointer" does not equal an extra I/O: D4 (Checksum Location) requires parent node to store child checksum, constraining which node it lives in, not which struct.

**Basis**:

- D2 (RAID Stripe Policy) settled "multi-replica writes identical ciphertext N times" => all replicas must share same nonce; storing per pointer duplicates same value N times, and writing one replica wrong yields two incompatible decryption results. Testable form is I-6.7 (replicas share a nonce).
- Production implementations exhibit same shape (inspected live 2026-08-28, not verified in this project): bcachefs `struct bch_extent_ptr` is exactly 64 bits (`offset:44, dev:8, gen:8` plus four flags), checksum / nonce / compression info moved to independent variable-length entries (`bch_extent_crc32/64/128`) within same bkey, one crc entry applying contiguously to all following ptr entries; nonce itself unpersisted, `extent_nonce()` in `fs/bcachefs/checksum.h` computed live from `bversion` (96 bits, in bkey header) plus small per-crc offset (`crc64.nonce:10` / `crc128.nonce:13`).
- No experiments: two-section split is a format architectural trade-off, underpinned by settled rule (replicas share nonce) and production shapes, with no quantifiable metric.

**Owed**: None.

#### Settled Item 14: Position Entries Listed Individually, Cannot Be Computed from Single Origin via Geometric Formula

**Settlement**: N position entries listed individually, each carrying device identity; not deduced from single origin plus size for remaining columns.

**Scope**: Governs expression of position entries, not constituent segments (settled item 4). Prerequisite is D2 (RAID Stripe Policy) settled item 1 defining "variable width" as **per-occurrence variable**—listing physical placements individually with device identities and avoiding chunk mapping tables is the exact premise required by this deduction. Isomorphic to D2 (RAID Stripe Policy) settled item 1: buys "no pointer to wrong disk", not "failure domain independence".

**Basis**:

- ZFS can deduce all RAID-Z column positions from "one DVA + asize" (`BP_GET_ASIZE` summing asize of 3 DVAs), predicated on top-level vdev child count being **fixed at pool creation** (inspected live, not verified in this project); D2 (RAID Stripe Policy) settled item 1 negated this premise.
- No experiments: deducibility is directly determined by stripe width granularity, a deduction with no quantifiable metric.

**Owed**: None.

#### Settled Item 15: Value Range of Extent Offset from Origin and Version 1 Value

**Settlement**: Value range of extent offset from origin is capped by **maximum extent size**, not file size; range in this project is [0, 32768), **15 bits sufficient**. Version 1 field is constantly 0—splitting is a reserved feature, disabled in version 1; bits reserved per settled item 3 "fixed-width reserved slots", justified by policy rather than usage.

**Scope**: Addresses value range; width of 2 bytes lives in settled item 11 offset table. D9 (Encryption) settled item 12 item 2 estimated 32–64 bits based on file size, an overestimate. Basis for narrower range in this project is D4 (Checksum Location) settled item 7: extent on disk strictly = unit = 32768.

**Basis**:

- Production inspection (inspected live 2026-08-28 in `fs/bcachefs/extents_format.h`, not verified in this project): `crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`, matching `CRC128_SIZE_MAX (1U << 13)` => value range capped by maximum extent size.
- No experiments: value range is arithmetic dividing two settled clauses, with no quantifiable metric; version 1 disablement settled by D4 (Checksum Location) settled item 7.

**Owed**: None.

### Chain Reactions with Other Decisions

| Decision | Chain point |
|---|---|
| D4 (Checksum Location) | Settled item 13 demotes "total pointer width" from constraint to budget, rewriting D4 (Checksum Location) into two-section form; logical extent strictly occupying 32768 given by D4 (Checksum Location) settled item 7, relied upon by settled items 3 and 15 |
| D2 (RAID Stripe Policy) | Prerequisite for settled items 1 and 14 is variable stripe width, settled as per-occurrence variable by D2 (RAID Stripe Policy) settled item 1 |
| D9 (Encryption) | Nonce carrier settled half by settled item 13 (in header, one per logical extent); registry tables for algorithm type and compression live in D9 (Encryption), expected AAD cannot be read back from pointer (D9 (Encryption) settled item 16) |
| D12 (Target Media) | Mount comparison table in D12 (Target Media) settled item 4 rejects mount on device identity mismatch, based on position entries carrying dev in settled item 1. ⚠️ Criterion requires "whether **address arithmetic** consuming it carries device identity", btrfs address arithmetic includes device identity in chunk tree; criterion holds, leap "carried => must be in pointer" falls |
| D15 (Format Freeze Policy) | Pointer field layout is Layer 2 of D15 (Format Freeze Policy) settled item 7; central mapping entry field table in settled item 6 also lands in Layer 2 (explicitly cited in basis of D15 (Format Freeze Policy) settled item 1) |
| D8 (Core Index Structure) | Pointer width enters internal entries and tree table entries, several widths in D8 (Core Index Structure) settled items 6 / 8 adjust with settled items 7 / 8; viability of wide pointers couples with 16 KiB nodes in D8 (Core Index Structure) settled item 2 |

## Historical Versions

Historical entries for D19 (Block Pointer Structure & Width Budget) are centralized in [decisions-history.md](../decisions-history.md).
