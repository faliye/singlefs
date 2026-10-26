## E187 Cost of Persisting Quarantine Marks — Run (both segments run, the fork sheet row is decidable; all 18 mutations caught)

[中文](187-隔离落盘代价.md) · **English** · [日本語](187-隔離マークの永続化コスト.ja.md)

Pre-registration: `research/prompts/e187-preregistration.md` (section twelve, revisions one through five: placement rules refined against `crates/`, root-slot and system-configuration-slot values obtained by reading code, and micro-history construction and segment-two measurement basis refined; written after unit tests and before the artifact). It addresses the row in `research/prompts/m4-i315-quarantine-mark-cost-forks.md`: for “quarantine intact on disk,” should the mark not be persisted (`no_mark`), or should option C use placement A (`span_bit`, borrowing bit 0x4000 of the span field) or placement B (`quarantine_tree`, a new tree in the tree table)?

Apparatus: `scripts/experiment-apparatus/e187_quarantine_mark_cost.rs` (independently handwritten counting model, no code shared with `crates/`), `[[bin]]` name `e187-quarantine-mark-cost`; 33 unit tests (plus two for the thread registration slot module). Mutation table `scripts/mutation/tables/e187_quarantine_mark_cost.tsv`: 18 mutations, all caught (see “Unit tests and mutations”).

### Current status

- Segment-one artifact `research/results/e187-quarantine-mark-cost-segment-one-2026-10-21.out`: 250 lines, ending `E7RESULT name=done emitted=250`, `sha256sum` `e56f7e3dbaaf7bf73d06cab9c715a3ef757959fbcf3c6a563de28fcbd975cdb6`. The same source model was run once in draft and matches the artifact byte for byte by `cmp` (a deterministic model: this only establishes absence of hidden state; evidential strength rests on absolute-value assertions and mutations).
- All pinned values for eight micro-histories, three positive controls, and stop conditions (13 code-reading checks and 30 format constants checked verbatim in `crates/` source) hold.
- **Extra writes per quarantine (single event, `geometry_default`)**: A additionally writes the quarantined unit's leaf to each disk when that leaf would not otherwise be rewritten (4 writes, 65536 bytes); if ancestors are also clean, 8 writes, 131072 bytes; if the leaf was already dirty, 0. B's first tree creation adds 2 writes, 32768 bytes; when the tree exists and the old placement is in a cold leaf, 6 writes, 98304 bytes. Which of A or B is smaller changes with the sample point.
- **Rollback revival and clearing marks**: revival adds 4 writes for A (clearing 0x4000 on both disks), and 0 for B (deletion empties the tree, old placement in a hot leaf) or 4 (old placement in a cold leaf). Releasing a quarantined record adds 0 for both C variants.
- **Bit-flip classification table**: both C variants correctly pass `quarantine_intact` with a correctly written mark (`no_mark` falsely fails it). With a wrong mark, both C variants miss defects: a leak marked on both disks passes under both checker variants; `mark_or_today` also passes `quarantine_garbled` when the marks are cleared on both disks. Failure condition `mark_hides_a_leak` is hit.
- **Old readers for placement A**: five code-reading checks agree with the apparatus arithmetic. Old readers interpret 0x4001 / 0x4002 as spans 16385 / 16386, none fitting in a leaf. The allocation-record-tree reader and GPU judge reject them; the pool checker fails I-1.1 (block header states its logical address), but its non-judging pass accepts the record and uses span 16385 in accounting checks. Failure condition `old_readers_reject_the_mark` is recorded as hit under its weakest interpretation.
- **Segment-two artifact** `research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out`: 6933 lines, ending `E7RESULT name=done emitted=6933`, `sha256sum` `638631a8705a0f6e530d0026ff3d61d12b39d4cc32268acc722509698770b199`; 320 cells (density 0 / 1 / 3 / 10, rollback rate 0 / 1, U = 200 / 20000, K = 8 / 64, two geometries, seeds 1–5), each run once on all three arms, 960 histories. The old segment-two artifact `research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out` is void as a whole, not used for any number, and kept, not deleted: two of its cells have `schedule_realized=false`, so invalidation condition `scheduled_events_realized` was hit; the cause was the apparatus's rollback scheduling step (one failed draw dropped the event). After the apparatus was fixed, this new artifact was produced; the two differ only in those two cells' rows and the summary rows (see “Segment-two artifact: rerun, completeness and verdict row”).
- **Extra writes per quarantine on workloads**: counting only the quarantining publication, A's per-cell mean lies between -2.2 and 10.6 writes; the pooled mean is 0.0138 writes (U = 200) and 0.4026 writes (U = 20000) when the quarantined unit's leaf is rewritten anyway, and 4.0000 and 4.5903 writes when it is not. B's per-cell mean lies between -3 and 7.8 writes, both groups between 2.4 and 3.8 writes. Which of A and B is smaller changes with the sample point: in the 120 cells with U = 200 A is always smaller; in the 120 cells with U = 20000 A is smaller in 40, B in 78, and they are equal in 2.
- **Rollback revival on workloads**: per revived unit, A's per-cell mean is 0 to 0.4 writes (U = 200) and -2.8 to 5.2 writes (U = 20000); B's is -0.4 to 3.6 and -5.2 to 6.8.
- **Shares and bands**: at density 10 A reaches at most band two and B reaches band three (17 cells, all with U = 200), so failure condition `quarantine_cost_negligible` is hit; at densities 1 and 3 neither placement has a cell in band three; within one density the band changes with U, K and geometry, recorded as “depends on the sample point”.
- **Final-state misses and false failures, trajectories**: in the universe without flipped marks, `no_mark` has 4992 false failures and 0 misses, and the four C combinations have 0 false failures and 0 misses, recorded as “C dominates”; in the universe with flipped marks `mark_or_today` misses 4966 and `mark_only` misses 0, so failure condition `mark_hides_a_leak` is hit on `mark_or_today`. Of the 5600 intact quarantine events of `no_mark`, 606 end at 0, all in cells with rollback rate 1, so failure condition `no_mark_false_red_persists` is hit; the 2800 at rollback rate 0 all end positive.
- The fork sheet row is decidable (see “Fork table”).

### Rerun

```bash
# 仓根；两段产物逐字节复跑登记在 scripts/experiment-apparatus/replay.sh 两行：
#   E187|e187-quarantine-mark-cost|segment-one|e187-quarantine-mark-cost-segment-one-2026-10-21.out|exact
#   E187|e187-quarantine-mark-cost|segment-two|e187-quarantine-mark-cost-segment-two-2026-10-22.out|exact
cd scripts/experiment-apparatus
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e187-quarantine-mark-cost 的单测; 范围=1 个测试目标; 为什么=复核单测' \
  bash ../hosts/capped.sh 31 bash ../hosts/run-with-memory-cap.sh 3G cargo test --release --bin e187-quarantine-mark-cost
bash ../hosts/capped.sh 31 cargo build --release --bin e187-quarantine-mark-cost
bash ../hosts/run-with-memory-cap.sh 1G target/release/e187-quarantine-mark-cost segment-one > <新文件名>.out
bash ../hosts/capped.sh 31 bash ../hosts/run-with-memory-cap.sh 4G target/release/e187-quarantine-mark-cost segment-two > <新文件名>.out
MUTATE_MEMORY_MAX=3G MUTATE_TIMEOUT=900 nice -n 19 bash ../mutation/mutate.sh e187-quarantine-mark-cost e187_quarantine_mark_cost.rs ../mutation/tables/e187_quarantine_mark_cost.tsv
```

### Measurements: extra writes for quarantine, rollback revival and releasing quarantined records (segment one, eight micro-histories)

Each arm minus `no_mark`, excluding root and system configuration slots (identical for all three arms); bytes summed across both disks, with the single-disk basis being half. Eight `name=micro_pin` rows from `e187-quarantine-mark-cost-segment-one-2026-10-21.out`, copied in full:

