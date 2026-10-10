//! 第 10z 批代码轮第一轮攻方模型：P2（一张卡 Out of Memory 只停那一张）的块队与写库次序。
//! `BlocksForTheCards` 一字不差抄自冻结副本 `crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2048–2106 行（去掉 cfg）；
//! 卡线程那一圈照第 2250–2266 行（领一块、判、报了 Out of Memory 就交回、退出），写库照第 2001–2011 行（按块序，前面那块没到就等），
//! 收尾照第 2428–2438 行。卡上的线程换成按挂钟排的事件（不起线程）：每张卡判一块要 `seconds` 那么久。
struct BlocksForTheCards {
    pending_blocks: usize,
    next_pending: std::sync::atomic::AtomicUsize,
    given_back: std::sync::Mutex<Vec<usize>>,
}

impl BlocksForTheCards {
    fn new(pending_blocks: usize) -> Self {
        Self {
            pending_blocks,
            next_pending: std::sync::atomic::AtomicUsize::new(0),
            given_back: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn take_a_block(&self) -> Option<usize> {
        let mut given_back = self
            .given_back
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(smallest) = given_back.iter().copied().min() {
            given_back.retain(|pending_index| *pending_index != smallest);
            return Some(smallest);
        }
        drop(given_back);
        let pending_index = self
            .next_pending
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        (pending_index < self.pending_blocks).then_some(pending_index)
    }

    fn give_back(&self, pending_index: usize) {
        self.given_back
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(pending_index);
    }
}

/// 一趟：`seconds[c]` 是卡 c 判一块的挂钟，卡 0 判它领的第 `oom_at` 块（从 1 数）时报 Out of Memory（判到那一块末尾才报）。
/// 交回（写进库的块数、判完交去写库的块数、这一趟的结局）。同一刻两件事先办卡号大的（卡 1 先领）。
fn one_run(pending: usize, seconds: &[u64], oom_at: usize) -> (usize, usize, &'static str) {
    let blocks = BlocksForTheCards::new(pending);
    let mut busy: Vec<Option<(u64, usize)>> = seconds
        .iter()
        .map(|s| blocks.take_a_block().map(|b| (*s, b)))
        .collect();
    let mut taken_by_card_0 = 1;
    let (mut stored, mut judged) = (0usize, 0usize);
    let mut waiting = std::collections::BTreeSet::new();
    let mut out_of_memory = false;
    while let Some((card, (at, block))) = busy
        .iter()
        .enumerate()
        .filter_map(|(c, b)| b.map(|b| (c, b)))
        .min_by_key(|(c, (at, _))| (*at, std::cmp::Reverse(*c)))
    {
        if card == 0 && taken_by_card_0 == oom_at {
            blocks.give_back(block);
            out_of_memory = true;
            busy[0] = None;
            continue;
        }
        judged += 1;
        waiting.insert(block);
        while waiting.remove(&stored) {
            stored += 1;
        }
        busy[card] = blocks.take_a_block().map(|b| (at + seconds[card], b));
        if card == 0 && busy[0].is_some() {
            taken_by_card_0 += 1;
        }
    }
    let outcome = match (out_of_memory, stored >= pending) {
        (false, _) => "judged",
        (true, true) => "every-block-judged",
        (true, false) => "not-run",
    };
    (stored, judged, outcome)
}

#[test]
fn a_block_given_back_after_the_other_card_left_is_not_judged_and_the_blocks_after_it_are_judged_but_not_stored() {
    const PENDING: usize = 20;
    for seconds in [[1u64, 1], [2, 1], [3, 1]] {
        let mut not_run = 0;
        for oom_at in 1..=PENDING {
            let (stored, judged, outcome) = one_run(PENDING, &seconds, oom_at);
            if outcome != "judged" {
                println!("ATTACK scenario=P2 seconds={seconds:?} oom_at_block_of_card_0={oom_at} stored={stored} judged={judged} judged_not_stored={} outcome={outcome}", judged - stored);
            }
            if outcome == "not-run" {
                not_run += 1;
            }
        }
        println!("ATTACK scenario=P2-summary seconds={seconds:?} not_run_cells={not_run}");
    }
    assert_eq!(one_run(20, &[2, 1], 7).2, "not-run");
}

/// 一张卡装下了全部状态（卡 1 没装，第 2404–2405 行不起它）：卡 0 一报就是本次未跑，卡 1 闲着。
#[test]
fn with_every_state_on_the_first_card_one_out_of_memory_is_not_run_while_the_second_card_is_idle() {
    let (stored, judged, outcome) = one_run(20, &[1], 5);
    println!("ATTACK scenario=P2-one-card-placed oom_at_block_of_card_0=5 stored={stored} judged={judged} outcome={outcome}");
    assert_eq!((stored, judged, outcome), (4, 4, "not-run"));
}
