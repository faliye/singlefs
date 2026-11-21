//! 对拍结论按块：块键、整条流的块按组分给几台几段、一段段在各自的线程上判、按块序存，复用或现判。

use super::*;

// ============================== 对拍结论按块：块键、按块复用或现判 ==============================

/// 一条流逐状态对拍时一块几个状态：与崩溃放量同一个块宽（[`crate::crash_amplification::STATES_PER_BLOCK`]）。
pub const STATES_PER_PAIRING_BLOCK: u64 = crate::crash_amplification::STATES_PER_BLOCK;

/// 一块的输入摘要（块键）：整条流那一份对拍输入摘要（调用方只按输入数据算：基线镜像、写表、切段与展开、版本表、判表这类）拼上这一块的
/// 序号段 `[start, end)`，取 SHA-256。进库时它再拼判器引到的被检测产品代码（[`pairing_conclusion_key_of_the_inputs`]），存在配对结论那一张表里
/// （`verdict_store` 文件头登记的表，不另开表、不另开键空间）：块的输入数据与判器引到的产品代码相同的下一趟复用，两边判器与比法自己的代码不进键。
#[must_use]
pub fn pairing_block_inputs_digest(
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    ordinals_of_the_block: &std::ops::Range<u64>,
) -> [u8; DIGEST_BYTES] {
    let mut bytes = b"crash judge pairing block\n".to_vec();
    bytes.extend_from_slice(flow_inputs_digest);
    bytes.extend_from_slice(&ordinals_of_the_block.start.to_le_bytes());
    bytes.extend_from_slice(&ordinals_of_the_block.end.to_le_bytes());
    sha256_digest(&bytes)
}

/// 一条流按块对拍下来几块复用、几块现判、几块归别的份，与同一时刻手里最多压着几块的结论（宿主内存只随它涨，不随整条流的块数涨）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairingBlocksOfTheFlow {
    pub blocks_reused: u64,
    pub blocks_judged_now: u64,
    /// 归别的份的块（[`PairingBlocksShare`]）：这一份不读库、不判、不折进总账；不是 0 时总账不是整条流的。
    pub blocks_left_to_other_shares: u64,
    /// 落在这一趟走的组之外的块（[`PairingBlocksShare::only_the_groups_of_pairing_blocks_in`]）：同样不读库、不判、不折进总账；
    /// 不是 0 时总账只是那几组的状态，不是整条流的。
    pub blocks_outside_the_groups_of_this_run: u64,
    pub most_block_conclusions_held_at_once: usize,
}

/// 判一段连着的块：交这一段里每块的序号段，按同一次序交回每块的结论字节与两边一致不一致；交 `None` 是判那一侧报「本次未跑」。
/// 几段在各自的线程上同时判（[`pairing_conclusions_of_the_blocks_reused_or_judged`]）。
pub type JudgeOfAStretchOfPairingBlocks<'closure> =
    dyn Fn(&[std::ops::Range<u64>]) -> Option<Vec<PairingJudged>> + Sync + 'closure;

/// 这一趟领一条流的哪些块（几台机器分着判同一条流）：块按每段 `most_blocks_judged_together` 块分成组，一组归一份（判一段要连着的块，
/// 一块一块轮着分，CPU 那一侧每块都要重建一次撕裂表）；哪组归哪一份只由权重表、流的输入摘要与组的序号定，几份各算各的、不用商量，
/// 每一块恰好归一份。块键（[`pairing_block_inputs_digest`]）不带份额：哪一份判的，判出的块照同一把键进同一份库，合并那一趟按整份（`0/1`）全部复用。
/// 流的输入摘要只错开各条流从哪一份轮起：一块的小流不会全落在同一份上。
/// 另可只走一段组（[`Self::only_the_groups_of_pairing_blocks_in`]：一条流按状态序号区间拆成几条用例，每条只领自己那一段序号的组），
/// 与几台分份叠用：组落在这一段里、又归这一份，才算领。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingBlocksShare {
    share_of_this_run: crate::crash_amplification::HostShare,
    weights_of_the_shares: crate::crash_amplification::WeightedShares,
    groups_of_this_run: GroupsOfPairingBlocksOfThisRun,
}

