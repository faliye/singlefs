//! 各模式：acq（取号流的崩溃状态 × 每盘再坏一槽）、rollback（取号写报错回卷流的崩溃状态）、alpha（离线出路）、count（只数状态）。
use crate::common_admission_pool::{fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices, plain_devices_on, OVERWRITE_BYTES, content_of};
use crate::{probe_devices_on, starting_point, ProbeDevice};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, visit_crash_state_range_of_the_read_back_flow_with_slices,
    CrashPointReplaySegmentExpansion,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{next_instance_generation_to_acquire, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub const STATE_LIMIT_PER_RUN: u64 = 1_000_000;

pub fn arm_name() -> String {
    format!(
        "acquisition={} rollback={}",
        std::env::var("SINGLEFS_PROBE_ACQUISITION").unwrap_or_else(|_| "one_round".into()),
        std::env::var("SINGLEFS_PROBE_ROLLBACK").unwrap_or_else(|_| "per_written".into())
    )
}

pub fn threads() -> usize {
    std::env::var("SINGLEFS_CRASH_POINT_REPLAY_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(1)
}

pub fn spacing() -> u64 {
    u64::from(parameters_of_the_384_slot_small_devices().geometry.fixed_structure_slot_spacing)
}

pub fn reds(image: &MemoryPool) -> Vec<String> {
    let mut names: Vec<String> = check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then(|| invariant.to_string()))
        .collect();
    names.sort();
    names.dedup();
    names
}

pub fn variant_name(debug: &str) -> String {
    debug.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or("").to_string()
}

/// 一块盘两槽：每槽 `g<世代>i<实例>t<tail>` 或 `bad`。
pub fn sysconf_signature(image: &MemoryPool) -> String {
    let spacing = spacing();
    image
        .devices
        .keys()
        .map(|identity| {
            let slots: Vec<String> = [0, spacing]
                .iter()
                .map(|offset| {
                    let bytes = image.devices[identity].read(DeviceOffsetInBytes(*offset), 4096);
                    match SystemConfiguration::parse_slot(&bytes) {
                        Ok(slot) => format!(
                            "g{}i{}t{}",
                            slot.quantities.slot_generation, slot.quantities.journal_instance.0, slot.quantities.journal_tail
                        ),
                        Err(_) => "bad".to_string(),
                    }
                })
                .collect();
            format!("d{}[{}]", identity.0, slots.join(","))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

pub fn mount_outcome<D: BlockDevice>(parameters: &MakeFilesystemParameters, devices: &mut Vec<(DeviceIdentity, D)>) -> Result<u32, String> {
    match mount_writable(parameters, devices) {
        Ok(mounted) => Ok(mounted.output.instance.0),
        Err(error) => Err(variant_name(&format!("{error:?}"))),
    }
}

pub fn formula<D: BlockDevice>(parameters: &MakeFilesystemParameters, devices: &mut [(DeviceIdentity, D)]) -> u32 {
    let writer = PoolWriter::new(parameters, devices);
    next_instance_generation_to_acquire(&writer).map_or(u32::MAX, |instance| instance.0)
}

/// 录一次可写挂载（`publish_after` 时挂上之后经会话再覆盖写一次），交回录制流、取到的号、挂载的结局。
pub fn record_mount(base: &MemoryPool, publish_after: bool, fail_sysconf_write_from: Option<(u64, bool)>) -> (Vec<singlefs_harness::RetainedOperation>, Result<u32, String>) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<ProbeDevice>)> = probe_devices_on(base, spacing())
        .into_iter()
        .map(|(identity, mut device)| {
            if let Some((first, only_once)) = fail_sysconf_write_from {
                device.fail_sysconf_write_from = Some(first);
                device.only_once = only_once;
            }
            (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
        })
        .collect();
    let outcome = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => {
            let instance = mounted.output.instance.0;
            if publish_after {
                let (_output, mut session) = MountedSession::of_the_mount(mounted);
                let content = content_of(OVERWRITE_BYTES, 99);
                session
                    .publish_user_change(&mut devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: 1_788_000_999 }))
                    .expect("挂上之后会话覆盖写一次");
            }
            Ok(instance)
        }
        Err(error) => Err(format!("{error:?}").chars().take(220).collect()),
    };
    (stream.retained_operations(), outcome)
}

