//! m3-close-i718-oldslot-r2 攻方腿原型：池级 checker 的 I-7.18 装了候选开关（`tools/walk-candidate-switch*.py`）之后，
//! 在环洞池（S 从 `I718_SLOTS_PER_REGION` 取）上造几段历史，手挑的镜像逐个候选打判定（`cells`），
//! 再把下一次发布那一段写表交崩溃点重放的枚举（`sample_crash_states_on_the_cpu`，每一段都展开，序号从 0 走到底）逐状态判（`enumerate`）。
//! 只放在三方腿的仓副本里编、直接执行编出来的二进制；不进入库的用例。
mod ring_hole_pools;

use ring_hole_pools::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools, pool_of,
    recording_of, slots_per_region_of_the_ring_hole_pools, RingHoleDevices,
    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, set_i718_candidate_for_the_attack_probe, I718Candidate};
use singlefs_checker::image::ImageReader;
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
    writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use std::collections::BTreeMap;

const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 428;
const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: usize = 436;
const ROOT_SLOT_BYTES: usize = 512;

/// 今天的判法（也是丙）、甲、第一轮攻方量过的甲-任一圈与丁甲、第二轮丁甲的三种读法。
const CANDIDATES: [(I718Candidate, &str); 7] = [
    (I718Candidate::CurrentOrBing, "today"),
    (I718Candidate::Jia, "jia"),
    (I718Candidate::JiaAnyLapOfTheNextSlot, "jia_any_lap"),
    (I718Candidate::DingJiaAnyLapWithTheNextRecord, "ding_r1"),
    (I718Candidate::DingRoundTwoSameInstance, "ding_r2_same_instance"),
    (I718Candidate::DingRoundTwoAnyInstance, "ding_r2_any_instance"),
    (I718Candidate::DingRoundTwoAnyInstanceRawValues, "ding_r2_raw_values"),
];

fn ring_length() -> u64 {
    3 * slots_per_region_of_the_ring_hole_pools()
}

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

fn empty_devices() -> RingHoleDevices {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect()
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件（txg 3）。
fn fresh_writer() -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = empty_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } =
        mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let current = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile { content: &content_of_the_version(0), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS },
            instance,
            current.record_bytes(),
        )
        .expect("第一个文件")
    };
    Writer { devices, allocator, current, instance }
}

/// 一次录下来的写：之前的盘面、写表、按屏障与 FUA 切的段。
struct Recorded {
    before: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
}

/// 在一张盘面上可写挂载（新实例：写行、暖机），录下这次挂载的全部写；交回录下的写与写者（这一版没有文件时没有写者）。
fn recorded_mount(image: &MemoryPool) -> (Recorded, Option<Writer>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    let Mounted { output, allocator, current } =
        mount_writable(&parameters, &mut recording).expect("可写挂载");
    let (writes, segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let mut after = image.clone();
    after.apply_writes(&writes);
    let writer = match current {
        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => Some(Writer {
            devices: devices_of(&after),
            allocator,
            current: output_of(current),
            instance: output.instance,
        }),
        PoolVersion::WithoutFile(_) => None,
    };
    (Recorded { before: image.clone(), writes, segments }, writer)
}

/// 下一次覆盖写，录下它的写：交回写之前的盘面与这一次的写（写者前进到写完）。
fn recorded_overwrite(writer: &mut Writer) -> Recorded {
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
            FirstFile { content: &content_of_the_version(salt), write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt },
            writer.instance,
        )
        .expect("覆盖写")
    };
    let (writes, segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let mut after = before.clone();
    after.apply_writes(&writes);
    writer.devices = devices_of(&after);
    writer.current = output;
    Recorded { before, writes, segments }
}

fn overwrite_through(writer: &mut Writer, last: u64) {
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while writer.current.root.checkpoint_txg.0 < last {
        recorded_overwrite(writer);
    }
}

