#!/usr/bin/env python3
"""把 E177 装置拷贝成「候选量」副本：加环境变量 E177_CANDIDATE（retire / zero / all_suppressed）与 E177_MERGE（absorb / absorb_borrow）。
用法：apply_patch.py <原装置.rs> <输出.rs>。每处替换的旧串必须在原文里恰好出现一次。"""
import sys

source = open(sys.argv[1], encoding="utf-8").read()


def replace_once(text, old, new):
    count = text.count(old)
    if count != 1:
        sys.exit(f"旧串出现 {count} 次，不是 1 次：{old[:80]!r}")
    return text.replace(old, new)


# E1 imports
source = replace_once(source, "use std::sync::Mutex;\n", "use std::sync::Mutex;\nuse std::sync::OnceLock;\nuse std::cell::RefCell;\nuse std::collections::HashSet;\n")

# E2 candidate / merge policy
source = replace_once(
    source,
    "const PLANTED_UNIT: usize = usize::MAX;\n",
    '''const PLANTED_UNIT: usize = usize::MAX;

/// 副本装置（m3-leaf-death-r1 正推腿）加的候选：环境变量 E177_CANDIDATE。
/// retire = 删空 / 被吸收的容器身份写一条退役记录（读侧丢掉整组）；zero = 删空的容器照样落一个零条目版本（写序最新、不被树引用、同发布内释放）；
/// all_suppressed = 只改读侧：容器里每条记录都被墓碑压掉就不当活单元收（只对臂 B 有意义）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Candidate {
    Baseline,
    RetireRecord,
    ZeroVersion,
    AllSuppressedIsDead,
}

fn candidate() -> Candidate {
    static CHOSEN: OnceLock<Candidate> = OnceLock::new();
    *CHOSEN.get_or_init(|| match std::env::var("E177_CANDIDATE").as_deref() {
        Ok("retire") => Candidate::RetireRecord,
        Ok("zero") => Candidate::ZeroVersion,
        Ok("all_suppressed") => Candidate::AllSuppressedIsDead,
        _ => Candidate::Baseline,
    })
}

/// 合并政策：环境变量 E177_MERGE。absorb = E176 的 M2（相邻两片合起来不超过 233 条就左吸收右）；
/// absorb_borrow = E176 的 M3（再加：删完那片不满 117 条且有右邻，从右邻借最小的 ⌊(右 − 本片) ÷ 2⌋ 条，均分借）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MergePolicy {
    None,
    Absorb,
    AbsorbAndBorrow,
}

fn merge_policy() -> MergePolicy {
    static CHOSEN: OnceLock<MergePolicy> = OnceLock::new();
    *CHOSEN.get_or_init(|| match std::env::var("E177_MERGE").as_deref() {
        Ok("absorb") => MergePolicy::Absorb,
        Ok("absorb_borrow") => MergePolicy::AbsorbAndBorrow,
        _ => MergePolicy::None,
    })
}

const BORROW_FLOOR: u64 = 117;

thread_local! {
    /// 这条线程上正在跑的那一个 Simulation 已经写下的容器退役记录：(容器号, 容器出生代)，出生树恒为 inode 树、类型恒为 2。
    static RETIRED_IDENTITIES: RefCell<HashSet<(u64, u64)>> = RefCell::new(HashSet::new());
}
''',
)

# E3 LogicalPlan fields
source = replace_once(
    source,
    "    accounting_keys: BTreeSet<KeyFields>,\n    released_units: Vec<UnitId>,\n}\n",
    "    accounting_keys: BTreeSet<KeyFields>,\n    released_units: Vec<UnitId>,\n    /// 候选 retire：这次发布里不再存在的 inode 叶容器身份。\n    retired_containers: Vec<(u64, u64)>,\n    /// 候选 zero：这次发布里要补写零条目版本的容器身份。\n    zero_versions: Vec<(u64, u64)>,\n}\n",
)
source = replace_once(
    source,
    "        accounting_keys: accounting_keys_of(txg),\n        released_units: Vec::new(),\n    };\n",
    "        accounting_keys: accounting_keys_of(txg),\n        released_units: Vec::new(),\n        retired_containers: Vec::new(),\n        zero_versions: Vec::new(),\n    };\n",
)

