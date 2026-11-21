//! 新底座臂的聚合、各种判定行、汇总行与整趟。

use super::*;

/// 聚合的键：(族，None 是全部族；判定格)。F11 回退之后那一档取族 = F11。
type AggregateKey = (Option<&'static str>, VerdictKey);

/// 第四段拼产物时从各项的行里读回、再聚合的轨迹（判定行由它算，第八节「判定函数把门槛当参数」照用 `status_by_threshold`）。
#[derive(Debug, Default)]
struct SegmentFourAggregate {
    pub(super) trajectories: BTreeMap<(AggregateKey, Reader, usize), [QuantityTrajectory; 6]>,
    differences: BTreeMap<(AggregateKey, Reader, usize), [QuantityTrajectory; 4]>,
    pub(super) provenance_differences: BTreeMap<(AggregateKey, Reader), u64>,
    read_out_empty_trajectories: BTreeMap<Reader, u64>,
}

impl SegmentFourAggregate {
    fn trajectory(&self, key: AggregateKey, reader: Reader, scope: usize) -> [QuantityTrajectory; 6] {
        self.trajectories.get(&(key, reader, scope)).copied().unwrap_or_default()
    }

    fn difference(&self, key: AggregateKey, reader: Reader, scope: usize) -> [QuantityTrajectory; 4] {
        self.differences.get(&(key, reader, scope)).copied().unwrap_or_default()
    }

    fn has_data_for(&self, key: AggregateKey) -> bool {
        self.trajectories.keys().any(|(stored_key, _, _)| *stored_key == key)
    }
}

/// 第四段的读者按产物行里的 `base= shrink= effect= window=` 认回来（两张读者表的并集）。
fn segment_four_reader_of(fields: &BTreeMap<&str, &str>) -> Option<Reader> {
    segment_four_original_path_readers().into_iter().chain(segment_four_stub_path_readers()).find(|reader| {
        fields.get("base") == Some(&reader.base.label())
            && fields.get("shrink") == Some(&reader.shrink.label())
            && fields.get("effect") == Some(&reader.effect.label())
            && fields.get("window") == Some(&reader.window.label())
    })
}

fn scope_position_of(fields: &BTreeMap<&str, &str>) -> usize {
    let scope = fields.get("scope").copied().unwrap_or_default();
    SCAN_SCOPES.iter().position(|known| *known == scope).unwrap_or_else(|| panic!("轨迹行的 scope 认得出：{scope}"))
}

fn aggregate_segment_four(items: &[SegmentFourItem], item_lines: &[Vec<String>]) -> SegmentFourAggregate {
    let mut aggregate = SegmentFourAggregate::default();
    for (item, lines) in items.iter().zip(item_lines) {
        let SegmentFourItem::HistoryCell { history, cell } = item else {
            continue;
        };
        let verdict_key = VerdictKey { cell: *cell, batch: history.batch() };
        let keys = [(None, verdict_key), (Some(history.family()), verdict_key)];
        for line in lines {
            let fields = fields_of(line);
            match fields.get("name").copied() {
                Some("trajectory") => {
                    let Some(reader) = segment_four_reader_of(&fields) else {
                        continue;
                    };
                    let scope = scope_position_of(&fields);
                    let parsed: Vec<QuantityTrajectory> = COUNTED_QUANTITIES.iter().map(|quantity| quantity_trajectory_from(&fields, quantity)).collect();
                    for key in keys {
                        let entry = aggregate.trajectories.entry((key, reader, scope)).or_default();
                        for (stored, observed) in entry.iter_mut().zip(&parsed) {
                            stored.absorb(*observed);
                        }
                    }
                    if scope == SCOPE_ALL && number_field(&fields, "read_out_empty_scans") > 0 {
                        *aggregate.read_out_empty_trajectories.entry(reader).or_default() += 1;
                    }
                }
                Some("difference") => {
                    let Some(reader) = segment_four_reader_of(&fields) else {
                        continue;
                    };
                    let scope = scope_position_of(&fields);
                    let parsed: Vec<QuantityTrajectory> = DIFFERENCE_QUANTITIES.iter().map(|quantity| quantity_trajectory_from(&fields, quantity)).collect();
                    for key in keys {
                        let entry = aggregate.differences.entry((key, reader, scope)).or_default();
                        for (stored, observed) in entry.iter_mut().zip(&parsed) {
                            stored.absorb(*observed);
                        }
                    }
                }
                Some("reader_extras") => {
                    if let (Some(reader), Some(total)) = (segment_four_reader_of(&fields), fields.get("provenance_differences_total")) {
                        for key in keys {
                            *aggregate.provenance_differences.entry((key, reader)).or_default() += total.parse::<u64>().expect("条数是数字");
                        }
                    }
                }
                Some(_) | None => {}
            }
        }
    }
    aggregate
}

/// 第四段的读者：B-on、B-stub、B-tsz 在同一 (X2, X3, W) 上。
pub(super) const fn reader_with(base: BaseArm, shrink: ShrinkArm, effect: UnlinkEffectArm, window: ReadableWindow) -> Reader {
    Reader { base, shrink, effect, window }
}

const fn shrink_axis_resurrection_question(shrink: ShrinkArm) -> &'static str {
    match shrink {
        ShrinkArm::RecordSize => "Q-X2-1",
        ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => "Q-X2-2",
    }
}

