//! 云端攻方 m2-layer0-scale-r1 的探针（只在草稿副本里，不入库装置）：内存稀疏盘上重建固定脚本与自造历史，
//! 按「约简甲」（段内非原地覆盖的写一律取 ∅）对照全量子集，逐状态比恢复报告、oracle、checker 与记录核对器。
#![allow(dead_code, clippy::all)]

mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{mount_rollback, mount_writable, raise_rollback_floor, unmount, RollbackTarget, ShadowLedger};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion, PoolWriter, TransactionOutput};
use singlefs_harness::crash::{
    check_records, closed_form_state_count, newest_persisted_root, oracle_violation_for_versions,
    writes_and_segments_with_stream_indexes_and_entries, CrashImage, MemoryPool, PublishedVersion, RecordCheck,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedEntrySpan, RecordedPublishEntry, RecordingBlockDevice, RetainedOperation, SharedStream};

// ───────────── 可开关读故障的内存盘 ─────────────
#[derive(Clone, Default)]
pub struct ReadFaults(Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>>);
impl ReadFaults {
    fn set(&self, ranges: Vec<(DeviceIdentity, u64, u64)>) { *self.0.borrow_mut() = ranges; }
    fn clear(&self) { self.0.borrow_mut().clear(); }
    fn hits(&self, device: DeviceIdentity, start: u64, end: u64) -> bool {
        self.0.borrow().iter().any(|(d, s, e)| *d == device && *s < end && start < *e)
    }
}
pub struct Dev { id: DeviceIdentity, inner: SparseBlockDevice, faults: ReadFaults }
impl BlockDevice for Dev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.faults.hits(self.id, offset.0, offset.0 + buffer.len() as u64) {
            return Err(injected_block_device_error("探针读"));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> { self.inner.barrier() }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes { self.inner.probe_physical_block_size() }
    fn size_in_bytes(&self) -> u64 { self.inner.size_in_bytes() }
}
type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<Dev>)>;

// ───────────── 内存里的池与脚本 ─────────────
pub struct Sim {
    pub stream: SharedStream,
    pub devices: Devices,
    pub faults: ReadFaults,
    pub allocator: PoolAllocator,
    pub output: TransactionOutput,
    pub versions: Vec<PublishedVersion>,
    pub mkfs_ops: usize,
    pub base: MemoryPool,
}

pub fn second_content() -> Vec<u8> { (0..4100usize).map(|i| u8::try_from((i * 7 + 3) % 253).unwrap()).collect() }
pub fn third_content() -> Vec<u8> { (0..2500usize).map(|i| u8::try_from((i * 7 + 11) % 253).unwrap()).collect() }
pub fn later_content(seed: usize) -> Vec<u8> { (0..3000 + seed).map(|i| u8::try_from((i * 5 + seed) % 251).unwrap()).collect() }

fn version(instance: u32, txg: u64, content: &[u8]) -> PublishedVersion {
    PublishedVersion { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg), content: content.to_vec() }
}

