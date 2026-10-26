//! m3-close-i718-oldslot-r2 辩方腿原型（第一段：装置与造盘）。
//! 在仓副本里给池级 checker 装 `tools/walk-defense-switch.py`（I-7.18 真判不动，同时算出各候选的判定放进线程局部变量），
//! 在环洞池（每区槽数 S 由环境变量 `I718_S` 给，默认 4；环长 3S；区域归盘 0 / 1 / 0）上造历史，逐张镜像打各候选的判定。
//! 只放在三方腿的副本里跑，不进入库的用例。
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools, pool_of, publish_empty,
    recording_of, RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, I718_ALL};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, Mounted, RollbackTarget};
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const ROOT_CHECKSUM_OFFSET: usize = 138;
const FLOOR_OFFSET: usize = 130;
const THRESHOLD_OFFSET: usize = 428;
const PIN_OFFSET: usize = 436;
const ROOT_SLOT_BYTES: usize = 512;
const NAMES: [&str; 11] = [
    "today", "jia_gt", "jia_seed", "yi_literal", "yi_gate", "any_lap", "ding_jia", "ding_jia_same",
    "ding_jia_single", "ding_yi", "ding_yi_same",
];

fn slots() -> u64 {
    std::env::var("I718_S").ok().and_then(|v| v.parse().ok()).unwrap_or(4)
}

fn ring() -> u64 {
    3 * slots()
}

/// `I718_FEW` 设了：大 S 下少取几个点（扫描点、洞的起点、钉槽圈数），免得一格跑过 225 秒。
fn few() -> bool {
    std::env::var("I718_FEW").is_ok()
}

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

fn blank_devices() -> RingHoleDevices {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                SparseBlockDevice::new(DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect()
}

/// mkfs → 可写挂载（实例 1）→ 再 `extra_mounts` 次可写挂载（每次一串零单元发布）→ 第一个文件。
fn fresh_writer(extra_mounts: u64) -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = blank_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    for _ in 0..extra_mounts {
        mounted = mount_writable(&parameters, &mut devices).expect("再一次可写挂载（零单元发布）");
    }
    let Mounted { output, mut allocator, current } = mounted;
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
    Writer { devices, allocator, current, instance }
}

fn writer_mounted_on(image: &MemoryPool) -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(image);
    let Mounted { output, allocator, current } =
        mount_writable(&parameters, &mut devices).expect("可写挂载");
    Writer { devices, allocator, current: output_of(current), instance: output.instance }
}

fn txg_of(writer: &Writer) -> u64 {
    writer.current.root.checkpoint_txg.0
}

