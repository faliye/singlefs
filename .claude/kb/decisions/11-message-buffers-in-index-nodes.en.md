## D11 Message Buffers in Index Nodes —— Settled

[中文](11-索引节点要不要留消息缓冲区.md) · **English** · [日本語](11-インデックスノード内のメッセージバッファ要否.ja.md)

D11 (Message Buffers in Index Nodes) governs whether to retain a message buffer in the index node disk format to accumulate small writes in internal nodes and flush them in batches instead of modifying leaves on every write (Bε-tree style)—namely the specific form of D10 (True Answer to Random Writes / Fragmentation) Candidate A. Its essence is "whether the authoritative value of a key is allowed to temporarily not reside in its final location", a mental model on the same level as "how space accounting works" (.claude/rules/format-evolution.md "mental models must evolve slowly"). It governs retention, the value of ε, branching criteria, how staleness is detected, whether buffers can be recomputed, and draining before rebuilds. What it does not govern: node size belongs to D8 (Core Index Structure) Settled Item 2; integration between the write buffer frontend and node message buffers belongs to D8 (Core Index Structure) Settled Item 12; message buffers land in layer 4 (internal layout of index nodes) of D15 (Format Freeze Policy). In the first version, all trees have ε = 0 (Settled Item 7).

### Settled Items

D11 (Message Buffers in Index Nodes) sub-items maintain a single numbering scheme without renumbering.

| # | Item | Decision |
|---|---|---|
| 1 | **Whether to retain message buffers** | Retain: keep message buffers in the disk format of index nodes; not enabled in the first version (Settled Item 7) **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **What value ε takes** | Future value 0.65 (fanout 119, buffer 665 entries), disabled in first version; re-run E56 (Message Buffer Gains vs Epsilon) with current pivots and message widths before enabling **Status: Settled.** **Progress: Unimplemented.** |
| 3 | **Staleness detection mechanism** | Detectable, mechanism piggybacks on the two recomputation sources of Settled Item 4; detection logic must not reuse message generation or application code **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Whether buffered messages can be recomputed solely from units** | Recomputable from authoritative state, sources are units (with write sequence) + instance table + ancestor table + tombstone units; buffer remains purely derived, no expansion to authoritative state **Status: Settled.** **Progress: No code object.** |
| 5 | **Draining contract before index rebuilds** | Draining is a prerequisite enforced by the rebuild entry point, not relying on caller discipline **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Whether ε is a single value or branches by layout** | Layout axis does not branch; tree axis branches by "whether pending flush messages can be recomputed solely from authoritative state", trees that cannot (inode tree only) have ε = 0 **Status: Settled.** **Progress: No code object.** |
| 7 | **ε for the other four trees and tree table units; trees of height 1; whether the first version sets ε = 0 across all trees** | All trees have ε = 0 in the first version (including extent and accounting), root of height 1 tree is leaf with no buffer **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Whether to retain message buffers

**Settled**: Retain. Keep a message buffer in the index node disk format, with buffers residing in internal nodes; the alternative of making messages an independent tree (`pending_ops`) has been rejected. In the first version, all trees have ε = 0 and are disabled (Settled Item 7).

**Scope**: This settlement is a verdict reached on top of two known open issues, not because they have vanished: Cost 2 (rebuildability degenerating from "detectable unreachable" to "undetectable stale") remains reasoning and unquantified; format branch criteria in .claude/rules/fs-design.md judge it as a negative improvement—message buffers cannot buy "wrongly used -> mount fails", and the same rule explicitly states "criteria must not include 'this is faster'" => performance gains do not constitute a pass under that criterion. It takes a step toward the root cause in [pitfalls.md](../pitfalls.md) item 10 (rebuildability not being a design goal), so Settled Item 3 (staleness detection mechanism) is not optional. Cost 1 (checker discriminative power dropping to zero for incremental Δ form) does not hold in this project because D8 (Core Index Structure) Settled Item 1 takes idempotent full values; if any class of accounting cannot be made idempotent, that class must have an independent recomputation path sharing no verdict code with runtime (C12 (shared delta semantics)).

**Basis**:

- E56 (Message Buffer Gains vs Epsilon): On 16 KiB nodes, gains are a strong function of ε, significant across four tree scales; node size determines ε resolution, not whether gains exist.
- E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta): Cost 1 holds only under incremental Δ form; under idempotent full value form, independent checker recomputations catch semantic write errors—after adopting idempotent full values, it does not block "retain".
- User settlement, dates and history in changelog; arguments rejecting "messages as independent tree" are also in changelog.

**Owed**: C52 (Treating Stale as Valid): safety-net check for Cost 2; C349 (no clause on moving node message buffers during reorganization): how messages in node buffers move during collapse or rebalance.

#### Settled Item 2: What value ε takes

**Settled**: ε = 0.65 is a future value, not the setting for the first version (Settled Item 7): fanout F = 119, buffer B = 665 entries, both contending for the same 16 KiB node bytes. Whether to enable and how much to enable will be settled after E56 (Message Buffer Gains vs Epsilon) is re-run with current pivots and message widths.

