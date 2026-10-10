//! 造出来的几张镜像（P1 取哪条记录的新根段、P2 全零树表指针、P4 Q1 / Q2 / Q4）：新旧两份 checker 的判定，与 core 两种挂载的结局。
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::MemoryPool;

use crate::craft::{self, read, write, SLOT};
use crate::drive::{Driver, Op};
use crate::judge::compare;
use crate::random::{image_with_prefix, root_writes_with_first_record};

const SHOWN: [&str; 9] = ["I-2.1", "I-2.5", "I-3.1", "I-3.10", "I-5.4", "I-5.5", "I-7.2", "I-7.4", "I-9.28"];

/// 一张镜像：新旧 checker 判红的、几条要看的不变量新旧两边的种类与原文头、core 只读恢复与可写挂载的结局。
pub fn report(label: &str, width: HistoryDeviceWidth, image: &MemoryPool) {
    let compared = compare(image);
    let old_red: Vec<&str> = compared.old.iter().filter(|(_, k)| k.0 == 'V').map(|(n, _)| *n).collect();
    println!("B34X label={label} new_red={:?} old_red={old_red:?} rv={:?} next_mount={}", compared.new_violations(), compared.versions_applied_only_by_records, compared.next_mount_version);
    for name in SHOWN {
        let (new, old) = (&compared.new[name], &compared.old[name]);
        println!(
            "B34X   {label} {name} new={}[{}] old={}[{}]",
            new.0,
            new.1.chars().take(160).collect::<String>(),
            old.0,
            old.1.chars().take(80).collect::<String>()
        );
    }
    let fl = crate::judge::fl_kinds(image);
    let fl_red: Vec<&str> = fl.iter().filter(|(_, k)| k.0 == 'V').map(|(n, _)| *n).collect();
    println!(
        "B34X   {label} FL red={fl_red:?} I-5.4={}[{}]",
        fl["I-5.4"].0,
        fl["I-5.4"].1.chars().take(160).collect::<String>()
    );
    let recovered = recover(image, JournalPolicy::Consult);
    let recovered_text = match &recovered.outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile {root:?}"),
        RecoveryOutcome::FileRead { root, content } => format!("FileRead {root:?} bytes={}", content.len()),
        RecoveryOutcome::Failed { root, failure } => format!("Failed {root:?} {failure:?}"),
    };
    let mut driver = Driver::from_image(width, image);
    let mounted = driver.execute(Op::Mount);
    println!(
        "B34X mount label={label} recover={} writable={}",
        recovered_text.chars().take(200).collect::<String>(),
        mounted.chars().take(200).collect::<String>()
    );
}

/// 跑一串用户动作（不崩）；交回驱动。
pub fn history(width: HistoryDeviceWidth, ops: &[Op]) -> Driver {
    let mut driver = Driver::mkfs(width);
    for op in ops {
        let outcome = driver.execute(*op);
        assert!(outcome.starts_with("ok"), "{op:?} → {outcome}");
    }
    driver
}

/// 做一步、崩在它最后一次根槽写之前：交回崩溃镜像（那一步的单元与记录都落了、根没落）。
pub fn crash_before_the_last_root(driver: &mut Driver, op: Op) -> MemoryPool {
    let before = driver.image();
    let start = driver.stream.operation_count();
    let outcome = driver.execute(op);
    assert!(outcome.starts_with("ok"), "{op:?} → {outcome}");
    let ops = driver.ops_since(start);
    let roots = root_writes_with_first_record(driver, &ops);
    let (root_index, _) = *roots.last().expect("这一步写了根");
    image_with_prefix(&before, &ops, root_index)
}

/// 根槽里读得出的根的 txg（两盘），从新到旧。
pub fn root_txgs(image: &MemoryPool) -> Vec<u64> {
    crate::drive::ring_roots_newest_first(image).into_iter().map(|(txg, _)| txg).collect()
}

/// 环里 (实例, txg) 的那几条记录（每块盘）：(盘, 计数器, 偏移)。
pub fn records_of(image: &MemoryPool, ring_bytes: u64, txg: u64) -> Vec<(u32, u64, u64)> {
    let mut out = Vec::new();
    for device in craft::DEVICES {
        for (counter, offset, bytes) in craft::records(image, device, ring_bytes) {
            if u64::from_le_bytes(bytes[26..34].try_into().expect("8")) == txg {
                out.push((device, counter, offset));
            }
        }
    }
    out
}