```text
67:E7RESULT name=micro_pin history=micro_clean_leaf span_bit_extra=4/65536 span_bit_expected=4/65536 quarantine_tree_extra=2/32768 quarantine_tree_expected=2/32768 holds=true
72:E7RESULT name=micro_pin history=micro_clean_ancestor span_bit_extra=8/131072 span_bit_expected=8/131072 quarantine_tree_extra=2/32768 quarantine_tree_expected=2/32768 holds=true
76:E7RESULT name=micro_pin history=micro_dirty_leaf span_bit_extra=0/0 span_bit_expected=0/0 quarantine_tree_extra=2/32768 quarantine_tree_expected=2/32768 holds=true
80:E7RESULT name=micro_pin history=micro_journal_crossing span_bit_extra=6/73728 span_bit_expected=6/73728 quarantine_tree_extra=2/32768 quarantine_tree_expected=2/32768 holds=true
84:E7RESULT name=micro_pin history=micro_second_quarantine_tree_cold span_bit_extra=4/65536 span_bit_expected=4/65536 quarantine_tree_extra=6/98304 quarantine_tree_expected=6/98304 holds=true
89:E7RESULT name=micro_pin history=micro_revival_clean span_bit_extra=4/65536 span_bit_expected=4/65536 quarantine_tree_extra=0/0 quarantine_tree_expected=0/0 holds=true
93:E7RESULT name=micro_pin history=micro_revival_tree_cold span_bit_extra=4/65536 span_bit_expected=4/65536 quarantine_tree_extra=4/65536 quarantine_tree_expected=4/65536 holds=true
97:E7RESULT name=micro_pin history=micro_release_quarantined span_bit_extra=0/0 span_bit_expected=0/0 quarantine_tree_extra=0/0 quarantine_tree_expected=0/0 holds=true
```

| Micro-history | Publication measured | A (writes / bytes) | B (writes / bytes) | By structure (`name=micro` in the same artifact) |
|---|---|---|---|---|
| `micro_clean_leaf` | Quarantine (B creates tree for the first time) | 4 / 65536 | 2 / 32768 | A entirely in allocation-record tree; B entirely in quarantine tree, plus one 200-byte tree-table entry (logical bytes) |
| `micro_clean_ancestor` | Quarantine, quarantined unit in leaf 172 | 8 / 131072 | 2 / 32768 | A: leaf 172 and level-1 node covering [137228, 274456), each on both disks |
| `micro_dirty_leaf` | Quarantine, leaf already rewritten | 0 / 0 | 2 / 32768 | — |
| `micro_journal_crossing` | As `micro_clean_leaf`, 63 named entries | 6 / 73728 | 2 / 32768 | A adds one journal record (2 writes, 8192 bytes on two disks), 65 > 64 named entries; B has 64 and adds none |
| `micro_second_quarantine_tree_cold` | Second quarantine, B's old placement in leaf 70 | 4 / 65536 | 6 / 98304 | B: quarantine tree 2 writes + leaf 70 on both disks 4 writes |
| `micro_revival_clean` | Rollback revival | 4 / 65536 | 0 / 0 | A clears mark; B deletes to empty, old placement in leaf 62 |
| `micro_revival_tree_cold` | Rollback revival, B's old placement in leaf 70 | 4 / 65536 | 4 / 65536 | — |
| `micro_release_quarantined` | Releasing quarantined record | 0 / 0 | 0 / 0 | All three arms rewrite leaf 75 (including `no_mark` itself) |

- Pairwise comparison alongside absolute-value pins: A exceeds B on `micro_clean_leaf` (line 68 `span_bit_more_than_quarantine_tree=true`, A 4, B 2); B exceeds A on `micro_second_quarantine_tree_cold` (line 85 `quarantine_tree_more_than_span_bit=true quarantine_tree=6 span_bit=4`).
- Write-count positive control (line 68 in full): `E7RESULT name=positive_control control=write_counting history=micro_clean_leaf plain_release_minus_no_mark=4/65536 expected=4/65536 span_bit_minus_no_mark=4 quarantine_tree_minus_no_mark=2 span_bit_more_than_quarantine_tree=true holds=true`.
- In the version after rollback revival, the checker for each arm does not fail `mark_agrees_with_references` on the revived unit's pair of records (lines 86–88, 90–92 `mark_agreement_red_after_revival=false`): A cleared 0x4000 on both disks; B removed both entries.

### Measurements: bit-flip classification table (segment one, constructed disk states)

The artifact contains 120 `name=flip_classification` rows (5 arm/checker combinations × 4 record classes × 3 mark states × 2 versions). The table below is the 60 rows for “the version after the last root referencing it left the I-3.1 candidate window” (`version=after_window`), extracted cell by cell from the artifact by one `python3 -I` command (command and original output in the executor report). The 60 rows for “the quarantine version” differ only in I-3.1 in the quarantine checks (the old root remains in the window and I-3.1 does not fail); the same command counted zero cells differing in anything else. Cells give failed checks (or `green`) and ground-truth outcome. Ground truth: `quarantine_intact`, `quarantine_garbled`, and `normal` with correct marks are valid disks; leaks and any incorrectly written marks are defects.

| Record class | Mark | `no_mark/today` | `span_bit/mark_only` | `span_bit/mark_or_today` | `quarantine_tree/mark_only` | `quarantine_tree/mark_or_today` |
|---|---|---|---|---|---|---|
| `quarantine_intact` | `written_correctly` | I-3.1+I-3.11+I-3.15（false_red） | green（correct_green） | green（correct_green） | green（correct_green） | green（correct_green） |
| `quarantine_intact` | `wrong_on_both_devices` | I-1.1（caught） | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） |
| `quarantine_intact` | `wrong_on_one_device` | I-1.1（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） |
| `quarantine_garbled` | `written_correctly` | green（correct_green） | green（correct_green） | green（correct_green） | green（correct_green） | green（correct_green） |
| `quarantine_garbled` | `wrong_on_both_devices` | I-1.1（caught） | I-3.1+I-3.11+I-3.15（caught） | green（passed_defect） | I-3.1+I-3.11+I-3.15（caught） | green（passed_defect） |
| `quarantine_garbled` | `wrong_on_one_device` | I-1.1（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） |
| `leak` | `written_correctly` | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） | I-3.1+I-3.11+I-3.15（caught） |
| `leak` | `wrong_on_both_devices` | I-1.1（caught） | green（passed_defect） | green（passed_defect） | green（passed_defect） | green（passed_defect） |
| `leak` | `wrong_on_one_device` | I-1.1（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） | I-3.1+I-3.11+I-3.15+mark_agrees_with_references（caught） |
| `normal` | `written_correctly` | green（correct_green） | green（correct_green） | green（correct_green） | green（correct_green） | green（correct_green） |
| `normal` | `wrong_on_both_devices` | I-1.1（caught） | mark_agrees_with_references（caught） | mark_agrees_with_references（caught） | mark_agrees_with_references（caught） | mark_agrees_with_references（caught） |
| `normal` | `wrong_on_one_device` | I-1.1（caught） | mark_agrees_with_references（caught） | mark_agrees_with_references（caught） | mark_agrees_with_references（caught） | mark_agrees_with_references（caught） |

- A bit flip on the medium (node checksum mismatch) fails I-2.1 (Checksum Matches Content) in all three arms, shown separately on line 229 and excluded from missed defects and false failures.
- “Wrongly written” in the `no_mark` column means setting the same bit 0x4000: today's readers interpret it as an out-of-bounds span, I-1.1 fails, and the quarantine-check pass no longer considers it.
- Missed defects and false failures are counted in two universes (lines 230–239 copied in full):

```text
230:E7RESULT name=constructed_pass_and_false_red arm=no_mark checker=today universe=marks_not_flipped passed_defects=0 false_reds=2 judged_disks=8 reading=baseline
231:E7RESULT name=constructed_pass_and_false_red arm=no_mark checker=today universe=marks_flipped passed_defects=0 false_reds=0 judged_disks=16 reading=baseline
232:E7RESULT name=constructed_pass_and_false_red arm=span_bit checker=mark_only universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=8 reading=mark_dominates
233:E7RESULT name=constructed_pass_and_false_red arm=span_bit checker=mark_only universe=marks_flipped passed_defects=2 false_reds=0 judged_disks=16 reading=mark_does_not_solve_this_shape
234:E7RESULT name=constructed_pass_and_false_red arm=span_bit checker=mark_or_today universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=8 reading=mark_dominates
235:E7RESULT name=constructed_pass_and_false_red arm=span_bit checker=mark_or_today universe=marks_flipped passed_defects=4 false_reds=0 judged_disks=16 reading=mark_does_not_solve_this_shape
236:E7RESULT name=constructed_pass_and_false_red arm=quarantine_tree checker=mark_only universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=8 reading=mark_dominates
237:E7RESULT name=constructed_pass_and_false_red arm=quarantine_tree checker=mark_only universe=marks_flipped passed_defects=2 false_reds=0 judged_disks=16 reading=mark_does_not_solve_this_shape
238:E7RESULT name=constructed_pass_and_false_red arm=quarantine_tree checker=mark_or_today universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=8 reading=mark_dominates
239:E7RESULT name=constructed_pass_and_false_red arm=quarantine_tree checker=mark_or_today universe=marks_flipped passed_defects=4 false_reds=0 judged_disks=16 reading=mark_does_not_solve_this_shape
```

