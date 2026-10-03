#![allow(clippy::all, dead_code, unused_imports)]
// 草稿：改前（fcc4d340 的 replay_journal）与改后对同一批输入逐个比结局（返回值、读盘日志、是否 panic）。
use super::*;
use crate::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber, TreeIdentifier};
use crate::journal::{back_chain_of, record_offset, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, NamedUnit};
use crate::pointer::{LocationEntry, NodePointer};
use crate::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 { (self.next() >> 11) % n }
    fn chance(&mut self, num: u64, den: u64) -> bool { self.below(den) < num }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind { Good, Bad, Absent }
#[derive(Clone, Copy, Debug)]
enum Mirrors { Both, First, Second, None, AlteredFirstGoodSecond, AlteredBoth }
#[derive(Clone, Debug)]
struct NamedSpec { class: u8, kinds: [Kind; 2] }
#[derive(Clone, Debug)]
struct Spec {
    instance: u32, counter: u64, txg: u64, ordinal: u32, ends: bool, commit: bool, tx: u64,
    chain_ok: bool, mirrors: Mirrors, named: Vec<NamedSpec>,
}

#[derive(Default, Clone)]
struct DiskImage {
    blocks: HashMap<(u32, u64), Vec<u8>>,
    units: HashMap<(u32, u64), Kind>,
}
struct Disk { image: DiskImage, log: RefCell<Vec<(u32, u64, usize)>> }
impl PoolReader for Disk {
    fn device_identities(&self) -> Vec<DeviceIdentity> { vec![DeviceIdentity(1), DeviceIdentity(2)] }
    fn device_size_in_bytes(&self, _d: DeviceIdentity) -> Option<u64> { Some(1 << 40) }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: usize) -> Option<Vec<u8>> {
        self.log.borrow_mut().push((device.0, offset.0, length));
        let key = (device.0, offset.0);
        if let Some(b) = self.image.blocks.get(&key) {
            return Some(b.clone());
        }
        match self.image.units.get(&key) {
            Some(Kind::Good) => Some(vec![0u8; length]),
            Some(Kind::Bad) => Some(vec![1u8; length]),
            _ => None,
        }
    }
    fn journal_record_offsets_hint(&self, _d: DeviceIdentity, _s: DeviceOffsetInBytes, _r: u64) -> Option<Vec<DeviceOffsetInBytes>> { None }
}

struct Scenario {
    root: RootRecord,
    records: BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    image: DiskImage,
    ring_bytes: u64,
    verify: bool,
    specs: Vec<Spec>,
    root_txg: u64,
    root_instance: u32,
}

fn root_of(instance: u32, txg: u64) -> RootRecord {
    RootRecord {
        filesystem_identifier: [9; 16],
        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        tree_table: NodePointer::empty_root(),
        tree_identifier_watermark: 5,
        rollback_floor: CheckpointTxg(0),
        instance_table: NodePointer::empty_root(),
        mapping_root: NodePointer::empty_root(),
        allocation_record_tree_root: NodePointer::empty_root(),
    }
}

