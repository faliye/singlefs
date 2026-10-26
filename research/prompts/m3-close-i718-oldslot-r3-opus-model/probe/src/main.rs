//! m3-close-i718-oldslot-r3 攻方腿原型：池级 checker 的 I-7.18 装了候选开关（`tools/walk-candidate-switch*.py`，第三轮另加丁乙与丁甲加第 0 代根）
//! 之后，在环洞池（S 从 `I718_SLOTS_PER_REGION` 取）上造第三轮要的几段历史，逐张镜像逐个候选打 I-7.18、I-7.19 的判定（`z2`、`z3`），
//! 并照一份独立于 walk.rs 开关的写法（`independent_verdicts`）从根槽字节另算今天与丁甲；`enumerate` 只算并（不超过 10⁶ 时）展开几条流。
//! 第二轮那一份（`research/prompts/m3-close-i718-oldslot-r2-opus-model/probe/src/main.rs`）拷来改写；只放在三方腿的仓副本里编、直接执行。
mod ring_hole_pools;

use ring_hole_pools::{
    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
    parameters_of_the_ring_hole_pools, pool_of, recording_of, slots_per_region_of_the_ring_hole_pools,
    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS,
    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
};
use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::{check_pool_image, set_i718_candidate_for_the_attack_probe, I718Candidate};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, unmount, Mounted};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 428;
const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: usize = 436;
const ROOT_SLOT_BYTES: usize = 512;

/// 今天（也是丙）、甲（第一轮）、乙字面（第一轮）、丁甲同实例（第二轮打穿的读法）、丁甲（第三轮正文）、丁甲加第 0 代根（本腿零轮）、丁乙（第三轮正文）。
const CANDIDATES: [(I718Candidate, &str); 7] = [
    (I718Candidate::CurrentOrBing, "today"),
    (I718Candidate::Jia, "jia"),
    (I718Candidate::YiLiteral, "yi_literal"),
    (I718Candidate::DingRoundTwoSameInstance, "ding_jia_same_instance"),
    (I718Candidate::DingRoundTwoAnyInstance, "ding_jia"),
    (I718Candidate::DingJiaGenerationZero, "ding_jia_gen0"),
    (I718Candidate::DingYiRoundThree, "ding_yi"),
];

fn ring_length() -> u64 {
    3 * slots_per_region_of_the_ring_hole_pools()
}

/// 一个写者进程：盘、分配器、现行那一版、实例代号、这次挂载的影子账（卸载要它）。
struct Writer {
    devices: RingHoleDevices,
    allocator: PoolAllocator,
    current: TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: singlefs_core::mount::ShadowLedger,
}

fn output_of(version: PoolVersion) -> TransactionOutput {
    match version {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => panic!("这一版没有文件"),
    }
}

fn empty_devices() -> RingHoleDevices {
    (0..2u32)
        .map(|number| (DeviceIdentity(number), SparseBlockDevice::new(DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, PhysicalBlockSizeInBytes(512))))
        .collect()
}

fn mkfs_image() -> MemoryPool {
    let mut devices = empty_devices();
    make_filesystem(&parameters_of_the_ring_hole_pools(), &mut devices).expect("mkfs");
    pool_of(&devices)
}

/// 在一张盘面上可写挂载；这一版没有文件就发第一个文件。交回写者。
fn writer_mounted_on(image: &MemoryPool) -> Writer {
    let parameters = parameters_of_the_ring_hole_pools();
    let mut devices = devices_of(image);
    let Mounted { output, mut allocator, current } = mount_writable(&parameters, &mut devices).expect("可写挂载");
    let instance = output.instance;
    let current = match current {
        PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => output,
        PoolVersion::WithoutFile(_) => {
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
        }
    };
    Writer { devices, allocator, current, instance, shadow_ledger: output.shadow_ledger }
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件。
fn fresh_writer() -> Writer {
    writer_mounted_on(&mkfs_image())
}

/// 在一张盘面上可写挂载，崩在这次挂载第 `ordinal` 次根槽写之前（之前的写全落）：交回崩溃后的盘面与那一次根槽写的 txg。
fn crash_a_mount_before_its_root_write(image: &MemoryPool, ordinal: usize) -> (MemoryPool, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    mount_writable(&parameters, &mut recording).expect("被截的那次挂载");
    let (writes, _) = writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    let cut = root_write_index(&writes, ordinal);
    let txg = RootRecord::parse_slot(writes[cut].bytes().expect("根槽写带字节"), &FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS)
        .expect("根槽写自证")
        .checkpoint_txg
        .0;
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..cut]);
    (crashed, txg)
}

