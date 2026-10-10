#!/usr/bin/env python3
"""m3-batch29-code-r1 攻方：从冻结副本的 walk.rs 造两份变体（只在攻方仓副本里用，不入库代码）。
  probe：加只计数的原子探针（不改任何判定），六格：
    0 hv          child_unit_class_other_than_the_entry_type_names 交回 Some（孩子是另一类那一支）
    1 hb_inode    judge_an_inode_entry_whose_child_does_not_read_back（inode 树孩子两份都读不出）
    2 hb_container 墓碑树 / 谱系树容器两份都读不出那一支
    3 unwalked    judge_every_copy_of_a_child_the_walk_does_not_go_into 被调
    4 unwalked_mismatch 上一格里一份对不上
    5 l1_type0    墓碑树 / 谱系树层级 1 节点里类型段 0 的条目（walk_the_internal_node_below_a_container_tree_entry 只记走读失败那一支）
  fix：probe 之上再加攻方改法 F-L0（零轮）：第 5 格那一支照第 29 批的共用函数判每一份副本（不登记引用，引用照旧不记）。
  anylen：probe 之上把第 29 批 F-α 的长度口径换成「16 KiB 与 32 KiB 两种长度有一种对得上就算对得上」（攻方对照臂 F-任一长，零轮），只为量口径。
用法：python3 make_variants.py <冻结副本 walk.rs> <出 probe 路径> <出 fix 路径> <出 anylen 路径>
每一处替换要求旧串恰好命中一次，否则退出 3。
"""
import sys

PROBE_DEFS = '''
/// m3-batch29-code-r1 攻方探针：只计数，不改判定。
pub static OPUS_B29_PROBE: [std::sync::atomic::AtomicU64; 6] = [
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
];
fn opus_b29_probe(index: usize) {
    OPUS_B29_PROBE[index].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // 设了 OPUS_B29_PROBE_LOG 时每走到一次往那个文件追加一行「格号 线程名」（libtest 的线程名就是用例名）；只写这份日志，不改判定
    if let Ok(path) = std::env::var("OPUS_B29_PROBE_LOG") {
        use std::io::Write as _;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let line = format!("{index} {}\n", std::thread::current().name().unwrap_or("-"));
            let _ = file.write_all(line.as_bytes());
        }
    }
}
/// 六格探针的当前计数。
#[must_use]
pub fn opus_b29_probe_counts() -> [u64; 6] {
    std::array::from_fn(|index| OPUS_B29_PROBE[index].load(std::sync::atomic::Ordering::Relaxed))
}
'''

PROBES = [
    ("    class_other_than_the_entry_type_names\n}\n\n/// 走读不往下读的孩子的一份副本",
     "    if class_other_than_the_entry_type_names.is_some() {\n        opus_b29_probe(0);\n    }\n    class_other_than_the_entry_type_names\n}\n\n/// 走读不往下读的孩子的一份副本"),
    ("    birth_tree_in_the_child_pointer_head: u64,\n) {\n",
     "    birth_tree_in_the_child_pointer_head: u64,\n) {\n    opus_b29_probe(1);\n"),
    ("            let identity_birth_tree = identity.0;\n            self.judgements.judge(\n                identity_invariant,\n                child.birth_tree == identity_birth_tree,",
     "            opus_b29_probe(2);\n            let identity_birth_tree = identity.0;\n            self.judgements.judge(\n                identity_invariant,\n                child.birth_tree == identity_birth_tree,"),
    ("        let mut some_copy_matches = false;\n",
     "        let mut some_copy_matches = false;\n        opus_b29_probe(3);\n"),
    ("            some_copy_matches |= matches;\n",
     "            if !matches {\n                opus_b29_probe(4);\n            }\n            some_copy_matches |= matches;\n"),
    ("        if node.level <= 1 {\n            self.walk_failures.push(format!(",
     "        if node.level <= 1 {\n            opus_b29_probe(5);\n            self.walk_failures.push(format!("),
]

FIX = [
    ("        if node.level <= 1 {\n            opus_b29_probe(5);\n            self.walk_failures.push(format!(",
     "        if node.level <= 1 {\n            opus_b29_probe(5);\n            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &format!(\"{name}内部条目的子指针\"));\n            self.walk_failures.push(format!("),
]

ANYLEN = [
    ("    let lengths_to_compare = match length_of_the_class_in_the_header {\n        Some(unit_bytes) => vec![unit_bytes],\n        None => vec![node_bytes(), data_unit_bytes()],\n    };\n",
     "    let _ = length_of_the_class_in_the_header;\n    let lengths_to_compare = vec![node_bytes(), data_unit_bytes()];\n"),
]


def replace_each(text, pairs, what):
    for old, new in pairs:
        count = text.count(old)
        if count != 1:
            print(f"{what}: 旧串命中 {count} 次：{old[:60]!r}", file=sys.stderr)
            sys.exit(3)
        text = text.replace(old, new)
    return text


def main():
    source, probe_out, fix_out, anylen_out = sys.argv[1:5]
    with open(source, encoding="utf-8") as handle:
        text = handle.read()
    probed = replace_each(text, PROBES, "probe") + PROBE_DEFS
    fixed = replace_each(probed, FIX, "fix")
    anylen = replace_each(probed, ANYLEN, "anylen")
    for path, content in [(probe_out, probed), (fix_out, fixed), (anylen_out, anylen)]:
        with open(path, "x", encoding="utf-8") as handle:
            handle.write(content)


if __name__ == "__main__":
    main()
