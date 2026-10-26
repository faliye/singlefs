## E157 Count Model for Two Clauses on Concurrent Line 1 — Partially run (first segment on 2026-09-22, deterministic model; rerun on 2026-10-04 with header 311 → 487 and named entries 67 → 64: artifact and 22 unit tests produced, mutation table of 10 entries (3 deleted, 3 added) run in full both before and after the change, 10/10 caught after the change; only fork-sheet row 1 was run — quantities 1.1a/b, 1.2a/b/c, 1.3a/b/c, controls P1/P2a/P2b, anchors A1–A5/B1–B3; C491 blocking row 2 (with multiple records, which named entry holds the shared intrinsic block is unsettled) was not touched in this segment)

[中文](157-并行线一两条条款的计数模型.md) · **English** · [日本語](157-並行ライン-1-の二つの条項の計数モデル.ja.md)

<!-- doc-lint:not-numbers A1 A3 A4 A5 B1 B2 B3 B4 B5 B6 P1 P2a P2b P3 P4 M1 M2 M3 M4 M5 M6 M13 F1 F2 F3 F4 F5 F6 F7 F8 V1 V2 V3 V4 V5 -->
In this document, A1, A3–A5, B1–B6, P1–P4, M1–M13, F1–F8, V1–V5, and the item written in the body as "anchor Jia-2", are all the anchor / control / mutation / failure-and-voiding clause numbers that `e157-preregistration.md` registered for itself before the run; the registration slot lives in that file, and they are not kb D / E / C / I numbers (the number of that item in the original registration is the letter A followed by the digit 2; it is not written here as that bare number — it collides with an edge case of the registration of D9 (Encryption), which uses the same symbol for "the system configuration is plaintext"; the two have different meanings, and writing the bare number would collide these two unrelated things).

**Question**: Pre-run registration `e157-preregistration.md` (728 lines). The subject under test is one of the two unsettled clauses blocking parallel line 1 of the milestone "overwrite, release, rollback and reuse" — `UndecidedClauseBlockingMoreThanOneDataUnit::ExtentLeafKeyOffsetUnit`, i.e. C490 (Offset field in extent leaf key lacks defined unit), corresponding to row 1 of the fork sheet `m2-line-one-clauses-forks.md`. A pure arithmetic count model: it does not run the publish path of `crates/` and does not measure wall-clock time; two arms: 1-Jia (the third key segment is written as `n × C`, a file byte offset) and 1-Yi (the third key segment is written as `n` itself, a unit index).

### What this segment did

Apparatus: `scripts/experiment-apparatus/e157_parallel_line_one_clauses.rs` (the new `[[bin]]` was appended to the end of `scripts/experiment-apparatus/Cargo.toml`).

1. **Format constants checked item by item against `crates/singlefs-format/src/lib.rs` (the check required by halt clause V6, looked up fresh before producing the artifact)**: `NODE_BYTES=16384`, `EXTENT_NODE_HEADER_BYTES=163` (my own computation as `86+2×24+29`, without calling the format function — format keeps only two scalars, `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE` / `INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT`; the name `index_node_header_bytes` has a private copy in each of format, the checker, and every experiment apparatus, none calling the others, code review item 11); this apparatus's `JOURNAL_NAMED_ENTRIES_PER_RECORD` is the literal 64 (`:81`), and the compile-time assertion starting at `:82` uses the pure function `named_entries_per_record_for_header_bytes` (`:76`) to check it is consistent with the header width, `EXTENT_LEAF_RECORD_BYTES=24+88=112`, `JOURNAL_HEADER_BYTES=487` (the current value after the 2026-10-04 change of header 311 → 487, `e157-r2-prereg.md`), `JOURNAL_RECORD_BYTES=4096`, `JOURNAL_NAMED_ENTRY_BYTES=56`, `DATA_UNIT_PAYLOAD_OFFSET=134`. Looked up fresh after the apparatus was changed on 2026-10-04 and before the artifact was produced (command `grep -nE '^pub const (JOURNAL_HEADER_BYTES|JOURNAL_NAMED_ENTRY_BYTES|JOURNAL_NAMED_ENTRIES_PER_RECORD):' crates/singlefs-format/src/lib.rs`, output verbatim):

