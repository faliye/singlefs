# Milestone "Clones and Multiple Writable Heads": Tasks, Acceptance, and Ordering

[中文](14-fourteenth-txn.md) · **English** · [日本語](14-fourteenth-txn.ja.md)

**Measurement basis**: Planning document, pre-filed on 2026-10-12, serving as a draft. The user settled Milestones 7 through 10, asking "what might you have missed, how do you think milestones should be ordered", and the main agent suggested placing 11 through 15 sequentially, based on the principle that changes to on-disk formats are scheduled before format freeze, and freeze before external availability (`records/2026-10-11-里程碑五打包与决策进度列.md` Section 7); on the same day the user requested "write it into /home/fy5090/code/singlefs/.claude/kb/milestone, pre-fill it", settling this order: "this is acceptable, adjustments needed during execution". **It does not require correctness**: revise before starting each item. The prerequisite is that all exit criteria for Milestones 3 through 13 are fully met, especially read-only snapshots, deadlist, and GC from Milestone 4 ([04-fourth-txn.md](04-fourth-txn.md)); if any item is missing when previous milestones close out, reschedule this document according to what is missing.

**How to read**: Three fields per item—"Existing in repo" lists which clauses, owed checks, and code this item bears down on in the repository (inspected on 2026-10-12, with sources); "Tasks" is a table: number, what to do, clauses and owed checks borne down upon, acceptance (prove each goes red first), dependencies; "To be settled before starting" lists open questions that must undergo three-way arbitration or ask the user before work begins. Anything labeled "inferred" is a judgment by the main agent without measurement. Task numbering is written as "Item·Point" (I·1, II·3), used solely within this document. Owed check numbers were all open on the day of filing, to be checked individually before starting work.

**Two new transaction types added**: Creating a clone (branching a writable head from a snapshot) and destroying a writable head; both follow the same transaction layer, and destruction is batchable and resumable (D6 (Snapshot Implementation Model) settled item 8 criterion 5 "bounded destruction").

## Exit Criteria (Unsettled)

Exit criteria and whether version 1 adopts the O(branch count) optimization from D6 (Snapshot Implementation Model) settled item 10 are left to user decision in Phase 0. Main agent's inferred candidates (inferred, unmeasured):

| # | Candidate Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | One tree per head, reference count side table, 3 fixed metadata records per head land on disk, on-disk patterns registered in `.claude/kb/layout/` with filename starting with `14-` | I |
| 2 | Destroying a writable head: livelist event stream, fork point snapshots forbidden to destroy, batched resumption | II |
| 3 | Whole-domain 0 red for crash streams of cloning and head destruction; shared blocks across heads do not read stale under GC relocation | III |

## Grouping and Ordering

| Phase | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Check snapshots, deadlist, and GC delivered by Milestone 4; settle exit criteria | Clones build on snapshots | Unstarted |
| 1 Cloning | I·1 → I·2 | Trees and side tables precede per-head deadlists | Unstarted |
| 2 Destruction | II·1 → II·2 | Depends on I | Unstarted |
| 3 Coverage | III·1 | Depends on I, II | Unstarted |

**Estimation**: Not estimated. Record `records/2026-10-11-里程碑五打包与决策进度列.md` Section 3 inferred 2–3 weeks (inferred, unmeasured).

## I. Cloning

