//! 按偏移写进理想模型对拍（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节；部分写 core 实现员报告「部分写进模型对拍那一半」）：
//! 执行器经挂着的会话发 `UserChange::WriteAtOffset`（`HistoryOperation::WriteAtOffset`），会话交回的每一版（先走空的各步、这一次、其间推的抬 F）逐版交
//! 理想模型比——内容按偏移盖、size = max(写之前, 终点)、空隙与洞读 0，从偏移所在单元起到写之后最后一个单元一起重写（R4），数据单元的分配代按单元号
//! 逐个比，上段叶条目的标签在加单元、洞被写满之后照换；RefFS 翻译把第一个文件的一次按偏移写翻成一个 `WriteAtOffset` 连同它那次发布。这里钉三样：
//! 两段写死的历史（第一个文件一段走遍五格与快照下的写、交 RefFS 判；别的文件一段走遍几格与快照下的写），每一版模型都对得上、每一步真落到的格钉死；
//! 按偏移写那一组比重在它自己那一段种子下（4 段、每段 40 步）每一段都跑完、模型每一版都对得上、五格每一格都真落到过、RefFS 不判红。

mod common;

use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, ContentChoice, ContentLength,
    GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution, HistoryOperation,
    HistoryOperationKind, HistorySeed, HistoryStartingPoint, InodeCountChoice, PartialWriteChoice,
    PartialWriteOffsetCell, SnapshotChoice, StepOutcome, TruncationChoice, TruncationTarget,
    WriteOfAnInodeChoice, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::model::{ModelJudgementCounts, ObservedReadBack};
use singlefs_harness::model_comparison::observed_read_back;
use singlefs_harness::reference_filesystem_translation::{
    ReferenceFileSystemCrashMoment, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
};
use singlefs_harness::SharedStream;

/// 按偏移写那一组比重的种子区间：从种子基往后的这一段（「要多测别的历史，只往后追加新的种子区间」，`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`
/// 的文档注释；截断那一组在 70 000 起、按 inode 号写那几组在 80 000 起）。
const SEED_OFFSET_OF_THE_PARTIAL_WRITE_SAMPLING: u64 = 90_000;
/// 按偏移写那一组比重随时跑的那一批几段、每段几步（快档规模，同截断那一组）。
const PARTIAL_WRITE_SAMPLING_SEEDS: u64 = 4;
const PARTIAL_WRITE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;

fn payload() -> u64 {
    u64::try_from(data_unit_payload_capacity()).expect("32634")
}

/// 按偏移写那一步：写 `inode_selector` 点名的号（1 在只有第一个文件、或第一个文件旁边还有别的号时都取第一个文件；0 取最大的号），偏移落在 `cell`，
/// 偏移与长度按 `selector` 取。
fn write_at_offset(
    inode_selector: u64,
    cell: PartialWriteOffsetCell,
    selector: u64,
) -> HistoryOperation {
    HistoryOperation::WriteAtOffset(PartialWriteChoice {
        inode_selector,
        cell,
        selector,
        fill_seed: 11 + selector,
    })
}

/// 走顺序写入口的 `data_units` 个单元的内容：前面的单元装满净荷，最后一个装 1 + `selector` 字节（`selector` 小于净荷）。
fn content_of_data_units(data_units: u64, selector: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units,
            selector,
        },
        fill_seed: 5,
    }
}

/// 写第一个文件的那几步（`inode_selector` 1：活着的号里不是最大的那一个按位置取，树里只有它）。
const THE_FIRST_FILE: u64 = 1;
/// 写最大的那个号的那几步（`inode_selector` 是 3 的倍数取最大的号）。
const THE_LARGEST_INODE: u64 = 0;

