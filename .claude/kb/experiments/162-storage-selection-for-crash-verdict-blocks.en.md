## E162 Storage Backend Selection for Crash Amplification Judgment — S1 Decisive Tier Delivered, S2 / S3 Not Yet Run

[中文](162-崩溃放量判定块存储选型.md) · **English** · [日本語](162-クラッシュ増幅判定ブロックのストレージ選択.ja.md)

<!-- doc-lint:not-numbers V15 -->

Pre-registration `e162-preregistration.md`; question list `m2-crash-store-r1-forks.md` (S1, S2, S3). Under the user's decision (section-twelve revision six), a feasibility tier was first run solely to verify the apparatus runs and is usable (20 kills per arm for S1, 10⁴ blocks per arm for S2). S1 and S2 were marked “unjudged (feasibility tier only)”; S1-large, four geometry samples (G-bs10, G-batch16, G-sparse, G-random) and S3 were not run. Later that day the S1 decisive tier (segment two) completed the registered 200-kill main cell for each of four arms and checked all sufficiency conditions (6.1, 11.1 V1–V6): **S1 is sufficient to decide: none of the four arms flipped in these 200 kills** (lost blocks, corrupt blocks, confirmed/unconfirmed read errors, phantom blocks and open failures all zero). S2/S3 were not run in segment two; see “S1 decisive tier (2026-09-27, segment two).” Apparatus `scripts/experiment-apparatus/e162_crash_verdict_block_store.rs` is an independent handwritten model outside `crates/`, not touching `crates/singlefs-core` or `crates/singlefs-checker`. Earlier that day all 23 unit tests passed three times under the 4G memory cap; before the decisive S1 run the same tests were rerun and again passed 23 / failed 0 (`BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` worked around the prior page's `librocksdb-sys` build blocker; see limitations). Mutation table `scripts/mutation/tables/e162_crash_verdict_block_store.tsv` has 13 rows (current `wc -l`, unchanged; source unchanged in this segment); earlier that day `mutate.sh` caught **all** (invalid 0, no failure 0, memory-cap 0, timeout 0), restored and baseline remained green. Source was not changed and mutation table was not rerun in this segment.

Four arms: F (hardened files: temporary file + fsync + rename + directory fsync), R1 (redb, `set_quick_repair(true)`), R0 (redb, default repair), K (RocksDB, synchronous writes + WAL).

### What was done

- **S1**: each arm executes the first 20 rows of the kill-point plan (18 kills during writes + 2 during opening, fixed seed `seed_kill=0xe162000000000003`); reopen and verify each block after each kill (Q1a–Q1f). Positive controls PC-L, PC-S, PC-P, PC-O run on all arms; PC-T (half file) defined only for F under registration 5.3.
- **S2**: each arm writes 10⁴ blocks (`states_per_block=65536`), reporting sustained write rate Q2a, cold/warm random-read latency by block key after close and reopen Q2d, and occupancy/write-byte ratio Q2e. PC-rate, PC-read, PC-space all run. S1-large, G-bs10, G-batch16, G-sparse, G-random and S3 are excluded from this feasibility tier under revision six.
- **Anchors**: 22 independently computed section 7.2 anchors (SplitMix64 reference, CRC-32C standard check value, block-value generation, key encoding, permutation, kill plan, arithmetic etc., pinned to verbatim output of registration 13.3 anchor script) are compared individually with apparatus startup self-check.

### Result rows copied from artifacts

**S1 verdict rows** (`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-feasibility.out`):

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:73:E7RESULT name=verdict part=s1 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:51:E7RESULT name=verdict part=s1 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:51:E7RESULT name=verdict part=s1 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:51:E7RESULT name=verdict part=s1 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

