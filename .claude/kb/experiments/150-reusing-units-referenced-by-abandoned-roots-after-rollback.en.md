## E150 Rollback Reuse of Units Referenced by Abandoned Roots — Run (twice on 2026-09-13, deterministic model, 16 unit tests / all 13 mutations caught); the rollback shape it measured (an instance switch at mount) no longer exists since D23 (Journal Role & Format) confirmed item 14 made rollback a forward publish while mounted, so its conclusions are history only
<!-- doc-lint:not-numbers R0 R1 R2 R3 R4 R5 M2 M3 M13 -->
[中文](150-回退复用被抛弃的根引用的单元.md) · **English** · [日本語](150-ロールバック後に破棄されたルートが参照するユニットを再利用する.ja.md)


**Question**: Are the two cases in C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused) reachable in one counting model: a crash during rollback publish after units but before the record, and reuse after rollback confirmation followed by making all roots of the rollback instance unreadable? Do the two fixes—shadow accounting (units referenced by abandoned roots cannot be reused until those roots leave the root ring) and tombstones (overwrite abandoned root slots with tombstones using FUA before rollback)—block each case, and what does each cost? This is a prerequisite stated by D16 (Publish Semantics) settled item 8 (warm-up arm 戊).

Pre-run registration `e150-preregistration.md` contains the question, verbatim subject under test, model, three arms, six criteria, failure clauses and limitations. Its final section corrects an anchor halfway through apparatus construction, before any artifact: the baseline must also protect the rollback candidate set under I-7.4 (Blocks in Recent K Generations Not Reused), changing the anchor slot from 1 to 3.

**Conclusion (after the run)**: Both cases of C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused) are reachable in the model. The baseline has violations in 3 of 5 rollback-publish crash states and 19 of 26 combinations where all roots of the rollback instance are unreadable. Both shadow accounting and tombstones have zero violations in both cases. Costs: shadow accounting quarantines 2–3 additional units under the geometry with a ring capacity of 12, falling to zero only at the 11th publish after the rollback root (when the slot containing the newest abandoned root is overwritten by rotation). Tombstones add one FUA per abandoned root in the ring on each rollback (two here); if a crash occurs after the tombstones persist, recovery has already fallen back to R_old. Thus D23 (Journal Role & Format) settled item 14's statement that “rollback and its first new root are published together and there is no persistent effect before that” no longer holds under the tombstone arm. **The choice between fixes is a cost trade-off for the user** (a failure clause fixed before the run).

