//! m3-close-i718-oldslot-r1 攻方腿原型：在仓副本里给池级 checker 的 I-7.18 装候选开关（`tools/walk-candidate-switch.py`），
//! 在环洞池（S = 4、环长 12、区域归盘 0 / 1 / 0）上造几段历史，逐张镜像、逐个候选打印 I-7.18 的判定。
//! 只放在三方腿的副本里跑，不进入库的用例。
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools, pool_of,
    recording_of, RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image, set_i718_candidate_for_the_attack_probe, I718Candidate,
    I718_PROBE_COUNTERS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 428;
const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: usize = 436;
const ROOT_SLOT_BYTES: usize = 512;

const CANDIDATES: [(I718Candidate, &str); 8] = [
    (I718Candidate::CurrentOrBing, "today_or_bing"),
    (I718Candidate::Jia, "jia"),
    (I718Candidate::YiLiteral, "yi_literal"),
    (I718Candidate::YiAfterTheFirstLap, "yi_after_first_lap"),
    (I718Candidate::JiaAnyLapOfTheNextSlot, "jia_any_lap_of_next_slot"),
    (I718Candidate::JiaAnyUnreadableSlot, "jia_any_unreadable_slot"),
    (I718Candidate::DingJiaAnyLapWithTheNextRecord, "ding_jia_with_next_record"),
    (I718Candidate::DingYiWithTheNextRecord, "ding_yi_with_next_record"),
];

/// 一个写者进程：盘、分配器、现行那一版、实例代号。
struct Writer {
    devices: RingHoleDevices,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
}

fn output_of(version: PoolVersion) -> TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => panic!("这一版没有文件"),
    }
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件（txg 3）。
fn fresh_writer() -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices: RingHoleDevices = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(
                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
                    PhysicalBlockSizeInBytes(512),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted {
        output,
        mut allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let current = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content_of_the_version(0),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
            },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    Writer {
        devices,
        allocator,
        current,
        instance,
    }
}

/// 在一张盘面上可写挂载（新实例，写行、暖机），交回写者。
fn writer_mounted_on(image: &MemoryPool) -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(image);
    let Mounted {
        output,
        allocator,
        current,
    } = mount_writable(&parameters, &mut devices).expect("可写挂载");
    Writer {
        devices,
        allocator,
        current: output_of(current),
        instance: output.instance,
    }
}

/// 下一次覆盖写，录下它的写：交回写之前的盘面与这一次的写（写者前进到写完）。
fn recorded_overwrite(writer: &mut Writer) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let before = pool_of(&writer.devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let salt = writer.current.root.checkpoint_txg.0 + 1;
    let output = {
        let mut pool_writer = PoolWriter::new(&parameters, recording.as_mut_slice());
        publish_overwrite(
            &mut pool_writer,
            &mut writer.allocator,
            &writer.current,
            FirstFile {
                content: &content_of_the_version(salt),
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
            },
            writer.instance,
        )
        .expect("覆盖写")
    };
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let mut after = before.clone();
    after.apply_writes(&writes);
    writer.devices = devices_of(&after);
    writer.current = output;
    (before, writes)
}

/// txg 那一槽的盘与偏移。
fn root_slot_of(txg: u64) -> (DeviceIdentity, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(
        CheckpointTxg(txg),
        parameters.geometry.root_ring_slots_per_region,
    );
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    (
        device,
        slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0,
    )
}

fn slot_bytes(image: &MemoryPool, txg: u64) -> Option<Vec<u8>> {
    let (device, offset) = root_slot_of(txg);
    image.devices.get(&device)?;
    singlefs_core::recovery::PoolReader::read(
        image,
        device,
        singlefs_core::address::DeviceOffsetInBytes(offset),
        ROOT_SLOT_BYTES,
    )
}

/// 环里读得出、自证过的根的 (txg, 实例)，按 txg 排。
fn ring_of(image: &MemoryPool, newest_guess: u64) -> Vec<(u64, u32)> {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut ring: Vec<(u64, u32)> = (0..12u64)
        .filter_map(|slot_value| {
            let bytes = slot_bytes(image, slot_value + 12 * (newest_guess / 12))?;
            RootRecord::parse_slot(&bytes, &parameters.filesystem_identifier)
                .map(|root| (root.checkpoint_txg.0, root.instance.0))
        })
        .collect();
    ring.sort_unstable();
    ring
}

