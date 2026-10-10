#!/usr/bin/env python3
"""第三层补丁（接在 apply_patch2.py 之后）：数事件。每次发布数「删空摘片、左吸收右、均分借」各几次，累计成 name=candidate_cost 行
（只在 E177_CANDIDATE 或 E177_MERGE 非默认时输出，基线产物逐字不变）。用法：apply_patch3.py <第二层输出.rs> <输出.rs>"""
import sys
source = open(sys.argv[1], encoding="utf-8").read()

def replace_once(text, old, new):
    count = text.count(old)
    if count != 1:
        sys.exit(f"旧串出现 {count} 次，不是 1 次：{old[:80]!r}")
    return text.replace(old, new)

source = replace_once(source, "    range_tombstones: Vec<(u64, u64, u64, u64)>,\n}\n", "    range_tombstones: Vec<(u64, u64, u64, u64)>,\n    drops: u64,\n    absorbs: u64,\n    borrows: u64,\n}\n")
source = replace_once(source, "        range_tombstones: Vec::new(),\n    };\n", "        range_tombstones: Vec::new(),\n        drops: 0,\n        absorbs: 0,\n        borrows: 0,\n    };\n")
# 删空摘片
source = replace_once(source, "    if plan.inode_containers[current].records.is_empty() {\n        note_removed(plan, current, &mut first_removed_position);\n", "    if plan.inode_containers[current].records.is_empty() {\n        plan.drops += 1;\n        note_removed(plan, current, &mut first_removed_position);\n")
# 左吸收右（M2 循环里）
source = replace_once(source, "                        note_removed(plan, at + 1, &mut first_removed_position);\n", "                        plan.absorbs += 1;\n                        note_removed(plan, at + 1, &mut first_removed_position);\n")
# 借
source = replace_once(source, "                        plan.inode_containers[current].records.extend(moved);\n", "                        plan.borrows += 1;\n                        plan.inode_containers[current].records.extend(moved);\n")
# 借完那一对的吸收
source = replace_once(source, "                                note_removed(plan, current + 2, &mut first_removed_position);\n", "                                plan.absorbs += 1;\n                                note_removed(plan, current + 2, &mut first_removed_position);\n")
# Simulation 累计字段
source = replace_once(source, "    slots_reclaimed_by_publish: BTreeMap<u64, u64>,\n}\n", "    slots_reclaimed_by_publish: BTreeMap<u64, u64>,\n    event_drops: u64,\n    event_absorbs: u64,\n    event_borrows: u64,\n    event_publishes_with_ceased_identity: u64,\n    event_range_tombstones: u64,\n}\n")
source = replace_once(source, "            slots_reclaimed_by_publish: BTreeMap::new(),\n        };\n", "            slots_reclaimed_by_publish: BTreeMap::new(),\n            event_drops: 0,\n            event_absorbs: 0,\n            event_borrows: 0,\n            event_publishes_with_ceased_identity: 0,\n            event_range_tombstones: 0,\n        };\n")
source = replace_once(source, "        RANGE_TOMBSTONES.with(|ranges| ranges.borrow_mut().extend(plan.range_tombstones.iter().copied()));\n", "        RANGE_TOMBSTONES.with(|ranges| ranges.borrow_mut().extend(plan.range_tombstones.iter().copied()));\n        self.event_drops += plan.drops;\n        self.event_absorbs += plan.absorbs;\n        self.event_borrows += plan.borrows;\n        self.event_range_tombstones += length_as_u64(plan.range_tombstones.len());\n        if plan.drops + plan.absorbs > 0 {\n            self.event_publishes_with_ceased_identity += 1;\n        }\n")
# 输出行
source = replace_once(
    source,
    "    )];\n    let mut cells = Vec::new();\n    for arm in simulation.write_path.arms() {",
    "    )];\n    if candidate() != Candidate::Baseline || merge_policy() != MergePolicy::None {\n        lines.push(format!(\n            \"name=candidate_cost {label} geometry={} candidate={:?} merge_policy={:?} drops={} absorbs={} borrows={} publishes_with_ceased_identity={} range_tombstones={}\",\n            geometry.name(),\n            candidate(),\n            merge_policy(),\n            simulation.event_drops,\n            simulation.event_absorbs,\n            simulation.event_borrows,\n            simulation.event_publishes_with_ceased_identity,\n            simulation.event_range_tombstones,\n        ));\n    }\n    let mut cells = Vec::new();\n    for arm in simulation.write_path.arms() {",
)
open(sys.argv[2], "w", encoding="utf-8").write(source)
print("ok")
