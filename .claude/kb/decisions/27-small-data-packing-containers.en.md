## D27 Small-Data Packing Container —— Settled

[中文](27-小数据打包容器.md) · **English** · [日本語](27-小データパッキングコンテナ.ja.md)

D27 (Small-Data Packing Container) governs how space waste from small objects padding to full units (D4 (Checksum Location) Settled Item 2: 512 B takes 64×, 1 KiB takes 32×, 4 KiB takes 8×) is reclaimed: background compaction moves multiple small objects into a shared container, with write path unmodified by a single byte. Prerequisite is the smallfile row of D25 (Target Workload Priority), which provides this route with a denominator. What it introduces is a unit class, not a separate track: opening no second verification semantics, no second accounting book, and no second recovery semantics; a container remains a unit following the identical whole-unit checksum verification, identical accounting book, and identical recovery path. Thus it does not constitute a "format-level second track" per D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 6—per-slot individual nonce and MAC would be second verification semantics, and after Settled Items 7 and 9 closed that route, this statement is backed by clauses. Nor does it overturn D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 3: inline threshold remains 0, file data never inlined into inodes by a single byte; containers are carriers on the data side, not inodes. Out of scope: packing tails of two files into the same unit on write path (D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 3 judged negative, which stands untouched; Settled Item 10 explains why its rationale does not reach the "moving out" pattern).

### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Form: online packing vs background compaction, and container corruption remedies** | Background compaction moves into container, write path unmodified; containers receive no extra remedy, relying on 2 replicas of D2 (RAID Stripe Policy) Settled Item 6, lost only when both fail **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Demarcation boundaries** | Only objects ≤ 4096 bytes enter containers (4 KiB tier cap 7), larger objects become standalone units per existing rules **Status: Settled.** **Progress: Unimplemented.** |
| 3 | **Unit class code occupied by containers, and fields in class identity** | Takes a new code (content class) from 4–15 of unit class registry, not reusing Code 3; new class establishes its own validity invariant **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Container index placement and key encoding** | No container index opened: release judgment follows point queries in central mapping slot by slot; dead containers discovered by region scans of background compaction **Status: Settled.** **Progress: Unimplemented.** |
| 5 | **Placement of live slot counts per container** | No live slot count maintained **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Compaction trigger criteria and batch size** | Packing follows central mapping key order, one transaction covering multiple of mapping leaves, opening one container per tier; hysteresis follows birth generation; continuous patrol with rate limiting, no watermarks **Status: Settled.** **Progress: Unimplemented.** |
| 7 | **Carrier for slot nonce and MAC** | Single MAC per whole container, no per-slot individual nonce and MAC **Status: Settled.** **Progress: Unimplemented.** |
| 8 | **Loading disciplines adopted on data side** | ①③ adopted, ② rejected, ④ fixed-width records retained—fixed slot width, object padded to slot width **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **AAD composition and domain separators for containers** | AAD binds the container identity, following the approach of code 3 by adopting four fields: birth tree, packed record type, container sequence number, and container birth generation **Status: Settled.** **Progress: Unimplemented.** |
| 10 | **Slots remain COW** | In-place mutation of any slot within written container strictly forbidden; if an item requires in-place mutation, abandon D27 (Small-Data Packing Container) entirely rather than relaxing this rule **Status: Settled.** **Progress: Unimplemented.** |
| 11 | **Scope: captures space waste of consecutive small packets only** | Only captures space waste of consecutive small packets, avoiding changing pipelines or introducing complex loops; "extra step" measured by I/O count and indirection levels **Status: Settled.** **Progress: Unimplemented.** |
| 12 | **Whether size class table enters format** | Excluded from format: format layer commits only to container header declaring slot width `W` and slot count; available `W` choices belong to compactor policy **Status: Settled.** **Progress: Unimplemented.** |
| 13 | **Size class table selected for version 1** | Geometric ratio by slot width: `W_min` = 64, common ratio 1.125, trailing tier pinned at 4096, 36 tiers total; policy, not format **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Form: online packing vs background compaction, and container corruption remedies

