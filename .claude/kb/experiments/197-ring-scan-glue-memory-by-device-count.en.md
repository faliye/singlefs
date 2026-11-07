## E197 Ring scan glue memory by device count — harness written, artifact not run

[中文](197-扫环胶水内存随盘数.md) · **English** · [日本語](197-リングスキャン接着部のメモリとディスク数.ja.md)

Pre-run registration `research/prompts/e197-preregistration.md` (the main agent cut the wording of three questions before the harness was written, see line 5 of the registration). It answers rows 甲 (upper bound) and 乙 (how to change) of the fork sheet `research/prompts/m3-close-c653-forks.md`. The clause under test is C653 (Ring scan glue memory grows with device count) (see the corresponding row in `.claude/kb/checks-owed.md`): once the ring-scan stretch is in Verus, the glue `scan_journal` first collects every self-verifying record from every device into one Vec and only then builds the table, so the peak grows with the device count.

The harness `crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs` is an in-repo harness: it calls the `scan_journal` in the implementation and the unchanged `first_copy_of_each_record_key` directly, and copies three pieces of glue into the harness as the 乙二 arm and two positive controls, without writing a separate model. It has 35 unit tests (counted with `grep -c '#\[test\]'`, including one child-process entry test that measures nothing when it is not running inside a child process). The driver and the report script are in `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/` (`drive.py`, `report.py`). The mutation rows are in `crates/mutations.tsv` (registration revision three removed the counting-allocator row, added two rows and changed the mutation of one). **This page only records that the harness is written; the artifact has not been run even once.**

### Current state

- The harness is in the main workspace. Compiling it and running its unit tests belong to the verification crate and so to the main agent. The harness calls the ring-scan kernel directly and `singlefs-core` does not re-export it, so `[dependencies]` in `crates/singlefs-verification/Cargo.toml` has `singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }`.
- `size_of` of `JournalRecord`: the registration's hand count in section seven (anchor 6) is 376, but counting the struct fields one by one gives 320 (four node pointers, not five). The harness unit test pins 320, and the resident-page formula (registration revision three, section four) substitutes 320 too; the value it actually reads is what counts, and a mismatch goes into a registration revision as it is, without changing the original registration text.
- In anchor 7 of section seven of the registration, the prediction for split_pc originally said "same as the baseline", and the harness added the capacities of two Vecs in the allocator measure (about 2.4% apart on 4 devices, about −3% for K = 64 and about −16% for K = 1 on 3 devices). Once registration revision three switches to the resident-page measure, the record Vec and the key table count only the pages that were touched, so split_pc has the same formula as the baseline and the capacity difference of those two Vecs no longer appears (inferred; the harness unit test pins the two formulas equal); split on 1 device equals the baseline.
- The self-test of the driver has run green (10 items). After the harness and the report script were changed to registration revision three (counting allocator removed, the in-process measure is now the resident-page peak H), the harness unit tests and the self-test and breakage switches of `report.py` have not been run yet; they go to the main agent.
- The artifact has not been run even once; rows 甲 and 乙 of the fork sheet are both still open and there is not a single verdict line. In the path registration, the line numbers come from the source in the main workspace.

### Rerun

```bash
# Repository root. The in-repo harness is in the verification crate; compiling it and running its unit tests belong to the main agent; the binary is built under compile-then-swap/target/release/ in the scratch directory.
task-server/task submit -- python3 scripts/hosts/compile-then-swap.py <scratch copy> crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs --scratch <scratch directory> --release
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e197_scan_journal_glue_memory_by_device_count 的单测; 范围=1 个测试目标; 为什么=核 E197 装置的 35 条单测' task-server/task submit -- cargo test --release -p singlefs-verification --bin e197_scan_journal_glue_memory_by_device_count
# Artifact (after the unit tests are green): the stop clause and the differential set first (checks), then the key-by-key comparison (compare), then the segments (k64-full, k64-half, k1-full); each cell is one service attempt (one shard), and the memory cap and the host are set by the service
BINARY=<scratch directory>/compile-then-swap/target/release/e197_scan_journal_glue_memory_by_device_count
task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py checks --binary $BINARY --out research/results/e197-<date>-checks.out
task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py compare --binary $BINARY --out research/results/e197-<date>-compare.out
# Print the full task-server/task submit commands to hand in, one per line (nothing is submitted or run): batch 0 is checks and compare, batches 1–10 are 6 segment tasks each (the three segments k64-full, k64-half, k1-full; repeat numbers 1–5 for each segment; four groups for each repeat number); submit them one by one, as printed
python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py list-submissions --binary $BINARY --date <date>
# After each task has an outcome, the main agent saves the service status directly: task-server/task status <parent task number> --json > research/results/e197-<date>-status-<parent task number>.json; build the table once all are saved (the verdict file is created exclusively and refused if it exists)
task-server/task submit -- python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py --artifacts research/results/e197-<date>-checks.out research/results/e197-<date>-compare.out research/results/e197-<date>-k*-r*-*.out --status research/results/e197-<date>-status-*.json --out research/results/e197-<date>-verdict.out
```

