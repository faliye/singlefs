//! 快照那几步上的两个观察者（里程碑四一·4；崩溃点重放与崩溃放量两路共用一份代码）：读恢复落到的那一版的每个活快照（只记计数），
//! 与可写挂载一次、续做把 unlink 与删快照走到空（交回挂完的盘，判段在它上面跑池级 checker）。四条快照流的崩溃枚举用例调这里的同一份；
//! 崩溃放量那一路照 `common_crash_points::OBSERVERS_ON_THE_SNAPSHOT_STEPS` 把它们挂在快照那几步的节点上（与标准前缀上的可写挂载续做观察者同一个口径：
//! 名字住在设点的公共模块，代码住在这一份，观察者的版本从这一份出发算，`crash_identity::ConclusionVersionRequest::Observer`）。
//! 另读的输入是这条流的每一版（`common_snapshot_prefix::VersionOfTheFlow`），各条快照流不同：同一个快照节点上的观察者结论按流各存一份（去重键带另读的输入）。
#![allow(
    dead_code,
    reason = "共用的观察者模块：每个测试文件只用其中几样，没用的在那个测试二进制里就是没用到"
)]

use super::common_snapshot_prefix::{
    assert_every_live_snapshot_reads_its_creation_content,
    assert_the_mount_continued_to_the_settled_shape, mount_writable_on_the_crash_disks,
    version_landed_on, LandedVersionOfTheFlow, PreparedSnapshotCrashFlow, SnapshotFlowStepKind,
    VersionOfTheFlow, POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER,
};
use singlefs_checker_tier::crash::CrashPointReplayObserverCounts;
use singlefs_checker_tier::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver,
    ObserveACrashStateHandingBackAnImage, ObserverDeduplication, ReadRangeOnTheCrashImage,
};
use singlefs_checker_tier::crash_state_enumeration::{
    CrashPointReplayObserver, CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_core::recovery::RecoveryReport;
use singlefs_harness::memory_pool::CrashImage;

/// 这一份的仓内路径（观察者代码文件，与 `file!()` 归一之后相同）。
pub const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/common_snapshot_crash_point_observers/mod.rs";

/// 读快照那个观察者的名字（与 `common_crash_points::ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion` 的名字相同）。
pub const SNAPSHOT_OF_THE_LANDED_VERSION: &str = "snapshot_of_the_landed_version";

/// 可写挂载续做那个观察者的名字（与 `common_crash_points::ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot` 的名字相同）。
pub const MOUNT_CONTINUATION_UNDER_A_SNAPSHOT: &str = "mount_continuation_under_a_snapshot";

/// 每个状态上核的一样（只记计数）：恢复落到流里哪一版，落到的那一版的每个活快照在这张崩溃镜像上读得出、读出建它那一刻的内容。
/// 只读流里每一版（不碰录流的池：它的录制流不能跨线程共用），各工作线程共用一份。
pub fn observe_the_snapshot_of_the_landed_version(
    versions_of_the_flow: &[VersionOfTheFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let effective_root = report
        .effective_root
        .expect("快档与全量的每个状态都择到根（oracle 另判）");
    match version_landed_on(versions_of_the_flow, effective_root) {
        LandedVersionOfTheFlow::BeforeTheFirstFile => {
            counts.add("landed_before_the_first_file", 1);
        }
        LandedVersionOfTheFlow::Version(version) => {
            let landed = version
                .kind
                .map_or("first_file", SnapshotFlowStepKind::counter_name);
            counts.add(&format!("landed_on_{landed}"), 1);
            let snapshots_read =
                assert_every_live_snapshot_reads_its_creation_content(image, version);
            counts.add(
                "live_snapshots_read_at_their_creation_content",
                u64::try_from(snapshots_read).expect("快照数装得进 u64"),
            );
        }
    }
}

/// 这一状态上可写挂载一次（删除开关同录流）：续做把 unlink 与删快照走到空、步数等于恢复落到的那一版还差几步，挂完之后快照、删除队列、deadlist 与
/// 留存的映射条目与录流时这一串删除的最后一批那一版逐项相同（`common_snapshot_prefix::assert_the_mount_continued_to_the_settled_shape`）。
/// 挂载发的写连同挂完的盘交回：判段在挂完的盘上跑池级 checker（判干净时 [`POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER`] 加一）。
/// 可写挂载与续做真跑实现，只能在 CPU 上（豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行）。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
pub fn observe_the_mount_continuation_under_a_snapshot(
    versions_of_the_flow: &[VersionOfTheFlow],
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let effective_root = report
        .effective_root
        .expect("快档与全量的每个状态都择到根（oracle 另判）");
    let landed = version_landed_on(versions_of_the_flow, effective_root);
    let mounted = mount_writable_on_the_crash_disks(&materialized_crash_image(image));
    let continuation_steps =
        assert_the_mount_continued_to_the_settled_shape(landed, &mounted.mounted);
    counts.add(&format!("mount_continuation_steps_{continuation_steps}"), 1);
    ImageAfterTheObserver {
        writes_on_the_crash_image: mounted.writes_of_the_mount,
        disks_after_the_observer: mounted.disks_after_the_mount,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &mounted.read_ranges_of_the_mount,
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER
            .to_string(),
    }
}

/// 崩溃点重放那一路的读快照观察者（另读的输入是流里每一版的摘要）。
pub fn snapshot_observer_of_every_state<'prepared>(
    prepared: &'prepared PreparedSnapshotCrashFlow,
    observe: &'prepared (dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
                    + Sync),
) -> CrashPointReplayObserver<'prepared> {
    CrashPointReplayObserver {
        name: SNAPSHOT_OF_THE_LANDED_VERSION,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
    }
}

