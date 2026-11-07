//! 冻结解除之后，冻结期间积下的 discard 在下一次进 discard 调用点时发出、不丢（C928（冻结着时整理照发 discard）；
//! 「冻结解除之后积下的 discard 何时发」条款没写，这条用例钉住；
//! 钉的行为：重发成功之后分配器整份换成冻结那次成立之后的那一份，
//! 待 discard 表 = 冻结之前积下的 + 那次发布自己回收的；重发本身不发 discard，留给下一次进三个调用点之一）。
//!
//! 历史与 `discard_while_a_publish_is_frozen_after_a_system_configuration_write_failure.rs` 同一段（`common::build_pool` 起的池，重开可写挂载、开会话）：
//! 1. 顺序写第一个文件 128 个数据单元：几整段 64 槽全是这一版的数据单元；
//! 2. 顺序写第一个文件回到一个数据单元：那些数据单元整份换下、进 defer；
//! 3. 建四次 inode：准入抬 F 的上限越过第 2 步的 txg；
//! 4. 直接调公开的准入抬 F：回收第 2 步换下的单元，那几整段可再分配、记进待 discard；
//! 5. 这几段里最低那一段之下每块盘还空着的槽都隔离（只动分配器的隔离位，不写盘）：下一次发布的数据单元落进最低那一段；
//! 6. 盘 0 的下一次系统配置槽写报块设备错，顺序写一个数据单元：单元、记录、根槽都落，轮换那一写报错，发布冻结；
//! 7. 冻结着调整理的显式入口：一段都不发；
//! 8. 原样重发冻结的那一次（`transaction::resend_the_frozen_publish`，调用方拆开会话自己重发），现行那一版换成重发交回的那一版；
//! 9. 再调整理的显式入口。
mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use common::{build_pool, geometry, parameters, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling,
};
use singlefs_core::mounted_session::{
    DiscardsAfterThePublishes, MountedSession, UserChange, UserChangeRefused,
};
use singlefs_core::transaction::{
    resend_the_frozen_publish, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_verification::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};

type FaultInjectingDevice = FaultInjectingBlockDevice<Recorded>;

/// 第 1 步顺序写的数据单元数：两槽一个，256 槽，落在用户数据最低的空槽对上，余下的连续往上，盖满几整段 64 槽。
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

fn segment_index_of(unit_area_start: u64, segment_start: SlotNumber) -> usize {
    usize::try_from((segment_start.0 - unit_area_start) / SLOTS_PER_SEGMENT)
        .expect("段号装得进 usize")
}

/// 这一版第一个文件的数据单元整段盖满的每一段（段里 32 个槽对都是这一版的数据单元），按起点升序。
fn segments_holding_only_data_units_of(
    version: &TransactionOutput,
    unit_area_start: u64,
) -> BTreeSet<SlotNumber> {
    let data_unit_slots: BTreeSet<u64> = version
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations[0].slot.0)
        .collect();
    data_unit_slots
        .iter()
        .map(|slot| segment_start_of(unit_area_start, SlotNumber(*slot)))
        .filter(|segment_start| {
            (segment_start.0..segment_start.0 + SLOTS_PER_SEGMENT)
                .step_by(2)
                .all(|slot| data_unit_slots.contains(&slot))
        })
        .collect()
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

/// 一次整理运行交回的 discard 账里发出去的段（交回 `Applied`、`Unsupported` 与报错的都算发了）。
fn segments_discarded(
    discards: &DiscardsAfterThePublishes,
) -> BTreeSet<(DeviceIdentity, SlotNumber)> {
    discards
        .applied
        .iter()
        .chain(discards.unsupported.iter())
        .chain(discards.failed.iter())
        .copied()
        .collect()
}

