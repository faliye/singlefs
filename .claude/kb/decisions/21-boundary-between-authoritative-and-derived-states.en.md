## D21 (Boundary Between Authoritative and Derived States) —— Settled

[中文](21-权威态与派生态的分界.md) · **English** · [日本語](21-権威態と派生態の境界.ja.md)

D21 (Boundary Between Authoritative and Derived States) governs how on-disk entities are divided into two classes: what authoritative state is, what derived state is, the criteria for determining classification, whether derived state can change formats arbitrarily, the single route for reconstruction, and the extension point reserved on units for line owners (form, size, how upper bounds are computed, five hard constraints, and what intra-quota space is). What it does not govern: field tables for unit headers and unit class codes are in D18 (Information Carried in a Block); pointer fields and widths are in D19 (Block Pointer Structure & Width Budget); the internal structure of the allocation record tree is in D3 (Space Allocation); snapshot accounting and sparse side-table fields are in D5 (Snapshot / Space Accounting Mechanism); freeze layering is in D15 (Format Freeze Policy); load-bearing surface itself is in D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment).

### Settled Items

**D21 (Boundary Between Authoritative and Derived States) items have a single numbering scheme**, continuously numbered with the "Unsettled Items" section, without reordering.

| # | Item | Ruling |
|---|---|---|
| 1 | **Whether reverse index is authoritative or derived state** | Authoritative state — core reverse mapping already resides in item 1 (units), not a fourth item added to the authoritative state inventory **Status: Settled.** **Progress: Implemented.** |
| 2 | **Fixed byte upper bound on custom space** | No prior constant established; upper bound computed from mount-time judgment suite; self-authenticating unit judgment deleted per Settled Item 4 **Status: Settled.** **Progress: Unimplemented.** |
| 3 | **Field measurement basis for extension point pointer** | Single pointer 7 bytes, unit in bytes (intra-quota offset); field stores "offset + 1", all-zero represents empty **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Which unit classes carry extension points** | Data units carry; index nodes conditionally carry (Hard Constraint 6); self-authenticating units do not carry **Status: Settled.** **Progress: Unimplemented.** |
| 5 | **State classification of allocation records** | Derived state: entire tree reconstructible from authoritative state; authoritative state inventory not expanded **Status: Settled.** **Progress: Implemented.** |
| 6 | **Where the "single unit has multiple referencers" half under sharing resides and its form** | Referencer list half requires no new structure; count half assigned to D5 (Snapshot / Space Accounting Mechanism) sparse side-table **Status: Settled.** **Progress: Unimplemented.** |
| 7 | **Form of space within extension point quota** | Consists of SingleFS units **Status: Settled.** **Progress: Unimplemented.** |
| 8 | **Whether Class 3 carries extension points; position and quota addressing within units** | Class 3 carries; N is pool-level declared value in system configuration, 0 in version 1; extension points abut unit tail, growing backward from end **Status: Settled.** **Progress: Implemented.** |
| 9 | **Contents of authoritative and derived states** | Authoritative state = units + accounting + root (root includes tree table units and instance table units); index is derived state, excluded from format freeze **Status: Settled.** **Progress: Implemented.** |
| 10 | **Criteria for determining state classification** | Judged by recomputability, querying semantics rather than bytes: reader receiving conservative answer upon reconstruction failure remains derived state; receiving wrong answer mandates expulsion or promotion to authoritative state **Status: Settled.** **Progress: No code object.** |
| 11 | **Prerequisite for "loss merely degrades performance"** | Derived state corruption must be detectable; fails for portions degrading into undetectable staleness **Status: Settled.** **Progress: No code object.** |
| 12 | **Whether derived state changes format** | No "format conversion" operation exists, only reconstruction: no compatibility shims or converters built **Status: Settled.** **Progress: No code object.** |
| 13 | **Route for rescue, tree replacement, and migration** | Shared identical scan-reconstruction path; sole backward-compatibility requirement is recognizing all historical unit formats **Status: Settled.** **Progress: Unimplemented.** |
| 14 | **What form the extension point takes** | A piece of reserved space plus a set of interfaces, where what is housed inside is a pointer rather than a block of opaque bytes **Status: Settled.** **Progress: Unimplemented.** |
| 15 | **Declaration basis for extension point size** | Each line declares value in system configuration (may be 0); upper bound takes fixed byte count, not proportional to unit size **Status: Settled.** **Progress: Implemented.** |
| 16 | **Evaluation timing for unit geometry** | Evaluated once at mount time, checked at runtime without computation; declaration judgments run per device; persisted position quantities partitioned by pool max(physical_block_size) **Status: Settled.** **Progress: Implemented.** |
| 17 | **Hard constraints on extension points** | Five rules: covered by checksum / authentication, checker ignores content, size format-declared, targets use SingleFS allocation and accounting, core-layer targets forbidden; lifecycle bound to host unit **Status: Settled.** **Progress: Unimplemented.** |
| 18 | **Usage of byte-by-byte comparison table** | Serves solely to refute "SingleFS is extravagant", establishing no upper bound; "comparing frugality" forbidden as judgment criterion **Status: Settled.** **Progress: No code object.** |
| 19 | **Which half of reverse mapping self-containment buys** | "Which object occupies this physical location" answered by reading single unit; shared multi-referencer half handled separately, narrowing reverse index scope **Status: Settled.** **Progress: No code object.** |

#### Settled Item 1: Reverse Index in Authoritative State, Core Resides in Units

**Ruling**: Reverse index belongs to authoritative state as self-contained information. It is not a fourth item on the authoritative state inventory: units inherently carry "whom I belong to" ⇒ "which object occupies this physical location" answered by reading that single unit (Settled Item 19) ⇒ core reverse mapping already resides in item 1 (units), landing in D1 (Data Mobility / Reverse Index) Settled Item 2.

**Scope**: Covers only one half. Under sharing (snapshots / reflink / deduplication), a unit may have multiple referencers that fixed-length unit headers cannot enumerate ⇒ "where multi-referencer data resides and its form" answered by Settled Item 6, outside this item's scope. All reverse index consumers fall within `.claude/rules/fs-design.md` Tier 3 (unconstrained): Tier 1 explicitly restricts "allocation, ENOSPC acceptance, defer windows, lifecycle judgments", which exclude reverse indexing.

**Evidence**:

- E109 (Runtime Costs of Three Location Authority Arms): Under hybrid position authority, cost of relocating shared unit is independent of referencing tree count, whereas pointer authority scales linearly — housing core reverse mapping inside units suffices without separate tree.
- User orientation 2026-08-31, ruled 2026-09-06, original wording in change history.

**Owed checks**: None.

