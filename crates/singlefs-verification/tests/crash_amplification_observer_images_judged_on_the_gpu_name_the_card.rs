//! 验证包模块：crash、crash_identity、crash_library、crash_amplification、verdict_store、gpu_unit_checks、crash_judge_tables
//! 配置开了 GPU 时观察者交回的镜像在卡上判（交回崩溃镜像本身的、把一处系统配置槽重写过判表上限的），结论点名判它的那张卡。
//! 观察者的判法单元按这份文件的路径登记在 `src/judging_units_entries.rs`，改名要连同那几条登记一起改。
//! 库是临时 prune-server 里的一次性库（`crash_library::DisposableLibrary`，临时 prune-server 由它自己在这个进程里起，起不来当场失败）。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
//! 有适配器而点名的卡找不到判红；产品路径此刻借不到显存（流水线那一趟照叫停的样子交回），同样报「本次未跑」。
//! 经环境跑的用例自己清、自己还原环境（K5，公共模块的环境守卫）。要 `gpu` 特性，不带它这份文件是空的。
#![cfg(feature = "gpu")]

mod common;
mod common_crash_points;

use std::path::PathBuf;

use common::{build_pool, file_content, geometry};
use singlefs_verification::crash_amplification::{
    card_priority_lines_of_the_caller, CrashPointSpan,
};
use singlefs_verification::crash_identity::ConclusionKind;
use singlefs_verification::crash_library::DisposableLibrary;
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, not_run_line,
};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::verdict_store::{PruneServerFailureHandling, VerdictStore};

const PATH_OF_THIS_TEST_FILE: &str =
    "crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs";
/// 只展开不超过 6 次写的段：新池新建文件那条流展开的十段共几十个状态，秒级。
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

/// 录好的一条流：崩溃枚举要的五样加两个崩溃点。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    crash_points: Vec<CrashPointSpan>,
}

/// mkfs → 取号 → 两次暖机 → 新池新建文件，按 `build_pool` 的做法录；崩溃点按录制流里暖机与发布的分界切
/// （同 `crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs`，设点的文件是这一份）。
fn record_new_pool_file_creation(tag: &str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let publish_boundary = pool.warm_up_operation_count - pool.mkfs_operation_count;
    let first_publish_write = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= publish_boundary)
        .expect("发布至少一次写");
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    let versions = vec![PublishedVersion {
        instance: pool.output.root.instance,
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let crash_points = vec![
        CrashPointSpan {
            name: "acquire_instance_and_warm_up".to_string(),
            code_file_in_repository: PATH_OF_THIS_TEST_FILE.to_string(),
            writes: 0..first_publish_write,
            ignored: false,
        },
        CrashPointSpan {
            name: "publish_first_file".to_string(),
            code_file_in_repository: PATH_OF_THIS_TEST_FILE.to_string(),
            writes: first_publish_write..writes.len(),
            ignored: false,
        },
    ];
    RecordedFlow {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions,
        crash_points,
    }
}

/// 用例的库与目录：一份一次性库（临时 prune-server 里这条用例独占的一段名字空间，起跑是空的，全仓那一份一行不写），它那份一次性临时目录里
/// 放一份开了 GPU 的多机配置（显卡先后照抄调用方给这个进程的那一份，`card_priority_lines_of_the_caller`；清场之下要清场之后那一份）。
struct GpuRunDirectory {
    configuration: PathBuf,
    library: DisposableLibrary,
}

impl GpuRunDirectory {
    fn new(tag: &str) -> Self {
        // 一次性库留着：钉交回镜像在卡上判、行里点名卡：要在卡上现判，复用了就不上卡
        let library = DisposableLibrary::new(&format!("gpu-run-{tag}"))
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let configuration = library.join("multi-host-with-gpu.env");
        let cards = card_priority_lines_of_the_caller(
            &std::env::current_dir().expect("取得到这个进程的当前目录（多机配置从这里起找）"),
        )
        .expect("本机的多机配置读得了");
        std::fs::write(&configuration, format!("ENABLE_GPU=1\n{cards}"))
            .expect("写得了开 GPU 的配置");
        Self {
            configuration,
            library,
        }
    }

    /// 这条用例那一份库的一个新客户端。
    fn open_the_library(&self) -> VerdictStore {
        self.library
            .open_store(PruneServerFailureHandling::ReturnTheError)
            .unwrap_or_else(|refusal| panic!("开不了这条用例的库：{refusal}"))
    }
}

/// 观察者「交回崩溃镜像本身」（第一条 GPU 观察者用例）：一个写都不发，交回的盘就是崩溃镜像按持久集合落好的那一份，判每一条不变量。
/// 观察者的身份是这个 fn 的闭包键（入口登记在 `singlefs_verification::judging_units_entries::OBSERVER_ENTRIES`）。
fn hand_back_the_crash_image(
    _ordinal: u64,
    image: &singlefs_verification::memory_pool::CrashImage<'_>,
    _report: &singlefs_core::recovery::RecoveryReport,
    _counts: &mut singlefs_verification::crash::CrashPointReplayObserverCounts,
) -> singlefs_verification::crash_amplification::ImageAfterTheObserver {
    let mut disks_after_the_observer = image.base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            disks_after_the_observer.apply_writes(std::slice::from_ref(write));
        }
    }
    singlefs_verification::crash_amplification::ImageAfterTheObserver {
        writes_on_the_crash_image: Vec::new(),
        ranges_read_on_the_crash_image:
            singlefs_verification::crash_amplification::ReadRangeOnTheCrashImage::whole_pool(
                &disks_after_the_observer,
            ),
        disks_after_the_observer,
        invariants_judged:
            singlefs_verification::crash_amplification::InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: "pool_checker_clean_on_the_crash_image".to_string(),
    }
}

