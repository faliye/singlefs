//! 回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」，C332（回退实例两个根都读不出时回退被撤销） 的修法，用户 2026-09-24 定）：
//! 回退在系统配置槽里记一张见证表（字段表之后、越过 512），一个条目 = 新实例代号 + R_old 的实例代号与 txg；
//! 择根先跳过被任一条目抛弃的根——(r_old, T_old) < (i, T)（实例代号为主比）且 i < N；
//! 见证随回退那一次挂载写行之后的第一次系统配置轮换写，之后每一次系统配置写都带着整张表；条数上限 R × S − 1；
//! 见证表读不出就是系统配置槽读不出。条目什么时候删随实现（删除规则见 `singlefs_core::mount` 的 `rollback_witness_entries_still_needed`）。
//!
//! 历史都从这一段起：A（第一个文件，txg 3）→ 覆盖写 B（txg 4）→ 覆盖写 C（txg 5），实例 1；进程退出、重开走管理员回退到 A 的根 (1, 3)：
//! 取号 2、写行那次发布（txg 6）写回退行、暖机一次（txg 7）。实例 2 的两条根是 txg 6、7。

mod common;

use common::{
    build_pool, crash_state_devices, disk_snapshot, file_content, geometry, parameters,
    publish_overwrite_in_process, replace_the_witness_on_one_device, BuiltPool, Recorded,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::{
    mount_rollback, mount_writable, MountError, Mounted, RollbackCandidateExclusion,
    RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, recover,
    rollback_witness_of_the_pool, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::rollback_witness::{RollbackWitnessEntry, RollbackWitnessTable};
use singlefs_core::root_ring::{target_for_publish, RootRingSlot};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_core::transaction::TransactionOutput;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, NamedRootRingSlots,
    PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};
use singlefs_harness::{RecordedOperationKind, RetainedOperation, SharedStream};

const ROLLBACK_TARGET: RollbackTarget = RollbackTarget {
    instance: InstanceGeneration(1),
    checkpoint_txg: CheckpointTxg(3),
};

fn content_of(seed: usize) -> Vec<u8> {
    (0..2600 + seed)
        .map(|index| u8::try_from((index * 7 + seed * 11) % 239).expect("小于 256"))
        .collect()
}

/// A、B、C 三版（实例 1），交回池与 C 那一版。
fn pool_through_the_third_version(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(1),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let third = publish_overwrite_in_process(
        &mut pool,
        &second,
        &content_of(2),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("C");
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(5));
    pool.output = third.clone();
    (pool, third)
}

/// A、B、C 之后进程退出、重开走回退到 (1, 3)：交回池（设备还开着、录制流接着录）与回退那一次挂载的输出。
fn pool_rolled_back_to_the_first_version(tag: &str) -> (BuiltPool, Mounted) {
    let (mut pool, _third) = pool_through_the_third_version(tag);
    let mut devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        ROLLBACK_TARGET,
        ShadowLedger::On,
    )
    .expect("回退到 (1, 3)");
    pool.devices = Some(devices);
    (pool, rolled_back)
}

/// 回退那一次挂载写出的全部根的 (实例, txg)。
fn roots_of_the_mount(mounted: &Mounted) -> Vec<(InstanceGeneration, CheckpointTxg)> {
    std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|version| (version.root().instance, version.root().checkpoint_txg))
        .collect()
}

/// 这几条根住的根环槽。
fn ring_slots_of(roots: &[(InstanceGeneration, CheckpointTxg)]) -> Vec<RootRingSlot> {
    roots
        .iter()
        .map(|(_, txg)| target_for_publish(*txg, parameters().geometry.root_ring_slots_per_region))
        .collect()
}

fn root_ring_slot_target(slots: &[RootRingSlot]) -> RootRingSlotTarget {
    RootRingSlotTarget {
        named_slots: NamedRootRingSlots::naming(slots),
        region_devices: parameters().region_devices,
        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
    }
}

const THE_ROLLBACK: RollbackWitnessEntry = RollbackWitnessEntry {
    new_instance: InstanceGeneration(2),
    rollback_target_instance: InstanceGeneration(1),
    rollback_target_txg: CheckpointTxg(3),
};

/// 验收（C332 那一格）：回退之后，回退实例的根（txg 6、7）在两块盘上都读不出。择根跳过被见证表抛弃的 B（1, 4）、C（1, 5），
/// 落到 R_old（1, 3）；重放在被抛弃的 B 的记录上停下——恢复读回 A，不回到被抛弃的时间线。
/// 改之前：择根不看见证、只按 (txg, 实例) 取最大，落到 C（1, 5）、读回 C，回退被静默撤销。
#[test]
fn with_the_rollback_instances_roots_unreadable_recovery_does_not_return_to_the_abandoned_timeline()
{
    let (pool, rolled_back) = pool_rolled_back_to_the_first_version("witness-c332");
    let roots_of_the_rollback = roots_of_the_mount(&rolled_back);
    assert_eq!(
        roots_of_the_rollback,
        vec![
            (InstanceGeneration(2), CheckpointTxg(6)),
            (InstanceGeneration(2), CheckpointTxg(7)),
        ],
        "回退：写行 txg 6、暖机 txg 7"
    );
    let image = pool.memory_pool();
    let reader = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        root_ring_slot_target(&ring_slots_of(&roots_of_the_rollback)),
    );
    let report = recover(&reader, JournalPolicy::Consult);
    assert!(reader.reads_refused() > 0, "回退实例那两条根的读真被拦下了");
    let RecoveryOutcome::FileRead { root, content } = &report.outcome else {
        panic!("要读回一个文件：{:?}", report.outcome);
    };
    assert_eq!(
        (*root, report.effective_root, report.journal.prefix_applied),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            Some((InstanceGeneration(1), CheckpointTxg(3))),
            0
        ),
        "择到 R_old、被抛弃的 B 的记录一条都不施加"
    );
    assert!(content == &file_content(), "读回 A，不是被抛弃的 B 或 C");
}

