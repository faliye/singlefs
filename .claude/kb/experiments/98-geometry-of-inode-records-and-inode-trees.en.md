## E98 Geometry of Inode Records and Inode Trees — Run (2026-09-03, Pure Arithmetic, 12 Unit Tests / 9 Mutations All Caught): **11 of 14 Fields Cannot Be Recalculated Solely from Authoritative State => Residing in Index Nodes Triggers D21 (Boundary Between Authoritative and Derived States) Hard Constraint 6, Packing into Shared Units Does Not Trigger**

[中文](98-inode记录与inode树的几何.md) · **English** · [日本語](98-inode-レコードと-inode-ツリーの幾何.ja.md)

**What it answers is D8 (Core Index Structure) settled item 6 (on-disk field table of inode records).** This item judged "bytes altering new-pool file creation: **yes**" — creating a file in a new pool inevitably requires writing its metadata.

### Cited Clauses (Copied Verbatim into Source Code Comments)

| Clause | The Sentence Used |
|---|---|
| D21 (Boundary Between Authoritative and Derived States) | Authoritative state = units + accounting + root; index is derived state; **cannot be recalculated solely from authoritative state ⇒ de facto promoted to authoritative state, authoritative state must expand items** |
| D18 (Information Carried in a Block) settled item 3 | Unit header 5-tuple `(unit_type_tag, tree_id, object_id, object_birth_gen, anchor_offset)` — **contains no POSIX attributes whatsoever** |
| D18 (Information Carried in a Block) settled item 7 | Common plaintext prefix 42 bytes; index node class identity segment = tree_id + level + key range settled to be carried by that tree + birth generation + fsid |
| D18 (Information Carried in a Block) settled item 2 | Nodes of **accounting tree** carry key range; dirent tree deferred. **Whether inode tree carries it is unstated in this clause** |
| D8 (Core Index Structure) settled item 3 | Data key = `(locality_id, inode, offset)`; `locality_id` inherited from parent directory upon creation, **never updated upon rename**; naming tree `(parent, name) → inode` has value containing only inode |
| D8 (Core Index Structure) settled item 2 | Node 16 KiB |
| D4 (Checksum Location) settled item 2 | Short extents padded (padding) to 32 KiB (1 KiB file occupies 32×) ⇒ `size` cannot derive on-disk occupancy |
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 | Inline threshold 0 ⇒ inode records do not hold file data |
| D16 (Publish Semantics) settled item 6 | Each publish increments `checkpoint_txg` by 1; fsync triggers are also publishes |
| D6 (Snapshot Implementation Model) | Multiple writable heads, each head with its own tree |

### Criteria (Hardcoded Before Run, Unchanged After Run)

1. **Absolute value assertions**: Under each record width tier, the number of records fitted into a 16 KiB leaf must strictly equal `(16384 − node_header) / record_width`, and tree height must be calculated independently from both leaf fanout and internal fanout tiers. **Inter-arm comparison alone is forbidden.**
2. **State category criterion (core of E98 (Geometry of Inode Records and Inode Trees))**: For each field, judge "can it be recalculated solely from authoritative state (units + accounting + root)", **counting the number of fields that cannot be recalculated**. Non-recalculable field count > 0 ⇒ inode record is not derived state; per criteria of D21 (Boundary Between Authoritative and Derived States), it either resides in units or authoritative state expands items — **both paths must be reported, without choosing for it.**
3. **Discriminative power of `locality_id` control**: Fabricate a batch of objects with `locality_id` in record inconsistent with prefix of extent key, counting objects "read as hole while checker judges green". **Arm with check must catch all; arm without check must miss all** — both numbers must be absolute values, not merely stating "with check is better".
4. **Ordering power of change count**: Modify same inode k times within same checkpoint, counting pairs "unable to order earlier vs later". When taking `checkpoint_txg`, this count must be > 0 (identical number) ⇒ **either add in-window sequence number, or obtain ordering from alternative source**, reported truthfully.
5. **Blast radius**: "Scan unit is node, not record" ⇒ how many inode records lost when one leaf node is lost. Absolute values.

### Failure Clauses (Hardcoded Before Run)

