//! 层 0 全量的断点续跑（层 0 规模三轮判决 R1–R5、U1–U3，实六；用户 2026-09-26 定「跑前实现」）：在第一个事务那条小流上
//! （mkfs → 取号 → 暖机 → A，按甲二展开，几十个状态）拿真的进度文件核续跑的每一条。
//! 「跑到一半被杀」这样造：跑完留着进度文件（只供测试的那一档），再把文件截到前几片、末尾留半行。
//! 名字里不带 layer0：它只跑甲二那几十个状态，平时跑得起。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::num::{NonZeroU64, NonZeroUsize};
use std::path::{Path, PathBuf};

use common::{build_pool, file_content, geometry};
use singlefs_checker_tier::crash::{
    enumerate_layer0_in_state_slices, quick_tier_expansion, Layer0ObserverCounts,
    Layer0Parallelism, Layer0SliceLength, Layer0Tally, Layer0WorkerThreadsSource,
};
use singlefs_checker_tier::layer0_progress::{
    Layer0ProgressFileAfterCompletion, Layer0ProgressFileNamePart, Layer0ProgressFileSettings,
    Layer0Resume, Layer0ResumeStart,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

struct FirstStream {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn first_stream(tag: &str) -> FirstStream {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽写");
    FirstStream {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// 每个用例一个自己的进度目录（先清空）。
fn fresh_progress_directory(test_name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "singlefs-crash-enumeration-resume-{}-{test_name}",
        std::process::id()
    ));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("清空上一次留下的进度目录");
    }
    directory
}

fn progress_files_in(directory: &Path) -> Vec<PathBuf> {
    if !directory.exists() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .expect("读进度目录")
        .map(|entry| entry.expect("目录项").path())
        .collect();
    files.sort();
    files
}

fn settings(
    directory: &Path,
    start: Layer0ResumeStart,
    after_completion: Layer0ProgressFileAfterCompletion,
) -> Layer0Resume {
    Layer0Resume::KeepProgressFile(Layer0ProgressFileSettings {
        directory: directory.to_path_buf(),
        input_fingerprint: Layer0ProgressFileNamePart::new("fingerprint0").expect("合法"),
        stream_name: Layer0ProgressFileNamePart::new("first_transaction_stream").expect("合法"),
        start,
        after_completion,
    })
}

fn one_state_per_slice_on(worker_threads: usize) -> Layer0Parallelism {
    Layer0Parallelism {
        worker_threads: NonZeroUsize::new(worker_threads).expect("不是 0"),
        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
        slice_length: Layer0SliceLength::StatesPerSlice(NonZeroU64::MIN),
    }
}

/// 跑一趟甲二：观察者把「读出文件 / 没有文件」记进观察者计数（续跑要接上的累计），另数这一趟它自己被调了几次（只数这一趟跑的片）。
fn enumerate_the_quick_tier_counting_observer_calls(
    stream: &FirstStream,
    versions: &[PublishedVersion],
    parallelism: Layer0Parallelism,
    resume: &Layer0Resume,
) -> (Layer0Tally, u64) {
    let mut observer_calls_in_this_run = 0u64;
    let tally = enumerate_layer0_in_state_slices(
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.root_index,
        versions,
        &quick_tier_expansion,
        parallelism,
        Some(&mut |_image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut Layer0ObserverCounts| {
            observer_calls_in_this_run += 1;
            match report.outcome {
                RecoveryOutcome::FileRead { .. } => counts.add("file_read_states", 1),
                RecoveryOutcome::NoFile { .. } => counts.add("no_file_states", 1),
                RecoveryOutcome::Failed { .. } => counts.add("failed_states", 1),
            }
        }),
        resume,
    );
    (tally, observer_calls_in_this_run)
}

/// 把进度文件截成「文件头 + 前 `kept_slice_lines` 行片行 + 下一行的前一半」：跑到一半被杀的样子。
fn cut_short(path: &Path, kept_slice_lines: usize) {
    let content = std::fs::read_to_string(path).expect("读进度文件");
    let lines: Vec<&str> = content.lines().collect();
    let mut cut = lines[..=kept_slice_lines].join("\n");
    cut.push('\n');
    let next_line = lines[kept_slice_lines + 1];
    cut.push_str(&next_line[..next_line.len() / 2]);
    std::fs::write(path, cut).expect("写截断的进度文件");
}

/// 跑到一半被杀之后续跑：读回的片不再跑、末尾半行那一片重跑，续跑的计数（连观察者计数与它看过的状态数）与一口气跑完的逐项相同；
/// 续跑这一趟观察者只被调了没读回的那几片的状态数；跑完进度文件删掉。
#[test]
fn a_run_cut_short_after_some_slices_resumes_to_the_same_tally_as_one_uninterrupted_run() {
    let stream = first_stream("resume-cut-short");
    let directory = fresh_progress_directory("cut-short");
    let (uninterrupted, uninterrupted_calls) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &Layer0Resume::NoProgressFile,
    );
    assert_eq!(uninterrupted_calls, uninterrupted.states);
    assert_eq!(uninterrupted.observed_states, uninterrupted.states);
    assert!(
        uninterrupted.states > 20,
        "甲二在第一条流上有几十个状态：{}",
        uninterrupted.states
    );

    let (kept_run, _) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
        ),
    );
    assert_eq!(kept_run, uninterrupted, "留进度文件的一趟与不留的逐项相同");
    let [progress_file] = progress_files_in(&directory)
        .try_into()
        .expect("恰好一个进度文件");
    let slice_lines = std::fs::read_to_string(&progress_file)
        .expect("读进度文件")
        .lines()
        .count()
        - 1;
    assert_eq!(
        u64::try_from(slice_lines).expect("行数"),
        uninterrupted.states,
        "每片一个状态：文件头之后每片一行"
    );

    let kept_slice_lines = 10;
    cut_short(&progress_file, kept_slice_lines);
    let (resumed, resumed_calls) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::Deleted,
        ),
    );
    assert_eq!(
        resumed, uninterrupted,
        "续跑之后的计数与一口气跑完的逐项相同"
    );
    assert_eq!(
        resumed_calls,
        uninterrupted.states - u64::try_from(kept_slice_lines).expect("行数"),
        "读回的 10 片不再跑、不再给观察者看；末尾半行那一片重跑"
    );
    assert_eq!(
        resumed.observer_counts, uninterrupted.observer_counts,
        "观察者的累计连读回那几片记的接上了"
    );
    assert!(
        progress_files_in(&directory).is_empty(),
        "跑完删掉进度文件：{:?}",
        progress_files_in(&directory)
    );
    std::fs::remove_dir_all(&directory).ok();
}

