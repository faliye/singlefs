//! 一格的驱动：跑一格、阳性对照的等式与删完之后的核对。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 一格的驱动
// ---------------------------------------------------------------------------------------------------------------------

/// 一格的设定（第六节「格」；布局 L-seq、压满方向 F-end 在第一段固定）。
#[derive(Clone, Copy, Debug)]
pub(super) struct CellSetting {
    pub(super) geometry: Geometry,
    pub(super) batch_data_units: u64,
    pub(super) arm: Arm,
    pub(super) data_units: u64,
    /// W 臂账里那一份用的 W_max（同一几何上 N-A 量到的）；N 臂不用。
    pub(super) set_aside_largest_new_slots: u64,
    pub(super) reclaim: ReclaimDuringTheDelete,
}

impl CellSetting {
    pub(super) fn label(&self) -> String {
        let reclaim = match self.reclaim {
            ReclaimDuringTheDelete::AsToday => "",
            ReclaimDuringTheDelete::SwitchedOff => "-noreclaim",
        };
        format!(
            "{}-K{}-Lseq-Fend-{}-N{}{reclaim}",
            self.geometry.label(),
            self.batch_data_units,
            self.arm.label(),
            self.data_units
        )
    }

    pub(super) fn ledger(&self) -> Option<(u64, u64)> {
        ledger_of_the_arm(
            self.arm,
            self.geometry.slots_per_region,
            self.set_aside_largest_new_slots,
        )
    }
}

/// 一条臂的账（按全部副本之和记的字节）：N 臂两项都是 0；W 臂已承诺预留 = (3S + 1) × W_max × 16384 × 2。
pub(super) fn ledger_of_the_arm(
    arm: Arm,
    slots_per_region: u64,
    set_aside_largest_new_slots: u64,
) -> Option<(u64, u64)> {
    match arm.ledger {
        LedgerArm::NothingSetAside => Some((0, 0)),
        LedgerArm::RingWindowSetAside => Some((
            0,
            ring_window_bound_slots(slots_per_region, set_aside_largest_new_slots)
                * SLOT_BYTES
                * DEVICE_COUNT,
        )),
    }
}

/// 压满那一步（5.3）按哪一份账压：账与设账的入口（装置里是补丁表接上的 `set_ledger`；单测不经补丁，账恒为第一版的两个 0，
/// 只测 N 臂，设账入口什么都不做）。
#[derive(Clone, Copy)]
pub(super) struct LedgerForTheFill {
    pub(super) ledger: Option<(u64, u64)>,
    pub(super) set_ledger: fn(Option<(u64, u64)>),
}