# E4 plan_inode_record_removal
old_start = source.index("/// 删一条 inode 记录（S5，`inode_tree.rs:286-317`）")
old_end = source.index("/// 写文件内容（覆盖写、顺序写、新池新建文件）")
new_function = '''/// 删一条 inode 记录（S5，`inode_tree.rs:286-317`）：删空的那片不落盘、右边各片左移、全部重写。
/// 副本加：E177_MERGE 非空时照 E176 的 M2 / M3 再做左吸收右与（M3）均分借；E177_CANDIDATE 决定一片容器身份消失时额外写什么。
fn plan_inode_record_removal(plan: &mut LogicalPlan, inode: u64) {
    let position = plan
        .inode_containers
        .iter()
        .position(|container| container.records.iter().any(|existing| existing.inode == inode))
        .expect("删的记录在树里");
    plan.inode_containers[position]
        .records
        .retain(|existing| existing.inode != inode);
    let mut changed_container_numbers: BTreeSet<u64> = BTreeSet::new();
    let mut first_removed_position: Option<usize> = None;
    let note_removed = |plan: &mut LogicalPlan, at: usize, first_removed_position: &mut Option<usize>| {
        let removed = plan.inode_containers.remove(at);
        release_if_carried(removed.origin, &mut plan.released_units);
        match candidate() {
            Candidate::RetireRecord => plan.retired_containers.push((removed.container_number, removed.container_birth)),
            Candidate::ZeroVersion => plan.zero_versions.push((removed.container_number, removed.container_birth)),
            Candidate::Baseline | Candidate::AllSuppressedIsDead => {}
        }
        *first_removed_position = Some(first_removed_position.map_or(at, |existing| existing.min(at)));
    };
    let mut current = position;
    if plan.inode_containers[current].records.is_empty() {
        note_removed(plan, current, &mut first_removed_position);
        // 删空那片之后，它的左邻（若有）就是后面检查合并的起点。
        current = current.saturating_sub(1);
        if plan.inode_containers.is_empty() {
            current = 0;
        }
    } else {
        changed_container_numbers.insert(plan.inode_containers[current].container_number);
    }
    if merge_policy() != MergePolicy::None && !plan.inode_containers.is_empty() {
        // M2：围绕 current 反复查相邻两对，和不超过 233 就左吸收右（右退役）。
        loop {
            let mut merged = false;
            let low = current.saturating_sub(1);
            let high = (current + 1).min(plan.inode_containers.len().saturating_sub(1));
            let mut at = low;
            while at < high.min(plan.inode_containers.len().saturating_sub(1)) {
                if at + 1 < plan.inode_containers.len() {
                    let sum = length_as_u64(plan.inode_containers[at].records.len() + plan.inode_containers[at + 1].records.len());
                    if sum <= INODE_LEAF_RECORDS {
                        let right_records = plan.inode_containers[at + 1].records.clone();
                        plan.inode_containers[at].records.extend(right_records);
                        changed_container_numbers.insert(plan.inode_containers[at].container_number);
                        note_removed(plan, at + 1, &mut first_removed_position);
                        current = at;
                        merged = true;
                        break;
                    }
                }
                at += 1;
            }
            if !merged {
                break;
            }
        }
        if merge_policy() == MergePolicy::AbsorbAndBorrow {
            // M3：这次一次都没并（first_removed_position 为空）、本片没删空、记录数不足下限且有右邻 ⇒ 均分借。
            if first_removed_position.is_none() && current + 1 < plan.inode_containers.len() {
                let own = length_as_u64(plan.inode_containers[current].records.len());
                let neighbour = length_as_u64(plan.inode_containers[current + 1].records.len());
                if own < BORROW_FLOOR && neighbour > own {
                    let borrowed = u64_as_index((neighbour - own) / 2);
                    if borrowed > 0 {
                        let moved: Vec<InodeRecordSummary> = plan.inode_containers[current + 1].records.drain(..borrowed).collect();
                        plan.inode_containers[current].records.extend(moved);
                        changed_container_numbers.insert(plan.inode_containers[current].container_number);
                        changed_container_numbers.insert(plan.inode_containers[current + 1].container_number);
                        // 借完让右邻与它右邻那一对照 M2 的规则查。
                        if current + 2 < plan.inode_containers.len() {
                            let sum = length_as_u64(plan.inode_containers[current + 1].records.len() + plan.inode_containers[current + 2].records.len());
                            if sum <= INODE_LEAF_RECORDS {
                                let right_records = plan.inode_containers[current + 2].records.clone();
                                plan.inode_containers[current + 1].records.extend(right_records);
                                note_removed(plan, current + 2, &mut first_removed_position);
                            }
                        }
                    }
                }
            }
        }
    }
    for index in 0..plan.inode_containers.len() {
        let number = plan.inode_containers[index].container_number;
        let shifted = first_removed_position.is_some_and(|removed_at| index >= removed_at);
        if shifted || changed_container_numbers.contains(&number) {
            mark_inode_container_rewritten(plan, index);
        }
    }
    mark_inode_root_rewritten(plan);
}

'''
source = source[:old_start] + new_function + source[old_end:]