/// 按崩溃点重放的计划逐个状态交给 `visit`（多线程，序号段切给每个线程）。交回状态总数。
pub fn for_each_state<R: Send + Default>(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    visit: &(dyn Fn(u64, &MemoryPool, &[RetainedWrite], &[bool], &mut R) + Sync),
) -> (u64, Vec<R>) {
    let expansion = |_: usize, _: &[usize]| CrashPointReplaySegmentExpansion::EveryProperSubset;
    let total = crash_state_count_with_torn_in_place_overwrites(base, writes, segments, &expansion);
    let limit = std::env::var("PROBE_STATE_LIMIT").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(total).min(total);
    let threads = threads().max(1) as u64;
    let chunk = limit.div_ceil(threads).max(1);
    let results = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for thread in 0..threads {
            let start = thread * chunk;
            let end = ((thread + 1) * chunk).min(limit);
            if start >= end {
                continue;
            }
            let results = &results;
            scope.spawn(move || {
                let mut accumulated = R::default();
                let slices = vec![None; segments.len()];
                visit_crash_state_range_of_the_read_back_flow_with_slices(base, writes, segments, &expansion, &slices, start..end, &mut |ordinal, _publish, image| {
                    let pool = materialize(image.base, image.writes, &image.persisted);
                    visit(ordinal, &pool, image.writes, &image.persisted, &mut accumulated);
                });
                results.lock().expect("锁").push(accumulated);
            });
        }
    });
    (limit, results.into_inner().expect("锁"))
}

#[derive(Default)]
pub struct Tally {
    pub states: u64,
    pub footprint_states: u64,
    pub outcomes: BTreeMap<String, BTreeMap<String, u64>>,
    pub raw_reds: BTreeMap<String, u64>,
    pub post_reds: BTreeMap<String, u64>,
    pub examples: BTreeMap<String, String>,
}

fn bump(map: &mut BTreeMap<String, u64>, key: &str) {
    *map.entry(key.to_string()).or_insert(0) += 1;
}

fn merge(tallies: Vec<Tally>) -> Tally {
    let mut all = Tally::default();
    for tally in tallies {
        all.states += tally.states;
        all.footprint_states += tally.footprint_states;
        for (arm, outcomes) in tally.outcomes {
            let into = all.outcomes.entry(arm).or_default();
            for (outcome, count) in outcomes {
                *into.entry(outcome).or_insert(0) += count;
            }
        }
        for (name, count) in tally.raw_reds {
            *all.raw_reds.entry(name).or_insert(0) += count;
        }
        for (name, count) in tally.post_reds {
            *all.post_reds.entry(name).or_insert(0) += count;
        }
        // 几个线程的例子取序号最小的那一个：输出与线程的先后无关。
        let ordinal_of = |example: &str| -> u64 {
            example.strip_prefix("ordinal=").and_then(|rest| rest.split(' ').next()).and_then(|n| n.parse().ok()).unwrap_or(u64::MAX)
        };
        for (key, example) in tally.examples {
            let keep = all.examples.get(&key).is_none_or(|kept| ordinal_of(&example) < ordinal_of(kept));
            if keep {
                all.examples.insert(key, example);
            }
        }
    }
    all
}

pub struct Prefix {
    pub name: &'static str,
    pub overwrites: usize,
    pub remount_first: bool,
    pub publish_after: bool,
}

pub const PREFIXES: [Prefix; 4] = [
    Prefix { name: "k0", overwrites: 0, remount_first: false, publish_after: false },
    Prefix { name: "k2", overwrites: 2, remount_first: false, publish_after: false },
    Prefix { name: "r1", overwrites: 1, remount_first: true, publish_after: false },
    Prefix { name: "k0_pub", overwrites: 0, remount_first: false, publish_after: true },
];

fn checker_on() -> bool {
    std::env::var("PROBE_CHECKER").as_deref() != Ok("0")
}

