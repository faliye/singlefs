## E176 Capacity and Write Cost of Three Inode Leaf Merge Candidates — run (two segments)

[中文](176-inode树叶合并三候选的容量与写代价.md) · **English** · [日本語](176-inode-葉のマージ3候補の容量と書き込みコスト.ja.md)

<!-- doc-lint:not-numbers M1 M2 M3 Q1 Q2 Q3 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15 V16 F1 F2 F3 F4 F5 F6 F7 -->

```text
Local arm names and notation on this page are lowercase (a1, a2, a3, a4 …). Preregistration, apparatus, artifacts and commands use the corresponding uppercase forms, which look like globally registered identifiers. Lowercase here only distinguishes the local names; use uppercase when reading artifacts.
```

Preregistration `e176-preregistration.md` (sha256 `f7599076bb469dcc483f98cb0f557aca5bc6d79759fe674b4b52f150b8291fa9` when segment two finished) includes three section-12 amendments: amendment 1 adds positive controls and a common-window interpretation for M3 f=59; amendment 2 moves the differential binary from an optional feature to a standalone manifest; amendment 3 adds three segment-two common windows, positive controls for three variants and mutations V14–V16, before the segment-two artifact. This answers row M of `m3-3-5-forks.md`: how to reclaim sparse inode-tree leaf containers, the merging half of C116 (Leaf container split / merge discipline lacks check that can fail) and C536 (inode-tree “borrow right into left” makes the container-number condition false).

The three candidates are M1, remove only empty leaves (the `crates/` behavior when E176 ran; `crates/` has since moved to M3); M2, left absorbs right only (merge adjacent leaves totaling at most 233 records); and M3, D8 (Core Index Structure) settled item 6 as written, left absorbs right plus left borrows from right (main sample floor f=117, borrow to balance). The independent hand-written model `scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs` shares no code with `crates/`. The M1 arm is additionally compared step by step against the two pure functions in the then-current `crates/singlefs-core/src/inode_tree.rs` by `scripts/experiment-apparatus/e176_crates_oracle/oracle.rs`; that comparison has not been redone since `crates/` switched to M3.

**Status:**

- Segment one completed: 22 sequences (W-R random insert/delete, p=0.55/0.75/0.95 with five seeds each; W-S sequential deletion at three points; W-A delete alternating IDs at four points) × four arm variants (M1, M2, M3, M3 f=59). Every section-6 row has verdicts at the main samples in all three families; each section-8 knob marked segment one has a point with the opposite direction. The question sheet's M-row sufficiency condition is met.
- Overall main-window verdicts (seven `name=verdict scope=row` lines in `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out`): Q1r says “merging buys multiple times the capacity” (all five W-R p=0.55 R12 values about 0.35); both Q2p positional rows Δ2p and Δ3p say “no flip,” due only to W-R p=0.75 seed 2 (0.1046 and 0.1100 just above 0.10); both content-count Q2c and both Q3 rows say “negligible.” The f=59 window's six verdicts match the main window (lines 770–775).
- Sensitivity: Q1r is workload-unstable. W-R p=0.95, all three W-S points and W-A retaining 3/4 have R12≥0.90 (“small cost”); W-A retaining 1/233 has 0.0711 (“multiple times”). Both Q2p rows are unstable across the insertion-probability knob (0.55 vs 0.75, seed 2).
- Segment two ran because a segment-one Q2/Q3 result lay within 0.05 of a threshold: both Q2p rows at W-R p=0.75 across five seeds ranged 0.0724–0.1100. The main agent requested it. The same 22 sequences add M3 borrowing just to the floor (`M3_just_enough`), M2 checking left first (`M2_left_first`) and M3 checking left first (`M3_left_first`). Common windows: `just_enough` = M1/M2/M3_just_enough; `left_first` = M1/M2_left_first/M3_left_first (amendment 3). The twelve row verdicts in `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` say:
  - All six left-first verdicts match the main window: Q2p rows “no flip” (W-R p=0.75 seed 2 Δ2p=0.1046, Δ3p=0.1111), and Q2c/Q3 rows “negligible.”
  - In the just-enough window both Q2p rows remain “no flip”; Q2c Δ3c and both Q3 rows change from “negligible” to “no flip.” Δ3c is 0.5018 at two W-S sequential-delete points and 0.4568 at W-A retaining 1/233, all below 1.00. No row is “significant” in any window, and Q2p is never “negligible.”
  - The amount-borrowed knob changes the verdict in 15 pairs, all from main-window “negligible” to just-enough “no flip”: Δ3p at W-R p=0.75 seeds 1, 4, 5, one pair each; four M3-related rows at two ascending W-S deletion points and W-A retaining 1/233, four pairs each. The neighbor-check-order knob gives the same verdict in all 132 pairs, including 99 bit-for-bit equal values for which moving the threshold cannot prove discrimination.
- No stop clause hit on either segment: the differential binary checks all 22 sequences stepwise until M1 rejects with zero differences, and local constants 233/135 match `crates/`. Segment two rebuilt the comparison with changed source and got a byte-identical artifact. No invalidation hit: segment-one class-B anchors match 26/26, segment two 29/29; all operation-stream prefix fingerprints agree across the three arms in 44 and 66 common windows respectively; both segments ran twice, at eight and one thread, with identical sha256. No class-C assertion failed (any failure would panic immediately without the final line). The main window and M1/M2/M3 arms' 396 segment-two rows are verbatim identical to segment one, and rerunning segment one with updated source produced the committed artifact byte for byte.

This is a deterministic model: two byte-identical runs show no hidden state or thread-count effect, not statistical stability. Evidence strength comes from absolute-value anchors, positive controls, independent differential and mutation tests.

### What was done

- **Model:** The leaf table stores container IDs in leaf order and record IDs ascending within each leaf. Insertion follows `crates/` at the time: append to the rightmost non-full leaf, otherwise split a new leaf at the end with container ID equal to the new record ID; reject and leave the tree unchanged if leaf count would exceed 135. Deletion removes the record and removes an empty leaf in all three arms. M2 then checks the new adjacent pair after removal, or both neighbors of the changed leaf if it remains; whenever their combined size is ≤233, left absorbs right, repeated until no pair fits. If M3 made no merge, the leaf remains nonempty with count <f and has a right neighbor, borrow its smallest ⌊(|R|−|X|)÷2⌋ records, then check the right neighbor and its right neighbor under M2's rule.
- **Two rewrite counts:** positional counts changed leaves plus every surviving leaf at or right of the first removed position (matching the rewrite list in `inode_tree.rs` `remove_record_from_leaf_containers` when written); content counts only changed leaves that remain after publication. One unlink counts only the final batch publication.
- **Stepwise lockstep:** four arm variants of a sequence advance together, each with its own generator and operation-stream SHA-256. A common window ends immediately before the first rejected operation by any of its three arms. Main window uses M1/M2/M3; f=59 window M1/M2/M3 f=59. Class-C assertions after each publication check ascending records, 1–233 records per leaf, container ID ≤ minimum record ID, strictly increasing container IDs in leaf order, nonoverlapping adjacent key ranges; I-9.4's third sentence is asserted for M1/M2 and counted for M3; adjacent sizes sum to ≥234 for M2/M3; content-count rewrites per operation ≤1 for M1/M2 and ≤2 for M3.
- **Workloads:** W-R uses splitmix64 with seed as initial state. Each step draws a 53-bit uniform u; insert if the live set is empty or u×100 < insertion percentage ×2^53 (exact decimal integer comparison), otherwise delete index next() mod length using swap_remove. W-S inserts 1000 then deletes D smallest or largest in each round. W-A inserts A consecutive IDs and deletes the registered fraction in ascending ID order. Each arm is limited to 2×10⁷ operations; all 88 segment-one arms reached rejection before the cap.
- **Differential:** Every step before M1 rejection feeds model M1 and `crates/` `write_records_into_leaf_containers` / `remove_record_from_leaf_containers`, comparing leaf count, each leaf's container ID/count/min/max, each positional rewrite-list item, content count, removed leaves, rejected step and cell. The binary imports the same model source with `#[path]` and builds with its own `Cargo.toml`, a separate workspace with read-only path dependencies on `singlefs-core` and `singlefs-format`. It is not an optional feature of `scripts/experiment-apparatus/Cargo.toml`: optional path dependencies still require Cargo to read manifests under `../../crates`, whereas `scripts/mutation/mutate.sh` copies only `research/` into a temporary directory, which would break the baseline for every mutation table (amendment 2, discovered on the first mutation run).
- **Discriminative power of the differential (§9):** Enable V1 in a draft copy of the model (M1 fails to record right-hand leaves in positional count when removing an empty leaf). The comparison reports the first difference on W-S's main point at operation 1233, as registered. Done once in each segment, byte-identical outputs; see comparison section.
- **Segment two (§5 and amendment 3):** The same model adds the `segment-two` entry, feeding each of 22 sequences to six variants and using `main`, `just_enough`, `left_first` common windows. M3_just_enough borrows f−|X| records (exactly to 117); M2_left_first and M3_left_first check left/X before X/right after deletion, retaining main floor and borrowing amount. The borrowing-amount knob compares four M3-related rows in main vs just_enough; neighbor-order compares all six in main vs left_first. Q1r is not judged on these knobs (section 6 fixes M2 at its main setting); M2_left_first live count at refusal is reported only.

### Replay

```
# Repository root; model in scripts/experiment-apparatus/ does not depend on crates/. SINGLEFS_THREAD_CAP affects elapsed time, not output
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
bash ../hosts/capped.sh 8 cargo build --release --bin e176-inode-leaf-merge-capacity-and-rewrites
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-capacity-and-rewrites segment-one
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-capacity-and-rewrites segment-two
# Registered replay: segment-one and segment-two entries
cd ../.. && bash scripts/experiment-apparatus/replay.sh E176
# Comparison (standalone manifest, read-only crates/ path dependencies; replay.sh does not build it)
cd scripts/experiment-apparatus
bash ../hosts/capped.sh 8 cargo build --release --manifest-path e176_crates_oracle/Cargo.toml --target-dir target
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-crates-oracle
# 22 unit tests and 16 mutations
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 cargo test --release --bin e176-inode-leaf-merge-capacity-and-rewrites
MUTATE_JOBS=4 MUTATE_CARGO_JOBS=2 MUTATE_MEMORY_MAX=4G bash ../mutation/mutate.sh e176-inode-leaf-merge-capacity-and-rewrites e176_inode_leaf_merge_capacity_and_rewrites.rs ../mutation/tables/e176_inode_leaf_merge_capacity_and_rewrites.tsv
```