/// 下一次发布（`empty` 为真是空发布，否则覆盖写），录下它的写：交回发布之前的盘面与这一次的写（写者前进到写完）。
fn recorded_publish(writer: &mut Writer, empty: bool) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let before = pool_of(&writer.devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let salt = txg_of(writer) + 1;
    let output = {
        let mut pool_writer = PoolWriter::new(&parameters, recording.as_mut_slice());
        if empty {
            publish_empty(&mut pool_writer, &mut writer.allocator, &writer.current, writer.instance)
                .expect("空发布")
        } else {
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
        }
    };
    let (writes, _segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let mut after = before.clone();
    after.apply_writes(&writes);
    writer.devices = devices_of(&after);
    writer.current = output;
    (before, writes)
}

fn overwrite_through(writer: &mut Writer, last: u64) {
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while txg_of(writer) < last {
        recorded_publish(writer, false);
    }
}

// ───────────── 第二段：槽、环、改字节、各种崩溃镜像、逐张判 ─────────────

fn root_slot_of(txg: u64) -> (DeviceIdentity, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    (device, slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0)
}

fn slot_bytes_at(image: &MemoryPool, device: DeviceIdentity, offset: u64) -> Option<Vec<u8>> {
    image.devices.get(&device)?;
    PoolReader::read(image, device, DeviceOffsetInBytes(offset), ROOT_SLOT_BYTES)
}

fn root_in_slot_of(image: &MemoryPool, txg: u64) -> Option<RootRecord> {
    let parameters = parameters_of_the_ring_hole_pools();
    let (device, offset) = root_slot_of(txg);
    RootRecord::parse_slot(&slot_bytes_at(image, device, offset)?, &parameters.filesystem_identifier)
}

/// 环里全部槽读得出、自证过的根（txg, 实例），按 txg 排。
fn ring_of(image: &MemoryPool) -> Vec<(u64, u32)> {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut found = Vec::new();
    for region in 0..3u64 {
        for slot in 0..slots() {
            let device = parameters.region_devices[usize::try_from(region).expect("区域号")];
            let offset = slot_offset(RootRingSlot { region, slot }, parameters.geometry.fixed_structure_slot_spacing).0;
            if let Some(bytes) = slot_bytes_at(image, device, offset) {
                if let Some(root) = RootRecord::parse_slot(&bytes, &parameters.filesystem_identifier) {
                    found.push((root.checkpoint_txg.0, root.instance.0));
                }
            }
        }
    }
    found.sort_unstable();
    found.dedup();
    found
}

fn newest_of(image: &MemoryPool) -> (u64, u32) {
    *ring_of(image).iter().max_by_key(|(txg, instance)| (*txg, *instance)).expect("环里有根")
}

/// 改最新根的 t、F、p（重封自证校验和）；只改根槽，不改 journal 记录。
fn rewrite_root_fields(image: &mut MemoryPool, txg: u64, threshold: u64, floor: u64, pinned: u8) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = slot_bytes_at(image, device, offset).expect("根槽读得出");
    bytes[THRESHOLD_OFFSET..THRESHOLD_OFFSET + 8].copy_from_slice(&threshold.to_le_bytes());
    bytes[FLOOR_OFFSET..FLOOR_OFFSET + 8].copy_from_slice(&floor.to_le_bytes());
    bytes[PIN_OFFSET] = pinned;
    let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_CHECKSUM_OFFSET);
    bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    image
        .devices
        .get_mut(&device)
        .expect("那块盘")
        .write(DeviceOffsetInBytes(offset), &bytes);
}

fn fields_of_newest(image: &MemoryPool) -> (u64, u64, u64, u8) {
    let (txg, _) = newest_of(image);
    let root = root_in_slot_of(image, txg).expect("最新根");
    (txg, root.reclaim_threshold.0, root.rollback_floor.0, root.pinned_ring_slot.0)
}

/// 一次发布的写里某一类写第一次出现的位置。
fn first_index_of(writes: &[RetainedWrite], kind: StepKind) -> Option<usize> {
    writes.iter().position(|write| write.kind == kind)
}

struct CrashImages {
    /// 发布之前：N 做完的样子（下一槽里住着旧根，没有 N+1 的记录）
    healthy: MemoryPool,
    /// N+1 的单元落了、记录没落
    before_record: MemoryPool,
    /// N+1 的单元与记录落了、根槽没落（下一槽里还住着旧根）
    before_root: MemoryPool,
    zero: MemoryPool,
    torn_like_the_enumeration: MemoryPool,
    torn_half: MemoryPool,
    root_write_bytes: usize,
}

fn crash_images_of_the_root_slot_write(before: &MemoryPool, writes: &[RetainedWrite]) -> CrashImages {
    let root_index = first_index_of(writes, StepKind::RootRecordFua).expect("这次发布的根槽写");
    let record_index = first_index_of(writes, StepKind::JournalRecord).expect("这次发布的记录");
    let root_write = &writes[root_index];
    let mut before_the_root = before.clone();
    before_the_root.apply_writes(&writes[..root_index]);
    let mut before_the_record = before.clone();
    before_the_record.apply_writes(&writes[..record_index]);
    let new_bytes = root_write.bytes().expect("根槽写带字节").to_vec();
    let old_bytes = PoolReader::read(before, root_write.device, root_write.offset, new_bytes.len()).expect("旧字节");
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
    let torn_half: Vec<u8> = new_bytes[..half].iter().chain(old_bytes[half..].iter()).copied().collect();
    // 崩溃点重放枚举的撕裂取法（`crash.rs` 的 `torn_image_of_in_place_overwrite` 的文档，另写一份）：新旧不同的那一截前一半新、后一半旧。
    let first = (0..new_bytes.len()).find(|index| new_bytes[*index] != old_bytes[*index]);
    let torn_enum: Vec<u8> = match first {
        None => new_bytes.clone(),
        Some(first) => {
            let last = (0..new_bytes.len()).rev().find(|index| new_bytes[*index] != old_bytes[*index]).expect("有最后一个");
            let first_old = first + (last - first + 1) / 2;
            new_bytes[..first_old].iter().chain(old_bytes[first_old..].iter()).copied().collect()
        }
    };
    CrashImages {
        healthy: before.clone(),
        before_record: before_the_record,
        before_root: before_the_root.clone(),
        zero: with_slot(vec![0u8; new_bytes.len()]),
        torn_like_the_enumeration: with_slot(torn_enum),
        torn_half: with_slot(torn_half),
        root_write_bytes: new_bytes.len(),
    }
}

