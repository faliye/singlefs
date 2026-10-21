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
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs"]
mod scenarios_of_rollback_by_a_forward_publish;

use common::{build_pool, disk_snapshot, geometry, parameters, BuiltPool, IMAGE_BYTES};
use common_corrupted_allocation_record::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain;
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_rollback_by_a_forward_publish::{
    abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up,
    build_through_third_publish, content_of, first_root, overwrite_in_process, read_root_slot,
    roll_back, root_slot_position_of, third_content, write_root_slot, ThroughTheThirdPublish,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, TreeIdentifier};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocation_record_tree::{
    AllocationRecordTreeNode, AllocationRecordTreeNodePosition,
};
use singlefs_core::allocator::{
    unit_area_slots_of_device, AllocationRecord, DeviceFreeMap, PoolAllocator,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_writable, mount_writable_with_test_only_switches, Mounted, RollbackError, ShadowLedger,
};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::TREE_KIND_ALLOCATION;
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration,
    highest_tree_identifier_watermark_in_the_ring, readable_roots, scan_journal, walk_to_file,
    RecoveryFailure,
};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::TransactionUnit;
use singlefs_format::UNIT_AREA_START_SLOT;
use singlefs_harness::fault_injection::{
    NamedRootRingSlots, PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget,
};
use singlefs_harness::memory_pool::writes_and_segments;
use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments};
use singlefs_harness::{RecordedOperation, RecordedOperationKind};
use std::collections::BTreeSet;

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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::rolling_back_to_the_first_version_while_mounted_publishes_one_root_carrying_its_state_and_cold_start_reads_the_first_content,
    );
}

/// 回退那次发布 D「每段写几个」（`layout/01-first-txn.md` 八节「实例切换 / 管理员回退」那一行，原来没有用例钉）：
/// 从录制流上截回退那次发布（回退调用之前流里有几步，到调用返回），按崩溃点重放同一条切段规则切（`split_into_segments`，屏障按设备记）。
/// D 孤立看是 [24 个单元写] 屏障 [两份记录] 屏障 [根 FUA] [两份系统配置槽] 屏障：重写 12 个单元、两块盘各一份，共 29 次写。
/// 整条流（mkfs 之后）到 D 为止 38 段（取号写两轮之后，mkfs 那个进程里与之后那次可写挂载的取号各多一段两份系统配置槽写；
/// checker 档固定脚本那条流 `RollbackToFirstVersion` 那一臂钉的段数跟着要改，那边不在这一批里）：
/// C 的两份系统配置槽写过了发布末尾那道屏障、自成一段，D 的单元写不与它并段。
/// 不跑 checker 与冷启动，debug 下单跑几秒，是快档用例。
#[test]
fn rolling_back_to_the_first_version_writes_its_units_in_one_segment_before_the_records_the_root_and_the_system_configuration(
) {
    let ThroughTheThirdPublish { mut pool, .. } =
        build_through_third_publish("rollback-publish-segments");
    let operations_before_the_rollback = pool.stream.operation_count();
    roll_back(&mut pool, first_root()).expect("回退到 A");
    let rollback_version = pool.output.clone();
    let operations_of_the_rollback: Vec<RecordedOperation> =
        pool.stream.operations()[operations_before_the_rollback..].to_vec();
    let segments_of_the_rollback = split_into_segments(&operations_of_the_rollback, &geometry());
    let writes_of_the_rollback = operations_of_the_rollback
        .iter()
        .filter(|operation| match operation.kind {
            // 切段时 discard 算一次写（`segments` 文件头），这里与段里的写数同一个口径。
            RecordedOperationKind::Write
            | RecordedOperationKind::WriteForceUnitAccess
            | RecordedOperationKind::WriteZeroes
            | RecordedOperationKind::Discard => true,
            RecordedOperationKind::Barrier => false,
        })
        .count();
    let retained = pool.retained_operations();
    let (writes_after_mkfs, segments_after_mkfs) =
        writes_and_segments(&retained[pool.mkfs_operation_count..], &geometry());
    let segment_lengths_after_mkfs: Vec<usize> = segments_after_mkfs.iter().map(Vec::len).collect();
    assert_eq!(
        segment_sizes_text(&segments_of_the_rollback),
        "24+2+1+2",
        "D 孤立看的段序列：单元写一段、记录一段、根一段、系统配置一段"
    );
    assert_eq!(
        segment_kinds_text(&segments_of_the_rollback),
        "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        "D 的单元写只有单元写一种，不取号、不写系统配置之外的原地写"
    );
    assert_eq!(writes_of_the_rollback, 29, "D 共 29 次写：24 + 2 + 1 + 2");
    assert_eq!(
        rollback_version.rewritten.len(),
        12,
        "D 重写 12 个单元，每块盘各一份：{:?}",
        rollback_version.rewritten
    );
    assert_eq!(
        segment_lengths_after_mkfs,
        vec![
            2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16,
            2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2
        ],
        "整条流到 D 为止的段序列：C 的系统配置槽写自成一段、D 的单元写自成一段"
    );
    assert_eq!(
        writes_after_mkfs.len(),
        170 + 29,
        "到 C 170 次写（两次取号各两轮、每轮两写）、D 29 次写"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_next_overwrite_releases_the_resurrected_units_and_every_invariant_holds,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::after_rolling_back_to_the_first_version_the_third_versions_root_is_still_a_candidate_and_rolling_back_to_it_reads_the_third_content,
    );
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write,
    );
}

