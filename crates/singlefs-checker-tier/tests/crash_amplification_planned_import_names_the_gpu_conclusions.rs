//! checker 档模块：crash、crash_point_replay_progress、crash_given_states、crash_identity、crash_library、crash_amplification、verdict_store、gpu_unit_checks、crash_verify_gpu、crash_judge_tables
//! 配置开了 GPU 时两台分片之后的按计划导入（第 62 批判决 Q4）：流水线那一路点名的键里有 GPU 判器那一种结论与 GPU 核对代码那一把摘要的核对行，
//! 给定状态表那一路点名的键里有 GPU 判器那一种结论。没配 GPU 的那几条（`crash_amplification_pipeline_records_checks_and_reuses.rs`）
//! 库里没有这两样，漏点了照绿；这里两条在卡上真判，漏点哪一样对应的那条就红。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
//! 有适配器而点名的卡找不到判红；产品路径此刻借不到显存（流水线那一趟照叫停的样子交回、给定状态表的判器报运行条件不满足），同样报「本次未跑」。
//! 经环境跑的用例自己清、自己还原环境（K5，公共模块的环境守卫）。要 `verdict-store,gpu` 两个特性，少一个这份文件是空的。
#![cfg(all(feature = "verdict-store", feature = "gpu"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash_amplification::{
    card_priority_lines_of_the_caller, CrashPointSpan, EnvironmentRun, GivenStatesEngine,
    GivenStatesItem, GivenStatesRun, GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStatesConclusionsOfATable, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_identity::ConclusionKind;
use singlefs_checker_tier::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start;
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_checker_tier::crash_verify_gpu::gpu_verifier_digest;
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, not_run_line,
};
use singlefs_checker_tier::verdict_store::VerdictStore;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_amplification_planned_import_names_the_gpu_conclusions.rs";
/// 只展开不超过 6 次写的段：新池新建文件那条流展开的十段共几十个状态，秒级。
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;
/// 给定状态表那一条：一项 4 张表，每张两个状态。
const GIVEN_STATES_TABLES: usize = 4;
const GIVEN_STATES_ITEM: &str = "given_states_gpu_import_probe";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../Cargo.lock")
        .canonicalize()
        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
        .parent()
        .expect("Cargo.lock 所在的目录就是仓根")
        .to_path_buf()
}

/// 录好的一条流：崩溃枚举要的五样加两个崩溃点。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    crash_points: Vec<CrashPointSpan>,
}