fn verdict_of_all(image: &MemoryPool) -> (String, Vec<&'static str>) {
    I718_ALL.with(|cell| *cell.borrow_mut() = None);
    let verdicts = check_pool_image(image);
    let red: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    let all = I718_ALL.with(|cell| *cell.borrow());
    let text = match all {
        Some(values) => values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","),
        None => "none".to_string(),
    };
    (text, red)
}

/// 一张镜像一行：`ROW cell= hist= S= image= kind= newest= t= F= p= next_slot_root= all=… red=…`。
fn row(cell: &str, hist: &str, image_name: &str, kind: &str, image: &MemoryPool) {
    let (text, red) = verdict_of_all(image);
    let (newest, instance) = newest_of(image);
    let root = root_in_slot_of(image, newest).expect("最新根");
    let next = root_in_slot_of(image, newest + 1).map(|r| r.checkpoint_txg.0);
    println!(
        "ROW cell={cell} hist={hist} S={} image={image_name} kind={kind} newest={newest} inst={instance} t={} F={} p={} next_slot_root={next:?} all={text} red={red:?}",
        slots(),
        root.reclaim_threshold.0,
        root.rollback_floor.0,
        root.pinned_ring_slot.0,
    );
}

// ───────────── 第三段：错写者扫描（以「N 做完的健康镜像」当判官）与格 A 一圈扫 ─────────────

fn all_of(image: &MemoryPool) -> Option<[i8; 11]> {
    let _ = check_pool_image(image);
    I718_ALL.with(|cell| *cell.borrow())
}

/// 错写者扫描：在 N 的根上把 (t', F', p') 逐个换掉，换完同样的字节同时放在「健康镜像」（下一槽里旧根完好）与「写坏镜像」上。
/// 健康镜像上今天的严格求等是判官：它说成立的 (t', F', p') 才是诚实写者会写的；写坏镜像上各候选对「判官说错」的组合收了几个。
fn scan(cell: &str, hist: &str, label: &str, healthy: &MemoryPool, failed: &MemoryPool) {
    if std::env::var("I718_NOSCAN").is_ok() {
        return;
    }
    let (n, t0, f0, p0) = fields_of_newest(failed);
    let s = slots();
    let mut floors: Vec<u64> = vec![f0, s, n.saturating_sub(3 * s) + 1];
    floors.retain(|f| *f <= n);
    floors.sort_unstable();
    floors.dedup();
    let mut pins: Vec<u8> = vec![255, p0];
    pins.sort_unstable();
    pins.dedup();
    // [legit_accepted, legit_red, legit_na, wrong_accepted, wrong_red, wrong_na]
    let mut counts = [[0u32; 6]; 11];
    let mut wrong_accepted: Vec<Vec<(u64, u64, u8)>> = vec![Vec::new(); 11];
    let mut total = 0u32;
    for threshold in 0..=n {
        for floor in &floors {
            for pin in &pins {
                let mut h = healthy.clone();
                rewrite_root_fields(&mut h, n, threshold, *floor, *pin);
                let oracle = all_of(&h).map(|a| a[0]).unwrap_or(-9);
                let mut x = failed.clone();
                rewrite_root_fields(&mut x, n, threshold, *floor, *pin);
                let Some(all) = all_of(&x) else { continue };
                total += 1;
                for (i, v) in all.iter().enumerate() {
                    let slot = match (oracle == 1, *v) {
                        (true, 1) => 0,
                        (true, 0) => 1,
                        (true, _) => 2,
                        (false, 1) => 3,
                        (false, 0) => 4,
                        (false, _) => 5,
                    };
                    counts[i][slot] += 1;
                    if slot == 3 {
                        wrong_accepted[i].push((threshold, *floor, *pin));
                    }
                }
            }
        }
    }
    println!(
        "SCAN cell={cell} hist={hist} label={label} S={s} N={n} true_t={t0} true_F={f0} true_p={p0} combos={total} floors={floors:?} pins={pins:?}"
    );
    for (i, name) in NAMES.iter().enumerate() {
        let c = counts[i];
        let sample: Vec<_> = wrong_accepted[i].iter().take(8).collect();
        println!(
            "  SCANCAND label={label} S={s} N={n} cand={name} legit_accepted={} legit_red={} legit_na={} wrong_accepted={} wrong_red={} wrong_na={} wrong_accepted_sample={sample:?}",
            c[0], c[1], c[2], c[3], c[4], c[5]
        );
    }
}