/// 一格的结局。
#[derive(Clone, Debug)]
pub(super) enum CellOutcome {
    /// 5.1 第 1、2 步在 `step` 被拒（梯子的上沿；P0 预期如此）。
    FileNotWritten { step: &'static str },
    /// 压满之前式子可用就 ≤ −16384，造不出满池。
    NotFillable,
    /// 删过了；`free_after_the_fill` 是压满之后每块盘的空闲槽（阳性对照拿它与 free_0 比）。
    Deleted {
        summary: Box<DeleteSummary>,
        floor_raise_publishes: u64,
        /// 最后一次删除发布里会话不再推抬 F 的原因（[`FloorRaisesOfTheStep::stop_label`]；阳性对照的行报它）。
        floor_raise_stop_of_the_last_publish: String,
        free_after_the_fill: Vec<u64>,
    },
}

/// 一格（[`run_the_cell_uncounted`]），判完记进这一片的进度（进度行读，不进产物）。
pub(super) fn run_the_cell(setting: &CellSetting, hooks: &DeletePathHooks) -> CellOutcome {
    let outcome = run_the_cell_uncounted(setting, hooks);
    count_a_judged_cell_on_this_slice();
    outcome
}

/// 一格：建池、核 S4 / B4（P4 上另核 A1）、按臂的账压满、删、核 S7（回收照今天的格）与 B5（回收关着的格），逐次行与格行都打出来。
fn run_the_cell_uncounted(setting: &CellSetting, hooks: &DeletePathHooks) -> CellOutcome {
    let label = setting.label();
    let mut pool = match pool_with_a_file(setting.geometry, setting.data_units) {
        PoolWithAFile::Written(pool) => *pool,
        PoolWithAFile::Refused { step, detail, .. } => {
            emit_product_line!(
                "E name=cell cell={label} first_file=refused step={step} detail={detail:?}"
            );
            return CellOutcome::FileNotWritten { step };
        }
    };
    let (root_level, allocation_record_tree_positions) =
        allocation_record_tree_positions_checked(&pool, hooks);
    if setting.geometry.point == DeviceSizePoint::Product {
        let holds = allocation_record_tree_positions == 529;
        emit_product_line!("E name=anchor_a anchor=A1 cell={label} allocation_record_tree_positions={allocation_record_tree_positions} expected=529 holds={holds}");
        if !holds {
            stop_with("f7_anchor_a1_does_not_hold", &format!("cell={label}"));
        }
    }
    let ledger = setting.ledger();
    (hooks.set_ledger)(ledger);
    let (free_after_the_fill, filled_slots_of_each_device) = match fill_from_the_unit_area_end(
        &mut pool,
    ) {
        FillOutcome::WithinOneSlotAtOrBelowZero {
            free_slots,
            filled_slots_of_each_device,
        } => (free_slots, filled_slots_of_each_device),
        FillOutcome::AlreadyBelowTheIntervalBeforeFilling { available_bytes } => {
            (hooks.set_ledger)(None);
            emit_product_line!("E name=cell cell={label} first_file=written fill=already_below_the_interval available_bytes={available_bytes:?}");
            return CellOutcome::NotFillable;
        }
    };
    let run = run_the_delete(
        &mut pool,
        hooks,
        setting.arm,
        setting.batch_data_units,
        ledger,
        setting.reclaim,
    );
    (hooks.set_ledger)(None);
    for reading in &run.readings {
        emit_product_line!("{}", reading.line(&label));
    }
    let admissions_over_the_limit = run
        .readings
        .iter()
        .filter(|reading| reading.floor_raise_publishes + 1 > PUBLISHES_IN_ONE_ADMISSION_AT_MOST)
        .count();
    if setting.arm.refusal == RefusalHandling::ThroughTheSession {
        emit_product_line!("E name=anchor_a anchor=A4 cell={label} admissions_over_eight_publishes={admissions_over_the_limit} holds={}", admissions_over_the_limit == 0);
    }
    match setting.reclaim {
        ReclaimDuringTheDelete::SwitchedOff => check_the_positive_control_equation(&run, &label),
        ReclaimDuringTheDelete::AsToday => {
            if run.is_completed {
                check_the_finished_delete(
                    &mut pool,
                    hooks,
                    setting,
                    &run,
                    &label,
                    &filled_slots_of_each_device,
                );
            }
        }
    }
    let summary = summary_of_the_delete(&run);
    let floor_raise_publishes: u64 = run
        .readings
        .iter()
        .map(|reading| reading.floor_raise_publishes)
        .sum();
    emit_product_line!(
        "E name=cell cell={label} first_file=written root_level={root_level} allocation_record_tree_positions={allocation_record_tree_positions} unit_area_slots={} free_after_fill={free_after_the_fill:?} floor_raise_publishes={floor_raise_publishes} {}",
        setting.geometry.unit_area_slots(),
        summary_fields(&summary, setting.geometry.slots_per_region)
    );
    CellOutcome::Deleted {
        summary: Box::new(summary),
        floor_raise_publishes,
        floor_raise_stop_of_the_last_publish: run
            .readings
            .last()
            .map_or_else(|| "none".to_owned(), |last| last.floor_raise_stop.clone()),
        free_after_the_fill,
    }
}

/// 第七节 B5（回收关着时）：空闲槽逐次只减不加，free_before_t = free_before_0 − Σ_{s<t}（W_s + 那一次推的抬 F 新占的槽），逐盘逐次断言等号。
/// 登记写的是 borrow_t = borrow_0 + Σ W_s；保留量 reserve_t 随 ckpt_cost 逐次变时那一形会对不上而测量没错，所以两形都算：
/// 空闲槽那一形不成立就停（V1），borrow 那一形对不上的次数照报（`borrow_form_mismatches`），V1 按哪一形判交主 agent。
fn check_the_positive_control_equation(run: &DeleteRun, label: &str) {
    let first = run.readings.first().expect("至少意图那一次");
    let mut expected_free = first.free_before_slots.clone();
    let mut expected_borrow = first.borrow_between_publishes();
    let mut borrow_form_mismatches = 0u64;
    for reading in &run.readings {
        if reading.free_before_slots != expected_free {
            stop_with(
                "v1_b5_positive_control_equation_broken",
                &format!(
                    "cell={label} t={} free_before={:?} expected={expected_free:?}",
                    reading.position, reading.free_before_slots
                ),
            );
        }
        borrow_form_mismatches += u64::from(reading.borrow_between_publishes() != expected_borrow);
        for (((free, borrow), new_slots), raise_slots) in expected_free
            .iter_mut()
            .zip(expected_borrow.iter_mut())
            .zip(&reading.new_slots)
            .zip(&reading.floor_raise_new_slots)
        {
            *free -= new_slots + raise_slots;
            *borrow += i128::from(new_slots + raise_slots);
        }
    }
    emit_product_line!(
        "E name=anchor_b anchor=B5 cell={label} publishes={} free_form_holds=true borrow_form_mismatches={borrow_form_mismatches}",
        run.readings.len()
    );
}

/// 停机条款 S7 判 checker 之前那一次「续做一遍发现无事可做」的删除发布（保留 0 个单元、inode 1 写 nlink = 0）：它是删除类发布，
/// 照意图那次与各批的规矩发——式子那一判关着、落点那一判照判（[`FormulaJudgementSkippedForDeletePublishes`]）。交回这一次的结局与
/// 其间式子那一判真判的次数（该是 0）。
fn publish_the_ledger_rewrite_as_a_delete_publish(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
) -> (StepOutcome, u64) {
    let formula_judgement_skipped_for_the_ledger_rewrite =
        FormulaJudgementSkippedForDeletePublishes::opened(hooks);
    (hooks.set_inode_written_with_zero_links)(Some(1));
    let ledger_rewrite = publish_the_step_directly(pool, hooks, 0, 0);
    (hooks.set_inode_written_with_zero_links)(None);
    (
        ledger_rewrite,
        formula_judgement_skipped_for_the_ledger_rewrite.closed(),
    )
}

/// 停机条款 S7：删完之后 inode 1 的数据单元为 0、删除发布次数 = 1 + ⌈N ÷ K⌉、池级 checker 在删完的镜像上不判红。
///
/// 压满（登记 5.3）只在分配器上标槽、不写盘：标掉的槽进了分配器的「已分配」，删除各次发布照 `transaction.rs` 把分配器的已分配写进
/// 记账行，盘上却没有任何东西引用它们；不撤掉就判，I-3.1、I-3.11 两条差的恰好是压满标掉的槽（例如
/// P2-S4-K1-Lseq-Fend-N-A-N1：盘 0 记账 292 槽、走读 49 槽，登记第十二节修订第 9 条）。所以判 checker 之前先撤掉压满，再发一次
/// 「续做一遍发现无事可做」的删除发布（保留 0 个单元、inode 1 照旧写 nlink = 0，D8 已定项 5 三态表第三行），最新根的记账按撤掉之后的
/// 分配器重写，checker 判这一版；删除各次的读数在这之前已经取完，这一次发布不进任何量。
fn check_the_finished_delete(
    pool: &mut MountedPool,
    hooks: &DeletePathHooks,
    setting: &CellSetting,
    run: &DeleteRun,
    label: &str,
    filled_slots_of_each_device: &[Vec<u64>],
) {
    let remaining_data_units = pool.current_file_version().data_pointers.len();
    let expected_publishes = delete_publish_count(setting.data_units, setting.batch_data_units);
    let publishes = u64::try_from(run.readings.len()).expect("发布数");
    let filled_slots: Vec<usize> = filled_slots_of_each_device.iter().map(Vec::len).collect();
    let allocated_slots_before_the_unmark: Vec<u64> = pool
        .session
        .allocator
        .devices
        .iter()
        .map(DeviceFreeMap::allocated_slots)
        .collect();
    unmark_the_filled_slots(
        &mut pool.session.allocator.devices,
        filled_slots_of_each_device,
    );
    let free_before_the_ledger_rewrite = pool.free_slots_of_each_device();
    let (ledger_rewrite, formula_judgements_during_the_ledger_rewrite) =
        publish_the_ledger_rewrite_as_a_delete_publish(pool, hooks);
    if formula_judgements_during_the_ledger_rewrite != 0 {
        stop_with(
            "formula_judgement_made_during_the_s7_ledger_rewrite",
            &format!("cell={label} judgements={formula_judgements_during_the_ledger_rewrite}"),
        );
    }
    if let Some(refusal) = ledger_rewrite.refusal {
        stop_with(
            "s7_ledger_rewrite_publish_after_the_unmark_refused",
            &format!(
                "cell={label} refusal={} filled_slots={filled_slots:?} free_before_the_ledger_rewrite={free_before_the_ledger_rewrite:?}",
                refusal.label()
            ),
        );
    }
    let violations = checker_violations(&format!("s7-finished-delete/{label}"), &pool.image());
    if remaining_data_units != 0 || publishes != expected_publishes || !violations.is_empty() {
        stop_with(
            "s7_finished_delete_is_not_what_the_patch_should_leave",
            &format!(
                "cell={label} remaining_data_units={remaining_data_units} publishes={publishes} expected_publishes={expected_publishes} filled_slots={filled_slots:?} allocated_slots_before_the_unmark={allocated_slots_before_the_unmark:?} checker_violations={violations:?}"
            ),
        );
    }
    emit_product_line!(
        "E name=s7_finished_delete cell={label} remaining_data_units={remaining_data_units} publishes={publishes} filled_slots={filled_slots:?} allocated_slots_before_the_unmark={allocated_slots_before_the_unmark:?} checker_violations=0"
    );
}

#[cfg(test)]
mod tests;