/// 写序 post（D23（journal 的角色与格式） 已定项 14「回退见证」：见证随写行之后的第一次系统配置轮换写，之后每一次系统配置写都带着整张表）：
/// 回退那一次挂载的六次系统配置写——取号两次带的还是空表（这次那一条不在里面）；写行那次发布的轮换两次、暖机的轮换两次都带着
/// (2, 1, 3)。之后同一个进程里的覆盖写、再一次可写挂载（取号、写行、暖机），每一次系统配置写都照带着它。
#[test]
fn the_witness_rides_from_the_first_rotation_after_the_row_publish_and_every_later_system_configuration_write(
) {
    let (mut pool, _third) = pool_through_the_third_version("witness-write-order");
    let operations_before_the_rollback = pool.retained_operations().len();
    let mut devices = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        ROLLBACK_TARGET,
        ShadowLedger::On,
    )
    .expect("回退到 (1, 3)");
    pool.devices = Some(devices);
    assert_eq!(
        rolled_back.output.rollback_witness_written.entries(),
        &[THE_ROLLBACK],
        "这次挂载写的见证表"
    );
    let witness_of_each_system_configuration_write = |from: usize,
                                                      observed_pool: &BuiltPool|
     -> Vec<Vec<RollbackWitnessEntry>> {
        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        observed_pool.retained_operations()[from..]
            .iter()
            .filter(|retained| {
                retained.operation.kind == RecordedOperationKind::Write
                    && (retained.operation.offset.0 == 0 || retained.operation.offset.0 == spacing)
                    && retained.operation.length == SYSTEM_CONFIGURATION_SLOT_BYTES
            })
            .map(|retained| {
                let bytes = retained.contents.as_ref().expect("录制流保留内容");
                SystemConfiguration::parse_slot(bytes)
                    .expect("写出去的系统配置槽自证得过")
                    .rollback_witness
                    .entries()
                    .to_vec()
            })
            .collect()
    };
    assert_eq!(
        witness_of_each_system_configuration_write(operations_before_the_rollback, &pool),
        vec![
            Vec::new(),
            Vec::new(),
            vec![THE_ROLLBACK],
            vec![THE_ROLLBACK],
            vec![THE_ROLLBACK],
            vec![THE_ROLLBACK],
        ],
        "取号两次空表；写行的轮换两次、暖机的轮换两次都带着这一次回退那一条"
    );

    let operations_before_the_later_writes = pool.retained_operations().len();
    pool.allocator = rolled_back.allocator;
    let current = rolled_back
        .current
        .into_file_version()
        .expect("回退到 A，现行那一版带文件");
    publish_overwrite_in_process(
        &mut pool,
        &current,
        &content_of(3),
        FIXED_WRITE_TIME_SECONDS + 180,
        InstanceGeneration(2),
    )
    .expect("回退之后覆盖写");
    let mut devices_for_the_remount = pool.reopen_recorded();
    let remounted =
        mount_writable(&parameters(), &mut devices_for_the_remount).expect("再可写挂载");
    pool.devices = Some(devices_for_the_remount);
    assert_eq!(
        remounted.output.rollback_witness_written.entries(),
        &[THE_ROLLBACK],
        "环里还住着实例 1 的根：条目照留"
    );
    let later =
        witness_of_each_system_configuration_write(operations_before_the_later_writes, &pool);
    assert!(
        later.len() >= 6 && later.iter().all(|entries| entries == &vec![THE_ROLLBACK]),
        "覆盖写的轮换、再挂载的取号、写行与暖机的轮换，每一次系统配置写都带着整张表：{later:?}"
    );
}

/// 回退之后覆盖写 `overwrites` 次（实例 2），交回池与最后那一版。
fn pool_rolled_back_and_overwritten(
    tag: &str,
    overwrites: usize,
) -> (BuiltPool, TransactionOutput) {
    let (mut pool, rolled_back) = pool_rolled_back_to_the_first_version(tag);
    pool.allocator = rolled_back.allocator;
    let mut current = rolled_back
        .current
        .into_file_version()
        .expect("回退到 A，现行那一版带文件");
    for seed in 0..overwrites {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(10 + seed),
            FIXED_WRITE_TIME_SECONDS + 200 + u64::try_from(seed).expect("次数"),
            InstanceGeneration(2),
        )
        .expect("回退之后覆盖写");
    }
    pool.output = current.clone();
    (pool, current)
}

