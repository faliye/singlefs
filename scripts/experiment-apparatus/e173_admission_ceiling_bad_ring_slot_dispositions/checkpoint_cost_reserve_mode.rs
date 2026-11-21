//! 重跑模式：保留池按检查点代价现算的几何与行、读数拆分、阳性对照与整个模式的运行。

use super::*;

// ── `checkpoint-cost-reserve` 模式：按 ckpt_cost 现算的保留池（重跑登记 `research/prompts/e173-r1-prereg.md` 第五节「第二段的两条臂」、第六节 Y1、Z1、Z3） ──

/// 第一段量出来的一个几何：S、池大小、c实_G（暖机与卸载那一串取得出的空发布新占固定点槽数的最大值）、
/// κ_G（重跑登记修订 4：接在有树那几版后面的发布上核心层 κ 的最小值）。每一行抄自第一段产物的 `name=x3_reserve_fit` 行
/// （`kappa_of_the_geometry=minimum_on_versions_with_trees`，行号写在 `product_line`），开跑先逐行比那份产物（停机条款 S5）。
/// S = 4 的几何另带第七节 B9 的期望（`first_request.py 4 <c实_G> 8 <P>` 的输出换成本装置的读数写法，修订里贴原样输出）。
#[derive(Clone, Copy, Debug)]
pub(super) struct CheckpointCostGeometry {
    pub(super) slots_per_region: u64,
    pub(super) pool_size: SampledPoolSize,
    pub(super) largest_actual_empty_publish_slots: i64,
    pub(super) checkpoint_cost_slots: i64,
    pub(super) product_line: usize,
    pub(super) first_request_under_the_code_reading: Option<&'static str>,
    pub(super) first_request_under_the_clause_reading: Option<&'static str>,
}

impl CheckpointCostGeometry {
    pub(super) fn label(self) -> String {
        format!("s{}_{}", self.slots_per_region, self.pool_size.label())
    }
}

/// 第一段产物（入库装置 `crates/singlefs-implementation-experiments/src/bin/e173_checkpoint_cost_against_empty_publish_cost.rs` 的输出，相对仓根）。
/// 没有第一段产物时是 None：模式开跑时停机条款 S5 停在「没有第一段产物」。
const SEGMENT_ONE_PRODUCT: Option<&str> = Some("research/results/e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out");

/// S = 4 的几何上第一次请求的期望（重跑登记第七节 B9）：`first_request.py 4 <c实_G> 8 <P>` 输出 `user_room=free-P` 与 `need` 两个数，
/// 本装置的读数写法是发成之后的空闲 − P = user_room − need（修订 12 同一口径：c = 5、P = 45 时 63 − 13 = 50）。
/// c实_G = 6 时 66 − 14 = 52，c实_G = 10 时 78 − 18 = 60；两种读法的 user_room 相同（P 只改空闲，不改空闲 − P）。
const FIRST_REQUEST_ON_THE_SMALL_EMPTY_PUBLISH_GEOMETRIES: &str = "accepted=true ceiling=47 sequence_publishes=3 floor=47 free_above_reserve_after=52";
const FIRST_REQUEST_ON_THE_LARGE_EMPTY_PUBLISH_GEOMETRY: &str = "accepted=true ceiling=47 sequence_publishes=3 floor=47 free_above_reserve_after=60";