Segment one took about 16 s at eight threads and 23 s at one; segment two about 19 s and 25 s respectively, including memory wrapper; comparison about 23 s at eight threads. Artifacts:

| File | Contents |
|---|---|
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` | Segment one: config, anchors/positive controls, 22 sequences × four variants, overall verdicts and sensitivity; 991 lines, sha256 `fe3a322978d570c3a6db180ac6e10d00a9e744c9b16b403f4e49105ef5f58dc4`. The two runs, with eight threads and one thread respectively, were identical byte for byte. One more run with the source changed in segment two was still identical byte for byte. |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` | Segment two: config, anchors/positive controls (47 cells, 29 registered answers), 22 sequences × six variants × three windows, overall verdicts, two-knob sensitivity; 1370 lines, sha256 `8906bfd7d87a516abbe61c74c410dfc00d9c967b57fbad916f08f14c21e60737`. Eight- and one-thread runs identical. |
| `e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out` | The differential for segment one: compare constants back against their source, compare the three shapes in the `crates/` unit tests, and compare each of the 22 sequences step by step until M1 is refused; 28 lines, sha256 `c0a6635e1cab890f1b9dc134df1fb43e53adc2ba2c993f59bbaef981f8c69ae3`. Identical byte for byte to the run made in the draft directory before the manifest was changed. |
| `e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out` | Segment-two comparison rebuilt from segment-two source, byte-identical to segment one, same sha256. |
| `e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out` | Discriminative power of the segment-one differential: the differential output after enabling V1 in the model, built on a copy in the draft directory; the verdict line is deliberately false |
| `e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out` | Discriminative power of the differential for segment two, with V1 enabled in the segment-two source: the artifact is identical byte for byte to the one from segment one; sha256 `29e6ab1ed77318939f8d377b7c90728c7a4028a24771d0e8aad080c71f909bb3`. |
| `e176-inode-leaf-merge-capacity-and-rewrites-unit-tests.out` | Segment-one unit tests, 14. |
| `e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out` | Segment-two unit tests, 22. |
| `e176-inode-leaf-merge-capacity-and-rewrites-mutations.out` | Segment-one complete 13-row mutation table output, stderr header first. |
| `e176-inode-leaf-merge-capacity-and-rewrites-mutations-2026-10-08.out` | Segment-two complete 16-row mutation table output from segment-two source, stderr header first. |

### Verdicts (full lines copied from artifacts)

All verdict lines listed by `grep -n 'name=verdict'`:

```
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:33:E7RESULT name=verdict scope=positive_controls anchors_registered=26 anchors_matching=26 all_registered_anchors_match=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:760:E7RESULT name=verdict scope=window_prefixes windows=44 windows_with_equal_prefixes=44 all_window_prefixes_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:761:E7RESULT name=verdict scope=f6_m1_reached points=22 m1_not_reached_points=0 m1_reached_on_every_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:762:E7RESULT name=verdict scope=f5_m3_borrows_on_main_points main_point_m3_borrows=5930 m3_borrowed_on_some_main_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:763:E7RESULT name=verdict scope=row row=Q1r window=main points=22 judged_points=22 verdict=merge_buys_double_capacity
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:764:E7RESULT name=verdict scope=row row=Q2p_delta2 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:765:E7RESULT name=verdict scope=row row=Q2p_delta3 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:766:E7RESULT name=verdict scope=row row=Q2c_delta2 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:767:E7RESULT name=verdict scope=row row=Q2c_delta3 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:768:E7RESULT name=verdict scope=row row=Q3p_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:769:E7RESULT name=verdict scope=row row=Q3c_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:770:E7RESULT name=verdict scope=row row=Q2p_delta2 window=floor59 points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:771:E7RESULT name=verdict scope=row row=Q2p_delta3 window=floor59 points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:772:E7RESULT name=verdict scope=row row=Q2c_delta2 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:773:E7RESULT name=verdict scope=row row=Q2c_delta3 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:774:E7RESULT name=verdict scope=row row=Q3p_delta32 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:775:E7RESULT name=verdict scope=row row=Q3c_delta32 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:990:E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=200 unstable_pairs=14 moved_to_unstable=162 equal_values_not_separable=38 every_separable_same_pair_moved_to_unstable=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
```

The first two artifacts' verdict lines contain no false or not_run fields; violation, mismatch, ambiguity and failure integer counts are zero. The third is the discriminative power self-test: `all_equal=false` and `crates_unit_test_shapes_equal=false` are expected because V1 is enabled and comparison must report a difference. `verdict=no_flip` is not a failure; it is section 6's “otherwise ⇒ no flip” category.

All verdict lines for segment two's comparison, discriminability check and artifact:

```
research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:49:E7RESULT name=verdict scope=positive_controls anchors_registered=29 anchors_matching=29 all_registered_anchors_match=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1128:E7RESULT name=verdict scope=window_prefixes windows=66 windows_with_equal_prefixes=66 all_window_prefixes_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1129:E7RESULT name=verdict scope=f6_m1_reached points=22 m1_not_reached_points=0 m1_reached_on_every_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1130:E7RESULT name=verdict scope=row row=Q1r window=main points=22 judged_points=22 verdict=merge_buys_double_capacity
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1131:E7RESULT name=verdict scope=row row=Q2p_delta2 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1132:E7RESULT name=verdict scope=row row=Q2p_delta3 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1133:E7RESULT name=verdict scope=row row=Q2c_delta2 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1134:E7RESULT name=verdict scope=row row=Q2c_delta3 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1135:E7RESULT name=verdict scope=row row=Q3p_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1136:E7RESULT name=verdict scope=row row=Q3c_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1137:E7RESULT name=verdict scope=row row=Q2p_delta2 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1138:E7RESULT name=verdict scope=row row=Q2p_delta3 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1139:E7RESULT name=verdict scope=row row=Q2c_delta2 window=just_enough points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1140:E7RESULT name=verdict scope=row row=Q2c_delta3 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1141:E7RESULT name=verdict scope=row row=Q3p_delta32 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1142:E7RESULT name=verdict scope=row row=Q3c_delta32 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1143:E7RESULT name=verdict scope=row row=Q2p_delta2 window=left_first points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1144:E7RESULT name=verdict scope=row row=Q2p_delta3 window=left_first points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1145:E7RESULT name=verdict scope=row row=Q2c_delta2 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1146:E7RESULT name=verdict scope=row row=Q2c_delta3 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1147:E7RESULT name=verdict scope=row row=Q3p_delta32 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1148:E7RESULT name=verdict scope=row row=Q3c_delta32 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1369:E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=205 unstable_pairs=15 moved_to_unstable=74 equal_values_not_separable=131 every_separable_same_pair_moved_to_unstable=true
```

The segment-two comparison has no false fields; the two false fields in its discriminability check are expected for the same reason as segment one. The segment-two artifact has no false or not_run verdict fields and `m1_not_reached_points=0`. Main-window verdicts at lines 1130–1136 match segment one's 763–769. `equal_values_not_separable=131` is not a failure: it counts pairs for which moving the threshold cannot prove discrimination because the two values are exactly equal. Section 8 did not specify this case, so it is reported separately rather than merged into another count.

### Anchors and positive controls (all 31 `name=anchor` lines from segment one)

All section-7 class-A/B cells plus the amended f=59 variant (a2–a6 report only, `matches=unregistered`). The space-amplification cell (unlimited M1 fills 10⁶ then deletes 90%) at line 29 has `ratio=9.98`, matching the “9.98 times” clause at the end of D8 item 6; F3 did not hit. F1 positive controls pass: a2 M2 live count 31455 > M1's 367, a3 M3 borrows once, a5 M2 merges once.

```
E7RESULT name=anchor cell=A1 arm=M1 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M2 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3_floor59 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A2 arm=M1 refused_insert=31688 live=367 containers=135 merges=0 borrows=0 matches=true
E7RESULT name=anchor cell=A2 arm=M2 refused_insert=62776 live=31455 containers=135 merges=134 borrows=0 matches=true
E7RESULT name=anchor cell=A2 arm=M3 refused_insert=62776 live=31455 containers=135 merges=134 borrows=918 matches=true
E7RESULT name=anchor cell=A2 arm=M3_floor59 refused_insert=62776 live=31455 containers=135 merges=134 borrows=58 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M1 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=true
E7RESULT name=anchor cell=A3 arm=M2 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=true
E7RESULT name=anchor cell=A3 arm=M3 content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176 matches=true
E7RESULT name=anchor cell=A3_boundary arm=M3 leaf0_largest=291 leaf1_smallest=292 matches=true
E7RESULT name=anchor cell=A3 arm=M3_floor59 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M1 content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M2 content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M3 content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M3_floor59 content=234 position=236 containers=3 merges=1 borrows=1 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M1 content=234 position=234 containers=5 merges=0 borrows=0 third_sentence_false=0 sizes=233/116/116/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M2 content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M3 content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M3_floor59 content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M1 content=233 position=233 containers=2 merges=0 borrows=0 third_sentence_false=0 sizes=116/117 matches=true
E7RESULT name=anchor cell=A6 arm=M2 content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=true
E7RESULT name=anchor cell=A6 arm=M3 content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233 matches=true
E7RESULT name=anchor cell=A6 arm=M3_floor59 content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M3_just_enough content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=117/233 matches=true
E7RESULT name=anchor cell=A5 arm=M2_left_first content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=C1 arm=M1_unlimited leaves=4292 live=100000 ideal=430 ratio=9.98 matches=true
E7RESULT name=anchor cell=crates_insert_500 arm=M1 sizes=233/233/34 container_numbers=1/234/467 matches=true
E7RESULT name=anchor cell=crates_insert_31456 arm=M1 refused_inode=31456 refused_containers=136 capacity=135 matches=true
E7RESULT name=anchor cell=crates_drop_middle arm=M1 position_rewritten=[1] dropped=1 matches=true
```

### Readings: Q1 (live records at refusal and R12 = L_M1 ÷ L_M2)

