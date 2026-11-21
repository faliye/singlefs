//! 跑着的流：三条流从建池到收尾的驱动的单测。

use super::*;

/// 单测里判池级 checker 的给定状态引擎（「能走两段的都走两段」）：每张要判的盘作一张给定状态表（基线就是那张盘、写表空、
/// 一个空持久集合），录入段写进这一次新建的一次性库（临时 prune-server 里这一次独占的一段名字空间，`crash_library::DisposableLibrary`；
/// 全仓那一份不碰；临时 prune-server 由它自己在这个进程里起，起不来当场失败），
/// 判段从库里读回判（配了 GPU 且编进了 `gpu` 特性在卡上判，否则由 CPU 判器判）。交回每张盘上池级 checker 判到而没违例的不变量与违例的不变量；
/// 只录或只判那一趟（两段分进程跑，拿不到同一份一次性库，打「本次未跑」那一行）、表归别的机器判的那一趟交 `None`（用例当场返回）。
/// 为什么每次都新建库：判到位库里不存，从库里复用来的结论判到位读回是 0（`crash_judge_conclusions` 的
/// `cross_check_words_of_a_stored_given_state`），在长期库里判时这几张从前一趟复用，「判到而成立」一条都没有
/// ；所以收尾核引擎用的库就是这一份一次性库、一张都没复用。
///
/// # Panics
/// 一次性库建不了、引擎起不了、判不了某一张（不退回别处判）；GPU 判不了某一张（判红）；
/// 引擎用的库不是这一份一次性库，或有一张是从库里复用的。
fn pool_checker_holds_and_violations_through_the_given_states_engine(
    images: &[(String, MemoryPool)],
) -> Option<Vec<(Vec<&'static str>, Vec<&'static str>)>> {
    use singlefs_verification::crash_amplification::{
        one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
    };
    use singlefs_verification::crash_given_states::{
        GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
    };
    use singlefs_verification::crash_library::{DisposableLibrary, RunStage};
    use singlefs_verification::verdict_store::PruneServerFailureHandling;
    const TEST_NAME: &str = "e171_three_oracle_collusion_after_the_oracle_fixes::tests::fixed";
    let library_of_the_judgement = match RunStage::of_this_process() {
        // 一次性库留着：钉这一趟一张都不复用（引擎用的就是这一份、每张现判）
        RunStage::RecordThenJudge => DisposableLibrary::new("e171-given-states-judged-afresh")
            .unwrap_or_else(|reason| panic!("{TEST_NAME} 的一次性库建不了：{reason}")),
        stage @ (RunStage::RecordOnly | RunStage::JudgeOnly) => {
            println!(
                "{}",
                singlefs_verification::gpu_memory_ledger::gpu_not_run_line(
                    TEST_NAME,
                    &format!(
                        "这几张盘在一次性库里现判，录入段与判段分进程跑（这一趟只跑 {}）时两段拿不到同一份库",
                        stage.name()
                    )
                )
            );
            return None;
        }
    };
    // 这一次新建的一次性库是空的：设成这个进程开的那一份，引擎起步时开的就是它（进程内的设置，不是环境变量）
    let _the_library_of_this_process =
        library_of_the_judgement.make_it_the_library_of_this_process();
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: TEST_NAME.to_string(),
            },
            &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let mut judged_images = Vec::with_capacity(images.len());
    let mut every_table_judged_here = true;
    for (table_name, image) in images {
        let judged = engine
            .judge_the_given_states(&GivenStatesOfATable {
                table_name: table_name.clone(),
                base: image,
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
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
                every_table_judged_here = false;
            }
        }
        let [conclusion] = judged.states.as_slice() else {
            panic!(
                "{table_name}：一个状态一份结论，交回 {} 份",
                judged.states.len()
            );
        };
        let invariants_with = |bits: u128| -> Vec<&'static str> {
            singlefs_checker::image::IMPLEMENTED_INVARIANTS
                .iter()
                .enumerate()
                .filter(|(position, _)| bits & (1u128 << position) != 0)
                .map(|(_, invariant)| *invariant)
                .collect()
        };
        judged_images.push((
            invariants_with(
                conclusion.words.pool_checker_judged_bits
                    & !conclusion.words.pool_checker_violated_bits,
            ),
            invariants_with(conclusion.words.pool_checker_violated_bits),
        ));
    }
    let run = engine.finish();
    assert_eq!(
        run.red_items(),
        Vec::<String>::new(),
        "GPU 判不了的盘照第三轮正文 H6 判红"
    );
    let library_name_of_the_judgement = library_of_the_judgement
        .open_store(PruneServerFailureHandling::ReturnTheError)
        .unwrap_or_else(|reason| panic!("{TEST_NAME} 的一次性库开不了：{reason}"))
        .library_name()
        .to_string();
    assert_eq!(
        run.library_name, library_name_of_the_judgement,
        "这几张盘判在这一次新建的一次性库里，不在全仓那一份里"
    );
    assert_eq!(
        run.tally.tables_reused, 0,
        "在这一次新建的一次性库里判，结论都该是这一趟现判的（判到位库里不存，复用来的读回是 0）"
    );
    every_table_judged_here.then_some(judged_images)
}