fn root_in_slot_of(image: &MemoryPool, txg: u64) -> Option<RootRecord> {
    let parameters = parameters_of_the_ring_hole_pools();
    RootRecord::parse_slot(&slot_bytes(image, txg)?, &parameters.filesystem_identifier)
}

/// 一次发布的写里根槽写的位置。
fn root_write_index(writes: &[RetainedWrite]) -> usize {
    writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("这次发布的根槽写")
}

/// 崩在根槽写之前（根槽写之前的写全落）、根槽写写成 0、根槽写撕在前半之后：三张恢复之前的镜像。
fn crash_images_of_the_root_slot_write(
    before: &MemoryPool,
    writes: &[RetainedWrite],
) -> Vec<(&'static str, MemoryPool)> {
    let index = root_write_index(writes);
    let root_write = &writes[index];
    let mut before_the_root = before.clone();
    before_the_root.apply_writes(&writes[..index]);
    let new_bytes = root_write.bytes().expect("根槽写带字节").to_vec();
    let old_bytes = singlefs_core::recovery::PoolReader::read(
        before,
        root_write.device,
        root_write.offset,
        new_bytes.len(),
    )
    .expect("旧字节");
    let with_slot = |bytes: Vec<u8>| {
        let mut image = before_the_root.clone();
        image.apply_writes(&[RetainedWrite {
            device: root_write.device,
            kind: root_write.kind,
            is_force_unit_access: root_write.is_force_unit_access,
            offset: root_write.offset,
            contents: WrittenContents::Bytes(bytes),
        }]);
        image
    };
    let half = new_bytes.len() / 2;
    let torn: Vec<u8> = new_bytes[..half]
        .iter()
        .chain(old_bytes[half..].iter())
        .copied()
        .collect();
    // 崩溃点重放枚举的撕裂态取法（`crates/singlefs-checker-tier/src/crash.rs` 的 `torn_image_of_in_place_overwrite`，这里照它的文档另写）：
    // 新旧不同的那一截前一半新、后一半旧。
    let first_differing = (0..new_bytes.len()).find(|index| new_bytes[*index] != old_bytes[*index]);
    let torn_like_the_enumeration: Vec<u8> = match first_differing {
        None => new_bytes.clone(),
        Some(first) => {
            let last = (0..new_bytes.len())
                .rev()
                .find(|index| new_bytes[*index] != old_bytes[*index])
                .expect("有第一个不同的字节就有最后一个");
            let first_old = first + (last - first + 1) / 2;
            new_bytes[..first_old]
                .iter()
                .chain(old_bytes[first_old..].iter())
                .copied()
                .collect()
        }
    };
    vec![
        (
            "root_slot_torn_like_the_crash_point_replay_enumeration",
            with_slot(torn_like_the_enumeration),
        ),
        ("crash_before_the_root_slot_write", before_the_root.clone()),
        ("root_slot_written_zero", with_slot(vec![0u8; new_bytes.len()])),
        ("root_slot_torn_after_the_first_half", with_slot(torn)),
    ]
}

/// 改一条根的 t 与 p（重封自证校验和）；只改根槽，不改那次发布的 journal 记录。
fn rewrite_threshold_of_root(image: &mut MemoryPool, txg: u64, threshold: u64, pinned: u8) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = slot_bytes(image, txg).expect("根槽读得出");
    bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
        .copy_from_slice(&threshold.to_le_bytes());
    bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET] = pinned;
    let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_RECORD_CHECKSUM_OFFSET);
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    image
        .devices
        .get_mut(&device)
        .expect("那块盘")
        .write(singlefs_core::address::DeviceOffsetInBytes(offset), &bytes);
}

fn without_device(image: &MemoryPool, device: u32) -> MemoryPool {
    let mut degraded = image.clone();
    degraded.devices.remove(&DeviceIdentity(device));
    degraded
}

fn verdict_text(verdict: Option<&InvariantVerdict>) -> String {
    match verdict {
        Some(InvariantVerdict::Holds) => "holds".to_string(),
        Some(InvariantVerdict::Violated(detail)) => format!("violated[{detail}]"),
        Some(InvariantVerdict::NotApplicable(reason)) => format!("not_applicable[{reason}]"),
        Some(other) => format!("{other:?}"),
        None => "absent".to_string(),
    }
}

