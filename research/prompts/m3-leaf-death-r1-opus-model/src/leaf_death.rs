// =============================================================================================
// m3-leaf-death-r1 攻方腿（Opus）加的部分：K1 四个候选、K2 的 R-Z 读法、K3 的合并与借（M2、M3 均分、M3 刚好够）。
// 写路径沿用 E177 装置（照 `crates/` 今天的规则手写的那一份）；这里只加：
//   - 合并策略（照 E176 装置 `e176_inode_leaf_merge_capacity_and_rewrites.rs` 第 213–319 行的 M2 / M3 规则重写一份，
//     重写清单按 `crates/` 的位置记法：内容变了的叶 + 第一个被摘的位置起右边每一片）；
//   - 候选二的两种写法（零条目版本留在树里 / 只落一次、下一次发布摘掉）；
//   - 候选一（退役记录）、候选三（记录死亡写死亡记录）放在模型专用区、不占槽（与 E177 臂 P 同一种理想化：只量「读侧有了它够不够」）；
//   - 读法：S（E177 的 R-S）、C1、C2、C3，与 K2 的 Z（数据单元按重建出的 inode 记录的出生代与 size 判死活）。
// =============================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MergeKind {
    M1,
    M2,
    M3Half,
    M3Just,
    /// K3「改借法让旧判据成立」：均分借完把右邻改号成它借完之后的最小 key，出生代不变；旧身份写退役记录。
    M3HalfRenameKeepBirth,
    /// 同上，改号的同时出生代换成这次发布的 txg。
    M3HalfRenameNewBirth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ZeroKind {
    NotWritten,
    Persist,
    OneShot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LeafPolicy {
    merge: MergeKind,
    zero: ZeroKind,
}

impl LeafPolicy {
    const TODAY: Self = Self { merge: MergeKind::M1, zero: ZeroKind::NotWritten };

    fn name(self) -> String {
        let merge = match self.merge {
            MergeKind::M1 => "M1",
            MergeKind::M2 => "M2",
            MergeKind::M3Half => "M3half",
            MergeKind::M3Just => "M3just",
            MergeKind::M3HalfRenameKeepBirth => "M3half_rename_keep_birth",
            MergeKind::M3HalfRenameNewBirth => "M3half_rename_new_birth",
        };
        let zero = match self.zero {
            ZeroKind::NotWritten => "dropped",
            ZeroKind::Persist => "zero_persist",
            ZeroKind::OneShot => "zero_oneshot",
        };
        format!("{merge}_{zero}")
    }
}

/// M3 的下限（E176 主取样点 f = 117）。
const M3_FLOOR: usize = 117;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LeafReader {
    /// E177 的 R-S：每组取最新读得出的一版。
    S,
    /// 候选一：退役记录点名的 (容器号, 容器出生代) 整组不收。
    C1,
    /// 候选二：一组最新读得出的那一版记录数为 0 ⇒ 整组死。
    C2,
    /// 候选三：记录死亡写死亡记录 (inode, 出生代, 死亡 txg)；死亡 txg ≥ 供出版本的诞生代号就不收；一片里一条都不收的容器整片不收。
    C3,
    /// 候选三借今天意图那次写的整对象墓碑当死亡记录（不另写）：死亡 txg = 意图 txg；供出版本的诞生代号 < 它才不收。
    C3IntentLt,
    /// 同上，诞生代号 ≤ 它就不收（意图那次就生效，E177 臂 B 的效力）。
    C3IntentLe,
    /// K2：C1 加数据单元按重建出的 inode 记录判（对象出生代相等且锚点 < size 折成的单元数）。
    C1Z,
    /// K2：C3 加同一条数据单元门。
    C3Z,
}

impl LeafReader {
    const fn name(self) -> &'static str {
        match self {
            LeafReader::S => "S",
            LeafReader::C1 => "C1",
            LeafReader::C2 => "C2",
            LeafReader::C3 => "C3",
            LeafReader::C3IntentLt => "C3_intent_lt",
            LeafReader::C3IntentLe => "C3_intent_le",
            LeafReader::C1Z => "C1Z",
            LeafReader::C3Z => "C3Z",
        }
    }
}

const LEAF_NAMESPACES: [(Namespace, &str); 5] = [
    (Namespace::Inode, "inode"),
    (Namespace::DeletedInodes, "deleted_inodes"),
    (Namespace::Extent, "extent"),
    (Namespace::AllocationUnitsOfTheVersion, "allocation_records"),
    (Namespace::CentralMappingUnitsOfTheVersion, "central_mapping"),
];

#[derive(Clone, Debug, Default)]
struct LeafCounts {
    retirements: u64,
    publishes_with_a_retirement: u64,
    merges: u64,
    borrows: u64,
    zero_versions: u64,
    oneshot_drops: u64,
    record_deaths: u64,
    removal_publishes: u64,
    containers_rewritten_in_removals: u64,
    most_containers_rewritten_in_a_removal: u64,
    third_sentence_false_pairs_peak: u64,
    versions_with_a_third_sentence_false_pair: u64,
    misrouted_records_peak: u64,
    versions_with_an_empty_container: u64,
    empty_containers_peak: u64,
    leaf_s_differs_from_e177_r_s: u64,
}

fn position_of_container_number(plan: &LogicalPlan, number: u64) -> usize {
    plan.inode_containers
        .iter()
        .position(|container| container.container_number == number)
        .expect("这次发布里点名的容器号还在树里")
}

