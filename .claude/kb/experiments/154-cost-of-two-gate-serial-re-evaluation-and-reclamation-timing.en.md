## E154 Cost of Two-Gate Serial Re-evaluation and Reclamation Timing —— Partially Run (2026-09-17, deterministic model, 30 unit tests / 14 mutations caught, H1 + H5-k-bit (6 variants) + H6 + H8 (6 variants, injection point moved to fill phase) + H4a/H4b (crash during empty publish) five history types, 24 arms, 48 geometry cells)

[中文](154-两道闸串-重判与回收时点的代价.md) · **English** · [日本語](154-2重ゲート直列再判定と回収タイミングのコスト.ja.md)

<!-- doc-lint:not-numbers F1 F1b F4 F5 G7 H1 H2 H3a H3b H4 H4a H4b H5 H6 H8 I1 PC1 PC2 PC4 PC5 Q1 Q2a Q2b Q3 Q4 Q5b Q6 Q7 R8 S1 S2 -->

Preregistration `e154-preregistration.md`. It asks: under the "two gates" defined by D3 (Space Allocation) settled item 9 and settled item 12, D16 (Publish Semantics) settled item 1, D28 (Mount-Time Commitments) settled items 1–4, and D23 (Journal Role & Format) settled item 14 across four readings: serial (literal) / serial-rejudge / merged / merged-full-deduction; three reclamation timings: G7 / F-deduct / F-double-deduct; and two push-empty stop policies: P-stop / P-full (4×3×2=24 arms), how much cost must be paid to rewrite the same size after deletion, whether `df` will lie, and whether it will wedge.

**Prepare material**: Provide cost figures to user-facing Fork 3 (measurement basis of two Fs) and Fork 4 (two gates) for alloc-basis round 3 verdict (`research/prompts/alloc-basis-r3-main-verification.md` section 6), waiting for citations when D3 (Space Allocation) settled item 12 and D16 (Publish Semantics) settled item 1 settle per forks; main agent stopped at stage 4, unmeasured parts see "This is not the full scale required by preregistration".

⚠️ **Main agent dispatched read-only audit and found three model bugs; numbers outside H1 cannot be used** (`alloc-basis-r3-forks-verify-e154.md`): (1) Row written by instance switch for old instance has `selected_root_txg` constantly 0 (harness lines 1540–1547), whereas D23 (Journal Role & Format) settled item 14 specifies writing the last published txg of that instance, causing old instance roots to be judged abandoned and reclaimed prematurely; hits on "issuing candidate root still referencing slot" under H4a/H4b, H5-1, and H8-1 are entirely caused by this, fixing this single place reruns hits to zero while H1 is cell-by-cell identical; it also shifts within-bound, out-of-bound, and wedged counts for these families; (2) Recursive switch dropped one row after write-row publish failure, coincidentally making k >= 2 families miss; (3) Rollback does not check whether target is still in the ring (implementation reports `RollbackTargetNotInRing`), affecting 60 cells in H6. Distributions of H5, H8, H6, H4a/H4b and "H8 eliminating degeneration" reported by E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing) are built on these bugs; they count only after fixing harness and rerunning.

⚠️ **This is not the full scale required by preregistration**. Section 5.4 of preregistration defines 12 histories (H1, H2, H3a, H3b, H4a, H4b, H5-k-bit six variants, H6). Stage 1 run only completed **H1**; stage 2 added **H5-k-bit all 6 variants** and **H6**; stage 3 clarified the cause of "degeneration" and determined it unfixable; stage 4 (same day, specified by main agent) added **H4a/H4b** (crash during empty publish and reopen) and **H8** (main-agent-specified H5 variant: identical k*bit, injection point moved from "rewrite after delete" acceptance to self-pushed empty publish in fill phase to raise F); H2 is still covered by n=1 sample point in 48-cell geometry. **H3a/H3b (concurrent requests) are still unimplemented**, preregistration criteria F1b (concurrent consumption) and per-attempt ROW details are still missing; ⚠️ **H4a/H4b degenerate into the identical code path in this model** (`attempt_publish` completes allocation + write root + persist atomically, not exposing "allocated but root not yet persisted" intermediate state, see "What was done in this stage" item 9), which is an equivalence dictated by model granularity, not lack of implementation. **Stop clause S1 was not performed at stage start, checked only after artifact finished running**: hashes of `mount.rs`/`transaction.rs` changed relative to stage 2 baseline (another session was editing `crates/`), handed to main agent per dispatch note "stop and report on mismatch"; whether six harness-implementation correspondences still hold was not re-verified in this stage, see "What it cannot answer" item 8.

### Results Copied Line-for-Line from Artifact

Real baseline (serial-rejudge * G7) broken down by stop policy, H1's 48 geometry cells * 2 stop policies = 96 runs:

```text
      4 stop=止 delete_then_rewrite_outcome_label=越界
      44 stop=止 delete_then_rewrite_outcome_label=达标
      24 stop=满 delete_then_rewrite_outcome_label=卡死
      24 stop=满 delete_then_rewrite_outcome_label=越界
```

(Command: `grep "^E7RESULT RUN" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage2.out | grep "baseline=true" | grep "history=H1" | awk '{for(i=1;i<=NF;i++){if($i~/^stop=/)s=$i; if($i~/^delete_then_rewrite_outcome_label=/)o=$i}; print s, o}' | sort | uniq -c`; these four lines are byte-for-byte identical with stage 1 artifact, `diff` has no output — H1 code path was not touched by the three fixes in this stage, see "What was done in this stage")

**Under P-stop, real baseline is "within-bound" on 44/48 cells** (K <= 3, boundary in D3 (Space Allocation) settled item 9 item 2), 4 cells "out-of-bound"; **under P-full, real baseline never reached "within-bound" even once** — 24 cells "out-of-bound", 24 cells "wedged" (Q5b, fixed-point allocation failure). Wedges appear only under P-full. This relates to preregistration section 10 failure clauses F1/F4, but **judging whether it truly overturns or establishes anything does not belong to this run** (that is inference, requiring three-way verification); here we report numbers only.

Overall distribution of H1 Q1 across all 24 arms * 48 cells (1152 runs):

```text
    180 delete_then_rewrite_outcome_label=卡死
     48 delete_then_rewrite_outcome_label=未成
    553 delete_then_rewrite_outcome_label=越界
    371 delete_then_rewrite_outcome_label=达标
```

Q6 (issuing candidate root still referencing slot, criterion for F5) is constantly 0 across all 1152 uncorrupted runs of H1: `grep "^E7RESULT RUN" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage2.out | grep "history=H1" | grep -v "unsafe_candidate_reuse_count_field=0 " | wc -l` -> `0`.

Distribution of non-zero counts for Q2a (`df` >= s but write s failed, D3 (Space Allocation) settled item 9 item 1) across all 1152 runs of H1:

```text
   1029 df_reported_enough_but_write_failed_count=0
     60 df_reported_enough_but_write_failed_count=1
     15 df_reported_enough_but_write_failed_count=2
     15 df_reported_enough_but_write_failed_count=3
      6 df_reported_enough_but_write_failed_count=4
      6 df_reported_enough_but_write_failed_count=5
     21 df_reported_enough_but_write_failed_count=6
```

Non-zero buckets sum to 60+15+15+6+6+21 = **123** runs (accounting for 10.7% of 1152) triggering Q2a.

**H5-k-bit (Q9 not applicable — H5 still asks "rewrite after delete", Q9 is exclusive to H6; reported here is Q1 for H5 itself)**: 24 arms * 48 cells * 6 variants (k, bit) = 6912 runs, Q1 distribution:

```text
   1690 卡死
   2046 退化
    512 越界
   2664 达标
```

(Command: `grep "^E7RESULT RUN" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage2.out | grep "history=H5-" | sed -n 's/.*delete_then_rewrite_outcome_label=\([^ ]*\).*/\1/p' | sort | uniq -c`) "Degenerated" = this acceptance did not reach the point of needing empty push at all (preregistration H4a/H5 degeneration clause); across 6 (k, bit) combinations, degenerated (arm, geometry) sets are byte-for-byte identical, all 341/1152 — degeneration depends only on gate1 state after 3 nudges, not on k or bit.

Real baseline (serial-rejudge * G7, both stop policies merged) across six combinations of H5:

```text
    134 卡死
    111 越界
    121 达标
    210 退化
```

(576 = 6 combinations * 96; command: `grep "^E7RESULT RUN" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage2.out | grep "history=H5-" | grep "baseline=true" | sed -n 's/.*delete_then_rewrite_outcome_label=\([^ ]*\).*/\1/p' | sort | uniq -c`)

**H6 (Q1 here is Q9 in preregistration section 6: how many non-empty publishes needed to rewrite after rollback)**: 24 arms * 48 cells = 1152 runs:

```text
    442 卡死
     90 未成
     43 越界
     69 达标
    508 退化
```

Real baseline on H6: 24 wedged, 5 unachieved, 3 out-of-bound, 33 within-bound, 31 degenerated (96 runs). For the proposition "H6: rewrite after isolating a batch of slots following rollback" itself — across 1152 runs **416 times** isolated slot count > 0 (`grep "^E7RESULT H6DETAIL" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage2.out | grep -v "isolated_slots_after_rollback=0$" | wc -l` -> `416`), maximum isolated slots reached 211, remaining 736 runs had 0 isolated slots or rollback never reached that step (most degenerated/unachieved/wedged tiers isolated 0).

