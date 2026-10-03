//! 攻方腿原型（m3-unmount-after-short-continuation-r1，Opus）：只在冻结副本的拷贝上跑，不进仓。
//! 环境变量 PROBE_MODE = narrow | allowance，PROBE_SLOTS = 逗号分隔的单元区槽数，PROBE_OVERWRITES、PROBE_UNITS（max = 一单元一单元长到被拒）、
//! PROBE_BATCHES（崩在意图之后第几批；u-1 = 最后一批之前）、PROBE_ARM = today | jia（甲：在本副本 transaction.rs 里实现，见 jia.patch）。
//! 每格一行 `CELL …`。
#![allow(clippy::too_many_lines, clippy::cast_possible_truncation)]

mod common_admission;

use common_admission::{
    content_of, start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
    DeletionQueueContinuationOutcome, FloorRaiseStop, MountError, MountSpaceAdmission,
    ParametersAndDeviceTableOfTheMount, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_unlink_intent, publish_unlink_step, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use std::collections::BTreeMap;

fn switches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(1).expect("1"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(str::to_string)
        .collect()
}

/// 准入一直判着的池（补足一侧）：单元区 `slots` 槽，参数取 384 槽那一档（6 MiB 环）。第一个文件被拒就交回 None。
fn start_with_admission_on(slots: u64) -> Option<PoolUnderTest<SparseBlockDevice>> {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let start = singlefs_core::allocator::UnitAreaStart::following_the_journal_ring(
        parameters.geometry.journal_ring_bytes,
    )
    .expect("起点")
    .slot()
    .0;
    let device_bytes = (start + slots) * SLOT_BYTES;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| (identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).ok()?;
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    allocator.set_deletion_switches(switches_of_one());
    let first_content = content_of(3000, 0);
    let (instance, first_version) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).ok()?;
        let warmed = warm_up(&mut writer, &genesis.root, instance).ok()?;
        let first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last()?,
            FirstFile { content: &first_content, write_time_seconds: FIRST_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .ok()?;
        (instance, first_version)
    };
    let parameters_and_device_table = ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices).ok()?;
    Some(PoolUnderTest {
        parameters,
        device_bytes,
        devices,
        session: Some(MountedSession {
            allocator,
            current: PoolVersion::WithFile(first_version),
            instance,
            shadow_ledger: ShadowLedger::On,
            parameters_and_device_table,
        }),
        write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: first_content,
        content_of_each_root: BTreeMap::new(),
    })
}

fn kind_of_publish_error(error: &PublishError) -> String {
    let text = format!("{error:?}");
    text.split(['(', ' ', '{']).next().unwrap_or("?").to_string()
}

fn kind_of_mount_error(error: &MountError) -> String {
    match error {
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_sequence,
            publishes_in_the_sequence,
            cause,
        } => format!(
            "Rehearsal[{refused_publish_in_the_sequence}/{publishes_in_the_sequence}:{}]",
            kind_of_publish_error(cause)
        ),
        other => {
            let text = format!("{other:?}");
            text.split(['(', ' ', '{']).next().unwrap_or("?").to_string()
        }
    }
}

fn stop_kind(stop: &FloorRaiseStop) -> String {
    match stop {
        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { publishes_pushed, publishes_of_the_next_raise } => {
            format!("Budget({publishes_pushed}+{publishes_of_the_next_raise})")
        }
        FloorRaiseStop::FloorAlreadyAtTheCeiling { floor, ceiling } => format!("AtCeiling({}/{})", floor.0, ceiling.0),
        FloorRaiseStop::FloorRaiseRefusedForSpace { refusal } => format!("RaiseRefused[{}]", kind_of_mount_error(refusal)),
    }
}

fn unblocked(session: &MountedSession) -> Vec<u64> {
    session.allocator.devices.iter().map(|map| map.unblocked_slots()).collect()
}

/// 卸载在拷贝上做一次：结局、以及做成时这一串每块盘新占的槽数（各次发布重写的角色的跨度之和）。
fn unmount_on_a_copy(
    pool: &PoolUnderTest<SparseBlockDevice>,
    session: &MountedSession,
    reclaim_everything_first: bool,
) -> (String, u64) {
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
        .devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect();
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    if reclaim_everything_first {
        allocator.reclaim_released_up_to(
            current.root().checkpoint_txg,
            singlefs_core::allocator::ReclaimedReuse::Immediately,
        );
    }
    match unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            let taken: u64 = raised
                .publishes
                .iter()
                .map(|publish| publish.rewritten.iter().map(|role| role.span_slots()).sum::<u64>())
                .sum();
            (format!("ok[{}]", raised.publishes.len()), taken)
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => ("ok-nothing".to_string(), 0),
        Err(error) => (format!("err[{}]", kind_of_mount_error(&error)), 0),
    }
}

