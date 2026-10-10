//! 正推腿：第 10z 批第五块 K3。对在 harness 档 `--lib -- gpu_memory_ledger` 里照绿的变异补的会红用例（只用公开函数；真代码上绿，变异上红）。
//! 变异名对应 research/prompts/m3-batch10z-e-code-r1-sonnet-model/mut/mutants.py。
use singlefs_harness::gpu_memory_ledger::{
    after_a_round_of_the_cards, judge_blocks_on_one_card, AfterARoundOfTheCards, BlockHandedToTheStore,
    BlockJudgedOnACard, BlocksForTheCards, CardLeftTheBlocks, DeviceConditionOfACard,
    DeviceWhenTheWorkReturned, RoundsOfTheCards, StopOfTheCards, StopOfTheRun, WorkOnTheCardOfThisThread,
};
use std::sync::mpsc::channel;
use std::time::Duration;

/// M05（判完块号记最后一块而不是最大）：三张卡各领一块 5、6、7；7 先判完、5 后判完、6 最后报 Out of Memory 交回。
/// 叫停之后下一张卡要领得到 6（判完的 7 在等它）。记「最后判完的」5 就领不到。
#[test]
fn m05_the_block_judged_last_is_not_the_largest_judged() {
    let blocks = BlocksForTheCards::new([5usize, 6, 7]);
    let (took_c_sender, took_c) = channel::<()>();
    let (took_d_sender, took_d) = channel::<()>();
    let (took_e_sender, took_e) = channel::<()>();
    let (finish_e_sender, finish_e) = channel::<()>();
    let (e_done_sender, e_done) = channel::<()>();
    let (finish_c_sender, finish_c) = channel::<()>();
    let (c_done_sender, c_done) = channel::<()>();
    let (finish_d_sender, finish_d) = channel::<()>();
    let calls = |_first: bool| {
        let first = std::cell::Cell::new(true);
        move || {
            if first.replace(false) {
                StopOfTheCards::NotRequested
            } else {
                StopOfTheCards::TheRunFailed
            }
        }
    };
    let left_of_d = std::thread::scope(|scope| {
        let blocks = &blocks;
        scope.spawn(move || {
            let stop_fn = calls(true);
            let _ = judge_blocks_on_one_card(
                blocks,
                &stop_fn,
                &mut |block| {
                    assert_eq!(block, 5);
                    took_c_sender.send(()).unwrap();
                    finish_c.recv().unwrap();
                    BlockJudgedOnACard::Judged(block)
                },
                &mut |_, _: usize| {
                    c_done_sender.send(()).unwrap();
                    BlockHandedToTheStore::Handed
                },
            );
        });
        took_c.recv_timeout(Duration::from_secs(20)).unwrap();
        let d = scope.spawn(move || {
            let stop_fn = calls(true);
            judge_blocks_on_one_card(
                blocks,
                &stop_fn,
                &mut |block| {
                    assert_eq!(block, 6);
                    took_d_sender.send(()).unwrap();
                    finish_d.recv().unwrap();
                    BlockJudgedOnACard::<usize>::OutOfMemory { reason: "d".to_string() }
                },
                &mut |_, _: usize| BlockHandedToTheStore::Handed,
            )
        });
        took_d.recv_timeout(Duration::from_secs(20)).unwrap();
        scope.spawn(move || {
            let stop_fn = calls(true);
            let _ = judge_blocks_on_one_card(
                blocks,
                &stop_fn,
                &mut |block| {
                    assert_eq!(block, 7);
                    took_e_sender.send(()).unwrap();
                    finish_e.recv().unwrap();
                    BlockJudgedOnACard::Judged(block)
                },
                &mut |_, _: usize| {
                    e_done_sender.send(()).unwrap();
                    BlockHandedToTheStore::Handed
                },
            );
        });
        took_e.recv_timeout(Duration::from_secs(20)).unwrap();
        // 7 先判完，再 5，最后 6 报 Out of Memory
        finish_e_sender.send(()).unwrap();
        e_done.recv_timeout(Duration::from_secs(20)).unwrap();
        finish_c_sender.send(()).unwrap();
        c_done.recv_timeout(Duration::from_secs(20)).unwrap();
        finish_d_sender.send(()).unwrap();
        d.join().unwrap()
    });
    assert!(matches!(left_of_d, CardLeftTheBlocks::OutOfMemory { .. }));
    assert_eq!(
        blocks.take_a_block(StopOfTheCards::RequestedByTheCaller),
        Some(6),
        "7 判完了在等 6：叫停之后下一张卡领得到 6（判完的块里最大的是 7，不是最后判完的 5）"
    );
}

