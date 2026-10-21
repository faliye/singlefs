//! 随机历史加按 inode 号写数据（里程碑「删除与多次 COW」一·1，用户 2026-10-18 定「三条都新写，先给 core 加多 inode 写数据」；
//! C445（模型对拍的取样点不随写路径增长）：新入口要有对拍跑得到它的取样点）：执行器经挂着的会话一次建几个 inode（`HistoryOperation::CreateInodes`）、
//! 按 inode 号整份写（`HistoryOperation::WriteOfAnInode`，`UserChange::OverwriteOfAnInode` 与 `UserChange::SequentialWriteOfAnInode`），
//! 会话交回的每一版交理想模型比——第一个文件之外每个 inode 的号与读回的内容、它们每个数据单元的分配代逐个比（`model::IdealModel::judge_and_advance`）。
//! 这里钉三样：一段写死的历史在第一个文件旁边建 160 个 inode、写号 2（顺序写三个单元）、写号 161（上段从根兼叶长成两层）、写号 150（两片下段叶）、
//! 写第一个文件、重开可写挂载、经 inode 1 写第一个文件、装不下的覆盖写被拒，每一版模型都对得上；删完第一个文件之后建 inode 再按 inode 号写，
//! 实现与模型都拒（K-U5，用户 2026-10-19 定「拦」）；按 inode 号写那一组比重在它自己那一段种子下跑完、模型每一步都对得上，
//! 取样点罩到「上段长高」与「写的号落在上段第 0 片叶之外」两形；unlink 按号选（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
//! 第三节 M4），两段写死的历史与那一组比重都罩到删别的号、删第一个文件而别的还在、删到空、上段降层四形，删除每一步模型都对得上。

use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, ContentChoice,
    ContentLength, GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint, InodeCountChoice,
    StepOutcome, WriteOfAnInodeChoice, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::model::ModelJudgementCounts;
use singlefs_harness::SharedStream;

/// 按 inode 号写那一组比重的种子区间：从种子基往后的这一段（「要多测别的历史，只往后追加新的种子区间」，`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`
/// 的文档注释；快照那一组在 40 000 起、整理那两组在 50 000、60 000 起、截断那一组在 70 000 起）。
const SEED_OFFSET_OF_THE_INODE_WRITE_SAMPLING: u64 = 80_000;
/// 随时跑的那一批几段、每段几步。
const INODE_WRITE_SAMPLING_SEEDS: u64 = 6;
const INODE_WRITE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;
/// 慢档那一批：同一个区间从偏移 0 起多跑几段。
const INODE_WRITE_SAMPLING_SEEDS_OF_THE_SLOW_TIER: u64 = 24;

fn inside_one_data_unit(selector: u64, fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector },
        fill_seed,
    }
}

fn spanning_data_units(data_units: u64, fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units,
            selector: fill_seed,
        },
        fill_seed,
    }
}

/// 按 inode 号写：号的选择子按 `WriteOfAnInodeChoice::inode_among` 的取法（3 的倍数取最大的号，别的取第 ⌊选择子 ÷ 3⌋ 个，按树里活着的号升序）。
fn write_of_an_inode(inode_selector: u64, content: ContentChoice) -> HistoryOperation {
    HistoryOperation::WriteOfAnInode(WriteOfAnInodeChoice {
        inode_selector,
        content,
    })
}