/// 一趟走一条流的哪几组块。
#[derive(Clone, Debug, PartialEq, Eq)]
enum GroupsOfPairingBlocksOfThisRun {
    EveryGroup,
    /// 组的序号落在这个半开区间里的。
    OnlyTheGroupsIn(std::ops::Range<u64>),
}

/// 状态序号区间 `ordinals` 落在一条流的哪几组块上（块宽 `states_per_block`、一组 `blocks_per_group` 块，组从序号 0 起排，同
/// [`pairing_conclusions_of_the_blocks_reused_or_judged`] 的分组）：交回组的半开区间。区间两头要落在组的边界上，终点另可以是整条流的状态数
/// `state_count`（最后一组不满）；不然领整组时会判到区间外的状态、或漏判区间里的，交 `Err`。
///
/// # Errors
/// 区间是空的、终点越过 `state_count`、起点不在组的边界上、终点既不在组的边界上也不是 `state_count`，或一组的状态数装不进 u64。
pub fn groups_of_pairing_blocks_holding_the_states(
    ordinals: &std::ops::Range<u64>,
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    blocks_per_group: std::num::NonZeroUsize,
) -> Result<std::ops::Range<u64>, String> {
    let states_per_group = u64::try_from(blocks_per_group.get())
        .ok()
        .and_then(|blocks| blocks.checked_mul(states_per_block.get()))
        .ok_or_else(|| {
            format!(
                "一组 {blocks_per_group} 块、一块 {states_per_block} 个状态：一组的状态数装不进 u64"
            )
        })?;
    if ordinals.start >= ordinals.end || ordinals.end > state_count {
        return Err(format!(
            "状态序号区间 {ordinals:?} 是空的或越过了整条流的 {state_count} 个状态"
        ));
    }
    if !ordinals.start.is_multiple_of(states_per_group) {
        return Err(format!(
            "状态序号区间 {ordinals:?} 的起点不在组的边界上（一组 {states_per_group} 个状态）"
        ));
    }
    if !ordinals.end.is_multiple_of(states_per_group) && ordinals.end != state_count {
        return Err(format!(
            "状态序号区间 {ordinals:?} 的终点既不在组的边界上（一组 {states_per_group} 个状态）、也不是整条流的状态数 {state_count}"
        ));
    }
    Ok(ordinals.start / states_per_group..ordinals.end.div_ceil(states_per_group))
}

impl PairingBlocksShare {
    /// 整条流都归这一趟（`0/1`）。
    #[must_use]
    pub fn whole_flow_in_one_share() -> Self {
        Self::with_this_share_of_these_weights(
            crate::crash_amplification::HostShare { index: 0, count: 1 },
            vec![1],
        )
    }

    /// 同 `self` 那一份，另只走组的序号落在 `groups` 里的组（替掉之前给过的组区间）；组区间由
    /// [`groups_of_pairing_blocks_holding_the_states`] 从状态序号区间算。
    #[must_use]
    pub fn only_the_groups_of_pairing_blocks_in(self, groups: std::ops::Range<u64>) -> Self {
        Self {
            share_of_this_run: self.share_of_this_run,
            weights_of_the_shares: self.weights_of_the_shares,
            groups_of_this_run: GroupsOfPairingBlocksOfThisRun::OnlyTheGroupsIn(groups),
        }
    }

    /// 第 `group_of_blocks` 组落不落在这一趟走的组里（没给组区间的一趟每一组都落在里面）。
    #[must_use]
    pub fn groups_of_this_run_hold(&self, group_of_blocks: u64) -> bool {
        match &self.groups_of_this_run {
            GroupsOfPairingBlocksOfThisRun::EveryGroup => true,
            GroupsOfPairingBlocksOfThisRun::OnlyTheGroupsIn(groups) => {
                groups.contains(&group_of_blocks)
            }
        }
    }

    /// 领 `share_of_this_run` 这一份，几份的权重是 `weights_of_the_shares`。
    ///
    /// # Panics
    /// 权重的个数与份数不同；权重全是 0。
    #[must_use]
    pub fn with_this_share_of_these_weights(
        share_of_this_run: crate::crash_amplification::HostShare,
        weights_of_the_shares: Vec<u64>,
    ) -> Self {
        assert_eq!(
            weights_of_the_shares.len(),
            share_of_this_run.count,
            "每一份一个权重"
        );
        Self {
            share_of_this_run,
            weights_of_the_shares: crate::crash_amplification::WeightedShares::new(
                weights_of_the_shares,
            ),
            groups_of_this_run: GroupsOfPairingBlocksOfThisRun::EveryGroup,
        }
    }