/// 一张镜像：先打环、最新根与它的 t、p、下一槽里的根，再逐个候选打 I-7.18 的判定与全部判红的不变量。
fn report(cell: &str, image_name: &str, image: &MemoryPool) {
    let ring = ring_of(image, 0);
    let newest = *ring
        .iter()
        .max_by_key(|(txg, instance)| (*txg, *instance))
        .expect("环里有根");
    let newest_root = root_in_slot_of(image, newest.0).expect("最新根");
    let next_slot = root_in_slot_of(image, newest.0 + 1)
        .map(|root| (root.checkpoint_txg.0, root.instance.0));
    println!(
        "cell={cell} image={image_name} devices={:?} ring={ring:?} newest={newest:?} t={} p={} F={} next_slot_root={next_slot:?}",
        image.devices.keys().map(|device| device.0).collect::<Vec<_>>(),
        newest_root.reclaim_threshold.0,
        newest_root.pinned_ring_slot.0,
        newest_root.rollback_floor.0,
    );
    for (candidate, name) in CANDIDATES {
        set_i718_candidate_for_the_attack_probe(candidate);
        I718_PROBE_COUNTERS.with(|counters| *counters.borrow_mut() = (0, 0));
        let verdicts = check_pool_image(image);
        let i718 = verdicts.iter().find(|(invariant, _)| *invariant == "I-7.18");
        let red: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(invariant, _)| *invariant)
            .collect();
        println!(
            "  cell={cell} image={image_name} candidate={name} I-7.18={} red={red:?}",
            verdict_text(i718.map(|(_, verdict)| verdict))
        );
    }
    set_i718_candidate_for_the_attack_probe(I718Candidate::CurrentOrBing);
}

fn overwrite_through(writer: &mut Writer, last: u64) {
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while writer.current.root.checkpoint_txg.0 < last {
        recorded_overwrite(writer);
    }
}

fn txg_of(writer: &Writer) -> u64 {
    writer.current.root.checkpoint_txg.0
}

/// 题面那一形：满环（3–14）之后覆盖写 txg 15，它的根槽就是环里最旧的 txg 3 住的那一槽。
fn cell_full_ring_next_publish() -> (MemoryPool, Vec<RetainedWrite>) {
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, 14);
    recorded_overwrite(&mut writer)
}

#[test]
fn cell_a_full_ring_root_slot_write_fails_on_the_oldest_root() {
    let (before, writes) = cell_full_ring_next_publish();
    for (name, image) in crash_images_of_the_root_slot_write(&before, &writes) {
        report("A", name, &image);
    }
    // 写者若漏算 txg 15 那一槽里的旧根（t 写成不带它的 4）：写坏那两张上谁都看不出，崩在根槽写之前那张上谁都红。
    for (name, mut image) in crash_images_of_the_root_slot_write(&before, &writes) {
        let pinned = root_in_slot_of(&image, 14).expect("txg 14").pinned_ring_slot.0;
        rewrite_threshold_of_root(&mut image, 14, 4, pinned);
        report("A-wrong-t-4", name, &image);
    }
}

/// 环洞落在下一槽上：实例 1 写到 `base_last`，实例 2 的可写挂载崩在写行那次根槽写之前（那一槽留着上一圈的根），实例 3 可写挂载之后覆盖写。
fn writer_after_a_hole(base_last: u64) -> (Writer, u64) {
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, base_last);
    let (crashed, row_txg) = crash_a_mount_before_its_row_root(&pool_of(&writer.devices));
    let third = writer_mounted_on(&crashed);
    println!(
        "hole: base_last={base_last} crashed_row_txg={row_txg} third_instance={} third_starts_at_txg={} ring={:?}",
        third.instance.0,
        txg_of(&third),
        ring_of(&pool_of(&third.devices), 0)
    );
    (third, row_txg)
}

