//! m3-formal-r1 攻方原型（副本，不入 crates）：不装任何工具，在冻结副本的 `replay_journal` 上做有界穷举。
//! 输入域 = 一段合法历史（所选根那次发布 P0 + 之后 m 次发布，每次 k 条记录，事务提交标记任取）的任意记录子集
//! （崩溃 + 损坏都算：已定项 14 注 1 本就要处理「那条读不出」），外加按模式开的几维：点名单元在不在、另一实例的孤记录、
//! 同 jsn 不同内容的残留、小环回绕、txg 顶到 u64 最大值。每个状态真跑一次 `replay_journal`（真 CRC、真读者字节、反向链真判）。
//! 四条判据（都照文字写，不看实现）：
//!   R  = M2 字面（结果层）：施加后的根等于所选根时间线上某个 R_j（j ≥ c）。
//!   P  = M2 前缀层：R，且 (c, j] 里每次发布的每条记录都在、点名单元两份都在（「不存在被前缀切开的发布」）。
//!   M  = P，且 (c, j] 的记录总数 ≤ 在飞上限（I-8.3 的「整段落在在飞记录数上限之内」）。
//!   X  = 全函数判据：j 恰为「从 c+1 起连续完整、累计不超过在飞上限」的最大那一次（D23 已定项 14 五条 + 已定项 22 断号即止）。
//!   F  = 不动点（M3 幂等在只读那条路上唯一写得出的形态）：把施加后的根当所选根、同一张记录表再重放一次，根不再动。
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish, NamedUnit,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::recovery::{replay_journal, PoolReader};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::unit::UNIT_CLASS_INDEX_NODE;

const FSID: u64 = 0x5346_5346_0000_0001;
const INSTANCE: u32 = 7;
const WM_BASE: u64 = 1000;
const WM_STRAY: u64 = 2000;
const WM_RESIDUE: u64 = 3000;
const UNIT_BYTES: usize = 16384;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Base,
    Units,
    UnitsTwoCopies,
    Stray,
    Residue,
    Wrap,
    Edge,
}

impl Mode {
    fn parse(s: &str) -> Mode {
        match s {
            "base" => Mode::Base,
            "units" => Mode::Units,
            "units2" => Mode::UnitsTwoCopies,
            "stray" => Mode::Stray,
            "residue" => Mode::Residue,
            "wrap" => Mode::Wrap,
            "edge" => Mode::Edge,
            other => panic!("未知模式 {other}"),
        }
    }
    fn ring_slots(self) -> u64 {
        match self {
            Mode::Wrap => 6,
            _ => 3 * 64,
        }
    }
    fn unit_copy_bits(self) -> usize {
        match self {
            Mode::Units => 1,
            Mode::UnitsTwoCopies => 2,
            _ => 0,
        }
    }
}

struct Rec {
    rec: JournalRecord,
    bytes: Vec<u8>,
    publish: usize,
    unit_bytes: Option<Vec<u8>>,
}

fn txg_of(t0: u64, j: usize) -> u64 {
    t0 + j as u64
}

fn root_of(t0: u64, j: usize) -> RootRecord {
    RootRecord {
        filesystem_identifier: [0x11; 16],
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: InstanceGeneration(INSTANCE),
        checkpoint_txg: CheckpointTxg(txg_of(t0, j)),
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: WM_BASE + j as u64,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
        allocation_record_tree_root: NodePointer::empty_root(),
    }
}

fn unit_slot(counter: u64, copy: u64) -> u64 {
    200_000 + counter * 2 + copy
}