/// 一次录下来的写：之前的盘面、写表、按屏障与 FUA 切的段。
struct Recorded {
    before: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
}

/// 在一张盘面上可写挂载，录下这次挂载的全部写（写行、暖机）。
fn recorded_mount(image: &MemoryPool) -> Recorded {
    let parameters = parameters_of_the_ring_hole_pools();
    let stream = SharedStream::retaining_contents();
    let mut recording = recording_of(image, &stream);
    mount_writable(&parameters, &mut recording).expect("可写挂载");
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
    Recorded { before: image.clone(), writes, segments }
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
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &fixed_geometry_of_the_ring_hole_pools());
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

fn txg_of(writer: &Writer) -> u64 {
    writer.current.root.checkpoint_txg.0
}

/// txg 那一槽的盘与偏移。
fn root_slot_of(txg: u64) -> (DeviceIdentity, u64) {
    let parameters = parameters_of_the_ring_hole_pools();
    let ring_slot = target_for_publish(CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    (device, slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0)
}

fn root_in_slot_of(image: &dyn ImageReader, txg: u64) -> Option<RootRecord> {
    let (device, offset) = root_slot_of(txg);
    RootRecord::parse_slot(&image.read(device.0, offset, ROOT_SLOT_BYTES)?, &FILESYSTEM_IDENTIFIER_OF_THE_RING_HOLE_POOLS)
}

/// 环里读得出、自证过的根（按槽取，槽值 0..3S），按 txg 排。
fn roots_of(image: &dyn ImageReader) -> Vec<RootRecord> {
    let mut roots: Vec<RootRecord> = (0..ring_length()).filter_map(|slot_value| root_in_slot_of(image, slot_value)).collect();
    roots.sort_unstable_by_key(|root| root.checkpoint_txg.0);
    roots
}

/// 一段写里第 `ordinal` 次根槽写的位置。
fn root_write_index(writes: &[RetainedWrite], ordinal: usize) -> usize {
    writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::RootRecordFua).nth(ordinal).expect("这一段的根槽写").0
}

/// 第 `ordinal` 次根槽写的四张：崩在它之前、它写成 0、照崩溃点重放的撕法撕（新旧不同那一截前一半新、后一半旧）、对半撕（前 256 字节新）。
fn crash_images_of_the_root_slot_write(before: &MemoryPool, writes: &[RetainedWrite], ordinal: usize) -> Vec<(&'static str, MemoryPool)> {
    let index = root_write_index(writes, ordinal);
    let root_write = &writes[index];
    let mut before_the_root = before.clone();
    before_the_root.apply_writes(&writes[..index]);
    let new_bytes = root_write.bytes().expect("根槽写带字节").to_vec();
    let old_bytes = singlefs_core::recovery::PoolReader::read(&before_the_root, root_write.device, root_write.offset, new_bytes.len()).expect("旧字节");
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
    let half: Vec<u8> = new_bytes[..ROOT_SLOT_BYTES / 2].iter().chain(old_bytes[ROOT_SLOT_BYTES / 2..].iter()).copied().collect();
    vec![
        ("before_root", before_the_root.clone()),
        ("zero", with_slot(vec![0u8; new_bytes.len()])),
        ("torn_enum", with_slot(torn)),
        ("torn_half", with_slot(half)),
    ]
}

