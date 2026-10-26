## E135 Dynamic Rollback Floor —— Run (2026-09-11, criteria fixed before run, 5 pre-run registrations added, no changes after run; 12 unit tests / 8 mutations all caught, N=5 rounds md5 consistent): **Zero fake candidates for whole ring and dynamic rollback floor across all failure worlds, three controls each caught in their target worlds; on near-full disk, rollback floor reduces forced publishes to ~1/11, at the cost of average rollback depth dropping to only 8 generations, and full stall during fill stage when empty publish overhead is 5 blocks**

[中文](135-动态回退下界.md) · **English** · [日本語](135-動的ロールバック下限.ja.md)

**This supplements arm 5 of D16 (Publish Semantics) settled item 1**: in that round, the user asked "can capacity be shrunk dynamically",
and the main agent drafted the form of "rollback floor F", which at that time had not undergone three-way argument nor entered the model.
Pre-run registration `e135-preregistration.md` (including 5 "pre-run additions" discovered while writing the harness and recorded before the first run).
⚠️ **The author of the registration biased toward arm D holding**, criterion 6 was specifically written against this bias, and it triggered once.

### Arms (Definitions Fixed Pre-Run)

"Fake candidate" is judged per the verbatim words of D22 (Synthesizing Unit Atomicity): a root in the candidate set has any block referenced by it already reused.