```
223:pub const JOURNAL_HEADER_BYTES: u64 = 487;
226:pub const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
229:pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;
```

   Verbatim equal to this apparatus's constants, not a single mismatch; V6 does not fire. Before the apparatus change (apparatus 311 / 67, `crates/` 487 / 64) V6 was in the fired state; that run served only as a comparison baseline (`replay.sh E157` (Count Model for Two Clauses on Concurrent Line 1) is byte-identical against the old artifact) and draws no conclusion.
2. **Quantities 1.1a / 1.1b** (node reads to locate one page, cold walk and mounted state): all 14 sample points of `DATA_UNIT_COUNT_SAMPLE_POINTS` × `NODE_BYTES_SAMPLE_POINTS` (4096/16384/65536) × `INTERNAL_ENTRY_BYTES_SAMPLE_POINTS` (110/136) × the page set for each N (`{p0, p_cross_unit_boundary, p_leaf_boundary_143_144 (only when N>144), p_last_unit}`) × both arms, 588 rows in total (`name=locate_page` in `e157-parallel-line-one-clauses-2026-09-22.out`).
3. **Quantities 1.2a / 1.2b / 1.2c** (leaf count, tree height, total node count): the same 14×3×2 sample table × two arms, 168 rows (`name=tree_shape`).
4. **Quantities 1.3a (differential; the registration downgraded it, so it is not a criterion) / 1.3b / 1.3c**: the four `(C₀, C₁)` pairs of `CAPACITY_PAIR_SAMPLE_POINTS` × 14 values of N × two arms, 112 rows (`name=capacity_conversion`). Quantity 1.3a coincides verbatim with "the four sets of numbers computed at registration time" in section 4 of the registration (unit test `量1_3a_与登记第四节命令三的四组数逐字重合`).
5. **Controls P1, P2a, P2b**: each run against both arms (`name=p1_control` 4 rows, `name=p2a_zero_point` / `name=p2b_effective` 8 rows each); all three move in the expected direction of section 5.2 of the registration (see the section "Result lines copied verbatim from the artifact").
6. **Anchors A1–A5 (7-Jia) / B1–B3 (7-Yi)**: three classes — `name=anchors`, `name=b5_geometry`, `name=b6_straddle_page` — 8 rows in total, each pinned down by a unit test.
7. **Geometry sensitivity (section 8.3 of the registration)**: the cells of 1.1/1.2 already ran through all three `NODE_BYTES` levels (4096/16384/65536) and both internal entry widths (110/136), not merely "at least one direction"; the capacity sample table of 1.3b/1.3c itself contains three directions: `Δ=+1` (32634 → the day 28 are reserved), `Δ=-2` (32634 → 32632), and `Δ` reversed (the day 28 are reserved → 32634). **Self-test of discriminative power** (`name=discrimination_self_check`): moving the leaf-count threshold to `N=40`, between `NODE_BYTES=4096` (leaf capacity 35) and `16384` (leaf capacity 144), the leaf count of the two levels goes from 1 to 2, `threshold_between_them=true` (F6 does not fire).
8. **Mutations M1–M6, M13**: `scripts/mutation/tables/e157_parallel_line_one_clauses.tsv`, `cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e157-parallel-line-one-clauses e157_parallel_line_one_clauses.rs ../mutations/e157_parallel_line_one_clauses.tsv` → the 2026-09-22 run **caught all 7, 0 invalid, 0 not-red**, closing with "restored, baseline still all green" (`/tmp/e157-mutate-output.txt` verbatim in the report); after the table was changed to 10 entries (M1–M6, M13, M17–M19) on 2026-10-04, the post-change run **caught all 10, 0 invalid, 0 not-red** (`e157-mutate-after-2026-10-04-h487.log`, see the section "Rerun"). M2 was missed on the first run (not a single test turned red) — because the original assertions all compared the return value of `extent_tree_height` against the return value of **another call to the same function**, so when M2 broke both sides together it could not be seen (`test-discipline.md` "only letting multiple arms compare against each other"); after adding an assertion that pins absolute values (`树高绝对值_两片叶到三层各钉一个数`), M2 turned red.
9. `naming-lint.sh` passes on the new file (0 violations; 18 single-letter / letter-plus-digit identifiers were changed along the way, including the loop variable `n` changed to `total_data_units`, and anchor test function names of the form `a1`/`b2`/`p1` replaced with `锚点甲一`/`锚点乙二`/`对照一`).

### Result lines copied verbatim from the artifact

