//! m3-reclaim-window-r1 攻方原型（Opus 腿）。在冻结副本的拷贝上编（`crates/attack-rw-probe`），只当二进制跑，不是测试目标。
//! 起点：mkfs（两块 128 MiB 内存稀疏盘、区域归属 0 / 1 / 0、S 取 env `S`）→ 实例 1 第一个文件 → 覆盖写到 txg `C`。
//! 历史：在起点上连崩 `K` 次「写行那次的单元与记录已持久、根槽没持久」（K1 那一类洞，每崩一次跳一个 txg），
//! 再做一次完整的可写挂载（录写流），之后覆盖写 `J` 次（用户动作，J 放开扫）。
//! 判：每次发布落满之后（快照）跑池级 checker、列根环里窗口外的有效根、只留一条根整读、按 D16 定义算回退候选集逐条整读、
//! 丢一块盘之后剩下那块盘恢复 + 整读；可选对整段写流跑层 0 枚举（每个崩溃状态在三种丢盘下恢复 + 整读）。
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    PoolReader, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const FSID: [u8; 16] = *b"RECLAIM-WINDOW-1";
/// 每块盘的字节数：env `DEV_MIB`（默认 128）。
fn device_bytes() -> u64 {
    env_u64("DEV_MIB", 128) << 20
}
const JOURNAL_BYTES: u64 = 1 << 20;
const T0: u64 = 1_790_000_000;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_list(name: &str, default: &str) -> Vec<u64> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("数"))
        .collect()
}