/// 改一条根的 t 与 p（重封自证校验和）；只改根槽，不改那次发布的 journal 记录。
fn rewrite_threshold_of_root(image: &mut MemoryPool, txg: u64, threshold: u64, pinned: u8) {
    let (device, offset) = root_slot_of(txg);
    let mut bytes = image.read(device.0, offset, ROOT_SLOT_BYTES).expect("根槽读得出");
    bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8].copy_from_slice(&threshold.to_le_bytes());
    bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET] = pinned;
    let digest = wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_RECORD_CHECKSUM_OFFSET);
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
    image.devices.get_mut(&device).expect("那块盘").write(DeviceOffsetInBytes(offset), &bytes);
}

/// 把 txg 那一槽事后写成 0（读故障或写坏都用它造）。
fn zero_the_slot_of(image: &MemoryPool, txg: u64) -> MemoryPool {
    let mut zeroed = image.clone();
    let (device, offset) = root_slot_of(txg);
    zeroed.devices.get_mut(&device).expect("那块盘").write(DeviceOffsetInBytes(offset), &[0u8; ROOT_SLOT_BYTES]);
    zeroed
}

fn short_verdict(verdict: Option<&InvariantVerdict>) -> &'static str {
    match verdict {
        Some(InvariantVerdict::Holds) => "G",
        Some(InvariantVerdict::Violated(_)) => "R",
        Some(InvariantVerdict::NotApplicable(_)) => "NA",
        Some(_) => "other",
        None => "absent",
    }
}

/// 独立于 walk.rs 开关的另一份判法（Z3 复核用）：只读根槽字节，环里每条自证过的根都当有效（这几段历史里没有被抛弃的根，
/// 与 walk.rs 按实例表滤出来的不同时打 `independent_filter_differs`）。今天：t == max(F, min(环里全部根的 txg, p 的钉值))；
/// 丁甲：另在「下一槽读不出、有 txg = N+1 的记录（取 walk.rs 那一份扫出的布尔）、N+1 >= 3S」时，把 N+1−k·3S 逐个补进去现算。
fn independent_verdicts(image: &dyn ImageReader, record_any: bool) -> String {
    let roots = roots_of(image);
    let Some(newest) = roots.last() else { return "independent=empty".to_string() };
    let length = ring_length();
    let n = newest.checkpoint_txg.0;
    let floor = newest.rollback_floor.0;
    let t = newest.reclaim_threshold.0;
    let pin = u64::from(newest.pinned_ring_slot.0);
    let pinned = (pin < length).then_some(pin);
    let recompute = |extra: Option<u64>| {
        let lowest = roots.iter().map(|root| root.checkpoint_txg.0).chain(pinned).chain(extra).min().expect("至少有最新根");
        lowest.max(floor)
    };
    let today = t == recompute(None);
    let next_unreadable = root_in_slot_of(image, n + 1).is_none();
    let widened = next_unreadable
        && record_any
        && n + 1 >= length
        && (1..=(n + 1) / length).any(|laps| t == recompute(Some(n + 1 - laps * length)));
    format!("independent_today={} independent_ding_jia={}", if today { "G" } else { "R" }, if today || widened { "G" } else { "R" })
}