Artifact: `research/results/e157-parallel-line-one-clauses-2026-10-04-h487.out` (902 lines, `E7RESULT name=done emitted=902`, equal to the 902 from `grep -c '^E7RESULT'` on the file; `replay.sh` registers this one). It differs from `e157-parallel-line-one-clauses-2026-09-22.out` only in two fields on the `name=anchors` line, `a1_named_entries_per_record=` and `b2_named_entries_per_record_check=` (67 → 64):

```
$ diff e157-parallel-line-one-clauses-2026-09-25-h311-replay.out research/results/e157-parallel-line-one-clauses-2026-10-04-h487.out
1c1
< E7RESULT name=anchors a1_named_entries_per_record=67 a2_named_entry_bytes=56 a3_extent_leaf_record_bytes=112 a3_extent_node_header_bytes=163 a4_payload_capacity_today=32634 b1_leaf_capacity_16384=144 b2_named_entries_per_record_check=67 b3_payload_capacity_today=32634 b3_payload_capacity_reserve28=32635
---
> E7RESULT name=anchors a1_named_entries_per_record=64 a2_named_entry_bytes=56 a3_extent_leaf_record_bytes=112 a3_extent_node_header_bytes=163 a4_payload_capacity_today=32634 b1_leaf_capacity_16384=144 b2_named_entries_per_record_check=64 b3_payload_capacity_today=32634 b3_payload_capacity_reserve28=32635
```

The whole lines quoted in ①②③ are found verbatim in the new artifact (`grep -c -F` gives 1 line each).

**① Quantities 1.1a / 1.1b: the two arms are equal point by point on all 294 `(N, node_bytes, internal_entry_bytes, page_label)` combinations, 0 differences** (checked by script, see "Rerun"). One cell as an example (N=145, the page that crosses leaves):

```
E7RESULT name=locate_page arm=jia n=145 node_bytes=16384 internal_entry_bytes=110 page_label=p_leaf_boundary_143_144 page=1147 cold_walk_node_reads=3 mounted_leaf_reads=2
E7RESULT name=locate_page arm=yi n=145 node_bytes=16384 internal_entry_bytes=110 page_label=p_leaf_boundary_143_144 page=1147 cold_walk_node_reads=3 mounted_leaf_reads=2
```

**② Quantities 1.2a / 1.2b / 1.2c: the two arms are equal point by point on all 84 `(N, node_bytes, internal_entry_bytes)` combinations, 0 differences**:

```
E7RESULT name=tree_shape arm=jia n=145 node_bytes=16384 internal_entry_bytes=110 leaves=2 height=2 node_count=3
E7RESULT name=tree_shape arm=yi n=145 node_bytes=16384 internal_entry_bytes=110 leaves=2 height=2 node_count=3
```

**The result of ①② is not an apparatus fault**: the three controls P1 (after the geometry shrinks, the leaf count must rise and the tree height must not decrease), P2a (at `Δ=0` both arms restore everything correctly), and P2b (at `Δ≠0` at least one arm reports wrongly restored keys) all move in the expected direction of section 5.2 of the registration (unit tests `对照一_几何变小之后叶数必须涨树高必须不减`, `对照二_容量不变时两臂的两个量都必须还原全对`, `对照三_容量变化时两个量至少一个臂报出还原不对的key` all green). Per registered F3: **this is a legitimate result; record honestly that "these two cells ①② cannot separate the arms"**, and row 1 is instead judged by ③.

**③ Quantity 1.3b: both arms are 0 on all 84 `(C₀, C₁, N)` combinations with `Δ≠0`, likewise inseparable** (checked by script):

```
E7RESULT name=capacity_conversion arm=jia n=145 c0=32634 c1=32635 delta=1 unit_index_correct_count=1 byte_interval_correct_count=0 aad_anchor_correct_count=145
E7RESULT name=capacity_conversion arm=yi n=145 c0=32634 c1=32635 delta=1 unit_index_correct_count=145 byte_interval_correct_count=0 aad_anchor_correct_count=1
```

The same line (`n=145 c0=32634 c1=32635`) also carries the readings of quantity 1.3a: for 1-Jia the "unit index restored correctly" count is 1, for 1-Yi 145 — consistent with the numbers computed at registration time in section 4 of the registration; quantity 1.3a has been downgraded to a differential quantity and does not enter the judgment.

