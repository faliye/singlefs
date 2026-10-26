## D1 Data Mobility / Reverse Index —— Settled

[中文](01-数据可移动性-反向索引.md) · **English** · [日本語](01-データ移動性-逆インデックス.ja.md)

D1 (Data Mobility / Reverse Index) governs the **criterion aspect**: whether the system must be able to answer "who references this physical block", what data can enter a unit's self-contained section, whether to reserve extension points, what enters the self-contained section versus extension points, how many pointers self-contained immutable data carries, and how old versions disconnect. Out of scope: which exact fields actually entered, their widths, and classification by unit classes, which are in D18 (Information Carried in a Block); byte layout of extension points (quota ceiling, which unit classes carry them, pointer width and how "empty" is encoded) in D21 (Boundary Between Authoritative and Derived States) settled item 2 / 3 / 4; registry and counts for the multiple-referencers half in D21 (Boundary Between Authoritative and Derived States) settled item 6 and D5 (Snapshot / Space Accounting Mechanism) settled item 6.

### Settled Items

**D1 (Data Mobility / Reverse Index) settled items use a single numbering sequence**, never reordered.

| # | Item | Settlement |
|---|---|---|
| 1 | **What can enter a unit's self-contained section** | Unit data is self-contained; data "characteristic of this unit and immutable" such as identity values can all enter **Status: Settled.** **Progress: Implemented.** |
| 2 | **Whether reverse index counts as self-contained data** | Counts—the body of the reverse mapping is inside the unit, no reverse index tree needed **Status: Settled.** **Progress: Implemented.** |
| 3 | **Whether to reserve customizable extension points** | Yes; and it is either empty or a pointer, no inlined data **Status: Settled.** **Progress: Implemented.** |
| 4 | **What enters self-contained section, what enters extension point** | Separated by "whether it changes with version"—immutable enters self-contained section, what can differ per version enters extension point **Status: Settled.** **Progress: Implemented.** |
| 5 | **How many pointers for self-contained immutable data, and how old versions disconnect** | Unique single pointer, pointing to birth owner; new version written means old version disconnected, disconnection done by tree without modifying old unit **Status: Settled.** **Progress: Implemented.** |
| 6 | **Withdrawn · This number no longer used** | Formerly settled "generational data in extension point is a stack of depth 5, released in batch by gc upon expiration", established and withdrawn same day **Status: Settled.** **Progress: No code object.** |
| 7 | **Whether reverse index is required, when to design into format** | Required, and designed into format day-1, post-hoc patch forbidden **Status: Settled.** **Progress: Implemented.** |
| 8 | **Boundary with D18 (Information Carried in a Block)** | D1 (Data Mobility / Reverse Index) settles criteria, D18 (Information Carried in a Block) settles field table; field table changes when criteria change, field table arrangement does not modify criteria in reverse **Status: Settled.** **Progress: No code object.** |

#### Settled Item 1: Characteristic Immutable Data Such as Identity Values Can All Enter Self-Contained Section

**Settlement**: **Unit data is self-contained.** Anything that is "characteristic of this unit and **immutable**"—identity values being one category—can enter the self-contained section. The default answer is **include**, never trimming fields to save space (`.claude/rules/fs-design.md` "do not sacrifice self-containment to save space").

**Scope**: Governs what **can** enter, not which fields actually entered (D18 (Information Carried in a Block)). ⚠️ **The criterion hinges on the word "immutable", which is more fundamental than existing prohibited lists**: D18 (Information Carried in a Block) settled item 3 prohibits physical placement / device ID from block headers on the grounds that "physical locations never enter AAD ⇒ placing them in block headers makes an unauthenticated, silently tamperable field"; under this item's criterion, **physical placement is not immutable data at all**—blocks get moved (the very reason D1 (Data Mobility / Reverse Index) exists is to enable movement) ⇒ it cannot even get past the door. Both rationales point the same way; this item's is the higher-level one. ⚠️ **One class is immutable yet still forbidden**: filenames and btree internal node separator keys. The reason they are kept out is **not mutability, but leakage surface**—D9 (Encryption) settled item 5 scope verbatim: the candidate "leave plaintext in index internal nodes" was rejected because internal nodes carry separator keys, and dirent tree separator keys are filenames, presenting a far larger leakage surface than ZFS (ZFS indirect blocks are pure pointer arrays with no keys). **Thus that prohibition must stand separately and cannot be overridden by this item.**

