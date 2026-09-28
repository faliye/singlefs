This is a fact-table-filling task about a counting model of a copy-on-write file system's journal cost. Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings marked with asterisks or hashes. Give numbered answers. Do not write source-code line numbers or result-file line numbers in your answer; when you cite a fact, name its row id only (for example F3, G1, Q2). For every numbered answer, add one sentence stating what fact or observation would overturn that answer. All facts below are given as fixed data; treat every row as literally true and do not question it. Where a question is not settled by the given rows, say so explicitly rather than inventing a missing fact. Never write the exact same word of five or more letters twice in a row separated only by a single space (for example never write RISECONV RISECONV back to back); if a value would otherwise repeat immediately after itself, replace the second occurrence with the word same instead of writing the label again.

Background. Two publish arms are compared: arm intent (called 甲) writes every dirty leaf plus all its ancestors plus the root slot plus one record on every fsync. Arm wal_leaf (called 乙) only writes the leaf plus one record on every fsync, and defers ancestors and the root to a periodic checkpoint. A third arm called 丙 defers ancestor content differently (it names the ancestor's location in the record instead of writing the content); 丙 has never been measured by any experiment. A counting model computes, for a batch of consecutive fsyncs, the ratio of blocks written by intent divided by blocks written by wal_leaf, as a function of two inputs: batch size (how many fsyncs are grouped before the ratio is read) and stream count (how many independent, non-overlapping subtrees the batched operations touch). The model is exact block accounting, not measured time.

G1. Gate G23.2, from decisions/23-journal的角色与格式.md (file name for your own bookkeeping only, do not put a line number in your answer):
G1a. Gate G23.2 is named gap-between-arm-and-does-not-converge; what it asks is whether, as batch size grows, the gap between arm 甲 and arm 丙 does not converge.
G1b. Its current measured status is answered-for-乙, but bounded: in the E16 multistream workload, going from batch 1 to batch 10 the ratio grows from 3.40x to 3.71x — it does not converge, it widens instead.
G1c. This only holds while batch is less than or equal to stream count. A 56-cell scan (streams times batch, on the original geometry used at the time) shows the peak lands near batch approximately equal to streams; for the 16-stream row, by batch 50 the ratio has already dropped to 1.99, and by batch 200 to 1.24.
G1d. Warning: arm 丙 itself has never been measured.
G2. Arm 丙 likewise does not write ancestor content into the record; it is meaningful only if axis one (whether every fsync publishes the root) is reopened.

Q1. The applicability sentence being re-checked in this round: batching cannot absorb the benefit of deferring ancestor writes only while batch size is less than or equal to the concurrent stream count; once batch size exceeds the stream count, the ratio monotonically collapses toward 1.
Q2. A later check of that same sentence's two halves, on the real-baseline geometry (root_children=11, called G2 in the pre-registration, not to be confused with gate G23.2): batching-cannot-absorb-while-batch-is-less-than-or-equal-to-streams fails in 12 of the 21 stream-count rows of that scan — adjacent-batch drops exceeding a 1 percent tolerance, by 1.3 to 4.7 percent, mostly near batch=2. Ratio-monotonically-collapses-beyond-batch-equals-streams: both its sub-claims (no further rise, and settling within one quarter of the peak excess by batch = 16 times streams) hold in all 21 rows, but the phrase collapses toward 1 is imprecise — by batch=16384, 12 of the 21 rows (streams 2 to 32) already dip below 1, down to 0.9927-0.9968, not merely close to 1.
Q3. A newer finding (second segment of the same experiment): the stream count at which the interior peak disappears (the row's classification becomes NoInteriorPeak, called 丙行 in the source) shifts monotonically later as root_children (the number of children directly under the root node) increases: at root_children=2, all 21 scanned stream-count rows are already NoInteriorPeak; at root_children=11 (today's real baseline), NoInteriorPeak starts at streams=24; at root_children=17, it starts at streams=32; at root_children=147 (called G3 in the pre-registration), the near-diagonal classification (NearDiagonal, called 甲行) extends out to streams approximately 128, a wide-plateau classification (WidePlateau, called 宽平台) extends to streams=128, the moved-diagonal classification (MovedDiagonal, called 乙行) extends to streams=256, and only at streams=384 does the row flip to NoInteriorPeak — under the same tree height, raising root_children from 11 to 147 delays the NoInteriorPeak starting point from streams=24 to streams=384, a 16-fold delay.

Fact table. Source files, given once here, not to be repeated in your answer: FILE1 is research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out; FILE2 is research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out. Every row below is one name=row_summary line from one of those two files, at the fixed geometry and stream count named. peak_batch_low is the smallest batch value in that row's peak set (the batch at which the ratio first reaches its row maximum). settles_holds is the row's own settles_near_one_beyond_sixteen_streams_holds field: true means that by batch = 16 times streams, the ratio has come back down to within one quarter of the peak's excess over 1; false means it has not (the ratio is still far from 1, or has climbed back up, by that point on the batch axis). final_ratio is the ratio at the largest batch value scanned in that row (batch=16384 in every row below).

Definitions to apply mechanically, do not reinterpret them:
CONV1 applies when peak_batch_low = 1 and settles_holds = true (the ratio is already at its row maximum at batch=1, and comes back down near 1 as batch grows: it converges from batch=1 onward, with no rise phase).
RISECONV applies when peak_batch_low > 1 and settles_holds = true (the ratio rises from batch=1 to a peak at some batch greater than 1, then falls back down near 1: it rises, then converges).
NOCONV applies when settles_holds = false, regardless of peak_batch_low (the ratio has not come back down near 1 by batch = 16 times streams; it does not converge in the range scanned).

Geometry group four_gibibyte_file, root_children=7 (FILE1 line with name=geometry_config geometry=four_gibibyte_file):
F1 streams=2 peak_batch_low=1 settles_holds=true final_ratio=0.992561
F2 streams=16 peak_batch_low=1 settles_holds=true final_ratio=0.993638
F3 streams=24 peak_batch_low=1 settles_holds=true final_ratio=0.994137
F4 streams=256 peak_batch_low=1 settles_holds=true final_ratio=1.007890
F5 streams=384 peak_batch_low=1 settles_holds=true final_ratio=1.015503
F6 streams=1024 peak_batch_low=1 settles_holds=true final_ratio=1.040604

Geometry group one_tebibyte_file, root_children=11, the real baseline (FILE1 line with name=geometry_config geometry=one_tebibyte_file):
F7 streams=2 peak_batch_low=2 settles_holds=true final_ratio=0.992681
F8 streams=16 peak_batch_low=15 settles_holds=true final_ratio=0.994913
F9 streams=24 peak_batch_low=1 settles_holds=true final_ratio=0.995881
F10 streams=256 peak_batch_low=1 settles_holds=true final_ratio=1.023525
F11 streams=384 peak_batch_low=1 settles_holds=true final_ratio=1.038926
F12 streams=1024 peak_batch_low=1 settles_holds=true final_ratio=1.115299

Geometry group sixteen_tebibyte_file, root_children=2 (FILE1 line with name=geometry_config geometry=sixteen_tebibyte_file):
F13 streams=2 peak_batch_low=1 settles_holds=true final_ratio=0.992742
F14 streams=16 peak_batch_low=3 settles_holds=true final_ratio=0.995345
F15 streams=24 peak_batch_low=5 settles_holds=true final_ratio=0.996765
F16 streams=256 peak_batch_low=1 settles_holds=true final_ratio=1.033526
F17 streams=384 peak_batch_low=1 settles_holds=true final_ratio=1.048842
F18 streams=1024 peak_batch_low=1 settles_holds=true final_ratio=1.122286

Geometry group full_root_level_four, root_children=147, same tree height (node_levels=4) as one_tebibyte_file (FILE2 line with name=geometry_config geometry=full_root_level_four):
F19 streams=2 peak_batch_low=2 settles_holds=true final_ratio=0.992681
F20 streams=16 peak_batch_low=15 settles_holds=true final_ratio=0.995207
F21 streams=24 peak_batch_low=23 settles_holds=true final_ratio=0.996650
F22 streams=256 peak_batch_low=28 settles_holds=true final_ratio=1.031722
F23 streams=384 peak_batch_low=1 settles_holds=true final_ratio=1.047129
F24 streams=1024 peak_batch_low=1 settles_holds=true final_ratio=1.123477

Geometry group two_fifty_six_mebibyte_file, root_children=58 (FILE2 line with name=geometry_config geometry=two_fifty_six_mebibyte_file):
F25 streams=2 peak_batch_low=1 settles_holds=true final_ratio=1.969697
F26 streams=16 peak_batch_low=1 settles_holds=true final_ratio=1.970169
F27 streams=24 peak_batch_low=1 settles_holds=true final_ratio=1.973833
F28 streams=256 peak_batch_low=1 settles_holds=true final_ratio=1.977631
F29 streams=384 peak_batch_low=1 settles_holds=true final_ratio=2.008919
F30 streams=1024 peak_batch_low=1 settles_holds=false final_ratio=1.977631

Geometry group small_fanout_one_tebibyte_file, root_children=17, fanout=128 (same 1 TiB unit count as one_tebibyte_file) (FILE2 line with name=geometry_config geometry=small_fanout_one_tebibyte_file):
F31 streams=2 peak_batch_low=2 settles_holds=true final_ratio=0.993475
F32 streams=16 peak_batch_low=15 settles_holds=true final_ratio=0.996019
F33 streams=24 peak_batch_low=20 settles_holds=true final_ratio=0.997029
F34 streams=256 peak_batch_low=1 settles_holds=true final_ratio=1.024770
F35 streams=384 peak_batch_low=1 settles_holds=true final_ratio=1.040141
F36 streams=1024 peak_batch_low=1 settles_holds=true final_ratio=1.116417

Geometry group large_fanout_one_tebibyte_file, root_children=2, fanout=296 (same 1 TiB unit count as one_tebibyte_file) (FILE2 line with name=geometry_config geometry=large_fanout_one_tebibyte_file):
F37 streams=2 peak_batch_low=1 settles_holds=true final_ratio=0.989079
F38 streams=16 peak_batch_low=1 settles_holds=true final_ratio=0.990817
F39 streams=24 peak_batch_low=1 settles_holds=true final_ratio=0.991773
F40 streams=256 peak_batch_low=1 settles_holds=true final_ratio=1.019592
F41 streams=384 peak_batch_low=1 settles_holds=true final_ratio=1.034963
F42 streams=1024 peak_batch_low=1 settles_holds=true final_ratio=1.073209

Task. The grid to fill has one row per (geometry, stream-count segment) pair: 7 geometries times 3 segments equals 21 rows. Segment LOW is represented by the two boundary fact rows at streams=2 and streams=16 for that geometry. Segment MID is represented by the two boundary fact rows at streams=24 and streams=256. Segment HIGH is represented by the two boundary fact rows at streams=384 and streams=1024.

For each of the 21 (geometry, segment) rows, do the following:
1. Apply the CONV1 / RISECONV / NOCONV definitions to the low-boundary fact row of that segment and separately to the high-boundary fact row of that segment.
2. State both results. If they are the same category, write that category name once for low-boundary and write the word same (not the category name again) for high-boundary. If they differ, state both, labeled low-boundary and high-boundary, and say the segment is mixed rather than picking one.
3. Name the fact row ids you used (for example F7, F8).
4. State the effect on gate G23.2's current answered-for-乙-but-bounded status for that cell: support, oppose, or irrelevant, with one sentence of reasoning. Consider explicitly that G23.2's own text (G1a) is about arm 甲 versus arm 丙, that arm 丙 has never been measured (G1d), and that every fact row in this prompt is arm intent versus arm wal_leaf (甲 versus 乙), not arm 丙 — decide whether that gap between what G23.2 asks and what was measured changes your support/oppose/irrelevant answer, and say so.
5. State one observation that would overturn your answer for that cell.

Give your 21 answers as a table with columns: geometry, segment, low-boundary category, high-boundary category (write the word same here instead of repeating the category name when it matches low-boundary), same-or-mixed, fact row ids used, effect on G23.2 (support/oppose/irrelevant), what would refute this row. Do not put two adjacent cells next to each other with no separator other than a space if they would hold the identical word; use the pipe character between every pair of adjacent cells.

After the table, answer these three additional questions, numbered 22, 23, 24:

22. Across all 42 fact rows, list every row id where the category is NOCONV. For each one, state in one sentence why NOCONV contradicts or does not contradict the word bounded in G1b.

23. Compare F9, F10, F11 (streams=24, 256, 384 on the real baseline, root_children=11) against the specific numbers quoted in G1c (batch 50 ratio 1.99 and batch 200 ratio 1.24, both for the 16-stream row). Note that G1c's numbers are for streams=16, which is fact row F8's stream count, not F9/F10/F11's. State whether F8 alone is enough to check G1c's two numbers, or whether checking them requires data not present in this fact table (name what would be needed).

24. Given G2's statement that arm 丙 is only meaningful if axis one is reopened, and given that this whole fact table is about arm 甲 versus arm 乙, state in one or two sentences whether any answer in your 21-row table above can, on its own, change gate G23.2's status from answered-for-乙 to answered-for-丙 — or whether that would require a different experiment entirely. Give the observation that would overturn your answer.