/// 删除规则（实二二三定，写进报告）：条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、自证过，而且其中没有一条根的实例代号落在
/// [r_old, N) 里。回退之后覆盖写 22 次（txg 8..=29）：根环 24 个槽住着实例 2 的 txg 6..=29，一条实例 1 的根都不剩——
/// 下一次可写挂载把 (2, 1, 3) 删掉，写出去的表是空的。
/// 同一段历史、挂载时根环里有一个槽（txg 29 那一槽）一直读不出：那一槽按「可能住着被抛弃的根」算，条目照留。
#[test]
fn an_entry_is_dropped_only_when_every_ring_slot_is_readable_and_no_root_lies_between_the_target_and_the_new_instance(
) {
    let (mut pool, last) = pool_rolled_back_and_overwritten("witness-drop", 22);
    assert_eq!(last.root.checkpoint_txg, CheckpointTxg(29));
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
    assert_eq!(
        remounted.output.rollback_witness_written,
        RollbackWitnessTable::EMPTY,
        "根环里没有实例 1 的根了：条目删掉"
    );
    pool.devices = Some(devices);
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    assert_eq!(
        system_configuration.rollback_witness,
        RollbackWitnessTable::EMPTY,
        "盘上择到的那一槽里的见证表也是空的"
    );

    let (mut kept_pool, _) = pool_rolled_back_and_overwritten("witness-kept-unreadable-slot", 22);
    let unreadable = root_ring_slot_target(&ring_slots_of(&[(
        InstanceGeneration(2),
        CheckpointTxg(29),
    )]));
    let plan = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule::every_read_of_named_root_ring_slots_fails(unreadable),
    );
    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = kept_pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let remounted_with_an_unreadable_slot =
        mount_writable(&parameters(), &mut failing_devices).expect("一个根槽读不出照样可写挂载");
    assert!(plan.fired_count() > 0, "那一槽的读真被拦下了");
    assert_eq!(
        remounted_with_an_unreadable_slot
            .output
            .rollback_witness_written
            .entries(),
        &[THE_ROLLBACK],
        "有一个根槽读不出：条目照留"
    );
    drop(failing_devices);
    kept_pool.devices = None;
}

/// 回退目标被见证表抛弃也不在回退候选集里（它在被抛弃的时间线上）：回退实例的根都读不出时，最新根落回 R_old，
/// R_old 那一版的实例表里没有回退行，只按实例表判会把 C（1, 5）放回候选集。在任何写之前拒绝，盘上逐字节不变。
#[test]
fn a_rollback_target_abandoned_by_the_witness_is_not_a_candidate() {
    let (mut pool, rolled_back) = pool_rolled_back_to_the_first_version("witness-candidate");
    let unreadable = root_ring_slot_target(&ring_slots_of(&roots_of_the_mount(&rolled_back)));
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let plan = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule::every_read_of_named_root_ring_slots_fails(unreadable),
    );
    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let refused = mount_rollback(
        &parameters(),
        &mut failing_devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(5),
        },
        ShadowLedger::On,
    );
    assert!(
        matches!(
            refused,
            Err(MountError::RollbackTargetNotACandidate {
                exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,
                ..
            })
        ),
        "被见证表抛弃的 C 不是候选：{refused:?}"
    );
    drop(failing_devices);
    pool.devices = None;
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "在任何写之前拒绝：系统配置槽、根环、录制流步数逐项不变"
    );
}

/// 把两块盘上四个系统配置槽里的见证表换成 `table`（其余字段照旧、整槽校验和重封）：造「表已经很满」的池。
fn replace_the_witness_in_every_system_configuration_slot(
    pool: &mut BuiltPool,
    table: RollbackWitnessTable,
) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let devices = pool.devices.as_mut().expect("镜像还开着");
    for (_, device) in devices.iter_mut() {
        for offset in [0, spacing] {
            let mut bytes =
                vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")];
            device
                .read_at(DeviceOffsetInBytes(offset), &mut bytes)
                .expect("读系统配置槽");
            let mut system_configuration =
                SystemConfiguration::parse_slot(&bytes).expect("两槽都自证过");
            system_configuration.rollback_witness = table;
            device
                .write_at(
                    DeviceOffsetInBytes(offset),
                    &system_configuration.to_slot(),
                    WriteDurability::Plain,
                )
                .expect("写回系统配置槽");
        }
    }
}

