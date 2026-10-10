//! 造出来的镜像（都是坏盘，不是合法历史）：R1 认的「下一次挂载先施加的那一版」在合法历史走不到的几支上与 core 施加的第一次发布比。
//! - multi：mkfs、挂载、第一个文件（A）之后做两步用户动作 B、C（从一张表里放开扫，不写死），再把 B、C 写的每一条根槽都撕掉
//!   （根槽里那 512 字节翻一个字节，自证不过）：最新根退回 A 的，重放要连着施加 B、C 的几次发布。四个变体：
//!   clean（不再动）、stop-last（最后一次发布的一条记录的一个点名单元盘 0 那一份翻一个字节）、stop-first（第一次发布的那一份翻）、
//!   gap-first（第一次发布有两条以上记录时抹掉它的第一条两份）。
//! - txggap：A 之后一次覆盖写 B 崩在根槽之前（记录都落了），把 B 的记录 txg 从 T+1 改成 T+2、重封：锚点在，重放照收，R1 认 (实例, T+2)。
//! - inflight：journal 环缩到 `R` = 256 条（在飞上限 ⌊R ÷ 3⌋ = 85），一次大顺序写的数据单元数取上限上下几个，崩在根槽之前。
//! 每张镜像：新 checker、pre36 判红的、探针认的那一版与重放结局、core 施加的第一次与最后一次、core 只读恢复与可写挂载的结局。
use singlefs_core::address::DeviceIdentity;
use singlefs_core::journal::JournalRecord;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;

use crate::core::{classify, core_view};
use crate::craft::{self, read, write, SLOT};
use crate::drive::{ring_roots_newest_first, Driver, Op};
use crate::judge::{self, reds};
use crate::random::{image_with, WIDTHS};
use crate::tally::Tally;

pub fn report(label: &str, width: HistoryDeviceWidth, params: &singlefs_core::make_filesystem::MakeFilesystemParameters, image: &MemoryPool, tally: &mut Tally) {
    let new = judge::new(image);
    let (probe_verdicts, probe) = judge::probe(image);
    let (pre, pre_probe) = judge::pre36(image);
    let core = core_view(image);
    let nm = probe.next_mount.as_ref().map(|(i, t, s)| (*i, *t, s.as_slice()));
    let class = classify(nm, core.as_ref().and_then(|c| c.first.as_ref()), &core);
    let pre_nm = pre_probe.next_mount.as_ref().map(|(i, t, s)| (*i, *t, s.as_slice()));
    let pre_class = classify(pre_nm, core.as_ref().and_then(|c| c.first.as_ref()), &core);
    let recovered = recover(image, JournalPolicy::Consult);
    let recovered_text = match &recovered.outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile {root:?}"),
        RecoveryOutcome::FileRead { root, content } => format!("FileRead {root:?} bytes={}", content.len()),
        RecoveryOutcome::Failed { root, failure } => format!("Failed {root:?} {failure:?}"),
    };
    let mut driver = Driver::from_image_with(width, params.clone(), image);
    let mounted = driver.execute(Op::Mount);
    let kind = label.split('/').next().unwrap_or(label);
    tally.add(format!("{kind} new-vs-core-first={class} pre36-vs-core-first={pre_class} replay={}", probe.replay_kind), label, String::new);
    if probe_verdicts != new {
        tally.add("SANITY probe!=new", label, String::new);
    }
    println!(
        "B36X label={label} new_red={:?} pre36_red={:?} newest={:?} next_mount={:?} pre36_next_mount={:?} replay={} replayed={:?} core_first={:?} core_last={:?} prefix={} vfail={} class={class} pre36_class={pre_class}",
        reds(&new),
        reds(&pre),
        probe.newest,
        probe.next_mount.as_ref().map(|v| (v.0, v.1)),
        pre_probe.next_mount.as_ref().map(|v| (v.0, v.1)),
        probe.replay_kind,
        probe.replayed.as_ref().map(|v| (v.0, v.1, v.3)),
        core.as_ref().and_then(|c| c.first.as_ref().map(|f| f.0)),
        core.as_ref().and_then(|c| c.last.as_ref().map(|f| f.0)),
        core.as_ref().map_or(0, |c| c.prefix_applied),
        core.as_ref().map_or(0, |c| c.verification_failed),
    );
    println!(
        "B36X   mount label={label} recover={} effective={:?} writable={}",
        recovered_text.chars().take(160).collect::<String>(),
        recovered.effective_root,
        mounted.chars().take(160).collect::<String>()
    );
}

