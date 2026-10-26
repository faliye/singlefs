## E178 Counts of Leaf Containers and Record Death — run (four segments)

[中文](178-叶容器与记录死亡的计数.md) · **English** · [日本語](178-葉コンテナとレコードの死亡数.ja.md)

<!-- doc-lint:not-numbers X1 X2 X3 X4 H51 F1 F3 F8 F10 F11 F12 K1 K2 K3 K4 K5 K6 S1 S2 S3 S4 S5 S6 V1 V2 V3 V4 V5 V6 V7 V8 M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 FL0 FL1 FL2 FL3 FL4 FL5 FL6 FL7 FL8 FL9 FL10 B1 B2 BN1 BN2 K-2 P-1 -->

```text
Local arm names and notation on this page are written in lowercase (a1, a2, a3, a4 …): the preregistration, apparatus, artifacts and commands write them as the corresponding uppercase, which has the same shape as globally registered identifiers; the kb text switches to lowercase only to keep them apart from those identifiers, so match by uppercase when reading artifacts.
```

Preregistration `research/prompts/e178-preregistration.md` (in section twelve, "amendment 1" and "amendment 3" were written by the executor after the unit tests ran and before the artifacts; "amendment 2" is the two base arms added by the experiment designer, run as segment four). It answers four rows of the fork sheet `m3-leaf-death-forks.md`: X1 (how a death in which the container identity disappears is kept on disk: retirement record B-on, zero-entry version left in the tree B-stub, zero new on-disk data B-tsz; X2–X4 all take it as a premise), X2 (how the death where a file gets shorter is kept on disk: reading Z vs range tombstone T), X3 (from which publish the whole-object tombstone of unlink takes effect: the intent publish E-I vs the last batch E-L), X4 (how the I-9.4 (container number not above the smallest key) criterion is paired with borrowing: n1 raises the separator key, n2 changes identity on borrow). The registration runs in four segments: segment one X4, segment two X2 and the missed-live half of X3, segment three the crash half of X3, segment four the two new candidates for X1.

The apparatus is an independent hand-written counting model `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs`; it shares no code with `crates/` and does not depend on `crates/`. Each of the four segments has its own entry (`segment-one` through `segment-four`), and the artifacts of all four segments were run (see the table in "Replay").

**Status**:

- Segment one done: H51 (the 51 segments replayed segment by segment with the apparatus of E177 (Resurrection Count Without Per-Tree Tombstones)) and F10 (multiple objects with data, borrowing and absorption, 28 segments), under n1 / n2 × leaf-container capacity {233, 16} × merge threshold θ {0, ⌊capacity ÷ 4⌋, ⌊capacity ÷ 2⌋}, ran write paths only, without scanning, plus the full bad-image table of registration 5.6 (44 shapes × two key layouts). The X4 row of the fork sheet is sufficient to decide.
- On every borrow n2 writes one more code 3 and one more code 2 than n1 (the retired container and the retirement-tree root) and one more retirement record, in all four cells with θ > 0; the share of n2's extra writes among all write units of a history is 129–33003 ppm by cell (largest at capacity 16, θ = 8). At θ = 0 the two write streams are identical publish by publish.
- After the third sentence of I-9.4 (container number not above the smallest key) is replaced by "left maximum key < right separator key" (the n1 wording), 8 of the 44 shapes are "red under the 2026-10-08 I-9.4 (container number not above the smallest key) but green under n1": K-2, the three S-family shapes with wrong numbers 2, 9, 10 paired with the correct separator key, BN1 with s = 25, 30, 31, and s = 30 paired with right number 10; after adding I-9.12 (Separator Key Falls Outside Child Range), 7 shapes remain (BN1-s31 is judged red by I-9.12 (Separator Key Falls Outside Child Range)). Of the 8 shapes, BN1-s30 is the correct post-borrow n1 form written in the registration; the other 7 are bad images with a wrong number or a wrongly raised separator key. The two key layouts judge every shape the same. Failure clause FL8 (n1 misses) is hit.
- Not a single invalidation clause was hit, and stop differentials S1–S3 and S6 all matched. In the three cells with θ > 0, part of F10's segments have no borrow at all (V6 only declines to count those segments as results for X4; the family is not invalidated); see "failure, invalidation and stop clauses one by one".
- Amendment 1: on F11 (rollback), the PC-B2 written in the registration cannot measure container m of B-off-lin, so a directly constructed PC-B2-direct was added, and the sensitive sampling point of M3 in section nine is now carried by it; the main agent decided that the original PC-B2 on F11 is not taken as a result, and the sensitivity of Q-B2 is carried by PC-B2-direct.
- Segment two done (H51, F10, F11 × 12 cells = 996 items, 20 readers each scanning once after every publish): over the 48 judged cells (write-path cell × batch) of the main grid (B-on × Z / T × E-I / E-L-w / E-L-r × W ∈ {∞, 23}), container resurrection of base B-on and container false death after rollback are both 0; resurrection and false death (outside midway) of Z and T, and false death from midway shortening under both E-L arms, are all 0; missed live midway through unlink (inode, extent) under E-L-w and E-L-r is 0, while E-I's missed live midway is by definition 1 record and all remaining units on every midway scan. Each extra-write publish of T writes one more code 3 and one more code 2; the share of extra-write publishes by family ranges from 0 (F1) to 766891 ppm (F8, both E-L arms). Of the 26 rows of stability comparison across the four knobs (W, θ, capacity, batch), not one flipped.
- The invalidation and failure clauses of segment two are listed one by one in "failure, invalidation and stop clauses one by one": PC-B1 does not hold on three n2 segments of F11 (borrow gap θ = 4, θ = 8, gap where a moved object gets shorter θ = 8), and by the letter of V2 those three segments are invalidated (the cause is pinned by a unit test: an identity born inside the gap and gone again inside the gap cannot be read by any reader at the end, and PC-B1's expected count did not subtract it); PC-M by the letter of the registration (per reader extent m + 1, erased container inode m + record count, container m + 1) does not hold on 12 data-unit segments and 612 leaf-container segments (under W = ∞ an older readable version remains below the erased one), and the apparatus judges more loosely than the registration (≤), handed to the main agent; V8 by its letter hits the times when E-I readers read empty on midway scans in F8 and F3; F11 has 4 segments rolled back too far and 24 segments where the gap event did not occur. With these removed, not one verdict of the 48 judged cells changes (all 0; removing segments cannot turn 0 positive).
- Segment three done (12 prefixes of F12, 5448 crash points): at the crash points the missed live (inode m, extent m) of E-L-w and E-L-r is 0, and E-I by definition misses 1 record and the remaining units; for reading o2 (objects that have a record in the rebuilt view while their intent has already been published), E-I has 0 at every crash point and E-L-w, E-L-r have exactly 1 at every crash point, so FL6 is hit literally (this item is a direct consequence of the definitions of the two E-L arms; see "failure, invalidation and stop clauses one by one"); the resurrection peaks of the three namespaces in the continuation after recovery are all 0; the two W settings judge every row the same.
- Segment four done (registration amendment 2, amendment 3; 997 items: H51, F10, F11 × 12 cells, plus 1 constructed item PC-B2-direct): resurrection and false death of B-stub (zero-entry version left in the tree, paired only with n1, question one n2-none) and B-tsz (zero new on-disk data, supply rule P-C, paired only with E-I) are 0 on all judged cells, side by side with B-on in the same cells without a single cell's difference. B-stub's 12 readers × 12 n1 judged cells (Q-Bs-1, Q-Bs-2, including the after-rollback tier of F11) and B-tsz's 4 readers × 24 judged cells (the inode r and extent r differences of Q-Bt-1, Q-Bt-2) are all 0; B-tsz's container r is 1–166 by cell (a consequence of the definition: the last version of a dead identity is read as is, pinned by a12, equal to B-off-ret at every scan).
- The positive controls of both new bases hold: PC-Bs2 (on the rollback construction B-stub's container m and inode m are 0, while B-stub-lin-off with the lineage term turned off gives 1, 1), PC-Bt3 (B-tsz's inode m 0, and 1 when tombstones are not filtered by lineage), PC-Bs1 962 items, PC-Bt1 1724 items, PC-Bt2-mid 224 items (220 of them with hand-counted value > 0), the two paths agree and the expectations hold. The original PC-Bt2 (the final scan) has hand-counted value 0 on all 224 items, so it cannot measure anything, as amendment 3 states.
- With the base changed, the verdicts of X2 and X3 do not change: in Q-Bs-4 (B-stub) 384 rows and Q-Bt-4 (B-tsz) 192 rows, not one verdict of the five sub-questions of Z and T changes between the new base and B-on, and the FL-Bx list is empty; under B-stub, missed live midway through unlink (Q-Bs-3) of E-L-w and E-L-r is 0, E-I by definition misses 1 record and the remaining units (peaks 1, 300), and the same holds for B-tsz (Q-Bt-3).
- B-stub costs (Q-Bs-5, Q-Bs-6): final stub count at most 56 (F1, F10, capacity 16), peak total leaves at most 66 (F3, capacity 16), highest final stub share 1000000 ppm (F8, whose only leaf is emptied by deletion and becomes a stub); versions that carry a zero-entry version and hit I-9.13 (type-2 container nonempty) make up 1056–670710 ppm of all publications (by family, capacity, θ, batch); no version has more total leaves than the 135 the root can hold. How many units the B-stub write path writes per emptying and per absorption was not registered as a quantity, and the artifact does not contain it.
- Invalidation and failure clauses: pairing (V-Bs1), anchors (a10–a12), stub count read (V-Bs3), and B-tsz vs B-off-ret (V-Bt1) were not hit; V-Bt2: the 43176 trajectory rows of B-on and B-off-ret in segment four are identical line by line to the rows of the same readers in segment two. FL-Bs1, FL-Bt1, FL-B3, FL-Bs2, FL-Bx were not hit. Hit literally, in the same form as in segment two, are V8 (E-I readers read empty midway through unlink in F3 and F8, 2448 trajectories on the new bases) and PC-M by the letter of the registration (1128 items); both touch only report-only cells, see "failure, invalidation and stop clauses one by one".

Deterministic model: every work item runs twice and is compared item by item, and changing the thread count leaves the artifact unchanged; this shows there is no hidden state, not statistical stability. The strength of the evidence rests on anchors that pin absolute values, positive controls, stop differentials and the mutation table. Segment one rerun on the changed apparatus is byte-identical to the old artifact; the B-on and B-off-ret trajectories of segment four are identical line by line to the segment-two artifact (V-Bt2); segment four additionally sampled 20 items (`--shard 0/50 --no-reuse`, all n1 items), recomputed them in a separate item directory, and they are byte-identical to the item files used to assemble the artifact. The segment-four line of `replay.sh` was not rerun (see "What it cannot answer").

### What was done

- **Write path**: new-pool file creation (txg 3, instance 1 of the mkfs process), the writable mount's row write and warm-up (instance 2), then three kinds of changes through the session (write a file = replace the object's whole content with d new units, create an empty inode, unlink = drain the empty queue + intent + all batches). New leaf-container numbers land only in the rightmost leaf, which splits at the end when full (right-half number = number of the record that triggered it, birth generation = this txg); after deleting one record, merge by M(θ): an emptied leaf is not written to disk and all leaves to its right shift left and are fully rewritten; when |C| < θ and a right neighbour exists, absorb it (|C| + |R| ≤ capacity) or borrow ⌈(|R| − |C|) ÷ 2⌉ records, and if C is the rightmost leaf, let the left neighbour absorb it. On a borrow, n1 keeps the right leaf's identity and raises the separator key to the right leaf's new smallest key; n2 gives the right leaf a new identity (container number = smallest key after the borrow, birth generation = this txg) and writes a retirement record for the old identity. The publish in which an identity disappears (emptied, absorbed, n2 borrow) appends one 34-byte retirement record; a retirement-tree container holds 959 records, and the publish with a new record rewrites the open container and the root. The five tombstone write streams (from Z / T × E-I / E-L-w / E-L-r) have one tombstone tree each; segment one only counts the units they write.
- **Segment-one quantities**: each work item (one history × capacity × θ) runs the write path once for n1 and once for n2, paired publish by publish: on a borrowing publish, n2 − n1 of code 3, code 2 and retirement-record count, and every other publish must be identical unit by unit; every version is judged by I-9.13 (type-2 container nonempty), the 2026-10-08 I-9.4 (container number not above the smallest key), n1's I-9.4 (container number not above the smallest key), and I-9.12 (Separator Key Falls Outside Child Range), these four judgements being the model self-check V7 of section eleven; counts of borrows, absorptions, emptyings and the two counts "a moved object later gets shorter / is unlinked" (V6).
- **Full table of 5.6**: K (the six bad-image shapes currently in `crates/`, copied test by test), S (number given wrongly at a split, 22 shapes), BN1 (after an n1 borrow, 9 shapes), BN2 (after an n2 borrow, 4 shapes), DV (wrong direction, 3 shapes); every shape is judged by the 2026-10-08 I-9.4 (container number not above the smallest key), n1's I-9.4 (container number not above the smallest key), I-9.12 (Separator Key Falls Outside Child Range) and routing (each key takes the last entry whose separator key is ≤ it; is the leaf it lands on the one holding it); the second key layout multiplies keys by 100 and takes the midpoint for numbers and separator keys that fall between two keys.
- **Stop differentials** (section eleven of the registration; read only the `crates/` source and test assertions, write the same judgements in the apparatus and compare item by item; `crates/` was not compiled or run): S1 split at the end (`crates/singlefs-core/src/inode_tree.rs:181-283`), S2 delete one record (`:286-318`), S3 the steps of unlink (`crates/singlefs-core/src/unlink.rs:118-130`), S6 the judgements K-0 to K-5 (`crates/singlefs-checker/src/walk.rs:2768-2789`, `:3067`, `:3079`, `:3245-3266` and the assertions at `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:936`, `:947`, `:962`). S4 and S5 are written too and are reported in segment two.
- Where the registration did not pin things down, the apparatus took one reading, written as items 1–9 of the apparatus file header (the container namespace counts only type 2; X2's liveness judgement uses the tree before the whole-object tombstone is applied; T's range rule only recognizes records with range start > 0; the rollback publish does not rewrite the open containers of the tombstone tree and the retirement tree; every user change first drains the empty queue; F11's "keep deleting in ascending order" = delete the live object with the smallest number; F12's re-creation of 5 empty inodes is one publish, and the P-m prefix does not apply the overwrite rule; the measurement basis of "all write units" in Q-X4-1; the planted "largest write order"). Segment one only uses item 8.
- Wall clock: segment one's 474 work items on 6 threads, including compilation, about 29 seconds (the real line of `time`; not in the artifact).
- **Segment two**: every work item runs the write path twice (the first run fixes the positive-control scans, the two runs are compared scan by scan), and in the second run 20 readers each read once after every publish (registration 5.4: candidates → newest selection → base → rebuild the inode tree and whole-object tombstones → data units → compare with the truth); the main path selects the newest by the group index, and the final scan and all positive controls are additionally compared against the full-scan reference path. Positive controls PC-B1, PC-B2 (F11), PC-B2-direct, PC-Z, PC-T, PC-G0, PC-EL, PC-ELr, PC-M. Verdict lines merge the trajectories of H51, F10, F11 by "write-path cell × batch" (`segment_two_verdict_lines`), and the after-rollback tier merges only F11. Started by the main agent, `E178_THREADS=5`, memory wrapper 8G, about 21 minutes (difference of the modification times of two files in the draft directory; not in the artifact).
- **Segment three**: each F12 prefix is simulated only once, and a crash branches off at the intent version and at every batch that is not the last: the next step's units are all written to disk, the root is not, and at the moment of the crash 20 readers each read once (the two newest-selection paths compared), then recovery with a changed instance generation, continuation until the queue is empty, creation of 5 empty inodes, and a scan after every publish. Positive controls PC-O (planting and removing the tombstone, two items) and PC-M. Started by the main agent, `E178_THREADS=5`, 8G, about 5 minutes.
- **Segment four**: every work item (H51, F10, F11 × 12 cells, plus 1 constructed item PC-B2-direct, 997 items in all) runs the original write path twice with 18 readers (12 B-on, 4 B-tsz, 2 B-off-ret); n1 cells additionally run the B-stub write path twice with 12 readers, paired version by version with the original write path (V-Bs1), and a "difference" only counts when the pairing holds (the new arm minus B-on with the same (X2, X3, W)). B-stub write path: in the publish where an identity disappears (emptied, absorbed) it writes one zero-entry version in place, which stays in leaf order from then on and is never rewritten; the leaves to its right do not shift left, and no retirement record is written; a stub takes no new numbers and is not a merge neighbour (reading 10 of the apparatus file header). B-tsz readers drop no version, supply among several records of the same number by change count (P-C, ties broken by write order), and apply the whole-object tombstone as E-I. Each item, once computed, lands atomically in the item directory, and restarting the same command only fills in items not yet on disk. The main agent started the launch script written by the executor via `detached-run.sh`: this host ran `--shard 0/2` on 14 threads (499 judged fresh), the second host's shard was refused by the memory wrapper (exit 252, it needed 50G while the slice's total cap is 40.2 GiB; the command did not run), this host computed the other half (498 judged fresh, 499 reused), and with all 997 items present the artifact was assembled (0 judged fresh, 997 reused); launch log `~/.local/state/singlefs/detached-runs/e178-segment-four/log`. From the first to the last item file the modification times span about 15 minutes (difference from `ls -l`; not in the artifact).

