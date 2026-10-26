## E82 In-Flight Synthesis for Admission —— Run (2026-09-02, pure arithmetic, 7 unit tests / 6 mutations caught): **Three intuitive approaches each produce countable violations: reading only published values oversells 80/120, issuing credits for intra-window releases prematurely reuses 20 cells, forgetting tombstone reservations deadlocks all 100 deletions after disk fills**

[中文](82-准入的在飞合成.md) · **English** · [日本語](82-受付-アドミッション-のインフライト合成.ja.md)

All six terms of the admission / acceptance inequality (authoritative text D28 (Mount-Time Commitments) settled item 1, formerly residing in D3 (Space Allocation) settled item 4 prior to 2026-09-13) were **published** metrics at the time, whereas admission occurs mid-window—the synthesis layer of "admission reading = published value + in-flight overlay" had zero coverage repository-wide (C87 (Synthesized value of admission reading undefined)). E82 (In-Flight Synthesis for Admission) casts three silently erroneous implementations into countable violations, providing must-red test cases for admission acceptance paths.

### Criteria (Hardcoded Before Run, Unmodified After Run; All Closed-Form)

1. published_only oversells exactly = K×R − available; overlay overselling is constantly 0 and approves exactly ⌊available/R⌋.
2. Credit-issuing arm prematurely reuses exactly = B requirement − creditless available; correct arm is constantly 0.
3. Forgetting tombstone reservation ⇒ deletion deadlocks after disk full exactly = attempted count; reservation form deadlocks constantly 0, storing ⌊CAP/2⌋ objects.
4. Three violation counters each yield non-zero readings (C60 (Constant Readings Lack Fault Injection Self-Test)); correct arm all 0.

### Actual Run (CAP=100 Manual Scale; Full Table in Artifacts)

| Arm | Result |
|---|---|
| published_only (K=4 × R=30, published available 40) | All 4 pass gate, **overselling 80 cells**—failing only mid-allocation |
| **overlay (correct form: published − in-flight approved)** | Exactly 1 approved, 3 rejected, overselling **0** |
| credit_frees (treating intra-window releases as available) | B passes gate, **20 cells consume blocks that defer should hold**—D16 (Publish Semantics) settled item 15 silently bypassed |
| forget_tombstone (admission omits "unraveling self") | Stores 100 objects, after disk full **all 100 deletions deadlock**; reservation form stores 50 objects, 0 deadlocks |

⇒ **Correct form in one sentence**: admission reading = published statistics − in-flight approved; intra-window releases **do not issue credits** (admission-side form of D16 (Publish Semantics) settled item 15); "unraveling self" includes the tombstone cell (executable reading of D3 (Space Allocation) settled item 2). Each of the three is paired with a counter that turns red, ready for crash-point replay and admission unit tests.
**Correct form settled (2026-09-02): D28 (Mount-Time Commitments) settled item 2 (formerly D3 (Space Allocation) settled item 6 prior to 2026-09-13).**

### Failure Clauses (Hardcoded Before Run)

Criteria 1–4 fully hit. Did not trigger voiding.

### What It Cannot Answer

Pure arithmetic, 0 file operations. Does not model defer window duration, multi-device dimension, unreclaimable capacity; "one tombstone cell" is an upper-bound measurement basis prior to settling granularity in E83 (Granularity of Tombstones)—once granularity settles constants change accordingly, while the verdict shape remains unchanged.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e82_admission_overlay.rs` (`cargo run --release --bin e82-admission-overlay`), raw output `e82-admission-overlay-2026-09-02.out` (8 lines, concluding line `emitted=8`).
- Pure arithmetic, identical md5 across 5 rounds. **7 unit tests, 6 mutations caught item by item, 0 blind spots**, mutation table `scripts/mutation/tables/e82_admission_overlay.tsv` (`bash scripts/mutation/mutate.sh e82-admission-overlay scripts/experiment-apparatus/e82_admission_overlay.rs scripts/mutation/tables/e82_admission_overlay.tsv`).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D28 (Mount-Time Commitments) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D28 (Mount-Time Commitments) settled item 1 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 15 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 17 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 4 | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 6 | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) settled item 15 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E82 (In-Flight Synthesis for Admission) are centralized in [experiments-history.md](../experiments-history.md).