#### Settled Item 2: Upper Bound is Artifact of Mount-Time Judgments, Not Prior Constant

**Ruling**: No hardcoded byte constant established. Line owners declare N in system configuration (may be 0); mount time uses judgment suite answering "mountable / unmountable", not "runs slower" (Settled Item 16). With Settled Item 4 settling "self-authenticating units do not carry", that judgment is removed, leaving three:

| Refusal Reason | Threshold (E43 (Extension Point Byte Upper Bound) Measured) |
|---|---|
| Payload squeezed to 0 | 4 KiB unit **3990**, 128 KiB unit **130966** |
| Node cannot hold single pointer | 2 KiB node **1944**, 64 KiB node **65432** |
| Tear isolation fails (multiple slots in single atomic unit) | Independent of N, judges slot width |

Self-authenticating unit judgment deleted (evidence: Settled Item 4 settles "self-authenticating units do not carry") ⇒ tightest threshold relaxes from journal record header tier (417 on 512) back to index node tier 1944, widening by an order of magnitude. Tree height excluded from judgment: E43 (Extension Point Byte Upper Bound) 864 threshold is soft reference guideline; tree height is derived state performance attribute, not causing mount failure; 16 KiB node with 128 bytes ⇒ fanout 408→404, tree height unchanged.

**Scope**: Answers "N is so large it breaks core layer", not "N is too small to be useful", which line owners judge independently. Table thresholds computed against unit header widths contemporary to E43 (Extension Point Byte Upper Bound); header width revisions require recomputation (C89 (Leftover Recomputation from Closing the Unit Header)); 408 fanout computed with old pointer widths from E29 (Blast Radius of a Single Failed Node) parameters; current node pointer is 86 (D19 (Block Pointer Structure & Width Budget) Settled Item 11); −1.0% ratio unadjusted for new widths. Identical declaration may yield differing verdicts on distinct devices (512 slot width declaration passes on 512 atomic width but triggers `reject_slot_shares_atomic` on 4Kn) ⇒ image mountability depends on host hardware; mount refusal constitutes safe failure mode, though resembling failure in field.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Thresholds for four refusal reasons delimited, none establishing binding upper bound — upper bound must emerge as an artifact of judgments, not prior constants.
- User settlement (2026-08-31): Question reframed from "what byte count upper bound takes" to "which worst-case scenario in core layer this upper bound must protect", original wording in change history.

**Owed checks**: C89 (Leftover Recomputation from Closing the Unit Header): E43 (Extension Point Byte Upper Bound) thresholds recomputed with reservation-inclusive header widths.

#### Settled Item 3: Pointer 7 Bytes Measured in Bytes, Stores Offset + 1, All-Zero Means Empty

**Ruling**: ① Single pointer 7 bytes, unit in bytes (intra-quota offset, E43 (Extension Point Byte Upper Bound) lowest bound tier: each pointer 7 bytes, 128-byte quota holds 18; 2 bytes in bytes cannot address 128 KiB units, 16 KiB granularity option discarded); ② "Empty" encoding: field stores "intra-quota offset + 1", all-zero denotes empty — 0 byte overhead, decoding costs single subtraction, avoiding conflict where "offset 0 is valid target"; ③ External investigation in [prior-art.md](../prior-art.md) Section 9: ext4 / XFS / btrfs / ZFS / APFS all employ "bounded space in record + external pointer when overflowing", empty indicated by count 0 rather than all-zero pointer, aligned with ②; none stores single pointer, burden of proof on SingleFS (D1 (Data Mobility / Reverse Index) Settled Item 3 and Settled Item 17 Hard Constraint 5).

**Scope**: Those 7 bytes designate intra-quota offsets, not placements in D19 (Block Pointer Structure & Width Budget) Settled Item 4 coordinate system (device 4 + 16 KiB slot 6 = 10 bytes, exceeding 7); per Settled Item 17 Hard Constraint 5, expressing core coordinate system is unnecessary. Length field omitted; quota span implied by system configuration declared N and extension point origin (Settled Item 8); extension point empty in version 1 — SingleFS has no active line owners ⇒ zero bytes allocated within extension points. Semantic half (either empty or single pointer, no inline data) authoritatively recorded in D1 (Data Mobility / Reverse Index) Settled Item 3.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Byte-measured pointers land in lowest bound tier; 2 bytes in bytes cannot address 128 KiB units.
- User settlement (2026-09-13), original wording in change history; external investigation [prior-art.md](../prior-art.md) Section 9.

**Owed checks**: C21 (Extension point pointer escaping accounting): Judgment ② checks decoded value (field − 1) falls within declared quota span of this unit, treating all-zero as empty and skipping.

#### Settled Item 4: Data Units Carry, Index Nodes Conditionally Carry, Self-Authenticating Units Do Not Carry

**Ruling**: Evaluated independently across three distinct categories:

| Category | Ruling | Core Supporting Evidence |
|---|---|---|
| **Data units** | **Carry** | No premise opposes adoption. Has parent pointer carrying checksum ⇒ Settled Item 17 Hard Constraint 1 (falls within verification coverage) satisfied automatically; avoids atomic width conflicts |
| **Index nodes** | **Conditionally carry** | Condition requires "content recomputable solely from data unit extension points and not counted separately in ENOSPC". Cost is non-blocking: 16 KiB node with 128 bytes ⇒ fanout 408→404 (−1.0%), tree height unchanged |
| **Self-authenticating units** (root slots, journal record headers) | **Do not carry** | Lack **parent pointers carrying checksums**; tear judgment width = runtime probed `physical_block_size` (D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Corollary 3). Exceeding quota causes **mount failure** (structural cliff) rather than fanout reduction (performance slope); **no reconstruction path exists** — self-authenticating unit corruption is crash recovery issue, not scan rebuild issue |

Index node condition formulated as Hard Constraint 6 (following the five rules of Settled Item 17):

> **Index node extension points may host only derived caches recomputable solely from data unit extension points, and shall not be counted separately in ENOSPC acceptance.**

Rationale: Extension point quota is authoritative state (enters ENOSPC acceptance), whereas index nodes are derived state; placing an uncomputable authoritative value into a reconstructible container de facto elevates it to authoritative state, which is absent from authoritative state inventory ⇒ "index reconstructible from units" and "nodes carry non-reconstructible authoritative bytes" cannot both hold true. Observable failure mode is post-reconstruction quota accounting mismatch ⇒ false ENOSPC.