**③ Quantity 1.3c: the only quantity in this segment that separates the two arms stably on all sample points, and independent of `Δ`** — the AAD-anchor recomputation of 1-Jia always equals `N`, that of 1-Yi always equals 1 (checked point by point over 16 groups = 4 `(C₀, C₁)` pairs × 4 values of N, 0 exceptions; unit test `量1_3c_一甲恒对一乙只对第零条`). When `N≥2` the difference `N−1 ≥ 1` always points to 1-Jia.

**Geometry / Δ sensitivity**: the "inseparable" of ①② holds on all three `NODE_BYTES` levels (4096/16384/65536) and both internal entry widths (110/136), and does not flip with geometry; the "inseparable" of 1.3b and the "1-Jia always wins" of 1.3c in ③ also hold in all three directions `Δ=+1`, `Δ=-2`, and `Δ` reversed, and do not flip with the sign or magnitude of `Δ`. One item worth recording on its own: at `Δ=-2` (32634 → 32632), `N=20736/20737`, quantity 1.3a of 1-Jia (the differential quantity) changes from "always 1" to "always 16316" — this is the result of the feasible interval of `⌊n·C₀/C₁⌋=n` expanding, when `Δ<0`, to `n<C₁/|Δ|=16316` as `n·|Δ|<C₁`; it is not a criterion, and is recorded here only to state honestly that this number varies with `(C₀,C₁,N)` (the reverse of the "two measurement bases coincide on some level" warned about in `.claude/rules/mutation-sampling.md` "class five": here it is the same measurement basis giving different numbers on different levels).

### Rerun

```
bash scripts/experiment-apparatus/replay.sh E157
```

Reports "byte-identical" (run live on 2026-10-04, after the registration row was repointed to the new artifact: `byte-identical 1 / timing-only differs 0 / mismatch 0 / cannot run 0 / conclusion assertion misses 0`). Code `scripts/experiment-apparatus/e157_parallel_line_one_clauses.rs` (`cargo run --release --bin e157-parallel-line-one-clauses`), raw output `research/results/e157-parallel-line-one-clauses-2026-10-04-h487.out` (a single run, not a splice of multiple rounds; the 2026-09-22 and 2026-09-25 copies differ from it only in two fields on the `name=anchors` line, see the section "Result lines copied verbatim from the artifact"). Unit tests: `cd scripts/experiment-apparatus && cargo test --release --manifest-path scripts/experiment-apparatus/Cargo.toml --bin e157-parallel-line-one-clauses` → `22 passed` (1 added on 2026-09-24, and on 2026-10-04 one added for the pure function giving how many entries each record holds). The mutation table has 10 entries: M1–M6, M13, M17 (entries per record counted one byte too many), M18 (entries per record counted one byte too few), M19 (header width shifted one byte within the same band); on 2026-10-04 three entries were deleted — header width back to 307, header width over the upper boundary, header width over the lower boundary (their header-width values make the entries per record differ from 64, hitting the compile-time assertion at `:82`, so they do not compile: a registered inference, never run separately; replaced by the three entries M17–M19). The two full-table reruns, before the change (old apparatus, old table) and after the change, were run by the main agent, with logs `e157-mutate-before-2026-10-04-h487.log` (old tree, 10 entries in the table) and `e157-mutate-after-2026-10-04-h487.log` (10 entries in the table); command: `cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e157-parallel-line-one-clauses e157_parallel_line_one_clauses.rs ../mutations/e157_parallel_line_one_clauses.tsv`.

```
$ for p in before after; do f=research/results/e157-mutate-$p-2026-10-04-h487.log; echo "$p: $(for s in '✅' '⏭' '❌' '💥' '⚠️' '⏱' '🧱'; do printf '%s=%s ' "$s" "$(grep -c "^$s" $f)"; done)"; done
before: ✅=8 ⏭=2 ❌=0 💥=0 ⚠️=0 ⏱=0 🧱=0 
after: ✅=10 ⏭=0 ❌=0 💥=0 ⚠️=0 ⏱=0 🧱=0 
$ grep -E '^(计数：|已还原)' e157-mutate-after-2026-10-04-h487.log
计数：内存撞顶 0 条（上限 2G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条
已还原，基线仍全绿
$ grep -E '^(⏭|✅) +\[M1[456]_' e157-mutate-before-2026-10-04-h487.log | cut -c1-110
✅ [M14_头宽退回307] 红：tests::named_entries_per_record_remainder_matches_head_311
⏭  [M15_头宽过上边界] 变异导致编译失败，本条无效（不计入盲区，也不算命中）
⏭  [M16_头宽过下边界] 变异导致编译失败，本条无效（不计入盲区，也不算命中）
```

