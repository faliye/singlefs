## E137 Performance Disparity Across Mapping Key Forms —— Run (2026-09-11)

[中文](137-映射key形态的性能差距.md) · **English** · [日本語](137-マッピングキー形態の性能格差.ja.md)

**The question asked**: After seeing the cost rows of E136 (Cost Rows for Mapping Key Forks), the user stated "costs do not reveal performance disparity"—what E133 (Format Cost of Mapping Key Candidates), E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms), and E136 (Cost Rows for Mapping Key Forks) reported were format quantities (entry width, fanout, levels, leaf-level bytes), without touching key ordering.
E137 (Performance Disparity Across Mapping Key Forms) measures three things using a real ordered leaf structure: blocks written per fsync (formula from D25 (Target Workload Priority)), mapping tree blocks COW'd per unit moved by background workloads, and node misses per uniform random read.
Three arms and two configurations share names and definitions with E136 (Cost Rows for Mapping Key Forks): logical identity + write-order key (`k1_seq`), hybrid path (`k1_seq_class_reuse`), tightened class reuse of write-order key (`class_reuse_tagged`); `all` (all pointers enter mapping) and `code1_only` (Code 1 only).
**Reports numbers only, does not judge winner/loser**: its numbers enter the cost row of D19 (Block Pointer Structure & Width Budget) settled item 6, forks decided by the user.

**Fixed pre-run**: `e137-preregistration.md` (written prior to harness; postscript 1 before first run, adding definitions only; postscript 2 after unit tests ran for the first time and before first artifact run, see "This is a Counting Model, Not an Implementation").

