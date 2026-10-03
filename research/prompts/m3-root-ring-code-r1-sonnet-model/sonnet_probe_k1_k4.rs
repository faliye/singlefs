//! 正推腿探针（草稿，不入库）：K1 卸载记号其余位在读者与池级 checker 两处的口径；K4 写行之后第一个文件版本、记录已持久根槽没持久那一形。
mod common;

use common::{
    build_pool, format_pool, parameters, tear_the_root_slot_written_for_txg, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::PoolReader;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

fn not_holding(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| !matches!(verdict, InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect()
}

fn set_marker_field_of_record(image: &mut MemoryPool, counter: u64, field: u32, corrupt_tree_table: bool) {
    let fsid = singlefs_core::unit::unit_filesystem_identifier(&parameters().filesystem_identifier);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
        let bytes = PoolReader::read(image, device, offset, 4096).expect("读得到");
        let mut record = JournalRecord::parse(&bytes, fsid).expect("改之前自证过");
        if corrupt_tree_table {
            for location in &mut record.new_tree_table.locations {
                location.unit_checksum ^= 1;
            }
        }
        let mut bytes = record.to_bytes();
        bytes[459..463].copy_from_slice(&field.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
        bytes[46..78].copy_from_slice(&digest);
        image.devices.get_mut(&device).unwrap().write(offset, &bytes);
    }
}

fn report(tag: &str, image: &MemoryPool) {
    match mount_read_only(image) {
        Ok(read_only) => println!(
            "PROBE {tag}: core effective_root txg={} instance={} above_water={} prefix_applied={}; checker verdicts={:?}",
            read_only.effective_root.checkpoint_txg.0,
            read_only.effective_root.instance.0,
            read_only.journal.above_water,
            read_only.journal.prefix_applied,
            summary(image)
        ),
        Err(error) => println!("PROBE {tag}: core mount_read_only ERR {error:?}; checker verdicts={:?}", summary(image)),
    }
}

fn summary(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(name, verdict)| !matches!(verdict, InvariantVerdict::Holds) && !matches!((name, verdict), (_, InvariantVerdict::NotApplicable(_)) if false))
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, verdict)| format!("{name}: {verdict:?}"))
        .collect()
}

#[test]
fn probe_unmount_marker_bits() {
    for (label, field, corrupt) in [
        ("marker=1,tt-corrupt", 1u32, true),
        ("marker=2,tt-corrupt", 2u32, true),
        ("marker=0x80000000,tt-corrupt", 0x8000_0000u32, true),
        ("marker=2,intact", 2u32, false),
    ] {
        let pool = build_pool(&format!("sonnet-probe-marker-{field}-{corrupt}"));
        let mut image = pool.memory_pool();
        tear_the_root_slot_written_for_txg(&mut image, 3);
        set_marker_field_of_record(&mut image, 3, field, corrupt);
        report(label, &image);
    }
    // baseline: untouched, root 3 torn
    let pool = build_pool("sonnet-probe-marker-baseline");
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 3);
    report("baseline-root3-torn", &image);
    report("baseline-untouched", &pool.memory_pool());
}

#[test]
fn probe_first_file_after_a_row_record_only() {
    let mut formatted = format_pool("sonnet-probe-first-file-after-row");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let mut mounted = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    let (root_without_file, record_bytes_without_file) = match &mounted.current {
        PoolVersion::WithoutFile(v) => (v.root, v.record_bytes.clone()),
        _ => panic!("无文件版"),
    };
    let publish_parameters = parameters();
    let first_file = {
        let devices = formatted.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root_without_file,
            FirstFile { content: &vec![0x5A; 3000], write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            mounted.output.instance,
            &record_bytes_without_file,
        )
        .expect("第一个文件版本")
    };
    let whole = formatted.memory_pool();
    report("first-file-after-row WHOLE", &whole);
    let mut image = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, first_file.root.checkpoint_txg.0);
    report("first-file-after-row ROOT-TORN", &image);
}

