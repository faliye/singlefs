//! 只跑写路径的那一段：工作项、产物行、停机核对、并行与判定行。

use super::*;

/// 第一段的一个工作项：一段历史 × 一个容量 × 一个 θ，N1、N2 各跑两遍（第十一节 V4 的逐遍比）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SegmentOneItem {
    pub(super) history: History,
    pub(super) leaf_capacity: u64,
    pub(super) merge_threshold: u64,
}

#[derive(Clone, Debug)]
struct SegmentOneResult {
    pub(super) item: SegmentOneItem,
    raise_separator: WritePathRun,
    renew_identity: WritePathRun,
    pair: PairStatistics,
    pub(super) two_runs_identical: bool,
}

fn segment_one_items() -> Vec<SegmentOneItem> {
    let mut items = Vec::new();
    for history in histories_replayed_from_the_e177_segment_one().into_iter().chain(many_objects_with_data_histories()) {
        for leaf_capacity in LEAF_CAPACITIES {
            for merge_threshold in merge_thresholds_of(leaf_capacity) {
                items.push(SegmentOneItem { history, leaf_capacity, merge_threshold });
            }
        }
    }
    items
}

fn run_segment_one_item(item: SegmentOneItem) -> SegmentOneResult {
    let cell_of = |borrow_arm| WritePathCell { borrow_arm, leaf_capacity: item.leaf_capacity, merge_threshold: item.merge_threshold };
    let mut runs = Vec::new();
    let mut two_runs_identical = true;
    for borrow_arm in BORROW_ARMS {
        let first = run_write_path(item.history, cell_of(borrow_arm));
        let second = run_write_path(item.history, cell_of(borrow_arm));
        two_runs_identical &= first == second;
        runs.push(first);
    }
    let renew_identity = runs.pop().expect("N2 那一遍");
    let raise_separator = runs.pop().expect("N1 那一遍");
    let pair = pair_statistics(&raise_separator, &renew_identity);
    SegmentOneResult { item, raise_separator, renew_identity, pair, two_runs_identical }
}

fn write_path_line(item: SegmentOneItem, borrow_arm: BorrowArm, run: &WritePathRun) -> String {
    let cell = WritePathCell { borrow_arm, leaf_capacity: item.leaf_capacity, merge_threshold: item.merge_threshold };
    let today = TombstoneStream::WholeAtIntent.position();
    format!(
        "name=write_path {} {} publishes={} refused_steps={} borrows={} absorbs={} drops={} moved_then_shortened={} moved_then_unlinked={} retirement_records={} units_data={} units_inode_leaf={} units_inode_root={} units_retirement_container={} units_retirement_root={} units_tombstone_container_today={} units_tombstone_root_today={} v7_versions={} v7_i913_red={} v7_today_i94_red={} v7_n1_i94_red={} v7_i912_red={} k6_judged={} k6_violations={} a1_violations={} a8_judged={} a8_violations={} does_not_fit_at={}",
        item.history.label(),
        cell.label(),
        run.publishes,
        run.refused_steps,
        run.borrows,
        run.absorbs,
        run.drops,
        run.moved_then_shortened,
        run.moved_then_unlinked,
        run.written.retirement_records,
        run.written.data_units,
        run.written.inode_leaf_containers,
        run.written.inode_roots,
        run.written.retirement_containers,
        run.written.retirement_roots,
        run.written.tombstone_containers[today],
        run.written.tombstone_roots[today],
        run.self_check.versions,
        run.self_check.leaves_hold_a_record_red,
        run.self_check.today_container_numbers_red,
        run.self_check.raised_separator_container_numbers_red,
        run.self_check.separators_red,
        run.assertions.judged_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged"),
        run.assertions.violations_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged"),
        run.assertions.violations_of("A1_history_publishes") + run.assertions.violations_of("A1_history_refused_steps"),
        run.assertions.judged_of("A8_retirements_equal_identity_disappearances"),
        run.assertions.violations_of("A8_retirements_equal_identity_disappearances"),
        run.does_not_fit_at.map_or_else(|| "none".to_string(), |txg| txg.to_string()),
    )
}

