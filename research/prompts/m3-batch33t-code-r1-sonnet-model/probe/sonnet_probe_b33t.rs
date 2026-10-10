//! 正推腿探针（draft）：P1 = C692 下限与新池新建文件实有落点数逐几何比；P2/P3 = F1 与 F2 的水位、C 与候选豁免的交叉。
mod common_unwitnessed_publish;

use std::collections::BTreeMap;

use common_unwitnessed_publish::*;
use singlefs_core::address::{DeviceIdentity, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, UnitAreaStart};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    placements_on_each_device_after_the_new_pool_file_creation, recover, JournalPolicy,
};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::e142_parameters;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn counts(records: &[AllocationRecord]) -> BTreeMap<u32, usize> {
    let mut m = BTreeMap::new();
    for r in records {
        *m.entry(r.device.0).or_insert(0) += 1;
    }
    m
}

fn probe_geometry(
    label: &str,
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    device_bytes: u64,
    overwrites: u32,
) {
    let ring = parameters.geometry.journal_ring_bytes;
    let ua = match UnitAreaStart::following_the_journal_ring(ring) {
        Ok(ua) => ua,
        Err(e) => {
            println!("PROBE1 {label} UNIT_AREA_OFF_BOUNDARY {e:?}");
            return;
        }
    };
    let sizes = [(DeviceIdentity(0), device_bytes), (DeviceIdentity(1), device_bytes)];
    let predicted =
        std::panic::catch_unwind(|| placements_on_each_device_after_the_new_pool_file_creation(&sizes, ua));
    let predicted = match predicted {
        Ok(p) => p.0.to_string(),
        Err(_) => "PANIC".to_string(),
    };
    let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut pool = PoolUnderTest::<SparseBlockDevice>::start_after_the_first_file_on_devices_of(
            parameters.clone(),
            device_bytes,
            |_, d| d,
        );
        let at_creation = counts(&pool.current_file_version().allocation_records);
        let mut min_after = usize::MAX;
        let mut done = 0;
        for _ in 0..overwrites {
            if pool.overwrite(OVERWRITE_BYTES).is_err() {
                break;
            }
            done += 1;
            let c = counts(&pool.current_file_version().allocation_records);
            min_after = min_after.min(*c.values().min().unwrap());
        }
        (at_creation, min_after, done)
    }));
    match built {
        Ok((at_creation, min_after, done)) => println!(
            "PROBE1 {label} ua_slot={} predicted={predicted} actual_at_creation={at_creation:?} min_per_device_over_{done}_overwrites={}",
            ua.slot().0,
            if min_after == usize::MAX { "n/a".to_string() } else { min_after.to_string() }
        ),
        Err(_) => println!("PROBE1 {label} predicted={predicted} POOL_BUILD_PANIC"),
    }
}

#[test]
fn p1_new_pool_file_creation_lower_bound_vs_actual_per_geometry() {
    probe_geometry(
        "small384",
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        40,
    );
    for unit_area_slots in [240u64, 256, 320, 512, 1024, 4096] {
        let parameters = parameters_of_the_384_slot_small_devices();
        let ua = UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes)
            .unwrap()
            .slot()
            .0;
        let bytes = (ua + 815 + unit_area_slots) * SLOT_BYTES;
        probe_geometry(&format!("small_unit_area_{unit_area_slots}"), parameters, bytes, 30);
    }
    for (label, ring_bytes) in [("ring2MiB", 2u64 << 20), ("ring3MiB", 3 << 20), ("ring64MiB", 64 << 20), ("ring1GiB", 1 << 30)] {
        let mut parameters = e142_parameters(512, 512);
        parameters.geometry.journal_ring_bytes = ring_bytes;
        probe_geometry(&format!("{label}_dev4GiB"), parameters, 4u64 << 30, 12);
    }
    for (label, dev) in [("default_dev1GiB", 1u64 << 30), ("default_dev4GiB", 4 << 30), ("default_dev16GiB", 16 << 30), ("default_dev64GiB", 64 << 30)] {
        probe_geometry(label, e142_parameters(512, 512), dev, 12);
    }
    probe_geometry("io_min8192_dev4GiB", e142_parameters(4096, 8192), 4u64 << 30, 12);
}