/// 第一段量过的 9 个几何，每行抄自 `SEGMENT_ONE_PRODUCT` 的 `name=x3_reserve_fit … kappa_of_the_geometry=minimum_on_versions_with_trees` 那一行
/// （行号在 `product_line`；G-min 在三个 S 上都找得到：S = 4、8 取 64 MiB，S = 16 取 128 MiB，失败条款 F6 没触发）。
/// 开跑先逐行比那份产物（停机条款 S5）。
pub(super) const CHECKPOINT_COST_GEOMETRIES: &[CheckpointCostGeometry] = &[
    CheckpointCostGeometry {
        slots_per_region: 4,
        pool_size: SampledPoolSize::SmallestCompleting { mebibytes: 64 },
        largest_actual_empty_publish_slots: 6,
        checkpoint_cost_slots: 18,
        product_line: 3299,
        first_request_under_the_code_reading: Some(FIRST_REQUEST_ON_THE_SMALL_EMPTY_PUBLISH_GEOMETRIES),
        first_request_under_the_clause_reading: Some(FIRST_REQUEST_ON_THE_SMALL_EMPTY_PUBLISH_GEOMETRIES),
    },
    CheckpointCostGeometry {
        slots_per_region: 4,
        pool_size: SampledPoolSize::OneGibibyte,
        largest_actual_empty_publish_slots: 6,
        checkpoint_cost_slots: 130,
        product_line: 3322,
        first_request_under_the_code_reading: Some(FIRST_REQUEST_ON_THE_SMALL_EMPTY_PUBLISH_GEOMETRIES),
        first_request_under_the_clause_reading: Some(FIRST_REQUEST_ON_THE_SMALL_EMPTY_PUBLISH_GEOMETRIES),
    },
    CheckpointCostGeometry {
        slots_per_region: 4,
        pool_size: SampledPoolSize::FourGibibytes,
        largest_actual_empty_publish_slots: 10,
        checkpoint_cost_slots: 543,
        product_line: 3345,
        first_request_under_the_code_reading: Some(FIRST_REQUEST_ON_THE_LARGE_EMPTY_PUBLISH_GEOMETRY),
        first_request_under_the_clause_reading: Some(FIRST_REQUEST_ON_THE_LARGE_EMPTY_PUBLISH_GEOMETRY),
    },
    CheckpointCostGeometry {
        slots_per_region: 8,
        pool_size: SampledPoolSize::SmallestCompleting { mebibytes: 64 },
        largest_actual_empty_publish_slots: 6,
        checkpoint_cost_slots: 18,
        product_line: 3368,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    },
    CheckpointCostGeometry {
        slots_per_region: 8,
        pool_size: SampledPoolSize::OneGibibyte,
        largest_actual_empty_publish_slots: 6,
        checkpoint_cost_slots: 130,
        product_line: 3391,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    },
    CheckpointCostGeometry {
        slots_per_region: 8,
        pool_size: SampledPoolSize::FourGibibytes,
        largest_actual_empty_publish_slots: 10,
        checkpoint_cost_slots: 543,
        product_line: 3414,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    },
    CheckpointCostGeometry {
        slots_per_region: 16,
        pool_size: SampledPoolSize::SmallestCompleting { mebibytes: 128 },
        largest_actual_empty_publish_slots: 6,
        checkpoint_cost_slots: 26,
        product_line: 3437,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    },
    CheckpointCostGeometry {
        slots_per_region: 16,
        pool_size: SampledPoolSize::OneGibibyte,
        largest_actual_empty_publish_slots: 6,
        checkpoint_cost_slots: 130,
        product_line: 3460,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    },
    CheckpointCostGeometry {
        slots_per_region: 16,
        pool_size: SampledPoolSize::FourGibibytes,
        largest_actual_empty_publish_slots: 10,
        checkpoint_cost_slots: 543,
        product_line: 3483,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    },
];

/// 第一段产物里一个几何的 (S, 池大小写法, κ_G, c实_G, 行号)：只认 `name=x3_reserve_fit` 里 `kappa_of_the_geometry=minimum_on_versions_with_trees` 的行。
fn checkpoint_cost_rows_of_the_segment_one_product(product: &str) -> Vec<(String, i64, i64, usize)> {
    let mut rows = Vec::new();
    for (line_index, line) in product.lines().enumerate() {
        if !line.starts_with("E7RESULT name=x3_reserve_fit ") || !line.contains(" kappa_of_the_geometry=minimum_on_versions_with_trees ") {
            continue;
        }
        let field = |name: &str| line.split(' ').find_map(|token| token.strip_prefix(name)).map(str::to_string);
        let geometry = field("geometry=");
        let kappa = field("kappa=").and_then(|text| text.parse::<i64>().ok());
        let largest = field("largest_actual_empty_publish_slots=").and_then(|text| text.parse::<i64>().ok());
        if let (Some(geometry), Some(kappa), Some(largest)) = (geometry, kappa, largest) {
            rows.push((geometry, kappa, largest, line_index + 1));
        }
    }
    rows
}

/// 停机条款 S5：常量表与第一段产物逐行比。交回对不上的那几处（空表示对得上）。
pub(super) fn segment_one_mismatches(table: &[CheckpointCostGeometry], product: &str) -> Vec<String> {
    let product_rows = checkpoint_cost_rows_of_the_segment_one_product(product);
    let mut mismatches = Vec::new();
    for row in table {
        let label = row.label();
        match product_rows.iter().find(|(geometry, ..)| *geometry == label) {
            None => mismatches.push(format!("{label}:not_in_the_product")),
            Some((_, kappa, largest, line_number)) => {
                if *kappa != row.checkpoint_cost_slots {
                    mismatches.push(format!("{label}:kappa_table={}_product={kappa}", row.checkpoint_cost_slots));
                }
                if *largest != row.largest_actual_empty_publish_slots {
                    mismatches.push(format!("{label}:actual_empty_publish_table={}_product={largest}", row.largest_actual_empty_publish_slots));
                }
                if *line_number != row.product_line {
                    mismatches.push(format!("{label}:line_table={}_product={line_number}", row.product_line));
                }
            }
        }
    }
    for (geometry, ..) in &product_rows {
        match SampledPoolSize::of_the_geometry_label(geometry) {
            None => mismatches.push(format!("{geometry}:unrecognized_geometry")),
            Some((slots_per_region, pool_size)) => {
                if !table.iter().any(|row| row.slots_per_region == slots_per_region && row.pool_size == pool_size) {
                    mismatches.push(format!("{geometry}:not_in_the_table"));
                }
            }
        }
    }
    mismatches
}

