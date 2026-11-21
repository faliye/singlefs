//! 删除（5.1 第 4、5 步）：删除的计划、逐步发布、抬 F 与谷值钩子。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 删除（5.1 第 4、5 步）
// ---------------------------------------------------------------------------------------------------------------------

/// 删除阶段回收开不开（阳性对照 PC-noreclaim 关）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReclaimDuringTheDelete {
    AsToday,
    SwitchedOff,
}

/// 一次删除发布（意图那次记 0）被拒的种类（登记 1.2「被落点那一判拒」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeleteRefusal {
    /// `PublishError::SpaceAdmissionRefused`：删除各批式子那一判关着（调用计数核过），只能来自落点那一判。
    Landing,
    /// `PublishError::PlacementRefused`。
    Placement,
    /// B 臂：推满抬 F 之后仍拒。
    NoSpaceAfterRaisingTheFloor,
}

impl DeleteRefusal {
    pub(super) fn label(self) -> &'static str {
        match self {
            DeleteRefusal::Landing => "landing",
            DeleteRefusal::Placement => "placement",
            DeleteRefusal::NoSpaceAfterRaisingTheFloor => "no_space_after_raising_the_floor",
        }
    }
}

/// 一次删除发布的读数（逐盘的量按盘的次序）。
#[derive(Clone, Debug)]
pub(super) struct DeletePublishReading {
    /// 第几次删除发布，意图那次记 0。
    pub(super) position: u64,
    kept_data_units: u64,
    pub(super) reserve_slots: Vec<i128>,
    pub(super) checkpoint_cost: u64,
    pub(super) free_before_slots: Vec<u64>,
    /// W_t；被拒的那一次是 0。
    pub(super) new_slots: Vec<u64>,
    /// 这一次里推的抬 F 空发布（B 臂）的发布数与它们新占的槽。
    pub(super) floor_raise_publishes: u64,
    pub(super) floor_raise_new_slots: Vec<u64>,
    /// 被拒的那一次里会话不再推抬 F 的原因（[`FloorRaisesOfTheStep::stop_label`]；A 臂与发成的那几次是 `none`）。
    pub(super) floor_raise_stop: String,
    /// 谷值钩子在会话的分配器上记根时读到的每块盘最小空闲槽（抬 F 预演在拷贝上记的不算，[`hook_readings_on_the_allocator_of_the_session`]）；
    /// 一次都没读到（被拒且没有一串抬 F 落盘）时取发布之前的空闲槽。
    valley_free_slots: Vec<u64>,
    /// 谷值与 free_before − W_t 逐盘比过（S5）；推了抬 F 的那几次不比，谷值直接取钩子读数（第十二节修订）。
    is_valley_checked_against_the_new_slots: bool,
    pub(super) shape: Option<PublishShape>,
    pub(super) refusal: Option<DeleteRefusal>,
}

impl DeletePublishReading {
    pub(super) fn borrow_at_the_valley(&self) -> Vec<i128> {
        self.reserve_slots
            .iter()
            .zip(&self.valley_free_slots)
            .map(|(reserve, valley)| reserve - i128::from(*valley))
            .collect()
    }

    pub(super) fn borrow_between_publishes(&self) -> Vec<i128> {
        self.reserve_slots
            .iter()
            .zip(&self.free_before_slots)
            .map(|(reserve, free)| reserve - i128::from(*free))
            .collect()
    }

