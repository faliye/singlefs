//! C322（取号那一步的屏障怎么放没有条款） 第一轮反推腿的模型 B，只用 std。
//!
//! 问：取号那几次超级块写之后，本实例的第一次发布先写什么——
//!   · mkfs 之后第一次可写挂载：暖机空发布，开头就是一道屏障（layout/01-first-txn.md 八「空发布」一行）；
//!   · 非干净结束之后的恢复、实例切换、管理员回退：先写实例表单元与 COW 单元，之后才是第一道屏障
//!     （D18（块里携带什么信息） 已定项 11「行……只在恢复与回退时写、与第一个新根同一次发布」；
//!      layout/01-first-txn.md 八「实例切换 / 管理员回退」一行的预想段序列 [实例表单元 + COW 单元] 屏障 [记录] ……）。
//! 三条臂下按层 0 同一条切段规则（屏障关段、FUA 写关自己那一段、段内任意整写子集、前面的段全持久）枚举崩溃状态，
//! 逐状态问 J2（下一次取号 = max(全部超级块, 根环) + 1 撞不撞上盘上单元写序 / journal 记录 / 根已带着的实例代号）
//! 与 J3（甲改过的 I-7.7 在这个状态上判成立还是违例）。
//! 臂：甲 = 取号两次写之后不加屏障；乙 = 盘 0 → 屏障 → 盘 1 → 屏障；甲″（本腿加的对照）= 两盘写完 → 一道屏障。
//! 一次写只抽象成 (种类, 盘, 实例代号)；超级块 481 字节住一个 512 字节扇区里，撕裂只能是旧或新（模型 A 核过），不另建撕裂态。

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
    label: &'static str,
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Write(PlannedWrite),
    Barrier,
}

/// 取号那两次超级块写之后怎么收段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    /// 甲：不加屏障，靠之后第一道屏障收段。
    NoBarrierAfterAcquisition,
    /// 乙：盘 0 → 屏障 → 盘 1 → 屏障。
    SerialDiskZeroThenDiskOne,
    /// 甲″：两盘写完 → 一道屏障（不串行）。
    OneBarrierAfterBothSuperblocks,
}

impl Arm {
    const EVERY_ARM: [Arm; 3] = [
        Arm::NoBarrierAfterAcquisition,
        Arm::SerialDiskZeroThenDiskOne,
        Arm::OneBarrierAfterBothSuperblocks,
    ];
    const fn name(self) -> &'static str {
        match self {
            Arm::NoBarrierAfterAcquisition => "jia(no_barrier)",
            Arm::SerialDiskZeroThenDiskOne => "yi(serial_disk0_barrier_disk1_barrier)",
            Arm::OneBarrierAfterBothSuperblocks => "jia_double_prime(both_then_one_barrier)",
        }
    }
}

/// 取号之后本实例的第一次发布是哪一种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstPublish {
    WarmUpEmptyPublish,
    RecoveryRowPublish,
    InstanceSwitchPublish,
    AdministratorRollbackPublish,
}

impl FirstPublish {
    const EVERY_KIND: [FirstPublish; 4] = [
        FirstPublish::WarmUpEmptyPublish,
        FirstPublish::RecoveryRowPublish,
        FirstPublish::InstanceSwitchPublish,
        FirstPublish::AdministratorRollbackPublish,
    ];
    const fn name(self) -> &'static str {
        match self {
            FirstPublish::WarmUpEmptyPublish => "first_mount_after_mkfs(warm_up_empty_publish)",
            FirstPublish::RecoveryRowPublish => "mount_after_unclean_end(recovery_row_publish)",
            FirstPublish::InstanceSwitchPublish => "instance_switch(row_and_reissued_units)",
            FirstPublish::AdministratorRollbackPublish => "administrator_rollback(rollback_rows_and_units)",
        }
    }
    /// 取号之前的实例代号：mkfs 之后是 0，其余格按「第一个事务写完之后」取 1。
    const fn instance_before_acquisition(self) -> u32 {
        match self {
            FirstPublish::WarmUpEmptyPublish => 0,
            FirstPublish::RecoveryRowPublish
            | FirstPublish::InstanceSwitchPublish
            | FirstPublish::AdministratorRollbackPublish => 1,
        }
    }
}

