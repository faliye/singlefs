//! CPU 路线判一块：崩溃放量判段没配 GPU 时，一条核对线程判一块里的一段状态、交回存库之前的那几样。
//!
//! 它是判镜像的定义那一组判法单元（`judgement_of_the_crash_images`，`crate::judging_units_entries`）的入口之一：镜像判定的版本
//! 不分路线（「数据不与设备绑定」「GPU 判器代码进键」，`crate::crash_identity::judgement_units_key_with_the_gpu_judge`），
//! 所以 CPU 路线逐块判的代码一改，两条路线判过的块一起换版本、一起现判；不登进去，这里改了块照样复用。
//! 放在判器一侧、不放在流水线（`crate::crash_amplification`）里：判器的闭包里不许有流水线的单元
//! （`crate::judging_units_entries::PIPELINE_FILES_THE_JUDGES_MUST_NOT_REACH`）。谁调它、日志怎么打、结论怎么存都留在流水线，不进这一组的键。

use std::ops::Range;

use crate::crash::{
    judge_crash_state_range_of_the_read_back_flow_with_slices, CrashPointReplayVersionTable,
    CrashStatesOfTheReadBackFlow,
};
use crate::crash_judge_conclusions::{
    self_verified_system_configuration_slots_of_each_device, RecoveryWordsOfState,
    SelfVerifiedSystemConfigurationSlotsOfTheStates, StateCountsOfABlock,
};

/// 一块（或它的一段）留几条红状态的违例正文给门禁与人看。
pub const RED_TEXTS_KEPT: usize = 64;

/// CPU 路线判完的一块的一段（还没写库）：每个状态判红的各项、红状态的正文。
pub struct CpuJudgedBlock {
    /// 这一段在它那个崩溃点里的序号区间（点内序号）。
    pub local: Range<u64>,
    pub items_of_each_state: Vec<Vec<String>>,
    /// 逐状态的恢复字（`CrashStateJudgement::recovery_words`，判器一侧 `crate::crash_judge_conclusions`；不挂 `gpu` 特性，每个状态都有）。
    pub recovery_words_of_each_state: Vec<RecoveryWordsOfState>,
    pub state_counts: StateCountsOfABlock,
    /// 红状态的点内序号与违例正文，最多 [`RED_TEXTS_KEPT`] 条。
    pub red_texts: Vec<(u64, String)>,
    /// 逐状态的逐盘自证过的系统配置槽数（CPU 参照 `crate::crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`）。
    pub self_verified_system_configuration_slots_of_the_states:
        SelfVerifiedSystemConfigurationSlotsOfTheStates,
}

/// 判一个崩溃点里 `local` 那一段状态（一条核对线程）：状态取自调用方建好、各线程共用的那一份（`crash_states_of_the_flow`，
/// 枚举的表一个阶段只建一份）；`ordinals_of_the_point` 是这个点在整条流里的状态序号区间，`local` 按点内序号算。
///
/// # Panics
/// 整条流的状态数小于这个点的序号区间的末端（调用方给的点与流对不上）；一块的状态数装不进 `usize`。
#[must_use]
pub(crate) fn judge_a_piece_of_a_block_on_the_cpu(
    crash_states_of_the_flow: &CrashStatesOfTheReadBackFlow<'_>,
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    ordinals_of_the_point: Range<u64>,
    local: Range<u64>,
) -> CpuJudgedBlock {
    let mut items_of_each_state =
        Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
    // 恢复字的失败点码表在判器一侧、不挂 gpu 特性（`crate::crash_judge_conclusions`）：没编进卡的二进制判的 CPU 结论照样带恢复字
    let mut recovery_words_of_each_state: Vec<RecoveryWordsOfState> =
        Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
    let mut state_counts = StateCountsOfABlock::default();
    let mut red_texts = Vec::new();
    let mut self_verified_system_configuration_slots_of_the_states =
        SelfVerifiedSystemConfigurationSlotsOfTheStates::default();
    let ordinals =
        ordinals_of_the_point.start + local.start..ordinals_of_the_point.start + local.end;
    let stream_state_count = judge_crash_state_range_of_the_read_back_flow_with_slices(
        crash_states_of_the_flow,
        judged_root_index,
        versions,
        ordinals,
        &mut |ordinal, publish, image, judgement| {
            // 判段 CPU 那一路交回的逐盘自证槽数：两槽全废盘面观察者在判段读它（GPU 那一路是走读内核第 25 字）
            self_verified_system_configuration_slots_of_the_states.push_a_state(
                self_verified_system_configuration_slots_of_each_device(image),
            );
            let items: Vec<String> = judgement.red_items();
            if !items.is_empty() && red_texts.len() < RED_TEXTS_KEPT {
                red_texts.push((
                    ordinal - ordinals_of_the_point.start,
                    judgement.violation_text(image),
                ));
            }
            items_of_each_state.push(items);
            state_counts.count_the_cpu_judgement(judgement);
            state_counts.count_the_publish_of_a_state(publish);
            recovery_words_of_each_state.push(judgement.recovery_words());
        },
    );
    assert!(
        stream_state_count >= ordinals_of_the_point.end,
        "这一块的序号越过了整条流的状态数"
    );
    CpuJudgedBlock {
        local,
        items_of_each_state,
        recovery_words_of_each_state,
        state_counts,
        red_texts,
        self_verified_system_configuration_slots_of_the_states,
    }
}