fn build(root_instance: u32, root_txg: u64, specs: &[Spec], ring_slots: u64, verify: bool, no_disk: bool) -> Scenario {
    let ring_bytes = ring_slots * 4096;
    let mut sorted: Vec<&Spec> = specs.iter().collect();
    sorted.sort_by_key(|s| (s.instance, s.counter));
    sorted.dedup_by_key(|s| (s.instance, s.counter));
    let mut records: BTreeMap<(InstanceGeneration, u64), JournalRecord> = BTreeMap::new();
    let mut image = DiskImage::default();
    let mut slot_seed = 1000u64;
    let mut out_specs = Vec::new();
    for s in sorted {
        out_specs.push(s.clone());
        let previous_bytes = if s.counter >= 1 && s.counter - 1 < (1u64 << 48) && !no_disk {
            records.get(&(InstanceGeneration(s.instance), s.counter.wrapping_sub(1))).map(|p| p.to_bytes())
        } else { None };
        let mut back_chain = previous_bytes.as_ref().map_or(0x1234, |b| back_chain_of(b));
        if !s.chain_ok { back_chain ^= 0x55; }
        let mut named = Vec::new();
        for n in &s.named {
            let unit_bytes = match n.class {
                UNIT_CLASS_DATA | UNIT_CLASS_PACKED => 32768usize,
                UNIT_CLASS_INDEX_NODE => 16384,
                _ => 16384,
            };
            let good_crc = crc32_castagnoli(&vec![0u8; unit_bytes]);
            let mut locs = [LocationEntry { device: DeviceIdentity(1), slot: SlotNumber(0), unit_checksum: good_crc }; 2];
            for (i, kind) in n.kinds.iter().enumerate() {
                slot_seed += 1;
                let dev = (i as u32) + 1;
                locs[i] = LocationEntry { device: DeviceIdentity(dev), slot: SlotNumber(slot_seed), unit_checksum: good_crc };
                if *kind != Kind::Absent {
                    image.units.insert((dev, locs[i].slot.to_device_offset().0), *kind);
                }
            }
            named.push(NamedUnit { locations: locs, unit_class: n.class, birth_tree: TreeIdentifier(1), birth_txg: CheckpointTxg(1), key_tail: [0; 10] });
        }
        let record = JournalRecord {
            instance: InstanceGeneration(s.instance), counter: s.counter, checkpoint_txg: CheckpointTxg(s.txg),
            transaction: s.tx, is_commit: s.commit,
            ordinal_within_publish: JournalRecordOrdinalWithinPublish(s.ordinal),
            place_in_publish: if s.ends { JournalRecordPlaceInPublish::LastRecordOfThePublish } else { JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow },
            back_chain, filesystem_identifier: 7,
            new_tree_table: NodePointer::empty_root(), new_mapping_root: NodePointer::empty_root(),
            new_tree_identifier_watermark: 100u64.wrapping_add(s.counter).wrapping_add(1000 * u64::from(s.instance)),
            new_rollback_floor: CheckpointTxg(s.counter), named,
        };
        if !no_disk && s.counter >= 1 && s.counter < (1u64 << 48) {
            let off = record_offset(s.counter, ring_bytes).0;
            let good = record.to_bytes();
            let altered = { let mut r = record.clone(); r.transaction ^= 0xFF; r.to_bytes() };
            let (a, b): (Option<Vec<u8>>, Option<Vec<u8>>) = match s.mirrors {
                Mirrors::Both => (Some(good.clone()), Some(good)),
                Mirrors::First => (Some(good), None),
                Mirrors::Second => (None, Some(good)),
                Mirrors::None => (None, None),
                Mirrors::AlteredFirstGoodSecond => (Some(altered), Some(good)),
                Mirrors::AlteredBoth => (Some(altered.clone()), Some(altered)),
            };
            if let Some(a) = a { image.blocks.insert((1, off), a); }
            if let Some(b) = b { image.blocks.insert((2, off), b); }
        }
        records.insert((record.instance, record.counter), record);
    }
    Scenario { root: root_of(root_instance, root_txg), records, image, ring_bytes, verify, specs: out_specs, root_txg, root_instance }
}

type Outcome = Result<Result<(JournalScanReport, RootRecord), RecoveryFailure>, ()>;
type OldFn = fn(&dyn PoolReader, &RootRecord, u64, &BTreeMap<(InstanceGeneration, u64), JournalRecord>, bool) -> Result<(JournalScanReport, RootRecord), RecoveryFailure>;

fn run(f: OldFn, sc: &Scenario) -> (Outcome, Vec<(u32, u64, usize)>) {
    let disk = Disk { image: sc.image.clone(), log: RefCell::new(Vec::new()) };
    IN_RUN.with(|x| x.set(true));
    let r = catch_unwind(AssertUnwindSafe(|| f(&disk, &sc.root, sc.ring_bytes, &sc.records, sc.verify))).map_err(|_| ());
    IN_RUN.with(|x| x.set(false));
    let log = disk.log.borrow().clone();
    (r, log)
}

#[derive(Default, Debug)]
struct Tally { total: u64, diverged: u64, panics_old: u64, panics_new: u64, errs: u64, applied_some: u64, verif_failed: u64, anchor_err: u64, shown: u64 }

fn compare(old: OldFn, sc: &Scenario, t: &mut Tally, label: &str) {
    let (o, olog) = run(old, sc);
    let (n, nlog) = run(replay_journal, sc);
    t.total += 1;
    if o.is_err() { t.panics_old += 1; }
    if n.is_err() { t.panics_new += 1; }
    match &n { Ok(Err(_)) => t.errs += 1, Ok(Ok((rep, _))) => { if rep.prefix_applied > 0 { t.applied_some += 1; } if rep.verification_failed > 0 { t.verif_failed += 1; } } _ => {} }
    if o != n || olog != nlog {
        t.diverged += 1;
        if t.shown < 3 {
            t.shown += 1;
            eprintln!("DIVERGE[{label}] root=({},{}) ring_slots={} verify={}\n specs={:?}\n old={:?}\n new={:?}\n oldlog={:?}\n newlog={:?}", sc.root_instance, sc.root_txg, sc.ring_bytes/4096, sc.verify, sc.specs, o, n, olog, nlog);
        }
    }
}