**Settlement**: Writes follow existing rules forming standalone units (padding behavior of D4 (Checksum Location) Settled Item 2 unchanged); after object write finishes, resident compaction of D26 (Background Compaction and Placement Reclamation) moves it into a shared container. Container count = ⌈N / container capacity⌉, independent of publish cadence; criterion is "how large this **already written** object is", branch variables not existing during operations, automatically satisfying Hard Requirement 5 of `.claude/rules/fs-design.md`. How much lost when container corrupts, what remedies compensate: no extra remedy, relying on 2 replicas of D2 (RAID Stripe Policy) Settled Item 6 (w ≥ 2, zero-redundancy stripes forbidden from issuance)—gain multiplier and permanent data loss radius multiplier are the exact same number (both equal container capacity `cap`), lost only when both replicas fail. Costs knowingly accepted: objects land on disk twice (written bytes = existing × (1 + 1/container capacity)); zero space saved before compaction finishes (peak usage = existing padded usage); an unreadable container = all objects in that container lost together, isomorphic to C100 (Inode Container Loss Is Permanent With No Redundancy Clause Covering It) but heavier on data side—indices can be rebuilt from units, while user data has no upstream; compaction itself writes bytes.

**Scope**: Governs where small objects reside after writing and remedies when containers corrupt; write path unmodified by a single byte. Gain multiplier holds only under compaction ordered by central mapping key sequence per Settled Item 6 (net gain across all three tiers is negative when compactor orders by container-filling sequence).

**Basis**:

- E114 (Overall Accounting of Small-Data Packing Containers): Online packing gains hinge on how many small objects land in a single publish, showing negative ledger when publishing per write; background compaction form gains are independent of publish cadence, at the cost of objects landing on disk twice and saving no space before compaction completes.
- E149 (Costs of Three Repair Options for Broken Packed Containers): Multiple replicas do not alter loss radius; lowering cap reduces gain and radius together; abandoning packing sets both to 1; without extra remedy, gain and radius both equal cap, lost only when both replicas fail.
- User settlement: Verbatim words in changelog.

**Owed**: C186 (arm added mid-run falls outside pre-run failure clause scope): `packbg` arm producing this item falls outside any pre-run hardcoded failure clauses; C174 (Three columns of smallfile row unmeasured): coverage denominator; C202 (idle overhead and hysteresis K of packing patrol lack values); C391 (small-data packing container owes D25 proof): D25 (Target Workload Priority) states "a permanent format branch serves only a secondary workload, requiring independent justification under `.claude/rules/fs-design.md` branch criteria", with five hard requirements of `.claude/rules/fs-design.md` also on proof checklist.

#### Settled Item 2: Demarcation boundaries

**Settlement**: Only objects ≤ 4096 bytes enter containers, larger objects forming standalone units per existing rules. 4 KiB tier cap 7: space gain 6.91×, write byte ratio 1.1429, losing 7 upon container failure. 16 KiB cell is a hard boundary, not a tradeoff: container cannot accommodate a second object ⇒ zero space saved while written bytes double. The 4 KiB boundary is chosen between "coverage" and "gain"; 8 KiB tier (2.98×) remains positive but collapses significantly.

**Scope**: Numbers computed per E114 (Overall Accounting of Small-Data Packing Containers) geometry (container header 107, full self-describing per slot 47, w = 2); new class lacks field tables for header width and slot width; breakeven threshold shifts with replica count w. Changing boundary to 2 KiB does not change how 2 KiB objects pack, only turning 2–4 KiB segment into radius 1 standalone 32 KiB units.

**Basis**:

- E114 (Overall Accounting of Small-Data Packing Containers): Background compaction gains collapse with object size, remaining positive at 8 KiB tier, while 16 KiB tier writes twice needlessly without saving space.
- E116 (Packed Container Balance: Metadata Writes and Compaction Strategy): Breakeven achieved if and only if `cap > w + 1`; at w = 2 threshold falls precisely at 8 KiB, supporting 4 KiB boundary by closed form, not merely gain curve collapse.
- E149 (Costs of Three Repair Options for Broken Packed Containers): 2 KiB boundary only turns 2–4 KiB segment into radius 1; cap ceiling restricts only 512 B and 1 KiB tiers, having no effect on 4 KiB boundary.
- User settlement: Verbatim words in changelog.

