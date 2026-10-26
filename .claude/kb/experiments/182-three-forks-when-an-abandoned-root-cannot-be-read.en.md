## E182 Three Forks for Unreadable Abandoned Roots — partially run (the first segment's 18 histories and supplemental arm's nine histories have been run; B1 and B4 have sufficient evidence for judgment; B2 belongs to segment two, which has not been written)

[中文](182-被抛弃根读不出的三条岔路.md) · **English** · [日本語](182-読めない破棄済みルートをめぐる-3-つの分岐.ja.md)

<!-- doc-lint:not-numbers B1 B2 B4 Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 A1 A3 A4 I1 I2 Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 -->

Preregistration: `research/prompts/e182-preregistration.md`. It has two segments: segment one answers B1 and B4 on single-device root selection, and segment two answers B2 within one process; this page covers only segment one. It answers three rows in fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md`: B1 (the hidden root's record is also unreadable and single-device recovery selects it: current policy C, A1 inspects the newer instance's records, A2 inspects the instance generation recorded by system configuration; B is undefined and no arm is constructed), B4 (tighten the first publication of the new instance to avoid the txg corresponding to the unreadable root slot, without changing the implementation, by classifying states), and B2 (segment two). The background is C554, the shadow accounting's inability to account for a temporarily unreadable root abandoned by crash recovery: the first of three forms remains outstanding, as do D23 (Journal Role & Format) settled item 14's pre-quarantine and B-configuration continuation. E158 (Four Forks for Root Selection and Repair) measures how B-configuration continuation prevents lost writes; this experiment measures the form it cannot prevent.

The committed apparatus is `crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs` (4982 lines, a read-only driver of public `singlefs-core` entry points; no changes to other `crates/` files).

**Current status:**

- The committed apparatus has 26 unit tests (`grep -c '#\[test\]'` on its source; the main agent reported “test result: ok. 26 passed; 0 failed”). The header states acceptance and run conditions, `main` begins with `preflight`, and two tests pin these. Fifteen mutation rows in `crates/mutations.tsv` have names beginning “E182（被抛弃根读不出的三条岔路） ” (`grep -c '^[E]182 ' crates/mutations.tsv`): six for segment one, four for the supplemental arm, and five for B4's own positive control. The long-run script `scripts/experiment-apparatus/e182-single-device-long-run.sh` and replay registration in `scripts/experiment-apparatus/replay.sh` under E182, `@driver_e182||e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out|exact`, were written, pointing to the new artifact regenerated after adding B4's control. The three mutation counts await checker-tier-crates-mutation-replay at commit.
- Segment-one artifact `research/results/e182-unreadable-abandoned-root-forks-single-device.out` (1204 lines, final `E7RESULT name=done emitted=1204`) was run: 18 histories, long-run script reported “judged now 432, reused 0,” sentinel rejudged zero shards; progress logs are in `~/.local/share/singlefs/work/e182-long-run/` (not committed). Readings and named cases appear in “Segment-one results,” whose line numbers refer to this artifact. After adding B4's own positive control (registration amendment 12.12), `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out` was regenerated (1211 lines, final `E7RESULT name=done emitted=1211`). Removing seven `control=b4_class_a_to_class_b` lines and `name=done` leaves it line-for-line identical to the prior artifact, with verdict lines verbatim identical. Control readings appear in “B4's own positive control”; replay points to the new artifact and the old is retained.
- Supplemental arm “overwrite retaining s units” (registration amendment 12.11, specified by the main agent in the assignment): the apparatus gained `single-device-units-kept` mode and nine histories, four tests, four mutation rows, long-run script `scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh` (`detached-run.sh long-run-check` returned 0), and replay row `@driver_e182_units_kept||e182-unreadable-abandoned-root-forks-single-device-units-kept.out|exact` under E182. The apparatus was copied into `crates/` with B4's control. Supplemental artifact `research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out` (758 lines, final `E7RESULT name=done emitted=758`) was run: nine histories, “judged now 214, reused 0,” sentinel rejudged zero shards. Readings and named cases are in “Supplemental-arm results,” whose line numbers refer to that artifact; replay points to it.
- Registration amendments are in `research/prompts/e182-preregistration.md` section 12 (12.1–12.12): change the resume key to the function unit closure fingerprint; rewrite wall-clock estimates from measurements; missing disk identity remains unreadable during single-device recovery; `Overwrite` calls `publish_overwrite`, so after the first overwrite the file has one unit and s affects only its first-file stage; use two states only; run positive controls; arm artifact names; mutation rows; tests that could not run; `walk.rs` source fingerprint differs from registration; supplemental overwrite retains s units; add B4's own positive control.

### What was done

1. The apparatus follows registration section 5: local constants compared back to `crates/` (`anchors` mode); instance 1 (mkfs → writable mount → first file of s units → p `Overwrite` operations, the final publication being hidden root R_h; recorded stream cut after the root-slot FUA write or after the publication returns); instance 2 (writable mount on a device wrapper returning zeroes for R_h's root slot, journal record and newly written units, then three `Overwrite` operations; its recorded stream is enumerated); crash-state enumeration (all proper subsets for segments with at most 16 writes, fast sampling for larger segments); recovery with both disks and with each disk independently surviving (C, A1 and A2); “covered” checked byte-by-byte against an omniscient log; B4 state classification and counting “first publication's txg lands in the hidden-root slot”; and detection and judgment positive controls.
2. One shard is one history × one segment of its recorded stream, plus the fully durable state at the end of the whole stream. Results are atomically stored in a shard result store keyed by function unit closure fingerprint + toolchain line + history name + segment number + expansion method. Restarting the same command fills only missing judgments; completion reports “judged now N, reused M.” `E182_SHARD_SHARE=<part>/<total>` distributes shards across machines by shard number modulo total.
3. Compilation: `scripts/hosts/compile-then-swap.py --release` compiled in a repository copy and swapped the whole result into the main workspace (the first attempt had a single E0599, fixed before success).
4. The supplemental arm (amendment 12.11) was written in a draft copy of the apparatus and type-checked against the main workspace's compiled rlib with `rustc --emit=metadata`, both normal and `--test`, without warnings. It was neither compiled and linked nor run at that point. It added two `OverwriteShape` variants, `publish_the_replacing_write` (`OneUnit` calls `publish_overwrite`, `KeepingTheFirstFileUnits` calls `publish_sequential_write`), and `single-device-units-kept` mode (nine histories prefixed `units-kept.`, with an additional `overwrite_shape` config field). `single-device` history names, labels and config lines remain verbatim unchanged; its artifact should remain byte-identical.
5. B4's own positive control (amendment 12.12, last sentence of registration 5.2): for each starting point take two class-a states, the earliest in segment order and the latest counting backwards, change one byte on disk 0 in a unit referenced by the hidden root, and require reclassification to class b. Run only histories with hidden roots in the confirmed tier (class a appears only in that family): six `single-device`, three `single-device-units-kept`, each one `control=b4_class_a_to_class_b` line followed by one `control=b4_class_a_to_class_b_total` summary. A history from which no class-a state can be selected is marked `not_made`, not passed. The apparatus gained three tests and five mutation rows beginning “E182（被抛弃根读不出的三条岔路） B4 对照”.
6. Header and entry point: add `admission: always` because replay and long-run scripts must rerun the same segment with unchanged inputs; `inputs-changed` would reject a second run. Add `run-condition: command python3`. `main` begins with `preflight()`, which starts `preflight.py check`; a judgment exit 78 makes the apparatus exit 78 too. Forced execution with `--force` adds a first artifact line `name=preflight forced=…`; it is absent when conditions are satisfied, leaving output byte-identical to output without this layer. In forced `list-items` it goes to standard error. Two tests pin that the judgment returns a summary only when its first line begins `forced`, and that the header has exactly one admission and one run condition with `preflight` first in `main`.
7. Supplemental artifact (amendment 12.11): `scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh` compiled the apparatus in the main workspace and ran `single-device-units-kept` through the memory wrapper with each machine's default thread count; all nine histories and 214 shards were judged now. See “Supplemental-arm results” for readings and interpretation.

### Replay

```bash
# Repository root; committed apparatus via the two scripts/experiment-apparatus/replay.sh entries
# (driver_e182 runs single-device; driver_e182_units_kept runs single-device-units-kept;
# both registered under E182). Heavy tier: main agent runs with SINGLEFS_HEAVY_TESTS=commit on commit.
bash scripts/experiment-apparatus/replay.sh E182