fn merge(a: &mut Tally, b: Tally) {
    a.total += b.total; a.diverged += b.diverged; a.panics_old += b.panics_old; a.panics_new += b.panics_new;
    a.errs += b.errs; a.applied_some += b.applied_some; a.verif_failed += b.verif_failed; a.shown += b.shown;
}

thread_local! { static IN_RUN: std::cell::Cell<bool> = std::cell::Cell::new(false); }
fn silence() { std::panic::set_hook(Box::new(|info| { if !IN_RUN.with(|f| f.get()) { eprintln!("HARNESS PANIC {info}"); } })); }
fn threads() -> u64 { std::env::var("DIFF_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8) }

fn sharded(old: OldFn, label: &'static str, total: u64, make: impl Fn(u64) -> Scenario + Sync) -> Tally {
    silence();
    let n_threads = threads();
    let mut all = Tally::default();
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for shard in 0..n_threads {
            let make = &make;
            handles.push(scope.spawn(move || {
                let mut t = Tally::default();
                let mut i = shard;
                while i < total {
                    let sc = make(i);
                    compare(old, &sc, &mut t, label);
                    i += n_threads;
                }
                t
            }));
        }
        for h in handles { merge(&mut all, h.join().unwrap()); }
    });
    eprintln!("RESULT[{label}] {all:?}");
    all
}

// ---------- 随机一族 ----------
fn random_scenario(seed: u64, i: u64, with_extremes: bool) -> Scenario {
    let mut r = Rng(seed ^ (i.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1));
    for _ in 0..3 { r.next(); }
    let root_instance = r.below(3) as u32;
    let root_txg = match r.below(20) { 0 => u64::MAX, 1 if with_extremes => u64::MAX - 1, 2 => 0, k => 1 + (k % 3) };
    let ring_slots = 1 + r.below(45);
    let verify = r.chance(1, 2);
    let n = r.below(10) as usize;
    let mut specs = Vec::new();
    let mut counter = [1u64, 2, 3, 5, 7, (1 << 48) - 3, 1 << 47][r.below(7) as usize];
    let mut txg = root_txg.saturating_add(r.below(3));
    let mut ordinal = 1u32;
    let mut instance = if r.chance(8, 10) { root_instance } else { root_instance.wrapping_add(1 + r.below(2) as u32) };
    let mut tx = 10u64;
    // 所选根自己那次发布的 0–3 条
    let root_publish = r.below(4);
    for k in 0..root_publish {
        let e = k + 1 == root_publish && r.chance(9, 10); specs.push(random_spec(&mut r, root_instance, counter, root_txg, (k + 1) as u32, e, tx));
        counter += 1;
        tx += r.below(2);
    }
    for _ in 0..n {
        // 偶尔换实例、换 txg、跳号、乱序号
        if r.chance(1, 12) { instance = if r.chance(1, 2) { root_instance } else { instance.wrapping_add(1) }; }
        if r.chance(1, 10) { counter += 1 + r.below(3); }
        if r.chance(1, 3) { txg = txg.saturating_add(1); ordinal = 1; } else if r.chance(1, 20) { txg = txg.saturating_sub(1); }
        let mut ord = ordinal;
        if r.chance(1, 14) { ord = r.below(4) as u32; }
        let ends = r.chance(1, 3);
        specs.push(random_spec(&mut r, instance, counter, txg, ord, ends, tx));
        counter += 1;
        ordinal = if ends { 1 } else { ordinal + 1 };
        if ends { txg = txg.saturating_add(1); }
        tx += r.below(3);
        if r.chance(1, 25) { // 重复键（后者覆盖前者的行为由 build 去重）
            let c = counter.saturating_sub(1 + r.below(3));
            let e2 = r.chance(1, 2); specs.push(random_spec(&mut r, instance, c, txg, ordinal, e2, tx));
        }
    }
    build(root_instance, root_txg, &specs, ring_slots, verify, false)
}