Interpretation 1: when an arm first rejects an insertion, count records in the last accepted version. All 88 arms reached rejection (line 761 `m1_not_reached_points=0`; M2/M3 also `stop=refused`), so none is merely an upper-bound point. Columns: sequence, L_M1, L_M2, L_M3 (Q1b, supplementary), L_M3 f=59, R12, verdict. Command from repository root and output:

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} live[v["sequence"],v["arm"]]=v["live_at_stop"]} /name=point_verdict/ && / window=main / && / row=Q1r /{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} print v["sequence"], live[v["sequence"],"M1"], live[v["sequence"],"M2"], live[v["sequence"],"M3"], live[v["sequence"],"M3_floor59"], v["value"], v["verdict"]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
```

```
wr_p055_seed1 8070 22452 22116 22452 0.3594 merge_buys_double_capacity
wr_p055_seed2 8177 22422 21909 21995 0.3647 merge_buys_double_capacity
wr_p055_seed3 7978 22190 21867 22190 0.3595 merge_buys_double_capacity
wr_p055_seed4 7933 22823 21820 22771 0.3476 merge_buys_double_capacity
wr_p055_seed5 8083 22396 22260 21912 0.3609 merge_buys_double_capacity
wr_p075_seed1 21360 24167 23551 24167 0.8838 no_flip
wr_p075_seed2 21173 24427 24264 24427 0.8668 no_flip
wr_p075_seed3 21378 24225 24225 24225 0.8825 no_flip
wr_p075_seed4 20994 24299 23980 24299 0.8640 no_flip
wr_p075_seed5 21157 24325 24163 24325 0.8698 no_flip
wr_p095_seed1 29865 29865 29865 29865 1.0000 no_merge_capacity_cost_small
wr_p095_seed2 29810 29811 29811 29811 1.0000 no_merge_capacity_cost_small
wr_p095_seed3 29802 29803 29803 29803 1.0000 no_merge_capacity_cost_small
wr_p095_seed4 29815 29815 29815 29815 1.0000 no_merge_capacity_cost_small
wr_p095_seed5 29821 29821 29821 29821 1.0000 no_merge_capacity_cost_small
ws_a1000_d500_smallest_first 31245 31245 31245 31245 1.0000 no_merge_capacity_cost_small
ws_a1000_d950_smallest_first 31319 31319 31284 31278 1.0000 no_merge_capacity_cost_small
ws_a1000_d500_largest_first 31455 31455 31455 31455 1.0000 no_merge_capacity_cost_small
wa_keep1of2_a1000 17132 27054 25464 27054 0.6333 no_flip
wa_keep1of2_a2330 16773 31455 31455 31455 0.5332 no_flip
wa_keep1of233_a2330 2236 31455 31081 31081 0.0711 merge_buys_double_capacity
wa_keep3of4_a1000 24606 25478 25478 25478 0.9658 no_merge_capacity_cost_small
```

Q1r's overall verdict is “merging buys multiple times the capacity”: all five W-R p=0.55 main-sample R12 values are 0.3476–0.3647, below 0.50. The unfavorable half for M2/M3 is also present: all three W-S points, five W-R p=0.95 seeds and W-A retaining 3/4 have R12≥0.90 (small cost without merging). M1 and M2 have identical live counts at refusal on all three W-S points: sequential deletion empties whole leaves, which M1 also removes. W-A retaining 1/2, the main sample, is 0.6333, “no flip.” M3's live count at refusal is below M2 for 12 sequences, equal for 10 and greater for none (W-R p=0.55: 21820–22260 versus 22190–22823): borrowing bought no additional capacity on these workloads. Counting command and output:

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} live[v["sequence"],v["arm"]]=v["live_at_stop"]; seq[v["sequence"]]=1} END{less=0; equal=0; more=0; for(s in seq){ if (live[s,"M3"]+0 < live[s,"M2"]+0) less++; else if (live[s,"M3"]+0 == live[s,"M2"]+0) equal++; else more++ } print "m3_less_than_m2=" less, "equal=" equal, "m3_more_than_m2=" more}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
m3_less_than_m2=12 equal=10 m3_more_than_m2=0
```

### Readings: Q2 and Q3 (difference in mean pages rewritten per unlink in the main window)

Δ = (arm A sum − arm B sum) ÷ number of final batches in the window. The smallest of the 44 windows has 1643 final batches (minimum extracted from `name=window` `unlinks`), so no sample is insufficient. Negative means that arm rewrites fewer pages than M1. Command and output:

```
awk '/name=point_verdict/ && / window=main / && !/ row=Q1r /{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} out[v["sequence"]]=out[v["sequence"]] " " v["row"] "=" v["value"]; if(!(v["sequence"] in seen)){seen[v["sequence"]]=1; order[++n]=v["sequence"]}} END{for(i=1;i<=n;i++) print order[i] out[order[i]]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
```