/// 条数上限 R × S − 1（第一版 S = 8：23 条）。删除规则删不掉的条目占满了表（造出来的：23 条 (k + 3, 1, k + 5)，k = 0..=22——
/// 环里住着实例 1 的根，按第一条删除规则一条都删不掉；新实例代号与目标一起严格递增，彼此罩不住，也罩不住这一次那一条 (2, 1, 3)
/// （新实例 2 小于 3），第二条删除规则同样一条都删不掉；它们只抛弃实例 1 里 txg 越过 5 的根与实例 2 起的根，环里一条都没有），
/// 再回退一次就要写 24 条：写满之后怎么办条款没写（C547（回退见证表的删除规则与写满没有条款）） ⇒ 第一版不支持，在取号之前返回
/// `RollbackWitnessTableFullWhoseHandlingIsUndecided`，盘上逐字节不变。只钉「返回这个成员、盘上不变」，不钉之后的行为。
/// 同一个池上普通的可写挂载不加条目，照常挂上。
#[test]
fn a_rollback_whose_witness_table_would_exceed_its_capacity_is_refused_before_any_write() {
    let (mut pool, _third) = pool_through_the_third_version("witness-full");
    let crowded = RollbackWitnessTable::of_entries(
        (0..23u32).map(|position| RollbackWitnessEntry {
            new_instance: InstanceGeneration(position + 3),
            rollback_target_instance: InstanceGeneration(1),
            rollback_target_txg: CheckpointTxg(u64::from(position) + 5),
        }),
        23,
    )
    .expect("23 条正好装满");
    replace_the_witness_in_every_system_configuration_slot(&mut pool, crowded);
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let mut devices = pool.reopen_recorded();
    let refused = mount_rollback(
        &parameters(),
        &mut devices,
        ROLLBACK_TARGET,
        ShadowLedger::On,
    );
    assert!(
        matches!(
            refused,
            Err(MountError::Recovery(
                RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided {
                    entries: 24,
                    capacity: 23,
                }
            ))
        ),
        "删不掉的 23 条加这一次那一条是 24 条：{refused:?}"
    );
    pool.devices = Some(devices);
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "在取号之前拒绝：系统配置槽、根环、录制流步数逐项不变"
    );
    let mut devices_for_the_writable_mount = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_for_the_writable_mount)
        .expect("普通的可写挂载不加条目，照常挂上");
    assert_eq!(mounted.output.rollback_witness_written, crowded);
    pool.devices = Some(devices_for_the_writable_mount);
}

/// 可写挂载在回退实例的根都读不出时同样不回到被抛弃的时间线：择到 R_old（1, 3）、一条被抛弃的记录都不施加，新实例接在 A 后面。
#[test]
fn a_writable_mount_with_the_rollback_instances_roots_unreadable_builds_on_the_rollback_target() {
    let (mut pool, rolled_back) =
        pool_rolled_back_to_the_first_version("witness-mount-under-faults");
    let unreadable = root_ring_slot_target(&ring_slots_of(&roots_of_the_mount(&rolled_back)));
    let plan = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule::every_read_of_named_root_ring_slots_fails(unreadable),
    );
    let mut failing_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let mounted = mount_writable(&parameters(), &mut failing_devices).expect("可写挂载");
    assert_eq!(
        (
            (
                mounted.output.chosen_root.instance,
                mounted.output.chosen_root.checkpoint_txg
            ),
            (
                mounted.output.effective_root.instance,
                mounted.output.effective_root.checkpoint_txg
            ),
            mounted.output.journal.prefix_applied
        ),
        (
            (InstanceGeneration(1), CheckpointTxg(3)),
            (InstanceGeneration(1), CheckpointTxg(3)),
            0
        ),
        "择到 R_old、被抛弃的记录一条都不施加"
    );
    assert_eq!(
        mounted.output.rollback_witness_written.entries(),
        &[THE_ROLLBACK],
        "回退实例的根读不出：条目照留"
    );
    drop(failing_devices);
    pool.devices = None;
    let image: MemoryPool = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert!(
        matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_content()),
        "撤了读故障之后冷启动读回的仍是 A：{:?}",
        report.outcome
    );
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(judged, _)| *judged == invariant)
        .map(|(_, verdict)| verdict)
        .expect("checker 每次都报全部第一版不变量")
}

fn violated(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect()
}

/// 池级 checker 的两条新判定（不变量条文草稿交书记员）：
/// - I-7.10（回退见证表各槽自洽、各盘一致）：每个自证过的系统配置槽里的见证表解得开，同一个新实例代号在各盘各槽里记的回退目标相同；
/// - I-7.11（所选根不被见证表抛弃，且见证与所选根的实例表对得上）：所选根的实例表罩得住见证表里新实例代号不大于它的每一条。
///
/// 阳性对照：回退之后、再挂载之后的整条流镜像上两条都成立，checker 一条不红。
#[test]
fn the_pool_checker_holds_both_witness_invariants_after_a_rollback() {
    let (mut pool, rolled_back) = pool_rolled_back_to_the_first_version("witness-checker-green");
    pool.allocator = rolled_back.allocator;
    let mut devices = pool.reopen_recorded();
    mount_writable(&parameters(), &mut devices).expect("回退之后再挂载");
    pool.devices = Some(devices);
    let image = pool.memory_pool();
    assert_eq!(violated(&image), Vec::<&str>::new());
    for invariant in ["I-7.10", "I-7.11"] {
        assert_eq!(
            verdict_of(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant}"
        );
    }
}