fn parameters(s: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FSID,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(s)
                .expect("S 在区间里"),
        },
    }
}
fn geometry(s: u64) -> FixedGeometry {
    let p = parameters(s);
    FixedGeometry {
        fixed_structure_slot_spacing: p.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: p.geometry.journal_ring_bytes,
        root_ring_slots_per_region: p.geometry.root_ring_slots_per_region,
    }
}
/// 第 salt 版内容：长度 3000（一个数据单元装得下），各版不同。
fn content(salt: u64) -> Vec<u8> {
    let salt = usize::try_from(salt % 251).expect("小");
    (0..3000usize).map(|i| u8::try_from((i + salt * 7 + 3) % 251).expect("小")).collect()
}
type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
type Recorded = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(id, d)| (*id, d.image.clone())).collect(),
        device_size_in_bytes: device_bytes(),
    }
}
fn pool_of_recorded(devices: &Recorded) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(id, d)| (*id, d.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: device_bytes(),
    }
}
fn devices_of(pool: &MemoryPool) -> Devices {
    pool.devices
        .iter()
        .map(|(id, image)| {
            let mut d = SparseBlockDevice::new(device_bytes(), PhysicalBlockSizeInBytes(512));
            d.image = image.clone();
            (*id, d)
        })
        .collect()
}
fn recording_of(pool: &MemoryPool, stream: &SharedStream) -> Recorded {
    devices_of(pool)
        .into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone())))
        .collect()
}
fn without_device(image: &MemoryPool, lost: Option<u32>) -> MemoryPool {
    MemoryPool {
        devices: image.devices.iter().filter(|(id, _)| Some(id.0) != lost).map(|(id, d)| (*id, d.clone())).collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 只读挂载、整读每个活文件：交回（实例, txg, 内容哈希）或错。
fn read_everything(reader: &dyn PoolReader) -> Result<(u32, u64, u64), String> {
    let mounted = mount_read_only(reader).map_err(|e| format!("ro:{}", format!("{e:?}").chars().take(60).collect::<String>()))?;
    let mut bytes = Vec::new();
    for record in mounted.mounted.inode_records().to_vec() {
        if record.nlink == singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED {
            continue;
        }
        let file = mounted.mounted.open_file(reader, InodeNumber(record.inode))
            .map_err(|e| format!("open:{}", format!("{e:?}").chars().take(60).collect::<String>()))?;
        let out = file.read_at(reader, singlefs_core::address::FileOffsetInBytes(0), record.size)
            .map_err(|e| format!("read:{}", format!("{e:?}").chars().take(60).collect::<String>()))?;
        bytes.extend_from_slice(&record.inode.to_le_bytes());
        bytes.extend_from_slice(&out.bytes);
    }
    Ok((mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0, singlefs_harness::fnv1a_64(&bytes)))
}

fn violations(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, v)| match v {
            InvariantVerdict::Violated(_) => Some(name.to_string()),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 环里自证过的根（不按实例表滤：这一族历史里没有被抛弃的根，K1 那几次崩连根都没落）。
fn ring_roots(image: &MemoryPool) -> Vec<RootRecord> {
    let Ok(sc) = choose_system_configuration(image) else { return Vec::new() };
    let im = sc.immutable;
    every_root_ring_slot(&im.sizes)
        .into_iter()
        .filter_map(|slot| match read_root_ring_slot(image, &im.region_devices, &im.sizes, &im.filesystem_identifier, slot) {
            RootRingSlotReading::SelfVerified(root) => Some(root),
            RootRingSlotReading::Bad(_) => None,
        })
        .collect()
}

/// 只留 (实例, txg) 那一条根的镜像（别的根槽清零）。
fn with_only_root(image: &MemoryPool, keep: (u32, u64)) -> MemoryPool {
    let mut copy = image.clone();
    let Ok(sc) = choose_system_configuration(&copy) else { return copy };
    let im = sc.immutable;
    let width = usize::try_from(im.sizes.physical_block_size).expect("宽");
    for slot in every_root_ring_slot(&im.sizes) {
        let device = im.region_devices[usize::try_from(slot.region).expect("区")];
        if !copy.devices.contains_key(&device) {
            continue;
        }
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(&copy, &im.region_devices, &im.sizes, &im.filesystem_identifier, slot) {
            if (root.instance.0, root.checkpoint_txg.0) != keep {
                let offset = slot_offset(slot, im.sizes.fixed_structure_slot_spacing);
                copy.devices.get_mut(&device).expect("盘").write(offset, &vec![0u8; width]);
            }
        }
    }
    copy
}

/// 每个已发布版本（实例, txg）落满那一刻整读出来的哈希。
type Expected = BTreeMap<(u32, u64), u64>;

fn note_version(expected: &mut Expected, image: &MemoryPool, label: &str) {
    match read_everything(image) {
        Ok((inst, txg, hash)) => {
            expected.insert((inst, txg), hash);
        }
        Err(e) => panic!("{label}：落满的版本整读不出：{e}"),
    }
}

/// 起点：mkfs → 实例 1 → 第一个文件 → 覆盖写到 txg c。
fn build_start(s: u64, c: u64, expected: &mut Expected, start_versions: &mut Vec<PublishedVersion>) -> MemoryPool {
    let p = parameters(s);
    let mut devices: Devices = (0..2u32)
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(device_bytes(), PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&p, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } = mount_writable(&p, &mut devices).expect("第一次可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut w = PoolWriter::new(&p, devices.as_mut_slice());
        publish_first_file(&mut w, &mut allocator, current.root(), FirstFile { content: &content(0), write_time_seconds: T0 }, instance, current.record_bytes())
            .expect("第一个文件")
    };
    note_version(expected, &pool_of(&devices), "第一个文件");
    start_versions.push(PublishedVersion { instance, checkpoint_txg: current.root.checkpoint_txg, content: content(0) });
    while current.root.checkpoint_txg.0 < c {
        let salt = current.root.checkpoint_txg.0 + 1;
        let mut w = PoolWriter::new(&p, devices.as_mut_slice());
        current = publish_overwrite(&mut w, &mut allocator, &current, FirstFile { content: &content(salt), write_time_seconds: T0 + salt }, instance)
            .expect("覆盖写");
        drop(w);
        note_version(expected, &pool_of(&devices), "起点覆盖写");
        start_versions.push(PublishedVersion { instance, checkpoint_txg: current.root.checkpoint_txg, content: content(salt) });
    }
    pool_of(&devices)
}

/// 一次完整的可写挂载、挂载之后不写（写行 + 暖机都落满）：新实例的几版内容与挂载之前那一版相同。
fn full_mount_without_writes(image: &MemoryPool, s: u64, body: &[u8], expected: &mut Expected, start_versions: &mut Vec<PublishedVersion>) -> MemoryPool {
    let p = parameters(s);
    let mut devices = devices_of(image);
    let mounted = mount_writable(&p, &mut devices).expect("不写的完整挂载");
    let instance = mounted.output.instance;
    for v in std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter()) {
        start_versions.push(PublishedVersion { instance, checkpoint_txg: v.root().checkpoint_txg, content: body.to_vec() });
    }
    drop(mounted);
    let after = pool_of(&devices);
    let (_, _, hash) = read_everything(&after).expect("不写的完整挂载整读");
    for v in start_versions.iter().filter(|v| v.instance == instance) {
        expected.insert((instance.0, v.checkpoint_txg.0), hash);
    }
    after
}

/// 一次可写挂载崩在写行那次根槽写之前（写行那次的单元与记录已持久，根槽与之后的写都没落）。交回崩后盘面与写行 txg。
fn crash_a_mount_before_its_row_root(image: &MemoryPool, s: u64) -> (MemoryPool, u64) {
    let p = parameters(s);
    let stream = SharedStream::retaining_contents();
    let mut rec = recording_of(image, &stream);
    let mounted = mount_writable(&p, &mut rec).expect("被截的那次挂载");
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let (writes, _segments) = writes_and_segments(&stream.retained_operations(), &geometry(s));
    let cut = writes
        .iter()
        .position(|w| {
            w.kind == StepKind::RootRecordFua
                && w.bytes().and_then(|b| RootRecord::parse_slot(b, &FSID)).is_some_and(|r| r.checkpoint_txg.0 == row_txg)
        })
        .expect("写行那次的根槽写");
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..cut]);
    (crashed, row_txg)
}

