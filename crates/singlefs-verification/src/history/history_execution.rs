//! 逐步执行一段历史：交出每一步的镜像、跑到池级 checker 的违例为止，连同根环转没转、抬 F 落没落在被弃的根上这几样判定。

use super::*;

/// 下一张交出去的镜像在交出去的次序里的下标（从 0 数）：已经交了几张就是几。
fn step_image_index_of_the_next_hand_out(tally: &HistoryTally) -> usize {
    usize::try_from(tally.step_images_handed_out)
        .expect("一段历史交出去的镜像张数不超过步数加一，装得进 usize")
}

/// 把一张池级 checker 本来要判的镜像交给每步镜像那个钩子，记一张。
fn hand_out_a_step_image(
    step_images: &mut dyn FnMut(&StepImage<'_>),
    tally: &mut HistoryTally,
    step_image: &StepImage<'_>,
) {
    tally.step_images_handed_out += 1;
    step_images(step_image);
}

/// 跑一段历史：在 `execution.device_width` 那么宽的两块盘上，起点之后与每一步写过盘之后把那一刻的镜像交给观察者（harness 不跑池级 checker，
/// [`PerStepChecker`]）。第一次失败（执行器判出的、模型对不上、panic）就停。盘上的写与屏障录进 `stream`（崩溃注入给开了内容保留的流）。
pub fn execute_history_with(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    execute_history_with_faults(
        history,
        execution,
        stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        observer,
    )
}

/// 同 [`execute_history_with`]，再按调用方给的注入计划在读 / 写 / 刷盘上注入故障（增补 3 第 4 件）。
/// 计划不开时与 [`execute_history_with`] 逐字节相同：注入层只数调用，每一次都原样交给录制器。
pub fn execute_history_with_faults(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    fault_plan: &SharedFaultPlan,
    observer: &mut dyn FnMut(&StepObservation<'_>),
) -> HistoryRun {
    execute_history_handing_out_every_step_image(
        history,
        execution,
        stream,
        fault_plan,
        observer,
        &mut |_| {},
    )
}

/// 经每步镜像那个钩子交出去的一张镜像：池级 checker 本来在这一刻判的那一张（起点之后、每一步写过盘之后，起点段报错而 mkfs 做完了的那一张）。
/// 给定状态引擎把它作为给定状态写进判定库：录制流开了内容保留时，这张镜像就是空盘叠上录制流头 `operations_recorded` 步（录制流只记落了盘的写）。
pub struct StepImage<'step> {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    /// 这一刻录制流里有几步（写与屏障；[`SharedStream::operation_count`]）。
    pub operations_recorded: usize,
    pub image: &'step MemoryPool,
    /// 到这一刻为止实现交回了几份隔离（[`HistoryRun::copies_quarantined_by_the_implementation`] 的头这么多份；这一步自己交回的在里面）。
    pub copies_quarantined_by_the_implementation_up_to_this_step: usize,
}

/// 同 [`execute_history_with_faults`]，另把池级 checker 本来要判的每一张镜像交给 `step_images`（harness 一条不变量都不判，
/// 历史只由模型、执行器的判定与 panic 停下）。交出去的次序就是历史里的次序，历史停下的那一步写过盘也交（那一步不调观察者）。
pub fn execute_history_handing_out_every_step_image(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    fault_plan: &SharedFaultPlan,
    observer: &mut dyn FnMut(&StepObservation<'_>),
    step_images: &mut dyn FnMut(&StepImage<'_>),
) -> HistoryRun {
    execute_history_stopping_at_the_given_pool_checker_violations(
        history,
        execution,
        stream,
        fault_plan,
        observer,
        step_images,
        &mut |_| Vec::new(),
    )
}

/// 同 [`execute_history_handing_out_every_step_image`]，另由调用方给每一张交出去的镜像上池级 checker 判红的那几条
/// （`given_pool_checker_violations` 收那一张在交出去的次序里的下标，从 0 数）：给的不空，历史就停在那一步、那几条违例进停下那一步的观察，
/// 与 harness 就地判、判红就停同一个口径（池级 checker 归给定状态引擎两段）。只给故障注入的合并入口
/// （`crate::fault_injection::inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`）用：它把给定状态引擎
/// 判回的每步镜像结论交进来，截短重跑那一遍。
pub(crate) fn execute_history_stopping_at_the_given_pool_checker_violations(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    stream: &SharedStream,
    fault_plan: &SharedFaultPlan,
    observer: &mut dyn FnMut(&StepObservation<'_>),
    step_images: &mut dyn FnMut(&StepImage<'_>),
    given_pool_checker_violations: &mut dyn FnMut(usize) -> Vec<(&'static str, String)>,
) -> HistoryRun {
    let mut tally = HistoryTally::default();
    let mut outcomes: Vec<StepOutcome> = Vec::new();
    let mut pool_slot: Option<HistoryPool> = None;
    let position = Cell::new(StepPosition::StartingPoint);
    // 起点段报错（会话没建起来）时 mkfs 写了几步：给定状态引擎把交出去的镜像建成给定状态表时，mkfs 那几步进基线
    let operations_written_by_make_filesystem_before_the_start_failed: Cell<Option<usize>> =
        Cell::new(None);
    let mut completed_after_the_root_ring_turned = false;
    let body_result = with_panic_capture(|| -> Option<FailureObservation> {
        let started = HistoryPool::start(
            history.starting_point,
            execution.device_width,
            execution.space_admission,
            stream,
            fault_plan,
        );
        let (started_pool, starting_verdict) = match started {
            Ok(started) => started,
            Err(failure) => {
                operations_written_by_make_filesystem_before_the_start_failed
                    .set(Some(failure.operations_written_by_make_filesystem));
                // 起点段有一个入口返回了错误（只有故障注入把注入点摆进起点段时走得到）。mkfs 自己就报错的那一格不交镜像、
                // 也没有池级 checker 的结论：盘上没有一个做完了的文件系统，checker 判的每一条都没有对象。
                let violations = if failure.step.finished_filesystem_is_on_the_devices() {
                    let step_image_index = step_image_index_of_the_next_hand_out(&tally);
                    hand_out_a_step_image(
                        step_images,
                        &mut tally,
                        &StepImage {
                            position: StepPosition::StartingPoint,
                            operation_kind: None,
                            operations_recorded: stream.operation_count(),
                            image: &failure.image,
                            copies_quarantined_by_the_implementation_up_to_this_step: 0,
                        },
                    );
                    given_pool_checker_violations(step_image_index)
                } else {
                    Vec::new()
                };
                let mut observation = failure_observation(
                    &failure.image,
                    StepPosition::StartingPoint,
                    None,
                    violations,
                    None,
                );
                observation.harness_judgement =
                    Some(HarnessJudgement::TheStartingPointReturnedAnError {
                        step: failure.step,
                        error: failure.error,
                    });
                return Some(observation);
            }
        };
        let pool = pool_slot.insert(started_pool);
        let mut roots_written_by_the_model_before_this_step =
            measure_the_roots_the_model_wrote_in_one_step(
                &mut tally,
                &pool.model,
                0,
                StepPosition::StartingPoint,
            );
        let mut image = pool.image();
        let mut checked_stream_length = stream.operation_count();
        let starting_point_step_image_index = step_image_index_of_the_next_hand_out(&tally);
        hand_out_a_step_image(
            step_images,
            &mut tally,
            &StepImage {
                position: StepPosition::StartingPoint,
                operation_kind: None,
                operations_recorded: checked_stream_length,
                image: &image,
                copies_quarantined_by_the_implementation_up_to_this_step: pool
                    .copies_quarantined_by_the_implementation
                    .len(),
            },
        );
        let violations_after_the_starting_point =
            given_pool_checker_violations(starting_point_step_image_index);
        if let Some(verdict) = &starting_verdict {
            tally.note_model_verdict(verdict);
        }
        let starting_disagreement = starting_verdict.and_then(|verdict| verdict.disagreement);
        // 起点判红一律停。
        if !violations_after_the_starting_point.is_empty() || starting_disagreement.is_some() {
            let mut observation = failure_observation(
                &image,
                StepPosition::StartingPoint,
                None,
                violations_after_the_starting_point,
                None,
            );
            observation.model_disagreement = starting_disagreement;
            return Some(observation);
        }
        observer(&StepObservation {
            position: StepPosition::StartingPoint,
            operation: None,
            outcome: None,
            image: &image,
            model: &pool.model,
            free_bytes_reported_by_the_current_version: free_bytes_reported_by_the_current_version(
                pool.session.as_ref(),
            ),
            current_version_of_trees: pool
                .session
                .as_ref()
                .and_then(|session| session.current.version_of_trees()),
            size_requested_by_the_truncation: None,
            inode_named_by_the_truncation: None,
            partial_write_requested: None,
        });
        for (step_index, operation) in history.operations.iter().enumerate() {
            let step_position = StepPosition::Operation(step_index);
            position.set(step_position);
            let AppliedStep {
                outcome,
                harness_judgement,
                model_verdict,
            } = apply_operation(pool, operation, step_index);
            pool.note_the_copies_quarantined_in_the_step(step_position);
            roots_written_by_the_model_before_this_step =
                measure_the_roots_the_model_wrote_in_one_step(
                    &mut tally,
                    &pool.model,
                    roots_written_by_the_model_before_this_step,
                    step_position,
                );
            tally.note_outcome(operation, &outcome);
            if let Some(requested) = pool.partial_writes_requested.get(&step_index) {
                tally.note_partial_write_offset_cell(requested.offset_cell, &outcome);
            }
            if let Some(verdict) = &model_verdict {
                tally.note_model_verdict(verdict);
            }
            let model_disagreement = model_verdict.and_then(|verdict| verdict.disagreement);
            if let Some(session) = &pool.session {
                tally.most_publishes_in_one_mount = tally
                    .most_publishes_in_one_mount
                    .max(session.publishes_in_this_mount);
                tally.highest_checkpoint_txg = tally
                    .highest_checkpoint_txg
                    .max(session.current.root().checkpoint_txg.0);
                tally.most_allocation_records_in_one_version = tally
                    .most_allocation_records_in_one_version
                    .max(u64::try_from(session.allocator.records().len()).expect("条数装得进 u64"));
            }
            let raised_floor = if let StepOutcome::Applied(AppliedEffect::RaisedFloor {
                new_floor,
                ..
            }) = &outcome
            {
                Some(*new_floor)
            } else {
                None
            };
            outcomes.push(outcome);
            let stream_length = stream.operation_count();
            let mut violations = Vec::new();
            let mut raised_floor_lands_only_on_abandoned = None;
            if stream_length == checked_stream_length {
                tally.checker_runs_skipped_because_nothing_was_written += 1;
            } else {
                let image_before_this_step = std::mem::replace(&mut image, pool.image());
                checked_stream_length = stream_length;
                let step_image_index = step_image_index_of_the_next_hand_out(&tally);
                hand_out_a_step_image(
                    step_images,
                    &mut tally,
                    &StepImage {
                        position: step_position,
                        operation_kind: Some(operation.kind()),
                        operations_recorded: stream_length,
                        image: &image,
                        copies_quarantined_by_the_implementation_up_to_this_step: pool
                            .copies_quarantined_by_the_implementation
                            .len(),
                    },
                );
                violations = given_pool_checker_violations(step_image_index);
                if !violations.is_empty() {
                    raised_floor_lands_only_on_abandoned = raised_floor.and_then(|new_floor| {
                        raised_floor_lands_only_on_abandoned_roots(
                            &image_before_this_step,
                            new_floor,
                        )
                    });
                }
            }
            if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()
            {
                let mut observation = failure_observation(
                    &image,
                    step_position,
                    Some(operation.kind()),
                    violations,
                    None,
                );
                observation.harness_judgement = harness_judgement;
                observation.model_disagreement = model_disagreement;
                observation.raised_floor_lands_only_on_abandoned_roots =
                    raised_floor_lands_only_on_abandoned;
                return Some(observation);
            }
            observer(&StepObservation {
                position: step_position,
                operation: Some(operation),
                outcome: outcomes.last(),
                image: &image,
                model: &pool.model,
                free_bytes_reported_by_the_current_version:
                    free_bytes_reported_by_the_current_version(pool.session.as_ref()),
                current_version_of_trees: pool
                    .session
                    .as_ref()
                    .and_then(|session| session.current.version_of_trees()),
                size_requested_by_the_truncation: pool
                    .sizes_requested_by_the_truncations
                    .get(&step_index)
                    .copied(),
                inode_named_by_the_truncation: pool
                    .inodes_named_by_the_truncations
                    .get(&step_index)
                    .copied(),
                partial_write_requested: pool.partial_writes_requested.get(&step_index).copied(),
            });
        }
        let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(&image);
        completed_after_the_root_ring_turned =
            root_ring_has_turned(newest_ring_root_txg, root_ring_slot_count);
        None
    });
    let failure = match body_result {
        Ok(failure) => failure,
        Err(panic) => {
            let step_position = position.get();
            let operation_kind = match step_position {
                StepPosition::StartingPoint => None,
                StepPosition::Operation(step_index) => history
                    .operations
                    .get(step_index)
                    .map(HistoryOperation::kind),
            };
            // panic 之后的盘面照样读一遍根环（读本身也包在捕获里：坏到 checker 的解析也 panic 时只丢掉这两个事实）。
            let (newest_ring_root_txg, root_ring_slot_count) = pool_slot
                .as_ref()
                .and_then(|pool| {
                    with_panic_capture(|| newest_ring_root_and_slot_count(&pool.image())).ok()
                })
                .unwrap_or((None, None));
            Some(FailureObservation {
                position: step_position,
                operation_kind,
                violations: Vec::new(),
                panic: Some(panic),
                newest_ring_root_txg,
                root_ring_slot_count,
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            })
        }
    };
    if let Some(pool) = &pool_slot {
        tally.most_successful_mounts_in_one_history = pool.successful_mounts;
        tally.most_mount_attempts_in_one_history = pool.mount_attempts;
    }
    let ending = match failure {
        None => {
            tally.histories_completed = 1;
            if completed_after_the_root_ring_turned {
                tally.histories_that_turned_the_root_ring = 1;
            }
            HistoryEnding::Completed
        }
        Some(observation) => {
            if observation.root_ring_has_turned() {
                tally.histories_that_turned_the_root_ring = 1;
            }
            let ending = classify_failure(observation);
            match &ending {
                HistoryEnding::KnownRed { form, .. } => {
                    tally.histories_ended_known_red.insert(*form, 1);
                }
                HistoryEnding::NewFinding { .. } => tally.histories_ended_new_finding = 1,
                HistoryEnding::Completed => {}
            }
            ending
        }
    };
    tally.histories = 1;
    tally.histories_with_a_deletion_batch_keeping_data_units = u64::from(
        tally
            .model_counts
            .deletion_batches_keeping_data_units_matched
            > 0,
    );
    tally.histories_with_a_walk_of_several_steps_before_a_user_change = u64::from(
        tally
            .model_counts
            .walks_of_several_steps_before_a_user_change_matched
            > 0,
    );
    tally.histories_with_a_walk_before_a_user_change_refused_for_space = u64::from(
        tally
            .model_counts
            .walks_before_a_user_change_refused_for_space_matched
            > 0,
    );
    HistoryRun {
        ending,
        outcomes,
        tally,
        // mkfs 在 `HistoryPool::start` 里跑：起点段报错时（`pool_slot` 没填）报那一刻 mkfs 写了的步数；它自己 panic 时流里还没有别的东西，报 0。
        operations_written_by_make_filesystem: pool_slot.as_ref().map_or_else(
            || {
                operations_written_by_make_filesystem_before_the_start_failed
                    .get()
                    .unwrap_or(0)
            },
            |pool| pool.operations_written_by_make_filesystem,
        ),
        committed_versions_at_the_end: pool_slot.as_ref().map_or_else(Vec::new, |pool| {
            pool.model
                .committed_versions()
                .into_iter()
                .map(|(key, content)| (key, content.map(|content| content.to_vec())))
                .collect()
        }),
        earliest_stream_index_after_each_power_loss: pool_slot
            .as_ref()
            .map_or_else(Vec::new, |pool| {
                pool.earliest_stream_index_after_each_power_loss.clone()
            }),
        sizes_requested_by_the_truncations: pool_slot.as_ref().map_or_else(BTreeMap::new, |pool| {
            pool.sizes_requested_by_the_truncations.clone()
        }),
        inodes_named_by_the_truncations: pool_slot.as_ref().map_or_else(BTreeMap::new, |pool| {
            pool.inodes_named_by_the_truncations.clone()
        }),
        partial_writes_requested: pool_slot
            .as_ref()
            .map_or_else(BTreeMap::new, |pool| pool.partial_writes_requested.clone()),
        // panic 的那一步收进来、还没记上步号的那几份，记在 panic 的那一步上
        copies_quarantined_by_the_implementation: pool_slot.map_or_else(Vec::new, |mut pool| {
            pool.note_the_copies_quarantined_in_the_step(position.get());
            pool.copies_quarantined_by_the_implementation
        }),
    }
}

fn failure_observation(
    image: &MemoryPool,
    position: StepPosition,
    operation_kind: Option<HistoryOperationKind>,
    violations: Vec<(&'static str, String)>,
    panic: Option<CapturedPanic>,
) -> FailureObservation {
    let (newest_ring_root_txg, root_ring_slot_count) = newest_ring_root_and_slot_count(image);
    FailureObservation {
        position,
        operation_kind,
        violations,
        panic,
        newest_ring_root_txg,
        root_ring_slot_count,
        harness_judgement: None,
        model_disagreement: None,
        raised_floor_lands_only_on_abandoned_roots: None,
        record_check: RecordCheck::default(),
    }
}

/// 抬 F 之前的镜像上，txg = `new_floor` 的根是不是全属于被抛弃的实例（F 落在回退留下的空档里）：按最新根指着的实例表，
/// 有行 (i, T) 且根的 txg > T 的 i 就是被抛弃的（与 `recovery::root_is_abandoned_by_the_instance_table` 同一句）。那个 txg 上一条根都没有、
/// 系统配置或最新根或它的实例表读不出，都是 None（不算落在空档里）。
/// 读法用的是实现的 `recovery::readable_roots` / `choose_root` / `instance_table_of_root`（代码三方攻方的 P1 就这么读）：
/// checker 判候选集时解实例表的那一段不对外（`singlefs-checker` 的 `walk.rs` 里 `Walk::instance_table_rows`），这里没另写一份解析。
#[must_use]
pub fn raised_floor_lands_only_on_abandoned_roots(
    image_before_raising: &dyn PoolReader,
    new_floor: CheckpointTxg,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image_before_raising).ok()?;
    let roots_at_floor: Vec<RootRecord> = readable_roots(
        image_before_raising,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.checkpoint_txg == new_floor)
    .collect();
    if roots_at_floor.is_empty() {
        return None;
    }
    let newest_root = choose_root(image_before_raising, &system_configuration)?;
    let newest_table = instance_table_of_root(image_before_raising, &newest_root)?;
    Some(roots_at_floor.iter().all(|root| {
        newest_table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    }))
}

/// 按 checker 的读法：根环里最新那条根的 txg 与一圈的槽数 R × S。
#[must_use]
pub fn newest_ring_root_and_slot_count(image: &MemoryPool) -> (Option<u64>, Option<u64>) {
    newest_ring_root_and_slot_count_of(image)
}

/// 同上，读的盘面由调用方给：崩溃注入（增补 3 第 3 件）要在崩溃镜像（`crash::CrashImage`）上也拿到这两个事实
/// （失败观察里的「根环转过一圈」照样报出来）。
#[must_use]
pub fn newest_ring_root_and_slot_count_of(image: &dyn ImageReader) -> (Option<u64>, Option<u64>) {
    let (roots, root_ring_slot_count) = ring_roots_newest_first_of(image);
    (roots.first().map(|(txg, _)| *txg), root_ring_slot_count)
}

/// 根环转过一圈：盘上最新根的 txg ≥ R × S，第 0 代根的槽已被盖过。
pub(super) fn root_ring_has_turned(
    newest_ring_root_txg: Option<u64>,
    root_ring_slot_count: Option<u64>,
) -> bool {
    matches!(
        (newest_ring_root_txg, root_ring_slot_count),
        (Some(newest), Some(slot_count)) if newest >= slot_count
    )
}
