## E170 First Version of O3 (independent specification executor) — second rerun completed (Q1–Q5 decisive)

[中文](170-O3规约执行器第一版.md) · **English** · [日本語](170-O3-仕様実行器の初版.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 S1 S2 S3 S4 V2 V8 XE1 XE2 XE3 XC1 P1 P2 P3 P4 P5 P8 P9 M2 V2b G01 G02 G03 F1 F2 F3 F4 F6 F7 G5 G20 V5 -->
<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 U8 XC2 XE4 G04 G12 G13 G14 G15 G16 G17 G18 G19 G21 G22 V7 V9 V10 V3 P10 F5 F9 F10 F11 S5 S5b V11 G6 -->
<!-- doc-lint:not-numbers I5 K270 V21 XC3 XC6 XC7 XC8 XC9 -->

```text
Local arm names and symbols on this page are lowercase (a1, a2, a3, a4, ...). Preregistration, apparatus, artifacts and commands use the corresponding uppercase forms, matching global registration identifiers; the KB uses lowercase to distinguish them. Read artifact identifiers as uppercase.
```

Preregistration: `e170-preregistration.md` (section 12, revision 1). Questions in `o3-spec-executor-questions.md`: Q1, can a Python executor written solely from the KB decide whether a crash state belongs to the specification's allowed set? Q2, do executor and implementation agree on the same states? Q3, does the executor have discriminative power? Q4, how long per state, and can it enter the quick tier?

**Status:** The second rerun (2026-10-04; see its section) used a new question sheet, `e170-r2-questions.md`: two separate executor directories, predicates W and T, and an update for the 487-byte journal record header. All five questions are decisive; Q4 chooses A, and E171's first rerun uses v2-487. The first rerun (2026-10-03, strict mode) also ran three stages: Q1 decidable, Q2 consistent, Q3 has discriminative power. The first run completed all three stages: an exporter reading `crates/` exported two crash-point replay streams, sampled three batches totaling 972 states and recorded implementation verdicts; a Python executor reading only the KB was written, frozen and run on the batches; an operator reading `crates/` then wrote a comparator, constructed bad-state and blind-test families, and ran mutations without changing the executor. Literal section 6 verdicts:

| Question | Verdict | Evidence | Qualifications |
|---|---|---|---|
| Q1 | First run: undecidable; first rerun stage 1: decidable | First run: all 450 batch-1 states hit G01, G02 or G03 specification gaps. After five changes to match the 2026-10-03 KB and freezing, strict batches 1 and 2 had zero gaps in 450 and 126 states respectively. | The three missing statements were added to the KB; the rerun used a newly unfrozen and changed directory, leaving the original frozen executor untouched. See `e170-r1-o3-spec-executor-2026-10-03-s1-executor.out`. |
| Q2 | First run: consistent (Q2-a 0 / 450); first rerun second stage: consistent (strict execution mode, re-exported batch 1 Q2-a 0 / 450, batch 2 Q2-d 0 / 126, Q2-pc 1) | First run: `e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out`; first rerun: `e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out` | Both runs resulted in the executor deciding "in" for all states (972 states in the first run, 576 states total in the first rerun across batch 1 and batch 2) and the implementation passing all states; in the first run, both F1 and F6 triggered; in the first rerun, F1 did not trigger (Q3 has discriminative power), and the two threshold conditions of F6 (replay >= 1, ③ > 0) passed, but the two execution paths "the selected root is not the root with the largest txg among self-validated roots" and "received >= 2 slots in ③" were 0 in the batch, so "consistent" does not cover them; the cross-table still has only the single cell "executor in, implementation passes", and the cell where the executor is "not in" while the implementation is red has zero states in the batch (see the "First rerun" section) |
| Q3 | First run literally: no (Q3-a 4 / 26, Q3-b 0 / 40); first rerun: yes (P 0 / 26, blind 0 / 40, P8 expectation revised to in) | `e170-o3-spec-executor-2026-10-02-s3-bad-states.out`; `e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out`. | All four accepted rows were P8, due to the interpretation-3 lower bound. The preregistration was revised before artifacts to expect in; landing v_{n−1} remained unchanged. The original loss remains recorded. Additional P10, the P8 modification on Mk⁺ expecting out to test A(c)'s lower bound, was correct 2 / 2; blind seed changed with the newly frozen executor. |
| Q4 | Can enter the quick tier | Stage 2 batch 1: 0.170246 s with six processes; stage 3: 0.195638 s with eight, 0.662894 s with one. | Both timings included load from other sessions. |

### What stage 1 did

| Item | Location | Description |
|---|---|---|
| Exporter | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` | Checked-in driver and observer; `dump --output-directory <directory> [--compare-with <previous export directory>]`. |
| Export files | `scripts/experiment-apparatus/target/e170-dumps/<stream>/` (not in git) | Ten files per stream; SHA-256 in artifact `name=file` rows and below. |
| Artifact | `e170-o3-spec-executor-2026-10-02-s1-export.out` | 142 `E7RESULT` lines from the second run, independent of run count. |
| Timing artifact | `e170-o3-spec-executor-2026-10-02-s1-export-timing.out` | Twelve `E7TIMING` lines: per-batch `judge_crash_image` time and hashes of the two timing-bearing files. |
| Mutations | Last three rows of `crates/mutations.tsv` | Section 9 XE1–XE3, names prefixed E170. |

- Stream 1, new-pool file creation, copies `build_pool` in `tests/common/mod.rs`: number acquisition, warm-up and creation calls, recording stream length after each. The complete stream including mkfs equals `build_pool` operation by operation.
- Stream 2 copies the branch of `prepare` in `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`: fixed script through E and normal unmount, 15 calls.
- A `judge_crash_state_range` callback supplies each sampled `CrashImage`; it is reduced to a persistence vector and hashed. A separately timed `judge_crash_image` call on the same image agrees item by item, with zero mismatches in both `name=rejudge` lines.
- Stops S1 (36 local-constant comparisons with crates) and S4 (rebuild solely from exports and compare each digest against `states.tsv`) passed. V2 ran twice; see below.

Two unspecified formatting choices preserve registration 5.3 columns: `states.tsv` uses batch numbers 1, 2 and 3 (5.3 says “1 or 2”, 5.4 requires all three); the fully persisted state belongs to no segment and uses segment count as its number (11 and 78), matching `segment_of_state` in `crates/singlefs-checker-tier/src/crash.rs`.

### Result lines transcribed from artifacts (removed artifacts have one-line pointers)

`e170-o3-spec-executor-2026-10-02-s1-export.out`:

```
E7RESULT name=stream stream=new_pool_file_creation operations=57 writes=41 segments=11 tearable_writes=8 calls=3 full_states=16777260 quick_tier_states=46 sampled_states=82
E7RESULT name=batch stream=new_pool_file_creation batch=1 states=53 segments_touched=12 states_with_a_torn_write=20 pool_checker_and_record_checker_both_pass=53 pool_checker_red_states=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=2 states=7 segments_touched=1 states_with_a_torn_write=0 pool_checker_and_record_checker_both_pass=7 pool_checker_red_states=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=3 states=22 segments_touched=1 states_with_a_torn_write=0 pool_checker_and_record_checker_both_pass=22 pool_checker_red_states=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-export.out, line 74 (name=stream)
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=1 states=397 segments_touched=79 states_with_a_torn_write=115 pool_checker_and_record_checker_both_pass=397 pool_checker_red_states=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=2 states=119 segments_touched=17 states_with_a_torn_write=0 pool_checker_and_record_checker_both_pass=119 pool_checker_red_states=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=3 states=374 segments_touched=17 states_with_a_torn_write=0 pool_checker_and_record_checker_both_pass=374 pool_checker_red_states=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=s4 stream=new_pool_file_creation rebuilt_states=82 digest_mismatches=0
E7RESULT name=s4 stream=fixed_script_through_e_then_unmount rebuilt_states=890 digest_mismatches=0
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 rejudge_mismatches=0 states_where_pool_checker_or_record_checker_is_red=0 v2_compared=true v2_differing_files=4 v2_differing_files_without_timing=0
```

- On Q2's implementation side, pool-level checker and record checker accepted all 972 sampled states (82 and 890); neither oracle was red on any state.
- Each batch-1 segment with a torn write (four in stream 1, 23 in stream 2) had only eight states, all already selected by the batch-1 grid. A separate torn-state scan found no new state (27 `name=torn_scan … scanned=8 found=0 wanted=4` lines). Batch 1 included 20 and 115 torn-write states from the grid. Batches 2 and 3 covered only large unit-write segments, one and 17 respectively; all points in small segments were merged into batch 1.
- Single-thread implementation `judge_crash_image` on sampled states: stream 1 median 1.23–1.26 ms; stream 2 median 27.6–27.9 ms, p95 53.1–53.7 ms, maximum 227 ms. These timings are not Q4, which measures the executor.

Counts per batch and per stream are counted from export files using this command, agreeing with artifact `name=batch` rows:

```
for s in new_pool_file_creation fixed_script_through_e_then_unmount; do awk -F'\t' -v s=$s '{n[$4]++; if($5~/1/) t[$4]++} END{for(b in n) print s, "batch="b, "states="n[b], "with_torn="t[b]+0}' scripts/experiment-apparatus/target/e170-dumps/$s/states.tsv | sort; done
new_pool_file_creation batch=1 states=53 with_torn=20
new_pool_file_creation batch=2 states=7 with_torn=0
new_pool_file_creation batch=3 states=22 with_torn=0
fixed_script_through_e_then_unmount batch=1 states=397 with_torn=115
fixed_script_through_e_then_unmount batch=2 states=119 with_torn=0
fixed_script_through_e_then_unmount batch=3 states=374 with_torn=0
```

### V2 and revision 1

V2 requires identical SHA-256 for each file in two exports. `impl-verdicts.tsv` includes per-state timing required by 5.3, and its digest is recorded in `manifest.tsv`; both must differ between runs, yielding four `identical=false` files. Revision 1 adds V2b, comparing after removing the timing column and corresponding manifest row: all four match (`v2_differing_files_without_timing=0`), and the other eight files are byte-identical. Revision 1 leaves it to the main agent to decide validity when literal V2 triggers but V2b does not.

```
E7RESULT name=v2 stream=new_pool_file_creation file=impl-verdicts.tsv identical=false
E7RESULT name=v2_without_timing stream=new_pool_file_creation file=impl-verdicts.tsv identical=true
E7RESULT name=v2 stream=new_pool_file_creation file=manifest.tsv identical=false
E7RESULT name=v2_without_timing stream=new_pool_file_creation file=manifest.tsv identical=true
E7RESULT name=v2 stream=fixed_script_through_e_then_unmount file=impl-verdicts.tsv identical=false
E7RESULT name=v2_without_timing stream=fixed_script_through_e_then_unmount file=impl-verdicts.tsv identical=true
E7RESULT name=v2 stream=fixed_script_through_e_then_unmount file=manifest.tsv identical=false
E7RESULT name=v2_without_timing stream=fixed_script_through_e_then_unmount file=manifest.tsv identical=true
```

First and second run `E7RESULT` lines otherwise match line by line, excluding `name=config`, `name=verdict`, `name=done` and the second run's `name=v2` lines.

### Export file SHA-256

Under `scripts/experiment-apparatus/target/e170-dumps/` (`cd scripts/experiment-apparatus/target/e170-dumps && sha256sum */*`). Timing-bearing file hashes are also in `name=file_with_timing`, the other eight in `name=file`. Stage 2 checks V8 against this table.

| Stream | File | sha256 |
|---|---|---|
| new_pool_file_creation | base-sectors.bin | `0358e613eb190dd9a9212a6282fa7671976349e42f7fd448999239a69216ce35` |
| new_pool_file_creation | ops.tsv | `f6f57075c22ae45dead0dc2f1942d837901a91bc0a6c35e371fcc71c29070d0b` |
| new_pool_file_creation | ops-payload.bin | `d0f2928b9462af5cb45d550fc824209cbbfe4a8b5a4dab803a849535cb2e18e3` |
| new_pool_file_creation | calls.tsv | `c0ca8a14aaeb45d80f8db2a7d3a06a4197f50141685493ea6a44087fd7d10bab` |
| new_pool_file_creation | torn.tsv | `08fec2c3c6de44a767ce5b7806942d5d275548292600a9252c344ad5d3506311` |
| new_pool_file_creation | torn-payload.bin | `f76cdeaefc1b9f43cba7b070533f0772a838ca0e8c96773c400413a402f22754` |
| new_pool_file_creation | states.tsv | `9f556acc7db0847539d7b4466cda13dd92dca97ddab2475ad1dfefebe9b154e5` |
| new_pool_file_creation | impl-verdicts.tsv | `34e3289a159b291f869c3afea255492cadcab385e7c32a77bb590f2e68724cc7` |
| new_pool_file_creation | quick-tier.tsv | `4602e4f1e9944d9b0db07c391650c3958a25fc96727328b5d809484d6bdac65c` |
| new_pool_file_creation | manifest.tsv | `43825e1619b30f942913c960ac3a7f27d63b8b391b88beb104964eb52c4a6802` |
| fixed_script_through_e_then_unmount | base-sectors.bin | `0358e613eb190dd9a9212a6282fa7671976349e42f7fd448999239a69216ce35` |
| fixed_script_through_e_then_unmount | ops.tsv | `b46474482c727550cf66ca6130d5798761e31861611bf52f805348e96d40bceb` |
| fixed_script_through_e_then_unmount | ops-payload.bin | `a95c3d7a1bae509e9be89e516f418f3ea2ec58cbeb32088599a04247bdffd77f` |
| fixed_script_through_e_then_unmount | calls.tsv | `3170af975da1862300086e3c08f52a14f3ceb3920c6e8ab0044760108fb529f8` |
| fixed_script_through_e_then_unmount | torn.tsv | `6dba83810dd26ea3269a8e8447606ca95a069ffb6b1bf8d12a5bd4956c0e0fa8` |
| fixed_script_through_e_then_unmount | torn-payload.bin | `f6701bbfbefa36877d97961923a08cbce3bcce7002586c84e872b0dbef671706` |
| fixed_script_through_e_then_unmount | states.tsv | `8c8ffbd5a0694a404f4b0fcfeaecbde4d5e5eb744b776e3951d8f757f00e0ed3` |
| fixed_script_through_e_then_unmount | impl-verdicts.tsv | `b8282c02a79208fb922490b8f53685c736d0fafcb77549544d5f347faff3190e` |
| fixed_script_through_e_then_unmount | quick-tier.tsv | `9cbf31ebda49e12196d98a347843a30d000b888f3f3f5877c18314f40b36487f` |
| fixed_script_through_e_then_unmount | manifest.tsv | `535cb2c67fa44ccece8bb7f62e04a4327e1172e46d3fe347b100a5fdb03b8a7a` |

The two `base-sectors.bin` files are identical: both base images are immediately after mkfs.

### Stage 2: executor, Q1 and Q4

The executor reads only `.claude/kb/decisions/`, `crates/singlefs-checker/invariants/`, `layout/` and the other sections permitted by the registration, not `crates/`. The 20-row gap table `spec-gaps.tsv` lists G01–G20. Strict mode reports a specification gap on hitting any row; provisional mode continues using the specified provisional interpretation (5.6). Once all three batches were judged, the executor was frozen: hashes of its 15 files are in the artifact, and their digest's first 16 digits form the seed. The six-worker and single-worker artifacts `e170-o3-spec-executor-2026-10-02-s2-executor.out` and `e170-o3-spec-executor-2026-10-02-s2-executor-single-thread.out` have identical verdict lines apart from `time_ns` and `worker`.

`e170-o3-spec-executor-2026-10-02-s2-executor.out`:

```
name=verdict batch=1 stream=new_pool_file_creation states=53 strict_spec_gap_states=53 strict_spec_gap_identifiers=G01,G02,G03 strict_given_states=0 provisional_in=53 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
name=verdict batch=1 stream=fixed_script_through_e_then_unmount states=397 strict_spec_gap_states=397 strict_spec_gap_identifiers=G01,G02,G03 strict_given_states=0 provisional_in=397 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
name=gap_hits batch=1 stream=new_pool_file_creation G01=53 G02=53 G03=10
name=gap_hits batch=1 stream=fixed_script_through_e_then_unmount G01=397 G02=397 G03=60
name=timing batch=1 workers=6 states=450 total_ns=170246313 sum_state_ns=874287307 largest_worker_sum_ns=147931302 worker_processes=6 contains_self_check=pass
name=freeze_seed sha256_prefix16=8546b1258e427c34
```

- Q1: Q1-a = 3 identifiers and 450 states: G01, the KB does not specify magic values for three self-validating structures; G02, it does not say that row order is on-disk order for the system configuration field table, first ten journal-header fields and the position entry; G03, it does not specify where system configuration slot 1 is when slot 0 fails self-validation. Q1-b (rows requiring crates to write) = 0; stage 2 was uncertain about G01's category. Q1-c = 0 / 450; Q1-e = 450 / 450.
- All 972 states in three batches are provisionally in, and all image digests match (S2 does not trigger). a2–a7, a6 and a11 hold; see the registration issues for a1.
- Q4: batch 1 took 0.170246 s with six processes and 0.682941 s with one (single-thread artifact: `name=timing batch=1 workers=1 states=450 total_ns=682940841 sum_state_ns=633492202 largest_worker_sum_ns=633492202 worker_processes=1 contains_self_check=pass`). Extrapolation from batch 3 gives 0.189993 / 0.670388 s. All four are ≤ 60 s, so the executor can enter the quick tier. A budget between the two points, e.g. 0.4 s, admits one but not the other; the decision segment separates them (calculated).

### Stage 3: comparator and Q2

`comparator_q2_implementation_verdicts.py`, in the executor directory but excluded from its frozen hash because its name starts with comparator, reads stage 2's frozen six-process `name=state` verdicts and stage 1's `impl-verdicts.tsv` and compares each state. “Implementation accepts” means no pool-level checker Violated result and both record-checker flags false (interpretation 6). It neither changes the executor nor rejudges states.

`e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out`:

```
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 4 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 9 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 10 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 16 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 7 (name=q2_cross)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 10 (name=q2_cross)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 22 (name=q2e_from_segment_two)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 23 (name=q2e_from_segment_two)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 28 (name=q2_positive_control)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-q2-comparator.out, line 29 (name=verdict)
```

- Q2-a = 0 / 450 (batch 1), Q2-d = 0 / 126 (batch 2); both points agree, F7 does not trigger. Batch 3 additionally has 0 / 396; Q2-f = 0, with identical (instance generation, txg) landings for each state; Q2-pc = 1, so V5 does not trigger.
- Q2-c: strict mode returned neither in nor out for any state (Q1 undecidable), so there are zero comparable states and no number under this measurement basis.
- The cross-table contains only a single cell: executor "in", and both implementation checks pass. The executor never decided "not in" on any state, and the implementation never went red on any state; "consistent" means consistency where both sides passed all states.
- Q2-e: states replaying ≥ 1 record numbered 6 and 51. Literal “chosen root is not the greatest txg on disk,” counting all disk root slots whether self-valid or not, was zero in both streams; F6 triggers and agreement does not cover this path. Stage 2 inferred, without measurement, that legal states structurally cannot make this count positive.
- Section 8 trajectory: batch-1 distinct states along segment order covered 12 and 79 segments, with peak 0, zero positive segments and 0 in the final segment (`name=q2_trajectory measure=q2a`).

### Stage 3: bad-state table and Q3

The checked-in exporter gained `bad-states --dump-directory <export directory> --seed-hex <seed>`. It re-drives both streams, first checks `base-sectors.bin`, `ops.tsv`, `ops-payload.bin`, `calls.tsv` byte for byte against exports, then constructs section 5.5 base states F, Mk, Mk⁺, Mk⁻ and blind-test call boundaries and byte patches. Root records, journal records and system configuration slots are rewritten and resealed using core writers `RootRecord::to_slot`, `JournalRecord::to_bytes`, `wide_checksum_with_field_zeroed`; fake-root landing uses core `target_for_publish` and `slot_offset`. Each row is separately judged by the implementation: the patch is appended to the write table as a persisted system-configuration-slot write, which `publishes_of_one_recording` does not include in any publish. Seed `8546b1258e427c34` from stage 2's freeze feeds splitmix64; both streams share one sequence, with stream 1's 20 rows drawn first. Each stream writes `bad-states.tsv`, `bad-patches.bin` (row, base persistence vector, two digests and patch for the executor), `bad-expected.tsv` (expectations for the comparator only), `bad-impl-verdicts.tsv` and `bad-seed.tsv` under the untracked export directory.

The frozen executor judges bad states via `comparator_q3_bad_states.py`: `executor_core.judge_state` is called unchanged; during evaluation the `build_crash_image` name is replaced by a wrapper that builds unchanged and overlays the patch, while `recover` is wrapped solely to record the landing projection. A(c) is computed from the base vector, excluding the patch as a write. Expectations and implementation verdicts are opened only after all verdicts are written. The initial check confirms current frozen lines match stage 2 line for line (`freeze_identical_to_segment_two=true`) and exporter seed equals frozen seed.

`e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out`:

```
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out, line 78 (name=bad_family)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out, line 168 (name=bad_family)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out, line 76 (name=bad_family)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out, line 166 (name=bad_family)
E7RESULT name=verdict bad_s1_failed_checks=0 bad_rows=76 impl_panics=0
```

`e170-o3-spec-executor-2026-10-02-s3-bad-states.out`:

```
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states.out, line 161 (name=q3_counts)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states.out, line 162 (name=verdict)
```

Counts by family and verdict (command and unchanged output):

```
grep 'name=bad_comparison' e170-o3-spec-executor-2026-10-02-s3-bad-states.out | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); f[kv[1]]=substr($i,length(kv[1])+2)} print f["family"], f["provisional"]}' | sort | uniq -c
     40 B not_in
     10 N in
      4 P in
     22 P not_in
```

- Q3-a = 4 / 26: P8-F and P8-Mk, two rows per stream. All other P1–P5 and P9 cases on both bases and streams are out. G5 compares base F to Mk: 2 each, same direction.
- Q3-b = 0 / 40: blind P2-type 13, P3-type 12, P5-type 15; F base 9, call boundaries 31. Count types and bases by filtering `name=bad_comparison` and `family=B`, splitting `name=` on colons and counting the second and last components.
- Q3-c = 0: all 76 strict rows hit gaps G01 and G02; provisional mode judges every row.
- Q3-d = 0: every P landing matches 5.5, including no landing for P1 and v_{n−1} for P8; the others land on a projection absent from the version table. The whole projection, including both pointers, is compared.
- Q3-e: all ten N rows are in and land as expected (n5 at v_{p−1}); F4 does not trigger.
- Q3-f, auxiliary for the three-oracle collusion test: the implementation accepts both checks on 28 of 66 P and blind rows; F3 does not trigger because this is not all rows. P1 on both bases reports recovery failure. P5-F in both streams and stream 2 P5-Mk are pool-level checker red, while stream 1 P5-Mk is accepted. P2, P3, P4, P8 on F and P8 on Mk are accepted. P2–P4 on Mk and P9 on Mk⁺ are record-checker red for claimed state missing a unit. n1-F is pool-level checker red on both streams although executor says in. The 28 is an upper bound: patches recorded as system-configuration-slot writes are excluded from publishes, and record-checker `root_without_record` never sets on the 76 rows, so rows that should have been rejected by that criterion cannot be excluded. It supports “not all,” not a measure of implementation discrimination strength.
- Mk is the return boundary of stream 1's first call (warm-up) and stream 2's seventh (txg 10 publish); Mk⁺ advances zero calls. Mk⁻ is the only named-unit publish (1, 3) in stream 1 and the middle one (2, 11) of 17 in stream 2. F, Mk, Mk⁻ and blind call-boundary vectors all appear in crash replay indices when checking only segment endpoints. Mk⁺ does not, because one extra unit write is persisted off a boundary.
- Determinism: bad states were run once on a draft copy in the export directory and then once formally; result rows were byte-for-byte identical, and all ten bad-state files were byte-for-byte identical.

### Stage 3: executor mutations

Mutation table `scripts/mutation/tables/e170_o3_spec_executor.tsv` has 12 rows, x1–x11 and XC1. Before dispatch, `mutation_driver.py` checks each anchor occurs exactly once across the executor's Python files; each mutation is applied in a draft copy, compiled and run against 36 unit tests and `comparator_mutation_checks.py` (166 checks, 160 true on baseline: all bad-state verdicts and landings, a1's landing half, a2–a7, a6, a11, batch-1 rejudgment against implementation, and comparator positive control). Copies are removed and the baseline rerun in a fresh copy.

Two artifacts are byte-identical, SHA-256 `fb734a82f35b1915dd85649e27f1db0894278c6737c0c197303972f750a63080`. In the first table x10 anchored `record.watermark <= root_watermark` in prefix selection, after the executor already stopped when instance differs from the chosen root; at that point record and root share an instance and the orderings agree, making this mutation equivalent for every input. The anchor was moved to the chain-head candidate `above = …record.watermark > root_watermark`, the only cross-instance ordering comparison, and rerun as `e170-o3-spec-executor-2026-10-02-s3-mutations-2026-10-02.out`, referenced by `replay.sh`. Results do not include anchor text and were identical; x10 still did not go red. Equivalence was not added as a test because the unit-test file is frozen.

```
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-mutations.out, line 27 (name=restored)
Artifact line: git show 5d0b7e7c1667:research/results/e170-o3-spec-executor-2026-10-02-s3-mutations.out, line 28 (name=verdict)
```

| Mutation | Outcome | Caught by | Did the section 9 sensitive sample change? |
|---|---|---|---|
| x1 root selection takes the minimum | ✅ | 18 unit tests, 98 checks including a1 landing, a11, batch-1 rejudgment | Yes |
| x2 lower instance generation wins tie | ✅ | Unit tests only, 3 on small pool | Registration specified no sensitive sample; none in bad or sampled states |
| x3 jsn merely greater than previous | ✅ | One unit test, two n2 rows | Yes |
| x4 ignore last-record flag | ✅ | One unit test, four P4 rows | Named n3 unchanged: inferred that when both count as last, the earlier is still corruption because another record follows its last flag, and landing remains v_n. Four P4 verdicts and landings no longer match. |
| x5 do not verify named units | ✅ | One unit test, two n5 landings | Yes |
| x6 choose smallest system-configuration generation | ✅ | Two unit tests, a4 | Yes |
| x7 omit root-slot self-validation checksum | ✅ | Two unit tests, two n4 and four P8 landings | Yes |
| x8 always use version 0 as lower bound | ✅ | One unit test only | P8 already in on baseline, so it cannot flip; no other sensitive bad or sampled state |
| x9 compare versions only by (instance generation, txg) | ✅ | Two P9 rows; no unit test red | Yes |
| x10 prioritize txg in record watermark | ❌ | — | At chain-head anchor, order differs only if the chosen root's covered record is unreadable and another instance's record exists in the ring (inference). No sensitive sample was registered; neither stream contains a discarded instance's record with greater txg. Per mutation-sampling rule category 3, record uncovered sensitive sample, not equivalent. |
| x11 skip unreadable slot and continue | ✅ | One unit test, two n2 rows | Yes |
| XC1 comparator treats acceptance as always true | ✅ | Q2-pc | Yes |

Eleven of twelve mutations were caught; x10 did not go red. x2 and x8 were caught only by unit tests, with no sensitive bad or sampled states.

### Q4 stage-3 measurement

The unchanged executor and same export ran all batches once each with eight workers and one: `e170-o3-spec-executor-2026-10-02-s3-executor-rerun.out`, `e170-o3-spec-executor-2026-10-02-s3-executor-rerun-single-thread.out`. Verdict lines match stage 2 except `time_ns` and `worker`; frozen lines and seed match.

```
name=timing batch=1 workers=8 states=450 total_ns=195638070 sum_state_ns=1329348512 largest_worker_sum_ns=169178758 worker_processes=8 contains_self_check=pass
name=timing batch=1 workers=1 states=450 total_ns=662893759 sum_state_ns=617497518 largest_worker_sum_ns=617497518 worker_processes=1 contains_self_check=pass
```

Extrapolating batch 1 from batch 3 gives 0.201575 s with eight and 0.699101 s with one, using stage 2's arithmetic. All four ≤ 60 s, consistent with stage 2. Other sessions ran cargo and broad mutation tests at the time; timings are one observation only.

### Three registration issues (decided by the main agent)

- **a1 “A(F) has exactly one element”:** Under interpretation 3, m(c) for fully persisted state is the last write's stream index; the final call's return point is stream length and greater than m(c), so that version is not acknowledged. A(F) = {(1, 2), (1, 3)}, observed as two elements (`name=anchor_assertion id=a1 stream=new_pool_file_creation holds=false landed=(1, 3) allowed=[(1, 2), (1, 3)] landed_part=true single_element_part=false`). The main agent recorded F2 per section 10: not invalidated, anchor unchanged, landing half holds.
- **P8 expectation “out”:** Registration 5.5 assumes v_n acknowledged (ack = n), but under interpretation 3 the final call is not acknowledged on F or Mk, so ack = n − 1 and landing v_{n−1} is in A(c). The main agent left the expectation unchanged before artifacts and reported the result: all four P8 are in and land at v_{n−1}, giving Q3-a = 4. Under the weakest reading Q3 literally says “no”; the reason is inconsistency between the interpretation-3 lower bound and P8's assumed expectation, not a root or prefix misread. x8 consequently lacked its expected P8 sensitive sample.
- **Literal second Q2-e item:** maximum txg read from nonzero root ring slots regardless of self-validation is zero in batch 1 of both streams; F6 triggers.
- Literal V2 of the first stage (every file has identical sha256 across two exports) triggered: `impl-verdicts.tsv` includes the elapsed time column required by preregistration 5.3, which necessarily differs between two runs along with `manifest.tsv` recording its digest; the lead agent judged that the two clauses in the preregistration contradicted each other, accepted revision V2b, and recorded the trigger of literal V2 truthfully (basis: the E170 first-stage handoff entry in dispatch record `records/2026-10-01-里程碑三形式化接入.md`).

### First rerun (strict mode, 2026-10-03)

Preregistration `research/prompts/e170-r1-prereg.md` (section 12 revision 1, written before artifacts); question list `e170-r1-questions.md`: Q1 (after the KB filled in the three specification gaps G01–G03, whether the executor modified strictly according to the KB text can decide "in / not in" for every state in batch 1 from the first run), Q2 (whether verdicts from strict execution mode agree with the implementation), Q3 (when the bad-state table is rejudged against corrected P8 expectations, whether the executor possesses discriminative power). The rerun was divided into three stages: stage 1 answered Q1; stage 2 answered Q2 (re-export directory, frozen executor judging on re-exported batch 1 and batch 2, comparator comparing state-by-state); stage 3 answered Q3 (exporter new mode, bad-state table, executor mutations). All three stages were executed, and all three rows of the question list reached sufficient evidence.

#### Stage 1: Q1 (executor unfrozen once, five changes against 2026-10-03 KB)

The original frozen executor was copied to `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`; all 15 hashes matched its original frozen lines (V9), and no byte in the original directory changed. Five KB-based changes c1–c5 were frozen and run on batches 1 and 2 of the original `scripts/experiment-apparatus/target/e170-dumps/`; all 20 hashes matched the table above (V8). All four strict verdict lines show zero states hitting a specification gap: Q1 is decidable. Batch 2's 126 states agree in direction with batch 1's 450, so it is not unstable.

| Measure | Value | Artifact or command |
|---|---|---|
| Q1-a, batch-1 gap states | 0 / 53 and 0 / 397, total 0 / 450; identifiers `zero` | `name=verdict` batch 1 in `research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out` |
| Q1-b, rows requiring crates | 0 of 22: nine two-readings, ten missing statements, three filled by KB | Count column 5 of `spec-gaps.tsv` with awk. |
| Q1-c number of states in batch 1 giving in / not in ÷ total number of states in batch 1 | 1 (450 / 450) | Same as `strict_given_states` on lines 496 and 501 |
| Q1-d, five changes have grounds | All five have citations below; five additional changes outside the 5.2 table are separately listed for the main agent to decide under V10 | See Q1-d below |
| Q1-e, batch-2 gaps | 0 / 7 and 0 / 119, total 0 / 126, same direction | Lines 635, 640 |
| Q1-f rows among G04–G20 supported by existing sentences in the KB; step-7 unit magic | Itemized per-row conclusions are recorded in the section "Q1-f: rows among G04–G20 supported by existing sentences in the KB as of 2026-10-03 and step-7 unit magic" | See the KB source paths and line numbers itemized in that section |
| Q1-g, strict re-exported batch 1 | Not run: counted only if batches differ, established after re-export | Stage 2 |
| Q1-h, provisional batch-1 fraction decided | 1: `provisional_in` 53 + 397, `provisional_not_in` 0 | Lines 496, 501 |

Exact artifact lines, numbered as in `e170-r1-o3-spec-executor-2026-10-03-s1-executor.out`:

```
496:name=verdict batch=1 stream=new_pool_file_creation states=53 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=53 provisional_in=53 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
501:name=verdict batch=1 stream=fixed_script_through_e_then_unmount states=397 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=397 provisional_in=397 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
635:name=verdict batch=2 stream=new_pool_file_creation states=7 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=7 provisional_in=7 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
640:name=verdict batch=2 stream=fixed_script_through_e_then_unmount states=119 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=119 provisional_in=119 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
499:name=gap_trajectory batch=1 stream=new_pool_file_creation segments=12 peak=0 positive_segments=0 last_segment_value=0
504:name=gap_trajectory batch=1 stream=fixed_script_through_e_then_unmount segments=79 peak=0 positive_segments=0 last_segment_value=0
3:name=dump_check stream=new_pool_file_creation count_mismatches=0 checksum_mismatches=0 version_table_anomalies=0
19:name=dump_check stream=fixed_script_through_e_then_unmount count_mismatches=0 checksum_mismatches=0 version_table_anomalies=0
9:name=anchor_assertion id=A1 stream=new_pool_file_creation holds=true landed=(1, 3) allowed=[(1, 2), (1, 3)] landed_part=true two_element_part=true
14:name=anchor_assertion id=A13 stream=new_pool_file_creation holds=true slot_one_routes={0: ('1', 4096, 4096), 1: ('1', 4096, 4096)}
15:name=anchor_assertion id=A12 stream=new_pool_file_creation holds=true magic_values={'system_configuration': '53465342', 'root_record': '53465352', 'journal_record': '5346534a'} chosen_slots=2 roots=4 records=6
```

All 11 `name=anchor_assertion` lines hold: a6 on both streams; a1–a5, a7, a12, a13, and a11 on stream 2. All 576 state verdicts are `strict=in`, with `digest_mismatches=0` (S2) and `count_mismatches=0` (S3). Section 7 corrected a1's expected set to exactly (1, 2), (1, 3); the observed set matches the first run.

**Freeze lines**, 15 files, artifact lines 645–659; seed line 660 is `85441ae491419411`, replacing original seed `8546b1258e427c34`:

```
645:name=freeze file=crash_dump.py sha256=7a671b85d8346f2aee45982a888e2111e98495f5570ecfc57a348ee7595d30dd
646:name=freeze file=crash_image.py sha256=e391dbe8377014258f1978005062d4cb14891b8565ca420a6a5b731ad4387a5f
647:name=freeze file=crc32c_castagnoli.py sha256=34b9f571bc7d0e9dc32dce56b166536a6c26581b9f4744e7c3a14b0ef69bea6d
648:name=freeze file=executor_core.py sha256=7bdbc912ee84b4419f6b9bd0d1f5eee870641b118d335072301f3cf9ecc52ec6
649:name=freeze file=executor_main.py sha256=acdae4aa3f1a6a3fde3dcaa19bddd0ed667d2ccf0855d5040bc416cf7c310a67
650:name=freeze file=journal_ring.py sha256=9f78d7ed299d51998d8e67d1b6c28c006e8ea778f6d18cb6bb9332afee640679
651:name=freeze file=on_disk_layout.py sha256=86e1a92cab056ca3dd908be2d25cd2f4a25c9ce7e7c896e4704d58716ec02d51
652:name=freeze file=replay_prefix.py sha256=ff2f52d4f93910a88dbab37bfeafaecc9d7b58fe0e7c139923e6de771bbff9da
653:name=freeze file=root_ring.py sha256=6ac949d183997789419d1aeeb448d00bc341b26216ce7c22b6691879a7a2f89e
654:name=freeze file=spec-gaps.tsv sha256=32ec144a900d09baedc88869f4e392a003866d566e41f1a4df611a67f0ace4f7
655:name=freeze file=spec_gaps.py sha256=b71a245a49438a24e53f4a3b469ad965c4d9661bcf2d7fe87ac8b90a18c82e8c
656:name=freeze file=synthetic_pool_for_tests.py sha256=69368e07c746dade040618b7277b27d7a36f557d2fcac86ccf4048a1c8e40f63
657:name=freeze file=system_configuration.py sha256=a07a280f3c96d8be8cedeaf9f92bddb8a5d7101b50e7e494357c2fdcdb16b394
658:name=freeze file=test_executor_on_synthetic_pool.py sha256=8b745ac92da2db667f96fb7d756a25faa3a05c9f23d7619dc74a3105ddb435f3
659:name=freeze file=version_table.py sha256=d407a8c1a53f58aa5a85a6af4eff215844b838d0a48871aaee4b567446a52ba4
660:name=freeze_seed sha256_prefix16=85441ae491419411
```

Four hashes (`crash_dump.py`, `crash_image.py`, `crc32c_castagnoli.py`, `replay_prefix.py`) match the original freeze; the other 11 changed. The executor was not changed after freezing; stages 2 and 3 check these 15 lines individually.

#### Five changes (Q1-d)

| Change | Files and functions | Basis |
|---|---|---|
| c1 magic | Constants `SYSTEM_CONFIGURATION_MAGIC`, `ROOT_RECORD_MAGIC`, `JOURNAL_RECORD_MAGIC` in `on_disk_layout.py`; `slot_is_self_verified`, `root_is_self_verified`, `verify_record` check magic and reject the instance on mismatch, without G01. | Three magic rows in `.claude/kb/layout/01-first-txn.md`: SFSB, SFSR, SFSJ; line 160 includes magic, whole-slot checksum, format version, incompat bits and encryption type in self-validation. |
| c2 offsets | `on_disk_layout.py` adds `assert_every_offset_the_layout_lines_list_holds` and transcribed offset tables: 46 configuration fields plus three cleaning watermarks and per-region device identity suboffsets; 22 journal-header fields; 15 root fields; three location-entry sections and pointer offsets 50, 64 and named-item offsets 0, 14. Assert each on load; `spans_from_row_order` cites four layout passages; no G02. | Layout's configuration offsets, 14-byte location-entry internals, header offsets and milestone paragraph in section 7 all state row order equals on-disk order. |
| c3 slot-1 offset | `choose_system_configuration` reads slot 0 on each disk, then uses ① that disk's usable slot-0 spacing; ② first usable slot-0 spacing in device order, `spacing_from_first_usable_slot_zero`; ③ when no usable slot 0 exists, try each spacing and choose greatest generation, then smallest offset on tie. Usability is `slot_zero_is_usable`; `SlotOneReading` records path and offset. No G03. Device order is undefined in KB, recorded as G21. | D22 settled item 16 and layout's system-configuration slot positions. |
| c4 two exhaustive-search exits | `slot_one_by_trying_every_spacing`: a self-valid slot with out-of-range pool-level fields refuses the entire pool (L = none, existing `pool_level_refusal_of_slot`); unrecognized incompat bits make a slot unselectable without offset check, still G14. Their ordering is unspecified, G22. | D22 item 16 and the layout slot-location passage. |
| c5 gap table | `spec-gaps.tsv` marks G01–G03 filled on 2026-10-02 and cites KB in column 9; first eight columns of G04–G20 stay byte-identical with `-` in column 9; add G21 and G22 and identifiers in `spec_gaps.py`. | Layout lines 109, 369, 338, 158, 216, 359, 365, 160 and D22 item 16, cited per row. |

Five changes falling outside the 5.2 table (registration V10: any place lacking grounds is recorded as lacking grounds, and Q1's verdict is referred to the lead agent; each item provided grounds within the registration, and whether they count as falling outside 5.2 is for the lead agent to judge):

1. `executor_main.py` `anchor_assertions` changes a1 to `allowed == [(1, 2), (1, 3)]`, and adds a12 (three magics, selected slots, four roots and first four bytes of six records) and a13 (both stream-1 F disks take ① for slot 1 at offset 4096), per section 7.
2. Reporting fields (preregistration revision 1 item 4): in `executor_core.py` `judge_state`, added `slot_one=` (which step each disk followed, selected slot number, offset, and generation number) and `chosen_below_largest_self_verified=` to summary and verdict rows; in `executor_main.py` `summarise_verdicts`, added to `name=path_counts` the disk × state counts for slot 1 following steps ①②③ and for step ③ receiving >= 2 slots as required by Q2-e, and newly added `name=gap_trajectory` (per-segment counts for Q1-a in section 8).
3. Remove unused `gaps` parameters from `slot_is_self_verified` and `root_is_self_verified` after c1, along with a `version_table.py` call argument and unused import.
4. Change synthetic fixture magics to literal SFSB, SFSR, SFSJ and add `magic` parameters to `root_record` and `journal_record`.
5. Add U1–U6, U8 and three revision-1 supplements to `test_executor_on_synthetic_pool.py`; rewrite old G01–G03 tests for c1–c5.

#### Q1-f: KB support for G04–G20; step-7 unit magic

Section 5.2 permits only five changes. These rows are documented without changing code; failure to find a sentence by the keyword grep does not prove it absent.

| Row | KB statement | Does using it change the trigger? |
|---|---|---|
| G13 | The passage "**Location of system configuration slot 0 and slot 1**" in `.claude/kb/layout/01-first-txn.md` lists format version among the items for "self-validation" (failure means the slot is unavailable); I-7.13 (system configuration pool-level fields within range accepted by readers) of `crates/singlefs-checker/invariants/02-根环.md` still lists format version among the eight items for whole-pool rejection | No: the two sentences coexist, still representing two interpretations; trigger condition remains as before: "a slot appears that passes self-validation but whose format version is not 1" |
| G14 | D22 item 16 and layout line 160: in exhaustive search, unknown incompat bits make a slot unselectable without offset check; slot-0 usability includes incompat. | No: the new text covers ③ and slot-0 usability, not slot 1 read by ① or ②. Trigger remains a self-valid slot whose incompat bits are not exactly {bit 1}; provisional unselectable interpretation agrees. |
| G19 | Scope of D22 (composition of unit atomicity) settled item 16: the rule "when generation numbers are equal take the smaller offset" only has an object on residues created by media damage after mkfs zeroing; it is a tie-breaker rule between candidate slots within a tier (③) | No: tie-breaking between slot 0 and slot 1 is still not written in the KB (I-7.16 (relationship between system configuration slot generation number and slot number) only specifies "when equal, both are only permitted to be 1", without specifying which slot readers take) |
| G04–G12, G15–G18, G20 | Grepped using keywords across `decisions/22`, `decisions/23`, `crates/singlefs-checker/invariants/`, and `layout/01` (pool-level geometry, root slot location, fsid, either copy, chain head, first entry sequence number, prefix of at most N entries, same txg, commit marker, root slot width, end-of-stripe flags for two stripes), finding no new sentences that exist only now (matches were only the existing items in each row's "Basis" column) | No sentences to rely on, trigger condition unchanged |
| Step-7 named-unit `SFSU` | The magic row of the unit-header field table in `.claude/kb/layout/01-first-txn.md` specifies unit-header magic values; the table "Settled item 7: first version unit-header field table by unit class" in `.claude/kb/decisions/18-块里携带什么信息.md` writes the magic row as a scanning probe ("reject via magic + header checksum", used on scanning paths). As for how named units are validated in replay, item 4 of D23 (journal role and format) settled item 14 and D16 (publish semantics) settled item 7 only state "both copies can be read and checksums are correct" | Not a single sentence lists magic in step 7: step 7 does not check `SFSU`, and no new specification gap is opened |

#### Unit tests and mutations (stage 1)

- All 53 unit tests passed via an in-memory wrapper (`e170-r1-o3-spec-executor-2026-10-03-s1-unit-tests.out`: `Ran 53 tests`, `OK`). U1's three magic branches, U2–U6, U8, c2 offset assertions, G21 and G22 have tests; U7 is omitted as registered.
- Mutation table `scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv` (21 rows: x1–x20, XC2; XC1 awaits writing the comparator in stages 2 and 3, and XE4 is appended to `crates/mutations.tsv`, neither in stage 1). Unit-test tier outcomes (replacing, compiling, and running 53 unit tests one by one on a copy of the executor directory with driver `mutation_driver.py`, unfrozen): trailing line of `e170-r1-o3-spec-executor-2026-10-03-s1-unit-level-mutations.out` reports `counts rows=21 caught=21 invalid=0 survived=0 anchor_error=0`; all 21 rows were caught by unit tests, with 0 invalid and 0 failing to turn red. These are numbers at the unit-test tier, not the final three numbers of preregistration section 9: sensitive points for x1–x11 on batch states of both streams require verdicts of stage 3; x2 still has no sensitive points on batch states, the sensitive point for x10 is only in synthetic pool unit tests (added by preregistration revision 1 item 1), and sensitive points for x12–x19 are only in synthetic pool unit tests, producing no output changes on batch states of both streams.
- Initially x9 and x10 survived and x17 was caught only by G22's test. Revision 1 specified added samples, U5 generation and x20 replacement; after these changes all 21 were caught.

#### Path counts (auxiliary, original export batches 1 and 2)

Four `name=path_counts` lines, artifact 498, 503, 637, 642: batch 1 replayed ≥ 1 record in 6 + 51 states. Slot 1 used ③ for 4 + 24 = 28 disk × states (both disks in 14 states), ② for 8 + 48; ③ collected ≥ 2 slots zero times; `chosen_root_below_largest_self_verified_txg` and `pool_refused` were all zero. Batch 2 used ① throughout. Choosing maximum generation and breaking equal-generation ties by smaller offset under ③ were covered only by synthetic tests U3 and U4.

#### Stage 2: re-export and Q2

**Re-export:** The unchanged 2026-10-03 `dump` exporter ran twice, to `scripts/experiment-apparatus/target/e170-second-run-dumps-v2-first-run/` and then `scripts/experiment-apparatus/target/e170-second-run-dumps/`, comparing with the first. Both exited 0 with six-thread and 4 GB limits; an earlier 8 GB wrapper invocation exited 252 before the command ran, so its output is excluded. `e170-r1-o3-spec-executor-2026-10-03-s2-export.out` has 142 `E7RESULT` lines, ending `name=done emitted=142`; the provenance artifact records three git identifiers and file comparisons, excluding changing git identifiers from `replay.sh`.

- Both batch-1 sets are the same under interpretation 2: the eight non-timing files per stream, 16 total, match the original `scripts/experiment-apparatus/target/e170-dumps/` byte for byte. After removing timing, `impl-verdicts.tsv` matches for all 82 and 890 states. Literal V2 triggers on four timing-related files, V2b does not; V2b is accepted as in the first run. Q1-g is not counted (G6: same batch, one point).
- Batch 11 mkfs added one zero write per disk before the recorded stream, without changing any nonzero sector, so the stream is unchanged. Only two absolute self-check values changed: mkfs operations local and crates counts 23 → 25 and warm-up return point 45 → 47; both hold.
- The implementation is the workspace `crates/` at re-export, `git_head=fcc4d3404c806b83f335d1c95eb303e5fe287fd2`; staged and working-tree diff hashes are provenance lines 5–6.

**Executor and comparator:** The frozen executor (15 files, seed `85441ae491419411`) judged batch 1 and batch 2 in the re-export directory as-is, producing artifact `e170-r1-o3-spec-executor-2026-10-03-s2-executor.out` (661 lines; 15 lines of `name=freeze` + `name=freeze_seed` are line-for-line identical to lines 645–660 of the stage 1 artifact, with `diff` producing no output); under strict execution mode, all 576 states (53 + 397 of batch 1, 7 + 119 of batch 2) were `strict=in`, `digest=match`, and 0 states hit a specification gap. Comparator `comparator_q2_implementation_verdicts.py` (in the new directory, filename starting with comparator, not frozen) was copied from the first run and modified to read the strict execution column, with Q2-pc taking the first state where strict execution mode yielded a verdict in batch 1; artifact `e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out`.

`e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out`:

```
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 2 (name=freeze_check)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 5 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 17 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 12 (name=q2)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 33 (name=q2_positive_control)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out, line 34 (name=verdict)
```

| Measure | Value | Evidence |
|---|---|---|
| Q2-a state-by-state differences in re-exported batch 1 | 0 / 450 (stream 1: 53, stream 2: 397, 0 differences each) | The row `name=q2 measure=q2a` in artifact (citation line 2); `grep 'name=q2_cross measure=q2a' …-s2-q2-comparator.out` outputs `exec_in_impl_pass=53` and `exec_in_impl_pass=397` |
| Q2-c strict gap, excluded | 0 | `not_given=0` |
| Q2-d batch 2 | 0 / 126, same direction, F7 does not trigger | Comparator |
| Q2-e paths | ≥ 1 replayed record: 6 + 51; slot 1 via ③: 4 + 24 disk × states, ①: 94 + 722, ②: 8 + 48; chosen root below largest self-verified txg: 0; ≥ 2 slots in ③: 0; out: 0; torn-write images: 20 + 115; pool refusal: 0. Both F6 thresholds pass, but zero-count paths are not covered by agreement. | Executor `name=path_counts batch=1`, lines 498 and 503 |
| Q2-f differing (instance generation, txg) landings | 0 | `q2f_landed_pairs_differ=0` |
| Q2-g provisional vs implementation | 0 / 450; batch 2 0 / 126 | Comparator |
| Q2-pc | 1; V5 does not trigger | Positive control |
| Trajectory | Batch 1: 12 and 79 segments, peak 0, zero positive segments, last 0. Batch 2: 1 and 17 segments, likewise zero. | `name=q2_trajectory` |
| Q2-i re-export vs original | 16 non-timing files identical; zero state differences after removing timing from `impl-verdicts.tsv`. | Provenance |

Verdict lines (two artifacts of the second stage): the line matching `grep -n 'name=verdict' e170-r1-o3-spec-executor-2026-10-03-s2-export.out`, with `s1_failed_checks=0 s4_digest_mismatches=0 …`, has no false / not_run fields, violation counters are all 0, and the only non-zero field is `v2_differing_files=4` (the four timing-bearing files triggered by literal V2); the four `name=verdict` lines in executor artifact (lines 496, 501, 635, 640) have `strict_spec_gap_states=0`, `judgement_failures=0`, and `digest_mismatches=0`; the comparator artifact has `name=verdict all_agreed=true diff_count=0` (line 16).

Verdict for Q2: consistent (strict execution mode, 0 differences out of 450 states in re-exported batch 1, 0 differences out of 126 states in batch 2, both points in the same direction). Beyond literal reading: the executor judged "in" for all 576 states, and the implementation passed all states; the cross-table has only the single cell "executor in, implementation passes", and the cell where executor is "not in" while implementation is red has zero states in the batch; F1 did not trigger (Q3 discriminative power is established); both threshold conditions of F6 (replay >= 1, ③ > 0) passed, but the two execution paths "the selected root is not the root with the largest txg among self-validated roots" and "received >= 2 slots in ③" were 0 in the batch, so "consistent" does not cover them.

#### Stage 3: new exporter mode, bad-state table and Q3

New mode `bad-states-second-run --dump-directory <export directory> --seed-hex <seed>` uses the same checked-in exporter `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`, read-only, driving and observing without reading any new `crates/` constants. With the same dump and seed its rows match original `bad-states` except P8's four expectations change to `in` with landing unchanged at v_{n−1}, and P10 is appended per stream. P10 applies P8's modification on Mk⁺: flip one bit at root n offset 122 and one in each copy of record n, without resealing; expect out, landing v_{n−1}. P10 follows the blind family and draws no random number, leaving other rows, patches and blind samples unchanged. `BadTableVersion` (`FirstRun`, `SecondRun`) is passed to `bad_rows_of_stream` and `bad_states_of_stream`; `BadShape::NewestRootAndRecordUnreadable` takes expected verdict, with the same patch in all three uses. P10 is a separate family outside Q3-a. Original modes only change the table-version argument.

- **S5:** Copy the re-export directory twice before any `bad-*` files, run original and new modes with original seed `8546b1258e427c34`. For both streams the first 38 rows of `bad-states.tsv`, `bad-impl-verdicts.tsv`, `bad-seed.tsv` match; row 39 is P10. Only P8's expectation in rows 5 and 11 changes from `not_in` to `in`. Original `bad-patches.bin` is a prefix of the new one, 157192 vs 157195 bytes, adding one root and two record patch bytes. Commands and results are `name=s5_file` and `name=s5_expected_diff` in provenance.
- **Original mode unchanged:** The 2026-10-03 original mode run on a copy of the re-export directory produced `bad-states.tsv`, `bad-patches.bin`, `bad-expected.tsv`, and `bad-seed.tsv` that are byte-for-byte identical to the files of the same name in the first export (source file `name=original_mode_vs_first_run`, four `identical=true` entries for each of the two streams); `bad-impl-verdicts.tsv` differed because the implementation changed (counts are recorded in the paragraph "Q3-f and comparison with the first run"). Standard output of the original mode differed by 44 lines (22 pairs) from the first-run artifact `e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out`: two apparatus self-check `bad_s1` rows changed from 23 / 45 to 25 / 47, and the remaining 20 pairs were columns and counts of implementation verdicts in `bad_row`, `bad_family`, and `bad_file`. The 2026-10-03 output was saved as `e170-o3-spec-executor-2026-10-02-s3-bad-states-export-2026-10-03.out` (175 lines), and the row registering E170 (first O3 specification executor) bad-state export in `replay.sh` was redirected to point to it.
- **S5b:** Each stream's P10 base persistence vector and base image hash equal P9-Mk+ in that stream; presence of the two rows also agrees. Three `bad_s1` checks per stream yield `bad_s1_failed_checks=0`.

Artifact `e170-r1-o3-spec-executor-2026-10-03-s3-bad-states-export.out` (formal single copy, seed taken from the first-stage freeze row `85441ae491419411`):

```
E7RESULT name=config segment=s3 mode=bad_states_second_run seed_hex=85441ae491419411 seed_u64=9602829874452599825 blind_rows_per_stream=20 patched_writes_kind=system_configuration_slot judge_threads=1
E7RESULT name=verdict bad_s1_failed_checks=0 bad_rows=78 impl_panics=0
```

The 78 rows comprise per stream 38 rows (13 P: P1–P5 and P8 on F and Mk, plus P9; five N; 20 blind) plus P10. Verdict: `bad_s1_failed_checks=0`, `impl_panics=0`.

**Comparator and verdict:** The new directory's `comparator_q3_bad_states.py` was copied and changed to use strict verdict, separate P10, add Q3-pc, and verify freeze lines against stage 1; it works with `comparator_mutation_checks.py`. The frozen executor builds images from base vectors, overlays patches and judges without opening `bad-expected.tsv` or `bad-impl-verdicts.tsv` in its judging half. The 168-line artifact `e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out`:

```
Artifact line: git show db44c1542cef:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out, line 2 (name=freeze_check)
Artifact line: git show db44c1542cef:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out, line 165 (name=q3_counts)
Artifact line: git show db44c1542cef:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out, line 166 (name=q3_positive_control)
Artifact line: git show db44c1542cef:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-bad-states.out, line 167 (name=verdict)
```

| Measure | Value | Evidence |
|---|---|---|
| Q3-a differing verdict among 26 P instances | 0; four P8 in, 22 P1–P5/P9 out; zero on F, Mk and Mk⁺, G5 same direction | `name=q3_counts scope=both` |
| Q3-b blind rows in | 0 / 40, 20 per stream | `name=q3_counts` |
| Q3-c strict gaps | 0 in 68 P, blind and P10 rows | `strict_spec_gap_rows=0` |
| Q3-d P landing differences | 0: P1 L = none, P8 L = v_{n−1}, others outside version table | Same counts |
| Q3-e N instances out | 0 / 10; landing differences 0 | Same counts |
| Q3-f implementation accepts both checks | 15 / 68: P 9 / 26, P10 2 / 2, blind 4 / 40 | Export `name=bad_family` |
| Q3-g P10 in or landing differs from v_{n−1} | 0 / 2; F10 does not trigger | `q3g_p10_judged_in_or_landing_differs=0` |
| Q3-h provisional Q3-a and Q3-b | P errors 0, blind in 0; registered only for Q3-c > 0, computed incidentally here | `q3h_` fields |
| Q3-pc | 1: flip the new-pool file creation stream's P1-F expectation and find exactly one difference; V11 does not trigger | `q3_positive_control` |

Q3 is discriminating: all 26 P instances and 40 blind rows match expectations in strict mode; Q3-c is zero and all are decided. F and Mk agree in direction (G5). The original loss, Q3-a 4 / 26 with P8 expected out, remains recorded. P8 was corrected by preregistration 5.5 before any rerun artifact, based on interpretation 3; all four verdicts and landings remain identical to original observations, serving only as regression checks. Discrimination rests on P1–P5, P9 and the new-seed blind family. P10 is outside Q3, both correct.

**Q3-f comparison with original:** On the same original bad-state table and seed, the 2026-10-03 implementation differs. `bad-impl-verdicts.tsv` changed from 7143 to 7378 bytes for stream 1 and 7799 to 8110 for stream 2; implementation columns changed in 14 `bad_row` lines, five and nine respectively. The 44 differing output lines comprise two self-check, 14 bad-row, four bad-family and two bad-file pairs. Recomputing with the original comparator gives 21 rather than 28 of 66 P and blind rows accepted by both checks: stream 1 18 → 14, stream 2 10 → 7. Fewer accepted means the post-batch-11 implementation is stricter on these states; Q2 and Q3 verdicts do not change, but the original upper bound 28 does not reproduce on 2026-10-03.

#### Mutations (stage 3, registration section 9)

`scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv` has 22 rows x1–x20, XC1 and XC2, with XC1 after XC2. Unfrozen `mutation_driver_with_batch_and_bad_state_checks.py` leaves stage 1's unit-level driver unchanged. Each copy is mutated, compiled and run against 53 unit tests and `comparator_mutation_checks.py`, which has 172 named baseline assertions plus `__complete__`: verdict and landing for 78 bad states, Q3-pc, anchors, batch-1 rejudgment against implementation and Q2-pc. A formerly true assertion turning false or a red unit test catches the mutation. Artifact `e170-r1-o3-spec-executor-2026-10-03-s3-mutations.out`:

```
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-mutations.out, line 24 (name=baseline)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-mutations.out, line 47 (name=restored)
Artifact line: git show 5d0b7e7c1667:research/results/e170-r1-o3-spec-executor-2026-10-03-s3-mutations.out, line 48 (name=verdict)
```

- Caught 22, invalid 0, survived 0, crash 0 and timeout 0. All 22 anchors hit exactly once; final `restored baseline_still_green=true`.
- Eleven also go red on batch or bad-state checks: x1, x3–x9, x11, x20, XC1. Eleven are caught only by synthetic unit tests and have no sensitive batch or bad-state sample: x2, x10, x12–x19, XC2. x2 has no registered sensitive point and three red unit tests; x10 and x12–x19 are sensitive only in synthetic tests, XC2 at U8.
- Section 9 cases: x8 changes both P10 verdicts from out to in and makes a1's two-element half red, three `red=` entries; x9 hits P9-Mk+ on both streams; x6 hits stream-1 a4; x4 hits four P4 rows; XC1 hits Q2-pc `q2_pc:exactly_one`, with all 53 unit tests green.
- Driver is not `mutate.sh` and has no memory-limit / timeout columns; wrapper exits 0, timeout and crash counts 0.

**XE4**, changing P10's new-mode base from Mk⁺ to Mk, is appended to `crates/mutations.tsv`. Named test `the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base` builds both tables for stream 1 and checks only P8's verdict column differs, P10 is last, its base vector matches P9-Mk+ and patch matches P8-Mk. The exporter has 18 unit tests, including two new stage-3 tests for CLI and table comparison; `compile-then-swap.py` compiled `cargo test --no-run` in a repository copy but did not run them (checker tier). The full `crates/mutations.tsv` rerun belongs to submission gate checker-tier-crates-mutation-replay; XE4's three counts await that gate. Separately the mutated exporter was compiled and its new mode run in a draft repository copy, not as a unit test: four S5b checks, vector and digest on both streams, turned `holds=false`, `bad_s1_failed_checks=4`, exit 3.

#### Measurements not run after sufficient evidence

All three questions were decisive: strict batch 1 in both streams for Q1, state-by-state re-exported batch 1 for Q2, every bad-state row in strict mode for Q3. Not run: Q1-g because batches are identical; batch 3 and Q4 because not applicable to this question sheet; full `crates/mutations.tsv` replay and exporter unit tests, reserved for the checker-tier submission gate; and heavy tier `replay.sh` replay, also reserved for submission. Q3-h is registered as unreported when Q3-c = 0, but the comparator computed it incidentally and the table reports it.

### Second rerun (add predicates W and T and 487-byte header, 2026-10-04)

Preregistration `research/prompts/e170-r2-prereg.md`, section 12 revisions 1 and 2 and five final main-agent determinations; question sheet `e170-r2-questions.md`: Q1, does v2 match the first-rerun executor item by item on all controls and Q2 bad images, excluding cells the two changes should alter? Q2, after W, are E171 rows 242 and 269 states out or in? Q3, after requiring witness ≥ acknowledged version's rotation tail, are row 270 states out, and are legal histories falsely red? Q4, which O3 executor should E171's first rerun use? Q5, after the format update, do controls and Q2 bad images re-exported from the current tree match v2 on s4 exports cell by cell? W is the writable mount witness predicate, T is witness ≥ acknowledged version's rotation tail. Scope and criteria are fixed in section 1 and revision 1. All five are decisive:

| Question | Verdict | Evidence |
|---|---|---|
| Q1 | A: same | `e170-r2-o3-spec-executor-2026-10-04-s2-comparator-q1.out`: zero out-of-scope differences; eight in-scope differences are exactly the eight input-3 and input-4 P8 cells. |
| Q2 | A: out | `s2-comparator-q2-q3.out` `name=q2_counts row=242` and `row=269`: all 15 and 63 K states out. |
| Q3 | A: row 270 out, zero false reds on legal histories | Same artifact lines 47, 68–70: all 68 K270 states out; zero false reds in 1,168 legal history states and 792 + 972 geometry states. |
| Q4 | A: use v2-487 for E171's first rerun | Revision 1 interpretation 15: Q1 A, Q2 A, Q3 not C, Q5 A. |
| Q5 | A: isomorphic | `s2-q5-comparator.out` verdict `q5a=0`, `q5_verdict=jia`. |

#### Four arms

| Arm | Directory under `scripts/experiment-apparatus/` | Freeze | Description |
|---|---|---|---|
| v1 | `e170_second_run_o3_spec_executor/` | 15 files, seed `85441ae491419411` | First-rerun frozen version unchanged byte for byte; reads 311-byte journal header. |
| v2 | `e170_o3_spec_executor_with_witness_predicates/` | 16 files, seed `9640bd162f9c7d86` | Copy v1's 15 frozen files; new `witness_predicates.py` computes W and T. Original v1 decision is the pre-verdict; if pre-verdict is in and W true or T false, decide out, otherwise retain it. Reads 311 bytes. |
| v2-487 | `e170_o3_spec_executor_with_witness_predicates_journal_header_487/` | 16 files, seed `ce39b484ec109c7a` | v2 format patch: 487-byte journal header and seven new-root fields, using KB journal format and registration 12.8 expressions, not `crates/`. |
| v1-487 | `e170_second_run_o3_spec_executor_journal_header_487/` | 15 files, seed `0ae1d855d57a2ff6` | v1 with the identical format patch; only used to judge Z family in stage 4, which did not run. |

#### Inputs

| Input | Source | States |
|---|---|---|
| Input 1 | `scripts/experiment-apparatus/target/e170-dumps/`, original 311-format export | Two streams, batches 1 and 2: 576; batch 3 geometry points: another 396. |
| Input 2 | `scripts/experiment-apparatus/target/e170-second-run-dumps/`, first-rerun stage-2 re-export | Same as input 1. |
| Input 3 | Bad-state table in same directory | 39 rows per stream plus eight unpatched images for four bases across two streams: 86. |
| Input 4 | `scripts/experiment-apparatus/target/e171-dumps/e170-bad-states-copy-s2/` and `known-bad-s2/` | 38 rows per stream, 41 known-bad images, eight bases: 125. |
| Input 5 | Original v1 frozen unit-test file run against v2 modules | 53 tests. |
| Rows 242, 269, 270 | `scripts/experiment-apparatus/target/e171-dumps/rows-s4-f4/row-<row>/dump/`, exported by individually driving rows on E171's frozen s4 tree | 730, 576, 516 states. |
| s4 baseline | `scripts/experiment-apparatus/target/e171-dumps/baseline-run1/` | Batches 1–3: 972 states, Q3 geometry points. |
| Z1, Z3, Z4, Z5 | Re-export on the 2026-10-04 tree under `scripts/experiment-apparatus/target/e170-r2-today-dumps/`: `e170-dump`, `e170-bad-states-second-run`, `e170-bad-states-copy-s2` plus `known-bad-s2`, `e171-baseline-run1`, in 487 format | Z1 and Z5: 972 states each in batches 1–3; Z3: 78 bad rows and eight base images; Z4: 76 bad rows, 43 known-bad images, eight bases. |

Inputs 1–5 are I1–I5 in interpretation 5; none of these dumps are tracked in git. Inputs 1–4, the three s4 rows and s4 baseline were SHA-256 checked against `manifest.tsv` in `e170-r2-o3-spec-executor-2026-10-04-s2-inputs.out`; Z inputs in `s2-q5-inputs.out`.

#### Stages and artifacts

| Stage | Work | Artifacts (prefix `e170-r2-o3-spec-executor-2026-10-04-`) |
|---|---|---|
| Stage 1 A | Write and freeze v2; v2 judges input 1 batches 1–2; copy v2-487 and v1-487 and apply the same format patch; unit tests and unit-level mutations for all three. | `s1-executor`, `s1-unit-tests`, `s1-unit-level-mutations` and six `s1a-` files without `-2026-10-05`. |
| Z family re-export, started by main agent three times | Export Z1, Z1r (repeat Z1 for determinism), Z3, Z4, Z5 on the 2026-10-04 tree. | `s2b-today-export`, `s2b-today-export-r2`, `s2b-today-export-r3`; third is valid. |
| Stage 1 B | v2-487 judges Z1 batches 1–2, format anchors, freeze v2-487 and v1-487. | `s1b-executor` |
| Stage 2 | Q1–Q3: rerun v1, v2 judges inputs 1–4, three s4 rows and s4 baseline; reference predicates and comparators. | 35 `s2-` files excluding `s2-q5-`. |
| Stage 2 Q5 half | v2-487 judges Z1, Z3, Z4, Z5; Q5 comparator, Q5-d, Q4-fmt-487 and known-bad recopy verification. | 14 `s2-q5-` files without `-2026-10-05` and three `s2b-known-bad-recopy` files. |
| Stage 3 | Mutations testing Q1–Q3 discrimination. | `s3-mutations` |
| Stage 4 | v1-487 judges Z family only if Q4 is B or all Q5 B differences are class (ii). | Not run after sufficient evidence. |
| Replay registration (2026-10-05) | Nine replay artifacts in result-line form. | See replay section. |

`ls research/results/ | grep '^e170-r2-'`: 75 files, 66 from the second rerun and nine from the 2026-10-05 replay registration:

```
e170-r2-o3-spec-executor-2026-10-04-s1a-format-patch-comparison.out
e170-r2-o3-spec-executor-2026-10-04-s1a-format-patch-comparison-r2.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-level-mutations-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-level-mutations.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-tests-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-tests.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-level-mutations-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-level-mutations.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-tests-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-tests.out
e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out
e170-r2-o3-spec-executor-2026-10-04-s1-executor.out
e170-r2-o3-spec-executor-2026-10-04-s1-unit-level-mutations.out
e170-r2-o3-spec-executor-2026-10-04-s1-unit-tests.out
e170-r2-o3-spec-executor-2026-10-04-s2b-known-bad-recopy-mutations.out
e170-r2-o3-spec-executor-2026-10-04-s2b-known-bad-recopy.out
e170-r2-o3-spec-executor-2026-10-04-s2b-known-bad-recopy-pc3.out
e170-r2-o3-spec-executor-2026-10-04-s2b-today-export.out
e170-r2-o3-spec-executor-2026-10-04-s2b-today-export-r2.out
e170-r2-o3-spec-executor-2026-10-04-s2b-today-export-r3.out
e170-r2-o3-spec-executor-2026-10-04-s2-comparator-q1.out
e170-r2-o3-spec-executor-2026-10-04-s2-comparator-q2-q3.out
e170-r2-o3-spec-executor-2026-10-04-s2-i5-v1-tests-and-helper-on-v2.out
e170-r2-o3-spec-executor-2026-10-04-s2-i5-v1-tests-on-v2.out
e170-r2-o3-spec-executor-2026-10-04-s2-inputs.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-comparator.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-format-constants.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-inputs.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-pc-comparator.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-pc-v2-z1.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-pc-v2-z3-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-pc-v2-z4-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-pc-v2-z5.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-record-paths.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-run-status.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z1-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z1.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z3-images-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z3-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z4-images-2026-10-05.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z4-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z5.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-e171-s4-baseline.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-hand-made-cases.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-i1.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-i2.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-i3-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-i4-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-row-242.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-row-269.out
e170-r2-o3-spec-executor-2026-10-04-s2-reference-row-270.out
e170-r2-o3-spec-executor-2026-10-04-s2-run-status.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-against-stored.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-i1.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-i2.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-i3-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-i4-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-row-242.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-row-269.out
e170-r2-o3-spec-executor-2026-10-04-s2-v1-row-270.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-e171-s4-baseline.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-i1-against-segment-one.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-i1.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-i2.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-i3-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-i4-images.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-row-242.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-row-269.out
e170-r2-o3-spec-executor-2026-10-04-s2-v2-row-270.out
e170-r2-o3-spec-executor-2026-10-04-s2-x21-row-242.out
e170-r2-o3-spec-executor-2026-10-04-s2-x21-row-269.out
e170-r2-o3-spec-executor-2026-10-04-s2-x27-row-270.out
e170-r2-o3-spec-executor-2026-10-04-s3-mutations.out
e170-r2-o3-spec-executor-2026-10-05-s2b-bad-states-second-run-export.out
e170-r2-o3-spec-executor-2026-10-05-s2b-dump-compare.out
```

#### Q1: item-by-item comparison of the two versions

Compare current v1 with v2 on inputs 1–5, v1 `strict=` and `provisional=` against v2 pre-verdict fields, excluding `time_ns` and `worker`. Out-of-scope differences: 0. In scope: (a) eight, exactly P8-F and P8-Mk on both streams for inputs 3 and 4, v1 in → v2 out with `failed_predicates=W`; B5 holds 8 / 8. Categories (b) and (c): 0. Current v1 matches seven checked-in artifacts per state (Q1-d 0, V9 does not trigger). All 53 input-5 unit tests pass. Q1-pc flips one state and the comparator reports exactly one additional difference, V5 does not trigger.

```
name=i5_counts ran=53 passed=53 not_passed=0 final_ok=true
name=verdict fields_differ=0 legal_flip=0 patched_flip_without_predicate=0 not_in_to_in=0 other_verdict_change=0 in_range_a=8 in_range_b=0 same=1355 compared=1363 missing=0 out_of_range_total=0 legal_flip_with_W=0 legal_flip_with_T=0 q1g_before_not_in_with_W_true=42 q1g_before_not_in_with_T_false=0
name=verdict q1d_differing_or_missing_states=0 v9_triggered=false
```

There are zero `name=q1_out_of_range` lines. Auxiliary Q1-g `q1g_before_not_in_with_W_true=42` counts patched states already out where W is true, 22 from input 3 and 20 from input 4; verdicts do not change. v2's current input-1 batches 1 and 2 match stage 1 state by state and field by field, Q1-d 0, V9 not triggered.

#### Q2: W, rows 242 and 269

K consists of states where reference predicate W_ref is true. All 15 K states on row 242 and 63 on row 269 are strictly out under v2 (Q2-a 0); none hit a gap. Both batches individually choose A, so F5 does not trigger. v2 W and W_ref disagree on zero states (Q2-c 0; S1 does not trigger). All three W branches occur: row 242 `last_record` 5, `record_at_counter` 10; row 269 `last_record` 47, `record_at_counter` 6, `both_unreadable` 10, matching reference branch by branch. Discrimination: x21, W always false, judges 15 and 63 K states in, so F7 does not trigger; Q2-pc flips a state and adds exactly one difference.

```
name=q2_counts row=242 K=15 q2a_K_judged_in=0 q2d_K_spec_gap=0 q2c_witness_disagreements=0
name=q2_counts row=269 K=63 q2a_K_judged_in=0 q2d_K_spec_gap=0 q2c_witness_disagreements=0
name=mutation_run mutation=X21 row=242 states=730 judged_in_inside_the_class=15 at_least_one=true
name=mutation_run mutation=X21 row=269 states=576 judged_in_inside_the_class=63 at_least_one=true
```

Auxiliary Q2-i: states with fewer than two self-valid system-configuration slots on some disk are 245 / 730 on row 242 and 135 / 576 on row 269; O3 does not judge this. Under E171 interpretation 3's batch-level row verdict, v1 accepts both batches and v2 rejects both for each row (Q2-f).

#### Q3: T, row 270 and legal histories

K270 consists of 68 states with reference T_ref false: stream 1 3 + 2 and stream 2 30 + 33, batches 1 + 2. v2 judges all out (Q3-a 0). Legal history comprises 1,168 states from input 1 and 2 batches 1–2 plus eight unpatched bases each in inputs 3 and 4; geometry points comprise input 1 and 2 batch 3, 792 states, plus the 972-state s4 baseline. All three groups have zero T-induced false reds and zero W-induced flips. T is evaluated on 1,034 previous-legal-history states and not on 134; 672 lie on the boundary witness = τ. Both Q3-e quantities are positive, F4 does not trigger. v2 T and T_ref differ on zero states (Q3-c 0). x27, T always true, judges all 68 K270 states in; Q3-pc and Q3-pc2 each add exactly one difference.

```
name=q3_row_counts row=270 K270=68 q3a_K270_judged_in=0 K270_spec_gap=0 q3c_tail_check_disagreements=0
name=mutation_run mutation=X27 row=270 states=516 judged_in_inside_the_class=68 at_least_one=true
name=legal_group_totals group=previous_legal_history states=1168 q3d_misred_T=0 legal_flip_W=0 t_evaluated=1034 t_true=1034 t_false=0 t_not_evaluated=134 t_counter_equals_tau=672 t_counter_above_tau=362 ref_t_evaluated=1034 ref_counter_equals_tau=672 ref_w_true=0 q3c_tail_check_disagreements=0 witness_disagreements=0 digest_mismatches=0 reference_digest_mismatches=0 reference_missing=0
name=legal_group_totals group=batch_three states=792 q3d_misred_T=0 legal_flip_W=0 t_evaluated=792 t_true=792 t_false=0 t_not_evaluated=0 t_counter_equals_tau=616 t_counter_above_tau=176 ref_t_evaluated=792 ref_counter_equals_tau=616 ref_w_true=0 q3c_tail_check_disagreements=0 witness_disagreements=0 digest_mismatches=0 reference_digest_mismatches=0 reference_missing=0
name=legal_group_totals group=e171_s4_baseline states=972 q3d_misred_T=0 legal_flip_W=0 t_evaluated=906 t_true=906 t_false=0 t_not_evaluated=66 t_counter_equals_tau=640 t_counter_above_tau=266 ref_t_evaluated=906 ref_counter_equals_tau=640 ref_w_true=0 q3c_tail_check_disagreements=0 witness_disagreements=0 digest_mismatches=0 reference_digest_mismatches=0 reference_missing=0
```

Additionally, the reference artifact has one line per publish per stream, 176 total: zero cases where rotation-written tail differs from the last record counter in that publish, or the two disks' rotation tails differ (Q3-f). T was not evaluated for any of row 242's 730 states: 636 have no self-valid rotation write for that publish, 94 have ack at version 0. The faulty implementation orders system-configuration writes before root-slot writes, while interpretation 3 recognizes only rotation writes after the root-slot write and before the next barrier.

#### Q5: isomorphism after the 487-byte header update

v2-487 judges Z1 batches 1–3, Z3, Z4, Z5 batches 1–3; reference verdicts come from v2 on frozen 311 exports: primarily `e170-r2-o3-spec-executor-2026-10-04-s2-v2-i2.out`, with i1 auxiliary, i3-images, i4-images and e171-s4-baseline. Compare each cell under interpretation 14 by family and corresponding control. Zero evaluated cells differ: zero class (i), class (ii), or new-gap-only differences; the E171-tree subset Z4 and Z5 also has zero. Comparable counts are `e170_control` 24, `e171_control` 24, `base` 16, `p_rows` 54, `n_rows` 20, `blind` 80, `known_bad` 41; every family has ≥ 1, so F11 does not trigger. Two incomparable cells are Z4 known-bad image indices 41 and 42 because input 4 has only 41 rows, interpretation 19; v2-487 judges both out.

```
name=q5b_family family=e170_control comparable=24 differs=0 not_comparable=0
name=q5b_family family=e171_control comparable=24 differs=0 not_comparable=0
name=q5b_family family=base comparable=16 differs=0 not_comparable=0
name=q5b_family family=p_rows comparable=54 differs=0 not_comparable=0
name=q5b_family family=n_rows comparable=20 differs=0 not_comparable=0
name=q5b_family family=blind comparable=80 differs=0 not_comparable=0
name=q5b_family family=known_bad comparable=41 differs=0 not_comparable=2
name=verdict q5a=0 q5a_class_i=0 q5a_class_ii=0 q5a_only_new_gaps=0 families_without_comparable_cells=none q5_verdict=jia q5a_all_class_ii=false e171_tree_subset_q5a=0 b15_failures=0 b15_cells=8 b17_holds=true integrity_failures=0 q5_pc2_exactly_one=true
```

Prerequisites pass: Q5-d compares v2-487 format constants against `crates/singlefs-format/src/lib.rs` constants and `crates/singlefs-core/src/journal.rs` write order in the Z snapshot, zero differences among 36 items. Q4-fmt-487 values for v2-487, KB decision and implementation line 223 are all 487; layout's 311 is the stale sentence registered by interpretation 11. The 43 known-bad images were recopied from 2026-10-04 cases per revision 2: three counting methods 43 / 43 / 43, zero recopy differences and zero item differences (Q5-i, S6 not triggered). B15 holds 8 / 8 and B17 holds.

```
name=q5d_verdict differing=0 compared=36 stop_s4=false
name=q4_fmt_487 executor_v2_487=487 decision_23_item_4=487 decision_line=150 implementation=487 implementation_line=223 all_three_equal=true layout_line_357=311
name=known_bad_recopy_verdict counts_equal=True recopy_differences=0 item_differences=0 names_resolved_equal=True i4_target_names_equal=True
```

Discrimination: Q5-pc runs unmodified 311-byte v2 on Z, producing 161 differing cells all class (i), V14 not triggered. Q5-pc2 changes one copied row and Q5-a goes 0 → 1, V15 not triggered. Q5-pc3 changes one copied recopy-verification value, `recopy_differences=1`, V21 not triggered.

```
name=verdict q5a=161 q5a_class_i=161 q5a_class_ii=0 q5a_only_new_gaps=0 families_without_comparable_cells=none q5_verdict=yi q5a_all_class_ii=false e171_tree_subset_q5a=99 b15_failures=0 b15_cells=0 b17_holds=not_run integrity_failures=0 q5_pc2_exactly_one=not_run
```

Interpretation: cell descriptions use `bad-expected.tsv` columns 2–5 and 9, with recorded stream positions `return_point`, `root_operation`, `first_unit_write` in column 9 normalized as `#`; in stream 2 every such number in the 2026-10-04 export exceeds the reference by four. Literal byte comparison moves 41 cells from same to incomparable, 21 in Z3 stream 2 and 20 in Z4 stream 2. P, N and blind families retain comparable stream-1 cells and Q5 remains A, per main-agent stage-2 determination. Auxiliary Q5-e: 2,101 states have self-valid records on every disk; 170 replay to a version newer than the selected root; 2,009 have `witness_branch=last_record`.

#### Q4: which version for E171's first rerun

Revision 1 interpretation 15: Q1 A, Q2 A, Q3 A (not C), Q5 A, so Q4 A. Hand E171 interpretation 9 v2-487 at `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`. Its 17 frozen lines, 16 files and seed `ce39b484ec109c7a`, are in `e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out`; lines 753–769 tagged `arm=v2_487` are the same set. Ten `name=format_constants` lines from `s2b-today-export-r3.out` accompany it. Stage 2 Q5 verified the v2-487 freeze before and after the run against stage 1 B, V17 not triggered. The original interpretation-9 decision, auxiliary Q4-original, is also A (v2).

```
name=freeze file=crash_dump.py sha256=7a671b85d8346f2aee45982a888e2111e98495f5570ecfc57a348ee7595d30dd
name=freeze file=crash_image.py sha256=e391dbe8377014258f1978005062d4cb14891b8565ca420a6a5b731ad4387a5f
name=freeze file=crc32c_castagnoli.py sha256=34b9f571bc7d0e9dc32dce56b166536a6c26581b9f4744e7c3a14b0ef69bea6d
name=freeze file=executor_core.py sha256=37f22a1ef8589399221a308918b856a635bdf3ddaa2caefa954127aaf7b47776
name=freeze file=executor_main.py sha256=a9bd9104b386ea4f7d2747eb302759bf1e06d048163ef661426b0092b9d96429
name=freeze file=journal_ring.py sha256=19c2cbc74801963f1f6a149c8995c28056a2dad98c22bfb084e2aeaf5a8787b9
name=freeze file=on_disk_layout.py sha256=b4b2cf97e3391cfcd144823db514c75c700e88e5bc470634a6a5a98aec60e149
name=freeze file=replay_prefix.py sha256=a36cb9c1772eae7cfefba68b8912ad339b96a5383066970c2cb083cdf4537840
name=freeze file=root_ring.py sha256=6ac949d183997789419d1aeeb448d00bc341b26216ce7c22b6691879a7a2f89e
name=freeze file=spec-gaps.tsv sha256=98792f7bbe6d534a7e5ba840421db80db0fc4de4107238f69a43b147571986e4
name=freeze file=spec_gaps.py sha256=60f2f8b468cc68a2651716bd4ca3193380d0858935a846a5e1b4f28755685f5f
name=freeze file=synthetic_pool_for_tests.py sha256=ac97036edb1273218ad0eb67dc441d99c5a8a1f0f31b99bbf027d6c8dd0ff977
name=freeze file=system_configuration.py sha256=a07a280f3c96d8be8cedeaf9f92bddb8a5d7101b50e7e494357c2fdcdb16b394
name=freeze file=test_executor_on_synthetic_pool.py sha256=1704804cd38f824988f06cf7e4a88f6fb58da2011a08f10da7a8c8d4159b4de5
name=freeze file=version_table.py sha256=d407a8c1a53f58aa5a85a6af4eff215844b838d0a48871aaee4b567446a52ba4
name=freeze file=witness_predicates.py sha256=8499ebe274eb2919be0a637a63466da32d9a3a549b35ef131ba317d1d8925e3e
name=freeze_seed sha256_prefix16=ce39b484ec109c7a
```

#### Z-family re-export and V16

The main agent started three runs on the 2026-10-04 workspace snapshot. The first `e170-r2-o3-spec-executor-2026-10-04-s2b-today-export.out` exited 3: two successful steps, two failed, six skipped. Exporter S1 pinned fixed-script-through-E-then-unmount writes at 477, while current `crates/` had 481. Both E170 and E171 exporters' pinned values were updated to final crates values: 481 writes, 1,678,377,204 full states, two-state closed form 1,678,377,089. E171 recopied 43 known-bad images from 2026-10-04 case files per revision 2. The second run was rejected by the build memory wrapper's peak gate and ran no lines; it remains recorded. Third `s2b-today-export-r3.out` exited 0 on all ten steps and passed all snapshot checks, snapshot digest `d32a8b2da6ca653362537837457909d6194b69867f233ab55f6f4e04c64c9cec`.

```
name=device_line step=z1r line=E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 rejudge_mismatches=0 states_where_pool_checker_or_record_checker_is_red=0 v2_compared=true v2_differing_files=4 v2_differing_files_without_timing=0
name=z1_vs_z1r_summary same=16 different=4 one_side_only=0
name=done steps_exit_zero=10 steps_exit_nonzero=0 steps_skipped=0 snapshot_checks_failed=0 product=research/results/e170-r2-o3-spec-executor-2026-10-04-s2b-today-export-r3.out
```

Literal V16 hit: Z1 and Z1r byte comparison yields 16 identical, four differing files, each stream's timing-bearing `impl-verdicts.tsv` and digest-bearing `manifest.tsv`. The only difference is per-state `judge_crash_image` nanoseconds in the final column. Timing varies every run, so literal V16 always hits and cannot distinguish deterministic from nondeterministic export. Following the evidence-discipline rule for a criterion that cannot discriminate arms, the main agent did not invalidate Z; the exporter's timing-excluded criterion `v2_differing_files_without_timing=0` passes, and after removing the last column the 82 and 890 rows match respectively. Z is deterministic.

#### Stage 1: v2 and the two format arms

- v2 adds `witness_predicates.py` to compute W, τ and T using v1 functions for reading slots, scanning records and identifying a version's rotation writes. `judge_state` computes pre-verdict first, then W and T in `verdicts_after_the_witness_predicates`, preserving original verdict fields and adding `strict_before_witness=`, `provisional_before_witness=`, `witness_counter=`, `witness_branch=`, `witness_compared=`, `witness_true=`, `ack_rotation_tail=`, `tail_check=`, `failed_predicates=`. `executor_main.py` adds `name=witness_summary` and B1–B3. Gap table unchanged; W and T exposed no KB ambiguity. Synthetic tests add six W, five T, one version and three driver tests to v1's 53, total 68.
- Four executor-author interpretations: W's second branch compares instance generation first; “readable” excludes a record with last flag followed by another record of the same publish, which KB treats as corruption; “two slots per disk” means slot 0 and v1's selected slot 1 under ①②③; add `witness_compared=` to verdict lines. KB on 2026-10-04 specifies 487-byte journal header, but v2 remains 311 per registration 5.2; this hits S2 and revision 1 records it for the two 487 arms.
- On input 1 batches 1–2, all 576 states have W true zero times, T false zero times, `strict_in_to_not_in=0`; all 14 section-7 A anchors and B1–B3 hold.
- Identical format patch in both arms: journal offsets change to 487-byte header with seven new-root fields, fsid at 463, MAC at 471, end 487; replay reconstructs root from new-root fields. New gap G23: new-root unmount marker has nonzero bits other than bit 0, category missing statement, provisionally treat record as corrupt. V19 confirms identical patches across 15 shared files. Earlier `s1a-format-patch-comparison.out` is invalid: reused temporary file failed to be written starting with the second comparison, so all 15 compared empty patches; retained in archive.
- Stage 1 B: all 576 Z1 batches 1–2 states are `strict=in`, `provisional=in`, `digest=match`; W true and T false each zero. B4-487, B8–B14 and B16 all hold; B14 has only state 43, landing at (1, 3) by replay. Freeze v2-487 at lines 676–692 and v1-487 at lines 770–785, seed `0ae1d855d57a2ff6`.

```
name=verdict batch=1 stream=new_pool_file_creation states=53 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=53 provisional_in=53 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
name=verdict batch=1 stream=fixed_script_through_e_then_unmount states=397 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=397 provisional_in=397 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
name=verdict batch=2 stream=new_pool_file_creation states=7 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=7 provisional_in=7 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
name=verdict batch=2 stream=fixed_script_through_e_then_unmount states=119 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=119 provisional_in=119 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
```

#### Independence and invalidation clauses

- Stage 1 A: the v2 author followed dispatch and read the designer report containing current and HEAD `crates/` format constants and the reader's treatment of non-bit-0 unmount marker bits as corruption. Listing registration section headings with grep also exposed command comments in section 13. The main agent decided V7 and V18 do not trigger: W and T code does not use format constants, the observed HEAD value 311 is old and does not imply the 487 patch, and G23's provisional reading affects only provisional mode, while strict mode reports a gap. v2 and both arms remain valid.
- Stage 1 B: grep similarly exposed 19 command-comment lines in section 13, three lines of 12.16 and one stage-1-A report line paraphrasing format constants. The main agent found V7 applies to the v2 author and V18 to the format-patch author; this stage changed no existing arm files and those constants also appear in the KB journal-format text. Neither clause triggers and both freezes are accepted.
- Stage 2: the executor saw the x21 mutation-table original and v2 freeze exclusion list only after writing and trial-running W_ref and T_ref. The main agent did not invalidate it.
- No invalidation clause triggered except literal V16, which does not invalidate Z; S1, S3, S4, S6 do not trigger and S2 is recorded.

#### Mutations (second rerun)

- Unit layer, stage 1 A: v2 x21–x31 caught 11 / 11; x32–x36 caught 5 / 5 each on v2-487 and v1-487. x32 and x33 fail on load at offset-table assertions.
- Stage 2 ran x21 on rows 242 and 269, and x27 on row 270; measurements appear under Q2 and Q3.
- Stage 3 `e170-r2-o3-spec-executor-2026-10-04-s3-mutations.out`: caught 34, invalid 0, survived 0, timed out 0. v1 x1–x20 on copies of v2's frozen files caught 20 / 20 at unit level, x20 on load, showing v2 retains v1 discrimination; v2 x22–x26 and x28–x31 caught 9 / 9 with unit and batch layers; XC3–XC7, five comparator and reference-predicate mutations, 5 / 5. Final baseline restored and green; v1 and v2 freeze lines match before and after.

```
Restored; baseline still all green (both v1 and v2 directories' per-file sha256 match before the run; unit baseline in name=unit_baseline)
Counts (total): caught 34, invalid 0, survived 0, timed out 0, original text not hit exactly once 0; 34 / 34 table rows
```

- Batch layer: x22, x23, x28, x29 change input-1 stream-1 batch verdicts as registered: x22 changes 407 in → out and B1 false; x23 changes 24 in → out, all land (0, 0), B3 false; x28 changes 424 in → out, with 396 T true → false; x29 affects 332 states, including 22 stream-1 B2-type states. XC6 makes W_ref always false: besides Q2-b → 0, Q2-c has 78 disagreements, which would trigger S1. XC7 makes Q3-c disagreements 0 → 258.
- Stage 2 Q5 half: XC8 and XC9 (known-bad image recopy checks) caught 2 / 2 (`e170-r2-o3-spec-executor-2026-10-04-s2b-known-bad-recopy-mutations.out`); XC9 went red on `item_differences` changing 0 → 2, not on the cell "differences before and after recopy" written in preregistration 12.14, recorded as it happened (preregistration "Lead agent recognizes third pass of re-export preparation handoff" item 2).
- Not run: batch-layer x32, x35, x36 on Z1 stream-1 batch 1; stage-3 dispatch listed only x22–x26, x28–x31 and v1 x1–x20, before v2-487 was frozen. Also not run: v1 table XC1 and XC2.

#### Measurements not run after sufficient evidence

All five questions are decisive. Stage 4, v1-487 on Z compared to v1 on 311 exports and B11–B14 on v1-487, runs only if Q4 is B or every Q5 B difference is class (ii). Q5-g, per-state Z vs 311 comparison, is auxiliary and not run after sufficient evidence. Full replay of nine `replay.sh` rows is heavy tier and reserved for submission.

### Path and conclusion register

| # | Path | Calculation | Source | Shared inputs |
|---|---|---|---|---|
| 1 | Executor in / out | From KB text only: select the system configuration, root selection, ring scan, select prefix, compute A(c), accept if L ∈ A(c). 972 sampled states and 76 bad rows; enumerated points, not exhaustive. | `scripts/experiment-apparatus/e170_o3_spec_executor/executor_core.py:95` `judge_state` | Neutral exported bytes recorded by implementation writer; KB decisions, invariants, layout (byte tables mostly written from implementation); interpretations 2, 3 for projection and A(c); G01–G20 provisional readings. |
| 2 | Implementation verdict | Run `crates/` recovery, pool-level checker and record checker on the same crash image; both checks must accept. | `crates/singlefs-checker-tier/src/crash.rs:1353` `judge_crash_image` | Same exported bytes from implementation `CrashImage`; format constants and core reader/writer. |
| 3 | Per-state comparison | Compare path 1 verdict and path 2 `impl-verdicts.tsv` by state ID, with one-state-flip positive control. | `scripts/experiment-apparatus/e170_o3_spec_executor/comparator_q2_implementation_verdicts.py:61` `disagreements` | State ID from `states.tsv`; interpretation 6, both checks accept. |
| 4 | Implementation image digest | Read `CrashImage` directly, feed nonzero sectors to SHA-256. | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:804` `crash_image_digest` | Section 5.3 digest measurement basis. |
| 5 | Executor image digest | Build image independently from neutral format and persistence vector, then hash, overlaying patches afterward. | `scripts/experiment-apparatus/e170_o3_spec_executor/crash_image.py:91` `digest` | Section 5.3 digest measurement basis; torn bytes in `torn-payload.bin`. |
| 6 | Executor decides in / not in (first rerun, frozen new directory) | Same as path 1, modified in five places according to the 2026-10-03 KB (magic, field offsets, offset for reading slot 1, two exits per tier, gap table); 576 states across batch 1 and batch 2, plus 78 bad-state rows | `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/executor_core.py:119` `judge_state` | Same as path 1; plus 2026-10-03 KB (magic rows of the three field tables in `layout/01-first-txn.md` (system configuration `SFSB`, root record `SFSR`, journal record `SFSJ`), three passages "**Offsets in the system configuration field table**", "**Inside position entries (14 bytes)**", and "**Offsets in the record-header field table**", the "**Milestones**" passage in "7. Publishing (root records and root slots)", the "**Location of system configuration slot 0 and slot 1**" passage, and D22 (composition of unit atomicity) settled item 16) |
| 7 | Q3 comparison (executor verdicts against expectations) | Compare strict execution mode verdicts for each bad-state row against the expectation table row by row, where the expectation table is written by the exporter according to modification rules; includes positive control flipping one row of expectations | `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/comparator_q3_bad_states.py:173` `compare_row` | Exporter-written expectations (`bad-expected.tsv`, produced by the same code that modifies patches); preregistration 5.5 P table, N table, P10 and interpretation 3 |
| 8 | v2 in / out, second rerun W and T | Path 6 gives pre-verdict; where L(c) exists, compute W from witness and last record or record at witness, T from witness ≥ ack(c)'s rotation-written tail. Pre-verdict in becomes out if W true or T false. Inputs 1–4, bad images, three s4 rows and baseline are enumerated points. | `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/witness_predicates.py:201` `evaluate_witness_predicates` | Path 6; interpretations 2–4 definitions of W, τ, T; KB configuration fields (journal tail offset 469), 311-byte journal header; L(c), ack(c) from path 6. |
| 9 | Reference W_ref, T_ref | Written separately in stage 2 without opening v2 W/T code: read slots and journal and obtain each publish's rotation writes from recorded stream, using KB statements transcribed in section 2. Same states as path 8. | `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/witness_reference.py:256` `witness_reference`, `:376` `tail_check_reference` | v1 `crash_image`, `crash_dump`, `crc32c_castagnoli`; same exported bytes; interpretations 2, 3; L(c), ack(c) from v2 verdicts, i.e. path 6; layout slot and record-header tables. |
| 10 | v2-487 in / out | Path 8 with 487-byte header and seven new-root fields; replay reconstructs root from those fields. Z family, enumerated points. | `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/executor_core.py:132` `judge_state` | Path 8; KB journal format and expressions transcribed in 12.8 for offsets; bytes exported from 2026-10-04 `crates/`. |
| 11 | Format constants, Q5-d implementation side | Read eight constants in snapshot `crates/singlefs-format/src/lib.rs` and 25 write-order statements in `JournalRecord::to_bytes` in `crates/singlefs-core/src/journal.rs`; widths come from writes; compare to path 10's `on_disk_layout.py`. | `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/comparator_q5_format_constants.py:70` `library_constants`, `:83` `write_order_in_the_implementation` | Both snapshot source files; four new-root pointer widths inferred from implementation's segment-width assertion, equal to `NODE_POINTER_BYTES`. |

Digest comparison is anchored in source at `executor_core.judge_state` `digest_matches`; all 972 samples and 76 bad rows agree. Verdict comparisons are linked by comparator and two mutation-check assertions `q2_rejudged_batch1`, not a direct assertion connecting both paths. Implementation-recorded exported bytes and KB byte tables written from implementation can cause paths 1 and 2 to fail together, accepting the same incorrectly written bytes.

In the second rerun `comparator_witness_classes.py` compares path 8 W/T with path 9 W_ref/T_ref per state, reporting zero Q2-c and Q3-c disagreements without directly asserting the two paths together. Shared L(c), ack(c), the three v1 modules or exported bytes can make both wrong together. `comparator_q5_format_constants.py` compares paths 10 and 11 and reports `name=q5d_verdict`, zero differences in 36 items, again only counts; if the KB format text was written from implementation, both can be wrong together.

### Replay

The checked-in checker-tier exporter is started via `cargo run`:

```
bash scripts/hosts/run-with-memory-cap.sh 12G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- dump --output-directory <新目录甲>
bash scripts/hosts/run-with-memory-cap.sh 12G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- dump --output-directory <新目录乙> --compare-with <新目录甲>
```

`scripts/experiment-apparatus/replay.sh` `driver_e170_r2_dump_compare` executes both in temporary directories against current `crates/`, comparing only the second run's `E7RESULT` lines exactly. Existing output directories are rejected, exit 2; `scripts/experiment-apparatus/target/e170-dumps/` is not overwritten.

Stage 3 requires the ten stage-1 files in the export directory; it rejects existing five bad-state files, exit 2:

```
bash scripts/hosts/run-with-memory-cap.sh 12G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <导出目录> --seed-hex 8546b1258e427c34
cd scripts/experiment-apparatus/e170_o3_spec_executor
PYTHONDONTWRITEBYTECODE=1 python3 comparator_q3_bad_states.py --dumps <导出目录> --frozen-verdicts <仓根>/e170-o3-spec-executor-2026-10-02-s2-executor.out --output <新文件>
PYTHONDONTWRITEBYTECODE=1 python3 comparator_q2_implementation_verdicts.py --dumps <导出目录> --frozen-verdicts <同上> --output <新文件>
PYTHONDONTWRITEBYTECODE=1 python3 mutation_driver.py --table <仓根>/scripts/mutation/tables/e170_o3_spec_executor.tsv --dumps <导出目录> --frozen-verdicts <同上> --scratch <草稿目录> --output <新文件>
PYTHONDONTWRITEBYTECODE=1 python3 executor_main.py --dumps <导出目录> --batches 1,2,3 --workers 8 --output <新文件>
```

The first run's four stage-3 replay drivers `driver_e170_s3_bad_states_export`, `driver_e170_s3_bad_states`, `driver_e170_s3_q2_comparator`, `driver_e170_s3_mutations`, sharing a temporary export and exact comparison, were removed from `replay.sh`; the reason is under the second-rerun replay registration and history is in git. Four stage-2 and Q4 measurements include per-state timings and were not registered; verdict lines match after removing `time_ns` and `worker`. Stage-3 Python artifacts were rerun on a draft export copy and were byte-identical.

First rerun stage 1, Python rather than compiled code; each command used memory and thread caps:

```
cd scripts/experiment-apparatus/e170_second_run_o3_spec_executor
E170_TEST_SCRATCH=<草稿目录>/testscratch python3 -m unittest -v test_executor_on_synthetic_pool
python3 executor_main.py --dumps <仓根>/scripts/experiment-apparatus/target/e170-dumps --batches 1,2 --workers 6 --output <新文件>
python3 mutation_driver.py . <仓根>/scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv <草稿目录>
```

None of the three stage-1 artifacts are registered in `replay.sh`: the executor output contains per-state times and process IDs and relies on the untracked original export, which the current exporter cannot reproduce; unit tests and unit-level mutations contain test timings and random temporary directory names. A separate timing-free stage-1 replay artifact is for the main agent to decide.

First rerun stages 2 and 3, each command under memory and thread caps; `<导出目录>` is `scripts/experiment-apparatus/target/e170-second-run-dumps`. Stage 3 writes into that same directory and rejects preexisting five `bad-*` files with exit 2:

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 6 cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- dump --output-directory <导出目录甲>
bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 6 cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- dump --output-directory <导出目录> --compare-with <导出目录甲>
bash scripts/hosts/run-with-memory-cap.sh 6G bash scripts/hosts/capped.sh 6 cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states-second-run --dump-directory <导出目录> --seed-hex 85441ae491419411
cd scripts/experiment-apparatus/e170_second_run_o3_spec_executor
python3 executor_main.py --dumps <导出目录> --batches 1,2 --workers 6 --output <新文件>
python3 comparator_q2_implementation_verdicts.py --dumps <导出目录> --frozen-verdicts <上一条的产物> --freeze-reference <仓根>/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out --output <新文件>
python3 comparator_q3_bad_states.py --dumps <导出目录> --freeze-reference <仓根>/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out --output <新文件>
python3 mutation_driver_with_batch_and_bad_state_checks.py --table <仓根>/scripts/mutation/tables/e170_second_run_o3_spec_executor.tsv --dumps <导出目录> --frozen-verdicts <第二条 Python 命令的执行器产物> --scratch <草稿目录> --output <新文件>
```

First-rerun replay drivers were updated at the end of the second rerun: `driver_e170_r1_s2_q2_comparator` was deleted, `driver_e170_r1_s3_bad_states_export`, `driver_e170_r1_s3_bad_states`, and `driver_e170_r1_s3_mutations` were redirected to drivers and artifacts of the second rerun, and `driver_e170` was redirected to new artifacts of the two re-export passes (documented row by row in the "Second-rerun replay registration" paragraph). The registration version prior to replacement (nine rows, `driver_e170` pointing to first-rerun stage 2 re-export artifact `e170-r1-o3-spec-executor-2026-10-03-s2-export.out`, with old artifact `e170-o3-spec-executor-2026-10-02-s1-export.out` archived) remains in git, and those artifacts are retained in `research/results/`. The stage 2 executor artifact `…-s2-executor.out` contains per-state elapsed times and process IDs and does not enter replay, remaining in `research/results/` as fixed input for comparators and mutation drivers.

**Second-rerun replay registration** (installed 2026-10-05): the E170 (first O3 specification executor) section in `scripts/experiment-apparatus/replay.sh` was replaced with nine registered replay rows for the second rerun (`grep -n 'driver_e170' scripts/experiment-apparatus/replay.sh` matches lines 246–254, corresponding to the nine rows at the end of the experiment inventory table), each running one of the nine replay drivers. All nine use strict execution mode, and all nine are byte-for-byte identical.

| Line | Driver | Artifact compared | Origin |
|---|---|---|---|
| 254 | `driver_e170_r2_dump_compare` | `e170-r2-o3-spec-executor-2026-10-05-s2b-dump-compare.out` | Redirected from `driver_e170`: two dumps, compare second to first, keep second's `E7RESULT`. |
| 255 | `driver_e170_r2_bad_states_second_run_export` | `e170-r2-o3-spec-executor-2026-10-05-s2b-bad-states-second-run-export.out` | Redirected from first-rerun stage-3 exporter, seed `85441ae491419411`. |
| 256 | `driver_e170_r2_v2_487_unit_tests` | `e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-tests-2026-10-05.out` | New: 74 v2-487 unit tests. |
| 257 | `driver_e170_r2_v1_487_unit_tests` | `e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-tests-2026-10-05.out` | New: 59 v1-487 unit tests. |
| 258 | `driver_e170_r2_v2_487_unit_level_mutations` | `e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-level-mutations-2026-10-05.out` | Redirected from first-rerun stage-3 mutations: x32–x36, v2-487 unit layer. |
| 259 | `driver_e170_r2_v1_487_unit_level_mutations` | `e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-level-mutations-2026-10-05.out` | New: x32–x36, v1-487 unit layer. |
| 260 | `driver_e170_r2_v2_487_z1` | `e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z1-2026-10-05.out` | New: v2-487 judges freshly exported Z1 batches 1–3, one process. |
| 261 | `driver_e170_r2_v2_487_z3_images` | `e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z3-images-2026-10-05.out` | Redirected from first-rerun stage-3 bad states: v2-487 judges fresh Z3. |
| 262 | `driver_e170_r2_v2_487_z4_images` | `e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z4-images-2026-10-05.out` | New: v2-487 judges fresh Z4. |

Five 2026-10-04 registration lines, 255–258 and 260, and their exclusive drivers were removed: original stage-3 `driver_e170_s3_bad_states_export`, `driver_e170_s3_bad_states`, `driver_e170_s3_q2_comparator`, `driver_e170_s3_mutations`, and first-rerun `driver_e170_r1_s2_q2_comparator`. The first four used first-generation executor `e170_o3_spec_executor/`, original seed `8546b1258e427c34`, bad-state table, comparator and mutations; the last used first-rerun Q2 comparator. The main agent determined these have no equivalent under the 2026-10-05 format, removed them from replay, kept git history and artifacts in `research/results/`.

None of the 66 original artifacts from the second rerun can serve directly as retained baseline artifacts for replay rows: they do not follow the E7RESULT protocol (not a single line begins with `E7RESULT`, nor is there a closing row such as `E7RESULT name=done emitted=N`); gate 2 of `replay.sh` judges them as "cannot run: missing concluding row name=done"; furthermore, the artifacts contain items that vary across runs (the stderr line output by memory wrappers, wall-clock timing such as `Ran 74 tests in 0.242s`, per-state `time_ns=` and `total_ns=`, process ID `worker=`, randomized temporary directory names and absolute paths), and `strip_timing` cannot strip `time_ns`, `total_ns`, or `worker`. Broken down by input: 11 copies are pure Python with all inputs in git; 21 copies depend on freshly exported Z family or files in `crates/`; 34 copies depend on frozen 311-format exports (not tracked in git, and the current exporter cannot produce 311 format either, so no replay rows were added per preregistration 5.5). Therefore, replay rows produce results in the form of result-line artifacts: newly added `run_replay_result_lines.py` in the v2-487 directory (prefixed with `run_`, not frozen; `unit-tests` records unit test outcomes one by one without elapsed time, `wrap` rewrites raw output from executor and image-judging scripts into `E7RESULT` rows, deleting fields ending in `_ns` and `worker=`, and replacing absolute paths with their final segment); `mutation_driver_unit_level.py` in the v2 directory only modified output rows, leaving which mutations were chosen, how replacements were made, and how redness was judged completely untouched; frozen files were not modified at all. The two unit-test tier mutation rows filtered out XC8 and XC9 when executing tables (original source is in `run_known_bad_recopy_check.py`, not governed by unit tests; preregistration "Lead agent recognizes stage 2 Q5 handoff" item 3).

Verification: four pure-Python drivers ran twice, both runs and checked-in artifacts were `cmp` identical and passed gate 2. Against original 2026-10-04 artifacts, all 74 and 59 test outcomes and frozen hashes match, as do 5 / 5 mutation outcomes. The main agent ran each of five export-dependent lines twice, six `cargo run` calls per run: all five were byte-identical across runs and passed gate 2. Both export outputs also match Z-family third re-export byte for byte: stripping `name=device_line step=z1r line=` and comparing with `e170-r2-o3-spec-executor-2026-10-05-s2b-dump-compare.out` exits 0; step z3 likewise matches bad-state export. Z1's six verdict lines match the 2026-10-04 Z1 artifact after removing `E7RESULT `, and its 972 sorted state lines match after removing `_ns` and `worker=`. Checked-in verdict and completion lines follow; v1-487 mutation counts are identical to v2-487 and not repeated.

```
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 rejudge_mismatches=0 states_where_pool_checker_or_record_checker_is_red=0 v2_compared=true v2_differing_files=4 v2_differing_files_without_timing=0
E7RESULT name=verdict bad_s1_failed_checks=0 bad_rows=78 impl_panics=0
E7RESULT name=unit_tests_summary arm=v2-487 ran=74 recorded=74 ok=74 fail=0 error=0 skipped=0 expected_failure=0 unexpected_success=0 successful=true
E7RESULT name=unit_tests_summary arm=v1-487 ran=59 recorded=59 ok=59 fail=0 error=0 skipped=0 expected_failure=0 unexpected_success=0 successful=true
E7RESULT name=mutation_counts caught=5 invalid=0 not_red=0 timeout=0 total=5 table_rows=5
E7RESULT name=verdict batch=1 stream=new_pool_file_creation states=53 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=53 provisional_in=53 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
E7RESULT name=verdict batch=1 stream=fixed_script_through_e_then_unmount states=397 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=397 provisional_in=397 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
E7RESULT name=verdict batch=2 stream=new_pool_file_creation states=7 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=7 provisional_in=7 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
E7RESULT name=verdict batch=2 stream=fixed_script_through_e_then_unmount states=119 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=119 provisional_in=119 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
E7RESULT name=verdict batch=3 stream=new_pool_file_creation states=22 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=22 provisional_in=22 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
E7RESULT name=verdict batch=3 stream=fixed_script_through_e_then_unmount states=374 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=374 provisional_in=374 provisional_not_in=0 judgement_failures=0 digest_mismatches=0
E7RESULT name=done emitted=117
E7RESULT name=done emitted=158
```

The nine `replay.sh` rows were not run together; heavy tier, reserved for submission.

### Unit tests and mutations

The exporter originally had 13 unit tests, covering the three-batch sampling formula, persistence-vector reduction (torn = 1), exclusion of zero sectors from digests and absolute values of two digests, closed forms for both streams' segment lengths and write counts, and byte-identical `CrashImage` versus neutral-format reconstruction on a small pool. They compiled via `cargo test --no-run` but did not run at checker tier. XE1–XE3 were appended to `crates/mutations.tsv`, each naming a failing unit test; three counts await checker-tier-crates-mutation-replay at submission. Registration section 9 says stage 1 cannot catch XE3 in the stop-clause sense because both digests use the same function; its named test directly pins whether the digest skips zero sectors.

Stage 3 added three exporter tests: two reference splitmix64 outputs from state 0, ⌈K/2⌉ selection and `bad-states` CLI. Total 16, compiled but not run; no `crates/mutations.tsv` mutation was added for that mode. The executor has 36 tests, all green both before each mutation and after restoration; see stage-3 executor mutations.

First-rerun stage 3 added two more exporter tests, for the new mode CLI and comparison of both tables on stream 1 including P10 base and patch. Total 18, compiled but not run. XE4 was appended naming the latter test. First rerun executor: 53 tests, 22 mutations; see its sections.

Second rerun synthetic-pool tests all passed: v2 68, v2-487 74, v1-487 59. Eight hand-built true/false reference-predicate cases all passed, as did v1's 53 frozen tests against v2; an auxiliary run also substituted v1's synthetic-pool helper. Mutations: unit-level x21–x31, 11; x32–x36, five on each format arm; stage 3, 34; XC8 and XC9, two: all caught, none invalid or surviving. Tables are `scripts/mutation/tables/e170_o3_spec_executor_with_witness_predicates.tsv` (x21–x31 plus XC3–XC7, 16) and `scripts/mutation/tables/e170_o3_spec_executor_with_witness_predicates_journal_header_487.tsv` (x32–x36 plus XC8/XC9, seven). Fixed-script-through-E-and-unmount pinned values in both E170 and E171 exporters and their unit tests were updated to the 2026-10-04 `crates/` values.

### What this cannot answer

- First-rerun stage-1 executor was modified and run by the same operator, so independence is weaker than the first run. Before modifying it the operator read preregistration sections 3, 4 and 13 including `crates/` function names and line numbers and original measurements, three header comments in `crates/mutations.tsv`, and one default line in `scripts/experiment-apparatus/replay.sh`. No `crates/` implementation code, `research/results/` verdicts, `impl-verdicts.tsv` or `bad-*` files were read. The main agent determined literal V7 did not trigger after the handoff and accepted the executor.
- Original Q1 undecidability describes the KB when the executor was written. After five changes for 2026-10-03 KB, strict mode decided original exported batches 1 and 2, 450 + 126 sampled states, without hitting any of the 19 remaining G04–G22 gaps. This does not prove other states, full enumeration or other histories cannot hit them; not hitting them also does not establish that their provisional readings are correct, despite not drawing those readings from `crates/`.
- Original Q2 consistency means both sides accept all 972 sampled states, with no executor-out or implementation-red state. After F1 it is not evidence that the executor judges correctly; after F6 it does not cover selection of a root other than the maximum on-disk txg. Samples are not exhaustive: stream 2 has 1,662,648,564 total states. Batch-1 torn scanning found no new states; every torn-write state came from the grid.
- Q3 discrimination was measured only on section 5.5's listed corruption methods and three blind types. P8's four in verdicts follow interpretation 3's lower bound; they do not distinguish lack of executor discrimination from interpretation 3 already allowing P8.
- Implementation-side discrimination is only counted in Q3-f, not judged by E170. Its patch is a persisted system-configuration-slot write, so the record checker's second criterion can treat it as a later-write explanation, as in n5; the executor instead excludes the patch as a write.
- Original Q4 timings are two one-off observations with other load. The 60-second budget is registration section 6's budget, not a measured quick-tier gate budget.
- First-run executor mutation x10 had no sensitive sample and survived; x2 and x8 were caught only by unit tests. All 22 first-rerun mutations x1–x20, XC1 and XC2 were caught by tests plus checks, but 11 (x2, x10, x12–x19, XC2) have no sensitive batch or bad state and were caught only by synthetic tests; do not claim all 22 were caught by verdicts. XE4's three formal counts are absent until submission replay; only S5b going red on a draft mutated exporter was observed.
- First-rerun Q3's positive result covers 26 P instances, 40 blind rows and two correct P10 rows: strict in/out and landing agree with expectations for section 5.5's P1–P5, P9, P10 and blind P2/P3/P5 corruptions. P8's expectation was revised before artifacts under interpretation 3; executor and expectation share that reading, so P8 cannot establish whether the executor or reading 3 is right. The exporter, which reads `crates/`, generates both patches and expectations from the same corruption code; a shared error is invisible to row comparison. Other corruption methods, fields and positions were not measured.
- First-rerun Q2's implementation is `crates/` at re-export, with git head and staged/worktree diff hashes in provenance. By stage-3 bad-state generation the staged diff had changed. Batch implementation verdicts come from stage 2 `impl-verdicts.tsv`, bad-state implementation verdicts from stage 3 `bad-impl-verdicts.tsv`.
- Original Q3-f 28 became 21 on 2026-10-03 with the same original bad-state table, seed and comparator; the old artifact describes the implementation at the original run. First-rerun Q3-f is 15 / 68 for P, P10 and blind on a different table and seed, so the denominator differs from 28 and 21.
- Second-rerun Q1–Q3 use only frozen 311-format exports, inputs 1–4, three s4 rows and s4 baseline; v1 and v2 read 311-byte headers. The 2026-10-04 format is covered by Q5: v2-487 on Z is isomorphic to v2 on 311 at interpretation-14 family/control cells, not per state; Q5-g was not run after sufficient evidence. Stream positions are normalized to `#`; literal comparison makes 41 more cells incomparable but Q5 remains A. Known-bad indices 41 and 42 have no input-4 counterpart and are only reported as auxiliary v2-487 verdicts.
- W/T discrimination was measured only on registered state classes: Q2 K from E171's two faulty implementations at rows 242 and 269, Q3 K270 from row 270. KB-based W_ref does not filter by fsid, omits the member where both slots of a disk fail self-validation, and chooses maximum when several exist. Its agreement with v2 on these inputs only shows no sampled state separated those readings, not that the readings are equivalent; S3 did not trigger. W_ref and v2 share v1 L(c), ack(c), `crash_image`, `crash_dump`, `crc32c_castagnoli` and exported bytes, which can make both wrong together.
- Real streams cannot distinguish τ selection: x30, τ as last-record counter, and x31, τ as minimum rotation-write tail, change no field on input 1 or three s4 rows. All 176 reference publish rows have rotation tail equal to last-record counter and equal across both disks. Only synthetic tests pin the distinction; Q3's zero legal-history false reds are insensitive to τ selection.
- Section 9's claim that x24–x26, x30, x31 have sensitive points only in synthetic tests and do not change real-stream outputs needs qualification. On registered input 1, x24, x25, x30, x31 change no output field; x26 changes no verdict but changes `witness_counter` in 44 states. On auxiliary s4 rows x24 changes ten row-269 verdicts out → in, exactly the `both_unreadable` ten; x26 changes six verdicts each on rows 242 and 269 out → in, and 31 on row 270 in → out. The claim holds for x25, x30, x31 across all four exports, and only verdicts for x24/x26 on input 1.
- x23's original text `if witness_counter == 0:` appears twice in the v2 directory, in `witness_predicates.py` and stage-2 `witness_reference.py`. Stage-3 driver counts only frozen files and finds one; the stage-1 unit-level driver counting a full-directory copy would find two and exit 3.
- Z uses the workspace snapshot at re-export, digest `d32a8b2da6ca653362537837457909d6194b69867f233ab55f6f4e04c64c9cec`; replay lines freshly export `crates/` at replay time. Two export replay outputs matching the third Z export byte for byte only show that the registration-time `crates/` export for these two streams was unchanged.
- The second rerun's three independence edge cases, stage 1 A, stage 1 B and stage 2, were judged not to trigger V7/V18 by main-agent determination, not machine verification.

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D13 verification route, settled item 6 | Supports | Unaffected 2026-10-15: full artifact lines changed to pointers; numbers, conclusions and artifacts unchanged. |
| D13 (verification route) settled item 4 | Preparation | 2026-10-15 unaffected: verbatim-copied artifact lines replaced with pointers to artifacts; numbers, conclusions, and artifacts unchanged |
| D22 (composition of unit atomicity) settled item 16 | Preparation | 2026-10-15 unaffected: verbatim-copied artifact lines replaced with pointers to artifacts; numbers, conclusions, and artifacts unchanged |
| D13 (verification route) settled item 1 | Preparation | 2026-10-15 unaffected: verbatim-copied artifact lines replaced with pointers to artifacts; numbers, conclusions, and artifacts unchanged |
| D23 (journal role and format) settled item 14 | No effect | 2026-10-15 unaffected: verbatim-copied artifact lines replaced with pointers to artifacts; numbers, conclusions, and artifacts unchanged |
| D16 (publish semantics) settled item 7 | No effect | 2026-10-15 unaffected: verbatim-copied artifact lines replaced with pointers to artifacts; numbers, conclusions, and artifacts unchanged |

## Historical Versions

### 2026-10-05: second rerun completed (Q1–Q5 decisive)

Switched to a new question list `e170-r2-questions.md`: two new executor directories were created (v2 adding predicates W and T; v2-487 further following the 487-byte journal record header), leaving the first-rerun version unchanged. Q1 A (0 items out of scope, 8 in-scope items precisely matching the eight P8 cells), Q2 A (E171 (three-oracle collusion test) lines 242 and 269 K 15 and 63 states all judged "not in"), Q3 A (line 270 K270 68 states all judged "not in", 0 false reds on legal history), Q5 A (0 differences across comparable cells of seven families), Q4 A (E171 first rerun uses v2-487); all 34 stage-3 mutations were caught. Page status was updated from "first rerun completed (three stages)" to "second rerun completed (Q1–Q5 reached sufficient evidence)"; added a sentence on the second rerun to the current status paragraph; added a "Second rerun" section; added paths 8–11 to "Registration of paths and conclusions"; the "Replay" section documented replacing the nine E170 (first O3 specification executor) rows in `replay.sh` with the nine rows of the second rerun (deleting four rows of the first-generation executor and one row of the first-rerun Q2 comparator, redirecting four rows, adding five rows) and the rationale for why the 66 original artifacts could not be replayed directly; added the second-rerun items to "Unit tests and mutations" and "Limitations"; re-reviewed the four rows in "Affected decisions", which remain unaffected with unchanged relationships.

### 2026-10-03: first-rerun stages 2 and 3 completed (Q2, Q3)

Stage 2 re-exported twice; both batch-1 sets match the original batch, 16 files byte-identical. Frozen executor judged batches 1 and 2; Q2 consistent 0 / 450 and 0 / 126, Q2-pc 1. Stage 3 added `bad-states-second-run` with P8 expected in and P10 per stream, passing S5 and S5b; Q3 discriminating, P 0 / 26, blind 0 / 40, P10 0 / 2 wrong. All 22 executor mutations caught; XE4 appended. Status updated to all three first-rerun stages completed; Q2/Q3 split by run, limitations expanded, four decisions reviewed. D13 item 6 marked changed because O3's first-version status sentence in scope needed updating by the main agent in the same commit. Replay lines 240–242 redirected to regenerated artifacts and four first-rerun drivers added.

### 2026-10-03: first-rerun stage 1 completed (Q1)

The executor was unfrozen once, modified in five places according to the KB of that day (magic, field offsets, offset for reading slot 1, two exits per tier, gap table), copied to a new directory `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`, and frozen; on the original export, strict execution mode encountered 0 states hitting "specification gap" across 450 states of batch 1 and 126 states of batch 2, and Q1 was decidable. Page status changed from "completed (three stages, four rows of question list reached sufficient evidence)" to "first rerun partially completed"; the Q1 row of the status table was split into first run and first rerun; the first limitation in "Limitations" (frozen executor did not rerun strict execution mode) was replaced with this run's readings and its scope; added D22 (composition of unit atomicity) settled item 16 row to the "Affected decisions" table, and re-reviewed three D13 (verification route) rows. Q2 and Q3 belonged to stages 2 and 3 and had no numbers yet.

### 2026-10-02: stages 2 and 3 completed

KB-only executor written and frozen: all 450 strict batch-1 states hit gaps, Q1 undecidable; all 972 provisional states in; batch 1 six-process 0.170246 s. Stage 3: Q2-a 0 / 450, Q2-pc 1; exporter gained `bad-states`, 76 rows, Q3-a 4 / 26 all P8, Q3-b 0 / 40; 11 of 12 executor mutations caught, x10 survived; Q4 eight-process 0.195638 s. Status changed from preregistration written and stage 1 running to completed.

### 2026-10-02: page created, stage 1 completed

Exporter written, two streams exported twice: literal V2 had four timing-bearing files differ, V2b all identical. All 36 S1 checks and 972 S4 states passed; implementation accepted all 972 samples.
