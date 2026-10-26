## D6 Snapshot Implementation Model —— Settled

[中文](06-快照实现模型.md) · **English** · [日本語](06-スナップショット実装モデル.ja.md)

D6 (Snapshot Implementation Model) governs which on-disk mechanism implements snapshots and writable heads: whether to implement multiple writable heads, which mechanism to adopt, what structure is needed to destroy a writable head, the entry width and tree creation timing for that structure, whether rolling back a writable head to snapshot state is provided, and what criteria a snapshot implementation model must pass for adoption. Out of scope: snapshot space accounting (`birth` txg, deadlist, sparse side table, metrics inventory) in D5 (Snapshot / Space Accounting Mechanism); index tree key layout, tree table entry field table, and tree kind codes in D8 (Core Index Structure); central mapping key width and encoding in D19 (Block Pointer Structure & Width Budget); administrator rollback by publish in D23 (Journal Role & Format) settled item 14; alignment between snapshot points and checkpoint boundaries in D16 (Publish Semantics).

### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Specific mechanism for multiple writable heads** | Adopt ②, each head has its own tree; accept a reference count side table plus 3 fixed metadata items per head **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Livelist entry key and width, and whether that shared tree is created in new-pool file creation** | Entries 36 bytes uniform width, empty value; shared tree registered in tree table day-1, new-pool file creation adds one tree table entry **Status: Settled.** **Progress: Implemented.** |
| 3 | **Second structure for destroying a writable head: whether needed, what shape, where it lives** | Needed; takes ZFS livelist event stream shape, FREE recorded the moment a block is killed; carrier is a shared tree, key prefixed by head tree ID **Status: Settled.** **Progress: Unimplemented.** |
| 4 | **Whether rolling back a writable head to snapshot state (rollback) is provided in version 1** | Not provided; to return to a snapshot's state, clone a new writable head from that snapshot and destroy old head as usual **Status: Settled.** **Progress: No code object.** |
| 5 | **Whether to support multiple writable heads, how cost is paid** | Support, paying on lookup: O(N) reverse index lookups per snapshot deletion, in exchange for multiple writable heads and no loss / no leak **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **What multiple writable heads broke in D5 (Snapshot / Space Accounting Mechanism)** | Breaks O(1) property, not correctness: structural data loss is zero, what fails is leakage and misattribution from over-verdicts **Status: Settled.** **Progress: No code object.** |
| 7 | **Whether partial overwrite across snapshots produces sub-unit fragmentation** | None in version 1: whole-unit read-modify-write plus extent strictly equals unit ⇒ partial overwrite takes whole-unit COW **Status: Settled.** **Progress: Implemented.** |
| 8 | **Criteria for a snapshot implementation model to be acceptable** | Nine criteria, acceptable if and only if all simultaneously satisfied; derived from mechanisms and methodology, not from decision standing **Status: Settled.** **Progress: No code object.** |
| 9 | **Which rationales are forbidden from rejecting candidate snapshot implementation models** | Six false commandments deliberately excluded from criteria; invalid to reject candidates using them **Status: Settled.** **Progress: No code object.** |
| 10 | **Whether that multiple writable heads O(N) has been optimized** | Nested interval labeling plus deletion frontier validated at model layer to pull back to O(branch count); implementation follows pay-on-lookup until two bounds are cleared **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Multiple Writable Heads Adopts Each Head Has Its Own Tree

**Settlement**: Adopt ②——**each head has its own tree**, accepting costs on its side (one reference count side table + 3 fixed metadata items per head), replacing the constant "read path descends H−1 additional tree levels" of ①-prefix. Candidate ① (bcachefs **snapshot ID in low bits of key**) is actually two branches—low bits vs prefix, where the cost of deleting a head differs entirely (full scan vs range operations); the low-bit branch is **strictly dominated** by prefix, eliminated first.