```
wr_p055_seed1 Q2p_delta2=-0.0260 Q2p_delta3=-0.0157 Q2c_delta2=0.0012 Q2c_delta3=0.0111 Q3p_delta32=0.0103 Q3c_delta32=0.0099
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=-0.0301 Q2c_delta2=0.0013 Q2c_delta3=0.0115 Q3p_delta32=0.0092 Q3c_delta32=0.0102
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=-0.0131 Q2c_delta2=0.0012 Q2c_delta3=0.0111 Q3p_delta32=0.0099 Q3c_delta32=0.0099
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=-0.0076 Q2c_delta2=0.0013 Q2c_delta3=0.0118 Q3p_delta32=0.0097 Q3c_delta32=0.0105
wr_p055_seed5 Q2p_delta2=-0.0219 Q2p_delta3=-0.0128 Q2c_delta2=0.0013 Q2c_delta3=0.0108 Q3p_delta32=0.0091 Q3c_delta32=0.0095
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.0724 Q2c_delta2=0.0000 Q2c_delta3=0.0047 Q3p_delta32=-0.0158 Q3c_delta32=0.0047
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1100 Q2c_delta2=0.0000 Q2c_delta3=0.0038 Q3p_delta32=0.0054 Q3c_delta32=0.0038
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0822 Q2c_delta2=0.0000 Q2c_delta3=0.0040 Q3p_delta32=0.0026 Q3c_delta32=0.0040
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.0786 Q2c_delta2=0.0000 Q2c_delta3=0.0041 Q3p_delta32=-0.0199 Q3c_delta32=0.0041
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.0936 Q2c_delta2=0.0000 Q2c_delta3=0.0046 Q3p_delta32=0.0001 Q3c_delta32=0.0046
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0299 Q2c_delta2=0.0000 Q2c_delta3=0.0342 Q3p_delta32=0.0299 Q3c_delta32=0.0342
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0300 Q2c_delta2=0.0000 Q2c_delta3=0.0343 Q3p_delta32=0.0300 Q3c_delta32=0.0343
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0055 Q2c_delta2=0.0000 Q2c_delta3=0.0016 Q3p_delta32=0.0004 Q3c_delta32=0.0016
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.0457 Q2c_delta2=0.0004 Q2c_delta3=0.0275 Q3p_delta32=0.0265 Q3c_delta32=0.0271
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

Keep the two accounting conventions separate:

- **Positional (the `crates/` convention at the time):** both Δ2p and Δ3p overall verdicts are “no flip.” Both are ≤0.10 at the main samples; all five W-R p=0.55 seeds are negative (−0.0393 to −0.0076). When M1 removes an empty leaf, all surviving leaves to its right shift position and are rewritten, up to 133 in one operation; M2 merges earlier so fewer leaves remain on the right at removal. The unfavorable half for “merging is negligible”: W-R p=0.75 Δ2p=0.0796–0.1046 and Δ3p=0.0724–0.1100, both slightly above 0.10 for seed 2. M1 removed only the rightmost leaf in that family (`position_largest=1`); M2/M3 merges moved leaves to the right, up to 90–105 in one operation.
- **Content:** Δ2c and Δ3c overall are “negligible.” Largest Δ3c values are W-S's 0.0342/0.0343 and W-A retaining 1/233 at 0.0275; maximum Δ2c is 0.0013.
- **Q3 (M3 − M2):** both conventions “negligible,” maximum 0.0343 (W-S delete 950 in order, content). Borrowing actually occurred at main points (`main_point_m3_borrows=5930`, F5); averaged over unlinks it is less than 0.035 pages.

Rewrite distributions at main-sample windows, copied verbatim; five histogram buckets count 0, 1, 2, 3–9 and 10–134 pages:

```
39:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M1 unlinks=35448 content_sum=35406 position_sum=38633 content_mean=0.9988 position_mean=1.0898 content_largest=1 position_largest=127 content_histogram=42/35406/0/0/0 position_histogram=1/35406/0/0/41
40:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M2 unlinks=35448 content_sum=35447 position_sum=37712 content_mean=1.0000 position_mean=1.0639 content_largest=1 position_largest=46 content_histogram=1/35447/0/0/0 position_histogram=1/35320/1/25/101
41:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M3 unlinks=35448 content_sum=35798 position_sum=38076 content_mean=1.0099 position_mean=1.0741 content_largest=2 position_largest=48 content_histogram=1/35096/351/0/0 position_histogram=1/34971/351/24/101
72:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M1 unlinks=37793 content_sum=37743 position_sum=41605 content_mean=0.9987 position_mean=1.1009 content_largest=1 position_largest=130 content_histogram=50/37743/0/0/0 position_histogram=1/37743/0/2/47
73:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M2 unlinks=37793 content_sum=37792 position_sum=40119 content_mean=1.0000 position_mean=1.0615 content_largest=1 position_largest=45 content_histogram=1/37792/0/0/0 position_histogram=1/37657/3/27/105
74:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M3 unlinks=37793 content_sum=38176 position_sum=40467 content_mean=1.0101 position_mean=1.0708 content_largest=2 position_largest=45 content_histogram=1/37408/384/0/0 position_histogram=1/37278/385/28/101
105:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M1 unlinks=36583 content_sum=36535 position_sum=39784 content_mean=0.9987 position_mean=1.0875 content_largest=1 position_largest=133 content_histogram=48/36535/0/0/0 position_histogram=3/36535/0/0/45
106:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M2 unlinks=36583 content_sum=36580 position_sum=38942 content_mean=0.9999 position_mean=1.0645 content_largest=1 position_largest=48 content_histogram=3/36580/0/0/0 position_histogram=3/36447/2/28/103
107:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M3 unlinks=36583 content_sum=36942 position_sum=39303 content_mean=1.0098 position_mean=1.0744 content_largest=2 position_largest=49 content_histogram=3/36218/362/0/0 position_histogram=3/36088/363/27/102
138:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M1 unlinks=37574 content_sum=37524 position_sum=40519 content_mean=0.9987 position_mean=1.0784 content_largest=1 position_largest=131 content_histogram=50/37524/0/0/0 position_histogram=2/37524/0/1/47
139:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M2 unlinks=37574 content_sum=37572 position_sum=39871 content_mean=0.9999 position_mean=1.0611 content_largest=1 position_largest=46 content_histogram=2/37572/0/0/0 position_histogram=2/37436/3/31/102
140:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M3 unlinks=37574 content_sum=37966 position_sum=40235 content_mean=1.0104 position_mean=1.0708 content_largest=2 position_largest=44 content_histogram=2/37178/394/0/0 position_histogram=2/37045/395/32/100
171:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M1 unlinks=37279 content_sum=37228 position_sum=40474 content_mean=0.9986 position_mean=1.0857 content_largest=1 position_largest=127 content_histogram=51/37228/0/0/0 position_histogram=4/37228/0/1/46
172:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M2 unlinks=37279 content_sum=37275 position_sum=39658 content_mean=0.9999 position_mean=1.0638 content_largest=1 position_largest=47 content_histogram=4/37275/0/0/0 position_histogram=4/37139/3/28/105
173:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M3 unlinks=37279 content_sum=37630 position_sum=39997 content_mean=1.0094 position_mean=1.0729 content_largest=2 position_largest=47 content_histogram=4/36920/355/0/0 position_histogram=4/36788/356/27/104
534:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M1 unlinks=30500 content_sum=30370 position_sum=39229 content_mean=0.9957 position_mean=1.2862 content_largest=1 position_largest=133 content_histogram=130/30370/0/0/0 position_histogram=0/30370/0/6/124
535:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M2 unlinks=30500 content_sum=30370 position_sum=39229 content_mean=0.9957 position_mean=1.2862 content_largest=1 position_largest=133 content_histogram=130/30370/0/0/0 position_histogram=0/30370/0/6/124
536:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M3 unlinks=30500 content_sum=31413 position_sum=40142 content_mean=1.0299 position_mean=1.3161 content_largest=2 position_largest=133 content_histogram=0/29587/913/0/0 position_histogram=0/29457/913/6/124
633:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M1 unlinks=16500 content_sum=16500 position_sum=16500 content_mean=1.0000 position_mean=1.0000 content_largest=1 position_largest=1 content_histogram=0/16500/0/0/0 position_histogram=0/16500/0/0/0
634:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M2 unlinks=16500 content_sum=16500 position_sum=16583 content_mean=1.0000 position_mean=1.0050 content_largest=1 position_largest=4 content_histogram=0/16500/0/0/0 position_histogram=0/16467/0/33/0
635:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M3 unlinks=16500 content_sum=16526 position_sum=16590 content_mean=1.0016 position_mean=1.0055 content_largest=2 position_largest=4 content_histogram=0/16474/26/0/0 position_histogram=0/16441/38/21/0
```

### Trajectories (section 8)

The predicate consumes the leaf-container page count (reject if >135). Each arm reports peak pages, publication count at 135 and final pages; minimum and final-before-refusal fill; and, for M3, peak number of adjacent pairs violating the third sentence of I-9.4 (container number not above the smallest key), publications with a positive count and final count. Minimum fill is 0.0043 (=1÷233) in all 88 arms, at initial inode creation. This only describes the starting point, not sparse deletion; use final fill for the latter. Main-sample trajectory rows:

```
34:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=78967 operations_accepted=78966 live_at_stop=8070 live_over_31455=0.2566 containers_peak=135 publishes_at_135=651 containers_final=135 fill_smallest=0.0043 fill_final=0.2566 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=fe4304e70876af50f0fa91b8f2c75791c72d36710fdc199dddfffe13d732a1f6
35:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=230361 operations_accepted=230360 live_at_stop=22452 live_over_31455=0.7138 containers_peak=135 publishes_at_135=961 containers_final=135 fill_smallest=0.0043 fill_final=0.7138 merges=429 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=d71ba0162292de7d49f990dd26d0a8a0ca1a8264ebf95e4f3c6f7f38339e3711
36:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=226987 operations_accepted=226986 live_at_stop=22116 live_over_31455=0.7031 containers_peak=135 publishes_at_135=774 containers_final=135 fill_smallest=0.0043 fill_final=0.7031 merges=427 borrows=1020 i94_third_false_peak=45 i94_third_false_positive_publishes=224603 i94_third_false_final=41 consumed_prefix_sha256=068dd9dcc4c0fcf20ceeeba91e5e38383cc4e6cb4a34b17db8b8d9e8c6bfb60b
37:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=230361 operations_accepted=230360 live_at_stop=22452 live_over_31455=0.7138 containers_peak=135 publishes_at_135=961 containers_final=135 fill_smallest=0.0043 fill_final=0.7138 merges=429 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=d71ba0162292de7d49f990dd26d0a8a0ca1a8264ebf95e4f3c6f7f38339e3711
67:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=83764 operations_accepted=83763 live_at_stop=8177 live_over_31455=0.2600 containers_peak=135 publishes_at_135=414 containers_final=135 fill_smallest=0.0043 fill_final=0.2600 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=5afa3f6043a5318a7b34d32dfb6f6026d7cef5496eb8025df9220a2318f62fb7
68:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=231163 operations_accepted=231162 live_at_stop=22422 live_over_31455=0.7128 containers_peak=135 publishes_at_135=869 containers_final=135 fill_smallest=0.0043 fill_final=0.7128 merges=422 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=583feaff08554eee9716e557803ce97d7cf45dcb040d06b8c32d311c5bbd29c3
69:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=226290 operations_accepted=226289 live_at_stop=21909 live_over_31455=0.6965 containers_peak=135 publishes_at_135=609 containers_final=135 fill_smallest=0.0043 fill_final=0.6965 merges=430 borrows=1029 i94_third_false_peak=44 i94_third_false_positive_publishes=221873 i94_third_false_final=40 consumed_prefix_sha256=f05ae2f3871ea2821bd23aa97983f8503deb14b6ba96a2e97415ba0a6785dc09
70:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=227128 operations_accepted=227127 live_at_stop=21995 live_over_31455=0.6993 containers_peak=135 publishes_at_135=418 containers_final=135 fill_smallest=0.0043 fill_final=0.6993 merges=434 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=481 i94_third_false_final=0 consumed_prefix_sha256=828525d844500cf89c5b0d5c924022bad7ae3605871882afed2fd3bff2a23f01
100:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=81145 operations_accepted=81144 live_at_stop=7978 live_over_31455=0.2536 containers_peak=135 publishes_at_135=860 containers_final=135 fill_smallest=0.0043 fill_final=0.2536 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=fa40f6fa0318543033abd9c7fb1815326ad7f6e44f60abb0f39d5b89e6e4c056
101:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=226565 operations_accepted=226564 live_at_stop=22190 live_over_31455=0.7055 containers_peak=135 publishes_at_135=517 containers_final=135 fill_smallest=0.0043 fill_final=0.7055 merges=406 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=761e85f85e52f879b3ddfac416f02a96e4f142cbea8fb99e92ae78fcc6aed648
102:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=223438 operations_accepted=223437 live_at_stop=21867 live_over_31455=0.6952 containers_peak=135 publishes_at_135=473 containers_final=135 fill_smallest=0.0043 fill_final=0.6952 merges=406 borrows=950 i94_third_false_peak=39 i94_third_false_positive_publishes=219845 i94_third_false_final=36 consumed_prefix_sha256=79d49208dbd51d8c5a97528d345a18f1dc75948794454ee27c3ffb98820663bf
103:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=226565 operations_accepted=226564 live_at_stop=22190 live_over_31455=0.7055 containers_peak=135 publishes_at_135=517 containers_final=135 fill_smallest=0.0043 fill_final=0.7055 merges=406 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=761e85f85e52f879b3ddfac416f02a96e4f142cbea8fb99e92ae78fcc6aed648
133:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=83082 operations_accepted=83081 live_at_stop=7933 live_over_31455=0.2522 containers_peak=135 publishes_at_135=672 containers_final=135 fill_smallest=0.0043 fill_final=0.2522 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=b1cb5e13c8eb2872837027dd4e6efced911dbd51ee052f0db59e4d2af111a6c3
134:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=229232 operations_accepted=229231 live_at_stop=22823 live_over_31455=0.7256 containers_peak=135 publishes_at_135=936 containers_final=135 fill_smallest=0.0043 fill_final=0.7256 merges=408 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=799af22c99b01dad0a9069d884d81e6fff284b13b3e22814ccb768912e660b65
135:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=220027 operations_accepted=220026 live_at_stop=21820 live_over_31455=0.6937 containers_peak=135 publishes_at_135=453 containers_final=135 fill_smallest=0.0043 fill_final=0.6937 merges=395 borrows=981 i94_third_false_peak=43 i94_third_false_positive_publishes=215205 i94_third_false_final=42 consumed_prefix_sha256=b07328935fa974782873a50ce2936b459454b7acc1da713b58cfa434a8af04ea
136:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=228432 operations_accepted=228431 live_at_stop=22771 live_over_31455=0.7239 containers_peak=135 publishes_at_135=402 containers_final=135 fill_smallest=0.0043 fill_final=0.7239 merges=408 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=347 i94_third_false_final=0 consumed_prefix_sha256=d90c6c10aeb892b21485fa66808ae4f65c172cc475802204e88d7e881205c0cb
166:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=82642 operations_accepted=82641 live_at_stop=8083 live_over_31455=0.2570 containers_peak=135 publishes_at_135=411 containers_final=135 fill_smallest=0.0043 fill_final=0.2570 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0a565b850da4ed7e69f6217e4518839373fad02f0a377d28380a93eed1138447
167:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=224797 operations_accepted=224796 live_at_stop=22396 live_over_31455=0.7120 containers_peak=135 publishes_at_135=408 containers_final=135 fill_smallest=0.0043 fill_final=0.7120 merges=407 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=16eb9a1342ae8c1dada946306ff2bd89c0746725dbffec13a982ac78576fcf56
168:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=223717 operations_accepted=223716 live_at_stop=22260 live_over_31455=0.7077 containers_peak=135 publishes_at_135=696 containers_final=135 fill_smallest=0.0043 fill_final=0.7077 merges=399 borrows=1001 i94_third_false_peak=42 i94_third_false_positive_publishes=220737 i94_third_false_final=38 consumed_prefix_sha256=dd7c20e7d9320424e738ef15abe21786782c370a3fee1dce44f36e49d6b01301
169:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=220315 operations_accepted=220314 live_at_stop=21912 live_over_31455=0.6966 containers_peak=135 publishes_at_135=411 containers_final=135 fill_smallest=0.0043 fill_final=0.6966 merges=388 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=574 i94_third_false_final=0 consumed_prefix_sha256=beba41c2b4e574d2fb2cb0b69d3bec5bbe75bb30d0a377f771f132484466afe1
529:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M1 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
530:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M2 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
531:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M3 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=130 borrows=913 i94_third_false_peak=1 i94_third_false_positive_publishes=44919 i94_third_false_final=1 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
532:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=130 borrows=131 i94_third_false_peak=1 i94_third_false_positive_publishes=22321 i94_third_false_final=1 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
628:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M1 stop=refused refused_operation=50133 operations_accepted=50132 live_at_stop=17132 live_over_31455=0.5447 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.5447 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=254c551cd054829735b28c412d02a379d6a0645758e962936459248375911f66
629:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M2 stop=refused refused_operation=80055 operations_accepted=80054 live_at_stop=27054 live_over_31455=0.8601 containers_peak=135 publishes_at_135=528 containers_final=135 fill_smallest=0.0043 fill_final=0.8601 merges=106 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0146ed25b25641ce21fcb425ea93c20cf1918917204687cd3a9548be1fdfaf20
630:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M3 stop=refused refused_operation=74465 operations_accepted=74464 live_at_stop=25464 live_over_31455=0.8095 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.8095 merges=81 borrows=36 i94_third_false_peak=19 i94_third_false_positive_publishes=71807 i94_third_false_final=18 consumed_prefix_sha256=2e2bb7b45ee7a2dd79d0d34d754b9252faa3222ad1addb3a58b0f9d714d352e9
631:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=80055 operations_accepted=80054 live_at_stop=27054 live_over_31455=0.8601 containers_peak=135 publishes_at_135=528 containers_final=135 fill_smallest=0.0043 fill_final=0.8601 merges=106 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0146ed25b25641ce21fcb425ea93c20cf1918917204687cd3a9548be1fdfaf20
```

The `fill_smallest` field groups to `88 fill_smallest=0.0043`.

Cumulative rewrite difference S(t)=Σ(arm−M1), with t traversing final batches in the main window; main samples, complete rows below. Under positional counting at W-R p=0.55, final values are negative but peaks positive: S first rises, then M1's large leaf-removal rewrites (up to 133 pages each) pull it below zero. Reporting only the final value misses the earlier interval; the precise positions of removals within the window were not recorded.

```
42:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M2-M1 notation=position peak=122 positive_single_differences=127 final=-921
43:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M2-M1 notation=content peak=41 positive_single_differences=41 final=41
44:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M3-M1 notation=position peak=297 positive_single_differences=476 final=-557
45:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M3-M1 notation=content peak=392 positive_single_differences=392 final=392
75:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M2-M1 notation=position peak=192 positive_single_differences=135 final=-1486
76:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M2-M1 notation=content peak=49 positive_single_differences=49 final=49
77:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M3-M1 notation=position peak=450 positive_single_differences=514 final=-1138
78:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M3-M1 notation=content peak=433 positive_single_differences=433 final=433
108:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M2-M1 notation=position peak=56 positive_single_differences=133 final=-842
109:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M2-M1 notation=content peak=45 positive_single_differences=45 final=45
110:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M3-M1 notation=position peak=163 positive_single_differences=492 final=-481
111:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M3-M1 notation=content peak=407 positive_single_differences=407 final=407
141:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M2-M1 notation=position peak=25 positive_single_differences=135 final=-648
142:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M2-M1 notation=content peak=48 positive_single_differences=48 final=48
143:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M3-M1 notation=position peak=168 positive_single_differences=527 final=-284
144:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M3-M1 notation=content peak=442 positive_single_differences=442 final=442
174:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M2-M1 notation=position peak=39 positive_single_differences=136 final=-816
175:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M2-M1 notation=content peak=47 positive_single_differences=47 final=47
176:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M3-M1 notation=position peak=198 positive_single_differences=486 final=-477
177:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M3-M1 notation=content peak=402 positive_single_differences=401 final=402
537:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M2-M1 notation=position peak=0 positive_single_differences=0 final=0
538:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M2-M1 notation=content peak=0 positive_single_differences=0 final=0
539:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M3-M1 notation=position peak=913 positive_single_differences=913 final=913
540:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M3-M1 notation=content peak=1043 positive_single_differences=1043 final=1043
636:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M2-M1 notation=position peak=83 positive_single_differences=33 final=83
637:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M2-M1 notation=content peak=0 positive_single_differences=0 final=0
638:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M3-M1 notation=position peak=90 positive_single_differences=59 final=90
639:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M3-M1 notation=content peak=26 positive_single_differences=26 final=26
```

### Geometry / knob sensitivity (section 8)

For each knob compare the main or reference point against its opposite once per each of seven rows: `two_points_same` for equal verdicts, `unstable` otherwise. Workload points pair by seed for W-R (including p=0.75); retaining 1/233 exists only at A=2330, so use W-A retaining 1/2 at A=2330 as reference. Compare main vs f=59 windows for the four M3-related rows in all 22 sequences. For notation, compare Q2p/Q2c and Q3p/Q3c at seven main points. Self-test of discriminative power requires moving a boundary between two same-verdict values to make verdicts differ. Grouped by knob; fourth column is moved result, “-” means already different and no proof attempted:

```
awk '/name=sensitivity/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} key=v["knob"] " " v["result"] " " (v["moved_result"]==""?"-":v["moved_result"]); count[key]++} END{for(k in count) print count[k], k}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out | sort -k2,2 -k3,3
```

```
7 batch_size_2330 two_points_same unstable
32 borrow_floor_59 two_points_same equal_values_cannot_be_separated
56 borrow_floor_59 two_points_same unstable
2 deleted_fraction_0.95 two_points_same equal_values_cannot_be_separated
5 deleted_fraction_0.95 two_points_same unstable
3 deletion_order_largest_first two_points_same equal_values_cannot_be_separated
4 deletion_order_largest_first two_points_same unstable
28 insert_probability_0.75 two_points_same unstable
7 insert_probability_0.75 unstable -
30 insert_probability_0.95 two_points_same unstable
5 insert_probability_0.95 unstable -
6 kept_fraction_1of233 two_points_same unstable
1 kept_fraction_1of233 unstable -
6 kept_fraction_3of4 two_points_same unstable
1 kept_fraction_3of4 unstable -
1 rewrite_notation_content two_points_same equal_values_cannot_be_separated
20 rewrite_notation_content two_points_same unstable
```

Of 200 same-verdict pairs, 162 become different when the boundary moves. For 38 the values are exactly equal and the boundary cannot move between them (`equal_values_not_separable=38`, line 990); section 8 did not specify reporting this case, so it is separated: 32 on f knob (eight sequences × four rows; W-R p=0.95 five seeds, W-S descending, W-A retaining 1/2 at A=2330, and W-A retaining 3/4 all show zero borrows in M3 and M3 f=59), five on the two W-S knobs and one on notation (both values 0.0000 or 1.0000). These 38 establish only equal point values, not discrimination.

All 14 pairs with different verdicts:

```
776:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed1 main_value=0.3594 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed1 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
783:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed2 main_value=0.3647 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed2 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
790:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed3 main_value=0.3595 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed3 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
797:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed4 main_value=0.3476 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed4 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
804:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed5 main_value=0.3609 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed5 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
811:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed1 main_value=0.3594 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed1 opposite_value=0.8838 opposite_verdict=no_flip result=unstable
818:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed2 main_value=0.3647 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed2 opposite_value=0.8668 opposite_verdict=no_flip result=unstable
819:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q2p_delta2 main=wr_p055_seed2 main_value=-0.0393 main_verdict=negligible opposite=wr_p075_seed2 opposite_value=0.1046 opposite_verdict=no_flip result=unstable
820:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q2p_delta3 main=wr_p055_seed2 main_value=-0.0301 main_verdict=negligible opposite=wr_p075_seed2 opposite_value=0.1100 opposite_verdict=no_flip result=unstable
825:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed3 main_value=0.3595 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed3 opposite_value=0.8825 opposite_verdict=no_flip result=unstable
832:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed4 main_value=0.3476 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed4 opposite_value=0.8640 opposite_verdict=no_flip result=unstable
839:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed5 main_value=0.3609 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed5 opposite_value=0.8698 opposite_verdict=no_flip result=unstable
860:E7RESULT name=sensitivity knob=kept_fraction_3of4 row=Q1r main=wa_keep1of2_a1000 main_value=0.6333 main_verdict=no_flip opposite=wa_keep3of4_a1000 opposite_value=0.9658 opposite_verdict=no_merge_capacity_cost_small result=unstable
867:E7RESULT name=sensitivity knob=kept_fraction_1of233 row=Q1r main=wa_keep1of2_a2330 main_value=0.5332 main_verdict=no_flip opposite=wa_keep1of233_a2330 opposite_value=0.0711 opposite_verdict=merge_buys_double_capacity result=unstable
```

Q1r is unstable for insertion probability (0.55 vs 0.95 and 0.55 vs 0.75, all five seeds in both comparisons) and retained fraction (1/2 vs 3/4 and 1/2 vs 1/233). Both Q2p rows are unstable only at seed 2 for insertion probability 0.55 vs 0.75. Q2c and both Q3 conventions agree at both points on every knob.

### Segment two: anchors and positive controls

Segment-two artifact lines 2–32 are verbatim identical to segment one's 2–32 (diff has no output). Lines 33–48 are 16 new cells for three variants: amendment 3 judges a1 against registered answers, reports the rest without judging.

```
E7RESULT name=anchor cell=A1 arm=M3_just_enough refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M2_left_first refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3_left_first refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A2 arm=M3_just_enough refused_insert=62776 live=31455 containers=135 merges=134 borrows=15197 matches=unregistered
E7RESULT name=anchor cell=A2 arm=M2_left_first refused_insert=62776 live=31455 containers=135 merges=134 borrows=0 matches=unregistered
E7RESULT name=anchor cell=A2 arm=M3_left_first refused_insert=62776 live=31455 containers=135 merges=134 borrows=918 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M2_left_first content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M3_left_first content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M3_just_enough content=349 position=351 containers=3 merges=1 borrows=116 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M2_left_first content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M3_left_first content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M3_just_enough content=350 position=352 containers=4 merges=1 borrows=116 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M3_left_first content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M3_just_enough content=349 position=349 containers=1 merges=1 borrows=116 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M2_left_first content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M3_left_first content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233 matches=unregistered
```

Left-first results on the registered hand-built a2–a6 cells equal the main knob cell by cell for M2 and M3; those cells cannot discriminate the knob. The unit test with three leaves sized 117/117/117 does discriminate (amendment 3 item 5; mutation V14 makes it fail). Just-enough borrows 116 times on a4–a6 while the main knob borrows seven. Inference, not observed stepwise: borrowing only to 117 means the next deletion falls below the floor and causes another borrow.

### Segment-two readings: Q2/Q3 across three windows

The set of numbers for the main window is identical word for word to the segment-one table in “Readings: Q2, Q3” (the 396 main-window rows in the segment-two artifact are identical to the same-named rows from segment one). For the other two windows, here are the command, run from the repository root with `W` set to `just_enough` or `left_first`, and its unmodified output:

```
awk -v W=just_enough '/name=point_verdict/ && $0 ~ (" window=" W " ") {delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} out[v["sequence"]]=out[v["sequence"]] " " v["row"] "=" v["value"]; if(!(v["sequence"] in seen)){seen[v["sequence"]]=1; order[++n]=v["sequence"]}} END{for(i=1;i<=n;i++) print order[i] out[order[i]]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out
```

`just_enough` (M1/M2/M3_just_enough):

```
wr_p055_seed1 Q2p_delta2=-0.0260 Q2p_delta3=0.0177 Q2c_delta2=0.0012 Q2c_delta3=0.0454 Q3p_delta32=0.0437 Q3c_delta32=0.0442
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=0.0255 Q2c_delta2=0.0013 Q2c_delta3=0.0661 Q3p_delta32=0.0648 Q3c_delta32=0.0648
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=0.0381 Q2c_delta2=0.0012 Q2c_delta3=0.0616 Q3p_delta32=0.0611 Q3c_delta32=0.0604
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=0.0377 Q2c_delta2=0.0013 Q2c_delta3=0.0567 Q3p_delta32=0.0550 Q3c_delta32=0.0554
wr_p055_seed5 Q2p_delta2=-0.0219 Q2p_delta3=0.0376 Q2c_delta2=0.0013 Q2c_delta3=0.0637 Q3p_delta32=0.0595 Q3c_delta32=0.0624
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.1106 Q2c_delta2=0.0000 Q2c_delta3=0.0192 Q3p_delta32=0.0225 Q3c_delta32=0.0192
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1282 Q2c_delta2=0.0000 Q2c_delta3=0.0216 Q3p_delta32=0.0236 Q3c_delta32=0.0216
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0957 Q2c_delta2=0.0000 Q2c_delta3=0.0141 Q3p_delta32=0.0161 Q3c_delta32=0.0141
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.1096 Q2c_delta2=0.0000 Q2c_delta3=0.0186 Q3p_delta32=0.0111 Q3c_delta32=0.0186
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.1068 Q2c_delta2=0.0000 Q2c_delta3=0.0266 Q3p_delta32=0.0133 Q3c_delta32=0.0266
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.4975 Q2c_delta2=0.0000 Q2c_delta3=0.5018 Q3p_delta32=0.4975 Q3c_delta32=0.5018
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.4975 Q2c_delta2=0.0000 Q2c_delta3=0.5018 Q3p_delta32=0.4975 Q3c_delta32=0.5018
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0053 Q2c_delta2=0.0000 Q2c_delta3=0.0002 Q3p_delta32=0.0002 Q3c_delta32=0.0002
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.4749 Q2c_delta2=0.0004 Q2c_delta3=0.4568 Q3p_delta32=0.4558 Q3c_delta32=0.4564
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