| Clause | Disposition When Triggered |
|---|---|
| **Positive control, run on every arm**: Arm without check in criterion 3 must miss all (missed detections = inconsistent objects); arm in criterion 2 "assuming attributes can be recalculated from unit header" must compute non-recalculable fields as 0 | Any failure ⇒ mechanism not in model, **whole round voided** |
| Base bytes of node header remain unsettled in repo ⇒ calculate once under each of three lower bound tiers 58 / 67 / 76 from E73 (Fanout Cost of Node Key Interval) | Tiers yield differing directions ⇒ conclusion written as "dependent on node header" |
| **Reverse acceptance clause**: If criterion 2 counts "non-recalculable fields > 0" **and** "residing in units" path is negated by blast radius of criterion 5 | Conclusion is "D21 (Boundary Between Authoritative and Derived States) must expand items", **recorded faithfully, no forced non-expansion** |

### Actual Measurement (2026-09-03, Pure Arithmetic, N=5 Rounds Byte-Identical, 11 Unit Tests / 8 Mutations All Caught)

**Measurement basis and re-run**:

- Code `scripts/experiment-apparatus/e98_inode_record.rs` (`cargo run --release --bin e98-inode-record`), raw output `e98-inode-record-2026-09-07.out` (rerun 2026-09-07 per header widths established that day; the one prior to reconciliation is `e98-inode-record-2026-09-03.out`, retained untouched in `results/`) (74 lines, ending `emitted=74`).
- Mutation table `scripts/mutation/tables/e98_inode_record.tsv`, 9 entries hit entry by entry, 0 judged equivalent; 12 unit tests.
- Hooked into `scripts/experiment-apparatus/replay.sh` (exact tier).
- ⚠️ **Header widths remain at old values, owing one rerun** (C89 (Leftover Recomputation from Closing the Unit Header) ④ ⑤): source code `DATA_UNIT_HEADER = 91`, current is 105 (`format-const: DATA_UNIT_HEADER_BYTES`); index node headers calculated at 58 / 67 / 76, current are 68 / 77 / 86. Neither flips when calculated across header width intervals of E102 (Unit Class Registry Table and Packed Record Units) (packed 233 falls within [9, 148], index leaf 116 identical on 16 KiB node), **but rerun required before citing these two numbers** — old numbers serve only as reference.

#### Criterion 2: 11 of 14 Fields Cannot Be Recalculated Solely from Authoritative State

| Recalculable (3) | Relying On What |
|---|---|
| `inode` | D18 (Information Carried in a Block) settled item 3 5-tuple object ID |
| `obj_birth` | Object birth generation of same 5-tuple |
| `blocks` | Counting units of this object (D4 (Checksum Location) settled item 2 padding to 32 KiB, fixed width per unit) |

Eleven non-recalculable: `locality_id` / `mode` / `uid` / `gid` / `nlink` / `size` / `rdev` / `atime` / `mtime` / `ctime` / `change_count`.
⚠️ Reason why `size` cannot be recalculated must be stated clearly: **scanning cannot yield size when trailing end is a hole** — anchor offset only reaches the furthest written point.

#### Form Criterion: Only "Packing into Shared Units" Avoids Triggering D21 (Boundary Between Authoritative and Derived States) Hard Constraint 6

D21 (Boundary Between Authoritative and Derived States) Hard Constraint 6 verbatim: "**Placing an uncomputable authoritative value into a reconstructible container de facto promotes it to authoritative state**... 'index reconstructible from units' and 'nodes carrying non-reconstructible authoritative bytes' **cannot both be true**."

| Form | Container | Triggers Hard Constraint 6 | How Many Entries per Container |
|---|---|---|---|
| Residing in index node value | 16 KiB index node (**derived state**) | **Yes** | 116 |
| **Packing into shared units** | 32 KiB packed record unit (D18 (Information Carried in a Block) settled item 11 code 3; **item 1 of authoritative state**) | **No** | **233** |
| Positive control: assuming all 14 fields can be computed | Neither triggers | No | — |

**Measurement note (2026-09-05)**: The arm "residing in index node value" (116 entries / internal fanout 243) was ruled out by D8 (Core Index Structure) settled item 6, numbers retained solely for traceability; current form is "leaf is container" (internal entries 93, fanout 175), geometry see E103 (Update Cost of Inode Packing Path 1) arm 3.
The response to criterion 4 "in-window ordering power" is in record table of D8 (Core Index Structure) settled item 6: multiple modifications within same publish window land only latest version on disk (internal nodes do not buffer). "Packing does not trigger hard constraint 6" under leaf-is-container is read per D21 (Boundary Between Authoritative and Derived States) "state category judged by recalculability": internal nodes are reconstructible from units, while leaf itself is a unit.

=> **Packed form follows tombstone precedent of D18 (Information Carried in a Block) settled item 10** ("extent record + packed shared unit").

#### Criteria 1 / 5: Geometry and Blast Radius

