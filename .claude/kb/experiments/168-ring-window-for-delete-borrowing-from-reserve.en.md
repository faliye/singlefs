## E168 Ring Window for Borrowing Reserve Capacity on Deletion — Partially Run (segment one complete)

[中文](168-删除借保留量的环窗口.md) · **English** · [日本語](168-削除がリザーブを借りる際のリングウィンドウ.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 P0 P1 P2 P3 P4 P5 S1 S2 S3 S4 S5 S6 S7 V1 V2 V3 V4 V5 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 B1 B2 B3 B4 B5 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 U1 U2 U3 U4 U5 U6 U7 U8 U9 U10 U11 U12 U13 U14 U15 U16 R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 R11 R12 R13 R14 R15 R16 R17 R19 R20 K1 -->

```text
Local arm names and symbols on this page are lowercase (a1, a2, a3, a4 …). The pre-registration, apparatus, artifacts and commands use the corresponding uppercase forms, which look like global registration identifiers; lowercase in the KB body distinguishes them. Match to uppercase when reading artifacts.
```

Pre-registration `e168-preregistration.md` (section twelve revisions 1–17); question list `m3-ring-window-questions.md`: Q1 (how the peak of borrowing from reserve changes with disk size and ring length, and whether the reserve covers it), Q2 (whether “(ring length + 1) × W upper bound” bounds the peak), Q3 (whether to account separately for the ring-window amount outside the reserve), Q4 (whether resuming work during mount can make mount fail). Dispatch batches `records/2026-10-01-删除与并发事务.md#3` (page creation), `#10` (segment one checked in).

**Current status**: segment one (registration 5.8: N-A and N-B at P0, P1, P2, P4 × S = 4 / 16 × K = 144 / 1, L-seq and F-end; eight positive-control cells) was completed on 2026-10-03 by the main agent with `SINGLEFS_HEAVY_TESTS=user-request`, and artifacts were checked in. Under the literal wording of registration 5.8, Q1 is sufficiently answered “not covered” if any cell is refused: 65 delete cells were refused, smallest counterexample P1, S = 16, K = 1, N = 8 (149 unit-area slots). But registration line 400 designates K = 144 as the main decision column and K = 1 as sensitivity; at K = 144 only N-A at P1, S = 16 was refused (7 cells, all on the delete path rewriting all lower extent leaves), while N-B had no refusals. The main agent judged K = 1 and N-A on the full-rewrite path not covered, recording one loss; the K = 144 main column has not been measured on the tail-leaf-only path required by D8 (Core Index Structure) settled item 16 clause 2, so Q1 remains open. Q2 is not sufficiently answered: zero cells exceed (3S + 1) × W_max; W_max grows with file size to the top of the ladder in 14 groups (F4 triggered); L-scatter and P5, the remaining cells in 5.8, were not run. Q3 and Q4 were not run (segments two and three). The user decided on 2026-10-03 to register and rerun segment two with real implementation delete batches (tail leaf only). Delete batches are merged into the main repository (core `crates/singlefs-core/src/unlink.rs`), but segment two has not run (current `ls research/results/e168-*` shows only segment-one artifacts from 2026-10-03).

### Apparatus and patch table

| Item | Location | Description |
|---|---|---|
| Checked-in apparatus | `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs` | Drives and observes only `crates/` publish path, acceptance readings (`admission_reading_before_a_publish`, `checkpoint_cost_of_the_version_to_build_on`), in-mount reclamation and session floor F raises; modes `s1-history` and `segment1 [--geometry <geometry>]` |
| Delete-path patch table | `scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv` | Four columns (description, file, old string, new string), 19 rows: R1–R16 wire the second-round delete attacker prototype, adding test-only delete entry, nlink = 0, switch and count for the formula check, accounting, reclamation-off switch and valley hook; R13 additionally adds a test-only entry for K1; R19/R20 add a session delete batch; R17 replaces the apparatus stub |
| Stub | Apparatus `delete_path_hooks` | Main-workspace version emits `E name=stop reason=delete_path_patch_absent` and exits 3 on any call (registration revision 2) |

Each of the 19 patch-table rows matched exactly once in the segment-one copy (`e168-delete-ring-window-borrow-patch-anchors-2026-10-03.log`, all 19 lines `1`); after patching it compiled. On 2026-10-03, recounting against that day's main-workspace `crates/` (substring count only, no compile) still gave exactly one hit each.

Segment-one procedure (registration 5.8): compare every local constant with `crates/`; section-seven B1–B3 anchors; three-step P0 outcomes (S = 4 / 16); N-A and N-B at P1(S = 4 / 16), P2(S = 4 / 16), P4(S = 4 / 16), K = 144 and K = 1, L-seq, F-end. Sweep ladder 8.4 until M3 saturation, file cannot be written, or apparatus limit 32768 data units (revision 6); PC-noreclaim positive controls for four arms × two geometries (P1 × S = 16, P4 × S = 4). For each cell create a pool, check S4 / B4 (also a1 = 529 for P4), fill according to arm, then delete. Each delete publish checks S3 (reserve = 8 + 13c), S5, S6 and zero calls to the formula check. After deletion check S7: before checker, return fill-marked slots to free and publish one more delete reserving zero units, so accounting is rewritten against the allocator after fill removal; one `E name=s7_finished_delete` per cell (revision 9). Check B5 in cells with reclamation disabled.

**Registered quantities not implemented in this version**: a3 (number of valid-root slots in the root ring); 8.1's borrow after deletion plus 3S empty publishes; 8.2's discriminative power self-test (move threshold between two values and rejudge); M4's transition reserve and checkpoint-pool halves (artifact reports reserve only; split the halves from `checkpoint_cost=` using 8 + 13c); M5's lower/upper extent-tree heights and number of allocation-record leaves changed by the batch; segment two (W-arm ladder, L-scatter, F-start, P5); segment three (Q4's M1 resume patch, fill allocation-record publish, and crash/remount at three points).

### Artifacts

All are from the 2026-10-03 segment-one r8 run. Apparatus lines begin `E name=`; checked-in `.out` files use `e168_wrap` in `scripts/experiment-apparatus/replay.sh` to replace the prefix with `E7RESULT name=` and add `emitted=` to the final line. Reversing this gives byte-identical original output (checked separately for four files).

| File | Contents | Lines |
|---|---|---|
| `e168-delete-ring-window-borrow-segment1-2026-10-03.out` (original not retained: 115 MB, ignored by .gitignore, never committed) | All segment-one cells (`segment1`), including eight positive controls; one `name=publish` per delete publish, 175835 such lines. Not checked in: 115 MB, ignored by .gitignore line 32, only on the machine used to run it; see “raw output not checked in” under segment-one measurements | 177054 |
| `research/results/e168-delete-ring-window-borrow-positive-controls-2026-10-03.out` | Positive-control smoke test (`positive-controls`, eight cells at two geometries), run separately before segment1; the 710 lines in these eight cells match identically named segment1 cells byte for byte | 1336 |
| `research/results/e168-delete-ring-window-borrow-s1-history-2026-10-03.out` | Stop condition S1: main workspace, without patch table, runs `s1-history` | 4 |
| `e168-delete-ring-window-borrow-s1-history-patched-2026-10-03.out` | Same history in patched copy with all switches off; byte-identical to `e168-delete-ring-window-borrow-s1-history-2026-10-03.out` | 4 |
| `e168-delete-ring-window-borrow-unit-tests-2026-10-03.log` | Main-workspace unit tests: 27, 26 passed and one ignored because it requires a patched copy | 56 |
| `e168-delete-ring-window-borrow-unit-tests-patched-2026-10-03.log` | Copy's unit tests, including ignored test: 27 passed | 35 |
| `e168-delete-ring-window-borrow-mutants-2026-10-03.log` | 14 E168 mutations in `crates/mutations.tsv` demonstrated failing individually, 14 `✅` lines | 14 |
| `e168-delete-ring-window-borrow-patch-anchors-2026-10-03.log` | Per-row hit counts for 19 patch-table rows in copy (S2) | 19 |
| `e168-delete-ring-window-borrow-segment1-tree-2026-10-03.sha256` | Per-file SHA-256 for `crates/`, `Cargo.toml`, `Cargo.lock`, `.cargo` used to build segment-one binary (excluding patch table; apparatus row `595ad82c…`) | 242 |

### Segment-one measurements

Verdict rows (line 177053 of segment1, line 1335 of smoke-test artifact):

```
原始输出未留存：research/results/e168-delete-ring-window-borrow-segment1-2026-10-03.out 第 177053 行（name=verdict；这份产物 115 MB，被 .gitignore 第 32 行忽略，只在跑它的那台机器上）
E7RESULT name=verdict mode=positive-controls w_max_groups=2 w_max_of_each_geometry={"P1-S16": 196, "P4-S4": 15} positive_control_cells=8 positive_control_cells_without_refusal=0 positive_control_session_cells_refused_other_than_no_space_after_raising_the_floor=0 positive_control_cells_not_holding=0
```

One row per group (fields from group rows, last column is its line in segment1; “deleted” excludes cells where the first file could not be written: on P2, N = 144 at K = 144 and N = 192 at K = 1):

| Group | Deleted | Refused | M3 | M5 W grows with file size | W_max (slots) | (3S + 1) × W_max | Ladder ceiling | Line |
|---|---|---|---|---|---|---|---|---|
| P1 S=4 K=144 N-A | 13 | 0 | GrowingWithoutSaturation | true | 217 | 2821 | apparatus_file_size_limit_32768 | 524 |
| P1 S=4 K=144 N-B | 13 | 0 | GrowingWithoutSaturation | true | 217 | 2821 | apparatus_file_size_limit_32768 | 1048 |
| P1 S=4 K=1 N-A | 18 | 10 | GrowingWithoutSaturation | true | 59 | 767 | none | 12741 |
| P1 S=4 K=1 N-B | 18 | 9 | GrowingWithoutSaturation | true | 59 | 767 | none | 24826 |
| P1 S=16 K=144 N-A | 11 | 7 | FewerThanThreePointsPastTheRing | true | 217 | 10633 | apparatus_file_size_limit_32768 | 25127 |
| P1 S=16 K=144 N-B | 11 | 0 | GrowingWithoutSaturation | true | 217 | 10633 | apparatus_file_size_limit_32768 | 25747 |
| P1 S=16 K=1 N-A | 18 | 15 | FewerThanThreePointsPastTheRing | true | 196 | 9604 | none | 26030 |
| P1 S=16 K=1 N-B | 18 | 10 | GrowingWithoutSaturation | true | 206 | 10094 | none | 73909 |
| P2 S=4 K=144 N-A | 0 | 0 | FewerThanThreePointsPastTheRing | false | 0 | 0 | file_not_written_at_144_step_sequential_write | 73912 |
| P2 S=4 K=144 N-B | 0 | 0 | FewerThanThreePointsPastTheRing | false | 0 | 0 | file_not_written_at_144_step_sequential_write | 73914 |
| P2 S=4 K=1 N-A | 12 | 1 | GrowingWithoutSaturation | false | 13 | 169 | file_not_written_at_192_step_sequential_write | 74130 |
| P2 S=4 K=1 N-B | 12 | 1 | GrowingWithoutSaturation | false | 13 | 169 | file_not_written_at_192_step_sequential_write | 74358 |
| P2 S=16 K=144 N-A | 0 | 0 | FewerThanThreePointsPastTheRing | false | 0 | 0 | file_not_written_at_144_step_sequential_write | 74360 |
| P2 S=16 K=144 N-B | 0 | 0 | FewerThanThreePointsPastTheRing | false | 0 | 0 | file_not_written_at_144_step_sequential_write | 74362 |
| P2 S=16 K=1 N-A | 10 | 6 | FewerThanThreePointsPastTheRing | false | 13 | 637 | file_not_written_at_192_step_sequential_write | 74466 |
| P2 S=16 K=1 N-B | 10 | 6 | FewerThanThreePointsPastTheRing | false | 13 | 637 | file_not_written_at_192_step_sequential_write | 74588 |
| P4 S=4 K=144 N-A | 13 | 0 | GrowingWithoutSaturation | true | 214 | 2782 | apparatus_file_size_limit_32768 | 75098 |
| P4 S=4 K=144 N-B | 13 | 0 | GrowingWithoutSaturation | true | 214 | 2782 | apparatus_file_size_limit_32768 | 75621 |
| P4 S=4 K=1 N-A | 12 | 0 | Saturated | false | 15 | 195 | none | 75934 |
| P4 S=4 K=1 N-B | 12 | 0 | Saturated | false | 15 | 195 | none | 76259 |
| P4 S=16 K=144 N-A | 11 | 0 | GrowingWithoutSaturation | true | 214 | 10486 | apparatus_file_size_limit_32768 | 76867 |
| P4 S=16 K=144 N-B | 11 | 0 | GrowingWithoutSaturation | true | 214 | 10486 | apparatus_file_size_limit_32768 | 77486 |
| P4 S=16 K=1 N-A | 18 | 0 | GrowingWithoutSaturation | true | 214 | 10486 | none | 126897 |
| P4 S=16 K=1 N-B | 18 | 0 | GrowingWithoutSaturation | true | 214 | 10486 | none | 176326 |

The table was generated from the artifact by `grep -n '^E7RESULT name=group ' e168-delete-ring-window-borrow-segment1-2026-10-03.out | awk '{split($1,h,":"); delete v; for(i=2;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]} print v["point"], v["S"], v["K"], v["arm"], v["cells_deleted"], v["cells_refused"], v["m3_saturation"], v["m5_w_grows_with_file_size"], v["w_max"], v["ring_window_bound"], v["ladder_ceiling"], h[1]}'`. The ladder ceiling `none` in each P1 and P4 K = 1 group means: P1 at both S and P4 S = 16 continued multiplying by 2 until reaching revision 6's six-extension limit (P1 S = 4 to N = 6144, S = 16 to N = 24576), not disk exhaustion; both P4 S = 4 groups saturated and stopped without extension at N = 96. P1 finds a minimum usable disk for each N (revisions 10 and 15), so disk and reserve grow with N within a group.

Each reported measurement was counted from the segment1 artifact by a command selecting `grep '^E7RESULT name=publish '` or `name=cell ` and calculating fields with awk; positive-control `-noreclaim` cells are excluded. Main commands (`F` is the segment1 artifact path, `kv` splits a row into fields):

```
kv='{delete v; for(i=1;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]}}'
# 拒在哪一判：臂、拒绝种类、那一步落了盘的抬 F 次数
grep '^E7RESULT name=publish ' $F | grep -v noreclaim | grep -v ' refused=none$' | awk "$kv"'{split(v["cell"],c,"-"); print c[6]"-"c[7], v["refused"], v["floor_raise_publishes"]}' | sort | uniq -c
# M1：成功发布上 borrow_valley 是否恒等于 reserve − valley_free
grep '^E7RESULT name=publish ' $F | awk "$kv"'v["refused"]=="none"{n++; if (v["borrow_valley_device0"]!=v["reserve_device0"]-v["valley_free_device0"] || v["borrow_valley_device1"]!=v["reserve_device1"]-v["valley_free_device1"]) bad++} END{print n, bad+0}'
# W 跟剩下的 extent 下段叶数走
grep '^E7RESULT name=publish cell=P4-S4-K144-Lseq-Fend-N-A-N27648 ' $F | awk "$kv"'{c[v["ordinary_slots"]-int((v["kept_data_units"]+143)/144)]++} END{for(x in c) print x, c[x]}'
# 被拒那一刻的空闲对同一格上一次成功删除发布的 W
grep '^E7RESULT name=publish ' $F | grep -v noreclaim | awk "$kv"'{k=v["cell"]; if(v["refused"]=="none") w[k]=v["new_slots_device0"]; else print (v["free_before_device0"]+0>=w[k]+0)}' | sort | uniq -c
```

- **Which check refused**: all 39 N-A cells were refused at the placement check (`refused=landing`); all 26 N-B cells were refused after raising F (`refused=no_space_after_raising_the_floor`). All 26 sequences of session F raises were refused in rehearsal (`floor_raise_stop=floor_raise_refused_for_space_in_the_rehearsal_at_publish_k_of_n`, k / n = 1 / 2, 1 / 3, 2 / 2, 2 / 3, 3 / 3 with counts 1, 5, 16, 2, 2); none of a sequence was persisted.
- **Smallest counterexample** (segment1 line 25777): after filling, each disk has 86 free slots = reserve 86. Each delete publish newly occupies 10–11 slots; the eighth (t = 7) is refused at placement with 9 free slots per disk. At S = 16 the ring holds 48 roots, and slots written by deletion cannot be reclaimed until roots rotate out; seven delete publishes fit within one revolution.

```
E7RESULT name=cell cell=P1-S16-K1-Lseq-Fend-N-A-N8 first_file=written root_level=1 allocation_record_tree_positions=3 unit_area_slots=149 free_after_fill=[86, 86] floor_raise_publishes=0 publishes=8 completed=false refused=landing refused_at=7 borrow_at_refusal=77 apex=77 apex_at=7 reserve_at_apex=86 apex_minus_reserve_at_apex=-9 apex_minus_reserve_minimum=-9 m1_reserve_covers_apex=true delta_flip=1 reserve_minimum=86 reserve_maximum=86 reserve_at_start=86 reserve_at_last_publish=86 apex_between=77 apex_between_at=7 positive_valley_publishes=8 positive_between_publishes=7 borrow_before_last_publish=77 intent_new_slots=11 largest_batch_new_slots=11 largest_new_slots=11 w_exceeds_landing_publishes=0 largest_w_per_mille_of_landing=1000
```

- **M1 cannot detect “peak exceeds reserve”**: among 175762 successful delete publishes, `borrow_valley` differs from `reserve − valley_free` on either disk zero times. Valley free is a nonnegative slot count, so apex − reserve_at_apex = −(peak publish's valley free) ≤ 0; `q1_cells_apex_over_reserve_k144=0` and `q1_cells_apex_over_reserve_k1=0` follow by definition and carry no information. Q1 “not covered” can only be judged from M2 refusals (revision 1 ③ separates the two judgments).
- **M3's second reversal observation** (unsaturated and peak position > 3S + 2) appears only at K = 1: P1 S = 4 both arms (16th, N = 6144), P1 S = 16 N-B (279th, N = 1536), P4 S = 16 both arms (104th, N = 24576). At K = 144 all groups peak within 3S + 2 (maximum 11th for S = 4, 47th for S = 16); those groups are unsaturated because the peak grows with W as file size grows.
- **Why W grows with file size**: in P4 S = 4 K = 144 N = 27648, across 193 delete publishes, `ordinary_slots` − ⌈`kept_data_units` ÷ 144⌉ takes only 4, 5, 7 (2, 146, 45 times). Every delete publish rewrites every remaining lower extent leaf, so W follows the remaining leaf count. D8 settled item 16 clause 2 defines a delete batch that “rewrites only the truncated lower leaf and its ancestors”; the patch-table path (prototype wiring, truncating extent to first M units through `publish_version`) is not that path. With only one lower leaf (N ≤ 144) both rewrite the same leaf (inference; not measured on tail-only path).
- **M6 (W versus placement demand)**: W > W_land occurs 0 times; maximum W ÷ W_land is 1000‰ in each P1/P2 group (equal), 27‰–203‰ in P4 (placement-demand ckpt_cost counts the entire allocation record tree).
- **M7 (ring-window bound)**: among 272 cells, maximum apex − (3S + 1) × W_max is −56 (P4 S = 4 K = 1 N = 14); none exceeds it.
- **Free slots at refusal versus previous W** (inferred proxy: no W reading for the refused publish, so compare previous successful delete publish in the cell): 47 cells have free < previous W; 18 have free ≥ previous W (7 P1 S = 16 K = 144 N-A, 7 P1 S = 16 K = 1 N-A, 2 N-B there, 1 P1 S = 4 K = 1 N-A, 1 P2 S = 16 K = 1 N-A). Most of these 18 likely fail because placement demand (ordinary allocation + ckpt_cost) exceeds actual newly occupied slots.
- **Positive controls**: all eight cells refuse (A `refusal=landing`, B `refusal=no_space_after_raising_the_floor`); three of four B cells have `floor_raise_publishes=2 floor_raise_stop=floor_already_at_the_ceiling`; P1 S = 16 N-B has `floor_raise_publishes=0 floor_raise_stop=floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2`. `publishes_per_admission_would_be_exceeded` never appears. Eight B5 rows.
- **Anchors and stops**: a1 112 rows, a4 140 rows, B1–B3 9 rows, zero `holds=false`. All 207 completed-delete cells' `name=s7_finished_delete` have `checker_violations=0`; no stop rows. At both S values P0 refused the writable mount (lines 13–14 `first_file=refused step=first_mount`), excluded from Q1 under F8.
- **δ (8.3, F6)**: among completed cells with δ ≤ 3, K = 144 has 2 N-A and 7 N-B; K = 1 has 17 N-A and 21 N-B. Annotate their completion “unmodeled items could reverse it.”
- **M8a preliminary reading** (segment-two quantity, numbers only, no verdict): group `ring_window_bound` minus minimum `reserve_minimum` among cells in that group is > 0 for 16 groups and < 0 for four P4 S = 4 groups (K = 144 −4207, K = 1 −6794); P1 group minimum reserve is for the smallest-N disk.

### Handoff table (segment one, Q1–Q4)

| Row | Status | Evidence | Missing | Can remaining measurements reverse it? |
|---|---|---|---|---|
| Q1 | Open: literal registration 5.8 supports “not covered,” but only K = 1 sensitivity and N-A on full-rewrite path; K = 144 main column unmeasured on tail-only path (main-agent judgment) | `q1_cells_refused=65`; smallest counterexample segment1 line 25777; group refusal counts above | Only 2 groups reached “file-size sweep until peak no longer grows” saturation (`q1_groups_unsaturated=22`); P5, F-start unrun. Under section eleven's sufficient-to-decide stop, no more of these cells run solely for Q1 after refusal | Cannot retract 65 refusals; remaining measurements can only add refusals. The 7 N-A K = 144 refusals depend on rewriting all lower extent leaves; tail-only path unmeasured. K = 1 and N ≤ 144: 44 cells have identical behavior under both paths (inference) |
| Q2 | Insufficient; F4 triggered (M5 grows with file size in 14 groups), M7 zero cells | `q2_cells_apex_over_ring_window=0`, `q2_groups_w_growing=14`, `q2_w_exceeds_landing_publishes=0` | L-scatter (scattered extent and allocation-record leaves), P5 (allocation-record-tree third level); K = 144 groups still growing at apparatus ceiling N = 27648; write W_max from endpoint as “≥ 217 / 214 slots (file-size ceiling)” | Can move toward “no” (L-scatter might make M7 or M6 > 0); cannot undo F4. Its growth comes from full lower-extent rewrite; whether W grows under tail-only is unmeasured |
| Q3 | Not run (segment two) | Both M8a inputs present in segment-one artifact; preliminary values above | W-A, W-B arms, M8b, M9 | Yes |
| Q4 | Not run (segment three) | — | Patch table lacks M1 resume and fill allocation-record publish; worst Q1 geometry can be taken from refused segment-one cells | Yes |

### Rerun

All are checker tier, run by the main agent with `SINGLEFS_HEAVY_TESTS` or with user consent. `scripts/experiment-apparatus/replay.sh` registers two rows (`driver_e168_s1_history`, `driver_e168_positive_controls`): the driver copies `crates/` to a temporary directory, applies patch table, compiles, runs and wraps, then compares byte for byte against checked-in artifacts. Segment1 takes about 9 hours and peaks at about 35 GB; it is not in the registry (user decision 2026-10-03). Rechecking requires an explicit user request and manual steps 2, 4, 5. Checked-in artifacts were built against the `crates/` recorded in `e168-delete-ring-window-borrow-segment1-tree-2026-10-03.sha256`; several files in `crates/singlefs-core/` were later changed by other sessions, so compare the manifest first if replay differs. Manual steps (`<草稿>` is an empty directory outside the repository):

```
# 1. 单测（主工作区那一份）
bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-checker-tier --bin e168_delete_ring_window_borrow
# 2. 套补丁表的副本（旧串必须各恰好命中一次，否则停：停机条款 S2）
mkdir -p <草稿>/tree && rsync -a --exclude target Cargo.toml Cargo.lock .cargo crates <草稿>/tree/
python3 -c 'import sys
root=sys.argv[2]; bad=0
for line in open(sys.argv[1],encoding="utf-8"):
    line=line.rstrip("\n")
    if not line or line.startswith("#"): continue
    _,path,old,new=line.split("\t"); old=old.replace("\\n","\n"); new=new.replace("\\n","\n")
    text=open(root+"/"+path,encoding="utf-8").read(); hits=text.count(old); print(path,hits); bad+=hits!=1
    if hits==1: open(root+"/"+path,"w",encoding="utf-8").write(text.replace(old,new,1))
sys.exit(1 if bad else 0)' scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv <草稿>/tree
# 3. 停机条款 S1：两份二进制各跑一遍 s1-history，输出逐字节相同
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e168_delete_ring_window_borrow -- s1-history > <草稿>/s1-main.out
(cd <草稿>/tree && bash <仓>/scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e168_delete_ring_window_borrow -- s1-history) > <草稿>/s1-patched.out
cmp <草稿>/s1-main.out <草稿>/s1-patched.out
# 4. 阳性对照冒烟与第一段（副本里；segment1 可按几何拆开跑：--geometry P0-S4 / P1-S4 / P1-S16 / P2-S4 / P2-S16 / P4-S4 / P4-S16）
(cd <草稿>/tree && E168_THREADS=4 E168_LARGE_SLICE_THREADS=1 bash <仓>/scripts/hosts/run-with-memory-cap.sh 20G cargo run -q --release -p singlefs-checker-tier --bin e168_delete_ring_window_borrow -- positive-controls) > <草稿>/pc.out
(cd <草稿>/tree && E168_THREADS=4 E168_LARGE_SLICE_THREADS=1 bash <仓>/scripts/hosts/run-with-memory-cap.sh 36G cargo run -q --release -p singlefs-checker-tier --bin e168_delete_ring_window_borrow -- segment1) > <草稿>/segment1.out
# 5. 包成入库的样子再比
awk '/^E name=/{ sub(/^E name=/, "E7RESULT name="); n++; if ($0 == "E7RESULT name=done") { print $0 " emitted=" n; next } print }' <草稿>/segment1.out | cmp - e168-delete-ring-window-borrow-segment1-2026-10-03.out
```

Running `segment1` or `positive-controls` in the main workspace emits only `E name=stop reason=delete_path_patch_absent`, exits 3 and emits no measurements.

### Unit tests and mutations

27 unit tests (`grep -c '^    #\[test\]' crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs`), one `#[ignore]` requires a patched copy: `segment_one_on_p2_s4_prints_the_same_lines_on_one_thread_and_on_eight_threads`. On 2026-10-03 the main workspace passed 26, ignored 1; the copy passed 27 (two unit-test logs). Thirteen tests added in revisions 10–17 pin the formula check disabled for S7's delete publish, slice-executor ordering and thread count, P1 reading C's minimum disk and ring scaling with record count, positive-control `free_0`, dropping roots recorded on copies during F-raise rehearsal from valley readings, reporting three positive-control numbers separately, and U12 interpretation B. Of the earliest 14, eleven pin pure arithmetic: ring length 3S, two root-level boundaries (68208 / 68209, 11527152 / 11527153 slots), five K1 anchors (529, 5, 5, 161, 41451), minimum mkfs disks (17874944, 3221225472), reserve 8 + 13c, fill interval (−16384, 0], M1 ≤ 0, M3 saturation rule, M5 strict growth, 8.3 δ, ladder and delete-publish counts. Three others pin stop-condition data collection: discard pre-delete readings when opening the valley hook (revision 8); six free-map counts return to pre-fill values after fill removal; P2 S = 4 pool fill, removal and one overwrite produce no checker failure (revision 9). The last creates a pool and runs checker.

32 apparatus mutations appear at the end of `crates/mutations.tsv` (names start “E168（删除借保留量的环窗口） ”; `grep -c '^[E]168 ' crates/mutations.tsv`), covering registration section nine U4–U10, U12, U13 and other arithmetic, measurement and revision 8–17 changes. The 2026-10-03 run demonstrated 14 failures individually (13 added by revisions 10–17 plus “ladder point 3S + 2 written as 3S + 3”): 14 caught, 0 invalid, 0 not failing (`e168-delete-ring-window-borrow-mutants-2026-10-03.log`). The three counts for the complete 32-row table await the `checker-tier-crates-mutation-replay` gate on submission. Section-nine U1, U2, U3, U11, U16 are caught only by artifact stop / invalidation conditions (S5, B5, S6, positive controls, formula-check call count); U14 and U15 belong to segment three.

### What it cannot answer

- Delete path is not D8 settled item 16 clause 2: patch-table prototype rewrites all remaining lower extent leaves on every delete publish (see “why W grows”); the clause requires only the truncated lower leaf and its ancestors. W's growth, seven K = 144 refusals and group W_max are values for this path; tail-only outcomes are unmeasured.
- M1 is always ≤ 0 by definition, so Q1's “peak > reserve” half cannot be observed by this apparatus; “not covered” is judged only from refusal.
- Placement-check demand calculates ckpt_cost for the entire allocation record tree, larger than actual new delete-publish occupancy (W is only 2.7%–20.3% of demand on P4). In 18 of 65 refused cells, free space at refusal is at least the previous delete publish's W (inferred proxy); these refusals arise from inflated check demand, not truly insufficient free slots for a batch.
- P1 finds a minimum usable disk per N, so group peak growth with N mixes in growth in disk and reserve; P2 and P4 use fixed disks.
- Patch table deletes only inode 1 (the only one with data on 2026-10-03), with no tombstone, intent record or `deleted_inodes`. W and peak are measured with these zero (registration 5.6, 8.3); δ ≤ 3 cells above.
- Two copies fixed, S only 4 and 16, only L-seq layout and F-end fill direction; P3, P5, S = 8 not run.
- Checked-in artifacts were built on the `crates/` in `e168-delete-ring-window-borrow-segment1-tree-2026-10-03.sha256`. Other sessions later changed several main-workspace `crates/singlefs-core/` files including extent_tree.rs, records.rs, recovery.rs; results replayed on current `crates/` have not been compared.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D3 (Space Allocation) settled item 2 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D3 (Space Allocation) settled item 17 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D28 (Mount-Time Commitments) settled item 1 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D28 (Mount-Time Commitments) settled item 3 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D28 (Mount-Time Commitments) settled item 4 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D8 (Core Index Structure) settled item 5 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D8 (Core Index Structure) settled item 16 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |
| D16 (Publish Semantics) settled item 1 | Background | 2026-10-18 unaffected: body changed the never-committed segment-one artifact reference to filename only and noted that the original was not retained; conclusion and numbers unchanged |

## Historical Versions

### 2026-10-01: page created, apparatus written

Apparatus, patch table, 11 unit tests and 15 apparatus mutations were written and compiled (main workspace swapped in using `scripts/hosts/compile-then-swap.py`; copy compiled after applying patch table). Unit tests, mutations and segment-one artifact had not run; status “in progress (apparatus).” The previous two dispatched executors did not return results; this version continued from their patch-table draft.

### 2026-10-01 to 2026-10-02: segment-one runs not checked in

The main agent ran segment one with the prefix. Runs before r8 stopped on apparatus or interpretation and outputs were not checked in; reasons and corrections are in pre-registration section twelve revisions 8–16: valley hook retained pre-delete readings (revision 8); S7 checker counted fill-marked slots in accounting (9, 14); original P1 could not fill (10); positive control read wrong disk (11); P1's 1 MiB journal ring could not write more than 86 units (15, user decided ring scales with N); U12 positive-control interpretation and valley capturing F-raise rehearsal readings (16, 17, user chose interpretation B).

### 2026-10-03: segment one checked in, status “in progress (segment one complete)”

Segment-one r8 (twelfth executor's apparatus, SHA-256 `595ad82c…`, swapped into main workspace by main agent): unit tests, 14 demonstrated mutation failures, S2, S1, positive-control smoke and segment1 all passed with no stop rows; nine artifacts checked in, `scripts/experiment-apparatus/replay.sh` registered three rows. Q1 sufficient under F2 for “not covered” (65 refusals); Q2 insufficient (M7 zero cells, F4 triggered); Q3/Q4 unrun. Three interpretations added to measurements: M1 is ≤ 0 by definition, delete path rewrites all lower extent leaves (unlike D8 settled item 16 clause 2), and 18 cells were refused due to oversized placement demand. The affected-decisions table was reviewed row by row using segment-one numbers; D28 settled items 3/4, D8 settled item 16, D16 settled item 1 added, all background.
