//! 给定状态引擎的判器与库：GPU 判器与 CPU 判器在成对场景镜像和比字段表宽的树条目上对拍、被抛弃根的点名与别的主机判的结论、旋钮栏进块键、同一个用例名的两份库是两段名字空间。

use super::*;

// ─── GPU 判器与 CPU 判器在这几类镜像上对拍（编进 `gpu` 特性才有） ───

/// 一个交镜像的 harness 场景函数。
#[cfg(feature = "gpu")]
type ScenarioHandingOutImages = fn(&mut PoolCheckerImages);

/// 对拍取的场景：干净的镜像、改坏记录 / 根槽 / 链指针 / 树根的镜像、隔离豁免、不适用、产品旋钮下判红、一批单扣崩溃态，各取一两个；
/// 再加 unlink 路径上 inode 叶借、借完再吸收那两个（七百来个 inode：GPU 走读内核 inode → 对象出生代表那张表要装得下，
/// 这两个场景在对拍里由 GPU 判，那张表收窄时变异副本里也抓得到）；再加 I-3.15 隔离判据读一份头用不了、没有 key 指着的单元那一个
/// （C1「乙二」：组里读不读没打对不上映射记号的那一份，扫描内核与 `walk.rs` 各写一份）；
/// 再加只认隔离记号的四个世界（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」：两块盘都带记号的豁免、
/// 一块盘带一块盘不带、带记号而被最新根引用、没记号的照判违例；扫描内核 `c_quarantine_exempted`、I-3.15 那一组与 I-3.17 的三句和 `walk.rs`
/// 各写一份，对拍逐位比）；再加同一单元两条记录分落两棵分配记录树、只有一条带记号那一个（I-3.17 第三句按最新根全部分配记录树合起来归组，
/// 扫描内核 `c_judge_the_quarantine_marks_of_the_records_read` 与 `walk.rs` `judge_quarantine_marks_against_the_references` 各写一份）。
/// 两块盘都带记号的豁免那一个世界由隔离豁免那一个场景交（`quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back`）：
/// `quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies` 交的两张与它交的两张逐字节相同（两条照同一个造法造，
/// 块键只由内容定），列进来同一趟里后一张复用前一张判过的块、对拍那一趟就不是全在卡上判的，所以不列（[`images_of_the_paired_scenarios`] 断言每张只出现一次）。
#[cfg(feature = "gpu")]
const SCENARIOS_PAIRED_ON_THE_GPU_AND_THE_CPU: [(&str, ScenarioHandingOutImages); 18] = [
    (
        "writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page",
        scenarios_of_instance_table_page_full::run_the_scenario_where_writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page,
    ),
    (
        "the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment",
        scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::run_the_scenario_where_the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment,
    ),
    (
        "chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant",
        scenarios_of_instance_table_chain_read::run_the_scenario_where_chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant,
    ),
    (
        "row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it",
        scenarios_of_multi_record_publishes_of_transaction_zero::run_the_scenario_where_row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it,
    ),
    (
        "commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens",
        scenarios_of_multi_record_publishes_of_transaction_zero::run_the_scenario_where_commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens,
    ),
    (
        "with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens",
        scenarios_of_journal_last_record_flag_of_a_multi_record_publish::run_the_scenario_where_with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens,
    ),
    (
        "the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root",
        scenarios_of_accounting_and_mapping_tree_split::run_the_scenario_where_the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root,
    ),
    (
        "with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations",
        scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::run_the_scenario_where_with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations,
    ),
    (
        "quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back",
        scenarios_of_release_checksum_quarantine::run_the_scenario_where_quarantined_record_carrying_the_mark_is_exempted_from_the_allocated_statistics_whether_or_not_its_copy_reads_back,
    ),
    (
        "quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup",
        scenarios_of_release_checksum_quarantine::run_the_scenario_where_a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup,
    ),
    (
        "the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries",
        scenarios_of_an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor::run_the_scenario_where_the_checkers_effective_floor_counts_the_floor_an_abandoned_root_carries,
    ),
    (
        "capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry",
        scenarios_of_container_trees_grow_a_level_when_the_root_is_full::run_the_scenario_where_a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant_on_the_product_entry,
    ),
    (
        "leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing,
    ),
    (
        "the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it",
        scenarios_of_inode_leaf_absorb_and_borrow_on_the_unlink_path::run_the_scenario_where_the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it,
    ),
    (
        "quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence,
    ),
    (
        "record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence,
    ),
    (
        "quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back,
    ),
    // I-3.17 第三句的归组口径：同一单元的两条记录分落两棵分配记录树。GPU 扫描内核只在一棵树里归组时这一张判成立、CPU 判红
    (
        "unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence",
        scenarios_of_quarantine_mark_agrees_with_the_references::run_the_scenario_where_a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence,
    ),
];

/// 对拍那几个场景交出来的镜像，名字前面接上场景名（几个场景合进一趟，名字不撞）。
///
/// # Panics
/// 有两张镜像的内容（基线镜像、写表、切段、持久集合）逐样相同：块键只由内容定，同一趟里后一张复用前一张判过的块，
/// 对拍那一趟「状态都是这一趟在卡上判的」那一条就不成立，换一个场景或去掉重复的那一个。
#[cfg(feature = "gpu")]
fn images_of_the_paired_scenarios() -> Vec<PoolCheckerImage> {
    let mut images: Vec<PoolCheckerImage> = Vec::new();
    for (scenario_name, scenario) in SCENARIOS_PAIRED_ON_THE_GPU_AND_THE_CPU {
        let mut handed_out = PoolCheckerImages::collected_for_the_given_states_engine();
        scenario(&mut handed_out);
        for mut image in handed_out.into_collected() {
            image.label = format!("{scenario_name}/{}", image.label);
            let same_content_handed_out_earlier = images.iter().find(|earlier| {
                earlier.image == image.image
                    && earlier.writes == image.writes
                    && earlier.segments == image.segments
                    && earlier.persisted_sets == image.persisted_sets
            });
            assert!(
                same_content_handed_out_earlier.is_none(),
                "对拍那一批里每张镜像只出现一次：{} 与 {:?} 逐字节相同",
                image.label,
                same_content_handed_out_earlier.map(|earlier| earlier.label.as_str())
            );
            images.push(image);
        }
    }
    images
}

