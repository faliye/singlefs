#!/usr/bin/env python3
"""第二层补丁（接在 apply_patch.py 之后）：候选 shrink_range——文件变短那次发布写一条区间墓碑 (对象, 出生代, 起始单元序号, 死亡 txg)，
读法 R-S 对臂 B 丢掉「起始单元序号以后、诞生 txg 不晚于死亡 txg」的数据单元组。区间墓碑只在模型里（不占槽，像臂 P 的死亡记录），代价另算。
用法：apply_patch2.py <第一层输出.rs> <输出.rs>"""
import sys
source = open(sys.argv[1], encoding="utf-8").read()

def replace_once(text, old, new):
    count = text.count(old)
    if count != 1:
        sys.exit(f"旧串出现 {count} 次，不是 1 次：{old[:80]!r}")
    return text.replace(old, new)

source = replace_once(source, "    AllSuppressedIsDead,\n}\n", "    AllSuppressedIsDead,\n    ShrinkRange,\n}\n")
source = replace_once(source, '        Ok("all_suppressed") => Candidate::AllSuppressedIsDead,\n', '        Ok("all_suppressed") => Candidate::AllSuppressedIsDead,\n        Ok("shrink_range") => Candidate::ShrinkRange,\n')
source = replace_once(source, "            Candidate::Baseline | Candidate::AllSuppressedIsDead => {}\n", "            Candidate::Baseline | Candidate::AllSuppressedIsDead | Candidate::ShrinkRange => {}\n")
source = replace_once(
    source,
    "    static RETIRED_IDENTITIES: RefCell<HashSet<(u64, u64)>> = RefCell::new(HashSet::new());\n}\n",
    "    static RETIRED_IDENTITIES: RefCell<HashSet<(u64, u64)>> = RefCell::new(HashSet::new());\n    /// 候选 shrink_range：文件变短那次发布写下的区间墓碑 (对象, 对象出生代, 起始单元序号, 死亡 txg)；只在模型里，不占槽。\n    static RANGE_TOMBSTONES: RefCell<Vec<(u64, u64, u64, u64)>> = RefCell::new(Vec::new());\n}\n",
)
source = replace_once(source, "    zero_versions: Vec<(u64, u64)>,\n}\n", "    zero_versions: Vec<(u64, u64)>,\n    range_tombstones: Vec<(u64, u64, u64, u64)>,\n}\n")
source = replace_once(source, "        zero_versions: Vec::new(),\n    };\n", "        zero_versions: Vec::new(),\n        range_tombstones: Vec::new(),\n    };\n")
# 记录：plan_file_rewrite 里变短才记
source = replace_once(
    source,
    "fn plan_file_rewrite(plan: &mut LogicalPlan, previous: &Version, data_units: u64) {\n",
    "fn plan_file_rewrite(plan: &mut LogicalPlan, previous: &Version, data_units: u64) {\n    if candidate() == Candidate::ShrinkRange && length_as_u64(previous.file_units.len()) > data_units {\n        plan.range_tombstones.push((FIRST_FILE_INODE, plan.file_object_birth, data_units, plan.txg));\n    }\n",
)
source = replace_once(
    source,
    "        RETIRED_IDENTITIES.with(|retired| retired.borrow_mut().extend(plan.retired_containers.iter().copied()));\n",
    "        RETIRED_IDENTITIES.with(|retired| retired.borrow_mut().extend(plan.retired_containers.iter().copied()));\n        RANGE_TOMBSTONES.with(|ranges| ranges.borrow_mut().extend(plan.range_tombstones.iter().copied()));\n",
)
source = replace_once(
    source,
    "        RETIRED_IDENTITIES.with(|retired| retired.borrow_mut().clear());\n",
    "        RETIRED_IDENTITIES.with(|retired| retired.borrow_mut().clear());\n        RANGE_TOMBSTONES.with(|ranges| ranges.borrow_mut().clear());\n",
)
source = replace_once(
    source,
    "            if reader == Reader::SelectCurrentGatedByObject && !inode_tree_objects.contains(&(*object, *object_birth)) {\n                continue;\n            }\n            accepted_data_units.push(*unit_id);\n",
    "            if reader == Reader::SelectCurrentGatedByObject && !inode_tree_objects.contains(&(*object, *object_birth)) {\n                continue;\n            }\n            if candidate() == Candidate::ShrinkRange && arm == Arm::TodaysTombstones {\n                let birth_of_the_version = unit_by_id(disk, inputs.planted, *unit_id).birth_txg();\n                let anchor = match group {\n                    SelectionGroup::DataUnit { anchor_unit, .. } => *anchor_unit,\n                    SelectionGroup::PackedContainer { .. } => 0,\n                };\n                let cut_by_a_range = RANGE_TOMBSTONES.with(|ranges| {\n                    ranges.borrow().iter().any(|(range_object, range_birth, start_unit, death_txg)| {\n                        range_object == object && range_birth == object_birth && anchor >= *start_unit && birth_of_the_version <= *death_txg\n                    })\n                });\n                if cut_by_a_range {\n                    continue;\n                }\n            }\n            accepted_data_units.push(*unit_id);\n",
)
open(sys.argv[2], "w", encoding="utf-8").write(source)
print("ok")