/// 改几条记录的字节（`change` 收整条 4096），重封、重链。
pub fn rewrite_records(image: &mut MemoryPool, ring_bytes: u64, which: &[(u32, u64, u64)], change: impl Fn(&mut [u8])) {
    let mut changed = std::collections::BTreeSet::new();
    for (device, counter, offset) in which {
        let mut bytes = read(image, *device, *offset, 4096);
        change(&mut bytes);
        craft::reseal_record(&mut bytes);
        write(image, *device, *offset, &bytes);
        changed.insert((*device, *counter));
    }
    craft::rechain(image, ring_bytes, changed);
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对同盘首尾相接的记录，前一条跨度加一：两条罩住同一个槽。改了交回 true。
/// 偏移照 `checker_known_bad_images.rs`：条数 102、条目起点 135、一条 20 字节、跨度在条目 10 起 2 字节（最高位是已释放标志）。
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

/// 那一版（单元头诞生代号 = txg）自己写的第一片改得动的分配记录叶：两块盘那一份都改、重封、沿引用链补到记录。交回叶的槽号。
pub fn overlap_in_the_version(image: &mut MemoryPool, ring_bytes: u64, txg: u64) -> Option<u64> {
    for (slot, length) in craft::units(image) {
        let bytes = read(image, 0, slot * SLOT, length);
        if bytes[6] != 2 || bytes[51] != 10 || u64::from_le_bytes(bytes[72..80].try_into().expect("8")) != txg {
            continue;
        }
        let mut probe = bytes.clone();
        if !widen_one_record(&mut probe) {
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

/// 把 (实例, txg) 那次发布的第一条记录（两盘各一份）的新根段换成 `stale_segment`（364 字节），重封、重链。
pub fn stale_first_record(image: &mut MemoryPool, ring_bytes: u64, txg: u64, stale_segment: &[u8]) -> usize {
    let all = records_of(image, ring_bytes, txg);
    let first_counter = all.iter().map(|(_, counter, _)| *counter).min().expect("有记录");
    let firsts: Vec<_> = all.into_iter().filter(|(_, counter, _)| *counter == first_counter).collect();
    rewrite_records(image, ring_bytes, &firsts, |bytes| {
        bytes[craft::SEGMENT_OFFSET_IN_RECORD..craft::SEGMENT_OFFSET_IN_RECORD + 364].copy_from_slice(stale_segment);
    });
    firsts.len()
}

/// 新根段（364 字节）照一条根记录的七段拼出来。
pub fn segment_of_root(root_bytes: &[u8]) -> Vec<u8> {
    let mut segment = vec![0u8; 364];
    for (s, r) in craft::SEGMENT_FROM_ROOT.iter().cloned() {
        segment[s].copy_from_slice(&root_bytes[r]);
    }
    segment
}

/// txg 那条根的根记录字节（两盘里先找到的那一份）。
pub fn root_bytes_of(image: &MemoryPool, txg: u64) -> Vec<u8> {
    craft::root_positions(image)
        .into_iter()
        .map(|(device, offset)| read(image, device, offset, 512))
        .find(|bytes| &bytes[..4] != [0u8; 4] && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == txg)
        .expect("那条根在根环里")
}

pub fn run() {
    let w = HistoryDeviceWidth::FourGibibytes;
    // P1：一次发布切成三条记录（顺序写三个数据单元），崩在根槽之前：下一次挂载先施加的那一版只在三条记录里。
    let mut d = history(w, &[Op::Mount, Op::FirstFile(5)]);
    let ring = d.params.geometry.journal_ring_bytes;
    let crash = crash_before_the_last_root(&mut d, Op::SequentialWrite(1));
    let newest = root_txgs(&crash)[0];
    let v = newest + 1;
    println!("B34X P1 newest_root_txg={newest} version_txg={v} records_of_the_version={:?}", records_of(&crash, ring, v));
    report("s0-clean-next-mount", w, &crash);
    let stale = segment_of_root(&root_bytes_of(&crash, newest));
    let mut a = crash.clone();
    println!("B34X stale_first_record rewrote={}", stale_first_record(&mut a, ring, v, &stale));
    report("s1-stale-first-record", w, &a);
    let mut b = crash.clone();
    println!("B34X overlap leaf={:?}", overlap_in_the_version(&mut b, ring, v));
    report("s2-overlap", w, &b);
    let mut c = b.clone();
    println!("B34X stale_first_record rewrote={}", stale_first_record(&mut c, ring, v, &stale));
    report("s3-overlap-and-stale-first-record", w, &c);
    // 只改第一条记录的新根段之后照常可写挂载：那一版成了「由记录施加出来的那一版」，再发一次覆盖写之后判。
    let mut after = Driver::from_image(w, &a);
    for op in [Op::Mount, Op::Overwrite(21)] {
        println!("B34X s4 {op:?} → {}", after.execute(op));
    }
    report("s4-stale-first-record-then-mount-and-overwrite", w, &after.image());

    // P2：全零树表指针。z0 干净；z1 最新根；z2 / z3 更早的候选根。
    let d = history(w, &[Op::Mount, Op::FirstFile(5), Op::Overwrite(7), Op::Overwrite(9)]);
    let base = d.image();
    let txgs = root_txgs(&base);
    println!("B34X P2 root_txgs={txgs:?}");
    report("z0-clean", w, &base);
    for (label, txg) in [("z1-newest-root-zero", txgs[0]), ("z2-older-root-zero", txgs[1]), ("z3-first-file-root-zero", txgs[2])] {
        let mut image = base.clone();
        println!("B34X {label} txg={txg} roots_rewritten={}", craft::rewrite_root(&mut image, ring, txg, |bytes| bytes[36..122].fill(0)));
        report(label, w, &image);
    }
    // P2：z2 那张镜像上挂载之后回退到树表指针全零的那条根（core 读这条根的另一条路）。
    {
        let mut image = base.clone();
        craft::rewrite_root(&mut image, ring, txgs[1], |bytes| bytes[36..122].fill(0));
        let mut rolled = Driver::from_image(w, &image);
        for op in [Op::Mount, Op::RollbackToTxg(txgs[1])] {
            println!("B34X z2-rollback {op:?} → {}", rolled.execute(op).chars().take(200).collect::<String>());
        }
    }
    // P2：F 之下的根树表指针全零（抬 F 之后）。
    let d4 = history(w, &[Op::Mount, Op::FirstFile(5), Op::Overwrite(7), Op::Overwrite(9), Op::Overwrite(11), Op::RaiseFloor(3)]);
    let base4 = d4.image();
    let txgs4 = root_txgs(&base4);
    let floor = u64::from_le_bytes(root_bytes_of(&base4, txgs4[0])[130..138].try_into().expect("8"));
    println!("B34X z4 root_txgs={txgs4:?} newest_root_floor={floor}");
    if let Some(below) = txgs4.iter().copied().find(|txg| *txg < floor && *txg > 0) {
        let mut image = base4.clone();
        craft::rewrite_root(&mut image, ring, below, |bytes| bytes[36..122].fill(0));
        report(&format!("z4-root-below-floor-zero-txg{below}"), w, &image);
    }
    // Q1：下一次挂载先施加的那一版（记录里）树表指针全零。
    let mut d6 = history(w, &[Op::Mount, Op::FirstFile(5), Op::Overwrite(7)]);
    let crash6 = crash_before_the_last_root(&mut d6, Op::Overwrite(11));
    let v6 = root_txgs(&crash6)[0] + 1;
    let mut z6 = crash6.clone();
    rewrite_records(&mut z6, ring, &records_of(&crash6, ring, v6), |bytes| bytes[99..185].fill(0));
    report("z6-next-mount-version-zero", w, &z6);
    // Q1：由记录施加出来的那一版（挂载之后）树表指针全零。
    let mut d5 = history(w, &[Op::Mount, Op::FirstFile(5), Op::Overwrite(7)]);
    let crash5 = crash_before_the_last_root(&mut d5, Op::Overwrite(13));
    let v5 = root_txgs(&crash5)[0] + 1;
    let mut after5 = Driver::from_image(w, &crash5);
    for op in [Op::Mount, Op::Overwrite(15)] {
        println!("B34X z5 {op:?} → {}", after5.execute(op));
    }
    let clean5 = after5.image();
    report("z5-clean-version-applied-only-by-records", w, &clean5);
    let mut z5 = clean5.clone();
    rewrite_records(&mut z5, ring, &records_of(&clean5, ring, v5), |bytes| bytes[99..185].fill(0));
    report("z5-version-applied-only-by-records-zero", w, &z5);
    // Q4：第一个文件是空的（从 mkfs 水位第一次建树、没有数据单元）。
    let q4 = history(w, &[Op::Mount, Op::FirstFile(u64::MAX)]);
    report("q4-empty-first-file", w, &q4.image());
    let q4b = history(w, &[Op::Mount, Op::FirstFile(5)]);
    report("q4-control-first-file", w, &q4b.image());
}