**Scope**: D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) "units make no distinction between metadata and data" constrains format uniformity (shared unit header, shared self-contained fields), not "every class must carry extension points" — the latter is functional configuration, the former identity; generalizing former into latter is misapplication. Hard Constraint 6 is not verbatim Settled Item 10 criterion: its source set is "data unit extension points", far narrower than "authoritative state". Class 3 (packed record unit) unjudged in this item — per criterion for data and self-authenticating units (presence of parent pointer carrying checksum), it possesses one ⇒ should carry, but this item defers ruling; Class 3 ruling settled in Settled Item 8.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Among four refusal reasons, self-authenticating unit criterion differs in specification (demanding header fits without requiring 56-byte named item remainder), pressing tightest upper bound to self-authenticating tier; settling "do not carry" relaxes tightest upper bound to index node tier.
- Three-way argument (2026-08-31, three legs): Forward leg supplies "conditionally carry" criteria, backward leg adds "not counted separately in ENOSPC", local leg "do not carry" rejected while identified vulnerability adopted (post-reconstruction quota mismatch).
- User settlement (2026-08-31), original wording in change history.

**Owed checks**: C108 (Extension Point Ownership Has No Verdict for Packed Record Units).

#### Settled Item 5: Allocation Records Belong to Derived State

**Ruling**: Allocation record tree in D3 (Space Allocation) Settled Item 3 is entirely reconstructible from authoritative state and does not enter authoritative state inventory. Authoritative state remains "units + accounting + root", without inventory expansion.

Does not conflict with D3 (Space Allocation) Settled Item 1 "authoritative recording point", answering distinct questions:

| Question | Answered By | Measurement Basis |
|---|---|---|
| Querying "whether this placement belongs to active version" at runtime | Allocation record tree (D3 (Space Allocation) Settled Item 1 verbatim "authoritative recording point") | **Runtime measurement basis**: on-the-fly scanning forbidden; immediate answer required when queried |
| Entire tree lost; whether recoverable | Recoverable; reconstructed from units + accounting + root | **Recovery measurement basis**: state classification |

Four consequences:

| # | Consequence |
|---|---|
| 1 | Format **arbitrarily modifiable** (Settled Item 12), excluded from D15 (Format Freeze Policy) Layer 2 freeze |
| 2 | Must satisfy Settled Item 11: **derived state corruption must be detectable**. If allocation record corruption exhibits undetectable staleness, "loss merely degrades performance" fails |
| 3 | **Cannot serve as witness.** In D18 (Information Carried in a Block) Settled Item 5 `map_provenance` Level 2 promotion, witness originally designated as allocation record; derived structures requiring reconstruction cannot witness ⇒ Level 2 promotion requires alternative witness or reopening. Level 1 and Level 0 unaffected |
| 4 | **Relocation forwarding information expelled from allocation record tree.** "Where block originally at this placement moved" is historical event; scanning observes only current state, unable to reconstruct history ⇒ forwarding information either resides in authoritative state (constructed as units, sharing mechanism with D18 (Information Carried in a Block) Settled Item 10 tombstones) or omitted entirely (reverting to "modifying K snapshot trees" or "pinning old placements"). Placed in D26 (Background Compaction and Placement Reclamation) Settled Item 5 |

**Scope**: "Entirely reconstructible from authoritative state" holds not due to word "tree", but because tree content recomputable solely from authoritative state (Settled Item 10); citing it requires retaining this qualification. Unrecoverable byte `release_generation` in tree (added per C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) Ruling P4, segment table in D3 (Space Allocation) Settled Item 7 sets 8 bytes, matching width and domain of D22 (Synthesizing Unit Atomicity) Settled Item 7 `checkpoint_txg`) passes under Settled Item 10 semantic criteria: consumers require only answer to predicate "release_generation ≤ root ring oldest root checkpoint_txg"; reconstruction treats it as latest generation, holding placements for extra K generations before reclamation without incorrect reads; forwarding information failure causes silent stale reads, requiring expulsion. Byte-level criteria would cause conflict, justifying rejection of byte-level criteria. C73 (Allocation record state classification unsettled) resolved by this item.

**Evidence**:

- E59 (Can Buffered Messages Recompute Solely from Units): With units alone, multiple readable copies of single key fail to distinguish active value, failure count matching overwrite count; with accounting added, divergence across four message classes drops to zero — allocation record reconstruction source matches buffered message reconstruction source, with accounting in authoritative state.
- User settlement (2026-09-03 state classification, 2026-09-09 semantic criteria), original wording in change history.

**Owed checks**: C73 (Allocation record state classification unsettled): Level 2 promotion witness alternative found or reopened.

#### Settled Item 6: Under Sharing, Referencer List Requires No Structure, Count Handled by D5 (Snapshot / Space Accounting Mechanism)

**Ruling**: Core reverse mapping resides in units (Settled Item 1), while fixed-length unit headers cannot enumerate multiple referencers (Settled Item 19) ⇒ remaining half divides into two distinct problems:

| Half | Question Answered | Ruling |
|---|---|---|
| **Referencer list** | "Who is referencing me" | **Requires no new structure.** D19 (Block Pointer Structure & Width Budget) Settled Item 5 adopts hybrid position authority; moving shared unit modifies single central mapping entry, **requiring no knowledge of referencers** ⇒ sole consumer disappears |
| **Shared count** | "Does anyone reference me besides birth owner" | **Outside this item's scope**: belongs to D5 (Snapshot / Space Accounting Mechanism) Premise 3 (removing one reference when second live reference exists is not death); housed in closed **sparse side-table** of D5 (Snapshot / Space Accounting Mechanism) (verbatim "D5 primary + sparse side-table secondary"), field table settled in D5 (Snapshot / Space Accounting Mechanism) Settled Item 6 (mapping key 27 + count 4) |

**Scope**: Reflink remains blocked post-ruling, stalled on count half — D5 (Snapshot / Space Accounting Mechanism) verbatim requires "any feature introducing second live reference to same block must reopen" D5, and reflink / deduplication never triggered reopening clause (triggered by D6 (Snapshot Implementation Model) Settled Item 1 cross-head sharing) ⇒ first reflink requires reopening D5 (Snapshot / Space Accounting Mechanism), expanding side-table scope from cross-head sharing to reflink. Candidate space retains unexcluded, unadopted alternative: no structure built, traversing by placement before batch relocation — Settled Item 1 assigns consumers to `.claude/rules/fs-design.md` Tier 3 ⇒ valid; under hybrid authority it lacks consumers, unneeded, but serves as ready answer if reverting to pointer authority, blocked only by race condition (between scanning and creating shared references), lacking mutual exclusion or conflict detection in repo. Cost of hybrid authority lies not in relocation / publish / read, but in two consistency hard rules measured in E96 (Consistency of Hybrid Architecture) (unit header or hint must carry block birth, frees routed through mapping rather than hint). Does not alter new-pool file creation bytes: initial runnable target lacks snapshots, reflink, and deduplication ⇒ zero shared units exist ⇒ entry count is 0.