fn describe_mount(label: &str, recorded: &RecordedPublish, ranges: &RangesReadingBackAllZero) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
    match mount_writable(&recorded.pool.parameters, &mut devices) {
        Err(e) => println!("PROBE23 {label} MOUNT_REFUSED {e:?}"),
        Ok(mounted) => {
            let eff = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
            let mut free_units = Vec::new();
            for w in recorded.unit_writes() {
                let dm = mounted.allocator.devices.iter().find(|d| d.device == w.device).unwrap();
                let slot = SlotNumber(w.offset.0 / SLOT_BYTES);
                free_units.push((w.device.0, slot.0, w.length_in_bytes() / SLOT_BYTES, dm.is_free(slot)));
            }
            let (mount_writes, _s, _i) = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
            let row_root = mount_writes.iter().position(|w| w.kind == StepKind::RootRecordFua);
            let mut image: MemoryPool = crash_image.clone();
            if let Some(rr) = row_root {
                image.apply_writes(&mount_writes[..rr]);
            }
            let overwritten = units_of_r_overwritten_on(recorded, &image);
            let report = recover(&image, JournalPolicy::Consult);
            let violations = checker_violations_on(&image);
            println!("PROBE23V {label} pool_checker_violations_on_the_image_before_the_row_root={violations:?}");
            println!(
                "PROBE23 {label} MOUNTED effective_root={:?} r_key={:?} (dev,slot,slots,is_free_after_mount)={:?} units_of_R_overwritten_before_row_root={:?} recover_after_that: effective_root={:?} outcome_is_file_read={}",
                eff,
                recorded.key,
                free_units,
                overwritten,
                report.effective_root,
                matches!(report.outcome, singlefs_core::recovery::RecoveryOutcome::FileRead { .. })
            );
        }
    }
}

#[test]
fn p23_f1_water_mark_and_c_candidate_exemption_under_the_f2_fallback() {
    // 三条记录的发布 R，根槽落盘、轮换没落。
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
    let base = RangesReadingBackAllZero::default();
    // A：根槽读回全零 + 全部单元 + 中间记录：F1 该拒（现有用例形）
    describe_mount(
        "A_root_zero_units_zero_mid_record_zero",
        &recorded,
        &base.clone().with_the_writes(&[recorded.root_slot_write()]).with_the_writes(&recorded.unit_writes()).with_the_writes(&recorded.record_writes_of_the_record(2)),
    );
    // B：根槽读得出 + 全部单元 + 中间记录读回全零
    describe_mount(
        "B_root_READABLE_units_zero_mid_record_zero",
        &recorded,
        &base.clone().with_the_writes(&recorded.unit_writes()).with_the_writes(&recorded.record_writes_of_the_record(2)),
    );
    // C：根槽读得出 + 全部单元读回全零，记录都读得出
    describe_mount(
        "C_root_READABLE_units_zero_records_readable",
        &recorded,
        &base.clone().with_the_writes(&recorded.unit_writes()),
    );
    // D：根槽读得出 + 只有数据单元（两槽）读回全零，账读得出
    let data_units: Vec<_> = recorded.unit_writes().into_iter().filter(|w| w.length_in_bytes() == 2 * SLOT_BYTES).collect();
    println!("PROBE23 data_unit_writes={}", data_units.len());
    describe_mount(
        "D_root_READABLE_only_data_units_zero_account_readable",
        &recorded,
        &base.clone().with_the_writes(&data_units),
    );
    // E：同 D 但记录中间一条也读回全零
    describe_mount(
        "E_root_READABLE_only_data_units_zero_mid_record_zero",
        &recorded,
        &base.clone().with_the_writes(&data_units).with_the_writes(&recorded.record_writes_of_the_record(2)),
    );
}

#[test]
fn p1b_min_per_device_count_under_a_long_mixed_history_on_small_devices() {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let ua = UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes).unwrap();
    let sizes = [(DeviceIdentity(0), device_bytes), (DeviceIdentity(1), device_bytes)];
    let predicted = placements_on_each_device_after_the_new_pool_file_creation(&sizes, ua).0;
    let mut pool = PoolUnderTest::<SparseBlockDevice>::start_after_the_first_file_on_devices_of(parameters, device_bytes, |_, d| d);
    let mut min_count = usize::MAX;
    let mut ops = 0u32;
    let mut refused = 0u32;
    for i in 0..400u32 {
        let r = if i % 3 == 0 { pool.sequential_write(100 + (i as usize % 7) * 20_000) } else { pool.overwrite(OVERWRITE_BYTES) };
        match r { Ok(_) => ops += 1, Err(_) => { refused += 1; } }
        let c = counts(&pool.current_file_version().allocation_records);
        min_count = min_count.min(*c.values().min().unwrap());
    }
    println!("PROBE1B predicted_bound={predicted} ops_ok={ops} refused={refused} min_per_device_count_seen={min_count}");
    assert!(min_count >= predicted);
}

#[test]
fn p4_read_recovery_fields_on_an_image_where_recover_falls_back() {
    use singlefs_harness::mount_transparency::{read_recovery_fields, ComparedVersion};
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::Overwrite);
    // S1：R 已确认；较新的系统配置槽与 R 的单元读回全零
    let confirmed = recorded.image_with_the_whole_publish_persisted();
    let rotation_of_r: Vec<_> = recorded.writes.iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).collect();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes()).with_the_writes(&rotation_of_r);
    let devices = devices_reading_back_all_zero_on(&confirmed, &ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    println!("PROBE4 S1 recover effective_root={:?} outcome_is_file_read={}", report.effective_root, matches!(report.outcome, singlefs_core::recovery::RecoveryOutcome::FileRead { .. }));
    for compared in [ComparedVersion::TheRecoveredVersion, ComparedVersion::TheVersionBeforeTheRecoveredOne] {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read_recovery_fields(&devices, compared)));
        match r {
            Ok(f) => println!("PROBE4 S1 {compared:?} read_recovery_fields OK effective_root={:?}", f.report.effective_root),
            Err(p) => {
                let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                println!("PROBE4 S1 {compared:?} read_recovery_fields PANICKED: {msg}");
            }
        }
    }
}

