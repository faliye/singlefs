//! m3-close-c656-r1 攻方腿原型（只在草稿目录的仓副本里编、经内存包装直接执行编出来的二进制）。
//! 旧池：两块盘新 mkfs（旧 fsid、旧 io_min）→ 取号 → 暖机 → 第一个文件 → u−2 次覆盖写，全在同一个进程里（实例 1）。
//! 新 mkfs：两块盘都重用，按 `make_filesystem_in_order` 的写序录成写流；崩溃状态照崩溃点重放的枚举域取：
//! `enumerate_crash_states_with_expansion`，每一段都按 `EveryProperSubset` 展开（与 `enumerate_crash_states_versions` 同一个展开），
//! 原地覆写取三态；挂一个观察者逐态看择到哪份系统配置、恢复读回什么、只读挂载、（可选）可写挂载、池级 checker。
//! 模式：
//!   count <历史> <写序>            写流段形、闭式、全量状态数
//!   run <历史> <写序>              全量枚举，打观察者计数
//!   image <历史> <形状>            不经写流，直接在旧池镜像上摆几种盘面（撕裂形状由这里定），逐个看
//! 历史名：o<旧 io_min>-u<u>-f<same|diff>；写序：today | invalidate | sysconf-first | new-then-sysconf。
#![allow(clippy::all)]

use std::collections::{BTreeMap, HashSet};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_library::StageOutcome;
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem, CrashPointReplayObserver,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, make_filesystem_in_order,
    MakeFilesystemOrderUnderAttack, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const JOURNAL_RING_BYTES: u64 = 6 << 20;
/// journal 环 6 MiB 时单元区从槽 1408 起；单元区给 4096 槽（64 MiB），覆盖写 21 次不撞空间准入。
const DEVICE_BYTES: u64 = (1408 + 4096) * 16384;
const NEW_FSID: [u8; 16] = *b"c656-attack-new!";
const OLD_FSID: [u8; 16] = *b"c656-attack-old!";
/// 辩方腿加：与新 fsid 的前 8 字节也不同（r1 两个原型的 fdiff 只差第 9 字节起，单元头只带 fsid 前 8 字节时分不出）。
const OLD_FSID_PREFIX_DIFFERENT: [u8; 16] = *b"old-prefixdiff!!";
const WRITE_TIME: u64 = 1_788_000_000;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

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

fn content(seed: u64) -> Vec<u8> {
    (0..3000u64).map(|i| u8::try_from((i * 137 + seed * 11 + 3) % 251).expect("<256")).collect()
}

fn image_of(devices: &Devices) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: DEVICE_BYTES }
}

fn devices_of(pool: &MemoryPool) -> Devices {
    pool.devices.iter().map(|(i, image)| {
        let mut d = fresh_device();
        d.image = image.clone();
        (*i, d)
    }).collect()
}

