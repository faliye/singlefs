## E181 Landing Determination and Mount Lower Bound — Partially run (first-segment apparatus, 18 self-tests, mutation rows and long-run script written; first-segment artifact not yet run, second segment not written)

[中文](181-落得下那一判与挂载下界.md) · **English** · [日本語](181-配置可能性の判定とマウント下限.ja.md)

<!-- doc-lint:not-numbers F9 F10 F11 F14 P1 P2 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 A1 A3 A4 B1 B2 B3 G10 L1 L2 L3 Y1 Y2 Y3 Y4 Y5 Y6 Y7 -->

Pre-run registration: `research/prompts/e181-preregistration.md` (two segments: the first addresses F9, F11 and F10 on the fixed point and placement-fit check; the second addresses F14 on truncated disks; this page covers only the first). It answers four rows of fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md`: F9 (whether the fixed-point upper bound for the placement-fit check should include the additional central mapping tree deletions and insertions from this publish's ordinary allocations: P1 includes them; P2 does not and reports `PlacementRefused` if the fixed point cannot obtain space); F11 (whether the preliminary refusal check should calculate and compare the lowest placement on each disk, measuring scan wall time as the unit area grows); F10 (whether the placement-fit check should count two-slot commit-intrinsic blocks in aligned slot pairs, first finding a reachable disk state G10 with enough unblocked slots but too few aligned free pairs); and F14 (second segment). The background is owed checks C573 (fixed point upper bound misses additional delete-inserts during publish with regular allocation), C574 (can-land judge does not verify two-slot alignment and same slot across disks), and C582 (no clause on which lower bound of mount disk capacity should take). The clauses under test are D28 (Mount-Time Commitments) settled items 1 and 4, and D3 (Space Allocation) settled items 5, 8, 10 and 12.

The checked-in apparatus is `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs`. It drives public `singlefs-core` entry points read-only and changes no other file in `crates/`. Each quantity that would read a `crates/` constant instead uses a local constant copied from its kb registration; `check_local_constants_against_the_crates` compares them individually.

**Current status**:

- The apparatus, 18 self-tests (`grep -c '#\[test\]'` on its source = 18; the same assertions are in a `claims` module, called once by `#[test]` and once by the artifact's `selfcheck` segment), 17 mutation rows (names start with experiment E181 (Landing Determination and Mount Lower Bound), to be appended to `crates/mutations.tsv` after the apparatus exists in the main workspace), long-run script `scripts/experiment-apparatus/e181-landing-check-long-run.sh`, and E181 (Landing Determination and Mount Lower Bound) replay registration in `scripts/experiment-apparatus/replay.sh` (driver `@driver_e181`, artifact `e181-landing-check-fixed-point-and-mount-lower-bound-checks.out`, exact comparison) are written. This artifact has not been run and no raw output was retained (neither in `research/results/` nor in repository history).
- The unit tests could not be run in this segment: tests of the checker tier package are classified as heavy tier at package level and are left to the main agent. The apparatus's `selfcheck` segment passed all 18 checks individually on a binary built in the scratch directory; those are apparatus trial readings, not an artifact. The three mutation counts await the checker-tier-crates-mutation-replay gate at commit time.
- The first-segment artifact `research/results/e181-landing-check-fixed-point-and-mount-lower-bound-landing.out` has not been run and no original exists (it was never run, so no commit can supply it). F9's edge histories plus F10's seeded walks are estimated at one or two hours or more per run; F10's 10 steps were not measured on G_default because that `measure` slice must first fill the data side and cannot finish in the foreground. The long-run script reads the extrapolation after the `measure` slice and stops if it exceeds two hours (registration section 5.6).
- Registration revisions are in section 12 of `research/prompts/e181-preregistration.md` (12.1–12.10: add G_small_unit_area geometry, add a step-4 arm to HF9-three-level, take a superset of edge histories, add real states to F11, verdict-row fields, 17 mutation rows, and four substitutions differing from the registration: filling method, marked states, G_unequal state, and unit-test procedure).

### What was done