**S1 positive-control rows** (PC-S / PC-P / PC-O / PC-L for all arms; F additionally PC-T):

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=F open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=F open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=F open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=file:_FORMAT_不对
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=F write_path=file-lazy-rename write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=14 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:51:E7RESULT name=positive_control control=PC-T arm=F write_kills=18 write_kills_with_read_error=16 q1b=0 extended_to_200=false ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=R1 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=R1 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=R1 open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=redb:_I/O_error:_Not_a_redb_database:_magic_number_mismatch
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=R1 write_path=redb-durability-none write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=R0 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=R0 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=R0 open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=redb:_I/O_error:_Not_a_redb_database:_magic_number_mismatch
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=R0 write_path=redb-durability-none write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=K open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=K open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=K open_failed=true opened_after_repair=yes q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true open_error=rocksdb:_IO_error:_No_such_file_or_directory:_While_opening_a_file_for_sequentially_reading:_/tmp/e162-3127058/pc-o/MANIFEST-999999:_No_such_file_or_directory
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=K write_path=rocksdb-no-wal write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
```

The four arms' PC-S/PC-P/PC-O match field by field except `open_error` wording, all `ok=true`; K's PC-O additionally reports `opened_after_repair=yes` (reopen succeeds after `DB::repair`), absent for F/R1/R0 because registration defined repair only for K.

**S2 verdict rows**:

```
research/results/e162-crash-verdict-block-store-2026-09-27-s2-F-feasibility.out:41:E7RESULT name=verdict part=s2 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33530296.2 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R1-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33593904.5 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R0-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=19339984.1 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-K-feasibility.out:41:E7RESULT name=verdict part=s2 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=48638773.4 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
```

**S2 sustained throughput, random-read latency and occupancy ratio** (`name=throughput`, `name=random_read`, `name=occupancy_summary`, artifact lines 11, 39, 40, 37):

| Arm | Sustained throughput (states/s) | Commit latency median/p99/max (µs) | Cold read median/p99 (µs) | Warm read median/p99 (µs) | Peak occupancy ratio `peak_ratio_high` | After-close occupancy ratio `after_close_ratio` |
|---|---|---|---|---|---|---|
| F | 33 530 296.2 | 1529 / 5036 / 583066 | 321 / 399 | 123 / 144 | 1.0639 | 1.0625 |
| R1 | 33 593 904.5 | 1550 / 4352 / 115139 | 230 / 352 | 5 / 8 | 2.0029 | 2.0021 |
| R0 | 19 339 984.1 | 903 / 8299 / 2558476 | 229 / 366 | 5 / 11 | 2.0039 | 2.0017 |
| K | 48 638 773.4 | 804 / 1293 / 1733112 | 157 / 367 | 12 / 21 | 2.1495 | 1.0013 |

All four arms have `q2e_flips_at_1_9e10`, `q2e_flips_at_1_5e10`, `q2e_flips_at_1_9e10_using_low` = `false`: multiplying these feasibility occupancy ratios by 1.9×10¹⁰ or 1.5×10¹⁰ bytes remains within available bytes reported by `df -B1` at start (F 1,979,726,131,200; R1 1,979,601,149,952; R0 1,979,590,852,608; K 1,980,708,077,568). R1/R0 after close occupy about twice written bytes (redb forms close, `peak_ratio_high` 2.0029 / 2.0039); K peaks at 2.1495 during writes but drops to 1.0013 after close (RocksDB compaction returns occupancy near written bytes); F stays around 1.06. These are single raw readings from feasibility only, 10⁴ blocks and `samples=10`, not repeated or judged and not final conclusions; the throughput differences (R0 visibly slower than the others) are likewise provisional.

**Anchors verdict**: `research/results/e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out` contains `E7RESULT name=verdict part=anchors anchor_mismatches=0` and `E7RESULT name=done emitted=27`; all 22 `name=anchor` rows have `matches_registration=true` (2026-09-28 path migration rerun is byte-identical to pre-rename `e162-crash-verdict-block-store-2026-09-27-anchors.out`, verified with `cmp`).

### Verdict fields called out

Among verdict rows listed by `grep -n 'name=verdict' <产物>`, no field is boolean `false`. Every non-`true`, nonzero field is listed below; all are within the feasibility scope fixed in section-twelve revision six, expected narrowing rather than apparatus failure:

- `judgement=not_judged_feasibility_tier_only` (all eight S1/S2 artifacts): this is the exact wording required by revision six of the registration—the user settled on 2026-09-27 that this run would verify only runnability and usability, and that neither S1 nor S2 would reach a conclusive sufficiency judgment based on the feasibility tier data; while the literal field name looks like “unjudged,” it is in fact this way by design.
- `pc_t_ok=not_applicable` (the three S1 artifacts for R1, R0, and K): PC-T (the truncated half-file control) is defined only for the F-direct candidate under registration section 5.3; since R1, R0, and K do not have this control, it is supposed to be `not_applicable`, rather than an unexecuted check.
- `v6_kills_inside_commit_ok=not_applicable` (all 4 S1): V6 threshold applies to 180 kills during writes (11.1 V6); feasibility has only 20 kills, 18 during writes. Source returns not applicable by tier, matching revision six: report number inside commit but do not judge.
- `s1_large=not_run_feasibility_tier` (all four S2 artifacts): revision six of the registration explicitly specifies that S1-large is not run in this feasibility tier, and the field truthfully reports “not run,” as expected.
- `q2a_verdict=pending_e161` (all four S2 artifacts): the Q2a threshold R_AB requires the t_state delivered by E161 (deduplication and segmentation timing of crash amplification); on the same day, E161 (deduplication and segmentation timing of crash amplification) also delivered only its feasibility tier (see “What it cannot answer”), so the threshold does not yet have a formal value to substitute in, and the field truthfully reports “waiting for E161 (deduplication and segmentation timing of crash amplification).”

All `q1a`–`q1f` in four S1 verdicts are 0: no lost or silently corrupt blocks, confirmed/unconfirmed read errors, phantoms or open failures in 20 feasibility kills. `anchor_mismatches=0`: all 22 anchors match registration. No integer field denoting violation, mismatch, ambiguity or failure is positive.

### Rerun

The registered row compares `research/results/e162-verdict-store-power-cut-2026-09-27-selftest-r2.out` (49 lines, penultimate `E7RESULT name=verdict part=anchors anchor_mismatches=0`).

`replay.sh` registers only deterministic anchors (S1/S2 contain kill timing and measurements, so two runs naturally differ):

```
bash scripts/experiment-apparatus/replay.sh E162
```

```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out
```

(2026-09-28 path migration: the registered row for E162 (storage selection for crash verdict blocks) in `replay.sh` was changed to point to this new artifact; it is byte-for-byte identical to `e162-crash-verdict-block-store-2026-09-27-anchors.out` prior to renaming, see “Historical Versions”.)

Direct feasibility reruns of anchors, S1 and S2 (separate commands, `<臂>` is F, R1, R0 or K; S1/S2 spawn and kill subprocesses and write anew, with timing noise, so no byte comparison to checked-in output):

```
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- anchors
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂> feasibility
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s2 <臂> feasibility
```

Unit tests: `cd scripts/experiment-apparatus && bash ../hosts/run-with-memory-cap.sh 4G cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store` (earlier that day 23 passed / 0 failed; at the time this page was written, rerun was blocked by the `librocksdb-sys` environment, see limitations).

### S1 decisive tier (2026-09-27, segment two)

Revision six approved only feasibility; this segment was dispatched by the main agent (inheriting user settlement “can run, can use, then hand verification to Milestone 2 remaining tasks,” `records/2026-09-24-里程碑二收尾调度.md` section three) to extend S1 from 20 kills to the registered 200. S1 sufficiency conditions (5.5, 6.1, 11.1 V1–V6) were unchanged. S2/S3 were not run and are left for another segment when the machine is idle.

**⚠ Missing registration amendment (executor oversight)**: before running this segment, “revision seven” documenting the narrowed scope was not appended to `e162-preregistration.md` section twelve, contrary to the discipline requiring it before the run. This only expands feasibility sample size to the originally registered amount and changes none of the criteria in sections six, seven or nine, so the judgments below are unaffected; after artifacts exist, discipline prohibits changing the registration. The gap is handed to the main agent to decide whether to add a note or accept it.

#### What was done

- **S1 (decisive tier)**: all four arms ran the original 200-kill plan (`registered` tier, no `feasibility` argument, source `CRASH_MAIN_KILL_CYCLES=200`), reopening and checking every block after each kill (Q1a–Q1f); PC-L, PC-S, PC-P, PC-O and F's PC-T were all rerun on fresh databases. Unit tests were rerun beforehand, source unchanged and mutations not rerun.
- **S2/S3**: not run in this segment; resume in a later segment.

#### Result rows copied from artifacts

**S1 verdict rows** (`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`):

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:258:E7RESULT name=verdict part=s1 arm=F tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:235:E7RESULT name=verdict part=s1 arm=R1 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=179 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:233:E7RESULT name=verdict part=s1 arm=R0 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=178 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:234:E7RESULT name=verdict part=s1 arm=K tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

**S1 main-cell summary rows** (`grep -n 'name=kill_campaign_summary campaign=S1-main' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`, cumulative 200-kill counts and reopen time):

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:257:E7RESULT name=kill_campaign_summary campaign=S1-main arm=F write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=180 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=2171 reopen_median_microseconds=1100 reopen_last_microseconds=2171 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:234:E7RESULT name=kill_campaign_summary campaign=S1-main arm=R1 write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=179 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=1407 reopen_median_microseconds=1015 reopen_last_microseconds=439 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:232:E7RESULT name=kill_campaign_summary campaign=S1-main arm=R0 write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=178 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=399403 reopen_median_microseconds=124691 reopen_last_microseconds=472 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:233:E7RESULT name=kill_campaign_summary campaign=S1-main arm=K write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=180 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=3271087 reopen_median_microseconds=11411 reopen_last_microseconds=5818 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
```

