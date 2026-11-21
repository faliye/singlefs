//! 分区格的整跑、各臂与对照的格规格、各问的判定与逐问、失败条款、几何敏感性与第一段的产物行。

use super::*;

struct PartitionCellResult {
    specification: CellSpecification,
    pub(super) aggregate: Aggregate,
    pub(super) rounds: RoundsOutcome,
}

fn run_partition_cell(specification: CellSpecification) -> PartitionCellResult {
    let model = Model::new(specification);
    let enumeration = enumerate_schedules(&model, should_memoize(&specification));
    let rounds = simulate_rounds(&model);
    PartitionCellResult { specification, aggregate: enumeration.aggregate, rounds }
}

fn arm_cell_specifications(device_count: usize) -> Vec<CellSpecification> {
    let mut specifications = Vec::new();
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            for shape_kind in [ShapeKind::SameShape, ShapeKind::Disjoint] {
                for initial_has_file in [true, false] {
                    for transaction_count in [2usize, 3] {
                        specifications.push(specification_of(
                            arm,
                            ArmVariant::Normal,
                            stage,
                            shape_kind,
                            initial_has_file,
                            transaction_count,
                            device_count,
                        ));
                    }
                }
            }
        }
    }
    specifications
}

fn control_cell_specifications(device_count: usize) -> Vec<CellSpecification> {
    let mut specifications = Vec::new();
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            for initial_has_file in [true, false] {
                for transaction_count in [2usize, 3] {
                    specifications.push(specification_of(
                        arm,
                        ArmVariant::PositiveControl,
                        stage,
                        ShapeKind::SameShape,
                        initial_has_file,
                        transaction_count,
                        device_count,
                    ));
                }
            }
        }
    }
    specifications
}

// ── 判定行 ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Question {
    UserDataSlots,
    CommitGeneratedSlots,
    InodeNumbers,
    TreeIdentifiers,
    TransactionNumbers,
    JsnAndJournalSlots,
    CheckpointTxgAndRootRingSlots,
    ConflictRetries,
    SynchronousRounds,
}

const DUPLICATE_QUESTIONS: [Question; 7] = [
    Question::UserDataSlots,
    Question::CommitGeneratedSlots,
    Question::InodeNumbers,
    Question::TreeIdentifiers,
    Question::TransactionNumbers,
    Question::JsnAndJournalSlots,
    Question::CheckpointTxgAndRootRingSlots,
];