/// 一张镜像逐个候选的 I-7.18 与 I-7.19 判定，连同最新根、t、p、F、下一槽、有没有 N+1 的记录与今天判红的全部不变量。
/// `ding_yi_literal_i719`：丁乙照正文「照 I-7.18 条文里「不判」那一类的写法记」连 I-7.19 一起不判的那一读法（I-7.18 报不适用的那几张记 NA）。
fn verdict_line(image: &dyn ImageReader) -> String {
    let roots = roots_of(image);
    let Some(newest) = roots.last() else { return "ring=empty".to_string() };
    let n = newest.checkpoint_txg.0;
    let next_slot = root_in_slot_of(image, n + 1).map(|root| (root.checkpoint_txg.0, root.instance.0));
    let mut i718 = Vec::new();
    let mut i719 = Vec::new();
    let mut today_red = Vec::new();
    let (mut record_same, mut record_any) = (false, false);
    let mut ding_yi_i718_not_applicable = false;
    for (candidate, name) in CANDIDATES {
        set_i718_candidate_for_the_attack_probe(candidate);
        let verdicts = check_pool_image(image);
        let of = |invariant: &str| short_verdict(verdicts.iter().find(|(i, _)| *i == invariant).map(|(_, v)| v));
        i718.push(format!("{name}={}", of("I-7.18")));
        i719.push(format!("{name}={}", of("I-7.19")));
        if candidate == I718Candidate::DingYiRoundThree {
            ding_yi_i718_not_applicable = of("I-7.18") == "NA";
        }
        if candidate == I718Candidate::CurrentOrBing {
            today_red = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(i, _)| *i).collect();
            record_same = singlefs_checker::walk::I718_NEXT_RECORD_PRESENT.with(std::cell::Cell::get);
            record_any = singlefs_checker::walk::I718_NEXT_RECORD_PRESENT_OF_ANY_INSTANCE.with(std::cell::Cell::get);
        }
    }
    set_i718_candidate_for_the_attack_probe(I718Candidate::CurrentOrBing);
    let ding_yi_i719 = i719.last().expect("丁乙那一列").clone();
    let literal = if ding_yi_i718_not_applicable { "NA" } else { ding_yi_i719.rsplit('=').next().expect("值") };
    format!(
        "newest=({n}, {}) t={} p={} F={} next_slot={next_slot:?} rec=same:{record_same},any:{record_any} | I-7.18 {} | I-7.19 {} ding_yi_literal={literal} | {} | today_red={today_red:?}",
        newest.instance.0,
        newest.reclaim_threshold.0,
        newest.pinned_ring_slot.0,
        newest.rollback_floor.0,
        i718.join(" "),
        i719.join(" "),
        independent_verdicts(image, record_any),
    )
}

fn report(cell: &str, image_name: &str, image: &dyn ImageReader) {
    println!("S={} cell={cell} image={image_name} {}", slots_per_region_of_the_ring_hole_pools(), verdict_line(image));
}

/// 一次覆盖写的根槽写坏：四张镜像逐张打。
fn report_failed_overwrite(cell: &str, writer: &mut Writer) {
    let recorded = recorded_overwrite(writer);
    for (name, image) in crash_images_of_the_root_slot_write(&recorded.before, &recorded.writes, 0) {
        report(cell, name, &image);
    }
}

/// 在写者现在的盘面上试一次下一次覆盖写（写坏），写者不前进：交回四张镜像。
fn failed_overwrite_images(writer: &Writer) -> Vec<(&'static str, MemoryPool)> {
    let mut probe = Writer {
        devices: devices_of(&pool_of(&writer.devices)),
        allocator: writer.allocator.clone(),
        current: writer.current.clone(),
        instance: writer.instance,
        shadow_ledger: writer.shadow_ledger.clone(),
    };
    let recorded = recorded_overwrite(&mut probe);
    crash_images_of_the_root_slot_write(&recorded.before, &recorded.writes, 0)
}

fn ring_txgs(image: &dyn ImageReader) -> Vec<(u64, u32)> {
    roots_of(image).iter().map(|root| (root.checkpoint_txg.0, root.instance.0)).collect()
}

/// 每到一个 txg 都试一次下一次覆盖写写坏（`keep(N)` 为真的那几个 N 才打出来），写者照常前进到 `last`。
fn sweep_failed_overwrites(cell: &str, writer: &mut Writer, last: u64, keep: &dyn Fn(u64) -> bool) {
    // 迭代上界：txg 逐次加一，到 `last` 为止。
    while txg_of(writer) < last {
        let n = txg_of(writer);
        if keep(n) {
            for (name, image) in failed_overwrite_images(writer) {
                report(&format!("{cell}-N{n}"), name, &image);
            }
        }
        recorded_overwrite(writer);
    }
}