/// txg 那一槽的盘与偏移。
fn root_slot_of(txg: u64) -> (DeviceIdentity, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    (device, slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0)
}

fn slot_bytes(image: &dyn ImageReader, txg: u64) -> Option<Vec<u8>> {
    let (device, offset) = root_slot_of(txg);
    image.read(device.0, offset, ROOT_SLOT_BYTES)
}

fn root_in_slot_of(image: &dyn ImageReader, txg: u64) -> Option<RootRecord> {
    let parameters = parameters_of_the_ring_hole_pools();
    RootRecord::parse_slot(&slot_bytes(image, txg)?, &parameters.filesystem_identifier)
}

/// 环里读得出、自证过的根的 (txg, 实例)，按 txg 排。
fn ring_of(image: &dyn ImageReader) -> Vec<(u64, u32)> {
    let mut ring: Vec<(u64, u32)> = (0..ring_length())
        .filter_map(|slot_value| root_in_slot_of(image, slot_value).map(|root| (root.checkpoint_txg.0, root.instance.0)))
        .collect();
    ring.sort_unstable();
    ring
}

/// 一段写里第 `ordinal` 次根槽写的位置。
fn root_write_index(writes: &[RetainedWrite], ordinal: usize) -> usize {
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .nth(ordinal)
        .expect("这一段的根槽写")
        .0
}

/// 第 `ordinal` 次根槽写：崩在它之前（之前的写全落）、它写成 0、它照崩溃点重放的撕法撕（新旧不同的那一截前一半新、后一半旧，
/// `crates/singlefs-checker-tier/src/crash.rs` 的 `torn_image_of_in_place_overwrite` 的文档，这里另写）。
fn crash_images_of_the_root_slot_write(
    before: &MemoryPool,
    writes: &[RetainedWrite],
    ordinal: usize,
) -> Vec<(&'static str, MemoryPool)> {
    let index = root_write_index(writes, ordinal);
    let root_write = &writes[index];
    let mut before_the_root = before.clone();
    before_the_root.apply_writes(&writes[..index]);
    let new_bytes = root_write.bytes().expect("根槽写带字节").to_vec();
    let old_bytes = singlefs_core::recovery::PoolReader::read(&before_the_root, root_write.device, root_write.offset, new_bytes.len())
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
    let torn = match (0..new_bytes.len()).find(|index| new_bytes[*index] != old_bytes[*index]) {
        None => new_bytes.clone(),
        Some(first) => {
            let last = (0..new_bytes.len()).rev().find(|index| new_bytes[*index] != old_bytes[*index]).expect("有第一个就有最后一个");
            let first_old = first + (last - first + 1) / 2;
            new_bytes[..first_old].iter().chain(old_bytes[first_old..].iter()).copied().collect()
        }
    };
    vec![
        ("crash_before_the_root_slot_write", before_the_root.clone()),
        ("root_slot_written_zero", with_slot(vec![0u8; new_bytes.len()])),
        ("root_slot_torn_like_the_enumeration", with_slot(torn)),
    ]
}

/// 改一条根的 t 与 p（重封自证校验和）；只改根槽，不改那次发布的 journal 记录。
fn rewrite_threshold_of_root(image: &mut MemoryPool, txg: u64, threshold: u64, pinned: u8) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = slot_bytes(image, txg).expect("根槽读得出");
    bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8].copy_from_slice(&threshold.to_le_bytes());
    bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET] = pinned;
    let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_RECORD_CHECKSUM_OFFSET);
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    image.devices.get_mut(&device).expect("那块盘").write(singlefs_core::address::DeviceOffsetInBytes(offset), &bytes);
}

fn short_verdict(verdict: Option<&InvariantVerdict>) -> &'static str {
    match verdict {
        Some(InvariantVerdict::Holds) => "holds",
        Some(InvariantVerdict::Violated(_)) => "violated",
        Some(InvariantVerdict::NotApplicable(_)) => "not_applicable",
        Some(_) => "other",
        None => "absent",
    }
}