/// I-7.10 的两份坏镜像：① 盘 1 两槽把新实例 2 的回退目标记成 (1, 4)、盘 0 记 (1, 3)——同一次回退两种说法；
/// ② 盘 0 两槽的见证表条数改成 24（S = 8 的池上限 23），整槽校验和重封过——校验和罩得住的槽里见证表解不开。两份都只红 I-7.10。
#[test]
fn contradicting_or_unparseable_witness_tables_redden_the_witness_consistency_invariant() {
    let (pool, _) = pool_rolled_back_to_the_first_version("witness-checker-i-7-10");
    let image = pool.memory_pool();

    let mut contradicting = image.clone();
    replace_the_witness_on_one_device(
        &mut contradicting,
        DeviceIdentity(1),
        RollbackWitnessTable::of_entries(
            [RollbackWitnessEntry {
                rollback_target_txg: CheckpointTxg(4),
                ..THE_ROLLBACK
            }],
            23,
        )
        .expect("一条"),
    );
    assert!(
        matches!(verdict_of(&contradicting, "I-7.10"), InvariantVerdict::Violated(detail) if detail.contains("两种说法")),
        "同一个新实例两种回退目标：{:?}",
        verdict_of(&contradicting, "I-7.10")
    );

    let mut unparseable = image;
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for offset in [0, spacing] {
        let mut bytes = singlefs_core::recovery::PoolReader::read(
            &unparseable,
            DeviceIdentity(0),
            DeviceOffsetInBytes(offset),
            slot_bytes,
        )
        .expect("系统配置槽读得到");
        bytes[481] = 24;
        let digest = wide_checksum_with_field_zeroed(
            &bytes,
            slot_bytes,
            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
        );
        bytes[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        unparseable
            .devices
            .get_mut(&DeviceIdentity(0))
            .expect("有这块盘")
            .write(DeviceOffsetInBytes(offset), &bytes);
    }
    assert!(
        matches!(verdict_of(&unparseable, "I-7.10"), InvariantVerdict::Violated(detail) if detail.contains("解不开")),
        "校验和过而见证表解不开：{:?}",
        verdict_of(&unparseable, "I-7.10")
    );
}

/// I-7.11 的坏镜像：没回退过的池（A、B 之后重开可写挂载，实例 2 写的行是 (1, 4, W)），往四个系统配置槽里塞一条见证 (2, 1, 3)——
/// 说实例 2 是回退到 (1, 3) 建的，可所选根（实例 2）的实例表里实例 1 那一行记的是 4、罩不住它 ⇒ I-7.11 红。
/// 塞进去的条目还把 B（1, 4）判成被抛弃：checker 的候选集随之少了 B，B 引用的单元不再算进遍历、记账里还占着，I-3.1 跟着红——
/// 这一红是见证与实例表说反话的直接投影，两条之外一条都不红。
#[test]
fn a_witness_entry_the_chosen_roots_instance_table_does_not_cover_reddens_the_witness_table_invariant(
) {
    let mut pool = build_pool("witness-checker-i-7-11");
    let first = pool.output.clone();
    publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(1),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let mut devices = pool.reopen_recorded();
    mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    pool.devices = Some(devices);
    let mut image = pool.memory_pool();
    assert_eq!(
        violated(&image),
        Vec::<&str>::new(),
        "改之前 checker 一条不红"
    );
    let planted = RollbackWitnessTable::of_entries([THE_ROLLBACK], 23).expect("一条");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        replace_the_witness_on_one_device(&mut image, device, planted);
    }
    assert_eq!(
        violated(&image),
        vec!["I-3.1", "I-7.11"],
        "红 I-7.11，外加被抛弃的 B 掉出候选集的 I-3.1"
    );
    assert!(
        matches!(verdict_of(&image, "I-7.11"), InvariantVerdict::Violated(detail) if detail.contains("罩不住")),
        "{:?}",
        verdict_of(&image, "I-7.11")
    );
}

/// 回退到 mkfs 的第 0 代根 (0, 0)（故障注入快档打中的形状：种子 7463871032432355114 第 7 步 `CloseAndMountRollback`）：
/// 见证条目 (N, 0, 0)；实例 0 不写行（D18（块里携带什么信息） 已定项 11），它只有 txg 0 那一条根，I-7.11 不要它那一行——
/// 所选根的实例表只有中间实例 [1, N) 的 (i, 0, 0) 行。整条流镜像上 checker 一条不红，I-7.11 成立。
#[test]
fn a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero(
) {
    let (mut pool, _third) =
        pool_through_the_third_version("witness-checker-rollback-to-mkfs-root");
    let rolled_back = {
        let mut devices = pool.reopen_recorded();
        let mounted = mount_rollback(
            &parameters(),
            &mut devices,
            RollbackTarget {
                instance: InstanceGeneration(0),
                checkpoint_txg: CheckpointTxg(0),
            },
            ShadowLedger::On,
        )
        .expect("回退到 mkfs 的第 0 代根");
        pool.devices = Some(devices);
        mounted
    };
    assert_eq!(
        rolled_back.output.rollback_witness_written.entries(),
        &[RollbackWitnessEntry {
            new_instance: InstanceGeneration(2),
            rollback_target_instance: InstanceGeneration(0),
            rollback_target_txg: CheckpointTxg(0),
        }],
        "见证条目 (2, 0, 0)"
    );
    let image = pool.memory_pool();
    assert_eq!(violated(&image), Vec::<&str>::new());
    assert_eq!(verdict_of(&image, "I-7.11"), InvariantVerdict::Holds);
}

/// 代码轮第二轮判决 Z9 那几段历史的起点：A（第一个文件，txg 3）之后接连覆盖写到 txg `last_txg`，实例 1。
fn pool_overwritten_through_txg(tag: &str, last_txg: u64) -> BuiltPool {
    let mut pool = build_pool(tag);
    let mut current = pool.output.clone();
    let mut seed = 1usize;
    // 迭代上界是 last_txg − 3 次覆盖写；每一次 txg 加一。
    while current.root.checkpoint_txg.0 < last_txg {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(seed),
            FIXED_WRITE_TIME_SECONDS + 60 * u64::try_from(seed).expect("次数"),
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        seed += 1;
    }
    pool.output = current;
    pool
}