# Single-machine long run: compile apparatus, use memory wrapper and per-machine default threads,
# save artifact under a previously unused research/results/ name; reuse identical shard-store keys.
bash scripts/experiment-apparatus/e182-single-device-long-run.sh

# Supplemental arm “overwrite retains s units”: single-device-units-kept mode, nine histories;
# artifact research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out
bash scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh

# Apparatus modes (through memory wrapper: bash scripts/hosts/run-with-memory-cap.sh <limit> target/release/e182_unreadable_abandoned_root_forks <mode>)
#   anchors              constants comparison, I1, I2, A4, source and closure fingerprints (no history, not committed)
#   list-items           list selectable histories (18 segment-one, then nine supplemental, prefixed units-kept.)
#   measure [history]    build 18 histories and report state counts, sample 100 states for wall time (not committed)
#   outcome-samples [history]  recover outcomes for three states (not committed)
#   single-device [--full] [--item <history>]…
#   single-device-units-kept [--full] [--item <history>]…
#   Each mode first checks admission and run conditions; --force adds name=preflight forced=… to the artifact.
```

Each history and each combined “family × geometry” in the artifact has one `name=counts` row with named counts and multiple `name=ratio` rows reporting each section-6 Q's numerator, denominator, and integer parts per ten thousand. Values below one part per ten thousand round to 0, so inspect numerator and denominator. The three Q7 rows do not name classes in the row; they are ordered class a, class b, class c. Named counts are `b4.class_a`, `b4.class_b`, `b4.class_c` in `name=counts`.

Shard splitting, resumption and multi-machine operation are declared in the long-run script header. `bash scripts/hosts/detached-run.sh long-run-check -- bash scripts/experiment-apparatus/e182-single-device-long-run.sh` returned 0.

### Segment-one results (18 histories, interpretation per amendment 12.4)

Artifact `research/results/e182-unreadable-abandoned-root-forks-single-device.out` has 1204 lines (`wc -l`), sha256 prefix `6cbe26dbe4198973`; this section's first line numbers refer to it. This old artifact is not registered for replay. Full E7RESULT lines quoted below come from registered `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out`, regenerated after B4's own control was added and keys renamed to describe operations (e.g. `question=Q1` became `question=today.pick.hidden_root_covered`). Removing the seven B4 control lines and final line yields the same numbers line by line as the old artifact. Parenthesized introductory line numbers refer to the newer artifact. Parts per ten thousand are integers; values below one part per ten thousand become 0, so inspect numerator and denominator.

**Completeness self-check** (command and exact output):

```text
$ sed -E 's/^(E7RESULT name=[a-zA-Z0-9_]+).*/\1/' research/results/e182-unreadable-abandoned-root-forks-single-device.out | sort | uniq -c | sort -rn
   1056 E7RESULT name=ratio
     68 E7RESULT name=anchor
     24 E7RESULT name=counts
     18 E7RESULT name=positive_control
     18 E7RESULT name=history
     10 E7RESULT name=constant_check
      7 E7RESULT name=source_file
      1 E7RESULT name=verdict
      1 E7RESULT name=done
      1 E7RESULT name=config
$ grep 'name=ratio' research/results/e182-unreadable-abandoned-root-forks-single-device.out | sed -E 's/.* scope=([a-z]+) label=.*/\1/' | sort | uniq -c
    264 aggregate
    792 history
$ grep 'name=history ' research/results/e182-unreadable-abandoned-root-forks-single-device.out | sed -E 's/.*history=(hidden-root-confirmed|hidden-root-unconfirmed|no-hidden-root)\..*/\1/' | sort | uniq -c
      6 hidden-root-confirmed
      6 hidden-root-unconfirmed
      6 no-hidden-root