/// 一张镜像逐个候选的 I-7.18 判定，连同环、最新根、下一槽、journal 里有没有 N+1 的记录（同实例 / 任一实例）与今天判红的全部不变量。
fn verdict_line(image: &dyn ImageReader) -> String {
    let ring = ring_of(image);
    let Some(&newest) = ring.iter().max() else { return "ring=empty".to_string() };
    let root = root_in_slot_of(image, newest.0).expect("最新根");
    let next_slot = root_in_slot_of(image, newest.0 + 1).map(|root| (root.checkpoint_txg.0, root.instance.0));
    let mut parts = Vec::new();
    let mut today_red = Vec::new();
    let (mut record_same, mut record_any) = (false, false);
    let chosen = std::env::var("I718_CANDIDATE_NAMES").ok();
    for (candidate, name) in CANDIDATES {
        if candidate != I718Candidate::CurrentOrBing
            && chosen.as_deref().is_some_and(|names| !names.split(',').any(|chosen_name| chosen_name == name))
        {
            continue;
        }
        set_i718_candidate_for_the_attack_probe(candidate);
        let verdicts = check_pool_image(image);
        let i718 = verdicts.iter().find(|(invariant, _)| *invariant == "I-7.18").map(|(_, verdict)| verdict);
        parts.push(format!("{name}={}", short_verdict(i718)));
        if candidate == I718Candidate::CurrentOrBing {
            today_red = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(i, _)| *i).collect();
            record_same = singlefs_checker::walk::I718_NEXT_RECORD_PRESENT.with(std::cell::Cell::get);
            record_any = singlefs_checker::walk::I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(std::cell::Cell::get);
        }
    }
    set_i718_candidate_for_the_attack_probe(I718Candidate::CurrentOrBing);
    format!(
        "newest={newest:?} t={} p={} F={} next_slot={next_slot:?} record_same_instance={record_same} record_any_instance={record_any} {} today_red={today_red:?}",
        root.reclaim_threshold.0, root.pinned_ring_slot.0, root.rollback_floor.0, parts.join(" ")
    )
}

fn report(geometry_and_cell: &str, image_name: &str, image: &dyn ImageReader) {
    println!("cell={geometry_and_cell} image={image_name} {}", verdict_line(image));
}

fn txg_of(writer: &Writer) -> u64 {
    writer.current.root.checkpoint_txg.0
}

/// 环洞：实例 1 写到 `base_last`，下一个实例的可写挂载崩在写行那次根槽写之前（那一槽留着上一圈的根），再下一个实例可写挂载。
fn writer_after_a_hole(base_last: u64) -> (Writer, u64) {
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, base_last);
    let (crashed, row_txg) = crash_a_mount_before_its_row_root(&pool_of(&writer.devices));
    let (_, third) = recorded_mount(&crashed);
    (third.expect("有文件的一版"), row_txg)
}