Record 140 bytes (identity segment 24 + attribute segment 84 + reserved 32).
**Across three tiers of node headers (58 / 67 / 76), leaf fanout constantly 116** => conclusion does not depend on node header.
Internal fanout 243 (key = inode 8 + child pointer 59); 1e6 inodes tree height 3, 1e9 tree height 4.

**Permanent loss when losing one container** (supplemented 2026-09-04; versions 1 and 2 published this number only on btree line, two forms never compared side-by-side => reverse acceptance clause structurally untouchable):

| Form | How Many Entries per Container | **Permanent Loss** | Needs Redundancy / Detectability Clause |
|---|---|---|---|
| Residing in index node value | 116 | **116** | Yes |
| Packing into shared units | 233 | **233** | Yes |
| Positive control: assuming all 14 fields can be computed back | — | **0** | No |

⚠️ **Permanent loss of both forms is non-zero**, whereas E29 (Blast Radius of a Single Failed Node) measured permanent loss of index nodes as **constant 0** — relying precisely on "index reconstructible from units", which inode records fail to satisfy.
=> **The phrase in reverse acceptance clause "blast radius negating residing in units" cannot evaluate** (no one set a threshold);
what this dimension truly exposes: **regardless of form chosen, a redundancy / detectability clause is required**, and permanent loss of packed form is exactly **2.00×** that of the other.

⚠️ **Head discriminator bit withdrawn**: D6 (Snapshot Implementation Model) settled item 1 settles "each head with its own tree", note in D8 (Core Index Structure) settled item 3 verbatim "two heads no longer land in same key slot" => this bit **has no code object**. Both arms measured: internal fanout 236 with 2-byte discriminator bit, 243 without — **7 slots paid for nothing**.

#### Criteria 3 / 4: Discriminative Power of Two Controls

`locality_id` control (1 million objects, 1000 inconsistencies): **Arm with check caught 1000, silent 0; arm without check caught 0, silently read holes 1000**. Negative control (zero inconsistencies) both arms 0.

Change count taking `checkpoint_txg`: 8 modifications within same checkpoint => **28 pairs** unable to order (C(8,2)); 64 modifications => 2016 pairs. Equipping 8-bit in-window sequence number zeroes it, but at 300 modifications 8 bits becomes insufficient again.
=> **Change count cannot order two versions in same window by checkpoint_txg alone.**

### What It Cannot Answer

Pure arithmetic geometric model: no btree implementation, no directories, no crash-point replay, 0 file operations.
**Does not answer wallclock.** Does not settle width of index node class identity segment for D18 (Information Carried in a Block) settled item 7 — scanned across tiers per its admitted assumptions.

⚠️ **Rerun once on 2026-09-07 per header widths established that day** (the step requested by user before settling D18 (Information Carried in a Block) settled item 14). Prior to 2026-09-07, the three base node header tiers in source code were **58 / 67 / 76**, which were lower bounds on the day E73 (Fanout Cost of Node Key Interval) ran; two subsequent widenings were never landed in source code, supplemented together this time: write sequence 10 (supplementary note to D18 (Information Carried in a Block) settled item 7) + reserved bits 28 (D18 (Information Carried in a Block) settled item 7 verbatim "v1 reserves field bits for nonce code and MAC", per nonce code 12 of settled item 14 arm A and MAC 16 of D9 (Encryption) settled item 2) => **96 / 105 / 114**. Both sets of headers kept in source code running grid by grid; old set was where this conclusion stood prior to 2026-09-07.
**Conclusion did not flip**, changing only intermediate quantities. Artifact replaced with the one rerun on same day.

**Intermediate quantities**: Internal fanout 243 → **242**, leaf fanout 116 and tree height (height 3 for 1e6, height 4 for 1e8) completely unchanged.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D11 (Message Buffers in Index Nodes) settled item 6 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 6 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) | Provisioning | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) | Provisioning | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) settled item 2 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) settled item 6 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D6 (Snapshot Implementation Model) settled item 1 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) settled item 2 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-04
- Supplemented measurement of **permanent loss** of two forms (116 / 233), providing input for reverse acceptance clause. Rationale: round 2 three-way argument pointed out reverse acceptance clause had no input in code.

### 2026-09-03

#### (Part 1)
- Design registration, criteria and failure clauses hardcoded before run. Rationale: D8 (Core Index Structure) settled item 6 judged "bytes altering new-pool file creation: yes", while its prior two rounds of argumentation stalled on field table, with no one questioning state category.

#### (Part 2)
- Run on same day and written back to measurement. Criteria 1..N and failure clauses unchanged by a single word.