fn drop_container_at(plan: &mut LogicalPlan, position: usize, first_dropped: &mut Option<usize>, changed: &mut BTreeSet<u64>) {
    let removed = plan.inode_containers.remove(position);
    release_if_carried(removed.origin, &mut plan.released_units);
    plan.retired_identities.push((removed.container_number, removed.container_birth));
    changed.remove(&removed.container_number);
    *first_dropped = Some(first_dropped.map_or(position, |first| first.min(position)));
}

fn pair_fits_in_one_container(plan: &LogicalPlan, left: usize) -> bool {
    plan.inode_containers[left].records.len() + plan.inode_containers[left + 1].records.len()
        <= u64_as_index(INODE_LEAF_RECORDS)
}

fn absorb_right_neighbour(plan: &mut LogicalPlan, left: usize, first_dropped: &mut Option<usize>, changed: &mut BTreeSet<u64>) {
    let absorbed_records = plan.inode_containers[left + 1].records.clone();
    drop_container_at(plan, left + 1, first_dropped, changed);
    plan.inode_containers[left].records.extend(absorbed_records);
    changed.insert(plan.inode_containers[left].container_number);
    plan.merges += 1;
}

/// E176 `merge_until_no_pair_fits` 的先查右边那一种。
fn merge_until_no_pair_fits(plan: &mut LogicalPlan, start: u64, first_dropped: &mut Option<usize>, changed: &mut BTreeSet<u64>) -> bool {
    let mut current = start;
    let mut merged = false;
    loop {
        let position = position_of_container_number(plan, current);
        let right_fits = position + 1 < plan.inode_containers.len() && pair_fits_in_one_container(plan, position);
        let left_fits = position > 0 && pair_fits_in_one_container(plan, position - 1);
        if right_fits {
            absorb_right_neighbour(plan, position, first_dropped, changed);
        } else if left_fits {
            absorb_right_neighbour(plan, position - 1, first_dropped, changed);
            current = plan.inode_containers[position - 1].container_number;
        } else {
            return merged;
        }
        merged = true;
    }
}

/// E176 `borrow_from_right_if_below_floor`：借右邻最小的几条，两片都保留身份；借完右邻与它右邻那一对照 M2 查。
fn borrow_from_right_if_below_floor(plan: &mut LogicalPlan, borrower: u64, merge: MergeKind, first_dropped: &mut Option<usize>, changed: &mut BTreeSet<u64>) {
    let position = position_of_container_number(plan, borrower);
    if position + 1 >= plan.inode_containers.len() {
        return;
    }
    let in_borrower = plan.inode_containers[position].records.len();
    if in_borrower >= M3_FLOOR {
        return;
    }
    let in_lender = plan.inode_containers[position + 1].records.len();
    let moved = match merge {
        MergeKind::M3Half | MergeKind::M3HalfRenameKeepBirth | MergeKind::M3HalfRenameNewBirth => (in_lender - in_borrower) / 2,
        MergeKind::M3Just => M3_FLOOR - in_borrower,
        MergeKind::M1 | MergeKind::M2 => unreachable!("只有 M3 借"),
    };
    assert!(moved >= 1 && moved < in_lender, "借的条数在 [1, 右邻记录数) 里");
    let moved_records: Vec<InodeRecordSummary> = plan.inode_containers[position + 1].records.drain(..moved).collect();
    plan.inode_containers[position].records.extend(moved_records);
    let mut lender = plan.inode_containers[position + 1].container_number;
    if matches!(merge, MergeKind::M3HalfRenameKeepBirth | MergeKind::M3HalfRenameNewBirth) {
        let old_birth = plan.inode_containers[position + 1].container_birth;
        plan.retired_identities.push((lender, old_birth));
        changed.remove(&lender);
        let new_number = plan.inode_containers[position + 1].records[0].inode;
        plan.inode_containers[position + 1].container_number = new_number;
        if merge == MergeKind::M3HalfRenameNewBirth {
            plan.inode_containers[position + 1].container_birth = plan.txg;
        }
        lender = new_number;
    }
    changed.insert(borrower);
    changed.insert(lender);
    plan.borrows += 1;
    merge_until_no_pair_fits(plan, lender, first_dropped, changed);
}

/// 删一条 inode 记录（unlink 最后一批那次发布），按合并策略与候选二的写法。M1 + 不落盘 与 E177 原来那一份逐格同义
/// （删空就摘、右边各片左移全部重写；没删空只重写它那一片）。
fn plan_inode_record_removal(plan: &mut LogicalPlan, inode: u64, policy: LeafPolicy) {
    let position = plan
        .inode_containers
        .iter()
        .position(|container| container.records.iter().any(|existing| existing.inode == inode))
        .expect("删的记录在树里");
    plan.inode_containers[position].records.retain(|existing| existing.inode != inode);
    let holding = plan.inode_containers[position].container_number;
    let mut changed: BTreeSet<u64> = BTreeSet::new();
    let mut first_dropped: Option<usize> = None;
    if plan.inode_containers[position].records.is_empty() {
        match policy.zero {
            ZeroKind::Persist | ZeroKind::OneShot => {
                assert_eq!(policy.merge, MergeKind::M1, "零条目版本的两种写法只配 M1");
                changed.insert(holding);
                plan.zero_versions += 1;
            }
            ZeroKind::NotWritten => {
                drop_container_at(plan, position, &mut first_dropped, &mut changed);
                if policy.merge != MergeKind::M1 && position > 0 && position < plan.inode_containers.len() {
                    let left = plan.inode_containers[position - 1].container_number;
                    merge_until_no_pair_fits(plan, left, &mut first_dropped, &mut changed);
                }
            }
        }
    } else {
        changed.insert(holding);
        match policy.merge {
            MergeKind::M1 => {}
            MergeKind::M2 => {
                merge_until_no_pair_fits(plan, holding, &mut first_dropped, &mut changed);
            }
            MergeKind::M3Half | MergeKind::M3Just | MergeKind::M3HalfRenameKeepBirth | MergeKind::M3HalfRenameNewBirth => {
                let merged = merge_until_no_pair_fits(plan, holding, &mut first_dropped, &mut changed);
                if !merged {
                    borrow_from_right_if_below_floor(plan, holding, policy.merge, &mut first_dropped, &mut changed);
                }
            }
        }
    }
    let mut positions: BTreeSet<usize> = changed.iter().map(|number| position_of_container_number(plan, *number)).collect();
    if let Some(first) = first_dropped {
        positions.extend(first..plan.inode_containers.len());
    }
    for position in positions {
        mark_inode_container_rewritten(plan, position);
    }
    mark_inode_root_rewritten(plan);
}

