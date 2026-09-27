//! 里程碑「覆盖写、释放、回退与复用」步 4 的验收：管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14）。
//! 固定脚本到 C：A（实例 1，txg 3）、B（txg 4）、重开取号 2（写行 txg 5、暖机 txg 6、7）、C（txg 8）。挂着的时候回退到 A 的根 (1, 3)：
//! 发一次普通发布 D（txg 9，实例仍是 2）——新根的 inode 树、extent 树取 A 的，树表、分配记录树、中央映射树与记账统计量按调整之后的分配器现算，
//! 实例表照 C 的；C 引用、D 不引用的用户可见单元释放（释放代 9），A 引用而 C 的账记成已释放的改回已分配（分配代写回 3）。
//! 不取号、不写实例表行、不施加也不删除任何记录，B、C 不被抛弃。
//!
//! 同一个文件里还有：影子账（D23 已定项 14 射程：理由是崩溃恢复抛弃的时间线）的用例——被抛弃的根由崩溃恢复造出；
//! 与 C558（回退目标是环里最旧的根时回退那次发布会写坏它） 判可达性的用例。

mod common;
mod common_corrupted_allocation_record;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, disk_snapshot,
    file_content, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use common_corrupted_allocation_record::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::allocator::{
    unit_area_slots_of_device, AllocationRecord, DeviceFreeMap, PoolAllocator,
};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    is_a_user_visible_unit, mount_writable, mount_writable_with_test_only_switches,
    roll_back_by_a_forward_publish, Mounted, RollbackCandidateExclusion, RollbackError,
    RollbackTarget, RolledBack, ShadowLedger,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::TREE_KIND_ALLOCATION;
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration,
    highest_tree_identifier_watermark_in_the_ring, readable_roots, recover, scan_journal,
    user_visible_tree_root_pointers, walk_to_file, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, resend_the_frozen_publish, FirstFile, PoolWriter,
    PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_format::UNIT_AREA_START_SLOT;
use singlefs_harness::fault_injection::{
    injected_block_device_error, NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots,
    RootRingSlotTarget,
};
use singlefs_harness::memory_pool::MemoryPool;
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C 的池，连同 A、B、C 三版（进程内存里的，D 之前的样子）。
struct ThroughTheThirdPublish {
    pool: BuiltPool,
    first: TransactionOutput,
    third: TransactionOutput,
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> ThroughTheThirdPublish {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    ThroughTheThirdPublish { pool, first, third }
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

fn third_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
    }
}

/// 挂着的时候回退：交回回退的结局，做成时 `pool.output` 已换成回退那次发布之后的一版。
fn roll_back(pool: &mut BuiltPool, target: RollbackTarget) -> Result<RolledBack, RollbackError> {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &publish_parameters,
        devices,
        &mut pool.allocator,
        &mut pool.output,
        target,
    )
}

/// 池级 checker 一条违例都没有；交回全部判定，调用方再点名要真被判过（不是「不适用」）的那几条。
fn verdicts_without_any_violation(
    image: &MemoryPool,
    step: &str,
) -> Vec<(&'static str, InvariantVerdict)> {
    let verdicts = check_pool_image(image);
    let violated: Vec<(&str, &InvariantVerdict)> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, verdict)| (*invariant, verdict))
        .collect();
    assert!(
        violated.is_empty(),
        "{step}：池级 checker 一条违例都没有：{violated:?}"
    );
    verdicts
}

fn assert_judged_and_holding(
    verdicts: &[(&'static str, InvariantVerdict)],
    invariants: &[&str],
    step: &str,
) {
    for invariant in invariants {
        assert!(
            verdicts
                .iter()
                .any(|(name, verdict)| name == invariant && *verdict == InvariantVerdict::Holds),
            "{step}：{invariant} 真被判过且成立：{:?}",
            verdicts.iter().find(|(name, _)| name == invariant)
        );
    }
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一条根槽在盘上的位置：哪块盘、哪个偏移。槽距是 `fixed_structure_slot_spacing`，槽自己的判定宽度是 `physical_block_size`。
fn root_slot_position_of(txg: u64) -> (DeviceIdentity, DeviceOffsetInBytes) {
    (
        region_device(txg),
        slot_offset(
            target_for_publish(
                CheckpointTxg(txg),
                parameters().geometry.root_ring_slots_per_region,
            ),
            parameters().geometry.fixed_structure_slot_spacing,
        ),
    )
}

fn root_slot_bytes() -> usize {
    usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")
}

fn read_root_slot(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    (device, offset): (DeviceIdentity, DeviceOffsetInBytes),
) -> Vec<u8> {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("这条根的根槽所在的盘");
    let mut slot_bytes = vec![0u8; root_slot_bytes()];
    recorded.read_at(offset, &mut slot_bytes).expect("读根槽");
    slot_bytes
}

fn write_root_slot(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    (device, offset): (DeviceIdentity, DeviceOffsetInBytes),
    slot_bytes: &[u8],
) {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("这条根的根槽所在的盘");
    recorded
        .write_at(offset, slot_bytes, WriteDurability::Plain)
        .expect("写根槽");
}

/// 读一个单元在某块盘上的那一份。
fn read_unit_at(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    location: LocationEntry,
    unit_bytes: usize,
) -> Vec<u8> {
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == location.device)
        .expect("这个单元所在的盘");
    let mut bytes = vec![0u8; unit_bytes];
    recorded
        .read_at(location.slot.to_device_offset(), &mut bytes)
        .expect("读单元");
    bytes
}

fn write_unit_to_every_location(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    locations: [LocationEntry; 2],
    bytes: &[u8],
) {
    for location in locations {
        let (_, recorded) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("这个单元所在的盘");
        recorded
            .write_at(
                location.slot.to_device_offset(),
                bytes,
                WriteDurability::Plain,
            )
            .expect("写单元");
    }
}

/// 把一个单元在 `locations` 各份上翻一个字节（改坏，不重封）。
fn corrupt_unit_copies(
    devices: &mut [(DeviceIdentity, impl BlockDevice)],
    locations: &[LocationEntry],
    unit_bytes: usize,
) {
    for location in locations {
        let mut bytes = read_unit_at(devices, *location, unit_bytes);
        bytes[200] ^= 0xff;
        let (_, device) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("这个单元所在的盘");
        device
            .write_at(
                location.slot.to_device_offset(),
                &bytes,
                WriteDurability::Plain,
            )
            .expect("改坏一份单元");
    }
}

/// 一版的用户可见单元在那一版里的落点（第一条位置条目的槽；两盘同槽），按角色。
fn user_visible_slots(version: &TransactionOutput) -> BTreeSet<(TransactionUnit, u64)> {
    version
        .units
        .iter()
        .filter(|unit| is_a_user_visible_unit(unit.identity))
        .map(|unit| (unit.identity, unit.slot.0))
        .collect()
}

fn record_of(
    records: &[AllocationRecord],
    device: DeviceIdentity,
    slot: u64,
) -> Option<AllocationRecord> {
    records
        .iter()
        .copied()
        .find(|record| record.device == device && record.slot.0 == slot)
}

/// 验收第一条：回退到 A 的根 (1, 3) 是一次普通发布 D (2, 9)——jsn 接着 C，只重写固定点单元（不写数据单元、不写实例表）；
/// D 的实例表指针就是 C 的，两棵用户可见树的根指针与 A 树表里的逐字节相同；C 的四个用户可见单元释放（释放代 9），
/// A 的四个在 C 的账里已释放的逐盘改回已分配（分配代 3）；水位照 C 的；系统配置的实例代号仍是 2；冷启动择 D 读回第一次的内容；
/// checker 全绿，I-3.1、I-3.9、I-3.11 真被判过。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content(
) {
    let ThroughTheThirdPublish {
        mut pool,
        first,
        third,
        ..
    } = build_through_third_publish("step-four-forward");
    let rolled_back = roll_back(&mut pool, first_root()).expect("回退到 A");
    let rollback_version = pool.output.clone();
    assert_eq!(
        (
            rollback_version.root.instance,
            rollback_version.root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(9)),
        "D 接着 C：txg 9、实例仍是 2（不取号）"
    );
    assert_eq!(
        rollback_version.record.counter,
        third.record.counter + 1,
        "jsn 接着 C 那一条"
    );
    assert_eq!(
        rollback_version.record.transaction, 0,
        "D 不写数据单元，事务号写 0（同空发布）"
    );
    assert!(
        rollback_version
            .rewritten
            .iter()
            .all(|role| !is_a_user_visible_unit(*role) && !role.is_a_page_of_the_instance_table()),
        "D 只重写固定点单元：{:?}",
        rollback_version.rewritten
    );
    assert_eq!(
        rollback_version.root.instance_table, third.root.instance_table,
        "实例表照 C 的：同一条指针、同一份字节"
    );
    let image = pool.memory_pool();
    assert_eq!(
        user_visible_tree_root_pointers(&image, &rollback_version.root).expect("D 的树表"),
        user_visible_tree_root_pointers(&image, &first.root).expect("A 的树表"),
        "D 树表里 inode 树与 extent 树的根指针与 A 的逐字节相同"
    );
    assert_eq!(
        user_visible_slots(&rollback_version),
        user_visible_slots(&first),
        "D 的用户可见单元就是 A 的"
    );
    let released: BTreeSet<(TransactionUnit, u64)> = rolled_back
        .released_user_visible_units
        .iter()
        .map(|(unit, placement)| (*unit, placement.slot.0))
        .collect();
    assert_eq!(
        released,
        user_visible_slots(&third),
        "释放集：C 引用、D 不引用的用户可见单元（A 与 C 一个都不共用）"
    );
    assert!(rolled_back.quarantined_user_visible_copies.is_empty());
    let expected_resurrected: BTreeSet<(DeviceIdentity, u64, CheckpointTxg)> =
        user_visible_slots(&first)
            .into_iter()
            .flat_map(|(_, slot)| {
                [DeviceIdentity(0), DeviceIdentity(1)]
                    .map(|device| (device, slot, CheckpointTxg(3)))
            })
            .collect();
    let resurrected: BTreeSet<(DeviceIdentity, u64, CheckpointTxg)> = rolled_back
        .resurrected_user_visible_copies
        .iter()
        .map(|copy| {
            (
                copy.device,
                copy.placement.slot.0,
                copy.allocation_generation,
            )
        })
        .collect();
    assert_eq!(
        resurrected, expected_resurrected,
        "复活集：A 的四个用户可见单元在 C 的账里都已释放（B 换下的），逐盘改回、分配代写回 3"
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for (_, slot) in user_visible_slots(&first) {
            let record = record_of(&rollback_version.allocation_records, device, slot)
                .expect("A 的单元在 D 的账里有记录");
            assert!(
                !record.is_released && record.generation == CheckpointTxg(3),
                "D 的账：A 的单元仍分配、分配代 3：{record:?}"
            );
        }
        for (_, slot) in user_visible_slots(&third) {
            let record = record_of(&rollback_version.allocation_records, device, slot)
                .expect("C 的单元在 D 的账里有记录");
            assert!(
                record.is_released && record.generation == CheckpointTxg(9),
                "D 的账：C 的单元已释放、释放代 9：{record:?}"
            );
        }
    }
    assert_eq!(
        rollback_version.inode_number_watermark(),
        third.inode_number_watermark(),
        "inode 号水位 = max(C 内存里的, 环里的)"
    );
    assert_eq!(
        rollback_version.root.tree_identifier_watermark,
        third.root.tree_identifier_watermark
    );
    assert_eq!(
        choose_system_configuration(&image)
            .expect("系统配置")
            .quantities
            .journal_instance,
        InstanceGeneration(2),
        "系统配置的实例代号仍是 2"
    );
    let verdicts = verdicts_without_any_violation(&image, "D 之后");
    assert_judged_and_holding(&verdicts, &["I-3.1", "I-3.9", "I-3.11", "I-9.6"], "D 之后");
    let cold = pool.reopen_cold();
    let report = recover(&cold, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(9)),
            content: file_content()
        },
        "冷启动择 D、读回第一次的内容"
    );
}