**Scope**: Governs which mechanism multiple writable heads adopts, not accounting within each head (D5 (Snapshot / Space Accounting Mechanism)), nor what structure destroys a head (settled item 3). ② **breaks D5 (Snapshot / Space Accounting Mechanism) premise 3** (at most one live reference per block)—the reference count side table is required precisely for this reason; acceptability of ② depends on whether the side table fits in cache. The "H descents" of ①-prefix is the upper bound for the simplest form; skiplists / bitmaps / ancestor set caching could push it down, but no model has been built. This settlement **may be overturned during implementation testing** (aligned with `.claude/rules/format-evolution.md` "until the first external user appears, disk format is soft"), but discipline of overturning remains: edit text immediately and record rationale in [decisions-history.md](../decisions-history.md) under D6 (Snapshot Implementation Model) section, silent changes forbidden.

**Four Triggered Cascades**:

| # | What Was Triggered | Disposal |
|---|---|---|
| 1 | **D5 (Snapshot / Space Accounting Mechanism) reopening clause triggered** | D5 (Snapshot / Space Accounting Mechanism) states verbatim: "**Any feature introducing a 'second live reference to the same block' must first reopen D5 (Snapshot / Space Accounting Mechanism)**", and ②'s cross-head sharing is precisely a second live reference. ⇒ Reopened and closed per D5 (Snapshot / Space Accounting Mechanism)'s own correct form: **D5 (Snapshot / Space Accounting Mechanism) primary + sparse side table secondary (ZFS BRT shape)** |
| 2 | **D5 (Snapshot / Space Accounting Mechanism) settled item 18 "clones / writable snapshots add three rules" section converted from reference to mandatory** | Each head maintains its own copy of `previous_snapshot_txg` and deadlist; **new clone's `previous_snapshot_txg` must initialize to origin snapshot's txg** (setting 0 loses data); fork-point snapshots forbidden from destruction; destroying a writable head requires a second structure (settled item 3 livelist shape) |
| 3 | **D8 (Core Index Structure) three-segment key collision resolution** | Under ② each head has a tree ⇒ three-segment key is unique **within tree**, collision vanishes. ⚠️ **Nonce obligation remains**: D9 (Encryption) settled item 4 settled nonce stored per extent; **generation must still guarantee no collision across heads** |
| 4 | **D9 (Encryption) settled item 6 scope hole "AAD lacks snapshot dimension", "patched along the way" invalid under each head having its own tree** | Readers taking tree dimension along lookup path catch invalid mounts, **but blocks born before clone point and legitimately inherited into new head are also rejected**—tree dimension cannot distinguish legitimate inheritance from invalid mounts, while cross-head sharing is the explicitly retained shape in cascade 1 ⇒ using verbatim breaks it. Feasible form is birth (tree, txg) into AAD + pointer carries declaration + clone ancestor table acceptance, authoritative record in D9 (Encryption) settled item 6 |

**Basis**:

- E52 (Operational Costs of Two Multi-Writable-Head Mechanisms): Empirical measurements across three mechanisms on five metrics: deleting a head, point lookup component, side table entries, total key bytes, and fixed metadata—①-low-bit surpassed ② only on side table and metadata, while ①-prefix matched it identically there and collapsed by three orders of magnitude on deleting heads ⇒ strictly dominated; remaining tradeoff is a single sentence.
- E12 (Batched Sequential Append vs Unbatched Random Page RMW): Side table cost—virtually free when fitting in cache; approximately one device I/O per deletion when spilling.
- E35 (Two Forms of Multi-Writable-Heads): Disproved "three-segment key + **single tree** + multiple heads"; under ② three-segment key is unique within tree.
- User settlement (accepting costs on ②'s side), original wording in history.

**Owed**: When ②'s reference count side table shares incremental semantics with runtime, I-3.1 (Allocated Statistics Match) fails to catch additions and subtractions written incorrectly—**no check number monitors this cell**. C12 (shared delta semantics) cleared on 2026-09-21 the cell between checker and implementation (gate checker-independence-and-sync checker-implementation-disjoint cell judges dependency closure intersection leaves only constant modules), while side table and runtime reside together within `singlefs-core`, where checker-implementation-disjoint does not judge whether two code segments within one crate share identical incremental semantics implementation.

#### Settled Item 2: Livelist Entry 36 Bytes Uniform Width, Shared Tree Registered in Tree Table Day-1

**Settlement**: ① Entry key = head tree ID 8 + central mapping key **27 (fixed width)** (D19 (Block Pointer Structure & Width Budget) settled item 6 code 1 shape; code 2 / code 3 25-byte key **zero-padded at tail to 27**, D19 (Block Pointer Structure & Width Budget) settled item 10) + event type 1 byte (ALLOC / FREE occupies separate byte, not folded into class tag), total **36 bytes uniform width**, empty value; ② That shared tree **registered in tree table day-1**, new-pool file creation adds one 200-byte tree table entry (D8 (Core Index Structure) settled item 8 entry width), zero root pointer. Modifies new-pool file creation bytes: **Yes**—width of livelist entry in tree table; entries themselves write zero rows in new-pool file creation.

**Scope**: Governs entry key and width, and when shared tree is created, not when entries are written (settled item 3 settles FREE recorded the moment a block is killed). Entry identity must be central mapping key, position entry strictly forbidden—D19 (Block Pointer Structure & Width Budget) settled item 5 rule 1 verbatim: "release always goes via mapping, never via hints", invalidating old shape recorded by position entry. Each FREE prepays 36 bytes; whether it enters "how much is needed to disentangle oneself" clause remains owed. Both changes (entries returning from "36 / 34 dual width" to "36 uniform width", tree table entry widening) were user modal decisions without three-way argument, overturnable.

**Basis**:

- E146 (Width and Cost of Livelist Entries Identified by Mapping Key): Entry width, leaf and internal fanout, tree height, and prepayment per FREE across four candidate shapes after identity settled by mapping key—36 bytes uniform width tier has tree height identical to 35-byte shape.
- E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout): Those 21 scale cells are the grid judging tree height; tree table entries 200, threshold and point lookup depth under rerun with 81 trees per level are there.
- User settlement (36 uniform width entries, shared tree registered day-1), original wording in history.

**Owed**: C260 (livelist entry prepayment omits the amount for unwinding itself).

#### Settled Item 3: Second Structure for Destroying a Writable Head: Needed, ZFS Event Stream Shape, One Shared Tree

**Settlement**: Settle three matters together.

1. **Needed**. Destroying a writable head requires a second on-disk structure, taking D5 (Snapshot / Space Accounting Mechanism) verbatim ZFS livelist—ALLOC / FREE paired, grouped by birth txg, incremental, condensable, **must include condense**. Three other candidates eliminated: traversal cannot provide pending deletion footprint needed for admission at intent publish; reusing sparse side table misses blocks with reference count 1; missing functionality requires retracting a premise already consumed downstream.
2. **FREE recorded the moment a block is killed**, filtered by birth later than clone moment, without waiting for block to be truly released. Under the alternative, heads that created and then deleted snapshots would release the same block twice.
3. **Carrier takes one shared tree**, key prefixed by head tree ID. A tree per head is not simpler—it introduces four unique mechanisms (head-to-livelist link, tree creation per clone, snapshots not copying livelist, tree teardown), while a shared tree deletes by head prefix, following range delete long required by D8 (Core Index Structure) settled item 5 verbatim: "range delete directly discards whole nodes without inserting N tombstones". A tree per head buys point lookups 1–2 levels shallower and mutual isolation among heads.

