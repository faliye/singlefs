//! 池级 checker（O2）：已知坏镜像与经给定状态引擎判取样态。

use super::*;

// ───────────────────────────── 阳性对照 O2：已知坏镜像族第一条（登记 5.2） ─────────────────────────────

/// 把一个崩溃镜像物化成 `MemoryPool`：基镜像拷一份，按写表次序施加持久了的写。
fn materialized_memory_pool(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    let persisted_writes: Vec<RetainedWrite> = image
        .writes
        .iter()
        .zip(&image.persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write, _)| write.clone())
        .collect();
    pool.apply_writes(&persisted_writes);
    pool
}

/// 阳性对照 O2：把已知坏镜像族的第一条改法施加在这一态的物化镜像上，再交 `check_pool_image`（只为计时，名单走两段）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn pool_checker_on_the_first_known_bad_image(
    image: &CrashImage<'_>,
) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image(&first_known_bad_image_of(image))
}

/// 这一态的物化镜像上施加已知坏镜像族的第一条改法之后的那张盘（阳性对照 O2 判的那一张）。
fn first_known_bad_image_of(image: &CrashImage<'_>) -> MemoryPool {
    let clean = materialized_memory_pool(image);
    let entries = known_bad_images(&clean);
    let (_, apply_the_first_entry) = entries.first().expect("已知坏镜像表至少一条");
    let mut bad = clean.clone();
    apply_the_first_entry(&mut bad);
    bad
}

/// 这份装置判取样那几态 O2 名单的给定状态引擎（进程里一份，第一次用时起：O2 的名单按判到位与违例位能两段，
/// 计时与按件 panic 留 CPU）：每张要判的盘作一张给定状态表，录入段写进家目录下给定状态表那一类的判定库，判段从库里读回判
/// （配了 GPU 且编进了 `gpu` 特性在卡上判，否则由 CPU 判器判），判过的复用。产物只收这份装置自己打的行：引擎不调 `finish`，不打收尾行。
/// 照第一段装置（`e171_three_oracle_collusion.rs`）的同名函数。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_experiment(
) -> &'static singlefs_verification::crash_amplification::GivenStatesEngine {
    static ENGINE: std::sync::OnceLock<
        singlefs_verification::crash_amplification::GivenStatesEngine,
    > = std::sync::OnceLock::new();
    ENGINE.get_or_init(|| {
        singlefs_verification::crash_amplification::GivenStatesEngine::start_from_environment(
            singlefs_verification::crash_amplification::GivenStatesItem {
                item_name: "e171_three_oracle_collusion_after_the_oracle_fixes::fixed".to_string(),
            },
            &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    })
}

/// 取样的一态上 O2 判红的不变量名（按 `IMPLEMENTED_INVARIANTS` 的次序，与 [`violated_invariant_names`] 同一个次序），走两段：
/// 这一态的物化镜像（阳性对照那一态是改坏的那一张，[`first_known_bad_image_of`]）作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）
/// 交 [`given_states_engine_of_this_experiment`]。表名带流、序号与镜像摘要（同名的表内容就相同，判过的复用）。
///
/// 交回 `None` 是这一趟没判名单：引擎只录了这张表、交占位结论（配置开了 GPU 而这份二进制没编进 `gpu` 特性，或设了只录那一段），
/// 判归编进特性的那一趟；引擎开跑时已打了本次未跑那一行（`given_states_not_run_line_without_the_gpu_feature`），
/// 这里不退回 CPU 判，也不把占位当 panic 记进按件 panic 的计数。
///
/// # Panics
/// 引擎判不了这张表（不退回别处判）；这张表归别的机器判（实验装置不分片）。
pub(super) fn pool_checker_red_names_of_a_sampled_state_through_the_given_states_engine(
    stream_name: StreamName,
    ordinal: u64,
    image_digest: &str,
    image: &CrashImage<'_>,
    pool_checker_control: bool,
) -> Option<Vec<String>> {
    use singlefs_verification::crash_given_states::{
        GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
    };
    let (table_name, judged_image) = if pool_checker_control {
        (
            format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}/first-known-bad-entry",
                stream_name.directory_name()
            ),
            first_known_bad_image_of(image),
        )
    } else {
        (
            format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}",
                stream_name.directory_name()
            ),
            materialized_memory_pool(image),
        )
    };
    let judged = given_states_engine_of_this_experiment()
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.clone(),
            base: &judged_image,
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
        GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost => panic!(
            "实验装置不分片：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_SHARE（{table_name} 归别的机器判）"
        ),
    }
    let [conclusion] = judged.states.as_slice() else {
        panic!(
            "{table_name}：一个状态一份结论，交回 {} 份",
            judged.states.len()
        );
    };
    Some(
        conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(invariant, _)| invariant.to_string())
            .collect(),
    )
}
