## E177 Resurrection Count Without Per-Tree Tombstones — Run (sufficient for judgment)

[中文](177-逐树不写墓碑的复活数.md) · **English** · [日本語](177-ツリーごとに墓石を書かない場合の復活数.ja.md)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 K1 K3 K4 K5 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15 V16 V17 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 W1 W2 W3 W4 W5 W6 W7 W8 M1 M11 S2 S16 S17 X5 PC1 PC2 W5b -->

```text
Local arm names and symbols on this page are lowercase (a1, a2, a3, a4 …). The pre-registration, apparatus, artifacts and commands use corresponding uppercase forms, matching global registration identifiers. Lowercase in the knowledge base body distinguishes them from those identifiers; match against uppercase when reading artifacts.
```

Pre-registration `e177-preregistration.md`: “revision 1” in section twelve was written by the experiment designer before the apparatus; “executor revision 2 (checklist)” was written by the executor before the apparatus, with the “G-tight peak live slots” line added before the artifact. This answers C109 (which trees' key deaths write tombstones) in fork sheet `m3-3-5-forks.md`: for each `TREE_KIND_` tree-kind constant in `crates/singlefs-core/src/records.rs` (11 when E177 began; `grep -c "pub const TREE_KIND_"` now gives 12, the extra one being the later `TREE_KIND_RETIREMENT`), how many keys scan rebuild resurrects if no tombstone is written at death, and whether trees without objects can be measured.

The apparatus is an independently handwritten model `scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones.rs`; it shares no code with and does not depend on `crates/`. The write path was handwritten rule by rule from `crates/` as of 2026-10-08 (registered section-three S2–S16, revision 1 W1–W8, executor revision 2 checks 2–4); three read paths were written separately.

**Status**:

- Segment one complete: 51 histories (14 create/delete empty inode, 32 rewrite/delete large file, 5 mixed random) × three geometries (G-ring, the main sample following `crates/`; G-wide and G-tight, model-defined extremes) × three arms (N no tombstone, B whole-object tombstone, P death record for each key death) × three readers (R-U reads every key in every readable unit, R-S selects the current version as settled in the knowledge base, R-G adds object gating to R-S) × 13 tree rows. Q1, Q2 and Q4–Q14 for fork C109 all have statuses, sufficient for judgment; segment two (administrator rollback) was not run.
- Arm N at main sample G-ring, reader R-S: extent, inode, allocation record tree (both measurement basis variants), and `deleted_inodes` are “resurrected”; accounting tree is “measured zero”; tombstone tree, lineage tree and livelist, sparse side table, deadlist, logged_ops have “no object”. R-G matches R-S in every cell. Statuses agree in all three geometries (Q13, all 52 rows `three_point_same=true`). Resurrection peaks vary: inode tree 460 in G-ring, 3 in G-wide and G-tight.
- Code 5 central mapping tree (the additional tree in t2, with objects): “resurrected” on both write paths, both readers and all three geometries.
- Four candidates without objects—dirent, reverse index, free space, LRU—have no writer in `crates/` and no complete on-disk carrier in the knowledge base (dirent has only key encoding); all are “not measurable”. The knowledge base candidate list itself ends in “…”.
- Arm B (whole-object tombstone, actual baseline): inode tree and `deleted_inodes` no longer resurrect, instead “r = 0 but misses live” (between unlink intent and final batch, tombstone suppresses the still-live nlink = 0 record, failure F4); extent, allocation records and mapping still resurrect. Whole-object tombstones cannot cover death caused by shortening a file (overwriting a multi-unit file into one unit): selecting only the 16 F8 e = OU histories, extent and allocation records resurrect under both readers (r peak 299).
- Failure F1 (clause may be wrong) triggered: D8 (Core Index Structure) settled item 6 then said “type 2 record death writes no tombstone; death is represented by the current leaf version”. This fails under R-S without retirement records: inode tree resurrects in arm N/R-S. Mechanism: emptied leaf containers are not persisted, so the latest readable versions of their group still contain dead records; once newer versions in the group are overwritten in G-ring, current-version selection falls back to an older version containing more records. Three parties in three rounds decided not to overturn the entire sentence but to change its wording; the user decided to write a retirement record when container identity disappears (verdict `research/prompts/m3-leaf-death-r3-main-verification.md` section five), and D8 settled item 6 was changed to three levels of death. Neither criteria nor model changed.
- Not a single invalidation clause was triggered, and not a single stop clause was triggered (S-ring1 and S-ring4 were judged in the checklist; all eight anchors for S-ring2 matched; S-ring3 never failed to obtain a placement).

Deterministic model: each history ran twice in the same process and was compared line by line (all 51 identical). This establishes absence of hidden state, not statistical stability. Evidence strength comes from absolute-value anchors, positive controls, the differential of two paths and the mutation table.

### What was done

- **Write path**: two mkfs units (instance table at 50176, two slots; tree table at 50178, one slot), zero-unit warm-ups at txg 1 and 2 and new-pool file creation at txg 3 in the mkfs process, writable mount write-row at txg 4 and warm-up at txg 5, then four session operations at each step (overwrite, sequential write, create empty inode, unlink). Each publish follows `crates/`: first release all placements replaced in this publish, then take placements in bump order, then record root. Rewritten units follow `crates/`: inode leaf container holds 233 records, empty leaves are not persisted, all shifted left on the right are rewritten; extent tree upper section is root and leaf plus lower section (144 units per leaf); allocation record tree is position-addressed by slot number (leaf covers 812 slots, fanout 169, root level 2 for two 4 GiB disks), rewrite set reaches the same fixed point as `crates/`; accounting and central mapping use the same split and shrink rules as `code_two_tree.rs` (477 entries per accounting leaf, 294 per mapping leaf); at most one `deleted_inodes` key; tombstone container holds 815 entries and only grows.
- **Three geometries**: G-ring follows the `crates/` product path, reclaiming by ring within a mount (ring length 24, threshold max(effective F, oldest valid root in ring)); when reclaimed slots are reallocated, allocation records are rewritten or deleted per W6. G-wide has the same placement rules but never returns replaced slots and has no unit-area end. G-tight chooses the lowest slot number, does not model clustered segments, and when placement fails returns every held slot except those replaced by the current publish.
- **Readers**: R-U reads every key in every readable unit (incremental counts, with a separately written full-scan reference path as a differential of the last scan); R-S groups by (birth tree, container number, container birth generation) and (birth tree, object, object birth generation, anchor), selects the latest readable version, recalculates extent, allocation records and mappings from accepted units, selects each accounting row's greatest generation and `deleted_inodes` records with nlink = 0. R-G additionally accepts only data units whose object has a record in the inode tree reconstructed by R-S.
- **Arms**: N writes no tombstones; B writes a whole-object tombstone at unlink intent, suppressing inode records, data units, extent records and `deleted_inodes` entries matching (object, birth generation); P writes a death record per key death (a model-only area consuming no slots) and rejects any record whose death record is at least as recent as the supplying unit.
- **Unspecified registration points for which the apparatus chose a reading** (executor revision 2 check 5, for main-agent acceptance): one allocation-record key per slot number (same slot on both disks); count d as present in an earlier version but absent in the version at scan time (death followed by rebirth does not count); under R-S/R-G, ground truth for allocation records and mapping uses this version's data units and packed containers, so both allocation record measurement basis variants match cell by cell; neither side compares instance-table units; writable mount's number acquisition gives 2; G-tight disk size is 1.25 × the G-wide peak of “live slots in this version + slots replaced in this publish”. The registration's literal last reading (count live slots only) was also run, one `name=tight_literal_reading` row per history and write path: 50 of 102 fit (all 28 create/delete empty inode, 6/10 mixed random, 16/64 large file), excluded from judgment. These 102 rows overturn executor revision 2's inference that none of 51 histories fit after their first few publishes; see “What it cannot answer”.
- **Positive controls**: on each history's final scan, plant one dead key for each of three arms, three readers and each tree with deaths (PC1; arm B separately tests tombstone-identifiable and not identifiable; arm P also measures after deleting k's death record), and erase a unit containing a live key (PC2; expected value supplied by a separate hand-calculated function).
- The apparatus did not time wall clock; compiling and running the full artifact with eight threads (each history twice, all positive controls included) completed within a 240-second foreground limit.

