//! 池级 checker 经给定状态引擎在一张盘上的判定与标签，与一次发布落在根环上的位置。

use super::*;

/// 这份装置判池级 checker 的给定状态引擎（进程里一份，第一次用时起；能走两段的都走两段）：每张要判的盘作一张给定状态表，
/// 录入段写进家目录下给定状态表那一类的判定库，判段从库里读回判（配了 GPU 且编进了 `gpu` 特性在卡上判，否则由 CPU 判器判），判过的复用。
/// 产物只收这份装置自己打的行：引擎不调 `finish`，不打收尾行。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_experiment() -> &'static GivenStatesEngine {
    static ENGINE: OnceLock<GivenStatesEngine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "e156_allocation_basis_counts::fixed".to_string(),
            },
            &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    })
}

/// 池级 checker 在这张盘上判 `invariant` 的结果：那张盘作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）交
/// [`given_states_engine_of_this_experiment`] 两段判。结论里只有判到位与违例位：判到而没违例是成立，违例带细节（这一趟 CPU 判的才是原句），
/// 没判到的「不适用」与「部分没判」分不开、一律记不适用（产物只打 holds / violated / not_applicable，入库产物里没有 partially_judged）。
///
/// # Panics
/// 引擎判不了这张表（不退回别处判）；这一趟只录、或这张表归别的机器判（实验装置不分两趟跑、不分片）。
pub(super) fn verdict(table_name: &str, pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
    let Some(position) = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|implemented| *implemented == invariant)
    else {
        return InvariantVerdict::NotApplicable("checker 没有报这一条");
    };
    let judged = given_states_engine_of_this_experiment()
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: pool,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| {
            panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
        });
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => panic!(
            "实验装置不分只录、只判两趟跑，也不分片：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与 SINGLEFS_CRASH_AMPLIFICATION_SHARE（{table_name}）"
        ),
    }
    let [conclusion] = judged.states.as_slice() else {
        panic!(
            "{table_name}：一个状态一份结论，交回 {} 份",
            judged.states.len()
        );
    };
    let bit = 1u128 << position;
    if conclusion.words.pool_checker_violated_bits & bit != 0 {
        let detail = conclusion
            .pool_checker_violations()
            .into_iter()
            .find(|(violated, _)| *violated == invariant)
            .map(|(_, detail)| detail)
            .expect("违例位置了，pool_checker_violations 就列着这一条");
        InvariantVerdict::Violated(detail)
    } else if conclusion.words.pool_checker_judged_bits & bit != 0 {
        InvariantVerdict::Holds
    } else {
        InvariantVerdict::NotApplicable(
            "池级 checker 没判这一条（不适用或部分没判；两段判的结论里这两种不分）",
        )
    }
}
pub(super) fn is_red(verdict: &InvariantVerdict) -> bool {
    matches!(verdict, InvariantVerdict::Violated(_))
}
pub(super) fn verdict_label(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "holds",
        InvariantVerdict::Violated(_) => "violated",
        InvariantVerdict::NotApplicable(_) => "not_applicable",
        InvariantVerdict::PartiallyJudged(_) => "partially_judged",
    }
}

/// 一次发布落在根环上的 (设备, 偏移)：`target_for_publish` + `slot_offset` 都是公开的几何函数，
/// 不是登记第 ① 条禁的「决定工作量/几何/门槛的常量」。
pub(super) fn root_slot_location(
    txg: CheckpointTxg,
    region_devices: &[DeviceIdentity; 3],
    spacing: u32,
    slots_per_region: RootRingSlotsPerRegion,
) -> (u32, u64) {
    let target = target_for_publish(txg, slots_per_region);
    let device = region_devices[usize::try_from(target.region).expect("区域号落在 3 以内")].0;
    (device, slot_offset(target, spacing).0)
}
