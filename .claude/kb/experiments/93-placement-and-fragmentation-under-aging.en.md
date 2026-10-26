## E93 Placement and Fragmentation Under Aging — Run (2026-09-03, 14 unit tests / 11 mutations all caught): Fragmentation is O(1) incrementally maintainable; clustered segments are not self-sustaining without compaction feeding empty segments; neighbor rewriting has a ceiling, compaction continues down with budget

[中文](93-老化下的放置与碎片度.md) · **English** · [日本語](93-老化下の配置とフラグメンテーション度.ja.md)

**Problem**: D26 (Background Compaction and Placement Reclamation) settled item 1 lacks two prerequisites — the quantity of "how much degradation occurs without compaction", and a computable definition of fragmentation (`.claude/rules/fs-design.md` hard requirement 1: criteria must be explicit, computable, and written down); settled item 4 needs to judge "whether structural anti-aging needs to touch node layout"; settled item 6 (user settlement: persist it) needs the observable quantity itself. E10 (Aging: Structural Resistance vs Post-hoc Compaction) owes a file system and cannot run; E93 (Placement and Fragmentation Under Aging) measures the placement policy half first at the counting model tier.

⚠️ **E93 (Placement and Fragmentation Under Aging) does not replace E10 (Aging: Structural Resistance vs Post-hoc Compaction)**: the criterion in D26 (Background Compaction and Placement Reclamation) main conclusion that "one of the two legs can be degraded" belongs to the one hardcoded by E10 (Aging: Structural Resistance vs Post-hoc Compaction) itself (③ reaching the level of ②); E93 (Placement and Fragmentation Under Aging) does not touch that verdict.

### Model

L fixed-width objects queued in key order (can be read as 16 KiB index leaves of D8 (Core Index Structure) settled item 2, or as 32 KiB data units of D4 (Checksum Location) settled item 7 — both types are fixed width, model does not distinguish; reading as which type only affects interpretation of "more runs => whose scan is slower"), device with S slots, fill ratio L/S. Initially written sequentially in key order (sequential creation form). Each checkpoint dirties a batch of objects per workload, COW rewritten to new slots; old slots enter defer, available for reuse only in next checkpoint (modeling D16 (Publish Semantics) settled item 15).

**Candidate definition of fragmentation (verified object)**: `F = runs / L`, runs = count of physically continuous segments when traversing all objects in key order. Required to be **incrementally maintainable**: when object moves from slot a to slot b, only adjacency relationships with two key-adjacent objects change => O(1) update; audit path full-scan recalculation does not share code with incremental path (division of labor between box 1 + box 2 of `.claude/rules/fs-design.md`).

Four arms (placement policies) × two workload sets (uniform random single-point rewrite / segment rewrite, segment length 8):

| Arm | Policy |
|---|---|
| first_fit | Lowest empty slot first (control: classic aging profile) |
| bump_seg | Dirty objects in same checkpoint sorted by key and written contiguously into clustered segment; falls back to first_fit and counts when no empty segment. D3 (Space Allocation) settled item 5 only settled commit-endogenous block clustering; generalizing it to all COW writes is candidate policy, not settled item |
| bump_neighbor(R) | bump_seg + write-path neighbor rewrite: each dirty key segment rewrites ≤ R clean neighbors on each side — candidate form of "structural anti-aging" without format changes |
| bump_compact(B) | bump_seg + round-robin compaction of B objects in key order budgeted per checkpoint — modeling leg 1 of D26 (Background Compaction and Placement Reclamation) |

### Criteria (Hardcoded Before Run 2026-09-03, Unchanged After Run)

1. **Manual calculation anchor**: 8-object small scenario, scripted rewrite runs hardcoded per manual calculation (absolute value assertion).
2. **Conservation**: On every sample point, every arm, every seed, incremental runs == full-scan recalculation runs; occupied slot count constantly == L; empty slots + occupied == S; deferred slots not reusable in current checkpoint.
3. **Positive control of measurement, all four arms pass gate**: Via test-specific forced placement switch (bypassing policy) placing objects into fully isolated layout, measurement pipeline of every arm must report runs == L; placing back into key-order continuous layout must report runs == 1.
4. **Whether aging workload is severe enough**: first_fit × uniform random workload runs to final term runs ≥ L/4 — isomorphic to invalidation clause of E10 (Aging: Structural Resistance vs Post-hoc Compaction): failing to measure degradation only indicates workload was not severe enough.
5. **Three questions to answer (only reporting numbers, no degradation verdict)**:
   a. runs trajectory of first_fit and bump_seg — model-tier answer to "how much degradation without compaction";
   b. whether bump_seg empty segment supply exhausts in steady state (fallback count trajectory) — whether clustered segment policy self-sustains without compaction;
   c. on cells with similar extra write budgets, between bump_neighbor and bump_compact whose runs is lower and ratio magnitude.
