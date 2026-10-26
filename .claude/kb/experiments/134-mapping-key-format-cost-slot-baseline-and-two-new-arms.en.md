## E134 Mapping Key Format Cost: Slot Baseline and Two New Arms — Run (2026-09-11)

[中文](134-映射key格式代价槽基线与两条新臂.md) · **English** · [日本語](134-マッピングキーフォーマットコスト-スロット基準と2新アーム.ja.md)

**What is being asked**: Supplementing two things to the cost table of E133 (Format Cost of Mapping Key Candidates) — reporting how many extra bytes D27 (Small-Data Packing Container) slots require under each of two baselines;
plus adding two arms: the tightened form of class-reused write sequence key (keys prefixed with 1-byte class tag, tightened across three steps in D19 (Block Pointer Structure & Width Budget) settled item 6 Round 3) and the hybrid path provided by the Round 3 backward leg.
**Reports numbers only, passes no win/loss judgment**: `.claude/rules/fs-design.md` states verbatim: "'competing on economy' constitutes no criterion whatsoever".

**Why it was needed**: E133 (Format Cost of Mapping Key Candidates) took "slots already have 5-tuple 33 and write sequence 10" as the slot baseline, where the 10 bytes of write sequence came from the definition of the `pack` arm in E114 (Overall Accounting of Small-Data Packing Containers)
(47 = 5-tuple 33 + write sequence 10 + slot table 4), whereas in D27 (Small-Data Packing Container) body text, slots only have 5-tuples, and `grep -c write_sequence` is 0
(pointed out by the Round 3 backward leg for D19 (Block Pointer Structure & Width Budget) settled item 6, verified by main agent inspection). The artifacts of E133 (Format Cost of Mapping Key Candidates) were already referenced line by line by legs in that round, so instead of in-place modification, a new number was established.

