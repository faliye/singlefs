## E166 Shared State Partitioning for Parallel Commits — Partially Run (segment one; second rerun 2026-10-02: tree-ID renumbering)

[中文](166-并行提交的共享状态分法.md) · **English** · [日本語](166-並列コミットの共有状態の分割方法.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q7 Q9 Q10 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15 V16 V17 V18 V19 Q-1 Q-10 Q-13 Q-4 Q-7 Q-8 Q-9 -->

Pre-registration `e166-preregistration.md` (section-twelve revisions recorded); Q4 in `m3-concurrency-questions.md`: how to divide shared state among in-flight transactions (whole exclusive, partitioned, optimistic). Subject is an independently handwritten counting model (`scripts/experiment-apparatus/e166_parallel_commit_shared_state_partition.rs`, no shared code with `crates/`, not a checked-in implementation apparatus). It enumerates every interleaving of two or three transactions and counts cases where two obtain the same placement or number (duplicate allocations) and optimistic-arm conflict recomputations. The implementation has no in-flight transactions (serialization via exclusive `&mut` borrow, not locks). Registration section three lists shared state based on current `crates/`; this page records segment one and the second rerun after tree-ID renumbering (R3 in `m3-treeid-rerun-questions.md`).

Artifacts:

- `e166-parallel-commit-shared-state-partition-2026-10-01.out`: first run, 1315 `E7RESULT` rows, SHA-256 `7b9a829ba5568e13d9c4f15941bdb0fb3fb8c20d27a3a0ec9465ba5d4ac1ffeb`. Five consecutive runs of the same binary have identical hashes (deterministic model: no hidden state, not statistical stability). The quoted result rows below are from this artifact.
- `research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out`: main artifact of the second rerun (apparatus after tree ID renumbering), 1315 rows of `E7RESULT`, sha256 `64467cf0efc4222f901747db3980b5eb601d9f9567932e592c11516f2fff6e29`; the E166 (partitioning shared state for parallel commit) registered line in `replay.sh` points to this copy. For remaining artifacts and readings, see the “Second rerun (tree ID renumbering)” section.

Current apparatus has 3374 lines and SHA-256 `e9b0c4ee05cf7eaa57f0e8a27b7c08dcdfa7177d3339f67e42e65846b12f8fb4` (first-run version 3371 lines, SHA-256 `fcc86011669a312315ac95be4355ad62e48b0c8d926f826c478d31b69ed4cc06`, tree-ID types `u8`). Mutation table `scripts/mutation/tables/e166_parallel_commit_shared_state_partition.tsv` now has 21 rows (first run 19), SHA-256 `c0376dea6ad705e088950f9208da302fcf84f09ea487cfe86178b078baf199cb`.

### What was done

Segment one (registration 5.5): item-by-item differential of the stop conditions, all positive controls and section-seven anchors, Q-1 through Q-9 for six arms (A whole exclusive, 「A字面」 (A, literal candidate wording), 「P区」 (partitioned by region), 「P设」 (partitioned by device), 「O整」 (optimistic, whole-version validation), 「O读」 (optimistic, read-set validation)) under default geometry (D = 2 disks, 4 root-ring slots per region, journal ring 196608 slots, regions = transactions, enough reserve), and first reverse geometry sample D = 1. Cell = arm × Q1 candidate (Q1甲 parallel preparation/serial persistence, Q1乙 multiple transactions per publish, Q1丙 multiple in-flight publishes) × shape (same/disjoint) × initial state (file/nofile) × transactions (2/3) × disks (2/1), 288 cells. Positive controls add 144 cells (six controls each disabling its own guard × three Q1 candidates × same shape × two initial states × n 2/3 × disks 2/1).

- Stop checks: before running, read each registered `crates/` file:line in section eleven (`allocator.rs:732`, `:1410`, `:159`, `:1377`, `:1494`, `:1522`; `transaction.rs:4839`, `:4848`, `:6266`–`:6270`, `:109`, `:2390`, `:3420`, `:4732`, `:4733`, `:3553`–`:3559`; `root_ring.rs:114`–`:115`; `journal.rs:196`–`:201`; `singlefs-format/src/lib.rs:184`, `:185`, `:191`, `:213`, `:238`). All rules matched what the registry says, no stop. Apparatus copies these as local constants with source comments; `local_constants_agree_with_the_crates_values_they_copy` and group-A anchors compare them.
- 21 unit tests: `cargo test --release --bin e166-parallel-commit-shared-state-partition` → `21 passed; 0 failed`; `grep -c '#\[test\]' scripts/experiment-apparatus/e166_parallel_commit_shared_state_partition.rs` gives 21.
- 19 mutations V1–V19: 19 caught, 0 invalid, 0 not failing; `mutate.sh` ends “memory-cap 0 (limit 10G), timeouts 0” and “restored, baseline still green.”
- Memoized-state cap `MEMO_STATE_LIMIT = 4_000_000` (line 1408): largest segment-one cell (「P设」 unlocked, D = 2, n = 3) reached 1,330,186 states and 2.5 GiB process peak. Without the cap V10 hit the 10G limit, neither caught nor missed and failing the mutation run; with the cap it assertion-fails at the limit and is caught. This is an apparatus resource cap, changes no criterion or cell result; segment-one artifact used the capped binary.

### Result rows copied from artifact

All transcribed directly from `e166-parallel-commit-shared-state-partition-2026-10-01.out`; the beginning of each line is the line number from that file itself (obtained via `grep -n`), with `E7RESULT ` omitted from the beginning of each line.

Self-check and anchors (lines 1 and 94; 92 `name=anchor` rows all `ok=true`, registration section-seven groups A 8 + B + C match individually, plus supplemental anchors below):

```
1:name=config experiment=E166 segment=1 default_devices=2 sample_devices=1 user_data_pairs=16 commit_pool_slots=64 root_slots_per_region=4 journal_ring_slots=196608 device_disagreement_retry_cap=2 regions=transactions inode_reservation=exact shapes=same,disjoint initial=file,nofile transactions=2,3 stages=Q1甲,Q1乙,Q1丙
94:name=verdict anchors_checked=92 anchors_mismatched=0 anchors_all_ok=true
```

Counts of result lines by family and cell count per arm (counted by a single command, output as-is):

```
$ grep -o 'name=[a-z_]*' e166-parallel-commit-shared-state-partition-2026-10-01.out | sort | uniq -c
     92 name=anchor
    288 name=cell
      1 name=config
    144 name=control_cell
      1 name=discrimination_selfcheck
      1 name=done
    138 name=geometry_sensitivity
    192 name=rounds
    288 name=trajectory
    170 name=verdict
```

Each arm has 48 `cell` and 48 `trajectory` rows (3 Q1 candidates × 2 shapes × 2 initial states × 2 n × 2 disk counts); `rounds` has 48 each only for 「P区」, 「P设」, 「O整」, 「O读」 (registration does not report R_n for A and 「A字面」).

#### Q-1 through Q-7 (duplicates): five of 126 verdict rows have total duplicates > 0

`grep -c 'no_duplicate_in_any_cell=true'` gives 121, false gives 5. The five rows:

```
1064:name=verdict question=Q-4 arm=A字面 stage=Q1甲 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=46944 no_duplicate_in_any_cell=false
1066:name=verdict question=Q-4 arm=A字面 stage=Q1丙 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=46944 no_duplicate_in_any_cell=false
1082:name=verdict question=Q-5 arm=A字面 stage=Q1甲 judgement=judged cells=8 cells_with_duplicates=8 duplicate_total=11952 no_duplicate_in_any_cell=false
1083:name=verdict question=Q-5 arm=A字面 stage=Q1乙 judgement=judged cells=8 cells_with_duplicates=8 duplicate_total=11952 no_duplicate_in_any_cell=false
1084:name=verdict question=Q-5 arm=A字面 stage=Q1丙 judgement=judged cells=8 cells_with_duplicates=8 duplicate_total=11952 no_duplicate_in_any_cell=false
```

Arm-by-arm reading (each cell written out separately by Q1 candidate, not merged):

| Arm | Q-1 through Q-7 under three Q1 candidates | Judgment (`judgement=`) |
|---|---|---|
| A (whole exclusive, proponents' form) | All 0 | Implied by definition, unjudged |
| 「A字面」 (candidate wording literally; lock covers only allocator, root ring, journal write pointer, inode watermark) | Transaction number X duplicates in Q1甲, Q1乙, Q1丙 (all 8 cells each); tree ID T duplicates only in no-file cells for Q1甲 and Q1丙 (4 of 8 per candidate; batch publisher issues T in Q1乙, so none). U, G, I, J, JS, TXG, RS all 0 | Judged (failure 7 premise holds) |
| 「P区」 | All 0 under default regions = transactions and enough reserve; Q-4 is judged and also 0 | Q-4 judged, rest implied |
| 「P设」 | Q-1–Q-7 all 0 with locking, each disk table taken under its own lock | Q-1 known before run; Q-2–Q-7 judged |
| 「O整」 | All 0 | Implied, unjudged |
| 「O读」 | All 0 | Implied, unjudged |

#### Q-8 (conflict recomputation)

```
1142:name=verdict question=Q-8 arm=P设 stage=Q1甲 judgement=known_before_run cells=8 retries_total=12094527412260 cells_with_retries=6 device_truncated=311170212792 region_fallbacks=0 region_deadlocks=0
1143:name=verdict question=Q-8 arm=P设 stage=Q1乙 judgement=known_before_run cells=8 retries_total=12094527412260 cells_with_retries=6 device_truncated=311170212792 region_fallbacks=0 region_deadlocks=0
1144:name=verdict question=Q-8 arm=P设 stage=Q1丙 judgement=known_before_run cells=8 retries_total=12094527412260 cells_with_retries=6 device_truncated=311170212792 region_fallbacks=0 region_deadlocks=0
1145:name=verdict question=Q-8 arm=O整 stage=Q1甲 judgement=definition_implied cells=8 retries_total=4576 cells_with_retries=8 device_truncated=0 region_fallbacks=0 region_deadlocks=0
1146:name=verdict question=Q-8 arm=O整 stage=Q1乙 judgement=definition_implied cells=8 retries_total=4576 cells_with_retries=8 device_truncated=0 region_fallbacks=0 region_deadlocks=0
1147:name=verdict question=Q-8 arm=O整 stage=Q1丙 judgement=definition_implied cells=8 retries_total=4576 cells_with_retries=8 device_truncated=0 region_fallbacks=0 region_deadlocks=0
1148:name=verdict question=Q-8 arm=O读 stage=Q1甲 judgement=judged cells=8 retries_total=4576 cells_with_retries=8 device_truncated=0 region_fallbacks=0 region_deadlocks=0
1149:name=verdict question=Q-8 arm=O读 stage=Q1乙 judgement=judged cells=8 retries_total=2512 cells_with_retries=6 device_truncated=0 region_fallbacks=0 region_deadlocks=0
1150:name=verdict question=Q-8 arm=O读 stage=Q1丙 judgement=judged cells=8 retries_total=4576 cells_with_retries=8 device_truncated=0 region_fallbacks=0 region_deadlocks=0
```

A, 「A字面」, 「P区」 have `retries_total=0` for every Q1 candidate (lines 1133–1141). 「P设」 per-disk disagreement and truncation are `device_disagreements`, `truncated` in cell rows. 「O读」 has no retries in 2 of 8 Q1乙 cells (disjoint, n = 2: transaction 0 reads only inode start, transaction 1 only one user-data pair). Under Q1甲 and Q1丙 「O读」 and 「O整」 totals match cell by cell (unit test `read_set_retries_equal_whole_version_retries_when_base_version_is_in_the_read_set`; failure 5 not triggered).

#### Q-9 (synchronization rounds R_n, only “no more than serial” is decidable)

```
1157:name=verdict question=Q-9 arm=P区 stage=Q1甲 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=8 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1158:name=verdict question=Q-9 arm=P区 stage=Q1乙 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=8 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1159:name=verdict question=Q-9 arm=P区 stage=Q1丙 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=8 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1160:name=verdict question=Q-9 arm=P设 stage=Q1甲 judgement=judged cells=8 cells_with_fewer_rounds_than_serial=8 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1161:name=verdict question=Q-9 arm=P设 stage=Q1乙 judgement=judged cells=8 cells_with_fewer_rounds_than_serial=8 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1162:name=verdict question=Q-9 arm=P设 stage=Q1丙 judgement=judged cells=8 cells_with_fewer_rounds_than_serial=8 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1163:name=verdict question=Q-9 arm=O整 stage=Q1甲 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=0 cells_without_round_count=0 rounds_equal_serial_in_every_cell=true
1164:name=verdict question=Q-9 arm=O整 stage=Q1乙 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=0 cells_without_round_count=0 rounds_equal_serial_in_every_cell=true
1165:name=verdict question=Q-9 arm=O整 stage=Q1丙 judgement=definition_implied cells=8 cells_with_fewer_rounds_than_serial=0 cells_without_round_count=0 rounds_equal_serial_in_every_cell=true
1166:name=verdict question=Q-9 arm=O读 stage=Q1甲 judgement=judged cells=8 cells_with_fewer_rounds_than_serial=0 cells_without_round_count=0 rounds_equal_serial_in_every_cell=true
1167:name=verdict question=Q-9 arm=O读 stage=Q1乙 judgement=judged cells=8 cells_with_fewer_rounds_than_serial=4 cells_without_round_count=0 rounds_equal_serial_in_every_cell=false
1168:name=verdict question=Q-9 arm=O读 stage=Q1丙 judgement=judged cells=8 cells_with_fewer_rounds_than_serial=0 cells_without_round_count=0 rounds_equal_serial_in_every_cell=true
```

`rounds_equal_serial_in_every_cell=false` means some cell commits more than one transaction per round under that Q1 candidate (the model-level “no more than serial” condition does not hold), not failure. 「O读」 uses fewer than n rounds only in Q1乙 disjoint shape (n = 2: one round; n = 3: two rounds, 2 then 1 commits); same shape equals n. Under Q1甲/Q1丙 BASE version is in read set, R_n = n. A and 「A字面」 report `judgement=not_reported` (1151–1156). 「P区」 and 「P设」 give each transaction a different region or lowest free pair in the same synchronization round, R_n = 1; 「P设」 per-disk disagreement appears only in interleavings (Q-8), not synchronized rounds.

#### Failure conditions

```
1169:name=verdict failure=2 clause=A_default_geometry_duplicate triggered=false
1170:name=verdict failure=4 clause=ruler_cannot_separate_candidates triggered=false
1171:name=verdict failure=5 clause=read_set_not_as_registered triggered=false
1172:name=verdict failure=6 clause=device_partition_does_not_converge triggered=true cells_with_truncation=18
1173:name=verdict failure=7 clause=literal_lock_misses_shared_state triggered=true cells_with_x_or_t_duplicates=24
1174:name=verdict failure=8 clause=judgement_flips_with_q1_candidate triggered=true flipping_columns=A字面:T
1175:name=verdict failure=9 clause=read_set_below_whole_in_stage_two_same_shape triggered=false
```

Failure 6 (「P设」 truncation > 0 at D = 2) was known before running (registration section four C: n = 2: 3366; n = 3: 155,468,679,642). Under the weakest reading record one loss: 2026-10-01 implementation refuses when disks disagree about the same user-data placement (`PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported` in `allocator.rs`); some schedules do not converge within retry cap 2, cap not increased. Failure 7 (literal whole-exclusive definition omits transaction number and tree-ID watermark) triggers, one loss under weakest reading; A proponents' form unaffected. Failure 8 (judgment flips by Q1 candidate) triggers only 「A字面」's tree ID: not an apparatus error; that column is judged only under Q1甲 and Q1丙. Failures 1/3 checked by 92 anchors (0 mismatches); 2, 4, 5, 9 not triggered.

### Absolute-value assertions and positive controls

All 92 anchors `ok=true`: group A (serial baseline S and single txg in Q1乙, section-seven A 1–8), group B (B 1–7 including locked/unlocked 「P设」 for D = 1/2 and n = 2/3; unlocked n = 3 D = 2 has 15,204,801,979,811,387,676 schedules exceeding u64, apparatus uses u128), group C (schedule count = (n·k)! ÷ (k!)ⁿ, serial baseline duplicates/retries zero, memoized 「P设」 summary equals per-leaf at n = 2), supplemental anchors (「A字面」 X/T outside lock, independent Python `literal_outside_anchor.py` in registration section twelve) and structural extensions for single-number resources and G in Q1乙/Q1丙 controls. Six controls (A unlocked, 「A字面」 unlocked, 「P区」 same, 「P设」 unlocked, 「O整」 no guard, 「O读」 no guard) disable their own guards in 144 `control_cell` cells.

### Geometry sensitivity

First reverse sample D = 1: `grep -c 'outcome=two_points_same'` gives 138, `outcome=unstable` 0 (one per arm × Q1 candidate × question, Q-1–Q-7 and Q-9); discriminative power self-test `discrimination_selfcheck status=not_needed_every_cell_two_points_same` (no threshold movement when judgments match, section eight). D = 1 is structurally the same calculation for non-「P设」 arms: when disk tables change synchronously, U counts one starting slot once and disk count is not state (section-twelve 2.1), so their matching judgments are structural; 「P设」 rows are measured (duplicates 0 and R_n = 1 for locked D = 1/2). Disjoint shape and no-file initial state are run across all cells, individually reported (「O读」 Q1乙 Q-8/Q-9 and 「A字面」 T vary). Segment-two samples (K = n − 1, insufficient multiple-inode reserve, S = 1, ring slots 2) and supplemental Q-10 are marked “not run after sufficient decision.”

### Mutations

First run: 19 V1–V19 caught, invalid 0, not failing 0 (table after second rerun has 21; see rerun section). Original `mutate.sh` output `e166-parallel-commit-shared-state-partition-2026-10-01-mutations.out` has 22 lines, not replayed bytewise; mutation lane has its own baseline/restoration gates:

```
$ for s in ✅ ⏭ ❌ 💥 ⚠️ ⏱ 🧱; do echo "$s $(grep -c "^$s" e166-parallel-commit-shared-state-partition-2026-10-01-mutations.out)"; done
✅ 19
⏭ 0
❌ 0
💥 0
⚠️ 0
⏱ 0
🧱 0
```

- V8 (「P区」 COMMIT does not release region) was expected insensitive in segment one, pending segment two; apparatus adds a K = n − 1 model-path unit test `one_region_for_two_transactions_makes_the_second_claim_wait_for_the_first_commit` (cell not reported in artifact), so it was caught in segment one (12.3).
- V19 (Q-9 commit order reversed) is insensitive under default disjoint shape: ascending and descending both give [2, 1] commits per round. Exhaustive n ≤ 7 read-set conflict shapes (`round_order_search.py` in section twelve) find ascending and descending R_n never equal, only distribution per round changes. A custom shape (transaction 0 creates inode only, 1 writes unit only, 2 does both) gives ascending [2, 1], descending [1, 2], pinned by a unit test. V19 is caught by per-round commit counts, not R_n.
- V5/V6 are caught in Q1乙 disjoint shape by tests that 「O读」 validation includes the user pair remaining free and issues transaction number at commit (`read_set_optimistic_checks_the_user_pair_and_takes_the_transaction_number_at_the_commit_point`).

### Rerun

`bash scripts/experiment-apparatus/replay.sh E166` (partitioning shared state for parallel commit): (registered row E166 (partitioning shared state for parallel commit) `|e166-parallel-commit-shared-state-partition||e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out|exact`). A single run takes about half a minute (the version prior to adding the memoization limit took 34.6 seconds of `real` time in a process with 5 threads and a 4G memory limit); the process peak of 2.5 GiB is `%M` from `/usr/bin/time`, and the measured version merely adds one line printing the state count to stderr, with all remaining source code identical. After rerunning, the artifact is compared byte-by-byte against `e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out`. The first-run artifact `e166-parallel-commit-shared-state-partition-2026-10-01.out` is retained, though the replay registration row no longer points to it; rerunning the apparatus from before the renumbering as-is (arm e1) is byte-for-byte identical to it, see the “Second rerun (tree ID renumbering)” section.

### Second rerun (tree-ID renumbering)

Registration `e166-r2-prereg.md` (SHA-256 `428b9dd48c0dee737f84362f6fbc0e85691c5f7b2f4fbe787f0c5b6d35bba903`, section-twelve revisions recorded); R3 in `m3-treeid-rerun-questions.md`: are E166 conclusions line-by-line identical under new tree-ID and watermark constants? Input changes in `crates/`: mkfs watermark 11 → 65536, after first file version 19 → 65548, IDs issued for first file version 8 → 12 (D8 (Core Index Structure) settled item 11 and settled item 8 ② “issue 12 tree IDs consecutively from watermark”). Apparatus changed to clause wording, not to `crates/` or old artifact: tree-ID family (two watermarks, count, shared `tree_watermark`, held `tree_start`, group-A6 start `Vec`) widened from `u8` to `u32` (`TreeIdentifierNumber`); both watermarks changed, 13 anchor expectations changed (A6 final watermark now literal `"65548"`, formerly constant itself), new values pinned by tests. Row types, counts, format unchanged.

Artifacts (`sha256sum`, `wc -l`):

| File (under `research/results/`) | Contents | sha256 | Lines |
|---|---|---|---|
| `e166-parallel-commit-shared-state-partition-r2-arm-old-2026-10-02.out` | Arm e1: unchanged pre-renumbering apparatus rerun | `7b9a829ba5568e13d9c4f15941bdb0fb3fb8c20d27a3a0ec9465ba5d4ac1ffeb` | 1315 |
| `e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out` | Arm e2: post-renumbering apparatus, main artifact | `64467cf0efc4222f901747db3980b5eb601d9f9567932e592c11516f2fff6e29` | 1315 |
| `e166-r2-comparisons-2026-10-02.out` | Q-11 through Q-13: comparison output of the main artifact against the first-run artifact | `c4948e64bb1aa44a7415a4c2be372002f00a42148dc2861e21927dafa2b68676` | 13 |
| `e166-r2-controls-2026-10-02.out` | PC-d / PC-c comparator positive controls | `a4fb511743a5259dfe4d2e6c1b10a567f182f72feecf3d61b0d92720c701c7fd` | 42 |
| `e166-r2-compare-script-2026-10-02.txt` | Full comparison script (shared by Q-11 through Q-13 and PC-d) | `dadbb1b8f05ed8a055767bc27a34724c75de249afc7bac53024b02d7e1730626` | 181 |
| `e166-r2-make-controls-script-2026-10-02.txt` | Full script producing three PC-d copies | `979db4c75c7739398b163c3c3dbba1a4d811ddea2becfb6e25b3b4dc0d714aed` | 63 |
| `e166-parallel-commit-shared-state-partition-r2-mutations-before-2026-10-02.out` | 19 mutations against unchanged apparatus | `d9f43876d174c911f17af4fcbd8ab5b128f22b5a1226e9305a15e5cf847d943a` | 22 |
| `e166-parallel-commit-shared-state-partition-r2-mutations-after-2026-10-02.out` | 21 mutations against modified apparatus | `3b9d8b208ccfe0a51a8e19370ebac516fac3bbed3302c4edb56daa444a67bf9b` | 24 |

The comparison script does not enter verbatim replay; replay commands for artifacts and comparison output: `bash scripts/experiment-apparatus/replay.sh E166` (partitioning shared state for parallel commit) compares the main artifact; for comparison output, running `python3 e166-r2-compare-script-2026-10-02.txt` directly is not possible (suffix is `.txt`), so copy it to `.py` and then run `python3 <copy> e166-parallel-commit-shared-state-partition-2026-10-01.out research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out`.

**R3 answer: which rows changed—two rows, not “line-by-line identical.”** Full 13-line `e166-r2-comparisons-2026-10-02.out` (OLD = first artifact, NEW = main; `class=yi` is Q-12 class B: only tree-ID-derived fields differ with correct ratio):

```
compare old=e166-parallel-commit-shared-state-partition-2026-10-01.out new=e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out
Q-11 conclusion_lines old=309 new=309 paired=309 differing_pairs=2 unpaired=0 differences=2
Q-11 DIFF pair=59 class=yi differing_keys=duplicate_total
Q-11   OLD E7RESULT name=verdict question=Q-4 arm=A字面 stage=Q1甲 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=46944 no_duplicate_in_any_cell=false
Q-11   NEW E7RESULT name=verdict question=Q-4 arm=A字面 stage=Q1甲 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=70416 no_duplicate_in_any_cell=false
Q-11 DIFF pair=61 class=yi differing_keys=duplicate_total
Q-11   OLD E7RESULT name=verdict question=Q-4 arm=A字面 stage=Q1丙 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=46944 no_duplicate_in_any_cell=false
Q-11   NEW E7RESULT name=verdict question=Q-4 arm=A字面 stage=Q1丙 judgement=judged cells=8 cells_with_duplicates=4 duplicate_total=70416 no_duplicate_in_any_cell=false
Q-12 scope=verdict(D=2 算出的格) jia=0 yi=2 bing=0
Q-12 scope=geometry_sensitivity jia=0 yi=0 bing=0
Q-13 lines old=1315 new=1315 paired=1315 unpaired=0
Q-13 classes same=1228 yi_derived_ratio_ok=74 anchor_section_seven=13 other=0
Q-13 devices=1 (cell,trajectory,rounds) classes same=368 yi_derived_ratio_ok=16 anchor_section_seven=0 other=0
```

Conclusion rows are `name=verdict`, `name=geometry_sensitivity`, `name=discrimination_selfcheck`: 309 each (170 + 138 + 1), paired by order and compared bytewise. Two differing rows are 「A字面」 tree-ID duplicate totals in first artifact lines 1064/1066: 46944 → 70416, 70416 × 2 = 46944 × 3, exactly 12 / 8. Judgment fields `judgement=`, `cells=`, `cells_with_duplicates=`, `no_duplicate_in_any_cell=` unchanged. All zero `duplicate_total` Q-4 rows in first run (A, 「P区」, 「P设」, 「O整」, 「O读」 under three Q1 candidates and 「A字面」 Q1乙), all geometry sensitivity rows, and self-check `status=not_needed_every_cell_two_points_same` are byte-identical. Registered failure 11 (judgment flip, class A ≥ 1), failure 12 (class C ≥ 1 or Q-13 “other” ≥ 1), failure 13 (row set changed) did not trigger: A 0, C 0, other 0, unpaired 0, both 1315 rows.

Of 1315 Q-13 pairs: 1228 same; 74 only tree-ID-derived values differ with correct ratio (40 `control_cell` rows' `T_total`/`T_max_single`, 16 `cell` rows' same, 16 `trajectory` rows' `T_peak`, 2 `verdict` rows' `duplicate_total`; counted by Python, not manually); 13 named section-seven anchors (A6 three, B2 eight, supplemental no-file two; `observed=` and `expected=` both equal registered new values, `ok=true`); other 0. D = 1's 384 `cell`, `trajectory`, `rounds` rows: 368 same, 16 class B, other 0. D = 2 and D = 1 classifications agree (no class A/C), not marked unstable.

Guard cells (commands and original output):

```
$ cmp e166-parallel-commit-shared-state-partition-r2-arm-old-2026-10-02.out research/results/e166-parallel-commit-shared-state-partition-2026-10-01.out; echo "cmp_exit=$?"
cmp_exit=0
$ diff e166-parallel-commit-shared-state-partition-r2-arm-old-2026-10-02.out e166-parallel-commit-shared-state-partition-2026-10-01.out | wc -l
0
$ grep -n 'name=verdict anchors_checked' research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out e166-parallel-commit-shared-state-partition-r2-arm-old-2026-10-02.out
research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out:94:E7RESULT name=verdict anchors_checked=92 anchors_mismatched=0 anchors_all_ok=true
research/results/e166-parallel-commit-shared-state-partition-r2-arm-old-2026-10-02.out:94:E7RESULT name=verdict anchors_checked=92 anchors_mismatched=0 anchors_all_ok=true
```

- Q-14 (e1 equals first artifact): `cmp` exit 0 and `diff | wc -l` 0; first artifact is the unchanged pre-renumbering apparatus reproduced on 2026-10-02.
- Q-15 anchors: both main and e1 summaries `anchors_checked=92 anchors_mismatched=0 anchors_all_ok=true`; exactly 13 of 92 differ from first run and match independently calculated new section-seven values (A6 three `65536,0,0`, `65548`, twelve IDs `65536,…,65547`; B2 eight `T=48/4`, `T=1296/72`; supplemental no-file two `T=216/18`, `T=34992/1620`).
- Q-16 (determinism): outputs of five runs of the identical binary all have sha256 `64467cf0efc4222f901747db3980b5eb601d9f9567932e592c11516f2fff6e29` (a deterministic model, demonstrating that there is no hidden state, rather than demonstrating statistical stability).
- Q-17 seven mutation columns: before, 19 caught, invalid 0, not failing 0 and 💥/⚠️/⏱/🧱 0; after, 21 caught, invalid 0, not failing 0, other columns 0, V13/V20/V21 fail. Both end restored and baseline green. V20 (subtract 1 from both watermarks) caught by `local_constants_agree_with_the_crates_values_they_copy` (65536, 65548, 12) and `serial_baseline_and_stage_definitions_match_the_clause_values` (A6 three, final watermark literal). Constant test alone catches it; not solely due to literal expectation. All 21 unit tests pass.
- PC-d comparator discrimination (`e166-r2-controls-2026-10-02.out`): three copies of first artifact compared with same script. α unchanged: 0 conclusion differences, all 1315 pairs same. β changes line 1007 Q-1 `judgement=` to `pc_flipped` and line 1061 Q-4 `duplicate_total=` 0 → 1: 2 conclusion differences, class A 1 and C 1. γ changes line 97 `control_cell` `T_total=` 32 → 48: 0 conclusion differences, exactly 1 full-row pair classified tree-ID-derived with correct ratio. PC-c: β versus first `cmp` says `differ: byte 506471, line 1007`.

#### Verdict rows: every non-passing field

`grep -n 'name=verdict'` gives 170 rows each in main and e1, identical except two `duplicate_total=` values. Each has 16 rows containing `=false` or `not_run`: two 「A字面」 Q-4 `no_duplicate_in_any_cell=false` (tree-ID duplicates 70416, first 46944); three 「A字面」 Q-5 (11952, unchanged); seven Q-9 `rounds_equal_serial_in_every_cell=false` (three each 「P区」/「P设」 and one 「O读」 Q1乙, meaning some cell commits more than one transaction per round, not failure); four `triggered=false` failures 2/4/5/9, i.e. not triggered. Three `triggered=true`: failure 6 `cells_with_truncation=18`, failure 7 `cells_with_x_or_t_duplicates=24`, failure 8 `flipping_columns=A字面:T`, same as first run and already recorded (failure 6 known before run, 7/8 counted one loss under weakest reading). Violation/mismatch/ambiguity counts: `anchors_mismatched=0`; no `not_run`.

### What the second rerun cannot answer

- R3 asks whether this independent model's conclusions change with constants, not whether `crates/` is correct. Correspondence relies on reading names before run (`TREE_IDENTIFIER_WATERMARK_AT_MKFS` 65536, `TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH` 65548, twelve `OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER`) and tests; not an implementation apparatus and does not prove `crates/` matches clauses.
- Both value (11 → 65536) and count (8 → 12) changed together; no separate point, e.g. mkfs 65536 but still issuing 8 contradicts registered reading 9 and item 8 ②, so not modeled. Separate runs need new dispatch by main agent.
- Model uses tree IDs only to issue a consecutive batch from watermark, add a constant to watermark, and compare equality; thus changing (watermark, count) changes only batch length (inferred from code). Artifacts confirm all differences are tree-ID-derived at exactly 12 / 8, no others. They do not prove invariance under other operations such as issuance by class.
- No samples beyond section seven were added for disk count, shape, initial state or transaction count; difference structure matches sampled cells (Q-13 D = 1 also 16 class-B rows), not extrapolated to unrun cells.

### Path and conclusion registration

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Rust counting model | State machine enumerates all interleavings of per-arm step sequences per leaf or memoized state, computes holders − 1 for each resource | `scripts/experiment-apparatus/e166_parallel_commit_shared_state_partition.rs:840 apply_step`; `:1424 explore`; `:577 take_resources` | Local constants with `crates/` file:line sources in comments and registration section eleven; 5.1 model geometry; 5.2 arm definitions |
| 2 | Independent Python design anchors | Closed-form / memoized counts for positive controls and partition arms (`anchors.py`, `anchors_partition2.py`, `anchors_tree.py`, `anchors_literal.py`) plus `literal_outside_anchor.py` | No implementation: full text checked into `e166-preregistration.md` sections thirteen and twelve, no independent `.py` files | 5.1 geometry (16 pairs, 64 slots, 2 nodes, initial t0 = 5, etc.); 5.2 arms; disk rules (lowest free pair, issue only if disks agree) from same `crates/` sources |
| 3 | Section-seven group-A clause formulas | Serial baseline S individually calculated from clause formulas (txg + 1, ring slot, transaction number, inode and tree-ID watermarks) | `scripts/experiment-apparatus/e166_parallel_commit_shared_state_partition.rs:1628 anchors_from_the_clauses` | `crates/` file:line (section eleven), apparatus imports no `crates/` constants/functions |

Assertions link path 1 to 2 in `anchors_from_independent_calculations` (line 1735) and `anchors_from_structural_counts` (2067), with 92 `name=anchor` rows `ok=true`; path 1 to 3 uses `anchors_from_the_clauses`. Paths 1 and 2 share registered geometry and arm definitions: wrong registration or geometry can make both wrong but equal; group-A clause/actual-code anchors and mutation table pin those. Values sweep all model schedules (all interleavings of two/three transactions), not samples.

### What it cannot answer

- Model-level counts only, no timing or throughput. Q4 reversal's second half “conflict rate lowers throughput below serial” is decidable only to “no more than serial” (Q-9); “below” requires recomputation and validation times, i.e. Q1 local-device throughput.
- Only 2/3 transactions, one initial value, two shapes, six arms; no duplicate counts for more transactions, more complex shapes, merging tree contents (Q3), crash prefix (Q2/Q7), admission oversell (Q5).
- 「A字面」 X/T duplicates show the four things named literally in candidate do not cover transaction number and tree watermark touched on each `crates/` publish; not proof whole exclusive is infeasible: A proponents' form locks from read to write-back and is all 0. Which reading defines whole exclusive belongs to main agent.
- 「P设」 at D = 2 can disagree on lowest free pair and fail to converge within retry cap 2: weakest reading of failure 6, not proof device partitioning is impossible. A different form requires the three steps in `.claude/singlefs-ai-sop/rules/evidence-discipline.md` on pre-fixed arm definitions and is a new experiment.
- 「O读」 uses fewer than n rounds in Q1乙 only for disjoint read sets; same shape (each transaction creates inode) equals n as 「O整」.
- Default geometry where number of regions = number of transactions and sufficient reserve: FALLBACK and deadlock paths for 「P区」 are never reached under the default geometry (in the artifact `region_fallbacks=0`, `region_deadlocks=0`); the model path is verified only by unit tests, and the artifact contains no readings (sampling points of the second segment).
- ACC accounting lost updates and BASE lost updates (Q-10) unmodeled; FRZ, SC, BC (frozen publishes, system configuration slot generation, reverse chain) listed as unmeasured.
- 「O读」 no-guard and 「O整」 no-guard controls use the same code (12.2.3); no separate control for 「O读」's “issue transaction number at commit” guard. V6 mutation catches it.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 2 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |
| D13 (Verification Route) settled item 10 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |
| D3 (Space Allocation) settled item 6 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |
| D28 (Mount-Time Commitments) settled item 2 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |
| D16 (Publish Semantics) settled item 4 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |
| D8 (Core Index Structure) settled item 11 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |
| D8 (Core Index Structure) settled item 8 | Background | 2026-10-18 unaffected: prior artifacts archived in version control (recoverable from git history), body references reduced to filenames; conclusions and numbers unchanged |

## Historical Versions

### 2026-10-01: page created, segment one run

Apparatus, 21 tests, mutation table (19 all caught), first artifact and replay registration completed the same day. Segment two (remaining section-eight samples and Q-10) not run, registered as “not run after sufficient decision.”

### 2026-10-02: second rerun (tree-ID renumbering)

R3 compares conclusion rows after tree-ID watermarks 11 / 19 → 65536 / 65548 and first-file issuance 8 → 12. Apparatus tree-ID family widened `u8` → `u32`, 13 anchors updated, mutation V13 re-anchored and V20/V21 added (all 21 caught), unit tests remain 21. Arm e1 (unchanged pre-renumbering apparatus) matches first artifact bytewise. Main artifact `research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out`: 2 of 309 conclusion pairs differ (「A字面」 Q-4 tree-ID duplicates in Q1甲/Q1丙 46944 → 70416, ratio 12 / 8), no judgment field changes. Of 1315 pairs, 1228 same, 74 tree-ID-derived changes at ratio, 13 named anchors, 0 other. E166 `replay.sh` entry points to main artifact.