fn superblock_write(disk: usize, instance: u32, label: &'static str) -> Step {
    Step::Write(PlannedWrite {
        kind: WriteKind::SuperblockSlot,
        disk,
        instance,
        label,
    })
}

fn other_write(kind: WriteKind, disk: usize, instance: u32, label: &'static str) -> Step {
    Step::Write(PlannedWrite {
        kind,
        disk,
        instance,
        label,
    })
}

fn acquisition_steps(arm: Arm, instance: u32) -> Vec<Step> {
    let disk_zero = superblock_write(0, instance, "acquire_superblock@disk0");
    let disk_one = superblock_write(1, instance, "acquire_superblock@disk1");
    match arm {
        Arm::NoBarrierAfterAcquisition => vec![disk_zero, disk_one],
        Arm::SerialDiskZeroThenDiskOne => vec![disk_zero, Step::Barrier, disk_one, Step::Barrier],
        Arm::OneBarrierAfterBothSuperblocks => vec![disk_zero, disk_one, Step::Barrier],
    }
}

/// 一次空发布：屏障 → 记录 × 2 盘 → 屏障 → 根槽 FUA → 超级块 × 2 盘（D16（发布语义） 已定项 7 的顺序，单元集为空）。
fn empty_publish_steps(instance: u32, root_disk: usize, labels: [&'static str; 5]) -> Vec<Step> {
    vec![
        Step::Barrier,
        other_write(WriteKind::JournalRecord, 0, instance, labels[0]),
        other_write(WriteKind::JournalRecord, 1, instance, labels[1]),
        Step::Barrier,
        other_write(WriteKind::RootRecordForceUnitAccess, root_disk, instance, labels[2]),
        superblock_write(0, instance, labels[3]),
        superblock_write(1, instance, labels[4]),
    ]
}

/// 非空的第一次发布先写哪两个 COW 单元（实例表单元之外的那一个）；暖机空发布没有单元。
fn first_publish_unit_labels(first_publish: FirstPublish) -> Option<[&'static str; 2]> {
    match first_publish {
        FirstPublish::WarmUpEmptyPublish => None,
        FirstPublish::RecoveryRowPublish => Some([
            "rewritten_fixed_point_node@disk0",
            "rewritten_fixed_point_node@disk1",
        ]),
        FirstPublish::InstanceSwitchPublish => Some([
            "reissued_checkpoint_data_unit@disk0",
            "reissued_checkpoint_data_unit@disk1",
        ]),
        FirstPublish::AdministratorRollbackPublish => Some([
            "rollback_publish_cow_unit@disk0",
            "rollback_publish_cow_unit@disk1",
        ]),
    }
}

/// 取号之后本实例的第一次发布，再接一次空发布（暖机）。
fn first_publish_steps(first_publish: FirstPublish, instance: u32) -> Vec<Step> {
    let second_empty_publish = empty_publish_steps(
        instance,
        0,
        [
            "empty_publish_2_record@disk0",
            "empty_publish_2_record@disk1",
            "empty_publish_2_root@disk0",
            "empty_publish_2_superblock@disk0",
            "empty_publish_2_superblock@disk1",
        ],
    );
    let mut steps = match first_publish_unit_labels(first_publish) {
        None => empty_publish_steps(
            instance,
            1,
            [
                "empty_publish_1_record@disk0",
                "empty_publish_1_record@disk1",
                "empty_publish_1_root@disk1",
                "empty_publish_1_superblock@disk0",
                "empty_publish_1_superblock@disk1",
            ],
        ),
        Some(cow_labels) => vec![
            other_write(WriteKind::Unit, 0, instance, "instance_table_unit@disk0"),
            other_write(WriteKind::Unit, 1, instance, "instance_table_unit@disk1"),
            other_write(WriteKind::Unit, 0, instance, cow_labels[0]),
            other_write(WriteKind::Unit, 1, instance, cow_labels[1]),
            Step::Barrier,
            other_write(WriteKind::JournalRecord, 0, instance, "first_record@disk0"),
            other_write(WriteKind::JournalRecord, 1, instance, "first_record@disk1"),
            Step::Barrier,
            other_write(WriteKind::RootRecordForceUnitAccess, 1, instance, "first_root@disk1"),
            superblock_write(0, instance, "first_superblock@disk0"),
            superblock_write(1, instance, "first_superblock@disk1"),
        ],
    };
    steps.extend(second_empty_publish);
    steps
}

/// 切段：屏障关掉它之前那一段（段空时不开新段），FUA 写关掉自己所在的那一段——与 crash.rs 的 writes_and_segments 同一条规则。
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

/// 一个崩溃状态在盘上留下的实例代号：各盘择到的超级块、根环、journal 记录、单元写序。
struct StateView {
    superblock_instance_by_disk: [u32; 2],
    root_instances: BTreeSet<u32>,
    record_instances: BTreeSet<u32>,
    unit_instances: BTreeSet<u32>,
}

/// 取号之前盘上已有的东西都带 `instance_before`（mkfs 之后那一格 journal 里还没有记录）；之后按持久的写叠上去。
fn view_state(writes: &[PlannedWrite], persisted: &[bool], instance_before: u32) -> StateView {
    let mut view = StateView {
        superblock_instance_by_disk: [instance_before; 2],
        root_instances: BTreeSet::from([instance_before]),
        record_instances: if instance_before == 0 {
            BTreeSet::new()
        } else {
            BTreeSet::from([instance_before])
        },
        unit_instances: BTreeSet::from([instance_before]),
    };
    for (planned, is_persisted) in writes.iter().zip(persisted) {
        if !*is_persisted {
            continue;
        }
        match planned.kind {
            WriteKind::SuperblockSlot => view.superblock_instance_by_disk[planned.disk] = planned.instance,
            WriteKind::Unit => {
                view.unit_instances.insert(planned.instance);
            }
            WriteKind::JournalRecord => {
                view.record_instances.insert(planned.instance);
            }
            WriteKind::RootRecordForceUnitAccess => {
                view.root_instances.insert(planned.instance);
            }
        }
    }
    view
}

impl StateView {
    fn highest_superblock_instance(&self) -> u32 {
        self.superblock_instance_by_disk[0].max(self.superblock_instance_by_disk[1])
    }
    fn highest_root_instance(&self) -> u32 {
        self.root_instances.iter().copied().max().unwrap_or(0)
    }
    fn superblocks_unequal(&self) -> bool {
        self.superblock_instance_by_disk[0] != self.superblock_instance_by_disk[1]
    }
    /// 甲的取号规则：max(全部超级块, 根环) + 1。
    fn next_acquired_instance(&self) -> u32 {
        self.highest_superblock_instance().max(self.highest_root_instance()) + 1
    }
    /// J2：下一次取号撞上盘上根、记录或单元已带着的实例代号。
    fn next_acquisition_reuses_an_instance_on_disk(&self) -> bool {
        let next = self.next_acquired_instance();
        self.root_instances.contains(&next)
            || self.record_instances.contains(&next)
            || self.unit_instances.contains(&next)
    }
    fn superblocks_at_least_root_ring(&self) -> bool {
        let highest_root = self.highest_root_instance();
        self.superblock_instance_by_disk
            .iter()
            .all(|instance| *instance >= highest_root)
    }
    /// 今天的 I-7.7 字面：各超级块相等，且 ≥ 根环里任一根。
    fn literal_invariant_holds(&self) -> bool {
        !self.superblocks_unequal() && self.superblocks_at_least_root_ring()
    }
    /// 甲改过的 I-7.7：相等，或不等时根环与 journal 里没有带最大那个超级块实例代号的根或记录。
    /// 甲的文字没说「≥ 根环里任一根」那半句删不删，`keep_root_ring_clause` 两种读法都算。
    fn jia_rewritten_invariant_holds(&self, keep_root_ring_clause: bool) -> bool {
        let highest = self.highest_superblock_instance();
        let unequal_branch_holds =
            !self.root_instances.contains(&highest) && !self.record_instances.contains(&highest);
        (!self.superblocks_unequal() || unequal_branch_holds)
            && (!keep_root_ring_clause || self.superblocks_at_least_root_ring())
    }
    /// 本腿给的替代检查：盘上每个根、journal 记录、单元写序的实例代号 ≤ 各盘择到的超级块实例代号里最大的那个。
    fn every_carried_instance_at_most_highest_superblock(&self) -> bool {
        let highest = self.highest_superblock_instance();
        self.root_instances
            .iter()
            .chain(&self.record_instances)
            .chain(&self.unit_instances)
            .all(|instance| *instance <= highest)
    }
}

#[derive(Default)]
struct PathTally {
    states: u64,
    superblocks_unequal: u64,
    reuse: u64,
    reuse_while_literal_holds: u64,
    reuse_while_jia_holds_keeping_root_clause: u64,
    reuse_while_jia_holds_dropping_root_clause: u64,
    literal_violations: u64,
    jia_violations_keeping_root_clause: u64,
    replacement_check_violations: u64,
    first_reuse_example: Option<Vec<&'static str>>,
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
        let view = view_state(&writes, persisted, instance_before);
        tally.states += 1;
        let reuse = view.next_acquisition_reuses_an_instance_on_disk();
        if view.superblocks_unequal() {
            tally.superblocks_unequal += 1;
        }
        if !view.literal_invariant_holds() {
            tally.literal_violations += 1;
        }
        if !view.jia_rewritten_invariant_holds(true) {
            tally.jia_violations_keeping_root_clause += 1;
        }
        if !view.every_carried_instance_at_most_highest_superblock() {
            tally.replacement_check_violations += 1;
        }
        if reuse {
            tally.reuse += 1;
            if view.literal_invariant_holds() {
                tally.reuse_while_literal_holds += 1;
            }
            if view.jia_rewritten_invariant_holds(true) {
                tally.reuse_while_jia_holds_keeping_root_clause += 1;
            }
            if view.jia_rewritten_invariant_holds(false) {
                tally.reuse_while_jia_holds_dropping_root_clause += 1;
            }
            if tally.first_reuse_example.is_none() {
                tally.first_reuse_example = Some(
                    writes
                        .iter()
                        .zip(persisted)
                        .filter(|(_, is_persisted)| **is_persisted)
                        .map(|(planned, _)| planned.label)
                        .collect(),
                );
            }
        }
    });
    let segment_sizes = segments.iter().map(Vec::len).collect();
    (tally, segment_sizes, barrier_count(&steps))
}