/// Z2-第 0 代根：只做过 mkfs 的池上第一次可写挂载崩在第 `ordinal` 次根槽写之前（写行 txg 1 或暖机 txg 2），
/// 区域 1 或 2 的槽 0 里 mkfs 的第 0 代根（txg 0）就留过第一圈；再挂、建文件、覆盖写到它那一槽被下一次发布写坏。对照：同样的写没有那次崩溃。
fn z2_generation_zero() {
    let length = ring_length();
    for ordinal in [0usize, 1] {
        let (crashed, hole_txg) = crash_a_mount_before_its_root_write(&mkfs_image(), ordinal);
        println!("S={} cell=Z2-gen0-crash{ordinal} hole_txg={hole_txg} ring_after_crash={:?}", slots_per_region_of_the_ring_hole_pools(), ring_txgs(&crashed));
        let mut writer = writer_mounted_on(&crashed);
        println!("S={} cell=Z2-gen0-crash{ordinal} after_mount txg={} ring={:?}", slots_per_region_of_the_ring_hole_pools(), txg_of(&writer), ring_txgs(&pool_of(&writer.devices)));
        sweep_failed_overwrites(&format!("Z2-gen0-crash{ordinal}"), &mut writer, hole_txg + length + 1, &|n| n + 3 >= length && n <= hole_txg + length);
    }
    let mut control = fresh_writer();
    sweep_failed_overwrites("Z2-gen0-control", &mut control, length + 3, &|n| n + 3 >= length && n <= length + 2);
}

/// Z2-几个洞：连着两次挂载都崩在写行根槽之前（两个洞挨着），或隔几版各崩一次（两个洞隔开），再挂、写过两圈，每个 N 试一次写坏。
fn z2_two_holes() {
    let length = ring_length();
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, length + 2);
    let (first, first_hole) = crash_a_mount_before_its_root_write(&pool_of(&writer.devices), 0);
    let (second, second_hole) = crash_a_mount_before_its_root_write(&first, 0);
    let mut adjacent = writer_mounted_on(&second);
    println!("S={} cell=Z2-holes-adjacent holes=[{first_hole}, {second_hole}] ring={:?}", slots_per_region_of_the_ring_hole_pools(), ring_txgs(&pool_of(&adjacent.devices)));
    let start = txg_of(&adjacent);
    sweep_failed_overwrites("Z2-holes-adjacent", &mut adjacent, start + 2 * length, &|_| true);

    let mut writer = fresh_writer();
    overwrite_through(&mut writer, length + 2);
    let (first, first_hole) = crash_a_mount_before_its_root_write(&pool_of(&writer.devices), 0);
    let mut middle = writer_mounted_on(&first);
    let target = txg_of(&middle) + 3;
    overwrite_through(&mut middle, target);
    let (second, second_hole) = crash_a_mount_before_its_root_write(&pool_of(&middle.devices), 0);
    let mut apart = writer_mounted_on(&second);
    println!("S={} cell=Z2-holes-apart holes=[{first_hole}, {second_hole}] ring={:?}", slots_per_region_of_the_ring_hole_pools(), ring_txgs(&pool_of(&apart.devices)));
    let start = txg_of(&apart);
    sweep_failed_overwrites("Z2-holes-apart", &mut apart, start + 2 * length, &|_| true);
}

/// Z2-洞与抬 F：洞之后的实例写几版、正常卸载（推空发布把 F 抬到现行那一版），再挂、写到洞那一槽之后，每个 N 试一次写坏。
fn z2_hole_and_raised_floor() {
    let length = ring_length();
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, length + 2);
    let (crashed, hole) = crash_a_mount_before_its_root_write(&pool_of(&writer.devices), 0);
    let mut after_hole = writer_mounted_on(&crashed);
    let target = txg_of(&after_hole) + 2;
    overwrite_through(&mut after_hole, target);
    let parameters = parameters_of_the_ring_hole_pools();
    let mut current = PoolVersion::WithFile(after_hole.current.clone());
    let unmounted = unmount(&parameters, &mut after_hole.devices, &mut after_hole.allocator, &mut current, after_hole.shadow_ledger.clone())
        .expect("正常卸载");
    let image = pool_of(&after_hole.devices);
    println!(
        "S={} cell=Z2-hole-floor hole={hole} unmounted={} ring={:?} floor_of_newest={}",
        slots_per_region_of_the_ring_hole_pools(),
        matches!(unmounted, singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(_)),
        ring_txgs(&image),
        roots_of(&image).last().map_or(0, |root| root.rollback_floor.0)
    );
    let mut raised = writer_mounted_on(&image);
    let start = txg_of(&raised);
    sweep_failed_overwrites("Z2-hole-floor", &mut raised, start + length + 2, &|_| true);
}