/// 旧池：mkfs → 取号 → 暖机 → 第一个文件 → u−2 次覆盖写。交回每一版（实例 1、txg 3 起）的内容。
fn old_pool(old: &MakeFilesystemParameters, u: u32) -> (Devices, Vec<PublishedVersion>) {
    let mut devices: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
    let genesis = make_filesystem(old, &mut devices).expect("旧池 mkfs");
    let mut allocator = allocator_after_make_filesystem(old, &devices, &genesis);
    let mut versions = Vec::new();
    {
        let mut writer = PoolWriter::new(old, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = content(7);
        let mut previous = publish_first_file(&mut writer, &mut allocator, warmed.roots.last().expect("两代根"),
            FirstFile { content: &first, write_time_seconds: WRITE_TIME }, instance, &warmed.last_record_bytes).expect("第一个文件");
        versions.push(PublishedVersion { instance: previous.root.instance, checkpoint_txg: previous.root.checkpoint_txg, content: first });
        for k in 0..u.saturating_sub(2) {
            let c = content(8 + u64::from(k));
            previous = publish_overwrite(&mut writer, &mut allocator, &previous,
                FirstFile { content: &c, write_time_seconds: WRITE_TIME + 1 + u64::from(k) }, instance).expect("覆盖写");
            versions.push(PublishedVersion { instance: previous.root.instance, checkpoint_txg: previous.root.checkpoint_txg, content: c });
        }
    }
    (devices, versions)
}

struct History {
    name: String,
    old: MakeFilesystemParameters,
    new: MakeFilesystemParameters,
    base: MemoryPool,
    versions: Vec<PublishedVersion>,
}

fn history(name: &str) -> History {
    // o<io_min>-u<u>-f<same|diff>
    let parts: Vec<&str> = name.split('-').collect();
    assert_eq!(parts.len(), 3, "历史名 o<io_min>-u<u>-f<same|diff>：{name}");
    let io_min: u32 = parts[0].strip_prefix('o').expect("o").parse().expect("io_min");
    let u: u32 = parts[1].strip_prefix('u').expect("u").parse().expect("u");
    let old_fsid = match parts[2] { "fsame" => NEW_FSID, "fdiff" => OLD_FSID, "fpdiff" => OLD_FSID_PREFIX_DIFFERENT, other => panic!("{other}") };
    let old = parameters(old_fsid, io_min);
    let (devices, versions) = old_pool(&old, u);
    History { name: name.to_string(), old, new: parameters(NEW_FSID, 512), base: image_of(&devices), versions }
}

/// 辩方腿加：`latest-root-last` 需要 mkfs 先读旧池的两样（最新旧根住在哪个区域的哪个槽、单元写进哪几个全 0 的槽），
/// 这里从历史里直接算出来传给写序；量的只是写序，不是「mkfs 读旧池」那一段。
fn order_of(name: &str, h: &History) -> MakeFilesystemOrderUnderAttack {
    match name {
        "latest-root-last" => {
            let last = h.versions.last().expect("旧池至少一版");
            let target = singlefs_core::root_ring::target_for_publish(last.checkpoint_txg, h.new.geometry.root_ring_slots_per_region);
            let total_slots = DEVICE_BYTES / 16384;
            let unit_slot = total_slots - 256;
            for (_, d) in devices_of(&h.base) {
                let mut buffer = vec![0u8; 3 * 16384];
                d.read_at(DeviceOffsetInBytes(unit_slot * 16384), &mut buffer).expect("读");
                assert!(buffer.iter().all(|b| *b == 0), "单元落点不是全 0");
            }
            MakeFilesystemOrderUnderAttack::LatestRootLast { latest_region: target.region, latest_slot: target.slot, unit_slot }
        }
        "today" => MakeFilesystemOrderUnderAttack::Today,
        "invalidate" => MakeFilesystemOrderUnderAttack::InvalidateFirst,
        "sysconf-first" => MakeFilesystemOrderUnderAttack::SystemConfigurationFirst,
        "new-then-sysconf" => MakeFilesystemOrderUnderAttack::NewBytesThenSystemConfiguration,
        other => panic!("写序 {other}"),
    }
}

fn flow(h: &History, order: MakeFilesystemOrderUnderAttack) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices_of(&h.base)
        .into_iter()
        .map(|(i, d)| (i, RecordingBlockDevice::with_shared_stream(i, d, stream.clone())))
        .collect();
    make_filesystem_in_order(&h.new, &mut recorded, order).expect("新 mkfs");
    let fixed = FixedGeometry {
        fixed_structure_slot_spacing: h.new.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: h.new.geometry.journal_ring_bytes,
        root_ring_slots_per_region: h.new.geometry.root_ring_slots_per_region,
    };
    writes_and_segments(&stream.retained_operations(), &fixed)
}

fn every_proper_subset(_segment_index: usize, _segment: &[usize]) -> CrashPointReplaySegmentExpansion {
    CrashPointReplaySegmentExpansion::EveryProperSubset
}

fn snake(camel: &str) -> String {
    let mut out = String::new();
    for (index, character) in camel.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 { out.push('_'); }
            out.push(character.to_ascii_lowercase());
        } else if character.is_ascii_alphanumeric() {
            out.push(character);
        } else {
            out.push('_');
        }
    }
    out
}

fn variant(debug: &str) -> String {
    snake(debug.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or(""))
}