- **Existing in repo**: D6 (Snapshot Implementation Model) settled item 1 "adopts ②, each head has its own tree; accepts a reference count side table plus 3 fixed metadata records per head"; D5 (Snapshot / Space Accounting Mechanism) settled item 18 "each head holds its own predecessor snapshot number and deadlist, initialized according to origin; fork point snapshots and heads with their own live snapshots forbidden to destroy; destroying a head requires livelist"; D21 (Boundary Between Authoritative and Derived States) settled item 6 "the checklist half requires no new structures; the counting half belongs to D5 (Snapshot / Space Accounting Mechanism) sparse side table". `grep -rl livelist crates/*/src` hits only pool-level checker and format constants (`crates/singlefs-checker/src/walk.rs`, `crates/singlefs-format/src/lib.rs`), absent in core layer. Owed checks: C236 (sparse side table deemed mandatory yet lacks any placement), C110 (block header consistency reading method for cross-head sharing and post-encryption metadata), C347 (no clause for what clone head claims when ancestor table is unreadable), C765 (I-9.29 literally does not distinguish trees, false positive with clone heads), C767 (retirement record lacks head ID, clone head causes reader to misjudge death).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | One tree per head, heads in tree table, reference count side table; I-9.30 (step rows of inode number watermark match watermark, records, and lineage) adds ⑤ (step H strictly increasing) split per tree: ⑤ does not falsely turn red during the single-head period, and splitting it per tree waits for the trees to be split here (three-way verdict `research/prompts/m3-close-c693-r3-main-verification.md` section 2, the row on tombstones and the head-deletion step row; inferred) | D6 (Snapshot Implementation Model) settled item 1; D21 (Boundary Between Authoritative and Derived States) settled item 6; C236 (sparse side table deemed mandatory yet lacks any placement), C765 (I-9.29 literally does not distinguish trees, false positive with clone heads), C767 (retirement record lacks head ID, clone head causes reader to misjudge death); C117 (device segment has no value for metrics without device dimension) (which initial rows must be written upon publish creating new tree, attribution is inferred, verify before starting); C342 (tree ID high-water mark rolls back and reissues when root is unreadable) (moved from Milestone 3 on 2026-10-19, user on 2026-10-18 settled retaining as known scope, owed check remains open, see registry rows); C726 (I-9.29 does not compare versions of same instance when instance table is unreadable), C727 (GPU I-9.29 cross-version table 2048 cells, whole run halts when deletion fills), C737 (I-9.29 still unjudgeable when instance table is partially unreadable) (moved from Milestone 3 on 2026-10-19, attribution is inferred, verify before starting, remaining owed per registry rows) | Bad image with side table count undercounted by one turns red | Phase 0 |
| I·2 | Predecessor snapshot number and deadlist per head, initialized according to origin | D5 (Snapshot / Space Accounting Mechanism) settled item 18; C347 (no clause for what clone head claims when ancestor table is unreadable), C110 (block header consistency reading method for cross-head sharing and post-encryption metadata) | Mutation failing to initialize according to origin turns red | I·1 |

## II. Destroying Writable Heads

- **Existing in repo**: D6 (Snapshot Implementation Model) settled item 3 "required; adopts ZFS livelist event stream form, FREE recorded at the moment block is killed; carrier is a shared tree, key prefixed with tree ID of the head"; D6 (Snapshot Implementation Model) settled item 5 "implemented, paying search cost: O(N) reverse index lookups per deleted snapshot"; D6 (Snapshot Implementation Model) settled item 10 "nested interval labeling plus deletion frontier verified at model layer to recover O(branch count); before two bounds repaid, implementation continues paying lookup cost". Owed checks: C35 (O(N) for multiple writable heads never optimized), C260 (livelist entry prepay does not account for unwinding itself), C265 (release boundary for destroying writable head written as fixed cloning timestamp), C266 (livelist entry identity uses position hint that can expire), C270 (whether fork point gate child count includes clones), C271 (at what moment livelist FREE is recorded has no definition), C272 (second destruction gate lacks O(1) check method), C337 (no clause for when destroyed head ceases to count as live reference).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | livelist event stream: FREE recorded at moment block is killed | D6 (Snapshot Implementation Model) settled item 3; C266 (livelist entry identity uses position hint that can expire), C271 (at what moment livelist FREE is recorded has no definition), C260 (livelist entry prepay does not account for unwinding itself) | Mutation omitting one FREE record turns red | I |
| II·2 | Destroying head: fork point gate, batched resumption, release boundary; the head-deletion step row (user settled on 2026-10-20 to move it here from Milestone 3), which first requires settling the per-head lineage record form: all three candidates for the head-deletion step row collide with I-9.30 (step rows of inode number watermark match watermark, records, and lineage) judged per head | D5 (Snapshot / Space Accounting Mechanism) settled item 18; D6 (Snapshot Implementation Model) settled items 5, 10; C265 (release boundary for destroying writable head written as fixed cloning timestamp), C270 (whether fork point gate child count includes clones), C272 (second destruction gate lacks O(1) check method), C337 (no clause for when destroyed head ceases to count as live reference), C35 (O(N) for multiple writable heads never optimized) | Destruction of fork point snapshot rejected; mid-destruction crash whole-domain 0 red | II·1 |

## III. Crash Coverage

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | One stream each for creating clone, writing to two heads separately, GC relocating cross-head shared blocks, destroying one head, connected into crash amplification | D13 (verification route) settled items 9, 15 | Whole-domain 0 red; differential against model extended to multiple heads | I, II |

**Not in exit criteria**: Cross-head non-colliding nonces under encryption (D6 (Snapshot Implementation Model) settled item 1 table row 3 "nonce obligation remains" and D9 (encryption) settled item 4, along with Milestone 15 [15-fifteenth-txn.md](15-fifteenth-txn.md)).
