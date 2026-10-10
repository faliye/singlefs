//! 第 10z 批第六块代码轮攻方（opus）模型：K2-2「照叫停收尾时头一句 Out of Memory 那一行」打不打（实现员自定第 5 处：取「这一趟有卡报过」）。
//! harness 档的 `RoundsOfTheCards`、`BlocksForTheCards`、`judge_blocks_on_one_card`、`next_card_to_start`、`StopRequestRememberedInThisRun`
//! 原样调用；checker 档外层循环（`crash_amplification.rs` 第 2950–3160 行）照次序抄成单线程一份：卡一张一张跑完。
//! 两张卡：头一轮状态全装在卡 0（卡 1 不起），卡 0 判第 `oom_at` 块时报 Out of Memory；叫停文件在第 `stop_at` 次问叫停时出现、之后一直在。
use singlefs_harness::gpu_memory_ledger::{
    judge_blocks_on_one_card, next_card_to_start, AfterARoundOfTheCards, BlockHandedToTheStore,
    BlockJudgedOnACard, StopOfTheCards, StopRequestRememberedInThisRun, RoundsOfTheCards,
};
use std::sync::atomic::{AtomicUsize, Ordering};

const BLOCKS: usize = 6;

#[derive(Debug, PartialEq, Eq)]
struct Ending {
    outcome: String,
    line_this_trip: bool,
    line_this_round: bool,
    blocks_stored: usize,
    rounds: usize,
}

fn run(oom_at: Option<usize>, stop_at: Option<usize>) -> Ending {
    let asks = AtomicUsize::new(0);
    let reader = || {
        let ask = asks.fetch_add(1, Ordering::SeqCst);
        stop_at.is_some_and(|at| ask >= at)
    };
    let stop_request = StopRequestRememberedInThisRun::new(&reader);
    let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(BLOCKS);
    let mut judged_by_card0 = 0;
    let mut round_number = 0;
    loop {
        round_number += 1;
        // 这一轮挑卡：报过的不再用；头一轮全装在卡 0，之后装在剩下的卡上
        let cards: Vec<u32> = [0_u32, 1].into_iter().filter(|card| !rounds.is_short_of_memory(*card)).collect();
        let states_placed: Vec<u64> = if round_number == 1 { vec![BLOCKS as u64, 0] } else { vec![BLOCKS as u64] };
        let blocks = rounds.take_the_blocks_of_the_round();
        let stop_of_the_cards = || {
            if stop_request.is_requested() { StopOfTheCards::RequestedByTheCaller } else { StopOfTheCards::NotRequested }
        };
        let mut out_of_memory: Vec<(u32, String)> = Vec::new();
        let mut finished: Vec<u32> = Vec::new();
        let mut position = 0;
        let mut arrived: Vec<usize> = Vec::new();
        loop {
            let card = cards[position];
            let left = judge_blocks_on_one_card(
                &blocks,
                &stop_of_the_cards,
                &mut |block| {
                    if card == 0 && oom_at == Some(judged_by_card0) {
                        return BlockJudgedOnACard::OutOfMemory { reason: format!("卡 0 判块 {block} 时 Out of Memory") };
                    }
                    if card == 0 { judged_by_card0 += 1; }
                    BlockJudgedOnACard::Judged(block)
                },
                &mut |block, judged| { arrived.push(block); let _ = judged; BlockHandedToTheStore::Handed },
            );
            match left {
                singlefs_harness::gpu_memory_ledger::CardLeftTheBlocks::OutOfMemory { reason } => out_of_memory.push((card, reason)),
                _ => finished.push(card),
            }
            match next_card_to_start(&states_placed, position, &blocks, stop_of_the_cards()) {
                Some(next) => position = next,
                None => break,
            }
        }
        for block in arrived {
            rounds.stored_in_order().arrive(block, block, &mut |_| Ok(()));
        }
        let had_out_of_memory_this_round = !out_of_memory.is_empty();
        let after = rounds.after_the_round(&blocks, &cards, finished, &out_of_memory, stop_request.stop_of_the_run());
        let line = rounds.not_run_line_of_a_stop_after_an_out_of_memory("crash_amplification_product_path").is_some();
        match after {
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
            | AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => continue,
            AfterARoundOfTheCards::StoppedOnRequest => {
                return Ending {
                    outcome: "StoppedOnRequest".into(),
                    line_this_trip: line,
                    line_this_round: line && had_out_of_memory_this_round,
                    blocks_stored: rounds.blocks_stored(),
                    rounds: round_number,
                };
            }
            other => {
                return Ending {
                    outcome: format!("{other:?}"),
                    line_this_trip: false,
                    line_this_round: false,
                    blocks_stored: rounds.blocks_stored(),
                    rounds: round_number,
                };
            }
        }
    }
}

/// 扫卡 0 在第几块报 Out of Memory（含不报）与叫停在第几次问时出现（含不叫停）：数照「这一趟」打而照「这一轮」不打的段。
#[test]
fn k2_2_the_not_run_line_of_a_stop_counts_an_out_of_memory_of_an_earlier_round() {
    let mut histories = 0;
    let mut differ = 0;
    let mut printed_trip = 0;
    let mut printed_round = 0;
    let mut completes_without_the_out_of_memory = 0;
    for oom_at in (0..BLOCKS).map(Some).chain([None]) {
        for stop_at in (0..BLOCKS + 6).map(Some).chain([None]) {
            let ending = run(oom_at, stop_at);
            histories += 1;
            printed_trip += usize::from(ending.line_this_trip);
            printed_round += usize::from(ending.line_this_round);
            if ending.line_this_trip != ending.line_this_round {
                differ += 1;
                // 同一个叫停时刻（按问叫停的次数算）、卡 0 不报 Out of Memory 的那一段：判完了，就是这一次 Out of Memory 拖慢了、叫停才截住
                let without = run(None, stop_at);
                if without.outcome == "EveryBlockStored" {
                    completes_without_the_out_of_memory += 1;
                }
                println!(
                    "ATTACK k2_2 oom_at={oom_at:?} stop_at={stop_at:?} {ending:?} same_stop_without_oom={} stored_without_oom={}",
                    without.outcome, without.blocks_stored
                );
            }
        }
    }
    println!("ATTACK k2_2 summary histories={histories} printed_this_trip={printed_trip} printed_this_round={printed_round} differ={differ} differ_that_complete_without_the_oom={completes_without_the_out_of_memory}");
}
