//! 第一个文件删完之后的按 inode 号写与按号删，在回退交错的历史里交理想模型比（代码轮判决
//! `research/prompts/m3-close-w5-code-r1-main-verification.md` 第三节第 1 到 3 条）：
//! - β 只认这一版父链上的 inode 1 墓碑（第 1 条）：攻方腿的定向历史（删 1、建 3、回退到删之前、再删 1、回退到建 3 那一版、写 3）与它的种子 200
//!   那一段交错历史的形态（回退到第一次删完之后那一段、再回退一次、写那一段里建的号）各写成一段执行器历史，经挂着的会话发、每一版交模型比；
//!   模型照「回退那次的父是回退目标」带父链上那一次死亡 txg（`model::ModelRoot::death_txg_of_the_first_file`），实现读墓碑树与谱系树
//!   （`transaction::TransactionOutput::death_txg_of_the_deleted_first_file`），两边要对上。
//! - 随机历史在第一个文件删完之后也按号删别的文件（第 3 条，`history.rs` 的 `apply_unlink`）：按 inode 号写那一组比重的快档种子里数
//!   「接在第一个文件已删完的一版后面删别的号」那一形，至少一次。
//!
//! 攻方腿自己那段交错历史（它自己的小模型，种子 200 与 1051）照原样另钉在
//! `writes_by_inode_number_in_the_interleaved_histories_of_the_attack.rs`。

use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, AppliedEffect, ContentChoice,
    ContentLength, GeneratedHistory, GenerationWeights, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryRun, HistorySeed, HistoryStartingPoint, InodeCountChoice,
    RollbackTargetChoice, StepOutcome, WriteOfAnInodeChoice, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::model::ModelJudgementCounts;
use singlefs_harness::SharedStream;

/// K-U5 β 那一拒的成员名（会话交回 `Publish`，原因是它）。
const BETA_REFUSAL_MEMBER: &str =
    "DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion";

/// 按 inode 号写那一组比重的快档种子区间（同 `writes_by_inode_number_in_random_histories_judged_by_the_model.rs` 的
/// `SEED_OFFSET_OF_THE_INODE_WRITE_SAMPLING`、`INODE_WRITE_SAMPLING_SEEDS`、`INODE_WRITE_SAMPLING_OPERATIONS_PER_HISTORY`：数的就是那几段快档历史）。
const SEED_OFFSET_OF_THE_INODE_WRITE_SAMPLING: u64 = 80_000;
const INODE_WRITE_SAMPLING_SEEDS: u64 = 6;
const INODE_WRITE_SAMPLING_OPERATIONS_PER_HISTORY: usize = 40;

fn one_data_unit_of(fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::InsideOneDataUnit {
            selector: fill_seed,
        },
        fill_seed,
    }
}

/// 建一个 inode（号接着水位）。
fn create_one_inode() -> HistoryOperation {
    HistoryOperation::CreateInodes(InodeCountChoice { count_drawn: 1 })
}

/// 按 inode 号写活着的号里最大的那一个（选择子 3 的倍数取最大的号，`WriteOfAnInodeChoice::inode_among`）。
fn write_of_the_largest_live_inode(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::WriteOfAnInode(WriteOfAnInodeChoice {
        inode_selector: 0,
        content: one_data_unit_of(fill_seed),
    })
}

/// 按 inode 号写活着的号里最小的那一个（选择子 1：第 ⌊1 ÷ 3⌋ = 0 个）。
fn write_of_the_smallest_live_inode(fill_seed: u64) -> HistoryOperation {
    HistoryOperation::WriteOfAnInode(WriteOfAnInodeChoice {
        inode_selector: 1,
        content: one_data_unit_of(fill_seed),
    })
}