**Basis**: No experiment: This item settles the criterion "what can enter" itself, not whether any specific field should be added—byte accounting for individual fields is measured in D18 (Information Carried in a Block) and D21 (Boundary Between Authoritative and Derived States); direction of criterion derives from `.claude/rules/fs-design.md` "do not sacrifice self-containment to save space", which explicitly states it governs whether fields already proven useful should be cut to save space. User settlement, original wording in history.

**Owed**: None.

#### Settled Item 2: Reverse Index Counts as Self-Contained Data

**Settlement**: **The body of the reverse mapping resides in the unit itself**—the unit carries "who I belong to" ⇒ "which object is at this physical location" can be answered by reading only that single unit, without needing a reverse index tree. ⇒ "Reverse index taking authoritative state" is not adding a fourth item to the authoritative list, but **recognizing it is already in the first item (units)**.

**Scope**: **Only one half.** Under sharing (snapshot / reflink / deduplication), a unit can have **multiple** referencers, which a fixed-length unit header cannot enumerate ⇒ the remaining half is not in this item: the list half is settled by D21 (Boundary Between Authoritative and Derived States) settled item 6 (relocation asks no referencers once location authority is hybrid, needing no new structures), while the count half belongs to the sparse side table in D5 (Snapshot / Space Accounting Mechanism) settled item 6. Nor does this item settle what the pointer in the unit header looks like; that is in settled item 5.

**Basis**: No experiment: This item is a conclusion directly read from the format discipline "unit self-describing"—the unit header already contains "who I belong to", adding no on-disk fields nor modifying any widths, with no measurable quantity; what needs measurement is the cost of the multiple-referencers half, which is already outside this item's scope. User settlement, original wording in history.

**Owed**: None.

#### Settled Item 3: Reserve Customizable Extension Points, Either Empty or a Pointer

**Settlement**: **Each unit reserves one customizable extension point**, with **mutually exclusive** semantics: **either empty, or a single pointer**. ⇒ **No inlined data in extension points**. This rounds off the semantics of D21 (Boundary Between Authoritative and Derived States) settled item 3: prior to 2026-09-06, that item settled only "pointers must not overlap with core-layer primary fields" and "length need not be specified", without stating "whether anything other than a pointer can be housed"—settled item 3 stated on 2026-09-06 that it cannot.

**Scope**: Governs extension point **semantics**, not byte layout. Authoritative record of byte layout is in D21 (Boundary Between Authoritative and Derived States), not repeated here: ceiling calculated from mount-time judge group (settled item 2), which unit classes carry it (settled item 4: data unit carried, index node conditionally carried, self-proving unit not carried), the 7-byte-per-pointer tier and its three former owed checks (width and unit, how "empty" is encoded, external research) completely settled by settled item 3 (in-quota byte offset + 1, all-zero means empty). ⚠️ "A single pointer" is semantic; D21 (Boundary Between Authoritative and Derived States) settled item 3's "128-byte quota holds 18" is capacity arithmetic, not an item-count clause—the extension point holds only one 7-byte pointer; the rest of the quota space is the target area it points to.

**Basis**:

- E43 (Extension Point Byte Upper Bound): Measured four rejection criteria thresholds against the "holds one pointer" shape; core-layer side can afford it, shape itself rejected by no judge; byte-measured pointers fell in the cheapest tier of lower-bound range, whereas ratio-based ceilings clamp nothing on large units.
- User settlement, original wording in history.

**Owed**: None.

#### Settled Item 4: Distinguish Self-Contained Section and Extension Points by "Whether It Changes with Version"

**Settlement**: **A single criterion**—whether a field **changes with version**. What is immutable enters the self-contained section; **what can differ per version enters the extension point**.

