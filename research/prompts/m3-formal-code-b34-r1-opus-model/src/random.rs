//! 随机历史：用户动作按会话状态抽（不写死后缀），按种子在某一步上崩在它第 j 次根槽写之前（那一步的单元与记录都落了、根没落），
//! 再由下一步（同样随机抽）起新进程。每一步之后的镜像，与每一步里每次根槽写之前那一刻（记录全落、或只落第一条记录的第一份）
//! 的镜像，都交新旧两份 checker 对拍。后两类是层 0 枚举域里的状态（更早的段全持久、当前段的一个子集）。
use singlefs_harness::history::{HistoryDeviceWidth, SeededRandomSource};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::StepKind;
use singlefs_harness::RetainedOperation;

use crate::drive::{Driver, Op};
use crate::judge::compare;
use crate::tally::Tally;

pub const WIDTHS: [HistoryDeviceWidth; 3] = [
    HistoryDeviceWidth::UnitAreaOf256Slots,
    HistoryDeviceWidth::UnitAreaOf384Slots,
    HistoryDeviceWidth::FourGibibytes,
];

/// 按会话状态抽一步。
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
            0..=4 => Op::Overwrite(selector),
            5 => Op::SequentialWrite(selector),
            6 => Op::Unlink,
            7..=9 => Op::Rollback(rng.below(8)),
            10..=11 => Op::RaiseFloor(rng.below(64)),
            12..=13 => Op::Unmount,
            _ => Op::Overwrite(selector),
        },
    }
}

/// 一步的录制流里每次根槽写的下标，与它之前那次发布里第一条 journal 记录写的下标（有的话）。
pub fn root_writes_with_first_record(driver: &Driver, ops: &[RetainedOperation]) -> Vec<(usize, Option<usize>)> {
    let geometry = driver.geometry();
    let mut out = Vec::new();
    let mut first_record_since_the_last_root: Option<usize> = None;
    for (index, retained) in ops.iter().enumerate() {
        match geometry.classify(&retained.operation) {
            StepKind::JournalRecord => {
                first_record_since_the_last_root.get_or_insert(index);
            }
            StepKind::RootRecordFua => {
                out.push((index, first_record_since_the_last_root.take()));
            }
            _ => {}
        }
    }
    out
}

pub fn image_with_prefix(before: &MemoryPool, ops: &[RetainedOperation], end: usize) -> MemoryPool {
    let mut image = before.clone();
    image.apply(&ops[..end]);
    image
}

/// 一段历史：`steps` 步，每步四分之一的机会崩在它的某次根槽写之前。`tally` 给了就逐张对拍；交回驱动、随机源与一行摘要。
pub fn play(
    seed: u64,
    steps: usize,
    mut tally: Option<&mut Tally>,
) -> (Driver, SeededRandomSource, String) {
    let mut rng = SeededRandomSource::from_seed(seed);
    let width = WIDTHS[rng.below(WIDTHS.len() as u64) as usize];
    let mut driver = Driver::mkfs(width);
    let mut log: Vec<String> = Vec::new();
    for step in 0..steps {
        let op = draw_op(driver.state(), &mut rng);
        let crash = rng.below(4) == 0;
        let crash_choice = rng.below(16);
        let before = driver.image();
        let start = driver.stream.operation_count();
        let outcome = driver.execute(op);
        let ops = driver.ops_since(start);
        let roots = root_writes_with_first_record(&driver, &ops);
        if let Some(tally) = tally.as_deref_mut() {
            for (root_index, first_record) in &roots {
                let label = format!("seed={seed} step={step} op={op:?} cut=before-root@{root_index}");
                tally.note(&label, &compare(&image_with_prefix(&before, &ops, *root_index)));
                if let Some(first) = first_record {
                    let label = format!("seed={seed} step={step} op={op:?} cut=first-record@{first}");
                    tally.note(&label, &compare(&image_with_prefix(&before, &ops, first + 1)));
                }
            }
        }
        let short = outcome.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
        if crash && !roots.is_empty() {
            let (root_index, _) = roots[(crash_choice % roots.len() as u64) as usize];
            let crash_image = image_with_prefix(&before, &ops, root_index);
            driver = Driver::from_image(width, &crash_image);
            log.push(format!("{op:?}[{short}]!crash@root{}", crash_choice % roots.len() as u64));
        } else {
            if let Some(tally) = tally.as_deref_mut() {
                let label = format!("seed={seed} step={step} op={op:?} after");
                tally.note(&label, &compare(&driver.image()));
            }
            log.push(format!("{op:?}[{short}]"));
        }
    }
    let line = format!("B34H seed={seed} width={width:?} steps={steps} :: {}", log.join(" "));
    (driver, rng, line)
}

pub fn run_history(seed: u64, steps: usize, tally: &mut Tally) -> String {
    play(seed, steps, Some(tally)).2
}
