# C72 切分收口·sonnet-forward 原样输出（2026-09-03）

## Overall verdict: F11 survives, but only with an amendment to D6's wording

The chain F1→F2→F3→F4 is internally sufficient to establish "v1 = whole-unit-only, one pointer per unit, extent_off≡0" as a *derived consequence*, not a new assumption. Where the derivation snags is F8: as paraphrased, D6 currently attributes "cross-snapshot partial-overwrite fragmentation" to splitting/extent_off, and that attribution is inconsistent with F2 once F2 is taken at face value. That's a defect in D6's existing text, not a defect in F11's reasoning — but it must be fixed for F11 to be adoptable, and I can't certify it fixed without D6's actual current wording (only F8's paraphrase is in evidence here).

### (a) Does any v1 operation produce real splitting (extent_off ≠ 0)?

Walk the concrete case F8 points at: file's unit U0 is snapshotted, then live file writes 100 bytes inside that unit. Per **F2**, this is RMW: read U0, verify MAC, modify, recompute checksum for the *whole* unit, write the *whole* new unit U1 to a new location. Snapshot keeps pointing at U0 (untouched, self-contained, MAC still valid); live file's pointer flips to U1 (whole unit, offset 0, fresh nonce/MAC). Nothing here needs a pointer that locates a sub-range inside a shared ciphertext — the two versions are two separate, self-contained whole units. **F2 already rules out the alternative** ("store a delta + reference the unchanged rest of the old unit," which is what would actually require extent_off ≠ 0) by settling that there is *no partial in-place overwrite* and *no partial-unit-referencing-new-location* — it's always full-unit RMW to a new location. So **F8's characterization of D6 doesn't survive F2**: D6's "paid for by extent_off" framing describes a mechanism (delta-splitting to save the cost of a full-unit copy under snapshot pinning) that F2 already foreclosed. That's the amendment target, not a refutation of F11.

Reflink/dedup: sharing a whole unit at offset 0 from two pointers is not splitting — extent_off stays 0, it's ordinary reference sharing (consistent with F10's "what changes is whether the MAC gets copied into k copies," which is exactly reflink/dedup, not splitting).

RAID rebuild/resilver, journal replay: these reproduce existing physical/logical state (from parity or from a logged commit that itself, per F2, was already whole-unit); nothing in F1–F10 gives them license to originate a new pointer topology, so they inherit "no splitting" from whatever they're replaying/rebuilding.

**Gap, not refutation**: D2 (defrag/restripe) and D18 (tombstone packing) are named in the question but have **zero F-facts given about their content**. I cannot forward-derive "no splitting" for them — this is unverified, not disproven. F11 needs an explicit check against D2/D18's actual text before "v1 has no splitting" can be fully certified.

**Verdict on (a)**: no *given* v1 operation produces extent_off ≠ 0, once D6's current wording is corrected to match F2. D2/D18 are open.

### (b) Does "reserved, disabled in v1" contradict settled text, or is "delete the concept" forced?

Not only does it not contradict — **F9 already says this**: "extent_off width budget... reserved regardless of whether splitting happens." That's the settled policy already; F11 isn't inventing reserved-but-off, it's applying a rule that's already on the books, and F5 gives the working precedent (compression bits reserved day-1, unused in v1, not "compression doesn't exist"). "Delete the concept entirely" would mean contradicting F9's own text, which explicitly anticipates the off-case. Nothing in F1–F10 forces the stronger deletion ruling — F6's stated mechanism (N sub-extents sharing one ciphertext/nonce/MAC via offset) is structurally exactly what compression-packing would need if a future compression design chooses to pack multiple compressed sub-extents into the slack of one unit (plausible, matches F11's own text) — though note F5 leaves compression's actual mechanism **undecided**, so this specific future justification is a hypothesis, not settled. It doesn't need to be settled for (b): F9's blanket "reserve regardless" already carries the weight.

**Verdict on (b)**: no contradiction; deletion is not forced; F9 alone justifies "reserved, off."

### (c) Does re-grounding D9 item 4's reason (F7) break any AAD/encryption reasoning?

No breakage, but a tense/framing fix is needed. In v1, extent_off is constant 0 for every unit, so whether it's included in the AAD is *unobservable* in v1 — one pointer, one AAD value, one MAC, always self-consistent either way. F7's stated math (N AADs against 1 MAC → N−1 verification failures) is a **real constraint that only bites once N > 1**, i.e., only if/when splitting is enabled. So the re-grounded reason ("must stay excluded so that *if* splitting is ever turned on, it doesn't break then") is logically sound and actually *more* accurate than F7's current phrasing, which reads as if splitting is a live, present-tense mechanism ("under splitting... every normal read"). The ruling (exclude from AAD) is unchanged; only the tense of the justification needs to shift from present/active to conditional/reserved — matching F10's own hedge.

**Verdict on (c)**: ruling intact, no broken reasoning; F7's tense needs to change from active to conditional.

### (d) Is extent_off dead weight in v1, should it be dropped?

No — dropping it would contradict the fixed-width "reserve, don't grow later" policy **already settled** for this exact field (F9) and applied identically to the compression-algorithm field (F5), which nobody is proposing to drop despite being equally unused in v1. Singling out extent_off for removal while keeping the compression bits reserved would be an inconsistency the given facts don't support. F10 confirms the reserved bits are already "occupied whether used or not" and cost nothing extra to keep (no first-transaction byte change either way).

**Verdict on (d)**: reserved bits are justified; not dead weight to drop, by direct application of F9.

## Exact wording changes required

1. **D6 (snapshot model)** — the phrase describing "cross-snapshot partial-overwrite fragmentation" as a cost "already pre-paid by D9 day-1 reserved field #2": remove the attribution to splitting/extent_off. Replace with: this scenario resolves via ordinary whole-unit COW under D4 item 3 (F2) — old unit retained unchanged for the snapshot, live write produces a new whole unit at a new location; no sub-unit sharing occurs. **This is the one change I cannot certify without D6's actual current text** — only F8's paraphrase was supplied here; the assembler should re-read D6 directly before applying this edit.

2. **F6 / D9's description of extent_off's existence reason** — change from present-tense ("after splitting, N sub-extents SHARE...") to conditional/reserved-future tense: "IF splitting is ever enabled (not in v1; v1 has extent_off≡0 for every unit)..."

3. **F7 / D9 item 4's stated reason** — reword from "prevents a v1 bug" framing to: "extent_off is excluded from the AAD to preserve forward-compatibility for a currently-inactive, reserved mechanism; this costs nothing in v1 (extent_off≡0 makes the AAD-inclusion choice unobservable today), and is required so that *if* splitting is later enabled, N sub-extents sharing one MAC won't cause N−1 verification failures."

4. **Before final sign-off on F11**, D2 (defrag/restripe) and D18 (tombstone packing) must be read directly and checked for any DATA-extent operation producing multiple pointers into one physical unit at different offsets. Not covered by F1–F10 as given; currently an open item, not a settled "no."