**Scope**: ε is reversible, while "retain or not" is irreversible: per D21 (Boundary Between Authoritative and Derived States), index is derived state => modifying ε = rebuild index, without changing format, so Settled Item 2 can be adjusted in the future based on results of Settled Item 3 (staleness detection mechanism), whereas Settled Item 1 cannot. The only force pushing ε downward is the exposure surface of Cost 2, which lacks numbers. The future values of 0.65, fanout 119, buffer 665 entries originated from pivot 48 and message width 16 in E56 (Message Buffer Gains vs Epsilon), both of which are invalidated inputs (C351 (Two Inputs for Epsilon 0.65 Obsoleted While Numbers Remain in Text)).

**Basis**:

- E56 (Message Buffer Gains vs Epsilon): ε = 0.65 consistently outperforms ε = 0.5 across four tree scales, with point lookup cost only a few percent higher; tree height is a dependent variable of (leaf count, fanout), and paying one extra layer has no decisive cost penalty.
- User settlement, dates and history in changelog.

**Owed**: C351 (Two Inputs for Epsilon 0.65 Obsoleted While Numbers Remain in Text): re-run E56 (Message Buffer Gains vs Epsilon) with current pivots and message widths; before re-running, 0.65 / 119 / 665 in text must carry "future value".

#### Settled Item 3: Staleness detection mechanism

**Settled**: Cost 2 (rebuildability degenerating from "detectable unreachable" to "undetectable stale") has a safety net, not intractable. The detection mechanism piggybacks on the two recomputation sources of Settled Item 4 without requiring a third component:

| Question | Answered By |
|---|---|
| Whether this placement is current version | **Unit intrinsic write sequence + instance table + ancestor table** (C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) settled P2 / P3: write sequence (instance generation, txn number) combined with instance table judges "published or not", ancestor table judges "visible to this tree", largest write sequence among visible candidates is current version) |
| Whether this key is already dead | **Tombstone unit** (settled that deletion must write a unit) |

**Detection logic must not reuse message generation or application code.** Otherwise it repeats the shape of bcachefs—catching deviations during application while missing errors in semantics themselves.

**Scope**: Settles "detectable", owes "proving it is detectable". Space accounting cannot answer "whether this placement is alive" on a per-placement basis (D3 (Space Allocation) Settled Item 1 states verbatim that the accounting tree cannot answer for individual placements), so the current version question is not delegated to accounting.

**Basis**:

- E104 (Current Version Determination in Scan Rebuild): With write sequence, instance table rows, ancestor table, death write sequence, container retirement records, and sweep gates all active, the all-rules arm correctly identifies current versions across all worlds and seeds without resurrecting deleted keys, while each ablation misjudges on target worlds—this set of inputs is sufficient and each bears load.
- E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta): When checker replays using the runtime accumulation function, it compares against itself and misses all semantic write errors—detection logic must not reuse that code.
- User settlement, dates and history in changelog.

**Owed**: C52 (Treating Stale as Valid): construct an image with "buffered message lost, corresponding leaf retaining old value", injected N times must be caught N times (criterion evaluates fault injection rate, not sample pass rate); disabling the detection mechanism on the same image must turn red to green.

#### Settled Item 4: Whether buffered messages can be recomputed solely from units

**Settled**: Yes, sources are "units (with write sequence) + instance table + ancestor table + tombstone units" => **buffers remain purely derived state, D21 (Boundary Between Authoritative and Derived States) "authoritative state = units + accounting + root" does not expand** (instance table units, like tree table units, are an indirection layer of the root; authoritative state gains no new items). Intrinsic unit write sequence determines publication (bounded by instance table), ancestor table determines tree-axis visibility, tombstone units determine key death.

**Scope**: It is not "solely from units"—the answer to that narrower formulation is no; what holds is recomputation from **authoritative state**. Accounting cannot answer whether individual placements are alive (D3 (Space Allocation) Settled Item 1), and this half is replaced by write sequence, instance table, and ancestor table mechanisms (C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) settlement).

**Basis**:

- E59 (Can Buffered Messages Recompute Solely from Units): Recomputing solely from units fails to distinguish current values on overwrites, resurrects deletions, and causes accounting errors; units + accounting, sequence numbers + tombstone units arms pass all four categories—"solely from units" does not hold, recomputing from authoritative state holds.
- E104 (Current Version Determination in Scan Rebuild): After replacing the oracle accounting in Arm B of E59 (Can Buffered Messages Recompute Solely from Units) with write sequence + instance table + ancestor table mechanisms, verdicts remain completely correct—accounting in the source can be replaced by this set of mechanisms.
- User settlement, dates and history in changelog.

**Owed**: None.

#### Settled Item 5: Draining contract before index rebuilds

**Settled**: **Before any index rebuild (changing ε, changing node size, corruption recovery), all buffer tiers (message buffers inside nodes; the write buffer frontend is an in-memory structure not cleared by rebuilds, remaining ahead of the tree in read order, as specified in round 3 of C161 (two buffer levels coexist but their hand-off is not settled)) must be drained first, and this step is enforced by the rebuild entry point rather than relying on caller discipline.**