#[test]
fn p5_three_sites_on_the_same_image_with_only_the_data_units_unreadable() {
    use singlefs_core::mounted_read::mount_read_only;
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let data_units: Vec<_> = recorded.unit_writes().into_iter().filter(|w| w.length_in_bytes() == 2 * SLOT_BYTES).collect();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&data_units);
    let devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let report = recover(&devices, JournalPolicy::Consult);
    println!("PROBE5 recover: effective_root={:?} outcome={}", report.effective_root, match &report.outcome { singlefs_core::recovery::RecoveryOutcome::FileRead{root,..} => format!("FileRead root={root:?}"), singlefs_core::recovery::RecoveryOutcome::NoFile{root} => format!("NoFile root={root:?}"), singlefs_core::recovery::RecoveryOutcome::Failed{root,..} => format!("Failed root={root:?}") });
    match mount_read_only(&devices) {
        Ok(m) => println!("PROBE5 read_only: Ok chosen={:?} effective={:?}", (m.chosen_root.instance, m.chosen_root.checkpoint_txg), (m.effective_root.instance, m.effective_root.checkpoint_txg)),
        Err(e) => println!("PROBE5 read_only: Err {e:?}"),
    }
    let mut devices_w = devices_reading_back_all_zero_on(&crash_image, &ranges);
    match mount_writable(&recorded.pool.parameters, &mut devices_w) {
        Ok(m) => println!("PROBE5 writable: Ok effective={:?}", (m.output.effective_root.instance, m.output.effective_root.checkpoint_txg)),
        Err(e) => println!("PROBE5 writable: Err {e:?}"),
    }
}

#[test]
fn p6_time_of_the_rehearsal_per_call_debug_build() {
    let sizes = [(DeviceIdentity(0), 4u64 << 30), (DeviceIdentity(1), 4u64 << 30)];
    let ua = UnitAreaStart::of_the_default_journal_ring();
    let started = std::time::Instant::now();
    let mut last = 0;
    for _ in 0..200 {
        last = placements_on_each_device_after_the_new_pool_file_creation(&sizes, ua).0;
    }
    println!("PROBE6 200 calls took {:?} (value {last}) in a debug build", started.elapsed());
}

#[test]
fn p1c_min_per_device_count_through_unlink_and_new_inodes_on_small_devices() {
    use singlefs_core::mounted_session::UserChange;
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let ua = UnitAreaStart::following_the_journal_ring(parameters.geometry.journal_ring_bytes).unwrap();
    let sizes = [(DeviceIdentity(0), device_bytes), (DeviceIdentity(1), device_bytes)];
    let predicted = placements_on_each_device_after_the_new_pool_file_creation(&sizes, ua).0;
    let mut pool = PoolUnderTest::<SparseBlockDevice>::start_after_the_first_file_on_devices_of(parameters, device_bytes, |_, d| d);
    for _ in 0..3 { pool.overwrite(OVERWRITE_BYTES).unwrap(); }
    let before = counts(pool.session().allocator.records());
    let wt = pool.write_time_seconds + 1;
    let r = pool.publish(UserChange::Unlink { inode: singlefs_core::address::InodeNumber(singlefs_core::transaction::FIRST_INODE_NUMBER), write_time_seconds: wt }, None);
    println!("PROBE1C unlink_ok={} allocator_records_per_device before={before:?} after_unlink={:?} predicted_bound={predicted}", r.is_ok(), counts(pool.session().allocator.records()));
    let r2 = pool.create_one_inode();
    println!("PROBE1C create_inode_ok={} after={:?}", r2.is_ok(), counts(pool.session().allocator.records()));
}

#[test]
fn p7_the_rehearsal_on_devices_shorter_than_the_unit_area_start() {
    let ua = UnitAreaStart::of_the_default_journal_ring();
    for bytes in [1u64 << 20, 600u64 << 20, 800u64 << 20] {
        let sizes = [(DeviceIdentity(0), bytes), (DeviceIdentity(1), bytes)];
        let r = std::panic::catch_unwind(|| placements_on_each_device_after_the_new_pool_file_creation(&sizes, ua).0);
        match r {
            Ok(v) => println!("PROBE7 device_bytes={bytes} unit_area_start_bytes={} value={v}", ua.slot().0 * SLOT_BYTES),
            Err(p) => {
                let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                println!("PROBE7 device_bytes={bytes} unit_area_start_bytes={} PANICKED: {}", ua.slot().0 * SLOT_BYTES, msg.chars().take(160).collect::<String>());
            }
        }
    }
}
