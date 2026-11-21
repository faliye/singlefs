//! 每张卡一条工作线程：照这一波的计划先换出、再装上判器，按次序判各批，每判完一批就把结论交回服务主循环，整轮做完报一声空出来了。
//! 判器按流的键缓存在 [`JudgesOnACard`] 的实现里（真卡上是 `crate::gpu_judges_on_a_card::GpuJudgesOnACard`；不上卡的测试用替身），
//! 不在计划里换出的判器一直留着，下一波派回这条流时不重传表、不重编管线。
//! 一批里几截序号首尾相接的，并成一段交判器判（一段在判器一次派活装得下时就是一次派活）。每装上一个判器报一声（主循环照它打装判器那一行）。
//!
//! 接住 Out of Memory：判的时候（或核的时候）panic，先问判器缓存这张卡的设备怎样（[`JudgesOnACard::after_a_panic`]，不认 panic 的字句）：
//! - 设备没丢、这一回记过 Out of Memory：判器缓存已丢掉卡上全部判器；丢掉的缓冲与上一波新表的暂存还挂在 wgpu 的待写列表里，要等下一次提交做完
//!   才还给驱动，所以先在这张卡上做一次提交、等它做完（[`JudgesOnACard::settle_after_running_out_of_memory`]），再报一声接住了
//!   （交回丢了几个判器与 panic 的那一句），接着报空出来了、照常收下一波；这一波没交回结论的截由主循环撤回重排（`crate::service_loop` 文件头「接住 Out of Memory」）。
//!   那一次提交没等成的报这张卡坏了；
//! - 设备因显存不够丢了：判器缓存已重开这张卡的设备，报一声接住了（原因里写重开了设备），照常收下一波；重开没成的答设备丢了；
//! - 设备没丢、也没记过 Out of Memory（判法自己的 panic，例：装表时表超过绑定上限的断言）：判器缓存已丢掉卡上全部判器，报一声判法 panic 了
//!   （交回丢了几个判器、panic 时在做哪一步、panic 的那一句），接着报空出来了、照常收下一波：这是那一步那几块的毛病，不是卡的；
//! - 设备为别的原因丢了：报这张卡坏了（带设备丢没丢、PCI 总线号与原因），这条线程不再收活。
//!
//! panic 时在做哪一步（[`PanickedWhile`]）：换出、装哪一条流的判器、判哪一批（那一批的几截）、核哪一组核对块，每一步开始之前记下。
//!
//! 弄坏开关 out-of-memory-taken-for-a-card-failure 下设备没丢的 Out of Memory 也报卡坏、线程退出；card-not-settled-after-running-out-of-memory 下
//! 接住之后不做那一次提交、不等；device-lost-for-want-of-memory-taken-for-a-card-failure 下因显存不够丢了、已重开设备的也报卡坏；
//! judge-panic-taken-for-a-card-failure 下判法自己的 panic 也报卡坏。
//!
//! 判的账：每一段量挂钟（交判器到判器交回，这条线程这一段时间占着这张卡），按态数摊给段里各截；判器交回这一段派了几次活，
//! 段里每一块记一次（同一块在一段里有两截只记在头一截上）。服务按块、按卡打汇总行用它（`crate::serving_from_the_library` 文件头「日志」），
//! 不靠验证包逐次派活打的那两种行。
//!
//! 核对：主循环也可以派一组核对块（核对待判行那一族，`crate::service_loop` 文件头「核对」）给空着的卡：先换出主循环为它腾地方点名的判器，
//! 再照这一组的材料建一个核对器、按次序核每一块（[`JudgesOnACard::verify_the_facts`]，核完丢掉核对器），交回每块的结论位与每块分到的挂钟
//! （整组的挂钟按态数摊），再报一声空出来了。核的时候 panic 与判的时候 panic 同一套：显存不够的接住，判法自己 panic 的报判法 panic 了，
//! 设备为别的原因丢了才报这张卡坏了。

use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;

use crate::batch_scheduling::{Batch, BatchPiece, CardPlanOfTheWave, PendingBlockIdentifier};
use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_memory_ledger::CardIndex;
use crate::facts_verifier_family::FactsVerifierMaterial;
use crate::pending_pool::{CardWorkOnAPiece, StateVerdict};
use crate::stream_identity::StreamKey;

/// 判器判一段交回的：按序号次序的结论，与判这一段在卡上派了几次活（判器自己数的）。
#[derive(Debug)]
pub struct JudgedOrdinals {
    pub verdicts: Vec<StateVerdict>,
    pub dispatches: u64,
}

