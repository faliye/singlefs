//! 第 5 次跑的计数、量行、阳性对照与判决行。

use super::*;

/// 一个单位交回的东西：按它访问节点的深度优先序排的全部对。
#[derive(Debug, Default)]
pub(super) struct UnitOutput {
    pub(super) pairs: Vec<PairRecord>,
    pub(super) nodes_visited: u64,
    /// `enumerate` 展开时前提不满足的分支数（记为前提不满足，不算错）。
    pub(super) transition_failures: u64,
}

/// 处理一个单位：自己从 `bootstrap` 重放前缀、重建节点，沿用 `enumerate` 走子树，对每个节点的三种断链方式各判一个对；
/// 整个单位判完才把它现判的判定一次写进库，判到一半出错的一条都不写。
pub(super) fn process_unit(
    unit: &EnumerationUnit,
    domain: &FifthRunDomain,
    arm: &str,
    judgements: &ReusableJudgements,
) -> Result<UnitOutput, String> {
    let parameters = parameters_for(&domain.geometry);
    let start = replay_steps(&domain.geometry, unit.starting_overwrites, &unit.prefix)?;
    let geometry_view = independent_geometry(&start.pool)?;
    let depth = u64::try_from(unit.prefix.len()).expect("深度装得进 u64");
    let depth_remaining = if unit.whole_subtree {
        domain.step_limit.saturating_sub(depth)
    } else {
        0
    };
    let judging = PairJudging {
        geometry: &domain.geometry,
        parameters: &parameters,
        arm,
        starting_overwrites: unit.starting_overwrites,
        judgements,
    };
    let mut output = UnitOutput::default();
    let mut awaiting_write = JudgementsAwaitingWrite::new();
    let mut failure: Option<String> = None;
    let mut summary = EnumerationSummary::default();
    {
        let mut visit = |node: &SimNode, _events_live: u64, _roots_live: u64| {
            if failure.is_some() {
                return;
            }
            output.nodes_visited += 1;
            for replay_break in FIFTH_RUN_REPLAY_BREAKS {
                match judging.judge(node, replay_break, &mut awaiting_write) {
                    Ok(pair) => output.pairs.push(pair),
                    Err(error) => {
                        failure = Some(error);
                        return;
                    }
                }
            }
        };
        enumerate(
            &start,
            depth_remaining,
            &parameters,
            &geometry_view,
            &mut summary,
            &mut visit,
        );
    }
    if let Some(error) = failure {
        return Err(error);
    }
    output.transition_failures =
        u64::try_from(summary.errors.len()).expect("失败的分支数装得进 u64");
    // 只处理节点自己的单位不展开，它的分支由列单位那一趟展开：失败的分支在这里数，切深不改总数。
    if !unit.whole_subtree {
        output.transition_failures +=
            expansion_of_node(&start, &parameters, &geometry_view).transition_failures;
    }
    judgements.library.write_judgements(&awaiting_write)?;
    Ok(output)
}

/// 按原子游标动态派活（登记 5.7 第 3 条）：工人循环取下一个没做的单位，结果放进单位自己的格子；全部收回之后按单位序交回。
/// 收回的单位数不等于派出去的、或有工人 panic，交 `Err`（不把失败吃掉）。
pub(super) fn run_units_in_parallel(
    units: &[EnumerationUnit],
    threads: usize,
    process: &(dyn Fn(&EnumerationUnit) -> Result<UnitOutput, String> + Sync),
) -> Result<Vec<UnitOutput>, String> {
    let cursor = std::sync::atomic::AtomicUsize::new(0);
    let completed = std::sync::atomic::AtomicUsize::new(0);
    let progress_interval = (units.len() / 20).max(1);
    let slots: Vec<std::sync::Mutex<Option<Result<UnitOutput, String>>>> =
        units.iter().map(|_| std::sync::Mutex::new(None)).collect();
    let mut panicked_workers = 0usize;
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.clamp(1, units.len().max(1)))
            .map(|_| {
                scope.spawn(|| loop {
                    let index = cursor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(unit) = units.get(index) else {
                        break;
                    };
                    let result = process(unit);
                    *slots[index].lock().expect("单位格子的锁") = Some(result);
                    let done = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if done.is_multiple_of(progress_interval) || done == units.len() {
                        eprintln!("r5 progress units_done={done}/{}", units.len());
                    }
                })
            })
            .collect();
        for worker in workers {
            if worker.join().is_err() {
                panicked_workers += 1;
            }
        }
    });
    if panicked_workers > 0 {
        return Err(format!(
            "stop reason=worker_panic workers={panicked_workers}"
        ));
    }
    let mut outputs = Vec::with_capacity(units.len());
    for (index, slot) in slots.into_iter().enumerate() {
        match slot.into_inner().expect("单位格子的锁") {
            Some(Ok(output)) => outputs.push(output),
            Some(Err(error)) => {
                return Err(format!(
                    "stop reason=unit_failed unit={index} prefix={} detail={error}",
                    step_sequence_text(&units[index].prefix)
                ))
            }
            None => {}
        }
    }
    if outputs.len() != units.len() {
        return Err(format!(
            "stop reason=unit_count dispatched={} delivered={}",
            units.len(),
            outputs.len()
        ));
    }
    Ok(outputs)
}

