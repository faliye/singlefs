## E159 Fsync Latency vs Concurrency: Group Commit and WAL Arms — Rig Written, Timing Not Run (2026-09-24, continuation session; the pure counting part is deterministic, 55 unit tests / all 17 mutations caught; the five rounds of the formal-timing main grid, PC1–PC3, AA, G8, G1, G6 have not been run; the rig's `RECORD_HEADER_BYTES` is still 311 and the named-item pin is 67, and it was not re-run after the header became 496 and the named-item pin 64; owed check C776 (E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds), E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) apparatus stopped at header 311 and named-item pin 67))

[中文](159-fsync等待时间随并发数组提交与wal两臂.md) · **English** · [日本語](159-並行数に伴うfsync待機時間-グループコミットとWALの2アーム.ja.md)

<!-- doc-lint:registry name-col=2 -->

| ID | Short name |
|---|---|
| J1 | Arm A crosses M at some k |
| J4 | p99 crosses M at some k |

**Question**: Pre-registered in `e159-preregistration.md` — fork sheet `m2-fsync-wait-forks.md`, line 1: "Is the wait time of arm A (today's approach: each fsync writes the dirty leaf plus all ancestors, batched and group-committed per D16 (Publish Semantics) settled item 10) acceptable?" We measure "how long one fsync waits from issue to return" and how it varies with concurrency k on arm A and the two WAL arms (wal_full-K10 and B-M-K10; both are kept, each judged on its own). The rig is at `scripts/experiment-apparatus/e159_fsync_wait_group_commit.rs` (an independent hand-written model, not a checked-in rig — `crates/` has no group commit; Section 3, item 2.3, searched just now: zero hits). Latency is measured with real runs (real machine, real O_DIRECT device, real threads) rather than converted from a counting model; the reasons are in the pre-registration "Section 5, 5.1".