/// 写死的那一段：起点带第一个文件（树里只有 inode 1）。号的选择子按建完 160 个之后树里活着的 161 个号（1、2 … 161）算。
fn history_writing_several_inodes_beside_the_first_file() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            // 第 0 步：建 160 个（号 2 … 161，都在最左那片叶容器里）。
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 160 }),
            // 第 1 步：号 2 顺序写三个单元（下段一片叶）。
            write_of_an_inode(4, spanning_data_units(3, 0x0201)),
            // 第 2 步：号 161 覆盖写（上段一片叶罩 143 个号，号 161 罩不进根兼叶：上段长成两层）。
            write_of_an_inode(0, inside_one_data_unit(77, 0x1611)),
            // 第 3 步：号 150 顺序写 145 个单元（下段两片叶，号落在上段第 1 片叶，不再长层）。
            write_of_an_inode(448, spanning_data_units(145, 0x1501)),
            // 第 4 步：写第一个文件（上段第 0 片叶到根那一串重写，别的文件照抄）。
            HistoryOperation::PublishOverwrite(inside_one_data_unit(9, 0x0101)),
            // 第 5 步：重开可写挂载：每一版从盘上重建，三个文件与 157 个空 inode 交模型比。
            HistoryOperation::CloseAndMountWritable,
            // 第 6 步：经 inode 号写 inode 1（就是覆盖写第一个文件），写成空。
            write_of_an_inode(
                1,
                ContentChoice {
                    length: ContentLength::Empty,
                    fill_seed: 0,
                },
            ),
            // 第 7 步：号 2 覆盖写装不下的内容：必拒（一个数据单元的契约）。
            write_of_an_inode(
                4,
                ContentChoice {
                    length: ContentLength::OneByteAboveDataUnitPayloadCapacity,
                    fill_seed: 0x0202,
                },
            ),
            // 第 8 步：unlink 按号选（步号 8：没有数据单元的别的文件里第 2 个，号 5）：删一个 size 0 的 inode，它没有上段条目，
            // 最后一批 extent 树整棵照抄、上段根不重写。
            HistoryOperation::Unlink,
            // 第 9 步：号 161 再写一次（跨两次挂载读回之后）。
            write_of_an_inode(0, spanning_data_units(2, 0x1612)),
            // 第 10 步：冷启动读回第一个文件。
            HistoryOperation::ColdStartRecover,
        ],
    }
}

/// 写死的那一段每一版模型都对得上（跑完、没有新发现）：按 inode 号写第一个文件之外的文件比过 4 次（第 1、2、3、9 步），其中长高 1 次（第 2 步）、
/// 号落在上段第 0 片叶之外 3 次（第 2、3、9 步）；装不下的那次是模型要求的拒；第 8 步 unlink 删了一个 size 0 的 inode（删别的号一次）；
/// 第一个文件之外的 inode 逐版比过号与内容。
/// 判别力：按 inode 号写的入口把别的号写成第一个文件（`crates/mutations.tsv` 以「一·1 模型对拍：」起头的两行，顺序写与覆盖写各一行）时
/// 第一个文件的内容与那个号的内容在模型那里对不上，这一段停在第 1 步（顺序写）或第 2 步（覆盖写）。
#[test]
fn a_fixed_history_writing_several_inodes_agrees_with_the_model_and_grows_the_extent_upper_segment()
{
    let history = history_writing_several_inodes_beside_the_first_file();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for step in [0, 1, 2, 3, 4, 6, 8, 9] {
        assert!(
            matches!(
                run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::Published { .. })
            ),
            "第 {step} 步做成：{:?}",
            run.outcomes[step]
        );
    }
    assert!(
        matches!(&run.outcomes[7], StepOutcome::Refused { member } if member.contains("ContentExceedsDataUnit")),
        "装不下的覆盖写被拒：{:?}",
        run.outcomes[7]
    );
    let counts = &run.tally.model_counts;
    println!("{counts:?}");
    assert_eq!(
        (
            counts.writes_of_an_inode_other_than_the_first_matched,
            counts.writes_growing_the_extent_upper_segment_matched,
            counts.writes_past_the_first_extent_upper_leaf_matched,
            counts.required_refusals_matched,
        ),
        (4, 1, 3, 1),
        "按 inode 号写第一个文件之外的文件、其中长高的、写的号落在第 0 片叶之外的、模型要求的拒"
    );
    assert!(
        counts.inodes_other_than_the_first_compared > 0
            && counts.contents_of_files_other_than_the_first_compared > 0,
        "第一个文件之外的 inode 逐版比过号与内容：{counts:?}"
    );
    assert_eq!(
        counts.deletions_of_an_inode_other_than_the_first_matched, 1,
        "第 8 步删 size 0 的 inode 5，最后一批比过"
    );
}