/// 汇总用的计数：名字到数；`raise_to` 取最大，`lower_to` 取最小（没记过的是 `None`）。
#[derive(Default)]
pub(super) struct Tally {
    pub(super) values: BTreeMap<String, u64>,
    lowest: BTreeMap<String, u64>,
}

impl Tally {
    fn lower_to(&mut self, name: &str, value: u64) {
        let entry = self.lowest.entry(name.to_string()).or_insert(value);
        *entry = (*entry).min(value);
    }

    fn lowest_of(&self, name: &str) -> Option<u64> {
        self.lowest.get(name).copied()
    }

    pub(super) fn add(&mut self, name: &str, amount: u64) {
        *self.values.entry(name.to_string()).or_insert(0) += amount;
    }

    fn raise_to(&mut self, name: &str, value: u64) {
        let entry = self.values.entry(name.to_string()).or_insert(0);
        *entry = (*entry).max(value);
    }

    pub(super) fn count_of(&self, name: &str) -> u64 {
        self.values.get(name).copied().unwrap_or(0)
    }
}

/// 一格 = 一个被抛弃根在一个对里（变体都按它分组）。
pub(super) struct RootCell<'record> {
    pub(super) pair: &'record PairRecord,
    pub(super) root: TimelineRoot,
}

impl RootCell<'_> {
    pub(super) fn twin(&self) -> Option<&VariantResult> {
        self.pair
            .variant(self.root, VariantSpecification::Twin)
            .filter(|result| result.mounted_ok())
    }

    pub(super) fn account(
        &self,
        aspect: LedgerAspect,
        severity: FaultSeverity,
        form: UnreadableForm,
    ) -> Option<&VariantResult> {
        self.pair
            .variant(
                self.root,
                VariantSpecification::Account {
                    aspect,
                    severity,
                    form,
                },
            )
            .filter(|result| result.applicable)
    }

    pub(super) fn root_slot(&self, form: UnreadableForm) -> Option<&VariantResult> {
        self.pair
            .variant(self.root, VariantSpecification::RootSlot { form })
            .filter(|result| result.applicable)
    }
}

pub(super) fn root_cells(pairs: &[PairRecord]) -> Vec<RootCell<'_>> {
    let mut cells = Vec::new();
    for pair in pairs {
        if let PairOutcome::Constructed {
            readable_abandoned, ..
        } = &pair.outcome
        {
            for &root in readable_abandoned {
                cells.push(RootCell { pair, root });
            }
        }
    }
    cells
}

/// 某一盘上 `variant` 的隔离槽数比孪生少。
fn isolated_less_on_some_device(variant: &VariantResult, twin: &VariantResult) -> bool {
    twin.isolated_by_device.iter().any(|(device, twin_slots)| {
        let slots = variant
            .isolated_by_device
            .iter()
            .find(|(candidate, _)| candidate == device)
            .map_or(0, |(_, slots)| *slots);
        slots < *twin_slots
    })
}

/// 每一盘上 `variant` 的隔离槽数都比孪生少（阳性对照 skip 臂那一句）。
fn isolated_less_on_every_device(variant: &VariantResult, twin: &VariantResult) -> bool {
    !twin.isolated_by_device.is_empty()
        && twin.isolated_by_device.iter().all(|(device, twin_slots)| {
            let slots = variant
                .isolated_by_device
                .iter()
                .find(|(candidate, _)| candidate == device)
                .map_or(0, |(_, slots)| *slots);
            slots < *twin_slots
        })
}

/// 孪生每块盘上被隔离的槽数不少于这块盘上被抛弃根独占的槽数（锚点 A3）。
fn twin_isolates_every_exclusive_slot(twin: &VariantResult) -> bool {
    twin.exclusive_by_device.iter().all(|(device, exclusive)| {
        twin.isolated_by_device
            .iter()
            .find(|(candidate, _)| candidate == device)
            .map_or(0, |(_, slots)| *slots)
            >= *exclusive
    })
}

/// 账读不出的 Q1 至 Q5 与锚点 A3、续写终点的计数（登记第六节）；单份坏、两份坏分开记。
pub(super) fn tally_account_quantities(cells: &[RootCell], geometry: &Geometry) -> Tally {
    let mut tally = Tally::default();
    for cell in cells {
        let Some(twin) = cell.twin() else {
            tally.add("twin_missing_or_mount_error", 1);
            continue;
        };
        tally.add("twin_cells", 1);
        if twin.count != Some(0) {
            tally.add("q2c_twin_count_not_zero", 1);
        }
        if twin.exclusive_slots == 0 {
            tally.add("exclusive_slots_empty_cells", 1);
        }
        if !twin_isolates_every_exclusive_slot(twin) {
            tally.add("twin_isolates_fewer_than_exclusive", 1);
        }
        if twin.reuse_peak() > 0 {
            tally.add("q5_twin_cells_with_reuse", 1);
            tally.raise_to("q5_twin_reuse_peak", twin.reuse_peak());
        }
        if let Some(step) = twin.first_positive_reuse_step() {
            tally.lower_to("q5_first_positive_step", step);
        }
        if twin.continued_to_txg != Some(continuation_end_txg(cell.root, geometry)) {
            tally.add("continuation_not_at_the_end_txg", 1);
        }
        for aspect in [LedgerAspect::AllocationRecordTree, LedgerAspect::TreeTable] {
            for severity in [
                FaultSeverity::Both,
                FaultSeverity::Disk0,
                FaultSeverity::Disk1,
            ] {
                for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
                    let Some(variant) = cell.account(aspect, severity, form) else {
                        continue;
                    };
                    tally_one_account_variant(&mut tally, severity, variant, twin);
                }
            }
        }
    }
    tally
}