/// 候选二「只落一次」：上一次发布留下的零条目容器这次摘掉（不写），右边各片左移、全部重写。
fn drop_empty_containers_left_from_the_last_publish(plan: &mut LogicalPlan) {
    let mut first: Option<usize> = None;
    let mut position = 0;
    while position < plan.inode_containers.len() {
        if plan.inode_containers[position].records.is_empty() {
            let removed = plan.inode_containers.remove(position);
            release_if_carried(removed.origin, &mut plan.released_units);
            plan.oneshot_drops += 1;
            first = Some(first.map_or(position, |earlier| earlier.min(position)));
        } else {
            position += 1;
        }
    }
    if let Some(first) = first {
        for later in first..plan.inode_containers.len() {
            mark_inode_container_rewritten(plan, later);
        }
        mark_inode_root_rewritten(plan);
    }
}

/// 一种读法读出的东西（五个名字空间）。
fn leaf_reading(
    reader: LeafReader,
    inputs: &ScanInputs<'_>,
    retired: &BTreeMap<(u64, u64), u64>,
    record_deaths: &BTreeMap<(u64, u64), u64>,
    intent_tombstones: &BTreeMap<(u64, u64), u64>,
) -> ReadingResult {
    let disk = inputs.disk;
    let lists = inputs.unit_lists;
    let mut newest: BTreeMap<SelectionGroup, (WriteOrder, UnitId)> = BTreeMap::new();
    for unit_id in lists.data_units.iter().chain(&lists.inode_containers) {
        if inputs.is_candidate(*unit_id) {
            consider_for_selection(&mut newest, *unit_id, &disk.units[*unit_id]);
        }
    }
    let uses_retirement = matches!(reader, LeafReader::C1 | LeafReader::C1Z);
    let uses_record_deaths = matches!(reader, LeafReader::C3 | LeafReader::C3Z);
    let gates_data_units = matches!(reader, LeafReader::C1Z | LeafReader::C3Z);
    let mut reading = ReadingResult::empty();
    let mut accepted_containers: Vec<UnitId> = Vec::new();
    // inode → (对象出生代, 单元数, 供出版本的诞生代号)：同一个号出现在几片里（合并、借之后旧版还读得出）时取诞生代号最大的那一版。
    let mut records_by_inode: BTreeMap<u64, (u64, u64, u64)> = BTreeMap::new();
    for (group, (_, unit_id)) in &newest {
        let SelectionGroup::PackedContainer { container_number, container_birth, .. } = group else {
            continue;
        };
        if uses_retirement && retired.contains_key(&(*container_number, *container_birth)) {
            continue;
        }
        let unit = &disk.units[*unit_id];
        let UnitContents::InodeRecords(records) = &unit.contents else {
            continue;
        };
        if reader == LeafReader::C2 && records.is_empty() {
            continue;
        }
        let mut kept_any = false;
        for record in records {
            if uses_record_deaths
                && record_deaths
                    .get(&(record.inode, record.object_birth))
                    .is_some_and(|death| *death >= unit.birth_txg())
            {
                continue;
            }
            let intent = intent_tombstones.get(&(record.inode, record.object_birth));
            if reader == LeafReader::C3IntentLt && intent.is_some_and(|death| unit.birth_txg() < *death) {
                continue;
            }
            if reader == LeafReader::C3IntentLe && intent.is_some_and(|death| unit.birth_txg() <= *death) {
                continue;
            }
            kept_any = true;
            let fields = single_field_key(record.inode);
            reading.note(Namespace::Inode, fields, unit);
            if record.nlink == 0 {
                reading.note(Namespace::DeletedInodes, fields, unit);
            }
            let entry = records_by_inode
                .entry(record.inode)
                .or_insert((record.object_birth, record.units, unit.birth_txg()));
            if unit.birth_txg() > entry.2 {
                *entry = (record.object_birth, record.units, unit.birth_txg());
            }
        }
        if (uses_record_deaths || matches!(reader, LeafReader::C3IntentLt | LeafReader::C3IntentLe)) && !kept_any && !records.is_empty() {
            continue;
        }
        accepted_containers.push(*unit_id);
    }
    let mut accepted_data_units: Vec<UnitId> = Vec::new();
    for (group, (_, unit_id)) in &newest {
        let SelectionGroup::DataUnit { object, object_birth, anchor_unit, .. } = group else {
            continue;
        };
        if gates_data_units {
            match records_by_inode.get(object) {
                Some((birth, units, _)) if birth == object_birth && anchor_unit < units => {}
                _ => continue,
            }
        }
        accepted_data_units.push(*unit_id);
    }
    for unit_id in &accepted_data_units {
        let unit = &disk.units[*unit_id];
        if let UnitContents::DataUnit { object, anchor_unit, .. } = unit.contents {
            reading.note(Namespace::Extent, extent_key(object, anchor_unit), unit);
        }
    }
    for unit_id in accepted_data_units.iter().chain(&accepted_containers) {
        let unit = &disk.units[*unit_id];
        reading.note(Namespace::AllocationUnitsOfTheVersion, single_field_key(unit.slot), unit);
        if let Some(mapping_fields) = unit.mapping_key {
            reading.note(Namespace::CentralMappingUnitsOfTheVersion, mapping_fields, unit);
        }
    }
    reading
}

