//! m3-close-c656-r2 攻方原型（只在仓副本里编、经内存包装直接执行编出来的二进制；不调 checker 档测试）。
//! 崩溃状态取崩溃点重放同一个计划（`visit_crash_state_range_of_the_read_back_flow_with_slices`，副本里改成 pub），展开取 `full_expansion`
//! （每段任意真子集、原地覆写带撕裂镜像）。每个状态：择系统配置、择根、恢复读回、池级 checker、只读挂载、按择到的那份参数可写挂载；
//! 整池拒的状态另在拷贝上放开「用户接着做什么」：照今天的写序再跑 mkfs，fsid 取新池的与旧池的两种，
//! 跑之前先过一遍丙七的「mkfs 读旧池」守卫（每盘 [0, 根环基址) 里逐 4096 字节找自证得过的系统配置，fsid 前 8 字节与要用的相同就拒）。
//! 流与历史都在这里自己造（旧池：mkfs + 取号 + 暖机 + 第一个文件 + 覆盖写），不从崩溃枚举用例拷。
//! 用法：
//!   c656-r2-attack-probe count <历史> <写序>
//!   c656-r2-attack-probe run <历史> <写序> [起] [止]
//!   c656-r2-attack-probe final <历史> <写序>          （mkfs 全落之后：择系统配置、恢复、池级 checker、丙七守卫）
//!   c656-r2-attack-probe handbuilt <历史> <写序> <落了的写下标,逗号分隔>   （手搭的一个状态，不是枚举）
//!   历史：same（旧池 fsid 与新池相同）、prefix（fsid 前 8 字节相同、后 8 字节不同）、distinct（前 8 字节就不同），
//!         三份都是旧池 io_min 512、两块盘都重用、u=5，新池 io_min 512；
//!         mixed-origin（盘 0 来自池 P：io_min 512、fsid P；盘 1 来自池 Q：io_min 65536、fsid Q；两池各 u=5；新池 io_min 512、fsid 新）
//!         q-with-blank-d0、q-intact（对照：池 Q 的盘 1 配一块全新空盘、配池 Q 自己的盘 0；只看 count 打的 base_choice 与 base_recover）
//!   写序：today、yi、yi-unitclear、yi-unitclear-early、yi-barrier-d0
//!   环境变量 PROBE_RERUN=0：整池拒的状态上不再跑 mkfs（省挂钟），判定列写 `refused rerun_not_tried`
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
    allocator_after_make_filesystem, make_filesystem, make_filesystem_with_order, MakeFilesystemParameters, MkfsOrderForProbe,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::root_ring::{region_start, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableSizes};
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// 一次调用不超过 10⁶ 态（派发定义第 3b 步）。
const STATE_LIMIT_PER_RUN: u64 = 1_000_000;
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const DEVICE_SLOTS: u64 = 1408 + 1007 + 384;
const DEVICE_BYTES: u64 = DEVICE_SLOTS * 16384;
const FSID_NEW: [u8; 16] = *b"c656-r2-new-pool";
/// 与新池 fsid 前 8 字节相同（单元头、journal 记录只带这 8 字节），后 8 字节不同。
const FSID_PREFIX: [u8; 16] = *b"c656-r2-old-pool";
/// 前 8 字节就与新池不同。
const FSID_DISTINCT: [u8; 16] = *b"OLDPOOL!distinct";
const FSID_Q: [u8; 16] = *b"QQPOOL!!other-q!";
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

/// 旧池的用户动作：mkfs（今天的写序）+ 取号 + 暖机 + 第一个文件 + (u − 2) 次覆盖写。交回每一版的内容与最后一版的根 (实例, txg)。
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

struct History {
    name: String,
    order_name: String,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    recorded: usize,
    /// 盘 0 上那个旧池（mkfs 之前盘上可挂的那个池）的参数、各版内容与最后一版的根。
    old_parameters: MakeFilesystemParameters,
    old_versions: Vec<Vec<u8>>,
    old_last_root: (u32, u64),
    /// 盘 1 来自另一个池时（只 mixed-origin 有）：那个池的参数、各版内容与最后一版的根。
    other: Option<(MakeFilesystemParameters, Vec<Vec<u8>>, (u32, u64))>,
    new_parameters: MakeFilesystemParameters,
}