- Unflipped-mark universe (same universe for all three arms, 8 disk states: 4 classes × 2 versions): `no_mark` has 2 false failures (`quarantine_intact` in both versions), 0 missed defects; the four C combinations have 0 false failures and 0 missed defects, recorded as “C dominates.”
- Flipped-mark universe (16 disk states, all defective): `no_mark` misses 0; `mark_only` misses 2 (leak marked on both disks in both versions), `mark_or_today` misses 4 (also `quarantine_garbled` with both marks cleared in both versions). There are no valid disks in this universe, so false failures are always 0. The literal classification is “C does not solve this shape,” which here does not distinguish the arms (the “hit does not distinguish arms” case in `.claude/singlefs-ai-sop/rules/evidence-discipline.md`, “the criterion itself can be wrong: classify the hit first”). Both counts are presented to the user side by side.
- A and B have identical cells: on the checker side the only difference is where the mark lives (one span-field bit or an entry in the quarantine tree). This table cannot distinguish placements; they differ only in write volume and old-reader behavior.

### Measurements: old readers for placement A (segment one, arithmetic and code reading)

```text
240:E7RESULT name=old_reader_arithmetic borrowed_bit=0x4000 record_span=1 span_field=0x4001 span_read=16385 legal_starts_per_leaf=812 accepted=0
241:E7RESULT name=old_reader_arithmetic borrowed_bit=0x4000 record_span=2 span_field=0x4002 span_read=16386 legal_starts_per_leaf=406 accepted=0
242:E7RESULT name=old_reader_arithmetic borrowed_bit=0x0004 record_span=1 span_field=0x0005 span_read=5 legal_starts_per_leaf=812 accepted=808
243:E7RESULT name=old_reader_arithmetic borrowed_bit=0x0004 record_span=2 span_field=0x0006 span_read=6 legal_starts_per_leaf=406 accepted=404
```

| Old reader (code read in current workspace) | Fields read (span field 0x4001 / 0x4002) | Subsequent behavior |
|---|---|---|
| Core record parser `crates/singlefs-core/src/allocator.rs:80` `AllocationRecord::parse` | Device and slot as written; span 16385 / 16386; allocation generation as written; freed = no | Returns to caller, without judging here |
| Allocation record tree reader `crates/singlefs-core/src/allocation_record_tree.rs:273` `record_fits_in_its_leaf`, `:738` | Same | Rejects as `AllocationRecordOutsideThePoolGeometry`: mount and recovery tree reads fail |
| Recovery geometry check `crates/singlefs-core/src/recovery.rs:2361` `allocation_records_fit_the_pool_geometry` (called at `:2512`) | Device, slot, span 16385 / 16386 | Tree read rejects earlier, so this is not reached; in isolation rejects only if the span crosses the unit-area end or covers another record |
| Pool-level checker `crates/singlefs-checker/src/walk.rs:5293` `parse_allocation_record`, `:4394`, `:6171`, `:5752` | Same as core | Judging pass fails I-1.1; non-judging pass (`allocation_record_tree_without_judging`) accepts the record and passes span 16385 into references used by accounting checks |
| GPU judge `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2566` | Span = `span_field & 0x7FFF` = 16385 / 16386; span field stored unchanged | Walk stops at `FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY` |

- All five code-reading checks agree point by point with apparatus arithmetic (bit 0x4000 makes every record fail to fit its leaf), so the stop condition does not trigger. Under the reversal wording in section six of the registration, four of the five reject or fail; the pool checker also uses the misread span after failing it, so this is not “all five reject.” Placement A falls under “changing the meaning of an existing field” in the D15 (Format Freeze Policy) settled item 9 decision table. Which instance applies (“old readers reject all nonzero bits” or “incorrect behavior without an error”) is an inference handed to the main agent.
- Low-bit 0x0004 control: 808 / 812 and 404 / 406 records still fit their leaves, with spans read as 5 / 6.

### Measurements: extra writes per quarantine and per rollback revival on segment-two workloads

Measurement basis: in each cell the same series of quarantine and rollback events runs once on each of the three arms; each arm minus `no_mark`, excluding root and system configuration slots. “Per quarantine” is reported under both algorithms of registration section six: counting only the quarantining publication (artifact fields `per_event_*`), and the whole-history difference divided by the event count (fields `whole_history_*`). 240 cells have events (densities 1, 3, 10), each with 10 × density events. Every number in this section is counted from artifact `e187-quarantine-mark-cost-segment-two-2026-10-22.out` by these commands (run from the repository root):

```bash
grep 'name=extra_writes ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep -v ' events=0 ' | awk '{for(i=3;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} split(v["per_event_device_writes_mean_min_max"],m,"/"); k=v["arm"]" file_size="v["file_size"]; x=m[1]+0; if(!(k in lo)||x<lo[k])lo[k]=x; if(!(k in hi)||x>hi[k])hi[k]=x; if(!(k in mn)||m[2]+0<mn[k])mn[k]=m[2]+0; if(!(k in mx)||m[3]+0>mx[k])mx[k]=m[3]+0; n[k]++; if(x>0)p[k]++; else if(x==0)z[k]++; else g[k]++} END{for(k in n)print k, "cells="n[k], "mean_low="lo[k], "mean_high="hi[k], "min="mn[k], "max="mx[k], "mean_positive="p[k]+0, "mean_zero="z[k]+0, "mean_negative="g[k]+0}' | sort
```

```text
quarantine_tree file_size=20000 cells=120 mean_low=-3 mean_high=7.8 min=-36 max=36 mean_positive=111 mean_zero=2 mean_negative=7
quarantine_tree file_size=200 cells=120 mean_low=1.2 mean_high=4 min=-6 max=10 mean_positive=120 mean_zero=0 mean_negative=0
span_bit file_size=20000 cells=120 mean_low=-2.2 mean_high=10.6 min=-26 max=34 mean_positive=118 mean_zero=1 mean_negative=1
span_bit file_size=200 cells=120 mean_low=-0.2667 mean_high=1.6 min=-8 max=8 mean_positive=82 mean_zero=37 mean_negative=1
```

```bash
grep 'name=extra_writes ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep -v ' events=0 ' | awk '{for(i=3;i<=NF;i++){p=index($i,"=");v[substr($i,1,p-1)]=substr($i,p+1)} for(j=0;j<2;j++){f=(j==0)?"per_event_device_writes_mean_when_the_leaf_is_rewritten_anyway":"per_event_device_writes_mean_when_the_leaf_is_not"; split(v[f],q,"[(]n="); c=q[2]+0; k=v["arm"]" file_size="v["file_size"]" "((j==0)?"leaf_rewritten_anyway":"leaf_not_rewritten"); if(q[1]!="none")t[k]+=q[1]*c; e[k]+=c}} END{for(k in e)if(e[k]>0)printf "%s events=%d pooled_mean=%.4f\n", k, e[k], t[k]/e[k]; else print k, "events=0"}' | sort
```

```text
quarantine_tree file_size=20000 leaf_not_rewritten events=4835 pooled_mean=3.5355
quarantine_tree file_size=20000 leaf_rewritten_anyway events=765 pooled_mean=3.7046
quarantine_tree file_size=200 leaf_not_rewritten events=381 pooled_mean=3.2913
quarantine_tree file_size=200 leaf_rewritten_anyway events=5219 pooled_mean=2.4338
span_bit file_size=20000 leaf_not_rewritten events=4835 pooled_mean=4.5903
span_bit file_size=20000 leaf_rewritten_anyway events=765 pooled_mean=0.4026
span_bit file_size=200 leaf_not_rewritten events=381 pooled_mean=4.0000
span_bit file_size=200 leaf_rewritten_anyway events=5219 pooled_mean=0.0138
```

