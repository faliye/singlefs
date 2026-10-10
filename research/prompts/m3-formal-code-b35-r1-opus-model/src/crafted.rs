//! 造出来的镜像与流（P1 取哪条记录、P2 全零树表指针、P3 四条对拍流的 CPU 变异对应与「拿掉根槽写」的构造）：
//! 新 checker、pre35、三份 CPU 变异的判定，与 core 两种挂载的结局。造盘工具照第 34 批攻方原型（`craft.rs` 与 `p2.rs`）。
use std::collections::BTreeSet;

use singlefs_checker_tier::crash::{full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;

use crate::craft::{self, read, write, SLOT};
use crate::drive::{Driver, Op};
use crate::judge::{self, reds};
use crate::random::{cuts, image_with};

/// 一张镜像：几份 checker 判红的、core 只读恢复与可写挂载的结局。
pub fn report(label: &str, width: HistoryDeviceWidth, image: &MemoryPool) {
    let new = judge::new(image);
    let (pre, _) = judge::pre35(image);
    let (_, probe) = judge::probe(image);
    let line = |name: &str, v: &judge::Verdicts| format!("{name}={:?}", reds(v));
    println!(
        "B35X label={label} {} {} {} {} {} next_mount={:?} versions={:?} replayed={:?}",
        line("new", &new),
        line("pre35", &pre),
        line("mutr", &judge::mutant("mutr", image)),
        line("mutn", &judge::mutant("mutn", image)),
        line("mutz", &judge::mutant("mutz", image)),
        probe.next_mount.as_ref().map(|v| (v.0, v.1)),
        probe.versions.iter().map(|v| (v.0, v.1)).collect::<Vec<_>>(),
        probe.replayed.as_ref().map(|v| (v.0, v.1, v.3)),
    );
    for name in ["I-2.1", "I-3.1", "I-3.10", "I-4.1", "I-4.8", "I-5.4", "I-5.5", "I-7.2", "I-7.4", "I-9.28", "I-9.5", "I-4.7", "I-4.6", "I-3.4", "I-9.29"] {
        let k = &new[name];
        println!("B35X   {label} {name} new={}[{}]", k.0, k.1.chars().take(220).collect::<String>());
    }
    let recovered = recover(image, JournalPolicy::Consult);
    let recovered_text = match &recovered.outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile {root:?}"),
        RecoveryOutcome::FileRead { root, content } => format!("FileRead {root:?} bytes={}", content.len()),
        RecoveryOutcome::Failed { root, failure } => format!("Failed {root:?} {failure:?}"),
    };
    let mut driver = Driver::from_image(width, image);
    let mounted = driver.execute(Op::Mount);
    println!(
        "B35X mount label={label} recover={} effective={:?} writable={}",
        recovered_text.chars().take(200).collect::<String>(),
        recovered.effective_root,
        mounted.chars().take(200).collect::<String>()
    );
}

pub fn history(width: HistoryDeviceWidth, ops: &[Op]) -> Driver {
    let mut driver = Driver::mkfs(width);
    for op in ops {
        let outcome = driver.execute(*op);
        assert!(outcome.starts_with("ok"), "{op:?} → {outcome}");
    }
    driver
}

/// 做一步：交回那一步之前的镜像、那一步的录制流、最后一次根槽写的下标。
pub fn step(driver: &mut Driver, op: Op) -> (MemoryPool, Vec<RetainedOperation>, usize) {
    let before = driver.image();
    let start = driver.stream.operation_count();
    let outcome = driver.execute(op);
    assert!(outcome.starts_with("ok"), "{op:?} → {outcome}");
    let ops = driver.ops_since(start);
    let geometry = driver.geometry();
    let root = ops.iter().rposition(|o| geometry.classify(&o.operation) == StepKind::RootRecordFua).expect("这一步写了根");
    (before, ops, root)
}

pub fn root_txgs(image: &MemoryPool) -> Vec<u64> {
    crate::drive::ring_roots_newest_first(image).into_iter().map(|(txg, _)| txg).collect()
}