/// 删完第一个文件之后建 inode、再按 inode 号写（覆盖写与顺序写各一次）：实现拒成 `DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`
/// （D8（核心索引结构） 已定项 16 射程 K-U5，用户 2026-10-19 定「拦，K-U5 照旧欠着」），模型要求的也是这一拒、拒之前一个写都不发，两次都比过。
/// 判别力：实现那一判删掉（`crates/mutations.tsv` 以「一·1 K-U5：」起头的那一行）时写做成了，模型说该拒，这一段停在第 2 步。
#[test]
fn a_data_write_after_the_first_file_was_deleted_is_refused_by_the_implementation_as_the_model_requires(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::Unlink,
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 3 }),
            write_of_an_inode(0, inside_one_data_unit(12, 0x0301)),
            write_of_an_inode(1, spanning_data_units(2, 0x0302)),
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for step in [0, 1] {
        assert!(
            matches!(
                run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::Published { .. })
            ),
            "第 {step} 步做成：{:?}",
            run.outcomes[step]
        );
    }
    for step in [2, 3] {
        assert!(
            matches!(&run.outcomes[step], StepOutcome::Refused { member }
                if member.contains("DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion")),
            "第 {step} 步删完之后按 inode 号写被拒（K-U5）：{:?}",
            run.outcomes[step]
        );
    }
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.data_writes_refused_after_the_first_file_was_deleted_matched,
            counts.writes_of_an_inode_other_than_the_first_matched,
        ),
        (2, 0),
        "模型要求的 K-U5 那一拒比过两次，一次按 inode 号写都没做成"
    );
}

/// 写死的那一段（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4「模型对拍逐版比删除各步」）：第一个文件旁边建 160 个 inode，
/// 写号 2（三个单元）、号 161（上段长到层 1）、号 150（145 个单元、两片下段叶），之后 unlink 按号选（[`HistoryOperation::Unlink`] 用步号当选择子）：
/// 第 4 步删号 2、第 7 步删号 161（还剩号 150，上段仍是层 1）、第 10 步删号 150（先一个不是最后一批的批截掉 144 个，最后一批之后有条目的只剩 inode 1，
/// 上段降回层 0——上一版的第 0 片叶照抄成根）、第 12 步删第一个文件（树里还有 157 个 size 0 的 inode，这一版归带文件的一版），之后第 13 步给删之前
/// 就有的 inode 4 写数据照写、第 15 步给删完之后建的 inode 162 写数据必拒（K-U5 β）。每一版连同上段每个节点的分配代都交模型比。
fn history_deleting_files_beside_the_first_file_until_only_size_zero_inodes_remain(
) -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            // 第 0 步：建 160 个（号 2 … 161，与第一个文件同在最左那片叶容器里）。
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 160 }),
            // 第 1 步：号 2 顺序写三个单元。
            write_of_an_inode(4, spanning_data_units(3, 0x0211)),
            // 第 2 步：号 161 覆盖写（上段长到层 1）。
            write_of_an_inode(0, inside_one_data_unit(31, 0x1621)),
            // 第 3 步：号 150 顺序写 145 个单元（下段两片叶）。
            write_of_an_inode(448, spanning_data_units(145, 0x1511)),
            // 第 4 步：unlink（步号 4：带数据的别的文件 [2, 150, 161] 里第 0 个）删号 2。
            HistoryOperation::Unlink,
            // 第 5 步：重开可写挂载。
            HistoryOperation::CloseAndMountWritable,
            // 第 6 步：写第一个文件。
            HistoryOperation::PublishOverwrite(inside_one_data_unit(17, 0x0111)),
            // 第 7 步：unlink（步号 7：带数据的别的文件里最大的）删号 161。
            HistoryOperation::Unlink,
            // 第 8 步：写第一个文件。
            HistoryOperation::PublishOverwrite(inside_one_data_unit(23, 0x0112)),
            // 第 9 步：重开可写挂载。
            HistoryOperation::CloseAndMountWritable,
            // 第 10 步：unlink（步号 10：带数据的别的文件 [150] 里第 0 个）删号 150：一个不是最后一批的批、最后一批降层。
            HistoryOperation::Unlink,
            // 第 11 步：重开可写挂载。
            HistoryOperation::CloseAndMountWritable,
            // 第 12 步：unlink（步号 12：第一个文件）删第一个文件，树里还有 157 个 size 0 的 inode。
            HistoryOperation::Unlink,
            // 第 13 步：按 inode 号写删之前就有的号（活着的号升序里第 1 个，号 4）：照写（β）。
            write_of_an_inode(4, spanning_data_units(2, 0x0411)),
            // 第 14 步：建一个（号 162，删完第一个文件之后建的）。
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 1 }),
            // 第 15 步：按 inode 号写最大的号 162：必拒（K-U5 β）。
            write_of_an_inode(0, inside_one_data_unit(5, 0x1622)),
        ],
    }
}