impl Question {
    pub(super) fn label(self) -> &'static str {
        match self {
            Question::UserDataSlots => "Q-1",
            Question::CommitGeneratedSlots => "Q-2",
            Question::InodeNumbers => "Q-3",
            Question::TreeIdentifiers => "Q-4",
            Question::TransactionNumbers => "Q-5",
            Question::JsnAndJournalSlots => "Q-6",
            Question::CheckpointTxgAndRootRingSlots => "Q-7",
            Question::ConflictRetries => "Q-8",
            Question::SynchronousRounds => "Q-9",
        }
    }

    fn resources(self) -> Vec<Resource> {
        match self {
            Question::UserDataSlots => vec![Resource::UserDataSlot],
            Question::CommitGeneratedSlots => vec![Resource::CommitGeneratedSlot],
            Question::InodeNumbers => vec![Resource::InodeNumber],
            Question::TreeIdentifiers => vec![Resource::TreeIdentifier],
            Question::TransactionNumbers => vec![Resource::TransactionNumber],
            Question::JsnAndJournalSlots => vec![Resource::JsnCounter, Resource::JournalRingSlot],
            Question::CheckpointTxgAndRootRingSlots => vec![Resource::CheckpointTxg, Resource::RootRingSlot],
            Question::ConflictRetries | Question::SynchronousRounds => Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Judgement {
    /// 登记第六节最后一列没列：这一格的结果不能从那条臂的定义直接推出，拿来判。
    Judged,
    /// 登记第六节最后一列列了：能从定义直接推出，照报、当锚点，不判。
    DefinitionImplied,
    /// 登记第四节丙：跑前已经算出整格。
    KnownBeforeRun,
    NotReported,
}

impl Judgement {
    pub(super) fn label(self) -> &'static str {
        match self {
            Judgement::Judged => "judged",
            Judgement::DefinitionImplied => "definition_implied",
            Judgement::KnownBeforeRun => "known_before_run",
            Judgement::NotReported => "not_reported",
        }
    }
}

/// 登记第六节表最后一列「定义推出、不判的格」逐行落成代码。
fn judgement_of(question: Question, arm: Arm) -> Judgement {
    match (question, arm) {
        (Question::UserDataSlots, Arm::DevicePartition) => Judgement::KnownBeforeRun,
        (Question::UserDataSlots, Arm::LiteralLock) => Judgement::Judged,
        (Question::UserDataSlots, Arm::WholeLock)
        | (Question::UserDataSlots, Arm::RegionPartition)
        | (Question::UserDataSlots, Arm::OptimisticWholeVersion)
        | (Question::UserDataSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::CommitGeneratedSlots, Arm::LiteralLock) | (Question::CommitGeneratedSlots, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::CommitGeneratedSlots, Arm::WholeLock)
        | (Question::CommitGeneratedSlots, Arm::RegionPartition)
        | (Question::CommitGeneratedSlots, Arm::OptimisticWholeVersion)
        | (Question::CommitGeneratedSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::InodeNumbers, Arm::LiteralLock) | (Question::InodeNumbers, Arm::DevicePartition) => Judgement::Judged,
        (Question::InodeNumbers, Arm::WholeLock)
        | (Question::InodeNumbers, Arm::RegionPartition)
        | (Question::InodeNumbers, Arm::OptimisticWholeVersion)
        | (Question::InodeNumbers, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::TreeIdentifiers, Arm::LiteralLock)
        | (Question::TreeIdentifiers, Arm::RegionPartition)
        | (Question::TreeIdentifiers, Arm::DevicePartition) => Judgement::Judged,
        (Question::TreeIdentifiers, Arm::WholeLock)
        | (Question::TreeIdentifiers, Arm::OptimisticWholeVersion)
        | (Question::TreeIdentifiers, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::TransactionNumbers, Arm::LiteralLock) | (Question::TransactionNumbers, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::TransactionNumbers, Arm::WholeLock)
        | (Question::TransactionNumbers, Arm::RegionPartition)
        | (Question::TransactionNumbers, Arm::OptimisticWholeVersion)
        | (Question::TransactionNumbers, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::JsnAndJournalSlots, Arm::LiteralLock) | (Question::JsnAndJournalSlots, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::JsnAndJournalSlots, Arm::WholeLock)
        | (Question::JsnAndJournalSlots, Arm::RegionPartition)
        | (Question::JsnAndJournalSlots, Arm::OptimisticWholeVersion)
        | (Question::JsnAndJournalSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::CheckpointTxgAndRootRingSlots, Arm::LiteralLock)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::DevicePartition) => Judgement::Judged,
        (Question::CheckpointTxgAndRootRingSlots, Arm::WholeLock)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::RegionPartition)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::OptimisticWholeVersion)
        | (Question::CheckpointTxgAndRootRingSlots, Arm::OptimisticReadSet) => Judgement::DefinitionImplied,
        (Question::ConflictRetries, Arm::LiteralLock) | (Question::ConflictRetries, Arm::OptimisticReadSet) => {
            Judgement::Judged
        }
        (Question::ConflictRetries, Arm::DevicePartition) => Judgement::KnownBeforeRun,
        (Question::ConflictRetries, Arm::WholeLock)
        | (Question::ConflictRetries, Arm::RegionPartition)
        | (Question::ConflictRetries, Arm::OptimisticWholeVersion) => Judgement::DefinitionImplied,
        (Question::SynchronousRounds, Arm::OptimisticReadSet) | (Question::SynchronousRounds, Arm::DevicePartition) => {
            Judgement::Judged
        }
        (Question::SynchronousRounds, Arm::OptimisticWholeVersion) | (Question::SynchronousRounds, Arm::RegionPartition) => {
            Judgement::DefinitionImplied
        }
        (Question::SynchronousRounds, Arm::WholeLock) | (Question::SynchronousRounds, Arm::LiteralLock) => {
            Judgement::NotReported
        }
    }
}

