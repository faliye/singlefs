use super::*;

/// 整条路在一个小池上走一遍（每块盘取默认环长下 mkfs 认的最小一块 3072 MiB、P = 2048：内存盘懒分配，没写过的块不占内存；
/// 真实三档的造池太久，进不了同包测试）：造起步镜像（一次发布写满一个 16 单元的文件，活数据文件与等长的交错垫片文件交替写、删垫片）、
/// 几何成形（碎片段九成以上恰好 32 槽活、九成以上活槽恰好是段的前 32 槽，greedy 照核心层重放选中只装活数据单元的段），CPU 臂与 CPU 多判臂各交替跑一次，
/// 都真搬了批、写的与读到的逐个相同、读到的等于期望，每一项记账照设置走 CPU、条数不少于批数，多判臂的里层计时大于 0、对不上的条数是 0。
/// 一个场景：同一份起步镜像上两条臂按次序跑。
#[test]
fn the_small_pool_relocates_batches_and_the_cpu_arms_write_and_read_the_same() {
    let image = small_pool_image();
    // 几何核两半都断言：后一半判强制选段后备点名的那一段（登记第十二节修订第 8 条）；greedy 照核心层重放要选的那一段只打出来。
    eprintln!(
        "E202 小池几何：greedy_picks_a_fragment_like_segment={} {:?}",
        image.construction.geometry.greedy.is_fragment_like(),
        image.construction.geometry
    );
    assert!(
        image.construction.geometry.is_formed(),
        "{:?}",
        image.construction
    );
    assert!(
        image.construction.is_every_spacer_slot_released,
        "{:?}",
        image.construction
    );
    assert_eq!(
        image.construction.live_data_units_found,
        SMALL_POOL_LIVE_DATA_UNITS
    );
    // 造池量到的 inode 记录：造数据挂载之前那几条等于三档算术用的本地常量；峰值是它加活数据文件 P ÷ K（小池 2048 ÷ 16 = 128 个）
    // 加垫片文件 P ÷ 32 + 1（64 个交错垫片文件加补齐文件 = 65 个），片数等于核心层写同样条数进空树的片数。
    let construction = &image.construction;
    assert_eq!(
        construction.inode_records_before_the_data_mount, INODE_RECORDS_BEFORE_THE_DATA_MOUNT,
        "{construction:?}"
    );
    assert_eq!(construction.live_data_files, 128, "{construction:?}");
    assert_eq!(construction.spacer_files, 65, "{construction:?}");
    assert_eq!(
        construction.spacer_files,
        spacer_files_of_the_construction(SMALL_POOL_LIVE_DATA_UNITS),
        "{construction:?}"
    );
    // 碎片段里的活槽是前半一整片（活数据文件先于它那个交错垫片文件写），与几何核前一半同一个九成。
    assert!(
        count_as_float(
            construction
                .fragment_live_slot_layout
                .segments_live_in_exactly_the_front_half
        ) >= SHARE_OF_WRITTEN_SEGMENTS_WITH_HALF_LIVE_AT_LEAST
            * count_as_float(construction.geometry.written_segments),
        "{construction:?}"
    );
    assert_eq!(
        construction.inode_records_at_the_peak,
        INODE_RECORDS_BEFORE_THE_DATA_MOUNT + 128 + construction.spacer_files,
        "{construction:?}"
    );
    assert_eq!(
        inode_leaf_containers_of_a_one_level_tree(construction.inode_records_at_the_peak),
        Ok(construction.inode_leaf_containers_at_the_peak),
        "{construction:?}"
    );
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let setting = LockstepRunSetting {
        image: &image,
        discrete_gpus: &[],
        discrete_gpus_of_the_every_card_arm: &[],
        memory_leaked: &memory_leaked,
        vulkan_instance: None,
        user_changes: LOCKSTEP_USER_CHANGES_LADDER[0],
        batch_units: BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT,
    };
    let plain = run_lockstep_once(&setting, LOCKSTEP_ARMS[0], 0).expect("CPU 臂跑得成");
    let rejudged = run_lockstep_once(&setting, LOCKSTEP_ARMS[3], 0).expect("CPU 多判臂跑得成");
    for run in [&plain, &rejudged] {
        assert!(run.batches > 0, "{} 搬了批", run.arm.name());
        assert!(run.content_matches_the_expectation);
        assert_eq!(
            run.path_anchor.compute_records_on_the_path_of_the_settings,
            run.path_anchor.compute_records
        );
        assert!(
            run.path_anchor.compute_records >= run.batches,
            "每一批至少一项记账：{:?}",
            run.path_anchor
        );
        assert!(
            run.most_compaction_publishes_between_two_user_changes
                <= LOCAL_PUBLISH_BUDGET_GIVEN_BY_A_USER_CHANGE
        );
        assert_eq!(
            run.calls_failing_the_inclusion_checks, 0,
            "{:?}",
            run.windows_failing_the_inclusion_checks
        );
        assert!(
            run.forced_fallback.intents_issued() > 0,
            "{:?}",
            run.forced_fallback
        );
        assert_eq!(
            run.forced_fallback.intents_issued_off_the_forced_region(),
            0,
            "{:?}",
            run.forced_fallback
        );
        assert!(
            run.forced_fallback
                .choices
                .iter()
                .all(|choice| choice.forced.is_fragment_like()),
            "{:?}",
            run.forced_fallback
        );
    }
    let forced_region_starts = |run: &LockstepRunSummary| -> Vec<u64> {
        run.forced_fallback
            .choices
            .iter()
            .filter_map(|choice| choice.forced.start_slot)
            .collect()
    };
    assert_eq!(
        forced_region_starts(&plain),
        forced_region_starts(&rejudged)
    );
    assert_eq!(plain.recording_digest, rejudged.recording_digest);
    assert_eq!(plain.content_digest, rejudged.content_digest);
    assert_eq!(plain.batches, rejudged.batches);
    assert!(rejudged.inner_rejudge_cpu > Duration::ZERO);
    assert!(rejudged.rejudge.entries_rejudged > 0);
    assert_eq!(rejudged.rejudge.mismatches, 0);
    assert_eq!(plain.inner_rejudge_cpu, Duration::ZERO);
}

/// 甲类第 1–4 行合判：相邻两次用户改动之间 21 次整理发布、某一项记账走错、多判的批判定加不齐，各判不成立。
#[test]
fn the_class_one_anchors_of_a_lockstep_run_break_past_twenty_publishes_between_two_user_changes() {
    let holding_path = PathAnchor {
        compute_records: 5,
        compute_records_on_the_path_of_the_settings: 5,
        batches: 2,
    };
    let anchors = GpuWorkAnchors::default();
    assert!(class_one_anchors_of_a_lockstep_run_hold(
        holding_path,
        ComputeOfTheArm::Cpu,
        anchors,
        0,
        20
    ));
    assert!(!class_one_anchors_of_a_lockstep_run_hold(
        holding_path,
        ComputeOfTheArm::Cpu,
        anchors,
        0,
        21
    ));
    assert!(!class_one_anchors_of_a_lockstep_run_hold(
        PathAnchor {
            compute_records_on_the_path_of_the_settings: 4,
            ..holding_path
        },
        ComputeOfTheArm::Cpu,
        anchors,
        0,
        20
    ));
    assert!(!class_one_anchors_of_a_lockstep_run_hold(
        holding_path,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        anchors,
        1,
        20
    ));
}
