## E186 Unknown Trees: Accounting Dimension Rederived —— Run (first run: Q1 and Q2 sufficient to judge, mutation table 17 entries all caught; second run: Q1 sufficient to judge, mutation table 26 entries all caught)

[中文](186-不认识的树-记账维重推.md) · **English** · [日本語](186-未知のツリー・記帳次元の再導出.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 U1 U2 U3 U4 U5 U6 U7 U8 U9 U10 U11 U12 U13 B1 B2 W1 W2 T0 -->

Preregistration `research/prompts/e186-preregistration.md` (section 12, amendment one: negative control B judges only the clauses counted under reading B1; the two incidental quantities are computed in the same pass as the judged quantities and reported without being judged; written after the unit tests and before the artifact). It answers two rows of the question list `research/prompts/c693-forks.md`, only their first segment: Q1 (when an old writer meets an unknown tree, which arm the accounting dimension takes: `Carry` / `NoCarry` / keep-with-root) and Q2 (whether the "EIO but the position is still there" of `RejectEntry` in the flags dimension counts toward the newly added position). The readings are fixed: reading B takes B1 (㊀ ㊁ of D8 (Core Index Structure) settled item 8 are not in the clause list), reading W takes W2 (rows written per generation = all rows in this version of the accounting tree); B2 and W1 are reported only as incidental quantities.

Harness `scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs` (an independent hand-written model, pure integer arithmetic, sharing no code with `crates/`), `[[bin]]` name `e186-unknown-tree-accounting-rederive`, 22 unit tests at the first run (35 after the second run, see the section "Second Run"); mutation table `scripts/mutation/tables/e186_unknown_tree_accounting_rederive.tsv` with 17 entries at the first run (26 after the second run). 11520 sample points (the 120 points of E113 (Unknown Trees: Saturated Arm Set) × four tiers of publish generations `G` × six (`s`, `lpc`) pairs × two tiers of `r_tree_dim` × two disk geometries, E113 and implementation), 15 cells per point (5 flags arms × 3 accounting arms), 172800 cells in total. Lexicographic order over seven positions, compared position by position, smaller is better: wrong release → semantic violation → dropped rows (tuple measurement basis) → silent → unwritable generations → number of settled clauses to amend → rows written per generation (W2). Position 6 is the one the user added on 2026-10-20.

### Current State

- Artifact `research/results/e186-unknown-tree-accounting-rederive.out.gz`: 186312 lines, last line `E7RESULT name=done emitted=186312`, `sha256sum` `f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a`. The same binary run again into the draft directory, `cmp` byte-identical (deterministic model: this only shows there is no hidden state; evidence strength comes from mutations and from the assertions that pin absolute values).
- Memory peak (peak table of `run-with-memory-cap.sh`, cap 3G): unit tests with compilation 416 MiB, artifact 321 MiB.
- 9 positive controls, negative controls A, B and C, the two ledger paths, 24 anchors, the two model-side stop clauses and three discriminative power self-tests all hold; no verdict line contains `false`.
- **Q1 (accounting dimension)**: on legal points ((`s`, `lpc`) ≠ (0, 0)) the judgement is "depends on the sample point". With today's values `r_tree_dim = 1` and `h_new > 0`, on those 3200 points `Carry` is never a winner: on the 1280 points with `lpc = 0` the winner is `NoCarry` (position 6 separates it from keep-with-root, position 7 separates it from `Carry`), on the 1920 points with `lpc = 1` `NoCarry` and keep-with-root tie (position 7 separates them from `Carry`). With `r_tree_dim = 2` and `h_new > 0`, on those 3200 points the winner is `Carry` (position 3, dropped rows, separates it from `NoCarry`; position 6 separates it from keep-with-root). On the 1600 points with `h_new = 0` all three tie.
- **Q2 (`RejectEntry` against `Skip`)**: under the reading "counted", in each of the two groups of 4800 legal points, 1200 points (all points with `w_gens = 0`) tie on positions 1–5 and separate on position 6 (`Skip` smaller); under "not counted" 0 points do. On those 1200 points the flags winner changes tier with the reading: under "counted" it is `Skip` (plus `PerBit` when `registered = 4`), under "not counted" `Skip` and `RejectEntry` tie; on the 3600 points with `w_gens > 0` the winner is `RejectEntry` under both readings. The discriminative power self-test turns from green to red (8640 cells, 0 cells not turning red).
- By the letter of the reverse acceptance clause in section 10 of the registration: on legal points the Q1 accounting winner is not {`Carry`}; under the two Q2 readings the flags winner differs on the points with `w_gens = 0`. The consequences the clause writes down (㊀ flips, ㊁ reopens) are inferences, handed to the main agent for a three-way argument; this page does not judge them.
- Mutation table 17 entries: 17 caught, 0 invalid, 0 not red (the main agent ran `mutate.sh`, cap 3G; all six columns of the closing "counts:" line are 0, "restored, baseline still all green", log `~/.local/share/singlefs/work/m3-close-w8-e186-run2/mutate-1.log`).

### Rerun

```bash
# 仓根；产物逐字节复跑登记在 scripts/experiment-apparatus/replay.sh：E186|e186-unknown-tree-accounting-rederive||e186-unknown-tree-accounting-rederive.out.gz|exact
cd scripts/experiment-apparatus
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 e186-unknown-tree-accounting-rederive 的单测; 范围=1 个测试目标; 为什么=复核单测' \
  bash ../hosts/capped.sh 31 bash ../hosts/run-with-memory-cap.sh 3G cargo test --release --bin e186-unknown-tree-accounting-rederive
bash ../hosts/capped.sh 31 cargo build --release --bin e186-unknown-tree-accounting-rederive
bash ../hosts/run-with-memory-cap.sh 3G target/release/e186-unknown-tree-accounting-rederive > <新文件名>.out
nice -n 19 bash ../mutation/mutate.sh e186-unknown-tree-accounting-rederive e186_unknown_tree_accounting_rederive.rs ../mutation/tables/e186_unknown_tree_accounting_rederive.tsv
```

### Readings: Q1 Accounting Winners (judged readings B1 and W2, Q2 read as "counted"; under both Q2 readings Q1 is identical point by point, `research/results/e186-unknown-tree-accounting-rederive.out.gz` line 186048)

`research/results/e186-unknown-tree-accounting-rederive.out.gz` lines 186059, 186060, 186126, 186127, 186131, 186132:

```
E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=Carry+NoCarry+KeepWithRoot:1600|NoCarry:1280|NoCarry+KeepWithRoot:1920
E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_2 arm=all distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
E7RESULT name=verdict check=q1_winners reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=RejectEntry distinct_winner_sets=3 judgement=depends_on_point winner_sets=Carry+NoCarry+KeepWithRoot:1600|NoCarry:1280|NoCarry+KeepWithRoot:1920
E7RESULT name=verdict check=q1_winners reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=Skip distinct_winner_sets=3 judgement=depends_on_point winner_sets=Carry+NoCarry+KeepWithRoot:1600|NoCarry:1280|NoCarry+KeepWithRoot:1920
E7RESULT name=verdict check=q1_winners reading=b1_w2_q2_counted group=legal_r_tree_dim_2 arm=RejectEntry distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
E7RESULT name=verdict check=q1_winners reading=b1_w2_q2_counted group=legal_r_tree_dim_2 arm=Skip distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
```

Under the two flags arms `Ignore` and `ReadOnly` the accounting arms have no point of action, and the three tie on all points (`same_set_everywhere`); `PerBit` equals `Skip` when `registered = 4` and `ReadOnly` when `registered = 0`, and the `PerBit` line of the positive controls holds. Points on the E113 measurement basis ((`s`, `lpc`) = (0, 0), not legal today) are reported separately: under both tiers of `r_tree_dim`, on the 640 points with `h_new > 0`, the winner under `Skip` and `RejectEntry` is `Carry` (separated on position 3), which differs from the judgement on legal points with `r_tree_dim = 1`; both are stated side by side as measured.

First separating position (legal points, same counts for `Skip` and `RejectEntry`): with `r_tree_dim = 1`, position 6 on 1280 points, position 7 on 1920 points, all equal on 1600 points; with `r_tree_dim = 2`, position 3 on 3200 points, all equal on 1600 points. How it was counted:

```bash
grep 'name=summary kind=q1_first_separating reading=b1_w2_q2_counted group=legal' research/results/e186-unknown-tree-accounting-rederive.out.gz | grep -E 'arm=(Skip|RejectEntry) ' | awk '{print $5,$6,$7,$8}'
```

```
group=legal_r_tree_dim_1 arm=RejectEntry value=6 points=1280
group=legal_r_tree_dim_1 arm=RejectEntry value=7 points=1920
group=legal_r_tree_dim_1 arm=RejectEntry value=none points=1600
group=legal_r_tree_dim_1 arm=Skip value=6 points=1280
group=legal_r_tree_dim_1 arm=Skip value=7 points=1920
group=legal_r_tree_dim_1 arm=Skip value=none points=1600
group=legal_r_tree_dim_2 arm=RejectEntry value=3 points=3200
group=legal_r_tree_dim_2 arm=RejectEntry value=none points=1600
group=legal_r_tree_dim_2 arm=Skip value=3 points=3200
group=legal_r_tree_dim_2 arm=Skip value=none points=1600
```

The seven positions of one point (implementation two-disk geometry, `h_new = 2`, `s = 1`, `lpc = 0`, `r_tree_dim = 1`, `w_gens = 3`, `G = 10`, point 8824), field `vector_q2_counted` of lines 141192 to 141194 of `research/results/e186-unknown-tree-accounting-rederive.out.gz`: `RejectEntry_Carry` is `0,0,0,0,0,1,20`, `RejectEntry_NoCarry` is `0,0,0,0,0,1,18`, `RejectEntry_KeepWithRoot` is `0,0,0,0,0,7,20` (clauses false under keep-with-root: `U1:1/10/1+U2:1/10/1+U3:1/10/1+U4:1/10/1+U6:1/10/1+U7:1/10/1+U12:1/10/1`, U12 counted only under B2). W2 20 = its own rows 15 + one first-level step row for the known head + 2 × (`s` + `lpc` + `r_tree_dim`); what `NoCarry` lacks is the "rows of this generation" row of each head, the watermark tuple can still be read from the step rows, and dropped rows are 0.

### Readings: Q2 (`RejectEntry` Against `Skip`, Flags Winners Under Both Readings)

`research/results/e186-unknown-tree-accounting-rederive.out.gz` lines 186275, 186276, 186279, 186280, 186291, 186292, 186044:

```
E7RESULT name=verdict check=q2_overall_winner_flags_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=RejectEntry:3600|Skip:600|Skip+PerBit:600
E7RESULT name=verdict check=q2_overall_winner_flags_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_2 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=RejectEntry:3600|Skip:600|Skip+PerBit:600
E7RESULT name=verdict check=q2_overall_winner_flags_arms reading=b1_w2_q2_not_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=RejectEntry:3600|Skip+RejectEntry:600|Skip+RejectEntry+PerBit:600
E7RESULT name=verdict check=q2_overall_winner_flags_arms reading=b1_w2_q2_not_counted group=legal_r_tree_dim_2 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=RejectEntry:3600|Skip+RejectEntry:600|Skip+RejectEntry+PerBit:600
E7RESULT name=verdict check=q2_reject_entry_versus_skip group=legal_r_tree_dim_1 points=4800 separable_points_q2_counted=1200 separable_points_q2_not_counted=0 flags_winners_changed_points_b1_w2=1200 flags_winners_changed_points_b2_w2=1200 judgement_b1_w2=changes_tier
E7RESULT name=verdict check=q2_reject_entry_versus_skip group=legal_r_tree_dim_2 points=4800 separable_points_q2_counted=1200 separable_points_q2_not_counted=0 flags_winners_changed_points_b1_w2=1200 flags_winners_changed_points_b2_w2=1200 judgement_b1_w2=changes_tier
E7RESULT name=verdict check=self_proof_q2_reject_entry_versus_skip_position_six_where_w_gens_0 checked=8640 not_turning_red=0 turns_red=true
```

Flags winners by accounting arm (`check=q2_flags_winners`, artifact lines 186193 to 186264): `Carry` and keep-with-root have the same shape as the overall winners; under `NoCarry`, on legal points with `r_tree_dim = 2` the winners include `ReadOnly` (1600 points) and `ReadOnly` plus `PerBit` (1600 points) — there `NoCarry` drops rows on position 3, while the read-only mount drops none and loses on position 5. The real baseline `RejectEntry_Carry` (the cell of today's ㊀ ㊁ text) is among the overall winners in 2800 of the 9600 (point, reading) pairs on legal points with `r_tree_dim = 1` (1200 under "counted", 1600 under "not counted"), and under neither reading on the 3200 points with `h_new > 0`; with `r_tree_dim = 2` it is on 3600 points under "counted" and 4800 points under "not counted". How it was counted:

```bash
grep ' name=point_verdict ' research/results/e186-unknown-tree-accounting-rederive.out.gz | awk '{for(i=1;i<=NF;i++){if($i~/^legal=/)l=$i; if($i~/^r_tree_dim=/)r=$i; if($i~/^overall_winners_q2_counted=/)c=$i; if($i~/^overall_winners_q2_not_counted=/)n=$i} ic=(c~/RejectEntry_Carry/)?"base_in_counted":"base_out_counted"; in_=(n~/RejectEntry_Carry/)?"base_in_not_counted":"base_out_not_counted"; print l, r, ic, in_}' | sort | uniq -c
```

```
    720 legal=0 r_tree_dim=1 base_in_counted base_in_not_counted
    240 legal=0 r_tree_dim=1 base_out_counted base_in_not_counted
    720 legal=0 r_tree_dim=2 base_in_counted base_in_not_counted
    240 legal=0 r_tree_dim=2 base_out_counted base_in_not_counted
   1200 legal=1 r_tree_dim=1 base_in_counted base_in_not_counted
    400 legal=1 r_tree_dim=1 base_out_counted base_in_not_counted
   3200 legal=1 r_tree_dim=1 base_out_counted base_out_not_counted
   3600 legal=1 r_tree_dim=2 base_in_counted base_in_not_counted
   1200 legal=1 r_tree_dim=2 base_out_counted base_in_not_counted
```

### Geometry Sensitivity (Section 8 of the Registration)

`check=sensitivity`, 19 lines (artifact lines 186293 to 186311): the four classes `lpc` 0 ↔ 1, `r_tree_dim` 1 ↔ 2, (`s`, `lpc`) (0, 0) ↔ the other five pairs, and `h_new` 0 ↔ 2 / 4 all "depend on the sample point" for the Q1 accounting winners; `w_gens` 0 ↔ 1 / 3 / 8 and `registered` 0 ↔ 4 "depend on the sample point" for Q2; the three pairs E113 geometry ↔ implementation two-disk geometry, `G` 1 ↔ 10 and `k_keep` 1 ↔ 20 are "same judgement at both points" for both Q1 and Q2 (`flipped_points=0`). Changing the geometry does not change the judgement: the two geometries differ only in their own rows and the step rows of the known head, and the three accounting arms each gain the same number.

### Artifact Completeness, Verdict Lines and Fields That Did Not Pass

```bash
P=research/results/e186-unknown-tree-accounting-rederive.out.gz
awk '{print $2}' $P | sort | uniq -c
grep ' name=cell ' $P | grep -o ' arm=[A-Za-z_]*' | sort | uniq -c | awk '{print $1}' | sort | uniq -c
grep ' name=cell ' $P | awk '{print NF}' | sort | uniq -c
grep -n 'name=verdict' $P | grep -c '=false\|not_run'
```

```
 172800 name=cell
      1 name=config
      1 name=done
      2 name=incidental
  11520 name=point_verdict
   1680 name=summary
    308 name=verdict
     15 11520
 172800 26
0
```

15 cells with 11520 lines each, 26 fields per line; all three `ReadOnly` cells and the half of the three `PerBit` cells with `registered = 0` (5760 lines each) have no new version, and their trajectory field reads `no_new_version`, not 0. 308 verdict lines, none with a `false` / `not_run` field; the counts that mean violation, mismatch or failure (`failures`, `mismatches`, `mismatches_b1`, `mismatches_b2`, `not_turning_red`, `points_where_q1_differs`) are all 0. The counts that are above 0 without meaning a failure are result counts: `separable_points_q2_counted`, `flags_winners_changed_points_*`, `flipped_points`, `distinct_winner_sets`.

### Voiding, Failure and Stop Clauses, One by One

| Clause | Result | Source (`research/results/e186-unknown-tree-accounting-rederive.out.gz`) |
|---|---|---|
| 11520 points, 172800 cells | Holds | line 186002 `count_ok=true` |
| Positive controls (one per arm, two for `RejectEntry`) | all 9 `failures=0` | lines 186003 to 186011 |
| Negative control A (`w_gens = 0` does not release wrongly) | 43200 cells, 0 failures | line 186012 |
| Negative control B (with `h_new = 0` the accounting arms have no effect; judges only B1 clauses) | 672000 items, 0 failures; incidental: U12 false on 0 cells, U13 false on 46080 cells (the non-`RejectEntry` cells among 57600, judged per arm, reported without being judged) | lines 186013, 186014 |
| Negative control C (does not read `k_keep`) | 829440 items, 0 failures | line 186016 |
| Two ledger paths (harness predicates against the hand-computed ledger of section 7) | 172800 cells, 0 differences under both B1 and B2 | line 186017 |
| Class A and class B anchors | 24 entries `mismatches=0` | lines 186018 to 186041 |
| Three discriminative power self-tests | all turn from green to red (8640 / 2400 / 4800 cells, 0 not turning red) | lines 186044 to 186046 |
| Hits an arm that does not discriminate (points where all fifteen cells are equal) | 0 points under both readings | line 186047 |
| Stop clauses 1 and 2 (row count, model side of the step-row reading differential against `crates/`) | `w1_expected=15 w2_expected=16`, 0 differences; step-row reading on 20160 cells, 0 differences | lines 186042, 186043 |
| Source side of stop clauses 1 to 4 (checked before running the artifact) | `accounting_entries_of_this_publish` (`crates/singlefs-core/src/transaction.rs:6755`, pool-level `ALL` 3 rows, per-disk `ALL` 6 rows, step rows copied as is), `inode_number_watermark_steps` (`:3623`) and the syntactic reading starting at `crates/singlefs-checker/src/walk.rs:2649` are all unchanged; the grep for the clause list rerun gives 24 lines, all files and line numbers within the ranges read in section 13 of the registration; the two `quote-kb.py` re-transcriptions of section 2 are byte-identical to the designer's copy under `cmp` | commands and output in the runner's report |
| Discrepancy known before running, not triggering a stop | when `crates/` meets an unknown flags bit it rejects the whole tree table, while the settlement of ㊁ is EIO for that entry; written as is, the main agent decides whether to record it as an owed check | last line of section 3 of the registration |

### Unit Tests and Mutations

- Unit tests (the count at the first run; today's file has 35, see the section "Second Run"): at that time `grep -c '#\[test\]' scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs` printed `22`; one run after amending negative control B: "test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.63s".
- Mutations: table of 17 entries (names taken verbatim from section 9 of the registration), each of the 17 originals matched once in today's source; whole table run: 17 caught, 0 invalid, 0 not red, closing "counts: memory cap hit 0 (cap 3G), timeouts 0, not scheduled 0, squeezed out by the total cap 0, GPU memory short 0, anchors broken 0, judged now 17, reused 0, refused 0", "restored, baseline still all green".

### Fork Table (Question List `research/prompts/c693-forks.md`)

| Fork | Sufficient to judge / what is still missing | Can the remaining quantities flip it | Command that computes it |
|---|---|---|---|
| Q1 three arms of the accounting dimension | Sufficient: all 172800 cells computed; on legal points the accounting winner and the first separating position are reported for every point and every flags arm; judgement "depends on the sample point" (with `r_tree_dim = 1` and `h_new > 0` the winner is `NoCarry` or `NoCarry` plus keep-with-root, with `r_tree_dim = 2` and `h_new > 0` the winner is `Carry`, with `h_new = 0` all three tie) | No: only the mutation table was left unrun; it checks whether the assertions can go red and does not change the numbers in the artifact; an uncaught mutation would weaken the evidence strength, not the readings | `grep 'name=verdict check=q1_winners reading=b1_w2_q2_counted group=legal_' research/results/e186-unknown-tree-accounting-rederive.out.gz`, plus the `check=q1_overall_winner_accounting_arms` and `check=sensitivity` lines |
| Q2 whether the semantics of `RejectEntry` count | Sufficient: flags winners computed under both readings; on the 1200 points with `w_gens = 0` (per group) position 6 separates and the winner changes tier; the self-test of `RejectEntry` against `Skip` turns from green to red (8640 cells) | No: same as Q1 | `grep 'name=verdict check=q2_' research/results/e186-unknown-tree-accounting-rederive.out.gz`, plus `check=self_proof_q2_reject_entry_versus_skip_position_six_where_w_gens_0` |

Both rows are sufficient to judge; the incidental quantities `per_generation_rows_dropped` and `i313_syntactic_violations` are computed in the same pass as the judged quantities per amendment one and reported in the cell lines without being judged.

### Measurement Basis

- Dropped rows count dimension tuples: the number of (statistic, tree) pairs of unknown trees that have rows in R and no row at all in R_G; the count by rows is the incidental quantity `per_generation_rows_dropped`.
- Step rows follow the reading of the range of D5 (Snapshot / Space Accounting Mechanism) settled item 2: split by (statistic, tree), kept only when the watermark of that head changed in that publish (`WatermarkChangeHistory` in the harness), not the syntactic reading of `crates/` today, which does not split by tree; the latter goes only into the incidental quantities `i313_syntactic_violations` and `crates_step_reading_differences`.
- Position 6 counts only U1–U11, which are counted under reading B1: a predicate false on any version counts 1, vacuously true counts nothing; under the Q2 reading "counted", each `RejectEntry` cell gets 1 more. U12 and U13 count only under B2, and the `clauses_false` field of the cell lines lists them.
- Position 7 takes W2: the maximum over R_1 … R_G of the total row count of the accounting tree; cells with no new version (`ReadOnly`, `PerBit` with `registered = 0`) count 0 and lose on position 5.
- Under `Ignore` the writer believes it knows those trees and maintains their rows as its own; all three accounting arms are computed the `Carry` way, and the accounting arms have no point of action.
- The harness is deterministic arithmetic: one run equals several, so no run count is reported.

### What It Cannot Answer

- It measures an independent hand-written model, not the paths of `crates/`: `crates/` has no path for an old writer handling unknown trees, and no cloned heads (its step-row reading does not split by tree); the differential against `crates/` covers only the three stop-clause items.
- Most of the position-6 value of each arm follows directly from the arm definitions and the clause texts (section 4, item 3 of the registration) and is not experimental evidence; what it measures is at which points position 6 is reached, how it changes with `lpc` and `r_tree_dim`, and whether the harness predicates agree with the hand-computed ledger cell by cell.
- `r_tree_dim = 2` is an assumed sample point: a second statistic with a tree dimension does not exist today and was built as "only rows of this generation, no step rows". Q1 judges opposite ways on the two tiers of `r_tree_dim`; which tier represents the future is a trade-off, not a reading.
- Position 7 counts rows, not nodes or units written; whether writing fewer rows means writing fewer nodes was not measured.
- U9 judges only ① of I-9.30 (step rows of inode number watermark match watermark, records, and lineage); ② and ④ need inode records and lineage, which the model does not have.
- What the judgement means for ㊀ ㊁ (flip, reopen, which clauses to amend) is inference and has not gone through a three-way argument.

### Second Run (2026-10-20, Rerun With Changed Criteria)

**Why there is a second run**: cell W1 of section 2 of the judgement `research/prompts/m3-close-c693-r1-main-verification.md` found two errors in the criteria of the first run — U5 (D8 (Core Index Structure) settled item 12, "including the accounting rows carried by old writers of the ㊀ kind of settled item 8") counted 0 as "vacuously true when not written" for `NoCarry` and keep-with-root, while the rows that half-sentence refers to do not exist at all under those two arms; the reverse acceptance clause did not state the consequence of "depends on the sample point" or where overlapping branches belong, and the 1600 points with `h_new = 0`, where the three accounting arms are equal by definition, were counted into the judgement. Changing the criteria makes a new experiment, so a second run was made per the section "第二次跑（重跑登记，2026-10-20）" (second run, rerun registration) of `research/prompts/e186-preregistration.md`; the sections of the first run and the conclusion text above are kept as they were, and the two runs stand side by side. **This run is not blind**: the criteria were changed after the first-run artifact, and before running, the Q1 winners under the new criteria had already been computed by the attacker from the first-run artifact (section W1 of `research/prompts/m3-close-c693-r1-opus-output.md`) and derived once more by the designer (section 4, item 2 of the registration); what the second run measures is not "who wins" but that the numbers the harness computes from the new predicates agree with the hand-computed ledger cell by cell, that the controls and self-tests hold, and that the two criteria agree cell by cell except for U5. The weight of the finding "㊀ does not flip" comes from the reading of U5 and the rewritten reverse acceptance clause, not from newly measured numbers.

**Only two places in the criteria changed** (opening of the second run in the registration): ① the U5 predicate: the number N of rows of unknown trees in R is 0 ⇒ true; N > 0 and the rows rewritten this time for unknown trees under a changed code are empty ⇒ false (to amend); N > 0 and non-empty ⇒ true only if every row's generation segment = this version's txg (under `Carry` as in the original predicate). ② the reverse acceptance clause: judging ㊀ looks only at the points with `h_new > 0` among legal points (3200 points per `r_tree_dim` group, 6400 points combined), the 1600 points with `h_new = 0` (3200 combined) are reported and not judged; by whether the winner set contains `Carry` there are seven shapes, each with one consequence. Everything else (readings B1 and W2, position 7 by rows written per generation, 11520 points, 15 cells, the other twelve of U1–U13) is unchanged to the letter.

The harness is the same bin with an added argument `--judgement <vacuous-u5|dangling-u5>`; without the argument it equals `vacuous-u5`; `grep -c '#\[test\]'` on today's file prints `35` (the 22 tests of the first run kept as they were, one ledger-differential test among them extended to compare under both criteria, plus 13 new ones); mutation table 26 entries (the 17 of the first run plus 9). Section 12 of the registration records amendment one (adds verification lines, fixes the artifact line format, criteria unchanged).

**Artifact** `research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz`: 186285 lines, last line `E7RESULT name=done emitted=186285`, `sha256sum` `ef2bd29e33654aa9d3dab846a98bd297d9fa74c14fbd096648da68316197705e`, command `bash scripts/hosts/run-with-memory-cap.sh 3G <二进制> --judgement dangling-u5` (cap 3G, exit code 0). **A note on provenance**: during this run the merge lock of the main workspace was held by another session and cargo could not start in the main workspace, so the binary was built in a draft copy from the same source (`diff` against `scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs` in the main workspace prints nothing), not the one under `scripts/experiment-apparatus/target/`; the byte-for-byte check is left to the second row of the rerun table (`replay.sh E186`, the main-workspace binary built and compared on the spot). The run without the argument (same draft binary) gives no `cmp` output against the committed first-run artifact, byte-identical (the first-run artifact's `sha256sum` is `f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a`).

```bash
# 复跑：登记在 scripts/experiment-apparatus/replay.sh（两行 E186，第二行带 --judgement dangling-u5，exact）
cd scripts/experiment-apparatus
bash ../hosts/capped.sh 31 cargo build --release --bin e186-unknown-tree-accounting-rederive
bash ../hosts/run-with-memory-cap.sh 3G target/release/e186-unknown-tree-accounting-rederive --judgement dangling-u5 > <新文件名>.out
nice -n 19 bash ../mutation/mutate.sh e186-unknown-tree-accounting-rederive e186_unknown_tree_accounting_rederive.rs ../mutation/tables/e186_unknown_tree_accounting_rederive.tsv
```

#### Readings: Judgement of the Q1 Reverse Acceptance Clause (judged readings B1 and W2, Q2 read as "counted"; the judged point set is the legal points with `h_new > 0`)

`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz` lines 186276 to 186280:

```
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_r_tree_dim_1 judged=yes judged_points=3200 excluded_points_h_new_0=1600 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_r_tree_dim_2 judged=yes judged_points=3200 excluded_points_h_new_0=1600 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_pooled judged=yes judged_points=6400 excluded_points_h_new_0=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:6400
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=e113_style_r_tree_dim_1 judged=no judged_points=640 excluded_points_h_new_0=320 form=unique_carry consequence=carry_stands winner_sets=Carry:640
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=e113_style_r_tree_dim_2 judged=no judged_points=640 excluded_points_h_new_0=320 form=unique_carry consequence=carry_stands winner_sets=Carry:640
```

Judging ㊀ looks only at the `legal_pooled` line: on the 6400 judged points, the accounting arms among the overall winners of the fifteen cells of every point are {`Carry`}, shape `unique_carry`, and the consequence is that ㊀ does not flip. Each of the two groups is also `unique_carry`; so are the two groups on the E113 measurement basis (not legal today, reported separately only). Side by side with the first run (for the first-run judgement see "Readings: Q1 Accounting Winners" above; the old-style line `check=q1_overall_winner_accounting_arms` of the same artifact is still printed in the second-run artifact, lines 186023 and 186024; it counts the 1600 points with `h_new = 0` into the groups, so it is still marked `depends_on_point`, and it is not this run's judgement):

| | First run (`research/results/e186-unknown-tree-accounting-rederive.out.gz`) | Second run (the artifact above) |
|---|---|---|
| U5 predicate | vacuously true when not written | N = 0 true; N > 0 and not written false; written: by generation segment |
| `legal_r_tree_dim_1` | line 186059 `depends_on_point`: `Carry+NoCarry+KeepWithRoot:1600\|NoCarry:1280\|NoCarry+KeepWithRoot:1920` | line 186276 `unique_carry`: `Carry:3200` (the 1600 points with `h_new = 0` are not judged) |
| `legal_r_tree_dim_2` | line 186060 `depends_on_point`: `Carry:3200\|Carry+NoCarry+KeepWithRoot:1600` | line 186277 `unique_carry`: `Carry:3200` |
| Combined | no such line | line 186278 `unique_carry`: `Carry:6400` |

Where they differ: on the 3200 points with `r_tree_dim = 1` and `h_new > 0`, in the first run `NoCarry` on the 1280 points with `lpc = 0` had 0 on position 6 (U5 vacuously true) and on position 7 had `h_new` × `r_tree_dim` rows fewer than `Carry` (each unknown head lacks the rows of that generation), so it won; the new U5 makes position 6 of `NoCarry` 1, and `Carry` wins on position 6. The seven positions of one point (implementation two-disk geometry, `h_new = 2`, `s = 1`, `lpc = 0`, `r_tree_dim = 1`, `w_gens = 3`, `G = 10`, point 8824), field `vector_q2_counted` of artifact lines 141192 to 141194: `RejectEntry_Carry` is `0,0,0,0,0,1,20`, `RejectEntry_NoCarry` is `0,0,0,0,0,2,18`, `RejectEntry_KeepWithRoot` is `0,0,0,0,0,8,20` (the same point in the first run was `…,1,20`, `…,1,18`, `…,7,20`); field `clauses_false`: `Carry` is `none`, `NoCarry` is `U5:1/10/1+U12:1/10/1`, keep-with-root gains `U5:1/10/1` (U12 counted only under B2).

Geometry sensitivity (`check=sensitivity`, Q1): the two pairs `lpc` 0 ↔ 1 and `r_tree_dim` 1 ↔ 2 become "same judgement at both points" under the new criteria (in the first run they "depended on the sample point"), lines 186257 and 186258:

```
E7RESULT name=verdict check=sensitivity pair=lpc_0_to_1_where_h_new_positive_and_r_tree_dim_1 verdict=q1_accounting_winners pairs=1920 judgement=two_points_same flipped_points=0 flipped_geometry=none flipped_k_keep=none flipped_w_gens=none flipped_h_new=none flipped_registered=none flipped_g_gens=none flipped_s=none flipped_lpc=none flipped_r_tree_dim=none
E7RESULT name=verdict check=sensitivity pair=r_tree_dim_1_to_2 verdict=q1_accounting_winners pairs=5760 judgement=two_points_same flipped_points=0 flipped_geometry=none flipped_k_keep=none flipped_w_gens=none flipped_h_new=none flipped_registered=none flipped_g_gens=none flipped_s=none flipped_lpc=none flipped_r_tree_dim=none
```

The only Q1 pairs that still "depend on the sample point" are `h_new` 0 ↔ 2 and 0 ↔ 4 (lines 186268 and 186269, 3840 points each): with `h_new = 0` the three accounting arms are equal by definition (negative control B), with `h_new > 0` `Carry` wins alone, so the only flipping variable is `h_new`, which is exactly why the second run takes the points with `h_new = 0` out of the judged point set. Q2 is not re-judged (the artifact still computes it; the 732 lines of `check=q2_*` and `kind=q2_*` give an empty `diff` against the first-run artifact).

#### Controls, Self-Proofs and Stop Clauses (Second Run)

| Item | Result | Source (second-run artifact) |
|---|---|---|
| Three positive controls for the new U5 (`Carry` true on every version; `NoCarry` and keep-with-root false on every version, versions false = `G`) | all `checked=19200 failures=0` | lines 185972 to 185974 |
| New self-test (on points with `h_new = 2`, `lpc = 1`, `r_tree_dim = 1`, switching U5 of `NoCarry` back to vacuously true lowers position 6 by 1) | turns from green to red, `checked=2400 not_turning_red=0` (the three self-tests of the first run also turn from green to red) | line 186010 |
| The original nine positive controls, negative controls A, B and C, the two ledger paths (the `dangling-u5` ledger against the new hand-computed table cell by cell), 24 anchors | hold, no verdict line contains `false` | `grep -n 'name=verdict' <产物>` |
| Stop clause 5 (the artifact without the argument is byte-identical to the first run's) | draft binary, `cmp` prints nothing; the check with the main-workspace binary is in the first row of the rerun table | command above |
| Stop clause 6 (the two criteria agree cell by cell, no cell outside U5 may differ; the Q2 judgement may not differ) | `cells=172800 mismatches=0`, `points=11520 mismatches=0`; the 732 lines of `check=q2_*` and `kind=q2_*` give an empty `diff` | lines 186282, 186283 |
| Stop clause 7 (the new U5 judged by N agrees with judging by `h_new > 0`) | `points=11520 mismatches=0` | line 186284 |
| Voiding addendum (judged points 3200 / 3200 / 6400, not judged 1600 / 1600 / 3200, shape not `no_judged_points`) | `checked=3 mismatches=0` | line 186281 |

Artifact completeness (registered arms × histories × geometries counted once):

```bash
P=research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz
awk '{print $2}' $P | sort | uniq -c
grep ' name=cell ' $P | grep -o ' arm=[A-Za-z_]*' | sort | uniq -c | awk '{print $1}' | sort | uniq -c
grep -n 'name=verdict' $P | grep -c '=false\|not_run'
```

```
 172800 name=cell
      1 name=config
      1 name=done
      2 name=incidental
  11520 name=point_verdict
   1640 name=summary
    321 name=verdict
     15 11520
0
```

15 cells with 11520 lines each; 321 verdict lines (308 in the first run; the extra 13 are three positive controls, one self-test, five acceptance-clause lines, and four stop-clause and voiding-addendum lines), none with a `false` / `not_run` field; the counts that mean violation, mismatch or failure (`failures`, `mismatches`, `mismatches_b1`, `mismatches_b2`, `not_turning_red`, `points_where_q1_differs`) are all 0.

#### Unit Tests and Mutations (Second Run)

- Unit tests: `grep -c '#\[test\]' scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs` prints `35`; run in the draft copy: "test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.75s" (inside the test process each criterion computes the 11520 points only once, so it is shorter than the 33.63 seconds of the first run). The unit tests in the main workspace did not run because of the merge lock; the baseline of the mutation table was built by `mutate.sh` in its copy from the main-workspace source file, "restored, baseline still all green".
- Mutations: table of 26 entries (the 17 of the first run plus the 9 of section 9 of the registration), each of the 26 originals matched once in today's source; whole table run: 26 caught, 0 invalid, 0 not red, closing "counts: memory cap hit 0 (cap 3G), timeouts 0, not scheduled 0, squeezed out by the total cap 0, GPU memory short 0, anchors broken 0, judged now 26, reused 0, refused 0", "restored, baseline still all green", `✅` 26 entries. The log is `mutate-1.log` in the runner's draft directory. The 17 entries of the first run are all still caught by the unit tests under the new criteria; `no_carry_waits_k_generations` makes `NoCarry` write rows the `Carry` way when `G` is not greater than `k_keep`, and the new U5 follows (U5 of `NoCarry` turns from false to true at those points); it is caught by the `k_keep` differential, the positive controls under the new criteria and the first-run `NoCarry` unit tests.
- The nine new entries are caught by these tests: the four for shape consequences (`verdict_judges_h_new_zero_points`, `verdict_tie_with_carry_flips`, `verdict_tie_without_carry_keeps_carry`, `verdict_mixed_counts_as_carry_everywhere`) by the shape unit tests on synthetic input, `every_winner_form_gets_its_registered_consequence` and `points_with_no_unknown_writable_head_are_reported_but_not_judged` (the artifact only reaches the `unique_carry` line, the other lines rely only on this group of unit tests, a fix for insensitive sample points); `default_judgement_is_dangling` by the unit tests of the first run; `u5_by_arm_label` by the ledger differential and `dangling_u5_counts_nothing_where_the_accounting_arm_does_not_act_or_there_is_no_unknown_head`; the other three by the unit tests and positive controls of the new U5.

#### Fork Table (Q1 of Question List `research/prompts/c693-forks.md`)

| Fork | Sufficient to judge / what is still missing | Can the remaining quantities flip it | Command that computes it |
|---|---|---|---|
| Q1 three arms of the accounting dimension (second run, new criteria) | Sufficient: under `dangling-u5` all 172800 cells computed; the 6400 judged points of `legal_pooled` have shape `unique_carry`, consequence ㊀ does not flip; each of the two groups is also `unique_carry`; controls, self-tests, stop clauses 6 and 7 and the voiding addendum hold | No: what remains undone is the byte-for-byte check of the main-workspace binary (`replay.sh E186`); it checks the artifact bytes and does not change the judgement | `grep 'name=verdict check=q1_carry_acceptance_form' research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz` |

#### What It Cannot Answer (Second Run Addendum)

- The weight of the second run's "㊀ does not flip" comes from the reading of U5 and the rewritten reverse acceptance clause, not from newly measured numbers (the value of the new U5 follows directly from the arm definitions, `.claude/singlefs-ai-sop/rules/test-discipline.md` "门槛不能是臂定义的同义反复", a threshold must not be a tautology of the arm definition); it is the result under the reading set by cell W1 of the judgement, the result under the first run's reading is still the section above, the two stand side by side, and which one bears the weight is left to the main agent.
- What the judgement means for ㊀ is still inference and has not gone through a three-way argument.
- Otherwise as in the first run: an independent hand-written model, `crates/` has no path for an old writer handling unknown trees; `r_tree_dim = 2` is an assumed sample point; position 7 counts rows, not nodes.

### Paths and Conclusions Registry

Conclusions "position 6 as computed by the harness from predicates written from the clause texts agrees cell by cell with the hand-computed ledger of section 7 of the registration" and "rows written per generation equal the closed forms in the registration": the same quantity computed once on each of two paths and then compared (from the second run on, the ledger differential compares under each of the two criteria).

| # | Path | How it computes | Source placement | Shared items it reads |
|---|---|---|---|---|
| 1 | Harness predicates | Each U predicate judged on every version R_1 … R_G, false on any version counts 1, summed by reading; sweeps all 172800 cells | `scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs:685 clause_holds_on_version` | How the U1–U13 predicates are written in section 6 of the registration; sample-point geometry; the step-row reading `WatermarkChangeHistory`; the versions derived by `published_history_of`; `effective_flags_policy_of` (`PerBit` degeneration, points of action of the flags arms) |
| 2 | Hand-computed ledger | The clause ledger (B1) of section 7 of the registration and the B2 additions, looked up per cell | `scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs:743 clause_units_from_the_hand_ledger` | The section 7 ledger (the same derivation as section 4, item 3); `effective_flags_policy_of`, shared with path 1, so if it is wrong both paths go wrong together |
| 3 | Harness counts rows version by version | Counts all rows of the accounting tree of each version (W2) and the rows whose generation segment equals this version's txg (W1) | `scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs:885 evaluate_cell` | The constants for the own rows of the two geometries and the step rows of the known head (original values of E113; 3 + 6 × 2 of `crates/` and one level of step rows) |
| 4 | Closed forms | The `python3 -I -c` of items 9 and 10 of section 13 of the registration computes by formula at five enumerated points (class B table of section 7); it does not sweep the domain, because the closed forms are in the registration only to pin absolute values | No implementation: a one-line command in the registration, no script kept in the repository | The same group of geometry constants; the three formulas of section 4, item 6 |

Cross-path comparison assertions: path 1 against path 2 is pinned in the unit test `device_predicates_and_the_hand_ledger_agree_on_every_cell_under_both_readings` (`:2751`, compares under both criteria) and in the artifact `check=ledger_two_paths`; path 3 against path 4 is pinned in `anchor_failures` (`:1798`) and the unit test `rows_written_match_the_registered_closed_forms`.

### Influenced Decisions

| Decision item | Relationship | Retrospective |
|---|---|---|
| D8 (Core Index Structure) settled item 8 | Preparation | 2026-10-20 unaffected: the basis paragraph of that item cites E112 (Legacy Writer Encountering Unknown Trees) and E113 (Unknown Trees: Saturated Arm Set), not E186 (Unknown Trees: Accounting Dimension Rederived), and this page only prepares material; the first run, by the letter of the old criteria, has Q1 winners without `Carry` on legal points with `r_tree_dim = 1`, the second run, by the new criteria (`unique_carry` on the judged points), has ㊀ not flipping, the two stand side by side, and the Q2 winner changes tier with the reading; whether this cell should be raised to supporting or overturning needs a three-way argument and is left to the main agent; the decision body was not changed this time |
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | Preparation | 2026-10-20 unaffected: the basis paragraph of that item does not cite E186 (Unknown Trees: Accounting Dimension Rederived); keep-with-root needs an exception for it when `lpc = 0` (U1 false); this page only measures clause counts and sample points, the second run only changed U5 and the reverse acceptance clause and did not touch U1, and the decision body was not changed this time |
| D8 (Core Index Structure) settled item 12 | Preparation | 2026-10-20 unaffected: the basis paragraph of that item does not cite E186 (Unknown Trees: Accounting Dimension Rederived); the U5 of the second run reads the half-sentence of its settlement "including the accounting rows carried by old writers of the ㊀ kind of settled item 8"; under `NoCarry` and keep-with-root those rows do not exist, the clause refers to nothing and counts as "to amend"; this page only prepares material, and the decision body was not changed this time |

## Historical Versions

The history entries of E186 (Unknown Trees: Accounting Dimension Rederived) are collected in [experiments-history.md](../experiments-history.md).
