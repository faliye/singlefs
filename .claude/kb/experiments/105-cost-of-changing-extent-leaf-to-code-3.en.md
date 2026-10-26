## E105 Cost of Changing Extent Leaf to Code 3 — Run (2026-09-05, pure arithmetic, 6 unit tests / 5 mutants caught): **After changing extent tree leaves from code 2 index leaves to code 3 packed record units, byte ratio for single update and point lookup is determined by tree height on both sides: constantly 1.000 on segments where Arm B is one level shorter than Arm A (wide records N ∈ [38417, 77028] and [7529537, 15097488], occupying 13% of logarithmic axis); on remaining N 1 + 1/tree height (1.5 / 1.333 / 1.25); across 1e6 entries batch 10 / 100 / 1000 dispersed updates more expensive by 1.44× / 1.66× / 1.78×, on 1e4 entries at batch saturation 1.000; three node header tiers identical number per cell, narrow records isomorphic**

[中文](105-extent叶改码3的代价.md) · **English** · [日本語](105-extentリーフのコード3化コスト.ja.md)

**This tests the price of opposed Arm B in Section 5 of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalized proposal round 5**: Arm B adds write sequences only to code 2 / 3, leaving data unit headers untouched, at the cost of changing extent tree leaves to code 3 packed record units (data extents reached along reconstructed extent tree). Round 4 backward leg 6.3 demanded "measure rather than estimate the cost of Arm B", which E105 (Cost of Changing Extent Leaf to Code 3) measures directly; determination residual of Arm B is measured by `wseq_meta_only` arm of E104 (Current Version Determination in Scan Rebuild).

**Preparation**: Price of opposed Arm B in C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalized proposal round 5. The final decision took Arm A (write sequence on data unit header); Arm B was not adopted, so this number does not support any decision item; if the recomputability criterion in D21 (Boundary Between Authoritative and Derived States) settled item 10 should redetermine extent leaf modality in the future, the cost tier is read from this page.

### Cited Items (Copied Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D8 (Core Index Structure) settled items 2 / 3 | Node 16 KiB; extent record = key 24 (locality 8 + inode 8 + offset 8) + position pointer 59 = 83 |
| D18 (Information Carried in a Block) settled items 7 / 11 | Index node header three tiers 58 / 67 / 76, packed record unit header 93 (all values prior to C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalization; values after adding write sequence 10 shown in row for "C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalized round 5 P1") |
| D23 (Journal Role & Format) settled item 1 Option A | Update cost = dirty leaf + all ancestors rewritten once each (same algorithm as E103 (Update Cost of Inode Packing Path 1)) |
| C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalized round 5 P1 | Class identity segment adds 10-byte write sequence ⇒ node header 68 / 77 / 86, container header 103 |
| D4 (Checksum Location) | Unit 32 KiB |

### Arms and Criteria (Locked Before Run, Unchanged After Run)

Arm A = current form: leaf is code 2 index leaf (16 KiB, fanout 196); Arm B = Arm B: leaf is 32 KiB code 3 container (wide record 393 entries per container, narrow record 628 entries), internal nodes unchanged. Nodes touched by k dispersed updates calculated by L × (1 − (1 − 1/L)^k), w = 2.
1. Single update (k = 1) calculates "written nodes / units count × size × w" for each form, N ∈ {1e4, 1e6, 1e8}, node header 68 / 77 / 86 per cell; container header 107 (node header three tiers still 68 / 77 / 86, changed to 72 / 81 / 90 under C288 (collateral changes from mapped key settlement did not land in other decisions) item ⑦).
2. Batch k ∈ {1, 10, 100, 1000}; saturation control: when k is much larger than nodes in a level, touched nodes converge to node count of that level.
3. Point lookup: tree height node reads (Arm A) vs tree height − 1 node reads + 1 unit read of 32 KiB (Arm B).
4. Positive control: when container takes 16 KiB with identical record width, leaf level costs of both forms must be equal (same ruler).
5. Negative control: k = 0 ⇒ 0 on both sides.
6. Narrow record 52 (round 4 backward leg 6.4 estimate: object ID 8 + birth generation 8 + anchor offset 8 + position entry 14 × 2) runs separate tier to observe sensitivity.

### Measurements (2026-09-05, artifact 116 lines, `emitted=116`)

| N | Arm A Tree Height | Arm B Tree Height | Single Update Arm B / Arm A | Point Lookup Bytes Arm B / Arm A |
|---|---|---|---|---|
| 1e4 | 2 | 2 | 1.500 | 1.500 |
| 1e6 | 3 | 3 | 1.333 | 1.333 |
| 1e8 | 4 | 4 | 1.250 | 1.250 |