    /// 这个进程领哪一份：与崩溃放量同一对变量（`crate::crash_amplification::host_share_and_weights_from_the_environment`），没设是整份。
    ///
    /// # Errors
    /// 份额或权重写错。
    pub fn of_this_process() -> Result<Self, String> {
        let (share_of_this_run, weights_of_the_shares) =
            crate::crash_amplification::host_share_and_weights_from_the_environment()?;
        Ok(Self::with_this_share_of_these_weights(
            share_of_this_run,
            weights_of_the_shares,
        ))
    }

    /// 输入摘要是 `flow_inputs_digest` 的流第 `group_of_blocks` 组块归不归这一趟：落在这一趟走的组里（[`Self::groups_of_this_run_hold`]）、
    /// 又归这一份。
    #[must_use]
    pub fn claims_this_group_of_pairing_blocks(
        &self,
        flow_inputs_digest: &[u8; DIGEST_BYTES],
        group_of_blocks: u64,
    ) -> bool {
        let where_the_flow_starts_in_the_rotation = u64::from(u32::from_le_bytes(
            flow_inputs_digest[..4]
                .try_into()
                .expect("摘要不止 4 个字节"),
        ));
        self.groups_of_this_run_hold(group_of_blocks)
            && self
                .weights_of_the_shares
                .owner_of_block(where_the_flow_starts_in_the_rotation + group_of_blocks)
                == self.share_of_this_run.index
    }
}

/// 判一段块的那条线程收尾时交回的。
struct FinishedPairingStretch {
    started_as: u64,
    /// 判那一段 panic 了是 `Err`（收的那一头等在飞的都收尾之后接着往外抛）。
    judged: std::thread::Result<Option<Vec<PairingJudged>>>,
}

/// 等着按序号次序折进总账的一项。
enum PairingBlocksAwaitingTheLedger {
    Reused {
        ordinals: std::ops::Range<u64>,
        conclusion: Vec<u8>,
    },
    /// `conclusions` 是 `None`：这一段还在判，或判那一侧报了「本次未跑」。
    Stretch {
        started_as: u64,
        blocks: Vec<std::ops::Range<u64>>,
        conclusions: Option<Vec<PairingJudged>>,
    },
}

/// 判那一侧为什么不再往下判了。
enum PairingJudgingStopped {
    /// 有一段交了 `None`（判那一侧报「本次未跑」）：那一段一块都不存。
    NotRun,
    /// 有一段 panic 了：照原样接着往外抛。
    Panicked(Box<dyn std::any::Any + Send>),
}

/// 一条流这一趟正在判的那几段与等着折进总账的块：只在起 [`pairing_conclusions_of_the_blocks_reused_or_judged`] 的那条线程上动。
struct PairingStretchesOfTheRun<'run> {
    library: &'run CrashLibrary,
    key_of_the_block: &'run dyn Fn(&std::ops::Range<u64>) -> PairingConclusionKey,
    take_a_block_conclusion: &'run mut dyn FnMut(&std::ops::Range<u64>, &[u8]),
    blocks_of_the_flow: PairingBlocksOfTheFlow,
    stretches_judged_at_once: usize,
    /// 等着折进总账的块（在判的算上）至多这么多：头上那一段迟迟判不完时，走块那一头停下来等，身后复用的、判完的结论不无限堆。
    most_blocks_awaiting_the_ledger: usize,
    awaiting_the_ledger: VecDeque<PairingBlocksAwaitingTheLedger>,
    stretches_started: u64,
    stretches_finished: u64,
    stretches_in_flight: usize,
    judging_stopped: Option<PairingJudgingStopped>,
    finished_stretches: mpsc::Receiver<FinishedPairingStretch>,
}

