//! m3-w3-r2 攻方腿（Opus）原型，仓副本装置，不进 crates/。
//! 流在这里自己造（mkfs → 取号 → 暖机 → 第一个文件 → 建 inode 的发布 P），不从任何用例拷。
//! 崩溃状态照层 0 的枚举域取：`enumerate_layer0_selecting_versions_observing_each_state`，每一段都展开。
//! 每个崩溃状态：（可选的故障）→ 可写挂载 → 用户步（建 1 / 建 2 / 回退到环里第 k 新的根 / 重挂 / unlink）全部放开扫到给定长度，
//! 每一步之后读镜像：池级 checker 原样判，W3 各候选按影子历史判。影子历史只按「这一版接在哪一版后面」记，
//! 「接在哪一版后面」取实现真走的那一版（挂载交回的 effective_root、会话的 current、回退交回的新版本）。
#![allow(clippy::too_many_lines, clippy::too_many_arguments)]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
#[allow(dead_code, unused_imports, clippy::all)]
mod common;

mod shadow;

use std::collections::BTreeMap;
use std::time::Instant;

use common::FIXED_WRITE_TIME_SECONDS;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_harness::segments::FixedGeometry;

/// 盘宽：环境变量 W3_WIDTH（4g、384、256、240），默认 256。
fn width() -> HistoryDeviceWidth {
    match std::env::var("W3_WIDTH").as_deref().unwrap_or("256") {
        "4g" => HistoryDeviceWidth::FourGibibytes,
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
        other => panic!("W3_WIDTH 不认：{other}"),
    }
}
fn parameters() -> MakeFilesystemParameters { width().parameters() }
fn geometry() -> FixedGeometry { width().fixed_geometry() }
fn device_bytes() -> u64 { width().device_bytes() }

fn crash_state_devices(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool], stream: &SharedStream) -> Vec<(DeviceIdentity, Dev)> {
    base.devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = singlefs_harness::memory_pool::SparseBlockDevice::new(device_bytes(), PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            for (write, is_persisted) in writes.iter().zip(persisted) {
                if *is_persisted && write.device == *identity {
                    write.contents.apply_to(&mut device.image, write.offset);
                }
            }
            (*identity, singlefs_harness::RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

fn memory_pool_of_sparse_devices(devices: &[(DeviceIdentity, Dev)]) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: device_bytes(),
    }
}
use shadow::{Hist, Judge};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    enumerate_layer0_against_the_version_table_or_one_shard, full_expansion, layer0_state_count_with_torn_in_place_overwrites,
    Layer0EnumerationOutcome, Layer0ObserverCounts, Layer0Parallelism, Layer0StateObserving, Layer0VersionTable,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, InodeNumber};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{readable_roots, PoolReader};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, warm_up, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use singlefs_checker_tier::layer0_progress::Layer0Resume;

type Dev = singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>;

fn devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, Dev)> {
    crash_state_devices(image, &[], &[], &SharedStream::new())
}

fn records_of(version: &TransactionOutput) -> Vec<(u64, u64, u32)> {
    let mut out: Vec<(u64, u64, u32)> = version
        .inode_leaf_containers
        .iter()
        .flat_map(|leaf| leaf.contents.records.iter().map(|record| (record.inode, record.object_birth.0, record.nlink)))
        .collect();
    out.sort_unstable();
    out
}

fn key_of(version: &TransactionOutput) -> (u32, u64) {
    (version.root.instance.0, version.root.checkpoint_txg.0)
}

/// 前缀：mkfs → 取号 → 暖机 → 第一个文件 (1, 3)，再按 `creates_before` 在实例 1 里建 inode，最后录 P：建 `p_count` 个。
struct Prefix {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    shadows: BTreeMap<(u32, u64), Hist>,
    p_key: (u32, u64),
    p_new: Vec<(u64, u64)>,
}

fn empty_pool() -> MemoryPool {
    MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], device_bytes())
}