# E5 Role::ZeroInodeContainer
source = replace_once(source, "    InodeContainer(usize),\n    InodeRoot,\n    AllocationNode(AllocationNode),", "    InodeContainer(usize),\n    ZeroInodeContainer(usize),\n    InodeRoot,\n    AllocationNode(AllocationNode),")
source = replace_once(
    source,
    "            Role::InstanceTable | Role::InodeContainer(_) | Role::TombstoneContainer(_) => {",
    "            Role::InstanceTable | Role::InodeContainer(_) | Role::ZeroInodeContainer(_) | Role::TombstoneContainer(_) => {",
)
source = replace_once(
    source,
    "    for (position, container) in plan.inode_containers.iter().enumerate() {\n        if container.origin == PlannedUnit::Rewritten {\n            roles.push(Role::InodeContainer(position));\n        }\n    }\n",
    "    for (position, container) in plan.inode_containers.iter().enumerate() {\n        if container.origin == PlannedUnit::Rewritten {\n            roles.push(Role::InodeContainer(position));\n        }\n    }\n    for index in 0..plan.zero_versions.len() {\n        roles.push(Role::ZeroInodeContainer(index));\n    }\n",
)

# E6 build_units
source = replace_once(
    source,
    "                Role::InodeRoot => (UnitOwner::InodeRoot, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),\n                Role::AllocationNode(node) => {",
    '''                Role::ZeroInodeContainer(index) => {
                    let (container_number, container_birth) = plan.zero_versions[*index];
                    group = Some(SelectionGroup::PackedContainer {
                        birth_tree: TREE_IDENTIFIER_INODE,
                        record_type: PACKED_TYPE_INODE,
                        container_number,
                        container_birth,
                    });
                    (UnitOwner::InodeContainer, UNIT_CLASS_PACKED, UnitContents::InodeRecords(Vec::new()))
                }
                Role::InodeRoot => (UnitOwner::InodeRoot, UNIT_CLASS_INDEX_NODE, UnitContents::WithoutKeys),
                Role::AllocationNode(node) => {''',
)