**Hardcoded before running**: `e134-preregistration.md` (written before apparatus). ⚠️ The author of the registration had already inspected all numbers from E133 (Format Cost of Mapping Key Candidates);
the twelve arms shared with it must reproduce its committed artifact field by field (Criterion 7); the only new elements are the second slot baseline and the two new arms.

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E134
# Run apparatus directly:
cd scripts/experiment-apparatus && cargo run --release --bin e134_map_key_slot_baselines
```

Code: `scripts/experiment-apparatus/e134_map_key_slot_baselines.rs`,
raw output: `e134-map-key-slot-baselines-2026-09-11.out` (single run, 118 lines, final line `emitted=115`).

### This Is a Counting Model, Not an Implementation

No I/O, no random source ⇒ running N times is necessarily byte-for-byte identical; evidence strength derives from **20 unit tests + 24 mutations all caught**,
plus a cross-apparatus gate: every row of the twelve shared arms must be found verbatim in the committed artifact of E133 (Format Cost of Mapping Key Candidates) (matching that specific field in the slot cell).
Expected values for the two new arms and two slot baselines originate from independent Python arithmetic (using only registered raw quantities, not reading apparatus formulas).

### Two New Arms and Two Slot Baselines

| Code | Form |
|---|---|
| `class_reuse_tagged` | Class-reused write sequence key after tightening: keys prefixed with 1-byte class tag, Code 1 at 27, Code 2 / 3 at 25; pointer and unit header increments identical to `class_reuse` (class tag provided by lookup path and unit header); slot key carries its own 1-byte class tag, **not assuming it can reuse the 1-byte unit type tag in the 5-tuple** (value domains of the two were never defined) |
| `k1_seq_class_reuse` | Hybrid path (similarly tagged with class): Code 1 takes `k1_seq` key 51 (5-tuple inherently prefixed with 1-byte unit type tag), pointer to Code 1 +50, Code 1 header zero bytes; Code 2 / 3 takes 25, pointer to Code 2 / 3 +24, Code 2 / 3 headers each +4 |

| Slot baseline | What slot already has | Field in artifact |
|---|---|---|
| E114 (Overall Accounting of Small-Data Packing Containers) `pack` arm | 5-tuple 33 + write sequence 10 (= `packed_slot_extra` of E133 (Format Cost of Mapping Key Candidates)) | `slot_extra_over_e114_pack` |
| D27 (Small-Data Packing Container) body text | 5-tuple 33 only | `slot_extra_over_five_tuple` |

Forms and measurement bases for the remaining twelve arms are identical to "Arms and Measurement Basis" in E133 (Format Cost of Mapping Key Candidates), not repeated here.

### Results (Transcribed Verbatim from `e134-map-key-slot-baselines-2026-09-11.out`)

Header widths and two slot baselines across fourteen arms:

```text
E7RESULT name=headers arm=bing code1=133 code3=131 code2=96/105/114 slot_extra_over_e114_pack=0 slot_extra_over_five_tuple=0 livelist_identity_code1=33
E7RESULT name=headers arm=k1 code1=133 code3=131 code2=96/105/114 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=8 livelist_identity_code1=41
E7RESULT name=headers arm=k1_full code1=133 code3=131 code2=96/105/114 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=8 livelist_identity_code1=41
E7RESULT name=headers arm=k1_seq code1=133 code3=131 code2=96/105/114 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=18 livelist_identity_code1=51
E7RESULT name=headers arm=k2 code1=141 code3=139 code2=104/113/122 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=8 livelist_identity_code1=8
E7RESULT name=headers arm=k2_instance code1=141 code3=139 code2=104/113/122 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=12 livelist_identity_code1=12
E7RESULT name=headers arm=k3 code1=137 code3=135 code2=100/109/118 slot_extra_over_e114_pack=12 slot_extra_over_five_tuple=12 livelist_identity_code1=20
E7RESULT name=headers arm=k3_instance code1=137 code3=135 code2=100/109/118 slot_extra_over_e114_pack=12 slot_extra_over_five_tuple=16 livelist_identity_code1=24
E7RESULT name=headers arm=class_reuse code1=133 code3=135 code2=100/109/118 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=18 livelist_identity_code1=26
E7RESULT name=headers arm=k4 code1=143 code3=141 code2=106/115/124 slot_extra_over_e114_pack=18 slot_extra_over_five_tuple=18 livelist_identity_code1=18
E7RESULT name=headers arm=uuid16 code1=149 code3=147 code2=112/121/130 slot_extra_over_e114_pack=16 slot_extra_over_five_tuple=16 livelist_identity_code1=16
E7RESULT name=headers arm=slotgen12 code1=145 code3=143 code2=108/117/126 slot_extra_over_e114_pack=12 slot_extra_over_five_tuple=12 livelist_identity_code1=12
E7RESULT name=headers arm=class_reuse_tagged code1=133 code3=135 code2=100/109/118 slot_extra_over_e114_pack=9 slot_extra_over_five_tuple=19 livelist_identity_code1=27
E7RESULT name=headers arm=k1_seq_class_reuse code1=133 code3=135 code2=100/109/118 slot_extra_over_e114_pack=8 slot_extra_over_five_tuple=18 livelist_identity_code1=51
```

Widths, fanouts, mapping tree thresholds, and pool-scale levels of two new arms side-by-side with `class_reuse`:

```text
E7RESULT name=widths arm=class_reuse data_pointer=85 node_pointer=83 extent_leaf_entry=109 extent_internal_entry=107 inode_internal=117 ledger_internal=105 tree_table_entry=145 root_record=242 map_entry_code1=54 map_entry_code23=52
E7RESULT name=fanouts arm=class_reuse extent_leaf=149 extent_internal=152 inode_internal=139 ledger_internal=155 tree_table_per_level=112 map_leaf=302 map_internal=149 inode_leaf=233
E7RESULT name=thresholds arm=class_reuse tree=map l2=303 l3=44999 l4=6704703 l5=999000599
E7RESULT name=map_at_pool arm=class_reuse data_units_levels=4 data_units_leaf_bytes=26213531648 with_index_levels=4 with_index_leaf_bytes=26433814528
E7RESULT name=widths arm=class_reuse_tagged data_pointer=85 node_pointer=83 extent_leaf_entry=109 extent_internal_entry=107 inode_internal=117 ledger_internal=105 tree_table_entry=145 root_record=242 map_entry_code1=55 map_entry_code23=53
E7RESULT name=fanouts arm=class_reuse_tagged extent_leaf=149 extent_internal=152 inode_internal=139 ledger_internal=155 tree_table_per_level=112 map_leaf=296 map_internal=148 inode_leaf=233
E7RESULT name=thresholds arm=class_reuse_tagged tree=map l2=297 l3=43809 l4=6483585 l5=959570433
E7RESULT name=map_at_pool arm=class_reuse_tagged data_units_levels=4 data_units_leaf_bytes=26744881152 with_index_levels=4 with_index_leaf_bytes=26969636864
E7RESULT name=widths arm=k1_seq_class_reuse data_pointer=109 node_pointer=83 extent_leaf_entry=133 extent_internal_entry=107 inode_internal=117 ledger_internal=105 tree_table_entry=145 root_record=242 map_entry_code1=79 map_entry_code23=53
E7RESULT name=fanouts arm=k1_seq_class_reuse extent_leaf=122 extent_internal=152 inode_internal=139 ledger_internal=155 tree_table_per_level=112 map_leaf=206 map_internal=121 inode_leaf=233
E7RESULT name=thresholds arm=k1_seq_class_reuse tree=map l2=207 l3=24927 l4=3016047 l5=364941567
E7RESULT name=map_at_pool arm=k1_seq_class_reuse data_units_levels=5 data_units_leaf_bytes=38429540352 with_index_levels=5 with_index_leaf_bytes=38752485376
```

| Cell | Reading |
|---|---|
| D27 (Small-Data Packing Container) slot extra bytes (slot has 5-tuple only) | Among candidates: 8 (`k1` / `k1_full` / `k2`) to 19 (`class_reuse_tagged`); the difference exceeding E114 (Overall Accounting of Small-Data Packing Containers) baseline comes from the six arms requiring write sequence or instance generation in keys: `k1_seq`, `class_reuse`, `k1_seq_class_reuse` 8 → 18, `class_reuse_tagged` 9 → 19, `k3_instance` 12 → 16, `k2_instance` 8 → 12 |
| Cost of class tag (`class_reuse` → `class_reuse_tagged`) | Pointers, unit headers, root records, tree table entries remain unchanged by a single byte; mapping entries 54 / 52 → 55 / 53, mapping leaf fanout 302 → 296, internal fanout 149 → 148; mapping tree remains 4 levels at E109 (Runtime Costs of Three Location Authority Arms) pool scale, leaf layer bytes 26 213 531 648 → 26 744 881 152 |
| Hybrid path | Pointer to Code 1 109, extent leaf fanout 122, mapping leaf fanout 206, mapping tree 5 levels, cell-by-cell equal to `k1_seq`; mapping internal fanout 121 (`k1_seq` 129, pointer to Code 2 / 3 wider); Code 2 / 3 headers and pointer to Code 2 / 3 equal to `class_reuse_tagged` |

### What It Cannot Answer

- **Does not judge winners or losers**, nor provide a ranking of "who is better".
- Which code the class tag takes: D19 (Block Pointer Structure & Width Budget) settled item 6 Round 1 adversarial debate tightened this to "taking the code of the named object in registry of D18 (Information Carried in a Block) settled item 11, with objects in D27 (Small-Data Packing Container) slots constant at Code 1". Whether the initial byte of 5-tuples in slots also writes Code 1 awaits the field table of D27 (Small-Data Packing Container) new classes; when writing Code 1, `class_reuse_tagged` can reuse that 1 byte, reducing slot increment by 1 under both baselines; here the non-reused figure is reported. The hybrid path's Code 1 key directly uses the first byte of 5-tuple as class tag (slot increment 18), effectively defaulting to reusable — the two new arms assume differently here, differing by 1 byte.
- Remaining scope identical to "What It Cannot Answer" in E133 (Format Cost of Mapping Key Candidates): calculates only format widths, fanout, and level thresholds, not CPU, cache, or I/O.

### Mutations

All 24 mutations caught. 19 mutations from E133 (Format Cost of Mapping Key Candidates) imported verbatim (M18 text includes arm name, otherwise hitting three times in new arms); plus 5 new mutations:
omitting 1-byte class tag, slot 5-tuple-only baseline omitting write sequence, hybrid path Code 2 / 3 omitting class tag, artifact swapping the two slot baselines, hybrid path Code 1 pointer using the wrong type.
**The swapping mutation is caught solely by the cross-apparatus gate** — the two baselines differ in values across only six of fourteen arms, while unit tests pinning values arm by arm check only fields, not artifact rows.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 6 | Supports | 2026-09-21 unaffected: artifact merely retrieved from archive state (apparatus source code reads it via include_str! at compile time), byte-for-byte identical to pre-archive; body text, conclusions, and numbers untouched |
| D19 (Block Pointer Structure & Width Budget) settled item 8 | Supports | 2026-09-21 unaffected: artifact merely retrieved from archive state (apparatus source code reads it via include_str! at compile time), byte-for-byte identical to pre-archive; body text, conclusions, and numbers untouched |
| D27 (Small-Data Packing Container) settled item 3 | Preparation | 2026-09-21 unaffected: artifact merely retrieved from archive state (apparatus source code reads it via include_str! at compile time), byte-for-byte identical to pre-archive; body text, conclusions, and numbers untouched |
| D18 (Information Carried in a Block) | Does not affect | 2026-09-21 unaffected: artifact merely retrieved from archive state (apparatus source code reads it via include_str! at compile time), byte-for-byte identical to pre-archive; body text, conclusions, and numbers untouched |

## Historical Versions

### 2026-09-11

- Established and completed run.
- "What It Cannot Answer" section: class tag item rewritten to current state tightened after D19 (Block Pointer Structure & Width Budget) settled item 6 Round 1 adversarial debate, stating explicitly that the two new arms make different assumptions on slots (pointed out by backward leg). Artifact, source code, and mutation table untouched.