impl PairingStretchesOfTheRun<'_> {
    /// 此刻至多几段一起判：头一段判完之前只有它一段（录入段在它里面走完、判表在它里面过库，后面几段读的是录好的），之后 `stretches_judged_at_once` 段。
    fn stretches_allowed_in_flight_now(&self) -> usize {
        if self.stretches_finished == 0 {
            1
        } else {
            self.stretches_judged_at_once
        }
    }

    /// 等一段判完：判出来的逐块存进库（后面的段先判完也当场存，存进库的不等总账），记进等着折进总账的那一项，能折的折掉。
    ///
    /// # Errors
    /// 库写失败。
    ///
    /// # Panics
    /// 交回的结论块数与交出去的块数不同。
    fn wait_for_a_pairing_stretch_to_finish(&mut self) -> Result<(), String> {
        let FinishedPairingStretch {
            started_as: finished_stretch,
            judged,
        } = self
            .finished_stretches
            .recv()
            .expect("判一段的线程收尾都交一回（panic 也接住交），在飞的段不是 0 才等");
        self.stretches_in_flight -= 1;
        self.stretches_finished += 1;
        let conclusions = match judged {
            Ok(Some(conclusions)) => conclusions,
            Ok(None) => {
                if self.judging_stopped.is_none() {
                    self.judging_stopped = Some(PairingJudgingStopped::NotRun);
                }
                return Ok(());
            }
            Err(panicked) => {
                self.judging_stopped = Some(PairingJudgingStopped::Panicked(panicked));
                return Ok(());
            }
        };
        let awaiting_it = self
            .awaiting_the_ledger
            .iter_mut()
            .find_map(|awaiting| match awaiting {
                PairingBlocksAwaitingTheLedger::Stretch {
                    started_as,
                    blocks,
                    conclusions: conclusions_of_the_stretch,
                } if *started_as == finished_stretch => Some((blocks, conclusions_of_the_stretch)),
                PairingBlocksAwaitingTheLedger::Stretch { .. }
                | PairingBlocksAwaitingTheLedger::Reused { .. } => None,
            })
            .expect("起一段时把它排进了等着折进总账的那一队，折掉之前它已判完");
        let (blocks, conclusions_awaiting) = awaiting_it;
        assert_eq!(
            conclusions.len(),
            blocks.len(),
            "判一段块交回的结论块数与交出去的块数相同"
        );
        store_the_judged_pairing_blocks(self.library, self.key_of_the_block, blocks, &conclusions)?;
        *conclusions_awaiting = Some(conclusions);
        self.note_the_block_conclusions_held();
        self.take_the_blocks_at_the_head_of_the_queue();
        Ok(())
    }

    /// 手里此刻压着几块的结论（判完、复用的还没轮到折进总账的），记下最多的一次。
    fn note_the_block_conclusions_held(&mut self) {
        let held: usize = self
            .awaiting_the_ledger
            .iter()
            .map(|awaiting| match awaiting {
                PairingBlocksAwaitingTheLedger::Reused { .. } => 1,
                PairingBlocksAwaitingTheLedger::Stretch { conclusions, .. } => {
                    conclusions.as_ref().map_or(0, Vec::len)
                }
            })
            .sum();
        self.blocks_of_the_flow.most_block_conclusions_held_at_once = self
            .blocks_of_the_flow
            .most_block_conclusions_held_at_once
            .max(held);
    }

    /// 队头有结论的逐块按序号次序折进总账，折完就丢；队头那一段还在判（或报了本次未跑）就停。
    fn take_the_blocks_at_the_head_of_the_queue(&mut self) {
        // 迭代上界是队长；跨轮只有队在变短
        while let Some(head) = self.awaiting_the_ledger.front() {
            match head {
                PairingBlocksAwaitingTheLedger::Stretch {
                    conclusions: None, ..
                } => return,
                PairingBlocksAwaitingTheLedger::Stretch {
                    conclusions: Some(_),
                    ..
                }
                | PairingBlocksAwaitingTheLedger::Reused { .. } => {}
            }
            match self
                .awaiting_the_ledger
                .pop_front()
                .expect("队头刚看过，还在")
            {
                PairingBlocksAwaitingTheLedger::Reused {
                    ordinals,
                    conclusion,
                } => {
                    (self.take_a_block_conclusion)(&ordinals, &conclusion);
                    self.blocks_of_the_flow.blocks_reused += 1;
                }
                PairingBlocksAwaitingTheLedger::Stretch {
                    blocks,
                    conclusions,
                    ..
                } => {
                    for (ordinals, judged) in blocks.iter().zip(conclusions.iter().flatten()) {
                        (self.take_a_block_conclusion)(ordinals, &judged.conclusion);
                        self.blocks_of_the_flow.blocks_judged_now += 1;
                    }
                }
            }
        }
    }

    /// 等到队里放得下 `blocks` 块（在飞的段判完腾地方）。不再往下判了就不等。
    ///
    /// # Errors
    /// 库写失败。
    fn wait_for_room_for_blocks_awaiting_the_ledger(
        &mut self,
        blocks: usize,
    ) -> Result<(), String> {
        // 迭代上界是在飞的段数：每一轮等完一段；在飞的是 0 时队已折空（队头只可能是在判的段）
        while self.judging_stopped.is_none()
            && self.stretches_in_flight > 0
            && self.blocks_awaiting_the_ledger() + blocks > self.most_blocks_awaiting_the_ledger
        {
            self.wait_for_a_pairing_stretch_to_finish()?;
        }
        Ok(())
    }

    fn blocks_awaiting_the_ledger(&self) -> usize {
        self.awaiting_the_ledger
            .iter()
            .map(|awaiting| match awaiting {
                PairingBlocksAwaitingTheLedger::Reused { .. } => 1,
                PairingBlocksAwaitingTheLedger::Stretch { blocks, .. } => blocks.len(),
            })
            .sum()
    }

    /// 等到在飞的段少于此刻许的段数。不再往下判了就不等。
    ///
    /// # Errors
    /// 库写失败。
    fn wait_for_a_free_place_among_the_stretches_in_flight(&mut self) -> Result<(), String> {
        // 迭代上界是在飞的段数：每一轮等完一段
        while self.judging_stopped.is_none()
            && self.stretches_in_flight >= self.stretches_allowed_in_flight_now()
        {
            self.wait_for_a_pairing_stretch_to_finish()?;
        }
        Ok(())
    }

    /// 等在飞的段都收尾（不再往下判了也等：它们判完的照样存进库）。
    ///
    /// # Errors
    /// 库写失败。
    fn wait_for_every_stretch_in_flight(&mut self) -> Result<(), String> {
        // 迭代上界是在飞的段数：每一轮等完一段
        while self.stretches_in_flight > 0 {
            self.wait_for_a_pairing_stretch_to_finish()?;
        }
        Ok(())
    }
}