| Arm | Reallocatable Predicate | Candidate Set | Action on Tight Disk (Acceptance Failure) |
|---|---|---|---|
| A Whole ring (oldest root by persisted ring) | Released ∧ release generation ≤ txg of oldest persisted root in ring | All valid roots in ring per instance table | Push empty: perform empty publishes continuously until releases in window are returned, report ENOSPC only if still insufficient |
| B Whole ring (oldest root by arithmetic, control) | Same as A, but oldest root = current − N + 1 | Same as A | Same as A |
| C K takes lower bound 3 | Released ∧ release generation ≤ current − 3 + 1 | All valid roots in ring per instance table (today's literal text of D23 (Journal Role & Format) settled item 14) | Do nothing else |
| D Dynamic rollback floor | Released ∧ release generation ≤ max(F active value, oldest persisted root in ring) | Valid roots with txg ≥ F recovery value; F recovery value = max F across all self-witnessing legal roots in ring (including roots on abandoned timelines) | Raise F in one step to current − K_min + 1, subsequent roots record it; monotonic non-decreasing; becomes active K_min persisted publishes after the first root carrying it is persisted |
| E No wait for activation (control) | Same as D, but F becomes active upon persistence | Same as D | Same as D |
| F Only trust chosen root's F (control) | Same as D | Same as D, but F recovery value only takes chosen root's own | Same as D |

K_min = max(crash recovery 2, two-disk drop disk 0 worst rollback 2 + 1) = 3 (verbatim two-disk geometry of C222 (Lower bound 2 of I-7.4 conflicts with 3 required by version 1 two-disk geometry)).

### Worlds

| World | What Happens |
|---|---|
| `steady` | Only perform delete/create and publish |
| `torn_newest` | Root slot write torn during a publish, followed immediately by crash; recovery selects newest, rolling back one generation |
| `disk0_loss` | Disk 0 (regions 0, 2) lost completely after a publish; recovery selects newest, rolling back at most two generations |
| `slot_write_fail` | Root slot write fails during a publish, old root remains in slot; txg advances one slot before re-publishing |
| `admin_rollback` | Administrator rollback to the oldest root in candidate set, followed by publish |
| `near_full` | 70% warm-up for 2N + 10 windows, normal write path creates without deleting to reach 97%, then repeated delete/create |
| `rollback_first_publish` | Construction of C281 (First publish after rollback reuses blocks still referenced by abandoned timeline), reported separately, not used to judge arms |

In the four failure worlds, the three F arms each experience one forced tight disk condition before failure; failure falls on the 1st or K_min + 1-th non-empty window after tight disk (odd/even seeds half each);
in rollback world, rollback falls on the K_min + 3-th non-empty window after tight disk (pre-run addition item 1).

Measurement basis: R = 3, S ∈ {1, 4, 16}, N = 3S; intra-region slot and disk assignment per expected layout in [layout/01-first-txn.md](../layout/01-first-txn.md);
capacity 4000 blocks; each non-empty publish releases and allocates 13 blocks (user objects 8 + metadata 5, measurement basis of E123 (Trade-off Costs Between K and Rollback Depth));
reserve pool 13 blocks, visible only to publish; empty publish overhead c_empty ∈ {1, 5}; 24 seeds, 400 non-empty publishes per world.

### Verdicts

**Criterion 1 holds (cross-harness gate)**—both readings match cell-by-cell with the formula in E123 (Trade-off Costs Between K and Rollback Depth) (artifact `e135-rollback-floor-2026-09-11.out`, copied verbatim):

```text
E7RESULT name=crit1_pinned arm=A s=1 ring_depth=3 pinned_before=39 expect_before=39 pinned_after=26 expect_after=26
E7RESULT name=crit1_pinned arm=A s=4 ring_depth=12 pinned_before=156 expect_before=156 pinned_after=143 expect_after=143
E7RESULT name=crit1_pinned arm=A s=16 ring_depth=48 pinned_before=624 expect_before=624 pinned_after=611 expect_after=611
```

⇒ The 624 blocks in E123 (Trade-off Costs Between K and Rollback Depth) is the reading at the moment "before publish, when releases of this window are still in flight"; right after publish completes it is 611.

**Criterion 2 holds** (cross-harness gate, against N − K in E92 (Worst-Case Block Reuse Delay Requirements)), same artifact:

```text
E7RESULT name=crit2_fake arm=C s=1 fake_before=0 expect_before=0 fake_after=0 expect_after=0
E7RESULT name=crit2_fake arm=C s=4 fake_before=9 expect_before=9 fake_after=8 expect_after=8
E7RESULT name=crit2_fake arm=C s=16 fake_before=45 expect_before=45 fake_after=44 expect_after=44
```

⇒ The N − K in E92 (Worst-Case Block Reuse Delay Requirements) is the reading at the moment "before publish".

**Criterion 5 (reclaimed volume) holds**:

```text
E7RESULT name=crit5_reclaim arm=D s=1 publications_to_active=4 expect_publications=4 reclaimed=0 expect_reclaimed=0
E7RESULT name=crit5_reclaim arm=D s=4 publications_to_active=4 expect_publications=4 reclaimed=65 expect_reclaimed=65
E7RESULT name=crit5_reclaim arm=D s=16 publications_to_active=4 expect_publications=4 reclaimed=533 expect_reclaimed=533
```

⇒ Becomes active on the 4th publish after raising F; at the active moment, frees max(0, N − 7) × 13 additional blocks. When S = 1, N = K_min, F cannot exceed the oldest root, and this arm is equivalent to arm A at that tier.

**Summary line for criteria 3, 4, 6** (copied verbatim):

```text
E7RESULT name=verdict crit1_holds=1 crit2_holds=1 crit5_holds=1 crit3_b_slot_write_fail=1368 crit3_e_disk0_loss=12 crit3_f_admin_rollback=13521 crit4_a_fakes=0 crit4_d_fakes=0 crit6_stalls_a_cost1=0 crit6_stalls_d_cost1=0 crit6_stalls_a_cost5=0 crit6_stalls_d_cost5=24
```

- **Criterion 3 (discriminative power) caught in all three cells**: oldest root by arithmetic yielded 1368 fake candidates in slot write failure; no wait for activation yielded 12 in disk loss; only trusting chosen root's F yielded 13521 in administrator rollback.
  ⇒ The three mechanisms (oldest root by on-disk content, K_min publish activation delay, taking max F across whole ring on recovery) **produce fake candidates when removed in their respective target worlds**, none needs to be marked "untested".
  ⚠️ This only shows they bear load in that particular world, not that other forms cannot replace them.
- **Criterion 4 (correctness) is 0 for both arm A and arm D**: across 6 worlds (excluding C281 (First publish after rollback reuses blocks still referenced by abandoned timeline)) × 3 tiers of S × 24 seeds, zero fake candidates occurred.
- **Criterion 6 (disk full) triggered, arm D lost once in the disk full cell**: when empty publish overhead is 5 blocks and S = 16, all 24 seeds of arm D stalled during fill stage, while arm A had zero stalls.
  Mechanism: after F activates, the reallocatable bound stays at F; the next F raise must wait another K_min publishes to activate, and each of these empty publishes writes 5 blocks while releasing none
  (the oldest root in ring has not crossed the active F) ⇒ 4 × 5 = 20 blocks, exceeding the 13-block reserve pool. Arm A's push empty frees one generation per empty publish, avoiding this.
  When overhead is 1 block, 4 × 1 < 13, that tier does not stall.

**Costs on near-full disk** (extracted field-by-field from artifact lines `name=cell … world=near_full`, summed across 24 seeds; "window" refers to non-empty windows in counting stage, 8784 total for S = 4, 7056 for S = 16):

| Empty Publish Overhead | S | Arm | Forced Publishes | Forced Publishes per Window | Rollback Depth (Avg / Min) | Fake Candidates | Spurious ENOSPC | Stalls | Occupancy Post-Fill |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 4 | A | 5424 | 0.62 | 12.00 / 12 | 0 | 0 | 0 | 3885 |
| 1 | 4 | C | 0 | 0 | 12.00 / 12 | 70272 | 0 | 0 | 3885 |
| 1 | 4 | D | 7008 | 0.80 | 8.01 / 4 | 0 | 0 | 0 | 3885 |
| 1 | 16 | A | 62016 | 8.79 | 48.00 / 48 | 0 | 0 | 0 | 3885 |
| 1 | 16 | C | 0 | 0 | 48.00 / 48 | 310464 | 0 | 0 | 3885 |
| 1 | 16 | D | 5664 | 0.80 | 8.00 / 4 | 0 | 0 | 0 | 3885 |
| 5 | 4 | A | 12264 | 1.40 | 12.00 / 12 | 0 | 0 | 0 | 3885 |
| 5 | 4 | D | 8736 | 0.99 | 7.52 / 4 | 0 | 0 | 0 | 3885 |
| 5 | 16 | A | 331680 | 47.0 | 48.00 / 48 | 0 | **24** | 0 | **3749** (cannot fill to 97%) |
| 5 | 16 | D | — | — | — | — | — (did not enter counting stage) | **24** (all stalled during fill stage) | 3872 |

⇒ The three arms each pay for one thing on near-full disk: arm A pays in forced publishes (8.8 to 47 per window when S = 16, heavier as window widens),
arm D pays in rollback depth (average drops to ~8 generations, minimum 4), arm C pays in fake candidates (candidate set per run-time literal text is whole ring, while it only protects 3 generations).
Spurious ENOSPC (measurement basis: `df` = unused + released, pinned blocks and reserve pool both count as free) is non-zero in only one cell: when empty publish overhead is 5 blocks and S = 16, arm A (and control arm B by arithmetic oldest root) each had 24 occurrences—push empty reduced it to sporadic, not to zero; arm D did not enter the counting stage in that cell, reading does not count as 0. All other cells are 0, real ENOSPC is entirely 0.

**Criterion 7 holds across all arms** (construction of C281 (First publish after rollback reuses blocks still referenced by abandoned timeline), copied verbatim):

```text
E7RESULT name=crit7_c281 a=72 b=69 c=72 d=72 e=72 f=72
```

⇒ In a crash before the root slot lands on disk during that rollback publish, across 72 total instances over three S tiers, nearly all are unrecoverable (arm B had 3 lucky cases at S = 16).
The "present across all arms" in C281 (First publish after rollback reuses blocks still referenced by abandoned timeline) is confirmed here, independent of defer window size.

### Harness Changes Prior to Official Artifact (Faithfully Recorded)

Artifact `e135-rollback-floor-2026-09-11.out` was produced by the final version of the harness; outputs of earlier versions were overwritten and not retained.
Every item in the table was modified prior to the official artifact, without altering a single word of criteria or failure clauses:

| # | Discovery | How Discovered | What Changed |
|---|---|---|---|
| 1 | Fault placement was one window later than preregistration text; the extra window caused F to be carried by three consecutive roots, control arm E never caught in disk loss | Discriminative power unit test went red | Aligned "1st window after tight disk" to the tight disk window itself per preregistration text |
| 2 | In the C281 (First publish after rollback reuses blocks still referenced by abandoned timeline) world, rollback replaced allocator with the chosen root's copy; damage from reuse was tracked per that copy, unable to see content written later by abandoned timeline | Unit test for that world went red | Recorded on-disk state prior to rollback, subsequent reuses track damage against it |
| 3 | Control arm F unit test only sampled S = 4, where margin for F elevation was only 1-2 generations | Unit test went red | Added S = 16 per category 3 of `.claude/rules/mutation-sampling.md` |
| 4 | Fill stage on near-full disk only drew from "unused blocks", failing to reach 97% under S = 16, measuring a different world than registered | Arm A forced publish count scaled inversely with S (fewer on S = 16 than S = 4) | Changed to normal write path creating without deleting, wrote actual reached occupancy into artifact |
| 5 | Stalls during fill stage were wiped by counting reset, falsely reading as "0 stalls" in that cell | Arm D had 0 checks in one cell while reporting 0 stalls | Propagated fill stage stalls and "seed count halted during fill stage" into counting |
| 6 | Fix in item 2 only took effect in C281 (First publish after rollback reuses blocks still referenced by abandoned timeline) world; errors like "rollback row not written" in rollback world were invisible to harness | Mutation M8 (rollback does not write rollback row) failed to turn any test red in round 1 | Generalized item 2 to every administrator rollback, added unit test "arm A zero fake candidates after administrator rollback"; artifact remained byte-for-byte identical (md5 `bd9c8039bdb8fb08e3a00628b6121ab5` before and after) |

### Questions It Cannot Answer

1. **Wall clock**: all accounted by publishes. The conversion table "heavy load ~17 ms, idle ~4 min" does not come from here.
2. **Win/loss in criterion 6 cell depends on an unsettled parameter**: whether reserve pool 13 blocks or (K_min + 1) × empty publish overhead is larger. Empty publish overhead 1 block vs 5 blocks is an assumption;
   whether reserve pool should be sized by (K_min + 1) × empty publish overhead, and whether F should be raised before reserve pool is exhausted, are not measured here.
3. **Real devices, journal replay window, instance switchover** are not modeled; c2 in E92 (Worst-Case Block Reuse Delay Requirements) (in-flight record ceiling) has no value in this repo, not fabricated here.
4. Slot formulas and disk ownership use assumptions; with a different formula, K_min and three target worlds may require recomputation.
5. Allocator selects smallest reallocatable block by index; under a different selection, numbers in near-full disk table change, but the zero fake candidate cells should not (predicate is selection-independent), untested.
6. "How deep a rollback suffices" is a product judgment; the experiment only provides costs.
7. **No fault injected during interval between raising F and activation**: forced tight disk proceeded directly to F activation, faults all fell after activation.
   D16 (Publish Semantics) settled item 1 round 2 backward leg placed root slot write failure into that interval followed by losing disk 0; dynamic rollback floor produced fake candidates (verdict in D16 (Publish Semantics) settled item 1).
   Arm A's zero is near-constructive in this model—when candidate set and boundary derive from same roots, fake candidates are impossible; informative cells are only where they decouple (abandoned timeline, write-failed slots, dropped disks).
8. **Rollback depth counts roots, not distinct user states**: empty publishes from push empty and F elevation each count as one root. Converted by non-empty windows (mean approximation),
   S = 16 and empty publish overhead 1 block gives ~4.9 distinct states for arm A, ~4.4 for arm D; at overhead 5 blocks arm A gives ~1.0 (pointed out by D16 (Publish Semantics) settled item 1 round 2 backward leg).

### Measurement Basis and Re-running

- Code `scripts/experiment-apparatus/e135_rollback_floor.rs` (`cd scripts/experiment-apparatus && cargo run --release --bin e135_rollback_floor`),
  original output `e135-rollback-floor-2026-09-11.out` (157 lines, last line `name=done emitted=157`), ~9s per run.
- Mutation table `scripts/mutation/tables/e135_rollback_floor.tsv`, 8 mutations **all caught**
  (`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e135_rollback_floor e135_rollback_floor.rs ../mutations/e135_rollback_floor.tsv`).
- Deterministic model: consistent md5 across 5 runs only proves absence of hidden state, not statistical stability.
- Re-run:

```bash
bash scripts/experiment-apparatus/replay.sh E135
```

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D16 (Publish Semantics) | Material preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 14 | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D22 (Synthesizing Unit Atomicity) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-11
- Established and run. Criteria and failure clauses fixed pre-run; 5 items registered during harness development (all before first run); harness modified in 6 places prior to official artifact, see "Harness Changes Prior to Official Artifact".
