## E183 Capacity Slope, Read Cache Small-Disk Admission, and Sweep Cost Metrics — partially run (segment one H4 has sufficient evidence; segment two H2 run, the two R2e publish rows produce no class, literal registration insufficient; segment three H5 run, R5c does not emerge, literal registration insufficient; segment four H3 not run)

[中文](183-容量斜率读缓存小盘准入与清扫阈值的代价数.md) · **English** · [日本語](183-容量勾配・読み取りキャッシュ・小容量ディスクの受け入れ・掃除しきい値のコスト.ja.md)

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 R2a R2b R2c R2d R2e R2f R3a R3b R3c R4a R4b R4c R4d R5a R5b R5c R5d X14 X15 K10 K30 K31 K32 K33 K34 A30 A31 A33 A34 A35 A36 A37 A38 A41 V30 V31 V32 V33 V34 V35 V36 V37 T2 T3 T4 T5 G1 G2 G3 G4 G5 G6 G7 G8 M60 M64 M65 M70 M71 M72 M73 M74 M75 M76 M77 M78 M79 PC1 S4 S8 S16 L1 X17 X19 X21 X3 X5 X22 X25 X27 R2g R2h V38 V40 G9 G10 G11 G12 T7 T8 T9 A43 Q8 Q13 Q14 -->

Preregistration `research/prompts/e183-preregistration.md` has four segments: H4, H2, H5, H3. Its section 12 amendments, three for segment-one continuation and seven after the segment-two/three apparatus was written, all precede the corresponding artifacts. The question sheet `research/prompts/m3-close-cost-measurement-questions.md` has H2–H5; segment one answers H4 only: on the smallest disk that can be formatted and mounted writable, how large are the six admission-formula terms other than capacity, allocated and unreclaimable space, and which dominates? The subjects are current `crates/` mkfs, writable mount and session publication; the formula comes from D28 (Mount-Time Commitments) settled items 1 and 3.

The committed apparatus is `crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs`, a read-only driver of `singlefs-core` public entry points, without changes to other `crates/` files. Disks are sparse in-memory disks; S=4,8,16, journal ring L=1 MiB, plus one S=8/L=4 MiB point; default disks are 4 GiB × 2, L=768 MiB. The independent half is a pure Python anchor script that does not read `crates/`.

**Current status:**

