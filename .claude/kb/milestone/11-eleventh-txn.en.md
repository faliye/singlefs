# Milestone "Directory and POSIX Surface": Work Items, Acceptance, and Ordering

[中文](11-eleventh-txn.md) · **English** · [日本語](11-eleventh-txn.ja.md)

**Measurement basis**: Planning document, filed ahead of time on 2026-10-12 based on user settlement (verbatim: "POSIX goes to milestone 9, FUSE goes to milestone 10"; on the same day requested: "Write it in /home/fy5090/code/singlefs/.claude/kb/milestone write it ahead of time"), this is a draft. **It does not require correctness**: revisit and revise before starting work on each item. Directories were decided by the user to be implemented after Milestone 4 (recorded verbatim in "Not in exits" of [04-fourth-txn.md](04-fourth-txn.md)), landing in this document. The precondition is that all exits of Milestones 3 through 10 are achieved (Milestone 10 is [10-tenth-txn.md](10-tenth-txn.md)); if any item is missing when closing the previous milestones, this document is reordered based on what is missing.

**Reading guide**: Three boxes per item—"Existing in repo" lists which clauses, owed checks, and code this item is backed by in the repository (checked on 2026-10-12, with sources); "Work items" is a table: number, description, backing clauses and owed checks, acceptance (each proven to turn red first), dependencies; "Must be settled before starting work" lists unsettled questions that must go through three-party evaluation or consult the user before starting work. Anything marked "inferred" is an unmeasured judgment by the main agent. Work item numbering is formatted as "item·point" (I·1, II·3), used only within this document. Owed check numbers were all open on the day of filing; re-verify each before starting work.

**How many new transaction types**: Creating directories, deleting directories, renaming, and hard links are each separate new transactions (`.claude/rules/fs-design.md` "start from transactions, not from features"), traversing the same transaction layer and the same intent mechanism without opening an alternative publish path; newly added commit steps enter the shared `CommitStep` enum.

## Exits (One Settled, Remainder Unsettled)

The user settled one exit on 2026-10-22:

| # | Exit | Belongs to which item |
|---|---|---|
| 1 | Unscoped run of crash-point replay and crash amplification: finish judging what is new and keep judging what has never been judged, at least 10 billion states per milestone (settled by user on 2026-10-22). Run in two stages (recording stage on CPU, judging stage on GPU, all observers attached), without narrowing the enumeration domain, all 0 red. The gate selects the blocks to judge by unit key itself: ① the crash streams new in this milestone and the blocks whose unit key changed; ② blocks never judged under the current key, at least 5 billion states. When the two together exceed 10 billion states, this milestone judges at least 10 billion states; when together they come to less than 10 billion, judge all of it. The state count is the states actually judged by the unscoped runs in this milestone. On red, pin it into a case per the "A red from the gate is pinned into a case that can be rejudged on its own" section of `.claude/rules/test-script-discipline.md`, fix it, then rerun. | Crash-point replay and crash amplification (cross-milestone) |

Exits and to what layer the POSIX surface is implemented are left to user decision in Stage 0. Candidate exits inferred by the main agent (inferred, unmeasured):

| # | Candidate exit | Belongs to which item |
|---|---|---|
| 1 | Decisions regarding directory structure established and settled (which tree directory entries live in, keys, hard link counts, rename atomicity), on-disk format registered in `.claude/kb/layout/` with the same milestone number (filename starting with `11-`) | I |
| 2 | Settled POSIX operation set lands in implementation, RefFS expanded to identical set, differential testing against model matches item by item | II |
| 3 | One crash stream per new transaction type, achieving zero red across the board | III |
| 4 | Interaction with features landed earlier each has one test case and one must-red world: snapshots, GC, truncation, packing, parallel commit, multiple files, and the transactions added by each milestone ordered before this file (settled by the user on 2026-11-16; the shape follows Milestone 5 Exit 5) | III |

## Grouping and Ordering

| Stage | Description | Rationale for ordering | Status |
|---|---|---|---|
| 0 Handover | Check operations currently supported in `crates/`; settle scope and exits of POSIX surface | Number of decisions to establish can only be known after scope is settled | Not started |
| 1 Establish decisions | I·1 → I·2 | Establish decisions before writing code | Not started |
| 2 Implementation | II·1 ∥ II·2, then II·3 | RefFS and implementation are mutually independent (RefFS does not import implementation); differential testing after both sides exist | Not started |
| 3 Coverage | III·1 | Depends on 2 | Not started |

**Addendum (settled by the user on 2026-11-16)**: the design rounds of I·1 and I·2 move up to run alongside Milestone 5 X·1, by three-way argument only and without code; the implementation stays in this file.

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

- **Existing in repo**: Operation enum of RefFS contains nine types: creating first file, overwrite, explicit publish, fsync, rollback, create snapshot, destroy snapshot, truncate, write at offset (checked 2026-11-16; multiple files are added by Milestone 5 X·3) (`enum Operation` in `crates/singlefs-reference-filesystem/src/lib.rs`); ideal model in `crates/singlefs-verification/src/model.rs`.
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

## IV. Multi-Tenant Isolation (moved from milestone ten; decided by the user on 2026-11-11)


