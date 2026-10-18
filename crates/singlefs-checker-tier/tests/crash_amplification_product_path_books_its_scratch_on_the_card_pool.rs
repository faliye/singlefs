//! checker 档模块：crash_library、crash_amplification、gpu_unit_checks、crash_judge_dispatch
//! 起过 GPU 用例共用的卡池的进程里，崩溃放量产品路径（`crash_amplification::run_from_environment`）在卡上建的缓冲都在这个进程的显存账上借
//! （`crash_amplification` 文件头第 5 条）：第 ① 段判器的草稿区、第 ① 段由 CPU 判时单元校验和对拍的缓冲、第 ② 段核对的缓冲。
//! 两条用例各在新池新建文件那条流上按环境真跑一趟，看这一趟交回的借的量，再看用的那一刻核账（判器建好与判完、对拍与核对做完时那一笔借据
//! 还在账上，第 10t 批代码轮判决 K4-1、第 10x 批代码轮判决 K3-1）：哪一样改回不借、或借了又在用完之前还掉，对应的那条就红。
//! 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」（经这个进程的卡池判，`gpu_test_start::gpu_card_pool_or_report_not_run`），
//! 有适配器而点名的卡找不到判红；卡上此刻借不到、或卡上 wgpu 报了 Out of Memory，产品路径打它自己的「本次未跑」那一行、这一趟照叫停的样子交回，
//! 这里同样报「本次未跑」。要 `verdict-store,gpu` 两个特性，少一个这份文件是空的。
#![cfg(all(feature = "verdict-store", feature = "gpu"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::path::PathBuf;

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash_amplification::{CrashPointSpan, EnvironmentRun, GpuCardsSource};
use singlefs_checker_tier::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_checker_tier::crash_library::DisposableLibraryDirectory;
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, not_run_line,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs";
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

/// mkfs → 取号 → 两次暖机 → 新池新建文件，按 harness 档 `build_pool` 的做法录；崩溃点按录制流里暖机与发布的分界切。
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

/// 用例的目录（一次性目录：起跑是空的、用完删，长期库一行不写）：一份开了 GPU 的多机配置（单机）与一份库。
/// 配置里的显卡先后照抄本机这一趟的
/// （清场之下要清场之后那一份显卡先后：用例自己写上，`crash_amplification::card_priority_lines_of_the_caller`，抄的是调用方给这个进程的那一份）。
struct GpuRunDirectory {
    configuration: PathBuf,
    library: PathBuf,
    _directory: DisposableLibraryDirectory,
}

impl GpuRunDirectory {
    fn new(tag: &str) -> Self {
        let directory = DisposableLibraryDirectory::new(&format!("booking-{tag}"))
            .expect("流水线单元测试那一类里这条用例的目录");
        let configuration = directory.join("multi-host-with-gpu.env");
        let cards = singlefs_checker_tier::crash_amplification::card_priority_lines_of_the_caller(
            &std::env::current_dir().expect("取得到这个进程的当前目录（多机配置从这里起找）"),
        )
        .expect("本机的多机配置读得了");
        std::fs::write(&configuration, format!("ENABLE_GPU=1\n{cards}"))
            .expect("写得了开 GPU 的配置");
        let library = directory.join("library");
        Self {
            configuration,
            library,
            _directory: directory,
        }
    }
}

/// 在这个进程的卡池上按环境真跑一趟：配置开 GPU、单机（判器、核对两类都走 GPU，崩溃点重放合并第三轮判决 J5；CPU 不在同一段判同一批），
/// 库起跑清空过（每个块都要判）。
/// 本机没卡（卡池报了「本次未跑」）、或产品路径显存不够（这一趟照叫停的样子交回），交 `None`，调用的用例报「本次未跑」当场返回。
fn amplify_on_the_card_pool(test_name: &str) -> Option<EnvironmentRun> {
    gpu_card_pool_or_report_not_run(test_name)?;
    let recorded = record_new_pool_file_creation(test_name);
    let directory = GpuRunDirectory::new(test_name);
    let environment = common_crash_points::CrashAmplificationEnvironment::cleared_then_set(&[
        (
            "SINGLEFS_MULTI_HOST_CONFIG",
            directory.configuration.as_os_str(),
        ),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
            directory.library.as_os_str(),
        ),
    ]);
    let run = common_crash_points::amplify_prepared_flow(
        test_name,
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        recorded.crash_points.clone(),
        LARGEST_EXPANDED_SEGMENT_WRITES,
    );
    drop(environment);
    drop(directory);
    assert_eq!(
        run.gpu_cards_taken_from,
        GpuCardsSource::TheCardPoolOfThisProcess,
        "这个进程起过卡池：产品路径照池里的卡用"
    );
    if run.stopped_early {
        println!(
            "{}",
            not_run_line(
                test_name,
                "产品路径显存不够：此刻借不到或卡上报了 Out of Memory（上面 test=crash_amplification::run_from_environment 那一行）"
            )
        );
        return None;
    }
    Some(run)
}