1. The apparatus follows registration section 5: local constants and comparison (`constants`), section 7 anchors A1–A4 and B1 (`anchors`), F9's three histories (`f9`: five product-capacity levels, three-level, edge), F11 scan wall time and unequal per-disk placements (`f11`), F10 positive controls and seeded walks (`f10-controls`, `f10-narrow`, `f10-unequal`, `f10-small`), and ten F10 steps measured before the run (`f10-measure`).
2. The pool is an in-memory copy-on-write disk (copies share sealed layers and each has its own top layer), so copying a full pool does not copy several GiB of sectors. Each of 64 seeds walks 300 steps; thread counts follow each host's default heavy-work count, results are merged in seed order, and thread count does not change the artifact.
3. The long-run script splits the apparatus into eight slices (`checks`, `f11`, `f9`, `measure`, `controls`, `narrow`, `unequal`, `small`), one process and output per slice. It joins completed slices in a fixed order into a multi-segment artifact and saves the first slice separately for replay registration. Restarting the same command fills in only unfinished slices.

### Rerun

```bash
# 仓根；入库装置，经 scripts/experiment-apparatus/replay.sh 的一行登记（driver_e181 只跑确定性的那一片：constants anchors selfcheck f10-controls-small，约半分钟）
bash scripts/experiment-apparatus/replay.sh E181

# 第一段全部（编装置、八片、拼产物；产物另存到 research/results/ 下还没被占的名字；F10 外推超过 2 小时在 measure 片之后停、退 4）
bash scripts/experiment-apparatus/e181-landing-check-long-run.sh

# 装置的段（经内存包装：bash scripts/hosts/run-with-memory-cap.sh <上限> target/release/e181_landing_check_fixed_point_and_mount_lower_bound <段…>）
#   constants anchors selfcheck           常量回比、A1 至 A4 与 B1、18 条自证
#   f9 / f11 / f10-measure                F9 三段历史、F11 挂钟、F10 的 10 步外推
#   f10-controls / f10-controls-small     F10 的阳性对照（G窄、G小单元区）
#   f10-narrow / f10-unequal / f10-small  F10 的走种子（环境变量 E181_THREADS 给线程数，E181_F10_SEEDS、E181_F10_STEPS 缩小规模，缩过的写进产物）
```

F11's `median_nanoseconds` and `maximum_nanoseconds`, and seconds in `f10_measure_*`, are timings and vary with machine load. The artifact's final line records `loadavg1` and `cpus` at the time of the run. Segment durations and round counts go only to standard error, not the artifact.

### What it cannot answer

- The first segment answers only F9, F11 and F10. F14 (the reason mounting a truncated disk fails) belongs to the second segment, whose code is not yet in the apparatus; F14 remains open in the fork sheet.
- F9 Q4 measures only the number of writes for which the placement-fit check passes but the fixed point refuses; it does not measure the outcome seen by the user after the session drains and raises F. F9 edge histories call `publish_sequential_write` directly, bypassing the session.
- F10 cannot fill everything: a publish can contain at most 65536 journal records, so one write cannot fill the out-of-segment slot pairs in G_narrow, and G10's starting state is less restrictive than the registration requires. The G_small_unit_area arm reduces the gap to about 20%, but is a geometry outside the registration (revisions 12.1 and 12.7). The session layer has neither overwrite at an offset nor a test-only switch to move commit-intrinsic blocks away from bump. Thus whether G10 is unreachable cannot be answered; only whether it occurs in these reachable histories can be answered (registration section 10, Y4).
- In F11, `half_marked` and `ninety_percent_marked` are marked through the allocator's public entry point, not produced by real writes. `half_real` and `ninety_percent_real` exist only for G_narrow, and the 90% case may be rejected by the journal-record limit.
- Sample points: G_default, G_narrow, G_unequal and G_wide (scan only), plus G_small_unit_area; sparse in-memory disks; two disks.
- This page does not yet have passing unit tests, the three mutation counts, or readings from the first-segment artifact.

### Path and conclusion registration