Three node header tiers (68 / 77 / 86) and two record widths (83 / 52) yield identical ratio per cell across these three N: at leaf level Arm B is 32 KiB vs Arm A's 16 KiB, tree heights identical, ratio determined purely by tree height.
**Ratios at the three sample points are properties of sampling, not properties of the model** (hit by round 5 backward leg 6.4): tree heights on both sides are step functions of N, Arm A steps up at 196 × 196^k, Arm B steps up at 393 × 196^k; in the intermediate interval Arm B is one level shorter than Arm A — wide records at N ∈ [38417, 77028] and [7529537, 15097488], narrow records at [61349, 123088] and [12024209, 24125248] (`name=equal_height` lines), each occupying ln(393 / 196) / ln(196) = 13.2% of logarithmic axis. On those N single update byte ratio is constantly 1.000, point lookup bytes identical with Arm B reading one fewer node. Analytical lower bound: container fanout ≥ leaf fanout ⇒ Arm B at most one level shorter ⇒ ratio ≥ 1, no parameters yield below 1. When N ≤ 196 Arm A is one level, Arm B is two levels (single container still has one code 2 root node above), ratio 2.0.
Batch updates equal at saturation: 1e4 entries, k = 1000 touches all leaves / containers on both sides, 26 containers of 32 KiB = 52 leaves of 16 KiB, byte ratio 1.000 (1.141 at k = 100).
1e6 entries, header 68, wide record batch: k = 1 → 1.333 (98304 vs 131072 bytes), k = 10 → 1.444, k = 100 → 1.662, k = 1000 → 1.782; larger k puts Arm B at greater disadvantage as more containers are touched, 32 KiB each.
Geometry: Arm A leaf fanout 196, 5103 leaves; Arm B 393 entries per container, 2545 containers, two internal levels.

- Criterion 4: Leaf level costs of Arm B match Arm A per cell across three headers; Criterion 5: k = 0 yields 0 on both sides; Criterion 2: diff between touched nodes and level node count < 0.5% at k = 10⁶.
- Assertions nailing absolute values: `fanouts_are_absolute` (196 / 393 / 628 / internal 196), `single_update_is_pinned` (98304 vs 131072; containers 393 × 2545; leaves 5103; tree height 3 vs 3), `equal_height_segments_are_pinned` (segment bounds 38417 / 77028 / 7529537 / 15097488 and narrow records 61349 / 123088, within segment write bytes equal, read one fewer node, outside segment returns to 1 + 1/tree height).

### Mutation Testing: 5 Mutants Caught

Includes "fanout calculates container per node", "tree height undercounts one level", "touched nodes unsaturating", "code 3 side misses internal levels", "point lookup misses that unit read".
⚠️ In round 1 "fanout calculates container per node" **turned zero tests red**: single update bytes do not vary with container count, lacking assertions nailing container fanout and container count. Caught after adding three absolute values 393 / 2545 / 5103; artifact unchanged (same md5).

### What It Cannot Answer

Pure arithmetic, 0 file operations. Does not answer wall clock, does not answer cache hits; does not answer message buffering cell of D11 (Message Buffers in Index Nodes) (both arms alter its source inventory, question of E56 (Message Buffer Gains vs Epsilon)); does not answer determination residual under Arm B when extent leaves lost (E104 (Current Version Determination in Scan Rebuild) `wseq_meta_only` arm); k updates modeled as uniformly dispersed, clustered update patterns on few containers unrun (under that pattern Arm B touches fewer containers, ratio returns to single update tier, never below 1).

### Measurement Basis and Rerun

- Code `scripts/experiment-apparatus/e105_extent_leaf_packed.rs` (`cargo run --release --bin e105-extent-leaf-packed`, run under `research/`), raw output `e105-extent-leaf-packed-2026-09-12.out` (116 lines, ending line `emitted=116`; version with container header 107, differs from prior version `e105-extent-leaf-packed-2026-09-05.out` in results/ only in config lines; mutation rerun `e105-mutate-2026-09-12.log` caught all 5).
- Mutation table `scripts/mutation/tables/e105_extent_leaf_packed.tsv`, log `e105-mutate-2026-09-05.log` (`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e105-extent-leaf-packed e105_extent_leaf_packed.rs ../mutations/e105_extent_leaf_packed.tsv`).
- Rerun comparison: line for E105 (Cost of Changing Extent Leaf to Code 3) in `bash scripts/experiment-apparatus/replay.sh`.
- Container header constant named `PACKED_UNIT_HEADER_BYTES` in source, identical to registered name in kb ⇒ bound by format-constants cell in `.claude/gate.d/code-source-discipline.sh` gate.
  (When run on 2026-09-05 it was temporarily named `PACKED_HDR_WITH_WSEQ` to prevent gate treating 103 as drift from then-registered 93;
  on day C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) finalized registered value updated to 103, restored to registered name on 2026-09-06, artifact byte-for-byte unchanged.)

### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D21 (Boundary Between Authoritative and Derived States) | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 11 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D4 (Checksum Location) settled item 5 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D23 (Journal Role & Format) settled item 1 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D11 (Message Buffers in Index Nodes) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-06

- Container header constant restored to registered name `UNIT_HDR_PACKED` (previously `PACKED_HDR_WITH_WSEQ`), bound by format constants gate;
  value remains 103, 6 unit tests and 5 mutations all green, artifact byte-for-byte unchanged.
  Basis: condition written at project initiation — "restore to registered name after C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) landing updates registered value to 103" — satisfied.

### 2026-09-05

- Project initiated and completed. Basis: round 4 backward leg 6.3 "measure rather than estimate cost of Arm B".
- Mutation "fanout calculates container per node" turned zero tests red in round 1, patched by pinning container fanout 393, container count 2545, Arm A leaves 5103 three absolute values; artifact unchanged.
- Round 5 backward leg 6.4 hit "1.25–1.78× is selected range": added `equal_cost_segments` and `name=equal_height` six artifact lines (110 lines to 116 lines), unit test pinned segment boundaries; conclusion changed from "more expensive by 1.25–1.78×" to "determined by tree height, 1.000 on equal height segments".

