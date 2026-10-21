//! m3-close-c656-r1 攻方第二次抽样原型（只在仓副本里编、经内存包装直接执行编出来的二进制；不调 checker 档测试）。
//! 崩溃状态取崩溃点重放同一个计划（`visit_crash_state_range_of_the_read_back_flow_with_slices`，副本里改成 pub），展开取 `full_expansion`
//! （每段任意真子集、原地覆写带撕裂镜像），每个状态：择系统配置、择根、恢复读回、池级 checker、只读挂载、按择到的那份参数可写挂载；
//! 整池拒的状态另在拷贝上照今天的写序再跑一次 mkfs、看新池能不能恢复成空池且 checker 绿。
//! 流与历史都在这里自己造（旧池：mkfs + 取号 + 暖机 + 第一个文件 + 覆盖写），不从崩溃枚举用例拷。
//! 用法：c656-s2-attack-probe count <历史> <写序>；c656-s2-attack-probe run <历史> <写序> [起] [止]
//!   历史：x512-same、x512-diff、x65536-same、x65536-diff（旧池 io_min x、两块盘都重用、u=5，新池 io_min 512，fsid 同或不同）、
//!         leftover（旧旧池 A：io_min 65536、fsid A、u=5 → 今天的 mkfs 做池 B：io_min 512、fsid B、u=2 → 新 mkfs：io_min 65536、fsid A）
//!   写序：today、yi、jia
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
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;
use std::sync::Mutex;

/// 一次调用不超过 10⁶ 态（派发定义第 3b 步）。
const STATE_LIMIT_PER_RUN: u64 = 1_000_000;
const JOURNAL_RING_BYTES: u64 = 6 << 20;
const DEVICE_SLOTS: u64 = 1408 + 1007 + 384;
const DEVICE_BYTES: u64 = DEVICE_SLOTS * 16384;
const FSID_A: [u8; 16] = *b"c656-s2-pool-A!!";
const FSID_B: [u8; 16] = *b"c656-s2-pool-B!!";
const FSID_NEW: [u8; 16] = *b"c656-s2-new-pool";
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
    /// 旧池（mkfs 之前盘上可挂的那个池）的参数、各版内容与最后一版的根。
    old_parameters: MakeFilesystemParameters,
    old_versions: Vec<Vec<u8>>,
    old_last_root: (u32, u64),
    /// 更早一个池（只 leftover 有）的各版内容。
    older_versions: Vec<Vec<u8>>,
    new_parameters: MakeFilesystemParameters,
    note: String,
}

fn order_of(name: &str) -> MkfsOrderForProbe {
    match name {
        "today" => MkfsOrderForProbe::Today,
        "yi" => MkfsOrderForProbe::InvalidateFirst,
        // 甲候选的单元落点：盘末尾往回 64 槽、对齐到 64 槽（旧池在这几块盘上没写到那里，构造时逐字节核全 0）
        "jia" => MkfsOrderForProbe::OldPoolUntouchedUntilCommit { unit_slot: (DEVICE_SLOTS - 64) / 64 * 64 },
        other => panic!("未知写序 {other}"),
    }
}