**Rerun command** (`exact` mode, byte-for-byte comparison with the retained artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E150
# 直接跑装置：
cd scripts/experiment-apparatus && cargo run --release --bin e150-rollback-reuse-of-abandoned-roots
```

Code: `scripts/experiment-apparatus/e150_rollback_reuse_of_abandoned_roots.rs`; raw output: `e150-rollback-reuse-of-abandoned-roots-2026-09-13-admission.out` (second run, 91 lines, last line `emitted=91`; the replay entry points to it; pre-run registration `e150-r2-prereg.md`). The second run adds only four `name=admission_term` / `name=admission_answer` lines; everything else is identical to the first run's archived `e150-rollback-reuse-of-abandoned-roots-2026-09-13.out` (87 lines). Mutation table `scripts/mutation/tables/e150_rollback_reuse_of_abandoned_roots.tsv`: all 13 caught.

**Second run (discriminative power self-test for C318 (Units isolated by shadow accounting not in admission inequality), after the user settlement on the ninth term of D28 (Mount-Time Commitments) settled item 1)**: The three acceptance terms are calculated independently: allocated = the current root's accounting; deferred pending release = units referenced by valid roots in the rollback candidate set but not by the current root; abandoned-root-only = units referenced only by abandoned roots. The allocator independently counts “slots not in forbidden under shadow accounting.” Rows copied verbatim from the artifact:

```text
E7RESULT name=admission_term arm=shadow_accounts follow_up=0 allocated=4 deferred=5 abandoned_only=2 usable_with_term=21 deliverable=21 usable_without_term=23 gap=2 equal_with_term=true
E7RESULT name=admission_term arm=shadow_accounts follow_up=5 allocated=4 deferred=10 abandoned_only=2 usable_with_term=16 deliverable=16 usable_without_term=18 gap=2 equal_with_term=true
E7RESULT name=admission_term arm=shadow_accounts follow_up=11 allocated=4 deferred=11 abandoned_only=0 usable_with_term=17 deliverable=17 usable_without_term=17 gap=0 equal_with_term=true
E7RESULT name=admission_answer usable_with_term_equals_deliverable_in_every_row=true gap_equals_quarantine_before_drain=true gap_zero_at_drain=true criterion=E150_A1_A2
```

⇒ Usable slots with the ninth term equal the number the allocator can deliver in all three rows; without it, the count overstates availability by exactly the two quarantined blocks, returning to zero at the 11th publish.

### This is a deterministic model, not a measurement

There is no I/O or random source ⇒ N runs are byte-identical. Evidence comes from **absolute values pinned in 16 unit tests and all 13 mutations caught** (mutation 13 drops the ninth acceptance term, so usable slots no longer equal the number the allocator can deliver): baseline does not protect the candidate set; shadow accounting degenerates to baseline; no tombstone is written; root selection chooses the oldest; root selection ignores the instance table; the violation check always passes; the new root's txg is T_old; the root ring does not rotate across regions; the closed form is one short; FUA is not a segment boundary; faults are not injected; the new table after rollback ignores the allocator. Every mutation makes a test fail.

### Model

One disk; root ring of 3 regions × 4 slots (capacity 12 roots); unit area of 32 slots. Each root's accounting = units it references + the instance-table unit it points to. Publish writes in the order in D16 (Publish Semantics) settled item 7. Recovery selects “the newest readable root that passes self-verification + the instance table it points to,” and checks each referenced unit for the version that root wrote.

### Arms

| Arm | Allocator | Additional rollback writes |
|---|---|---|
| Baseline | Looks only at current root accounting | None |
| F-A shadow accounting | Takes only slots free in the accounting of every readable root in the ring | None |
| F-B tombstones | Same as baseline | Before rollback publish, overwrite every root slot of the abandoned timeline with a tombstone using FUA |

### Criteria (fixed before the run on 2026-09-13, not to be changed afterward)

Hand-calculated anchors, conservation, positive control (baseline must have violations in both cases), zero violations for both fixes plus discriminative power self-test, costs (how long shadow accounting quarantines, how many extra tombstone writes), and the question to answer. The full text is in the pre-run registration.

### Verdict (under the pre-run criteria; rows copied verbatim from the artifact)

```text
E7RESULT name=config devices=1 ring_regions=3 ring_slots_per_region=4 unit_slots=32 dirty_per_publish=1 normal_publishes=5 rollback_target_txg=3 warm_up_empty_publishes=3 follow_up_max=12 fua_is_boundary=true model=counting file_ops=0
E7RESULT name=anchor arm=baseline table_slot=3 states=3 violations=1 chosen_after_units_only=Some(2)
E7RESULT name=anchor arm=shadow_accounts table_slot=4 states=3 violations=0 chosen_after_units_only=Some(2)
E7RESULT name=anchor arm=tombstones table_slot=3 states=4 violations=0 chosen_after_units_only=Some(1)
E7RESULT name=case1 arm=baseline states=5 closed_form=5 violations=3 tombstone_writes=0 first_violation=segment=0 subset=0b1 chosen=5 overwritten=[6]
E7RESULT name=case1 arm=shadow_accounts states=5 closed_form=5 violations=0 tombstone_writes=0 first_violation=none
E7RESULT name=case1 arm=tombstones states=7 closed_form=7 violations=0 tombstone_writes=2 first_violation=none
E7RESULT name=case2 arm=baseline warm_up=false follow_up=0 faults=1 chosen=Some(5) chosen_is_abandoned=true violation=1 quarantined_total=5 quarantined_by_abandoned=0
E7RESULT name=case2 arm=baseline warm_up=true follow_up=0 faults=4 chosen=Some(5) chosen_is_abandoned=true violation=1 quarantined_total=5 quarantined_by_abandoned=0
E7RESULT name=verdict baseline_case1=3 baseline_case2=19 shadow_case1=0 shadow_case2=0 tombstones_case1=0 tombstones_case2=0 shadow_clean=true tombstones_clean=true shadow_quarantine_zero_at_follow_up=11
```

| # | Criterion | Result |
|---|---|---|
| 1 | Hand-calculated anchor (corrected) | Baseline writes the instance-table unit to slot 3; recovery chooses R2 when only units persist, with a violation. Shadow accounting writes slot 4, zero violations. Tombstones use slot 3, recovery chooses R1, zero violations. All three match hand calculation |
| 2 | Conservation | Crash-state count = closed form (5 / 5 / 7); deterministic recovery; every root references previously written units; allocated and free sets in accounting do not intersect; the forward timeline does not reallocate candidate-set units |
| 3 | Positive control | Baseline case one has violations in 3 / 5 states (first: once the instance-table unit persists, recovery mounts the txg 5 root whose unit 6 has been overwritten); case two has 19 violations in 26 combinations (the remaining 7 have P ≥ 11, only rollback-instance roots remain in the ring, all are unreadable and no root can be selected) |
| 4 | Both fixes | Shadow accounting and tombstones have zero violations in both cases. Mutations M2 (shadow accounting degenerates to baseline) and M3 (no tombstone written) make the discriminative power self-test; both make tests fail |
| 5 | Cost | Shadow accounting: units quarantined solely because of abandoned roots are 2 at P = 0..8, 3 at P = 9..10 and 0 from P = 11 (newest abandoned root occupies slot 9 and is overwritten at txg 17); total quarantined with the rollback candidate set is 11–13 at steady state, already required by I-7.4 (Blocks in Recent K Generations Not Reused). Tombstones: 2 FUA writes per rollback (= abandoned roots in the ring); when tombstones persist but the new root does not, recovery chooses R_old |
| 7 | Warm-up and fault count | Without warm-up the rollback instance has only one root and one fault falls back to an abandoned root; after 3 empty warm-up publishes it takes 4 faults (on one disk, counted by root slots) |
| 6 | Question to answer | Both fixes block both cases. Shadow accounting costs space and time until abandoned roots leave the ring; tombstones cost writes and the “no persistent effect before” statement. See C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused) for which statements of D23 (Journal Role & Format) settled item 14 and I-7.4 (Blocks in Recent K Generations Not Reused) would change |

### What it cannot answer

One disk; journal records are only ordering points; the defer window and F_effective are outside the model; only reuse is checked, not header erasure.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D28 (Mount-Time Commitments) settled item 1 | Supports | 2026-09-19 unaffected: usable slots with the ninth term equal allocator-deliverable slots in all three rows; omitting it overstates by 2 quarantined blocks. 2026-10-01 unaffected: owed-check short names in the text were renamed (C143 (inode number watermark falls back and reissues after crash recovery lands on an old root), C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused), C498 (no clause specifies warm-up counts for subsequent mounts or remount after crash), C542 (Rehearsal Before Number Acquisition Does Not Check Disks; Bad Disk Plus Old Roots All Overwritten Burns Numbers)), removing the wording of rollback at mount; conclusion, numbers and artifact unchanged; 2026-10-22 unaffected: only the title status now says the rollback shape it measured no longer exists and its conclusions are history only; numbers and artifacts unchanged |
| D16 (Publish Semantics) settled item 8 | Supports | 2026-09-19 unaffected: without warm-up one fault falls to an abandoned root; after 3 empty warm-up publishes it takes 4. 2026-10-01 unaffected: owed-check short names in the text were renamed (C143 (inode number watermark falls back and reissues after crash recovery lands on an old root), C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused), C498 (no clause specifies warm-up counts for subsequent mounts or remount after crash), C542 (Rehearsal Before Number Acquisition Does Not Check Disks; Bad Disk Plus Old Roots All Overwritten Burns Numbers)), removing the wording of rollback at mount; conclusion, numbers and artifact unchanged; 2026-10-22 unaffected: only the title status now says the rollback shape it measured no longer exists and its conclusions are history only; numbers and artifacts unchanged |
| D16 (Publish Semantics) settled item 7 | No impact | 2026-09-19 unaffected: only its publish order was modeled. 2026-10-01 unaffected: owed check short names in the text were renamed (C143 (inode number watermark falls back and reissues after crash recovery lands on an old root), C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused), C498 (no clause specifies warm-up counts for subsequent mounts or remount after crash), C542 (Rehearsal Before Number Acquisition Does Not Check Disks; Bad Disk Plus Old Roots All Overwritten Burns Numbers)), removing the wording of rollback at mount; conclusion, numbers and artifact unchanged; 2026-10-22 unaffected: only the title status now says the rollback shape it measured no longer exists and its conclusions are history only; numbers and artifacts unchanged |
| D23 (Journal Role & Format) settled item 14 | Supports | 2026-09-21 unaffected: E150 (Rollback Reuse of Units Referenced by Abandoned Roots) measures whether units referenced by the abandoned timeline can be reused after rollback (shadow accounting), not the discriminator for failure handling. The read-only review added by fix B neither writes to disk nor touches allocator accounting, so reachability and zero-violation conclusions for both cases remain unchanged. 2026-10-01 unaffected: owed-check short names in the text were renamed (C143 (inode number watermark falls back and reissues after crash recovery lands on an old root), C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused), C498 (no clause specifies warm-up counts for subsequent mounts or remount after crash), C542 (Rehearsal Before Number Acquisition Does Not Check Disks; Bad Disk Plus Old Roots All Overwritten Burns Numbers)), removing the wording of rollback at mount; conclusion, numbers and artifact unchanged; 2026-10-22 unaffected: only the title status now says the rollback shape it measured no longer exists and its conclusions are history only; numbers and artifacts unchanged |

## Historical Versions

History entries for E150 (Rollback Reuse of Units Referenced by Abandoned Roots) are collected in [experiments-history.md](../experiments-history.md).
