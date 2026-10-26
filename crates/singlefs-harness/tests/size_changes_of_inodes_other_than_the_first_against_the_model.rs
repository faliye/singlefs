//! 第一个文件之外带数据单元的 inode 缩短与变长逐版交理想模型比（判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5：
//! 只对模型，模型这一侧补齐四样——胶水按标签与截断条目读别的文件、截断字段按 inode 比、崩溃读回按 inode 给允许集合、快照与整理与截断三臂对
//! 别的文件带数据作答）。执行器经挂着的会话发 `UserChange::Truncate` 点名别的 inode（第一个文件活着而树里还有别的 inode 时选择子最高位是 1 的那一半，
//! `history::apply_truncate`），会话交回的每一版（先走空的各步、截断意图、各批、最后一批、变长那一次）逐版交理想模型比：别的文件读回的内容、
//! 每个单元按单元号的分配代、每条记录的截断字段、快照与 deadlist。每一步之后另按最新那条根重建这一版、按号读别的文件，交模型给的允许集合判
//! （[`singlefs_harness::model::crash_recovery_disagreement_of_the_inodes_other_than_the_first`]）。
//!
//! 钉两样：一段写死的历史走遍别的 inode 截断（截到单元中间、截到 0、截回洞里、截到单元边界）、变长（跨单元留洞）、快照下截断、整理之后截断，
//! 每一版模型都对得上、计数钉住；别的 inode 带数据单元时改 size 那一组比重在它自己那一段种子下跑完、模型每一步都对得上、别的 inode 的改 size、
//! 别的文件带数据单元时建快照与整理都真比过。
//! 判别力：模型或胶水在任一臂退回只认第一个文件（别的 inode 带数据单元时改 size、建快照、整理不答，走空截断的一步把别的 inode 当没有单元，
//! 快照下换下的别的文件的单元不进 deadlist，别的文件的截断字段不比或从错的偏移读）时这一段判对不上或计数为 0（`crates/mutations.tsv` 以
//! 「m4 多 inode 改 size 模型：」起头的几行）。

use std::collections::BTreeMap;

use singlefs_core::address::{FileOffsetInBytes, InodeNumber};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::recovery::{choose_root, choose_system_configuration};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, CompactionChoice,
    CompactionRunEnding, ContentChoice, ContentLength, GeneratedHistory, GenerationWeights,
    HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed,
    HistoryStartingPoint, InodeCountChoice, StepObservation, StepOutcome, TruncationChoice,
    TruncationTarget, WriteOfAnInodeChoice, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::model::{
    crash_recovery_disagreement_of_the_inodes_other_than_the_first,
    ModelContentsOfTheInodesOtherThanTheFirst, ModelJudgementCounts, ModelRootKey, ObservedFile,
};
use singlefs_harness::model_comparison::model_root_key;
use singlefs_harness::SharedStream;

/// 这一组比重的种子区间：从种子基往后的这一段（「要多测别的历史，只往后追加新的种子区间」，`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` 的文档注释；
/// 截断那一组在 70 000 起）。
const SEED_OFFSET_OF_THE_SAMPLING: u64 = 80_000;
/// 随时跑的那一批几段、每段几步（快档规模，同截断那一组）。
const SAMPLING_SEEDS: u64 = 4;
const SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;

/// 截断点名第一个文件之外的 inode（第一个文件活着时）：选择子最高位置 1，高 32 位取别的 inode 里第几个（按号升序，从 0 数），低 32 位交给目标的类。
fn selector_naming_the_other_inode(index_among_the_other_inodes: u64, low_bits: u64) -> u64 {
    (1 << 63) | (index_among_the_other_inodes << 32) | low_bits
}

/// 截断一次（目标按类取，`selector` 照 [`TruncationChoice`] 的读法）。
fn truncate(target: TruncationTarget, selector: u64) -> HistoryOperation {
    HistoryOperation::Truncate(TruncationChoice { target, selector })
}