#[allow(clippy::too_many_arguments)]
fn make_record(
    instance: u32,
    counter: u64,
    txg: u64,
    transaction: u64,
    is_commit: bool,
    ordinal: u32,
    last: bool,
    back_chain: u32,
    watermark: u64,
    unit: Option<&[u8]>,
) -> JournalRecord {
    let named = match unit {
        None => Vec::new(),
        Some(content) => {
            let checksum = crc32_castagnoli(content);
            vec![NamedUnit {
                locations: [0, 1].map(|copy| LocationEntry {
                    device: DeviceIdentity(0),
                    slot: SlotNumber(unit_slot(counter, copy)),
                    unit_checksum: checksum,
                }),
                unit_class: UNIT_CLASS_INDEX_NODE,
                birth_tree: TreeIdentifier(0),
                birth_txg: CheckpointTxg(txg),
                key_tail: [0; 10],
            }]
        }
    };
    JournalRecord {
        instance: InstanceGeneration(instance),
        counter,
        checkpoint_txg: CheckpointTxg(txg),
        transaction,
        is_commit,
        ordinal_within_publish: JournalRecordOrdinalWithinPublish(ordinal),
        place_in_publish: if last {
            JournalRecordPlaceInPublish::LastRecordOfThePublish
        } else {
            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
        },
        back_chain,
        filesystem_identifier: FSID,
        new_tree_table: NodePointer::empty_root(),
        new_mapping_root: NodePointer::empty_root(),
        new_tree_identifier_watermark: watermark,
        new_rollback_floor: CheckpointTxg(0),
        named,
    }
}

fn unit_content(counter: u64) -> Vec<u8> {
    let mut content = vec![(counter % 251) as u8; UNIT_BYTES];
    content[..8].copy_from_slice(&counter.to_le_bytes());
    content
}

fn finish(rec: JournalRecord, publish: usize, unit_bytes: Option<Vec<u8>>) -> Rec {
    let bytes = rec.to_bytes();
    assert_eq!(JournalRecord::parse(&bytes, FSID).as_ref(), Some(&rec), "往返");
    Rec { rec, bytes, publish, unit_bytes }
}

/// 一段合法历史：P0（所选根 R_0 那次发布）..Pm，计数器从 1 起连续，反向链照实算。
fn build_history(ks: &[usize], commit_mask: u32, with_units: bool, t0: u64) -> Vec<Rec> {
    let mut out: Vec<Rec> = Vec::new();
    let (mut counter, mut transaction, mut bit) = (1u64, 1u64, 0u32);
    for (j, &k) in ks.iter().enumerate() {
        for index in 0..k {
            let last = index + 1 == k;
            let is_commit = last || (commit_mask >> bit) & 1 == 1;
            if !last {
                bit += 1;
            }
            let unit = with_units.then(|| unit_content(counter));
            let back_chain = out.last().map_or(0, |previous| back_chain_of(&previous.bytes));
            let rec = make_record(
                INSTANCE, counter, txg_of(t0, j), transaction, is_commit, (index + 1) as u32,
                last, back_chain, WM_BASE + j as u64, unit.as_deref(),
            );
            if is_commit {
                transaction += 1;
            }
            out.push(finish(rec, j, unit));
            counter += 1;
        }
    }
    out
}

/// 另一实例（INSTANCE + 1）的一条孤记录：接着计数器与 txg 写、本实例第一条反向链 0（C533 那一形：写行记录落了、根没落）。
fn stray_record(history: &[Rec], t0: u64, m: usize) -> Rec {
    let counter = history.len() as u64 + 1;
    let rec = make_record(
        INSTANCE + 1, counter, txg_of(t0, m + 1), 1_000_000, true, 1, true, 0, WM_STRAY, None,
    );
    finish(rec, usize::MAX, None)
}

/// 同 jsn、不同内容的残留：第 i 条换一个新根段（水位），反向链照原样（前一条相同），自证校验和重算。
fn residue_of(original: &Rec) -> Rec {
    let mut rec = original.rec.clone();
    rec.new_tree_identifier_watermark = WM_RESIDUE + original.publish as u64;
    finish(rec, original.publish, original.unit_bytes.clone())
}