### Replay

```
# repository root; the model is in scripts/experiment-apparatus/ (does not depend on crates/), the thread count reads E178_THREADS (default 6), which affects only wall clock, not output
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
E178_THREADS=6 bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 6 cargo run -q --release --bin e178-leaf-and-record-death-counts -- segment-one
E178_THREADS=5 bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 5 cargo run -q --release --bin e178-leaf-and-record-death-counts -- segment-two
E178_THREADS=5 bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 5 cargo run -q --release --bin e178-leaf-and-record-death-counts -- segment-three
cd ../..
# segment four (sharding, resume, multi-host): the item library defaults to ~/.local/share/singlefs/experiments/e178/segment-four/ (the environment variable E178_SEGMENT_FOUR_LIBRARY can change it); the old variable E178_SEGMENT_FOUR_ITEMS is deprecated and exits 2 if set; reading and writing the item library must go through scripts/experiment-apparatus/device-code-key.py run, which gives --code-key-file, and starting the apparatus directly only accepts --no-reuse; computes one shard only and only lands item files
E178_THREADS=6 bash scripts/hosts/run-with-memory-cap.sh 8G python3 scripts/experiment-apparatus/device-code-key.py run . --binary e178-leaf-and-record-death-counts --entry e7_index_bench::segment_four_item_lines --when-refused=--no-reuse -- segment-four --shard 0/2
# segment four: compute the missing items and assemble the artifact once all 997 items are present; starting the apparatus directly with --no-reuse judges every item fresh and neither reads nor writes the item library
E178_THREADS=6 bash scripts/hosts/run-with-memory-cap.sh 8G python3 scripts/experiment-apparatus/device-code-key.py run . --binary e178-leaf-and-record-death-counts --entry e7_index_bench::segment_four_item_lines --when-refused=--no-reuse -- segment-four
# the registered replay (four lines of scripts/experiment-apparatus/replay.sh: segment-one, segment-two, segment-three, segment-four --no-reuse)
bash scripts/experiment-apparatus/replay.sh E178
# 44 unit tests and the mutation table of 22 entries
cd scripts/experiment-apparatus
bash ../hosts/run-with-memory-cap.sh 8G bash ../hosts/capped.sh 6 cargo test --release --bin e178-leaf-and-record-death-counts
MUTATE_JOBS=4 bash ../mutation/mutate.sh e178-leaf-and-record-death-counts e178_leaf_and_record_death_counts.rs ../mutation/tables/e178_leaf_and_record_death_counts.tsv
```

The whole launch procedure of segment four (this host and the second host run one shard each, fetch back the item files, compute the missing items, count up to the full 997 items and then assemble) is in `run-segment-four.sh` in the executor's draft directory (`detached-run.sh long-run-check` judged its long-run declaration complete), started by the main agent; artifact `research/results/e178-leaf-and-record-death-counts-segment-four.out`.

| File | What it is |
|---|---|
| `research/results/e178-leaf-and-record-death-counts-segment-one.out` | Segment one: config, four rows of constant anchors, four rows of stop differentials, 88 rows of the full 5.6 table, 948 rows of write paths, 474 rows of borrow pairing, assertion totals and anchors, two rows of positive controls, 98 rows of verdicts (1626 lines, sha256 `9644a139c0c01402a18918f232e6fbafcfc6caa96d68578441e66ccc30f3a8ea`) |
| `e178-leaf-and-record-death-counts-unit-tests.out` | Unit-test output at segment one (32 tests, including the compile lines), superseded by `…-unit-tests-2026-10-08.out` after the apparatus was changed |
| `e178-leaf-and-record-death-counts-mutations.out` | Output of `mutate.sh` over the whole table (15 entries) at segment one (the first line of stderr comes first), superseded by `…-mutations-2026-10-08.out` |
| `e178-leaf-and-record-death-counts-unit-tests-2026-10-08.out` | Unit-test output after segment four was added (44 tests, including the compile lines; sha256 `cc3519224f26f975e9a36f37492f7bb87f88fb2710e599754a017b038e280368`) |
| `e178-leaf-and-record-death-counts-mutations-2026-10-08.out` | stdout of `mutate.sh` over the whole table (22 entries) after segment four was added (sha256 `60c198835f92e6bb0314c5aa16e67803ff90055070edc2d0d2ccf415e9e1b536`) |
| `research/results/e178-leaf-and-record-death-counts-segment-two.out` | Segment two (73284 lines, last line `E7RESULT name=done emitted=73284`, sha256 `e73d4aa979f0f683ae7a46f35009926631365cd8be1f3f04640c308a5e052bab`) |
| `research/results/e178-leaf-and-record-death-counts-segment-three.out` | Segment three (4335 lines, last line `E7RESULT name=done emitted=4335`, sha256 `30565bfcba92fce5ba3be3a6bae3385eea82a89da97968caa7c284bbf7978fc3`) |
| `research/results/e178-leaf-and-record-death-counts-segment-four.out` | Segment four (140179 lines, last line `E7RESULT name=done emitted=140179`, sha256 `17c292f968a235134376444fc9aa20833bf55c68245e7c43c92b004dc954668e`; apparatus source file sha256 `93193bd5929e7867b7d18ea5c8b0d6af9e32ceffed5c866f073ba8e5c0d1dfa6`, the same value as in the key on the first line of each item file) |

Line counts of each row type in the artifact (`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-one.out | sort | uniq -c`):

```
      6 name=anchor
      4 name=assertion
    474 name=borrow_cost
      1 name=config
      1 name=done
      2 name=positive_control
     88 name=shape
      4 name=stop_check
     98 name=verdict
    948 name=write_path
```

948 = 79 segments × 12 write-path cells. Counting grouped by family, N, capacity and θ, each group has as many rows as that family has segments (F1 14, F10 28, F8 32, F3 5), and none of the 12 cells is missing:

```
grep 'name=write_path' research/results/e178-leaf-and-record-death-counts-segment-one.out | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["family"], v["borrow_arm"], v["capacity"], v["theta"]}' | sort | uniq -c | awk '{print $1}' | sort | uniq -c
     12 14
     12 28
     12 32
     12 5
```

### Verdicts (full lines copied from `research/results/e178-leaf-and-record-death-counts-segment-one.out`)

Q-X4-1, Q-X4-2 (the two per-cell rows among lines 1528–1569; in the two cells with θ = 0 the borrow count is 0 and the two write streams are identical publish by publish):

```
E7RESULT name=verdict question=Q-X4-1 capacity=233 theta=0 histories=79 borrow_publishes=0 extra_code3_per_borrow_count=0 extra_code3_per_borrow_smallest=none extra_code3_per_borrow_largest=none extra_code3_per_borrow_total=0 extra_code2_per_borrow_count=0 extra_code2_per_borrow_smallest=none extra_code2_per_borrow_largest=none extra_code2_per_borrow_total=0 n2_extra_share_ppm_over_histories_with_borrows_count=0 n2_extra_share_ppm_over_histories_with_borrows_smallest=none n2_extra_share_ppm_over_histories_with_borrows_largest=none n2_extra_share_ppm_over_histories_with_borrows_total=0 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=233 theta=0 n2_retirements_per_borrow_count=0 n2_retirements_per_borrow_smallest=none n2_retirements_per_borrow_largest=none n2_retirements_per_borrow_total=0 n1_retirements_per_borrow_count=0 n1_retirements_per_borrow_smallest=none n1_retirements_per_borrow_largest=none n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=233 theta=58 histories=79 borrow_publishes=12 extra_code3_per_borrow_count=12 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=12 extra_code2_per_borrow_count=12 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=12 n2_extra_share_ppm_over_histories_with_borrows_count=6 n2_extra_share_ppm_over_histories_with_borrows_smallest=147 n2_extra_share_ppm_over_histories_with_borrows_largest=1100 n2_extra_share_ppm_over_histories_with_borrows_total=2805 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=233 theta=58 n2_retirements_per_borrow_count=12 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=12 n1_retirements_per_borrow_count=12 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=233 theta=116 histories=79 borrow_publishes=102 extra_code3_per_borrow_count=102 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=102 extra_code2_per_borrow_count=102 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=102 n2_extra_share_ppm_over_histories_with_borrows_count=21 n2_extra_share_ppm_over_histories_with_borrows_smallest=146 n2_extra_share_ppm_over_histories_with_borrows_largest=5464 n2_extra_share_ppm_over_histories_with_borrows_total=23828 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=233 theta=116 n2_retirements_per_borrow_count=102 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=102 n1_retirements_per_borrow_count=102 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=16 theta=0 histories=79 borrow_publishes=0 extra_code3_per_borrow_count=0 extra_code3_per_borrow_smallest=none extra_code3_per_borrow_largest=none extra_code3_per_borrow_total=0 extra_code2_per_borrow_count=0 extra_code2_per_borrow_smallest=none extra_code2_per_borrow_largest=none extra_code2_per_borrow_total=0 n2_extra_share_ppm_over_histories_with_borrows_count=0 n2_extra_share_ppm_over_histories_with_borrows_smallest=none n2_extra_share_ppm_over_histories_with_borrows_largest=none n2_extra_share_ppm_over_histories_with_borrows_total=0 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=16 theta=0 n2_retirements_per_borrow_count=0 n2_retirements_per_borrow_smallest=none n2_retirements_per_borrow_largest=none n2_retirements_per_borrow_total=0 n1_retirements_per_borrow_count=0 n1_retirements_per_borrow_smallest=none n1_retirements_per_borrow_largest=none n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=16 theta=4 histories=79 borrow_publishes=222 extra_code3_per_borrow_count=222 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=222 extra_code2_per_borrow_count=222 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=222 n2_extra_share_ppm_over_histories_with_borrows_count=12 n2_extra_share_ppm_over_histories_with_borrows_smallest=137 n2_extra_share_ppm_over_histories_with_borrows_largest=16448 n2_extra_share_ppm_over_histories_with_borrows_total=64634 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=16 theta=4 n2_retirements_per_borrow_count=222 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=222 n1_retirements_per_borrow_count=222 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
E7RESULT name=verdict question=Q-X4-1 capacity=16 theta=8 histories=79 borrow_publishes=1505 extra_code3_per_borrow_count=1505 extra_code3_per_borrow_smallest=1 extra_code3_per_borrow_largest=1 extra_code3_per_borrow_total=1505 extra_code2_per_borrow_count=1505 extra_code2_per_borrow_smallest=1 extra_code2_per_borrow_largest=1 extra_code2_per_borrow_total=1505 n2_extra_share_ppm_over_histories_with_borrows_count=45 n2_extra_share_ppm_over_histories_with_borrows_smallest=129 n2_extra_share_ppm_over_histories_with_borrows_largest=33003 n2_extra_share_ppm_over_histories_with_borrows_total=466033 differences_outside_borrows=0 borrow_kinds_disagree=0 a4_a6_definition_holds=true a5_zero_threshold_streams_identical=true judgement=reported
E7RESULT name=verdict question=Q-X4-2 capacity=16 theta=8 n2_retirements_per_borrow_count=1505 n2_retirements_per_borrow_smallest=1 n2_retirements_per_borrow_largest=1 n2_retirements_per_borrow_total=1505 n1_retirements_per_borrow_count=1505 n1_retirements_per_borrow_smallest=0 n1_retirements_per_borrow_largest=0 n1_retirements_per_borrow_total=0 definition_holds=true judgement=reported
```

### Readings: extra writes and retirement records per borrow (Q-X4-1, Q-X4-2, Q-X4-3)