/// 崩溃点重放那一路的可写挂载续做观察者（交回挂完的盘）：录入段在 CPU 上挂、按恢复镜像的字节去重，判段在挂完的盘上判池级 checker。
pub fn mount_continuation_observer_under_a_snapshot<'prepared>(
    prepared: &'prepared PreparedSnapshotCrashFlow,
    observe: &'prepared ObserveACrashStateHandingBackAnImage<'prepared>,
) -> CrashPointReplayObserverHandingBackAnImage<'prepared> {
    CrashPointReplayObserverHandingBackAnImage {
        name: MOUNT_CONTINUATION_UNDER_A_SNAPSHOT,
        extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
        observe,
        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
    }
}

/// 一条快照流按环境跑一趟崩溃放量（展开上限、库目录、判器照 `common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line`），
/// 挂两张登记表上的观察者：标准前缀节点上的（`common_crash_points::OBSERVERS_ON_THE_STANDARD_PREFIX`，可写挂载续做，代码在 `common_crash_point_observers`，
/// 另读删除流的发布表）与快照那几步节点上的（`common_crash_points::OBSERVERS_ON_THE_SNAPSHOT_STEPS`，这一份的两个，另读这条流的每一版）。
/// 观察者走两段：录入段在 CPU 上看（交回镜像的按恢复镜像去重），判段在挂完的盘上判池级 checker（配了 GPU 上卡）；两段各打一行。
/// 观察者计数的断言每一趟都判（这一趟拼的与库里复用的块读回来的合起来）：每个观察者至少看过一个状态、一个状态都不判红；可写挂载的两个观察者
/// 每个状态都挂过一次、挂完的盘都过了池级 checker。只录、只排派活计划、叫停、判段报本次未跑的那一趟不判。
///
/// # Panics
/// 登记表里快照节点的入口名不是 `common_snapshot_prefix::SnapshotFlowStepKind` 的某一个；或上面任何一条断言不成立。
#[cfg(feature = "verdict-store")]
pub fn amplify_a_snapshot_flow(
    flow_name: &str,
    prepared: &PreparedSnapshotCrashFlow,
    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
    default_expand_up_to: usize,
) -> singlefs_checker_tier::crash_amplification::EnvironmentRun {
    use super::common_crash_point_observers;
    use super::common_crash_points::{
        self, ObserverOfTheSnapshotSteps, ObserverOfTheStandardPrefix,
        DELETION_BATCHES_BEFORE_THE_FINAL_BATCH, OBSERVERS_ON_THE_SNAPSHOT_STEPS,
        OBSERVERS_ON_THE_STANDARD_PREFIX,
    };
    use singlefs_checker_tier::crash_amplification::{
        NodeOfAnObserver, ObserverCode, ObserverOnTheNodes,
    };
    use singlefs_checker_tier::crash_identity::ObserverName;
    let entry_names_of_the_snapshot_steps = [
        SnapshotFlowStepKind::CreateSnapshot,
        SnapshotFlowStepKind::OverwriteUnderTheSnapshot,
        SnapshotFlowStepKind::UnlinkIntent,
        SnapshotFlowStepKind::UnlinkFinalBatch,
        SnapshotFlowStepKind::SnapshotDestructionIntent,
        SnapshotFlowStepKind::SnapshotDestructionBatch,
        SnapshotFlowStepKind::SnapshotDestructionFinalBatch,
    ]
    .map(SnapshotFlowStepKind::entry_name);
    for (entry_name, _observers_of_the_node) in OBSERVERS_ON_THE_SNAPSHOT_STEPS {
        assert!(
            entry_names_of_the_snapshot_steps.contains(entry_name),
            "快照节点登记表里的入口名 {entry_name} 是快照流某一步的入口名"
        );
    }
    let mount_continuation_inputs =
        common_crash_point_observers::mount_continuation_inputs_of_the_unlink_stream(&format!(
            "{flow_name}-observer-inputs"
        ));
    let observe_the_mount_on_the_standard_prefix =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            common_crash_point_observers::observe_the_mount_continuation(
                &mount_continuation_inputs,
                image,
                report,
                counts,
            )
        };
    let versions_of_the_flow = prepared.versions_of_the_flow.as_slice();
    let observe_the_snapshot =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_snapshot_of_the_landed_version(versions_of_the_flow, image, report, counts);
        };
    let observe_the_mount_under_a_snapshot =
        |_ordinal: u64,
         image: &CrashImage<'_>,
         report: &RecoveryReport,
         counts: &mut CrashPointReplayObserverCounts| {
            observe_the_mount_continuation_under_a_snapshot(
                versions_of_the_flow,
                image,
                report,
                counts,
            )
        };
    let observer_name = |name: &str| ObserverName::new(name).expect("观察者的名字非空、不长");
    let nodes_of = |entry_names: Vec<&str>| -> Vec<NodeOfAnObserver> {
        entry_names
            .into_iter()
            .map(|entry_name| NodeOfAnObserver {
                setting_file: common_crash_points::THIS_FILE.to_string(),
                entry_name: entry_name.to_string(),
            })
            .collect()
    };
    let mut observers: Vec<ObserverOnTheNodes<'_>> = Vec::new();
    let standard_prefix_observer = ObserverOfTheStandardPrefix::WritableMountAndContinuation;
    observers.push(ObserverOnTheNodes {
        observer_name: observer_name(standard_prefix_observer.name()),
        observer_code_file: common_crash_point_observers::THIS_FILE.to_string(),
        extra_inputs_digest: mount_continuation_inputs.digest(),
        nodes: nodes_of(
            OBSERVERS_ON_THE_STANDARD_PREFIX
                .iter()
                .filter(|(_entry_name, observers_of_the_node)| {
                    observers_of_the_node.contains(&standard_prefix_observer)
                })
                .map(|(entry_name, _observers_of_the_node)| *entry_name)
                .collect(),
        ),
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe_the_mount_on_the_standard_prefix,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        },
    });
    let snapshot_observers = [
        ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion,
        ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot,
    ];
    for snapshot_observer in snapshot_observers {
        observers.push(ObserverOnTheNodes {
            observer_name: observer_name(snapshot_observer.name()),
            observer_code_file: THIS_FILE.to_string(),
            extra_inputs_digest: prepared.digest_of_the_versions_of_the_flow(),
            nodes: nodes_of(
                OBSERVERS_ON_THE_SNAPSHOT_STEPS
                    .iter()
                    .filter(|(_entry_name, observers_of_the_node)| {
                        observers_of_the_node.contains(&snapshot_observer)
                    })
                    .map(|(entry_name, _observers_of_the_node)| *entry_name)
                    .collect(),
            ),
            observe: match snapshot_observer {
                ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion => {
                    ObserverCode::CountsOnly(&observe_the_snapshot)
                }
                ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot => {
                    ObserverCode::HandsBackAnImage {
                        observe: &observe_the_mount_under_a_snapshot,
                        deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
                    }
                }
            },
        });
    }
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            flow_name,
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            prepared.judged_root_index,
            &prepared.versions_with_a_file,
            &prepared.versions_without_a_file,
            crash_points,
            default_expand_up_to,
            &observers,
        );
    if let Some(recording) = &run.observer_recording {
        println!(
            "CRASH_POINT_REPLAY_OBSERVERS_RECORDING flow={flow_name} images_observed_fresh={} states_reusing_an_observed_image={} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={}",
            recording.images_observed_here,
            recording.states_reusing_an_observed_image,
            recording.blocks_recorded_here,
            recording.states_recorded_here,
            recording.blocks_already_recorded
        );
    }
    let Some(observing) = &run.observing else {
        return run;
    };
    println!(
        "CRASH_POINT_REPLAY_OBSERVERS flow={flow_name} images_judged_fresh={} images_judged_on_the_gpu_here={} image_judgements_reused={} blocks_observed_here={} states_observed_here={} states={:?} red={:?}",
        observing.images_judged_here,
        observing.images_judged_on_the_gpu_here,
        observing.image_judgements_reused,
        observing.blocks_observed_here,
        observing.states_observed_here,
        observing.states_of_each_observer,
        observing.red_states_of_each_observer
    );
    let observer_names = [
        standard_prefix_observer.name(),
        ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion.name(),
        ObserverOfTheSnapshotSteps::MountContinuationUnderASnapshot.name(),
    ];
    for name in observer_names {
        let states = observing
            .states_of_each_observer
            .get(name)
            .copied()
            .unwrap_or(0);
        assert!(
            states > 0,
            "快照流 {flow_name} 经过标准前缀与快照节点，观察者 {name} 一个状态都没看"
        );
        assert_eq!(
            observing
                .red_states_of_each_observer
                .get(name)
                .copied()
                .unwrap_or(0),
            0,
            "快照流 {flow_name}：观察者 {name} 判红的状态（这一趟看出来的前几个）：{:?}",
            observing.red_texts
        );
        let counts = observing
            .counts_of_each_observer
            .get(name)
            .cloned()
            .unwrap_or_default();
        let count_named = |counter: &str| counts.get(counter).copied().unwrap_or(0);
        let mount_continuation_steps_at_most = DELETION_BATCHES_BEFORE_THE_FINAL_BATCH.max(
            super::common_snapshot_prefix::SNAPSHOT_DESTRUCTION_BATCHES_BEFORE_THE_FINAL_BATCH,
        ) + 1;
        let is_a_mount_observer =
            name != ObserverOfTheSnapshotSteps::SnapshotOfTheLandedVersion.name();
        if is_a_mount_observer {
            assert_eq!(
                (0..=mount_continuation_steps_at_most)
                    .map(|steps| count_named(&format!("mount_continuation_steps_{steps}")))
                    .sum::<u64>(),
                states,
                "快照流 {flow_name}：观察者 {name} 看的每个状态都可写挂载过一次"
            );
            assert_eq!(
                count_named(POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER),
                states,
                "快照流 {flow_name}：观察者 {name} 看的每个状态挂完的盘都过了池级 checker（判段判，配了 GPU 上卡）"
            );
        }
    }
    run
}