struct Image<'a> {
    journal: BTreeMap<u64, &'a [u8]>,
    units: BTreeMap<u64, &'a [u8]>,
    reads: std::cell::Cell<u64>,
    bytes_read: std::cell::Cell<u64>,
}

impl PoolReader for Image<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        vec![DeviceIdentity(0)]
    }
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        (device.0 == 0).then_some(1 << 40)
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        if device.0 != 0 {
            return None;
        }
        self.reads.set(self.reads.get() + 1);
        let found = if length == 4096 {
            self.journal.get(&offset.0)
        } else if length == UNIT_BYTES {
            self.units.get(&offset.0)
        } else {
            None
        };
        found.map(|bytes| {
            self.bytes_read.set(self.bytes_read.get() + bytes.len() as u64);
            bytes.to_vec()
        })
    }
    fn journal_record_offsets_hint(
        &self,
        _device: DeviceIdentity,
        _ring_start: DeviceOffsetInBytes,
        _ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        None
    }
}

#[derive(Default, Clone)]
struct Tally {
    states: u64,
    viol: [u64; 5],
    first: [Option<String>; 5],
    errors: u64,
    panics: u64,
    applied_multi_record_publish: u64,
    applied_any: u64,
    reads: u64,
    bytes_read: u64,
}

impl Tally {
    fn add(&mut self, other: &Tally) {
        self.states += other.states;
        for i in 0..5 {
            self.viol[i] += other.viol[i];
            if self.first[i].is_none() {
                self.first[i].clone_from(&other.first[i]);
            }
        }
        self.errors += other.errors;
        self.panics += other.panics;
        self.applied_multi_record_publish += other.applied_multi_record_publish;
        self.applied_any += other.applied_any;
        self.reads += other.reads;
        self.bytes_read += other.bytes_read;
    }
}

fn compositions(n: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    for mask in 0..(1u32 << (n - 1)) {
        let (mut parts, mut current) = (Vec::new(), 1);
        for b in 0..n - 1 {
            if (mask >> b) & 1 == 1 {
                parts.push(current);
                current = 1;
            } else {
                current += 1;
            }
        }
        parts.push(current);
        out.push(parts);
    }
    out
}

/// 施加后的根是哪一个 R_j（整条根逐字段相等）；都不是 ⇒ None（外来的根：别的实例、残留的新根段）。
fn classify(rebuilt: &RootRecord, t0: u64, m: usize) -> Option<usize> {
    (0..=m).find(|&j| *rebuilt == root_of(t0, j))
}

struct Built<'a> {
    image: Image<'a>,
    table: BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    present: Vec<bool>,
}

#[allow(clippy::too_many_arguments)]
fn build_state<'a>(
    history: &'a [Rec], residues: &'a [Rec], stray: Option<&'a Rec>, s: u64, u: u64, e: usize,
    copy_bits: usize, ring_bytes: u64,
) -> Built<'a> {
    let n = history.len();
    let mut journal: BTreeMap<u64, &'a Rec> = BTreeMap::new();
    for i in 0..n {
        if (s >> i) & 1 == 1 {
            let rec = if !residues.is_empty() && e == i + 1 { &residues[i] } else { &history[i] };
            journal.insert(record_offset(rec.rec.counter, ring_bytes).0, rec);
        }
    }
    if let (Some(stray), 1) = (stray, e) {
        journal.insert(record_offset(stray.rec.counter, ring_bytes).0, stray);
    }
    let mut units: BTreeMap<u64, &'a [u8]> = BTreeMap::new();
    let mut unit_ok = vec![true; n];
    for (i, rec) in history.iter().enumerate() {
        if let Some(content) = &rec.unit_bytes {
            for copy in 0..2u64 {
                let bit = if copy_bits == 1 { i } else { 2 * i + copy as usize };
                if (u >> bit) & 1 == 1 {
                    units.insert(SlotNumber(unit_slot(rec.rec.counter, copy)).to_device_offset().0, content);
                } else {
                    unit_ok[i] = false;
                }
            }
        }
    }
    let present = (0..n)
        .map(|i| {
            unit_ok[i]
                && journal
                    .get(&record_offset(history[i].rec.counter, ring_bytes).0)
                    .is_some_and(|rec| rec.rec.counter == history[i].rec.counter)
        })
        .collect();
    let table = journal.values().map(|rec| ((rec.rec.instance, rec.rec.counter), rec.rec.clone())).collect();
    let image = Image {
        journal: journal.iter().map(|(offset, rec)| (*offset, rec.bytes.as_slice())).collect(),
        units,
        reads: std::cell::Cell::new(0),
        bytes_read: std::cell::Cell::new(0),
    };
    Built { image, table, present }
}