fn history(name: &str, order_name: &str) -> History {
    let order = order_of(order_name);
    let (devices, old_parameters, old_versions, old_last_root, older_versions, new_parameters) = match name {
        "x512-same" | "x512-diff" | "x65536-same" | "x65536-diff" => {
            let io: u32 = if name.starts_with("x512") { 512 } else { 65536 };
            let same = name.ends_with("same");
            let old_parameters = parameters(if same { FSID_NEW } else { FSID_A }, io);
            let mut old: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (versions, last) = make_and_use(&mut old, &old_parameters, 5, 7);
            (old, old_parameters, versions, last, Vec::new(), parameters(FSID_NEW, 512))
        }
        "leftover" => {
            let a = parameters(FSID_A, 65536);
            let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            let (a_versions, _) = make_and_use(&mut devices, &a, 5, 7);
            let b = parameters(FSID_B, 512);
            let (b_versions, b_last) = make_and_use(&mut devices, &b, 2, 40);
            (devices, b, b_versions, b_last, a_versions, parameters(FSID_A, 65536))
        }
        other => panic!("未知历史 {other}"),
    };
    let base = image_of(&devices);
    let mut note = String::new();
    if let MkfsOrderForProbe::OldPoolUntouchedUntilCommit { unit_slot } = order {
        let all_zero = [DeviceIdentity(0), DeviceIdentity(1)].iter().all(|d| {
            base.read(*d, DeviceOffsetInBytes(unit_slot * SLOT_BYTES), usize::try_from(3 * SLOT_BYTES).expect("3 槽")).expect("读得出").iter().all(|b| *b == 0)
        });
        assert!(all_zero, "甲候选的单元落点上旧池写过字节");
        note = format!("jia_unit_slot={unit_slot} old_bytes_there_all_zero={all_zero}");
    }
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
    History { name: name.into(), order_name: order_name.into(), base, writes, segments, recorded: recorded_count, old_parameters, old_versions, old_last_root, older_versions, new_parameters, note }
}