`left_first` (M1/M2_left_first/M3_left_first):

```
wr_p055_seed1 Q2p_delta2=-0.0248 Q2p_delta3=-0.0147 Q2c_delta2=0.0012 Q2c_delta3=0.0113 Q3p_delta32=0.0101 Q3c_delta32=0.0101
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=-0.0288 Q2c_delta2=0.0013 Q2c_delta3=0.0108 Q3p_delta32=0.0105 Q3c_delta32=0.0096
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=-0.0128 Q2c_delta2=0.0012 Q2c_delta3=0.0108 Q3p_delta32=0.0102 Q3c_delta32=0.0096
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=-0.0091 Q2c_delta2=0.0013 Q2c_delta3=0.0116 Q3p_delta32=0.0081 Q3c_delta32=0.0103
wr_p055_seed5 Q2p_delta2=-0.0233 Q2p_delta3=-0.0134 Q2c_delta2=0.0013 Q2c_delta3=0.0106 Q3p_delta32=0.0100 Q3c_delta32=0.0093
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.0864 Q2c_delta2=0.0000 Q2c_delta3=0.0045 Q3p_delta32=-0.0017 Q3c_delta32=0.0045
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1111 Q2c_delta2=0.0000 Q2c_delta3=0.0042 Q3p_delta32=0.0066 Q3c_delta32=0.0042
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0822 Q2c_delta2=0.0000 Q2c_delta3=0.0040 Q3p_delta32=0.0026 Q3c_delta32=0.0040
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.0875 Q2c_delta2=0.0000 Q2c_delta3=0.0041 Q3p_delta32=-0.0110 Q3c_delta32=0.0041
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.0936 Q2c_delta2=0.0000 Q2c_delta3=0.0046 Q3p_delta32=0.0001 Q3c_delta32=0.0046
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0299 Q2c_delta2=0.0000 Q2c_delta3=0.0342 Q3p_delta32=0.0299 Q3c_delta32=0.0342
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0300 Q2c_delta2=0.0000 Q2c_delta3=0.0343 Q3p_delta32=0.0300 Q3c_delta32=0.0343
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0055 Q2c_delta2=0.0000 Q2c_delta3=0.0016 Q3p_delta32=0.0004 Q3c_delta32=0.0016
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.0457 Q2c_delta2=0.0004 Q2c_delta3=0.0275 Q3p_delta32=0.0265 Q3c_delta32=0.0271
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

Write one conclusion for each of the two accounting conventions and each of the two knobs; do not combine them into a single sentence:

- **Left-first positional:** Δ2p/Δ3p remain “no flip”; W-R p=0.75 seed 2 has 0.1046/0.1111, above 0.10. W-R p=0.55 main samples remain negative (−0.0393 to −0.0091). Largest absolute change for either row is 0.014 (p=0.75 seed 1 Δ3p 0.0724→0.0864), with no verdict change.
- **Left-first content and Q3:** all four remain “negligible”; maximum 0.0343 at W-S ascending delete 950, same as main.
- **Just-enough positional:** Δ3p remains “no flip,” but points above 0.10 increase from one to four (W-R p=0.75 seeds 1/2/4/5: 0.1106/0.1282/0.1096/0.1068). Two W-S ascending points reach 0.4975, W-A retaining 1/233 reaches 0.4749, and W-R p=0.55 main points 0.0177–0.0381. None reaches 1.00.
- **Just-enough content and Q3:** Δ3c and both Q3 rows move from “negligible” to “no flip.” Unfavorable to negligible borrowing cost: W-S main point ascending delete 500 gives Δ3c=0.5018 and Q3 positional/content 0.4975/0.5018; W-A retaining 1/233 gives 0.4558–0.4568. None reaches 1.00, so no point hits “significant.”
- Δ2p and Δ2c are identical cell by cell in just_enough and main because they compare the same M1/M2 arms.

Trajectories and borrow counts: live records at refusal for each arm, borrow counts for three M3 variants and merge counts for two M2 variants. Command and output:

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} s=v["sequence"]; if(!(s in seen)){seen[s]=1; order[++n]=s} live[s,v["arm"]]=v["live_at_stop"]; br[s,v["arm"]]=v["borrows"]; mg[s,v["arm"]]=v["merges"]} END{print "sequence M1 M2 M3 M3_just_enough M2_left_first M3_left_first | borrows M3 M3_just_enough M3_left_first | merges M2 M2_left_first"; for(i=1;i<=n;i++){s=order[i]; print s, live[s,"M1"], live[s,"M2"], live[s,"M3"], live[s,"M3_just_enough"], live[s,"M2_left_first"], live[s,"M3_left_first"], "|", br[s,"M3"], br[s,"M3_just_enough"], br[s,"M3_left_first"], "|", mg[s,"M2"], mg[s,"M2_left_first"]}}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out
```