/// 写死的那一段删别的号、删第一个文件而别的还在、上段降层，每一版模型都对得上（跑完、没有新发现），四样计数逐个钉住。
/// 判别力：模型删除那一臂删错号（`crates/mutations.tsv` 以「M4 模型删除那一臂：」起头的那一行）时第 4 步那一版第一个文件之外的 inode 号的集合对不上；
/// 规范形计划降层时把根当成这次重写（同一处起头的另一行）时第 10 步最后一批那一版上段根的分配代对不上。
#[test]
fn a_fixed_history_deleting_files_beside_the_first_file_agrees_with_the_model_at_every_deletion_step(
) {
    let history = history_deleting_files_beside_the_first_file_until_only_size_zero_inodes_remain();
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for step in [0, 1, 2, 3, 4, 6, 7, 8, 10, 12, 13, 14] {
        assert!(
            matches!(
                run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::Published { .. })
            ),
            "第 {step} 步做成：{:?}",
            run.outcomes[step]
        );
    }
    assert!(
        matches!(&run.outcomes[15], StepOutcome::Refused { member }
            if member.contains("DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion")),
        "第 15 步给删完之后建的 inode 写数据被拒（K-U5 β）：{:?}",
        run.outcomes[15]
    );
    let counts = &run.tally.model_counts;
    println!("{counts:?}");
    assert_eq!(
        (
            counts.deletions_of_an_inode_other_than_the_first_matched,
            counts.deletions_of_the_first_file_beside_other_inodes_matched,
            counts.final_deletion_batches_emptying_the_version_matched,
            counts.deletions_lowering_the_extent_upper_segment_matched,
            counts.deletion_batches_keeping_data_units_matched,
        ),
        (3, 1, 0, 1, 1),
        "删别的号 3 次（号 2、161、150）、删第一个文件而别的还在 1 次、删到空 0 次、上段降层 1 次（删号 150）、不是最后一批的批 1 次（号 150 的 145 个单元）"
    );
    assert_eq!(
        (
            counts.writes_of_an_inode_other_than_the_first_matched,
            counts.data_writes_refused_after_the_first_file_was_deleted_matched,
        ),
        (4, 1),
        "按 inode 号写第一个文件之外的文件做成 4 次（第 1、2、3、13 步）、K-U5 β 那一拒 1 次（第 15 步）"
    );
}