**Re-run commands** (`exact` mode, byte-by-byte comparison against archived artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E137
# Run harness directly:
cd scripts/experiment-apparatus && cargo run --release --bin e137_map_key_performance
```

Code `scripts/experiment-apparatus/e137_map_key_performance.rs`,
original output `e137-map-key-performance-2026-09-11.out` (single run, 388 lines, last line `emitted=374`; 38s on host).

### This is a Counting Model, Not an Implementation

Random source is PRNG with fixed seed, no I/O ⇒ running N times is deterministically identical byte-for-byte; evidence strength derives from **15 unit tests + 27 mutations all caught**, plus three controls:
- **Cross-harness gate**: capacity of map leaf / map internal / extent leaf / extent internal equals cell-by-cell the rows `fanouts` and `code1_only_fanouts` in E136 (Cost Rows for Mapping Key Forks) archived artifact (one unit test reads that artifact directly);
- **Positive control** (run for every arm and configuration): 0 misses after warm-up when cache fits all nodes; misses equal walked nodes when cache is 1 node;
- **Discriminative power control**: changing Code 1 key of class reuse write-order key to per-file order (width unchanged) changed map leaves from 2 055 to 20 129 in multi-stream append batch 10, and from 3 782 to 563 in file delete.

```
E7RESULT name=positive_control config=all arm=k1_seq all_cached_misses=0 one_node_cache_misses=60000 walked_nodes=60000
E7RESULT name=positive_control config=all arm=k1_seq_class_reuse all_cached_misses=0 one_node_cache_misses=60000 walked_nodes=60000
E7RESULT name=positive_control config=all arm=class_reuse_tagged all_cached_misses=0 one_node_cache_misses=60000 walked_nodes=60000
E7RESULT name=positive_control config=code1_only arm=k1_seq all_cached_misses=0 one_node_cache_misses=60000 walked_nodes=60000
E7RESULT name=positive_control config=code1_only arm=k1_seq_class_reuse all_cached_misses=0 one_node_cache_misses=60000 walked_nodes=60000
E7RESULT name=positive_control config=code1_only arm=class_reuse_tagged all_cached_misses=0 one_node_cache_misses=60000 walked_nodes=60000
E7RESULT name=discrimination_control multistream_map_leaves_own=2055 multistream_map_leaves_swapped=20129 file_delete_map_leaves_own=3782 file_delete_map_leaves_swapped=563
```

Small scenarios in unit tests (2 files × 2 units, 2 streams, leaf capacity 2, fanout 2) were hand-calculated. ⚠️ First run had one red cell in unit tests: multi-stream append cell was hand-calculated per registered reading (each stream appends to the end of its already filled file), while harness version 1 had each stream append to a new file—modified the harness, keeping registration untouched; same audit revealed snapshot delete overwrites also diverged from registration (registration was within one file, harness version 1 was full-image), restored to single file, added row `snapshot_delete_global` (postscript 2, all prior to first artifact run).

### Measurement Basis

- **Blocks written per fsync** = distinct extent leaves touched + distinct internal nodes at each level + distinct map leaves touched + distinct internal nodes at each level + 1 root slot + 1 record (formula from D25 (Target Workload Priority)); under `all` config, each extent node COW'd deletes an old Code 2 mapping entry and inserts a new one, counted in the mapping tree half.
- **Image**: 4 096 files × 256 units = 1 048 576 Code 1 units; 16 streams take turns appending, each stream writing full files under its own name in sequence, one checkpoint every 64 writes; physical placement = global order of writes. Each arm constructs one copy using identical operation sequence.
- **Key ordering**: Code 1 under logical identity + write-order key and hybrid path ordered by (object, object birth generation, anchor offset, birth epoch, write order), entries of same file adjacent, new and old versions adjacent; Code 1 under class reuse write-order key ordered by (birth txg, write order), entries written in same checkpoint adjacent. Code 2 enters mapping only under `all` config: logical identity + write-order key ordered by (level, key interval lower bound, birth epoch), other two ordered by (birth txg, birth seq).
- **Leaves**: ordered array, splits when full—starts new leaf when appending to the rightmost leaf end, splits 50/50 otherwise; no merging on empty delete. Internal levels approximated as "leaf position / fanout raised to level power", all three arms using identical approximation.
- **Workloads**: `seq_append` sequentially appends to a new file; `multistream_append` 16 streams alternate appending to the end of the file each last filled in image; `rand_overwrite` random overwrite within one existing file; `rand_overwrite_global` random overwrite across full image; `metaheavy_create` consecutively creates single-unit small files; each of these five evaluated at batch 1 and batch 10, 20 000 writes each. Four background workloads: `file_delete` randomly deletes 256 files, one checkpoint per deletion; `snapshot_delete` takes snapshot then overwrites 20 000 times in one file, releasing old versions retained by snapshot by birth epoch; `snapshot_delete_global` same as `snapshot_delete` but overwrites span full image; `relocation` relocates 20 000 units in placement order; latter three do one checkpoint every 256 units.
- **Cache**: LRU, counted by nodes, capacity sized at 10% / 30% / 60% of total node count in the two trees of logical identity + write-order key under same config, all three arms using same absolute capacity (2 425 / 7 275 / 14 550 nodes under `code1_only`, 2 414 / 7 244 / 14 488 under `all`). Each read traverses extent tree root-to-leaf; stale-hint portion (190‰ and 370‰) then traverses mapping tree root-to-leaf; warm-up 100 000 reads uncounted, 100 000 measured.

### Results (Copied Verbatim from `e137-map-key-performance-2026-09-11.out`)

**Images for each arm across two configurations**:

```
E7RESULT name=image config=all arm=k1_seq map_leaf_capacity=206 map_internal_fanout=129 extent_leaf_capacity=122 extent_internal_fanout=164 map_leaves=8342 extent_leaves=15642 total_nodes=24147
E7RESULT name=image config=all arm=k1_seq_class_reuse map_leaf_capacity=206 map_internal_fanout=121 extent_leaf_capacity=122 extent_internal_fanout=152 map_leaves=8275 extent_leaves=15642 total_nodes=24091
E7RESULT name=image config=all arm=class_reuse_tagged map_leaf_capacity=296 map_internal_fanout=148 extent_leaf_capacity=149 extent_internal_fanout=152 map_leaves=7128 extent_leaves=12032 total_nodes=19291
E7RESULT name=image config=code1_only arm=k1_seq map_leaf_capacity=206 map_internal_fanout=129 extent_leaf_capacity=122 extent_internal_fanout=164 map_leaves=8445 extent_leaves=15642 total_nodes=24251
E7RESULT name=image config=code1_only arm=k1_seq_class_reuse map_leaf_capacity=206 map_internal_fanout=129 extent_leaf_capacity=122 extent_internal_fanout=164 map_leaves=8445 extent_leaves=15642 total_nodes=24251
E7RESULT name=image config=code1_only arm=class_reuse_tagged map_leaf_capacity=296 map_internal_fanout=159 extent_leaf_capacity=149 extent_internal_fanout=164 map_leaves=3543 extent_leaves=12032 total_nodes=15674
```

The four tables in "Results" are extracted row-by-row from artifact (written once when seeds 1 / 2 / 3 are identical), not computed separately.
**Blocks written per fsync, configuration `code1_only`** (includes 1 root slot + 1 record; written once when seeds 1 / 2 / 3 are identical):

| Workload | Batch | Logical ID + Write-Order Key | Hybrid Path | Class Reuse Write-Order Key |
|---|---|---|---|---|
| `seq_append` | 1 | 8.000 | 8.000 | 8.000 |
| `seq_append` | 10 | 8.104 | 8.104 | 8.088 |
| `multistream_append` | 1 | 8.024 | 8.024 | 8.012 |
| `multistream_append` | 10 | 27.501 | 27.501 | 17.683 |
| `rand_overwrite` | 1 | 8.000 | 8.000 | 8.033 |
| `rand_overwrite` | 10 | 12.502 / 12.212 / 12.705 | 12.502 / 12.212 / 12.705 | 10.396 / 10.383 / 10.447 |
| `rand_overwrite_global` | 1 | 8.000 | 8.000 | 9.978 / 9.976 / 9.977 |
| `rand_overwrite_global` | 10 | 42.931 / 42.925 / 42.849 | 42.931 / 42.925 / 42.849 | 43.494 / 43.504 / 43.447 |
| `metaheavy_create` | 1 | 8.000 | 8.000 | 8.000 |
| `metaheavy_create` | 10 | 8.104 | 8.104 | 8.088 |

**Blocks written per fsync, configuration `all`** (includes 1 root slot + 1 record; written once when seeds 1 / 2 / 3 are identical):

| Workload | Batch | Logical ID + Write-Order Key | Hybrid Path | Class Reuse Write-Order Key |
|---|---|---|---|---|
| `seq_append` | 1 | 10.009 | 9.624 | 9.229 |
| `seq_append` | 10 | 10.164 | 9.778 | 9.351 |
| `multistream_append` | 1 | 10.803 | 9.960 | 9.250 |
| `multistream_append` | 10 | 30.921 | 29.995 | 19.515 |
| `rand_overwrite` | 1 | 11.000 / 12.000 / 12.000 | 10.000 | 9.820 / 9.825 / 9.817 |
| `rand_overwrite` | 10 | 15.267 / 17.046 / 17.111 | 14.766 / 13.950 / 14.835 | 13.805 / 13.806 / 13.822 |
| `rand_overwrite_global` | 1 | 11.411 / 11.404 / 11.412 | 12.084 / 12.083 / 12.085 | 12.902 / 12.899 / 12.901 |
| `rand_overwrite_global` | 10 | 55.511 / 55.470 / 55.408 | 57.809 / 57.810 / 57.732 | 56.316 / 56.328 / 56.273 |
| `metaheavy_create` | 1 | 10.009 | 9.624 | 9.229 |
| `metaheavy_create` | 10 | 10.164 | 9.778 | 9.351 |

**Background workloads, mapping tree blocks amortized per unit** (leaf + internal; seeds 1 / 2 / 3):

| Configuration | Workload | Logical ID + Write-Order Key | Hybrid Path | Class Reuse Write-Order Key |
|---|---|---|---|---|
| `code1_only` | `file_delete` | 0.019 | 0.019 | 0.065 |
| `code1_only` | `snapshot_delete` | 0.027 | 0.027 | 0.062 / 0.066 / 0.066 |
| `code1_only` | `snapshot_delete_global` | 0.421 / 0.422 / 0.424 | 0.421 / 0.422 / 0.424 | 0.190 |
| `code1_only` | `relocation` | 0.083 / 0.083 / 0.081 | 0.083 / 0.083 / 0.081 | 0.015 |
| `all` | `file_delete` | 0.033 / 0.032 / 0.032 | 0.036 | 0.136 / 0.135 / 0.136 |
| `all` | `snapshot_delete` | 0.027 | 0.027 | 0.117 / 0.117 / 0.121 |
| `all` | `snapshot_delete_global` | 0.410 / 0.413 / 0.413 | 0.411 / 0.411 / 0.413 | 0.351 / 0.350 / 0.349 |
| `all` | `relocation` | 0.084 / 0.083 / 0.080 | 0.082 / 0.083 / 0.080 | 0.018 |

**Uniform random read, node misses per read** (cache capacity is permille of total node count in the two trees of logical identity + write-order key under same config, all three arms using same absolute capacity; seeds 1 / 2 / 3):

| Configuration | Cache ‰ | Stale Hint ‰ | Logical ID + Write-Order Key | Hybrid Path | Class Reuse Write-Order Key |
|---|---|---|---|---|---|
| `code1_only` | 100 | 190 | 1.053 / 1.056 / 1.057 | 1.053 / 1.056 / 1.057 | 0.999 / 1.002 / 1.001 |
| `code1_only` | 100 | 370 | 1.228 / 1.233 / 1.234 | 1.228 / 1.233 / 1.234 | 1.154 / 1.159 / 1.158 |
| `code1_only` | 300 | 190 | 0.775 / 0.773 / 0.776 | 0.775 / 0.773 / 0.776 | 0.616 / 0.617 / 0.616 |
| `code1_only` | 300 | 370 | 0.940 / 0.940 / 0.943 | 0.940 / 0.940 / 0.943 | 0.719 / 0.722 / 0.722 |
| `code1_only` | 600 | 190 | 0.384 / 0.384 / 0.386 | 0.384 / 0.384 / 0.386 | 0.075 / 0.075 / 0.074 |
| `code1_only` | 600 | 370 | 0.517 / 0.517 / 0.519 | 0.517 / 0.517 / 0.519 | 0.089 / 0.091 / 0.089 |
| `all` | 100 | 190 | 1.053 / 1.056 / 1.057 | 1.054 / 1.057 / 1.057 | 1.013 / 1.016 / 1.016 |
| `all` | 100 | 370 | 1.228 / 1.233 / 1.233 | 1.228 / 1.233 / 1.234 | 1.191 / 1.196 / 1.195 |
| `all` | 300 | 190 | 0.775 / 0.773 / 0.777 | 0.776 / 0.774 / 0.777 | 0.661 |
| `all` | 300 | 370 | 0.938 / 0.939 / 0.941 | 0.940 / 0.940 / 0.942 | 0.828 / 0.831 / 0.832 |
| `all` | 600 | 190 | 0.384 / 0.383 / 0.386 | 0.386 / 0.385 / 0.386 | 0.194 / 0.196 / 0.196 |
| `all` | 600 | 370 | 0.513 / 0.514 / 0.515 | 0.516 / 0.514 / 0.516 | 0.307 / 0.309 / 0.308 |

**Complete lines for representative cells** (seed 1)—Code 1 only, multi-stream append batch 10, global random overwrite batch 1; all pointers in mapping, sequential append batch 1, global random overwrite batch 10:

```
E7RESULT name=fsync config=code1_only workload=multistream_append batch=10 arm=k1_seq seed=1 fsyncs=2000 extent_leaves=10.153 extent_internal=2.703 map_leaves=10.090 map_internal=2.555 blocks=27.501
E7RESULT name=fsync config=code1_only workload=multistream_append batch=10 arm=k1_seq_class_reuse seed=1 fsyncs=2000 extent_leaves=10.153 extent_internal=2.703 map_leaves=10.090 map_internal=2.555 blocks=27.501
E7RESULT name=fsync config=code1_only workload=multistream_append batch=10 arm=class_reuse_tagged seed=1 fsyncs=2000 extent_leaves=10.126 extent_internal=2.529 map_leaves=1.027 map_internal=2.000 blocks=17.683
E7RESULT name=fsync config=code1_only workload=rand_overwrite_global batch=1 arm=k1_seq seed=1 fsyncs=20000 extent_leaves=1.000 extent_internal=2.000 map_leaves=1.000 map_internal=2.000 blocks=8.000
E7RESULT name=fsync config=code1_only workload=rand_overwrite_global batch=1 arm=k1_seq_class_reuse seed=1 fsyncs=20000 extent_leaves=1.000 extent_internal=2.000 map_leaves=1.000 map_internal=2.000 blocks=8.000
E7RESULT name=fsync config=code1_only workload=rand_overwrite_global batch=1 arm=class_reuse_tagged seed=1 fsyncs=20000 extent_leaves=1.000 extent_internal=2.000 map_leaves=1.999 map_internal=2.978 blocks=9.978
E7RESULT name=fsync config=all workload=seq_append batch=1 arm=k1_seq seed=1 fsyncs=20000 extent_leaves=1.000 extent_internal=2.000 map_leaves=2.009 map_internal=3.000 blocks=10.009
E7RESULT name=fsync config=all workload=seq_append batch=1 arm=k1_seq_class_reuse seed=1 fsyncs=20000 extent_leaves=1.000 extent_internal=2.000 map_leaves=2.009 map_internal=2.615 blocks=9.624
E7RESULT name=fsync config=all workload=seq_append batch=1 arm=class_reuse_tagged seed=1 fsyncs=20000 extent_leaves=1.000 extent_internal=2.000 map_leaves=2.006 map_internal=2.223 blocks=9.229
E7RESULT name=fsync config=all workload=rand_overwrite_global batch=10 arm=k1_seq seed=1 fsyncs=2000 extent_leaves=9.998 extent_internal=10.571 map_leaves=20.673 map_internal=12.267 blocks=55.511
E7RESULT name=fsync config=all workload=rand_overwrite_global batch=10 arm=k1_seq_class_reuse seed=1 fsyncs=2000 extent_leaves=9.998 extent_internal=10.601 map_leaves=22.524 map_internal=12.685 blocks=57.809
E7RESULT name=fsync config=all workload=rand_overwrite_global batch=10 arm=class_reuse_tagged seed=1 fsyncs=2000 extent_leaves=9.999 extent_internal=10.474 map_leaves=22.189 map_internal=11.653 blocks=56.316
```

**Background workloads** (Code 1 only, seed 1) and **random read** (Code 1 only, cache 60%, stale hint 190‰, seed 1):

```
E7RESULT name=background config=code1_only workload=file_delete arm=k1_seq seed=1 units=65536 checkpoints=256 map_leaves=778 map_internal=513 extent_leaves=1216 map_blocks_per_unit=0.019
E7RESULT name=background config=code1_only workload=file_delete arm=k1_seq_class_reuse seed=1 units=65536 checkpoints=256 map_leaves=778 map_internal=513 extent_leaves=1216 map_blocks_per_unit=0.019
E7RESULT name=background config=code1_only workload=file_delete arm=class_reuse_tagged seed=1 units=65536 checkpoints=256 map_leaves=3782 map_internal=539 extent_leaves=1003 map_blocks_per_unit=0.065
E7RESULT name=background config=code1_only workload=snapshot_delete_global arm=k1_seq seed=1 units=19789 checkpoints=78 map_leaves=8110 map_internal=235 extent_leaves=0 map_blocks_per_unit=0.421
E7RESULT name=background config=code1_only workload=snapshot_delete_global arm=k1_seq_class_reuse seed=1 units=19789 checkpoints=78 map_leaves=8110 map_internal=235 extent_leaves=0 map_blocks_per_unit=0.421
E7RESULT name=background config=code1_only workload=snapshot_delete_global arm=class_reuse_tagged seed=1 units=19789 checkpoints=78 map_leaves=3599 map_internal=177 extent_leaves=0 map_blocks_per_unit=0.190
E7RESULT name=background config=code1_only workload=relocation arm=k1_seq seed=1 units=20000 checkpoints=79 map_leaves=1498 map_internal=177 extent_leaves=0 map_blocks_per_unit=0.083
E7RESULT name=background config=code1_only workload=relocation arm=k1_seq_class_reuse seed=1 units=20000 checkpoints=79 map_leaves=1498 map_internal=177 extent_leaves=0 map_blocks_per_unit=0.083
E7RESULT name=background config=code1_only workload=relocation arm=class_reuse_tagged seed=1 units=20000 checkpoints=79 map_leaves=146 map_internal=158 extent_leaves=0 map_blocks_per_unit=0.015
E7RESULT name=read config=code1_only arm=k1_seq cache_permille=600 cache_nodes=14550 stale_permille=190 seed=1 reads=100000 misses=38491 misses_per_read=0.384
E7RESULT name=read config=code1_only arm=k1_seq_class_reuse cache_permille=600 cache_nodes=14550 stale_permille=190 seed=1 reads=100000 misses=38491 misses_per_read=0.384
E7RESULT name=read config=code1_only arm=class_reuse_tagged cache_permille=600 cache_nodes=14550 stale_permille=190 seed=1 reads=100000 misses=7510 misses_per_read=0.075
```

### Reading These Tables

1. **Under Code 1 only, logical identity + write-order key matches hybrid path cell-by-cell**: Code 1 key, leaf capacity, and fanout are identical across both arms (reported as such in artifact).
2. **When all pointers enter mapping, these two differ only in the Code 2 half**: sequential append, multi-stream append, small file creation: hybrid path writes 0.39 to 0.93 fewer blocks per fsync; single-file overwrite: hybrid path also writes fewer (batch 1 saves 1 to 2 blocks); global random overwrite: hybrid path writes 0.67 to 2.34 more blocks; file delete: 0.032–0.033 vs 0.036 per unit, remaining three background workloads differ by ≤ 0.002; random read misses differ by ≤ 0.003.
3. **The major disparity lies in Code 1 key ordering, i.e., class reuse write-order key vs the other two, with direction flipping across workloads** (numbers cited in item 3 are taken from Code 1-only branch; same direction under all pointers in mapping):
   - Where it saves: multi-stream append batch 10 writes 36% fewer blocks per fsync (27.501 vs 17.683)—10 writes in batch fall into 10 files, per-file ordered mapping touches 10 leaves, birth-ordered mapping touches only the rightmost leaf; relocation costs 0.015 vs 0.081–0.083 per unit; snapshot delete after global overwrite costs 0.190 vs 0.421–0.424; single-file overwrite batch 10 costs 10.383–10.447 vs 12.212–12.705.
   - Where it costs: global random overwrite batch 1 writes 25% more blocks per fsync (9.976–9.978 vs 8.000)—in per-file ordering old and new versions reside in same leaf, in birth ordering new version lands in rightmost leaf while old version is elsewhere, touching an extra leaf and its spine; file delete costs 0.065 vs 0.019 per unit; single-file snapshot delete costs 0.062–0.066 vs 0.027.
   - Sequential append and small file creation are virtually identical across all three (batch 1 are all 8.000).
4. **Disparity in random read is not driven by key ordering**: in uniform random read every leaf has equal probability of access, ordering has no effect; disparity stems from node counts—class reuse write-order key has narrower entries (E136 (Cost Rows for Mapping Key Forks) pointer to Code 1 is 85 vs 109, map entry 55 vs 79), two trees total 15 674 nodes vs 24 251 under Code 1 only, so the same absolute capacity covers a larger fraction: at cache 60%, misses per read are 0.074–0.075 vs 0.384–0.386 (stale hint 190‰).

### Questions It Cannot Answer

- Does not calculate wall-clock time or CPU; block count is the measurement basis recognized by D25 (Target Workload Priority); barrier frequency is independent of key form, not reported.
- **Physical placement = write order**: relocation moves contiguous segments in placement order, which equals birth order in this model—this represents the most favorable condition for class reuse write-order key in relocation and snapshot deletion; how much this advantage shrinks after aging decouples placement from birth order is not measured.
- Pool is 1 048 576 units, tree heights identical across three arms; the one-level difference reported by E136 (Cost Rows for Mapping Key Forks) at pool scale was not layered in.
- Builds extent tree and mapping tree only; accounting tree, allocation record tree, and inode tree also COW upon each publish identically across arms, not modeled.
- Read path traverses mapping only upon stale hint for data unit; the portion where extent node pointers themselves have stale hints requiring mapping traversal under `all` config was not in registration, not measured.
- Single-file snapshot delete retains only 256 old versions, released in a single checkpoint.
- How often Code 2 / Code 3 nodes are moved, and the cost of disposing C163 (identical write sequence across fixed point rounds defeats tie-breaking), remain unquantified (same as E136 (Cost Rows for Mapping Key Forks)).

### Mutations

27 mutations all caught. Key ones: rightmost append splits 50/50, 50/50 split reports only new leaf, first key does not update on smaller key, internal level grouping coarsened by one step, root slot and record each count 0, cache eviction evicts newest and capacity off by one, both Code 1 keys modified in one segment, hybrid path Code 1 partially takes birth order, multi-stream appends to new files, current file selected as first (in small image one stream has only one file, uncaught; caught by sample point in dedicated unit test with multiple files per stream), Code 2 old entries missed during deletion, Code 2 entries flushed even in Code 1 only, relocation dirties extent leaf, snapshot retention criterion inverted, single-file snapshot changed to full-image, dirty leaves not cleared after publish, batch boundaries offset by one.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 6 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 8 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-11

- Established and completed run. During first unit test run, multi-stream append cell went red; discovered two discrepancies between harness and preregistration (multi-stream appending to new files, snapshot delete overwriting full image); aligned harness back to registration and added `snapshot_delete_global`, recorded in preregistration postscript 2, all prior to first artifact run.