/// 读出来的 (实例, txg, 哈希) 与那一版落满时整读的比：ok / bad（哈希不同）/ unknown（没见过的版本）/ 错。
fn classify(result: &Result<(u32, u64, u64), String>, expected: &Expected) -> String {
    match result {
        Ok((inst, txg, hash)) => match expected.get(&(*inst, *txg)) {
            Some(h) if h == hash => format!("ok@{inst}:{txg}"),
            Some(_) => format!("BAD@{inst}:{txg}"),
            None => format!("UNKNOWN@{inst}:{txg}"),
        },
        Err(e) => format!("ERR[{e}]"),
    }
}

/// 一份落满的快照上判：checker、窗口外有效根（只留它整读）、D16 回退候选集（只留它整读）、丢一块盘之后剩下那块盘整读。
fn judge_snapshot(label: &str, image: &MemoryPool, s: u64, expected: &Expected) -> (bool, String) {
    let roots = ring_roots(image);
    let newest = roots.iter().map(|r| r.checkpoint_txg.0).max().unwrap_or(0);
    let lower = (newest + 1).saturating_sub(3 * s); // 窗口 = txg ≥ lower（txg > N − 3S）
    let floor = choose_system_configuration(image).map(|sc| sc.quantities.rollback_floor.0).unwrap_or(0);
    let mut bad = false;
    let mut outside = Vec::new();
    let mut candidates = Vec::new();
    let mut ids: Vec<(u32, u64)> = roots.iter().map(|r| (r.instance.0, r.checkpoint_txg.0)).collect();
    ids.sort_unstable();
    for (inst, txg) in &ids {
        let inside = *txg >= lower;
        if inside && *txg < floor {
            continue;
        }
        let read = classify(&read_everything(&with_only_root(image, (*inst, *txg))), expected);
        if inside {
            if !read.starts_with("ok@") {
                bad = true;
            }
            candidates.push(format!("{inst}:{txg}={read}"));
        } else {
            outside.push(format!("{inst}:{txg}={read}"));
        }
    }
    let mut lone = Vec::new();
    for lost in [0u32, 1] {
        let survivor = without_device(image, Some(lost));
        let read = classify(&read_everything(&survivor), expected);
        if !read.starts_with("ok@") {
            bad = true;
        }
        lone.push(format!("lost{lost}:{read}"));
    }
    let v = violations(image);
    if !v.is_empty() {
        bad = true;
    }
    let line = format!(
        "{label} newest={newest} window_lower={lower} F={floor} checker=[{}] outside_window=[{}] candidates=[{}] lone=[{}]",
        v.join("+"), outside.join(" "), candidates.join(" "), lone.join(" ")
    );
    (bad, line)
}