/// 择到的系统配置是哪一池：新池那一份（新 fsid、世代号 1、实例代号 0、新槽距）、旧池（旧 fsid、旧槽距；带世代号）、别的、或择不到（带错因）。
fn choice_of(h: &History, reader: &dyn PoolReader) -> (String, Option<MakeFilesystemParameters>) {
    match choose_system_configuration(reader) {
        Ok(sc) => {
            let spacing = sc.immutable.sizes.fixed_structure_slot_spacing;
            let fsid = sc.immutable.filesystem_identifier;
            let new_pattern = fsid == h.new.filesystem_identifier && sc.quantities.journal_instance.0 == 0
                && sc.quantities.slot_generation == 1 && spacing == h.new.geometry.fixed_structure_slot_spacing;
            if new_pattern {
                ("new".into(), Some(h.new.clone()))
            } else if fsid == h.old.filesystem_identifier && spacing == h.old.geometry.fixed_structure_slot_spacing {
                (format!("old_g{}", sc.quantities.slot_generation), Some(h.old.clone()))
            } else {
                ("other".into(), None)
            }
        }
        Err(e) => (format!("err_{}", variant(&format!("{e:?}"))), None),
    }
}

fn recovery_class(h: &History, outcome: &RecoveryOutcome) -> String {
    let last = h.versions.last().expect("旧池至少一版");
    match outcome {
        RecoveryOutcome::FileRead { root, content } => {
            if *root == (last.instance, last.checkpoint_txg) && *content == last.content {
                "oldlast".into()
            } else if *content == last.content {
                format!("oldlast_at_i{}t{}", root.0 .0, root.1 .0)
            } else if let Some(v) = h.versions.iter().find(|v| v.content == *content) {
                format!("oldolder_t{}_at_i{}t{}", v.checkpoint_txg.0, root.0 .0, root.1 .0)
            } else {
                format!("othercontent_at_i{}t{}", root.0 .0, root.1 .0)
            }
        }
        RecoveryOutcome::NoFile { root } => {
            if *root == (InstanceGeneration(0), CheckpointTxg(0)) { "newempty".into() } else { format!("nofile_i{}t{}", root.0 .0, root.1 .0) }
        }
        RecoveryOutcome::Failed { failure, .. } => format!("fail_{}", variant(&format!("{failure:?}"))),
    }
}

fn materialize(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, persisted) in image.writes.iter().zip(&image.persisted) {
        if *persisted { pool.apply_writes(std::slice::from_ref(write)); }
    }
    pool
}

fn writable_enabled() -> bool { std::env::var("PROBE_WRITABLE").as_deref() == Ok("1") }

/// 一个盘面的全部判定拼成一个计数名（只有小写字母、数字、_）。
fn judge<R: PoolReader + singlefs_checker::image::ImageReader>(h: &History, reader: &R, outcome: &RecoveryOutcome, pool_for_writable: impl FnOnce() -> MemoryPool) -> String {
    let (choice, params) = choice_of(h, reader);
    let rec = recovery_class(h, outcome);
    let red = check_pool_image(reader).iter().any(|(_, v)| matches!(v, InvariantVerdict::Violated(_)));
    let ro = match mount_read_only(reader) {
        Ok(m) => format!("ro_i{}t{}", m.effective_root.instance.0, m.effective_root.checkpoint_txg.0),
        Err(e) => format!("ro_err_{}", variant(&format!("{e:?}"))),
    };
    let rw = if !writable_enabled() {
        "rw_skipped".to_string()
    } else {
        match params {
            None => "rw_not_tried".into(),
            Some(p) => {
                let mut devices = devices_of(&pool_for_writable());
                match mount_writable(&p, &mut devices) {
                    Ok(m) => format!("rw_ok_i{}t{}", m.output.effective_root.instance.0, m.output.effective_root.checkpoint_txg.0),
                    Err(e) => format!("rw_err_{}", variant(&format!("{e:?}"))),
                }
            }
        }
    };
    let mixed = (choice == "new" && rec != "newempty" && !rec.starts_with("fail_"))
        || (choice.starts_with("old") && rec == "newempty");
    let intact = choice.starts_with("old") && rec == "oldlast";
    format!("{choice}__{rec}__{}__{ro}__{rw}__{}{}", if red { "red" } else { "green" },
        if mixed { "mixed" } else { "notmixed" }, if intact { "__oldintact" } else { "" })
}