enum Outcome {
    Reached(Option<usize>),
    Error,
    Panic,
}

fn run_replay(built: &Built<'_>, root: &RootRecord, ring_bytes: u64, t0: u64, m: usize) -> Outcome {
    match catch_unwind(AssertUnwindSafe(|| {
        replay_journal(&built.image, root, ring_bytes, &built.table, true)
    })) {
        Ok(Ok((_report, rebuilt))) => Outcome::Reached(classify(&rebuilt, t0, m)),
        Ok(Err(_)) => Outcome::Error,
        Err(_) => Outcome::Panic,
    }
}

fn run_job(mode: Mode, ks: &[usize], commit_mask: u32) -> Tally {
    let mut tally = Tally::default();
    let m = ks.len() - 1;
    let t0 = if mode == Mode::Edge { u64::MAX - m as u64 } else { 5 };
    let copy_bits = mode.unit_copy_bits();
    let history = build_history(ks, commit_mask, copy_bits > 0, t0);
    let n = history.len();
    let ring_slots = mode.ring_slots();
    let ring_bytes = ring_slots * 4096;
    let limit = (ring_slots / 3) as usize;
    let stray = (mode == Mode::Stray).then(|| stray_record(&history, t0, m));
    let residues: Vec<Rec> = if mode == Mode::Residue { history.iter().map(residue_of).collect() } else { Vec::new() };
    let extra_choices = match mode {
        Mode::Stray => 2,
        Mode::Residue => n + 1,
        _ => 1,
    };
    let first_of: Vec<usize> = ks.iter().scan(0, |acc, &k| { let start = *acc; *acc += k; Some(start) }).collect();
    for c in 0..=m {
        let root = root_of(t0, c);
        for s in 0..(1u64 << n) {
            for u in 0..(1u64 << (n * copy_bits)) {
                for e in 0..extra_choices {
                    let built = build_state(&history, &residues, stray.as_ref(), s, u, e, copy_bits, ring_bytes);
                    let whole = |q: usize| (first_of[q]..first_of[q] + ks[q]).all(|i| built.present[i]);
                    let (mut reach, mut total) = (c, 0usize);
                    for q in c + 1..=m {
                        if whole(q) && total + ks[q] <= limit { reach = q; total += ks[q]; } else { break; }
                    }
                    let outcome = run_replay(&built, &root, ring_bytes, t0, m);
                    tally.states += 1;
                    tally.reads += built.image.reads.get();
                    tally.bytes_read += built.image.bytes_read.get();
                    let edge_error_allowed = mode == Mode::Edge && c == m;
                    let (ok, desc_got) = match outcome {
                        Outcome::Reached(Some(j)) if j >= c => {
                            if j > c { tally.applied_any += 1; }
                            if (c + 1..=j).any(|q| ks[q] >= 2) { tally.applied_multi_record_publish += 1; }
                            let p = (c + 1..=j).all(whole);
                            let within: usize = (c + 1..=j).map(|q| ks[q]).sum();
                            let again = run_replay(&built, &root_of(t0, j), ring_bytes, t0, m);
                            let fixed = match again {
                                Outcome::Reached(Some(j2)) => j2 == j,
                                Outcome::Error => mode == Mode::Edge && j == m,
                                _ => false,
                            };
                            ([true, p, p && within <= limit, j == reach, fixed], format!("j={j}"))
                        }
                        Outcome::Reached(other) => ([false, false, false, false, true], format!("外来或回退的根 {other:?}")),
                        Outcome::Error => { tally.errors += 1; ([edge_error_allowed, edge_error_allowed, edge_error_allowed, edge_error_allowed, true], "Err".to_string()) }
                        Outcome::Panic => { tally.panics += 1; ([false, false, false, false, true], "panic".to_string()) }
                    };
                    for i in 0..5 {
                        if !ok[i] {
                            tally.viol[i] += 1;
                            if tally.first[i].is_none() {
                                tally.first[i] = Some(format!(
                                    "ks={ks:?} commit={commit_mask:b} c={c} s={s:0w$b} u={u:b} e={e} got={desc_got} X期望j={reach}", w = n
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    tally
}

/// M3 单调那一半的两种读法：根槽也进崩溃状态（R_1..R_m 各一位，R_0 恒在），所选根 = 最高那一位。
/// (i) 「施加的事务集」= 重放这一步施加的发布 (c, j]；(ii) = 落到的版本 j 所含的发布 [1, j]。数 A ⊆ B 而不单调的对数。
fn mono_job(ks: &[usize], commit_mask: u32) -> (u64, u64, u64, Option<String>, Option<String>) {
    let m = ks.len() - 1;
    let t0 = 5;
    let history = build_history(ks, commit_mask, false, t0);
    let n = history.len();
    // PROBE_MONO_RING_SLOTS=6 ⇒ 在飞上限 2 条（上限咬人的那一档）；默认 192 槽、上限 64 条。
    let ring_slots: u64 = std::env::var("PROBE_MONO_RING_SLOTS").ok().and_then(|v| v.parse().ok()).unwrap_or(3 * 64);
    let ring_bytes = ring_slots * 4096;
    let bits = n + m;
    let mut reached = vec![(0usize, 0usize); 1 << bits];
    for state in 0..(1u64 << bits) {
        let s = state & ((1 << n) - 1);
        let roots = state >> n;
        let c = (1..=m).rev().find(|j| (roots >> (j - 1)) & 1 == 1).unwrap_or(0);
        let built = build_state(&history, &[], None, s, 0, 0, 0, ring_bytes);
        let j = match run_replay(&built, &root_of(t0, c), ring_bytes, t0, m) {
            Outcome::Reached(Some(j)) => j,
            _ => panic!("原实现在单调域上不该出错"),
        };
        reached[state as usize] = (c, j);
    }
    let (mut pairs, mut viol_i, mut viol_ii) = (0u64, 0u64, 0u64);
    let (mut first_i, mut first_ii) = (None, None);
    for b in 0..(1usize << bits) {
        let mut a = b;
        loop {
            pairs += 1;
            let ((ca, ja), (cb, jb)) = (reached[a], reached[b]);
            if ja > ca && !(cb <= ca && jb >= ja) {
                viol_i += 1;
                if first_i.is_none() { first_i = Some(format!("ks={ks:?} A={a:0w$b}(c={ca},j={ja}) ⊆ B={b:0w$b}(c={cb},j={jb})", w = bits)); }
            }
            if ja > jb {
                viol_ii += 1;
                if first_ii.is_none() { first_ii = Some(format!("ks={ks:?} A={a:0w$b}(c={ca},j={ja}) ⊆ B={b:0w$b}(c={cb},j={jb})", w = bits)); }
            }
            if a == 0 { break; }
            a = (a - 1) & b;
        }
    }
    (pairs, viol_i, viol_ii, first_i, first_ii)
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let args: Vec<String> = std::env::args().collect();
    let mode_name = args.get(1).map_or("base", String::as_str).to_string();
    let n_min: usize = args.get(2).map_or(1, |v| v.parse().expect("n_min"));
    let n_max: usize = args.get(3).map_or(6, |v| v.parse().expect("n_max"));
    let threads: usize = args.get(4).map_or(4, |v| v.parse().expect("threads"));
    for n in n_min..=n_max {
        let mut jobs: Vec<(Vec<usize>, u32)> = Vec::new();
        let single_only = std::env::var("PROBE_SINGLE_RECORD_PUBLISHES").is_ok_and(|v| v == "1");
        if mode_name == "adv" {
            let started = Instant::now();
            let next = AtomicUsize::new(0);
            let work: Vec<(u8, u64)> = (0..3u8).flat_map(|a| (0..64u64).map(move |f| (a, f))).collect();
            let total = Mutex::new((0u64, 0u64, 0u64, None::<String>, None::<String>));
            std::thread::scope(|scope| {
                for _ in 0..threads {
                    scope.spawn(|| loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(&(anchor, prefix)) = work.get(index) else { break };
                        let (st, vr, vp, fr, fp) = adv_job(n, anchor, prefix);
                        let mut t = total.lock().expect("锁");
                        t.0 += st; t.1 += vr; t.2 += vp;
                        if t.3.is_none() { t.3 = fr; }
                        if t.4.is_none() { t.4 = fp; }
                    });
                }
            });
            let t = total.into_inner().expect("锁");
            println!("mode=adv n={n} states={} viol_R={} viol_P={} secs={:.2}", t.0, t.1, t.2, started.elapsed().as_secs_f64());
            println!("  first_R: {}", t.3.unwrap_or_else(|| "-".into()));
            println!("  first_P: {}", t.4.unwrap_or_else(|| "-".into()));
            continue;
        }
        for ks in compositions(n) {
            if single_only && ks.iter().any(|&k| k != 1) {
                continue;
            }
            let free_bits = n - ks.len();
            for commit_mask in 0..(1u32 << free_bits) {
                jobs.push((ks.clone(), commit_mask));
            }
        }
        let started = Instant::now();
        let next = AtomicUsize::new(0);
        if mode_name == "mono" {
            let total = Mutex::new((0u64, 0u64, 0u64, None::<String>, None::<String>));
            std::thread::scope(|scope| {
                for _ in 0..threads {
                    scope.spawn(|| loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some((ks, commit_mask)) = jobs.get(index) else { break };
                        let (p, vi, vii, fi, fii) = mono_job(ks, *commit_mask);
                        let mut t = total.lock().expect("锁");
                        t.0 += p; t.1 += vi; t.2 += vii;
                        if t.3.is_none() { t.3 = fi; }
                        if t.4.is_none() { t.4 = fii; }
                    });
                }
            });
            let t = total.into_inner().expect("锁");
            println!(
                "mode=mono n={n} shapes={} pairs={} viol_applied_by_replay={} viol_version_reached={} secs={:.2}",
                jobs.len(), t.0, t.1, t.2, started.elapsed().as_secs_f64()
            );
            println!("  first_i: {}", t.3.unwrap_or_else(|| "-".into()));
            println!("  first_ii: {}", t.4.unwrap_or_else(|| "-".into()));
            continue;
        }
        let mode = Mode::parse(&mode_name);
        let total = Mutex::new(Tally::default());
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some((ks, commit_mask)) = jobs.get(index) else { break };
                    let tally = run_job(mode, ks, *commit_mask);
                    total.lock().expect("锁").add(&tally);
                });
            }
        });
        let t = total.into_inner().expect("锁");
        println!(
            "mode={mode_name} n={n} shapes={} states={} viol_R={} viol_P={} viol_M={} viol_X={} viol_F={} errors={} panics={} applied_any={} applied_multi={} reads={} bytes_read={} secs={:.2}",
            jobs.len(), t.states, t.viol[0], t.viol[1], t.viol[2], t.viol[3], t.viol[4], t.errors, t.panics,
            t.applied_any, t.applied_multi_record_publish, t.reads, t.bytes_read, started.elapsed().as_secs_f64()
        );
        for (i, name) in ["R", "P", "M", "X", "F"].iter().enumerate() {
            if let Some(example) = &t.first[i] {
                println!("  first_{name}: {example}");
            }
        }
    }
}

/// 对抗域（CRC 当未解释函数时模型放进来的那一类：自证过、字段任取的记录）：所选根 (INSTANCE, 5)，
/// 根那次发布三种锚：无 / 一条带末条标志 / 两条都带；之后 n 条记录计数器连续，每条 txg∈{6,7}、序号∈{1,2}、末条∈{0,1}、
/// 提交∈{0,1}、事务号∈{1,2}、反向链对/错，共 64 种。新根段的水位 = 5000 + 计数器，施加后的根看得出施加到哪一条。
/// 判据两条：R_adv = 施加后的根是所选根或某条带末条标志的记录的新根段；P_adv = 声明式前缀判据（照 D23 已定项 14 五条、已定项 4 读者规则、
/// I-8.6 写，不照实现的循环）：被施加的那一段从链首起计数器连续，按 txg 分组，每组序号 1..k、只有第 k 条带末条标志且带提交标记、
/// 组内换事务号之前上一条带提交标记、反向链在前一条在盘上时对得上、组末之后同 txg 没有别的读得出的记录，前缀止于一组的末条。
fn adv_job(n: usize, anchor_kind: u8, field_prefix: u64) -> (u64, u64, u64, Option<String>, Option<String>) {
    let ring_bytes = 3 * 64 * 4096;
    let root = root_of(5, 0);
    let per = 64u64;
    let total = per.pow(n as u32 - 1);
    let (mut states, mut viol_r, mut viol_p) = (0u64, 0u64, 0u64);
    let (mut first_r, mut first_p) = (None, None);
    for rest in 0..total {
        let code = field_prefix * total + rest;
        let mut recs: Vec<Rec> = Vec::new();
        let anchors: Vec<(u64, bool)> = match anchor_kind { 0 => vec![], 1 => vec![(1, true)], _ => vec![(1, true), (2, true)] };
        for &(counter, last) in &anchors {
            let back_chain = recs.last().map_or(0, |p| back_chain_of(&p.bytes));
            let rec = make_record(INSTANCE, counter, 5, 1, true, counter as u32, last, back_chain, 5000 + counter, None);
            recs.push(finish(rec, 0, None));
        }
        let base = anchors.len() as u64 + 1;
        let mut fields = Vec::new();
        for i in 0..n {
            let f = (code >> (6 * i)) & 63;
            let (txg, ord, last, commit, tx, bc_ok) = (6 + (f & 1), 1 + ((f >> 1) & 1) as u32, (f >> 2) & 1 == 1, (f >> 3) & 1 == 1, 1 + ((f >> 4) & 1), (f >> 5) & 1 == 0);
            fields.push((txg, ord, last, commit, tx, bc_ok));
            let counter = base + i as u64;
            let good = recs.last().map_or(0, |p| back_chain_of(&p.bytes));
            let back_chain = if bc_ok { good } else { good ^ 1 };
            let rec = make_record(INSTANCE, counter, txg, tx, commit, ord, last, back_chain, 5000 + counter, None);
            recs.push(finish(rec, 1, None));
        }
        let s = (1u64 << recs.len()) - 1;
        let built = build_state(&recs, &[], None, s, 0, 0, 0, ring_bytes);
        states += 1;
        let outcome = catch_unwind(AssertUnwindSafe(|| replay_journal(&built.image, &root, ring_bytes, &built.table, true)));
        let (ok_r, ok_p, got) = match outcome {
            Ok(Ok((_, rebuilt))) => {
                if rebuilt == root {
                    (true, true, "无施加".to_string())
                } else {
                    let upto = recs.iter().position(|r| 5000 + r.rec.counter == rebuilt.tree_identifier_watermark);
                    match upto {
                        None => (false, false, "外来根".to_string()),
                        Some(k) => {
                            let r_ok = recs[k].rec.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
                                && rebuilt.checkpoint_txg == recs[k].rec.checkpoint_txg;
                            (r_ok, r_ok && declarative_prefix_ok(&recs, anchors.len(), k), format!("施加到计数器 {}", recs[k].rec.counter))
                        }
                    }
                }
            }
            Ok(Err(_)) => (anchor_kind == 2, anchor_kind == 2, "Err".to_string()),
            Err(_) => (false, false, "panic".to_string()),
        };
        let desc = || format!("n={n} 锚={anchor_kind} 字段(txg,序号,末条,提交,事务,链对)={fields:?} got={got}");
        if !ok_r { viol_r += 1; if first_r.is_none() { first_r = Some(desc()); } }
        if !ok_p { viol_p += 1; if first_p.is_none() { first_p = Some(desc()); } }
    }
    (states, viol_r, viol_p, first_r, first_p)
}

/// 声明式前缀判据：recs[anchor_count..=k] 里从链首到 k 这一段（链首：有带末条标志的锚 ⇒ 锚后第一条；无 ⇒ 第一条须 txg 6、序号 1）。
fn declarative_prefix_ok(recs: &[Rec], anchor_count: usize, k: usize) -> bool {
    if k < anchor_count { return false; }
    let prefix: Vec<&JournalRecord> = recs[anchor_count..=k].iter().map(|r| &r.rec).collect();
    let anchored = recs[..anchor_count].iter().any(|r| r.rec.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish);
    if !anchored && (prefix[0].checkpoint_txg.0 != 6 || prefix[0].ordinal_within_publish.0 != 1) { return false; }
    for (i, r) in recs.iter().enumerate().skip(anchor_count).take(k + 1 - anchor_count) {
        if i > 0 && r.rec.back_chain != back_chain_of(&recs[i - 1].bytes) { return false; }
    }
    let mut groups: Vec<Vec<&JournalRecord>> = Vec::new();
    for r in &prefix {
        match groups.last_mut() {
            Some(g) if g.last().expect("非空").place_in_publish != JournalRecordPlaceInPublish::LastRecordOfThePublish => g.push(r),
            _ => groups.push(vec![r]),
        }
    }
    for g in &groups {
        let last = g.last().expect("非空");
        if last.place_in_publish != JournalRecordPlaceInPublish::LastRecordOfThePublish || !last.is_commit { return false; }
        for (idx, r) in g.iter().enumerate() {
            if r.ordinal_within_publish.0 as usize != idx + 1 || r.checkpoint_txg != g[0].checkpoint_txg { return false; }
            if idx > 0 && !g[idx - 1].is_commit && g[idx - 1].transaction != r.transaction { return false; }
        }
        let follows = if std::env::var("PROBE_ADV_FOLLOW").is_ok_and(|v| v == "monotone") {
            // 读法乙（照「同一实例里计数器与 txg 一起往上走」收窄）：从下一个计数器起按升序看，txg 越过这一条就停。
            recs.iter().filter(|o| o.rec.counter > last.counter)
                .take_while(|o| o.rec.checkpoint_txg <= last.checkpoint_txg)
                .any(|o| o.rec.checkpoint_txg == last.checkpoint_txg)
        } else {
            // 读法甲（D23 已定项 4 字面）：带末条标志的那条之后，同一 (实例, txg) 里还有记录。
            recs.iter().any(|o| o.rec.counter > last.counter && o.rec.checkpoint_txg == last.checkpoint_txg)
        };
        if follows { return false; }
    }
    true
}