fn borrow_cost_line(result: &SegmentOneResult) -> String {
    let pair = result.pair;
    format!(
        "name=borrow_cost {} capacity={} theta={} borrow_publishes={} {} {} {} {} differences_outside_borrows={} borrow_kinds_disagree={} n2_counted_units={} n1_counted_units={} n2_extra_share_ppm={} user_changes_identical={} publish_counts_identical={} streams_identical={} two_runs_identical={}",
        result.item.history.label(),
        result.item.leaf_capacity,
        result.item.merge_threshold,
        pair.borrow_publishes,
        pair.extra_code_three_per_borrow.fields("extra_code3_per_borrow"),
        pair.extra_code_two_per_borrow.fields("extra_code2_per_borrow"),
        pair.retirement_records_per_borrow_renew_identity.fields("n2_retirements_per_borrow"),
        pair.retirement_records_per_borrow_raise_separator.fields("n1_retirements_per_borrow"),
        pair.publishes_outside_borrows_with_a_difference,
        pair.borrow_kinds_disagree,
        pair.renew_identity_counted_units,
        pair.raise_separator_counted_units,
        pair.extra_share_in_parts_per_million(),
        pair.user_changes_identical,
        pair.publish_counts_identical,
        pair.streams_identical,
        result.two_runs_identical,
    )
}

/// 5.6 全表（两种键布局）：每一形三个谓词的红绿与路由对不对。
#[allow(clippy::type_complexity, reason = "这个元组类型只在这一处出现，起类型别名只多一个只用一次的名字")]
fn shape_table_lines() -> (Vec<String>, Vec<(&'static str, &'static str, String, PredicateVerdicts)>) {
    let mut lines = Vec::new();
    let mut verdicts = Vec::new();
    for named in catalogue_of_shapes() {
        for (layout, shape) in [("dense", named.shape.clone()), ("sparse", sparse_layout_of(&named.shape))] {
            let judged = judge_the_shape(&shape);
            let leaves: Vec<String> = shape
                .leaves
                .iter()
                .map(|leaf| {
                    let keys: Vec<String> = leaf.record_keys.iter().map(u64::to_string).collect();
                    format!("{}:{}", leaf.container_number, keys.join("/"))
                })
                .collect();
            let separators: Vec<String> = shape.separators.iter().map(u64::to_string).collect();
            lines.push(format!(
                "name=shape family={} shape={} layout={layout} leaves={} separators={} today_i94={} n1_i94={} i912={} routing={}",
                named.family,
                named.name,
                leaves.join(","),
                separators.join(","),
                red_or_green(judged.today_container_numbers_hold),
                red_or_green(judged.raised_separator_container_numbers_hold),
                red_or_green(judged.separators_hold),
                if judged.routing_holds { "right" } else { "wrong" },
            ));
            verdicts.push((named.family, layout, named.name.clone(), judged));
        }
    }
    (lines, verdicts)
}