/// 场景：冻结着一次发布时调整理入口（一段都不发），原样重发冻结的那一次，再调整理入口。
/// 预期：重发之后这一次整理把冻结之前抬 F 回收、记进待 discard、整段仍可再分配的每一段在每块盘上都发了 discard；
/// 装着冻结那次落点的段一段都不发；这一次交回的 discard 账不再点名冻结那次的 txg。
/// 判别力：重发成功之后清空待 discard 表（`crates/mutations.tsv`「C928 重发冻结的那一次成功之后清空待 discard 表（积下的 discard 丢了）」）
/// 时这几段一段都不发，这一条红在「冻结之前回收的那几段重发之后发了 discard」。
#[test]
fn discards_accumulated_while_a_publish_is_frozen_reach_the_segments_reclaimed_before_the_freeze_at_the_compaction_run_after_the_resend(
) {
    let parameters = parameters();
    let mut pool = build_pool("discard-after-the-frozen-publish-is-resent");
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
    .expect("顺序写 128 个数据单元");
    let segments_of_the_large_version =
        segments_holding_only_data_units_of(file_version_of(&session), unit_area_start);
    let lowest_segment_of_the_large_version = *segments_of_the_large_version
        .first()
        .expect("这一版的数据单元连续落在用户数据最低的空槽对上，至少盖满一整段");

    write_time_seconds += 1;
    publish_sequential_write_of_the_first_file(
        &mut session,
        &mut devices,
        &content_of(SMALL_CONTENT_BYTES, 0x22),
        write_time_seconds,
    )
    .expect("顺序写回一个数据单元：128 个数据单元整份换下");

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
    assert!(
        matches!(raised, RaiseToTheAdmissionCeiling::Raised(_)),
        "四个非空的状态之后上限高于 F：推了抬 F，实际 {raised:?}"
    );
    for segment_start in &segments_of_the_large_version {
        for device_map in &session.allocator.devices {
            assert!(
                device_map.segment_is_reusable_as_a_whole(segment_index_of(
                    unit_area_start,
                    *segment_start
                )),
                "{:?}：抬 F 回收之后段 {} 整段可再分配（记进待 discard）",
                device_map.device,
                segment_start.0
            );
        }
    }

    let device_identities: Vec<DeviceIdentity> = session
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.device)
        .collect();
    for device in &device_identities {
        session
            .allocator
            .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(
                *device,
                SlotNumber(unit_area_start),
                lowest_segment_of_the_large_version.0 - unit_area_start,
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
    let txg_of_the_frozen_publish = frozen.checkpoint_txg();
    let segments_of_the_frozen_units: BTreeSet<SlotNumber> = frozen
        .writes()
        .units
        .iter()
        .map(|unit| segment_start_of(unit_area_start, unit.slot))
        .collect();

    let run_while_frozen = session
        .run_compaction(&mut devices)
        .expect("冻结着：盘表是挂载的那一份、现行那一版有树");
    assert_eq!(
        run_while_frozen
            .discards
            .txg_of_the_frozen_publish_withholding_every_discard,
        Some(txg_of_the_frozen_publish),
        "前提：冻结着的那一次整理一段都不发，交回的 discard 账点名冻结那次的 txg"
    );
    assert!(
        segments_discarded(&run_while_frozen.discards).is_empty(),
        "前提：冻结着的那一次整理一段都没发：{:?}",
        run_while_frozen.discards
    );

    let resent = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        resend_the_frozen_publish(&mut writer, &mut session.allocator)
            .expect("故障撤了，原样重发冻结的那一次")
    };
    session.current = resent.expect("分配器上冻结着一次：重发交回那一版");
    assert!(
        session.allocator.frozen_publish().is_none(),
        "重发成功之后不再冻结"
    );

    let run_after_the_resend = session
        .run_compaction(&mut devices)
        .expect("重发之后：盘表是挂载的那一份、现行那一版有树");
    assert_eq!(
        run_after_the_resend
            .discards
            .txg_of_the_frozen_publish_withholding_every_discard,
        None,
        "重发之后没有冻结着的发布，discard 不再被扣着"
    );
    let discarded = segments_discarded(&run_after_the_resend.discards);
    assert!(
        discarded
            .iter()
            .all(|(_, segment)| !segments_of_the_frozen_units.contains(segment)),
        "装着冻结那次（重发之后落定的那一次）落点的段不发 discard：发了 {discarded:?}，那次落点所在的段 {segments_of_the_frozen_units:?}"
    );
    let segments_still_reusable_as_a_whole: Vec<SlotNumber> = segments_of_the_large_version
        .iter()
        .copied()
        .filter(|segment_start| !segments_of_the_frozen_units.contains(segment_start))
        .filter(|segment_start| {
            session.allocator.devices.iter().all(|device_map| {
                device_map.segment_is_reusable_as_a_whole(segment_index_of(
                    unit_area_start,
                    *segment_start,
                ))
            })
        })
        .collect();
    assert!(
        !segments_still_reusable_as_a_whole.is_empty(),
        "前提：冻结之前回收、不装冻结那次落点的段里至少有一段到这一次整理之后仍整段可再分配（第 1 步盖满了几整段）：大文件那几段 {segments_of_the_large_version:?}，冻结那次落点所在的段 {segments_of_the_frozen_units:?}"
    );
    let not_discarded: Vec<(DeviceIdentity, SlotNumber)> = segments_still_reusable_as_a_whole
        .iter()
        .flat_map(|segment_start| {
            device_identities
                .iter()
                .map(move |device| (*device, *segment_start))
        })
        .filter(|device_and_segment| !discarded.contains(device_and_segment))
        .collect();
    assert!(
        not_discarded.is_empty(),
        "冻结之前回收的那几段重发之后发了 discard、不丢：没发的（盘, 段起点）{not_discarded:?}，这一次发了 {discarded:?}"
    );
}