Before the change **8 caught, 2 invalid, 0 not-red**; after the change **10 caught, 0 invalid, 0 not-red**; the columns of each sum to 10, equal to the 10 rows of each table, both end with "restored, baseline still all green", and memory-cap hits, timeouts, unschedulable, and squeezed-out are all 0. Registered voiding clause V2 (M17, M18, or M19 not red): all three are "caught" — M17 and M18 turn red in `named_entries_per_record_for_header_bytes_straddles_the_sixty_four_item_band`, M19 turns red in `named_entries_per_record_remainder_matches_head_487` ⇒ **V2 does not fire**; V7 (after changing the constants, an existing mutation gets a worse status): comparing the two logs mutation by mutation by name, the existing M1–M6, M13, 7 in total, are "caught" both times; present before and absent after are M14, M15, M16; present after and absent before are M17, M18, M19 (`diff` shows only these six differences) ⇒ **V7 does not fire**. The invalid column did not grow: the 2 before the change (M15, M16) already failed to compile on the old tree before the change (header 311, named entries 67) — zero coverage that existed before and was not caused by this constant change; after deleting these three and replacing them with M17–M19 the invalid column is 0. M14 measured "caught" before the change (the registered inference said it also would not compile; that half of the M14 inference was wrong); M14–M16 were not run separately on the new tree (header 487, named entries 64), and the reason for deleting them is still the registered inference (the header-width values make entries per record differ from 64, hitting the compile-time assertion at `:82`).
Self-test of discriminative power (unit test, done by hand once, restored): changing the expectation for a header width of 512 in the unit test of how many entries each record holds to 63 turns red (`left: 64`, `right: 63`); changing the expectation for a header width of 513 to 64 turns red (`left: 63`, `right: 64`).

Category counts (aligned with "after the artifact is out, first self-check that it is complete"):

```
grep -oE 'name=[a-z0-9_]+' research/results/e157-parallel-line-one-clauses-2026-10-04-h487.out | sort | uniq -c
```

```
      1 name=anchors
      6 name=b5_geometry
      1 name=b6_straddle_page
    112 name=capacity_conversion
      1 name=config
      1 name=discrimination_self_check
      1 name=done
    588 name=locate_page
      4 name=p1_control
      8 name=p2a_zero_point
      8 name=p2b_effective
      2 name=real_baseline
    168 name=tree_shape
      1 name=unit_count_boundary
```

(The fourteen classes total `1+6+1+112+1+1+1+588+4+8+8+2+168+1 = 902`, equal to `emitted=902`; all 14 classes present, none missing.)

### Path and conclusion registry

For the form see `.claude/rules/format-evolution.md` "the 'Path and conclusion registry' of an experiment page". Quantity 1.3a (differential quantity) and geometry anchors B1/B5/B6 each have two paths that share no code:

| # | Path | How computed | Source placement | Shared items read |
|---|---|---|---|---|
| 1 | **Independent Python formula registered before the run** (command 3 of section 13-Yi of the registration) | Hand-written expressions such as `sum(1 for n in range(N) if (n*C_write)//C_read==n)` (the original registration uses subscripts `C₀`/`C₁`); the four sets of numbers were computed at registration time, without reading this apparatus | No implementation: only a one-off shell session recorded in the pre-run registration file, not an executable in the repository (the pre-run registration was archived at 31128273, original text in the parent commit of that commit) | The capacity `C₀=32634` of the day it was written, the capacity `C₁=32635` of the day it was read, `N∈{1,2,145,20737}` |
| 2 | **This apparatus's `reconstruction_outcome`** | A Rust function implementing the formulas of 1.3a/1.3b in section 6.1 of the registration, written independently, not calling path 1 | `scripts/experiment-apparatus/e157_parallel_line_one_clauses.rs:236` `fn reconstruction_outcome` | `N∈{1,2,145,20737}` (the same sample points as path 1), plus all `(C₀,C₁)` combinations |
| 3 | **Independent Python formula registered before the run** (command 1 / command 2 of section 13-Yi of the registration) | Hand-written expressions such as `(16384-163)//112`, `(nb-163)//110` computing leaf capacity and fanout | No implementation: like path 1, only a one-off shell session recorded in the pre-run registration file, not an executable in the repository (the pre-run registration was archived at 31128273, original text in the parent commit of that commit) | `EXTENT_NODE_HEADER_BYTES=163`, `EXTENT_LEAF_RECORD_BYTES=112` |
| 4 | **This apparatus's `extent_leaf_capacity` / `extent_internal_fanout`** | Rust functions implementing the same formula, written independently | `scripts/experiment-apparatus/e157_parallel_line_one_clauses.rs:124` `fn extent_leaf_capacity`, `scripts/experiment-apparatus/e157_parallel_line_one_clauses.rs:130` `fn extent_internal_fanout` | `EXTENT_NODE_HEADER_BYTES=163`, `EXTENT_LEAF_RECORD_BYTES=112` (the same pair of constants as path 3) |