fn random_spec(r: &mut Rng, instance: u32, counter: u64, txg: u64, ordinal: u32, ends: bool, tx: u64) -> Spec {
    let mirrors = match r.below(12) {
        0 => Mirrors::First, 1 => Mirrors::Second, 2 => Mirrors::None, 3 => Mirrors::AlteredFirstGoodSecond, 4 => Mirrors::AlteredBoth, _ => Mirrors::Both,
    };
    let mut named = Vec::new();
    for _ in 0..r.below(3) {
        let class = [UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED, 77][r.below(4) as usize];
        let k = |r: &mut Rng| match r.below(8) { 0 => Kind::Bad, 1 => Kind::Absent, _ => Kind::Good };
        named.push(NamedSpec { class, kinds: [k(r), k(r)] });
    }
    Spec { instance, counter, txg, ordinal, ends, commit: !r.chance(1, 8), tx, chain_ok: !r.chance(1, 10), mirrors, named }
}

fn total_of(name: &str, default: u64) -> u64 { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default) }

#[test]
fn random_family() {
    let seed = total_of("DIFF_SEED", 1);
    let t = sharded(super::zz_old::replay_journal_before, "random", total_of("DIFF_N", 300_000), move |i| random_scenario(seed, i, true));
    assert_eq!(t.diverged, 0);
}

// ---------- 穷举一族：结构 ----------
// 每条记录：gap{0,1} × txg{1,2,3} × ordinal{1,2} × ends{0,1} × commit{0,1} × tx{同,异} × instance{根,根+1}
fn structural_spec(code: u64, prev_counter: u64, prev_tx: u64, with_instance: bool) -> (Spec, u64, u64) {
    let mut c = code;
    let gap = c % 2; c /= 2;
    let txg = 1 + c % 3; c /= 3;
    let ordinal = 1 + (c % 2) as u32; c /= 2;
    let ends = c % 2 == 1; c /= 2;
    let commit = c % 2 == 1; c /= 2;
    let tx_diff = c % 2; c /= 2;
    let other_instance = with_instance && c % 2 == 1;
    let counter = prev_counter + 1 + gap;
    let tx = prev_tx + tx_diff;
    (Spec { instance: if other_instance { 2 } else { 1 }, counter, txg, ordinal, ends, commit, tx, chain_ok: true, mirrors: Mirrors::Both, named: vec![] }, counter, tx)
}

fn anchor_specs(variant: u64) -> Vec<Spec> {
    let mk = |counter, ends, ordinal| Spec { instance: 1, counter, txg: 1, ordinal, ends, commit: true, tx: 1, chain_ok: true, mirrors: Mirrors::Both, named: vec![] };
    match variant {
        0 => vec![],
        1 => vec![mk(1, true, 1)],
        2 => vec![mk(1, true, 1), mk(2, true, 2)], // 两条带末条标志：错误
        3 => vec![mk(1, false, 1)],                 // 锚点读不出
        _ => vec![mk(1, false, 1), mk(2, true, 2)],
    }
}

fn structural_scenario(idx: u64, k: u32, with_instance: bool, verify: bool, ring_slots: u64) -> Scenario {
    let per = if with_instance { 96u64 } else { 48u64 };
    let anchor = idx % 5;
    let mut rest = idx / 5;
    let mut specs = anchor_specs(anchor);
    let mut counter = specs.last().map_or(0, |s| s.counter);
    let mut tx = 1u64;
    for _ in 0..k {
        let code = rest % per; rest /= per;
        let (s, c, t) = structural_spec(code, counter, tx, with_instance);
        specs.push(s); counter = c; tx = t;
    }
    build(1, 1, &specs, ring_slots, verify, false)
}

#[test]
fn exhaustive_structural() {
    for (k, with_instance, verify, ring_slots) in [(1u32, true, false, 40u64), (2, true, true, 40), (3, false, true, 40), (3, false, false, 40), (3, false, true, 6), (3, true, true, 40)] {
        let per = if with_instance { 96u64 } else { 48 };
        let total = 5 * per.pow(k);
        let label: &'static str = Box::leak(format!("structural k={k} inst={with_instance} verify={verify} slots={ring_slots}").into_boxed_str());
        let t = sharded(super::zz_old::replay_journal_before, label, total, move |i| structural_scenario(i, k, with_instance, verify, ring_slots));
        assert_eq!(t.diverged, 0);
    }
}