/// 一次挂载的录制流里写行那次发布的根 FUA 在第几步（这次挂载的第一次 FUA 写）。
fn first_root_write(operations: &[RetainedOperation]) -> usize {
    operations
        .iter()
        .position(|operation| {
            operation.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("挂载写过根")
}

/// 根 FUA 之后紧跟着的系统配置轮换（两块盘各写一次槽，偏移在前两个系统配置槽里）走完的那一步。
fn end_of_the_rotation_after(operations: &[RetainedOperation], root_write: usize) -> usize {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    let rotation_writes = operations[root_write + 1..]
        .iter()
        .take_while(|operation| {
            operation.operation.kind == RecordedOperationKind::Write
                && operation.operation.offset.0 < 2 * spacing
        })
        .count();
    assert_eq!(rotation_writes, 2, "根 FUA 之后两块盘各轮换一次系统配置槽");
    root_write + 1 + rotation_writes
}

/// 连着回退时每一次挑哪条根当目标。
#[derive(Clone, Copy, Debug)]
enum RollbackTargetChoice {
    /// 一直回退到同一条根；它不再是候选（离开根环或被抛弃）之后回退到最新的根。
    TheSameTargetWhileItIsACandidateThenTheNewestRoot(RollbackTarget),
    /// 每一次回退到最新的根（空回退）。
    TheNewestRoot,
}

/// 连着回退里一次回退挂载的结局。
#[derive(Debug)]
enum RollbackAttempt {
    /// 做成了：这次挂载取的新实例代号，与它从写行那次发布的轮换起写的见证表条数。
    Mounted {
        new_instance: InstanceGeneration,
        witness_entries_written: usize,
    },
    Refused(MountError),
}

/// 连着回退 `mounts` 次，每一次回退挂载都崩在写行那次发布的系统配置轮换落盘之后、暖机之前（一次挂载只落一条根：
/// 代码轮第二轮判决第三节那一格，根环全读得出）。交回每一次的结局；有一次被拒（不是「不是候选」）就停在那一次。
fn rollbacks_each_crashing_right_after_the_row_publish_rotation(
    tag: &str,
    choice: RollbackTargetChoice,
    mounts: usize,
) -> Vec<RollbackAttempt> {
    let pool = pool_overwritten_through_txg(tag, 30);
    let mut image = pool.memory_pool();
    drop(pool);
    let mut attempts = Vec::new();
    // 迭代上界是 mounts 次回退；跨轮携带的是镜像（每一次施加到崩溃那一步为止）。
    for _mount in 0..mounts {
        let system_configuration = choose_system_configuration(&image).expect("系统配置");
        let newest = choose_root(&image, &system_configuration).expect("有根");
        let newest_target = RollbackTarget {
            instance: newest.instance,
            checkpoint_txg: newest.checkpoint_txg,
        };
        let targets = match choice {
            RollbackTargetChoice::TheSameTargetWhileItIsACandidateThenTheNewestRoot(target) => {
                vec![target, newest_target]
            }
            RollbackTargetChoice::TheNewestRoot => vec![newest_target],
        };
        let mut this_attempt = None;
        for target in targets {
            let stream = SharedStream::retaining_contents();
            let mut devices = crash_state_devices(&image, &[], &[], &stream);
            match mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On) {
                Ok(mounted) => {
                    let operations = stream.retained_operations();
                    let persisted =
                        end_of_the_rotation_after(&operations, first_root_write(&operations));
                    image.apply(&operations[..persisted]);
                    this_attempt = Some(RollbackAttempt::Mounted {
                        new_instance: mounted.output.instance,
                        witness_entries_written: mounted
                            .output
                            .rollback_witness_written
                            .entries()
                            .len(),
                    });
                    break;
                }
                Err(MountError::RollbackTargetNotACandidate { .. }) => {}
                Err(other) => {
                    this_attempt = Some(RollbackAttempt::Refused(other));
                    break;
                }
            }
        }
        let this_attempt = this_attempt.expect("最新的根恒是候选");
        let refused = matches!(this_attempt, RollbackAttempt::Refused(_));
        attempts.push(this_attempt);
        if refused {
            break;
        }
    }
    attempts
}

/// 代码轮第二轮判决 Z9-A 的 A1 那段历史（攻方 `research/prompts/m2-final-code-r2-opus-model/tests/r2_opus_z9.rs` 的 `z9_a1`）：
/// 覆盖写到 txg 30，之后连着 24 次回退，目标一直是 (1, 29)（它离开根环之后改回退到最新的根），每一次都崩在写行那次的轮换之后、暖机之前。
/// 删除规则加「被同一张表里另一条罩住的删」（D23（journal 的角色与格式） 已定项 14「回退见证的实现取法」①）之后，
/// 同一个目标的 (k, 1, 29) 被后来的 (k + 1, 1, 29) 罩住就删，见证表恒至多两条，24 次回退一次都不写满。
/// 只有第一条删除规则时这段历史在第 24 次写满（取号之前拒，`RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 }`）。
#[test]
fn rolling_back_to_the_same_target_twenty_four_times_with_every_mount_crashing_after_the_row_publish_rotation_never_fills_the_witness_table(
) {
    let attempts = rollbacks_each_crashing_right_after_the_row_publish_rotation(
        "witness-z9-a1-same-target",
        RollbackTargetChoice::TheSameTargetWhileItIsACandidateThenTheNewestRoot(RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(29),
        }),
        24,
    );
    assert_eq!(attempts.len(), 24, "{attempts:#?}");
    for (position, attempt) in attempts.iter().enumerate() {
        match attempt {
            RollbackAttempt::Mounted {
                new_instance,
                witness_entries_written,
            } => {
                assert_eq!(
                    *new_instance,
                    InstanceGeneration(u32::try_from(position + 2).expect("次数")),
                    "第 {} 次回退取的号：每次回退一个新号",
                    position + 1
                );
                assert!(
                    *witness_entries_written <= 2,
                    "第 {} 次回退写出的见证表 {witness_entries_written} 条：被罩住的删掉之后同一个目标至多留两条",
                    position + 1
                );
            }
            RollbackAttempt::Refused(error) => {
                panic!("第 {} 次回退被拒：{error:?}", position + 1)
            }
        }
    }
}