**Q3 (B=∞ counterfactual, H1 all 24 arms * 48 cells)**: Compared against B=8 baseline, `unbounded_budget_diverged_count=125` (artifact final line) — across 1152 runs, 125 times Q1 outcome differed between the two budgets (different Q1 label, or different K, or different wedged state). Real baseline (serial-rejudge * G7) accounts for 24 of these 125 times (`grep "^E7RESULT Q3" … | grep "gate=串重判-G7" | grep "diverged=true" | wc -l` -> `24`), all falling under `stop=满`; serial-rejudge * G7 * P-stop never diverged even once — B=8 never became a binding constraint under the `stop=止` branch.

All 12 positive controls have teeth (artifact final line, 4 corruptions * 3 history types H1 / H5(root_write_failure) / H6(rollback)):

```text
E7RESULT DONE runs=10656 pc1_hit_any=true pc2_hit_any=true pc4_hit_any=true pc5_hit_any=true pc1_root_write_failure_hit_any=true pc2_root_write_failure_hit_any=true pc4_root_write_failure_hit_any=true pc5_root_write_failure_hit_any=true pc1_rollback_hit_any=true pc2_rollback_hit_any=true pc4_rollback_hit_any=true pc5_rollback_hit_any=true unbounded_budget_diverged_count=125
```

**H8 (stage 4, 24 arms * 48 cells * 3k * 2bit = 6912 runs) Q1 distribution**:

```text
   1164 卡死
    287 越界
   3733 达标
   1728 退化
```

(Command: `grep "^E7RESULT RUN" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out | grep "history=H8" | grep -o "delete_then_rewrite_outcome_label=[^ ]*" | sort | uniq -c`)

**Real baseline (serial-rejudge * G7, stop/full both stop policies 288 cells each) degenerated cells on H8: 0/288 (stop), 0/288 (full)** — identical arm and combination on H5 (original) was **210/288 (stop)**, 0/288 (full) (consistent with 73% in stage 3 diagnosis); H8 eliminates H5's only non-structural widespread degeneration (serial-rejudge-G7-stop 210/288) as well as all 6 "merged / merged-full-deduct" * stop arms degeneration (18/288 each, 108 rows total) — remaining 1728 rows of degeneration on H8 come **entirely and exclusively** from 6 arms of the "serial" family (6*288=1728), line-by-line check matching H5 at 288/288 (definition of "serial" dictates it never reaches self-pushed empty publish at any injection point, unchanged by shifting injection point, see preregistration section 12 stage 3 category 1 diagnosis).

**H4a/H4b (each 24 arms * 48 cells = 1152 runs) Q1 distribution, byte-for-byte identical between the two**:

```text
    116 卡死
    185 达标
    851 退化
```

(Command: `grep "^E7RESULT RUN" e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out | grep "history=H4a" | grep -o "delete_then_rewrite_outcome_label=[^ ]*" | sort | uniq -c`; `history=H4b` shares method and distribution. Erasing `history` tag from both and performing byte-for-byte `diff` yields 0 lines different). 116 "wedged" cells concentrate in near-full disk geometries (unlisted per cell); 851 "degenerated" cells = this arm in this acceptance never reaches "self-pushed empty publish" (first gate does not re-evaluate, or passes gate directly without re-evaluation), sharing form with H1/H5 degeneration clauses.

⚠️ **Artifact self-inspection revealed a pre-existing issue not introduced by 2026-09-17 stage 4 run** (details in preregistration section 12 stage 4 item 10): real baseline (serial-rejudge * G7) has rows satisfying `unsafe_candidate_reuse_count_field > 0` across H5 (stage 2 artifact, untouched), H6 (stage 2 artifact, untouched), and H4a/H4b (stage 4) — per preregistration section 10 F5, this judges "reclamation timing unsafe: issued slots still referenced by candidate roots => that arm (including real baseline) disqualified on correctness". Counts: H5 real baseline 84/6912 rows (H5 all 24 arms 1328/6912 rows), H6 real baseline 15/1152 rows (H6 all 24 arms 270/1152 rows), H4a/H4b real baseline 39/1152 rows each; H1 all 24 arms 0/1152 rows (experiment page reported only this prior to stage 4). **This criterion was never mentioned by any version of reports before stage 4**; whether "real baseline disqualified" holds requires three-way review; here we report numbers only without verdict.

### Reproduction Commands

```bash
bash scripts/experiment-apparatus/replay.sh E154
# Run harness directly:
cd scripts/experiment-apparatus && cargo run --release --bin e154-two-gates-serial-rejudge-and-reclaim-timing
```