/// D18（块里携带什么信息） 已定项 11 的全局已发布谓词（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2），按条文四支。
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

#[derive(Clone, Copy, Debug)]
struct OrphanUnit {
    instance: u32,
    transaction: u64,
    birth_txg: u64,
    class: UnitClassForPredicate,
}

fn published_predicate(
    unit: OrphanUnit,
    rows: &[InstanceRow],
    mount_root_instance: u32,
    mount_root_txg: u64,
) -> PublishedVerdict {
    if unit.instance > mount_root_instance {
        return PublishedVerdict::Corrupt;
    }
    if unit.instance == mount_root_instance {
        return if unit.birth_txg <= mount_root_txg {
            PublishedVerdict::Published
        } else {
            PublishedVerdict::Unpublished
        };
    }
    match rows.iter().find(|row| row.instance == unit.instance) {
        None => PublishedVerdict::Published,
        Some(row) => {
            let published = match unit.class {
                UnitClassForPredicate::DataUnitCodeOne => {
                    unit.birth_txg <= row.published_txg
                        || unit.transaction <= row.highest_applied_transaction
                }
                UnitClassForPredicate::NodeOrPackedCodeTwoOrThree => {
                    unit.birth_txg <= row.published_txg
                }
            };
            if published {
                PublishedVerdict::Published
            } else {
                PublishedVerdict::Unpublished
            }
        }
    }
}