/// mkfs → 取号 → 两次暖机 → 新池新建文件，按 harness 档 `build_pool` 的做法录；崩溃点按录制流里暖机与发布的分界切
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
            code_file_in_repository: THIS_FILE.to_string(),
            writes: 0..first_publish_write,
            ignored: false,
        },
        CrashPointSpan {
            name: "publish_first_file".to_string(),
            code_file_in_repository: THIS_FILE.to_string(),
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

/// 用例的目录（家目录下流水线单元测试那一类的长期库里以用例标签命名的一份，起跑先清空、跑完留着）：一份开了 GPU 的多机配置
/// （显卡先后照抄调用方给这个进程的那一份，`card_priority_lines_of_the_caller`；清场之下要清场之后那一份）与两份库（第二台的、本机的）。
struct GpuImportDirectory {
    configuration: PathBuf,
    other_host_library: PathBuf,
    this_host_library: PathBuf,
}

impl GpuImportDirectory {
    fn new(tag: &str) -> Self {
        let directory = amplification_unit_tests_library_directory_emptied_at_the_start(&format!(
            "gpu-import-{tag}"
        ))
        .expect("流水线单元测试那一类里这条用例的目录");
        let configuration = directory.join("multi-host-with-gpu.env");
        let cards =
            card_priority_lines_of_the_caller(&repository_root()).expect("本机的多机配置读得了");
        std::fs::write(&configuration, format!("ENABLE_GPU=1\n{cards}"))
            .expect("写得了开 GPU 的配置");
        let other_host_library = directory.join("other-host-library");
        let this_host_library = directory.join("this-host-library");
        Self {
            configuration,
            other_host_library,
            this_host_library,
        }
    }
}

/// 一条录好的流经公共模块按环境跑一趟（展开上限默认 6）。环境由调用方的守卫定。
fn amplify(recorded: &RecordedFlow, flow_name: &str) -> EnvironmentRun {
    common_crash_points::amplify_prepared_flow(
        flow_name,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        recorded.crash_points.clone(),
        LARGEST_EXPANDED_SEGMENT_WRITES,
    )
}

fn open_the_library(directory: &Path) -> VerdictStore {
    VerdictStore::open_existing(directory)
        .unwrap_or_else(|error| panic!("开得了库 {}：{error:?}", directory.display()))
        .0
}

/// 流水线那一路（`crash_amplification` 导入那一段）：配置开 GPU，新池新建文件那条流 3 块，平滑加权轮转把第 1 块（发布那个点）分给 1/2。
/// 第二台（1/2）在自己的一份库里判那一块（只有 GPU 判器那一份结论）、在卡上核（GPU 核对代码的核对行）；本机（0/2）判第 0、2 块，
/// 再带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<第二台的库>` 跑一趟。导入之后 GPU 判器那一种一块没判的都没有，导入的结论块数等于第二台库里的
/// 结论块数（1）；导入那一趟第 ③ 段在卡上核、拿 GPU 核对代码的核对行比，没有一块缺核对行；第二台那一块的 GPU 核对行导进来、读回相同。
/// 判别力（`crates/mutations.tsv`）：排键时漏掉 GPU 判器那一种，第二台那一块的 GPU 结论导不进来、对账剩 1 块；
/// 排键时漏掉 GPU 核对代码那一把摘要，导入那一趟比对缺 1 块核对行。
#[test]
fn importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row() {
    let test_name =
        "importing_the_other_share_on_the_gpu_brings_its_gpu_judge_conclusion_and_its_gpu_verifier_row";
    if gpu_card_pool_or_report_not_run(test_name).is_none() {
        return;
    }
    let recorded = record_new_pool_file_creation("gpu-pipeline-import");
    let directory = GpuImportDirectory::new("pipeline");
    let mut environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
        (
            "SINGLEFS_MULTI_HOST_CONFIG",
            directory.configuration.as_os_str(),
        ),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
            directory.other_host_library.as_os_str(),
        ),
        ("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2")),
    ]);
    let other_share_run = amplify(&recorded, "gpu-pipeline-import");
    environment.set(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        directory.this_host_library.as_os_str(),
    );
    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
    let this_share_run = amplify(&recorded, "gpu-pipeline-import");
    environment.set(
        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        directory.other_host_library.as_os_str(),
    );
    let importing_run = amplify(&recorded, "gpu-pipeline-import");
    drop(environment);
    if [&other_share_run, &this_share_run, &importing_run]
        .iter()
        .any(|run| run.stopped_early)
    {
        println!(
            "{}",
            not_run_line(
                test_name,
                "产品路径显存不够：此刻借不到或卡上报了 Out of Memory（上面 test=crash_amplification::run_from_environment 那一行）"
            )
        );
        return;
    }
    assert!(
        other_share_run.gpu_judge.is_some(),
        "前提：配置开了 GPU，第二台那一份在卡上判"
    );
    assert_eq!(
        other_share_run
            .verification
            .as_ref()
            .map(|outcome| outcome.verifier),
        Some("gpu"),
        "前提：配置开了 GPU，第二台那一份在卡上核"
    );
    assert_eq!(
        (
            other_share_run.checking.blocks_checked,
            this_share_run.checking.blocks_checked
        ),
        (1, 2),
        "前提：1/2 判第 1 块，0/2 判第 0、2 块"
    );
    assert_eq!(
        this_share_run.unjudged_blocks, 1,
        "前提：导入之前 0/2 的库里第二台那一块没判"
    );
    let other_share_store = open_the_library(&directory.other_host_library);
    let other_share_conclusions = other_share_store
        .stored_conclusion_keys()
        .expect("列得出第二台那一份的结论");
    assert_eq!(
        other_share_conclusions.len(),
        1,
        "前提：第二台那一份只有发布那一块的 GPU 判器结论（配了 GPU 只由 GPU 判器判，CPU 不在同一段判同一批）：{other_share_conclusions:?}"
    );
    assert!(
        other_share_conclusions
            .iter()
            .any(|(_, kind, _)| *kind == ConclusionKind::GpuJudge),
        "前提：第二台那一份库里有 GPU 判器那一种结论：{other_share_conclusions:?}"
    );
    assert_eq!(
        importing_run.unjudged_blocks, 0,
        "导入之后 GPU 判器那一种一块没判的都没有：第二台那一块的 GPU 判器结论导进来了"
    );
    assert_eq!(
        importing_run.imported_blocks,
        u64::try_from(other_share_conclusions.len()).expect("块数装得进 u64"),
        "导入的结论块数等于第二台那一份的结论块数（GPU 判器那一块）"
    );
    let comparison_of_the_import = importing_run
        .comparison
        .as_ref()
        .expect("配置开了 GPU：导入那一趟在卡上核了、比了");
    assert_eq!(
        (
            comparison_of_the_import.blocks_without_verifier,
            comparison_of_the_import.blocks_without_judge,
            comparison_of_the_import.disagreements
        ),
        (0, 0, 0),
        "导入那一趟第 ③ 段拿 GPU 核对代码的核对行比：第二台那一块的 GPU 核对行导进来了，没有一块缺核对行"
    );
    let this_share_store = open_the_library(&directory.this_host_library);
    let gpu_verifier = gpu_verifier_digest();
    for (block_key, _, _) in &other_share_conclusions {
        let verifier_row_of_the_other_share = other_share_store
            .read_verifier_block(block_key, &gpu_verifier)
            .expect("读得了第二台的核对行");
        assert!(
            verifier_row_of_the_other_share.is_some(),
            "前提：第二台在卡上核过块 {}",
            block_key.stream_or_node_name.as_str()
        );
        assert_eq!(
            this_share_store
                .read_verifier_block(block_key, &gpu_verifier)
                .expect("读得了导入之后的核对行"),
            verifier_row_of_the_other_share,
            "块 {} 的 GPU 核对行：导入之后的本机库与第二台的读回不同",
            block_key.stream_or_node_name.as_str()
        );
    }
    drop((other_share_store, this_share_store));
    drop(directory);
}