### Replay

```
# Repository root; model is in scripts/experiment-apparatus/ (independent of crates/). E177_THREADS defaults to 8 and affects wall clock only, not output
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
bash ../hosts/capped.sh 8 cargo build --release --bin e177-per-tree-resurrection-without-tombstones
E177_THREADS=8 bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 8 cargo run -q --release --bin e177-per-tree-resurrection-without-tombstones -- segment-one
# Run only the H-ring anchor
bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 8 cargo run -q --release --bin e177-per-tree-resurrection-without-tombstones -- anchor
# Registered replay (scripts/experiment-apparatus/replay.sh entry: segment-one)
cd ../.. && bash scripts/experiment-apparatus/replay.sh E177
# 23 unit tests and 17 mutations
cd scripts/experiment-apparatus
bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 8 cargo test --release --bin e177-per-tree-resurrection-without-tombstones
MUTATE_JOBS=4 MUTATE_MEMORY_MAX=4G bash ../mutation/mutate.sh e177-per-tree-resurrection-without-tombstones e177_per_tree_resurrection_without_tombstones.rs ../mutation/tables/e177_per_tree_resurrection_without_tombstones.tsv
```

| File | Contents |
|---|---|
| `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out` | Segment one: config, eight H-ring anchor rows, `name=simulation` and assertion rows for 51 histories × three geometries × two write paths, 17,901 trajectory rows, 18,490 positive controls, 102 G-tight literal-reading rows, 51 per-history two-run comparisons, 280 verdicts, 26 assertion totals (37,523 lines, sha256 `8708b822a6f5dca9ff4699de82ae2e3b9ccf7d90c4dcb1a68097febb5e2bbbb5`) |
| `e177-per-tree-resurrection-without-tombstones-unit-tests.out` | Unit-test output (23 tests, including memory wrapper and compile lines) |
| `e177-per-tree-resurrection-without-tombstones-mutations.out` | `mutate.sh` output for the whole table (17 mutations; first stderr line comes first) |

Row counts by kind (`awk '{print $2}' research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out | sort | uniq -c`):

```
      8 name=anchor
    306 name=assertions
     26 name=assertion_total
      1 name=config
     51 name=determinism
      1 name=done
  18490 name=positive_control
     51 name=segment_assertions
    306 name=simulation
    102 name=tight_literal_reading
  17901 name=trajectory
    280 name=verdict
```

17,901 = 51 histories × 3 geometries × 3 arms × 3 readers × 13 tree rows. Grouping by family, geometry, arm and reader gives “number of histories × 13” in every group (182 create/delete empty inode, 416 rewrite/delete large file, 65 mixed random), with no missing groups:

```
grep 'name=trajectory' research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["family"], v["geometry"], v["arm"], v["reader"]}' | sort | uniq -c | awk '{print $1}' | sort | uniq -c
```

### Verdict (whole rows copied from `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out`)

Main sample G-ring (Q9–Q12), arms N and B, reader R-S (R-G statuses match cell by cell; see end of section):