/// 写死的那一段删到空（判决第三节 M4「删到空」）：第一个文件旁边建 inode 2、写两个单元，第 4 步 unlink 删号 2、第 6 步 unlink 删第一个文件——
/// 树里一条记录都不剩，那一版是有树而文件集合为空的一版、上段是一个零条目根兼叶；每一版模型都对得上。
/// 判别力：最后一批把「删的是树里最后一条」答成树里还有记录（`crates/mutations.tsv` 以「M4 模型删除那一臂：」起头、说删空那一行）时第 6 步那一版
/// 的角色集合对不上。
#[test]
fn a_fixed_history_deleting_another_file_and_then_the_first_file_empties_the_version_as_the_model_requires(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 1 }),
            // 第 1 步：活着的号 [1, 2] 里第 1 个（号 2），顺序写两个单元。
            write_of_an_inode(4, spanning_data_units(2, 0x0221)),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(inside_one_data_unit(11, 0x0121)),
            // 第 4 步：unlink（步号 4：带数据的别的文件 [2] 里第 0 个）删号 2。
            HistoryOperation::Unlink,
            HistoryOperation::CloseAndMountWritable,
            // 第 6 步：unlink（步号 6：第一个文件）删第一个文件，树里一条都不剩。
            HistoryOperation::Unlink,
        ],
    };
    let run = execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    for step in [0, 1, 3, 4, 6] {
        assert!(
            matches!(
                run.outcomes[step],
                StepOutcome::Applied(AppliedEffect::Published { .. })
            ),
            "第 {step} 步做成：{:?}",
            run.outcomes[step]
        );
    }
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.deletions_of_an_inode_other_than_the_first_matched,
            counts.deletions_of_the_first_file_beside_other_inodes_matched,
            counts.final_deletion_batches_emptying_the_version_matched,
        ),
        (1, 0, 1),
        "删别的号 1 次（号 2）、删第一个文件时树里已没有别的 inode、删到空 1 次：{counts:?}"
    );
}

/// 按 inode 号写那一组比重在它自己那一段种子里跑前 `seeds` 段：每一段都跑完、模型每一步都对得上。交回合起来的模型计数与两种操作的结局。
struct InodeWriteSampling {
    histories: u64,
    writes_of_an_inode: singlefs_harness::history::OperationTally,
    inode_creations: singlefs_harness::history::OperationTally,
    unlinks: singlefs_harness::history::OperationTally,
    model_counts: ModelJudgementCounts,
}

fn run_the_inode_write_sampling(seeds: u64) -> InodeWriteSampling {
    let mut sampling = InodeWriteSampling {
        histories: 0,
        writes_of_an_inode: singlefs_harness::history::OperationTally::default(),
        inode_creations: singlefs_harness::history::OperationTally::default(),
        unlinks: singlefs_harness::history::OperationTally::default(),
        model_counts: ModelJudgementCounts::default(),
    };
    for offset in 0..seeds {
        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE
            .wrapping_add(SEED_OFFSET_OF_THE_INODE_WRITE_SAMPLING)
            .wrapping_add(offset);
        let history = generate_history_with_weights(
            HistorySeed(seed),
            INODE_WRITE_SAMPLING_OPERATIONS_PER_HISTORY,
            &GenerationWeights::WRITES_BY_INODE_NUMBER,
        );
        let run = execute_history_with(
            &history,
            HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
            &SharedStream::new(),
            &mut |_| {},
        );
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "种子 {seed}（偏移 {offset}）：{:?}",
            history.operations
        );
        sampling.histories += 1;
        for (kind, total) in [
            (
                HistoryOperationKind::WriteOfAnInode,
                &mut sampling.writes_of_an_inode,
            ),
            (
                HistoryOperationKind::CreateInodes,
                &mut sampling.inode_creations,
            ),
            (HistoryOperationKind::Unlink, &mut sampling.unlinks),
        ] {
            let tally = run
                .tally
                .operations_by_kind
                .get(&kind)
                .copied()
                .unwrap_or_default();
            total.applied += tally.applied;
            total.refused += tally.refused;
            total.not_applicable += tally.not_applicable;
        }
        sampling.model_counts.add(&run.tally.model_counts);
    }
    let counts = &sampling.model_counts;
    println!(
        "按 inode 号写那一组比重 {} 段（每段 {INODE_WRITE_SAMPLING_OPERATIONS_PER_HISTORY} 步）：按 inode 号写 {:?}、建 inode {:?}、unlink {:?}；模型比过按 inode 号写第一个文件之外的文件 {} 次（长高 {}、号落在上段第 0 片叶之外 {}），删完之后按 inode 号写被拒（K-U5）{} 次，第一个文件之外的 inode 逐版比过 {} 个（其中有数据的 {} 个）；删除四形：删别的号 {}、删第一个文件而别的还在 {}、删到空 {}、上段降层 {}（不是最后一批的批 {}）",
        sampling.histories,
        sampling.writes_of_an_inode,
        sampling.inode_creations,
        sampling.unlinks,
        counts.writes_of_an_inode_other_than_the_first_matched,
        counts.writes_growing_the_extent_upper_segment_matched,
        counts.writes_past_the_first_extent_upper_leaf_matched,
        counts.data_writes_refused_after_the_first_file_was_deleted_matched,
        counts.inodes_other_than_the_first_compared,
        counts.contents_of_files_other_than_the_first_compared,
        counts.deletions_of_an_inode_other_than_the_first_matched,
        counts.deletions_of_the_first_file_beside_other_inodes_matched,
        counts.final_deletion_batches_emptying_the_version_matched,
        counts.deletions_lowering_the_extent_upper_segment_matched,
        counts.deletion_batches_keeping_data_units_matched
    );
    sampling
}