fn tally_one_account_variant(
    tally: &mut Tally,
    severity: FaultSeverity,
    variant: &VariantResult,
    twin: &VariantResult,
) {
    let both_copies = severity == FaultSeverity::Both;
    tally.add("q1_account_variants", 1);
    if !variant.mounted_ok() {
        tally.add("q1_mount_errors", 1);
        tally.add(&format!("q1_mount_error_member.{}", variant.error), 1);
        return;
    }
    if both_copies {
        if !variant.intercepted_all {
            tally.add("both_copies_not_intercepted", 1);
            return;
        }
        tally.add("both_copies_judged", 1);
        if variant.count != Some(variant.expected_count) {
            tally.add("q2a_count_not_through", 1);
        }
        if variant.shared {
            tally.add("q3b_shared_cases", 1);
            if isolated_less_on_some_device(variant, twin) {
                tally.add("q3b_isolated_less_than_twin", 1);
            }
        } else {
            tally.add("q3a_unshared_cases", 1);
            if variant.isolated_by_device != twin.isolated_by_device {
                tally.add("q3a_isolated_not_twin", 1);
            }
        }
        tally.add("q4_cases", 1);
        if variant.reuse_final() > twin.reuse_final() {
            tally.add("q4_reuse_more_than_twin", 1);
        }
        if variant.reuse_peak() > 0 {
            tally.add("q4_variants_with_reuse", 1);
            tally.raise_to("q4_reuse_peak", variant.reuse_peak());
        }
        if let Some(step) = variant.first_positive_reuse_step() {
            tally.lower_to("q4_first_positive_step", step);
        }
        tally.add("q4_reuse_final_sum", variant.reuse_final());
        tally.raise_to("q4_reuse_final_max", variant.reuse_final());
    } else {
        let bin = if variant.intercepted_all {
            "intercepted"
        } else {
            "not_intercepted"
        };
        tally.add(&format!("single_copy_variants.{bin}"), 1);
        if variant.count != Some(0) {
            tally.add(&format!("q2b_count_not_zero.{bin}"), 1);
        }
        tally.add("q3c_cases", 1);
        if variant.isolated_by_device != twin.isolated_by_device {
            tally.add("q3c_isolated_not_twin", 1);
        }
    }
}

/// 根槽读不出（C554 那一形）的 Q6 计数：只数被抛弃根里根槽读得出的只有一条、独占落点非空的格。
pub(super) fn tally_root_slot_quantities(cells: &[RootCell]) -> Tally {
    let mut tally = Tally::default();
    for cell in cells {
        let PairOutcome::Constructed {
            readable_abandoned, ..
        } = &cell.pair.outcome
        else {
            continue;
        };
        let Some(twin) = cell.twin() else {
            continue;
        };
        if readable_abandoned.len() != 1 || twin.exclusive_slots == 0 {
            continue;
        }
        tally.add("q6_cells", 1);
        for form in [UnreadableForm::ReadFails, UnreadableForm::ReadsZeros] {
            let Some(variant) = cell.root_slot(form) else {
                continue;
            };
            tally.add("q6_variants", 1);
            if !variant.mounted_ok() {
                tally.add("q6_mount_errors", 1);
                continue;
            }
            if variant.count != Some(0) {
                tally.add("q6a_count_not_zero", 1);
            }
            let twin_total = i64::try_from(twin.isolated_total()).expect("槽数装得进 i64");
            let variant_total = i64::try_from(variant.isolated_total()).expect("槽数装得进 i64");
            let exclusive = i64::try_from(twin.exclusive_slots).expect("槽数装得进 i64");
            let shortfall = twin_total - variant_total;
            if shortfall == exclusive {
                tally.add("q6b_shortfall_equals_exclusive", 1);
            }
            if shortfall < exclusive {
                tally.add("q6c_shortfall_below_exclusive", 1);
            }
            if shortfall > 0 {
                tally.add("q6d_shortfall_positive", 1);
            }
            if variant.reuse_final() > twin.reuse_final() {
                tally.add("q6e_reuse_more_than_twin", 1);
            }
            if variant.reuse_peak() > 0 {
                tally.add("q6e_variants_with_reuse", 1);
                tally.raise_to("q6e_reuse_peak", variant.reuse_peak());
            }
            if variant.isolated_total() == 0 {
                tally.add("q6f_isolated_total_zero", 1);
            }
        }
    }
    tally
}