**Evidence**:

- E109 (Runtime Costs of Three Location Authority Arms): Relocation, publish, and read costs measured across three position authority arms — under central mapping authority, moving unit has constant cost independent of referencing tree count; hybrid arm performs well across all three dimensions, enabling referencer list consumer to vanish entirely.
- E52 (Operational Costs of Two Multi-Writable-Head Mechanisms): Column in sparse side-table verbatim titled "Side-table entries (cross-head shared blocks)" — side-table records cross-head sharing, not reflink / deduplication.
- Tripartite rulings: Output in `research/prompts/d21-share-r1-{sonnet,opus,local}-output.md` and `research/prompts/d21-share-r2-{sonnet,opus,local}-output.md`; user settlement (2026-09-06), original wording in change history.

**Owed checks**: C142 (compaction after first reflink creates silent stale read); C177 (Three unreviewed sites unlocked post-C34 invalidation).

#### Settled Item 7: Space Within Extension Point Quota Consists of SingleFS Units

**Ruling**: Quota space obtained by line owners via extension points consists of SingleFS units, not raw bytes.

Resolves Settled Item 3 ①: verbatim asks "whether requiring equivalent integrity to core position entries" — core position entries are pointers carrying **target** checksums, querying target integrity, while "what target is" was never defined. Structured as units, targets inherently carry unit headers, checksums, and MACs, routing integrity through existing core mechanisms ⇒ 7-byte pointer requires no redundant checksum, keeping width intact.

Simultaneously closes measured vulnerability: flipping byte within quota space previously triggered no failure repo-wide — host unit MAC and payload CRC do not cover it (outside host 32768 bytes), position entry 4 bytes cover host placement, scrub / scan rebuild / rescue explicitly skip it per Settled Item 17 Hard Constraint 3, checker ignores content per Hard Constraint 2, [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) contains zero references to extension points. Matches failure mode prohibited by Hard Constraint 1 "falling outside ⇒ unmonitored corruption zone".

**Scope**: Does not conflict with Hard Constraint 5. Hard Constraint 5 prohibits: "object gains reference unknown to core layer ⇒ D5 (Snapshot / Space Accounting Mechanism) `birth`/`death` accounting fails (computes root reachability, while reference lacks root path)"; quota units are not core-layer objects: bound to host units per Settled Item 17, independently unreachable from any root, generating no second reference ⇒ "structured as units" describes format, "core-layer object" describes reachability identity, representing distinct concepts. Two owed debts alter no new-pool file creation bytes (N = 0, zero line owners): ① which class code quota units use — D18 (Information Carried in a Block) Settled Item 11 registry lists only Codes 1 / 2 / 3, quota units being neither data units nor index nodes; ② whether entering scan rebuild claiming rules — entering requires identity section for claiming, omitting causes scan to classify them as orphans.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Pointer width measured by intra-quota offset; structuring targets as units obviates redundant checksums, preserving width tier.
- Three-way argument (2026-09-09) backward leg measured byte flip in quota span producing no repo failures, verified by main agent; user settlement (2026-09-09), original wording in change history.

**Owed checks**: C221 (Class code and claiming rules for quota units unsettled).

#### Settled Item 8: Class 3 Carries Extension Point; Extension Point Abuts Unit Tail, N Resides in System Configuration

**Ruling**: Class 3 (packed record unit) carries extension points, sharing judgment criterion with data units (possession of parent pointer carrying checksum, Settled Item 4). N is pool-level quantity declared by line owners in system configuration (Settled Item 15), not unit field, with N = 0 in version 1; system configuration geometry segment carries 4-byte "extension point declared value N" (D22 (Synthesizing Unit Atomicity) Settled Item 9), written as 0 in version 1. Extension points abut unit tail, growing backward from end, keeping payload origin invariant to N.

**Scope**: Governs only Class 3 adoption, position within unit, and quota addressing; pointer byte measurement basis in Settled Item 3, intra-quota space form in Settled Item 7, size upper bound in Settled Item 2. Index node adoption remains conditional per Settled Item 4.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Positive payload constraint computed as "unit size − unit header − 1"; extension point growing backward from end keeping payload origin invariant ensures constraint holds across all unit size classes.
- User settlement (2026-09-13 pre-coding audit settled Class 3 and positioning; 2026-09-14 settled system configuration 4 bytes), original wording in change history.

**Owed checks**: C108 (Extension Point Ownership Has No Verdict for Packed Record Units).

#### Settled Item 9: Authoritative State = Units + Accounting + Root; Index is Derived State

**Ruling**: On-disk entities divided into two classes:

| Class | Contents | Properties |
|---|---|---|
| **Authoritative state** | **Units + accounting + root** | Loss causes data loss. Participates in format freeze as permanent contract |
| **Derived state** | **Index (tree)** | Loss merely degrades performance. **Excluded from format freeze** |

"Root" includes indirection layers: root record + tree table units + instance table units — all three traversed from checkpoint tree root pointer per I-7.2 (Latest Root Fully Traversable); not indexes, loss causes data loss. Authoritative state inventory contains three items, without expansion.

Purchased properties:

| Status Quo | Post-Demarcation |
|---|---|
| D15 (Format Freeze Policy) Layer 2 freeze (pointer field layout) is permanent contract | Demoted to **local cache format**; switching to distinct tree requires no backward compatibility |
| D12 (Target Media) N lines share single tree with distinct layouts | N lines **employ distinct index structures**, sharing only unit format |

⇒ D12 (Target Media) "optimizing aggressively for specific media" sits cleaner on derived state side than layout side.

**Scope**: Classification governed by Settled Item 10 recomputability, not word "tree"; allocation records belong to derived state (Settled Item 5), core reverse mapping belongs to item 1 (Settled Item 1); deadlist entries and pinned unit mapping entries are authoritative state (loss unrecoverable, causing pinned units to orphan), while destruction queue (Code 14 entries marked destroying) is derived state. If both instance table copies unreadable, zero rows exist, causing all units across old instances to fall into "no rows ⇒ published", encompassing timelines abandoned by crash recovery: mount read-only, stop reconstruction at Level 1 with explicit report, avoiding ambiguity and silent pass-through — degradation path re-exposes C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) rather than mitigating it.

**Evidence**:

- E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units): Metadata mapping reconstructed from authoritative side exhibits zero omissions and zero errors, confirming "index is derived state" holds for metadata; data block gap equals total data block count, caused by lack of self-description rather than broken reconstruction path.
- E104 (Current Version Determination in Scan Rebuild): When write sequence + instance table rows + ancestor table + death write sequence + container retirement records + sweep gates all present, across varied scenarios wrong versions / resurrections / dead-judged-alive / ambiguities remain strictly 0; ten ablations each non-zero on target scenarios — root section must include instance table units; discarding them as indexes reverts to ablation arms.
- User settlement (2026-08-28): Trees accelerate lookups while units carry self-contained information, making trees acceleration structures rather than sources of truth; original wording in change history.
- Ruling `research/prompts/m4-snapshot-r1-main-verification.md` Section 3 S9: Deadlist entries and pinned mapping entries are authoritative state; destruction queue is derived state (deduced by forward leg, not attacked by the attacker).

**Owed checks**: None.

#### Settled Item 10: State Classification Judged by Recomputability, Queries Semantics Rather Than Bytes

**Ruling**: Criterion: whether content is recomputable solely from authoritative state, ignoring word "tree". Internal nodes recomputable from leaves ⇒ derived state; leaves hosting uncomputable values (inode tree leaves are Code 3 packed record units, D8 (Core Index Structure) Settled Item 6) ⇒ authoritative state. Accounting is established instance: recomputable offline via traversal, but requiring immediate answers when queried, hence authoritative state. Unit class codes do not determine state classification — Code 2 nodes of accounting trees are authoritative state.

"Recomputable" queries semantics, not bytes. Exact form: when value unrecoverable, whether reader receives conservative answer or wrong answer — conservative answer ⇒ remains derived state; wrong answer ⇒ mandates expulsion from structure or elevation to authoritative state.

| Value | Whether bytes can be reconstructed | What the reader obtains when reconstruction fails | What judgment is rendered |
|---|---|---|---|
| D18 (Information Carried in a Block) Code 2 identity section `birth generation` / `write_sequence` / birth sequence number — node referenced only by live head | No | **Conservative answer**: Rebuilding node from leaves regenerates key segments; parent pointer and mapping entry replaced with new values in same reconstruction pass; reader observes consistent new values | Derived state |
| D18 (Information Carried in a Block) Code 2 identity section `birth generation` / `write_sequence` / birth sequence number — node referenced by live snapshot or clone | No | **Wrong answer**: No reconstruction rule avoids wrong answers without making pool permanently read-only | **Authoritative state**: Loss causes data loss, protected by metadata replication (D18 (Information Carried in a Block) Settled Item 4), not reconstruction |
| `release_generation` in Settled Item 5 tree | No | **Conservative answer**: Treated as latest generation ⇒ placements held for extra K generations before reclamation, avoiding wrong reads | Derived state |
| Relocation forwarding information | No | **Wrong answer**: Silently reads stale pointers | **Mandates expulsion** (implemented per Settled Item 5 Consequence 4) |

⇒ "Uncomputable" does not equal "enters authoritative state"; bridged by "whether value retains consumers, and whether reconstruction failure answers conservatively or incorrectly".

**Scope**: Do not cite this as verbatim Settled Item 4 Hard Constraint 6 — Hard Constraint 6 explicitly states "derived caches recomputable solely from **data unit extension points**", omitting "authoritative state"; it represents narrower instantiation of this criterion for extension points with smaller source set; applying its wording elsewhere silently dilutes criterion. Defeats two proposals: "judging by bytes" (falsely classifies Code 2 identity section as authoritative state, invalidating Settled Item 5) and "applying criterion per field" (field-level byte rules trigger identical false positive). Rationale for Code 2 row claiming "original value has zero consumers" invalid since D19 (Block Pointer Structure & Width Budget) Settled Item 8 (all pointers enter mapping) — write sequence and birth sequence in Code 2 headers are central mapping key segments, read by parent pointers and mapping at runtime; ruling retained, rationale rewritten: keys and referencers updated simultaneously during reconstruction.

**Evidence**:

- E59 (Can Buffered Messages Recompute Solely from Units): Source set separation confirmed — "from units alone" collapses 2 of 4 classes; "from authoritative state" yields zero errors.
- Tripartite rulings `research/prompts/c292-r1-main-verification.md` and `research/prompts/c292-r2-main-verification.md`: Code 2 identity section referenced solely by live head yields conservative answer upon reconstruction (key and referencer updated together); referenced by live snapshot or clone yields wrong answer, requiring elevation to authoritative state.
- User settlement (2026-09-09 semantic criteria, 2026-09-13 split Code 2 into two rows), original wording in change history.

**Owed checks**: C292 (The Reason Why the Code-2 Identity Segment Has Zero Consumers No Longer Holds).

#### Settled Item 11: "Loss Merely Degrades Performance" Requires Prerequisite: Derived State Corruption Must Be Detectable

**Ruling**: Settled Item 9 "derived state (index) loss merely degrades performance" and Settled Item 12 "derived state format modifiable arbitrarily" rely on prerequisite: "system detects corruption whenever derived state fails". If format choice degrades corruption from "detectable unreachability" into "undetectable staleness", both assertions fail; requiring either abandoning choice or establishing detectability mechanism accompanied by checker.

Criterion focal point: To preserve assertion, format choice must answer "whether message in buffer is recomputable solely from units" — yes ⇒ remains pure derived state, mitigated by adding detectability mechanism; no ⇒ de facto elevated to authoritative state, requiring Settled Item 9 inventory expansion and unit-equivalent persistence and crash recovery guarantees. Answered: source is "units (with write sequence) + instance table + ancestor table + tombstone units" ⇒ inventory unexpanded, authoritatively recorded in D11 (Message Buffers in Index Nodes) Settled Item 4, detectability mechanism in D11 (Message Buffers in Index Nodes) Settled Item 3.

**Scope**: Two assertions must not be conflated — "reconstructible" is weak version; "carefree because corruption detected" is strong version; Settled Item 9 proves reconstruction feasibility (unit self-containment ⇒ single scan reconstructs), unproving decidability of reconstruction trigger timing. This item alters no boundaries in Settled Item 9, anchoring when "derived state format modifiable arbitrarily" holds. Specific touchpoint is D11 (Message Buffers in Index Nodes) message buffer: if authoritative value of key resides temporarily in internal node buffer, discarding internal node leaves scan-reconstructed tree with valid headers, checksums, and Merkle chains but stale content (D11 (Message Buffers in Index Nodes) Cost 2, undetected by existing [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) entries).

**Evidence**:

- E59 (Can Buffered Messages Recompute Solely from Units): Narrower inquiry "solely from units" fails under testing, succeeding when recomputing from authoritative state, which includes accounting — corruption pattern "valid headers and checksums with stale content" observed under unit-only arm.