fn judge_family(cell: &str, hist: &str, images: &CrashImages) {
    row(cell, hist, "healthy_n", "legit", &images.healthy);
    row(cell, hist, "before_record", "legit", &images.before_record);
    row(cell, hist, "before_root", "legit", &images.before_root);
    row(cell, hist, "zero", "legit", &images.zero);
    row(cell, hist, "torn_enum", "legit", &images.torn_like_the_enumeration);
    row(cell, hist, "torn_half", "legit", &images.torn_half);
}

/// 一圈一圈扫：同一个写者从第一个文件一路写到 N+1 <= `until`，每次发布都造全套崩溃镜像（`empty_every` 非 0 时每隔几次发布做一次空发布）。
fn sweep(cell: &str, hist: &str, extra_mounts: u64, empty_every: u64, until: u64, scan_at: &[u64]) {
    let mut writer = fresh_writer(extra_mounts);
    let mut count = 0u64;
    while txg_of(&writer) < until {
        count += 1;
        let empty = empty_every > 0 && count % empty_every == 0;
        let (before, writes) = recorded_publish(&mut writer, empty);
        let n = txg_of(&writer) - 1;
        let images = crash_images_of_the_root_slot_write(&before, &writes);
        let rows_from: u64 = std::env::var("I718_ROWS_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if std::env::var("I718_NOROWS").is_err() && n >= rows_from {
            judge_family(cell, hist, &images);
        }
        if count == 1 {
            println!("ROOTWRITE S={} root_slot_write_bytes={} sector_bytes_of_the_enumeration=512", slots(), images.root_write_bytes);
        }
        if scan_at.contains(&n) && (!few() || n == ring() + 2 || n == 2 * ring() - 1) {
            scan(cell, hist, &format!("{hist}_n{n}_zero"), &images.healthy, &images.zero);
        }
    }
}

#[test]
fn cell_a_sweep_overwrites_only() {
    let s = ring();
    sweep("A", "overwrite", 0, 0, 2 * s + 2, &[s - 4, s - 1, s + 2, s + slots() + 1, 2 * s - 1]);
}

/// 只扫一个点（不打各镜像的行）：大 S 下把扫描与逐镜像分开跑。
#[test]
fn cell_a_scan_only() {
    let s = ring();
    sweep("A", "overwrite", 0, 0, s + 3, &[s + 2]);
}

/// journal 环（1 MiB、每条记录一个 4096 字节槽）转过一圈以上之后，题面形的写坏镜像上「点名 N+1 的记录」这一前提还判不判得出。
#[test]
fn cell_w_journal_ring_wrapped() {
    sweep("W", "journal_wrapped", 0, 0, 330, &[]);
}

#[test]
fn cell_h1_sweep_with_empty_publishes_every_second() {
    let s = ring();
    sweep("H1", "empty_every_2", 0, 2, 2 * s + 2, &[s + 2]);
}

#[test]
fn cell_h2_sweep_with_two_extra_mounts_before_the_first_file() {
    let s = ring();
    sweep("H2", "extra_mounts_2", 2, 0, 2 * s + 2, &[s + 2]);
}

// ───────────── 第四段：格 B（环洞落在下一槽、重复的洞）与格 F（挂载时钉住的槽轮到下一槽） ─────────────

fn writer_after_a_hole(base_last: u64) -> (Writer, u64) {
    let mut writer = fresh_writer(0);
    overwrite_through(&mut writer, base_last);
    let (crashed, row_txg) = crash_a_mount_before_its_row_root(&pool_of(&writer.devices));
    (writer_mounted_on(&crashed), row_txg)
}