| Capacity | θ | Borrowing publishes (total over 79 segments) | Per borrow, n2 − n1 of code 3 / code 2 | Retirement records per borrow n2 / n1 | n2's extra writes among all write units (segments with borrows only, ppm) |
|---|---|---|---|---|---|
| 233 | 0 | 0 | — | — | — |
| 233 | 58 | 12 | 1 / 1 (all 12 times) | 1 / 0 | 147–1100 (6 segments) |
| 233 | 116 | 102 | 1 / 1 (all 102 times) | 1 / 0 | 146–5464 (21 segments) |
| 16 | 0 | 0 | — | — | — |
| 16 | 4 | 222 | 1 / 1 (all 222 times) | 1 / 0 | 137–16448 (12 segments) |
| 16 | 8 | 1505 | 1 / 1 (all 1505 times) | 1 / 0 | 129–33003 (45 segments) |

The numbers in this table are taken from the twelve rows copied in the verdicts section. Outside borrows, n2 and n1 publishes are identical unit by unit (`differences_outside_borrows=0`), and the two arms' counts of borrows, absorptions and emptyings are identical segment by segment (`merge_counts_differ_between_n1_n2=0` on every row of Q-X4-3). "1 + 1 extra writes and 1 retirement record per borrow" is a consequence of the arm definitions (registration section four, item 5), and the a6 one-borrow small history in section seven pins the absolute value (the anchors section); it is not a threshold, and these cells only report numbers. The denominator of the share follows reading 8: data units + leaf containers + inode-tree root + retirement containers + retirement-tree root + the tombstone containers and tombstone-tree root of the 2026-10-08 tombstone write stream.

The six F10 rows of Q-X4-3 (full lines):

```
E7RESULT name=verdict question=Q-X4-3 capacity=233 theta=0 family=F10 segments=28 borrows=0 absorbs=0 drops=56 moved_then_shortened=0 moved_then_unlinked=0 publishes=68181 n2_retirement_records=56 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=0 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=233 theta=58 family=F10 segments=28 borrows=8 absorbs=56 drops=0 moved_then_shortened=718 moved_then_unlinked=4428 publishes=68112 n2_retirement_records=64 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=24 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=233 theta=116 family=F10 segments=28 borrows=68 absorbs=56 drops=0 moved_then_shortened=990 moved_then_unlinked=5712 publishes=68099 n2_retirement_records=124 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=14 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=16 theta=0 family=F10 segments=28 borrows=0 absorbs=0 drops=1036 moved_then_shortened=0 moved_then_unlinked=0 publishes=68181 n2_retirement_records=1036 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=0 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=16 theta=4 family=F10 segments=28 borrows=148 absorbs=1036 drops=0 moved_then_shortened=894 moved_then_unlinked=5904 publishes=68185 n2_retirement_records=1184 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=20 judgement=reported
E7RESULT name=verdict question=Q-X4-3 capacity=16 theta=8 family=F10 segments=28 borrows=1000 absorbs=1036 drops=0 moved_then_shortened=1170 moved_then_unlinked=8026 publishes=68189 n2_retirement_records=2036 merge_counts_differ_between_n1_n2=0 does_not_fit_segments=0 v6_segments_without_coverage=0 judgement=reported
```

V7 (model write-path self-check, full lines; n1 versions being red under the 2026-10-08 I-9.4 (container number not above the smallest key) is a consequence of the definition, so only version counts are reported):

```
E7RESULT name=verdict question=V7 capacity=233 theta=0 borrow_arm=N1 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=0 borrow_arm=N2 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=58 borrow_arm=N1 versions=88115 i913_red=0 today_i94_red=2956 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=58 borrow_arm=N2 versions=88115 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=116 borrow_arm=N1 versions=88102 i913_red=0 today_i94_red=9190 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=233 theta=116 borrow_arm=N2 versions=88102 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=0 borrow_arm=N1 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=0 borrow_arm=N2 versions=88184 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=4 borrow_arm=N1 versions=88188 i913_red=0 today_i94_red=5870 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=4 borrow_arm=N2 versions=88188 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=8 borrow_arm=N1 versions=88192 i913_red=0 today_i94_red=36890 n1_i94_red=0 i912_red=0 required_green_holds=true
E7RESULT name=verdict question=V7 capacity=16 theta=8 borrow_arm=N2 versions=88192 i913_red=0 today_i94_red=0 n1_i94_red=0 i912_red=0 required_green_holds=true
```

### Readings: what n1 does and does not catch on the existing and newly added bad images (Q-X4-4 to Q-X4-7)

The full table of 88 rows is at artifact lines 10–97 (`grep 'name=shape ' research/results/e178-leaf-and-record-death-counts-segment-one.out`), one row per shape giving the red or green of the three predicates and the routing. Summary (full lines; three rows each for the dense layout `dense` and the sparse layout `sparse`, plus one G-K row):

```
E7RESULT name=verdict question=Q-X4-4 layout=dense family=all shapes=44 today_red_n1_green=8 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s31,BN1-s30-m10 n1_misses_shapes=true
E7RESULT name=verdict question=Q-X4-5 layout=dense family=all shapes=44 today_or_i912_red_n1_and_i912_green=7 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s30-m10 i912_does_not_cover_the_misses=true
E7RESULT name=verdict question=Q-X4-6 layout=dense family=all misses_with_wrong_routing=1 names=BN1-s31 n1_misses_hurt_lookup=true
E7RESULT name=verdict question=Q-X4-4 layout=sparse family=all shapes=44 today_red_n1_green=8 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s31,BN1-s30-m10 n1_misses_shapes=true
E7RESULT name=verdict question=Q-X4-5 layout=sparse family=all shapes=44 today_or_i912_red_n1_and_i912_green=7 names=K-2,S-m2-separator_twenty,S-m9-separator_twenty,S-m10-separator_twenty,BN1-s25,BN1-s30,BN1-s30-m10 i912_does_not_cover_the_misses=true
E7RESULT name=verdict question=Q-X4-6 layout=sparse family=all misses_with_wrong_routing=1 names=BN1-s31 n1_misses_hurt_lookup=true
E7RESULT name=verdict question=G-K shapes=44 verdicts_differ_between_layouts=0 names=none stable=true
```

The 8 shapes that are "red under the 2026-10-08 I-9.4 (container number not above the smallest key) but green under n1's I-9.4 (container number not above the smallest key)", and three legal controls (full lines, dense layout):

```
E7RESULT name=shape family=K shape=K-2 layout=dense leaves=1:1/10,8:20/30 separators=1,15 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m2-separator_twenty layout=dense leaves=1:1/10,2:20/30 separators=1,20 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m9-separator_twenty layout=dense leaves=1:1/10,9:20/30 separators=1,20 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m10-separator_twenty layout=dense leaves=1:1/10,10:20/30 separators=1,20 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=S shape=S-m20-separator_twenty layout=dense leaves=1:1/10,20:20/30 separators=1,20 today_i94=green n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN1 shape=BN1-s25 layout=dense leaves=1:1/10/20,20:30 separators=1,25 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN1 shape=BN1-s30 layout=dense leaves=1:1/10/20,20:30 separators=1,30 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN1 shape=BN1-s31 layout=dense leaves=1:1/10/20,20:30 separators=1,31 today_i94=red n1_i94=green i912=red routing=wrong
E7RESULT name=shape family=BN1 shape=BN1-s30-m10 layout=dense leaves=1:1/10/20,10:30 separators=1,30 today_i94=red n1_i94=green i912=green routing=right
E7RESULT name=shape family=BN2 shape=BN2-m30 layout=dense leaves=1:1/10/20,30:30 separators=1,30 today_i94=green n1_i94=green i912=green routing=right
```

