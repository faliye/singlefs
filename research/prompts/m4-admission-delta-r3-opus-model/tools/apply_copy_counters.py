#!/usr/bin/env python3
"""在一棵仓副本里给发布路径套上三个只读出的计数（m4-admission-delta-r3 攻方 opus 的模型），不改任何判定。

只改 `crates/singlefs-core/src/transaction.rs` 三个函数体里各一处、文件末尾加一个模块：
- `publish_version_of_trees_relocating_units_after_a_check`：整份拷分配器（失败就换回去的那一份）之前记一次；
- `judge_the_publish_before_any_read_or_write`：进门记一次（一次发布在两道准入之前的那一判）；
- `settle_the_allocation_record_tree`：每一轮整份拷分配器（预演那一份）之前记一次。
每个锚点在它那个函数体里必须恰好命中一次，否则退 3、一个字节不写。计数按线程记（同一个测试二进制里几条用例并行也各数各的）。

用法：python3 apply_copy_counters.py <仓副本根>
退出码：0 套上；2 用法错或文件不在；3 函数头或锚点命中不是一次。只改给的那棵副本。
"""
import pathlib
import sys

EDITS = [
    (
        "\nfn publish_version_of_trees_relocating_units_after_a_check<Device: BlockDevice, Refusal, Check>(\n",
        "    let allocator_before_this_publish = allocator.clone();\n",
        "    m4_admission_delta_r3_attack_counts::note_a_whole_allocator_copy_before_the_judgement();\n"
        "    let allocator_before_this_publish = allocator.clone();\n",
    ),
    (
        "\nfn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(\n",
        "    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator\n",
        "    m4_admission_delta_r3_attack_counts::note_a_judgement();\n"
        "    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator\n",
    ),
    (
        "\nfn settle_the_allocation_record_tree(\n",
        "        let mut rehearsal = allocator.clone();\n",
        "        m4_admission_delta_r3_attack_counts::note_a_whole_allocator_copy_in_a_settling_round();\n"
        "        let mut rehearsal = allocator.clone();\n",
    ),
]
MODULE = """
/// 攻方模型（m4-admission-delta-r3 opus）只在副本里套上的计数：发布路径在两道准入之前整份拷了几次分配器、判了几次。只读出、不进任何判定。
pub mod m4_admission_delta_r3_attack_counts {
    use std::cell::Cell;

    /// 到此刻本线程上数到的三样。
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct CopiesAndJudgements {
        /// `publish_version_of_trees_relocating_units_after_a_check` 里失败就换回去的那一份整份拷贝。
        pub whole_allocator_copies_before_the_judgement: u64,
        /// `judge_the_publish_before_any_read_or_write` 进了几次。
        pub judgements: u64,
        /// `settle_the_allocation_record_tree` 每一轮预演的那一份整份拷贝。
        pub whole_allocator_copies_in_settling_rounds: u64,
    }

    thread_local! {
        static COUNTED: Cell<CopiesAndJudgements> = const {
            Cell::new(CopiesAndJudgements {
                whole_allocator_copies_before_the_judgement: 0,
                judgements: 0,
                whole_allocator_copies_in_settling_rounds: 0,
            })
        };
    }

    fn update(change: impl FnOnce(&mut CopiesAndJudgements)) {
        COUNTED.with(|counted| {
            let mut now = counted.get();
            change(&mut now);
            counted.set(now);
        });
    }

    pub fn note_a_whole_allocator_copy_before_the_judgement() {
        update(|now| now.whole_allocator_copies_before_the_judgement += 1);
    }

    pub fn note_a_judgement() {
        update(|now| now.judgements += 1);
    }

    pub fn note_a_whole_allocator_copy_in_a_settling_round() {
        update(|now| now.whole_allocator_copies_in_settling_rounds += 1);
    }

    /// 交回本线程到此刻数到的，并清零。
    #[must_use]
    pub fn take() -> CopiesAndJudgements {
        COUNTED.with(|counted| counted.replace(CopiesAndJudgements::default()))
    }
}
"""


def main() -> int:
    if len(sys.argv) != 2:
        print("用法：python3 apply_copy_counters.py <仓副本根>", file=sys.stderr)
        return 2
    path = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/transaction.rs"
    if not path.is_file():
        print(f"不在：{path}", file=sys.stderr)
        return 2
    text = path.read_text(encoding="utf-8")
    for head, anchor, replacement in EDITS:
        if text.count(head) != 1:
            print(f"函数头命中 {text.count(head)} 次，不是一次：{head.strip()}", file=sys.stderr)
            return 3
        start = text.index(head)
        end = text.index("\n}\n", start) + 3
        body = text[start:end]
        hits = body.count(anchor)
        if hits != 1:
            print(f"锚点在 {head.strip()} 的函数体里命中 {hits} 次，不是一次：{anchor.strip()}", file=sys.stderr)
            return 3
        text = text[:start] + body.replace(anchor, replacement) + text[end:]
    path.write_text(text + MODULE, encoding="utf-8")
    print("name=copy_counters_applied edits=3 module=m4_admission_delta_r3_attack_counts")
    return 0


if __name__ == "__main__":
    sys.exit(main())
