## E99 Deduplication of Write Buffer Entries and Seq — Run (2026-09-03, Pure Counting, 12 Unit Tests / 10 Mutations All Caught): **Key of Accounting Tree Already Contains Generation => seq Needs Only 4-Byte In-Window Sequence Number; Putting Another Copy of txg in seq Is Double Accounting**

[中文](99-writebuffer条目与seq的去重.md) · **English** · [日本語](99-writebuffer条目与seq的去重.ja.md)

**What it answers is D8 (core index structure) settled item 7 (`seq` carried by write buffer entries)**: how wide, where it lives, and whether monotonic across mounts. This item judged "bytes altering new-pool file creation: **yes**".

### Cited Clauses (Copied Verbatim into Source Code Comments)

| Clause | The Sentence Used |
|---|---|
| D8 (core index structure) write buffer hard requirement 3 | "**Entries must carry a sequence number (seq); this is a format requirement**"; upon flush, sorting by key **discards temporal order**, and deduplication rule "latter wins" ⇒ "latter" must be carried by entry itself; **relative order of entries with same key is undefined under unstable sort** |
| D8 (core index structure) write buffer hard requirement 2 | Entry form must be idempotent complete value, not incremental Δ |
| D5 (Snapshot / Space Accounting Mechanism) settled item 3 rationale item 3 | Accounting traverses write buffer frontend, entries carry seq |
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Accounting key 22 bytes, entry 30 bytes |
| D16 (publish semantics) settled item 6 | Each publish increments `checkpoint_txg` by 1; fsync triggers are also publishes; **never rolls back (rollback)** |
| D23 (Role and Format of Journal) settled item 9 | `jsn` 10 bytes, sequence number and instance code divided in width — **existing solution to same class of problem** |
| E44 (Local Measurement and Cost of Sequence Number Bit-width) | Local machine 2785 publishes/sec; 48 bits lasts 3202 years; widening jsn to 12 bytes has zero cost |
| E17 (Upper Bound on Write Buffer Merging) | Initial version omitted seq, two arms failed to match truth on the spot |

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Bit width accounting must be calculated, no estimating**: How many bits each of two segments in `seq = (checkpoint_txg, in-window sequence)` occupies, and total bytes combined, **must be absolute values**. Given `checkpoint_txg` is 64 bits (D22 (How Unit Atomicity is Synthesized) settled item 7), **not a single bit remains in 8 bytes** ⇒ model must report the fact that "available bits for in-window sequence = 0", rather than glossing over.
2. **Deduplication determinism**: Four arms run the identical sequence of updates — ① no seq; ② wallclock; ③ counter zeroed on each mount; ④ `(checkpoint_txg, window sequence)` — **counting occurrences of "picking wrong winner"**. Absolute values required for every arm, no inter-arm comparison alone.
3. **Monotonicity across mounts**: Insert a mount in middle of update sequence, rerun criterion 2. Error count of arm ③ must **rise**, arm ④ must remain constant 0.
4. **Cost of truncation**: If truncating `checkpoint_txg` to `w` bits to yield window bits, how many times is "wrong winner picked" in same update sequence after wraparound? Wraparound period given per 2785 publishes/sec of E44 (Local Measurement and Cost of Sequence Number Bit-width).
5. **How wide in-window sequence must be**: Maximum times same key can be written within one publish window determines bit width lower bound of window segment. Inversely solved per `T_dirty` = 2 GiB in D16 (publish semantics) settled item 5 and 30-byte entry in D5 (Snapshot / Space Accounting Mechanism) settled item 5.

### Failure Clauses (Hardcoded Before Run)

| Clause | Disposition When Triggered |
|---|---|
| **Positive control, run on every arm**: Arm ① without seq must err most; arm ③ with inserted mount must err more than without | Not holding ⇒ deduplication model not connected, **whole round voided** |
| **Negative control**: Change update sequence to writing same key only once; error count across all four arms must be 0 | Not 0 ⇒ model already errs on unambiguous inputs, whole round voided |
| **Reverse acceptance clause**: If criterion 1 computes 8 bytes insufficient, and criterion 4 computes truncation introduces > 0 wrong selections | Conclusion is "**8 bytes insufficient, seq must widen**", written faithfully, **no forcing window segment to 0 bits** |

### Actual Measurement (2026-09-03, Pure Counting, N=5 Rounds Byte-Identical, 12 Unit Tests / 10 Mutations All Caught + 2 Equivalent Mutations Archived)

**Measurement basis and re-run**: Code `scripts/experiments/e99_writebuffer_sequence.rs`, artifact `e99-writebuffer-seq-2026-09-03.out` (37 lines, ending `emitted=37`), mutation table `scripts/mutation/tables/e99_writebuffer_sequence.tsv`, hooked into `scripts/experiments/replay.sh`.

#### Criteria 1 + 5: **8 Bytes Cannot Fit; 12 Bytes Is Sufficient**

| seq Width | Left for Window Sequence After `checkpoint_txg` Takes 64 Bits |
|---|---|
| 8 bytes | **0 bits** |
| 10 bytes | 16 bits |
| **12 bytes** | **32 bits** |

Lower bound demand for window sequence: One publish window holds at most **71,582,788** accounting entries (`T_dirty` 2 GiB ÷ 30-byte entry of D5 (Snapshot / Space Accounting Mechanism) settled item 5) ⇒ **at least 27 bits**.
=> If seq must **carry its own copy** of `checkpoint_txg`, neither 8 nor 10 bytes suffices; 12 bytes suffices.

#### Criterion 6 (Supplemented Round 2): Key of Accounting Tree **Already Contains Generation**, seq Need Not Redundantly Carry txg