/// A6 的一借小历史 MB：容量 16、θ = 8，建 inode 1..32（两片各 16），按升序 unlink 2..10。
fn run_the_single_borrow_micro_history(borrow_arm: BorrowArm) -> Simulation {
    let cell = WritePathCell { borrow_arm, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: TEST_LEAF_CAPACITY / 2 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    simulation.apply(UserChange::NewInodes { count: 31 });
    for object in 2..=10 {
        simulation.apply(UserChange::Unlink { object });
    }
    simulation
}

/// A6 逐条：恰好一次借、挪 5 条、左 12 右 11、右片最小 key 22；N2 − N1 = 码 3 + 1、码 2 + 1、退役记录 + 1；N2 下右片新号 22、出生代 = 那次 txg。
fn single_borrow_anchor_holds() -> (bool, String) {
    let raise_separator = run_the_single_borrow_micro_history(BorrowArm::RaiseSeparator);
    let renew_identity = run_the_single_borrow_micro_history(BorrowArm::RenewIdentity);
    let borrows: Vec<(usize, &PublishRecord)> = renew_identity
        .publish_records
        .iter()
        .enumerate()
        .filter(|(_, record)| matches!(record.merge, MergeAction::BorrowedFromTheRightNeighbour { .. }))
        .collect();
    let other_merges = renew_identity
        .publish_records
        .iter()
        .filter(|record| !matches!(record.merge, MergeAction::NoMerge | MergeAction::BorrowedFromTheRightNeighbour { .. }))
        .count();
    let Some((position, borrow)) = borrows.first().copied() else {
        return (false, "no_borrow".to_string());
    };
    let raised = &raise_separator.publish_records[position];
    let leaves = &renew_identity.version.leaves;
    let sizes: Vec<usize> = leaves.iter().map(|leaf| leaf.records.len()).collect();
    let right_smallest = leaves.get(1).and_then(|leaf| leaf.records.first()).map(|record| record.inode);
    let right_identity = leaves.get(1).map(|leaf| leaf.identity);
    let raised_right_separator = raise_separator.version.leaves.get(1).map(|leaf| leaf.separator);
    let raised_right_number = raise_separator.version.leaves.get(1).map(|leaf| leaf.identity.number);
    let holds = borrows.len() == 1
        && other_merges == 0
        && borrow.merge == MergeAction::BorrowedFromTheRightNeighbour { records_moved: 5 }
        && sizes == vec![12, 11]
        && right_smallest == Some(22)
        && right_identity.is_some_and(|identity| identity.number == 22 && identity.birth == borrow.txg)
        && raised_right_separator == Some(22)
        && raised_right_number == Some(17)
        && borrow.written.code_three_counted() == raised.written.code_three_counted() + 1
        && borrow.written.code_two_counted() == raised.written.code_two_counted() + 1
        && borrow.written.retirement_records == raised.written.retirement_records + 1
        && renew_identity.publish_records.iter().zip(&raise_separator.publish_records).enumerate().all(|(other_position, (renewed, raised_other))| {
            other_position == position || renewed.written == raised_other.written
        });
    let detail = format!(
        "borrows={} other_merges={other_merges} merge={:?} sizes={sizes:?} right_smallest={right_smallest:?} n2_right_identity_number={:?} n1_right_separator={raised_right_separator:?} n1_right_number={raised_right_number:?} extra_code3={} extra_code2={} extra_retirements={}",
        borrows.len(),
        borrow.merge,
        right_identity.map(|identity| identity.number),
        difference_or_marker(borrow.written.code_three_counted(), raised.written.code_three_counted()),
        difference_or_marker(borrow.written.code_two_counted(), raised.written.code_two_counted()),
        difference_or_marker(borrow.written.retirement_records, raised.written.retirement_records),
    );
    (holds, detail)
}

/// 停机对拍 S1：末尾分裂（`crates/singlefs-core/src/inode_tree.rs:181-283`）——满了才分裂、右半号 = 触发那条记录的号、出生代 = 本次 txg、
/// 左半不动、新号只落最右叶。在容量 16 与 233 上各分裂一次。
fn stop_check_split_matches_the_implementation() -> (bool, String) {
    let mut every_capacity_holds = true;
    let mut details = Vec::new();
    for leaf_capacity in [TEST_LEAF_CAPACITY, INODE_LEAF_RECORDS] {
        let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity, merge_threshold: 0 };
        let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
        simulation.apply(UserChange::NewInodes { count: leaf_capacity - 1 });
        let one_leaf_before = simulation.version.leaves.len() == 1 && length_as_u64(simulation.version.leaves[0].records.len()) == leaf_capacity;
        let left_before = simulation.version.leaves[0].clone();
        simulation.apply(UserChange::NewInodes { count: 1 });
        let split_txg = simulation.version.txg;
        let leaves = &simulation.version.leaves;
        let holds = one_leaf_before
            && leaves.len() == 2
            && leaves[0].identity == left_before.identity
            && leaves[0].records == left_before.records
            && leaves[0].unit == left_before.unit
            && leaves[1].identity == ContainerIdentity::inode_leaf(leaf_capacity + 1, split_txg)
            && leaves[1].separator == leaf_capacity + 1
            && leaves[1].records.len() == 1
            && simulation.publish_records.last().is_some_and(|record| record.written.inode_leaf_containers == 1 && record.written.inode_roots == 1);
        every_capacity_holds &= holds;
        details.push(format!("capacity_{leaf_capacity}={holds}"));
    }
    (every_capacity_holds, details.join(","))
}

/// 停机对拍 S2：删一条（`inode_tree.rs:286-318`）——删完还有记录只重写那片；删空不落盘、右边各片左移且全部重写、根照样重写；删光交回零片。
fn stop_check_removal_matches_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: TEST_LEAF_CAPACITY, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    simulation.apply(UserChange::NewInodes { count: 47 });
    simulation.apply(UserChange::Unlink { object: 20 });
    let partial = written_and_merge_of_the_last_publish(&simulation);
    for object in (17..=32).filter(|object| *object != 20) {
        simulation.apply(UserChange::Unlink { object });
    }
    let emptied = written_and_merge_of_the_last_publish(&simulation);
    let leaves_after_emptying: Vec<u64> = simulation.version.leaves.iter().map(|leaf| leaf.identity.number).collect();
    let mut last_record = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    last_record.apply(UserChange::Unlink { object: FIRST_FILE_INODE });
    let gone = written_and_merge_of_the_last_publish(&last_record);
    let holds = partial.is_some_and(|(written, merge)| written.inode_leaf_containers == 1 && written.inode_roots == 1 && merge == MergeAction::NoMerge)
        && emptied.is_some_and(|(written, merge)| written.inode_leaf_containers == 1 && written.inode_roots == 1 && merge == MergeAction::EmptiedAndDropped)
        && leaves_after_emptying == vec![1, 33]
        && gone.is_some_and(|(written, merge)| written.inode_leaf_containers == 0 && written.inode_roots == 1 && merge == MergeAction::EmptiedAndDropped)
        && last_record.version.leaves.is_empty();
    (holds, format!("partial={partial:?} emptied={emptied:?} leaves_after_emptying={leaves_after_emptying:?} last_record={gone:?}"))
}