**Scope**: Governs whether this structure is needed, what shape, and where it lives, not entry key and width (settled item 2). Concomitant effects and costs: point lookups 1–2 levels deeper than per-head tree, entries for all heads live in one tree; tree table second-tier threshold is 39 heads for shared tree, 27 heads for per-head tree; pre-batch reservation requiring "how many entries in this head's livelist" cannot count by prefix range, needing a per-head counter that can only live in the remaining 8 reserved bytes of head's tree table entry; user's initial statement "trading space for complexity" does not hold here—shared tree livelist consumes more bytes, buying fewer mechanisms. Relies on two rules supplemented same day in D5 (Snapshot / Space Accounting Mechanism): writable heads with their own live snapshots forbidden from destruction; fork-point gate `children` counts clones. Local ZFS source `dsl_dataset.c` records FREE on block-killing path—clue, unverified in this project. ⚠️ Thresholds and point lookup depths take numbers from E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout), which corrected E131 (Two Livelist Carriers: Values and Costs)—E131 (Two Livelist Carriers: Values and Costs) calculated on one data tree per head without pool-level trees; its reported head counts must not be cited.

**Basis**:

- E130 (Does Per-Head Livelist Pass Bounded Destruction): Verified livelist shape passes criterion 5 (bounded destruction) of settled item 8, provided it includes condense.
- E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout): Tree table level thresholds and point lookup depth for two carriers under true tree count and true internal fanout.
- User settlement (settling three matters together, adopting shared tree after re-judging under corrected measurement basis from round three adversarial debate), original wording in history; process of three rounds of debate in [decisions-history.md](../decisions-history.md) and [2026-09-10-D6项2三轮对抗.md](../../../records/2026-09-10-D6项2三轮对抗.md).

**Owed**: C271 (when livelist FREE is recorded is undefined).

#### Settled Item 4: Rolling Back a Writable Head to Snapshot State Not Provided in Version 1

**Settlement**: Version 1 provides no operation to "roll back a writable head to the state of one of its snapshots" (ZFS `rollback` family). To return to a snapshot's state, clone a new writable head from that snapshot (D5 (Snapshot / Space Accounting Mechanism) "clones / writable snapshots add three rules", a ready-made action under settled item 1's per-head tree), and destroy the old head per settled item 3 as usual.

**Scope**: Governs whether version 1 provides this operation, not future implementations. Administrator rollback in D23 (Journal Role & Format) settled item 14 is pool-wide rollback by publish hosted on root ring, not this operation. Human-scale "return to old state" thus reduces to cloning plus administrator rollback; before narrowing root ring commitment to administrator rollback, revisit this item first. Create new item if this operation is needed in future; then answer: what happens to generations of original head after rollback, how accounting follows, and how it coexists with instance generations (instance table rollback row deleted along with administrator rollback converting to forward publish while mounted, 2026-09-26, D23 (Journal Role & Format) settled item 14).

**Basis**: No experiment: C223 (rolling a writable head back to a snapshot state is undefined repo-wide) when established stated a choice of two—define operation here, or state version 1 omits it with substitute; writing "version 1 excludes" counts as clearing it; this is a feature tradeoff, and version 1 having no such operation means no measurable quantity. User settlement adopted "not provided in version 1", original wording in history.

**Owed**: None (C223 (rolling a writable head back to a snapshot state is undefined repo-wide) cleared 2026-09-14 following user settlement "not provided in version 1", see cleared owed check table).

#### Settled Item 5: Multiple Writable Heads Supported, Cost Paid on Lookup

**Settlement**: **Support multiple writable heads, paying on lookup**—pay O(N) reverse index lookups per snapshot deletion, in exchange for multiple writable heads and **no loss / no leak**. Neither alternative path is taken: not supporting multiple writable heads (O(1) preserved, constant 0 lookups, but relinquishing feature); supporting while refusing to pay (multiple writable heads + O(1), but **reclamation rate drops to zero**, unusable).

