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

// ═════════════ m2-layer0-scale-r2 云端攻方（Opus）追加：甲二的等价（M1）═════════════
// 判红用层 0 自己的评估函数 `evaluate_state_for_versions`（与 `enumerate_layer0*` 每个状态调的是同一个），
// 红 = 计数里任何一项违例非 0（oracle 两遍、记录核对器两条、checker 任一条）。
use singlefs_harness::crash::{enumerate_layer0_selecting_versions, evaluate_state_for_versions, Layer0Tally};

pub fn tally_red(t: &Layer0Tally) -> bool {
    t.violations > 0 || t.ignored_violations > 0 || t.record_root_without_record > 0
        || t.record_claimed_state_missing_unit > 0 || t.checker_violated_states.values().any(|n| *n > 0)
}
pub fn tally_why(t: &Layer0Tally) -> String {
    let mut parts = Vec::new();
    if let Some(v) = &t.first_violation { parts.push(format!("oracle={}", v.chars().take(160).collect::<String>())); }
    if let Some(v) = &t.first_ignored_violation { parts.push(format!("oracle_ignored={}", v.chars().take(120).collect::<String>())); }
    if t.record_root_without_record > 0 { parts.push("record:root_without_record".into()); }
    if t.record_claimed_state_missing_unit > 0 { parts.push("record:claimed_state_missing_unit".into()); }
    for (i, d) in &t.checker_first_violation { parts.push(format!("{i}:{}", d.chars().take(200).collect::<String>())); }
    parts.join(" | ")
}
/// 层 0 的评估函数评一个状态。
pub fn l0_eval(stream: &Layer0Stream, persisted: Vec<bool>) -> Layer0Tally {
    let mut t = Layer0Tally::default();
    let _ = evaluate_state_for_versions(&stream.base, &stream.writes, persisted, stream.judged_root_index, &stream.versions, &mut t);
    t
}

/// 甲二的全部状态：每段每个原地子集 I × COW ∈ {∅, 全集}；整段全落那一个归下一段（最后加全部持久那一个）。
/// 返回 (状态数, 红的状态数, 前几条红的原因)。
pub fn arm_a2_all_states(stream: &Layer0Stream, threads: usize) -> (u64, u64, Vec<String>) {
    let mut states: Vec<(usize, u64, bool, Vec<bool>)> = Vec::new();
    for s in 0..stream.segments.len() {
        let seg = &stream.segments[s];
        let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
        let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
        let prefix = stream.prefix(s);
        let full_i = (1u64 << in_place.len()) - 1;
        for imask in 0..=full_i {
            let mut p = prefix.clone();
            for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { p[*w] = true; } }
            if !(cow.is_empty() && imask == full_i) { states.push((s, imask, false, p.clone())); }
            if !cow.is_empty() && imask != full_i {
                let mut q = p.clone(); for w in &cow { q[*w] = true; }
                states.push((s, imask, true, q));
            }
        }
    }
    states.push((stream.segments.len(), 0, true, vec![true; stream.writes.len()]));
    let next = AtomicUsize::new(0);
    let out = Mutex::new((0u64, 0u64, Vec::<String>::new()));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::Relaxed);
                let Some((s, imask, all, p)) = states.get(k) else { break };
                let t = l0_eval(stream, p.clone());
                let mut o = out.lock().unwrap();
                o.0 += 1;
                if tally_red(&t) { o.1 += 1; if o.2.len() < 4 { o.2.push(format!("seg={s} imask={imask:#b} cow_all={all} {}", tally_why(&t))); } }
            });
        }
    });
    out.into_inner().unwrap()
}

// ───────────── M1 搜索：段内 COW 子集上判定不单调（∅ 与全集都不红、某个真子集红）的形状 ─────────────
fn overlaps(a: &RetainedWrite, b: &RetainedWrite) -> bool {
    a.device == b.device && a.offset.0 < b.offset.0 + b.length_in_bytes() && b.offset.0 < a.offset.0 + a.length_in_bytes()
}

/// 便宜判定（不跑恢复）：记录核对器两条 + 池级 checker；effective_root 用整段 COW 取 ∅ 那一个状态恢复出来的（段里没有 journal 记录时不随 COW 变）。
pub fn cheap_red(stream: &Layer0Stream, persisted: &[bool], eff: Option<(InstanceGeneration, CheckpointTxg)>) -> Option<String> {
    let image = CrashImage { base: &stream.base, writes: &stream.writes, persisted: persisted.to_vec() };
    let r = check_records(&image, eff);
    let mut parts = Vec::new();
    if r.root_without_record { parts.push("record:root_without_record".to_string()); }
    if r.claimed_state_missing_unit { parts.push("record:claimed_state_missing_unit".to_string()); }
    for (i, v) in check_pool_image(&image) { if let InvariantVerdict::Violated(d) = v { parts.push(format!("{i}:{}", d.chars().take(200).collect::<String>())); } }
    if parts.is_empty() { None } else { Some(parts.join(" | ")) }
}

