#!/usr/bin/env python3
"""在一棵仓副本里给 `settle_the_allocation_record_tree` 套上逐轮记录（m4-admission-delta-r2 攻方 opus 的模型）。

只在 `crates/singlefs-core/src/transaction.rs` 那一个函数体里改三处、文件末尾加一个模块，不改任何判定：
- 进第一轮之前开一条新的调用记录；
- 只释放、没取落点就回头的那一轮记一个空表；
- 取落点那一圈记下每个角色在分配器拷贝上取到的槽，一轮一张表。
每个锚点在那个函数体里必须恰好命中一次，否则退 3、一个字节不写。

用法：python3 apply_settle_trace.py <仓副本根>
退出码：0 套上；2 用法错或文件不在；3 锚点命中不是一次。只改给的那棵副本。
"""
import pathlib
import sys

FUNCTION_HEAD = "\nfn settle_the_allocation_record_tree(\n"
EDITS = [
    (
        "    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n",
        "    m4_admission_delta_r2_attack_trace::begin_a_settling_call();\n"
        "    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n",
    ),
    (
        "            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));\n"
        "            continue;\n",
        "            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));\n"
        "            m4_admission_delta_r2_attack_trace::note_a_round(Vec::new());\n"
        "            continue;\n",
    ),
    (
        "        for identity in &resolved.rewritten_roles {\n"
        "            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;\n"
        "        }\n",
        "        let mut m4_admission_delta_r2_attack_round: m4_admission_delta_r2_attack_trace::RoundPlacements =\n"
        "            Vec::new();\n"
        "        for identity in &resolved.rewritten_roles {\n"
        "            let placement = allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;\n"
        "            m4_admission_delta_r2_attack_round.push((*identity, placement.slot.0));\n"
        "        }\n"
        "        m4_admission_delta_r2_attack_trace::note_a_round(m4_admission_delta_r2_attack_round);\n",
    ),
]
MODULE = """
/// 攻方模型（m4-admission-delta-r2 opus）只在副本里套上的记录：[`settle_the_allocation_record_tree`] 每一次调用的每一轮在分配器拷贝上
/// 给每个角色取到的槽，按取的次序（只释放、没取落点就回头的那一轮记一个空表）。只读出、不进任何判定。
pub mod m4_admission_delta_r2_attack_trace {
    use super::TransactionUnit;
    use std::sync::Mutex;

    /// 一轮里每个角色取到的槽（两盘同槽，记槽号），按 bump 次序。
    pub type RoundPlacements = Vec<(TransactionUnit, u64)>;

    static SETTLING_CALLS: Mutex<Vec<Vec<RoundPlacements>>> = Mutex::new(Vec::new());

    pub fn begin_a_settling_call() {
        SETTLING_CALLS
            .lock()
            .expect("记录锁没有中毒")
            .push(Vec::new());
    }

    pub fn note_a_round(round: RoundPlacements) {
        if let Some(call) = SETTLING_CALLS.lock().expect("记录锁没有中毒").last_mut() {
            call.push(round);
        }
    }

    /// 交回到此为止记下的每一次调用（每次一张按轮的表），并清空。
    #[must_use]
    pub fn take_the_settling_calls() -> Vec<Vec<RoundPlacements>> {
        std::mem::take(&mut *SETTLING_CALLS.lock().expect("记录锁没有中毒"))
    }
}
"""


def main() -> int:
    if len(sys.argv) != 2:
        print("用法：python3 apply_settle_trace.py <仓副本根>", file=sys.stderr)
        return 2
    path = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/transaction.rs"
    if not path.is_file():
        print(f"不在：{path}", file=sys.stderr)
        return 2
    text = path.read_text(encoding="utf-8")
    if text.count(FUNCTION_HEAD) != 1:
        print(f"函数头命中 {text.count(FUNCTION_HEAD)} 次，不是一次", file=sys.stderr)
        return 3
    start = text.index(FUNCTION_HEAD)
    end = text.index("\n}\n", start) + 3
    body = text[start:end]
    for anchor, replacement in EDITS:
        hits = body.count(anchor)
        if hits != 1:
            print(f"锚点在函数体里命中 {hits} 次，不是一次：{anchor.splitlines()[0].strip()}", file=sys.stderr)
            return 3
        body = body.replace(anchor, replacement)
    path.write_text(text[:start] + body + text[end:] + MODULE, encoding="utf-8")
    print("name=settle_trace_applied edits=3 module=m4_admission_delta_r2_attack_trace")
    return 0


if __name__ == "__main__":
    sys.exit(main())