impl Sim {
    /// mkfs → 取号 1 → 暖机 → A（txg 3），全在内存稀疏盘上，录制流开内容保留。
    pub fn first_transaction() -> Sim {
        let stream = SharedStream::retaining_contents();
        let faults = ReadFaults::default();
        let mut devices: Devices = (0..2u32)
            .map(|n| {
                let id = DeviceIdentity(n);
                let dev = Dev { id, inner: SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)), faults: faults.clone() };
                (id, RecordingBlockDevice::with_shared_stream(id, dev, stream.clone()))
            })
            .collect();
        let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
        let mkfs_ops = stream.operations().len();
        let base = memory_pool_of(&stream.retained_operations());
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
        let content = file_content();
        let output = {
            let params = parameters();
            let mut pool = PoolWriter::new(&params, &mut devices);
            let instance = acquire_instance(&mut pool).expect("取号");
            let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
            publish_first_file(&mut pool, &mut allocator, warm.roots.last().unwrap(),
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS }, instance, &warm.last_record_bytes)
                .expect("第一个事务")
        };
        Sim { stream, devices, faults, allocator, output, versions: vec![version(1, 3, &file_content())], mkfs_ops, base }
    }

    pub fn overwrite(&mut self, content: &[u8], instance: u32) -> TransactionOutput {
        let params = parameters();
        let mut writer = PoolWriter::new(&params, self.devices.as_mut_slice());
        let out = publish_overwrite(&mut writer, &mut self.allocator, &self.output,
            FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, InstanceGeneration(instance))
            .expect("覆盖写");
        self.versions.push(version(instance, out.root.checkpoint_txg.0, content));
        self.output = out.clone();
        out
    }

    fn push_mount_versions(&mut self, mounted: &singlefs_core::mount::Mounted, content: &[u8]) {
        let mut all = vec![&mounted.output.row_publish];
        all.extend(mounted.output.warm_up_publishes.iter());
        for v in all {
            let root = v.root();
            self.versions.push(version(root.instance.0, root.checkpoint_txg.0, content));
        }
    }

    /// 进程重开、可写挂载（内存盘上就是丢掉分配器，从盘上重新挂）。`content` 是挂上之后现行那一版的文件内容。
    pub fn remount(&mut self, content: &[u8]) -> singlefs_core::mount::MountOutput {
        let mounted = mount_writable(&parameters(), &mut self.devices).expect("可写挂载");
        self.push_mount_versions(&mounted, content);
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.clone().into_file_version().expect("带文件");
        mounted.output
    }

    pub fn rollback(&mut self, instance: u32, txg: u64, content: &[u8], shadow: ShadowLedger) {
        let mounted = mount_rollback(&parameters(), &mut self.devices,
            RollbackTarget { instance: InstanceGeneration(instance), checkpoint_txg: CheckpointTxg(txg) }, shadow)
            .expect("回退");
        self.push_mount_versions(&mounted, content);
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.clone().into_file_version().expect("带文件");
    }

    pub fn raise_floor(&mut self, floor: u64, content: &[u8]) {
        let mut current = self.output.clone();
        let raised = raise_rollback_floor(&parameters(), &mut self.devices, &mut self.allocator, &mut current,
            CheckpointTxg(floor), ShadowLedger::On).expect("抬 F");
        for p in &raised.publishes {
            self.versions.push(version(p.root.instance.0, p.root.checkpoint_txg.0, content));
        }
        self.output = current;
    }

    pub fn unmount_now(&mut self, content: &[u8]) {
        let mut current = PoolVersion::WithFile(self.output.clone());
        let stream = self.stream.clone();
        let devices = &mut self.devices;
        let allocator = &mut self.allocator;
        let unmounted = stream
            .record_entry(RecordedPublishEntry::Unmount, || unmount(&parameters(), devices, allocator, &mut current, ShadowLedger::On))
            .expect("卸载");
        for p in &unmounted.publishes {
            self.versions.push(version(p.root.instance.0, p.root.checkpoint_txg.0, content));
        }
        self.output = current.into_file_version().expect("带文件");
    }

    pub fn stream_for_layer0(&self) -> Layer0Stream {
        let ops = self.stream.retained_operations();
        let spans: Vec<RecordedEntrySpan> = self
            .stream
            .entry_spans()
            .into_iter()
            .map(|span| RecordedEntrySpan { entry: span.entry, operations: (span.operations.start - self.mkfs_ops)..(span.operations.end - self.mkfs_ops) })
            .collect();
        let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(&ops[self.mkfs_ops..], &spans, &geometry());
        let judged = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).unwrap();
        Layer0Stream { base: self.base.clone(), writes, segments, judged_root_index: judged, versions: self.versions.clone() }
    }
}