All arms have `cycles_verified=200`, `q1a_lost` through `q1f_timeouts` and `repaired_openings` all 0; `q1g_kills_inside_commit` 180/179/178/180 all ≥ 100 (V6 threshold among 180 write-phase kills), `write_phase_kills=180`, `confirmations_before_write_kills=6178`, matching A8 anchor “sum of n_i over 180 write-phase rows = 6178.” All PC-S, PC-P, PC-O, PC-L and F's PC-T are `ok=true`: `q1b=1` at block 7, `q1e=1`, `open_failed=true` (K also `opened_after_repair=yes`); PC-T now reports `write_kills_with_read_error=18` (16 in feasibility's 20 kills; first 20 plan rows use same seed, but cumulative `q1d`/`q1c` measurement basis differ). 18/18 matches V5 “Q1c+Q1d ≥ 1 among 18 write-phase kills”; all match registered pins and pass (checked with `grep -n 'name=positive_control' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`, not recopied).

Reopen time Q1h (trajectory, not used for flip judgment) differs markedly: F median 1100 µs, peak 2171 µs; R1 median 1015 µs, peak 1407 µs; R0 median 124,691 µs (~125 ms), peak 399,403 µs (~0.4 s); K median 11,411 µs, peak 3,271,087 µs (~3.27 s). K has `calibration_interfered=false`; its peak is not calibration interference but a single extreme main-cell reopen coinciding with another local session running `cargo test`, one trajectory high point not typical latency. R0 (redb default repair) is two orders of magnitude slower reopening than R1 (quick repair), matching redb 4.3.0 documentation's description that default crash recovery walks the entire database while quick repair is nearly immediate (registration 4.6). This is a trajectory observation, not an S1 criterion.