/// 一截判出的结论与它分到的账。
#[derive(Debug)]
pub struct JudgedPiece {
    pub piece: BatchPiece,
    pub verdicts: Vec<StateVerdict>,
    pub work: CardWorkOnAPiece,
}

/// 判的时候 panic 之后这张卡的设备怎样（[`JudgesOnACard::after_a_panic`]，文件头「接住 Out of Memory」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardAfterAPanic {
    /// 设备没丢、这一回记过 Out of Memory：卡上全部判器已丢掉，`judges_dropped` 是丢了几个。
    RanOutOfMemory { judges_dropped: usize },
    /// 设备因显存不够丢了：卡上全部判器已丢掉、这张卡的设备已重开，`reason` 说丢的原因与重开了。
    DeviceReopenedAfterRunningOutOfMemory {
        judges_dropped: usize,
        reason: String,
    },
    /// 设备为别的原因丢了（或因显存不够丢了而重开没成）：`reason` 是记下的丢失原因。
    DeviceLost { reason: String },
    /// 设备没丢、也没记过 Out of Memory（判法自己的 panic）：卡上全部判器已丢掉，`judges_dropped` 是丢了几个。
    PanickedOtherwise { judges_dropped: usize },
}

/// panic 的那一刻工作线程在做哪一步（文件头「panic 时在做哪一步」）：主循环照它认是哪几块的毛病。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanickedWhile {
    /// 换出计划点名的判器。
    Evicting,
    /// 装这条流的判器。
    Loading(StreamKey),
    /// 判这一批（这几截）。
    Judging(Vec<BatchPiece>),
    /// 核这一组核对块。
    Verifying(FactsJobIdentifier),
}

/// 一张卡上的判器缓存：按流的键存，照计划装上、换出、判。
pub trait JudgesOnACard {
    /// 装一个判器要的东西（真卡上是读回的打包判表与内核源码）。
    type StreamMaterials: Send + Sync;

    /// 装上这条流的判器，一次派活至多 `states_per_dispatch` 个态。
    fn load_the_judge(
        &mut self,
        stream: StreamKey,
        materials: &Self::StreamMaterials,
        states_per_dispatch: u64,
    );

    /// 换出这条流的判器（表与草稿区交回驱动）。
    fn evict_the_judge(&mut self, stream: StreamKey);

    /// 判这条流一段连着的序号，结论按序号次序，连同这一段派了几次活。
    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> JudgedOrdinals;

    /// 照 `material` 在这张卡上建一个核对器，按次序核 `blocks` 里每一块，交回每块每个态的结论位（与 `blocks` 同序，每块按态序号次序）；
    /// 核完丢掉核对器，它借的显存随之还回去（主循环借给它之前照账腾过地方）。
    fn verify_the_facts(
        &mut self,
        material: &dyn FactsVerifierMaterial,
        blocks: &[Range<u64>],
    ) -> Vec<Vec<u32>>;

    /// 判（或核）的时候 panic 之后调（文件头「接住 Out of Memory」）：设备因显存不够丢了的，丢掉卡上全部判器、重开设备，交
    /// [`CardAfterAPanic::DeviceReopenedAfterRunningOutOfMemory`]（重开没成交 [`CardAfterAPanic::DeviceLost`]）；为别的原因丢了交
    /// [`CardAfterAPanic::DeviceLost`]；设备没丢而上一次 panic 之后（或造这一份之后）记过 Out of Memory 的，丢掉卡上全部判器、交
    /// [`CardAfterAPanic::RanOutOfMemory`]；别的丢掉卡上全部判器、交 [`CardAfterAPanic::PanickedOtherwise`]。
    /// 缺省交 `PanickedOtherwise`、丢了 0 个（不看设备、不留判器的替身）。
    fn after_a_panic(&mut self) -> CardAfterAPanic {
        CardAfterAPanic::PanickedOtherwise { judges_dropped: 0 }
    }