**Scope**: Governs whether to support and how cost is paid, not which mechanism is adopted (settled item 1). **Snapshot destruction falls into third cell of `.claude/rules/fs-design.md` three-cell discipline** (background, resumable, non-decision path—no commandments) ⇒ O(N) traversal here is explicitly permitted by project discipline. ⚠️ Exemption subject is **snapshot destruction**, and exempted quantity is "O(N) reverse index lookups per snapshot deletion"; **it does not cover destroying a writable head**—D5 (Snapshot / Space Accounting Mechanism) defines snapshot destruction verbatim as "merging its deadlist **into the next newer side**... **without full-disk scan**", whereas destroying a writable head releases `birth > origin.txg` blocks "scattered across the tree"; the two are not the same path. Nor does it constitute "therefore writable head path must have on-disk structure"—reason column of third cell states verbatim "they follow 'write intent → batching → resumable'", a prerequisite judged under settled item 2. ⚠️ "Attribution to original creator" (btrfs squota mechanism) cannot serve as release verdict: squota governs only quota attribution, not liveness verdicts; cross-citing between them is forbidden. btrfs backref resolution numbers are not suitable for direct adoption—no such component exists in this project's design ([prior-art.md](../prior-art.md) marked as "qualitative evidence only").

**Basis**:

- E26 (Lookup Cost and Accuracy of Three Accounting Models): Measured reverse index lookups per snapshot deletion across three accounting models; log-log slope of pay-on-lookup arm demonstrates cost paid is not "a bit more expensive", but falls from constant into O(N); in same artifact, "attribution to original creator" under multiple writable heads + 70% sharing mistakenly released all writable heads, where erroneous release is data loss direction.
- E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads): Leakage rate of "support while refusing to pay" arm is bit-for-bit identical to "never recording death"—it buys zero data loss with "reclaiming not a single block", an unusable status determined empirically, not inferred.
- User settlement "I insist this is the future"—a product direction judgment, not deduced from cost; adopting "pay on lookup" follows `.claude/rules/fs-design.md` "when criteria cannot be formulated, implement only one path first, and implement the slow but correct one first". Original wording in history.

**Owed**: C35 (Multi-Writable-Head O(N) Unoptimized).

#### Settled Item 6: Multiple Writable Heads Breaks O(1) Property of D5 (Snapshot / Space Accounting Mechanism), Not Correctness

**Settlement**: D5 (Snapshot / Space Accounting Mechanism) clause "if D6 (Snapshot Implementation Model) adopts multiple writable heads candidate, O(1) property fails" **has already been triggered**. What must be read accurately is: what fails is **O(1) property, not correctness**—data loss direction is structurally zero (provable, not merely empirical), and failure manifests as space leakage and misattribution from over-verdicts.

**Scope**: Governs "which half fails", not how to pull O(N) back (settled item 10). Concrete counterexample: mainline txg=10 creates writable snapshot X; in X at txg=30 write a block (birth=30); mainline txg=35 creates snapshot Y. Under D5 (Snapshot / Space Accounting Mechanism) condition `30 ≤ 35 < death` → verdicts "Y references it", **whereas Y cannot see it at all** (that key is mounted on X's branch) ⇒ over-verdict ⇒ space leak + misattribution, **yet I-3.1 (Allocated Statistics Match) stays completely green** (block still referenced by exactly one place, total ledger balances)—precisely the shape settled item 8 condition 1 guards against.

**Basis**:

- E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads): Failure manifestation is "unbounded leak + zero reclamation", not data loss; leakage rate bit-for-bit identical to "never recording death".
- User settlement provided alongside settled item 5, original wording in history.

**Owed**: C35 (Multi-Writable-Head O(N) Unoptimized).

#### Settled Item 7: Partial Overwrite Across Snapshots Produces No Sub-Unit Fragmentation in Version 1

**Settlement**: Version 1 **produces no fragmentation**. Whole-unit read-modify-write in D4 (Checksum Location) settled item 3 + extent strictly equals unit in settled item 7 ⇒ partial overwrite across snapshots takes **whole-unit COW**—snapshot preserves old whole unit, live version writes a brand-new whole unit, offsets on both sides are 0, with no sub-unit sharing.

**Scope**: Governs whether cross-snapshot sub-unit fragmentation appears in version 1, not what happens once splitting is enabled. D9 (Encryption) settled item 12 item 2 "extent offset from original extent origin" reserves bits for **splitting**, while splitting is a reserved feature disabled in version 1 ⇒ cost in version 1 is write amplification of whole-unit COW, not fragmentation. The "constant ancestor filtering overhead on read path" ceased to exist after settled item 1 adopted ②, and was never re-accounted.

