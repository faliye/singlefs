//! 整理夹着回退那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`）的历史与写死的几段历史在 harness 的执行器上跑、
//! 拷下观察者交出的镜像：`crash_given_states_of_every_step_of_the_compaction_and_rollback_histories.rs` 的各条用例与
//! `crash_judge_gpu_tables_at_their_capacity.rs` 里一版分配记录多于 4096 条的那条用例共用（种子区间只在这里定一处）。
//! 只依赖 `singlefs-harness`，不依赖 checker 档。

use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GeneratedHistory, GenerationWeights,
    HistoryExecution, HistoryOperation, HistoryRun, HistorySeed,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// 整理夹着回退那一组比重的种子区间：从种子基往后的这一段（整理那一组在偏移 50000 起，这一段在它之后）。
pub const SEED_OFFSET_OF_THE_COMPACTION_AND_ROLLBACK_SAMPLING: u64 = 60_000;

/// 一段历史在 harness 的执行器上跑完：执行器交回的这一趟，与交给观察者、要过池级 checker 的那几张镜像（表名、镜像）。
pub struct HistoryWithItsStepImages {
    pub run: HistoryRun,
    pub step_images: Vec<(String, MemoryPool)>,
}

/// 在两块 4 GiB 的盘上跑 `history`，`image_is_judged` 认下的那几步交给观察者的镜像拷下来（表名以 `table_name_prefix` 起头、接这一步的位置）。
/// 观察者在每一步没判出失败时调（起点之后一次、之后每一步一次），交的就是 harness 档原先就地交池级 checker 的那一张。
pub fn run_the_history_keeping_the_step_images(
    history: &GeneratedHistory,
    table_name_prefix: &str,
    image_is_judged: &dyn Fn(Option<&HistoryOperation>) -> bool,
) -> HistoryWithItsStepImages {
    let mut step_images: Vec<(String, MemoryPool)> = Vec::new();
    let run = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation| {
            if image_is_judged(observation.operation) {
                step_images.push((
                    format!("{table_name_prefix}/step-image/{:?}", observation.position),
                    observation.image.clone(),
                ));
            }
        },
    );
    HistoryWithItsStepImages { run, step_images }
}

/// 种子区间里偏移 `offset` 那一段的种子。
#[must_use]
pub fn seed_of_the_compaction_and_rollback_sampling(offset: u64) -> u64 {
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
        .wrapping_add(SEED_OFFSET_OF_THE_COMPACTION_AND_ROLLBACK_SAMPLING)
        .wrapping_add(offset)
}

/// 那一组比重在种子 `seed` 下生成的 `operations` 步历史。
#[must_use]
pub fn compaction_and_rollback_history(seed: u64, operations: usize) -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(seed),
        operations,
        &GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK,
    )
}

/// 观察者交出的镜像只拷回退那几步之后的（与 harness 档原先那一条同一个取法：回退那一版的映射照抄现行那一版时红在回退那一步）。
#[must_use]
pub fn is_a_rollback_step(operation: Option<&HistoryOperation>) -> bool {
    matches!(operation, Some(HistoryOperation::RollBackWhileMounted(_)))
}