fn kinds_line(writes: &[RetainedWrite]) -> String {
    writes.iter().enumerate().map(|(i, w)| format!("{i}:{}@d{}o{}+{}", w.kind.name(), w.device.0, w.offset.0, w.length_in_bytes())).collect::<Vec<_>>().join(",")
}

fn mode_count(h: &History, order_name: &str) {
    let (writes, segments) = flow(h, order_of(order_name, h));
    let full = crash_state_count_with_torn_in_place_overwrites(&h.base, &writes, &segments, &every_proper_subset);
    let fresh_base = image_of(&vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())]);
    let fresh = crash_state_count_with_torn_in_place_overwrites(&fresh_base, &writes, &segments, &every_proper_subset);
    let shape: Vec<usize> = segments.iter().map(Vec::len).collect();
    let last = h.versions.last().expect("一版");
    println!("name=count h={} order={order_name} writes={} segment_sizes={shape:?} closed_form_two_state={} states_full_on_this_base={full} states_full_on_fresh_devices={fresh} old_last=i{}t{} old_generation={}",
        h.name, writes.len(), closed_form_state_count(&segments), last.instance.0, last.checkpoint_txg.0, last.checkpoint_txg.0 + 2);
    println!("name=kinds h={} order={order_name} {}", h.name, kinds_line(&writes));
}

fn mode_run(h: &History, order_name: &str) {
    let (writes, segments) = flow(h, order_of(order_name, h));
    let recorded = writes.len();
    let total = crash_state_count_with_torn_in_place_overwrites(&h.base, &writes, &segments, &every_proper_subset);
    assert!(total <= 1_000_000, "一次全量 {total} 态超过约 10^6，不跑");
    let judged_root_index = (0..recorded).rev().find(|i| writes[*i].kind == StepKind::RootRecordFua).expect("有根写");
    let without = [PublishedVersionWithoutAFile { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }];
    let versions = CrashPointReplayVersionTable::new(&h.versions, &without);
    let segment_of = |index: usize| segments.iter().position(|s| s.contains(&index)).unwrap_or(segments.len());
    let examples: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    let observe = |ordinal: u64, image: &CrashImage<'_>, report: &RecoveryReport, counts: &mut CrashPointReplayObserverCounts| {
        let first_withheld = (0..recorded).find(|i| !image.persisted[*i]).unwrap_or(recorded);
        let segment = segment_of(first_withheld);
        let torn: Vec<usize> = (recorded..image.writes.len()).filter(|i| image.persisted[*i]).collect();
        let verdict = judge(h, image, &report.outcome, || materialize(image));
        let key = format!("s{segment}__{}__{verdict}", if torn.is_empty() { "whole" } else { "torn" });
        counts.add(&key, 1);
        if examples.lock().expect("锁").insert(key.clone()) {
            let withheld: Vec<String> = (0..recorded).filter(|i| !image.persisted[*i]).map(|i| format!("{i}")).collect();
            let torn_text: Vec<String> = torn.iter().map(|i| format!("{}:{}@d{}o{}", i, image.writes[*i].kind.name(), image.writes[*i].device.0, image.writes[*i].offset.0)).collect();
            println!("name=example h={} order={order_name} ordinal={ordinal} key={key} withheld=[{}] torn_images_landed=[{}]", h.name, withheld.join(","), torn_text.join(","));
        }
    };
    let observer = CrashPointReplayObserver { name: "c656_attack_probe", extra_inputs_digest: [0u8; 32], observe: &observe };
    let item_name = format!("c656-attack.{}.{order_name}.rw{}", h.name, u8::from(writable_enabled()));
    let item = CrashPointReplayItem { name: &item_name, setting_file: "crates/c656-attack-probe/src/main.rs" };
    let started = std::time::Instant::now();
    let shape: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("name=run h={} order={order_name} writes={recorded} segment_sizes={shape:?} states_full={total} writable={}", h.name, writable_enabled());
    let outcome = enumerate_crash_states_with_expansion(item, &h.base, &writes, &segments, judged_root_index, versions, &every_proper_subset, &[observer]);
    let tally = match outcome {
        StageOutcome::Judged(tally) => tally,
        other => panic!("判段没交回整份结果：{other:?}"),
    };
    println!("name=done h={} order={order_name} states={} observed={} violations={} seconds={:.1}", h.name, tally.states, tally.observed_states, tally.violations, started.elapsed().as_secs_f64());
    println!("name=checker h={} order={order_name} {}", h.name, tally.checker_counts_by_invariant());
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    for (name, count) in tally.observer_counts.iter() {
        println!("name=count h={} order={order_name} count={count} {name}", h.name);
        let parts: Vec<&str> = name.split("__").collect();
        // 段、选择、恢复分项相加
        *by.entry(format!("seg={} choice={}", parts[0], parts[2])).or_insert(0) += count;
        *by.entry(format!("seg={} choice={} rec={}", parts[0], parts[2], parts[3])).or_insert(0) += count;
        if name.contains("__mixed") { *by.entry(format!("seg={} mixed", parts[0])).or_insert(0) += count; }
        if name.contains("__oldintact") { *by.entry(format!("seg={} oldintact_by_recovery", parts[0])).or_insert(0) += count; }
    }
    for (k, v) in by { println!("name=sum h={} order={order_name} count={v} {k}", h.name); }
}