fn cells_of(
    results: &[PartitionCellResult],
    arm: Arm,
    stage: ParallelStage,
) -> Vec<&PartitionCellResult> {
    results.iter().filter(|result| result.specification.arm == arm && result.specification.stage == stage).collect()
}

fn duplicates_of_question(result: &PartitionCellResult, question: Question) -> u128 {
    question
        .resources()
        .iter()
        .map(|resource| result.aggregate.duplicate_total[resource.index()])
        .sum()
}

fn emit_question_verdicts(emitter: &mut Emitter, default_cells: &[PartitionCellResult]) {
    for question in DUPLICATE_QUESTIONS {
        for arm in ALL_ARMS {
            for stage in ALL_STAGES {
                let cells = cells_of(default_cells, arm, stage);
                let cells_with_duplicates =
                    cells.iter().filter(|result| duplicates_of_question(result, question) > 0).count();
                let duplicate_total: u128 = cells.iter().map(|result| duplicates_of_question(result, question)).sum();
                emit(
                    emitter,
                    &format!(
                        "name=verdict question={} arm={} stage={} judgement={} cells={} cells_with_duplicates={} \
                         duplicate_total={} no_duplicate_in_any_cell={}",
                        question.label(),
                        arm.label(ArmVariant::Normal),
                        stage.label(),
                        judgement_of(question, arm).label(),
                        cells.len(),
                        cells_with_duplicates,
                        duplicate_total,
                        cells_with_duplicates == 0
                    ),
                );
            }
        }
    }
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            let cells = cells_of(default_cells, arm, stage);
            let retries_total: u128 = cells.iter().map(|result| result.aggregate.retries_total).sum();
            let cells_with_retries = cells.iter().filter(|result| result.aggregate.retries_total > 0).count();
            let truncated: u128 = cells.iter().map(|result| result.aggregate.truncated).sum();
            let fallbacks: u128 = cells.iter().map(|result| result.aggregate.fallbacks_total).sum();
            let deadlocks: u128 = cells.iter().map(|result| result.aggregate.deadlocks).sum();
            emit(
                emitter,
                &format!(
                    "name=verdict question=Q-8 arm={} stage={} judgement={} cells={} retries_total={} cells_with_retries={} \
                     device_truncated={} region_fallbacks={} region_deadlocks={}",
                    arm.label(ArmVariant::Normal),
                    stage.label(),
                    judgement_of(Question::ConflictRetries, arm).label(),
                    cells.len(),
                    retries_total,
                    cells_with_retries,
                    truncated,
                    fallbacks,
                    deadlocks
                ),
            );
        }
    }
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            let cells = cells_of(default_cells, arm, stage);
            let judgement = judgement_of(Question::SynchronousRounds, arm);
            if judgement == Judgement::NotReported {
                emit(
                    emitter,
                    &format!(
                        "name=verdict question=Q-9 arm={} stage={} judgement={} cells={} cells_with_fewer_rounds_than_serial=not_reported",
                        arm.label(ArmVariant::Normal),
                        stage.label(),
                        judgement.label(),
                        cells.len()
                    ),
                );
                continue;
            }
            let fewer_rounds = cells
                .iter()
                .filter(|result| {
                    result
                        .rounds
                        .rounds
                        .is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") < result.specification.transaction_count)
                })
                .count();
            let without_rounds = cells.iter().filter(|result| result.rounds.rounds.is_none()).count();
            emit(
                emitter,
                &format!(
                    "name=verdict question=Q-9 arm={} stage={} judgement={} cells={} cells_with_fewer_rounds_than_serial={} \
                     cells_without_round_count={} rounds_equal_serial_in_every_cell={}",
                    arm.label(ArmVariant::Normal),
                    stage.label(),
                    judgement.label(),
                    cells.len(),
                    fewer_rounds,
                    without_rounds,
                    fewer_rounds == 0 && without_rounds == 0
                ),
            );
        }
    }
}

