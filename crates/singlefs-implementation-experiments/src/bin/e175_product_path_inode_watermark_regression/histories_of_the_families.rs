//! 每个族的第几段历史是哪一串操作：定向族、定向穷举、带回滚的随机族与已有的随机族，操作的代码与会不会结束会话。
use super::*;

/// D 族：D1 一段、D2(k) 六段（k = 1..=6）、D3 一段。
pub(super) const DIRECTED_HISTORY_COUNT: u64 = 8;
const LARGEST_DIRECTED_ROLLBACK_INDEX: u64 = 6;

fn ring_root(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

/// D 族第 `index` 段的操作表与名字（登记第五节 D 那一行）。
fn directed_history(index: u64) -> (String, Vec<HistoryOperation>) {
    match index {
        0 => ("D1".to_string(), Vec::new()),
        rollback_index @ 1..=LARGEST_DIRECTED_ROLLBACK_INDEX => (
            format!("D2({rollback_index})"),
            vec![
                HistoryOperation::Unlink,
                HistoryOperation::CreateInode,
                HistoryOperation::CreateInode,
                ring_root(rollback_index),
                HistoryOperation::CreateInode,
            ],
        ),
        7 => (
            "D3".to_string(),
            vec![
                HistoryOperation::Unlink,
                HistoryOperation::CreateInode,
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::CreateInode,
                HistoryOperation::ColdStartRecover,
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::CreateInode,
            ],
        ),
        other => panic!("D 族只有 {DIRECTED_HISTORY_COUNT} 段，序号 {other} 越界"),
    }
}

/// F3a 中段字母表（登记第五节，11 个，次序就是编号次序）。
fn directed_exhaustive_middle_alphabet() -> [HistoryOperation; 11] {
    [
        HistoryOperation::CreateInode,
        ring_root(1),
        ring_root(2),
        ring_root(3),
        ring_root(5),
        ring_root(8),
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::ColdStartRecover,
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
            steps_above_current_floor: 1,
        }),
        HistoryOperation::Unlink,
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed: 5,
        }),
    ]
}

/// 中段长 0..=`longest` 的序列一共几条：Σ 11^L。
pub(super) fn directed_exhaustive_sequences_up_to_middle_length(longest: u32) -> u64 {
    (0..=longest)
        .map(|length| DIRECTED_EXHAUSTIVE_ALPHABET_SIZE.pow(length))
        .sum()
}

/// F3a 第 `index` 条的中段（字母表编号，按长度从短到长、同长按 11 进制从小到大排）。
fn directed_exhaustive_middle_letters(index: u64) -> Vec<u64> {
    let mut remaining = index;
    let mut length = 0_u32;
    // 迭代上界：每轮 remaining 至少减 1（11^L ≥ 1），减到落进某一长度那一段就出循环。
    loop {
        let sequences_of_this_length = DIRECTED_EXHAUSTIVE_ALPHABET_SIZE.pow(length);
        if remaining < sequences_of_this_length {
            break;
        }
        remaining -= sequences_of_this_length;
        length += 1;
    }
    let mut letters_most_significant_first = Vec::new();
    for position in (0..length).rev() {
        let place_value = DIRECTED_EXHAUSTIVE_ALPHABET_SIZE.pow(position);
        letters_most_significant_first.push(remaining / place_value);
        remaining %= place_value;
    }
    letters_most_significant_first
}

fn directed_exhaustive_operations(index: u64) -> Vec<HistoryOperation> {
    let alphabet = directed_exhaustive_middle_alphabet();
    let mut operations = vec![
        HistoryOperation::Unlink,
        HistoryOperation::CreateInode,
        HistoryOperation::CreateInode,
    ];
    for letter in directed_exhaustive_middle_letters(index) {
        operations.push(alphabet[usize::try_from(letter).expect("字母表编号 < 11")]);
    }
    operations.push(HistoryOperation::CreateInode);
    operations.push(HistoryOperation::CreateInode);
    operations
}

/// F2 的比重（登记第五节那张表；三张操作表各和为 100，`random_with_rollbacks_weight_tables_each_sum_to_one_hundred_and_generate_sixty_steps` 钉住）。
static RANDOM_WITH_ROLLBACKS_STARTING_POINTS: [(HistoryStartingPoint, u64); 1] =
    [(HistoryStartingPoint::AfterFirstFile, 1)];