fn bases() -> Vec<u64> {
    let r = ring();
    if let Some(only) = std::env::var("I718_B_BASE").ok().and_then(|v| v.parse().ok()) {
        return vec![only];
    }
    if few() {
        vec![r, 2 * r - 1]
    } else {
        vec![r, r + 1, r + slots(), 2 * r - 1]
    }
}

/// 洞那一槽轮到下一槽：写到 N（N+1 落在洞上），N+1 的根槽写失败。重复三圈：洞那一槽再被一次崩在写行之前的挂载盖不掉，旧内容一圈比一圈老。
#[test]
fn cell_b_hole_in_the_next_slot() {
    let r = ring();
    for base in bases() {
        let (mut writer, mut row_txg) = writer_after_a_hole(base);
        let repeats: u64 = std::env::var("I718_B_REPEATS").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
        for repeat in 0..repeats {
            overwrite_through(&mut writer, row_txg + r - 1);
            let hist = format!("hole_base{base}_repeat{repeat}");
            let image_at_n = pool_of(&writer.devices);
            let (before, writes) = recorded_publish(&mut writer, false);
            let images = crash_images_of_the_root_slot_write(&before, &writes);
            judge_family("B", &hist, &images);
            if repeat + 1 == repeats && slots() <= 5 {
                scan("B", &hist, &format!("{hist}_zero"), &images.healthy, &images.zero);
            }
            // 下一圈：洞那一槽上再崩一次挂载（写行的根槽写之前），旧内容留着；写者在崩溃后的盘面上重新挂载
            let (crashed, next_row_txg) = crash_a_mount_before_its_row_root(&image_at_n);
            writer = writer_mounted_on(&crashed);
            row_txg = next_row_txg;
        }
    }
}

fn pinned_slot_lap(base: u64, label: &str) {
    let r = ring();
    let mut writer = fresh_writer(0);
    overwrite_through(&mut writer, base);
    let (before, writes) = recorded_publish(&mut writer, false);
    let failed = crash_images_of_the_root_slot_write(&before, &writes).zero;
    let mut second = writer_mounted_on(&failed);
    overwrite_through(&mut second, base + r);
    let (before2, writes2) = recorded_publish(&mut second, false);
    let images = crash_images_of_the_root_slot_write(&before2, &writes2);
    let hist = format!("pinned_after_base{base}_{label}");
    judge_family("F", &hist, &images);
    let (n, t, floor, pin) = fields_of_newest(&images.healthy);
    let mut bad_t = images.healthy.clone();
    rewrite_root_fields(&mut bad_t, n, floor, floor, pin);
    row("F", &hist, "healthy_n_bad_t_as_floor", "bad", &bad_t);
    let mut bad_p = images.healthy.clone();
    rewrite_root_fields(&mut bad_p, n, t, floor, 255);
    row("F", &hist, "healthy_n_bad_p_255", "bad", &bad_p);
    for (name, image) in [("before_root", &images.before_root), ("zero", &images.zero)] {
        let mut in_flight_bad_p = image.clone();
        rewrite_root_fields(&mut in_flight_bad_p, n, t, floor, 255);
        row("F", &hist, &format!("{name}_bad_p_255"), "bad", &in_flight_bad_p);
        let mut in_flight_bad_t = image.clone();
        rewrite_root_fields(&mut in_flight_bad_t, n, floor, floor, pin);
        row("F", &hist, &format!("{name}_bad_t_as_floor"), "bad", &in_flight_bad_t);
    }
    scan("F", &hist, &format!("{hist}_zero"), &images.healthy, &images.zero);
}

#[test]
fn cell_f_pinned_slot_comes_round_to_the_next_slot() {
    let r = ring();
    pinned_slot_lap(r + 2, "first_revolution");
    if !few() {
        pinned_slot_lap(2 * r, "second_revolution");
    }
}

// ───────────── 第五段：格 G（下一次发布是新实例的写行）、格 R（下一次发布是回退）、格 Y/BR（事后读坏） ─────────────

/// 一次可写挂载（新实例）：录下它的写，交回挂载之前的盘面与写（写行那次发布是它的第一次根槽写）。
fn recorded_mount(image: &MemoryPool) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    mount_writable(&parameters, &mut recording).expect("可写挂载");
    let (writes, _segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    (image.clone(), writes)
}