/// 环里 (实例, txg) 的那几条记录（每块盘，逐槽扫、不按计数器停）：(盘, 计数器, 偏移)。
pub fn records_of(image: &MemoryPool, ring_bytes: u64, instance: u32, txg: u64) -> Vec<(u32, u64, u64)> {
    let mut out = Vec::new();
    for device in craft::DEVICES {
        for slot in 0..ring_bytes / 4096 {
            let offset = singlefs_format::JOURNAL_RING_START_SLOT * SLOT + slot * 4096;
            let bytes = read(image, device, offset, 4096);
            let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
            let Some(record) = singlefs_core::journal::JournalRecord::parse(&bytes, fsid) else { continue };
            if record.instance.0 == instance && record.checkpoint_txg.0 == txg {
                out.push((device, record.counter, offset));
            }
        }
    }
    out
}

/// 改几条记录（`change` 收整条 4096），重封；同实例紧跟着的记录重链（`craft::rechain`）。
pub fn rewrite_records(image: &mut MemoryPool, ring_bytes: u64, which: &[(u32, u64, u64)], change: impl Fn(&mut [u8])) {
    let mut changed = BTreeSet::new();
    for (device, counter, offset) in which {
        let mut bytes = read(image, *device, *offset, 4096);
        change(&mut bytes);
        craft::reseal_record(&mut bytes);
        write(image, *device, *offset, &bytes);
        changed.insert((*device, *counter));
    }
    craft::rechain(image, ring_bytes, changed);
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对同盘首尾相接的记录，前一条跨度加一（偏移照 `checker_known_bad_images.rs`）。
fn widen_one_record(bytes: &mut [u8]) -> bool {
    if bytes[50] != 0 || bytes[51] != 10 {
        return false;
    }
    let count = usize::from(u16::from_le_bytes([bytes[102], bytes[103]]));
    let at = |b: &[u8], i: usize| {
        let s = 135 + 20 * i;
        let mut slot = [0u8; 8];
        slot[..6].copy_from_slice(&b[s + 4..s + 10]);
        (s, u32::from_le_bytes(b[s..s + 4].try_into().expect("4")), u64::from_le_bytes(slot), u16::from_le_bytes([b[s + 10], b[s + 11]]))
    };
    for i in 0..count.saturating_sub(1) {
        let (s, device, slot, span_field) = at(bytes, i);
        let (_, next_device, next_slot, _) = at(bytes, i + 1);
        let span = span_field & 0x7fff;
        if device == next_device && slot + u64::from(span) == next_slot {
            bytes[s + 10..s + 12].copy_from_slice(&((span_field & 0x8000) | (span + 1)).to_le_bytes());
            return true;
        }
    }
    false
}

/// 那一版（单元头诞生代号 = txg）自己写的第一片改得动的分配记录叶：两块盘都改、重封、沿引用链补到记录。
pub fn overlap_in_the_version(image: &mut MemoryPool, ring_bytes: u64, txg: u64) -> Option<u64> {
    for (slot, length) in craft::units(image) {
        let bytes = read(image, 0, slot * SLOT, length);
        if bytes[6] != 2 || bytes[51] != 10 || u64::from_le_bytes(bytes[72..80].try_into().expect("8")) != txg {
            continue;
        }
        if !widen_one_record(&mut bytes.clone()) {
            continue;
        }
        let mut changes = Vec::new();
        for device in craft::DEVICES {
            let mut copy = read(image, device, slot * SLOT, length);
            let before = singlefs_core::checksum::crc32_castagnoli(&copy);
            if !widen_one_record(&mut copy) {
                continue;
            }
            craft::reseal_unit(&mut copy);
            write(image, device, slot * SLOT, &copy);
            changes.push((device, slot, before, singlefs_core::checksum::crc32_castagnoli(&copy)));
        }
        craft::propagate(image, ring_bytes, changes);
        return Some(slot);
    }
    None
}

pub fn segment_of_root_bytes(root_bytes: &[u8]) -> Vec<u8> {
    let mut segment = vec![0u8; 364];
    for (s, r) in craft::SEGMENT_FROM_ROOT.iter().cloned() {
        segment[s].copy_from_slice(&root_bytes[r]);
    }
    segment
}

pub fn root_bytes_of(image: &MemoryPool, txg: u64) -> Vec<u8> {
    craft::root_positions(image)
        .into_iter()
        .map(|(device, offset)| read(image, device, offset, 512))
        .find(|bytes| &bytes[..4] != [0u8; 4] && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == txg)
        .expect("那条根在根环里")
}

/// (实例, txg) 那次发布计数器最小的那条记录（两块盘各一份）的新根段换成 `stale`。
pub fn stale_first_record(image: &mut MemoryPool, ring_bytes: u64, instance: u32, txg: u64, stale: &[u8]) -> usize {
    let all = records_of(image, ring_bytes, instance, txg);
    let first = all.iter().map(|(_, c, _)| *c).min().expect("有记录");
    let firsts: Vec<_> = all.into_iter().filter(|(_, c, _)| *c == first).collect();
    rewrite_records(image, ring_bytes, &firsts, |bytes| bytes[99..99 + 364].copy_from_slice(stale));
    firsts.len()
}

/// 一段录制流（`base` 之上的 `ops`）按层 0 的枚举域全展开（`judge_layer0_state_range` + `full_expansion`），每个状态：新 checker（层 0 那一份）
/// 与 CPU 变异 `mutant` 判红的集合比，探针记这一状态上认出的两类版本。打一行汇总与第一处不同。
pub fn enumerate_flow(label: &str, base: &MemoryPool, ops: &[RetainedOperation], geometry: &singlefs_harness::segments::FixedGeometry, mutant: &str, threads: u64) {
    let (writes, segments) = writes_and_segments(ops, geometry);
    let count = layer0_state_count_with_torn_in_place_overwrites(base, &writes, &segments, &full_expansion);
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    println!("B35F label={label} mutant={mutant} segments={sizes:?} closed_form={} layer0_states={count}", singlefs_harness::memory_pool::closed_form_state_count(&segments));
    if count > 1_000_000 || std::env::var("B35_COUNT_ONLY").as_deref() == Ok("1") {
        println!("B35F-SKIP label={label} states={count}");
        return;
    }
    let judged_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
    let per = count.div_ceil(threads);
    type Out = (u64, u64, BTreeSet<&'static str>, BTreeSet<&'static str>, Option<String>, u64, u64);
    let outs: Vec<Out> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (writes, segments) = (&writes, &segments);
                scope.spawn(move || {
                    let mut out: Out = (0, 0, BTreeSet::new(), BTreeSet::new(), None, 0, 0);
                    let range = (t * per).min(count)..((t + 1) * per).min(count);
                    judge_layer0_state_range(base, writes, segments, judged_root_index, &[], &full_expansion, range, &mut |ordinal, image, judgement| {
                        let new = reds(&judge::kinds_of_new(&judgement.pool_checker));
                        let mutated = reds(&judge::mutant(mutant, image));
                        let (_, probe) = judge::probe(image);
                        out.0 += 1;
                        if new != mutated {
                            out.1 += 1;
                            if out.4.is_none() {
                                out.4 = Some(format!("ordinal={ordinal} new={new:?} {mutant}={mutated:?} versions={:?} next_mount={:?}", probe.versions.iter().map(|v| (v.0, v.1)).collect::<Vec<_>>(), probe.next_mount.as_ref().map(|v| (v.0, v.1))));
                            }
                        }
                        if !probe.versions.is_empty() {
                            out.5 += 1;
                        }
                        if probe.next_mount.is_some() {
                            out.6 += 1;
                        }
                        out.2.extend(new);
                        out.3.extend(mutated);
                    });
                    out
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("线程")).collect()
    });
    let mut total: Out = (0, 0, BTreeSet::new(), BTreeSet::new(), None, 0, 0);
    for o in outs {
        total.0 += o.0;
        total.1 += o.1;
        total.2.extend(o.2);
        total.3.extend(o.3);
        if total.4.is_none() {
            total.4 = o.4;
        }
        total.5 += o.5;
        total.6 += o.6;
    }
    assert_eq!(total.0, count, "判过的状态数等于状态数");
    println!(
        "B35F-DONE label={label} mutant={mutant} states={} states_with_versions_applied_only_by_records={} states_with_next_mount_version={} differing_states={} new_red_union={:?} {mutant}_red_union={:?}",
        total.0, total.5, total.6, total.1, total.2, total.3
    );
    if let Some(first) = total.4 {
        println!("B35F-FIRST label={label} {first}");
    }
}