- K-2 (right number 8, separator key 15): on the day of the run (2026-10-08) only the third sentence of I-9.4 (container number not above the smallest key) speaks up (the comment of the "second leaf's container number 8" test case in `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`); n1 replaces the third sentence with a comparison against the separator key, and the shape turns green. The cell predicted by registration section four, item 4 ("this shape is not red after n1") matched; that match does not count as independent confirmation (the registration's own words).
- S family wrong numbers 2, 9, 10 paired with the correct separator key 20: the number is given wrongly at the split but the separator key is right; n1's four sentences and I-9.12 (Separator Key Falls Outside Child Range) are all green, and routing is right too. When the same wrong number is paired with "separator key = number" (the way `crates/singlefs-core/src/inode_tree.rs` took the separator key straight from the container number when E178 (Counts of Leaf Containers and Record Death) ran; the current form is the `separator_key_in_the_parent_entry` field, function `separator_key`), n1's I-9.4 (container number not above the smallest key) and I-9.12 (Separator Key Falls Outside Child Range) both judge red and routing is wrong (the three rows such as `shape=S-m2-separator_equals_number` in the artifact).
- BN1-s25 (separator key raised not far enough), BN1-s30-m10 (right number 10): n1 and I-9.12 (Separator Key Falls Outside Child Range) are both green, routing is right. BN1-s31 (raised too far) is green under n1, red under I-9.12 (Separator Key Falls Outside Child Range), routing wrong, and is the only shape in Q-X4-6.
- BN1-s30 is the correct post-borrow n1 form written in registration 5.6 ("30 (correct)"); its being red under the 2026-10-08 I-9.4 (container number not above the smallest key) is a consequence of n1's definition, and it was counted into the lists of Q-X4-4 and Q-X4-5 by the letter of the criterion; without it, Q-X4-4 still has 7 shapes and Q-X4-5 still 6, and FL8 is hit all the same.
- None of the four BN2 shapes and three DV shapes is "red under the 2026-10-08 I-9.4 (container number not above the smallest key) but green under n1"; K-1, K-3, K-4, K-5 are red together under the 2026-10-08 I-9.4 (container number not above the smallest key) and under n1.
- Among the per-family verdict lines (artifact lines 1578–1606), the ones reading false are the cells those families did not hit: families BN2 and DV have `n1_misses_shapes=false` (Q-X4-4), `i912_does_not_cover_the_misses=false` (Q-X4-5), `n1_misses_hurt_lookup=false` (Q-X4-6); families K and S have `n1_misses_hurt_lookup=false` (Q-X4-6) on both layouts. Families BN2 and DV have no shape that n1 misses; families K and S have shapes n1 misses, but the misses do not hurt routing.

### Anchors, stop differentials and positive controls (full lines copied from artifact lines 2–9 and 1520–1527)

```
E7RESULT name=anchor id=K1 inode_leaf_records=233 expected=233 holds=true
E7RESULT name=anchor id=K2 tombstone_container_records=815 expected=815 holds=true
E7RESULT name=anchor id=K3 root_node_leaf_containers=135 expected=135 holds=true
E7RESULT name=anchor id=A2 retirement_record_bytes=34 retirement_container_records=959 expected=34,959 holds=true
E7RESULT name=stop_check id=S1 holds=true detail=capacity_16=true,capacity_233=true
E7RESULT name=stop_check id=S2 holds=true detail=partial=Some((UnitsWritten { data_units: 0, inode_leaf_containers: 1, inode_roots: 1, retirement_containers: 0, retirement_roots: 0, retirement_records: 0, tombstone_containers: [0, 1, 0, 0, 1], tombstone_roots: [0, 1, 0, 0, 1] }, NoMerge)) emptied=Some((UnitsWritten { data_units: 0, inode_leaf_containers: 1, inode_roots: 1, retirement_containers: 1, retirement_roots: 1, retirement_records: 1, tombstone_containers: [0, 1, 0, 0, 1], tombstone_roots: [0, 1, 0, 0, 1] }, EmptiedAndDropped)) leaves_after_emptying=[1, 33] last_record=Some((UnitsWritten { data_units: 0, inode_leaf_containers: 0, inode_roots: 1, retirement_containers: 1, retirement_roots: 1, retirement_records: 1, tombstone_containers: [0, 1, 0, 0, 1], tombstone_roots: [0, 1, 0, 0, 1] }, EmptiedAndDropped))
E7RESULT name=stop_check id=S3 holds=true detail=steps_hold=true size_at_the_intent=Some(9790200) after_the_first_batch=Some(InodeRecord { inode: 1, object_birth: 3, nlink: 0, size_bytes: 5090904 })
E7RESULT name=stop_check id=S6 holds=true detail=K-0=true,K-1=true,K-2=true,K-3=true,K-4=true,K-5=true
E7RESULT name=assertion assertion=A1_history_publishes judged=612 violated=0
E7RESULT name=assertion assertion=A1_history_refused_steps judged=612 violated=0
E7RESULT name=assertion assertion=A8_retirements_equal_identity_disappearances judged=184 violated=0
E7RESULT name=assertion assertion=K6_split_right_half_numbered_after_the_record_left_half_unchanged judged=12654 violated=0
E7RESULT name=anchor id=A5 zero_threshold_streams_identical=true holds=true
E7RESULT name=anchor id=A6 borrows=1 other_merges=0 merge=BorrowedFromTheRightNeighbour { records_moved: 5 } sizes=[12, 11] right_smallest=Some(22) n2_right_identity_number=Some(22) n1_right_separator=Some(22) n1_right_number=Some(17) extra_code3=1 extra_code2=1 extra_retirements=1 holds=true
E7RESULT name=positive_control id=PC-N zero_borrow_histories_identical=true single_borrow_history_holds=true holds=true
E7RESULT name=positive_control id=PC-K today_i94_reddens_k1_to_k5=true k0_all_green=true n1_i94_reddens_bn1_s20=true holds=true
```

### Readings: segment two (base B-on, the missed-live half of X2 and X3)

Line counts of each row type in the artifact (`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-two.out | sort | uniq -c`):

```
    196 name=anchor
     12 name=assertion
      1 name=config
      1 name=done
   2988 name=extra_writes
   5437 name=positive_control
    996 name=simulation
     26 name=stability
      2 name=stop_check
  61680 name=trajectory
   1945 name=verdict
```

996 = 83 segments × 12 cells; 61680 = 948 items × 20 readers × 3 kinds of scan + F11's 48 items × 20 × 5 kinds. Verdict lines counted by question and status (`grep 'name=verdict' … | awk` taking `question=` and `status=` / `judgement=`, `fl1_cell_excluded=`):

```
     48 question=Q-B1 status=zero
     24 question=Q-B2 status=not_run
     24 question=Q-B2 status=zero
    144 question=Q-X2-1 status=zero fl1_cell_excluded=false
    144 question=Q-X2-2 status=zero fl1_cell_excluded=false
    144 question=Q-X2-3 status=zero fl1_cell_excluded=false
    144 question=Q-X2-4 status=zero fl1_cell_excluded=false
    192 question=Q-X2-5 status=zero fl1_cell_excluded=false
     36 question=Q-X2-6 judgement=reported
    180 question=Q-X2-7 judgement=reported
     96 question=Q-X3-1 status=reported fl1_cell_excluded=false
    192 question=Q-X3-1 status=zero fl1_cell_excluded=false
     96 question=Q-X3-2 status=reported fl1_cell_excluded=false
    192 question=Q-X3-2 status=zero fl1_cell_excluded=false
    288 question=Q-X3-3 status=incidental
      1 question=summary
```

The 24 `not_run` rows of Q-B2 are the judged cells with batch = 1 (F11 only has batch 144). The 96 + 96 `reported` rows are all the E-I arm (missed live midway is a consequence of E-I's definition; numbers reported only, pinned by a7).

Full lines (same artifact; the four X2 main cells at capacity 16, θ = 8, n1, batch 1, the midway missed live at capacity 233, θ = 0, and the extra writes at θ = 8):

```
E7RESULT name=verdict question=Q-X2-1 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=Z effect=E-I window=inf extent_r_peak=0 extent_r_peak_at=0 extent_r_positive_scans=0 extent_r_last=0 inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-3 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=Z effect=E-I window=inf extent_m_outside_unlink_peak=0 extent_m_outside_unlink_peak_at=0 extent_m_outside_unlink_positive_scans=0 extent_m_outside_unlink_last=0 inode_m_outside_unlink_peak=0 inode_m_outside_unlink_peak_at=0 inode_m_outside_unlink_positive_scans=0 inode_m_outside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-2 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=T effect=E-I window=inf extent_r_peak=0 extent_r_peak_at=0 extent_r_positive_scans=0 extent_r_last=0 inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-4 borrow_arm=N1 capacity=16 theta=8 batch=1 base=B-on shrink=T effect=E-I window=inf extent_m_outside_unlink_peak=0 extent_m_outside_unlink_peak_at=0 extent_m_outside_unlink_positive_scans=0 extent_m_outside_unlink_last=0 inode_m_outside_unlink_peak=0 inode_m_outside_unlink_peak_at=0 inode_m_outside_unlink_positive_scans=0 inode_m_outside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-1 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-I window=inf inside_unlink_peak=1 inside_unlink_peak_at=5 inside_unlink_positive_scans=24441 inside_unlink_last=1 status=reported fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-2 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-I window=inf inside_unlink_peak=300 inside_unlink_peak_at=5 inside_unlink_positive_scans=24441 inside_unlink_last=1 status=reported fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-1 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-L-w window=inf inside_unlink_peak=0 inside_unlink_peak_at=0 inside_unlink_positive_scans=0 inside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X3-1 borrow_arm=N1 capacity=233 theta=0 batch=1 base=B-on shrink=Z effect=E-L-r window=inf inside_unlink_peak=0 inside_unlink_peak_at=0 inside_unlink_positive_scans=0 inside_unlink_last=0 status=zero fl1_cell_excluded=false
E7RESULT name=verdict question=Q-X2-6 borrow_arm=N1 capacity=16 theta=8 effect=E-I extra_code3_per_multi_write_publish_count=1753 extra_code3_per_multi_write_publish_smallest=1 extra_code3_per_multi_write_publish_largest=1 extra_code3_per_multi_write_publish_total=1753 extra_code2_per_multi_write_publish_count=1753 extra_code2_per_multi_write_publish_smallest=1 extra_code2_per_multi_write_publish_largest=1 extra_code2_per_multi_write_publish_total=1753 not_one_and_one=0 judgement=reported
E7RESULT name=verdict question=Q-X2-7 borrow_arm=N1 capacity=16 theta=8 family=F10 effect=E-I multi_write_publishes=1652 publishes=68189 share_ppm=24226 judgement=reported
E7RESULT name=verdict question=Q-X2-7 borrow_arm=N1 capacity=16 theta=8 family=F10 effect=E-L-w multi_write_publishes=16809 publishes=68189 share_ppm=246506 judgement=reported
E7RESULT name=verdict question=Q-B1 borrow_arm=N2 capacity=16 theta=8 batch=144 window=inf container_r_peak=0 container_r_peak_at=0 container_r_positive_scans=0 container_r_last=0 status=zero
E7RESULT name=verdict question=Q-B2 borrow_arm=N2 capacity=16 theta=8 batch=144 window=inf rollback_histories=true container_m_peak=0 container_m_peak_at=0 container_m_positive_scans=0 container_m_last=0 inode_m_peak=0 inode_m_peak_at=0 inode_m_positive_scans=0 inode_m_last=0 status=zero
E7RESULT name=verdict question=summary v2_positive_controls_paths_agree=true positive_controls_closed_form_holds=true v3_last_scan_paths_agree=true v4_two_runs_identical=true v5_anchors_hold=false v6_f10_cells_without_coverage=116 f11_rollbacks_too_far=4 f11_events_missing=24
```

Each of the 36 rows of Q-X2-6 (12 cells × 3 X3 arms) has minimum and maximum both 1 and `not_one_and_one=0`. Q-X2-7 merges the shares of the 12 cells by family and X3 arm (ppm; F11's batch is 144 and objects have at most 5 units, so there is no batch that is not the last, and both E-L arms equal E-I):

The `rollback_histories` column of Q-B2 (segment-two artifact, `grep -c 'rollback_histories=false'` and `=true` give 24 rows each): the cells reading false have no history with a rollback, and the apparatus does not record Q-B2 into the verdict (in `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs` it calls `judged("Q-B2"…)` only when `rollback_trajectories` has a value); in those rows every container_m quantity is 0.

| Family | E-I | E-L-w | E-L-r |
|---|---|---|---|
| F1 | 0 | 0 | 0 |
| F8 | 10135 | 766891 | 766891 |
| F3 | 27218 | 27218 | 27218 |
| F10 | 24226–24258 | 245510–246506 | 245510–246506 |
| F11 | 18448–30647 | 18448–30647 | 18448–30647 |

Under E-I, the 96 peak rows of Q-X3-1 are all 1 and the 96 peak rows of Q-X3-2 are all 300 (`grep 'question=Q-X3-[12] ' … | grep 'effect=E-I' | grep -oE 'inside_unlink_peak=[0-9]+' | sort | uniq -c` outputs `96 inside_unlink_peak=1`, `96 inside_unlink_peak=300`).

Positive controls counted by ID (`grep 'name=positive_control' … | awk` taking `id=`, `holds=`, `paths_agree=`, `closed_form_holds=`):

```
      3 id=PC-B1 holds=false
    957 id=PC-B1 holds=true
     48 id=PC-B2
      1 id=PC-B2-direct holds=true paths_agree=true
    996 id=PC-EL paths_agree=true closed_form_holds=true
    996 id=PC-ELr paths_agree=true closed_form_holds=true
    144 id=PC-G0 holds=true
    576 id=PC-M-data paths_agree=true closed_form_holds=true
    612 id=PC-M-leaf paths_agree=true closed_form_holds=true
    552 id=PC-T paths_agree=true closed_form_holds=true
    552 id=PC-Z paths_agree=true closed_form_holds=true
```

The three rows where PC-B1 does not hold, and PC-B2-direct (full lines, lines 68389, 68613, 71299, 8):

```
E7RESULT name=positive_control id=PC-B1 family=F11 n=300 gap_event=borrow batch=144 borrow_arm=N2 capacity=16 theta=4 disappeared_identities=4 final_container_r_inf=3 disappearance_scans_without_r_23=0 holds=false
E7RESULT name=positive_control id=PC-B1 family=F11 n=300 gap_event=borrow batch=144 borrow_arm=N2 capacity=16 theta=8 disappeared_identities=6 final_container_r_inf=4 disappearance_scans_without_r_23=0 holds=false
E7RESULT name=positive_control id=PC-B1 family=F11 n=300 gap_event=moved_object_shortened batch=144 borrow_arm=N2 capacity=16 theta=8 disappeared_identities=5 final_container_r_inf=4 disappearance_scans_without_r_23=0 holds=false
E7RESULT name=positive_control id=PC-B2-direct rolled_back=true b_on_container_m=0 b_off_lin_container_m=1 b_off_ret_container_m=0 expected=0,1,0 paths_agree=true holds=true
```

### Readings: segment three (the crash half of X3)

Line counts of each row type in the artifact (`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-three.out | sort | uniq -c`):

```
      1 name=config
     12 name=crash_prefix
   1440 name=crash_summary
      1 name=done
    288 name=positive_control
   2593 name=verdict
```

All 12 prefixes were found (`grep -c 'found=true'` outputs 12): four P-1 (capacity 233, 16 × n1, n2, θ = 0) and eight P-m (θ 58, 116 at capacity 233 and θ 4, 8 at capacity 16 × n1, n2; P-m needs a borrow or an absorption, so θ = 0 is not run). Each prefix has 454 crash points (u = 3, 145, 300 × batch 1, 144: 3 + 145 + 300 + 1 + 2 + 3), 5448 in total (the sum of `crash_points=` over the 72 rows of B-on × Z × E-I × ∞). Verdict lines counted by question, reader arm and status:

```
    288 question=Q-X3-4 base=B-on effect=E-I status=reported
    288 question=Q-X3-4 base=B-on effect=E-L-r status=zero
    288 question=Q-X3-4 base=B-on effect=E-L-w status=zero
    288 question=Q-X3-5 base=B-on effect=E-I status=incidental
    288 question=Q-X3-5 base=B-on effect=E-L-r status=incidental
    288 question=Q-X3-5 base=B-on effect=E-L-w status=incidental
    288 question=Q-X3-O2 base=B-on effect=E-I status=zero
    288 question=Q-X3-O2 base=B-on effect=E-L-r status=fl6_last_batch_effect_shows_extra_objects
    288 question=Q-X3-O2 base=B-on effect=E-L-w status=fl6_last_batch_effect_shows_extra_objects
      1 question=summary
```

o2 of Q-X3-O2 counted by arm (`o2_smallest`, `o2_largest` and "positive at every crash point"): the 288 E-I rows are all 0 / 0; the 288 rows each of E-L-w and E-L-r are all 1 / 1, positive at every crash point. The three resurrection peaks of the 864 rows of Q-X3-5 are all 0. E-I's Q-X3-4: inode m minimum and maximum both 1, extent m maximum 3, 145, 300 in 96 rows each (= u). The two W settings compared row by row: not one of the 1296 pairs flipped (counted by a draft script pairing on all fields except W).

Full lines (same artifact, lines 4082, 4083, 4094, 4095, 4106, 4107, 4334):

```
E7RESULT name=verdict question=Q-X3-O2 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-I window=inf o2_count=3 o2_smallest=0 o2_largest=0 o2_total=0 positive_crash_points=0 status=zero
E7RESULT name=verdict question=Q-X3-4 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-I window=inf inode_m_count=3 inode_m_smallest=1 inode_m_largest=1 inode_m_total=3 extent_m_count=3 extent_m_smallest=12 extent_m_largest=300 extent_m_total=468 status=reported
E7RESULT name=verdict question=Q-X3-O2 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-w window=inf o2_count=3 o2_smallest=1 o2_largest=1 o2_total=3 positive_crash_points=3 status=fl6_last_batch_effect_shows_extra_objects
E7RESULT name=verdict question=Q-X3-4 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-w window=inf inode_m_count=3 inode_m_smallest=0 inode_m_largest=0 inode_m_total=0 extent_m_count=3 extent_m_smallest=0 extent_m_largest=0 extent_m_total=0 status=zero
E7RESULT name=verdict question=Q-X3-O2 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-r window=inf o2_count=3 o2_smallest=1 o2_largest=1 o2_total=3 positive_crash_points=3 status=fl6_last_batch_effect_shows_extra_objects
E7RESULT name=verdict question=Q-X3-4 prefix=P-m borrow_arm=N1 capacity=16 theta=8 u=300 batch=144 base=B-on shrink=Z effect=E-L-r window=inf inode_m_count=3 inode_m_smallest=0 inode_m_largest=0 inode_m_total=0 extent_m_count=3 extent_m_smallest=0 extent_m_largest=0 extent_m_total=0 status=zero
E7RESULT name=verdict question=summary v2_positive_controls_paths_agree=true positive_controls_closed_form_holds=true v3_paths_agree=true prefixes_missing=0
```

Positive controls PC-O-plant, PC-O-remove, PC-M-data, PC-M-leaf have 72 rows each (12 prefixes × 6 (u, b)), and in every row `paths_agree=` and `closed_form_holds=` both equal that row's `crash_points=`.

### Readings: segment four (the two new bases of X1, B-stub and B-tsz)

Line counts of each row type in the artifact (`awk '{print $2}' research/results/e178-leaf-and-record-death-counts-segment-four.out | sort | uniq -c`):

```
      4 name=anchor
     17 name=assertion
      1 name=config
  30840 name=difference
      1 name=done
   9080 name=item_assertion
   9754 name=positive_control
     10 name=positive_control_tally
   9960 name=reader_extras
   1008 name=side_by_side
   1494 name=simulation
     98 name=stability
    498 name=stub_leaves
  74016 name=trajectory
   3398 name=verdict
```

1494 = 996 items of the original write path + 498 items of the B-stub write path (only the 6 write-path cells of n1); counted by family (`grep 'name=simulation' … | awk` taking `family=` and `path=`) F1 168 / 84, F3 60 / 30, F8 384 / 192, F10 336 / 168, F11 48 / 24, and every row has `two_runs_identical=true last_scan_paths_agree=true`. 74016 = original write path 18 readers × (948 × 3 + 48 × 5) kinds of scan + B-stub write path 12 readers × (474 × 3 + 24 × 5); by base B-on 37008, B-off-ret 6168, B-tsz 12336, B-stub 18504. 30840 = B-stub minus B-on 18504 + B-tsz minus B-on 12336. Every family produced the shape under test: in every family × capacity × θ × batch cell B-stub wrote stubs (3–518, Q-Bs-5), the B-tsz container r peak is ≥ 1 in every cell (dead identities did occur), and PC-Bs1, PC-Bt1 took scans on every family × both W settings (summary-row fields `pc_bs1_taken`, `pc_bt1_taken`).

Verdict lines counted by question, quantity and status (`grep 'name=verdict question=Q-B[st]-[1-4] ' … | awk` taking `question=`, `quantity=`, `tier=`, `status=` / `judgement=`, `sub_question=`, `differs=`, then merged by question):

| Question | Rows | Status |
|---|---|---|
| Q-Bs-1 B-stub resurrection (container r, inode r, extent r above B-on) | 432 | all `zero` |
| Q-Bs-2 B-stub false death (container m, inode m, extent m above B-on; the all tier and the after-rollback tier) | 864 | 648 `zero`; 216 `not_run` (the after-rollback tier of cells with batch = 1; F11 only has batch 144) |
| Q-Bs-3 missed live midway through unlink under B-stub | 288 | E-L-w, E-L-r 192 rows `zero`; E-I 96 rows `reported` (consequence of the definition) |
| Q-Bs-4 X2 under B-stub (Q-X2-1 to Q-X2-5) | 384 | all `differs=false`, the new base and B-on are both `zero` |
| Q-Bt-1 B-tsz resurrection | 288 | the 192 rows of inode r, extent r above B-on are `zero`; container r 96 rows `reported` (consequence of the definition) |
| Q-Bt-2 B-tsz false death | 672 | 432 `zero`; 144 `not_run` (as in Q-Bs-2); 96 rows of the count of records the two supply rules choose differently, all `total=0` |
| Q-Bt-3 B-tsz missed live midway through unlink | 192 | all `reported` (paired only with E-I, consequence of the definition) |
| Q-Bt-4 X2 under B-tsz (Q-X2-1 to Q-X2-4) | 192 | all `differs=false`, the new base and B-on are both `zero` |
| Q-Bs-5, Q-Bs-6 | 42 each | `reported` |

Midway peaks of Q-Bs-3 and Q-Bt-3 under E-I: inode m always 1, extent m always 300 (`grep -E 'question=Q-B[st]-3 ' … | grep 'effect=E-I'` counts 48 + 48 rows for Q-Bs-3 and 96 + 96 rows for Q-Bt-3), the same values as B-on's E-I in segment two. B-tsz's container r peak is 1–166 by cell, while B-on in the same cell is 0 (`b_on_peak=0`).

Full lines (same artifact; the two new bases in the cell capacity 16, θ = 8, n1, batch 144, W = ∞, Z × E-I, the midway missed live of E-L-w in the same cell, B-tsz's container r in the same cell for n2, plus the summary, FL-Bx and the two constructions):

```
E7RESULT name=verdict question=Q-Bs-1 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=container_r container_r_peak=0 container_r_peak_at=0 container_r_positive_scans=0 container_r_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-1 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=inode_r inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-2 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I tier=after_rollback quantity=container_m container_m_peak=0 container_m_peak_at=0 container_m_positive_scans=0 container_m_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-2 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I tier=after_rollback quantity=inode_m inode_m_peak=0 inode_m_peak_at=0 inode_m_positive_scans=0 inode_m_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bs-3 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-L-w quantity=inode_m inode_m_inside_unlink_peak=0 inode_m_inside_unlink_peak_at=0 inode_m_inside_unlink_positive_scans=0 inode_m_inside_unlink_last=0 b_on_status=zero status=zero
E7RESULT name=verdict question=Q-Bs-3 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-L-w quantity=extent_m extent_m_inside_unlink_peak=0 extent_m_inside_unlink_peak_at=0 extent_m_inside_unlink_positive_scans=0 extent_m_inside_unlink_last=0 b_on_status=zero status=zero
E7RESULT name=verdict question=Q-Bs-4 base=B-stub borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I sub_question=Q-X2-1 new_base_status=zero b_on_status=zero differs=false
E7RESULT name=verdict question=Q-Bt-1 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=container_r container_r_peak=56 container_r_peak_at=1799 container_r_positive_scans=25219 container_r_last=56 b_on_peak=0 status=reported
E7RESULT name=verdict question=Q-Bt-1 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=inode_r inode_r_peak=0 inode_r_peak_at=0 inode_r_positive_scans=0 inode_r_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bt-2 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I tier=after_rollback quantity=inode_m inode_m_peak=0 inode_m_peak_at=0 inode_m_positive_scans=0 inode_m_last=0 b_on_peak=0 status=zero
E7RESULT name=verdict question=Q-Bt-2 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=provenance_differences total=0 judgement=reported
E7RESULT name=verdict question=Q-Bt-4 base=B-tsz borrow_arm=N1 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I sub_question=Q-X2-1 new_base_status=zero b_on_status=zero differs=false
E7RESULT name=verdict question=Q-Bt-1 base=B-tsz borrow_arm=N2 capacity=16 theta=8 batch=144 window=inf shrink=Z effect=E-I quantity=container_r container_r_peak=166 container_r_peak_at=1799 container_r_positive_scans=26876 container_r_last=166 b_on_peak=0 status=reported
E7RESULT name=verdict question=FL-Bx x2_cells_whose_judgement_changes_with_the_base=0 cells=none
E7RESULT name=verdict question=summary v2_paths_disagree=0 v2_closed_form_fails=0 v3_last_scan_paths_disagree=0 v4_runs_not_identical=0 v5_anchors_hold=true v6_f10_cells_without_coverage=116 v8_new_base_trajectories_with_empty_reads=2448 v_bs1_pairing_fails=0 v_bs3_segments_with_stub_read_mismatches=0 does_not_fit_segments=0 f11_rollbacks_too_far=4 f11_events_missing=24 pc_bt2_segments_with_a_positive_hand_count=0 pc_bt2_mid_segments_with_a_positive_hand_count=220 pc_bs1_taken=F1/W23,F1/Winf,F10/W23,F10/Winf,F11/W23,F11/Winf,F3/W23,F3/Winf,F8/W23,F8/Winf pc_bt1_taken=F1/W23,F1/Winf,F10/W23,F10/Winf,F11/W23,F11/Winf,F3/W23,F3/Winf,F8/W23,F8/Winf construction=PC-Bs2=true,PC-Bt3=true
E7RESULT name=positive_control id=PC-Bs2 rolled_back=true b_stub_container_m=0 b_stub_inode_m=0 b_stub_extent_m=0 b_stub_lin_off_container_m=1 b_stub_lin_off_inode_m=1 b_stub_lin_off_extent_m=0 expected=0,0,1,1,0 paths_agree=true holds=true
E7RESULT name=positive_control id=PC-Bt3 rolled_back=true b_tsz_inode_m=0 b_tsz_container_m=0 b_tsz_extent_m=0 b_tsz_inode_m_ignoring_lineage=1 expected=0,1 paths_agree=true holds=true
```

These rows are at artifact lines 136748, 136749, 136754, 136755, 136783, 136784, 136759, 136830, 136831, 136837, 136843, 136841, 138568, 138960, 140178, 135646, 135647.

B-stub's leaf count, stub share and versions hitting I-9.13 (type-2 container nonempty) (the 84 rows of Q-Bs-5, Q-Bs-6 merged by family and capacity, taking the maximum or the range over the θ and batch settings; command and raw output):

```
grep -E 'question=Q-Bs-[56] ' research/results/e178-leaf-and-record-death-counts-segment-four.out | awk '{for(i=3;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]}; k=v["family"]" "v["capacity"]; if(v["question"]=="Q-Bs-5"){sw[k]=sw[k]","v["stubs_written"]; if(v["stubs_last_largest"]>st[k])st[k]=v["stubs_last_largest"]; if(v["leaves_peak_largest"]>lp[k])lp[k]=v["leaves_peak_largest"]; if(v["stub_share_last_ppm_largest"]>sh[k])sh[k]=v["stub_share_last_ppm_largest"]; ov[k]+=v["versions_over_the_root"]} else {p=v["share_ppm"]+0; if(!(k in mn)||p<mn[k])mn[k]=p; if(p>mx[k])mx[k]=p}} END{for(k in st) print k, "stubs_written="substr(sw[k],2), "stubs_last_max="st[k], "leaves_peak_max="lp[k], "stub_share_last_ppm_max="sh[k], "versions_over_the_root="ov[k], "i913_false_share_ppm="mn[k]"-"mx[k]}' | sort
F10 16 stubs_written=518,518,518,518,518,518 stubs_last_max=56 leaves_peak_max=57 stub_share_last_ppm_max=982456 versions_over_the_root=0 i913_false_share_ppm=264180-526791
F10 233 stubs_written=28,28,28,28,28,28 stubs_last_max=3 leaves_peak_max=4 stub_share_last_ppm_max=750000 versions_over_the_root=0 i913_false_share_ppm=56343-462809
F11 16 stubs_written=57,26,25 stubs_last_max=18 leaves_peak_max=19 stub_share_last_ppm_max=947368 versions_over_the_root=0 i913_false_share_ppm=340586-543564
F11 233 stubs_written=4,5,5 stubs_last_max=1 leaves_peak_max=2 stub_share_last_ppm_max=500000 versions_over_the_root=0 i913_false_share_ppm=1056-286353
F1 16 stubs_written=518,518,518 stubs_last_max=56 leaves_peak_max=57 stub_share_last_ppm_max=982456 versions_over_the_root=0 i913_false_share_ppm=386173-670710
F1 233 stubs_written=28,28,28 stubs_last_max=3 leaves_peak_max=4 stub_share_last_ppm_max=750000 versions_over_the_root=0 i913_false_share_ppm=82675-594982
F3 16 stubs_written=3,3,4 stubs_last_max=2 leaves_peak_max=66 stub_share_last_ppm_max=35087 versions_over_the_root=0 i913_false_share_ppm=366358-499727
F3 233 stubs_written=3,3,3 stubs_last_max=2 leaves_peak_max=6 stub_share_last_ppm_max=333333 versions_over_the_root=0 i913_false_share_ppm=366358-366358
F8 16 stubs_written=16,16,16,16,16,16 stubs_last_max=1 leaves_peak_max=1 stub_share_last_ppm_max=1000000 versions_over_the_root=0 i913_false_share_ppm=15473-106666
F8 233 stubs_written=16,16,16,16,16,16 stubs_last_max=1 leaves_peak_max=1 stub_share_last_ppm_max=1000000 versions_over_the_root=0 i913_false_share_ppm=15473-106666
```

`stubs_written` is the total over segments, one number per θ (and batch); `i913_false_share_ppm` is the share of versions with a zero-entry version among all publishes (`share_ppm` of Q-Bs-6; denominator as reading 3 of section one). The larger θ, the more versions hit I-9.13 (type-2 container nonempty) (F1 capacity 16: θ 0, 4, 8 give 386173, 615828, 670710 ppm, with the first stub at the earliest at publish 71, 49, 41); the inferred cause is that at θ > 0 absorbed leaves also leave stubs, so the first stub comes earlier; not measured separately. Anchors a10 and a11 both matched (`assertion=A10_stubs_written_equal_final_stubs_equal_a8 judged=28 violated=0`, `assertion=A11_first_stub_publish_and_versions_with_a_stub judged=4 violated=0`). The total leaf count never exceeded the 135 leaves the root can hold (42 rows `versions_over_the_root=0`), so "stubs make the root grow taller" did not occur in these histories, and the model does not count the writes of a root growing taller either.

The three bases side by side (`name=side_by_side` 1008 rows = 84 family × write-path cell × batch × Z / T × three X3 arms × two W settings; each cell reads "resurrection container/inode/extent : false death container/inode/extent"):

```
grep 'name=side_by_side' research/results/e178-leaf-and-record-death-counts-segment-four.out | awk '{for(i=1;i<=NF;i++){if($i ~ /^b_on=/)a=$i; if($i ~ /^b_stub=/)b=$i; if($i ~ /^b_tsz=/)c=$i}; sub(/^b_tsz=[0-9]+\//,"b_tsz=<container_r>/",c); print a, b, c}' | sort | uniq -c
    168 b_on=0/0/0:0/0/0 b_stub=0/0/0:0/0/0 b_tsz=<container_r>/0/0:0/0/0
    336 b_on=0/0/0:0/0/0 b_stub=0/0/0:0/0/0 b_tsz=n/a
    168 b_on=0/0/0:0/0/0 b_stub=n/a b_tsz=<container_r>/0/0:0/0/0
    336 b_on=0/0/0:0/0/0 b_stub=n/a b_tsz=n/a
```

`n/a` marks cells where that arm is not paired: B-stub is not paired with n2 (question one n2-none), B-tsz is not paired with E-L-w, E-L-r. Two full lines (lines 139585, 139729):

```
E7RESULT name=side_by_side family=F10 borrow_arm=N1 capacity=16 theta=8 batch=1 window=inf shrink=Z effect=E-I resurrected_c_i_e_and_misjudged_c_i_e b_on=0/0/0:0/0/0 b_stub=0/0/0:0/0/0 b_tsz=56/0/0:0/0/0
E7RESULT name=side_by_side family=F10 borrow_arm=N2 capacity=16 theta=8 batch=1 window=inf shrink=Z effect=E-I resurrected_c_i_e_and_misjudged_c_i_e b_on=0/0/0:0/0/0 b_stub=n/a b_tsz=166/0/0:0/0/0
```

Two comparisons the executor made in addition (the registration did not write them as criteria; corroboration only): B-stub's 18504 trajectory rows, with `base=` replaced, are identical line by line to the 18504 B-on rows of n1 in the same cells (r and m of container, inode, extent are the same on every range); the 6168 rows of B-tsz × Z × E-I and the 6168 rows of B-off-ret × Z × E-I, with `base=` replaced, are identical line by line. The latter is what registration amendment 2, part four, item 1 inferred, "under P-W the two are the same reader"; P-C and P-W did not choose a single different record on these histories (96 rows of `provenance_differences total=0`), so it holds as well. The answers of those B-tsz × Z cells were therefore already in the segment-two artifact of B-off-ret × Z × E-I before segment four ran, and the executor of segments two and three had read that artifact, so those cells are not blind judgements (registration amendment 2, part four, item 1 requires saying so).

V-Bt2 (segment four's B-on and B-off-ret compared row by row with the same readers in the same cells of segment two): segment two has 4 more g0 readers per cell (B-on × g0 × E-I / E-L-w × two W settings, used for segment two's positive controls, not in the reader table of segment four in amendment 2, part five); with them removed, the rows are identical line by line:

```
grep '^E7RESULT name=trajectory' research/results/e178-leaf-and-record-death-counts-segment-four.out | grep -E 'base=(B-on|B-off-ret) ' | sort > four.txt
grep '^E7RESULT name=trajectory' research/results/e178-leaf-and-record-death-counts-segment-two.out | grep -E 'base=(B-on|B-off-ret) ' | sort > two.txt
grep -v ' shrink=G0 ' two.txt > two-without-g0.txt; wc -l four.txt two.txt two-without-g0.txt; cmp four.txt two-without-g0.txt && echo identical
   43176 four.txt
   55512 two.txt
   43176 two-without-g0.txt
identical
```

Positive-control tallies and assertion totals (full lines, lines 140168–140177 and the segment-four rows among lines 140151–140167):

```
E7RESULT name=positive_control_tally id=PC-Bs1 count=962 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Bt1 count=1724 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Bt2 count=224 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Bt2-mid count=224 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-EL count=1494 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-ELr count=1494 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-M-data count=864 paths_disagree=0 closed_form_fails=0 registered_literal_fails=18
E7RESULT name=positive_control_tally id=PC-M-leaf count=1110 paths_disagree=0 closed_form_fails=0 registered_literal_fails=1110
E7RESULT name=positive_control_tally id=PC-T count=828 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=positive_control_tally id=PC-Z count=828 paths_disagree=0 closed_form_fails=0 registered_literal_fails=0
E7RESULT name=assertion path=segment_four assertion=A12_tombstones_only_container_r_equals_without_retirement judged=4374408 violated=0
E7RESULT name=assertion path=segment_four assertion=A12_tombstones_only_final_container_r_equals_a8 judged=112 violated=0
E7RESULT name=assertion path=segment_four assertion=V-Bs1_pairing_holds judged=498 violated=0
E7RESULT name=assertion path=segment_four assertion=V-Bs3_stubs_read_equal_the_current_stubs judged=498 violated=0
E7RESULT name=assertion path=segment_four assertion=V2_positive_control_closed_form_holds judged=9752 violated=0
E7RESULT name=assertion path=segment_four assertion=V2_positive_control_paths_agree judged=9752 violated=0
```

`registered_literal_fails` is computed only for the two PC-M items (the apparatus's `segment_four_positive_control_line` judges the registered letter only for PC-M); the 0 in that column for the other controls means not judged, not holding literally; PC-T is judged as in segment two, more loosely than the registration. PC-M by the letter of the registration still does not hold (18 data-unit items, all 1110 leaf-container items), while the apparatus's ≤ judgement holds, for the same cause as in segment two. Hand-counted values of the 224 PC-Bt2-mid items (`grep 'id=PC-Bt2-mid ' … | grep -oE 'hand_extent_r_z=[0-9]+' | sort | uniq -c`): 0 for 4 items, 1 for 114 items, 2 for 36 items, 3 for 32 items, 4 for 38 items. The 98 rows of stability comparison (Q-Bs-1 to Q-Bs-4, Q-Bt-1, Q-Bt-2, Q-Bt-4 over the four knobs W, θ, capacity, batch) are all `stable=true` with flip count 0.

| Clause | Hit or not | Basis |
|---|---|---|
| FL0 the clause may be wrong | Not hit: the four rows of K1, K2, K3 and a2 are `holds=true`, and K6 is `violated=0` over 12654 splits | artifact lines 2–5, line 1523 |
| FL8 n1 misses | Hit: Q-X4-4 has 8 shapes on both layouts, the list as is in the verdicts section; K-2 is judged green, so the cell predicted by registration section four, item 4 was not wrong | artifact lines 1596, 1620 |
| FL9 n2's "I-9.4 (container number not above the smallest key) unchanged" does not hold | Not hit: on the six cells, n2's counts of versions red under the 2026-10-08 I-9.4 (container number not above the smallest key) and under I-9.12 (Separator Key Falls Outside Child Range) are all 0 | the twelve V7 rows |
| FL10 n1's changed I-9.4 (container number not above the smallest key) is not green on legal images | Not hit: on the six cells, n1's counts of versions red under n1's I-9.4 (container number not above the smallest key) and under I-9.12 (Separator Key Falls Outside Child Range) are all 0 | the twelve V7 rows |
| FL1 the base loses a cell | Not hit: the 48 rows of Q-B1 and the 24 rows of Q-B2 with rollback histories are all `status=zero`, and the verdict lines have `fl1_cell_excluded=false` on 1344 rows | segment-two verdict lines |
| FL2 Z loses a cell, FL3 T loses a cell, FL4 both lose | Not hit: Q-X2-1 to Q-X2-4 have 144 rows each, all 0 | segment-two verdict lines |
| FL5 "last batch takes effect" also misses live | Not hit: Q-X3-1, Q-X3-2 of E-L-w, E-L-r are 0 on 192 rows each (segment two); Q-X3-4 is 0 on 288 rows each (segment three) | verdict lines of segments two and three |
| FL6 "last batch takes effect" leaves extra objects after a crash | Hit literally: Q-X3-O2 under E-L-w, E-L-r is 1 at every crash point (576 rows). Counted as one loss each for E-L-w and E-L-r. This item is a direct consequence of the two arms' definitions: midway through unlink, the two E-L arms by definition leave N's record in the rebuilt view (registration 5.3 "midway through unlink o's record is still in the tree"), and the crash points all lie between the intent and the last batch, so o2 counts exactly this record; the threshold is a tautology of the arm definition (`test-discipline.md` "do not write a criterion as a conjunction; a threshold must not be a tautology of the arm definition"), so whether this cell counts as a criterion is handed to the main agent | segment-three verdict lines |
| FL7 E-I leaves extra objects after a crash | Not hit: the 288 Q-X3-O2 rows of E-I are all 0 | segment-three verdict lines |
| V1 history reproduction does not match | Not hit: the two a1 assertions are judged 612 times each (51 segments × 6 cells × 2 arms) with 0 violations; the publish count and refused-step count of the five F3 segments are the same as the two segments in the `name=simulation` row with `geometry=wide` in the artifact of E177 (Resurrection Count Without Per-Tree Tombstones) | artifact lines 1520–1521; apparatus constant `E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS` |
| V4 nondeterminism | Not hit: the 474 work items are identical item by item over two runs (`grep -c 'two_runs_identical=true'` on `name=borrow_cost` rows counts 474), and the user-change sequences of n1 and n2 are identical segment by segment; the two whole artifacts with `E178_THREADS=6` and `=3` both have sha256 `9644a139…a8ea` | artifact line 1625 |
| V5 independent anchors do not match | Not hit: a2, a5, a6, a8 all match (a8 judged 184 times, 0 violations) | artifact lines 5, 1522, 1524, 1525 |
| V6 path not reached | Partly hit: among F10's four cells with θ > 0, capacity 233, θ = 58 has 24 segments, θ = 116 has 14 segments, and capacity 16, θ = 4 has 20 segments with no borrow at all (the counts of moved objects getting shorter and being unlinked are > 0 in every segment); all 28 segments of capacity 16, θ = 8 were reached. Those segments are not results for X4; every cell has segments that were reached, so the family is not invalidated. The unreached segments have a borrow count of 0 and add no observation to Q-X4-1, Q-X4-2 | `v6_segments_without_coverage` of the six F10 rows of Q-X4-3 |
| V7 model write-path self-check | Not hit: the cells required to be all green are all 0; I-9.13 (type-2 container nonempty) is 0 in all 12 rows | the twelve V7 rows |
| V1 | Not hit: the two a1 assertions are judged 612 times each, 0 violations | `name=assertion` rows of segment two |
| V2 (segment two) PC-B1 | Hit on three segments: F11's borrow gap (capacity 16, θ = 4, n2), borrow gap (θ = 8, n2), and gap where a moved object gets shorter (θ = 8, n2), `holds=false`, with 4, 6, 5 identities that disappeared while B-off-ret's final container r is 3, 4, 4. Those three segments are invalidated literally. Cause (the unit test `identities_born_and_retired_inside_the_abandoned_gap_are_what_pc_b1_overcounts` recomputes the same (4, 3), (6, 4), (5, 4) and pins them): every version of the missing identities has a birth txg that is not on the final parent chain (new identities that n2's borrow gave the right leaf inside the gap, which disappeared again inside the gap), so no reader can read them; PC-B1's expected count counts "identities that disappeared" and did not subtract this kind. The V5 summary cell (`v5_anchors_hold=false`) is exactly these three violations (`assertion=PC-B1_without_retirement_resurrects_every_disappeared_identity judged=960 violated=3`); the other 11 assertions all have 0 violations | segment-two lines 68389, 68613, 71299, 73281, 73283 |
| V2 (segment two) PC-M | Does not hold by the letter of the registration: the registration writes "every reader extent m + 1; at the erasure of a container, inode m + that leaf's record count, container m + 1"; in the artifact, 12 of the 576 items erasing a data unit are + 0 for all readers (the group of the erased unit still has other versions, `single_version=false`), and not one of the 612 items erasing a leaf container is + record count for all readers (readers with W = ∞ all fall back to the readable previous version of that leaf; in 132 items the 10 readers with W = 23 are + record count). The apparatus judges with ≤ (`erasure_positive_controls`), looser than the registration, so `closed_form_holds=true`. By the letter V2 is hit; whether to invalidate is handed to the main agent | `id=PC-M-` rows of segment two; command in the report |
| V2 (segment two) PC-T | The apparatus only judges that the increments of Z and g0 are 0, and does not compare T's increment against the count written in the registration (the reference path is the same as the main path, `paths_agree=true`); of the 552 items, T readers' increments are all positive on 308, partly positive on 220, all 0 on 24. Looser than the registration, handed to the main agent | `id=PC-T ` rows of segment two |
| V2 (segment two) others | PC-Z, PC-EL, PC-ELr: the two paths agree and the expectations hold; PC-G0 holds on 144 items (= a3); PC-B2-direct holds; the original PC-B2 on F11 was decided by the main agent not to be a result | positive-control counts |
| V3 the two counting paths disagree | Not hit: segment two `v3_last_scan_paths_agree=true`, segment three `v3_paths_agree=true` | summary rows of both segments |
| V4 nondeterminism | Not hit (segment two): the 996 items are identical scan by scan over two runs (`grep -c 'two_runs_identical=true last_scan_paths_agree=true'` on `name=simulation` rows counts 996); changing the thread count was not compared (segments two and three were run only once) | summary row of segment two |
| V5 | a3, a4, a7, a8, a9, K4, K6 all have 0 violations; the summary row's `v5_anchors_hold=false` comes only from the three PC-B1 violations (the V2 row) | segment-two lines 73271–73282 |
| V6 | Among F10's 224 items with θ > 0, 116 items have a 0 among the three counts of borrows, getting shorter after being moved and being unlinked after being moved (`v6_f10_cells_without_coverage=116`); each of the 8 (N, capacity, θ) cells has segments that were reached (capacity 16, θ = 4: 8 segments each, θ = 8: 28 segments each; capacity 233, θ = 58: 4 segments each, θ = 116: 14 segments each; same for n1, n2), so the family is not invalidated. F11 has 24 items where the gap event did not occur (`f11_events_missing=24`, mostly θ = 0 with no borrow or absorption); those items are not results for PC-B2 | `name=simulation family=F10` rows of segment two; command in the report |
| F11 rolled back too far | 4 segments (gap where a moved object gets shorter, capacity 233, θ = 58 and capacity 16, θ = 8, n1, n2) have rollback distance 21, recorded literally as "rolled back too far" by the registration; those segments are not results | `too_far=true` on the `id=PC-B2` rows of segment two |
| V8 reads empty | Hit literally: E-I readers (B-on's Z, T, g0, B-off-ret, B-off-lin) on all 384 items of F8 and on 24 of the 60 items of F3 had midway scans where the inode record set read out is empty while the truth is not (E-I by definition drops the only record); those scans are invalidated by the registration and are not recorded as m = |T|. The two E-L arms never did. All of these are midway through unlink and touch only E-I's Q-X3-1, Q-X3-2 (numbers reported only, not judged), not the verdicts of X2 or E-L | `read_out_empty_scans` on `name=trajectory` rows of segment two; command in the report |
| Invalidation clauses of segment three | V2: the two PC-O items and the two PC-M items agree on both paths over the 5448 crash points and the expectations written in the apparatus hold (PC-M again with the ≤ judgement); V3 not hit; all prefixes found | summary row of segment three |
| Stop S1, S2, S3, S6 | All matched | artifact lines 6–9 |
| Stop S4, S5 | All matched: `E7RESULT name=stop_check id=S4 holds=true detail=record=Some(InodeRecord { inode: 1, object_birth: 3, nlink: 0, size_bytes: 97902 }) tombstones=[TombstoneRecord { object: 1, object_birth: 3, range_start_bytes: 0, death_txg: 7 }] intent_txg=7`, `E7RESULT name=stop_check id=S5 holds=true detail=container_sizes=[815, 1] container_numbers=[1, 2] intent_publishes=816` | segment-two lines 6, 7 |
| Stop (segment four) | No new differential items; after the apparatus was changed for segment four, the unit tests of S1–S6 still ran and all passed (unit-test output) | `e178-leaf-and-record-death-counts-unit-tests-2026-10-08.out` |
| FL-Bs1 B-stub loses a cell | Not hit: the 432 rows of Q-Bs-1 and the 648 judged rows of Q-Bs-2 are all `zero`; on the PC-Bs2 construction B-stub's own container m, inode m, extent m are all 0 | segment-four verdict lines; line 135646 |
| FL-Bt1 B-tsz loses a cell | Not hit: the 192 rows of the inode r and extent r differences of Q-Bt-1 and the 432 judged rows of Q-Bt-2 are all `zero`; on the PC-Bt3 construction B-tsz's own inode m, container m, extent m are all 0 | segment-four verdict lines; line 135647 |
| FL-B3 all three bases lose the same cell | Not hit: none of the three bases lost a single cell | the 1008 side-by-side rows |
| FL-Bs2 under B-stub "last batch takes effect" also misses live | Not hit: the 192 E-L-w, E-L-r rows of Q-Bs-3 are all `zero`; side by side with FL5 not being hit under B-on | segment-four verdict lines |
| FL-Bx the verdict of X2 changes with the base | Not hit: the 384 rows of Q-Bs-4 and the 192 rows of Q-Bt-4 are all `differs=false`, the list is empty (`x2_cells_whose_judgement_changes_with_the_base=0 cells=none`) | line 138960 |
| V-Bs1 pairing premise | Not hit: 498 items of the pairing assertion, 0 violations | line 140160 |
| V-Bs2 B-stub anchors | Not hit: a10 judged 28 times, a11 judged 4 times, 0 violations | lines 140156, 140157 |
| V-Bs3 stub count read | Not hit: 498 items with 0 violations, summary `v_bs3_segments_with_stub_read_mismatches=0` | lines 140161, 140178 |
| V-Bt1 B-tsz and B-off-ret | Not hit: a12 judged 4374408 times scan by scan and 112 times at the end, 0 violations; question two takes P-C, so the P-W sentence is not judged, but the two rules chose 0 different records, and the trajectories are identical line by line too ("Readings: segment four") | lines 140158, 140159 |
| V-Bt2 the re-run B-on differs from segment two | Not hit: 43176 rows identical line by line (the extra 12336 rows in segment two are g0 readers that segment four does not run) | the command in "Readings: segment four" |
| V2 | The two paths agree and the apparatus's expectations all hold (9752 times, 0 violations); PC-Bs1, PC-Bt1, PC-Bt2-mid hold; the original PC-Bt2's "at least one segment > 0 at F10 θ > 0" cannot be measured, as amendment 3 states (all 224 items have hand-counted value 0, `pc_bt2_segments_with_a_positive_hand_count=0`), handed to the main agent to decide whether it still counts as a result; PC-M does not hold by the letter of the registration on 1128 items (as in segment two), and PC-T's judgement is looser than the registration as in segment two; both handed to the main agent | lines 140168–140178 |
| V3, V4 (segment four) | Not hit: `v3_last_scan_paths_disagree=0`, `v4_runs_not_identical=0`, the 1494 write paths are identical scan by scan over two runs; this host computed in two shards, computed the missing items and assembled, and independence from thread count and shard count is corroborated by the 20 sampled items recomputed byte-identically | line 140178; `name=simulation` rows |
| V6 | Same as segment two: 116 items of F10 θ > 0 did not reach all three counts (`v6_f10_cells_without_coverage=116`), and every cell has segments that were reached; F11 has 24 items without the gap event and 4 segments rolled back too far | line 140178 |
| V8 | Hit literally, in the same form as segment two: on the new bases 2448 trajectories (B-stub 816, B-tsz 1632) have scans that read empty, all E-I readers, all in F3 and F8; the inode m peaks of these trajectories in the "outside midway" tier are all 0 (reading empty while the truth is not necessarily gives m ≥ 1), so all empty reads are midway through unlink and touch only the E-I cells of the report-only Q-Bs-3 and Q-Bt-3 | line 140178; command in the executor's report |

How the V6 segments with no borrow at all split (the n2 arm, F10, θ > 0; `b0` is 0 borrows, `s+ u+` means getting shorter and being unlinked are both > 0; the same number appears once per batch size):

```
grep 'name=write_path family=F10' research/results/e178-leaf-and-record-death-counts-segment-one.out | grep 'borrow_arm=N2' | grep -v 'theta=0 ' | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]} print v["capacity"], v["theta"], v["n"], v["batch"], (v["borrows"]>0?"b+":"b0"), (v["moved_then_shortened"]>0?"s+":"s0"), (v["moved_then_unlinked"]>0?"u+":"u0")}' | sort | uniq -c | sort -k2,3n
      1 16 4 300 144 b+ s+ u+
      1 16 4 300 1 b+ s+ u+
      3 16 4 900 144 b+ s+ u+
      3 16 4 900 1 b+ s+ u+
      4 16 4 900 144 b0 s+ u+
      4 16 4 900 1 b0 s+ u+
      6 16 4 300 144 b0 s+ u+
      6 16 4 300 1 b0 s+ u+
      7 16 8 300 144 b+ s+ u+
      7 16 8 300 1 b+ s+ u+
      7 16 8 900 144 b+ s+ u+
      7 16 8 900 1 b+ s+ u+
      2 233 58 900 144 b+ s+ u+
      2 233 58 900 1 b+ s+ u+
      5 233 58 900 144 b0 s+ u+
      5 233 58 900 1 b0 s+ u+
      7 233 116 300 144 b0 s+ u+
      7 233 116 300 1 b0 s+ u+
      7 233 116 900 144 b+ s+ u+
      7 233 116 900 1 b+ s+ u+
      7 233 58 300 144 b0 s+ u+
      7 233 58 300 1 b0 s+ u+
```

### Unit tests and mutations

Samples and red proof of the resume key (apparatus code key) (2026-10-15): the verdict line of `e178-leaf-and-record-death-counts-resume-key-samples-2026-10-15.out` is `key_samples=8 key_samples_holding=8 f_k1_missed=0 f_k2_extra=0 unit_test_samples=1`; the red proof of the two item-key mutations M23 (item key misses a parameter) and M24 (item key carries an ordinal) is in `e178-leaf-and-record-death-counts-resume-key-prove-red-2026-10-15.out`, both exit 101, and after restoring `restored exit=0 passed=54 failed=0`.

After segment four was added: 44 unit tests (`grep -c '^test .* \.\.\. ok$' e178-leaf-and-record-death-counts-unit-tests-2026-10-08.out` outputs 44; line 51 of the same file is `test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s`). The 12 new pins: the two SHA-256 standard values; a10, a11 (H51 F1 n = 300 ascending, capacity 16, θ = 0: 18 stubs, 19 final leaves, first stub at publication 71, 537 versions with a stub; n = 900 capacity 233 even first then odd: 3 stubs, 4 leaves); V-Bs1 (version-by-version pairing of the two write paths on F10's two segments, F3 and the F11 absorption gap, with stubs and absorptions actually present); the PC-Bs2 and PC-Bt3 constructions; B-stub final container r 0, 18 stubs read, each read equal to the current stub count (V-Bs3); B-tsz final container r 18 (= B-off-ret, a12), inode r 0, PC-Bt1 holding at both W settings; B-stub container r always 0 in the F11 absorption gap; PC-Bt2 final hand count 0, PC-Bt2-mid hand count > 0 while extent r of that scan under P-C is 0; the pairing, positive controls and assertions of one segment-four work item; resume (artifacts assembled with reuse off and with reuse on are identical line by line, and a shard computes only the items assigned to it); parameters; the cause of the three PC-B1 segments in segment two.

Mutation table `scripts/mutation/tables/e178_leaf_and_record_death_counts.tsv` grown to 22 entries (the seven M16–M22 of amendment 2, part ten; each of the 22 anchors matches exactly once): caught 22, invalid 0, not red 0; `💥`, `⚠️`, `⏱`, `🧱` 0 each; closing lines 「计数：内存撞顶 0 条（上限 16G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条」「已还原，基线仍全绿」 (the two lines 「计数：」 and 「已还原，基线仍全绿」 in `e178-leaf-and-record-death-counts-mutations-2026-10-08.out`). M18 is not sensitive at the sampling point written in the registration (the final scan) and is caught by the PC-Bt2-mid scan (amendment 3); the cell M15 added (B-tsz's inode m + 1 on the PC-Bt3 construction) is caught by the unit test of that construction.

At segment one: 32 unit tests (`grep -c '^test .* \.\.\. ok$' e178-leaf-and-record-death-counts-unit-tests.out` outputs 32; line 39 of the same file is `test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.43s`). Pinned absolute values: K1, K2, K3, a2 and the payload capacity; the shape count of each family of 5.6; the verdicts of K-2, BN1-s20, BN1-s30; the midpoints of the sparse layout; a1 (F1 n = 300 publish count 607, one F8 segment 13, the five F3 segments the same as the artifact of E177 (Resurrection Count Without Per-Tree Tombstones)); a5 (θ = 0, the two write streams the same); a6 (one-borrow small history); a8 (F1 n = 300, capacity 16, 18 retirements); a3 (F8 n_u = 3 overwrite before deletion, g0's extent r = 2, Z and T 0); a7 (F8 n_u = 300, b = 144, E-I midway extent m 300, 156, 12); stop differentials S1–S6; the boundary of ring windows 23 and 24; the parent chain skipping the segment abandoned by a rollback; moving the verdict threshold turns green into red; trajectory peaks, positive-scan counts and final values. Differential kind: newest selection by the group index and by the full scan are the same at the end of three histories; on one segment-two work item (F10 n = 300), PC-Z, PC-T, PC-EL, PC-ELr, PC-M agree on both paths and the registered expectations hold; three crash points of segment three at P-1, u = 3, b = 1 agree on both paths, and the two PC-O items hold.

Mutation table `scripts/mutation/tables/e178_leaf_and_record_death_counts.tsv` 15 entries (M1–M15 of registration section nine; each anchor matches exactly once): caught 15, invalid 0, not red 0; `💥`, `⚠️`, `⏱`, `🧱` 0 each; closing lines 18, 19 「计数：内存撞顶 0 条（上限 6G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条」「已还原，基线仍全绿」 (`e178-leaf-and-record-death-counts-mutations.out`). M3 is not sensitive at the F11 sampling point written in the registration (amendment 1) and is caught by the unit test of the direct construction; M7 is caught only by the unit test of the F10 merge history; M10 is caught by the unit test of the crash branch of F12 P-1, u = 145, b = 1.

Catching all 22 only means that the places with assertions were exercised by mutation: the assembly of verdict lines (`segment_one_cell_verdict_lines`, `shape_verdict_lines`, `segment_two_verdict_lines`, `segment_three_verdict_lines`, and segment four's `new_base_verdict_lines`, `side_by_side_lines`, `stub_leaf_verdict_lines`, `segment_four_summary_lines`), the stability comparison (`stability_lines`) and the read-back and aggregation of item files (`aggregate_segment_four`) are not touched by any mutation.

### Fork table (rows X1, X2, X3, X4 of `m3-leaf-death-forks.md`)

| Row of the fork sheet | Sufficient to decide / what is still missing | Can the remaining quantities flip it | Command computing it from the artifact |
|---|---|---|---|
| X4: how the I-9.4 (container number not above the smallest key) criterion is paired with borrowing (n1 / n2) | Sufficient to decide. Both flip observations were measured: n2 writes one more code 3, one more code 2 and one more retirement record per borrow (in the four cells with θ > 0, 12, 102, 222, 1505 borrows, every time; 129–33003 ppm of all write units); 8 bad-image shapes that n1 misses (7 after adding I-9.12 (Separator Key Falls Outside Child Range), of which BN1-s30 is n1's correct form), and the kind where the number is given wrongly at a split (K-2, three S-family shapes) is green under both n1's I-9.4 (container number not above the smallest key) and I-9.12 (Separator Key Falls Outside Child Range). Both sufficiency conditions are met: the extra writes were measured on histories with borrows (every θ > 0 cell of F10 has segments that borrowed; V6 only removes segments that did not borrow); the 44 shapes × two key layouts are listed in full, and the two layouts judge every shape the same | No. Segments two, three and four do not measure X4's quantities (segment four's B-stub is paired only with n1, fixed by question one n2-none: choosing B-stub leaves X4 with only n1, which is a coupling between the two rows, not a measured flip); if Q-B1, Q-B2 of segment two hit FL1 in some cell, that cell's X4 numbers are not used for the decision by the registration, but 1 + 1 per borrow, 1 retirement record and the full 5.6 table change neither with the reader nor with the cell | `grep -E 'question=(Q-X4-1\|Q-X4-2\|Q-X4-3\|Q-X4-4\|Q-X4-5\|Q-X4-6\|Q-X4-7\|G-K) ' research/results/e178-leaf-and-record-death-counts-segment-one.out`; shapes row by row `grep 'name=shape '` |
| X2: how the death where a file gets shorter is kept on disk (Z / T) | Sufficient to decide under base B-on: on H51's 51 segments, F10's borrows and absorptions (every θ > 0 cell has segments with borrows, with moved objects getting shorter, with moved objects being unlinked) and F11, resurrection (Q-X2-1, Q-X2-2) and false death outside midway (Q-X2-3, Q-X2-4) of Z and T are 0 on 48 judged cells × both W settings, false death from midway shortening under both E-L arms (Q-X2-5) is 0 too, and the four knobs do not flip; each time the range tombstone writes 1 more each of code 3 and code 2 (Q-X2-6), and the share of extra-write publishes by family is 0–766891 ppm (Q-X2-7). Neither candidate loses on resurrection or false death; the only difference left is T's write cost. Premise: the three PC-B1 segments hit literally by V2, PC-M (judged more loosely than the registration) and those midway scans of V8 are handed to the main agent to accept; removing them leaves the verdicts unchanged. Still sufficient after changing the base: in segment four, Q-Bs-4 (B-stub, n1's 12 judged cells × Z / T × three X3 arms × W) 384 rows and Q-Bt-4 (B-tsz, 24 cells × Z / T × E-I × W) 192 rows are all `differs=false`, the FL-Bx list is empty, and X2's verdict does not depend on these three bases | No: the registered quantities have all been run. The only uncovered combinations are B-stub × n2 (not paired, fixed by question one) and B-tsz × E-L (not paired by the arm definition), which are not unrun quantities | `grep -E 'question=Q-X2-[1-7] ' research/results/e178-leaf-and-record-death-counts-segment-two.out`; `grep 'name=stability question=Q-X2' …`; `grep -E 'question=(Q-Bs-4\|Q-Bt-4\|FL-Bx) ' research/results/e178-leaf-and-record-death-counts-segment-four.out` |
| X3: from which publish the whole-object tombstone takes effect (E-I / E-L) | Sufficient to decide (under base B-on): in the missed-live half, the midway inode m and extent m of E-L-w and E-L-r are 0 on 48 cells × W, and E-I by definition misses 1 record and all remaining units on every midway scan (peaks 1, 300); in the crash half, the missed live of both E-L arms is 0 over 5448 crash points, o2 is 0 under E-I and 1 at every crash point under both E-L arms (FL6 hit literally, a consequence of the arm definition), and the resurrection peaks after recovery are all 0. The missed-live half does not change after changing the base: under B-stub the midway missed live of E-L-w and E-L-r (Q-Bs-3) is 0 on 192 rows, E-I by definition has peaks 1, 300, and FL-Bs2 was not hit. B-tsz is paired only with E-I (arm definition), so choosing B-tsz means X3 takes E-I; that is a coupling between the two rows, not a measured flip | The missed-live half cannot. The crash half was not run under the two new bases (registration amendment 2, part five states that segment four does not run F12), so the crash-point missed live and o2 of the two E-L arms under B-stub have no numbers; the registration continues it only if the main agent judges that it can flip X1 | `grep -E 'question=Q-X3-[123] ' …segment-two.out`; `grep -E 'question=Q-X3-(o2\|4\|5) ' research/results/e178-leaf-and-record-death-counts-segment-three.out`; `grep -E 'question=Q-B[st]-3 ' research/results/e178-leaf-and-record-death-counts-segment-four.out` |
| X1: how a death in which the container identity disappears is kept on disk (bases: retirement record B-on / zero-entry version B-stub / zero new on-disk data B-tsz) | The correctness side is already sufficient to decide: on the same set of histories H51, F10 (borrowing and absorption) and F11 (rollback), resurrection and false death are 0 for all three bases (B-on: Q-B1, Q-B2; B-stub: Q-Bs-1, Q-Bs-2, 1080 rows over both settings; B-tsz: the inode r and extent r differences of Q-Bt-1 plus Q-Bt-2, 624 rows in total); B-tsz container r of 1–166 by cell is a consequence of the definition; missed live during unlink is 0 for both E-L arms under B-on and B-stub, and E-I (the only arm B-tsz pairs with) misses by definition; the three rollback constructions of positive controls PC-B2-direct, PC-Bs2 and PC-Bt3 all hold. B-stub's cost was measured: peak total leaves at most 66, final stub count at most 56, highest final stub share 1000000 ppm, versions hitting I-9.13 (type-2 container nonempty) 1056–670710 ppm of publications, and no version over 135 leaves. Still missing: the numbers of the three bases side by side for the flip observation "extra units written per emptying / merge" — registration amendment 2 did not register the write units of the B-stub write path, and the artifact does not contain them (B-on's retirement writes are in the segment-one `name=write_path` rows, and B-tsz adds zero to the base); the crash half (F12) under the two new bases has not been run | Yes: resurrection and false death are the same for the three bases (all 0), and the choice in this row is left with only the cost side; whether B-stub's units written per emptying and per absorption (one stub written in place, the leaves to the right not shifted left and rewritten) or B-on's retirement writes are larger has not been measured, and measuring it could flip it; likewise the crash half has no numbers | `grep -E 'question=(Q-Bs-[1-6]\|Q-Bt-[1-4]\|FL-Bx) ' research/results/e178-leaf-and-record-death-counts-segment-four.out`; `grep 'name=side_by_side' …segment-four.out`; `grep -E 'question=Q-B[12] ' …segment-two.out` |

### What it cannot answer

- Segment one runs write paths only, without scanning; the numbers for X2, X3 and base B-on are in segments two and three, those for B-stub and B-tsz in segment four.
- How many units the B-stub write path writes per emptying and per absorption was not measured (registration amendment 2 did not register this quantity, and the artifact only has the stub count), so the write costs of the three bases cannot be put side by side; this item is still missing from X1's flip observations. B-stub is paired only with n1 and B-tsz only with E-I, so B-stub × n2 and B-tsz × E-L have no numbers; the crash half (F12) under the two new bases was not run.
- That resurrection and false death are 0 for all three bases is a number in this model: B-stub's readings are identical line by line to B-on's, and B-tsz × Z to B-off-ret's, both corroborating the same conclusion: in a model where readers select the newest by registration 5.4 and knob W only takes ∞ and 23, the three ways of writing deaths make readers read the same things; the model has no allocator, no extent tree, no concurrency, and no growing code-2 root (B-stub's total leaf count is at most 66 leaves, never reaching 135).
- The `replay.sh` line of segment four (`segment-four --no-reuse`) was not rerun: inferred at about 35 minutes on 6 threads of this host (inferred: the assembling run of 997 items took about 15 minutes on 14 threads), which the executor could not finish in the foreground, so it was handed to the main agent; the determinism of segment four rests only on three things: every item identical scan by scan over two runs, 20 sampled items recomputed byte-identically in another item directory, and V-Bt2.
- "Z, T and both E-L arms are 0 on 48 cells" is a number in this model: readers select the newest by registration 5.4, knob W only takes ∞ and 23, and the model has no allocator, no extent tree, no concurrency; the same inode number has only one object birth generation in these histories (numbers are not reissued after crash recovery), so the "object birth generation = b" term of Z has no sampling point (last paragraph of registration section nine).
- The cell where FL6 is hit (o2 = 1 at the crash points of both E-L arms) is a consequence of the arm definition, not a new fact measured from the model; E-I's o2 = 0 likewise.
- PC-M and PC-T of segment two are judged more loosely than the registration states (see the row of the invalidation clauses), so those two controls only prove "the two paths agree" and "Z, g0 do not move", not the exact increments written in the registration.
- The bad images are the 44 enumeration points of registration 5.6 plus one sparse layout, not the domain; "n1 misses 8 shapes" speaks only of the numbers in this table.
- 1 + 1 extra writes and 1 retirement record per borrow is a consequence of the n2 arm definition, and the model writes exactly this number because it follows the definition; it proves that the model wrote by the definition (pinned by a6), not the cost of n2 once landed in `crates/`. The denominator of the share is the measurement basis of reading 8; other bases (including the extent tree, allocation records, accounting) were not computed.
- Borrowing, absorption, n1 and retirement records on this page were all absent from `crates/` when the apparatus ran; `crates/singlefs-core/src/inode_tree.rs` now has absorption and borrowing (n1: the separator key takes the smallest key of the right leaf's current version), `crates/singlefs-core/src/retirement_tree.rs` now has retirement records, and the third sentence of I-9.4 (container number not above the smallest key) has been deleted (n2 not chosen, not landed); at the time, the differential against `crates/` covered only the θ = 0 write path (S1–S3) and the then I-9.4 (container number not above the smallest key), I-9.12 (Separator Key Falls Outside Child Range) judgements (S6), all written as the same judgements in the apparatus after reading the source and test assertions, with `crates/` neither compiled nor run, and the differential was not redone.
- Leaf-container capacity 16 is a test-only knob the model set for itself; the two θ settings are points given by the registration, and the number of borrows varies with the deletion order (those V6 segments had no borrow at all).
- What amendment 1 records (on F11, PC-B2 cannot measure B-off-lin's container m) holds on all of F11 in segment two as well: 48 items `b_off_lin_container_m_peak_after_rollback=0`, and the main agent decided it is not a result.
- The wall clock of segment four is not in the artifact: from the first to the last item file the modification times span about 15 minutes (this host, 14 threads; the second host's shard did not run and this host computed it instead).

### Paths and conclusion register

| # | Path | How it computes | Source placement | Which shared items it reads |
|---|---|---|---|---|
| 1 | Write-path simulation | One `prepare` per publish: change leaf containers, files, tombstone trees and the retirement tree by the rules of registration 5.1, and count the units written this time | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:1725 prepare`, `:1941 adopt` | format constants (233, 815, 959, 135, 32634); merge policy M(θ); batch size |
| 2 | Closed-form publish count (a1) | F1 = 3 + ⌈(n − 1) ÷ 50⌉ + 2(n − 1); F8 = 3 + (k + 1) + [overwrite before deletion] + 1 + max(1, ⌈units at intent ÷ b⌉); F3 copies those two segments of the artifact of E177 (Resurrection Count Without Per-Tree Tombstones) | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:3094 run_history` (the judgements at lines 3107, 3126, 3157), `:2972 E177_MIXED_RANDOM_PUBLISHES_AND_REFUSALS` | batch size 144, 50 created per step; F3 uses the same splitmix64 |
| 3 | Hand count of the one-borrow small history (a6) | The output of `anchors2.py` in registration section thirteen (5 moved, left 12 right 11, right leaf's smallest 22) copied as an assertion | No implementation: the hand count is in the registration, and the apparatus only has the assertion `:3489 single_borrow_anchor_holds` | capacity 16, θ = 8, the formula for the number borrowed |
| 4 | Newest selection by the group index (main path) | Walk each group's version list from newest to oldest and take the first candidate | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:2404 select_by_the_group_index` | candidate predicate `:2335 is_candidate` via `:2351 enters_the_selection`; reader judgement `:2470 judge_reader` |
| 5 | Newest selection by a full scan (reference path) | Scan every unit on disk once and take the maximum (write order, unit ordinal) per group | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:2426 select_by_a_full_scan` | candidate predicate `:2335 is_candidate` via `:2351 enters_the_selection`; reader judgement `:2470 judge_reader` |
| 6 | Stub closed form (a11) | first stub = 3 + ⌈(n − 1) ÷ 50⌉ + 2(min(n, 2 × capacity) − 1), versions with a stub = total publications − first stub + 1 (registration 13.3 `anchors3.py`) | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:5873 first_stub_publish_and_versions_with_a_stub` | 50 created per step; the reading that the second leaf is the first to be emptied |
| 7 | B-stub write path and the original write path (V-Bs1) | Run each write path once and compare txg, live-leaf digest, data-unit digest, merge actions and scans version by version | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:5883 pairing_holds` | write-path simulation (the same `prepare` as path 1, differing only in how deaths are written) |
| 8 | Hand counts of PC-Bs1, PC-Bt2 | Record count, record by record, in the version read back after erasing the stub (PC-Bs1 inode r); unit-by-unit comparison of the two supply rules (PC-Bt2 extent r) | `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:5506 hand_count_of_records_a_stub_erasure_brings_back`, `:5609 hand_count_of_units_the_oldest_copies_bring_back` | newest selection of the reference path (path 5); the current version of the tombstone tree |

Line numbers are those of the segment-four apparatus (source file sha256 `93193bd5929e7867b7d18ea5c8b0d6af9e32ceffed5c866f073ba8e5c0d1dfa6`, which produced the segment-four artifact); the artifacts of segments one to three were produced by the version before the change (`e0512d3f…1897`), and after the change segment one reran byte-identically.

Cross-path comparison assertions: path 1 against path 2 is pinned in the a1 judgement of `run_history` (segment two 612 times, 0 violations); path 1 against path 3 in `single_borrow_anchor_holds` (segment-one artifact line 1525 `holds=true`); path 4 against path 5 in `:4367 group_index_equals_the_full_scan` and `:4135 positive_control` (segment two `v3_last_scan_paths_agree=true`, positive controls' `paths_agree` all true; segment three `v3_paths_agree=true`); path 6 against path 1 in the a11 judgement (passes in the unit test; segment-four artifact `assertion=A11_first_stub_publish_and_versions_with_a_stub judged=4 violated=0`); path 7 in V-Bs1 (segment four 498 items, 0 violations); path 8 against the readers in `closed_form_holds` of PC-Bs1, PC-Bt2. Shared-item questions: paths 1 and 2 share the batch size and the inode-creation step, so when both are wrong together a1 still matches, and what guards against it is S3's differential against `crates/`; paths 4 and 5 share `is_candidate` and `judge_reader`, so when the candidate predicate or the reader judgement is written wrongly both paths go wrong together, and what guards against it is the expectations of PC-Z, PC-EL, PC-ELr, PC-O written in the registration and the unit tests of M1–M22; the two write paths of path 7 share everything in `prepare` except how deaths are written, so when `prepare` is wrong both go wrong together and the pairing still holds, and what guards against it is S1–S3 and a1; path 8 uses the reference path's newest selection, so when newest selection is wrong the hand count and the readers go wrong together, and what guards against it is V3's two-path comparison.

### Decisions affected

| Decision item | Relationship | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 6 | Supports | 2026-10-21 unaffected: archiving the records of the previous round only changed paths pointing at deleted files to bare file names, conclusions and artifacts unchanged; previously, 2026-10-18 unaffected: the unit tests and mutations section added by name the artifacts of the 10-15 resume-key samples and the item-key red proof, and six rows of the paths and conclusion register gained file paths, conclusions and numbers unchanged |
| D8 (Core Index Structure) settled item 16 | Preparation | 2026-10-21 unaffected: archiving the records of the previous round only changed paths pointing at deleted files to bare file names, conclusions and artifacts unchanged; previously, 2026-10-18 unaffected: the unit tests and mutations section added by name the artifacts of the 10-15 resume-key samples and the item-key red proof, and six rows of the paths and conclusion register gained file paths, conclusions and numbers unchanged |
| D18 (Information Carried in a Block) settled item 11 | Supports | 2026-10-21 unaffected: archiving the records of the previous round only changed paths pointing at deleted files to bare file names, conclusions and artifacts unchanged; previously, 2026-10-18 unaffected: the unit tests and mutations section added by name the artifacts of the 10-15 resume-key samples and the item-key red proof, and six rows of the paths and conclusion register gained file paths, conclusions and numbers unchanged |
| D18 (Information Carried in a Block) settled item 24 | Preparation | 2026-10-21 unaffected: archiving the records of the previous round only changed paths pointing at deleted files to bare file names, conclusions and artifacts unchanged; previously, 2026-10-18 unaffected: the unit tests and mutations section added by name the artifacts of the 10-15 resume-key samples and the item-key red proof, and six rows of the paths and conclusion register gained file paths, conclusions and numbers unchanged |
| D18 (Information Carried in a Block) settled item 10 | Preparation | 2026-10-21 unaffected: archiving the records of the previous round only changed paths pointing at deleted files to bare file names, conclusions and artifacts unchanged; previously, 2026-10-18 unaffected: the unit tests and mutations section added by name the artifacts of the 10-15 resume-key samples and the item-key red proof, and six rows of the paths and conclusion register gained file paths, conclusions and numbers unchanged |
| D23 (Journal Role & Format) settled item 14 | Does not affect | 2026-10-21 unaffected: archiving the records of the previous round only changed paths pointing at deleted files to bare file names, conclusions and artifacts unchanged; previously, 2026-10-18 unaffected: the unit tests and mutations section added by name the artifacts of the 10-15 resume-key samples and the item-key red proof, and six rows of the paths and conclusion register gained file paths, conclusions and numbers unchanged |

## Historical Versions

### 2026-10-08: page created, segment one run

The preregistration was written before the apparatus; the executor wrote amendment 1 after the unit tests ran and before the artifacts (on F11, PC-B2 cannot measure B-off-lin's container m, PC-B2-direct added, and M3's sensitive sampling point now carried by it). Segment one ran, and the X4 row of the fork sheet is sufficient to decide; the apparatus, unit tests and mutations of segments two and three were written but not run.

### 2026-10-08: readings of segment four

The main agent ran segment four (artifact `research/results/e178-leaf-and-record-death-counts-segment-four.out`; the second host's shard was refused by the memory wrapper and this host computed it instead), and the executor checked the verdict lines, the invalidation and failure clauses and V-Bt2 row by row: resurrection and false death of B-stub and B-tsz are 0, and the verdicts of X2 and X3 do not change with the base; the status changed from "partly run (segments one to three)" to "run (four segments)". The correctness side of X1 is sufficient to decide; the write-cost side lacks B-stub's write units per emptying (not registered). The segment-four line of `replay.sh` was not rerun.

### 2026-10-08: readings of segments two and three, apparatus of segment four

The main agent ran segments two and three (artifacts `research/results/e178-leaf-and-record-death-counts-segment-two.out`, `…-segment-three.out`), and the executor checked the verdict lines and the invalidation and failure clauses row by row, so the rows X2 and X3 are sufficient to decide under base B-on; the status changed from "partly run (segment one)" to "partly run (segments one to three)". Registration amendment 2 (experiment designer) added the two base arms B-stub and B-tsz as segment four, and the fork sheet gained an X1 row; following it, the executor added the segment-four entry to the apparatus, 44 unit tests and a mutation table of 22 entries, and wrote amendment 3 (PC-Bt2-mid); the segment-four artifact was not run. The unit-test and mutation outputs were replaced by `…-unit-tests-2026-10-08.out`, `…-mutations-2026-10-08.out`, and the two old files are kept.