/// 取号之前：实例 1，最后发布的根 (1, 3)。第一次尝试取实例 2，第一次发布 txg 4 的单元落盘、超级块没落盘、崩。
/// 甲：下一次取号 = max(1, 1) + 1 = 2（复用），恢复写行 [1, 2) = (1, 3, 0)，复用的实例 2 的根依次 txg 4、5。
/// 超级块已带 2 时（乙 / 甲″ 下单元落了盘就必然如此）：下一次取号 = 3，写行 [1, 3) = (1, 3, 0)、(2, 3, 0)。
fn report_predicate_flip() {
    let row_for = |instance: u32| InstanceRow {
        instance,
        published_txg: 3,
        highest_applied_transaction: 0,
    };
    let orphans = [
        OrphanUnit {
            instance: 2,
            transaction: 1,
            birth_txg: 4,
            class: UnitClassForPredicate::DataUnitCodeOne,
        },
        OrphanUnit {
            instance: 2,
            transaction: 0,
            birth_txg: 4,
            class: UnitClassForPredicate::NodeOrPackedCodeTwoOrThree,
        },
    ];
    for orphan in orphans {
        let before_next_acquisition = published_predicate(orphan, &[], 1, 3);
        let reused_root_four = published_predicate(orphan, &[row_for(1)], 2, 4);
        let reused_root_five = published_predicate(orphan, &[row_for(1)], 2, 5);
        let fresh_root_four = published_predicate(orphan, &[row_for(1), row_for(2)], 3, 4);
        let fresh_root_five = published_predicate(orphan, &[row_for(1), row_for(2)], 3, 5);
        println!(
            "PREDICATE orphan={orphan:?} mount_root(1,3)={before_next_acquisition:?} jia_reused_instance2_root_txg4={reused_root_four:?} jia_reused_instance2_root_txg5={reused_root_five:?} fresh_instance3_root_txg4={fresh_root_four:?} fresh_instance3_root_txg5={fresh_root_five:?}"
        );
    }
}

