## E210 Four reader rules for rewriting the capacity table when a disk grows —— apparatus written, artifacts not run: the independently hand-written counting model, its unit tests and the mutation table (23 rows) are written and two replay rows are in `replay.sh`; the build, the unit tests, the mutation table and both artifact segments have not been run once, so there are no readings

[中文](210-盘变大改容量表的读者规则四臂.md) · **English** · [日本語](210-ディスク拡大時の容量表書き換えにおける読者規則四案.ja.md)

Pre-run registration `research/prompts/e210-preregistration.md`, fork sheet `research/prompts/m4-capacity-grow-r1-forks.md` with three rows. The question: the system configuration of each disk in the pool (two rotating slots per disk) carries a per-disk capacity table; the writable mount after a disk grows writes the new table into every slot of every disk, and power may fail halfway; after the power loss the user may leave things alone, shrink the disk back, grow it again, or swap a disk back to an earlier whole-disk image. When the tables in the slots on the disks are not all the same, which table does the reader take as the table in force, which boards does it refuse for the whole pool, and does the read-only mount name the disks it refuses: these are the four candidate reader rules of row 1 (Arms A, B, C and D, plus a baseline arm); row 2 counts how many system configuration slot writes and how many barriers one writable mount that grows the disk performs under each rule; row 3 counts the whole slots still usable after the growth when it crosses a root-level boundary of the allocation record tree, under the two ways of recording nothing at all and recording up to the boundary.

**This page records what the apparatus looks like on disk. The build, the unit tests, the mutation table and the artifacts have not been run once, so there are no readings, no row of the fork sheet is judged, and no decision is overturned or established.**

### Current state

- Apparatus: `scripts/experiment-apparatus/e210_capacity_grow_reader_rules.rs` (bin `e210-capacity-grow-reader-rules`): an independently hand-written counting model that shares no code with `crates/` and is not a checked-in apparatus; the command-line argument selects one artifact segment: `write-counts-and-root-level` (rows 2 and 3) and `reader-rules-on-histories` (row 1).
- Mutation table: `scripts/mutation/tables/e210_capacity_grow_reader_rules.tsv`, 23 rows (section nine of the registration); the equivalent mutation (dropping "the tables in the valid slots are not all the same" from the abandon branch of Arm D) is not in the table and is kept on record by the unit test `abandoning_is_never_reachable_when_every_table_is_the_same`.
- Replay registration: two E210 rows in `scripts/experiment-apparatus/replay.sh`, criterion `exact`; the planned names of the retained artifacts are `e210-capacity-grow-reader-rules-write-counts-and-root-level-2026-11-18.out` and `e210-capacity-grow-reader-rules-reader-rules-on-histories-2026-11-18.out`, neither of which exists yet.
- Unit tests: 36 in the apparatus, plus 2 for the shared thread default brought in by `include!`; none has been run.
- Not done this time: the third segment of section five of the registration (geometry sensitivity and the self-test of discriminative power, that is, stability) is not dispatched; the invariance check over the four combinations of generation parity and over another set of order-preserving integers is a voiding clause and is computed in the row 1 artifact segment.
- The hand-derived outcomes of items 8 to 13 of section four of the registration each take one `hand_derivation` line in the row 1 artifact, beside the readings; a hand derivation is not a conclusion.

### Measurement basis