**Owed checks**: C52 (Treating Stale as Valid); C13 (Checker Judgment Ineffective). Independent path established for "recomputing from authoritative state" — gate checker-independence-and-sync cell checker-implementation-disjoint verifies checker and implementation share no code (C12 (shared delta semantics) repaid 2026-09-21, see checks-owed repayment table); correctness of recomputed result tracked under C13 (Checker Judgment Ineffective).

#### Settled Item 12: Derived State Has No "Format Conversion" Operation, Only "Reconstruction"

**Ruling**: No compatibility shims built for index formats, and no converters written. Authoritative state format per D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) represents layer that must not change — format conversions apply only when it changes, which is avoided; derived state format changes arbitrarily because it reconstructs entirely from units. Combining both conclusions: no "old format → new format" conversion problem exists ⇒ "converters" should never be constructed. Constructing converter requires proving conversion correctness, whereas reconstruction requires proving only "reconstructible from units" and "reconstruction is idempotent" — both strictly weaker and easier to verify than conversion correctness.

**Scope**: "Arbitrarily modifiable format" bounded by Settled Item 11 prerequisite (corruption must be detectable). Governs index side; authoritative state format evolution governed by D15 (Format Freeze Policy). Reconstruction path governed by Settled Item 13.

**Evidence**:

- E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units): Mapping reconstructed from authoritative side matches original table entry-by-entry (criterion is entry-by-entry match, not merely producing a table) — reconstruction path viable, eliminating need for conversion path.

**Owed checks**: None.

#### Settled Item 13: Rescue, Tree Replacement, and Migration Share Identical Path

**Ruling**: Units are self-contained ⇒ scanning the entire disk once retrieves their contents. The party performing reconstruction decides what shape of index to rebuild. These three operations are the same code path in the implementation:

| Scenario | Operation |
|---|---|
| Rescue following index corruption | Scan → reconstruct index per line rules |
| Switching to distinct tree | Scan → reconstruct index per new rules |
| **Migration to alternative line (different media layout)** | Scan → **reconstruct per target line rules** (potentially rearranging data layout) |

This path is the sole location in SingleFS requiring backward compatibility, with narrow surface: scan rebuild path must recognize all historically produced unit formats, and nothing more. Normal file system read/write paths need not read legacy images. Cost of "recognizing N unit formats" is N parsers, not N read paths — self-describing units mean scanning requires no anchors: scanning linearly, each unit describes itself.

Two constraints, omitted documentation risking assumption of "universal reconstruction":

1. **Scanning reconstructs indexes, not consistent points.** Scanning discovers all extant, unoverwritten units on disk, including interrupted transaction remnants. Selecting "latest consistent set" requires roots (D16 (Publish Semantics) checkpoints) — **trees are expendable, roots are not.**
2. **When encryption is active, self-description resides in ciphertext.** I-6.2 (plaintext header field allowlist) plaintext header whitelist covers magic / format version / flags / nonce generation / length / MAC, **excluding** logical address, generation, tree ID, owning object — items required by scan rebuild ⇒ **without key, reconstruction impossible, permitting only scrub.** Aligns with D9 (Encryption) trade-off (minimizing leakage surface over moving metadata to plaintext), applied here to reconstruction path.

**Scope**: C10 (Historical unit formats unreadable) guards against "new code fails to read legacy images undetected"; regression target per this item is scan rebuild path reading historical unit formats and reconstructing identical content digests. Input sufficiency for reconstruction tracked in C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) ruling.

**Evidence**:

- E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units): Data blocks lacking self-description exhibit reconstruction gap matching total data block count, vanishing once self-description added — scan rebuild viability depends on unit self-description, not index structure.

**Owed checks**: C10 (Historical unit formats unreadable).

#### Settled Item 14: Extension Point Form: Reserved Space Plus Interface Set, Containing Pointer

**Ruling**: Absolute unit minimalism not pursued. Units reserve space for line owners **alongside an interface set** — structured as extension points rather than opaque bytes: line owners may place a **pointer** inside, using the key for SingleFS-agnostic operations (e.g., faster media-specific positioning). SingleFS permits this, reserving space and interfaces.

**Scope**: Either empty or single pointer, no inline data; semantic half authoritatively recorded in D1 (Data Mobility / Reverse Index) Settled Item 3; pointer measurement basis in Settled Item 3, size in Settled Item 15, upper bound in Settled Item 2, intra-quota space form in Settled Item 7, constraints in Settled Item 17. Pointers forbidden from overlapping core primary fields (aligned with Settled Item 17 Hard Constraint 5).

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Evaluating four refusal thresholds under "contains pointer" form reveals none imposing binding upper bounds — form affordable for core layer.
- User settlement (2026-08-28): Storage is cheapest resource; trading space for performance and safety is worthwhile, embodying D12 (Target Media) "line creator maintains line" in unit format; original wording in change history.

**Owed checks**: None.

#### Settled Item 15: Extension Point Size Declared per Line, Upper Bound Takes Fixed Byte Count

**Ruling**: Extension point size declared per line, not variable per unit — each line declares value in system configuration (line owners decide, may choose 0), with upper bound set by repo. Intermediate option adopted among three alternatives:

| Form | Line Owner | Cost |
|---|---|---|
| Global fixed value | Demands 200 bytes while repo fixes 128; must compromise | None |
| **Declared per line** | **Decides independently** | Near zero: intra-line geometry remains uniform |
| Variable per unit | Maximum flexibility | Each unit incurs length field; **geometry loses uniformity**, scan rebuild and accounting computed per unit |

⇒ Intermediate option is scalable for line owners and fixed for format geometry. Third option useful only if different units on same line require differing sizes, a requirement currently non-existent.

Upper bound takes fixed byte count, not proportional to unit size. Rationale is semantic — hosts per-object semantic fields, independent of unit size. Proportional limits lose control on large units: 1% on 128 KiB unit is **1310 bytes**, **6×** total SingleFS unit metadata (212 bytes, 2-replica tier); fixed 128 bytes is 3.1% on 4 KiB and 0.098% on 128 KiB — higher proportion on 4 KiB reflects "4 KiB units are expensive across all designs" (Settled Item 18 table), not defect of this item.

**Scope**: Specific value answered by Settled Item 2 (no prior constant; derived from mount judgments); N designation in system configuration settled in Settled Item 8.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Thresholds evaluated against declared value judgments depend solely on unit and node sizes and atomic widths, independent of "N as percentage of unit" — proportional limits fail to constrain large units.
- User settlement (2026-08-28 declared per line, upper bound takes fixed byte count), original wording in change history.

**Owed checks**: None.