- **Verbatim**: "5 also has permission issues, because using O_DIRECT requires designing permissions" "in other words calling it multi-tenant isolation might be more appropriate" confirming "Yes, we want Linux permission model to judge permissions ourselves in o_direct mode" (User 2026-10-12).
- **Existing in repo**: On-disk format of inode records reserves 4 bytes each for mode / uid / gid / nlink at offset 24 (field table in settled item 6 of D8 (Core Index Structure)); `InodeRecord` in `crates/singlefs-core/src/records.rs` only carries inode, object birth generation, nlink, size, change count, write timestamp; writing to disk hardcodes mode to `0o100644`, uid and gid to 0, reading skips these three slots (writing and parsing of `InodeRecord`, checked in `crates/singlefs-core/src/records.rs`). How attributes are extended is linked to C115 (overflow path of inode attributes lacks placement), originally ordered in Milestone 11 I·2 (placement of attributes and extended attributes). Space accounting follows `.claude/rules/fs-design.md` "accounting is a byproduct of transactions, not post-hoc traversal" falling in first slot; per-tenant quotas must likewise be maintained incrementally upon commit. Checked 2026-11-16: `InodeRecord` also carries the truncation field; writing and parsing the record are inside the Verus-verified region (the lemmas in `crates/singlefs-core/src/records.rs`, `.claude/gate.d/verus-pins.tsv`), so IV·2 changes these three fields together with the proofs.
- **Work items**:

| # | Description | Backing clauses and owed checks | Acceptance | Dependencies |
|---|---|---|---|---|
| IV·1 | Design round (same round as I·1): how identity is obtained (inferred: disk-holding process obtains uid, gid, and supplementary groups of peer process via Unix socket); mode, uid, gid fall in slots reserved at offset 24 of inode records; how far alignment goes with Linux kernel judgments for regular files (beyond owner / group / others rwx tiers and root, whether setuid / setgid / sticky bits, POSIX ACLs, capabilities like `CAP_DAC_OVERRIDE` are recognized); whether listing (enumeration by inode number) filters by permission; raw disk itself governed by Linux device node permissions (which user disk-holding process runs as, which group disk belongs to); per-uid quotas maintained incrementally upon commit | Settled item 6 of D8 (Core Index Structure); C115 (overflow path of inode attributes lacks placement); metric table in D5 (Snapshot / Space Accounting Mechanism); settled item 1 of D16 (Publish Semantics) "acceptance"; C71 (Unrecognized statistics tag silently ignored) (quota per uid is the first new statistical dimension planned, attribution inferred, verify before starting work) | Three-party verdict filed in `research/prompts/`, settlements written back to kb; decisions require experiments | Stage 0; if the per-tenant-key path is chosen, also Milestone 15 I·2 and II |
| IV·2 | Implementation: inode records read and write mode, uid, gid (no longer hardcoding `0o100644` and 0); owner takes caller upon file creation; chmod, chown, quota modification are separate transactions traversing same transaction layer (`.claude/rules/fs-design.md` "start from transactions, not from features"); session entry judges read, write, truncate, delete, listing per caller uid, gid, supplementary groups and file mode; per-uid space quotas maintained incrementally upon commit, acceptance rejects accordingly | Settlement of IV·1 | Each new test proven to turn red first; differential testing against model matches item by item across interleaved multi-uid history | IV·1; I·2 |
| IV·3 | Unauthorized access test cases and differential testing: one test case per permission tier in Linux (writing without write bit, reading without read bit, modifying another file's mode, non-root changing owner, writing after exhausting own quota), all rejected; identical tuple (uid, gid, supplementary groups, mode, operation) judged via kernel on ext4 matches singlefs judgment item by item; streams crashing mid-chmod or chown connect to crash amplification; per-uid quota ledger in crash streams equals per-uid calculated by checker traversal | `.claude/rules/fs-design.md` "accounting is a byproduct of transactions, not post-hoc traversal" (runtime and checker use different algorithms) | Unauthorized access cases all rejected, differential testing matches item by item; removing entry check causes unauthorized test cases to turn red | IV·2 |

- **Must be settled before starting work**:
  1. Processes capable of directly opening raw disks bypass any file-level judgment (able to read every byte on disk); hence isolation based solely on permission bits requires tenants not to touch disks directly: multiple sessions can only take the form of "single disk-holding process serving multiple client sessions". To allow tenants to open disks directly, encryption partitioning keys by tenant is required, with encryption in Milestone 15 ([15-fifteenth-txn.md](15-fifteenth-txn.md)). Choosing between the two determines multi-session form in I·1 (inferred, unmeasured); if the latter is chosen, IV comes after Milestone 15.
  2. How far alignment with Linux goes: implementing rwx and root only, or including setuid / setgid / sticky bits, POSIX ACLs, and capabilities together.
- **To change after the move**: IV·1 originally says "in the same round as I·1", which means milestone ten I·1 (sessions and the multi-session form); the settlement of that round is an input here; where mode, uid and gid live is settled in the same round as I·2 (where attributes and extended attributes live); the execute bit of directories, path traversal and the sticky bit enter the scope of the judgment in IV·1; the per-uid quota keeps the approach IV·2 originally describes.

**Not in exits**: FUSE mount (Milestone 12 [12-twelfth-txn.md](12-twelfth-txn.md)), encryption AAD on directory entries (Milestone 15), cross-file packing and inlining on write path (settled item 3 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) judged negative).
