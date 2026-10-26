## E101 Cost and Expressiveness of Reserved Bits in Node Header — Run (2026-09-03, Pure Arithmetic, 7 Unit Tests / 5 Mutations All Caught): **Costs Did Not Drop a Single Cell, Tree Height Unchanged, but Upper Bound on Count of Tag Values for Node Group Arm Unobtainable => Reserving Bits Cannot Flip "Yes" of D26 (Background Compaction and Placement Reclamation) Settled Item 4**

[中文](101-节点头留位的代价与表达力.md) · **English** · [日本語](101-ノードヘッダ予約ビットのコストと表現力.ja.md)

**What it answers is the format half of D26 (Background Compaction and Placement Reclamation) settled item 4 (what form structural anti-aging takes)**: whether reserving a tag field in index node headers can flip the verdict "bytes altering new-pool file creation" of this item from **yes** to **no**.

### Cited Clauses (Copied Verbatim into Source Code Comments)

| Clause | The Sentence Used |
|---|---|
| D26 (Background Compaction and Placement Reclamation) settled item 4 | "Bytes altering new-pool file creation: **yes** (Rationale: creating files in a new pool must write index nodes, and this item directly alters node layout ⇒ without settling it, node bytes for new-pool file creation cannot be written)" |
| D18 (Information Carried in a Block) settled item 7 | Index node class identity segment = tree_id + level + key range settled to be carried by that tree + birth generation + fsid; common plaintext prefix 42 bytes |
| D19 (Block Pointer Structure & Width Budget) settled item 3 | **Fixed-width reservation**, reserved when without encryption; "reason is policy not usage" |
| D8 (Core Index Structure) settled item 2 | Node 16 KiB, pinned as constant |
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Accounting key 22 bytes, entry 30 bytes |
| D3 (Space Allocation) settled item 7 | Allocation record entry = device identity 4 + 16 KiB slot number 6 + span 2 + generation 8 |
| E93 (Placement and Fragmentation Under Aging) | Policy forms (clustered segment placement + neighbor rewrite radius R) contain fragmentation, but saturate at F ≈ 66%, budget capped by workload geometry |
| E95 (Aging Node Layout Arm) | **Unrun**: registered two format arms are "node group" and "generational separation" |
| D15 (Format Freeze Policy) | Decision table ranks "altering meaning of existing fields" as the most dangerous category |

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Cost must be calculated as absolute values**: Reserving 0 / 1 / 2 / 4 bytes per tier, calculating leaf fanout, internal fanout, and tree height tier by tier across three trees with settled entry widths (accounting 30, allocation records 20, inode records per tiers of E98 (Geometry of Inode Records and Inode Trees)). **Fanout strictly equals `(16384 − node_header − range_field) / entry_width`.**
2. **Expressiveness lower bound**: Formalize the two arms registered in E95 (Aging Node Layout Arm) into "one tag per node", **counting how many mutually distinct tag values each requires** ⇒ lower bound of tag width. If uncomputable, report "cannot express", no estimating.
3. **Flip criterion**: Verbatim rationale of D26 (Background Compaction and Placement Reclamation) settled item 4 judging "yes" is "this item directly alters node layout ⇒ without settling it, node bytes for new-pool file creation cannot be written". After reserving bits, **are node bytes written by new-pool file creation byte-for-byte identical under "whichever arm is chosen in future"** — identical means rationale dissolves and verdict can flip to no; **different means flipping is forbidden**, faithfully recording where differences lie.
4. **Collision count**: When tag width is insufficient, two groups of nodes required to be separate by the two arms will be forced to share a tag value. **Count forced merged groups**, and when width is sufficient this number must strictly equal 0.

### Failure Clauses (Hardcoded Before Run)

| Clause | Disposition When Triggered |
|---|---|
| **Positive control, run on every arm**: When tag width takes 0, collision counts for both arms must be > 0; when taking sufficiently wide, both must strictly equal 0 | Not holding ⇒ expressiveness dimension not in model, **whole round voided** |
| **Negative control**: When there is only one group, collision count under any width (including 0 bits) must equal 0 | Not 0 ⇒ model already errs on trivial inputs, whole round voided |
| **Reverse acceptance clause**: If criterion 3 judges "node bytes of new-pool file creation vary with selected arm" | Conclusion is "reserving bits cannot flip that verdict, D26 (Background Compaction and Placement Reclamation) settled item 4 stands before the first line of code as before", written faithfully |
| E95 (Aging Node Layout Arm) unrun ⇒ take the sentence written at its registration for two arm forms, **no expanding on one's own** | Expanding means proving oneself using arms fabricated by oneself |

### Actual Measurement (2026-09-03, Pure Arithmetic, N=5 Rounds Byte-Identical, 7 Unit Tests / 5 Mutations All Caught)