#### Settled Item 16: Unit Geometry Evaluated Once at Mount Time, Judgments Executed per Device

**Ruling**: Unit geometry evaluated once at mount time, checked at runtime without computation. Upper bound is not hardcoded constant, but suite of mount-time judgments returning "mountable / unmountable", not "runs slower" — embodying `.claude/rules/fs-design.md` "converting misuse into mount refusal". Inputs are quantities invariant during operation: declared size in system configuration, format unit and node sizes, `physical_block_size` probed **per device across pool** — **judgments run per device, any failure refusing mount**.

Four judgments measured in E43 (Extension Point Byte Upper Bound) (covering all four refusal reasons):

| Judgment | Upon Violation |
|---|---|
| Payload must be positive: declared value ≤ unit size − unit header 105 − 1 | Refuse mount |
| Index node must hold at least one pointer | Refuse mount |
| Single atomic unit may hold at most one root slot | Refuse mount |
| Self-authenticating unit header must fall entirely within single `physical_block_size` | Refuse mount |

Per-device judgments apply to declared values, failing for persisted position quantities: root slot start offset is **on-disk fact** fixed at mkfs, unalterable; in heterogeneous pools, partitioning by smaller `physical_block_size` leaves larger disk permanently failing, where "mount refusal is safe failure mode" no longer holds — offsets unalterable, data unretrievable ⇒ **position quantities partitioned by pool `max(physical_block_size)`**, mirroring I-8.1 (Ring Geometry Sufficiently Large) "computed at mkfs, verified at mount"; **per-device judgments govern only declared values.**

**Scope**: Governs unit geometry — unit and node sizes, extension point declared values, atomic widths for self-authenticating units. Excludes stripe width: runtime output of allocation path, whose inputs include write size and writable device set, both absent from input inventory (D2 (RAID Stripe Policy) Settled Item 6). Singular phrasing erroneous: D2 (RAID Stripe Policy) settles disks need not be identical size, D19 (Block Pointer Structure & Width Budget) settles position entries carry dev ⇒ intra-pool geometry heterogeneity valid; treating device-level quantities as pool-level scalars violates C17 (Device Geometry as Global Parameter). Self-authenticating unit judgment deleted per Settled Item 4; table "unit header 105" reflects E43 (Extension Point Byte Upper Bound) contemporary class identity width; current declared length origin is reservation-inclusive 134 (D18 (Information Carried in a Block) Settled Item 16). Three corner cases: four judgments lack one input (root slot start offset; E41 (Root Ring Slot Geometry 2×4×256) proves half-slot offset straddles 2 atomic units, four judgments blind to offset misplacement); identical declaration mounting on 512-byte atomic width may fail on 4Kn, mount refusal representing safe failure mode though appearing as failure in field; mountability becomes machine-dependent predicate rather than format contract; D15 (Format Freeze Policy) freeze judgment table functions over "old reader / new reader", lacking row for "same reader, different devices", requiring addition. Adding disk with larger `physical_block_size` leaves persisted offsets unrecoverable, covered in D2 (RAID Stripe Policy) Settled Item 3.

**Evidence**:

- E43 (Extension Point Byte Upper Bound): Thresholds for four refusal reasons delimited and reachable — judgment structure actionable.
- E41 (Root Ring Slot Geometry 2×4×256): Half-slot offset straddles 2 atomic units, tear isolation judgment incomplete — judgments blind to offset misplacement.
- User settlement (2026-08-30): Unit geometry evaluated once at mount time, checked at runtime without computation; original wording in change history.

**Owed checks**: C17 (Device Geometry as Global Parameter): Discriminative power self-test covers this item; C89 (Leftover Recomputation from Closing the Unit Header): Four judgments recomputed with reservation-inclusive header widths.

#### Settled Item 17: Five Hard Constraints on Extension Points, Lifecycle Bound to Host Unit

**Ruling**: First three apply to opaque bytes; last two unique to "contains pointer" form, without which extension points corrupt accounting and lifecycles.