Code `scripts/experiment-apparatus/e154_two_gates_serial_rejudge_and_reclaim_timing.rs`, raw output `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage4.out` (37346 lines, last line `emitted=37346`; ran ~5–6 minutes, two independent runs byte-for-byte identical — local host smoke run and official run differed only in field names of `DONE` line, caused by stage 4 renaming H5 variant variable family, field values byte-for-byte consistent). Stage 2 artifact `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage2.out` (18722 lines) and stage 1 artifact `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17.out` (1250 lines) have H1/H5/H6 sections matching stage 4 artifact corresponding fields byte-for-byte (`diff` has no output); both and stage 4 `-stage4.out` are archived in git history (retrieved via `git show 3cff909^:research/results/<filename>`). The E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing) row in `replay.sh` currently points to `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out` (rerun after subscript renaming 2026-09-24, differing from `-stage4.out` only in renamed identifiers). Preregistration `e154-preregistration.md` (including four-stage revisions, see section 12), mutation table `scripts/mutation/tables/e154_two_gates_serial_rejudge_and_reclaim_timing.tsv` (14 mutations all caught, `bash scripts/mutation/mutate.sh e154-two-gates-serial-rejudge-and-reclaim-timing scripts/experiment-apparatus/e154_two_gates_serial_rejudge_and_reclaim_timing.rs scripts/mutation/tables/e154_two_gates_serial_rejudge_and_reclaim_timing.tsv`).

### This is a Counting Model, Not an Implementation; Differences Between Measurement Basis and Preregistration

Pure arithmetic, no I/O, no concurrency, no random sources; running identical input N times is necessarily identical (rerun of stage 4 artifact is byte-for-byte identical, `diff` has no output); strength of evidence comes from 30 unit tests and 14 caught mutations, not from round counts (`.claude/singlefs-ai-sop/rules/test-discipline.md` premise that "single observation does not count" applies to observations with variation; this harness has none).

**What was done in this stage** (items 1–6 are stage 2 records, detailed in preregistration section 12 revisions items 3–6; items 7–11 are stage 4 additions, detailed in preregistration section 12 stage 4):

