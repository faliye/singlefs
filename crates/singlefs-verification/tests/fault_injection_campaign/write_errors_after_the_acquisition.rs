//! 获取之后的写错：系统配置槽上失败的发布冻结、下一次发布盖不掉它的根的单元，获取之后的写错把新实例留在系统配置里、报告点名它，整理崩溃之后的挂载获取之后写错算获取的实例留下、不算新发现。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_fault_injection_campaign::run_the_scenario_where_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
) {
    scenarios_of_fault_injection_campaign::run_the_scenario_where_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(&mut PoolCheckerImages::left_to_the_given_states_engine());
}

/// 起点（mkfs、取号 1、暖机、第一个文件）之后只做一步：关掉会话、可写挂载（取号 2 → 写行 → 暖机）。
fn history_that_mounts_once_after_the_first_file() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::CloseAndMountWritable],
    }
}

/// 那一步可写挂载里的第几次写：取号是头四次（取号写两轮，每轮两块盘各一次系统配置槽写），写行那次发布的第一个单元写是第 5 次；
/// 写行那次发布一共 23 次写（实例表两盘 2 次、分配记录树按位置寻址这一次重写的节点两盘 10 次、记账 / 映射 / 树表各两盘 6 次、
/// 记录两盘 2 次、根槽 1 次、系统配置槽轮换 2 次），暖机第一次空发布的第一个单元写是第 4 + 23 + 1 = 28 次
/// （这一步一共 48 次写：暖机一次 21 次）。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）之前那棵树只有一个节点，
/// 写行那次 15 次写、这一格摆在第 18 次；照旧摆在第 18 次就落进写行那次、记成「新号一条根都没有」。
const FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT: u64 = 5;

const FIRST_WARM_UP_WRITE_OF_THE_MOUNT: u64 = 28;

/// C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）按的那一边（认了，写成已知行为；
/// D23（journal 的角色与格式） 已定项 16 的射程）断言：取号之后写行或暖机那一次写报块设备错，挂载返回错误，
/// 重开的盘上系统配置的实例代号**还是新号 2**（没回卷成 1），这一次记进故障注入的失败账
/// （`FaultInjectionTally::acquired_instances_left_after_a_failed_mount`）、报告里点名。两格各一条：
/// 写行那次的第一个单元写报错 ⇒ 新号一条根都没有（烧掉）；暖机第一次的第一个单元写报错 ⇒ 新号已有写行那次的根。
///
/// 判别力自证：把挂载改成「发布报错之后把系统配置的实例代号写回旧号」（回卷，`crates/mutations.tsv` 里那一条），
/// 重开的盘上系统配置是 1，这一格记不上，这里红。
#[test]
fn write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it(
) {
    let history = history_that_mounts_once_after_the_first_file();
    let mount_step = FaultedSegment::Operation {
        step_index: 0,
        operation_kind: HistoryOperationKind::CloseAndMountWritable,
    };
    for (call_within_the_mount, expected_kind, expected_highest_root_instance) in [
        (
            FIRST_ROW_PUBLISH_WRITE_OF_THE_MOUNT,
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            InstanceGeneration(1),
        ),
        (
            FIRST_WARM_UP_WRITE_OF_THE_MOUNT,
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot,
            InstanceGeneration(2),
        ),
    ] {
        let injection = inject_one_fault_into_the_segment(
            &history,
            HANDING_EVERY_STEP_IMAGE_OUT,
            InjectedFault::WriteFails,
            mount_step,
            call_within_the_mount,
        );
        let rendered = injection.tally.render();
        print_uncaptured(&format!(
            "── C378：可写挂载第 {call_within_the_mount} 次写报错 ──\n{rendered}"
        ));
        assert_eq!(
            injection.acquired_instances_left,
            vec![AcquiredInstanceLeft {
                drawn: injection.drawn_faults[0],
                kind: expected_kind,
                system_configuration_instance_before_the_step: InstanceGeneration(1),
                system_configuration_instance_after_the_reopen: InstanceGeneration(2),
                highest_root_instance_after_the_reopen: Some(expected_highest_root_instance),
            }],
            "取号之后第 {call_within_the_mount} 次写报错：重开的盘上系统配置还是新号 2、不回卷成 1\n{rendered}"
        );
        assert_eq!(
            injection.tally.acquired_instances_left_after_a_failed_mount,
            BTreeMap::from([(expected_kind.name(), 1)]),
            "失败账记下这一次\n{rendered}"
        );
        assert!(
            rendered.contains("取号之后挂载报错、新号不回卷（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）")
                && rendered.contains(&format!("  {}：1 次", expected_kind.name())),
            "报告里点名这一格\n{rendered}"
        );
        // 注入真打在那一步可写挂载的单元写上，那一步返回了错误：不是别处先停了。
        assert_eq!(
            injection.tally.faults_by_injection_point,
            BTreeMap::from([("CloseAndMountWritable/unit_write".to_string(), 1)]),
            "注入点\n{rendered}"
        );
        assert_eq!(
            injection
                .tally
                .faults_by_outcome
                .get(FaultOutcome::SurfacedAsAnError.name())
                .copied(),
            Some(1),
            "可写挂载返回了错误\n{rendered}"
        );
        // 记下的错误成员恰好一次、而且是发布那一步的：空表上「每一个都是」恒真，所以先钉条数。
        assert_eq!(
            injection.tally.refusal_members.values().sum::<u64>(),
            1,
            "注入那一步返回的错误成员记了一次\n{rendered}"
        );
        assert!(
            injection
                .tally
                .refusal_members
                .keys()
                .all(|member| member.starts_with("MountError::Publish(")),
            "返回的是发布那一步的错误成员\n{rendered}"
        );
        assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
        assert!(
            injection.new_findings.is_empty(),
            "认下的行为不算新发现，别的也不许有\n{rendered}"
        );
    }
}