fn memory_pool_of(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    pool.apply(operations);
    pool
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Script { B, C, D, E, E18 }

/// 固定脚本（照 second_transaction_step_zero_layer0.rs 的 prepare，逐步同名同参），内存盘上重建。
pub fn fixed_script(script: Script) -> Sim {
    let mut sim = Sim::first_transaction();
    sim.overwrite(&second_content(), 1);
    if script == Script::B { return sim; }
    sim.remount(&second_content());
    sim.overwrite(&third_content(), 2);
    if script == Script::C { return sim; }
    sim.rollback(1, 3, &file_content(), ShadowLedger::On);
    if script == Script::D { return sim; }
    let mut latest = Vec::new();
    for seed in [17usize, 19, 23, 29] { latest = later_content(seed); sim.overwrite(&latest, 3); }
    sim.raise_floor(11, &latest);
    sim.overwrite(&later_content(31), 3);
    if script == Script::E { return sim; }
    sim.overwrite(&later_content(37), 3);
    sim
}

// ───────────── 层 0 流与逐状态观测 ─────────────
pub struct Layer0Stream {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Obs {
    pub consulted: RecoveryReport,
    pub ignored: RecoveryReport,
    pub oracle_consulted: Option<String>,
    pub oracle_ignored: Option<String>,
    pub checker: Vec<(&'static str, InvariantVerdict)>,
    pub records: RecordCheck,
}
impl Obs {
    pub fn bad(&self) -> bool {
        self.oracle_consulted.is_some() || self.oracle_ignored.is_some()
            || self.checker.iter().any(|(_, v)| matches!(v, InvariantVerdict::Violated(_)))
            || self.records.root_without_record || self.records.claimed_state_missing_unit
    }
    pub fn why(&self) -> String {
        let mut parts = Vec::new();
        if let Some(r) = &self.oracle_consulted { parts.push(format!("oracle={r}")); }
        if let Some(r) = &self.oracle_ignored { parts.push(format!("oracle_ignored={r}")); }
        for (i, v) in &self.checker { if let InvariantVerdict::Violated(d) = v { parts.push(format!("{i}:{}", d.chars().take(160).collect::<String>())); } }
        if self.records.root_without_record { parts.push("record:root_without_record".into()); }
        if self.records.claimed_state_missing_unit { parts.push("record:claimed_state_missing_unit".into()); }
        parts.join(" | ")
    }
}

impl Layer0Stream {
    pub fn observe(&self, persisted: Vec<bool>) -> Obs {
        let newest = newest_persisted_root(&self.writes, &persisted);
        let image = CrashImage { base: &self.base, writes: &self.writes, persisted };
        let consulted = recover(&image, JournalPolicy::Consult);
        let ignored = recover(&image, JournalPolicy::Ignore);
        let oracle_consulted = oracle_violation_for_versions(&consulted.outcome, consulted.effective_root, newest, &self.versions);
        let oracle_ignored = oracle_violation_for_versions(&ignored.outcome, ignored.effective_root, newest, &self.versions);
        let checker = check_pool_image(&image);
        let records = check_records(&image, consulted.effective_root);
        Obs { consulted, ignored, oracle_consulted, oracle_ignored, checker, records }
    }
    pub fn prefix(&self, segment_index: usize) -> Vec<bool> {
        let mut p = vec![false; self.writes.len()];
        for seg in &self.segments[..segment_index] { for w in seg { p[*w] = true; } }
        p
    }
    pub fn is_cow(&self, write_index: usize) -> bool { self.writes[write_index].kind == StepKind::UnitWrite }
    pub fn sizes(&self) -> Vec<usize> { self.segments.iter().map(Vec::len).collect() }
    /// 约简甲的闭式：段内原地覆盖 k 个、COW n 个——n > 0 时 2^k 个状态（COW 取 ∅），n = 0 时 2^k − 1（同全量）。
    pub fn reduced_count(&self) -> u64 {
        1 + self.segments.iter().map(|seg| {
            let k = seg.iter().filter(|w| !self.is_cow(**w)).count() as u32;
            let n = seg.len() as u32 - k;
            if n > 0 { 1u64 << k } else { (1u64 << k) - 1 }
        }).sum::<u64>()
    }
    pub fn kinds_text(&self) -> String {
        self.segments.iter().map(|seg| {
            let c = seg.iter().filter(|w| self.is_cow(**w)).count();
            let mut kinds: BTreeMap<&'static str, usize> = BTreeMap::new();
            for w in seg { *kinds.entry(self.writes[*w].kind.name()).or_insert(0) += 1; }
            let _ = c;
            format!("[{}]", kinds.iter().map(|(k, n)| format!("{k}×{n}")).collect::<Vec<_>>().join(","))
        }).collect::<Vec<_>>().join(" ")
    }
    /// 写表下标 → 它属于哪次发布（后面第一条根槽写的 txg）。
    pub fn publish_txg_of(&self, index: usize) -> Option<u64> {
        self.writes[index..].iter().find(|w| w.kind == StepKind::RootRecordFua)
            .map(|w| u64::from_le_bytes(w.bytes().unwrap()[28..36].try_into().unwrap()))
    }
}

// ───────────── 约简甲对照：段内每个原地覆盖子集 I，比 (I, S) 与 (I, ∅) ─────────────
pub struct SegmentComparison {
    pub segment_index: usize,
    pub in_place: usize,
    pub cow: usize,
    pub compared: u64,
    pub differing: u64,
    pub differing_verdict: u64,
    pub bad_only_when_reduced_away: u64,
    pub first_differences: Vec<String>,
}

fn xorshift(state: &mut u64) -> u64 { *state ^= *state << 13; *state ^= *state >> 7; *state ^= *state << 17; *state }

/// `exhaustive`：COW 子集全枚举（非空的全部）；否则取全集、单点、去单点与 `random` 个随机子集。
pub fn compare_segment(stream: &Layer0Stream, segment_index: usize, exhaustive: bool, random: usize, threads: usize) -> SegmentComparison {
    let seg = &stream.segments[segment_index];
    let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
    let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
    let n = cow.len();
    let full: u64 = if n == 0 { 0 } else { (1u64 << n) - 1 };
    let mut masks: Vec<u64> = Vec::new();
    if n > 0 {
        if exhaustive { masks.extend(1..=full); } else {
            masks.push(full);
            for b in 0..n { masks.push(1 << b); masks.push(full ^ (1 << b)); }
            let mut st = 0x9e37_79b9_7f4a_7c15u64 ^ (segment_index as u64);
            for _ in 0..random { let m = xorshift(&mut st) & full; if m != 0 { masks.push(m); } }
            masks.sort_unstable(); masks.dedup();
        }
    }
    let prefix = stream.prefix(segment_index);
    let mut result = SegmentComparison { segment_index, in_place: in_place.len(), cow: n, compared: 0, differing: 0, differing_verdict: 0, bad_only_when_reduced_away: 0, first_differences: Vec::new() };
    if n == 0 { return result; }
    let result = Mutex::new(result);
    for imask in 0..(1u64 << in_place.len()) {
        let mut base_p = prefix.clone();
        for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { base_p[*w] = true; } }
        let rep = stream.observe(base_p.clone());
        let next = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| loop {
                    let k = next.fetch_add(1, Ordering::Relaxed);
                    let Some(m) = masks.get(k) else { break };
                    let mut p = base_p.clone();
                    for (b, w) in cow.iter().enumerate() { if m & (1 << b) != 0 { p[*w] = true; } }
                    let o = stream.observe(p);
                    let mut r = result.lock().unwrap();
                    r.compared += 1;
                    if o != rep {
                        r.differing += 1;
                        if o.bad() != rep.bad() {
                            r.differing_verdict += 1;
                            if o.bad() && !rep.bad() { r.bad_only_when_reduced_away += 1; }
                        }
                        if r.first_differences.len() < 4 {
                            let what = describe_difference(&rep, &o);
                            r.first_differences.push(format!("in_place_mask={imask:#b} cow_mask={m:#b}: {what}"));
                        }
                    }
                });
            }
        });
    }
    result.into_inner().unwrap()
}