**Basis**: No experiment: Directly deduced from D4 (Checksum Location) settled item 3 / settled item 7 and D9 (Encryption) settled item 12; version 1 lacks sub-unit sharing as an object, with no measurable quantity; measurement awaits enabling splitting feature, at which time new items and experiments will be established.

**Owed**: None.

#### Settled Item 8: Nine Criteria for Snapshot Implementation Model

**Settlement**: **A snapshot implementation model may be adopted if and only if it simultaneously satisfies all nine criteria.** These nine criteria are derived from mechanism and methodology, **not from the standing of any decision**—rejecting candidates via "this candidate breaks D5 (Snapshot / Space Accounting Mechanism) premise ⇒ adopting it invalidates D5 (Snapshot / Space Accounting Mechanism)" is forbidden; that is conditional statement "D6 (Snapshot Implementation Model) → ¬D5 (Snapshot / Space Accounting Mechanism)", reading it backward closes into a circle.

| # | Condition | Basis |
|---|---|---|
| 1 | **Single predicate**: A unique predicate "S references b" exists, **simultaneously** driving read visibility and space reclamation | Asymmetric consequences of divergence: visible set larger than accounting set ⇒ **data loss**; reverse ⇒ leak + misattribution, structurally invisible to I-3.1 (Allocated Statistics Match). D5 (Snapshot / Space Accounting Mechanism) conservative default discipline **catches only the latter** |
| 2 | **Decision-path incrementality + independent audit**: Accounting metrics entering admission control / ENOSPC / defer verdicts maintained incrementally at commit, cost invariant with disk capacity; and an independent recalculation path **sharing no verdict code with runtime** exists | D3 (Space Allocation) admission control; C12 (shared delta semantics) and its homologous failure observed in bcachefs |
| 3 | **Boundary alignment + complete view**: Snapshot point falls on checkpoint boundary, pinned after all data mutations of current checkpoint; no "partially visible" or "user-visible but unpublished" snapshots exist | D16 (Publish Semantics) settled item 14. `birth` resolution reaches only checkpoint ⇒ blocks cannot express "born before or after snapshot point", and `birth == S.txg` (data loss) and `death == S.txg` (leak) **simultaneously lose definitive answers** |
| 4 | **Bounded creation**: Atomically completed within single checkpoint, cost invariant with scale, and **touches zero bytes of any published block** | D8 (Core Index Structure) "unbounded must batch" × D16 (Publish Semantics) "snapshot point must not cross checkpoint boundary" mutually exclusive. Second leg from D9 (Encryption): touching published blocks requires re-encryption ⇒ new nonce + MAC changes ⇒ rewrite along references to root |
| 5 | **Bounded destruction**: Batchable, resumable, worst-case space requirement per batch bounded and pre-reservable before batch | D3 (Space Allocation) "releasing space requires no space allocation". ⚠️ D5 (Snapshot / Space Accounting Mechanism) evades this cell via missing feature "fork-point snapshots forbidden from destruction", rather than satisfying it |
| 6 | **No second transaction layer**: Introduces no publish path bypassing journal, adding no structural steps to commit protocol | Domain of crash points is transaction layer state machine ⇒ two layers form a **Cartesian product**; and D16 (Publish Semantics) defines crash semantics as "root + arbitrary prefix of journal", with no total order between two journals ⇒ specification itself undefined |
| 7 | **Recoverable by scan rebuild**: Inputs for snapshot set and visibility verdicts reconstructible under condition of having only "all blocks + their self-describing headers" | Scanning checker of D13 (Verification Route) settled item 12 and I-6.1 (nonce not reused) stage ② both build upon it |
| 8 | **Small-scale judgeable**: Correctness divergence can be **force-triggered** at smoke-tier scale, with test switches directly constructing that state | `.claude/rules/fs-design.md` hard requirement 2 "unable to enter equals untested" |
| 9 | **Identifiers unrecycled + layout-agnostic**: **Published** snapshot identifiers monotonic and unrecycled, bit width written into format with declared upper bound on lifetime creations | Recycling a snapshot ID makes stale on-disk keys **become visible** in new snapshots, homologous to C11 (Feature Bit Number Skip). Layout-agnostic half comes from D12 (Target Media) and pitfall 13 |