```
37321:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=extent status=resurrected r_peak=300 m_peak=0 d_peak=300 segments_judged=51 segments_excluded=0 families_with_resurrection=F3,F8 pc_matched=7 pc_mismatched=0 pc_not_plantable=32 r_in_peak=300 r_out_peak=276 F10_depends_on_placement=false
37322:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=inode status=resurrected r_peak=460 m_peak=0 d_peak=899 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=38 pc_mismatched=0 pc_not_plantable=32 r_in_peak=3 r_out_peak=459 F10_depends_on_placement=false
37323:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=allocation_records_a status=resurrected r_peak=301 m_peak=0 d_peak=1833 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=70 pc_mismatched=0 pc_not_plantable=0 r_in_peak=301 r_out_peak=276 F10_depends_on_placement=false
37324:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=allocation_records_b status=resurrected r_peak=301 m_peak=0 d_peak=1833 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=70 pc_mismatched=0 pc_not_plantable=0 r_in_peak=301 r_out_peak=276 F10_depends_on_placement=false
37325:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=accounting status=measured_zero r_peak=0 m_peak=0 d_peak=27270 segments_judged=51 segments_excluded=0 families_with_resurrection=none pc_matched=102 pc_mismatched=0 pc_not_plantable=0 r_in_peak=0 r_out_peak=0 F10_depends_on_placement=false
37329:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=deleted_inodes status=resurrected r_peak=3 m_peak=0 d_peak=899 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=19 pc_mismatched=0 pc_not_plantable=32 r_in_peak=3 r_out_peak=2 F10_depends_on_placement=false
37330:name=verdict question=Q9 geometry=ring arm=N reader=R-S tree=tombstone status=no_object r_peak=0 m_peak=0 d_peak=0 segments_judged=51 segments_excluded=0 families_with_resurrection=none pc_matched=0 pc_mismatched=0 pc_not_plantable=0 r_in_peak=0 r_out_peak=0 F10_depends_on_placement=false
37345:name=verdict question=Q11 geometry=ring arm=B reader=R-S tree=extent status=resurrected r_peak=299 m_peak=300 d_peak=300 segments_judged=51 segments_excluded=0 families_with_resurrection=F3,F8 pc_matched=37 pc_mismatched=0 pc_not_plantable=39 r_in_peak=299 r_out_peak=187 F10_depends_on_placement=false
37346:name=verdict question=Q11 geometry=ring arm=B reader=R-S tree=inode status=zero_but_missed_live r_peak=0 m_peak=1 d_peak=899 segments_judged=51 segments_excluded=0 families_with_resurrection=none pc_matched=70 pc_mismatched=0 pc_not_plantable=51 r_in_peak=0 r_out_peak=0 F10_depends_on_placement=false
37347:name=verdict question=Q11 geometry=ring arm=B reader=R-S tree=allocation_records_a status=resurrected r_peak=299 m_peak=300 d_peak=1833 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=121 pc_mismatched=0 pc_not_plantable=0 r_in_peak=299 r_out_peak=187 F10_depends_on_placement=false
37349:name=verdict question=Q11 geometry=ring arm=B reader=R-S tree=accounting status=measured_zero r_peak=0 m_peak=0 d_peak=27270 segments_judged=51 segments_excluded=0 families_with_resurrection=none pc_matched=102 pc_mismatched=0 pc_not_plantable=0 r_in_peak=0 r_out_peak=0 F10_depends_on_placement=false
37353:name=verdict question=Q11 geometry=ring arm=B reader=R-S tree=deleted_inodes status=zero_but_missed_live r_peak=0 m_peak=1 d_peak=899 segments_judged=51 segments_excluded=0 families_with_resurrection=none pc_matched=51 pc_mismatched=0 pc_not_plantable=51 r_in_peak=0 r_out_peak=0 F10_depends_on_placement=false
37354:name=verdict question=Q11 geometry=ring arm=B reader=R-S tree=tombstone status=no_death r_peak=0 m_peak=0 d_peak=0 segments_judged=51 segments_excluded=0 families_with_resurrection=none pc_matched=0 pc_mismatched=0 pc_not_plantable=0 r_in_peak=0 r_out_peak=0 F10_depends_on_placement=false
37393:name=verdict question=Q12 geometry=ring arm=N reader=R-S tree=central_mapping status=resurrected r_peak=301 m_peak=0 d_peak=4263 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=70 pc_mismatched=0 pc_not_plantable=0 r_in_peak=301 r_out_peak=276 F10_depends_on_placement=false
37395:name=verdict question=Q12 geometry=ring arm=B reader=R-S tree=central_mapping status=resurrected r_peak=299 m_peak=300 d_peak=4373 segments_judged=51 segments_excluded=0 families_with_resurrection=F1,F3,F8 pc_matched=107 pc_mismatched=0 pc_not_plantable=14 r_in_peak=299 r_out_peak=187 F10_depends_on_placement=false
```

Lineage, livelist, sparse side table, deadlist and logged_ops all have `status=no_object` in all 70 verdict cells: `grep 'name=verdict' <产物> | grep -v 'question=Q13' | grep -E 'tree=(lineage|livelist|sparse_side_table|deadlist|logged_operations) ' | grep -vc 'status=no_object'` outputs `0` (remove final `-v` to count 70). The 20 Q13 rows say `wide=no_object ring=no_object tight=no_object`.

R-G verdict rows (Q10 and the R-G half of Q11/Q11_OU, lines 37333–37392) have the same status as the R-S row for the same question, geometry, arm and tree in every cell: pairing by (question, geometry, arm, tree) and comparing `status` yields no difference (awk pairing on the artifact in “Replay” outputs nothing).

### Readings: per-tree statuses, three geometries side by side (Q1, Q4, Q6, Q8, Q9, Q11, Q12, Q13)

Reader R-S; statuses and r/m peaks by arm, tree and geometry across all 51 histories. The five trees with “no object” are omitted. Command:

```
grep 'name=verdict' research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out | grep -E 'question=(Q1|Q4|Q6|Q8_Q1|Q8_Q4|Q8_Q6|Q9|Q11|Q12) ' | grep 'reader=R-S' | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["arm"], v["tree"], v["geometry"], v["status"], "r_peak=" v["r_peak"], "m_peak=" v["m_peak"]}' | sort -k1,1 -k2,2 -k3,3 | grep -v 'no_object'
```