    pub(super) fn line(&self, cell_label: &str) -> String {
        let per_device = |name: &str, values: &[String]| -> String {
            values
                .iter()
                .enumerate()
                .map(|(device, value)| format!("{name}_device{device}={value}"))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let strings_of =
            |values: &[i128]| values.iter().map(ToString::to_string).collect::<Vec<_>>();
        let strings_of_unsigned =
            |values: &[u64]| values.iter().map(ToString::to_string).collect::<Vec<_>>();
        let shape = self.shape.map_or_else(
            || "ordinary_slots=none fixed_point_slots=none switch_reserve_slots=none landing_demand_slots=none w_exceeds_landing=none".to_owned(),
            |shape| {
                format!(
                    "ordinary_slots={} fixed_point_slots={} switch_reserve_slots={} landing_demand_slots={} w_exceeds_landing={}",
                    shape.ordinary_slots,
                    shape.fixed_point_slots,
                    shape.switch_reserve_slots,
                    shape.landing_demand_slots,
                    u8::from(self.new_slots.iter().any(|new_slots| *new_slots > shape.landing_demand_slots))
                )
            },
        );
        format!(
            "E name=publish cell={cell_label} t={} kept_data_units={} checkpoint_cost={} {} {} {} {} {} {} {} floor_raise_publishes={} floor_raise_stop={} valley_checked={} {} refused={}",
            self.position,
            self.kept_data_units,
            self.checkpoint_cost,
            per_device("reserve", &strings_of(&self.reserve_slots)),
            per_device("free_before", &strings_of_unsigned(&self.free_before_slots)),
            per_device("new_slots", &strings_of_unsigned(&self.new_slots)),
            per_device("floor_raise_new_slots", &strings_of_unsigned(&self.floor_raise_new_slots)),
            per_device("valley_free", &strings_of_unsigned(&self.valley_free_slots)),
            per_device("borrow_valley", &strings_of(&self.borrow_at_the_valley())),
            per_device("borrow_between", &strings_of(&self.borrow_between_publishes())),
            self.floor_raise_publishes,
            self.floor_raise_stop,
            u8::from(self.is_valley_checked_against_the_new_slots),
            shape,
            self.refusal.map_or("none", DeleteRefusal::label),
        )
    }
}

/// 删除的每一步：(保留的数据单元数 M, 删后 size)。意图那次留全部、size 不变；之后从尾部每批截 K 个，直到 0。
fn plan_of_the_delete(current: &TransactionOutput, batch_data_units: u64) -> Vec<(u64, u64)> {
    let payload_bytes = u64::try_from(data_unit_payload_capacity()).expect("净荷字节数");
    let mut kept = u64::try_from(current.data_pointers.len()).expect("单元数");
    let mut steps = vec![(
        kept,
        current
            .inode_record
            .expect("删之前那一版有第一个文件那条记录")
            .size,
    )];
    while kept > 0 {
        kept = kept.saturating_sub(batch_data_units);
        steps.push((kept, kept * payload_bytes));
    }
    steps
}

fn new_slots_of_each_device(output: &TransactionOutput) -> Vec<u64> {
    (0..DEVICE_COUNT)
        .map(|device_number| {
            new_slots_on_device(
                output,
                DeviceIdentity(u32::try_from(device_number).expect("盘号")),
            )
        })
        .collect()
}

fn summed_new_slots_of_the_floor_raises(floor_raise_outputs: &[&TransactionOutput]) -> Vec<u64> {
    let mut summed = vec![0; usize::try_from(DEVICE_COUNT).expect("盘数")];
    for output in floor_raise_outputs {
        for (total, new_slots) in summed.iter_mut().zip(new_slots_of_each_device(output)) {
            *total += new_slots;
        }
    }
    summed
}

/// 一步删除发布的结局：成了的新一版（与推的抬 F 空发布），或被拒的种类（与推的抬 F 空发布）。
pub(super) struct StepOutcome {
    published: Option<TransactionOutput>,
    floor_raise_outputs: Vec<TransactionOutput>,
    floor_raises: FloorRaisesOfTheStep,
    pub(super) refusal: Option<DeleteRefusal>,
}

/// 一步删除发布里会话推抬 F 的样子（B 臂；A 臂与没推的是 [`FloorRaisesOfTheStep::none`]）。谷值钩子在每次记根时读空闲槽，而抬 F
/// 那一串先在分配器的一份拷贝上整串预演（`mount::rehearse_the_publishes_raising_the_floor`，预演也走记根）、预演过了才真发：落了盘的
/// 一串 n 次空发布，钩子先读到拷贝上的 n 次、再读到真发的 n 次；在预演里第 k 次被拒的那一串，钩子读到拷贝上的 k − 1 次，一次都没真发。
/// 拷贝上的读数不是会话分配器的空闲槽，不进谷值（登记 1.2 的 free_t 是分配器这块盘此刻的空闲槽；第十二节修订第 16 条）。
/// 例如 P1-S16-K1-Lseq-Fend-N-B-N88-noreclaim 的 t = 7：落点拒（9 槽 < 11），抬 F 那一串（两次各 6 槽）
/// 在预演里第 2 次被拒，钩子读到拷贝上第 1 次之后的 3 槽，谷值被记成 3、borrow 记成 83，而会话分配器的空闲一直是 9。
#[derive(Clone, Debug, PartialEq, Eq)]
struct FloorRaisesOfTheStep {
    /// 落了盘的每一串的空发布数，按先后。
    publishes_of_each_landed_chain: Vec<usize>,
    /// 最后推的那一串在预演里第 k 次被拒时它在拷贝上记过的根数 k − 1；没有这样一串是 0。
    rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: usize,
    /// 被拒那一次会话不再推的原因；发成的、A 臂的是 `none`。
    stop_label: String,
}

impl FloorRaisesOfTheStep {
    fn none() -> Self {
        Self {
            publishes_of_each_landed_chain: Vec::new(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
            stop_label: "none".to_owned(),
        }
    }

    /// 发成的那一次：推过的几串都落了盘，不再推。
    fn of_a_published_step(floor_raises: &[RaisedFloor]) -> Self {
        Self {
            publishes_of_each_landed_chain: floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .collect(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
            stop_label: "none".to_owned(),
        }
    }

    /// 推满仍拒的那一次（`UserChangeRefused::NoSpaceAfterRaisingTheFloor`）：落了盘的几串与不再推的原因。抬 F 那一串真发途中失败
    /// （`MountError::RaiseFloorSequencePublishFailed`）时，前面已落盘的几次不在 `floor_raises` 里，抬 F 新占的槽与 B5 的等式都数不全，
    /// 交回 Err（调用方停机）。
    fn of_a_refused_step(
        floor_raises: &[RaisedFloor],
        stop: &FloorRaiseStop,
    ) -> Result<Self, String> {
        let (stop_label, rehearsal_root_recordings) = match stop {
            FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => {
                ("publishes_per_admission_would_be_exceeded".to_owned(), 0)
            }
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {
                ("floor_already_at_the_ceiling".to_owned(), 0)
            }
            FloorRaiseStop::FloorRaiseRefusedForSpace { refusal } => {
                let MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    ..
                } = refusal.as_ref()
                else {
                    return Err(format!("{refusal:?}"));
                };
                (
                    format!("floor_raise_refused_for_space_in_the_rehearsal_at_publish_{refused_publish_in_the_sequence}_of_{publishes_in_the_sequence}"),
                    refused_publish_in_the_sequence - 1,
                )
            }
        };
        Ok(Self {
            publishes_of_each_landed_chain: floor_raises
                .iter()
                .map(|raised| raised.publishes.len())
                .collect(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal:
                rehearsal_root_recordings,
            stop_label,
        })
    }
}

/// 这一步里钩子在会话的分配器上读到的那几条（按先后）：每一串落了盘的抬 F 先丢掉它在拷贝上预演的 n 条、留下真发的 n 条，发成的这一次留 1 条，
/// 最后在预演里被拒的那一串丢掉它在拷贝上的 k − 1 条（[`FloorRaisesOfTheStep`]）。钩子读到的条数与这一拆法对不上交回 None（调用方停机：
/// 有一处记根不在这套账里，谷值读的就不知道是哪个分配器）。
fn hook_readings_on_the_allocator_of_the_session(
    hook_readings: Vec<Vec<u64>>,
    floor_raises: &FloorRaisesOfTheStep,
    is_published: bool,
) -> Option<Vec<Vec<u64>>> {
    let mut remaining = hook_readings.into_iter();
    let mut on_the_allocator = Vec::new();
    for publishes_of_the_chain in &floor_raises.publishes_of_each_landed_chain {
        for _rehearsed_on_a_copy in 0..*publishes_of_the_chain {
            remaining.next()?;
        }
        for _published_on_the_allocator in 0..*publishes_of_the_chain {
            on_the_allocator.push(remaining.next()?);
        }
    }
    if is_published {
        on_the_allocator.push(remaining.next()?);
    }
    for _rehearsed_on_a_copy_then_refused in
        0..floor_raises.rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal
    {
        remaining.next()?;
    }
    if remaining.next().is_some() {
        return None;
    }
    Some(on_the_allocator)
}

/// A 臂：直接在会话的分配器与现行那一版上发这一批，被拒原样交回。
pub(super) fn publish_the_step_directly(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    kept_data_units: u64,
    size_after_the_batch: u64,
) -> StepOutcome {
    let previous = pool.current_file_version().clone();
    let result = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        (hooks.publish_unlink_batch)(
            &mut writer,
            &mut pool.session.allocator,
            &previous,
            kept_data_units,
            size_after_the_batch,
            pool.session.instance,
        )
    };
    match result {
        Ok(output) => {
            pool.session.current = PoolVersion::WithFile(std::sync::Arc::new(output.clone()));
            StepOutcome {
                published: Some(output),
                floor_raise_outputs: Vec::new(),
                floor_raises: FloorRaisesOfTheStep::none(),
                refusal: None,
            }
        }
        Err(error) => {
            let refusal = if matches!(error, PublishError::SpaceAdmissionRefused(_)) {
                DeleteRefusal::Landing
            } else if matches!(error, PublishError::PlacementRefused { .. }) {
                DeleteRefusal::Placement
            } else {
                stop_with(
                    "unexpected_publish_error",
                    &format!("geometry={} error={error:?}", pool.geometry.label()),
                )
            };
            StepOutcome {
                published: None,
                floor_raise_outputs: Vec::new(),
                floor_raises: FloorRaisesOfTheStep::none(),
                refusal: Some(refusal),
            }
        }
    }
}

/// B 臂：经会话发（`MountedSession::publish_user_change`），落点被拒由会话推抬 F 再判。
fn publish_the_step_through_the_session(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    kept_data_units: u64,
    size_after_the_batch: u64,
) -> StepOutcome {
    let change = (hooks.unlink_batch_change)(kept_data_units, size_after_the_batch);
    match pool.session.publish_user_change(&mut pool.devices, change) {
        Ok(published) => StepOutcome {
            published: Some(pool.current_file_version().clone()),
            floor_raises: FloorRaisesOfTheStep::of_a_published_step(&published.floor_raises),
            floor_raise_outputs: published
                .floor_raises
                .into_iter()
                .flat_map(|raised| raised.publishes)
                .collect(),
            refusal: None,
        },
        Err(refusal) => {
            let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
                stop_with(
                    "unexpected_user_change_refusal",
                    &format!("geometry={} refusal={refusal:?}", pool.geometry.label()),
                )
            };
            let floor_raises =
                FloorRaisesOfTheStep::of_a_refused_step(&no_space.floor_raises, &no_space.stop)
                    .unwrap_or_else(|refusal_of_the_raise| {
                        stop_with(
                            "floor_raise_publishes_landed_outside_the_refusal",
                            &format!(
                                "geometry={} refusal_of_the_raise={refusal_of_the_raise}",
                                pool.geometry.label()
                            ),
                        )
                    });
            StepOutcome {
                published: None,
                floor_raise_outputs: no_space
                    .floor_raises
                    .into_iter()
                    .flat_map(|raised| raised.publishes)
                    .collect(),
                floor_raises,
                refusal: Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor),
            }
        }
    }
}