**Owed**: C361 (New packing container class lacks field table for header width and slot width): recompute cap and gains once field tables settle; C174 (Three columns of smallfile row unmeasured).

#### Settled Item 3: Unit class code occupied by containers, and fields in class identity

**Settlement**: Takes a new code (content class) from 4–15 of unit class registry, not reusing Code 3. Registry codes 4–15 verbatim "reserved for future content classes" (D18 (Information Carried in a Block) Settled Item 11), and containers hold file data ⇒ it is a content class. New class establishes its own validity invariant, without adopting I-1.7 (packing container validity and judgment order).

**Scope**: Settles solely "which code it occupies" and "establishing its own validity invariant without adopting I-1.7 (packing container validity and judgment order)"; fields in class identity segment unfinalized—Settled Items 7 and 9 settle single MAC per container and AAD composition, without providing field table; specific code in 4–15 unpinned. Reusing Code 3 collides with C154 (stripe member table lacks registration of packed record type): stripe membership table unofficially occupies an unregistered type code.

**Basis**:

- No experiment: Which code it occupies is registry categorization, reasoned by three clauses—reusing Code 3 either reinterprets existing fields in container header or modifies judgment chain of I-1.7 (packing container validity and judgment order), both falling into out-of-scope table of Settled Item 11; registering fixed-width types per size tier under Code 3 second-level registry requires padding objects to tier width (600 B padded to 1024 wastes 41%), reintroducing waste that D27 (Small-Data Packing Container) aims to eliminate; incompat cost identical between paths (D18 (Information Carried in a Block) Settled Item 11 verbatim "new unit class ⇒ incompat; new packed record type defaults to incompat"), saving nothing and providing no justification for reuse.
- Three-way argument: Code and field table round (`_d27-format-crypto-background.md`, cloud legs `d27-format-crypto-forward-sonnet.md`, `d27-format-crypto-reverse-opus.md`) and round 5 (`_d27-r5-boundary.md`, `d27-r5-forward-sonnet.md`, `d27-r5-attack-sonnet.md`, `d27-r5-local.out`).
- User settlement: Verbatim words in changelog.

**Owed**: C361 (New packing container class lacks field table for header width and slot width); C362 (Code in registry unpinned for new packing container class).

#### Settled Item 4: Container index placement and key encoding

**Settlement**: No container index opened. Settled Item 11 burden of proof for new structures is "prove that without it D27 (Small-Data Packing Container) cannot hold", which cannot be proven here: no runtime path queries "how many live slots remain in this container"; completely dead containers discovered when background compaction sweeps that region—target state of compaction intent in D26 (Background Compaction and Placement Reclamation) Settled Item 2 verbatim "within `R` there exists no placement with allocation gen ≤ `c0` and unreleased" ⇒ compactor enumerates placements by region; release judgment follows point queries in central mapping slot by slot (D19 (Block Pointer Structure & Width Budget) Settled Item 5: central mapping is sole entry point for dereference and release judgment); residing in tier three of accounting discipline (background, resumable, non-decision path, no commandments)—`.claude/rules/fs-design.md` verbatim names "compaction", D26 (Background Compaction and Placement Reclamation) text similarly verbatim "compaction belongs to tier three, **no commandments**—can traverse, can be slow".