```


There are 18 history rows, 24 counts rows (18 histories plus six combined family × geometry), 18 positive-control rows (one detection control per history), and 1056 ratio rows (24 labels × 44 each: 792 + 264); all three families have six histories. All 68 anchors have `status=holds` (`grep 'name=anchor' … | sed -E 's/.*status=([a-z_]+).*/\1/' | sort | uniq -c` yields `68 holds`). A1–A4, I1, I2 and constant comparisons do not trigger Z7.

**Verdict row** (line 1203; one result from `grep -n 'name=verdict'`):

```text
E7RESULT name=verdict constants_and_anchor_mismatches=0 histories_with_consistency_problems=0 histories_whose_mount_was_refused=0 detection_control_failures=0 arm_witness_control=holds source_files_differing_from_registration=1 path_reached_on_the_default_unconfirmed_histories=true hidden_root_covered_states_on_the_default_geometry=16 shadow_ledger_off_covered_states=not_run first_publish_txg_hits_the_hidden_root_slot_states=1311176
```

No field is `false`; one is `not_run`, and one integer count of violations, mismatches, ambiguity or failures is greater than zero:

| Field | Value | Reason | Expected by registration? |
|---|---|---|---|
| `shadow_ledger_off_covered_states` | `not_run` | The follow-up for a genuinely reached path runs only if “default geometry, hidden-root families, selected and covered total = 0” (12.6). The total is 16, so the condition failed and the follow-up did not run. | Yes |
| `source_files_differing_from_registration` | 1 | Line 29: `crates/singlefs-checker/src/walk.rs` registered `ee5e1663c2241de8`, current `21ee0c2a1c97fb18` (changed by another session). The apparatus does not call `check_pool_image`; segment one uses only three checker decoding functions in `image.rs`, which was not named in the registration and is modified relative to HEAD. Stop condition 4 also requires those behaviors to have changed; whether they did was not reviewed line by line. | **Uncertain**, handed to main agent |

The remaining `constants_and_anchor_mismatches`, `histories_with_consistency_problems`, `histories_whose_mount_was_refused`, `detection_control_failures` are all 0; `arm_witness_control=holds` (line 122: `E7RESULT name=anchor anchor=PC-arms subject=hidden-root-unconfirmed.default.units-64.overwrites-10 status=holds detail=state=segment=2,kept_disk=0_examined=1_failure=none`), and `path_reached_on_the_default_unconfirmed_histories=true`. The names `hidden_root_covered_states_on_the_default_geometry=16` (Z1 trigger count) and `first_publish_txg_hits_the_hidden_root_slot_states=1311176` (Q8 numerators summed over four hidden-root family × geometry groups: 393364 × 2 + 262224 × 2) do not indicate failure by themselves; they are recorded because they determine Z1 and Z4.

**B1: root selection in single-device recovery** (default geometry, summed over each disk separately surviving; “selected and covered” = Q1, “hidden root selected” = Q6): lines 958, 961, 1048, 1051 (registered artifact lines 965, 968, 1055, 1058):

```text
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=aggregate label=hidden-root-confirmed.default.kept_both_disks_summed numerator=8 denominator=786792 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=aggregate label=hidden-root-confirmed.default.kept_both_disks_summed numerator=32 denominator=786792 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed numerator=8 denominator=786792 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed numerator=32 denominator=786792 per_ten_thousand=0
```


A2 (inspect instance generation in system configuration) and A1 (inspect newer instance's records), lines 1052–1058 and no-hidden-root controls 1142–1148 (registered artifact 1059–1065 and 1149–1155):

```text
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed.configuration_instance numerator=393276 denominator=786760 per_ten_thousand=4998
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root.then_none scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed.configuration_instance numerator=393276 denominator=786760 per_ten_thousand=4998
E7RESULT name=ratio question=configuration_instance.on_a_covered_hidden_root.then_legal scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed.configuration_instance numerator=0 denominator=8 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.avoided_a_legal_root scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed.newer_instance_record numerator=0 denominator=786760 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.on_a_covered_hidden_root.then_legal scope=aggregate label=hidden-root-unconfirmed.default.kept_both_disks_summed.newer_instance_record numerator=4 denominator=8 per_ten_thousand=5000
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root scope=aggregate label=no-hidden-root.default.kept_both_disks_summed.configuration_instance numerator=327758 denominator=655708 per_ten_thousand=4998
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root.then_none scope=aggregate label=no-hidden-root.default.kept_both_disks_summed.configuration_instance numerator=327758 denominator=655708 per_ten_thousand=4998
E7RESULT name=ratio question=configuration_instance.on_a_covered_hidden_root.then_legal scope=aggregate label=no-hidden-root.default.kept_both_disks_summed.configuration_instance numerator=0 denominator=0 per_ten_thousand=none
E7RESULT name=ratio question=newer_instance_record.avoided_a_legal_root scope=aggregate label=no-hidden-root.default.kept_both_disks_summed.newer_instance_record numerator=0 denominator=655708 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.on_a_covered_hidden_root.then_legal scope=aggregate label=no-hidden-root.default.kept_both_disks_summed.newer_instance_record numerator=0 denominator=0 per_ten_thousand=none
```


Readings use the weakest interpretation for each criterion: they establish the behavior only for these starting points and the two tiers.

| Question | Reading | Failure clause |
|---|---|---|
| Q1, cost of C on default geometry | Each tier has eight (state, surviving disk) pairs that select an abandoned root which is covered, denominator 786792. Both tiers total 16, the verdict's `hidden_root_covered_states_on_the_default_geometry=16`. All 16 are in s=1 histories (two histories per tier, four each); s=64 has zero (see s axis). | Z1 triggers (default total >0). |
| Q2 | No root selected: 0 (all `Q2_no_root` numerators 0). Root selected but file read fails: eight, equal to Q1. | No clause; report only. |
| Q6, path actually reached | Each tier selects the hidden root in 32 (state, surviving disk) pairs: eight covered, 24 not. Narrow geometry with disk 1 kept has Q6 = 0 (lines 994 and 1084, label `….narrow.kept_1`). | Z8 does not trigger (Q6 >0). |
| Q3, A2 wrongly avoids legal roots | In 49.98% of legal single-device recoveries (393276 / 786760), it avoids the legal root selected today; all then select no root (`Q3_then_no_root` numerator equals Q3). The no-hidden-root control family is 49.98% (327758 / 655708). Surviving disk 0 and 1 are asymmetric for hidden-root families: default about 16.7% / 83.3%, narrow 0 / about 99.97%; unconfirmed lines 1028, 1040, 1073, 1085 have 65548 / 393372, 327728 / 393388, 0 / 262224, 262180 / 262240. | Z2 triggers (Q3 >0). |
| Q4, A2 repairs covered roots | 0 / 8; A2 avoids all eight states selecting a covered root, and then finds no root in all (`configuration_instance.kept0.on_a_covered_hidden_root.then_none=4`, likewise four with disk 1). | Z3 triggers (repair fraction <1). |
| Q5, A1 | Wrongly avoids legal root 0 / 786760; repairs 4 / 8 = 50%. The other four still select a covered hidden root because the new instance's record did not land on the surviving disk. | Z3 triggers (repair fraction <1). |

**B4: avoid the txg corresponding to the unreadable root slot on the new instance's first publication** (without changing implementation; classify states after instance 2's row-publication root-slot write becomes durable). Default geometry, lines 970–977 and 1060–1067 (registered artifact 977–984 and 1067–1074):

```text
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=aggregate label=hidden-root-confirmed.default numerator=196634 denominator=393364 per_ten_thousand=4998
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=aggregate label=hidden-root-confirmed.default numerator=196730 denominator=393364 per_ten_thousand=5001
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=aggregate label=hidden-root-confirmed.default numerator=0 denominator=393364 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=aggregate label=hidden-root-confirmed.default numerator=393364 denominator=393364 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_confirmed_none_covered scope=aggregate label=hidden-root-confirmed.default numerator=196634 denominator=393364 per_ten_thousand=4998
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_some_unit_covered scope=aggregate label=hidden-root-confirmed.default numerator=196730 denominator=393364 per_ten_thousand=5001
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_unconfirmed_none_covered scope=aggregate label=hidden-root-confirmed.default numerator=0 denominator=393364 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=aggregate label=hidden-root-confirmed.default numerator=393364 denominator=393364 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=aggregate label=hidden-root-unconfirmed.default numerator=0 denominator=393364 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=aggregate label=hidden-root-unconfirmed.default numerator=196730 denominator=393364 per_ten_thousand=5001
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=aggregate label=hidden-root-unconfirmed.default numerator=196634 denominator=393364 per_ten_thousand=4998
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=aggregate label=hidden-root-unconfirmed.default numerator=393364 denominator=393364 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_confirmed_none_covered scope=aggregate label=hidden-root-unconfirmed.default numerator=0 denominator=393364 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_some_unit_covered scope=aggregate label=hidden-root-unconfirmed.default numerator=196730 denominator=393364 per_ten_thousand=5001
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_unconfirmed_none_covered scope=aggregate label=hidden-root-unconfirmed.default numerator=196634 denominator=393364 per_ten_thousand=4998
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=aggregate label=hidden-root-unconfirmed.default numerator=393364 denominator=393364 per_ten_thousand=10000
```

The three Q7 rows are ordered class a, b, c. Q8 = 10000 / 10000: on both default and narrow geometries, every state after row publication becomes durable in the two hidden-root families places today's first-publication txg in the hidden root's slot (A3 holds: instance 2 obtains the same txg as the hidden root), and the hidden root's slot is overwritten (`B4_hidden_root_slot_overwritten` is also 10000 / 10000). Z4 (tightening changes no state in these histories) does not trigger. Class a / b / c shares vary greatly with s; see the s axis below.

**The s axis** (amendment 12.4: overwrite uses `publish_overwrite`, so the hidden root references one data unit; s affects only the preceding release ledger). Under default geometry and the unconfirmed tier, Q1, Q6, Q7, Q8, `B4_hidden_root_slot_overwritten`, and history rows for s=1 (p=5) and s=64 (p=10) are old lines 148, 151, 160–163, 167 and 32; 553, 556, 565–568, 572 and 77; registered-artifact lines 155, 158, 167–170, 174 and 32; and 560, 563, 572–575, 579 and 80:

```text
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5 numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5 numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5 numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5 numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=history label=hidden-root-unconfirmed.default.units-1.overwrites-5 numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10.kept_both_disks_summed numerator=0 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10 numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10 numerator=24 denominator=131112 per_ten_thousand=1
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10 numerator=131088 denominator=131112 per_ten_thousand=9998
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10 numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=history label=hidden-root-unconfirmed.default.units-64.overwrites-10 numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=history history=hidden-root-unconfirmed.default.units-1.overwrites-5 hidden_root=1:8 fault_ranges={"journal_record":_2,_"root_slot":_1,_"unit":_24} segments=25 writes=162 states_closed_form=131120 expansions={"final_state":_1,_"full_subsets":_21,_"quick_in_place_subsets_times_unit_writes_none_or_all":_4} mount_refused=None mount_stopped_after=None instance=2 first_publish_root=2:8 warm_up_publishes=2 shadow_ledger_branch=shadow_ledger=on isolated_slots_per_device=[(0,_0),_(1,_0)] abandoned_roots_unreadable=0 core_ring_slots_holding_an_abandoned_root=[] reads_intercepted={"journal_record":_2,_"root_slot":_8,_"unit":_0} ranges_written_over={"journal_record":_2,_"root_slot":_1,_"unit":_20} consistency_problems=[]
E7RESULT name=history history=hidden-root-unconfirmed.default.units-64.overwrites-10 hidden_root=1:14 fault_ranges={"journal_record":_2,_"root_slot":_1,_"unit":_24} segments=25 writes=158 states_closed_form=131120 expansions={"final_state":_1,_"full_subsets":_21,_"quick_in_place_subsets_times_unit_writes_none_or_all":_4} mount_refused=None mount_stopped_after=None instance=2 first_publish_root=2:14 warm_up_publishes=2 shadow_ledger_branch=shadow_ledger=on isolated_slots_per_device=[(0,_0),_(1,_0)] abandoned_roots_unreadable=0 core_ring_slots_holding_an_abandoned_root=[] reads_intercepted={"journal_record":_2,_"root_slot":_8,_"unit":_0} ranges_written_over={"journal_record":_2,_"root_slot":_1,_"unit":_2} consistency_problems=[]
```


In s=1 histories Q1 is four (disks 0 and 1 summed) and every post-row state is class b (a unit referenced by the hidden root has been covered). For s=64 Q1 is zero, class b only 24 states (24 / 131112 for p=10 and 24 / 65570 for p=5, two to four parts per ten thousand); the rest are class c in the unconfirmed tier and class a in the confirmed tier. Of the 24 unit ranges returning zero, the number overwritten by instance 2 also depends on s: 20 for s=1 and two for s=64. These differences do not arise from the number of units referenced by the hidden root: under amendment 12.4 both reference one data unit. Their cause was not measured. One unverified inference is that for s=64, units released during the first-file stage enter the defer queue and influence where instance 2's writes land. “Overwrite retains s units” makes the hidden root reference s data units; the supplemental artifact answers this axis: class b is 100% in histories retaining s=64 (versus two to four parts per ten thousand without retention), and Q1 becomes nonzero at (s=64, p=10).

**Two fork-sheet rows** (each cell points to commands; `F` denotes segment-one `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out` or supplemental `research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out`, each run separately):

| Fork | Sufficient evidence / remaining gaps | Can remaining measurements reverse it? | Commands |
|---|---|---|---|
| B1: hidden-root record also unreadable, single-device recovery selects it | **Sufficient**: fork-sheet sufficiency asks for both proportions measured once in the committed apparatus, same starting points, all crash states. The s axis covers segment one's s=1 and s=64 one-unit overwrites and supplemental s=64 retained. C Q1 per default tier: segment one 8 / 786792 (all s=1), supplemental 4 / 393396 (all (s=64,p=10), disk 0 retained): **Q1 is nonzero**. Supplemental both tiers total eight, verdict `hidden_root_covered_states_on_the_default_geometry=8`, so Z1 also triggers there. A2 Q3 is 49.98% for all three shapes (supplemental 196642 / 393384; segment one 393276 / 786760), independent of s; Q4 supplemental 0 / 4, segment one 0 / 8. A1 repairs 2 / 4 and 4 / 8 respectively and wrongly avoids zero legal roots. Not covered: instance 2's post-mount overwrites still have one unit, p only 5 and 10, two states only, and two disks only (see limitations). | Registration leaves no B1 measurement: the shadow-ledger-off follow-up condition (default hidden-root-family total Q1 = 0) fails in both artifacts, so `not_run` is expected. Unregistered shapes such as post-mount overwrites retaining s, other p or a third state can change Q1's absolute count. Q1 >0 and A2 49.98% have already appeared; further measurement is the main agent's decision. | `grep 'name=ratio question=Q1 scope=aggregate' F`; `grep 'name=ratio question=Q1 scope=history' F | grep kept_both` (by starting point); `grep 'name=ratio question=Q3 scope=aggregate' F`; `grep 'name=ratio question=Q4 scope=aggregate' F`; `grep 'name=ratio question=Q5_repaired ' F`; Q6 similarly; `grep -n 'name=verdict' F`. |
| B4: tighten first publication to avoid unreadable-root-slot txg | **Sufficient**: measure each of the same starting points once in the committed apparatus. Q8 = 100% (every hidden-root family on default/narrow geometry in both artifacts is 10000 / 10000), so Z4 does not trigger. Class a/b share varies with history shape: s=64 one-unit overwrite has 24 class-b states (two to four parts per ten thousand), and confirmed-tier class a is 99.96%–99.98%; s=1 and supplemental retained s=64 have all post-row states in class b and empty class a (supplemental `b4.class_b` equals `b4.states`, with no `b4.class_a`; see supplemental comparison table). B4's own positive control: in regenerated segment-one output three s=64 histories hold and three s=1 are `not_made`; all three supplemental histories are `not_made` (summary `trials=0 status=not_made`), not invalid under 12.12. This is not a selection defect: selector and counter use the same states and classifier; class a is empty for this shape (supplemental items 1–4). | Registration leaves no B4 measurement. “Class a becomes class b” was controlled only in three s=64 one-unit-overwrite histories; the six confirmed-tier s=1 and supplemental histories could not be tested. Supplemental class b = 100% relies on the same classifier and nine `detect_covered` controls all holding. Unregistered shapes may change a/b shares, but both extremes (two parts per ten thousand and 100%) were measured. | `grep 'name=ratio question=Q7 scope=aggregate' F`; `grep 'question=Q8' F`; `grep -o 'b4\.[a-z_.]*=[0-9]*' F`; `grep 'control=b4_class_a_to_class_b' F`; `grep -n 'name=verdict' F`. |

### B4's own positive control (last sentence of registration 5.2; amendment 12.12)

Artifact `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out`: 1211 lines, sha256 prefix `8d8ede1c3d7bfb51`, keys renamed to describe methods; prior to renaming the prefix was `f35161e60edc4410`. For each starting point choose two class-a states, earliest in segment order and latest in reverse order, change one byte of a unit referenced by the hidden root on disk 0, and require reclassification to class b. Run only confirmed-tier hidden-root histories, the only family containing class a: six histories each have a row at lines 42, 58, 74, 90, 106, 122; summary at line 128. Complete lines:

```text
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=hidden-root-confirmed.default.units-1.overwrites-5 class_confirmed_none_covered_states_chosen=0 trials=0 class_confirmed_none_covered_before_the_change=0 changed_to_class_some_unit_covered=0 no_unit_to_change=0 status=not_made
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=hidden-root-confirmed.default.units-1.overwrites-10 class_confirmed_none_covered_states_chosen=0 trials=0 class_confirmed_none_covered_before_the_change=0 changed_to_class_some_unit_covered=0 no_unit_to_change=0 status=not_made
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=hidden-root-confirmed.default.units-64.overwrites-5 class_confirmed_none_covered_states_chosen=2 trials=2 class_confirmed_none_covered_before_the_change=2 changed_to_class_some_unit_covered=2 no_unit_to_change=0 status=holds
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=hidden-root-confirmed.default.units-64.overwrites-10 class_confirmed_none_covered_states_chosen=2 trials=2 class_confirmed_none_covered_before_the_change=2 changed_to_class_some_unit_covered=2 no_unit_to_change=0 status=holds
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=hidden-root-confirmed.narrow.units-64.overwrites-10 class_confirmed_none_covered_states_chosen=2 trials=2 class_confirmed_none_covered_before_the_change=2 changed_to_class_some_unit_covered=2 no_unit_to_change=0 status=holds
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=hidden-root-confirmed.narrow.units-1.overwrites-5 class_confirmed_none_covered_states_chosen=0 trials=0 class_confirmed_none_covered_before_the_change=0 changed_to_class_some_unit_covered=0 no_unit_to_change=0 status=not_made
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered_total histories=6 histories_not_made=3 histories_violated=0 trials=6 changed_to_class_some_unit_covered=6 status=holds
```


Completeness self-check (command and exact output):

```text
$ sed -E 's/^(E7RESULT name=[a-zA-Z0-9_]+).*/\1/' research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out | sort | uniq -c | sort -rn
   1056 E7RESULT name=ratio
     68 E7RESULT name=anchor
     25 E7RESULT name=positive_control
     24 E7RESULT name=counts
     18 E7RESULT name=history
     10 E7RESULT name=constant_check
      7 E7RESULT name=source_file
      1 E7RESULT name=verdict
      1 E7RESULT name=done
      1 E7RESULT name=config