/// 验收「释放与复活」：D 之后接着覆盖写 E，E 换下的是 D 引用的 A 的那几个单元——它们在 D 的账里是仍分配的（复活过），
/// 发布路径照常释放；checker 全绿。只释放不复活时 D 的账里 A 的单元是已释放的：I-3.11 在 D 上红，E 被释放判定路径拒
/// （`ReleaseTargetAlreadyReleased`，第一轮判决第二节 F1 里 H3 那一格）——`crates/mutations.tsv` 里拿掉复活那一步的那一行证它。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds(
) {
    let ThroughTheThirdPublish { mut pool, .. } =
        build_through_third_publish("step-four-forward-then-overwrite");
    roll_back(&mut pool, first_root()).expect("回退到 A");
    let after_the_rollback = pool.memory_pool();
    let verdicts_after_the_rollback = verdicts_without_any_violation(&after_the_rollback, "D 之后");
    assert_judged_and_holding(&verdicts_after_the_rollback, &["I-3.9", "I-3.11"], "D 之后");
    let fourth_content = content_of(3300, 17);
    let fourth = overwrite_in_process(&mut pool, &fourth_content, InstanceGeneration(2));
    assert_eq!(fourth.root.checkpoint_txg, CheckpointTxg(10));
    let verdicts_after_the_next_overwrite =
        verdicts_without_any_violation(&pool.memory_pool(), "E 之后");
    assert_judged_and_holding(
        &verdicts_after_the_next_overwrite,
        &["I-3.9", "I-3.11"],
        "E 之后",
    );
    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&cold, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(10)),
            content: fourth_content
        }
    );
}

/// 验收「D 之后再回退到 C 的根」：向前发布不抛弃 B、C，D 之后 C (2, 8) 仍是候选；再回退一次（txg 10）读回第三次的内容，checker 全绿。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content(
) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-forward-back-to-c");
    roll_back(&mut pool, first_root()).expect("回退到 A");
    roll_back(&mut pool, third_root()).expect("D 之后再回退到 C");
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(10));
    assert_eq!(user_visible_slots(&pool.output), user_visible_slots(&third));
    verdicts_without_any_violation(&pool.memory_pool(), "回退到 C 之后");
    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&cold, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(10)),
            content: third_content()
        }
    );
}

/// 崩溃恢复造出被抛弃的根（D23（journal 的角色与格式） 已定项 14 射程：影子账与按实例表判抛弃留着，理由是崩溃恢复——
/// 第一轮判决 H6 那一形）：C 之后进程退出，C 的根槽与 C 的数据单元（两份）暂时读不出（暂存、清零），重开可写挂载——择根落到
/// (2, 7)，C 那条记录施加前验点名单元失败、不施加；新实例 3 写行与暖机（txg 9、10）。再把暂存的字节原样写回：C 的根 (2, 8)
/// 又读得出，按实例 3 的实例表判是被抛弃的。交回那次挂载——它看不见 C，影子账隔离 0；之后的挂载才看得见 C。
fn abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(
    pool: &mut BuiltPool,
    third: &TransactionOutput,
) -> Mounted {
    let mounted = abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(pool, third);
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(7)),
        "择根落到 (2, 7)，C 那条记录不施加"
    );
    assert_eq!(mounted.output.instance, InstanceGeneration(3));
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(10),
        "实例 3 写行 txg 9、暖机 txg 10"
    );
    mounted
}