// ---------- 穷举一族：反向链断点 × 点名项（k=3，结构 48^3）----------
fn chain_named_scenario(idx: u64) -> Scenario {
    let anchor = idx % 2; // 0: 无锚点条, 1: 锚点
    let mut rest = idx / 2;
    let mask = rest % 8; rest /= 8;
    let named_mask = rest % 27; rest /= 27;
    let mut specs = anchor_specs(if anchor == 0 { 0 } else { 1 });
    let mut counter = specs.last().map_or(0, |s| s.counter);
    let mut tx = 1;
    let mut nm = named_mask;
    for j in 0..3 {
        let code = rest % 48; rest /= 48;
        let (mut s, c, t) = structural_spec(code, counter, tx, false);
        s.chain_ok = (mask >> j) & 1 == 0;
        let kind = [Kind::Good, Kind::Bad, Kind::Absent][(nm % 3) as usize]; nm /= 3;
        s.named = vec![NamedSpec { class: UNIT_CLASS_DATA, kinds: [Kind::Good, kind] }];
        specs.push(s); counter = c; tx = t;
    }
    build(1, 1, &specs, 40, true, false)
}

#[test]
fn exhaustive_chain_and_named() {
    let total = 2 * 8 * 27 * 48u64.pow(3);
    sharded(super::zz_old::replay_journal_before, "chain+named k=3", total, chain_named_scenario);
}

// ---------- 计数器 u64::MAX：panic 行为 ----------
fn max_counter_scenario(i: u64) -> Scenario {
    let mut r = Rng(0xABCD ^ (i + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut specs = Vec::new();
    let base = [u64::MAX - 3, u64::MAX - 2, u64::MAX - 1, u64::MAX][r.below(4) as usize];
    for k in 0..r.below(4) {
        let (a, b, c) = (1 + r.below(3), 1 + r.below(2) as u32, r.chance(1, 2)); let mut s = random_spec(&mut r, 1, base.saturating_add(k), a, b, c, 5);
        s.named = vec![];
        specs.push(s);
    }
    build(1, 1 + r.below(2), &specs, 40, false, true)
}

#[test]
fn max_counter_family() {
    sharded(super::zz_old::replay_journal_before, "max-counter", 20_000, max_counter_scenario);
}

// ---------- 阳性对照：改前的四个变异体，差分必须能红 ----------
#[test]
fn positive_controls_are_red() {
    let muts: [(&str, OldFn); 4] = [
        ("m1 去掉 take(limit)", super::zz_old_m1::replay_journal_before),
        ("m2 点名失败不断", super::zz_old_m2::replay_journal_before),
        ("m3 序号 FIRST 改 >2", super::zz_old_m3::replay_journal_before),
        ("m4 max 只看第一条", super::zz_old_m4::replay_journal_before),
    ];
    silence();
    for (name, f) in muts {
        let mut t = Tally::default();
        for i in 0..40_000 {
            compare(f, &random_scenario(7, i, true), &mut t, name);
        }
        eprintln!("CONTROL[{name}] diverged={} / {}", t.diverged, t.total);
        assert!(t.diverged > 0, "{name} 没被差分抓到");
    }
}

#[test]
fn exhaustive_structural_quick() {
    for (k, with_instance, verify, ring_slots) in [(2u32, true, true, 40u64), (3, false, true, 40), (3, false, true, 6)] {
        let per = if with_instance { 96u64 } else { 48 };
        let total = 5 * per.pow(k);
        let label: &'static str = Box::leak(format!("structural-quick k={k} inst={with_instance} verify={verify} slots={ring_slots}").into_boxed_str());
        sharded(super::zz_old::replay_journal_before, label, total, move |i| structural_scenario(i, k, with_instance, verify, ring_slots));
    }
}

// ---------- 键与记录字段对不上的表（公开函数入参没有类型挡着；扫环产出的表键恒等于字段）----------
fn rekeyed_scenario(i: u64) -> Scenario {
    let mut sc = random_scenario(11, i, false);
    let mut r = Rng(0xFEED ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let old: Vec<_> = std::mem::take(&mut sc.records).into_iter().collect();
    let mut out = BTreeMap::new();
    for ((inst, counter), rec) in old {
        let key = if r.chance(1, 6) {
            (InstanceGeneration(inst.0.wrapping_add(r.below(2) as u32)), counter.wrapping_add(r.below(3)).min((1u64 << 48) - 1))
        } else { (inst, counter) };
        out.entry(key).or_insert(rec);
    }
    sc.records = out;
    sc
}
#[test]
fn key_mismatch_family() {
    sharded(super::zz_old::replay_journal_before, "key-mismatch", 300_000, rekeyed_scenario);
}