**Scope**: Governs whether candidates enter the door, not how adopted mechanisms evolve. ⚠️ **The words "published" in condition 9** limit scope: verb in reason sentence is "become visible", where visibility acts on **keys in trees**; orphan units left by checkpoints never successfully published are in no tree, and the two gates judging them (published predicate, select new key) and orphan reclamation conditions do not consult tree ID ⇒ unpublished identifiers being reissued fall outside this criterion's scope. Criteria with discriminative power among the nine are 1, 4, 5, 8; remaining five are symmetric or satisfied on both sides.

**Basis**:

- E26 (Lookup Cost and Accuracy of Three Accounting Models): Asymmetric consequences of divergence in condition 1 were measured—predicate "attribution to original creator" under multiple writable heads + 70% sharing mistakenly released all writable heads, where erroneous release is data loss direction.
- E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads): Reverse direction (accounting set larger than visible set) results in unbounded leak + zero reclamation, while I-3.1 (Allocated Statistics Match) stays completely green—quantifiable evidence for condition 1 "structurally invisible to I-3.1 (Allocated Statistics Match)".
- 2026-08-26 three-way argument: Nine criteria settled by that round, process in history.

**Owed**: None. Condition 2's requirement of "independent recalculation path sharing no verdict code with runtime" is satisfied by checker-independence-and-sync gate's checker-implementation-disjoint cell: dependency closure intersection between checker and implementation leaves only constant modules, and checker source contains zero citations of `singlefs_core` (C12 (shared delta semantics) cleared 2026-09-21).

#### Settled Item 9: Six False Commandments Forbidden from Rejecting Candidates

**Settlement**: Six items in this table are **deliberately excluded** from the nine criteria of settled item 8. Using them to reject candidates equals using a decision that was originally alterable to reject a potentially superior model.

| Forbidden to Use | Why |
|---|---|
| "Never post-hoc traversal" | Governs only **runtime decision path**, with three existing exemptions: I-3.1 (Allocated Statistics Match) full-disk traversal, D8 (Core Index Structure) intent batching, D13 (Verification Route) settled item 8 intergenerational incremental scrub |
| "I-3.1 (Allocated Statistics Match) is the sole mechanism" | Bidirectionally disproved by I-3.5 (precision of the reference interval) "both must exist" and known blind spots of C12 (shared delta semantics) |
| "Must be birth txg + deadlist" | That is an **instance** of D5 (Snapshot / Space Accounting Mechanism); criteria require only predicate consistency of condition 1 |
| "Adding another format branch" | Zero current cost before freeze (`.claude/rules/format-evolution.md` explicitly states this freedom "must be fully used"), and project has already spent format branches: D12 (Target Media) |
| "Separator key sensitivity" | **Sensitivity is a function of encoding, not btree property**: replacing dirent name component with hash leaves both properties D8 (Core Index Structure) relies upon unaffected. ⚠️ This is reasoning, unverified in this project, without accounting hash encoding costs on readdir order and collision handling |
| "Total verification volume too large / gates cannot finish" | D13 (Verification Route) settled item 9 placed unbounded verification volume into full run tier; wall clock constrains only quick tier |

**Scope**: Governs prohibiting these six from serving as rejection rationales, not whether they are valid in their own right—"never post-hoc traversal" remains valid on runtime decision path; this item merely forbids using it to reject candidate snapshot implementation models.

