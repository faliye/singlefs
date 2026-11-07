## E203 Step cost of the scans before admission — not run (first-segment apparatus, unit tests and mutation table written; the artifact has not been run once)

[中文](203-准入之前扫类的步代价.md) · **English** · [日本語](203-受理前スキャン系のステップコスト.ja.md)

Pre-registration `research/prompts/e203-preregistration.md` (706 lines). Fork sheet `research/prompts/m4-admission-delta-forks.md`, three rows: B1 today, B8 item-by-item by delta, B4 change the scope. It measures the steps and wall clock of the scan-type steps each publish runs before the two acceptances, across three orders of magnitude of unit area and record counts, including the shape at 16 TiB per disk after 10^6 unlinks (that cell is extrapolated only, no history is built).

**This page records only that the pre-registration and the first-segment apparatus are written; the artifact has not been run once.** The anchor check, the model's unit tests and mutation, and the counter-insertion script's self-test have been run once by the main agent (the self-test turned one case red); the changes made after that have not been rerun.

### Current state

- First-segment apparatus written (2026-11-05):
  - Measuring path: `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` (inserts only counters or only timers into a copy of the main-workspace implementation, no judgement changed; `--build counts|timing`, `--check-anchors`, `--selftest`, `--mutation-check`), history cases `scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs` (one case per each of 19 sample points), entry script `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh`.
  - Derived path: counting model `scripts/experiment-apparatus/e203_admission_scan_cost.rs` (bin name `e203-admission-scan-cost` in `Cargo.toml`), 28 unit tests (the 17 of the first writing pin the registration's section-7 anchors, the reading wiring, the extrapolation, the summaries and the timing verdicts; the 11 added at the close pin stop clauses 3 and 4, the ③2 reconciliation, void clause 8, the extrapolation at the user-scale and the end-to-end readings of failure clauses F1, F2, F3, F5 and F6), mutation table `scripts/mutation/tables/e203_admission_scan_cost.tsv` with 30 entries, and the mutation table of the counter-insertion script `scripts/experiment-apparatus/e203_admission_scan_cost_counters.mutations.tsv` with 6 entries.
- Continuation of the first segment (2026-11-05): the counting sites and the timing sites of all 36 rows of the registered list are written (the 12 rows not landed and the 4 half-landed rows of the first writing are filled in; two cases in the counter-insertion script's self-test turn red when a row lacks either site); all B8 input readings are read from the artifact (a reading that is missing is recorded as a gap, not as 0); the model's report is wired to Q1 to Q13, the positive and negative controls, the stop and void clauses and the failure clauses F1 to F6, and the B1 timing positive control is written; the timing guards are tightened from function level to statement level (registration section 12, revision 1). None of the three fork rows is decidable yet: the artifact has not been run once.
- Close of the first segment (2026-11-05): the self-test case "the timing build carries the statement-level guards" turned red because the check counted the timing guards only in `transaction.rs` and `allocator.rs` while the timing build changes 7 files (derived from the source, to be confirmed when the self-test is rerun); the check now compares each file against the count its edits expect, and mutation c6 (one timing guard not inserted) was added; names were changed to full words; the model gained B8's and B4's step counts at the user-scale (the `user_scale` rows of Q6, Q8 and Q9, extrapolated the same way as Q4), the ③2 reconciliation on five rows, void clause 8 ("not measurable") and unit tests for stop clauses 3 and 4; failure clauses F1, F2, F3, F5 and F6 now have end-to-end cases, and F6 reads "not judged" when no cell was checked. The unit tests, mutation and self-test have not been rerun.
- The copy of the implementation now carries every file a build needs (in the first segment's compile smoke the copy lacked `prune-server/`, `.cargo/` and `.claude/heavy-thread-default`).
- The six items the registration left for the main agent to confirm were all confirmed; done as registered, no reading changed.
- Status: not run.

### Fork table

| Candidate | Enough-to-decide condition | Now |
|---|---|---|
| B1 today | Q1–Q3 bands on the U axis and the three record axes u, D, F; section-7 checks ③1 and ③2 all pass; Q4 has a verdict in all four cells; Q5 checks out | all artifacts missing |
| B8 item by item by delta | Q6 band and Q7 numbers on all four axes; B8 positive and negative controls pass | all artifacts missing |
| B4 change the scope | Q8 band and Q9 numbers on all four axes; B4 positive and negative controls pass | all artifacts missing |

### Rerun

```bash
# Measuring path: check anchors first (read only), then run by segment; each command is submitted by the main agent via task-server
python3 -I scripts/experiment-apparatus/e203_admission_scan_cost_counters.py --check-anchors .
bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build counts --work <new directory> --segment first
bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build timing --work <new directory> --segment first
# Derived path: run the model on the artifacts
scripts/experiment-apparatus/target/release/e203-admission-scan-cost report <counts artifacts>…
```

### Measurement basis

- No raw output kept: the experiment has not been run.
- "Before acceptance" is cut into three segments reported separately: caller segment (entry to the call of prepare), before the placement judgement, between the two judgements; after acceptance is reported as a count only. A scan-type step is one step per entry of the innermost loop body; a whole copy counts the sum of the element counts of every collection in the copied value.
- Timing uses a release build, a single thread, and drops the `lookup-work-counters` feature; each point is measured five rounds in the same mount and after a remount.
- Positive control: each arm has a switch that turns off the step it saves and makes it grow once with the unit area; negative control: the steps assembling the units for the landing judgement depend only on this publish's roles.
- The registration writes "not read is not 0" as a void clause: a missing counter row, state row or completion marker voids that round, which is rerun. An overwrite cell that still lacks five rounds after the top-up is dropped whole and named "not measurable" in the artifact, never counted as 0.

### Criteria and failure clauses

- Criteria: growth order α in three bands (α ≥ 0.5 grows with the axis, α ≤ 0.25 does not, in between no verdict); at the user-scale shape the scan wall clock of one overwrite before acceptance is "≥ 1 ms" if all five rounds are ≥ 2 ms, "< 1 ms" if all are ≤ 0.5 ms, otherwise unstable.
- Six failure clauses F1–F6, each followed by what observation triggers it; original text in section 10 of the registration.
- Eight void clauses and five stop clauses in section 11 of the registration.

### What it cannot answer

- Only on a 6 MiB ring, two disks, overwrite and unlink; ring length, disk count, truncation, snapshots and compaction moves are not swept (registration section 8, item 4).
- The numbers at 16 TiB per disk are extrapolated, not measured; tombstone containers are measured only to 13 (123 in the second segment), 1227 is derived.
- The cost of copying bytes belongs to E199 (cost of copying the previous version at publish); this page asks only about scans.
- Steps outside the list are covered by the residual row (Q10); index builds and code-two planning with no attribution guard are recorded on one off-list row and reported separately, not dropped.
- Some readings are the apparatus's own operationalisation (the registration did not spell them out): B8's `m_round` is the placements released in a fixed-point round plus the roles newly allocated; `dirty` is the number of records that differ before and after one call; `dm` is twice the rewritten roles (one out, one in); in the maintenance term `m_rec` is the same sum in the last round and `m_seg` takes it as an upper bound; tree heights are read from the root node headers. B8's step count at the user-scale is extrapolated the same way as Q4 (the `user_scale` rows of Q6, Q8 and Q9, derived); the bands still use the measured points only and the user-scale rows are reported as numbers.
- s9, s10 and s17 count record numbers only, because the shape copies sit in Verus blocks where nothing can be inserted; s3's timer covers the whole writer-call statement (the copy is inside a Verus block) and so includes the writer's non-copy work; these four rows are copy rows and stay out of the scan sum.
- s32 counts only the release of the instance-table chain (overwrites and unlinks at the sample points write no row, so it mostly is not reached; inferred, not measured).

### Paths and conclusions register

| # | Path | How computed | Source location | Shared items read |
|---|---|---|---|---|
| 1 | Measuring path (B1 counts) | Counters only inserted into a copy of the implementation, the history cases print `E203RESULT` rows per point and state; the sample points are 19 enumerated points, not the whole domain | `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` `edits`, `scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs` `run_the_point` | the code under test itself; the implementation's own constants; the `PoolUnderTest` pool setup |
| 2 | Derived path (counting model) | Pure arithmetic: B8 and B4 step formulas, resident memory, α and banding, extrapolation; anchor unit tests pin absolute values | `scripts/experiment-apparatus/e203_admission_scan_cost.rs` `by_delta_rows`, `narrowed_steps_by_segment`, `full_report` | local constants (sources noted in the file header, the implementation's values in the artifact header compared one by one); the B8 column of registration section 5 |

The reconciliation of the two paths (registration section 7, check ③2) covers the element counts of rows s1 and s24 and the steps of five rows that can be written as a formula from the size readings (the plan for the other files and the set of inodes with an entry must be equal; the linear find over the other files, the sum over the cluster segments and the linear scan for an empty segment are upper bounds); the other 21 rows take table lengths of another version, answer positions or per-round deltas, so no formula that eats the size readings alone exists, and the artifact names them on one `three_two_rows_not_reconciled` row; this is still owed.

### Affected decisions

| Decision item | Relation | Review |
|---|---|---|
| D8 (core index structure) settled item 16 | groundwork | 2026-11-05 unaffected: the artifact has not been run, the registration and first-segment apparatus are written; the sentence "the step cost before acceptance does not grow with the total container or record count" in the scope of settled item 16 waits for the artifact |

## Historical Versions

### 2026-11-05

- File created, registering E203 (step cost of the scans before admission) pre-registration and first-segment apparatus; the artifact has not been run once.
- Close of the first segment: the self-test's guard count in the counter-insertion script now runs per file and names are full words; the model gained the user-scale steps, the ③2 reconciliation on five rows, void clause 8, unit tests for stop clauses 3 and 4 and end-to-end cases for the failure clauses; the artifact is still unrun.