/// 两张镜像逐盘逐 512 字节比：不同的扇区按落在哪一类结构上数（系统配置、根环、journal 环、单元区）。
pub fn image_difference(a: &MemoryPool, b: &MemoryPool, geometry: &singlefs_harness::segments::FixedGeometry) -> String {
    use singlefs_core::address::DeviceOffsetInBytes;
    let mut counts: std::collections::BTreeMap<String, u64> = Default::default();
    for (identity, device) in &a.devices {
        let other = &b.devices[identity];
        let mut sectors: BTreeSet<u64> = BTreeSet::new();
        for d in [device, other] {
            for sector in d.written_sectors().keys() {
                sectors.insert(*sector * 512);
            }
        }
        for offset in sectors {
            let x = device.read(DeviceOffsetInBytes(offset), 512);
            let y = other.read(DeviceOffsetInBytes(offset), 512);
            if x != y {
                let kind = geometry.classify(&singlefs_harness::RecordedOperation {
                    device: *identity,
                    kind: singlefs_harness::RecordedOperationKind::Write,
                    offset: DeviceOffsetInBytes(offset),
                    length: 512,
                    content_hash: 0,
                });
                *counts.entry(format!("dev{} {}", identity.0, kind.name())).or_default() += 1;
            }
        }
    }
    format!("{counts:?}")
}