/// 真池上走一遍取号、暖机：O1-对拍那一栏没有红（照抄原装置那一条：核心改坏取号逐盘核时这一条红，装置不记红时这一条不红）。
#[test]
fn an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red() {
    let mut live = LiveStream::start(
        StreamName::NewPoolFileCreation,
        "e171r1-unit-acquire-and-warm-up-comparison",
        RunSwitches::default(),
    );
    let _ = live.step_acquire_and_warm_up();
    let calls_red_in_the_comparison: Vec<&'static str> = live
        .driver
        .calls
        .iter()
        .filter(|call| matches!(call.model, ModelVerdictOfCall::Disagrees { .. }))
        .map(|call| call.label)
        .collect();
    assert!(
        calls_red_in_the_comparison.is_empty(),
        "取号或暖机被拒、O1-对拍记了红：{calls_red_in_the_comparison:?}"
    );
}

/// 真池上取号、暖机的绝对值：两次调用都成功、模型都不比、模型没停、第一次取到的号是 1；两次调用都记了交回值与五样内存对象。
#[test]
fn an_acquire_and_a_warm_up_on_a_fresh_pool_succeed_and_the_model_is_not_asked() {
    let mut live = LiveStream::start(
        StreamName::NewPoolFileCreation,
        "e171r1-unit-acquire-and-warm-up-absolute",
        RunSwitches::default(),
    );
    assert!(live.step_acquire_and_warm_up());
    let labels: Vec<&'static str> = live.driver.calls.iter().map(|call| call.label).collect();
    assert_eq!(labels, vec!["acquire", "warm_up"]);
    assert!(live
        .driver
        .calls
        .iter()
        .all(|call| call.outcome == CallOutcome::Succeeded
            && call.model == ModelVerdictOfCall::NotModelled
            && call.return_value.is_some()
            && call.memory_objects.len() == 5));
    assert_eq!(live.driver.model_stopped_at_call, None);
    assert_eq!(live.instance, Some(InstanceGeneration(1)));
}

/// 流三照读法 2 驱动：删第一个文件（意图一次、走空一步）、一次建一个 inode 到叶容器第一次变成两片，再多建一次
/// （V9：停在叶容器变成两片的那一次（第 K 次），少建之后那一次，这里红）。取号、暖机之外的每次调用都对拍过、对得上，变成两片的那一次与之后那一次也是
/// （理想模型那件修理之后胶水按片记叶容器的角色，登记修订 2 第 2.2 节）：没有一次「模型接不了这份观测」，模型一次都不停。
#[test]
fn the_third_stream_stops_one_creation_after_the_inode_leaf_splits() {
    let mut live = LiveStream::start(
        StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
        "e171r1-unit-third-stream",
        RunSwitches::default(),
    );
    live.run_the_script();
    let split = live.split_publish.expect("叶容器变成了两片");
    assert_eq!(live.inode_creations, split + 1);
    assert_eq!(
        live.output
            .as_ref()
            .map(|output| output.inode_leaf_containers.len()),
        Some(2)
    );
    assert_eq!(live.unlink_calls, 2);
    assert!(live
        .driver
        .calls
        .iter()
        .all(|call| call.outcome == CallOutcome::Succeeded));
    assert!(!live.driver.calls.iter().any(|call| {
        matches!(
            call.model,
            ModelVerdictOfCall::ModelCannotTakeTheObservation { .. }
        )
    }));
    assert_eq!(live.driver.model_stopped_at_call, None);
    assert!(live.driver.calls.iter().all(|call| {
        call.model == ModelVerdictOfCall::Agrees
            || call.label == "acquire"
            || call.label == "warm_up"
    }));
}