fn build_prefix(creates_before: &[u64], p_count: u64) -> Prefix {
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&empty_pool(), &[], &[], &stream);
    let params = parameters();
    let genesis = make_filesystem(&params, &mut devices).expect("mkfs");
    let mut allocator = singlefs_core::make_filesystem::allocator_after_make_filesystem(&params, &devices, &genesis);
    let content: Vec<u8> = (0..3000usize).map(|i| u8::try_from(i % 251).expect("小")).collect();
    let mut shadows = BTreeMap::new();
    let mut versions = Vec::new();
    let (start, end, p_key, p_new) = {
        let mut writer = PoolWriter::new(&params, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first = publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("第一个文件");
        let mut hist = Hist::default();
        hist.publish(first.root.checkpoint_txg.0, 0, first.inode_number_watermark(), &records_of(&first));
        shadows.insert(key_of(&first), hist);
        versions.push(PublishedVersion { instance, checkpoint_txg: first.root.checkpoint_txg, content: content.clone() });
        let mut current = first;
        for (index, count) in creates_before.iter().enumerate() {
            let next = publish_new_inodes(&mut writer, &mut allocator, &current, *count, FIXED_WRITE_TIME_SECONDS + 10 + index as u64, instance)
                .expect("前缀里建 inode");
            let h = shadows[&key_of(&current)].clone().after_publish(&current, &next, &records_of(&next));
            shadows.insert(key_of(&next), h);
            versions.push(PublishedVersion { instance, checkpoint_txg: next.root.checkpoint_txg, content: content.clone() });
            current = next;
        }
        let start = stream.operation_count();
        let p = publish_new_inodes(&mut writer, &mut allocator, &current, p_count, FIXED_WRITE_TIME_SECONDS + 100, instance)
            .expect("P：建 inode 的发布");
        let end = stream.operation_count();
        let p_records = records_of(&p);
        let p_new: Vec<(u64, u64)> = p_records.iter().filter(|r| r.1 == p.root.checkpoint_txg.0).map(|r| (r.0, r.1)).collect();
        let h = shadows[&key_of(&current)].clone().after_publish(&current, &p, &p_records);
        shadows.insert(key_of(&p), h);
        versions.push(PublishedVersion { instance, checkpoint_txg: p.root.checkpoint_txg, content: content.clone() });
        (start, end, key_of(&p), p_new)
    };
    let operations = stream.retained_operations();
    let mut base = empty_pool();
    base.apply(&operations[..start]);
    let (writes, segments) = writes_and_segments(&operations[start..end], &geometry());
    let judged_root_index = writes
        .iter()
        .enumerate()
        .filter(|(_, w)| w.kind == StepKind::RootRecordFua)
        .map(|(i, _)| i)
        .last()
        .expect("P 有一条根槽写");
    Prefix { base, writes, segments, judged_root_index, versions, shadows, p_key, p_new }
}

use singlefs_core::mount::{ParametersAndDeviceTableOfTheMount, ShadowLedger};

#[derive(Clone)]
struct Sim {
    image: MemoryPool,
    allocator: PoolAllocator,
    current: PoolVersion,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    pdt: ParametersAndDeviceTableOfTheMount,
    shadows: BTreeMap<(u32, u64), Hist>,
    /// P 被这次恢复抛弃时，P 发的 (号, 出生代)。
    abandoned_p: Vec<(u64, u64)>,
}

fn version_of(current: &PoolVersion) -> &TransactionOutput {
    current.version_of_trees().expect("有树的一版")
}

fn session_of(sim: &Sim) -> MountedSession {
    MountedSession {
        allocator: sim.allocator.clone(),
        current: sim.current.clone(),
        instance: sim.instance,
        shadow_ledger: sim.shadow_ledger,
        parameters_and_device_table: sim.pdt.clone(),
    }
}

