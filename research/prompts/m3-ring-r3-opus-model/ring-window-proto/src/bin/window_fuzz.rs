//! m3-ring-r3 攻方原型 H1（一圈窗口第二次抽样）：放开用户动作的随机历史 + 全域静态判法。
//! 一段历史 = 若干「命」：每一命是一次可写挂载（新实例）加一串用户动作（覆盖写 O、空发布 E、准入抬 F、回退到随机候选 R），
//! 收尾是崩溃（层 0 的合法截点：前几段全落 + 截到的那一段任一子集）、正常卸载或直接退出；崩溃之后的盘面就是下一命的起点。
//! 每一命录成一条流，对流里每一个单元写 w 判：w 落了的崩溃状态（前 k 段全落 + 第 k 段任一子集）里，w 盖住的槽有没有被
//! 那个状态的回退候选集（有效 ∧ txg ≥ F_生效 ∧ 窗口臂再加一圈窗口，锚在那个状态读得出的最新根）里某条根引用（候选类），
//! 或被环里一条被抛弃的根引用（被抛弃类，I-7.4 后一半）。另一条路：每一命收尾的盘面跑池级 checker。
//! 挂载时的读故障（FUZZ_FAULTS=1）：随机藏一个根环槽（只在挂载那一次读失败），或在上一命崩在「根槽已落、系统配置没轮换」时
//! 藏住那条根与它那次发布的记录（造被抛弃的根，同 C554 那一形的做法）。
//!
//! 用法：window_fuzz <起始种子> <S> <段数> <today|window>
//! 环境：FUZZ_FAULTS=1 开读故障；PROTO_JOURNAL_BYTES 改 journal 环长（环绕回）；SINGLEFS_SHADOW_NO_WINDOW / SINGLEFS_ROLLBACK_NO_WINDOW /
//! FUZZ_CHECKER_NO_WINDOW=1 是四处漏改的臂（分配器始终按臂开窗口）。
#![allow(clippy::too_many_lines)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use ring_window_proto::{build_plain, content, geometry, parameters, publish_empty, recording_of, violated, HidingDevice, WRITE_TIME};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, effective_rollback_floor, instance_table_of_root,
    readable_roots_with_ring_slots, root_is_abandoned_by_the_instance_table, PoolReader,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const SLOT: u64 = 16384;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next() % n } }
    fn chance(&mut self, percent: u64) -> bool { self.below(100) < percent }
}

fn placements_of(reader: &dyn PoolReader, root: &RootRecord) -> BTreeSet<(u32, u64)> {
    let mut out = BTreeSet::new();
    if let Ok(records) = allocation_records_under_root(reader, root) {
        for r in records.iter().filter(|r| !r.is_released) {
            for s in r.slot.0..r.slot.0 + u64::from(r.span_slots) {
                out.insert((r.device.0, s));
            }
        }
    }
    for (locations, span) in [(&root.instance_table.locations, 2u64), (&root.tree_table.locations, 1u64)] {
        for l in locations.iter() {
            if l.slot.0 != 0 {
                for s in l.slot.0..l.slot.0 + span {
                    out.insert((l.device.0, s));
                }
            }
        }
    }
    out
}

struct Classified {
    newest: RootRecord,
    candidates: Vec<RootRecord>,
    abandoned: Vec<RootRecord>,
    outside: Vec<u64>,
}

fn classify(reader: &dyn PoolReader, window: bool) -> Option<Classified> {
    let configuration = choose_system_configuration(reader).ok()?;
    let im = &configuration.immutable;
    let roots: Vec<RootRecord> = readable_roots_with_ring_slots(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier).into_iter().map(|(_, r)| r).collect();
    let newest = *roots.iter().max_by_key(|r| (r.checkpoint_txg, r.instance))?;
    let table = instance_table_of_root(reader, &newest)?;
    let floor = effective_rollback_floor(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier);
    let ring_len = 3 * im.sizes.root_ring_slots_per_region.count();
    let (mut candidates, mut abandoned, mut outside) = (Vec::new(), Vec::new(), Vec::new());
    for r in roots {
        if root_is_abandoned_by_the_instance_table(&r, &table) {
            abandoned.push(r);
            continue;
        }
        if r.checkpoint_txg < floor && r != newest {
            continue;
        }
        if !window || r.checkpoint_txg.0 + ring_len > newest.checkpoint_txg.0 {
            candidates.push(r);
        } else {
            outside.push(r.checkpoint_txg.0);
        }
    }
    Some(Classified { newest, candidates, abandoned, outside })
}

#[derive(Default)]
struct Judged {
    unit_writes: u64,
    candidate_hits: u64,
    abandoned_hits: u64,
    first_candidate_hit: Option<String>,
    first_abandoned_hit: Option<String>,
    outside_seen: BTreeSet<u64>,
}