/// Z2-多实例交替：每个实例只写 1、2、3 版就崩（不卸载），下一个实例接着挂；环里住着好几个实例的根之后，
/// 每个 N 试一次覆盖写写坏，再在几个点上试下一次挂载的写行、第一次暖机写坏。
fn z2_alternating_instances() {
    let length = ring_length();
    let mut writer = fresh_writer();
    let mut round = 0u64;
    // 迭代上界：每轮至少前进一版（挂载另发写行与暖机），到两圈为止。
    while txg_of(&writer) < 2 * length {
        let target = txg_of(&writer) + round % 3 + 1;
        overwrite_through(&mut writer, target);
        writer = writer_mounted_on(&pool_of(&writer.devices));
        round += 1;
    }
    println!("S={} cell=Z2-alternating rounds={round} ring={:?}", slots_per_region_of_the_ring_hole_pools(), ring_txgs(&pool_of(&writer.devices)));
    for _ in 0..length {
        let n = txg_of(&writer);
        for (name, image) in failed_overwrite_images(&writer) {
            report(&format!("Z2-alternating-overwrite-N{n}"), name, &image);
        }
        if round % 3 == 0 {
            let mount = recorded_mount(&pool_of(&writer.devices));
            for ordinal in 0..2 {
                for (name, image) in crash_images_of_the_root_slot_write(&mount.before, &mount.writes, ordinal) {
                    report(&format!("Z2-alternating-mount-N{n}-root{ordinal}"), name, &image);
                }
            }
        }
        let target = n + round % 2 + 1;
        overwrite_through(&mut writer, target);
        writer = writer_mounted_on(&pool_of(&writer.devices));
        round += 1;
    }
}

/// Z1-丁乙与 I-7.19：钉槽轮到这个实例写掉之后写者没解钉（I-7.19 条文点名要判出的那一类），t 照留着的钉算；
/// 再下一次发布写坏。打 N 做完那张与写坏那几张，各候选的 I-7.18、I-7.19。
fn z1_pinned_slot_written_and_not_unpinned() {
    let length = ring_length();
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, length + 2);
    let failed_txg = length + 3;
    let zero = failed_overwrite_images(&writer).into_iter().find(|(name, _)| *name == "zero").expect("写成 0 那张").1;
    let mut pinned = writer_mounted_on(&zero);
    overwrite_through(&mut pinned, failed_txg + length - 1);
    let carrying = roots_of(&pool_of(&pinned.devices)).last().map(|root| (root.reclaim_threshold.0, root.pinned_ring_slot.0)).expect("带钉的根");
    overwrite_through(&mut pinned, failed_txg + length);
    let newest = txg_of(&pinned);
    let honest = roots_of(&pool_of(&pinned.devices)).last().map(|root| (root.reclaim_threshold.0, root.pinned_ring_slot.0)).expect("最新根");
    println!("S={} cell=Z1-not-unpinned carrying_before={carrying:?} honest_after_writing_the_pinned_slot={honest:?}", slots_per_region_of_the_ring_hole_pools());
    let bad_threshold = carrying.0.min(honest.0);
    let mut healthy = pool_of(&pinned.devices);
    report("Z1-not-unpinned-honest", "healthy_n", &healthy);
    rewrite_threshold_of_root(&mut healthy, newest, bad_threshold, carrying.1);
    report("Z1-not-unpinned-bad", "healthy_n", &healthy);
    for (name, mut image) in failed_overwrite_images(&pinned) {
        report("Z1-not-unpinned-honest", name, &image);
        rewrite_threshold_of_root(&mut image, newest, bad_threshold, carrying.1);
        report("Z1-not-unpinned-bad", name, &image);
    }
}

