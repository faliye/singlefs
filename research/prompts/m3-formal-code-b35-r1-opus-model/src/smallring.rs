//! 退路（由记录施加出来的那一版「环里读不到带末条标志的那一条时退回先到的那一条」）在合法历史上走不走得到：
//! journal 环缩到 `R` 条记录（mkfs 收：在飞上限 ⌊R ÷ 3⌋ ≥ 1），B（三条记录的顺序写）崩在根槽之前、新进程挂载施加 B；
//! 之后一条记录一次的覆盖写把计数器推到与 B 的第一条同槽，再发一次多条记录的顺序写，让它的记录那一段同时盖 B 的首条与末条的槽；
//! 那一步按层 0 全展开（`judge_layer0_state_range` + `full_expansion`），每个状态逐项记（`tally::note_image`，含与 core 当初施加的那一版比）。
use singlefs_checker_tier::crash::{full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::writes_and_segments;
use singlefs_harness::segments::StepKind;

use crate::crafted::{records_of, root_txgs};
use crate::drive::{Driver, Op};
use crate::judge::kinds_of_new;
use crate::random::{execute_noting_the_ledger, image_with};
use crate::tally::{note_image, Ledger, Tally};

fn counters_in_the_ring(image: &singlefs_harness::memory_pool::MemoryPool, ring: u64) -> Vec<(u32, u64, u64, bool)> {
    let mut out = Vec::new();
    for slot in 0..ring / 4096 {
        let offset = singlefs_format::JOURNAL_RING_START_SLOT * crate::craft::SLOT + slot * 4096;
        let bytes = crate::craft::read(image, 0, offset, 4096);
        let fsid = u64::from_le_bytes(bytes[463..471].try_into().expect("8"));
        if let Some(r) = singlefs_core::journal::JournalRecord::parse(&bytes, fsid) {
            out.push((r.instance.0, r.counter, r.checkpoint_txg.0, r.place_in_publish == singlefs_core::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish));
        }
    }
    out.sort_by_key(|r| r.1);
    out
}

pub fn run() {
    let ring_records: u64 = std::env::args().nth(2).and_then(|v| v.parse().ok()).unwrap_or(15);
    let threads: u64 = std::env::args().nth(3).and_then(|v| v.parse().ok()).unwrap_or(1);
    let w = HistoryDeviceWidth::FourGibibytes;
    let mut params = w.parameters();
    params.geometry.journal_ring_bytes = ring_records * 4096;
    // 第 5 个参数给了就把根环的 S 换成它（格式许 4..16；不给 = mkfs 第一版的 8，三个区域共 24 条根）。
    if let Some(s) = std::env::args().nth(5).and_then(|v| v.parse::<u64>().ok()) {
        params.geometry.root_ring_slots_per_region = singlefs_core::root_ring::RootRingSlotsPerRegion::from_system_configuration_field(s).expect("S 在区间里");
    }
    let big: u64 = std::env::args().nth(4).and_then(|v| v.parse().ok()).unwrap_or(150);
    let ring = params.geometry.journal_ring_bytes;
    let mut ledger = Ledger::new();
    let mut d = Driver::mkfs_with(w, params.clone());
    for op in [Op::Mount, Op::FirstFile(5)] {
        println!("B35S {op:?} → {}", d.execute(op));
    }
    let before = d.image();
    let start = d.stream.operation_count();
    println!("B35S SequentialWrite(1) → {}", d.execute(Op::SequentialWrite(1)));
    let ops = d.ops_since(start);
    let geometry = d.geometry();
    let root = ops.iter().rposition(|o| geometry.classify(&o.operation) == StepKind::RootRecordFua).expect("根");
    let crash = image_with(&before, &ops, &(0..root).collect::<Vec<_>>());
    let b_txg = root_txgs(&crash)[0] + 1;
    let b_records = records_of(&crash, ring, 1, b_txg);
    let c1 = b_records.iter().map(|r| r.1).min().expect("B 有记录");
    let ck = b_records.iter().map(|r| r.1).max().expect("B 有记录");
    println!("B35S ring_records={ring_records} b_txg={b_txg} b_counters={c1}..={ck} ring={:?}", counters_in_the_ring(&crash, ring));
    let mut d = Driver::from_image_with(w, params.clone(), &crash);
    println!("B35S Mount → {}", execute_noting_the_ledger(&mut d, Op::Mount, &mut ledger));
    println!("B35S ledger={:?}", ledger.keys().collect::<Vec<_>>());
    // 推计数器：先用大写一次推很多条，离 c1 + R 不到 2 × 一次大写的条数时改用一条记录一次的覆盖写，下一条记录的计数器 n 等于 c1 + R 时停。
    let mut selector = 100;
    let mut big_step = 0;
    loop {
        let next = counters_in_the_ring(&d.image(), ring).iter().map(|r| r.1).max().unwrap_or(0) + 1;
        if next == c1 + ring_records {
            break;
        }
        if next > c1 + ring_records {
            println!("B35S 推过头了：next={next} 目标 {}", c1 + ring_records);
            return;
        }
        selector += 1;
        let remaining = c1 + ring_records - next;
        let op = if remaining >= 2 { Op::BigWrite(remaining.min(big)) } else { Op::Overwrite(selector) };
        let outcome = d.execute(op);
        let after = counters_in_the_ring(&d.image(), ring).iter().map(|r| r.1).max().unwrap_or(0) + 1;
        big_step = big_step.max(after - next);
        println!("B35S {op:?} → {} next {next}→{after} roots={:?}", outcome.chars().take(40).collect::<String>(), root_txgs(&d.image()).len());
        if !outcome.starts_with("ok") {
            println!("B35S overwrite refused: {outcome}");
            return;
        }
    }
    let base = d.image();
    {
        let mut tally = Tally::default();
        note_image(&mut tally, "before-the-lapping-step", &base, None, &ledger);
        tally.print("B35S-BASE");
    }
    println!("B35S before the lapping step ring={:?} roots={:?}", counters_in_the_ring(&base, ring), root_txgs(&base));
    let start = d.stream.operation_count();
    println!("B35S lapping BigWrite({}) → {}", ck - c1 + 1, d.execute(Op::BigWrite(ck - c1 + 1)));
    let ops = d.ops_since(start);
    let (writes, segments) = writes_and_segments(&ops, &geometry);
    let count = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    println!("B35S lapping step segments={:?} layer0_states={count} ring_after={:?}", segments.iter().map(Vec::len).collect::<Vec<_>>(), counters_in_the_ring(&d.image(), ring));
    if count > 1_000_000 {
        // 全展开超过约 10⁶，不跑；只取记录那一段的整写子集（更早的段全持久、记录那一段任取一个子集，层 0 枚举域里的状态，不含撕裂的那几种）。
        println!("B35S-SKIP full layer0 states={count}; 只判记录那一段的整写子集");
        let records: Vec<usize> = ops.iter().enumerate().filter(|(_, o)| geometry.classify(&o.operation) == StepKind::JournalRecord).map(|(i, _)| i).collect();
        let first = records[0];
        let mut tally = Tally::default();
        for mask in 0u32..(1 << records.len()) {
            let chosen: Vec<usize> = (0..first).chain(records.iter().enumerate().filter(|(k, _)| mask & (1 << k) != 0).map(|(_, i)| *i)).collect();
            let image = image_with(&base, &ops, &chosen);
            let (_, probe) = crate::judge::probe(&image);
            let fallback: Vec<_> = probe.from_last.iter().filter(|(_, last)| !last).map(|(k, _)| *k).collect();
            if !fallback.is_empty() {
                println!("B35S   mask={mask:0width$b} fallback={fallback:?} versions={:?} ring={:?}", probe.versions.iter().map(|v| (v.0, v.1)).collect::<Vec<_>>(), counters_in_the_ring(&image, ring).iter().filter(|r| r.0 == 1).collect::<Vec<_>>(), width = records.len());
            }
            note_image(&mut tally, &format!("lapping record-subset mask={mask:b}"), &image, None, &ledger);
        }
        tally.print("B35S-RECORDS");
    } else {
    let judged_root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
    let per = count.div_ceil(threads);
    let tallies: Vec<Tally> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (base, writes, segments, ledger) = (&base, &writes, &segments, &ledger);
                scope.spawn(move || {
                    let mut tally = Tally::default();
                    let range = (t * per).min(count)..((t + 1) * per).min(count);
                    judge_layer0_state_range(base, writes, segments, judged_root_index, &[], &full_expansion, range, &mut |ordinal, image, judgement| {
                        note_image(&mut tally, &format!("lapping ordinal={ordinal}"), image, Some(kinds_of_new(&judgement.pool_checker)), ledger);
                    });
                    tally
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("线程")).collect()
    });
    let mut total = Tally::default();
    for t in tallies {
        total.absorb(t);
    }
    total.print("B35S-T");
    }
    // 之后再推一圈：B 的记录全被盖掉之后，这一版 checker 还认不认、记账那几条红不红。
    for _ in 0..ring_records.div_ceil(big) {
        let outcome = d.execute(Op::BigWrite(big));
        if !outcome.starts_with("ok") {
            println!("B35S later overwrite refused: {outcome}");
            break;
        }
    }
    let mut tally = Tally::default();
    note_image(&mut tally, "after-one-more-lap", &d.image(), None, &ledger);
    tally.print("B35S-LATER");
}