const fn shrink_axis_misjudged_death_question(shrink: ShrinkArm) -> &'static str {
    match shrink {
        ShrinkArm::RecordSize => "Q-X2-3",
        ShrinkArm::RangeTombstone | ShrinkArm::ObjectGateOnly => "Q-X2-4",
    }
}

/// 一个读者在 Q-X2-1 至 Q-X2-5 口径下的判定（与第二段 `segment_two_verdict_lines` 同门槛同量）：(子问题, 状态)。
fn shrink_axis_statuses(trajectories_all: &[QuantityTrajectory; 6], trajectories_outside: &[QuantityTrajectory; 6], trajectories_inside: &[QuantityTrajectory; 6], reader: Reader) -> Vec<(&'static str, &'static str)> {
    let mut statuses = vec![
        (shrink_axis_resurrection_question(reader.shrink), status_by_threshold(trajectories_all[QUANTITY_EXTENT_RESURRECTED].peak, "resurrects")),
        (shrink_axis_misjudged_death_question(reader.shrink), status_by_threshold(trajectories_outside[QUANTITY_EXTENT_MISSED].peak, "misjudges_death")),
    ];
    if reader.effect != UnlinkEffectArm::AtIntent {
        statuses.push(("Q-X2-5", status_by_threshold(trajectories_inside[QUANTITY_EXTENT_MISSED].peak, "misjudges_in_batches")));
    }
    statuses
}