/// Q0：造出被抛弃根的规模。节点 = 同一个 (n1, 步序列) 的三个对。
pub(super) fn tally_construction(pairs: &[PairRecord]) -> Tally {
    let mut tally = Tally::default();
    let mut nodes_with_a_construction: BTreeSet<(u64, String)> = BTreeSet::new();
    tally.add("pairs", u64::try_from(pairs.len()).expect("对数装得进 u64"));
    for pair in pairs {
        let node_key = (pair.starting_overwrites, step_sequence_text(&pair.steps));
        match &pair.outcome {
            PairOutcome::Constructed {
                abandoned,
                readable_abandoned,
            } => {
                tally.add("constructed", 1);
                tally.add(
                    &format!("constructed_by_break.{}", pair.replay_break.name()),
                    1,
                );
                tally.add(
                    &format!("constructed_by_last_step.{}", last_step_kind(&pair.steps)),
                    1,
                );
                if abandoned.len() > 1 {
                    tally.add("multi_root_pairs", 1);
                }
                if readable_abandoned.is_empty() {
                    tally.add("constructed_without_readable_root", 1);
                }
                if pair.steps.is_empty()
                    && pair.replay_break == ReplayBreak::FirstCopyOfTheFirstNamedUnit
                {
                    tally.add("depth_zero_unit_first_copy_constructed", 1);
                }
                nodes_with_a_construction.insert(node_key);
            }
            PairOutcome::NotApplicable(reason) => {
                tally.add("not_applicable", 1);
                tally.add(&format!("not_applicable_reason.{reason}"), 1);
            }
            PairOutcome::NotConstructed(reason) => {
                tally.add("not_constructed", 1);
                tally.add(&format!("not_constructed_reason.{reason}"), 1);
            }
        }
    }
    tally.add(
        "constructed_nodes",
        u64::try_from(nodes_with_a_construction.len()).expect("节点数装得进 u64"),
    );
    tally
}

/// 同一个前缀的计数拼成 `名=数,名=数`（名字去掉前缀；BTreeMap 的次序，确定）。
fn fields_with_prefix(tally: &Tally, prefix: &str) -> String {
    let fields = tally
        .values
        .iter()
        .filter_map(|(name, value)| {
            name.strip_prefix(prefix)
                .map(|rest| format!("{rest}={value}"))
        })
        .collect::<Vec<String>>()
        .join(",");
    dash_when_empty(fields)
}

/// 一个对的一行产物（登记第五节「每个构造成立的对一行结果」；未成立的也各一行，写原因）。排序键 = (n1, 步序列文本, 断链方式)。
fn pair_row(arm: &str, geometry: &Geometry, pair: &PairRecord) -> (String, String) {
    let steps_text = step_sequence_text(&pair.steps);
    let sort_key = format!(
        "{:02}/{steps_text}/{}",
        pair.starting_overwrites,
        pair.replay_break.name()
    );
    let variants = dash_when_empty(
        pair.variants
            .iter()
            .map(|record| {
                format!(
                    "{}:{}={}",
                    timeline_root_text(record.root),
                    record.specification.name(),
                    record.result.to_text().replace(' ', ",")
                )
            })
            .collect::<Vec<String>>()
            .join("|"),
    );
    let line = format!(
        "name=r5_pair arm={arm} slots_per_region={} ring={} n1={} steps={steps_text} b={} {} variants={variants}",
        geometry.slots_per_region,
        geometry.journal_ring_bytes,
        pair.starting_overwrites,
        pair.replay_break.name(),
        pair.outcome.to_text()
    );
    (sort_key, line)
}

/// 全部对的产物行，按键序排（登记 5.7 第 4 条：产物里行按键序排，与线程数、单位次序无关）。
pub(super) fn pair_rows_in_key_order(
    arm: &str,
    geometry: &Geometry,
    pairs: &[PairRecord],
) -> Vec<String> {
    let mut rows: Vec<(String, String)> = pairs
        .iter()
        .map(|pair| pair_row(arm, geometry, pair))
        .collect();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows.into_iter().map(|(_, line)| line).collect()
}

fn quantity_line(
    arm: &str,
    domain: &FifthRunDomain,
    name: &str,
    fields: &str,
    verdict: &str,
) -> String {
    format!(
        "name={name} arm={arm} slots_per_region={} ring={} {fields} verdict={verdict}",
        domain.geometry.slots_per_region, domain.geometry.journal_ring_bytes
    )
}

fn zero_holds(value: u64) -> &'static str {
    if value == 0 {
        "holds"
    } else {
        "violated"
    }
}