/// 删除期间谷值钩子（S5）的取数口。钩子在 `record_root_written_by_this_process` 每次记根时都读（建池时的挂载暖机、写文件、重挂载与前面
/// 各格的发布都记根；压满只标槽、不记根），读数攒在线程里等装置取走；只有 [`ValleyHookDuringTheDelete::opened_at_the_start_of_the_delete`] 造得出这个取数口，而它先把
/// 删除开始之前攒下的读数取走丢掉，删除各步之间再读到的才是「删除发布之外」的读数（登记第十二节修订第 8 条）。
struct ValleyHookDuringTheDelete {
    pub(super) take_free_slots_recorded_at_each_root: fn() -> Vec<Vec<u64>>,
}

impl ValleyHookDuringTheDelete {
    fn opened_at_the_start_of_the_delete(
        take_free_slots_recorded_at_each_root: fn() -> Vec<Vec<u64>>,
    ) -> Self {
        drop(take_free_slots_recorded_at_each_root());
        Self {
            take_free_slots_recorded_at_each_root,
        }
    }

    /// 自开口或上一次取走以来每次记根读到的每块盘空闲槽。
    fn take_readings_since_the_last_take(&self) -> Vec<Vec<u64>> {
        (self.take_free_slots_recorded_at_each_root)()
    }
}