/// 叶序上「左容器最大 key ≥ 右容器号」的对数（I-9.4 第二句后半为假、I-9.12 按分隔 key = 容器号判也为假的那几对），
/// 与按 D8 已定项 6 的查找规则（最后一个容器号 ≤ 目标 key 的那片）走不到自己那片的记录数。
fn leaf_order_structure(version: &Version) -> (u64, u64, u64) {
    let containers = &version.inode_containers;
    let mut third_false = 0;
    for pair in containers.windows(2) {
        if let Some(largest) = pair[0].records.last() {
            if largest.inode >= pair[1].container_number {
                third_false += 1;
            }
        }
    }
    let mut misrouted = 0;
    for (position, container) in containers.iter().enumerate() {
        for record in &container.records {
            let routed = containers.iter().rposition(|candidate| candidate.container_number <= record.inode);
            if routed != Some(position) {
                misrouted += 1;
            }
        }
    }
    let empty = length_as_u64(containers.iter().filter(|container| container.records.is_empty()).count());
    (third_false, misrouted, empty)
}

impl Simulation {
    fn start_leaf(geometry: Geometry, policy: LeafPolicy) -> Self {
        let mut simulation = Self::start(geometry, WritePath::WithoutTombstones, PRODUCT_UNLINK_BATCH_DATA_UNITS, None, false);
        simulation.policy = policy;
        simulation.leaf_mode = true;
        simulation
    }

    fn note_leaf_publish(&mut self, plan: &LogicalPlan, txg: u64, operation: PublishOperation) {
        let before: BTreeMap<u64, u64> = self
            .version
            .inode_containers
            .iter()
            .flat_map(|container| container.records.iter().map(|record| (record.inode, record.object_birth)))
            .collect();
        let after: BTreeSet<u64> = plan
            .inode_containers
            .iter()
            .flat_map(|container| container.records.iter().map(|record| record.inode))
            .collect();
        for (inode, birth) in before {
            if !after.contains(&inode) {
                self.record_deaths.insert((inode, birth), txg);
                self.leaf_counts.record_deaths += 1;
            }
        }
        for identity in &plan.retired_identities {
            self.retired.insert(*identity, txg);
            self.leaf_counts.retirements += 1;
        }
        if !plan.retired_identities.is_empty() {
            self.leaf_counts.publishes_with_a_retirement += 1;
        }
        self.leaf_counts.merges += plan.merges;
        self.leaf_counts.borrows += plan.borrows;
        self.leaf_counts.zero_versions += plan.zero_versions;
        self.leaf_counts.oneshot_drops += plan.oneshot_drops;
        if let PublishOperation::UnlinkIntent { inode } = operation {
            if let Some(record) = self.version.live_record_of(inode) {
                self.intent_tombstones.insert((inode, record.object_birth), txg);
            }
        }
        if let PublishOperation::UnlinkFinal { .. } = operation {
            self.leaf_counts.removal_publishes += 1;
            let rewritten = length_as_u64(
                plan.inode_containers
                    .iter()
                    .filter(|container| container.origin == PlannedUnit::Rewritten)
                    .count(),
            );
            self.leaf_counts.containers_rewritten_in_removals += rewritten;
            self.leaf_counts.most_containers_rewritten_in_a_removal =
                self.leaf_counts.most_containers_rewritten_in_a_removal.max(rewritten);
        }
    }

    fn measure_leaf_cells(&mut self, txg: u64) {
        let is_in_unlink = !self.version.deleted_queue.is_empty();
        let readers: &[LeafReader] = match self.policy.zero {
            ZeroKind::NotWritten => &[
                LeafReader::S,
                LeafReader::C1,
                LeafReader::C3,
                LeafReader::C3IntentLt,
                LeafReader::C3IntentLe,
                LeafReader::C1Z,
                LeafReader::C3Z,
            ],
            ZeroKind::Persist | ZeroKind::OneShot => &[LeafReader::S, LeafReader::C2],
        };
        let mut trajectories = std::mem::take(&mut self.leaf_trajectories);
        {
            let inputs = ScanInputs {
                disk: &self.disk,
                unit_lists: &self.unit_lists,
                death_time: &self.death_time,
                planted: None,
                erased: None,
            };
            for reader in readers {
                let reading = leaf_reading(*reader, &inputs, &self.retired, &self.record_deaths, &self.intent_tombstones);
                if *reader == LeafReader::S {
                    let e177 = select_current_reading(Reader::SelectCurrent, Arm::WithoutTombstones, &inputs);
                    let same = LEAF_NAMESPACES.iter().all(|(namespace, _)| {
                        let position = namespace.position();
                        reading.keys[position].keys().eq(e177.keys[position].keys())
                    });
                    if !same {
                        self.leaf_counts.leaf_s_differs_from_e177_r_s += 1;
                    }
                }
                for (namespace, _) in LEAF_NAMESPACES {
                    let position = namespace.position();
                    let deaths = length_as_u64(self.ever_seen[position].len() - self.truth[position].len());
                    let measurement = measure_against_truth(&reading.keys[position], &self.truth[position], txg);
                    trajectories.entry((*reader, namespace)).or_default().observe(txg, measurement, deaths, is_in_unlink);
                }
            }
        }
        self.leaf_trajectories = trajectories;
        let (third_false, misrouted, empty) = leaf_order_structure(&self.version);
        let counts = &mut self.leaf_counts;
        counts.third_sentence_false_pairs_peak = counts.third_sentence_false_pairs_peak.max(third_false);
        counts.versions_with_a_third_sentence_false_pair += u64::from(third_false > 0);
        counts.misrouted_records_peak = counts.misrouted_records_peak.max(misrouted);
        counts.versions_with_an_empty_container += u64::from(empty > 0);
        counts.empty_containers_peak = counts.empty_containers_peak.max(empty);
    }
}

