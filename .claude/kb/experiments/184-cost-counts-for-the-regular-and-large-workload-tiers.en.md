## E184 Cost Counts of Regular Layer and Large Workload Layer — partially run (segment one, “workload scale and overlap,” run and rerun in full once on the new segment shapes, G1 decidable; segments two through four not run)

[中文](184-常规层与大负载层的代价计数.md) · **English** · [日本語](184-通常層と大規模負荷層のコスト計数.ja.md)

<!-- doc-lint:not-numbers J2 J3 G1 G2 G3 G4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 T0 T2 T3 T4 T5 T7 A1 A3 A4 A5 A8 A9 A10 A11 A12 B1 B2 B3 B4 V1 V2 V3 V4 V5 V6 V7 V9 V10 V11 V12 V13 V14 V15 V16 V17 V18 V19 V20 V21 V22 V23 V24 V25 V26 V27 R4 FR1 FR2 FR3 XR1 XR2 XR3 XR4 XR5 -->

Preregistration `research/prompts/e184-preregistration.md` (ten amendments in section 12, amendments 6–10 written after apparatus unit tests and before output). It answers four rows of question sheet `research/prompts/m3-close-c409-cost-questions.md`: “workload scale and overlap” (the sheet asks workload sizes and overlap with the smoke tier; registration separates scale, state-count interval overlap, shared block keys and same-key blocks in the long-term library); J2 cold throughput and library increment; J3 block-key invalidation surface; and “fixed overhead of two implementation placements.” These correspond to G1–G3 of fork sheet `research/prompts/m3-close-c409-r1-forks.md`; G4 is not covered by the registration. Work is split into four segments. Segment one answers only “workload scale and overlap” (G1): which workloads belong in the regular tier in D13 (Verification Route), settled item 9's tier table, how their state-count intervals overlap the smoke tier's registered streams, and how many block keys they share. Other tested provisions are D13 settled item 17's block key (derived only from content) and D13 settled item 4's segmentation measurement basis (used by the apparatus). C409 (normal tier and large workload tier lack stages in gate) is the associated owed check.

The apparatus consists of two committed programs: `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` (all subcommands and inline unit tests) and `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts_write_dump.rs` (records and dumps the write stream only). All block keys are computed using crates' `crash_points_of_the_steps`, `plan_crash_points`, and `PlannedCrashPoint::block_key`, not a separate implementation. Segment one is divided into 18 jobs by long-run script `scripts/experiment-apparatus/e184-segment-one-long-run.sh` (`--build-here`, output headers record both binaries' sha256; 17 freshly run, zero reused; the registration condition for job 18, `library`, was not met, so output says `state=not_run`). There are two workload families: arm R (fixed-seed random histories, 20 consecutive seeds starting at the seed base) and arm F (overwrite chains, with u = 1, 4, 16 data units per overwrite). The baseline is four re-recorded smoke-tier streams (R4 in the registration).

**Current status:**

