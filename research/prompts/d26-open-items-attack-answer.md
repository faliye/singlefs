# D26 open items — adversarial-attack leg (Opus, 2026-09-03), verbatim

Reviewed the dossier only. All attacks below are built from F1–F11, E74/E81/E93/E94. Where an attack needs a fact the dossier does not carry, I mark it conditional.

## Cross-cutting problem that hits P1, P3, P5, P6

Two transfer gaps limit how much weight either model can carry, and three proposals lean past them.

- E94's tree does not match D8/D11. E94 is fanout 16, 4096 keys, no message buffer. D8/D11 is fanout 119 with a 665-entry message buffer per 16 KiB node (F3). Tree depth, node touch counts, and the mechanism of a reference update (direct node rewrite vs. an inserted message that flushes later) all differ. E94's absolute counts (106/181/244 nodes, 11–36 deadlist entries) therefore do not transfer; only its structural findings do — K-invariance vs. K-linearity, per-batch rewriting of all K+1 snapshot roots, zero deadlist rewrites under central_map, birth-belongs-to-version. Proposals that cite E94 counts as budgets (P3c) are over-reading; proposals that cite its structure (P3's handoff to the user) are not.
- E93 models neither stripes nor crashes. It is fixed-size objects in slots. There are no wide stripes, no parity, no pinning, no intent/redo, no metadata-vs-data distinction. Every settled clause about pinned stripes (F10) and about resumable batched intents (F4) is outside its model.

## P1 — transaction shape

Attack. "Ordinary transaction sequence" is false under the currently settled position architecture. D19 is settled (F8): block pointers carry position entries inline in every referencing tree. That is E94's ptr_rewrite arm, and E94's finding for it is that moving a 64-slot batch rewrites all K+1 root records every batch, with the note that snapshot roots are published identity anchors and per-batch root drift breaks anything pinned to them. An ordinary transaction publishes a new live root; it never mutates the roots of already-published snapshots. So under D19-as-written, a compaction batch performs an operation that no other transaction in the system performs, against objects (published snapshot roots) that the rest of the design treats as immutable anchors. Whether that is a new CommitStep member or a novel use of CasRootPointer / AppendRootRecord against snapshot roots, D13 (F5) makes the enum the thing crash-replay sampling is defined over — and the crash semantics of "rewrite K published snapshot roots in one transaction" have never been sampled. P1's conclusion ("no new member, therefore leg 1 stays non-format-level") also does no work: F2 already asserts leg 1 is non-format-level. What P1 needed to establish — that compaction adds no unsampled commit-point composition — it does not address.

Second, smaller: P1 lists only WriteDataBlock / WriteInternalNode, but D2 item 11 (F10) assigns the lazy re-lay of partially-freed wide stripes to leg 1, which necessarily involves WriteStripeParity and a crash-atomicity requirement (parity must never cover a mix of old and new cells) that a plain data move does not have.

Third: D8 item 5's middle crash state is "intent + partial work, idempotently redone" (F4). For truncate, redo is idempotent. For a move, redo re-derives destinations from an allocator whose state changed across the crash, so the redone work is not the same work. P1 inherits D8 item 5 without saying what makes a move batch idempotent.

Settling observation. Today, in a model: extend E94 with an "immutable published snapshot roots" constraint and check whether ptr_rewrite can complete a move batch at all without rewriting them — if it cannot, P1's claim is architecture-conditional and must be stated as such. Also today: add a crash/redo arm (replay the last batch after truncation) and check that reference state, accounting, and F all converge. Whether a snapshot-root rewrite is crash-safe in practice needs the real filesystem plus D13 sampling.

Severity: blocks — not the eventual answer, but the claim as written is only true under central_map, which P3 explicitly refuses to settle in this round.

## P2 — does compaction reserve space

Attack 1 (the load-bearing one). "Deletion never depends on compaction" is false for the pinned-stripe class. D2 item 11 (F10): partially freeing a wide stripe pins it and moves its cells into the unreclaimable statistic. D3 item 4 (F6): available subtracts unreclaimable. So deleting a cell in a wide stripe converts allocated into unreclaimable — net zero change to available. The delete operation completes (tombstone prepaid, D3 item 2), but it returns no space. The only path that returns that space is leg 1 of D26. Under a full disk, compaction stalls; under a stall, further deletes only add pinned cells; unreclaimable ratchets monotonically. That is a state in which no amount of deletion can clear ENOSPC — precisely what D3's principle "freeing space must itself never require allocating space" exists to forbid. P2's move of reclassifying compaction as "consuming, not freeing" does not dissolve this; it relabels the one operation that is the freeing path for that class. (Conditional: the dossier does not say whether wide stripes exist in the phase-1 pure-SSD line (F10, D12 item 5). If they do not, the ratchet is deferred, not eliminated — and P2 is written as a general answer.)

Attack 2. "Same admission control as any writer" understates the asymmetry. F6 item 6 grants no credit for releases inside the window, and F11 (D16 rule 2) keeps space freed in checkpoint C out of the allocatable set until C publishes. So compaction is strictly negative on available for its entire in-flight window, by the batch plus its metadata COW, and it needs that headroom exactly when headroom is scarce. It is the hardest admission case in the system, not an equal one.

Attack 3. Compaction's own commit-intrinsic blocks fall under D3 item 5: they must come from a contiguous bump segment or pay E81's penalty (12-unit fsync → 5909 blocks, 65 fixpoint rounds, vs. 2 rounds / 28 blocks clustered). The trigger P5 proposes fires when empty segments are scarce, so the first compaction transaction after supply collapse is the one most likely to be denied a clustering segment. Bootstrap circularity, unmodeled by E93 (no metadata/data split).

Standing-reserve sub-question — do E93's fallback numbers demand it?

No, they suggest it; the demand, if it exists, comes from elsewhere.

- E93 measured fallback over all allocations (8192 objects, 64 dirty per checkpoint), with no separation of commit-intrinsic metadata. The quantity that would settle the reserve — the clustering-success rate for commit-generated blocks under D3 item 5 — was not measured. Transferring 91.8–93.8% (no compaction) or 50% (max budget) to the commit path is an unlicensed extrapolation.
- The 50% residual actually cuts against P2's framing. P2 says "compaction is what sustains empty-segment supply." E93 says compaction is the only force that moved the number, and it moved it from 88% to 50%. Half of all allocations still cannot enter a clustering segment at maximum budget. If clustering is load-bearing for commits, budget-driven compaction does not deliver it; a segment set held out of general allocation would, and that is a different mechanism than "more budget."
- E93 also has no fill-level sweep — one point, 80% fill. A standing reserve raises effective fill for everyone else, and E93 shows saturation is fast at 80%. The reserve's own second-order cost is unmeasured.
- The genuine demands are correctness-shaped, not performance-shaped: (a) the pinned-stripe ratchet above, and (b) whether D3 item 2's prepaid tombstone cost is priced at the clustered or the scattered figure. If it is priced clustered (28 blocks) but the system can only deliver scattered (5909), the prepay is a lie under exactly the conditions it was invented for; if it is priced scattered, no reserve is needed for correctness and usable capacity collapses instead. The dossier does not say which — unverified dependency, and it is the fork that decides this sub-question.

Settling observations. Today, in a model: (i) add a metadata/data split and a held-out-reserve arm to E93, and report clustering-success rate for commit-intrinsic allocations only, with and without reserve, across a fill sweep; (ii) add wide stripes with partial-free pinning and check whether unreclaimable ratchets under fill-delete-refill with compaction disabled by ENOSPC. Real-filesystem only: whether the ratchet is reachable under real workloads.

Severity: blocks for the "deletion never depends on compaction" justification and the D3-principle dismissal. The standing-reserve sub-question is correctly left open, but for the wrong reason.

## P3 — snapshot accounting

Attack. P3(a) is the best-supported claim in the set (E94 negative arm: violations exactly equal to the independently counted number of moved versions referenced by ≥1 snapshot, all 60 cells). The attack is not on its truth but on its bookkeeping: D5 as quoted (F7) defines birth as "the checkpoint number at which the block was PUBLISHED." A moved block is published at the new position at a later checkpoint. Under D5's literal text, birth does change on move. P3(a) therefore is not an answer to D26 item 5; it is an amendment to a settled definition in D5, and if it is not recorded as one, an implementer reading D5 verbatim will reassign birth and reproduce E94's negative arm in production.

Second: P3(a) is also architecture-conditional, which P3 does not say. If a snapshot's reference is positional (D19, settled, F8), then keeping birth fixed while the position changes means the snapshot's own tree points at a stale position; the only repairs are rewriting the snapshot's tree (P1's problem) or resolving through a mapping layer (central_map). So P3 asserts (a) unconditionally while deferring the architecture that makes it coherent.