// ---------------------------------------------------------------------------------------------
// 历史：E177 第一段的 51 段（主取样点照它），加合并历史（用户决定的几步——建多少、删哪几个号、什么时候写文件——随机放开扫）
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeafHistory {
    E177(HistorySpec),
    /// 先建 n0 个 inode（每次 50），再走 steps 步：p% 建 1–20 个，5% 覆盖写，3% 顺序写 1–200 个单元，其余 unlink 一个活号（含 1 号）：
    /// focus% 的删挑一片「盯着的」容器（每 50 步换一片，随机挑）里的随机号，其余在全部活号里随机挑。
    MergeRandom { seed: u64, p_percent: u64, n0: u64, steps: u64, focus_percent: u64 },
    /// K3 改号借法的撞号历史（逐步写死，用户动作就是这几步）：一次发布建 2..700（四片，后三片出生代相同 T）；
    /// 删 Y=(467,T) 的 699（Y 与 Z=(700,T) 之和 233，Y 吸收 Z，Z 退役），再删 Y 的 477..698；建 125 个（701..825，落进最右的 Y）；
    /// 删 X=(234,T) 的 235..351（X 从 233 删到 116 条，X+Y=252 > 233 不并，X 从 Y 均分借 (136−116)/2 = 10 条 = 467..476，
    /// Y 借完的最小 key 是 700）。
    RenameCollision,
    /// 同上，但前缀的 n0 − 1 个 inode 在一次发布里建（同一次发布分裂出的几片容器出生代相同）。
    MergeRandomBigBuild { seed: u64, p_percent: u64, n0: u64, steps: u64, focus_percent: u64 },
}

impl LeafHistory {
    fn label(self) -> String {
        match self {
            LeafHistory::E177(spec) => spec.label(),
            LeafHistory::MergeRandom { seed, p_percent, n0, steps, focus_percent } => {
                format!("family=MR seed={seed} p={p_percent} n0={n0} steps={steps} focus={focus_percent}")
            }
            LeafHistory::MergeRandomBigBuild { seed, p_percent, n0, steps, focus_percent } => {
                format!("family=MRB seed={seed} p={p_percent} n0={n0} steps={steps} focus={focus_percent}")
            }
            LeafHistory::RenameCollision => "family=RC".to_string(),
        }
    }

    fn batch(self) -> u64 {
        match self {
            LeafHistory::E177(spec) => spec.batch(),
            LeafHistory::MergeRandom { .. } | LeafHistory::MergeRandomBigBuild { .. } | LeafHistory::RenameCollision => {
                PRODUCT_UNLINK_BATCH_DATA_UNITS
            }
        }
    }
}

fn run_leaf_history(history: LeafHistory, simulation: &mut Simulation) {
    match history {
        LeafHistory::E177(spec) => run_history(spec, simulation),
        LeafHistory::RenameCollision => {
            simulation.apply(UserChange::NewInodes { count: 699 });
            simulation.apply(UserChange::Unlink { inode: 699 });
            for inode in 477..=698 {
                simulation.apply(UserChange::Unlink { inode });
            }
            simulation.apply(UserChange::NewInodes { count: 125 });
            for inode in 235..=351 {
                simulation.apply(UserChange::Unlink { inode });
            }
        }
        LeafHistory::MergeRandom { seed, p_percent, n0, steps, focus_percent }
        | LeafHistory::MergeRandomBigBuild { seed, p_percent, n0, steps, focus_percent } => {
            if matches!(history, LeafHistory::MergeRandomBigBuild { .. }) {
                simulation.apply(UserChange::NewInodes { count: n0 - 1 });
            } else {
                let mut created = 1;
                while created < n0 {
                    let count = NEW_INODES_PER_STEP.min(n0 - created);
                    simulation.apply(UserChange::NewInodes { count });
                    created += count;
                }
            }
            let mut random = SeededRandom::new(seed.wrapping_mul(0x51ED_2701).wrapping_add(p_percent * 1000 + focus_percent));
            let mut focus_position = 0_u64;
            for step in 0..steps {
                if step % 50 == 0 {
                    focus_position = random.below(64);
                }
                let draw = random.below(100);
                let change = if draw < p_percent {
                    UserChange::NewInodes { count: 1 + random.below(20) }
                } else if draw < p_percent + 5 {
                    UserChange::Overwrite
                } else if draw < p_percent + 8 {
                    UserChange::SequentialWrite { data_units: 1 + random.below(200) }
                } else {
                    let live = live_inodes_of(&simulation.version);
                    if live.is_empty() {
                        continue;
                    }
                    let containers = &simulation.version.inode_containers;
                    let focused: Vec<u64> = if containers.is_empty() {
                        Vec::new()
                    } else {
                        let position = u64_as_index(focus_position % length_as_u64(containers.len()));
                        containers[position].records.iter().filter(|record| record.nlink > 0).map(|record| record.inode).collect()
                    };
                    let use_focus = random.below(100) < focus_percent && !focused.is_empty();
                    let pool = if use_focus { focused } else { live };
                    UserChange::Unlink { inode: pool[u64_as_index(random.below(length_as_u64(pool.len())))] }
                };
                simulation.apply(change);
            }
        }
    }
}