#[test]
fn probe_marker_two_intact_details() {
    use singlefs_core::recovery::{choose_root, choose_system_configuration, replay_journal, scan_journal};
    let pool = build_pool("sonnet-probe-marker-details");
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 3);
    set_marker_field_of_record(&mut image, 3, 2, false);
    let system_configuration = choose_system_configuration(&image).expect("sysconf");
    let chosen = choose_root(&image, &system_configuration).expect("root");
    println!("PROBE details chosen root txg={} instance={}", chosen.checkpoint_txg.0, chosen.instance.0);
    let records = scan_journal(&image, &system_configuration);
    println!("PROBE details scanned records keys={:?}", records.keys().map(|k| (k.0 .0, k.1)).collect::<Vec<_>>());
    let result = replay_journal(&image, &chosen, system_configuration.immutable.sizes.journal_ring_bytes, &records, true);
    match result {
        Ok((journal, rebuilt)) => println!("PROBE details replay ok prefix_applied={} above_water={} rebuilt txg={}", journal.prefix_applied, journal.above_water, rebuilt.checkpoint_txg.0),
        Err(e) => println!("PROBE details replay ERR {e:?}"),
    }
    // control: pristine pool, root 2 chosen by tearing root 3 AND removing record 3 entirely
    let pool2 = build_pool("sonnet-probe-record-gone");
    let mut image2 = pool2.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image2, 3);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image2.devices.get_mut(&device).unwrap().write(record_offset(3, JOURNAL_RING_DEFAULT_BYTES), &vec![0u8; 4096]);
    }
    report("control: root3 torn and record3 zeroed", &image2);
    let _ = DeviceOffsetInBytes(0);
}

#[test]
fn probe_root_two_pointers() {
    use singlefs_core::recovery::{choose_root, choose_system_configuration};
    let pool = build_pool("sonnet-probe-root2");
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 3);
    let sc = choose_system_configuration(&image).unwrap();
    let chosen = choose_root(&image, &sc).unwrap();
    println!("PROBE root2 tree_table slots={:?}", chosen.tree_table.locations.map(|l| l.slot.0));
    println!("PROBE root2 instance_table slots={:?}", chosen.instance_table.locations.map(|l| l.slot.0));
    println!("PROBE root2 mapping slots={:?}", chosen.mapping_root.locations.map(|l| l.slot.0));
    println!("PROBE root2 alloc slots={:?}", chosen.allocation_record_tree_root.locations.map(|l| l.slot.0));
    println!("PROBE root3(output) tree_table slots={:?} mapping={:?}", pool.output.root.tree_table.locations.map(|l| l.slot.0), pool.output.root.mapping_root.locations.map(|l| l.slot.0));
}

#[test]
fn probe_marker_bits_core_set_vs_checker_prefix() {
    use singlefs_core::recovery::{choose_root, choose_system_configuration, replay_journal, scan_journal};
    for (label, field, corrupt) in [("marker=1,tt-corrupt", 1u32, true), ("marker=2,tt-corrupt", 2u32, true), ("marker=0x80000000,tt-corrupt", 0x8000_0000u32, true)] {
        let pool = build_pool(&format!("sonnet-probe-core-vs-checker-{field}"));
        let mut image = pool.memory_pool();
        tear_the_root_slot_written_for_txg(&mut image, 3);
        set_marker_field_of_record(&mut image, 3, field, corrupt);
        let sc = choose_system_configuration(&image).unwrap();
        let chosen = choose_root(&image, &sc).unwrap();
        let records = scan_journal(&image, &sc);
        let (journal, _) = replay_journal(&image, &chosen, sc.immutable.sizes.journal_ring_bytes, &records, true).unwrap();
        println!("PROBE2 {label}: core scan keys={:?} prefix_applied={} ; checker={:?}", records.keys().map(|k| (k.0 .0, k.1)).collect::<Vec<_>>(), journal.prefix_applied, summary(&image));
    }
}