/// 代码轮第二轮判决 Z9-A 的 A2 那段历史（攻方 `z9_a2`）：覆盖写到 txg 30，之后每一次都回退到最新的根（空回退），每一次都崩在写行那次的轮换之后。
/// 每一次的目标比上一次的新，(k + 1, k, T) 罩不住 (k, k − 1, T − 1)，删除规则两条都删不掉；实例 1 那条根一直在根环里，第一条删除规则也不删。
/// 第 24 次在取号之前被拒、返回写满那个成员（判决第四节第 4 条：这一格是已知的，C547（回退见证表的删除规则与写满没有条款） 记着），
/// 前 23 次都做成、第 k 次写出 k 条。
#[test]
fn rolling_back_to_the_newest_root_every_time_with_every_mount_crashing_after_the_row_publish_rotation_still_fills_the_witness_table_on_the_twenty_fourth(
) {
    let attempts = rollbacks_each_crashing_right_after_the_row_publish_rotation(
        "witness-z9-a2-newest-root",
        RollbackTargetChoice::TheNewestRoot,
        24,
    );
    assert_eq!(attempts.len(), 24, "{attempts:#?}");
    for (position, attempt) in attempts[..23].iter().enumerate() {
        assert!(
            matches!(
                attempt,
                RollbackAttempt::Mounted { witness_entries_written, .. }
                    if *witness_entries_written == position + 1
            ),
            "第 {} 次回退做成、写出 {} 条：{attempt:?}",
            position + 1,
            position + 1
        );
    }
    assert!(
        matches!(
            &attempts[23],
            RollbackAttempt::Refused(MountError::Recovery(
                RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided {
                    entries: 24,
                    capacity: 23,
                }
            ))
        ),
        "第 24 次写满、在取号之前拒：{:?}",
        attempts[23]
    );
}

/// 代码轮第二轮判决 Z9-B（攻方 `z9_b_crash_between_row_root_and_its_rotation_never_gets_the_witness`）：A、B、C（实例 1）之后回退到 (1, 3)，
/// 回退那一次挂载崩在写行那次的根 FUA 落盘之后、系统配置轮换之前——回退行 (1, 3, 0, 回退) 已经在新根 (2, 6) 指着的实例表里，
/// 见证条目 (2, 1, 3) 一次都没落盘。之后照常可写挂载 m 次，再让比被抛弃的 C（1, 5）新的根全读不出，看恢复落在哪。
/// - m = 1..3：每次可写挂载在删除规则之前按所选根实例表里的回退行把缺的条目补回来（D23（journal 的角色与格式） 已定项 14「回退见证的
///   实现取法」⑥：N 取回退行之后第一条 T ≠ 0 的行的实例，没有就是所选根自己的实例 2），见证里有 (2, 1, 3)，恢复跳过被抛弃的 B、C，
///   落到 R_old (1, 3)、读回 A。不补的实现在这三格落到 C。
/// - m = 0：见证与回退行都在「写行的根落了、轮换还没落」那一格之后才有得补，一次挂载都没有就补不到，恢复照旧落到被抛弃的 C——
///   这是用户选 post 写序时认下的窗口（`research/prompts/m2-witness-r1-main-verification.md`），这里钉住它今天的样子。
#[test]
fn after_a_rollback_mount_crashed_between_the_row_root_and_its_rotation_each_later_writable_mount_restores_the_missing_witness(
) {
    for later_writable_mounts in 0..=3usize {
        let (pool, _third) =
            pool_through_the_third_version(&format!("witness-z9-b-{later_writable_mounts}"));
        let mut image = pool.memory_pool();
        drop(pool);
        let stream = SharedStream::retaining_contents();
        let mut devices = crash_state_devices(&image, &[], &[], &stream);
        mount_rollback(
            &parameters(),
            &mut devices,
            ROLLBACK_TARGET,
            ShadowLedger::On,
        )
        .expect("回退到 (1, 3)");
        let operations = stream.retained_operations();
        image.apply(&operations[..=first_root_write(&operations)]);
        // 迭代上界是 later_writable_mounts 次挂载；每一次整次写完、施加到镜像上。
        for _mount in 0..later_writable_mounts {
            let mount_stream = SharedStream::retaining_contents();
            let mut mount_devices = crash_state_devices(&image, &[], &[], &mount_stream);
            mount_writable(&parameters(), &mut mount_devices).expect("可写挂载");
            image.apply(&mount_stream.retained_operations());
        }
        let system_configuration = choose_system_configuration(&image).expect("系统配置");
        let witness = rollback_witness_of_the_pool(&image, &system_configuration).entries();
        let newer_than_the_abandoned_c: Vec<RootRingSlot> = readable_roots(
            &image,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        )
        .into_iter()
        .filter(|root| {
            (root.checkpoint_txg, root.instance) > (CheckpointTxg(5), InstanceGeneration(1))
        })
        .map(|root| {
            target_for_publish(
                root.checkpoint_txg,
                parameters().geometry.root_ring_slots_per_region,
            )
        })
        .collect();
        assert!(
            !newer_than_the_abandoned_c.is_empty(),
            "m = {later_writable_mounts}：回退之后的根至少有写行那一条"
        );
        let unreadable = PoolReaderWithUnreadableRootRingSlots::new(
            &image,
            root_ring_slot_target(&newer_than_the_abandoned_c),
        );
        let outcome = recover(&unreadable, JournalPolicy::Consult).outcome;
        if later_writable_mounts == 0 {
            assert_eq!(
                witness,
                Vec::new(),
                "m = 0：见证那一次写没落盘，也还没有一次挂载补它"
            );
            assert!(
                matches!(
                    &outcome,
                    RecoveryOutcome::FileRead { root, .. }
                        if *root == (InstanceGeneration(1), CheckpointTxg(5))
                ),
                "m = 0：post 写序认下的窗口，恢复照旧落到被抛弃的 C（1, 5）：{outcome:?}"
            );
        } else {
            assert_eq!(
                witness,
                vec![THE_ROLLBACK],
                "m = {later_writable_mounts}：可写挂载按回退行补回了 (2, 1, 3)"
            );
            assert_eq!(
                outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(1), CheckpointTxg(3)),
                    content: file_content(),
                },
                "m = {later_writable_mounts}：恢复跳过被抛弃的 B、C，落到 R_old（1, 3）读回 A"
            );
        }
    }
}