fn simulate_leaf(history: LeafHistory, geometry: Geometry, policy: LeafPolicy) -> Simulation {
    let mut simulation = Simulation::start_leaf(geometry, policy);
    simulation.batch_data_units = history.batch();
    run_leaf_history(history, &mut simulation);
    simulation
}

fn leaf_lines(history: LeafHistory, simulation: &Simulation) -> Vec<String> {
    let geometry = match simulation.geometry {
        Geometry::Ring => "ring",
        Geometry::Wide => "wide",
        Geometry::Tight => "tight",
    };
    let head = format!("{} geometry={geometry} policy={}", history.label(), simulation.policy.name());
    let mut lines = Vec::new();
    for ((reader, namespace), trajectory) in &simulation.leaf_trajectories {
        let tree = LEAF_NAMESPACES
            .iter()
            .find(|(candidate, _)| candidate == namespace)
            .map(|(_, name)| *name)
            .expect("只量这五个名字空间");
        lines.push(format!(
            "name=leaf_cell {head} reader={} tree={tree} r_peak={} r_final={} r_positive_scans={} r_out_peak={} m_peak={} m_final={} m_positive_scans={} m_positive_scans_in_unlink={} d_peak={}",
            reader.name(),
            trajectory.resurrected_peak,
            trajectory.resurrected_final,
            trajectory.resurrected_positive_scans,
            trajectory.outside_window_peak,
            trajectory.missed_peak,
            trajectory.missed_final,
            trajectory.missed_positive_scans,
            trajectory.missed_positive_scans_in_unlink,
            trajectory.deaths_peak,
        ));
    }
    let counts = &simulation.leaf_counts;
    lines.push(format!(
        "name=leaf_counts {head} publishes={} stopped_at={} retirements={} publishes_with_a_retirement={} merges={} borrows={} zero_versions={} oneshot_drops={} record_deaths={} removal_publishes={} containers_rewritten_in_removals={} most_containers_rewritten_in_a_removal={} third_sentence_false_pairs_peak={} versions_with_a_third_sentence_false_pair={} misrouted_records_peak={} versions_with_an_empty_container={} empty_containers_peak={} leaf_s_differs_from_e177_r_s={} assertion_violations={}",
        simulation.publishes,
        optional_txg(simulation.does_not_fit_at),
        counts.retirements,
        counts.publishes_with_a_retirement,
        counts.merges,
        counts.borrows,
        counts.zero_versions,
        counts.oneshot_drops,
        counts.record_deaths,
        counts.removal_publishes,
        counts.containers_rewritten_in_removals,
        counts.most_containers_rewritten_in_a_removal,
        counts.third_sentence_false_pairs_peak,
        counts.versions_with_a_third_sentence_false_pair,
        counts.misrouted_records_peak,
        counts.versions_with_an_empty_container,
        counts.empty_containers_peak,
        counts.leaf_s_differs_from_e177_r_s,
        simulation
            .assertions
            .judged
            .keys()
            .filter(|name| **name != "K4_emptied_container_not_written")
            .map(|name| simulation.assertions.violations_of(name))
            .sum::<u64>(),
    ));
    lines
}

fn leaf_jobs(mode: &str) -> Vec<(LeafHistory, Geometry, LeafPolicy)> {
    let mut jobs = Vec::new();
    let geometries = [Geometry::Ring, Geometry::Wide];
    if mode == "leaf-death" || mode == "leaf-death-e177" {
        for spec in segment_one_histories() {
            for geometry in geometries {
                for zero in [ZeroKind::NotWritten, ZeroKind::Persist, ZeroKind::OneShot] {
                    jobs.push((LeafHistory::E177(spec), geometry, LeafPolicy { merge: MergeKind::M1, zero }));
                }
            }
        }
    }
    if mode == "leaf-death" || mode == "leaf-death-merges" {
        for n0 in [700, 1400] {
            for p_percent in [10, 30] {
                for focus_percent in [0, 70] {
                for seed in 1..=8 {
                    let history = LeafHistory::MergeRandom { seed, p_percent, n0, steps: 1000, focus_percent };
                    for geometry in geometries {
                        for merge in [MergeKind::M1, MergeKind::M2, MergeKind::M3Half, MergeKind::M3Just] {
                            jobs.push((history, geometry, LeafPolicy { merge, zero: ZeroKind::NotWritten }));
                        }
                        jobs.push((history, geometry, LeafPolicy { merge: MergeKind::M1, zero: ZeroKind::OneShot }));
                    }
                }
                }
            }
        }
    }
    if mode == "rename-collision" {
        for merge in [MergeKind::M3Half, MergeKind::M3HalfRenameKeepBirth, MergeKind::M3HalfRenameNewBirth] {
            jobs.push((LeafHistory::RenameCollision, Geometry::Wide, LeafPolicy { merge, zero: ZeroKind::NotWritten }));
            jobs.push((LeafHistory::RenameCollision, Geometry::Ring, LeafPolicy { merge, zero: ZeroKind::NotWritten }));
        }
    }
    if mode == "leaf-death-rename" {
        for n0 in [700, 1400] {
            for focus_percent in [0, 70] {
                for seed in 1..=8 {
                    let history = LeafHistory::MergeRandomBigBuild { seed, p_percent: 10, n0, steps: 1000, focus_percent };
                    for merge in [MergeKind::M3Half, MergeKind::M3HalfRenameKeepBirth, MergeKind::M3HalfRenameNewBirth] {
                        jobs.push((history, Geometry::Wide, LeafPolicy { merge, zero: ZeroKind::NotWritten }));
                    }
                }
            }
        }
    }
    jobs
}