/// 把攒着的那一段连着的块排进队、在一条线程上判（没攒着块、已经不再往下判了就只清空攒着的）；此刻在飞的段满了先等，起完再满了也等：
/// 一次只许一段时，走下一块之前这一段已经判完、存进库、折进总账。
///
/// # Errors
/// 库写失败。
fn start_judging_the_pending_pairing_blocks<'scope, 'environment: 'scope>(
    scope: &'scope std::thread::Scope<'scope, 'environment>,
    stretches: &mut PairingStretchesOfTheRun<'_>,
    pending_blocks: &mut Vec<std::ops::Range<u64>>,
    judge_the_blocks: &'environment JudgeOfAStretchOfPairingBlocks<'environment>,
    finished_stretches: &mpsc::Sender<FinishedPairingStretch>,
) -> Result<(), String> {
    if pending_blocks.is_empty() {
        return Ok(());
    }
    stretches.wait_for_a_free_place_among_the_stretches_in_flight()?;
    stretches.wait_for_room_for_blocks_awaiting_the_ledger(pending_blocks.len())?;
    if stretches.judging_stopped.is_some() {
        pending_blocks.clear();
        return Ok(());
    }
    let blocks = std::mem::take(pending_blocks);
    let blocks_of_the_thread = blocks.clone();
    let started_as = stretches.stretches_started;
    stretches.stretches_started += 1;
    stretches.stretches_in_flight += 1;
    stretches
        .awaiting_the_ledger
        .push_back(PairingBlocksAwaitingTheLedger::Stretch {
            started_as,
            blocks,
            conclusions: None,
        });
    let finished_stretches = finished_stretches.clone();
    scope.spawn(move || {
        // 接住 panic 交回去：收的那一头等在飞的都收尾之后照原样接着抛，判红的消息不变
        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            judge_the_blocks(&blocks_of_the_thread)
        }));
        finished_stretches
            .send(FinishedPairingStretch { started_as, judged })
            .expect("收的那一头活到这个作用域收尾之后");
    });
    stretches.wait_for_a_free_place_among_the_stretches_in_flight()
}

