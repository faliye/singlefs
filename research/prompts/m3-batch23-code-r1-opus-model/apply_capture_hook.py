#!/usr/bin/env python3
"""m3-batch23-code-r1 攻方：在仓副本的 walk.rs 里加一个只记数、不改判定的钩子（不入库代码）。
记两样：删除态扫描与墓碑那一遍用的版本清单（txg、树表指针 86 字节），与 I-9.29 那一判滤没滤得掉被抛弃的根。
用法：apply_capture_hook.py <副本里的 crates/singlefs-checker/src/walk.rs>；每处锚点要恰好命中一次，不然退 1。"""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

def once(anchor, replacement):
    global text
    count = text.count(anchor)
    if count != 1:
        sys.exit(f"锚点命中 {count} 次：{anchor[:80]!r}")
    text = text.replace(anchor, replacement)

once(
    "    let system_configurations = chosen_system_configurations(reader);\n",
    "    attack_capture::clear();\n    let system_configurations = chosen_system_configurations(reader);\n",
)
once(
    "    judge_deletion_state_of_the_versions(\n        reader,\n        &versions_to_scan_the_deletion_state_of,\n",
    "    attack_capture::record(\n"
    "        versions_to_scan_the_deletion_state_of\n"
    "            .iter()\n"
    "            .map(|version| (version.checkpoint_txg, version.tree_table_pointer.to_vec()))\n"
    "            .collect(),\n"
    "        abandoned_roots_filtering == AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,\n"
    "    );\n"
    "    judge_deletion_state_of_the_versions(\n        reader,\n        &versions_to_scan_the_deletion_state_of,\n",
)
text += '''
/// m3-batch23-code-r1 攻方钩子（只在仓副本里）：记下版本清单与 I-9.29 那一判的滤法，判定一处不改。
#[doc(hidden)]
pub mod attack_capture {
    use std::cell::RefCell;
    thread_local! {
        static VERSIONS: RefCell<Option<(Vec<(u64, Vec<u8>)>, bool)>> = const { RefCell::new(None) };
    }
    pub(super) fn clear() {
        VERSIONS.with(|cell| *cell.borrow_mut() = None);
    }
    pub(super) fn record(versions: Vec<(u64, Vec<u8>)>, filtered: bool) {
        VERSIONS.with(|cell| *cell.borrow_mut() = Some((versions, filtered)));
    }
    /// 最近一次 `check_pool_image` 在这条线程上用的版本清单（txg、树表指针）与「最新根的实例表滤得掉被抛弃的根」；没走到那一步是 None。
    #[must_use]
    pub fn last() -> Option<(Vec<(u64, Vec<u8>)>, bool)> {
        VERSIONS.with(|cell| cell.borrow().clone())
    }
}
'''
open(path, "w", encoding="utf-8").write(text)
print("钩子已加")
