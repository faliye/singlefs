//! 取样态的观测：录入段录表、判段判表的单测。

use super::*;

#[test]
fn sampled_states_on_several_threads_write_the_same_files_as_on_one_thread() {
    StreamToSampleInThePairing::new_pool_file_creation()
        .assert_several_threads_write_the_same_files_as_one_thread(
            &PoolCheckerTablesKeptInMemory::judging_with(
                &pool_checker_red_names_of_a_given_image_on_the_cpu,
            ),
        );
}

/// 同一条对拍走产品路径：每态一张给定状态表录进判定库、判段交引擎，表名与表的内容与产品路径相同。
#[cfg(feature = "gpu")]
#[test]
fn sampled_states_through_the_given_states_engine_on_several_threads_write_the_same_files_as_on_one_thread(
) {
    StreamToSampleInThePairing::new_pool_file_creation()
        .assert_several_threads_write_the_same_files_as_one_thread(&GivenStatesTablesInTheLibrary);
}

/// 录入段取样：O2 开着时每个取样态录下一张盘、一张都不判（判归判段）。
#[test]
fn recording_the_sampled_states_records_a_table_for_every_state_and_judges_none() {
    let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
    let tables = PoolCheckerTablesKeptInMemory::judging_with(
        &pool_checker_red_names_of_a_given_image_on_the_cpu,
    );
    let observations = stream_to_sample
        .observations_recorded_in(SEVERAL_THREADS, &tables)
        .expect("这条流上实现不 panic");
    assert!(!observations.is_empty(), "取样取到了态");
    assert_eq!(tables.tables_judged(), 0, "录入段一张盘都不判");
    assert_eq!(
        tables.tables_recorded(),
        observations.len(),
        "每个取样态录下一张盘"
    );
    assert!(
        observations.iter().all(|observation| matches!(
            observation.pool_checker,
            PoolCheckerOfAState::RecordedForTheJudgeStage(_)
        )),
        "录入段交出的观测里 O2 都是录下了、没判"
    );
}

#[test]
fn panic_in_a_state_reports_the_same_text_on_several_threads_as_on_one_thread() {
    const PANIC_MESSAGE: &str = "线程对拍里造的实现自崩";
    let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
    let sampled_in_order: Vec<u64> = stream_to_sample
        .sampled
        .batches
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<u64>>()
        .into_iter()
        .collect();
    let position_of_the_panic = sampled_in_order.len() / 2;
    let ordinal_of_the_panic = sampled_in_order[position_of_the_panic];
    let panicking_at_that_state: &PoolCheckerTableOfASampledState<'_> =
        &move |_stream_name, ordinal, _image_digest, _image, _pool_checker_control| {
            assert_ne!(ordinal, ordinal_of_the_panic, "{PANIC_MESSAGE}");
            RecordedTableReference {
                table_name: String::new(),
                digest: [0; DIGEST_BYTES],
            }
        };
    let expected_ending = format!(
        ":after_ordinal_{}",
        sampled_in_order[position_of_the_panic - 1]
    );
    for threads in [NonZeroUsize::MIN, SEVERAL_THREADS] {
        let refusal = stream_to_sample
            .observations_on(threads, panicking_at_that_state)
            .err()
            .expect("那一态上 panic，这一趟要交回 Err")
            .crash;
        assert!(
            refusal.contains(PANIC_MESSAGE) && refusal.ends_with(&expected_ending),
            "{threads} 条线程：{refusal}，要以 {expected_ending} 收尾"
        );
    }
}
