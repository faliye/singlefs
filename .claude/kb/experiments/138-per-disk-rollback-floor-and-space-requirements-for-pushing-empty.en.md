## E138 Per-Disk Rollback Floor and Space Requirements for Pushing Empty — Run (2026-09-11)

[中文](138-按盘回退下界与推空的空间要求.md) · **English** · [日本語](138-ディスク別ロールバック下限と空押しの容量要件.ja.md)

**What is being asked**: Two matters following the verdict of Round 2 three-way argument on D16 (Publish Semantics) settled item 1 —
① Whether the dynamic rollback floor (Arm G) — effective per disk, taking cap per disk (user settlement: retaining at least 4 persistent roots, including the newest persistent valid root on each disk), with reservation pool counting window metadata —
holds after fault injection into the interval from raising F to activation;
② Under the identical `df` measurement basis (deducting respective reservation pools and worst-case pushing-empty residue) against whole ring + push empty (Arm A), whether the two items of D3 (Space Allocation) settled item 9 are met, what the respective bounds are, and how much each under-reports on `df`.

**Hardcoded before running**: `e138-preregistration.md` (written before apparatus; reservation pool formulas and Arm G's F-raising timing were modified while coding apparatus, both prior to the first run, see "Historical Versions" in `138-按盘回退下界与推空的空间要求.md`).
The registration author held two leanings (Arm G holds; Arm A might meet standard under the new `df` measurement basis), explicitly documented in registration. **The outcome was: the first leaning was wrong, the second was right.**

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E138
# Run apparatus directly:
cd scripts/experiment-apparatus && cargo run --release --bin e138_per_disk_floor
```

Code: `scripts/experiment-apparatus/e138_per_disk_floor.rs` (adapted from source of E135 (Dynamic Rollback Floor)),
raw output: `e138-per-disk-floor-2026-09-11.out` (single run, 317 lines, final line `emitted=317`; native release ~43 seconds).

### This Is a Counting Model, Not an Implementation

No I/O, random source is hardcoded seed ⇒ running N times is necessarily byte-for-byte identical (run twice, md5 matches, demonstrating solely the absence of hidden state).
Evidence strength derives from **15 unit tests + 9 mutations all caught** (`scripts/mutation/tables/e138_per_disk_floor.tsv`);
mutation M6 (activation taking per-disk maximum on recovery) was not red on any unit test in Round 1, classified per `.claude/rules/mutation-sampling.md` as sampling point insensitive, caught after supplementing unit test
`recovery_does_not_activate_floor_carried_on_only_one_disk`.
Two cross-apparatus gates (locked block count of E123 (Trade-off Costs Between K and Rollback Depth), N − K of E92 (Worst-Case Block Reuse Delay Requirements)), sharing identical values with E135 (Dynamic Rollback Floor).

### Arms

| Arm | Description | Role |
|---|---|---|
| A `ring_persisted` | Reallocatable ⟺ release generation ≤ oldest valid root persisted in ring; on acceptance failure, first publishes pending window, then up to N − 1 consecutive empty publishes | Candidate |
| C `k_runtime` | Release generation ≤ newest persistent root + 1 − 3 | Control: K = 3 is insufficient under skipped numbers |
| D `floor_txg` | Arm D of E135 (Dynamic Rollback Floor) verbatim: raising F and activation both calculated by txg | Control: activation calculated by txg |
| G `floor_per_disk` | Upper bound for raising F = min(newest persistent valid root on each surviving disk, 4th newest persistent valid root); new F activates only after written into at least one persistent root on each surviving disk; on acceptance failure, first publishes pending window (txg t), raises F to t at once when cap reaches t, up to 6 empty publishes | Under test |
| G_count | G, but activation modified to "3 more persistent publishes after root carrying new F" | Control |
| G_txgcap | G, but upper bound modified to newest persistent root + 1 − 4 | Control |

Reservation pool (pre-run formula): A = 2 × 5 + (N − 1) × empty publish overhead; G = 2 × 5 + 6 × empty publish overhead. Worst-case pushing-empty residue: A = 5 + (N − 2) × empty publish overhead; G = 5 + 5 × empty publish overhead.
`df_guaranteed` = unused + released − reservation pool − worst-case pushing-empty residue; `df_raw` deducts nothing (measurement basis of E135 (Dynamic Rollback Floor)). Criteria use only the former.

### Verdicts (Per Criteria Hardcoded Before Run)

Transcribed verbatim from `e138-per-disk-floor-2026-09-11.out`:

```text
E7RESULT name=verdict crit1_holds=1 crit2_d_activation_fakes=584 crit2_c_activation_unrecoverable=281 crit2_gcount_double_fakes=171 crit2_gtxgcap_retention=0 crit2_a_raw_false_enospc_s16_cost5=24 crit3_a_fakes=0 crit3_a_unrecoverable=0 crit3_g_fakes=0 crit3_g_unrecoverable=0 crit4_a_false_guaranteed=0 crit4_g_false_guaranteed=0 crit5_a_bound_exceeded_cells=0 crit5_g_bound_exceeded_cells=0 crit5_a_write_after_delete_failed=0 crit5_g_write_after_delete_failed=104 crit6_a_stalls=0 crit6_g_stalls=0 crit7_g_retention=0
E7RESULT name=crit6_min_no_stall_reserve a_cost1=51 a_cost5=239 g_cost1=14 g_cost5=35
E7RESULT name=crit8_c281 a=72 c=72 d=72 g=72 g_count=72 g_txg_cap=72
```

| # | Criterion | Result |
|---|---|---|
| 1 | Apparatus vs pre-run formula | All matched: across three placement positions of push-empty probe at t, A uses N publishes, withholding 5 + (N − 2) × overhead; G uses 6 or 7, withholding at most 5 + 5 × overhead; at S = 1 both use 3 publishes (artifact `crit1_push` rows all `holds=1`) |
| 2 | Whether control arms are caught | D 584, C 281, G_count 171, A's raw false ENOSPC 24, all caught. **G_txgcap is 0 ⇒ "taking cap per disk" half recorded as unverified**: both target worlds raise F first then fail writes, where newest root on disk 1 when raising F is ≥ newest − 2; this control demands "raising F after skipped numbers from write failure", which does not exist in E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)'s world |
| 3 | Correctness | A and G have zero fake candidates and zero unrecoverable states across all fault worlds (including 16 write failure combinations in activation interval × 8 crash windows × losing two disks once each) |
| 4 | Item 1: No false ENOSPC allowed | Both A and G are 0 (raw column non-zero, proving detector turns red) |
| 5 | Item 2: Usable within bounded steps after deletion | **Arm G eliminated**: deleting an object after full disk under empty publish overhead of 5 blocks causes subsequent write to still report ENOSPC, 24 / 24 seeds each for S = 4 and S = 16 (on failure, `df_guaranteed` also reports < 8, not false); 7 tiers also appear among tiers ≥ pre-run formula in reservation pool sweep. **Arm A all succeeded**. Neither arm exceeded respective bounds (A = N, G = 7) |
| 6 | Stalls | Both arms 0 when reservation pool ≥ pre-run formula; minimum non-stalling reservation pool in sweep: A 51 / 239, G 14 / 35 (overhead 1 / 5 blocks); those two numbers for Arm A happen to be the bottom edge of sweep (pre-run formula − 6), true requirement unmeasured; G's numbers lie inside sweep interval |
| 7 | Minimum retention | G is 0 |
| 8 | C281 (First publish after rollback reuses blocks still referenced by abandoned timeline) | 72 across all six arms, present in every arm (consistent with "present in every arm" row of C281 (First publish after rollback reuses blocks still referenced by abandoned timeline)) |

Tiers where Arm G failed during reservation pool sweep (transcribed verbatim):

```text
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=1 reserve=22 formula=16 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=38 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=39 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=40 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=41 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=42 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=46 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=47 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
E7RESULT name=reserve_sweep arm=G_floor_per_disk s=16 empty_cost=5 reserve=48 formula=40 stalls=0 false_enospc_guaranteed=0 write_after_delete_failed=8 max_publications=7 fakes=0
```

⚠️ **Failures are not monotonic with reservation pool**: at overhead 5 blocks, 38–42 and 46–48 fail, 43–45 do not; at overhead 1 block, only 22 fails.

**Why Arm G lost** (main agent reproduced and logged balances in out-of-tree replica, `research/prompts/d16-item1-r3-main-verification.md`; D16 (Publish Semantics) settled item 1 Round 3 forward leg first reported "handling up to release still loses"): blocks withheld after one handling depend on which region root carrying new F lands on, either 5 + 4 × overhead or 5 + 5 × overhead; acceptance admits based on currently withheld amount, while `df_guaranteed` deducts worst-case tier ⇒ upon fill, system halts in state where `df_guaranteed` signed value is −3 (overhead 5 blocks).
In this state, deleting 8 blocks leaves only 5 allocatable when next handling lands on larger tier, failing write; succeeding when landing on smaller tier — hence each seed fails only once, succeeding after one fewer object.
**Root cause lies in acceptance, not handling**: handling up to release (publish count still 6 or 7) fails seed by seed alike; after changing acceptance to charge by worst-case residue (allocatable takes min of "reallocatable − reservation pool" and `df_guaranteed`), near-full disk deletion-then-write failures across four cells in replica are all 0. That form did not enter fault worlds and lacked preregistration, hence constitutes no verdict.

### Costs (Transcribed Verbatim from `e138-per-disk-floor-2026-09-11.out`, `near_full` Cells, 24 Seeds Total)

```text
E7RESULT name=near_full arm=A_ring_persisted s=1 empty_cost=1 reserve=12 residue=6 hidden_blocks=18 hidden_bytes=294912 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=3 bound=3 admissions_under_pressure=9648 forced_per_window_x100=219 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=3 avg_candidates_x100=300 avg_nonempty_x100=162
E7RESULT name=near_full arm=G_floor_per_disk s=1 empty_cost=1 reserve=16 residue=10 hidden_blocks=26 hidden_bytes=425984 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=7 bound=7 admissions_under_pressure=9648 forced_per_window_x100=220 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=3 avg_candidates_x100=300 avg_nonempty_x100=161
E7RESULT name=near_full arm=A_ring_persisted s=1 empty_cost=5 reserve=20 residue=10 hidden_blocks=30 hidden_bytes=491520 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=3 bound=3 admissions_under_pressure=9624 forced_per_window_x100=220 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=3 avg_candidates_x100=300 avg_nonempty_x100=162
E7RESULT name=near_full arm=G_floor_per_disk s=1 empty_cost=5 reserve=40 residue=30 hidden_blocks=70 hidden_bytes=1146880 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=7 bound=7 admissions_under_pressure=9624 forced_per_window_x100=221 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=3 avg_candidates_x100=300 avg_nonempty_x100=161
E7RESULT name=near_full arm=A_ring_persisted s=4 empty_cost=1 reserve=21 residue=15 hidden_blocks=36 hidden_bytes=589824 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=12 bound=12 admissions_under_pressure=9768 forced_per_window_x100=886 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=12 avg_candidates_x100=1200 avg_nonempty_x100=211
E7RESULT name=near_full arm=G_floor_per_disk s=4 empty_cost=1 reserve=16 residue=10 hidden_blocks=26 hidden_bytes=425984 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=7 bound=7 admissions_under_pressure=9744 forced_per_window_x100=479 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=845 avg_nonempty_x100=264
E7RESULT name=near_full arm=A_ring_persisted s=4 empty_cost=5 reserve=65 residue=55 hidden_blocks=120 hidden_bytes=1966080 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=12 bound=12 admissions_under_pressure=9624 forced_per_window_x100=899 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=12 avg_candidates_x100=1200 avg_nonempty_x100=210
E7RESULT name=near_full arm=G_floor_per_disk s=4 empty_cost=5 reserve=40 residue=30 hidden_blocks=70 hidden_bytes=1146880 false_enospc_raw=48 false_enospc_guaranteed=0 true_enospc=48 write_after_delete_failed=24 max_publications=7 bound=7 admissions_under_pressure=9696 forced_per_window_x100=484 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=842 avg_nonempty_x100=261
E7RESULT name=near_full arm=A_ring_persisted s=16 empty_cost=1 reserve=57 residue=51 hidden_blocks=108 hidden_bytes=1769472 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=48 bound=48 admissions_under_pressure=10200 forced_per_window_x100=3622 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=48 avg_candidates_x100=4800 avg_nonempty_x100=230
E7RESULT name=near_full arm=G_floor_per_disk s=16 empty_cost=1 reserve=16 residue=10 hidden_blocks=26 hidden_bytes=425984 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=7 bound=7 admissions_under_pressure=9984 forced_per_window_x100=486 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=1039 avg_nonempty_x100=431
E7RESULT name=near_full arm=A_ring_persisted s=16 empty_cost=5 reserve=245 residue=235 hidden_blocks=480 hidden_bytes=7864320 false_enospc_raw=24 false_enospc_guaranteed=0 true_enospc=24 write_after_delete_failed=0 max_publications=48 bound=48 admissions_under_pressure=9624 forced_per_window_x100=3928 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=48 avg_candidates_x100=4800 avg_nonempty_x100=221
E7RESULT name=near_full arm=G_floor_per_disk s=16 empty_cost=5 reserve=40 residue=30 hidden_blocks=70 hidden_bytes=1146880 false_enospc_raw=48 false_enospc_guaranteed=0 true_enospc=48 write_after_delete_failed=24 max_publications=7 bound=7 admissions_under_pressure=9720 forced_per_window_x100=485 stalls=0 halted_before_counting=0 fakes=0 unrecoverable=0 retention_violations=0 min_candidates=5 avg_candidates_x100=981 avg_nonempty_x100=395
```

Reading method: `hidden_blocks` = reservation pool + worst-case pushing-empty residue, which is the block count under-reported on `df` compared to E135 (Dynamic Rollback Floor) measurement basis (`hidden_bytes` converted at 16 KiB per block);
`forced_per_window_x100` is forced publish count per user window × 100; `avg_nonempty_x100` is non-empty publish roots in candidate set × 100 (empty publishes and push-empty roots do not alter user-visible state).

- Arm A has a full ring of candidate roots on near-full disks (48 roots at S = 16), **yet average non-empty roots is only ~2.2** — ring is almost entirely empty roots generated by push-empty; Arm G has ~4.0 (minimum 5 roots).
  E135 (Dynamic Rollback Floor) "rollbackable generations" counted roots; E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) counts non-empty roots for the first time.
- Forced publishes of Arm A: at S = 16 ~36–39 times per user window, scaling linearly with S; Arm G ~4.9 times, independent of S.
- `df` under-reporting: Arm A at S = 16, overhead 5 blocks is 480 blocks (7.5 MiB at 16 KiB/block), scaling linearly with N; Arm G is 70 blocks, independent of S.

### What It Cannot Answer

1. Wall clock: all counted by publishes.
2. Real devices: counting model, does not touch devices.
3. Journal replay window: models root-level references only, not records.
4. Instance switching unmodeled; its mount-time reservation superimposed with reservation pool unmeasured.
5. Empty publish overhead takes only 1 and 5 block constants; real overhead scales with tree height, so reservation pool after mount may no longer suffice.
6. Capacity 4000 blocks: proportion of `df` under-reported blocks to capacity cannot transfer to real disks; reports only blocks and bytes.
7. Slot formulas and disk allocation use layout/01-first-txn.md vision; two disks only.
8. **World lacks the cell 'crash during activation interval without losing disks'**: `torn_newest` pushed F to activate on both disks before crashing. How recovery takes active values is pinned solely by unit test `recovery_does_not_activate_floor_carried_on_only_one_disk`.
9. **'Taking cap per disk' unmeasured** (Criterion 2 G_txgcap is 0): measuring it requires worlds with 'raising F after skipped numbers from write failure'.
10. **Near-full disk world injected no faults whatsoever**: 'false ENOSPC is 0' and 'bound N / 7 publishes' hold only on fault-free near-full disks. Per D16 (Publish Semantics) settled item 1 Round 3 backward leg and main agent reproduction (`d16-item1-r3-main-verification-2.md`): one root slot write failure during push-empty causes Arm A false ENOSPC and bound becomes 2N − 1; persistent root slot write failure causes Arm A to report ENOSPC and stall at 70% occupancy.
11. `df_guaranteed` did not deduct mount-time reservation of instance switching; empty publish overhead takes constants, real overhead scales with tree height, formula must deduct worst case, yet worst case lacks measurement basis.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D16 (Publish Semantics) | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D3 (Space Allocation) settled item 9 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-12
- Replaced "Why Arm G lost" with reproduced mechanism (original was deductive, right only about "placement dictates residue" half, suggested fix "do all to full" cannot fix); Criterion 6 cell added note that Arm A reservation pool count is scan bottom edge; "What It Cannot Answer" item 10 replaced with fault-free near-full disk scope and added item 11; "about 4.8 times" modified to artifact's 4.86 ≈ 4.9. Rationale: D16 (Publish Semantics) settled item 1 Round 3 three-way argument.

### 2026-09-11
- Created. Three changes made while coding apparatus (reservation pool adding previous window metadata, Arm G waiting for cap to reach t before raising, expected value for S = 1), all prior to first run;
  preregistration originally written to `e137-preregistration.md`, overwriting an uncommitted file with identical name from another session, restored verbatim from that session's logs, renumbered to E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty).