#[test]
fn cell_b_hole_two_laps_old_in_the_next_slot_then_its_root_slot_write_fails() {
    let (mut writer, row_txg) = writer_after_a_hole(14);
    overwrite_through(&mut writer, row_txg + 11);
    let (before, writes) = recorded_overwrite(&mut writer);
    for (name, image) in crash_images_of_the_root_slot_write(&before, &writes) {
        report("B", name, &image);
    }
    // 写者把 t 写成 txg row_txg（下一槽「旧内容 = N+1−3S」那条根，其实那一槽装的是更早一圈的根）：比真值高、是不安全的方向。
    for (name, mut image) in crash_images_of_the_root_slot_write(&before, &writes) {
        let newest = row_txg + 11;
        let pinned = root_in_slot_of(&image, newest).expect("最新根").pinned_ring_slot.0;
        rewrite_threshold_of_root(&mut image, newest, row_txg, pinned);
        report("B-wrong-t-row", name, &image);
    }
}

#[test]
fn cell_c_whole_device_lost_on_a_ring_without_holes() {
    let mut writer = fresh_writer();
    for last in 13..=19u64 {
        overwrite_through(&mut writer, last);
        let image = pool_of(&writer.devices);
        for device in [0u32, 1] {
            report("C", &format!("newest{last}_without_device{device}"), &without_device(&image, device));
        }
    }
}

#[test]
fn cell_d_whole_device_lost_holding_the_hole_while_the_next_slot_survives() {
    let (mut writer, row_txg) = writer_after_a_hole(15);
    let start = txg_of(&writer);
    for last in start..=row_txg + 11 {
        overwrite_through(&mut writer, last);
        let image = pool_of(&writer.devices);
        report("D", &format!("newest{last}_whole"), &image);
        report("D", &format!("newest{last}_without_device1"), &without_device(&image, 1));
    }
}

#[test]
fn cell_e_first_lap_every_image_and_a_wrong_threshold() {
    let mut writer = fresh_writer();
    for last in 3..=11u64 {
        overwrite_through(&mut writer, last);
        let image = pool_of(&writer.devices);
        report("E", &format!("newest{last}"), &image);
        let mut wrong = image.clone();
        let root = root_in_slot_of(&wrong, last).expect("最新根");
        rewrite_threshold_of_root(&mut wrong, last, root.reclaim_threshold.0 + 1, root.pinned_ring_slot.0);
        report("E-wrong-t-plus-one", &format!("newest{last}"), &wrong);
    }
}

/// 题面那一形写成 0 之后的恢复：实例 2 可写挂载时 txg 15 那一槽读回全零、钉住；覆盖写到下一槽正是那一槽为止。
#[test]
fn cell_f_slot_pinned_at_the_mount_is_the_next_slot() {
    let (before, writes) = cell_full_ring_next_publish();
    let zeroed = crash_images_of_the_root_slot_write(&before, &writes)
        .into_iter()
        .find(|(name, _)| *name == "root_slot_written_zero")
        .expect("写成 0 那张")
        .1;
    let mut writer = writer_mounted_on(&zeroed);
    println!(
        "F: second_instance={} starts_at_txg={} ring={:?}",
        writer.instance.0,
        txg_of(&writer),
        ring_of(&pool_of(&writer.devices), 0)
    );
    for last in txg_of(&writer)..=26u64 {
        overwrite_through(&mut writer, last);
        report("F", &format!("newest{last}"), &pool_of(&writer.devices));
    }
    let image = pool_of(&writer.devices);
    let root = root_in_slot_of(&image, 26).expect("txg 26");
    let mut threshold_as_floor = image.clone();
    rewrite_threshold_of_root(&mut threshold_as_floor, 26, root.rollback_floor.0, root.pinned_ring_slot.0);
    report("F-bad-t-as-floor", "newest26", &threshold_as_floor);
    let mut pin_dropped = image.clone();
    rewrite_threshold_of_root(&mut pin_dropped, 26, root.reclaim_threshold.0, 255);
    report("F-bad-p-255", "newest26", &pin_dropped);
}

