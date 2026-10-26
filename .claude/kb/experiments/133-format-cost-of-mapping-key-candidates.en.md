## E133 Format Cost of Mapping Key Candidates — Run (2026-09-11)

[中文](133-映射key各候选的格式代价.md) · **English** · [日本語](133-マッピングキー各候補のフォーマットコスト.ja.md)

**What is being asked**: For every mapping key that appeared in the two rounds of three-way argument for D19 (Block Pointer Structure & Width Budget) settled item 6,
how many bytes each incurs when projected onto format —
two kinds of pointers (pointing to Code 1 data units vs pointing to Code 2 / Code 3), three classes of unit headers, root records, tree table entries, mapping entries —
as well as how they alter the fanout and level thresholds of extent trees, inode trees, accounting trees, and mapping trees.
**Reports numbers only, passes no win/loss judgment**: `.claude/rules/fs-design.md` states verbatim: "'competing on economy' constitutes no criterion whatsoever". All its criteria are apparatus reconciliation.

**Why it was needed**: D19 (Block Pointer Structure & Width Budget) settled item 6 must be submitted to the user for settlement, and before submission every path must have cost figures;
prior to E133 (Format Cost of Mapping Key Candidates), the repository had E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) which only calculated pointer +16, and E109 (Runtime Costs of Three Location Authority Arms) which only swept key 33 / 40 tiers.