/// 重开可写挂载一次，影子账按 `shadow_ledger`；池换成挂载交回的分配器与现行版本，交回那次挂载。
fn remount_with_the_shadow_ledger(pool: &mut BuiltPool, shadow_ledger: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable_with_test_only_switches(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        shadow_ledger,
    )
    .expect("重开可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone();
    mounted
}

/// 验收「候选集」：根环里没有的 (2, 42)、被崩溃恢复抛弃的时间线上的 (2, 8)、树表 0 条的暖机根 (1, 2) 都在任何写之前拒，
/// 报各自那一条排除；盘上逐字节不变（系统配置四槽、根环、录制流一步不多），分配器与现行版本不动。txg 低于 F_生效 那一格在
/// `rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`（F 只住系统配置时同样拒）。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write(
) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-forward-candidates");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    for (target, expected_exclusion) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(42),
            },
            RollbackCandidateExclusion::NotInRing,
        ),
        (
            third_root(),
            RollbackCandidateExclusion::OnAbandonedTimeline,
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(2),
            },
            RollbackCandidateExclusion::VersionWithoutFile,
        ),
    ] {
        let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
        let current_before = pool.output.clone();
        let records_before = pool.allocator.records().to_vec();
        match roll_back(&mut pool, target) {
            Err(RollbackError::TargetNotACandidate {
                target: refused,
                exclusion,
            }) => {
                assert_eq!(refused, target);
                assert_eq!(exclusion, expected_exclusion, "{target:?}");
            }
            other => panic!("回退到 {target:?} 要拒成 {expected_exclusion:?}，交回 {other:?}"),
        }
        assert_eq!(
            disk_snapshot(&pool.memory_pool(), &pool.stream),
            before,
            "{target:?}：拒之前盘上一个字节都不动"
        );
        assert_eq!(pool.output, current_before);
        assert_eq!(pool.allocator.records(), records_before.as_slice());
    }
    roll_back(&mut pool, first_root()).expect("A (1, 3) 仍是候选：实例 1 那一行是 (1, 4, ·)");
    verdicts_without_any_violation(&pool.memory_pool(), "崩溃恢复抛弃 C 之后回退到 A");
}

/// 验收「拒」：在任何写之前拒的几种，各一条，拒之后盘上逐字节不变、分配器与现行版本不动——
/// A 引用的单元在 C 的账里没有记录；仍分配而分配代不同；A 那一版的账有一个节点读不出；C 那一版的账有一个节点读不出；
/// 复活集里一个单元在一块盘上的那一份校验和对不上；两版的树号不同（条款没写，第一版不支持）。
/// 前两格的账按内存里那一份判（判定与之后的复活、释放读同一份），用例把分配器换成少一条、改了分配代的一份来造；
/// 后几格改盘上的字节。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_current_account_without_the_data_unit_of_the_first_version_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::CurrentAccountWithoutTheDataUnitOfTheFirstVersion,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_current_account_allocating_the_data_slot_of_the_first_version_under_another_generation_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::CurrentAccountAllocatingTheDataSlotOfTheFirstVersionUnderAnotherGeneration,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_allocation_record_tree_root_of_the_first_version_unreadable_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::AllocationRecordTreeRootOfTheFirstVersionUnreadable,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_allocation_record_tree_root_of_the_third_version_unreadable_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::AllocationRecordTreeRootOfTheThirdVersionUnreadable,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_data_unit_of_the_first_version_corrupt_on_device_one_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::DataUnitOfTheFirstVersionCorruptOnDeviceOne,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_tree_identifiers_of_the_current_version_differ_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::TreeIdentifiersOfTheCurrentVersionDiffer,
    );
}

/// 验收「拒」那几种在任何写之前拒的破坏，一种一条用例。
#[derive(Clone, Copy, Debug)]
enum RefusalBreakage {
    CurrentAccountWithoutTheDataUnitOfTheFirstVersion,
    CurrentAccountAllocatingTheDataSlotOfTheFirstVersionUnderAnotherGeneration,
    AllocationRecordTreeRootOfTheFirstVersionUnreadable,
    AllocationRecordTreeRootOfTheThirdVersionUnreadable,
    DataUnitOfTheFirstVersionCorruptOnDeviceOne,
    TreeIdentifiersOfTheCurrentVersionDiffer,
}

fn refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
    breakage: RefusalBreakage,
) {
    let ThroughTheThirdPublish {
        mut pool,
        first,
        third,
        ..
    } = build_through_third_publish(&format!("step-four-forward-refusal-{breakage:?}"));
    let data_slot_of_the_first_version = first.data_pointers[0].locations[0].slot;
    let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
    let data_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let records_of_the_current_account = pool.allocator.records().to_vec();
    let crafted_account = |edit: &dyn Fn(&mut Vec<AllocationRecord>)| {
        let mut records = records_of_the_current_account.clone();
        edit(&mut records);
        PoolAllocator::rebuild_from_records(
            vec![
                DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
                DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
            ],
            records,
        )
    };
    match breakage {
        RefusalBreakage::CurrentAccountWithoutTheDataUnitOfTheFirstVersion => {
            pool.allocator = crafted_account(&|records| {
                records.retain(|record| record.slot != data_slot_of_the_first_version);
            });
        }
        RefusalBreakage::CurrentAccountAllocatingTheDataSlotOfTheFirstVersionUnderAnotherGeneration => {
            pool.allocator = crafted_account(&|records| {
                for record in records.iter_mut() {
                    if record.slot == data_slot_of_the_first_version {
                        record.is_released = false;
                        record.generation = CheckpointTxg(7);
                    }
                }
            });
        }
        RefusalBreakage::AllocationRecordTreeRootOfTheFirstVersionUnreadable => {
            let pointer = first
                .allocation_record_tree
                .pointer_of(AllocationRecordTreeNode::Root)
                .expect("A 的分配记录树有根");
            let devices = pool.devices.as_mut().expect("镜像还开着");
            corrupt_unit_copies(devices, &pointer.locations, node_bytes);
        }
        RefusalBreakage::AllocationRecordTreeRootOfTheThirdVersionUnreadable => {
            let pointer = third
                .allocation_record_tree
                .pointer_of(AllocationRecordTreeNode::Root)
                .expect("C 的分配记录树有根");
            let devices = pool.devices.as_mut().expect("镜像还开着");
            corrupt_unit_copies(devices, &pointer.locations, node_bytes);
        }
        RefusalBreakage::DataUnitOfTheFirstVersionCorruptOnDeviceOne => {
            let device_one_copy = first.data_pointers[0]
                .locations
                .iter()
                .copied()
                .find(|location| location.device == DeviceIdentity(1))
                .expect("A 的数据单元在盘 1 上有一份");
            let devices = pool.devices.as_mut().expect("镜像还开着");
            corrupt_unit_copies(devices, &[device_one_copy], data_bytes);
        }
        RefusalBreakage::TreeIdentifiersOfTheCurrentVersionDiffer => {
            pool.output.tree_identifiers.extent = TreeIdentifier(999);
        }
    }
    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    let current_before = pool.output.clone();
    let records_before = pool.allocator.records().to_vec();
    let refusal = roll_back(&mut pool, first_root()).expect_err("在任何写之前拒");
    let is_the_expected_refusal = match (breakage, &refusal) {
        (
            RefusalBreakage::CurrentAccountWithoutTheDataUnitOfTheFirstVersion,
            RollbackError::UserVisibleUnitWithoutItsRecordInTheCurrentAccount {
                unit: TransactionUnit::Data(_),
                slot,
                ..
            },
        ) => *slot == data_slot_of_the_first_version,
        (
            RefusalBreakage::CurrentAccountAllocatingTheDataSlotOfTheFirstVersionUnderAnotherGeneration,
            RollbackError::UserVisibleUnitStillAllocatedUnderAnotherGeneration {
                unit: TransactionUnit::Data(_),
                current_account_generation,
                target_account_generation,
                ..
            },
        ) => {
            *current_account_generation == CheckpointTxg(7)
                && *target_account_generation == CheckpointTxg(3)
        }
        (
            RefusalBreakage::AllocationRecordTreeRootOfTheFirstVersionUnreadable,
            RollbackError::TargetVersionUnreadable { target, .. },
        ) => *target == first_root(),
        (
            RefusalBreakage::AllocationRecordTreeRootOfTheThirdVersionUnreadable,
            RollbackError::CurrentAccountUnreadable { .. },
        ) => true,
        (
            RefusalBreakage::DataUnitOfTheFirstVersionCorruptOnDeviceOne,
            RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {
                unit: TransactionUnit::Data(_),
                device,
                slot,
            },
        ) => *device == DeviceIdentity(1) && *slot == data_slot_of_the_first_version,
        (
            RefusalBreakage::TreeIdentifiersOfTheCurrentVersionDiffer,
            RollbackError::TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided {
                target,
            },
        ) => *target == first_root(),
        _ => false,
    };
    assert!(is_the_expected_refusal, "{breakage:?}：交回 {refusal:?}");
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "{breakage:?}：拒之前盘上一个字节都不动"
    );
    assert_eq!(pool.output, current_before, "{breakage:?}");
    assert_eq!(
        pool.allocator.records(),
        records_before.as_slice(),
        "{breakage:?}：分配器不动"
    );
}