**Measurement basis and re-run**: Code `scripts/experiment-apparatus/e101_node_tag_reserve.rs`, artifact `e101-node-tag-reserve-2026-09-03.out` (50 lines, ending `emitted=50`), mutation table `scripts/mutation/tables/e101_node_tag_reserve.tsv`, hooked into `scripts/experiment-apparatus/replay.sh`.

#### Criterion 1: Cost Too Small to Constitute Objection

When reserving 4-byte tags, leaf fanout across three trees with settled entry widths **did not drop a single cell**:
accounting 544 → 544, allocation records 816 → 816, inode 116 → 116; **tree height completely unchanged across four width tiers**.
Measurement note (2026-09-05): "inode 116 → 116" calculated per "residing in index leaf" geometry ruled out by D8 (Core Index Structure) settled item 6; current form (leaf is container, internal entries 93, fanout 175) unrerun, conclusion of this arm does not represent current form.

#### Criteria 2 + 3: Flip **Does Not Hold**, and Solely Due to One Arm

| Arm | How Many Mutually Distinct Tag Values Required | 0 Bytes | 1 Byte | 2 Bytes | 4 Bytes |
|---|---|---|---|---|---|
| Generational separation | **49** (D22 (Synthesizing Unit Atomicity) settled item 2: K = 3S + 1, S ≤ 16) | Forced merge 48 | **0** | 0 | 0 |
| Node group | **Upper bound unobtainable** | Cannot judge | Cannot judge | Cannot judge | Cannot judge |

=> Across all four width tiers, "whether node bytes of new-pool file creation are stable" **all judged no** =>
**Reserving bits cannot flip "yes" of D26 (Background Compaction and Placement Reclamation) settled item 4**, that item standing before first line of code as before.

⚠️ **How to read this cell after 2026-09-13**: D26 (Background Compaction and Placement Reclamation) settled item 4 user settlement decided not to enter format — node group arm retired by E127 (Whether to Store Group Identity Under Splits and Merges) (the arm whose width could not be settled is gone), generational separation never halved on any of six cells per E95 (Aging Node Layout Arm) criterion 5 => neither arm needs reserved bits; the reason "reserving bits cannot flip" disappeared, but conclusion headed to other side: do not reserve.
⚠️ **There was only one reason it could not flip**: E95 (Aging Node Layout Arm) admitted "two format arms are **candidates not exhaustive**" and "how many bytes group identity and aging generation occupy in real format... **model does not calculate**" => no settled clause gave an upper bound for group count of node groups. **Generational separation arm needs only 1 byte** — the problem was not cost, but inability to settle width for the other arm.

Positive control: when replacing both arms with bounded ones, flip immediately holds => this dimension truly entered model.
Negative control: when there is only one group, collision count under any width (including 0 bits) is 0.

### What It Cannot Answer

Pure arithmetic: no aging workload, no compaction implementation, no actual measurement of E95 (Aging Node Layout Arm), 0 file operations.
**Does not answer "which anti-aging form is better"** — that belongs to E95 (Aging Node Layout Arm) and E10 (Aging: Structural Resistance vs Post-hoc Compaction); E101 (Cost and Expressiveness of Reserved Bits in Node Header) answers only "is reserving bits sufficient to postpone choice to that time".

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D26 (Background Compaction and Placement Reclamation) settled item 4 | Support | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: costs did not drop a single cell across four reserved width tiers and tree height unchanged; sole reason byte verdict could not flip was group count of node groups lacked upper bound; that reason disappeared as E127 (Whether to Store Group Identity Under Splits and Merges) retired node group, reopening 1-byte aging generation route on byte axis) |
| D18 (Information Carried in a Block) settled item 7 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: took index node class identity segment and common plaintext prefix only as input measurement basis) |
| D19 (Block Pointer Structure & Width Budget) settled item 3 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: took "fixed-width reservation, reserved when unencrypted" only as precedent for action of reserving bits) |
| D8 (Core Index Structure) settled item 2 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: node 16 KiB treated only as geometric constant) |
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: accounting key and entry width treated only as inputs to fanout arithmetic) |
| D3 (Space Allocation) settled item 7 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: allocation record entry width treated only as input to fanout arithmetic) |
| D15 (Format Freeze Policy) | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: cited ranking "altering meaning of existing fields" as most dangerous category only as boundary for reserved bit form) |
| D22 (Synthesizing Unit Atomicity) settled item 2 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: took K = 3S + 1, S ≤ 16 only to compute upper bound of tag value count for generational separation arm) |

## Historical Versions

### 2026-09-03

#### (Part 1)
- Design registration, criteria and failure clauses hardcoded before run. Rationale: D26 (Background Compaction and Placement Reclamation) settled item 4 currently blocks before first line of code, while its two prerequisite experiments (E95 (Aging Node Layout Arm) / E10 (Aging: Structural Resistance vs Post-hoc Compaction)) were unrun, and along present course it would block indefinitely.

#### (Part 2)
- Run on same day and written back to measurement. Criteria 1..N and failure clauses unchanged by a single word.