fn find_cell<'results>(
    results: &'results [PartitionCellResult],
    specification: &CellSpecification,
) -> Option<&'results PartitionCellResult> {
    results.iter().find(|result| {
        result.specification.arm == specification.arm
            && result.specification.variant == specification.variant
            && result.specification.stage == specification.stage
            && result.specification.shape_kind == specification.shape_kind
            && result.specification.initial_has_file == specification.initial_has_file
            && result.specification.transaction_count == specification.transaction_count
    })
}

/// 登记第十节失败条款 1–9 逐条按产物里的格算一次（`triggered` = 条款的前件满足）。
fn emit_failure_clauses(emitter: &mut Emitter, default_cells: &[PartitionCellResult]) {
    let nonzero_duplicates = |result: &PartitionCellResult| result.aggregate.duplicate_total.iter().any(|total| *total > 0);
    let whole_lock_default_duplicates = default_cells
        .iter()
        .filter(|result| result.specification.arm == Arm::WholeLock)
        .filter(|result| nonzero_duplicates(result))
        .count();
    emit(
        emitter,
        &format!("name=verdict failure=2 clause=A_default_geometry_duplicate triggered={}", whole_lock_default_duplicates > 0),
    );
    let all_duplicates_zero = default_cells.iter().all(|result| !nonzero_duplicates(result));
    let read_set_equals_whole_everywhere = default_cells
        .iter()
        .filter(|result| result.specification.arm == Arm::OptimisticReadSet)
        .all(|read_set_cell| {
            let whole_version_specification = CellSpecification { arm: Arm::OptimisticWholeVersion, ..read_set_cell.specification };
            find_cell(default_cells, &whole_version_specification)
                .is_some_and(|whole_cell| whole_cell.aggregate.retries_total == read_set_cell.aggregate.retries_total)
        });
    emit(
        emitter,
        &format!(
            "name=verdict failure=4 clause=ruler_cannot_separate_candidates triggered={}",
            all_duplicates_zero && read_set_equals_whole_everywhere
        ),
    );
    let mut read_set_differs_in_stage_one_or_three = 0;
    let mut read_set_below_whole_in_stage_two_same_shape = 0;
    for read_set_cell in default_cells.iter().filter(|result| result.specification.arm == Arm::OptimisticReadSet) {
        let whole_version_specification = CellSpecification { arm: Arm::OptimisticWholeVersion, ..read_set_cell.specification };
        let whole_cell = find_cell(default_cells, &whole_version_specification).expect("每个 O读 格都有同参数的 O整 格");
        let stage_uses_base_version = read_set_cell.specification.stage.base_version_in_read_set();
        if stage_uses_base_version && whole_cell.aggregate.retries_total != read_set_cell.aggregate.retries_total {
            read_set_differs_in_stage_one_or_three += 1;
        }
        if read_set_cell.specification.stage == ParallelStage::OnePublishCarriesManyTransactions
            && read_set_cell.specification.shape_kind == ShapeKind::SameShape
            && read_set_cell.aggregate.retries_total < whole_cell.aggregate.retries_total
        {
            read_set_below_whole_in_stage_two_same_shape += 1;
        }
    }
    emit(
        emitter,
        &format!(
            "name=verdict failure=5 clause=read_set_not_as_registered triggered={}",
            read_set_differs_in_stage_one_or_three > 0
        ),
    );
    let device_cells_with_truncation = default_cells
        .iter()
        .filter(|result| result.specification.arm == Arm::DevicePartition)
        .filter(|result| result.aggregate.truncated > 0)
        .count();
    emit(
        emitter,
        &format!(
            "name=verdict failure=6 clause=device_partition_does_not_converge triggered={} cells_with_truncation={}",
            device_cells_with_truncation > 0,
            device_cells_with_truncation
        ),
    );
    let literal_lock_cells_with_tree_or_transaction_duplicates = default_cells
        .iter()
        .filter(|result| result.specification.arm == Arm::LiteralLock)
        .filter(|result| {
            result.aggregate.duplicate_total[Resource::TreeIdentifier.index()] > 0
                || result.aggregate.duplicate_total[Resource::TransactionNumber.index()] > 0
        })
        .count();
    emit(
        emitter,
        &format!(
            "name=verdict failure=7 clause=literal_lock_misses_shared_state triggered={} cells_with_x_or_t_duplicates={}",
            literal_lock_cells_with_tree_or_transaction_duplicates > 0,
            literal_lock_cells_with_tree_or_transaction_duplicates
        ),
    );
    let mut stage_flipping_columns: Vec<String> = Vec::new();
    for arm in ALL_ARMS {
        for resource in ALL_RESOURCES {
            let positive_in_stage: Vec<bool> = ALL_STAGES
                .iter()
                .map(|stage| {
                    cells_of(default_cells, arm, *stage)
                        .iter()
                        .any(|result| result.aggregate.duplicate_total[resource.index()] > 0)
                })
                .collect();
            if positive_in_stage.iter().any(|positive| *positive) && positive_in_stage.iter().any(|positive| !*positive) {
                stage_flipping_columns.push(format!("{}:{}", arm.label(ArmVariant::Normal), resource.label()));
            }
        }
    }
    emit(
        emitter,
        &format!(
            "name=verdict failure=8 clause=judgement_flips_with_q1_candidate triggered={} flipping_columns={}",
            !stage_flipping_columns.is_empty(),
            if stage_flipping_columns.is_empty() { "none".to_string() } else { stage_flipping_columns.join(";") }
        ),
    );
    emit(
        emitter,
        &format!(
            "name=verdict failure=9 clause=read_set_below_whole_in_stage_two_same_shape triggered={}",
            read_set_below_whole_in_stage_two_same_shape > 0
        ),
    );
}