/// 验收「水位」：C 那一版新建了三个 inode（号 2、3、4，水位 5），回退那一刻 C 的根槽读不出——环里读得出的根带的 inode 号水位都是 2；
/// D 的 inode 号水位取 max(C 内存里的 5, 环里的 2) = 5，D 之后再新建的 inode 从 5 起、不重发 2–4。树 ID 水位同一个取法。
/// 只取环里读得出的根时 D 的水位是 2——`crates/mutations.tsv` 里那一行证这条断言红。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_rollback_takes_the_watermarks_from_the_current_version_in_memory_when_its_root_is_unreadable(
) {
    let mut pool = build_pool("step-four-forward-watermark");
    let _second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut reopened = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut reopened).expect("可写挂载");
    pool.devices = Some(reopened);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third = {
        let publish_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let previous = pool.output.clone();
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            3,
            FIXED_WRITE_TIME_SECONDS + 120,
            InstanceGeneration(2),
        )
        .expect("C 新建三个 inode")
    };
    pool.output = third.clone();
    assert_eq!(third.root.checkpoint_txg, CheckpointTxg(8));
    assert_eq!(third.inode_number_watermark(), 5);
    let third_root_slot = root_slot_position_of(8);
    let saved_third_root_slot = {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let saved = read_root_slot(open_devices, third_root_slot);
        write_root_slot(open_devices, third_root_slot, &vec![0u8; root_slot_bytes()]);
        saved
    };
    roll_back(&mut pool, first_root()).expect("回退到 A：C 的根读不出不碍事，候选与账都读得出");
    assert_eq!(
        pool.output.inode_number_watermark(),
        5,
        "D 的 inode 号水位 = max(C 内存里的 5, 环里读得出的根的 2)"
    );
    assert_eq!(
        pool.output.root.tree_identifier_watermark, third.root.tree_identifier_watermark,
        "树 ID 水位同一个取法"
    );
    let after_the_rollback = {
        let publish_parameters = parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let previous = pool.output.clone();
        publish_new_inodes(
            &mut writer,
            &mut pool.allocator,
            &previous,
            2,
            FIXED_WRITE_TIME_SECONDS + 180,
            InstanceGeneration(2),
        )
        .expect("D 之后新建两个 inode")
    };
    let inode_numbers: BTreeSet<u64> = after_the_rollback
        .inode_leaf_containers
        .iter()
        .flat_map(|container| container.contents.records.iter().map(|record| record.inode))
        .collect();
    assert_eq!(
        inode_numbers,
        BTreeSet::from([1, 5, 6]),
        "D 之后新建的号从 5 起：C 发过的 2–4 不重发"
    );
    {
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        write_root_slot(open_devices, third_root_slot, &saved_third_root_slot);
    }
    let verdicts = verdicts_without_any_violation(&pool.memory_pool(), "D、新建两个 inode 之后");
    assert_judged_and_holding(&verdicts, &["I-9.6"], "D、新建两个 inode 之后");
}

/// 被抛弃的根 C 下面走到文件：影子账要护的就是它引用的单元（崩溃恢复落回它的那一格，第一轮判决 H6）。
fn walk_to_the_file_under_the_abandoned_third_version_root(
    reader: &dyn singlefs_core::recovery::PoolReader,
) -> Result<Option<Vec<u8>>, RecoveryFailure> {
    let system_configuration = choose_system_configuration(reader).expect("系统配置");
    let abandoned_third_version_root = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| (root.instance, root.checkpoint_txg) == (InstanceGeneration(2), CheckpointTxg(8)))
    .expect("C 的根 (2, 8) 还在根环里、读得出");
    walk_to_file(reader, &abandoned_third_version_root, &mut 0)
}

/// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入；被抛弃的根由崩溃恢复造出
/// （`abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up`，第一轮判决 H6 那一形）：抛弃 C 的那次挂载看不见 C，
/// 下一次重开才看得见——影子账关着，重开之后下一版的数据单元落回 C 的数据槽，沿被抛弃的根 C 走到文件读不回第三次的内容；
/// 开着，只被 C 引用的 14 个槽隔离，第三次的内容原样读回。两臂报的分支名不同。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn without_the_shadow_ledger_a_publish_after_a_recovery_reuses_the_abandoned_data_slot_and_the_abandoned_root_reads_a_torn_unit(
) {
    publish_after_a_recovery_that_abandoned_the_third_version(ShadowLedger::Off);
}

/// 同一条脚本、影子账开着：只被 C 引用的 14 个槽隔离，下一版的数据单元不落回 C 的数据槽，沿 C 读回第三次的内容。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content(
) {
    publish_after_a_recovery_that_abandoned_the_third_version(ShadowLedger::On);
}

/// 崩溃恢复抛弃 C 之后按给定的影子账开关重开、再覆盖写一次，核隔离的槽数、下一版的数据槽与沿 C 读回的内容。
fn publish_after_a_recovery_that_abandoned_the_third_version(shadow_ledger: ShadowLedger) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish(&format!("step-four-shadow-{shadow_ledger:?}"));
    let data_slot_of_the_third_version = third.data_pointers[0].locations[0].slot.0;
    let abandoning =
        abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    assert_eq!(
        abandoning.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "抛弃 C 的那次挂载看不见 C"
    );
    let remounted = remount_with_the_shadow_ledger(&mut pool, shadow_ledger);
    assert_eq!(
        remounted.output.shadow_ledger_branch,
        shadow_ledger.branch_name(),
        "挂载报出的分支名要与传进去的那一臂相符"
    );
    let expected_isolated = match shadow_ledger {
        ShadowLedger::On => 14,
        ShadowLedger::Off => 0,
    };
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![
            (DeviceIdentity(0), expected_isolated),
            (DeviceIdentity(1), expected_isolated)
        ],
        "{shadow_ledger:?}：只被 C 引用的槽——C 那一次发布写的 14 个"
    );
    let fourth = overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(4));
    let fourth_data_slot = fourth.data_pointers[0].locations[0].slot.0;
    let walked_under_the_abandoned_third_version =
        walk_to_the_file_under_the_abandoned_third_version_root(&pool.memory_pool());
    match shadow_ledger {
        ShadowLedger::Off => {
            assert_eq!(
                fourth_data_slot, data_slot_of_the_third_version,
                "影子账关着：下一版的数据单元落回 C 的数据槽"
            );
            assert_ne!(
                walked_under_the_abandoned_third_version,
                Ok(Some(third_content())),
                "C 的数据单元被盖掉，沿 C 读不回第三次的内容"
            );
        }
        ShadowLedger::On => {
            assert_ne!(fourth_data_slot, data_slot_of_the_third_version);
            assert_eq!(
                walked_under_the_abandoned_third_version,
                Ok(Some(third_content())),
                "影子账开着：沿 C 读回第三次的内容"
            );
        }
    }
}

