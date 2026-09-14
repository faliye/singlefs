//! C322（取号那一步的屏障怎么放没有条款） 第二轮攻方腿（Opus）的模型 K，只用 std，不依赖仓里的 crate。
//! A 段（K2 / K3 / K4）：取号 → 本实例第一次发布 → 暖机，按层 0 同一条切段规则（屏障关段、FUA 写关自己那一段、
//!   前面的段全持久 + 当前段任意真子集 + 全持久那一个）枚举崩溃状态；逐状态算下一次取号撞不撞号、四种 I-7.7 候选判什么、
//!   回退途中崩溃时超级块带不带新号、回退留下的孤儿与上一个实例的孤儿在下一次挂载时被已发布谓词判成什么。
//! B 段（K5）：世代号规则 × 「全部超级块」的读法 × 臂，枚举第一个事务之后连着三次挂载的历史。
//! C 段（K4）：点名镜像逐个过四种检查。
//! 一次写抽象成 (种类, 盘, 实例代号, txg)；超级块 481 字节住一个 512 字节扇区，撕裂只能是旧或新（第一轮模型 A 核过）。

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriteKind {
    SuperblockSlot,
    Unit,
    JournalRecord,
    RootRecordForceUnitAccess,
}

#[derive(Clone, Copy, Debug)]
struct PlannedWrite {
    kind: WriteKind,
    disk: usize,
    instance: u32,
    /// 根记录的 checkpoint_txg、单元的诞生代号；超级块与记录填 0（不用）。
    txg: u64,
    label: &'static str,
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Write(PlannedWrite),
    Barrier,
}

/// 取号那一步的臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    /// 甲：取号两写之后不加屏障。
    Jia,
    /// 甲″：两盘写完 → 一道完成了的屏障。
    JiaDoublePrime,
    /// 乙′：盘 0 → 屏障 → 盘 1 → 屏障。
    YiPrime,
    /// K2 的「改取号」：取号不先写超级块，新号只随本实例第一次发布末尾那两次超级块写落盘。
    NumberOnlyInOwnPublish,
}

impl Arm {
    const EVERY_ARM: [Arm; 4] = [
        Arm::Jia,
        Arm::JiaDoublePrime,
        Arm::YiPrime,
        Arm::NumberOnlyInOwnPublish,
    ];
    const fn name(self) -> &'static str {
        match self {
            Arm::Jia => "jia",
            Arm::JiaDoublePrime => "jia_double_prime",
            Arm::YiPrime => "yi_prime",
            Arm::NumberOnlyInOwnPublish => "number_only_in_own_publish",
        }
    }
}

/// 取号之后本实例的第一次发布是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstPublish {
    WarmUpEmptyPublish,
    RecoveryRows,
    InstanceSwitch,
    AdministratorRollback,
    /// K3 候选：写行那次发布之前先推一次抬 F 的空发布（D16（发布语义） 已定项 1「准入不够时先推空发布抬 F」没说恢复那次挂载除外）。
    EmptyPublishBeforeRecoveryRows,
}

impl FirstPublish {
    const EVERY_KIND: [FirstPublish; 5] = [
        FirstPublish::WarmUpEmptyPublish,
        FirstPublish::RecoveryRows,
        FirstPublish::InstanceSwitch,
        FirstPublish::AdministratorRollback,
        FirstPublish::EmptyPublishBeforeRecoveryRows,
    ];
    const fn name(self) -> &'static str {
        match self {
            FirstPublish::WarmUpEmptyPublish => "first_mount_after_mkfs",
            FirstPublish::RecoveryRows => "mount_after_unclean_end",
            FirstPublish::InstanceSwitch => "instance_switch",
            FirstPublish::AdministratorRollback => "administrator_rollback",
            FirstPublish::EmptyPublishBeforeRecoveryRows => "empty_publish_before_recovery_rows",
        }
    }
    /// 取号之前的实例代号：mkfs 之后是 0；其余格按「第一个事务写完之后实例 1 非干净结束」取 1。
    const fn instance_before_acquisition(self) -> u32 {
        match self {
            FirstPublish::WarmUpEmptyPublish => 0,
            FirstPublish::RecoveryRows
            | FirstPublish::InstanceSwitch
            | FirstPublish::AdministratorRollback
            | FirstPublish::EmptyPublishBeforeRecoveryRows => 1,
        }
    }
    /// 取号之前根环里最大的 txg：mkfs 之后 0；第一个事务之后 3（first-txn-layout.md 零的 t10）。
    const fn highest_root_txg_before(self) -> u64 {
        match self {
            FirstPublish::WarmUpEmptyPublish => 0,
            FirstPublish::RecoveryRows
            | FirstPublish::InstanceSwitch
            | FirstPublish::AdministratorRollback
            | FirstPublish::EmptyPublishBeforeRecoveryRows => 3,
        }
    }
}

fn write_step(kind: WriteKind, disk: usize, instance: u32, txg: u64, label: &'static str) -> Step {
    Step::Write(PlannedWrite {
        kind,
        disk,
        instance,
        txg,
        label,
    })
}

fn acquisition_steps(arm: Arm, instance: u32) -> Vec<Step> {
    let disk_zero = write_step(WriteKind::SuperblockSlot, 0, instance, 0, "acquire_superblock@disk0");
    let disk_one = write_step(WriteKind::SuperblockSlot, 1, instance, 0, "acquire_superblock@disk1");
    match arm {
        Arm::Jia => vec![disk_zero, disk_one],
        Arm::JiaDoublePrime => vec![disk_zero, disk_one, Step::Barrier],
        Arm::YiPrime => vec![disk_zero, Step::Barrier, disk_one, Step::Barrier],
        Arm::NumberOnlyInOwnPublish => Vec::new(),
    }
}

