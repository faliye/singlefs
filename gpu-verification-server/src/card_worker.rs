//! 每张卡一条工作线程：照这一波的计划先换出、再装上判器，按次序判各批，每判完一批就把结论交回服务主循环，整轮做完报一声空出来了。
//! 判器按流的键缓存在 [`JudgesOnACard`] 的实现里（真卡上是 `crate::gpu_judges_on_a_card::GpuJudgesOnACard`；不上卡的测试用替身），
//! 不在计划里换出的判器一直留着，下一波派回这条流时不重传表、不重编管线。
//! 一批里几截序号首尾相接的，并成一段交判器判（一段在判器一次派活装得下时就是一次派活）。判的时候 panic（卡丢了、驱动报错）：
//! 报这张卡坏了、交回原因，这条线程不再收活。

use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use crate::batch_scheduling::{Batch, BatchPiece, CardPlanOfTheWave};
use crate::card_memory_ledger::CardIndex;
use crate::pending_pool::StateVerdict;
use crate::stream_identity::StreamKey;

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

    /// 判这条流一段连着的序号，结论按序号次序。
    fn judge_ordinals(&mut self, stream: StreamKey, ordinals: Range<u64>) -> Vec<StateVerdict>;
}

/// 交给一张卡的一波：计划，与计划里每个要装的判器的材料（与 `plan.loads` 同序）。
pub struct WaveForTheCard<Materials> {
    pub plan: CardPlanOfTheWave,
    pub materials_of_the_loads: Vec<Arc<Materials>>,
}

pub enum CardWorkerCommand<Materials> {
    CarryOutTheWave(WaveForTheCard<Materials>),
    Stop,
}

/// 工作线程交回主循环的。
#[derive(Debug)]
pub enum CardWorkerReport {
    /// 一批判完：每一截与它的结论。
    BatchJudged {
        card: CardIndex,
        pieces: Vec<(BatchPiece, Vec<StateVerdict>)>,
    },
    /// 这一波做完，卡空出来了。
    RoundFinished { card: CardIndex },
    /// 判的时候 panic 了：这张卡不再收活。
    CardFailed { card: CardIndex, reason: String },
}

/// 一张卡的工作线程：收一波做一波，收到 `Stop` 或命令那一头关了就退出。
pub fn card_worker_loop<Judges: JudgesOnACard>(
    card: CardIndex,
    judges: &mut Judges,
    commands: &Receiver<CardWorkerCommand<Judges::StreamMaterials>>,
    report: &mut dyn FnMut(CardWorkerReport),
) {
    while let Ok(command) = commands.recv() {
        let wave = match command {
            CardWorkerCommand::CarryOutTheWave(wave) => wave,
            CardWorkerCommand::Stop => return,
        };
        assert_eq!(wave.plan.card, card, "这一波排给的不是这张卡");
        let carried_out = catch_unwind(AssertUnwindSafe(|| {
            carry_out_the_wave(judges, &wave, &mut *report);
        }));
        if let Err(panic) = carried_out {
            report(CardWorkerReport::CardFailed {
                card,
                reason: panic_message(panic.as_ref()),
            });
            return;
        }
        report(CardWorkerReport::RoundFinished { card });
    }
}

fn carry_out_the_wave<Judges: JudgesOnACard>(
    judges: &mut Judges,
    wave: &WaveForTheCard<Judges::StreamMaterials>,
    report: &mut dyn FnMut(CardWorkerReport),
) {
    assert_eq!(
        wave.plan.loads.len(),
        wave.materials_of_the_loads.len(),
        "要装的判器与交来的材料份数对不上"
    );
    for stream in &wave.plan.evictions {
        judges.evict_the_judge(*stream);
    }
    for (load, materials) in wave.plan.loads.iter().zip(&wave.materials_of_the_loads) {
        judges.load_the_judge(load.stream, materials, load.states_per_dispatch);
    }
    for batch in &wave.plan.batches {
        let pieces = judge_the_batch(judges, batch);
        report(CardWorkerReport::BatchJudged {
            card: wave.plan.card,
            pieces,
        });
    }
}

/// 判一批：序号首尾相接的几截并成一段判，再按截拆回。
fn judge_the_batch<Judges: JudgesOnACard>(
    judges: &mut Judges,
    batch: &Batch,
) -> Vec<(BatchPiece, Vec<StateVerdict>)> {
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
        let mut verdicts = judges
            .judge_ordinals(batch.stream, ordinals.clone())
            .into_iter();
        assert_eq!(
            verdicts.len(),
            usize::try_from(ordinals.end - ordinals.start).expect("装得进 usize"),
            "判器交回的结论条数与这一段的态数对不上"
        );
        for piece in run {
            let count =
                usize::try_from(piece.ordinals.end - piece.ordinals.start).expect("装得进 usize");
            judged.push((piece.clone(), verdicts.by_ref().take(count).collect()));
        }
        run_start = run_end;
    }
    judged
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = panic.downcast_ref::<String>() {
        return text.clone();
    }
    "判的时候 panic 了，panic 的内容不是字符串".to_string()
}