/// 一趟判下来的逐张镜像逐状态的对拍字，与引擎那一趟。
#[cfg(feature = "gpu")]
struct WordsOfARun {
    words: Vec<Vec<singlefs_verification::crash_judge_conclusions::CrossCheckWords>>,
    run: GivenStatesRun,
}

/// 同一批镜像的两趟：配了 GPU 的那一趟与没开 GPU 的那一趟。
#[cfg(feature = "gpu")]
struct PairedRuns {
    images: Vec<PoolCheckerImage>,
    on_the_gpu: WordsOfARun,
    on_the_cpu: WordsOfARun,
}

/// 同一批镜像在环境守卫里判一趟：多机配置指到 `configuration`、库是 `library` 那一份一次性库（设成这个进程开的那一份），开 GPU 时卡是调用方给这个进程的那一份
/// （守卫把一趟读的变量清掉之后，`common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run` 交给引擎：服务交了卡串的设卡串，
/// 清场之下是清场之后那一份；没交卡串的不设、不借卡；那份文件头写了为什么不抄进配置；
/// 这一台是 HOSTS 里哪一项照调用方那份多机配置上判法认出的，不经环境交；写法同 `crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`）。
/// 交回逐张镜像逐状态的对拍字与引擎那一趟。
#[cfg(feature = "gpu")]
fn words_of_the_paired_images_judged_under(
    images: &[PoolCheckerImage],
    configuration: &std::path::Path,
    library: &DisposableLibrary,
    gpu_enabled: bool,
    fault: singlefs_verification::crash_amplification::FaultPutOnTheGpuSide,
) -> WordsOfARun {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    let cards_of_the_caller =
        gpu_enabled.then(common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run);
    let engine = GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side(
        GivenStatesItem {
            item_name: format!("{}::gpu-and-cpu-pairing", ITEM_NAME),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        fault,
    )
    .expect("引擎照这一趟的配置起得来");
    let words = images
        .iter()
        .map(|image| {
            judge_one_image(&engine, image, ContainerCapacityKnobs::PRODUCT)
                .states
                .iter()
                .map(|state| state.words)
                .collect()
        })
        .collect();
    let run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    WordsOfARun { words, run }
}

/// 同一批镜像一趟配了 GPU、一趟没开 GPU，各写一份自己的一次性库（临时 prune-server 里以用例标签命名的一段名字空间，起跑是空的）。
#[cfg(feature = "gpu")]
fn paired_images_judged_on_the_gpu_and_on_the_cpu(
    tag: &str,
    fault: singlefs_verification::crash_amplification::FaultPutOnTheGpuSide,
) -> PairedRuns {
    let directory_of = |run: &str| {
        // 一次性库留着：GPU 与 CPU 两趟对拍、GPU 一侧塞故障：两趟各判各的、都要现判
        DisposableLibrary::new(&format!("harness-scenario-images-gpu-pairing-{tag}-{run}"))
            .unwrap_or_else(|refusal| panic!("{refusal}"))
    };
    let images = images_of_the_paired_scenarios();
    let gpu_directory = directory_of("on-the-gpu");
    let gpu_configuration = gpu_directory.join("multi-host-with-gpu.env");
    // 配置只写开关：显卡先后照调用方那一份经环境变量交（`words_of_the_paired_images_judged_under`）
    std::fs::write(&gpu_configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");
    let on_the_gpu = words_of_the_paired_images_judged_under(
        &images,
        &gpu_configuration,
        &gpu_directory,
        true,
        fault,
    );
    let cpu_directory = directory_of("on-the-cpu");
    let cpu_configuration = cpu_directory.join("multi-host-on-the-cpu.env");
    std::fs::write(&cpu_configuration, "ENABLE_GPU=0\n").expect("写得了配置");
    let on_the_cpu = words_of_the_paired_images_judged_under(
        &images,
        &cpu_configuration,
        &cpu_directory,
        false,
        singlefs_verification::crash_amplification::FaultPutOnTheGpuSide::NoFault,
    );
    PairedRuns {
        images,
        on_the_gpu,
        on_the_cpu,
    }
}

/// GPU 与 CPU 对拍（两趟独立、都从库里读）：对拍那几个场景交出来的同一批镜像一趟由 GPU 判器判、一趟由 CPU 判器判，逐张逐状态的对拍字相同；
/// GPU 那一趟判不了的只许是场景许它建不出判表的那几张（这一批里没有）。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
#[cfg(feature = "gpu")]
#[test]
fn the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words()
{
    use singlefs_verification::crash_amplification::{FaultPutOnTheGpuSide, JudgedBy};
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_images_of_the_paired_harness_scenarios_judged_on_the_gpu_and_on_the_cpu_give_the_same_words";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let PairedRuns {
        images,
        on_the_gpu: WordsOfARun {
            words: gpu_words,
            run: gpu_run,
        },
        on_the_cpu: WordsOfARun {
            words: cpu_words,
            run: cpu_run,
        },
    } = paired_images_judged_on_the_gpu_and_on_the_cpu("agrees", FaultPutOnTheGpuSide::NoFault);
    assert_eq!(
        gpu_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU 的那一趟：GPU 判器判"
    );
    assert_eq!(
        cpu_run.judged_by,
        JudgedBy::CpuJudge,
        "没开 GPU 的那一趟：CPU 判器判"
    );
    assert!(
        gpu_run.tally.states_judged_by_the_gpu_here > 0 && gpu_run.tally.states_reused == 0,
        "GPU 那一趟从空库起、状态都是这一趟在卡上判的：{:?}",
        gpu_run.tally
    );
    assert_eq!(
        gpu_run.tally.states_the_gpu_could_not_judge,
        Vec::<String>::new(),
        "GPU 判器判得了这一批每一张（写表为空的表、改坏的镜像都在里面）"
    );
    let mismatches: Vec<String> = images
        .iter()
        .zip(gpu_words.iter().zip(&cpu_words))
        .filter(|(_, (on_the_gpu, on_the_cpu))| on_the_gpu != on_the_cpu)
        .map(|(image, (on_the_gpu, on_the_cpu))| {
            format!("{}：GPU {on_the_gpu:?}；CPU {on_the_cpu:?}", image.label)
        })
        .collect();
    println!(
        "HARNESS_SCENARIO_IMAGES_GPU_PAIRING images={} states={} mismatches={}",
        images.len(),
        gpu_words.iter().map(Vec::len).sum::<usize>(),
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "{} 张镜像上 GPU 一趟与 CPU 一趟的对拍字不同：\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

/// 上一条的判别力：往 GPU 那一侧塞一个已知故障（每张表第一个状态的第一条不变量违例位翻过来），GPU 那一趟的对拍字与 CPU 那一趟不同。
#[cfg(feature = "gpu")]
#[test]
fn fault_put_on_the_gpu_side_makes_the_gpu_run_on_the_paired_harness_scenario_images_differ_from_the_cpu_run(
) {
    use singlefs_verification::crash_amplification::FaultPutOnTheGpuSide;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "fault_put_on_the_gpu_side_makes_the_gpu_run_on_the_paired_harness_scenario_images_differ_from_the_cpu_run";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let paired = paired_images_judged_on_the_gpu_and_on_the_cpu(
        "fault",
        FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
    );
    assert_ne!(
        paired.on_the_gpu.words, paired.on_the_cpu.words,
        "GPU 那一侧塞了故障，两趟的对拍字却相同"
    );
}

// ============================== I-7.4 点名的被抛弃根（GPU 判器加细节输出）与旋钮栏进块键 ==============================

/// 别的机器判的、照用的结论也带 I-7.4 点名的被抛弃根：被抛弃根那一半判红的镜像（[`image_whose_abandoned_root_walks_into_an_overwritten_unit`]）
/// 照本机配置判（配了 GPU 只由 GPU 判器判）。判表的那一台在这条用例自己那一份库里判两趟：第一趟现判、存进库；第二趟库里这张表记着红，
/// 照「记红、撤绿、下趟现判」不复用、现判。另一台（两台分片、权重 0,1，它是 0/2，一张表都不归它）在同一份库上
/// 判同一张表（两台都直接写服务那一份）：这张表归判表那一台、库里记着红也照用，不现判、不交占位。三趟的结论都红 I-7.4、都点名 C 那一条根。
/// 判别力（`crates/mutations.tsv`）：存进库时点名的根丢掉，照用的结论点名的是空的，这条红；归别的机器判的表记着红也当成这台判的，
/// 另一台不照用、又不归它判，交占位、留给别的机器，这条红；这台判的表记着红也照用，判表那一台第二趟整张复用，这条红。
#[test]
fn the_abandoned_root_the_pool_checker_names_comes_back_with_the_red_conclusions_another_host_judged(
) {
    const TEST_NAME: &str =
        "the_abandoned_root_the_pool_checker_names_comes_back_with_the_red_conclusions_another_host_judged";
    let (image, expected_root) = image_whose_abandoned_root_walks_into_an_overwritten_unit(
        "given-states-named-abandoned-root-another-host",
    );
    let table = table_of_one_image("被抛弃根那一半判红的镜像", &image);
    let pool_checker_image = PoolCheckerImage {
        label: table.table_name.clone(),
        image: image.clone(),
        writes: Vec::new(),
        segments: Vec::new(),
        persisted_sets: vec![Vec::new()],
        expected: PoolCheckerExpectation::only_the_named_invariants(),
        the_gpu_may_refuse_its_tables_because: None,
    };
    // 判表那一台的库在它的两趟之间传，另一台在同一份库上起（两台都直接写服务那一份）
    let Some(library_of_the_judging_host) = LibraryOfTheTest::create_or_report_not_run(TEST_NAME)
    else {
        return;
    };
    let mut runs_of_the_judging_host = Vec::new();
    for library_at_the_start in [
        LibraryOfTheTestAtTheStart::EmptiedWhenTheRecordStageRuns,
        LibraryOfTheTestAtTheStart::KeptFromTheEarlierRunOfTheTest,
    ] {
        let engine = start_the_engine_on_the_library_of_the_test(
            TEST_NAME,
            &library_of_the_judging_host,
            library_at_the_start,
        );
        let judged = judge_one_image(
            &engine,
            &pool_checker_image,
            ContainerCapacityKnobs::PRODUCT,
        );
        let run = engine.finish();
        if !the_run_counts(&run, std::slice::from_ref(&judged), TEST_NAME) {
            return;
        }
        runs_of_the_judging_host.push((judged, run));
    }
    let [(judged_first, judged_first_run), (judged_again, judged_again_run)] =
        runs_of_the_judging_host.as_slice()
    else {
        panic!("判表那一台两趟各交回一份结论");
    };
    let engine_of_the_other_host = start_the_engine_on_the_library_of_the_test_as_a_share(
        TEST_NAME,
        &library_of_the_judging_host,
        "0,1",
        "0/2",
    );
    let judged_by_the_other_host = judge_one_image(
        &engine_of_the_other_host,
        &pool_checker_image,
        ContainerCapacityKnobs::PRODUCT,
    );
    let run_of_the_other_host = engine_of_the_other_host.finish();
    if !the_run_counts(
        &run_of_the_other_host,
        std::slice::from_ref(&judged_by_the_other_host),
        TEST_NAME,
    ) {
        return;
    }
    assert_eq!(
        (
            judged_first_run.tally.tables_judged_here,
            judged_first_run.tally.tables_reused
        ),
        (1, 0),
        "判表那一台第一趟从清空的库起、现判：{:?}",
        judged_first_run.tally
    );
    assert_eq!(
        (
            judged_again_run.tally.tables_judged_here,
            judged_again_run.tally.tables_reused,
            judged_again_run.tally.tables_stored_red_judged_again
        ),
        (1, 0, 1),
        "判表那一台第二趟：库里这张表记着红，不复用、现判（记红、撤绿、下趟现判）：{:?}",
        judged_again_run.tally
    );
    assert_eq!(
        (
            run_of_the_other_host.tally.tables_judged_here,
            run_of_the_other_host.tally.tables_reused,
            run_of_the_other_host.tally.tables_stored_red_judged_again,
            run_of_the_other_host.tally.tables_left_to_the_other_hosts
        ),
        (0, 1, 0, 0),
        "另一台：这张表归判表那一台，库里记着红也照用，不现判、不交占位：{:?}",
        run_of_the_other_host.tally
    );
    for (which_run, judged) in [
        ("判表那一台第一趟现判的", judged_first),
        ("判表那一台第二趟现判的", judged_again),
        ("另一台照用的", &judged_by_the_other_host),
    ] {
        let [state] = judged.states.as_slice() else {
            panic!("一张镜像一个状态");
        };
        assert!(
            is_violated(state, singlefs_verification::crash_given_states::INVARIANT_WHOSE_DETAIL_NAMES_ABANDONED_ROOTS),
            "{which_run}：被抛弃根那一半判 I-7.4 红（违例 {:?}）",
            state.pool_checker_violations()
        );
        assert_eq!(
            state.abandoned_roots_named_by_the_pool_checker,
            [expected_root],
            "{which_run}：I-7.4 的违例细节点名被抛弃的 C 那一条根"
        );
    }
}

/// 一张给定状态表在环境守卫里判一趟（用例自己的一份多机配置与一份一次性库，起跑是空的）：开 GPU 那一趟卡是调用方给这个进程的那一份
/// （`common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run`：服务交了卡串的设卡串，没交的不设、不借卡），
/// 这一台是 HOSTS 里哪一项照调用方那份多机配置上判法认出的。交回这张表的结论与引擎那一趟。
#[cfg(feature = "gpu")]
fn conclusions_of_a_table_judged_with_the_gpu(
    table: &GivenStatesOfATable<'_>,
    tag: &str,
    gpu_enabled: bool,
) -> (
    singlefs_verification::crash_given_states::GivenStatesConclusionsOfATable,
    GivenStatesRun,
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    use singlefs_verification::crash_given_states::GivenStatesJudge;
    use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
    // 一次性库留着：GPU 与 CPU 对拍的一侧：这张表要这一趟在这一侧现判
    let library = DisposableLibrary::new(tag).unwrap_or_else(|refusal| panic!("{refusal}"));
    let directory: &std::path::Path = &library;
    let configuration = directory.join("multi-host.env");
    std::fs::write(
        &configuration,
        if gpu_enabled {
            "ENABLE_GPU=1\n"
        } else {
            "ENABLE_GPU=0\n"
        },
    )
    .expect("写得了配置");
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    let cards_of_the_caller =
        gpu_enabled.then(common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run);
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!(
                "{}::abandoned-root-named-on-the-gpu-and-the-cpu {tag}",
                ITEM_NAME
            ),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("引擎照这一趟的配置起得来");
    let judged = engine
        .judge_the_given_states(table)
        .unwrap_or_else(|reason| panic!("判不了（不退回别处判）：{reason}"));
    let run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    (judged, run)
}

/// GPU 与 CPU 对拍 I-7.4 点名的被抛弃根（两趟独立、各从自己那一份库读回）：被抛弃根那一半判红的镜像
/// （[`image_whose_abandoned_root_walks_into_an_overwritten_unit`]）一趟由 GPU 判器判（走读内核记下点名的根）、一趟由 CPU 判器判
/// （从违例细节里读出），两趟的对拍字相同、点名的根逐个相同，都是 C 那一条。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：走读内核不记点名的根，GPU 那一趟点名的是空的，这条红。
#[cfg(feature = "gpu")]
#[test]
fn the_gpu_judge_names_the_same_abandoned_root_as_the_cpu_judge_on_an_image_whose_abandoned_root_walks_into_an_overwritten_unit(
) {
    use singlefs_verification::crash_amplification::JudgedBy;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    const TEST_NAME: &str =
        "the_gpu_judge_names_the_same_abandoned_root_as_the_cpu_judge_on_an_image_whose_abandoned_root_walks_into_an_overwritten_unit";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let (image, expected_root) = image_whose_abandoned_root_walks_into_an_overwritten_unit(
        "given-states-named-abandoned-root-paired",
    );
    let table = table_of_one_image("被抛弃根那一半判红的镜像", &image);
    let (on_the_gpu, gpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-named-abandoned-root-on-the-gpu",
        true,
    );
    let (on_the_cpu, cpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-named-abandoned-root-on-the-cpu",
        false,
    );
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        (
            gpu_run.tally.states_judged_by_the_gpu_here,
            gpu_run.tally.states_the_gpu_could_not_judge.clone()
        ),
        (1, Vec::<String>::new()),
        "GPU 那一趟从空库起、这一个状态在卡上判得了"
    );
    let [gpu_state] = on_the_gpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    let [cpu_state] = on_the_cpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    assert_eq!(
        gpu_state.words, cpu_state.words,
        "两趟的对拍字相同（违例位、判到位、两遍恢复落到的根）"
    );
    assert!(
        is_violated(
            cpu_state,
            singlefs_verification::crash_given_states::INVARIANT_WHOSE_DETAIL_NAMES_ABANDONED_ROOTS
        ),
        "被抛弃根那一半判 I-7.4 红：{:?}",
        cpu_state.pool_checker_violations()
    );
    assert_eq!(
        cpu_state.abandoned_roots_named_by_the_pool_checker,
        [expected_root],
        "CPU 判器从违例细节里读出的是被抛弃的 C 那一条根"
    );
    assert_eq!(
        gpu_state.abandoned_roots_named_by_the_pool_checker,
        cpu_state.abandoned_roots_named_by_the_pool_checker,
        "GPU 判器点名的根与 CPU 判器的逐个相同"
    );
}

/// 旋钮栏进块键（给定状态表加旋钮栏）：同一张镜像（墓碑容器压到一片 1 条写出来的那一张）、同一个表名，
/// 旋钮栏一张不填（产品容量：I-9.21 红）、一张填墓碑容器旋钮值 1（一条不红），在同一趟、同一份清空过的库里先后判：两张各判各的、
/// 一张都不复用，判出的违例各照各的旋钮栏。判别力（`crates/mutations.tsv`）：块键不收旋钮栏，后判的那张复用先判那张的结论、判出 I-9.21 红，这条红。
#[test]
fn two_tables_of_one_name_and_one_image_under_different_container_capacity_knobs_are_each_judged_under_their_own_knobs(
) {
    use scenarios_of_container_trees_grow_a_level_when_the_root_is_full::{
        knobs_of_the_tombstone_container_only_at,
        pool_with_a_capped_tombstone_tree_handing_its_product_image_out,
        TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
    };
    use singlefs_checker::image::NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE;
    const TEST_NAME: &str =
        "two_tables_of_one_name_and_one_image_under_different_container_capacity_knobs_are_each_judged_under_their_own_knobs";
    let pool = pool_with_a_capped_tombstone_tree_handing_its_product_image_out(
        &mut PoolCheckerImages::left_to_the_given_states_engine(),
    );
    let image = PoolCheckerImage {
        label: "墓碑容器压到一片 1 条".to_string(),
        image: pool.memory_pool(),
        writes: Vec::new(),
        segments: Vec::new(),
        persisted_sets: vec![Vec::new()],
        expected: PoolCheckerExpectation::only_the_named_invariants(),
        the_gpu_may_refuse_its_tables_because: None,
    };
    let Some(library_of_the_test) = LibraryOfTheTest::create_or_report_not_run(TEST_NAME) else {
        return;
    };
    let engine = start_the_engine(TEST_NAME, &library_of_the_test);
    let under_the_product_capacity =
        judge_one_image(&engine, &image, ContainerCapacityKnobs::PRODUCT);
    let under_the_tombstone_knob = judge_one_image(
        &engine,
        &image,
        knobs_of_the_tombstone_container_only_at(TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB),
    );
    let run = engine.finish();
    let conclusions = [under_the_product_capacity, under_the_tombstone_knob];
    if !the_run_counts(&run, &conclusions, TEST_NAME) {
        return;
    }
    assert_eq!(
        (run.tally.tables_judged_here, run.tally.tables_reused),
        (2, 0),
        "同名、同一张镜像、旋钮栏不同的两张表各判各的，一张都不复用：{:?}",
        run.tally
    );
    let violated_of = |judged: &ConclusionsOfAnImage| -> Vec<&'static str> {
        let [state] = judged.states.as_slice() else {
            panic!("一张镜像一个状态");
        };
        IMPLEMENTED_INVARIANTS
            .iter()
            .copied()
            .filter(|invariant| is_violated(state, invariant))
            .collect()
    };
    assert_eq!(
        violated_of(&conclusions[0]),
        [NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
        "旋钮栏不填（产品容量）：只红 I-9.21"
    );
    assert_eq!(
        violated_of(&conclusions[1]),
        Vec::<&str>::new(),
        "旋钮栏填墓碑容器旋钮值 1：一条不红"
    );
}

// ============================== 叶条目比字段表宽的记账树（GPU 收 inode 号水位行那一处的叶条目宽要等于 34） ==============================

/// 记账树一片叶的条目比字段表宽一个字节（34 → 35，每一条末尾补一个 0）的镜像：两棵树压小写出第一个文件版本
/// （`scenarios_of_accounting_and_mapping_tree_split` 的 [`SMALL_ACCOUNTING`]，记账树两层四片叶），改记账树按 bump 次序第一片叶、
/// 补好引用链上的校验和（`common_tree_split::propagate_the_new_checksum`），镜像上只剩这一处说话。CPU 主走读照 I-1.10 判条目宽（等于字段表宽），
/// 不等就不往下走、这棵树不算走全，I-9.30 收不到水位行、报不适用；GPU 扫描内核收水位行那一处照同一条判据（叶 34、内部 108 都要等于）。
///
/// [`SMALL_ACCOUNTING`]: scenarios_of_accounting_and_mapping_tree_split::SMALL_ACCOUNTING
#[cfg(feature = "gpu")]
fn image_whose_accounting_leaf_entries_are_wider_than_the_field_table(
) -> singlefs_verification::memory_pool::MemoryPool {
    use common_tree_split::{
        capacities, propagate_the_new_checksum, write_unit_to_the_same_slot_on_both_devices,
        TreeSplitPool,
    };
    use scenarios_of_accounting_and_mapping_tree_split::{
        node_units, SMALL_ACCOUNTING, SMALL_CENTRAL_MAPPING,
    };
    use singlefs_core::checksum::crc32_castagnoli;
    use singlefs_core::transaction::MultiLevelCodeTwoTree;
    use singlefs_core::unit::{build_index_node, parse_index_node};
    let pool = TreeSplitPool::with_the_first_file_version_under(capacities(
        SMALL_ACCOUNTING,
        SMALL_CENTRAL_MAPPING,
    ));
    let output = &pool.output;
    let leaf_unit = node_units(output, MultiLevelCodeTwoTree::Accounting)
        .into_iter()
        .find(|unit| {
            parse_index_node(&unit.bytes)
                .expect("记账树的节点解得开")
                .level
                == 0
        })
        .expect("压小的记账树长成两层，有叶");
    let leaf = parse_index_node(&leaf_unit.bytes).expect("记账树的叶解得开");
    let wider_entries: Vec<Vec<u8>> = leaf
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    let wider_leaf = build_index_node(
        leaf.tree,
        leaf.level,
        leaf.key_width,
        &leaf.smallest_key,
        &leaf.largest_key,
        leaf.birth_txg,
        &common::parameters().filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(leaf.entry_width + 1).expect("条目宽装得进 u16"),
        &wider_entries,
    );
    let mut image = pool.memory_pool();
    write_unit_to_the_same_slot_on_both_devices(&mut image, leaf_unit.slot.0, &wider_leaf);
    let units: Vec<(u64, usize)> = output
        .units
        .iter()
        .map(|unit| (unit.slot.0, unit.bytes.len()))
        .collect();
    propagate_the_new_checksum(
        &mut image,
        &units,
        (
            leaf_unit.slot.0,
            crc32_castagnoli(&leaf_unit.bytes),
            crc32_castagnoli(&wider_leaf),
        ),
    );
    image
}

/// GPU 与 CPU 对拍叶条目比字段表宽的记账树（两趟独立、各从自己那一份库读回）：[`image_whose_accounting_leaf_entries_are_wider_than_the_field_table`]
/// 一趟由 GPU 判器判、一趟由 CPU 判器判，对拍字相同；CPU 那一趟 I-1.10 红、I-9.30 没判到（这棵树不算走全、一行水位行都不收）。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：GPU 扫描内核收水位行时叶条目宽改回只要求不小于 34，GPU 那一趟读过这片叶、判到 I-9.30，这条红。
#[cfg(feature = "gpu")]
#[test]
fn an_accounting_leaf_wider_than_its_field_table_is_judged_alike_on_the_gpu_and_the_cpu_and_its_watermark_rows_stay_unread(
) {
    use singlefs_verification::crash_amplification::JudgedBy;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    /// 条目宽要等于字段表宽的那一条（池级 checker 主走读照它判、不等就不往下走）。
    const ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE: &str = "I-1.10";
    /// 记账树 inode 号水位行那一条（只取走全了的记账树的行）。
    const INODE_NUMBER_WATERMARK_STEPS: &str = "I-9.30";
    const TEST_NAME: &str =
        "an_accounting_leaf_wider_than_its_field_table_is_judged_alike_on_the_gpu_and_the_cpu_and_its_watermark_rows_stay_unread";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let image = image_whose_accounting_leaf_entries_are_wider_than_the_field_table();
    let table = table_of_one_image("记账树一片叶的条目宽 35", &image);
    let (on_the_gpu, gpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-accounting-leaf-wider-than-its-field-table-on-the-gpu",
        true,
    );
    let (on_the_cpu, cpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-accounting-leaf-wider-than-its-field-table-on-the-cpu",
        false,
    );
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        (
            gpu_run.tally.states_judged_by_the_gpu_here,
            gpu_run.tally.states_the_gpu_could_not_judge.clone()
        ),
        (1, Vec::<String>::new()),
        "GPU 那一趟从空库起、这一个状态在卡上判得了"
    );
    let [gpu_state] = on_the_gpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    let [cpu_state] = on_the_cpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    assert!(
        is_violated(cpu_state, ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE),
        "前提：CPU 主走读判条目宽比字段表宽（{ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE}）红：{:?}",
        cpu_state.pool_checker_violations()
    );
    assert!(
        !is_judged(cpu_state, INODE_NUMBER_WATERMARK_STEPS),
        "前提：这棵记账树不算走全，CPU 判 {INODE_NUMBER_WATERMARK_STEPS} 不适用"
    );
    assert_eq!(
        gpu_state.words, cpu_state.words,
        "两趟的对拍字相同（违例位、判到位、两遍恢复落到的根）"
    );
}

// ============================== 内部条目比字段表宽的记账树根（C780，I-1.10：码 2 内部节点的条目宽要等于字段表宽） ==============================

/// GPU 与 CPU 对拍记账树根的内部条目比字段表宽一个字节（108 → 109）的镜像，比到恢复字（两趟独立、各从自己那一份库读回）：镜像是 harness 场景
/// `scenarios_of_accounting_and_mapping_tree_split::image_with_the_root_damaged` 交出的那张（`RootDamage::EntryWiderThanTheInternalEntryFieldTable`：
/// 两棵树压小写出第一个文件版本，只改记账树的根、补好引用链上的校验和）。核心层恢复读码 2 树照 C780（恢复只拒窄于字段表的条目宽）：
/// 条目宽不等于字段表宽就拒，看 journal 那一遍失败在 `EntryWidthDiffersFromItsFieldTable`；CPU 池级 checker 红 I-1.10。一趟由 GPU 判器判、
/// 一趟由 CPU 判器判，对拍字整份相同：不只池级 checker 的违例位与判到位，连两遍恢复的结局、失败点与落到的根。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：GPU 恢复内核读码 2 树时内部节点的条目宽改回只拒窄于字段表的，GPU 那一趟照读这个根、恢复不在这里失败，
/// 恢复那几个字与 CPU 不同，这条红（同一张镜像经 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root` 只比池级 checker，抓不到）。
#[cfg(feature = "gpu")]
#[test]
fn an_accounting_tree_root_wider_than_its_internal_field_table_is_refused_alike_by_the_gpu_and_the_cpu_recovery(
) {
    use common_tree_split::{capacities, TreeSplitPool};
    use scenarios_of_accounting_and_mapping_tree_split::{
        image_with_the_root_damaged, RootDamage, SMALL_ACCOUNTING, SMALL_CENTRAL_MAPPING,
    };
    use singlefs_core::transaction::MultiLevelCodeTwoTree;
    use singlefs_verification::crash_amplification::JudgedBy;
    use singlefs_verification::crash_judge_conclusions::{
        GPU_FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE, RECOVERY_OUTCOME_CODE_FAILED,
    };
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    /// 条目宽要等于字段表宽的那一条（池级 checker 主走读照它判）。
    const ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE: &str = "I-1.10";
    const TEST_NAME: &str =
        "an_accounting_tree_root_wider_than_its_internal_field_table_is_refused_alike_by_the_gpu_and_the_cpu_recovery";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let pool = TreeSplitPool::with_the_first_file_version_under(capacities(
        SMALL_ACCOUNTING,
        SMALL_CENTRAL_MAPPING,
    ));
    let image = image_with_the_root_damaged(
        &pool,
        MultiLevelCodeTwoTree::Accounting,
        RootDamage::EntryWiderThanTheInternalEntryFieldTable,
    );
    let table = table_of_one_image("记账树根的内部条目宽 109", &image);
    let (on_the_gpu, gpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-accounting-root-wider-than-its-internal-field-table-on-the-gpu",
        true,
    );
    let (on_the_cpu, cpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-accounting-root-wider-than-its-internal-field-table-on-the-cpu",
        false,
    );
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        (
            gpu_run.tally.states_judged_by_the_gpu_here,
            gpu_run.tally.states_the_gpu_could_not_judge.clone()
        ),
        (1, Vec::<String>::new()),
        "GPU 那一趟从空库起、这一个状态在卡上判得了"
    );
    let [gpu_state] = on_the_gpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    let [cpu_state] = on_the_cpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    assert!(
        is_violated(cpu_state, ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE),
        "前提：CPU 主走读判条目宽比字段表宽（{ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE}）红：{:?}",
        cpu_state.pool_checker_violations()
    );
    assert_eq!(
        (
            cpu_state.words.consulted_pass_outcome,
            cpu_state.words.consulted_pass_failure_point
        ),
        (
            RECOVERY_OUTCOME_CODE_FAILED,
            GPU_FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE
        ),
        "前提：CPU 看 journal 那一遍恢复失败在条目宽与字段表不符（C780 宽于字段表也拒）"
    );
    assert_eq!(
        gpu_state.words, cpu_state.words,
        "两趟的对拍字整份相同（违例位、判到位、两遍恢复的结局、失败点与落到的根）"
    );
}