/// Z2-p 的其余取值：满环无洞、N = 2·3S+1，最新根的 p 换成每一个槽值与 255，t 取真值、按这个 p 在健康镜像上现算的值、F；
/// 健康镜像与写坏（写 0、枚举撕法）镜像各打一遍。
fn z2_pinned_values() {
    let length = ring_length();
    let newest = 2 * length + 1;
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, newest);
    let healthy = pool_of(&writer.devices);
    let root = roots_of(&healthy).last().cloned().expect("最新根");
    let failed: Vec<(&'static str, MemoryPool)> =
        failed_overwrite_images(&writer).into_iter().filter(|(name, _)| matches!(*name, "zero" | "torn_enum")).collect();
    let ring_min = roots_of(&healthy).first().expect("环里有根").checkpoint_txg.0;
    for pin in (0..length).chain(std::iter::once(255)) {
        let consistent = if pin < length { ring_min.min(pin) } else { ring_min }.max(root.rollback_floor.0);
        let mut thresholds = vec![root.reclaim_threshold.0, consistent, root.rollback_floor.0];
        thresholds.sort_unstable();
        thresholds.dedup();
        for threshold in thresholds {
            let pin_byte = u8::try_from(pin).expect("一个字节");
            let cell = format!("Z2-p-scan-p{pin}-t{threshold}");
            let mut image = healthy.clone();
            rewrite_threshold_of_root(&mut image, newest, threshold, pin_byte);
            report(&cell, "healthy_n", &image);
            for (name, base) in &failed {
                let mut image = base.clone();
                rewrite_threshold_of_root(&mut image, newest, threshold, pin_byte);
                report(&cell, name, &image);
            }
        }
    }
}

/// 环洞：实例 1 写到 `base_last`，下一个实例的可写挂载崩在写行那次根槽写之前，再下一个实例可写挂载。
fn writer_after_a_hole(base_last: u64) -> (Writer, u64) {
    let mut writer = fresh_writer();
    overwrite_through(&mut writer, base_last);
    let (crashed, row_txg) = crash_a_mount_before_its_root_write(&pool_of(&writer.devices), 0);
    (writer_mounted_on(&crashed), row_txg)
}

/// Z3-第一轮格 B：L 放开扫一圈，洞之后写到 N = 写行 + 3S − 1，N+1 落在洞上写坏。
fn z3_cell_b() {
    let length = ring_length();
    for base_last in length..2 * length {
        let (mut writer, row_txg) = writer_after_a_hole(base_last);
        overwrite_through(&mut writer, row_txg + length - 1);
        report_failed_overwrite(&format!("Z3-B-L{base_last}"), &mut writer);
    }
}

/// Z3-第一轮格 F：L 放开扫一圈，L+1 那次根槽写成 0，下一个实例挂上（那一槽钉住）、写到 N = L + 3S（下一槽正是钉住那一槽）；
/// N 做完那张上合法、t 写成 F、p 写 255 三份。
fn z3_cell_f() {
    let length = ring_length();
    for base_last in length..2 * length {
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, base_last);
        let zero = failed_overwrite_images(&writer).into_iter().find(|(name, _)| *name == "zero").expect("写成 0 那张").1;
        let mut pinned = writer_mounted_on(&zero);
        overwrite_through(&mut pinned, base_last + length);
        let healthy = pool_of(&pinned.devices);
        let root = roots_of(&healthy).last().cloned().expect("最新根");
        let n = root.checkpoint_txg.0;
        report(&format!("Z3-F-L{base_last}-legit"), "healthy_n", &healthy);
        let mut bad = healthy.clone();
        rewrite_threshold_of_root(&mut bad, n, root.rollback_floor.0, root.pinned_ring_slot.0);
        report(&format!("Z3-F-L{base_last}-bad_t_as_floor"), "healthy_n", &bad);
        let mut bad = healthy.clone();
        rewrite_threshold_of_root(&mut bad, n, root.reclaim_threshold.0, 255);
        report(&format!("Z3-F-L{base_last}-bad_p_255"), "healthy_n", &bad);
    }
}