/// 写死的整理历史里强制选的那一段在候选段（0 号盘上装着未释放记录、不是挂载开着的聚簇段，按段起点升序）里的位置，同
/// `compaction_while_mounted_in_random_histories.rs` 的同名常量：起点之后可写挂载一次，候选是第一个文件的数据单元那一段与
/// 第一个文件那次发布的提交内生块那一段，取后一段。
const REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH: u64 = 1;

/// 起点（mkfs、取号 1、暖机、第一个文件）之后：可写挂载（取号 2）；挂着时整理第一个文件那次发布的那一段，一批一个、推一批就崩，
/// 再可写挂载（取号 3 → 写行 → 暖机）。整理的签发与那一批在号 2 下发。
fn history_whose_compaction_crashes_after_one_batch_then_mounts_writable() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector: REGION_SELECTOR_OF_THE_SEGMENT_OF_THE_FIRST_PUBLISH,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            }),
        ],
    }
}

/// 不注入跑一遍这段历史，核第 `step_index` 步是一趟发了批、崩掉之后又可写挂载做成的整理，数这一步发出的写里头一次写下实例代号
/// `acquired_instance` 的根的那次根槽写是这一步的第几次写（1 起）：推批的几次发布写的根带的是这次挂载之前的号，带新号的头一条根是
/// 批间崩溃之后那次挂载写行那次发布的根，取号那两次系统配置槽写在它之前。`inject_one_fault_into_the_segment` 按「这一段发出的第几次写」摆注入点。
fn ordinal_of_the_row_publish_root_slot_write_of_the_mount_after_the_compaction_crash(
    history: &GeneratedHistory,
    step_index: usize,
    acquired_instance: InstanceGeneration,
) -> u64 {
    let stream = SharedStream::retaining_contents();
    let mut stream_length_after: BTreeMap<StepPosition, usize> = BTreeMap::new();
    let run = execute_history_with(
        history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        &stream,
        &mut |observation| {
            stream_length_after.insert(observation.position, stream.operation_count());
        },
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "不注入时这段历史跑完：{:?}",
        run.outcomes
    );
    assert!(
        matches!(
            run.outcomes[step_index],
            StepOutcome::Applied(AppliedEffect::Compacted {
                batches_published: 1..,
                crashed_then_mounted: true,
                ..
            })
        ),
        "前提：第 {step_index} 步整理发了批、崩掉之后可写挂载做成：{:?}",
        run.outcomes[step_index]
    );
    let step_start = match step_index.checked_sub(1) {
        None => stream_length_after[&StepPosition::StartingPoint],
        Some(previous) => stream_length_after[&StepPosition::Operation(previous)],
    };
    let step_end = stream_length_after[&StepPosition::Operation(step_index)];
    let geometry = HANDING_EVERY_STEP_IMAGE_OUT.device_width.fixed_geometry();
    let writes_of_the_step: Vec<_> = stream.retained_operations()[step_start..step_end]
        .iter()
        .filter(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
        .cloned()
        .collect();
    let index = writes_of_the_step
        .iter()
        .position(|retained| {
            geometry.classify(&retained.operation) == StepKind::RootRecordFua
                && root_identity_written_by(
                    retained
                        .contents
                        .as_deref()
                        .expect("开了内容保留的流，根槽写带着字节"),
                )
                .0 == acquired_instance
        })
        .unwrap_or_else(|| panic!("第 {step_index} 步没写过带实例代号 {acquired_instance:?} 的根"));
    u64::try_from(index + 1).expect("一步至多几百次写")
}

/// C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）落在挂着时整理那一步里（判决 `research/prompts/m4-segment-code-r3-main-verification.md`
/// 第三节 J5 的后续：故障注入的比重 `broad-compaction` 抽得到整理）：整理推一批、崩掉之后可写挂载一次（取号 3），写行那次发布的根槽写报块设备错，
/// 挂载返回错误；重开的盘上系统配置的实例代号还是新号 3（这一步之前是 2，整理那几次发布不换号），根环里没有带号 3 的根。
/// 这一次认成取号之后挂载报错、新号没回卷那一格（`acquired_instances_left`、失败账），不算新发现。
///
/// 判别力自证：故障注入把整理那一步当成不取号的操作（`system_configuration_instance_before_a_mount_step` 那一臂交 `None`，
/// `crates/mutations.tsv` 里那一条），这一格记不上，这里红。
#[test]
fn write_error_after_the_acquisition_of_the_mount_after_a_compaction_crash_counts_as_the_acquired_instance_left_and_not_a_new_finding(
) {
    const COMPACTION_STEP: usize = 1;
    let instance_before_the_compaction_step = InstanceGeneration(2);
    let instance_acquired_after_the_crash = InstanceGeneration(3);
    let history = history_whose_compaction_crashes_after_one_batch_then_mounts_writable();
    let call_within_the_step =
        ordinal_of_the_row_publish_root_slot_write_of_the_mount_after_the_compaction_crash(
            &history,
            COMPACTION_STEP,
            instance_acquired_after_the_crash,
        );
    let injection = inject_one_fault_into_the_segment(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::WriteFails,
        FaultedSegment::Operation {
            step_index: COMPACTION_STEP,
            operation_kind: HistoryOperationKind::CompactWhileMounted,
        },
        call_within_the_step,
    );
    let rendered = injection.tally.render();
    print_uncaptured(&format!(
        "── C378 落在整理那一步：批间崩溃之后的可写挂载、这一步第 {call_within_the_step} 次写报错 ──\n{rendered}"
    ));
    // 注入真打在整理那一步的根槽写上，那一步返回了挂载发布的错误：不是别处先停了。
    assert_eq!(
        injection.tally.faults_by_injection_point,
        BTreeMap::from([("CompactWhileMounted/root_record_fua".to_string(), 1)]),
        "注入点\n{rendered}"
    );
    assert_eq!(
        injection
            .tally
            .faults_by_outcome
            .get(FaultOutcome::SurfacedAsAnError.name())
            .copied(),
        Some(1),
        "整理那一步返回了错误\n{rendered}"
    );
    assert!(
        injection.tally.refusal_members.values().sum::<u64>() == 1
            && injection
                .tally
                .refusal_members
                .keys()
                .all(|member| member.starts_with("MountError::Publish(")),
        "返回的是挂载里发布那一步的错误成员，恰好一次\n{rendered}"
    );
    assert_eq!(
        injection.acquired_instances_left,
        vec![AcquiredInstanceLeft {
            drawn: injection.drawn_faults[0],
            kind: AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot,
            system_configuration_instance_before_the_step: instance_before_the_compaction_step,
            system_configuration_instance_after_the_reopen: instance_acquired_after_the_crash,
            highest_root_instance_after_the_reopen: Some(instance_before_the_compaction_step),
        }],
        "取号之后写行那次的根槽写报错：重开的盘上系统配置还是新号 3、根环里最大的号是 2，记进这一格\n{rendered}"
    );
    assert_eq!(
        injection.tally.acquired_instances_left_after_a_failed_mount,
        BTreeMap::from([(
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot.name(),
            1
        )]),
        "失败账记下这一次\n{rendered}"
    );
    assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
    assert!(
        injection.new_findings.is_empty(),
        "认下的行为不算新发现，别的也不许有：{:?}\n{rendered}",
        injection.new_findings
    );
}