fn order_of(name: &str) -> MkfsOrderForProbe {
    match name {
        "today" => MkfsOrderForProbe::Today,
        "yi" => MkfsOrderForProbe::InvalidateFirst,
        "yi-unitclear" => MkfsOrderForProbe::InvalidateFirstThenClearUnitArea,
        "yi-unitclear-early" => MkfsOrderForProbe::ClearUnitAreaTogetherWithInvalidation,
        "yi-barrier-d0" => MkfsOrderForProbe::InvalidateFirstBarrierOnlyOnTheFirstDevice,
        other => panic!("未知写序 {other}"),
    }
}

fn history(name: &str, order_name: &str) -> History {
    let order = order_of(order_name);
    let new_parameters = parameters(FSID_NEW, 512);
    let (devices, old_parameters, old_versions, old_last_root, other) = match name {
        "same" | "prefix" | "distinct" => {
            let fsid = match name { "same" => FSID_NEW, "prefix" => FSID_PREFIX, _ => FSID_DISTINCT };
            let old_parameters = parameters(fsid, 512);
            let mut old: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (versions, last) = make_and_use(&mut old, &old_parameters, 5, 7);
            (old, old_parameters, versions, last, None)
        }
        "mixed-origin" => {
            let p = parameters(FSID_DISTINCT, 512);
            let mut pair_p: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (p_versions, p_last) = make_and_use(&mut pair_p, &p, 5, 7);
            let q = parameters(FSID_Q, 65536);
            let mut pair_q: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (q_versions, q_last) = make_and_use(&mut pair_q, &q, 5, 70);
            let devices: Devices = vec![(DeviceIdentity(0), clone_device(&pair_p[0].1)), (DeviceIdentity(1), clone_device(&pair_q[1].1))];
            (devices, p, p_versions, p_last, Some((q, q_versions, q_last)))
        }
        // 对照：盘 1 来自池 Q（同 mixed-origin 的造法），盘 0 是一块全新的空盘——池 Q 只剩盘 1 时挂得上哪一版
        "q-with-blank-d0" | "q-intact" => {
            let p = parameters(FSID_DISTINCT, 512);
            let mut pair_p: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (p_versions, p_last) = make_and_use(&mut pair_p, &p, 5, 7);
            let q = parameters(FSID_Q, 65536);
            let mut pair_q: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (q_versions, q_last) = make_and_use(&mut pair_q, &q, 5, 70);
            let first = if name == "q-intact" { clone_device(&pair_q[0].1) } else { fresh_device() };
            let devices: Devices = vec![(DeviceIdentity(0), first), (DeviceIdentity(1), clone_device(&pair_q[1].1))];
            (devices, p, p_versions, p_last, Some((q, q_versions, q_last)))
        }
        other => panic!("未知历史 {other}"),
    };
    let base = image_of(&devices);
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
        .iter()
        .map(|(i, d)| (*i, RecordingBlockDevice::with_shared_stream(*i, clone_device(d), stream.clone())))
        .collect();
    make_filesystem_with_order(&new_parameters, &mut recorded, order).expect("新 mkfs");
    let fixed = FixedGeometry {
        fixed_structure_slot_spacing: new_parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: new_parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: new_parameters.geometry.root_ring_slots_per_region,
    };
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &fixed);
    let recorded_count = writes.len();
    History { name: name.into(), order_name: order_name.into(), base, writes, segments, recorded: recorded_count, old_parameters, old_versions, old_last_root, other, new_parameters }
}

