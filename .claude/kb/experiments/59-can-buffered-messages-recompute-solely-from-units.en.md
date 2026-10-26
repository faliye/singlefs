## E59 (Can Buffered Messages Recompute Solely from Units) — Run (2026-08-31)

[中文](59-缓冲消息能不能只从单元重算.md) · **English** · [日本語](59-バッファメッセージはユニットのみから再計算可能か.ja.md)

Answers D11 (Message Buffers in Index Nodes) settled item 4 (original question before settlement), verbatim:

> Whether that message in the buffer can be recomputed solely from units.
> If yes ⇒ buffer remains purely derived state; if no ⇒ it is promoted to authoritative state,
> and D21 (Boundary Between Authoritative and Derived States) "authoritative state = unit + accounting + root" must be expanded.

**This is the sole remaining existential prerequisite following D11 (Message Buffers in Index Nodes) settled item 1 (retain).**

### Clauses Read Verbatim Before Modeling (verify-before-claiming.md: No Modeling from Memory)

| Clause | Crucial Verbatim Sentence for E59 (Can Buffered Messages Recompute Solely from Units) |
|---|---|
| D18 (Information Carried in a Block) settled item 3 | Header fields are `(unit_type_tag, tree_id, object_id, object_birth_txg, anchor_offset)`; **forbids physical placement / device_id**. ⇒ Header contains **no** "which write count this is"—`object_birth_txg` is the object's birth generation |
| D18 (Information Carried in a Block) settled item 1 | Lineage rewrite sequence number "whether to implement and bit width" is **unsettled** ⇒ It is an arm variable, not an existing fact |
| D8 (Core Index Structure) settled item 1 | Entry form adopts **idempotent full value** (including tombstones), not incremental Δ |
| D21 (Boundary Between Authoritative and Derived States) | Authoritative state = **unit + accounting + root**; index is derived state |
| D11 (Message Buffers in Index Nodes) settled item 2 | 16 KiB node, ε=0.65 ⇒ fanout 119, buffer 665 entries |

### Criteria (Hardcoded Pre-Run)

Discrepancy count across four message classes for Arm A (solely from units): all 0 ⇒ **recomputable**; partial classes > 0 ⇒ **split by class**;
all classes > 0 ⇒ **not recomputable**; all 0 when stale replicas invisible, > 0 when visible ⇒ conclusion is **conditioned on that premise**, premise listed separately.

### What Constitutes Failure (Hardcoded Pre-Run, Forbidden from Modifying Post-Run)

| Clause | Implementation Status |
|---|---|
| Scanned unit count ≠ independently computed expected count ⇒ entire round scrapped | 7 arms × 3 workload groups **cell-by-cell identical** |
| Arm D (sequence number + explicit tombstone unit) any class ≠ 0 ⇒ entire round scrapped | All four classes 0 |
| Injecting k corruptions into any arm without reporting exactly k additional `map_wrong` ⇒ comparator lacks discriminative power, entire round scrapped | Injected 1 / 5 / 17 corruptions across each of 7 arms, reported one by one |
| Resident capacity computed as 0 ⇒ entire round scrapped | 1024 / 8192 / 32768 leaves yield 6650 / 46550 / 186200 entries |
| Conclusion direction diverges from D11 (Message Buffers in Index Nodes) settled item 1 ⇒ record faithfully | See "What It Says About Settled Item 1" |

### Results (2026-08-31, 5 Rounds Byte-Identical): Criterion "Split by Class"

Seven arms, three workload groups (256 / 4096 / 16384 objects, each with 25% overwrite, 12.5% deletion).
Group "16384 objects, 4096 overwrites, 2048 deletions, 16 trees":

| Arm | Indistinguishable Replicas | Mapping Errors | Resurrected Deleted Keys | Reverse Index Missing | Trees with Accounting Errors | Total Discrepancies |
|---|---|---|---|---|---|---|
| **Arm A Solely from Units** (literal question of settled item 4) | **4096** | 0 | **2048** | 0 | **16** | **2064** |
| **Arm G Solely from Units + Allocator Reusing Low Addresses** | **4096** | **4096** | **2048** | **4096** | **16** | **10256** |
| Arm B Unit + Accounting | 0 | 0 | 0 | 0 | 0 | **0** |
| Arm C Unit + Lineage Rewrite Sequence Number | 0 | 0 | **2048** | 0 | **16** | **2064** |
| Arm D Sequence Number + Explicit Tombstone Unit (Full Control) | 0 | 0 | 0 | 0 | 0 | **0** |
| Arm E Stale Replicas Unreadable | 0 | 0 | 0 | 0 | 0 | **0** |
| Arm F Header Fully Wiped (Negative Control) | 0 | Missing 14336 | 0 | **14336** | **16** | 28688 |