/// Q0 至 Q7 的汇总行（登记第六节）：每个量各占一行、各报各的判定，不合取。
#[allow(
    clippy::too_many_lines,
    reason = "一个量一行，行与行之间没有共享的中间量，拆开只是把行挪个地方"
)]
pub(super) fn quantity_lines(
    arm: &str,
    domain: &FifthRunDomain,
    nodes: u64,
    transition_failures: u64,
    tallies: [&Tally; 3],
) -> Vec<String> {
    let [construction, account, root_slot] = tallies;
    let line = |name: &str, fields: String, verdict: &str| {
        quantity_line(arm, domain, name, &fields, verdict)
    };
    let mut lines = vec![line(
        "r5_q0",
        format!(
            "nodes={nodes} pairs={} constructed={} not_applicable={} not_constructed={} constructed_nodes={} multi_root_pairs={} constructed_without_readable_root={} depth_zero_unit_first_copy_constructed={} transition_failures={transition_failures} constructed_by_break={} constructed_by_last_step={} not_applicable_reasons={} not_constructed_reasons={}",
            construction.count_of("pairs"),
            construction.count_of("constructed"),
            construction.count_of("not_applicable"),
            construction.count_of("not_constructed"),
            construction.count_of("constructed_nodes"),
            construction.count_of("multi_root_pairs"),
            construction.count_of("constructed_without_readable_root"),
            construction.count_of("depth_zero_unit_first_copy_constructed"),
            fields_with_prefix(construction, "constructed_by_break."),
            fields_with_prefix(construction, "constructed_by_last_step."),
            fields_with_prefix(construction, "not_applicable_reason."),
            fields_with_prefix(construction, "not_constructed_reason."),
        ),
        if construction.count_of("constructed_nodes") > 0 {
            "valid"
        } else {
            "void_not_constructible"
        },
    )];
    lines.push(line(
        "r5_q1",
        format!(
            "account_variants={} mount_errors={} members={} both_copies_not_intercepted={} single_copy_not_intercepted={} twin_missing_or_mount_error={}",
            account.count_of("q1_account_variants"),
            account.count_of("q1_mount_errors"),
            fields_with_prefix(account, "q1_mount_error_member."),
            account.count_of("both_copies_not_intercepted"),
            account.count_of("single_copy_variants.not_intercepted"),
            account.count_of("twin_missing_or_mount_error"),
        ),
        zero_holds(account.count_of("q1_mount_errors")),
    ));
    lines.push(line(
        "r5_q2a",
        format!(
            "both_copies_judged={} count_not_through={}",
            account.count_of("both_copies_judged"),
            account.count_of("q2a_count_not_through")
        ),
        zero_holds(account.count_of("q2a_count_not_through")),
    ));
    lines.push(line(
        "r5_q2b",
        format!(
            "single_copy_intercepted={} count_not_zero_intercepted={} single_copy_not_intercepted={} count_not_zero_not_intercepted={}",
            account.count_of("single_copy_variants.intercepted"),
            account.count_of("q2b_count_not_zero.intercepted"),
            account.count_of("single_copy_variants.not_intercepted"),
            account.count_of("q2b_count_not_zero.not_intercepted"),
        ),
        zero_holds(
            account.count_of("q2b_count_not_zero.intercepted")
                + account.count_of("q2b_count_not_zero.not_intercepted"),
        ),
    ));
    lines.push(line(
        "r5_q2c",
        format!(
            "twin_cells={} twin_count_not_zero={}",
            account.count_of("twin_cells"),
            account.count_of("q2c_twin_count_not_zero")
        ),
        zero_holds(account.count_of("q2c_twin_count_not_zero")),
    ));
    lines.push(line(
        "r5_q3a",
        format!(
            "unshared_cases={} isolated_not_twin={}",
            account.count_of("q3a_unshared_cases"),
            account.count_of("q3a_isolated_not_twin")
        ),
        zero_holds(account.count_of("q3a_isolated_not_twin")),
    ));
    lines.push(line(
        "r5_q3b",
        format!(
            "shared_cases={} isolated_less_than_twin={}",
            account.count_of("q3b_shared_cases"),
            account.count_of("q3b_isolated_less_than_twin")
        ),
        zero_holds(account.count_of("q3b_isolated_less_than_twin")),
    ));
    lines.push(line(
        "r5_q3c",
        format!(
            "single_copy_cases={} isolated_not_twin={} exclusive_slots_empty_cells={}",
            account.count_of("q3c_cases"),
            account.count_of("q3c_isolated_not_twin"),
            account.count_of("exclusive_slots_empty_cells")
        ),
        zero_holds(account.count_of("q3c_isolated_not_twin")),
    ));
    lines.push(line(
        "r5_q4",
        format!(
            "cases={} reuse_more_than_twin={} variants_with_reuse={} reuse_peak={} reuse_final_sum={} reuse_final_maximum={} first_positive_step={}",
            account.count_of("q4_cases"),
            account.count_of("q4_reuse_more_than_twin"),
            account.count_of("q4_variants_with_reuse"),
            account.count_of("q4_reuse_peak"),
            account.count_of("q4_reuse_final_sum"),
            account.count_of("q4_reuse_final_max"),
            optional_number_text(account.lowest_of("q4_first_positive_step")),
        ),
        zero_holds(account.count_of("q4_reuse_more_than_twin")),
    ));
    lines.push(line(
        "r5_q5",
        format!(
            "twin_cells={} twin_cells_with_reuse={} reuse_peak={} first_positive_step={}",
            account.count_of("twin_cells"),
            account.count_of("q5_twin_cells_with_reuse"),
            account.count_of("q5_twin_reuse_peak"),
            optional_number_text(account.lowest_of("q5_first_positive_step")),
        ),
        if account.count_of("q5_twin_cells_with_reuse") == 0 {
            "none"
        } else {
            "found"
        },
    ));
    lines.push(line(
        "r5_q6",
        format!(
            "cells={} variants={} mount_errors={} count_not_zero={} shortfall_equals_exclusive={} shortfall_below_exclusive={} shortfall_positive={} reuse_more_than_twin={} variants_with_reuse={} reuse_peak={} isolated_total_zero={}",
            root_slot.count_of("q6_cells"),
            root_slot.count_of("q6_variants"),
            root_slot.count_of("q6_mount_errors"),
            root_slot.count_of("q6a_count_not_zero"),
            root_slot.count_of("q6b_shortfall_equals_exclusive"),
            root_slot.count_of("q6c_shortfall_below_exclusive"),
            root_slot.count_of("q6d_shortfall_positive"),
            root_slot.count_of("q6e_reuse_more_than_twin"),
            root_slot.count_of("q6e_variants_with_reuse"),
            root_slot.count_of("q6e_reuse_peak"),
            root_slot.count_of("q6f_isolated_total_zero"),
        ),
        zero_holds(root_slot.count_of("q6a_count_not_zero")),
    ));
    lines.push(line(
        "r5_q7",
        format!(
            "isolated_not_twin_unshared={} reuse_more_than_twin={}",
            account.count_of("q3a_isolated_not_twin"),
            account.count_of("q4_reuse_more_than_twin"),
        ),
        if account.count_of("q3a_isolated_not_twin") > 0
            && account.count_of("q4_reuse_more_than_twin") > 0
        {
            "teeth_on_both"
        } else {
            "toothless_on_some"
        },
    ));
    lines.push(line(
        "r5_anchors",
        format!(
            "twin_isolates_fewer_than_exclusive={} continuation_not_at_the_end_txg={}",
            account.count_of("twin_isolates_fewer_than_exclusive"),
            account.count_of("continuation_not_at_the_end_txg"),
        ),
        zero_holds(
            account.count_of("twin_isolates_fewer_than_exclusive")
                + account.count_of("continuation_not_at_the_end_txg"),
        ),
    ));
    lines
}