1. Checked baseline hash of `crates/` specified by main agent (addressing stop clause S1), re-read section 3 "correspondence between harness and implementation" items 1–6 line by line, did not trigger stop.
2. Implemented H5-k-bit (root slot write failure proceeds with instance switch per R8): `FailureInjector` injects continuous k failures starting from the n-th root slot write within this acceptance under "first / overwrite" two bit positions; `push_one_empty_publish` / `perform_instance_switch` share identical failure closure (roots of switch row-write publish and warm-up publish are also queried, corresponding to preregistration "roots of switch row-write publish count towards continuous failures"); during implementation found and fixed bug where `attempt_publish` root slot write failure branch did not advance `pool.next_txg` (this branch was never reached by H1 before, zero coverage).
3. Implemented H6 (administrator rollback): `perform_rollback` loads slot state snapshot from `R_old` (newest persisted valid root before publish in which Y was committed) (`RootRecord` adds two snapshot fields, `fixpoint_slots_snapshot` / `instance_table_slots_snapshot`, avoiding guessing current fixed-point/instance-table slot numbers based on assumption "allocator always issues from lowest empty slot"); shadow accounting isolates slots "referenced only by abandoned roots, unreferenced by candidate roots, and absent from R_old's own snapshot" per narrow reading, writes rollback row, rollback publish is non-empty, followed by warm-up and attempting write of Y2. During implementation found rollback publish itself could genuinely fail allocation (near-full disk), replaced original `.expect()` with normal wedged handling.
4. Supplemented Q3 (B=∞ counterfactual): H1 all 24 arms * 48 cells compared against B=8 baseline (real rerun, not approximation), budget computed per preregistration 5.5 formula `10 * 3 * S` (guaranteeing termination).
5. Ran positive controls on H5/H6 as well (instructed by main agent): found during execution that two "degeneration" branches of H5 originally did not distinguish "genuine wedge" from "this acceptance did not take the intended path" — PC5 (`zero_reserves`) on k=1 / bit=first geometry degenerated on all 24 arms rather than wedging; re-probed with k=3 / bit=overwrite, most arms (except `serial` family and `serial-rejudge-G7-stop`) wedged genuinely on identical cell, showing original degeneration branch missed verdicts; after fix (`degenerate_or_wedged_summary` queries `pool.allocation_failure_hit` first), all 12 positive controls had teeth.
6. Output format retained repository's existing `E7RESULT` convention (`Emitter`).
7. Implemented H8: added `LazyFailureInjector`, sharing identical "continuous k times" semantics with `FailureInjector`, differing in that `window_start` is computed dynamically using current `last_txg` only when actually queried for the first time (first self-pushed empty publish in this run) — identical injector can fall on the episode of first self-pushed empty publish in fill phase to raise F, or fall back naturally to H5 injection point when fill phase does not self-push. To allow closure to read `last_txg` before querying `failures`, changed failure closure signatures of `push_one_empty_publish` / `perform_instance_switch` / `admit_request` / `admit_request_inner` / `retry_loop` / `fill_pool` from `FnMut() -> bool` to `FnMut(u64) -> bool` (pure signature refactoring; verified 23 existing unit tests remained green before adding new functions); added `failures` parameter to `fill_pool`, H1/H5/H6 pass `&mut |_last_txg| false`, behaving byte-for-byte identically to pre-parameter state.
8. Implemented H4a/H4b: added `crash_and_reopen` (selects maximum txg root unjudged by `is_abandoned`, loads its slot state snapshot, clears withheld/shadow accounting, resets switchover reservation to 4 units, reuses existing `perform_instance_switch` for "new instance, write row, warm-up") and `run_crash_mid_push_history` (shares H3a prefix to 3 nudges, allows exactly 1 self-pushed empty publish to persist, then crashes and reopens, continuing retry loop). ⚠️ H4a and H4b share the identical code path in this model (`attempt_publish` completes "release -> allocate -> write root -> persist" atomically, not exposing "allocated but root not yet persisted" intermediate state) — verified against artifact: RUN lines stripped of `history` tag show 0 differing lines on byte-for-byte `diff`, unit test `h4a_and_h4b_crash_points_are_observationally_equivalent_in_this_model` nails this equivalence as an assertion.
9. Ran positive controls on H4/H8 as well (instructed by main agent), 4 corruptions * 2 new history types = 8 items, all with teeth (8 fields including `pc1_crash_mid_push_hit_any` on `DONE` line all `true`).
10. Artifact homogeneity self-check revealed a pre-existing issue (not introduced by stage 4, verified present in stage 2 artifact): real baseline has rows triggering F5 (`unsafe_candidate_reuse_count_field > 0`) across H5/H6/H4a/H4b, unmentioned in reports and experiment page prior to stage 4 — see second to last paragraph of "Results Copied Line-for-Line from Artifact" and preregistration section 12 stage 4 item 10.
11. ⚠️ **Stop clause S1 was not performed at stage 4 start**: proceeded directly from stage 2/3 artifacts; checked hashes of `mount.rs`/`transaction.rs` only after artifact finished running — both changed relative to stage 2 baseline (another session was editing `crates/`, `transaction.rs` had 767 added / 367 deleted lines relative to HEAD). Per literal S1 this triggers stop; S2 (re-verifying harness-implementation correspondence 1–6) was not performed in stage 4. Stage 4's H8/H4a/H4b are built on top of the six correspondences verified in stage 2 without further changes, not re-deriving any item from the 2026-09-17 version of `mount.rs`/`transaction.rs`; whether these six still hold on that version was not verified in stage 4, handled per dispatch instruction "stop and report on mismatch" for main agent to decide whether to dispatch another round of verification.

**H1 code path was untouched by the three fixes in stage 2** (fail=true branch, degeneration branch, rollback publish; H1 never injects failures or rolls back): H1 portion of artifact is byte-for-byte identical with stage 1 (verified in "Results Copied Line-for-Line from Artifact"), H1 figures verified in stage 1 report and preregistration need no re-verification; H5/H6 portions of artifact are byte-for-byte identical with stage 2 (except `DONE` line field name differences from renaming in this stage).

**Implementation trade-offs beyond preregistration** (technical, not altering criteria in sections 6, 7, 9, listed item by item, including those carried over from stage 1 and added in this stage):

1. **ROW (per-attempt) details are still unimplemented**, emitting only `RUN` (one row per run), `PC` (one row per positive control), `H5DETAIL` / `H6DETAIL` (one row each for supplementary diagnostic quantities). Rationale identical to stage 1: full run ROW detail artifact volume is too massive.
2. **Q2b (acceptance passed but allocation failed) is constantly 0**: this model executes single-threaded sequentially; gate2 judgment and actual allocation use identical state at the identical moment, with no divergence.
3. **Q4 (reclamation deferred by how many publishes) uses coarse measurement basis, not `D_acct` / `D_alloc` two components required by preregistration**: `H5DETAIL` row reports `recovery_publishes_until_covered` — total root slot write attempts (including failed ones) from start of this acceptance until both disks cover new F; this is a merged figure; `D_acct` (step recognized first by accounting) and `D_alloc` (step allocator can actually issue) require per-attempt (ROW-level) breakdown to separate, which this file lacks.
4. **H3a/H3b (concurrent requests) are still unimplemented**, F1b and Q7 judgments under concurrency remain vacant; **H4a/H4b (crash during empty push) are implemented, but are the identical code path in this model** (see "What was done in this stage" item 8), not two independently verified paths — separating them requires splitting `attempt_publish` into "allocate" and "write root persist" two separately crashable steps, which this stage did not do.
5. **H6 object table snapshot is simplified, not the full preregistration form**: `pool.object_slots` (object id -> slot number) is not realigned per `R_old`, tracking only "which padding objects currently remain undeleted"; objects potentially deleted after rollback are already popped from queue along with `object_slots` and will not be referenced again, so this simplification does not affect Q9/Q6/Q7 computation, but also does not simulate "objects reappearing after rollback" (preregistration question itself did not ask this).
6. Output format retained repository's existing `E7RESULT` convention (`Emitter`), rationale identical to stage 1.

