#!/usr/bin/env python3
"""gen_alt3.py：记账树单独退回线性，换另外两条用例各判一次；写 alt3-mutations.tsv。"""
H = "-p singlefs-harness --test publish_lookup_by_identity -- --exact "
old = """                let carried = units_of_the_previous_version
                    .expect(
                        "照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来",
                    )
                    .unit(tree.role_of_node(*previous_position, &previous_tree.shape));"""
def new(tree):
    return f"""                let carried = if !matches!(tree, MultiLevelCodeTwoTree::{tree}) {{
                    units_of_the_previous_version
                        .expect("照抄的节点只出现在接着上一版的发布里：上一版的单元索引与上一版一起交进来")
                        .unit(tree.role_of_node(*previous_position, &previous_tree.shape))
                }} else {{
                    previous_version.unit(tree.role_of_node(*previous_position, &previous_tree.shape))
                }};"""
T1 = "a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site"
T2 = "a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once"
rows = [("ALT3a 记账树单独退回线性（无快照整份重写用例判）", "Accounting", T1),
        ("ALT3b 记账树单独退回线性（快照下整份重写与删快照用例判）", "Accounting", T2),
        ("ALT3c deadlist 树单独退回线性（无快照整份重写用例判）", "Deadlist", T1)]
with open("alt3-mutations.tsv", "w", encoding="utf-8") as out:
    for name, tree, test in rows:
        out.write("\t".join([name, "crates/singlefs-core/src/transaction.rs", old.replace("\n", "\\n"), new(tree).replace("\n", "\\n"), H + test, test]) + "\n")
