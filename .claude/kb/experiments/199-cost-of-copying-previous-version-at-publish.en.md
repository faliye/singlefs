## E199 Cost of Copying the Previous Version at Publish — Partially Run (Segment One Run: None of the Four Fork Sheet Rows Is Judgeable Yet, the Ding Whole-New-Base Cell "< 2 at Every p" Does Not Flip, the Bing Call-Site List Cell Is Judgeable After the Main Agent's Check; Segment Two Not Written)

[中文](199-发布照抄上一版的代价.md) · **English** · [日本語](199-前回バージョンをそのままコピーする公開のコスト.ja.md)

Fourth pre-run registration `research/prompts/e199-preregistration-r4.md` (superseding the third, `research/prompts/e199-preregistration-r3.md`; the first three are frozen, and what the fourth does not cover follows the third). Before the segment one artifact ran, the fourth fixed five things: the degenerate forms of the Yi2 and Yi1 positive controls, how the Yi1 copied bytes are assembled, the three resident-peak formulas of Jia, Bing and Ding (Ding gains the residual term) and the order of the cumulative 10⁶ unlinks; it also asks for the Bing call sites to be read one by one. Answers the tested clause in `.claude/rules/fs-design.md` "Accounting is a by-product of transactions, not a retrospective traversal" and the D16 (Publish Semantics) and D8 (Core Index Structure) items it cites: the bytes each publish copies from the previous version's `TransactionOutput` before and after acceptance, and the bytes recomputed from all units in the same two segments each time. The four arms map to the four rows of the fork sheet `research/prompts/m4-transaction-output-copy-forks.md`: Jia (today's approach, judged with Jia after B2), Yi (sharing between versions, judged with the Yi2 persistent structure), Bing (the previous version handed in by value), Ding (deltas only, plus compaction).

### Current Status

- Segment one artifact `research/results/e199-transaction-output-copy-2026-11-05-r2.out` (sha256 `b154621313f8844024a038b1139f27904ab8ffad768debf370e61d3b397d454a`, 1050 lines, `grep -c '^E7RESULT'` also gives 1050, last line `E7RESULT name=done emitted=1050`; measured; below, "the artifact" means this one). It supersedes the older `research/results/e199-transaction-output-copy-2026-11-05.out` (sha256 `3b3529fcbbb1305c4621c9995da7c58ee65a2efbc3b6c93ad826353d682e1e96`, 1050 lines): the older one is kept, unchanged and not deleted, and the two differ only in the names on lines 1038 and 1039 (the Ding pair of `cumulative_ratio` in the cumulative lines), the other 1048 lines being byte-identical (measured: `diff` of the two lists only those two lines in their old and new versions, 4 lines in all). The apparatus `scripts/experiment-apparatus/e199_transaction_output_copy.rs` (a pure arithmetic counting model; the version that produced this artifact has sha256 `5bce2b3193d427d4664149e42ac3bc2c62f9a2704d6a950ae34d5e91ff4585b4`, 2619 lines; the older artifact came from the version with sha256 `2af77f466495280d72bf12fdd1de36439ebf24068d87a9dd35bfedeab586f581`, 2569 lines, and the two versions differ only in the name of the Ding pair in the cumulative lines, see the last item of "What It Cannot Answer") follows the fourth registration: the Yi2 positive control takes "Yi2's own persistent chunked vectors with sharing turned off, every collection copied whole", the Yi1 positive control takes "sharing turned off, each item reverting to Jia after B2", the Yi1 copied bytes follow the three rules of the fourth registration item by item (reported only, not judged), Ding's resident peak gains the residual term, and the ratio of the runner's first reading (only the touched collections copied whole) is reported on its own line without a verdict. All 63 unit tests pass (`test result: ok. 63 passed; 0 failed`); all 57 entries of the mutation table `scripts/mutation/tables/e199_transaction_output_copy.tsv` (the 41 of the third registration, the 12 segment one entries of section 9 of the fourth, and the 4 cumulative-rename entries) are caught; the `mutate.sh` closing lines verbatim: "计数：抓到 57 条、没红 0 条、无效 0 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条、现判 57 条、复用 0 条、拒算 0 条" (count: 57 caught, 0 survived, 0 invalid, 0 unfinished, 0 name blind spots, 0 stale anchors, 57 judged now, 0 reuse, 0 refused) and "已还原，基线仍全绿" (restored, baseline still all green). Segment one does not go through the crash judging stage and does not use `--features gpu` (fixed in section 5 of the fourth registration).
- No stop clause triggered. Stop clause 1 ① of the third registration: after the apparatus was revised on 2026-11-05, 7 `.rs` files under `crates/singlefs-core/src/` in the main worktree (`compaction_plan.rs`, `container_tree.rs`, `mounted_read.rs`, `mounted_session.rs`, `mount.rs`, `recovery.rs`, `transaction.rs`) had been replaced by other changes, so the full sweep was rerun per clause 1 ②③: comparing the six hit lists by "file, enclosing function, verbatim line", each had 0 new and 0 removed hits, and of 202 hits 143 only moved line and 59 did not move; the function lists went from 844 → 854 before acceptance and 780 → 790 after, the 10 extra are all a new reader added to `recovery.rs`, and their bodies have zero hits of the three patterns (measured). Per clause 1 ③ this does not stop; the new line numbers are in the "runner revision" of section 12 of the fourth registration. The three "removed" greps of clause 2 still hit nothing. Constant check (the first row of section 10 and stop clause 3 of section 11, artifact line 2, verbatim):

```text
E7RESULT name=constant_checks clause_constants_hold=true implementation_constants_hold=true memory_widths_hold=true
```

- Completeness of the artifact (step 4b, measured): 40 sampling points, each with one line each of `name=geometry`, `name=jia_after_patch_parts`, `name=shared_bytes`, `name=yi1_copied`, `name=read_hops`, `name=resident_peak`, one `name=arm_bytes` line per arm for six arms (`grep 'name=arm_bytes ' <artifact> | grep -o 'arm=[a-z0-9_]*' | sort | uniq -c` gives 40 for each of `bing_by_value`, `ding_whole_new_base_compaction`, `ding_without_compaction`, `jia_after_patch`, `jia_today`, `yi2_persistent`), and one `name=ratio` line for each of the twelve judged cells (480 in all). Line counts per kind, the output of `grep -o '^E7RESULT name=[a-z0-9_]*' <artifact> | sed 's/^E7RESULT //' | sort | uniq -c` folded onto one line:

```text
name=arm_bytes 240; name=config 1; name=constant_checks 1; name=cumulative_arm 4; name=cumulative_ratio 6; name=cumulative_unlinks 1; name=ding_history 14; name=ding_history_jia 1; name=ding_smallest_period 2; name=discriminative_power 3; name=discriminative_power_midpoint 18; name=done 1; name=geometry 40; name=growth_order 6; name=jia_after_patch_parts 40; name=judgement 12; name=memory_widths 1; name=not_in_this_segment 9; name=positive_control 5; name=positive_control_reported_only 1; name=ratio 480; name=read_hops 40; name=recompute_positive_control 3; name=resident_peak 40; name=shared_bytes 40; name=verdict 1; name=yi1_copied 40;
```

- All twelve judged cells are judged "does not lean to keeping Jia" at G0 (copied-byte ratios before and after acceptance, three pairs each, two forms; recompute ratios in both segments, three pairs each; a form A zero denominator is judged ≥ 2 per the registration). Artifact lines 16–27, verbatim:

```text
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_yi2_persistent_copied_before_admission forms=200.6006/199.2003 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_bing_by_value_copied_before_admission forms=zero_denominator/28392.8103 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_ding_without_compaction_copied_before_admission forms=zero_denominator/28392.8103 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_yi2_persistent_copied_after_admission forms=192.8014/79.2828 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_bing_by_value_copied_after_admission forms=17469.9292/132.9447 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_ding_without_compaction_copied_after_admission forms=zero_denominator/133.9565 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_yi2_persistent_recomputed_before_admission forms=28.2808 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_bing_by_value_recomputed_before_admission forms=28.2808 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_ding_without_compaction_recomputed_before_admission forms=28.2808 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_yi2_persistent_recomputed_after_admission forms=6.9648 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_bing_by_value_recomputed_after_admission forms=6.9648 side_at_two=does_not_lean_keep_jia
E7RESULT name=ratio point=main_geometry cell=jia_after_patch_over_ding_without_compaction_recomputed_after_admission forms=6.9648 side_at_two=does_not_lean_keep_jia
```

- Geometry sensitivity (the `sensitivity=` and `at_points=` parts of the `name=judgement` lines, artifact lines 979–990; each cell rerun at G-small, s = 1, a_T = 815000, I = 1, φ = ½ and the focused tier): of the twelve cells two are stable and ten are "unstable", and the only flipping points are G-small and a_T = 815000; at s = 1, I = 1, φ = ½ and the focused tier all twelve cells sit on the same side as at G0, so the clause "the touched-leaf tier decides the outcome" did not trigger. Output of `grep 'name=judgement' <artifact> | grep -o 'sensitivity=[a-z_0-9+]*' | sort | uniq -c` verbatim: `2 sensitivity=stable`, `7 sensitivity=unstable_flipped_at_small_geometry`, `3 sensitivity=unstable_flipped_at_small_geometry+tombstones_added_815000`.

| Cell (Jia after B2 ÷ that arm) | G0 | Flipping point and the side it flips to | Artifact lines |
|---|---|---|---|
| Copied before acceptance ÷ Bing, ÷ Ding (two cells) | Does not lean to keeping Jia | Stable | lines 980, 981 |
| Copied before acceptance ÷ Yi2, copied after acceptance ÷ Yi2 | Does not lean to keeping Jia | G-small leans to keeping Jia (form A / B 0.7512 / 0.7860, 0.4754 / 0.9119, lines 928, 931) | lines 979, 982 |
| Copied after acceptance ÷ Bing, ÷ Ding | Does not lean to keeping Jia | At G-small the two forms fall on the two sides of 2, recorded "unstable" (12.8490 / 1.0878, zero_denominator / 1.0959, lines 932, 933) | lines 983, 984 |
| Recompute before acceptance ÷ Yi2, ÷ Bing, ÷ Ding | Does not lean to keeping Jia | G-small leans to keeping Jia (1.0059, lines 934–936) | lines 985–987 |
| Recompute after acceptance ÷ Yi2, ÷ Bing, ÷ Ding | Does not lean to keeping Jia | G-small leans to keeping Jia (1.0060, lines 937–939); a_T = 815000 leans to keeping Jia (1.9104, lines 289–291) | lines 988–990 |

- Cells on the "leans to keeping Jia" or "unstable" side at sampling points outside the judgement (reported only, judgement only at G0): the four points with tombstone records u ≤ 10⁴, plus G-small and a_T = 815000. Along the tombstone-record sweep every point from u = 10⁵ up has all twelve cells ≥ 2. The recompute ratio is 1.0041–1.0043 at u = 0, 10, 10³ and 1.1239 / 1.0601 (before / after acceptance) at u = 10⁴; the after-acceptance copy ratio at u ≤ 10³ has cells whose form B drops below 2, recorded "unstable" (÷ Yi2 at u = 0 is 4.8193 / 1.6514, line 43). Output of `grep 'name=ratio ' <artifact> | grep -v 'side_at_two=does_not_lean_keep_jia' | awk '{print $3, $NF}' | sort | uniq -c` verbatim:

```text
      8 point=small_geometry side_at_two=leans_keep_jia
      2 point=small_geometry side_at_two=unstable
      6 point=tombstone_records_0 side_at_two=leans_keep_jia
      3 point=tombstone_records_0 side_at_two=unstable
      6 point=tombstone_records_10000 side_at_two=leans_keep_jia
      6 point=tombstone_records_1000 side_at_two=leans_keep_jia
      1 point=tombstone_records_1000 side_at_two=unstable
      6 point=tombstone_records_10 side_at_two=leans_keep_jia
      3 point=tombstone_records_10 side_at_two=unstable
      3 point=tombstones_added_815000 side_at_two=leans_keep_jia
```

- Discriminative power (section 8 of the third registration: move the threshold between the G0 and G-small values, and the cell must switch from one side to the other): all three examples switch (artifact lines 991–993, verbatim); of the 18 `name=discriminative_power_midpoint` lines, 15 are `outcome=midpoint_flips_true` and 3 are `outcome=zero_denominator_no_threshold_between` (form A of copied before acceptance ÷ Bing, ÷ Ding and copied after acceptance ÷ Ding: the denominator is 0 at both G0 and G-small, so no threshold fits between them).

