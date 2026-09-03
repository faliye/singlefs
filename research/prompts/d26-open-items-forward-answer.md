# D26 open items — forward-derivation leg (Sonnet, 2026-09-03), verbatim

Scope note up front: the dossier never states the "existing verdict" (YES/NO on first-transaction bytes) for item 4 or item 6 as an F-numbered fact — P4 and P6 each assert a prior verdict without a citable source in this document. Both derivations import that premise from outside the dossier; flagged once here, applies to both.

## P1 — transaction shape

Rests on: F1 (cell 3), F3 (D8 index structure), F4 (D8 item 5 intent mechanism), F5 (CommitStep enum).

- New-location writes = WriteDataBlock, reference updates = WriteInternalNode, publish = CasRootPointer/AppendRootRecord — follows from F5; these are pre-existing enum members and the claim holds independent of which position-authority architecture (P3) is eventually chosen, since both E94 architectures (ptr_rewrite and central_map) only touch tree nodes and root records.
- "No new structural step needed, so leg 1 stays non-format-level" — follows from F5's own rule that parameter changes (how many nodes/repetitions) are not new steps.
- "Unbounded whole rides D8-item-5's intent mechanism" — hidden assumption: F4 names truncate and big deletes explicitly; compaction is not on that list. Extending the "exactly three post-crash states" guarantee to compaction batches is a plausible analogy, not a verified one — no experiment in the dossier crash-tests a compaction intent record.
- Note: F2 already states leg 1 is "explicitly NOT format-level" as a settled characterization, so P1's conclusion is partly restating a given fact rather than deriving a new one; its actual contribution is the narrower claim that the shared enum needs no diff, which does follow.

Verdict: holds only if D8-item-5's intent mechanism is confirmed (by crash-replay, not yet run) to generalize to compaction batches.

## P2 — space reservation

Rests on: F6 (D3 principle, items 2/4/5/6), F10 (pinned stripes/unreclaimable), F11 (D16 rule 2).

- "Freeing-never-allocates governs the freeing path only; compaction is a consuming op" — follows; F6 item 2 ties the prepaid-tombstone guarantee to deletion specifically, and F6 item 4's inequality already has a dedicated slot ("committed reservations") for in-flight writers, which compaction fits without special-casing.
- "Old locations free via defer after publish" — follows directly from F11 (space freed within checkpoint C not allocatable until C publishes) plus F6 item 6 (in-flight overlay is pure derived state).
- "Stalling compaction under a full disk is acceptable because deletion never depends on compaction" — follows deductively from F6 item 2 (tombstone cost prepaid at write time); this is a correct deduction from an already-settled decision, not something requiring new measurement.
- Unaddressed: whether compaction's own metadata COW (counted in its reservation) recursively needs the same tombstone prepayment as any other COW write. The dossier's principle (F6) is general enough to cover this, so it's a minor, not a real gap.
- The standing-reserve question is correctly left open rather than answered, citing E93's finding that compaction is the only sustaining force for empty-segment supply — appropriately scoped, not overclaimed.

Verdict: holds as written.

## P3 — snapshot accounting interaction

Rests on: F7 (birth definition), F8 (D9 items 5/7, open item 9), F9 (D21 open item 1), E94 (all three arms + negative arm), E74.

- (a) birth never changes on move — follows, direct citation of E94's negative arm (violations = exactly the moved-and-referenced count) and consistent with F7's definition of birth as tied to the publish checkpoint, not to physical position.
- (b) deadlist rewrite cost depends on key choice — follows for the ptr_rewrite number (11–36 rewrites, explicitly stated as "deadlist entries keyed by position" in the dossier). For central_map, the dossier reports "zero deadlist rewrites" but does not explicitly state the deadlist is logical-identity-keyed in that arm — P3 infers the key scheme from the observed rewrite count. Reasonable inference, but it is an inference, not a quoted fact: mild hidden assumption.
- (c) allocation-record cost — follows, direct citation of E74.
- The proposal correctly declines to resolve which architecture owns position authority, deferring to C34/D9-item-9/D21-item-1, and hands over E94's numbers as input rather than as a settled answer.

Verdict: holds as written, precisely because it stops short of resolving the architecture question it correctly identifies as unsettled.

## P4 — form of structural aging resistance (the flip)

Rests on: F3 (locality_id, D11 epsilon/fanout), F6 (D3 item 5/E81), E93 (all arms), and the explicit E93 disclaimer.