fn content_class(h: &History, root: (u32, u64), bytes: &[u8]) -> String {
    let last = h.old_versions.last().expect("旧池至少一版");
    if bytes == last.as_slice() && root == h.old_last_root {
        "old_last_same_root".into()
    } else if bytes == last.as_slice() {
        "old_last_other_root".into()
    } else if h.old_versions.iter().any(|c| c.as_slice() == bytes) {
        "old_older_version".into()
    } else if h.older_versions.iter().any(|c| c.as_slice() == bytes) {
        "content_of_the_pool_before_the_old_pool".into()
    } else {
        "other_content".into()
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

fn bump(map: &mut BTreeMap<String, u64>, key: String) {
    *map.entry(key).or_insert(0) += 1;
}

/// 一个状态的写落法：第一个没落的写在第几段（全落是 `all`）、没落的写、落了的撕裂镜像。
fn describe_cut(h: &History, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String, bool) {
    let first_withheld = (0..h.recorded).find(|i| !persisted[*i]);
    let phase = match first_withheld {
        Some(w) => format!("seg{}", h.segments.iter().position(|s| s.contains(&w)).unwrap_or(h.segments.len())),
        None => "all".to_string(),
    };
    let withheld: Vec<String> = (0..h.recorded).filter(|i| !persisted[*i]).map(|i| format!("{i}:{}@d{}", h.writes[i].kind.name(), h.writes[i].device.0)).collect();
    // 撕裂镜像排在录制的写之后（计划的写表 `all_writes`，比录制流长）
    let torn: Vec<String> = (h.recorded..all_writes.len()).filter(|i| persisted[*i]).map(|i| format!("{i}:{}@d{}o{}", all_writes[i].kind.name(), all_writes[i].device.0, all_writes[i].offset.0)).collect();
    let torn_any = !torn.is_empty();
    (phase, format!("withheld=[{}] torn_images_landed=[{}]", withheld.join(","), torn.join(",")), torn_any)
}

fn judge(h: &History, pool: &MemoryPool, all_writes: &[RetainedWrite], persisted: &[bool]) -> (String, String) {
    let (phase, cut, torn) = describe_cut(h, all_writes, persisted);
    let torn_zero = (h.recorded..all_writes.len()).any(|k| persisted[k] && all_writes[k].kind == StepKind::ZeroFill);
    let new_spacing = h.new_parameters.geometry.fixed_structure_slot_spacing;
    let old_spacing = h.old_parameters.geometry.fixed_structure_slot_spacing;
    let (which, chosen_parameters, root) = match choose_system_configuration(pool) {
        Ok(c) => {
            let fsid = c.immutable.filesystem_identifier;
            let spacing = c.immutable.sizes.fixed_structure_slot_spacing;
            let generation = c.quantities.slot_generation;
            let which = if fsid == h.new_parameters.filesystem_identifier && spacing == new_spacing && generation == 1 {
                "new"
            } else if fsid == h.old_parameters.filesystem_identifier && spacing == old_spacing {
                "old"
            } else if fsid == FSID_A {
                "pool_before_old"
            } else {
                "other"
            };
            let root = choose_root(pool, &c).map_or("none".to_string(), |r| format!("i{}t{}", r.instance.0, r.checkpoint_txg.0));
            let p = if which == "old" { Some(&h.old_parameters) } else { Some(&h.new_parameters) };
            (format!("{which}@{spacing}g{generation}"), p, root)
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
    // 判定：择到新池的要是第 0 代空池；择到旧池的读回要与最后确认的那一版逐字节相同；别的池一律算混合
    let verdict = if which.starts_with("new@") {
        if root == "i0t0" && recovered == "NoFile(i0t0)" {
            "new_complete".to_string()
        } else if root == "none" {
            // 择到新池的系统配置而根环里没有一条自证得过的根：恢复报 NoValidRoot，等于整池拒
            "refused_new_system_configuration_without_a_root".to_string()
        } else {
            format!("MIXTURE_OR_BROKEN_NEW(root={root})")
        }
    } else if which.starts_with("old@") {
        if recovered.ends_with(":old_last_same_root") { "old_readback_intact".to_string() } else { "OLD_READBACK_NOT_INTACT".to_string() }
    } else if which.starts_with("Err:") {
        // 整池拒：照今天的写序在拷贝上再跑一次 mkfs，看新池是不是空池、checker 绿
        let mut devices: Devices = plain_devices_on(pool);
        let rerun = match make_filesystem(&h.new_parameters, &mut devices) {
            Ok(_) => {
                let after = image_of(&devices);
                format!("rerun_mkfs={} rerun_red={:?}", recover_label(h, &after), reds(&after))
            }
            Err(e) => format!("rerun_mkfs=Err:{}", variant(&format!("{e:?}"))),
        };
        format!("refused {rerun}")
    } else {
        format!("MIXTURE_OTHER_POOL({which})")
    };
    let key = format!("phase={phase} torn={torn} torn_zero={torn_zero} choice={which} root={root} recover={recovered} raw_red={raw_red:?} read_only={read_only} writable={writable} verdict={verdict}");
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
    if mode == "fresh" {
        // 对照：全新的两块空盘上照今天的写序 mkfs（新池参数 io_min 512 与 65536 各一次），完成之后跑池级 checker 与恢复
        for io in [512u32, 65536] {
            let p = parameters(FSID_NEW, io);
            let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
            make_filesystem(&p, &mut devices).expect("mkfs");
            let image = image_of(&devices);
            let recovered = match recover(&image, JournalPolicy::Consult).outcome {
                RecoveryOutcome::NoFile { root } => format!("NoFile(i{}t{})", root.0 .0, root.1 .0),
                other => format!("{:?}", variant(&format!("{other:?}"))),
            };
            println!("name=fresh io_min={io} recover={recovered} raw_red={:?}", reds(&image));
        }
        return;
    }
    let h = history(args.get(2).map(String::as_str).expect("历史"), args.get(3).map(String::as_str).expect("写序"));
    let total = crash_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
    let shape: Vec<usize> = h.segments.iter().map(Vec::len).collect();
    let kinds: Vec<String> = h.writes[..h.recorded].iter().map(|w| format!("{}@d{}o{}", w.kind.name(), w.device.0, w.offset.0)).collect();
    println!("name=history h={} order={} writes={} segment_sizes={shape:?} closed_form_two_state={} states_with_torn={total} old_last_root={:?} {} kinds={kinds:?}", h.name, h.order_name, h.recorded, closed_form_state_count(&h.segments), h.old_last_root, h.note);
    if mode == "count" {
        return;
    }
    assert_eq!(mode, "run", "用法：count|run <历史> <写序> [起] [止]");
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
                    let keep = tally.examples.get(&key).map_or(true, |(o, _)| ordinal < *o);
                    if keep {
                        tally.examples.insert(key.clone(), (ordinal, cut));
                    }
                    bump(&mut tally.signatures, key);
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