```bash
grep 'name=extra_writes ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep ' rollback_rate=1 ' | grep -v ' events=0 ' | awk '{for(i=3;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} split(v["revival_device_writes_per_revived_mean_min_max"],m,"/"); k=v["arm"]" file_size="v["file_size"]; x=m[1]+0; if(!(k in lo)||x<lo[k])lo[k]=x; if(!(k in hi)||x>hi[k])hi[k]=x; if(!(k in mn)||m[2]+0<mn[k])mn[k]=m[2]+0; if(!(k in mx)||m[3]+0>mx[k])mx[k]=m[3]+0; n[k]++} END{for(k in n)print k, "cells="n[k], "revival_mean_low="lo[k], "revival_mean_high="hi[k], "revival_min="mn[k], "revival_max="mx[k]}' | sort
```

```text
quarantine_tree file_size=20000 cells=60 revival_mean_low=-5.2 revival_mean_high=6.8 revival_min=-20 revival_max=22
quarantine_tree file_size=200 cells=60 revival_mean_low=-0.4 revival_mean_high=3.6 revival_min=-4 revival_max=6
span_bit file_size=20000 cells=60 revival_mean_low=-2.8 revival_mean_high=5.2 revival_min=-20 revival_max=26
span_bit file_size=200 cells=60 revival_mean_low=0 revival_mean_high=0.4 revival_min=0 revival_max=4
```

```bash
grep 'name=extra_writes ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep -v ' events=0 ' | awk '{for(i=3;i<=NF;i++){p=index($i,"=");v[substr($i,1,p-1)]=substr($i,p+1)} c=v["density"]" "v["rollback_rate"]" "v["file_size"]" "v["window"]" "v["geometry"]" "v["seed"]; split(v["per_event_device_writes_mean_min_max"],m,"/"); one[c,v["arm"]]=m[1]+0; two[c,v["arm"]]=v["whole_history_device_writes_per_event"]+0; u[c]=v["file_size"]} END{for(c in u){a=one[c,"span_bit"];b=one[c,"quarantine_tree"]; r=(a<b)?"span_bit_cheaper":((a>b)?"quarantine_tree_cheaper":"equal"); print "isolation_publish file_size="u[c], r; a=two[c,"span_bit"];b=two[c,"quarantine_tree"]; r=(a<b)?"span_bit_cheaper":((a>b)?"quarantine_tree_cheaper":"equal"); print "whole_history file_size="u[c], r}}' | sort | uniq -c
```

```text
      2 isolation_publish file_size=20000 equal
     78 isolation_publish file_size=20000 quarantine_tree_cheaper
     40 isolation_publish file_size=20000 span_bit_cheaper
    120 isolation_publish file_size=200 span_bit_cheaper
     27 whole_history file_size=20000 quarantine_tree_cheaper
     93 whole_history file_size=20000 span_bit_cheaper
      6 whole_history file_size=200 quarantine_tree_cheaper
    114 whole_history file_size=200 span_bit_cheaper
```

The apparatus's own summary (comparing the quarantining publication only; artifact line 6915, copied in full):

```text
6915:E7RESULT name=cheaper_placement_summary span_bit_cheaper_cells=160 quarantine_tree_cheaper_cells=78 equal_cells=2 reading=depends_on_sampling_point
```

| Quantity (device writes) | A `span_bit` | B `quarantine_tree` |
|---|---|---|
| Quarantining publication, per-cell mean, U = 200 | -0.2667 to 1.6 | 1.2 to 4 |
| Same, U = 20000 | -2.2 to 10.6 | -3 to 7.8 |
| Minimum and maximum of a single event (both U together) | -26 and 34 | -36 and 36 |
| Leaf rewritten anyway, pooled mean, U = 200 / 20000 | 0.0138 / 0.4026 | 2.4338 / 3.7046 |
| Leaf not otherwise rewritten, pooled mean, U = 200 / 20000 | 4.0000 / 4.5903 | 3.2913 / 3.5355 |
| Rollback revival, per revived unit, per-cell mean, U = 200 | 0 to 0.4 (single 0 to 4) | -0.4 to 3.6 (single -4 to 6) |
| Same, U = 20000 | -2.8 to 5.2 (single -20 to 26) | -5.2 to 6.8 (single -20 to 22) |

- A's writes follow whether that leaf is dirty anyway: close to 0 when it is rewritten anyway, around 4 when it is not, matching the 0 and 4 of the segment-one micro-histories; B's two groups both lie between 2.4 and 3.8 and do not follow this.
- Negative values are real: after the first quarantine the three arms' placements and node ages diverge (A changed a record, B has an extra tree), so in a later publication an arm can write less than `no_mark`; this difference is exactly what registration section six asks for and is reported as is. Cells with a negative mean: 2 for A, 7 for B (`mean_negative` in the output of the command counting means by arm and U).
- Which of A and B is smaller (reversal wording of registration section six: same sign at all sample points records that placement as cheaper; a sign change across sample points records “depends on the sample point” and lists the points): both algorithms change sign, recorded as “depends on the sample point”. Under the quarantining-publication algorithm, all 24 parameter points with U = 200 have A smaller for all five seeds; of the 24 parameter points with U = 20000, 21 change sign between seeds and 3 have B smaller for all five seeds (lines 6882, 6904, 6911, `name=cheaper_placement_by_point`). Under the whole-history algorithm, at U = 200 A is smaller in 114 cells and B in 6; at U = 20000 A in 93 and B in 27.

### Measurements: segment-two shares and bands (by quarantine-event density)

Measurement basis: share = bytes an arm writes beyond `no_mark` over one history ÷ total bytes written by `no_mark`, reported under both bases (excluding / including root and system configuration slots); band one < 0.1%, band two 0.1% to 1%, band three ≥ 1%. The 16 per-density summary rows (copied in full from the artifact):

```text
6723:E7RESULT name=share_tier_by_density arm=span_bit density=0 scope=without_root_and_system_configuration cells=80 highest_tier=1 lowest_tier=1 tier_counts=80/0/0 largest_share_ppm=0.0000
6732:E7RESULT name=share_tier_by_density arm=span_bit density=0 scope=with_root_and_system_configuration cells=80 highest_tier=1 lowest_tier=1 tier_counts=80/0/0 largest_share_ppm=0.0000
6741:E7RESULT name=share_tier_by_density arm=span_bit density=1 scope=without_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=74/6/0 largest_share_ppm=2564.9696
6750:E7RESULT name=share_tier_by_density arm=span_bit density=1 scope=with_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=74/6/0 largest_share_ppm=2555.1301
6759:E7RESULT name=share_tier_by_density arm=span_bit density=3 scope=without_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=67/13/0 largest_share_ppm=3471.5275
6768:E7RESULT name=share_tier_by_density arm=span_bit density=3 scope=with_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=67/13/0 largest_share_ppm=3458.1479
6777:E7RESULT name=share_tier_by_density arm=span_bit density=10 scope=without_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=70/10/0 largest_share_ppm=6926.6487
6786:E7RESULT name=share_tier_by_density arm=span_bit density=10 scope=with_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=70/10/0 largest_share_ppm=6866.0309
6795:E7RESULT name=share_tier_by_density arm=quarantine_tree density=0 scope=without_root_and_system_configuration cells=80 highest_tier=1 lowest_tier=1 tier_counts=80/0/0 largest_share_ppm=0.0000
6804:E7RESULT name=share_tier_by_density arm=quarantine_tree density=0 scope=with_root_and_system_configuration cells=80 highest_tier=1 lowest_tier=1 tier_counts=80/0/0 largest_share_ppm=0.0000
6813:E7RESULT name=share_tier_by_density arm=quarantine_tree density=1 scope=without_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=53/27/0 largest_share_ppm=3807.4299
6822:E7RESULT name=share_tier_by_density arm=quarantine_tree density=1 scope=with_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=53/27/0 largest_share_ppm=3773.8325
6831:E7RESULT name=share_tier_by_density arm=quarantine_tree density=3 scope=without_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=41/39/0 largest_share_ppm=6879.0440
6840:E7RESULT name=share_tier_by_density arm=quarantine_tree density=3 scope=with_root_and_system_configuration cells=80 highest_tier=2 lowest_tier=1 tier_counts=41/39/0 largest_share_ppm=6818.3874
6849:E7RESULT name=share_tier_by_density arm=quarantine_tree density=10 scope=without_root_and_system_configuration cells=80 highest_tier=3 lowest_tier=1 tier_counts=26/37/17 largest_share_ppm=19270.9736
6858:E7RESULT name=share_tier_by_density arm=quarantine_tree density=10 scope=with_root_and_system_configuration cells=80 highest_tier=3 lowest_tier=1 tier_counts=26/37/17 largest_share_ppm=19102.3258
```