#[test]
fn cell_g_the_next_publish_is_the_row_of_a_new_instance() {
    let r = ring();
    for last in if few() { vec![r + 2, 2 * r + 1] } else { vec![r + 2, 2 * r + 1, 2 * r + slots()] } {
        let mut writer = fresh_writer(0);
        overwrite_through(&mut writer, last);
        let (before, writes) = recorded_mount(&pool_of(&writer.devices));
        let images = crash_images_of_the_root_slot_write(&before, &writes);
        let hist = format!("new_instance_row_after_n{last}");
        judge_family("G", &hist, &images);
        scan("G", &hist, &format!("{hist}_zero"), &images.healthy, &images.zero);
    }
}

fn recorded_rollback(writer: &mut Writer, target: RollbackTarget) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let before = pool_of(&writer.devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    roll_back_by_a_forward_publish(&parameters, &mut recording, &mut writer.allocator, &mut writer.current, target)
        .expect("回退");
    let (writes, _segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let mut after = before.clone();
    after.apply_writes(&writes);
    writer.devices = devices_of(&after);
    (before, writes)
}

/// 回退到环里最旧的根，回退那次发布（N+1）的根槽正是那条最旧根住的槽（C560 的来历）。
#[test]
fn cell_r_the_next_publish_is_a_rollback_to_the_oldest_ring_root() {
    let r = ring();
    for last in [r + 2, 2 * r + 1] {
        let mut writer = fresh_writer(0);
        overwrite_through(&mut writer, last);
        let target = RollbackTarget {
            instance: writer.instance,
            checkpoint_txg: CheckpointTxg(last + 1 - r),
        };
        let (before, writes) = recorded_rollback(&mut writer, target);
        let images = crash_images_of_the_root_slot_write(&before, &writes);
        let hist = format!("rollback_to_oldest_after_n{last}");
        judge_family("R", &hist, &images);
        scan("R", &hist, &format!("{hist}_zero"), &images.healthy, &images.zero);
    }
}

fn zero_the_slot_of(image: &MemoryPool, txg: u64) -> MemoryPool {
    let (device, offset) = root_slot_of(txg);
    let mut damaged = image.clone();
    damaged.devices.get_mut(&device).expect("那块盘").write(DeviceOffsetInBytes(offset), &vec![0u8; ROOT_SLOT_BYTES]);
    damaged
}

/// 事后读坏（位腐）：健康镜像上没有任何在飞的发布，某一槽读成全零。满环时读坏最旧的（= 下一槽）、环没满时读坏最旧的、满环时读坏居中的。
#[test]
fn cell_y_and_br_read_corruption_after_the_fact() {
    let r = ring();
    let mut writer = fresh_writer(0);
    let mut count = 0u64;
    while txg_of(&writer) < 2 * r + 1 {
        let n = txg_of(&writer);
        let image = pool_of(&writer.devices);
        let ring_now = ring_of(&image);
        let oldest = ring_now.iter().map(|(txg, _)| *txg).min().expect("环里有根");
        let middle = ring_now[ring_now.len() / 2].0;
        if n + 1 < r {
            row("Y", "young_ring", &format!("n{n}_oldest_root_unreadable"), "bitrot", &zero_the_slot_of(&image, oldest));
        } else {
            row("BR", "full_ring", &format!("n{n}_oldest_root_unreadable"), "bitrot", &zero_the_slot_of(&image, n + 1 - r));
            row("BR", "full_ring", &format!("n{n}_middle_root_unreadable"), "bitrot_middle", &zero_the_slot_of(&image, middle));
        }
        count += 1;
        recorded_publish(&mut writer, false);
    }
    println!("BRDONE S={} publishes={count}", slots());
}

/// 第一圈里写失败的槽（那一槽读回全零）被可写挂载钉住，钉值 = 那一槽的位置 = 它再轮到时的 N+1 − 3S：
/// 这时「p 写 255 而其实钉着」的坏写者与诚实写者（下一槽里旧根完好、t = N+1 − 3S）在镜像上逐字节相同。
#[test]
fn cell_f2_pinned_slot_whose_failure_was_in_the_first_revolution() {
    pinned_slot_lap(ring() - 4, "first_lap_failure");
}