/// 一张镜像逐个候选的 I-7.18 判定，压成一行（扫描用）。
fn scan_line(cell: &str, base_last: u64, image_name: &str, image: &MemoryPool) {
    let ring = ring_of(image, 0);
    let newest = *ring
        .iter()
        .max_by_key(|(txg, instance)| (*txg, *instance))
        .expect("环里有根");
    let root = root_in_slot_of(image, newest.0).expect("最新根");
    let mut verdicts = Vec::new();
    for (candidate, name) in CANDIDATES {
        set_i718_candidate_for_the_attack_probe(candidate);
        let all = check_pool_image(image);
        let verdict = match all.iter().find(|(invariant, _)| *invariant == "I-7.18") {
            Some((_, InvariantVerdict::Holds)) => "holds",
            Some((_, InvariantVerdict::Violated(_))) => "violated",
            Some((_, InvariantVerdict::NotApplicable(_))) => "not_applicable",
            _ => "other",
        };
        verdicts.push(format!("{name}={verdict}"));
    }
    set_i718_candidate_for_the_attack_probe(I718Candidate::CurrentOrBing);
    println!(
        "scan cell={cell} base_last={base_last} image={image_name} newest={newest:?} t={} p={} {}",
        root.reclaim_threshold.0,
        root.pinned_ring_slot.0,
        verdicts.join(" ")
    );
}

fn zero_and_torn(before: &MemoryPool, writes: &[RetainedWrite]) -> Vec<(&'static str, MemoryPool)> {
    crash_images_of_the_root_slot_write(before, writes)
        .into_iter()
        .filter(|(name, _)| *name != "crash_before_the_root_slot_write")
        .collect()
}

/// 用户动作放开扫：实例 1 覆盖写到几（`SCAN_BASE_FROM`..=`SCAN_BASE_TO`，环长 12 的一圈 12 个值）之后，
/// 题面形（A）、环洞落在下一槽（B）、挂载时钉住的槽轮到下一槽（F，连同两份坏写者）各取写坏的镜像。
#[test]
fn scan_the_base_length_over_one_revolution() {
    let from: u64 = std::env::var("SCAN_BASE_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    let to: u64 = std::env::var("SCAN_BASE_TO").ok().and_then(|v| v.parse().ok()).unwrap_or(23);
    for base_last in from..=to {
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, base_last);
        let (before, writes) = recorded_overwrite(&mut writer);
        let images_of_a = zero_and_torn(&before, &writes);
        for (name, image) in &images_of_a {
            scan_line("A", base_last, name, image);
        }
        let (mut hole_writer, row_txg) = writer_after_a_hole(base_last);
        overwrite_through(&mut hole_writer, row_txg + 11);
        let (hole_before, hole_writes) = recorded_overwrite(&mut hole_writer);
        for (name, image) in zero_and_torn(&hole_before, &hole_writes) {
            scan_line("B", base_last, name, &image);
        }
        let zeroed = &images_of_a
            .iter()
            .find(|(name, _)| *name == "root_slot_written_zero")
            .expect("写成 0 那张")
            .1;
        let mut pinned_writer = writer_mounted_on(zeroed);
        let pinned_slot_txg = base_last + 1;
        let newest = pinned_slot_txg + 12 * ((txg_of(&pinned_writer) + 12 - pinned_slot_txg) / 12) - 1;
        let newest = if newest < txg_of(&pinned_writer) { newest + 12 } else { newest };
        overwrite_through(&mut pinned_writer, newest);
        let image = pool_of(&pinned_writer.devices);
        scan_line("F", base_last, "legit", &image);
        let root = root_in_slot_of(&image, newest).expect("最新根");
        let mut threshold_as_floor = image.clone();
        rewrite_threshold_of_root(&mut threshold_as_floor, newest, root.rollback_floor.0, root.pinned_ring_slot.0);
        scan_line("F", base_last, "bad_t_as_floor", &threshold_as_floor);
        let mut pin_dropped = image.clone();
        rewrite_threshold_of_root(&mut pin_dropped, newest, root.reclaim_threshold.0, 255);
        scan_line("F", base_last, "bad_p_255", &pin_dropped);
        // 下一次发布（它的根槽就是钉着的那一槽）在飞：记录已落、根槽写之前崩，或根槽写成 0。
        let (in_flight_before, in_flight_writes) = recorded_overwrite(&mut pinned_writer);
        for (name, in_flight) in crash_images_of_the_root_slot_write(&in_flight_before, &in_flight_writes) {
            if name == "root_slot_torn_after_the_first_half" {
                continue;
            }
            scan_line("F-next-in-flight", base_last, name, &in_flight);
            let mut in_flight_pin_dropped = in_flight.clone();
            rewrite_threshold_of_root(&mut in_flight_pin_dropped, newest, root.reclaim_threshold.0, 255);
            scan_line("F-next-in-flight-bad-p-255", base_last, name, &in_flight_pin_dropped);
        }
    }
}