```
sequence M1 M2 M3 M3_just_enough M2_left_first M3_left_first | borrows M3 M3_just_enough M3_left_first | merges M2 M2_left_first
wr_p055_seed1 8070 22452 22116 22391 22416 22163 | 1020 6212 1024 | 429 428
wr_p055_seed2 8177 22422 21909 22306 22493 21909 | 1029 6774 989 | 422 424
wr_p055_seed3 7978 22190 21867 22277 22293 21757 | 950 6298 931 | 406 409
wr_p055_seed4 7933 22823 21820 22317 22823 22308 | 981 6273 1012 | 408 408
wr_p055_seed5 8083 22396 22260 22096 22396 21704 | 1001 6615 944 | 407 407
wr_p075_seed1 21360 24167 23551 24167 24167 24011 | 53 222 52 | 20 20
wr_p075_seed2 21173 24427 24264 24427 24427 24264 | 42 245 51 | 22 22
wr_p075_seed3 21378 24225 24225 24388 24225 24225 | 50 172 50 | 22 22
wr_p075_seed4 20994 24299 23980 24299 24299 23980 | 56 249 51 | 23 23
wr_p075_seed5 21157 24325 24163 24325 24325 24163 | 53 322 53 | 23 23
wr_p095_seed1 29865 29865 29865 29865 29865 29865 | 0 0 0 | 0 0
wr_p095_seed2 29810 29811 29811 29811 29811 29811 | 0 0 0 | 1 1
wr_p095_seed3 29802 29803 29803 29803 29803 29803 | 0 0 0 | 1 1
wr_p095_seed4 29815 29815 29815 29815 29815 29815 | 0 0 0 | 0 0
wr_p095_seed5 29821 29821 29821 29821 29821 29821 | 0 0 0 | 0 0
ws_a1000_d500_smallest_first 31245 31245 31245 31245 31245 31245 | 913 15174 913 | 0 0
ws_a1000_d950_smallest_first 31319 31319 31284 31275 31319 31284 | 17277 286396 17277 | 4 4
ws_a1000_d500_largest_first 31455 31455 31455 31455 31455 31455 | 0 0 0 | 0 0
wa_keep1of2_a1000 17132 27054 25464 27054 27054 25464 | 36 4 36 | 106 106
wa_keep1of2_a2330 16773 31455 31455 31455 31455 31455 | 0 0 0 | 130 130
wa_keep1of233_a2330 2236 31455 31081 31081 31455 31081 | 179568 2951927 179568 | 29004 29004
wa_keep3of4_a1000 24606 25478 25478 25478 25478 25478 | 0 0 0 | 33 33
```