/// 最后那次完整挂载（录写流）+ 覆盖写 j 次。交回（挂载之前的盘面, 写表, 段, 已发布版本, 每次根槽写落下那一刻的快照）。
struct Run {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
    root_indexes: Vec<usize>,
    mount_line: String,
}

fn mount_and_overwrite(base: &MemoryPool, s: u64, j: u64, expected: &mut Expected) -> Result<Run, String> {
    let p = parameters(s);
    let stream = SharedStream::retaining_contents();
    let mut rec = recording_of(base, &stream);
    let mut mounted = mount_writable(&p, &mut rec).map_err(|e| format!("mount:{e:?}"))?;
    let instance = mounted.output.instance;
    let mount_line = format!(
        "mount instance={} chosen={} effective={} row={} warm_up={:?} current={} alloc_after_mount=[{}]",
        instance.0,
        mounted.output.chosen_root.checkpoint_txg.0,
        mounted.output.effective_root.checkpoint_txg.0,
        mounted.output.row_publish.root().checkpoint_txg.0,
        mounted.output.warm_up_publishes.iter().map(|v| v.root().checkpoint_txg.0).collect::<Vec<_>>(),
        mounted.current.root().checkpoint_txg.0,
        mounted.allocator.devices.iter().map(|d| format!("dev{}:alloc={},free={},deferred={},isolated={}", d.device.0, d.allocated_slots(), d.free_slots(), d.deferred_slots(), d.isolated_slots())).collect::<Vec<_>>().join(" ")
    );
    let PoolVersion::WithFile(mut current) = mounted.current.clone() else {
        return Err("现行那一版不带文件".into());
    };
    let mut contents: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let start_content = read_file(&pool_of_recorded(&rec));
    for v in std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter()) {
        contents.insert(v.root().checkpoint_txg.0, start_content.clone());
    }
    for _ in 0..j {
        let salt = current.root.checkpoint_txg.0 + 1;
        let body = content(salt + 101);
        let mut w = PoolWriter::new(&p, rec.as_mut_slice());
        current = publish_overwrite(&mut w, &mut mounted.allocator, &current, FirstFile { content: &body, write_time_seconds: T0 + salt }, instance)
            .map_err(|e| format!("overwrite:{e:?}"))?;
        contents.insert(salt, body);
    }
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry(s));
    let root_indexes: Vec<usize> = writes.iter().enumerate().filter(|(_, w)| w.kind == StepKind::RootRecordFua).map(|(i, _)| i).collect();
    // 每次根槽写落下那一刻（它之前的写全落、之后的全没落）各一份快照，整读进 expected。
    for &i in &root_indexes {
        let mut img = base.clone();
        img.apply_writes(&writes[..=i]);
        note_version(expected, &img, "新实例的发布");
    }
    let versions = contents
        .into_iter()
        .map(|(txg, c)| PublishedVersion { instance, checkpoint_txg: CheckpointTxg(txg), content: c })
        .collect();
    Ok(Run { base: base.clone(), writes, segments, versions, root_indexes, mount_line })
}

/// 第一个文件的内容（只读挂载整读，这一族只有一个文件）。
fn read_file(image: &MemoryPool) -> Vec<u8> {
    let mounted = mount_read_only(image).expect("只读挂载");
    let record = mounted.mounted.inode_records().to_vec().into_iter().next().expect("有文件");
    let file = mounted.mounted.open_file(image, InodeNumber(record.inode)).expect("打开");
    file.read_at(image, singlefs_core::address::FileOffsetInBytes(0), record.size).expect("读").bytes
}