/// 影子账只住内存，所以每次挂载都重算：崩溃恢复抛弃 C 之后重开一次（实例 4）、覆盖写一次，再普通重开一次（实例 5），
/// 被抛弃根 C 引用的槽照样隔离、重开后的发布一个都不落上去。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_plain_remount_after_a_recovery_that_abandoned_roots_keeps_the_isolation() {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-shadow-remount");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    let first_remount = remount_with_the_shadow_ledger(&mut pool, ShadowLedger::On);
    assert_eq!(first_remount.output.instance, InstanceGeneration(4));
    overwrite_in_process(&mut pool, &content_of(2000, 40), InstanceGeneration(4));
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("普通重开");
    pool.devices = Some(devices);
    pool.allocator = remounted.allocator.clone();
    pool.output = remounted
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone();
    assert_eq!(remounted.output.instance, InstanceGeneration(5));
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "按实例 5 那一版实例表判被抛弃的根 C 引用的 14 个槽，普通重开照样隔离"
    );
    assert_eq!(remounted.output.abandoned_roots_unreadable, 0);
    let slots_of_the_third_version: BTreeSet<u64> = third
        .placements()
        .iter()
        .filter(|placement| {
            pool.output
                .placements()
                .iter()
                .all(|current| current.slot != placement.slot)
        })
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    let next = overwrite_in_process(&mut pool, &content_of(2100, 41), InstanceGeneration(5));
    for placement in next.placements() {
        for slot in placement.slot.0..placement.slot.0 + placement.span {
            assert!(
                !slots_of_the_third_version.contains(&slot),
                "重开后的发布落到了被抛弃根 C 引用的槽 {slot}"
            );
        }
    }
}

/// 被抛弃根 C 的树表单元在两块盘上都改坏：影子账罩不到 C，挂载照样成功、只计数一条读不出的被抛弃根，只被 C 引用的 14 个槽罩不到
/// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount() {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-torn-abandoned");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    let mut devices = pool.reopen_recorded();
    corrupt_unit_copies(
        &mut devices,
        &third.root.tree_table.locations,
        usize::try_from(singlefs_format::NODE_BYTES).expect("16384"),
    );
    let remounted =
        mount_writable(&parameters(), &mut devices).expect("被抛弃根的树表撕裂不拒绝挂载");
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "C 读不出");
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "只被 C 引用的 14 个槽罩不到（C 读得出时隔离 14，见上一条用例）"
    );
}

/// 被抛弃根 C 那棵账最左那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」，链上校验和逐道重算（panic 面普查 R7）：
/// 挂载不 panic，C 计成一条读不出的被抛弃根，只被 C 引用的槽罩不到。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn an_abandoned_roots_allocation_record_whose_span_runs_past_the_unit_area_is_counted_and_does_not_panic(
) {
    let ThroughTheThirdPublish {
        mut pool, third, ..
    } = build_through_third_publish("step-four-abandoned-span");
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up(&mut pool, &third);
    let mut devices = pool.reopen_recorded();
    let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
    let allocation_tree_pointer = third
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_ALLOCATION)
        .expect("C 的树表里有分配记录树")
        .root;
    let lowest_leaf_of_device_zero = third
        .allocation_record_tree
        .nodes
        .iter()
        .filter_map(|(node, _)| match node {
            AllocationRecordTreeNode::BelowTheRoot(position)
                if position.level == 0 && position.device == DeviceIdentity(0) =>
            {
                Some(*node)
            }
            AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root => None,
        })
        .min()
        .expect("盘 0 上至少一片装着记录的叶");
    let path_pointers: Vec<_> = [
        AllocationRecordTreeNode::Root,
        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
            level: 1,
            device: DeviceIdentity(0),
            index_in_device: 0,
        }),
        lowest_leaf_of_device_zero,
    ]
    .into_iter()
    .map(|node| {
        third
            .allocation_record_tree
            .pointer_of(node)
            .expect("C 那棵树里有这个节点")
    })
    .collect();
    let root_slot_position = root_slot_position_of(third.root.checkpoint_txg.0);
    let mut root_slot = read_root_slot(&mut devices, root_slot_position);
    let mut tree_table_node =
        read_unit_at(&mut devices, third.root.tree_table.locations[0], node_bytes);
    let mut allocation_tree_nodes_from_the_root: Vec<Vec<u8>> = path_pointers
        .iter()
        .map(|pointer| read_unit_at(&mut devices, pointer.locations[0], node_bytes))
        .collect();
    let unit_area_end_slot = UNIT_AREA_START_SLOT + unit_area_slots_of_device(IMAGE_BYTES);
    let what = move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
        &mut root_slot,
        &mut tree_table_node,
        &mut allocation_tree_nodes_from_the_root,
        unit_area_end_slot,
    )
    .expect("C 那棵账最左那片叶装着记录");
    for (pointer, bytes) in path_pointers
        .iter()
        .zip(&allocation_tree_nodes_from_the_root)
    {
        write_unit_to_every_location(&mut devices, pointer.locations, bytes);
    }
    write_unit_to_every_location(
        &mut devices,
        third.root.tree_table.locations,
        &tree_table_node,
    );
    write_root_slot(&mut devices, root_slot_position, &root_slot);
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let abandoned_root_on_disk = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .find(|root| {
        root.instance == third.root.instance && root.checkpoint_txg == third.root.checkpoint_txg
    })
    .expect("C 的根槽自证校验和重算过，仍然读得出");
    let ledger = allocation_records_under_root(&devices, &abandoned_root_on_disk);
    assert!(
        matches!(
            ledger,
            Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽"
            })
        ),
        "C 那棵账该报「不在它所在叶按位置罩的那一段里」，实际交回的是 {ledger:?}（{what}；分配记录树根在槽 {:?}）",
        allocation_tree_pointer.locations.map(|location| location.slot)
    );
    let remounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|error| panic!("被抛弃根那棵账里的坏记录不拒绝挂载：{error:?}（{what}）"));
    assert_eq!(remounted.output.abandoned_roots_unreadable, 1, "{what}");
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "只被 C 引用的 14 个槽罩不到，与树表撕裂那一条同一个数"
    );
}

/// 根槽读不出、它那次发布的记录还在：树 ID 水位按记录新根段带的算（C342（树 ID 水位在根读不出时退回去重发） 的载体，
/// `recovery::highest_tree_identifier_watermark_in_the_ring`）。可写挂载写行那次发布取的就是它。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record(
) {
    let pool = build_pool("step-four-unreadable-file-version-root-watermark");
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let immutable = &system_configuration.immutable;
    let file_version_root_slot = target_for_publish(
        CheckpointTxg(3),
        parameters().geometry.root_ring_slots_per_region,
    );
    let unreadable = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        RootRingSlotTarget {
            named_slots: NamedRootRingSlots::naming(&[file_version_root_slot]),
            region_devices: immutable.region_devices,
            fixed_structure_slot_spacing: immutable.sizes.fixed_structure_slot_spacing,
        },
    );
    let readable = readable_roots(
        &unreadable,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    );
    assert!(
        !readable.is_empty()
            && readable
                .iter()
                .all(|root| root.checkpoint_txg != CheckpointTxg(3)),
        "(1, 3) 那一槽读不出、别的根读得出"
    );
    assert_eq!(
        readable
            .iter()
            .map(|root| root.tree_identifier_watermark)
            .max(),
        Some(11),
        "读得出的根都带 mkfs 种下的 11"
    );
    let records = scan_journal(&unreadable, &system_configuration);
    assert!(
        records
            .values()
            .any(|record| record.new_tree_identifier_watermark == 19),
        "(1, 3) 那次发布的记录还在环里，新根段带 19"
    );
    assert_eq!(
        highest_tree_identifier_watermark_in_the_ring(
            &unreadable,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            &records,
        ),
        Some(19),
        "根读不出、记录还在：水位按记录带的 19 算，不退回读得出的根带的 11"
    );
}

/// C558（回退目标是环里最旧的根时回退那次发布会写坏它） 的注入方式：回退那次发布的根槽写那一次，按 `mode` 失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootSlotWriteFailure {
    /// 一个字节都不写，报块设备错。
    RefusedBeforeWriting,
    /// 只写下前一半（新内容），后一半留旧内容，报块设备错：撕裂的根槽。
    TornAfterTheFirstHalf,
    /// 整槽写成 0，报块设备错。
    ZeroesWritten,
}