/// 删除类发布一律不判式子那一判、落点那一判照判（登记 1.2「被落点那一判拒」、5.1 第 4 步「删除各批（含意图那次）不判式子那一判」）：
/// 意图那次、各批，与停机条款 S7 判 checker 之前那一次「续做一遍发现无事可做」的删除发布（登记第十二节修订第 9 条）都经它发。
/// 只有 [`FormulaJudgementSkippedForDeletePublishes::opened`] 造得出它：开口时关掉式子那一判、记下那一刻的判数；
/// [`FormulaJudgementSkippedForDeletePublishes::closed`] 把式子那一判开回来，交回其间真判了几次（该是 0）。
/// 没经它、式子那一判开着，P1-S4-K1-Lseq-Fend-N-A-N1 的 S7 那一次发布就会被它拒掉（登记第十二节修订第 14 条）。
pub(super) struct FormulaJudgementSkippedForDeletePublishes {
    pub(super) set_formula_judgement_skipped: fn(bool),
    pub(super) formula_judgements_made: fn() -> u64,
    formula_judgements_when_opened: u64,
}

impl FormulaJudgementSkippedForDeletePublishes {
    pub(super) fn opened(hooks: &DeletePathHooks) -> Self {
        (hooks.set_formula_judgement_skipped)(true);
        Self {
            set_formula_judgement_skipped: hooks.set_formula_judgement_skipped,
            formula_judgements_made: hooks.formula_judgements_made,
            formula_judgements_when_opened: (hooks.formula_judgements_made)(),
        }
    }