#[test]
fn probe_dump_records_and_roots_for_hand_decoding() {
    use singlefs_core::root_ring::{slot_offset, target_for_publish};
    use singlefs_core::address::CheckpointTxg;
    let pool = build_pool("sonnet-probe-dump");
    let image = pool.memory_pool();
    let dir = std::path::Path::new("/tmp/claude-1000/m3-root-ring-code-r1-sonnet/dump");
    std::fs::create_dir_all(dir).unwrap();
    for counter in 1..=3u64 {
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let bytes = PoolReader::read(&image, device, record_offset(counter, JOURNAL_RING_DEFAULT_BYTES), 4096).unwrap();
            std::fs::write(dir.join(format!("record-{counter}-dev{}.bin", device.0)), bytes).unwrap();
        }
    }
    for txg in 1..=3u64 {
        let target = target_for_publish(CheckpointTxg(txg), parameters().geometry.root_ring_slots_per_region);
        let device = parameters().region_devices[usize::try_from(target.region).unwrap()];
        let bytes = PoolReader::read(&image, device, slot_offset(target, parameters().geometry.fixed_structure_slot_spacing), usize::try_from(parameters().geometry.physical_block_size).unwrap()).unwrap();
        std::fs::write(dir.join(format!("root-txg{txg}.bin")), bytes).unwrap();
    }
    println!("PROBE dump done; first-file output record counter={} txg={}", pool.output.record.counter, pool.output.root.checkpoint_txg.0);
}

fn all_verdicts_of(image: &MemoryPool, names: &[&str]) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(name, _)| names.contains(name))
        .map(|(name, verdict)| format!("{name}={verdict:?}"))
        .collect()
}

#[test]
fn probe_first_file_after_row_wrong_alloc_root_in_record() {
    use common::rewrite_journal_record_and_rechain_the_records_after_it;
    let mut formatted = format_pool("sonnet-probe-ffar-wrong");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let mut mounted = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    let (root_without_file, record_bytes_without_file) = match &mounted.current {
        PoolVersion::WithoutFile(v) => (v.root, v.record_bytes.clone()),
        _ => panic!("无文件版"),
    };
    let publish_parameters = parameters();
    let first_file = {
        let devices = formatted.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer, &mut mounted.allocator, &root_without_file,
            FirstFile { content: &vec![0x5A; 3000], write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            mounted.output.instance, &record_bytes_without_file,
        ).expect("第一个文件版本")
    };
    let counter = first_file.record.counter;
    let names = ["I-3.1", "I-4.1", "I-3.10"];
    let mut right = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut right, first_file.root.checkpoint_txg.0);
    println!("PROBE3 right record (alloc root empty): {:?}", all_verdicts_of(&right, &names));
    let mut wrong = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut wrong, first_file.root.checkpoint_txg.0);
    let previous_alloc_root = root_without_file.allocation_record_tree_root;
    rewrite_journal_record_and_rechain_the_records_after_it(&mut wrong, counter, |record| {
        record.new_allocation_record_tree_root = previous_alloc_root;
    });
    println!("PROBE3 WRONG record (alloc root = previous row version's non-empty root): {:?}", all_verdicts_of(&wrong, &names));
    let mut wrong_instance = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut wrong_instance, first_file.root.checkpoint_txg.0);
    let previous_instance_table = root_without_file.instance_table;
    let mut changed = first_file.root.instance_table; changed = if changed == previous_instance_table { changed } else { previous_instance_table };
    rewrite_journal_record_and_rechain_the_records_after_it(&mut wrong_instance, counter, |record| {
        record.new_instance_table = changed;
    });
    println!("PROBE3 record instance_table = previous version's ({}): {:?}", first_file.root.instance_table == previous_instance_table, all_verdicts_of(&wrong_instance, &names));
}