/// 按 inode 号写 `data_units` 个单元（走顺序写入口，前面的单元装满净荷，最后一个装 1 + `selector` 字节）；`inode_selector` 照
/// [`WriteOfAnInodeChoice::inode_among`] 的读法取号。
fn write_of_an_inode(inode_selector: u64, data_units: u64, selector: u64) -> HistoryOperation {
    HistoryOperation::WriteOfAnInode(WriteOfAnInodeChoice {
        inode_selector,
        content: ContentChoice {
            length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units,
                selector,
            },
            fill_seed: 13,
        },
    })
}

/// 写死的那一段（起点是第一个文件，P 是净荷 32634 字节）：0 在第一个文件旁边建 inode 2、3；1 按号写 inode 3 三个单元（2 P + 101 字节）；
/// 2 按号写 inode 2 一百四十六个单元（145 P + 51 字节）；3 截 inode 3 到单元中间（意图重写尾单元、最后一批放掉 size 之外的）；4 inode 3 跨单元变长
/// （留洞，标签换成 3 或 4）；5 建快照；6 快照下截 inode 2 到 0（意图重写单元 0、一批放 144 个、最后一批放剩下的：换下的都生在快照之前，进 deadlist）；
/// 7 快照下 inode 3 截回洞里；8 挂着时整理一趟、推一批就崩、可写挂载一次（签发了的话整理意图留在 logged_ops 里）；9 inode 3 截到单元边界；
/// 10 第一个文件截到 0（别的文件带数据单元时第一个文件的截断照答）；11 按号整份写 inode 3 两个单元（截断字段清零）。
fn history_through_the_size_changes_of_the_other_inodes() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 2 }),
            write_of_an_inode(0, 3, 100),
            write_of_an_inode(4, 146, 50),
            truncate(
                TruncationTarget::IntoTheMiddleOfADataUnit,
                selector_naming_the_other_inode(1, 3),
            ),
            truncate(
                TruncationTarget::GrowingAcrossDataUnits,
                selector_naming_the_other_inode(1, 4),
            ),
            HistoryOperation::CreateSnapshot,
            truncate(
                TruncationTarget::ToZero,
                selector_naming_the_other_inode(0, 0),
            ),
            truncate(
                TruncationTarget::BackIntoTheHole,
                selector_naming_the_other_inode(1, 2),
            ),
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector: 0,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 1 },
            }),
            truncate(
                TruncationTarget::ToADataUnitBoundary,
                selector_naming_the_other_inode(1, 1),
            ),
            truncate(TruncationTarget::ToZero, 0),
            write_of_an_inode(0, 2, 7),
        ],
    }
}

