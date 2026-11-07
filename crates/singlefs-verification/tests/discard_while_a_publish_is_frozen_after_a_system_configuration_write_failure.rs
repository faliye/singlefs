//! 冻结着一次发布时整理入口发的 discard（C928（冻结着时整理照发 discard））：
//! 一次用户改动那次发布的单元、记录、根槽都落了，系统配置槽轮换那一写报块设备错，发布冻结（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」），
//! 分配器整份退回发布之前；退回的那一份里，抬 F 刚回收空、还没发过 discard 的段仍是整段可再分配。冻结之后调整理入口，discard 不许发到装着冻结那次
//! 落点的段上：冻结那次的根已经 FUA 落盘，重发之前断电，恢复择到它，它引用的单元所在的段收过 discard 就读不回它的单元。
//!
//! 历史在 `common::build_pool` 起的池上（默认几何，单元区很宽、准入不卡），重开可写挂载、开会话：
//! 1. 顺序写第一个文件 64 个数据单元（128 槽）：至少一整段 64 槽全是这一版的数据单元，取其中最低的一段当被判的段；
//! 2. 顺序写第一个文件回到一个数据单元：那 64 个数据单元整份换下、进 defer（释放代是这一次的 txg）；
//! 3. 建四次 inode（四个非空的状态）：准入抬 F 的上限越过第 2 步的 txg；
//! 4. 直接调公开的准入抬 F（会话里被空间拒时 `publish_user_change` 推的就是这一串）：回收第 2 步换下的单元，被判的段整段可再分配、记进待 discard，
//!    这一串之后没有会话的 discard；
//! 5. 被判的段之下每块盘还空着的槽都隔离（只动分配器的隔离位，不写盘）：池子只剩回收出来的那几段可放用户数据（攻方那一形里池子接近满），
//!    下一次发布的数据单元落进被判的段；
//! 6. 盘 0 的下一次系统配置槽写报块设备错，顺序写第一个文件一个数据单元：单元、记录、根槽都落，轮换那一写报错，发布冻结；
//! 7. 调整理的显式入口 `MountedSession::run_compaction`。
mod common;

use common::{build_pool, geometry, parameters, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_verification::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use std::sync::Arc;

type FaultInjectingDevice = FaultInjectingBlockDevice<Recorded>;

/// 第 1 步顺序写的数据单元数：两槽一个，256 槽。落在用户数据最低的空槽对上，先填挂载之后单元区低处零散的空槽对，余下的连续往上，
/// 至少盖满一整段 64 槽。
const DATA_UNITS_OF_THE_LARGE_VERSION: usize = 128;
/// 一段的槽数（D3（空间分配） 已定项 10 的 64 槽对齐段）。
const SLOTS_PER_SEGMENT: u64 = 64;
/// 第 3 步建 inode 的次数：准入抬 F 的上限取第 4 新的不同状态，四次之后它越过第 2 步。
const INODE_CREATIONS_BEFORE_THE_FLOOR_RAISE: u64 = 4;
/// 第 2 步与第 6 步写的内容：一个数据单元以内。
const SMALL_CONTENT_BYTES: usize = 3000;

fn content_of(length: usize, seed: u8) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from(index % 241).expect("小于 256") ^ seed)
        .collect()
}

fn file_version_of(session: &MountedSession) -> &TransactionOutput {
    session
        .current
        .file_version()
        .expect("这段历史里现行那一版都带文件")
}

fn publish_sequential_write_of_the_first_file(
    session: &mut MountedSession,
    devices: &mut Vec<(DeviceIdentity, FaultInjectingDevice)>,
    content: &[u8],
    write_time_seconds: u64,
) -> Result<(), UserChangeRefused> {
    session
        .publish_user_change(
            devices,
            UserChange::SequentialWrite(FirstFile {
                content,
                write_time_seconds,
            }),
        )
        .map(|_published| ())
}

/// 槽 `slot` 所在那一段的起点（单元区起点起 64 槽对齐）。
fn segment_start_of(unit_area_start: u64, slot: SlotNumber) -> SlotNumber {
    SlotNumber(unit_area_start + (slot.0 - unit_area_start) / SLOTS_PER_SEGMENT * SLOTS_PER_SEGMENT)
}