**Hardcoded before running**: `e133-preregistration.md` (written before apparatus). Its "Postscript" section recorded two changes prior to formal execution:
① To supplement sensitive sampling points for mutations, **the apparatus author had already inspected figures of all arms via independent arithmetic before the formal run**;
② Four arms were supplemented after Round 2 three-way argument, and pointer increments were split into two quantities: "pointing to Code 1 vs pointing to Code 2/3". Criteria remained unchanged by a single word.

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E133
# Run apparatus directly:
cd scripts/experiment-apparatus && cargo run --release --bin e133_map_key_format_cost
```

Code: `scripts/experiment-apparatus/e133_map_key_format_cost.rs`,
raw output: `e133-map-key-format-cost-2026-09-11.out` (single run, 102 lines, final line `emitted=99`).

### This Is a Counting Model, Not an Implementation

No I/O, no random source ⇒ running N times is necessarily byte-for-byte identical; evidence strength derives from **16 unit tests + 19 mutations all caught**.
In unit tests, widths, seven fanouts, and mapping tree levels for each arm are pinned to absolute values, with expected values originating from independent Python arithmetic (using only registered raw quantities, not reading apparatus formulas).

### Arms and Measurement Basis

| Code | Form (Source: D19 (Block Pointer Structure & Width Budget) settled item 6 and Change History 2026-09-11 (Part 1)) |
|---|---|
| `bing` | Current: pointers carry no birth identity; mapping key takes 5-tuple 33 per pre-Round-1 vision. **Baseline only, not a candidate** |
| `k1` | Logical identity key verbatim per Round 2 background materials: each pointer adds only birth tree + birth txg |
| `k1_full` | Strongest form of logical identity key: pointers to Code 1 also carry object ID, object birth generation, anchor offset; pointers to Code 2 / 3 carry only birth tree + birth txg |
| `k1_seq` | Logical identity + write sequence key: adds write sequence 10 to strongest form |
| `k2` | Surrogate sequence key (pool-wide monotonic sequence 8, plus birth tree + birth txg; root record adds high-water mark 8) |
| `k2_instance` | Intra-instance counter key (instance generation 4 + intra-instance counter 8, plus birth tree + birth txg; no high-water mark) |
| `k3` | Birth identity key (birth tree 8 + birth txg 8 + birth sequence 4) |
| `k3_instance` | Birth identity + instance key (further adds instance generation 4; instance generation borrowed from existing write sequence) |
| `class_reuse` | Class-reused write sequence key (Code 1: birth tree + birth txg + write sequence; Code 2 / 3: birth tree + birth txg + instance generation + birth sequence) |
| `k4` | Birth placement key (position entry frozen as first two segments of key, plus birth txg + birth tree) |
| `uuid16` | Random 16-byte identifier key proposed by Round 2 local leg |
| `slotgen12` | Birth placement + per-slot generation key proposed by Round 2 local leg (cost of persistent per-slot generation table unmodeled) |

Measurement basis (individual sources documented in preregistration and apparatus constants): node 16384, reference tree node header 64, mapping node header 76, tree table net bytes per level 16284,
today's child pointer 59 = header 31 + position entry 14 × 2, pool scale takes E109 (Runtime Costs of Three Location Authority Arms) 483 183 820 data units ("with index nodes" tier adds ⌈N / 119⌉).

### Results (Transcribed Verbatim from `e133-map-key-format-cost-2026-09-11.out`)

Widths:

```text
E7RESULT name=widths arm=bing data_pointer=59 node_pointer=59 extent_leaf_entry=83 extent_internal_entry=83 inode_internal=93 ledger_internal=81 tree_table_entry=121 root_record=194 map_entry_code1=61 map_entry_code23=n/a
E7RESULT name=widths arm=k1 data_pointer=75 node_pointer=75 extent_leaf_entry=99 extent_internal_entry=99 inode_internal=109 ledger_internal=97 tree_table_entry=137 root_record=226 map_entry_code1=69 map_entry_code23=n/a
E7RESULT name=widths arm=k1_full data_pointer=99 node_pointer=75 extent_leaf_entry=123 extent_internal_entry=99 inode_internal=109 ledger_internal=97 tree_table_entry=137 root_record=226 map_entry_code1=69 map_entry_code23=n/a
E7RESULT name=widths arm=k1_seq data_pointer=109 node_pointer=75 extent_leaf_entry=133 extent_internal_entry=99 inode_internal=109 ledger_internal=97 tree_table_entry=137 root_record=226 map_entry_code1=79 map_entry_code23=n/a
E7RESULT name=widths arm=k2 data_pointer=83 node_pointer=83 extent_leaf_entry=107 extent_internal_entry=107 inode_internal=117 ledger_internal=105 tree_table_entry=145 root_record=250 map_entry_code1=36 map_entry_code23=36
E7RESULT name=widths arm=k2_instance data_pointer=87 node_pointer=87 extent_leaf_entry=111 extent_internal_entry=111 inode_internal=121 ledger_internal=109 tree_table_entry=149 root_record=250 map_entry_code1=40 map_entry_code23=40
E7RESULT name=widths arm=k3 data_pointer=79 node_pointer=79 extent_leaf_entry=103 extent_internal_entry=103 inode_internal=113 ledger_internal=101 tree_table_entry=141 root_record=234 map_entry_code1=48 map_entry_code23=48
E7RESULT name=widths arm=k3_instance data_pointer=83 node_pointer=83 extent_leaf_entry=107 extent_internal_entry=107 inode_internal=117 ledger_internal=105 tree_table_entry=145 root_record=242 map_entry_code1=52 map_entry_code23=52
E7RESULT name=widths arm=class_reuse data_pointer=85 node_pointer=83 extent_leaf_entry=109 extent_internal_entry=107 inode_internal=117 ledger_internal=105 tree_table_entry=145 root_record=242 map_entry_code1=54 map_entry_code23=52
E7RESULT name=widths arm=k4 data_pointer=75 node_pointer=75 extent_leaf_entry=99 extent_internal_entry=99 inode_internal=109 ledger_internal=97 tree_table_entry=137 root_record=226 map_entry_code1=46 map_entry_code23=46
E7RESULT name=widths arm=uuid16 data_pointer=91 node_pointer=91 extent_leaf_entry=115 extent_internal_entry=115 inode_internal=125 ledger_internal=113 tree_table_entry=153 root_record=258 map_entry_code1=44 map_entry_code23=44
E7RESULT name=widths arm=slotgen12 data_pointer=77 node_pointer=77 extent_leaf_entry=101 extent_internal_entry=101 inode_internal=111 ledger_internal=99 tree_table_entry=139 root_record=230 map_entry_code1=40 map_entry_code23=40
```

Unit headers and extra bytes required for D27 (Small-Data Packing Container) slots:

```text
E7RESULT name=headers arm=bing code1=133 code3=131 code2=96/105/114 packed_slot_extra=0 livelist_identity_code1=33
E7RESULT name=headers arm=k1 code1=133 code3=131 code2=96/105/114 packed_slot_extra=8 livelist_identity_code1=41
E7RESULT name=headers arm=k1_full code1=133 code3=131 code2=96/105/114 packed_slot_extra=8 livelist_identity_code1=41
E7RESULT name=headers arm=k1_seq code1=133 code3=131 code2=96/105/114 packed_slot_extra=8 livelist_identity_code1=51
E7RESULT name=headers arm=k2 code1=141 code3=139 code2=104/113/122 packed_slot_extra=8 livelist_identity_code1=8
E7RESULT name=headers arm=k2_instance code1=141 code3=139 code2=104/113/122 packed_slot_extra=8 livelist_identity_code1=12
E7RESULT name=headers arm=k3 code1=137 code3=135 code2=100/109/118 packed_slot_extra=12 livelist_identity_code1=20
E7RESULT name=headers arm=k3_instance code1=137 code3=135 code2=100/109/118 packed_slot_extra=12 livelist_identity_code1=24
E7RESULT name=headers arm=class_reuse code1=133 code3=135 code2=100/109/118 packed_slot_extra=8 livelist_identity_code1=26
E7RESULT name=headers arm=k4 code1=143 code3=141 code2=106/115/124 packed_slot_extra=18 livelist_identity_code1=18
E7RESULT name=headers arm=uuid16 code1=149 code3=147 code2=112/121/130 packed_slot_extra=16 livelist_identity_code1=16
E7RESULT name=headers arm=slotgen12 code1=145 code3=143 code2=108/117/126 packed_slot_extra=12 livelist_identity_code1=12
```

Fanouts:

```text
E7RESULT name=fanouts arm=bing extent_leaf=196 extent_internal=196 inode_internal=175 ledger_internal=201 tree_table_per_level=134 map_leaf=267 map_internal=177 inode_leaf=233
E7RESULT name=fanouts arm=k1 extent_leaf=164 extent_internal=164 inode_internal=149 ledger_internal=168 tree_table_per_level=118 map_leaf=236 map_internal=140 inode_leaf=233
E7RESULT name=fanouts arm=k1_full extent_leaf=132 extent_internal=164 inode_internal=149 ledger_internal=168 tree_table_per_level=118 map_leaf=236 map_internal=140 inode_leaf=233
E7RESULT name=fanouts arm=k1_seq extent_leaf=122 extent_internal=164 inode_internal=149 ledger_internal=168 tree_table_per_level=118 map_leaf=206 map_internal=129 inode_leaf=233
E7RESULT name=fanouts arm=k2 extent_leaf=152 extent_internal=152 inode_internal=139 ledger_internal=155 tree_table_per_level=112 map_leaf=453 map_internal=179 inode_leaf=233
E7RESULT name=fanouts arm=k2_instance extent_leaf=147 extent_internal=147 inode_internal=134 ledger_internal=149 tree_table_per_level=109 map_leaf=407 map_internal=164 inode_leaf=233
E7RESULT name=fanouts arm=k3 extent_leaf=158 extent_internal=158 inode_internal=144 ledger_internal=161 tree_table_per_level=115 map_leaf=339 map_internal=164 inode_leaf=233
E7RESULT name=fanouts arm=k3_instance extent_leaf=152 extent_internal=152 inode_internal=139 ledger_internal=155 tree_table_per_level=112 map_leaf=313 map_internal=152 inode_leaf=233
E7RESULT name=fanouts arm=class_reuse extent_leaf=149 extent_internal=152 inode_internal=139 ledger_internal=155 tree_table_per_level=112 map_leaf=302 map_internal=149 inode_leaf=233
E7RESULT name=fanouts arm=k4 extent_leaf=164 extent_internal=164 inode_internal=149 ledger_internal=168 tree_table_per_level=118 map_leaf=354 map_internal=175 inode_leaf=233
E7RESULT name=fanouts arm=uuid16 extent_leaf=141 extent_internal=141 inode_internal=130 ledger_internal=144 tree_table_per_level=106 map_leaf=370 map_internal=152 inode_leaf=233
E7RESULT name=fanouts arm=slotgen12 extent_leaf=161 extent_internal=161 inode_internal=147 ledger_internal=164 tree_table_per_level=117 map_leaf=407 map_internal=183 inode_leaf=233
```

Mapping tree at E109 (Runtime Costs of Three Location Authority Arms) pool scale:

```text
E7RESULT name=map_at_pool arm=bing data_units_levels=4 data_units_leaf_bytes=29649764352 with_index_levels=4 with_index_leaf_bytes=29898915840
E7RESULT name=map_at_pool arm=k1 data_units_levels=4 data_units_leaf_bytes=33544437760 with_index_levels=4 with_index_leaf_bytes=33826324480
E7RESULT name=map_at_pool arm=k1_full data_units_levels=4 data_units_leaf_bytes=33544437760 with_index_levels=4 with_index_leaf_bytes=33826324480
E7RESULT name=map_at_pool arm=k1_seq data_units_levels=5 data_units_leaf_bytes=38429540352 with_index_levels=5 with_index_leaf_bytes=38752485376
E7RESULT name=map_at_pool arm=k2 data_units_levels=4 data_units_leaf_bytes=17475682304 with_index_levels=4 with_index_leaf_bytes=17622548480
E7RESULT name=map_at_pool arm=k2_instance data_units_levels=4 data_units_leaf_bytes=19450822656 with_index_levels=4 with_index_leaf_bytes=19614285824
E7RESULT name=map_at_pool arm=k3 data_units_levels=4 data_units_leaf_bytes=23352475648 with_index_levels=4 with_index_leaf_bytes=23548706816
E7RESULT name=map_at_pool arm=k3_instance data_units_levels=4 data_units_leaf_bytes=25292292096 with_index_levels=4 with_index_leaf_bytes=25504825344
E7RESULT name=map_at_pool arm=class_reuse data_units_levels=4 data_units_leaf_bytes=26213531648 with_index_levels=4 with_index_leaf_bytes=26433814528
E7RESULT name=map_at_pool arm=k4 data_units_levels=4 data_units_leaf_bytes=22362963968 with_index_levels=4 with_index_leaf_bytes=22550888448
E7RESULT name=map_at_pool arm=uuid16 data_units_levels=4 data_units_leaf_bytes=21395914752 with_index_levels=4 with_index_leaf_bytes=21575712768
E7RESULT name=map_at_pool arm=slotgen12 data_units_levels=4 data_units_leaf_bytes=19450822656 with_index_levels=4 with_index_leaf_bytes=19614285824
```

Reconciliation:

```text
E7RESULT name=positive_control arms=12 violations=0
E7RESULT name=cross_apparatus e109_map_leaf_at_33=267 e128_child_pointer_at_16=75
E7RESULT name=done emitted=99
```

Cells read directly from lines transcribed in this section without extrapolation:

| Cell | Reading |
|---|---|
| Mapping tree levels (counting data units only / with index nodes) | Across twelve arms, only `k1_seq` is 5 levels; all others are 4 levels |
| Mapping tree leaf layer bytes (counting data units only) | Minimum `k2` 17 475 682 304, maximum `k1_seq` 38 429 540 352; `bing` 29 649 764 352 |
| extent leaf fanout | `bing` 196; eleven candidates range between 122 (`k1_seq`) and 164 (`k1` / `k4`) |
| Code 1 header | `k1` / `k1_full` / `k1_seq` / `class_reuse` remains 133; `k3` / `k3_instance` 137; `k2` / `k2_instance` 141; `k4` 143; `slotgen12` 145; `uuid16` 149 |
| Root record | 226 (`k1` / `k1_full` / `k1_seq` / `k4`) to 258 (`uuid16`), all within 512-byte slot |
| Tree table entries per level | `bing` 134; candidates range between 106 (`uuid16`) and 118 |
| D27 (Small-Data Packing Container) slot extra bytes | 8 to 18 bytes (baseline takes E114 (Overall Accounting of Small-Data Packing Containers) `pack` arm slot; numbers when slot has 5-tuple only are in E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms)) |

⚠️ Pointers for both `k1` and `k4` are 75, **byte-for-byte identical on the pointer side**; difference between the two lies solely in key width and header width.

### What It Cannot Answer

- **Does not judge winners or losers**, nor provide a ranking of "who is better".
- Calculates only format widths, fanout, and level thresholds; **does not calculate CPU, cache, or I/O** (which belong to the class of E128 (Point Lookup Cost of Pointers with Birth (tree, txg))).
- Mapping tree calculated by Code 1 key; Code 2 / 3 entries estimated as ⌈N / 119⌉. **The logical identity family cannot provide fixed-width keys for Code 2** (its Code 2 keys carry key interval lower bounds, width varying per tree), reported as `n/a` in artifact.
- Increments in Code 2 headers **are not fed back into fanout**: node header 64 in the fanout formula does not share the same measurement basis with the three tiers of Code 2 headers in D18 (Information Carried in a Block); header widths are reported as is.
- D27 (Small-Data Packing Container) slots report only increments: which fields the slot carries is stated in D27 (Small-Data Packing Container) settled item 3 as "still undecided". ⚠️ **Increment baseline assumes E114 (Overall Accounting of Small-Data Packing Containers) `pack` arm** (slot already having 5-tuple 33 + write sequence 10); in D27 (Small-Data Packing Container) body text, slots only have 5-tuples; numbers under that baseline and two new arms appear in E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms).
- Accounting tree leaf entry takes 30 from D5 (Snapshot / Space Accounting Mechanism) settled item 5; under D8 (Core Index Structure) settled item 7 where seq resides in value, it is 34; both measurement bases have identical values across all arms, shifting only the absolute value of the accounting tree threshold.
- Both pointers in root records calculated assuming "pointing to Code 2 / 3"; whether the instance table unit pointer directly held by root records needs a key depends on exemptions such as C248 (bootstrap exemption of tree table units unwritten).

### Mutations

All 19 mutations caught. Initial 3 uncaught mutations classified under `.claude/rules/mutation-sampling.md` tripartite division:
Swapping node header 64 and 76 (two mutations) was **sampling point insensitivity** — registered 196 / 164 / 175 / 201 / 267 / 236 happened to have identical values under both measurement bases; supplemented branching sample points (inode internal entry 111 bytes for `slotgen12`, mapping leaf entry 48 bytes for `k3`);
mapping internal fanout mistakenly using leaf entry width was a **true blind spot**, supplemented with arm-by-arm absolute value assertions. After splitting two kinds of pointers, two more mutations added (extent internal entry mistakenly using pointer to Code 1, strongest logical identity form widening node pointers too), both caught.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 6 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D27 (Small-Data Packing Container) settled item 3 | Preparation | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-11

- Added a sentence to body: slot increment baseline is E114 (Overall Accounting of Small-Data Packing Containers) arm assumption; numbers recalculated per D27 (Small-Data Packing Container) body are in E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms); source code modified only that single comment, artifact preserved as is.
- Established and completed run. Version 1 of apparatus added 40 bytes of strongest logical identity form to all pointers, corrected to two pointer increments before running;
  Version 1 figures were reported only once in conversation ("strongest logical identity adds one layer to mapping tree"), already corrected to user — what adds a layer is logical identity + write sequence key.