```text
E7RESULT name=discriminative_power cell=jia_after_patch_over_yi2_persistent_copied_before_admission threshold=1/1 main_geometry=does_not_lean_keep_jia small_geometry=leans_keep_jia flips=true
E7RESULT name=discriminative_power cell=jia_after_patch_over_yi2_persistent_recomputed_before_admission threshold=5/1 main_geometry=does_not_lean_keep_jia small_geometry=leans_keep_jia flips=true
E7RESULT name=discriminative_power cell=jia_after_patch_over_yi2_persistent_recomputed_after_admission threshold=5/1 main_geometry=does_not_lean_keep_jia small_geometry=leans_keep_jia flips=true
```

- All positive controls pass (voiding clauses 3 and 8 did not trigger): the u = 10⁶ to u = 10⁵ ratio of the five arms' degenerate forms is 9.2257–9.7597 (threshold 9), and the three recompute ones are 8.3247 (threshold 8); the Yi2 reading "only the touched collections copied whole" gives 5.2928, reported only, not judged (section 6 of the fourth registration). Artifact lines 970–978, verbatim:

```text
E7RESULT name=positive_control arm=jia_after_patch_copied_before tombstone_records_1e5=41402308 tombstone_records_1e6=404072244 ratio=9.7597 threshold=9 holds=true
E7RESULT name=positive_control arm=yi2_persistent_sharing_turned_off tombstone_records_1e5=20775040 tombstone_records_1e6=202620136 ratio=9.7531 threshold=9 holds=true
E7RESULT name=positive_control arm=yi1_reference_counted_sharing_turned_off tombstone_records_1e5=41402308 tombstone_records_1e6=404072244 ratio=9.7597 threshold=9 holds=true
E7RESULT name=positive_control arm=bing_cloned_before_handing_in tombstone_records_1e5=8866368 tombstone_records_1e6=81798495 ratio=9.2257 threshold=9 holds=true
E7RESULT name=positive_control arm=ding_period_1_whole_new_base tombstone_records_1e5=8863344 tombstone_records_1e6=81793919 ratio=9.2283 threshold=9 holds=true
E7RESULT name=recompute_positive_control arm=yi2_persistent tombstone_records_1e5=111200 tombstone_records_1e6=925712 ratio=8.3247 threshold=8 holds=true
E7RESULT name=recompute_positive_control arm=bing_by_value tombstone_records_1e5=111200 tombstone_records_1e6=925712 ratio=8.3247 threshold=8 holds=true
E7RESULT name=recompute_positive_control arm=ding_without_compaction tombstone_records_1e5=111200 tombstone_records_1e6=925712 ratio=8.3247 threshold=8 holds=true
E7RESULT name=positive_control_reported_only arm=yi2_persistent_touched_collections_whole tombstone_records_1e5=440992 tombstone_records_1e6=2334088 ratio=5.2928 judged=no reason=registered_reading_is_sharing_turned_off
```

- Jia's growth order (reported, not a criterion): the log-log slopes of the copies before acceptance over u = 10⁵ → 10⁶ and 10⁶ → 10⁷ are 0.98943 and 0.99892; recompute before acceptance 0.91849 and 0.98988, after acceptance 0.92037 and 0.99026. Artifact lines 964–969, verbatim:

```text
E7RESULT name=growth_order quantity=jia_after_patch_copied_before from_tombstone_records=1e5 to_tombstone_records=1e6 lower=41402308 upper=404072244 slope=0.98943
E7RESULT name=growth_order quantity=jia_after_patch_copied_before from_tombstone_records=1e6 to_tombstone_records=1e7 lower=404072244 upper=4030722004 slope=0.99892
E7RESULT name=growth_order quantity=jia_after_patch_recomputed_before from_tombstone_records=1e5 to_tombstone_records=1e6 lower=218912 upper=1814496 slope=0.91849
E7RESULT name=growth_order quantity=jia_after_patch_recomputed_before from_tombstone_records=1e6 to_tombstone_records=1e7 lower=1814496 upper=17727192 slope=0.98988
E7RESULT name=growth_order quantity=jia_after_patch_recomputed_after from_tombstone_records=1e5 to_tombstone_records=1e6 lower=111200 upper=925712 slope=0.92037
E7RESULT name=growth_order quantity=jia_after_patch_recomputed_after from_tombstone_records=1e6 to_tombstone_records=1e7 lower=925712 upper=9051788 slope=0.99026
```

- Resident peaks (threshold 1, arm ÷ Jia ≥ 1 judges "no smaller than Jia"): at G0 Bing ÷ Jia is 0.3360, judged "smaller than Jia" (artifact line 15, verbatim); over the 256-publish run Ding gives 0.6716–0.9686 with a whole new base and 0.3390–0.9686 with fold in place, so all seven p values and both ways are judged "smaller than Jia" (`ding_resident_peak_over_jia` and `ding_resident_side_at_one=less_than_jia` on lines 1013–1019 and 1021–1027). The clause "read path or peak reversed" did not trigger for these three arms in segment one; the Yi1 and Yi2 peaks belong to segment two.

```text
E7RESULT name=resident_peak point=main_geometry jia_after_patch=245898497 bing_by_value=82623711 bing_over_jia=0.3360 bing_side_at_one=less_than_jia
```

- Ding's compaction amortization and layer trajectory (256 publishes in a row from G0, p ∈ {1, 2, 3, 4, 8, 64, 1024}, two compaction ways; artifact lines 1013–1028). The whole-new-base copy ratio after acceptance (Jia ÷ Ding, total over the 256 publishes, two forms) is < 2 at p = 1, 2 and ≥ 2 at p ≥ 3: "< 2 at every p scanned" does not hold, Ding (whole new base) does not flip, and the failure clause "Ding saves less than twofold at every p" did not trigger. "The smallest p with a ratio ≥ 2" is 3, inside the scan range rather than at an endpoint; on the fold-in-place row the compaction copy is 0 at every p, form A has a zero denominator and form B is 134.0180, and the smallest p is written `at_most_1` (≤ the lower end of the scan, 1).