/// 根槽里 txg 等于 `txg` 的每一条（两盘）翻第 100 个字节（树表指针里），自证不过。交回撕了几条。
pub fn tear_roots_of_txg(image: &mut MemoryPool, txg: u64) -> usize {
    let mut torn = 0;
    for (device, offset) in craft::root_positions(image) {
        let mut bytes = read(image, device, offset, 512);
        if bytes[..4] == [0u8; 4] || u64::from_le_bytes(bytes[28..36].try_into().expect("8")) != txg {
            continue;
        }
        bytes[100] ^= 0x5a;
        write(image, device, offset, &bytes);
        torn += 1;
    }
    torn
}

/// 环里 (实例, txg) 的那几条记录（每块盘逐槽扫）：(盘, 计数器, 偏移)。
pub fn records_of(image: &MemoryPool, ring_bytes: u64, instance: u32, txg: u64) -> Vec<(u32, u64, u64)> {
    let mut out = Vec::new();
    for device in craft::DEVICES {
        for slot in 0..ring_bytes / 4096 {
            let offset = singlefs_format::JOURNAL_RING_START_SLOT * SLOT + slot * 4096;
            let bytes = read(image, device, offset, 4096);
            let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
            let Some(record) = JournalRecord::parse(&bytes, fsid) else { continue };
            if record.instance.0 == instance && record.checkpoint_txg.0 == txg {
                out.push((device, record.counter, offset));
            }
        }
    }
    out.sort_by_key(|r| (r.1, r.0));
    out
}

/// (实例, txg) 那次发布计数器最大（`last` = true）或最小的那条记录里第一个点名单元，盘 0 上那一份翻一个字节。交回翻没翻成。
pub fn corrupt_a_named_unit(image: &mut MemoryPool, ring_bytes: u64, instance: u32, txg: u64, last: bool) -> bool {
    let records = records_of(image, ring_bytes, instance, txg);
    let pick = if last { records.iter().rev().find(|r| r.0 == 0) } else { records.iter().find(|r| r.0 == 0) };
    let Some((device, _, offset)) = pick else { return false };
    let bytes = read(image, *device, *offset, 4096);
    let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
    let Some(record) = JournalRecord::parse(&bytes, fsid) else { return false };
    let Some(named) = record.named.first() else { return false };
    let location = named.locations.iter().find(|l| l.device == DeviceIdentity(0)).unwrap_or(&named.locations[0]);
    let unit_offset = location.slot.0 * SLOT;
    let mut unit = read(image, location.device.0, unit_offset, 4096);
    unit[1000] ^= 0xff;
    write(image, location.device.0, unit_offset, &unit);
    true
}

/// 一次发布的根槽之前、新进程起来之前那一刻的镜像（记录全落、根槽没落）：交回 (镜像, 那次发布的 txg)。
fn crash_before_the_last_root(driver: &mut Driver, op: Op) -> Option<(MemoryPool, String)> {
    let before = driver.image();
    let start = driver.stream.operation_count();
    let outcome = driver.execute(op);
    if !outcome.starts_with("ok") {
        return None;
    }
    let ops = driver.ops_since(start);
    let geometry = driver.geometry();
    let root = ops.iter().rposition(|o| geometry.classify(&o.operation) == StepKind::RootRecordFua)?;
    Some((image_with(&before, &ops, &(0..root).collect::<Vec<_>>()), outcome))
}