/// 第 ① 段在卡上判（配置开了 GPU）：每张用到的卡上，判器的草稿区都在显存账上借了，判器用的就是借来的那张
/// （额度是借的量加驱动那一份，开出来的草稿区不超过借的），判器建好那一刻与这张卡判完那一刻那一笔借据都还在账上。哪天这一支改回不借，
/// 借的量是 `None`，借了又在判之前或判完之前还掉，核账是 `Some(false)`，判完那一刻不核是 `None`，这条都红。
#[test]
fn the_product_path_judges_on_scratch_it_booked_on_the_card_pool() {
    let Some(run) =
        amplify_on_the_card_pool("the_product_path_judges_on_scratch_it_booked_on_the_card_pool")
    else {
        return;
    };
    let gpu_judge = run
        .gpu_judge
        .as_ref()
        .expect("配置开了 GPU、有卡：第 ① 段在卡上判");
    assert!(
        !gpu_judge.cards.is_empty(),
        "新建的空库里每个块都要判：至少一张卡判了"
    );
    for card in &gpu_judge.cards {
        let booked = card.scratch_booked_mebibytes.unwrap_or_else(|| {
            panic!(
                "起过卡池的进程，序号 {} 的卡上判器的草稿区要在显存账上借：没借",
                card.index
            )
        });
        assert_eq!(
            card.memory_budget_mebibytes,
            booked + MEBIBYTES_LEFT_FOR_THE_DRIVER,
            "序号 {} 的卡：判器用的是借来的那张（额度是借的量加驱动那一份）",
            card.index
        );
        assert!(
            card.scratch_mebibytes <= booked,
            "序号 {} 的卡：判器开的草稿区 {} MiB 不超过借的 {booked} MiB",
            card.index,
            card.scratch_mebibytes
        );
        assert_eq!(
            card.scratch_lease_on_the_books_when_the_judge_was_built,
            Some(true),
            "序号 {} 的卡：判器建好那一刻核账，草稿区那一笔借据要还在账上（借了又在判之前还掉要红）",
            card.index
        );
        assert_eq!(
            card.scratch_lease_on_the_books_when_the_judging_finished,
            Some(true),
            "序号 {} 的卡：判完那一刻再核一次账，草稿区那一笔借据要还在账上（建好之后、判完之前还掉要红）",
            card.index
        );
    }
}

/// 第 ② 段在卡上核（配置开了 GPU，判器与核对都走 GPU）：核对在卡上建的缓冲在显存账上借了（借的量不为 0）、核完那一刻那一笔借据还在账上、
/// 第 ② 段在卡上核了。改回不借，借的量是 `None`；借了又在用完之前还掉，核账不是 `Some(true)`：这条都红。
#[test]
fn the_product_path_books_the_facts_verifier_buffers_on_the_card_pool() {
    let Some(run) = amplify_on_the_card_pool(
        "the_product_path_books_the_facts_verifier_buffers_on_the_card_pool",
    ) else {
        return;
    };
    assert!(
        run.facts_verifier_buffers_booked_mebibytes
            .is_some_and(|booked| booked > 0),
        "第 ② 段核对的缓冲要在显存账上借：{:?}",
        run.facts_verifier_buffers_booked_mebibytes
    );
    assert_eq!(
        run.facts_verifier_buffers_lease_on_the_books_after_verifying,
        Some(true),
        "第 ② 段核完那一刻核账：核对的缓冲那一笔借据要还在账上"
    );
    assert_eq!(
        run.verification.map(|outcome| outcome.verifier),
        Some("gpu"),
        "配置开了 GPU：第 ② 段在卡上核"
    );
}