/// 流二的卸载经模型的卸载答对拍：基线上那一次 `agrees`；阳性对照「O1-对拍-卸载」把观测里第一条根的 F 加一，恰那一次 `disagrees`
/// （V1：卸载那一步不调模型的卸载答，这里红）。
#[test]
fn the_unmount_is_compared_with_the_model_and_its_control_turns_only_the_unmount_red() {
    let mut clean = LiveStream::start(
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
        "e171r1-unit-unmount-clean",
        RunSwitches::default(),
    );
    clean.run_the_script();
    let unmount_of = |live: &LiveStream| -> ModelVerdictOfCall {
        live.driver
            .calls
            .iter()
            .find(|call| call.label == UNMOUNT_CALL_LABEL)
            .expect("流二走到了卸载")
            .model
            .clone()
    };
    assert_eq!(unmount_of(&clean), ModelVerdictOfCall::Agrees);
    assert_eq!(clean.driver.model_stopped_at_call, None);
    let mut controlled = LiveStream::start(
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
        "e171r1-unit-unmount-control",
        RunSwitches {
            positive_control: Some(PositiveControl::ComparisonOfTheUnmount),
            disabled_oracle: None,
            writable_mount_probe: false,
        },
    );
    controlled.run_the_script();
    assert!(matches!(
        unmount_of(&controlled),
        ModelVerdictOfCall::Disagrees { .. }
    ));
    assert_eq!(
        controlled
            .driver
            .calls
            .iter()
            .filter(|call| matches!(call.model, ModelVerdictOfCall::Disagrees { .. }))
            .count(),
        1
    );
}

// ─────────── 登记修订 6 第 6.5 节：U10–U12（根环表）与 U13–U16（Q5 判决路四个出口） ───────────

/// U11 截停的那一次：流三建到第 30 个 inode（调查报告第 4 节 N = 30 同形，最新根 txg 35），环已转过 24 槽。
const INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING: usize = 30;
/// U11 取样点敏感的那一条：末次发布的最新根 txg 至少这么大（txg 26 起的那条根才盖掉 mkfs 之后第一条还引用旧单元的根，发布不到
/// txg 26 时不装根环表也是绿，`.claude/rules/mutation-sampling.md` 第三类）。
const LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING: u64 = 30;
/// core 断言（`crates/singlefs-core/src/allocator.rs` 的 `PoolAllocator::record_root_written_by_this_process`，
/// `expect` 的消息）里独有的一段原文；U12 只认这一段。
const CORE_ASSERTION_TEXT_OF_AN_ALLOCATOR_WITHOUT_A_ROOT_RING: &str =
    "分配器没装根环表却记了一条根";

/// 修订 6 之前的造法（旧冻结树装置 `LiveStream::start` 第 1069–1083 行同形）：`PoolAllocator::new` 加 `mark_format_time_units`，
/// 不装根环表。只在 U12 里用。
fn allocator_without_the_root_ring_as_before_revision_6() -> PoolAllocator {
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    allocator
}