/// 一条新底座读者在一个判定格、一档 W 上的判定行（Q-Bs-1 至 Q-Bs-4 或 Q-Bt-1 至 Q-Bt-4；修订 2 之七）。
fn new_base_verdict_lines(
    aggregate: &SegmentFourAggregate,
    verdict_key: VerdictKey,
    reader: Reader,
    statuses: &mut Vec<JudgedStatus>,
    differing_shrink_axis_cells: &mut Vec<String>,
) -> Vec<String> {
    let (prefix, base_label) = match reader.base {
        BaseArm::TombstonesOnly => ("Q-Bt", "B-tsz"),
        BaseArm::ZeroRecordStubs
        | BaseArm::ZeroRecordStubsIgnoringLineage
        | BaseArm::RetirementOnLineage
        | BaseArm::WithoutRetirement
        | BaseArm::RetirementIgnoringLineage => ("Q-Bs", "B-stub"),
    };
    let all_key: AggregateKey = (None, verdict_key);
    let rollback_key: AggregateKey = (Some("F11"), verdict_key);
    let paired = reader_with(BaseArm::RetirementOnLineage, reader.shrink, reader.effect, reader.window);
    let key_label = format!("borrow_arm={} capacity={} theta={} batch={}", verdict_key.cell.borrow_arm.label(), verdict_key.cell.leaf_capacity, verdict_key.cell.merge_threshold, verdict_key.batch);
    let arms = format!("shrink={} effect={}", reader.shrink.label(), reader.effect.label());
    let mut lines = Vec::new();
    let mut judged = |question_and_arms: String, status: &'static str| {
        statuses.push(JudgedStatus {
            question_and_arms,
            borrow_arm: verdict_key.cell.borrow_arm,
            leaf_capacity: verdict_key.cell.leaf_capacity,
            theta_class: theta_class_of(verdict_key.cell),
            batch: verdict_key.batch,
            window: reader.window,
            status,
        });
    };
    let new_all = aggregate.trajectory(all_key, reader, SCOPE_ALL);
    let base_all = aggregate.trajectory(all_key, paired, SCOPE_ALL);
    let difference_all = aggregate.difference(all_key, reader, SCOPE_ALL);
    // 复活：容器 r、inode r、extent r 差（新臂多出的方向）。B-tsz 的容器 r 是定义的推论，只报数（A12 钉）。
    let resurrection = [
        ("container_r", new_all[QUANTITY_CONTAINER_RESURRECTED], base_all[QUANTITY_CONTAINER_RESURRECTED].peak, "resurrects_containers"),
        ("inode_r", new_all[QUANTITY_INODE_RESURRECTED], base_all[QUANTITY_INODE_RESURRECTED].peak, "resurrects_records"),
        ("extent_r_more_than_b_on", difference_all[DIFFERENCE_EXTENT_RESURRECTED_MORE], 0, "resurrects_more_units"),
    ];
    for (quantity, trajectory, base_peak, positive) in resurrection {
        let reported_only = reader.base == BaseArm::TombstonesOnly && quantity == "container_r";
        let status = if reported_only { "reported" } else { status_by_threshold(trajectory.peak, positive) };
        if !reported_only {
            judged(format!("{prefix}-1 {arms} quantity={quantity}"), status);
        }
        lines.push(format!(
            "name=verdict question={prefix}-1 base={base_label} {key_label} window={} {arms} quantity={quantity} {} b_on_peak={base_peak} status={status}",
            reader.window.label(),
            trajectory.fields(quantity),
        ));
    }
    // 误判死亡：「全部」（不在 unlink 途中的扫描）与「F11 回退之后」两档。
    for (tier, key, scope) in [("all", all_key, SCOPE_OUTSIDE_UNLINK), ("after_rollback", rollback_key, SCOPE_AFTER_ROLLBACK_OUTSIDE_UNLINK)] {
        let has_data = aggregate.has_data_for(key);
        let new_tier = aggregate.trajectory(key, reader, scope);
        let base_tier = aggregate.trajectory(key, paired, scope);
        let difference_tier = aggregate.difference(key, reader, scope);
        let misjudged = [
            ("container_m", new_tier[QUANTITY_CONTAINER_MISSED], base_tier[QUANTITY_CONTAINER_MISSED].peak, "misjudges_container_death"),
            ("inode_m", new_tier[QUANTITY_INODE_MISSED], base_tier[QUANTITY_INODE_MISSED].peak, "misjudges_record_death"),
            ("extent_m_more_than_b_on", difference_tier[DIFFERENCE_EXTENT_MISSED_MORE], 0, "misjudges_more_units"),
        ];
        for (quantity, trajectory, base_peak, positive) in misjudged {
            let status = if has_data { status_by_threshold(trajectory.peak, positive) } else { "not_run" };
            if has_data {
                judged(format!("{prefix}-2 {arms} tier={tier} quantity={quantity}"), status);
            }
            lines.push(format!(
                "name=verdict question={prefix}-2 base={base_label} {key_label} window={} {arms} tier={tier} quantity={quantity} {} b_on_peak={base_peak} status={status}",
                reader.window.label(),
                trajectory.fields(quantity),
            ));
        }
    }
    // unlink 途中漏活：E-I 只报数（A7），E-L 两臂峰值 > 0 判漏活；并排 B-on 同一格的判定。
    let new_inside = aggregate.trajectory(all_key, reader, SCOPE_INSIDE_UNLINK);
    let base_inside = aggregate.trajectory(all_key, paired, SCOPE_INSIDE_UNLINK);
    for (quantity, position) in [("inode_m", QUANTITY_INODE_MISSED), ("extent_m", QUANTITY_EXTENT_MISSED)] {
        let (status, base_status) = if reader.effect == UnlinkEffectArm::AtIntent {
            ("reported", "reported")
        } else {
            (status_by_threshold(new_inside[position].peak, "leaks"), status_by_threshold(base_inside[position].peak, "leaks"))
        };
        if reader.effect != UnlinkEffectArm::AtIntent {
            judged(format!("{prefix}-3 {arms} quantity={quantity}"), status);
        }
        lines.push(format!(
            "name=verdict question={prefix}-3 base={base_label} {key_label} window={} {arms} quantity={quantity} {} b_on_status={base_status} status={status}",
            reader.window.label(),
            new_inside[position].fields(&format!("{quantity}_inside_unlink")),
        ));
    }
    // X2 在新底座下的判定与 B-on 下并排（Q-Bs-4 / Q-Bt-4）。
    let new_shrink_axis_statuses = shrink_axis_statuses(&new_all, &aggregate.trajectory(all_key, reader, SCOPE_OUTSIDE_UNLINK), &new_inside, reader);
    let base_shrink_axis_statuses = shrink_axis_statuses(&base_all, &aggregate.trajectory(all_key, paired, SCOPE_OUTSIDE_UNLINK), &base_inside, paired);
    for ((sub_question, new_status), (_, base_status)) in new_shrink_axis_statuses.iter().zip(&base_shrink_axis_statuses) {
        let differs = new_status != base_status;
        judged(format!("{prefix}-4 {arms} sub_question={sub_question}"), new_status);
        if differs {
            differing_shrink_axis_cells.push(format!("{prefix}-4:{sub_question}:{}:{}:{}:{}:{}:{}", base_label, verdict_key.cell.label().replace(' ', ","), verdict_key.batch, reader.window.label(), reader.shrink.label(), reader.effect.label()));
        }
        lines.push(format!(
            "name=verdict question={prefix}-4 base={base_label} {key_label} window={} {arms} sub_question={sub_question} new_base_status={new_status} b_on_status={base_status} differs={differs}",
            reader.window.label()
        ));
    }
    if reader.base == BaseArm::TombstonesOnly {
        lines.push(format!(
            "name=verdict question=Q-Bt-2 base=B-tsz {key_label} window={} {arms} quantity=provenance_differences total={} judgement=reported",
            reader.window.label(),
            aggregate.provenance_differences.get(&(all_key, reader)).copied().unwrap_or(0)
        ));
    }
    lines
}