// ============================== 叶条目比字段表宽的 deadlist 树（I-1.10：deadlist 叶条目宽要等于 43） ==============================

/// deadlist 树根（根兼叶）的条目比字段表宽一个字节（43 → 44，每一条末尾补一个 0）的镜像：新池新建文件、建快照、快照下覆盖写
/// （换下的单元进 deadlist），改 deadlist 树根、按原来的身份重建重封，新的整单元校验和沿引用链补到根槽
/// （`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`），镜像上只剩这一处说话。造法与 harness
/// `crates/singlefs-verification/tests/checker_judges_deadlist_entry_widths_against_the_field_table.rs` 的同名场景相同（测试目标之间互相引不到，这里另写一份）。
#[cfg(feature = "gpu")]
fn image_whose_deadlist_leaf_entries_are_wider_than_the_field_table(
) -> singlefs_verification::memory_pool::MemoryPool {
    use common::{
        build_pool, parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS,
        OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
    };
    use common_pointer_head_corruption::{
        node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
        tree_table_entry_of_kind, unit_bytes_at,
    };
    use singlefs_core::address::InstanceGeneration;
    use singlefs_core::records::TREE_KIND_DEADLIST;
    use singlefs_core::transaction::{publish_create_snapshot, PoolWriter};
    use singlefs_core::unit::{build_index_node, parse_index_node};
    /// deadlist 叶条目字段表的宽：头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空（D5（快照 / 空间记账机制） 已定项 12）。
    const DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES: usize = 43;
    /// 建快照与快照下覆盖写都在实例 1 上（`build_pool` 取的号）。
    const INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE: InstanceGeneration = InstanceGeneration(1);
    let mut pool = build_pool("given-states-deadlist-leaf-wider-than-its-field-table");
    let publish_parameters = parameters();
    let before_the_snapshot = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &before_the_snapshot,
        INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE,
    )
    .expect("建快照");
    let before_the_overwrite = pool.output.clone();
    let overwrite_content: Vec<u8> = (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 11) % 239).expect("小于 256"))
        .collect();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &before_the_overwrite,
        &overwrite_content,
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE,
    )
    .expect("快照下覆盖写");
    let mut image = pool.memory_pool();
    let deadlist_root = tree_table_entry_of_kind(&image, TREE_KIND_DEADLIST).root;
    let leaf = parse_index_node(&unit_bytes_at(
        &image,
        the_slot_of(&deadlist_root),
        node_bytes(),
    ))
    .expect("deadlist 树根解得开");
    assert!(
        leaf.level == 0
            && !leaf.entries.is_empty()
            && leaf.entry_width == DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES,
        "前提：deadlist 树根兼叶、装着条目，写者写的条目宽等于字段表宽（层级 {}、{} 条、条目宽 {}）",
        leaf.level,
        leaf.entries.len(),
        leaf.entry_width
    );
    let wider_entries: Vec<Vec<u8>> = leaf
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    let wider_leaf = build_index_node(
        leaf.tree,
        leaf.level,
        leaf.key_width,
        &leaf.smallest_key,
        &leaf.largest_key,
        leaf.birth_txg,
        &parameters().filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES + 1).expect("条目宽装得进 u16"),
        &wider_entries,
    );
    replace_the_unit_and_propagate_its_checksum(
        &mut image,
        the_slot_of(&deadlist_root),
        &wider_leaf,
    );
    image
}