/// 手挑的镜像：每格先合法写者，再几份坏写者（只改最新根的 t、p 字节）。
fn cells() {
    let length = ring_length();
    let geometry = format!("S{}", slots_per_region_of_the_ring_hole_pools());
    // A：满环、无洞，覆盖写 N+1 的根槽落在最旧根那一槽上写坏。N 取两圈之后，坏写者 t 早一圈（t − 3S，仍 ≡ N+1 (mod 3S)）。
    let newest = 2 * length + 1;
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, newest);
    let stable = pool_of(&writer.devices);
    let recorded = recorded_overwrite(&mut writer);
    for (name, image) in crash_images_of_the_root_slot_write(&recorded.before, &recorded.writes, 0) {
        report(&format!("{geometry}-A"), name, &image);
        let root = root_in_slot_of(&image, newest).expect("最新根");
        let mut older = image.clone();
        rewrite_threshold_of_root(&mut older, newest, root.reclaim_threshold.0 - length, root.pinned_ring_slot.0);
        report(&format!("{geometry}-A-bad-t-one-lap-older"), name, &older);
    }
    // R（W3）：同一个满环，没有在飞的发布，最旧根那一槽（正是下一槽）事后读坏（写成 0）：读故障，不是写失败，journal 里没有 N+1 的记录。
    let mut read_fault = stable.clone();
    let (device, offset) = root_slot_of(newest + 1);
    read_fault.devices.get_mut(&device).expect("那块盘").write(singlefs_core::address::DeviceOffsetInBytes(offset), &vec![0u8; ROOT_SLOT_BYTES]);
    report(&format!("{geometry}-R-full-ring-oldest-read-fault"), "stable", &read_fault);
    // R 环没满：新池写到 txg 5，最旧根那一槽事后读坏。
    let mut young = fresh_writer();
    overwrite_through(&mut young, 5);
    let young_image = pool_of(&young.devices);
    let oldest = ring_of(&young_image).first().copied().expect("环里有根");
    let mut young_fault = young_image.clone();
    let (device, offset) = root_slot_of(oldest.0);
    young_fault.devices.get_mut(&device).expect("那块盘").write(singlefs_core::address::DeviceOffsetInBytes(offset), &vec![0u8; ROOT_SLOT_BYTES]);
    report(&format!("{geometry}-R-ring-not-full-oldest-read-fault-oldest{}", oldest.0), "stable", &young_fault);
    // G：实例 1 写到 N（满环、无洞）之后进程停，下一个实例可写挂载：写行那次发布 txg N+1 的根槽落在最旧根那一槽，写坏；
    // G2：写行那次落了，第一次暖机空发布的根槽写坏。N 放开扫一圈（由用户决定的那一步）。
    for newest in 2 * length..3 * length {
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, newest);
        let (mount, _) = recorded_mount(&pool_of(&writer.devices));
        let root_writes = mount.writes.iter().filter(|write| write.kind == StepKind::RootRecordFua).count();
        for ordinal in 0..root_writes.min(2) {
            let cell = if ordinal == 0 { "G-row-publish" } else { "G2-first-warm-up-publish" };
            for (name, image) in crash_images_of_the_root_slot_write(&mount.before, &mount.writes, ordinal) {
                report(&format!("{geometry}-{cell}-N{newest}"), name, &image);
            }
        }
    }
    // B：环洞落在下一槽（第一轮格 B 的形），换几何再量。
    let (mut hole_writer, row_txg) = writer_after_a_hole(length + 2);
    overwrite_through(&mut hole_writer, row_txg + length - 1);
    let recorded = recorded_overwrite(&mut hole_writer);
    for (name, image) in crash_images_of_the_root_slot_write(&recorded.before, &recorded.writes, 0) {
        report(&format!("{geometry}-B-hole-in-the-next-slot"), name, &image);
    }
    // I：环洞在别的槽（还在环里、是最旧的有效根），下一槽写坏。坏写者：按一圈窗口滤根（D16 已废的写法，t = N+1−3S）、t = N+2−3S。
    let (mut hole_writer, row_txg) = writer_after_a_hole(length + 2);
    let newest = row_txg + 4;
    overwrite_through(&mut hole_writer, newest);
    let recorded = recorded_overwrite(&mut hole_writer);
    for (name, image) in crash_images_of_the_root_slot_write(&recorded.before, &recorded.writes, 0) {
        report(&format!("{geometry}-I-hole-elsewhere"), name, &image);
        let root = root_in_slot_of(&image, newest).expect("最新根");
        for (bad, threshold) in [("window-filter-t-N+1-3S", newest + 1 - length), ("t-N+2-3S", newest + 2 - length)] {
            let mut wrong = image.clone();
            rewrite_threshold_of_root(&mut wrong, newest, threshold, root.pinned_ring_slot.0);
            report(&format!("{geometry}-I-hole-elsewhere-bad-{bad}"), name, &wrong);
        }
    }
    // K（W4 零单元发布、无文件版本）：mkfs 之后只反复可写挂载、不建文件，转过一圈之后再挂一次，写行那次的根槽写坏。
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = empty_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut image = pool_of(&devices);
    // 迭代上界：每次挂载至少发一次，转过两圈就停。
    while ring_of(&image).last().map_or(0, |root| root.0) < 2 * length {
        let (mount, _) = recorded_mount(&image);
        let mut after = mount.before.clone();
        after.apply_writes(&mount.writes);
        image = after;
    }
    let (mount, _) = recorded_mount(&image);
    for (name, image) in crash_images_of_the_root_slot_write(&mount.before, &mount.writes, 0) {
        report(&format!("{geometry}-K-no-file-pool-row-publish"), name, &image);
    }
}

