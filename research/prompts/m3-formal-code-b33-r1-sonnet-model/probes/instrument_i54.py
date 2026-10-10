#!/usr/bin/env python3
"""在草稿副本的 walk.rs 里加探针（只在副本里；撤回用 mut.py restore，本脚本走 mut.py apply-spec 同样的备份目录）。
探针打到 stderr：B33F1（树表指针非零而树表读不出，F1 那一支）、B33C（树表 0 条、根记录持有的分配记录树走不到叶，33-c 那一支）、
B33ZERO（树表指针全零、根记录那一项非零而那棵树走不到叶：二、末节那一处，今天整版不进评估）。线程名就是测试名。
用法：instrument_i54.py TREE"""
import os, shutil, sys
tree = sys.argv[1]
relative = "crates/singlefs-checker/src/walk.rs"
path = os.path.join(tree, relative)
text = open(path, encoding="utf-8").read()
os.makedirs(os.path.join(tree, ".mutbak"), exist_ok=True)
backup = os.path.join(tree, ".mutbak", relative.replace("/", "__"))
assert not os.path.exists(backup), "已有备份没撤回"
shutil.copy2(path, backup)

def swap(old, new):
    global text
    assert text.count(old) == 1, old
    text = text.replace(old, new, 1)

swap("""            if tree_table_read_back.is_none() {
                versions_whose_tree_table_does_not_read_back.push(format!(""",
"""            if tree_table_read_back.is_none() {
                eprintln!("B33F1 thread={:?} root=({},{})", std::thread::current().name(), root.instance, root.checkpoint_txg);
                versions_whose_tree_table_does_not_read_back.push(format!(""")
swap("""            && !this_version_allocation_record_tree_walked_to_its_leaves
        {
            versions_whose_root_record_allocation_record_tree_does_not_read_back.push(format!(""",
"""            && !this_version_allocation_record_tree_walked_to_its_leaves
        {
            eprintln!("B33C thread={:?} root=({},{})", std::thread::current().name(), root.instance, root.checkpoint_txg);
            versions_whose_root_record_allocation_record_tree_does_not_read_back.push(format!(""")
swap("""        if this_version_carries_trees_and_a_zero_root_record_allocation_tree_root {
            versions_where_the_root_record_allocation_tree_root_is_zero.push((""",
"""        if tree_table_pointer.all_zero
            && !parse_node_pointer(&root.record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT]).all_zero
            && !this_version_allocation_record_tree_walked_to_its_leaves
        {
            eprintln!("B33ZERO thread={:?} root=({},{})", std::thread::current().name(), root.instance, root.checkpoint_txg);
        }
        if this_version_carries_trees_and_a_zero_root_record_allocation_tree_root {
            versions_where_the_root_record_allocation_tree_root_is_zero.push((""")
open(path, "w", encoding="utf-8").write(text)
print("INSTRUMENTED", relative)
