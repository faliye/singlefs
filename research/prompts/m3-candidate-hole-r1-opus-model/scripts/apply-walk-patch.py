#!/usr/bin/env python3
"""用法：apply-walk-patch.py <仓副本根>。把攻方原型的洞表与候选判法并进副本的池级 checker，给副本 core 的回收门槛加一个坏写者开关。
每处原文必须在文件里恰好命中一次，命中不是一次就停（不改半份）。只改副本，不碰冻结副本与主仓。"""
import pathlib
import shutil
import sys

tree = pathlib.Path(sys.argv[1])
model = pathlib.Path(__file__).resolve().parent.parent
walk = tree / "crates/singlefs-checker/src/walk.rs"
mount = tree / "crates/singlefs-core/src/mount.rs"


def replace_once(text, old, new, what):
    count = text.count(old)
    if count != 1:
        sys.exit(f"{what}：原文命中 {count} 次，不是 1 次")
    return text.replace(old, new)


w = walk.read_text()
if "hole_proto.rs" in w:
    sys.exit("walk.rs 已经打过原型补丁")
w = replace_once(
    w,
    """    if previous_instances
        .iter()
        .any(|previous_instance| bridged_from(*previous_instance))
    {
        return HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows;
    }
    HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
}""",
    """    let by_rows = if previous_instances
        .iter()
        .any(|previous_instance| bridged_from(*previous_instance))
    {
        HoleInTheCandidateSet::HoleBridgedByTheInstanceTableRows
    } else {
        HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows
    };
    proto_bridge_override(previous_txg, version_txg, by_rows)
}""",
    "gap-row 收尾",
)
w = replace_once(
    w,
    """    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");""",
    """    proto_hole_table_store(
        &roots,
        roots[newest_index].2.instance,
        newest_txg,
        &instance_table_rows,
        &versions_applied_only_by_records,
        &journal_records_by_device,
        window,
        effective_rollback_floor,
    );
    for version in &versions_applied_only_by_records {
        let mismatches_before = walk.judgements.violation_count("I-2.1");""",
    "洞表落点",
)
w = replace_once(
    w,
    """    let accounting_was_written_under_the_effective_floor =
        roots[newest_index].2.rollback_floor == effective_rollback_floor;
""",
    """    let accounting_was_written_under_the_effective_floor =
        roots[newest_index].2.rollback_floor == effective_rollback_floor;
    let proto_i31_judge = proto_i31_per_device(
        reader,
        filesystem_identifier_low,
        newest_root_view,
        &roots,
        &roots_below_the_floor,
        &walked_slots_of_every_walked_version,
        effective_rollback_floor,
        deferred_threshold,
        &mut index_node_cache,
    );
""",
    "I-3.1 判法开关",
)
old_judge = """            if accounting_was_written_under_the_effective_floor {
                judgements.judge("I-3.1", allocated == Some(walked), || {
                    format!(
                        "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}（其中隔离豁免 {quarantined_exempted}、F 之下仍在 defer 的 {deferred_below_the_floor}）{}",
                        mechanism()
                    )
                });
            }"""
new_judge = """            if accounting_was_written_under_the_effective_floor {
                let proto_verdict = proto_i31_judge(*device, walked, deferred_below_the_floor);
                proto_note(format!("I31 device={device} allocated={allocated:?} walked={walked} deferred_high={deferred_below_the_floor} proto={proto_verdict:?}"));
                match proto_verdict {
                    ProtoI31::Exact => judgements.judge("I-3.1", allocated == Some(walked), || {
                        format!(
                            "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}（其中隔离豁免 {quarantined_exempted}、F 之下仍在 defer 的 {deferred_below_the_floor}）{}",
                            mechanism()
                        )
                    }),
                    ProtoI31::NotApplicable => judgements.not_applicable(
                        "I-3.1",
                        "原型 HOLE_I31=na：洞表里有存在过却走不到的那一代",
                    ),
                    ProtoI31::Red { missing } => judgements.judge("I-3.1", false, || {
                        format!("盘 {device}：（原型 e1ws 逐槽）遍历走到的槽里有 {missing} 个在最新根的账里没有记录{}", mechanism())
                    }),
                    ProtoI31::Records { low, high, uncounted } => judgements.judge(
                        "I-3.1",
                        uncounted == 0 && allocated.is_some_and(|value| value >= low && value <= high),
                        || {
                            format!("盘 {device}：（原型 rc 记录形）记账的已分配 {allocated:?} 不在按记录算的 [{low}, {high}] 里，或遍历走到的槽里有 {uncounted} 个不在记账该算的记录里{}", mechanism())
                        },
                    ),
                    ProtoI31::Interval { upper } => judgements.judge(
                        "I-3.1",
                        allocated.is_some_and(|value| value >= walked && value <= upper),
                        || {
                            format!(
                                "盘 {device}：（原型区间形）记账的已分配 {allocated:?} 不在 [{walked}, {upper}] 里{}",
                                mechanism()
                            )
                        },
                    ),
                }
            }"""
w = replace_once(w, old_judge, new_judge, "I-3.1 判定")
w += '\ninclude!("hole_proto.rs");\n'
shutil.copy(model / "walk-patch/hole_proto.rs", walk.parent / "hole_proto.rs")
walk.write_text(w)

m = mount.read_text()
m = replace_once(
    m,
    """    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}""",
    """    // m3-candidate-hole-r1 攻方原型的坏写者开关（只在攻方副本里）：HOLE_BAD_WRITER=m4a 取变异表第 148 行那一改法（门槛 = 最旧有效根 + 1）。
    if std::env::var("HOLE_BAD_WRITER").is_ok_and(|value| value == "m4a") {
        return oldest_valid_root.map_or(effective_floor, |oldest| {
            effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest })
        });
    }
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}""",
    "回收门槛坏写者开关",
)
mount.write_text(m)
print("patched", walk, mount)