/// 写表里每次根槽写各一版（内容由调用方给），连同写之前那一版。
fn versions_of(base_newest: (u64, u32), base_content: Vec<u8>, writes: &[RetainedWrite], content_of_the_writes: &[u8]) -> Vec<PublishedVersion> {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(base_newest.1),
        checkpoint_txg: CheckpointTxg(base_newest.0),
        content: base_content,
    }];
    for write in writes.iter().filter(|write| write.kind == StepKind::RootRecordFua) {
        let root = RootRecord::parse_slot(write.bytes().expect("根槽写带字节"), &parameters.filesystem_identifier).expect("根槽写自证");
        versions.push(PublishedVersion { instance: root.instance, checkpoint_txg: root.checkpoint_txg, content: content_of_the_writes.to_vec() });
    }
    versions
}

/// 一条流交崩溃点重放的枚举（每一段都展开、序号从 0 走到底），逐状态打 I-7.18 各候选的判定，按（最新根、下一槽读不读得出、
/// 有没有 N+1 的记录、各候选判定）计数。先算全量状态数，超过 10⁶ 不跑。
fn enumerate_stream(name: &str, base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], judged_root_ordinal: usize, versions: &[PublishedVersion]) -> u64 {
    use singlefs_checker_tier::crash::{crash_state_count_with_torn_in_place_overwrites, full_expansion};
    use singlefs_checker_tier::crash_state_enumeration::{sample_crash_states_on_the_cpu, CrashPointReplayItem};
    let states = crash_state_count_with_torn_in_place_overwrites(base, writes, segments, &full_expansion);
    println!("stream={name} writes={} segments={} states={states}", writes.len(), segments.len());
    if states > 1_000_000 {
        println!("stream={name} skipped: states over 10^6");
        return 0;
    }
    let judged_root_index = root_write_index(writes, judged_root_ordinal);
    // 几个进程分序号段跑（`I718_ORDINAL_SHARE=i/n`）：段与段首尾相接、合起来正好 0..states，每个状态只判一次。
    let (share, shares) = std::env::var("I718_ORDINAL_SHARE")
        .ok()
        .and_then(|text| text.split_once('/').and_then(|(i, n)| Some((i.parse::<u64>().ok()?, n.parse::<u64>().ok()?))))
        .unwrap_or((0, 1));
    let ordinals = share * states / shares..(share + 1) * states / shares;
    println!("stream={name} share={share}/{shares} ordinals={ordinals:?}");
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    let started = std::time::Instant::now();
    let outcome = sample_crash_states_on_the_cpu(
        CrashPointReplayItem { name, setting_file: "research/prompts/m3-close-i718-oldslot-r2-opus-model/probe/src/main.rs" },
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &full_expansion,
        ordinals,
        &mut |_ordinal, image, _judgement| {
            let line = verdict_line(image);
            // 计数键去掉 t、p、F 之外随状态变的部分不需要：环与下一槽都已在行里，整行就是键。
            *tally.entry(line).or_insert(0) += 1;
        },
    );
    let judged = outcome.judged();
    println!("stream={name} judged_states={judged:?} seconds={:.1}", started.elapsed().as_secs_f64());
    for (line, count) in &tally {
        println!("stream={name} count={count} {line}");
    }
    states
}