pub fn describe_difference(rep: &Obs, o: &Obs) -> String {
    let mut parts = Vec::new();
    if rep.consulted != o.consulted { parts.push(format!("consulted {:?}/{:?} jr={:?} → {:?}/{:?} jr={:?}", short(&rep.consulted.outcome), rep.consulted.effective_root, rep.consulted.journal, short(&o.consulted.outcome), o.consulted.effective_root, o.consulted.journal)); }
    if rep.ignored != o.ignored { parts.push(format!("ignored {:?} → {:?}", short(&rep.ignored.outcome), short(&o.ignored.outcome))); }
    if rep.oracle_consulted != o.oracle_consulted { parts.push(format!("oracle {:?} → {:?}", rep.oracle_consulted, o.oracle_consulted)); }
    for ((i, a), (_, b)) in rep.checker.iter().zip(&o.checker) { if a != b { parts.push(format!("{i} {} → {}", short(a), short(b))); } }
    if rep.records != o.records { parts.push(format!("records {:?} → {:?}", rep.records, o.records)); }
    parts.join(" ; ")
}
fn short<T: std::fmt::Debug>(t: &T) -> String { format!("{t:?}").chars().take(220).collect() }

pub fn threads() -> usize { std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(4) }

// ───────────── 写落点的静态核对：单元写盖不盖之前写过的扇区 ─────────────
pub fn overlaps_report(stream: &Layer0Stream) -> Vec<String> {
    let mut out = Vec::new();
    for (i, w) in stream.writes.iter().enumerate() {
        if w.kind != StepKind::UnitWrite { continue; }
        let (s, e) = (w.offset.0, w.offset.0 + w.length_in_bytes());
        let base_written = stream.base.devices[&w.device].written_sectors_in(w.offset, w.length_in_bytes());
        let earlier: Vec<String> = stream.writes[..i].iter().enumerate()
            .filter(|(_, x)| x.device == w.device && x.offset.0 < e && s < x.offset.0 + x.length_in_bytes())
            .map(|(j, x)| format!("#{j}({},txg{:?})", x.kind.name(), stream.publish_txg_of(j)))
            .collect();
        if !earlier.is_empty() || !base_written.is_empty() {
            out.push(format!("write#{i} dev{} slot{} txg{:?} overwrites: stream={} mkfs_base_sectors={}",
                w.device.0, s / 16384, stream.publish_txg_of(i), earlier.join(","), base_written.len()));
        }
    }
    out
}

// ───────────── 用例 ─────────────
fn report_stream(name: &str, stream: &Layer0Stream) {
    println!("STREAM {name} writes={} segments={} sizes={:?} closed_form={} reduced_closed_form={}",
        stream.writes.len(), stream.segments.len(), stream.sizes(), closed_form_state_count(&stream.segments), stream.reduced_count());
    println!("KINDS {name} {}", stream.kinds_text());
    for line in overlaps_report(stream) { println!("OVERWRITE {name} {line}"); }
}

#[test]
fn p1_fixed_script_streams_and_overwrites() {
    for (name, script) in [("B", Script::B), ("C", Script::C), ("D", Script::D), ("E", Script::E), ("E18", Script::E18)] {
        let sim = fixed_script(script);
        report_stream(name, &sim.stream_for_layer0());
    }
}

