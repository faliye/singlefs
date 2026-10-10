#!/usr/bin/env python3
"""用法：check_constants.py <冻结树根>   cost_model.py 里用到的格式常量与规划规则，逐个在冻结树里 grep -nF 一遍，打 文件:行号 和命中的原行；
一个都不许缺（缺了退 1）。再把几条派生数（477、150、135）用冻结树里的头宽公式重算一遍。"""
import subprocess, sys, os
root = sys.argv[1]
ITEMS = [
    ("记账条目 34 字节", "crates/singlefs-format/src/lib.rs", "pub const ACCOUNTING_ENTRY_BYTES: u64 = 34;"),
    ("记账 key 22 字节", "crates/singlefs-format/src/lib.rs", "pub const ACCOUNTING_KEY_BYTES: u64 = 22;"),
    ("记账内部条目 108 字节", "crates/singlefs-format/src/lib.rs", "pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108;"),
    ("记账节点头 159 字节", "crates/singlefs-format/src/lib.rs", "pub const ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES: u64 = 159;"),
    ("码 2 节点 16384 字节", "crates/singlefs-format/src/lib.rs", "pub const NODE_BYTES: u64 = 16384;"),
    ("码 3 容器 32768 字节", "crates/singlefs-format/src/lib.rs", "pub const DATA_UNIT_BYTES: u64 = 32768;"),
    ("打包容器记录区起点 136", "crates/singlefs-format/src/lib.rs", "pub const PACKED_UNIT_RECORDS_OFFSET: u64 = 136;"),
    ("谱系记录 16 字节", "crates/singlefs-format/src/lib.rs", "pub const LINEAGE_RECORD_BYTES: u64 = 16;"),
    ("谱系容器 2039 条", "crates/singlefs-format/src/lib.rs", "pub const LINEAGE_CONTAINER_RECORDS: u64 = 2039;"),
    ("墓碑记录 40 字节", "crates/singlefs-format/src/lib.rs", "pub const TOMBSTONE_RECORD_BYTES: u64 = 40;"),
    ("容器树内部条目 120 字节", "crates/singlefs-format/src/lib.rs", "pub const INODE_INTERNAL_ENTRY: u64 = 120;"),
    ("子指针 86 字节", "crates/singlefs-format/src/lib.rs", "pub const NODE_POINTER_BYTES: u64 = 86;"),
    ("一代 15 行", "crates/singlefs-format/src/lib.rs", "pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;"),
    ("树表 11 条", "crates/singlefs-format/src/lib.rs", "pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;"),
    ("统计量 12 = inode 号水位", "crates/singlefs-core/src/records.rs", "pub const STATISTIC_INODE_WATERMARK: u16 = 12;"),
    ("记账 key 字段宽 2+8+4+8", "crates/singlefs-core/src/code_two_tree.rs", "pub const ACCOUNTING: Self = Self(&[2, 8, 4, 8]);"),
    ("叶从中间切（左半 ⌈n÷2⌉）", "crates/singlefs-core/src/code_two_tree.rs", "let right_keys = keys.split_off(keys.len().div_ceil(2));"),
    ("内部节点同样从中间切", "crates/singlefs-core/src/code_two_tree.rs", "let right_children = children.split_off(children.len().div_ceil(2));"),
    ("先删后插", "crates/singlefs-core/src/code_two_tree.rs", "for deleted in previous_keys.difference(keys_after_this_publish) {"),
    ("再插这一版有而上一版没有的", "crates/singlefs-core/src/code_two_tree.rs", "for inserted in keys_after_this_publish.difference(&previous_keys) {"),
    ("路径上的节点整条 COW", "crates/singlefs-core/src/code_two_tree.rs", "/// 这次碰过它：路径上的节点整条 COW（D8（核心索引结构） 已定项 11 ①）。"),
    ("记账行每发布现算 15 行", "crates/singlefs-core/src/transaction.rs", "fn accounting_entries_of_this_publish("),
    ("记账树整树按这一版的行重装", "crates/singlefs-core/src/transaction.rs", "let accounting_built = build_multi_level_tree("),
    ("容器树根满长高", "crates/singlefs-core/src/container_tree.rs", "pub fn after_a_container_is_opened_at_the_right_end(&self, entries_per_node: usize) -> Self {"),
    ("容器树根 135 条", "crates/singlefs-core/src/container_tree.rs", "pub fn containers_one_container_tree_root_holds() -> usize {"),
    ("回退的水位：环里读得出的根", "crates/singlefs-core/src/mount.rs", "let ring_roots = readable_roots("),
    ("回退的水位：取环里的 max", "crates/singlefs-core/src/mount.rs", "let inode_number_watermark = highest_inode_number_watermark_in_the_ring("),
    ("回退的水位：与内存里的取 max", "crates/singlefs-core/src/mount.rs", "ring.max(current.inode_number_watermark())"),
    ("回退读 cur 的实例表（读不出就拒）", "crates/singlefs-core/src/mount.rs", "let current_table = instance_table_chain_of_root(&*devices, &current.root)"),
    ("表读不出：回退拒", "crates/singlefs-core/src/mount.rs", ".map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?"),
    ("候选集按 cur 的实例表判被抛弃", "crates/singlefs-core/src/mount.rs", "&current_table,"),
    ("按实例表判抛弃的函数", "crates/singlefs-core/src/recovery.rs", "pub fn root_is_abandoned_by_the_instance_table("),
    ("按实例表判抛弃的判法", "crates/singlefs-core/src/recovery.rs", ".any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)"),
    ("读不出的根不算", "crates/singlefs-core/src/mount.rs", "Ok(RebuiltVersion::WithoutFile) | Err(_) => None,"),
    ("树 ID 水位取环里全部根（含被抛弃的）", "crates/singlefs-core/src/mount.rs", ".fold(current.root.tree_identifier_watermark, u64::max);"),
    ("谱系记录追加", "crates/singlefs-core/src/mount.rs", "let lineage_record = LineageRecord {"),
    ("崩溃状态闭式", "crates/singlefs-harness/src/segments.rs", "/// E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。"),
    ("池级 checker 已实现 84 条", "crates/singlefs-checker/src/image.rs", "pub const IMPLEMENTED_INVARIANTS: [&str; 84] = ["),
]
bad = 0
for name, rel, text in ITEMS:
    path = os.path.join(root, rel)
    out = subprocess.run(["grep", "-nF", "--", text, path], capture_output=True, text=True).stdout.splitlines()
    if not out:
        print(f"✗ 缺 {name}: {rel} 里没有「{text}」"); bad += 1
    else:
        print(f"✓ {name}: {rel}:{out[0].split(':', 1)[0]}" + (f"（另 {len(out)-1} 处）" if len(out) > 1 else ""))
# 头宽公式（unit.rs:143 的实现）：key 宽 22 → 159；容量 = (16384 − 头) ÷ 条目宽
text = open(os.path.join(root, "crates/singlefs-core/src/unit.rs"), encoding="utf-8").read()
assert "index_node_plain_header_end(key_width) + reserved_bytes()" in text
hdr = lambda k: 137 + k
leaf, internal, rootc = (16384 - hdr(22)) // 34, (16384 - hdr(22)) // 108, (16384 - hdr(8)) // 120
print(f"派生：叶 {leaf} 条、内部 {internal} 个孩子、容器树根 {rootc} 个孩子（头宽按 137 + key 宽，key 宽 22 对 159）")
assert (hdr(22), leaf, internal, rootc) == (159, 477, 150, 135)
print(f"check-constants 查了 {len(ITEMS)} 项，缺 {bad} 项")
sys.exit(1 if bad else 0)