    /// 接住设备没丢的 Out of Memory、丢掉全部判器之后调（文件头「接住 Out of Memory」）：在这张卡上做一次提交、等它做完，
    /// 让 wgpu 待写列表里挂着的暂存与丢掉的缓冲还给驱动，再收下一波。缺省什么都不做（不上卡的替身）。
    ///
    /// # Errors
    /// 那一次提交没等成：交原因。
    fn settle_after_running_out_of_memory(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// 这张卡的 PCI 总线号（卡坏那一行写它）；不在真卡上的是 `None`。
    fn pci_bus_identifier(&self) -> Option<String> {
        None
    }
}

/// 主循环派出去的一组核对块的号（主循环给，工作线程交回时带着）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactsJobIdentifier(pub u64);

/// 交给一张卡的一组核对块：先换出哪几条流的判器（主循环为它腾的地方）、建核对器的材料、按次序核的几块。
pub struct FactsJobForTheCard {
    pub job: FactsJobIdentifier,
    pub evictions: Vec<StreamKey>,
    pub material: Arc<dyn FactsVerifierMaterial>,
    pub blocks: Vec<Range<u64>>,
}

/// 交给一张卡的一波：计划，与计划里每个要装的判器的材料（与 `plan.loads` 同序）。
pub struct WaveForTheCard<Materials> {
    pub plan: CardPlanOfTheWave,
    pub materials_of_the_loads: Vec<Arc<Materials>>,
}

pub enum CardWorkerCommand<Materials> {
    CarryOutTheWave(WaveForTheCard<Materials>),
    VerifyTheFacts(FactsJobForTheCard),
    Stop,
}

/// 工作线程交回主循环的。
#[derive(Debug)]
pub enum CardWorkerReport {
    /// 一批判完：每一截、它的结论与它分到的账。
    BatchJudged {
        card: CardIndex,
        pieces: Vec<JudgedPiece>,
    },
    /// 一组核对块核完：每块的结论位（与派来的块同序）与每块分到的卡上挂钟（纳秒）。
    FactsVerified {
        card: CardIndex,
        job: FactsJobIdentifier,
        bits_of_each_block: Vec<Vec<u32>>,
        card_nanoseconds_of_each_block: Vec<u64>,
    },
    /// 装上了一个判器（每装一个报一次，次序同计划里的 `loads`）。
    JudgeLoaded {
        card: CardIndex,
        stream: StreamKey,
        states_per_dispatch: u64,
    },
    /// 判（或核）的时候显存不够：设备没丢、那一次提交等完了，或设备因显存不够丢了、已重开（文件头「接住 Out of Memory」）；
    /// 卡上的判器全丢了，这一轮没判完的由主循环撤回；随后照样报空出来了。
    OutOfMemoryCaught {
        card: CardIndex,
        judges_dropped: usize,
        reason: String,
    },
    /// 判（或核）的时候判法自己 panic 了、设备没丢（文件头「接住 Out of Memory」）：卡上的判器全丢了，`panicked_while` 是 panic 时在做哪一步；
    /// 随后照样报空出来了。
    JudgePanicked {
        card: CardIndex,
        judges_dropped: usize,
        panicked_while: PanickedWhile,
        reason: String,
    },
    /// 这一波（或这一组核对块）做完，卡空出来了。
    RoundFinished { card: CardIndex },
    /// 设备为别的原因丢了（或接住之后那一次提交没等成）：这张卡不再收活。
    CardFailed {
        card: CardIndex,
        reason: String,
        device_lost: bool,
        pci_bus_identifier: Option<String>,
    },
}

/// 一张卡的工作线程：收一波做一波（或一组核对块），收到 `Stop` 或命令那一头关了就退出；panic 了照文件头「接住 Out of Memory」办。
pub fn card_worker_loop<Judges: JudgesOnACard>(
    card: CardIndex,
    judges: &mut Judges,
    commands: &Receiver<CardWorkerCommand<Judges::StreamMaterials>>,
    break_switches: &BreakSwitches,
    report: &mut dyn FnMut(CardWorkerReport),
) {
    while let Ok(command) = commands.recv() {
        let mut panicked_while = PanickedWhile::Evicting;
        let carried_out = match command {
            CardWorkerCommand::CarryOutTheWave(wave) => {
                assert_eq!(wave.plan.card, card, "这一波排给的不是这张卡");
                catch_unwind(AssertUnwindSafe(|| {
                    carry_out_the_wave(judges, &wave, &mut *report, &mut panicked_while);
                }))
            }
            CardWorkerCommand::VerifyTheFacts(job) => catch_unwind(AssertUnwindSafe(|| {
                verify_the_facts_of_the_job(card, judges, &job, &mut *report, &mut panicked_while);
            })),
            CardWorkerCommand::Stop => return,
        };
        if let Err(panic) = carried_out {
            let reason = panic_message(panic.as_ref());
            if !took_the_panic(
                card,
                judges,
                break_switches,
                &mut *report,
                panicked_while,
                reason,
            ) {
                return;
            }
        }
        report(CardWorkerReport::RoundFinished { card });
    }
}

/// 一波（或一组核对块）panic 了之后照文件头「接住 Out of Memory」报：接住了（这张卡照常收活）交 `true`，报了卡坏（线程该退出）交 `false`。
fn took_the_panic<Judges: JudgesOnACard>(
    card: CardIndex,
    judges: &mut Judges,
    break_switches: &BreakSwitches,
    report: &mut dyn FnMut(CardWorkerReport),
    panicked_while: PanickedWhile,
    reason: String,
) -> bool {
    let pci_bus_identifier = judges.pci_bus_identifier();
    let failed = |why: String, device_lost: bool| CardWorkerReport::CardFailed {
        card,
        reason: why,
        device_lost,
        pci_bus_identifier: pci_bus_identifier.clone(),
    };
    match judges.after_a_panic() {
        CardAfterAPanic::RanOutOfMemory { judges_dropped }
            if !break_switches.is_on(BreakSwitch::OutOfMemoryTakenForACardFailure) =>
        {
            if !break_switches.is_on(BreakSwitch::CardNotSettledAfterRunningOutOfMemory) {
                if let Err(unsettled) = judges.settle_after_running_out_of_memory() {
                    report(failed(
                        format!(
                            "{}（接住 Out of Memory 之后在这张卡上做一次提交、等它做完，没等成：{unsettled}）",
                            reason.trim_end()
                        ),
                        false,
                    ));
                    return false;
                }
            }
            report(CardWorkerReport::OutOfMemoryCaught {
                card,
                judges_dropped,
                reason,
            });
            true
        }
        CardAfterAPanic::DeviceReopenedAfterRunningOutOfMemory {
            judges_dropped,
            reason: reopened,
        } if !break_switches.is_on(BreakSwitch::DeviceLostForWantOfMemoryTakenForACardFailure) => {
            report(CardWorkerReport::OutOfMemoryCaught {
                card,
                judges_dropped,
                reason: format!("{}（{reopened}）", reason.trim_end()),
            });
            true
        }
        CardAfterAPanic::PanickedOtherwise { judges_dropped }
            if !break_switches.is_on(BreakSwitch::JudgePanicTakenForACardFailure) =>
        {
            report(CardWorkerReport::JudgePanicked {
                card,
                judges_dropped,
                panicked_while,
                reason,
            });
            true
        }
        CardAfterAPanic::DeviceReopenedAfterRunningOutOfMemory {
            reason: reopened, ..
        } => {
            report(failed(
                format!("{}（设备丢了：{reopened}）", reason.trim_end()),
                true,
            ));
            false
        }
        CardAfterAPanic::DeviceLost { reason: lost } => {
            report(failed(
                format!("{}（设备丢了：{lost}）", reason.trim_end()),
                true,
            ));
            false
        }
        CardAfterAPanic::RanOutOfMemory { .. } | CardAfterAPanic::PanickedOtherwise { .. } => {
            report(failed(reason, false));
            false
        }
    }
}

fn carry_out_the_wave<Judges: JudgesOnACard>(
    judges: &mut Judges,
    wave: &WaveForTheCard<Judges::StreamMaterials>,
    report: &mut dyn FnMut(CardWorkerReport),
    panicked_while: &mut PanickedWhile,
) {
    assert_eq!(
        wave.plan.loads.len(),
        wave.materials_of_the_loads.len(),
        "要装的判器与交来的材料份数对不上"
    );
    *panicked_while = PanickedWhile::Evicting;
    for stream in &wave.plan.evictions {
        judges.evict_the_judge(*stream);
    }
    for (load, materials) in wave.plan.loads.iter().zip(&wave.materials_of_the_loads) {
        *panicked_while = PanickedWhile::Loading(load.stream);
        judges.load_the_judge(load.stream, materials, load.states_per_dispatch);
        report(CardWorkerReport::JudgeLoaded {
            card: wave.plan.card,
            stream: load.stream,
            states_per_dispatch: load.states_per_dispatch,
        });
    }
    for batch in &wave.plan.batches {
        *panicked_while = PanickedWhile::Judging(batch.pieces.clone());
        let pieces = judge_the_batch(judges, batch);
        report(CardWorkerReport::BatchJudged {
            card: wave.plan.card,
            pieces,
        });
    }
}

/// 核一组核对块（文件头「核对」）：换出点名的判器，核，整组的挂钟按态数摊给各块，交回。
///
/// # Panics
/// 核对器交回的块数或某一块的结论位条数与派来的对不上；同 [`JudgesOnACard::verify_the_facts`]。
fn verify_the_facts_of_the_job<Judges: JudgesOnACard>(
    card: CardIndex,
    judges: &mut Judges,
    job: &FactsJobForTheCard,
    report: &mut dyn FnMut(CardWorkerReport),
    panicked_while: &mut PanickedWhile,
) {
    *panicked_while = PanickedWhile::Evicting;
    for stream in &job.evictions {
        judges.evict_the_judge(*stream);
    }
    *panicked_while = PanickedWhile::Verifying(job.job);
    let verifying_began = Instant::now();
    let bits_of_each_block = judges.verify_the_facts(job.material.as_ref(), &job.blocks);
    let job_nanoseconds = u64::try_from(verifying_began.elapsed().as_nanos()).unwrap_or(u64::MAX);
    assert_eq!(
        bits_of_each_block.len(),
        job.blocks.len(),
        "核对器交回的块数与派来的对不上"
    );
    for (bits, ordinals) in bits_of_each_block.iter().zip(&job.blocks) {
        assert_eq!(
            u64::try_from(bits.len()).ok(),
            Some(ordinals.end - ordinals.start),
            "核对器交回块 {ordinals:?} 的结论位条数与它的态数对不上"
        );
    }
    let states_of_the_job: u64 = job
        .blocks
        .iter()
        .map(|ordinals| ordinals.end - ordinals.start)
        .sum();
    let card_nanoseconds_of_each_block = job
        .blocks
        .iter()
        .map(|ordinals| {
            if states_of_the_job == 0 {
                0
            } else {
                share_of_the_run(
                    job_nanoseconds,
                    ordinals.end - ordinals.start,
                    states_of_the_job,
                )
            }
        })
        .collect();
    report(CardWorkerReport::FactsVerified {
        card,
        job: job.job,
        bits_of_each_block,
        card_nanoseconds_of_each_block,
    });
}

/// 一段的挂钟按态数摊给其中一截（向下取整，各截之和不超过整段）。
///
/// # Panics
/// `run_states` 是 0（一段至少一截、一截至少一个态）。
fn share_of_the_run(run_nanoseconds: u64, piece_states: u64, run_states: u64) -> u64 {
    assert!(run_states > 0, "一段至少一个态");
    let share = u128::from(run_nanoseconds) * u128::from(piece_states) / u128::from(run_states);
    u64::try_from(share).expect("一截的份额不超过整段的挂钟")
}

/// 判一批：序号首尾相接的几截并成一段判，再按截拆回，每截带上它分到的账（文件头「判的账」）。
fn judge_the_batch<Judges: JudgesOnACard>(judges: &mut Judges, batch: &Batch) -> Vec<JudgedPiece> {
    let mut judged = Vec::with_capacity(batch.pieces.len());
    let mut run_start = 0;
    while run_start < batch.pieces.len() {
        let mut run_end = run_start + 1;
        while run_end < batch.pieces.len()
            && batch.pieces[run_end].ordinals.start == batch.pieces[run_end - 1].ordinals.end
        {
            run_end += 1;
        }
        let run = &batch.pieces[run_start..run_end];
        let ordinals = run[0].ordinals.start..run[run.len() - 1].ordinals.end;
        let run_states = ordinals.end - ordinals.start;
        let judging_began = Instant::now();
        let JudgedOrdinals {
            verdicts,
            dispatches,
        } = judges.judge_ordinals(batch.stream, ordinals.clone());
        let run_nanoseconds = u64::try_from(judging_began.elapsed().as_nanos()).unwrap_or(u64::MAX);
        assert_eq!(
            verdicts.len(),
            usize::try_from(run_states).expect("装得进 usize"),
            "判器交回的结论条数与这一段的态数对不上"
        );
        let mut verdicts = verdicts.into_iter();
        let mut blocks_given_the_dispatches: Vec<PendingBlockIdentifier> =
            Vec::with_capacity(run.len());
        for piece in run {
            let piece_states = piece.ordinals.end - piece.ordinals.start;
            let count = usize::try_from(piece_states).expect("装得进 usize");
            let dispatches_of_the_piece = if blocks_given_the_dispatches.contains(&piece.block) {
                0
            } else {
                blocks_given_the_dispatches.push(piece.block);
                dispatches
            };
            judged.push(JudgedPiece {
                piece: piece.clone(),
                verdicts: verdicts.by_ref().take(count).collect(),
                work: CardWorkOnAPiece {
                    dispatches: dispatches_of_the_piece,
                    card_nanoseconds: share_of_the_run(run_nanoseconds, piece_states, run_states),
                },
            });
        }
        run_start = run_end;
    }
    judged
}

/// panic 带的那一句（`&str` 或 `String`；别的写一句说明）。
pub(crate) fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = panic.downcast_ref::<String>() {
        return text.clone();
    }
    "判的时候 panic 了，panic 的内容不是字符串".to_string()
}