fn run_sampled(name: &str, stream: &Layer0Stream, random: usize) {
    let t = threads();
    let started = std::time::Instant::now();
    let mut total = 0;
    for s in 0..stream.segments.len() {
        let c = compare_segment(stream, s, stream.segments[s].iter().filter(|w| stream.is_cow(**w)).count() <= 10, random, t);
        if c.cow == 0 { continue; }
        total += c.compared;
        println!("PREMISE {name} seg={} in_place={} cow={} compared={} differing={} differing_verdict={} bad_only_when_cow_nonempty={} txg={:?}",
            c.segment_index, c.in_place, c.cow, c.compared, c.differing, c.differing_verdict, c.bad_only_when_reduced_away,
            stream.publish_txg_of(stream.segments[s][0]));
        for d in &c.first_differences { println!("  DIFF {name} seg={} {d}", c.segment_index); }
    }
    println!("PREMISE_TOTAL {name} compared={total} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
}

#[test]
fn p2_premise_sampled_on_fixed_scripts() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    for (name, script) in [("B", Script::B), ("D", Script::D), ("E18", Script::E18)] {
        let sim = fixed_script(script);
        run_sampled(name, &sim.stream_for_layer0(), random);
    }
}

// ───────────── 自造历史 H1：挂载时暂时读不出 dev1 上 B 写下的扇区（根槽、单元、记录那一份）→ 恢复落回 A、B 被抛弃 → 撤故障 → 新实例发布 ─────────────
pub fn abandon_by_transient_fault(overwrites_after: usize) -> (Sim, singlefs_core::mount::MountOutput) {
    let mut sim = Sim::first_transaction();
    sim.overwrite(&second_content(), 1);
    let l0 = sim.stream_for_layer0();
    let ranges: Vec<(DeviceIdentity, u64, u64)> = (0..l0.writes.len())
        .filter(|i| l0.publish_txg_of(*i) == Some(4) && l0.writes[*i].device == DeviceIdentity(1))
        .map(|i| (l0.writes[i].device, l0.writes[i].offset.0, l0.writes[i].offset.0 + l0.writes[i].length_in_bytes()))
        .collect();
    println!("H1 faulted_ranges_on_dev1={}", ranges.len());
    sim.faults.set(ranges);
    let out = sim.remount(&file_content());
    sim.faults.clear();
    println!("H1 mount_under_fault chosen=({},{}) effective=({},{}) instance={} rows={:?} row_publish_txg={} warmups={:?}",
        out.chosen_root.instance.0, out.chosen_root.checkpoint_txg.0, out.effective_root.instance.0, out.effective_root.checkpoint_txg.0,
        out.instance.0, out.rows_written, out.row_publish.root().checkpoint_txg.0,
        out.warm_up_publishes.iter().map(|p| p.root().checkpoint_txg.0).collect::<Vec<_>>());
    for k in 0..overwrites_after { sim.overwrite(&later_content(41 + k), out.instance.0); }
    (sim, out)
}

/// 每段：约简甲留下的状态里有几个坏的、抽样的全量子集里有几个坏的，各给第一处。
fn verdicts_full_sampled_vs_reduced(name: &str, stream: &Layer0Stream, random: usize) {
    let t = threads();
    let mut reduced_bad = 0u64; let mut reduced_states = 0u64;
    let mut sampled_bad = 0u64; let mut sampled_states = 0u64;
    for s in 0..stream.segments.len() {
        let seg = &stream.segments[s];
        let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
        let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
        let prefix = stream.prefix(s);
        for imask in 0..(1u64 << in_place.len()) {
            if cow.is_empty() && imask == (1u64 << in_place.len()) - 1 { continue; }
            let mut p = prefix.clone();
            for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { p[*w] = true; } }
            let o = stream.observe(p);
            reduced_states += 1;
            if o.bad() { reduced_bad += 1; if reduced_bad <= 3 { println!("  REDUCED_BAD {name} seg={s} imask={imask:#b} {}", o.why()); } }
        }
        if !cow.is_empty() {
            let c = compare_segment(stream, s, cow.len() <= 10, random, t);
            sampled_states += c.compared;
            if c.differing > 0 {
                println!("  SEG {name} seg={s} txg={:?} in_place={} cow={} compared={} differing={} differing_verdict={} bad_only_when_cow_nonempty={}",
                    stream.publish_txg_of(seg[0]), c.in_place, c.cow, c.compared, c.differing, c.differing_verdict, c.bad_only_when_reduced_away);
                for d in &c.first_differences { println!("    DIFF {d}"); }
            }
            sampled_bad += c.bad_only_when_reduced_away;
        }
    }
    let all = stream.observe(vec![true; stream.writes.len()]);
    println!("VERDICTS {name} reduced_states={} reduced_bad={reduced_bad} sampled_cow_states={sampled_states} bad_only_with_cow_nonempty={sampled_bad} every_write_persisted_bad={} {}",
        reduced_states + 1, all.bad(), all.why());
}

#[test]
fn p3_abandoned_timeline_by_transient_read_fault() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    let (sim, _out) = abandon_by_transient_fault(1);
    let stream = sim.stream_for_layer0();
    report_stream("H1", &stream);
    verdicts_full_sampled_vs_reduced("H1", &stream, random);
}