#### Verdict fields called out

In the four `grep -n 'name=verdict'` rows:

- No field is boolean `false`.
- No field is `not_run`.
- `pc_t_ok=not_applicable` (the three artifacts for R1, R0, and K): PC-T is defined only for F; for the other three arms it was supposed to be `not_applicable`, rather than not run; interpreted the same way as in the feasibility tier.
- `calibration_interfered=true` (F only; R1/R0/K false): F's S1 calibration hit V15's interference rerun limit (`INTERFERENCE_MAXIMUM_RERUNS=3`, all four attempts encountered another session's `cargo test`). Under registration 11.1 V15 report with interference marker after at most three reruns; S1 is not invalidated (V15 invalidates that timing only; S1 judges Q1a–Q1f, not calibration time).
- Integer counts indicating violations, mismatches, ambiguity or failures (`q1a`–`q1f`, `repaired_openings`) are all 0 for all arms.
- Other fields (`judgement=judged_by_registration`, `pc_l_ok`/`pc_s_ok`/`pc_p_ok`/`pc_o_ok=true`, `commit_waits_for_device=true`, `v6_kills_inside_commit_ok=true`, `a_s1_ok=true`, `s1_positive_quantities=none`) are expected passes.

Under 6.1 and 11.1 V1–V6, all positive controls pass (V1–V5 do not invalidate), all arms pass V6 (Q1g ≥ 100 of 180 write-phase kills, observed 178–180), and Q1a–Q1f are all 0 over 200 main-cell kills. **S1 is sufficient: in this SIGKILL model, redb (quick and default repair), RocksDB (sync write + WAL), and hardened files all had no confirmed block loss, silent corruption, phantom blocks or open failures.** Under section ten's pre-fixed “would I accept the opposite result” reading, F not flipping cannot establish that it is safer than databases (4.10 notes F's zero in these cells follows directly from definitions and kernel semantics, not F's own recovery code). State “no candidate flipped under this SIGKILL model,” not “equally safe.”

#### Rerun

Unit tests (rerun once in this segment to verify the build with the changed environment still passes):

```
cd scripts/experiment-apparatus && BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash ../hosts/capped.sh 4 bash ../hosts/run-with-memory-cap.sh 8G cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store
```

23 passed / 0 failed; the previously documented `librocksdb-sys` / `bindgen` blocker was worked around by `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include`.

These four decisive-tier artifacts do not enter `replay.sh` for byte-by-byte comparison, for the same reason as the feasibility tier: S1 launches a new subprocess every time and kills it according to real wall-clock time, reopening after real persistence latency on disk, so two runs are not the same batch of bytes (witnessed firsthand in this segment: F encountered four interference reruns this time, whereas the feasibility tier did not encounter a single one, with different bytes resulting from the same code and the same batch of seeds). Direct rerun command (`<臂>` takes one of F, R1, R0, K; omitting `feasibility` runs the decisive tier, completing the full 200 kills):

```
BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash scripts/hosts/capped.sh 4 bash scripts/hosts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂>
```

The registry for this segment was unchanged (before 2026-09-28 path migration), only `driver_e162_anchors`; replay output matches the feasibility section's copied output byte for byte:

```
bash scripts/experiment-apparatus/replay.sh E162
```

```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out
```

### What it cannot answer

- **S1 is sufficient for judgment (2026-09-27 second segment supplemental run), S2 remains unjudged**: all four S1 arms each completed the originally registered 200 main-cell kills, all Q1a–Q1f verdict rows returned 0, meeting the sufficiency condition, and the positive controls as well as the V6 threshold passed across all four arms. S2 only ran 20 kills per arm in the feasibility tier, and its 200 kills as well as Q2a–Q2b have not been judged.
- **S3 was not run at all**: sustained throughput of sending blocks over the network, consistency after network disconnection and reconnection, and cross-machine cells were explicitly specified by revision six of registration section 12 as not run in the feasibility tier, so S3 currently has zero data.
- **S1-large and geometry samples G-bs10, G-batch16, G-sparse, G-random unrun**: whether occupancy ratio, random reads and write rate reverse relative to main point is unknown.
- **The threshold for Q2a does not yet have a formal value**: R_AB = (P_A + P_B) ÷ t_state requires the formal t_state delivered by E161 (deduplication and segmentation timing of crash amplification); on the same day, E161 (deduplication and segmentation timing of crash amplification) also delivered only its feasibility tier (experiment page [161-崩溃放量的去重与分段耗时.md](161-崩溃放量的去重与分段耗时.md), g1–g5 unjudged). In that artifact of E161 (deduplication and segmentation timing of crash amplification), the row `name=g3 cell=first_small` reports `states=37 state_elapsed_ns=77898270` (the row in `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out`), which divided by state count yields roughly 2 105 359 ns/state (~2.11 ms/state)——**this is a rough number from a single sampling cell subject to interference (the `calibration_interfered` category fields in the same artifact and the body text of the E161 (deduplication and segmentation timing of crash amplification) experiment page note that compilation and other apparatuses were running continuously on the machine during the run), serving only as an order-of-magnitude reference, and is not substituted into any judgment of E162 (storage selection for crash verdict blocks)**; the formal t_state will only be available after E161 (deduplication and segmentation timing of crash amplification) runs across the full domain under the weighted closed form in registration section 6.0.
- **Occupancy ratio run once, `samples=10`**: whether R1/R0's ~2× after-close occupancy and K's peak 2.1495 then compaction to 1.0013 vary with write volume or reverse at G-bs10's small blocks / G-sparse's compressible values was not tested.
- **13 mutations originate in the earlier version before the source change** (earlier 2026-09-27; caught/invalid/not-failing counts for `scripts/mutation/tables/e162_crash_verdict_block_store.tsv` were run after that change; source unchanged in segment two, table not rerun, counts still from earlier that day).
- **Section twelve lacks “revision seven”**: S1 decisive tier was run in segment two without first amending registration; see the gap at the start of that section.
- **`librocksdb-sys` now compiled, but prior blocker's cause uninvestigated**: earlier page-writing unit-test rerun found `bindgen` could not find `stdbool.h` (only `libclang1-18` runtime, no `clang` executable, standard headers absent from resource-dir). In segment two `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` let tests and all four arms' `cargo build`/`cargo run` pass. It works around the blocker, but the actual environment change between attempts and why the header is missing without it were not investigated. Set it as in rerun commands before next source change.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 4 | Background | 2026-10-24 unaffected: apparatus verified by replay.sh, output byte-for-byte identical with retained artifact, conclusions and numbers unchanged; prior 2026-10-18 unaffected: rerun section identifies the artifact registered in replay.sh; conclusions and numbers unchanged |

## Historical Versions

### 2026-09-27: feasibility tier

- Page created. Under user settlement 2026-09-27 (section-twelve revision six), feasibility only: 20 S1 kills and 10⁴ S2 blocks per arm; S1-large, four geometry points and S3 unrun. Eight artifacts (`research/results/e162-crash-verdict-block-store-2026-09-27-{s1,s2}-{F,R1,R0,K}-feasibility.out`) plus anchors; `scripts/experiment-apparatus/replay.sh` registered anchors only (`driver_e162_anchors`).

### 2026-09-27: S1 decisive tier (segment two)

- In accordance with main agent dispatch (inheriting user settlement from 2026-09-27, see the “S1 Decisive Tier” section), S1 was expanded from the 20 kills of the feasibility tier to the originally registered 200 kills, running once for each of the four arms (F, R1, R0, K); judgment criteria (section 6.1, section 11.1 V1–V6) remain unchanged. S1 is now sufficient for judgment: across the 200 kills on the four arms, Q1a–Q1f are all 0, and both positive controls and the V6 threshold passed. S2 and S3 were not run in this segment. Four new artifacts (`research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`) were produced, neither overwriting nor deleting the eight old artifacts from the feasibility tier; `replay.sh` did not add new registrations (reason same as feasibility tier: S1 has different wall-clock kill points each time, so no bytewise comparison is done). The failure to add “revision seven” to the registration before running was an oversight of this segment's executor, handed over to the main agent for handling (see the beginning of the “S1 Decisive Tier” section). Before running, unit tests were rerun with 23 passed / 0 failed (`BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` resolved the `librocksdb-sys` build blocker documented in the prior version), without modifying source code and without rerunning the mutation table.

### 2026-09-28: path migration

- Repository-root git-ignored multi-host config renamed `layer0-shard.env` → `multi-host.env` under user settlement. In `scripts/experiment-apparatus/e162_crash_verdict_block_store.rs`, configuration filename read by `peer_logical_processors` and preceding comment changed. That function is called only from S2/geometry sensitivity `throughput_context`, not anchors. Conclusions, numbers and artifacts unchanged: 23 tests passed / 0 failed (same rerun command, `/tmp/claude-1000/e162-config-rename/unit-test.log`); new `research/results/e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out` matched pre-rename `e162-crash-verdict-block-store-2026-09-27-anchors.out` byte-for-byte via `cmp`. E162 anchors registration in `replay.sh` points to new artifact; old artifact not modified or deleted.