/// 判完的一段逐块存进库。
///
/// # Errors
/// 库写失败。
fn store_the_judged_pairing_blocks(
    library: &CrashLibrary,
    key_of_the_block: &dyn Fn(&std::ops::Range<u64>) -> PairingConclusionKey,
    blocks: &[std::ops::Range<u64>],
    conclusions: &[PairingJudged],
) -> Result<(), String> {
    for (ordinals, judged) in blocks.iter().zip(conclusions) {
        library
            .lock()
            .store_pairing_conclusion(
                &key_of_the_block(ordinals),
                &StoredPairingConclusion {
                    conclusion: judged.conclusion.clone(),
                    agreement: judged.agreement,
                    circumstances: JudgementCircumstances::of_this_process_now(),
                },
            )
            .map_err(|error| format!("对拍块结论写不进库：{error:?}"))?;
    }
    Ok(())
}

/// 按序号次序把 `[0, state_count)` 按 `states_per_block` 切块走一遍：落在这一趟走的组之外的、归别的份的各数各的、不碰（前面攒着的先交出去判，一段块要连着）；
/// 库里有两边一致的结论就复用，没有的攒成一段连着的块交出去判。判那一侧不再往下判了就收手。
///
/// # Errors
/// 库读写失败。
#[allow(
    clippy::too_many_arguments,
    reason = "走一遍块的输入就是这几样：作用域、这一趟的账、流的输入摘要、状态数、块宽、一次判几块、这一趟领哪些块、判一段块、交回判完的那一头；收成一个类型只为让签名变短"
)]
fn walk_the_pairing_blocks_of_the_flow<'scope, 'environment: 'scope>(
    scope: &'scope std::thread::Scope<'scope, 'environment>,
    stretches: &mut PairingStretchesOfTheRun<'_>,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    most_blocks_judged_together: std::num::NonZeroUsize,
    share: &PairingBlocksShare,
    judge_the_blocks: &'environment JudgeOfAStretchOfPairingBlocks<'environment>,
    finished_stretches: &mpsc::Sender<FinishedPairingStretch>,
) -> Result<(), String> {
    let blocks_of_a_group =
        u64::try_from(most_blocks_judged_together.get()).expect("一次判的块数装得进 u64");
    let mut pending_blocks: Vec<std::ops::Range<u64>> =
        Vec::with_capacity(most_blocks_judged_together.get());
    let mut block_sequence = 0u64;
    let mut block_start = 0u64;
    while block_start < state_count && stretches.judging_stopped.is_none() {
        let ordinals = block_start
            ..block_start
                .saturating_add(states_per_block.get())
                .min(state_count);
        block_start = ordinals.end;
        let group_of_blocks = block_sequence / blocks_of_a_group;
        block_sequence += 1;
        if !share.groups_of_this_run_hold(group_of_blocks) {
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
            stretches
                .blocks_of_the_flow
                .blocks_outside_the_groups_of_this_run += 1;
            continue;
        }
        if !share.claims_this_group_of_pairing_blocks(flow_inputs_digest, group_of_blocks) {
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
            stretches.blocks_of_the_flow.blocks_left_to_other_shares += 1;
            continue;
        }
        let stored_conclusion = stretches
            .library
            .lock()
            .read_pairing_conclusion(&(stretches.key_of_the_block)(&ordinals))
            .map_err(|error| format!("读对拍块结论失败：{error:?}"))?
            .filter(stored_pairing_conclusion_is_reused);
        if let Some(stored) = stored_conclusion {
            // 复用的这一块排在攒着的那一段后面：总账按序号次序收
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
            stretches.wait_for_room_for_blocks_awaiting_the_ledger(1)?;
            stretches
                .awaiting_the_ledger
                .push_back(PairingBlocksAwaitingTheLedger::Reused {
                    ordinals,
                    conclusion: stored.conclusion,
                });
            stretches.take_the_blocks_at_the_head_of_the_queue();
            stretches.note_the_block_conclusions_held();
            continue;
        }
        pending_blocks.push(ordinals);
        if pending_blocks.len() >= most_blocks_judged_together.get() {
            start_judging_the_pending_pairing_blocks(
                scope,
                stretches,
                &mut pending_blocks,
                judge_the_blocks,
                finished_stretches,
            )?;
        }
    }
    start_judging_the_pending_pairing_blocks(
        scope,
        stretches,
        &mut pending_blocks,
        judge_the_blocks,
        finished_stretches,
    )
}