fn devices_copy(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices_copy_of(&pool.devices, pool.device_bytes)
}

fn devices_copy_of(devices: &[(DeviceIdentity, SparseBlockDevice)], device_bytes: u64) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy = SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

fn session_copy(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

/// 池的一份拷贝（盘逐字节、会话逐项）：在它上面试写不动原来那份。
fn copy_of(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: devices_copy(pool),
        session: pool.session.as_ref().map(session_copy),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: BTreeMap::new(),
    }
}

/// 一格：建池、覆盖写、长文件、意图与前几批、崩、重挂、看续做与卸载；交回一行。
fn one_cell(mode: &str, slots: u64, overwrites: usize, units: &str, batches: &str) -> String {
    let mut pool = match mode {
        "narrow" => {
            let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                slots,
            );
            pool.session.as_mut().expect("会话").allocator.set_deletion_switches(switches_of_one());
            pool
        }
        _ => match start_with_admission_on(slots) {
            Some(pool) => pool,
            None => return format!("CELL {mode} S{slots} O{overwrites} U{units} B{batches} setup=first-file-refused"),
        },
    };
    let mut overwrites_done = 0;
    for _ in 0..overwrites {
        if pool.overwrite(OVERWRITE_BYTES).is_err() {
            break;
        }
        overwrites_done += 1;
    }
    let capacity = data_unit_payload_capacity();
    let units_reached = if units == "max" {
        // 一次顺序写从 1 个单元长到 n 个：在池的拷贝上二分找准入放行的最大 n，再在池上真写（一单元一单元长会把换下的单元攒在 defer 里）。
        let (mut low, mut high) = (1_usize, usize::try_from(slots).expect("槽数") / 2 + 1);
        while low < high {
            let middle = (low + high).div_ceil(2);
            let mut copy = copy_of(&pool);
            if copy.sequential_write((middle - 1) * capacity + 100).is_ok() {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        if low > 1 {
            pool.sequential_write((low - 1) * capacity + 100).expect("拷贝上放行的那一次在池上同样放行");
        }
        low
    } else {
        let target: usize = units.parse().expect("单元数");
        if pool.sequential_write((target - 1) * capacity + 100).is_ok() {
            target
        } else {
            // 一次长不到：一单元一单元长，停在被拒之前。
            let mut reached = 1;
            while reached < target && pool.sequential_write(reached * capacity + 100).is_ok() {
                reached += 1;
            }
            reached
        }
    };
    let batches_wanted: usize = match batches {
        "half" => units_reached / 2,
        "last" => units_reached.saturating_sub(1),
        other => other.parse().expect("批数"),
    };
    let (setup_note, batches_done) = {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut current = session.current.version_of_trees().expect("有树").clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance) {
            Ok(next) => {
                current = next;
                let mut done = 0;
                let mut note = "ok".to_string();
                while done < batches_wanted {
                    match publish_unlink_step(
                        &mut writer,
                        &mut session.allocator,
                        &current,
                        InodeNumber(FIRST_INODE_NUMBER),
                        switches_of_one().batch_size,
                        session.instance,
                    ) {
                        Ok(next) => {
                            current = next;
                            done += 1;
                        }
                        Err(error) => {
                            note = format!("batch-refused[{}]", kind_of_publish_error(&error));
                            break;
                        }
                    }
                }
                (note, done)
            }
            Err(error) => (format!("intent-refused[{}]", kind_of_publish_error(&error)), 0),
        }
    };
    if setup_note.starts_with("intent-refused") {
        return format!(
            "CELL {mode} S{slots} O{overwrites}/{overwrites_done} U{units}/{units_reached} B{batches}/{batches_done} setup={setup_note}"
        );
    }
    pool.session = None;
    let image_at_the_crash = pool.image();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches_of_one(),
    );
    let mounted = match mounted {
        Ok(mounted) => mounted,
        Err(error) => {
            return format!(
                "CELL {mode} S{slots} O{overwrites}/{overwrites_done} U{units}/{units_reached} B{batches}/{batches_done} setup={setup_note} mount-err[{}]",
                kind_of_mount_error(&error)
            )
        }
    };
    let admission = match &mounted.output.space_admission {
        MountSpaceAdmission::AdmittedBeforeAcquisition => "pre-ok".to_string(),
        MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. } => format!("after-raises[{}]", floor_raises.len()),
        MountSpaceAdmission::StillShortAfterTheFloorRaises { stop, .. } => format!("mount-short[{}]", stop_kind(stop)),
        other => format!("{other:?}").chars().take(30).collect(),
    };
    let steps = mounted.output.deletion_queue_continuation.len();
    // 续做每一步每块盘新占的槽（这一步重写的角色的跨度之和），取最大；与环窗口用的 W 比。
    let max_step_slots: u64 = mounted
        .output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.rewritten.iter().map(|role| role.span_slots()).sum::<u64>())
        .max()
        .unwrap_or(0);
    let raises = mounted.output.floor_raises_pushed_by_the_continuation.len();
    let outcome = match &mounted.output.deletion_queue_continuation_outcome {
        DeletionQueueContinuationOutcome::WalkedToEmpty => "Empty".to_string(),
        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { last_refusal, stop } => {
            format!("Short[{}|{}]", kind_of_publish_error(last_refusal), stop_kind(stop))
        }
    };
    let (_, session) = MountedSession::of_the_mount(mounted);
    let unblocked_now = unblocked(&session);
    let ckpt = singlefs_core::admission::checkpoint_cost_of_the_version_to_build_on(
        session.current.version_of_trees(),
        &session.allocator,
    )
    .0;
    let (unmount_result, _) = unmount_on_a_copy(&pool, &session, false);
    let mut line = format!(
        "CELL {mode} S{slots} O{overwrites}/{overwrites_done} U{units}/{units_reached} B{batches}/{batches_done} setup={setup_note} adm={admission} steps={steps} max-step-slots={max_step_slots} raises={raises} outcome={outcome} unblocked={unblocked_now:?} ckpt={ckpt} unmount={unmount_result}"
    );
    if !unmount_result.starts_with("ok") {
        let (relaxed, need) = unmount_on_a_copy(&pool, &session, true);
        // 每个单元落每块盘一份：need 是每块盘的槽数。
        let short: Vec<i64> = unblocked_now.iter().map(|free| need as i64 - *free as i64).collect();
        line.push_str(&format!(" need-per-device-on-relaxed-copy={relaxed}:{need} need-minus-unblocked={short:?}"));
        // 丙1 的循环：卸载那一串回收出来的槽不扣住（拷贝上先按卸载的回收门槛回收、立刻可用，再卸载），之后再挂；最多 4 圈。
        let mut bing_devices = devices_copy(&pool);
        let mut bing_session = session_copy(&session);
        let mut bing_cycles = Vec::new();
        for _ in 0..4 {
            bing_session.allocator.reclaim_released_up_to(
                bing_session.current.root().checkpoint_txg,
                singlefs_core::allocator::ReclaimedReuse::Immediately,
            );
            match unmount(&pool.parameters, &mut bing_devices, &mut bing_session.allocator, &mut bing_session.current, bing_session.shadow_ledger) {
                Ok(_) => bing_cycles.push("u-ok".to_string()),
                Err(error) => {
                    bing_cycles.push(format!("u-err:{}", kind_of_mount_error(&error)));
                    break;
                }
            }
            match mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                &mut bing_devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                switches_of_one(),
            ) {
                Ok(again) => {
                    let steps_again = again.output.deletion_queue_continuation.len();
                    let empty = matches!(
                        again.output.deletion_queue_continuation_outcome,
                        DeletionQueueContinuationOutcome::WalkedToEmpty
                    );
                    bing_cycles.push(format!("m-ok:{steps_again}{}", if empty { ":empty" } else { "" }));
                    let (_, next_session) = MountedSession::of_the_mount(again);
                    bing_session = next_session;
                    if empty {
                        break;
                    }
                }
                Err(error) => {
                    bing_cycles.push(format!("m-err:{}", kind_of_mount_error(&error)));
                    break;
                }
            }
        }
        line.push_str(&format!(" bing1-loop={bing_cycles:?}"));
    }
    if outcome.starts_with("Short") {
        // 之后一次用户改动（覆盖写 100 字节）。
        let mut change_session = MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        };
        let mut change_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
            .devices
            .iter()
            .map(|(identity, device)| {
                let mut copy = SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512));
                copy.image = device.image.clone();
                (*identity, copy)
            })
            .collect();
        let content = vec![9_u8; 100];
        let change = change_session.publish_user_change(
            &mut change_devices,
            UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: FIRST_WRITE_TIME_SECONDS + 99 }),
        );
        let change_kind = match &change {
            Ok(_) => "ok".to_string(),
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => {
                format!("nospace[{}]", stop_kind(&no_space.stop))
            }
            Err(other) => format!("{other:?}").chars().take(40).collect(),
        };
        line.push_str(&format!(" change={change_kind}"));
        // 那一次用户改动之后（会话走空里推过的抬 F 留在拷贝上）再卸载。
        let after_change = PoolUnderTest {
            parameters: pool.parameters.clone(),
            device_bytes: pool.device_bytes,
            devices: change_devices,
            session: None,
            write_time_seconds: pool.write_time_seconds,
            content_of_the_current_version: Vec::new(),
            content_of_each_root: BTreeMap::new(),
        };
        let (unmount_after_change, _) = unmount_on_a_copy(&after_change, &change_session, false);
        line.push_str(&format!(" unmount-after-change={unmount_after_change}"));
        // 卸载—再挂循环（今天与甲的走法：卸载照抬 F）：最多 4 圈，每圈记卸载结局与再挂之后续做停在哪。
        let mut cycle_devices = devices_copy_of(&pool.devices, pool.device_bytes);
        let mut cycle_session = session_copy(&session);
        let mut cycles_with_unmount = Vec::new();
        for _ in 0..4 {
            match unmount(&pool.parameters, &mut cycle_devices, &mut cycle_session.allocator, &mut cycle_session.current, cycle_session.shadow_ledger) {
                Ok(_) => cycles_with_unmount.push("u-ok".to_string()),
                Err(error) => {
                    cycles_with_unmount.push(format!("u-err:{}", kind_of_mount_error(&error)));
                    break;
                }
            }
            match mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                &mut cycle_devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                switches_of_one(),
            ) {
                Ok(again) => {
                    let steps_again = again.output.deletion_queue_continuation.len();
                    let empty = matches!(
                        again.output.deletion_queue_continuation_outcome,
                        DeletionQueueContinuationOutcome::WalkedToEmpty
                    );
                    cycles_with_unmount.push(format!("m-ok:{steps_again}{}", if empty { ":empty" } else { "" }));
                    let (_, next_session) = MountedSession::of_the_mount(again);
                    cycle_session = next_session;
                    if empty {
                        break;
                    }
                }
                Err(error) => {
                    cycles_with_unmount.push(format!("m-err:{}", kind_of_mount_error(&error)));
                    break;
                }
            }
        }
        line.push_str(&format!(" unmount-loop={cycles_with_unmount:?}"));
        // 乙那一臂的循环：卸载一个字节都不写 = 盘停在挂载之后；再可写挂载最多 4 圈，每圈记续做走了几步、队列还剩几条、卸载结局。
        let mut loop_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
            .devices
            .iter()
            .map(|(identity, device)| {
                let mut copy = SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512));
                copy.image = device.image.clone();
                (*identity, copy)
            })
            .collect();
        let mut cycles = Vec::new();
        for _ in 0..4 {
            match mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                &mut loop_devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                switches_of_one(),
            ) {
                Ok(again) => {
                    let steps_again = again.output.deletion_queue_continuation.len();
                    let empty = matches!(
                        again.output.deletion_queue_continuation_outcome,
                        DeletionQueueContinuationOutcome::WalkedToEmpty
                    );
                    cycles.push(format!("ok:{steps_again}{}", if empty { ":empty" } else { "" }));
                    if empty {
                        break;
                    }
                }
                Err(error) => {
                    cycles.push(format!("err:{}", kind_of_mount_error(&error)));
                    break;
                }
            }
        }
        line.push_str(&format!(" yi-loop={cycles:?}"));
    }
    let _ = image_at_the_crash;
    line
}

#[test]
fn unmount_attack_probe_sweep() {
    let mode = std::env::var("PROBE_MODE").unwrap_or_else(|_| "narrow".to_string());
    for slots in env_list("PROBE_SLOTS", "256") {
        for overwrites in env_list("PROBE_OVERWRITES", "0") {
            for units in env_list("PROBE_UNITS", "14") {
                for batches in env_list("PROBE_BATCHES", "1") {
                    let line = one_cell(
                        &mode,
                        slots.parse().expect("槽数"),
                        overwrites.parse().expect("覆盖写次数"),
                        &units,
                        &batches,
                    );
                    println!("{line}");
                }
            }
        }
    }
}