Dereferencing requires no new pointer: slots carry logical identity 5-tuple (D1 (Data Mobility / Reverse Index) Settled Item 5 requires self-contained immutable data carry single pointer), while central mapping key is another group of fields (D19 (Block Pointer Structure & Width Budget) Settled Item 6: Code 1 key = class tag 1 + birth tree 8 + birth txg 8 + write sequence 10, total 27 bytes) ⇒ position entry points to container placement as usual, claiming slot within container by central mapping key after reading full unit—key segments in slot match pre-packed unit header byte by byte, class tag constant Code 1 (D19 (Block Pointer Structure & Width Budget) Settled Item 6 tightening ⑩㈠); "whether this is container or standalone unit" answered by unit's own class tag (I-1.6 (class tag registered and equal in three places)). D19 (Block Pointer Structure & Width Budget) Settled Items 3 / 4 / 5 zero modifications, mapping value adding no discriminator bits. ⇒ Everything added by D27 (Small-Data Packing Container) consists of two items only: one unit class with its field table, and one additional branch in background compaction scanning; central mapping, allocation records, and accounting keys untouched by a single byte.

**Scope**:

- Reclamation timing has no upper bound: dead container awaits compactor sweeping its region; when compaction runs or halts is settled by D26 (Background Compaction and Placement Reclamation) Settled Item 1 (D26 (Background Compaction and Placement Reclamation) Settled Item 4 grounds policy half in D3 (Space Allocation) Settled Item 8 and D26 (Background Compaction and Placement Reclamation) Settled Item 1); during this time those 32768 bytes remain allocated without references, homologous to "unreclaimable bytes" watermark in D26 (Background Compaction and Placement Reclamation) Settled Item 1, not newly introduced by D27 (Small-Data Packing Container).
- Costs accurately recorded: 4-byte ciphertext checksum in position entry covers full 32 KiB unit (D4 (Checksum Location) Settled Item 1) ⇒ N mapping entries of same container each store identical checksum value; rewriting container modifies all N entries.
- Segments of key in slot specified in settlement paragraph (C288 (collateral changes from mapped key settlement did not land in other decisions) ⑤); remaining fields of slot (slot table, object actual length location, byte layout, slot width) settled alongside unfinalized class identity table of Settled Item 3 (C361 (New packing container class lacks field table for header width and slot width)).

**Basis**:

- No experiment: Load-bearing relies on mechanical census and point check—three legs found zero hits on "container" across allocation, ENOSPC acceptance, defer window, snapshot liveness, `df`; statistics in D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 lack container dimension; attacker leg's "permanent leak" premises "discovery requires index", disproven by region enumeration in D26 (Background Compaction and Placement Reclamation) Settled Item 2.
- Three-way argument: Round 5 (`_d27-r5-boundary.md`, `d27-r5-forward-sonnet.md`, `d27-r5-attack-sonnet.md`, `d27-r5-local.out`).
- User settlement: Verbatim words in changelog.

**Owed**: C361 (New packing container class lacks field table for header width and slot width): slot half.

#### Settled Item 5: Placement of live slot counts per container

**Settlement**: No live slot count maintained. Settled Item 10 excludes it from containers (storing in container requires modifying dead slot flags within container after objects move out); Settled Item 11 burden of proof fails to justify it: no runtime path queries "how many live slots remain in this container", dead containers discovered by region scans of background compaction (Settled Item 4).

**Scope**: Dead slot flags / liveness bits / refcounts strictly forbidden from residing in containers (Settled Item 10); cost is unbounded reclamation timing, detailed in Settled Item 4 scope.

**Basis**:

- No experiment: Same mechanical census as Settled Item 4—zero hits across allocation, ENOSPC acceptance, defer window, snapshot liveness, `df`; D5 (Snapshot / Space Accounting Mechanism) Settled Item 4 accounting statistics lack container dimension.
- Three-way argument: Round 5 (`_d27-r5-boundary.md`, `d27-r5-forward-sonnet.md`, `d27-r5-attack-sonnet.md`, `d27-r5-local.out`).
- User settlement: Verbatim words in changelog.

**Owed**: None.

#### Settled Item 6: Compaction trigger criteria and batch size

**Settlement**: Packing and reclamation are distinct jobs with different traversals—packing (moving small objects into containers) follows **central mapping key order**, dominated by mapping writes; reclamation (releasing fully dead containers) hitches a ride on **region scans** of D26 (Background Compaction and Placement Reclamation), enabling Settled Items 4 and 5 to avoid new structures. Five fixed rules:

1. **Ordering**: Packing traverses central mapping key order, **not container-filling order**.
2. **Batching**: One compaction transaction covers a contiguous key interval, sized to an **integer multiple of mapping leaves** ⇒ **each leaf COWed only once**.
3. **Multi-tier concurrency**: Traversal along key order concurrently maintains one open container per 36 tiers of Settled Item 13, objects routed by tier. **Mapping entries still update in key order** ⇒ tier partitioning does not compromise Rule 2.
4. **Hysteresis**: Move only objects satisfying `current checkpoint sequence − object birth generation ≥ K`. **Zero new fields, zero new tracking**—under COW modifying an object produces a new birth generation, so "age since last modification" equals "age since birth", which already resides in object header.
5. **Start condition**: **Continuous patrol with rate limiting**, no watermarks; zero new states, zero new watermarks, leaving D26 (Background Compaction and Placement Reclamation) Settled Item 1 untouched; conforms to Settled Item 11 "no complex loops".

**Scope**: No dedicated watermark for packing—three watermarks of D26 (Background Compaction and Placement Reclamation) Settled Item 1 (empty clustered segment count / fragmentation / unreclaimable bytes) address fullness, fragmentation, and unreclaimable bytes, none measuring "how much packing can free". **Form** of `K` is fixed as "round-trip cost ÷ single migration gain", while **numerical value requires churn probability**, unmeasured across repo; idling mapping read overhead during patrol when idle unmeasured.

**Basis**:

- E116 (Packed Container Balance: Metadata Writes and Compaction Strategy): Sign of breakeven ratio governed by compaction strategy—following key order COWs mapping leaves once, breaking even; container-filling order with batch 1 scatters leaves randomly, failing breakeven.
- User settlement: Verbatim words in changelog.

**Owed**: C202 (idle overhead and hysteresis K of packing patrol lack values).

#### Settled Item 7: Carrier for slot nonce and MAC

**Settlement**: Single MAC per whole container, no per-slot individual nonce and MAC; D19 (Block Pointer Structure & Width Budget) Settled Item 3 and D9 (Encryption) Settled Item 4 untouched by a single byte. Individual MAC per slot constitutes second verification semantics, while D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 6 expressly forbids format-level second tracks—avoiding second verification semantics, second accounting books, and second recovery semantics.

**Scope**: Narrows pending nonce attribution in C68 (Invariant Allowlist Conflicts with Settled Clauses) from "consequences of two paths" to "consequences of one path"—that cell verbatim cited by Settled Item 7 as "the same hole"; narrowing to one does not equate to paying off. Encryption excluded from initial runnable version (D9 (Encryption) Settled Item 10); this item executes only when encryption is active.

**Basis**:

- No experiment: Semantic tradeoff without measurable quantities—individual MAC per slot creates second verification semantics, colliding with D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 6.
- User settlement: Verbatim words in changelog.

**Owed**: C68 (Invariant Allowlist Conflicts with Settled Clauses): untouched.

#### Settled Item 8: Loading disciplines adopted on data side

**Settlement**:

| Discipline | Adopted on Data Side | Basis |
|---|---|---|
| ① One content type per container | **Adopted** | I-1.7 (packing container validity and judgment order) verbatim "container header declares one packed record type, one birth tree" |
| ② One generation per container | **Rejected** | Discipline ② of D18 (Information Carried in a Block) Settled Item 11 scoped to "generalize to packed record types reclaimed by generation", exempting individually dying types; data-side objects die individually |
| ③ One tree per container | **Adopted** | Same invariant as ① |
| ④ Fixed-width records | **Adopted (retained)** | User settlement |

**④ Fixed form**: A container declares one slot width `W`, all slots within container sharing width, objects padded to `W`; start of slot `i` = header + `i × W`, **requiring no slot directory**; validity judgment isomorphic to I-1.7 (packing container validity and judgment order) (`slot count × W ≤ declared length`), borne by new class's own invariant.

