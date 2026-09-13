//! E150：回退复用被抛弃的根引用的单元 —— C314 那两格的计数模型，与两种修法各付什么。
//!
//! ## 模型（判据与失败条款的权威登记在 research/prompts/e150-preregistration.md，跑前写死 2026-09-13）
//!
//! 一块盘。根环 3 区域 × 4 槽（区域 = txg mod 3、槽 = (txg div 3) mod 4，D22 已定项 2），单元区 32 个槽。
//! 每个根记录 = (实例代号, txg, 引用的单元 → 期望的写者, 实例表单元槽号 → 期望的表号)；根的「账」= 引用的单元 + 实例表单元。
//! 一次发布按 D16 已定项 7：COW 单元 → 屏障 → journal 记录（只作序点）→ 屏障 → 根槽 FUA。
//! 管理员回退按 D23 已定项 14：新实例代号、新实例表单元（回退行 + 中间实例行）、新根 txg = 环里最大 + 1，账从 R_old 载入。
//! 恢复：最新可读的根 + 它自己指的那份实例表；表判它的实例无效就退到下一个；选定后逐个核对所引用单元的写者。
//! 违例 = 恢复挂上的根引用的单元里有一个被别人覆写过。
//!
//! | 臂 | 分配器 | 回退多写什么 |
//! |---|---|---|
//! | 基线 | 只看当前根的账 | 无 |
//! | F-A 影子账 | 只从环里每一个可读根的账都空闲的槽取 | 无 |
//! | F-B 墓碑 | 同基线 | 回退发布前把被抛弃时间线的每个根槽用墓碑 FUA 覆写 |
//!
//! ## 模型取法（不是条款）
//!
//! mkfs 把实例表单元放在单元区最高的槽（让锚点的编号照跑前登记走）；journal 记录不占单元槽；
//! 空发布只写记录与根；单元的「内容」就是写者的身份，不建校验和——覆写即可判。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

const RING_REGIONS: u64 = 3;
const RING_SLOTS_PER_REGION: u64 = 4;
const RING_CAPACITY: usize = (RING_REGIONS * RING_SLOTS_PER_REGION) as usize;
const UNIT_SLOTS: usize = 32;
const DIRTY_UNITS_PER_PUBLISH: usize = 1;
const INITIAL_DATA_UNITS: usize = 3;
const NORMAL_PUBLISHES_BEFORE_ROLLBACK: u64 = 5;
const ROLLBACK_TARGET_TXG: u64 = 3;
const WARM_UP_EMPTY_PUBLISHES_MAXIMUM: u64 = 3;
const FOLLOW_UP_PUBLISHES_MAXIMUM: u64 = 12;

/// 根槽在环里的编号：区域 = txg mod R，槽 = (txg div R) mod S（D22 已定项 2 的跨区轮转）。
fn ring_slot_for(txg: u64) -> usize {
    ((txg % RING_REGIONS) * RING_SLOTS_PER_REGION + (txg / RING_REGIONS) % RING_SLOTS_PER_REGION) as usize
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Baseline,
    ShadowAccounts,
    Tombstones,
}

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::Baseline => "baseline",
            Arm::ShadowAccounts => "shadow_accounts",
            Arm::Tombstones => "tombstones",
        }
    }
}