const B_AND_C: [Op; 7] = [
    Op::Overwrite(11),
    Op::SequentialWrite(12),
    Op::SequentialWrite(14),
    Op::CreateInode,
    Op::Unlink,
    Op::RaiseFloor(3),
    Op::Rollback(1),
];

pub fn run() {
    let mut tally = Tally::default();
    // multi
    for width in WIDTHS {
        for b in B_AND_C {
            for c in B_AND_C {
                let mut d = Driver::mkfs(width);
                let ring = d.params.geometry.journal_ring_bytes;
                for op in [Op::Mount, Op::FirstFile(5)] {
                    assert!(d.execute(op).starts_with("ok"));
                }
                let a_txg = ring_roots_newest_first(&d.image())[0].0;
                let (ob, oc) = (d.execute(b), d.execute(c));
                if !ob.starts_with("ok") || !oc.starts_with("ok") {
                    tally.add(format!("multi skipped (B or C not ok)"), &format!("{width:?} {b:?} {c:?}"), || format!("{ob} / {oc}"));
                    continue;
                }
                let end = d.image();
                let newer: Vec<(u64, u32)> = ring_roots_newest_first(&end).into_iter().filter(|(t, _)| *t > a_txg).collect();
                let mut clean = end.clone();
                for (txg, _) in &newer {
                    tear_roots_of_txg(&mut clean, *txg);
                }
                let label = format!("{width:?} B={b:?} C={c:?} a_txg={a_txg} torn={:?}", newer.iter().map(|r| r.0).collect::<Vec<_>>());
                report(&format!("multi-clean/{label}"), width, &d.params, &clean, &mut tally);
                let instance = newer.first().map_or(1, |r| r.1);
                let (first_txg, last_txg) = (newer.iter().map(|r| r.0).min(), newer.iter().map(|r| r.0).max());
                if let (Some(first_txg), Some(last_txg)) = (first_txg, last_txg) {
                    let mut stop_last = clean.clone();
                    if first_txg != last_txg && corrupt_a_named_unit(&mut stop_last, ring, instance, last_txg, true) {
                        report(&format!("multi-stop-last/{label}"), width, &d.params, &stop_last, &mut tally);
                    }
                    let mut stop_first = clean.clone();
                    if corrupt_a_named_unit(&mut stop_first, ring, instance, first_txg, false) {
                        report(&format!("multi-stop-first/{label}"), width, &d.params, &stop_first, &mut tally);
                    }
                    let first_records = records_of(&clean, ring, instance, first_txg);
                    let counters: std::collections::BTreeSet<u64> = first_records.iter().map(|r| r.1).collect();
                    if counters.len() >= 2 {
                        let mut gap = clean.clone();
                        let lowest = *counters.iter().next().expect("有");
                        for (device, counter, offset) in &first_records {
                            if *counter == lowest {
                                write(&mut gap, *device, *offset, &vec![0u8; 4096]);
                            }
                        }
                        report(&format!("multi-gap-first/{label}"), width, &d.params, &gap, &mut tally);
                    }
                }
            }
        }
    }
    // txggap
    for width in WIDTHS {
        let mut d = Driver::mkfs(width);
        let ring = d.params.geometry.journal_ring_bytes;
        for op in [Op::Mount, Op::FirstFile(5)] {
            assert!(d.execute(op).starts_with("ok"));
        }
        let (a_txg, instance) = ring_roots_newest_first(&d.image())[0];
        for op in [Op::Overwrite(21), Op::SequentialWrite(22)] {
            let mut d2 = Driver::from_image(width, &d.image());
            assert!(d2.execute(Op::Mount).starts_with("ok"));
            let (a2_txg, instance2) = ring_roots_newest_first(&d2.image())[0];
            let Some((mut image, _)) = crash_before_the_last_root(&mut d2, op) else { continue };
            report(&format!("txggap-base/{width:?} {op:?} a_txg={a2_txg}"), width, &d2.params, &image, &mut tally);
            let b = records_of(&image, ring, instance2, a2_txg + 1);
            let mut changed = std::collections::BTreeSet::new();
            for (device, counter, offset) in &b {
                let mut bytes = read(&image, *device, *offset, 4096);
                bytes[26..34].copy_from_slice(&(a2_txg + 2).to_le_bytes());
                craft::reseal_record(&mut bytes);
                write(&mut image, *device, *offset, &bytes);
                changed.insert((*device, *counter));
            }
            craft::rechain(&mut image, ring, changed);
            report(&format!("txggap/{width:?} {op:?} a_txg={a2_txg} records={}", b.len()), width, &d2.params, &image, &mut tally);
        }
        let _ = (a_txg, instance);
    }
    // inflight
    // 环长要让单元区起点落在 64 槽的段边界上（mkfs 拒别的），最短 256 条（1 MiB）。
    for ring_records in [256u64] {
        let width = HistoryDeviceWidth::FourGibibytes;
        let mut params = width.parameters();
        params.geometry.journal_ring_bytes = ring_records * 4096;
        for units in [ring_records / 3 - 1, ring_records / 3, ring_records / 3 + 1, ring_records / 3 + 4] {
            let mut d = Driver::mkfs_with(width, params.clone());
            for op in [Op::Mount, Op::FirstFile(5)] {
                assert!(d.execute(op).starts_with("ok"));
            }
            match crash_before_the_last_root(&mut d, Op::BigWrite(units)) {
                Some((image, outcome)) => {
                    let (a_txg, instance) = ring_roots_newest_first(&image)[0];
                    let n = records_of(&image, ring_records * 4096, instance, a_txg + 1).len();
                    report(&format!("inflight/ring={ring_records} limit={} units={units} b_record_copies={n} outcome={}", ring_records / 3, outcome.split_whitespace().next().unwrap_or("")), width, &params, &image, &mut tally);
                }
                None => println!("B36X inflight ring={ring_records} units={units} BigWrite not ok"),
            }
        }
    }
    tally.print("B36XT");
}