fn judge(base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], window: bool, out: &mut Judged, life: u64) {
    let mut image = base.clone();
    let mut cache: BTreeMap<(u32, u64, [u8; 8]), BTreeSet<(u32, u64)>> = BTreeMap::new();
    for (k, segment) in segments.iter().enumerate() {
        let units: Vec<usize> = segment.iter().copied().filter(|i| writes[*i].kind == StepKind::UnitWrite).collect();
        if !units.is_empty() {
            if let Some(c) = classify(&image, window) {
                out.outside_seen.extend(c.outside.iter().copied());
                let mut key_of = |r: &RootRecord| {
                    let mut tag = [0u8; 8];
                    tag.copy_from_slice(&r.mapping_root.locations[0].slot.0.to_le_bytes());
                    (r.instance.0, r.checkpoint_txg.0, tag)
                };
                let mut referenced: BTreeMap<(u32, u64), u64> = BTreeMap::new();
                for r in &c.candidates {
                    let key = key_of(r);
                    let set = cache.entry(key).or_insert_with(|| placements_of(&image, r)).clone();
                    for p in set { referenced.entry(p).or_insert(r.checkpoint_txg.0); }
                }
                let mut abandoned_ref: BTreeMap<(u32, u64), u64> = BTreeMap::new();
                for r in &c.abandoned {
                    let key = key_of(r);
                    let set = cache.entry(key).or_insert_with(|| placements_of(&image, r)).clone();
                    for p in set { abandoned_ref.entry(p).or_insert(r.checkpoint_txg.0); }
                }
                for i in units {
                    out.unit_writes += 1;
                    let w = &writes[i];
                    let first = w.offset.0 / SLOT;
                    let last = (w.offset.0 + w.length_in_bytes().max(1) - 1) / SLOT;
                    for slot in first..=last {
                        if let Some(txg) = referenced.get(&(w.device.0, slot)) {
                            out.candidate_hits += 1;
                            if out.first_candidate_hit.is_none() {
                                out.first_candidate_hit = Some(format!("life{life}/seg{k}/w{i}/dev{}/slot{slot}/cand_txg{txg}/cand_inst{}/newest{}", w.device.0, c.candidates.iter().find(|r| r.checkpoint_txg.0 == *txg).map_or(0, |r| r.instance.0), c.newest.checkpoint_txg.0));
                            }
                        }
                        if let Some(txg) = abandoned_ref.get(&(w.device.0, slot)) {
                            out.abandoned_hits += 1;
                            if out.first_abandoned_hit.is_none() {
                                out.first_abandoned_hit = Some(format!("life{life}/seg{k}/w{i}/dev{}/slot{slot}/aband_txg{txg}/newest{}", w.device.0, c.newest.checkpoint_txg.0));
                            }
                        }
                    }
                }
            }
        }
        for i in segment {
            image.apply_writes(std::slice::from_ref(&writes[*i]));
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed0: u64 = a[1].parse().expect("种子");
    let s: u64 = a[2].parse().expect("S");
    let count: u64 = a[3].parse().expect("段数");
    let arm = a[4].clone();
    // 臂：today（今天）；window（四处同改，检查侧可用 SINGLEFS_SHADOW_NO_WINDOW / SINGLEFS_ROLLBACK_NO_WINDOW / FUZZ_CHECKER_NO_WINDOW 漏改一处）；
    // checkeronly（漏改分配器：分配器照今天、checker 与判法开窗口）。
    let window = arm == "window" || arm == "checkeronly";
    if window {
        if arm == "window" {
            std::env::set_var("SINGLEFS_WINDOW_ANCHOR", "ring");
        }
        if !std::env::var("FUZZ_CHECKER_NO_WINDOW").is_ok_and(|v| v == "1") {
            std::env::set_var("SINGLEFS_CHECKER_WINDOW", "1");
            std::env::set_var("SINGLEFS_CHECKER_WINDOW_RING_LEN", (3 * s).to_string());
        }
    }
    let faults = std::env::var("FUZZ_FAULTS").is_ok_and(|v| v == "1");
    // 截点取法的概率（百分比）：挂载里就崩、截在末次根槽写之前 / 根槽之后系统配置之前 / 末尾 14 段里任一段的上界；默认 25 与 35,50,85。
    let max_actions: u64 = std::env::var("FUZZ_MAX_ACTIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(3 * s + 7);
    let p_mount_crash: u64 = std::env::var("FUZZ_P_MOUNT_CRASH").ok().and_then(|v| v.parse().ok()).unwrap_or(25);
    let cuts: Vec<u64> = std::env::var("FUZZ_P_CUTS").ok().map_or(vec![35, 50, 85], |v| v.split(',').map(|x| x.parse().expect("百分比")).collect());
    let parameters = parameters(s);
    let geo = geometry(&parameters);
    for seed in seed0..seed0 + count {
        let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ s);
        let start_txg = 1 + rng.below(3 * s + 6);
        let (mut image, _) = build_plain(s, start_txg);
        let mut log = format!("start{start_txg}");
        let mut judged = Judged::default();
        let mut reds: Vec<String> = Vec::new();
        let mut refusals: BTreeMap<String, u64> = BTreeMap::new();
        let mut abandon_next: Option<Vec<(DeviceIdentity, u64, u64)>> = None;
        let lives = 2 + rng.below(4);
        let mut aborted = String::new();
        for life in 0..lives {
            let stream = SharedStream::retaining_contents();
            let hidden: Rc<RefCell<Vec<(DeviceIdentity, u64, u64)>>> = Rc::new(RefCell::new(Vec::new()));
            if faults {
                if let Some(ranges) = abandon_next.take() {
                    hidden.borrow_mut().extend(ranges);
                    log.push_str(" |hideNewest");
                } else if rng.chance(30) {
                    let slot = target_for_publish(CheckpointTxg(rng.below(3 * s)), parameters.geometry.root_ring_slots_per_region);
                    let device = parameters.region_devices[usize::try_from(slot.region).expect("区域")];
                    let off = slot_offset(slot, parameters.geometry.fixed_structure_slot_spacing).0;
                    hidden.borrow_mut().push((device, off, off + 512));
                    let occupant = choose_system_configuration(&image).ok().and_then(|c| {
                        readable_roots_with_ring_slots(&image, &c.immutable.region_devices, &c.immutable.sizes, &c.immutable.filesystem_identifier)
                            .into_iter().find(|(rs, _)| *rs == slot).map(|(_, r)| r.checkpoint_txg.0)
                    });
                    let newest = classify(&image, false).map(|c| c.newest.checkpoint_txg.0);
                    log.push_str(&format!(" |hideSlot{}:{}=txg{occupant:?}/newest{newest:?}", slot.region, slot.slot));
                }
            }
            let mut devices: Vec<(DeviceIdentity, HidingDevice<_>)> = recording_of(&image, &stream)
                .into_iter()
                .map(|(id, rec)| (id, HidingDevice { device: id, inner: rec, hidden: hidden.clone() }))
                .collect();
            let mounted = match mount_writable(&parameters, &mut devices) {
                Ok(m) => m,
                Err(e) => {
                    aborted = format!("life{life}:mount_err:{}", format!("{e:?}").chars().take(80).collect::<String>());
                    break;
                }
            };
            hidden.borrow_mut().clear();
            let instance = mounted.output.instance;
            log.push_str(&format!(" |m{}@{}", instance.0, mounted.output.row_publish.root().checkpoint_txg.0));
            let mut allocator = mounted.allocator.clone();
            let Some(mut current) = mounted.current.clone().into_file_version() else {
                aborted = format!("life{life}:no_file");
                break;
            };
            let crash_in_mount = rng.chance(p_mount_crash);
            let actions = if crash_in_mount { 0 } else { rng.below(max_actions) };
            for step in 0..actions {
                let roll = rng.below(100);
                if roll < 60 {
                    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                    match publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(seed + life * 100 + step), write_time_seconds: WRITE_TIME + step }, instance) {
                        Ok(o) => { current = o; log.push('O'); }
                        Err(e) => { *refusals.entry(format!("O:{}", format!("{e:?}").chars().take(40).collect::<String>())).or_default() += 1; log.push('o'); }
                    }
                } else if roll < 72 {
                    let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                    match publish_empty(&mut pool, &mut allocator, &current, instance) {
                        Ok(o) => { current = o; log.push('E'); }
                        Err(_) => log.push('e'),
                    }
                } else if roll < 84 {
                    match raise_rollback_floor_to_the_admission_ceiling(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On) {
                        Ok(_) => log.push('F'),
                        Err(e) => { *refusals.entry(format!("F:{}", format!("{e:?}").chars().take(40).collect::<String>())).or_default() += 1; log.push('f'); }
                    }
                } else {
                    let any = rng.chance(50);
                    let target = classify(&devices, window && !any).and_then(|c| {
                        let pool: Vec<RootRecord> = c.candidates.into_iter().filter(|r| r.checkpoint_txg != c.newest.checkpoint_txg).collect();
                        if pool.is_empty() { None } else { Some(pool[usize::try_from(rng.below(pool.len() as u64)).expect("小")]) }
                    });
                    match target {
                        None => log.push('_'),
                        Some(t) => match roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut current, RollbackTarget { instance: t.instance, checkpoint_txg: t.checkpoint_txg }) {
                            Ok(_) => log.push_str(&format!("R{}", t.checkpoint_txg.0)),
                            Err(e) => { *refusals.entry(format!("R:{}", format!("{e:?}").chars().take(40).collect::<String>())).or_default() += 1; log.push_str(&format!("r{}", t.checkpoint_txg.0)); }
                        },
                    }
                }
            }
            let end = rng.below(100);
            let crash = crash_in_mount || end < 60;
            if !crash && end < 80 {
                let mut version = PoolVersion::WithFile(current.clone());
                match stream.record_entry(singlefs_harness::RecordedPublishEntry::Unmount, || unmount(&parameters, &mut devices, &mut allocator, &mut version, ShadowLedger::On)) {
                    Ok(_) => log.push('U'),
                    Err(e) => { *refusals.entry(format!("U:{}", format!("{e:?}").chars().take(40).collect::<String>())).or_default() += 1; log.push('u'); }
                }
            }
            drop(devices);
            let operations = stream.retained_operations();
            let (writes, segments, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(&operations, &stream.entry_spans(), &geo);
            judge(&image, &writes, &segments, window && !std::env::var("FUZZ_JUDGE_NO_WINDOW").is_ok_and(|v| v == "1"), &mut judged, life);
            let mut next = image.clone();
            if crash && !segments.is_empty() {
                let roots: Vec<usize> = segments.iter().enumerate().filter(|(_, seg)| seg.iter().any(|i| writes[*i].kind == StepKind::RootRecordFua)).map(|(k, _)| k).collect();
                let pick = rng.below(100);
                let (k, subset_all_out, cut_kind) = if pick < cuts[0] && !roots.is_empty() {
                    (*roots.last().expect("有根"), true, "beforeLastRoot")
                } else if pick < cuts[1] && !roots.is_empty() {
                    (roots.last().expect("有根") + 1, true, "afterLastRootBeforeRotation")
                } else if pick < cuts[2] {
                    let lo = segments.len().saturating_sub(14);
                    (lo + usize::try_from(rng.below((segments.len() - lo) as u64)).expect("小"), false, "nearEnd")
                } else {
                    (usize::try_from(rng.below(segments.len() as u64)).expect("小"), false, "anywhere")
                };
                let k = k.min(segments.len());
                for seg in &segments[..k] {
                    for i in seg { next.apply_writes(std::slice::from_ref(&writes[*i])); }
                }
                if k < segments.len() && !subset_all_out {
                    for i in &segments[k] {
                        if rng.chance(50) { next.apply_writes(std::slice::from_ref(&writes[*i])); }
                    }
                }
                log.push_str(&format!("C{cut_kind}{k}/{}", segments.len()));
                if cut_kind == "afterLastRootBeforeRotation" && faults && rng.chance(70) {
                    // 下一次挂载藏住这条根与它那次发布的记录：恢复落到上一条根，这条根在下一次写行之后被抛弃。
                    let last_root_seg = *roots.last().expect("有根");
                    let root_write = segments[last_root_seg].iter().copied().find(|i| writes[*i].kind == StepKind::RootRecordFua).expect("根槽写");
                    let mut ranges = vec![(writes[root_write].device, writes[root_write].offset.0, writes[root_write].offset.0 + writes[root_write].length_in_bytes())];
                    let first_of_publish = segments[..last_root_seg].iter().rposition(|seg| seg.iter().any(|i| writes[*i].kind == StepKind::RootRecordFua)).map_or(0, |p| p + 1);
                    for seg in &segments[first_of_publish..last_root_seg] {
                        for i in seg {
                            // 藏那次发布的单元（记录点名的单元读不出 ⇒ 记录不施加，但记录自证过、仍算进 first_txg，写行不会取同一个 txg）。
                            if writes[*i].kind == StepKind::UnitWrite {
                                ranges.push((writes[*i].device, writes[*i].offset.0, writes[*i].offset.0 + writes[*i].length_in_bytes()));
                            }
                        }
                    }
                    abandon_next = Some(ranges);
                }
            } else {
                next.apply_writes(&writes);
            }
            let red = violated(&next);
            if !red.is_empty() {
                reds.push(format!("life{life}:{red:?}"));
            }
            image = next;
        }
        println!(
            "FUZZ seed={seed} s={s} arm={arm} faults={faults} journal={} lives={lives} unit_writes={} candidate_hits={} abandoned_hits={} first_candidate_hit={:?} first_abandoned_hit={:?} outside_seen={} checker_reds={reds:?} refusals={refusals:?} aborted={aborted:?} log={log}",
            parameters.geometry.journal_ring_bytes, judged.unit_writes, judged.candidate_hits, judged.abandoned_hits, judged.first_candidate_hit, judged.first_abandoned_hit, judged.outside_seen.len()
        );
    }
}