/// 新池新建文件那条流上的几张给定状态表：第 k 张两个状态，持久了前 k·写数/张数 次写的与全部持久的（一条录制流的前缀都是合法的崩溃状态），
/// 表名带 k（同 `crash_amplification_pipeline_records_checks_and_reuses.rs` 那一份）。
fn given_states_tables_of_the_flow(
    recorded: &RecordedFlow,
    table_count: usize,
) -> Vec<GivenStatesOfATable<'_>> {
    let write_count = recorded.writes.len();
    (0..table_count)
        .map(|table_index| {
            let persisted_prefix_writes = table_index * write_count / table_count;
            GivenStatesOfATable {
                table_name: format!("gpu-import-probe-table-{table_index}"),
                base: &recorded.base,
                writes: &recorded.writes,
                segments: &recorded.segments,
                seams: &[],
                persisted_sets: vec![
                    (0..write_count)
                        .map(|write_index| write_index < persisted_prefix_writes)
                        .collect(),
                    vec![true; write_count],
                ],
            }
        })
        .collect()
}

/// 照此刻的环境起一个给定状态表的引擎，判完 `tables` 每一张、收尾，交回每张的结论与这一趟。配了 GPU 而判器此刻借不到显存
/// （判器报 [`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 那一行）时收尾、报「本次未跑」，交 `None`。
fn judge_every_given_states_table_or_report_not_run(
    test_name: &str,
    tables: &[GivenStatesOfATable<'_>],
) -> Option<(Vec<GivenStatesConclusionsOfATable>, GivenStatesRun)> {
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: GIVEN_STATES_ITEM.to_string(),
            setting_file: THIS_FILE.to_string(),
            seeds_and_scale: "new_pool_file_creation gpu_import_probe".to_string(),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        &repository_root(),
    )
    .expect("配了 GPU 的给定状态表引擎起得来");
    let mut conclusions = Vec::with_capacity(tables.len());
    for table in tables {
        match engine.judge_the_given_states(table) {
            Ok(conclusions_of_the_table) => conclusions.push(conclusions_of_the_table),
            Err(reason) if reason.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX) => {
                drop(engine.finish());
                println!("{}", not_run_line(test_name, &reason));
                return None;
            }
            Err(reason) => panic!(
                "项 {GIVEN_STATES_ITEM} 判得了表 {}：{reason}",
                table.table_name
            ),
        }
    }
    Some((conclusions, engine.finish()))
}