All 132 arms reached refusal. At the W-S main sample just-enough borrows 15174 times versus 913 with the main knob. Its content histogram in that window has 0 pages zero times, 1 page 15326 times and 2 pages 15174 times (line 809):

```
809:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=just_enough arm=M3_just_enough unlinks=30500 content_sum=45674 position_sum=54403 content_mean=1.4975 position_mean=1.7837 content_largest=2 position_largest=133 content_histogram=0/15326/15174/0/0 position_histogram=0/15196/15174/6/124
```

M2_left_first live count at refusal is report-only; §6 does not judge Q1r on this knob. W-R p=0.55 gives 22293–22823, same order of magnitude as main M2's 22190–22823.

### Segment two: sensitivity of two knobs

Counts by knob (at repo root, output as-is; the third column is verdict comparison, the fourth column is after moving the boundary, where `-` indicates the two points were already different originally and require no self-validation):

```
awk '/name=sensitivity/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} key=v["knob"] " " v["result"] " " (v["moved_result"]==""?"-":v["moved_result"]); count[key]++} END{for(k in count) print count[k], k}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out | sort -k2,2 -k3,3
```

```
32 borrow_amount_just_enough two_points_same equal_values_cannot_be_separated
41 borrow_amount_just_enough two_points_same unstable
15 borrow_amount_just_enough unstable -
33 check_order_left_first two_points_same unstable
99 check_order_left_first two_points_same equal_values_cannot_be_separated
```

Of the 205 same-verdict pairs, 74 pairs flip to different verdicts after moving the boundary; the count of pairs where moving the boundary cannot provide self-validation is 131 (where values at the two points are bitwise identical; line 1369 `equal_values_not_separable=131`, listed separately, not merged into other counts). Counted by sequence (`awk` reading sensitivity lines where `moved_result=equal_values_cannot_be_separated`, counting by knob and main point sequence name): "how much to borrow" accounts for 32 pairs = 8 sequences × 4 rows, exactly the 8 sequences where neither M3 nor M3_just_enough borrowed even once (five seeds of W-R p = 0.95, W-S descending, W-A retaining 1/2 A = 2330, and W-A retaining 3/4; for the number of borrows, see the table of trajectory and borrow counts in section "Segment two readings: Q2, Q3 (three windows)"); "which pair to check first" accounts for 99 pairs = 14 sequences with all six rows identical (three points of W-S, four points of W-A, five seeds of W-R p = 0.95, seeds 3 and 5 of p = 0.75) totaling 84 pairs, plus 15 pairs where some rows are identical on the five seeds of W-R p = 0.55 and seeds 1, 2, 4 of p = 0.75. For these 131 pairs, one can only state "values at the two points are identical", with no self-validation of discriminative power. The 15 pairs with differing verdicts all reside on the "how much to borrow" knob (lines 1169, 1181, 1185, 1209–1216, 1229–1232; listed by `grep -n 'name=sensitivity.* result=unstable' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out`, copying four full lines as examples):

```
1169:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2p_delta3 main=wr_p075_seed1@M3 main_value=0.0724 main_verdict=negligible opposite=wr_p075_seed1@M3_just_enough opposite_value=0.1106 opposite_verdict=no_flip result=unstable
1209:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2p_delta3 main=ws_a1000_d500_smallest_first@M3 main_value=0.0299 main_verdict=negligible opposite=ws_a1000_d500_smallest_first@M3_just_enough opposite_value=0.4975 opposite_verdict=no_flip result=unstable
1210:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2c_delta3 main=ws_a1000_d500_smallest_first@M3 main_value=0.0342 main_verdict=negligible opposite=ws_a1000_d500_smallest_first@M3_just_enough opposite_value=0.5018 opposite_verdict=no_flip result=unstable
1230:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2c_delta3 main=wa_keep1of233_a2330@M3 main_value=0.0275 main_verdict=negligible opposite=wa_keep1of233_a2330@M3_just_enough opposite_value=0.4568 opposite_verdict=no_flip result=unstable
```

Q2p/Q2c/Q3 are unstable on borrowing amount for four M3-related rows at W-R p=0.75, W-S ascending and W-A retaining 1/233; on neighbor order both points have the same verdict.

### Comparison (stop clause, full lines)

Complete `e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out`:

```
1:E7RESULT name=oracle_constants model_leaf_record_capacity=233 crates_inode_leaf_records=233 model_root_node_leaf_container_capacity=135 crates_leaf_containers_one_root_node_holds=135 equal=true
2:E7RESULT name=oracle_crates_unit_test_shape model=sizes=233/233/34,container_numbers=1/234/467 crates=sizes=233/233/34,container_numbers=1/234/467 equal=true
3:E7RESULT name=oracle_crates_unit_test_shape model=refused_inode=31456,refused_containers=136,capacity=135 crates=refused_inode=31456,refused_containers=136,capacity=135 equal=true
4:E7RESULT name=oracle_crates_unit_test_shape model=position_rewritten=[1],dropped=1 crates=position_rewritten=[1],dropped=1 equal=true
5:E7RESULT name=oracle sequence=wr_p055_seed1 operations_compared=78967 unlinks_compared=35448 m1_refused_operation=78967 first_difference=none
6:E7RESULT name=oracle sequence=wr_p055_seed2 operations_compared=83764 unlinks_compared=37793 m1_refused_operation=83764 first_difference=none
7:E7RESULT name=oracle sequence=wr_p055_seed3 operations_compared=81145 unlinks_compared=36583 m1_refused_operation=81145 first_difference=none
8:E7RESULT name=oracle sequence=wr_p055_seed4 operations_compared=83082 unlinks_compared=37574 m1_refused_operation=83082 first_difference=none
9:E7RESULT name=oracle sequence=wr_p055_seed5 operations_compared=82642 unlinks_compared=37279 m1_refused_operation=82642 first_difference=none
10:E7RESULT name=oracle sequence=wr_p075_seed1 operations_compared=42169 unlinks_compared=10404 m1_refused_operation=42169 first_difference=none
11:E7RESULT name=oracle sequence=wr_p075_seed2 operations_compared=42502 unlinks_compared=10664 m1_refused_operation=42502 first_difference=none
12:E7RESULT name=oracle sequence=wr_p075_seed3 operations_compared=42241 unlinks_compared=10431 m1_refused_operation=42241 first_difference=none
13:E7RESULT name=oracle sequence=wr_p075_seed4 operations_compared=42701 unlinks_compared=10853 m1_refused_operation=42701 first_difference=none
14:E7RESULT name=oracle sequence=wr_p075_seed5 operations_compared=42524 unlinks_compared=10683 m1_refused_operation=42524 first_difference=none
15:E7RESULT name=oracle sequence=wr_p095_seed1 operations_compared=33152 unlinks_compared=1643 m1_refused_operation=33152 first_difference=none
16:E7RESULT name=oracle sequence=wr_p095_seed2 operations_compared=33173 unlinks_compared=1681 m1_refused_operation=33173 first_difference=none
17:E7RESULT name=oracle sequence=wr_p095_seed3 operations_compared=33191 unlinks_compared=1694 m1_refused_operation=33191 first_difference=none
18:E7RESULT name=oracle sequence=wr_p095_seed4 operations_compared=33180 unlinks_compared=1682 m1_refused_operation=33180 first_difference=none
19:E7RESULT name=oracle sequence=wr_p095_seed5 operations_compared=33190 unlinks_compared=1684 m1_refused_operation=33190 first_difference=none
20:E7RESULT name=oracle sequence=ws_a1000_d500_smallest_first operations_compared=92246 unlinks_compared=30500 m1_refused_operation=92246 first_difference=none
21:E7RESULT name=oracle sequence=ws_a1000_d950_smallest_first operations_compared=1184620 unlinks_compared=576650 m1_refused_operation=1184620 first_difference=none
22:E7RESULT name=oracle sequence=ws_a1000_d500_largest_first operations_compared=92456 unlinks_compared=30500 m1_refused_operation=92456 first_difference=none
23:E7RESULT name=oracle sequence=wa_keep1of2_a1000 operations_compared=50133 unlinks_compared=16500 m1_refused_operation=50133 first_difference=none
24:E7RESULT name=oracle sequence=wa_keep1of2_a2330 operations_compared=47064 unlinks_compared=15145 m1_refused_operation=47064 first_difference=none
25:E7RESULT name=oracle sequence=wa_keep1of233_a2330 operations_compared=67197 unlinks_compared=32480 m1_refused_operation=67197 first_difference=none
26:E7RESULT name=oracle sequence=wa_keep3of4_a1000 operations_compared=40607 unlinks_compared=8000 m1_refused_operation=40607 first_difference=none
27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
28:E7RESULT name=done emitted=28
```

Discriminative power self-test: with V1 enabled in the model, `e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out` reports first differences in seven sequences. At the W-S main sample this is operation 1233; the `crates/` rewrite list has length 4, model 0, matching registration §9. The `crates/` “remove empty middle leaf” unit-test shape also differs. The other 15 sequences (W-R p=0.75/0.95, W-S descending, all four W-A) never removed a non-rightmost leaf before M1 rejection, so the positional-shift branch was not reached by comparison.

```
4:E7RESULT name=oracle_crates_unit_test_shape model=position_rewritten=[],dropped=1 crates=position_rewritten=[1],dropped=1 equal=false
20:E7RESULT name=oracle sequence=ws_a1000_d500_smallest_first operations_compared=1233 unlinks_compared=233 m1_refused_operation=none first_difference=operation=1233 Unlink(233) positional rewrite list differs: crates [0, 1, 2, 3] (length 4), model [] (length 0)
27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
```