⇒ **Split by class, and the four classes do not share parallel standing**:

| Message Class | Recomputable Solely from Units? | Sticking Point |
|---|---|---|
| **Mapping Class** (logical identity → placement) | **Condition holds** | When multiple readable replicas exist for same key, header contains no field to discern current value. Indistinguishable count **equals overwrite count exactly** |
| **Reverse Index Class** | Same as mapping class | Represents opposite side of mapping class, co-existing and perishing item by item |
| **Tombstone Class** (deletion) | **Fails, without condition** | Deletions leave **no units on disk** ⇒ scanning inevitably resurrects deleted keys, resurrection count **equals deletion count exactly** |
| **Accounting Class** (live unit count per tree) | **Cascades error with tombstone class** | Derived from "which units are alive"; prior cell error mandates accounting error, each tree pushed higher in measurement |

### Three Decisive Absolute Values (Not Inter-Arm Comparisons)

1. **Keys unable to distinguish current value = overwrite count** (64 / 1024 / 4096, cell-by-cell identical).
2. **Resurrected keys = deletion count** (32 / 512 / 2048), **with Arm C (with sequence number) losing none**—
   lineage rewrite sequence number buys the overwrite cell, but cannot buy the deletion cell.
3. Missing entry count when headers fully wiped = total live keys (224 / 3584 / 14336).

### Arm A "Map Error = 0" Was Luck; Arm G Unpacked It

Arm A had zero mapping class errors—**yet the rebuilder had no way to know it was correct** (indistinguishable replicas remained 4096).
It succeeded solely because allocation in the model proceeded monotonically forward: "scanning by physical address and picking the highest address" happened to equal "picking the newest version".

**No settled clause guarantees this**: D3 (Space Allocation) does not mandate monotonic allocation,
and for COW allocators to reuse released space is standard behavior. Arm G modified only one thing—overwrites landed at lower addresses—
and the identical rebuilding code **bet wrong on every single item**, error count matching overwrite count exactly.

⚠️ ⇒ **"Arm A mapping class equals 0" must never be cited as "mapping class can be recomputed solely from units".**
The permissible citation is: **it depends on an allocator property that nobody has ever settled.**

### Arm B: Closed by Accounting, Not by Units

Unit + accounting arm had all four classes at 0, **because "is this placement still alive" is precisely what accounting tracks**.
D21 (Boundary Between Authoritative and Derived States) settled authoritative state = unit + **accounting** + root ⇒ accounting does not perish alongside index.

⚠️ **However, this arm relies on an unsettled premise**: it assumes accounting can answer "alive or dead" **per-placement**.
Neither D3 (Space Allocation) nor D5 (Snapshot / Space Accounting Mechanism) settled this granularity—
D5 (Snapshot / Space Accounting Mechanism) records usage **with respect to sets**, not "whether a given placement belongs to current active version".
⇒ Arm B provides the **shape of an answer**, not the answer: adopting it requires settling accounting granularity first.

### What It Says About D11 (Message Buffers in Index Nodes) Settled Item 1

Settled item 1 (retain) stated it was an "adjudication made over two known unsettled issues", one being Cost 2
(rebuildability degrades from "detectable unreachable" to "undetectable stale", **reasoned rather than quantified**).

E59 (Can Buffered Messages Recompute Solely from Units) quantified a figure for the **fallback path** of Cost 2, not for Cost 2 itself:
it measured "whether it can be recomputed from disk after buffer is lost", whereas Cost 2 asks "whether an alarm sounds upon miscalculating during recomputation".
**The two are adjacent links in the same chain; E59 (Can Buffered Messages Recompute Solely from Units) touched only the former.**
The latter link remains an owed check, landing in [checks-owed.md](../checks-owed.md) C52 (Treating Stale as Valid).

### The Half It Cannot Answer