/// 一次空发布（D16（发布语义） 已定项 7 的顺序，单元集为空）：屏障 [记录 × 2 盘] 屏障 [根 FUA] [超级块 × 2 盘]。
fn empty_publish_steps(instance: u32, txg: u64, root_disk: usize, labels: [&'static str; 5]) -> Vec<Step> {
    vec![
        Step::Barrier,
        write_step(WriteKind::JournalRecord, 0, instance, 0, labels[0]),
        write_step(WriteKind::JournalRecord, 1, instance, 0, labels[1]),
        Step::Barrier,
        write_step(WriteKind::RootRecordForceUnitAccess, root_disk, instance, txg, labels[2]),
        write_step(WriteKind::SuperblockSlot, 0, instance, 0, labels[3]),
        write_step(WriteKind::SuperblockSlot, 1, instance, 0, labels[4]),
    ]
}

/// 写行那次发布（first-txn-layout.md 第 392 行的预想段序列）：[实例表单元 + 另一个 COW 单元] 屏障 [记录] 屏障 [根 FUA] [超级块]。
/// 根的标签恒为 "rows_root"：它挂的实例表里有这次写的行。
fn rows_publish_steps(instance: u32, txg: u64, root_disk: usize, second_unit: [&'static str; 2]) -> Vec<Step> {
    vec![
        write_step(WriteKind::Unit, 0, instance, txg, "rows_unit@disk0"),
        write_step(WriteKind::Unit, 1, instance, txg, "rows_unit@disk1"),
        write_step(WriteKind::Unit, 0, instance, txg, second_unit[0]),
        write_step(WriteKind::Unit, 1, instance, txg, second_unit[1]),
        Step::Barrier,
        write_step(WriteKind::JournalRecord, 0, instance, 0, "rows_record@disk0"),
        write_step(WriteKind::JournalRecord, 1, instance, 0, "rows_record@disk1"),
        Step::Barrier,
        write_step(WriteKind::RootRecordForceUnitAccess, root_disk, instance, txg, "rows_root"),
        write_step(WriteKind::SuperblockSlot, 0, instance, 0, "rows_superblock@disk0"),
        write_step(WriteKind::SuperblockSlot, 1, instance, 0, "rows_superblock@disk1"),
    ]
}

const WARM_UP_FIRST: [&str; 5] = [
    "warm_up_1_record@disk0",
    "warm_up_1_record@disk1",
    "warm_up_1_root",
    "warm_up_1_superblock@disk0",
    "warm_up_1_superblock@disk1",
];
const WARM_UP_SECOND: [&str; 5] = [
    "warm_up_2_record@disk0",
    "warm_up_2_record@disk1",
    "warm_up_2_root",
    "warm_up_2_superblock@disk0",
    "warm_up_2_superblock@disk1",
];
const EMPTY_BEFORE_ROWS: [&str; 5] = [
    "f_raise_record@disk0",
    "f_raise_record@disk1",
    "f_raise_root",
    "f_raise_superblock@disk0",
    "f_raise_superblock@disk1",
];

/// 根落哪块盘：区域 = txg mod 3，区域归属 0 / 1 / 0（D22（单元原子性怎么合成） 已定项 16 第 4 句）。
const fn root_disk_for(txg: u64) -> usize {
    if txg % 3 == 1 {
        1
    } else {
        0
    }
}

/// 取号之后本实例的写：第一次发布 + 暖机（新实例的根推到两块盘为止，D16（发布语义） 已定项 8）。
fn first_publish_steps(first_publish: FirstPublish, instance: u32) -> Vec<Step> {
    let first_txg = first_publish.highest_root_txg_before() + 1;
    let second_txg = first_txg + 1;
    let first_disk = root_disk_for(first_txg);
    let second_disk = root_disk_for(second_txg);
    let mut steps = match first_publish {
        FirstPublish::WarmUpEmptyPublish => empty_publish_steps(instance, first_txg, first_disk, WARM_UP_FIRST),
        FirstPublish::RecoveryRows => rows_publish_steps(
            instance,
            first_txg,
            first_disk,
            ["fixed_point_node@disk0", "fixed_point_node@disk1"],
        ),
        FirstPublish::InstanceSwitch => rows_publish_steps(
            instance,
            first_txg,
            first_disk,
            ["reissued_data_unit@disk0", "reissued_data_unit@disk1"],
        ),
        FirstPublish::AdministratorRollback => rows_publish_steps(
            instance,
            first_txg,
            first_disk,
            ["rollback_cow_unit@disk0", "rollback_cow_unit@disk1"],
        ),
        FirstPublish::EmptyPublishBeforeRecoveryRows => {
            empty_publish_steps(instance, first_txg, first_disk, EMPTY_BEFORE_ROWS)
        }
    };
    let second = match first_publish {
        FirstPublish::EmptyPublishBeforeRecoveryRows => rows_publish_steps(
            instance,
            second_txg,
            second_disk,
            ["fixed_point_node@disk0", "fixed_point_node@disk1"],
        ),
        FirstPublish::WarmUpEmptyPublish
        | FirstPublish::RecoveryRows
        | FirstPublish::InstanceSwitch
        | FirstPublish::AdministratorRollback => {
            empty_publish_steps(instance, second_txg, second_disk, WARM_UP_SECOND)
        }
    };
    steps.extend(second);
    steps
}

/// 切段：屏障关掉它之前那一段（段空时不开新段），FUA 写关掉自己所在的那一段（crash.rs 的 writes_and_segments 同一条规则）。
fn split_into_segments(steps: &[Step]) -> (Vec<PlannedWrite>, Vec<Vec<usize>>) {
    let mut writes = Vec::new();
    let mut segments = Vec::new();
    let mut current_segment = Vec::new();
    for step in steps {
        match step {
            Step::Barrier => {
                if !current_segment.is_empty() {
                    segments.push(std::mem::take(&mut current_segment));
                }
            }
            Step::Write(planned) => {
                writes.push(*planned);
                current_segment.push(writes.len() - 1);
                if planned.kind == WriteKind::RootRecordForceUnitAccess {
                    segments.push(std::mem::take(&mut current_segment));
                }
            }
        }
    }
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }
    (writes, segments)
}

/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个（crash.rs 的 enumerate_layer0_selecting 同形）。
fn for_each_crash_state(write_total: usize, segments: &[Vec<usize>], visit: &mut dyn FnMut(&[bool])) {
    let mut persisted_before = vec![false; write_total];
    for segment in segments {
        let full_mask: u64 = (1u64 << segment.len()) - 1;
        for mask in 0..full_mask {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if mask & (1u64 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            visit(&persisted);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    visit(&persisted_before);
}

/// 一个崩溃状态在盘上留下的实例代号；取号之前盘上已有的根 {0, 旧号}、记录 {旧号}、单元 {0, 旧号}。
struct StateView {
    superblock_instance_by_disk: [u32; 2],
    root_instances: BTreeSet<u32>,
    record_instances: BTreeSet<u32>,
    unit_instances: BTreeSet<u32>,
    /// 新号已持久的根：(标签, txg)。
    new_roots: Vec<(&'static str, u64)>,
    /// 新号已持久的单元：(标签, 诞生代号)。
    new_units: Vec<(&'static str, u64)>,
    persisted_labels: Vec<&'static str>,
}

fn view_state(writes: &[PlannedWrite], persisted: &[bool], instance_before: u32, new_instance: u32) -> StateView {
    let mut view = StateView {
        superblock_instance_by_disk: [instance_before; 2],
        root_instances: BTreeSet::from([0, instance_before]),
        record_instances: if instance_before == 0 {
            BTreeSet::new()
        } else {
            BTreeSet::from([instance_before])
        },
        unit_instances: BTreeSet::from([0, instance_before]),
        new_roots: Vec::new(),
        new_units: Vec::new(),
        persisted_labels: Vec::new(),
    };
    for (planned, is_persisted) in writes.iter().zip(persisted) {
        if !*is_persisted {
            continue;
        }
        view.persisted_labels.push(planned.label);
        match planned.kind {
            WriteKind::SuperblockSlot => view.superblock_instance_by_disk[planned.disk] = planned.instance,
            WriteKind::Unit => {
                view.unit_instances.insert(planned.instance);
                if planned.instance == new_instance {
                    view.new_units.push((planned.label, planned.txg));
                }
            }
            WriteKind::JournalRecord => {
                view.record_instances.insert(planned.instance);
            }
            WriteKind::RootRecordForceUnitAccess => {
                view.root_instances.insert(planned.instance);
                if planned.instance == new_instance {
                    view.new_roots.push((planned.label, planned.txg));
                }
            }
        }
    }
    view
}

impl StateView {
    fn highest_superblock(&self) -> u32 {
        self.superblock_instance_by_disk[0].max(self.superblock_instance_by_disk[1])
    }
    fn highest_root(&self) -> u32 {
        self.root_instances.iter().copied().max().unwrap_or(0)
    }
    fn superblocks_unequal(&self) -> bool {
        self.superblock_instance_by_disk[0] != self.superblock_instance_by_disk[1]
    }
    fn carried(&self) -> BTreeSet<u32> {
        let mut carried = self.root_instances.clone();
        carried.extend(&self.record_instances);
        carried.extend(&self.unit_instances);
        carried
    }
    /// 甲″ / 乙′ 共用的取号规则：max(全部超级块, 根环) + 1（两盘一致的世代号规则下「全部槽」与「择到的」同值，B 段另量）。
    fn next_acquired_instance(&self) -> u32 {
        self.highest_superblock().max(self.highest_root()) + 1
    }
    fn next_acquisition_collides(&self) -> bool {
        self.carried().contains(&self.next_acquired_instance())
    }
    fn superblocks_at_least_root_ring(&self) -> bool {
        let highest_root = self.highest_root();
        self.superblock_instance_by_disk.iter().all(|instance| *instance >= highest_root)
    }
    /// 今天的 I-7.7 字面：各超级块相等，且 ≥ 根环里任一根。
    fn literal_holds(&self) -> bool {
        !self.superblocks_unequal() && self.superblocks_at_least_root_ring()
    }
    /// 第一轮甲写的改写：相等，或不等时根环与 journal 里没有带最大那个超级块实例代号的根或记录；保留「≥ 根环」那半句。
    fn jia_rewrite_holds(&self) -> bool {
        let highest = self.highest_superblock();
        let unequal_branch = !self.root_instances.contains(&highest) && !self.record_instances.contains(&highest);
        (!self.superblocks_unequal() || unequal_branch) && self.superblocks_at_least_root_ring()
    }
    /// Q7 ①：盘上每个根、记录、单元写序的实例代号 ≤ 各盘超级块实例代号的最大者。
    fn q7_one_holds(&self) -> bool {
        let highest = self.highest_superblock();
        self.carried().iter().all(|instance| *instance <= highest)
    }
    /// Q7 ②：各盘超级块实例代号不等时，较大者不出现在任何根、记录、单元写序里。
    fn q7_two_holds(&self) -> bool {
        !self.superblocks_unequal() || !self.carried().contains(&self.highest_superblock())
    }
    /// 四种检查，次序：字面、甲的改写、Q7 ①、Q7 ②。
    fn check_verdicts(&self) -> [bool; 4] {
        [self.literal_holds(), self.jia_rewrite_holds(), self.q7_one_holds(), self.q7_two_holds()]
    }
}

/// D18（块里携带什么信息） 已定项 11 的全局已发布谓词（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2），按条文四支（第一轮模型 B 同一段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishedVerdict {
    Published,
    Unpublished,
    Corrupt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitClassForPredicate {
    DataUnitCodeOne,
    NodeOrPackedCodeTwoOrThree,
}

#[derive(Clone, Copy, Debug)]
struct InstanceRow {
    instance: u32,
    published_txg: u64,
    highest_applied_transaction: u64,
}

fn published_predicate(
    instance: u32,
    transaction: u64,
    birth_txg: u64,
    class: UnitClassForPredicate,
    rows: &[InstanceRow],
    mount_root_instance: u32,
    mount_root_txg: u64,
) -> PublishedVerdict {
    if instance > mount_root_instance {
        return PublishedVerdict::Corrupt;
    }
    if instance == mount_root_instance {
        return if birth_txg <= mount_root_txg {
            PublishedVerdict::Published
        } else {
            PublishedVerdict::Unpublished
        };
    }
    let Some(row) = rows.iter().find(|row| row.instance == instance) else {
        return PublishedVerdict::Published;
    };
    let published = match class {
        UnitClassForPredicate::DataUnitCodeOne => {
            birth_txg <= row.published_txg || transaction <= row.highest_applied_transaction
        }
        UnitClassForPredicate::NodeOrPackedCodeTwoOrThree => birth_txg <= row.published_txg,
    };
    if published {
        PublishedVerdict::Published
    } else {
        PublishedVerdict::Unpublished
    }
}

/// 两类孤儿在「下一次挂载（普通恢复，不回退）发布了它的第一个根」之后被判成什么，返回 (新号孤儿有判已发布的, 旧号孤儿有判已发布的)。
/// 新号孤儿 = 本实例写行那次发布的单元而那次的根没落；旧号孤儿 = 旧实例非干净结束时在飞 checkpoint 的单元（写序 (旧号, 1)、诞生 txg = 旧根环最大 + 1）。
/// 下一次挂载按 D18（块里携带什么信息） 已定项 11 写行：给 [所选根的实例, 新实例) 里还没有行的实例写 (i, 所选根 txg, 0)，第一个根 txg = 所选根 + 1。
/// 回退那条路径的 R_old 取 (旧号, 1)，回退行 (旧号, 1, 0)。
fn next_mount_orphan_verdicts(view: &StateView, first_publish: FirstPublish, new_instance: u32) -> (bool, bool) {
    let instance_before = first_publish.instance_before_acquisition();
    if instance_before == 0 {
        return (false, false);
    }
    let highest_txg_before = first_publish.highest_root_txg_before();
    let rows_published_txg = match first_publish {
        FirstPublish::AdministratorRollback => 1,
        FirstPublish::WarmUpEmptyPublish
        | FirstPublish::RecoveryRows
        | FirstPublish::InstanceSwitch
        | FirstPublish::EmptyPublishBeforeRecoveryRows => highest_txg_before,
    };
    let (selected_instance, selected_txg) = view
        .new_roots
        .iter()
        .map(|(_, txg)| (new_instance, *txg))
        .max_by_key(|(_, txg)| *txg)
        .unwrap_or((instance_before, highest_txg_before));
    let rows_root_durable = view.new_roots.iter().any(|(label, _)| *label == "rows_root");
    let mut rows = Vec::new();
    if rows_root_durable {
        rows.push(InstanceRow {
            instance: instance_before,
            published_txg: rows_published_txg,
            highest_applied_transaction: 0,
        });
    }
    let next = view.next_acquired_instance();
    for instance in selected_instance..next {
        if !rows.iter().any(|row| row.instance == instance) {
            rows.push(InstanceRow {
                instance,
                published_txg: selected_txg,
                highest_applied_transaction: 0,
            });
        }
    }
    let mount_root_txg = selected_txg + 1;
    let classes = [
        (1u64, UnitClassForPredicate::DataUnitCodeOne),
        (0u64, UnitClassForPredicate::NodeOrPackedCodeTwoOrThree),
    ];
    let judged_published = |instance: u32, birth_txg: u64| {
        classes.iter().any(|(transaction, class)| {
            published_predicate(instance, *transaction, birth_txg, *class, &rows, next, mount_root_txg)
                == PublishedVerdict::Published
        })
    };
    let new_orphans_published = !rows_root_durable
        && view
            .new_units
            .iter()
            .any(|(_, birth_txg)| judged_published(new_instance, *birth_txg));
    let previous_orphans_published = judged_published(instance_before, highest_txg_before + 1);
    (new_orphans_published, previous_orphans_published)
}

#[derive(Default)]
struct PathTally {
    states: u64,
    superblocks_unequal: u64,
    collisions: u64,
    first_collision: Option<Vec<&'static str>>,
    /// 下标：[字面, 甲的改写, Q7 ①, Q7 ②] × [红, 红且撞号, 红且不撞号, 撞号而绿]。
    checks: [[u64; 4]; 4],
    new_number_in_superblock_without_new_root: u64,
    first_new_number_without_new_root: Option<Vec<&'static str>>,
    new_orphans_published: u64,
    previous_orphans_published: u64,
    first_previous_orphan_published: Option<Vec<&'static str>>,
}

/// 屏障道数：连续几道之间没有写的记成一道（与 SharedStream 的记法相同）。
fn barrier_count(steps: &[Step]) -> usize {
    let mut total = 0;
    let mut previous_was_barrier = false;
    for step in steps {
        match step {
            Step::Barrier => {
                if !previous_was_barrier {
                    total += 1;
                }
                previous_was_barrier = true;
            }
            Step::Write(_) => previous_was_barrier = false,
        }
    }
    total
}

fn tally_path(arm: Arm, first_publish: FirstPublish) -> (PathTally, Vec<usize>, usize) {
    let instance_before = first_publish.instance_before_acquisition();
    let new_instance = instance_before + 1;
    let mut steps = acquisition_steps(arm, new_instance);
    steps.extend(first_publish_steps(first_publish, new_instance));
    let (writes, segments) = split_into_segments(&steps);
    let mut tally = PathTally::default();
    for_each_crash_state(writes.len(), &segments, &mut |persisted| {
        let view = view_state(&writes, persisted, instance_before, new_instance);
        tally.states += 1;
        let collides = view.next_acquisition_collides();
        if view.superblocks_unequal() {
            tally.superblocks_unequal += 1;
        }
        if collides {
            tally.collisions += 1;
            tally.first_collision.get_or_insert_with(|| view.persisted_labels.clone());
        }
        for (check_position, holds) in view.check_verdicts().iter().enumerate() {
            let counts = &mut tally.checks[check_position];
            if !holds {
                counts[0] += 1;
                if collides {
                    counts[1] += 1;
                } else {
                    counts[2] += 1;
                }
            } else if collides {
                counts[3] += 1;
            }
        }
        if view.superblock_instance_by_disk.contains(&new_instance) && view.new_roots.is_empty() {
            tally.new_number_in_superblock_without_new_root += 1;
            tally
                .first_new_number_without_new_root
                .get_or_insert_with(|| view.persisted_labels.clone());
        }
        let (new_orphans, previous_orphans) = next_mount_orphan_verdicts(&view, first_publish, new_instance);
        if new_orphans {
            tally.new_orphans_published += 1;
        }
        if previous_orphans {
            tally.previous_orphans_published += 1;
            tally
                .first_previous_orphan_published
                .get_or_insert_with(|| view.persisted_labels.clone());
        }
    });
    (tally, segments.iter().map(Vec::len).collect(), barrier_count(&steps))
}

fn report_paths() {
    for first_publish in FirstPublish::EVERY_KIND {
        for arm in Arm::EVERY_ARM {
            let (tally, segment_sizes, barriers) = tally_path(arm, first_publish);
            let check_text = |position: usize| {
                let counts = tally.checks[position];
                format!("{}/{}/{}/{}", counts[0], counts[1], counts[2], counts[3])
            };
            println!(
                "PATH path={} arm={} segments={segment_sizes:?} barriers={barriers} states={} superblocks_unequal={} collisions={} new_number_in_superblock_without_new_root={} new_orphans_published_at_next_mount={} previous_orphans_published_at_next_mount={} CHECKS(red/red_on_collision/red_without_collision/green_on_collision) literal={} jia_rewrite={} q7_one={} q7_two={}",
                first_publish.name(),
                arm.name(),
                tally.states,
                tally.superblocks_unequal,
                tally.collisions,
                tally.new_number_in_superblock_without_new_root,
                tally.new_orphans_published,
                tally.previous_orphans_published,
                check_text(0),
                check_text(1),
                check_text(2),
                check_text(3)
            );
            let examples = [
                ("first_collision", &tally.first_collision),
                ("first_new_number_in_superblock_without_new_root", &tally.first_new_number_without_new_root),
                ("first_previous_orphan_published", &tally.first_previous_orphan_published),
            ];
            for (what, example) in examples {
                if let Some(persisted) = example {
                    println!("PATH_EXAMPLE path={} arm={} {what}={persisted:?}", first_publish.name(), arm.name());
                }
            }
        }
    }
}

// ---------------- B 段（K5）：世代号规则 × 「全部超级块」的读法 × 臂 ----------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationRule {
    /// G1：每次超级块写（取号、发布末尾的轮换、回卷）都写「两盘择到的世代号的最大者 + 1」，两盘同一个值。
    PoolWideHighestPlusOne,
    /// G2：每盘各数，写「这块盘择到的世代号 + 1」（D22（单元原子性怎么合成） 已定项 16 字面「每写一次 +1」）。
    PerDiskPlusOne,
    /// 今天的实现：取号恒写 2，发布之后写 txg + 2（transaction.rs 第 49、174–178 行）；回卷今天没有代码，按取号同一个 2 算。
    TodayConstantTwoThenTxgPlusTwo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SuperblockReading {
    /// 「全部超级块」= 每盘两槽里全部自证过的槽。
    EverySelfVerifiedSlot,
    /// 「全部超级块」= 每盘择到的那一份（校验和过且世代号最大，相等取槽 0），再跨盘取最大。
    ChosenSlotPerDisk,
    /// 只看世代号跨盘择一份（C322 那条自证的变异形态）。
    SingleSlotByGeneration,
    /// 今天 recovery::choose_superblock 返回的：第一块盘择到的那一份。
    FirstDiskChosen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotContent {
    generation: u64,
    instance: u32,
}

fn chosen_position(disk_slots: &[SlotContent; 2]) -> usize {
    if disk_slots[1].generation > disk_slots[0].generation {
        1
    } else {
        0
    }
}

#[derive(Clone, Debug)]
struct HistoryState {
    slots: [[SlotContent; 2]; 2],
    root_ring: Vec<(u32, u64)>,
    carried: BTreeSet<u32>,
    newest_slot_overwrites: u32,
}

impl HistoryState {
    /// 第一个事务写完（first-txn-layout.md 零的 w6 / t11）：两盘槽 0 = 世代号 4、槽 1 = 世代号 5，都是实例 1；根环 (0,0)(1,1)(1,2)(1,3)。
    fn after_first_transaction() -> Self {
        let disk_slots = [
            SlotContent { generation: 4, instance: 1 },
            SlotContent { generation: 5, instance: 1 },
        ];
        Self {
            slots: [disk_slots, disk_slots],
            root_ring: vec![(0, 0), (1, 1), (1, 2), (1, 3)],
            carried: BTreeSet::from([0, 1]),
            newest_slot_overwrites: 0,
        }
    }
    fn chosen(&self, disk: usize) -> SlotContent {
        self.slots[disk][chosen_position(&self.slots[disk])]
    }
    fn superblock_base(&self, reading: SuperblockReading) -> u32 {
        match reading {
            SuperblockReading::EverySelfVerifiedSlot => self
                .slots
                .iter()
                .flat_map(|disk_slots| disk_slots.iter())
                .map(|slot| slot.instance)
                .max()
                .unwrap_or(0),
            SuperblockReading::ChosenSlotPerDisk => self.chosen(0).instance.max(self.chosen(1).instance),
            SuperblockReading::SingleSlotByGeneration => {
                let on_disk_zero = self.chosen(0);
                let on_disk_one = self.chosen(1);
                if on_disk_one.generation > on_disk_zero.generation {
                    on_disk_one.instance
                } else {
                    on_disk_zero.instance
                }
            }
            SuperblockReading::FirstDiskChosen => self.chosen(0).instance,
        }
    }
    fn highest_root_instance(&self) -> u32 {
        self.root_ring.iter().map(|(instance, _)| *instance).max().unwrap_or(0)
    }
    fn highest_root_txg(&self) -> u64 {
        self.root_ring.iter().map(|(_, txg)| *txg).max().unwrap_or(0)
    }
    fn next_instance(&self, reading: SuperblockReading) -> u32 {
        self.superblock_base(reading).max(self.highest_root_instance()) + 1
    }
    fn generation_for(&self, rule: GenerationRule, disk: usize, publish_txg: Option<u64>) -> u64 {
        match rule {
            GenerationRule::PoolWideHighestPlusOne => self.chosen(0).generation.max(self.chosen(1).generation) + 1,
            GenerationRule::PerDiskPlusOne => self.chosen(disk).generation + 1,
            GenerationRule::TodayConstantTwoThenTxgPlusTwo => publish_txg.map_or(2, |txg| txg + 2),
        }
    }
    /// 槽 = 世代号 mod 2；写到这块盘此刻择到的那一槽上就记一次「覆写最新槽」（两槽轮换在这块盘上退化成一槽）。
    fn write_superblock(&mut self, disk: usize, generation: u64, instance: u32) {
        let position = usize::try_from(generation % 2).expect("0 或 1");
        if position == chosen_position(&self.slots[disk]) {
            self.newest_slot_overwrites += 1;
        }
        self.slots[disk][position] = SlotContent { generation, instance };
    }
}

/// 一次可写挂载崩在哪：取号两写各落没落、本实例的单元 / 根 / 发布末尾的超级块轮换各落没落。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MountOutcome {
    NothingDurable,
    AcquisitionDiskZeroOnly,
    AcquisitionDiskOneOnly,
    /// 盘 1 的取号写报错、盘 0 回卷成旧代号且回卷落了（D18（块里携带什么信息） 已定项 11「先把已经写出的那几份回卷成旧代号」）。
    AcquisitionDiskZeroRolledBack,
    BothThenUnits,
    BothThenUnitsAndRoot,
    BothThenRootAndRotationDiskZero,
    BothThenRootAndRotationDiskOne,
    BothThenRootAndRotationBoth,
    /// 以下三格只有甲走得到：本实例的单元落了、取号写没落全。
    UnitsWithAcquisitionNone,
    UnitsWithAcquisitionDiskZeroOnly,
    UnitsWithAcquisitionDiskOneOnly,
}

impl MountOutcome {
    const fn acquisition_disks(self) -> [bool; 2] {
        match self {
            Self::NothingDurable | Self::UnitsWithAcquisitionNone => [false, false],
            Self::AcquisitionDiskZeroOnly | Self::AcquisitionDiskZeroRolledBack | Self::UnitsWithAcquisitionDiskZeroOnly => {
                [true, false]
            }
            Self::AcquisitionDiskOneOnly | Self::UnitsWithAcquisitionDiskOneOnly => [false, true],
            Self::BothThenUnits
            | Self::BothThenUnitsAndRoot
            | Self::BothThenRootAndRotationDiskZero
            | Self::BothThenRootAndRotationDiskOne
            | Self::BothThenRootAndRotationBoth => [true, true],
        }
    }
    const fn units_durable(self) -> bool {
        match self {
            Self::NothingDurable
            | Self::AcquisitionDiskZeroOnly
            | Self::AcquisitionDiskOneOnly
            | Self::AcquisitionDiskZeroRolledBack => false,
            Self::BothThenUnits
            | Self::BothThenUnitsAndRoot
            | Self::BothThenRootAndRotationDiskZero
            | Self::BothThenRootAndRotationDiskOne
            | Self::BothThenRootAndRotationBoth
            | Self::UnitsWithAcquisitionNone
            | Self::UnitsWithAcquisitionDiskZeroOnly
            | Self::UnitsWithAcquisitionDiskOneOnly => true,
        }
    }
    /// 根落了之后发布末尾那两次超级块写各落没落；根没落返回 None。
    const fn root_and_rotation(self) -> Option<[bool; 2]> {
        match self {
            Self::BothThenUnitsAndRoot => Some([false, false]),
            Self::BothThenRootAndRotationDiskZero => Some([true, false]),
            Self::BothThenRootAndRotationDiskOne => Some([false, true]),
            Self::BothThenRootAndRotationBoth => Some([true, true]),
            Self::NothingDurable
            | Self::AcquisitionDiskZeroOnly
            | Self::AcquisitionDiskOneOnly
            | Self::AcquisitionDiskZeroRolledBack
            | Self::BothThenUnits
            | Self::UnitsWithAcquisitionNone
            | Self::UnitsWithAcquisitionDiskZeroOnly
            | Self::UnitsWithAcquisitionDiskOneOnly => None,
        }
    }
}

const JIA_DOUBLE_PRIME_OUTCOMES: [MountOutcome; 9] = [
    MountOutcome::NothingDurable,
    MountOutcome::AcquisitionDiskZeroOnly,
    MountOutcome::AcquisitionDiskOneOnly,
    MountOutcome::AcquisitionDiskZeroRolledBack,
    MountOutcome::BothThenUnits,
    MountOutcome::BothThenUnitsAndRoot,
    MountOutcome::BothThenRootAndRotationDiskZero,
    MountOutcome::BothThenRootAndRotationDiskOne,
    MountOutcome::BothThenRootAndRotationBoth,
];
/// 乙′：盘 0 先写先刷，「只落盘 1」不可达。
const YI_PRIME_OUTCOMES: [MountOutcome; 8] = [
    MountOutcome::NothingDurable,
    MountOutcome::AcquisitionDiskZeroOnly,
    MountOutcome::AcquisitionDiskZeroRolledBack,
    MountOutcome::BothThenUnits,
    MountOutcome::BothThenUnitsAndRoot,
    MountOutcome::BothThenRootAndRotationDiskZero,
    MountOutcome::BothThenRootAndRotationDiskOne,
    MountOutcome::BothThenRootAndRotationBoth,
];
const JIA_OUTCOMES: [MountOutcome; 12] = [
    MountOutcome::NothingDurable,
    MountOutcome::AcquisitionDiskZeroOnly,
    MountOutcome::AcquisitionDiskOneOnly,
    MountOutcome::AcquisitionDiskZeroRolledBack,
    MountOutcome::BothThenUnits,
    MountOutcome::BothThenUnitsAndRoot,
    MountOutcome::BothThenRootAndRotationDiskZero,
    MountOutcome::BothThenRootAndRotationDiskOne,
    MountOutcome::BothThenRootAndRotationBoth,
    MountOutcome::UnitsWithAcquisitionNone,
    MountOutcome::UnitsWithAcquisitionDiskZeroOnly,
    MountOutcome::UnitsWithAcquisitionDiskOneOnly,
];

fn outcomes_for(arm: Arm) -> &'static [MountOutcome] {
    match arm {
        Arm::Jia => &JIA_OUTCOMES,
        Arm::JiaDoublePrime => &JIA_DOUBLE_PRIME_OUTCOMES,
        Arm::YiPrime => &YI_PRIME_OUTCOMES,
        Arm::NumberOnlyInOwnPublish => &[],
    }
}

/// 一次可写挂载：按读法取号、按规则写超级块、按结局落盘。返回 (这次取号撞上盘上已有的号, 这次的读法没看见某个自证过的槽里更大的号)。
fn apply_mount(state: &mut HistoryState, rule: GenerationRule, reading: SuperblockReading, outcome: MountOutcome) -> (bool, bool) {
    let reading_missed_a_slot =
        state.superblock_base(reading) < state.superblock_base(SuperblockReading::EverySelfVerifiedSlot);
    let instance = state.next_instance(reading);
    let collided = state.carried.contains(&instance);
    let instance_before_on_disk_zero = state.chosen(0).instance;
    let acquisition_generations = [state.generation_for(rule, 0, None), state.generation_for(rule, 1, None)];
    let acquisition_disks = outcome.acquisition_disks();
    for disk in 0..2 {
        if acquisition_disks[disk] {
            state.write_superblock(disk, acquisition_generations[disk], instance);
        }
    }
    if outcome == MountOutcome::AcquisitionDiskZeroRolledBack {
        let generation = state.generation_for(rule, 0, None);
        state.write_superblock(0, generation, instance_before_on_disk_zero);
    }
    if outcome.units_durable() {
        state.carried.insert(instance);
    }
    if let Some(rotation_disks) = outcome.root_and_rotation() {
        let txg = state.highest_root_txg() + 1;
        state.root_ring.push((instance, txg));
        let generations = [state.generation_for(rule, 0, Some(txg)), state.generation_for(rule, 1, Some(txg))];
        for disk in 0..2 {
            if rotation_disks[disk] {
                state.write_superblock(disk, generations[disk], instance);
            }
        }
    }
    (collided, reading_missed_a_slot)
}

#[derive(Default)]
struct GenerationTally {
    histories: u64,
    collision: u64,
    reading_missed_a_slot: u64,
    newest_slot_overwrite: u64,
    first_collision: Option<(Vec<MountOutcome>, [[SlotContent; 2]; 2])>,
    /// 末态上 Q7 ① 按两种读法各判一次：[择到的, 全部槽] × [红而按这次取号的读法不撞号, 撞号而绿]。
    final_q7_one: [[u64; 2]; 2],
}

fn report_generation_rules() {
    let rules = [
        GenerationRule::PoolWideHighestPlusOne,
        GenerationRule::PerDiskPlusOne,
        GenerationRule::TodayConstantTwoThenTxgPlusTwo,
    ];
    let readings = [
        SuperblockReading::EverySelfVerifiedSlot,
        SuperblockReading::ChosenSlotPerDisk,
        SuperblockReading::SingleSlotByGeneration,
        SuperblockReading::FirstDiskChosen,
    ];
    for arm in [Arm::Jia, Arm::JiaDoublePrime, Arm::YiPrime] {
        for rule in rules {
            for reading in readings {
                let outcomes = outcomes_for(arm);
                let mut tally = GenerationTally::default();
                for first in outcomes {
                    for second in outcomes {
                        for third in outcomes {
                            let history = [*first, *second, *third];
                            let mut state = HistoryState::after_first_transaction();
                            let mut collided = false;
                            let mut missed = false;
                            for outcome in history {
                                let (this_collided, this_missed) = apply_mount(&mut state, rule, reading, outcome);
                                collided |= this_collided;
                                missed |= this_missed;
                            }
                            tally.histories += 1;
                            if collided {
                                tally.collision += 1;
                                tally.first_collision.get_or_insert_with(|| (history.to_vec(), state.slots));
                            }
                            tally.reading_missed_a_slot += u64::from(missed);
                            tally.newest_slot_overwrite += u64::from(state.newest_slot_overwrites > 0);
                            let risk = state.carried.contains(&state.next_instance(reading));
                            let checker_readings = [SuperblockReading::ChosenSlotPerDisk, SuperblockReading::EverySelfVerifiedSlot];
                            for (position, checker_reading) in checker_readings.iter().enumerate() {
                                let base = state.superblock_base(*checker_reading);
                                let red = state.carried.iter().any(|instance| *instance > base);
                                tally.final_q7_one[position][0] += u64::from(red && !risk);
                                tally.final_q7_one[position][1] += u64::from(risk && !red);
                            }
                        }
                    }
                }
                println!(
                    "GENERATION arm={} rule={rule:?} reading={reading:?} histories={} collision={} reading_missed_a_slot={} newest_slot_overwrite={} final_q7_one(chosen_reading)_false_red/missed={}/{} final_q7_one(every_slot_reading)_false_red/missed={}/{} first_collision={:?}",
                    arm.name(),
                    tally.histories,
                    tally.collision,
                    tally.reading_missed_a_slot,
                    tally.newest_slot_overwrite,
                    tally.final_q7_one[0][0],
                    tally.final_q7_one[0][1],
                    tally.final_q7_one[1][0],
                    tally.final_q7_one[1][1],
                    tally.first_collision
                );
            }
        }
    }
}

// ---------------- C 段（K4）：点名镜像逐个过四种检查 ----------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObjectKind {
    Root,
    Record,
    Unit,
}

struct NamedImage {
    name: &'static str,
    /// 各盘超级块的实例代号（读法写在名字里）。
    superblock_instances: Vec<u32>,
    /// (种类, 实例代号, fsid 与本池相同)。
    objects: Vec<(ObjectKind, u32, bool)>,
}

impl NamedImage {
    fn highest_superblock(&self) -> u32 {
        self.superblock_instances.iter().copied().max().unwrap_or(0)
    }
    fn unequal(&self) -> bool {
        self.superblock_instances.iter().any(|instance| *instance != self.superblock_instances[0])
    }
    /// 根与记录按 checker 今天的读法只数 fsid 相同的（valid_roots 与 scan_journal 都先比 fsid）。
    fn home_instances(&self, kinds: &[ObjectKind]) -> Vec<u32> {
        self.objects
            .iter()
            .filter(|(kind, _, same_filesystem)| *same_filesystem && kinds.contains(kind))
            .map(|(_, instance, _)| *instance)
            .collect()
    }
    /// Q7 ① ② 数的对象：本池的根、记录、单元，再看要不要把 fsid 不同的单元也数进去（扫描方向若不先比 fsid）。
    fn counted_instances(&self, count_foreign_units: bool) -> Vec<u32> {
        self.objects
            .iter()
            .filter(|(kind, _, same_filesystem)| *same_filesystem || (count_foreign_units && *kind == ObjectKind::Unit))
            .map(|(_, instance, _)| *instance)
            .collect()
    }
    fn superblocks_at_least_root_ring(&self) -> bool {
        let highest_root = self.home_instances(&[ObjectKind::Root]).into_iter().max().unwrap_or(0);
        self.superblock_instances.iter().all(|instance| *instance >= highest_root)
    }
    fn literal(&self) -> bool {
        !self.unequal() && self.superblocks_at_least_root_ring()
    }
    fn jia_rewrite(&self) -> bool {
        let highest = self.highest_superblock();
        let carried = self.home_instances(&[ObjectKind::Root, ObjectKind::Record]);
        (!self.unequal() || !carried.contains(&highest)) && self.superblocks_at_least_root_ring()
    }
    fn q7_one(&self, count_foreign_units: bool) -> bool {
        let highest = self.highest_superblock();
        self.counted_instances(count_foreign_units).iter().all(|instance| *instance <= highest)
    }
    fn q7_two(&self, count_foreign_units: bool) -> bool {
        !self.unequal() || !self.counted_instances(count_foreign_units).contains(&self.highest_superblock())
    }
    fn next_acquisition_collides(&self) -> bool {
        let highest_root = self.home_instances(&[ObjectKind::Root]).into_iter().max().unwrap_or(0);
        let next = self.highest_superblock().max(highest_root) + 1;
        self.home_instances(&[ObjectKind::Root, ObjectKind::Record, ObjectKind::Unit]).contains(&next)
    }
}

fn image(name: &'static str, superblock_instances: &[u32], objects: &[(ObjectKind, u32, bool)]) -> NamedImage {
    NamedImage {
        name,
        superblock_instances: superblock_instances.to_vec(),
        objects: objects.to_vec(),
    }
}

fn report_named_images() {
    use ObjectKind::{Record, Root, Unit};
    let after_first_mount = [(Root, 0, true), (Unit, 0, true)];
    let after_first_transaction = [(Root, 0, true), (Root, 1, true), (Record, 1, true), (Unit, 0, true), (Unit, 1, true)];
    let with = |base: &[(ObjectKind, u32, bool)], extra: &[(ObjectKind, u32, bool)]| {
        let mut objects = base.to_vec();
        objects.extend_from_slice(extra);
        objects
    };
    let images = [
        image("jia_recovery_path_collision_rows_unit_disk0", &[1, 1], &with(&after_first_transaction, &[(Unit, 2, true)])),
        image("first_mount_acquisition_disk0_only", &[1, 0], &after_first_mount),
        image("first_mount_acquisition_disk1_only", &[0, 1], &after_first_mount),
        image("second_mount_acquisition_disk0_only", &[2, 1], &after_first_transaction),
        image("second_mount_acquisition_disk1_only", &[1, 2], &after_first_transaction),
        image("rollback_in_flight_superblocks_only_jia_double_prime", &[2, 2], &after_first_transaction),
        image("rollback_in_flight_rows_unit_disk0_jia_double_prime", &[2, 2], &with(&after_first_transaction, &[(Unit, 2, true)])),
        image("rollback_number_only_in_own_publish_rows_unit_disk0", &[1, 1], &with(&after_first_transaction, &[(Unit, 2, true)])),
        image("foreign_disk1_same_instance_1", &[1, 1], &with(&after_first_transaction, &[(Root, 1, false), (Record, 1, false), (Unit, 1, false)])),
        image("foreign_disk1_higher_instance_5", &[1, 5], &with(&after_first_transaction, &[(Root, 5, false), (Record, 5, false), (Unit, 5, false)])),
        image(
            "foreign_disk1_lower_instance_1_home_3",
            &[3, 1],
            &with(&after_first_transaction, &[(Root, 3, true), (Record, 3, true), (Unit, 3, true), (Root, 1, false), (Unit, 1, false)]),
        ),
        image("remkfs_residue_old_filesystem_unit_instance_57", &[1, 1], &with(&after_first_transaction, &[(Unit, 57, false)])),
        image("roll_back_written_disk0_chosen_reading", &[1, 1], &after_first_transaction),
        image("roll_back_written_disk0_every_slot_reading", &[2, 1], &after_first_transaction),
        image("roll_back_written_disk0_disk1_landed_despite_error_chosen_reading", &[1, 2], &after_first_transaction),
        image("three_disks_disk2_never_opened_by_this_mount", &[2, 2, 1], &with(&after_first_transaction, &[(Root, 2, true), (Record, 2, true), (Unit, 2, true)])),
        image("today_code_hidden_acquisition_units_durable_chosen_reading", &[1, 1], &with(&after_first_transaction, &[(Unit, 2, true)])),
        image("today_code_hidden_acquisition_units_durable_every_slot_reading", &[2, 2], &with(&after_first_transaction, &[(Unit, 2, true)])),
    ];
    for named in &images {
        println!(
            "IMAGE name={} superblocks={:?} next_acquisition_collides={} literal={} jia_rewrite={} q7_one(fsid_filtered)={} q7_one(counting_foreign_units)={} q7_two(fsid_filtered)={} q7_two(counting_foreign_units)={}",
            named.name,
            named.superblock_instances,
            named.next_acquisition_collides(),
            named.literal(),
            named.jia_rewrite(),
            named.q7_one(false),
            named.q7_one(true),
            named.q7_two(false),
            named.q7_two(true)
        );
    }
}

fn main() {
    report_paths();
    report_generation_rules();
    report_named_images();
}
