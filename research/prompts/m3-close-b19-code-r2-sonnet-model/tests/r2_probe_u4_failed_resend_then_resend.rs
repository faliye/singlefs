//! 辩方腿 m3-close-b19-code-r2-sonnet 的探针（U4）：冻结之后重发先失败一次、再成功，之后进 discard 调用点（整理显式入口、用户改动发成之后）。
//! 历史搭建逐字取自 `discard_after_the_frozen_publish_is_resent_reaches_the_segments_reclaimed_before_the_freeze.rs`。

mod common;

use std::collections::BTreeSet;

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
use singlefs_core::transaction::PublishError;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
use singlefs_harness::fault_injection::{
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

struct FrozenHistory {
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    session: MountedSession,
    devices: Vec<(DeviceIdentity, FaultInjectingDevice)>,
    plan: SharedFaultPlan,
    segments_of_the_large_version: BTreeSet<SlotNumber>,
    segments_of_the_frozen_units: BTreeSet<SlotNumber>,
    device_identities: Vec<DeviceIdentity>,
    unit_area_start: u64,
    write_time_seconds: u64,
}

fn frozen_history() -> FrozenHistory {
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

    let _ = txg_of_the_frozen_publish;
    FrozenHistory { parameters, session, devices, plan, segments_of_the_large_version, segments_of_the_frozen_units, device_identities, unit_area_start, write_time_seconds }
}

fn assert_the_large_version_segments_were_discarded_on_every_device(
    history: &FrozenHistory,
    discards: &DiscardsAfterThePublishes,
    label: &str,
) {
    let discarded = segments_discarded(discards);
    assert!(
        discarded
            .iter()
            .all(|(_, segment)| !history.segments_of_the_frozen_units.contains(segment)),
        "{label}：装着冻结那次落点的段不发 discard：发了 {discarded:?}"
    );
    let still_reusable: Vec<SlotNumber> = history
        .segments_of_the_large_version
        .iter()
        .copied()
        .filter(|segment_start| !history.segments_of_the_frozen_units.contains(segment_start))
        .filter(|segment_start| {
            history.session.allocator.devices.iter().all(|device_map| {
                device_map.segment_is_reusable_as_a_whole(segment_index_of(
                    history.unit_area_start,
                    *segment_start,
                ))
            })
        })
        .collect();
    assert!(!still_reusable.is_empty(), "{label}：前提：至少一段仍整段可再分配");
    let not_discarded: Vec<(DeviceIdentity, SlotNumber)> = still_reusable
        .iter()
        .flat_map(|segment_start| {
            history
                .device_identities
                .iter()
                .map(move |device| (*device, *segment_start))
        })
        .filter(|device_and_segment| !discarded.contains(device_and_segment))
        .collect();
    println!("PROBE name={label} discarded={} still_reusable={}", discarded.len(), still_reusable.len());
    assert!(not_discarded.is_empty(), "{label}：没发的 {not_discarded:?}，发了 {discarded:?}");
}

fn resend(history: &mut FrozenHistory) -> Result<(), PublishError> {
    let mut writer = PoolWriter::new(&history.parameters, history.devices.as_mut_slice());
    match resend_the_frozen_publish(&mut writer, &mut history.session.allocator)? {
        Some(version) => {
            history.session.current = version;
            Ok(())
        }
        None => panic!("分配器上冻结着一次：重发该交回那一版"),
    }
}

/// 冻结 → 重发先因系统配置槽轮换再报错而失败一次（那次发布照旧冻结）→ 冻结着调整理入口仍一段不发 → 故障撤了重发成功 → 整理显式入口。
#[test]
fn u4_a_resend_that_fails_once_keeps_the_pending_discards_for_the_compaction_run_after_the_good_resend() {
    let mut history = frozen_history();
    history.plan.arm(the_next_system_configuration_write_on_device_zero_fails());
    let first_resend = resend(&mut history);
    history.plan.disarm();
    println!("PROBE name=u4a_first_resend result={:?}", first_resend.as_ref().map(|()| "ok").map_err(|e| format!("{e:?}").chars().take(60).collect::<String>()));
    assert!(first_resend.is_err(), "前提：重发时轮换那一写又报错");
    assert!(history.session.allocator.frozen_publish().is_some(), "失败的重发之后那次发布照旧冻结");
    let run_while_still_frozen = history.session.run_compaction(&mut history.devices).expect("冻结着");
    assert!(segments_discarded(&run_while_still_frozen.discards).is_empty(), "重发失败之后仍冻结：一段不发");
    resend(&mut history).expect("故障撤了：重发成功");
    assert!(history.session.allocator.frozen_publish().is_none());
    let run = history.session.run_compaction(&mut history.devices).expect("重发之后");
    assert_the_large_version_segments_were_discarded_on_every_device(&history, &run.discards, "u4a_after_failed_then_good_resend");
}

/// 冻结 → 重发成功 → 用户改动发成（NewInodes）：discard 调用点是 `publish_user_change` 末尾那一处。
#[test]
fn u4_b_the_user_change_after_the_resend_discards_the_segments_reclaimed_before_the_freeze() {
    let mut history = frozen_history();
    resend(&mut history).expect("故障撤了：重发成功");
    history.write_time_seconds += 1;
    let published = history
        .session
        .publish_user_change(
            &mut history.devices,
            UserChange::NewInodes { count: 1, write_time_seconds: history.write_time_seconds },
        )
        .expect("重发之后建一个 inode");
    let discarded = segments_discarded(&published.discards_after_the_change);
    let mut reusable_now_but_not_discarded = Vec::new();
    for segment_start in history
        .segments_of_the_large_version
        .iter()
        .filter(|segment| !history.segments_of_the_frozen_units.contains(segment))
    {
        let index = segment_index_of(history.unit_area_start, *segment_start);
        for device_map in &history.session.allocator.devices {
            let reusable = device_map.segment_is_reusable_as_a_whole(index);
            let was_discarded = discarded.contains(&(device_map.device, *segment_start));
            println!("PROBE name=u4b_segment device={:?} segment={} reusable_after_the_change={reusable} discarded_by_the_change={was_discarded}", device_map.device, segment_start.0);
            if reusable && !was_discarded {
                reusable_now_but_not_discarded.push((device_map.device, *segment_start));
            }
        }
    }
    assert!(
        discarded.iter().all(|(_, segment)| !history.segments_of_the_frozen_units.contains(segment)),
        "装着冻结那次落点的段不发 discard"
    );
    println!("PROBE name=u4b_summary discarded={} reusable_now_but_not_discarded={reusable_now_but_not_discarded:?}", discarded.len());
    assert!(
        reusable_now_but_not_discarded.is_empty(),
        "用户改动发成之后仍整段可再分配的段都发了 discard（被这次改动占去一槽的段不发）：没发的 {reusable_now_but_not_discarded:?}"
    );
}