/// 一块盘上某一个偏移的第一次写按 `mode` 失败，之后照转；别的写、读、屏障都照转。
struct FirstWriteAtOneOffsetFails<Inner: BlockDevice> {
    inner: Inner,
    failing_offset: DeviceOffsetInBytes,
    mode: RootSlotWriteFailure,
    has_failed: bool,
}

impl<Inner: BlockDevice> BlockDevice for FirstWriteAtOneOffsetFails<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        if offset == self.failing_offset && !self.has_failed {
            self.has_failed = true;
            match self.mode {
                RootSlotWriteFailure::RefusedBeforeWriting => {}
                RootSlotWriteFailure::TornAfterTheFirstHalf => {
                    // 按整个物理块写（录制设备不收不满一块的写）：前一半是新内容，后一半是槽里原有的旧内容。
                    let mut torn = vec![0u8; bytes.len()];
                    self.inner.read_at(offset, &mut torn)?;
                    let half = bytes.len() / 2;
                    torn[..half].copy_from_slice(&bytes[..half]);
                    self.inner.write_at(offset, &torn, WriteDurability::Plain)?;
                }
                RootSlotWriteFailure::ZeroesWritten => {
                    self.inner
                        .write_at(offset, &vec![0u8; bytes.len()], WriteDurability::Plain)?;
                }
            }
            return Err(injected_block_device_error("回退那次发布的根槽写"));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// C558 的池：A（txg 3）、B（txg 4）之后同一个进程里覆盖写到 txg 10，重开取号 2（写行、暖机），再覆盖写到 txg 27：
/// 根环 24 槽里装的是 txg 4–27，最旧的是 B (1, 4)。回退到 B 的那次发布是 txg 28，它的根槽就是 B 住的那一槽（28 与 4 差一圈）。
/// 转过根环之前重开一次，让转环时的分配器是挂载重建的那一份——`build_pool` 的分配器没有根环表，转过根环之后 I-3.1（已分配统计对得上）
/// 红（C551（`build_pool` 的分配器没有根环表））。交回池与 B 的内容。
fn build_a_full_ring_whose_oldest_root_is_the_second_version(tag: &str) -> (BuiltPool, Vec<u8>) {
    let mut pool = build_pool(tag);
    let content_of_the_oldest_ring_root = content_of(1500, 23);
    overwrite_in_process(
        &mut pool,
        &content_of_the_oldest_ring_root,
        InstanceGeneration(1),
    );
    let mut overwrite = 0usize;
    while pool.output.root.checkpoint_txg.0 < 10 {
        overwrite_in_process(
            &mut pool,
            &content_of(1000 + overwrite, 29 + overwrite),
            InstanceGeneration(1),
        );
        overwrite += 1;
    }
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("转环之前重开");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    while pool.output.root.checkpoint_txg.0 < 27 {
        overwrite_in_process(
            &mut pool,
            &content_of(1000 + overwrite, 29 + overwrite),
            InstanceGeneration(2),
        );
        overwrite += 1;
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(27));
    assert_eq!(
        root_slot_position_of(28),
        root_slot_position_of(4),
        "回退那次发布（txg 28）的根槽就是 B 住的那一槽"
    );
    (pool, content_of_the_oldest_ring_root)
}

fn oldest_ring_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(4),
    }
}

/// 回退到 B，回退那次发布的根槽写按 `mode` 失败：交回回退的错。设备换回录制设备装回池里（注入只包这一次）。
fn roll_back_to_the_oldest_ring_root_with_its_root_slot_write_failing(
    pool: &mut BuiltPool,
    mode: RootSlotWriteFailure,
) -> RollbackError {
    let (failing_device, failing_offset) = root_slot_position_of(28);
    let devices = pool.devices.take().expect("镜像还开着");
    let mut wrapped: Vec<(DeviceIdentity, FirstWriteAtOneOffsetFails<Recorded>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FirstWriteAtOneOffsetFails {
                    inner: device,
                    failing_offset: if identity == failing_device {
                        failing_offset
                    } else {
                        DeviceOffsetInBytes(u64::MAX)
                    },
                    mode,
                    has_failed: false,
                },
            )
        })
        .collect();
    let refusal = roll_back_by_a_forward_publish(
        &parameters(),
        &mut wrapped,
        &mut pool.allocator,
        &mut pool.output,
        oldest_ring_root(),
    )
    .expect_err("根槽写注入失败，回退那次发布报错");
    assert!(
        wrapped.iter().any(|(_, device)| device.has_failed),
        "{mode:?}：注入真的打在了那一次根槽写上"
    );
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );
    refusal
}

/// C558 判可达性（主 agent 2026-09-26 立：两块盘、回退目标取环里最旧那一条、在回退那次发布的根槽写上逐点注入失败，看之后同一条回退还能不能再做）：
/// 回退目标 B (1, 4) 是环里最旧的根，回退那次发布 D（txg 28）的根槽就是 B 住的那一槽。根槽写三种失败（一字节不写、写下一半撕裂、写成 0）
/// 各跑两条路：
/// - 同一个进程里：D 冻结在分配器上（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」），再调同一条回退在任何读写之前拒成
///   `PublishFrozenAfterAWriteFailureIsNotResentYet`；原样重发 D 做成，冷启动读回 B 的内容——回退做成了。
/// - 进程在重发之前崩了：D 的记录在根槽写之前已持久（D16（发布语义） 已定项 7 的持久顺序），恢复由记录重建 D 的根（已定项 15），
///   读回 B 的内容；B 的根槽哪怕被撕坏，回退也已经生效，不用再做一次。
///
/// 两条路上回退都没有丢，C558 说的「退不回去」这两条路上走不到。checker 在重发之后与崩溃之后重开时全绿；崩在重发之前、
/// B 的根槽被写坏的两格，恢复之前的镜像上 I-3.1（已分配统计对得上） 红，与普通发布同形（见用例里那一段）。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash(
) {
    failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
        RootSlotWriteFailure::RefusedBeforeWriting,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash(
) {
    failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
        RootSlotWriteFailure::TornAfterTheFirstHalf,
    );
}

#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash(
) {
    failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
        RootSlotWriteFailure::ZeroesWritten,
    );
}