### Measurement basis

- The number judged is the memory peak that the unified task service records for each attempt (the service takes the larger of the value sampled per round and the one the memory wrapper reads from its own cgroup's `memory.peak` when it exits, including page cache and the outer shell itself, so it errs high and never low; the driver applies no wrapper, the main agent saves the output of `task-server/task status <parent task number> --json` after each task has an outcome, and report.py reads that archive): the median of 5 runs of a cell, minus the median of the empty cell of the same geometry. The in-process resident-page peak H (at the start of the measuring window write 5 to `/proc/self/clear_refs` and read `VmRSS` as the starting point; after the return read `VmHWM`; H is the difference) is only used to cross-check, not judged.
- Geometry: the default ring of 196 608 record slots per device, a 4 GiB device, 1–4 devices (3 devices is the opposite-direction sample point), 64 and 1 named entries per record, record slots full and half full; the key of a cell is (arm, devices, fill, named entries, repeat number), and the peak is the `memory_peak` that the service recorded for that attempt (the result line carries `task=` and `attempt=`, and report.py matches it against the status archive).
- How H is measured: at the start of the measuring window (after the pool is built and the probe record is encoded, before the arm is called) write 5 to `/proc/self/clear_refs`, which resets the process's `VmHWM` to the current resident size, and read `VmRSS` as the starting point; after the return and before the returned value is released (for the two arms that read through the cache, before the cache is released) read `VmHWM`; H = (`VmHWM` − starting point) × 1024 bytes. A run where the write fails or any reading cannot be read is voided (V11) and never counted as 0. The trajectory and the end value read `VmRSS` minus the starting point. The harness has no counting allocator and no `unsafe`; H is a number for the whole process, so every unit test that reads H measures each cell in a child process of its own.
- The cells that measure peaks do not run `scan_journal` once more on the tables the arms return (that would double the peak); they only print a 64-bit digest (the CRC-32C of each record's 4096-byte encoding, folded in key order), compared with the digest of the baseline arm's cell of the same geometry when the report is made. Equality key by key and value by value is judged by dedicated cells that do not measure peaks: the 36 `--compare` cells (three glue arms × 1–4 devices × three geometries; each cell runs the arm first and then `scan_journal` and compares with `==`) and the four differential cases (also compared with `==` on a small ring).

### Fork table

| Fork | Status | What is missing / can the remaining measurements flip it |
|---|---|---|
| 甲 upper bound | Not judged | The artifact has not been run even once; verdict lines exist only after the registered measurements are done (K = 64 full and half full, K = 1 as the opposite-direction sample point, 1–4 devices) |
| 乙 how to change | Not judged | Same; whether the 乙二 arm is doable is judged by the differential set D and the digests of the four mirrored-image cells, and the proof-obligation change for 乙一 is not in this harness |

### What it cannot answer

- The two-pass reading and writing of 乙二 (first collect only the keys and call the kernel, then read again and parse only the first copies) is not in the definition of 乙二 in the fork sheet; the registration lists it as an item to be acknowledged and does not measure it.
- When a writable mount scans the ring, `ReadStageCache` keeps the whole of the record bytes it read. That stretch is not inside `scan_journal` and the fork sheet has no row for it, so it is not measured.
- 乙一 (change the kernel to take one device at a time) needs a changed Verus kernel and proof, which is not a memory measurement and not measured by this harness; if 乙二 is judged not doable the main agent dispatches it separately.
- Only 1–4 devices are measured. If the registered device-count limit is above 4, the byte count of 甲二 is an extrapolation; `crates/` has no pool device-count limit, and the main agent decides what it is.

### Paths and conclusions registry

The conclusion "how the peak of `scan_journal` grows with the device count": the same quantity is computed along three paths and compared (the in-process resident peak H, the cgroup, and the resident-page formula).

| # | Path | How it is computed | Source placement | Shared items read |
|---|---|---|---|---|
| 1 | In-process resident peak H | At the start of the measuring window write 5 to `/proc/self/clear_refs` to reset `VmHWM` and read `VmRSS` as the starting point; after the return (for the two arms that read through the cache, before the cache is released) read `VmHWM`; H = the difference × 1024; 5 runs per cell, the three geometry levels of the default ring, no other device counts | `crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:156 read_from` and `crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:131 open_with` | Record size 4096, ring slot count 196 608, `size_of` (pinned by unit tests); shares with path 2 the measure "pages that were touched and are still resident" (H counts the high-water mark of the process's resident pages, path 2 counts the high-water mark of the pages charged to the scope), so a fault that leaves pages untouched or uncounted would make both wrong together, and only the formula of path 3 can see it |
| 2 | cgroup peak | The `memory_peak` that the service records for each attempt (the larger of the value sampled per round and the cgroup `memory.peak` read when the memory wrapper exits); the main agent saves `task-server/task status <parent task number> --json`, the driver and report.py read that archive, and the empty cell is subtracted | `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py:265 pinned_reading` | Shares no code with path 1; the only shared items are "one process per cell, the same binary" |
| 3 | By resident-page formula | The resident-page formula R of `resident_prediction.py` in section four of registration revision three, with the measured `size_of` substituted; one copy in the report script and one in the harness, both pinned to the design numbers of revision three | `crates/singlefs-verification/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:344 predicted_resident_peak_bytes` and `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py:129 predicted_resident_peak` | Record size, size of a named entry, the block header and 16-byte alignment of glibc, 6 keys per B-tree leaf (the split point of ascending insertion in the standard library, inferred); if this were wrong, the ±5% of path 1 against the prediction would be wrong with it, which is why path 2 cross-checks it (the `S3_two_measures` row of the report script); paths 1 and 2 both count only touched pages, so when they are wrong together only this formula can see it |

Cross-path assertions: path 1 against path 3 is pinned in the harness unit test `the_resident_peak_of_every_scanning_arm_stays_within_the_prediction`; path 1 against path 2 is the `S3_two_measures` row of the report script (it judges H against P), and there is not yet a real reading of it before the artifact is run, so that one is still owed.

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D13 (Verification Route) settled item 1 | Preparation | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-04 unaffected: registration revision three changes the in-process measure from the counting allocator to the resident-page peak H, the harness, the report script and the source locations in the registry follow, and the judged P, the conclusions and the artifacts are unchanged (no cell of the artifacts has entered a verdict); previously, 2026-11-03 unaffected: only the wording of the rerun commands was changed and the source locations in the registry table were updated to match the current source; conclusions, numbers and artifacts unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-22 not affected: the "owed" paragraph of that item carries C653 (Ring scan glue memory grows with device count), the harness is written and the artifact is not run, and the basis paragraph does not cite E197 (Ring scan glue memory by device count), so this cell only records preparation; whether it should be promoted to support once the artifact is run goes to the main agent |
| D23 (Journal Role & Format) settled item 19 | Preparation | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-04 unaffected: registration revision three only changes the in-process measure; the default ring of 768 MiB and "ring ≤ device capacity ÷ 4" are still used only as a source of geometry; previously, 2026-11-03 unaffected: only the wording of the rerun commands was changed and the source locations in the registry table were updated to match the current source; conclusions, numbers and artifacts unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-22 not affected: the default ring of 768 MiB and "ring ≤ device capacity ÷ 4" are used only as a source of geometry, and the local constants in the harness are copied from it and checked back against `crates/` |
| D23 (Journal Role & Format) settled item 12 | Preparation | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-04 unaffected: registration revision three only changes the in-process measure; the 4 KiB record size and 64 named entries per record are still used only as a source of geometry; previously, 2026-11-03 unaffected: only the wording of the rerun commands was changed and the source locations in the registry table were updated to match the current source; conclusions, numbers and artifacts unchanged; previously, 2026-11-01 unaffected: only the rerun launch commands were changed; previously, 2026-10-22 not affected: the 4 KiB record size and 64 named entries per record are used only as a source of geometry, again with local constants and a check-back |

## Historical Versions

### 2026-11-04

- The harness no longer has a counting allocator (the user decided on 2026-11-03 that "E197 no longer uses the counting allocator"; registration revision three): the in-process measure becomes the resident-page peak H (write 5 to `/proc/self/clear_refs` to reset, read `VmRSS` as the starting point, read `VmHWM` after the return); the prediction of A7, the trajectory and the end value, the two measures of S3, the report script and the mutation rows follow; the judged P, the thresholds and the artifacts are unchanged. The peak readings of the 2026-11-02 batch of artifacts were judged unusable according to the dispatch record (line 2322 of the dispatch record that starts with 2026-10-18 under `records/2026-10-18-*.md`), and the whole batch has to be rerun.

### 2026-11-02

- Produced first batch of artifacts (checks, compare, and part of k64-full shards and status files for rounds 1 and 2):
  - `research/results/e197-2026-11-02-checks.out`
  - `research/results/e197-2026-11-02-compare.out`
  - `research/results/e197-2026-11-02-k64-full-r1-glue-1.out`
  - `research/results/e197-2026-11-02-k64-full-r1-glue-2.out`
  - `research/results/e197-2026-11-02-k64-full-r1-glue-3.out`
  - `research/results/e197-2026-11-02-k64-full-r1-read-stage-cache.out`
  - `research/results/e197-2026-11-02-k64-full-r2-glue-1.out`
  - `research/results/e197-2026-11-02-k64-full-r2-glue-2.out`
  - `research/results/e197-2026-11-02-status-5012.json`
  - `research/results/e197-2026-11-02-status-5014.json`
  - `research/results/e197-2026-11-02-status-5016.json`
  - `research/results/e197-2026-11-02-status-5018.json`
  - `research/results/e197-2026-11-02-status-5020.json`
  - `research/results/e197-2026-11-02-status-5022.json`

### 2026-10-22

- Created page, registering the E197 (Ring scan glue memory by device count) in-repo harness, driver and report scripts, three measurement paths, and decision review.
