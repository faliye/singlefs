//! m3-close-c656-r3 辩方原型（只在仓副本里编、经内存包装直接执行编出来的二进制；不调 checker 档测试）。
//! 与 m3-close-c656-r2 攻方原型的区别：mkfs 一行不改（`make_filesystem` 就是入库那一份），
//! 乙与甲七的写序由「入库 mkfs 的录制流重排」得到（写的字节一个不变，只换次序、插屏障、加一段清单元区的写零）；
//! 仓副本里唯一的改动是崩溃点重放逐态展开函数的可见性（`tools/patch_copy.py`）。
//! 用法：
//!   c656-r3-defense-probe streams <历史>                    四种写序各一行：段序列、操作数、闭式、带撕裂全量、做完的盘面是否逐字节等于今天的
//!   c656-r3-defense-probe run <历史> <写序> [起] [止]       逐态：择系统配置、择根、恢复读回、池级 checker、只读挂载、按择到那份的参数可写挂载；整池拒的态另试三种守卫与再跑 mkfs
//!   c656-r3-defense-probe final <历史> <写序>               mkfs 全落之后的同一套判
//!   c656-r3-defense-probe lag <次数>                         盘 1 的写全丢（崩溃态：盘 0 落了、盘 1 没落），连续取号几次后两盘系统配置世代号差多少、还挂不挂得上
//!   历史：same（旧池 fsid 与新池相同）、prefix（前 8 字节相同、后 8 字节不同）、distinct（前 8 字节不同）、edge8（只有第 8 字节不同）
//!   写序：today、yi（乙）、yi-unitclear（乙 + 甲七，作废屏障之后清单元区）、yi-unitclear-early（甲七与作废同段）
//!   环境变量 PROBE_RERUN=0：整池拒的态上不再跑 mkfs（省挂钟）
#![allow(dead_code, clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::plain_devices_on;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, no_segment_slices,
    visit_crash_state_range_of_the_read_back_flow_with_slices,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::root_ring::{region_start, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments, FixedGeometry};
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, RetainedOperation, SharedStream};
use std::collections::BTreeMap;
use std::sync::Mutex;