- locality_id as an existing key-prefix mechanism — follows from F3 (already settled, D8 item 3).
- Clustered bump-segment placement as an aging-resistance component — follows from F6/E81 for the "reduces metadata-write fixpoint cost" claim, but E93 itself shows clustered allocation without compaction is not self-sustaining (91.8–93.8% fallback) — P4's own framing implicitly relies on leg 1 (compaction) to keep this working, which is consistent with "stacks with leg 1" but weakens the claim that leg 2 is doing independent structural work.
- Neighbor co-rewrite as a no-format-change technique — follows, E93 explicitly calls this a "no-format-change aging-resistance form," but the same result shows it saturates at F≈66% and is budget-capped by workload geometry — i.e., on its own it is insufficient, and the dossier's own numbers show compaction (leg 1), not this policy knob, is what drives F down further (10–21% at 5x budget).
- "No competition with D11's message buffer for node bytes" — hidden assumption, not a derived result. The dossier states outright that E93 has "no message-buffer batching statistics" and "no arm for format-level node-layout changes," and explicitly "does NOT replace E10." There is no fact or experiment in this dossier that tests interaction with the real B-tree's message buffer (epsilon = 0.65, fanout 119, per F3). Absence of a tested competition is not evidence of no competition.
- The premise that item 4's prior verdict was "YES" is asserted, not sourced to any F-number in the dossier (see scope note above).

Explicit answer to the flip question: the YES→NO flip is not justified by the dossier's evidence alone; it requires E10. Every load-bearing step for "policy is sufficient, format need not change" routes through exactly the part of the design space (message-buffer interaction, real time-axis aging, format-level alternative) that the dossier says E93 does not cover. P4's own "risk exposure" clause concedes this ("if E10 later shows the policy form is insufficient... the flip reverts"), which is itself an admission that the flip is provisional, not derived — the proposal's action language ("flips to NO," "no longer blocked") overstates what that concession supports.

Verdict: does not follow as an unconditional flip; holds only as a provisional, reversible working assumption pending E10.

## P5 — trigger criterion

Rests on: F1 (cell 1/cell 2 split), F6/E81, E93 (F metric and fallback data).

- F = runs/L, O(1)-maintainable with independent full-scan agreement — follows directly from E93 and matches F1's cell 1/cell 2 discipline (runtime O(1), checker independently traverses and agrees) essentially exactly.
- Empty-segment watermark tied to allocation self-sustainability and E81's clustering benefit — follows from E93's fallback-rate data and F6's E81 citation.
- "F guards scan locality" — not a phrase used in the dossier's description of E93; this is a reasonable gloss on what the metric measures (contiguity in key order) but is an interpretive extension, not a quoted result.
- Deferring thresholds to E10 while settling only the form — follows appropriately, since E93 explicitly lacks a time axis and the dossier flags E93 as not a substitute for E10.

Verdict: holds as written.

## P6 — observability

Rests on: E93 (F, fallback, watermark), E94 (per-batch node counts), F1 (cell 1), F4 (batch/resumability), F9 (stall-window precedent).

- F reading, empty-segment watermark, fallback counter — follows, direct E93 outputs.
- Per-round moved bytes/batch count — follows, matches E94's measured per-batch node-rewrite counts.
- Compaction sweep cursor — hidden assumption: not an experimentally observed quantity in either E93 or E94; it's inferred from F4's general batch/resumability mechanism, same unverified extension flagged under P1.
- "Nothing persisted" — gap: if these values must be O(1)-maintained on the runtime decision path (F1 cell 1) but are never persisted, they must be reconstructed by a full scan after every crash/remount. The dossier does not say whether that reconstruction is a bounded one-time mount cost (acceptable) or a repeated cost, and this is exactly the shape of risk F9 already flags for the reverse index ("if derived, compaction cannot run until index rebuilt after loss — a stall window with zero test coverage today"). P6 does not address whether its own observables inherit that same untested stall window.
- "Existing verdict NO stands" — premise not sourced to an F-number in this dossier (see scope note).

Verdict: holds only if the post-crash reconstruction cost of these unpersisted values is shown to be bounded/one-time; this is currently unverified and structurally analogous to F9's flagged gap.

## Closing

The single weakest inference is P4's YES→NO flip: it treats E93's results on an abstract object/slot model as sufficient grounds to unblock a format freeze, while the dossier itself states, in the same breath as those results, that E93 has no message-buffer statistics and no format-level arm and does not replace E10. Every other proposal's weak points are either honestly scoped as open (P2's reserve question, P3's architecture question, P5's thresholds) or are minor unverified extensions of an existing mechanism (P1's and P6's reliance on D8-item-5's intent shape generalizing to compaction). P4's is the only one where an unverified step is used to actively change a blocking verdict on a format freeze. The single measurement that would close it is E10 itself, specifically run with an arm that varies node-layout format alongside the policy techniques, and with message-buffer batching statistics recorded so the "no competition with D11" claim can be checked rather than assumed. A secondary, cheaper fix: source the "existing verdict" premises for items 4 and 6 to an actual document location so the flip and the "stands" claims rest on a citable fact rather than an asserted one.