$ grep -c 'control=b4_class_a_to_class_b' research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out
7
```


There are 25 positive-control rows: 18 detection controls plus seven B4 controls. Other row-type counts match the old artifact. The only differences from it are these seven rows and the last line: `diff <(grep -v 'control=b4_class_a_to_class_b' research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out | grep -v 'name=done') <(grep -v 'name=done' research/results/e182-unreadable-abandoned-root-forks-single-device.out)` produces no output. Verdict lines (new line 1210, old line 1203) are verbatim identical, `detection_control_failures=0`.

Readings for six confirmed-tier hidden-root histories:

| History | Class-a states chosen | Still class a before change | Class b after change | Status |
|---|---|---|---|---|
| `hidden-root-confirmed.default.units-1.overwrites-5` (line 42) | 0 | 0 | 0 | `not_made` |
| `hidden-root-confirmed.default.units-1.overwrites-10` (line 58) | 0 | 0 | 0 | `not_made` |
| `hidden-root-confirmed.default.units-64.overwrites-5` (line 74) | 2 | 2 | 2 | `holds` |
| `hidden-root-confirmed.default.units-64.overwrites-10` (line 90) | 2 | 2 | 2 | `holds` |
| `hidden-root-confirmed.narrow.units-64.overwrites-10` (line 106) | 2 | 2 | 2 | `holds` |
| `hidden-root-confirmed.narrow.units-1.overwrites-5` (line 122) | 0 | 0 | 0 | `not_made` |

Summary: six histories, three not made (`histories_not_made=3`), none violated, all six trials changed class a to b, `status=holds`. “Holds” means every trial actually made changed class, not that all six histories passed: the three s=1 histories made no trial. They cannot make class-a states because existing segment-one counts show `b4.class_b = b4.states` and no `b4.class_a` in each (command and exact output):

```text
$ grep 'name=counts ' research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out | grep 'history=hidden-root-confirmed\.[a-z]*\.units-1\.' | grep -o 'history=[^ ]*\|b4\.class_a=[0-9]*\|b4\.class_b=[0-9]*\|b4\.states=[0-9]*'
history=hidden-root-confirmed.default.units-1.overwrites-5
b4.class_b=131112
b4.states=131112
history=hidden-root-confirmed.default.units-1.overwrites-10
b4.class_b=65570
b4.states=65570
history=hidden-root-confirmed.narrow.units-1.overwrites-5
b4.class_b=131112
b4.states=131112
```


The single verdict line (`grep -n 'name=verdict' research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out`, line 1210) has no `false`; one `not_run` (`shadow_ledger_off_covered_states`, as in the old artifact, expected); one nonzero integer among violation, mismatch, ambiguity and failure counts (`source_files_differing_from_registration=1`, unchanged, uncertain and handed to main agent). The B4 control adds no verdict field, only its failure count to `detection_control_failures`, unchanged when all pass.

### Supplemental-arm results (overwrite retains s units, interpretation under amendment 12.11)

Artifact `research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out`: 758 lines, sha256 prefix `dc375a17d7645dc1` after keys were renamed to describe methods (previously `06290cf9ab907930`), final `E7RESULT name=done emitted=758`. Nine histories: three s=64 starting points (default p=5, p=10; narrow p=10) × three families, names prefixed `units-kept.`. The long-run script reported “judged now 214, reused 0”; sentinel rejudged zero shards, and the last progress-log shard took 3413.5 seconds (under `~/.local/share/singlefs/work/e182-long-run/`, not committed). Full E7RESULT rows quoted here come from this artifact, with line numbers in introductions. Parts per ten thousand are integers; values below one part per ten thousand show 0, so inspect numerator and denominator.

**Completeness self-check** (command and exact output):

```text
$ sed -E 's/^(E7RESULT name=[a-zA-Z0-9_]+).*/\1/' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | sort | uniq -c | sort -rn
    660 E7RESULT name=ratio
     41 E7RESULT name=anchor
     15 E7RESULT name=counts
     13 E7RESULT name=positive_control
     10 E7RESULT name=constant_check
      9 E7RESULT name=history
      7 E7RESULT name=source_file
      1 E7RESULT name=verdict
      1 E7RESULT name=done
      1 E7RESULT name=config