| Arm | Tree | G-ring (main) | G-wide | G-tight |
|---|---|---|---|---|
| N | extent | resurrected, r 300 | resurrected, r 300 | resurrected, r 300 |
| N | inode | resurrected, r 460 | resurrected, r 3 | resurrected, r 3 |
| N | allocation records (a and b identical) | resurrected, r 301 | resurrected, r 301 | resurrected, r 301 |
| N | accounting | measured zero | measured zero | measured zero |
| N | `deleted_inodes` | resurrected, r 3 | resurrected, r 3 | resurrected, r 3 |
| N | central mapping (code 5) | resurrected, r 301 | resurrected, r 301 | resurrected, r 301 |
| B | extent | resurrected, r 299, m 300 | same | same |
| B | inode | r = 0 but misses live, m 1 | same | same |
| B | allocation records (a and b identical) | resurrected, r 299, m 300 | same | same |
| B | accounting | measured zero | same | same |
| B | `deleted_inodes` | r = 0 but misses live, m 1 | same | same |
| B | tombstone | no death | same | same |
| B | central mapping (code 5) | resurrected, r 299, m 300 | same | same |

Arm N tombstone and lineage trees and four trees codes 6, 7, 8, 12, and arm B lineage and codes 6, 7, 8, 12: no object. All 52 Q13 rows (37397–37448) have `three_point_same=true`, `F9_flips_with_geometry=false`; no tree flips with geometry (F9 not triggered; neither did Q8 governed by F6).

Where resurrection comes from (mechanisms inferred from write-path and reader definitions, checked only on the specifically named trajectories here):

- inode and `deleted_inodes`: an emptied inode leaf container is not persisted (`remove_record_from_leaf_containers` in `crates/singlefs-core/src/inode_tree.rs`), so its group's latest readable versions still contain dead records, including the nlink = 0 record, resurrecting `deleted_inodes` too. When newer versions are overwritten in G-ring, selection falls back to an older version with more records: inode peak 460 in G-ring, 459 outside the ring window (see Q14). Checked trajectories: create/delete empty inode n=900 ascending, G-wide, N, R-S has `r_peak=3 r_peak_at=1821 r_positive_scans=869 r_final=3` (three emptied, unpersisted containers each retain one); n=300 has `r_peak=1 r_peak_at=609 r_positive_scans=1 r_final=1` (one container). Same n=900 history in G-ring `r_peak=460`, `r_in_peak=1`, `r_out_peak=459` (`grep 'name=trajectory family=F1 n=900 order=ascending geometry=ring arm=N reader=R-S tree=inode '`).
- extent, allocation records and mapping: old data units of a deleted or shortened file remain readable until overwritten; after grouping by (object, birth generation, anchor) they remain the latest version of that group, so extent keys, slot numbers and mapping keys return together. Allocation records and mapping also have the contribution from emptied inode leaf containers that are not persisted.
- Accounting: R-S takes the highest-generation row for each row, the 15 rows written by the current version. Under R-U the same tree resurrects (F3: r peak 27,270 in G-wide, 1,050 G-ring, 1,155 G-tight); R-S “measured zero” only means no resurrection under a current-value reading.

### Readings: whole-object tombstones do not cover file shortening (Q11 separately listed)

Only the 16 e = OU large-file rewrite/delete histories (overwrite to one unit before unlink), arm B, G-ring:

```
grep -E 'name=verdict question=Q11_OU .*tree=(extent|allocation_records_a) ' research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out
37369:name=verdict question=Q11_OU geometry=ring arm=B reader=R-S tree=extent status=resurrected r_peak=299 m_peak=1 d_peak=300 segments_judged=16 segments_excluded=0 families_with_resurrection=F8 pc_matched=37 pc_mismatched=0 pc_not_plantable=39 r_in_peak=299 r_out_peak=0 F10_depends_on_placement=false F7_only_one_family=true
37371:name=verdict question=Q11_OU geometry=ring arm=B reader=R-S tree=allocation_records_a status=resurrected r_peak=299 m_peak=1 d_peak=1811 segments_judged=16 segments_excluded=0 families_with_resurrection=F8 pc_matched=121 pc_mismatched=0 pc_not_plantable=0 r_in_peak=299 r_out_peak=0 F10_depends_on_placement=false F7_only_one_family=true
37381:name=verdict question=Q11_OU geometry=ring arm=B reader=R-G tree=extent status=resurrected r_peak=299 m_peak=1 d_peak=300 segments_judged=16 segments_excluded=0 families_with_resurrection=F8 pc_matched=2 pc_mismatched=0 pc_not_plantable=74 r_in_peak=299 r_out_peak=0 F10_depends_on_placement=false F7_only_one_family=true
37383:name=verdict question=Q11_OU geometry=ring arm=B reader=R-G tree=allocation_records_a status=resurrected r_peak=299 m_peak=1 d_peak=1811 segments_judged=16 segments_excluded=0 families_with_resurrection=F8 pc_matched=89 pc_mismatched=0 pc_not_plantable=32 r_in_peak=299 r_out_peak=0 F10_depends_on_placement=false F7_only_one_family=true
```

Extent keys 1 … n_u − 1 that die in the shortening publish have no tombstone (a whole-object tombstone is written only at unlink intent), and return under both readers.

### Readings: Q14, G-ring resurrection split inside and outside ring window

Every “resurrected” cell has r_in peak > 0; F10 triggered in none (`grep 'name=verdict' <产物> | grep -c 'F10_depends_on_placement=true'` outputs `0`). Arm N inode r_in peaks at 3 and r_out at 459: inside the window comes from the latest version of a container emptied without being persisted; outside comes from selection of older versions and depends on placement. Inside-window peaks for extent, allocation records and mapping are 300, 301, 301 (N) and 299 (B), so they resurrect even counting only the ring window.