static RANDOM_WITH_ROLLBACKS_WITH_SESSION_CLOSED: [(HistoryOperationKind, u64); 2] = [
    (HistoryOperationKind::CloseAndMountWritable, 90),
    (HistoryOperationKind::ColdStartRecover, 10),
];
static RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITHOUT_FILE: [(HistoryOperationKind, u64); 5] = [
    (HistoryOperationKind::PublishFirstFile, 70),
    (HistoryOperationKind::CloseAndMountWritable, 10),
    (HistoryOperationKind::RollBackWhileMounted, 10),
    (HistoryOperationKind::ColdStartRecover, 5),
    (
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
        5,
    ),
];
static RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITH_FILE: [(HistoryOperationKind, u64); 7] = [
    (HistoryOperationKind::Unlink, 40),
    (HistoryOperationKind::RollBackWhileMounted, 20),
    (HistoryOperationKind::PublishOverwrite, 15),
    (HistoryOperationKind::CloseAndMountWritable, 10),
    (
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
        5,
    ),
    (HistoryOperationKind::RaiseRollbackFloor, 5),
    (HistoryOperationKind::ColdStartRecover, 5),
];
static RANDOM_WITH_ROLLBACKS_AFTER_THE_FILE_WAS_DELETED: [(HistoryOperationKind, u64); 7] = [
    (HistoryOperationKind::CreateInode, 35),
    (HistoryOperationKind::RollBackWhileMounted, 35),
    (HistoryOperationKind::CloseAndMountWritable, 10),
    (
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
        5,
    ),
    (HistoryOperationKind::ColdStartRecover, 5),
    (HistoryOperationKind::RaiseRollbackFloor, 5),
    (HistoryOperationKind::PublishOverwrite, 5),
];

/// `named` 借 `DeletingFilesOfSeveralLowerLeaves` 当标签（登记第五节）：复现靠这张表加种子，不靠环境变量。
pub(super) const RANDOM_WITH_ROLLBACKS_WEIGHTS: GenerationWeights = GenerationWeights {
    name: "E175 F2：偏重回退与建 inode",
    named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
    starting_points: &RANDOM_WITH_ROLLBACKS_STARTING_POINTS,
    with_session_closed: &RANDOM_WITH_ROLLBACKS_WITH_SESSION_CLOSED,
    with_session_open_without_file: &RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITHOUT_FILE,
    with_session_open_with_file: &RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITH_FILE,
    with_session_open_after_the_file_was_deleted: Some(
        &RANDOM_WITH_ROLLBACKS_AFTER_THE_FILE_WAS_DELETED,
    ),
    rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
    overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
};

/// G-draw（登记第八节）：F2 那组比重，只把回退目标的抽法换成 `RecentUniformOrBeyond`（偏向最近四条与环外）。
pub(super) const RANDOM_WITH_ROLLBACKS_WEIGHTS_DRAWING_RECENT_UNIFORM_OR_BEYOND: GenerationWeights =
    GenerationWeights {
        name: "E175 G-draw：偏重回退与建 inode，回退目标照第一版抽法",
        named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
        starting_points: &RANDOM_WITH_ROLLBACKS_STARTING_POINTS,
        with_session_closed: &RANDOM_WITH_ROLLBACKS_WITH_SESSION_CLOSED,
        with_session_open_without_file: &RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITHOUT_FILE,
        with_session_open_with_file: &RANDOM_WITH_ROLLBACKS_WITH_SESSION_OPEN_WITH_FILE,
        with_session_open_after_the_file_was_deleted: Some(
            &RANDOM_WITH_ROLLBACKS_AFTER_THE_FILE_WAS_DELETED,
        ),
        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
        overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
    };

/// 一个随机族的一段：种子 = 种子基 + 序号，标签写种子。
fn random_history_item(family: Family, index: u64, weights: &GenerationWeights) -> HistoryItem {
    let seed = HistorySeed(SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE.wrapping_add(index));
    let operation_count = family
        .operations_per_random_history()
        .expect("随机族才走这里");
    HistoryItem {
        family,
        index,
        label: format!("seed{}", seed.0),
        history: singlefs_verification::history::generate_history_with_weights(
            seed,
            operation_count,
            weights,
        ),
    }
}