fn content_class(h: &History, root: (u32, u64), bytes: &[u8]) -> String {
    let classify = |versions: &[Vec<u8>], last_root: (u32, u64), tag: &str| -> Option<String> {
        let last = versions.last()?;
        if bytes == last.as_slice() && root == last_root {
            Some(format!("{tag}_last_same_root"))
        } else if bytes == last.as_slice() {
            Some(format!("{tag}_last_other_root"))
        } else if versions.iter().any(|c| c.as_slice() == bytes) {
            Some(format!("{tag}_older_version"))
        } else {
            None
        }
    };
    if let Some(c) = classify(&h.old_versions, h.old_last_root, "old") {
        return c;
    }
    if let Some((_, versions, last)) = &h.other {
        if let Some(c) = classify(versions, *last, "other_old") {
            return c;
        }
    }
    "other_content".into()
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

/// 丙七的「mkfs 读旧池」守卫（推的最强形，只在原型里）：每块盘 [0, 根环基址) 里逐 4096 字节试解一份系统配置，
/// 自证得过的收下它的 fsid；有一份与要用的 fsid 前 8 字节相同就拒（单元头与 journal 记录只带前 8 字节）。
/// 交回 (拒不拒, 找到的 fsid 前 8 字节去重后的个数)。
fn guard_refuses(pool: &MemoryPool, fsid: &[u8; 16]) -> (bool, usize) {
    let base = region_start(0).0;
    let mut found: std::collections::BTreeSet<[u8; 8]> = std::collections::BTreeSet::new();
    for device in pool.devices.keys() {
        let mut offset = 0;
        while offset + 4096 <= base {
            if let Some(bytes) = PoolReader::read(pool, *device, DeviceOffsetInBytes(offset), 4096) {
                if bytes.iter().any(|b| *b != 0) {
                    if let Ok(c) = SystemConfiguration::parse_slot(&bytes) {
                        let mut low = [0u8; 8];
                        low.copy_from_slice(&c.immutable.filesystem_identifier[..8]);
                        found.insert(low);
                    }
                }
            }
            offset += 4096;
        }
    }
    (found.iter().any(|low| low[..] == fsid[..8]), found.len())
}

/// 整池拒之后用户接着做什么：照今天的写序再跑 mkfs，fsid 取新池的与旧池的两种；先过丙七守卫，再看新池恢复与池级 checker。
fn rerun_after_refusal(h: &History, pool: &MemoryPool) -> String {
    let mut parts = Vec::new();
    let mut fsids = vec![("new", h.new_parameters.filesystem_identifier)];
    if h.old_parameters.filesystem_identifier != h.new_parameters.filesystem_identifier {
        fsids.push(("old", h.old_parameters.filesystem_identifier));
    }
    for (tag, fsid) in fsids {
        let (refused, found) = guard_refuses(pool, &fsid);
        let p = MakeFilesystemParameters { filesystem_identifier: fsid, ..h.new_parameters };
        let mut devices: Devices = plain_devices_on(pool);
        let outcome = match make_filesystem(&p, &mut devices) {
            Ok(_) => {
                let after = image_of(&devices);
                format!("{}|{:?}", recover_label(h, &after), reds(&after))
            }
            Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
        };
        parts.push(format!("rerun_{tag}[guard={} found={found}]={outcome}", if refused { "refuse" } else { "allow" }));
    }
    parts.join(" ")
}

/// 一个状态的写落法：第一个没落的写在第几段（全落是 `all`）、没落的写、落了的撕裂镜像（撕裂镜像排在录制的写之后）。
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

fn parameters_of_fsid<'a>(h: &'a History, fsid: &[u8; 16]) -> Option<(&'static str, &'a MakeFilesystemParameters)> {
    if *fsid == h.new_parameters.filesystem_identifier {
        Some(("new", &h.new_parameters))
    } else if *fsid == h.old_parameters.filesystem_identifier {
        Some(("old", &h.old_parameters))
    } else if let Some((q, _, _)) = &h.other {
        (*fsid == q.filesystem_identifier).then_some(("other_old", q))
    } else {
        None
    }
}