/// 下一次取号的超级块槽写用哪个世代号：条款与实现今天各说各的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenerationRule {
    /// D22（单元原子性怎么合成） 已定项 16 字面：每盘槽世代号每写一次 +1。
    PerDiskIncrement,
    /// 实现的写法（transaction.rs 一次写给每块盘同一个世代号）推广到之后的挂载：全池最大 + 1。
    PoolWideHighestPlusOne,
    /// 今天代码里的常量：取号那次恒写世代号 2（SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION）。
    TodayConstantTwoAtAcquisition,
}

impl GenerationRule {
    const EVERY_RULE: [GenerationRule; 3] = [
        GenerationRule::PerDiskIncrement,
        GenerationRule::PoolWideHighestPlusOne,
        GenerationRule::TodayConstantTwoAtAcquisition,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotContent {
    generation: u64,
    instance: u32,
}

type DiskSlots = [SlotContent; 2];

/// 一块盘两槽里择世代号大的（相等取槽 0，与 checker 的 chosen_superblocks 同写法）。
fn newest_slot(slots: &DiskSlots) -> SlotContent {
    if slots[1].generation > slots[0].generation {
        slots[1]
    } else {
        slots[0]
    }
}

/// 甲的取号规则：全部超级块（两盘各两槽）里最大的实例代号 + 1（这一格根环里没有更大的根）。
fn instance_for_acquisition(disks: &[DiskSlots; 2]) -> u32 {
    disks
        .iter()
        .flat_map(|slots| slots.iter())
        .map(|slot| slot.instance)
        .max()
        .unwrap_or(0)
        + 1
}

fn acquisition_generation(rule: GenerationRule, disks: &[DiskSlots; 2], disk: usize) -> u64 {
    match rule {
        GenerationRule::PerDiskIncrement => newest_slot(&disks[disk]).generation + 1,
        GenerationRule::PoolWideHighestPlusOne => {
            newest_slot(&disks[0])
                .generation
                .max(newest_slot(&disks[1]).generation)
                + 1
        }
        GenerationRule::TodayConstantTwoAtAcquisition => 2,
    }
}

/// 一次取号：`persisted_disks` 标的盘写上，其余没落盘（崩溃）；槽 = 世代号 mod 2。
fn apply_acquisition(rule: GenerationRule, disks: &mut [DiskSlots; 2], persisted_disks: [bool; 2]) {
    let instance = instance_for_acquisition(disks);
    let generations = [
        acquisition_generation(rule, disks, 0),
        acquisition_generation(rule, disks, 1),
    ];
    for disk in 0..2 {
        if persisted_disks[disk] {
            let slot_position = usize::try_from(generations[disk] % 2).expect("0 或 1");
            disks[disk][slot_position] = SlotContent {
                generation: generations[disk],
                instance,
            };
        }
    }
}

/// 只看取号那几个超级块写时各臂的崩溃状态：甲与甲″ 两盘同段；乙只有前缀。
fn acquisition_crash_states(arm: Arm) -> Vec<[bool; 2]> {
    match arm {
        Arm::NoBarrierAfterAcquisition | Arm::OneBarrierAfterBothSuperblocks => {
            vec![[false, false], [true, false], [false, true], [true, true]]
        }
        Arm::SerialDiskZeroThenDiskOne => vec![[false, false], [true, false], [true, true]],
    }
}

/// J4：第一个事务写完之后（两盘都是槽 0 世代 4、槽 1 世代 5、实例 1）连着两次取号、每次都可能只落一部分；
/// 数「只看世代号」择到的实例代号比各盘择到的最大实例代号小的历史。
fn report_generation_rules() {
    let after_first_mount = [
        SlotContent { generation: 4, instance: 1 },
        SlotContent { generation: 5, instance: 1 },
    ];
    for arm in Arm::EVERY_ARM {
        for rule in GenerationRule::EVERY_RULE {
            let mut histories = 0;
            let mut generation_choice_lower = 0;
            let mut some_slot_above_generation_choice = 0;
            let mut first_example = None;
            for first_crash in acquisition_crash_states(arm) {
                for second_crash in acquisition_crash_states(arm) {
                    let mut disks = [after_first_mount, after_first_mount];
                    apply_acquisition(rule, &mut disks, first_crash);
                    apply_acquisition(rule, &mut disks, second_crash);
                    histories += 1;
                    let newest = [newest_slot(&disks[0]), newest_slot(&disks[1])];
                    let by_generation = if newest[1].generation > newest[0].generation { newest[1] } else { newest[0] };
                    if by_generation.instance != newest[0].instance.max(newest[1].instance) {
                        generation_choice_lower += 1;
                        first_example.get_or_insert((first_crash, second_crash, disks));
                    }
                    if by_generation.instance < instance_for_acquisition(&disks) - 1 {
                        some_slot_above_generation_choice += 1;
                    }
                }
            }
            println!("GENERATION arm={} rule={rule:?} histories={histories} generation_only_choice_below_highest_chosen={generation_choice_lower} some_slot_instance_above_generation_choice={some_slot_above_generation_choice} first_example={first_example:?}", arm.name());
        }
    }
}

fn main() {
    for first_publish in FirstPublish::EVERY_KIND {
        for arm in Arm::EVERY_ARM {
            let (tally, segment_sizes, barriers) = tally_path(arm, first_publish);
            println!(
                "PATH first_publish={} arm={} segments={segment_sizes:?} barriers={barriers} states={} superblocks_unequal={} literal_I-7.7_violations={} jia_I-7.7_violations(keep_root_clause)={} replacement_check_violations={} reuse={} reuse_while_literal_I-7.7_holds={} reuse_while_jia_I-7.7_holds(keep_root_clause)={} reuse_while_jia_I-7.7_holds(drop_root_clause)={} first_reuse_persisted={:?}",
                first_publish.name(),
                arm.name(),
                tally.states,
                tally.superblocks_unequal,
                tally.literal_violations,
                tally.jia_violations_keeping_root_clause,
                tally.replacement_check_violations,
                tally.reuse,
                tally.reuse_while_literal_holds,
                tally.reuse_while_jia_holds_keeping_root_clause,
                tally.reuse_while_jia_holds_dropping_root_clause,
                tally.first_reuse_example
            );
        }
    }
    report_predicate_flip();
    report_generation_rules();
}
