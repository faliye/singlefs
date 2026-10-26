## E146 Width and Cost of Livelist Entries Identified by Mapping Key — Run (2026-09-13, counting model, 8 unit tests / all 12 mutations caught)

[中文](146-livelist条目按映射key定身份之后的宽度与代价.md) · **English** · [日本語](146-livelist-エントリをマッピング-key-で識別した場合の幅とコスト.ja.md)

The question is D6 (Snapshot Implementation Model) settled item 2 ①: the key and width of a livelist entry. The settled clauses require identity to use the central mapping key (D19 (Block Pointer Structure & Width Budget) settled item 5 rule 1: “all releases go through the mapping, not the hint”). The shared tree key starts with the head's 8-byte tree ID (D6 (Snapshot Implementation Model) settled item 3), followed by the mapping key (D19 (Block Pointer Structure & Width Budget) settled item 6: 27 bytes for code 1, 25 for codes 2 / 3). The 24-byte form used by E130 (Does Per-Head Livelist Pass Bounded Destruction) was invalidated by C266 (livelist entry identity uses position hint that can expire); C260 (livelist entry prepayment omits the amount for unwinding itself) consequently cannot yet fix the prepayment per FREE.
Here we calculate bytes, fanout, height, prepayment and the byte difference for new-pool file creation for each way of identifying by mapping key. **We report numbers without choosing a winner**; the choice belongs to D6 (Snapshot Implementation Model) settled item 2.

**Result in one sentence**: Padding to the widest key and folding event type into the class tag gives a 35-byte entry (leaf fanout 463, internal 137), with 35 bytes prepaid per FREE = 10 basis points of a 32 KiB unit. An event byte gives 36 (leaf 450); splitting into two trees gives 33 for the code 2 / 3 tree (leaf 491), but conflicts with D6 (Snapshot Implementation Model) settled item 3's literal “one shared tree.” A 14-byte position hint in the value gives 49 (leaf 330). On E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout)'s 21 scale cells, the 35-byte form is one level taller than the invalidated 24-byte form in only two cells (the two with 65,536 total entries: height 3 versus 2). Day-1 registration of a shared tree adds 200 bytes to new-pool file creation (one tree table entry, fitting in the first-version tree table unit); if an empty tree also requires a root node, add 16384 + 56 + 53 = 16693. Lazy creation adds 0 bytes.