/// 补见证的 N 取「回退行之后第一条 T ≠ 0 的行的实例」，不取「回退行之后第一个有行的实例」：回退跨过实例的那一格。
/// A、B、C（实例 1），重开可写挂载（取号 2、写行、暖机），再覆盖写一版 D（实例 2）；之后回退到 (1, 3)，新实例 3 写行 (1, 3, 0, 回退)
/// 与中间实例 (2, 0, 0)，崩在写行那次的根 FUA 之后、轮换之前；再可写挂载一次。补回来的条目是 (3, 1, 3)——它抛弃实例 2 的根；
/// 照字面取第一个有行的实例 2，补出来的 (2, 1, 3) 罩不住实例 2 的根，D 之后的根全读不出时恢复落到被抛弃的 D。
#[test]
fn the_witness_restored_after_a_rollback_across_instances_names_the_instance_that_rolled_back_not_the_intermediate_one(
) {
    let mut pool = pool_through_the_third_version("witness-restored-across-instances").0;
    let mut devices_of_the_remount = pool.reopen_recorded();
    let remounted =
        mount_writable(&parameters(), &mut devices_of_the_remount).expect("重开可写挂载（实例 2）");
    pool.devices = Some(devices_of_the_remount);
    pool.allocator = remounted.allocator;
    let current = remounted
        .current
        .into_file_version()
        .expect("C 之后重开，现行那一版带文件");
    let fourth = publish_overwrite_in_process(
        &mut pool,
        &current,
        &content_of(3),
        FIXED_WRITE_TIME_SECONDS + 180,
        InstanceGeneration(2),
    )
    .expect("实例 2 的覆盖写");
    let abandoned_root = (fourth.root.instance, fourth.root.checkpoint_txg);
    let mut image = pool.memory_pool();
    drop(pool);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        ROLLBACK_TARGET,
        ShadowLedger::On,
    )
    .expect("回退到 (1, 3)");
    assert_eq!(rolled_back.output.instance, InstanceGeneration(3));
    let operations = stream.retained_operations();
    image.apply(&operations[..=first_root_write(&operations)]);
    let mount_stream = SharedStream::retaining_contents();
    let mut mount_devices = crash_state_devices(&image, &[], &[], &mount_stream);
    mount_writable(&parameters(), &mut mount_devices).expect("可写挂载（实例 4）");
    image.apply(&mount_stream.retained_operations());
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    assert_eq!(
        rollback_witness_of_the_pool(&image, &system_configuration).entries(),
        vec![RollbackWitnessEntry {
            new_instance: InstanceGeneration(3),
            ..THE_ROLLBACK
        }],
        "补回来的是做那次回退的实例 3，不是中间实例 2"
    );
    let newer_than_the_abandoned_root: Vec<RootRingSlot> = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| (root.checkpoint_txg, root.instance) > (abandoned_root.1, abandoned_root.0))
    .map(|root| {
        target_for_publish(
            root.checkpoint_txg,
            parameters().geometry.root_ring_slots_per_region,
        )
    })
    .collect();
    let unreadable = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        root_ring_slot_target(&newer_than_the_abandoned_root),
    );
    assert_eq!(
        recover(&unreadable, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content(),
        },
        "实例 2 的根被 (3, 1, 3) 抛弃：恢复落到 R_old（1, 3）读回 A"
    );
}