/// 一格的位置文本（阳性对照行里点名选中了哪一格）。
fn cell_location_text(cell: &RootCell) -> String {
    format!(
        "n1={} steps={} b={} a={}",
        cell.pair.starting_overwrites,
        step_sequence_text(&cell.pair.steps),
        cell.pair.replay_break.name(),
        timeline_root_text(cell.root)
    )
}

/// 阳性对照 PC1-a、PC1-b（账两份读不出）：选取按固定次序（节点的深度优先序、断链方式次序、被抛弃根的集合序）取第一个单元不被其余可读根共享、
/// 独占落点非空的格。today 臂必须出的句子与 skip 臂必须出的句子各自一个字段，判定词只看这一条臂自己的那几句。
pub(super) fn positive_control_on_the_account(
    arm: &str,
    domain: &FifthRunDomain,
    cells: &[RootCell],
    control: &str,
    aspect: LedgerAspect,
) -> (String, &'static str) {
    let selected = cells.iter().find_map(|cell| {
        let twin = cell.twin()?;
        let variant = cell.account(aspect, FaultSeverity::Both, UnreadableForm::ReadFails)?;
        (!variant.shared && twin.exclusive_slots > 0).then_some((cell, twin, variant))
    });
    let Some((cell, twin, variant)) = selected else {
        return (
            quantity_line(
                arm,
                domain,
                "r5_positive_control",
                &format!("control={control}"),
                "not_constructible",
            ),
            "not_constructible",
        );
    };
    let mount_ok = variant.mounted_ok();
    let count_is_through = variant.count == Some(variant.expected_count);
    let isolated_equals_twin = variant.isolated_by_device == twin.isolated_by_device;
    let twin_count_zero = twin.count == Some(0);
    let reuse_equals_twin = variant.reuse_final() == twin.reuse_final();
    let isolated_below_the_twin_everywhere = isolated_less_on_every_device(variant, twin);
    let reuse_more_than_twin = variant.reuse_final() > twin.reuse_final();
    let passed = if arm == "skip" {
        mount_ok && count_is_through && isolated_below_the_twin_everywhere && reuse_more_than_twin
    } else {
        mount_ok && count_is_through && isolated_equals_twin && twin_count_zero && reuse_equals_twin
    };
    let verdict = verdict_word(passed);
    (
        quantity_line(
            arm,
            domain,
            "r5_positive_control",
            &format!(
                "control={control} selected=\"{}\" tag_test_mount_ok={mount_ok} tag_test_count_is_through={count_is_through} tag_test_isolated_equals_twin={isolated_equals_twin} tag_test_twin_count_zero={twin_count_zero} tag_test_reuse_equals_twin={reuse_equals_twin} tag_arm_isolated_less_on_every_device={isolated_below_the_twin_everywhere} tag_arm_reuse_more_than_twin={reuse_more_than_twin}",
                cell_location_text(cell)
            ),
            verdict,
        ),
        verdict,
    )
}