fn failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_is_resent_in_process_or_replayed_after_a_crash(
    mode: RootSlotWriteFailure,
) {
    let (mut pool, content_of_the_oldest_ring_root) =
        build_a_full_ring_whose_oldest_root_is_the_second_version(&format!(
            "step-four-c558-in-process-{mode:?}"
        ));
    let refusal =
        roll_back_to_the_oldest_ring_root_with_its_root_slot_write_failing(&mut pool, mode);
    assert!(
        matches!(&refusal, RollbackError::Publish(failed) if matches!(failed.cause, PublishError::BlockDevice(_))),
        "{mode:?}：{refusal:?}"
    );
    assert!(
        pool.allocator.frozen_publish().is_some(),
        "{mode:?}：D 冻结着"
    );
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(27),
        "{mode:?}：调用方的现行版本还是 txg 27"
    );
    match roll_back(&mut pool, oldest_ring_root()) {
        Err(RollbackError::PublishFrozenAfterAWriteFailureIsNotResentYet {
            checkpoint_txg,
            ..
        }) => assert_eq!(checkpoint_txg, CheckpointTxg(28), "{mode:?}"),
        other => panic!("{mode:?}：冻结着时再回退要先拒：{other:?}"),
    }
    let resent = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        resend_the_frozen_publish(&mut writer, &mut pool.allocator)
            .expect("原样重发 D")
            .expect("冻结着一次")
    };
    pool.output = resent.into_file_version().expect("D 带文件");
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(28));
    verdicts_without_any_violation(&pool.memory_pool(), &format!("{mode:?}：重发之后"));
    let cold = pool.reopen_cold();
    let report = recover(&cold, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(28)))
    );
    assert!(
        matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == content_of_the_oldest_ring_root),
        "{mode:?}：重发之后读回 B 的内容：{:?}",
        report.outcome
    );

    let (mut crashed, content_of_the_oldest_ring_root_in_the_crashed_pool) =
        build_a_full_ring_whose_oldest_root_is_the_second_version(&format!(
            "step-four-c558-crash-{mode:?}"
        ));
    roll_back_to_the_oldest_ring_root_with_its_root_slot_write_failing(&mut crashed, mode);
    let violated_before_recovery: Vec<&str> = check_pool_image(&crashed.memory_pool())
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect();
    let expected_violated_before_recovery: &[&str] = match mode {
        RootSlotWriteFailure::RefusedBeforeWriting => &[],
        // B 的根槽被写坏、D 的记录还没被哪次恢复施加：最新根 txg 27 的账还算着只被 B 引用、释放代 5 的单元，遍历里少了 B——
        // I-3.1（已分配统计对得上） 记账多于遍历。普通覆盖写在同一格上把根槽写坏，checker 判出的是逐字相同的一条（实三草稿里量过），
        // 与回退无关；恢复重开之后全绿（下面）。这一格 checker 该不该判红交主 agent（实三报告「交主 agent 的设计问题」）。
        RootSlotWriteFailure::TornAfterTheFirstHalf | RootSlotWriteFailure::ZeroesWritten => {
            &["I-3.1"]
        }
    };
    assert_eq!(
        violated_before_recovery, expected_violated_before_recovery,
        "{mode:?}：崩在重发之前、恢复之前的镜像上判红的不变量"
    );
    let cold_after_the_crash = crashed.reopen_cold();
    let report_after_the_crash = recover(&cold_after_the_crash, JournalPolicy::Consult);
    assert_eq!(
        report_after_the_crash.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(28))),
        "{mode:?}：D 的记录已持久，恢复由记录重建 D 的根"
    );
    assert!(
        matches!(&report_after_the_crash.outcome, RecoveryOutcome::FileRead { content, .. } if *content == content_of_the_oldest_ring_root_in_the_crashed_pool),
        "{mode:?}：崩在重发之前，读回 B 的内容：{:?}",
        report_after_the_crash.outcome
    );
    drop(cold_after_the_crash);
    let mut devices = crashed.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("崩溃之后可写挂载");
    assert_eq!(
        (
            remounted.output.effective_root.instance,
            remounted.output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(28)),
        "{mode:?}：新实例接在 D 后面"
    );
    crashed.devices = Some(devices);
    verdicts_without_any_violation(&crashed.memory_pool(), &format!("{mode:?}：崩溃之后重开"));
}

/// C542 剩下的普通可写挂载那一支（`checks-owed.md` C542 行「所选根是最新根时，这次挂载的写行与暖机能不能把环里比它旧的有效根全盖掉」，
/// 调度记录实现批次排法实四那一项「C542 普通挂载那一支用例判可达」）：**盖不掉，走不到**。取号之前的预演不做释放前读盘核
/// （`mount.rs` 的 `dry_run_of_the_publishes_after_acquisition`），它与真发只在「这一串换下、核出对不上被隔离的那一份在这一串里被回收」时分叉；
/// 这一串换下的单元释放代 ≥ 这一串第一次发布的 txg（28），回收要环里最旧的有效根 ≥ 28，而所选的最新根 txg 27 在这一串之后还在环里
/// （这一串写 2–3 条根，只盖环里最旧的那几槽，根环 24 槽）⇒ 这一串自己换下的一个都回收不了，预演与真发取的落点逐项相同，谈不上取号之后被落点拒。
///
/// 搭法：满环（txg 4–27，最旧 B (1, 4)，最新 (2, 27)）之后进程退出，盘 1 上最新那一版记账树根那一份改坏一字节（写行那次要换下它）；
/// 普通可写挂载：所选根是最新根 (2, 27)，写行 txg 28 读盘核出盘 1 那一份对不上、两份的记录都留在已分配（D19 已定项 5）；
/// 挂载做成（取号 3），环里 txg 27 那条根还在；这一串换下的（释放代 ≥ 28）在挂载交回的账里全是已释放、还在 defer 里，一个都没被这一串回收；
/// 这一串取的落点没有一个是被隔离的那一槽。池级 checker：坏的那一份仍被环里的 txg 27 引用，I-2.1 / I-4.8 / I-7.4 照判红
/// （改坏盘本身的投影），I-3.1 / I-3.11 成立（隔离的那一份核出坏的，按单元豁免）。
///
/// 所选根不是最新根的那一形（崩溃恢复落到环里最旧的根、更新的根槽都读不出、它们的记录验不过）这一串会盖掉所选根、回收这一串自己换下的，
/// `mount.rs` 那段注释说的分叉在那一形上走得到；取号之后被落点拒要池子紧到只剩那一槽合政策，实四乙在空间准入照判的小盘上扫过没扫出来（报告里写范围）。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge(
) {
    let (mut pool, _) =
        build_a_full_ring_whose_oldest_root_is_the_second_version("step-four-c542-newest-root");
    let newest = pool.output.clone();
    assert_eq!(newest.root.checkpoint_txg, CheckpointTxg(27));
    // 写行那次要换下、释放之前读盘核的一个单元：最新那一版的记账树根（经映射寻址，释放前核它每一份；树表与映射树自举豁免，不核）。
    let accounting_root_slot = newest.unit(TransactionUnit::AccountingTree).slot;
    let mut devices = pool.reopen_recorded();
    {
        let (_, device_one) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == DeviceIdentity(1))
            .expect("盘 1");
        let offset = accounting_root_slot.to_device_offset();
        let mut bytes = vec![0u8; usize::try_from(singlefs_format::NODE_BYTES).expect("16384")];
        device_one
            .read_at(offset, &mut bytes)
            .expect("读盘 1 那一份记账树根");
        bytes[100] ^= 0xff;
        device_one
            .write_at(offset, &bytes, WriteDurability::Plain)
            .expect("改坏盘 1 那一份记账树根");
    }
    let mounted = mount_writable(&parameters(), &mut devices)
        .expect("所选根是最新根：写行那次核出一份坏的、隔离它，挂载照常做成");
    pool.devices = Some(devices);
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.row_publish.root().checkpoint_txg
        ),
        (
            InstanceGeneration(3),
            InstanceGeneration(2),
            CheckpointTxg(27),
            CheckpointTxg(28)
        ),
        "所选根是环里最新的 (2, 27)；写行 txg 28"
    );
    let chain: Vec<&TransactionOutput> = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|publish| {
            publish
                .file_version()
                .expect("所选那一版带文件，这一串每一次都带文件")
        })
        .collect();
    let quarantined: Vec<(DeviceIdentity, u64)> = chain
        .iter()
        .flat_map(|version| {
            version
                .quarantined_after_release_checksum_mismatch
                .iter()
                .map(|copy| (copy.device, copy.placement.slot.0))
        })
        .collect();
    assert_eq!(
        quarantined,
        vec![
            (DeviceIdentity(0), accounting_root_slot.0),
            (DeviceIdentity(1), accounting_root_slot.0)
        ],
        "写行那次读盘核出盘 1 那一份记账树根对不上：两块盘上的记录一起留在已分配"
    );
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let ring_txgs: BTreeSet<u64> = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| root.checkpoint_txg.0)
    .collect();
    assert!(
        ring_txgs.contains(&27),
        "这一串写完，所选的最新根 txg 27 还在环里（这一串只盖最旧的几槽）：{ring_txgs:?}"
    );
    // 写行那次换下的：最新那一版里、写行那次重写了的角色原来住的槽（释放代 28，这一串自己换下的）。
    let row_publish = chain[0];
    let replaced_by_the_row_publish: Vec<SlotNumber> = row_publish
        .rewritten
        .iter()
        .filter_map(|role| newest.units.iter().find(|unit| unit.identity == *role))
        .map(|unit| unit.slot)
        .collect();
    assert!(
        !replaced_by_the_row_publish.is_empty(),
        "写行那次重写了最新那一版的固定点"
    );
    let taken_by_the_chain: BTreeSet<SlotNumber> = chain
        .iter()
        .flat_map(|version| {
            version
                .rewritten
                .iter()
                .map(|role| version.unit(*role).slot)
        })
        .collect();
    for slot in &replaced_by_the_row_publish {
        assert!(
            !taken_by_the_chain.contains(slot),
            "写行那次换下的槽 {slot:?} 没被这一串再发出去"
        );
        for device_map in &mounted.allocator.devices {
            assert!(
                !device_map.is_free(*slot),
                "盘 {}：写行那次换下的槽 {slot:?} 挂载之后还不是空闲槽（在 defer 里或隔离着）：环里最旧的有效根没越过 txg 28，这一串回收不了它",
                device_map.device.0
            );
        }
    }
    for version in &chain {
        assert!(
            version
                .rewritten
                .iter()
                .all(|role| version.unit(*role).slot != accounting_root_slot),
            "这一串取的落点没有一个是被隔离的那一槽"
        );
    }
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated,
        vec!["I-2.1", "I-4.8", "I-7.4"],
        "盘 1 那一份记账树根坏在盘上、仍被环里的 txg 27 引用：只红这一份坏单元的投影；{verdicts:?}"
    );
    for invariant in ["I-3.1", "I-3.11"] {
        assert!(
            verdicts
                .iter()
                .any(|(name, verdict)| *name == invariant && *verdict == InvariantVerdict::Holds),
            "{invariant} 成立：隔离的那一份核出坏的，按单元豁免；{verdicts:?}"
        );
    }
}

