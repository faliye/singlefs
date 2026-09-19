#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r1：三处改法，只打在仓副本上（主工作区不打）。只在本腿的副本上量过、被攻过零轮。
P1（Z2）：已知红第 1 条收窄——抬 F 那一步之前的镜像上，新 F 那个 txg 的根按最新根指着的实例表是被抛弃的（F 落在回退留下的空档里）才算；
    FailureObservation 加一个字段 `raised_floor_lands_on_abandoned_root`，只在抬 F 那一步判红时按实现的读法（singlefs_core::recovery）现算。
P2（Z3）：冷启动 `recover` 报 Failed(…) 算失败（签名「冷启动读回」），不再记成 Applied 往下走。
P3（Z3）：快档的「路径跑到了」加一组：装得下的四种内容长度（0、[1, 载荷容量 − 2]、载荷容量 − 1、载荷容量）各至少一次发成（Ok）。
用法：python3 proposal-patch.py <副本根> [P1,P2,P3]
"""
import os, sys
root = sys.argv[1]
which = set((sys.argv[2] if len(sys.argv) > 2 else "P1,P2,P3").split(","))

def replace_once(path, old, new):
    full = os.path.join(root, path)
    text = open(full, encoding="utf-8").read()
    assert text.count(old) == 1, (path, old[:70], text.count(old))
    open(full, "w", encoding="utf-8").write(text.replace(old, new))

H = "crates/singlefs-harness/src/history.rs"
T = "crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs"

if "P1" in which:
    replace_once(H, "    /// 根环一圈的槽数 R × S（按 checker 从超级块读出的几何）；读不到时 None。\n    pub root_ring_slot_count: Option<u64>,\n}",
        "    /// 根环一圈的槽数 R × S（按 checker 从超级块读出的几何）；读不到时 None。\n    pub root_ring_slot_count: Option<u64>,\n"
        "    /// 抬 F 那一步判红时：抬之前的镜像上新 F 那个 txg 的根是不是被抛弃的（按最新根指着的实例表）；别的情形 None。\n"
        "    pub raised_floor_lands_on_abandoned_root: Option<bool>,\n}")
    replace_once(H, "    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)\n        && !observation.root_ring_has_turned()\n",
        "    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)\n        && observation.raised_floor_lands_on_abandoned_root == Some(true)\n        && !observation.root_ring_has_turned()\n")
    replace_once(H, "                panic: Some(panic),\n                newest_ring_root_txg,\n                root_ring_slot_count,\n            })",
        "                panic: Some(panic),\n                newest_ring_root_txg,\n                root_ring_slot_count,\n                raised_floor_lands_on_abandoned_root: None,\n            })")
    replace_once(H, "        panic,\n        newest_ring_root_txg,\n        root_ring_slot_count,\n    }\n}",
        "        panic,\n        newest_ring_root_txg,\n        root_ring_slot_count,\n        raised_floor_lands_on_abandoned_root: None,\n    }\n}\n\n"
        "/// 抬 F 那一步之前的镜像上，新 F 那个 txg 的根按最新根指着的实例表是不是被抛弃的；读不出或不是抬 F 的 Ok 结局就 None。\n"
        "fn raised_floor_lands_on_abandoned_root(image: &MemoryPool, outcome: Option<&StepOutcome>) -> Option<bool> {\n"
        "    let Some(StepOutcome::Applied(AppliedEffect::RaisedFloor { new_floor, .. })) = outcome else {\n        return None;\n    };\n"
        "    let superblock = singlefs_core::recovery::choose_superblock(image).ok()?;\n"
        "    let roots = singlefs_core::recovery::readable_roots(image, &superblock.region_devices, &superblock.geometry, &superblock.filesystem_identifier);\n"
        "    let newest = singlefs_core::recovery::choose_root(image, &superblock)?;\n"
        "    let table = singlefs_core::recovery::instance_table_of_root(image, &newest)?;\n"
        "    let at_floor: Vec<bool> = roots.iter().filter(|root| root.checkpoint_txg == *new_floor).map(|root| {\n"
        "        table.rows.iter().any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)\n"
        "    }).collect();\n"
        "    if at_floor.is_empty() { return None; }\n"
        "    Some(at_floor.iter().all(|abandoned| *abandoned))\n}")
    replace_once(H, "            } else {\n                image = pool.image();\n                checked_stream_length = stream_length;\n                let violations = violations_on(&image, &mut tally);\n                if !violations.is_empty() {\n                    return Some(failure_observation(\n                        &image,\n                        step_position,\n                        Some(operation.kind()),\n                        violations,\n                        None,\n                    ));\n                }",
        "            } else {\n                let image_before_this_step = std::mem::replace(&mut image, pool.image());\n                checked_stream_length = stream_length;\n                let violations = violations_on(&image, &mut tally);\n                if !violations.is_empty() {\n                    let mut observation = failure_observation(\n                        &image,\n                        step_position,\n                        Some(operation.kind()),\n                        violations,\n                        None,\n                    );\n                    observation.raised_floor_lands_on_abandoned_root = raised_floor_lands_on_abandoned_root(&image_before_this_step, outcomes.last());\n                    return Some(observation);\n                }")

if "P2" in which:
    replace_once(H, "            outcomes.push(outcome);\n            let stream_length = stream.operation_count();\n",
        "            outcomes.push(outcome);\n"
        "            if let Some(StepOutcome::Applied(AppliedEffect::Recovered { outcome: recovered })) = outcomes.last() {\n"
        "                if recovered.starts_with(\"Failed(\") {\n"
        "                    let detail = recovered.clone();\n"
        "                    return Some(failure_observation(&image, step_position, Some(operation.kind()), vec![(\"冷启动读回\", detail)], None));\n"
        "                }\n            }\n"
        "            let stream_length = stream.operation_count();\n")

if "P3" in which:
    replace_once(H, "    pub content_lengths_attempted: BTreeMap<&'static str, u64>,\n",
        "    pub content_lengths_attempted: BTreeMap<&'static str, u64>,\n    /// 发布的内容长度，按长度种类 → 入口返回 Ok 几次。\n    pub content_lengths_applied: BTreeMap<&'static str, u64>,\n")
    replace_once(H, "        for (member, count) in &other.refusals_by_member {\n",
        "        for (length, count) in &other.content_lengths_applied {\n            *self.content_lengths_applied.entry(length).or_insert(0) += count;\n        }\n        for (member, count) in &other.refusals_by_member {\n")
    replace_once(H, "        if let (Some(content), true) = (content, entrance_was_called) {\n",
        "        if let (Some(content), StepOutcome::Applied(_)) = (content, outcome) {\n            *self.content_lengths_applied.entry(content.length.name()).or_insert(0) += 1;\n        }\n        if let (Some(content), true) = (content, entrance_was_called) {\n")
    replace_once(T, "    assert!(\n        count_of(&tally.recovery_outcomes, \"FileRead\") >= 1,\n",
        "    for length in [\n        ContentLength::Empty,\n        ContentLength::InsideOneDataUnit { selector: 0 },\n        ContentLength::OneByteBelowDataUnitPayloadCapacity,\n        ContentLength::ExactlyDataUnitPayloadCapacity,\n    ] {\n"
        "        assert!(\n            tally.content_lengths_applied.get(length.name()).copied().unwrap_or(0) >= 1,\n            \"装得下的内容长度「{}」一次都没发成\",\n            length.name()\n        );\n    }\n"
        "    assert!(\n        count_of(&tally.recovery_outcomes, \"FileRead\") >= 1,\n")
print("patched", root, sorted(which))