#[derive(Default, Debug)]
struct Tally {
    states: u64,
    mount_ok: u64,
    mount_refused: BTreeMap<String, u64>,
    p_abandoned_states: u64,
    p_abandoned_with_readable_root_states: u64,
    restore_clobbered: u64,
    images: u64,
    legal_images: u64,
    illegal_images: u64,
    cross_abandoned_reissue_images: u64,
    rollback_jumps: u64,
    rollbacks: u64,
    ops_refused: BTreeMap<String, u64>,
    shadow_missing: u64,
    fp: [u64; 13],
    miss: [u64; 13],
    fp_example: Vec<Option<String>>,
    real_red_legal: BTreeMap<String, u64>,
    real_red_illegal: BTreeMap<String, u64>,
    real_first_red: BTreeMap<String, String>,
}

fn mount_sim(image: MemoryPool, shadows: BTreeMap<(u32, u64), Hist>, p_key: (u32, u64), p_new: &[(u64, u64)]) -> Result<(Sim, (u32, u64)), String> {
    let mut devices = devices_of(&image);
    let mounted = mount_writable(&parameters(), &mut devices).map_err(|e| {
        let s = format!("{e:?}");
        s.split(|c: char| c == '(' || c == ' ' || c == '{').next().unwrap_or("?").to_string()
    })?;
    let effective = (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0);
    let (_output, session) = MountedSession::of_the_mount(mounted);
    let image = memory_pool_of_sparse_devices(&devices);
    let mut shadows = shadows;
    let base = shadows.get(&effective).cloned();
    let current_key = key_of(version_of(&session.current));
    let abandoned_p = if effective.1 < p_key.1 { p_new.to_vec() } else { Vec::new() };
    match base {
        Some(h) => {
            shadows.insert(current_key, h);
        }
        None => return Err(format!("shadow_missing_for_{}_{}", effective.0, effective.1)),
    }
    Ok((
        Sim {
            image,
            allocator: session.allocator,
            current: session.current,
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            pdt: session.parameters_and_device_table,
            shadows,
            abandoned_p,
        },
        effective,
    ))
}

const ALPHABET: [char; 7] = ['c', 'C', 'u', '1', '2', '3', 'm'];

fn apply_op(sim: &Sim, op: char, step: u64, tally: &mut Tally) -> Option<Sim> {
    let params = parameters();
    let mut devices = devices_of(&sim.image);
    let previous = version_of(&sim.current).clone();
    let prev_hist = sim.shadows.get(&key_of(&previous)).cloned()?;
    let mut next = sim.clone();
    match op {
        'c' | 'C' | 'u' => {
            let mut session = session_of(sim);
            let change = match op {
                'c' => UserChange::NewInodes { count: 1, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1000 + step },
                'C' => UserChange::NewInodes { count: 2, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1000 + step },
                _ => {
                    let victim = records_of(&previous).iter().filter(|r| r.0 >= 2 && r.2 > 0).map(|r| r.0).max()?;
                    UserChange::Unlink { inode: InodeNumber(victim), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1000 + step }
                }
            };
            match session.publish_user_change(&mut devices, change) {
                Ok(_) => {}
                Err(e) => {
                    let s = format!("{e:?}");
                    *tally.ops_refused.entry(format!("{op}:{}", s.split(|c: char| c == '(' || c == ' ' || c == '{').next().unwrap_or("?"))).or_insert(0) += 1;
                    return None;
                }
            }
            let new = version_of(&session.current).clone();
            let h = prev_hist.after_publish(&previous, &new, &records_of(&new));
            next.shadows.insert(key_of(&new), h);
            next.allocator = session.allocator;
            next.current = session.current;
        }
        '1' | '2' | '3' => {
            let k = usize::from(u8::try_from(op).expect("ascii") - b'0');
            let mut roots = readable_roots(&sim.image, &params.region_devices, &params.geometry, &params.filesystem_identifier);
            roots.sort_by_key(|r| std::cmp::Reverse((r.checkpoint_txg, r.instance)));
            roots.dedup_by_key(|r| (r.checkpoint_txg, r.instance));
            let others: Vec<_> = roots.into_iter().filter(|r| (r.instance.0, r.checkpoint_txg.0) != key_of(&previous)).collect();
            let target = others.get(k - 1)?;
            let mut allocator = sim.allocator.clone();
            let mut current = previous.clone();
            let target_key = RollbackTarget { instance: target.instance, checkpoint_txg: target.checkpoint_txg };
            if let Err(e) = roll_back_by_a_forward_publish(&params, &mut devices, &mut allocator, &mut current, target_key) {
                let s = format!("{e:?}");
                *tally.ops_refused.entry(format!("{op}:{}", s.split(|c: char| c == '(' || c == ' ' || c == '{').next().unwrap_or("?"))).or_insert(0) += 1;
                return None;
            }
            tally.rollbacks += 1;
            if current.inode_number_watermark() > previous.inode_number_watermark() {
                tally.rollback_jumps += 1;
            }
            let h = prev_hist.after_rollback(&previous, &current, target.checkpoint_txg.0, &records_of(&current));
            next.shadows.insert(key_of(&current), h);
            next.allocator = allocator;
            next.current = PoolVersion::of_a_version_of_trees(current);
        }
        'm' => {
            let image = memory_pool_of_sparse_devices(&devices);
            match mount_sim(image, sim.shadows.clone(), (0, u64::MAX), &[]) {
                Ok((mut remounted, _)) => {
                    remounted.abandoned_p = sim.abandoned_p.clone();
                    return Some(remounted);
                }
                Err(e) => {
                    *tally.ops_refused.entry(format!("m:{e}")).or_insert(0) += 1;
                    return None;
                }
            }
        }
        _ => unreachable!(),
    }
    next.image = memory_pool_of_sparse_devices(&devices);
    Some(next)
}