6. Deterministic model with seeded RNG: 5 seeds demonstrate absence of hidden state and seed sensitivity, not statistical stability (`.claude/singlefs-ai-sop/rules/test-discipline.md` deterministic experiment clause).

### Failure Clauses (Hardcoded Before Run) and Verdicts

| Clause | Verdict |
|---|---|
| Positive control (criterion 3) misses on any arm | **Passed**: All seven arm configurations scatter_runs=8192, compact_runs=1 |
| Criterion 4 misses | **Passed**: first_fit × uniform random across five seeds final term runs 8182–8188 (≥ L/4 = 2048) |
| bump_neighbor not better than first_fit | Not triggered (better than first_fit, but see ceiling in measurement item 3) |
| bump_compact not better than bump_seg | Not triggered |

### Measurement Results (L=8192, S=10240 (80% fill), segment length G=64, dirty per checkpoint D=64, T=2000 rounds, median of 5 seeds)

#### 1. How much degradation without compaction: Both workloads rush to saturation quickly

| Arm | Uniform random runs (% of L) | Segment rewrite runs (% of L) | Extra writes |
|---|---|---|---|
| first_fit | **8186 (99.9%)** | 7997 (97.6%) | 0 |
| bump_seg | **8186 (99.9%)** | 7312 (89.3%) | 0 |

Trajectory: first_fit × uniform random reaches **8184 at t=500 (average ~3.9 rewrites per object)**, plateaus thereafter.
=> Model-tier "no compaction" is not slow degradation, but **surges to peak within hundreds of checkpoints**.

#### 2. Clustered segment policy does not self-sustain without compaction feeding empty segments (criterion 5b)

bump_seg fallback rate (proportion of allocations unable to enter clustered segment) across all five seeds is **91.8%–93.8%**: scattered releases almost never aggregate into an entirely empty segment. Compaction budget is the sole force suppressing it — fallback rates of bump_cp_b32 / b128 / b256 (uniform random) are ~88% / 73% / 50% respectively.
=> Policy of **"clustered segment allocation" rides on compaction**; commit-endogenous block clustering expected by E81 (Fixed Point of Commits) likewise requires someone clearing empty segments in steady state.

#### 3. Neighbor rewriting has a ceiling; compaction continues down with budget (criterion 5c)

| Arm | Uniform random runs (extra write multiple) | Segment rewrite runs (extra write multiple) |
|---|---|---|
| bump_nb_r1 | 5497 (2.98×) | 5181 (1.25×) |
| bump_nb_r2 | **5460 (4.92×)** | 5111 (1.50×) |
| bump_cp_b32 | 7502 (1.50×) | 6073 (1.50×) |
| bump_cp_b128 | 6095 (3.00×) | 3922 (3.00×) |
| bump_cp_b256 | 4791 (5.00×) | **1339 (5.00×)** |

- **Equal budget cells trade wins**: Uniform random 3× tier neighbor wins (5497 vs 6095), 5× tier compaction wins (4791 vs 5460); segment rewrite 1.5× tier neighbor wins (5111 vs 6073).
- **Neighbor rewriting saturates**: Raising R from 1 to 2, uniform random only moves from 5497 to 5460 (while budget rises from 3× to 4.9×) — speed of subsequent rewrites scattering clusters catches up with clustering speed, steady state stuck at F ≈ 66%. Moreover its budget is capped by workload geometry: under segment workload R=2 can only spend 1.5×, **nowhere to spend even if wanting more**.
- **Compaction suppresses segment workload to F ≈ 11.5–21.1%** (five seeds `f_pct` individually: 16.35 / 20.39 / 21.12 / 11.52 / 13.04, corresponding to `runs_final` 1339 / 1670 / 1730 / 944 / 1068, median 1339).
  ⚠️ **Trajectory is upward not downward**: copying the entire line in artifact:
  `E7RESULT name=traj arm=bump_cp_b256 load=runs8 seed=0 runs=0:1,500:914,1000:1147,1500:1202,2000:1339`
  — monotonically rises from 914 to 1339. **The sentence "compaction does not saturate, trajectory still declining over time" was withdrawn on 2026-09-06**: it contradicts the same line of the same artifact, and reversing direction causes downstream to underestimate problems (stopping condition of D26 (Background Compaction and Placement Reclamation) settled item 1 rests precisely on this sentence).