/// 续跑的切法不随线程数变：1 个线程跑完留下的进度文件，换 8 个线程续跑整份认得出（按线程数定片长的那一档在续跑时改按状态数定）。
#[test]
fn a_resumable_run_slices_the_same_way_on_any_number_of_worker_threads() {
    let stream = first_stream("resume-threads");
    let directory = fresh_progress_directory("threads");
    let scaled_to = |worker_threads: usize| Layer0Parallelism {
        worker_threads: NonZeroUsize::new(worker_threads).expect("不是 0"),
        worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
        slice_length: Layer0SliceLength::ScaledToWorkerThreads,
    };
    let (on_one_thread, _) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        scaled_to(1),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
        ),
    );
    let (on_eight_threads, calls_on_eight_threads) =
        enumerate_the_quick_tier_counting_observer_calls(
            &stream,
            &stream.versions,
            scaled_to(8),
            &settings(
                &directory,
                Layer0ResumeStart::ResumeFromTheProgressFile,
                Layer0ProgressFileAfterCompletion::Deleted,
            ),
        );
    assert_eq!(on_eight_threads, on_one_thread);
    assert_eq!(
        calls_on_eight_threads, 0,
        "每一片都从进度文件读回：换了线程数片方案照旧对得上"
    );
    std::fs::remove_dir_all(&directory).ok();
}

/// 同一条流、同一组状态只换了版本表（层 0 规模第三轮 U1）：是另一格进度文件，不拿前一格的片、也不删前一格。
#[test]
fn a_progress_file_of_another_versions_table_is_neither_used_nor_removed() {
    let stream = first_stream("resume-versions");
    let directory = fresh_progress_directory("versions");
    enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
        ),
    );
    let [first_file] = progress_files_in(&directory)
        .try_into()
        .expect("一个进度文件");
    let first_file_content = std::fs::read(&first_file).expect("读");
    let mut other_versions = stream.versions.clone();
    other_versions[0].content = b"another versions table".to_vec();
    let (with_other_versions, calls_with_other_versions) =
        enumerate_the_quick_tier_counting_observer_calls(
            &stream,
            &other_versions,
            one_state_per_slice_on(4),
            &settings(
                &directory,
                Layer0ResumeStart::ResumeFromTheProgressFile,
                Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
            ),
        );
    assert_eq!(
        calls_with_other_versions, with_other_versions.states,
        "换了版本表：一片都不从前一格读回"
    );
    assert_eq!(progress_files_in(&directory).len(), 2, "两格进度文件并存");
    assert_eq!(
        std::fs::read(&first_file).expect("前一格还在"),
        first_file_content,
        "前一格一个字节都没动"
    );
    std::fs::remove_dir_all(&directory).ok();
}