fn written_and_merge_of_the_last_publish(simulation: &Simulation) -> Option<(UnitsWritten, MergeAction)> {
    simulation.publish_records.last().map(|record| (record.written, record.merge))
}

/// 停机对拍 S3：unlink 的步（`crates/singlefs-core/src/unlink.rs:118-130`）——剩下 > b ⇒ 留 剩下 − b、size = 留下 × 32634；剩下 ≤ b ⇒ 最后一批。
fn stop_check_unlink_steps_match_the_implementation() -> (bool, String) {
    let cell = WritePathCell { borrow_arm: BorrowArm::RaiseSeparator, leaf_capacity: INODE_LEAF_RECORDS, merge_threshold: 0 };
    let mut simulation = Simulation::start(cell, PRODUCT_UNLINK_BATCH_DATA_UNITS, false);
    let steps_hold = [(300, Some(156)), (145, Some(1)), (144, None), (1, None), (0, None)].iter().all(|(remaining, kept)| {
        match (simulation.unlink_step_for(FIRST_FILE_INODE, *remaining), kept) {
            (Operation::UnlinkBatch { data_units_kept, .. }, Some(expected_kept)) => data_units_kept == *expected_kept,
            (Operation::UnlinkFinal { .. }, None) => true,
            (
                Operation::UnlinkBatch { .. }
                | Operation::UnlinkFinal { .. }
                | Operation::FirstFile
                | Operation::RowPublish
                | Operation::WarmUp
                | Operation::WriteFile { .. }
                | Operation::NewInodes { .. }
                | Operation::UnlinkIntent { .. }
                | Operation::RollBack { .. },
                Some(_) | None,
            ) => false,
        }
    });
    simulation.apply(UserChange::WriteFile { object: FIRST_FILE_INODE, data_units: 300 });
    simulation.publish(&Operation::UnlinkIntent { object: FIRST_FILE_INODE });
    let size_at_the_intent = simulation.version.record_of(FIRST_FILE_INODE).map(|record| record.size_bytes);
    let first_batch = simulation.next_unlink_step(FIRST_FILE_INODE);
    simulation.publish(&first_batch);
    let after_the_first_batch = simulation.version.record_of(FIRST_FILE_INODE);
    let holds = steps_hold
        && size_at_the_intent == Some(300 * DATA_UNIT_PAYLOAD_BYTES)
        && after_the_first_batch.is_some_and(|record| record.size_bytes == 156 * DATA_UNIT_PAYLOAD_BYTES && record.nlink == 0)
        && simulation.version.data_units_of(FIRST_FILE_INODE) == 156;
    (holds, format!("steps_hold={steps_hold} size_at_the_intent={size_at_the_intent:?} after_the_first_batch={after_the_first_batch:?}"))
}

/// 停机对拍 S6：K-0 至 K-5 的判定与 `crates/` 那几条测试断言的相同——K-0 全绿；K-1 至 K-5 只红 I-9.4（I-9.12 绿）。
pub(super) fn stop_check_known_bad_images_match_the_implementation() -> (bool, String) {
    let mut details = Vec::new();
    let mut every_shape_holds = true;
    for named in catalogue_of_shapes().into_iter().filter(|named| named.family == "K") {
        let verdicts = judge_the_shape(&named.shape);
        let holds = if named.name == "K-0" {
            verdicts.today_container_numbers_hold && verdicts.separators_hold && every_leaf_holds_a_record(&named.shape)
        } else {
            !verdicts.today_container_numbers_hold && verdicts.separators_hold && every_leaf_holds_a_record(&named.shape)
        };
        every_shape_holds &= holds;
        details.push(format!("{}={holds}", named.name));
    }
    (every_shape_holds, details.join(","))
}

/// 第七节 K1–K3、A2 的常量锚点。
pub(super) fn constant_anchor_lines() -> Vec<String> {
    vec![
        format!("name=anchor id=K1 inode_leaf_records={INODE_LEAF_RECORDS} expected=233 holds={}", INODE_LEAF_RECORDS == 233),
        format!(
            "name=anchor id=K2 tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} expected=815 holds={}",
            TOMBSTONE_CONTAINER_RECORDS == 815
        ),
        format!(
            "name=anchor id=K3 root_node_leaf_containers={ROOT_NODE_LEAF_CONTAINERS} expected=135 holds={}",
            ROOT_NODE_LEAF_CONTAINERS == 135
        ),
        format!(
            "name=anchor id=A2 retirement_record_bytes={RETIREMENT_RECORD_BYTES} retirement_container_records={RETIREMENT_CONTAINER_RECORDS} expected=34,959 holds={}",
            RETIREMENT_RECORD_BYTES == 34 && RETIREMENT_CONTAINER_RECORDS == 959
        ),
    ]
}