| p | Whole-new-base copy ratio after acceptance (form A / form B) | Layer peak / publishes with layers / final | Whole-new-base resident peak ÷ Jia | Fold-in-place resident peak ÷ Jia | Artifact lines |
|---|---|---|---|---|---|
| 1 | 0.9927 / 0.9928 | 0 / 0 / 0 | 0.6716 | 0.3390 | lines 1013, 1021 |
| 2 | 1.9855 / 1.9710 | 1 / 128 / 0 | 0.6741 | 0.3415 | lines 1014, 1022 |
| 3 | 2.9899 / 2.9462 | 2 / 171 / 1 | 0.6766 | 0.3439 | lines 1015, 1023 |
| 4 | 3.9710 / 3.8849 | 3 / 192 / 0 | 0.6791 | 0.3464 | lines 1016, 1024 |
| 8 | 7.9419 / 7.5508 | 7 / 224 / 0 | 0.6890 | 0.3564 | lines 1017, 1025 |
| 64 | 63.5342 / 43.3204 | 63 / 252 / 0 | 0.8280 | 0.4954 | lines 1018, 1026 |
| 1024 | zero_denominator / 134.0180 | 256 / 256 / 256 | 0.9686 | 0.9686 | lines 1019, 1027 |

```text
E7RESULT name=ding_smallest_period compaction=whole_new_base jia_resident_peak=246007215 smallest_period_with_copied_after_ratio_at_least_two=3
E7RESULT name=ding_smallest_period compaction=fold_in_place jia_resident_peak=246007215 smallest_period_with_copied_after_ratio_at_least_two=at_most_1
```

- Cumulative 10⁶ unlinks (2 × 10⁶ publishes, Ding compacting to a whole new base every 64 publishes): the copy ratios and recompute ratios of all three pairs in both segments are judged "does not lean to keeping Jia"; both forms of the copy ratio are ≥ 36.9704 and the recompute ratios are 5.5843–16.5371. Artifact lines 1029–1039, verbatim (the denominator of the two `cumulative_ratio` lines `pair=…_over_ding_period_64_whole_new_base` is taken from the line `cumulative_arm arm=ding_period_64_whole_new_base`: 81657856835790 ÷ 1290415051799 = 63.2803, inferred, computed from the two lines. In the older artifact that pair is named `…_over_ding_without_compaction`, which does not match the arm it uses, and the numbers follow "order of the cumulative 10⁶ unlinks" in section 5 of the registration; the apparatus was renamed and this artifact was rerun, and compared line by line with the older one only these two names differ, see the last item of "What It Cannot Answer"):

```text
E7RESULT name=cumulative_unlinks unlinks=1000000 publishes=2000000 ding_period=64 installed_before=25359019628 installed_after=943853358320
E7RESULT name=cumulative_arm arm=jia_after_patch copied_before=305008881086000 copied_after=81657856835790 recomputed_before=1857440211560 recomputed_after=946909104340
E7RESULT name=cumulative_arm arm=yi2_persistent copied_before=2093814072928 copied_after=502330749906 recomputed_before=112319425340 recomputed_after=157169338117
E7RESULT name=cumulative_arm arm=bing_by_value copied_before=0 copied_after=7266010952 recomputed_before=112319425340 recomputed_after=157169338117
E7RESULT name=cumulative_arm arm=ding_period_64_whole_new_base copied_before=0 copied_after=1290415051799 recomputed_before=112319425340 recomputed_after=169567566962
E7RESULT name=cumulative_ratio pair=jia_after_patch_over_yi2_persistent segment=before_admission copied_without_installed=145.6714 copied_with_installed=143.9402 copied_verdict=does_not_lean_keep_jia recomputed=16.5371 recomputed_verdict=does_not_lean_keep_jia
E7RESULT name=cumulative_ratio pair=jia_after_patch_over_yi2_persistent segment=after_admission copied_without_installed=162.5579 copied_with_installed=57.1170 copied_verdict=does_not_lean_keep_jia recomputed=6.0248 recomputed_verdict=does_not_lean_keep_jia
E7RESULT name=cumulative_ratio pair=jia_after_patch_over_bing_by_value segment=before_admission copied_without_installed=zero_denominator copied_with_installed=12028.6290 copied_verdict=does_not_lean_keep_jia recomputed=16.5371 recomputed_verdict=does_not_lean_keep_jia
E7RESULT name=cumulative_ratio pair=jia_after_patch_over_bing_by_value segment=after_admission copied_without_installed=11238.3338 copied_with_installed=86.8468 copied_verdict=does_not_lean_keep_jia recomputed=6.0248 recomputed_verdict=does_not_lean_keep_jia
E7RESULT name=cumulative_ratio pair=jia_after_patch_over_ding_period_64_whole_new_base segment=before_admission copied_without_installed=zero_denominator copied_with_installed=12028.6290 copied_verdict=does_not_lean_keep_jia recomputed=16.5371 recomputed_verdict=does_not_lean_keep_jia
E7RESULT name=cumulative_ratio pair=jia_after_patch_over_ding_period_64_whole_new_base segment=after_admission copied_without_installed=63.2803 copied_with_installed=36.9704 copied_verdict=does_not_lean_keep_jia recomputed=5.5843 recomputed_verdict=does_not_lean_keep_jia
```

- Verdict line (`grep -n 'name=verdict' <artifact>` finds only line 1049, verbatim): no `false` field; `execution_clock=not_run` is the execution wall-clock ratio, which the registration assigns to segment two, as expected.

```text
E7RESULT name=verdict segment=counting_model clause_constants_hold=true implementation_constants_hold=true memory_widths_hold=true discriminative_power_examples_flip=true positive_control_jia_after_patch_copied_before=true positive_control_yi2_persistent_sharing_turned_off=true positive_control_yi1_reference_counted_sharing_turned_off=true positive_control_bing_cloned_before_handing_in=true positive_control_ding_period_1_whole_new_base=true recompute_positive_control_yi2_persistent=true recompute_positive_control_bing_by_value=true recompute_positive_control_ding_without_compaction=true execution_clock=not_run
```