1. **Count model is not implementation**, zero file operations, sharing identical limitation with E54 (How Many Generations to Retain in Accounting) and E57 (Authoritative Placement for Plaintext-Side Field Set).
2. **Does not rebuild Bε trees**: geometry serves only to compute "how many entries the whole tree buffer holds" (delineating resident window),
   not participating in determining whether any message class is recomputable. Whether a message can be recomputed is independent of which node it currently occupies.
3. **No concurrency, no crash points**: rebuilding occurs on a quiescent disk. Real recovery must execute post-crash,
   introducing the question of "where recovery replay resumes", on which E59 (Can Buffered Messages Recompute Solely from Units) provides zero numbers.
4. **Deletion modeling has only one form** (leaving no traces on disk). If D3 (Space Allocation) settles "releases must write intent records",
   that record itself acts as a tombstone, requiring re-running the third cell of E59 (Can Buffered Messages Recompute Solely from Units).
5. **Does not touch encryption**: conflict between A6 (AAD expected values not sourced from plaintext side) and rebuild path is governed by D18 (Information Carried in a Block) settled item 5;
   E59 (Can Buffered Messages Recompute Solely from Units) assumes headers are readable.
6. **Four message classes were selected by E59 (Can Buffered Messages Recompute Solely from Units)**, not supplied by settled clauses. If buffers accommodate other classes in future, each requires separate evaluation.

### Mutation Testing: 16 Corruptions, 16 Caught, Zero Blind Spots

Includes "Arm A peeking at accounting", "Arm G reverting to monotonic allocation", "Sequence number picking minimum rather than maximum", "Tombstones read as live",
"Accounting class always computed correctly", "Buffer entry count altered by one tier".

⚠️ **First round contained a genuine blind spot**: reverse index column had only a single differential assertion `reverse_index_bad == map_wrong`,
which ran on Arm A—**both numbers were 0**, and `0 == 0` proved nothing. Changing missing entry branch count to `+= 0`
failed to turn a single test red. After first round, updated to pin three absolute values across three arms (0 / overwrite count / total live keys).
**This exemplifies test-discipline.md "comparing multiple arms mutually fails to detect when all arms err together"**,
caught by mutation testing rather than manual review.

⚠️ **Another item investigated from "blind spot" into "dead code"**: in comparator, the branch "same placement pointing to a different key"
could not be turned red by any mutation—investigation confirmed it was **unreachable**: ownership of a placement is governed by that placement's own header,
so both sides cannot yield conflicting keys for identical placement. Formulated into assertion once clarified; model explodes on contradiction.

### Prerequisites

None. Pure count model.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e59_message_recompute.rs` (7 arms, 14 unit tests)
  (`cargo run --release --bin e59_message_recompute`),
  raw output `e59-msg-recompute-2026-08-31.out` (26 lines, matching `emitted=26`).
- Mutation table `scripts/mutation/tables/e59_message_recompute.tsv`.
- Re-run differential: line for E59 (Can Buffered Messages Recompute Solely from Units) in `bash scripts/experiment-apparatus/replay.sh`.

⚠️ **E59 (Can Buffered Messages Recompute Solely from Units) numbers did not undergo three-way argument; D11 (Message Buffers in Index Nodes) settled item 4
was settled by user on 2026-08-31.** User settlement does not require three-way argument (which governs inferences), but implies
**the interpretive layer of "what these numbers demonstrate" was not verified by second or third legs**—
citing E59 (Can Buffered Messages Recompute Solely from Units) figures to support further conclusions still requires three-way argument.

### Affected Decisions

| Decision Item | Relationship | Retrospective |
|---|---|---|
| D15 (Format Freeze Policy) settled item 9 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 11 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) settled item 4 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) settled item 6 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) settled item 2 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 8 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 10 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 11 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 5 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-31

- Project initiated and run, concluding on criterion "split by class". 5 rounds byte-identical, 16 mutations caught.
  Basis: after D11 (Message Buffers in Index Nodes) settled items 1 / 2 settled, settled item 4 was the sole remaining existential prerequisite; user requested experiment to resolve it.
- Added Arm G (allocator reusing low addresses). Basis: Arm A mapping class at 0 was an artifact of monotonic allocation—a property **nobody had settled**; omitting this arm would attribute luck as a conclusion.
- Replaced differential assertion in reverse index column with three absolute values. Basis: mutation testing reported genuine blind spot (`0 == 0`).