Third, on P3(c): "already quantified by E74" cites the cheapest of four costs. E74 covers the allocation-record btree only (2.6x for physically adjacent batches, tree costs 0.055% of capacity). The dossier contains no measurement of reverse-index update cost per move (D21, F8/F9), and E94's tree counts do not transfer (fanout mismatch above). Calling the accounting cost quantified is selective.

Fourth, a coupling P3 omits: D21 open item 1 (F9) says that if the reverse index is derived, compaction cannot run until it is rebuilt after loss — a stall window with zero test coverage. Chained with the P2 ratchet, "derived reverse index" becomes a path to unrecoverable ENOSPC, not merely a stall. P3 hands the architecture question to the user without this consequence attached.

Settling observation. Today: an E94 arm that keeps birth fixed and enforces positional snapshot references, to show the repair is forced. Today: a reverse-index cost arm. The stall window needs the real filesystem.

Severity: weakens for (a) and (b) — but blocks the recording step: D5's birth definition must be amended in the same round, not left as an implication.

## P4 — form of structural aging resistance

For the flip. Format decisions are one-way; adding an unspecified node field now is worse than adding one later behind an incompat bit. D11 has already spent the 16 KiB node budget on a measured allocation (ε = 0.65, fanout 119, 665 entries) — a leg-2 field would degrade a quantified design on an unquantified hunch. E93 does establish neighbor co-rewrite as a real, no-format-change aging-resistance form, and at equal low budget it beats compaction in two of three head-to-head cells (5497 vs 6095 at 3x uniform; 5111 vs 6073 at 1.5x run). P4 states its revert condition explicitly. The D8 freeze being blocked has real downstream cost.