fn run_leaf_jobs(jobs: &[(LeafHistory, Geometry, LeafPolicy)], threads: usize) -> Vec<Vec<String>> {
    let next = AtomicUsize::new(0);
    let collected: Mutex<Vec<Option<Vec<String>>>> = Mutex::new(vec![None; jobs.len()]);
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let position = next.fetch_add(1, Ordering::SeqCst);
                if position >= jobs.len() {
                    break;
                }
                let (history, geometry, policy) = jobs[position];
                let simulation = simulate_leaf(history, geometry, policy);
                let lines = leaf_lines(history, &simulation);
                collected.lock().expect("锁没有中毒")[position] = Some(lines);
            });
        }
    });
    collected
        .into_inner()
        .expect("锁没有中毒")
        .into_iter()
        .map(|lines| lines.expect("每一件都收回了"))
        .collect()
}

/// 候选四的判别子：两段历史，最后一次发布之后读得出的码 1 / 码 3 单元（组、写序、内容）逐个相同、根环上最新 txg 相同，而真值不同。
/// A：建 n 个 inode，unlink x（它那一片只装它一条）只发到意图，再发一次零单元的空发布（挂载暖机、为空间推的抬 F 那一类）；
/// B：同一前缀，unlink x 发完意图与最后一批（那一片删空、不落盘）。
fn indistinguishable_pair_lines() -> Vec<String> {
    let mut lines = Vec::new();
    for (geometry, prefix_overwrites) in [(Geometry::Wide, 0_u64), (Geometry::Ring, 0), (Geometry::Ring, 40)] {
        for inode_count in [234_u64, 467, 700] {
            let x = inode_count;
            let build = |finish: bool| {
                let mut simulation = Simulation::start_leaf(geometry, LeafPolicy::TODAY);
                for _ in 0..prefix_overwrites {
                    simulation.apply(UserChange::Overwrite);
                }
                let mut created = 1;
                while created < inode_count {
                    let count = NEW_INODES_PER_STEP.min(inode_count - created);
                    simulation.apply(UserChange::NewInodes { count });
                    created += count;
                }
                simulation.publish_or_stop(PublishOperation::UnlinkIntent { inode: x });
                if finish {
                    simulation.publish_or_stop(PublishOperation::UnlinkFinal { inode: x });
                } else {
                    simulation.publish_or_stop(PublishOperation::WarmUp);
                }
                simulation
            };
            let a = build(false);
            let b = build(true);
            let authoritative = |simulation: &Simulation| -> Vec<String> {
                let mut units: Vec<String> = simulation
                    .disk
                    .units
                    .iter()
                    .filter(|unit| unit.is_readable && (unit.class == UNIT_CLASS_DATA || unit.class == UNIT_CLASS_PACKED))
                    .map(|unit| format!("{:?}|{:?}|{:?}|{:?}", unit.owner, unit.group, unit.write_order, unit.contents))
                    .collect();
                units.sort();
                units
            };
            let units_a = authoritative(&a);
            let units_b = authoritative(&b);
            let x_alive_in_a = a.version.live_record_of(x).is_some();
            let x_alive_in_b = b.version.live_record_of(x).is_some();
            let inputs_of = |simulation: &Simulation| {
                let inputs = ScanInputs {
                    disk: &simulation.disk,
                    unit_lists: &simulation.unit_lists,
                    death_time: &simulation.death_time,
                    planted: None,
                    erased: None,
                };
                let reading = leaf_reading(LeafReader::S, &inputs, &BTreeMap::new(), &BTreeMap::new(), &BTreeMap::new());
                reading.keys[Namespace::Inode.position()].contains_key(&single_field_key(x))
            };
            lines.push(format!(
                "name=indistinguishable geometry={} prefix_overwrites={prefix_overwrites} inode_count={inode_count} x={x} latest_txg_a={} latest_txg_b={} code1_code3_readable_units_a={} code1_code3_readable_units_b={} units_identical={} x_alive_a={x_alive_in_a} x_alive_b={x_alive_in_b} reader_s_reads_x_a={} reader_s_reads_x_b={}",
                match geometry {
                    Geometry::Ring => "ring",
                    Geometry::Wide => "wide",
                    Geometry::Tight => "tight",
                },
                a.version.txg,
                b.version.txg,
                units_a.len(),
                units_b.len(),
                units_a == units_b,
                inputs_of(&a),
                inputs_of(&b),
            ));
        }
    }
    lines
}

// ---------------------------------------------------------------------------------------------
// 回退示例（手造单元，不经分配器）：候选一的退役记录在管理员回退之后杀不杀活容器，按「退役记录带不带死亡 txg」
// 与「回退那次发布的退役树随现行那一版还是随 R_old」两个旋钮，以及读侧按不按谱系择现行（C615 的读侧补上之后）。
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct HandUnit {
    /// (种类, 容器号, 容器出生代)：种类 2 = inode 叶容器，种类 3 = 退役记录容器。
    group: (u64, u64, u64),
    birth: u64,
    inode_records: Vec<u64>,
    retirements: Vec<((u64, u64), u64)>,
}