/// 交回时的并排表（修订 2 之七末段）：每个 (族 × 写路径格 × 批 × X2 × X3 × W) 一行，三个底座的复活（容器 r、inode r、extent r 峰值）
/// 与误判死亡（不在途中的容器 m、inode m、extent m 峰值）并排；B-stub 只在 N1 的格、B-tsz 只在 E-I 有。
fn side_by_side_lines(aggregate: &SegmentFourAggregate, families: &[&'static str], verdict_keys: &[VerdictKey]) -> Vec<String> {
    let peaks = |key: AggregateKey, reader: Reader| {
        let all = aggregate.trajectory(key, reader, SCOPE_ALL);
        let outside = aggregate.trajectory(key, reader, SCOPE_OUTSIDE_UNLINK);
        format!(
            "{}/{}/{}:{}/{}/{}",
            all[QUANTITY_CONTAINER_RESURRECTED].peak,
            all[QUANTITY_INODE_RESURRECTED].peak,
            all[QUANTITY_EXTENT_RESURRECTED].peak,
            outside[QUANTITY_CONTAINER_MISSED].peak,
            outside[QUANTITY_INODE_MISSED].peak,
            outside[QUANTITY_EXTENT_MISSED].peak
        )
    };
    let mut lines = Vec::new();
    for family in families {
        for verdict_key in verdict_keys {
            let key: AggregateKey = (Some(family), *verdict_key);
            if !aggregate.has_data_for(key) {
                continue;
            }
            for window in READABLE_WINDOWS {
                for effect in UNLINK_EFFECT_ARMS {
                    for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                        let stub = if verdict_key.cell.borrow_arm == BorrowArm::RaiseSeparator {
                            peaks(key, reader_with(BaseArm::ZeroRecordStubs, shrink, effect, window))
                        } else {
                            "n/a".to_string()
                        };
                        let tombstones_only = if effect == UnlinkEffectArm::AtIntent {
                            peaks(key, reader_with(BaseArm::TombstonesOnly, shrink, effect, window))
                        } else {
                            "n/a".to_string()
                        };
                        lines.push(format!(
                            "name=side_by_side family={family} {} batch={} window={} shrink={} effect={} resurrected_c_i_e_and_misjudged_c_i_e b_on={} b_stub={stub} b_tsz={tombstones_only}",
                            verdict_key.cell.label(),
                            verdict_key.batch,
                            window.label(),
                            shrink.label(),
                            effect.label(),
                            peaks(key, reader_with(BaseArm::RetirementOnLineage, shrink, effect, window)),
                        ));
                    }
                }
            }
        }
    }
    lines
}