fn judge(h: &History, pool: &MemoryPool, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String) {
    let (phase, cut, torn) = describe_cut(h, all_writes, persisted);
    let (which, chosen_parameters, root) = match choose_system_configuration(pool) {
        Ok(c) => {
            let fsid = c.immutable.filesystem_identifier;
            let spacing = c.immutable.sizes.fixed_structure_slot_spacing;
            let generation = c.quantities.slot_generation;
            let (tag, p) = match parameters_of_fsid(h, &fsid) {
                // 同 fsid 历史里新旧 fsid 相同：世代 1 且槽距是新池的算新池，否则算旧池
                Some(("new", p)) if h.old_parameters.filesystem_identifier == fsid && generation != 1 => ("old", Some(&h.old_parameters).map(|x| x).or(Some(p))),
                Some((tag, p)) => (tag, Some(p)),
                None => ("unknown", None),
            };
            let root = choose_root(pool, &c).map_or("none".to_string(), |r| format!("i{}t{}", r.instance.0, r.checkpoint_txg.0));
            (format!("{tag}@{spacing}g{generation}"), p, root)
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
    // 判定：择到新池的要是第 0 代空池；择到旧池（或盘 1 来自的那个池）的读回要与它最后确认的那一版逐字节相同；
    // 整池拒的放开用户接着做的事（再跑 mkfs，两种 fsid）
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
    } else if which.starts_with("other_old@") {
        if recovered.ends_with(":other_old_last_same_root") { "other_old_readback_intact".to_string() } else { "OTHER_OLD_READBACK_NOT_INTACT".to_string() }
    } else if which.starts_with("Err:") {
        // PROBE_RERUN=0 时不放开「再跑 mkfs」那一步（只为省挂钟；报告里写明哪几份历史关了它）
        if std::env::var("PROBE_RERUN").map_or(true, |v| v != "0") {
            format!("refused {}", rerun_after_refusal(h, pool))
        } else {
            "refused rerun_not_tried".to_string()
        }
    } else {
        format!("UNKNOWN_POOL({which})")
    };
    let key = format!("phase={phase} torn={torn} choice={which} root={root} recover={recovered} raw_red={raw_red:?} read_only={read_only} writable={writable} verdict={verdict}");
    (key, cut)
}

#[derive(Default)]
struct Tally {
    states: u64,
    signatures: BTreeMap<String, u64>,
    examples: BTreeMap<String, (u64, String)>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    let h = history(args.get(2).map(String::as_str).expect("历史"), args.get(3).map(String::as_str).expect("写序"));
    let total = crash_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
    let shape: Vec<usize> = h.segments.iter().map(Vec::len).collect();
    let kinds: Vec<String> = h.writes[..h.recorded].iter().map(|w| format!("{}@d{}o{}", w.kind.name(), w.device.0, w.offset.0)).collect();
    let base_choice = match choose_system_configuration(&h.base) {
        Ok(c) => format!("{}g{}", String::from_utf8_lossy(&c.immutable.filesystem_identifier), c.quantities.slot_generation),
        Err(e) => format!("Err:{}", variant(&format!("{e:?}"))),
    };
    let closed = if shape.iter().all(|n| *n < 64) { closed_form_state_count(&h.segments).to_string() } else { "too_long".into() };
    println!("name=history h={} order={} writes={} segment_sizes={shape:?} closed_form_two_state={closed} states_with_torn={total} old_last_root={:?} base_choice={base_choice} base_recover={} kinds={kinds:?}", h.name, h.order_name, h.recorded, h.old_last_root, recover_label(&h, &h.base));
    match mode {
        "count" => {}
        "final" => {
            let all = vec![true; h.recorded];
            let pool = materialize(&h.base, &h.writes, &all);
            let (key, _) = judge(&h, &pool, &h.writes, &all);
            let (refuse_new, found_new) = guard_refuses(&h.base, &h.new_parameters.filesystem_identifier);
            println!("name=final h={} order={} guard_on_the_old_pool_for_the_new_fsid={} found={found_new} {key}", h.name, h.order_name, if refuse_new { "refuse" } else { "allow" });
        }
        "handbuilt" => {
            let landed: Vec<usize> = args.get(4).expect("落了的写下标").split(',').map(|s| s.parse().expect("下标")).collect();
            let persisted: Vec<bool> = (0..h.recorded).map(|i| landed.contains(&i)).collect();
            let pool = materialize(&h.base, &h.writes, &persisted);
            let (key, cut) = judge(&h, &pool, &h.writes, &persisted);
            println!("name=handbuilt h={} order={} landed={landed:?} {key} :: {cut}", h.name, h.order_name);
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
        other => panic!("用法：count|run|final|handbuilt，收到 {other}"),
    }
}
