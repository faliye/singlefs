# Milestone "Directory and POSIX Surface": Work Items, Acceptance, and Ordering

[中文](11-eleventh-txn.md) · **English** · [日本語](11-eleventh-txn.ja.md)

**Measurement basis**: Planning document, filed ahead of time on 2026-10-12 based on user settlement (verbatim: "POSIX goes to milestone 9, FUSE goes to milestone 10"; on the same day requested: "Write it in /home/fy5090/code/singlefs/.claude/kb/milestone write it ahead of time"), this is a draft. **It does not require correctness**: revisit and revise before starting work on each item. Directories were decided by the user to be implemented after Milestone 4 (recorded verbatim in "Not in exits" of [04-fourth-txn.md](04-fourth-txn.md)), landing in this document. The precondition is that all exits of Milestones 3 through 10 are achieved (Milestone 10 is [10-tenth-txn.md](10-tenth-txn.md)); if any item is missing when closing the previous milestones, this document is reordered based on what is missing.

**Reading guide**: Three boxes per item—"Existing in repo" lists which clauses, owed checks, and code this item is backed by in the repository (checked on 2026-10-12, with sources); "Work items" is a table: number, description, backing clauses and owed checks, acceptance (each proven to turn red first), dependencies; "Must be settled before starting work" lists unsettled questions that must go through three-party evaluation or consult the user before starting work. Anything marked "inferred" is an unmeasured judgment by the main agent. Work item numbering is formatted as "item·point" (I·1, II·3), used only within this document. Owed check numbers were all open on the day of filing; re-verify each before starting work.

**How many new transaction types**: Creating directories, deleting directories, renaming, and hard links are each separate new transactions (`.claude/rules/fs-design.md` "start from transactions, not from features"), traversing the same transaction layer and the same intent mechanism without opening an alternative publish path; newly added commit steps enter the shared `CommitStep` enum.

## Exits (One Settled, Remainder Unsettled)

The user settled one exit on 2026-10-22:

| # | Exit | Belongs to which item |
|---|---|---|
| 1 | Crash full run: finish what is new, then catch up on the backlog, at least 10 billion states per milestone (settled by user on 2026-10-22). Run in two stages (recording stage on CPU, judging stage on GPU, all observers attached), without narrowing the enumeration domain, all 0 red. ① First run the crash streams new in this milestone and the blocks whose unit key changed; ② then catch up from the backlog registry (set up in Milestone 4), at least 5 billion states, in registry order, resuming where the previous run stopped. When new and backlog together exceed 10 billion states, this milestone runs at least 10 billion states and the part of the new work that does not fit goes to the end of the backlog registry; when together they come to less than 10 billion, run all of it. On red, pin it back into the quick tier per the "A red in a full run is pinned back into the quick tier" section of `.claude/rules/test-script-discipline.md`, fix it, then rerun. | Crash full run (cross-milestone) |

Exits and to what layer the POSIX surface is implemented are left to user decision in Stage 0. Candidate exits inferred by the main agent (inferred, unmeasured):

| # | Candidate exit | Belongs to which item |
|---|---|---|
| 1 | Decisions regarding directory structure established and settled (which tree directory entries live in, keys, hard link counts, rename atomicity), on-disk format registered in `.claude/kb/layout/` with the same milestone number (filename starting with `11-`) | I |
| 2 | Settled POSIX operation set lands in implementation, RefFS expanded to identical set, differential testing against model matches item by item | II |
| 3 | One crash stream per new transaction type, achieving zero red across the board | III |

## Grouping and Ordering

| Stage | Description | Rationale for ordering | Status |
|---|---|---|---|
| 0 Handover | Check operations currently supported in `crates/`; settle scope and exits of POSIX surface | Number of decisions to establish can only be known after scope is settled | Not started |
| 1 Establish decisions | I·1 → I·2 | Establish decisions before writing code | Not started |
| 2 Implementation | II·1 ∥ II·2, then II·3 | RefFS and implementation are mutually independent (RefFS does not import implementation); differential testing after both sides exist | Not started |
| 3 Coverage | III·1 | Depends on 2 | Not started |

**Estimate**: Not estimated. Record `records/2026-10-11-里程碑五打包与决策进度列.md` Section 3 inferred 3–4 weeks (inferred, unmeasured).

## I. Decisions on Directory Structure