/// 一个状态之后「每盘再坏一槽」的八条故障臂与不加故障那一臂。
fn judge_after_faults(ordinal: u64, pool: &MemoryPool, footprint: bool, acquired: u32, tally: &mut Tally) {
    let parameters = parameters_of_the_384_slot_small_devices();
    let spacing = spacing();
    let mut devices = plain_devices_on(pool);
    let outcome = mount_outcome(&parameters, &mut devices);
    let key = match &outcome {
        Ok(instance) if footprint && *instance <= acquired => "COLLIDE".to_string(),
        Ok(_) => "mounted".to_string(),
        Err(name) => name.clone(),
    };
    bump(tally.outcomes.entry("none".into()).or_default(), &key);
    if key == "COLLIDE" || key != "mounted" {
        tally.examples.entry(format!("none/{key}")).or_insert_with(|| format!("ordinal={ordinal} sysconf={}", sysconf_signature(pool)));
    }
    if outcome.is_ok() && checker_on() {
        let after = MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: pool.device_size_in_bytes };
        for name in reds(&after) {
            bump(&mut tally.post_reds, &name);
        }
    }
    let full = std::env::var("PROBE_FULL_ARMS").as_deref() == Ok("1");
    let mut arms: Vec<(&str, Vec<(usize, u64)>)> = Vec::new();
    for kind in ["flip", "read_error"] {
        for (slot_zero, slot_one) in [(0, 0), (0, spacing), (spacing, 0), (spacing, spacing)] {
            arms.push((kind, vec![(0, slot_zero), (1, slot_one)]));
        }
    }
    for disk in [0usize, 1] {
        for offset in [0, spacing] {
            arms.push(("fsid", vec![(disk, offset)]));
        }
    }
    for (kind, places) in arms {
        {
            let arm = format!(
                "{kind}/{}",
                places.iter().map(|(disk, offset)| format!("d{disk}s{}", offset / spacing.max(1))).collect::<Vec<_>>().join("/")
            );
            let make = || {
                let mut devices = probe_devices_on(pool, spacing);
                for (index, offset) in places.iter().copied() {
                    let device = &mut devices[index].1;
                    if kind == "fsid" {
                        let mut bytes = vec![0u8; 4096];
                        device.inner.read_at(DeviceOffsetInBytes(offset), &mut bytes).expect("读那一槽");
                        if let Ok(mut slot) = SystemConfiguration::parse_slot(&bytes) {
                            slot.immutable.filesystem_identifier[0] ^= 0x5A;
                            device.inner.write_at(DeviceOffsetInBytes(offset), &slot.to_slot(), WriteDurability::Plain).expect("写别池的槽");
                        } else {
                            bytes[100] ^= 0xFF;
                            device.inner.write_at(DeviceOffsetInBytes(offset), &bytes, WriteDurability::Plain).expect("写回");
                        }
                    } else if kind == "flip" {
                        let mut bytes = vec![0u8; 512];
                        device.inner.read_at(DeviceOffsetInBytes(offset), &mut bytes).expect("读那一扇区");
                        bytes[100] ^= 0xFF;
                        device.inner.write_at(DeviceOffsetInBytes(offset), &bytes, WriteDurability::Plain).expect("写回翻坏的");
                    } else {
                        device.unreadable.push(offset);
                    }
                }
                devices
            };
            let next = formula(&parameters, &mut make());
            let key = if footprint && next <= acquired || full {
                match mount_outcome(&parameters, &mut make()) {
                    Ok(instance) if footprint && instance <= acquired => "COLLIDE".to_string(),
                    Ok(_) => "mounted".to_string(),
                    Err(name) => name,
                }
            } else {
                "formula_above_acquired".to_string()
            };
            bump(tally.outcomes.entry(arm.clone()).or_default(), &key);
            if key == "COLLIDE" {
                tally.examples.entry(format!("{arm}/COLLIDE")).or_insert_with(|| format!("ordinal={ordinal} sysconf={}", sysconf_signature(pool)));
            }
        }
    }
}

fn print_tally(mode: &str, prefix: &str, tally: &Tally) {
    let arm = arm_name();
    println!("probe={mode} prefix={prefix} {arm} states_judged={} footprint_states={}", tally.states, tally.footprint_states);
    for (fault, outcomes) in &tally.outcomes {
        println!("probe={mode} prefix={prefix} {arm} fault={fault} outcomes={outcomes:?}");
    }
    println!("probe={mode} prefix={prefix} {arm} raw_checker_red_states_by_invariant={:?}", tally.raw_reds);
    println!("probe={mode} prefix={prefix} {arm} post_mount_checker_red_states_by_invariant={:?}", tally.post_reds);
    for (key, example) in &tally.examples {
        println!("probe={mode} prefix={prefix} {arm} example={key} {example}");
    }
}

fn counts(base: &MemoryPool, operations: &[singlefs_harness::RetainedOperation]) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, u64, u64) {
    let (writes, segments) = writes_and_segments(operations, &fixed_geometry_of_the_384_slot_small_devices());
    let expansion = |_: usize, _: &[usize]| CrashPointReplaySegmentExpansion::EveryProperSubset;
    let torn = crash_state_count_with_torn_in_place_overwrites(base, &writes, &segments, &expansion);
    let closed = closed_form_state_count(&segments);
    (writes, segments, closed, torn)
}

fn is_sysconf(write: &RetainedWrite) -> bool {
    write.kind == singlefs_harness::segments::StepKind::SystemConfigurationSlot
}