Lowest and highest band per parameter point within one density (U, geometry, K; five seeds per point):

```bash
grep 'name=share_tier_by_point ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | awk '{for(i=3;i<=NF;i++){p=index($i,"=");v[substr($i,1,p-1)]=substr($i,p+1)} k=v["arm"]" density="v["density"]" "v["scope"]; t=v["lowest_tier"]"-"v["highest_tier"]; s[k]=s[k]" U="v["file_size"]"/"v["geometry"]"/K="v["window"]":"t} END{for(k in s)print k s[k]}' | sort -k1,1 -k2,2V -k3,3
```

```text
quarantine_tree density=0 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-1 U=20000/geometry_default/K=8:1-1 U=20000/geometry_default/K=64:1-1 U=20000/geometry_wide/K=8:1-1 U=20000/geometry_wide/K=64:1-1
quarantine_tree density=0 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-1 U=20000/geometry_default/K=8:1-1 U=20000/geometry_default/K=64:1-1 U=20000/geometry_wide/K=8:1-1 U=20000/geometry_wide/K=64:1-1
quarantine_tree density=1 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-2 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
quarantine_tree density=1 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-2 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
quarantine_tree density=3 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:2-2 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:2-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
quarantine_tree density=3 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:2-2 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:2-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
quarantine_tree density=10 without_root_and_system_configuration U=200/geometry_default/K=8:1-3 U=200/geometry_default/K=64:1-2 U=200/geometry_wide/K=8:2-3 U=200/geometry_wide/K=64:2-3 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
quarantine_tree density=10 with_root_and_system_configuration U=200/geometry_default/K=8:1-3 U=200/geometry_default/K=64:1-2 U=200/geometry_wide/K=8:2-3 U=200/geometry_wide/K=64:2-3 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
span_bit density=0 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-1 U=20000/geometry_default/K=8:1-1 U=20000/geometry_default/K=64:1-1 U=20000/geometry_wide/K=8:1-1 U=20000/geometry_wide/K=64:1-1
span_bit density=0 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-1 U=20000/geometry_default/K=8:1-1 U=20000/geometry_default/K=64:1-1 U=20000/geometry_wide/K=8:1-1 U=20000/geometry_wide/K=64:1-1
span_bit density=1 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-1 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
span_bit density=1 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-1 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
span_bit density=3 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-2 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
span_bit density=3 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-2 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
span_bit density=10 without_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
span_bit density=10 with_root_and_system_configuration U=200/geometry_default/K=8:1-1 U=200/geometry_default/K=64:1-1 U=200/geometry_wide/K=8:1-1 U=200/geometry_wide/K=64:1-2 U=20000/geometry_default/K=8:1-2 U=20000/geometry_default/K=64:1-2 U=20000/geometry_wide/K=8:1-2 U=20000/geometry_wide/K=64:1-2
```

Cells in band three:

```bash
grep 'name=extra_writes ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | awk '{for(i=3;i<=NF;i++){p=index($i,"=");v[substr($i,1,p-1)]=substr($i,p+1)} if(v["tier_without_root"]==3||v["tier_with_root"]==3)print "arm="v["arm"], "density="v["density"], "file_size="v["file_size"], "tier_without_root="v["tier_without_root"], "tier_with_root="v["tier_with_root"]}' | sort | uniq -c
```

```text
     17 arm=quarantine_tree density=10 file_size=200 tier_without_root=3 tier_with_root=3
```

Both arms of the cell with the largest share (copied in full from the artifact) and the discriminative power self-test (line 6931 in full):

```text
5424:E7RESULT name=extra_writes density=10 rollback_rate=0 file_size=200 window=64 geometry=geometry_wide seed=4 arm=span_bit events=100 per_event_device_writes_mean_min_max=1.2000/-8/8 per_event_bytes_mean_min_max=19660.8000/-131072/131072 per_event_device_writes_mean_when_the_leaf_is_rewritten_anyway=0.6588(n=85) per_event_device_writes_mean_when_the_leaf_is_not=4.2667(n=15) per_event_structure_sums extra_allocation_record_tree=120/1966080 extra_central_mapping=0/0 extra_quarantine_tree=0/0 extra_journal=0/0 whole_history_device_writes_per_event=41.6800 whole_history_bytes_per_event=682885.1200 share_without_root_ppm=6926.6487 tier_without_root=2 share_with_root_ppm=6866.0309 tier_with_root=2 revival_device_writes_per_revived_mean_min_max=none/none/none revival_bytes_per_revived_mean_min_max=none/none/none
5425:E7RESULT name=extra_writes density=10 rollback_rate=0 file_size=200 window=64 geometry=geometry_wide seed=4 arm=quarantine_tree events=100 per_event_device_writes_mean_min_max=3.3200/-2/10 per_event_bytes_mean_min_max=54394.8800/-32768/163840 per_event_device_writes_mean_when_the_leaf_is_rewritten_anyway=3.0824(n=85) per_event_device_writes_mean_when_the_leaf_is_not=4.6667(n=15) per_event_structure_sums extra_allocation_record_tree=132/2162688 extra_central_mapping=0/0 extra_quarantine_tree=200/3276800 extra_journal=0/0 whole_history_device_writes_per_event=115.9600 whole_history_bytes_per_event=1899888.6400 share_without_root_ppm=19270.9736 tier_without_root=3 share_with_root_ppm=19102.3258 tier_with_root=3 revival_device_writes_per_revived_mean_min_max=none/none/none revival_bytes_per_revived_mean_min_max=none/none/none
6931:E7RESULT name=share_tier_discrimination arm=span_bit density=10 rollback_rate=0 window=8 geometry=geometry_default seed=1 scope=without_root_and_system_configuration small_file_share_ppb=-8131 large_file_share_ppb=120101 moved_tier_two_from_ppb=55986 moved_tiers=1/2 restored_tiers=1/1 separates_when_moved=true restored_equals_original=true
```

- The three reversal sentences of registration section six, one by one: “both placements are in band one in every cell at density 10” does not hold (line 6777, A `tier_counts=70/10/0`; line 6849, B `tier_counts=26/37/17`); “either placement enters band three in some cell at density ≤ 3” did not occur (the third band is 0 in `tier_counts` of all eight rows for densities 1 and 3); “within one density the band changes with U or geometry” holds, recorded as “depends on the sample point”: at density 1 A's four points with U = 200 are all in band one and its four points with U = 20000 reach band two; at density 10 three of B's four points with U = 200 reach band three (the `geometry_default`, K = 64 point reaches band two), and its four points with U = 20000 reach band two.
- The two bases give the same band at every point: in the parameter-point output, the two rows for the same arm and density are identical apart from the basis name; the 17 band-three cells are in band three under both bases.
- The share's numerator is the whole-history difference, not only the write of the mark itself: on line 5425 B's quarantining publication averages 3.3200 writes per event, while the whole history averages 115.9600 per event; on line 5424 A has 1.2000 against 41.6800. After divergence the difference can be positive or negative; on line 6931 A's share in the U = 200 cell is `small_file_share_ppb=-8131`.
- Discriminative power self-test (last paragraph of registration section eight): with the band bound moved between A's shares at the U = 200 and U = 20000 points, the two points fall into different bands (`moved_tiers=1/2`, `separates_when_moved=true`); moved back, the original judgment is restored (`restored_equals_original=true`).

### Measurements: segment-two final-state misses and false failures, and the `no_mark` false-failure trajectory

In the final state every unit still quarantined is classified by copy condition and age, and normal units form one class; each class and mark state gets one disk state. Summed over all 320 cells (artifact lines 6916–6930, copied in full):