/// 工作项按片并行，产物按工作项的次序收（与线程数无关）。
pub(super) fn run_in_parallel<Item: Copy + Send + Sync, Output: Send>(
    items: &[Item],
    threads: usize,
    work: impl Fn(Item) -> Output + Sync,
) -> Vec<Output> {
    let next_item = AtomicUsize::new(0);
    let collected: Mutex<Vec<Option<Output>>> = Mutex::new((0..items.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            scope.spawn(|| loop {
                let position = next_item.fetch_add(1, Ordering::SeqCst);
                if position >= items.len() {
                    break;
                }
                let output = work(items[position]);
                collected.lock().expect("收结果的锁没有中毒：工作线程不在持锁时 panic")[position] = Some(output);
            });
        }
    });
    collected
        .into_inner()
        .expect("收结果的锁没有中毒")
        .into_iter()
        .map(|output| output.expect("每个工作项都收回了"))
        .collect()
}

/// V7 要全绿的那几句（第十一节 V7）：θ = 0 与 N2 按今天的 I-9.4 与 I-9.12，N1 按 N1 的 I-9.4 与 I-9.12；I-9.13 都要绿。
pub(super) fn self_check_holds(borrow_arm: BorrowArm, merge_threshold: u64, tally: SelfCheckTally) -> bool {
    let today_required = merge_threshold == 0 || borrow_arm == BorrowArm::RenewIdentity;
    let raised_required = borrow_arm == BorrowArm::RaiseSeparator;
    tally.leaves_hold_a_record_red == 0
        && tally.separators_red == 0
        && (!today_required || tally.today_container_numbers_red == 0)
        && (!raised_required || tally.raised_separator_container_numbers_red == 0)
}

/// F10 θ > 0 那一格算不算结果（第十一节 V6）：借、「被挪过的对象之后变短」「被挪过的对象之后被 unlink」三个次数都不为 0。
pub(super) const fn path_was_covered(run: &WritePathRun) -> bool {
    run.borrows > 0 && run.moved_then_shortened > 0 && run.moved_then_unlinked > 0
}