// ───────────── 自造历史 H2：B 之后正常卸载（F 抬到 4）→ 重挂 → 覆盖写若干次（复用 A 的落点）→ 再卸载 → 重挂 → 覆盖写 ─────────────
pub fn unmount_history(overwrites: usize) -> Sim {
    let mut sim = Sim::first_transaction();
    sim.overwrite(&second_content(), 1);
    sim.unmount_now(&second_content());
    let out = sim.remount(&second_content());
    let mut last = second_content();
    for k in 0..overwrites { last = later_content(51 + k); sim.overwrite(&last, out.instance.0); }
    sim.unmount_now(&last);
    let out2 = sim.remount(&last);
    sim.overwrite(&later_content(71), out2.instance.0);
    sim
}

#[test]
fn p4_unmount_raise_and_reuse() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    let sim = unmount_history(3);
    let stream = sim.stream_for_layer0();
    report_stream("H2", &stream);
    verdicts_full_sampled_vs_reduced("H2", &stream, random);
}

#[test]
fn p5_fixed_script_verdicts() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    for (name, script) in [("E18", Script::E18)] {
        let sim = fixed_script(script);
        verdicts_full_sampled_vs_reduced(name, &sim.stream_for_layer0(), random);
    }
}

/// 每段三种取法各有几个坏状态：甲（COW 取 ∅）、甲二（COW 取 ∅ 或全集）、抽样的全量子集（COW ≤ 12 个时全枚举）。
fn three_arms(name: &str, stream: &Layer0Stream, random: usize) {
    let t = threads();
    let (mut a, mut a2, mut full) = (0u64, 0u64, 0u64);
    let (mut na, mut na2, mut nfull) = (0u64, 0u64, 0u64);
    for s in 0..stream.segments.len() {
        let seg = &stream.segments[s];
        let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
        let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
        let prefix = stream.prefix(s);
        let (mut sa, mut sa2) = (0u64, 0u64);
        for imask in 0..(1u64 << in_place.len()) {
            let mut p = prefix.clone();
            for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { p[*w] = true; } }
            let is_full_segment = cow.is_empty() && imask == (1u64 << in_place.len()) - 1;
            if !is_full_segment {
                let o = stream.observe(p.clone());
                na += 1; na2 += 1;
                if o.bad() { sa += 1; sa2 += 1; if sa <= 2 { println!("  ARM_A_BAD {name} seg={s} imask={imask:#b} {}", o.why()); } }
            }
            if !cow.is_empty() && imask != (1u64 << in_place.len()) - 1 {
                let mut q = p.clone();
                for w in &cow { q[*w] = true; }
                let o = stream.observe(q);
                na2 += 1;
                if o.bad() { sa2 += 1; if sa2 - sa <= 2 { println!("  ARM_A2_BAD {name} seg={s} imask={imask:#b} cow=all {}", o.why()); } }
            }
        }
        a += sa; a2 += sa2;
        // 全量：原地覆盖子集 × COW 子集（抽样或全枚举），去掉整段全持久那一个。
        let mut sf = 0u64;
        if !cow.is_empty() {
            let c = compare_segment(stream, s, cow.len() <= 12, random, t);
            // compare_segment 只比 COW 非空的；COW 为空的那 2^k 个就是甲的状态。
            nfull += c.compared;
            sf = c.bad_only_when_reduced_away;
            if c.differing > 0 { for d in c.first_differences.iter().take(2) { println!("  FULL_DIFF {name} seg={s} {d}"); } }
        }
        full += sf;
        if sa + sf + sa2 > 0 {
            println!("ARMSEG {name} seg={s} txg={:?} in_place={} cow={} arm_a_bad={sa} arm_a2_bad={sa2} full_sampled_extra_bad={sf}",
                stream.publish_txg_of(seg[0]), in_place.len(), cow.len());
        }
    }
    println!("ARMS {name} arm_a_states={} arm_a_bad={a} arm_a2_states={} arm_a2_bad={a2} full_sampled_cow_states={nfull} full_sampled_bad_beyond_arm_a={full}",
        na + 1, na2 + 1);
}

#[test]
fn p6_three_arms_on_fixed_scripts_and_histories() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(32);
    let which = std::env::var("PROBE_WHICH").unwrap_or_else(|_| "C".to_string());
    let stream = match which.as_str() {
        "B" => fixed_script(Script::B).stream_for_layer0(),
        "C" => fixed_script(Script::C).stream_for_layer0(),
        "D" => fixed_script(Script::D).stream_for_layer0(),
        "E18" => fixed_script(Script::E18).stream_for_layer0(),
        "H1" => abandon_by_transient_fault(1).0.stream_for_layer0(),
        "H2" => unmount_history(3).stream_for_layer0(),
        other => panic!("{other}"),
    };
    report_stream(&which, &stream);
    three_arms(&which, &stream, random);
}