/// 进度文件里一整行少一个字段（校验和照新正文重算过，对得上）：整份作废、每一片重跑（U2），计数照旧对。
#[test]
fn a_progress_file_whose_whole_line_lacks_a_field_is_discarded_and_every_slice_reruns() {
    let stream = first_stream("resume-missing-field");
    let directory = fresh_progress_directory("missing-field");
    let (kept_run, _) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
        ),
    );
    let [progress_file] = progress_files_in(&directory)
        .try_into()
        .expect("一个进度文件");
    let content = std::fs::read_to_string(&progress_file).expect("读");
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    let (body, _checksum) = lines[3].rsplit_once(" checksum=").expect("有校验和");
    let without_failed_states: Vec<&str> = body
        .split(' ')
        .filter(|field| !field.starts_with("failed_states="))
        .collect();
    let body_without_the_field = without_failed_states.join(" ");
    lines[3] = format!(
        "{body_without_the_field} checksum={:08x}",
        crc32_castagnoli(body_without_the_field.as_bytes())
    );
    std::fs::write(&progress_file, format!("{}\n", lines.join("\n"))).expect("写");
    let (rerun, rerun_calls) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::Deleted,
        ),
    );
    assert_eq!(rerun_calls, kept_run.states, "整份作废：每一片都重跑");
    assert_eq!(rerun, kept_run);
    std::fs::remove_dir_all(&directory).ok();
}

/// 强制从头跑的开关（R5）：进度文件完好，照样每一片重跑。
#[test]
fn starting_over_reruns_every_slice_even_with_a_valid_progress_file() {
    let stream = first_stream("resume-start-over");
    let directory = fresh_progress_directory("start-over");
    let (kept_run, _) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::ResumeFromTheProgressFile,
            Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
        ),
    );
    let (started_over, started_over_calls) = enumerate_the_quick_tier_counting_observer_calls(
        &stream,
        &stream.versions,
        one_state_per_slice_on(4),
        &settings(
            &directory,
            Layer0ResumeStart::StartOverDiscardingTheProgressFile,
            Layer0ProgressFileAfterCompletion::Deleted,
        ),
    );
    assert_eq!(started_over_calls, kept_run.states);
    assert_eq!(started_over, kept_run);
    std::fs::remove_dir_all(&directory).ok();
}

/// 判红的那一趟删进度文件（R4、R5）：观察者在第 12 个状态上 panic，这一趟没跑完；进度文件不留，下一趟从头跑。
/// panic 那一片的片行没写（片行在观察者看完之后才写），删文件由 panic 展开时做。
// crash-case-check:not-a-crash-case 测的是断点续跑装置在判红时删进度文件：枚举第一条流的快档展开（quick_tier_expansion），观察者在第 12 个状态上判红就停，单跑几秒；不是要登记的放量用例
#[test]
fn a_run_that_goes_red_leaves_no_progress_file_behind() {
    let stream = first_stream("resume-red-run");
    let directory = fresh_progress_directory("red-run");
    let resume = settings(
        &directory,
        Layer0ResumeStart::ResumeFromTheProgressFile,
        Layer0ProgressFileAfterCompletion::KeptForTheTestThatInspectsIt,
    );
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut observed = 0u64;
        enumerate_layer0_in_state_slices(
            &stream.base,
            &stream.writes,
            &stream.segments,
            stream.root_index,
            &stream.versions,
            &quick_tier_expansion,
            one_state_per_slice_on(1),
            Some(&mut |_image: &CrashImage<'_>,
                       _report: &RecoveryReport,
                       _counts: &mut Layer0ObserverCounts| {
                observed += 1;
                assert!(observed < 12, "观察者在第 12 个状态上判红");
            }),
            &resume,
        )
    }));
    assert!(outcome.is_err(), "观察者判红，这一趟 panic");
    assert!(
        progress_files_in(&directory).is_empty(),
        "判红的那一趟不留进度文件：{:?}",
        progress_files_in(&directory)
    );
    std::fs::remove_dir_all(&directory).ok();
}