### Arms and Geometry

24 arms = 4 gate readings (serial, serial-rejudge, merged, merged-full-deduct) * 3 reclamation timings (G7, F-deduct, F-double-deduct) * 2 push-empty stop policies (P-stop, P-full), defined word-for-word identical to preregistration section 5.3. 48 geometry cells = capacity (600, 4000) * object unit count (1, 16) * c_max (4, 9) * slots per zone (4, 8) * padding (0, 1, 2).

- **H1**: Fill (write object X, then write 1-unit padding objects one by one until ENOSPC) -> padding (delete pad padding objects) -> delete X -> retry loop (write Y; if unsuccessful, nudge once and delete one padding object, at most 3S+5 times).
- **H5-k-bit**: Same fill / padding / delete X as H1, followed by **3 consecutive nudges without attempts in between** (differing from H1's alternating "attempt then nudge"); in this acceptance (attempt write Y), root slot write fails continuously k times; bit=first starts from 1st root slot write in this acceptance, bit=overwrite starts from last root slot write needed to cover two disks (identical txg mod 3 formula as I1 anchor in section 4); failure proceeds with instance switch per R8; normal retry loop follows.
- **H6**: H1 runs until Y succeeds (unsuccessful records "degenerated") -> nudge 2 more times -> administrator rollback to R_old (newest persisted valid root before publish in which Y was committed, required to be in candidate set, records "rollback unreachable" degeneration if absent) -> rollback and warm-up -> attempt write Y2, entering retry loop.
- **H8** (stage 4, main-agent-specified H5 variant): Identical k*bit definition as H5-k-bit, differing only in injection point — not injected into empty push of "rewrite after delete" acceptance, but into **empty publish self-pushed in fill phase to raise F**; `LazyFailureInjector` computes `window_start` dynamically only when actually self-pushing empty publish for first time in this run; when fill phase does not self-push throughout, identical injector falls back naturally to H5 injection point.
- **H4a/H4b** (stage 4): Same prefix as H3a to 3 nudges; attempt write Y; in this acceptance allow exactly 1 self-pushed empty publish to persist, then crash and reopen (`crash_and_reopen`: selects maximum txg root unjudged abandoned, loads its slot state, acquires new instance, writes row, warms up — word-for-word identical to R8 "instance switch"); retry loop continues (nudge counter continues counting). ⚠️ H4a (crash after root persisted) and H4b (crash before root persisted) degenerate into the identical code path in this model, see "What was done in this stage" item 8.

### Positive Controls

PC1 (F carried by empty push constantly takes F_top, not raised), PC2 (`df` invariably reports U_all+Rel), PC4 (G7 family altered to reclaim as soon as first root carrying new F persists; F-deduct family altered to reclaim without withholding), PC5 (reserves all 0) — four corruptions run against H1 / H5 (k=1 bit=first, PC5 uses k=3 bit=overwrite) / H6 / H4a (`CrashPoint::AfterFirstPush`) / H8 (k=1 bit=first, PC5 uses k=3 bit=overwrite, identical combination to H5), all 20 items report `true` on artifact final line; PC5 against H1 and against H5 (k=3 bit=overwrite) each have unit tests pinning that at least one arm wedges (`positive_control_pc5_wedges_when_reserves_zero`, `positive_control_pc5_wedges_root_write_failure_when_reserves_zero`).

### What It Cannot Answer

1. **H3a/H3b (concurrent requests) are still unimplemented** — F1b, Q7 judgment under concurrent scenarios remain vacant. **H4a/H4b are implemented, but are the identical code path in this model** (see "What was done in this stage" item 8), not two independently verified paths.
2. **Q4 has only coarse measurement basis** (`recovery_publishes_until_covered`, total root slot write attempts in one acceptance), not `D_acct` / `D_alloc` two components required by preregistration — those two numbers require ROW-level detail to separate.
3. **H6 object table is not realigned per R_old** (see "Implementation trade-offs beyond preregistration" item 5); this simplification does not affect quantities reported in this stage, but does not imply covering scenarios beyond preregistration questions.
4. **The pattern "under P-full real baseline is entirely out-of-bound or wedged, wedges appear only in P-full"** (discovered in stage 1) was checked in this stage only on H1 and only within this simplified model, without verifying whether it is model bias; likewise, **whether specific distribution numbers reported by H5/H6 (wedged/degenerated/out-of-bound/within-bound counts) are meaningful criteria or artifacts of unsettled interpretations** was not investigated in this stage — drawing conclusions requires three-way review.
5. **Not contrasted against real system (`crates/`)**; when writing this stage on 2026-09-17, `crates/` had no implementations for two gates, `df`, instance switch, administrator rollback (confirmed by live check in preregistration section 3; administrator rollback subsequently landed as `roll_back_by_a_forward_publish`, remaining items unverified); six harness-implementation correspondence checks in this round covered only implemented raising F / F_effective / region attribution / write-row warm-up four items, re-verified in this stage (see preregistration section 12 item 3), still covering only these four.
6. **12 mutations in mutation table cover only places directly hit by unit tests in newly written code of this stage**: exclusion logic in `perform_rollback` for "do not isolate if referenced only by candidate root" and "do not isolate if already allocated in R_old snapshot" lack dedicated absolute-value mutations (covered only indirectly by `run_rollback_history_isolates_slots_after_rollback` checking "isolated count > 0"); fine-grained coverage is lower than "first/overwrite" window and wedged judgment items.
7. **Arms * cells unmeasured due to massive "degeneration" on H5 and H6, listed item by item (checked live in stage 3, commands and artifacts in report)** — "degeneration" means this acceptance did not take the intended path, not counted towards within-bound or out-of-bound, nor into any criteria; evaluated whether altering "how history is built" without changing criteria could produce shapes on these cells, conclusion was **cannot** (evidence in preregistration section 12 stage 3 item 7, reasons given per arm per cell; temporary diagnostics deleted after run, code, artifact, mutation table, `replay.sh` verified unchanged):
   - H5 (288 cells per arm): "serial" family 6 arms all 288/288 (100%, structural — "serial" by definition never pushes empty on gate1 failure; H5 requires failure injection during empty push, which never happens in this arm); "serial-rejudge-G7-stop" 210/288 (73%, "fill" phase itself pushes empty for this arm, raising F in advance, needing real empty push only on 13/48 geometry cells where n=16*S=4); "serial-rejudge-G7-full" and "serial-rejudge*F-deduct (4 arms)" 0/288.
   - H6 (48 cells per arm): "serial" family 6 arms 40/48 (83%, primary cause is Y's own retries rotate nearly a full turn of root ring, rollback target `R_old` kicked out before Y succeeds); "serial-rejudge*F-double-deduct-stop" 32/48, "serial-rejudge*F-deduct-stop" 31/48, "serial-rejudge-G7-full" 24/48; "merged / merged-full-deduct-stop" (6 arms) 13/48 each; "merged / merged-full-deduct-full" and "serial-rejudge*F-deduct-full" (8 arms) 12/48 each; "serial-rejudge-G7-stop" 7/48 (lowest).
8. **Stop clause S1 was not performed at stage 4 start** (see "What was done in this stage" item 11): hashes of `mount.rs`/`transaction.rs` changed relative to stage 2 baseline; S2 (whether six harness-implementation correspondences hold on 2026-09-17 version) was not re-verified in stage 4, handed to main agent to decide whether to dispatch another round.
9. **F5 (reclamation timing unsafe) has genuine hits on real baseline** (see second to last paragraph of "Results Copied Line-for-Line from Artifact"): real baseline on H5, H6, H4a, H4b has rows with `unsafe_candidate_reuse_count_field > 0`, unmentioned in reports and experiment page prior to stage 4 — whether "real baseline disqualified" holds requires three-way review to judge. **An unverified conjecture (not conclusion)**: `perform_instance_switch` (invoked by H5 root slot write failure path and H4 `crash_and_reopen`) writes `selected_root_txg: 0` for `InstanceRow` padded for "skipped instance" — this is correct for R8's original "extent of previous instance failure unknown, assume worst case", but for scenarios like H4 where "previous instance did not fail anything, process merely reopened", it marks roots already successfully persisted by that instance as "abandoned", making candidate set (input to F5 criterion) overly wide; this conjecture was not verified in stage 4 with code or artifact, merely an observation from reading code, requiring three-way review to judge whether correct and whether to change.

### Impacted Decisions

| Decision Item | Relation | Review |
|---|---|---|
| D3 (Space Allocation) settled item 12 | Prepare material | 2026-10-11 Unaffected: wording changes only (time words, positional references, self-references); conclusion, numbers, and artifact unchanged |
| D16 (Publish Semantics) | Prepare material | 2026-10-21 unaffected: only the trilingual language-switch line was added; previously, 2026-10-11 was unaffected: wording changes only (time words, positional references, self-references); conclusion, numbers, and artifact unchanged |
| D3 (Space Allocation) settled item 9 | No impact | 2026-10-11 Unaffected: wording changes only (time words, positional references, self-references); conclusion, numbers, and artifact unchanged |
| D28 (Mount-Time Commitments) | No impact | 2026-10-21 unaffected: only the trilingual language-switch line was added; previously, 2026-10-11 was unaffected: wording changes only (time words, positional references, self-references); conclusion, numbers, and artifact unchanged |
| D23 (Journal Role & Format) settled item 14 | No impact | 2026-10-11 Unaffected: wording changes only (time words, positional references, self-references); conclusion, numbers, and artifact unchanged |

## Historical Versions

### 2026-09-17 (Stage 1)
- Initial creation. Execution stage found two implementation bugs (unmarked allocation, failure propagation under PC5) and fixed them, adding corresponding unit tests; due to implementation duration, reduced preregistration 12 histories to H1 only; Q2b/Q3/Q4 simplified or left vacant, recorded item by item in "Implementation trade-offs beyond preregistration"; whether remaining histories need completing left to main agent.

### 2026-09-17 (Stage 2)
- Added H5-k-bit all 6 variants and H6; changed Q3 to real B=∞ counterfactual rerun (covering H1 all 24 arms * 48 cells); ran positive controls on H5/H6 as well; execution stage found and fixed three implementation issues (root slot write failure not advancing txg, H6 rollback publish potentially failing allocation, H5/H6 degeneration branches originally not distinguishing genuine wedge), recorded item by item in preregistration section 12 items 4–6; unit tests increased from 16 to 23, mutation table from 7 to 12 items, all caught; H3a/H3b/H4a/H4b and ROW details remain unimplemented, left for next stage. Artifact saved as `-stage2.out`, stage 1 artifact retained; E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing) row in `replay.sh` updated to new artifact.

### 2026-09-17 (Stage 3)
- Clarified reasons for "degeneration" concentrating on H5/H6 arms (four categories, listed item by item in "What it cannot answer" item 7 and preregistration section 12 stage 3 item 7); determined unfixable without altering criteria; investigation added temporary diagnostics only (`eprintln!` and one temporary unit test), completely deleted immediately after run and re-verified — 23 unit tests remained green, artifact byte-for-byte identical with stage 2, 12 mutations all caught (0 invalid, 0 non-red), `replay.sh` rerun of E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing) reported byte agreement, producing no new artifact, `replay.sh` line unchanged. H3a/H3b/H4a/H4b, ROW details, Q4 exact components remain unimplemented, left for next stage; status remains "Partially Run".