fn run_layer0(run: &Run, s: u64, expected: &Expected, label: &str) {
    let states = closed_form_state_count(&run.segments);
    let cap = env_u64("LAYER0_MAX", 1_000_000);
    if states > cap {
        println!("{label} LAYER0_SKIPPED states={states} cap={cap}");
        return;
    }
    let started = Instant::now();
    let judged = *run.root_indexes.last().expect("有根槽写");
    let mut tally_by_lost: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    let mut samples: Vec<String> = Vec::new();
    let mut observe = |image: &singlefs_harness::memory_pool::CrashImage<'_>, _report: &singlefs_core::recovery::RecoveryReport| {
        let persisted: Vec<RetainedWrite> = image.writes.iter().zip(&image.persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect();
        let mut full = image.base.clone();
        full.apply_writes(&persisted);
        for lost in [None, Some(0u32), Some(1u32)] {
            let survivor = without_device(&full, lost);
            let read = classify(&read_everything(&survivor), expected);
            let kind = read.split('@').next().unwrap_or("").split('[').next().unwrap_or("").to_string();
            let name = lost.map_or("none".to_string(), |d| format!("lost{d}"));
            *tally_by_lost.entry(name.clone()).or_default().entry(kind.clone()).or_default() += 1;
            if kind != "ok" && samples.len() < 6 {
                let n = image.persisted.iter().filter(|p| **p).count();
                samples.push(format!("{name} persisted={n}/{} read={read}", image.persisted.len()));
            }
        }
    };
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &run.base, &run.writes, &run.segments, judged, &run.versions, &|_, _| true, &mut observe,
    );
    let _ = s;
    println!(
        "{label} LAYER0 states={states} tally_states={} failed={} violations={} checker_violated={:?} by_lost={:?} elapsed={:.1}s",
        tally.states, tally.failed_states, tally.violations, tally.checker_violated_states, tally_by_lost, started.elapsed().as_secs_f64()
    );
    for sample in samples {
        println!("{label}   sample {sample}");
    }
}

fn main() {
    let s = env_u64("S", 4);
    let c = env_u64("C", 3 * s * 2);
    let ks = env_list("K", "0,1,2");
    let js = env_list("J", "0,1,2,3");
    let layer0 = env_u64("LAYER0", 0) == 1;
    let started = Instant::now();
    let mut expected = Expected::new();
    let mut start_versions = Vec::new();
    let start = build_start(s, c, &mut expected, &mut start_versions);
    println!("START s={s} c={c} ring={} build={:.1}s", 3 * s, started.elapsed().as_secs_f64());
    let max_k = *ks.iter().max().expect("K");
    let mut image = start;
    for _ in 0..env_u64("PRE", 0) {
        image = full_mount_without_writes(&image, s, &content(c), &mut expected, &mut start_versions);
    }
    let mut skipped = Vec::new();
    for k in 0..=max_k {
        if ks.contains(&k) {
            for &j in &js {
                let label = format!("s={s} c={c} k={k} j={j}");
                let mut exp = expected.clone();
                match mount_and_overwrite(&image, s, j, &mut exp) {
                    Ok(mut run) => {
                        run.versions.extend(start_versions.iter().cloned());
                        println!("{label} {} skipped_row_txgs={skipped:?}", run.mount_line);
                        let mut any_bad = false;
                        for (n, &i) in run.root_indexes.iter().enumerate() {
                            let mut img = run.base.clone();
                            img.apply_writes(&run.writes[..=i]);
                            let (bad, line) = judge_snapshot(&format!("{label} root#{n}"), &img, s, &exp);
                            any_bad |= bad;
                            if bad || env_u64("VERBOSE", 0) == 1 {
                                println!("{line}");
                            }
                        }
                        println!("{label} SNAPSHOTS roots={} any_bad={any_bad}", run.root_indexes.len());
                        if layer0 {
                            run_layer0(&run, s, &exp, &label);
                        }
                    }
                    Err(e) => println!("{label} RUN_ERR {e}"),
                }
            }
        }
        if k < max_k {
            let (next, row) = crash_a_mount_before_its_row_root(&image, s);
            skipped.push(row);
            image = next;
        }
    }
    println!("DONE elapsed={:.1}s", started.elapsed().as_secs_f64());
}