fn segment_shape(writes: &[RetainedWrite], segments: &[Vec<usize>]) -> String {
    segments
        .iter()
        .map(|segment| {
            let sysconf = segment.iter().filter(|index| is_sysconf(&writes[**index])).count();
            format!("{}s{}", segment.len(), sysconf)
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn mode_acq(prefix_names: &[String]) {
    for prefix in PREFIXES.iter().filter(|prefix| prefix_names.iter().any(|name| name == prefix.name)) {
        let (base, _root) = starting_point(prefix.overwrites, prefix.remount_first);
        let (operations, outcome) = record_mount(&base, prefix.publish_after, None);
        let acquired = outcome.expect("录的那一次可写挂载做成");
        let (writes, segments, closed, torn) = counts(&base, &operations);
        let sysconf_writes = writes.iter().filter(|write| is_sysconf(write)).count();
        println!(
            "probe=acq_counts prefix={} {} acquired={acquired} writes={} sysconf_writes={sysconf_writes} segments={} closed_form_states={closed} states_with_torn={torn} segment_shape={}",
            prefix.name, arm_name(), writes.len(), segments.len(), segment_shape(&writes, &segments)
        );
        if torn > STATE_LIMIT_PER_RUN || std::env::var("PROBE_COUNT_ONLY").as_deref() == Ok("1") {
            println!("probe=acq_skipped prefix={} states_with_torn={torn}", prefix.name);
            continue;
        }
        let started = std::time::Instant::now();
        let (judged, tallies) = for_each_state(&base, &writes, &segments, &|ordinal, pool, all_writes, persisted, tally: &mut Tally| {
            tally.states += 1;
            let footprint = all_writes.iter().zip(persisted).any(|(write, p)| *p && !is_sysconf(write));
            if footprint {
                tally.footprint_states += 1;
            }
            if checker_on() {
                for name in reds(pool) {
                    bump(&mut tally.raw_reds, &name);
                }
            }
            judge_after_faults(ordinal, pool, footprint, acquired, tally);
        });
        let tally = merge(tallies);
        eprintln!("timing prefix={} judged={judged} seconds={:.1}", prefix.name, started.elapsed().as_secs_f64());
        print_tally("acq", prefix.name, &tally);
    }
}

fn mode_rollback(prefix_names: &[String]) {
    let rounds = if std::env::var("SINGLEFS_PROBE_ACQUISITION").as_deref() == Ok("two_rounds") { 2 } else { 1 };
    for prefix in PREFIXES.iter().filter(|prefix| prefix_names.iter().any(|name| name == prefix.name)) {
        let (base, _root) = starting_point(prefix.overwrites, prefix.remount_first);
        for failing in 1..=(2 * rounds as u64) {
            for only_once in [true, false] {
                let (operations, outcome) = record_mount(&base, false, Some((failing, only_once)));
                let (writes, segments, closed, torn) = counts(&base, &operations);
                let name = format!("{}_fail{failing}_{}", prefix.name, if only_once { "once" } else { "onward" });
                println!(
                    "probe=rollback_counts case={name} {} outcome={} writes={} segments={} closed_form_states={closed} states_with_torn={torn} segment_shape={}",
                    arm_name(), outcome.as_ref().map_or_else(|e| e.clone(), |i| format!("mounted {i}")), writes.len(), segments.len(), segment_shape(&writes, &segments)
                );
                let mut end = base.clone();
                end.apply(&operations);
                println!("probe=rollback_end case={name} {} sysconf={}", arm_name(), sysconf_signature(&end));
                if torn > STATE_LIMIT_PER_RUN {
                    continue;
                }
                let (_judged, tallies) = for_each_state(&base, &writes, &segments, &|ordinal, pool, _w, _p, tally: &mut Tally| {
                    tally.states += 1;
                    if checker_on() {
                        for red in reds(pool) {
                            bump(&mut tally.raw_reds, &red);
                        }
                    }
                    let parameters = parameters_of_the_384_slot_small_devices();
                    let mut devices = plain_devices_on(pool);
                    let key = match mount_outcome(&parameters, &mut devices) {
                        Ok(_) => "mounted".to_string(),
                        Err(error) => error,
                    };
                    if key != "mounted" {
                        tally.examples.entry(format!("next_mount/{key}")).or_insert_with(|| format!("ordinal={ordinal} sysconf={}", sysconf_signature(pool)));
                    }
                    bump(tally.outcomes.entry("next_mount_no_fault".into()).or_default(), &key);
                });
                print_tally("rollback", &name, &merge(tallies));
            }
        }
    }
}

pub fn run(args: &[String]) {
    let mode = args.first().map(String::as_str).unwrap_or("");
    match mode {
        "acq" => mode_acq(&args[1..]),
        "rollback" => mode_rollback(&args[1..]),
        "alpha" => crate::probe_alpha::run(&args[1..]),
        _ => panic!("用法：opus-r2-probe acq|rollback|alpha <前缀名…>"),
    }
}