/// 管理员回退到根环里从新到旧第 `index_from_newest` 条根（0 是最新那条）。
fn roll_back_to_the_ring_root(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

fn execute_judged_by_the_model(operations: Vec<HistoryOperation>) -> HistoryRun {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    execute_history_with(
        &history,
        HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES,
        &SharedStream::new(),
        &mut |_| {},
    )
}

fn assert_published(run: &HistoryRun, steps: &[usize], what: &str) {
    for step in steps {
        assert!(
            matches!(
                run.outcomes[*step],
                StepOutcome::Applied(AppliedEffect::Published { .. })
            ),
            "{what}：第 {step} 步做成：{:?}",
            run.outcomes[*step]
        );
    }
}

fn assert_rolled_back(run: &HistoryRun, steps: &[usize], what: &str) {
    for step in steps {
        assert!(
            matches!(
                run.outcomes[*step],
                StepOutcome::Applied(AppliedEffect::RolledBack { .. })
            ),
            "{what}：第 {step} 步回退做成：{:?}",
            run.outcomes[*step]
        );
    }
}

fn assert_refused_by_beta(run: &HistoryRun, step: usize, what: &str) {
    assert!(
        matches!(&run.outcomes[step], StepOutcome::Refused { member } if member.contains(BETA_REFUSAL_MEMBER)),
        "{what}：第 {step} 步按 inode 号写被 β 拒：{:?}",
        run.outcomes[step]
    );
}

/// 攻方腿的定向历史（`research/prompts/m3-close-w5-code-r1-opus-model/attack_histories_of_multi_file_writes_and_unlinks.rs` 的
/// `beta_after_two_rollbacks_lets_a_write_to_an_inode_born_after_the_deletion_through`）写成执行器历史：第一个文件旁边建 inode 2、写它，
/// 删第一个文件（第 2 步：带数据的别的文件里没有 size 0 的，按号选落到第一个文件；意图、最后一批两条根），建 inode 3、写 3 被 β 拒，
/// 回退到写 2 那一版（根环从新到旧第 3 条：建 3、最后一批、意图之后），再删第一个文件（第 6 步：步号模 3 是 0 取第一个文件），
/// 回退到建 3 那一版（从新到旧第 3 条：最后一批、意图、上一次回退之后），写 3：这一版的父链上第一个文件死于第 2 步，3 生在那之后，β 拒；
/// 第 6 步删的那一次被第 7 步的回退抛下。每一版模型都对得上（跑完、没有新发现），β 那一拒比过两次。
/// 判别力：实现的 β 改回「墓碑树里 inode 1 的墓碑取最大的死亡 txg」（`crates/mutations.tsv` 以「K-U5 β 父链：」起头那一行）时第 8 步写 3 做成、
/// 模型要求拒，这一段停在第 8 步；模型回退那次照抄现行那一版的死亡 txg（以「K-U5 β 父链：模型」起头那一行）时模型在第 8 步说该放、实现拒了。
#[test]
fn the_directed_history_of_the_attack_refuses_the_write_after_two_rollbacks_as_the_model_requires()
{
    let run = execute_judged_by_the_model(vec![
        create_one_inode(),
        write_of_the_largest_live_inode(0x0201),
        HistoryOperation::Unlink,
        create_one_inode(),
        write_of_the_largest_live_inode(0x0301),
        roll_back_to_the_ring_root(3),
        HistoryOperation::Unlink,
        roll_back_to_the_ring_root(3),
        write_of_the_largest_live_inode(0x0302),
    ]);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert_published(&run, &[0, 1, 2, 3, 6], "攻方的定向历史");
    assert_rolled_back(&run, &[5, 7], "攻方的定向历史");
    assert_refused_by_beta(&run, 4, "回退之前写 3");
    assert_refused_by_beta(&run, 8, "回退到建 3 那一版之后写 3");
    let counts = &run.tally.model_counts;
    assert_eq!(
        counts.data_writes_refused_after_the_first_file_was_deleted_matched, 2,
        "模型要求的 β 那一拒比过两次：{counts:?}"
    );
}

/// 攻方种子 200 那一段交错历史的形态（`research/prompts/m3-close-w5-code-r1-opus-model/out/run-seed200-trace.log`：删 1、建、回退到删之前、
/// 再删 1、回退到第一次删完之后建的那一版、再回退到同一段里更晚的一版，写那一段里建的号）写成执行器历史：建 inode 2、写它，删第一个文件
/// （第 2 步），建 3、建 4，回退到写 2 那一版（从新到旧第 4 条），再删第一个文件（第 6 步），回退到建 3 那一版（从新到旧第 4 条：最后一批、
/// 意图、上一次回退、建 4 之后），再回退到建 4 那一版（从新到旧第 4 条：上一次回退、最后一批、意图、再上一次回退之后）——这一版的父链是
/// 「建 4 ← 建 3 ← 第一次删 1 ← 写 2」，跳过了两条谱系记录；写最大的号 4 被 β 拒（4 生在第一次删完之后），写最小的号 2 照写（删之前就有的）。
/// 判别力：实现的 β 改回取最大（第 6 步的死亡 txg 晚于 4 的出生代）时第 9 步做成、模型要求拒；改成「落在任何一段 (T_old, T_rb) 开区间里的都不算」
/// 而不看那条谱系记录在不在父链上时两次死亡都被排掉、第 9 步同样做成。
#[test]
fn the_interleaved_shape_of_the_attack_seed_two_hundred_refuses_the_write_after_three_rollbacks_as_the_model_requires(
) {
    let run = execute_judged_by_the_model(vec![
        create_one_inode(),
        write_of_the_largest_live_inode(0x0211),
        HistoryOperation::Unlink,
        create_one_inode(),
        create_one_inode(),
        roll_back_to_the_ring_root(4),
        HistoryOperation::Unlink,
        roll_back_to_the_ring_root(4),
        roll_back_to_the_ring_root(4),
        write_of_the_largest_live_inode(0x0411),
        write_of_the_smallest_live_inode(0x0212),
    ]);
    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.outcomes);
    assert_published(&run, &[0, 1, 2, 3, 4, 6, 10], "种子 200 的形态");
    assert_rolled_back(&run, &[5, 7, 8], "种子 200 的形态");
    assert_refused_by_beta(&run, 9, "三次回退之后写 4");
    let counts = &run.tally.model_counts;
    assert_eq!(
        (
            counts.data_writes_refused_after_the_first_file_was_deleted_matched,
            counts.writes_of_an_inode_other_than_the_first_matched,
        ),
        (1, 2),
        "β 那一拒比过一次（第 9 步），按 inode 号写做成两次（第 1、10 步）：{counts:?}"
    );
}