Cross-path comparison assertions: `量1_3a_与登记第四节命令三的四组数逐字重合` (:660) `assert_eq!`s the output of path 2 against the four sets of numbers of path 1 verbatim; `锚点乙一_一叶一百四十四条` (:543) and `锚点乙五_三档node_bytes下的叶容量与扇出` (:550) `assert_eq!` the output of path 4 against the numbers of path 3 verbatim. ⚠️ The shared items of the two paths are exactly the same (both read the same definitions of `EXTENT_NODE_HEADER_BYTES`/`EXTENT_LEAF_RECORD_BYTES`); this cross-check verifies "whether two independent transcriptions of the formula contain a slip", not "whether the formula itself is right" — whether the formula itself is right is guaranteed separately by the item-by-item check against `crates/singlefs-format/src/lib.rs` (item 1 of "What this segment did").

### What it cannot answer

- **Row 2 of the fork sheet** (C491 (Which record names shared endogenous block when multiple records undefined)): not a word of it was touched in this segment; left for the second segment.
- **Quantities 1.4, 2.1–2.4**: the registration explicitly says "side quantities, not run once the judgment is reached" / belong to row 2; not run in this segment.
- **Which extent-tree internal node entry width (110/136) is correct**: zero hits in `crates/`; this apparatus treats it as a pure knob and sweeps two sample points, choosing no default; the "inseparable" conclusion of ①② at these two sample points is unaffected by it (both levels were tried).
- **The relation between the "locate by ordinal position" read used in `crates/` and E157 (Count Model for Two Clauses on Concurrent Line 1)**: section 3 of the registration already states that this is a workaround while the clause is unsettled; E157 (Count Model for Two Clauses on Concurrent Line 1) models the byte-offset read per the milestone acceptance criteria; the two are not the same thing, and this segment did not re-verify that statement.
- **F5 (67 entries per record vs 71 entries per record, measurement bases at odds)**: belongs to B4 of section 7-Yi of the registration, beyond the scope of this segment (this segment goes only up to B3), neither fired nor checked.
- **Execution order of the halt-clause check**: it should have been finished before running the mutation table and the artifact (the pre-run registration and the executor definition both say so); this segment actually did the item-by-item cross-check of the format constants only after the mutation table had run and the artifact had been written to disk (see item 1 of "What this segment did") — the check itself passed and does not affect any number in this segment, but the order is wrong, recorded here honestly, not concealed.

### Affected decisions

| Decision item | Relation | Review |
|---|---|---|
| D8 (Core Index Structure) settled item 3 | Supports | 2026-10-24 unaffected: apparatus admission fingerprint header added, conclusions and artifact data byte-for-byte unchanged; prior 2026-10-18 unaffected: the previous round's artifact was archived into the repository (recoverable from git history), the body references were changed to keep only the file name, the conclusion and the numbers unchanged |
| D8 (Core Index Structure) settled item 11 | Stock-up | 2026-10-24 unaffected: apparatus admission fingerprint header added, conclusions and artifact data byte-for-byte unchanged; prior 2026-10-18 unaffected: the previous round's artifact was archived into the repository (recoverable from git history), the body references were changed to keep only the file name, the conclusion and the numbers unchanged |
| D23 (Journal Role & Format) settled item 12 | Stock-up | 2026-10-24 unaffected: apparatus admission fingerprint header added, conclusions and artifact data byte-for-byte unchanged; prior 2026-10-18 unaffected: the previous round's artifact was archived into the repository (recoverable from git history), the body references were changed to keep only the file name, the conclusion and the numbers unchanged |
| D23 (Journal Role & Format) settled item 17 | Stock-up | 2026-10-24 unaffected: apparatus admission fingerprint header added, conclusions and artifact data byte-for-byte unchanged; prior 2026-10-18 unaffected: the previous round's artifact was archived into the repository (recoverable from git history), the body references were changed to keep only the file name, the conclusion and the numbers unchanged |
| D9 (Encryption) | Not affected | 2026-10-24 unaffected: apparatus admission fingerprint header added, conclusions and artifact data byte-for-byte unchanged; prior 2026-10-18 unaffected: the previous round's artifact was archived into the repository (recoverable from git history), the body references were changed to keep only the file name, the conclusion and the numbers unchanged |
| D8 (Core Index Structure) settled item 11 | Supports | 2026-10-24 unaffected: apparatus admission fingerprint header added, conclusions and artifact data byte-for-byte unchanged; prior 2026-10-18 unaffected: the previous round's artifact was archived into the repository (recoverable from git history), the body references were changed to keep only the file name, the conclusion and the numbers unchanged |