**This segment (continuing an executor that was interrupted by hitting the session quota) only got as far as "rig and smoke test"**: the formal timing (5 rounds of the main grid under this machine's measurement basis, PC1–PC3, AA, G8, G1, G6) was not run — the machine currently has concurrent compilation, which would contaminate the timing (the same kind of concern as `.claude/kb/tooling.md`, "performance numbers in the repo hold only on this machine"); it will be dispatched separately when the machine is idle.

### What this segment did

1. **Rig**: copied verbatim the constants, K1–K4, the 5.3 group formulas and the single-shot/batch functions of the three arms from lines 36–2058 of `scripts/experiment-apparatus/e155_fourth_run_group_commit_concurrency.rs` (pre-registration stop clause S2 requires this block to have an empty diff against the source file, item by item; checked, see "Re-run"); newly wrote the group-commit queue (`SharedQueue`, `drain_batch_from_queue`), the WAL checkpoint thread (`checkpoint_loop`, `should_trigger_checkpoint`), the closed-loop caller (`caller_loop`), O_DIRECT I/O (`RealBlockDevice`), the fake devices used by structural unit tests (`RecordingBlockDevice`, `SleepingBlockDevice`), statistics (quantiles, Mann–Whitney A, rule R, verdicts for J1 (arm A crosses M at some k)/J2a/J4 (p99 crosses M at some k)), PC1 busy-wait injection, and the three subcommands `cell`/`probe-device`/`anchors`.
2. **Stop clause S1 (item-by-item differential of the anchor cell against `crates/`)**: unit tests pin arm A at F1, P=1, k=1 to a single-device total of 172544 bytes and 11 write calls (`jia_single_fsync_batch_matches_crates_test_table`, consistent item by item with the sums of the `crates/` test table in Section 3); the publish order unit → barrier → record → barrier → root slot FUA → system configuration slot (read twice first, then written once) is checked step by step (`jia_single_fsync_batch_step_sequence_matches_crates_order`), and the root-slot durable instant is no earlier than the root-slot FUA (`jia_durable_instant_is_not_earlier_than_the_root_slot_fua`). S1 does not trigger.
3. **Stop clause S2 (copied block diff is empty)**: `diff <(sed -n '39,2061p' e159_...) <(sed -n '36,2058p' e155_...)` exits with code 0.
4. **Stop clause S3 (guest `logical_block_size`)**: the real-machine VM probe subcommand `probe-device` reads back `logical_block_size=512`, consistent with the main geometry constant `physical_block_size_bytes`; does not trigger.
5. **Unit tests and mutations**: all 54 unit tests green (`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit`); the mutation table `scripts/mutation/tables/e159_fsync_wait_group_commit.tsv` has 14 entries (M1–M14, covering all targets of Section 9: arm A publish with the second barrier removed, release right after the first barrier, WAL arms mixing up the planning function, group commit on but batch capped at 1, `t_issue` missing the queueing segment, PC1 busy-wait moved to after release, random rounding changed to floor rounding, `T_time` constant misread, system configuration slot not read before write, root-slot width changed wrongly, J1 (arm A crosses M at some k) boundary comparison operator changed wrongly, five-round merge changed to majority-of-rounds decides, open-loop `issued_at` changed to the actual issue instant, `release_to_wake` missing the wake-up segment); `mutate.sh` was run twice (once before and once after the source change), both times 14/14 all caught, with the closing line "restored, baseline still all green". M3–M6 (the four that depend on the group-commit queue and real-concurrency timing) originally had no unit test covering the real-concurrency engine; this segment adds 6 unit tests to fill the gap: `plan_batch_for_arm_routes_each_arm_to_its_own_solver_not_a_different_one` (M3), `drain_batch_from_queue_collects_everything_when_group_commit_is_on` / `_collects_only_one_when_group_commit_is_off` (M4), `closed_loop_queueing_delay_is_counted_when_group_commit_is_off` (M5, uses `SleepingBlockDevice` with a fixed barrier delay to force queueing, no real disk needed), `pc1_injected_wait_lands_before_release_not_after` (M6, also using `SleepingBlockDevice`), `checkpoint_time_threshold_constant_is_five_seconds` (M8, extracts the `T_time` constant into the named constant `CHECKPOINT_TIME_THRESHOLD`, so `run_cell_subcommand` and the unit test share the same placement instead of each writing its own literal).
6. **The real-machine smoke test ran into a real bug, now fixed**: the original `RealBlockDevice::read_at` read data into an ordinary `vec![0u8; length]` (unaligned) and then passed it to the O_DIRECT `read_exact_at`; the first real run (`--arm jia --k 1024`) immediately `panicked ... EINVAL`; before the 2026-09-24 fix, this path was covered by 0 unit tests (all old unit tests only used `RecordingBlockDevice`/`SleepingBlockDevice`). The fix: read data directly into an `AlignedBuffer` aligned to `ALIGNMENT_BYTES` (new `AlignedBuffer::as_mut`), then copy it out as the return value after the read; after the fix, the same command with k=4 no longer panics. See "Result lines copied whole from artifacts" for the artifact.
7. **Real-machine smoke test: one cell on each of the three arms ran through** (`VM_DISK_PREALLOCATE=1`, short duration, only to verify the rig runs for real without crashing, not to produce formal numbers): arm A k=1, wal_full-K10 k=2 (duration 4s, did not reach `T_time`=5s or `T_dirty`=256MiB, checkpoint_count=0, as expected), B-M-K10 k=2 (duration 6s, triggered one checkpoint; the checkpoint mechanism really ran through). All three arms, the group-commit engine and the WAL checkpoint thread were verified to run end to end, with no new panic.
8. **`print_anchors` refactor**: changed to emit `E7RESULT` lines through `Emitter` and to end with `name=done emitted=N` (before the 2026-09-24 refactor it used bare `println!`, which does not satisfy the completeness gate of `scripts/experiment-apparatus/replay.sh`), and changed the single-device byte computation for arm A's line from `outcome.total_bytes() / DEVICE_COUNT` (a wrong approximation for the root-slot field, which would also halve the single-device-only 512-byte root slot) to `jia_outcome_single_device_bytes`, shared with the unit tests (moved to a module-top-level function); the output goes from 172288 back to the correct 172544, matching the B4 anchor.
9. **Naming discipline**: `naming-lint.sh` passes on this file throughout (the only hit is the `O_DIRECT` external flag name annotated with `naming-lint:external`, which is not a violation).

### Result lines copied whole from artifacts (lines whose artifacts have been moved out of the workspace are written as a single pointer line)

`e159-fsync-wait-group-commit-2026-09-24-anchors.out` (pure arithmetic, byte-for-byte re-runnable, see "Re-run"):

```text
E7RESULT name=anchor label=jia_p1_f1_k1_single_device_bytes value=172544
E7RESULT name=anchor label=write_ahead_log_full_k10_p1_f1_k1_single_device_bytes value=102400
E7RESULT name=anchor label=yi_m_k10_p1_f1_k1_single_device_bytes value=86016
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k1_single_device_expected_bytes value=365346.000
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k16_single_device_expected_bytes value=4313362.000
E7RESULT name=done emitted=6
```

`e159-fsync-wait-group-commit-2026-09-24-smoke.out` (real-machine smoke evidence, not part of the byte-by-byte comparison of `replay.sh` — real latency differs on every run — it only proves the rig runs for real; see that file for full context): the `E159RESULT` lines of one cell on each of the three arms after the fix, for example the B-M-K10 cell:

```text
产物行见 git show 5d0b7e7c1667:research/results/e159-fsync-wait-group-commit-2026-09-24-smoke.out 第 175 行（E7RESULT E159RESULT arm=yi-M-K10）
```

`containment_violations=0` (the V5 containment self-check was not violated), `checkpoint_count=1` (the WAL checkpoint mechanism really triggered once).

### Re-run

```
bash scripts/experiment-apparatus/replay.sh E159
```

Only the `anchors` subcommand line is registered (pure arithmetic, touches no real device, byte-for-byte re-runnable): the `anchors` line of E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) in `scripts/experiment-apparatus/replay.sh`, artifact `e159-fsync-wait-group-commit-2026-09-24-anchors.out`, judging method `exact`, run before handing back, verdict "byte-identical". The real-machine smoke test (S3 probe and one cell on each of three arms) is not registered in `replay.sh` — latency numbers differ on every run and there is no byte-level criterion to pin; the re-run commands are stored verbatim in the header of the file `e159-fsync-wait-group-commit-2026-09-24-smoke.out`, and can be run manually when a re-run is needed, for example:

```
cargo build --release --target x86_64-unknown-linux-musl --bin e159-fsync-wait-group-commit -p e7-index-bench
bash scripts/devices/vm-bench.sh <musl 二进制路径> probe-device --device /dev/vda
VM_DISK_PREALLOCATE=1 bash scripts/devices/vm-bench.sh <musl 二进制路径> cell --device /dev/vda \
  --arm yi-m --k 2 --group-commit on --warmup-seconds 0.3 --duration-seconds 6 --seed 5 \
  --file-count 10000 --family f8a
```

Unit tests: `cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit` → `55 passed; 0 failed` (2026-09-24 header 307 → 311 re-run added 1). Mutations: `cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e159-fsync-wait-group-commit e159_fsync_wait_group_commit.rs ../mutations/e159_fsync_wait_group_commit.tsv` → 17/17 all caught (M15/M16/M17 added), closing line "restored, baseline still all green" (run twice, once before and once after the source change, same conclusion).

### What it cannot answer

- **The flipped observation on line 1 of the fork sheet still has no answer**: the formal timing (5 rounds of the main grid × 3 arms × 8 values of k, PC1–PC3, AA, G8, G1, G6) has not run a single cell, so the three criteria J1 (arm A crosses M at some k)/J2a/J4 (p99 crosses M at some k) have no values yet. This segment only proves that the rig can really run through, that the byte measurement basis matches `crates/`, and that the mutation table catches everything.
- **The S4 write-volume budget (pre-registration 5.7) has not been worked out**: there are only scattered smoke readings (for example arm A k=4, duration 3s wrote about 58.5 MB, roughly 17.7 MB/s, but this was measured on a sparse disk with sectors touched for the first time, without the "write the whole disk once before measuring" step, so the throughput after real warm-up may be noticeably higher and the real byte total may therefore be underestimated by this number); the precise budget total was not accumulated cell by cell over all cells of the first section (24 main-grid cells + PC1/PC2/PC3/AA/anchors + 24 G8 cells + 12 G1 cells + 8 G6 cells, each 11s or 2s, 5 rounds), so it is also not confirmed whether the 1 TiB stop line would trigger. Before the next segment runs the formal timing, this accounting must be worked out first, or a representative cell must be measured on a warmed-up disk and extrapolated.
- **The "write the whole disk once before measuring" step (pre-registration 5.3) is not yet implemented in the rig**: this segment's smoke test only used `VM_DISK_PREALLOCATE=1` (`fallocate` preallocation, not a full-disk write), so the first smoke test without this parameter hit samples=0 (not a bug, but the host ext4 metadata overhead of touching sectors of a sparse disk for the first time); after fixing read_at, the "write the whole disk once" step itself has not been verified.
- **The three opposite-direction sampling points, G8 open-loop Poisson arrival, G1 no sharing, and G6 `T_time=200ms`, have not been run**, so the discriminative power self-test of Section 8's "oppositely-directed sampling points" has not been done either.
- **The four positive controls and noise floor, PC1/PC2/PC3/AA, have not been formally run** (there is only the informal smoke test: the B-M-K10 cell triggered one real checkpoint, which shows the checkpoint mechanism works, and is not a formal PC-series verdict).