/// Q-Bs-5、Q-Bs-6 按 (族, 写路径格, 批) 聚合的数（修订 2 之七，报数不设门槛）。
fn stub_leaf_verdict_lines(items: &[SegmentFourItem], item_lines: &[Vec<String>]) -> Vec<String> {
    #[derive(Default)]
    struct StubLeafTotals {
        segments: u64,
        versions: u64,
        versions_with_an_empty_inode_leaf: u64,
        stubs_written: u64,
        leaves_peak_largest: u64,
        stubs_last_largest: u64,
        stub_share_last_ppm_largest: u64,
        versions_over_the_root: u64,
        first_stub_publish_smallest: Option<u64>,
    }
    let mut totals: BTreeMap<(&'static str, VerdictKey), StubLeafTotals> = BTreeMap::new();
    for (item, lines) in items.iter().zip(item_lines) {
        let SegmentFourItem::HistoryCell { history, cell } = item else {
            continue;
        };
        for line in lines.iter().filter(|line| line.starts_with("name=stub_leaves ")) {
            let fields = fields_of(line);
            let entry = totals.entry((history.family(), VerdictKey { cell: *cell, batch: history.batch() })).or_default();
            entry.segments += 1;
            entry.versions += number_field(&fields, "versions");
            entry.versions_with_an_empty_inode_leaf += number_field(&fields, "versions_with_an_empty_inode_leaf");
            entry.stubs_written += number_field(&fields, "stubs_written");
            entry.leaves_peak_largest = entry.leaves_peak_largest.max(number_field(&fields, "leaves_peak"));
            entry.stubs_last_largest = entry.stubs_last_largest.max(number_field(&fields, "stubs_last"));
            entry.stub_share_last_ppm_largest = entry.stub_share_last_ppm_largest.max(number_field(&fields, "stub_share_ppm_last"));
            entry.versions_over_the_root += number_field(&fields, "versions_over_the_root");
            if let Some(first) = fields.get("first_stub_publish").and_then(|value| value.parse::<u64>().ok()) {
                entry.first_stub_publish_smallest = Some(entry.first_stub_publish_smallest.map_or(first, |smallest| smallest.min(first)));
            }
        }
    }
    let mut lines = Vec::new();
    for ((family, verdict_key), entry) in &totals {
        let share = (entry.versions_with_an_empty_inode_leaf * 1_000_000).checked_div(entry.versions).unwrap_or(0);
        lines.push(format!(
            "name=verdict question=Q-Bs-5 family={family} {} batch={} segments={} stubs_written={} leaves_peak_largest={} stubs_last_largest={} stub_share_last_ppm_largest={} versions_over_the_root={} judgement=reported",
            verdict_key.cell.label(),
            verdict_key.batch,
            entry.segments,
            entry.stubs_written,
            entry.leaves_peak_largest,
            entry.stubs_last_largest,
            entry.stub_share_last_ppm_largest,
            entry.versions_over_the_root,
        ));
        lines.push(format!(
            "name=verdict question=Q-Bs-6 family={family} {} batch={} segments={} versions={} versions_with_an_empty_inode_leaf={} share_ppm={share} first_stub_publish_smallest={} judgement=reported",
            verdict_key.cell.label(),
            verdict_key.batch,
            entry.segments,
            entry.versions,
            entry.versions_with_an_empty_inode_leaf,
            entry.first_stub_publish_smallest.map_or_else(|| "none".to_string(), |first| first.to_string()),
        ));
    }
    lines
}

/// 第四段从各项的行拼出整份产物：配置、常量锚点、各项的行（按项的次序）、判定行、并排表、Q-Bs-5/6、几何敏感性、断言总账、汇总。
/// 只读各项交回的行，不读项是现判的还是复用的（续跑与一次跑完的产物逐字节相同）。
pub(super) fn assemble_segment_four(items: &[SegmentFourItem], item_lines: &[Vec<String>]) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=four work_items={} original_path_readers={} stub_path_readers={} stub_path_borrow_arms=N1 provenance=P-C capacities=233,16 thetas=0,quarter,half windows=inf,23 ring_window={READABLE_WINDOW_RING_PUBLISHES}",
        items.len(),
        segment_four_original_path_readers().len(),
        segment_four_stub_path_readers().len(),
    )];
    lines.extend(constant_anchor_lines());
    for item_lines_of_one in item_lines {
        lines.extend(item_lines_of_one.iter().cloned());
    }
    let aggregate = aggregate_segment_four(items, item_lines);
    let mut verdict_keys: Vec<VerdictKey> = Vec::new();
    let mut families: Vec<&'static str> = Vec::new();
    for item in items {
        if let SegmentFourItem::HistoryCell { history, cell } = item {
            let key = VerdictKey { cell: *cell, batch: history.batch() };
            if !verdict_keys.contains(&key) {
                verdict_keys.push(key);
            }
            if !families.contains(&history.family()) {
                families.push(history.family());
            }
        }
    }
    verdict_keys.sort();
    let mut statuses = Vec::new();
    let mut differing_shrink_axis_cells = Vec::new();
    for verdict_key in &verdict_keys {
        for window in READABLE_WINDOWS {
            if verdict_key.cell.borrow_arm == BorrowArm::RaiseSeparator {
                for effect in UNLINK_EFFECT_ARMS {
                    for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                        let reader = reader_with(BaseArm::ZeroRecordStubs, shrink, effect, window);
                        lines.extend(new_base_verdict_lines(&aggregate, *verdict_key, reader, &mut statuses, &mut differing_shrink_axis_cells));
                    }
                }
            }
            for shrink in [ShrinkArm::RecordSize, ShrinkArm::RangeTombstone] {
                let reader = reader_with(BaseArm::TombstonesOnly, shrink, UnlinkEffectArm::AtIntent, window);
                lines.extend(new_base_verdict_lines(&aggregate, *verdict_key, reader, &mut statuses, &mut differing_shrink_axis_cells));
            }
        }
    }
    lines.push(format!(
        "name=verdict question=FL-Bx x2_cells_whose_judgement_changes_with_the_base={} cells={}",
        differing_shrink_axis_cells.len(),
        if differing_shrink_axis_cells.is_empty() { "none".to_string() } else { differing_shrink_axis_cells.join(";") }
    ));
    lines.extend(side_by_side_lines(&aggregate, &families, &verdict_keys));
    lines.extend(stub_leaf_verdict_lines(items, item_lines));
    lines.extend(stability_lines(&statuses));
    lines.extend(segment_four_summary_lines(items, item_lines, &aggregate));
    lines
}