1. The pool has two disks (device identities 0 and 1) with exactly two slots each; disk sizes and capacity tables are both counted in whole slots (16 KiB per slot). The reader only recognizes valid slots (slot generation, table, intent marker); a torn slot cannot be read.
2. A history is a tuple over five dimensions: family, shape, footprint, power-loss point, user action; each history has two mounts, writable and read-only. The power-loss points are the enumeration after each arm's write order is cut into persistence segments by the pool barriers: each system configuration slot write takes one of three states (not persisted, persisted, neither old nor new readable), plus one end-of-use point. The four-write order (Arms A, B and D and the baseline arm) has 18 raw states and the six-write order (Arm C) has 27; the history tuples number 1487 writable for the four-write order and 2207 for the six-write order, with the same number read-only. These numbers are computed by the registration and pinned by the apparatus unit tests; they are not readings.
3. Five arms: Arm A (take the entry-wise lowest when the tables in the valid slots form a chain, refuse the whole pool otherwise), Arm B (take the entry-wise highest), Arm C (write one round of intent-marked slots first; when a slot on the disks carries a marker, take the old table), Arm D (read with the lowest, compare against the probe with the highest, and a growth that was not fully written may be abandoned by writing the old table back), and the baseline arm (accept only when all tables are equal).
4. Three control readers, each run once on the history set of every arm: allow everything (must leak), reject every writable mount (must misrefuse), and the class judge (the answer is given by construction; its leaks and avoidable misrefusals must be 0).
5. Two predicates: a leak (some unit's placement goes past the end of some disk as it is now, and the writable mount is made, or the read-only mount is made without naming that disk); a misrefusal (the writable mount is refused while no placement goes out of bounds). Misrefusal is reported under two readings side by side: the literal reading and the avoidable reading (no history in that observation class goes out of bounds); the literal reading is expected to hold on every arm and so cannot tell the arms apart; the misrefusal half of fork sheet row 1 is flipped by the avoidable reading, as the main agent has accepted.
6. Row 2: the write-order generator counts slot writes and the pool barriers that have writes before them, with no hand-filled constants; the registration expects 6 and 3 for Arm C and 4 and 2 for the rest, an expectation and not a reading.
7. Row 3: the level arithmetic is written again, not copied from `crates/`, and checked in a differential against the four level readings pinned by the `crates/` unit tests; six sampling points: below the boundary, across one boundary and across two boundaries, plus a synthetic control point and the two one-over-the-boundary points that the mutations need. How the second way ("record up to the boundary") spreads over two disks is the reading of unsettled item 2 in section one of the registration.
8. Determinism: the model has no random source, clock or concurrency; the same apparatus run twice gives byte-identical standard output, and the strength of the evidence rests on pinned numbers and mutations, not on the number of rounds.

### Rerun

```bash
bash scripts/experiment-apparatus/replay.sh E210
```

Criterion `exact`; until the retained artifacts exist these two rows report the retained artifact missing.

### Paths and conclusions register

The level readings of row 3 are the same quantity calculated along two paths that share no code and then compared:

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Level arithmetic written separately in the apparatus | The smallest root level R ≥ 1 such that the sum over disks of ⌈whole slots ÷ slots covered by one cell of level R−1⌉ does not exceed the fan-out 169; slots covered by one cell = 812 × 169 to the power R−1; the inputs are the six sampling points of the registration and the four inputs of the differential, not a sweep over the whole domain | `scripts/experiment-apparatus/e210_capacity_grow_reader_rules.rs:1735 root_level_of` | Leaf coverage 812 and fan-out 169 (copied from two constants in `crates/singlefs-format/src/lib.rs`), and the level definition itself (item 7 of section three of the registration, read from `allocation_record_tree.rs`); if these are wrong, both paths are wrong together |
| 2 | Geometry evaluation in `crates/` | `AllocationRecordTreeGeometry::of_devices` takes the absolute slot count of each disk and finds the same smallest root level; this page only reads the four readings its unit tests assert and does not call it | `crates/singlefs-core/src/allocation_record_tree.rs:97 of_devices` | The same two constants and level definition as above |

The cross-path comparison assertion is pinned in the apparatus unit test `root_level_arithmetic_agrees_with_the_four_readings_pinned_in_the_crates`: the four level readings asserted by three `crates/` unit tests are copied into the apparatus constant table `CRATES_ROOT_LEVEL_READINGS` and compared with what the apparatus calculates; the outputs of the two paths are not joined and compared directly, and the main agent confirms with a separate task that those three `crates/` tests still give these values today.

### What it cannot answer

- The capacity table has no implementation in `crates/` yet: on the reader-rule side there is no implementation to compare against; only the write order, the barriers, the generation numbers and the root-level arithmetic have `crates/` implementations to compare against.
- It only counts "made or refused, which disks are named", does not simulate writes after the mount, and does not answer the details of how a later publish writes units into the newly added disk area.
- The reader rules do not look at slot numbers or generation numbers, only at the order of sizes; disk sizes and table entries take small integers, and real slot counts are used only in the group of quantities of row 3.
- What table the marker slots of Arm C carry, how several slots are taken when "take the old table", how Arm D recognizes "not fully written", and how the second way of row 3 spreads over two disks are the readings written in unsettled items 2 and 3 of section one of the registration; anyone who thinks an arm should carry a stricter guard is describing another arm, which needs another name and another registration.
- The "after the growth was written" image is implemented with family D (an earlier growth was already written); another point in time (for example the other timeline of a cloned pool) needs another history family.
- Stability (geometry sensitivity and the self-test of discriminative power) is not done this time, so row 1 cannot write "holds at which sampling points".

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D12 (target media) settled item 4 | Preparation | 2026-11-18 not affected: the artifacts have not been run once and there are no readings; this page only prepares the clause "refuse a disk that shrinks, allow and record one that grows" by measuring the readings of each reader rule on the power-loss histories of growing a disk and rewriting the table; the evidence of this item does not cite E210 (Four reader rules for rewriting the capacity table when a disk grows), and `.claude/decision-links-pending` currently has no entries |
| D22 (synthesizing unit atomicity) settled item 8 | Preparation | 2026-11-18 not affected: the artifacts have not been run once; the write order, barriers and power-loss points of the apparatus are built on the two rounds of number acquisition writes of this item and do not judge it; the evidence does not cite this experiment |
| D22 (synthesizing unit atomicity) settled item 16 | Preparation | 2026-11-18 not affected: the artifacts have not been run once; the generation numbers and slot placement of the apparatus follow the slot rotation rule of this item and do not judge it; the evidence does not cite this experiment |
| D13 (verification route) settled item 4 | Does not affect | 2026-11-18 not affected: it only borrows its persistence-segment and third-state basis to enumerate the power-loss points and judges no clause of it |
| D2 (RAID stripe policy) settled item 3 | Does not affect | 2026-11-18 not affected: the registration lists it as one of the clauses under test; this experiment measures reader rules and write counts and does not judge its direction |

## Historical versions

None (initial output).