/// 一步的录制流里拿掉 (实例, txg) 那条根的根槽写（照对拍流 `record_three_unit_file_applied_only_by_its_records_after_a_mount` 的做法）。
pub fn without_root_write(ops: &[RetainedOperation], geometry: &singlefs_harness::segments::FixedGeometry, instance: u32, txg: u64) -> Vec<RetainedOperation> {
    ops.iter()
        .filter(|o| {
            !(geometry.classify(&o.operation) == StepKind::RootRecordFua
                && o.contents.as_ref().is_some_and(|b| u32::from_le_bytes(b[24..28].try_into().expect("4")) == instance && u64::from_le_bytes(b[28..36].try_into().expect("8")) == txg))
        })
        .cloned()
        .collect()
}

/// 改一条录制流里某几条 journal 记录写的内容（`change` 收整条 4096、改完重封；同一次发布里后面的记录按改过的头重链）。
pub fn rewrite_record_writes(ops: &mut [RetainedOperation], geometry: &singlefs_harness::segments::FixedGeometry, pick: impl Fn(&[u8]) -> bool, change: impl Fn(&mut [u8])) -> usize {
    use singlefs_core::journal::{back_chain_of, JournalRecord};
    let mut changed = 0;
    // 按录制次序：改过的那条的新头决定同实例计数器大 1 的那条的反向链（两块盘各一份，内容相同）。
    let mut new_head_of: std::collections::BTreeMap<(u32, u64), Vec<u8>> = Default::default();
    for o in ops.iter_mut() {
        if geometry.classify(&o.operation) != StepKind::JournalRecord {
            continue;
        }
        let bytes = o.contents.as_mut().expect("内容保留");
        let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
        let Some(mut record) = JournalRecord::parse(bytes, fsid) else { continue };
        let key = (record.instance.0, record.counter);
        let mut touched = false;
        if let Some(previous) = new_head_of.get(&(key.0, key.1 - 1)) {
            record.back_chain = back_chain_of(previous);
            let rewritten = record.to_bytes();
            if rewritten != *bytes {
                bytes.copy_from_slice(&rewritten);
                touched = true;
            }
        }
        if pick(bytes) {
            change(bytes);
            craft::reseal_record(bytes);
            touched = true;
            changed += 1;
        }
        if touched {
            new_head_of.insert(key, bytes.clone());
        }
    }
    changed
}