### Readings: Q5, missed live records only during unlink in arm B

All 45 Q5 rows (37449–37493) are arm B; m is always zero for N and P in every cell (`grep 'question=Q5 ' <产物> | grep -vc 'arm=B'` outputs `0`). In each row m > 0 only between intent and final batch (`m_positive_scans_outside_unlink=0`): inode and `deleted_inodes` have m peak 1 (the whole-object tombstone written at intent suppresses the nlink = 0 record still in the tree); extent, allocation records and mapping have m peak 300 (before batches shorten the file, tombstone suppresses still-live data units). This is the shape specified by F4: r and m in arm B depend on the tombstone effect fixed in registration 5.3 (effective at intent). If K-W5b is settled as “effective only after deletion completes”, those Q4/Q11 cells do not hold. One original row:

```
37453:name=verdict question=Q5 geometry=ring arm=B reader=R-S tree=inode m_peak=1 m_positive_scans=9905 m_positive_scans_in_unlink=9905 m_positive_scans_outside_unlink=0
```

### Readings: Q7, can keyspaces without objects be measured?

Candidates are drawn from the “extent / inode / dirent / reverse index / free space / accounting …” list in `.claude/kb/decisions/08-核心索引结构.md` settled item 14 (“one btree implementation + multiple independent keyspaces + two front ends, derived trees may be heterogeneous”) and LRU in settled item 7 (write buffer entry `seq`: four bytes, in value) and its index row, removing extent, inode and accounting which already have objects. The list ends with “…”, so is not exhaustive.

| Candidate | Writer in `crates/` (registration 13.2 `grep -rln -i '<名>' crates/*/src --include=*.rs`) | Key and on-disk carrier in knowledge base (registration 13.2 `grep -rn '<名>' .claude/kb/decisions/ .claude/kb/layout/`) | Verdict |
|---|---|---|---|
| dirent | `grep -rln -i 'dirent\|目录项' crates/*/src --include=*.rs` has zero hits | Only key encoding `(parent_inode, name)` (dirent entry in `.claude/kb/decisions/08-核心索引结构.md` settled item 5: “whiteout deletion accepted, but batch deletion must use intent log”); no unit class, container shape or record field table | not measurable |
| reverse index | One hit, `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:2439`, a documentation comment, not a node writer | No key (line 229 only requires the deduplication window be bounded by `T_dirty` when the key is defined); only entry seq width is fixed (line 227) | not measurable |
| free space | Zero hits | Appears only in lists (D8 settled item 14; D21 settled item 18, “the bytewise comparison table is only for rebutting ‘we are extravagant’”) | not measurable |
| LRU | Both hits are letters within `FaultLocalRule` (`e165_replay_bounded_enumeration.rs`, `e156_allocation_basis_counts.rs`), not LRU | No key; only seq width and ε = 0 fixed (lines 227, 365) | not measurable |

None has a writer in `crates/`, and none has both key and carrier fully defined in the knowledge base; under the two section-one registered conditions all are “not measurable”. No resurrection count can be given for these additional t2 trees until a keyspace list and carrier definitions exist.

### Anchors (whole rows copied from artifact lines 2–9, H-ring anchor governed by S-ring2)

```
name=anchor history=H-ring-anchor assertion=A17_start txg_after_the_mount=5 publishes_from_txg_3=3 instance_table_slot=50176 instance_table_span=2 tree_table_slot=50178 tree_table_span=1 effective_floor=0 isolated_slots=0 holds=true
name=anchor history=H-ring-anchor assertion=byte_table_new_pool_file_data_unit txg=3 slot=50180 holds=true
name=anchor history=H-ring-anchor assertion=A14_i txg=25 slot=50178 free=false holds=true
name=anchor history=H-ring-anchor assertion=A14_ii txg=26 slot=50178 free=true slots_reclaimed=1 holds=true
name=anchor history=H-ring-anchor assertion=A14_iii txg=27 data_unit_slot=50178 holds=true
name=anchor history=H-ring-anchor assertion=A15_equality txg=27 slot=50178 released_at=3 ring_length=24 holds=true
name=anchor history=H-ring-anchor assertion=A15_inequality violations=0 holds=true
name=anchor history=H-ring-anchor assertion=unit_scan_cross_check_every_scan violations=0 holds=true
```

### Assertion totals (whole rows copied from artifact lines 37494–37522)

```
name=assertion_total assertion=A2_accounting_rows_per_version judged=120018 violated=0
name=assertion_total assertion=A3_unit_scan_on_wide_reads_every_death judged=260039 violated=0
name=assertion_total assertion=A4_death_records_zero judged=1560234 violated=0
name=assertion_total assertion=A5_A10_unlink_publishes judged=54054 violated=0
name=assertion_total assertion=A6_accounting_deaths_at_the_end judged=459 violated=0
name=assertion_total assertion=A7_inode_deaths_at_the_end judged=126 violated=0
name=assertion_total assertion=A11_extent_deaths_at_the_end judged=288 violated=0
name=assertion_total assertion=A12_overwrite_kills_all_but_one_extent_key judged=96 violated=0
name=assertion_total assertion=A13_death_records_zero_on_ring judged=780117 violated=0
name=assertion_total assertion=A16_ring_equals_wide_before_the_first_reuse judged=102 violated=0
name=assertion_total assertion=A18_user_data_outside_cluster_segments judged=77176 violated=0
name=assertion_total assertion=A19_readable_units_equal_written_minus_overwritten judged=120018 violated=0
name=assertion_total assertion=positive_control_matches judged=14243 violated=0
name=assertion_total assertion=scans_equal_publishes judged=306 violated=0
name=assertion_total assertion=A7_deleted_inodes_deaths_at_the_end judged=126 violated=0
name=assertion_total assertion=A12_overwrite_releases_every_old_data_unit judged=96 violated=0
name=assertion_total assertion=A15_reuse_waits_a_ring_length judged=483651 violated=0
name=assertion_total assertion=A18_opened_segment_was_empty judged=9201 violated=0
name=assertion_total assertion=unit_scan_incremental_equals_brute_force judged=306 violated=0
name=assertion_total assertion=kinds_6_7_8_11_12_have_no_units judged=306 violated=0
name=assertion_total assertion=ring_reuses_at_least_once_in_a_long_history judged=46 violated=0
name=assertion_total assertion=K1_tombstone_containers judged=42 violated=0
name=assertion_total assertion=K3_tombstone_death_txg_is_the_intent_txg judged=27027 violated=0
name=assertion_total assertion=K4_containers_after_building judged=84 violated=0
name=assertion_total assertion=K4_container_number_is_its_first_record judged=84 violated=0
name=assertion_total assertion=K4_emptied_container_not_written judged=168 violated=0
name=verdict scope=voiding voiding_assertion_violations=0 nondeterministic_segments=0 A1_K5_eleven_tree_kinds=true void=false
name=verdict scope=clause K_violations=0 F1_clause_may_be_wrong=false
name=verdict scope=stop S_ring_2_anchors_hold=true stop=false
```