**Scope**: Governs classification, not expression once categorized (the shape settled in settled item 3). The two do not conflict: mutable items belong to extension points under settled item 4, and in extension points are always expressed as **pointers**—to house a mutable **value**, place a pointer to that value. ⚠️ **"Mutable" is not in-place modification**: units are immutable (the other half of settled item 1's criterion) ⇒ changing a single byte in an extension point equals writing a new unit; **the precise meaning of "frequently mutable" is "can differ per version"**, not "the same unit being modified repeatedly". ⚠️ **Never use "extension point is within checksum / authentication coverage of whole unit" as a justification**: that conflicts with "keyless side must be able to move data", and after D9 (Encryption) settled item 5 settled that the keyless side only scrubs and never moves, this conflicting rationale no longer bears weight; the only remaining valid reason for physical pointers in extension points is written in D18 (Information Carried in a Block) settled item 3 scope—**cannot generate silent misreads**. ⚠️ "Either empty" means it is **allowed** to be empty; **in version 1 it is empty**—D21 (Boundary Between Authoritative and Derived States) settled item 3 verbatim: "not a single caller line exists in this project ⇒ not a single byte inside extension points". Settled item 4 and settled item 1 are two sides of the same criterion: settled item 1 says "characteristic **immutable** data can enter self-contained section", while settled item 4 says "version-varying items stay out of self-contained section, entering extension points"; together they form the complete boundary.

**Basis**: No experiment: This item settles a binary criterion (whether it changes with version), determining no field widths nor adding on-disk bytes—extension point byte accounting is consumed by D21 (Boundary Between Authoritative and Derived States) settled item 2 / 3, with no quantities to measure here. User settlement, original wording in history.

**Owed**: None.

#### Settled Item 5: Self-Contained Immutable Data Carries a Unique Pointer, Old Versions Disconnected by Tree

**Settlement**: **Self-contained immutable data carries a unique pointer**—"who I belong to", a 5-tuple (tree ID / object ID / object birth generation / anchor offset). **"Unique" refers to a unique chain**: every new data unit starts a unique chain; new data generated ⇒ old chain disconnected ⇒ new link established. **New version written means old version disconnected, and disconnection is completed by the tree**—tree pointers are updated to point to the new unit; **the old unit is not modified** (it is immutable; modifying it violates both COW and D2 (RAID Stripe Policy) "do not induce device RMW"). Two independent scope statements: **fs does not implicitly retain old versions** (unless a snapshot is **explicitly** created, old versions are released on schedule); **backup is not the file system's business** (if users want backup, that is a data copying concern unrelated to COW; this sentence fends off an entire class of feature creep).

**Scope**: ⚠️ **It refers to the birth owner, not "all current referencers".** Under sharing, a unit can have multiple referencers that a fixed-length unit header cannot enumerate; the multiple-referencers half is handled by the second half of settled item 2, landing in D21 (Boundary Between Authoritative and Derived States) settled item 6; **without this qualification, settled item 5 would be read as "this project does not support sharing"**. ⚠️ **Must include the word "implicitly"**—snapshots are precisely "long-term retention of old versions", the very reason D5 (Snapshot / Space Accounting Mechanism)'s deadlist mechanism exists; removing "implicitly" conflicts directly with D5 (Snapshot / Space Accounting Mechanism) / D6 (Snapshot Implementation Model). ⚠️ **Measurement basis of "expiration"** (two settled gates, lacking either makes this statement false): placements referenced by snapshots enter deadlist rather than defer per D3 (Space Allocation) settled item 7, released only upon snapshot destruction; reuse forbidden for recent K generations (I-7.4 (Blocks in Recent K Generations Not Reused), K ≤ root ring depth) ⇒ the precise reading of "released on schedule" is **"those unreferenced by any root (including snapshots) and past K generations"**.

**Basis**: No experiment: This item settles the structural criterion of "how many pointers, who disconnects"; exact widths of 5-tuple segments and whether they enter headers are settled and measured in D18 (Information Carried in a Block) field tables; "unique single pointer" itself produces no measurable quantity. User settlement, original wording in history.

**Owed**: None.

#### Settled Item 6: Withdrawn · This Number No Longer Used

**Settlement**: Withdrawn, this number is no longer used. Original topic: generational data in extension points is a stack of depth 5, released in batch by gc upon expiration. Three-way argument across three legs unanimously judged it bought nothing and was a **structural violation**—violating D21 (Boundary Between Authoritative and Derived States) hard constraint 5 "pointers must not point to core-layer objects", and creating precisely the counterexample C21 (Extension point pointer escaping accounting) used in its self-test of discriminative power.

**Scope**: Established and withdrawn on the same day, 2026-09-06. **Numbers are not recycled**—no bytes exist on disk, but repository history contains entries referencing it; recycling would leave those citations dangling. Pre-withdrawal arguments remain only in changelog, not current design; if withdrawal rationales later become invalid, withdrawn conclusions do not automatically return (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` "invalidation of withdrawal reasons does not mean withdrawn conclusions automatically return").

**Basis**: No experiment: Established and withdrawn on the same day; three legs unanimously judged it a structural violation, verifiable purely against settled clauses, without running nor needing to run measurable quantities. Full withdrawal basis text in [decisions-history.md](../decisions-history.md).

**Owed**: None.

#### Settled Item 7: Reverse Index Required, Designed into Format Day-1

**Settlement**: **Required. And designed into format day-1, post-hoc patch forbidden.** Balance / defrag / shrink / remove device / erasure coding all require answering "who references this physical block".

**Scope**: Governs whether and when it enters the format, not what it looks like (settled item 2 settles the body in units, settled item 5 settles the pointer). Contemporary comparisons serve only as clues, not evidence (`.claude/singlefs-ai-sop/rules/evidence-discipline.md`: how other projects do things is a clue, not evidence): btrfs chose to have it but **grew it post-hoc**, becoming its most complex, bug-ridden part; ZFS chose not to have it in exchange for simplicity, at the cost of twenty years without shrink or defrag. This project seeks to solve shrink and defrag, hence choosing to require it—but it must be designed, not patched. **Bucket generation cannot replace it**: it holds only for cached replicas, governs only "invalidation" not "relocation", and wraparound handling is a full index traversal, while bcachefs itself already migrated to backpointers. Two of these points (bucket gen holds only for cached data; gen wraparound superseded by backpointers) are quoted in [prior-art.md](../prior-art.md) 7.10; the other two (governs only invalidation not relocation; wraparound is a full index traversal) come from contemporary examination of Linux 6.17 `fs/bcachefs/` `buckets.c` and `alloc_background.h` on 2026-08-26, citations in history. None of the four are verified in this project. Bucket generation may be merged as an optimization for "cached replica reclamation", but is ineffective for D9 (Encryption) settled item 5—settled item 5 is "scrub only, no movement", leaving the question "can keyless move data" without a code object.

**Basis**: No experiment: This item is a product direction tradeoff (whether to have this capability, when to enter format); measurable quantities lie downstream—extension point byte accounting measured in D21 (Boundary Between Authoritative and Derived States), unit header field byte accounting measured in D18 (Information Carried in a Block); external surveys raise hypotheses per rule, not entering arguments. User settlement, original wording in history.

**Owed**: None.

#### Settled Item 8: Boundary with D18 (Information Carried in a Block)

**Settlement**: One sentence on each side, never muddled. **D1 (Data Mobility / Reverse Index) settles criteria**—what can enter self-contained section, what enters extension points; **D18 (Information Carried in a Block) settles field tables**—which fields actually entered, their widths, and classification by unit classes. **When criteria change, D18 (Information Carried in a Block) field tables update accordingly; field table arrangement does not modify criteria in reverse.**

**Scope**: Governs division of labor between these two decisions, not a third. Extension point **byte layout** resides neither in this item nor in D18 (Information Carried in a Block), but in D21 (Boundary Between Authoritative and Derived States) settled item 2 / 3 / 4.

**Basis**: No experiment: This is a division-of-labor clause settling which decision has authority, producing no on-disk bytes nor measurable quantities; it prevents the same matter being written half in two places and cross-referencing each other's half-sentences. User decision, original wording in history.

**Owed**: None.

## Historical Versions

Historical entries for D1 (Data Mobility / Reverse Index) are centralized in [decisions-history.md](../decisions-history.md).