D5 (Snapshot / Space Accounting Mechanism) settled item 5 verbatim settles key = `(statistic_tag 2, tree_id 8, device 4, **generation 8**)`, D16 (publish semantics) settled item 6 settles "generation = checkpoint number", and rider clause of D5 (Snapshot / Space Accounting Mechanism) settled item 2 verbatim supplements "cross-publish merging of accounting entries is **no longer feasible** (generation entered key)".

=> **Candidate deduplication entries of same key inevitably belong to same publish window** ⇒ seq needs only in-window sequence:

| Key Form | Required seq Width | Is txg Redundant with Existing Segment in Key |
|---|---|---|
| Accounting tree (key contains generation) | **4 bytes** | **Yes** |
| Reverse index / LRU (key layout unsettled in repo) | **Not applicable** | Cannot judge |

⚠️ **The 12-byte figure is double accounting** — it copied "generation 8" already in key once more into seq.
⚠️ **This holds only for trees whose key contains generation**; key layouts of the other two trees are unsettled, model reports "not applicable", no extrapolation.
⚠️ Consequently, the cross-mount table of criteria 2 / 3 is **outside scope on accounting tree**: entries of same key cannot coexist across mounts by key structure. That table answers "what if key lacked generation", remaining meaningful for the other two trees.

#### Criteria 2 + 3: Only Arm Derived from `checkpoint_txg` Remains Correct Across Mounts

1000 keys, 8 updates each, arrival order discarded after sorting by key:

| Arm | Without Inserted Mount (Wrong / Undefined) | With One Inserted Mount (Wrong / Undefined) |
|---|---|---|
| No seq | 0 / **1000** | 0 / **1000** |
| Wallclock | 0 / 0 | **1000** / 0 |
| Counter zeroed on each mount | 0 / 0 | 0 / **1000** |
| **`txg + window sequence`** | **0 / 0** | **0 / 0** |

⚠️ **"Wrong" and "undefined" must be counted separately** — D8 (core index structure) write buffer hard requirement 3 verbatim states "under unstable sort, **relative order of entries with same key is undefined**", model must not pick a side for implementation.
Negative control: all four arms 0 / 0 when writing same key only once.

#### Criterion 4: Cost of Truncating "Generation" — It Collides with **Key Identity**, Not Ordering

Generation entered key per D5 (Snapshot / Space Accounting Mechanism) settled item 5 ⇒ after truncating it, wrapped-around checkpoints **collapse into same key**, erroneously merging entries. Countable violations (supplemented 2026-09-04; versions 1 and 2 only printed wraparound period, never connecting number to any violation count):

| Generation Segment Width | 1,000 Publishes | 100,000 Publishes | 10 Million Publishes |
|---|---|---|---|
| 8 bits | **1464** pairs | 19,481,280 pairs | 195,307,500,032 pairs |
| 16 bits | 0 | 34,464 pairs | **757,947,392** pairs |
| 32 bits | 0 | 0 | 0 |
| 48 bits (settled lower bound) | 0 | 0 | **0** |

Wraparound period (per 2785 publishes/sec of E44 (Local Measurement and Cost of Sequence Number Bit-width)): Retaining 32 bits ⇒ **17 days** (identical number to "32 bits 17.8 days" in rider clause of D5 (Snapshot / Space Accounting Mechanism) settled item 2); retaining 48 bits ⇒ **3204 years** (same order of magnitude as "48 bits lasts 3202 years" in E44 (Local Measurement and Cost of Sequence Number Bit-width)).

### What It Cannot Answer

Pure counting model: no write buffer implementation, no real flush, no concurrency, no crash-point replay, 0 file operations.
**Does not answer "whether write buffer lands on disk"** — that is an unwritten prerequisite in repo; E99 (Deduplication of Write Buffer Entries and Seq) reports impacts under both "landing" and "not landing" readings, without settling for it.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (core index structure) settled item 7 | Support | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: settlement sentence of settled item 7 named E99 (Deduplication of Write Buffer Entries and Seq) (seq width 4 bytes, residing in value), measuring precisely its three questions; D8 (core index structure) rewritten per slimmed form on 2026-09-20, rules of this item unchanged by a single word) |
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: served only as input — accounting key ending with "generation 8" is premise of criterion 6 (candidate deduplication entries of same key inevitably belong to same publish window); E99 (Deduplication of Write Buffer Entries and Seq) does not evaluate how key segments are arranged) |
| D16 (publish semantics) | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: served only as input — "generation = checkpoint number, incremented by 1 per publish, never rolls back" and initial value of `T_dirty` taken from it to inversely solve window sequence bit width lower bound 27 bits) |
| D22 (How Unit Atomicity is Synthesized) settled item 7 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: served only as input — `checkpoint_txg` is 64 bits in root record field table, criterion 1 used it to calculate "not a single bit left in 8 bytes") |
| D23 (Role and Format of Journal) settled item 9 | Unaffected | 2026-09-27 unaffected: terminology renaming only touched wording, conclusions, numbers, and artifacts unchanged (prior review 2026-09-20: unaffected: served only as input — `jsn` divides width between instance code and counter, cited only as existing solution to same class of problem, no criterion resting upon it) |

## Historical Versions

### 2026-09-04
- Criterion 4 connected to countable violations (truncating generation ⇒ key collapse), and criterion 6 supplemented (generation already in key). Rationale: round 2 tripartite argumentation pointed out reverse acceptance clause had no input in code.

### 2026-09-03

#### (Part 1)
- Design registration, criteria and failure clauses hardcoded before run. Rationale: D8 (core index structure) settled item 7 exposed by criterion 9 of E97 (Entry Encoding for Accounting and Allocation Records), with three questions unanswered.

#### (Part 2)
- Run on same day and written back to measurement. Criteria 1..N and failure clauses unchanged by a single word.