Against the flip. E93 states in its own limitations that it has no arm for format-level node-layout changes, no time axis, no message-buffer batching statistics, and does not replace E10. The flip's premise is "the policy form suffices" — that is a comparison against the format form, and the model has no format arm. What E93 actually measured about the policy lever is that it saturates around F = 66% and is geometry-capped (only 1.5x extra writes on the run workload even at R = 2), while the lever that keeps scaling to F = 10–21% is compaction — i.e. leg 1, which was never what blocked D8. P4 resolves leg 2 by pointing at leg 1's success.

Two further points. First, one of the three named ingredients has zero support and is structurally incapable of doing the job: locality_id is inherited at creation and never updated on rename (F3). A static, never-repaired key prefix preserves initial locality; it cannot respond to aging by construction. E93 models no locality_id and no renames. Second, the "no competition with D11's message buffer" argument is unfalsifiable in the current evidence base, since E93 explicitly lacks message-buffer batching statistics — P4 cannot claim to avoid a problem the model cannot see.

The decisive asymmetry. The revert is not free. Today, flipping back costs nothing because no images exist. The entire purpose of the flip is to unblock D8's node-format freeze so images can start existing. If E10 later shows a node-layout change is needed, the revert is an incompat format break against live images, not a retraction. The flip converts a reversible open item into an irreversible one, on a model that has no arm for the alternative. Note also that the conservative hedge — reserving an incompat bit plus a reserved node-layout region so a later change is expressible — is itself a first-transaction-bytes change. So "NO" is only correct if the project accepts a future incompat break as the fallback.

My call: the flip is not supported today. The correct narrowing is P5-shaped: settle leg 2's runtime policy component now (clustered bump segments, optional R; drop locality_id from the aging-resistance list or justify it separately), and keep leg 2's format component open with item 4's verdict at YES until E10 or until a model arm exists for node-layout change. The cost of staying YES is bounded and known (D8's freeze waits). The cost of flipping wrong is unbounded.

Settling observation. Definitively: E10 — real filesystem, time axis, message-buffer batching — not obtainable today, which is the point. Partially obtainable today: an E93 successor with a node-layout arm, modeling node-level fragmentation and split/merge under aging with and without a hypothetical age/generation field. If a maximally favorable hypothetical field cannot beat the policy arm in the model, that is real evidence for NO.

Severity: blocks.

## P5 — trigger criterion

Attack 1. The proposed dual watermark is blind to the workload that a settled decision already assigned to leg 1. D2 item 11 (F10) makes leg 1 responsible for lazily re-laying pinned wide stripes under a budget, and pinned cells sit in unreclaimable. A disk can hold plenty of empty segments and low F while unreclaimable ratchets. Neither watermark fires; leg 1 has a settled duty with no trigger. This is a gap in the form, not in the thresholds, so P5's "item 1 narrows from criterion unknown to threshold values unknown" is not yet earned. E93 cannot supply the missing third watermark (no stripe model).

