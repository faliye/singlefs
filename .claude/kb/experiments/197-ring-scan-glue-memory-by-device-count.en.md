## E197 Ring scan glue memory by device count — harness written, artifact not run

[中文](197-扫环胶水内存随盘数.md) · **English** · [日本語](197-リングスキャン接着部のメモリとディスク数.ja.md)

Pre-run registration `research/prompts/e197-preregistration.md` (the main agent cut the wording of three questions before the harness was written, see line 5 of the registration). It answers rows 甲 (upper bound) and 乙 (how to change) of the fork sheet `research/prompts/m3-close-c653-forks.md`. The clause under test is C653 (Ring scan glue memory grows with device count) (see the corresponding row in `.claude/kb/checks-owed.md`): once the ring-scan stretch is in Verus, the glue `scan_journal` first collects every self-verifying record from every device into one Vec and only then builds the table, so the peak grows with the device count.

The harness `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs` is an in-repo harness: it calls the `scan_journal` in the implementation and the unchanged `first_copy_of_each_record_key` directly, and copies three pieces of glue into the harness as the 乙二 arm and two positive controls, without writing a separate model. It has 24 unit tests. The driver and the report script are in `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/` (`drive.py`, `report.py`). The 13 mutation rows are prepared in the executor's scratch directory and are appended to the end of `crates/mutations.tsv` once the harness is in the main workspace. **This page only records that the harness is written; the artifact has not been run even once.**

### Current state