pub fn run() {
    let w = HistoryDeviceWidth::FourGibibytes;
    let threads: u64 = std::env::args().nth(2).and_then(|v| v.parse().ok()).unwrap_or(1);
    let txg_of = |b: &[u8]| u64::from_le_bytes(b[26..34].try_into().expect("8"));
    // ── A：下一次挂载先施加的那一版（三条记录的发布 B，崩在 B 的根槽之前）。s3 = B 的分配记录叶里两条记录罩住同一个槽 + 第一条记录写成上一版的新根段。
    let mut d = history(w, &[Op::Mount, Op::FirstFile(5)]);
    let ring = d.params.geometry.journal_ring_bytes;
    let geometry = d.geometry();
    let (before_b, ops_b, root_b) = step(&mut d, Op::SequentialWrite(1));
    let crash = image_with(&before_b, &ops_b, &(0..root_b).collect::<Vec<_>>());
    let a_txg = root_txgs(&crash)[0];
    let b_txg = a_txg + 1;
    println!("B35X A a_txg={a_txg} b_txg={b_txg} records_of_b={:?}", records_of(&crash, ring, 1, b_txg));
    report("A-s0-clean-crash", w, &crash);
    let stale = segment_of_root_bytes(&root_bytes_of(&crash, a_txg));
    let mut s3 = crash.clone();
    println!("B35X A overlap leaf={:?} stale_rewrote={}", overlap_in_the_version(&mut s3, ring, b_txg), stale_first_record(&mut s3, ring, 1, b_txg, &stale));
    report("A-s3-overlap-and-stale-first", w, &s3);
    // s3 那三条记录写（两块盘各一份）的每个子集：记录那一段之前的全落（含改过的叶与补过的树），记录那一段取子集。
    {
        let slots: Vec<(u32, u64, u64)> = records_of(&s3, ring, 1, b_txg);
        let mut base = s3.clone();
        for (device, _, offset) in &slots {
            write(&mut base, *device, *offset, &read(&before_b, *device, *offset, 4096));
        }
        let mut differing = 0;
        let mut both = Vec::new();
        for mask in 0u32..(1 << slots.len()) {
            let mut image = base.clone();
            for (i, (device, _, offset)) in slots.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    write(&mut image, *device, *offset, &read(&s3, *device, *offset, 4096));
                }
            }
            let new = reds(&judge::new(&image));
            let mutn = reds(&judge::mutant("mutn", &image));
            let pre = reds(&judge::pre35(&image).0);
            if new != mutn {
                differing += 1;
            }
            both.push(format!("{mask:06b}:new={new:?}/mutn={mutn:?}/pre35={pre:?}"));
        }
        println!("B35X A-subsets slots={:?} states={} new!=mutn={differing}", slots.iter().map(|s| (s.0, s.1)).collect::<Vec<_>>(), 1u32 << slots.len());
        for line in both {
            if line.contains("I-") {
                println!("B35X   A-subset {line}");
            }
        }
    }

    // ── B：R_DATA 那条对拍流的形：B 由记录施加出来（挂载之后），第一条记录写成上一版的新根段。合法造法 = 崩在 B 的根槽之前、新进程挂载、覆盖写；
    // 对拍流的造法 = 录制时 B 的根落了、挂载读到 B 的根、覆盖写，之后从流里拿掉 B 的根槽写。两份末态逐扇区比、各判一遍，再各展开挂载与覆盖写那一段。
    for stale_first in [false, true] {
        let tag = if stale_first { "stale-first" } else { "clean" };
        let mut legal_base = crash.clone();
        if stale_first {
            stale_first_record(&mut legal_base, ring, 1, b_txg, &stale);
        }
        let mut legal = Driver::from_image(w, &legal_base);
        let start = legal.stream.operation_count();
        let outcomes: Vec<String> = [Op::Mount, Op::Overwrite(21)].iter().map(|op| legal.execute(*op)).collect();
        let legal_ops = legal.ops_since(start);
        let legal_end = legal.image();
        // 对拍流的造法：B 的根照常落（录制），重开挂载、覆盖写，再拿掉 B 的根槽写。
        let mut ops_with_root: Vec<RetainedOperation> = ops_b.clone();
        let mut root_written = before_b.clone();
        root_written.apply(&ops_with_root);
        let mut reopened = Driver::from_image(w, &root_written);
        let start = reopened.stream.operation_count();
        let outcomes2: Vec<String> = [Op::Mount, Op::Overwrite(21)].iter().map(|op| reopened.execute(*op)).collect();
        let constructed_suffix = reopened.ops_since(start);
        let mut b_without_root = without_root_write(&ops_with_root, &geometry, 1, b_txg);
        if stale_first {
            let first_counter = records_of(&crash, ring, 1, b_txg).iter().map(|r| r.1).min().expect("有").to_le_bytes();
            let n = rewrite_record_writes(&mut b_without_root, &geometry, |b| u32::from_le_bytes(b[16..20].try_into().expect("4")) == 1 && txg_of(b) == b_txg && b[20..26] == first_counter[..6], |b| b[99..99 + 364].copy_from_slice(&stale));
            println!("B35X B-{tag} constructed: rewrote {n} record writes");
        }
        ops_with_root.clear();
        let mut constructed_base = before_b.clone();
        constructed_base.apply(&b_without_root);
        let mut constructed_end = constructed_base.clone();
        constructed_end.apply(&constructed_suffix);
        println!("B35X B-{tag} legal outcomes={outcomes:?} constructed outcomes(recorded with B's root)={outcomes2:?}");
        println!("B35X B-{tag} crash-image vs constructed-base differ: {}", image_difference(&legal_base, &constructed_base, &geometry));
        println!("B35X B-{tag} legal-end vs constructed-end differ: {}", image_difference(&legal_end, &constructed_end, &geometry));
        if std::env::var("B35_DUMP").as_deref() == Ok("1") {
            dump_difference(&format!("B-{tag} legal-end(a) vs constructed-end(b)"), &legal_end, &constructed_end, &geometry);
            dump_difference(&format!("B-{tag} crash(a) vs constructed-base(b)"), &legal_base, &constructed_base, &geometry);
        }
        report(&format!("B-{tag}-legal-end"), w, &legal_end);
        report(&format!("B-{tag}-constructed-end"), w, &constructed_end);
        enumerate_flow(&format!("B-{tag}-legal-mount-and-overwrite"), &legal_base, &legal_ops, &geometry, "mutr", threads);
        enumerate_flow(&format!("B-{tag}-constructed-mount-and-overwrite"), &constructed_base, &constructed_suffix, &geometry, "mutr", threads);
    }

    // ── C：Q3 / Q1 全零树表指针。C1 新池新建文件那次发布的根与记录树表指针全零（对拍流 new-pool-publish-tree-table-pointer-all-zero 的形）；
    // C2 B 由记录施加出来、三条记录树表指针全零（对拍流 three-unit-file-applied-only-by-its-records-all-zero-tree-table-pointer 的形，合法造法）。
    {
        let mut d1 = history(w, &[Op::Mount]);
        let (before, mut ops, _) = step(&mut d1, Op::FirstFile(5));
        let n = rewrite_record_writes(&mut ops, &geometry, |_| true, |b| b[99..185].fill(0));
        for o in ops.iter_mut() {
            if geometry.classify(&o.operation) == StepKind::RootRecordFua {
                let b = o.contents.as_mut().expect("内容");
                b[36..122].fill(0);
                craft::reseal_root(b);
            }
        }
        let mut end = before.clone();
        end.apply(&ops);
        println!("B35X C1 rewrote {n} record writes");
        let root_index = ops.iter().rposition(|o| geometry.classify(&o.operation) == StepKind::RootRecordFua).expect("根");
        report("C1-new-pool-records-all-zero-before-root", w, &image_with(&before, &ops, &(0..root_index).collect::<Vec<_>>()));
        report("C1-new-pool-root-and-records-all-zero-end", w, &end);
        enumerate_flow("C1-new-pool-root-and-records-all-zero", &before, &ops, &geometry, "mutz", threads);
    }
    {
        let mut c2_base = crash.clone();
        let all = records_of(&crash, ring, 1, b_txg);
        rewrite_records(&mut c2_base, ring, &all, |b| b[99..185].fill(0));
        report("C2-crash-next-mount-version-all-zero", w, &c2_base);
        let mut legal = Driver::from_image(w, &c2_base);
        let start = legal.stream.operation_count();
        let outcome = legal.execute(Op::Mount);
        println!("B35X C2 mount over all-zero records → {}", outcome.chars().take(160).collect::<String>());
        let _ = legal.ops_since(start);
        // 挂载拒了：「由记录施加出来」那一形只能先干净挂载、再改记录（与对拍流 / 坏镜像同法：挂载之后改盘上的记录）。
        let mut clean = Driver::from_image(w, &crash);
        let start = clean.stream.operation_count();
        for op in [Op::Mount, Op::Overwrite(21)] {
            clean.execute(op);
        }
        let mut suffix = clean.ops_since(start);
        let mut z5 = clean.image();
        let z5_records = records_of(&z5, ring, 1, b_txg);
        rewrite_records(&mut z5, ring, &z5_records, |b| b[99..185].fill(0));
        report("C2-applied-only-by-records-all-zero-end", w, &z5);
        // 流的形：基镜像里 B 的三条记录就已全零（挂载那一刻读到的是干净的，录制流不变）——对拍流同样是录制之后改记录写。
        let n = rewrite_record_writes(&mut suffix, &geometry, |b| txg_of(b) == b_txg, |b| b[99..185].fill(0));
        println!("B35X C2 suffix record writes of B rewritten={n}");
        enumerate_flow("C2-applied-only-by-records-all-zero(base records zeroed)", &c2_base, &suffix, &geometry, "mutz", threads);
    }

    // ── D：重放施加不止一次发布时，下一次挂载先施加的那一版全零而最后那一版完好（坏镜像：抹掉 txg+1 那条根，让两次发布都只在记录里）。
    {
        let mut d7 = history(w, &[Op::Mount, Op::FirstFile(5), Op::Overwrite(7)]);
        let (before, ops, root) = step(&mut d7, Op::Overwrite(9));
        let image = image_with(&before, &ops, &(0..root).collect::<Vec<_>>());
        let txgs = root_txgs(&image);
        let mut z7 = image.clone();
        for (device, offset) in craft::root_positions(&z7) {
            let bytes = read(&z7, device, offset, 512);
            if &bytes[..4] != [0u8; 4] && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == txgs[0] {
                write(&mut z7, device, offset, &[0u8; 512]);
            }
        }
        report("D-two-publishes-above-the-root-clean", w, &z7);
        let mut z7z = z7.clone();
        rewrite_records(&mut z7z, ring, &records_of(&z7, ring, 1, txgs[0]), |b| b[99..185].fill(0));
        report("D-two-publishes-first-all-zero", w, &z7z);
    }

    // ── E：末条落了、首条没落的发布之后挂载（恢复断号、不施加），挂载那几次发布的切口上新 checker 认不认它、判它什么。
    for (keep, after) in [("records-2..", &[Op::Overwrite(31), Op::SequentialWrite(7)][..]), ("record-", &[Op::Overwrite(31)][..]), ("records-1..1", &[Op::Overwrite(31)][..])] {
        last_records_without_the_first_then_mount(w, &[Op::Mount, Op::FirstFile(5)], 1, keep, after);
        last_records_without_the_first_then_mount(HistoryDeviceWidth::UnitAreaOf256Slots, &[Op::Mount, Op::FirstFile(5)], 1, keep, after);
    }
    // ── Q4：从 mkfs 水位第一次建树而没有数据单元（空的第一个文件）。
    let q4 = history(w, &[Op::Mount, Op::FirstFile(u64::MAX)]);
    report("Q4-empty-first-file", w, &q4.image());
    let _ = cuts;
}