Attack 2. "Both quantities maintained O(1) and audited — E93 validated both properties for F." E93 validated them for F only. The empty-segment watermark is unmeasured, and it is the one with a scaling problem: maintaining a fully-free-segment count in O(1) requires per-segment occupancy state, which is O(capacity / segment size). F1 cell 1 says runtime decision cost "must not grow with disk capacity" without distinguishing time from space (conditional — the dossier does not resolve which is meant).

Attack 3. The watermark's definition is incomplete against two settled clauses that both care about it. F11 keeps space freed in checkpoint C out of the allocatable set until C publishes; F6 item 4 subtracts defer-pending-release from available; F6 item 6 grants no credit for in-window releases. So the empty-segment reading on the commit path is systematically pessimistic within a checkpoint, by an amount that scales with checkpoint size. P5 does not say whether the watermark counts defer-pending-release segments, and with hysteresis near a boundary that choice determines whether compaction fires for space that is about to arrive.

Attack 4. E93's audit agreement holds "at every sample point in every arm" — within E93's operation set, which has no crash and no redo. D8 item 5's middle crash state re-applies partial work idempotently (F4). An O(1) incremental F counter that is not idempotent under redo will drift from the checker's full scan, and the drift will be invisible until an audit. E93 has no arm for this.

Settling observations. All four are today-obtainable model work: a stripe/pin arm; an explicit empty-segment-watermark arm with an independent full-scan audit (mirroring what was done for F); a defer-window arm that varies checkpoint size; and a crash/redo arm that truncates and replays the last batch, then re-audits. Threshold values need E10.

Severity: blocks for attack 1 (form is incomplete); weakens for 2–4.

## P6 — observability

Attack 1 (policy-not-format dodge). The compaction sweep cursor cannot be a runtime-only reading. D8 item 5 (F4) requires unbounded work to be resumable via a persisted intent, with the three post-crash states defined in terms of it — and P1 leans on exactly this mechanism. The sweep's resume point and P6's "sweep cursor" observable are the same quantity. Either it lives in the intent record (persisted, contradicting "nothing persisted") or it is re-derived from scratch each mount, in which case the "idempotently redone" state restarts the sweep and progress is not guaranteed under a crash loop. F4 says the intent is an ordinary key in a logged-ops tree, so first-transaction bytes may indeed be unaffected — but a value schema that every future reader must parse is a format commitment regardless of which tree it lives in. That is the dodge: the item is being closed on "the bytes don't move" while a persisted schema is being introduced.

Attack 2 (the stronger one). "Nothing persisted" and P5's cell-1 compliance are in tension. The empty-segment watermark, F, and the fallback counter are trigger inputs on a runtime decision path (F1 cell 1). If none is persisted, then after every mount they either start blind or must be recomputed — and recomputing them is a full traversal, which is what E93's audit does. So the choice is: an O(capacity) mount scan, or persistence. F1 cell 1 rejects capacity-scaling cost for decision inputs, and D3's admission already reads published stats (F6 item 6), which implies persisted space accounting exists. Given that, "nothing persisted" is a preference, not a necessity, and P6 asserts it rather than arguing it. By P6's own rule ("persisting any of them reopens the item"), the item is already reopened.

Settling observation. Today: measure mount-time recompute cost for the empty-segment watermark and F as a function of modeled capacity in an E93 successor, and check whether the trigger's blind window after mount is bounded. The persisted-vs-derived choice for the reverse index (D21 open item 1, F9) should be decided in the same breath, since it has the same shape and a worse failure mode.

Severity: blocks the reasoning behind the "NO" verdict (attack 2); weakens the observable list (attack 1 — four of five are fine).

## Unverified dependencies flagged above

1. Whether D3 item 2's prepaid tombstone cost is priced at E81's clustered (28 blocks) or scattered (5909 blocks) figure. This decides P2's standing-reserve question.
2. Whether F1 cell 1's "cost must not grow with disk capacity" constrains time only or also space. This decides P5 attack 2 and P6 attack 2.
3. Whether wide stripes (D2 item 11) exist in the phase-1 pure-SSD line (D12 item 5). This decides whether P2's ratchet is live now or deferred.
4. E93's object/slot size — without it, E94's node counts cannot be converted to a metadata:data ratio for a compaction batch.
5. Whether published snapshot roots are formally immutable. The dossier calls them "published identity anchors," which implies it but does not state it; P1's severity depends on it.