a3 holds in all 260,039 G-wide cells, including mixed-random histories in which an extent key dies and is born again (d is judged against the version at scan time). a4 and a13 (arm P r and m always zero) hold in all 2,340,351 cells combined. All 51 histories are identical line by line over two runs (`grep -c 'two_runs_identical=true' <产物>` gives `51`).

### Positive controls (5.5)

Of 18,490 positive-control rows, 14,243 were successfully planted or erased and incremental and expected values all match; 4,247 were “not plantable”; zero mismatches:

```
grep 'name=positive_control' research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["matches"]}' | sort | uniq -c
   4247 not_plantable
  14243 true
```

The 14,243 matching rows by type: PC1 4,881; PC1_tombstoned (arm B identified by tombstone, expected +0) 1,962; PC1_without_its_death_record (arm P death record removed, expected +1) 2,390; PC1_newer_accounting_row (R-S/R-G accounting cell plants a higher-generation row instead) 918; PC2 4,092.

“Not plantable” and untested cells, listed faithfully:

- R-U on G-wide reads every dead key (a3: r = d), leaving no dead key not accepted by the reader to plant for PC1. Those cells are “resurrected” or “no death” and do not rely on the positive control. The same applies to extent under R-S on G-wide.
- In arm B all dead inode and `deleted_inodes` keys are identified by tombstones; the “not identified by a tombstone” case cannot be planted. Only the identified case was run (expected +0; all matched).
- R-G extent selects only dead keys whose object has a record in the R-S reconstructed inode tree (executor revision 2 check 5 line 13); histories without one are not plantable.
- PC2 was never run for `deleted_inodes`: at each history's final scan the queue is empty and no live key can be erased. Those cells are “resurrected” or “r = 0 but misses live”, not “measured zero”, so their judgments do not depend on it.
- Only accounting tree under R-S/R-G is “measured zero”; in those cells PC1 (higher-generation row) and PC2 (erase current accounting leaf, expected 15) both matched in all three arms and geometries.

### Failure, invalidation and stop clauses

| Clause | Triggered? | Basis |
|---|---|---|
| F1 clause may be wrong | Yes: Q1/Q9 inode is “resurrected” (line 37322 etc.). K1/K3/K4/K5 all pass (assertion total K_violations=0). Artifact line 37521 `F1_clause_may_be_wrong=false` calculates only the K half: apparatus omitted the “Q1 inode resurrected” half, so use Q1/Q9 verdict rows | Three-party decision: sentence fails under R-S without retirement records; change wording rather than overturn all of it (`research/prompts/m3-leaf-death-r1-main-verification.md` section two K1). D8 settled item 6 changed to three levels of death; criteria and model unchanged |
| F2 R-S omitted a current-selection element effective from version one | Literal trigger not met: registration 13.2 grep for “retirement record / sweep gate / ancestor table” with “version one” hits one line (`.claude/kb/decisions/18-块里携带什么信息.md` line 263, packed-record unit class-identity segment table, about AAD and ancestor table, not an element effective in “version-one single-head clean history”); `grep -rn '退役记录' crates --include=*.rs \| wc -l` gives `0`. Three-party judgment says triggered: retirement records were registered in the knowledge base and should be written from the first merge (D18 settled item 11 packed record type 3; `remove_record_from_leaf_containers` in `crates/singlefs-core/src/inode_tree.rs` calls emptying a degenerate left-absorbs-right case), but R-S does not use them. Neither literal registered trigger was met; effect is limited to experiment-page wording (r1 section two K1 and r2 section two F2 in `research/prompts/m3-leaf-death-r1-main-verification.md` and `research/prompts/m3-leaf-death-r2-main-verification.md`) | Inode resurrection comes from an emptied container not persisted and nothing written to say its group is gone; Q1/Q9 resurrection was measured under R-S without retirement records. User decided container identity disappearance writes a retirement record (death txg, into retirement tree; r3 verdict section five); E178 has zero container resurrection and false death with retirement records. At the time `crates/` lacked a writer (now has `append_retirement_records_to_the_open_container` in `crates/singlefs-core/src/retirement_tree.rs`) |
| F3 accounting R-S borrows the current-value reading | Triggered as expected: R-U resurrects, R-S measured zero | Accounting “measured zero” only says it does not resurrect under current-value reading, not that scan rebuild cannot accept older-generation rows into the tree |
| F4 arm B tombstone effect chosen by apparatus | Triggered: m > 0 in 45 arm-B cells, m = 0 in same N cells (Q5) | Q4/Q11 cells state “under apparatus-selected tombstone effect, effective at intent” |
| F5 suspicious all-zero | Not triggered: Q1/Q2 have “resurrected” | — |
| F6/F9 flip with geometry | Not triggered: all 52 Q13 rows match at three points | — |
| F7 resurrection in only one family | Triggered only in six Q11_OU rows (only F8 e = OU selected, so F8 alone by definition); no verdict cell across all 51 histories resurrects in just one family | `grep -c 'F7_only_one_family=true' <产物>` gives `6`, all `question=Q11_OU` |
| F10 G-ring resurrection relies only on outside-window records | Not triggered: every cell's r_in peak > 0 | — |
| Invalidation: positive controls, a1–a19, arm P, scan count, two linewise runs, units in codes 6/7/8/12, unreadable rows | None triggered | Assertion totals 37494–37519 all `violated=0`; line 37520 `void=false` |
| Stop S-ring1, S-ring4 | Not triggered | Executor revision 2 checks 3 and 4, before apparatus |
| Stop S-ring2 | Not triggered | Eight anchor rows all `holds=true` |
| Stop S-ring3 | Not triggered: none of 102 G-ring simulations failed to find placement or exceeded occupancy cap | `grep 'name=simulation' <产物> \| grep -v 'does_not_fit_at=none' \| wc -l` gives `0`; likewise zero `exceeded_the_premise_at` rows |
| Stop E172 (Dry Run of Bytes Written by Deletion) export comparison | Not compared as written: export is older than `crates/` (previous executor report section three); main agent must rerun `deletion_write_dump` in the heavy tier | Revision 1 section 12.10 |