/// 按 inode 号写那一组比重的快档那几段（同 `writes_by_inode_number_in_random_histories_judged_by_the_model.rs` 的
/// `the_inode_write_weights_reach_both_extent_upper_segment_shapes_and_every_history_agrees_with_the_model` 跑的 6 段、每段 40 步）：
/// 每一段都跑完、模型每一步都对得上，接在第一个文件已删完的一版后面删别的号的最后一批比过至少一次（第 3 条）。
/// 判别力：执行器在第一个文件已删完的一版上不按号删（`crates/mutations.tsv` 以「随机历史删完第一个文件之后按号删：」起头那一行）时一次都没有，这一条红。
// harness-test-granularity:one-scenario 同一组比重的快档那几段历史合起来数一形，每段各自建池，数的是这一组快档罩没罩到
#[test]
fn the_fast_inode_write_histories_delete_another_inode_after_the_first_file_was_deleted() {
    let mut counts = ModelJudgementCounts::default();
    for offset in 0..INODE_WRITE_SAMPLING_SEEDS {
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
        counts.add(&run.tally.model_counts);
    }
    println!("{counts:?}");
    assert!(
        counts.deletions_of_an_inode_other_than_the_first_after_the_first_file_was_deleted_matched
            >= 1,
        "快档那几段里接在第一个文件已删完的一版后面删别的号至少一次：{counts:?}"
    );
}