fn segment_one_cell_verdict_lines(results: &[SegmentOneResult]) -> Vec<String> {
    let mut lines = Vec::new();
    for leaf_capacity in LEAF_CAPACITIES {
        for merge_threshold in merge_thresholds_of(leaf_capacity) {
            let in_cell: Vec<&SegmentOneResult> = results
                .iter()
                .filter(|result| result.item.leaf_capacity == leaf_capacity && result.item.merge_threshold == merge_threshold)
                .collect();
            let mut extra_code_three = Extremes::default();
            let mut extra_code_two = Extremes::default();
            let mut renew_retirements = Extremes::default();
            let mut raise_retirements = Extremes::default();
            let mut extra_share = Extremes::default();
            let mut borrow_publishes = 0;
            let mut outside_differences = 0;
            let mut kinds_disagree = 0;
            let mut streams_identical = 0;
            for result in &in_cell {
                let pair = result.pair;
                borrow_publishes += pair.borrow_publishes;
                extra_code_three.absorb(pair.extra_code_three_per_borrow);
                extra_code_two.absorb(pair.extra_code_two_per_borrow);
                renew_retirements.absorb(pair.retirement_records_per_borrow_renew_identity);
                raise_retirements.absorb(pair.retirement_records_per_borrow_raise_separator);
                outside_differences += pair.publishes_outside_borrows_with_a_difference;
                kinds_disagree += pair.borrow_kinds_disagree;
                streams_identical += u64::from(pair.streams_identical);
                if pair.borrow_publishes > 0 {
                    extra_share.observe(pair.extra_share_in_parts_per_million());
                }
            }
            let every_extra_is_one = |extremes: Extremes| extremes.observations == 0 || (extremes.smallest == Some(1) && extremes.largest == Some(1));
            let definition_holds = every_extra_is_one(extra_code_three)
                && every_extra_is_one(extra_code_two)
                && outside_differences == 0
                && kinds_disagree == 0;
            let zero_threshold_streams_identical = merge_threshold != 0 || (borrow_publishes == 0 && streams_identical == length_as_u64(in_cell.len()));
            lines.push(format!(
                "name=verdict question=Q-X4-1 capacity={leaf_capacity} theta={merge_threshold} histories={} borrow_publishes={borrow_publishes} {} {} {} differences_outside_borrows={outside_differences} borrow_kinds_disagree={kinds_disagree} a4_a6_definition_holds={definition_holds} a5_zero_threshold_streams_identical={zero_threshold_streams_identical} judgement=reported",
                in_cell.len(),
                extra_code_three.fields("extra_code3_per_borrow"),
                extra_code_two.fields("extra_code2_per_borrow"),
                extra_share.fields("n2_extra_share_ppm_over_histories_with_borrows"),
            ));
            let retirements_hold = (renew_retirements.observations == 0 || (renew_retirements.smallest == Some(1) && renew_retirements.largest == Some(1)))
                && (raise_retirements.observations == 0 || raise_retirements.largest == Some(0));
            lines.push(format!(
                "name=verdict question=Q-X4-2 capacity={leaf_capacity} theta={merge_threshold} {} {} definition_holds={retirements_hold} judgement=reported",
                renew_retirements.fields("n2_retirements_per_borrow"),
                raise_retirements.fields("n1_retirements_per_borrow"),
            ));
            for family in ["F1", "F8", "F3", "F10"] {
                let in_family: Vec<&&SegmentOneResult> = in_cell.iter().filter(|result| result.item.history.family() == family).collect();
                let mut totals = [0_u64; 7];
                let mut not_covered = 0;
                let mut did_not_fit = 0;
                let mut arms_disagree = 0;
                for result in &in_family {
                    let renew = &result.renew_identity;
                    let raise = &result.raise_separator;
                    for (position, value) in [renew.borrows, renew.absorbs, renew.drops, renew.moved_then_shortened, renew.moved_then_unlinked, renew.publishes, renew.written.retirement_records]
                        .iter()
                        .enumerate()
                    {
                        totals[position] += value;
                    }
                    arms_disagree += u64::from((renew.borrows, renew.absorbs, renew.drops) != (raise.borrows, raise.absorbs, raise.drops));
                    did_not_fit += u64::from(renew.does_not_fit_at.is_some() || raise.does_not_fit_at.is_some());
                    if family == "F10" && merge_threshold > 0 && !path_was_covered(renew) {
                        not_covered += 1;
                    }
                }
                lines.push(format!(
                    "name=verdict question=Q-X4-3 capacity={leaf_capacity} theta={merge_threshold} family={family} segments={} borrows={} absorbs={} drops={} moved_then_shortened={} moved_then_unlinked={} publishes={} n2_retirement_records={} merge_counts_differ_between_n1_n2={arms_disagree} does_not_fit_segments={did_not_fit} v6_segments_without_coverage={not_covered} judgement=reported",
                    in_family.len(),
                    totals[0],
                    totals[1],
                    totals[2],
                    totals[3],
                    totals[4],
                    totals[5],
                    totals[6],
                ));
            }
            for borrow_arm in BORROW_ARMS {
                let mut tally = SelfCheckTally::default();
                for result in &in_cell {
                    let run = match borrow_arm {
                        BorrowArm::RaiseSeparator => &result.raise_separator,
                        BorrowArm::RenewIdentity => &result.renew_identity,
                    };
                    tally.add(run.self_check);
                }
                lines.push(format!(
                    "name=verdict question=V7 capacity={leaf_capacity} theta={merge_threshold} borrow_arm={} versions={} i913_red={} today_i94_red={} n1_i94_red={} i912_red={} required_green_holds={}",
                    borrow_arm.label(),
                    tally.versions,
                    tally.leaves_hold_a_record_red,
                    tally.today_container_numbers_red,
                    tally.raised_separator_container_numbers_red,
                    tally.separators_red,
                    self_check_holds(borrow_arm, merge_threshold, tally),
                ));
            }
        }
    }
    lines
}