# E7 publish: release zero units right away; register retired identities
source = replace_once(
    source,
    "        let reclaimed_placements = self.allocator.record_root(txg);\n",
    '''        let mut releases_for_the_scan = assembled.releases.clone();
        for (offset, role) in assembled.roles.iter().enumerate() {
            if let Role::ZeroInodeContainer(_) = role {
                let unit_id = first_new_unit + offset;
                let (slot, span_slots) = (self.disk.units[unit_id].slot, self.disk.units[unit_id].span_slots);
                self.allocator.release(slot, span_slots, txg);
                self.disk.units[unit_id].release_txg = Some(txg);
                releases_for_the_scan.push(unit_id);
            }
        }
        let reclaimed_placements = self.allocator.record_root(txg);
''',
)
source = replace_once(
    source,
    "        self.version = self.build_version(&plan, &assembled, &unit_of_role);\n        self.note_unit_lists(first_new_unit);\n",
    "        self.version = self.build_version(&plan, &assembled, &unit_of_role);\n        RETIRED_IDENTITIES.with(|retired| retired.borrow_mut().extend(plan.retired_containers.iter().copied()));\n        self.note_unit_lists(first_new_unit);\n",
)
source = replace_once(
    source,
    "        self.scan(txg, &assembled.releases, &written, &made_unreadable, &plan.tombstones_written);\n",
    "        self.scan(txg, &releases_for_the_scan, &written, &made_unreadable, &plan.tombstones_written);\n",
)
# start(): clear the thread-local
source = replace_once(
    source,
    "        let unit_area_slots = match geometry {\n            Geometry::Ring => Some(RING_GEOMETRY_UNIT_AREA_SLOTS),",
    "        RETIRED_IDENTITIES.with(|retired| retired.borrow_mut().clear());\n        let unit_area_slots = match geometry {\n            Geometry::Ring => Some(RING_GEOMETRY_UNIT_AREA_SLOTS),",
)

# E8 select_current_reading
source = replace_once(
    source,
    "    let mut reading = ReadingResult::empty();\n    let mut tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>> = HashMap::new();\n",
    '''    if candidate() == Candidate::RetireRecord {
        RETIRED_IDENTITIES.with(|retired| {
            let retired = retired.borrow();
            newest.retain(|group, _| match group {
                SelectionGroup::PackedContainer {
                    birth_tree,
                    record_type,
                    container_number,
                    container_birth,
                } if *birth_tree == TREE_IDENTIFIER_INODE && *record_type == PACKED_TYPE_INODE => {
                    !retired.contains(&(*container_number, *container_birth))
                }
                _ => true,
            });
        });
    }
    let mut reading = ReadingResult::empty();
    let mut tombstones_by_object: HashMap<u64, Vec<TombstoneRecordSummary>> = HashMap::new();
''',
)
source = replace_once(
    source,
    "    let mut inode_tree_objects: BTreeSet<(u64, u64)> = BTreeSet::new();\n    for (_, unit_id) in newest.values() {",
    '''    if candidate() == Candidate::AllSuppressedIsDead && arm == Arm::TodaysTombstones {
        accepted_containers.retain(|unit_id| {
            let unit = unit_by_id(disk, inputs.planted, *unit_id);
            match &unit.contents {
                UnitContents::InodeRecords(records) => {
                    records.is_empty()
                        || !records.iter().all(|record| {
                            suppressed(
                                record.inode,
                                OccurrenceTag::InodeRecord {
                                    inode: record.inode,
                                    object_birth: record.object_birth,
                                },
                            )
                        })
                }
                _ => true,
            }
        });
    }
    let mut inode_tree_objects: BTreeSet<(u64, u64)> = BTreeSet::new();
    for (_, unit_id) in newest.values() {''',
)

# E9 config marker
source = replace_once(
    source,
    "            let anchor = run_ring_anchor();\n            emitted_lines.extend(anchor.lines.iter().cloned());\n            let results = run_segments_in_parallel(&histories, threads);",
    "            emitted_lines.push(format!(\"name=candidate_run candidate={:?} merge_policy={:?}\", candidate(), merge_policy()));\n            let anchor = run_ring_anchor();\n            emitted_lines.extend(anchor.lines.iter().cloned());\n            let results = run_segments_in_parallel(&histories, threads);",
)
open(sys.argv[2], "w", encoding="utf-8").write(source)
print("ok")