/// 阳性对照 PC1-c（根槽读不出，C554 那一形）：选取 = 被抛弃根里根槽读得出的只有一条、独占落点非空的第一格。两条臂必须出同样的句子。
pub(super) fn positive_control_on_the_root_slot(
    arm: &str,
    domain: &FifthRunDomain,
    cells: &[RootCell],
) -> (String, &'static str) {
    let selected = cells.iter().find_map(|cell| {
        let PairOutcome::Constructed {
            readable_abandoned, ..
        } = &cell.pair.outcome
        else {
            return None;
        };
        let twin = cell.twin()?;
        let variant = cell.root_slot(UnreadableForm::ReadFails)?;
        (readable_abandoned.len() == 1 && twin.exclusive_slots > 0).then_some((cell, twin, variant))
    });
    let Some((cell, twin, variant)) = selected else {
        return (
            quantity_line(
                arm,
                domain,
                "r5_positive_control",
                "control=PC1-c",
                "not_constructible",
            ),
            "not_constructible",
        );
    };
    let mount_ok = variant.mounted_ok();
    let count_is_zero = variant.count == Some(0);
    let shortfall_equals_exclusive = twin.isolated_total() >= variant.isolated_total()
        && twin.isolated_total() - variant.isolated_total() == twin.exclusive_slots;
    let twin_isolates_at_least_exclusive = twin.isolated_total() >= twin.exclusive_slots;
    let passed =
        mount_ok && count_is_zero && shortfall_equals_exclusive && twin_isolates_at_least_exclusive;
    let verdict = verdict_word(passed);
    (
        quantity_line(
            arm,
            domain,
            "r5_positive_control",
            &format!(
                "control=PC1-c selected=\"{}\" tag_today_mount_ok={mount_ok} tag_today_count_is_zero={count_is_zero} tag_today_shortfall_equals_exclusive={shortfall_equals_exclusive} tag_test_twin_isolates_at_least_exclusive={twin_isolates_at_least_exclusive} isolated_total_zero_reported_only={} exclusive_slots={}",
                cell_location_text(cell),
                variant.isolated_total() == 0,
                twin.exclusive_slots
            ),
            verdict,
        ),
        verdict,
    )
}

/// 阳性对照 PC-孪生：每个构造成立格的孪生挂载做成、计数 0。
pub(super) fn positive_control_on_the_twin(
    arm: &str,
    domain: &FifthRunDomain,
    cells: &[RootCell],
) -> (String, &'static str) {
    let twins: Vec<&VariantResult> = cells.iter().filter_map(RootCell::twin).collect();
    if twins.is_empty() {
        return (
            quantity_line(
                arm,
                domain,
                "r5_positive_control",
                "control=PC-twin",
                "not_constructible",
            ),
            "not_constructible",
        );
    }
    let counted_zero = twins.iter().filter(|twin| twin.count == Some(0)).count();
    let passed = counted_zero == twins.len() && twins.len() == cells.len();
    let verdict = verdict_word(passed);
    (
        quantity_line(
            arm,
            domain,
            "r5_positive_control",
            &format!(
                "control=PC-twin tag_test_twins_mounted={} tag_test_twin_count_zero={counted_zero} cells={}",
                twins.len(),
                cells.len()
            ),
            verdict,
        ),
        verdict,
    )
}

/// 复用检测自己分得出差别：孪生的续写末态上，由装置把独占落点里一个落点的字节改写成别的字节，复用数必须 ≥ 1；
/// 不改字节时等于孪生续写里记下的复用数。交回 (不改, 改了, 孪生记下的)。
fn reuse_detection_counts(
    domain: &FifthRunDomain,
    cell: &RootCell,
) -> Result<(u64, u64, u64), String> {
    let twin = cell.twin().ok_or("复用检测要的孪生没有")?;
    let parameters = parameters_for(&domain.geometry);
    let node = replay_steps(
        &domain.geometry,
        cell.pair.starting_overwrites,
        &cell.pair.steps,
    )?;
    let PairConstruction::Constructed(pair) =
        construct_pair(&node, &parameters, cell.pair.replay_break)?
    else {
        return Err("复用检测重做抛弃步没有造成立".to_string());
    };
    let root_context = build_abandoned_root_context(&pair, cell.root)?;
    let InstrumentedMount {
        result, pool_after, ..
    } = run_instrumented_mount_writable(&pair.pool_after_abandonment, &parameters, &[], None, None);
    let mounted = result.map_err(|error| format!("复用检测的孪生挂载：{error:?}"))?;
    let session = Session {
        allocator: mounted.allocator,
        current: mounted.current,
        instance: mounted.output.instance,
    };
    let (_, _, end_state) = continue_with_overwrites(
        &pool_after,
        session,
        &parameters,
        &pair.pool_after_abandonment,
        &root_context.exclusive_placements,
        continuation_end_txg(cell.root, &domain.geometry),
    )?;
    let unchanged = placements_overwritten_between(
        &pair.pool_after_abandonment,
        &end_state,
        &root_context.exclusive_placements,
    );
    let &(device, slot, _) = root_context
        .exclusive_placements
        .iter()
        .next()
        .ok_or("复用检测要的独占落点是空的")?;
    let slot_length = usize::try_from(SLOT_BYTES_LOCAL).expect("槽长装得进 usize");
    let original = pair
        .pool_after_abandonment
        .read(device, slot * SLOT_BYTES_LOCAL, slot_length)
        .ok_or("复用检测读不到独占落点")?;
    let rewritten: Vec<u8> = original.iter().map(|byte| byte ^ 0xff).collect();
    let mut tampered = end_state.clone();
    tampered
        .devices
        .get_mut(&DeviceIdentity(device))
        .ok_or("复用检测的盘不在镜像里")?
        .write(DeviceOffsetInBytes(slot * SLOT_BYTES_LOCAL), &rewritten);
    let changed = placements_overwritten_between(
        &pair.pool_after_abandonment,
        &tampered,
        &root_context.exclusive_placements,
    );
    Ok((unchanged, changed, twin.reuse_final()))
}