/// Q-X4-4 至 Q-X4-7：按键布局、按族列「今天 I-9.4 红而 N1 的 I-9.4 绿」的形态（并上 I-9.12 前后）与路由错的。
fn shape_verdict_lines(verdicts: &[(&'static str, &'static str, String, PredicateVerdicts)]) -> Vec<String> {
    let mut lines = Vec::new();
    for layout in ["dense", "sparse"] {
        for family in ["K", "S", "BN1", "BN2", "DV", "all"] {
            let in_scope: Vec<&(&'static str, &'static str, String, PredicateVerdicts)> = verdicts
                .iter()
                .filter(|(shape_family, shape_layout, _, _)| *shape_layout == layout && (family == "all" || *shape_family == family))
                .collect();
            let missed_by_raised_separators: Vec<&str> = in_scope
                .iter()
                .filter(|(_, _, _, judged)| !judged.today_container_numbers_hold && judged.raised_separator_container_numbers_hold)
                .map(|(_, _, name, _)| name.as_str())
                .collect();
            let missed_with_separators: Vec<&str> = in_scope
                .iter()
                .filter(|(_, _, _, judged)| {
                    let today_with_separators = judged.today_container_numbers_hold && judged.separators_hold;
                    let raised_with_separators = judged.raised_separator_container_numbers_hold && judged.separators_hold;
                    !today_with_separators && raised_with_separators
                })
                .map(|(_, _, name, _)| name.as_str())
                .collect();
            let missed_and_misrouted: Vec<&str> = in_scope
                .iter()
                .filter(|(_, _, _, judged)| {
                    !judged.today_container_numbers_hold && judged.raised_separator_container_numbers_hold && !judged.routing_holds
                })
                .map(|(_, _, name, _)| name.as_str())
                .collect();
            let shown = |names: &[&str]| if names.is_empty() { "none".to_string() } else { names.join(",") };
            lines.push(format!(
                "name=verdict question=Q-X4-4 layout={layout} family={family} shapes={} today_red_n1_green={} names={} n1_misses_shapes={}",
                in_scope.len(),
                missed_by_raised_separators.len(),
                shown(&missed_by_raised_separators),
                !missed_by_raised_separators.is_empty(),
            ));
            lines.push(format!(
                "name=verdict question=Q-X4-5 layout={layout} family={family} shapes={} today_or_i912_red_n1_and_i912_green={} names={} i912_does_not_cover_the_misses={}",
                in_scope.len(),
                missed_with_separators.len(),
                shown(&missed_with_separators),
                !missed_with_separators.is_empty(),
            ));
            lines.push(format!(
                "name=verdict question=Q-X4-6 layout={layout} family={family} misses_with_wrong_routing={} names={} n1_misses_hurt_lookup={}",
                missed_and_misrouted.len(),
                shown(&missed_and_misrouted),
                !missed_and_misrouted.is_empty(),
            ));
            let count_red = |predicate: fn(&PredicateVerdicts) -> bool| in_scope.iter().filter(|(_, _, _, judged)| !predicate(judged)).count();
            lines.push(format!(
                "name=verdict question=Q-X4-7 layout={layout} family={family} shapes={} today_i94_red={} n1_i94_red={} i912_red={} routing_wrong={} judgement=table_listed_in_shape_lines",
                in_scope.len(),
                count_red(|judged| judged.today_container_numbers_hold),
                count_red(|judged| judged.raised_separator_container_numbers_hold),
                count_red(|judged| judged.separators_hold),
                count_red(|judged| judged.routing_holds),
            ));
        }
    }
    let dense: Vec<&(&'static str, &'static str, String, PredicateVerdicts)> =
        verdicts.iter().filter(|(_, layout, _, _)| *layout == "dense").collect();
    let differing: Vec<&str> = dense
        .iter()
        .filter(|(family, _, name, judged)| {
            verdicts
                .iter()
                .find(|(other_family, layout, other_name, _)| *layout == "sparse" && other_family == family && other_name == name)
                .is_some_and(|(_, _, _, sparse_judged)| sparse_judged != judged)
        })
        .map(|(_, _, name, _)| name.as_str())
        .collect();
    lines.push(format!(
        "name=verdict question=G-K shapes={} verdicts_differ_between_layouts={} names={} stable={}",
        dense.len(),
        differing.len(),
        if differing.is_empty() { "none".to_string() } else { differing.join(",") },
        differing.is_empty(),
    ));
    lines
}

/// PC-K：今天的 I-9.4 在 K-1 至 K-5 全红、K-0 全绿；N1 的谓词在 BN1「分隔 key 没抬」那一形（s = 20）红。
fn positive_control_shape_lines() -> Vec<String> {
    let catalogue = catalogue_of_shapes();
    let verdict_of = |name: &str| {
        catalogue.iter().find(|named| named.name == name).map(|named| judge_the_shape(&named.shape)).expect("形态表里有它")
    };
    let today_reddens_the_five_broken_shapes = ["K-1", "K-2", "K-3", "K-4", "K-5"].iter().all(|name| !verdict_of(name).today_container_numbers_hold);
    let healthy_shape_verdict = verdict_of("K-0");
    let healthy_shape_all_green = healthy_shape_verdict.today_container_numbers_hold && healthy_shape_verdict.raised_separator_container_numbers_hold && healthy_shape_verdict.separators_hold;
    let raised_reddens_unraised = !verdict_of("BN1-s20").raised_separator_container_numbers_hold;
    vec![format!(
        "name=positive_control id=PC-K today_i94_reddens_k1_to_k5={today_reddens_the_five_broken_shapes} k0_all_green={healthy_shape_all_green} n1_i94_reddens_bn1_s20={raised_reddens_unraised} holds={}",
        today_reddens_the_five_broken_shapes && healthy_shape_all_green && raised_reddens_unraised
    )]
}