/// M10（叫停、等着的块边界）：叫停、一张卡报了 Out of Memory、判完的块只有 1 块在等：还有别的卡就再起一轮，没有别的卡是本次未跑，都不是照叫停收尾。
#[test]
fn m10_one_waiting_judged_block_still_needs_another_round_or_not_run() {
    assert_eq!(
        after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 1),
        AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
    );
    assert_eq!(
        after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 0),
        AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
    );
    assert_eq!(
        after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
        AfterARoundOfTheCards::StoppedOnRequest
    );
}

fn panics(f: impl FnOnce() + std::panic::UnwindSafe) -> bool {
    std::panic::catch_unwind(f).is_err()
}

/// M13：没有卡报 Out of Memory 而判完的块还有在等前面那一块的：轮末断言 panic。
#[test]
fn m13_a_round_without_out_of_memory_leaves_no_judged_block_waiting() {
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
    let blocks = rounds.take_the_blocks_of_the_round();
    let _ = blocks.take_a_block(StopOfTheCards::NotRequested);
    rounds.stored_in_order().arrive(2, 2, &mut |_| Ok(()));
    assert_eq!(rounds.blocks_waiting(), 1);
    assert!(panics(std::panic::AssertUnwindSafe(|| {
        let _ = rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::Requested);
    })));
}

/// M14：没人叫停、没有卡报 Out of Memory，队里却还有没领走的块：轮末断言 panic。
#[test]
fn m14_a_round_without_a_stop_leaves_no_block_untaken() {
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
    let blocks = rounds.take_the_blocks_of_the_round();
    assert!(panics(std::panic::AssertUnwindSafe(|| {
        let _ = rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::NotRequested);
    })));
    // 叫停了就可以留
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
    let blocks = rounds.take_the_blocks_of_the_round();
    assert_eq!(
        rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::Requested),
        AfterARoundOfTheCards::StoppedOnRequest
    );
}

/// M17：这一趟头一句 Out of Memory 是第一张报的那一句，后面的不盖掉它。
#[test]
fn m17_the_first_out_of_memory_stays_the_first() {
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
    let blocks = rounds.take_the_blocks_of_the_round();
    let _ = rounds.after_the_round(&blocks, &[0, 1], [], &[(0, "第一句".to_string()), (1, "第二句".to_string())], StopOfTheRun::NotRequested);
    assert_eq!(rounds.first_out_of_memory(), Some("第一句"));
}

/// M22：一块交回两次要 panic。
#[test]
fn m22_giving_a_block_back_twice_panics() {
    let blocks = BlocksForTheCards::new([0usize, 1]);
    let first = blocks.take_a_block(StopOfTheCards::NotRequested).unwrap();
    blocks.give_back(first);
    assert!(panics(std::panic::AssertUnwindSafe(|| blocks.give_back(first))));
}

/// M23：同一段活上记了两次 Out of Memory，认出来的是最后那一次。
#[test]
fn m23_the_last_out_of_memory_of_a_work_is_the_one_told() {
    let condition = DeviceConditionOfACard::default();
    let work = WorkOnTheCardOfThisThread::begin();
    let before = condition.out_of_memory_reports();
    condition.record_an_out_of_memory("先报的".to_string());
    condition.record_an_out_of_memory("后报的".to_string());
    let told = condition.out_of_memory_explaining_a_failure_of_the_work(work.identifier(), before, DeviceWhenTheWorkReturned::NotLost);
    assert_eq!(told.as_deref(), Some("后报的"));
}