### 2026-09-17 (Stage 4)
- Added H8 (main-agent-specified H5 variant, injection point moved to fill phase self-pushed empty publish to raise F) and H4a/H4b (crash during empty publish and reopen); unit tests increased from 23 to 30, mutation table from 12 to 14 items, all caught; positive controls added 4 items each for H4/H8, all with teeth. **Headline result**: on H8, real baseline (serial-rejudge * G7) degeneration cells dropped to 0/288 (stop), 0/288 (full) — H5 (original) identical arm was 210/288 (stop); H4a and H4b produce byte-for-byte identical RUN lines in this model (equivalence dictated by atomicity granularity of this model, nailed by unit test). **Artifact self-inspection revealed a pre-existing issue** (not introduced in this stage): real baseline triggers F5 (issuing candidate root still referencing slot) across H5/H6/H4a/H4b, unmentioned in previous reports, requiring three-way review to judge correctness. **Stop clause S1 was not performed at stage start**: checked live only after artifact finished running; `mount.rs`/`transaction.rs` changed relative to stage 2 baseline (another session editing `crates/`), S2 not re-verified, handed to main agent per dispatch note. H3a/H3b, ROW details, Q4 exact components remain unimplemented; status remains "Partially Run".

### 2026-09-24 (Rerun After Subscript Renaming)
- Before change: harness `e154_two_gates_serial_rejudge_and_reclaim_timing.rs` and mutation table used prime marks or spellings for variant names (H5 prime, `H5PRIMEDETAIL`, `h5prime`, `merged-prime`, `F-deduct-prime`); row for this experiment in `scripts/experiment-apparatus/replay.sh` pointed to archived `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage4.out`.
- After change: renamed to H8, `H8DETAIL`, `h8`, `合-全扣`, `F-双扣` per repository-wide prohibition of subscripts (`.claude/rules/path-moves.md` "give variants new names, avoid subscripts"); rerun produced new artifact `research/results/e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out`, replay registration updated to point to it.
- Rationale: old artifact (`git show 3cff909^:research/results/e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage4.out`) replaced entirely across these five pairs of renames (hitting 6912, 96, 6912, 9336, 24736 places) matches new artifact byte-for-byte, numbers completely unchanged.