1. **Must fall within checksum / authentication coverage.** Falling outside ⇒ unmonitored corruption zone, and unauthenticated bytes under encryption, introducing attack surface (D9 (Encryption) AEAD boundaries cannot be breached).
2. **Checker forbidden from making judgments on content.** Content is opaque to SingleFS.
3. **Size must be declared by format** (in line's system configuration, Settled Item 15). Generic tooling (scrub, scan rebuild, rescue) must be able to skip it.
4. **Extension point pointer target space must use SingleFS allocation and accounting.** Line owners forbidden from carving out-of-band space — `.claude/rules/fs-design.md` Tier 1 settles "runtime decision paths forbidden from traversing"; ENOSPC acceptance computes worst-case before entry; uninventoried space invalidates calculations ⇒ target space allocated from **explicit quota**, maintained incrementally at commit; checker verifies "extension point target range recorded as allocated", **even without understanding content**.
5. **Extension point pointers forbidden from pointing to core-layer objects.** Pointers may target only space within extension point's own quota. Strictest rule: permitting core targets ⇒ object gains reference unknown to core layer ⇒ D5 (Snapshot / Space Accounting Mechanism) `birth`/`death` accounting fails immediately (computes root reachability, while reference lacks root path) ⇒ **extension point is closed subgraph, not general reference.**

**Lifecycle bound to host unit**: Host unit death ⇒ space within extension point quota freed concurrently. Unbound lifecycles leave core layer ignorant of reclamation timing, causing definitive leaks.

**Scope**: Hard Constraint 6 (index node extension points host only derived caches recomputable solely from data unit extension points, excluded from ENOSPC acceptance) in Settled Item 4, narrower than Settled Item 10 criterion. Constructing intra-quota space as SingleFS units satisfies Hard Constraint 1 automatically (Settled Item 7); Hard Constraint 5 compatible with "quota space is units", describing format versus reachability identity. Hard Constraints 2 and 3 place extension point content outside checker and generic tooling scope, making Settled Item 7 "quota space must be units" sole realization of Hard Constraint 1.

**Evidence**:

- Three-way argument (2026-09-09, backward leg verified, main agent audited): Flipping byte in quota range when extension point not structured as unit triggered zero repo failures, matching failure mode prohibited by Hard Constraint 1 "falling outside ⇒ unmonitored corruption zone" — observable form provided here, landing in Settled Item 7.
- No experiment: these five rules prevent whoever opens a line from bypassing accounting and lifecycle management; they do not measure a number whose magnitude can be compared. Their individually falsifiable forms are recorded in C21 (Extension point pointer escaping accounting). All three judgments operate on images; measurement becomes meaningful only once they can actually run.

**Owed checks**: C21 (Extension point pointer escaping accounting).

#### Settled Item 18: Byte-by-Byte Comparison Table Only Refutes "SingleFS is Extravagant"

**Ruling**: Sole purpose of table is refuting premise that "SingleFS is extravagant", **establishing no upper bound**. Space conservation is not SingleFS goal, aligned with `.claude/rules/fs-design.md` "do not sacrifice self-containment for space" ⇒ **"comparing frugality" forbidden as judgment criterion.**

**Measurement basis**: Counts solely "overhead bytes incurred to manage single unit" — excluding inodes, directory entries, free space, reverse indexes. External constants verified verbatim (2026-08-28): `btrfs_header` = 32+16+8+8+16+8+8+4+1 = **101** (`BTRFS_CSUM_SIZE 32`, `BTRFS_FSID_SIZE 16` defined in `include/uapi/linux/btrfs_tree.h` and `include/uapi/linux/btrfs.h`); `btrfs_file_extent_item` = 53; `btrfs_item` = 25; `SPA_BLKPTRSHIFT 7` ⇒ ZFS blkptr **128 bytes**; `EXT4_GOOD_OLD_INODE_SIZE 128`. SingleFS rows computed with Code 1 unit header **134** (D18 (Information Carried in a Block) Settled Item 16, including nonce / MAC / algorithm reservations) and pointer header **50** (D19 (Block Pointer Structure & Width Budget) Settled Items 7 / 11).

| System | 4 KiB Unit | 128 KiB Unit |
|---|---|---|
| ext2/ext3 (2001) | 4.1 B (0.101%) | 132.0 B (0.101%) |
| ext4 (2008, with extents) | 12.1 B (0.296%) | 16.0 B (0.012%) |
| btrfs (2007) | 82.5 B (2.014%) | 207.3 B (0.158%) |
| ZFS (2005) | 128.0 B (3.125%) | 128.0 B (0.098%) |
| **SingleFS (2 replicas)** | 212.0 B (5.176%) | **212.0 B (0.162%)** |
| SingleFS (4+2 stripe) | 268.0 B (6.543%) | 268.0 B (0.204%) |

**Allocation of SingleFS 212 bytes** (2-replica row): Unit header 134 + pointer header 50 + position entries ×2 (device 4 + slot 6 + ciphertext checksum 4) 28; 4+2 stripe is 134 + 50 + 14 × 6 = 268.

**Three conclusions**:

1. **"Early file systems were more frugal" is false.** ext2 omits self-description (0 data block bytes), but **lacks extents**, requiring 32 4-byte block numbers for 128 KiB = 132 bytes — same order of magnitude as SingleFS 212 bytes with integrity and encryption (132 vs 212), where 0.101% yields zero checksums, zero self-description, no COW, no snapshots, no authentication ⇒ disparity driven not by "storing maintenance data", but by **reference granularity**.
2. **True watershed is unit size, not era.** All checksummed designs take 2.0–6.5% on 4 KiB units and 0.10–0.20% on 128 KiB units; fixed-byte schemes exhibit **32× difference**.
3. **Table establishes no upper bound** (see "Ruling"). Figures retained, proving single fact: SingleFS shares order of magnitude with peers, without extravagance requiring justification.

**Scope**: SingleFS rows track Code 1 unit header and pointer header widths, recomputed if either changes; proportional extension point upper bound settled in Settled Item 15, independent of table absolute values.

**Evidence**:

- E6 (Encryption Algorithm Selection): Throughput gain of encryption / checksum units saturates beyond 16 KiB — independent rationale supporting Conclusion 2 (watershed is unit size, not era).

**Owed checks**: None.

#### Settled Item 19: Self-Containment Converts Half of Reverse Mapping into O(1)

**Ruling**: Units carry "whom I belong to" ⇒ "which object occupies this physical location" answered by reading single unit, without reverse index tree. Applies to only one half: under sharing (snapshots / reflink / deduplication), unit may have **multiple** referencers that fixed-length headers cannot enumerate ⇒ **reverse index remains necessary with narrowed scope** — from "answering all reverse queries" to "answering shared subset". Relieves burden on D1 (Data Mobility / Reverse Index): list half closed by Settled Item 6 (no new structure needed); count half handled by D5 (Snapshot / Space Accounting Mechanism) sparse side-table (D5 (Snapshot / Space Accounting Mechanism) Settled Item 6).

**Scope**: 5-tuple "whom I belong to" per D1 (Data Mobility / Reverse Index) Settled Item 5 designates birth owner only, answering "which object occupies this location", unable to answer "who else references it"; latter two halves addressed in Settled Item 6 and D5 (Snapshot / Space Accounting Mechanism).

**Evidence**:

- E109 (Runtime Costs of Three Location Authority Arms): Under hybrid position authority, relocating shared unit is independent of referencing tree count; "who references me" consumer vanishes, narrowing reverse index scope.

**Owed checks**: None.

### Interactions with Other Decisions

| Decision | Interaction Point |
|---|---|
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | Settled Item 9 is direct corollary of D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment): load-bearing surface is units ⇒ trees are not load-bearing surface ⇒ trees are derived state |
| D15 (Format Freeze Policy) | Freeze exit evaluated per **structure**, not per layer: state classification determined per structure (Settled Item 10); in Layer 4, Code 2 nodes of accounting tree are authoritative state while internal nodes of other trees are derived state, leaving no layer homogeneous — unit of freeze exit is single structure, four-layer diagram unadjusted; C10 (Historical unit formats unreadable) regression target shifted to **scan rebuild path** (Settled Item 13). Settled Item 16 owes row in freeze table for "same reader, different devices" |
| D12 (Target Media) | N lines employ distinct index structures; extension points embody "line creator maintains line" in unit format |
| D16 (Publish Semantics) | **Trees are expendable, roots are not** — D18 (Information Carried in a Block) settles "lost writes cannot be caught by block content"; determining "this is previous version" requires external witness by definition; checkpoint root provides that witness |
| D1 (Data Mobility / Reverse Index) | Reverse index scope narrowed to "shared subset" (Settled Item 19); core resides in D1 (Data Mobility / Reverse Index) Settled Item 2 |
| D9 (Encryption) | Extension points must reside within AEAD coverage (Settled Item 17 Hard Constraint 1); without key, reconstruction impossible, permitting only scrub (Settled Item 13) |
| D26 (Background Compaction and Placement Reclamation) | Relocation forwarding information expelled from allocation record tree, landing in D26 (Background Compaction and Placement Reclamation) Settled Item 5 (Settled Item 5 Consequence 4) |

## Historical Versions

D21 (Boundary Between Authoritative and Derived States) historical entries are consolidated in [decisions-history.md](../decisions-history.md).