/// 按块对拍（「必须要剪枝和复用」）：`[0, state_count)` 按 `states_per_block` 切块，按序号次序一块块走——不归 `share` 这一份的只数、不碰；
/// 库里有这一块两边一致的结论（键是 [`pairing_block_inputs_digest`]）就复用、不判；记着不一致（红）的与没有的一样现判、照同一把键
/// 存回（红的照样存：记红、换掉原有的那一行）。没有的攒成一段连着的块（至多 `most_blocks_judged_together` 块），
/// 在一条线程上交 `judge_the_blocks` 一次判完，至多 `stretches_judged_at_once` 段一起判（头一段判完之前只有它一段，
/// [`PairingStretchesOfTheRun::stretches_allowed_in_flight_now`]）；判完逐块存进库。每一块的结论（复用的与现判的）按序号次序交
/// `take_a_block_conclusion` 折进调用方的总账，交完就丢：手里同一时刻至多压着 `stretches_judged_at_once` × `most_blocks_judged_together` 块的结论，
/// 与整条流有几块无关。有一段交 `None`（判那一侧报「本次未跑」）：不再起新的段，
/// 在飞的等它们收尾、判完的照存，交 `Ok(None)`，交 `None` 的那一段一块都不存。有一段 panic：同样等在飞的收尾，再照原样往外抛。
///
/// # Errors
/// 库读写失败。
///
/// # Panics
/// `judge_the_blocks` 交回的结论块数与交给它的块数不同；`judge_the_blocks` panic。
#[allow(
    clippy::too_many_arguments,
    reason = "按块对拍的输入就是这几样：库、流的输入摘要、状态数、块宽、一次判几块、几段一起判、这一趟领哪些块、判一段块、收一块结论；收成一个类型只为让签名变短"
)]
pub fn pairing_conclusions_of_the_blocks_reused_or_judged(
    library: &CrashLibrary,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    states_per_block: std::num::NonZeroU64,
    most_blocks_judged_together: std::num::NonZeroUsize,
    stretches_judged_at_once: std::num::NonZeroUsize,
    share: &PairingBlocksShare,
    judge_the_blocks: &JudgeOfAStretchOfPairingBlocks<'_>,
    take_a_block_conclusion: &mut dyn FnMut(&std::ops::Range<u64>, &[u8]),
) -> Result<Option<PairingBlocksOfTheFlow>, String> {
    let key_of_the_block = |ordinals: &std::ops::Range<u64>| {
        pairing_conclusion_key_of_the_inputs(&pairing_block_inputs_digest(
            flow_inputs_digest,
            ordinals,
        ))
    };
    let (finished_stretches_sender, finished_stretches) = mpsc::channel();
    let mut stretches = PairingStretchesOfTheRun {
        library,
        key_of_the_block: &key_of_the_block,
        take_a_block_conclusion,
        blocks_of_the_flow: PairingBlocksOfTheFlow {
            blocks_reused: 0,
            blocks_judged_now: 0,
            blocks_left_to_other_shares: 0,
            blocks_outside_the_groups_of_this_run: 0,
            most_block_conclusions_held_at_once: 0,
        },
        stretches_judged_at_once: stretches_judged_at_once.get(),
        most_blocks_awaiting_the_ledger: stretches_judged_at_once.get()
            * most_blocks_judged_together.get(),
        awaiting_the_ledger: VecDeque::new(),
        stretches_started: 0,
        stretches_finished: 0,
        stretches_in_flight: 0,
        judging_stopped: None,
        finished_stretches,
    };
    std::thread::scope(|scope| {
        let walked = walk_the_pairing_blocks_of_the_flow(
            scope,
            &mut stretches,
            flow_inputs_digest,
            state_count,
            states_per_block,
            most_blocks_judged_together,
            share,
            judge_the_blocks,
            &finished_stretches_sender,
        );
        walked.and(stretches.wait_for_every_stretch_in_flight())
    })?;
    match stretches.judging_stopped {
        Some(PairingJudgingStopped::Panicked(panicked)) => std::panic::resume_unwind(panicked),
        Some(PairingJudgingStopped::NotRun) => Ok(None),
        None => Ok(Some(stretches.blocks_of_the_flow)),
    }
}

#[cfg(test)]
mod tests;