/// 段 s 里跟之前的单元写相交的 COW 写，按「经相交连成一片」分成若干组（组里含之前的写，用来连通；返回的只是段内的）。
pub fn relevant_components(stream: &Layer0Stream, s: usize) -> Vec<Vec<usize>> {
    let seg = &stream.segments[s];
    let start = seg[0];
    let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
    let earlier: Vec<usize> = (0..start).filter(|i| stream.writes[*i].kind == StepKind::UnitWrite).collect();
    let rel: Vec<usize> = cow.iter().copied().filter(|w| earlier.iter().any(|e| overlaps(&stream.writes[*w], &stream.writes[*e]))).collect();
    // 连通：两段内写经同一个之前的写（或彼此）相交就连起来。
    let mut comp: Vec<usize> = (0..rel.len()).collect();
    fn find(c: &mut Vec<usize>, x: usize) -> usize { if c[x] != x { let r = find(c, c[x]); c[x] = r; } c[x] }
    for i in 0..rel.len() { for j in i + 1..rel.len() {
        let (a, b) = (&stream.writes[rel[i]], &stream.writes[rel[j]]);
        let mirrored = a.device != b.device && a.offset == b.offset;
        let linked = mirrored || overlaps(a, b) || earlier.iter().any(|e| overlaps(a, &stream.writes[*e]) && overlaps(b, &stream.writes[*e]))
            // 隔一层：a 盖到的之前的写 e1 与 b 盖到的之前的写 e2 相交
            || earlier.iter().any(|e1| overlaps(a, &stream.writes[*e1]) && earlier.iter().any(|e2| overlaps(b, &stream.writes[*e2]) && overlaps(&stream.writes[*e1], &stream.writes[*e2])));
        if linked { let (ri, rj) = (find(&mut comp, i), find(&mut comp, j)); comp[ri] = rj; }
    } }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..rel.len() { let r = find(&mut comp, i); groups.entry(r).or_default().push(rel[i]); }
    groups.into_values().collect()
}

/// 在每段、每个原地子集 I 上，对每一组相交的 COW 写全枚举子集（组外的 COW 写取 ∅ 或全集两遍），便宜判定不单调就记下。
pub struct NonMonotone { pub seg: usize, pub imask: u64, pub others_all: bool, pub group: Vec<usize>, pub mask: u64, pub why: String }
pub fn search_non_monotone(stream: &Layer0Stream, max_group: usize, in_place_all_only: bool) -> (u64, Vec<NonMonotone>, usize) {
    let mut found = Vec::new();
    let mut checked = 0u64;
    let mut skipped_groups = 0usize;
    for s in 0..stream.segments.len() {
        let seg = &stream.segments[s];
        let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
        if cow.is_empty() { continue; }
        let groups = relevant_components(stream, s);
        if groups.is_empty() { continue; }
        let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
        let full_i = (1u64 << in_place.len()) - 1;
        let prefix = stream.prefix(s);
        let imasks: Vec<u64> = if in_place_all_only { vec![full_i] } else { (0..=full_i).collect() };
        for imask in imasks {
            let mut base_p = prefix.clone();
            for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { base_p[*w] = true; } }
            let eff = recover(&CrashImage { base: &stream.base, writes: &stream.writes, persisted: base_p.clone() }, JournalPolicy::Consult).effective_root;
            for group in &groups {
                if group.len() > max_group { skipped_groups += 1; continue; }
                for others_all in [false, true] {
                    let mut p0 = base_p.clone();
                    if others_all { for w in &cow { if !group.contains(w) { p0[*w] = true; } } }
                    let full = (1u64 << group.len()) - 1;
                    let red_at = |m: u64| { let mut p = p0.clone(); for (b, w) in group.iter().enumerate() { if m & (1 << b) != 0 { p[*w] = true; } } cheap_red(stream, &p, eff) };
                    let (lo, hi) = (red_at(0), red_at(full));
                    checked += 2;
                    if lo.is_some() || hi.is_some() { continue; }
                    for m in 1..full {
                        checked += 1;
                        if let Some(why) = red_at(m) { found.push(NonMonotone { seg: s, imask, others_all, group: group.clone(), mask: m, why }); break; }
                    }
                }
            }
        }
    }
    (checked, found, skipped_groups)
}