/// 这一版第一个文件的数据单元整段盖满的段里最低的那一段：段里 32 个槽对都是这一版的数据单元。
fn lowest_segment_holding_only_data_units_of(
    version: &TransactionOutput,
    unit_area_start: u64,
) -> SlotNumber {
    let data_unit_slots: std::collections::BTreeSet<u64> = version
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations[0].slot.0)
        .collect();
    let segment_starts: std::collections::BTreeSet<SlotNumber> = data_unit_slots
        .iter()
        .map(|slot| segment_start_of(unit_area_start, SlotNumber(*slot)))
        .collect();
    segment_starts
        .iter()
        .copied()
        .find(|segment_start| {
            (segment_start.0..segment_start.0 + SLOTS_PER_SEGMENT)
                .step_by(2)
                .all(|slot| data_unit_slots.contains(&slot))
        })
        .unwrap_or_else(|| {
            let data_units_by_segment: Vec<(u64, usize)> = segment_starts
                .iter()
                .map(|segment_start| {
                    (
                        segment_start.0,
                        data_unit_slots
                            .range(segment_start.0..segment_start.0 + SLOTS_PER_SEGMENT)
                            .count(),
                    )
                })
                .collect();
            panic!(
                "这一版的数据单元连续落在用户数据最低的空槽对上，该有一整段全是它们：各段（起点, 数据单元数）{data_units_by_segment:?}"
            )
        })
}

/// 盘 0 从上膛起数第一次系统配置槽写（两槽住在偏移 0 起的两个槽距之内）报块设备错，只这一次。
fn the_next_system_configuration_write_on_device_zero_fails() -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::WriteFails,
        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(0)),
        placement: FaultPlacement::OffsetBelow(
            SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                * u64::from(parameters().geometry.fixed_structure_slot_spacing),
        ),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(1),
    }
}