**Scope**: Governs only the rebuild entry point; write buffer frontend is not within "all buffer tiers" and is not cleared by rebuilds.

**Basis**:

- E56 (Message Buffer Gains vs Epsilon): Update advantages after draining are only marginally lower than without draining—draining itself is very cheap; what is costly is that "forgetting to drain" triggers no alarms, so criteria place enforcement on "who enforces" rather than "remembering to drain".
- User settlement, dates and history in changelog.

**Owed**: C390 (index rebuild entry does not enforce draining): the check that triggers red requires rebuild paths and checker to exist, and cannot be written yet.

#### Settled Item 6: Whether ε is a single value or branches by layout

**Settled**: **Layout axis does not branch; all layouts share the same ε** (future value 0.65, unsettled until E56 (Message Buffer Gains vs Epsilon) is re-run with current widths, see Settled Item 2; first version ε = 0, Settled Item 7). Tree axis branches separately, with the branching variable being the predicate from Settled Item 4: **can pending flush messages for this tree be recomputed solely from authoritative state (units + accounting + tombstone units)**—yes => retain buffer, ε takes future value 0.65 of Settled Item 2; no => internal nodes retain no message buffer, ε = 0. Trees going through write buffer frontend (accounting / reverse index / LRU) form another branching variable: ε constant 0, no node buffer retained, frontend drains directly to leaves before publication (D8 (Core Index Structure) Settled Item 12).

**Scope**: Only inode tree falls on the "cannot" side: inode attribute updates have no authoritative carrier before reaching leaves (Code 3 packed record units), and buffering into derived nodes violates Hard Invariant 6 of D21 (Boundary Between Authoritative and Derived States) Settled Item 4. Extent / dirent / reverse index are judged as "can"; accounting tree is judged as "can" per D21 (Boundary Between Authoritative and Derived States) Settled Item 10 "offline recomputable by traversal", though that column in E59 (Can Buffered Messages Recompute Solely from Units) was an oracle and not tested on physical media (C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild)). Branching variable is constant per tree, satisfying the five hard requirements of .claude/rules/fs-design.md (explicit criteria, creating an inode tree lands on this branch, verified separately by E103 (Update Cost of Inode Packing Path 1) and crash-point replay, observable node layout, invariant mid-operation); it lands in layer 4 (derived state) of D15 (Format Freeze Policy), not a format branch. To branch by layout in the future, first propose a qualified branching variable (explicit, computable, invariant mid-operation), then satisfy the five hard requirements of .claude/rules/fs-design.md item by item.

**Basis**:

- D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 4 already judged the sole candidate layout branching variable (persistent / transient) as invalid (flipped from outside operation scope by `linkat` / `fsync` / snapshots, violating hard requirement 5 of .claude/rules/fs-design.md), while the same rule specifies implementing only one path when criteria cannot be formulated; this path is cheap because it is reversible (modifying ε = rebuild index, no format change).
- E98 (Geometry of Inode Records and Inode Trees): Most fields in inode records cannot be recomputed solely from authoritative state—inode tree falls on the "cannot" side.
- E59 (Can Buffered Messages Recompute Solely from Units): Units + accounting arm passes all three categories of mapping, reverse index, and tombstones—extent / dirent / reverse index fall on the "can" side.
- User settlement, dates and history in changelog.

**Owed**: None.

#### Settled Item 7: ε for the other four trees and tree table units; trees of height 1; whether the first version sets ε = 0 across all trees

**Settled**: All trees have ε = 0 in the first version (including extent and accounting), root of height 1 tree is leaf with no buffer; D8 (Core Index Structure) Settled Item 6 "root invariant Code 2" applies only to inode tree. ε = 0.65 in Settled Item 2 is retained as a future value: whether and how much to enable will be decided after E56 (Message Buffer Gains vs Epsilon) is re-run with current pivots (105–117) and message widths (34–109).

**Scope**: Does not alter Settled Item 1—"retain or not" is irreversible, ε is reversible. 0.65, fanout 119, buffer 665 entries in D11 (Message Buffers in Index Nodes) Settled Item 1 / Settled Item 2 / Settled Item 6 are future values after write buffer frontend is enabled, not settings for the first version; whether to enable and how much awaits E56 (Message Buffer Gains vs Epsilon) re-run.

**Basis**: No experiments: disabling buffers in the first version is a design tradeoff, not a measurable quantity—that future value lacks supporting numbers (its two inputs are invalidated, C351 (Two Inputs for Epsilon 0.65 Obsoleted While Numbers Remain in Text)), ε is reversible while "retain or not" is irreversible, so taking ε = 0 first leaves Settled Item 1 untouched. Comprehensive pre-coding review (D11 (Message Buffers in Index Nodes) sections in `records/2026-09-13-总审核.md`) recommended this arm, user settlement, dates and history in changelog.

**Owed**: C351 (Two Inputs for Epsilon 0.65 Obsoleted While Numbers Remain in Text).

## Historical Versions

Historical entries for D11 (Message Buffers in Index Nodes) are collected in [decisions-history.md](../decisions-history.md).