/// 验收「拒」：在任何写之前拒的几种，各一条，拒之后盘上逐字节不变、分配器与现行版本不动——
/// A 引用的单元在 C 的账里没有记录；仍分配而分配代不同；A 那一版的账有一个节点读不出；C 那一版的账有一个节点读不出；
/// 复活集里一个单元在一块盘上的那一份校验和对不上；两版的树号不同（条款没写，第一版不支持）。
/// 前两格的账按内存里那一份判（判定与之后的复活、释放读同一份），用例把分配器换成少一条、改了分配代的一份来造；
/// 后几格改盘上的字节。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_current_account_without_the_data_unit_of_the_first_version_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::CurrentAccountWithoutTheDataUnitOfTheFirstVersion,
    );
}

#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_current_account_allocating_the_data_slot_of_the_first_version_under_another_generation_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::CurrentAccountAllocatingTheDataSlotOfTheFirstVersionUnderAnotherGeneration,
    );
}

#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_allocation_record_tree_root_of_the_first_version_unreadable_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::AllocationRecordTreeRootOfTheFirstVersionUnreadable,
    );
}

#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_allocation_record_tree_root_of_the_third_version_unreadable_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::AllocationRecordTreeRootOfTheThirdVersionUnreadable,
    );
}

#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn refusal_before_any_write_when_data_unit_of_the_first_version_corrupt_on_device_one_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
) {
    refusal_before_any_write_leaves_the_disk_the_allocator_and_the_current_version_unchanged(
        RefusalBreakage::DataUnitOfTheFirstVersionCorruptOnDeviceOne,
    );
}

#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write,
    );
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

/// C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入；被抛弃的根由崩溃恢复造出
/// （`abandon_the_third_version_by_a_recovery_that_lands_on_the_second_warm_up`，第一轮判决 H6 那一形）：抛弃 C 的那次挂载照 C694 的
/// `C_NEWER_ROOTS` 把 C 读得出的账里的 14 个槽在那一次挂载里隔离，这份隔离只住那一次挂载的内存——影子账关着，下一次重开不再隔离，
/// 之后下一版的数据单元落回 C 的数据槽，沿被抛弃的根 C 走到文件读不回第三次的内容；
/// 开着，只被 C 引用的 14 个槽隔离，第三次的内容原样读回。两臂报的分支名不同。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn without_the_shadow_ledger_a_publish_after_a_recovery_reuses_the_abandoned_data_slot_and_the_abandoned_root_reads_a_torn_unit(
) {
    publish_after_a_recovery_that_abandoned_the_third_version(ShadowLedger::Off);
}

/// 同一条脚本、影子账开着：只被 C 引用的 14 个槽隔离，下一版的数据单元不落回 C 的数据槽，沿 C 读回第三次的内容。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "抛弃 C 的那次挂载照 C_NEWER_ROOTS 隔离 C 那一次发布写的 14 个槽"
    );
    let remounted = remount_with_the_shadow_ledger(&mut pool, shadow_ledger);
    assert_eq!(
        remounted.output.shadow_ledger_branch,
        shadow_ledger.branch_name(),
        "挂载报出的分支名要与传进去的那一臂相符"
    );
    // C482（只供测试的开关有三个走了哪一条看不出来）：上一条两边都取同一臂的名字，两臂合成一个名字时照样绿；这里比另一臂，
    // 运行时才分得出走的是哪一臂（`.claude/rules/fs-design.md` 五条硬要求第 4 条「分支必须可观测」）。
    let the_other_arm = match shadow_ledger {
        ShadowLedger::On => ShadowLedger::Off,
        ShadowLedger::Off => ShadowLedger::On,
    };
    assert_ne!(
        remounted.output.shadow_ledger_branch,
        the_other_arm.branch_name(),
        "{shadow_ledger:?}：挂载报出的分支名与另一臂 {the_other_arm:?} 的不同"
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
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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

/// 被抛弃根 C 的树表单元在两块盘上都改坏：挂载照样成功（步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败），
/// 计数一条读不出的被抛弃根往上报，C 引用的落点从中央映射现算，只被 C 引用的 14 个槽照样隔离（与 C 读得出时同，见上一条用例；
/// C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "从中央映射现算，只被 C 引用的 14 个槽照样隔离（与 C 读得出时同，见上一条用例）"
    );
}

/// 被抛弃根 C 那棵账最左那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」，链上校验和逐道重算（panic 面普查 R7）：
/// 挂载不 panic，C 计成一条读不出的被抛弃根，引用的落点从中央映射现算，只被 C 引用的槽照样隔离。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "账解不开也从中央映射现算，只被 C 引用的 14 个槽照样隔离，与树表撕裂那一条同一个数（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定）"
    );
}

/// 根槽读不出、它那次发布的记录还在：树 ID 水位按记录新根段带的算（C342（树 ID 水位在根读不出时退回去重发） 的载体，
/// `recovery::highest_tree_identifier_watermark_in_the_ring`）。可写挂载写行那次发布取的就是它。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
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
        Some(65536),
        "读得出的根都带 mkfs 种下的 65536"
    );
    let records = scan_journal(&unreadable, &system_configuration);
    assert!(
        records
            .values()
            .any(|record| record.new_tree_identifier_watermark == 65549),
        "(1, 3) 那次发布的记录还在环里，新根段带 65549（十三棵树 65536..65548 之后）"
    );
    assert_eq!(
        highest_tree_identifier_watermark_in_the_ring(
            &unreadable,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            &records,
        ),
        Some(65549),
        "根读不出、记录还在：水位按记录带的 65549 算，不退回读得出的根带的 65536"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_refused_before_writing_is_resent_in_process_or_replayed_after_a_crash,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::writable_mount_on_the_newest_root_cannot_reclaim_what_its_own_chain_released_so_the_rehearsal_without_the_release_read_cannot_diverge,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
fn plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_rollback_by_a_forward_publish::inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_rollback_by_a_forward_publish::inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback,
    );
}