fn enumerate() {
    let length = ring_length();
    let geometry = format!("S{}", slots_per_region_of_the_ring_hole_pools());
    let which = std::env::var("I718_STREAMS").unwrap_or_else(|_| "A,A-bad,G,I-bad".to_string());
    let mut total = 0u64;
    let newest = 2 * length + 1;
    if which.contains("A") {
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, newest);
        let instance = writer.instance.0;
        let recorded = recorded_overwrite(&mut writer);
        let versions = versions_of((newest, instance), content_of_the_version(newest), &recorded.writes, &content_of_the_version(newest + 1));
        if which.split(',').any(|part| part == "A") {
            total += enumerate_stream(&format!("i718-r2-{geometry}-A-legit"), &recorded.before, &recorded.writes, &recorded.segments, 0, &versions);
        }
        if which.split(',').any(|part| part == "A-bad") {
            let mut bad_base = recorded.before.clone();
            let root = root_in_slot_of(&bad_base, newest).expect("最新根");
            rewrite_threshold_of_root(&mut bad_base, newest, root.reclaim_threshold.0 - length, root.pinned_ring_slot.0);
            total += enumerate_stream(&format!("i718-r2-{geometry}-A-bad-t-one-lap-older"), &bad_base, &recorded.writes, &recorded.segments, 0, &versions);
        }
    }
    if which.split(',').any(|part| part == "G") {
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, newest);
        let instance = writer.instance.0;
        let (mount, _) = recorded_mount(&pool_of(&writer.devices));
        let versions = versions_of((newest, instance), content_of_the_version(newest), &mount.writes, &content_of_the_version(newest));
        total += enumerate_stream(&format!("i718-r2-{geometry}-G-mount-after-a-full-ring"), &mount.before, &mount.writes, &mount.segments, 0, &versions);
    }
    if which.split(',').any(|part| part == "G-row") {
        // 缩历史长度：同一次挂载只留到写行那次发布为止（第一次暖机空发布根槽写所在那一段之前的段），每个留下的段照样整段展开。
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, newest);
        let instance = writer.instance.0;
        let (mount, _) = recorded_mount(&pool_of(&writer.devices));
        let second_root = root_write_index(&mount.writes, 1);
        let segments: Vec<Vec<usize>> = mount.segments.iter().take_while(|segment| !segment.contains(&second_root)).cloned().collect();
        let kept_writes = segments.iter().flatten().copied().max().expect("有写") + 1;
        let writes = mount.writes[..kept_writes].to_vec();
        let versions = versions_of((newest, instance), content_of_the_version(newest), &writes, &content_of_the_version(newest));
        total += enumerate_stream(&format!("i718-r2-{geometry}-G-row-publish-only"), &mount.before, &writes, &segments, 0, &versions);
    }
    if which.split(',').any(|part| part == "I-bad") {
        let (mut hole_writer, row_txg) = writer_after_a_hole(length + 2);
        let newest = row_txg + 4;
        overwrite_through(&mut hole_writer, newest);
        let instance = hole_writer.instance.0;
        let recorded = recorded_overwrite(&mut hole_writer);
        let versions = versions_of((newest, instance), content_of_the_version(newest), &recorded.writes, &content_of_the_version(newest + 1));
        let mut bad_base = recorded.before.clone();
        let root = root_in_slot_of(&bad_base, newest).expect("最新根");
        rewrite_threshold_of_root(&mut bad_base, newest, newest + 1 - length, root.pinned_ring_slot.0);
        total += enumerate_stream(&format!("i718-r2-{geometry}-I-bad-window-filter"), &bad_base, &recorded.writes, &recorded.segments, 0, &versions);
    }
    println!("total_states={total}");
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("cells") => cells(),
        Some("enumerate") => enumerate(),
        _ => panic!("用法：i718-round-two-probe cells|enumerate（S 从 I718_SLOTS_PER_REGION 取）"),
    }
}