### Unit tests and mutations

23 unit tests (`grep -c '^test .* \.\.\. ok$' e177-per-tree-resurrection-without-tombstones-unit-tests.out` gives 23; line 46 of that file `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.18s`). Absolute-value anchors include: eleven kind codes; capacities 233, 815, 135, 477/150, 294/143, root level 2, 211,968; 15 accounting rows; two hand-built split/shrink shapes for code-2 tree; eight H-ring anchors; F8 n_u = 3 publish count 8 and 105 accounting deaths; K1/K4 and 299 deaths for create/delete empty inode n=300; 3 extent deaths in e = OU; R-S selecting the latest version on a hand-built disk; R-G gating; `deleted_inodes` selecting nlink = 0; ring-window 23/24 boundary; verdict threshold movement; trajectory peaks. Differentials: R-U incremental versus brute-force on every scan of two histories, G-ring versus G-wide item by item before first reallocation, all positive controls matching in two histories.

Mutation table `scripts/mutation/tables/e177_per_tree_resurrection_without_tombstones.tsv`: 17 (registered V1–V11 and revision 1 V12–V17, each anchor hit exactly once), 17 caught, zero invalid or not failing; 💥/⚠️/⏱/🧱 all zero. Final lines 20–21 report zero memory caps (4G limit), timeouts, unscheduled, total-cap exclusions and insufficient GPU memory, and restored with baseline still all green (`e177-per-tree-resurrection-without-tombstones-mutations.out`). V7 (tombstone matching drops object birth generation) was expected insensitive at segment-one sample points and caught by the direct tombstone-predicate unit test. V5 (disable R-G gating) was caught by the hand-built-disk test, not registered PC1: under R-G, PC1 plants only keys whose object has an inode-tree record, yielding +1 with gating on or off.

All 17 caught only means asserted locations were mutation-tested: assembly of verdict rows (`verdict_lines`), Q13 three-point pairing and Q5 statistics are covered only by trajectory and threshold unit tests, with no mutations targeting them.

### Fork sheet (C109: which trees require tombstones when a key dies)

| Fork statement | Sufficient for judgment / what is missing | Can remaining measurements reverse it? | Command that computes it in artifact |
|---|---|---|---|
| Reversal observation: “for each tree, how many keys scan rebuild resurrects when no tombstone is written at death” (Q1/Q2/Q9/Q10; Q3 incidental) | Sufficient: each of 11 tree kinds has a status and r peak in three geometries under R-S/R-G (see per-tree readings) | Segment two (administrator rollback) only creates lineage-tree objects and adds rollback deaths; it does not affect these 11 tree kinds' statuses in segment-one histories. Lineage changes from “no object” to a number; other cells cannot flip | `grep 'name=verdict' <产物> \| grep -E 'question=(Q1\|Q2\|Q9\|Q10) '` |
| Reversal observation: “among trees without objects today, does any have nonzero resurrection?” (Q6/Q7/Q12) | Sufficient: code-5 central mapping has objects and resurrects; dirent, reverse index, free space and LRU are not measurable | No: the four unmeasurable ones have no `crates/` writers and segment two does not create objects for them | `grep -E 'question=(Q6\|Q12) ' <产物>`; Q7 is grep, see Q7 readings |
| Sufficiency: “measure resurrection counts for each of 11 tree kinds” (Q1/Q2/Q5/Q8/Q9–Q11/Q13/Q14) | Sufficient: every tree has a status in every verdict cell, no `nothing_judged`, three geometries agree | No | `grep 'name=verdict' <产物> \| grep -c 'status=nothing_judged'` gives `0`; 52 Q13 rows |
| Sufficiency: “state whether each extra t2 tree has measurable objects” (Q6/Q7) | Sufficient: code 5 has objects; four candidates are not measurable and missing elements specified | No; the knowledge-base candidate list ends in “…”, so its incompleteness is itself a known gap | As above |
| t1 candidate half-sentence “version one writes only inode objects” (Q4/Q11) | Sufficient: whole-object tombstones prevent inode/`deleted_inodes` resurrection but miss live records during unlink; extent/allocation/mapping still resurrect; file-shortening deaths not covered (Q11_OU) | Segment two does not change this cell; if K-W5b (when tombstones take effect) is settled differently, F4 cells require rerun | `grep -E 'question=(Q4\|Q11\|Q11_OU) ' <产物>` |

Segment two (F4 administrator rollback) was not run: dispatch prompt specified not to run it; registration section eleven's sufficient-decision stop marks it and Q3 “not run after sufficient judgment”. Q3 (arm N × R-U × G-tight) trajectory rows are in the artifact, computed incidentally in the same simulation, but there is no verdict row.