    pub(super) fn closed(self) -> u64 {
        (self.set_formula_judgement_skipped)(false);
        (self.formula_judgements_made)() - self.formula_judgements_when_opened
    }
}

/// 一条删除的结局：逐次读数（被拒那一次在最后），删完与否。
pub(super) struct DeleteRun {
    pub(super) readings: Vec<DeletePublishReading>,
    pub(super) is_completed: bool,
}

/// 5.1 第 4、5 步：意图那次与各批，删除各批不判式子那一判、落点那一判照判；每次发布之前读 reserve_t、free_t，之后读 W_t 与谷值。
/// 被拒就停在那里（A 臂直接交回，B 臂经会话推满仍拒）。删除期间 inode 1 记录写 nlink = 0；停机条款 S5、S6 与「式子那一判调用计数为 0」在这里判。
pub(super) fn run_the_delete(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    arm: Arm,
    batch_data_units: u64,
    ledger_of_the_arm: Option<(u64, u64)>,
    reclaim: ReclaimDuringTheDelete,
) -> DeleteRun {
    let steps = plan_of_the_delete(pool.current_file_version(), batch_data_units);
    (hooks.set_inode_written_with_zero_links)(Some(1));
    let formula_judgement_skipped_for_the_delete =
        FormulaJudgementSkippedForDeletePublishes::opened(hooks);
    (hooks.set_reclaim_switched_off)(reclaim == ReclaimDuringTheDelete::SwitchedOff);
    let valley_hook = ValleyHookDuringTheDelete::opened_at_the_start_of_the_delete(
        hooks.take_free_slots_recorded_at_each_root,
    );
    let mut readings = Vec::new();
    let mut is_completed = true;
    for (position, (kept_data_units, size_after_the_batch)) in (0u64..).zip(steps) {
        let (reserve_slots, checkpoint_cost) = reserve_reading(pool, hooks, ledger_of_the_arm);
        let free_before_slots = pool.free_slots_of_each_device();
        let stale_hook_readings = valley_hook.take_readings_since_the_last_take();
        if !stale_hook_readings.is_empty() {
            stop_with(
                "valley_hook_read_outside_a_delete_publish",
                &format!(
                    "geometry={} t={position} readings={}",
                    pool.geometry.label(),
                    stale_hook_readings.len()
                ),
            );
        }
        let outcome = match arm.refusal {
            RefusalHandling::Direct => {
                publish_the_step_directly(pool, hooks, kept_data_units, size_after_the_batch)
            }
            RefusalHandling::ThroughTheSession => publish_the_step_through_the_session(
                pool,
                hooks,
                kept_data_units,
                size_after_the_batch,
            ),
        };
        count_a_delete_publish_on_this_slice();
        let hook_readings = valley_hook.take_readings_since_the_last_take();
        let before_the_step = ReadingBeforeTheStep {
            position,
            kept_data_units,
            reserve_slots,
            checkpoint_cost,
            free_before_slots,
        };
        let reading = reading_of_the_step(pool, outcome, hook_readings, before_the_step);
        let is_refused = reading.refusal.is_some();
        readings.push(reading);
        if is_refused {
            is_completed = false;
            break;
        }
    }
    (hooks.set_inode_written_with_zero_links)(None);
    let formula_judgements_during_the_delete = formula_judgement_skipped_for_the_delete.closed();
    (hooks.set_reclaim_switched_off)(false);
    if formula_judgements_during_the_delete != 0 {
        stop_with(
            "formula_judgement_made_during_the_delete",
            &format!(
                "geometry={} judgements={formula_judgements_during_the_delete}",
                pool.geometry.label()
            ),
        );
    }
    DeleteRun {
        readings,
        is_completed,
    }
}