/// 一步之后在盘上只读挂载一次（择根、扫环、施加前缀，`mounted_read::mount_read_only`），按挂载态里的 inode 记录逐个读出第一个文件之外每个活着的
/// inode（从 0 读到 size；打不开或读不出交 [`ObservedFile::Undecodable`]）。挂不上（树表 0 条的一版）时落点取择到的那条根、没有别的 inode。
fn inodes_other_than_the_first_read_back(
    image: &MemoryPool,
) -> (ModelRootKey, BTreeMap<u64, ObservedFile>) {
    let Ok(mounted) = mount_read_only(image) else {
        let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
        let root = choose_root(image, &system_configuration).expect("择得到根");
        return (
            model_root_key(root.instance, root.checkpoint_txg),
            BTreeMap::new(),
        );
    };
    let read_back = mounted
        .mounted
        .inode_records()
        .iter()
        .filter(|record| record.inode != 1 && record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
        .map(|record| {
            let read = mounted
                .mounted
                .open_file(image, InodeNumber(record.inode))
                .map_err(|failure| format!("打开：{failure:?}"))
                .and_then(|file| {
                    file.read_at(image, FileOffsetInBytes(0), file.inode_record().size)
                        .map(|read| read.bytes)
                        .map_err(|failure| format!("读：{failure:?}"))
                });
            (
                record.inode,
                match read {
                    Ok(bytes) => ObservedFile::Decoded(bytes),
                    Err(what) => ObservedFile::Undecodable { what },
                },
            )
        })
        .collect();
    (
        model_root_key(
            mounted.effective_root.instance,
            mounted.effective_root.checkpoint_txg,
        ),
        read_back,
    )
}

/// 跑一段历史；每一步之后读回的别的文件交模型给的允许集合判（判红的第一处记下来），交回这一段的结局、各步结局、模型计数与判过几次读回。
struct RunAgainstTheModel {
    ending: HistoryEnding,
    outcomes: Vec<StepOutcome>,
    model_counts: ModelJudgementCounts,
    truncations_naming_another_inode: usize,
    read_backs_judged: u64,
    first_read_back_disagreement: Option<String>,
}

fn run_against_the_model(history: &GeneratedHistory) -> RunAgainstTheModel {
    let mut read_backs_judged = 0;
    let mut first_read_back_disagreement = None;
    let run = execute_history_with(
        history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |observation: &StepObservation<'_>| {
            let committed: BTreeMap<ModelRootKey, ModelContentsOfTheInodesOtherThanTheFirst> =
                observation
                    .model
                    .committed_versions_of_the_inodes_other_than_the_first()
                    .into_iter()
                    .collect();
            let (chosen, read_back) = inodes_other_than_the_first_read_back(observation.image);
            read_backs_judged += 1;
            if let Some(disagreement) =
                crash_recovery_disagreement_of_the_inodes_other_than_the_first(
                    &committed, chosen, &read_back,
                )
            {
                first_read_back_disagreement.get_or_insert_with(|| {
                    format!("{:?} 之后：{disagreement:?}", observation.position)
                });
            }
        },
    );
    let truncations_naming_another_inode = run
        .inodes_named_by_the_truncations
        .values()
        .filter(|inode| **inode != 1)
        .count();
    RunAgainstTheModel {
        ending: run.ending,
        outcomes: run.outcomes,
        model_counts: run.tally.model_counts,
        truncations_naming_another_inode,
        read_backs_judged,
        first_read_back_disagreement,
    }
}

/// 写死的那一段每一版模型都对得上、每一步之后读回的别的文件都在允许集合里，而且模型真的比到了别的 inode 的每一种：带数据单元的改 size
/// （截到单元中间、跨单元变长、截到 0、截回洞里、截到单元边界）、它截断的各批、快照下的那几次、别的文件带数据单元时建快照；
/// 别的文件的截断字段逐版逐条比过、其中写过的也比过。
/// 判别力：模型的哪一臂退回只认第一个文件（`crates/mutations.tsv` 以「m4 多 inode 改 size 模型：」起头的几行）这一段判对不上、或那一格计数是 0。
#[test]
fn a_fixed_history_of_size_changes_of_other_inodes_holding_data_agrees_with_the_model_at_every_version(
) {
    let run = run_against_the_model(&history_through_the_size_changes_of_the_other_inodes());
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for (step, outcome) in run.outcomes.iter().enumerate() {
        assert!(
            matches!(
                outcome,
                StepOutcome::Applied(
                    AppliedEffect::Published { .. }
                        | AppliedEffect::SnapshotPublished { .. }
                        | AppliedEffect::Compacted { .. }
                )
            ),
            "第 {step} 步做成：{outcome:?}"
        );
    }
    assert_eq!(
        run.first_read_back_disagreement, None,
        "每一步之后读回的别的文件都在模型给的允许集合里"
    );
    assert_eq!(
        run.read_backs_judged,
        u64::try_from(run.outcomes.len()).expect("步数") + 1,
        "起点与每一步之后都判过读回"
    );
    assert_eq!(
        run.truncations_naming_another_inode, 5,
        "第 3、4、6、7、9 步点名别的 inode，第 10 步点名第一个文件"
    );
    let counts = &run.model_counts;
    println!("{counts:?}");
    assert!(
        counts.size_changes_of_inodes_other_than_the_first_holding_data_units_matched >= 5
            && counts.truncation_steps_of_inodes_other_than_the_first_holding_data_units_matched
                >= 3,
        "带数据单元的别的 inode 改 size 那一次（五次）与它截断的各批都比过：{counts:?}"
    );
    assert!(
        counts.size_changes_of_inodes_other_than_the_first_under_a_snapshot_matched > 0
            && counts
                .snapshots_created_beside_files_other_than_the_first_holding_data_units_matched
                > 0
            && counts.versions_with_deadlist_entries_compared > 0,
        "快照下截别的 inode、别的文件带数据时建快照、deadlist 有条目的版都比过：{counts:?}"
    );
    // 第 8 步整理那一趟推到哪由选段与区域扫描定（模型不答，判决 G7）：这一段里它签不签发、意图留不留到第 9 步不钉，打出来看；
    // 别的文件带数据时整理那一格由比重那一条钉（`the_weights_of_size_changes_of_other_inodes_run_every_history_to_the_end_with_the_model_agreeing`）。
    println!(
        "第 8 步整理：{:?}；整理意图活着时截别的 inode {} 次",
        run.outcomes[8],
        counts.size_changes_of_inodes_other_than_the_first_beside_a_live_compaction_intent_matched
    );
    assert!(
        counts.truncation_fields_of_the_inodes_other_than_the_first_compared > 0
            && counts.written_truncation_fields_of_the_inodes_other_than_the_first_compared > 0,
        "别的文件的截断字段逐版逐条比过、其中写过的也比过：{counts:?}"
    );
    assert_eq!(
        counts.required_refusals_matched, 0,
        "这一段没有模型要求的拒"
    );
}