The conclusions concern two quantities: “fixed-point slots actually written” and “ckpt_cost.” Each quantity is computed by two paths sharing no code and compared.

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Actual written fixed point: role path | Sum the spans of roles actually written by this publish other than ordinary allocation | `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs:715 fixed_point_account` | Role classification `space_budget_of_role` (core layer), this publish's `rewritten` |
| 2 | Actual written fixed point: record path | In this version's allocation records, sum spans per disk for records allocated in this txg and not released, then subtract slots for ordinary allocation and instance-switch roles | `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs:715 fixed_point_account` | Same publish output; subtraction reads the same `space_budget_of_role`, so a role-classification error affects both paths |
| 3 | ckpt_cost: local recomputation | Add allocation record tree by K1, central mapping tree by per-level path formula, accounting tree, lineage, retirement, and tree table 1 | `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs:615 local_checkpoint_cost` | The accounting tree, lineage and retirement use the core layer's public functions of the same names; unit-area start and slot count, and tree height, are read from the allocator and root-node header |
| 4 | ckpt_cost: core layer | `checkpoint_cost_of_the_version_to_build_on_with_node_capacities` | `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs:691 core_checkpoint_cost` | Same three public functions as above |
| 5 | Hand calculation of K1 | Count positions covered by unit area [50176, 262144) at levels 0, 1 and 2; K1 at root levels 1, 2 and 3 is 525, 529 and 531 | `crates/singlefs-checker-tier/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound.rs:3592 allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three` | S_L = 812 × 169^L from D28 (Mount-Time Commitments) settled item 4 |

Cross-path comparisons: paths 3 and 4 must match before each write or the run stops (“B3 halt” in `write_units_from_bytes_and_account`); unit test `the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured` pins this across three geometries and two capacity levels. Paths 1 and 2 are pinned on the first file's version (unit test `the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths` pins 6 and 8); thereafter each successful write reports `true_fixed_point` and `true_fixed_point_by_records` side by side in the artifact, but no assertion links each successive comparison, which remains owed. Conclusion values are enumerated points: five unit counts, three geometries and two tree capacities, because the registration requests only those levels. Paths 1 and 2 share role classification and fail together if it is wrong.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D28 (Mount-Time Commitments) settled item 4 | Background | 2026-10-21 unaffected: only numbering and source-location line numbers were changed, and the text states both artifacts remain unrun with no originals retained; decision text unchanged. 2026-10-19 unaffected: the item's evidence does not cite E181 (Landing Determination and Mount Lower Bound), and the first artifact has not run; background only. After the artifact, inspect Q1–Q4 and refer the wording of C573 (fixed point upper bound misses additional delete-inserts during publish with regular allocation) to the main agent; decision text unchanged this time |
| D28 (Mount-Time Commitments) settled item 1 | Background | 2026-10-21 unaffected: only numbering and source-location line numbers changed, with both artifacts marked unrun and originals not retained; decision text unchanged. 2026-10-19 unaffected: the item's evidence does not cite E181 (Landing Determination and Mount Lower Bound); F9 and F10 leave the order of the three gates and the position of the placement-fit check for the user to decide. No artifact has run and the decision text was not changed |
| D3 (Space Allocation) settled item 8 | Background | 2026-10-21 unaffected: only numbering and source-location line numbers changed, with both artifacts marked unrun and originals not retained; decision text unchanged. 2026-10-19 unaffected: the item's evidence does not cite E181 (Landing Determination and Mount Lower Bound); Q7–Q9 measure same-slot placement across disks (F11) and the scan cost of placement. No artifact has run and the decision text was not changed |
| D3 (Space Allocation) settled item 10 | Background | 2026-10-21 unaffected: only numbering and source-location line numbers changed, with both artifacts marked unrun and originals not retained; decision text unchanged. 2026-10-19 unaffected: the item's evidence does not cite E181 (Landing Determination and Mount Lower Bound); F10's G10 needs the measurement basis for two-slot alignment of commit-intrinsic blocks and open segments. No artifact has run and the decision text was not changed |
| D2 (RAID Stripe Policy) settled item 19 | Background | 2026-10-21 unaffected: only numbering and source-location line numbers changed, with both artifacts marked unrun and originals not retained; decision text unchanged. 2026-10-19 unaffected: the item's evidence does not cite E181 (Landing Determination and Mount Lower Bound); alternative mount-side lower bounds are measured in the second segment. This page has no artifact and the decision text was not changed |

## Historical Versions

History entries for E181 (Landing Determination and Mount Lower Bound) are collected in [experiments-history.md](../experiments-history.md).