- The harness sits in a scratch copy. Compiling it and running its unit tests belong to the checker tier (heavy tier) and so to the main agent. Before it goes into the main workspace, `[dependencies]` in `crates/singlefs-checker-tier/Cargo.toml` needs `singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }`: the harness has to call the ring-scan kernel directly, and `singlefs-core` does not re-export it. That line is outside the executor's write scope, so the main agent adds it.
- `size_of` of `JournalRecord`: the registration's hand count in section seven (anchor 6) is 376, but counting the struct fields one by one gives 320 (four node pointers, not five). The harness unit test pins 320; the value it actually reads is what counts, and a mismatch goes into a registration revision as it is, without changing the original registration text.
- In anchor 7 of section seven of the registration, the prediction for split_pc says "same as the baseline" and leaves out the duplicate-record Vec kept until the return. The harness adds the capacities of two Vecs according to its own code (the duplicate-record Vec and the Vec of the last device's pass, in place of the baseline's single big Vec). The two formulas agree on 1 and 2 devices, differ by about 2.4% on 4 devices, and on 3 devices by about −3% for K = 64 and about −16% for K = 1. It goes into a registration revision once the unit test readings are in.
- The self-tests of the driver and the report script have run green: 10 items for `drive.py`, 18 items for `report.py`; 5 and 6 breakage switches each go red on the item they name (the original output is in the executor's report).
- The artifact has not been run even once; rows 甲 and 乙 of the fork sheet are both still open and there is not a single verdict line. In the path registration, the line numbers of the harness come from the scratch copy and are recounted once it is in the main workspace.

### Rerun

```bash
# Repository root. Compiling the in-repo harness and running its unit tests belong to the main agent (checker tier, heavy tier); the binary is built under compile-then-swap/target/release/ in the scratch directory.
bash scripts/hosts/capped.sh 31 python3 scripts/hosts/compile-then-swap.py <scratch copy> crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs --scratch <scratch directory> --release
SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e197_scan_journal_glue_memory_by_device_count 的单测; 范围=1 个测试目标; 为什么=核 E197 装置的 24 条单测' bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-checker-tier --bin e197_scan_journal_glue_memory_by_device_count
# Artifact (after the unit tests are green): the stop clause and the differential set first (checks), then the key-by-key comparison (compare), then one command per segment (k64-full, k64-half, k1-full); one thread per cell, 5 cells at a time, 8G cap per cell
BINARY=<scratch directory>/compile-then-swap/target/release/e197_scan_journal_glue_memory_by_device_count
python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py checks --binary $BINARY --out research/results/e197-scan-journal-glue-memory-by-device-count-<date>.out
python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py segment compare --binary $BINARY --out research/results/e197-scan-journal-glue-memory-by-device-count-<date>.out --peaks <peak table.tsv> --jobs 5
python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py segment k64-full --binary $BINARY --out research/results/e197-scan-journal-glue-memory-by-device-count-<date>.out --peaks <peak table.tsv> --jobs 5
python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py research/results/e197-scan-journal-glue-memory-by-device-count-<date>.out --append
```

### Measurement basis

- The number judged is the cgroup peak measured by the memory wrapper (the outer bash in the scope reads its own cgroup's `memory.peak` after the command exits, rounded up to MiB, including page cache and the outer shell itself, so it errs high and never low): the median of 5 runs of a cell, minus the median of the empty cell of the same geometry. The in-process allocator peak and `VmHWM` are only used to cross-check, not judged.
- Geometry: the default ring of 196 608 record slots per device, a 4 GiB device, 1–4 devices (3 devices is the opposite-direction sample point), 64 and 1 named entries per record, record slots full and half full; the peak-table key is `e197/<arm>/<devices>/<fill>/<named entries>/<repeat number>`.
- The counting allocator counts by the bytes of the `Layout` and counts a `realloc` by the difference between new and old size; the counts are kept per thread (the ring scan runs on the main thread, and each unit test measures on its own thread).
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

The conclusion "how the peak of `scan_journal` grows with the device count": the same quantity is computed along three paths and compared (the in-process allocator, the cgroup, and a formula).

| # | Path | How it is computed | Source placement | Shared items read |
|---|---|---|---|---|
| 1 | Allocator peak | The counting allocator records in-use bytes and the peak by the bytes of the `Layout`; before the ring scan the peak is reset to the current in-use bytes, and after the return the peak minus the start is read; 5 runs per cell, the three geometry levels of the default ring, no other device counts | `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:133 CountingAllocator` | Record size 4096, ring slot count 196 608, `size_of` (pinned by unit tests); if it were wrong, the other two paths would not be wrong with it |
| 2 | cgroup peak | The memory wrapper reads the `memory.peak` of its own cgroup in the scope; the driver reads it from the peak table and subtracts the empty cell | `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py:118 cgroup_peak_bytes` | Shares no code with path 1; the only shared items are "one process per cell, the same binary" |
| 3 | By formula | The formula of `predict.py` in section four of the registration, with the measured `size_of` substituted; one copy in the report script and one in the harness, both pinned to the design numbers of the registration | `crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs:322 predicted_allocator_peak_bytes` and `scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py:91 predicted_allocator_peak` | Record size, size of a named entry, 6 keys per B-tree leaf (the split point of ascending insertion in the standard library, inferred); if this were wrong, the ±5% of path 1 against the prediction would be wrong with it, which is why path 2 cross-checks it (the `S3_two_measures` row of the report script) |

Cross-path assertions: path 1 against path 3 is pinned in the harness unit test `the_allocator_peak_of_every_scanning_arm_stays_within_the_prediction`; path 1 against path 2 is the `S3_two_measures` row of the report script, and there is not yet a real reading of it before the artifact is run, so that one is still owed.

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D13 (Verification Route) settled item 1 | Preparation | 2026-10-22 not affected: the "owed" paragraph of that item carries C653 (Ring scan glue memory grows with device count), the harness is written and the artifact is not run, and the basis paragraph does not cite E197 (Ring scan glue memory by device count), so this cell only records preparation; whether it should be promoted to support once the artifact is run goes to the main agent |
| D23 (Journal Role & Format) settled item 19 | Preparation | 2026-10-22 not affected: the default ring of 768 MiB and "ring ≤ device capacity ÷ 4" are used only as a source of geometry, and the local constants in the harness are copied from it and checked back against `crates/` |
| D23 (Journal Role & Format) settled item 12 | Preparation | 2026-10-22 not affected: the 4 KiB record size and 64 named entries per record are used only as a source of geometry, again with local constants and a check-back |

## Historical Versions

### 2026-10-22

- Created page, registering the E197 (Ring scan glue memory by device count) in-repo harness, driver and report scripts, three measurement paths, and decision review.