```text
6916:E7RESULT name=terminal_pass_and_false_red_total arm=no_mark checker=today universe=marks_not_flipped passed_defects=0 false_reds=4992 judged_disks=3357367 reading=baseline
6917:E7RESULT name=terminal_pass_and_false_red_total arm=no_mark checker=today universe=marks_flipped passed_defects=0 false_reds=0 judged_disks=6714734 reading=baseline
6918:E7RESULT name=terminal_pass_and_false_red_total arm=span_bit checker=mark_only universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=3357367 reading=mark_dominates
6919:E7RESULT name=terminal_pass_and_false_red_total arm=span_bit checker=mark_only universe=marks_flipped passed_defects=0 false_reds=0 judged_disks=6714734 reading=mark_does_not_solve_this_shape
6920:E7RESULT name=terminal_pass_and_false_red_total arm=span_bit checker=mark_or_today universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=3357367 reading=mark_dominates
6921:E7RESULT name=terminal_pass_and_false_red_total arm=span_bit checker=mark_or_today universe=marks_flipped passed_defects=4966 false_reds=0 judged_disks=6714734 reading=mark_does_not_solve_this_shape
6922:E7RESULT name=terminal_pass_and_false_red_total arm=quarantine_tree checker=mark_only universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=3357605 reading=mark_dominates
6923:E7RESULT name=terminal_pass_and_false_red_total arm=quarantine_tree checker=mark_only universe=marks_flipped passed_defects=0 false_reds=0 judged_disks=6715210 reading=mark_does_not_solve_this_shape
6924:E7RESULT name=terminal_pass_and_false_red_total arm=quarantine_tree checker=mark_or_today universe=marks_not_flipped passed_defects=0 false_reds=0 judged_disks=3357605 reading=mark_dominates
6925:E7RESULT name=terminal_pass_and_false_red_total arm=quarantine_tree checker=mark_or_today universe=marks_flipped passed_defects=4966 false_reds=0 judged_disks=6715210 reading=mark_does_not_solve_this_shape
6926:E7RESULT name=trajectory_total arm=no_mark checker=today intact_events=5600 final_value_zero=606 final_value_positive=4994
6927:E7RESULT name=trajectory_total arm=span_bit checker=mark_only intact_events=5600 final_value_zero=5600 final_value_positive=0
6928:E7RESULT name=trajectory_total arm=span_bit checker=mark_or_today intact_events=5600 final_value_zero=5600 final_value_positive=0
6929:E7RESULT name=trajectory_total arm=quarantine_tree checker=mark_only intact_events=5600 final_value_zero=5600 final_value_positive=0
6930:E7RESULT name=trajectory_total arm=quarantine_tree checker=mark_or_today intact_events=5600 final_value_zero=5600 final_value_positive=0
```

The `no_mark` trajectories split by rollback rate and K:

```bash
grep 'name=trajectory ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep ' arm=no_mark ' | grep -v ' intact_events=0 ' | awk '{for(i=3;i<=NF;i++){p=index($i,"=");v[substr($i,1,p-1)]=substr($i,p+1)} k="rollback_rate="v["rollback_rate"]" window="v["window"]; z[k]+=v["final_value_zero"]; q[k]+=v["final_value_positive"]; split(v["peak_mean_min_max"],pk,"/"); if(pk[3]+0>mp[k])mp[k]=pk[3]+0; split(v["positive_versions_mean_min_max"],pv,"/"); if(!(k in lo)||pv[2]+0<lo[k])lo[k]=pv[2]+0; if(pv[3]+0>hi[k])hi[k]=pv[3]+0; o[k][v["allocated_statistic_first_red_offset_mean_min_max"]]=1} END{for(k in z){s=""; for(x in o[k])s=s" "x; print k, "final_value_zero="z[k], "final_value_positive="q[k], "peak_max="mp[k], "positive_versions_min="lo[k], "positive_versions_max="hi[k], "first_red_offsets:"s}}' | sort
```

```text
rollback_rate=0 window=64 final_value_zero=0 final_value_positive=1400 peak_max=3 positive_versions_min=10 positive_versions_max=9999 first_red_offsets: 64.0000/64/64
rollback_rate=0 window=8 final_value_zero=0 final_value_positive=1400 peak_max=3 positive_versions_min=16 positive_versions_max=9998 first_red_offsets: 8.0000/8/8
rollback_rate=1 window=64 final_value_zero=309 final_value_positive=1091 peak_max=3 positive_versions_min=12 positive_versions_max=10010 first_red_offsets: 64.0000/64/64 none/none/none
rollback_rate=1 window=8 final_value_zero=297 final_value_positive=1103 peak_max=3 positive_versions_min=9 positive_versions_max=10009 first_red_offsets: 8.0000/8/8 none/none/none
```

Among the 960 trajectory rows of the two C placements that have intact quarantine events, the number of rows whose peak is not 0:

```bash
grep 'name=trajectory ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep -v ' arm=no_mark ' | grep -v ' intact_events=0 ' | grep -vc 'peak_mean_min_max=0.0000/0/0'
```

```text
0
```

- Universe without flipped marks: `no_mark` has 4992 false failures and 0 misses; the four C combinations have 0 false failures and 0 misses, `reading=mark_dominates`, recorded per registration section six as “C dominates”.
- Universe with flipped marks: every disk state is a defect, so false failures are always 0. `mark_only` misses 0: the workload has no leaks (every row has `leak_units=0`), so the segment-one miss “a leak marked on both disks” cannot be produced here; `mark_or_today` misses 4966, from the cell where `quarantine_garbled` has its marks cleared on both disks. The reading falls literally under “C does not solve this shape”; as in segment one, this universe has no valid disk states, so that reading does not distinguish the arms here, and both sides' counts are handed to the user side by side.
- `judged_disks` is not equal across arms cell by cell (B 3357605, the other two arms 3357367): each arm counts normal disk states from the live units of its own final state, and the arms' live-unit counts differ cell by cell; the registration does not require equal disk-state counts across arms, so this is reported as is.
- Trajectories: of the 5600 intact quarantine events of `no_mark`, 606 end at 0, all in cells with rollback rate 1; the 2800 at rollback rate 0 all end positive, so the false failure stays red until the end of the history. The peak is at most 3 (I-3.1, I-3.11 and I-3.15 all red). I-3.1 turns red K versions after the quarantine: 8 when K = 8, 64 when K = 64, consistent with the “two K points in opposite directions” item of registration section eight. The events ending at 0 are those where a rollback revives the quarantined unit and the quarantine versions then leave the candidate window, clearing the false failure (unit test `a_revival_ends_the_false_red_once_the_quarantine_versions_leave_the_window` pins exactly this; this sentence is inferred from the apparatus rules, since the artifact has no per-event revival marker).
- For both C placements and both checker variants, all 5600 intact quarantine events end at 0 with peak 0: no false failure when the mark is written correctly, so failure condition `mark_removes_the_false_red` is not triggered on workloads either.

### Segment-two artifact: rerun, completeness and verdict row

The old segment-two artifact `research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out` (`sha256sum` `d5fa03adef0704075e9602fb8ddb56aa054527a4c92a13de096ba279fd0dc03a`) is void as a whole and not used for any number; it is kept, not deleted. Its verdict row has `scheduled_events_realized=false`: the two cells with density 1, rollback rate 1, U = 200, K = 64, `geometry_default`, seed=1 and seed=2, each scheduled 10 rollbacks but placed only 9. The cause was the apparatus's `schedule_of`: each quarantine drew a delay only once, and if the drawn rollback interval overlapped one already scheduled, the event was dropped. The fix keeps the first pass unchanged and, when it falls short, adds a second pass that picks a feasible delay between 1 and ⌊K ÷ 2⌋ for each dropped event, which brings the apparatus in line with the existing requirement in registration section five (“rollback events”); not a word of the registration changed, and cells already full after the first pass consume random numbers exactly as before. The new artifact was produced by a binary built from the fixed source (`sha256sum` `15ba81cfe0a4c0f6ac075283aecc96fc865efe6795ca9ca443243f7e1290cd9f`).

Compared line by line, apart from those two cells' own rows (new artifact lines 2733–2774) only summary rows differ:

```bash
diff research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep '^[<>]' | wc -l
diff research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | grep '^[<>]' | grep -v 'density=1 rollback_rate=1 file_size=200 window=64 geometry=geometry_default seed=[12] ' | awk '{print $1, $3}' | sort | uniq -c
```

```text
90
      1 < name=share_tier_by_density
      1 > name=share_tier_by_density
      9 < name=terminal_pass_and_false_red_total
      9 > name=terminal_pass_and_false_red_total
      1 < name=trajectory_total
      1 > name=trajectory_total
      1 < name=verdict
      1 > name=verdict
```

New artifact rows by result-row type:

```bash
awk '{print $1, $2}' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | sort | uniq -c
grep 'name=cell_checks ' research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out | awk '{print $(NF-1), $NF}' | sort | uniq -c
```

```text
    320 E7RESULT name=cell_checks
     48 E7RESULT name=cheaper_placement_by_point
      1 E7RESULT name=cheaper_placement_summary
      1 E7RESULT name=config
      1 E7RESULT name=done
    640 E7RESULT name=extra_writes
    960 E7RESULT name=history
      1 E7RESULT name=segment_two_config
     16 E7RESULT name=share_tier_by_density
    128 E7RESULT name=share_tier_by_point
      1 E7RESULT name=share_tier_discrimination
   3200 E7RESULT name=terminal_pass_and_false_red
     10 E7RESULT name=terminal_pass_and_false_red_total
   1600 E7RESULT name=trajectory
      5 E7RESULT name=trajectory_total
      1 E7RESULT name=verdict
    280 schedule_realized=true arms_agree_without_events=not_this_cell
     40 schedule_realized=true arms_agree_without_events=true
```

320 cells × three arms = 960 `name=history` rows; A and B each have one `name=extra_writes` row per cell, 640 rows; 6933 lines in total, equal to `emitted=6933`. All 320 cells have `schedule_realized=true`; the 40 cells with `arms_agree_without_events=true` are exactly the 40 cells with density 0 and rollback rate 0, and the other cells record `not_this_cell`.

The only verdict row (line 6932, copied in full):

```text
E7RESULT name=verdict segment=two quarantine_cost_negligible_holds=false span_bit_writes_nothing_extra_holds=false no_mark_false_red_persists_holds=false arms_agree_without_events=true scheduled_events_realized=true unreadable_counts=zero
```

| Field that did not hold | Why it has this value | Expected in registration? |
|---|---|---|
| `quarantine_cost_negligible_holds=false` | At density 10 A reaches at most band two (line 6777) and B band three (line 6849), the same under both bases | Registration did not state an expectation; the observed trigger matches the failure clause |
| `span_bit_writes_nothing_extra_holds=false` | Of the 240 cells with events, 200 have a positive per-cell mean for A's quarantining publication (`mean_positive` of the command counting means by arm and U in “Measurements: extra writes per quarantine and per rollback revival on segment-two workloads”: 82 + 118) | Yes: section ten says it must trigger on micro-histories and is reported as observed on workloads |
| `no_mark_false_red_persists_holds=false` | Line 6926 `final_value_zero=606`, all in cells with rollback rate 1 | Registration did not state an expectation |

### Segment-one artifact completeness, verdict row and fields that did not hold

By result-row type (command `for n in config stop_check … done; do grep -c "^E7RESULT name=$n " <产物>; done`, original output in executor report): `config` 1, `stop_check` 43 (13 code-reading + 30 source-verbatim checks), `clause_anchor` 7, `independent_anchor` 11, `micro` 25 (8 histories × 3 arms + `plain_release` 1), `micro_pin` 8, `positive_control` 12 (write count 1 + checker 10 + old reader 1), `pairwise_with_absolute` 1, `flip_classification` 120, `flip_classification_medium` 1, `constructed_pass_and_false_red` 10, `old_reader_arithmetic` 4, `old_reader_field_table` 5, `verdict` 1, `done` 1: total 250, matching `emitted=250`.

The only verdict row (line 249, copied in full):

```text
E7RESULT name=verdict segment=one stop_checks_hold=true clause_anchors_hold=true independent_anchors_hold=true micro_pins_hold=true positive_controls_hold=true revival_clears_the_mark=true old_readers_reject_the_mark_holds=false mark_hides_a_leak_holds=false mark_removes_the_false_red_holds=true span_bit_writes_nothing_extra_on_the_micro_histories_holds=false
```

| Field that did not hold | Why it has this value | Expected in registration? |
|---|---|---|
| `old_readers_reject_the_mark_holds=false` | Arithmetic half holds (0 records accepted with 0x4000), but in code the pool checker's non-judging pass uses span 16385 after the judging pass fails I-1.1 (block header states its logical address) (`walk.rs:5752`, `:6171`); recorded as hit under the weakest reading | Registration did not state an expectation; the observed trigger matches the failure clause |
| `mark_hides_a_leak_holds=false` | In the flipped-mark universe the four C combinations miss 2 / 4 defects, more than `no_mark`'s 0 (lines 233, 235, 237, 239) | Yes: sections four and ten say this is likely to trigger |
| `span_bit_writes_nothing_extra_on_the_micro_histories_holds=false` | A adds 4, 8, 6, 4, 4, 4 writes across the micro-histories (`micro_dirty_leaf` and `micro_release_quarantined` are 0) | Yes: section ten says it must trigger on micro-histories; for the workload half see “Segment-two artifact: rerun, completeness and verdict row” |

### Invalidation, failure and stop conditions, item by item (both segments)

Segment-one line numbers refer to `e187-quarantine-mark-cost-segment-one-2026-10-21.out`, segment-two line numbers to `e187-quarantine-mark-cost-segment-two-2026-10-22.out`.