/// 补洞用例（草稿）：卸载那一串两次空发布（txg 7、8）的根槽都没落、两条记录落了：所选根是 txg 6（不带卸载记号），施加 txg 7、8 两条记录；
/// 取记录里的 ⇒ 施加出来的根与 txg 8 当初写的逐字段相同（带卸载记号）；照抄所选根 ⇒ 卸载记号对不上。
#[test]
fn probe_unmount_marker_distinguishes_record_from_chosen_root() {
    use common::{overwrite_in_process_as_instance_one_and_advance_the_pool, CONTENT_BYTES_OF_PUBLISH_B};
    use singlefs_core::address::CheckpointTxg;
    use singlefs_core::mount::{unmount, ShadowLedger, Unmounted};
    use singlefs_core::root_record::UnmountMarker;
    let mut pool = build_pool("sonnet-probe-unmount-both-publishes");
    for fill_byte in [0xB0u8, 0xC0, 0xD0] {
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B]);
    }
    let mut current = PoolVersion::WithFile(pool.output.clone());
    let devices = pool.devices.as_mut().unwrap();
    let unmounted = unmount(&parameters(), devices, &mut pool.allocator, &mut current, ShadowLedger::On).expect("卸载");
    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else { panic!("应是抬 F 那一支") };
    let [_first, second_unmount_publish] = <[_; 2]>::try_from(raised.publishes).expect("两次空发布");
    assert_eq!(second_unmount_publish.root.checkpoint_txg, CheckpointTxg(8));
    assert_eq!(second_unmount_publish.root.unmount_marker, UnmountMarker::WrittenByTheUnmountSequence);
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 7);
    tear_the_root_slot_written_for_txg(&mut image, 8);
    let ro = mount_read_only(&image).expect("只读挂载");
    println!("PROBE4 chosen txg={} marker={:?}; effective txg={} marker={:?}; prefix_applied={}", ro.chosen_root.checkpoint_txg.0, ro.chosen_root.unmount_marker, ro.effective_root.checkpoint_txg.0, ro.effective_root.unmount_marker, ro.journal.prefix_applied);
    assert_eq!(ro.chosen_root.unmount_marker, UnmountMarker::NotWrittenByTheUnmountSequence);
    assert_eq!(ro.effective_root, second_unmount_publish.root, "施加出来的根与 txg 8 当初写的逐字段相同");
}

fn mount_writable_on_a_copy_then_image(image: &MemoryPool) -> Result<MemoryPool, String> {
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_harness::memory_pool::SparseBlockDevice;
    let params = parameters();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(params.geometry.physical_block_size));
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect();
    match mount_writable(&params, &mut devices) {
        Ok(_mounted) => Ok(MemoryPool {
            devices: devices.into_iter().map(|(identity, device)| (identity, device.image)).collect(),
            device_size_in_bytes: image.device_size_in_bytes,
        }),
        Err(error) => Err(format!("{error:?}")),
    }
}

#[test]
fn probe_next_mount_after_first_file_after_row_record_only() {
    use common::rewrite_journal_record_and_rechain_the_records_after_it;
    let mut formatted = format_pool("sonnet-probe-ffar-next-mount");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let mut mounted = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    let (root_without_file, record_bytes_without_file) = match &mounted.current {
        PoolVersion::WithoutFile(v) => (v.root, v.record_bytes.clone()),
        _ => panic!("无文件版"),
    };
    let publish_parameters = parameters();
    let first_file = {
        let devices = formatted.devices.as_mut().unwrap();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer, &mut mounted.allocator, &root_without_file,
            FirstFile { content: &vec![0x5A; 3000], write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            mounted.output.instance, &record_bytes_without_file,
        ).expect("第一个文件版本")
    };
    let counter = first_file.record.counter;
    let names = ["I-3.1", "I-4.1", "I-3.10", "I-7.4", "I-3.9", "I-8.10", "I-9.16"];
    let mut right = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut right, first_file.root.checkpoint_txg.0);
    let mut wrong = formatted.memory_pool();
    tear_the_root_slot_written_for_txg(&mut wrong, first_file.root.checkpoint_txg.0);
    let previous_alloc_root = root_without_file.allocation_record_tree_root;
    rewrite_journal_record_and_rechain_the_records_after_it(&mut wrong, counter, |record| {
        record.new_allocation_record_tree_root = previous_alloc_root;
    });
    for (label, image) in [("RIGHT record", &right), ("WRONG alloc root in record", &wrong)] {
        match mount_writable_on_a_copy_then_image(image) {
            Ok(after) => println!("PROBE5 {label}: writable mount ok; checker after the next mount: {:?}; all-violations={:?}", all_verdicts_of(&after, &names), summary(&after)),
            Err(error) => println!("PROBE5 {label}: writable mount refused: {error}"),
        }
    }
}