- Segment one completed all ten H4 items sequentially in one process (`all`): 459 artifact lines, independent A31 verification of 56 readings with zero differences. Artifact registered for replay `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`; verification `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out` (Rerun 1, produced again after the local sweep threshold constant was changed to 365; a line-by-line diff against the 2026-10-23 one differs only in line 6, and the verdict lines are identical; see “Rerun 1”; the two old 2026-10-23 artifacts are kept).
- The segment-one artifact registered for replay is one produced again after an upstream write-order change (only the pinned value was fixed; the conclusions are unchanged): number acquisition now writes in two rounds, and at the spot of owed check C755 (recovery and two mount triggers fail differently on rollback) every writable mount asks F2 once, so the per-disk counts (`name=h4_mount_tally`, one line per item, per state and per disk) gain one system configuration write and four system configuration reads. The reproduced artifact has the same line and byte counts as the 2026-10-18 one, `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18.out` (the original artifact, byte-identical over two runs, is kept): 459 lines, 152434 bytes; the reproduction was also run twice and `cmp` is byte-identical. They differ only in 28 `name=h4_mount_tally` lines (lines 106, 107, 117, 118, 183, 184, 194, 195, 275, 276, 286, 287, 355, 356, 366, 367, 381, 382, 392, 393, 407, 408, 418, 419, 433, 434, 444, 445), and each of those differs only in the one field `system_configuration=`: `r26/106496,w3/12288` became `r30/122880,w4/16384`. None of the verdict lines and reading lines this page cites are among these 28 lines, and they are identical in both. The segment-one SHA-256 pinned in the apparatus unit test was replaced at that time with this artifact's `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`, and after Rerun 1 on 2026-10-30 it is re-pinned to `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870` (for the state of swapping it into the main workspace see “Rerun 1”). The verification output differs from the 2026-10-18 one, `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18-anchor-check.out`, only in `product_sha256` on line 1.
- Three minimum disk sizes in MiB (R4a): S=4, mkfs 18, empty-pool mount 18, mount with file 28; S=8, 18/18/34; S=16, 18/18/49; S=8/L=4 MiB, 21/21/37. Minimum disk for mount with a file is 1.555 / 1.888 / 2.722 / 1.761 times minimum mkfs disk. All three minimum-disk searches passed monotonic checks C+1…C+16 MiB.
- PC-admission positive control under original registration criterion, checked only at empty-pool C_empty_mount(S) − 1 MiB, holds in zero of four small-disk search items: mkfs already fails `UnitAreaBeyondDevice`, so empty-pool mount is never reached and `SpaceAdmissionRefusedBeforeAcquisition` cannot be reported. Under V33's literal rule count one failure: the empty-pool halves of R4a–R4c do not emerge; minimum empty-pool size belongs to mkfs geometry, not the admission formula. Amendment-added PC-admission-file positive control holds in all four: at C_file_mount − 1 MiB the writable mount succeeds, admission outcome `StillShortAfterTheFloorRaises`.
- With a file on the minimum disk, admission reserve is 906256 / 961401 / 969728 ppm of capacity for S=4 / 8 / 16 (R4b, reporting only). Of six terms the ring window dominates at 692 / 812 / 888 per thousand (R4c); all four small-disk points, including L=4 MiB, share the same dominant term.
- On default disks with a file (R4d), admission reserve is 56305 / 77478 / 119824 ppm for S=4 / 8 / 16. The dominant term is mount-time commitment for S=4 (546 per thousand), and ring window for S=8,16 (569,721 per thousand), unstable across S. Empty-pool writable-mount entry is 99 ppm.
- K32's empty-pool half fails at the `before_a_publish` entry (one reading for each of seven items: the ring window is charged, whereas the clause says zero for an empty pool). G8 marks “this clause cell may be wrong.” At the `writable_mount` entry the empty-pool ring window is zero, so it holds.
- The apparatus now has 87 unit tests and 64 mutation rows for E183 (Capacity Slope, Read Cache Small-Disk Admission, and Sweep Cost Metrics) (counted on 2026-11-03: `grep -c '#\[test\]'` on the apparatus source file and `grep -c '^E183 ' crates/mutations.tsv`; segment two's unit tests and mutations were added in section 12 of registration `research/prompts/e183-r2-prereg.md`, and this page has no run numbers for them). The readings of the 2026-10-30 version follow: 73 apparatus unit tests, and the main agent reported 73 passed, 0 failed in release; 54 mutation rows (rows in `crates/mutations.tsv` whose names begin with E183 (Capacity Slope, Read Cache Small-Disk Admission, and Sweep Cost Metrics) plus a space, counted with `grep -c`). The three mutation numbers await gate crates-mutation-replay at commit; this segment did not run the full mutation table. When segment one was written there were 24 unit tests and 13 mutations (M60, M64, M65 from registration section 9 segment one; M70 to M79 added by the apparatus), and the unit tests and mutations added by later segments are in registration section 12.
- Segment three H5 completed: artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out` (19067 lines, 26 shards, 13 on each of two machines, using the same program file); independent verification `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out` has zero differences over 24 cells plus controls. Readings are in “Segment-three readings.”
- H5: R5a is available. For R5b A, θ=64,365 does not cross one-shard upper bound; θ=369,738 does, consistently across six geometry points. Original PC-difference arm hits all 24 cells, V35 holds, R5c/R5d do not emerge. New PC-difference-read-chain arm passes all 24 cells. R5c has no actual ratio, so discriminative power self-test cannot be made (V36); G5 is neither triggered nor cleared. Under registration section 11's requirement that R5c emerge, H5 is insufficient for judgment. See “Fork table (question sheet H5…).”
- Segment two (H2) finished (Rerun 2 registration `research/prompts/e183-r2-prereg.md`, split into 5 shards through the unified task service, one round per shard, dispatched to both hosts): artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out` (949 lines; 9 cells × 5 rounds = 45 processes, all “finished within the threshold”). R2a: the time of one allocation is sublinear in capacity (b 0.251 to 0.322); R2b: the time of one publish is linear in capacity (b 0.811 to 0.875); R2d: the peak extra memory of an allocation is basically unrelated to capacity (112 bytes each time) and that of a publish is linear (b 1.022; at 16 TiB − 1 MiB one publish takes 15568569548 extra bytes, about two allocator copies); R2c: the allocator copy is linear (about 3.625 bytes per slot per disk); R2e: allocation time is sublinear in record count, and the two publish rows produce no class (the registered X5 cannot be made). The largest sparse disk this machine can make (the largest size whose cells all finished within the 24 GiB threshold) is the ladder endpoint 16 TiB − 1 MiB (maximum process high-water mark 23629737984 bytes), and it is 4 TiB when the threshold is halved. G2 triggers (Q8's predictions that R2a is basically unrelated and that R2e allocation is linear are both overturned), and G11 records one loss (each host has fewer than 5 rounds, and most cells have a load first column ≥ 2.0). Under the conditions of registration section 11, H2 is literally insufficient (the two R2e publish rows are missing and cannot be filled within the registration); readings and the fork table are in the section “Rerun 2”.
- Segment four (H3) apparatus was not written; the H3 row remains open, and the H2 and H5 rows are literally insufficient under the registration.

### What was done

1. Three prerequisites were checked before starting, per section 11 stop condition and section 12 amendment 2. T2, T3, PC-heap were individually run before the artifact with `--item constants`, `--item anchors`, `--item positive-control-heap`; output was in executor draft directory, not the artifact: `constants_compared=21 constants_mismatched=0`, `anchor_mismatches=0`, `positive_control_heap_holds=true`.
2. Official binary compiled from final apparatus by `scripts/hosts/compile-then-swap.py --release`, source sha256 `766d029c01fe54e885ef186b8caee481c99b5d8bde0eaa990449abdafa60af45`, binary sha256 `97e5e01ea487719935bc44f368874db4c5a82a851f941269a72c9c50c8665eec`. Sources of `singlefs-core`, `singlefs-format`, `singlefs-verification` did not change after the binary: `find crates -name '*.rs' -newer <binary>` lists only another experiment's apparatus and E183 itself. Subject tree `git diff HEAD -- crates/ | sha256sum` was `7fd21f75d5e478c1342b53d0d54b123303ac7993b0e4af473d049dceea10d5ca` (stat: “190 files changed, 73166 insertions(+), 4603 deletions(-)”), different from section 3's `847e0e54…`; the prerequisite stop clause did not stop the run.
3. Artifact ran twice with 8G memory cap, 31-thread cap, single-threaded apparatus; both exit 0, `cmp` byte-identical (T4 deterministic check). Artifact sha256 `190704189a413410206bac906fd499b729abe0c376ad5a68a7b53ddc22aa1f24`.
4. A31 independent verification: `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py` uses same-directory `anchors_e183.py` (registration section 13 command-one anchor script, sha256 `d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d`) to recompute unit-region capacity and ring-window bytes for every artifact reading and compare.
5. Segment three H5: `scripts/experiment-apparatus/e183-sweep-history-shards.sh` started 26 shards. On the first pass each of two machines compiled separately, yielding different binary sha256, and the apparatus stopped at merge as registered; the second host's 13 shards were invalidated and moved aside. On the second pass it ran using this machine's program file and merged the artifact. Details in “Segment three H5: method, artifact and subject tree.”
6. H5 independent verification and table utility: `sweep_history_anchor_check.py` checks all 24 cells against the same anchor script, zero differences; `sweep_cell_table.py` reads a 24-cell table from the artifact and divides to calculate bytes written per mount. Both are in `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/`.
7. 2026-10-23, pinned value fix only: after the two-round number acquisition write and owed check C755 (recovery and two mount triggers fail differently on rollback) landed, the apparatus unit test "segment one `all` output is byte-identical to the pinned checked-in artifact" went red on the SHA-256 (current run `b4ff64f1…`, pinned `7ed2e16c…`). In a repository copy (the main workspace's `crates/` of that day, without `target/`) the pinned value was replaced with the SHA-256 of the current run and the source comment gained one sentence; the apparatus was built in release and `all` was run twice (memory cap 8G, thread cap 31): both exited 0, `cmp` byte-identical, sha256 `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`, the same value an investigator got in another copy. The A31 verification script was rerun on the new artifact: exit 0, 56 readings, zero differences. The two replay registration lines and the artifact `driver_e183_anchor_check` reads now point to the new files; the old artifact and the old verification output are kept.
8. 2026-10-30, Rerun 1: the apparatus's local sweep threshold constant followed the default value 365 of C690 (Default sweep trigger threshold unsettled); segment one `all` produced its artifact again, a line-by-line diff changes only line 6, the verdict lines are unchanged, and A31 agrees; the replay registration now points to the new artifact; the procedure and the numbers are in “Rerun 1”.

### Replay

```bash
# Repository root; committed apparatus via two scripts/experiment-apparatus/replay.sh entries:
# driver_e183 compiles and runs all; driver_e183_anchor_check runs A31.
# The verification crate has to be compiled; these two lines run in the experiment-replay cell of gate research-build-and-replay.
bash scripts/experiment-apparatus/replay.sh E183

# Apparatus itself, same source without replay registration
cargo run -q --release -p singlefs-verification --bin e183_capacity_read_cache_reserve_and_sweep_costs -- all

# A31 independent check, pure Python
python3 -B -I scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out
```

Two E183 rows in `scripts/experiment-apparatus/replay.sh`: driver `@driver_e183`, artifact `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`, exact comparison; driver `@driver_e183_anchor_check`, artifact `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`, exact. Apparatus entry points `--list-items`, `--item <item>`, `all`; ten items: `constants`, `positive-control-heap`, `anchors`, `h4-small-disk-slots-per-region-4`, `h4-small-disk-slots-per-region-8`, `h4-small-disk-slots-per-region-16`, `h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes`, `h4-default-disk-slots-per-region-4`, `h4-default-disk-slots-per-region-8`, `h4-default-disk-slots-per-region-16`. Each item's elapsed time and exit code go to standard error, not the artifact.

Artifact row counts by type (exact output of `awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out | sort | uniq -c`; the same command on the 2026-10-23 one gives identical output, and the `md5sum` of the two outputs is the same):

```text
      5 name=anchor_chain_rewrite
      1 name=anchor_default_ring_unit_area_start
     15 name=anchor_geometry
      1 name=config
     21 name=constant_compare
      1 name=discrimination_self_proof
      1 name=done
      7 name=h4_config
      8 name=h4_dominance_across_points
     14 name=h4_instance_rows
      4 name=h4_minimum
     12 name=h4_monotonicity
     28 name=h4_mount_tally
     28 name=h4_reading
     28 name=h4_reading_checks
     56 name=h4_reading_device
    203 name=h4_trial
      4 name=positive_control_admission
      1 name=positive_control_heap
     10 name=shard_done
     11 name=verdict
```


Counting registered “arm × history × geometry”: four small-disk searches and three default-disk items, each with two states (empty/file) × two reading entry points (`writable_mount`, `before_a_publish`) × two disks = 56 `h4_reading_device` rows, plus one `h4_reading` and one `h4_reading_checks` per reading, 28 each. Each small-disk search has one `h4_minimum`, one `positive_control_admission`, and three `h4_monotonicity` rows (mkfs, empty mount, file mount). The 203 `h4_trial` rows are 41 + 47 + 62 + 50 attempts for the four small-disk items plus three default-disk items. Counts agree; no missing rows or fields.

### Rerun 1 (local sweep threshold constant 369 changed to 365, only segment one rerun)

Rerun registration `research/prompts/e183-r1-prereg.md` (sha256 `9417c63368fbe1b6f6d0e04844214d07a98059ca1a2cd6bc409f608b5076fd59`; nothing is written in the section 12 “amendments” paragraph); question sheet `research/prompts/m3-close-b19-e183-threshold-365-questions.md`, two rows. Cause: the default value written in the row of C690 (Default sweep trigger threshold unsettled) was settled as 365 (by the user on 2026-10-29), line 74 of `crates/singlefs-core/src/instance_table_sweep.rs` already reads `Self(365)`, while the apparatus's local constant `SWEEP_THRESHOLD_PLACEHOLDER_ROWS` was still 369, and two quick tier cases went red on 2026-10-30 (reading in registration section 4: the comparison case reported “local 369, implementation 365”, and the byte-for-byte case went red at the same time). Only two lines of the apparatus were changed (the constant value from 369 to 365, and the doc comment now quotes the new registration entry; the constant name and the key name `sweep_threshold_placeholder_rows` in the artifact are unchanged), and then the segment-one `all` artifact was produced again; segments two, three and four were not touched this time.

Procedure and prerequisites (all checked on this pass):

- The four start prerequisites hold: `instance_table_sweep.rs:74` is `Self(365)`; `grep -n SWEEP_THRESHOLD_PLACEHOLDER_ROWS` on the apparatus hits two lines, 78 and 1879; the `sha256sum` of the two old 2026-10-23 artifacts matches the registration (`b4ff64f1…`, `da09f826…`); the `sha256sum` of `anchors_e183.py` and `small_disk_anchor_check.py` matches the registration.
- The apparatus was built by `scripts/hosts/compile-then-swap.py --release` and then swapped whole into the main workspace (exit 0); binary sha256 `4130ccac6b3f66d0817e2f318362f558e65ac658de28fe3abc685e898078d190`; the source file at the moment of the artifact has sha256 `56a6b495d92699d42eb06cccd8843f54557c9b06ba893be452000bc23a168810`, byte-identical to the draft copy, and it differs from the main workspace's copy before the swap (`b3b37608…`) only by those two lines. `find crates -name '*.rs' -newer <binary>` has zero hits under `singlefs-core`, `singlefs-format` and `singlefs-harness` (it lists only a few other files in the checker tier package).
- Subject tree: `git diff HEAD -- crates/ | sha256sum` is `e5f1218c991320c5456abbd300932d2aa1937a8790b7848fb2712de5deff0f0f`, the last line of `--stat` is “193 files changed, 54733 insertions(+), 5377 deletions(-)”, different from `97a92443…` (182 files) written in registration section 3, because other sessions kept changing the same tree; `singlefs-core`, which `all` uses, has no source file newer than the binary.
- `all` was run twice (in parallel, each through the memory wrapper at 8G with thread cap 31; the apparatus is single-threaded); both exited 0, `cmp` byte-identical, sha256 `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870`; the two runs together took 226 seconds. The artifact was landed as `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`: 459 lines, 152434 bytes, `grep -vc '^E7RESULT '` is 0, the last byte is a newline, last line `E7RESULT name=done emitted=459`.
- A31 verification: `python3 -B -I …/small_disk_anchor_check.py …/anchors_e183.py <new artifact>` exited 0, and its output was landed as `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out` (58 lines, sha256 `eaecf66cbfa6057b56bdf71e470d55998061388c6f5ad52c7fd307cc7b144c00`), last line `E7RESULT name=verdict item=anchor_check readings_checked=56 mismatches=0`.

Line-by-line diff (question sheet row 1; exact output of `diff` with the 2026-10-23 artifact first and the 2026-10-30 artifact second, 1 hunk):

```text
6c6
< E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=369 implementation=369 equal=true
---
> E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=365 implementation=365 equal=true
```

Verdicts (output of the comparison script `compare_e183_first_rerun.py`; the script and its output are in the executor's draft directory and not committed; pure Python, all twelve self-check cases pass):

| Quantity | Verdict |
|---|---|
| Line-by-line diff | Only line 6 changes: exactly one hunk, header `6c6`; the old line equals word for word the old line 6 quoted in the registration, and the new line equals word for word the new line pinned in the registration |
| New value of line 6 | Correct: `local=365 implementation=365 equal=true` |
| Line and byte counts | 459 lines, 152434 bytes, the same as the old artifact; 0 lines not starting with `E7RESULT` |
| Comparison verdict line | Correct: `name=verdict item=constants constants_compared=21 constants_mismatched=0` |
| Verdict lines (the 11 lines in “Verdicts” below) | Compared field by field, all 11 unchanged: the H4 verdict lines are unchanged |
| Determinism over two runs | The two runs are byte-identical |
| A31 verification | Agrees: exit 0, 56 readings, zero differences |
| A31 output diff | Only the artifact sha line changes: 1 hunk `1c1`, differing only in the value of `product_sha256=`, and the new value equals the new artifact's `sha256sum` |
| Whether the two sides agree | They agree: the artifact changes only line 6 and the verification output only its first line; nothing contradicts |
| Old artifacts not overwritten | Correct: after the runs the `sha256sum` of the two 2026-10-23 artifacts is still what the registration records |

The per-type row counts of the artifact are identical to the old artifact's (see the command in “Replay”); the new artifact has 11 `name=verdict` lines, identical as whole lines to the 11 lines copied in “Verdicts” and at the same line numbers (lines 23, 26, 49, 120, 197, 289, 369, 395, 421, 447, 458). The fields with value `false` among the 11 are: `positive_control_admission_holds=false` and `reading_checks_hold=false` on lines 120, 197, 289, 369; `reading_checks_hold=false` on lines 395, 421, 447; `dominant_item_stable_across_default_disk_points=false` on line 458. The reasons are the same as for the old artifact (mkfs fails first in the empty-pool half, the K32 empty-pool `before_a_publish` entry is recorded under G8, and the default-disk dominant term is unstable across S; see “Current status” at the top of this page), and all are expected by the registration; there is no `not_run`, and the integer counts of the violation, mismatch, ambiguity and failure kinds are all 0 (`panicked_trials=0`, `anchor_mismatches=0`, `constants_mismatched=0`, `invalid_device_conditions=0`).

Answer to question sheet row 1: after the constant is changed to 365, the segment-one `all` artifact changes only the comparison on line 6; no other line changes, and the 11 H4 verdict lines are unchanged. What registration P6 predicted (only line 6 changes) agrees with what was measured.

Swapping the pinned value and the replay registration: the verdict lines are unchanged, line 6 is correct and A31 agrees, so the registration lets it through; therefore the two replay registration lines and the artifact `driver_e183_anchor_check` reads now point to the new files (lines 1811, 1812, 3275 of `scripts/experiment-apparatus/replay.sh`), and two comparison-unit registration lines go into `.claude/gate.d/replay-comparison-units.tsv`; the segment-one SHA-256 and line count pinned in the apparatus (`SEGMENT_ONE_PINNED_SHA256` changed to `7c5add85…`, line count 459 unchanged) were changed in the draft copy, and the main agent swapped them into the main workspace through `compile-then-swap.py` (the whole command through `run-with-memory-cap.sh 24G`), exit 0 (before the swap the main workspace's copy pinned `b4ff64f1…`, while the apparatus already output `7c5add85…`). The two old artifacts are kept, neither renamed nor deleted.

Question sheet row 2: A31 agrees (table above). The two cases (`local_constants_equal_the_implementation_constants`, `segment_one_all_output_equals_the_registered_product_byte_for_byte`) are tests of the verification crate; the executor does not run them, the main agent ran them in the main workspace after the pinned value was swapped in, result “2 passed; 0 failed” (the 2026-10-30 entry “E183 U3 compile-then-swap” of scheduling record `records/2026-10-18-里程碑三收尾与近期欠账.md`); the red before the change is those two cases in the registration section 4 reading above.

Mutations: no mutation rows are added this time. The apparatus has 54 mutation rows; after the change the original text of each row still hits exactly once in the apparatus (counted in both the draft copy and the main workspace; rows not hitting exactly once: 0); no row in the mutation table has original text containing the constant line or the pinned value.

### Verdicts (full lines copied from `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out`, lines 23, 26, 49, 120, 197, 289, 369, 395, 421, 447, 458; the same 11 lines of the 2026-10-30 artifact are identical, see the previous section)

```text
E7RESULT name=verdict item=constants constants_compared=21 constants_mismatched=0
E7RESULT name=verdict item=positive-control-heap positive_control_heap_holds=true
E7RESULT name=verdict item=anchors anchor_mismatches=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-4 positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-8 positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-16 positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes positive_control_admission_holds=false make_filesystem_minimum_found=true empty_mount_minimum_found=true file_mount_minimum_found=true every_minimum_disk_monotone_over_the_window=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-default-disk-slots-per-region-4 both_states_admitted=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-default-disk-slots-per-region-8 both_states_admitted=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4-default-disk-slots-per-region-16 both_states_admitted=true reading_checks_hold=false panicked_trials=0
E7RESULT name=verdict item=h4_summary dominant_item_stable_across_small_disk_points=true dominant_item_stable_across_default_disk_points=false discrimination_self_proof_holds=true invalid_device_conditions=0
```


`grep -n 'name=verdict' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` lists eleven verdicts. Each failing field:

| Field | Artifact line | Value | Why it has this value | Expected by registration? |
|---|---|---|---|---|
| `positive_control_admission_holds=false` | 120, 197, 289, 369 (four small-disk search items) | false | Empty-pool half of PC-admission does not hold: `empty_mount_one_step_below=not_reached`; at C_empty_mount − 1 MiB (17 MiB, or 20 MiB when L = 4 MiB), mkfs first fails with `UnitAreaBeyondDevice`, so the empty mount is not reached | Not anticipated by original registration; count one failure under V33 text (see "Cancellation, failure, and stop clauses item by item"); the four PC-admission-file items added by amendment all hold |
| `reading_checks_hold=false` | 120, 197, 289, 369 (small disks), 395, 421, 447 (default disks) | false | In `name=h4_reading_checks` across all 7 items, `state=empty entry=before_a_publish` has `k32_ring_window_by_the_clauses=false`: on the publication entry, the empty-pool reading's ring window is deducted (`ring_window_bytes` is non-zero), whereas the clause (K10) states that an empty pool is 0; the remaining K30, K31, capacity, and K32 with a file state are all true | An anticipated outcome under the third registered amendment, recorded as G8 |
| `dominant_item_stable_across_default_disk_points=false` | 458 | false | With a file on the default disk, the dominant term is mount-time commitment for S = 4, and ring window for S = 8 and 16; two `name=h4_dominance_across_points group=default_disks state=file` rows have `stable=false` (artifact lines 455 and 456) | Expected possible outcome under registration (§8.2 "dominant term differs across points ⇒ unstable") |

All other verdicts: `constants_mismatched=0`, `anchor_mismatches=0`, seven `panicked_trials=0`, `invalid_device_conditions=0`; `positive_control_heap_holds=true`, four `every_minimum_disk_monotone_over_the_window=true`, `discrimination_self_proof_holds=true`, `dominant_item_stable_across_small_disk_points=true`, three `both_states_admitted=true`. Artifact `grep -c 'not_run'` is 0 and there are no empty fields (`grep -c '=\( \|$\)'` is 0).

### Readings: three minimum disks (R4a) and positive controls

Full lines from artifact lines 93, 97, 170, 174, 262, 266, 342, 346: one `h4_minimum` and one `positive_control_admission` for each search:

```text
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-4 make_filesystem_mebibytes=18 empty_mount_mebibytes=18 file_mount_mebibytes=28 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=25 file_mount_over_make_filesystem=1555permille trials=41 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-4 mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=17 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-8 make_filesystem_mebibytes=18 empty_mount_mebibytes=18 file_mount_mebibytes=34 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=31 file_mount_over_make_filesystem=1888permille trials=47 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-8 mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=17 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-16 make_filesystem_mebibytes=18 empty_mount_mebibytes=18 file_mount_mebibytes=49 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=46 file_mount_over_make_filesystem=2722permille trials=62 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-16 mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=17 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
E7RESULT name=h4_minimum item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes make_filesystem_mebibytes=21 empty_mount_mebibytes=21 file_mount_mebibytes=37 empty_mount_still_short_mebibytes=not_found file_mount_still_short_mebibytes=34 file_mount_over_make_filesystem=1761permille trials=50 panicked_trials=0
E7RESULT name=positive_control_admission item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch one_step_below_the_empty_mount_minimum_mebibytes=20 empty_mount_one_step_below=not_reached file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises holds=false
```

- Minimum disks in MiB: `make_filesystem_mebibytes` (C_mkfs) equals `empty_mount_mebibytes` (C_empty_mount); `file_mount_mebibytes` (C_file_mount) is 28 / 34 / 49 / 37 and `file_mount_over_make_filesystem` is 1555 / 1888 / 2722 / 1761 per mille. Search starts at 4 MiB in 1 MiB increments and ends 16 MiB after the largest minimum (`name=h4_config window_mebibytes=16`); the 1024 MiB cap was not reached and no minimum is an endpoint.
- Minimum “mounted but still short after raising the floor,” `file_mount_still_short_mebibytes`, is 25 / 31 / 46 / 34, three MiB below C_file_mount. Empty state reports `empty_mount_still_short_mebibytes=not_found`: no empty mount in the search was still short after the floor was raised.
- PC-admission empty half: `mount_with_the_test_only_switch_at_the_make_filesystem_minimum=ok:NotJudgedByTheTestOnlySwitch` holds (test-only switch allows mount on the minimum mkfs disk); `empty_mount_one_step_below=not_reached` because mkfs fails `UnitAreaBeyondDevice` at 17 MiB, or 20 MiB for L=4 MiB.
- PC-admission-file: `file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises` for all four, and all four `file_mount_mebibytes` are found. Counting:

```bash
P=research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out
grep 'name=positive_control_admission' $P | grep -c 'file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises'
grep 'name=positive_control_admission' $P | grep -c 'empty_mount_one_step_below=failed:SpaceAdmissionRefusedBeforeAcquisition'
grep 'name=h4_minimum' $P | grep -vc 'file_mount_mebibytes=not_found'
```


Exact outputs are `4`, `0`, `4` respectively.

Outcomes grouped into contiguous disk-size ranges within each small-disk search (`name=h4_trial`). For individual MiB rows: `grep 'name=h4_trial item=h4-small-disk-slots-per-region-8 ' $P | sed -e 's/.*device_mebibytes=\([0-9]*\) make_filesystem=\([^ ]*\) empty_mount=\([^ ]*\) first_file=\([^ ]*\) unmount=[^ ]* file_mount=\([^ ]*\) .*/\1 \2 \3 \4 \5/'`, with `P=research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out`; replace the item to read the others.

| Item | Disk size MiB | mkfs | Empty mount | First file | File mount |
|---|---|---|---|---|---|
| S = 4 | 4–7 | `RootRingBeyondDevice` | Not reached | Not reached | Not reached |
| S = 4 | 8–17 | `UnitAreaBeyondDevice` | Not reached | Not reached | Not reached |
| S = 4 | 18 | Success | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | Not reached |
| S = 4 | 19–24 | Success | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | Not reached |
| S = 4 | 25–27 | Success | `AdmittedBeforeAcquisition` | Success | `StillShortAfterTheFloorRaises` |
| S = 4 | 28–44 | Success | `AdmittedBeforeAcquisition` | Success | `AdmittedBeforeAcquisition` |
| S = 8 | 18 | Success | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | Not reached |
| S = 8 | 19–30 | Success | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | Not reached |
| S = 8 | 31–33 | Success | `AdmittedBeforeAcquisition` | Success | `StillShortAfterTheFloorRaises` |
| S = 8 | 34–50 | Success | `AdmittedBeforeAcquisition` | Success | `AdmittedBeforeAcquisition` |
| S = 16 | 18 | Success | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | Not reached |
| S = 16 | 19–45 | Success | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | Not reached |
| S = 16 | 46–48 | Success | `AdmittedBeforeAcquisition` | Success | `StillShortAfterTheFloorRaises` |
| S = 16 | 49–65 | Success | `AdmittedBeforeAcquisition` | Success | `AdmittedBeforeAcquisition` |
| S = 8, L = 4 MiB | 21 | Success | `AdmittedBeforeAcquisition` | `EmptyClusterSegmentsBelowTheFloor` | Not reached |
| S = 8, L = 4 MiB | 22–33 | Success | `AdmittedBeforeAcquisition` | `SpaceAdmissionRefused` | Not reached |
| S = 8, L = 4 MiB | 34–36 | Success | `AdmittedBeforeAcquisition` | Success | `StillShortAfterTheFloorRaises` |
| S = 8, L = 4 MiB | 37–53 | Success | `AdmittedBeforeAcquisition` | Success | `AdmittedBeforeAcquisition` |


Interpretation: empty-pool mount is admitted throughout from 1 MiB above C_empty_mount. Along the file-present path, first-file publication is first refused by admission (`SpaceAdmissionRefused`, 19–30 MiB for S=8), then mount succeeds but remains short after the floor is raised (31–33 MiB), and only 34 MiB is admitted. At C_file_mount available(d) is just 32692 bytes (`file_available_minimum_bytes=32692` in the S=8 34 MiB trial; at 33 MiB it is −1015884). On the mkfs minimum 18 MiB the first file fails `EmptyClusterSegmentsBelowTheFloor`, not admission refusal.

### Readings: individual terms and total on minimum disks (R4b), dominant term (R4c)

File-present state, `writable_mount` entry, device 0; full artifact lines 110, 187, 279, 359, for S=4,8,16 and S=8/L=4 MiB respectively, using each C_file_mount:

```text
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-4 state=file entry=writable_mount device=0 capacity_bytes=11534336 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=2883584 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=229376 ring_window_bytes=7241728 admission_reserve_bytes=10453068 available_bytes=425908 reserve_over_capacity_parts_per_million=906256 share_permille_mount_time_commitment=275 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=9 share_permille_checkpoint_reserve_pool_share=21 share_permille_ring_window=692 dominance=dominant:ring_window:692permille
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-8 state=file entry=writable_mount device=0 capacity_bytes=17825792 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=2883584 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=229376 ring_window_bytes=13926400 admission_reserve_bytes=17137740 available_bytes=32692 reserve_over_capacity_parts_per_million=961401 share_permille_mount_time_commitment=168 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=5 share_permille_checkpoint_reserve_pool_share=13 share_permille_ring_window=812 dominance=dominant:ring_window:812permille
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-16 state=file entry=writable_mount device=0 capacity_bytes=33554432 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=3276800 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=262144 ring_window_bytes=28901376 admission_reserve_bytes=32538700 available_bytes=360372 reserve_over_capacity_parts_per_million=969728 share_permille_mount_time_commitment=100 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=3 share_permille_checkpoint_reserve_pool_share=8 share_permille_ring_window=888 dominance=dominant:ring_window:888permille
E7RESULT name=h4_reading_device item=h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes state=file entry=writable_mount device=0 capacity_bytes=17825792 allocated_bytes=655360 unreclaimable_bytes=0 mount_time_commitment_bytes=2883584 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=229376 ring_window_bytes=13926400 admission_reserve_bytes=17137740 available_bytes=32692 reserve_over_capacity_parts_per_million=961401 share_permille_mount_time_commitment=168 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=5 share_permille_checkpoint_reserve_pool_share=13 share_permille_ring_window=812 dominance=dominant:ring_window:812permille
```


Both disks match item by item; in file-present state both entry points also match except `entry`. Normalizing device and checking duplicates yields zero non-pairs; normalizing device and entry on file-state rows yields `7 4` (seven distinct readings, four rows each).

| Point | Capacity, unit-area bytes | Mount-time commitment | Checkpoint reserve-pool share | Ring window | Admission reserve, six-term sum | Available(d) | Reserve / capacity ppm | Dominant |
|---|---|---|---|---|---|---|---|---|
| S=4, C=28 MiB | 11534336 | 2883584 | 229376 | 7241728 | 10453068 | 425908 | 906256 | Ring window 692‰ |
| S=8, C=34 MiB | 17825792 | 2883584 | 229376 | 13926400 | 17137740 | 32692 | 961401 | Ring window 812‰ |
| S=16, C=49 MiB | 33554432 | 3276800 | 262144 | 28901376 | 32538700 | 360372 | 969728 | Ring window 888‰ |
| S=8, L=4 MiB, C=37 MiB | 17825792 | 2883584 | 229376 | 13926400 | 17137740 | 32692 | 961401 | Ring window 812‰ |

Of the six terms, abandoned-root exclusive bytes and pending-delete share are zero in all 56 readings; `grep` for each returns 56. Committed-reservation share is 98380 bytes in all 28 file-state readings and zero in all 28 empty-state readings; grouping state and `committed_reservation_share_bytes` gives these two rows. The six-term sum in the table includes 98380. R4b's ratio on a minimum disk is nearly tautological: available(d) has just reached ≥0, so ratio ≈ 1 − allocated/capacity. Registration says report but do not treat as criterion; possible reversal rests on R4c and R4d.

R4c cross-point summary and discriminative power self-test, full artifact lines 449–457 and 459. Lines 449–450, `state=empty`, are the empty half withheld after V33's failure: archived but not results for R4a–R4c:

```text
E7RESULT name=h4_dominance_across_points group=small_disks state=empty entry=writable_mount points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:mount_time_commitment:952permille;h4-small-disk-slots-per-region-8=dominant:mount_time_commitment:952permille;h4-small-disk-slots-per-region-16=dominant:mount_time_commitment:952permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:mount_time_commitment:952permille stable=true
E7RESULT name=h4_dominance_across_points group=small_disks state=empty entry=before_a_publish points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:ring_window:951permille;h4-small-disk-slots-per-region-8=dominant:ring_window:974permille;h4-small-disk-slots-per-region-16=dominant:ring_window:986permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:ring_window:974permille stable=true
E7RESULT name=h4_dominance_across_points group=small_disks state=file entry=writable_mount points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:ring_window:692permille;h4-small-disk-slots-per-region-8=dominant:ring_window:812permille;h4-small-disk-slots-per-region-16=dominant:ring_window:888permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:ring_window:812permille stable=true
E7RESULT name=h4_dominance_across_points group=small_disks state=file entry=before_a_publish points=4 outcomes=h4-small-disk-slots-per-region-4=dominant:ring_window:692permille;h4-small-disk-slots-per-region-8=dominant:ring_window:812permille;h4-small-disk-slots-per-region-16=dominant:ring_window:888permille;h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes=dominant:ring_window:812permille stable=true
E7RESULT name=h4_dominance_across_points group=default_disks state=empty entry=writable_mount points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:mount_time_commitment:952permille;h4-default-disk-slots-per-region-8=dominant:mount_time_commitment:952permille;h4-default-disk-slots-per-region-16=dominant:mount_time_commitment:952permille stable=true
E7RESULT name=h4_dominance_across_points group=default_disks state=empty entry=before_a_publish points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:ring_window:995permille;h4-default-disk-slots-per-region-8=dominant:ring_window:997permille;h4-default-disk-slots-per-region-16=dominant:ring_window:998permille stable=true
E7RESULT name=h4_dominance_across_points group=default_disks state=file entry=writable_mount points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:mount_time_commitment:546permille;h4-default-disk-slots-per-region-8=dominant:ring_window:569permille;h4-default-disk-slots-per-region-16=dominant:ring_window:721permille stable=false
E7RESULT name=h4_dominance_across_points group=default_disks state=file entry=before_a_publish points=3 outcomes=h4-default-disk-slots-per-region-4=dominant:mount_time_commitment:546permille;h4-default-disk-slots-per-region-8=dominant:ring_window:569permille;h4-default-disk-slots-per-region-16=dominant:ring_window:721permille stable=false
E7RESULT name=discrimination_self_proof judgement=r4c_dominance readings=28 flipped=28 holds=true
E7RESULT name=done emitted=459
```


- File state, R4c result rows 451–452: ring window dominates all four small-disk points at 692 / 812 / 888 / 812 per mille, `stable=true`. Share increases with S; L=4 MiB and L=1 MiB have the same dominant item. All exceed the 500‰ majority threshold.
- Discriminative power self-test line 457: moving the R4c threshold to the other side of each reading's largest share flips all 28/28 from “dominant,” `holds=true`; V36 does not trigger.
- Empty state, retained for record: at `writable_mount`, mount-time commitment dominates at 952‰ (capacity only 1 MiB, six-term sum 344064 bytes, available(d) 655360). At `before_a_publish`, ring window dominates at 951 / 974 / 986 / 974‰; available(d) is negative −6160384 / −12451840 / −25034752 / −12451840 bytes, so admission for first-file publication on all four disks is refused.

### Readings: default disk (R4d)

Full artifact lines 374, 385, 400, 411, 426, 437, `writable_mount` entry, device 0, empty then file state for each S. Disks are 4 GiB × 2, L=768 MiB, unit-area capacity 3472883712 bytes:

```text
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-4 state=empty entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=49152 unreclaimable_bytes=0 mount_time_commitment_bytes=327680 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=0 checkpoint_reserve_pool_share_bytes=16384 ring_window_bytes=0 admission_reserve_bytes=344064 available_bytes=3472490496 reserve_over_capacity_parts_per_million=99 share_permille_mount_time_commitment=952 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=47 share_permille_ring_window=0 dominance=dominant:mount_time_commitment:952permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-4 state=file entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=819200 unreclaimable_bytes=0 mount_time_commitment_bytes=106889216 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=8896512 ring_window_bytes=79659008 admission_reserve_bytes=195543116 available_bytes=3276521396 reserve_over_capacity_parts_per_million=56305 share_permille_mount_time_commitment=546 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=45 share_permille_ring_window=407 dominance=dominant:mount_time_commitment:546permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-8 state=empty entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=49152 unreclaimable_bytes=0 mount_time_commitment_bytes=327680 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=0 checkpoint_reserve_pool_share_bytes=16384 ring_window_bytes=0 admission_reserve_bytes=344064 available_bytes=3472490496 reserve_over_capacity_parts_per_million=99 share_permille_mount_time_commitment=952 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=47 share_permille_ring_window=0 dominance=dominant:mount_time_commitment:952permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-8 state=file entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=819200 unreclaimable_bytes=0 mount_time_commitment_bytes=106889216 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=8896512 ring_window_bytes=153190400 admission_reserve_bytes=269074508 available_bytes=3202990004 reserve_over_capacity_parts_per_million=77478 share_permille_mount_time_commitment=397 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=33 share_permille_ring_window=569 dominance=dominant:ring_window:569permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-16 state=empty entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=49152 unreclaimable_bytes=0 mount_time_commitment_bytes=327680 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=0 checkpoint_reserve_pool_share_bytes=16384 ring_window_bytes=0 admission_reserve_bytes=344064 available_bytes=3472490496 reserve_over_capacity_parts_per_million=99 share_permille_mount_time_commitment=952 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=47 share_permille_ring_window=0 dominance=dominant:mount_time_commitment:952permille
E7RESULT name=h4_reading_device item=h4-default-disk-slots-per-region-16 state=file entry=writable_mount device=0 capacity_bytes=3472883712 allocated_bytes=819200 unreclaimable_bytes=0 mount_time_commitment_bytes=106889216 abandoned_root_exclusive_bytes=0 pending_delete_share_bytes=0 committed_reservation_share_bytes=98380 checkpoint_reserve_pool_share_bytes=8896512 ring_window_bytes=300253184 admission_reserve_bytes=416137292 available_bytes=3055927220 reserve_over_capacity_parts_per_million=119824 share_permille_mount_time_commitment=256 share_permille_abandoned_root_exclusive=0 share_permille_pending_delete_share=0 share_permille_committed_reservation_share=0 share_permille_checkpoint_reserve_pool_share=21 share_permille_ring_window=721 dominance=dominant:ring_window:721permille
```


| S | Empty reserve / capacity ppm | File reserve / capacity ppm | File-state dominant |
|---|---|---|---|
| 4 | 99 | 56305 | Mount-time commitment 546‰ |
| 8 | 99 | 77478 | Ring window 569‰ |
| 16 | 99 | 119824 | Ring window 721‰ |

For default disks with a file, ring-window bytes match A31's anchors: 79659008 / 153190400 / 300253184 for S=4 / 8 / 16; verification in `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out`. Mount-time commitment stays 106889216 bytes across S, exceeding the ring window for S=4; dominance switches between S=4 and 8. Empty `before_a_publish` reserve/capacity is 23036 / 44209 / 86555 ppm (lines 378, 404, 430); this entry charges the ring window even for an empty pool, failing K32's empty half.

### Invalidations, failures and stop conditions, item by item (segment one)

| Clause | Status | Reading |
|---|---|---|
| Starting prerequisite false (stop) | Pass | Registration section 12 amendment 2; `grep -n '问法待'` leaves only the stop clause quoting it; rerunning command one gives registered sha256. |
| T2 local constants comparison | Pass | Line 23 `constants_compared=21 constants_mismatched=0`. |
| T3 apparatus geometry versus anchor script | Pass | Line 49 `anchor_mismatches=0`; 15 `name=anchor_geometry`, five `name=anchor_chain_rewrite`. |
| T4 determinism over two runs | Pass | `cmp` of artifact and second output produces no output; both exit 0. |
| T5/C0 patch hit count not one or A41 mismatch (stop) | N/A | Segments three and four. |
| V30 PC-heap | Pass | Line 25 `E7RESULT name=positive_control_heap peak_over_baseline_bytes=67108864 lower_bytes=67108864 upper_bytes=67174400 holds=true`. |
| V33 PC-admission | **Empty half triggers, count one failure; file half added by amendment does not trigger** | All four small-disk items have `empty_mount_one_step_below=not_reached`, zero hold. Empty halves of R4a–R4c do not emerge; minimum disk belongs to mkfs geometry. File half: all four `file_mount_one_step_below_the_file_minimum=StillShortAfterTheFloorRaises`; file halves of R4a, R4b, R4c are reported. |
| V36 discriminative power self-test | Pass | Line 457 `flipped=28 holds=true`. |
| V37 unreadable is not zero | Not triggered | No empty artifact fields (`grep -c '=\( \|$\)'` is 0); `none` appears only in not-reached fields such as `file_available_minimum_bytes=none`. |
| G4 minimum disk nonmonotonic | Not triggered | All 12 `name=h4_monotonicity` rows have `monotone=true` and `failures_in_the_window_mebibytes=none`. |
| G7 K31 c differs between two sides | Not triggered | All 28 readings `k31_mount_time_commitment_by_the_clauses=true`; `checkpoint_cost_blocks_by_the_pool` equals `checkpoint_cost_blocks_by_the_commitment`. Values 1 in 14 readings, 14 in six, 16 in two, 543 in six. |
| G8 clause cell may be wrong | **Triggered in seven empty `before_a_publish` K32 readings** | Filtering `k32_ring_window_by_the_clauses=false` and `state=empty entry=before_a_publish` gives 7, equal to all false K32 readings. Other 21 are true: seven empty `writable_mount` have ring window 0, and 14 file-state entry readings match the clause formula. K30 pending-delete occupancy =0 is true for all 28. |
| G1, G2, G3, G5, G6 | N/A | Segments two through four. |

K32's location: D28 settled item 1's ring-window clause says only “permanent deduction, calculated from geometry.” Registration K32's empty-pool zero half derives from K10, the comment in `crates/singlefs-core/src/admission.rs` citing “ring window starts with first-file-version publication,” which covers only the writable-mount judgment. Implementation charges the empty pool's `before_a_publish` reading for S=4 / 8 / 16 at 6815744 / 13107200 / 25690112 bytes; the artifact's `ring_window_by_the_clauses_if_deducted_bytes` matches. The A31 script reads that entry as charged, consistent with implementation; neither script nor apparatus calculation is wrong. Implementation and clause disagree in this cell, recorded as “the clause cell may be wrong” without deciding which side is wrong.

### Unit tests and mutations

- Unit tests: at segment-one write-up, `grep -c '#[test]' crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` gave 24; main agent's `cargo test --release` reported 24 passed, 0 failed.
- Mutation table: The mutation rows of the merged apparatus were appended to the end of `crates/mutations.tsv`. Counting lines in `crates/mutations.tsv` whose names start with E183 (cost numbers for capacity slope, read cache, small-disk admission, and sweep thresholds) followed by a space (`grep -c`) yields `13`; the original text of each mutation row matches exactly once in the apparatus source file, all named test functions exist, and there are no backslash escapes other than `\n`. The three counts for caught / invalid / unkilled await the full rerun of the whole table under the `crates-mutation-replay` gate at commit time; this segment did not run mutations.

| Mutation | Change | Named test |
|---|---|---|
| M60 | Change counting allocator peak update from maximum to storing current value. | `counting_allocator_peak_stays_at_the_maximum_after_the_memory_is_freed` |
| M64 | Treat a mount still short after raising floor as successful in minimum-disk search. | `a_mount_that_stays_short_after_raising_the_floor_is_not_counted_as_admitted` |
| M65 | Omit ring window from admission-reserve sum. | `admission_reserve_adds_all_six_items_including_the_ring_window` |
| M70 | The read/write counting wrapper incorrectly counts the upper boundary of the system configuration region into the system configuration region itself | `fixed_regions_classify_offsets_by_their_landing_zone` |
| M71 | Omit last monotonicity-window point. | `monotonicity_lists_every_failure_in_the_sixteen_mebibyte_window_and_nothing_beyond` |
| M72 | Treat exactly half as not dominant. | `dominance_needs_at_least_half_of_the_total` |
| M73 | Omit N_switch in chain-rewrite page formula. | `chain_rewrite_bytes_cross_to_a_second_page_exactly_at_the_registered_row` |
| M74 | Undercount unit-area slots by one. | `local_geometry_reproduces_every_registered_script_row` |
| M75 | Stop search one step early. | `search_stops_sixteen_mebibytes_after_the_largest_of_the_three_minima` |
| M76 | Omit one factor of row-publication metadata from mount-time commitment. | `mount_time_commitment_by_the_clauses_matches_the_hand_computed_value` |
| M77 | Change expected empty-pool ring window to nonzero. | `reading_checks_hold_on_a_reading_built_by_the_clauses_and_fail_when_one_item_is_off` |
| M78 | Do not require both first-file creation and unmount to succeed for file state. | `the_file_state_needs_every_step_of_the_chain_to_have_succeeded` |
| M79 | Do not move discriminative power threshold past the maximum share. | `moving_the_dominance_threshold_across_the_largest_share_flips_the_class` |

Sampling: M64 changes output only on disks where still-short precedes admitted; all four small-disk artifact items have that condition, with `file_mount_still_short_mebibytes` three MiB below `file_mount_mebibytes`, so the artifact is sensitive. M72 differs only when the largest share is exactly 500‰; none of 56 artifact readings has that maximum (13 dominance values), so sensitivity is limited to the synthetic test point. M65 is sensitive in file state and on default disks, but not at empty `writable_mount` where ring window is zero.

### Fork table (question-sheet H4; registration section 11 requires R4a, R4c and R4d)

| Question | Status | Remaining | Could remaining measurements reverse it? | Command, P is the H4 artifact |
|---|---|---|---|---|
| H4 · R4a, three minima | Sufficient for file half: C_mkfs, C_empty_mount, C_file_mount and minimum still-short disk all obtained; V33 triggers on empty half, whose minimum belongs to mkfs geometry and is withheld. | None | Unrun H2/H3/H5 are unrelated to H4 and cannot reverse it. | `grep 'name=h4_minimum\|name=positive_control_admission' $P` |
| H4 · R4b, reserve / capacity, report only | File-state minimum: 906256 / 961401 / 969728 ppm; empty half withheld. | None | No; nearly tautological at minimum as registration states. | `grep 'name=h4_reading_device' $P | grep 'state=file entry=writable_mount device=0 '`, read `reserve_over_capacity_parts_per_million`. |
| H4 · R4c, dominant term | File half sufficient: ring window dominates four small points (692 / 812 / 888 / 812‰); default disk with file unstable across S; discriminative power 28/28. | None | No. | `grep 'name=h4_dominance_across_points\|name=discrimination_self_proof' $P`. |
| H4 · R4d, default ratio | File 56305 / 77478 / 119824 ppm for S=4 / 8 / 16; empty 99 ppm. | None | No; report only. | `grep 'name=h4_reading_device' $P | grep 'item=h4-default'`, same field. |

H4 is sufficient for judgment. Registration's remaining H2 and H3, marked “not run after sufficient evidence,” belong to other question-sheet rows, not H4. H5 was run in segment three; H2 and H3 remain open, and whether to resume them is the main agent's decision.

### Segment three (H5): procedure, artifact and subject tree

Artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out`: 19067 lines, ending `E7RESULT name=done emitted=19067`, sha256 `b25c75f5ae5fdba2181ce0fb44962fbe15b69062d709403ba4c81bd9581c3216`. Independent verification: `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out` (28 lines, pure Python; see replay below).

- 26 shards: 24 threshold-history cells (S = 4, 8, 16 × θ = 64, 365, 369, 738 × normal unmount or crash), one “hand to background” control, and a deterministic repeat of the first shard. This machine ran 13 (12 normal-unmount cells and control), the second ran 13 (12 crash cells and repeat). Both used the same program file, sha256 `ac1799e786281c7d2f8335b0358eb9a47d8b27711df7704b50ed755a06f926e5`, occurring 27 times in the artifact (26 `h5_config` and one `config`), with no second value.
```text
     27 binary_sha256=ac1799e786281c7d2f8335b0358eb9a47d8b27711df7704b50ed755a06f926e5
```

- Start procedure (recorded after merging, not added to preregistration): first, the two machines built separately; absolute compilation-tree paths embedded in the binaries made their hashes differ, and the apparatus stopped at merge as registered. All 13 shards on the second host were moved aside and invalidated (sha256 prefix `d3f1a82da397`, retained, not deleted). The second pass reused the local 13 and ran the other 13 on the second host using the identical binary sent from this machine. Long-run script `scripts/experiment-apparatus/e183-sweep-history-shards.sh`, sha256 `80501eb704b4d8a76e4d6027ee3840f557cbd8b15549dd182ac1386fbed15469`.
- Subject tree: binary built from `crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` (sha256 `a17a554d5d8ae7c8b1a9a040e9a0bc54c1648084d3c8ba019ea2245c092f16d8`, 73 unit tests, main agent's reading `test result: ok. 73 passed; 0 failed`). The second check for the “starting prerequisite not met” stop clause (registration §3's `847e0e54…` differs and §12 does not record a new value): at writing time `git diff HEAD -- crates/ | sha256sum` is `d471364a6f08c3f38e8091ea13a038891d81ae64b81122cf107b5ea334383acb`; diff stat ends “202 files changed, 108398 insertions(+), 4742 deletions(-)”. This is the tree at writing, not when the artifact ran. `find crates -name '*.rs' -newer <binary>` lists only `crates/singlefs-verification/src/crash_amplification.rs`, two files under its `tests/`, `common_truncation_crash_streams/mod.rs` and `crates/singlefs-verification/tests/common_compaction_prefix/mod.rs`; `singlefs-core` and `singlefs-format` sources have not changed since the binary.

- Counts of each type of result line in the artifact (verbatim output of `awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' $P | sort | uniq -c`, with `$P` set to the H5 artifact):
```text
      1 name=config
      1 name=discrimination_self_proof
      1 name=done
     25 name=h5_anchor_model
     26 name=h5_cell_summary
     25 name=h5_chain_read_differential
     26 name=h5_config
     75 name=h5_differential
      4 name=h5_geometry_sensitivity
  18852 name=h5_mount
      6 name=h5_point
     24 name=h5_threshold_verdicts
      1 name=verdict
```

Counting them: 26 `h5_config` and 26 `h5_cell_summary` (24 cells, control, repeat); 25 each of `h5_anchor_model` and `h5_chain_read_differential` (control has neither); 75 `h5_differential` (25 shards × 3: first below-threshold mount and two sweep mounts); 24 `h5_threshold_verdicts`, six `h5_point` (three S × two unmount kinds), four `h5_geometry_sensitivity`, and 18852 `h5_mount` rows. No rows or fields are missing: both the empty-field and `not_run` counts are zero.

### Replay (segment three H5)

```bash
python3 -B -I scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_history_anchor_check.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out
bash scripts/experiment-apparatus/replay.sh E183
```

The H5 artifact itself is not registered for byte-for-byte replay: `h5_differential` and `h5_cell_summary` contain `mount_nanoseconds` and `sweep_mount_nanoseconds`, which differ between runs and are not stripped by `strip_timing`; its closing line lacks `loadavg1` and `cpus`, so the timing mode would fail. Registered instead is exact replay of the deterministic independent checker output (`@driver_e183_h5_anchor_check`). The separate T4 deterministic core repeats the first shard on the second host: after removing wall-clock, process high-water mark, shard name and binary hash, all fields match (`repeat_of_the_first_shard_equal_outside_timing=true`). An independent normalized diff on this page also found no difference in 135 lines per side.

### Segment-three readings: row trajectories (R5a) and peak pages (R5b A)

Each of the 24 cells repeatedly mounts until two sweep publications succeed. The 18852 `h5_mount` rows record the apparatus's rows before and after writing, sweep outcome, deleted rows and the model's corresponding values. Counts (P is the H5 artifact):

```bash
grep 'name=h5_mount' $P | grep -o ' k33=[a-z_]*\| k34=[a-z_]*\| a36=[a-z_]*\|trajectory_matches_the_model=[a-z]*' | sort | uniq -c
grep 'name=h5_anchor_model' $P | grep -o 'matches_the_registered_script_output=[a-z]*' | sort | uniq -c
grep 'name=h5_mount' $P | grep -o 'sweep=[a-z_]*' | sort | uniq -c
```

```text
  18802  a36=not_applicable
     50  a36=true
  18852  k33=true
  18802  k34=not_applicable
     50  k34=true
  18852 trajectory_matches_the_model=true
     25 matches_the_registered_script_output=true
    400 sweep=handed_to_the_background
  18402 sweep=not_swept_at_or_below_the_threshold
     50 sweep=rows_deleted
```

- K33 (sweep publication iff rows0 > θ) is true for all 18852 mounts. K34 (deleted rows are exactly those for instances with no root in the root ring) and A36 (zero heads erased) are true for all 50 sweeps (two per cell plus two in the repeat), and not applicable otherwise. All 18852 trajectories match the model, and all 25 `h5_anchor_model` rows match the registered script output; T5 did not trigger.
- Independent checker `sweep_history_anchor_check.py`, without importing the apparatus or reading `crates/`, uses registration §13 command-one anchor script `anchors_e183.py`, sha256 `d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d`. It checks 24 cells' mount counts, two sweep positions, deleted and remaining rows, peak rows and pages and final txg, plus one control: zero mismatches. Final lines:

```text
E7RESULT name=anchor_check_control shard=h5-control-slots-per-region-8-handed-to-the-background-normal_unmount mounts=400 rows0_is_the_mount_number=True never_swept=True equal=True
E7RESULT name=verdict item=h5_anchor_check cells_checked=24 control_cells_checked=1 mismatches=0
E7RESULT name=done emitted=28
```

  `grep -c 'equal=True'` gives 25. Altering one cell's `rows_left` before feeding it to the checker returned 1 and named that cell `equal=False` (draft red test, not saved).
- A34 (peak rows0 = θ + 1; peak pages 1 at θ = 64, 365, 2 at 369 and 3 at 738) matches all 24 cells. A35's registered example for S=8, θ=369, normal unmount is `mounts=736 sweeps=[366, 366] residuals=[4, 4] mounts_at_sweep=[370, 736]`; artifact line 7929 is:

```text
E7RESULT name=h5_anchor_model shard=h5-slots-per-region-8-threshold-369-normal_unmount model_mounts=736 model_sweeps_mount_deleted_left=370:366:4,736:366:4 model_final_txg=4418 matches_the_registered_script_output=true
```

R5a and R5b A across all 24 cells. Summary line is the `h5_cell_summary` line number; the two sweep byte columns are `sweep_write_bytes` and `sweep_read_bytes`, not R5c/R5d results under V35. The table was read from the artifact with `python3 -I -B scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_cell_table.py $P table`:

| S | Unmount | θ | Summary line | Mounts | Peak rows | Peak pages | R5b A | Sweeps | Final rows | Sweep write bytes | Sweep read bytes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 4 | crash | 64 | 270 | 127 | 65 | 1 | not crossing | 2 | 3 | 344576 / 344576 | 436109312 / 436107264 |
| 4 | crash | 365 | 1743 | 729 | 366 | 1 | not crossing | 2 | 3 | 344576 / 344576 | 436109312 / 436107264 |
| 4 | crash | 369 | 3232 | 737 | 370 | 2 | crossing | 2 | 3 | 344576 / 344576 | 436205568 / 436201472 |
| 4 | crash | 738 | 6197 | 1475 | 739 | 3 | crossing | 2 | 3 | 344576 / 344576 | 436291584 / 436287488 |
| 4 | normal | 64 | 136 | 128 | 65 | 1 | not crossing | 2 | 2 | 344576 / 344576 | 435462144 / 435462144 |
| 4 | normal | 365 | 1007 | 730 | 366 | 1 | not crossing | 2 | 2 | 344576 / 344576 | 435462144 / 435462144 |
| 4 | normal | 369 | 2488 | 738 | 370 | 2 | crossing | 2 | 2 | 344576 / 344576 | 435558400 / 435558400 |
| 4 | normal | 738 | 4715 | 1476 | 739 | 3 | crossing | 2 | 2 | 410112 / 344576 | 435718144 / 435652608 |
| 8 | crash | 64 | 6460 | 123 | 65 | 1 | not crossing | 2 | 7 | 344576 / 344576 | 437692416 / 437690368 |
| 8 | crash | 365 | 7927 | 725 | 366 | 1 | not crossing | 2 | 7 | 344576 / 344576 | 437692416 / 437688320 |
| 8 | crash | 369 | 9410 | 733 | 370 | 2 | crossing | 2 | 7 | 344576 / 344576 | 437788672 / 437786624 |
| 8 | crash | 738 | 12369 | 1471 | 739 | 3 | crossing | 2 | 7 | 410112 / 344576 | 437932032 / 437864448 |
| 8 | normal | 64 | 6330 | 126 | 65 | 1 | not crossing | 2 | 4 | 344576 / 344576 | 435480576 / 435480576 |
| 8 | normal | 365 | 7195 | 728 | 366 | 1 | not crossing | 2 | 4 | 344576 / 344576 | 435480576 / 435480576 |
| 8 | normal | 369 | 8670 | 736 | 370 | 2 | crossing | 2 | 4 | 344576 / 344576 | 435576832 / 435576832 |
| 8 | normal | 738 | 10891 | 1474 | 739 | 3 | crossing | 2 | 4 | 410112 / 344576 | 435736576 / 435671040 |
| 16 | crash | 64 | 12620 | 115 | 65 | 1 | not crossing | 2 | 15 | 410112 / 344576 | 442494976 / 442429440 |
| 16 | crash | 365 | 14075 | 717 | 366 | 1 | not crossing | 2 | 15 | 344576 / 344576 | 442429440 / 442427392 |
| 16 | crash | 369 | 15546 | 725 | 370 | 2 | crossing | 2 | 15 | 410112 / 344576 | 442591232 / 442523648 |
| 16 | crash | 738 | 18493 | 1463 | 739 | 3 | crossing | 2 | 15 | 344576 / 344576 | 442585088 / 442580992 |
| 16 | normal | 64 | 12498 | 122 | 65 | 1 | not crossing | 2 | 8 | 344576 / 344576 | 435517440 / 435517440 |
| 16 | normal | 365 | 13351 | 724 | 366 | 1 | not crossing | 2 | 8 | 344576 / 344576 | 435517440 / 435517440 |
| 16 | normal | 369 | 14814 | 732 | 370 | 2 | crossing | 2 | 8 | 344576 / 344576 | 435613696 / 435613696 |
| 16 | normal | 738 | 17023 | 1470 | 739 | 3 | crossing | 2 | 8 | 410112 / 344576 | 435773440 / 435707904 |

Interpretation of R5b A (verbatim output of the 24 `name=h5_threshold_verdicts` entries, categorized by θ and peak page count):

```text
      6 threshold_rows=365 peak_chain_pages=1 r5b_reading_one_page=not_crossing
      6 threshold_rows=369 peak_chain_pages=2 r5b_reading_one_page=crossing
      6 threshold_rows=64 peak_chain_pages=1 r5b_reading_one_page=not_crossing
      6 threshold_rows=738 peak_chain_pages=3 r5b_reading_one_page=crossing
```

- At θ=64,365 all 12 cells peak at one page and 65/366 rows, not crossing the one-page bound. At θ=369,738 all 12 peak at 2/3 pages and 370/739 rows, crossing it and adding 4 × 32768 bytes per disk to switch reserve per writable mount (X19). Each θ has the same verdict across six geometry points: four `r5b_stable_across_points=true` and `geometry_unstable_thresholds=none`.
- Peak rows depend only on θ (θ + 1), not S or unmount type; those affect only residual rows: normal S=4/8/16 leaves 2/4/8 and crash leaves 3/7/15. The formula gives θ ≤ 365 for reading A (row term ≤ one page iff rows0 ≤ 366; X17). Measurements show 365 does not cross and 369 does; 366–368 were not measured.

### Segment-three readings: positive controls PC-no-sweep and PC-differential (V34, V35)

**PC-no-sweep passes the revised criterion:** zero sweep publications, rows0 increases by one per mount, final rows0 = 400. The “hand to background” control shard has 400 mounts, all `sweep=handed_to_the_background`, with rows0 equal to mount number. Configuration and summary, lines 18494 and 18895:

```text
E7RESULT name=h5_config shard=h5-control-slots-per-region-8-handed-to-the-background-normal_unmount slots_per_region=8 threshold_rows=none unmount_kind=normal_unmount mounts_at_most=400 device_bytes=4294967296 journal_ring_bytes=805306368 binary_sha256=ac1799e786281c7d2f8335b0358eb9a47d8b27711df7704b50ed755a06f926e5
E7RESULT name=h5_cell_summary shard=h5-control-slots-per-region-8-handed-to-the-background-normal_unmount slots_per_region=8 threshold_rows=none unmount_kind=normal_unmount writable_mounts=400 sweeps_done=0 peak_rows0=400 mounts_with_rows0_above_the_threshold=0 mounts_with_rows0_above_366=34 peak_chain_pages=2 mounts_with_two_or_more_chain_pages=34 final_rows=400 total_write_bytes=757080576 sweep_write_bytes=none sweep_read_bytes=none sweep_mount_nanoseconds=none positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true final_rows0=400 registered_final_rows0=400 original_registered_final_rows0=401 final_rows0_equals_the_registered_value=true
```

Of 400 mounts, 34 have rows0 > 366 and peak pages is 2, with no sweeps. `sweeps_done=0`, `final_rows0=400`, `registered_final_rows0=400` (amended from 401 in registration §12), and `original_registered_final_rows0=401` each appear once. V34 did not trigger; R5a and R5b A yield readings.

**Original PC-differential arm: all 24 cells hit V35, recorded as one failure.** Registration §5.3 originally requires the device write-byte and read-byte differences between the threshold and background versions of the first below-threshold mount both equal zero, and write-byte difference on the crossing mount > 0. Apply literally, without relaxation (registration §12 amendment). Grouped summary fields, excluding repeat and control:

```bash
grep 'name=h5_cell_summary' $P | grep -v 'shard=h5-repeat\|shard=h5-control' | grep -o 'positive_control_differential_below_the_threshold=[a-z]* positive_control_differential_at_the_first_sweep=[a-z]* differential_below_the_threshold_write_bytes=[0-9]* differential_below_the_threshold_read_bytes=[0-9]*' | sort | uniq -c
```

```text
     24 positive_control_differential_below_the_threshold=false positive_control_differential_at_the_first_sweep=true differential_below_the_threshold_write_bytes=0 differential_below_the_threshold_read_bytes=32768
```

The written bytes difference being 0 (for the mount that did not cross) and the written bytes difference for the crossing mount being > 0 hold across all 24 cells; however, the read bytes difference is 32768 and not 0, hitting all 24 cells: the original arm's condition that "the read bytes difference for the mount that did not cross = 0" fails on every single cell, yielding `positive_control_differential_below_the_threshold=false` across all 24 cells. By the literal terms of clause V35: R5c and R5d do not emerge for that cell; since all 24 cells were hit, R5c and R5d do not emerge overall. "The first mount that did not cross the threshold on each cell" is precisely the 1st mount (`rows0 = 1`), with full-line examples (S = 8, θ = 369, normal unmount, H5 artifact lines 7931, 8302, and 8669):

```text
E7RESULT name=h5_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=1 swept=false write_bytes_difference=0 read_bytes_difference=32768 mount_nanoseconds=3492017
E7RESULT name=h5_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=370 swept=true write_bytes_difference=344576 read_bytes_difference=435576832 mount_nanoseconds=92570354
E7RESULT name=h5_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=736 swept=true write_bytes_difference=344576 read_bytes_difference=435576832 mount_nanoseconds=76328517
```

**New PC-differential-chain-read arm, added by amendment and judged separately: all 24 pass.** Its criterion is read-byte difference equals bytes read for the instance-table chain on the threshold mount = pages × 32768, pages computed from that mount's model rows0 (1 implies 32768). Excluding the repeat:

```bash
grep 'name=h5_chain_read_differential' $P | grep -v 'shard=h5-repeat' | grep -o 'read_bytes_difference_equals_the_chain_read=[a-z]*' | sort | uniq -c
```

```text
     24 read_bytes_difference_equals_the_chain_read=true
```

Example for the same cell, line 7932:

```text
E7RESULT name=h5_chain_read_differential shard=h5-slots-per-region-8-threshold-369-normal_unmount mount=1 read_bytes_difference=32768 model_rows0=1 expected_chain_pages=1 expected_chain_read_bytes=32768 read_bytes_difference_equals_the_chain_read=true
```

The verdict reports `chain_read_differential_failed_cells=0` and `differential_control_failed_cells=24` separately.

Separate measurement from inference:

- Measured: on first below-threshold mount, threshold mode reads 32768 bytes more than background mode, one instance-table-chain page, in every cell; the new arm holds 24/24. Of the 50 sweep mounts, 44 have a 344576-byte write difference and six a 410112-byte difference:
```text
     44 write_bytes_difference=344576
      6 write_bytes_difference=410112
```
- Inferred, not established by this page's measurements: the threshold mount counts rows along the chain at the end and reads 32768 bytes; the background mode returns at the switch decision without reading. The new arm establishes only that the difference equals chain-read bytes, not which side read.
- Failure pattern: the original arm assumed the zero difference for all cells, but the threshold version must count rows at mount end (the inference above); this shared assumption fails across all θ arms and does not discriminate between them. The first pattern in `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` is “hit does not discriminate arms”: separately repair the shared assumption. This page records one loss under registration, neither replacing the original arm with the new one nor treating its read half as expected.

### Segment-three readings: R5c and R5d unavailable, discriminative power self-test, G5 and geometry sensitivity

**R5c and R5d unavailable (V35, all 24 cells).** Verbatim output of the categorized `r5c=` values across all 24 `name=h5_threshold_verdicts` lines, together with whole-line examples (S = 8, θ = 64 and θ = 369, normal unmount, H5 artifact lines 19041 and 19043):

```bash
grep 'name=h5_threshold_verdicts' $P | grep -o 'r5c=[a-z_0-9]*' | sort | uniq -c
```

```text
     24 r5c=void_v35
E7RESULT name=h5_threshold_verdicts cell=h5-slots-per-region-8-threshold-64-normal_unmount slots_per_region=8 unmount_kind=normal_unmount threshold_rows=64 peak_chain_pages=1 r5b_reading_one_page=not_crossing writable_mounts=126 total_write_bytes=237512192 ratio_to_the_smallest_write_per_mount_permille=none r5c=void_v35
E7RESULT name=h5_threshold_verdicts cell=h5-slots-per-region-8-threshold-369-normal_unmount slots_per_region=8 unmount_kind=normal_unmount threshold_rows=369 peak_chain_pages=2 r5b_reading_one_page=crossing writable_mounts=736 total_write_bytes=1388854784 ratio_to_the_smallest_write_per_mount_permille=none r5c=void_v35
```

The write amplification ratio for R5c (`ratio_to_the_smallest_write_per_mount_permille`) is `none` across all 24 cells. In the artifact, each cell still recorded the written bytes and read bytes for the two sweeps (the 24-cell reading table in the "Third segment readings: row-count trajectory (R5a) and peak page counts (R5b A)" section) and the device write bytes across the entire history; under V35, these are not treated as the results of R5c and R5d.

**Discriminative power self-test (registration §8 item ②, R5c) cannot be made: V36.** Artifact line 19065:

```text
E7RESULT name=discrimination_self_proof judgement=h5_r5c_write_amplification real_ratios=0 moved_doubled_boundary_permille=none flipped=0 holds=false
```

`real_ratios=0`: no real R5c ratios exist between which to move 1.10, so `holds=false` and `discrimination_self_proof_holds=false`. This follows from V35, not a second apparatus defect: the unit test `moving_the_acceptance_boundary_between_two_real_ratios_flips_a_verdict` for `acceptance_boundary_moved_between_real_ratios` passes in the main agent's 73-test run when fed [1000, 1050, 1300]. The actual artifact lacks ratios to supply. V36 therefore withholds R5c geometry sensitivity.

**G5 is neither triggered nor cleared.** Verdict `g5_points_without_a_candidate=6` and six `h5_point` lines:

```text
E7RESULT name=h5_point slots_per_region=4 unmount_kind=normal_unmount cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=4 unmount_kind=crash cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=8 unmount_kind=normal_unmount cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=8 unmount_kind=crash cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=16 unmount_kind=normal_unmount cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
E7RESULT name=h5_point slots_per_region=16 unmount_kind=crash cells=4 thresholds_passing_both=none g5_no_candidate_passes_both=true
```

G5's registered trigger requires every θ either to cross R5b or be unacceptable on R5c. θ=369,738 cross R5b; θ=64,365 do not, and their R5c is `void_v35`, not “unacceptable.” The condition is undecidable. The apparatus counts cells without R5c as failing both gates, hence `thresholds_passing_both=none` at all six points, but this is not the G5 trigger observation. Do not report “none of four candidates meets both”; report only that 369/738 cross the one-page bound, 64/365 do not, and the write-amplification gate has no result under the literal registration.

**Geometry sensitivity**, artifact lines 19061–19064:

```text
E7RESULT name=h5_geometry_sensitivity threshold_rows=64 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
E7RESULT name=h5_geometry_sensitivity threshold_rows=365 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
E7RESULT name=h5_geometry_sensitivity threshold_rows=369 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
E7RESULT name=h5_geometry_sensitivity threshold_rows=738 points=6 r5b_stable_across_points=true r5c_stable_across_points=true
```

The four `r5b_stable_across_points=true` values are readings: R5b agrees across all six points. The four `r5c_stable_across_points=true` are vacuous: all six equal `void_v35`, and the apparatus calls a singleton value set stable. V36 withholds R5c geometry sensitivity, so do not cite those as results.

**Division possible from the artifact (not an R5c result; only to inform whether the main agent should register a new arm).** W_θ (registration X21 ②) = total history device writes ÷ writable mounts, from `total_write_bytes` and `writable_mounts`, independent of PC-differential. V35 withholds all of R5c, so this division is not R5c and must not be judged against 1.10. Output of `python3 -I -B scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_cell_table.py $P write-per-mount`:

```text
S=4 crash: θ=64:每次挂载写942657字节:相对最小1005千分之一 θ=365:每次挂载写937965字节:相对最小1000千分之一 θ=369:每次挂载写938204字节:相对最小1000千分之一 θ=738:每次挂载写970459字节:相对最小1035千分之一
S=4 normal_unmount: θ=64:每次挂载写1885052字节:相对最小1000千分之一 θ=365:每次挂载写1886850字节:相对最小1001千分之一 θ=369:每次挂载写1887032字节:相对最小1001千分之一 θ=738:每次挂载写1919989字节:相对最小1019千分之一
S=8 crash: θ=64:每次挂载写930903字节:相对最小1005千分之一 θ=365:每次挂载写926363字节:相对最小1000千分之一 θ=369:每次挂载写926552字节:相对最小1000千分之一 θ=738:每次挂载写958993字节:相对最小1035千分之一
S=8 normal_unmount: θ=64:每次挂载写1885017字节:相对最小1000千分之一 θ=365:每次挂载写1886849字节:相对最小1001千分之一 θ=369:每次挂载写1887031字节:相对最小1001千分之一 θ=738:每次挂载写1920033字节:相对最小1019千分之一
S=16 crash: θ=64:每次挂载写944836字节:相对最小1001千分之一 θ=365:每次挂载写947102字节:相对最小1004千分之一 θ=369:每次挂载写943719字节:相对最小1000千分之一 θ=738:每次挂载写976771字节:相对最小1035千分之一
S=16 normal_unmount: θ=64:每次挂载写1884945字节:相对最小1000千分之一 θ=365:每次挂载写1886847字节:相对最小1001千分之一 θ=369:每次挂载写1887030字节:相对最小1001千分之一 θ=738:每次挂载写1920122字节:相对最小1019千分之一
```

Across four θ at six geometry points, bytes per mount relative to the minimum are 1000–1035‰; θ=738 is largest (1019–1035‰), θ=64–369 at most 1005‰. This mixes mount's own writes (about 1.9 MB normal, 0.9 MB crash) with sweep writes (344576 or 410112 each, two sweeps amortized over hundreds of mounts). Histories differ in length; θ=738 reaches three chain pages, so rewriting the chain on each mount is an inferred source of extra writes, not separately measured. It does not isolate sweep-publication write amplification.

### Segment three: verdict fields individually

Verbatim output of `grep -n 'name=verdict' $P` (the H5 artifact contains only this single verdict line, where `$P` refers to the H5 artifact):

```text
19066:E7RESULT name=verdict item=h5_summary positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true control_final_rows0=400 control_final_rows0_equals_the_revised_registered_value=true differential_control_failed_cells=24 chain_read_differential_failed_cells=0 r5b_crossing_thresholds=369,738 g5_points_without_a_candidate=6 geometry_unstable_thresholds=none discrimination_self_proof_holds=false repeat_of_the_first_shard_equal_outside_timing=true repeat_first_differing_line=none
```

The full line:

```text
E7RESULT name=verdict item=h5_summary positive_control_no_sweep_publish=true positive_control_rows0_plus_one_each_mount=true control_final_rows0=400 control_final_rows0_equals_the_revised_registered_value=true differential_control_failed_cells=24 chain_read_differential_failed_cells=0 r5b_crossing_thresholds=369,738 g5_points_without_a_candidate=6 geometry_unstable_thresholds=none discrimination_self_proof_holds=false repeat_of_the_first_shard_equal_outside_timing=true repeat_first_differing_line=none
```

Fields that did not pass are called out individually; fields whose values are `true` or `zero` and counts whose names do not indicate failure are not called out:

| Field | Artifact line | Value | Why it has this value | Expected by registration? |
|---|---|---|---|---|
| `differential_control_failed_cells=24` | 19066 | 24 | The original arm of PC-differential hit across all 24 cells: on the 1st mount that did not cross the threshold, the read byte difference is 32768, not 0 (in the "Third segment readings: positive controls PC-no-sweep and PC-differential (V34, V35)" section, `positive_control_differential_below_the_threshold=false` across 24 cells, `differential_below_the_threshold_read_bytes=32768`) | The registration amendment derived that "the read byte difference is not 0" and explicitly stated that hitting it records one loss (under V35, R5c and R5d are unavailable); not the expectation of the original registration |
| `discrimination_self_proof_holds=false` | 19066; `holds=false` at 19065 | false | No real R5c ratios to move the threshold between. | Consequence of V35; V36 withholds R5c geometry sensitivity. |
| `g5_points_without_a_candidate=6` | 19066; six `h5_point` lines at 19035, 19040, 19045, 19050, 19055, 19060 | 6 | Apparatus counts cells lacking R5c as having no candidate; θ=64,365 do not cross R5b at each point but lack R5c. | Not G5's trigger observation: G5 neither triggered nor cleared. |
| `r5b_crossing_thresholds=369,738` | 19066 | 369,738 | List, not failure flag: peak pages 2 and 3 cross the one-page bound. | Expected (A34); listed because its failure status is ambiguous. |

Fields outside the verdict line in the artifact whose values appear like failures: `positive_control_differential_below_the_threshold=false` (in `h5_cell_summary` lines, across the 24 cells plus the first-slice rerun for 25 lines total, which is the exact same issue as the `differential_control_failed_cells=24` row in the field-by-field verdict table); `r5c=void_v35` (in 24 `h5_threshold_verdicts` lines, between lines 19031 and 19059); `holds=false` (line 19065, corresponding to the `discrimination_self_proof_holds=false` row in the field-by-field verdict table). `g5_no_candidate_passes_both=true` (in 6 `h5_point` lines) has a name resembling failure but its value is true; its origin is detailed in the `g5_points_without_a_candidate=6` row of the field-by-field verdict table.

The remaining verdict values `positive_control_no_sweep_publish=true`, `positive_control_rows0_plus_one_each_mount=true`, `control_final_rows0_equals_the_revised_registered_value=true`, `geometry_unstable_thresholds=none`, `repeat_of_the_first_shard_equal_outside_timing=true` and `repeat_first_differing_line=none` pass; `control_final_rows0=400` is the revised criterion and `chain_read_differential_failed_cells=0`. No `not_run` or empty fields.

### Segment three: invalidation, failure and stop clauses

| Clause | Status | Reading |
|---|---|---|
| Starting prerequisite not met (stop) | Pass (current subject-tree reading is in the "Third segment (H5): execution, artifact, and subject tree" section) | The third segment's launch method is added retrospectively under the "Launch method" note in that H5 section, not placed in preregistration |
| T4 deterministic core | Pass | First shard and repeat, on different machines, match field by field over 135 normalized lines; verdict reports equality and no first difference; independent diff also found none. |
| T5 H5 trajectory differs from anchor | Not triggered | K33 true for 18852; K34/A36 true for 50 sweeps; all trajectories and 25 model rows match; independent check finds zero mismatches in 24 cells plus control. |
| V34 PC-no-sweep | Not triggered | Control: 400 mounts, no sweeps, final rows0=400 (amended criterion). |
| V35 PC-differential | **All 24 hit** | Original arm read difference 32768 ≠ 0; R5c/R5d unavailable for every cell; new chain-read arm passes all 24 and is reported separately. |
| V36 discriminative power self-test | **Triggered for R5c** | `real_ratios=0 holds=false`; R5b stability is measured, R5c stability vacuous. |
| V37 unavailable not treated as zero | Not triggered | No empty or `not_run` fields. |
| V30 PC-heap | Not applicable | H5 measures device reads/writes and rows, not heap; `peak_over_baseline` count zero. |
| G5 | Neither triggered nor cleared | See the section "Third segment readings: R5c and R5d unavailable, discriminative-power self-test, G5, and geometry sensitivity" |
| G1–G4, G6–G8, T2, T3, C0 patch hits not once or A41 mismatch (stop), V31–V33 | Not applicable | Belong to segments one, two and four. |

### Fork table (question sheet H5; registration §11 requires all R5a cells and R5b and R5c verdicts)

P is `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out`; T is `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs`.

| Question row | Current status | What is still missing | Could the remaining measurements reverse it? | Command used to compute it |
|---|---|---|---|---|
| H5 · R5a row trajectory | Available: all 24 trajectories, peak rows θ+1 (65/366/370/739), agree with anchor. | None | All registered cells ran; nothing remaining to reverse it. | `python3 -I -B $T/sweep_cell_table.py $P table`; `python3 -B -I $T/sweep_history_anchor_check.py $T/anchors_e183.py $P` |
| H5 · R5b A | Available: θ=64,365 do not cross; 369,738 cross (peak pages 2,3); consistent at six points. | None | No; deterministic, first shard repeated identically. | Group `h5_threshold_verdicts` by threshold, peak pages and `r5b_reading_one_page`. |
| H5 · R5c write amplification | Unavailable: V35 hits all 24. | R5c verdict; under literal registration, a new registration and 24-cell rerun with additional arm or revised criterion. | No registered cells remain; same apparatus rerun hits again. | Group `positive_control_differential_below_the_threshold` and `r5c` fields. |
| H5 · R5d sweep reads | Unavailable, same V35; artifact still prints 435462144–442591232 bytes per sweep. | Same as above | Same | Same |
| H5 · Geometry sensitivity and self-test | R5b stable; R5c and self-test unavailable (V36). | Same as R5c | Same | `grep 'name=h5_geometry_sensitivity\|name=discrimination_self_proof' $P` |
| H5 · Entire question | **Insufficient** under registration §11 because R5c unavailable. The question sheet's original criterion asks for histories through threshold at ≥3 candidate thresholds and peak rows and sweep-publication writes: all four candidates × six points ran and values exist, but writes are from the V35-hit differential. | Main agent decides sufficiency against registration and question sheet; a new registration is needed for write-amplification verdict. | No registered measurements remain; only new registration and rerun can change R5c. | H5 rows above. |

### Rerun 2 (first run of segment two H2: sharded through the unified task service, X3 now reads the process high-water mark)

Rerun registration `research/prompts/e183-r2-prereg.md` (sha256 `76257619fe9f7a1da89a433fe794ab345fc7a8ac3230f8e57e57e1fb6af3119a`; its section 12 amendments 12.1 to 12.3 were all written before this segment's artifact). The previous registration and question sheet that it builds on have been moved out of the working tree; retrieve them with `git show 59ce5a865^:research/prompts/e183-preregistration.md` (sha256 `c4609ca35a24704e5d4e52b2fe92503cda2d73efe18c39d503a3d5cd4e1a7f21`) and `git show 59ce5a865^:research/prompts/m3-close-cost-measurement-questions.md` (sha256 `c5dec701293cb41b2f3a7b291b8e85a4e3f5e837c60024415f21c3023e1ba2d5`). This segment answers only the H2 row of the question sheet: the pre-change baseline of C567 (allocation and publish costs linear in disk capacity), that is, the slopes of the time and peak memory of one allocation and one publish against disk capacity and against record count. There is only one arm (today's code, candidate D); 9 cells (capacity ladder 4 GiB, 64 GiB, 1 TiB, 4 TiB and 16 TiB − 1 MiB with n = 0; record cells at 4 GiB with n = 1024 and 10240; occupied cells at 64 GiB and 1 TiB with n = 10240) × 5 rounds, one process per cell per round; the disks are file-backed sparse images (two of equal size, L = 768 MiB, S = 8).

Procedure and prerequisites (all checked fresh for this segment):

- All five start prerequisites hold: the previous registration and question sheet can be retrieved as described and their `sha256sum` matches the registration (819 lines and 13 lines); the C567 row is at line 345 of `.claude/kb/checks-owed.md`, before “已还清” (paid off) at line 613, so it is still open; in `crates/singlefs-core/src/allocator.rs`, `pub fn lowest_user_data_slot(` (line 955) and `fn make_room_for_record_on_device(` (line 1144) each count 1 with `grep -c`, and `crates/singlefs-core/src/transaction.rs` has 2 occurrences of `let allocator_before_this_publish = allocator.clone();`; the run script counts 0 for `grep -c 'run-with-memory-cap\|capped\.sh\|nice -n'` and the apparatus counts 0 for `grep -c 'WRAPPER_EXIT'`; line 47 of `.claude/gate.d/stage-inputs.tsv` has the key `E183/h2`; the two unsettled items are covered by `修-1` and `修-2` in registration 12.1. `allocator.rs` was changed by another patch in this batch (`git diff HEAD --stat` shows 151 lines added and removed); the changed hunks are at `PlacementOnDevice`, root ring occupancy and `PoolAllocator`, and each of the two anchor functions still occurs exactly once, so under the registration this run is not voided.
- Subject tree: HEAD `59ce5a865`. The `git diff HEAD -- crates/ | sha256sum` at the moment the run started was not recorded (the item that the end of 12.1 asks for; the part that ran did not write it into the registration, and the registration is not changed after the artifact ran); taken fresh on the day the artifact was read, it is `c9e492c4a6c026fa44773dd865c5f380776dab6049e0109b9542a60687277595`, and the last line of `--stat` is “339 files changed, 89908 insertions(+), 16735 deletions(-)”. `find crates/singlefs-core crates/singlefs-format -name '*.rs' -newer <program file in the run directory>` finds 0 files, so the two packages under test have not changed since the program file was built; the apparatus source file's sha256 is now `eff3e9dd41ff88f187e2b011b27e1301e438827c35d06debfdacafe1bd0b2a87`, and it is not newer than the program file either.
- Procedure: the main agent submitted one `task-server/task submit -- bash scripts/experiment-apparatus/e183-capacity-cells.sh run` (task 5382, outcome `type=succeeded exit=0`), which the service split into a plan, 5 shards and a merge. The plan (subtask 5383) took three attempts: the two on this machine with 4096M and 6144M exited 250 (hit the cap), and the third, on the second host with 9216M, exited 0, passed the acceptance check and built the program file with sha256 `9016bb927d45ec09730735c405753921de6bf7f6075c3fde64fd2030224729e9` (235792472 bytes). The round-1 shard (subtask 5429) exited 250 on its first five attempts on this machine as the cap was raised from 4096M to 20736M, finishing no cell (3 to 13 seconds each), and its sixth attempt, on the second host with 31104M, finished all 9 cells; each of the other four shards finished in one attempt. Rounds and hosts: rounds 1, 2 and 4 on the second host, rounds 3 and 5 on this machine; rounds 3 and 5 on this machine started the program file built on the second host, the `h2 list-shards` guard passed, and registration unsettled item 2, “one binary starts on both hosts”, was measured to hold this time. The merge (subtask 5470) ran on the second host (registration 5.3 says the merge runs on this machine; the service does not pin a host, and the merge reads only the shard files in the run directory, so its verdicts do not depend on the host).
- Wall clock: for the attempt in which each of the five shards finished, the service recorded 121.9, 119.7, 135.9, 119.7 and 137.8 seconds; registration section 14 estimated 30 to 50 minutes per shard and 17 to 34 minutes for the occupied cell 1 TiB n = 10240 (Q14), both far above the measurement (that cell took 5 to 7 seconds in the service log).
- Artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out`: 949 lines, sha256 `c07d13a209d8a4c178d52d16000cba1f63599b06de9d0d0750f17c8460088bd7`; line 1 is the acceptance header `E7INPUT name=input_fingerprint key=E183/h2 sha256=dd3b75e7b8db0f070ee139ff62ea0b0fd9a3b5e9a7dc1f638351f3dadaad1dea units=4476`, the last line is `E7RESULT name=done emitted=948`, and the artifact has 0 `name=stop` lines.
- The run directory `<work root>/e183-h2-run/dd3b75e7b8db0f07/` also holds a 1.2G `target/`: the plan's first two attempts, compiling on this machine, were killed at the cap, so the run script's step “delete the build directory it created once the build is done” was never reached; it does not affect the artifact (the program file is the one built on the second host in the third attempt).

Replay (segment two H2):

```bash
# Repository root; really creates sparse images from 4 GiB to 16 TiB − 1 MiB (process high-water mark per cell up to about 22 GiB), submitted to the service by the main agent; with no arguments it runs all cells;
# the service splits it into a plan, 5 shards and a merge per the task-server-shards declaration in the script header. While the acceptance key E183/h2 is unchanged, the plan step exits 77 (an artifact with the same key exists)
task-server/task submit -- bash scripts/experiment-apparatus/e183-capacity-cells.sh run

# List whether each cell of each round is done; merge only the shards already in the run directory (without the service splitting the run)
bash scripts/experiment-apparatus/e183-capacity-cells.sh --list-items
bash scripts/experiment-apparatus/e183-capacity-cells.sh --merge-only --run-directory <run directory>
```

This run's run directory (shard files and per-cell logs, 1.5G together with the `target/` above) is deleted, per registration 5.3 “after the run”, once the artifact is on disk and this page is written; after deletion `--merge-only` has nothing to work on for this run, and checking it again needs a whole new run.

Segment two is not in the replay table of `scripts/experiment-apparatus/replay.sh` (registration 5.6 item 2: one run is split into 5 shards through the service and may go to both hosts, so the whole run does not fit in one task of the replay table; the artifact carries wall-clock fields and cannot be compared byte for byte), and no row is added to `.claude/gate.d/replay-comparison-units.tsv` (registration section 11 “comparison target”: segment two has no earlier artifact to compare with).

Count of each kind of result line in the artifact (verbatim output of `awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out | sort | uniq -c`):

```text
      1 name=config
      2 name=discrimination_self_proof
      1 name=done
    225 name=h2_allocate
     45 name=h2_allocator_copy
      9 name=h2_cell_determinism
      9 name=h2_cell_values
     18 name=h2_cell_values_by_host
     13 name=h2_class
     45 name=h2_config
      1 name=h2_largest_disk
      2 name=h2_occupancy_sensitivity
     45 name=h2_payload_constant
     45 name=h2_process
    225 name=h2_publish
      5 name=h2_round_hosts
      5 name=h2_service_attempt
     45 name=h2_shard_status
     65 name=h2_slope
     45 name=h2_steps
      1 name=h2_threshold_dependence
      3 name=h2_threshold_sensitivity
      1 name=input_fingerprint
     45 name=positive_control_flat
      1 name=positive_control_flat_slope
     45 name=positive_control_heap
      1 name=positive_control_linear
      1 name=verdict
```

Counting the registered “arm × cell × round”: one arm × 9 cells × 5 rounds = 45 processes, each with one `h2_config`, `h2_steps`, `h2_process`, `h2_shard_status`, `h2_allocator_copy`, `h2_payload_constant`, `positive_control_heap` and `positive_control_flat`, 45 of each; each process does 5 allocations and 5 publishes, 225 of each; 13 slope rows × 5 rounds = 65 `h2_slope`; each of the 9 cells has one `h2_cell_values` and one `h2_cell_determinism`, plus one `h2_cell_values_by_host` per host (9 × 2 hosts = 18); one `h2_service_attempt` for each of the 5 shards and one `h2_round_hosts` for each of the 5 rounds. The counts add up and no lines are missing; no cell-round lacks the high-water mark in `h2_process` (`=none` in the artifact appears only in the slopes of the two R2e publish rows, in `differing_fields` of the determinism check, in the empty classes of the largest size and in `largest_just_beyond_mebibytes` of discriminative power self-test ④, all of them empties the registration expects).

### Segment two: verdicts (full lines copied from `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out`, lines 849, 853, 864, 943, 944, 945, 946, 947, 948)

```text
E7RESULT name=h2_largest_disk threshold_bytes=25769803776 lower_edge_bytes=25702694912 completed_capacities_mebibytes=4096,65536,1048576,4194304,16777215 completed_count=5 largest_completed_mebibytes=16777215 largest_is_the_ladder_endpoint=true beyond_the_memory_threshold_mebibytes=none undeterminable_mebibytes=none filesystem_refused_mebibytes=none near_the_threshold_at_or_above_the_largest=none fewer_than_four_capacities=false
E7RESULT name=h2_threshold_dependence largest_at_half_mebibytes=4194304 largest_at_registered_mebibytes=16777215 largest_at_double_mebibytes=16777215 depends_on_the_threshold=true switches_between=half_and_registered
E7RESULT name=discrimination_self_proof judgement=h2_largest_disk capacity_mebibytes=4096 just_within_threshold_bytes=138219520 largest_just_within_mebibytes=4096 largest_just_beyond_mebibytes=none synthetic_high_water_marks_fall_on_two_sides=true used=real_high_water_marks holds=true
E7RESULT name=positive_control_linear copy_bytes_slope_at_least_0_8_every_round=true copy_time_slope_at_least_0_8_every_round=true
E7RESULT name=positive_control_flat_slope slope_below_0_2_every_round=true
E7RESULT name=h2_occupancy_sensitivity row=r2a near_empty_class=sublinear occupied_class=basically_unrelated agreement=unstable_depends_on_occupancy
E7RESULT name=h2_occupancy_sensitivity row=r2b near_empty_class=linear occupied_class=x5_as_registered_cannot_be_made agreement=not_judged_x5_as_registered_cannot_be_made
E7RESULT name=discrimination_self_proof judgement=h2_slope_class real_slopes=35 moved_boundary=0.812 flipped=1 synthetic_slopes_fall_in_two_classes=true used=real_slopes holds=true
E7RESULT name=verdict item=h2_summary r2a_class=sublinear r2b_class=linear r2d_allocate_class=basically_unrelated r2d_publish_class=linear r2e_allocate_class=sublinear r2e_publish_class=x5_as_registered_cannot_be_made r2e_allocate_peak_class=basically_unrelated r2e_publish_peak_class=x5_as_registered_cannot_be_made r2c_class=linear completed_capacities=5 fewer_than_four_capacities=false positive_control_linear_copy_bytes_holds=true positive_control_linear_copy_time_holds=true positive_control_flat_holds=true heap_control_failed_shards=0 unstable_slope_rows=none predictions_contradicted=r2a_not_basically_unrelated,r2e_allocate_not_linear discrimination_self_proof_holds=true hidden_state_cells=0 cell_rounds_within_the_threshold=45 cell_rounds_beyond_the_memory_threshold=0 cell_rounds_undeterminable=0 cell_rounds_filesystem_refused=0 cell_rounds_voided_high_water_mark=0 cell_rounds_near_the_threshold=0 ladder_rounds_across_hosts=0 unit_area_slots_differing_cell_rounds=0 largest_depends_on_the_threshold=true threshold_discrimination_self_proof_holds=true
```

Verdict line fields called out one by one (`grep -n 'name=verdict' <artifact>` gives only one line, line 948):

- `fewer_than_four_capacities=false` (line 948; the field of the same name on line 849 has the same value): the value is the boolean false, meaning that “fewer than four capacities finished” does not hold, so G6 did not trigger; expected by the registration.
- `r2e_publish_class=x5_as_registered_cannot_be_made` and `r2e_publish_peak_class=x5_as_registered_cannot_be_made` (line 948): not `not_run`, and called out because it is unclear whether they count as “did not pass”: the registered X5 (overwrite 4096 bytes within data unit 0) cannot be made on a file with n units, and the 2026-10-19 amendment in section 12 of the previous registration settled that these two rows are left without a class rather than produced, so this is a no-class outcome the registration expects, not 0 and not “basically unrelated”.
- `predictions_contradicted=r2a_not_basically_unrelated,r2e_allocate_not_linear` (line 948): neither a boolean nor a count, so called out because it is unclear: the prediction of Q8 in section 4 of the previous registration, “R2a basically unrelated, R2e allocation linear in record count”, is overturned by the measured classes, and G2 triggers (see the clauses item by item below); the clause anticipated this kind of observation, so it is not an apparatus error.
- The remaining counts (`heap_control_failed_shards`, `hidden_state_cells`, `cell_rounds_beyond_the_memory_threshold`, `cell_rounds_undeterminable`, `cell_rounds_filesystem_refused`, `cell_rounds_voided_high_water_mark`, `cell_rounds_near_the_threshold`, `ladder_rounds_across_hosts`, `unit_area_slots_differing_cell_rounds`) are all 0; the verdict line has no `not_run`.
- Discriminative power self-test ④ (line 864) uses the real high-water marks: with the lower edge moved onto the maximum high-water mark of the 5 rounds of the 4 GiB cell (`138219520 − 67108864 = 71110656` bytes), the largest size is 4 GiB, and moving it 1 byte lower leaves no size at all, so the size changes; in addition, the threshold sensitivity (line 853) really changes size between T ÷ 2 and T (4 TiB and 16 TiB − 1 MiB).

### Segment-two readings: slope classes (R2a, R2b, R2c, R2d, R2e) and positive controls

The b of each round is taken from artifact lines 865 to 941 (`grep -n 'name=h2_slope' <artifact>`), and the classes from the `name=h2_class` lines at 870 to 942; the class thresholds are those of section 6 of the previous registration (b ≥ 0.8 linear, 0.2 ≤ b < 0.8 sublinear, b < 0.2 basically unrelated; a row gets a class only when all 5 rounds fall in that class).

| Row | Quantity | Axis | b of rounds 1 to 5 | Class |
|---|---|---|---|---|
| R2a | Time of one allocation | Capacity (five sizes) | 0.284, 0.258, 0.322, 0.309, 0.251 | Sublinear |
| R2b | Time of one publish | Capacity | 0.813, 0.811, 0.875, 0.812, 0.830 | Linear |
| R2d allocation | Peak extra memory of one allocation | Capacity | 0.000 (5 rounds) | Basically unrelated (112 bytes each time) |
| R2d publish | Peak extra memory of one publish | Capacity | 1.022 (5 rounds) | Linear |
| R2c | Allocator copy bytes | Capacity | 1.023 (5 rounds) | Linear (reported only, not a criterion) |
| R2e allocation | Time of one allocation | Record count (three sizes) | 0.591, 0.523, 0.591, 0.646, 0.475 | Sublinear |
| R2e allocation peak | Peak extra memory of one allocation | Record count | −0.000 (5 rounds) | Basically unrelated |
| R2e publish, R2e publish peak | — | Record count | Not produced (`points=0`) | `x5_as_registered_cannot_be_made` |
| PC-linear (copy time) | Allocator copy time | Capacity | 1.306, 1.306, 1.330, 1.311, 1.333 | Linear; both halves of PC-linear pass |
| PC-flat | Peak extra memory of a discarded 1 MiB allocation | Capacity | 0.000 (5 rounds) | Basically unrelated; PC-flat passes |
| Geometry sensitivity R2a (occupied cells) | Time of one allocation | Capacity (4 GiB, 64 GiB, 1 TiB, n = 10240) | 0.039, 0.029, 0.060, 0.035, 0.103 | Basically unrelated; differs from the sublinear class on near-empty disks, recorded as “unstable (depends on occupancy)” |
| Geometry sensitivity R2b (occupied cells) | — | — | Not produced | `x5_as_registered_cannot_be_made` |

Close to the thresholds: in three of the five rounds R2b's b is 0.811 to 0.813, less than 0.02 above 0.8, the lower edge of linear; R2a's five rounds are 0.251 to 0.322, and the closest round is 0.051 above 0.2. Classes are judged against the thresholds as written, and no row straddles a threshold (`unstable_slope_rows=none`).

Bytes per slot per disk for R2c (`bytes_per_slot_per_device_millionths` of `name=h2_allocator_copy`; with `grep -o 'name=h2_allocator_copy .*millionths=[0-9]*' <artifact> | grep -o 'millionths=.*' | sort | uniq -c`, every value is the same across the 5 rounds): the five ladder sizes give 3.634529, 3.625487, 3.625033, 3.625008 and 3.625002, all within A37's [3.5, 4.0]; among the record and occupied cells, 4 GiB n = 1024 gives 3.759416, 4 GiB n = 10240 gives 4.822029 (outside the interval), 64 GiB n = 10240 gives 3.686226 and 1 TiB n = 10240 gives 3.628789. The 3.625 inferred by A37 is the bitmap and the segment counts and does not include the record table; the extra amount on small disks with many records is inferred to be the record table (inferred, not measured separately).

Slots between the start of the unit area and the placement returned by one allocation (`slots_from_unit_area_start` of `name=h2_allocate`; supplementary, explains R2a): all 225 allocations in the 9 cells give 2, including the three cells with n = 10240. The inference in 8.2 of the previous registration, “in the record cell with n = 10240 the first free slot is about 20 thousand slots from the start”, did not hold: occupied cells and near-empty disks have the same placement; the difference in allocation time on occupied cells comes from the record count (at 4 GiB, with 94, 2264 and 21034 records the allocation medians are 640, 1810 and 12260 nanoseconds, `name=h2_cell_values` lines 815, 835 and 839), not from the first free slot being farther from the start. So the “depends on occupancy” reading of the geometry sensitivity row should be understood as “depends on record count”.

### Segment-two readings: absolute values (R2f, by host)

`name=h2_cell_values_by_host` (lines 813 to 846) and `name=h2_cell_values` (marked “across hosts”, lines 815 to 847); this machine ran 2 rounds (rounds 3 and 5) and the second host 3 rounds (rounds 1, 2 and 4), so neither host has 5 rounds and the medians below are not called “5-round medians” (G11). Times are in nanoseconds, heap in bytes.

| Cell (capacity / units) | Host (rounds) | Allocation median (min to max) | Publish median (min to max) | Median peak extra memory of publish | Allocator copy bytes |
|---|---|---|---|---|---|
| 4 GiB / 0 | This machine (2) | 740 (380 to 3060) | 5085880 (4687854 to 7278243) | 3111920 | 1540808 |
| 4 GiB / 0 | Second host (3) | 460 (320 to 2530) | 6220286 (6139425 to 6286766) | 3111920 | 1540808 |
| 4 GiB / 0 | Across hosts (5) | 640 (320 to 3060) | 6219807 (4687854 to 7278243) | 3111920 | 1540808 |
| 64 GiB / 0 | This machine (2) | 1690 (1440 to 5100) | 16512061 (14824776 to 20534581) | 60128240 | 30048968 |
| 64 GiB / 0 | Second host (3) | 2060 (1370 to 5660) | 15576916 (15373824 to 19886910) | 60128240 | 30048968 |
| 64 GiB / 0 | Across hosts (5) | 1880 (1370 to 5660) | 15577281 (14824776 to 20534581) | 60128240 | 30048968 |
| 1 TiB / 0 | This machine (2) | 4470 (3420 to 7770) | 357879878 (353842369 to 475754672) | 972392556 | 486180008 |
| 1 TiB / 0 | Second host (3) | 3810 (2540 to 9510) | 291380102 (287646092 to 394520740) | 972392556 | 486180008 |
| 1 TiB / 0 | Across hosts (5) | 4080 (2540 to 9510) | 353842369 (287646092 to 475754672) | 972392556 | 486180008 |
| 4 TiB / 0 | This machine (2) | 5070 (3850 to 7730) | 1457141942 (1445782358 to 2023847045) | 3891628140 | 1945797800 |
| 4 TiB / 0 | Second host (3) | 4040 (2880 to 6720) | 1122337793 (1115925036 to 1541548423) | 3891628140 | 1945797800 |
| 4 TiB / 0 | Across hosts (5) | 4600 (2880 to 7730) | 1445782358 (1115925036 to 2023847045) | 3891628140 | 1945797800 |
| 16 TiB − 1 MiB / 0 | This machine (2) | 6760 (5570 to 7390) | 4896734085 (4796853448 to 6293868215) | 15568569548 | 7784268504 |
| 16 TiB − 1 MiB / 0 | Second host (3) | 5821 (4390 to 7230) | 3962762545 (3895826976 to 5208832836) | 15568569548 | 7784268504 |
| 16 TiB − 1 MiB / 0 | Across hosts (5) | 6190 (4390 to 7390) | 4796853448 (3895826976 to 6293868215) | 15568569548 | 7784268504 |
| 4 GiB / 1024 | Across hosts (5) | 1810 (1610 to 5950) | 6748361 (5216130 to 17213996), not X5 | 3418328 | 1593752 |
| 4 GiB / 10240 | Across hosts (5) | 12260 (11701 to 16440) | 11428831 (9906053 to 364095938), not X5 | 5941756 | 2044232 |
| 64 GiB / 10240 | Across hosts (5) | 14730 (13290 to 31840) | 15503175 (14935119 to 371069809), not X5 | 62958076 | 30552392 |
| 1 TiB / 10240 | Across hosts (5) | 15540 (13100 to 35120) | 80801087 (75143331 to 381817553), not X5 | 975223160 | 486683720 |

The by-host rows of the record and occupied cells are at artifact lines 833 to 846; only the across-host rows are listed here. Reading (the numbers are measured, the ratios are computed): at 16 TiB − 1 MiB one publish takes 15568569548 extra bytes, which is 2 times the 7784268504 bytes of one allocator copy plus 32540 bytes; on the five ladder sizes the publish extra is 2.0197, 2.0010, 2.0001, 2.0000 and 2.0000 times the copy bytes. The heap quantities (allocator copy, peak extra) are identical field by field on both hosts in the same cell; the second host's median publish time is 18.6%, 23.0% and 19.1% lower than this machine's at 1 TiB, 4 TiB and 16 TiB − 1 MiB.

### Segment-two readings: process high-water mark and “the largest sparse disk this machine can make” (R2g, X22)

Threshold T = 25769803776 bytes (24 GiB), lower edge 25702694912. All 45 cell-rounds “finished within the threshold” (45 `name=h2_shard_status` lines with `status=within_the_threshold class=within_the_threshold` and `near_the_threshold=false`; `grep 'name=h2_shard_status ' <artifact> | grep -o 'status=[^ ]* class=[^ ]*' | sort | uniq -c` prints one line, `45 status=within_the_threshold class=within_the_threshold`). H over the 5 rounds of each cell (`vm_high_water_mark_kibibytes` × 1024, `name=h2_process`):

| Cell (capacity / units) | H min | H max |
|---|---|---|
| 4 GiB / 0 | 70328320 | 71110656 |
| 4 GiB / 1024 | 292634624 | 292831232 |
| 4 GiB / 10240 | 2798190592 | 2799054848 |
| 64 GiB / 0 | 101654528 | 102383616 |
| 64 GiB / 10240 | 2858577920 | 2859716608 |
| 1 TiB / 0 | 1490620416 | 1491107840 |
| 1 TiB / 10240 | 3783000064 | 3783700480 |
| 4 TiB / 0 | 5944557568 | 5945462784 |
| 16 TiB − 1 MiB / 0 | 23627563008 | 23629737984 |

The largest of the 5 rounds' status lines for the 16 TiB − 1 MiB size (lines 772, 781, 790, 799, 808):

```text
E7RESULT name=h2_shard_status cell=h2-device-16777215-mebibytes-units-0 round=4 status=within_the_threshold class=within_the_threshold high_water_mark_bytes=23629737984 high_water_mark_before_kibibytes=4212 high_water_mark_after_kibibytes=68964 near_the_threshold=false host=第二台 image_filesystem=ext2/ext3 service_attempt=3/5:1 unit_area_slots=1073691584 registered_unit_area_slots=1073691584
```

It is 2072956928 bytes (about 1.93 GiB, computed) below the lower edge; the bound for “close to the threshold” is the program file's bytes plus 64 MiB = 302901336, so it is not close. **The largest sparse disk this machine can make**: the largest size whose cells all finished within threshold T on the two hosts used this time is the ladder endpoint 16 TiB − 1 MiB (registration X3: “when it falls on the ladder endpoint, write ‘≤ endpoint’”; nothing beyond the ladder was measured, and whether larger disks finish within the threshold is unknown). Threshold sensitivity (lines 850 to 853): at T ÷ 2 the largest size is 4 TiB (the H of 16 TiB − 1 MiB is above the lower edge of T ÷ 2), at 2T it is the same as at T, `depends_on_the_threshold=true`, and the size changes between T ÷ 2 and T.

PC-high-water-mark (V40): the high-water mark difference before and after PC-heap × 1024 in the 45 processes is 66109440 to 66961408 bytes, all ≥ 62914560 (45 `positive_control_heap` lines with `high_water_mark_control_holds=true`). PC-heap (V30): 45 lines with `peak_over_baseline_bytes=67108864 holds=true`.

Within the wrapper (R2h, X27): the service-recorded peak P(a) of that attempt is taken from `memory_peak` of each attempt of each subtask in `task-server/task status 5382 --json`; the apparatus side is `service_peak_needed_at_least_bytes` on lines 854 to 858 (= max H(a) − program file bytes − 64 MiB).

| Shard / attempt | Host | P(a) | Required at least | Holds |
|---|---|---|---|---|
| 0/5 attempt 6 | Second host | 25304449024 | 23325726632 | Holds |
| 1/5 attempt 1 | Second host | 25298878464 | 23326214056 | Holds |
| 2/5 attempt 1 | This machine | 25304514560 | 23325476776 | Holds |
| 3/5 attempt 1 | Second host | 25298677760 | 23326836648 | Holds |
| 4/5 attempt 1 | This machine | 25303404544 | 23324661672 | Holds |

The first five attempts of the round-1 shard (this machine, caps 4096M to 20736M, exit 250) finished no cell, so max H(a) has nothing to apply to. The service-recorded peak (about 23.57 GiB) is about 1.5 GiB above the maximum process high-water mark (about 22.01 GiB), which matches the page-cache magnitude inferred by registration Q13 (inferred, not measured separately); under X22 the service peak does not enter class judgement.

### Segment two: invalidation, failure and stop clauses item by item

| Clause | Observation | Judgement |
|---|---|---|
| T7 start prerequisites | All five hold (“Procedure and prerequisites” above) | No stop |
| T8 procedure prerequisites on the service side | No subtask exited 78; subtask exit codes were only 250 (hit the cap; the service raised the cap and requeued) and 0 | No stop |
| T9 geometry and A43 | `unit_area_slots_differing_cell_rounds=0`; `unit_area_slots` equals `registered_unit_area_slots` in all 45 status lines | No stop |
| T4 deterministic quantities differ between two rounds (also covering across hosts) | `name=h2_cell_determinism` is `rounds_compared=5 equal_across_rounds=true differing_fields=none` for all 9 cells, with the 5 rounds spanning two hosts | No stop |
| T2 local constants compared back | All 45 `name=h2_payload_constant` lines in the artifact are `equal=true`; the apparatus comparison unit test was not run in this segment, and its reading is not on this page | No stop on the artifact side; the unit test side was not checked |
| V30 PC-heap | All 45 lines pass | Not voided |
| V32 PC-linear, PC-flat | Lines 943 and 944 are both true | Not voided |
| V37 unreadable ≠ 0 | Every quantity of every cell and round is present; `=none` appears only where the registration expects it | Not voided |
| V38 within the wrapper | All 5 attempts hold | Not voided |
| V40 PC-high-water-mark | All 45 processes pass | Not voided |
| G1 a row falls in different classes across the 5 rounds | `unstable_slope_rows=none` | Not triggered |
| G2 the Q8 prediction of section 4 is wrong | R2a is sublinear (predicted basically unrelated) and R2e allocation is sublinear (predicted linear); R2c's b = 1.023 ≥ 0.8, and the per-disk per-slot bytes of the five ladder sizes are within A37's interval; the 4.822029 of record cell 4 GiB n = 10240 is outside A37's interval (A37 is written under R2c's “each size” row, and whether this cell is an object of A37 is unclear, so it is listed as is) | **Triggered**: prediction and measurement are put side by side as they are (table in the earlier section), and the text follows the measurement |
| G6 fewer than four capacities finished | 5 sizes | Not triggered |
| G9 the largest size is close to the threshold | `near_the_threshold_at_or_above_the_largest=none` | Not triggered |
| G10 a round spans hosts | All 5 rounds' `name=h2_round_hosts` are `one_host=true` | Not triggered |
| G11 wall clock no longer from a single machine (a loss recorded in advance) | The hosts of the 5 rounds are not all the same (rounds 1, 2 and 4 on the second host, rounds 3 and 5 on this machine); each shard log has one `/proc/loadavg` at the start and one at the end of each cell, and 32 of the 45 cell-rounds have at least one first column ≥ 2.0 (range 1.35 to 28.20; the 13 cell-rounds with both below 2.0 are 7 cells of round 2 and 6 cells of round 4; all 9 cells of rounds 1, 3 and 5 are ≥ 2.0) | **Triggered**: R2f is reported by host (earlier section), and the across-host values are not called “single-machine 5-round medians”; cells with high load are still classed (the reason under X25: within one round it is the same host, which only changes the intercept) |
| G12 cannot be judged | No service outcome file (no subtask recorded `memory-cap-at-maximum`) | Not triggered |

The G11 load readings come from each shard's service log (the lines in `task-server/task log 5429` to `5433 --all` that start with “格 ” and contain “负载”, 90 lines) and are not in the artifact; this page does not paste those lines verbatim (they carry epoch seconds); the count takes the first column of the two readings at the start and end of each cell-round, and a cell-round counts if either reading is ≥ 2.0.

### Fork table (question sheet H2; registration section 11's sufficiency condition for H2 is that R2a, R2b, R2d and R2e each produce a class or “unstable” and that at least 4 capacities finish)

`$P` is the H2 artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h2-2026-11-03.out`.

| Question row | Current status | What is still missing | Could the remaining measurements reverse it? | Command used to compute it |
|---|---|---|---|---|
| H2 · R2a allocation time against capacity | Produced: sublinear (5 rounds, b 0.251 to 0.322) | None | All of segment two's registered cells ran and none were skipped, so nothing remains to measure | `grep -n 'name=h2_slope row=r2a \|name=h2_class row=r2a ' $P` |
| H2 · R2b publish time against capacity | Produced: linear (5 rounds, b 0.811 to 0.875) | None | Same as above | `grep -n 'name=h2_slope row=r2b \|name=h2_class row=r2b ' $P` |
| H2 · R2d peak extra memory against capacity | Produced: allocation basically unrelated, publish linear (b 1.022) | None | Same as above (deterministic quantities, identical field by field over 5 rounds on two hosts) | `grep -n 'name=h2_class row=r2d_' $P` |
| H2 · R2e against record count | The two allocation rows are produced: time sublinear, peak basically unrelated; the two publish rows have no class (`x5_as_registered_cannot_be_made`) | Classes for the two publish rows: the registered X5 cannot be made on a file with n units (the 2026-10-19 amendment in section 12 of the previous registration); producing a class needs a different reading, which is a new registration | No: nothing left in the registration can fill these two rows | `grep -n 'name=h2_class row=r2e' $P` |
| H2 · capacities finished | 5 sizes (≥ 4), the largest 16 TiB − 1 MiB (ladder endpoint) | None | No | `grep -n 'name=h2_largest_disk' $P` |
| H2 · entire question row | Literally under registration section 11: R2a, R2b and R2d produced classes and 5 capacities finished, but R2e produced only the two allocation rows, so H2 is **literally insufficient** (the two R2e publish rows are missing). The question sheet's original sufficiency condition, “at least four disk capacities (including the largest sparse disk this machine can make and a disk with default geometry), each measuring the wall clock and peak memory of one allocation and one publish, plus a separate sweep over three record counts”: five capacities each measured the time and peak extra memory of allocation and publish over 5 rounds, and the record count was swept over three sizes (94, 2264 and 21034 records), but the publishes at those three sizes are not the registered X5 | The main agent decides sufficiency against the registration and question sheet | No unrun measurements remain in the registration; only a new registration can fill the two R2e publish rows | The H2 rows of this fork table |

### What it cannot answer

- Minimum disk with empty pool: V33 hit; mount at C_empty_mount − 1 MiB was not reached because mkfs failed first. This minimum belongs to mkfs geometry, and the empty halves of R4a–R4c are unavailable; artifact empty-pool rows are retained only as records (device 0 lines 99, 103, 176, 180, 268, 272, 348, 352; device 1 follows each).
- PC-admission-file proves only that at C_file_mount − 1 MiB mount succeeds but raising the floor remains insufficient, so the admission formula binds the minimum. It does not identify which of six terms binds (R4c), or establish finer than 1 MiB resolution.
- Samples: S=4,8,16; L=1 MiB plus S=8/L=4 MiB; default capacity only 4 GiB × 2/L=768 MiB. Sparse in-memory disks; file backend and other L not measured.
- Only one-file state measured; not multiple files, snapshots or deletion queue (the ring-window decision before mount when the deletion queue is nonempty is another path).
- How much candidate A (tightening to actual geometry) reduces minimum disk, and three or more disks, are out of scope (§11).
- Why K32 empty-pool `before_a_publish` charges ring window, and whether clause or implementation should change, is undecided: “this clause cell may be wrong.”
- There are no measurements on this page for the fourth segment (H3 read cache).
- What H2 cannot answer: the costs after candidates A, B and C are applied (only today's code was measured); the two R2e publish rows (the registered X5 cannot be made); disks larger than 16 TiB − 1 MiB (the ladder endpoint; the largest size is only known to be no smaller than it); “the largest sparse disk this machine can make” holds only for the registered threshold T and the two hosts used this time, and it is 4 TiB at T ÷ 2; a reading for a size on only one host is not extrapolated to the other (this time every cell had rounds on both hosts); the absolute times of R2f have fewer than 5 rounds on each host, and 32 of the 45 cell-rounds had a load first column ≥ 2.0 at the start or end, so they are not numbers from an idle single machine; the first free slot of the occupied cells never moved away from the start (all placements are 2), so the geometry sensitivity measured the effect of record count, not the effect of how far away the first free slot is; the attempt peak recorded by the service includes page cache and was not measured separately; the local file system holding the images reports ext2/ext3 for `stat -f -c %T` on both hosts (45 lines with `image_filesystem=ext2/ext3`), and other file systems were not measured; this segment did not itself run the apparatus unit tests or the three mutation numbers, and there are now 87 unit tests and 64 mutation rows for E183 (Capacity Slope, Read Cache Small-Disk Admission, and Sweep Cost Metrics) (`grep -c '#\[test\]'` on the apparatus source file, `grep -c '^E183 ' crates/mutations.tsv`).
- H5 cannot answer R5c/R5d (V35, 24 cells); sweep publication's own write amplification (per-mount writes are mixed in); why the threshold mount reads 32768 extra bytes (chain counting is inferred; the new arm establishes only equality to chain-read bytes); histories writing user data between mounts; background sweep enabled; bad-sector histories preventing deletion of all rows; θ between 64 and 365 or between 365 and 369 (366–368 unmeasured); third and subsequent sweeps (only two per cell); other capacities or file backend. Both machines used the same binary, but hardware/file system differences were not isolated; only one first-shard repeat matched.
- This segment did not itself run the three mutation numbers or all 73 tests; green tests are the main agent's reading, full mutation table awaits commit.

### Path and conclusion register

The conclusion belongs to the category where "the ring-window and unit-region capacities computed inside the apparatus agree with the independently calculated anchors": the exact same quantity is computed across three paths that do not share code and then compared.

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Core readings | Apparatus calls `admission_reading_of_a_writable_mount` and `admission_reading_before_a_publish` on returned mkfs/mount state; reads six current terms and capacity from `crates/`. | `crates/singlefs-core/src/admission.rs:487 ring_window_on_one_device` | D28 item 1 ring-window formula (3S+1) × W and upper bound on W; mkfs geometry; constants 812,169,144 from `singlefs_format`. A misread clause affects all three paths. |
| 2 | Apparatus-local port | The apparatus ports the anchor geometric arithmetic (first and last slots of the unit region, root tier, number of positions per tier N, W, and ring window) into Rust, compares item-by-item against registered script output in the `anchors` entry, and pins 15 rows in unit tests | `crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:315 ring_window_by_the_clauses` | Local constants (transcribed from the kb registration section and compared back against `crates/` via the `constants` entry); the exact same clause sentence |
| 3 | Anchor script | Pure Python calculates from clauses and kb constants; does not import apparatus or read `crates/`; A31 checker recomputes each artifact reading. | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py:28 ring_window_slots` | Same clause; constants 812,169,144, slot width 16384, journal ring start slot 1024. A31 expects the implementation's empty-pool `before_a_publish` deduction and is not independent of it for that cell. |
| 4 | H5 measured trajectory | Drives `mount_writable_with_instance_table_sweep_configuration` and unmount, reads rows and sweep outcomes each time; sparse memory disk. | `crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:5107 mount_once_in_the_history` | D18 item 11 trigger and reclamation clause, D28 item 3 page formula, `crates/` mount implementation. |
| 5 | H5 apparatus model | Rust port of anchor script's `h5()`; calculates per-shard model and compares each mount and registered output. | `crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:4386 sweep_history_by_the_anchor_model` | Same clauses: one row per writable mount, warm-up root replenishment, empty publications on normal unmount, deleting rows of instances with no root in ring. |
| 6 | H5 anchor script | Pure Python calculates mount and sweep positions, deletions, residuals, peaks and final txg. | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py:38 h5` | Same clauses; kb constants 369,32768,N_switch=3. |
| 7 | H5 independent check | Compares 24 artifact cells with path 6 and one background control. | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_history_anchor_check.py:14 fields_of` | Path 6 script and artifact's per-mount rows. |

Cross-path comparison assertions are pinned in both source code and artifacts: the `anchors` entry compares Path 2 against Path 3 at runtime (`anchor_mismatches=0`); the unit test `local_geometry_reproduces_every_registered_script_row` (line 2660 of the apparatus source file) pins Path 2 reproducing the 15 rows of the script; the A31 verification script compares Path 1 against Path 3 (`mismatches=0`, across 56 readings). The values of the conclusion are enumerated points, not a complete sweep of the domain: 7 sampled points × 2 states × 2 entry points × 2 disks, because registration only required these specific geometric points (Section 8, §8.2). If a clause sentence is misread and all three paths err together, these three paths cannot reveal it; the correctness of the clauses themselves relies on item-by-item verification against K30 through K34, and in this segment, K32 for an empty pool at the `before_a_publish` entry does not match the implementation. Paths 4 through 7 belong to H5: the conclusion is sequential agreement across 24 enumerated points (3 values of S × 4 values of θ × 2 unmount methods) plus one control cell, rather than sweeping the entire domain; Paths 5, 6, and 7 share the same batch of clause sentences read by Path 4, so when a clause sentence is misread they err together, whereas K33 and K34 are judged sequentially against the literal text of the clauses rather than relying on cross-path comparison; the independent check (Path 7) reads the artifact emitted by Path 4 and shares the same origin as Path 6.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D28 (Mount-Time Commitments), settled item 1 | Supporting material | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-03 unaffected: Rerun 2 ran only segment two H2, which measures the slopes of the time and peak memory of one allocation and one publish against disk capacity and record count (the pre-change baseline of C567 (allocation and publish costs linear in disk capacity)); it does not read the six terms of the admission formula, and the artifacts and readings of segments one and three are untouched. 2026-10-30 unaffected: Rerun 1 only changed the apparatus's local sweep threshold constant from 369 to 365 and produced the segment-one artifact again; a line-by-line diff changes only line 6, the 11 verdict lines are identical, the readings of R4a through R4d, K31 and K32 are unchanged, and segment three was not rerun. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds, and C755 (recovery and two mount triggers fail differently on rollback) asks F2 once per writable mount) segment one produced its artifact again with only the pinned value fixed; only the system configuration read and write counts of 28 `name=h4_mount_tally` lines changed, the readings and verdict lines of R4a through R4d, K31 and K32 are identical, and segment three was not rerun. 2026-10-21 unaffected: only abbreviated identifiers, clause names, section references and current source line numbers were updated; numbers, conclusions and artifacts unchanged. 2026-10-19 unaffected: rationale does not cite E183; ring window is 692–888‰ of admission reserve at minimum disk and K32 empty `before_a_publish` disagrees with clause. Main agent decides whether to promote as support or contradiction and amend clause; decision text unchanged. H5 did not touch this item. |
| D28, settled item 3 | Supporting material | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-03 unaffected: Rerun 2 ran only segment two H2, which measures the costs of allocation and publish against capacity and record count; it does not touch the switch reserve or the page count of the instance table, and the artifacts and readings of segments one and three are untouched. 2026-10-30 unaffected: Rerun 1 only changed the apparatus's local sweep threshold constant from 369 to 365 and produced the segment-one artifact again; a line-by-line diff changes only line 6, the 11 verdict lines are identical, the readings of R4a through R4d, K31 and K32 are unchanged, and segment three was not rerun. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds, and C755 (recovery and two mount triggers fail differently on rollback) asks F2 once per writable mount) segment one produced its artifact again with only the pinned value fixed; only the system configuration read and write counts of 28 `name=h4_mount_tally` lines changed, the readings and verdict lines of R4a through R4d, K31 and K32 are identical, and segment three was not rerun. 2026-10-21 unaffected for same editorial reasons; 2026-10-19 rationale does not cite E183 and K31 matches switch reserve and checkpoint reserve pool in all 28 readings, with no conflict requiring edits. H5 measured chain rewrite page steps: at peak rows 370/739 peak pages 2/3, matching model and anchor in all 24 cells (A34), consistent with max(1, ⌈(rows0 + N_switch) ÷ 369⌉); main agent decides whether to promote as support. |
| D18 (Information Carried in a Block), settled item 11 | Supporting material | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-03 unaffected: Rerun 2 ran only segment two H2, which tests the allocator and the publish path; H2 does only two writable mounts per cell, so the instance-table row count cannot reach the sweep threshold (inferred from the cell steps in 5.2 of the previous registration), and the artifacts and readings of segment three are untouched. 2026-10-30 unaffected: Rerun 1 only changed the apparatus's local sweep threshold constant from 369 to 365 and produced the segment-one artifact again (segment one's mounts take the product path, the instance-table row counts of its histories never reach the threshold, and the threshold is read only on the comparison line); a line-by-line diff changes only line 6, the 11 verdict lines are identical, and segment three was not rerun. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds, and C755 (recovery and two mount triggers fail differently on rollback) asks F2 once per writable mount) segment one produced its artifact again with only the pinned value fixed; only the system configuration read and write counts of 28 `name=h4_mount_tally` lines changed, the readings and verdict lines of R4a through R4d, K31 and K32 are identical, and segment three was not rerun. 2026-10-21 unaffected for same editorial reasons; 2026-10-19 rationale does not cite E183. H5 tested immediate sweep decision, erase and publication when instance-table rows exceed θ=64,365,369,738: all 18852 mounts match trigger predicate K33; all 50 sweeps delete only rows for instances with no root in ring (K34). Main agent decides whether to promote as support. R5c sweep-publication write amplification is unavailable; no conflict requiring a decision-text change. |

## Historical Versions

Historical entries for E183 (cost numbers for capacity slope, read cache, small-disk admission, and sweep thresholds) are collected in [experiments-history.md](../experiments-history.md).
