## E201 Three-oracle collusion full table on the current tree — Apparatus written, artifacts not run

[中文](201-三oracle合谋今树整表.md) · **English** · [日本語](201-3オラクル共謀現ツリー全表.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 P0 P1 P2 P3 T1 T2 T3 R7 G1 G2 G3 G4 G5 G6 G7 K2 K3 V1 V14 F3 F5 F6 F7 Z1 Z2 Z3 Z5 Z6 Z7 Z8 S1 S2 S6 S7 O1 O2 O3 -->

Preregistration `research/prompts/e201-preregistration.md` (section twelve revision 1 written before the apparatus, revision 2 written before any of today's artifacts; unit tests, mutations, self-test and artifacts have not run yet). It answers the two rows of the question list `research/prompts/e171-current-tree-full-table-questions.md`: Q1 asks, on today's main-worktree implementation, which cells change when the 278 rows of the two bad-implementation lists of E171 (Three-Oracle Collusion Test) are run through the three oracles row by row and compared cell by cell with the r1 run, and whether every changed cell traces to something that changed in between; Q2 asks whether each of the three columns misses more or fewer rows than in r1.

A new number rather than a rerun of E171: what runs is another implementation (today's main worktree), the run does not reuse the r1 revision 8 layer (frozen tree, announcement line, placeholders, orchestration pinned by sha256), and the comparison is newly written (preregistration section one). Readings from this run enter no criterion of the first or second E171 rerun, and the r1 readings enter no criterion here; r1 is only the other side of the cell-by-cell comparison (reading 12).

The three points the user decided in a dialog on 2026-11-02 (revision 1): Q2 is judged only on the rows that have per-column readings on both sides (D, at most 7 rows), and the whole-table misses per column are only described, not compared with r1; the stream-three cells count toward the judgeable scope, and segment two follows segment one; "today's tree" is the main worktree at the moment the task is submitted, with HEAD and the worktree change list recorded in the artifacts for attribution, and no announcement line.

### Current state

- Research-side apparatus `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/` (Python, sharing no code with `crates/`; the E171 comparators, `collusion_matrix.py` and the like are imported by path, not changed or copied):
  - `full_table_run.py`: the entry; its header declares sharding, GPU and resume. Three task-server steps — plan copies the current main-worktree tree, records the snapshot digest, HEAD and change list, checks the lists and anchors, looks up the reading 10 keys in prune-server, draws sentinels and splits the rows to run into two shards; each shard builds the unmutated device (gpu feature) and runs the baseline, shard 0 also runs a second baseline, the E170 export against the baseline, the O3 baseline and the nine positive controls twice, both shards run the empty mutation, then the row driver runs the assigned rows and each finished row is packed into the library; merge fetches every row from the library and produces T1–T3, the cell comparison, attribution, Q2, the whole-table matrix, controls and clause checks.
  - `current_tree_row_driver.py`: a copy of the E171 row driver without `nice`, with `--features gpu`, without `nice` on the O3 control step, accepting all nine positive controls; the "compile failure without substitution" branch kept only for replaying old E171 artifacts is removed.
  - `contrast_against_r1.py` (reading 7 keys and comparison), `attribution.py` (reading 8 G1–G7 and the wide attribution), `q2_misses.py` (reading 9 D and misses), `whole_table_matrix.py` (red counts and trajectories per row and column).
  - Three tables: `cell_classes.tsv` (reading 7 classification, sha256 `4c053fa2…`), `device_function_map.tsv` (G5, `95b93eb4…`), `research_function_map.tsv` (G7, `9761c84f…`); the sha256 values are pinned in code, and a mismatch stops comparison and attribution (revision 2).
- Six unit-test modules with 107 tests in all, run by task 6325 with `Ran 107 tests` and `OK`; the self-test `full_table_run.py --selftest` with four cells (reuse three questions, sentinel, reuse account, the stored row matching the full export word for word), task 6326 `FULL_TABLE_SELFTEST cells=4 passed=4 failed=0 failed_cells=-`; six breakage switches (tasks 5540–5545) each red in the named item; the mutation table `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` with 45 rows (V1–V14, 24 landing rows and 7 tree-copy rows), artifact `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-04-r2-s1-mutations.out` ending `E7RESULT name=verdict mutations=45 caught=45 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`.
- Intermediate and preparatory artifacts: initial 14-row mutation artifact (`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-s1-mutations.out`, ending `E7RESULT name=verdict mutations=14 caught=14 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`), second 38-row mutation artifact (`research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r2-s1-mutations.out`, ending `E7RESULT name=verdict mutations=38 caught=38 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`); shard planning stage artifacts `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-s1-plan.out`, `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r2-s1-plan.out`, `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r3-s1-plan.out`, `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-03-r4-s1-plan.out`.
- Neither segment has run; `research/results/` has no artifact of the two E201 segments yet.

### Rerun

From the repository root, each whole command submitted through `task-server/task submit`:

```
python3 -B -m unittest test_contrast_against_r1 test_q2_misses test_attribution test_whole_table_matrix test_current_tree_row_driver test_full_table_run   # --cwd the apparatus directory
python3 mutation_driver.py --table ../../mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv --scratch <scratch directory> --output <new artifact>   # --cwd the apparatus directory
python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py --selftest
python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run                  # segment one
python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run --stream-three   # segment two
```

Product names: `research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-<plan date>-<segment>-<part>.out`, segment `s1` (streams one and two, whole table) or `s2` (stream three), part plan, t1, t2, t3, contrast, attribution, q2, matrix, controls or verdict. The `replay.sh` registration is written once artifacts exist (preregistration 5.6 "at commit").

### Measurement basis

- "Today's tree": the main-worktree `Cargo.toml`, `Cargo.lock`, `.cargo`, `crates/` and `prune-server/`, together with the six repository-root files the crates read when they are built and run, `.claude/heavy-thread-default`, `scripts/crash/crash-point-replay-shard-configuration-check.sh`, `scripts/hosts/peer-host-lib.sh`, `.claude/scripts/preflight.sh`, `.claude/singlefs-ai-sop/scripts/preflight.sh` and `.claude/singlefs-ai-sop/scripts/preflight.py`, copied into the plan directory by the plan step (without `target`, including uncommitted changes of other sessions; `prune-server/` is the repository-root directory that the crates reach through a path dependency, added by preregistration revision 5 because the whole first-segment run stopped at S6 in task 5503; the six files are the one read by the compile-time `include_str!` and the multi-host configuration check that the device starts at run time from the tree it was built from, with the four files that check sources and runs, added by preregistration revision 6 because the whole first-segment run stopped at S6 again in task 5606); each shard syncs its lane trees from it and checks the snapshot digest and file sha256 values at start and end (invalidation Z3). These entries are not hard-coded; `copy_closure` in `scripts/hosts/lib_repository_copy.py` computes them on the spot (preregistration revision 7). `multi-host.env` is not copied: commands started in the copy are given the path of the main working tree's copy explicitly.
- The r1 side reads only three artifacts kept in git: P1 the segment-two comparator, P2 the segment-two row-driver artifact, P3 the segment-one comparator (read only after sha256 and line counts match, invalidation Z5); P0 contributes only the paths of its git status lines, for attribution.
- A changed cell = the same key and decision cell with different values on the two sides; red counts compare as "greater than 0", row-number sets compare as sets, and a key present on one side only is one changed "presence" cell. Pinned-value differences are reported separately and do not enter Q1; P2 fields such as `o3_status`, which say whether whole-table O3 ran, are recorded as today-only (revision 2 item 2.1).
- Attribution recognises only the seven narrow kinds (list, bad-implementation line, substitution, placement function, device function, oracle entry, research-side function); a cell with any hit is attributed, a cell with no hit but an uncheckable kind is uncheckable (F6). The r1 versions of the two comparators and `collusion_matrix.py` are not in the E171 research-side record, so that kind is uncheckable wherever it applies (revision 2 item 2.3).
- Q2's D: rows of R7 (rows 15, 23, 42, 242, 269, 270, 271) where P3 has per-column readings for streams one and two in batch 1, both P1 and today's T1 count the row in the denominator, and today's T3 also has per-column readings; the judgement uses the registration reading and batch 1, with batch 2 and original reading 5 reported alongside.

### Fork table

| Question-list row | Judgeable / still missing | Can the remaining measurements flip it |
|---|---|---|
| Q1 | Missing: unit tests, mutations and self-test, the segment-one run (cell comparison and attribution on the streams one and two whole table), segment two (sixteen rows on stream three) | Yes: all three candidates are open |
| Q2 | Missing: as Q1; the size of D is unknown before the run (empty goes to F4) | Yes: both candidates are open |

### What it cannot answer

- Whether the r1 readings were right (the question list's "not asked"); whether the verdict words change after withdrawing row 236 (judged by the second E171 rerun).
- Whole-table O3 and per-column readings for denominator rows on the r1 side: git does not have them, so only today's side is reported.
- "Attributed" only says the cell falls on one kind of change, not that the change caused it; using it as causation needs a three-way round.

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D13 (Verification Route) settled item 6 | Preparation | 2026-11-05 unaffected: only the usage and rerun notes were changed (one gate, two ways to run; the verification crate was renamed); conclusions, numbers, and artifacts unchanged; previously, 2026-11-03 unaffected: apparatus written, artifacts not run, no readings; E201 (Three-oracle collusion full table on the current tree) answers whether the E171 (Three-Oracle Collusion Test) verdicts change on today's implementation; the evidence block does not cite it, so only preparation |

## Historical Versions

### 2026-11-03

- Page created, registering the E201 (Three-oracle collusion full table on the current tree) research-side apparatus, unit tests, mutation table and decision review; preregistration revision 1 (the user's three decisions) and revision 2 (classification table and the two mapping tables); artifacts not run.