/// 单元槽里住的东西：数据（带写者根身份）、实例表（带表号）、空。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnitContent {
    Empty,
    Data { writer_root: u32 },
    InstanceTable { table: u32 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RootRecord {
    identity: u32,
    instance: u32,
    txg: u64,
    /// 引用的数据单元 → 发布时它的写者根身份（恢复核对的期望值）。
    references: BTreeMap<usize, u32>,
    instance_table_slot: usize,
    instance_table: u32,
}

impl RootRecord {
    /// 这个根的账：它引用的单元 + 它的实例表单元。
    fn account(&self) -> BTreeSet<usize> {
        let mut account: BTreeSet<usize> = self.references.keys().copied().collect();
        account.insert(self.instance_table_slot);
        account
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum RingSlot {
    Empty,
    Root(RootRecord),
    Tombstone,
}

/// 实例表：行 (实例, T, W)——实例 i 的根 (i, txg) 有效 ⟺ 无 i 的行，或 txg ≤ T。
#[derive(Clone, PartialEq, Eq, Debug)]
struct InstanceTable {
    rows: Vec<(u32, u64, u64)>,
}

impl InstanceTable {
    fn root_is_valid(&self, instance: u32, txg: u64) -> bool {
        self.rows.iter().filter(|(row_instance, _, _)| *row_instance == instance).all(|(_, valid_up_to, _)| txg <= *valid_up_to)
    }
}

/// 盘的持久态：根环、单元区、每个根槽可不可读（故障注入）。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Disk {
    ring: Vec<RingSlot>,
    units: Vec<UnitContent>,
    slot_unreadable: Vec<bool>,
}

impl Disk {
    fn new() -> Disk {
        Disk { ring: vec![RingSlot::Empty; RING_CAPACITY], units: vec![UnitContent::Empty; UNIT_SLOTS], slot_unreadable: vec![false; RING_CAPACITY] }
    }

    fn readable_roots(&self) -> Vec<RootRecord> {
        self.ring
            .iter()
            .enumerate()
            .filter(|(slot, _)| !self.slot_unreadable[*slot])
            .filter_map(|(_, ring_slot)| match ring_slot {
                RingSlot::Root(record) => Some(record.clone()),
                RingSlot::Empty | RingSlot::Tombstone => None,
            })
            .collect()
    }
}

/// 一次写请求：写到根槽或单元槽；FUA 只在根槽与墓碑上。
#[derive(Clone, PartialEq, Eq, Debug)]
enum WriteRequest {
    Unit { slot: usize, content: UnitContent },
    Root { slot: usize, record: RootRecord },
    Tombstone { slot: usize },
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum RecordedOperation {
    Write { request: WriteRequest, is_fua: bool },
    Barrier,
}

fn apply_write(disk: &mut Disk, request: &WriteRequest) {
    match request {
        WriteRequest::Unit { slot, content } => disk.units[*slot] = *content,
        WriteRequest::Root { slot, record } => disk.ring[*slot] = RingSlot::Root(record.clone()),
        WriteRequest::Tombstone { slot } => disk.ring[*slot] = RingSlot::Tombstone,
    }
}

/// 恢复的结果：选了哪个根、它引用的单元里有几个被覆写。
#[derive(Clone, PartialEq, Eq, Debug)]
struct RecoveryOutcome {
    chosen: Option<RootRecord>,
    overwritten_references: Vec<usize>,
}

impl RecoveryOutcome {
    fn is_violation(&self) -> bool {
        !self.overwritten_references.is_empty()
    }
}

/// 最新可读的根 + 它自己指的实例表判它有效 ⇒ 选它；实例表单元被覆写的根自证不过、跳过。
fn recover(disk: &Disk, tables: &[InstanceTable]) -> RecoveryOutcome {
    let mut candidates = disk.readable_roots();
    candidates.sort_by(|left, right| (right.txg, right.instance).cmp(&(left.txg, left.instance)));
    for candidate in candidates {
        let table_content = disk.units[candidate.instance_table_slot];
        let table = match table_content {
            UnitContent::InstanceTable { table } if table == candidate.instance_table => &tables[table as usize],
            UnitContent::InstanceTable { .. } | UnitContent::Data { .. } | UnitContent::Empty => continue,
        };
        if !table.root_is_valid(candidate.instance, candidate.txg) {
            continue;
        }
        let overwritten_references: Vec<usize> = candidate
            .references
            .iter()
            .filter(|(unit, expected_writer)| disk.units[**unit] != UnitContent::Data { writer_root: **expected_writer })
            .map(|(unit, _)| *unit)
            .collect();
        return RecoveryOutcome { chosen: Some(candidate), overwritten_references };
    }
    RecoveryOutcome { chosen: None, overwritten_references: Vec::new() }
}

/// 运行中的文件系统：当前根、实例、表、下一个根身份；写全部经录制器。
struct Model {
    arm: Arm,
    disk: Disk,
    tables: Vec<InstanceTable>,
    current: RootRecord,
    next_root_identity: u32,
    next_instance: u32,
    operations: Vec<RecordedOperation>,
    tombstone_writes: u64,
}

impl Model {
    fn mkfs(arm: Arm) -> Model {
        let mut disk = Disk::new();
        let table_slot = UNIT_SLOTS - 1;
        let mut tables = vec![InstanceTable { rows: Vec::new() }];
        disk.units[table_slot] = UnitContent::InstanceTable { table: 0 };
        let mut references = BTreeMap::new();
        for unit in 0..INITIAL_DATA_UNITS {
            disk.units[unit] = UnitContent::Data { writer_root: 0 };
            references.insert(unit, 0);
        }
        let genesis = RootRecord { identity: 0, instance: 1, txg: 0, references, instance_table_slot: table_slot, instance_table: 0 };
        disk.ring[ring_slot_for(0)] = RingSlot::Root(genesis.clone());
        tables.truncate(1);
        Model { arm, disk, tables, current: genesis, next_root_identity: 1, next_instance: 2, operations: Vec::new(), tombstone_writes: 0 }
    }

    fn record(&mut self, operation: RecordedOperation) {
        if let RecordedOperation::Write { request, .. } = &operation {
            apply_write(&mut self.disk, request);
        }
        self.operations.push(operation);
    }

    /// 分配器：当前账 + I-7.4 护着的回退候选集（环里可读、按当前实例表判有效的根）的账——这是基线；
    /// 被抛弃的根（表判无效）的账基线不护，影子账连它们一起护。最低槽号优先。
    fn forbidden_slots(&self) -> BTreeSet<usize> {
        let mut forbidden = self.current.account();
        let table = &self.tables[self.current.instance_table as usize];
        for root in self.disk.readable_roots() {
            if self.arm == Arm::ShadowAccounts || table.root_is_valid(root.instance, root.txg) {
                forbidden.extend(root.account());
            }
        }
        forbidden
    }

    fn allocate(&self, taken_this_publish: &BTreeSet<usize>) -> usize {
        let forbidden = self.forbidden_slots();
        (0..UNIT_SLOTS).find(|slot| !forbidden.contains(slot) && !taken_this_publish.contains(slot)).expect("单元区用尽")
    }

    /// 影子账隔离了多少（全部）：环里可读根的账的并集 − 当前账。稳态下不为 0——回退候选集本来就要 I-7.4 那样护着。
    fn quarantined_units(&self) -> usize {
        let mut union: BTreeSet<usize> = BTreeSet::new();
        for root in self.disk.readable_roots() {
            union.extend(root.account());
        }
        union.difference(&self.current.account()).count()
    }

    /// 只因被抛弃的根（按当前实例表判无效）才隔离的单元：它们的账的并集 − 有效根（含当前根）的账的并集。
    fn quarantined_by_abandoned_roots(&self) -> usize {
        let table = &self.tables[self.current.instance_table as usize];
        let mut abandoned: BTreeSet<usize> = BTreeSet::new();
        let mut valid: BTreeSet<usize> = self.current.account();
        for root in self.disk.readable_roots() {
            if table.root_is_valid(root.instance, root.txg) {
                valid.extend(root.account());
            } else {
                abandoned.extend(root.account());
            }
        }
        abandoned.difference(&valid).count()
    }

    fn maximum_txg_in_ring(&self) -> u64 {
        self.disk
            .ring
            .iter()
            .filter_map(|ring_slot| match ring_slot {
                RingSlot::Root(record) => Some(record.txg),
                RingSlot::Empty | RingSlot::Tombstone => None,
            })
            .max()
            .expect("环里至少有 mkfs 的根")
    }

    /// 一次发布：COW 脏单元（按 key 序取当前引用里最小的几个）→ 屏障 → 记录 → 屏障 → 根槽 FUA。
    /// `extra_unit` 是这次发布还要写的一个非数据单元（回退时的新实例表）。
    fn publish(&mut self, dirty_count: usize, new_instance_table: Option<InstanceTable>) -> RootRecord {
        let identity = self.next_root_identity;
        self.next_root_identity += 1;
        let mut taken: BTreeSet<usize> = BTreeSet::new();
        let mut references = self.current.references.clone();
        let (instance_table_slot, instance_table) = match new_instance_table {
            Some(table) => {
                let table_identifier = self.tables.len() as u32;
                self.tables.push(table);
                // 回退的判定（哪些根被抛弃）从这一刻起对分配器生效；表单元的槽号随后才定
                self.current.instance_table = table_identifier;
                let slot = self.allocate(&taken);
                taken.insert(slot);
                self.record(RecordedOperation::Write { request: WriteRequest::Unit { slot, content: UnitContent::InstanceTable { table: table_identifier } }, is_fua: false });
                (slot, table_identifier)
            }
            None => (self.current.instance_table_slot, self.current.instance_table),
        };
        // 脏的是写者最老的那几个单元（按写者身份、再按槽号）：让每次发布都碰到新槽，被抛弃的根才会引用 R_old 账里空闲的槽
        let mut by_age: Vec<(u32, usize)> = self.current.references.iter().map(|(unit, writer)| (*writer, *unit)).collect();
        by_age.sort_unstable();
        let dirty: Vec<usize> = by_age.iter().take(dirty_count).map(|(_, unit)| *unit).collect();
        for old_unit in dirty {
            let slot = self.allocate(&taken);
            taken.insert(slot);
            self.record(RecordedOperation::Write { request: WriteRequest::Unit { slot, content: UnitContent::Data { writer_root: identity } }, is_fua: false });
            references.remove(&old_unit);
            references.insert(slot, identity);
        }
        self.record(RecordedOperation::Barrier);
        // journal 记录只作序点：写到根环之外的地方，模型不建它的槽
        self.record(RecordedOperation::Barrier);
        let txg = self.current.txg + 1;
        let root = RootRecord { identity, instance: self.current.instance, txg, references, instance_table_slot, instance_table };
        self.record(RecordedOperation::Write { request: WriteRequest::Root { slot: ring_slot_for(txg), record: root.clone() }, is_fua: true });
        self.current = root.clone();
        root
    }

    /// 管理员回退到 target（环里可读的一个根）：新实例、新表（回退行 + 中间实例行）、新根 txg = 环里最大 + 1；
    /// 墓碑臂先把被抛弃时间线（target 之后、同实例）的根槽逐个墓碑 FUA。回退发布里带 dirty_count 个 COW 单元。
    fn rollback_to(&mut self, target: &RootRecord, dirty_count: usize) -> RootRecord {
        let abandoned: Vec<usize> = self
            .disk
            .ring
            .iter()
            .enumerate()
            .filter_map(|(slot, ring_slot)| match ring_slot {
                RingSlot::Root(record) if record.txg > target.txg => Some(slot),
                RingSlot::Root(_) | RingSlot::Empty | RingSlot::Tombstone => None,
            })
            .collect();
        let abandoned_instances: BTreeSet<u32> = self
            .disk
            .ring
            .iter()
            .filter_map(|ring_slot| match ring_slot {
                RingSlot::Root(record) if record.txg > target.txg && record.instance != target.instance => Some(record.instance),
                RingSlot::Root(_) | RingSlot::Empty | RingSlot::Tombstone => None,
            })
            .collect();
        let new_txg_minus_one = self.maximum_txg_in_ring();
        if self.arm == Arm::Tombstones {
            for slot in &abandoned {
                self.record(RecordedOperation::Write { request: WriteRequest::Tombstone { slot: *slot }, is_fua: true });
                self.tombstone_writes += 1;
            }
        }
        let new_instance = self.next_instance;
        self.next_instance += 1;
        let mut rows = self.tables[target.instance_table as usize].rows.clone();
        rows.push((target.instance, target.txg, 0));
        for instance in abandoned_instances {
            rows.push((instance, 0, 0));
        }
        // 账从 R_old 载入；实例代号换成新的；txg 从环里最大 + 1 起
        self.current = RootRecord { identity: target.identity, instance: new_instance, txg: new_txg_minus_one, references: target.references.clone(), instance_table_slot: target.instance_table_slot, instance_table: target.instance_table };
        self.publish(dirty_count, Some(InstanceTable { rows }))
    }
}

/// D13 已定项 4：屏障（与 FUA 写）切段，段内任意整写子集。返回段的操作下标列表。
fn split_into_segments(operations: &[RecordedOperation]) -> Vec<Vec<usize>> {
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for (index, operation) in operations.iter().enumerate() {
        match operation {
            RecordedOperation::Barrier => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            RecordedOperation::Write { is_fua: true, .. } => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                segments.push(vec![index]);
            }
            RecordedOperation::Write { is_fua: false, .. } => current.push(index),
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
    1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum::<u64>()
}

/// 枚举全部崩溃状态：前若干段全持久、当前段任意非空子集持久、之后没持久；对每个状态恢复并判违例。
fn enumerate_crash_states(base: &Disk, operations: &[RecordedOperation], tables: &[InstanceTable]) -> (u64, u64, Option<String>) {
    let segments = split_into_segments(operations);
    let mut states = 0u64;
    let mut violations = 0u64;
    let mut first_violation: Option<String> = None;
    let mut check = |disk: &Disk, description: String| {
        states += 1;
        let outcome = recover(disk, tables);
        if outcome.is_violation() {
            violations += 1;
            if first_violation.is_none() {
                first_violation = Some(format!("{description} chosen={} overwritten={:?}", outcome.chosen.as_ref().map_or(0, |root| root.identity), outcome.overwritten_references));
            }
        }
    };
    check(base, "nothing_persisted".into());
    let mut prefix = base.clone();
    for (segment_index, segment) in segments.iter().enumerate() {
        for subset in 1u64..(1u64 << segment.len()) {
            let mut disk = prefix.clone();
            for (bit, operation_index) in segment.iter().enumerate() {
                if subset & (1 << bit) != 0 {
                    if let RecordedOperation::Write { request, .. } = &operations[*operation_index] {
                        apply_write(&mut disk, request);
                    }
                }
            }
            check(&disk, format!("segment={segment_index} subset={subset:#b}"));
        }
        for operation_index in segment {
            if let RecordedOperation::Write { request, .. } = &operations[*operation_index] {
                apply_write(&mut prefix, request);
            }
        }
    }
    (states, violations, first_violation)
}

/// 场景一（判据 3 / 4 的 ① 格）：五次发布、回退到 txg 3、回退发布带两个 COW 单元，枚举回退发布的崩溃状态。
struct CaseOneOutcome {
    states: u64,
    closed_form: u64,
    violations: u64,
    first_violation: Option<String>,
    tombstone_writes: u64,
}

fn run_case_one(arm: Arm) -> CaseOneOutcome {
    let mut model = Model::mkfs(arm);
    let mut roots = vec![model.current.clone()];
    for _ in 0..NORMAL_PUBLISHES_BEFORE_ROLLBACK {
        roots.push(model.publish(DIRTY_UNITS_PER_PUBLISH, None));
    }
    let target = roots[ROLLBACK_TARGET_TXG as usize].clone();
    let base = model.disk.clone();
    model.operations.clear();
    model.rollback_to(&target, DIRTY_UNITS_PER_PUBLISH);
    let segments = split_into_segments(&model.operations);
    let (states, violations, first_violation) = enumerate_crash_states(&base, &model.operations, &model.tables);
    CaseOneOutcome { states, closed_form: closed_form_state_count(&segments), violations, first_violation, tombstone_writes: model.tombstone_writes }
}

/// 场景二（② 格）：回退发布完成后（可选暖机）跑 P 次普通发布，让回退实例的根全读不出，恢复并判违例。
struct CaseTwoOutcome {
    violations: u64,
    faults_injected: usize,
    quarantined_before_fault: usize,
    quarantined_by_abandoned_before_fault: usize,
    chosen_identity: Option<u32>,
    chosen_is_abandoned: bool,
}

fn run_case_two(arm: Arm, warm_up: bool, follow_up_publishes: u64) -> CaseTwoOutcome {
    let mut model = Model::mkfs(arm);
    let mut roots = vec![model.current.clone()];
    for _ in 0..NORMAL_PUBLISHES_BEFORE_ROLLBACK {
        roots.push(model.publish(DIRTY_UNITS_PER_PUBLISH, None));
    }
    let target = roots[ROLLBACK_TARGET_TXG as usize].clone();
    let abandoned_identities: BTreeSet<u32> = roots.iter().filter(|root| root.txg > target.txg).map(|root| root.identity).collect();
    let rollback_root = model.rollback_to(&target, DIRTY_UNITS_PER_PUBLISH);
    let rollback_instance = rollback_root.instance;
    if warm_up {
        for _ in 0..WARM_UP_EMPTY_PUBLISHES_MAXIMUM {
            model.publish(0, None);
        }
    }
    for _ in 0..follow_up_publishes {
        model.publish(DIRTY_UNITS_PER_PUBLISH, None);
    }
    let quarantined_before_fault = model.quarantined_units();
    let quarantined_by_abandoned_before_fault = model.quarantined_by_abandoned_roots();
    let mut faults_injected = 0usize;
    for slot in 0..RING_CAPACITY {
        if let RingSlot::Root(record) = &model.disk.ring[slot] {
            if record.instance == rollback_instance {
                model.disk.slot_unreadable[slot] = true;
                faults_injected += 1;
            }
        }
    }
    let outcome = recover(&model.disk, &model.tables);
    let chosen_identity = outcome.chosen.as_ref().map(|root| root.identity);
    CaseTwoOutcome {
        violations: u64::from(outcome.is_violation()),
        faults_injected,
        quarantined_before_fault,
        quarantined_by_abandoned_before_fault,
        chosen_identity,
        chosen_is_abandoned: chosen_identity.is_some_and(|identity| abandoned_identities.contains(&identity)),
    }
}

/// 判据 1 的手算锚点几何：8 个单元、mkfs 引用 {0,1}、发布脏 1 个。
fn anchor_model(arm: Arm) -> (Model, RootRecord) {
    let mut model = Model::mkfs(arm);
    // 锚点用 8 个单元：mkfs 的实例表挪到槽 7、数据单元只有 {0,1}
    let table_slot = 7usize;
    model.disk.units = vec![UnitContent::Empty; UNIT_SLOTS];
    model.disk.units[0] = UnitContent::Data { writer_root: 0 };
    model.disk.units[1] = UnitContent::Data { writer_root: 0 };
    model.disk.units[table_slot] = UnitContent::InstanceTable { table: 0 };
    model.current.references = BTreeMap::from([(0usize, 0u32), (1, 0)]);
    model.current.instance_table_slot = table_slot;
    model.disk.ring[ring_slot_for(0)] = RingSlot::Root(model.current.clone());
    let root_one = model.publish_anchor(0);
    let _root_two = model.publish_anchor(1);
    (model, root_one)
}

impl Model {
    /// 锚点专用：脏的是当前引用里从大数起第 n 个 key（跑前登记的锚点：txg 1 脏 {1}、txg 2 脏 {0}）。
    fn publish_anchor(&mut self, dirty_position_from_end: usize) -> RootRecord {
        let identity = self.next_root_identity;
        self.next_root_identity += 1;
        let mut references = self.current.references.clone();
        let old_unit = *self.current.references.keys().rev().nth(dirty_position_from_end).expect("锚点引用至少两个");
        let slot = (0..8).find(|slot| !self.forbidden_slots().contains(slot)).expect("锚点单元区 8 个");
        self.record(RecordedOperation::Write { request: WriteRequest::Unit { slot, content: UnitContent::Data { writer_root: identity } }, is_fua: false });
        references.remove(&old_unit);
        references.insert(slot, identity);
        self.record(RecordedOperation::Barrier);
        self.record(RecordedOperation::Barrier);
        let txg = self.current.txg + 1;
        let root = RootRecord { identity, instance: self.current.instance, txg, references, instance_table_slot: self.current.instance_table_slot, instance_table: self.current.instance_table };
        self.record(RecordedOperation::Write { request: WriteRequest::Root { slot: ring_slot_for(txg), record: root.clone() }, is_fua: true });
        self.current = root.clone();
        root
    }
}

struct AnchorOutcome {
    table_slot_chosen: usize,
    states: u64,
    violations: u64,
    chosen_after_table_only: Option<u32>,
}

fn run_anchor(arm: Arm) -> AnchorOutcome {
    let (mut model, root_one) = anchor_model(arm);
    let base = model.disk.clone();
    model.operations.clear();
    model.rollback_to(&root_one, 0);
    let table_slot_chosen = model.current.instance_table_slot;
    let (states, violations, _) = enumerate_crash_states(&base, &model.operations, &model.tables);
    // 「只有实例表单元（与墓碑）持久、新根没持久」那个状态
    let mut disk = base.clone();
    for operation in &model.operations {
        if let RecordedOperation::Write { request, .. } = operation {
            match request {
                WriteRequest::Unit { .. } | WriteRequest::Tombstone { .. } => apply_write(&mut disk, request),
                WriteRequest::Root { .. } => {}
            }
        }
    }
    let chosen_after_table_only = recover(&disk, &model.tables).chosen.map(|root| root.identity);
    AnchorOutcome { table_slot_chosen, states, violations, chosen_after_table_only }
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config devices=1 ring_regions={RING_REGIONS} ring_slots_per_region={RING_SLOTS_PER_REGION} unit_slots={UNIT_SLOTS} dirty_per_publish={DIRTY_UNITS_PER_PUBLISH} normal_publishes={NORMAL_PUBLISHES_BEFORE_ROLLBACK} rollback_target_txg={ROLLBACK_TARGET_TXG} warm_up_empty_publishes={WARM_UP_EMPTY_PUBLISHES_MAXIMUM} follow_up_max={FOLLOW_UP_PUBLISHES_MAXIMUM} fua_is_boundary=true model=counting file_ops=0"
        ))
    );

    // 判据 1：手算锚点，三条臂各一行
    for arm in [Arm::Baseline, Arm::ShadowAccounts, Arm::Tombstones] {
        let anchor = run_anchor(arm);
        let (expected_slot, expected_violations, expected_chosen) = match arm {
            Arm::Baseline => (3usize, true, Some(2u32)),
            Arm::ShadowAccounts => (4, false, Some(2)),
            Arm::Tombstones => (3, false, Some(1)),
        };
        assert_eq!(anchor.table_slot_chosen, expected_slot, "{:?} 锚点：实例表单元该落在槽 {expected_slot}", arm);
        assert_eq!(anchor.violations > 0, expected_violations, "{:?} 锚点：有没有违例与手算不符", arm);
        assert_eq!(anchor.chosen_after_table_only, expected_chosen, "{:?} 锚点：只有单元持久时恢复选的根与手算不符", arm);
        println!(
            "{}",
            emitter.emit_raw(&format!("name=anchor arm={} table_slot={} states={} violations={} chosen_after_units_only={:?}", arm.tag(), anchor.table_slot_chosen, anchor.states, anchor.violations, anchor.chosen_after_table_only))
        );
    }

    // 判据 2 / 3 / 4：场景一
    let mut case_one_violations: BTreeMap<&'static str, u64> = BTreeMap::new();
    for arm in [Arm::Baseline, Arm::ShadowAccounts, Arm::Tombstones] {
        let outcome = run_case_one(arm);
        assert_eq!(outcome.states, outcome.closed_form, "{:?} 崩溃状态数与闭式不符", arm);
        case_one_violations.insert(arm.tag(), outcome.violations);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=case1 arm={} states={} closed_form={} violations={} tombstone_writes={} first_violation={}",
                arm.tag(),
                outcome.states,
                outcome.closed_form,
                outcome.violations,
                outcome.tombstone_writes,
                outcome.first_violation.unwrap_or_else(|| "none".into())
            ))
        );
    }

    // 场景二：P × 暖机 × 臂
    let mut case_two_violations: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut shadow_quarantine_zero_at: Option<u64> = None;
    for arm in [Arm::Baseline, Arm::ShadowAccounts, Arm::Tombstones] {
        for warm_up in [false, true] {
            for follow_up in 0..=FOLLOW_UP_PUBLISHES_MAXIMUM {
                let outcome = run_case_two(arm, warm_up, follow_up);
                *case_two_violations.entry(arm.tag()).or_insert(0) += outcome.violations;
                if arm == Arm::ShadowAccounts && !warm_up && outcome.quarantined_by_abandoned_before_fault == 0 && shadow_quarantine_zero_at.is_none() {
                    shadow_quarantine_zero_at = Some(follow_up);
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=case2 arm={} warm_up={warm_up} follow_up={follow_up} faults={} chosen={:?} chosen_is_abandoned={} violation={} quarantined_total={} quarantined_by_abandoned={}",
                        arm.tag(),
                        outcome.faults_injected,
                        outcome.chosen_identity,
                        outcome.chosen_is_abandoned,
                        outcome.violations,
                        outcome.quarantined_before_fault,
                        outcome.quarantined_by_abandoned_before_fault
                    ))
                );
            }
        }
    }

    // 判据 3：阳性对照——基线两格都要有违例，否则整轮作废
    assert!(case_one_violations["baseline"] > 0 && case_two_violations["baseline"] > 0, "基线两格都必须有违例，否则模型看不见 C314");
    let shadow_clean = case_one_violations["shadow_accounts"] == 0 && case_two_violations["shadow_accounts"] == 0;
    let tombstones_clean = case_one_violations["tombstones"] == 0 && case_two_violations["tombstones"] == 0;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=verdict baseline_case1={} baseline_case2={} shadow_case1={} shadow_case2={} tombstones_case1={} tombstones_case2={} shadow_clean={shadow_clean} tombstones_clean={tombstones_clean} shadow_quarantine_zero_at_follow_up={}",
            case_one_violations["baseline"],
            case_two_violations["baseline"],
            case_one_violations["shadow_accounts"],
            case_two_violations["shadow_accounts"],
            case_one_violations["tombstones"],
            case_two_violations["tombstones"],
            shadow_quarantine_zero_at.map_or("never".to_string(), |follow_up| follow_up.to_string())
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1（按跑前登记的修正）：基线锚点——R0 {0,1}、R1 {0,2}、R2 {2,3}（I-7.4 护着 R0 / R1 的 0 与 1，R2 的新槽只能是 3）；
    /// 回退到 R1 后 R2 被抛弃、3 是它独占的，实例表单元落在槽 3，覆写 R2 的单元。
    #[test]
    fn anchor_baseline_reuses_abandoned_exclusive_unit_and_recovery_lands_on_abandoned_root() {
        let (model, root_one) = anchor_model(Arm::Baseline);
        assert_eq!(root_one.references.keys().copied().collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(model.current.references.keys().copied().collect::<Vec<_>>(), vec![2, 3], "R2 引用 {{2,3}}");
        let anchor = run_anchor(Arm::Baseline);
        assert_eq!(anchor.table_slot_chosen, 3);
        assert!(anchor.violations > 0);
        assert_eq!(anchor.chosen_after_table_only, Some(2), "只有实例表单元持久时恢复选 R2（txg 2 最新、可读）");
    }

    /// 判据 1：影子账锚点——{0,1,2,3} 全被环里的根引用，分配 4，零违例。
    #[test]
    fn anchor_shadow_accounts_allocates_slot_four_with_no_violation() {
        let anchor = run_anchor(Arm::ShadowAccounts);
        assert_eq!(anchor.table_slot_chosen, 4);
        assert_eq!(anchor.violations, 0);
    }

    /// 判据 1：墓碑锚点——先墓碑 R2 的槽，分配器同基线落槽 3，只有单元与墓碑持久时恢复选 R1。
    #[test]
    fn anchor_tombstones_make_recovery_choose_target_root() {
        let anchor = run_anchor(Arm::Tombstones);
        assert_eq!(anchor.table_slot_chosen, 3, "墓碑臂的分配器同基线");
        assert_eq!(anchor.violations, 0);
        assert_eq!(anchor.chosen_after_table_only, Some(1));
    }

    /// 前向时间线遵守 I-7.4：回退候选集里的根引用的单元不被重新分配——基线也如此，否则回退目标自己就不完整。
    #[test]
    fn forward_timeline_never_reallocates_candidate_roots_units() {
        let mut model = Model::mkfs(Arm::Baseline);
        let mut roots = vec![model.current.clone()];
        for _ in 0..NORMAL_PUBLISHES_BEFORE_ROLLBACK {
            roots.push(model.publish(DIRTY_UNITS_PER_PUBLISH, None));
        }
        for root in &roots {
            for (unit, writer) in &root.references {
                assert_eq!(model.disk.units[*unit], UnitContent::Data { writer_root: *writer }, "根 {} 引用的单元 {unit} 被覆写了", root.identity);
            }
        }
    }

    #[test]
    fn ring_slot_rotates_across_regions_then_within() {
        assert_eq!(ring_slot_for(0), 0);
        assert_eq!(ring_slot_for(1), 4);
        assert_eq!(ring_slot_for(2), 8);
        assert_eq!(ring_slot_for(3), 1);
        assert_eq!(ring_slot_for(12), 0, "12 次发布转满一圈");
    }

    /// 判据 2：崩溃状态数等于闭式；回退发布是「3 个单元写 | 记录段空 | 根 FUA」⇒ 1 + 7 + 1 = 9。
    #[test]
    fn crash_state_count_matches_closed_form() {
        let outcome = run_case_one(Arm::Baseline);
        assert_eq!(outcome.states, outcome.closed_form);
        assert_eq!(outcome.states, 5, "回退发布：实例表单元 + 1 个 COW 单元一段（3 个非空子集）、根 FUA 一段（1）、加上什么都没持久（1）");
        let tombstones = run_case_one(Arm::Tombstones);
        assert_eq!(tombstones.states, 5 + 2, "两个被抛弃的根（txg 4、5）各一道墓碑 FUA，各自成段");
        assert_eq!(tombstones.tombstone_writes, 2);
    }

    /// 判据 2：恢复确定性——同一个盘跑两遍选同一个根。
    #[test]
    fn recovery_is_deterministic() {
        let mut model = Model::mkfs(Arm::Baseline);
        for _ in 0..4 {
            model.publish(DIRTY_UNITS_PER_PUBLISH, None);
        }
        let first = recover(&model.disk, &model.tables);
        let second = recover(&model.disk, &model.tables);
        assert_eq!(first, second);
        assert_eq!(first.chosen.map(|root| root.txg), Some(4));
    }

    /// 判据 2：每个根的引用 ⊆ 已写过的槽，账里已分配与空闲不交。
    #[test]
    fn references_are_always_written_units() {
        let mut model = Model::mkfs(Arm::ShadowAccounts);
        for _ in 0..6 {
            let root = model.publish(DIRTY_UNITS_PER_PUBLISH, None);
            for (unit, writer) in &root.references {
                assert_eq!(model.disk.units[*unit], UnitContent::Data { writer_root: *writer });
            }
            let account = root.account();
            let free: BTreeSet<usize> = (0..UNIT_SLOTS).filter(|slot| !model.forbidden_slots().contains(slot)).collect();
            assert!(account.is_disjoint(&free));
        }
    }

    /// 判据 3：基线在场景一与场景二都有违例——模型看得见 C314。
    #[test]
    fn baseline_shows_violations_in_both_cases() {
        assert!(run_case_one(Arm::Baseline).violations > 0);
        let case_two = run_case_two(Arm::Baseline, false, 0);
        assert_eq!(case_two.violations, 1);
        assert!(case_two.chosen_is_abandoned, "回退实例的根读不出之后恢复落在被抛弃的根上");
        assert_eq!(case_two.faults_injected, 1, "无暖机：回退实例只有一个根，一个故障就够");
    }

    /// 判据 4：两条修法在两格上零违例。
    #[test]
    fn both_fixes_are_clean_in_both_cases() {
        for arm in [Arm::ShadowAccounts, Arm::Tombstones] {
            let case_one = run_case_one(arm);
            assert_eq!(case_one.violations, 0, "{:?} 场景一：{:?}", arm, case_one.first_violation);
            for warm_up in [false, true] {
                for follow_up in 0..=FOLLOW_UP_PUBLISHES_MAXIMUM {
                    assert_eq!(run_case_two(arm, warm_up, follow_up).violations, 0, "{:?} 场景二 warm_up={warm_up} P={follow_up}", arm);
                }
            }
        }
    }

    /// 判据 5：影子账的隔离量随后续发布降到 0，且不超过环容量。
    #[test]
    fn shadow_quarantine_drains_within_ring_capacity() {
        let first = run_case_two(Arm::ShadowAccounts, false, 0);
        assert!(first.quarantined_by_abandoned_before_fault > 0, "回退刚完成时被抛弃的根还在环里，隔离量 > 0");
        let drained = (0..=FOLLOW_UP_PUBLISHES_MAXIMUM).find(|follow_up| run_case_two(Arm::ShadowAccounts, false, *follow_up).quarantined_by_abandoned_before_fault == 0);
        assert_eq!(drained, Some(11), "R5 住槽 9，txg 17 才覆写它：回退根 txg 6 之后第 11 次发布");
    }

    /// 判据 5：暖机让回退实例多三个根，场景二要多注入三个故障才落到被抛弃的根上。
    #[test]
    fn warm_up_raises_the_number_of_faults_needed() {
        let without = run_case_two(Arm::Baseline, false, 0);
        let with = run_case_two(Arm::Baseline, true, 0);
        assert_eq!(without.faults_injected, 1);
        assert_eq!(with.faults_injected, 1 + WARM_UP_EMPTY_PUBLISHES_MAXIMUM as usize);
    }

    /// 实例表判无效的根被跳过：给 txg 5 的根所属实例写一行 (实例, 3, 0)，恢复退到 txg 3。
    #[test]
    fn recovery_skips_roots_the_instance_table_invalidates() {
        let mut model = Model::mkfs(Arm::Baseline);
        for _ in 0..5 {
            model.publish(DIRTY_UNITS_PER_PUBLISH, None);
        }
        model.tables[0].rows.push((1, 3, 0));
        let outcome = recover(&model.disk, &model.tables);
        assert_eq!(outcome.chosen.map(|root| root.txg), Some(3));
    }

    /// 回退发布持久之后、无故障时，恢复选回退实例的根（新根 txg = 环里最大 + 1 压得过被抛弃的根）。
    #[test]
    fn completed_rollback_wins_recovery_over_abandoned_roots() {
        for arm in [Arm::Baseline, Arm::ShadowAccounts, Arm::Tombstones] {
            let mut model = Model::mkfs(arm);
            let mut roots = vec![model.current.clone()];
            for _ in 0..NORMAL_PUBLISHES_BEFORE_ROLLBACK {
                roots.push(model.publish(DIRTY_UNITS_PER_PUBLISH, None));
            }
            let rollback_root = model.rollback_to(&roots[ROLLBACK_TARGET_TXG as usize].clone(), DIRTY_UNITS_PER_PUBLISH);
            assert_eq!(rollback_root.txg, NORMAL_PUBLISHES_BEFORE_ROLLBACK + 1, "新根 txg = 环里最大 + 1");
            let outcome = recover(&model.disk, &model.tables);
            assert_eq!(outcome.chosen.as_ref().map(|root| root.identity), Some(rollback_root.identity), "{:?}", arm);
            assert!(!outcome.is_violation());
        }
    }

    /// 墓碑臂：被抛弃的根槽在回退发布前被墓碑覆写，恢复读不到它们。
    #[test]
    fn tombstones_cover_every_abandoned_root() {
        let mut model = Model::mkfs(Arm::Tombstones);
        let mut roots = vec![model.current.clone()];
        for _ in 0..NORMAL_PUBLISHES_BEFORE_ROLLBACK {
            roots.push(model.publish(DIRTY_UNITS_PER_PUBLISH, None));
        }
        model.rollback_to(&roots[ROLLBACK_TARGET_TXG as usize].clone(), DIRTY_UNITS_PER_PUBLISH);
        assert_eq!(model.tombstone_writes, 2);
        for txg in [4u64, 5] {
            assert_eq!(model.disk.ring[ring_slot_for(txg)], RingSlot::Tombstone);
        }
    }
}