fn judge(sim: &Sim, label: &str, tally: &mut Tally, verbose: bool) {
    tally.images += 1;
    let version = version_of(&sim.current);
    let Some(h) = sim.shadows.get(&key_of(version)) else {
        tally.shadow_missing += 1;
        return;
    };
    let records = records_of(version);
    let tree: Vec<(u64, u64)> = records.iter().map(|r| (r.0, r.1)).collect();
    let wm = version.inode_number_watermark();
    let txg = version.root.checkpoint_txg.0;
    let red = Judge::verdicts(h, wm, txg, &tree);
    let legal = !h.reissue_on_the_timeline();
    let cross = tree.iter().any(|&(n, b)| sim.abandoned_p.iter().any(|&(pn, pb)| pn == n && pb != b));
    if cross {
        tally.cross_abandoned_reissue_images += 1;
    }
    let real: Vec<(String, String)> = check_pool_image(&sim.image)
        .into_iter()
        .filter_map(|(inv, v)| match v {
            InvariantVerdict::Violated(d) => Some((inv.to_string(), d)),
            _ => None,
        })
        .collect();
    if legal {
        tally.legal_images += 1;
    } else {
        tally.illegal_images += 1;
    }
    for (inv, detail) in &real {
        let map = if legal { &mut tally.real_red_legal } else { &mut tally.real_red_illegal };
        *map.entry(inv.clone()).or_insert(0) += 1;
        tally.real_first_red.entry(inv.clone()).or_insert_with(|| format!("{label} :: {}", detail.chars().take(300).collect::<String>()));
    }
    for (i, r) in red.iter().enumerate() {
        if legal && *r {
            tally.fp[i] += 1;
            if tally.fp_example[i].is_none() {
                tally.fp_example[i] = Some(format!(
                    "{label} wm={wm} txg={txg} tree={tree:?} rows_create={:?} rows_all={:?} log={:?} lineage={:?} issued={:?}",
                    h.rows_create, h.rows_all, h.log, h.lineage, h.issued
                ));
            }
        }
        if !legal && !*r {
            tally.miss[i] += 1;
        }
    }
    if verbose {
        let names: Vec<&str> = shadow::NAMES.iter().zip(red).filter(|(_, r)| *r).map(|(n, _)| *n).collect();
        let realn: Vec<&str> = real.iter().map(|(i, _)| i.as_str()).collect();
        println!(
            "W3R2TRACE {label} key={:?} wm={wm} tree={tree:?} legal={legal} cross={cross} rows_create={:?} rows_all={:?} log={:?} lineage={:?} red=[{}] real=[{}]",
            key_of(version), h.rows_create, h.rows_all, h.log, h.lineage, names.join(","), realn.join(",")
        );
    }
}