/// 实三交回 Q2 立的新欠账（满环时根槽写坏的恢复前镜像，checker 判 I-3.1 红）查原因：与回退无关，普通覆盖写同一格逐字相同；
/// 红的量正好是被盖掉的那条根独占的那几个单元。
///
/// 满环（txg 4–27，最旧 B (1, 4)）之后普通覆盖写 D（txg 28），D 的根槽就是 B 住的那一槽；那一次根槽写把整块写成 0、报错，进程在重发之前崩了。
/// 恢复之前的镜像：B 的根槽没了，D 的记录已持久（根槽写之前落盘）但还没被哪次恢复施加，环里最新的是 C（txg 27）。checker 只判 I-3.1 红，
/// 每块盘「记账的已分配 − 遍历全部有效根」= 只被 B 引用的那几个单元的字节数（229 376 = 14 槽）；这几个单元在 C 的账里是已释放、释放代 5
/// （txg 5 那次换下的），回收要环里最旧的有效根 ≥ 5——B 还在环里时回收不了，C 的账照算它们；B 的根槽被 D 那一次写坏之后遍历里没有 B，
/// 而 checker 只把「已被某次恢复施加」的记录并进遍历（`walk.rs` 的 `versions_applied_only_by_records`），D 不在其内，用的仍是 C 的账。
/// D 那一版的账（D 盖掉 B 的槽之后环里最旧的有效根是 5，释放代 5 的已回收）与这个环对得上。恢复（由 D 的记录重建 D）之后重开，全绿。
/// checker 在这一格上该不该判红、要不要把「根槽写坏、记录已持久没施加」那一版并进来，是 checker 判法的事，交主 agent。
#[test]
#[ignore = "harness 重档：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced(
) {
    let (mut pool, _) = build_a_full_ring_whose_oldest_root_is_the_second_version(
        "step-four-pre-recovery-allocated-statistic",
    );
    let image_before_the_overwrite = pool.memory_pool();
    let system_configuration =
        choose_system_configuration(&image_before_the_overwrite).expect("系统配置");
    let mut ring = readable_roots(
        &image_before_the_overwrite,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    ring.sort_by_key(|root| root.checkpoint_txg);
    assert_eq!(
        ring.iter()
            .map(|root| root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        (4..=27).collect::<Vec<u64>>(),
        "满环：txg 4–27"
    );
    let referenced_by =
        |root: &singlefs_core::root_record::RootRecord| -> BTreeSet<(DeviceIdentity, u64, u16)> {
            allocation_records_under_root(&image_before_the_overwrite, root)
                .expect("环里每条根的账读得出")
                .into_iter()
                .filter(|record| !record.is_released)
                .map(|record| (record.device, record.slot.0, record.span_slots))
                .collect()
        };
    let referenced_by_the_newer_roots: BTreeSet<(DeviceIdentity, u64)> = ring[1..]
        .iter()
        .flat_map(|root| {
            referenced_by(root)
                .into_iter()
                .map(|(device, slot, _)| (device, slot))
        })
        .collect();
    let only_the_oldest_references: Vec<(DeviceIdentity, u64, u16)> = referenced_by(&ring[0])
        .into_iter()
        .filter(|(device, slot, _)| !referenced_by_the_newer_roots.contains(&(*device, *slot)))
        .collect();
    let newest_account =
        allocation_records_under_root(&image_before_the_overwrite, &ring[23]).expect("C 的账");
    for (device, slot, _) in &only_the_oldest_references {
        assert!(
            newest_account.iter().any(|record| record.device == *device
                && record.slot.0 == *slot
                && record.is_released
                && record.generation == CheckpointTxg(5)),
            "只被 B 引用的盘 {} 槽 {slot} 在 C 的账里是已释放、释放代 5",
            device.0
        );
    }

    let (failing_device, failing_offset) = root_slot_position_of(28);
    let devices = pool.devices.take().expect("镜像还开着");
    let mut wrapped: Vec<(DeviceIdentity, FirstWriteAtOneOffsetFails<Recorded>)> = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FirstWriteAtOneOffsetFails {
                    inner: device,
                    failing_offset: if identity == failing_device {
                        failing_offset
                    } else {
                        DeviceOffsetInBytes(u64::MAX)
                    },
                    mode: RootSlotWriteFailure::ZeroesWritten,
                    has_failed: false,
                },
            )
        })
        .collect();
    let previous = pool.output.clone();
    let failed = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, wrapped.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile {
                content: &content_of(1700, 41),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(2),
        )
    };
    assert!(
        matches!(failed, Err(PublishError::BlockDevice(_))),
        "D 的根槽写报错：{:?}",
        failed.as_ref().err()
    );
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.inner))
            .collect(),
    );

    let verdicts = check_pool_image(&pool.memory_pool());
    let violations: Vec<(&str, &String)> = verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect();
    assert_eq!(
        violations
            .iter()
            .map(|(invariant, _)| *invariant)
            .collect::<Vec<_>>(),
        vec!["I-3.1"],
        "恢复之前的镜像：只有 I-3.1 红"
    );
    let (allocated, walked) =
        singlefs_harness::history::allocated_and_walked_bytes(violations[0].1)
            .expect("I-3.1 的说明里带着已分配与遍历两个数");
    let bytes_only_the_oldest_references_on_the_first_red_device: u64 = only_the_oldest_references
        .iter()
        .filter(|(device, _, _)| *device == DeviceIdentity(0))
        .map(|(_, _, span)| u64::from(*span) * singlefs_format::SLOT_BYTES)
        .sum();
    assert_eq!(
        (
            allocated - walked,
            bytes_only_the_oldest_references_on_the_first_red_device
        ),
        (229_376, 229_376),
        "盘 0：记账多于遍历的正好是只被 B 引用的那几个单元（14 槽）：{}",
        violations[0].1
    );

    let cold = pool.reopen_cold();
    assert_eq!(
        recover(&cold, JournalPolicy::Consult).effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(28))),
        "D 的记录已持久：恢复由记录重建 D"
    );
    drop(cold);
    let mut reopened_devices = pool.reopen_recorded();
    mount_writable(&parameters(), &mut reopened_devices).expect("恢复之后可写挂载");
    pool.devices = Some(reopened_devices);
    verdicts_without_any_violation(&pool.memory_pool(), "恢复之后重开");
}