/// 给定状态表那一路（`GivenStatesEngine::import_the_table_from_the_other_hosts`）：配置开 GPU，权重 0,1 让每张表都归 1/2 那一台；
/// 那一台在卡上判这一项的 4 张表（每张只存 GPU 判器那一块）、写一份库；本机这一份（0/2，一张表都不归它）带
/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<那一台的库>` 判同一批：每张都从那份库点名导入、照库里的复用（复用要 GPU 判器那一种在），
/// 一张都不现判。账：导入的块数等于那一台库里的结论块数（4 张 × GPU 判器一种 = 4）、复用 4 张、现判 0 张、留给别的机器 0 张。
/// 判别力（`crates/mutations.tsv`）：点名时漏掉 GPU 判器那一种，导入之后库里还是没有这张表 GPU 判器的结论，引擎判第一张表就报
/// 「导入之后库里还是没有」。
#[test]
fn given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions(
) {
    let test_name =
        "given_states_tables_judged_on_the_gpu_by_another_host_are_imported_with_their_gpu_judge_conclusions";
    if gpu_card_pool_or_report_not_run(test_name).is_none() {
        return;
    }
    let recorded = record_new_pool_file_creation("gpu-given-states-import");
    let tables = given_states_tables_of_the_flow(&recorded, GIVEN_STATES_TABLES);
    let directory = GpuImportDirectory::new("given-states");
    // 给定状态引擎不经公共模块的 run_from_environment 那一路（那一路配置里抄不到显卡先后时自己补）：调用方的显卡先后经环境变量交，
    // 配置里抄到的是空串时（副本里没有仓根 multi-host.env）清场之下照样起得来（common_gpu_cards_of_the_caller 文件头）
    let cards =
        common_crash_points::common_gpu_cards_of_the_caller::card_priority_value_of_the_caller(
            &repository_root(),
        );
    let mut environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
        (
            "SINGLEFS_MULTI_HOST_CONFIG",
            directory.configuration.as_os_str(),
        ),
        ("GPU_CARDS_BY_PRIORITY", OsStr::new(&cards)),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
            OsStr::new("0,1"),
        ),
        ("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2")),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
            directory.other_host_library.as_os_str(),
        ),
    ]);
    let Some((_, other_host_run)) =
        judge_every_given_states_table_or_report_not_run(test_name, &tables)
    else {
        return;
    };
    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
    environment.set(
        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
        directory.this_host_library.as_os_str(),
    );
    environment.set(
        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
        directory.other_host_library.as_os_str(),
    );
    let Some((_, importing_run)) =
        judge_every_given_states_table_or_report_not_run(test_name, &tables)
    else {
        return;
    };
    drop(environment);
    assert_eq!(
        (
            other_host_run.tally.tables_judged_here,
            other_host_run.tally.states_judged_by_the_gpu_here
        ),
        (4, 8),
        "前提：权重 0,1 下每张表都归 1/2，那一台在卡上判了 4 张表、8 个状态"
    );
    assert!(
        other_host_run.red_items().is_empty(),
        "前提：那一台判得干净（GPU 都判得了）：{:?}",
        other_host_run.red_items()
    );
    let other_host_store = open_the_library(&directory.other_host_library);
    let other_host_conclusions = other_host_store
        .stored_conclusion_keys()
        .expect("列得出那一台的结论");
    assert_eq!(
        other_host_conclusions
            .iter()
            .filter(|(_, kind, _)| *kind == ConclusionKind::GpuJudge)
            .count(),
        GIVEN_STATES_TABLES,
        "前提：那一台的库里每张表一块 GPU 判器结论"
    );
    assert_eq!(
        other_host_conclusions.len(),
        GIVEN_STATES_TABLES,
        "前提：那一台的库里每张表只有 GPU 判器那一块（配了 GPU 只由 GPU 判器判）"
    );
    drop(other_host_store);
    assert_eq!(
        (
            importing_run.tally.imported_blocks,
            importing_run.tally.tables_reused,
            importing_run.tally.tables_judged_here,
            importing_run.tally.tables_left_to_the_other_hosts
        ),
        (4, 4, 0, 0),
        "按名导入：导入 4 块（每张表 GPU 判器那一块）、复用 4 张、现判 0 张、留给别的机器 0 张"
    );
    assert!(
        importing_run.red_items().is_empty(),
        "导入那一趟一张都不现判、不红：{:?}",
        importing_run.red_items()
    );
    drop(directory);
}

/// 观察者走两段、池级 checker 上 GPU（用户 2026-10-13 定「观察者改走两段、池级 checker 上 GPU、按恢复结果去重」）：配置开了 GPU，
/// 一个交回镜像的观察者（交回崩溃镜像本身、一个写都不发，判每一条不变量）挂在发布那个点上：录入段在 CPU 上看、按恢复镜像去重，
/// 判段在卡上判观察者交回的盘——这一趟判的盘都在卡上判（`images_judged_on_the_gpu_here` 等于 `images_judged_here`），库里观察者那几块
/// 结论记的在哪判的是 GPU（卡型与驱动），每个状态判干净（新池新建文件这条流第 ① 段本来就一个红状态都没有）。
/// 判别力（`crates/mutations.tsv`）：卡上那一路把在哪判的写成 CPU，这条红。
#[test]
fn an_observer_handing_back_its_disks_is_judged_on_the_gpu_and_its_conclusion_names_the_card() {
    use singlefs_checker_tier::crash_amplification::{
        ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, NodeOfAnObserver,
        ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    };
    use singlefs_checker_tier::crash_identity::ObserverName;
    use singlefs_checker_tier::verdict_store::JudgingDevice;
    let test_name =
        "an_observer_handing_back_its_disks_is_judged_on_the_gpu_and_its_conclusion_names_the_card";
    if gpu_card_pool_or_report_not_run(test_name).is_none() {
        return;
    }
    let recorded = record_new_pool_file_creation("gpu-observer-images");
    let directory = GpuImportDirectory::new("observer-images");
    let environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
        (
            "SINGLEFS_MULTI_HOST_CONFIG",
            directory.configuration.as_os_str(),
        ),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
            directory.this_host_library.as_os_str(),
        ),
    ]);
    let observe = |_ordinal: u64,
                   image: &singlefs_harness::memory_pool::CrashImage<'_>,
                   _report: &singlefs_core::recovery::RecoveryReport,
                   _counts: &mut singlefs_checker_tier::crash::CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        let mut disks_after_the_observer = image.base.clone();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            if *is_persisted {
                disks_after_the_observer.apply_writes(std::slice::from_ref(write));
            }
        }
        ImageAfterTheObserver {
            writes_on_the_crash_image: Vec::new(),
            disks_after_the_observer,
            invariants_judged:
                InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
            counter_when_the_pool_checker_is_clean: "pool_checker_clean_on_the_crash_image"
                .to_string(),
        }
    };
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("crash_image_handed_back").expect("名字合规矩"),
        observer_code_file: THIS_FILE.to_string(),
        // 这个观察者不另读流程输入：交回的只是崩溃镜像本身（另读的输入摘要照一句说明取，不留全零）
        extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
            b"no extra inputs: the observer hands back the crash image itself",
        ),
        nodes: vec![NodeOfAnObserver {
            setting_file: THIS_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveredImageBytes,
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
    let store = open_the_library(&directory.this_host_library);
    let observer_conclusions: Vec<_> = store
        .stored_conclusion_keys()
        .expect("列得出库里的结论")
        .into_iter()
        .filter(|(_key, kind, _version)| matches!(kind, ConclusionKind::Observer { .. }))
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
    use singlefs_checker_tier::crash_amplification::{
        ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, NodeOfAnObserver,
        ObserverCode, ObserverDeduplication, ObserverOnTheNodes,
    };
    use singlefs_checker_tier::crash_identity::ObserverName;
    use singlefs_checker_tier::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION;
    let test_name =
        "an_observer_rewriting_one_location_past_the_judge_table_limit_is_still_judged_on_the_gpu";
    if gpu_card_pool_or_report_not_run(test_name).is_none() {
        return;
    }
    let recorded = record_new_pool_file_creation("gpu-observer-rewriting-one-location");
    let directory = GpuImportDirectory::new("observer-rewriting-one-location");
    let environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
        (
            "SINGLEFS_MULTI_HOST_CONFIG",
            directory.configuration.as_os_str(),
        ),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
            directory.this_host_library.as_os_str(),
        ),
    ]);
    let rewrites_of_the_slot = MAXIMUM_WRITES_PER_LOCATION + 6;
    let observe = |_ordinal: u64,
                   image: &singlefs_harness::memory_pool::CrashImage<'_>,
                   _report: &singlefs_core::recovery::RecoveryReport,
                   _counts: &mut singlefs_checker_tier::crash::CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
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
        let writes_on_the_crash_image: Vec<singlefs_harness::memory_pool::RetainedWrite> = (0
            ..rewrites_of_the_slot)
            .map(|_rewrite| singlefs_harness::memory_pool::RetainedWrite {
                device: first_device,
                kind: singlefs_harness::segments::StepKind::SystemConfigurationSlot,
                is_force_unit_access: true,
                offset: singlefs_core::address::DeviceOffsetInBytes(0),
                contents: singlefs_harness::memory_pool::WrittenContents::Bytes(slot_bytes.clone()),
            })
            .collect();
        disks_after_the_observer.apply_writes(&writes_on_the_crash_image);
        ImageAfterTheObserver {
            writes_on_the_crash_image,
            disks_after_the_observer,
            invariants_judged:
                InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
            counter_when_the_pool_checker_is_clean: "pool_checker_clean_after_the_rewrites"
                .to_string(),
        }
    };
    let observers = [ObserverOnTheNodes {
        observer_name: ObserverName::new("system_configuration_slot_rewritten_past_the_limit")
            .expect("名字合规矩"),
        observer_code_file: THIS_FILE.to_string(),
        // 这个观察者不另读流程输入：重写的字节取自崩溃镜像本身（另读的输入摘要照一句说明取，不留全零）
        extra_inputs_digest: singlefs_harness::sha256::sha256_digest(
            b"no extra inputs: the observer rewrites the system configuration slot of the crash image with its own bytes",
        ),
        nodes: vec![NodeOfAnObserver {
            setting_file: THIS_FILE.to_string(),
            entry_name: "publish_first_file".to_string(),
        }],
        observe: ObserverCode::HandsBackAnImage {
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveredImageBytes,
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