fn explore(sim: &Sim, script: &mut String, depth: usize, tally: &mut Tally, state_label: &str, verbose: bool) {
    if depth == 0 {
        return;
    }
    for op in ALPHABET {
        let step = u64::try_from(script.len()).expect("小");
        let Some(next) = apply_op(sim, op, step, tally) else { continue };
        script.push(op);
        judge(&next, &format!("{state_label} script={script}"), tally, verbose);
        explore(&next, script, depth - 1, tally, state_label, verbose);
        script.pop();
    }
}

fn parse_list(s: &str) -> Vec<u64> {
    if s == "-" {
        return Vec::new();
    }
    s.split(',').map(|x| x.parse().expect("数")).collect()
}

fn pool_of_crash_image(image: &CrashImage<'_>) -> MemoryPool {
    let devices = crash_state_devices(image.base, image.writes, &image.persisted, &SharedStream::new());
    memory_pool_of_sparse_devices(&devices)
}

/// E-B 的故障：P 的根槽与 P 写的单元整次挂载读回全 0（读回全 0 之外不动盘）；挂载之后那几处还是全 0 的（挂载没写过）原样写回。
fn hidden_ranges(writes: &[RetainedWrite], persisted: &[bool], p_write_count: usize) -> Vec<(DeviceIdentity, DeviceOffsetInBytes, u64)> {
    writes
        .iter()
        .zip(persisted)
        .take(p_write_count)
        .filter(|(w, p)| **p && matches!(w.kind, StepKind::UnitWrite | StepKind::RootRecordFua))
        .map(|(w, _)| (w.device, w.offset, w.length_in_bytes()))
        .collect()
}

fn counter_name(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect()
}

fn flush(prefix: &str, tally: &Tally, counts: &mut Layer0ObserverCounts) {
    let mut add = |name: &str, value: u64| {
        if value > 0 {
            counts.add(&counter_name(&format!("{prefix}{name}")), value);
        }
    };
    add("states", tally.states);
    add("mount_ok", tally.mount_ok);
    for (k, v) in &tally.mount_refused {
        add(&format!("mount_refused_{k}"), *v);
    }
    add("p_abandoned_states", tally.p_abandoned_states);
    add("p_abandoned_with_readable_root_states", tally.p_abandoned_with_readable_root_states);
    add("restore_clobbered", tally.restore_clobbered);
    add("images", tally.images);
    add("legal_images", tally.legal_images);
    add("illegal_images", tally.illegal_images);
    add("cross_abandoned_reissue_images", tally.cross_abandoned_reissue_images);
    add("rollbacks", tally.rollbacks);
    add("rollback_jumps", tally.rollback_jumps);
    add("shadow_missing", tally.shadow_missing);
    for (k, v) in &tally.ops_refused {
        add(&format!("op_refused_{k}"), *v);
    }
    for (i, name) in shadow::NAMES.iter().enumerate() {
        add(&format!("fp_{name}"), tally.fp[i]);
        add(&format!("miss_{name}"), tally.miss[i]);
    }
    for (k, v) in &tally.real_red_legal {
        add(&format!("real_red_legal_{k}"), *v);
    }
    for (k, v) in &tally.real_red_illegal {
        add(&format!("real_red_illegal_{k}"), *v);
    }
}