/// 两张镜像不同的扇区逐个列出：(盘, 偏移, 种类, 扇区内不同字节的区间)；journal 记录另把两边解出来的记录头字段并排打出来。
pub fn dump_difference(label: &str, a: &MemoryPool, b: &MemoryPool, geometry: &singlefs_harness::segments::FixedGeometry) {
    use singlefs_core::address::DeviceOffsetInBytes;
    let mut journal_offsets = BTreeSet::new();
    for (identity, device) in &a.devices {
        let other = &b.devices[identity];
        let mut sectors: BTreeSet<u64> = BTreeSet::new();
        for d in [device, other] {
            sectors.extend(d.written_sectors().keys().map(|s| *s * 512));
        }
        for offset in sectors {
            let x = device.read(DeviceOffsetInBytes(offset), 512);
            let y = other.read(DeviceOffsetInBytes(offset), 512);
            if x == y {
                continue;
            }
            let kind = geometry.classify(&singlefs_harness::RecordedOperation {
                device: *identity,
                kind: singlefs_harness::RecordedOperationKind::Write,
                offset: DeviceOffsetInBytes(offset),
                length: 512,
                content_hash: 0,
            });
            let mut ranges: Vec<(usize, usize)> = Vec::new();
            for i in 0..512 {
                if x[i] != y[i] {
                    match ranges.last_mut() {
                        Some(r) if r.1 + 4 >= i => r.1 = i,
                        _ => ranges.push((i, i)),
                    }
                }
            }
            println!("B35D {label} dev{} offset={offset} kind={} ranges={ranges:?} a={:02x?} b={:02x?}", identity.0, kind.name(),
                ranges.iter().take(3).map(|r| x[r.0..=r.1.min(r.0 + 15)].to_vec()).collect::<Vec<_>>(),
                ranges.iter().take(3).map(|r| y[r.0..=r.1.min(r.0 + 15)].to_vec()).collect::<Vec<_>>());
            if kind == StepKind::JournalRecord {
                journal_offsets.insert((identity.0, offset - (offset - singlefs_format::JOURNAL_RING_START_SLOT * SLOT) % 4096));
            }
        }
    }
    for (device, offset) in journal_offsets {
        for (side, image) in [("a", a), ("b", b)] {
            let bytes = read(image, device, offset, 4096);
            let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
            let parsed = singlefs_core::journal::JournalRecord::parse(&bytes, fsid);
            println!("B35D {label} journal dev{device} offset={offset} {side}={:?}", parsed.map(|r| (r.instance, r.counter, r.checkpoint_txg, r.transaction, r.is_commit, r.place_in_publish, r.back_chain, r.named.len())));
        }
    }
}