/// 观察者「把第一块盘的系统配置槽 0 号原样重写过判表一个落点的写数上限」：在崩溃镜像上把那一槽原样重写
/// `MAXIMUM_WRITES_PER_LOCATION` 加 6 次（字节与崩溃镜像上那一槽相同，盘不变），判每一条不变量。身份同上，入口登记在同一张表。
fn rewrite_the_first_system_configuration_slot_past_the_judge_table_limit(
    _ordinal: u64,
    image: &singlefs_verification::memory_pool::CrashImage<'_>,
    _report: &singlefs_core::recovery::RecoveryReport,
    _counts: &mut singlefs_verification::crash::CrashPointReplayObserverCounts,
) -> singlefs_verification::crash_amplification::ImageAfterTheObserver {
    use singlefs_verification::crash_amplification::{
        ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver,
    };
    use singlefs_verification::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION;
    let rewrites_of_the_slot = MAXIMUM_WRITES_PER_LOCATION + 6;
    let mut disks_after_the_observer = image.base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            disks_after_the_observer.apply_writes(std::slice::from_ref(write));
        }
    }
    let first_device = *disks_after_the_observer
        .devices
        .keys()
        .next()
        .expect("池里至少一块盘");
    let slot_bytes = singlefs_core::recovery::PoolReader::read(
        &disks_after_the_observer,
        first_device,
        singlefs_core::address::DeviceOffsetInBytes(0),
        4096,
    )
    .expect("系统配置槽 0 号在盘内");
    let writes_on_the_crash_image: Vec<singlefs_verification::memory_pool::RetainedWrite> = (0
        ..rewrites_of_the_slot)
        .map(
            |_rewrite| singlefs_verification::memory_pool::RetainedWrite {
                device: first_device,
                kind: singlefs_verification::segments::StepKind::SystemConfigurationSlot,
                is_force_unit_access: true,
                offset: singlefs_core::address::DeviceOffsetInBytes(0),
                contents: singlefs_verification::memory_pool::WrittenContents::Bytes(
                    slot_bytes.clone(),
                ),
            },
        )
        .collect();
    disks_after_the_observer.apply_writes(&writes_on_the_crash_image);
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        ranges_read_on_the_crash_image:
            singlefs_verification::crash_amplification::ReadRangeOnTheCrashImage::whole_pool(
                &disks_after_the_observer,
            ),
        disks_after_the_observer,
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: "pool_checker_clean_after_the_rewrites".to_string(),
    }
}