static PRINTED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// 每个候选的第一个误报例、每条真 checker 违例的第一例各打一行（各工作线程共用，先到先打）。
fn print_examples_once(exp: &str, tally: &Tally) {
    let mut printed = PRINTED.lock().expect("锁");
    for (i, name) in shadow::NAMES.iter().enumerate() {
        if let Some(e) = &tally.fp_example[i] {
            let key = format!("fp {exp} {name}");
            if !printed.contains(&key) {
                println!("W3R2FPEXAMPLE exp={exp} candidate={name} {e}");
                printed.push(key);
            }
        }
    }
    for (inv, first) in &tally.real_first_red {
        let key = format!("real {exp} {inv}");
        if !printed.contains(&key) {
            println!("W3R2REALFIRST exp={exp} {inv} {first}");
            printed.push(key);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map_or("segments", String::as_str);
    let creates_before = parse_list(args.get(1).map_or("-", String::as_str));
    let p_count: u64 = args.get(2).map_or(2, |s| s.parse().expect("P 建几个"));
    let prefix = build_prefix(&creates_before, p_count);
    let lens: Vec<usize> = prefix.segments.iter().map(Vec::len).collect();
    let closed = closed_form_state_count(&prefix.segments);
    let torn = layer0_state_count_with_torn_in_place_overwrites(&prefix.base, &prefix.writes, &prefix.segments, &full_expansion);
    println!(
        "W3R2STREAM width={:?} creates_before={creates_before:?} p_count={p_count} p_key={:?} p_new={:?} writes={} segments={lens:?} closed_form={closed} layer0_states_with_torn_overwrites={torn}",
        width(), prefix.p_key, prefix.p_new, prefix.writes.len()
    );
    if mode == "segments" {
        return;
    }
    let experiment = args.get(3).map_or("A", String::as_str).to_string();
    let scripts_arg = args.get(4).map_or("c".to_string(), String::clone);
    let depth: usize = scripts_arg.strip_prefix("dfs").map_or(0, |d| d.parse().expect("深"));
    let scripts: Vec<String> = if depth > 0 { Vec::new() } else { scripts_arg.split(',').map(str::to_string).collect() };
    let verbose_states: Vec<u64> = args.get(5).map_or(Vec::new(), |s| parse_list(s));
    let (shard, shards): (u64, u64) = std::env::var("W3_SHARD").ok().map_or((0, 1), |s| {
        let (a, b) = s.split_once('/').expect("W3_SHARD=i/n");
        (a.parse().expect("i"), b.parse().expect("n"))
    });
    assert!(torn <= 1_100_000, "这一段历史的层 0 状态数 {torn} 超过约 10⁶，不跑");
    let label = format!("width={:?} exp={experiment} creates_before={} p_count={p_count} scripts={scripts_arg} shard={shard}/{shards}", width(), args.get(1).map_or("-", String::as_str));
    let started = Instant::now();
    let p_write_count = prefix.writes.len();
    let state_counter = std::sync::atomic::AtomicU64::new(0);
    let observer = |image: &CrashImage<'_>, _report: &singlefs_core::recovery::RecoveryReport, counts: &mut Layer0ObserverCounts| {
        let _ = state_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let index = state_index_of(image);
        let only = !verbose_states.is_empty() && std::env::var("W3_ONLY_VERBOSE").is_ok();
        let selected = (only && verbose_states.contains(&index)) || (!only && splitmix(index) % shards == shard);
        // experiment 是「A:脚本;B:脚本」或单个字母（脚本取第 4 个参数）。
        for part in experiment.split(';') {
            let (exp, exp_scripts): (&str, Vec<String>) = match part.split_once(':') {
                Some((e, list)) => (e, list.split(',').map(str::to_string).collect()),
                None => (part, scripts.clone()),
            };
            let mut tally = Tally { fp_example: vec![None; shadow::NAMES.len()], ..Tally::default() };
            tally.states += 1;
            if selected {
                observe_one_state(&prefix, image, index, exp, depth, &exp_scripts, verbose_states.contains(&index), p_write_count, &mut tally);
            }
            print_examples_once(exp, &tally);
            flush(&format!("w3{}_", exp.to_ascii_lowercase()), &tally, counts);
        }
    };
    let outcome = enumerate_layer0_against_the_version_table_or_one_shard(
        &prefix.base,
        &prefix.writes,
        &prefix.segments,
        prefix.judged_root_index,
        Layer0VersionTable::with_a_file_only(&prefix.versions),
        &full_expansion,
        Layer0Parallelism::from_environment(),
        Layer0StateObserving::OnTheWorkerThreads(&observer),
        &Layer0Resume::from_environment("w3r2"),
    );
    let layer0 = match outcome {
        Layer0EnumerationOutcome::WholeStream(layer0) => layer0,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => {
            println!("W3R2SHARDDONE {label} {written:?}");
            println!("W3R2DONE {label} elapsed_s={:.1}", started.elapsed().as_secs_f64());
            return;
        }
    };
    println!(
        "W3R2LAYER0 {label} states={} observed_states={} violations={} root_persisted_states={} failed_states={} checker_violated={:?} first_violation={:?}",
        layer0.states, layer0.observed_states, layer0.violations, layer0.root_persisted_states, layer0.failed_states, layer0.checker_violated_states,
        layer0.first_violation
    );
    for (name, value) in layer0.observer_counts.iter() {
        println!("W3R2COUNT {label} {name}={value}");
    }
    println!("W3R2DONE {label} elapsed_s={:.1}", started.elapsed().as_secs_f64());
}

fn observe_one_state(
    prefix: &Prefix,
    image: &CrashImage<'_>,
    index: u64,
    experiment: &str,
    depth: usize,
    scripts: &[String],
    verbose: bool,
    p_write_count: usize,
    tally: &mut Tally,
) {
    let mut pool = pool_of_crash_image(image);
    let hidden = if experiment == "B" { hidden_ranges(image.writes, &image.persisted, p_write_count) } else { Vec::new() };
    let saved: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = hidden
        .iter()
        .map(|&(d, o, l)| (d, o, PoolReader::read(&pool, d, o, usize::try_from(l).expect("长")).expect("读")))
        .collect();
    for (d, o, bytes) in &saved {
        pool.devices.get_mut(d).expect("盘").write(*o, &vec![0u8; bytes.len()]);
    }
    match mount_sim(pool, prefix.shadows.clone(), prefix.p_key, &prefix.p_new) {
        Ok((mut sim, effective)) => {
            tally.mount_ok += 1;
            for (d, o, bytes) in &saved {
                let now = PoolReader::read(&sim.image, *d, *o, bytes.len()).expect("读");
                if now.iter().all(|b| *b == 0) {
                    sim.image.devices.get_mut(d).expect("盘").write(*o, bytes);
                } else {
                    tally.restore_clobbered += 1;
                }
            }
            if effective.1 < prefix.p_key.1 {
                tally.p_abandoned_states += 1;
                let params = parameters();
                let roots = readable_roots(&sim.image, &params.region_devices, &params.geometry, &params.filesystem_identifier);
                if roots.iter().any(|r| (r.instance.0, r.checkpoint_txg.0) == prefix.p_key) {
                    tally.p_abandoned_with_readable_root_states += 1;
                }
            }
            let state_label = format!("state={index} effective={effective:?}");
            judge(&sim, &format!("{state_label} script="), tally, verbose);
            if depth > 0 {
                explore(&sim, &mut String::new(), depth, tally, &state_label, verbose);
            }
            for script in scripts {
                let mut now = sim.clone();
                for (step, op) in script.chars().enumerate() {
                    let Some(next) = apply_op(&now, op, u64::try_from(step).expect("小"), tally) else { break };
                    judge(&next, &format!("{state_label} script={}", &script[..=step]), tally, verbose);
                    now = next;
                }
            }
        }
        Err(e) => {
            *tally.mount_refused.entry(e).or_insert(0) += 1;
        }
    }
}

/// 状态的序号：持久集合按位读成一个数（与枚举次序无关，只为分批、复现时点名）。前 64 次写按位，写表不超过 64 条。
fn state_index_of(image: &CrashImage<'_>) -> u64 {
    image.persisted.iter().enumerate().take(64).fold(0u64, |acc, (i, p)| if *p { acc | (1u64 << i) } else { acc })
}

fn splitmix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