/// 历史：前缀 A、B，之后按字母走：O 覆盖写、U 卸载再重挂、M 进程重开重挂、R 抬 F 到能抬的最高。
pub fn history(seq: &str) -> (Sim, String) {
    let mut sim = Sim::first_transaction();
    sim.overwrite(&second_content(), 1);
    let mut content = second_content();
    let mut instance = 1u32;
    let mut applied = String::new();
    for (k, c) in seq.chars().enumerate() {
        match c {
            'O' => { content = later_content(80 + k); sim.overwrite(&content, instance); applied.push('O'); }
            'B' => { content = (0..30_000 + k).map(|i| u8::try_from((i * 3 + k) % 241).unwrap()).collect(); sim.overwrite(&content, instance); applied.push('B'); }
            'S' => { content = vec![u8::try_from(k % 200).unwrap() + 1; 10 + k]; sim.overwrite(&content, instance); applied.push('S'); }
            'M' => { instance = sim.remount(&content).instance.0; applied.push('M'); }
            'U' => { sim.unmount_now(&content); instance = sim.remount(&content).instance.0; applied.push('U'); }
            'R' => { if sim.try_raise_floor_to_ceiling(&content) { applied.push('R'); } else { applied.push('-'); } }
            // 变异替身：把当前分配器的回收窗口置 0（C22「刚释放的块立即重分配」那一类），到下一次挂载为止。
            'Z' => { sim.allocator.set_reuse_window(singlefs_core::allocator::ReuseWindow::ForcedToZero); applied.push('Z'); }
            _ => unreachable!(),
        }
    }
    (sim, applied)
}

fn all_seqs(alphabet: &str, min_len: usize, max_len: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut frontier = vec![String::new()];
    for len in 1..=max_len {
        let mut next = Vec::new();
        for s in &frontier { for c in alphabet.chars() { next.push(format!("{s}{c}")); } }
        if len >= min_len { out.extend(next.iter().cloned()); }
        frontier = next;
    }
    out
}

/// 把一个不单调的点摆成完整持久向量：段前全持久、段内原地按 imask、组外 COW 按 others_all、组内按 mask。
pub fn state_of(stream: &Layer0Stream, nm: &NonMonotone, mask: u64) -> Vec<bool> {
    let seg = &stream.segments[nm.seg];
    let in_place: Vec<usize> = seg.iter().copied().filter(|w| !stream.is_cow(*w)).collect();
    let cow: Vec<usize> = seg.iter().copied().filter(|w| stream.is_cow(*w)).collect();
    let mut p = stream.prefix(nm.seg);
    for (b, w) in in_place.iter().enumerate() { if nm.imask & (1 << b) != 0 { p[*w] = true; } }
    if nm.others_all { for w in &cow { if !nm.group.contains(w) { p[*w] = true; } } }
    for (b, w) in nm.group.iter().enumerate() { if mask & (1 << b) != 0 { p[*w] = true; } }
    p
}

pub fn describe_write(stream: &Layer0Stream, w: usize) -> String {
    let x = &stream.writes[w];
    format!("#{w}(dev{} slot{}+{} txg{:?})", x.device.0, x.offset.0 / 16384, x.length_in_bytes() / 16384, stream.publish_txg_of(w))
}