$ grep 'name=ratio' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | sed -E 's/.* scope=([a-z]+) label=.*/\1/' | sort | uniq -c
    264 aggregate
    396 history
$ grep 'name=history ' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | sed -E 's/.*history=units-kept\.(hidden-root-confirmed|hidden-root-unconfirmed|no-hidden-root)\..*/\1/' | sort | uniq -c
      3 hidden-root-confirmed
      3 hidden-root-unconfirmed
      3 no-hidden-root
$ grep 'name=anchor' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | sed -E 's/.*status=([a-z_]+).*/\1/' | sort | uniq -c
     41 holds
```

Nine history, 15 counts (nine histories plus six family × geometry totals), 13 positive-control (nine detection, three B4, one B4 summary), and 660 ratio rows (15 labels × 44) are present; three histories in each family. All 41 anchors hold (nine × three plus 14 static; segment one had 18 × three plus the same 14). A1–A4, I1, I2 and constant comparison do not trigger Z7.

**Verdict row** (line 757; one `grep -n 'name=verdict'`):

```text
E7RESULT name=verdict constants_and_anchor_mismatches=0 histories_with_consistency_problems=0 histories_whose_mount_was_refused=0 detection_control_failures=1 arm_witness_control=holds source_files_differing_from_registration=1 path_reached_on_the_default_unconfirmed_histories=true hidden_root_covered_states_on_the_default_geometry=8 shadow_ledger_off_covered_states=not_run first_publish_txg_hits_the_hidden_root_slot_states=655588
```


Field-by-field identification: `grep -n 'name=verdict' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | grep -o '[a-z_]*=false\|[a-z_]*=not_run'` produces only:

```text
shadow_ledger_off_covered_states=not_run
```


There are no `false` fields, one `not_run`, and two nonzero integer counts among violations, mismatches, ambiguities and failures:

| Field | Value | Reason | Expected? |
|---|---|---|---|
| `detection_control_failures` | 1 | The apparatus adds B4 control failures: violated-history count plus one if total trials in the summary are zero (`ClassAControlTotal::failures`, line 3022, added to `detection_failures` after `emit_the_class_a_control_total`). Here `histories=3 histories_not_made=3 histories_violated=0 trials=0`, so 0 + 1 = 1. All nine `control=detect_covered` hold (commands below). This is the sole source, not a failed detection control. | The interpretation is registered in amendment 12.12: no class-a state means `not_made`, not invalid; verdict failure count adds one when the summary is not holds because trials=0. Value 1 directly follows the absence of class a in this family. The third executor's inference “all three supplemental histories have s=64, so should be possible” did not hold. |
| `source_files_differing_from_registration` | 1 | Line 29 identifies `crates/singlefs-checker/src/walk.rs`, registered `ee5e1663c2241de8`, actual `21ee0c2a1c97fb18`, the same line and value in both segment-one artifacts. `report_the_source_file_fingerprints` (from line 2694) reads the first 16 sha256 characters for seven registered files, counts mismatches and reports without stopping; the other six are `same=true` (lines 25–28, 30, 31). | Not expected in registration. The named walk.rs is outside the apparatus call path (below); the `image` module and `lib.rs` actually used have no registered fingerprints and are modified relative to HEAD. Their behavior was not reviewed line by line. Stop clause 4 requires those behaviors also to have changed, so execution does not stop; uncertainty is handed to the main agent. |
| `shadow_ledger_off_covered_states` | `not_run` | Follow-up for a genuinely reached path runs only if default-geometry hidden-root families total zero selected-and-covered states (12.6). Here the total is eight, so it did not run. | Yes. |

`hidden_root_covered_states_on_the_default_geometry=8` is the Z1 trigger count and `first_publish_txg_hits_the_hidden_root_slot_states=655588` sums Q8 numerators for four hidden-root family × geometry groups (196682 for each default tier, 131112 for each narrow tier). Their names do not themselves indicate a failure; the values determine Z1 and Z4.

```text
$ grep 'control=detect_covered' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | sed -E 's/.*status=([a-z_]+).*/\1/' | sort | uniq -c
      9 holds