/// 冻结着的时候调整理入口：discard 一段都不发到装着冻结那次发布落点的段上（被判的段就是其中之一）；不发的那一支看得出（交回的 discard 账点名冻结那次的 txg）。
/// 判别力：discard 开头不判冻结（变异「C928 冻结着时整理照发 discard」）时，退回发布之前的分配器里被判的段整段可再分配、在待 discard 表里，
/// 整理入口照发，这一条红在「discard 发到了装着冻结那次落点的段上」。
#[test]
fn compaction_run_while_a_publish_is_frozen_discards_no_segment_holding_the_units_of_the_frozen_publish(
) {
    let parameters = parameters();
    let mut pool = build_pool("compaction-while-a-publish-is-frozen");
    let plan = SharedFaultPlan::unarmed(geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectingDevice)> = pool
        .reopen_recorded()
        .into_iter()
        .map(|(identity, recorded)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, recorded, plan.clone()),
            )
        })
        .collect();
    let mounted = mount_writable(&parameters, &mut devices).expect("重开可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let unit_area_start = session.allocator.devices[0].unit_area_start().slot().0;
    let mut write_time_seconds = FIXED_WRITE_TIME_SECONDS;

    write_time_seconds += 1;
    publish_sequential_write_of_the_first_file(
        &mut session,
        &mut devices,
        &content_of(
            DATA_UNITS_OF_THE_LARGE_VERSION * data_unit_payload_capacity(),
            0x11,
        ),
        write_time_seconds,
    )
    .expect("顺序写 64 个数据单元");
    let judged_segment =
        lowest_segment_holding_only_data_units_of(file_version_of(&session), unit_area_start);

    write_time_seconds += 1;
    publish_sequential_write_of_the_first_file(
        &mut session,
        &mut devices,
        &content_of(SMALL_CONTENT_BYTES, 0x22),
        write_time_seconds,
    )
    .expect("顺序写回一个数据单元：64 个数据单元整份换下");
    let txg_releasing_the_large_version = file_version_of(&session).root.checkpoint_txg;

    for _creation in 0..INODE_CREATIONS_BEFORE_THE_FLOOR_RAISE {
        write_time_seconds += 1;
        session
            .publish_user_change(
                &mut devices,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds,
                },
            )
            .expect("建一个 inode");
    }

    let shadow_ledger = session.shadow_ledger;
    let PoolVersion::WithFile(current) = &mut session.current else {
        panic!("这段历史里现行那一版带文件")
    };
    let current = Arc::make_mut(current);
    let raised = raise_rollback_floor_to_the_admission_ceiling(
        &parameters,
        &mut devices,
        &mut session.allocator,
        current,
        shadow_ledger,
    )
    .expect("准入抬 F 到上限");
    let RaiseToTheAdmissionCeiling::Raised(raised) = raised else {
        panic!("四个非空的状态之后上限高于 F：推了抬 F，实际 {raised:?}")
    };
    assert!(
        raised.ceiling >= txg_releasing_the_large_version,
        "上限 {:?} 不低于换下 64 个数据单元的那一次 {txg_releasing_the_large_version:?}：它们这一串里回收",
        raised.ceiling
    );
    let judged_segment_index =
        usize::try_from((judged_segment.0 - unit_area_start) / SLOTS_PER_SEGMENT)
            .expect("段号装得进 usize");
    for device_map in &session.allocator.devices {
        assert!(
            device_map.segment_is_reusable_as_a_whole(judged_segment_index),
            "{:?}：抬 F 回收之后被判的段 {} 整段可再分配",
            device_map.device,
            judged_segment.0
        );
    }

    let device_identities: Vec<DeviceIdentity> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    for device in device_identities {
        session
            .allocator
            .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
                device,
                SlotNumber(unit_area_start),
                judged_segment.0 - unit_area_start,
            );
    }

    plan.arm(the_next_system_configuration_write_on_device_zero_fails());
    write_time_seconds += 1;
    let frozen_write = publish_sequential_write_of_the_first_file(
        &mut session,
        &mut devices,
        &content_of(SMALL_CONTENT_BYTES, 0x33),
        write_time_seconds,
    );
    plan.disarm();
    assert!(
        matches!(frozen_write, Err(UserChangeRefused::Publish { .. })),
        "系统配置槽轮换那一写报错：这次改动被拒、发布冻结，实际 {frozen_write:?}"
    );
    let frozen = session
        .allocator
        .frozen_publish()
        .expect("单元、记录、根槽都落了，轮换报错：发布冻结在分配器上");
    let segments_of_the_frozen_units: std::collections::BTreeSet<SlotNumber> = frozen
        .writes()
        .units
        .iter()
        .map(|unit| segment_start_of(unit_area_start, unit.slot))
        .collect();
    assert!(
        segments_of_the_frozen_units.contains(&judged_segment),
        "前提：冻结那次的数据单元落进被判的段 {}（之下的空槽都隔离了），实际落点所在的段 {:?}",
        judged_segment.0,
        segments_of_the_frozen_units
    );
    let txg_of_the_frozen_publish = frozen.checkpoint_txg();
    for device_map in &session.allocator.devices {
        assert!(
            device_map.segment_is_reusable_as_a_whole(judged_segment_index),
            "{:?}：分配器退回发布之前的那一份，被判的段仍整段可再分配",
            device_map.device
        );
    }

    let run = session
        .run_compaction(&mut devices)
        .expect("盘表是挂载的那一份、现行那一版有树");
    let discarded: Vec<(DeviceIdentity, SlotNumber)> = run
        .discards
        .applied
        .iter()
        .chain(run.discards.unsupported.iter())
        .chain(run.discards.failed.iter())
        .copied()
        .collect();
    assert!(
        discarded
            .iter()
            .all(|(_, segment)| !segments_of_the_frozen_units.contains(segment)),
        "discard 发到了装着冻结那次（txg {txg_of_the_frozen_publish:?}）落点的段上：发了 {discarded:?}，冻结那次落点所在的段 {segments_of_the_frozen_units:?}"
    );
    assert_eq!(
        run.discards
            .txg_of_the_frozen_publish_withholding_every_discard,
        Some(txg_of_the_frozen_publish),
        "冻结着的时候一段都不发，交回的 discard 账点名冻结那次的 txg"
    );
}