/// 第八节「几何敏感性」：D = 2（默认）与 D = 1（第一个反向取样点）上每格的判定逐格对一遍。
fn emit_geometry_sensitivity(emitter: &mut Emitter, default_cells: &[PartitionCellResult], sample_cells: &[PartitionCellResult]) {
    let mut unstable_groups: Vec<(String, u128, u128)> = Vec::new();
    for arm in ALL_ARMS {
        for stage in ALL_STAGES {
            let default_group = cells_of(default_cells, arm, stage);
            for question in DUPLICATE_QUESTIONS.into_iter().chain([Question::SynchronousRounds]) {
                if question == Question::SynchronousRounds && !reports_rounds(&default_group[0].specification) {
                    continue;
                }
                let mut differing = 0;
                let mut strongest_default = 0u128;
                let mut strongest_sample = 0u128;
                for default_cell in &default_group {
                    let sample_cell = find_cell(sample_cells, &default_cell.specification)
                        .expect("每个默认格都有 D = 1 的同参数格");
                    let sample_aggregate = &sample_cell.aggregate;
                    let sample_rounds = &sample_cell.rounds;
                    let (default_judgement, sample_judgement, default_value, sample_value) =
                        if question == Question::SynchronousRounds {
                            let serial = default_cell.specification.transaction_count;
                            let default_equal = default_cell.rounds.rounds.is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") == serial);
                            let sample_equal = sample_rounds.rounds.is_some_and(|rounds| usize::try_from(rounds).expect("轮数很小") == serial);
                            (
                                default_equal,
                                sample_equal,
                                u128::from(default_cell.rounds.rounds.unwrap_or(0)),
                                u128::from(sample_rounds.rounds.unwrap_or(0)),
                            )
                        } else {
                            let default_total = duplicates_of_question(default_cell, question);
                            let sample_total: u128 = question
                                .resources()
                                .iter()
                                .map(|resource| sample_aggregate.duplicate_total[resource.index()])
                                .sum();
                            (default_total == 0, sample_total == 0, default_total, sample_total)
                        };
                    if default_judgement != sample_judgement {
                        differing += 1;
                        strongest_default = strongest_default.max(default_value);
                        strongest_sample = strongest_sample.max(sample_value);
                    }
                }
                if differing > 0 {
                    unstable_groups.push((
                        format!("{}:{}:{}", arm.label(ArmVariant::Normal), stage.label(), question.label()),
                        strongest_default,
                        strongest_sample,
                    ));
                }
                emit(
                    emitter,
                    &format!(
                        "name=geometry_sensitivity arm={} stage={} question={} cells={} cells_with_different_judgement={} outcome={}",
                        arm.label(ArmVariant::Normal),
                        stage.label(),
                        question.label(),
                        default_group.len(),
                        differing,
                        if differing == 0 { "two_points_same" } else { "unstable" }
                    ),
                );
            }
        }
    }
    if unstable_groups.is_empty() {
        emit(emitter, "name=discrimination_selfcheck status=not_needed_every_cell_two_points_same");
    }
    for (group, default_value, sample_value) in unstable_groups {
        // 把门槛挪到两点的值之间：一边绿一边红。
        let threshold_twice = default_value + sample_value;
        let default_is_above = default_value * 2 > threshold_twice;
        let sample_is_above = sample_value * 2 > threshold_twice;
        emit(
            emitter,
            &format!(
                "name=discrimination_selfcheck group={group} default_value={default_value} sample_value={sample_value} \
                 threshold_between_flips_judgement={}",
                default_is_above != sample_is_above
            ),
        );
    }
}