/// Z3-第二轮格 G：满环之后另一实例可写挂载，写行那次根槽写坏；N 取 3S+2、6S+1、6S+4（第二轮辩方 S = 4 取的 14、25、28 同式）。
fn z3_cell_g() {
    let slots = slots_per_region_of_the_ring_hole_pools();
    for newest in [3 * slots + 2, 6 * slots + 1, 6 * slots + 4] {
        let mut writer = fresh_writer();
        overwrite_through(&mut writer, newest);
        let mount = recorded_mount(&pool_of(&writer.devices));
        for (name, image) in crash_images_of_the_root_slot_write(&mount.before, &mount.writes, 0) {
            report(&format!("Z3-G-N{newest}"), name, &image);
        }
    }
}

/// Z3-第二轮 BR：新池一路覆盖写，N 从 3S−1 到 6S，每个 N 把最旧根那一槽（N+1−3S，正是下一槽）事后写成 0。
fn z3_cell_br() {
    let length = ring_length();
    let mut writer = fresh_writer();
    for newest in length - 1..=2 * length {
        overwrite_through(&mut writer, newest);
        report(&format!("Z3-BR-N{newest}"), "oldest_root_unreadable", &zero_the_slot_of(&pool_of(&writer.devices), newest + 1 - length));
    }
}

/// 第三轮新造的几条流各算一次全量崩溃状态数（`crash_state_count_with_torn_in_place_overwrites`，每一段都展开）：
/// 只算数，不判（判那一步要么超过 10⁶、要么走不到「下一槽读不出」这一态，见报告「这条腿自己的限度」）。
fn enumerate_counts() {
    use singlefs_checker_tier::crash::{crash_state_count_with_torn_in_place_overwrites, full_expansion};
    let length = ring_length();
    let count = |name: &str, recorded: &Recorded| {
        let states = crash_state_count_with_torn_in_place_overwrites(&recorded.before, &recorded.writes, &recorded.segments, &full_expansion);
        println!("S={} stream={name} writes={} segments={} states={states}", slots_per_region_of_the_ring_hole_pools(), recorded.writes.len(), recorded.segments.len());
    };
    count("first-mount-on-a-mkfs-pool", &recorded_mount(&mkfs_image()));
    let (crashed, hole_txg) = crash_a_mount_before_its_root_write(&mkfs_image(), 0);
    count("mount-after-the-first-mount-crashed-before-its-row-root", &recorded_mount(&crashed));
    let mut writer = writer_mounted_on(&crashed);
    overwrite_through(&mut writer, hole_txg + length - 1);
    count("overwrite-onto-the-slot-generation-zero-survived-in", &recorded_overwrite(&mut writer));
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("z1") => z1_pinned_slot_written_and_not_unpinned(),
        Some("z2-gen0") => z2_generation_zero(),
        Some("z2-holes") => z2_two_holes(),
        Some("z2-floor") => z2_hole_and_raised_floor(),
        Some("z2-alternating") => z2_alternating_instances(),
        Some("z2-p") => z2_pinned_values(),
        Some("z3-b") => z3_cell_b(),
        Some("z3-f") => z3_cell_f(),
        Some("z3-g") => z3_cell_g(),
        Some("z3-br") => z3_cell_br(),
        Some("counts") => enumerate_counts(),
        _ => panic!("用法：i718-round-three-probe z1|z2-gen0|z2-holes|z2-floor|z2-alternating|z2-p|z3-b|z3-f|z3-g|z3-br|counts（S 从 I718_SLOTS_PER_REGION 取）"),
    }
}