#[test]
fn q1_search_non_monotone_cow_subsets_over_histories() {
    let t = threads();
    let started = std::time::Instant::now();
    let seqs: Vec<String> = if let Ok(list) = std::env::var("PROBE_SEQS") { list.split(',').map(str::to_string).collect() } else if let Ok(n) = std::env::var("PROBE_N") {
        let n: usize = n.parse().unwrap();
        let max_len: usize = std::env::var("PROBE_MAX_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
        let seed: u64 = std::env::var("PROBE_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let alphabet: Vec<char> = std::env::var("PROBE_ALPHABET").unwrap_or_else(|_| "OUMRS".into()).chars().collect();
        let mut st = 0x2545_f491_4f6c_dd1du64 ^ seed;
        (0..n).map(|_| { let len = 1 + (xorshift(&mut st) % max_len as u64) as usize; xorshift_seq(&mut st, &alphabet, len) }).collect()
    } else {
        let alphabet = std::env::var("PROBE_ALPHABET").unwrap_or_else(|_| "OU".to_string());
        let min_len: usize = std::env::var("PROBE_MIN_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
        let max_len: usize = std::env::var("PROBE_MAX_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
        all_seqs(&alphabet, min_len, max_len)
    };
    let all_in_place: bool = std::env::var("PROBE_ALL_IMASKS").map(|v| v != "1").unwrap_or(true);
    let verify_limit: usize = std::env::var("PROBE_VERIFY").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let (mut n_streams, mut n_found_streams, mut n_checked, mut verified) = (0usize, 0usize, 0u64, 0usize);
    for seq in &seqs {
        let (sim, applied) = history(seq);
        let stream = sim.stream_for_layer0();
        let (checked, found, skipped) = search_non_monotone(&stream, 14, all_in_place);
        n_streams += 1; n_checked += checked;
        println!("Q1 seq={seq} applied={applied} writes={} segments={} closed_form={} cheap_checked={checked} skipped_groups={skipped} non_monotone={}",
            stream.writes.len(), stream.segments.len(), closed_form_state_count(&stream.segments), found.len());
        if found.is_empty() { continue; }
        n_found_streams += 1;
        let mut by_why: BTreeMap<String, usize> = BTreeMap::new();
        for nm in &found { let key: String = nm.why.split(" | ").map(|p| p.split(':').next().unwrap_or("").to_string()).collect::<Vec<_>>().join("+"); *by_why.entry(key).or_insert(0) += 1; }
        println!("  NM_KINDS seq={seq} {by_why:?} is_exemption_chain_stream={}", !exemption_chain_candidates(&stream).is_empty());
        for nm in found.iter().take(3) {
            let g: Vec<String> = nm.group.iter().map(|w| describe_write(&stream, *w)).collect();
            println!("  NM seq={seq} seg={} txg={:?} imask={:#b} others_all={} group=[{}] mask={:#b} why={}",
                nm.seg, stream.publish_txg_of(stream.segments[nm.seg][0]), nm.imask, nm.others_all, g.join(","), nm.mask, nm.why);
        }
        if verified < verify_limit {
            verified += 1;
            let nm = &found[0];
            let full = (1u64 << nm.group.len()) - 1;
            for (label, m) in [("empty", 0u64), ("partial", nm.mask), ("all", full)] {
                let tl = l0_eval(&stream, state_of(&stream, nm, m));
                println!("  VERIFY seq={seq} {label} red={} {}", tally_red(&tl), tally_why(&tl));
            }
            let (n, bad, first) = arm_a2_all_states(&stream, t);
            println!("  ARM_A2 seq={seq} states={n} red={bad}");
            for f in first { println!("    ARM_A2_RED {f}"); }
        }
    }
    println!("Q1_TOTAL streams={n_streams} streams_with_non_monotone={n_found_streams} cheap_checked={n_checked} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
}

/// 相交链的静态账：段内 COW 写 w 盖到之前的单元写 e，e 又盖到更早的单元写 u 且 [e] 伸出 [u] 之外（跨度不对齐）。
#[test]
fn q2_overlap_chains() {
    let seqs: Vec<String> = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "UOOOUOOOUOOO".into()).split(',').map(str::to_string).collect();
    for seq in &seqs {
        let (sim, _) = history(seq);
        let st = sim.stream_for_layer0();
        let n = st.writes.len();
        let (mut gen2, mut misaligned2, mut chain3) = (0, 0, 0);
        for w in 0..n {
            if st.writes[w].kind != StepKind::UnitWrite { continue; }
            for e in 0..w {
                if st.writes[e].kind != StepKind::UnitWrite || !overlaps(&st.writes[w], &st.writes[e]) { continue; }
                gen2 += 1;
                let (a, b) = (&st.writes[w], &st.writes[e]);
                if a.offset.0 != b.offset.0 || a.length_in_bytes() != b.length_in_bytes() {
                    misaligned2 += 1;
                    if misaligned2 <= 6 { println!("  MISALIGNED seq={seq} {} over {}", describe_write(&st, w), describe_write(&st, e)); }
                }
                for u in 0..e {
                    if st.writes[u].kind != StepKind::UnitWrite || !overlaps(&st.writes[e], &st.writes[u]) { continue; }
                    chain3 += 1;
                    if chain3 <= 6 { println!("  CHAIN3 seq={seq} {} over {} over {}", describe_write(&st, w), describe_write(&st, e), describe_write(&st, u)); }
                }
            }
        }
        println!("Q2 seq={seq} writes={n} overlapping_pairs={gen2} misaligned_pairs={misaligned2} chains_of_three={chain3}");
    }
}

/// 记录核对器豁免链的静态配方：段 σ 里有 COW 写 x 盖到之前的单元写 l（l 盖过更早、已被认领的单元写 u，x 本身不碰 u），
/// 同一段里又有 COW 写 y 盖到 u。x 落、y 不落时 u 的这一份「不在、也没有在位的后写盖它」。
pub fn exemption_chain_candidates(st: &Layer0Stream) -> Vec<(usize, usize, usize, usize, usize)> {
    let mut out = Vec::new();
    for (s, seg) in st.segments.iter().enumerate() {
        let start = seg[0];
        let cow: Vec<usize> = seg.iter().copied().filter(|w| st.is_cow(*w)).collect();
        if cow.is_empty() { continue; }
        for u in 0..start {
            if st.writes[u].kind != StepKind::UnitWrite { continue; }
            // u 的发布的根要在 σ 之前（被认领）
            let Some(root_of_u) = (u..st.writes.len()).find(|i| st.writes[*i].kind == StepKind::RootRecordFua) else { continue };
            if root_of_u >= start { continue; }
            let ys: Vec<usize> = cow.iter().copied().filter(|y| overlaps(&st.writes[*y], &st.writes[u])).collect();
            if ys.is_empty() { continue; }
            for l in u + 1..start {
                if st.writes[l].kind != StepKind::UnitWrite || !overlaps(&st.writes[l], &st.writes[u]) { continue; }
                for x in &cow {
                    if overlaps(&st.writes[*x], &st.writes[l]) && !overlaps(&st.writes[*x], &st.writes[u]) {
                        out.push((s, u, l, *x, ys[0]));
                    }
                }
            }
        }
    }
    out
}

fn xorshift_seq(state: &mut u64, alphabet: &[char], len: usize) -> String {
    (0..len).map(|_| alphabet[(xorshift(state) % alphabet.len() as u64) as usize]).collect()
}

#[test]
fn q3_static_exemption_chains_over_sampled_histories() {
    let started = std::time::Instant::now();
    let n: usize = std::env::var("PROBE_N").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let max_len: usize = std::env::var("PROBE_MAX_LEN").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
    let seed: u64 = std::env::var("PROBE_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let alphabet: Vec<char> = std::env::var("PROBE_ALPHABET").unwrap_or_else(|_| "OUMR".into()).chars().collect();
    let mut st = 0x2545_f491_4f6c_dd1du64 ^ seed;
    let mut hits = 0usize;
    for k in 0..n {
        let len = 1 + (xorshift(&mut st) % max_len as u64) as usize;
        let seq = std::env::var("PROBE_SEQS").ok().map(|v| v.split(',').nth(k).unwrap_or("O").to_string()).unwrap_or_else(|| xorshift_seq(&mut st, &alphabet, len));
        let (sim, applied) = history(&seq);
        let stream = sim.stream_for_layer0();
        let c = exemption_chain_candidates(&stream);
        println!("Q3 k={k} seq={seq} applied={applied} writes={} segments={} chain_candidates={}", stream.writes.len(), stream.segments.len(), c.len());
        if !c.is_empty() {
            hits += 1;
            for (s, u, l, x, y) in c.iter().take(3) {
                println!("  CHAIN seq={seq} seg={s} u={} l={} x={} y={}", describe_write(&stream, *u), describe_write(&stream, *l), describe_write(&stream, *x), describe_write(&stream, *y));
            }
        }
    }
    println!("Q3_TOTAL histories={n} with_candidates={hits} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
}

/// 豁免链候选的逐项诊断：段 σ 的 ∅、只落 x（两盘）、全集三个状态上记录核对器的结论，与盖过 u 的每一次写在 ∅ 状态上在不在位。
#[test]
fn q4_diagnose_exemption_chain() {
    let seqs: Vec<String> = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "USUOOSSUURMSR".into()).split(',').map(str::to_string).collect();
    for seq in &seqs {
        let (sim, _) = history(seq);
        let st = sim.stream_for_layer0();
        let cands = exemption_chain_candidates(&st);
        for (s, u, l, x, y) in cands.iter().take(1) {
            let seg = &st.segments[*s];
            let cow: Vec<usize> = seg.iter().copied().filter(|w| st.is_cow(*w)).collect();
            let mut p0 = st.prefix(*s);
            for w in seg { if !st.is_cow(*w) { p0[*w] = true; } }
            let eff = recover(&CrashImage { base: &st.base, writes: &st.writes, persisted: p0.clone() }, JournalPolicy::Consult).effective_root;
            println!("Q4 seq={seq} seg={s} txg={:?} eff={:?} u={} l={} x={} y={}", st.publish_txg_of(seg[0]), eff, describe_write(&st, *u), describe_write(&st, *l), describe_write(&st, *x), describe_write(&st, *y));
            // 盖过 u 的写（σ 之前）
            let image0 = CrashImage { base: &st.base, writes: &st.writes, persisted: p0.clone() };
            use singlefs_core::recovery::PoolReader;
            let in_place = |img: &CrashImage<'_>, i: usize| { let w = &st.writes[i]; img.read(w.device, w.offset, w.length_in_bytes() as usize).is_some_and(|b| w.contents.still_on_disk(&b)) };
            for i in 0..seg[0] { if st.writes[i].kind == StepKind::UnitWrite && overlaps(&st.writes[i], &st.writes[*u]) { println!("  OVER_U {} in_place_at_empty={}", describe_write(&st, i), in_place(&image0, i)); } }
            // x 与 y 的镜像（另一盘同槽）
            let twin = |w: usize| cow.iter().copied().find(|c| *c != w && st.writes[*c].offset == st.writes[w].offset && st.writes[*c].device != st.writes[w].device);
            let xs: Vec<usize> = std::iter::once(*x).chain(twin(*x)).collect();
            let ys: Vec<usize> = std::iter::once(*y).chain(twin(*y)).collect();
            let mut px = p0.clone(); for w in &xs { px[*w] = true; }
            let mut pall = p0.clone(); for w in &cow { pall[*w] = true; }
            let mut pall_minus_y = pall.clone(); for w in &ys { pall_minus_y[*w] = false; }
            for (label, p) in [("empty", p0.clone()), ("x_only", px), ("all", pall), ("all_minus_y", pall_minus_y)] {
                let r = check_records(&CrashImage { base: &st.base, writes: &st.writes, persisted: p.clone() }, eff);
                let t = l0_eval(&st, p);
                println!("  STATE {label} records={r:?} l0_red={} {}", tally_red(&t), tally_why(&t));
            }
        }
    }
}

/// 候选的动态坐实：段 σ 原地写全落、COW 分别取 ∅、只落 x 与它的镜像、全集，用层 0 的评估函数判；∅ 与全集都不红、只落 x 红才算。
pub fn confirm_chain(st: &Layer0Stream) -> Option<(usize, usize, usize, usize, usize, String)> {
    for (s, u, l, x, y) in exemption_chain_candidates(st) {
        let seg = &st.segments[s];
        let cow: Vec<usize> = seg.iter().copied().filter(|w| st.is_cow(*w)).collect();
        let mut p0 = st.prefix(s);
        for w in seg { if !st.is_cow(*w) { p0[*w] = true; } }
        let twin = |w: usize| cow.iter().copied().find(|c| *c != w && st.writes[*c].offset == st.writes[w].offset && st.writes[*c].device != st.writes[w].device);
        let mut px = p0.clone(); px[x] = true; if let Some(t) = twin(x) { px[t] = true; }
        let mut pall = p0.clone(); for w in &cow { pall[*w] = true; }
        let (te, tx, ta) = (l0_eval(st, p0), l0_eval(st, px), l0_eval(st, pall));
        if !tally_red(&te) && tally_red(&tx) && !tally_red(&ta) { return Some((s, u, l, x, y, tally_why(&tx))); }
    }
    None
}

#[test]
fn q5_confirm_and_shrink() {
    let seqs: Vec<String> = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "USUOOSSUURMSR".into()).split(',').map(str::to_string).collect();
    let shrink = std::env::var("PROBE_SHRINK").map(|v| v == "1").unwrap_or(false);
    for seq in &seqs {
        let (sim, applied) = history(seq);
        let st = sim.stream_for_layer0();
        let c = confirm_chain(&st);
        println!("Q5 seq={seq} applied={applied} candidates={} confirmed={}", exemption_chain_candidates(&st).len(), c.is_some());
        if let Some((s, u, l, x, y, why)) = &c {
            println!("  CONFIRMED seq={seq} seg={s} txg={:?} u={} l={} x={} y={} why={why}", st.publish_txg_of(st.segments[*s][0]), describe_write(&st, *u), describe_write(&st, *l), describe_write(&st, *x), describe_write(&st, *y));
        }
        if !(shrink && c.is_some()) { continue; }
        let mut cur: Vec<char> = seq.chars().collect();
        loop {
            let mut improved = false;
            for i in 0..cur.len() {
                let cand: String = cur.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, ch)| *ch).collect();
                let (sim2, _) = history(&cand);
                if confirm_chain(&sim2.stream_for_layer0()).is_some() { cur = cand.chars().collect(); improved = true; println!("  SHRINK -> {cand}"); break; }
            }
            if !improved { break; }
        }
        println!("  SHRUNK seq={seq} minimal={}", cur.iter().collect::<String>());
    }
}

/// 坐实之后的整条流：甲二的全部状态（层 0 评估函数逐个评）、σ 段的写种类与状态数；σ 不超过 20 个写时用层 0 的枚举器只展开 σ 段全枚举。
#[test]
fn q6_whole_stream_arms_and_segment_enumeration() {
    let t = threads();
    let seqs: Vec<String> = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "UOOUOMSU".into()).split(',').map(str::to_string).collect();
    for seq in &seqs {
        let started = std::time::Instant::now();
        let (sim, applied) = history(seq);
        let st = sim.stream_for_layer0();
        let Some((s, u, l, x, y, why)) = confirm_chain(&st) else { println!("Q6 seq={seq} not_confirmed"); continue };
        let seg = &st.segments[s];
        let kinds: BTreeMap<&str, usize> = seg.iter().fold(BTreeMap::new(), |mut m, w| { *m.entry(st.writes[*w].kind.name()).or_insert(0) += 1; m });
        println!("Q6 seq={seq} applied={applied} writes={} segments={} closed_form_full={} arm_a2_closed_form={} sigma={s} sigma_len={} sigma_kinds={kinds:?} sigma_txg={:?}",
            st.writes.len(), st.segments.len(), closed_form_state_count(&st.segments),
            1 + st.segments.iter().map(|sg| { let k = sg.iter().filter(|w| !st.is_cow(**w)).count() as u32; let n = sg.len() as u32 - k; if n > 0 { (1u64 << k) * 2 - 1 } else { (1u64 << k) - 1 } }).sum::<u64>(),
            seg.len(), st.publish_txg_of(seg[0]));
        println!("  CHAIN u={} l={} x={} y={} why={why}", describe_write(&st, u), describe_write(&st, l), describe_write(&st, x), describe_write(&st, y));
        let (n, bad, first) = arm_a2_all_states(&st, t);
        println!("  ARM_A2 seq={seq} states={n} red={bad} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
        for f in first { println!("    ARM_A2_RED {f}"); }
        if seg.len() <= 20 {
            let tl = enumerate_layer0_selecting_versions(&st.base, &st.writes, &st.segments, st.judged_root_index, &st.versions, &|i, _| i == s);
            println!("  SIGMA_FULL seq={seq} states={} expected={} record_claimed_state_missing_unit={} violations={} ignored={} root_without_record={} checker_violations={} elapsed_seconds={:.1}",
                tl.states, (1u64 << seg.len()), tl.record_claimed_state_missing_unit, tl.violations, tl.ignored_violations, tl.record_root_without_record,
                tl.checker_violated_states.values().sum::<u64>(), started.elapsed().as_secs_f64());
        } else {
            println!("  SIGMA_FULL seq={seq} skipped sigma_len={} closed_form_sigma={}", seg.len(), (1u64 << seg.len()) - 1);
        }
    }
}

/// 今天层 0 跑的固定脚本（与第一轮的 H1、H2）上有没有豁免链候选。
#[test]
fn q7_exemption_chains_on_fixed_scripts() {
    let streams: Vec<(&str, Layer0Stream)> = vec![
        ("B", fixed_script(Script::B).stream_for_layer0()),
        ("C", fixed_script(Script::C).stream_for_layer0()),
        ("D", fixed_script(Script::D).stream_for_layer0()),
        ("E", fixed_script(Script::E).stream_for_layer0()),
        ("E18", fixed_script(Script::E18).stream_for_layer0()),
        ("H1", abandon_by_transient_fault(1).0.stream_for_layer0()),
        ("H2", unmount_history(3).stream_for_layer0()),
    ];
    for (name, st) in &streams {
        let c = exemption_chain_candidates(st);
        println!("Q7 stream={name} writes={} segments={} chain_candidates={} confirmed={}", st.writes.len(), st.segments.len(), c.len(), confirm_chain(st).is_some());
    }
}

/// σ 段里只动 x、x'、y、y' 四个写（其余 COW 取 ∅ 或全集、原地全落）：16 个组合各红不红。
#[test]
fn q8_four_write_truth_table() {
    let seq = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "UOOUOMSU".into());
    let (sim, _) = history(&seq);
    let st = sim.stream_for_layer0();
    let (s, _u, _l, x, y, _) = confirm_chain(&st).expect("坐实");
    let seg = &st.segments[s];
    let cow: Vec<usize> = seg.iter().copied().filter(|w| st.is_cow(*w)).collect();
    let twin = |w: usize| cow.iter().copied().find(|c| *c != w && st.writes[*c].offset == st.writes[w].offset && st.writes[*c].device != st.writes[w].device).unwrap();
    let four = [x, twin(x), y, twin(y)];
    for others_all in [false, true] {
        for m in 0..16u32 {
            let mut p = st.prefix(s);
            for w in seg { if !st.is_cow(*w) { p[*w] = true; } }
            if others_all { for w in &cow { if !four.contains(w) { p[*w] = true; } } }
            for (b, w) in four.iter().enumerate() { if m & (1 << b) != 0 { p[*w] = true; } }
            let t = l0_eval(&st, p);
            println!("Q8 seq={seq} others_all={others_all} x_dev0={} x_dev1={} y_dev0={} y_dev1={} red={} {}", m & 1, (m >> 1) & 1, (m >> 2) & 1, (m >> 3) & 1, tally_red(&t), tally_why(&t));
        }
    }
}

// ───────────── 我提的改法（只在我的模型上量过、被攻过零轮）：记录核对器第二条的豁免按扇区判 ─────────────
// 今天：一份副本「不在」且「没有一次更晚、已持久、而且整份还在位的写盖过它」才算缺席。
// 改法：一份副本缺席 ⟺ 它区间里有一个扇区上的字节不是它的，而这个扇区上没有一次更晚、已持久、在这个扇区上字节还在的写。
// （C513 那一道「复用证得出违反回收谓词不开脱」这里没搬，只量不单调那一格。）
use singlefs_harness::crash::root_identity_written_by;
pub fn variant_claimed_state_missing_unit(st: &Layer0Stream, persisted: &[bool], eff: Option<(InstanceGeneration, CheckpointTxg)>) -> bool {
    let image = CrashImage { base: &st.base, writes: &st.writes, persisted: persisted.to_vec() };
    use singlefs_core::recovery::PoolReader;
    const SECTOR: u64 = 512;
    let sector_bytes_of = |i: usize, sector_start: u64| { let w = &st.writes[i]; let mut b = vec![0u8; SECTOR as usize]; w.contents.copy_range_into((sector_start - w.offset.0) as usize, &mut b); b };
    let copy_missing = |c: usize| {
        let w = &st.writes[c];
        let (start, end) = (w.offset.0, w.offset.0 + w.length_in_bytes());
        let mut sector = start;
        while sector < end {
            let disk = image.read(w.device, DeviceOffsetInBytes(sector), SECTOR as usize).unwrap_or_default();
            if disk != sector_bytes_of(c, sector) {
                let explained = (c + 1..st.writes.len()).any(|later| {
                    let l = &st.writes[later];
                    persisted[later] && l.device == w.device && l.offset.0 <= sector && sector + SECTOR <= l.offset.0 + l.length_in_bytes()
                        && disk == sector_bytes_of(later, sector)
                });
                if !explained { return true; }
            }
            sector += SECTOR;
        }
        false
    };
    let Some((_, eff_txg)) = eff else { return false };
    let mut units: Vec<usize> = Vec::new();
    for (i, w) in st.writes.iter().enumerate() {
        match w.kind {
            StepKind::UnitWrite => units.push(i),
            StepKind::RootRecordFua => {
                let (_, txg) = root_identity_written_by(w.bytes().unwrap());
                let this: Vec<usize> = std::mem::take(&mut units);
                if eff_txg.0 >= txg.0 {
                    let mut by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
                    for u in this { by_offset.entry(st.writes[u].offset.0).or_default().push(u); }
                    if by_offset.values().any(|copies| copies.iter().all(|c| copy_missing(*c))) { return true; }
                }
            }
            _ => {}
        }
    }
    false
}

/// σ 段全部 2^|σ| 个状态上，今天的记录核对器第二条与按扇区判的改法各红几个；另在甲二的全部状态上对照两者（改法不许在那里多报少报）。
#[test]
fn q9_sector_wise_exemption_on_sigma() {
    let t = threads();
    let seq = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "UOOUOMSU".into());
    let (sim, _) = history(&seq);
    let st = sim.stream_for_layer0();
    let (s, ..) = confirm_chain(&st).expect("坐实");
    let seg = st.segments[s].clone();
    let n = seg.len();
    let prefix = st.prefix(s);
    let in_place: Vec<usize> = seg.iter().copied().filter(|w| !st.is_cow(*w)).collect();
    // 恢复实际走的根只随原地写的子集变（这一段里没有 journal 记录与根槽写）：每个原地子集恢复一次。
    let mut eff_by_imask = BTreeMap::new();
    for imask in 0..(1u64 << in_place.len()) {
        let mut p = prefix.clone();
        for (b, w) in in_place.iter().enumerate() { if imask & (1 << b) != 0 { p[*w] = true; } }
        eff_by_imask.insert(imask, recover(&CrashImage { base: &st.base, writes: &st.writes, persisted: p }, JournalPolicy::Consult).effective_root);
    }
    let next = AtomicUsize::new(0);
    let out = Mutex::new((0u64, 0u64, 0u64, 0u64));
    let started = std::time::Instant::now();
    std::thread::scope(|scope| { for _ in 0..t { scope.spawn(|| loop {
        let m = next.fetch_add(1, Ordering::Relaxed) as u64;
        if m >= (1u64 << n) { break; }
        let mut p = prefix.clone();
        let mut imask = 0u64; let mut ib = 0;
        for (b, w) in seg.iter().enumerate() { if m & (1 << b) != 0 { p[*w] = true; } if !st.is_cow(*w) { if m & (1 << b) != 0 { imask |= 1 << ib; } ib += 1; } }
        let eff = eff_by_imask[&imask];
        let today = check_records(&CrashImage { base: &st.base, writes: &st.writes, persisted: p.clone() }, eff).claimed_state_missing_unit;
        let variant = variant_claimed_state_missing_unit(&st, &p, eff);
        let mut o = out.lock().unwrap();
        o.0 += 1; if today { o.1 += 1; } if variant { o.2 += 1; } if today != variant { o.3 += 1; }
    }); } });
    let o = out.into_inner().unwrap();
    println!("Q9 seq={seq} sigma={s} sigma_states={} today_red={} sector_wise_red={} differ={} elapsed_seconds={:.1}", o.0, o.1, o.2, o.3, started.elapsed().as_secs_f64());
}

/// 回收窗口置 0 的历史上：甲、甲二、全量抽样三臂（第一轮的 three_arms），加不单调搜索。
#[test]
fn q10_forced_zero_reuse_window_arms() {
    let random: usize = std::env::var("PROBE_RANDOM").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let seqs: Vec<String> = std::env::var("PROBE_SEQS").unwrap_or_else(|_| "ZOOOO".into()).split(',').map(str::to_string).collect();
    for seq in &seqs {
        let started = std::time::Instant::now();
        let (sim, applied) = history(seq);
        let st = sim.stream_for_layer0();
        let (checked, found, skipped) = search_non_monotone(&st, 14, false);
        let c = exemption_chain_candidates(&st);
        println!("Q10 seq={seq} applied={applied} writes={} segments={} overlap_chain_candidates={} non_monotone={} cheap_checked={checked} skipped_groups={skipped}", st.writes.len(), st.segments.len(), c.len(), found.len());
        for nm in found.iter().take(2) { println!("  NM seg={} imask={:#b} mask={:#b} why={}", nm.seg, nm.imask, nm.mask, nm.why); }
        three_arms(seq, &st, random);
        println!("  Q10_DONE seq={seq} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
    }
}