/// 第一个文件那一段（P 是净荷 32634 字节）：0 顺序写 5 个单元（4 P + 1001 字节）；1 已有单元中间（单元 2 起点写 1 字节，单元 2–4 一起重写）；
/// 2 盖满一个单元（单元 1 整个净荷，单元 1–4 一起重写、单元 1 不读回）；3 跨末尾（末尾往前 6 字节起写 7 字节，变长 1 字节）；4 起点越过末尾留空隙
/// （偏移 6 P + 1002：单元 4 补满读回、单元 5 补整单元零、单元 6 读新槽残留）；5 跨单元变长一个净荷（单元 6 补零到满，单元 7 是洞，标签 3）；
/// 6 写进洞里（单元 7 读新槽残留，洞被写满、标签换回 1）；7 建快照；8 快照下盖满单元 0（八个单元一起重写，换下的进 deadlist）；9 读快照；
/// 10 可写挂载；11 挂载之后已有单元中间；12 冷启动。
fn history_of_the_first_file_through_every_offset_cell() -> GeneratedHistory {
    let payload = payload();
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(content_of_data_units(5, 1000)),
            write_at_offset(
                THE_FIRST_FILE,
                PartialWriteOffsetCell::InsideAnExistingDataUnit,
                2,
            ),
            write_at_offset(
                THE_FIRST_FILE,
                PartialWriteOffsetCell::CoveringAWholeExistingDataUnit,
                1,
            ),
            write_at_offset(THE_FIRST_FILE, PartialWriteOffsetCell::AcrossTheEnd, 5),
            write_at_offset(
                THE_FIRST_FILE,
                PartialWriteOffsetCell::PastTheEndLeavingAGap,
                2 * payload - 1,
            ),
            HistoryOperation::Truncate(TruncationChoice {
                target: TruncationTarget::GrowingAcrossDataUnits,
                selector: 0,
            }),
            write_at_offset(THE_FIRST_FILE, PartialWriteOffsetCell::IntoTheHole, 10),
            HistoryOperation::CreateSnapshot,
            write_at_offset(
                THE_FIRST_FILE,
                PartialWriteOffsetCell::CoveringAWholeExistingDataUnit,
                0,
            ),
            HistoryOperation::ReadSnapshot(SnapshotChoice {
                index_among_the_snapshots: 0,
            }),
            HistoryOperation::CloseAndMountWritable,
            write_at_offset(
                THE_FIRST_FILE,
                PartialWriteOffsetCell::InsideAnExistingDataUnit,
                4,
            ),
            HistoryOperation::ColdStartRecover,
        ],
    }
}

/// 第一个文件那一段每一版模型都对得上（跑完、没有新发现），每一步都做成，每一次按偏移写真落到的格就是抽的那一格（偏移与长度逐步钉绝对值），
/// 模型比过的按偏移写第一个文件 7 次、模型要求的拒一次都没有；冷启动读回的内容交 RefFS 判绿（翻译把每一次按偏移写翻成一个 `WriteAtOffset`）。
/// 判别力：模型把按偏移写答成整份重写（`crates/mutations.tsv` 里「m4 部分写进模型」起头的那一行）时第 1 步判对不上；空隙不补零、洞读的不是 0
/// 时第 4、6 步判对不上。
#[test]
fn a_fixed_history_of_the_first_file_through_every_offset_cell_agrees_with_the_model_and_the_reference_filesystem(
) {
    let history = history_of_the_first_file_through_every_offset_cell();
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let mut translation =
        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
    let mut read_back_after_the_last_step: Option<ObservedReadBack> = None;
    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
        translation.note_an_observed_step(observation, stream.operation_count());
        read_back_after_the_last_step = Some(observed_read_back(
            &recover(observation.image, JournalPolicy::Consult).outcome,
        ));
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for (step, outcome) in run.outcomes.iter().enumerate() {
        assert!(
            matches!(outcome, StepOutcome::Applied(_)),
            "第 {step} 步做成：{outcome:?}"
        );
    }
    let payload = payload();
    let size_after_the_overwrite = 4 * payload + 1001;
    assert_eq!(
        run.partial_writes_requested
            .iter()
            .map(|(step, requested)| (
                *step,
                requested.offset_in_bytes,
                requested.length_in_bytes,
                requested.offset_cell
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                1,
                2 * payload,
                1,
                PartialWriteOffsetCell::InsideAnExistingDataUnit
            ),
            (
                2,
                payload,
                payload,
                PartialWriteOffsetCell::CoveringAWholeExistingDataUnit
            ),
            (
                3,
                size_after_the_overwrite - 6,
                7,
                PartialWriteOffsetCell::AcrossTheEnd
            ),
            (
                4,
                6 * payload + 1002,
                1,
                PartialWriteOffsetCell::PastTheEndLeavingAGap
            ),
            (6, 7 * payload + 10, 1, PartialWriteOffsetCell::IntoTheHole),
            (
                8,
                0,
                payload,
                PartialWriteOffsetCell::CoveringAWholeExistingDataUnit
            ),
            (
                11,
                4 * payload,
                1,
                PartialWriteOffsetCell::InsideAnExistingDataUnit
            ),
        ],
        "每一次按偏移写交给会话的偏移、长度与真落到的格"
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.partial_writes_of_the_first_file_matched,
            counts.partial_writes_of_an_inode_other_than_the_first_matched,
            counts.required_refusals_matched,
        ),
        (7, 0, 0),
        "模型比过的按偏移写第一个文件、写别的文件、模型要求的拒"
    );
    let read_back = read_back_after_the_last_step.expect("起点那一步也交给观察者");
    let translated = translation.translate(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    assert_eq!(
        translated
            .judge_a_crash(
                ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
                &read_back
            )
            .vote,
        ReferenceFileSystemVote::Green
    );
}