/// U11 用的截停帮手（只在用例里用）：流三照 [`LiveStream::run_the_script`] 与 [`LiveStream::run_the_third_stream_after_the_first_file`]
/// 的次序走取号、暖机、第一个文件、删第一个文件（意图、再一步一步走空），再一次建一个 inode、建到第 `inode_creations` 次为止。
/// 任一次调用不成功就停在那里交回（用例判每次调用都成功）。迭代上界：走空那一段 `THIRD_STREAM_UNLINK_STEP_LIMIT`，建 inode `inode_creations`。
fn third_stream_through_inode_creations(inode_creations: usize) -> LiveStream {
    let mut live = LiveStream::start(
        StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
        "e171r1-unit-third-stream-past-one-turn-of-the-root-ring",
        RunSwitches::default(),
    );
    if !live.step_acquire_and_warm_up() || !live.step_publish_first_file() {
        return live;
    }
    let write_time_seconds = LOCAL_FIXED_WRITE_TIME_SECONDS + 60;
    let first_inode = InodeNumber(FIRST_INODE_NUMBER);
    if !live.step_third_stream_publish(
        "unlink_intent",
        IdealModel::answer_unlink_intent,
        &|writer, allocator, previous, instance| {
            publish_unlink_intent(
                writer,
                allocator,
                previous,
                first_inode,
                write_time_seconds,
                instance,
            )
        },
    ) {
        return live;
    }
    for _ in 0..THIRD_STREAM_UNLINK_STEP_LIMIT {
        let Some(inode) = live
            .output
            .as_ref()
            .and_then(|output| output.deleted_inodes.first().copied())
        else {
            break;
        };
        if !live.step_third_stream_publish(
            "unlink_step",
            IdealModel::answer_deletion_step,
            &|writer, allocator, previous, instance| {
                publish_unlink_step(
                    writer,
                    allocator,
                    previous,
                    inode,
                    UnlinkBatchSize::OneExtentLowerLeaf,
                    instance,
                )
            },
        ) {
            return live;
        }
    }
    for _ in 0..inode_creations {
        if !live.step_third_stream_publish(
            "create_inode",
            IdealModel::answer_create_inode,
            &|writer, allocator, previous, instance| {
                publish_new_inodes(writer, allocator, previous, 1, write_time_seconds, instance)
            },
        ) {
            return live;
        }
        live.inode_creations += 1;
    }
    live
}

/// U10（登记修订 6 第 6.5 节；V32：装根环表那一句删掉，三条流都由 `Some` 变 `None`、S1 启动核对不过，这里红；与 core 带不带
/// 断言无关）：三条流各 `LiveStream::start` 一次、不驱动，分配器都装着根环表，`{:?}` 文本与
/// `root_ring_occupancy_after_make_filesystem` 给这次 mkfs 现算的那一份相同；第 6.2 节第 1 条那条 S1 在三条流上都过。
#[test]
fn every_live_stream_starts_with_the_root_ring_of_make_filesystem() {
    for name in STREAM_NAMES {
        let label = name.directory_name();
        let live = LiveStream::start(
            name,
            &format!("e171r1-unit-root-ring-at-the-start-{label}"),
            RunSwitches::default(),
        );
        let installed = live
            .allocator
            .root_ring_occupancy()
            .unwrap_or_else(|| panic!("{label}：分配器没装根环表"));
        assert_eq!(
            format!("{installed:?}"),
            format!(
                "{:?}",
                root_ring_occupancy_after_make_filesystem(
                    &experiment_pool::parameters(),
                    &live.formatted.genesis
                )
            ),
            "{label}：装的是 mkfs 刚写下的那一份根环表"
        );
        let check_name = format!("{label}_allocator_carries_the_root_ring_of_make_filesystem");
        let root_ring_checks: Vec<&StopCheck> = live
            .checks
            .iter()
            .filter(|check| check.name == check_name)
            .collect();
        assert_eq!(
            root_ring_checks.len(),
            1,
            "{label}：start 里记了恰一条 {check_name}"
        );
        assert!(
            root_ring_checks[0].holds(),
            "{label}：S1 启动核对过：local={} crates={}",
            root_ring_checks[0].local,
            root_ring_checks[0].crates
        );
    }
}

