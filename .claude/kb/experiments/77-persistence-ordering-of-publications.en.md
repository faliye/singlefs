## E77 Persistence Ordering of Publishes — Run (2026-09-02, Crash Subset Exhaustion Model, 18 Unit Tests / All 8 Mutations Caught): **Exactly One Necessary Barrier (Prior to Root Slot), and Named Units Must Be Validated Prior to Replay Application**

[中文](77-发布的持久顺序.md) · **English** · [日本語](77-発行の永続順序.ja.md)

A single publish writes three classes of entities: COW units, journal records, and root slots. Across the entire repo only two deductive sentences touched persistence ordering among them
(D25 (Target Workload Priority) acknowledged "this is deduction, not empirical measurement"; D23 (Journal Role & Format) "already written to disk"
did not specify dispatched or persisted), while ordering between records and root slots and between units and records had **zero coverage** (confirmed by grep).
E77 (Persistence Ordering of Publishes) exhaustively evaluated all crash subsets under four barrier arrangements.

### Criteria (Hardcoded Before Run in Source Header, Unchanged After Run)

1. Violation = root slot persisted but recovery ≠ new state or traversal failure; or partial transaction recovered; or unit failing checksum grafted.
   When root slot is not persisted, both old and new states are valid (fsync did not return).
2. Crash state count must equal closed form (b_all 72 / b_ur 79 / b_rs 513 / b_none 1024); miscount invalidates entire round.
3. Positive control: Replay without validating named units (naive) must yield exactly 63 violations on b_rs (= 2⁶ − 1); failure to turn red invalidates round.
4. Which barriers are necessary is governed by numbers, without preconceived conclusions.

### Measurement (Model: 6 Units + 3 Records of Same Transaction (Commit Marker on Last) + 1 Root Slot; Arbitrary In-Segment Subset Persisted)

| Arm (Barrier Arrangement) | State Count | Violations (Validated Replay) | Violations (Naive Replay) | Holes in Record Stream |
|---|---|---|---|---|
| b_all [units][records][root slot] | 72 | **0** | 0 | 0 |
| b_ur [units][records+root slot] | 79 | **0** | 0 | **7** |
| b_rs [units+records][root slot] | 513 | **0** | **63** | 0 |
| b_none [all free] | 1024 | **504** | 567 | 448 |

Three conclusions, each pinned by closed forms:

1. **Data integrity requires only a single barrier: prior to root slot** (b_rs 0 violations, b_none 504 violations = all combinations
   where root slot present while units missing). This empirically confirms at model level the deduction in D25 (Target Workload Priority) of "2 persistence points per fsync":
   one FLUSH prior to root slot + root slot's own FUA is sufficient.
2. **No barrier needed between units and records, on premise that named units are verified item-by-item prior to replay application** — removing verification (naive),
   b_rs immediately manifests 63 silent graftings. ⇒ **"Each named item in record carries its own checksum" (D23 (Journal Role & Format)
   settled item 17) is not a decorative luxury, but the trade-off condition permitting this barrier to be omitted**; recovery path must genuinely verify it.
3. **Ordering of record → root slot buys record stream integrity, not data**: b_ur has 0 data violations, but possesses 7 states
   where "root slot on record while records absent" — inputs to record checker and back-chain (D23 (Journal Role & Format) settled item 8)
   contain holes in those states. Whether to pay a second barrier for this is a decision, not an experiment.

### Failure Clauses (Hardcoded Before Run)

Criteria 2 and 3 both hit (state count cell-by-cell equals closed form per arm; naive exactly 63). Invalidation not triggered.

### What It Cannot Answer

Pure enumeration model, 0 file operations. Does not answer: whether FLUSH is effective on real devices (scope of C6 (Wrong Block Layer Semantic Assumptions Written)),
cost of FUA vs FLUSH, interleaving of multiple publishes and ring wrap-around (scope of E78 (Starting Point for Replay)).
"Arbitrary subset in segment persists" is lower bound model of block layer guarantees (D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment):
"block layer never promised a bio would not be torn").

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e77_publish_order.rs`
  (`cargo run --release --bin e77-publish-order`),
  raw output `e77-publish-order-2026-09-02.out` (12 lines, ending with line `emitted=12`).
- Exhaustive enumeration without randomness, identical md5 across 5 rounds. **18 unit tests, all 8 mutations hit one-by-one, 0 blind spots**,
  mutation table `scripts/mutation/tables/e77_publish_order.tsv`
  (`bash scripts/mutation/mutate.sh e77-publish-order scripts/experiment-apparatus/e77_publish_order.rs scripts/mutation/tables/e77_publish_order.tsv`).
- ⚠️ Mutation testing caught two blind spots in model itself, subsequently fixed: recovery function "success" was previously self-reported
  (audit and audited shared origin), and traversal completeness was previously masked by audit fallback — both remedied via "prove it turns red first".

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 4 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-19 unaffected: segment model state counts across four barrier arrangements equal closed form per arm; crash states segmented by barrier, arbitrary in-segment subset persistence counted correctly |
| D16 (Publish Semantics) settled item 7 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-20 unaffected: without barrier prior to root slot, 504 of 1024 crash states violate; removing named unit verification prior to replay application, b_rs manifests 63 silent graftings — figures in settled item 7 originate here |
| D23 (Journal Role & Format) settled item 14 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-21 unaffected: E77 (Persistence Ordering of Publishes) measured whether barriers needed between publish steps and prefix measurement basis of pre-application named unit verification, not failure disposal discriminator; modification B changed only how discriminator triages failures, fourth prefix measurement basis holds as before |
| D23 (Journal Role & Format) settled item 17 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-19 unaffected: "each named item carries checksum" is trade-off condition for omitting barrier between units and records |
| D23 (Journal Role & Format) settled item 8 | Material prep | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-19 unaffected: when records and root slot in same segment, states exist where root slot present while records absent, creating holes in back-chain and record checker inputs; whether to pay second barrier for this is a decision |
| D25 (Target Workload Priority) settled item 5 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-20 unaffected: data integrity requires only one barrier prior to root slot, deduction in D25 (Target Workload Priority) of "2 persistence points per fsync" holds at model level — barrier measurement basis deduction holds firm, real-device numbers remain unmeasured |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-19 unaffected: "arbitrary in-segment subset persists" adopts lower bound model of block layer guarantees, citing solely that fact from D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) |

## Historical Versions

Historical entries for E77 (Persistence Ordering of Publishes) are concentrated in [experiments-history.md](../experiments-history.md).