/// GPU 与 CPU 对拍 deadlist 叶条目比字段表宽的镜像（两趟独立、各从自己那一份库读回）：
/// [`image_whose_deadlist_leaf_entries_are_wider_than_the_field_table`] 一趟由 GPU 判器判、一趟由 CPU 判器判，池级 checker 那几个字
/// （违例位、判到位、扫描重建的结局码）相同；CPU 那一趟 I-1.10 红（判决 `research/prompts/m3-close-w13-code-r1-main-verification.md`
/// 第二节 K5 那一行：池级 checker 的 deadlist 那一支按字段表宽判等于，GPU 走读内核同判）。两遍恢复那几个字不比：GPU 恢复内核读 deadlist 的
/// 条目宽与判退归下一批（`crash_judge_recovery.wgsl` 这一批不动）。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力（`crates/mutations.tsv`）：GPU 走读内核的 deadlist 树改回只守不窄于字段表，GPU 那一趟不红 I-1.10，违例位与 CPU 不同，这条红。
#[cfg(feature = "gpu")]
#[test]
fn deadlist_leaf_wider_than_its_field_table_is_judged_alike_by_the_pool_checker_on_the_gpu_and_the_cpu(
) {
    use singlefs_verification::crash_amplification::JudgedBy;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    /// 条目宽要等于字段表宽的那一条（池级 checker 主走读照它判）。
    const ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE: &str = "I-1.10";
    const TEST_NAME: &str =
        "deadlist_leaf_wider_than_its_field_table_is_judged_alike_by_the_pool_checker_on_the_gpu_and_the_cpu";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let image = image_whose_deadlist_leaf_entries_are_wider_than_the_field_table();
    let table = table_of_one_image("deadlist 树根兼叶的条目宽 44", &image);
    let (on_the_gpu, gpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-deadlist-leaf-wider-than-its-field-table-on-the-gpu",
        true,
    );
    let (on_the_cpu, cpu_run) = conclusions_of_a_table_judged_with_the_gpu(
        &table,
        "given-states-deadlist-leaf-wider-than-its-field-table-on-the-cpu",
        false,
    );
    assert_eq!(
        (gpu_run.judged_by, cpu_run.judged_by),
        (JudgedBy::GpuJudge, JudgedBy::CpuJudge),
        "一趟由 GPU 判器判、一趟由 CPU 判器判"
    );
    assert_eq!(
        (
            gpu_run.tally.states_judged_by_the_gpu_here,
            gpu_run.tally.states_the_gpu_could_not_judge.clone()
        ),
        (1, Vec::<String>::new()),
        "GPU 那一趟从空库起、这一个状态在卡上判得了"
    );
    let [gpu_state] = on_the_gpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    let [cpu_state] = on_the_cpu.states.as_slice() else {
        panic!("一张镜像一个状态");
    };
    assert!(
        is_violated(cpu_state, ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE),
        "前提：CPU 主走读判 deadlist 叶条目宽比字段表宽（{ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE}）红：{:?}",
        cpu_state.pool_checker_violations()
    );
    assert!(!gpu_state.words.gpu_could_not_judge, "GPU 判得了这一个状态");
    assert_eq!(
        (
            gpu_state.words.pool_checker_violated_bits,
            gpu_state.words.pool_checker_judged_bits,
            gpu_state.words.scan_rebuild_outcome
        ),
        (
            cpu_state.words.pool_checker_violated_bits,
            cpu_state.words.pool_checker_judged_bits,
            cpu_state.words.scan_rebuild_outcome
        ),
        "两趟池级 checker 那几个字相同（违例位、判到位、扫描重建的结局码）：GPU 违例 {:?}，CPU 违例 {:?}",
        gpu_state.pool_checker_violations(),
        cpu_state.pool_checker_violations()
    );
}