const STATE_LIMIT_PER_RUN: u64 = 1_000_000;
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const DEVICE_SLOTS: u64 = 1408 + 1007 + 384;
const DEVICE_BYTES: u64 = DEVICE_SLOTS * 16384;
const FSID_NEW: [u8; 16] = *b"c656-r3-new-pool";
const FSID_PREFIX: [u8; 16] = *b"c656-r3-old-pool";
const FSID_DISTINCT: [u8; 16] = *b"OLDPOOL!distinct";
const FSID_EDGE8: [u8; 16] = *b"c656-r3Xnew-pool";
const WRITE_TIME: u64 = 1_788_000_000;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn threads() -> u64 {
    std::env::var("SINGLEFS_CRASH_POINT_REPLAY_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(1).max(1)
}

fn variant(debug: &str) -> String {
    debug.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("").to_string()
}

fn parameters(fsid: [u8; 16], io_min: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: fsid,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: io_min,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(io_min),
            journal_ring_bytes: JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn fresh_device() -> SparseBlockDevice {
    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn clone_device(device: &SparseBlockDevice) -> SparseBlockDevice {
    let mut copy = fresh_device();
    copy.image = device.image.clone();
    copy
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: DEVICE_BYTES }
}

fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

fn content(seed: u64) -> Vec<u8> {
    (0..3000u64).map(|i| u8::try_from((i * 137 + seed * 11 + 3) % 251).expect("<256")).collect()
}

/// 旧池的用户动作：mkfs + 取号 + 暖机 + 第一个文件 + (u − 2) 次覆盖写。交回每一版的内容与最后一版的根 (实例, txg)。
fn make_and_use(devices: &mut Devices, parameters: &MakeFilesystemParameters, usage: u32, seed: u64) -> (Vec<Vec<u8>>, (u32, u64)) {
    let genesis = make_filesystem(parameters, devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(parameters, devices, &genesis);
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first = content(seed);
    let mut previous = publish_first_file(&mut writer, &mut allocator, warmed.roots.last().expect("两代根"), FirstFile { content: &first, write_time_seconds: WRITE_TIME }, instance, &warmed.last_record_bytes).expect("第一个文件");
    let mut versions = vec![first];
    for k in 0..usage - 2 {
        let c = content(seed + 1 + u64::from(k));
        previous = publish_overwrite(&mut writer, &mut allocator, &previous, FirstFile { content: &c, write_time_seconds: WRITE_TIME + 1 + u64::from(k) }, instance).expect("覆盖写");
        versions.push(c);
    }
    (versions, (previous.root.instance.0, previous.root.checkpoint_txg.0))
}

/// 入库 mkfs 今天的录制流（25 步）：设备 0 的 [0, 根环基址)、根环三区、journal 环清零（0..=4），设备 1 同（5..=9），
/// 四个单元写（10..=13），两道屏障（14、15），三条根 FUA（16..=18），四个系统配置槽（19..=22），两道屏障（23、24）。
fn record_today(devices: &Devices, new_parameters: &MakeFilesystemParameters) -> (Vec<RetainedOperation>, u64) {
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
        .iter()
        .map(|(i, d)| (*i, RecordingBlockDevice::with_shared_stream(*i, clone_device(d), stream.clone())))
        .collect();
    let output = make_filesystem(new_parameters, &mut recorded).expect("新 mkfs");
    let unit_area_start_slot = output.unit_area_start.slot().0;
    let ops = stream.retained_operations();
    assert_eq!(ops.len(), 25, "今天的 mkfs 录制流 25 步");
    let kinds: Vec<&str> = ops.iter().map(|o| o.operation.kind.name()).collect();
    let expected = [
        ("write_zeroes", 0u32), ("write_zeroes", 0), ("write_zeroes", 0), ("write_zeroes", 0), ("write_zeroes", 0),
        ("write_zeroes", 1), ("write_zeroes", 1), ("write_zeroes", 1), ("write_zeroes", 1), ("write_zeroes", 1),
        ("write", 0), ("write", 0), ("write", 1), ("write", 1), ("barrier", 0), ("barrier", 1),
        ("write_fua", 0), ("write_fua", 1), ("write_fua", 0),
        ("write", 0), ("write", 0), ("write", 1), ("write", 1), ("barrier", 0), ("barrier", 1),
    ];
    // 单元写与屏障的设备次序：10、11 在设备 0，12、13 在设备 1
    for (i, (kind, device)) in expected.iter().enumerate() {
        assert_eq!(kinds[i], *kind, "第 {i} 步种类");
        assert_eq!(ops[i].operation.device.0, *device, "第 {i} 步设备");
    }
    assert_eq!(ops[0].operation.offset.0, 0, "设备 0 第 0 步清 [0, 根环基址)");
    assert_eq!(ops[5].operation.offset.0, 0, "设备 1 第 5 步清 [0, 根环基址)");
    (ops, unit_area_start_slot)
}

/// 单元区里第 0 版两个单元之后到盘尾（按槽对齐）的写零，设备 `device` 上一步。
fn unit_area_clear(template: &RetainedOperation, device: u32, unit_area_start_slot: u64) -> RetainedOperation {
    // 实例表单元占 2 槽、树表单元第 0 版 1 槽，清零从它们之后起
    let start = (unit_area_start_slot + 3) * SLOT_BYTES;
    let end = DEVICE_BYTES / SLOT_BYTES * SLOT_BYTES;
    let mut op = template.clone();
    op.operation.device = DeviceIdentity(device);
    op.operation.offset = DeviceOffsetInBytes(start);
    op.operation.length = end - start;
    op
}

/// 把今天的录制流重排成各写序。乙 = 两盘各清 [0, 根环基址) → 两盘各一道屏障 → 两盘清根环与 journal 环 → 四个单元写 → 两道屏障 → 根 FUA → 系统配置 → 两道屏障，
/// 与 m3-close-c656-r2 攻方补丁的 `InvalidateFirst` 发的次序相同。
fn reorder(today: &[RetainedOperation], order: &str, unit_area_start_slot: u64) -> Vec<RetainedOperation> {
    let t = |i: usize| today[i].clone();
    let range = |a: usize, b: usize| (a..=b).map(|i| today[i].clone());
    let mut out: Vec<RetainedOperation> = Vec::new();
    match order {
        "today" => out.extend(today.iter().cloned()),
        "yi" => {
            out.extend([t(0), t(5), t(14), t(15)]);
            out.extend(range(1, 4));
            out.extend(range(6, 9));
            out.extend(range(10, 13));
            out.extend([t(14), t(15)]);
            out.extend(range(16, 24));
        }
        "yi-unitclear" => {
            out.extend([t(0), t(5), t(14), t(15)]);
            out.extend(range(1, 4));
            out.push(unit_area_clear(&today[0], 0, unit_area_start_slot));
            out.extend(range(6, 9));
            out.push(unit_area_clear(&today[5], 1, unit_area_start_slot));
            out.extend(range(10, 13));
            out.extend([t(14), t(15)]);
            out.extend(range(16, 24));
        }
        "yi-unitclear-early" => {
            out.extend([t(0), unit_area_clear(&today[0], 0, unit_area_start_slot), t(5), unit_area_clear(&today[5], 1, unit_area_start_slot), t(14), t(15)]);
            out.extend(range(1, 4));
            out.extend(range(6, 9));
            out.extend(range(10, 13));
            out.extend([t(14), t(15)]);
            out.extend(range(16, 24));
        }
        other => panic!("未知写序 {other}"),
    }
    out
}

struct History {
    name: String,
    order_name: String,
    base: MemoryPool,
    ops: Vec<RetainedOperation>,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    recorded: usize,
    old_parameters: MakeFilesystemParameters,
    old_versions: Vec<Vec<u8>>,
    old_last_root: (u32, u64),
    new_parameters: MakeFilesystemParameters,
    unit_area_start_slot: u64,
}

fn fixed_geometry(p: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: p.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: p.geometry.journal_ring_bytes,
        root_ring_slots_per_region: p.geometry.root_ring_slots_per_region,
    }
}

fn old_pool(name: &str) -> (Devices, MakeFilesystemParameters, Vec<Vec<u8>>, (u32, u64)) {
    let fsid = match name { "same" => FSID_NEW, "prefix" => FSID_PREFIX, "distinct" => FSID_DISTINCT, "edge8" => FSID_EDGE8, other => panic!("未知历史 {other}") };
    let old_parameters = parameters(fsid, 512);
    let mut old: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
    let (versions, last) = make_and_use(&mut old, &old_parameters, 5, 7);
    (old, old_parameters, versions, last)
}

fn history(name: &str, order_name: &str) -> History {
    let new_parameters = parameters(FSID_NEW, 512);
    let (devices, old_parameters, old_versions, old_last_root) = old_pool(name);
    let base = image_of(&devices);
    let (today, unit_area_start_slot) = record_today(&devices, &new_parameters);
    let ops = reorder(&today, order_name, unit_area_start_slot);
    let (writes, segments) = writes_and_segments(&ops, &fixed_geometry(&new_parameters));
    let recorded = writes.len();
    History { name: name.into(), order_name: order_name.into(), base, ops, writes, segments, recorded, old_parameters, old_versions, old_last_root, new_parameters, unit_area_start_slot }
}

fn content_class(h: &History, root: (u32, u64), bytes: &[u8]) -> String {
    let versions = &h.old_versions;
    let last_root = h.old_last_root;
    let last = versions.last();
    match last {
        Some(last) if bytes == last.as_slice() && root == last_root => "old_last_same_root".into(),
        Some(last) if bytes == last.as_slice() => "old_last_other_root".into(),
        _ if versions.iter().any(|c| c.as_slice() == bytes) => "old_older_version".into(),
        _ => "other_content".into(),
    }
}

fn recover_label(h: &History, pool: &MemoryPool) -> String {
    match recover(pool, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead(i{}t{}):{}", root.0 .0, root.1 .0, content_class(h, (root.0 .0, root.1 .0), &content)),
        RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(&format!("{failure:?}"))),
    }
}

fn reds(image: &MemoryPool) -> Vec<String> {
    let mut names: Vec<String> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then(|| invariant.to_string()))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// 守卫一（r2 攻方的最强形）：每盘 [0, 根环基址) 里逐 4096 字节试解系统配置，有一份自证过、fsid 前 8 字节与要用的相同就拒。
fn guard_system_configuration(pool: &MemoryPool, fsid: &[u8; 16]) -> bool {
    let base = region_start(0).0;
    for device in pool.devices.keys() {
        let mut offset = 0;
        while offset + 4096 <= base {
            if let Some(bytes) = PoolReader::read(pool, *device, DeviceOffsetInBytes(offset), 4096) {
                if bytes.iter().any(|b| *b != 0) {
                    if let Ok(c) = SystemConfiguration::parse_slot(&bytes) {
                        if c.immutable.filesystem_identifier[..8] == fsid[..8] {
                            return true;
                        }
                    }
                }
            }
            offset += 4096;
        }
    }
    false
}

/// 单元头里的 fsid 低 8 字节（魔数 SFSU，码 1：偏移 83；码 2：60 + 2 × bytes[51]；码 3：81；与 checker 的 `unit_write_order_instance` 同一张偏移表，不核头校验和）。
fn unit_header_fsid_low(bytes: &[u8]) -> Option<[u8; 8]> {
    if bytes.len() < 120 || &bytes[..4] != b"SFSU" {
        return None;
    }
    let offset = match bytes[6] {
        1 => 83,
        2 => 60 + 2 * usize::from(bytes[51]),
        3 => 81,
        _ => return None,
    };
    bytes.get(offset..offset + 8).map(|s| s.try_into().expect("8 字节"))
}

/// 守卫二：只读 mkfs 自己写第 0 版两个单元的那几个槽的头（单元区起点起 3 槽）。
/// 守卫三：读整个单元区每一槽的头。
fn guard_unit_headers(pool: &MemoryPool, fsid: &[u8; 16], unit_area_start_slot: u64, only_the_slots_mkfs_writes: bool) -> bool {
    let last_slot = if only_the_slots_mkfs_writes { unit_area_start_slot + 3 } else { DEVICE_BYTES / SLOT_BYTES };
    for device in pool.devices.keys() {
        for slot in unit_area_start_slot..last_slot {
            if let Some(bytes) = PoolReader::read(pool, *device, DeviceOffsetInBytes(slot * SLOT_BYTES), 512) {
                if let Some(low) = unit_header_fsid_low(&bytes) {
                    if low == fsid[..8] {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn guards_text(pool: &MemoryPool, fsid: &[u8; 16], unit_area_start_slot: u64) -> String {
    let flag = |b: bool| if b { "refuse" } else { "allow" };
    format!(
        "g_sysconf={} g_units_fixed={} g_units_all={}",
        flag(guard_system_configuration(pool, fsid)),
        flag(guard_unit_headers(pool, fsid, unit_area_start_slot, true)),
        flag(guard_unit_headers(pool, fsid, unit_area_start_slot, false))
    )
}

/// 整池拒之后用户接着做什么：照今天的写序再跑 mkfs，fsid 取新池的与旧池的两种；先看三种守卫，再看新池恢复与池级 checker。
fn rerun_after_refusal(h: &History, pool: &MemoryPool) -> String {
    let mut parts = Vec::new();
    let mut fsids = vec![("new", h.new_parameters.filesystem_identifier)];
    if h.old_parameters.filesystem_identifier != h.new_parameters.filesystem_identifier {
        fsids.push(("old", h.old_parameters.filesystem_identifier));
    }
    for (tag, fsid) in fsids {
        let guards = guards_text(pool, &fsid, h.unit_area_start_slot);
        let outcome = if std::env::var("PROBE_RERUN").map_or(true, |v| v != "0") {
            let p = MakeFilesystemParameters { filesystem_identifier: fsid, ..h.new_parameters };
            let mut devices: Devices = plain_devices_on(pool);
            match make_filesystem(&p, &mut devices) {
                Ok(_) => {
                    let after = image_of(&devices);
                    format!("{}|{:?}", recover_label(h, &after), reds(&after))
                }
                Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
            }
        } else {
            "rerun_not_tried".to_string()
        };
        parts.push(format!("rerun_{tag}[{guards}]={outcome}"));
    }
    parts.join(" ")
}

fn describe_cut(h: &History, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String, bool) {
    let first_withheld = (0..h.recorded).find(|i| !persisted[*i]);
    let phase = match first_withheld {
        Some(w) => format!("seg{}", h.segments.iter().position(|s| s.contains(&w)).unwrap_or(h.segments.len())),
        None => "all".to_string(),
    };
    let withheld: Vec<String> = (0..h.recorded).filter(|i| !persisted[*i]).map(|i| format!("{i}:{}@d{}", h.writes[i].kind.name(), h.writes[i].device.0)).collect();
    let torn: Vec<String> = (h.recorded..all_writes.len()).filter(|i| persisted[*i]).map(|i| format!("{i}:{}@d{}o{}", all_writes[i].kind.name(), all_writes[i].device.0, all_writes[i].offset.0)).collect();
    let torn_any = !torn.is_empty();
    (phase, format!("withheld=[{}] torn_images_landed=[{}]", withheld.join(","), torn.join(",")), torn_any)
}

fn judge(h: &History, pool: &MemoryPool, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String) {
    let (phase, cut, torn) = describe_cut(h, all_writes, persisted);
    let same_fsid = h.old_parameters.filesystem_identifier == h.new_parameters.filesystem_identifier;
    let (which, chosen_parameters, root) = match choose_system_configuration(pool) {
        Ok(c) => {
            let fsid = c.immutable.filesystem_identifier;
            let generation = c.quantities.slot_generation;
            // 新池的系统配置世代号是 1（mkfs 写的）；同 fsid 历史里世代号不是 1 的算旧池
            let is_new = fsid == h.new_parameters.filesystem_identifier && (!same_fsid || generation == 1);
            let (tag, p) = if is_new { ("new", &h.new_parameters) } else if fsid == h.old_parameters.filesystem_identifier { ("old", &h.old_parameters) } else { ("unknown", &h.old_parameters) };
            let root = choose_root(pool, &c).map_or("none".to_string(), |r| format!("i{}t{}", r.instance.0, r.checkpoint_txg.0));
            (format!("{tag}@{}g{generation}", c.immutable.sizes.fixed_structure_slot_spacing), Some(p), root)
        }
        Err(e) => (format!("Err:{}", variant(&format!("{e:?}"))), None, "-".to_string()),
    };
    let recovered = recover_label(h, pool);
    let raw_red = reds(pool);
    let read_only = match mount_read_only(pool) {
        Ok(m) => format!("Ok(i{}t{})", m.effective_root.instance.0, m.effective_root.checkpoint_txg.0),
        Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
    };
    let writable = match chosen_parameters {
        Some(p) => {
            let mut devices: Devices = plain_devices_on(pool);
            match mount_writable(p, &mut devices) {
                Ok(m) => format!("Ok(i{}t{})", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0),
                Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
            }
        }
        None => "not_tried".to_string(),
    };
    let verdict = if which.starts_with("new@") {
        if root == "i0t0" && recovered == "NoFile(i0t0)" {
            "new_complete".to_string()
        } else if root == "none" {
            "refused_new_system_configuration_without_a_root".to_string()
        } else {
            format!("MIXTURE_OR_BROKEN_NEW(root={root})")
        }
    } else if which.starts_with("old@") {
        if recovered.ends_with(":old_last_same_root") { "old_readback_intact".to_string() } else { "OLD_READBACK_NOT_INTACT".to_string() }
    } else if which.starts_with("Err:") {
        format!("refused {}", rerun_after_refusal(h, pool))
    } else {
        format!("UNKNOWN_POOL({which})")
    };
    let mut key = format!("phase={phase} torn={torn} choice={which} root={root} recover={recovered} raw_red={raw_red:?} read_only={read_only} writable={writable} verdict={verdict}");
    #[cfg(feature = "live-repair")]
    if std::env::var("PROBE_ALPHA").map_or(false, |v| v == "1") {
        key.push_str(&alpha_text(h, pool));
    }
    (key, cut)
}

/// 离线出路（α'，core 的 `system_configuration_repair`，只在打开 live-repair 时编）：在这个崩溃态上断言「旧池最后确认的那一版」（旧 fsid）与「新池第 0 代」（新 fsid），
/// 各在一份拷贝上试一次；写成了的再看恢复读回什么。
#[cfg(feature = "live-repair")]
fn alpha_text(h: &History, pool: &MemoryPool) -> String {
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::system_configuration_repair::{rewrite_system_configuration_slots_for_the_asserted_version, AssertedVersion};
    let attempt = |tag: &str, parameters: &MakeFilesystemParameters, version: (u32, u64)| -> String {
        let mut devices: Devices = plain_devices_on(pool);
        let asserted = AssertedVersion { instance: InstanceGeneration(version.0), checkpoint_txg: CheckpointTxg(version.1) };
        match rewrite_system_configuration_slots_for_the_asserted_version(parameters, &mut devices, asserted) {
            Ok(done) => {
                let after = image_of(&devices);
                format!("alpha_{tag}=Ok(rewrote={}) then {}", done.slots_rewritten.len(), recover_label(h, &after))
            }
            Err(e) => format!("alpha_{tag}=Refused:{}", variant(&format!("{e:?}"))),
        }
    };
    let mut text = format!(" {}", attempt("old_version", &h.old_parameters, h.old_last_root));
    text.push_str(&format!(" {}", attempt("new_genesis", &h.new_parameters, (0, 0))));
    text
}

#[derive(Default)]
struct Tally {
    states: u64,
    signatures: BTreeMap<String, u64>,
    examples: BTreeMap<String, (u64, String)>,
}

fn all_persisted_pool(h: &History) -> (MemoryPool, Vec<bool>) {
    let all = vec![true; h.recorded];
    (materialize(&h.base, &h.writes, &all), all)
}

fn per_device_generations(pool: &MemoryPool, spacing: u64) -> String {
    let mut parts = Vec::new();
    for device in pool.devices.keys() {
        let mut slots = Vec::new();
        for slot_index in 0..2u64 {
            let text = match PoolReader::read(pool, *device, DeviceOffsetInBytes(slot_index * spacing), 4096) {
                Some(bytes) if bytes.iter().any(|b| *b != 0) => match SystemConfiguration::parse_slot(&bytes) {
                    Ok(c) => format!("g{}/inst{}", c.quantities.slot_generation, c.quantities.journal_instance.0),
                    Err(_) => "torn".to_string(),
                },
                _ => "zero".to_string(),
            };
            slots.push(text);
        }
        parts.push(format!("d{}=[{}]", device.0, slots.join(",")));
    }
    parts.join(" ")
}

fn lag_mode(rounds: u32) {
    let new_parameters = parameters(FSID_NEW, 512);
    let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
    let (_versions, last) = make_and_use(&mut devices, &new_parameters, 5, 7);
    let spacing = u64::from(new_parameters.geometry.fixed_structure_slot_spacing);
    let show = |tag: &str, devices: &Devices| {
        let pool = image_of(devices);
        let pick = match choose_system_configuration(&pool) {
            Ok(c) => format!("g{}/inst{}", c.quantities.slot_generation, c.quantities.journal_instance.0),
            Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
        };
        let recovered = match recover(&pool, JournalPolicy::Consult).outcome {
            RecoveryOutcome::FileRead { root, .. } => format!("FileRead(i{}t{})", root.0 .0, root.1 .0),
            RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
            RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(&format!("{failure:?}"))),
        };
        let read_only = match mount_read_only(&pool) {
            Ok(m) => format!("Ok(i{}t{})", m.effective_root.instance.0, m.effective_root.checkpoint_txg.0),
            Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
        };
        println!("name=lag step={tag} slots: {} pick={pick} recover={recovered} read_only={read_only} raw_red={:?} old_last_root={last:?}", per_device_generations(&pool, spacing), reds(&pool));
    };
    show("start", &devices);
    for round in 1..=rounds {
        // 一次可写挂载（取号写两盘系统配置槽）；崩溃态：盘 0 的写落了、盘 1 的写一个没落
        let disk1_before = clone_device(&devices[1].1);
        let mut attempt: Devices = vec![(DeviceIdentity(0), clone_device(&devices[0].1)), (DeviceIdentity(1), clone_device(&devices[1].1))];
        let outcome = match mount_writable(&new_parameters, &mut attempt) {
            Ok(m) => format!("Ok(i{}t{})", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0),
            Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
        };
        println!("name=lag step=mount{round} writable_mount={outcome}");
        devices[0].1 = clone_device(&attempt[0].1);
        devices[1].1 = disk1_before;
        show(&format!("after_crash{round}"), &devices);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    if mode == "lag" {
        lag_mode(args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3));
        return;
    }
    if mode == "lag-acquire" {
        // lag-acquire <次数>：只重复「取号」这一步（它的两个系统配置槽写在同一段里），每次崩溃态 = 盘 0 的写落了、盘 1 的写一个没落。
        // 与 lag 的区别：lag 丢的是盘 1 在整次挂载里的全部写（含屏障之后的写，是「屏障只到一块盘」那一类故障）；这里只丢取号那一段里盘 1 的写，屏障照常生效。
        let rounds: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
        let new_parameters = parameters(FSID_NEW, 512);
        let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
        let (_versions, last) = make_and_use(&mut devices, &new_parameters, 5, 7);
        let spacing = u64::from(new_parameters.geometry.fixed_structure_slot_spacing);
        let show = |tag: &str, devices: &Devices| {
            let pool = image_of(devices);
            let pick = match choose_system_configuration(&pool) {
                Ok(c) => format!("g{}/inst{}", c.quantities.slot_generation, c.quantities.journal_instance.0),
                Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
            };
            let recovered = match recover(&pool, JournalPolicy::Consult).outcome {
                RecoveryOutcome::FileRead { root, .. } => format!("FileRead(i{}t{})", root.0 .0, root.1 .0),
                RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
                RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(&format!("{failure:?}"))),
            };
            let read_only = match mount_read_only(&pool) {
                Ok(m) => format!("Ok(i{}t{})", m.effective_root.instance.0, m.effective_root.checkpoint_txg.0),
                Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
            };
            let mut probe: Devices = plain_devices_on(&pool);
            let writable = match mount_writable(&new_parameters, &mut probe) {
                Ok(m) => format!("Ok(i{}t{})", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0),
                Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
            };
            println!("name=lagacq step={tag} slots: {} pick={pick} recover={recovered} read_only={read_only} writable_mount={writable} raw_red={:?} old_last_root={last:?}", per_device_generations(&pool, spacing), reds(&pool));
        };
        show("start", &devices);
        for round in 1..=rounds {
            // 取号录进一条流，崩溃态 = 流里除「盘 1 的最后一次写」以外全落（这一次写之前的屏障都生效）：
            // 入库取号一轮时就是盘 1 的唯一一次系统配置写，两轮时是第二轮盘 1 的那一次
            let base = image_of(&devices);
            let stream = SharedStream::retaining_contents();
            let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
                .iter()
                .map(|(i, d)| (*i, RecordingBlockDevice::with_shared_stream(*i, clone_device(d), stream.clone())))
                .collect();
            let acquired = {
                let mut writer = PoolWriter::new(&new_parameters, recorded.as_mut_slice());
                acquire_instance(&mut writer).map(|i| i.0).map_err(|e| variant(&format!("{e:?}")))
            };
            let ops = stream.retained_operations();
            let (writes, _segments) = writes_and_segments(&ops, &fixed_geometry(&new_parameters));
            let last_on_disk1 = (0..writes.len()).rev().find(|i| writes[*i].device == DeviceIdentity(1)).expect("盘 1 有写");
            let persisted: Vec<bool> = (0..writes.len()).map(|i| i != last_on_disk1).collect();
            println!("name=lagacq step=acquire{round} result={acquired:?} writes_in_the_acquisition={} withheld_write={last_on_disk1}", writes.len());
            let crashed = materialize(&base, &writes, &persisted);
            devices = plain_devices_on(&crashed);
            show(&format!("after_crash{round}"), &devices);
        }
        return;
    }
    if mode == "lifecycle-use" {
        // lifecycle-use <历史> <新池用到第几版>：旧池（u = 5）之上重做 mkfs 之后照旧池的用法（取号、暖机、第一个文件、覆盖写）用新池，对照组是全新空盘上同样的用法；
        // 看池级 checker 红哪些、恢复读回的是不是新池最后一版的字节
        let name = args.get(2).map(String::as_str).expect("历史");
        let usage: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
        let new_parameters = parameters(FSID_NEW, 512);
        let (mut devices, _old_parameters, old_versions, _last) = old_pool(name);
        let (new_versions, new_last) = make_and_use(&mut devices, &new_parameters, usage, 70);
        let mut control: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
        let (control_versions, control_last) = make_and_use(&mut control, &new_parameters, usage, 70);
        let describe = |tag: &str, devices: &Devices, versions: &Vec<Vec<u8>>, last: (u32, u64)| {
            let pool = image_of(devices);
            let recovered = match recover(&pool, JournalPolicy::Consult).outcome {
                RecoveryOutcome::FileRead { root, content } => format!("FileRead(i{}t{}):{}", root.0 .0, root.1 .0, if Some(&content) == versions.last() { "new_last_version" } else if old_versions.iter().any(|c| *c == content) { "OLD_POOL_CONTENT" } else { "other_content" }),
                RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
                RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(&format!("{failure:?}"))),
            };
            println!("name=lifecycle_use h={name} pool={tag} new_last_root={last:?} recover={recovered} raw_red={:?}", reds(&pool));
        };
        describe("remade_over_old_pool", &devices, &new_versions, new_last);
        describe("control_fresh_disks", &control, &control_versions, control_last);
        return;
    }
    if mode == "lifecycle" {
        // lifecycle <历史> <旧池再挂载几次> <新池做完后再挂载几次>：同 fsid / 前 8 字节相同的 mkfs 做完之后，I-7.7 红到什么时候
        let name = args.get(2).map(String::as_str).expect("历史");
        let old_extra: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
        let new_mounts: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(3);
        let new_parameters = parameters(FSID_NEW, 512);
        let (mut devices, old_parameters, _versions, last) = old_pool(name);
        for _ in 0..old_extra {
            let m = mount_writable(&old_parameters, &mut devices);
            println!("name=lifecycle step=old_extra_mount ok={}", m.is_ok());
        }
        let old_max_instance = {
            let pool = image_of(&devices);
            choose_system_configuration(&pool).map(|c| c.quantities.journal_instance.0).unwrap_or(u32::MAX)
        };
        println!("name=lifecycle h={name} old_extra_mounts={old_extra} old_pool_last_root={last:?} old_sysconf_instance={old_max_instance}");
        make_filesystem(&new_parameters, &mut devices).expect("新 mkfs");
        let show = |tag: &str, devices: &Devices| {
            let pool = image_of(devices);
            let inst = choose_system_configuration(&pool).map(|c| c.quantities.journal_instance.0).unwrap_or(u32::MAX);
            let rec = match recover(&pool, JournalPolicy::Consult).outcome {
                RecoveryOutcome::FileRead { root, .. } => format!("FileRead(i{}t{})", root.0 .0, root.1 .0),
                RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
                RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(&format!("{failure:?}"))),
            };
            println!("name=lifecycle step={tag} sysconf_instance={inst} recover={rec} raw_red={:?}", reds(&pool));
        };
        show("after_remake", &devices);
        for k in 1..=new_mounts {
            let r = mount_writable(&new_parameters, &mut devices);
            let ok = match &r { Ok(m) => format!("Ok(i{}t{})", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0), Err(e) => format!("Err:{}", variant(&format!("{e:?}"))) };
            drop(r);
            println!("name=lifecycle step=new_mount{k} writable_mount={ok}");
            show(&format!("after_new_mount{k}"), &devices);
        }
        return;
    }
    if mode == "guardcheck" {
        let name = args.get(2).map(String::as_str).expect("历史");
        let h = history(name, "today");
        for (tag, fsid) in [("old", h.old_parameters.filesystem_identifier), ("new", h.new_parameters.filesystem_identifier)] {
            println!("name=guardcheck h={name} on_the_old_pool_for_fsid={tag} unit_area_start_slot={} {}", h.unit_area_start_slot, guards_text(&h.base, &fsid, h.unit_area_start_slot));
        }
        let mut found = 0;
        let mut first = Vec::new();
        for device in h.base.devices.keys() {
            for slot in h.unit_area_start_slot..DEVICE_BYTES / SLOT_BYTES {
                if let Some(bytes) = PoolReader::read(&h.base, *device, DeviceOffsetInBytes(slot * SLOT_BYTES), 512) {
                    if &bytes[..4] == b"SFSU" {
                        found += 1;
                        if first.len() < 6 { first.push(format!("d{}s{slot}:class{}:{:?}", device.0, bytes[6], unit_header_fsid_low(&bytes).map(|l| String::from_utf8_lossy(&l).to_string()))); }
                    }
                }
            }
        }
        for slot in [h.unit_area_start_slot, h.unit_area_start_slot + 1, h.unit_area_start_slot + 2, h.unit_area_start_slot + 3] {
            let r = PoolReader::read(&h.base, DeviceIdentity(0), DeviceOffsetInBytes(slot * SLOT_BYTES), 512);
            println!("name=guardcheck_raw slot={slot} first64={:?}", r.map(|b| b.iter().take(16).cloned().collect::<Vec<u8>>()));
        }
        println!("name=guardcheck_units h={name} slots_with_unit_magic={found} first={first:?}");
        return;
    }
    if mode == "streams" {
        let name = args.get(2).map(String::as_str).expect("历史");
        let today = history(name, "today");
        let (today_pool, _) = all_persisted_pool(&today);
        for order in ["today", "yi", "yi-unitclear", "yi-unitclear-early"] {
            let h = history(name, order);
            let plain: Vec<RecordedOperation> = h.ops.iter().map(|o| o.operation.clone()).collect();
            let split = split_into_segments(&plain, &fixed_geometry(&h.new_parameters));
            let total = crash_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
            let closed = if h.segments.iter().all(|s| s.len() < 64) { closed_form_state_count(&h.segments).to_string() } else { "too_long".into() };
            let (pool, _) = all_persisted_pool(&h);
            println!(
                "name=stream h={name} order={order} operations={} writes={} segment_sizes={} closed_form_two_state={closed} states_with_torn={total} final_image_equals_today={} kinds={}",
                h.ops.len(), h.recorded, segment_sizes_text(&split), pool == today_pool, segment_kinds_text(&split)
            );
        }
        return;
    }
    let h = history(args.get(2).map(String::as_str).expect("历史"), args.get(3).map(String::as_str).expect("写序"));
    let total = crash_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
    println!("name=history h={} order={} writes={} segment_writes={:?} states_with_torn={total} old_last_root={:?}", h.name, h.order_name, h.recorded, h.segments.iter().map(Vec::len).collect::<Vec<_>>(), h.old_last_root);
    match mode {
        "final" => {
            let (pool, all) = all_persisted_pool(&h);
            let (key, _) = judge(&h, &pool, &h.writes, &all);
            println!("name=final h={} order={} {key}", h.name, h.order_name);
        }
        "run" => {
            let start: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            let end: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(total).min(total);
            assert!(end > start && end - start <= STATE_LIMIT_PER_RUN, "序号段空或超过 10⁶ 态");
            let started = std::time::Instant::now();
            let threads = threads();
            let chunk = (end - start).div_ceil(threads).max(1);
            let results = Mutex::new(Vec::new());
            std::thread::scope(|scope| {
                for thread in 0..threads {
                    let from = start + thread * chunk;
                    let to = (start + (thread + 1) * chunk).min(end);
                    if from >= to {
                        continue;
                    }
                    let (h, results) = (&h, &results);
                    scope.spawn(move || {
                        let mut tally = Tally::default();
                        let slices = no_segment_slices(h.segments.len());
                        let _ = visit_crash_state_range_of_the_read_back_flow_with_slices(&h.base, &h.writes, &h.segments, &full_expansion, &slices, from..to, &mut |ordinal, _publish, image| {
                            let pool = materialize(image.base, image.writes, &image.persisted);
                            let (key, cut) = judge(h, &pool, image.writes, &image.persisted);
                            tally.states += 1;
                            if tally.examples.get(&key).map_or(true, |(o, _)| ordinal < *o) {
                                tally.examples.insert(key.clone(), (ordinal, cut));
                            }
                            *tally.signatures.entry(key).or_insert(0) += 1;
                        });
                        results.lock().expect("锁").push(tally);
                    });
                }
            });
            let mut all = Tally::default();
            for t in results.into_inner().expect("锁") {
                all.states += t.states;
                for (k, v) in t.signatures { *all.signatures.entry(k).or_insert(0) += v; }
                for (k, (o, c)) in t.examples {
                    if all.examples.get(&k).map_or(true, |(kept, _)| o < *kept) { all.examples.insert(k, (o, c)); }
                }
            }
            println!("name=done h={} order={} states_judged={} range={start}..{end} seconds={:.1} threads={threads}", h.name, h.order_name, all.states, started.elapsed().as_secs_f64());
            for (k, v) in &all.signatures { println!("name=sig h={} order={} count={v} {k}", h.name, h.order_name); }
            for (k, (o, c)) in &all.examples { println!("name=example h={} order={} ordinal={o} {k} :: {c}", h.name, h.order_name); }
        }
        other => panic!("用法：streams|run|final|lag，收到 {other}"),
    }
}