#### 4. Fragmentation definition holds (criterion 2)

Incrementally O(1) maintained runs matches full-scan audit point-by-point on **all arms × workloads × seeds × sample points**; discriminative power of audit verified by unit test (directly corrupting layout bypassing incremental path, two paths must diverge).
=> `F = runs / L` satisfies `.claude/rules/fs-design.md` box 1 (incrementally maintained at commit, has answer the moment queried) and independent audit path (box 2), serving as candidate fragmentation definition for D26 (Background Compaction and Placement Reclamation) settled item 1 and observable for settled item 6.

### Model Revision Records (All occurred prior to formal round admission, criteria untouched)

1. **Neighbor arm allocation path fixed once**: Initial cluster fell back to first_fit slot-by-slot after segment exhausted, writing "clusters" that were physically discontinuous — setting up a strawman for neighbor rewrite (explicitly forbidden by `.claude/singlefs-ai-sop/rules/evidence-discipline.md`). After revision, neighbor arm and compaction batch place across whole segments via "open segment → new empty segment → contiguous run of empty slots → slot-by-slot fallback (counted)". Before revision compaction swept equal-budget cells; after revision traded wins — **this change altered conclusion direction, hence recorded here**.
2. **first_fit added reservation guard**: Slots not yet bumped in open segment are reserved area for both first_fit and run search; stealing causes double allocation (bidirectionally verified by mutations M7/M11). Admission round output byte-for-byte identical before and after guard => window was not hit under these parameters, guard is defensive rather than altering numbers.

### What It Cannot Answer

1. Counting model: no real I/O, no CPU, no cache. "Fewer runs => faster scan" is extrapolation from structural quantity to latency, latency half belongs to E10 (Aging: Structural Resistance vs Post-hoc Compaction).
2. Objects fixed width: variable-length extent fragmentation profile not modeled.
3. Two workload sets are endpoints, not distribution: where real aging workload (git checkout class) lands lacks evidence. Settled message buffer of D11 (Message Buffers in Index Nodes) batches leaf rewrites => real dirty set is closer to segment tier than uniform random, but batching statistics themselves not modeled.
4. What trigger threshold to take does not belong to E93 (Placement and Fragmentation Under Aging) — it only delivers fragmentation definition, incremental maintainability evidence, and degradation trajectories.
5. Anti-aging form on node layout side (format modification route) has no arm — ceiling of neighbor rewrite arm does not constitute evidence that "format modification is also useless".
6. No striping, no crash / redo, no metadata vs data split, fill ratio takes only 80% single point (pointed out by the backward leg of the three-way argument, 2026-09-03): partial stripe pinning of D2 (RAID Stripe Policy) settled item 11, intent redo of D8 (Core Index Structure) settled item 5, whether commit-endogenous blocks of D3 (Space Allocation) settled item 5 can enter clustered segments, are all outside model. Fallback rate is measurement basis of all allocations, **cannot** be treated as cluster success rate of commit-endogenous blocks.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e93_aging_placement.rs` (`cargo run --release --bin e93-aging-placement`), raw output `e93-aging-placement-2026-09-03.out` (107 lines, ending `emitted=107`).
- **N=5 rounds byte-for-byte consistent** (same md5). **14 unit tests, 11 mutations hit entry by entry, 0 blind spots**, mutation table `scripts/mutation/tables/e93_aging_placement.tsv` (`bash scripts/mutation/mutate.sh e93-aging-placement scripts/experiment-apparatus/e93_aging_placement.rs scripts/mutation/tables/e93_aging_placement.tsv`).
- Hooked into `scripts/experiment-apparatus/replay.sh` (exact tier).

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 4 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 1 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 3 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 6 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 11 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 12 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 8 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 14 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D10 (True Answer to Random Writes / Fragmentation) settled item 1 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 5 | Provisioning | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 2 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) settled item 7 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D2 (RAID Stripe Policy) settled item 11 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E93 (Placement and Fragmentation Under Aging) are consolidated in [experiments-history.md](../experiments-history.md).