/// 段内 COW 写只在 `chosen`（段内 COW 的下标，按段内次序）上全枚举，其余 COW 写固定成 `others_persisted`。
pub fn compare_segment_restricted(stream: &Layer0Stream, segment_index: usize, chosen: &[usize], others_persisted: bool, threads: usize) -> SegmentComparison {
    let seg = &stream.segments[segment_index];
    let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
    let cow_all: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
    let cow: Vec<usize> = chosen.iter().map(|k| cow_all[*k]).collect();
    let others: Vec<usize> = cow_all.iter().copied().filter(|w| !cow.contains(w)).collect();
    let n = cow.len();
    let prefix = stream.prefix(segment_index);
    let result = Mutex::new(SegmentComparison { segment_index, in_place: in_place.len(), cow: n, compared: 0, differing: 0, differing_verdict: 0, bad_only_when_reduced_away: 0, first_differences: Vec::new() });
    let started = std::time::Instant::now();
    for imask in 0..(1u64 << in_place.len()) {
        let mut base_p = prefix.clone();
        for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { base_p[*w] = true; } }
        let rep = stream.observe(base_p.clone());
        if others_persisted { for w in &others { base_p[*w] = true; } }
        let next = AtomicUsize::new(1);
        let limit = (1usize << n) - 1;
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    let mut local = (0u64, 0u64, 0u64, 0u64, Vec::<String>::new());
                    loop {
                        let m = next.fetch_add(1, Ordering::Relaxed);
                        if m > limit { break; }
                        let mut p = base_p.clone();
                        for (b, w) in cow.iter().enumerate() { if m & (1 << b) != 0 { p[*w] = true; } }
                        let o = stream.observe(p);
                        local.0 += 1;
                        if o != rep {
                            local.1 += 1;
                            if o.bad() != rep.bad() { local.2 += 1; if o.bad() { local.3 += 1; } }
                            if local.4.len() < 2 { local.4.push(format!("in_place_mask={imask:#b} chosen_mask={m:#b}: {}", describe_difference(&rep, &o))); }
                        }
                    }
                    let mut r = result.lock().unwrap();
                    r.compared += local.0; r.differing += local.1; r.differing_verdict += local.2; r.bad_only_when_reduced_away += local.3;
                    for d in local.4 { if r.first_differences.len() < 4 { r.first_differences.push(d); } }
                });
            }
        });
        println!("PROGRESS seg={segment_index} imask={imask:#b} states_so_far={} elapsed_seconds={:.1}", result.lock().unwrap().compared, started.elapsed().as_secs_f64());
    }
    result.into_inner().unwrap()
}

#[test]
fn p7_exhaustive_control() {
    let t = threads();
    let plan = std::env::var("PROBE_PLAN").unwrap_or_else(|_| "C13".to_string());
    let started = std::time::Instant::now();
    let (name, stream, seg, chosen, others): (&str, Layer0Stream, usize, Vec<usize>, bool) = match plan.as_str() {
        "C16_small" => ("C", fixed_script(Script::C).stream_for_layer0(), 16, (0..10).collect(), false),
        "C13" => ("C", fixed_script(Script::C).stream_for_layer0(), 13, (0..18).collect(), false),
        "C16" => ("C", fixed_script(Script::C).stream_for_layer0(), 16, (0..16).collect(), false),
        "E18_54_none" | "E18_54_all" => {
            let s = fixed_script(Script::E18).stream_for_layer0();
            let seg = 54usize;
            let cow_all: Vec<usize> = s.segments[seg].iter().copied().filter(|w| s.is_cow(*w)).collect();
            // 复用 A 数据单元（50180）那两次写一定在选中的里面，再补段内前 14 个 COW 写。
            let reuse: Vec<usize> = cow_all.iter().enumerate().filter(|(_, w)| s.writes[**w].offset.0 / 16384 == 50180).map(|(k, _)| k).collect();
            assert_eq!(reuse.len(), 2);
            let mut chosen = reuse.clone();
            for k in 0..cow_all.len() { if chosen.len() >= 16 { break; } if !chosen.contains(&k) { chosen.push(k); } }
            chosen.sort_unstable();
            ("E18", s, seg, chosen, plan == "E18_54_all")
        }
        other => panic!("{other}"),
    };
    println!("CONTROL plan={plan} stream={name} seg={seg} txg={:?} sizes_of_segment={} chosen={:?} others_persisted={others}",
        stream.publish_txg_of(stream.segments[seg][0]), stream.segments[seg].len(), chosen);
    let c = compare_segment_restricted(&stream, seg, &chosen, others, t);
    println!("CONTROL_RESULT plan={plan} compared={} differing={} differing_verdict={} bad_only_when_cow_nonempty={} elapsed_seconds={:.1}",
        c.compared, c.differing, c.differing_verdict, c.bad_only_when_reduced_away, started.elapsed().as_secs_f64());
    for d in &c.first_differences { println!("  CONTROL_DIFF {d}"); }
}