pub(super) fn run_segment_one(threads: usize) -> Vec<String> {
    let items = segment_one_items();
    let mut lines = vec![format!(
        "name=config experiment=E178 segment=one work_items={} histories_h51={} histories_f10={} capacities=233,16 thetas=0,quarter,half borrow_arms=N1,N2 inode_leaf_records={INODE_LEAF_RECORDS} tombstone_container_records={TOMBSTONE_CONTAINER_RECORDS} retirement_container_records={RETIREMENT_CONTAINER_RECORDS} root_node_leaf_containers={ROOT_NODE_LEAF_CONTAINERS} data_unit_payload_bytes={DATA_UNIT_PAYLOAD_BYTES} scanning=none",
        items.len(),
        histories_replayed_from_the_e177_segment_one().len(),
        many_objects_with_data_histories().len(),
    )];
    lines.extend(constant_anchor_lines());
    let stop_checks = [
        ("S1", stop_check_split_matches_the_implementation()),
        ("S2", stop_check_removal_matches_the_implementation()),
        ("S3", stop_check_unlink_steps_match_the_implementation()),
        ("S6", stop_check_known_bad_images_match_the_implementation()),
    ];
    for (name, (holds, detail)) in &stop_checks {
        lines.push(format!("name=stop_check id={name} holds={holds} detail={detail}"));
    }
    let (shape_lines, shape_verdicts) = shape_table_lines();
    lines.extend(shape_lines);
    let results = run_in_parallel(&items, threads, run_segment_one_item);
    let mut tally = AssertionTally::default();
    let mut every_run_identical = true;
    let mut user_changes_identical = true;
    let mut zero_threshold_streams_identical = true;
    for result in &results {
        lines.push(write_path_line(result.item, BorrowArm::RaiseSeparator, &result.raise_separator));
        lines.push(write_path_line(result.item, BorrowArm::RenewIdentity, &result.renew_identity));
        lines.push(borrow_cost_line(result));
        tally.merge(&result.raise_separator.assertions);
        tally.merge(&result.renew_identity.assertions);
        every_run_identical &= result.two_runs_identical;
        user_changes_identical &= result.pair.user_changes_identical && result.pair.publish_counts_identical;
        if result.item.merge_threshold == 0 {
            zero_threshold_streams_identical &= result.pair.streams_identical && result.pair.borrow_publishes == 0;
        }
    }
    let (single_borrow_holds, single_borrow_detail) = single_borrow_anchor_holds();
    for name in [
        "A1_history_publishes",
        "A1_history_refused_steps",
        "A8_retirements_equal_identity_disappearances",
        "K6_split_right_half_numbered_after_the_record_left_half_unchanged",
    ] {
        lines.push(format!(
            "name=assertion assertion={name} judged={} violated={}",
            tally.judged_of(name),
            tally.violations_of(name)
        ));
    }
    lines.push(format!("name=anchor id=A5 zero_threshold_streams_identical={zero_threshold_streams_identical} holds={zero_threshold_streams_identical}"));
    lines.push(format!("name=anchor id=A6 {single_borrow_detail} holds={single_borrow_holds}"));
    lines.push(format!(
        "name=positive_control id=PC-N zero_borrow_histories_identical={zero_threshold_streams_identical} single_borrow_history_holds={single_borrow_holds} holds={}",
        zero_threshold_streams_identical && single_borrow_holds
    ));
    lines.extend(positive_control_shape_lines());
    lines.extend(segment_one_cell_verdict_lines(&results));
    lines.extend(shape_verdict_lines(&shape_verdicts));
    let history_assertion_violations = tally.violations_of("A1_history_publishes") + tally.violations_of("A1_history_refused_steps");
    let split_numbering_violations = tally.violations_of("K6_split_right_half_numbered_after_the_record_left_half_unchanged");
    let constants_hold = INODE_LEAF_RECORDS == 233 && TOMBSTONE_CONTAINER_RECORDS == 815 && ROOT_NODE_LEAF_CONTAINERS == 135;
    let anchors_hold = RETIREMENT_CONTAINER_RECORDS == 959
        && zero_threshold_streams_identical
        && single_borrow_holds
        && tally.violations_of("A8_retirements_equal_identity_disappearances") == 0;
    let stop_checks_hold = stop_checks.iter().all(|(_, (holds, _))| *holds);
    lines.push(format!(
        "name=verdict question=summary v1_history_replay_holds={} v4_two_runs_identical={every_run_identical} v4_user_changes_identical_between_n1_n2={user_changes_identical} v5_anchors_hold={anchors_hold} fl0_clause_constants_and_k6_hold={} stop_checks_hold={stop_checks_hold}",
        history_assertion_violations == 0,
        constants_hold && split_numbering_violations == 0,
    ));
    lines
}

#[cfg(test)]
#[path = "write_path_runs_and_stop_checks_tests.rs"]
mod tests;