**Scope**: ④ pays two costs: padding waste reappears—objects between tiers padded to tier width, tier partitioning settled by Settled Item 13; compactor routes by size, which does not conflict with "following central mapping key order": compactor traverses key order while keeping one open container per tier, routing objects by tier, updating mapping entries in key order, filling containers by tier. Cost is open container count = tree count × tier count.

**Basis**:

- E119 (Padding Cost of Fixed-Width Slots): Fixed width itself is not expensive; coarse tiering is expensive; reverse clause "every fixed-width arm loses > 30% on every distribution" unfired, ④ un-reopened.
- User settlement: Verbatim words in changelog (main agent suggested slot directory, user chose retaining fixed width).

**Owed**: C106 (upper bound on open container count).

#### Settled Item 9: Container AAD composition and domain separators

**Settlement**: AAD binds container identity, fields following Code 3 practice `(birth tree, packed record type, container number, container birth gen)`—D18 (Information Carried in a Block) Settled Item 11 previously settled this set for Code 3, **not a newly invented scheme**. Following single MAC per container, this cell admits only one formulation:

| Formulation | Why It Fails / Holds |
|---|---|
| AAD binds **object** identity + single MAC per container | **Mechanistically excluded**: N AADs against 1 MAC ⇒ N−1 objects fail verification. Mechanism identical to D9 (Encryption) Settled Item 6 excluding extent splitting, verbatim "mismatch becomes inevitable outcome of normal operation ⇒ excluded" |
| Individual nonce and MAC per slot + AAD binds object identity | Premises route closed by Settled Item 7 |
| **AAD binds container identity** (adopted) | One AAD against one MAC, self-consistent |

**Scope**: Knowingly accept unmeasured cost—background compaction in Settled Item 6 moves objects, and AAD binding container identity implies **when an object with its slot moves to another container, AAD changes, requiring re-encryption**; D27 (Small-Data Packing Container) verbatim noted "cost unmeasured in E114 (Overall Accounting of Small-Data Packing Containers)" upon proposal; settlement does not alter this. Execution path exists only when encryption active.

**Basis**:

- No experiment: One AAD against one MAC being self-consistent is deduced from Settled Item 7 and mismatch mechanism of D9 (Encryption) Settled Item 6, not a new tradeoff.
- User settlement: Verbatim words in changelog.

**Owed**: C240 (uncomputed cost of re-encryption during relocation).

#### Settled Item 10: Slots remain COW

**Settlement**: Slots remain COW, principle inviolable; if violated, abandon. ⇒ **In-place mutation of any slot within a written container strictly forbidden.** This rule is D27 (Small-Data Packing Container) **veto point**: if any subsequent item can be achieved only by "modifying that byte inside the container", abandon D27 (Small-Data Packing Container) entirely, **never relaxing this rule to accommodate it**. Five points fixed immediately:

| Item | Fixed Form |
|---|---|
| Mutability of container | **Immutable once written**. In-place slot mutation, partial container COW, torn slots entirely nonexistent |
| Modifying packed object | Follows **unmodified** standalone unit write path writing new unit ("moving out"), leaving container untouched |
| Settled Item 5 (live slot count location) | **Cannot reside in container**—residing in container requires modifying dead slot flag after object moves out |
| Dead slot flag / liveness bit / refcount | **Strictly forbidden from container**, same rationale as Settled Item 5 |
| How compaction reclaims dead slots | Only **rewriting entirely** (writing new container, releasing old container as a whole), **in-place compaction forbidden** |

"Moving out" and Settled Item 1 represent two applications of the same mechanism; alternative self-consistent form "whole-container COW" suffers worst-case 58× write amplification, rejected.

**Scope**:

- Forms a pair with Settled Item 11: Settled Item 10 governs "no in-place mutation", Settled Item 11 governs "no outward expansion", both being veto points, neither relaxed to accommodate items.
- Answers D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 3 checkpoint. D14 (Dual-Track: Large/Small Files & Persistent/Transient) rejected "cross-file tail packing" because "two files sharing a unit ⇒ writing one COWs entire unit ⇒ **pointer of other file must also change**"; under "moving out", writing one object never COWs container—container untouched, other objects' pointers unmodified ⇒ that rationale fails for this form. Three qualifications, none omissible: ① D14 (Dual-Track: Large/Small Files & Persistent/Transient) rejection stands untouched—it governs packing two file tails into one unit on **write path**, different from "moving into container in background after write completes"; rationale merely does not reach the latter; ② "pointer of other object must also change" holds during **compaction**: compacting or rewriting a container modifies central mapping entries for all N objects (E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) counts this into migration writes); compaction is background resumable path in `.claude/rules/fs-design.md` tier three (no commandments), different from immediate write-path judgment; ③ answers **the verbatim question** of D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 3 ("multiple files sharing an atomic unit"), **and does not answer with performance numbers**—D14 (Dual-Track: Large/Small Files & Persistent/Transient) Settled Item 3 verbatim states "performance numbers cannot answer it".

**Basis**:

- No experiment: this veto-point principle was settled by the user and has no quantity to measure. It gives D27 (Small-Data Packing Container) an explicit, observable condition under which to abandon the approach.
- User settlement: Verbatim words in changelog.

**Owed**: C190 (no check turns red when in-place mutable fields appear in containers): until check arrives, this remains a written principle.

#### Settled Item 11: Scope: captures space waste of consecutive small packets only

**Settlement**: D27 (Small-Data Packing Container) exists for space waste of consecutive small packets, **not rebuilding something that alters pipelines or introduces complex loops**. Candidate forms face these two questions, falling within scope only if answering "no" to both:

1. Does it add an **extra step** to the pipeline of "reading a packed object"?
2. Does it **modify field semantics of any existing clause**?

"Extra step" in Criterion ① is **measured by I/O count and indirection levels, not operation count**:

| | Unpacked | Packed | Out of Scope (container index on read path) |
|---|---|---|---|
| I/O count | 1 | 1 | 2 |
| Indirection levels | 1 | 1 | 2 |
| Structures queried on read | 1 | 1 | 2 |

⇒ Claiming slot by central mapping key within the **buffer already held in hand** after reading container (Settled Item 4) **does not count as an extra step**—issues zero I/O, adds zero indirection, queries zero extra structures. Criterion ① blocks the path of "central mapping → container index → container".

**Within scope**: Adding a unit class (registry codes 4–15 verbatim "reserved for future content classes", occupying one incompat bit)—that is **adding a content class**, not altering pipeline; and field layout of this new class. **Out of scope**:

| Out-of-Scope Action | Rationale |
|---|---|
| Reinterpreting field semantics in Code 3 (packed record unit) container header | Modifies field semantics of existing clause |
| Adding dimension to allocation record value (D3 (Space Allocation) Settled Item 7 fixes `allocation_gen 8`) | Modifies field semantics of existing clause |
| Adding container dimension to accounting key (D5 (Snapshot / Space Accounting Mechanism) Settled Item 1 fixes `(stat_tag, tree_id, dev, gen)`) | Modifies field semantics of existing clause |
| Modifying central mapping value form (adding discriminator bit, switching to identity reference) | Adds extra step to dereference pipeline |
| Modifying I-1.7 (packing container validity and judgment order) step "record width == registered width" | Modifies existing clause; new class establishing own judgment exempt |
| Opening second verification semantics / second accounting book / second recovery semantics | Excluded in D27 (Small-Data Packing Container) opening paragraph |

**Burden of proof on new structures**: Any tree / ring / index D27 (Small-Data Packing Container) seeks to add **must first prove "without it D27 (Small-Data Packing Container) cannot hold"**. Unproven structures forbidden. Directly bears on Settled Items 4 (container index) and 5 (live slot count).

**Scope**: Forms a pair of veto points with Settled Item 10 (Settled Item 10 scope).

**Basis**:

- No experiment: Scope represents boundary drawn by user; taking I/O measurement basis for "extra step" justified because Criterion ① and settlement specify "pipelines" and "complex loops", measuring structures and indirection, not instruction counts.
- Three-way argument: Round 5 (`_d27-r5-boundary.md`, `d27-r5-forward-sonnet.md`, `d27-r5-attack-sonnet.md`, `d27-r5-local.out`), two of three legs judged by operation count; main agent verdict and user settlement selected I/O measurement basis.
- User settlement: Verbatim words in changelog.