/// E：一次多条记录的发布只落了末条那几条（首条没落，层 0 同一段里的子集），新进程挂载（恢复断号、不施加它），挂载那几次发布的每个切口与之后各判一遍。
/// 交回几张镜像各自的判定摘要；`sequential` 选顺序写的内容（切几条记录随它）。
pub fn last_records_without_the_first_then_mount(width: HistoryDeviceWidth, prefix: &[Op], sequential: u64, keep: &str, after: &[Op]) {
    let mut d = history(width, prefix);
    let geometry = d.geometry();
    let ring = d.params.geometry.journal_ring_bytes;
    let (before, ops, _) = step(&mut d, Op::SequentialWrite(sequential));
    let chosen = cuts(&d, &ops).into_iter().filter(|(name, _, _)| name.starts_with(keep)).last();
    let Some((name, _, chosen)) = chosen else {
        println!("B35E width={width:?} sequential={sequential} keep={keep}: 这一步没有这种切口");
        return;
    };
    let crash = image_with(&before, &ops, &chosen);
    let newest = root_txgs(&crash)[0];
    println!("B35E width={width:?} sequential={sequential} cut={name} newest_root_txg={newest} records_of_next={:?}", records_of(&crash, ring, d.session.as_ref().map_or(0, |s| s.instance.0), newest + 1));
    report(&format!("E-{keep}-crash"), width, &crash);
    let mut m = Driver::from_image(width, &crash);
    let mut ledger = crate::tally::Ledger::new();
    let mut tally = crate::tally::Tally::default();
    for op in std::iter::once(&Op::Mount).chain(after.iter()) {
        let before = m.image();
        let start = m.stream.operation_count();
        let outcome = crate::random::execute_noting_the_ledger(&mut m, *op, &mut ledger);
        let ops = m.ops_since(start);
        for (cut, _, chosen) in cuts(&m, &ops) {
            let label = format!("E-{keep} {op:?} cut={cut}");
            let image = image_with(&before, &ops, &chosen);
            if *op == Op::Mount {
                // 那次没施加的发布（txg newest + 1）自己写的单元：哪些槽在这一切口上已被新实例盖掉。
                let mut overwritten = Vec::new();
                for (slot, length) in craft::units(&crash) {
                    let old = read(&crash, 0, slot * SLOT, length);
                    let birth_offset = match old[6] { 1 => 75, 3 => 73, _ => 52 + 2 * usize::from(old[51]) };
                    if u64::from_le_bytes(old[birth_offset..birth_offset + 8].try_into().expect("8")) != newest + 1 {
                        continue;
                    }
                    if read(&image, 0, slot * SLOT, length) != old {
                        overwritten.push((old[6], slot));
                    }
                }
                let new = judge::new(&image);
                println!("B35E   {label} unapplied-publish units overwritten={overwritten:?} I-3.10={}[{}] I-5.4={}[{}] reds={:?}", new["I-3.10"].0, new["I-3.10"].1.chars().take(160).collect::<String>(), new["I-5.4"].0, new["I-5.4"].1.chars().take(100).collect::<String>(), reds(&new));
            }
            crate::tally::note_image(&mut tally, &label, &image, None, &ledger);
        }
        crate::tally::note_image(&mut tally, &format!("E-{keep} {op:?} after"), &m.image(), None, &ledger);
        println!("B35E {op:?} → {}", outcome.chars().take(120).collect::<String>());
    }
    tally.print(&format!("B35E-T {keep} seq={sequential}"));
    let _ = geometry;
}