| Condition | Outcome | Evidence |
|---|---|---|
| Invalidation `positive_controls_fail` | Not triggered | All 12 segment-one `name=positive_control` rows have `holds=true` |
| Invalidation `independent_anchor_fails` | Not triggered | All 11 segment-one `name=independent_anchor` and 8 `name=micro_pin` rows have `holds=true` |
| Invalidation `arms_agree_without_events`, `scheduled_events_realized` (segment two) | Not triggered | Segment-two line 6932 `arms_agree_without_events=true scheduled_events_realized=true`; the old segment-two artifact had `scheduled_events_realized=false` and is void as a whole, see “Segment-two artifact: rerun, completeness and verdict row” |
| Invalidation `product_incomplete` | Not triggered | Segment-one final `emitted=250` and segment-two final `emitted=6933` both match the actual row counts |
| Invalidation `unreadable_counts` | Not triggered | Deterministic model; segment-two line 6932 `unreadable_counts=zero` |
| Failure `clause_anchor_mismatch` | Not triggered | All 7 segment-one `name=clause_anchor` rows have `holds=true` |
| Failure `old_readers_reject_the_mark` | Hit (weakest interpretation) | See “Segment-one artifact completeness, verdict row and fields that did not hold” |
| Failure `mark_hides_a_leak` | Hit | Segment-one lines 233, 235, 237, 239; segment-two lines 6921 and 6925, where `mark_or_today` misses 4966, more than the 0 of `no_mark` on line 6917 (`mark_only` misses 0 on lines 6919 and 6923, not more than `no_mark`) |
| Failure `mark_removes_the_false_red` | Not triggered | Segment-one verdict `mark_removes_the_false_red_holds=true`; segment-two lines 6918, 6920, 6922, 6924 have 0 false failures, and both C placements have trajectory peak 0 |
| Failure `span_bit_writes_nothing_extra` | Hit (micro-histories and workloads) | Segment-one verdict; segment-two line 6932, with 200 of A's 240 cells with events having a positive mean |
| Failure `quarantine_cost_negligible` | Hit | Segment-two line 6777 A `highest_tier=2`, line 6849 B `highest_tier=3` |
| Failure `no_mark_false_red_persists` | Hit | Segment-two line 6926 `final_value_zero=606` |
| Stop (section eleven's line-by-line code checks and verbatim format-constant checks) | Not triggered | All 43 segment-one `name=stop_check` rows are `result=pass`; the two refinements recorded in revision one are handed to the main agent for confirmation |

### Unit tests and mutations

- 33 unit tests (`grep -c '#\[test\]' scripts/experiment-apparatus/e187_quarantine_mark_cost.rs` prints `33`), plus two in the thread registration slot module; after the scheduling fix `cargo test --release --bin e187-quarantine-mark-cost` passes: `test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s` (original output in the executor report). The 3 tests added since segment one: all 320 segment-two cells are checked cell by cell for event count, rollback count and pairwise disjoint rollback intervals; the two cells are run through `evaluate_cell` with `schedule_realized` asserted; the total device writes of `no_mark` at seed 1, U = 200, K = 8, `geometry_default` are pinned back to 453394 (registration section twelve, “value of the regression pin”; a regression pin, not an independent pin).
- Mutation table with 18 entries (the 17 of segment one plus `rollback_schedule_skips_the_second_draw`): caught 18, invalid 0, not red 0; `mutate.sh` ends with “计数：内存撞顶 0 条（上限 3G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、锚点失效 0 条、现判 4 条、复用 14 条、拒算 0 条” and “已还原，基线仍全绿” (4 judged afresh, 14 reused from the verdict library). All 18 mutations caught only shows that the code carrying assertions was mutation-tested, not that every place the conclusions rely on was tested.

### Fork table (fork sheet `research/prompts/m4-i315-quarantine-mark-cost-forks.md`, after segment two)

| One item in the decision-sufficiency conditions | Status | What remains / can remaining measurements reverse it? |
|---|---|---|
| Extra writes per quarantine (bytes, count) | Sufficient for decision | Single-event micro-history values (segment one) plus mean, min, max and the “leaf dirty anyway or not” grouping over 240 workload cells (segment two, `name=extra_writes`); A versus B changes sign with the sample point in both segments (segment-two line 6915 `reading=depends_on_sampling_point`), and the registration has no remaining measurement |
| Extra writes for rollback revival and clearing marks | Sufficient for decision | Single-event micro-history values (segment one) plus the 120 cells with rollback rate 1 (segment-two `revival_*` fields); releasing a quarantined record has only the micro-history value, and the registration states there is no density to sweep |
| Bit-flip classification table (outcome for each record class) | Sufficient for decision | 120 cells on constructed disk states (segment one); segment two gives final-state counts only and does not change any cell's judgment |
| Several sample points of quarantine-event density from zero to several per thousand | Sufficient for decision | 80 cells each at densities 0, 1, 3, 10 (segment-two lines 6723–6858, `name=share_tier_by_density`); the registration has no other density |
| Sample points where the two arms actually differ | Sufficient for decision | Each of the 240 cells with events has `events=` equal to 10 × density, greater than 0, and all 320 cells have `schedule_realized=true` |
| Old-reader interpretation of placement A checked field by field | Sufficient for decision | Five locations checked field by field, consistent with arithmetic (segment one) |
| (Reversal observation) misses and false failures of the three dispositions, and the `no_mark` false-failure trajectory | Sufficient for decision | Segment-two lines 6916–6930 |

The fork sheet row is decidable; the incidental quantity `tree_table_static_bytes` of registration section six was not run after the row became decidable (the registration says “incidental, not run once decidable”, and it does not correspond to any item of that row's sufficiency conditions).

### Measurement basis

- One device write = one unit written to one disk, or one journal record written to one disk. Bytes are summed over both disks (unit 16384 × slot count, journal record 4096); single-disk values are half. Root slots (512 bytes, one write per disk per publication) and system-configuration slots (4096, once per disk) are identical for all three arms and excluded from extra writes. Segment two reports shares under both bases.
- Micro-histories measure each arm minus `no_mark` for that publication; the same history is constructed separately for all three arms, identically except whether and where the mark is written at quarantine.
- Model placement rules were checked one by one against `crates/` (stop-condition code reading); two refinements against `crates/` are recorded in registration revision one.

### What it cannot answer

- The workload has no leaks (every segment-two row has `leak_units=0`): the miss of “a leak marked on both disks” exists only on the constructed disk states of segment one and is not counted in the final-state counts.
- The share's numerator is the whole-history difference, including the difference caused by placements and node ages diverging after the first quarantine (positive or negative), not only the write of the mark itself; for the mark's own write see the `per_event_*` column. Both are numbers of this counting model and were not run on `crates/`.
- Shares were measured only at two values of U, two values of K, two geometries, 10000 ordinary publications per history and 4 units overwritten per publication; other file sizes, publication counts and overwrite widths were not swept. The band already changes with U (“Measurements: segment-two shares and bands (by quarantine-event density)”), and which band other values would fall into is unknown.
- The classification table judges constructed disk states, not runs of the `crates/` checker. The C checker is a model of the strongest form acknowledged by C's proponents (registration section five), not present in `crates/`.
- A and B are cell-for-cell identical on the checker side, so the table cannot distinguish placements; they differ only in write volume and old-reader behavior.
- If the user decides `quarantine_intact` is not a valid disk state, reread the false-failure column accordingly; the judgments themselves remain unchanged.

### Path and conclusion registration

Conclusions “extra writes in the eight micro-histories equal the hand-derived values in registration section seven” and “what old readers do on records containing the borrowed bit”: obtain each quantity independently along two paths, then compare.

| # | Path | Calculation | Source location | Shared inputs read |
|---|---|---|---|---|
| 1 | Counting model | Construct each of three arms for each micro-history; run fixed point to count device writes and bytes for that publication; subtract `no_mark`. Eight enumerated points (registration section five construction), no domain sweep because pins were hand-derived only for these eight constructions | `scripts/experiment-apparatus/e187_quarantine_mark_cost.rs:2219 measure_micro_history` (fixed point at `:1564 plan_publish`) | Format constants (leaf 812, fanout 169, mapping leaf 294, journal 64 named entries per record); position-based allocation-record-tree writes; code-2 tree rules; placement rules (revision one) |
| 2 | Hand-derived pins | Work out the “micro-history pins” table in registration section seven from the constructions | No implementation: the section-seven hand calculation table; apparatus copies it into `MicroHistory::pinned_extra_writes` as expected values only | Same format constants and same “changed leaf rewrites ancestors and root” rule; an error affects both paths, so section eleven separately checks stop conditions against code and 30 format constants verbatim in `crates/` |
| 3 | Old-reader arithmetic | Apply old interpretation `span_field & !0x8000` and leaf-fit test at every legal start in a leaf; two borrowed bits × two spans | `scripts/experiment-apparatus/e187_quarantine_mark_cost.rs:1941 old_reader_arithmetic` | 812 leaf slots; version-one spans 1 and 2 |
| 4 | Old-reader code reading | Executor reads five old-reader sites in current workspace, recording fields and subsequent behavior | No implementation: code reading; conclusions copied into apparatus constant `OLD_READER_FIELD_TABLE` | Same format in which the span field's highest bit is the freed flag |

Cross-path comparison assertions: path 1 against path 2 is pinned in eight `micro_*` unit tests (`assert_the_micro_history_pins`) and artifact rows `name=micro_pin … holds=`. No assertion connects path 3 to path 4: the code-reading half is prose copied verbatim into the artifact, and the executor checked consistency site by site (see “old readers for placement A”); that assertion remains owed.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 5 | Supports | 2026-10-22 changed: the user chose placement A with a checker that recognizes only the mark; the item gains the on-disk isolation mark and the scope on old readers not getting past it, its evidence section now cites E187 (Cost of Persisting Quarantine Marks), and both sides land in the same change |
| D19 (Block Pointer Structure & Width Budget) settled item 8 | Background | 2026-10-22 unaffected: only the sentence exempting mapping tree, tree table and instance table was used to place B's quarantine-tree nodes in the mapping; its evidence section does not cite E187; segment two does not touch this sentence |
| D15 (Format Freeze Policy) settled item 1 | Background | 2026-10-22 unaffected: both C placements change the format, the judgment belongs to that decision, and this page measures cost only; segment two only adds cost numbers |
| D15 (Format Freeze Policy) settled item 2 | Background | 2026-10-22 unaffected: B falls under “add a keyspace” in the decision table based on the tree table's open list, which neither segment measured |
| D15 (Format Freeze Policy) settled item 9 | Background | 2026-10-22 unaffected: old-reader code reading places A under “change the meaning of an existing field”; the instance is an inference handed to the main agent; segment two has no old-reader measurement |
| D3 (Space Allocation) settled item 7 | Background | 2026-10-22 changed: the span segment row gains the sentence "second-highest bit 0x4000 borrowed as the isolation mark", pointing to D19 (Block Pointer Structure & Width Budget) settled item 5; pinned values (entry 20, span offset 10, 0x8000, version-one span 1 or 2) unchanged; segment two does not touch these items |
| D3 (Space Allocation) settled item 10 | Background | 2026-10-22 unaffected: model implements ② “not in any open clustered segment” as read in `crates/` (all segments opened in this mount count); revision one handed to the main agent for confirmation; segment-two workloads run under the same placement rules, and the decision text is unchanged |
| D3 (Space Allocation) settled item 11 | Background | 2026-10-22 unaffected: used only “span field in value, freed flag in highest bit”, which segment two does not touch |

## Historical Versions

Historical entries for E187 (Cost of Persisting Quarantine Marks) are collected in [experiments-history.md](../experiments-history.md).
