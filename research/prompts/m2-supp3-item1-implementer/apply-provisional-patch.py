# 用法：apply-provisional-patch.py <副本根>。把「已知红」清单第 1 条（待主 agent 定）与它的复现用例加进副本里的 history.rs 与测试文件。
import sys
root = sys.argv[1]
path = f"{root}/crates/singlefs-harness/src/history.rs"
text = open(path, encoding="utf-8").read()
anchor = "fn ring_turn_leaves_allocated_statistic_above_walked(observation: &FailureObservation) -> bool {\n    observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation)\n}\n"
assert text.count(anchor) == 1
text = text.replace(anchor, anchor + """
fn raise_after_rollback_leaves_allocated_statistic_above_walked(
    observation: &FailureObservation,
) -> bool {
    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)
        && !observation.root_ring_has_turned()
        && only_allocated_statistic_above_walked(observation)
}
""")
head = "pub const KNOWN_RED_FORMS: [KnownRedForm; 1] = [KnownRedForm {\n    closeout_table_row:"
assert text.count(head) == 1
text = text.replace(head, "pub const KNOWN_RED_FORMS: [KnownRedForm; 2] = [\n    KnownRedForm {\n        closeout_table_row:")
shape = "\n    shape: \"根环转过一圈之后"
assert text.count(shape) == 1
text = text.replace(shape, "\n        shape: \"根环转过一圈之后")
tail = "    matches: ring_turn_leaves_allocated_statistic_above_walked,\n}];"
assert text.count(tail) == 1
text = text.replace(tail, """        matches: ring_turn_leaves_allocated_statistic_above_walked,
    },
    KnownRedForm {
        closeout_table_row: "增补 2 收口表第 ？ 行（待主 agent 立行；随机历史第 1 件的新发现）",
        shape: "回退之后把 F 抬进回退目标根与新实例第一次发布之间（F 那个 txg 上没有有效根）：抬 F 那一步之后 I-3.1（已分配统计对得上） 红、记账的已分配大于遍历全部有效根得到的、别的不变量都不红、没有 panic、根环没转圈",
        matches: raise_after_rollback_leaves_allocated_statistic_above_walked,
    },
];""")
open(path, "w", encoding="utf-8").write(text)
test_path = f"{root}/crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs"
text = open(test_path, encoding="utf-8").read()
anchor = "/// 同一个种子跑两次，每一步的结局、收尾、计数逐项相同"
assert text.count(anchor) == 1
addition = open("/tmp/claude-1000/m2-supp3-item1/provisional-reproduction-test.rs", encoding="utf-8").read()
text = text.replace(anchor, addition + anchor)
use_anchor = "use singlefs_harness::SharedStream;\n"
assert text.count(use_anchor) == 1
text = text.replace(use_anchor, use_anchor + "use singlefs_harness::history::{AppliedEffect, FloorTargetChoice, RecordReuse, RollbackTargetChoice, StepOutcome};\nuse singlefs_core::address::CheckpointTxg;\n")
open(test_path, "w", encoding="utf-8").write(text)
print("patched", root)