⚠️ The "affected decisions" four rows are judged: D8 (Core Index Structure) settled item 3 was upgraded to supports on 2026-09-23 (user settlement: the offset segment takes the file byte offset, and the settlement and basis paragraphs of this sub-item were rewritten per this settlement); D8 (Core Index Structure) settled item 11 stays at stock-up — E157 (Count Model for Two Clauses on Concurrent Line 1) only swept the internal entry width as a knob at two levels, neither establishing nor overturning it; D23 (Journal Role & Format) settled item 12 and settled item 17 stay at stock-up — they are inputs to the second segment (C491 (Which record names shared endogenous block when multiple records undefined)), and the second segment has run no quantity at all.

## Historical Versions

### 2026-10-04: record header 311 → 487, named entries 67 → 64 rerun (first segment, question sheet row 4)

- Before, after, and basis: see the 2026-10-04 entry in the section "E157 (Count Model for Two Clauses on Concurrent Line 1)" of [experiments-history.md](../experiments-history.md).

### 2026-09-24: record header 307 → 311 rerun (first segment, question sheet row 4)

- Before: the apparatus constant `JOURNAL_HEADER_BYTES` was still 307, while `crates/singlefs-format/src/lib.rs` was already 311 (halt clause V6 was in the fired state; this change lifted it).
- After: once `e157-r1-prereg.md` fixed the criteria, the constant was changed to 311, and one new unit test pinning the remainder 33 was added (written as an addition, for the same reason as the compile-time overflow pit hit by the fifth rerun registration of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds)); none of the first segment's judgment cells (1.1a/1.1b, 1.2a/1.2b/1.2c, 1.3a/1.3b/1.3c) reads the header width, only anchors A1 and B2 read it, via the channel "one record holds 67 entries". The artifact is byte-identical (902 lines, empty `diff`), and also byte-identical to the existing `e157-parallel-line-one-clauses-2026-09-22.out`, with no new save and no change to `replay.sh`. 21 unit tests all green (1 added), all 10 mutations caught (M14/M15/M16 added, run once before and once after the constant change, 0 invalid, 0 not-red). **Question sheet row 4 records "unchanged"**. The second segment (C491 (Which record names shared endogenous block when multiple records undefined), fork sheet row 2) is not in this run: D23 (Journal Role & Format) settled item 17 is already settled, and this run does not resume because of the header width.
- Basis: `e157-r1-prereg.md`; `m2-header311-rerun-questions.md` row 4; D23 (Journal Role & Format) settled items 4 / 17.

### 2026-09-25 (artifact supplemented for the evidence-in-repo cell of gate doc-experiments)

- On 2026-09-25 the artifact was supplemented for the evidence-in-repo cell of gate doc-experiments, byte-identical: the previous rerun judged "byte-identical to the existing artifact" but did not save a new artifact, and the mtime of `e157-parallel-line-one-clauses-2026-09-22.out` was earlier than the moment the apparatus source was changed to 311, so gate no. 69 judged red. Today the registration row of this experiment in `replay.sh` was rerun, byte-identical to the artifact the registration row pointed at then, and saved as `e157-parallel-line-one-clauses-2026-09-25-h311-replay.out` (byte-identical to the old artifact, the old artifact kept, not deleted); the registration of that row in `replay.sh` now points to the new file; no change to the apparatus, no change to the conclusion.