/// 别的文件那一段（P 是净荷 32634 字节）：0 在第一个文件旁边建 inode 2；1 按 inode 号整份写 inode 2 三个单元（2 P + 501 字节）；2 已有单元中间
/// （单元 1 起点写 1 字节，单元 1、2 一起重写）；3 跨末尾（末尾往前 6 字节起写 7 字节）；4 起点越过末尾留空隙（单元 2 补满、单元 3 补零、单元 4
/// 读残留）；5 跨单元变长 inode 2（留洞）；6 写进 inode 2 的洞；7 建快照；8 快照下盖满 inode 2 的单元 0（全部单元一起重写，换下的进 deadlist）；
/// 9 可写挂载；10 挂载之后已有单元中间。
fn history_of_an_inode_other_than_the_first_through_the_offset_cells() -> GeneratedHistory {
    let payload = payload();
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 1 }),
            HistoryOperation::WriteOfAnInode(WriteOfAnInodeChoice {
                inode_selector: THE_LARGEST_INODE,
                content: content_of_data_units(3, 500),
            }),
            write_at_offset(
                THE_LARGEST_INODE,
                PartialWriteOffsetCell::InsideAnExistingDataUnit,
                1,
            ),
            write_at_offset(THE_LARGEST_INODE, PartialWriteOffsetCell::AcrossTheEnd, 5),
            write_at_offset(
                THE_LARGEST_INODE,
                PartialWriteOffsetCell::PastTheEndLeavingAGap,
                2 * payload - 1,
            ),
            HistoryOperation::Truncate(TruncationChoice {
                target: TruncationTarget::GrowingAcrossDataUnits,
                selector: 1 << 63,
            }),
            write_at_offset(THE_LARGEST_INODE, PartialWriteOffsetCell::IntoTheHole, 10),
            HistoryOperation::CreateSnapshot,
            write_at_offset(
                THE_LARGEST_INODE,
                PartialWriteOffsetCell::CoveringAWholeExistingDataUnit,
                0,
            ),
            HistoryOperation::CloseAndMountWritable,
            write_at_offset(
                THE_LARGEST_INODE,
                PartialWriteOffsetCell::InsideAnExistingDataUnit,
                1,
            ),
        ],
    }
}

/// 别的文件那一段每一版模型都对得上、每一步都做成，每一次按偏移写写的是 inode 2、真落到的格就是抽的那一格，模型比过的按偏移写别的文件 6 次、
/// 写第一个文件 0 次。判别力：模型对别的文件重写的那一段少算偏移所在的那一个单元（`crates/mutations.tsv` 一行）时第 2 步判对不上。
#[test]
fn a_fixed_history_of_an_inode_other_than_the_first_through_the_offset_cells_agrees_with_the_model()
{
    let history = history_of_an_inode_other_than_the_first_through_the_offset_cells();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for (step, outcome) in run.outcomes.iter().enumerate() {
        assert!(
            matches!(outcome, StepOutcome::Applied(_)),
            "第 {step} 步做成：{outcome:?}"
        );
    }
    assert_eq!(
        run.partial_writes_requested
            .iter()
            .map(|(step, requested)| (*step, requested.inode, requested.offset_cell))
            .collect::<Vec<_>>(),
        vec![
            (2, 2, PartialWriteOffsetCell::InsideAnExistingDataUnit),
            (3, 2, PartialWriteOffsetCell::AcrossTheEnd),
            (4, 2, PartialWriteOffsetCell::PastTheEndLeavingAGap),
            (6, 2, PartialWriteOffsetCell::IntoTheHole),
            (8, 2, PartialWriteOffsetCell::CoveringAWholeExistingDataUnit),
            (10, 2, PartialWriteOffsetCell::InsideAnExistingDataUnit),
        ],
        "每一次按偏移写写的号与真落到的格"
    );
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.partial_writes_of_an_inode_other_than_the_first_matched,
            counts.partial_writes_of_the_first_file_matched,
            counts.required_refusals_matched,
        ),
        (6, 0, 0),
        "模型比过的按偏移写别的文件、写第一个文件、模型要求的拒"
    );
}