/// 用例自己那一份一次性的库是这个进程独占的一段名字空间（`LibraryOfTheTest`）：同一个用例名建两份，名字空间不同，各带这个进程的进程号
/// （两个进程同跑这个测试二进制各是各的，不按用例名落在同一段——那样录入段开头清库会碰到另一个进程正在用的库）。
/// 判别力（`crates/mutations.tsv`）：名字空间退回只由标签定，两份撞成一段，这条红。
#[test]
fn two_libraries_of_one_test_name_in_one_process_are_different_namespaces_named_by_the_process() {
    const TEST_NAME: &str =
        "two_libraries_of_one_test_name_in_one_process_are_different_namespaces_named_by_the_process";
    let Some(one) = LibraryOfTheTest::create_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(another) = LibraryOfTheTest::create_or_report_not_run(TEST_NAME) else {
        return;
    };
    let namespace_of = |library: &LibraryOfTheTest| match library.key_space() {
        singlefs_verification::verdict_store::LibraryKeySpace::DisposableLibrary(namespace) => {
            String::from_utf8_lossy(&namespace).into_owned()
        }
        singlefs_verification::verdict_store::LibraryKeySpace::TheOneLibrary => {
            panic!("用例自己那一份不会是全仓那一份")
        }
    };
    assert_ne!(
        namespace_of(&one),
        namespace_of(&another),
        "同一个用例名的两份库是两段名字空间"
    );
    let process_identifier = std::process::id().to_string();
    for library in [&one, &another] {
        let namespace = namespace_of(library);
        assert!(
            namespace.contains(&process_identifier),
            "名字空间 {namespace} 带这个进程的进程号 {process_identifier}"
        );
    }
}
