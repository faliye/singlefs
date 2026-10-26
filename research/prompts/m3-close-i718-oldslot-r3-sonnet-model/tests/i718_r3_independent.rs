//! m3-close-i718-oldslot-r3 辩方腿原型（第一段：装置、读盘、造盘）。
//! 与前两轮四条腿的做法不同：不改池级 checker，不往 walk.rs 里装开关。池级 checker 原样跑每一张镜像，只取它对 I-7.18 的判定；
//! 各候选（丙、乙字面、乙加闸、丁乙、丁乙同实例、甲、丁甲）在这份原型里从镜像上自己读出来的字段（最新根的 t、F、p，环上读得出的根，
//! 下一槽，journal 里点名 N+1 的记录）后置分类。分类用的现算式先拿池级 checker 的原判核对（`calib=`）。
//! 只放在三方腿的副本里跑，不进入库的用例。
mod common_root_ring_holes;

use common_root_ring_holes::{
    content_of_the_version, crash_a_mount_before_its_row_root, devices_of,
    fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools, pool_of, publish_empty,
    recording_of, RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
    JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::PoolReader;
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

const ROOT_CHECKSUM_OFFSET: usize = 138;
const FLOOR_OFFSET: usize = 130;
const THRESHOLD_OFFSET: usize = 428;
const PIN_OFFSET: usize = 436;
const ROOT_SLOT_BYTES: usize = 512;
const JOURNAL_RECORD_SLOT_BYTES: usize = 4096;

fn slots() -> u64 {
    std::env::var("I718_S").ok().and_then(|v| v.parse().ok()).unwrap_or(4)
}

/// `I718_SHARD=k/n`：历史按序号取模分给 n 个进程，这一个进程只跑序号模 n 等于 k 的。没设就全跑。
fn mine(index: u64) -> bool {
    let Ok(text) = std::env::var("I718_SHARD") else { return true };
    let (k, n) = text.split_once('/').expect("I718_SHARD 写成 k/n");
    index % n.parse::<u64>().expect("n") == k.parse::<u64>().expect("k")
}

fn ring_length() -> u64 {
    3 * slots()
}

fn fsid_low() -> u64 {
    let fsid = parameters_of_the_ring_hole_pools().filesystem_identifier;
    u64::from_le_bytes(fsid[..8].try_into().expect("fsid 前 8 字节"))
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

/// mkfs → 可写挂载（实例 1）→ 第一个文件。
fn fresh_writer() -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = blank_devices();
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

fn txg_of(writer: &Writer) -> u64 {
    writer.current.root.checkpoint_txg.0
}

/// 一次挂载的录像：挂载之前的盘面、这次挂载的全部写；挂载之后接着写的写者（这一版没有文件时没有）。
struct MountRecording {
    before: MemoryPool,
    writes: Vec<RetainedWrite>,
}

fn recorded_mount(image: &MemoryPool) -> (MountRecording, Option<Writer>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    let Mounted { output, allocator, current } =
        mount_writable(&parameters, &mut recording).expect("可写挂载");
    let (writes, _segments) =
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
    (MountRecording { before: image.clone(), writes }, writer)
}

/// 下一次覆盖写，录下它的写：交回写之前的盘面与这一次的写（写者前进到写完）。
fn recorded_overwrite(writer: &mut Writer) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let before = pool_of(&writer.devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let salt = txg_of(writer) + 1;
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
    let (writes, _segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let mut after = before.clone();
    after.apply_writes(&writes);
    writer.devices = devices_of(&after);
    writer.current = output;
    (before, writes)
}

/// 空发布（文件不变）：与覆盖写同一个录法。
fn recorded_empty(writer: &mut Writer) -> (MemoryPool, Vec<RetainedWrite>) {
    let parameters = parameters_of_the_ring_hole_pools();
    let before = pool_of(&writer.devices);
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(&before, &stream);
    let output = {
        let mut pool_writer = PoolWriter::new(&parameters, recording.as_mut_slice());
        publish_empty(&mut pool_writer, &mut writer.allocator, &writer.current, writer.instance)
            .expect("空发布")
    };
    let (writes, _segments) =
        writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let mut after = before.clone();
    after.apply_writes(&writes);
    writer.devices = devices_of(&after);
    writer.current = output;
    (before, writes)
}

// ───────────── 读盘：根槽、环、journal 记录 ─────────────

fn root_slot_of(txg: u64) -> (DeviceIdentity, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    (device, slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0)
}

fn read_bytes(image: &MemoryPool, device: DeviceIdentity, offset: u64, length: usize) -> Option<Vec<u8>> {
    image.devices.get(&device)?;
    PoolReader::read(image, device, DeviceOffsetInBytes(offset), length)
}

fn root_in_slot_of(image: &MemoryPool, txg: u64) -> Option<RootRecord> {
    let (device, offset) = root_slot_of(txg);
    RootRecord::parse_slot(
        &read_bytes(image, device, offset, ROOT_SLOT_BYTES)?,
        &parameters_of_the_ring_hole_pools().filesystem_identifier,
    )
}

/// 环上每一个槽读得出、自证过的根：(txg, 实例)。槽按位置数，不按 txg 反查（读到的根 txg 对不对位置不管）。
fn ring_roots(image: &MemoryPool) -> Vec<(u64, u32)> {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut found = Vec::new();
    for region in 0..3u64 {
        for slot in 0..slots() {
            let device = parameters.region_devices[usize::try_from(region).expect("区域号")];
            let offset = slot_offset(
                singlefs_core::root_ring::RootRingSlot { region, slot },
                parameters.geometry.fixed_structure_slot_spacing,
            )
            .0;
            if let Some(bytes) = read_bytes(image, device, offset, ROOT_SLOT_BYTES) {
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

/// 两块盘的 journal 环里自证过的记录：(实例, checkpoint_txg)。环上 256 个槽逐个读（计数器 1..=256 恰好走满一圈）。
fn journal_records(image: &MemoryPool) -> Vec<(u32, u64)> {
    let mut found = Vec::new();
    for device in image.devices.keys() {
        for counter in 1..=(JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS / 4096) {
            let offset = record_offset(counter, JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS).0;
            if let Some(bytes) = read_bytes(image, *device, offset, JOURNAL_RECORD_SLOT_BYTES) {
                if let Some(record) = JournalRecord::parse(&bytes, fsid_low()) {
                    found.push((record.instance.0, record.checkpoint_txg.0));
                }
            }
        }
    }
    found.sort_unstable();
    found.dedup();
    found
}

/// 改某个 txg 那条根的 t、F、p（重封自证校验和）；只改根槽，不改 journal 记录。
fn rewrite_root_fields(image: &mut MemoryPool, txg: u64, threshold: u64, floor: u64, pinned: u8) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = read_bytes(image, device, offset, ROOT_SLOT_BYTES).expect("根槽读得出");
    bytes[THRESHOLD_OFFSET..THRESHOLD_OFFSET + 8].copy_from_slice(&threshold.to_le_bytes());
    bytes[FLOOR_OFFSET..FLOOR_OFFSET + 8].copy_from_slice(&floor.to_le_bytes());
    bytes[PIN_OFFSET] = pinned;
    let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_CHECKSUM_OFFSET);
    bytes[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    image.devices.get_mut(&device).expect("那块盘").write(DeviceOffsetInBytes(offset), &bytes);
}

/// 把某个 txg 的根槽整槽写成全零（模拟那一槽写失败或事后读坏）。
fn zero_the_slot_of(image: &mut MemoryPool, txg: u64) {
    let (device, offset) = root_slot_of(txg);
    image
        .devices
        .get_mut(&device)
        .expect("那块盘")
        .write(DeviceOffsetInBytes(offset), &vec![0u8; ROOT_SLOT_BYTES]);
}

// ───────────── 崩溃镜像：一次发布的根槽写之前 / 写坏 ─────────────

struct CrashImages {
    /// 这次发布之前：N 做完的样子（下一槽里住着旧根，没有 N+1 的记录）
    healthy: MemoryPool,
    /// N+1 的单元落了、记录没落
    before_record: MemoryPool,
    /// N+1 的单元与记录落了、根槽没落（下一槽里还住着旧根）
    before_root: MemoryPool,
    /// 根槽写成全 0
    zero: MemoryPool,
    /// 根槽写一半新一半旧
    torn_half: MemoryPool,
    /// 崩溃点重放枚举撕原地覆写的取法（`crash.rs` 的 `torn_image_of_in_place_overwrite` 文档：新旧不同的那一截前一半新、后一半旧）
    torn_enumeration_style: MemoryPool,
}

impl CrashImages {
    fn named(&self) -> [(&'static str, &MemoryPool); 6] {
        [
            ("healthy", &self.healthy),
            ("before_record", &self.before_record),
            ("before_root", &self.before_root),
            ("zero", &self.zero),
            ("torn_half", &self.torn_half),
            ("torn_enum", &self.torn_enumeration_style),
        ]
    }
}

/// `root_ordinal`：这段写里第几次根槽写（一次覆盖写只有 0；一次挂载的写行是 0）。
fn crash_images(before: &MemoryPool, writes: &[RetainedWrite], root_ordinal: usize) -> CrashImages {
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let root_index = root_indices[root_ordinal];
    let record_index = writes[..root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("这次发布的记录写在根槽写之前");
    // 记录写之前的第一条属于这次发布的写：从上一次根槽写之后算起，这里只要「记录落盘之前」那一刻（取这次发布第一条记录写）。
    let first_record_of_this_publish = writes[..root_index]
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord)
        .map(|(index, _)| index)
        .find(|index| root_ordinal == 0 || *index > root_indices[root_ordinal - 1])
        .unwrap_or(record_index);
    let base_of_this_publish = if root_ordinal == 0 {
        0
    } else {
        root_indices[root_ordinal - 1] + 1
    };
    let mut healthy = before.clone();
    healthy.apply_writes(&writes[..base_of_this_publish]);
    let mut before_record = before.clone();
    before_record.apply_writes(&writes[..first_record_of_this_publish]);
    let mut before_root = before.clone();
    before_root.apply_writes(&writes[..root_index]);
    let root_write = &writes[root_index];
    let new_bytes = root_write.bytes().expect("根槽写带字节").to_vec();
    let old_bytes = PoolReader::read(&before_root, root_write.device, root_write.offset, new_bytes.len())
        .unwrap_or_else(|| vec![0u8; new_bytes.len()]);
    let with_slot = |bytes: Vec<u8>| {
        let mut image = before_root.clone();
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
    let torn: Vec<u8> = new_bytes[..half].iter().chain(old_bytes[half..].iter()).copied().collect();
    let differing = |index: &usize| old_bytes[*index] != new_bytes[*index];
    let torn_enumeration_bytes: Vec<u8> = match (0..new_bytes.len()).find(differing) {
        None => new_bytes.clone(),
        Some(first_differing) => {
            let last_differing = (0..new_bytes.len()).rev().find(differing).expect("有第一个不同的就有最后一个");
            let first_old = first_differing + (last_differing - first_differing + 1) / 2;
            new_bytes[..first_old].iter().chain(old_bytes[first_old..].iter()).copied().collect()
        }
    };
    CrashImages {
        healthy,
        before_record,
        before_root: before_root.clone(),
        zero: with_slot(vec![0u8; new_bytes.len()]),
        torn_half: with_slot(torn),
        torn_enumeration_style: with_slot(torn_enumeration_bytes),
    }
}

// ───────────── 后置分类：字段读出来，候选自己算 ─────────────

struct Facts {
    txg: u64,
    instance: u32,
    t: u64,
    floor: u64,
    pinned: u8,
    ring: Vec<(u64, u32)>,
    next_unreadable: bool,
    record_names_next: bool,
    record_names_next_same_instance: bool,
}

fn facts_of(image: &MemoryPool) -> Option<Facts> {
    let ring = ring_roots(image);
    let (txg, instance) = *ring.iter().max_by_key(|(txg, instance)| (*txg, *instance))?;
    let root = root_in_slot_of(image, txg)?;
    let records = journal_records(image);
    Some(Facts {
        txg,
        instance,
        t: root.reclaim_threshold.0,
        floor: root.rollback_floor.0,
        pinned: root.pinned_ring_slot.0,
        next_unreadable: root_in_slot_of(image, txg + 1).is_none(),
        record_names_next: records.iter().any(|(_, record_txg)| *record_txg == txg + 1),
        record_names_next_same_instance: records
            .iter()
            .any(|(record_instance, record_txg)| *record_txg == txg + 1 && *record_instance == instance),
        ring,
    })
}

impl Facts {
    /// 池级 checker 的现算式在镜像上的那一份：max(F, min(环上读得出的根, p 的钉值))，`extra` 是补进环的一条。
    fn recomputed(&self, extra: Option<u64>) -> u64 {
        let mut least = self.ring.iter().map(|(txg, _)| *txg).min();
        let pinned = (u64::from(self.pinned) < ring_length() && self.pinned != 255).then_some(u64::from(self.pinned));
        for candidate in [pinned, extra].into_iter().flatten() {
            least = Some(least.map_or(candidate, |value| value.min(candidate)));
        }
        least.map_or(self.floor, |value| value.max(self.floor))
    }

    fn big(&self) -> bool {
        self.txg + 1 >= ring_length()
    }

    /// 旧内容取 N+1−k·3S 补进环之后的现算，k = 1 起，到 N+1−k·3S ≥ 0 止。
    fn fills(&self) -> Vec<u64> {
        (1..=((self.txg + 1) / ring_length()))
            .map(|k| self.recomputed(Some(self.txg + 1 - k * ring_length())))
            .collect()
    }
}

fn letter_of(verdict: Option<&InvariantVerdict>) -> char {
    match verdict {
        Some(InvariantVerdict::Holds) => 'G',
        Some(InvariantVerdict::Violated(_)) => 'R',
        Some(InvariantVerdict::NotApplicable(_)) => 'N',
        Some(InvariantVerdict::PartiallyJudged(_)) => 'P',
        None => '-',
    }
}

const CANDIDATES: [&str; 8] = ["today", "yi_lit", "yi_gate", "ding_yi", "ding_yi_same", "jia", "ding_jia", "p19"];

struct Judged {
    facts: Facts,
    letters: [char; 8],
    calib: &'static str,
    red: Vec<&'static str>,
}

/// 池级 checker 原样判一遍，取 I-7.18 的判定；各候选按镜像上读出来的字段后置分类。G 成立、R 判红、N 不适用。
fn judged(image: &MemoryPool) -> Result<Judged, (char, Vec<&'static str>)> {
    let verdicts = check_pool_image(image);
    let find = |name: &str| verdicts.iter().find(|(invariant, _)| *invariant == name).map(|(_, verdict)| verdict);
    let today = letter_of(find("I-7.18"));
    let p19 = letter_of(find("I-7.19"));
    let red: Vec<&'static str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect();
    let Some(facts) = facts_of(image) else {
        return Err((today, red));
    };
    let is_judged = today == 'G' || today == 'R';
    let calc0 = facts.recomputed(None);
    let calib = if is_judged {
        if (facts.t == calc0) == (today == 'G') { "ok" } else { "BAD" }
    } else {
        "na"
    };
    let unreadable = facts.next_unreadable;
    let big = facts.big();
    let pick = |applies: bool, replaced: char| if applies && is_judged { replaced } else { today };
    let fills = facts.fills();
    let accepts_one = facts.t == calc0 || fills.first().is_some_and(|value| *value == facts.t);
    let accepts_any = facts.t == calc0 || fills.contains(&facts.t);
    let letters = [
        today,
        pick(unreadable, 'N'),
        pick(unreadable && big, 'N'),
        pick(unreadable && big && facts.record_names_next, 'N'),
        pick(unreadable && big && facts.record_names_next_same_instance, 'N'),
        pick(unreadable && big, if accepts_one { 'G' } else { 'R' }),
        pick(unreadable && big && facts.record_names_next, if accepts_any { 'G' } else { 'R' }),
        p19,
    ];
    Ok(Judged { facts, letters, calib, red })
}

fn row(cell: &str, hist: &str, img: &str, image: &MemoryPool) {
    match judged(image) {
        Err((today, red)) => println!("ROW cell={cell} hist={hist} S={} img={img} newest=none today={today} red={red:?}", slots()),
        Ok(judged) => {
            let facts = &judged.facts;
            let mut text = String::new();
            for (name, letter) in CANDIDATES.iter().zip(judged.letters.iter()) {
                text.push_str(&format!(" {name}={letter}"));
            }
            println!(
                "ROW cell={cell} hist={hist} S={} img={img} N={} inst={} t={} F={} p={} next_unreadable={} rec_any={} rec_same={}{text} calib={} red={:?}",
                slots(),
                facts.txg,
                facts.instance,
                facts.t,
                facts.floor,
                facts.pinned,
                u8::from(facts.next_unreadable),
                u8::from(facts.record_names_next),
                u8::from(facts.record_names_next_same_instance),
                judged.calib,
                judged.red,
            );
        }
    }
}

fn rows_of(cell: &str, hist: &str, images: &CrashImages) {
    for (name, image) in images.named() {
        row(cell, hist, name, image);
    }
}

/// 错写者扫描：把 N 那条根的 (t, p) 逐个换掉（F 不动），换完同样的字节同时放在 `healthy`（下一槽里旧根完好或本来如此）和 `damaged` 上。
/// 以 `healthy` 上池级 checker 的原判当判官：它说成立的 (t, p) 才是诚实写者会写的；数 `damaged` 上各候选
/// 对「判官说错」的组合收了几个（wrong_accepted），和把「判官说对」的组合判红了几个（honest_rejected）。
fn scan(cell: &str, hist: &str, label: &str, healthy: &MemoryPool, damaged: &MemoryPool) {
    scan_with(cell, hist, label, healthy, damaged, false);
}

fn scan_with(cell: &str, hist: &str, label: &str, healthy: &MemoryPool, damaged: &MemoryPool, all_pins: bool) {
    if std::env::var("I718_NOSCAN").is_ok() {
        return;
    }
    let Some(facts) = facts_of(damaged) else { return };
    let (n, floor, true_p) = (facts.txg, facts.floor, facts.pinned);
    let mut wrong_accepted = [0usize; 7];
    let mut honest_rejected = [0usize; 7];
    let mut honest = 0usize;
    let mut total = 0usize;
    let mut accepted_examples: [Vec<(u64, u8)>; 7] = Default::default();
    let mut pins = vec![255u8];
    if true_p != 255 {
        pins.push(true_p);
    }
    if all_pins {
        pins = (0..u8::try_from(ring_length()).expect("环长装得进一个字节")).chain([255u8]).collect();
    }
    // t 从 0 扫到 N 全部（环洞形的诚实写者写的 t 比最新根早两圈以上）。
    let window_start = 0u64;
    let mut candidates_of_t: Vec<u64> = (window_start..=n).collect();
    for extra in [0, floor] {
        if !candidates_of_t.contains(&extra) {
            candidates_of_t.push(extra);
        }
    }
    for t in candidates_of_t {
        for pin in &pins {
            let mut good = healthy.clone();
            let mut bad = damaged.clone();
            rewrite_root_fields(&mut good, n, t, floor, *pin);
            rewrite_root_fields(&mut bad, n, t, floor, *pin);
            let Ok(oracle) = judged(&good) else { continue };
            let Ok(under_test) = judged(&bad) else { continue };
            if oracle.letters[0] == 'N' {
                continue;
            }
            total += 1;
            let honest_one = oracle.letters[0] == 'G';
            honest += usize::from(honest_one);
            for index in 0..7 {
                let accepted = under_test.letters[index] == 'G' || under_test.letters[index] == 'N';
                if accepted && !honest_one {
                    wrong_accepted[index] += 1;
                    accepted_examples[index].push((t, *pin));
                }
                if !accepted && honest_one {
                    honest_rejected[index] += 1;
                }
            }
        }
    }
    for index in 0..7 {
        println!(
            "SCAN cell={cell} hist={hist} S={} img={label} N={n} F={floor} p_true={true_p} cand={} total={total} honest={honest} wrong_accepted={} honest_rejected={} examples={:?}",
            slots(),
            CANDIDATES[index],
            wrong_accepted[index],
            honest_rejected[index],
            &accepted_examples[index][..accepted_examples[index].len().min(4)],
        );
    }
}

// ───────────── 格 ─────────────

fn sweep(cell: &str, hist: &str, writer: &mut Writer, last: u64, print_rows: bool, hook: &mut dyn FnMut(u64, &CrashImages)) {
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while txg_of(writer) < last {
        let n = txg_of(writer);
        let (before, writes) = recorded_overwrite(writer);
        let images = crash_images(&before, &writes, 0);
        if print_rows {
            rows_of(cell, hist, &images);
        }
        hook(n + 1, &images);
    }
}

fn quiet(_: u64, _: &CrashImages) {}

fn overwrite_quietly(writer: &mut Writer, last: u64) {
    while txg_of(writer) < last {
        recorded_overwrite(writer);
    }
}

/// 三份写者：诚实、t 写成 F、p 写成 255。五张镜像上的最新根（N）逐个改字段，各打一行。
fn bad_writer_rows(cell: &str, hist: &str, images: &CrashImages) {
    let facts = facts_of(&images.healthy).expect("健康镜像有根");
    for (label, t, p) in [
        ("honest", facts.t, facts.pinned),
        ("bad_t_as_floor", facts.floor, facts.pinned),
        ("bad_p_255", facts.t, 255u8),
    ] {
        for (name, image) in images.named() {
            let mut edited = image.clone();
            rewrite_root_fields(&mut edited, facts.txg, t, facts.floor, p);
            row(cell, &format!("{hist}/{label}"), name, &edited);
        }
    }
}

/// 格 A（题面形）加两种事后读坏：从第一个文件一路覆盖写，每次发布造五张镜像；健康镜像上再把环里最旧那条根的槽写成 0
/// （N+1 ≥ 3S 时是满环最旧根事后读坏，记 BR；之前是环没满时最旧根事后读坏，记 Y）。
#[test]
fn cell_a_with_the_read_faults_of_the_oldest_root() {
    let ring = ring_length();
    let mut writer = fresh_writer();
    let scan_at = [ring, ring + 2, 2 * ring, 2 * ring + 3];
    let last = std::env::var("I718_A_LAST").ok().and_then(|v| v.parse().ok()).unwrap_or(3 * ring + 2);
    sweep("A", "plain", &mut writer, last, true, &mut |next_txg, images| {
        let facts = facts_of(&images.healthy).expect("有根");
        let oldest = facts.ring.iter().map(|(txg, _)| *txg).min().expect("环非空");
        let mut rotted = images.healthy.clone();
        zero_the_slot_of(&mut rotted, oldest);
        row(if next_txg >= ring { "BR" } else { "Y" }, "plain", "oldest_slot_zero", &rotted);
        if scan_at.contains(&next_txg) {
            scan("A", "plain", "zero", &images.healthy, &images.zero);
        }
    });
}

/// 格 B（环洞落在下一槽）：实例 1 写到 L，实例 2 可写挂载崩在写行根槽写之前（洞），实例 3 可写挂载接着写，一直写到 N+1 落在洞上。
#[test]
fn cell_b_hole_at_the_next_slot() {
    let ring = ring_length();
    for l in ring..2 * ring {
        if !mine(l) {
            continue;
        }
        let mut first = fresh_writer();
        overwrite_quietly(&mut first, l);
        let (crashed, hole_txg) = crash_a_mount_before_its_row_root(&pool_of(&first.devices));
        let (_, third) = recorded_mount(&crashed);
        let mut writer = third.expect("洞之后的挂载有写者");
        let hist = format!("L={l},H={hole_txg}");
        let last = hole_txg + ring;
        sweep("B", &hist, &mut writer, last, true, &mut |next_txg, images| {
            if next_txg == last {
                scan("B", &hist, "zero", &images.healthy, &images.zero);
            }
        });
    }
}

/// 格 F（挂载时钉住的槽轮到下一槽）：N0+1 的根槽写成 0，新实例可写挂载把那一槽钉住，一直写到 N+1 回到那一槽；
/// 那一次发布的五张镜像上再造诚实、t 写成 F、p 写成 255 三份写者。
#[test]
fn cell_f_pinned_slot_comes_round_to_the_next_slot() {
    let ring = ring_length();
    for first_lap_next in ring..2 * ring {
        if !mine(first_lap_next) {
            continue;
        }
        let mut first = fresh_writer();
        overwrite_quietly(&mut first, first_lap_next - 1);
        let (before, writes) = recorded_overwrite(&mut first);
        let damaged = crash_images(&before, &writes, 0).zero;
        let (_, second) = recorded_mount(&damaged);
        let mut writer = second.expect("钉槽之后的挂载有写者");
        let hist = format!("N0plus1={first_lap_next}");
        let last = first_lap_next + ring;
        sweep("F", &hist, &mut writer, last, false, &mut |next_txg, images| {
            if next_txg == last {
                bad_writer_rows("F", &hist, images);
                scan("F", &hist, "zero", &images.healthy, &images.zero);
                scan("F", &hist, "before_root", &images.healthy, &images.before_root);
            }
        });
    }
}

/// 格 G（下一次发布是新实例的写行）：满环之后实例退出（崩，不卸载），新实例可写挂载，写行的根槽写失败。
#[test]
fn cell_g_row_publish_of_a_new_instance() {
    let ring = ring_length();
    for n in ring - 1..2 * ring + 1 {
        if !mine(n) {
            continue;
        }
        let mut first = fresh_writer();
        overwrite_quietly(&mut first, n);
        let image = pool_of(&first.devices);
        let (recording, _) = recorded_mount(&image);
        let images = crash_images(&recording.before, &recording.writes, 0);
        let hist = format!("N={n}");
        rows_of("G", &hist, &images);
        if n % ring == 1 {
            scan("G", &hist, "zero", &images.healthy, &images.zero);
        }
    }
}

// ───────────── 第三轮新增的格（前两轮都没量的形） ─────────────

use singlefs_core::mount::{unmount, ShadowLedger};

/// 环里同时有两个洞：实例 1 写到 L，实例 2 崩在写行根槽写之前（洞一），实例 3 写 gap 次再崩一次（洞二），实例 4 接着写，
/// 一直写到 N+1 先后落在洞一、洞二上。
#[test]
fn z2_two_holes_in_the_ring() {
    let ring = ring_length();
    for l in [ring + 1, ring + 5, 2 * ring - 1] {
        for gap in [1u64, 3, 6] {
            if !mine(l * 10 + gap) {
                continue;
            }
            let mut first = fresh_writer();
            overwrite_quietly(&mut first, l);
            let (crashed_once, hole_one) = crash_a_mount_before_its_row_root(&pool_of(&first.devices));
            let mut second = recorded_mount(&crashed_once).1.expect("写者");
            let target = txg_of(&second) + gap;
            overwrite_quietly(&mut second, target);
            let (crashed_twice, hole_two) = crash_a_mount_before_its_row_root(&pool_of(&second.devices));
            let mut third = recorded_mount(&crashed_twice).1.expect("写者");
            let hist = format!("L={l},gap={gap},H1={hole_one},H2={hole_two}");
            let last = hole_two + ring;
            sweep("Z2-two-holes", &hist, &mut third, last, true, &mut |next_txg, images| {
                if next_txg == hole_one + ring || next_txg == hole_two + ring {
                    scan("Z2-two-holes", &hist, "zero", &images.healthy, &images.zero);
                }
            });
        }
    }
}

/// 洞与抬 F 叠在一起：洞之后实例写 extra 次，正常卸载（F 抬到现行那一版），新实例接着写到 N+1 落在洞上。
#[test]
fn z2_hole_and_a_raised_floor() {
    let ring = ring_length();
    for l in [ring + 1, ring + 6] {
        for extra in [0u64, 2, 5] {
            if !mine(l * 10 + extra) {
                continue;
            }
            let mut first = fresh_writer();
            overwrite_quietly(&mut first, l);
            let (crashed, hole) = crash_a_mount_before_its_row_root(&pool_of(&first.devices));
            let mut second = recorded_mount(&crashed).1.expect("写者");
            let target = txg_of(&second) + extra;
            overwrite_quietly(&mut second, target);
            let parameters = parameters_of_the_ring_hole_pools();
            let mut current = PoolVersion::WithFile(second.current.clone());
            unmount(&parameters, &mut second.devices, &mut second.allocator, &mut current, ShadowLedger::On)
                .expect("正常卸载");
            let mut third = recorded_mount(&pool_of(&second.devices)).1.expect("写者");
            let hist = format!("L={l},extra={extra},H={hole}");
            let last = hole + ring;
            sweep("Z2-hole-raised-floor", &hist, &mut third, last, true, &mut |next_txg, images| {
                if next_txg == last {
                    scan("Z2-hole-raised-floor", &hist, "zero", &images.healthy, &images.zero);
                }
            });
        }
    }
}

/// 多个短命实例交替：每个实例写 k 次覆盖写就崩（不卸载），下一个实例可写挂载；每次挂载的写行与每次覆盖写都造五张镜像。
#[test]
fn z2_alternating_instances() {
    let ring = ring_length();
    for k in [1u64, 2, 4] {
        if !mine(k) {
            continue;
        }
        let first = fresh_writer();
        let mut image = pool_of(&first.devices);
        for round in 0..(3 * ring / (k + 3) + 3) {
            let (recording, writer) = recorded_mount(&image);
            let hist = format!("k={k},round={round}");
            rows_of("Z2-alt-row", &hist, &crash_images(&recording.before, &recording.writes, 0));
            let mut writer = writer.expect("写者");
            let target = txg_of(&writer) + k;
            sweep("Z2-alt-overwrite", &hist, &mut writer, target, true, &mut quiet);
            image = pool_of(&writer.devices);
        }
    }
}

/// p 的其余取值：格 F 的一个点上，N 那条根的 p 取 0 到 3S−1 与 255 全部。
#[test]
fn z2_every_pin_value_on_the_pinned_slot_cell() {
    let ring = ring_length();
    let first_lap_next = ring + 2;
    let mut first = fresh_writer();
    overwrite_quietly(&mut first, first_lap_next - 1);
    let (before, writes) = recorded_overwrite(&mut first);
    let damaged = crash_images(&before, &writes, 0).zero;
    let mut writer = recorded_mount(&damaged).1.expect("写者");
    let last = first_lap_next + ring;
    let hist = format!("N0plus1={first_lap_next}");
    sweep("Z2-pins", &hist, &mut writer, last, false, &mut |next_txg, images| {
        if next_txg == last {
            scan_with("Z2-pins", &hist, "zero", &images.healthy, &images.zero, true);
            scan_with("Z2-pins", &hist, "before_root", &images.healthy, &images.before_root, true);
        }
    });
}