/// 观察者走两段、池级 checker 上 GPU：配置开了 GPU，
/// 一个交回镜像的观察者（交回崩溃镜像本身、一个写都不发，判每一条不变量）挂在发布那个点上：录入段在 CPU 上看、按恢复镜像去重，
/// 判段在卡上判观察者交回的盘——这一趟判的盘都在卡上判（`images_judged_on_the_gpu_here` 等于 `images_judged_here`），库里观察者那几块
/// 结论记的在哪判的是 GPU（卡型与驱动），每个状态判干净（新池新建文件这条流第 ① 段本来就一个红状态都没有）。
/// 判别力（`crates/mutations.tsv`）：卡上那一路把在哪判的写成 CPU，这条红。
#[test]
fn an_observer_handing_back_its_disks_is_judged_on_the_gpu_and_its_conclusion_names_the_card() {
    use singlefs_verification::crash_amplification::{
        NodeOfAnObserver, ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    };
    use singlefs_verification::crash_identity::ObserverName;
    use singlefs_verification::verdict_store::JudgingDevice;
    let test_name =
        "an_observer_handing_back_its_disks_is_judged_on_the_gpu_and_its_conclusion_names_the_card";
    if gpu_card_pool_or_report_not_run(test_name).is_none() {
        return;
    }
    let recorded = record_new_pool_file_creation("gpu-observer-images");
    let directory = GpuRunDirectory::new("observer-images");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        directory.configuration.as_os_str(),
    )]);
    let observe = hand_back_the_crash_image;
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("crash_image_handed_back").expect("名字合规矩"),
        observer_code_file: PATH_OF_THIS_TEST_FILE.to_string(),
        // 这个观察者不另读流程输入：交回的只是崩溃镜像本身（另读的输入摘要照一句说明取，不留全零）
        extra_inputs_digest: singlefs_verification::sha256::sha256_digest(
            b"no extra inputs: the observer hands back the crash image itself",
        ),
        nodes: vec![NodeOfAnObserver {
            setting_file: PATH_OF_THIS_TEST_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        },
    }];
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            "gpu-observer-images",
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            recorded.judged_root_index,
            &recorded.versions,
            &[],
            recorded.crash_points.clone(),
            LARGEST_EXPANDED_SEGMENT_WRITES,
            &observers,
        );
    drop(environment);
    if run.stopped_early {
        println!(
            "{}",
            not_run_line(
                test_name,
                "产品路径显存不够：此刻借不到或卡上报了 Out of Memory（上面 test=crash_amplification::run_from_environment 那一行）"
            )
        );
        return;
    }
    let observing = run
        .observing
        .as_ref()
        .expect("挂了观察者、判了的那一趟有观察者那一份");
    assert!(
        observing.images_judged_here + observing.image_judgements_reused > 0,
        "发布那个点上至少一张盘判过：{observing:?}"
    );
    assert_eq!(
        observing.images_judged_on_the_gpu_here, observing.images_judged_here,
        "配置开了 GPU：这一趟判的盘都在卡上判"
    );
    assert_eq!(
        observing
            .red_states_of_each_observer
            .get("crash_image_handed_back")
            .copied()
            .unwrap_or(0),
        0,
        "{:?}",
        observing.red_texts
    );
    let states = observing
        .states_of_each_observer
        .get("crash_image_handed_back")
        .copied()
        .unwrap_or(0);
    assert_eq!(
        observing
            .counts_of_each_observer
            .get("crash_image_handed_back")
            .and_then(|counts| counts.get("pool_checker_clean_on_the_crash_image"))
            .copied()
            .unwrap_or(0),
        states,
        "每个状态交回的盘都判干净"
    );
    let store = directory.open_the_library();
    let observer_conclusions: Vec<_> = store
        .stored_conclusion_keys()
        .expect("列得出库里的结论")
        .into_iter()
        .filter(|(_key, kind, _version)| matches!(kind, ConclusionKind::Observer))
        .collect();
    assert!(!observer_conclusions.is_empty(), "库里有观察者那几块的结论");
    for (key, kind, version) in observer_conclusions {
        let stored = store
            .read_conclusion_block(&key, &kind, &version)
            .expect("读得了")
            .expect("列出来的结论读得回");
        assert!(
            matches!(stored.judged_on, JudgingDevice::Gpu { .. }),
            "观察者那一块的结论记的在哪判的是 GPU：{:?}",
            stored.judged_on
        );
    }
}

