## E145 Cost of Code 2 Self-Describing Header and Mapping Tree Key Width — Run (2026-09-13, Counting Model, 9 Unit Tests / 11 Mutations All Caught)

[中文](145-码2自描述头与映射树key宽的代价.md) · **English** · [日本語](145-コード2自己記述ヘッダとマッピングツリーキー幅のコスト.ja.md)

**It asks about the respective costs of C306 (code 2 header is not self-describing about key width) and C307 (how two key widths of the mapping tree fit into one tree with a fixed-width key).** The user directed: "code 2 may be self-describing".
**All six criteria pass**: the self-describing header (arm B) costs 5 more bytes per node, none of the five trees loses a single slot of fanout, and tree height does not change at all on a 10 TB pool;
when no tree table exists during the scan, the headers of arm A, by trying the registered widths one by one, resolve all 1000 nodes of the five registered trees, and none of the 200 nodes of unregistered trees, while self-describing headers resolve all 1200;
the three ways of fitting the mapping tree all give tree height 4 on a 10 TB pool, with zero-padding to 27 and two trees giving leaf fanouts 294 / 306, and self-describing entries 284. Only the numbers are reported; the form is settled by decision.

**Replay command** (`exact` mode, compared byte by byte with the retained artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E145
# Run the rig directly:
cd scripts/experiment-apparatus && cargo run --release --bin e145-self-describing-node-header
```

Code `scripts/experiment-apparatus/e145_self_describing_node_header.rs`, raw output `e145-self-describing-node-header-2026-09-16-tree-table-200.out` (57 lines, last line `emitted=57`; the version with tree table entries computed at 200 per settled item 8 of D8 (Core Index Structure); the previous version `e145-self-describing-node-header-2026-09-14-round2.out` computed at 148 differs from it only in 3 lines of `tree_table_bytes` under `name=mapping`, and the version before that `e145-self-describing-node-header-2026-09-13.out` computed at 145; both versions are kept as is; mutation replay `e145_self_describing_node_header-mutate-2026-09-16-tree-table-200.log`, all 11 caught),
pre-run registration `e145-preregistration.md`, mutation table `scripts/mutation/tables/e145_self_describing_node_header.tsv`.

### This Is a Counting Model, Not a Measurement

No I/O and no random source (the scan simulation uses a fixed seed) ⇒ N runs are byte-identical. The strength of evidence comes from the absolute values pinned in the 9 unit tests + all 11 mutations caught:
header base changed to 80, extra self-describing bytes changed to 4, fixed-width range changed to 16, arm A forgetting to subtract the entry count field inside the payload, tree height counted from 0, the registered width table dropping 27, arm A trying only the first four widths,
the self-describing scanner trusting the key width in the header without verifying the checksum, zero-padding arm entry changed to 53, two trees registering only one tree table entry, self-describing entries without the offset table, each with a red test.
⚠️ One equivalent mutation is on file: for arm A, "exactly one width passes the checksum" and "take the first width that passes" behave the same under a 32-byte checksum (two digests with different coverage will not both equal the same field), and a unit test pins "at most one width passes per node".

### Objects Under Test and Arms

Code 2 header (C306 (code 2 header is not self-describing about key width)), header width following E142 (New-Pool File Creation Dry Run) as 81 + 2 × key width:

| Arm | Header | How the scan resolves it |
|---|---|---|
| A today | 81 + 2 × key width | Without a tree table the key width is unknown: try the five registered widths one by one, and it counts as resolved only when exactly one width passes the header checksum; unregistered trees cannot be resolved |
| B self-describing | A + key width 1 + entry count 2 + entry width 2 | Read the key width in the header, then verify the header checksum |
| C fixed-width range | 81 + 2 × 32 + 5 = 150 | Fixed header width; the key width field says how many leading bytes of the range are valid |

Mapping tree (C307 (how two key widths of the mapping tree fit into one tree with a fixed-width key)): a zero-pad to 27 (entry 55), b two trees (code 1 tree 55, code 2 / code 3 tree 53, the tree table gains one entry of 200 bytes), c self-describing entries (1-byte key length + key + 28 + a 2-byte in-node offset table ⇒ 58 / 56).
Scale: 10 TB ÷ 32 KiB = 335 544 320 data units, code 2 / code 3 counted at 1/8 of that; tree height computed at full packing, internal node entry = key + 83 pointer.

### Verdict (Judged by the Criteria Fixed Before the Run, Lines Copied Verbatim from the Artifact)

```text
E7RESULT name=header tree=inode arm=A_today key_width=8 header_bytes=97 with_reserved=125
E7RESULT name=header tree=inode arm=B_self_describing key_width=8 header_bytes=102 with_reserved=130
E7RESULT name=header tree=inode arm=C_fixed_key_range key_width=8 header_bytes=150 with_reserved=178
E7RESULT name=fanout tree=inode arm=A_today payload_bytes=16255 leaf_entry_bytes=117 leaf_fanout=138 internal_fanout=178
E7RESULT name=fanout tree=inode arm=B_self_describing payload_bytes=16254 leaf_entry_bytes=117 leaf_fanout=138 internal_fanout=178
E7RESULT name=header tree=extent arm=B_self_describing key_width=24 header_bytes=134 with_reserved=162
E7RESULT name=fanout tree=extent arm=B_self_describing payload_bytes=16222 leaf_entry_bytes=109 leaf_fanout=148 internal_fanout=151
E7RESULT name=height tree=extent arm=B_self_describing leaf_entries=335544320 height=4
E7RESULT name=header tree=mapping arm=B_self_describing key_width=27 header_bytes=140 with_reserved=168
E7RESULT name=fanout tree=mapping arm=B_self_describing payload_bytes=16216 leaf_entry_bytes=55 leaf_fanout=294 internal_fanout=147
E7RESULT name=scan arm=A_today registered_nodes=1000 resolved=1000 unresolved=0 ambiguous=0 unregistered_nodes=200 unregistered_resolved=0 unregistered_unresolved=200
E7RESULT name=scan arm=B_self_describing registered_nodes=1000 resolved=1000 unresolved=0 ambiguous=0 unregistered_nodes=200 unregistered_resolved=200 unregistered_unresolved=0
E7RESULT name=mapping arm=a_pad_to_27 data_entry_bytes=55 node_entry_bytes=55 tree_table_entries=1 tree_table_bytes=200 data_leaf_fanout=294 node_leaf_fanout=294
E7RESULT name=mapping arm=b_two_trees data_entry_bytes=55 node_entry_bytes=53 tree_table_entries=2 tree_table_bytes=400 data_leaf_fanout=294 node_leaf_fanout=306
E7RESULT name=mapping arm=c_self_describing_entries data_entry_bytes=58 node_entry_bytes=56 tree_table_entries=1 tree_table_bytes=200 data_leaf_fanout=284 node_leaf_fanout=284
E7RESULT name=mapping_height arm=a_pad_to_27 data_height=4 node_height=4 reads_per_data_lookup=4
E7RESULT name=mapping_height arm=b_two_trees data_height=4 node_height=4 reads_per_data_lookup=4
E7RESULT name=verdict trees_with_height_difference=0 today_resolves_registered=1000 today_resolves_unregistered=0 today_ambiguous=0 self_describing_resolves_all=true
```

| # | Criterion | Result |
|---|---|---|
| 1 | Absolute header width | A at key width 8 / 12 / 22 / 24 / 27 gives 97 / 105 / 125 / 129 / 135, B adds 5 to each, C is constantly 150, equal to the registration cell by cell |
| 2 | Fanout | Five trees × three arms equal the hand-computed ⌊payload ÷ entry⌋ cell by cell; B relative to A loses not a single slot (A's payload already has to hold the 4-byte entry count / entry width, B only adds 1 byte of key width); C loses 2 on the allocation tree (812 → 810) |
| 3 | Tree height | Five trees on a 10 TB pool are 4 for all three arms (inode 41 943 040 entries, extent 335 544 320 entries, mapping 377 487 360 entries), `trees_with_height_difference=0` |
| 4 | Scan header resolution | A resolves 1000 of the registered trees, 0 of the unregistered, ambiguous 0; B / C each resolve 1200 |
| 5 | Three mapping arms | a 55 / 294, b 55 and 53 / 294 and 306 with the tree table 200 bytes larger, c 58 and 56 / 284 |
| 6 | Mapping tree height and reads | All three arms have tree height 4, one point lookup reads 4 nodes |

### What These Numbers Say

1. **Self-description is almost free**: B has 1 more byte per node than A (A also has to hold the entry count and entry width, just in the payload), and fanout and tree height are identical cell by cell. The C fixed-width range costs 21 to 53 more bytes and loses 2 slots of fanout on the allocation tree, buying only "constant header width".
2. **Arm A is not entirely unable to resolve; it cannot resolve unregistered trees**: trying the five registered widths one by one and requiring exactly one to pass has no ambiguity under a 32-byte header checksum; settled item 2 of D15 (Format Freeze Policy) settles that the tree table is an open list on day 1, and a future tree is, in the eyes of the old scanner, exactly those 200 unresolvable nodes.
   ⇒ The severity of C306 (code 2 header is not self-describing about key width) is "cannot resolve new trees under an open list", not "cannot resolve anything".
3. **The three ways of fitting the mapping tree cannot be told apart in tree height**: the leaf fanouts 294 / 306 of a and b are both 4 levels on a 10 TB pool; b has one more tree table entry and one more tree root to read; c has 3 more bytes per entry, loses 10 slots of fanout, and also loses the in-node fixed-width binary search.
   ⇒ By cost, C307 (how two key widths of the mapping tree fit into one tree with a fixed-width key) is a tie between a and b, and the difference lies in the clauses: a must state in settled item 6 of D19 (Block Pointer Structure & Width Budget) that "the key of code 2 / code 3 is zero-padded by two bytes to 27", while b must add a tree.

### What It Cannot Answer

1. Counting model: no time is measured; tree height is computed at full packing, and the real fill rate is lower and trees taller, but all three arms change together.
2. The scan header resolution builds only unit headers, not a real pool; unregistered trees use key width 16 to stand for "future trees".
3. It does not answer the D15 (Format Freeze Policy) layer: whether the self-describing field goes in layer 1 or layer 4 is settled by decision.
4. Determinism: N runs are identical.

### Affected Decisions

| Decision Item | Relation | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 18 | Supports | 2026-10-11 not affected: only wording changed (time words, positional references, and self-reference); conclusions, numbers, and artifacts are unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 6 | Preparation | 2026-10-11 not affected: only wording changed (time words, positional references, and self-reference); conclusions, numbers, and artifacts are unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 10 | Supports | 2026-10-11 not affected: only wording changed (time words, positional references, and self-reference); conclusions, numbers, and artifacts are unchanged |
| D15 (Format Freeze Policy) settled item 2 | Not affected | 2026-10-11 not affected: only wording changed (time words, positional references, and self-reference); conclusions, numbers, and artifacts are unchanged |
| D8 (Core Index Structure) settled item 11 | Supports | 2026-10-11 not affected: only wording changed (time words, positional references, and self-reference); conclusions, numbers, and artifacts are unchanged |
| D8 (Core Index Structure) settled item 8 | Not affected | 2026-10-11 not affected: only wording changed (time words, positional references, and self-reference); conclusions, numbers, and artifacts are unchanged |

## Historical Versions

### 2026-09-16
- Tree table entry 148 → 200 (settled item 8 of D8 (Core Index Structure), widened by user settlement on 2026-09-16), the rig's `TREE_TABLE_ENTRY_BYTES` changed accordingly, and the artifact moved from `e145-self-describing-node-header-2026-09-14-round2.out` to `e145-self-describing-node-header-2026-09-16-tree-table-200.out`; which lines changed is in [experiments-history.md](../experiments-history.md) 2026-09-16 (part one).

### 2026-09-14
- Tree table entry 145 → 148 (the tree table entry of settled item 8 of D8 (Core Index Structure) was rearranged by user settlement on 2026-09-14 with header ID 8 added and the root pointer going from 83 to 86, `format-const: TREE_TABLE_ENTRY_BYTES = 148`, and the gate phase "format constants synchronized between kb and experiment source" turned red the same day) rerun:
  the artifact moved to `e145-self-describing-node-header-2026-09-14-round2.out`, and of 57 lines **3 lines** changed, all in `tree_table_bytes` under `name=mapping` (a 145→148, b 290→296, c 145→148);
  the sections on entry width, fanout, tree height, and scan header resolution are unchanged verbatim. All 9 unit tests green, all 11 mutations caught.
  Formerly: the three arms' `tree_table_bytes` were 145 / 290 / 145.

### 2026-09-13
- Created. The pre-run registration was written before the rig; after the first round of mutations three were not caught (arm A forgetting to subtract the payload field, the registry dropping 27, arm A taking the first width that passes), and per `.claude/rules/mutation-sampling.md` they were triaged three ways: the first two got sensitive assertions added, the third was judged an equivalent mutation, put on file, and replaced with "try only the first four widths"; the criteria and expected values were never changed.