/// 删除四形（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4：删别的号、删第一个文件而别的还在、删到空、上段降层）
/// 每一形都比过、对得上。
fn assert_every_deletion_shape_was_judged(counts: &ModelJudgementCounts) {
    assert!(
        counts.deletions_of_an_inode_other_than_the_first_matched > 0
            && counts.deletions_of_the_first_file_beside_other_inodes_matched > 0
            && counts.final_deletion_batches_emptying_the_version_matched > 0
            && counts.deletions_lowering_the_extent_upper_segment_matched > 0,
        "删别的号、删第一个文件而别的还在、删到空、上段降层四形都比过：{counts:?}"
    );
}

/// 按 inode 号写那一组比重（`GenerationWeights::WRITES_BY_INODE_NUMBER`）在它自己那一段种子下（6 段、每段 40 步）：每一段都跑完、模型每一版都对得上，
/// 按 inode 号写第一个文件之外的文件做成过，取样点罩到「上段长高」与「写的号落在上段第 0 片叶之外」两形，unlink 按号选罩到删除四形。
/// 判别力：比重里按 inode 号写的份数改成 0 时一次都没比过，这一条红；按 inode 号写的入口写错号时那一段判对不上、这一条红；unlink 不按号选
/// （恒删第一个文件）时删别的号与上段降层两形一次都没有，这一条红。
#[test]
fn the_inode_write_weights_reach_both_extent_upper_segment_shapes_and_every_history_agrees_with_the_model(
) {
    let sampling = run_the_inode_write_sampling(INODE_WRITE_SAMPLING_SEEDS);
    let counts = &sampling.model_counts;
    assert!(
        counts.writes_of_an_inode_other_than_the_first_matched > 0
            && counts.writes_growing_the_extent_upper_segment_matched > 0
            && counts.writes_past_the_first_extent_upper_leaf_matched > 0,
        "按 inode 号写、上段长高、写的号落在第 0 片叶之外三样都比过：{counts:?}"
    );
    assert_every_deletion_shape_was_judged(counts);
}

/// 同一组比重、同一段种子多跑几段（慢档）：每一段都跑完、模型每一步都对得上，删完之后按 inode 号写那一拒（K-U5）也比过，删除四形都比过。
#[test]
#[ignore = "harness 慢档用例：按 inode 号写那一组比重 24 段、每段 40 步，每段几百个 inode 与几百个单元的文件逐版对拍"]
fn the_inode_write_weights_over_more_seeds_run_every_history_to_the_end_with_the_model_agreeing() {
    let sampling = run_the_inode_write_sampling(INODE_WRITE_SAMPLING_SEEDS_OF_THE_SLOW_TIER);
    assert!(
        sampling
            .model_counts
            .data_writes_refused_after_the_first_file_was_deleted_matched
            > 0,
        "删完之后按 inode 号写那一拒一次都没比过：{:?}",
        sampling.model_counts
    );
    assert_every_deletion_shape_was_judged(&sampling.model_counts);
}