pub(super) fn history_item(family: Family, index: u64) -> HistoryItem {
    let seed_base = SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE;
    match family {
        Family::Directed => {
            let (name, operations) = directed_history(index);
            HistoryItem {
                family,
                index,
                label: name,
                history: GeneratedHistory {
                    seed: HistorySeed(seed_base),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations,
                },
            }
        }
        Family::DirectedExhaustive => {
            let letters: Vec<String> = directed_exhaustive_middle_letters(index)
                .iter()
                .map(u64::to_string)
                .collect();
            HistoryItem {
                family,
                index,
                label: format!("middle[{}]", letters.join(",")),
                history: GeneratedHistory {
                    seed: HistorySeed(seed_base),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations: directed_exhaustive_operations(index),
                },
            }
        }
        Family::RandomWithRollbacks => {
            let seed = HistorySeed(seed_base.wrapping_add(index));
            HistoryItem {
                family,
                index,
                label: format!("seed{}", seed.0),
                history: singlefs_verification::history::generate_history_with_weights(
                    seed,
                    RANDOM_WITH_ROLLBACKS_OPERATIONS_PER_HISTORY,
                    &RANDOM_WITH_ROLLBACKS_WEIGHTS,
                ),
            }
        }
        Family::ExistingRandom(group) => random_history_item(family, index, &group.weights()),
        Family::Geometry(
            GeometryPoint::LongerRandomWithRollbacks
            | GeometryPoint::RandomWithRollbacksOnSmallDevices,
        ) => random_history_item(family, index, &RANDOM_WITH_ROLLBACKS_WEIGHTS),
        Family::Geometry(GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond) => {
            random_history_item(
                family,
                index,
                &RANDOM_WITH_ROLLBACKS_WEIGHTS_DRAWING_RECENT_UNIFORM_OR_BEYOND,
            )
        }
        Family::Geometry(GeometryPoint::LongerBroad) => {
            random_history_item(family, index, &GenerationWeights::BROAD)
        }
    }
}

/// 操作的短写（结果行里的操作表，凭它与族、序号能单独复现那一段）。
pub(super) fn operation_code(operation: &HistoryOperation) -> String {
    match operation {
        HistoryOperation::PublishFirstFile(content) => format!("first({:?}/{})", content.length, content.fill_seed),
        HistoryOperation::PublishOverwrite(content) => format!("overwrite({:?}/{})", content.length, content.fill_seed),
        HistoryOperation::PublishWithoutUnits => "empty".to_string(),
        HistoryOperation::CloseAndMountWritable => "mount".to_string(),
        HistoryOperation::RollBackWhileMounted(target) => format!("rollback({target:?})"),
        HistoryOperation::RaiseRollbackFloor(choice) => format!("raise({})", choice.steps_above_current_floor),
        HistoryOperation::ColdStartRecover => "cold".to_string(),
        HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => "crash".to_string(),
        HistoryOperation::Unlink => "unlink".to_string(),
        HistoryOperation::CreateInode => "create".to_string(),
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
            format!("powerloss({:?}/{})", content.length, content.fill_seed)
        }
        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            "mount-zero-slot".to_string()
        }
        // 只藏第一块盘那一槽的新步：与每块盘都藏那一步同属“挂载类”，操作表里单列一个短写，免得两步在结果行里混成一个码。
        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => {
            "mount-zero-slot-one".to_string()
        }
        // 以下八类 E175 的取号臂（F2 / F1 / 几何点各组比重）从不生成；为了在今天的 crates 上穷尽匹配而列，各给一个短写，免得混码。
        // 它们都不动 inode 水位的读法：建快照、读快照、删快照、整理、截断、按 inode 号写、按偏移写不发新的 inode 号；CreateInodes 发号，见下面 count_operation_specific 的注。
        HistoryOperation::CreateSnapshot => "snap-create".to_string(),
        HistoryOperation::ReadSnapshot(_) => "snap-read".to_string(),
        HistoryOperation::DestroySnapshot(_) => "snap-destroy".to_string(),
        HistoryOperation::CompactWhileMounted(_) => "compact".to_string(),
        HistoryOperation::Truncate(_) => "truncate".to_string(),
        HistoryOperation::CreateInodes(_) => "create-many".to_string(),
        HistoryOperation::WriteOfAnInode(_) => "write-inode".to_string(),
        HistoryOperation::WriteAtOffset(_) => "write-offset".to_string(),
    }
    .replace(' ', "")
}

/// 这一步是不是挂载 / 恢复类（关掉会话再起；Q5 的窗口到它为止，不论结局）。
pub(super) fn operation_ends_the_session(kind: HistoryOperationKind) -> bool {
    match kind {
        HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::ColdStartRecover
        | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => true,
        HistoryOperationKind::PublishFirstFile
        | HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::Unlink
        | HistoryOperationKind::CreateInode
        // 只读快照：会话不关（建快照是会话内一次发布，读快照不写盘），Q5 的窗口不在这里结束。
        | HistoryOperationKind::CreateSnapshot
        | HistoryOperationKind::ReadSnapshot
        | HistoryOperationKind::DestroySnapshot
        // 整理：只有“批间崩溃再可写挂载”那一种会关会话，但这里只有种类、看不到那一选择；按不结束算（窗口多开着，Q5 只会多记不会漏记）。E175 不生成这一类。
        | HistoryOperationKind::CompactWhileMounted
        | HistoryOperationKind::Truncate
        | HistoryOperationKind::CreateInodes
        | HistoryOperationKind::WriteOfAnInode
        | HistoryOperationKind::WriteAtOffset => false,
    }
}

#[cfg(test)]
mod tests;