/// 一个几何上的三种保留池读法：代码读法（现算）、条款读法（现算）、判别力对照（改回 10 + 7 c实）。
pub(super) fn checkpoint_cost_readings_of(row: CheckpointCostGeometry) -> [ReserveReading; 3] {
    let (pool_size, checkpoint_cost_blocks) = (row.pool_size, row.checkpoint_cost_slots);
    [
        ReserveReading::CheckpointCostCodeReading { pool_size, checkpoint_cost_blocks },
        ReserveReading::CheckpointCostClauseReading { pool_size, checkpoint_cost_blocks },
        ReserveReading::RevertedToTenPlusSevenActualEmptyPublishes { pool_size, checkpoint_cost_blocks },
    ]
}

/// 这个模式的池：每个几何 × 三种读法 × 三种池松紧；c 取 c实_G（原模型的 c 那一项，覆盖写的元数据那一半同样取它，第六节 X5 报差多少）。
pub(super) fn checkpoint_cost_pool_settings(table: &[CheckpointCostGeometry]) -> Vec<PoolSettings> {
    let mut pool_settings = Vec::new();
    for row in table.iter().copied() {
        for reserve_reading in checkpoint_cost_readings_of(row) {
            for fullness in POOL_FULLNESSES {
                pool_settings.push(PoolSettings {
                    geometry: RingGeometry { slots_per_region: row.slots_per_region },
                    metadata_blocks: row.largest_actual_empty_publish_slots,
                    reserve_reading,
                    fullness,
                });
            }
        }
    }
    pool_settings
}

/// 一格在一种保留池读法下的读数（Z1、Z3 与判别力对照逐格比的那几样）。
#[derive(Clone, Debug)]
pub(super) struct ReadingsOfACell {
    pub(super) slots_per_region: u64,
    pub(super) pool_size: &'static str,
    pub(super) fullness: PoolFullness,
    /// 除保留池读法之外的全部读数：每个量的聚合、六条轨迹、读坏槽住着非空根的位置与那几处的越过数、零点被拒次数。
    pub(super) readings: (BTreeMap<&'static str, Aggregate>, [TrajectoryAggregate; TRAJECTORY_SERIES], usize, Aggregate, i64),
    /// Y1 每一行在这一格上打不打中（Q1 那几行逐格取，Q2 那几行按两条 Q2 臂并排的一组取、记在全零不并那一格上）。
    pub(super) verdict_hits_by_row: Vec<(String, bool)>,
    pub(super) baseline_rejected: i64,
    pub(super) baseline_first_rejected_request: Option<usize>,
}

/// 格的键去掉保留池读法那一维：两种读法下的同一格用它配对。
fn cell_key_without_the_reserve(key: &CellKey) -> String {
    format!(
        "family={} kind={} persistence={} S={} metadata_blocks={} pool_size={} pool={} q1={} q2={}",
        key.family.label(),
        label_of_the_fault_kind(key.fault_kind),
        label_of_the_persistence(key.persistence),
        key.slots_per_region,
        key.metadata_blocks,
        key.reserve_reading.pool_size_label(),
        key.fullness.label(),
        key.arm.stuck_ceiling.label(),
        key.arm.all_zero_slot.label()
    )
}

fn readings_of_a_cell(cell: &CellSummary, verdict_hits_by_row: Vec<(String, bool)>) -> ReadingsOfACell {
    ReadingsOfACell {
        slots_per_region: cell.key.slots_per_region,
        pool_size: cell.key.reserve_reading.pool_size_label(),
        fullness: cell.key.fullness,
        readings: (cell.quantities.clone(), cell.trajectories, cell.non_empty_slot_positions, cell.crossed_at_non_empty_slot_positions, cell.baseline_rejected),
        verdict_hits_by_row,
        baseline_rejected: cell.baseline_rejected,
        baseline_first_rejected_request: cell.baseline_first_rejected_request,
    }
}

/// Q1 那几行在一格上的打中：不在那一行范围里的格不算，值取不到（读坏槽没住非空根）的不算。
fn stuck_ceiling_hits_of_a_cell(cell: &CellSummary) -> Vec<(String, bool)> {
    STUCK_CEILING_JUDGMENTS
        .iter()
        .filter(|judgment| (judgment.belongs_to_the_scope)(cell))
        .filter_map(|judgment| (judgment.value)(cell).map(|value| (judgment.judgment_label.to_string(), value > 0)))
        .collect()
}

/// Q2 那几行在一组（两条 Q2 臂并排）上的打中，范围（z 读坏或 H5 的洞）写进行名。
fn zero_slot_hits_of_a_group(result: &GroupResult) -> Vec<(String, bool)> {
    let pair = &result.pair;
    let scope = if result.not_merged.key.fault_kind.is_some() { "z_h1_to_h4" } else { "h5_holes" };
    [
        ("Q2-1-not-merged-has-more", pair.not_merged_crossed_more + pair.not_merged_lost_more > 0),
        ("Q2-1-merged-has-more", pair.merged_crossed_more + pair.merged_lost_more > 0),
        ("Q2-2-merged-all-enospc", result.merged.quantity("all_enospc").positive_positions > 0),
        ("Q2-2-merged-all-enospc-beyond-not-merged", pair.positions_merged_is_all_enospc_against_not_merged > 0),
    ]
    .into_iter()
    .map(|(label, hit)| (format!("{label}@{scope}"), hit))
    .collect()
}

/// 两组格按保留池读法分开：读法的产物写法 → (去掉读法的格键 → 读数)。
fn readings_by_reserve(stuck_ceiling_results: &[GroupResult], zero_slot_results: &[GroupResult]) -> BTreeMap<&'static str, BTreeMap<String, ReadingsOfACell>> {
    let mut by_reserve: BTreeMap<&'static str, BTreeMap<String, ReadingsOfACell>> = BTreeMap::new();
    let mut insert = |cell: &CellSummary, group_of_cells: &str, verdict_hits_by_row: Vec<(String, bool)>| {
        let previous = by_reserve
            .entry(cell.key.reserve_reading.label())
            .or_default()
            .insert(format!("{} cells={group_of_cells}", cell_key_without_the_reserve(&cell.key)), readings_of_a_cell(cell, verdict_hits_by_row));
        assert!(previous.is_none(), "一种读法下同一格只出现一次");
    };
    for result in stuck_ceiling_results {
        insert(&result.not_merged, "stuck_ceiling", stuck_ceiling_hits_of_a_cell(&result.not_merged));
    }
    for result in zero_slot_results {
        insert(&result.not_merged, "zero_slot", zero_slot_hits_of_a_group(result));
        insert(&result.merged, "zero_slot", Vec::new());
    }
    by_reserve
}

