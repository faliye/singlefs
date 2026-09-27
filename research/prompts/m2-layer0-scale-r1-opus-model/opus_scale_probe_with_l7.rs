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

// ───────────── L7：续跑原型 ─────────────
use singlefs_harness::crash::{
    default_slices_for_threads, enumerate_layer0_in_state_slices, enumerate_layer0_resumable, probe_read_progress, resume_plan_slices,
    slice_line, Layer0Parallelism, Layer0SliceLength, Layer0Tally, Layer0WorkerThreadsSource, ResumeMerge, ResumeParser,
};
use std::num::{NonZeroU64, NonZeroUsize};

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(std::env::var("PROBE_SCRATCH").unwrap_or_else(|_| "/tmp/claude-1000/m2-layer0-scale-r1-opus/l7-scratch".into()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn one_shot(stream: &Layer0Stream, versions: &[PublishedVersion], segments: &[Vec<usize>], expand: &dyn Fn(usize, &[usize]) -> bool) -> Layer0Tally {
    enumerate_layer0_in_state_slices(&stream.base, &stream.writes, segments, stream.judged_root_index, versions, expand,
        Layer0Parallelism { worker_threads: NonZeroUsize::MIN, worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller, slice_length: Layer0SliceLength::StatesPerSlice(NonZeroU64::MAX) }, None)
}

fn seeded_versions(stream: &Layer0Stream) -> Vec<PublishedVersion> {
    let mut v = stream.versions.clone();
    v.iter_mut().find(|x| x.instance == InstanceGeneration(1) && x.checkpoint_txg == CheckpointTxg(4)).unwrap().content = third_content();
    v
}

fn run_resumable(stream: &Layer0Stream, versions: &[PublishedVersion], segments: &[Vec<usize>], expand: &dyn Fn(usize, &[usize]) -> bool,
    per: u64, threads: usize, path: &std::path::Path, header: &str, parser: ResumeParser, merge: ResumeMerge, stop: Option<usize>)
    -> (Option<Layer0Tally>, singlefs_harness::crash::ResumeStats) {
    enumerate_layer0_resumable(&stream.base, &stream.writes, segments, stream.judged_root_index, versions, expand,
        NonZeroU64::new(per).unwrap(), threads, path, header, parser, merge, stop, None)
}

#[test]
fn p8_resume_every_byte_prefix_and_line_boundary() {
    let stream = fixed_script(Script::E18).stream_for_layer0();
    let versions = seeded_versions(&stream);
    let expand = |_s: usize, seg: &[usize]| seg.len() < 10;
    let per = 3u64;
    let reference = one_shot(&stream, &versions, &stream.segments, &expand);
    let slices = resume_plan_slices(&stream.writes, &stream.segments, &expand, NonZeroU64::new(per).unwrap());
    println!("L7 plan states={} slices={} reference_violations={} reference_ignored={} first={:?}", reference.states, slices.len(), reference.violations, reference.ignored_violations, reference.first_violation.as_deref().map(|s| s.chars().take(60).collect::<String>()));
    let header = "L0PROGRESS v1 input=probe plan=E18-seeded per=3";
    // 一口气从头跑（严格），文件按跑完的次序写；再拿参照的每片计数造两种次序的整份文件：正序与倒序（模拟并行时后面的片先跑完）。
    let path = scratch("p8-full.progress");
    let _ = std::fs::remove_file(&path);
    let (t, st) = run_resumable(&stream, &versions, &stream.segments, &expand, per, 10, &path, header, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
    assert_eq!(t.as_ref(), Some(&reference), "从头跑一遍与一口气跑完的参照逐项相同");
    println!("L7 fresh_run slices_evaluated={} states_evaluated={}", st.slices_evaluated, st.states_evaluated);
    // 每片的参照计数：逐片单独跑（1 线程、片长 per）取回。
    let mut per_slice: Vec<String> = Vec::new();
    {
        let p = scratch("p8-perslice.progress");
        let _ = std::fs::remove_file(&p);
        let _ = run_resumable(&stream, &versions, &stream.segments, &expand, per, 1, &p, header, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
        let text = std::fs::read_to_string(&p).unwrap();
        for line in text.lines().skip(1) { per_slice.push(format!("{line}\n")); }
        per_slice.sort_by_key(|l| l.split(' ').nth(1).unwrap().parse::<usize>().unwrap());
    }
    assert_eq!(per_slice.len(), slices.len());
    for (order_name, body) in [("forward", per_slice.concat()), ("reverse", per_slice.iter().rev().cloned().collect::<Vec<_>>().concat())] {
        let whole = format!("{header}\n{body}");
        let header_len = header.len() + 1;
        let (mut strict_bad, mut lenient_bad_states, mut lenient_bad_other, mut lenient_extra, mut lenient_silent_loss) = (0u64, 0u64, 0u64, 0u64, 0u64);
        let mut lenient_example = String::new();
        let p = scratch("p8-prefix.progress");
        for len in header_len..=whole.len() {
            let prefix = &whole.as_bytes()[..len];
            let complete_lines: std::collections::BTreeMap<usize, String> = String::from_utf8_lossy(prefix)[header_len..]
                .split_inclusive('\n').filter(|l| l.ends_with('\n'))
                .map(|l| (l.split(' ').nth(1).unwrap().parse::<usize>().unwrap(), l.to_string())).collect();
            for (parser, strict) in [(ResumeParser::Strict, true), (ResumeParser::Lenient, false)] {
                std::fs::write(&p, prefix).unwrap();
                let (got, _) = probe_read_progress(&p, header, &slices, parser);
                let got_ok = got.len() == complete_lines.len() && got.iter().all(|(i, tally)| {
                    let expect = &per_slice[*i];
                    complete_lines.contains_key(i) && slice_line(*i, &slices[*i], tally) == *expect
                });
                if !got_ok {
                    if strict { strict_bad += 1; } else {
                        let extra = got.iter().filter(|(i, _)| !complete_lines.contains_key(i)).count();
                        if extra > 0 { lenient_extra += 1; }
                        // 半行被收下、状态数整、而违例类比真值少：闭式核得过、违例被悄悄丢掉。
                        if got.iter().any(|(i, tally)| {
                            let truth: Vec<&str> = per_slice[*i].split(' ').collect();
                            let field = |k: &str| truth.iter().find_map(|t| t.strip_prefix(k)).and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);
                            !complete_lines.contains_key(i) && tally.states == slices[*i].end - slices[*i].start
                                && (tally.violations < field("violations=") || tally.ignored_violations < field("ignored_violations="))
                        }) { lenient_silent_loss += 1; }
                        let wrong_states = got.iter().any(|(i, tally)| tally.states != slices[*i].end - slices[*i].start);
                        if wrong_states { lenient_bad_states += 1 } else {
                            lenient_bad_other += 1;
                            if lenient_example.is_empty() {
                                let (i, tally) = got.iter().find(|(i, tally)| slice_line(*i, &slices[*i], tally) != per_slice[*i]).unwrap();
                                lenient_example = format!("prefix_len={len} slice={i} parsed_states={} parsed_violations={} parsed_ignored={} true_line_violations_field={}",
                                    tally.states, tally.violations, tally.ignored_violations,
                                    per_slice[*i].split(' ').find(|t| t.starts_with("violations=")).unwrap_or("?"));
                            }
                        }
                    }
                }
            }
        }
        println!("L7 byte_prefixes order={order_name} prefixes={} strict_wrong={strict_bad} lenient_wrong_states={lenient_bad_states} lenient_states_right_but_other_fields_wrong={lenient_bad_other} lenient_accepted_a_half_line={lenient_extra} lenient_half_line_with_full_states_and_fewer_violations={lenient_silent_loss}",
            whole.len() + 1 - header_len);
        if !lenient_example.is_empty() { println!("L7 lenient_example {lenient_example}"); }
        // 每个行边界上真的续跑到底：严格 + 按片号并、严格 + 按文件次序并。
        let mut boundaries = vec![header_len];
        let mut acc = header_len;
        for line in body.split_inclusive('\n') { acc += line.len(); boundaries.push(acc); }
        let (mut by_index_bad, mut file_order_first_wrong, mut file_order_counts_wrong, mut double_or_missing) = (0, 0, 0, 0);
        for b in &boundaries {
            for merge in [ResumeMerge::BySliceIndex, ResumeMerge::FileOrder] {
                std::fs::write(&p, &whole.as_bytes()[..*b]).unwrap();
                let (got, st) = run_resumable(&stream, &versions, &stream.segments, &expand, per, 10, &p, header, ResumeParser::Strict, merge, None);
                let got = got.unwrap();
                let from_file_states: u64 = (0..slices.len()).filter(|i| String::from_utf8_lossy(&whole.as_bytes()[..*b]).contains(&format!("\nS {i} "))).map(|i| slices[i].end - slices[i].start).sum();
                if from_file_states + st.states_evaluated != reference.states { double_or_missing += 1; }
                match merge {
                    ResumeMerge::BySliceIndex => if got != reference { by_index_bad += 1; },
                    ResumeMerge::FileOrder => {
                        if got.first_violation != reference.first_violation || got.first_ignored_violation != reference.first_ignored_violation || got.checker_first_violation != reference.checker_first_violation { file_order_first_wrong += 1; }
                        let mut a = got.clone(); let mut r = reference.clone();
                        a.first_violation = None; r.first_violation = None; a.first_ignored_violation = None; r.first_ignored_violation = None;
                        a.checker_first_violation.clear(); r.checker_first_violation.clear();
                        if a != r { file_order_counts_wrong += 1; }
                    }
                }
            }
        }
        println!("L7 line_boundaries order={order_name} boundaries={} by_slice_index_wrong={by_index_bad} file_order_first_violation_wrong={file_order_first_wrong} file_order_counts_wrong={file_order_counts_wrong} states_double_or_missing={double_or_missing}",
            boundaries.len());
    }
}

#[test]
fn p10_weak_keys_threads_versions_observer() {
    let stream = fixed_script(Script::E18).stream_for_layer0();
    let clean = stream.versions.clone();
    let seeded = seeded_versions(&stream);
    let expand = |_s: usize, seg: &[usize]| seg.len() < 10;
    let reference_clean = one_shot(&stream, &clean, &stream.segments, &expand);
    let reference_seeded = one_shot(&stream, &seeded, &stream.segments, &expand);
    // (a) 键里没有版本表：干净版本表跑完的进度，给种了违例的版本表续跑。
    let weak = "L0PROGRESS v1 input=probe stream=E18 expand=lt10 per=3";
    let p = scratch("p10-weak.progress");
    let _ = std::fs::remove_file(&p);
    let _ = run_resumable(&stream, &clean, &stream.segments, &expand, 3, 10, &p, weak, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
    let (got, st) = run_resumable(&stream, &seeded, &stream.segments, &expand, 3, 10, &p, weak, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
    let got = got.unwrap();
    println!("L7 weak_key_versions resumed_violations={} truth_violations={} resumed_states={} closed_form_check_passes={} slices_from_file={} states_evaluated={}",
        got.violations, reference_seeded.violations, got.states, got.states == reference_seeded.states, st.slices_from_file, st.states_evaluated);
    assert_eq!(got, reference_clean);
    // (b) 片方案不进键：今天的默认切法随线程数变。先 4 线程跑一半停下，再 8 线程接着跑，头行不带片长。
    let state_count = reference_clean.states;
    let s4 = default_slices_for_threads(state_count, 4);
    let s8 = default_slices_for_threads(state_count, 8);
    println!("L7 default_slicing state_count={state_count} slices_at_4_threads={} first={:?} slices_at_8_threads={} first={:?}", s4.len(), s4.first(), s8.len(), s8.first());
    let (big, big_segments) = kill_plan();
    let expand_big = kill_expand;
    let n_big = resume_plan_slices(&big.writes, &big_segments, &expand_big, NonZeroU64::new(u64::MAX).unwrap())[0].end;
    let a = default_slices_for_threads(n_big, 4);
    let b = default_slices_for_threads(n_big, 32);
    println!("L7 default_slicing_bigger state_count={n_big} per_slice_at_4_threads={} per_slice_at_32_threads={}", a[0].end - a[0].start, b[0].end - b[0].start);
    let header_no_per = "L0PROGRESS v1 input=probe stream=C expand=lt20";
    let p2 = scratch("p10-threads.progress");
    let _ = std::fs::remove_file(&p2);
    let per4 = a[0].end - a[0].start;
    let per32 = b[0].end - b[0].start;
    let _ = run_resumable(&big, &big.versions, &big_segments, &expand_big, per4, 10, &p2, header_no_per, ResumeParser::Lenient, ResumeMerge::BySliceIndex, Some(a.len() / 2));
    let (got2, st2) = run_resumable(&big, &big.versions, &big_segments, &expand_big, per32, 10, &p2, header_no_per, ResumeParser::Lenient, ResumeMerge::BySliceIndex, None);
    let truth2 = one_shot(&big, &big.versions, &big_segments, &expand_big);
    println!("L7 thread_change_lenient_no_per resumed_states={} truth_states={} slices_from_file={} states_evaluated={} equal={}",
        got2.as_ref().map(|t| t.states).unwrap_or(0), truth2.states, st2.slices_from_file, st2.states_evaluated, got2.as_ref() == Some(&truth2));
    let _ = std::fs::remove_file(&p2);
    let _ = run_resumable(&big, &big.versions, &big_segments, &expand_big, per4, 10, &p2, header_no_per, ResumeParser::Strict, ResumeMerge::BySliceIndex, Some(a.len() / 2));
    let (got3, st3) = run_resumable(&big, &big.versions, &big_segments, &expand_big, per32, 10, &p2, header_no_per, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
    println!("L7 thread_change_strict_no_per resumed_equal={} slices_from_file={} truncated_bytes={} states_evaluated={}",
        got3.as_ref() == Some(&truth2), st3.slices_from_file, st3.truncated_bytes, st3.states_evaluated);
    // (c) 观察者：续跑时已完成的片不再交给观察者。
    let p3 = scratch("p10-observer.progress");
    let _ = std::fs::remove_file(&p3);
    let _ = run_resumable(&stream, &clean, &stream.segments, &expand, 3, 10, &p3, weak, ResumeParser::Strict, ResumeMerge::BySliceIndex, Some(10));
    let mut observed = 0u64;
    let mut obs = |_: &CrashImage<'_>, _: &RecoveryReport| { observed += 1; };
    let (got4, _) = enumerate_layer0_resumable(&stream.base, &stream.writes, &stream.segments, stream.judged_root_index, &clean, &expand,
        NonZeroU64::new(3).unwrap(), 10, &p3, weak, ResumeParser::Strict, ResumeMerge::BySliceIndex, None, Some(&mut obs));
    println!("L7 observer resumed_tally_equal={} observer_saw={} states={}", got4.as_ref() == Some(&reference_clean), observed, reference_clean.states);
    // (d) 两个进程同时往同一份进度文件追加（同一批输入、同一个 TMPDIR）：各跑一半的片、再合并。
    let p5 = scratch("p10-concurrent.progress");
    let _ = std::fs::remove_file(&p5);
    let _ = run_resumable(&stream, &clean, &stream.segments, &expand, 3, 10, &p5, weak, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
    let text = std::fs::read_to_string(&p5).unwrap();
    let lines: Vec<&str> = text.lines().skip(1).collect();
    let doubled = format!("{weak}\n{}\n{}\n", lines.join("\n"), lines[..lines.len() / 2].join("\n"));
    std::fs::write(&p5, doubled).unwrap();
    let (got5, st5) = run_resumable(&stream, &clean, &stream.segments, &expand, 3, 10, &p5, weak, ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
    println!("L7 duplicate_lines resumed_equal={} duplicate_slice_lines_skipped={}", got5.as_ref() == Some(&reference_clean), st5.duplicate_slice_lines);
}

/// 真杀：子进程跑续跑原型，父进程在几个时刻 SIGKILL 它，再从进度文件续跑到底，与一口气跑完的比。
fn kill_plan() -> (Layer0Stream, Vec<Vec<usize>>) {
    let stream = fixed_script(Script::C).stream_for_layer0();
    // 把写行那次发布的 18 写单元段（第 13 段，纯 COW）拆成 14 + 4 两段：只为了把状态数压到两万上下，续跑的记账与段模型无关。
    let mut segments = stream.segments.clone();
    let seg13 = segments[13].clone();
    assert_eq!(seg13.len(), 18);
    segments.splice(13..14, [seg13[..14].to_vec(), seg13[14..].to_vec()]);
    (stream, segments)
}
fn kill_expand(_s: usize, seg: &[usize]) -> bool { seg.len() < 15 }

#[test]
fn p9_child_resumable_run() {
    let Ok(path) = std::env::var("PROBE_CHILD_PROGRESS") else { return };
    let (stream, segments) = kill_plan();
    let _ = run_resumable(&stream, &stream.versions, &segments, &kill_expand, 64, 10, std::path::Path::new(&path), "L0PROGRESS v1 kill-plan per=64", ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
}

#[test]
fn p9_kill_and_resume() {
    let (stream, segments) = kill_plan();
    let reference = one_shot(&stream, &stream.versions, &segments, &kill_expand);
    println!("L7 kill_plan states={} closed_form_of_expanded={}", reference.states,
        closed_form_state_count(&segments.iter().filter(|s| s.len() < 15).cloned().collect::<Vec<_>>()));
    let exe = std::env::current_exe().unwrap();
    let delays: Vec<u64> = std::env::var("PROBE_KILL_MS").unwrap_or_else(|_| "300,1500,4000,8000,12000".into()).split(',').map(|x| x.parse().unwrap()).collect();
    for delay in delays {
        let path = scratch(&format!("p9-kill-{delay}.progress"));
        let _ = std::fs::remove_file(&path);
        let mut child = std::process::Command::new(&exe)
            .args(["--exact", "p9_child_resumable_run", "--nocapture"])
            .env("PROBE_CHILD_PROGRESS", &path)
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(delay));
        let exited_before_kill = child.try_wait().unwrap().is_some();
        let _ = child.kill();
        let status = child.wait().unwrap();
        let bytes = std::fs::read(&path).map(|b| b.len()).unwrap_or(0);
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let complete_lines = text.matches('\n').count().saturating_sub(1);
        let half_line = !text.is_empty() && !text.ends_with('\n');
        let (got, st) = run_resumable(&stream, &stream.versions, &segments, &kill_expand, 64, 10, &path, "L0PROGRESS v1 kill-plan per=64", ResumeParser::Strict, ResumeMerge::BySliceIndex, None);
        println!("L7 kill delay_ms={delay} exited_before_kill={exited_before_kill} status={status} file_bytes={bytes} complete_slice_lines={complete_lines} half_line_at_end={half_line} resumed_from_file={} evaluated={} truncated_bytes={} equal_to_one_shot={}",
            st.slices_from_file, st.slices_evaluated, st.truncated_bytes, got.as_ref() == Some(&reference));
    }
}