- **Existing in repo**: No decision governing directory structure exists repo-wide. Related settled clauses: settled item 3 of D8 (Core Index Structure) ("key = `(locality_id, inode, offset)`; `locality_id` inherited from parent directory upon creation, deliberately not updated upon rename"); settled item 5 of D8 (Core Index Structure) accepts tombstones, operations potentially modifying unbounded extent counts write intent first then advance in batches; settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) implements no allocator hints in first version (three reopening conditions); settled item 3 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) has inlining threshold 0. Owed checks: C109 (which trees' key deaths write tombstones), C527 (rename overwrite borrowing wrong half of deletion rules), C2 (unbounded operation made into single transaction) (owed check for deleting large directories still open), C115 (overflow path of inode attributes lacks placement).
- **Work items**:

| # | Description | Backing clauses and owed checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Establish decisions: which tree directory entries live in, how keys are encoded, where hard link counts live, atomicity of rename (including target overwrite), how large directory deletions are batched | Settled items 3 and 5 of D8 (Core Index Structure); C109 (which trees' key deaths write tombstones), C527 (rename overwrite borrowing wrong half of deletion rules), C2 (unbounded operation made into single transaction); C778 (No clause on how partial deletion without changing size is recorded) (user assigned to Milestone 11 on 2026-10-12, explicitly named on 2026-10-19, see registry row for remaining owed check) | Three-party verdict filed in `research/prompts/`, settlements written back to kb; decisions require experiments, at least one counting experiment | Stage 0 |
| I·2 | Establish decisions: placement of attributes and extended attributes (timestamps, permissions, xattr overflow) | C115 (overflow path of inode attributes lacks placement) | Same as I·1 | Stage 0 |

- **Owed checks moved from Milestone 3** (2026-10-18): C778 (No clause on how partial deletion without changing size is recorded) belongs to this milestone (user assigned to Directory and POSIX surface on 2026-10-12, verbatim: "assign all to milestone 9", directory and POSIX surface was numbered 9 at the time and subsequently deferred to 11): recording of punch (partial deletion without changing size), not done in truncation section of Milestone 4 ([04-fourth-txn.md](04-fourth-txn.md) "VI. Truncation").
- **Must be settled before starting work**: Scope of POSIX surface (which among directories, rename, hard links, symlinks, xattr, permissions, timestamps enter this document); whether reopening conditions for settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) are triggered by directories.

## II. Implementation and Differential Testing

- **Existing in repo**: Operation enum of RefFS contains only five types: creating first file, overwrite, explicit publish, fsync, rollback (`enum Operation` in `crates/singlefs-reference-filesystem/src/lib.rs`); ideal model in `crates/singlefs-harness/src/model.rs`.
- **Work items**:

| # | Description | Backing clauses and owed checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | Implementation: directory tree and directory entries, path resolution, attributes settled in I·2 | Settlements of I·1 and I·2; C677 (Write time lacks test cases across three write path cells) (moved from Milestone 3 on 2026-10-19, attribution inferred, verify before starting work, see registry row for remaining owed check) | Each new test proven to turn red first; new invariant enters pool-level checker, one bad image each | I |
| II·2 | RefFS expanded to identical operation set, dependencies remain `std` only | `.claude/rules/verification.md` "Definitions and Names" row for RefFS | Cell checker-implementation-disjoint in gate checker-independence-and-sync remains green | I |
| II·3 | Differential testing against model expanded to directory operations | `.claude/rules/verification.md` "functional correctness relies on differential testing against model"; C445 (sampling points of model differential do not grow with write path) (moved from Milestone 3 on 2026-10-19, attribution inferred, verify before starting work, see registry row for remaining owed check) | Random history includes directory operations, differential testing matches item by item; mutation omitting synchronization of an operation turns red | II·1, II·2 |

## III. Crash Coverage

| # | Description | Backing clauses and owed checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | One crash stream per new transaction type (create, delete, rename, hard link), registered in `crash-case:` rows of `.claude/gate.d/stage-inputs.tsv` and `.claude/gate.d/crash-point-replay-items.tsv` | Settled items 9 and 15 of D13 (Verification Route) | Zero red across the board; mutation causing both or neither side to be visible upon mid-rename crash turns red | II |

**Not in exits**: FUSE mount (Milestone 12 [12-twelfth-txn.md](12-twelfth-txn.md)), encryption AAD on directory entries (Milestone 15), cross-file packing and inlining on write path (settled item 3 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) judged negative).
