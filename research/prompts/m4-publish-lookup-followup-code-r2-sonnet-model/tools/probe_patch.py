#!/usr/bin/env python3
"""probe_patch.py <探针副本根>：在探针副本的 transaction.rs 里给第 11 批没碰的几处线性查找加比较次数计数。

只用于量法（草稿目录里的副本），不进仓里的 crates。每处原文必须恰好命中一次，否则报错退出。
计数槽：0 首文件数据单元 rewritten.contains；1 分配记录树节点 rewritten_units.iter().find；2 has_a_placement 里 extent_lower_nodes.iter().any；
3 照抄文件下段节点 extent_lower_nodes.iter().find；4 装下段节点时 lower_nodes / file_lower_nodes 找孩子；5 搬迁 previous_roles_replaced.contains；
6 TransactionOutput::unit；7 file_other_than_the_first；8 上段叶条目按 inode 范围筛；9 恢复里按 inode 号找 inode 记录（recovery.rs）。
"""
import sys, pathlib
path = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/transaction.rs"
text = path.read_text(encoding="utf-8")

def once(old, new):
    global text
    n = text.count(old)
    if n != 1:
        sys.exit(f"anchor hits {n}: {old[:80]!r}")
    text = text.replace(old, new)

once('''const EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION: &str =
    "八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";
''', '''const EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION: &str =
    "八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";

pub static PROBE_COMPARISONS: [std::sync::atomic::AtomicU64; 16] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 16];
pub fn probe_count(slot: usize) {
    PROBE_COMPARISONS[slot].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}
pub fn probe_contains<T: PartialEq>(slot: usize, slice: &[T], wanted: &T) -> bool {
    for item in slice {
        probe_count(slot);
        if item == wanted {
            return true;
        }
    }
    false
}
pub fn probe_take() -> [u64; 16] {
    let mut taken = [0u64; 16];
    for (slot, cell) in PROBE_COMPARISONS.iter().enumerate() {
        taken[slot] = cell.swap(0, std::sync::atomic::Ordering::Relaxed);
    }
    taken
}
''')
once("Some(_) if !rewritten.contains(&identity) => carried_unit(",
     "Some(_) if !probe_contains(0, rewritten, &identity) => carried_unit(")
once('''                .rewritten_units
                .iter()
                .find(|unit| unit.identity == identity)
            {
                Some(allocation_record_tree_node_unit)''', '''                .rewritten_units
                .iter()
                .inspect(|_| probe_count(1))
                .find(|unit| unit.identity == identity)
            {
                Some(allocation_record_tree_node_unit)''')
once('''                    file.extent_lower_nodes
                        .iter()
                        .any(|(position_in_the_previous_version, _)| {''', '''                    file.extent_lower_nodes
                        .iter()
                        .inspect(|_| probe_count(2))
                        .any(|(position_in_the_previous_version, _)| {''')
once('''                                .extent_lower_nodes
                                .iter()
                                .find(|(position_in_the_previous_version, _)| {''', '''                                .extent_lower_nodes
                                .iter()
                                .inspect(|_| probe_count(3))
                                .find(|(position_in_the_previous_version, _)| {''')
once('''                            let (_, pointer) = lower_nodes
                                .iter()
                                .find(|(built, _)| *built == child)''', '''                            let (_, pointer) = lower_nodes
                                .iter()
                                .inspect(|_| probe_count(4))
                                .find(|(built, _)| *built == child)''')
once('''                                let (_, pointer) = file_lower_nodes
                                    .iter()
                                    .find(|(built, _)| *built == child)''', '''                                let (_, pointer) = file_lower_nodes
                                    .iter()
                                    .inspect(|_| probe_count(4))
                                    .find(|(built, _)| *built == child)''')
once("!previous_roles_replaced.contains(&role)", "!probe_contains(5, &previous_roles_replaced, &role)")
once('''        self.units
            .iter()
            .find(|unit| unit.identity == identity)
            .expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)''', '''        self.units
            .iter()
            .inspect(|_| probe_count(6))
            .find(|unit| unit.identity == identity)
            .expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)''')
once('''        self.files_other_than_the_first
            .iter()
            .find(|file| file.inode == inode)
    }''', '''        self.files_other_than_the_first
            .iter()
            .inspect(|_| probe_count(7))
            .find(|file| file.inode == inode)
    }''')
once('''                    upper_leaf_entries_of_the_files_other_than_the_first
                        .iter()
                        .filter(|entry| {''', '''                    upper_leaf_entries_of_the_files_other_than_the_first
                        .iter()
                        .inspect(|_| probe_count(8))
                        .filter(|entry| {''')
path.write_text(text, encoding="utf-8")
# 恢复路径：data_units_judged_against_the_inode_records 里按 inode 号找 inode 记录（槽 9）
rpath = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/recovery.rs"
rtext = rpath.read_text(encoding="utf-8")
old = """            .flat_map(|container| container.contents.records.iter())
            .find(|inode_record| inode_record.inode == inode)"""
assert rtext.count(old) == 1, "recovery anchor"
rpath.write_text(rtext.replace(old, """            .flat_map(|container| container.contents.records.iter())
            .inspect(|_| crate::transaction::probe_count(9))
            .find(|inode_record| inode_record.inode == inode)"""), encoding="utf-8")
print("probe_patch ok")