- Failure clauses one by one (the ten of section 10 of the second registration, the first clause's command per the third):

| Clause | What the segment one artifact shows | Triggered |
|---|---|---|
| Clause and implementation disagree | all three items on artifact line 2 are true | No |
| The investigation report's cost order does not hold | the main agent decided not to run segment three | Cannot trigger |
| Yi2 fails to grow "only with k and tree height" | measured at the equal-height points in segment two | Not judgeable in segment one |
| Result opposite to the expectation of section 4 | all twelve cells at G0 are ≥ 2, on the same side as the 200.6006 / 199.2003, 28.3, 6.96 computed in section 4; wall-clock belongs to segment two | Not for the byte half |
| Bytes do not separate wall-clock | needs the execution wall-clock of segment two | Not judgeable in segment one |
| Read path or peak reversed | the Bing and Ding resident peaks ÷ Jia are all < 1; read-path nanoseconds and Yi's peaks belong to segment two | No for these three arms in segment one |
| Ding saves less than twofold at every p | the whole-new-base ratio is ≥ 2 at p ≥ 3 | No |
| Recompute reversed | recompute ratios at G0 are 28.2808 and 6.9648 | No |
| The two forms fall on two sides | none at G0; at G-small the two cells copied after acceptance ÷ Bing, ÷ Ding, and seven cells at u ≤ 10³ | No at G0; the cells at those points are recorded "unstable" |
| The touched-leaf tier decides the outcome | at the focused tier all twelve cells sit on the same side as at the spread tier | No |

- The rule classification of the Bing call sites (by file and enclosing function, not read site by site, the classification script has no unit tests): it only stands for the hit counts and the scale and is not evidence that the Bing row of the fork sheet can be judged. Artifact `research/results/e199-transaction-output-copy-call-sites-2026-11-04.out` (lines 69, 539, 690 and lines 696–706, verbatim):

```text
E7RESULT name=grep grep=transaction_output_reference hits=469 command=grep+-rn+&TransactionOutput+crates/+--include=*.rs
E7RESULT name=grep grep=pool_version_in_core hits=150 command=grep+-rn+PoolVersion+crates/singlefs-core/src+--include=*.rs
E7RESULT name=grep grep=previous_version_clone hits=5 command=grep+-rn+current_version_of_trees.clone()\|TransactionOutput.*\.clone()+crates/+--include=*.rs
E7RESULT name=call_site_count grep=pool_version_in_core kind=other count=70
E7RESULT name=call_site_count grep=pool_version_in_core kind=publish_path_borrows_the_previous_version count=34
E7RESULT name=call_site_count grep=pool_version_in_core kind=session_holds_the_current_version count=46
E7RESULT name=call_site_count grep=previous_version_clone kind=crash_judge_and_harness_tests count=2
E7RESULT name=call_site_count grep=previous_version_clone kind=other count=2
E7RESULT name=call_site_count grep=previous_version_clone kind=session_holds_the_current_version count=1
E7RESULT name=call_site_count grep=transaction_output_reference kind=crash_judge_and_harness_tests count=342
E7RESULT name=call_site_count grep=transaction_output_reference kind=other count=22
E7RESULT name=call_site_count grep=transaction_output_reference kind=publish_path_borrows_the_previous_version count=88
E7RESULT name=call_site_count grep=transaction_output_reference kind=rollback count=7
E7RESULT name=call_site_count grep=transaction_output_reference kind=session_holds_the_current_version count=10
```

- The site-by-site Bing call-site list was delivered twice. The first, `research/results/e199-transaction-output-copy-call-sites-read-2026-11-05.out` (sha256 `b8e856ccaddb9402dd25d3160a57a6a0c5f94fc1ca8e8558abbf1fbdf8722237`, 753 lines; measured), was spot-checked by the main agent per "who checks" in section 5 of the fourth registration: the `read_class` of two sites, `admission.rs` 1212 and 1746, differed from what the main agent read, so per "any site differs, the whole list goes back to the runner to be reread" in that section it was sent back once (line 2 of the second artifact's header records this); the file is kept unchanged and this page no longer quotes its numbers. The second, `research/results/e199-transaction-output-copy-call-sites-read-2026-11-05-r2.out` (sha256 `cbf3dba0b2b445c0ef50bc10856b991b8469fd29a498a034a091d9293e6bf0eb`, 845 lines; measured), was reread in full by the runner on the main worktree as it stands: the three greps hit 487 + 153 + 5 = 645 sites (line 3 of the artifact header), and `grep -c '^E7RESULT name=call_site_read ' <list>` also gives 645 (measured); the rule classification covers 469 + 150 + 5 = 624 sites, the counts of the tree before the changes. Each site gets one line with `read_class`, `holds`, `change` and the rule classification's `rule_class`: 123 sites need changing, 522 do not, and 128 sites have a `read_class` that differs from `rule_class`. `read_class` is not decided by whether the function itself starts a publish but by on which path the version held at that site is borrowed or held (the six-class vocabulary and the order of classification are on lines 55–60 of the artifact header). Sites are split by whether `location` is under `crates/singlefs-core/src/`: the 282 sites in `core` were read function by function (signature, the lines using the value, the callers one level up); the other 363 sites in tests, the harness and apparatus (`grep '^E7RESULT name=call_site_read ' <list> | grep -c 'location=crates/singlefs-core/src/'` gives 282, `grep -vc` gives 363; measured) were classified by whether the name in the function body is handed into a publish, cloned, or only read on certain lines, so a value handed into a publish under another name is not recognized (inferred, not checked one by one).
- The main agent checked the second list: per the three-part method of "who checks" in section 5 of the fourth registration (1 every site whose `rule_class` differs from `read_class`; 2 of the remaining `change=required` sites, sorted by `location` in dictionary order, those whose position divided by 5 leaves 1; 3 of the rest, sorted the same way, those whose position divided by 20 leaves 1) it read the code at 169 sites (128 under part 1, 19 under part 2, 22 under part 3; `grep -c 'same=yes'` gives 169 and `grep -c 'same=no'` gives 0, measured), and all three columns (`read_class`, `holds`, `change`) are identical. The check record `research/results/e199-transaction-output-copy-call-sites-check-2026-11-05.out` (sha256 `bd00fc9b1f0f4ae255b038081506721875f85d66260835fa1fd940617804f459`, 173 lines) is the main agent's reading record, not an apparatus artifact, and is not in `replay.sh`; line 3 of its header also records a few sites whose justification sentence does not match the code but whose three columns are unaffected (`unmount` at `mount.rs` 4365, 4369, 4370, 4371, and `advance_compaction_by_one_publish` at `mounted_session.rs:4104`); the check compares only the three columns, so they are recorded as is. **Per "stop when judgeable" in section 11 of the fourth registration, the Bing cell "every call site (file:line) and how to change it" is recorded as judgeable; the Bing row as a whole is still not judgeable, still missing the wall-clock of segment two.** Lines 707–715, 844, 845 of the second list's artifact and the last line (line 173) of the check record, verbatim:

```text
E7RESULT name=call_site_read_count read_class=crash_judge_and_harness_tests change=not_required count=343
E7RESULT name=call_site_read_count read_class=crash_judge_and_harness_tests change=required count=34
E7RESULT name=call_site_read_count read_class=other change=not_required count=15
E7RESULT name=call_site_read_count read_class=publish_path_borrows_the_previous_version change=not_required count=73
E7RESULT name=call_site_read_count read_class=publish_path_borrows_the_previous_version change=required count=48
E7RESULT name=call_site_read_count read_class=rollback change=not_required count=7
E7RESULT name=call_site_read_count read_class=rollback change=required count=1
E7RESULT name=call_site_read_count read_class=session_holds_the_current_version change=not_required count=84
E7RESULT name=call_site_read_count read_class=session_holds_the_current_version change=required count=40
E7RESULT name=call_site_class_disagreement_total count=128
E7RESULT name=done emitted=785
E7RESULT name=call_site_check_total checked=169 rule1=128 rule2=19 rule3=22 different=0 verdict=judgeable
```

- Segment two (building the in-memory structures for real, a counting allocator, timing) is not written: the execution wall-clock ratio, the estimated wall-clock, Yi growing with C, read-path nanoseconds and the resident peaks of Yi1 and Yi2 have no numbers (the nine `name=not_in_this_segment` lines, artifact lines 1040–1048, say item by item where each belongs).

### Fork Table

The four rows of the fork sheet `research/prompts/m4-transaction-output-copy-forks.md`; per "implementation size and segments" in section 5 of the fourth registration segment one was never enough to judge (all four rows still need the wall-clock of segment two). `<artifact>` in the commands is `research/results/e199-transaction-output-copy-2026-11-05-r2.out`.

| Candidate | Status | Still missing | Can the remaining quantities flip it | Command that computes it |
|---|---|---|---|---|
| Jia, today's approach | Not judgeable | The byte half is complete: all twelve cells at G0 "do not lean to keeping Jia" (ten unstable), all twelve verdicts of the cumulative 10⁶ unlinks also "do not lean to keeping Jia", growth order 0.98943, 0.99892; still missing the execution wall-clock ratio of segment two | Yes: an execution wall-clock ratio < 2 at G0 judges "this pair leans to keeping Jia"; if it falls on the other side of 2 from the byte ratio, both lines are reported per "bytes do not separate wall-clock" | `grep 'name=ratio point=main_geometry\|name=judgement\|name=cumulative_ratio\|name=growth_order' <artifact>` |
| Yi, sharing between versions | Not judgeable | Yi2 copies 2014312 before and 421160 after acceptance at G0, both positive controls pass (9.7531, 9.7597), a read takes 5 / 3 extra hops (record / unit); still missing Yi growing with C, read-path nanoseconds, the resident peaks of Yi1 and Yi2 and the wall-clock of segment two | Yes: Yi2's copies growing ≥ 1% with C at the equal-height points flip the Yi row; a Yi2 positive control < 9 in segment two voids that Yi2 segment | `grep 'arm=yi2_persistent\|name=positive_control\|name=yi1_copied point=main_geometry\|name=read_hops point=main_geometry' <artifact>` |
| Bing, the previous version handed in by value | Not judgeable (the cell "every site that reads the previous version today (file:line) and how to change it" is judgeable; the row as a whole still needs segment two's wall-clock) | Bing copies 0 before and 4648 after acceptance at G0, resident peak ÷ Jia 0.3360; the call-site cell: the first site-by-site list was sent back once after the main agent's spot check, the second (645 sites, 123 to change) was checked by the main agent at 169 sites with all three columns identical and is recorded as judgeable; still missing the wall-clock of segment two | Yes: the wall-clock; the call-site cell is judgeable and the remaining measurements do not flip it (the list has no threshold, it is sent back only if a checked site differs, and the check passed) | `grep 'arm=bing_by_value\|name=resident_peak point=main_geometry' <artifact>`; `grep 'name=call_site_read_count' research/results/e199-transaction-output-copy-call-sites-read-2026-11-05-r2.out`; `grep 'name=call_site_check_total' research/results/e199-transaction-output-copy-call-sites-check-2026-11-05.out` |
| Ding, deltas only | Not judgeable | The cell "< 2 at every p" is judged in segment one: does not flip (smallest p = 3); the layer trajectory and resident peaks are complete; still missing the wall-clock and read-path nanoseconds of segment two | Yes: the wall-clock; the whole-new-base cell is judged on the segment one bytes, and segment two voids that Ding point or segment only if voiding clause 1 (the two segments disagree) triggers | `grep 'name=ding_history\|name=ding_smallest_period' <artifact>` |

Not run after the verdict was reached: none (no row is judgeable yet).

### Repro

```bash
# segment one artifact research/results/e199-transaction-output-copy-2026-11-05-r2.out (supersedes the older e199-transaction-output-copy-2026-11-05.out, which is no longer rerun): the E199 line of replay.sh, exact comparison
bash scripts/experiment-apparatus/replay.sh E199
# Bing call-site rule classification (e199-transaction-output-copy-call-sites-2026-11-04.out): the full source is copied into the artifact header on the lines starting with "#| "; from the repository root run python3 -I <that source> <repository root>
# site-by-site Bing call-site list (e199-transaction-output-copy-call-sites-read-2026-11-05-r2.out; the first one sent back is ...-read-2026-11-05.out) is a site-by-site reading and cannot be rerun; it is checked by the main agent reading the code site by site per "who checks" in section 5 of the fourth registration, and the check record e199-transaction-output-copy-call-sites-check-2026-11-05.out cannot be rerun either
```

### Measurement Basis

- Copied bytes: the bytes each place in the table "copy points of arm Jia" in section 3 of the registration duplicates in memory once more; installed bytes are the same for all four arms and listed on their own; recomputed bytes are listed on their own as a third kind.
- The segments before and after acceptance are reported separately, and no cell sums the two segments.
- Memory widths are the apparatus's own types (checked back with `size_of`), not the real widths of the Rust types in `crates/`.
- Two forms of byte ratio: form A counts copies only, form B counts copies plus installed bytes; when the two forms fall on the two sides of threshold 2 the cell is recorded as "unstable"; a zero denominator is reported as "denominator 0" and judged ≥ 2.
- Segment one is all formula arithmetic (a deterministic model); the strength of the evidence rests on the unit tests and mutations of the section 7 anchors, not on the number of rounds.
- Two Bing call-site lists: the rule classification is by file and enclosing function, not a site-by-site reading; the site-by-site list has one line per site, `holds` says which version the site holds (previous, current, both passed in, not a version) and `change` says whether it needs changing under Bing; the site-by-site list was delivered twice, the first was sent back after the main agent's spot check, the second was checked by the main agent, and the check record is a separate file.
- Judgement is only made at G0; at the geometry-sensitivity points only whether the cell flips counts, and the tombstone-record sweep and the other sampling points are reported only.
- Positive controls (fixed in section 5 of the fourth registration): the before-acceptance copy ratio of u = 10⁶ to u = 10⁵ passes at ≥ 9; the Yi2 degenerate form counts element bytes only (internal nodes and last-block rounding of the chunked vectors are not counted), the Yi1 degenerate form equals Jia after B2 item by item, Bing's is the rollback log plus the |V| cloned before handing in, Ding's is the |V'| of a whole new base at p = 1.
- Resident peaks (section 5 of the fourth registration): Jia = |V| + |V'| + one round (O + Rl) + recompute before acceptance ÷ s + after acceptance; Bing = |V| + installed after acceptance + rollback log + the old bytes of the k_old replaced containers and ρ index nodes + the residual term; Ding is computed publish by publish over the 256-publish run, with the residual term, and the compaction publish takes Jia after B2's after-acceptance recompute.

### Criteria and Failure Clauses

- Byte ratios before and after acceptance (Jia after B2 ÷ Yi2, ÷ Bing, ÷ Ding, two forms each) and the recompute ratio: at G0, < 2 judges "this pair leans to keeping Jia", ≥ 2 judges "does not lean to keeping Jia"; the cell is rerun at G-small, s = 1, a_T = 815000, I = 1, φ = ½ and the focused tier, and a flip is recorded as "unstable".
- Ding compaction amortization: if the whole-new-base ratio is < 2 at every p scanned, Ding (whole new base) flips.
- The execution wall-clock ratio, Yi growing with C (≥ 1% judges that it grows with C), per-read nanoseconds on the read path and the resident peaks of Yi1 and Yi2 belong to segment two.
- Voiding: a positive-control ratio < 9 (< 8 for the recompute one) voids that arm for that segment; a mismatched section 7 anchor is first checked against the section 5 formulas. Stop: a copy point or recompute point that does not match `crates/`, both sides are checked.

### What It Cannot Answer

- Segment one is only formula arithmetic: the absolute values of Jia get no differential comparison against the implementation in `crates/` (the main agent decided not to run segment three), and the memory widths are not checked against the implementation's Rust layout.
- Both wall-clocks (estimated and executed), Yi growing with C, read-path nanoseconds and the resident peaks of Yi1 and Yi2 cannot be answered by segment one.
- Yi1 is reported only, not judged; in segment one the Yi1 and Yi2 positive controls are given by formulas, and whether the measurement sees the O(C) copy needs the counting allocator of segment two.
- The second site-by-site Bing call-site list was checked by the main agent at 169 sites, all identical, and the Bing cell "every call site (file:line) and how to change it" is judgeable; this cell only says, for each site on the list, the class read, which version it holds and whether it needs changing; 169 sites were checked and the other 476 (arithmetic: 645 − 169) were not checked one by one, they rest on the method fixed before the run (every disagreement, one fifth of the `change=required` sites, one twentieth of the rest); a value handed into a publish under another name in the 363 sites of tests, the harness and apparatus is not recognized (inferred). The Bing row as a whole is still not judgeable.
- All twelve cells at G0 "do not lean to keeping Jia", but at tombstone records u ≤ 10⁴ and at G-small many cells lean to keeping Jia: the conclusion only covers the 10⁶-unlink scale the fork sheet asks about, and segment one draws no conclusion on whether Jia saves twofold on small pools.
- In the older artifact (`e199-transaction-output-copy-2026-11-05.out`) the Ding pair of `cumulative_ratio` is named `ding_without_compaction`, but its numbers come from the whole-new-base arm compacting every 64 publishes; the name does not match the numbers. The apparatus has been changed (`cumulative_arm_name` gives the one-line-per-arm lines and the three pair ratios the same name, the unit test `cumulative_ratio_pair_names_the_arm_whose_bytes_it_divides_by` pins the name to the arm, and the mutation table gained 4 entries), the artifact was rerun after the rename as `e199-transaction-output-copy-2026-11-05-r2.out`, and the older artifact is kept and superseded by it. The numbers were compared (comparison script): the older artifact has 2 lines of these two, the new one has 0 lines under the old name and 2 under the new, the lines are identical after the rename, and a direct `diff` differs only on lines 1038 and 1039 (2 old and 2 new lines, 4 in all); the rename did not move any number.

### Paths and Conclusions Registry

| # | Path | How it computes | Source placement | Shared items it reads |
|---|---|---|---|---|
| 1 | Apparatus counting model | The Rust apparatus computes copies, recompute, installed bytes, positive controls, resident peaks, Ding's 256-publish run and the cumulative 10⁶ unlinks arm by arm and geometry by geometry with the formulas of section 5 of the fourth registration (and of section 5 of the third and second that it keeps); the values are the 40 sampling points and p ∈ {1, 2, 3, 4, 8, 64, 1024}, enumerated points (the sampling points, event points and geometry-sensitivity points named in section 8 of the third registration are all among them), not a sweep of the whole domain | `scripts/experiment-apparatus/e199_transaction_output_copy.rs:1055 arm_bytes` | The formulas of section 5 and the split into "copy / recompute / installed"; the model constants of section 5 of the third registration (unit 32768, node 16384, fanout 135, 815 records per tombstone container and so on; the group taking implementation values is checked back against `crates/`); memory widths (the apparatus's own types); the G0 and G-small geometries |
| 2 | Designer's arithmetic draft | The designer's python drafts `model_r3.py` and `model_r4.py` implement the same formulas a second time and print the section 7 anchors; they compute only those anchor cells (enumerated points: G0, G-small, u = 10⁵, a_T = 0, a_L = 1, D = 32768, the focused tier, opening a new container, Ding p = 1 / 64 / 1024, the first 32 cumulative unlinks) | `research/prompts/e199-preregistration-r4.md:477 model_r4.py` | The same four items as the apparatus counting model: both paths follow the formulas, constants, memory widths and geometries written in the same registration |

The two paths are compared through literals pinned in the apparatus unit tests (the numbers computed by the designer's arithmetic draft copied into `assert_eq!`, e.g. `main_geometry_jia_after_patch_copies_before_admission`, `resident_peaks_of_jia_and_bing_at_the_main_geometry`); the designer's arithmetic draft is not rerun in the unit tests, and no assertion connects the outputs of the two paths directly, which is still owed. The two paths share the formulas of section 5 of the registration: if a formula is wrong both go wrong together and neither the unit tests nor the 57 mutations catch it; that half can only be checked by segment two (building the structures for real, the counting allocator, voiding clause 1), since the main agent decided not to run the segment three differential.

### Decision Look-back

| Decision item | Relationship | Look-back |
|---|---|---|
| D8 (Core Index Structure) settled item 16 | Preparation | 2026-11-05 Unaffected: the segment one artifact gave its numbers (all twelve cells at G0 do not lean to keeping Jia, ten unstable, Ding's whole new base does not flip) and none of the four fork sheet rows is judgeable yet; the Bing call-site list was replaced by the second one, checked by the main agent, which does not touch this item; the rerun artifact after the rename (`e199-transaction-output-copy-2026-11-05-r2.out`, differing from the older one only in the names on two lines) does not touch this item either; the basis of that item does not cite E199 (Cost of Copying the Previous Version at Publish), and the decision text did not change this time |
| D16 (Publish Semantics) settled item 1 | Preparation | 2026-11-05 Unaffected: the segment one artifact gave its numbers and all four rows still need the wall-clock of segment two; the Bing call-site cell was checked by the main agent at 169 sites, all identical, and is recorded as judgeable, and its change surface (123 sites to change, 48 of them on the path where a publish borrows the previous version) is preparation, not support for this item; the rerun artifact after the rename (`e199-transaction-output-copy-2026-11-05-r2.out`, differing from the older one only in the names on two lines) does not touch this item either; the basis of that item does not cite E199 (Cost of Copying the Previous Version at Publish), and the decision text did not change this time |
| D18 (Information Carried in a Block) settled item 24 | Preparation | 2026-11-05 Unaffected: the tombstone-record sweep (u = 0 to 10⁸) only gives the growth order of copied bytes with u and does not touch the tombstone record format, and replacing the Bing call-site list with the second one does not touch this item either; the rerun artifact after the rename (`e199-transaction-output-copy-2026-11-05-r2.out`, differing from the older one only in the names on two lines) does not touch this item either; the basis of that item does not cite E199 (Cost of Copying the Previous Version at Publish), and the decision text did not change this time |

## Historical Versions

### 2026-11-05

- The artifact was rerun after the Ding pair of the cumulative lines was renamed, and the page and the rerun registration now quote the new artifact: all 63 unit tests pass; all 57 entries of the whole mutation table are caught, closing lines verbatim "计数：抓到 57 条、没红 0 条、无效 0 条、没跑完 0 条、抓名字有盲区 0 条、锚点失效 0 条、现判 57 条、复用 0 条、拒算 0 条" and "已还原，基线仍全绿" (restored, baseline still all green); artifact `research/results/e199-transaction-output-copy-2026-11-05-r2.out`, 1050 lines (last line `E7RESULT name=done emitted=1050`), apparatus sha256 `5bce2b3193d427d4664149e42ac3bc2c62f9a2704d6a950ae34d5e91ff4585b4`, 2619 lines. Comparison: the older artifact has 2 lines under the old name, the new one 0 under the old name and 2 under the new, the lines are identical after the rename, and a direct `diff` differs on 4 lines (2 old and 2 new at lines 1038 and 1039). The E199 line of `replay.sh` now points at the new artifact; the older artifact is kept and superseded by it. The verdict line (line 1049) has only `execution_clock=not_run` (expected by the registration, the wall-clock is measured in segment two), and every other field is `true`.
  - **Basis**: lines 1038, 1039 and 1049 of the artifact `e199-transaction-output-copy-2026-11-05-r2.out`; runner report `~/.local/share/singlefs/work/e199-runner-segment-1-r8/result.md`; lines 54–57 of the mutation table `scripts/mutation/tables/e199_transaction_output_copy.tsv`.
- Site-by-site Bing call-site list: the first was sent back once after the main agent's spot check (the class differed at two sites, `admission.rs` 1212 and 1746), and the runner reread it in full and delivered the second, `research/results/e199-transaction-output-copy-call-sites-read-2026-11-05-r2.out` (645 sites, 123 to change); the main agent read the code at 169 sites by the three-part method of "who checks" in section 5 of the fourth registration, all three columns identical, check record `research/results/e199-transaction-output-copy-call-sites-check-2026-11-05.out`; the Bing cell "every call site (file:line) and how to change it" is recorded as judgeable and the Bing row as a whole is still not judgeable because of segment two's wall-clock; this page now quotes the second list and the check record. Also: in the apparatus the Ding pair of the cumulative `cumulative_ratio` was renamed `ding_period_64_whole_new_base` (numbers unchanged), one unit test and four mutations were added, and the artifact after the rename was still to be rerun at that time (rerun later, see the previous item).
- The segment one artifact ran and the status changed from "not run" to "partially run": artifact `research/results/e199-transaction-output-copy-2026-11-05.out`, 1050 lines (later superseded by `e199-transaction-output-copy-2026-11-05-r2.out`, see the previous item); 62 unit tests pass and all 53 mutations are caught; all twelve cells at G0 are judged "does not lean to keeping Jia", ten are "unstable" (flipping at G-small and a_T = 815000); the positive controls of the five arms and the three recompute ones all pass; the smallest whole-new-base p with a ratio ≥ 2 is 3, so that Ding cell does not flip; all twelve verdicts of the cumulative 10⁶ unlinks "do not lean to keeping Jia". Before that the full sweep was rerun per stop clause 1 ②③ of the third registration: 0 new, 0 removed, 143 only moved line, no stop, the line numbers went into section 12 of the fourth registration; the site-by-site Bing call-site list was delivered (the first one, 641 sites, was later sent back, see the next item); data-shape names in the apparatus were changed to full words (`jia_after_b2` → `jia_after_patch` and the like). All four fork sheet rows still need the wall-clock of segment two and are not judgeable.
- The pre-run registration moved to the fourth; the segment one apparatus was revised per the fourth (Yi2 and Yi1 positive controls, Yi1 copied bytes, residual term in Ding's resident peak), the mutation table grew to 53 entries, unit tests, mutation and the artifact have not run; after the revision 7 core `.rs` files in the main worktree were replaced, the full sweep is to be rerun; the rule classification of the Bing call sites is marked "not read site by site, not evidence for the Bing row".

### 2026-11-04

- The start-of-work stop clauses did not trigger; the segment one apparatus, unit tests and mutation table were written, the artifact has not run; the Bing call-site list produced its artifact.

### 2026-11-03

- File created, third pre-run registration recorded for E199 (Cost of Copying the Previous Version at Publish).