/// 直接在旧池镜像上摆盘面：每块盘 [0, 根环基址) 清零那一步只落了一部分的几种形状（崩溃点重放的撕裂镜像只取「前一半新、后一半旧」那一种）。
fn mode_image(h: &History) {
    let spacing = u64::from(h.old.geometry.fixed_structure_slot_spacing);
    let generation = h.versions.last().expect("一版").checkpoint_txg.0 + 2;
    let newest_slot = generation % 2;
    let zero = |pool: &mut MemoryPool, device: u32, slot: u64| {
        let mut devices = devices_of(pool);
        let d = &mut devices.iter_mut().find(|(i, _)| i.0 == device).expect("盘").1;
        d.write_at(DeviceOffsetInBytes(slot * spacing), &vec![0u8; 4096], WriteDurability::Plain).expect("写");
        *pool = image_of(&devices);
    };
    let shapes: Vec<(&str, Vec<(u32, u64)>)> = vec![
        ("untouched", vec![]),
        ("d0_both_slots", vec![(0, 0), (0, 1)]),
        ("d1_both_slots", vec![(1, 0), (1, 1)]),
        ("d0_newest_only", vec![(0, newest_slot)]),
        ("d0_older_only", vec![(0, 1 - newest_slot)]),
        ("d1_newest_only", vec![(1, newest_slot)]),
        ("d1_older_only", vec![(1, 1 - newest_slot)]),
        ("both_newest_only", vec![(0, newest_slot), (1, newest_slot)]),
        ("both_older_only", vec![(0, 1 - newest_slot), (1, 1 - newest_slot)]),
        ("d0_newest_d1_both", vec![(0, newest_slot), (1, 0), (1, 1)]),
        ("d0_both_d1_newest", vec![(0, 0), (0, 1), (1, newest_slot)]),
        ("d0_older_d1_both", vec![(0, 1 - newest_slot), (1, 0), (1, 1)]),
    ];
    for (shape, zeroed) in shapes {
        let mut pool = h.base.clone();
        for (device, slot) in &zeroed { zero(&mut pool, *device, *slot); }
        let report = recover(&pool, JournalPolicy::Consult);
        let verdict = judge(h, &pool, &report.outcome, || pool.clone());
        println!("name=image h={} generation={generation} newest_slot={newest_slot} shape={shape} zeroed={zeroed:?} {verdict}", h.name);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("");
    let h = history(args.get(2).map(String::as_str).expect("历史名"));
    match mode {
        "count" => mode_count(&h, args.get(3).map(String::as_str).expect("写序")),
        "run" => mode_run(&h, args.get(3).map(String::as_str).expect("写序")),
        "image" => mode_image(&h),
        _ => panic!("用法：c656-attack-probe count|run|image <历史> [写序]"),
    }
}