/// 观察者交回的盘在同一个位置上写了超过判表一个落点的写数上限那么多次（实测删除流的挂载续做把盘 0 的系统配置槽 0 号写到 70 次，
/// `TooManyWritesOnOneLocation`，判段整条流判不了）：一个交回镜像的观察者在崩溃镜像上把第一块盘的系统配置槽 0 号原样重写
/// [`MAXIMUM_WRITES_PER_LOCATION`] 加 6 次（字节与崩溃镜像上那一槽相同，盘不变），配置开了 GPU。每张盘同一位置上的写并成最后落盘的那一次之后
/// 判表建得出：这一趟判的盘都在卡上判（`images_judged_on_the_gpu_here` 等于 `images_judged_here`），一个状态都不红（没有「GPU 判不了」）。
/// 判别力（`crates/mutations.tsv`）：每张盘同一位置的写不并，单张盘也建不出判表、每个状态判红，这条红。
#[test]
fn an_observer_rewriting_one_location_past_the_judge_table_limit_is_still_judged_on_the_gpu() {
    use singlefs_verification::crash_amplification::{
        NodeOfAnObserver, ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    };
    use singlefs_verification::crash_identity::ObserverName;
    let test_name =
        "an_observer_rewriting_one_location_past_the_judge_table_limit_is_still_judged_on_the_gpu";
    if gpu_card_pool_or_report_not_run(test_name).is_none() {
        return;
    }
    let recorded = record_new_pool_file_creation("gpu-observer-rewriting-one-location");
    let directory = GpuRunDirectory::new("observer-rewriting-one-location");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        directory.configuration.as_os_str(),
    )]);
    let observe = rewrite_the_first_system_configuration_slot_past_the_judge_table_limit;
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("system_configuration_slot_rewritten_past_the_limit")
            .expect("名字合规矩"),
        observer_code_file: PATH_OF_THIS_TEST_FILE.to_string(),
        // 这个观察者不另读流程输入：重写的字节取自崩溃镜像本身（另读的输入摘要照一句说明取，不留全零）
        extra_inputs_digest: singlefs_verification::sha256::sha256_digest(
            b"no extra inputs: the observer rewrites the system configuration slot of the crash image with its own bytes",
        ),
        nodes: vec![NodeOfAnObserver {
            setting_file: PATH_OF_THIS_TEST_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        },
    }];
    let (run, _line) =
        common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
            "gpu-observer-rewriting-one-location",
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            recorded.judged_root_index,
            &recorded.versions,
            &[],
            recorded.crash_points.clone(),
            LARGEST_EXPANDED_SEGMENT_WRITES,
            &observers,
        );
    drop(environment);
    if run.stopped_early {
        println!(
            "{}",
            not_run_line(
                test_name,
                "产品路径显存不够：此刻借不到或卡上报了 Out of Memory（上面 test=crash_amplification::run_from_environment 那一行）"
            )
        );
        return;
    }
    let observing = run
        .observing
        .as_ref()
        .expect("挂了观察者、判了的那一趟有观察者那一份");
    assert!(
        observing.images_judged_here > 0,
        "这一趟至少一张盘现判（建不出判表的不存进库，没有可复用的）：{observing:?}"
    );
    assert_eq!(
        observing.images_judged_on_the_gpu_here, observing.images_judged_here,
        "每张盘同一位置上的写并过之后都建得出判表、都在卡上判"
    );
    assert_eq!(
        observing
            .red_states_of_each_observer
            .get("system_configuration_slot_rewritten_past_the_limit")
            .copied()
            .unwrap_or(0),
        0,
        "{:?}",
        observing.red_texts
    );
}