Before segment two, comparison and discriminability were each rebuilt and rerun using segment-two source. The comparison artifact is byte-identical to segment one (`all_equal=true`), and the V1 discriminability artifact is also byte-identical (mutant exits 1 and again reports W-S operation 1233). Comparison covers only M1, not the three segment-two variants.

### Unit tests and mutations

There are 22 unit tests on the segment-two source: `grep -c '^test .* \.\.\. ok$' e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out` outputs 22; the same file contains `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s` in `e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out`. Of segment one's 14 tests (`e176-inode-leaf-merge-capacity-and-rewrites-unit-tests.out`), those that pin absolute values: all 26 registered anchors, independent addition formulas for M1 a1/a2, three public SHA-256 vectors, first steps of four workload generators, five bad class-C forms, threshold boundary values, moved-boundary proof, threaded versus single-thread output, and equal operation-stream prefixes for three arms at W-R main points. The eight added for segment two pin its window table and three variants, anchors (29 registered of 47 total), right-first versus left-first final states and both rewrite counts for three 117/117/117 leaves (two each for M2/M3), threaded/single-thread equality for six variants, and sensitivity knob/row mapping including the `left_first` window. The random long-run class-C test adds M3_left_first.

Mutation table `scripts/mutation/tables/e176_inode_leaf_merge_capacity_and_rewrites.tsv` has 16 mutations (V1–V13 in §9 and V14–V16 in amendment 3), each matching exactly once in segment-two source: 16 caught, zero invalid, zero survived; 💥/⚠️/⏱/🧱 all zero; summary reports zero memory-cap, timeout, scheduling, total-cap or GPU-memory losses and baseline restored green. See `e176-inode-leaf-merge-capacity-and-rewrites-mutations-2026-10-08.out`; all 13 segment-one mutations were caught in the earlier output. V4 changes `live_inodes_at_refusal` shared by sequences/anchors; V13 immediately trips invalidation at W-R main sample; V9 trips class-C nonoverlapping adjacent key ranges; V14 is caught by the two left-first 117/117/117 tests, V15 by a3 just-enough registered answer, V16 by segment-two window-table test.

Catching all 16 only shows the asserted locations were mutation-tested, not every model line. Verdict and sensitivity arithmetic (`point_verdict`, `overall_verdict`, `moved_threshold_verdicts`) has boundary unit tests but no corresponding mutations; segment-two sensitivity window selection in `segment_two_sensitivity_lines` has a single unit-test check, no mutation.

### Fork sheet

| Fork row | Status | Evidence (the command that computes it from the artifact) | What is still missing; can the remaining quantities reverse the verdict? |
|---|---|---|---|
| M-row first flip observation: live count when M1 reaches 135 leaves | Sufficient for judgment | `live_at_stop` from `grep 'name=arm_run' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out`; the Q1r `awk` command in “Readings: Q1”; the segment-one overall verdict on line 763 and the segment-two main-window verdict on line 1130 agree | Nothing is missing. Both segments completed the registered quantities, with no quantities left to measure. Segment two did not change M1 or main M2 (the 396 rows for the main window and the three arms M1, M2 and M3 in segment two are identical to segment one), so the Q1r verdict is unchanged. Q1r being unstable across workloads, as described in the sensitivity section, is a measured property, not a missing quantity |
| M, second flip observation: leaves rewritten per unlink by M2/M3 relative to M1 | Sufficient. Positional Δ2p/Δ3p are “no flip” in all three windows. Content Δ2c “negligible” in all; Δ3c and Q3 “negligible” in main/left_first, “no flip” in just_enough. | Segment-one verdicts 764–775, segment-two 1131–1148, and Q2/Q3 awk with each window. | Nothing. Segment two is the final registered segment. Positional results never flip to “negligible” or “significant”; max Δ3p is 0.4975 just_enough, 0.1111 left_first. Borrow amount changes Δ3c/Q3 in 15 pairs, so unstable; no point in three windows reaches 1.00. |
| M sufficiency: both quantities measured for all three workload families | Sufficient | Segment-one lines 760/761 and segment-two 1128/1129 show equal prefixes and M1 reached at all points; all 88 and 132 `arm_run` rows respectively end `stop=refused`. | Nothing. |

### What it cannot answer

- Only leaf grouping and rewrite pages are counted. It does not construct separator-key routing in internal nodes (M3-borrowed IDs would route to the right leaf under separator keys, a question to answer along with changing I-9.4), crash-point replay (the other half of C116), or GPU-judgment modification effort. The M fork still depends on these unmeasured aspects.
- One operation per publication; batched inode creation, unlink's intent publication and intermediate batches are excluded (each changes one record in place and rewrites that leaf in all arms). The root is rewritten in the final batch of all arms and excluded.
- The 22 sequences are enumerated points, not the domain: W-R has only three probabilities and W-A four retained fractions. Q1r changes side with p and retained fraction here; long steady state or batch removal of a directory were not measured.
- M3 measured only two floors (117 and 59), two borrowing amounts (equal split and borrowing just enough to meet the floor), and two orders for checking neighboring pairs (right first and left first). The three knobs were not crossed: just-enough borrowing and left-first checking were measured only at f = 117. Other borrowing methods, such as borrowing to a different ratio, were not measured. The borrowing-amount knob reverses Δ3c and Q3, but which side intermediate values between the measured levels fall on has not been measured.
- For neighbor order, 99 pairs are exactly equal and cannot support moved-boundary proof. No stepwise count establishes whether those sequences ever reached the branch where both adjacent pairs fit.
- Each arm capped at 2×10⁷ operations, with no arm hitting it. W-A retain 1/233 M2 and M2_left_first consumed 13547775 operations before refusal, the longest in segment two.
- Minimum fill in all 88 segment-one and 132 segment-two arms is 1÷233 at first inode creation, not a measure of sparseness.
- The registered `replay.sh` command in the replay section currently exits 2 without running a line: whole-table shape and admission checks were blocked by a slash-containing path in E175 column 4 at segment-two writing time, and an E142 (New-Pool File Creation Dry Run) admission-key conflict when segment one was written, neither related to E176. Both artifacts were produced by manually running the same command twice and comparing bytes.

### Paths and conclusion register

| # | Path | Calculation | Source | Shared inputs |
|---|---|---|---|---|
| 1 | Independent handwritten model | Leaf table stores IDs only; insertion splits at end, empty leaves are removed, M2 merges and M3 borrows. Both rewrite conventions are calculated from changed IDs and first removed position. M1 is compared with path 2 at every step before refusal across 22 registered sequences (enumerated points, not domain); anchors compared with path 3. | `scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs:155 insert`, `:278 remove`, `:321 rewrites_of_this_publish` | 233 records/page and root fanout 135 copied from `crates/` and compared at path 2; positional definition read from then-current `remove_record_from_leaf_containers`; same workload generator feeds both sides of comparison. |
| 2 | Two `crates/` pure functions | Then-current M1: `write_records_into_leaf_containers` appends/splits at end and rejects >135 pages; `remove_record_from_leaf_containers` drops empty pages and puts right-hand pages on rewrite list. | `crates/singlefs-core/src/inode_tree.rs:181 write_records_into_leaf_containers`, `:286 remove_record_from_leaf_containers` | `INODE_LEAF_RECORDS` from format, `leaf_containers_one_root_node_holds` from inode_tree; path 1 workload generator for comparison. |
| 3 | Designer's independent Python reference | Same three arms for six hand-built registration §7 scenarios and the space-amplification cell only. | No committed implementation: `anchors.py` block beginning at `e176-preregistration.md` line 500, draft copy not committed. | 233, 135, same positional definition. It does not check the newly adjacent pair after X becomes empty and is removed (§13); the six cells are unaffected. |

Path 1 vs 2 is asserted in `scripts/experiment-apparatus/e176_crates_oracle/oracle.rs` `compare_one_sequence`, stopping at first difference. Path 1 vs 3 is pinned by `every_registered_anchor_matches_the_independent_reference` (source line 2154, 26 registered answers compared verbatim); segment-two test `segment_two_anchors_judge_a1_for_the_three_variants_and_only_report_the_rest` compares 29. Shared-input limitations: if 233/135 are wrong, all three paths can be wrong together; if their common positional interpretation is not what `crates/` intends, all three are wrong; generator exists only in path 1, so comparison cannot prove it follows registration (first steps of four generators are pinned by tests).

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure), settled item 6 | Support | 2026-10-17 unaffected: this round synchronizes wording, artifact references, paths and numbers only, leaving conclusion, values and artifacts unchanged. 2026-10-11 also unaffected: wording only (time, positional references, self-reference), with conclusion, values and artifacts unchanged. |

## Historical Versions

### 2026-10-08: segment two run

- **Before:** status “partly run (segment one; segment-two trigger met, awaiting request)”; three segment-two variants unrun, limitations said just-enough and left-first were unrun; 14 tests, 13 mutations; one `replay.sh` entry, segment-one.
- **After:** the status is “run (two segments).” The text adds segment two (22 sequences × six arm variants × three windows): anchors, Q2/Q3 readings for the two windows, trajectories and borrowing counts, sensitivity of the two knobs, and identification of the verdict lines. The 38 pairs with equal values at both points in segment one are described as the number of pairs for which the moved-threshold self-test cannot be done. There are 22 unit tests and 16 mutations (amendment 3 added V14–V16). The differential and discriminative power check were each rebuilt and rerun once using the segment-two source; both were byte-identical to segment one. One `segment-two` row was added to `replay.sh`. The row for D8 (Core Index Structure) settled item 6 remains supporting material, and its review was rewritten.
- **Evidence:** `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out`, lines 1137–1148 (overall verdicts for the two new windows), line 1369 `E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=205 unstable_pairs=15 moved_to_unstable=74 equal_values_not_separable=131 every_separable_same_pair_moved_to_unstable=true`; preregistration §12 amendment 3.

### 2026-10-08: page created, segment one run

Preregistration `e176-preregistration.md` preceded apparatus. After model tests but before artifact, §12 amendment 1 added f=59 positive controls and common-window interpretation. The first mutation run found the optional path dependency prevented `mutate.sh`'s copy from building, so amendment 2 moved comparison to a standalone manifest. Segment one completed; segment-two trigger was met, awaiting the main agent's request.