**Rerun command** (`exact` mode, byte-for-byte comparison with the retained artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E146
# 直接跑装置：
cd scripts/experiment-apparatus && cargo run --release --bin e146-livelist-entry-width
```

Code `scripts/experiment-apparatus/e146_livelist_entry_width.rs`; raw output `e146-livelist-entry-width-2026-09-16-tree-table-200.out` (130 lines, last line `emitted=130`; the version using 200 for a tree table entry). The preceding `e146-livelist-entry-width-2026-09-14-round2.out` used 148 and differs in only 6 lines (one `name=config` and five `name=new_pool_file_creation`); the earlier `e146-livelist-entry-width-2026-09-13.out` used 145. Both versions are retained unchanged. Mutation rerun `e146_livelist_entry_width-mutate-2026-09-16-tree-table-200.log`: all 12 caught.
Pre-run registration `e146-preregistration.md`; mutation table `scripts/mutation/tables/e146_livelist_entry_width.tsv` (all 12 caught,
`bash scripts/mutation/mutate.sh e146-livelist-entry-width scripts/experiment-apparatus/e146_livelist_entry_width.rs scripts/mutation/tables/e146_livelist_entry_width.tsv`).

### This is a counting model, not an implementation

There is no I/O or random source ⇒ N runs are necessarily byte-identical. Evidence comes from 8 unit tests and all 12 mutations caught; fanout, header width, tree height, prepayment and new-pool file creation bytes each have an absolute-value assertion pinned to a hand calculation in the pre-run registration.

### Arms (entry forms)

| Arm | key | value | Trees |
|---|---|---|---|
| `legacy_e130` | 24 (type tag 2 + position entry 14 + birth txg 8; invalidated form, control only) | 0 | 1 |
| `pad_event_in_tag` | head tree ID 8 + mapping key 27 (codes 2 / 3 padded to 27); ALLOC / FREE folded into the high bits of the class-tag byte | 0 | 1 |
| `pad_event_byte` | head tree ID 8 + mapping key 27 (codes 2 / 3 padded to 27) + event type 1 byte | 0 | 1 |
| `two_trees_event_in_tag` | one tree at 35 for code 1, one at 33 for codes 2 / 3 | 0 | 2 |
| `pad_event_in_tag_with_hint` | 35 | position entry 14 (non-authoritative hint) | 1 |

### Measurement basis

- Node geometry is taken verbatim from E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width): node 16384; code-2 header 81 + 2 × key width + reserve 28 (D18 (Information Carried in a Block) settled items 7 / 14); internal entry = key + 83-byte child pointer (D19 (Block Pointer Structure & Width Budget) settled item 7); full occupancy. E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) used header 64 and child pointer 59 under an obsolete measurement basis; those values are no longer used for this quantity.
- Scale grid copied from E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout): heads {1, 4, 16, 64, 256, 1024, 4096} × entries per head {1024, 65536, 1048576}. The two-tree arm splits entries code 1 : codes 2/3 = 8 : 1 (one node per 8 data units in E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width)).
- Prepayment per FREE = leaf-entry width (appending one FREE appends one leaf entry), rounded down in basis points of a 32 KiB unit.
- New-pool file creation: day-1 registration = one additional 200-byte tree table entry (D8 (Core Index Structure) settled item 8); first-version tree table has 5 + 1 ≤ 81 entries per unit (C157 (Tree table capacity computed with differing entry widths in two places); after the user's 2026-09-13 decision to register both livelist and sparse side table on day 1 there are 7, which still fit) ⇒ no extra unit. If the implementation requires a root node even for an empty tree, add one node 16384 + journal named entry 56 + code-2 mapping entry 53. Creation at first clone = 0 bytes.
- All 12 mutations caught: change head tree ID width to 4; code-1 mapping key to 25; omit ×2 in header width; set reserve to 0; round leaf fanout up; omit pointer from internal entry; add 2 per tree level; fail to split entries between two trees; calculate prepayment basis points against node size; omit mapping entry when an empty tree requires a root; omit event byte; change first-version tree table entry count to 112.

### Verdict rows copied verbatim from the artifact

```text
E7RESULT name=config node_bytes=16384 header_without_key_range=81 reserved=28 node_pointer=83 head_tree_identifier=8 mapping_key_data=27 mapping_key_node=25 unit_bytes=32768 tree_table_entry=200 data_units_per_node=8
E7RESULT name=node arm=legacy_e130 tree=shared key_bytes=24 entry_bytes=24 header_bytes=157 payload_bytes=16227 leaf_fanout=676 internal_fanout=151
E7RESULT name=node arm=pad_event_in_tag tree=shared key_bytes=35 entry_bytes=35 header_bytes=179 payload_bytes=16205 leaf_fanout=463 internal_fanout=137
E7RESULT name=node arm=pad_event_byte tree=shared key_bytes=36 entry_bytes=36 header_bytes=181 payload_bytes=16203 leaf_fanout=450 internal_fanout=136
E7RESULT name=node arm=two_trees_event_in_tag tree=data key_bytes=35 entry_bytes=35 header_bytes=179 payload_bytes=16205 leaf_fanout=463 internal_fanout=137
E7RESULT name=node arm=two_trees_event_in_tag tree=node key_bytes=33 entry_bytes=33 header_bytes=175 payload_bytes=16209 leaf_fanout=491 internal_fanout=139
E7RESULT name=node arm=pad_event_in_tag_with_hint tree=shared key_bytes=35 entry_bytes=49 header_bytes=179 payload_bytes=16205 leaf_fanout=330 internal_fanout=137
E7RESULT name=prepaid arm=legacy_e130 bytes_per_free=24 unit_share_basis_points=7
E7RESULT name=prepaid arm=pad_event_in_tag bytes_per_free=35 unit_share_basis_points=10
E7RESULT name=prepaid arm=pad_event_byte bytes_per_free=36 unit_share_basis_points=10
E7RESULT name=prepaid arm=two_trees_event_in_tag bytes_per_free=35 unit_share_basis_points=10
E7RESULT name=prepaid arm=pad_event_in_tag_with_hint bytes_per_free=49 unit_share_basis_points=14
E7RESULT name=new_pool_file_creation arm=pad_event_in_tag tree_table_entries=1 day1_null_root_bytes=200 day1_root_required_bytes=16693 lazy_bytes=0 tree_table_fits_first_unit=true
E7RESULT name=new_pool_file_creation arm=two_trees_event_in_tag tree_table_entries=2 day1_null_root_bytes=400 day1_root_required_bytes=33386 lazy_bytes=0 tree_table_fits_first_unit=true
E7RESULT name=sensitive entries=460 pad_event_in_tag_height=1 pad_event_byte_height=2
E7RESULT name=verdict pad_entry_bytes=35 legacy_entry_bytes=24 pad_leaf_fanout=463 legacy_leaf_fanout=676 grid_cells=21 cells_where_pad_taller_than_legacy=2 prepaid_pad=35 prepaid_legacy=24
```

### Tree height (21 scale cells; parentheses in the two-tree arm give each tree's height)

| Heads | Entries per head | Total entries | `legacy_e130` | `pad_event_in_tag` | `pad_event_byte` | `two_trees_event_in_tag` | `pad_event_in_tag_with_hint` |
|---|---|---|---|---|---|---|---|
| 1 | 1024 | 1024 | 2 | 2 | 2 | 2 (code 1: 2, codes 2/3: 1) | 2 |
| 1 | 65536 | 65536 | 2 | 3 | 3 | 2 (code 1: 2, codes 2/3: 2) | 3 |
| 1 | 1048576 | 1048576 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 3) | 3 |
| 4 | 1024 | 4096 | 2 | 2 | 2 | 2 (code 1: 2, codes 2/3: 1) | 2 |
| 4 | 65536 | 262144 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 2) | 3 |
| 4 | 1048576 | 4194304 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 3) | 3 |
| 16 | 1024 | 16384 | 2 | 2 | 2 | 2 (code 1: 2, codes 2/3: 2) | 2 |
| 16 | 65536 | 1048576 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 3) | 3 |
| 16 | 1048576 | 16777216 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 3) | 4 |
| 64 | 1024 | 65536 | 2 | 3 | 3 | 2 (code 1: 2, codes 2/3: 2) | 3 |
| 64 | 65536 | 4194304 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 3) | 3 |
| 64 | 1048576 | 67108864 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 3) | 4 |
| 256 | 1024 | 262144 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 2) | 3 |
| 256 | 65536 | 16777216 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 3) | 4 |
| 256 | 1048576 | 268435456 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 4) | 4 |
| 1024 | 1024 | 1048576 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 3) | 3 |
| 1024 | 65536 | 67108864 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 3) | 4 |
| 1024 | 1048576 | 1073741824 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 4) | 5 |
| 4096 | 1024 | 4194304 | 3 | 3 | 3 | 3 (code 1: 3, codes 2/3: 3) | 3 |
| 4096 | 65536 | 268435456 | 4 | 4 | 4 | 4 (code 1: 4, codes 2/3: 4) | 4 |
| 4096 | 1048576 | 4294967296 | 5 | 5 | 5 | 5 (code 1: 5, codes 2/3: 4) | 5 |

### What these numbers mean

1. **Identifying by mapping key raises entry width from 24 to 35 bytes** (+46%), reducing leaf fanout from 676 to 463. But height rises by one level in only two of 21 cells (3 versus 2 at 65,536 total entries); the other 19 have equal height ⇒ point lookup and insertion traverse nearly the same number of levels, and the main cost is bytes.
2. **The number needed by C260 (livelist entry prepayment omits the amount for unwinding itself)**: 35 bytes prepaid per FREE (36 with event byte, 49 with hint), 10 / 10 / 14 basis points of a 32 KiB unit. The 24 bytes (7 basis points) in E130 (Does Per-Head Livelist Pass Bounded Destruction) are invalidated. This amount has not yet entered the user-settled 2026-08-31 clause on how much is needed to unbind itself.
3. **Whether to fold event type into the class tag**: the one-byte tag uses only codes 1 / 2 / 3, leaving high bits free. Folding saves one byte, with leaf fanout 463 versus 450; the two have equal height in all 21 cells. This is an encoding trade-off, not a cost trade-off.
4. **Two trees** make only the code 2 / 3 tree two bytes narrower (leaf 491 versus 463); in the largest cell its node tree is one level shorter (4 versus 5). The cost is one extra tree table entry and conflict with D6 (Snapshot Implementation Model) settled item 3's literal “one shared tree”; changing it requires reopening item 3.
5. **A 14-byte position hint in the value** moves the field invalidated by C266 (livelist entry identity uses position hint that can expire) into the value: leaf fanout falls to 330, prepayment is 49, and the 1024-head × 1048576 cell gains a level (5 versus 4). Under D19 (Block Pointer Structure & Width Budget) settled item 5 rule 1, release cannot use the hint, so it buys no path ⇒ reported but not recommended.
6. **Day-1 versus lazy creation differs by 200 bytes for new-pool file creation** if an empty tree has a null root pointer, or 16693 bytes if it requires a root node; two trees double both. Lazy creation's 0 bytes costs the mechanism of creating the tree at first clone.
7. The 8-byte head tree ID prefix and 8-byte birth tree in the mapping key may be redundant (if a tree belongs to one head, the birth tree identifies that head). This belongs to D6 (Snapshot Implementation Model) settled item 2 / C267 (a writable head has no object in the tree table that can be pointed to); it is neither modeled nor omitted here.

### What it cannot answer

1. Where per-head counters live (C267 (a writable head has no object in the tree table that can be pointed to)), wall-clock cost of condense, and node occupancy (calculated fully packed; real B-trees are 50%–100%).
2. Whether the head tree ID prefix can be omitted and whether event type must be in the key (if ALLOC and FREE each have a record for the same block it must be in the key, otherwise both have the same key). The latter is a key-uniqueness requirement; the apparatus models it in the key.
3. Which form to choose: that is the decision for D6 (Snapshot Implementation Model) settled item 2.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 18 | Supports | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 11 | Supports | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D5 (Snapshot / Space Accounting Mechanism) settled item 6 | Supports | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D6 (Snapshot Implementation Model) settled item 2 | Supports | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D6 (Snapshot Implementation Model) settled item 3 | Background | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 6 | Background | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 5 | No impact | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 7 | No impact | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 8 | No impact | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 7 | No impact | 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-16
- Tree table entry 148 → 200 (D8 (Core Index Structure) settled item 8, widened by user settlement on 2026-09-16). The apparatus's `TREE_TABLE_ENTRY_BYTES` changed accordingly and artifact `e146-livelist-entry-width-2026-09-14-round2.out` was replaced by `e146-livelist-entry-width-2026-09-16-tree-table-200.out`; changed lines are in [experiments-history.md](../experiments-history.md), 2026-09-16 (first entry).

### 2026-09-14
- Rerun after tree table entry 145 → 148 (D8 (Core Index Structure) settled item 8: user settlement on 2026-09-14 rearranged the entry and added 8-byte head ID, root pointer 83 → 86, `format-const: TREE_TABLE_ENTRY_BYTES = 148`; the gate stage synchronizing format constants between kb and experiment source went red that day):
  artifact changed to `e146-livelist-entry-width-2026-09-14-round2.out`; **6 of 130 lines** changed: `tree_table_entry` in `name=config` 145→148, and `day1_null_root_bytes` in five `name=new_pool_file_creation` lines 145→148 (two-tree arm 290→296) and `day1_root_required_bytes` 16638→16641 (two trees 33276→33282).
  Entry widths, prepaid bytes, tree heights on the scale grid and discriminating sample rows were unchanged verbatim. All 8 unit tests passed.
  ⚠️ Mutation M12's original text still had `TREE_TABLE_FIRST_VERSION_ENTRIES = 5`, while the apparatus had changed it to 7 under the 2026-09-13 decision ⇒ it could no longer be replaced and the mutation run stopped (exit 3). After changing the original to 7, all 12 were caught.
  Previously: tree table entry 145, day-1 registration 145 / 16638 bytes (two trees 290 / 33276).

### 2026-09-13
- Created. The pre-run registration preceded the apparatus; criteria and failure clauses were unchanged.
- The user decided that day to use the 36-byte form with event byte and day-1 registration (D6 (Snapshot Implementation Model) settled item 2). The apparatus's first-version tree table entry-count constant was changed to 7; the artifact does not contain it and remained byte-identical.