/// 第四段的断言总账与汇总行（V2、V3、V4、V5、V6、V8、V-Bs1、V-Bs3、V-Bt1 与阳性对照的覆盖）。
fn segment_four_summary_lines(items: &[SegmentFourItem], item_lines: &[Vec<String>], aggregate: &SegmentFourAggregate) -> Vec<String> {
    let mut assertion_totals: BTreeMap<(String, String), (u64, u64)> = BTreeMap::new();
    let mut runs_not_identical = 0_u64;
    let mut last_scan_paths_disagree = 0_u64;
    let mut pairing_fails = 0_u64;
    let mut stub_read_mismatch_segments = 0_u64;
    let mut does_not_fit = 0_u64;
    let mut uncovered_many_objects_cells = 0_u64;
    let mut rollbacks_too_far = 0_u64;
    let mut rollback_events_missing = 0_u64;
    let mut controls: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
    let mut literal_failures: BTreeMap<String, u64> = BTreeMap::new();
    let mut reversed_provenance_positive = 0_u64;
    let mut reversed_provenance_positive_mid = 0_u64;
    let mut stub_erasure_families: BTreeSet<String> = BTreeSet::new();
    let mut dead_record_families: BTreeSet<String> = BTreeSet::new();
    let mut construction = Vec::new();
    for (item, lines) in items.iter().zip(item_lines) {
        let family = match item {
            SegmentFourItem::HistoryCell { history, .. } => history.family(),
            SegmentFourItem::DirectRollbackConstruction => "construction",
        };
        for line in lines {
            let fields = fields_of(line);
            match fields.get("name").copied() {
                Some("simulation") => {
                    runs_not_identical += u64::from(fields.get("two_runs_identical") == Some(&"false"));
                    last_scan_paths_disagree += u64::from(fields.get("last_scan_paths_agree") == Some(&"false"));
                    pairing_fails += u64::from(fields.get("pairing_holds") == Some(&"false"));
                    stub_read_mismatch_segments += u64::from(fields.get("stub_read_mismatch_scans").is_some_and(|value| *value != "0"));
                    does_not_fit += u64::from(fields.get("does_not_fit_at").is_some_and(|value| *value != "none"));
                    rollbacks_too_far += u64::from(fields.get("rollback_too_far") == Some(&"true"));
                    rollback_events_missing += u64::from(fields.get("rollback_event_found") == Some(&"false"));
                    let original_many_objects_with_merges = family == "F10" && fields.get("path") == Some(&"original") && fields.get("theta") != Some(&"0");
                    if original_many_objects_with_merges {
                        let covered = ["borrows", "moved_then_shortened", "moved_then_unlinked"].iter().all(|key| number_field(&fields, key) > 0);
                        uncovered_many_objects_cells += u64::from(!covered);
                    }
                }
                Some("positive_control") => {
                    let identifier = fields.get("id").copied().unwrap_or_default().to_string();
                    if matches!(identifier.as_str(), "PC-Bs2" | "PC-Bt3") {
                        construction.push(format!("{}={}", identifier, fields.get("holds").copied().unwrap_or("missing")));
                        continue;
                    }
                    let entry = controls.entry(identifier.clone()).or_default();
                    entry.0 += 1;
                    entry.1 += u64::from(fields.get("paths_agree") != Some(&"true"));
                    entry.2 += u64::from(fields.get("closed_form_holds") != Some(&"true"));
                    if fields.get("registered_literal_holds") == Some(&"false") {
                        *literal_failures.entry(identifier.clone()).or_default() += 1;
                    }
                    if identifier == "PC-Bt2" && fields.get("hand_extent_r_z").and_then(|value| value.parse::<i64>().ok()).is_some_and(|hand| hand > 0) {
                        reversed_provenance_positive += 1;
                    }
                    if identifier == "PC-Bt2-mid" && fields.get("hand_extent_r_z").and_then(|value| value.parse::<i64>().ok()).is_some_and(|hand| hand > 0) {
                        reversed_provenance_positive_mid += 1;
                    }
                    let window_taken = fields.get("window_taken").copied().unwrap_or_default();
                    if identifier == "PC-Bs1" {
                        stub_erasure_families.insert(format!("{family}/W{window_taken}"));
                    }
                    if identifier == "PC-Bt1" {
                        dead_record_families.insert(format!("{family}/W{window_taken}"));
                    }
                }
                Some("item_assertion") => {
                    let key = (fields.get("path").copied().unwrap_or_default().to_string(), fields.get("assertion").copied().unwrap_or_default().to_string());
                    let entry = assertion_totals.entry(key).or_default();
                    entry.0 += number_field(&fields, "judged");
                    entry.1 += number_field(&fields, "violated");
                }
                Some(_) | None => {}
            }
        }
    }
    let mut lines = Vec::new();
    let mut every_assertion_holds = true;
    for ((path, assertion), (judged, violated)) in &assertion_totals {
        every_assertion_holds &= *violated == 0;
        lines.push(format!("name=assertion path={path} assertion={assertion} judged={judged} violated={violated}"));
    }
    for (identifier, (count, paths_disagree, closed_form_fails)) in &controls {
        lines.push(format!(
            "name=positive_control_tally id={identifier} count={count} paths_disagree={paths_disagree} closed_form_fails={closed_form_fails} registered_literal_fails={}",
            literal_failures.get(identifier).copied().unwrap_or(0)
        ));
    }
    let new_base_read_out_empty: u64 = aggregate
        .read_out_empty_trajectories
        .iter()
        .filter(|(reader, _)| matches!(reader.base, BaseArm::ZeroRecordStubs | BaseArm::TombstonesOnly))
        .map(|(_, count)| *count)
        .sum();
    lines.push(format!(
        "name=verdict question=summary v2_paths_disagree={} v2_closed_form_fails={} v3_last_scan_paths_disagree={last_scan_paths_disagree} v4_runs_not_identical={runs_not_identical} v5_anchors_hold={every_assertion_holds} v6_f10_cells_without_coverage={uncovered_many_objects_cells} v8_new_base_trajectories_with_empty_reads={new_base_read_out_empty} v_bs1_pairing_fails={pairing_fails} v_bs3_segments_with_stub_read_mismatches={stub_read_mismatch_segments} does_not_fit_segments={does_not_fit} f11_rollbacks_too_far={rollbacks_too_far} f11_events_missing={rollback_events_missing} pc_bt2_segments_with_a_positive_hand_count={reversed_provenance_positive} pc_bt2_mid_segments_with_a_positive_hand_count={reversed_provenance_positive_mid} pc_bs1_taken={} pc_bt1_taken={} construction={}",
        controls.values().map(|(_, paths_disagree, _)| paths_disagree).sum::<u64>(),
        controls.values().map(|(_, _, closed_form_fails)| closed_form_fails).sum::<u64>(),
        if stub_erasure_families.is_empty() { "none".to_string() } else { stub_erasure_families.into_iter().collect::<Vec<String>>().join(",") },
        if dead_record_families.is_empty() { "none".to_string() } else { dead_record_families.into_iter().collect::<Vec<String>>().join(",") },
        construction.join(","),
    ));
    lines
}

/// 第四段入口：补算没落盘的项，997 项齐了拼产物。成功句打到 stderr（产物与续跑无关）：
/// 「现判、复用、键对不上」三个数（登记修订 4 之五），键对不上 > 0 时下一行写变了的单元。
pub(super) fn run_segment_four(threads: usize, options: &SegmentFourOptions) -> Vec<String> {
    let items = segment_four_items();
    let outcome = compute_or_reuse_the_items(&items, threads, options);
    eprintln!(
        "segment-four: 现判 {}、复用 {}、键对不上 {} 项（共 {} 项，库 prune-server，经 device-code-key.py 转交）",
        outcome.computed,
        outcome.reused,
        outcome.mismatched,
        items.len()
    );
    if let Some(sentence) = &outcome.changed_units {
        eprintln!("segment-four: {sentence}");
    }
    assemble_segment_four(&items, &outcome.item_lines)
}