/// 另两支（`crafted2`）：锚点认不定（所选根那次发布带末条标志的记录多出一条：把锚点那条抄到计数器 + 1 的槽上、改计数器与反向链、重封），
/// 所选根 txg 没有后继（最新根的 txg 改成 u64 的最大值、重封、同 txg 记录的新根段照抄）。两支都是坏盘。
pub fn run_extra() {
    use singlefs_core::journal::{back_chain_of, record_offset};
    let mut tally = Tally::default();
    for width in WIDTHS {
        let mut d = Driver::mkfs(width);
        let ring = d.params.geometry.journal_ring_bytes;
        for op in [Op::Mount, Op::FirstFile(5)] {
            assert!(d.execute(op).starts_with("ok"));
        }
        let (a_txg, instance) = ring_roots_newest_first(&d.image())[0];
        let base = d.image();
        let anchors = records_of(&base, ring, instance, a_txg);
        let mut doubled = base.clone();
        for (device, counter, offset) in &anchors {
            let mut bytes = read(&doubled, *device, *offset, 4096);
            let chain = back_chain_of(&bytes);
            bytes[20..26].copy_from_slice(&(counter + 1).to_le_bytes()[..6]);
            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
            craft::reseal_record(&mut bytes);
            write(&mut doubled, *device, record_offset(counter + 1, ring).0, &bytes);
        }
        report(&format!("anchor-undecided/{width:?} a=({instance}, {a_txg}) anchors={:?}", anchors.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>()), width, &d.params, &doubled, &mut tally);
        let mut max_txg = base.clone();
        let rewritten = craft::rewrite_root(&mut max_txg, ring, a_txg, |bytes| bytes[28..36].copy_from_slice(&u64::MAX.to_le_bytes()));
        report(&format!("no-successor/{width:?} a=({instance}, {a_txg}) roots_rewritten={rewritten}"), width, &d.params, &max_txg, &mut tally);
    }
    tally.print("B36XT2");
}
