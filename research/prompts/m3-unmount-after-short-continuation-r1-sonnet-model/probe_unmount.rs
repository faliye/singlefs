//! 草稿探针（正推腿，副本里的临时测试，不进仓）：续做推满仍放不下之后正常卸载做不做得成。
mod common_admission;

use common_admission::{
    plain_devices_on, start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::{ring_window_on_one_device, SpaceAdmission};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, unmount,
    DeletionQueueContinuationOutcome, FloorRaiseStop, MountError, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, PublishError,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{
    DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;

#[derive(Clone, Copy, Debug)]
enum Disk {
    Pad(HistoryDeviceWidth),
    Narrow(u64),
    /// 准入判着的盘，单元区 N 槽，第一个文件一次发布就是 L 个数据单元（第一个文件版本那次发布不扣环窗口）
    First(u64, u64),
}

fn switches(k: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: if k >= 144 {
            UnlinkBatchSize::OneExtentLowerLeaf
        } else {
            UnlinkBatchSize::CappedForTests(
                UnlinkBatchDataUnitsCappedForTests::new(k).expect("k 在范围内"),
            )
        },
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn violations_of(devs: &[(singlefs_core::address::DeviceIdentity, SparseBlockDevice)], device_bytes: u64) -> String {
    let image = singlefs_harness::memory_pool::MemoryPool {
        devices: devs.iter().map(|(id, d)| (*id, d.image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    };
    let v = common_admission::checker_violations_on(&image);
    if v.is_empty() { "green".to_string() } else { format!("RED{}:{:.60}", v.len(), v[0]) }
}

fn stats(allocator: &singlefs_core::allocator::PoolAllocator) -> String {
    allocator
        .devices
        .iter()
        .map(|d| {
            format!(
                "d{}[free={} unblocked={} deferred={} alloc={}]",
                d.device.0,
                d.free_slots(),
                d.unblocked_slots(),
                d.deferred_slots(),
                d.allocated_slots()
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn shortage(error: &MountError) -> String {
    match error {
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_sequence,
            publishes_in_the_sequence,
            cause,
        } => format!(
            "RehearsalRefused(pub {}/{}; {})",
            refused_publish_in_the_sequence,
            publishes_in_the_sequence,
            short_of(cause)
        ),
        other => format!("{:.80?}", other),
    }
}

fn short_of(cause: &PublishError) -> String {
    match cause {
        PublishError::SpaceAdmissionRefused(refused) => refused
            .short_devices
            .iter()
            .map(|s| {
                format!(
                    "Adm d{} avail={}sl demand={}sl missing={}sl",
                    s.device.0,
                    s.available.0 / 16384,
                    s.demand.0 / 16384,
                    (i128::from(s.demand.0) - s.available.0) / 16384
                )
            })
            .collect::<Vec<_>>()
            .join(";"),
        PublishError::PlacementRefused { unit, refusal } => format!("Placement {unit:?} {refusal:?}"),
        other => format!("{:.80?}", other),
    }
}

fn run_cell(disk: Disk, overwrites: usize, units: u64, crash_batches: usize, k: u64) -> String {
    let tag = format!("{disk:?} O{overwrites} U{units} B{crash_batches} K{k}");
    let mut pool: PoolUnderTest<SparseBlockDevice> = match disk {
        Disk::Pad(width) => {
            let mut pool = start_plain(width);
            if let Err(e) = pool.crash_and_mount_writable() {
                return format!("CELL {tag} => setup-mount-err {:.60?}", e);
            }
            pool
        }
        Disk::First(n, l) => match build_first_big(n, l) {
            Ok(pool) => pool,
            Err(message) => return format!("CELL {tag} => first-file-refused {message}"),
        },
        Disk::Narrow(n) => {
            start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                n,
            )
        }
    };
    for index in 1..=overwrites {
        if let Err(e) = pool.overwrite(OVERWRITE_BYTES) {
            return format!("CELL {tag} => overwrite{index}-refused {:.60?}", e);
        }
    }
    if units > 1 {
        let length = (usize::try_from(units).unwrap() - 1) * data_unit_payload_capacity() + 100;
        if let Err(e) = pool.sequential_write(length) {
            return format!("CELL {tag} => grow-refused {:.70?}", e);
        }
    }
    let rw = {
        let session = pool.session.as_ref().unwrap();
        ring_window_on_one_device(
            &session.allocator,
            pool.parameters.geometry.root_ring_slots_per_region,
            k.min(144),
        )
        .0
            / 16384
    };
    let ckpt = {
        let session = pool.session.as_ref().unwrap();
        singlefs_core::admission::checkpoint_cost_of_the_version_to_build_on(
            session.current.version_of_trees(),
            &session.allocator,
        )
        .0
    };
    let before_unlink = format!("ckpt_cost={ckpt} {}", stats(&pool.session.as_ref().unwrap().allocator));
    let mut direct_steps = 0;
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().unwrap();
        let mut current = session.current.version_of_trees().unwrap().clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            session.instance,
        ) {
            Ok(next) => current = next,
            Err(e) => return format!("CELL {tag} => intent-refused {}", short_of(&e)),
        }
        for _ in 0..crash_batches {
            match publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                switches(k).batch_size,
                session.instance,
            ) {
                Ok(next) => {
                    current = next;
                    direct_steps += 1;
                }
                Err(_) => break,
            }
        }
    }
    let at_crash = stats(&pool.session.as_ref().unwrap().allocator);
    pool.session = None;
    let mounted = match mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut pool.devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(k),
    ) {
        Ok(m) => m,
        Err(e) => {
            return format!(
                "CELL {tag} rw={rw} before={before_unlink} at-crash={at_crash} => mount-err {:.120?}",
                e
            )
        }
    };
    let outcome = match &mounted.output.deletion_queue_continuation_outcome {
        DeletionQueueContinuationOutcome::WalkedToEmpty => "Empty".to_string(),
        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { last_refusal, stop } => {
            let stop_kind = match stop {
                FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => "BudgetOut",
                FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => "AtCeiling",
                FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => "RaiseRefusedForSpace",
            };
            format!("Short[{stop_kind}|last={}]", short_of(last_refusal))
        }
    };
    let steps = mounted.output.deletion_queue_continuation.len();
    let raises = mounted.output.floor_raises_pushed_by_the_continuation.len();
    let queue_len = mounted
        .current
        .version_of_trees()
        .map_or(0, |v| v.deleted_inodes.len());
    let after_mount = stats(&mounted.allocator);
    let (_, session) = MountedSession::of_the_mount(mounted);
    // 卸载：在一份拷贝上试
    let image = pool.image();
    let mut devs = plain_devices_on(&image);
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let mut unmount_ok = false;
    let unmount_result = match unmount(&pool.parameters, &mut devs, &mut allocator, &mut current, session.shadow_ledger) {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(done)) => {
            unmount_ok = true;
            let slots: Vec<u64> = done
                .publishes
                .iter()
                .map(|p| p.rewritten.iter().map(|r| r.span_slots()).sum())
                .collect();
            format!("unmount-ok(publishes={} slots_each={:?})", done.publishes.len(), slots)
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "unmount-ok(nothing)".to_string(),
        Err(e) => format!("unmount-err[{}]", shortage(&e)),
    };
    let describe = |result: Result<singlefs_core::mount::Mounted, MountError>| -> String {
        match result {
            Ok(m) => format!(
                "ok({} steps={} raises={})",
                match &m.output.deletion_queue_continuation_outcome {
                    DeletionQueueContinuationOutcome::WalkedToEmpty => "Empty",
                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => "Short",
                },
                m.output.deletion_queue_continuation.len(),
                m.output.floor_raises_pushed_by_the_continuation.len()
            ),
            Err(e) => format!("err[{:.70?}]", e),
        }
    };
    let cycle = if unmount_ok {
        format!(
            "cycle-after-unmount-remount={}",
            describe(mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters,
                &mut devs,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                switches(k),
            ))
        )
    } else {
        "cycle=-".to_string()
    };
    // 候选乙新：卸载预演被拒时只把新 F 写进每块盘的系统配置（不发空发布），再挂
    let config_only = {
        let mut devs3 = plain_devices_on(&image);
        let version = session.current.version_of_trees();
        match version {
            None => "cfgF=-".to_string(),
            Some(version) => {
                let floor = version.root.checkpoint_txg;
                let counter = version.record.counter;
                let instance = version.record.instance;
                let wrote = {
                    let mut writer = PoolWriter::new(&pool.parameters, devs3.as_mut_slice());
                    singlefs_core::transaction::write_the_raised_floor_into_every_system_configuration(
                        &mut writer, floor, counter, instance,
                    )
                    .is_ok()
                };
                format!(
                    "cfgF[wrote={wrote}]-remount={}",
                    describe(mount_writable_with_test_only_switches_and_deletion_switches(
                        &pool.parameters,
                        &mut devs3,
                        SpaceAdmission::JudgedByTheFormula,
                        ShadowLedger::On,
                        switches(k),
                    ))
                )
            }
        }
    };
    let cfg_cycles = {
        let mut devs4 = plain_devices_on(&image);
        let mut trail = Vec::new();
        let mut round = 0;
        let mut current_version = session.current.clone();
        loop {
            round += 1;
            let Some(version) = current_version.version_of_trees() else { break };
            let (floor, counter, instance) = (version.root.checkpoint_txg, version.record.counter, version.record.instance);
            {
                let mut writer = PoolWriter::new(&pool.parameters, devs4.as_mut_slice());
                let _ = singlefs_core::transaction::write_the_raised_floor_into_every_system_configuration(&mut writer, floor, counter, instance);
            }
            let after_write = violations_of(&devs4, pool.device_bytes);
            match mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters, &mut devs4, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches(k)) {
                Ok(m) => {
                    let empty = matches!(m.output.deletion_queue_continuation_outcome, DeletionQueueContinuationOutcome::WalkedToEmpty);
                    trail.push(format!("{}:{}[w={},m={}]", m.output.deletion_queue_continuation.len(), if empty {"E"} else {"S"}, after_write, violations_of(&devs4, pool.device_bytes)));
                    current_version = m.current.clone();
                    if empty || round >= 8 { break; }
                }
                Err(e) => { trail.push(format!("err[{:.50?}]", e)); break; }
            }
        }
        format!("cfgF-cycles={}", trail.join(","))
    };
    let um_cycles = {
        let mut devs5 = plain_devices_on(&image);
        let mut alloc5 = session.allocator.clone();
        let mut cur5 = session.current.clone();
        let mut trail = Vec::new();
        for round in 1..=8 {
            match unmount(&pool.parameters, &mut devs5, &mut alloc5, &mut cur5, session.shadow_ledger) {
                Ok(_) => { trail.push(format!("u[{}]", violations_of(&devs5, pool.device_bytes))); }
                Err(e) => { trail.push(format!("U-err[{:.40}]", shortage(&e))); break; }
            }
            match mount_writable_with_test_only_switches_and_deletion_switches(
                &pool.parameters, &mut devs5, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches(k)) {
                Ok(m) => {
                    let empty = matches!(m.output.deletion_queue_continuation_outcome, DeletionQueueContinuationOutcome::WalkedToEmpty);
                    trail.push(format!("{}:{}", m.output.deletion_queue_continuation.len(), if empty {"E"} else {"S"}));
                    alloc5 = m.allocator.clone();
                    cur5 = m.current.clone();
                    if empty { break; }
                }
                Err(e) => { trail.push(format!("M-err[{:.50?}]", e)); break; }
            }
            let _ = round;
        }
        format!("um-cycles={}", trail.join(","))
    };
    // 再挂载：在 mount 之后的盘面（未卸载）拷贝上试
    let mut devs2 = plain_devices_on(&image);
    let remount = match mount_writable_with_test_only_switches_and_deletion_switches(
        &pool.parameters,
        &mut devs2,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(k),
    ) {
        Ok(m) => format!(
            "remount-ok({})",
            match &m.output.deletion_queue_continuation_outcome {
                DeletionQueueContinuationOutcome::WalkedToEmpty => "Empty",
                DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => "Short",
            }
        ),
        Err(e) => format!("remount-err[{:.90?}]", e),
    };
    let _ = PoolVersion::WithoutFile;
    format!(
        "CELL {tag} rw={rw} before={before_unlink} at-crash={at_crash} => steps={steps} raises={raises} queue={queue_len} outcome={outcome} after-mount={after_mount} {unmount_result} {remount} {cycle} {config_only} {cfg_cycles} {um_cycles}"
    )
}

#[test]
fn probe_grid() {
    let which = std::env::var("PROBE_GRID").unwrap_or_default();
    let mut cells: Vec<(Disk, usize, u64, usize, u64)> = Vec::new();
    match which.as_str() {
        // 复现实现员探针的几格
        "narrow" => {
            for n in [192_u64, 256, 320, 384] {
                for o in [0_usize, 2, 4] {
                    for u in [10_u64, 14, 20] {
                        for b in [0_usize, 1, 2] {
                            cells.push((Disk::Narrow(n), o, u, b, 1));
                        }
                    }
                }
            }
        }
        // 往大盘推：不补足的窄盘加宽单元区
        "narrow-wide" => {
            for n in [384_u64, 512, 640, 768, 896, 1024, 1200, 1600, 2400] {
                for o in [0_usize, 4] {
                    for u in [14_u64, 40] {
                        for b in [0_usize, 2] {
                            cells.push((Disk::Narrow(n), o, u, b, 1));
                        }
                    }
                }
            }
        }
        // 补足过的小盘（准入全程判着）
        "pad" => {
            for w in [
                HistoryDeviceWidth::UnitAreaOf240Slots,
                HistoryDeviceWidth::UnitAreaOf256Slots,
                HistoryDeviceWidth::UnitAreaOf384Slots,
            ] {
                for o in [0_usize, 4, 12] {
                    for u in [10_u64, 40, 100] {
                        for b in [0_usize, 1, 3] {
                            cells.push((Disk::Pad(w), o, u, b, 1));
                        }
                    }
                }
            }
        }
        "pad-prod" => {
            for w in [
                HistoryDeviceWidth::UnitAreaOf256Slots,
                HistoryDeviceWidth::UnitAreaOf384Slots,
            ] {
                for o in [0_usize, 4] {
                    for u in [10_u64, 100, 150, 200, 300] {
                        for b in [0_usize, 1] {
                            cells.push((Disk::Pad(w), o, u, b, 144));
                        }
                    }
                }
            }
        }
        "custom" => {
            let list = |name: &str| -> Vec<u64> {
                std::env::var(name)
                    .unwrap_or_default()
                    .split(',')
                    .filter(|x| !x.is_empty())
                    .map(|x| x.parse().unwrap())
                    .collect()
            };
            let disks: Vec<Disk> = std::env::var("PROBE_DISKS")
                .unwrap_or_default()
                .split(',')
                .filter(|x| !x.is_empty())
                .map(|x| match x {
                    "pad384" => Disk::Pad(HistoryDeviceWidth::UnitAreaOf384Slots),
                    "pad256" => Disk::Pad(HistoryDeviceWidth::UnitAreaOf256Slots),
                    "pad240" => Disk::Pad(HistoryDeviceWidth::UnitAreaOf240Slots),
                    other if other.starts_with('f') => {
                        let (n, l) = other[1..].split_once('-').unwrap();
                        Disk::First(n.parse().unwrap(), l.parse().unwrap())
                    }
                    other => Disk::Narrow(other.strip_prefix("n").unwrap().parse().unwrap()),
                })
                .collect();
            for d in &disks {
                for o in list("PROBE_O") {
                    for u in list("PROBE_U") {
                        for b in list("PROBE_B") {
                            for k in list("PROBE_K") {
                                cells.push((*d, usize::try_from(o).unwrap(), u, usize::try_from(b).unwrap(), k));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    for (disk, o, u, b, k) in cells {
        println!("{}", run_cell(disk, o, u, b, k));
    }
}

#[test]
fn first_file_threshold() {
    use singlefs_core::allocator::UnitAreaStart;
    use singlefs_format::SLOT_BYTES;
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let parameters = width.parameters();
    let start = UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
        .unwrap()
        .slot()
        .0;
    let mut n = std::env::var("PROBE_FROM").ok().and_then(|x| x.parse().ok()).unwrap_or(600_u64);
    let to: u64 = std::env::var("PROBE_TO").ok().and_then(|x| x.parse().ok()).unwrap_or(1300);
    let step: u64 = std::env::var("PROBE_STEP").ok().and_then(|x| x.parse().ok()).unwrap_or(25);
    while n <= to {
        let device_bytes = (start + n) * SLOT_BYTES;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
                width.parameters(),
                device_bytes,
                common_admission::plain_device,
            );
            let ok_remount = pool.crash_and_mount_writable().is_ok();
            let overwrite = pool.overwrite(OVERWRITE_BYTES).is_ok();
            (ok_remount, overwrite)
        }));
        match result {
            Ok((remount, overwrite)) => println!("FIRSTFILE N={n} first-file-ok remount-ok={remount} overwrite-ok={overwrite}"),
            Err(_) => println!("FIRSTFILE N={n} first-file-refused-or-setup-panic"),
        }
        n += step;
    }
}

fn build_first_big(
    unit_area_slots: u64,
    units: u64,
) -> Result<PoolUnderTest<SparseBlockDevice>, String> {
    use singlefs_core::allocator::UnitAreaStart;
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
    use singlefs_core::mount::ParametersAndDeviceTableOfTheMount;
    use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile};
    use singlefs_format::SLOT_BYTES;
    use std::collections::BTreeMap;
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let start = UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
        .unwrap()
        .slot()
        .0;
    let device_bytes = (start + unit_area_slots) * SLOT_BYTES;
    let mut devices: Vec<(singlefs_core::address::DeviceIdentity, SparseBlockDevice)> =
        [singlefs_core::address::DeviceIdentity(0), singlefs_core::address::DeviceIdentity(1)]
            .into_iter()
            .map(|identity| (identity, SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512))))
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).map_err(|e| format!("mkfs {e:?}"))?;
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let length = if units <= 1 { 3000 } else { (usize::try_from(units).unwrap() - 1) * data_unit_payload_capacity() + 100 };
    let first_content = common_admission::content_of(length, 0);
    let (instance, first_version) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).map_err(|e| format!("取号 {e:?}"))?;
        let warmed = warm_up(&mut writer, &genesis.root, instance).map_err(|e| format!("暖机 {e:?}"))?;
        let first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().unwrap(),
            FirstFile { content: &first_content, write_time_seconds: common_admission::FIRST_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .map_err(|e| format!("{:.80}", short_of(&e)))?;
        (instance, first_version)
    };
    let mut content_of_each_root = BTreeMap::new();
    content_of_each_root.insert(common_admission::root_key(&first_version.root), first_content.clone());
    let parameters_and_device_table = ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices)
        .map_err(|e| format!("{e:?}"))?;
    Ok(PoolUnderTest {
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
        write_time_seconds: common_admission::FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: first_content,
        content_of_each_root,
    })
}