**Owed**: None.

#### Settled Item 12: Whether size class table enters format

**Settlement**: Excluded from format. Format layer commits to two things only: container header contains a **slot width `W`** field (boundary ≤ 4096 ⇒ fits in 2 bytes) and a **slot count** field. Reading a slot is `header + i × W`—**readers have zero need to know what size class table looks like**. ⇒ **Which `W` choices are available belongs to compactor policy, not format constants.** Changing size class table leaves format untouched; legacy containers read as usual with self-declared `W`.

**Scope**: Does not buy "arbitrary future modifications": legacy container `W` is written in; changing size class table affects only **newly written** containers; rearranging old containers by new table requires compactor rewriting entirely (Settled Item 10 forbids in-place mutation). Width and position of `W` field fall outside this scope, settled alongside field table of Settled Item 3.

**Basis**:

- E119 (Padding Cost of Fixed-Width Slots): How tiers should partition depends on object size distribution; relative ranking of tables differs across distributions—size class tables can only be chosen per distribution, which remains unmeasured, freezing into format equating to freezing an unknown into permanent commitment.
- E121 (Optimal Points for Tiering by Cap and Tier Count): Optimal tier count also depends on workload scale; more tiers does not equal better.
- User settlement: Verbatim words in changelog.

**Owed**: C174 (Three columns of smallfile row unmeasured).

#### Settled Item 13: Size class table selected for version 1

**Settlement**: Geometric ratio by slot width: `W_min` = 64, common ratio 1.125, trailing tier pinned at 4096, 36 tiers total (choosing the common ratio of a geometric size-class table). **Tier width table (parentheses indicate tier `cap`, namely objects per container)**:

```
  64(305)   72(284)   81(263)   92(241)  104(222)  117(204)
 132(186)  149(170)  168(154)  189(140)  213(127)  240(115)
 270(104)   304(94)   342(84)   385(76)   434(68)   489(61)
  551(54)   620(49)   698(44)   786(39)   885(35)   996(31)
 1121(28)  1262(25)  1420(22)  1598(19)  1798(17)  2023(15)
 2276(14)  2561(12)  2882(11)   3243(9)   3649(8)  4096(7)
```

Policy, not format (Settled Item 12): modifiable after C174 (Three columns of smallfile row unmeasured) paid and workload measured, leaving format untouched; however, **affects newly written containers only**, old containers requiring compactor rewriting entirely (Settled Item 10 forbids in-place mutation).

**Scope**: When distribution and scale unknown, select the table "where no cell suffers significant loss", rather than "one cell optimal". 36 open containers = 1.15 MiB, **multiplied by trees processed concurrently by compactor**, multiplier determined by D19 (Block Pointer Structure & Width Budget) Settled Item 6: key led by birth tree after class tag; recomputation unperformed.

**Basis**:

- E120 (Choosing the Common Ratio of a Geometric Size-Class Table): Worst-case relative waste above `W_min` in geometric table is distribution-independent; floor segment examined by absolute bytes.
- E121 (Optimal Points for Tiering by Cap and Tier Count): More tiers does not equal better; geometric slicing by `cap` uniformly inferior; reducing `W_min` below 64 yields zero gain, raising to 128 incurs significant cost.
- E119 (Padding Cost of Fixed-Width Slots): Four equidistant tiers trigger pre-run failure clauses; tiers must concentrate on small end, not equidistant.
- E116 (Packed Container Balance: Metadata Writes and Compaction Strategy): Breakeven requires `cap ≥ 4`; every tier in this table has `cap` ≥ 7.
- User settlement: Verbatim words in changelog.

**Owed**: C174 (Three columns of smallfile row unmeasured); C106 (upper bound on open container count): open container count multiplied by tree count.

## Historical Versions

Historical entries for D27 (Small-Data Packing Container) are centralized in [decisions-history.md](../decisions-history.md).