$ grep -n 'path=crates/singlefs-checker/src/walk.rs' research/results/e182-unreadable-abandoned-root-forks-single-device.out research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out
research/results/e182-unreadable-abandoned-root-forks-single-device.out:29:E7RESULT name=source_file path=crates/singlefs-checker/src/walk.rs registered=ee5e1663c2241de8 actual=21ee0c2a1c97fb18 same=false
research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out:29:E7RESULT name=source_file path=crates/singlefs-checker/src/walk.rs registered=ee5e1663c2241de8 actual=21ee0c2a1c97fb18 same=false
research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out:29:E7RESULT name=source_file path=crates/singlefs-checker/src/walk.rs registered=ee5e1663c2241de8 actual=21ee0c2a1c97fb18 same=false
$ grep -c 'check_pool_image' crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs; grep -n '^use singlefs_checker' crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs
0
49:use singlefs_checker::image as checker_image;
50:use singlefs_checker::image::{ImageReader, PoolGeometry};
$ grep -n 'walk::' crates/singlefs-checker/src/image.rs | grep -vc '^[0-9]*:[[:space:]]*///'
0
```


The apparatus imports only `singlefs_checker::image` (`valid_roots`, `root_slot_positions`, `chosen_system_configurations`), not `check_pool_image`; `walk::` in `image.rs` occurs only in documentation comments (last command counts zero non-comment lines). Relative to HEAD, walk.rs has 6138 added and 559 deleted lines staged by another session (`git diff HEAD --stat -- crates/singlefs-checker/src/walk.rs`), so the mismatch names a file not called by the apparatus. `image.rs` and `lib.rs` (`image.rs` calls `check_root_slot` and `check_system_configuration_slot`) are modified relative to HEAD, with 159 and 226 changed lines respectively, and are not among the seven registered files.

**B4's own positive control (amendment 12.12) was not made in the supplemental artifact:** one line for each of three confirmed-tier hidden-root histories at 42, 58, 74 and summary line 80, copied in full:

```text
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=units-kept.hidden-root-confirmed.default.units-64.overwrites-5 class_confirmed_none_covered_states_chosen=0 trials=0 class_confirmed_none_covered_before_the_change=0 changed_to_class_some_unit_covered=0 no_unit_to_change=0 status=not_made
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=units-kept.hidden-root-confirmed.default.units-64.overwrites-10 class_confirmed_none_covered_states_chosen=0 trials=0 class_confirmed_none_covered_before_the_change=0 changed_to_class_some_unit_covered=0 no_unit_to_change=0 status=not_made
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered history=units-kept.hidden-root-confirmed.narrow.units-64.overwrites-10 class_confirmed_none_covered_states_chosen=0 trials=0 class_confirmed_none_covered_before_the_change=0 changed_to_class_some_unit_covered=0 no_unit_to_change=0 status=not_made
E7RESULT name=positive_control control=class_confirmed_none_covered_to_class_some_unit_covered_total histories=3 histories_not_made=3 histories_violated=0 trials=0 changed_to_class_some_unit_covered=0 status=not_made
```


All three have `class_a_states_chosen=0 trials=0`; summary `histories_not_made=3 trials=0 status=not_made`, `histories_violated=0`. None was “tried but failed to reclassify.” Under registration 12.12, failure to select class a is `not_made`, neither invalid nor passed. It contributes one to verdict `detection_control_failures`, as detailed above.

**A selector defect, or no class a in this shape?** The latter. Evidence below reads apparatus source and artifact; the apparatus was not rerun:

```text
$ grep 'name=counts ' research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out | grep 'history=units-kept\.hidden-root-confirmed\.[a-z]*\.units-64' | grep -o 'history=[^ ]*\|b4\.class_a=[0-9]*\|b4\.class_b=[0-9]*\|b4\.states=[0-9]*'
history=units-kept.hidden-root-confirmed.default.units-64.overwrites-5
b4.class_b=65570
b4.states=65570
history=units-kept.hidden-root-confirmed.default.units-64.overwrites-10
b4.class_b=131112
b4.states=131112
history=units-kept.hidden-root-confirmed.narrow.units-64.overwrites-10
b4.class_b=131112
b4.states=131112
```

1. Selection (`first_class_a_state_in_the_order`, from line 2898; `class_a_states_for_the_control`, from 2933) iterates segments from 0, expands every state with `for_each_state_of_segment`, and takes the first where the row-publication root-slot write is durable (`persisted[row_publish_write_index]`) and `class_of_the_hidden_root_in_the_state` returns `class_a`; then it takes one in reverse order. No sampling or truncation.
2. B4 counting (`count_the_hidden_root_classification`, from line 1870) enumerates the same states using the same `for_each_state_of_segment`, the same `persisted[row_write_index]` condition, and the same classifier, producing `b4.states` and `b4.class_*`. Thus no selected state is equivalent to no class a in the counts. All three confirmed supplemental histories have `b4.class_b = b4.states`, no `b4.class_a`; the selector has no opportunity to miss class a.
3. The reason is the states themselves: after row publication, at least one unit referenced by the hidden root is covered in every state (the classifier compares both disks byte-by-byte). Even the minimum state “row-publication root-slot write durable, previous segments all durable” is class b, so a prior segment already covered such a unit. Direct corroboration: Q1 = 4 at (s=64,p=10), states where the hidden-root slot remains in the ring, single-device recovery selects it, and its unit has been covered. Those can only occur before the row-publication root-slot write. At the same starting point without retention, the root references only one data unit; only 24 post-publication states are class b and confirmed-tier class a accounts for 99.96%–99.98% (three-shape comparison below).
4. Mechanism is inferred, not measured: the hidden root slot is unreadable, so instance 2's allocator rebuilds from an older ledger; the s data units and metadata units newly occupied by the hidden root appear free, and row-publication new units take the lowest free slots, landing there. The apparatus did not identify which row-publication unit write covered which slot, nor inspect allocator slot-selection order.
5. The third executor's expectation that all three s=64 supplemental histories could make class a was extrapolated from s=64 without retention, where three controls hold. The supplemental shape makes the hidden root reference s units and row publication takes their occupied slots, eliminating class a.

Thus this is not an apparatus defect and requires no apparatus change. The class-a-to-b direction remains untested by this control in three s=1 and three supplemental confirmed histories; it holds only in three s=64 one-unit-overwrite histories (registered artifact lines 74, 90, 106). Supplemental class b = 100% rests on the same shape-independent classifier and all nine `detect_covered` controls holding, not on the B4 control.

**B1: single-device recovery root selection** (nine supplemental histories; Q1 = selected and covered, Q6 = hidden root selected, and Q2 selected root with file-read failure equals Q1). Default and narrow geometry, summed over the two independently surviving disks (lines 512, 514, 515, 557, 560, 602, 604, 605, 647, 650):

```text
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed numerator=4 denominator=393396 per_ten_thousand=0
E7RESULT name=ratio question=today.outcome.failed_with_root scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed numerator=4 denominator=393396 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed numerator=12 denominator=393396 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=aggregate label=units-kept.hidden-root-confirmed.narrow.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed numerator=4 denominator=393396 per_ten_thousand=0
E7RESULT name=ratio question=today.outcome.failed_with_root scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed numerator=4 denominator=393396 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed numerator=12 denominator=393396 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
```


Split by starting point: lines 107, 110, 152, 155 for p=5; 242, 245, 287, 290 for default p=10; 377, 380, 422, 425 for narrow p=10:

```text
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=units-kept.hidden-root-unconfirmed.default.units-64.overwrites-5.kept_both_disks_summed numerator=0 denominator=131156 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=units-kept.hidden-root-unconfirmed.default.units-64.overwrites-5.kept_both_disks_summed numerator=4 denominator=131156 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=units-kept.hidden-root-confirmed.default.units-64.overwrites-5.kept_both_disks_summed numerator=0 denominator=131156 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=units-kept.hidden-root-confirmed.default.units-64.overwrites-5.kept_both_disks_summed numerator=4 denominator=131156 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=units-kept.hidden-root-unconfirmed.default.units-64.overwrites-10.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=units-kept.hidden-root-unconfirmed.default.units-64.overwrites-10.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=units-kept.hidden-root-confirmed.default.units-64.overwrites-10.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=units-kept.hidden-root-confirmed.default.units-64.overwrites-10.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=units-kept.hidden-root-unconfirmed.narrow.units-64.overwrites-10.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=units-kept.hidden-root-unconfirmed.narrow.units-64.overwrites-10.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root_covered scope=history label=units-kept.hidden-root-confirmed.narrow.units-64.overwrites-10.kept_both_disks_summed numerator=4 denominator=262240 per_ten_thousand=0
E7RESULT name=ratio question=today.pick.hidden_root scope=history label=units-kept.hidden-root-confirmed.narrow.units-64.overwrites-10.kept_both_disks_summed numerator=8 denominator=262240 per_ten_thousand=0
```


A2 (`configuration_instance`) and A1 (`newer_instance_record`), one default-geometry group each for confirmed and unconfirmed tiers (lines 516–523 and 606–613), and two default no-hidden-root control rows (696, 700):

```text
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed.configuration_instance numerator=196642 denominator=393384 per_ten_thousand=4998
E7RESULT name=ratio question=configuration_instance.on_a_covered_hidden_root.then_legal scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed.configuration_instance numerator=0 denominator=4 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.avoided_a_legal_root scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed.newer_instance_record numerator=0 denominator=393384 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.on_a_covered_hidden_root.then_legal scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed.newer_instance_record numerator=2 denominator=4 per_ten_thousand=5000
E7RESULT name=ratio question=newer_instance_record.on_a_covered_hidden_root.then_hidden_root_covered scope=aggregate label=units-kept.hidden-root-confirmed.default.kept_both_disks_summed.newer_instance_record numerator=2 denominator=4 per_ten_thousand=5000
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed.configuration_instance numerator=196642 denominator=393384 per_ten_thousand=4998
E7RESULT name=ratio question=configuration_instance.on_a_covered_hidden_root.then_legal scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed.configuration_instance numerator=0 denominator=4 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.avoided_a_legal_root scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed.newer_instance_record numerator=0 denominator=393384 per_ten_thousand=0
E7RESULT name=ratio question=newer_instance_record.on_a_covered_hidden_root.then_legal scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed.newer_instance_record numerator=2 denominator=4 per_ten_thousand=5000
E7RESULT name=ratio question=newer_instance_record.on_a_covered_hidden_root.then_hidden_root_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default.kept_both_disks_summed.newer_instance_record numerator=2 denominator=4 per_ten_thousand=5000
E7RESULT name=ratio question=configuration_instance.avoided_a_legal_root scope=aggregate label=units-kept.no-hidden-root.default.kept_both_disks_summed.configuration_instance numerator=131108 denominator=262312 per_ten_thousand=4998
E7RESULT name=ratio question=newer_instance_record.avoided_a_legal_root scope=aggregate label=units-kept.no-hidden-root.default.kept_both_disks_summed.newer_instance_record numerator=0 denominator=262312 per_ten_thousand=0
```


Readings use the weakest interpretation: only these starting points and two tiers are established.

| Question | Reading | Failure clause |
|---|---|---|
| Q1: C cost at default | Each tier 4 / 393396, total 8 (`hidden_root_covered_states_on_the_default_geometry=8`), all in (s=64,p=10), four per tier out of 262240 with disk 0 kept; p=5 is 0 / 131156. Narrow (64,10) is 4 / 262240 per tier; no-hidden-root control is 0. Segment one without retention at the same (64,10) had 0. | Z1 also triggers in supplemental (default total >0). **Q1 becomes nonzero:** retaining s units produces selected-and-covered abandoned roots at s=64, at the same scale as segment one's s=1, one per 98349 states (8 / 786792 versus 4 / 393396). |
| Q2 | No root selected 0; selected root but file read fails 4, equal to Q1. | No clause; report only. |
| Q6 path reached | Default each tier 12 / 393396 (p=5 four, p=10 eight); narrow each tier 8 / 262240. | Z8 does not trigger (Q6 >0). |
| Q3 A2 wrongly avoids legal roots | Confirmed default 196642 / 393384 (49.98%), narrow 131090 / 262232 (49.99%); unconfirmed default also 196642 / 393384; default no-hidden-root control 131108 / 262312 (49.98%). All three shapes (s=1, s=64 without retention, s=64 retained) match and do not depend on s. | Z2 triggers (Q3 >0). |
| Q4 A2 repairs covered | 0 / 4 per tier and geometry. | Z3 triggers (repair fraction <1). |
| Q5 A1 | Wrongly avoids legal roots 0 / 393384; repairs 2 / 4 = 50%, the other two still select the covered hidden root (`Q5_repaired_still_hidden_root_covered`). Segment one 4 / 8, also 50%. | Z3 triggers (repair fraction <1). |

**B4: avoid the unreadable-root-slot txg in the new instance's first publication** (six supplemental hidden-root histories; implementation unchanged, state classification; Q7 rows are class a, b, c in order). Default confirmed and unconfirmed tiers, lines 524–531 and 614–621:

```text
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=aggregate label=units-kept.hidden-root-confirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=0 denominator=196682 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=aggregate label=units-kept.hidden-root-unconfirmed.default numerator=196682 denominator=196682 per_ten_thousand=10000
```

Narrow geometry, confirmed and unconfirmed tiers, lines 569–576 and 659–666:

```text
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=aggregate label=units-kept.hidden-root-confirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_confirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_some_unit_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
E7RESULT name=ratio question=after_the_row_publish.first_publish_txg_lands_on_the_hidden_root_slot.class_unconfirmed_none_covered scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=0 denominator=131112 per_ten_thousand=0
E7RESULT name=ratio question=after_the_row_publish.hidden_root_slot_overwritten scope=aggregate label=units-kept.hidden-root-unconfirmed.narrow numerator=131112 denominator=131112 per_ten_thousand=10000
```

All four family × geometry groups have the same shape: class a and c are zero; class b accounts for 100% of post-row-publication states. Q8 = 10000 / 10000: today's first-publication txg lands in the hidden root's slot in every state and its root slot is overwritten; `B4_hidden_root_slot_overwritten` is also 10000 / 10000. Z4 does not trigger.

The three shapes together (confirmed tier, post-row-publication states; Q7's first two rows are class a and b, Q1 is `kept_both_disks_summed`). The first three data columns' line numbers refer to `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out`, the last three to the supplemental artifact:

| Starting point (confirmed tier) | Overwrite one unit: class a | Overwrite one unit: class b | Overwrite one unit: Q1 | Overwrite retaining s units: class a | Overwrite retaining s units: class b | Overwrite retaining s units: Q1 |
|---|---|---|---|---|---|---|
| Default (s,p)=(1,5) | 0 / 131112 (line 212) | 131112 / 131112 (213) | 4 / 262240 (200) | At s=1 both shapes identical; not separately built | Same | Same |
| Default (1, 10) | 0 / 65570 (line 347) | 65570 / 65570 (line 348) | 4 / 131156 (line 335) | At s = 1 both shapes are identical, not separately built | At s = 1 both shapes are identical, not separately built | At s = 1 both shapes are identical, not separately built |
| Default (64,5) | 65546 / 65570 (482) | 24 / 65570 (483) | 0 / 131156 (470) | 0 / 65570 (164) | 65570 / 65570 (165) | 0 / 131156 (152) |
| Default (64,10) | 131088 / 131112 (617) | 24 / 131112 (618) | 0 / 262240 (605) | 0 / 131112 (299) | 131112 / 131112 (300) | 4 / 262240 (287) |
| Narrow (1, 5) | 0 / 131112 (line 887) | 131112 / 131112 (line 888) | 4 / 262240 (line 875) | At s = 1 both shapes are identical, not separately built | At s = 1 both shapes are identical, not separately built | At s = 1 both shapes are identical, not separately built |
| Narrow (64,10) | 131088 / 131112 (752) | 24 / 131112 (753) | 0 / 262240 (740) | 0 / 131112 (434) | 131112 / 131112 (435) | 4 / 262240 (422) |

At the same starting point, when the hidden root references one data unit, class b has 24 states (two to four parts per ten thousand), class a 99.96%–99.98%, and Q1 is zero. When it references s=64 data units, class b is 100%, class a zero, and Q1 is four at p=10. At s=1 and retained s=64, class a is empty: in those shapes there is no confirmed root with uncovered units for tightening to protect.

**What this section answers and does not:**

- The s axis now covers three shapes: segment-one s=1, segment-one s=64 one-unit overwrite, supplemental s=64 retained. B1 Q1 becomes nonzero when s units are retained; Q3 and A1/A2 repair fractions are shape-independent. B4 class a/b shares swing between two to four parts per ten thousand class b and 100%; Q8 is 100% in all shapes.
- Not answered: instance 2's post-mount overwrites also retaining the shape of s units (registration amendment 12.11 explicitly specifies this is uncovered); p takes only 5 and 10; states take only two states: not-durable / durable; the mechanism of class b being 100% is inferred (point 4 in the paragraph “Is it an apparatus state selection defect, or is there originally no class a in this shape?”).
- B4's own positive control was `not_made` for the supplemental shape. To test class a → b in this shape, construct a class-a state separately, e.g. restore covered hidden-root-unit bytes in a post-row state from the omniscient log, then flip one byte on disk 0 and check reclassification. This new control is not in registration 12.12; it requires modifying `class_a_to_class_b_control`, adding tests and mutations, and regenerating the supplemental artifact. It was not done here.

### What it cannot answer

- Segment one answers only B1 and B4. B2, same-process lost writes and five candidates, belongs to segment two; the apparatus has no such code and fork-sheet B2 remains open.
- B, “fall back to a more conservative shadow-ledger set when an abandoned root is unreadable,” is undefined, so no arm is constructed.
- Crash states have only not-durable/durable values, not a third in-place-overwrite state. Segments with more than 16 writes use the quick tier, which cannot see a class requiring only some unit writes to land.
- In segment-one artifact `Overwrite` uses `publish_overwrite`; after the first overwrite the file has one unit, and hidden-root referenced-unit count does not depend on s (amendment 12.4). Supplemental retained-s arm measures that dimension (12.11), but changes only instance 1's overwrite to s-unit sequential writing; instance 2's three post-mount overwrites still have one unit. Post-mount retention of s units was not measured. Amendment 12.11 hypothesized that one transaction per unit could yield over 40 in-place writes in a segment, beyond closed-form expansion; unverified.
- B4's own positive control (registration 5.2 last sentence) in the new artifact covers only the three confirmed-tier histories with s = 64 (picking 2 class a states per history, all 6 trials changed from class a to class b); the three confirmed-tier histories with s = 1 cannot produce class a states (in every state the units referenced by the hidden root are already covered), marked `not_made`, so the direction of class a changing to class b was not tested by control on the s = 1 starting points, with only detection controls (18 `detect_covered` entries, looking only at the surviving disk) and unit test `the_hidden_root_class_looks_at_the_units_on_both_disks` (looking at both disks) covering the judgment of class b. In the supplemental arm artifact, the three confirmed-tier histories likewise failed to test once (every state after row-publication has units referenced by the hidden root already covered, missing `b4.class_a`), summary row `status=not_made`, with reasons and interpretation under “Supplemental-arm results”; thus the direction of class a changing to class b was tested by control only on the three histories with s = 64 overwriting one unit. The old artifact (1204 lines) does not have this control.
- A1/A2 outcomes report only selected roots, whether their referenced units are covered, and recovered reads (rerunning `recover` after avoidance), not whether content observed by the user is correct.
- Sampling points: four default, two narrow, two confirmation tiers; sparse in-memory disks; only two disks.
- Supplemental class b =100% and absent `b4.class_a` mechanism is inferred: a unit referenced by the hidden root was overwritten before the row-publication root-slot write, supported by four Q1 states; the guess is that instance 2 rebuilt the allocator from the older ledger and hidden-root-occupied slots appeared free. The apparatus did not identify which unit write covered which slot or inspect allocator order.
- Supplemental `source_files_differing_from_registration=1` names `crates/singlefs-checker/src/walk.rs`, which is not called. The checker `image` module and `lib.rs` actually called have no registered fingerprints and are modified relative to HEAD; behavioral changes were not reviewed line by line (commands in supplemental results).
- The three mutation numbers are not yet available; checker-tier-crates-mutation-replay supplies them at commit.

### Paths and conclusion registration

The conclusion concerns “roots selected and avoided in single-device recovery and enumerated counts.” Compute each quantity by two code-independent paths and compare.

| # | Path | Method | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | Core root selection | Call `choose_system_configuration` and `choose_root` on a single-device reader and select the self-validating root with maximum (txg, instance generation). | `crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs:1667 crates_choice` | Root ring positions and disk-region placement come from system configuration; `RootRecord::parse_slot` validates root slots. |
| 2 | Independent apparatus decoding | Call checker's `valid_roots` on the same reader and choose self-validating root with maximum (txg, instance generation). | `crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs:1673 independent_choice` | Same system-configuration bytes and reader; checker's `check_root_slot` validates roots without sharing core parsing. |
| 3 | Enumerator | Enumerate crash states by segment and expansion mode, count them, and compare with the built-in closed form. | `crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs:1189 for_each_state_of_segment` | Segments and writes from `writes_and_segments_with_stream_indexes`; expansion limit 16 is local. |
| 4 | Closed-form count | Full run segments: 2^m − 1; quick tier segments: 2^k × (any unit writes ? 2 : 1) − 1. In `anchors`, count ten (in-place writes, unit writes) combinations independently and with the enumerator. | `crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs:2439 enumerated_and_closed_form_state_counts` | Expansion limit 16; definitions of both expansion modes, registration section 5. |

Cross-path checks are pinned in source: paths 1 and 2 compare each state and disk and stop on mismatch (`evaluate_one_state`, “stop: only disk … core root selection … differs from apparatus independent decoding …”). Paths 3 and 4 are connected in ten I2 anchor cells and `enumerated_state_counts_are_pinned_to_independently_computed_numbers` (one cell each for 0, 7, 65535, 65535, 1, 127, 131071). A3 first-publication txg also compares `first_publish_txg_expected_by_the_clause` (apparatus independently scans root and journal rings, takes maximum plus one) with txg of the row root returned by mount. Conclusion values are the enumerated points, 18 histories and all crash states in each, because registration requires only these starting points. Both root-selection paths share system-configuration bytes and can fail together if those bytes are misread.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format), settled item 14 | Background material | 2026-10-19 unaffected: the item's evidence does not cite E182; it is background only. Segment-one and supplemental artifacts have run (Q1, Q3, Q4, Q5, Q6 above), B1 is sufficient: Q1 is nonzero in the retained-s shape (default each tier 4 / 393396), A2 Q3 is 49.98% for all three shapes. Whether to promote it to support and record C as a known edge case in scope is for the main agent. Regenerated output with B4 control has unchanged readings; decision text was not changed. |
| D16 (Publish Semantics), settled item 1 | Background material | 2026-10-19 unaffected: the item's evidence does not cite E182. Its “root slot reads bad this time” zero-slot handling relates to B4's tighter txg skip. Both artifacts have run (Q7/Q8 above), B4 sufficient: Q8 is 100% in all shapes, class b is two to four parts per ten thousand for one-unit overwrite and 100% for s=1 and retained s. B4's own control summary holds in regenerated segment one (three s=64 holds, three s=1 not_made) and is not_made in supplemental (no class a in any of three). Decision text was not changed. |

## Historical Versions

Historical entries for E182 (Three Forks When an Abandoned Root Cannot Be Read) are centralized in [experiments-history.md](../experiments-history.md).