/// Y1 里「多拒或多卡」的那四行（重跑登记第六节 Z1 门槛：Q1-1、Q1-1b、Q1-3、Q2-2；Q2-2 连原登记修订五补的「全零并比不并多卡」那一条）。
fn is_a_refusal_row(row_label: &str) -> bool {
    row_label == "Q1-1" || row_label == "Q1-1b" || row_label == "Q1-3" || row_label.starts_with("Q2-2-")
}

/// 一个几何上两种读法的分辨格数（重跑登记第六节 Z1）：配上对的格数、只代码读法打中、只条款读法打中、两边都打中。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ReadingSplit {
    pub(super) cells: usize,
    pub(super) only_code_hit: usize,
    pub(super) only_clause_hit: usize,
    pub(super) both_hit: usize,
}

/// Z1：逐格比代码读法与条款读法下 Y1 每一行打不打中，按 (行, S, 池大小) 数；另交回配不上对的格数（应为 0）。
pub(super) fn reading_splits(code: &BTreeMap<String, ReadingsOfACell>, clause: &BTreeMap<String, ReadingsOfACell>) -> (BTreeMap<(String, u64, &'static str), ReadingSplit>, usize) {
    let mut splits: BTreeMap<(String, u64, &'static str), ReadingSplit> = BTreeMap::new();
    let mut unpaired = clause.keys().filter(|key| !code.contains_key(*key)).count();
    for (key, code_cell) in code {
        let Some(clause_cell) = clause.get(key) else {
            unpaired += 1;
            continue;
        };
        for (row_label, code_hit) in &code_cell.verdict_hits_by_row {
            let Some(clause_hit) = clause_cell.verdict_hits_by_row.iter().find(|(label, _)| label == row_label).map(|(_, hit)| *hit) else {
                unpaired += 1;
                continue;
            };
            let split = splits.entry((row_label.clone(), code_cell.slots_per_region, code_cell.pool_size)).or_default();
            split.cells += 1;
            split.only_code_hit += usize::from(*code_hit && !clause_hit);
            split.only_clause_hit += usize::from(clause_hit && !*code_hit);
            split.both_hit += usize::from(*code_hit && clause_hit);
        }
    }
    (splits, unpaired)
}

/// Z1 的判定（重跑登记第六节 Z1 门槛与判定）与它的判别力自证（第八节：门槛挪到观测到的最大值 + 1 必须由真转假）。
fn emit_the_reading_splits(emitter: &mut Emitter, tally: &mut RunTally, by_reserve: &BTreeMap<&'static str, BTreeMap<String, ReadingsOfACell>>) {
    let empty = BTreeMap::new();
    let code = by_reserve.get("ckpt-code").unwrap_or(&empty);
    let clause = by_reserve.get("ckpt-clause").unwrap_or(&empty);
    let (splits, unpaired) = reading_splits(code, clause);
    tally.self_check(unpaired == 0 && !code.is_empty(), "V2_z1_cells_do_not_pair_up");
    let mut only_code_in_refusal_rows_largest = 0usize;
    let (mut only_code_in_refusal_rows, mut only_code_anywhere, mut only_clause_anywhere) = (0usize, 0usize, 0usize);
    for ((row_label, slots_per_region, pool_size), split) in &splits {
        emit(
            emitter,
            format!(
                "name=z1_reading_split id={row_label} S={slots_per_region} pool_size={pool_size} cells={} only_code_hit={} only_clause_hit={} both_hit={} refusal_row={}",
                split.cells,
                split.only_code_hit,
                split.only_clause_hit,
                split.both_hit,
                yes_no(is_a_refusal_row(row_label))
            ),
        );
        if is_a_refusal_row(row_label) {
            only_code_in_refusal_rows += split.only_code_hit;
            only_code_in_refusal_rows_largest = only_code_in_refusal_rows_largest.max(split.only_code_hit);
        }
        only_code_anywhere += split.only_code_hit;
        only_clause_anywhere += split.only_clause_hit;
    }
    let near_full_code_cells_with_path_evidence = code
        .values()
        .filter(|cell| cell.fullness.is_nearly_full() && cell.readings.2 > 0 && cell.readings.0.get("cap_active").is_some_and(|aggregate| aggregate.maximum > 0))
        .count();
    emit(
        emitter,
        format!(
            "name=z1_summary cells_paired={} unpaired={unpaired} only_code_hit_in_refusal_rows={only_code_in_refusal_rows} only_code_hit_anywhere={only_code_anywhere} only_clause_hit_anywhere={only_clause_anywhere} \
             code_reading_rejects_or_sticks_more={} readings_indistinguishable={} clause_reading_worse_on_some_row={} near_full_code_cells_with_cap_active_and_a_non_empty_bad_slot={near_full_code_cells_with_path_evidence}",
            code.len(),
            yes_no(only_code_in_refusal_rows > 0),
            yes_no(only_code_anywhere == 0 && only_clause_anywhere == 0),
            yes_no(only_clause_anywhere > 0)
        ),
    );
    let per_judgment = BTreeMap::from([("all_geometries", (usize::from(only_code_in_refusal_rows > 0), 1usize, Some(count_as_i64(only_code_in_refusal_rows_largest))))]);
    emit_the_threshold_shift_check(emitter, tally, "checkpoint-cost-reserve", "Z1-code-rejects-or-sticks-more", &per_judgment);
}

/// Z3 一个 (S, 池大小, 池松紧) 上的数：配上对的格、只代码读法零点被拒、只条款读法零点被拒、两边都拒、两边各自第一次被拒最早落在第几次请求。
pub(super) type ZeroPointCounts = BTreeMap<(u64, &'static str, &'static str), (usize, usize, usize, usize, Option<usize>, Option<usize>)>;

/// Z3：零点（去掉读故障那一次）的被拒次数，两种读法逐格比，按 (S, 池大小, 池松紧) 数。
fn emit_the_zero_point_comparison(emitter: &mut Emitter, tally: &mut RunTally, by_reserve: &BTreeMap<&'static str, BTreeMap<String, ReadingsOfACell>>) {
    let empty = BTreeMap::new();
    let code = by_reserve.get("ckpt-code").unwrap_or(&empty);
    let clause = by_reserve.get("ckpt-clause").unwrap_or(&empty);
    let counts = zero_point_counts(code, clause);
    let (mut code_only_total, mut clause_only_total, mut code_only_largest) = (0usize, 0usize, 0usize);
    for ((slots_per_region, pool_size, fullness), (cells, code_only, clause_only, both, code_first, clause_first)) in &counts {
        emit(
            emitter,
            format!(
                "name=z3_zero_point S={slots_per_region} pool_size={pool_size} pool={fullness} cells={cells} code_rejects_clause_does_not={code_only} clause_rejects_code_does_not={clause_only} both_reject={both} \
                 code_first_rejected_request={} clause_first_rejected_request={}",
                code_first.map_or("none".to_string(), |request| request.to_string()),
                clause_first.map_or("none".to_string(), |request| request.to_string())
            ),
        );
        code_only_total += code_only;
        clause_only_total += clause_only;
        code_only_largest = code_only_largest.max(*code_only);
    }
    emit(
        emitter,
        format!(
            "name=z3_summary code_rejects_more_without_a_read_fault={} clause_rejects_more_without_a_read_fault={} cells_code_only={code_only_total} cells_clause_only={clause_only_total}",
            yes_no(code_only_total > 0),
            yes_no(clause_only_total > 0)
        ),
    );
    let per_judgment = BTreeMap::from([("all_geometries", (usize::from(code_only_total > 0), 1usize, Some(count_as_i64(code_only_largest))))]);
    emit_the_threshold_shift_check(emitter, tally, "checkpoint-cost-reserve", "Z3-code-rejects-more-at-the-zero-point", &per_judgment);
}

pub(super) fn zero_point_counts(code: &BTreeMap<String, ReadingsOfACell>, clause: &BTreeMap<String, ReadingsOfACell>) -> ZeroPointCounts {
    let mut counts: ZeroPointCounts = BTreeMap::new();
    for (key, code_cell) in code {
        let Some(clause_cell) = clause.get(key) else {
            continue;
        };
        let entry = counts.entry((code_cell.slots_per_region, code_cell.pool_size, code_cell.fullness.label())).or_insert((0, 0, 0, 0, None, None));
        entry.0 += 1;
        entry.1 += usize::from(code_cell.baseline_rejected > 0 && clause_cell.baseline_rejected == 0);
        entry.2 += usize::from(clause_cell.baseline_rejected > 0 && code_cell.baseline_rejected == 0);
        entry.3 += usize::from(code_cell.baseline_rejected > 0 && clause_cell.baseline_rejected > 0);
        entry.4 = entry.4.into_iter().chain(code_cell.baseline_first_rejected_request).min();
        entry.5 = entry.5.into_iter().chain(clause_cell.baseline_first_rejected_request).min();
    }
    counts
}

/// 判别力对照（重跑登记第十二节修订，派发提示「保留池改回 10 + 7c 时新臂读数必与按 ckpt_cost 的不同」）：保留池改回 10 + 7 c实 的那一臂，
/// 逐格与按 κ_G 的两条臂比读数（除读法之外的全部读数）与 Y1 判定，按 (S, 池大小) 数不同的格；交回每条被比的臂有没有一格不同。
pub(super) fn emit_the_reverted_reserve_discrimination(emitter: &mut Emitter, tally: &mut RunTally, by_reserve: &BTreeMap<&'static str, BTreeMap<String, ReadingsOfACell>>) {
    let empty = BTreeMap::new();
    let reverted = by_reserve.get("reverted-10+7c").unwrap_or(&empty);
    for against in ["ckpt-code", "ckpt-clause"] {
        let counts = reverted_reserve_differences(reverted, by_reserve.get(against).unwrap_or(&empty));
        let mut cells_with_differing_readings = 0usize;
        for ((slots_per_region, pool_size), (cells, differing_readings, differing_verdicts)) in &counts {
            emit(
                emitter,
                format!(
                    "name=discrimination_reverted_reserve against={against} S={slots_per_region} pool_size={pool_size} cells={cells} cells_with_differing_readings={differing_readings} cells_with_a_differing_verdict={differing_verdicts}"
                ),
            );
            cells_with_differing_readings += differing_readings;
        }
        let compared_cells: usize = counts.values().map(|(cells, ..)| cells).sum();
        tally.self_check(compared_cells == reverted.len() && !reverted.is_empty(), "V2_reverted_reserve_cells_do_not_pair_up");
        emit(
            emitter,
            format!(
                "name=discrimination_reverted_reserve_summary against={against} cells={compared_cells} cells_with_differing_readings={cells_with_differing_readings} reverted_reserve_readings_differ={}",
                yes_no(cells_with_differing_readings > 0)
            ),
        );
    }
}

/// 按 (S, 池大小)：配上对的格、读数不同的格、Y1 判定不同的格。
pub(super) fn reverted_reserve_differences(reverted: &BTreeMap<String, ReadingsOfACell>, compared: &BTreeMap<String, ReadingsOfACell>) -> BTreeMap<(u64, &'static str), (usize, usize, usize)> {
    let mut counts: BTreeMap<(u64, &'static str), (usize, usize, usize)> = BTreeMap::new();
    for (key, reverted_cell) in reverted {
        let Some(compared_cell) = compared.get(key) else {
            continue;
        };
        let entry = counts.entry((reverted_cell.slots_per_region, reverted_cell.pool_size)).or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 += usize::from(reverted_cell.readings != compared_cell.readings);
        entry.2 += usize::from(reverted_cell.verdict_hits_by_row != compared_cell.verdict_hits_by_row);
    }
    counts
}

/// 第一次请求那一步的读数（第七节 B8、B9）：发成时写上限、那一串发了几次、F、发成之后空闲 − P；被拒时写拒因。
pub(super) fn first_request_reading(history: &PoolAfterHistory, settings: PoolSettings, arm: Arm) -> String {
    let mut session = start_a_session(history, None, settings, arm, Controls::none(), false);
    let record = session.run_one_request();
    if !record.accepted {
        return format!("accepted=false rejection={}", record.rejection.map_or("none", RejectionReason::label));
    }
    match session.ledger.raises.first() {
        None => format!("accepted=true raises=0 floor={} free_above_reserve_after={}", record.floor_after, record.free_above_the_reserve_after),
        Some(raise) => format!(
            "accepted=true ceiling={} sequence_publishes={} floor={} free_above_reserve_after={}",
            raise.truth_ceiling, raise.publishes, record.floor_after, record.free_above_the_reserve_after
        ),
    }
}

/// PC5（重跑登记第五节「阳性对照」、第七节 B8）：不读几何常量表，κ = 5、c = 5、d = 8、S = 4、H1、近满 k = 0、不注读故障、每一对臂。
/// 代码读法 P = κ = 5：那一串落不了点、拒因「抬 F 那一串自己取不到落点」；条款读法 P = 10 + 7 × 5 = 45 与改回 10 + 7c 的 P = 45：
/// 一串抬 F（53、54、55）之后发成，上限 47、F 47、发成之后空闲 − P = 63 − 13 = 50（原登记 B5 那一行同一组数）。
pub(super) fn emit_the_checkpoint_cost_positive_control(emitter: &mut Emitter, tally: &mut RunTally) {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let (pool_size, checkpoint_cost_blocks) = (SampledPoolSize::FourGibibytes, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let chain_lands = "accepted=true ceiling=47 sequence_publishes=3 floor=47 free_above_reserve_after=50";
    for (reserve_reading, expected) in [
        (ReserveReading::CheckpointCostCodeReading { pool_size, checkpoint_cost_blocks }, "accepted=false rejection=raise_no_placement"),
        (ReserveReading::CheckpointCostClauseReading { pool_size, checkpoint_cost_blocks }, chain_lands),
        (ReserveReading::RevertedToTenPlusSevenActualEmptyPublishes { pool_size, checkpoint_cost_blocks }, chain_lands),
    ] {
        let settings = PoolSettings { geometry, metadata_blocks: METADATA_BLOCKS_OF_THE_CONTROL_POOLS, reserve_reading, fullness: PoolFullness::NearlyFullWithNoSpareOverwrite };
        for arm in every_arm_pair() {
            let label = format!("PC5_{}_P{}_{}", reserve_reading.label(), settings.reserve_blocks(), label_of_the_arm(arm));
            emit_a_check(emitter, tally, false, &label, first_request_reading(&history, settings, arm), expected.to_string());
        }
    }
}

/// PC6（第七节 B9）：每个 S = 4 的几何上，两条读法、每一对臂的第一次请求等于常量表里抄下的 `first_request.py` 的期望。
pub(super) fn emit_the_geometry_first_requests(emitter: &mut Emitter, tally: &mut RunTally, table: &[CheckpointCostGeometry]) {
    for row in table.iter().copied().filter(|row| row.slots_per_region == SLOTS_PER_REGION_OF_THE_CONTROL_POOLS) {
        let geometry = RingGeometry { slots_per_region: row.slots_per_region };
        let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, row.largest_actual_empty_publish_slots);
        let [code_reading, clause_reading, _] = checkpoint_cost_readings_of(row);
        for (reserve_reading, expected) in [(code_reading, row.first_request_under_the_code_reading), (clause_reading, row.first_request_under_the_clause_reading)] {
            let settings = PoolSettings { geometry, metadata_blocks: row.largest_actual_empty_publish_slots, reserve_reading, fullness: PoolFullness::NearlyFullWithNoSpareOverwrite };
            for arm in every_arm_pair() {
                let label = format!("PC6_{}_{}_{}", row.label(), reserve_reading.label(), label_of_the_arm(arm));
                emit_a_check(emitter, tally, false, &label, first_request_reading(&history, settings, arm), expected.unwrap_or("expectation_missing").to_string());
            }
        }
    }
}

/// `checkpoint-cost-reserve` 模式（重跑登记第五节「第二段」、第十一节 S5）：先比几何常量表与第一段产物，对不上就停；
/// 再跑 PC5、PC6，再把第一段与第二段那两组格在每个几何 × 三种读法上各跑一遍。按 κ_G 的两条臂进 Y1 判定行与 Z1、Z3；
/// 改回 10 + 7c 那一臂只进格行与判别力对照。
pub(super) fn run_the_checkpoint_cost_reserve_mode(emitter: &mut Emitter, tally: &mut RunTally) {
    emit(
        emitter,
        format!(
            "name=config segment=checkpoint-cost-reserve geometries={} segment_one_product={} data_blocks_per_overwrite={DATA_BLOCKS_PER_OVERWRITE} admission_budget={PUBLISHES_PER_ADMISSION_AT_MOST} requests_per_mount=2*3S+{REQUESTS_PER_MOUNT_BASE} reserve_readings=ckpt-code:kappa,ckpt-clause:10+7kappa,reverted-10+7c:10+7c metadata_blocks=largest_actual_empty_publish_slots pools=near0,near2,roomy threads={}",
            CHECKPOINT_COST_GEOMETRIES.len(),
            SEGMENT_ONE_PRODUCT.unwrap_or("none"),
            worker_thread_count()
        ),
    );
    let mismatches = match SEGMENT_ONE_PRODUCT {
        None => vec!["no_segment_one_product".to_string()],
        Some(product) => match std::fs::read_to_string(repository_root().join(product)) {
            Ok(text) => segment_one_mismatches(CHECKPOINT_COST_GEOMETRIES, &text),
            Err(_) => vec!["segment_one_product_unreadable".to_string()],
        },
    };
    tally.halt_check(mismatches.is_empty(), "S5_geometry_table_differs_from_the_segment_one_product");
    emit(emitter, format!("name=s5_geometry_table mismatches={} list={}", mismatches.len(), if mismatches.is_empty() { "none".to_string() } else { mismatches.join(",") }));
    if !mismatches.is_empty() {
        emit(emitter, format!("name=stop clause=s5 halt_conditions={}", tally.halt_conditions.join(",")));
        return;
    }
    emit_the_checkpoint_cost_positive_control(emitter, tally);
    emit_the_geometry_first_requests(emitter, tally, CHECKPOINT_COST_GEOMETRIES);
    let pools = checkpoint_cost_pool_settings(CHECKPOINT_COST_GEOMETRIES);
    let (reverted_pools, judged_pools): (Vec<PoolSettings>, Vec<PoolSettings>) =
        pools.into_iter().partition(|pool| matches!(pool.reserve_reading, ReserveReading::RevertedToTenPlusSevenActualEmptyPublishes { .. }));
    let stuck_ceiling_results = run_groups_in_parallel(&stuck_ceiling_groups_over(&judged_pools));
    let zero_slot_results = run_groups_in_parallel(&zero_slot_groups_over(&judged_pools));
    let reverted_stuck_ceiling_results = run_groups_in_parallel(&stuck_ceiling_groups_over(&reverted_pools));
    let reverted_zero_slot_results = run_groups_in_parallel(&zero_slot_groups_over(&reverted_pools));
    for (segment, stuck_results, zero_results) in [
        ("ckpt-first", &stuck_ceiling_results, &zero_slot_results),
        ("reverted-first", &reverted_stuck_ceiling_results, &reverted_zero_slot_results),
    ] {
        for result in stuck_results.iter() {
            emit(emitter, render_a_cell(segment, &result.not_merged));
        }
        let zero_segment = if segment == "ckpt-first" { "ckpt-second" } else { "reverted-second" };
        emit_the_zero_slot_cells(emitter, zero_segment, zero_results);
    }
    emit_the_stuck_ceiling_judgments(emitter, tally, "ckpt-first", &stuck_ceiling_results);
    emit_the_zero_slot_judgments(emitter, tally, "ckpt-second", &zero_slot_results);
    let by_reserve = {
        let mut merged_readings = readings_by_reserve(&stuck_ceiling_results, &zero_slot_results);
        merged_readings.extend(readings_by_reserve(&reverted_stuck_ceiling_results, &reverted_zero_slot_results));
        merged_readings
    };
    emit_the_reading_splits(emitter, tally, &by_reserve);
    emit_the_zero_point_comparison(emitter, tally, &by_reserve);
    emit_the_reverted_reserve_discrimination(emitter, tally, &by_reserve);
    // 第一段那一组没有读回全零的槽，两条 Q2 臂在每条 Q1 臂上都必须逐项相同；第二段那一组只在不并臂上必须相同（原登记 F5）。
    let every_arm_must_agree: fn(StuckCeilingDisposition) -> bool = |_| true;
    let only_the_unmerged_arm_must_agree: fn(StuckCeilingDisposition) -> bool = |stuck_ceiling| stuck_ceiling == StuckCeilingDisposition::DoNotMergeTheBadSlots;
    for (segment, results, zero_slot_arms_must_agree_for) in [
        ("ckpt-first", &stuck_ceiling_results, every_arm_must_agree),
        ("ckpt-second", &zero_slot_results, only_the_unmerged_arm_must_agree),
        ("reverted-first", &reverted_stuck_ceiling_results, every_arm_must_agree),
        ("reverted-second", &reverted_zero_slot_results, only_the_unmerged_arm_must_agree),
    ] {
        let checks = check_the_segment(results, &zero_slot_arms_must_agree_for);
        emit_the_segment_verdict(emitter, tally, segment, checks);
    }
}