/// 一步删除发布之前读到的量。
struct ReadingBeforeTheStep {
    pub(super) position: u64,
    kept_data_units: u64,
    pub(super) reserve_slots: Vec<i128>,
    pub(super) checkpoint_cost: u64,
    pub(super) free_before_slots: Vec<u64>,
}

/// 一步的读数，停机条款 S5（谷值 = free_before − W_t）、S6（两盘 W 相等）与 V4（成了的发布钩子一次都没读到）在这里判。
fn reading_of_the_step(
    pool: &MountedPool,
    outcome: StepOutcome,
    hook_readings: Vec<Vec<u64>>,
    before: ReadingBeforeTheStep,
) -> DeletePublishReading {
    let floor_raise_references: Vec<&TransactionOutput> =
        outcome.floor_raise_outputs.iter().collect();
    let floor_raise_new_slots = summed_new_slots_of_the_floor_raises(&floor_raise_references);
    let floor_raise_publishes =
        u64::try_from(outcome.floor_raise_outputs.len()).expect("抬 F 的发布数");
    let new_slots = outcome.published.as_ref().map_or_else(
        || vec![0; floor_raise_new_slots.len()],
        new_slots_of_each_device,
    );
    if new_slots.windows(2).any(|pair| pair[0] != pair[1]) {
        stop_with(
            "s6_new_slots_differ_between_devices",
            &format!(
                "geometry={} t={} new_slots={new_slots:?}",
                pool.geometry.label(),
                before.position
            ),
        );
    }
    let hook_readings_count = hook_readings.len();
    let Some(hook_readings) = hook_readings_on_the_allocator_of_the_session(
        hook_readings,
        &outcome.floor_raises,
        outcome.published.is_some(),
    ) else {
        stop_with(
            "valley_hook_readings_do_not_match_the_root_recordings_of_the_step",
            &format!(
                "geometry={} t={} hook_readings={hook_readings_count} floor_raises={:?} published={}",
                pool.geometry.label(),
                before.position,
                outcome.floor_raises,
                outcome.published.is_some()
            ),
        )
    };
    if outcome.published.is_some() && hook_readings.is_empty() {
        stop_with(
            "v4_valley_hook_read_nothing",
            &format!("geometry={} t={}", pool.geometry.label(), before.position),
        );
    }
    let valley_free_slots: Vec<u64> = (0..before.free_before_slots.len())
        .map(|device_position| {
            hook_readings
                .iter()
                .map(|reading| reading[device_position])
                .min()
                .unwrap_or(before.free_before_slots[device_position])
        })
        .collect();
    let is_valley_checked_against_the_new_slots =
        outcome.published.is_some() && floor_raise_publishes == 0;
    if is_valley_checked_against_the_new_slots {
        let expected: Vec<u64> = before
            .free_before_slots
            .iter()
            .zip(&new_slots)
            .map(|(free, new)| free - new)
            .collect();
        if hook_readings.len() != 1 || hook_readings[0] != expected {
            stop_with(
                "s5_valley_differs_from_free_before_minus_new_slots",
                &format!(
                    "geometry={} t={} hook={hook_readings:?} expected={expected:?}",
                    pool.geometry.label(),
                    before.position
                ),
            );
        }
    }
    DeletePublishReading {
        position: before.position,
        kept_data_units: before.kept_data_units,
        reserve_slots: before.reserve_slots,
        checkpoint_cost: before.checkpoint_cost,
        free_before_slots: before.free_before_slots,
        new_slots,
        floor_raise_publishes,
        floor_raise_new_slots,
        floor_raise_stop: outcome.floor_raises.stop_label,
        valley_free_slots,
        is_valley_checked_against_the_new_slots,
        shape: outcome
            .published
            .as_ref()
            .map(|output| shape_of_the_publish(output, before.checkpoint_cost)),
        refusal: outcome.refusal,
    }
}

#[cfg(test)]
mod tests;