pub(super) fn emit_segment_one(emitter: &mut Emitter) {
    emit(
        emitter,
        &format!(
            "name=config experiment=E166 segment=1 default_devices={DEFAULT_DEVICE_COUNT} sample_devices={SAMPLE_DEVICE_COUNT} \
             user_data_pairs={USER_DATA_PAIR_COUNT} commit_pool_slots={COMMIT_GENERATED_POOL_SLOTS} \
             root_slots_per_region={DEFAULT_ROOT_SLOTS_PER_REGION} journal_ring_slots={DEFAULT_JOURNAL_RING_SLOTS} \
             device_disagreement_retry_cap={DEVICE_DISAGREEMENT_RETRY_CAP} regions=transactions inode_reservation=exact \
             shapes=same,disjoint initial=file,nofile transactions=2,3 stages=Q1甲,Q1乙,Q1丙"
        ),
    );
    let anchor_lines = all_anchor_lines();
    let mismatched = anchor_lines.iter().filter(|line| !line.matches()).count();
    for line in &anchor_lines {
        emit(
            emitter,
            &format!(
                "name=anchor id=\"{}\" observed=\"{}\" expected=\"{}\" ok={}",
                line.identifier,
                line.observed,
                line.expected,
                line.matches()
            ),
        );
    }
    emit(
        emitter,
        &format!(
            "name=verdict anchors_checked={} anchors_mismatched={} anchors_all_ok={}",
            anchor_lines.len(),
            mismatched,
            mismatched == 0
        ),
    );
    if mismatched > 0 {
        println!("{}", emitter.finish());
        std::process::exit(1);
    }
    for device_count in [DEFAULT_DEVICE_COUNT, SAMPLE_DEVICE_COUNT] {
        for specification in control_cell_specifications(device_count) {
            let result = run_partition_cell(specification);
            emit(emitter, &cell_line("control_cell", &result.specification, &result.aggregate));
        }
    }
    let mut default_cells: Vec<PartitionCellResult> = Vec::new();
    let mut sample_cells: Vec<PartitionCellResult> = Vec::new();
    for device_count in [DEFAULT_DEVICE_COUNT, SAMPLE_DEVICE_COUNT] {
        for specification in arm_cell_specifications(device_count) {
            let result = run_partition_cell(specification);
            emit(emitter, &cell_line("cell", &result.specification, &result.aggregate));
            emit(emitter, &trajectory_line(&result.specification, &result.aggregate));
            if reports_rounds(&result.specification) {
                emit(emitter, &rounds_line(&result.specification, &result.rounds));
            }
            if device_count == DEFAULT_DEVICE_COUNT {
                default_cells.push(result);
            } else {
                sample_cells.push(result);
            }
        }
    }
    emit_question_verdicts(emitter, &default_cells);
    emit_failure_clauses(emitter, &default_cells);
    emit_geometry_sensitivity(emitter, &default_cells, &sample_cells);
    println!("{}", emitter.finish());
}
