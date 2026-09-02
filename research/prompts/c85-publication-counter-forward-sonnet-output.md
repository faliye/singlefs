FORWARD DERIVATION - publication counter question (C85/C77), 2026-09-02, cloud leg A (Sonnet)

Verified against the repo: F1 (D16 fsync = early publication, shared checkpoint cadence) holds. F3 (D22 item 2 region = txg mod R; I-7.3 older-generation root must survive) holds. F4 (D5 birth/captured-txg/generation, K = root slot total + 1 per E54) holds. F5 (D23 item 9 widths and jsn rationale) holds. D16 new-rule 1 (snapshot pinned after this checkpoint's data changes; two same-window snapshots only with empty deadlist between) holds and is load-bearing.

Step 1, D16 item 5: fsync is the same publication event as a timed checkpoint, merely early; no carve-out for a minor fsync checkpoint that skips ring rotation or generation bookkeeping.

Step 2, D22 item 2 + I-7.3: ring placement takes txg as its only input. Two same-window publications sharing a txg compute the identical region and slot; the second overwrites the first while both are "the same generation" - I-7.3 silently defeated; a previously durable root is gone with no fallback. D22/I-7.3 as written presuppose a placement key that changes on every publication.

Step 3, D5 + new-rule 1: the O(1) death-classification proof rests on txg total order and judgement at death instant; new-rule 1 pins a snapshot after this checkpoint's changes and permits two same-txg snapshots only when nothing changed between them. If an fsync publishes mid-window and more data is written before a snapshot follows, coarse txg gives the fsync-root and the snapshot the same S.txg despite non-empty change in between - violating new-rule 1's precondition - and inclusive birth <= S.txg misattributes later blocks. Fine-grained txg satisfies new-rule 1 automatically.

Consequences of pinning: (a) region = txg mod R changes every publication, I-7.3 checkable as written. (b) every fsync gets its own birth stamp. (c) accounting generation ticks at fsync rate, up to 2785/s - a real new cost E54 never priced (E54 judged cost-per-tick, not ticks-per-second); K spans less wall-clock under fsync bursts. (d) 8-byte txg at 2785/s: over 1e11 years headroom. (e) checkpoint_txg becomes the E78 replay watermark at zero new format bytes.

Consequences of the alternative: ring rotation needs a different monotonic input; the only settled global monotonic counter is jsn, which is per-record, so a third counter or new coupling is minted either way. Replay classification of records sharing the root's coarse txg requires the separate 10-byte watermark to redo the ordering per-publication txg gives for free.

Verdict: pin the sentence. Strongest derivation: steps 2 and 3 - D22/I-7.3 and D5/new-rule 1 already presuppose a per-publication counter; the alternative relocates the need into jsn while leaving the ring collision unresolved. Wording amendments: D16's fsync clause should state the early publication receives its own checkpoint_txg; D5's K-generation retention should note generation-to-wallclock is no longer one-per-window.
