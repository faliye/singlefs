#!/usr/bin/env python3
"""在一棵仓副本里套上候选 A3 的模拟（m4-admission-delta-r1 攻方 opus-s2 的模型）：
`crates/singlefs-core/src/transaction.rs` 的 `judge_the_publish_before_any_read_or_write` 里，在走固定点
（`let settled = settle_the_allocation_record_tree(`）之前插一段式子那一判：需求取普通分配那一半的角色加 ckpt_cost（差量与几何算得出的上界），
已承诺预留取上一版记下的那个值。插入点在那份文件里必须恰好命中一次，否则退 3、一个字节不写。

用法：python3 apply_a3_emulation.py <仓副本根>
退出码：0 套上；2 用法错或文件不在；3 插入点命中不是一次。只改给的那棵副本，不碰别处。
"""
import pathlib
import sys

ANCHOR = "    let settled = settle_the_allocation_record_tree(\n"
INSERTION = """    // 攻方模型（m4-admission-delta-r1 opus-s2）：候选 A3 的模拟——式子那一判挪到全量规划（固定点）之前，
    // 需求取差量与几何算得出的上界：普通分配那一半的角色照计划数，固定点取 ckpt_cost；已承诺预留取上一版记下的那个值。
    let a3_formula_is_judged = match (allocator.space_admission(), deletion_and_lineage.step) {
        (SpaceAdmission::JudgedByTheFormula, step) => {
            !step.is_a_deletion_class_publish()
                && !deletion_and_lineage.snapshot.is_a_snapshot_destruction()
                && (publish_has_an_ordinary_allocation(&ordinary_allocation_roles)
                    || !relocations.is_empty())
        }
        (SpaceAdmission::SkippedByTheTestOnlySwitch, _) => false,
    };
    if a3_formula_is_judged {
        let bound_slots: u64 = ordinary_allocation_roles
            .iter()
            .map(|role| role.span_slots())
            .sum::<u64>()
            + checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
                previous,
                allocator,
                pool.code_two_tree_node_capacities(),
            )
            .0;
        let demand_bound: Vec<crate::admission::DemandOnDevice> =
            device_identities_of_the_accounting_rows
                .iter()
                .map(|device| crate::admission::DemandOnDevice {
                    device: *device,
                    bytes: crate::admission::BytesOnOneDevice::of_slots(bound_slots),
                })
                .collect();
        admit_on_every_device(
            &admission_reading_before_a_publish_on_a_ring_of(
                allocator,
                previous,
                crate::admission::committed_reservation_recorded_in(previous),
                pool.parameters.geometry.root_ring_slots_per_region,
            ),
            &demand_bound,
        )
        .map_err(PublishError::SpaceAdmissionRefused)?;
    }
"""


def main() -> int:
    if len(sys.argv) != 2:
        print("用法：python3 apply_a3_emulation.py <仓副本根>", file=sys.stderr)
        return 2
    target = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/transaction.rs"
    if not target.is_file():
        print(f"  ✗ 不在：{target}", file=sys.stderr)
        return 2
    text = target.read_text(encoding="utf-8")
    hits = text.count(ANCHOR)
    if hits != 1:
        print(f"  ✗ 插入点命中 {hits} 次（要恰好 1 次）：{target}", file=sys.stderr)
        return 3
    target.write_text(text.replace(ANCHOR, INSERTION + ANCHOR), encoding="utf-8")
    print(f"name=a3_emulation_applied target={target}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
