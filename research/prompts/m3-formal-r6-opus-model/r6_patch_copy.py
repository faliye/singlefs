#!/usr/bin/env python3
"""m3-formal-r6 攻方副本补丁（只改冻结副本的拷贝，不碰仓）：
1. 副本变异开关 R6_MUTATION=rollback_not_durable（只在拷贝里）：回退那次发布照常写单元、发单元之后那道屏障，
   之后的屏障一道都不发、根槽写不带 FUA，就交回 Ok（坏实现：记录、根、系统配置轮换都还没持久就报回退做成）。
2. 把 r6probe 加进工作区成员。
用法：python3 r6_patch_copy.py <拷贝的 tree 目录>；每处替换断言恰好命中一次。"""
import pathlib
import sys

tree = pathlib.Path(sys.argv[1])


def replace_once(rel, old, new):
    p = tree / rel
    text = p.read_text()
    n = text.count(old)
    assert n == 1, f"{rel}: 命中 {n} 次"
    p.write_text(text.replace(old, new))


TX = "crates/singlefs-core/src/transaction.rs"
replace_once(TX,
    "            CommitStep::Barrier => {\n                if self.has_writes_since_barrier {\n"
    "                    for (_, device) in self.devices.iter_mut() {\n                        device.barrier()?;\n                    }\n"
    "                    self.has_writes_since_barrier = false;\n",
    "            CommitStep::Barrier => {\n                if self.has_writes_since_barrier && !r6_skip_durability() {\n"
    "                    for (_, device) in self.devices.iter_mut() {\n                        device.barrier()?;\n                    }\n"
    "                    self.has_writes_since_barrier = false;\n                    r6_after_a_barrier();\n")
replace_once(TX,
    "                    root_slot,\n                    WriteDurability::ForceUnitAccess,\n                )?;\n",
    "                    root_slot,\n                    if r6_skip_durability() { WriteDurability::Plain } else { WriteDurability::ForceUnitAccess },\n                )?;\n")
p = tree / TX
p.write_text(p.read_text() + '''
/// r6probe 副本变异（只在攻方草稿副本里）：0 关；1 屏障不发、根槽写不带 FUA；2 等下一道屏障发完再进 1。
static R6_SKIP_DURABILITY: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
fn r6_skip_durability() -> bool {
    R6_SKIP_DURABILITY.load(std::sync::atomic::Ordering::SeqCst) == 1
}
fn r6_after_a_barrier() {
    let _ = R6_SKIP_DURABILITY.compare_exchange(2, 1, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst);
}
/// r6probe 副本变异开关的设置口（只在攻方草稿副本里）：2 = 下一道屏障之后不再保证持久；0 = 关。
pub fn r6_set_skip_durability(mode: u8) {
    R6_SKIP_DURABILITY.store(mode, std::sync::atomic::Ordering::SeqCst);
}
''')

MOUNT = "crates/singlefs-core/src/mount.rs"
replace_once(MOUNT,
    "    match publish_version(&mut pool, allocator, plan, Some(&previous)) {\n        Ok(published) => {\n            *current = published;\n            Ok(RolledBack {\n",
    "    let r6_skip = std::env::var(\"R6_MUTATION\").as_deref() == Ok(\"rollback_not_durable\");\n"
    "    if r6_skip { crate::transaction::r6_set_skip_durability(2); }\n"
    "    let r6_published = publish_version(&mut pool, allocator, plan, Some(&previous));\n"
    "    if r6_skip { crate::transaction::r6_set_skip_durability(0); }\n"
    "    match r6_published {\n        Ok(published) => {\n            *current = published;\n            Ok(RolledBack {\n")

cargo = tree / "Cargo.toml"
members = cargo.read_text()
anchor = '    "crates/singlefs-checker-tier",\n]'
assert members.count(anchor) == 1, "工作区成员表的末尾不是预期的样子"
cargo.write_text(members.replace(anchor, '    "crates/singlefs-checker-tier",\n    "crates/r6probe",\n]'))
print("patched")