/// U11（登记修订 6 第 6.5 节；V32 下两种 core 都红：带断言的 core 在第一个文件那次发布 panic，「每次调用都成功」红；
/// 不带断言的 core 不回收，txg 26 起 I-3.1 红）：流三驱动到第 30 次建 inode 为止，每次调用都成功；取号、暖机之后的每次发布之后，
/// 对整条流施加出的镜像（全部写落地）判 I-3.1 都成立；末次发布的最新根 txg 至少 30（根环已转过 24 槽，取样点敏感）。
#[test]
fn the_third_stream_past_one_turn_of_the_root_ring_keeps_the_allocated_counts_matching() {
    let live = third_stream_through_inode_creations(INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING);
    let unsuccessful_calls: Vec<(usize, &'static str, &CallOutcome)> = live
        .driver
        .calls
        .iter()
        .enumerate()
        .filter(|(_, call)| call.outcome != CallOutcome::Succeeded)
        .map(|(call_index, call)| (call_index, call.label, &call.outcome))
        .collect();
    assert!(
        unsuccessful_calls.is_empty(),
        "每次调用都成功：{unsuccessful_calls:?}"
    );
    assert_eq!(
        live.inode_creations, INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING,
        "建 inode 建到了截停那一次"
    );
    let labels_before_the_publishes: Vec<&'static str> = live
        .driver
        .calls
        .iter()
        .take(2)
        .map(|call| call.label)
        .collect();
    assert_eq!(
        labels_before_the_publishes,
        vec!["acquire", "warm_up"],
        "前两次调用是取号、暖机，不经分配器"
    );
    let operations = live.formatted.retained_operations();
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    let mut image = live.formatted.memory_pool_after_mkfs();
    // 跨轮带的是镜像上已经施加到 mkfs 之后第几条操作；每一轮只补施加这一次调用发出的那一截。
    let mut operations_applied_after_mkfs = 0usize;
    let mut images_after_each_publish: Vec<(String, MemoryPool)> = Vec::new();
    for (publish_index, call) in live.driver.calls.iter().skip(2).enumerate() {
        image.apply(
            &operations[mkfs_operation_count + operations_applied_after_mkfs
                ..mkfs_operation_count + call.return_point],
        );
        operations_applied_after_mkfs = call.return_point;
        images_after_each_publish.push((
            format!("third-stream/after-publish-{publish_index}-{}", call.label),
            image.clone(),
        ));
    }
    let Some(judged_images) = pool_checker_holds_and_violations_through_the_given_states_engine(
        &images_after_each_publish,
    ) else {
        return;
    };
    let mut publishes_judged = 0usize;
    for (call, (held, violated)) in live.driver.calls.iter().skip(2).zip(&judged_images) {
        assert!(
            held.contains(&"I-3.1"),
            "第 {publishes_judged} 次发布（{}）之后 I-3.1 成立：判到而成立的 {held:?}，违例的 {violated:?}",
            call.label
        );
        publishes_judged += 1;
    }
    assert_eq!(
        publishes_judged,
        live.driver.calls.len() - 2,
        "取号、暖机之后的每次发布都判过"
    );
    let latest_root_txg = live
        .output
        .as_ref()
        .expect("每次调用都成功，末次发布交回了那一版")
        .root
        .checkpoint_txg;
    assert!(
        latest_root_txg.0 >= LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING,
        "末次发布的最新根 txg 至少 {LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING}：{latest_root_txg:?}"
    );
}

/// U12（登记修订 6 第 6.5 节；只在带 core 断言的 `crates/` 上成立，修订 6 树的 core 不带、那棵树不编用例给
/// core 断言写的变异「删断言」下第 2 次调用成功，这里红）：`LiveStream::start` 之后把分配器换成修订 6 之前的造法（不装根环表），
/// 驱动流一三次调用：取号、暖机成功，第一个文件那次发布在 core 记根时 panic，消息带那条断言独有的一段原文。
#[test]
fn live_stream_without_the_root_ring_panics_in_core_at_its_first_root() {
    let mut live = LiveStream::start(
        StreamName::NewPoolFileCreation,
        "e171r1-unit-without-the-root-ring",
        RunSwitches::default(),
    );
    live.allocator = allocator_without_the_root_ring_as_before_revision_6();
    assert!(
        live.allocator.root_ring_occupancy().is_none(),
        "前提：旧造法不装根环表"
    );
    live.run_the_script();
    let calls = &live.driver.calls;
    let labels: Vec<&'static str> = calls.iter().map(|call| call.label).collect();
    assert_eq!(
        labels,
        vec!["acquire", "warm_up", "publish"],
        "流一三次调用"
    );
    assert_eq!(calls[0].outcome, CallOutcome::Succeeded, "取号成功");
    assert_eq!(calls[1].outcome, CallOutcome::Succeeded, "暖机成功");
    let CallOutcome::Panicked(message) = &calls[2].outcome else {
        panic!("第一个文件那次发布在 core 里 panic：{:?}", calls[2].outcome);
    };
    assert!(
        message.contains(CORE_ASSERTION_TEXT_OF_AN_ALLOCATOR_WITHOUT_A_ROOT_RING),
        "panic 的是第 31 批那条 core 断言：{message}"
    );
}