**Basis**: No experiment: Counterexamples to the six are settled clauses and existing exemptions (three traversal exemptions, I-3.5 (precision of the reference interval) "both must exist", pre-freeze format freedom, full run tier), not measurable quantities; the one requiring measurement (hash encoding cost on readdir ordering and collisions) is self-acknowledged as uncalculated, requiring experiments before use. 2026-08-26 three-way argument settled this table, process in history.

**Owed**: None.

#### Settled Item 10: Model Layer Optimization for O(N) Exists, Implementation Follows Pay-on-Lookup Until Two Bounds Cleared

**Settlement**: "Comparing txg with previous snapshot" in D5 (Snapshot / Space Accounting Mechanism) is essentially **interval labeling on a chain**; generalization to trees is **nested interval labeling** (preorder / postorder), converting "whether live snapshots remain in subtree" from traversal to single interval containment check, paired with a **deletion frontier** (each block records minimal antichain of branches where it was deleted, recorded on the spot at commit when "inheritance did not occur"—accounting is a byproduct of transactions). This path **validated at model layer**. **Until two bounds are cleared, implementation continues to follow pay-on-lookup per settled item 5.**

**Scope**: Governs standing of this optimization (validated at model layer, implementation unchanged), not when to switch. Two bounds: ① when tree grows, labeling requires order-maintenance / list-labeling incremental maintenance; mature solutions exist but unverified in this project; ② implementation re-verification on transaction layer and true reverse index including crash-point replay has not advanced a single step. "Tree vs DAG" resolved by settled item 1—one tree per head, cloned from single origin, no merge operations ⇒ lineage is tree, avoiding DAG-level difficulty.

**Basis**:

- E89 (Nested Interval Labeling and Deletion Frontier): All three criteria met—candidate arm has constant 0 false releases and omissions across all cells (against independent ground truth); on fixed tree expanding extents, reverse lookups for pay-on-lookup baseline grow with scale while candidate stays constant 0, frontier upper bound tracks branch count; under Linear topology frontier degrades back to constant form of D5 (Snapshot / Space Accounting Mechanism). Frontier degradation bound also measured by it, within antichain upper bound.
- E26 (Lookup Cost and Accuracy of Three Accounting Models): Pay-on-lookup baseline O(N) was measured by it, representing target this optimization pulls back.

**Owed**: C35 (Multi-Writable-Head O(N) Unoptimized): Both bounds (incremental labeling maintenance, implementation re-verification on transaction layer and true reverse index including crash-point replay) recorded under it.

### Cascades with Other Decisions

| Decision | Cascade Point |
|---|---|
| D5 (Snapshot / Space Accounting Mechanism) | Settled item 5 / settled item 6 triggered its reopening clause, now closed (primary + sparse side table secondary); settled item 1 converted its "clones / writable snapshots add three rules" from reference to mandatory |
| D8 (Core Index Structure) | Settled item 1 eliminates three-segment key collision (unique within tree); settled item 3 range delete follows its settled item 5 requirement; settled item 2 tree table entry width takes its settled item 8 |
| D9 (Encryption) | Settled item 1 cascade 4: AAD lacking snapshot dimension hole not "patched along the way" under tree-per-head, feasible form in D9 (Encryption) settled item 6; nonce collision-freedom across heads obligation remains |
| D19 (Block Pointer Structure & Width Budget) | Settled item 2 entry key takes central mapping key from its settled item 6, fixed width 27 from settled item 10 |
| D23 (Journal Role & Format) | Substitute path in settled item 4 and administrator rollback in its settled item 14 are distinct matters |
| D13 (Verification Route) | Settled item 8 condition 7 builds upon its scanning checker in settled item 12; settled item 9 cites its settled item 8 / settled item 9 |
| D16 (Publish Semantics) | Settled item 8 conditions 3 and 6 rest upon D16 (Publish Semantics) settled item 14 and crash semantics |
| [layout/01-first-txn.md](../layout/01-first-txn.md) | Settled item 2 adds one livelist entry to version 1 tree table (12 entries total) |

## Historical Versions

Historical entries for D6 (Snapshot Implementation Model) are centralized in [decisions-history.md](../decisions-history.md).
