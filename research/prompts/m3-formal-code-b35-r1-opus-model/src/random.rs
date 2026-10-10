//! 随机历史：用户动作按会话状态抽（不写死后缀）；每一步按种子可能崩在它某一次根槽写之前的某一个切口上（记录全落、记录只落一部分），
//! 由下一步（同样随机抽）起新进程。每一步之后的镜像、与每一次根槽写之前各切口的镜像，都交几份 checker 与 core 那一侧逐项记。
//! 切口都在层 0 枚举域里：更早的段全持久，记录那一段取一个子集（同一次发布的几条记录同一段，段内任何子集都可达）。
use singlefs_harness::history::{HistoryDeviceWidth, SeededRandomSource};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;

use crate::drive::{Driver, Op};
use crate::tally::{note_image, Ledger, Tally};

pub const WIDTHS: [HistoryDeviceWidth; 3] = [
    HistoryDeviceWidth::UnitAreaOf256Slots,
    HistoryDeviceWidth::UnitAreaOf384Slots,
    HistoryDeviceWidth::FourGibibytes,
];

/// 按会话状态抽一步（与第 34 批攻方原型同一张表，顺序写的比重抬高：多条记录的发布要它）。
pub fn draw_op(state: u8, rng: &mut SeededRandomSource) -> Op {
    let selector = rng.below(1 << 20);
    match state {
        0 => Op::Mount,
        1 => match rng.below(5) {
            0 => Op::Unmount,
            _ => Op::FirstFile(selector),
        },
        3 => match rng.below(4) {
            0 => Op::Unmount,
            1 => Op::Rollback(rng.below(8)),
            _ => Op::CreateInode,
        },
        _ => match rng.below(16) {
            0..=3 => Op::Overwrite(selector),
            4..=6 => Op::SequentialWrite(selector),
            7 => Op::Unlink,
            8..=9 => Op::Rollback(rng.below(8)),
            10..=11 => Op::RaiseFloor(rng.below(64)),
            12..=13 => Op::Unmount,
            _ => Op::Overwrite(selector),
        },
    }
}

fn counter_of(retained: &RetainedOperation) -> u64 {
    let bytes = retained.contents.as_ref().expect("录制流开着内容保留");
    let mut counter = [0u8; 8];
    counter[..6].copy_from_slice(&bytes[20..26]);
    u64::from_le_bytes(counter)
}

/// 一步的录制流里每一次根槽写之前的切口：(名字, 根槽写下标, 要落的写的下标表)。
pub fn cuts(driver: &Driver, ops: &[RetainedOperation]) -> Vec<(String, usize, Vec<usize>)> {
    let geometry = driver.geometry();
    let mut out = Vec::new();
    let mut records_since_the_last_root: Vec<usize> = Vec::new();
    for (index, retained) in ops.iter().enumerate() {
        match geometry.classify(&retained.operation) {
            StepKind::JournalRecord => records_since_the_last_root.push(index),
            StepKind::RootRecordFua => {
                let all: Vec<usize> = (0..index).collect();
                out.push((format!("before-root@{index}"), index, all));
                if let Some(&first) = records_since_the_last_root.first() {
                    let mut counters: Vec<u64> = records_since_the_last_root.iter().map(|i| counter_of(&ops[*i])).collect();
                    counters.dedup();
                    let k = counters.len();
                    let with = |chosen: &[u64]| -> Vec<usize> {
                        (0..first)
                            .chain(records_since_the_last_root.iter().copied().filter(|i| chosen.contains(&counter_of(&ops[*i]))))
                            .collect()
                    };
                    if k == 1 {
                        out.push((format!("one-copy@{index}"), index, (0..=first).collect()));
                    } else {
                        for j in 1..k {
                            out.push((format!("records-1..{j}-of-{k}@{index}"), index, with(&counters[..j])));
                        }
                        out.push((format!("records-2..{k}-of-{k}@{index}"), index, with(&counters[1..])));
                        out.push((format!("record-{k}-only-of-{k}@{index}"), index, with(&counters[k - 1..])));
                    }
                }
                records_since_the_last_root.clear();
            }
            _ => {}
        }
    }
    out
}

pub fn image_with(before: &MemoryPool, ops: &[RetainedOperation], chosen: &[usize]) -> MemoryPool {
    let mut image = before.clone();
    for index in chosen {
        image.apply(&ops[*index..*index + 1]);
    }
    image
}

/// 做一步 Mount 之前照恢复算 core 施加出来的那一版，挂上了就记进账本。
pub fn execute_noting_the_ledger(driver: &mut Driver, op: Op, ledger: &mut Ledger) -> String {
    let applied = if op == Op::Mount && driver.session.is_none() {
        crate::core::core_view(&driver.image()).and_then(|view| view.applied)
    } else {
        None
    };
    let outcome = driver.execute(op);
    if std::env::var("B35_DEBUG_LEDGER").as_deref() == Ok("1") && op == Op::Mount {
        println!("B35-LEDGER mount outcome={} applied={:?}", outcome.chars().take(60).collect::<String>(), applied.as_ref().map(|a| (a.0, a.2)));
    }
    if outcome.starts_with("ok") {
        if let Some((key, segment, _)) = applied {
            ledger.insert(key, segment);
        }
    }
    outcome
}

/// 一段历史：`steps` 步，每步四分之一的机会崩在它的某个切口上。`tally` 给了就逐张记；交回驱动、随机源、账本与一行摘要。
pub fn play(seed: u64, steps: usize, mut tally: Option<&mut Tally>) -> (Driver, SeededRandomSource, Ledger, String) {
    let mut rng = SeededRandomSource::from_seed(seed);
    let width = WIDTHS[rng.below(WIDTHS.len() as u64) as usize];
    let mut driver = Driver::mkfs(width);
    let mut ledger = Ledger::new();
    let mut log: Vec<String> = Vec::new();
    for step in 0..steps {
        let op = draw_op(driver.state(), &mut rng);
        let crash = rng.below(4) == 0;
        let crash_choice = rng.below(1 << 16);
        let before = driver.image();
        let start = driver.stream.operation_count();
        let outcome = execute_noting_the_ledger(&mut driver, op, &mut ledger);
        let ops = driver.ops_since(start);
        let step_cuts = cuts(&driver, &ops);
        if let Some(tally) = tally.as_deref_mut() {
            for (name, _, chosen) in &step_cuts {
                let label = format!("seed={seed} step={step} op={op:?} cut={name}");
                note_image(tally, &label, &image_with(&before, &ops, chosen), None, &ledger);
            }
        }
        let short = outcome.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
        if crash && !step_cuts.is_empty() {
            let (name, _, chosen) = &step_cuts[(crash_choice % step_cuts.len() as u64) as usize];
            driver = Driver::from_image(width, &image_with(&before, &ops, chosen));
            log.push(format!("{op:?}[{short}]!crash@{name}"));
        } else {
            if let Some(tally) = tally.as_deref_mut() {
                let label = format!("seed={seed} step={step} op={op:?} after");
                note_image(tally, &label, &driver.image(), None, &ledger);
            }
            log.push(format!("{op:?}[{short}]"));
        }
    }
    let line = format!("B35H seed={seed} width={width:?} steps={steps} :: {}", log.join(" "));
    (driver, rng, ledger, line)
}