/// 别的 inode 带数据单元时改 size 那一组比重（`GenerationWeights::SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST`）在它自己那一段种子下（快档规模
/// 4 段、每段 40 步）：每一段都跑完、模型每一版都对得上、每一步之后读回的别的文件都在允许集合里，别的 inode 的截断真进过入口、带数据单元的改 size
/// 模型真比过，别的文件带数据单元时建快照与整理模型也比过。判别力：比重里截断的份数改成 0、或执行器从不点名别的 inode 时这一条红；模型在别的 inode
/// 的某种发布上改错一样、或别的文件带数据单元时整理那一臂不答时那一段判对不上。
#[test]
fn the_weights_of_size_changes_of_other_inodes_run_every_history_to_the_end_with_the_model_agreeing(
) {
    let mut counts = ModelJudgementCounts::default();
    let mut truncations_naming_another_inode = 0;
    for offset in 0..SAMPLING_SEEDS {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::SIZE_CHANGES_OF_INODES_OTHER_THAN_THE_FIRST,
        );
        assert!(
            history
                .operations
                .iter()
                .any(|operation| operation.kind() == HistoryOperationKind::Truncate),
            "种子 {seed}（偏移 {offset}）抽到过截断：{:?}",
            history.operations
        );
        let run = run_against_the_model(&history);
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "种子 {seed}（偏移 {offset}）：{:?}",
            history.operations
        );
        assert_eq!(
            run.first_read_back_disagreement, None,
            "种子 {seed}（偏移 {offset}）：每一步之后读回的别的文件都在允许集合里"
        );
        truncations_naming_another_inode += run.truncations_naming_another_inode;
        counts.add(&run.model_counts);
    }
    println!(
        "别的 inode 改 size 那一组比重 {SAMPLING_SEEDS} 段：截断点名别的 inode {truncations_naming_another_inode} 次；模型计数 {counts:?}"
    );
    assert!(
        truncations_naming_another_inode > 0
            && counts.size_changes_of_inodes_other_than_the_first_holding_data_units_matched > 0,
        "截断点名过别的 inode、带数据单元的别的 inode 改 size 模型比过：{counts:?}"
    );
    assert!(
        counts.snapshots_created_beside_files_other_than_the_first_holding_data_units_matched > 0
            && counts
                .compaction_publishes_beside_files_other_than_the_first_holding_data_units_matched
                > 0,
        "别的文件带数据单元时建快照、整理模型都比过：{counts:?}"
    );
}