fn hand_lineage_contains(latest: u64, rollback_txg: u64, rollback_target: u64, txg: u64) -> bool {
    let mut cursor = latest;
    loop {
        if cursor == txg {
            return true;
        }
        if cursor == 0 || cursor < txg && cursor != rollback_txg {
            return false;
        }
        cursor = if cursor == rollback_txg { rollback_target } else { cursor - 1 };
    }
}

fn rollback_demo_lines() -> Vec<String> {
    let a: Vec<u64> = (1..=233).collect();
    let inode = |number: u64, birth: u64, unit_birth: u64, records: Vec<u64>| HandUnit {
        group: (2, number, birth),
        birth: unit_birth,
        inode_records: records,
        retirements: Vec::new(),
    };
    let retirement = |number: u64, birth: u64, unit_birth: u64, records: Vec<((u64, u64), u64)>| HandUnit {
        group: (3, number, birth),
        birth: unit_birth,
        inode_records: Vec::new(),
        retirements: records,
    };
    let (rollback_txg, rollback_target, latest) = (15_u64, 10_u64, 17_u64);
    let mut lines = Vec::new();
    for carried_from_current in [true, false] {
        let mut disk: Vec<HandUnit> = vec![
            inode(1, 3, 3, a.clone()),
            inode(234, 8, 9, vec![234, 235]),
            inode(467, 10, 10, vec![467]),
            // 被回退掉的那一段：删 234、235，B 删空不落盘、写退役记录 (234, 8)，死亡 txg 14。
            inode(234, 8, 11, vec![234, 235]),
            inode(234, 8, 12, vec![235]),
            inode(234, 8, 13, vec![235]),
            retirement(1, 14, 14, vec![((234, 8), 14)]),
            // 回退之后（txg 16 意图、17 最后一批）：C 删空不落盘、写退役记录 (467, 10)，死亡 txg 17。
            inode(467, 10, 16, vec![467]),
        ];
        if carried_from_current {
            disk.push(retirement(1, 14, 17, vec![((234, 8), 14), ((467, 10), 17)]));
        } else {
            disk.push(retirement(1, 17, 17, vec![((467, 10), 17)]));
        }
        let truth: BTreeSet<u64> = a.iter().copied().chain([234, 235]).collect();
        for by_lineage in [false, true] {
            for record_carries_txg in [false, true] {
                let on_lineage = |txg: u64| hand_lineage_contains(latest, rollback_txg, rollback_target, txg);
                let mut newest: BTreeMap<(u64, u64, u64), &HandUnit> = BTreeMap::new();
                for unit in &disk {
                    if by_lineage && !on_lineage(unit.birth) {
                        continue;
                    }
                    let slot = newest.entry(unit.group).or_insert(unit);
                    if unit.birth > slot.birth {
                        *slot = unit;
                    }
                }
                let mut retired: BTreeSet<(u64, u64)> = BTreeSet::new();
                for unit in newest.values().filter(|unit| unit.group.0 == 3) {
                    for (identity, death) in &unit.retirements {
                        if by_lineage && record_carries_txg && !on_lineage(*death) {
                            continue;
                        }
                        retired.insert(*identity);
                    }
                }
                let mut read: BTreeSet<u64> = BTreeSet::new();
                for unit in newest.values().filter(|unit| unit.group.0 == 2) {
                    if retired.contains(&(unit.group.1, unit.group.2)) {
                        continue;
                    }
                    read.extend(unit.inode_records.iter().copied());
                }
                let missed = truth.difference(&read).count();
                let resurrected = read.difference(&truth).count();
                lines.push(format!(
                    "name=rollback_demo retirement_tree_at_rollback={} select_by_lineage={by_lineage} retirement_record_carries_death_txg={record_carries_txg} resurrected={resurrected} missed={missed} missed_inodes={:?}",
                    if carried_from_current { "carried_from_current" } else { "taken_from_r_old" },
                    truth.difference(&read).copied().filter(|inode| *inode > 233).collect::<Vec<u64>>(),
                ));
            }
        }
    }
    lines
}

fn leaf_death_main(mode: &str, threads: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config model=m3-leaf-death-r1-opus mode={mode} base=E177_device_copy m3_floor={M3_FLOOR} inode_leaf_records={INODE_LEAF_RECORDS} ring_window={RING_WINDOW_IN_PUBLISHES}"
    )];
    match mode {
        "indistinguishable" => lines.extend(indistinguishable_pair_lines()),
        "rollback-demo" => lines.extend(rollback_demo_lines()),
        _ => {
            let all_jobs = leaf_jobs(mode);
            // 分批跑：LEAF_JOB_RANGE=起:止（左闭右开，按 leaf_jobs 的次序）；不设就跑全部。
            let (start, end) = std::env::var("LEAF_JOB_RANGE")
                .ok()
                .and_then(|range| {
                    let (start, end) = range.split_once(':')?;
                    Some((start.parse::<usize>().ok()?, end.parse::<usize>().ok()?))
                })
                .unwrap_or((0, all_jobs.len()));
            let end = end.min(all_jobs.len());
            let jobs = &all_jobs[start.min(end)..end];
            lines.push(format!("name=job_range mode={mode} total_jobs={} start={start} end={end}", all_jobs.len()));
            for job_lines in run_leaf_jobs(jobs, threads) {
                lines.extend(job_lines);
            }
        }
    }
    lines
}