/// 阳性对照 PC-复用检测：取第一个独占落点非空的孪生格。
pub(super) fn positive_control_on_the_reuse_detection(
    arm: &str,
    domain: &FifthRunDomain,
    cells: &[RootCell],
) -> (String, &'static str) {
    let Some(cell) = cells
        .iter()
        .find(|cell| cell.twin().is_some_and(|twin| twin.exclusive_slots > 0))
    else {
        return (
            quantity_line(
                arm,
                domain,
                "r5_positive_control",
                "control=PC-reuse-detection",
                "not_constructible",
            ),
            "not_constructible",
        );
    };
    match reuse_detection_counts(domain, cell) {
        Ok((unchanged, changed, recorded)) => {
            let passed = unchanged == recorded && changed >= 1;
            let verdict = verdict_word(passed);
            (
                quantity_line(
                    arm,
                    domain,
                    "r5_positive_control",
                    &format!(
                        "control=PC-reuse-detection selected=\"{}\" tag_test_unchanged_equals_twin_reuse={} tag_test_rewritten_counts_at_least_one={} unchanged={unchanged} rewritten={changed} twin_reuse={recorded}",
                        cell_location_text(cell),
                        unchanged == recorded,
                        changed >= 1
                    ),
                    verdict,
                ),
                verdict,
            )
        }
        Err(error) => (
            quantity_line(
                arm,
                domain,
                "r5_positive_control",
                &format!("control=PC-reuse-detection detail=\"{error}\""),
                "fail",
            ),
            "fail",
        ),
    }
}

/// 阳性对照 PC-空域：这一档造得出被抛弃根（N_c > 0）。
pub(super) fn positive_control_on_the_empty_domain(
    arm: &str,
    domain: &FifthRunDomain,
    constructed_nodes: u64,
) -> (String, &'static str) {
    let verdict = if constructed_nodes > 0 {
        "pass"
    } else {
        "not_constructible"
    };
    (
        quantity_line(
            arm,
            domain,
            "r5_positive_control",
            &format!("control=PC-empty-domain tag_test_constructed_nodes={constructed_nodes}"),
            verdict,
        ),
        verdict,
    )
}

/// 一趟的 sha256 摘要：产物行（不含 `r5_run_parameters`）按产物里的次序用换行拼起来。
pub(super) fn lines_digest(lines: &[String]) -> String {
    singlefs_unit_graph::sha256::sha256_of(lines.join("\n").as_bytes()).hexadecimal()
}

/// 判决行：每个量的失败类计数各一个字段，不合取（合取留给读的人）。
pub(super) fn verdict_line(
    arm: &str,
    domain: &FifthRunDomain,
    tallies: [&Tally; 3],
    controls_not_pass: u64,
) -> String {
    let [construction, account, root_slot] = tallies;
    quantity_line(
        arm,
        domain,
        "verdict",
        &format!(
            "constructed_nodes_positive={} account_mount_errors={} count_not_through={} single_copy_count_not_zero={} twin_count_not_zero={} unshared_isolated_not_twin={} shared_isolated_less_than_twin={} single_copy_isolated_not_twin={} reuse_more_than_twin={} root_slot_count_not_zero={} twin_isolates_fewer_than_exclusive={} continuation_not_at_the_end_txg={} positive_controls_not_pass={controls_not_pass}",
            construction.count_of("constructed_nodes") > 0,
            account.count_of("q1_mount_errors"),
            account.count_of("q2a_count_not_through"),
            account.count_of("q2b_count_not_zero.intercepted")
                + account.count_of("q2b_count_not_zero.not_intercepted"),
            account.count_of("q2c_twin_count_not_zero"),
            account.count_of("q3a_isolated_not_twin"),
            account.count_of("q3b_isolated_less_than_twin"),
            account.count_of("q3c_isolated_not_twin"),
            account.count_of("q4_reuse_more_than_twin"),
            root_slot.count_of("q6a_count_not_zero"),
            account.count_of("twin_isolates_fewer_than_exclusive"),
            account.count_of("continuation_not_at_the_end_txg"),
        ),
        "fields_independent",
    )
}

/// 节点数的松上界（登记 5.3、锚点 A2）：每个起点 Σ 分支数^l，l 从 0 到 L，分支数 ≤ 2 + 3S（覆盖写、挂载与每条可读根的回退）。
pub(super) fn node_count_upper_bound(domain: &FifthRunDomain) -> u64 {
    let branches = 2 + ROOT_RING_REGIONS_LOCAL * domain.geometry.slots_per_region;
    let per_starting_overwrites: u64 = (0..=domain.step_limit)
        .map(|depth| branches.pow(u32::try_from(depth).expect("深度装得进 u32")))
        .sum();
    per_starting_overwrites
        * u64::try_from(domain.starting_overwrites.len()).expect("起点数装得进 u64")
}

/// 一趟枚举交回的东西：产物行（不含 `r5_run_parameters`）与单位数、切深、现判与复用的个数。
pub(super) struct FifthRunReport {
    pub(super) lines: Vec<String>,
    pub(super) cut_depth: u64,
    pub(super) units: usize,
    pub(super) fresh: u64,
    pub(super) reused: u64,
}