### Decisions affected

This segment delivers only the rig and the smoke test, with no timing conclusion; the "basis" paragraph of every sub-item in the "Decisions affected" table does not cite E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) (`grep -n` over each decision file, zero hits), so per the format-evolution discipline all are recorded as "preparation", to be revisited after the formal timing finishes to see whether to upgrade them to support or refutation.

| Decision sub-item | Relation | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 1 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |
| D16 (Publish Semantics) settled item 2 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |
| D16 (Publish Semantics) settled item 5 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |
| D16 (Publish Semantics) settled item 7 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |
| D16 (Publish Semantics) settled item 10 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |
| D16 (Publish Semantics) settled item 11 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |
| D25 (Target Workload Priority) settled item 7 | Preparation | 2026-10-15 unaffected: the artifact lines copied whole were replaced by a pointer to the artifact; numbers, conclusions and artifact unchanged |

## Historical Versions

### 2026-09-24: Record header 307 → 311 re-run (anchors and smoke test, question sheet line 5)

- Before: the rig constant `RECORD_HEADER_BYTES` was still 307, while `crates/singlefs-format/src/lib.rs` was already 311. `H` enters the rig through only one channel (`NAMED_ITEMS_PER_RECORD_MAIN`), and it affects only the record-count path; the `anchors` artifact (write-volume anchors) is entirely unrelated to the timing conclusion on wait time — this time the timing segment is not run.
- After: once `e159-r1-prereg.md` had fixed the criteria, the constant was changed to 311, and one new unit test pinning remainder 33 was added (written as an addition, for the same reason as the fifth-run registration of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds)); the `anchors` artifact is byte-identical (also byte-identical to the existing `e159-fsync-wait-group-commit-2026-09-24-anchors.out`, no new file used). The byte-by-byte diff of the four write-volume functions shared with `e155_fourth_run_group_commit_concurrency.rs` (`solve_fixed_point_core`, `solve_jia_batch_publish`, `solve_write_ahead_log_batch_fsync`, `solve_write_ahead_log_batch_checkpoint`) is empty; S2 did not trigger. Real-machine smoke re-run: the S3 probe still reports `logical_block_size=512`; one cell each on arm A/wal_full-K10/B-M-K10, all exit codes 0, result lines complete, `emitted=` matches (`samples=0` — no fsync in the measurement window landed after warmup, unrelated to timing, this number is not judged), new evidence `e159-fsync-wait-group-commit-2026-09-24-h311-smoke.out`. 55 unit tests all green (1 added), 17 mutations all caught (M15/M16/M17 added, run once before and once after the constant change, 0 ineffective, 0 not red). **The scope of line 5 of the question sheet this time (the write-volume half and the smoke test) is recorded as "unchanged"**; the wait-time half is still unmeasured, left for the timing segment (dispatched separately when the machine is idle).
- Evidence: `e159-r1-prereg.md`; `m2-header311-rerun-questions.md` line 5; D23 (Journal Role & Format) settled item 4.

### 2026-09-25 (supplementing the artifact for gate 69)

- On 2026-09-25, for gate 69, the artifact was supplemented, byte-identical: the `anchors` artifact of the previous re-run was judged "byte-identical to the existing artifact" but no new artifact was saved separately; the mtime of `e159-fsync-wait-group-commit-2026-09-24-anchors.out` is earlier than the instant the rig source was changed to 311, so gate 69 went red. Today the registered line of this experiment in `replay.sh` was re-run, and it is byte-identical to the artifact the registered line pointed to at that time; saved separately as `e159-fsync-wait-group-commit-2026-09-25-h311-replay.out` (byte-identical to the old artifact, which is kept and not deleted), and the registration of that line in `replay.sh` now points to the new file; the rig and conclusions are unchanged.