/// 按偏移写那一组比重在它自己那一段种子里跑前 `seeds` 段：每一段都跑完、模型每一步都对得上；每段停下时冷启动读回的内容交 RefFS 判
/// （「序列里的操作都返回过之后崩溃」那一刻），不许判红（翻译停在写别的文件、unlink 那一步的段弃权）。交回每一格真落到的次数、RefFS 判绿的段数
/// 与模型比过的按偏移写。
struct PartialWriteSampling {
    offset_cells_attempted: std::collections::BTreeMap<&'static str, u64>,
    partial_writes_applied: u64,
    reference_filesystem_greens: u64,
    model_counts: ModelJudgementCounts,
}

fn run_the_partial_write_sampling(seeds: u64) -> PartialWriteSampling {
    let mut sampling = PartialWriteSampling {
        offset_cells_attempted: std::collections::BTreeMap::new(),
        partial_writes_applied: 0,
        reference_filesystem_greens: 0,
        model_counts: ModelJudgementCounts::default(),
    };
    for offset in 0..seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_PARTIAL_WRITE_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            PARTIAL_WRITE_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::PARTIAL_WRITES,
        );
        let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
        let stream = SharedStream::retaining_contents();
        let mut translation =
            ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
        let mut read_back_after_the_last_step: Option<ObservedReadBack> = None;
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            translation.note_an_observed_step(observation, stream.operation_count());
            read_back_after_the_last_step = Some(observed_read_back(
                &recover(observation.image, JournalPolicy::Consult).outcome,
            ));
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "种子 {seed}（偏移 {offset}）：{:?}",
            history.operations
        );
        let translated = translation.translate(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let read_back = read_back_after_the_last_step.expect("起点那一步也交给观察者");
        let judgement = translated.judge_a_crash(
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
            &read_back,
        );
        match judgement.vote {
            ReferenceFileSystemVote::Green => sampling.reference_filesystem_greens += 1,
            ReferenceFileSystemVote::Abstained(_) => {}
            ReferenceFileSystemVote::Red { reason } => {
                panic!("种子 {seed}（偏移 {offset}）：RefFS 判红：{reason}")
            }
        }
        for (cell, count) in &run.tally.partial_write_offset_cells_attempted {
            *sampling.offset_cells_attempted.entry(cell).or_insert(0) += count;
        }
        sampling.partial_writes_applied += run
            .tally
            .operations_by_kind
            .get(&HistoryOperationKind::WriteAtOffset)
            .map_or(0, |tally| tally.applied);
        sampling.model_counts.add(&run.tally.model_counts);
    }
    eprintln!(
        "按偏移写那一组比重 {seeds} 段：各格真落到 {:?}；做成 {} 次；RefFS 判绿 {} 段；模型比过写第一个文件 {}、写别的文件 {}",
        sampling.offset_cells_attempted,
        sampling.partial_writes_applied,
        sampling.reference_filesystem_greens,
        sampling.model_counts.partial_writes_of_the_first_file_matched,
        sampling
            .model_counts
            .partial_writes_of_an_inode_other_than_the_first_matched
    );
    sampling
}

/// 按偏移写那一组比重（`GenerationWeights::PARTIAL_WRITES`）在它自己那一段种子下（快档规模 4 段、每段 40 步）：每一段都跑完、模型每一版都对得上，
/// 判决第二节那五格（已有单元中间、盖满一个单元、跨末尾、起点越过末尾留空隙、写进洞里）每一格都真落到过，按偏移写第一个文件模型比过，RefFS 判过绿、
/// 一次都没判红。判别力：比重里按偏移写的份数改成 0、或偏移取样少了「起点越过末尾留空隙」那一格（`crates/mutations.tsv` 两行）时这一条红；
/// 模型或实现在按偏移写上改错一样时那一段判对不上、这一条红。
#[test]
fn the_partial_write_weights_land_on_every_offset_cell_and_every_history_agrees_with_the_model() {
    let sampling = run_the_partial_write_sampling(PARTIAL_WRITE_SAMPLING_SEEDS);
    for cell in PartialWriteOffsetCell::DRAWN {
        assert!(
            sampling
                .offset_cells_attempted
                .get(cell.name())
                .is_some_and(|count| *count > 0),
            "按偏移写的「{}」那一格在 {PARTIAL_WRITE_SAMPLING_SEEDS} 段里一次都没真落到过：{:?}",
            cell.name(),
            sampling.offset_cells_attempted
        );
    }
    assert!(
        sampling
            .model_counts
            .partial_writes_of_the_first_file_matched
            > 0,
        "按偏移写第一个文件模型一次都没比过：{:?}",
        sampling.model_counts
    );
    assert!(
        sampling.reference_filesystem_greens > 0,
        "RefFS 一段都没判过绿"
    );
}
