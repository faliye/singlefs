## E78 Starting Point for Replay —— Run (2026-09-02, count model, 16 unit tests / 9 mutations caught): **Superimposing stale tail × block reuse causes full-ring scanning recovery to self-abort on healthy images; solution is root with watermark or blind replay plus tail discard**

[中文](78-重放的起点.md) · **English** · [日本語](78-リプレイの開始点.ja.md)

D23 (Journal Role & Format) settled item 3 settled full-ring scanning recovery, and E24 (Recovery Algorithm: Does Trusting Tail First Lose Records) forced replay idempotence, concluding at the time that "stale tail merely causes extra work." **E24 (Recovery Algorithm: Does Trusting Tail First Lose Records) did not model block reuse**: I-7.4 (Blocks in Recent K Generations Not Reused) only reserves the most recent K generations (K ≤ root ring depth, on the order of dozens), whereas records in the ring number in the thousands—blocks named by records between stale tail and the latest root may have already been **legitimately** released and reused. Furthermore, E77 (Persistence Ordering of Publishes) has just proven that named units must be verified before applying replay. How to reconcile these two settled requirements, E78 (Starting Point for Replay) runs across four recovery algorithms.

### Criteria (Hardcoded Before Run in Source Header, Unmodified After Run)

1. Mismatch count pinned to closed form `max(0, M − L − PIN − tail)`, matching the independent reallocation tracker cell by cell; manual anchor points (M=13,tail=8)⇒0, (M=23,tail=16)⇒2. Void if mismatched.
2. Positive control: abort arm must abort on cells where closed form > 0. Failure to abort indicates the model lacks discriminative power and is voided.
3. Torn write discriminative power: under torn write scenarios, watermark arm and tail discard arm must set flag 1 with final state = M; skip arm must set flag 0 (silently swallowed).
4. Final state audit (no shared recovery code): recovery final state claimed to complete must match ground truth cell by cell; inequality eliminates that algorithm.

### Actual Run (L=4 Object Rotation COW, Ring 64 Slots; Primary Cell M=23 / PIN=1 / tail=16, Mismatch Closed Form = 2)

| Algorithm | Healthy Image | Torn Image (Record Present, Unit Not Landed) |
|---|---|---|
| **Abort arm**: replay from tail + verify on apply + abort on mismatch | **Aborted**—treats stale mismatch as corruption, **judges recovery failure on healthy image** | Also aborted (hits stale mismatch before hitting tear) |
| **Skip arm**: replay from tail + verify on apply + skip on mismatch | Completed, silently skips exactly 2 records | Completed, but **flag 0**—tear and stale share same counter, indistinguishable |
| **Watermark arm**: root carries jsn watermark, only replay > watermark | Completed, replays 0 records | Completed, **flag 1** (mismatch outside watermark falls in in-flight window ⇒ judged torn) |
| **Tail discard arm**: blindly replay full prefix + review at close + retry on tail discard | Completed, 0 retry rounds | Completed, exactly 1 round of tail discard, **flag 1**, remaining 7 records in window preserved |

Boundary cells confirm this is a **bug shape that bites only when tail is sufficiently stale**: when window < L + PIN (tail was just advanced), all four algorithms complete cleanly—a green test does not mean all is well. Increasing PIN monotonically decreases mismatches (2→1 when PIN 1→2).

### Three Conclusions

1. **The trio of "full-ring scan + stop on broken sequence + idempotence" does not close.** I-8.4 (replay idempotent) governs "repeated application of the same record", but cannot cover "blocks named by old records having been reused"—the latter is a **verification mismatch** during apply verification, not a harmless replay. The abort arm (treating mismatch as corruption) self-aborts on healthy images, representing an **availability failure**; the skip arm preserves availability at the cost of zero torn write discriminative power.
2. **Recovery requires a "replay floor", which had no placement when run on 2026-09-02.** The watermark arm resolution requires the root record to carry a jsn watermark field (width is 10 bytes in D23 (Journal Role & Format) settled item 9)—⚠️ If "every publish advances checkpoint_txg" holds, the existing `checkpoint_txg` in the record header can serve as the watermark; if one txg covers multiple publishes, records and roots under the same txg cannot be ordered, necessitating a separate field. **This depends on an equality that no one hardcoded**, with placement in [checks-owed.md](../checks-owed.md) C88 (Timeline discrimination for root ring unimplemented) (formerly C85 (Four Equalities of the Publish Counter Not Hard-Coded), merged 2026-10-11).
3. **The tail discard arm resolution requires no new format fields**, at the cost of blindly replaying the full window + at most as many tail discard rounds as the in-flight window; its correctness has a structural precondition (objects whose blocks were reused within the window must have later records), which this model proves holds automatically by construction under COW rewrites (pinned by unit tests). **Both watermark arm and tail discard arm pass all criteria; selecting which one is a decision—Settled (2026-09-02): adopt watermark arm, authoritative record in D23 (Journal Role & Format) settled item 14.**

### Failure Clauses (Hardcoded Before Run)

Criteria 1 and 2 fully hit (closed form == tracker, all four cells correct; abort arm indeed aborts on cells where closed form > 0). Did not trigger voiding.

### What It Cannot Answer

Count model, 0 file operations. Does not answer: how stale tail can become under real workloads (tail is written once with each publish, D23 (Journal Role & Format) settled item 3); implementation costs of watermark arm and tail discard arm; remnant records masquerading as prefix category (within range of E32 (Remnants of the Previous Timeline); this model's ring window ≤ ring length, no wrap-around).

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e78_replay_start.rs` (`cargo run --release --bin e78-replay-start`), raw output `e78-replay-start-2026-09-02.out` (42 lines, concluding line `emitted=42`).
- Deterministic model (FIFO reuse pool), identical md5 across 5 rounds. **16 unit tests, 9 mutations caught item by item, 0 blind spots**, mutation table `scripts/mutation/tables/e78_replay_start.tsv` (`bash scripts/mutation/mutate.sh e78-replay-start scripts/experiment-apparatus/e78_replay_start.rs scripts/mutation/tables/e78_replay_start.tsv`).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 14 | Supports | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only modified wording (temporal words, location references and self-references); conclusions, figures and artifacts unchanged |
| D23 (Journal Role & Format) settled item 3 | No impact | 2026-10-17 unaffected: this round only synchronized wording, artifact pointers, paths and numbers; conclusions, figures and artifacts unchanged; previously 2026-10-11 unaffected: only modified wording (temporal words, location references and self-references); conclusions, figures and artifacts unchanged |

## Historical Versions

Historical entries for E78 (Starting Point for Replay) are centralized in [experiments-history.md](../experiments-history.md).