- Second rerun (2026-10-24, rerun registration `research/prompts/e184-r2-prereg.md`): the whole segment-one artifact was rerun on today's main workspace on the new segment shapes (new pool 12 segments, snapshot creation 16 segments, multi-record 20 segments); artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out` (959 lines, final line `E7RESULT name=done segments=18 emitted=959`, `sha256sum` `5c5749effbda6daed7d8f329c6b2fc4fecd71074ff2dff794a996a0f3cfdac0c`), comparison output `research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out` (83 lines). Both question-sheet rows are decidable: the new numbers of anchors A1, anchor 2, A5, anchor 7 and X3 (16842785; 16777243 with 43 writes; 16777268 with 54; 1678377220 with 294; 16842815) are all “new number settled”, the 10 recomputation-versus-recording cells are `same=true` and the `selfcheck` job exits 0; the narrow reading's 40 cells, the wide reading's 62 cells and the 4 derived clauses have 0 words changed, recorded as “not flipped”. Interval S changed (lower bound 16777243, upper bound 5722079403, with two smoke streams that entered the repository on 2026-10-21); the fractions of the scopes move while the verdict words do not, and the R 400 steps level has inside fraction 0.100000, exactly on the threshold endpoint; B1 now records 1358954642 and the first part of F6 still triggers. Readings and conditions are given one by one in the section “Second rerun”.
- Output of the first run (2026-10-19, on the tree with one round of number acquisition writes; from this item through the “82 unit tests” item the readings are of this artifact) `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out`: 950 lines, final line `E7RESULT name=done segments=18 emitted=950`, sha256 `7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06` (key names, labels and stream names were expanded to remove abbreviations; numbers unchanged; prior hash `0704bc652549aa54477335b893627ada8ad04bedd76972a0a76ac14db4f39867`). 127 streams: 120 random histories (BROAD weighting, two 4 GiB disks, 20 each at 50, 100, 200 and 400 steps; 20 at 100 steps with a 240-slot disk width; 20 at 100 steps with maintenance weighting; the 80 BROAD streams across four levels are the main sample and the last two groups are the opposite-direction sampling points in 8.2), three overwrite chains (u = 1, 4, 16), and four re-recorded smoke streams. All 120 random histories have `ending=Completed`.
- “State-count interval overlap” (overlap with the smoke-tier interval; inside ≥ 0.5 is overlap, ≤ 0.1 separate): random histories by step level are 50 overlap (0.583333), 100 overlap (0.500000, exactly at threshold), 200 partial (0.200000), 400 separate (0.000000); all four combined partial (0.291667). All three overwrite-chain levels are separate (all state counts exceed the interval upper bound). Interval S uses 14 pinned smoke streams, upper bound 2438004868, lower bound 16777240.
- “Shared block keys” (fraction shared with the four re-recorded smoke streams; family maximum ≤ 0.05 is independent, family median ≥ 0.5 is renamed): all six random-history groups and overwrite chains u = 1, 4 are independent. Maximum shared fraction 0.002761 (overwrite u = 1); random-history maximum 0.001985 (50 steps). Eight of the 120 random histories (three at 200 steps, five at 400) and overwrite u = 16 exceed the 2^26 block-count upper bound, so plans are not generated and shared keys cannot be obtained (`not_planned`), not reported as zero.
- Geometry sensitivity 8.2: both opposite-direction samples at 100 random steps (240-slot disk width and maintenance weighting) have state-count interval overlap, as does the main sample. The step-count row (72 combined streams versus the 50-step level) is partial versus overlap, marked “unstable.” Overwrite chains have the same state-count result at u = 1, 4, 16 (separate); shared keys are unavailable for u = 16.
- Provision cell: anchor B1 (D13 settled item 9's range-write multi-record stream, 12230590578 states) re-records as 1358954634 (the segment-one artifact's reading; after number acquisition started writing in two rounds, `selfcheck` now records 1358954642, still `match=false`). This was expected by registration section 4 and triggers failure condition F6 without invalidating the experiment (line 77 of the second rerun's artifact is the same: records 1358954642, `match=false`).
- All 174 `name=control` positive controls have `passed=true`; the 12 pairs of seed-0/1 write-table digests, recorded twice for each random-history level, all match (X2 does not fire). All 28 stop checks pass. Nine of ten `name=anchor` rows have `match=true`; B1 has `match=false` (`selfcheck_done` excludes B1 from `anchors_mismatching` and reports `b1_matches=false` separately).
- 82 unit tests (main agent's release reading: 82 passed, 0 failed; `grep -c '#\[test\]'` finds 61 in the main apparatus and 21 in the dump binary, which imports the main apparatus through `#[path]`; 61 + 21 = 82). Twenty-six mutation rows were appended to `crates/mutations.tsv`; their three counts await the commit-time checker-tier-crates-mutation-replay gate, and this page has no mutation results. After the upstream write-order change only the pinned values were fixed; these 82 unit tests have not yet been rerun on the changed apparatus.
- After the upstream write-order change (number acquisition writes in two rounds: one more 2-write system configuration slot segment after mkfs) only the pinned values were fixed, and the conclusions are unchanged. The main apparatus re-copied the test pins and segment shapes from the numbers the four smoke-test files pin now (new-pool file-creation stream two-state 16777243, full run 16777268, A-two 54, 12 segments, 43 writes; fixed script stream 1678377095, 1678377220, 294, 80 segments; multi-record stream 1358954607, 1358954642, 20 segments, 117 writes; snapshot-creation stream 16842785, 16842815, 16 segments, 64 writes; mid-tier full run 167837792), computed anchors A1 and anchor 2 afresh from the new segment shapes (16842785, 16777243, 43 writes), and `selfcheck` produced `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` again (40 lines, exit 0): all 28 stop checks pass, 9 of the 10 `name=anchor` lines are `match=true`, and the B1 line is `match=false` (expected by the registration). The replay registration now points to this file; the 2026-10-19 one, `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out`, is kept and differs from the new one in 18 lines (lines 5, 7, 12, 13, 17, 18, 22, 23, 27 through 31 and 34 through 38: the start and end line numbers of the copied segments in the test files, the two-state and full-run counts of each stream, block counts, the A1, anchor 2 and write-count anchors, and the recorded values of the four full-run-count anchors and of B1). The state counts in the whole segment-one artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` are readings of the version with one round of number acquisition writes; it was rerun in full on the new segment shapes on 2026-10-24 (section “Second rerun”), and its verdict words are identical to this one cell by cell. The 16842807 hard-coded in X3 and the local constant of the J2 small-tier cell (apparatus `:2913`) were not changed; the new value in the registration is 16842815 and it must be changed before segment three starts.
- Segment two (J3 invalidation surface), segment three (J2 cold throughput, heavy tier) and segment four (fixed overhead of two placements) have no outer script and were not run. J2, J3 and the fixed-overhead question remain open; G2/G3 have no readings. G4 is outside the registration.

### What was done

1. The long-run script's `repo-reads` job checks the stop condition “conversion from random history to `CrashFlow` is absent in crates” (zero hits from `grep -rn "CrashFlow" crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-harness/src`); output line 3 has `result=pass hits=0 files_read=18`.
2. The 18 long-run jobs were performed in order. Each apparatus output is merged intact into the artifact, preceded by a `name=segment` row (job name, run count, cumulative seconds, lines, binary sha256). Their cumulative times sum to 5462 seconds (summation command in “Replay”).
3. Dumps are reproduced from deterministic seeds: seed 0 and 1 for each of six random-history levels were recorded twice and their write-table digests compared (12 `name=determinism` rows, all `equal=true`). At write time, all 120 history dumps were checked for “reconstructed base image equals the base obtained by layering the recorded stream's mkfs steps on empty disks” (`base_matches_the_stream=true`, six `dump_done` rows each with `base_mismatches=0`).
4. Overwrite chains u = 4, 16 use `publish_sequential_write` (sequential write: each overwrite is u single-unit transactions in one publication); u = 1 uses `publish_overwrite`. Crates lacks a multi-data-unit overwrite in one transaction; rationale and fallback are in amendment 6. The apparatus counts K overwrites: 30, 23 and 11 for u = 1, 4, 16 (output lines 71, 73, 75), with write counts 1003, 1042, 1050.
5. The full-run state count of overwrite chain u = 16 is 2954361402816462969; its block-count upper bound far exceeds 2^26, and amendment 10 says not to plan it. Three positive controls requiring full-run planning (change one base-image sector, name excluded from key, two expansions) were not run on this chain. The dump round-trip control compares only the base image (`keys_equal=not_planned`, output line 76 `kind=dump_round_trip_direct_base`).
6. 2026-10-23, pinned value fix only: after the two-round number acquisition write landed, 8 apparatus unit tests went red on pinned values (`selfcheck` exited 3: 6 stop checks fail, 4 anchor lines mismatch). In a repository copy (the main workspace's `crates/` of that day, without `target/`), `selfcheck` was first run with the original apparatus to measure the current values, and the readings matched line by line those an investigator measured in another copy. The four smoke-test files already held the two-round numbers, so the apparatus re-copied `PINNED_BY_THE_TESTS`, the three segment-shape constants and the mid-tier full-run-count constant from them, and computed A1, anchor 2 and the new-pool write-count anchor afresh from the new segment shapes with `python3 -c` (`s=[2,2,2,1,2,2,1,2,24,2,1,2,16,2,1,2]` gives 16 segments, 64 writes, 16842785; the first 12 segments give 43 writes, 16777243; the 20 segments of the multi-record stream give 117 writes, 1358954607); the 12 places in the inline unit tests that follow the pinned values were changed with them. The apparatus was built in release and `selfcheck` run: exit 0, artifact `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`; the replay registration now points to it.
7. 2026-10-24 second rerun: checked the new numbers of registration section 7 (the two recomputation routes equal the apparatus text item by item), reran all 18 jobs of segment one with the long-run script from a new working directory on today's main workspace (17 ran fresh, 0 reused; the first start exited 3 at the `repo-reads` job and produced no artifact, and after registration section 12 amendment 2 added one more way of recognizing a pinned value it was restarted in a new working directory), then compared the verdict words of the rerun artifact with the 2026-10-19 artifact cell by cell with `scripts/experiment-apparatus/e184_r2_verdict_compare.py` (narrow reading 40 cells, wide reading 62 cells, 4 derived clauses); the comparison script's 16 unit-test items were rerun green in this segment. Readings and conditions are given one by one in the section “Second rerun”.

### Replay

```bash
# repo root; the whole artifact is a long run (18 shards; a resume reuses the finished shards; how to launch it is in the long-run script header) and carries wall-clock fields, so it is not registered for byte-exact replay.
# the replay table registers the deterministic shard: selfcheck (stop checks T1 to T4 and the anchors; this shard takes 4 seconds in the artifact); the committed apparatus is heavy tier, run by the main agent at commit with SINGLEFS_HEAVY_TESTS=commit
bash scripts/experiment-apparatus/replay.sh E184

# 整份产物的起法（预计超过 2 小时，长活声明在脚本文件头；这一次各件累计 5462 秒，求和命令在后一段）
bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label e184-segment-one -- bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --build-here

# 各件累计秒数相加
grep 'name=segment segment=' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep -o 'seconds_total=[0-9]*' | cut -d= -f2 | paste -sd+ | bc

# second rerun (registration research/prompts/e184-r2-prereg.md 5.3): launched as above, only with a brand-new working directory; the script picks the artifact name itself and adds -r2 if the day already has one
bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label e184-segment-one-r2b -- env E184_WORK_DIRECTORY="$HOME/.local/share/singlefs/work/e184-segment-one-r2" bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --build-here

# cell-by-cell comparison (reads only the two artifacts, starts no apparatus; run the unit test first)
python3 scripts/experiment-apparatus/e184_r2_verdict_compare_test.py
python3 scripts/experiment-apparatus/e184_r2_verdict_compare.py research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out
```

The replay entry in `scripts/experiment-apparatus/replay.sh` is `@driver_e184||e184-regular-layer-cost-counts-selfcheck-2026-10-23.out|exact` under E184 (Cost Counts of Regular Layer and Large Workload Layer). The registered retained output `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` comprises the 39 lines of `selfcheck` run by the apparatus after the pinned-value fix plus `E7RESULT name=done emitted=40` (the driver adds a final line following E175, product-path inode-number watermark rollback and reissue). It differs from lines 31–69 of the full artifact (the original output of the `selfcheck` job on the version with one round of number acquisition writes) in 18 lines: `diff <(sed -n 31,69p <整份产物>) <(head -n 39 <留存产物>) | grep -c '^>'` prints `18`. The previously registered `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` matches those 39 lines verbatim (the same `diff` prints nothing). Other jobs contain wall-clock fields (`keys_seconds` per stream in `keys`, `seconds` in `record-fixed-chains`, timings in `progress`) and naturally differ between runs, so are not registered; see the long-run script header.

Counts of each result-line type in the artifact (verbatim output of `awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | sort | uniq -c`):

```text
     10 name=anchor
      7 name=chain_recorded
      1 name=chains_done
    174 name=control
     12 name=determinism
      1 name=done
    120 name=dump
      6 name=dump_done
     13 name=family_summary
    127 name=flow
      7 name=keys_done
     11 name=overlap
     10 name=pinned_smoke
      1 name=pinned_smoke_done
     15 name=pinned_smoke_missing
    247 name=progress
     18 name=segment
      1 name=segment_one_begin
      1 name=selfcheck_done
    114 name=share
      1 name=share_done
     11 name=share_group
      4 name=share_ignored
      9 name=share_not_planned
     29 name=stop_check
```

Count the registered “arm × history × geometry”: 127 `name=flow` = three overwrite chains + four smoke + six random-history groups × 20; 120 `name=dump` = six random groups × 20; 13 `name=family_summary` = three overwrite u values + four smoke streams + six random groups; 114 `name=share` = 123 candidates − nine `share_not_planned` (eight random histories and overwrite u = 16). Of 174 `name=control` rows, 118 `self_share` = 114 planned candidates + four smoke; `base_sector_changed`, `name_not_in_key`, and `two_expansions` each have 12 = six seed-0 random levels + overwrite u = 1, 4 + four smoke (overwrite u = 16 lacks these three because no full-run plan is made, amendment 10 item 2). Thirteen `dump_round_trip` = six seed-0 random levels + seven overwrite and smoke; seven `dump_round_trip_direct_base`. No missing rows or fields.

### Second rerun (2026-10-24): rerunning all of segment one on the new segment shapes

Rerun registration `research/prompts/e184-r2-prereg.md` (the reading is fixed as the wide reading; two amendments in section 12, both before the output: four readings of the cell-by-cell comparison script fixed, and one more way of recognizing a pinned value added to the long-run script). It answers two rows of question sheet `research/prompts/m3-close-e184-r2-questions.md`. Row one: what registration section 7 anchors A1, anchor 2, A5, anchor 7 and section 11 X3 take on the new segment shapes (new pool 12 segments, snapshot creation 16 segments, multi-record 20 segments). Row two: whether the verdict words of the whole segment-one artifact flip on the new segment shapes. The two apparatus source files are unchanged (`sha256sum` gives `8f529e12…` and `320822c4…`, equal to the registration header); the long-run script is the `c1f9176e…` version after amendment 2. The only new files this time are the comparison script `scripts/experiment-apparatus/e184_r2_verdict_compare.py`, its unit test `e184_r2_verdict_compare_test.py` and the mutation table `e184_r2_verdict_compare.mutations.tsv`. The tree is the day's main workspace: other changes to `crates/` since 2026-10-19 went into this run too, so whether the verdict words flip is reported only as “did they flip on today's tree” and is not attributed to the segment shapes (registration 1.1).

**Artifact and completeness**:

- Rerun artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out`: 959 lines, `sha256sum` `5c5749effbda6daed7d8f329c6b2fc4fecd71074ff2dff794a996a0f3cfdac0c`, final line `E7RESULT name=done segments=18 emitted=959`, and `emitted` equals the line count. The working directory was new; of the 18 jobs 17 ran fresh and 0 were reused; the `library` job writes `state=not_run reason=no_family_share_verdict_partial` (line 958; the registration counts it as complete). The jobs' cumulative time is 8403 seconds (5462 seconds before), of which the `share` job took 6273 seconds. An earlier start on the same day exited 3 at the `repo-reads` job and produced no artifact: the stream `large_file_overwritten_across_lower_leaves_stream`, newly in the smoke tier, pins a different quantity of the expanded segments, and neither pinning form known to the long-run script recognizes it; registration section 12 amendment 2 added the form “segment shape present but pin not recognized, record as missing pin”. This artifact comes from the restart in a new working directory. The long-run log is outside the repository and not committed.
- Comparison output `research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out`: 83 lines, `sha256sum` `a3f4d71226fed5c9d092bf753d5f2d6bd653647b96d2df369bf221b37bea2aaf`, final line `E7RESULT name=done emitted=83`. This segment ran the same command again and the output is byte-identical (`cmp`) to the committed file. The comparison script's unit test is 16 items, all green (final line “自证 16 项，红 0 项”, rerun in this segment); the mutation table has 9 rows, and the reading handed back by the segment that wrote the script (step 1 of registration table 5.4) is caught 9, invalid 0, did not turn red 0 (`mutate.sh` handles only Rust, so following the precedent of `e185_r1_compare.mutations.tsv` each row was broken in a copy and the unit test run); this segment did not rerun the mutations.
- Row counts per kind of result line and arm × history × geometry (counted by one command; compared with the earlier artifact only `name=pinned_smoke` differs, 10 to 12, and `name=pinned_smoke_missing`, 15 to 22; every other kind has the same count, with no missing rows or fields):

```bash
P=research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out
awk '{for(i=1;i<=NF;i++) if($i ~ /^name=/){print $i; break}}' $P | sort | uniq -c
grep 'name=flow ' $P | grep -o 'family=[a-z_]*' | sort | uniq -c
grep 'name=chain_recorded' $P | grep -o 'label=overwrite_chain-[^ ]* \|overwrites=[0-9]* writes=[0-9]*' | paste - -
```

```text
     10 name=anchor
      7 name=chain_recorded
      1 name=chains_done
    174 name=control
     12 name=determinism
      1 name=done
    120 name=dump
      6 name=dump_done
     13 name=family_summary
    127 name=flow
      7 name=keys_done
     11 name=overlap
     12 name=pinned_smoke
      1 name=pinned_smoke_done
     22 name=pinned_smoke_missing
    247 name=progress
     18 name=segment
      1 name=segment_one_begin
      1 name=selfcheck_done
    114 name=share
      1 name=share_done
     11 name=share_group
      4 name=share_ignored
      9 name=share_not_planned
     29 name=stop_check
      3 family=overwrite_chain
    120 family=random_history
      4 family=smoke
label=overwrite_chain-01_units_per_overwrite 	overwrites=30 writes=1005
label=overwrite_chain-04_units_per_overwrite 	overwrites=22 writes=1001
label=overwrite_chain-16_units_per_overwrite 	overwrites=11 writes=1052
```

The number of overwrites of overwrite chain u = 4 went from 23 to 22 (writes 1042 to 1001), while u = 1 and 16 are still 30 and 11; the apparatus counts the overwrites (“What was done” item 4) and no parameter was changed, so the u = 4 figure moved with the new segment shapes.

**Question-sheet row one: what anchors A1, anchor 2, A5, anchor 7 and X3 take on the new segment shapes** (old numbers copied from sections 7 and 11 of the original registration, which was not changed by a single character; new numbers were recomputed in registration 7.2, and this segment ran that recomputation script again: its `sha256sum` equals the one the registration records and its output is identical to the one the registration pastes. Full-formula route: 1 plus the sum of every segment's state count, with a system configuration slot segment of 2 writes taking three states; increment route: the old number plus the increment for each system configuration slot segment the new segment shape adds):

| Anchor | Old → new | Two recomputation routes | Rerun artifact (`expected=` / `actual=` / `match=`) | Comparison |
|---|---|---|---|---|
| A1 snapshot creation stream two-state `snapshot_creation_two_state` | 16842782 → 16842785 (15 segments 62 writes → 16 segments 64 writes) | full formula 16842785; increment 16842782 plus 3 | line 68 16842785 / 16842785 / true | line 67 `same=true` |
| Anchor 2 new pool two-state `new_pool_two_state` | 16777240 → 16777243 | full formula 16777243; increment plus 3 | line 69 16777243 / 16777243 / true | line 68 `same=true` |
| Anchor 2 new pool writes `new_pool_writes` | 41 → 43 | full formula 2 × 8 + 3 + 24 = 43; increment plus 2 | line 70 43 / 43 / true | line 69 `same=true` |
| A5 new pool full run `full_states_smoke-new_pool_file_creation_stream` | 16777260 → 16777268 (A-two 46 → 54) | full formula 16777268; increment plus 8 | line 73 16777268 / 16777268 / true; A-two 54 is `quick_tier=54` in `name=flow` line 120 | line 72 `same=true` |
| A5 fixed script full run `full_states_smoke-overwrite_fixed_script_stream` | 1678377204 → 1678377220 (two-state 1678377089 → 1678377095, A-two 278 → 294, segments 78 → 80) | full-formula route cannot be computed (neither the test file nor the apparatus pins the roles of the 80 segments); increment plus 16; another route 1678377095 plus 25 × 5 = 1678377220 | line 74 1678377220 / 1678377220 / true; two-state, A-two and segment count are in `name=flow` line 125 (`two_state=1678377095`, `quick_tier=294`, `segments=80`) | line 73 `same=true` |
| A5 snapshot creation stream full run `full_states_smoke-snapshot_creation_stream` | 16842807 → 16842815 | full formula 1 + 6 × 8 + 4 × 3 + 4 + (2^24 − 1) + (2^16 − 1) = 16842815; increment plus 8 | line 76 16842815 / 16842815 / true | line 75 `same=true` |
| Anchor 6 multi-record stream full run (incidental) `full_states_smoke-multi_record_publish_stream` | 1358954634 → 1358954642 (two-state 1358954604 → 1358954607, 19 segments 115 writes → 20 segments 117 writes) | full formula (roles inferred) 1358954642; increment plus 8 | line 75 1358954642 / 1358954642 / true; two-state and segment count are in `name=flow` line 115 | line 74 `same=true` |
| Anchor 7 second half and X3 small-item `states` | 16842807 → 16842815 | same as A5 snapshot creation stream full run; the two routes agree | the second half is the A5 snapshot creation stream full-run cell (line 76); the first half of anchor 7 needs the J2 cold run, not run this time | not applicable |
| B1 clause `multi_record_full_named_by_the_clause` | clause 12230590578 unchanged; recorded 1358954634 → 1358954642 | full formula 1358954642 | line 77 12230590578 / 1358954642 / false (expected by registration) | line 76 `same=true` (compares only `actual=`, registration section 12 amendment 1 item 2) |

- Self-check exit code: line 78 of the `selfcheck` job, `E7RESULT name=selfcheck_done stop_checks=28 failed=0 anchors=10 anchors_mismatching=0 clause_anchor_matches=false emitted=39`, recorded as “exit 0”. The 39 lines of the `selfcheck` job (artifact lines 40 to 78) are byte-identical to the first 39 lines of `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` (command in “Replay registration” at the end of this section).
- New value of X3: the small-item `states` differing from 16842815 voids the run (the original registration says 16842807); the other three conditions (`reused`, contains the self-check, monotone) are unchanged. The apparatus constant `SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST` (`crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs:2913`) was not changed this time and is still 16842807; it is used by `voided_cold_runs` at `:3035`. Only `compose` in segment three uses it, so it must become 16842815 before segment three starts, together with the two test cases that build hand-made readings (`cold_run_that_is_not_cold_or_not_monotone_or_not_the_pinned_item_is_void`, `the_compose_lines_give_the_pinned_cells_for_a_hand_made_load`); who changes it is for the main agent to decide.
- Candidate A (change the original registration to the new numbers, with an amendment record saying what changed, on what basis and when) and candidate B (leave the original numbers, set up a separate rerun registration that only swaps the numbers) are a choice of how to record, not something the readings flip; the table in section 7 of the rerun registration is B and also has ready the text that A would append to section 12 of the original registration; the main agent decided not to append it (B is done).

**Question-sheet row two: do the segment-one verdict words flip** (wide reading; the comparison script reads only the two artifacts):

| Reading | Cells | Number of cells | Cells whose word changed | Verdict | Comparison line |
|---|---|---|---|---|---|
| narrow | `stop_check` 29 cells (`result=`, all pass → pass), `anchor` 10 cells (`match=`, 9 true and the B1 cell false, same before and after), `selfcheck_done` 1 cell (`clause_anchor_matches=` false → false) | 40 | 0 | not_flipped | line 81 |
| wide | the 40 narrow cells plus `overlap` 11 cells and `share_group` 11 cells | 62 | 0 | not_flipped | line 82 |
| derived clauses | F1, F2, F3 and the second part of F6, false before and after | 4 | 0 | same | lines 63 to 66 |

The 22 `overlap` and `share_group` cells by scope (verdict words identical before and after; fractions copied from the same-named lines of the two artifacts; the earlier line numbers are in the reading sections for the 2026-10-19 artifact and the later ones are in the last column):

| Scope | interval-overlap inside fraction before → after | B verdict | maximum shared-block fraction before → after | C verdict | later lines (`share_group` / `overlap`) |
|---|---|---|---|---|---|
| R main sample combined (four step levels, 80 streams) | 0.291667 → 0.438356 | partial | 0.001985 → 0.002261 | independent | 937 / 938 |
| R 50 steps | 0.583333 → 0.923077 | overlap | 0.001985 → 0.002261 | independent | 945 / 946 |
| R 100 steps (main sample) | 0.500000 → 0.550000 | overlap | 0.000864 → 0.000986 | independent | 947 / 948 |
| R 100 steps, 240-slot disk width | 0.894737 → 0.894737 | overlap | 0.000000 → 0.000000 | independent | 953 / 954 |
| R 100 steps, maintenance weighting | 0.526316 → 0.789474 | overlap | 0.000664 → 0.000757 | independent | 955 / 956 |
| R 200 steps | 0.200000 → 0.350000 | partial | 0.000548 → 0.000637 | independent | 949 / 950 |
| R 400 steps | 0.000000 → 0.100000 | separate | 0.000153 → 0.000174 | independent | 951 / 952 |
| F main sample (u = 4) | 0.000000 → 0.000000 | separate | 0.000060 → 0.000061 | independent | 935 / 936 |
| F u = 1 | 0.000000 → 0.000000 | separate | 0.002761 → 0.002771 | independent | 939 / 940 |
| F u = 4 | 0.000000 → 0.000000 | separate | 0.000060 → 0.000061 | independent | 941 / 942 |
| F u = 16 | 0.000000 → 0.000000 | separate | unavailable → unavailable | not_planned | 943 / 944 |

- The fractions move and the words do not: interval S changed. Before, S had lower bound 16777240, upper bound 2438004868 and longest-segment range 24 to 30 (14 pinned smoke streams); now the lower bound is 16777243, the upper bound 5722079403 and the longest-segment range 24 to 32 (artifact line 936; the 12 `name=pinned_smoke` lines plus the four re-recorded streams make 16). The two additional streams are `one_file_overwritten_across_two_writable_remounts_stream` (artifact line 8, two-state 5675286699) and `several_inodes_overwritten_in_turn_stream` (line 10, two-state 5722079403, which set the new upper bound); both test files entered the repository only on 2026-10-21 (`git log --diff-filter=A` on the two test files, commit `0b926bd3f`) and are not in the earlier artifact. The candidate streams' own write counts and state counts also changed (median writes in `name=family_summary`: R 100 steps 2261.5 → 2281.5, R 400 steps 8580.5 → 8682.5), so the movement of the fractions mixes a wider S with changed candidate streams, and this run cannot separate them.
- Distance to the thresholds (the registration's “distance to the threshold” quantity is not run after decidability; these sentences are arithmetic on artifact lines, inferred): R 100 steps is 11 / 20 = 0.55, the threshold 0.5 includes its endpoint, so one fewer stream inside S is still overlap (10 / 20) and two fewer give partial; R 400 steps is 2 / 20 = 0.1, exactly on the endpoint of “≤ 0.1 is separate”, and one more stream falling into S (3 / 20 = 0.15) gives partial; R main sample combined is 32 / 73 = 0.438356, 0.061644 short of 0.5.
- The second part of F6 (among random-history streams with at least 1000 writes, the fraction whose two-state closed form does not exceed S's upper bound is ≥ 0.5): the combined scope has `at_most_smoke_interval_maximum_fraction=0.438356` (line 938), not triggered, same as before; read per group, 50 steps 0.923077, 100 steps 0.550000, 240-slot disk width 1.000000 and maintenance weighting 0.789474 are ≥ 0.5, while 200 steps 0.350000 and 400 steps 0.100000 fall short, the same four groups as before.
- By step level the interval overlap is still monotone for random histories: 50 steps overlap, 100 steps overlap, 200 steps partial, 400 steps separate (inside fractions 0.923077, 0.550000, 0.350000, 0.100000).
- Shared block keys: the six random-history groups and overwrite chains u = 1 and 4 are still all independent, with maximum shared-block fraction 0.002261 (R 50 steps, line 945); the nine streams that cannot be planned are the same as before (3 at 200 steps, 5 at 400 steps, overwrite chain u = 16; `name=share_not_planned` is on lines 698, 847, 848, 857, 872, 883, 884, 885, 892).

**Failure, invalidation and stop conditions, one by one** (registration sections 10 and 11; “line N” means the rerun artifact and “comparison” means the comparison output):

| Condition | Reading this time | Conclusion |
|---|---|---|
| FR1 the inferred “no flip” is overturned | comparison lines 81 and 82 `differing=0`, `verdict=not_flipped`; `grep -c 'same=false' <comparison>` gives 0 | not triggered |
| FR2 B1 no longer triggers F6 | line 77 `multi_record_full_named_by_the_clause` `match=false`, line 78 `clause_anchor_matches=false` | not triggered (the first part of F6 still triggers, as the registration expected) |
| FR3 F1, F2, F3 or the second part of F6 changes side | comparison lines 63 to 66, four lines `same=true` | not triggered |
| XR1 positive control failed | comparison line 77 `name=void_check family=control cells=174 failing=0`; zero `passed=false` lines in the artifact; the `name=segment` lines of the 17 freshly run jobs are all `state=done runs=1`; the long-run exit-code file holds 0 | not triggered |
| XR2 random-history determinism | comparison line 78 `family=determinism cells=12 failing=0`; 12 `equal=true` lines (artifact lines 184, 185, 275, 276, 366, 367, 457, 458, 548, 549, 639, 640) | not triggered |
| XR3 base image rebuilt from the dump | comparison line 79 `family=dump_done cells=6 failing=0`; all six `dump_done` lines have `base_mismatches=0` (lines 186, 277, 368, 459, 550, 641) | not triggered |
| smoke-tier pinned values | comparison line 80 `family=pinned_smoke cells=12 failing=0`; line 38 `pinned_smoke_done pinned=12 missing=22 mismatches=0 t6_hits=0` | not triggered |
| XR4 artifact incomplete | final line `name=done segments=18 emitted=959` equals the 959 of `wc -l`; of the 18 `name=segment` lines 17 are `state=done` and `library` is `not_run` (the registration counts that as complete) | not triggered |
| XR5 comparison unit test failed | `python3 scripts/experiment-apparatus/e184_r2_verdict_compare_test.py` final line “自证 16 项，红 0 项” | not triggered |
| X6, X7 | no unobtainable value was written as 0 (the 9 `not_planned` and the `not_run` of `library` are reported as they are); the 17 freshly run jobs all have `runs=1` with no restart, and the long-run log has no trace of the memory wrapper exiting with 245 or 250 to 254 | not triggered |
| S1 five checks before the start | checked in this segment: the `sha256` of the two apparatus source files equals the registration header; the long-run script `c1f9176e…` is the value recorded in registration section 12 amendment 2; `sha256sum` of the earlier artifact gives `7eacaffa…`; T6 is on artifact line 3, `result=pass hits=0 files_read=19`. That the working directory was new and the output of `--list-segments` were seen by the main agent at the start, and this segment did not see the original output | everything this segment could check holds; two items not seen |
| S2 long run exits 3 or 4 | all 29 `stop_check` cells of stop conditions T1 to T4 in the artifact are `result=pass` (`grep -c 'result=fail'` gives 0), `selfcheck_done failed=0 anchors_mismatching=0` | not triggered |
| S3 recomputation disagrees with the recording | all 10 `anchor_recompute` cells are `same=true` (comparison lines 67 to 76) | not triggered |
| S4 cells of the two artifacts do not line up | comparison lines 81 and 82 `verdict=not_flipped`, not `keys_differ`; the output has no `absent` or `duplicate` cell (`grep -c absent <comparison>` and `grep -c duplicate <comparison>` both give 0), and the script exits 0 | not triggered |

Both rows are decidable: row one, every number of registration section 7 is “new number settled”, the 10 cells are `same=true` and the self-check exits 0; row two, none of XR1 to XR5 triggered, and both the narrow and the wide reading record “not flipped”. The registration's “distance to the threshold” is marked “not run after decidability”, and segments two through four of the original registration are not covered this time.

**Verdict lines and non-passing fields**:

```bash
C=research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out
P=research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out
grep -n 'name=verdict' $C
for pat in 'name=verdict' 'passed=false' 'result=fail' 'equal=false'; do echo "$pat $(grep -c "$pat" $P)"; done
grep -cE 'failed=[1-9]|controls_failed=[1-9]|count_disagreements=[1-9]|base_mismatches=[1-9]|anchors_mismatching=[1-9]|mismatches=[1-9]' $P
grep -n 'match=false' $P
```

```text
81:E7RESULT name=verdict_comparison reading=narrow cells=40 differing=0 verdict=not_flipped
82:E7RESULT name=verdict_comparison reading=wide cells=62 differing=0 verdict=not_flipped
name=verdict 0
passed=false 0
result=fail 0
equal=false 0
0
77:E7RESULT name=anchor identifier=multi_record_full_named_by_the_clause expected=12230590578 actual=1358954642 match=false
```

The comparison output has 2 verdict lines (`name=verdict_comparison`), both with value `not_flipped` and `differing=0`. The rerun artifact prints no `name=verdict` lines; there is no `result=fail`, `passed=false` or `equal=false`, and the counts of violations, mismatches and failures are all 0. The fields that did not pass or could not be obtained, one by one:

| Field | Position | Value | Why this value | Expected by the registration? |
|---|---|---|---|---|
| `match=false` (B1) and `clause_anchor_matches=false` | artifact lines 77 and 78; comparison lines 39, 40 and 76 (`before=false after=false same=true`) | false | D13 (Verification Route) settled item 9's scope says 12230590578, and the apparatus re-recording of the multi-record stream gives 1358954642 (the clause's number predates the segment-shape change) | expected (registration section 4 and 7.1); the first part of F6 still triggers and FR2 does not |
| `before=false after=false` of the derived clauses F1, F2, F3 and the second part of F6 | comparison lines 63 to 66 | false | the value false means “the clause was not triggered”, not a failure | expected |
| `blocks_full=not_planned` (9 `name=flow` lines), `name=share_not_planned` (9 lines), `verdict=not_planned`, `keys_equal=not_planned` | artifact lines 113, 588, 589, 594, 673, 679, 680, 681, 685; lines 698, 847, 848, 857, 872, 883, 884, 885, 892; line 943; line 85 | not_planned | the block-count upper bound of the full-run plan exceeds 2^26, so no plan is built (original registration amendment 10) | expected |
| `state=not_run` (the `library` job) | artifact line 958 | not_run | the long-term library item runs only when a family's shared-block-key verdict is partial, and the six random-history groups and the overwrite chains are all independent | expected |

**Fork table** (the two rows of the question sheet):

| Question-sheet row | Status | What is still missing | Can the remaining quantities reverse it? | Command that computes it |
|---|---|---|---|---|
| Row one: what anchors A1, anchor 2, A5, anchor 7 and X3 take on the new segment shapes | Decidable: every “new number settled” holds, the 10 cells are `same=true`, and the self-check exits 0 | write the new X3 value 16842815 into the apparatus constant and the two hand-made-reading test cases (before segment three starts; who does it is for the main agent to decide); candidate A (appending to section 12 of the original registration) is not pursued; B is done | No: only a cell with `same=false` would reverse it, and none occurred; no other quantity in the registration touches this row | `grep -n 'name=anchor_recompute' <comparison>`; `grep -n 'name=selfcheck_done' <rerun artifact>` |
| Row two: do the segment-one verdict words flip | Decidable: not flipped (narrow reading 40 cells, wide reading 62 cells, 4 derived clauses, 0 words changed) | nothing | No: what remains is only “distance to the threshold” (incidental, not run after decidability) and segments two through four of the original registration (not covered this time); whether it flips depends only on whether the word of the same cell in the two artifacts changed | `grep -n 'name=verdict_comparison' <comparison>`; `grep -c 'same=false' <comparison>` (gives 0) |

**What this cannot answer**:

- That the verdict words do not flip says only that today's tree gives the same words as before; how much of the movement of the fractions comes from the two-round number acquisition and owed check C755 (recovery and two mount triggers fail differently on rollback), and how much from other changes to `crates/` since 2026-10-19 and from streams newly in the smoke tier, this run cannot separate (registration 1.1).
- The verdict words are literal results of thresholds and are not robust: the R 400 steps level `inside_fraction=0.100000` sits exactly on the endpoint of “≤ 0.1 is separate” (line 952), and R 100 steps at 0.550000 (11 / 20) is only one stream from 0.5 (line 948); a different batch of seeds, or one more smoke stream in S, could change the word.
- Interval S still contains only smoke streams whose two-state closed form is pinned (12 plus 4 re-recorded) and none of the unpinned ones: `name=pinned_smoke_missing` went from 15 lines to 22 (between artifact lines 6 and 37), among them `large-file-overwritten-across-lower-leaves-stream` with 1962 writes and longest segment 324 (artifact line 6; its two-state closed form is a 98-digit decimal number), above every family of candidate streams; `above` is therefore still relative to S and not to the true upper bound of the smoke tier. The largest write count among smoke streams with pinned constants went from 824 to 828, and candidate streams with at least 1000 writes are still all above it.
- Whether X3 really judges by 16842815 on the new segment shapes, and the first half of anchor 7, must wait for the J2 cold run of segment three; this time only the numbers are settled.
- The entries of “What this cannot answer” for the 2026-10-19 artifact that this run did not change (how version tables are taken, 20 seeds per level, sequential writes for overwrite chains, the arithmetic upper bounds of the 9 unobtainable streams, the discriminative-power self-test not located) hold for this artifact as well; when using their numbers, recompute them from this artifact.
- The cumulative 8403 seconds is not comparable with the earlier 5462 seconds: machine load and thread settings of the two runs were not aligned, so wall-clock numbers are not judged.

**Replay registration**: the whole segment-one artifact is still not registered for byte-exact replay (it carries wall-clock fields), and `scripts/experiment-apparatus/replay.sh` was not changed. The E184 (Cost Counts of Regular Layer and Large Workload Layer) line in the replay table, `@driver_e184`, runs only `selfcheck` and expects `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`; the `selfcheck` job output of this run is byte-identical to it, so that line does not need to be repointed:

```bash
diff <(sed -n 40,78p research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out) <(head -n 39 research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out); echo rc=$?
```

```text
rc=0
```

The two “not registered” lines for this experiment's two comparison objects in `.claude/gate.d/replay-comparison-units.tsv` (the header there records the owed-check number and the reason) are left alone: comparison object 1 is the whole earlier artifact, compared by the cell-by-cell script; comparison object 2 is the retained `selfcheck` artifact of 2026-10-23, which the `diff` in this section and the 10 `anchor_recompute` cells have already compared line by line.

The line numbers and figures in the reading sections for the 2026-10-19 artifact (“Readings: scale”, “Readings: state-count interval overlap and shared block keys”, “8.2 Geometry sensitivity: does the verdict flip?”, “Verdict lines and non-passing fields”, “Invalidation, failure and stop conditions, individually (segment one)”) belong to that earlier artifact and were not changed; this run's numbers are those of this section.

### Readings: scale

Lines 126, 127, 128, 129, 130, 131, 132, 223, 314, 405, 496, 587 and 678 of `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` (`name=family_summary`, one per group, each containing minimum / median / maximum; this output has wall-clock fields and is not registered for replay; command and verbatim output follow):

```text
$ grep -n 'name=family_summary' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
126:E7RESULT name=family_summary family=overwrite_chain group=overwrite_chain-01_units_per_overwrite flows=1 flows_with_writes_at_least_1000=1 flows_beyond_u64=0 writes=[minimum=1003 median=1003 maximum=1003] segments=[minimum=131 median=131 maximum=131] longest_segment=[minimum=28 median=28 maximum=28] log2_two_state=[minimum=32.554588898332504 median=32.554588898332504 maximum=32.554588898332504] log2_full=[minimum=32.55458893721156 median=32.55458893721156 maximum=32.55458893721156] log2_quick_tier=[minimum=8.768184324776927 median=8.768184324776927 maximum=8.768184324776927] blocks_full=[minimum=96357 median=96357 maximum=96357]
127:E7RESULT name=family_summary family=overwrite_chain group=overwrite_chain-04_units_per_overwrite flows=1 flows_with_writes_at_least_1000=1 flows_beyond_u64=0 writes=[minimum=1042 median=1042 maximum=1042] segments=[minimum=103 median=103 maximum=103] longest_segment=[minimum=36 median=36 maximum=36] log2_two_state=[minimum=38.087545743703906 median=38.087545743703906 maximum=38.087545743703906] log2_full=[minimum=38.087545744370736 median=38.087545744370736 maximum=38.087545744370736] log2_quick_tier=[minimum=12.584257887744567 median=12.584257887744567 maximum=12.584257887744567] blocks_full=[minimum=4456784 median=4456784 maximum=4456784]
128:E7RESULT name=family_summary family=overwrite_chain group=overwrite_chain-16_units_per_overwrite flows=1 flows_with_writes_at_least_1000=1 flows_beyond_u64=0 writes=[minimum=1050 median=1050 maximum=1050] segments=[minimum=55 median=55 maximum=55] longest_segment=[minimum=60 median=60 maximum=60] log2_two_state=[minimum=61.35755202769712 median=61.35755202769712 maximum=61.35755202769712] log2_full=[minimum=61.35755202769712 median=61.35755202769712 maximum=61.35755202769712] log2_quick_tier=[minimum=35.459431623065115 median=35.459431623065115 maximum=35.459431623065115] blocks_full=[minimum=none median=none maximum=none]
129:E7RESULT name=family_summary family=smoke group=smoke-multi_record_publish_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=115 median=115 maximum=115] segments=[minimum=19 median=19 maximum=19] longest_segment=[minimum=30 median=30 maximum=30] log2_two_state=[minimum=30.339850117539722 median=30.339850117539722 maximum=30.339850117539722] log2_full=[minimum=30.339850149388358 median=30.339850149388358 maximum=30.339850149388358] log2_quick_tier=[minimum=7.169925001442312 median=7.169925001442312 maximum=7.169925001442312] blocks_full=[minimum=20753 median=20753 maximum=20753]
130:E7RESULT name=family_summary family=smoke group=smoke-new_pool_file_creation_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=41 median=41 maximum=41] segments=[minimum=11 median=11 maximum=11] longest_segment=[minimum=24 median=24 maximum=24] log2_two_state=[minimum=24.000002063790333 median=24.000002063790333 maximum=24.000002063790333] log2_full=[minimum=24.00000378361336 median=24.00000378361336 maximum=24.00000378361336] log2_quick_tier=[minimum=5.523561956057013 median=5.523561956057013 maximum=5.523561956057013] blocks_full=[minimum=267 median=267 maximum=267]
131:E7RESULT name=family_summary family=smoke group=smoke-overwrite_fixed_script_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=481 median=481 maximum=481] segments=[minimum=78 median=78 maximum=78] longest_segment=[minimum=28 median=28 maximum=28] log2_two_state=[minimum=30.64441974337032 median=30.64441974337032 maximum=30.64441974337032] log2_full=[minimum=30.644419842221723 median=30.644419842221723 maximum=30.644419842221723] log2_quick_tier=[minimum=8.118941072723507 median=8.118941072723507 maximum=8.118941072723507] blocks_full=[minimum=25672 median=25672 maximum=25672]
132:E7RESULT name=family_summary family=smoke group=smoke-snapshot_creation_stream flows=1 flows_with_writes_at_least_1000=0 flows_beyond_u64=0 writes=[minimum=62 median=62 maximum=62] segments=[minimum=15 median=15 maximum=15] longest_segment=[minimum=24 median=24 maximum=24] log2_two_state=[minimum=24.005627118893454 median=24.005627118893454 maximum=24.005627118893454] log2_full=[minimum=24.00562926030627 median=24.00562926030627 maximum=24.00562926030627] log2_quick_tier=[minimum=5.882643049361842 median=5.882643049361842 maximum=5.882643049361842] blocks_full=[minimum=271 median=271 maximum=271]
223:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0050 flows=20 flows_with_writes_at_least_1000=12 flows_beyond_u64=0 writes=[minimum=382 median=1039.5 maximum=1464] segments=[minimum=82 median=170 maximum=229] longest_segment=[minimum=26 median=28 maximum=30] log2_two_state=[minimum=27.674636274596462 median=30.37566688831223 maximum=32.425363506393886] log2_full=[minimum=27.709519939008434 median=30.421581409516403 maximum=33.95203027866784] log2_quick_tier=[minimum=8.285402218862249 median=9.363596285096868 maximum=12.81117397072132] blocks_full=[minimum=3527 median=22108 maximum=253700]
314:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0100 flows=20 flows_with_writes_at_least_1000=20 flows_beyond_u64=0 writes=[minimum=1008 median=2261.5 maximum=2752] segments=[minimum=195 median=366.5 maximum=474] longest_segment=[minimum=26 median=30 maximum=36] log2_two_state=[minimum=28.599915012358327 median=31.084452500385645 maximum=36.21589557685162] log2_full=[minimum=28.930740613687696 median=33.9700202398081 maximum=40.70491949311818] log2_quick_tier=[minimum=9.49585502688717 median=11.9788464410077 maximum=16.960827787028194] blocks_full=[minimum=8105 median=260561.5 maximum=27348049]
405:E7RESULT name=family_summary family=random_history group=random_history-broad-UnitAreaOf240Slots-steps0100 flows=20 flows_with_writes_at_least_1000=19 flows_beyond_u64=0 writes=[minimum=835 median=3571.5 maximum=5888] segments=[minimum=175 median=825.5 maximum=1391] longest_segment=[minimum=20 median=24 maximum=26] log2_two_state=[minimum=22.96464898372668 median=25.891453207134532 maximum=27.486779525829036] log2_full=[minimum=22.96481112358067 median=25.891474243909215 maximum=28.048113272159032] log2_quick_tier=[minimum=9.467605550082997 median=11.5413876819633 maximum=12.296916206879288] blocks_full=[minimum=688 median=1653 maximum=5148]
496:E7RESULT name=family_summary family=random_history group=random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100 flows=20 flows_with_writes_at_least_1000=19 flows_beyond_u64=0 writes=[minimum=936 median=2166 maximum=2583] segments=[minimum=190 median=367 maximum=437] longest_segment=[minimum=26 median=30 maximum=34] log2_two_state=[minimum=28.92240039275743 median=31.136029108832314 maximum=34.47804282673022] log2_full=[minimum=29.32235347425211 median=31.814965400572902 maximum=39.872878591274734] log2_quick_tier=[minimum=9.535275376620802 median=11.60110735828604 maximum=16.87758370515652] blocks_full=[minimum=10544 median=57976 maximum=15362357]
587:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0200 flows=20 flows_with_writes_at_least_1000=20 flows_beyond_u64=0 writes=[minimum=2912 median=4327.5 maximum=5509] segments=[minimum=528 median=719.5 maximum=959] longest_segment=[minimum=28 median=32 maximum=36] log2_two_state=[minimum=29.359356900603476 median=33.43418243131835 maximum=36.81238419050781] log2_full=[minimum=29.35935876490942 median=37.32983738795073 maximum=43.40711311699381] log2_quick_tier=[minimum=10.891024190149498 median=16.724180842104673 maximum=23.212240026662197] blocks_full=[minimum=10956 median=1389411 maximum=27594991]
678:E7RESULT name=family_summary family=random_history group=random_history-broad-FourGibibytes-steps0400 flows=20 flows_with_writes_at_least_1000=20 flows_beyond_u64=0 writes=[minimum=4817 median=8580.5 maximum=10707] segments=[minimum=896 median=1430.5 maximum=1836] longest_segment=[minimum=28 median=34 maximum=40] log2_two_state=[minimum=31.460781202958813 median=35.062177803280576 maximum=40.108211687236725] log2_full=[minimum=31.460813602239426 median=40.64768052197316 maximum=49.37876086360342] log2_quick_tier=[minimum=13.613098794209224 median=19.315901203760895 maximum=26.39985199419267] blocks_full=[minimum=45798 median=6454063 maximum=55846164]
```

The two tables are calculated directly from these 13 rows (the script reads the artifact, not crates; the block-count column includes only planned streams, excluding those that could not be planned).

Scale:

| Group | Streams | Streams with ≥ 1000 writes | Writes min / median / max | Segments min / median / max | Longest segment min / median / max | Output line |
|---|---|---|---|---|---|---|
| F u=1 | 1 | 1 | 1003 / 1003 / 1003 | 131 / 131 / 131 | 28 / 28 / 28 | 126 |
| F u=4 | 1 | 1 | 1042 / 1042 / 1042 | 103 / 103 / 103 | 36 / 36 / 36 | 127 |
| F u=16 | 1 | 1 | 1050 / 1050 / 1050 | 55 / 55 / 55 | 60 / 60 / 60 | 128 |
| R 50  steps | 20 | 12 | 382 / 1039.50 / 1464 | 82 / 170 / 229 | 26 / 28 / 30 | 223 |
| R 100  steps（main sample） | 20 | 20 | 1008 / 2261.50 / 2752 | 195 / 366.50 / 474 | 26 / 30 / 36 | 314 |
| R 100  steps 240-slot disk width | 20 | 19 | 835 / 3571.50 / 5888 | 175 / 825.50 / 1391 | 20 / 24 / 26 | 405 |
| R 100  steps maintenance weighting | 20 | 19 | 936 / 2166 / 2583 | 190 / 367 / 437 | 26 / 30 / 34 | 496 |
| R 200  steps | 20 | 20 | 2912 / 4327.50 / 5509 | 528 / 719.50 / 959 | 28 / 32 / 36 | 587 |
| R 400  steps | 20 | 20 | 4817 / 8580.50 / 10707 | 896 / 1430.50 / 1836 | 28 / 34 / 40 | 678 |
| Smoke new-pool file-creation stream | 1 | 0 | 41 / 41 / 41 | 11 / 11 / 11 | 24 / 24 / 24 | 130 |
| Smoke snapshot-creation stream | 1 | 0 | 62 / 62 / 62 | 15 / 15 / 15 | 24 / 24 / 24 | 132 |
| Smoke multi-record publish stream | 1 | 0 | 115 / 115 / 115 | 19 / 19 / 19 | 30 / 30 / 30 | 129 |
| Smoke fixed-script overwrite stream | 1 | 0 | 481 / 481 / 481 | 78 / 78 / 78 | 28 / 28 / 28 | 131 |

State counts (base-2 logarithms):

| Group | log2 two-state closed form min / median / max | log2 full-run states min / median / max | log2 A-two states min / median / max | Full-run blocks min / median / max |
|---|---|---|---|---|
| F u=1 | 32.55 / 32.55 / 32.55 | 32.55 / 32.55 / 32.55 | 8.77 / 8.77 / 8.77 | 96357 / 96357 / 96357 |
| F u=4 | 38.09 / 38.09 / 38.09 | 38.09 / 38.09 / 38.09 | 12.58 / 12.58 / 12.58 | 4456784 / 4456784 / 4456784 |
| F u=16 | 61.36 / 61.36 / 61.36 | 61.36 / 61.36 / 61.36 | 35.46 / 35.46 / 35.46 | unavailable / unavailable / unavailable |
| R 50  steps | 27.67 / 30.38 / 32.43 | 27.71 / 30.42 / 33.95 | 8.29 / 9.36 / 12.81 | 3527 / 22108 / 253700 |
| R 100  steps（main sample） | 28.60 / 31.08 / 36.22 | 28.93 / 33.97 / 40.70 | 9.50 / 11.98 / 16.96 | 8105 / 260561.50 / 27348049 |
| R 100  steps 240-slot disk width | 22.96 / 25.89 / 27.49 | 22.96 / 25.89 / 28.05 | 9.47 / 11.54 / 12.30 | 688 / 1653 / 5148 |
| R 100  steps maintenance weighting | 28.92 / 31.14 / 34.48 | 29.32 / 31.81 / 39.87 | 9.54 / 11.60 / 16.88 | 10544 / 57976 / 15362357 |
| R 200  steps | 29.36 / 33.43 / 36.81 | 29.36 / 37.33 / 43.41 | 10.89 / 16.72 / 23.21 | 10956 / 1389411 / 27594991 |
| R 400  steps | 31.46 / 35.06 / 40.11 | 31.46 / 40.65 / 49.38 | 13.61 / 19.32 / 26.40 | 45798 / 6454063 / 55846164 |
| Smoke new-pool file-creation stream | 24.00 / 24.00 / 24.00 | 24.00 / 24.00 / 24.00 | 5.52 / 5.52 / 5.52 | 267 / 267 / 267 |
| Smoke snapshot-creation stream | 24.01 / 24.01 / 24.01 | 24.01 / 24.01 / 24.01 | 5.88 / 5.88 / 5.88 | 271 / 271 / 271 |
| Smoke multi-record publish stream | 30.34 / 30.34 / 30.34 | 30.34 / 30.34 / 30.34 | 7.17 / 7.17 / 7.17 | 20753 / 20753 / 20753 |
| Smoke fixed-script overwrite stream | 30.64 / 30.64 / 30.64 | 30.64 / 30.64 / 30.64 | 8.12 / 8.12 / 8.12 | 25672 / 25672 / 25672 |

- Writes rise with step level: random 50, 100, 200, 400-step medians are 1039.5, 2261.5, 4327.5, 8580.5 writes; two-state closed-form log2 medians 30.38, 31.08, 33.43, 35.06; full-state log2 medians 30.42, 33.97, 37.33, 40.65. Across the four levels full-run states grow faster than the two-state closed form: the largest 400-step stream reaches 2^49.38 full-run states versus 2^40.11 two-state.
- No stream's two-state closed form exceeds u64 (all 13 `family_summary` rows have `flows_beyond_u64=0`); overwrite u = 16 has 2954361402816462894 (log2 61.36), still within u64. Its longest segment has 60 writes, below 64.
- “Thousands of write requests” separates on the write-count axis: the maximum among 14 pinned smoke streams is 824 (ten constant-pinned streams have 126–824 writes; four re-recorded streams have 41–481); random histories and overwrite chains with ≥ 1000 writes exceed it. But eight of 20 streams in the 50-step level have fewer than 1000 writes, and one in each of the 240-slot and maintenance-weighted groups does too. They do not separate on the state-count axis; see “State-count interval overlap and shared block keys.”

### Readings: state-count interval overlap and shared block keys

Interval S is the two-state closed form of 14 pinned smoke streams: the ten constant-pinned streams on output lines 4–13 (`name=pinned_smoke`, all `match=true`) plus four re-recorded streams (`family_summary`, lines 129–132). Lower bound 16777240 (new-pool file creation), upper bound 2438004868 (`truncate_under_a_snapshot`); longest-segment interval 24–30. Fifteen streams that cannot be pinned are listed on output lines 14–28 (`name=pinned_smoke_missing`, each because `no_segment_shape_constant_with_a_pinned_closed_form`) and are excluded from S.

“State-count interval overlap” (the `inside` fraction is calculated over streams with ≥ 1000 writes; the longest-segment column gives the fraction whose longest segment is 24–30, separately reported as registered and not used in the verdict):

| Scope | Streams with ≥ 1000 writes | Inside S / above S / below S | inside fraction | Fraction with longest segment in smoke longest-segment interval | B verdict | Output line |
|---|---|---|---|---|---|---|
| R main sample combined (80 streams across four step levels) | 72 | 21 / 51 / 0 | 0.291667 | 0.486111 | partial | 929 |
| R 50  steps | 12 | 7 / 5 / 0 | 0.583333 | 1.000000 | overlap | 937 |
| R 100  steps (main sample) | 20 | 10 / 10 / 0 | 0.500000 | 0.600000 | overlap | 939 |
| R 100  steps 240-slot disk width | 19 | 17 / 0 / 2 | 0.894737 | 0.631579 | overlap | 945 |
| R 100  steps maintenance weighting | 19 | 10 / 9 / 0 | 0.526316 | 0.842105 | overlap | 947 |
| R 200  steps | 20 | 4 / 16 / 0 | 0.200000 | 0.450000 | partial | 941 |
| R 400  steps | 20 | 0 / 20 / 0 | 0.000000 | 0.100000 | separate | 943 |
| F main sample (u = 4) | 1 | 0 / 1 / 0 | 0.000000 | 0.000000 | separate | 927 |
| F u = 1 | 1 | 0 / 1 / 0 | 0.000000 | 1.000000 | separate | 931 |
| F u = 4 | 1 | 0 / 1 / 0 | 0.000000 | 0.000000 | separate | 933 |
| F u = 16 | 1 | 0 / 1 / 0 | 0.000000 | 0.000000 | separate | 935 |

“Shared block keys” (shared-block fraction is calculated by block and used for the verdict; reused-key fraction is an upper bound excluding plan hash and version table, separately reported and not used for the verdict):

| Scope | Planned streams / total streams | Shared-block fraction median / max | Reuse-key fraction (upper bound), median / max | State-weighted fraction median | C verdict | Output line |
|---|---|---|---|---|---|---|
| R main sample combined (80 streams across four step levels) | 72 / 80 | 0.000024 / 0.001985 | 0.000161 / 0.053737 | 0.000000 | independent | 928 |
| R 50  steps | 20 / 20 | 0.000284 / 0.001985 | 0.001355 / 0.053737 | 0.000000 | independent | 936 |
| R 100  steps (main sample) | 20 / 20 | 0.000028 / 0.000864 | 0.000222 / 0.007968 | 0.000000 | independent | 938 |
| R 100  steps 240-slot disk width | 20 / 20 | 0.000000 / 0.000000 | 0.000000 / 0.000000 | 0.000000 | independent | 944 |
| R 100  steps maintenance weighting | 20 / 20 | 0.000121 / 0.000664 | 0.000172 / 0.025228 | 0.000000 | independent | 946 |
| R 200  steps | 17 / 20 | 0.000005 / 0.000548 | 0.000062 / 0.005095 | 0.000000 | independent | 940 |
| R 400  steps | 15 / 20 | 0.000001 / 0.000153 | 0.000004 / 0.000153 | 0.000000 | independent | 942 |
| F main sample (u = 4) | 1 / 1 | 0.000060 / 0.000060 | 0.000060 / 0.000060 | 0.000057 | independent | 926 |
| F u = 1 | 1 / 1 | 0.002761 / 0.002761 | 0.002761 / 0.002761 | 0.002660 | independent | 930 |
| F u = 4 | 1 / 1 | 0.000060 / 0.000060 | 0.000060 / 0.000060 | 0.000057 | independent | 932 |
| F u = 16 | 0 / 1 | unavailable | unavailable | unavailable | not_planned（unavailable） | 934 |

The verdict words in both tables come from 22 `name=share_group` and `name=overlap` lines, output lines 926–947 (the output-line column identifies each scope; the artifact is not registered for replay, so command and verbatim output are shown):

```text
$ grep -nE 'name=(share_group|overlap) ' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
926:E7RESULT name=share_group scope=family:overwrite_chain_main flows=1 planned_flows=1 median_share_fraction=0.000060 maximum_share_fraction=0.000060 median_weighted_fraction=0.000057 median_reuse_fraction=0.000060 maximum_reuse_fraction=0.000060 verdict=independent
927:E7RESULT name=overlap scope=family:overwrite_chain_main smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.000000 verdict=separate
928:E7RESULT name=share_group scope=family:random_history_main flows=80 planned_flows=72 median_share_fraction=0.000024 maximum_share_fraction=0.001985 median_weighted_fraction=0.000000 median_reuse_fraction=0.000161 maximum_reuse_fraction=0.053737 verdict=independent
929:E7RESULT name=overlap scope=family:random_history_main smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=80 flows_with_writes_at_least_1000=72 inside=21 above=51 below=0 inside_fraction=0.291667 above_fraction=0.708333 at_most_smoke_interval_maximum_fraction=0.291667 longest_segment_inside_fraction=0.486111 verdict=partial
930:E7RESULT name=share_group scope=group:overwrite_chain-01_units_per_overwrite flows=1 planned_flows=1 median_share_fraction=0.002761 maximum_share_fraction=0.002761 median_weighted_fraction=0.002660 median_reuse_fraction=0.002761 maximum_reuse_fraction=0.002761 verdict=independent
931:E7RESULT name=overlap scope=group:overwrite_chain-01_units_per_overwrite smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=1.000000 verdict=separate
932:E7RESULT name=share_group scope=group:overwrite_chain-04_units_per_overwrite flows=1 planned_flows=1 median_share_fraction=0.000060 maximum_share_fraction=0.000060 median_weighted_fraction=0.000057 median_reuse_fraction=0.000060 maximum_reuse_fraction=0.000060 verdict=independent
933:E7RESULT name=overlap scope=group:overwrite_chain-04_units_per_overwrite smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.000000 verdict=separate
934:E7RESULT name=share_group scope=group:overwrite_chain-16_units_per_overwrite flows=1 planned_flows=0 verdict=not_planned
935:E7RESULT name=overlap scope=group:overwrite_chain-16_units_per_overwrite smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=1 flows_with_writes_at_least_1000=1 inside=0 above=1 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.000000 verdict=separate
936:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0050 flows=20 planned_flows=20 median_share_fraction=0.000284 maximum_share_fraction=0.001985 median_weighted_fraction=0.000000 median_reuse_fraction=0.001355 maximum_reuse_fraction=0.053737 verdict=independent
937:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0050 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=12 inside=7 above=5 below=0 inside_fraction=0.583333 above_fraction=0.416667 at_most_smoke_interval_maximum_fraction=0.583333 longest_segment_inside_fraction=1.000000 verdict=overlap
938:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0100 flows=20 planned_flows=20 median_share_fraction=0.000028 maximum_share_fraction=0.000864 median_weighted_fraction=0.000000 median_reuse_fraction=0.000222 maximum_reuse_fraction=0.007968 verdict=independent
939:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0100 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=20 inside=10 above=10 below=0 inside_fraction=0.500000 above_fraction=0.500000 at_most_smoke_interval_maximum_fraction=0.500000 longest_segment_inside_fraction=0.600000 verdict=overlap
940:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0200 flows=20 planned_flows=17 median_share_fraction=0.000005 maximum_share_fraction=0.000548 median_weighted_fraction=0.000000 median_reuse_fraction=0.000062 maximum_reuse_fraction=0.005095 verdict=independent
941:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0200 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=20 inside=4 above=16 below=0 inside_fraction=0.200000 above_fraction=0.800000 at_most_smoke_interval_maximum_fraction=0.200000 longest_segment_inside_fraction=0.450000 verdict=partial
942:E7RESULT name=share_group scope=group:random_history-broad-FourGibibytes-steps0400 flows=20 planned_flows=15 median_share_fraction=0.000001 maximum_share_fraction=0.000153 median_weighted_fraction=0.000000 median_reuse_fraction=0.000004 maximum_reuse_fraction=0.000153 verdict=independent
943:E7RESULT name=overlap scope=group:random_history-broad-FourGibibytes-steps0400 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=20 inside=0 above=20 below=0 inside_fraction=0.000000 above_fraction=1.000000 at_most_smoke_interval_maximum_fraction=0.000000 longest_segment_inside_fraction=0.100000 verdict=separate
944:E7RESULT name=share_group scope=group:random_history-broad-UnitAreaOf240Slots-steps0100 flows=20 planned_flows=20 median_share_fraction=0.000000 maximum_share_fraction=0.000000 median_weighted_fraction=0.000000 median_reuse_fraction=0.000000 maximum_reuse_fraction=0.000000 verdict=independent
945:E7RESULT name=overlap scope=group:random_history-broad-UnitAreaOf240Slots-steps0100 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=19 inside=17 above=0 below=2 inside_fraction=0.894737 above_fraction=0.000000 at_most_smoke_interval_maximum_fraction=1.000000 longest_segment_inside_fraction=0.631579 verdict=overlap
946:E7RESULT name=share_group scope=group:random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100 flows=20 planned_flows=20 median_share_fraction=0.000121 maximum_share_fraction=0.000664 median_weighted_fraction=0.000000 median_reuse_fraction=0.000172 maximum_reuse_fraction=0.025228 verdict=independent
947:E7RESULT name=overlap scope=group:random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100 smoke_interval_two_state_minimum=16777240 smoke_interval_two_state_maximum=2438004868 smoke_interval_longest_segment_minimum=24 smoke_interval_longest_segment_maximum=30 flows=20 flows_with_writes_at_least_1000=19 inside=10 above=9 below=0 inside_fraction=0.526316 above_fraction=0.473684 at_most_smoke_interval_maximum_fraction=0.526316 longest_segment_inside_fraction=0.842105 verdict=overlap
```

- By random-history step level, state-count overlap moves monotonically from overlap at 50 steps to separate at 400: `inside` fractions 0.583333, 0.500000, 0.200000, 0.000000. The 100-step group has exactly ten streams inside S and ten above, fraction exactly at the 0.5 threshold, hence overlap (`inside_fraction >= 0.5`). Its 20 sorted two-state closed forms are the command's output below; the tenth, 2092827143, is inside S, the eleventh, 2477261445, exceeds the upper bound 2438004868 by 1.6%:

```bash
grep 'name=flow family=random_history' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep 'label=random_history-broad-FourGibibytes-steps0100-' | grep -o ' two_state=[0-9]*' | cut -d= -f2 | sort -n | paste -sd' '
```

```text
406848100 604381514 617218498 1149051398 1545666995 1576928306 1617625859 1788805901 1827996579 2092827143 2477261445 7110525483 8232043213 9730458182 9900261778 18592432719 18648662610 22181118834 76742721965 79812495196
```

  Moving just one inside stream above the bound would make this level partial (9 inside, 0.45). The maintenance-weighted group is likewise close to the threshold (10 / 19 = 0.526316; one fewer inside yields 9 / 19 = 0.473684, partial); the 240-slot group at 17 / 19 is far from it.
- Shared block keys are independent in all scopes and shared blocks are rare: overwrite u = 1 shares 266 of 96357 blocks with smoke (output line 686, 16777259 shared states); the smoke new-pool file-creation stream has 267 blocks and 16777260 full-run states. The shared blocks should be its 266 blocks because overwrite and that stream begin with the same standard prefix, but this is inferred; the apparatus does not identify each block's origin. Overwrite u = 4 also shares 266 of 4456784 blocks. All 20 240-slot streams have `shared_blocks=0`.
- Reused-key fraction is an upper bound on block-key sharing: it reaches 0.053737 on the 50-step seed-06 stream (7 shared blocks of 4950, line 747), crossing the 0.05 threshold, but the verdict uses shared-block fraction (0.001414), so it does not change. The maximum reused-key fraction among all other streams is 0.025228 (maintenance-weighted seed 09, line 905).
- Streams that cannot be planned (block upper bound > 2^26): three random 200-step streams (seeds 12, 13, 18), five 400-step streams (06, 12, 13, 14, 18), and overwrite u = 16, nine in total. Their shared-key values are unavailable; amendment 10 specifies not to report zero or independent. Shared blocks cannot exceed 46963, the sum of blocks in the four re-recorded smoke streams (`blocks_full` 267, 271, 20753, 25672 in `name=flow`); each unplanned stream has at least “full-run states ÷ 65536” blocks. Dividing 46963 by this block count gives a shared-fraction upper bound: the largest of nine is 200-step seed 18 at 0.000494 (inferred from the artifact's `full=`, not measured by the apparatus), and u = 16 has 1.04e-9. This arithmetic bound does not replace `not_planned`. Commands and per-stream output are in “What this cannot answer.”

### 8.2 Geometry sensitivity: does the verdict flip?

Registration section 8.2 rejudges the same threshold at opposite-direction sample points: a different result from the main sample is “unstable,” the same result records two points.

| Verdict | Main sample | Opposite-direction sample points | Record |
|---|---|---|---|
| State-count interval overlap (random histories) | BROAD, 4 GiB, 100 steps: overlap (0.500000, output line 939) | 240-slot disk: overlap (0.894737, line 945); maintenance weighting: overlap (0.526316, line 947) | All three equal; record two points |
| Shared block keys (random histories) | independent (line 938) | 240-slot independent (line 944); maintenance weighting independent (line 946) | All three equal; record two points |
| State-count interval overlap (overwrite chains) | u = 4: separate (line 933) | u = 1 separate (line 931); u = 16 separate (line 935) | All three equal; record two points |
| Shared block keys (overwrite chains) | u = 4: independent (line 932) | u = 1 independent (line 930); u = 16 unavailable (line 934 `verdict=not_planned`) | u = 16 unavailable; neither two points nor stability recorded |
| State-count interval overlap (step counts) | All streams with ≥ 1000 writes combined: partial (0.291667, line 929) | 50-step group only: overlap (0.583333, line 937) | Different verdicts; “unstable” |

Step levels other than 100 also flip (200 steps partial, line 941; 400 separate, line 943): state-count interval overlap is not a step-independent verdict for random histories, moving from overlap to separate with step count. The last sentence of registration section 8 requires a self-test of discriminative power (move the threshold between two sample values and the verdict must change sides). Only two apparatus unit tests for inclusive endpoints and the partial band were found (`the_share_verdict_has_inclusive_thresholds_and_a_partial_band`, `the_overlap_verdict_has_inclusive_thresholds_and_a_partial_band`), not a flip test using two sample-point values. Mark this cell “not verified”; refer to the main agent to decide whether to add one.

### Verdict lines and non-passing fields

The output does not emit `name=verdict`; verdict words are in the `verdict=` fields of `name=share_group` and `name=overlap`. Commands and verbatim output for all 22 are in “State-count interval overlap and shared block keys.” Counts:

```bash
P=research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
grep -c 'name=verdict' $P
grep -c 'passed=false' $P
grep -c 'result=fail' $P
grep -c 'equal=false' $P
grep -cE 'failed=[1-9]|controls_failed=[1-9]|count_disagreements=[1-9]|base_mismatches=[1-9]|anchors_mismatching=[1-9]|mismatches=[1-9]' $P
grep -n 'match=false' $P
```

```text
0
0
0
0
0
68:E7RESULT name=anchor id=B1_multi_record_full_named_by_the_clause expected=12230590578 actual=1358954634 match=false
```

Verbatim from lines 38–39 of the 2026-10-19 `selfcheck` retained artifact `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` (lines 68–69 of full output; lines 38–39 of `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`, after the pinned-value fix, are `actual=1358954642 match=false` and the same `selfcheck_done`):

```text
38:E7RESULT name=anchor identifier=multi_record_full_named_by_the_clause expected=12230590578 actual=1358954634 match=false
39:E7RESULT name=selfcheck_done stop_checks=28 failed=0 anchors=10 anchors_mismatching=0 clause_anchor_matches=false emitted=39
```

Each non-passing or unavailable field (the `verdict=` values overlap, separate, partial and independent are verdict words, not failures):

| Field | Lines in the artifact | Value | Why it has this value | Within the registered expectations? |
|---|---|---|---|---|
| `match=false` (B1) and `b1_matches=false` | 68, 69 | false | D13 settled item 9's scope states 12230590578 states for the multi-record stream; re-recording gives 1358954634. The multi-record stream test header (`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs` line 22) records 12230590578 and 5435818083 from before C577 (Newest Root Not Witnessed by the System Configuration Is Not Covered by B) changed segment shape, when rotation was included in unit segments. The provision's number predates that change. | Expected (registration section 4 item 2), triggers the first F6 failure observation, not invalidating |
| `blocks_full=not_planned` (nine `name=flow`), `name=share_not_planned` (nine), `verdict=not_planned` (line 934), `keys_equal=not_planned` (line 76) | 104, 579, 580, 585, 664, 670, 671, 672, 676, 689, 838, 839, 848, 863, 874, 875, 876, 883, 934, 76 | not_planned | The full-run-plan block upper bound exceeds 2^26, so no plan is made (amendment 10); shared keys are unavailable for these nine. | Expected under amendment 10; overwrite u = 16 was anticipated; eight random histories were not anticipated in the registration but fall under amendment 10's upper-bound rule. |
| `state=not_run` (`library` job) | 949 | not_run | The long-term-library item runs only when a family is partial on shared keys; all six random groups and overwrite chains are independent. | Expected (registration section 6 library row) |

`name=flow` and `name=share_not_planned` on output lines 104, 579, 580, 585, 664, 670, 671, 672, 676, 689, 838, 839, 848, 863, 874, 875, 876, 883 (output not registered for replay; command and verbatim output):

```text
$ grep -nE 'blocks_full=not_planned|name=share_not_planned' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
104:E7RESULT name=flow family=overwrite_chain tier=candidate label=overwrite_chain-16_units_per_overwrite seed=none ending=NotAHistory:overwrites=11 writes=1050 segments=55 longest_segment=60 two_state=2954361402816462894 log2_two_state=61.3576 full=2954361402816462969 quick_tier=47244640401 blocks_full=not_planned blocks_quick=720941 points=56 keys_seconds=1.3 two_state_agrees_with_crates=true reused=false
579:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0200-seed12 seed=7463871032432355125 ending=Completed writes=4619 segments=759 longest_segment=36 two_state=88704156932 log2_two_state=36.3683 full=9921447726162 quick_tier=1203896 blocks_full=not_planned blocks_quick=777 points=760 keys_seconds=29.1 two_state_agrees_with_crates=true reused=false
580:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0200-seed13 seed=7463871032432355126 ending=Completed writes=5470 segments=892 longest_segment=36 two_state=120679041817 log2_two_state=36.8124 full=11663880818977 quick_tier=2580113 blocks_full=not_planned blocks_quick=928 points=893 keys_seconds=121.7 two_state_agrees_with_crates=true reused=false
585:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0200-seed18 seed=7463871032432355131 ending=Completed writes=5084 segments=857 longest_segment=34 two_state=47264499128 log2_two_state=35.4600 full=6233485413873 quick_tier=9718081 blocks_full=not_planned blocks_quick=1004 points=858 keys_seconds=29.0 two_state_agrees_with_crates=true reused=false
664:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed06 seed=7463871032432355119 ending=Completed writes=10707 segments=1836 longest_segment=36 two_state=157541863687 log2_two_state=37.1969 full=18690168524082 quick_tier=3347048 blocks_full=not_planned blocks_quick=1885 points=1837 keys_seconds=108.4 two_state_agrees_with_crates=true reused=false
670:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed12 seed=7463871032432355125 ending=Completed writes=9317 segments=1564 longest_segment=36 two_state=98945404591 log2_two_state=36.5259 full=10165605368686 quick_tier=1338332 blocks_full=not_planned blocks_quick=1582 points=1565 keys_seconds=55.7 two_state_agrees_with_crates=true reused=false
671:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed13 seed=7463871032432355126 ending=Completed writes=10422 segments=1730 longest_segment=36 two_state=164590854855 log2_two_state=37.2601 full=18429129377335 quick_tier=56989467 blocks_full=not_planned blocks_quick=2587 points=1731 keys_seconds=57.2 two_state_agrees_with_crates=true reused=false
672:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed14 seed=7463871032432355127 ending=Completed writes=8931 segments=1486 longest_segment=40 two_state=1185154009993 log2_two_state=40.1082 full=731961382353473 quick_tier=88541593 blocks_full=not_planned blocks_quick=2834 points=1487 keys_seconds=43.6 two_state_agrees_with_crates=true reused=false
676:E7RESULT name=flow family=random_history tier=candidate label=random_history-broad-FourGibibytes-steps0400-seed18 seed=7463871032432355131 ending=Completed writes=9607 segments=1606 longest_segment=36 two_state=127927716645 log2_two_state=36.8965 full=15453626179150 quick_tier=10948412 blocks_full=not_planned blocks_quick=1770 points=1607 keys_seconds=124.1 two_state_agrees_with_crates=true reused=false
689:E7RESULT name=share_not_planned flow=overwrite_chain-16_units_per_overwrite reason=no_blocks_planned
838:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0200-seed12 reason=no_blocks_planned
839:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0200-seed13 reason=no_blocks_planned
848:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0200-seed18 reason=no_blocks_planned
863:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed06 reason=no_blocks_planned
874:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed12 reason=no_blocks_planned
875:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed13 reason=no_blocks_planned
876:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed14 reason=no_blocks_planned
883:E7RESULT name=share_not_planned flow=random_history-broad-FourGibibytes-steps0400-seed18 reason=no_blocks_planned
```

### Invalidation, failure and stop conditions, individually (segment one)

| Condition | Segment reading | Conclusion |
|---|---|---|
| X1 positive control fails | All 174 `name=control` rows `passed=true` (`grep -c 'passed=false'` = 0); each `*_done` has `controls_failed=0`. Three full-run-plan controls on overwrite u = 16 were not run (amendment 10 item 2); dump round-trip still ran. | Not triggered |
| X2 random-history determinism | 12 `name=determinism` rows (seeds 0 and 1 per six levels, recorded twice), all `equal=true` (lines 175, 176, 266, 267, 357, 358, 448, 449, 539, 540, 630, 631). | Not triggered |
| X3, X4, X5 | Belong to segments two, three, four. | Not judged here |
| X6 unavailable is not zero | Nine unplannable streams say `not_planned`, not zero; `library` says `state=not_run`. | Handled as specified |
| X7 memory wrapper exits 245, 250–254 | All 17 fresh `name=segment` jobs have `state=done runs=1` (no restart); none left output due to a wrapper exit code. | Not triggered |
| Stop: constant comparison | Three `result=pass` lines 31–33. | Pass |
| T2 copied segments verbatim | Five `result=pass` lines 34–38 (four segments plus `segments=4_expected=4`). | Pass |
| T3 re-recorded values equal test-pinned constants | Eight `result=pass` lines 39, 40, 44, 45, 49, 50, 54, 55; ten constant-pinned streams `match=true`, `pinned_smoke_done` has `mismatches=0` (line 29). | Pass |
| T4 two full-state counting paths agree | Lines 41, 46, 51, 56 `result=pass`; `keys_done` for all 127 `name=flow` rows has `count_disagreements=0`. | Pass |
| T5 | Belongs to segment three. | Not judged here |
| Stop: no conversion from random history to `CrashFlow` in crates | Line 3 `result=pass hits=0 files_read=18`. | Pass |
| T7 segmentation asserts or panics | All 120 random histories `ending=Completed`, all six `dump_done` have `failed=0`, `keys_done` has `failed=0`. | Not triggered |
| F1 A cannot separate tiers | Trigger requires both random and overwrite families' state-count overlap to be overlap and both `above` fractions < 0.5. Combined random is partial (line 929), overwrite is separate with `above_fraction=1.000000` (line 927). | Not triggered |
| F2 B's name is misleading | Trigger if random shared keys are renamed: combined scope is independent, median 0.000024 (line 928). | Not triggered |
| F3 overwrite chain is merely renamed | Trigger if overwrite shared keys are renamed: u = 4 independent (line 932, 0.000060), u = 1 independent (line 930, 0.002761), u = 16 unavailable. | Not triggered (u = 16 has no reading) |
| F4, F5, F7 | Belong to segments two, three, four. | Not judged here |
| F6 this provision cell may be wrong | First observation: B1 full-run state count 1358954634 differs from 12230590578 (line 68). Second observation (fraction of ≥ 1000-write random streams whose two-state closed form does not exceed S upper bound ≥ 0.5): registration does not specify scope. Combined `at_most_s_max_fraction=0.291667` (line 929) does not trigger. Per group, 50 steps 0.583333 (937), 100 steps 0.500000 (939), 240 slots 1.000000 (945), maintenance weighting 0.526316 (947) are ≥ 0.5; 200 steps 0.200000 and 400 steps 0.000000 are below. | First triggers as expected; second's two readings differ, both shown without choosing. |

### Unit tests and mutations

- Tests: the main agent's release run yielded 82 passed, 0 failed (log outside repository, not committed). `grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` gives 61; the same count on `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts_write_dump.rs` gives 21. The dump binary imports the main apparatus as a module via `#[path]`; total 82. Acceptance of tests for overwrite u = 4/16, block-count bound and `preflight` is recorded in amendments 6, 8 and 10.
- Mutations: 26 rows appended to `crates/mutations.tsv` (`grep -c '^[E]184 ' crates/mutations.tsv` = 26), named V1–V7 and V9–V27; V8 deleted in amendment 1. The three counts (caught / invalid / did not turn red) await the commit-time checker-tier-crates-mutation-replay gate; this page has no mutation results.

### Fork table (question-sheet “workload scale and overlap,” corresponding to fork-sheet G1)

| Fork | Status | What is still missing | Can the remaining quantities reverse the verdict? | Command that computes it |
|---|---|---|---|---|
| G1 How to distinguish regular-tier workloads from smoke | Decidable: registration section 11 G1 requires verdict words for scale, state-count overlap and shared keys in both families' main and 8.2 samples; “unstable” counts. This segment meets it; shared keys for overwrite u = 16 are `not_planned` (unavailable). | Nothing required; the library condition did not hold, so it was not run; point-level reuse-key upper bounds were not calculated for nine unplannable streams. | Yes: S includes only 14 pinned smoke streams. Adding the 15 unpinned streams (including `position-addressed-trees`, two registered segments of 322 and 290 writes) would raise S's upper bound and move the 200/400-step and overwrite state-count verdicts from partial/separate toward overlap (inferred); the 100-step main sample is already at threshold. The nine unavailable shared-key streams have arithmetic bounds (max 0.000494) that cannot flip independent. | `grep -n 'name=overlap' <产物>`, `grep -n 'name=share_group' <产物>`, `grep -n 'blocks_full=not_planned' <产物>`, `grep -n 'name=share_not_planned' <产物>` |
| G2 | Not covered here (segments two and three) | J3 invalidation surface, J2 cold throughput and library increment | — | — |
| G3 | Not covered here (segment four, after G2) | Fixed overhead of two placements | — | — |
| G4 | Not covered by registration | — | — | — |

Registered flip observations for the three G1 fork candidates and this segment's readings (readings only, not a judgment of which candidate stands; that is inference):

| Candidate | Registered flip observation | Segment reading |
|---|---|---|
| A: register each workload's full-run state count and longest-segment writes, express tier-table scale as ranges | State-count overlap: if both families overlap and both `above` < 0.5, A cannot separate tiers by count (F1); if a family's `above` ≥ 0.5, A can describe regular as the part above smoke's upper bound. | F1 does not trigger. `above`: overwrite all three levels 1.000000; random 50 steps 0.416667, 100 steps 0.500000, 200 steps 0.800000, 400 steps 1.000000, 240 slots 0.000000, maintenance 0.473684 (lines 927, 931, 933, 935, 937, 939, 941, 943, 945, 947); all relative to S's 14 pinned smoke streams. |
| B: split by source; smoke consists of hand-written scripts and registered crash enumeration cases, regular of fully expanded fixed-seed random histories | Shared keys for random: renamed makes B misleading; independent means B adds genuinely new blocks. | All six random groups independent, max shared-block fraction 0.001985. Beyond the 112 plannable streams, eight are unavailable (largest arithmetic bound 0.000494, inferred). |
| C: do not split; merge regular into smoke | Shared keys for overwrite: renamed means overwrite is just renamed smoke; independent means merging is not merely renaming and smoke must judge the additional blocks. | u = 1, 4 independent (0.002761, 0.000060); u = 16 unavailable (arithmetic bound 1.04e-9, inferred). |

### What this cannot answer

- Interval S contains only the 14 pinned smoke streams (ten constant-pinned plus four re-recorded), upper bound 2438004868. Fifteen more registered smoke streams are unpinned (lines 14–28), including `position-addressed-trees`, with two registered segments of 322 and 290 writes in `.claude/gate.d/crash-point-replay-segments.tsv` lines 26–27. Its two-state closed form is at least 2^322 − 1 (inferred: a 322-write two-state segment has closed form 2^322 − 1), far above all random and overwrite streams. Thus `above` is relative to S, not the true smoke-tier maximum. The registration does not specify whether S should include this stream or whether to add a reading that includes streams registered by segment; this page does not decide.
- Block sharing between candidates and the other 25 registered smoke streams was not measured: only four were re-recorded, and `library` (same-key blocks already in the long-term library) did not meet its run condition.
- Block keys include the version table. In 78 of 120 random histories the final version has no file (`versions_with_a_file=0 versions_without_a_file=24`); a different version-table selection changes sharing. The reused-key fraction excludes version table and plan hash and is an upper bound, maximum 0.053737. Actual deployment of the regular tier has another version-table selection in registration scope. Count command and output:

```bash
  grep 'name=dump ' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep -o 'versions_with_a_file=[0-9]* versions_without_a_file=[0-9]*' | sort | uniq -c | sort -rn | head -3
  ```

```text
       78 versions_with_a_file=0 versions_without_a_file=24
       28 versions_with_a_file=24 versions_without_a_file=0
        3 versions_with_a_file=12 versions_without_a_file=12
  ```

- Random histories have only 20 seeds per level and overwrite chains one stream per level, with no distribution: family median and maximum for an overwrite level are that one stream. Geometry outside BROAD weighting, two 4 GiB disks, 240-slot width, maintenance weighting and 50–400 steps was not measured.
- The 100-step main sample's state-count overlap sits exactly on the threshold (10 / 20); changing one stream makes it partial. The overlap verdict is the registration threshold's literal result, not a robust conclusion.
- Overwrite u = 4, 16 uses sequential writes (u one-unit transactions per overwrite), not the single-transaction multi-unit overwrite in the original registration (absent from crates). No full-run plan was made for u = 16 and three full-run-plan controls were not run.
- Shared keys are unavailable for nine unplannable streams. An arithmetic bound (inferred, not measured): shared blocks cannot exceed 46963, the sum of the four re-recorded smoke streams, and each stream has at least full-run states ÷ 65536 blocks:

```bash
grep 'name=flow ' research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out | grep 'blocks_full=not_planned' | awk '{for(i=1;i<=NF;i++){if($i~/^label=/)l=substr($i,7); if($i~/^full=/)f=substr($i,6)} printf "%s full=%s blocks_at_least=%.0f share_at_most=%.3g\n", l, f, f/65536, 46963/(f/65536)}'
```

```text
overwrite_chain-u16 full=2954361402816462969 blocks_at_least=45079977459968 share_at_most=1.04e-09
random_history-broad-FourGibibytes-steps0200-seed12 full=9921447726162 blocks_at_least=151389278 share_at_most=0.00031
random_history-broad-FourGibibytes-steps0200-seed13 full=11663880818977 blocks_at_least=177976697 share_at_most=0.000264
random_history-broad-FourGibibytes-steps0200-seed18 full=6233485413873 blocks_at_least=95115439 share_at_most=0.000494
random_history-broad-FourGibibytes-steps0400-seed06 full=18690168524082 blocks_at_least=285189339 share_at_most=0.000165
random_history-broad-FourGibibytes-steps0400-seed12 full=10165605368686 blocks_at_least=155114828 share_at_most=0.000303
random_history-broad-FourGibibytes-steps0400-seed13 full=18429129377335 blocks_at_least=281206198 share_at_most=0.000167
random_history-broad-FourGibibytes-steps0400-seed14 full=731961382353473 blocks_at_least=11168844335 share_at_most=4.2e-06
random_history-broad-FourGibibytes-steps0400-seed18 full=15453626179150 blocks_at_least=235803622 share_at_most=0.000199
```

- The discriminative power self-test in registration section 8's final sentence (move threshold between two sample values and require the verdict to flip): no test taking the two sample values as inputs was found. Record “not verified.”
- No multi-machine run: segment one ran on a single machine, with no shard on the second host. Aggregate disk use of block-key files increases with step count (20 100-step streams total 15G), one reason for single-machine execution.
- No crash state was judged. This segment counts states, blocks and keys only and says nothing about whether the regular tier turns red.
- Wall-clock quantities (G2 cold throughput and library increment) were not measured here. `keys_seconds` in output is per-stream plan and block-key calculation time, not cold throughput.

### Paths and conclusions register

The conclusion is of the form “state counts in the apparatus agree with a separate crates path, constants pinned in test files, and hand-calculated expressions”: calculate the same quantity with multiple paths that do not share code, then compare.

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Local two-state closed-form recalculation | Apparatus computes 1 + Σ(2^segment writes − 1), returns `None` if a segment has ≥ 64 writes or sum exceeds u64; reports `two_state_agrees_with_crates` per stream. | `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs:234 two_state_closed_form` | D13 settled item 4's segment model and closed form; crates' segmentation supplies the same segments to both paths. |
| 2 | Crates two-state closed form | Call harness `closed_form_state_count`; panics if any segment has ≥ 64 writes. | `crates/singlefs-harness/src/memory_pool.rs:659 closed_form_state_count` | Same segmented input and same formula. |
| 3 | Full-run states: plan final node ordinal end | `plan_crash_points` creates a plan and takes the last point's `ordinals.end`. | `crates/singlefs-checker-tier/src/crash_amplification.rs:505 plan_crash_points` | `crash_points_of_the_steps` node segmentation, `full_expansion` (third state for in-place overwrites), same write table and segments. |
| 4 | Full-run states: count by segment range | `crash_state_count_with_torn_in_place_overwrites` with `full_expansion`. | `crates/singlefs-checker-tier/src/crash.rs:2418 crash_state_count_with_torn_in_place_overwrites` | Same expansion function and in-place-overwrite third-state rule; paths 3 and 4 can share errors in that expansion rule. |
| 5 | Test-pinned constants | Compare two-state closed form, full, and A-two for four re-recorded smoke streams with test-pinned values (stop T3). | `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:774 FULL_STATES_WITH_TWO_STATES_PER_WRITE` | Same tested stream; constants were pinned by test authors and are not independent of implementation at the time. |
| 6 | Hand calculation | Hand-written one-line Python computes A1's 16842785 from segment shape `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]` and the new-pool file-creation stream's two-state closed form 16777243 from its first 12 segments (with one round of number acquisition writes the shape lacks that leading 2 and A1 gives 16842782; item 6 of "What was done"). | No implementation: the `python3 -c` line in registration section 13.4 item 3 with the segment shape after the two-round number acquisition write swapped in, no script stored in repository. | Reads only D13 settled item 4's formula and segment shape, not crates. |

Cross-path comparison assertions are pinned in source and output: path 1 versus 2 is `two_state_agrees_with_crates=true` on all 127 `name=flow` rows; 3 versus 4 is `T4_two_counting_routes_*` and `count_disagreements=0` in each `keys_done`; 5 is `T3_recorded_values_*`; 6 versus 1 uses two anchors, snapshot stream two-state closed form 16842785 and new-pool file-creation stream 16777243 (both anchors are `match=true` on lines 68 and 69 of the second rerun's artifact, recomputation equal to recording; in the 2026-10-19 segment-one artifact they are 16842782 and 16777240, from the version with one round of number acquisition writes). Conclusions use sampled points, not the entire domain: 127 streams, four smoke streams and one hand-calculated snapshot stream, because the registration requires these families only. Paths 1–4 share the same segmentation and expansion rules, so errors in those inputs can affect them all undetected.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D13 (Verification Route), settled item 9 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-24 unaffected: rerunning all of segment one on the new segment shapes leaves the verdict words identical cell by cell (narrow reading 40 cells and wide reading 62 cells, 0 words changed; see the section “Second rerun”); B1 now records 1358954642 (line 77 of the rerun artifact), still different from the scope's 12230590578, and the first part of F6 still triggers; line 269 of the item's evidence paragraph says “No experiment … E184 (Cost Counts of Regular Layer and Large Workload Layer) segment one measured the scale and state-count interval overlap of candidate workloads, supporting material only”, so the relationship stays supporting material. “Thousands of write requests” in the regular-tier table: candidate streams with at least 1000 writes are still all above the pinned smoke streams (maximum 828 writes), and on the state axis it still runs from overlap to separate with step count (inside fractions 0.923077, 0.550000, 0.350000, 0.100000, the 400-step level exactly on the threshold endpoint). The main agent must decide whether to elevate these to support or refutation and whether to change the scope sentence; the decision text was not changed this time. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds) only the pinned values were fixed; `selfcheck` now records the four smoke streams equal item by item to the test pins and passes all stop checks; the whole segment-one artifact was not rerun, and the verdict words are inferred not to flip. B1 now records 1358954642, which still does not match the scope's 12230590578. 2026-10-19 unaffected: the item's evidence paragraph does not cite E184, so this page is supporting material only. “Thousands of write requests” in the regular-tier table separates from the 14 pinned smoke streams on the write axis (maximum 824), while on the state axis it moves from overlap to separate with step count. The scope's 12230590578 differs from today's multi-record stream 1358954634 (F6's first observation). The main agent must decide whether to elevate these to support or refutation and whether to change the scope sentence; the decision text was not changed this time. |
| D13 (Verification Route), settled item 17 | Supporting material | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-24 unaffected: all 12 `name_not_in_key` controls in the rerun artifact are `passed=true`, and the maximum shared-block fraction between random histories and the four re-recorded smoke streams is 0.002261 (R 50 steps, line 945 of the rerun artifact); the item's evidence paragraph does not cite E184 (Cost Counts of Regular Layer and Large Workload Layer), so this page is supporting material only; the main agent must decide whether to elevate these two points to support; the decision text was not changed this time. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds) only the pinned values were fixed; `selfcheck` now records the four smoke streams equal item by item to the test pins and passes all stop checks; the whole segment-one artifact was not rerun, and the verdict words are inferred not to flip. 2026-10-19 unaffected: the item's evidence paragraph does not cite E184, so this page is supporting material only. “Name is not in the key” holds along this apparatus path (all 12 `name_not_in_key` controls `passed=true`); the maximum shared-block fraction between random histories and the four re-recorded smoke streams is 0.001985. The main agent must decide whether to elevate these to support; the decision text was not changed. |
| D13 (Verification Route), settled item 4 | No effect | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-24 unaffected: in the rerun artifact the two-state closed forms and full-run counts of the four re-recorded smoke streams equal the test-pinned constants item by item (the eight T3 cells are `result=pass`, `mismatches=0` in `pinned_smoke_done`); no new conclusion. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds) only the pinned values were fixed; `selfcheck` now records the four smoke streams equal item by item to the test pins and passes all stop checks; the whole segment-one artifact was not rerun, and the verdict words are inferred not to flip. 2026-10-19 unaffected: apparatus segmentation follows this item; all two-state closed forms and full-run counts of four re-recorded smoke streams equal test-pinned constants (eight T3 passes). No new conclusion. |
| D13 (Verification Route), settled item 16 | No effect | 2026-10-26 unaffected: only outdated current-state sentences and pointers were changed, conclusions and artifacts unchanged; previously, 2026-10-24 unaffected: all 13 `family_summary` rows of the rerun artifact have `flows_beyond_u64=0`, no stream's two-state closed form exceeds u64, and registration by segment and sharding were not encountered. 2026-10-23 unaffected: after the upstream write-order change (number acquisition writes in two rounds) only the pinned values were fixed; `selfcheck` now records the four smoke streams equal item by item to the test pins and passes all stop checks; the whole segment-one artifact was not rerun, and the verdict words are inferred not to flip. 2026-10-19 unaffected: all 13 `family_summary` rows have `flows_beyond_u64=0`; no stream's two-state closed form exceeds u64, so registration by segment and sharding were not encountered in this segment. |

## Historical Versions

Historical entries for E184 (Cost Counts of Regular Layer and Large Workload Layer) are collected in [experiments-history.md](../experiments-history.md).