// ───────────── 用户动作放开扫：前缀（A、B，可选挂载时读故障）固定，之后的动作序列 ≤ 3 步，每步取 O（覆盖写）、M（重挂）、U（卸载再重挂）、R（抬 F 到能抬的最高）───────────
impl Sim {
    fn try_raise_floor_to_ceiling(&mut self, content: &[u8]) -> bool {
        // 从现行 txg − 1 往下试，第一个被接受的就是能抬的最高（抬不了就不抬、这一步记成空动作）。
        let current_txg = self.output.root.checkpoint_txg.0;
        for floor in (1..current_txg).rev() {
            let mut current = self.output.clone();
            let mut allocator = self.allocator.clone();
            match raise_rollback_floor(&parameters(), &mut self.devices, &mut allocator, &mut current, CheckpointTxg(floor), ShadowLedger::On) {
                Ok(raised) => {
                    for p in &raised.publishes { self.versions.push(version(p.root.instance.0, p.root.checkpoint_txg.0, content)); }
                    self.output = current; self.allocator = allocator;
                    return true;
                }
                Err(_) => continue,
            }
        }
        false
    }
}

fn sweep_sequences(max_len: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut frontier = vec![String::new()];
    for _ in 0..max_len {
        let mut next = Vec::new();
        for s in &frontier { for c in ['O', 'M', 'U', 'R'] { next.push(format!("{s}{c}")); } }
        out.extend(next.iter().cloned());
        frontier = next;
    }
    out
}

#[test]
fn p11_user_action_sweep() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let max_len: usize = std::env::var("PROBE_SWEEP_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let started = std::time::Instant::now();
    let (mut streams, mut a_misses, mut differing_streams) = (0, 0, 0);
    for fault in [false, true] {
        for seq in sweep_sequences(max_len) {
            let mut sim = Sim::first_transaction();
            sim.overwrite(&second_content(), 1);
            let mut content = second_content();
            let mut instance = 1u32;
            if fault {
                let l0 = sim.stream_for_layer0();
                let ranges: Vec<(DeviceIdentity, u64, u64)> = (0..l0.writes.len())
                    .filter(|i| l0.publish_txg_of(*i) == Some(4) && l0.writes[*i].device == DeviceIdentity(1))
                    .map(|i| (l0.writes[i].device, l0.writes[i].offset.0, l0.writes[i].offset.0 + l0.writes[i].length_in_bytes())).collect();
                sim.faults.set(ranges);
                content = file_content();
                instance = sim.remount(&content).instance.0;
                sim.faults.clear();
            }
            let mut applied = String::new();
            let mut k = 0usize;
            for c in seq.chars() {
                match c {
                    'O' => { content = later_content(80 + k); sim.overwrite(&content, instance); applied.push('O'); }
                    'M' => { instance = sim.remount(&content).instance.0; applied.push('M'); }
                    'U' => { sim.unmount_now(&content); instance = sim.remount(&content).instance.0; applied.push('U'); }
                    'R' => { if sim.try_raise_floor_to_ceiling(&content) { applied.push('R'); } else { applied.push('-'); } }
                    _ => unreachable!(),
                }
                k += 1;
            }
            let stream = sim.stream_for_layer0();
            let t = threads();
            let (mut a_bad, mut full_extra, mut differing) = (0u64, 0u64, 0u64);
            for s in 0..stream.segments.len() {
                let seg = &stream.segments[s];
                let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
                let cow_n = seg.len() - in_place.len();
                let prefix = stream.prefix(s);
                for imask in 0..(1u64 << in_place.len()) {
                    if cow_n == 0 && imask == (1u64 << in_place.len()) - 1 { continue; }
                    let mut p = prefix.clone();
                    for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { p[*w] = true; } }
                    if stream.observe(p).bad() { a_bad += 1; }
                }
                if cow_n > 0 {
                    let c = compare_segment(&stream, s, cow_n <= 8, random, t);
                    full_extra += c.bad_only_when_reduced_away;
                    differing += c.differing;
                }
            }
            if stream.observe(vec![true; stream.writes.len()]).bad() { a_bad += 1; }
            streams += 1;
            if a_bad == 0 && full_extra > 0 { a_misses += 1; }
            if differing > 0 { differing_streams += 1; }
            println!("SWEEP fault={fault} seq={seq} applied={applied} writes={} segments={} arm_a_bad={a_bad} full_sampled_extra_bad={full_extra} differing_states={differing}",
                stream.writes.len(), stream.segments.len());
        }
    }
    println!("SWEEP_TOTAL streams={streams} arm_a_misses_a_red_stream={a_misses} streams_with_any_observation_difference={differing_streams} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
}