### What it cannot answer

- Read side is a model: `crates/` has no scan-reconstruction reader (registered section three S17). R-S/R-G implement the knowledge base's settled current-version selection and D21 (Boundary Between Authoritative and Derived States) settled item 9. K-W5b is unsettled; arm B's tombstone effect is chosen by the apparatus (F4).
- Only single-head clean state: no snapshot or clone heads, crash branches, bad images or administrator rollback (segment two not run).
- G-ring placement was handwritten from `crates/` rules, not cross-checked slot by slot against `crates/` (E172 export is older, not compared). Eight anchors agree with `crates/` tests; no Q14 cell relies only on outside-window data, but G-ring's absolute peak (inode 460) depends on placement.
- G-tight disk size is based on “live slots + slots replaced in this publish” (executor revision 2 check 5). The literal registered version (live slots only) fits 50 of 102 runs (`grep 'name=tight_literal_reading' <产物>`), disproving revision 2's inferred claim that all 51 histories fail to fit after the first few publishes. Q8 was not rejudged under that literal version; three-party decision was not to rejudge: the forward leg measured identical Q8 verdict rows under the literal reading on a copy of the apparatus, not the checked-in apparatus (`research/prompts/m3-leaf-death-r1-main-verification.md` section two K4).
- Allocation record tree counts keys by slot number (same slot across both disks is one); under R-S/R-G its two conventions match cell by cell because of the ground-truth reading.
- 51 histories are enumeration points, not the domain; ring length only 24, disks only two 4 GiB disks, code-2 fanout only the product value.
- `deleted_inodes` has no PC2 (queue empty on final scan).
- The mechanisms in “Where resurrection comes from” are inferred from definitions and checked only on named trajectory rows.
- Artifact line 37521 `F1_clause_may_be_wrong=false` omits the Q1 inode half (see F1 above).

### Path and conclusion registration

| # | Path | Calculation | Source location | Shared inputs |
|---|---|---|---|---|
| 1 | R-U incremental count | On unit write, becoming unreadable, release, leaving ring window, key death and tombstone write, increment/decrement each key; maintain r, m, r_in on count changes; compute every scan for 51 histories × 3 geometries × 3 arms | `scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones.rs:2573 UnitScanReading`, `:2630 on_unit_readable` | Per-unit key list from `occurrences_of`; `tombstone_suppresses` predicate; death-time table |
| 2 | R-U brute-force scan | Scan all readable units from scratch each time, filter by arm, collect key set and compare with ground truth; used on each history's last scan, positive controls and every scan in unit tests | `scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones.rs:2980 unit_scan_reading_by_brute_force` | Same three inputs as path 1 |
| 3 | PC2 hand calculation | From erased unit, older version to which its group falls back, and per-key supplier counts in baseline reading, calculate expectations key by key without calling scan function | `scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones.rs:4303 erasure_for_the_hand_calculation`, `:4346 select_current_erasure` | Supplier counts in baseline reading (path 2 or R-S); tombstones and death-time table |

Cross-path assertions: path 1 against 2 is pinned at `:3873 cross_check_the_unit_scan` (last scan of each history, all 306 equal; unit test `:5841 the_incremental_unit_scan_equals_the_brute_force_scan_at_every_scan` compares every scan of two histories). Path 3 against R-S/R-U scanning is pinned by PC2 in `:4446 positive_controls` (all 4,092 rows match). Shared-input caveats: if `occurrences_of` misses an occurrence, paths 1 and 2 both miss it; a3 (r = d on G-wide) and PC1 guard against this. If the tombstone predicate is wrong all three paths are wrong; a direct tombstone-predicate unit test pins it. Death-time table is derived from successive ground-truth versions and shared by paths 1 and 2; a4/a13 (arm P always zero) guard it.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 8 | Evidence preparation | 2026-10-18 unaffected: readings now explicitly cite Q13 verdict field F9_flips_with_geometry=false, and two path/conclusion rows replace “same file” with file path; conclusions and numbers unchanged |
| D18 (Information Carried in a Block) settled item 10 | Evidence preparation | 2026-10-18 unaffected: readings now explicitly cite Q13 verdict field F9_flips_with_geometry=false, and two path/conclusion rows replace “same file” with file path; conclusions and numbers unchanged |
| D18 (Information Carried in a Block) settled item 24 | Evidence preparation | 2026-10-18 unaffected: readings now explicitly cite Q13 verdict field F9_flips_with_geometry=false, and two path/conclusion rows replace “same file” with file path; conclusions and numbers unchanged |
| D8 (Core Index Structure) settled item 6 | Support | 2026-10-18 unaffected: readings now explicitly cite Q13 verdict field F9_flips_with_geometry=false, and two path/conclusion rows replace “same file” with file path; conclusions and numbers unchanged |
| D8 (Core Index Structure) settled item 9 | Unaffected | 2026-10-18 unaffected: readings now explicitly cite Q13 verdict field F9_flips_with_geometry=false, and two path/conclusion rows replace “same file” with file path; conclusions and numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 9 | Unaffected | 2026-10-18 unaffected: readings now explicitly cite Q13 verdict field F9_flips_with_geometry=false, and two path/conclusion rows replace “same file” with file path; conclusions and numbers unchanged |

## Historical Versions

### 2026-10-08: page created, segment one run (sufficient for judgment)

Pre-registration was written before apparatus. Experiment designer wrote revision 1 (main agent chose G-wide not to follow `crates/`, added G-ring as main sample, replaced F2 with F8). Executor wrote revision 2 checklist before apparatus (S-ring1 and S-ring4 not triggered), adding “G-tight peak live slots” before artifact. Segment one completed; fork C109 was sufficient for judgment. Segment two was not run as specified by dispatch prompt.
