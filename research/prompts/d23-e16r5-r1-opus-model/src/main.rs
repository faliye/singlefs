//! d23-e16r5-r1 云端攻方（W1）的副本模型。
//!
//! 来源：`research/e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs`（入库装置，一个字节没动）。
//! 这里拷了它的几何、操作流生成器、两条臂的记账与逐行分类（峰值集合容差 0.1%、单调容差 1%、
//! 对角带因子 2，与入库装置同值），另加：
//! - 覆写记账的两种取法：`AsInFifthRun`（入库装置原样：甲按批内次数计、乙不把重新弄脏的单元移出「已落盘」）
//!   与 `Corrected`（甲按批内互异单元计、乙重新弄脏就移出「已落盘」，与原装置 `e16_journal.rs` 的
//!   `persisted.remove(&leaf)` 同义）。两种取法只在一个 checkpoint 窗口里同一单元被写两次时才不同。
//! - 第三条臂丙（E22 记录点名祖先落点的形态）：fsync 只写还没落过的脏单元 + 记录，记录点名这些单元
//!   与它们的祖先；两种点名口径（本次 fsync 去重的祖先 / 每个单元整条路径不去重）。
//! - 把树高、根下孩子数、扇出三个旋钮分开扫的几何点。
//!
//! 只在这份模型上量过、被攻过零轮。
use std::collections::HashSet;

// ── 今天的宽度（与入库装置同值，入库装置注明抄自 crates/singlefs-format/src/lib.rs） ──
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 67;
const INTENT_RECORD_BLOCKS: u64 = 1;
const INTENT_ROOT_SLOT_BLOCKS: u64 = 1;
const PEAK_SET_TOLERANCE: f64 = 0.001;
const MONOTONIC_TOLERANCE: f64 = 0.01;
const NEAR_DIAGONAL_BAND_FACTOR: f64 = 2.0;

// 入库装置里的几何单元数（抄自入库装置第 164-168 行）
const FOUR_GIBIBYTE_FILE_GEOMETRY_UNITS: u64 = 131_611;
const ONE_TEBIBYTE_FILE_GEOMETRY_UNITS: u64 = 33_692_212;
const FULL_ROOT_LEVEL_FOUR_GEOMETRY_UNITS: u64 = 457_419_312;
const SIXTEEN_TEBIBYTE_FILE_GEOMETRY_UNITS: u64 = 539_075_383;
const TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY_UNITS: u64 = 8_226;

#[derive(Clone)]
struct Geometry {
    name: String,
    leaf_units: u64,
    fanout: u64,
    units: u64,
}

fn span_in_units(leaf_units: u64, fanout: u64, level: u32) -> u64 {
    leaf_units * fanout.pow(level)
}

fn root_level_for(leaf_units: u64, fanout: u64, units: u64) -> u32 {
    let mut level = 0u32;
    while span_in_units(leaf_units, fanout, level) < units {
        level += 1;
    }
    level
}

impl Geometry {
    fn new(name: &str, leaf_units: u64, fanout: u64, units: u64) -> Geometry {
        Geometry { name: name.to_string(), leaf_units, fanout, units }
    }
    fn root_level(&self) -> u32 {
        root_level_for(self.leaf_units, self.fanout, self.units)
    }
    fn node_levels(&self) -> u32 {
        self.root_level() + 1
    }
    fn root_children(&self) -> u64 {
        let root_level = self.root_level();
        if root_level == 0 {
            1
        } else {
            self.units.div_ceil(span_in_units(self.leaf_units, self.fanout, root_level - 1))
        }
    }
    fn ancestor_indices(&self, unit: u64) -> Vec<(u32, u64)> {
        let root_level = self.root_level();
        (0..=root_level).map(|level| (level, unit / span_in_units(self.leaf_units, self.fanout, level))).collect()
    }
    /// 树高 `node_levels`、根下孩子数 `root_children`、扇出 `fanout`（叶罩 `leaf_units`）三者分开给定的几何点：
    /// 单元数取 `root_children × 根往下一层的跨度`，于是根下孩子数恰好是给定值、树高恰好是给定值。
    fn with_separated_knobs(leaf_units: u64, fanout: u64, node_levels: u32, root_children: u64) -> Geometry {
        assert!(node_levels >= 2, "树高至少 2 层才有「根下孩子」");
        assert!(root_children >= 2 && root_children <= fanout, "根下孩子数要在 2..=扇出 之间，树高才不变");
        let units = root_children * span_in_units(leaf_units, fanout, node_levels - 2);
        let name = format!("separated_leaf{leaf_units}_fanout{fanout}_levels{node_levels}_rootchildren{root_children}");
        let geometry = Geometry::new(&name, leaf_units, fanout, units);
        assert_eq!(geometry.node_levels(), node_levels, "几何点 {name} 的树高算出来不是给定值");
        assert_eq!(geometry.root_children(), root_children, "几何点 {name} 的根下孩子数算出来不是给定值");
        geometry
    }
}

fn deduplicated_ancestor_count(geometry: &Geometry, units: &[u64]) -> u64 {
    let mut ancestors: HashSet<(u32, u64)> = HashSet::new();
    for &unit in units {
        for pair in geometry.ancestor_indices(unit) {
            ancestors.insert(pair);
        }
    }
    ancestors.len() as u64
}

/// 覆写记账：一个 checkpoint 窗口里同一单元被写两次时，两条臂怎么记。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OverwriteAccounting {
    /// 入库装置原样：甲按批内操作次数计单元块（重复的也算），乙重新弄脏的单元不移出「已落盘」。
    AsInFifthRun,
    /// 甲按批内互异单元计，乙重新弄脏就移出「已落盘」（原装置 `e16_journal.rs` 的写法）。
    Corrected,
}

fn checkpoint_interval_for_batch(batch: u64) -> u64 {
    batch * 2000u64.div_ceil(batch)
}

fn operation_count_for_batch(batch: u64) -> u64 {
    10 * checkpoint_interval_for_batch(batch)
}

fn unit_for_cursor(cursor: u64, stream_count: u64, units_per_stream: u64, stream_index: u64) -> u64 {
    let offset_in_stream = (cursor / stream_count) % units_per_stream;
    stream_index * units_per_stream + offset_in_stream
}

/// 一格的四条臂每次 fsync 的块数平均值，外加「窗口里有没有同一单元被写两次」。
struct CellResult {
    streams: u64,
    batch: u64,
    intent_average: f64,
    write_ahead_log_leaf_average: f64,
    third_arm_deduplicated_average: f64,
    third_arm_whole_path_average: f64,
    overwrite_seen: bool,
}

impl CellResult {
    fn ratio_intent_over_write_ahead_log_leaf(&self) -> f64 {
        self.intent_average / self.write_ahead_log_leaf_average
    }
    fn ratio_intent_over_third_arm_deduplicated(&self) -> f64 {
        self.intent_average / self.third_arm_deduplicated_average
    }
    fn ratio_intent_over_third_arm_whole_path(&self) -> f64 {
        self.intent_average / self.third_arm_whole_path_average
    }
    /// 每次 fsync 的块数差：甲 − 乙（绝对差，不除以批）。
    fn difference_per_fsync_intent_minus_write_ahead_log_leaf(&self) -> f64 {
        self.intent_average - self.write_ahead_log_leaf_average
    }
}

fn average_of(samples: &[u64]) -> f64 {
    assert!(!samples.is_empty(), "空的代价序列——上游没跑到任何一次 fsync");
    samples.iter().sum::<u64>() as f64 / samples.len() as f64
}

/// 一格：甲、乙两条臂的记账逐行照入库装置 `simulate_cell`（第 295-340 行），`accounting` 选覆写的记法；
/// 丙两种点名口径与乙共用「还没落过的脏单元」这一集合，只是记录里多点名祖先。
fn simulate_cell(geometry: &Geometry, stream_count: u64, batch: u64, accounting: OverwriteAccounting) -> CellResult {
    let checkpoint_interval = checkpoint_interval_for_batch(batch);
    let operation_count = operation_count_for_batch(batch);
    let units_per_stream = geometry.units / stream_count;
    assert!(units_per_stream >= 1, "几何点 {} 的单元数 {} 不够分给 {stream_count} 条流", geometry.name, geometry.units);

    let mut intent_batch_units: Vec<u64> = Vec::with_capacity(batch as usize);
    let mut write_ahead_log_leaf_dirty: HashSet<u64> = HashSet::new();
    let mut write_ahead_log_leaf_persisted: HashSet<u64> = HashSet::new();
    let mut overwrite_seen = false;

    let mut intent_costs = Vec::new();
    let mut write_ahead_log_leaf_costs = Vec::new();
    let mut third_arm_deduplicated_costs = Vec::new();
    let mut third_arm_whole_path_costs = Vec::new();

    for cursor in 0..operation_count {
        let stream_index = cursor % stream_count;
        let unit = unit_for_cursor(cursor, stream_count, units_per_stream, stream_index);
        intent_batch_units.push(unit);
        let newly_dirty = write_ahead_log_leaf_dirty.insert(unit);
        if !newly_dirty {
            overwrite_seen = true;
            match accounting {
                OverwriteAccounting::AsInFifthRun => {}
                OverwriteAccounting::Corrected => {
                    write_ahead_log_leaf_persisted.remove(&unit);
                }
            }
        }

        let is_fsync = (cursor + 1) % batch == 0;
        let is_checkpoint = (cursor + 1) % checkpoint_interval == 0;

        if is_fsync {
            let intent_leaf_blocks = match accounting {
                OverwriteAccounting::AsInFifthRun => intent_batch_units.len() as u64,
                OverwriteAccounting::Corrected => intent_batch_units.iter().collect::<HashSet<_>>().len() as u64,
            };
            let ancestor_count = deduplicated_ancestor_count(geometry, &intent_batch_units);
            intent_costs.push(intent_leaf_blocks + ancestor_count + INTENT_RECORD_BLOCKS + INTENT_ROOT_SLOT_BLOCKS);
            intent_batch_units.clear();

            let unpersisted: Vec<u64> = write_ahead_log_leaf_dirty.difference(&write_ahead_log_leaf_persisted).copied().collect();
            let unpersisted_count = unpersisted.len() as u64;
            write_ahead_log_leaf_costs.push(unpersisted_count + unpersisted_count.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD));

            let named_ancestors = if unpersisted.is_empty() { 0 } else { deduplicated_ancestor_count(geometry, &unpersisted) };
            let deduplicated_entries = unpersisted_count + named_ancestors;
            third_arm_deduplicated_costs.push(unpersisted_count + deduplicated_entries.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD));
            let whole_path_entries = unpersisted_count * (geometry.node_levels() as u64 + 1);
            third_arm_whole_path_costs.push(unpersisted_count + whole_path_entries.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD));

            write_ahead_log_leaf_persisted = write_ahead_log_leaf_dirty.clone();
        }
        if is_checkpoint {
            write_ahead_log_leaf_dirty.clear();
            write_ahead_log_leaf_persisted.clear();
        }
    }

    CellResult {
        streams: stream_count,
        batch,
        intent_average: average_of(&intent_costs),
        write_ahead_log_leaf_average: average_of(&write_ahead_log_leaf_costs),
        third_arm_deduplicated_average: average_of(&third_arm_deduplicated_costs),
        third_arm_whole_path_average: average_of(&third_arm_whole_path_costs),
        overwrite_seen,
    }
}

const STREAM_AXIS: [u64; 21] = [2, 3, 4, 6, 7, 8, 11, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024];
const BATCH_AXIS_BASE: [u64; 53] = [
    1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 14, 16, 20, 24, 28, 32, 40, 48, 50, 56, 64, 67, 68, 70, 72, 73, 80, 96, 100,
    112, 128, 160, 192, 200, 224, 256, 320, 384, 448, 512, 640, 768, 1024, 1280, 1536, 2048, 3072, 4096, 6144, 8192,
    12288, 16384,
];

/// 批轴：与入库装置 `batch_axis_for` 同（基础表 ∪ 端点扩展 ∪ {S−1, S, S+1}）。
fn batch_axis_for(stream_count: u64) -> Vec<u64> {
    let mut batches: Vec<u64> = BATCH_AXIS_BASE.to_vec();
    if stream_count > 1 {
        batches.push(stream_count - 1);
    }
    batches.push(stream_count);
    batches.push(stream_count + 1);
    batches.sort_unstable();
    batches.dedup();
    batches
}

/// 逐行分类，判法照入库装置 `peak_set` 与 `classify_row` 的分类那一段（第 490-521 行）。
/// 名字照入库装置 `classification_name`：「丙行」是「峰值集合下界是批=1」（从批=1 起就不再上升），与臂丙无关。
fn classify_row(streams: u64, cells: &[CellResult], ratio_of: fn(&CellResult) -> f64) -> RowSummary {
    let mut sorted_cells: Vec<&CellResult> = cells.iter().collect();
    sorted_cells.sort_by_key(|cell| cell.batch);
    let peak_ratio = sorted_cells.iter().map(|cell| ratio_of(cell)).fold(f64::MIN, f64::max);
    let threshold = peak_ratio * (1.0 - PEAK_SET_TOLERANCE);
    let in_peak_set: Vec<u64> = sorted_cells.iter().filter(|cell| ratio_of(cell) >= threshold).map(|cell| cell.batch).collect();
    let peak_batch_low = *in_peak_set.iter().min().expect("峰值集合至少有最大值那一格");
    let peak_batch_high = *in_peak_set.iter().max().expect("峰值集合至少有最大值那一格");
    let batch_axis_top = sorted_cells.last().expect("这一行没有格").batch;
    let band_lower = streams as f64 / NEAR_DIAGONAL_BAND_FACTOR;
    let band_upper = streams as f64 * NEAR_DIAGONAL_BAND_FACTOR;
    let low_in_band = (peak_batch_low as f64) >= band_lower && (peak_batch_low as f64) <= band_upper;
    let high_in_band = (peak_batch_high as f64) >= band_lower && (peak_batch_high as f64) <= band_upper;
    let overlaps_band = (peak_batch_low as f64) <= band_upper && (peak_batch_high as f64) >= band_lower;
    let classification = if peak_batch_high == batch_axis_top {
        "端点"
    } else if peak_batch_low == 1 {
        "丙行"
    } else if low_in_band && high_in_band {
        "甲行"
    } else if !overlaps_band {
        "乙行"
    } else {
        "宽平台"
    };
    let first_cell = sorted_cells.first().expect("这一行没有格");
    let lowest_cell = sorted_cells.iter().min_by(|left_cell, right_cell| ratio_of(left_cell).total_cmp(&ratio_of(right_cell))).expect("这一行没有格");
    // 批 ≤ 流数这一段里有没有跌过 1% 以上（入库装置 Q5 同判法）
    let mut falls_within_batch_at_most_streams = false;
    for window in sorted_cells.windows(2) {
        if window[1].batch <= streams && ratio_of(window[1]) < ratio_of(window[0]) * (1.0 - MONOTONIC_TOLERANCE) {
            falls_within_batch_at_most_streams = true;
        }
    }
    RowSummary {
        streams,
        classification,
        peak_ratio,
        peak_batch_low,
        peak_batch_high,
        batch_one_ratio: ratio_of(first_cell),
        lowest_ratio: ratio_of(lowest_cell),
        lowest_batch: lowest_cell.batch,
        falls_within_batch_at_most_streams,
        overwrite_cells: sorted_cells.iter().filter(|cell| cell.overwrite_seen).count(),
    }
}

struct RowSummary {
    streams: u64,
    classification: &'static str,
    peak_ratio: f64,
    peak_batch_low: u64,
    peak_batch_high: u64,
    batch_one_ratio: f64,
    lowest_ratio: f64,
    lowest_batch: u64,
    falls_within_batch_at_most_streams: bool,
    overwrite_cells: usize,
}

fn row_summary_line(label: &str, geometry: &Geometry, arm_pair: &str, row: &RowSummary) -> String {
    format!(
        "name={label} geometry={} node_levels={} root_children={} fanout={} arm_pair={arm_pair} streams={} classification={} \
         peak_ratio={:.6} peak_batch_low={} peak_batch_high={} batch_one_ratio={:.6} lowest_ratio={:.6} lowest_batch={} \
         falls_within_batch_at_most_streams={} overwrite_cells={}",
        geometry.name, geometry.node_levels(), geometry.root_children(), geometry.fanout, row.streams, row.classification,
        row.peak_ratio, row.peak_batch_low, row.peak_batch_high, row.batch_one_ratio, row.lowest_ratio, row.lowest_batch,
        row.falls_within_batch_at_most_streams, row.overwrite_cells
    )
}

fn geometry_line(geometry: &Geometry, accounting: OverwriteAccounting) -> String {
    format!(
        "name=geometry_config geometry={} units={} leaf_units={} fanout={} root_level={} node_levels={} root_children={} accounting={:?}",
        geometry.name, geometry.units, geometry.leaf_units, geometry.fanout, geometry.root_level(), geometry.node_levels(),
        geometry.root_children(), accounting
    )
}

/// 一个几何点的全部行：流数轴上单元数够分的每个流数 × 它的批轴。
fn compute_rows(geometry: &Geometry, accounting: OverwriteAccounting) -> Vec<(u64, Vec<CellResult>)> {
    STREAM_AXIS
        .iter()
        .filter(|&&streams| geometry.units / streams >= 1)
        .map(|&streams| (streams, batch_axis_for(streams).into_iter().map(|batch| simulate_cell(geometry, streams, batch, accounting)).collect()))
        .collect()
}

/// 输出累加器：每写一行计一次数，最后一行报总行数（完整性闸，同入库装置 `name=done emitted=`）。
struct LineSink {
    lines: Vec<String>,
}

impl LineSink {
    fn push(&mut self, line: String) {
        self.lines.push(format!("OPUSMODEL {line}"));
    }
    fn finish_and_print(mut self) {
        let emitted = self.lines.len() + 1;
        self.lines.push(format!("OPUSMODEL name=done emitted={emitted}"));
        println!("{}", self.lines.join("\n"));
    }
}

/// 入库装置第一、二段跑过的七个几何点（名字与产物里的 `geometry=` 相同，扇出与叶罩照入库装置第 171-189 行）。
fn fifth_run_geometries() -> Vec<Geometry> {
    let today = |name: &str, units: u64| Geometry::new(name, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_INTERNAL_FANOUT, units);
    vec![
        today("four_gibibyte_file", FOUR_GIBIBYTE_FILE_GEOMETRY_UNITS),
        today("one_tebibyte_file", ONE_TEBIBYTE_FILE_GEOMETRY_UNITS),
        today("sixteen_tebibyte_file", SIXTEEN_TEBIBYTE_FILE_GEOMETRY_UNITS),
        today("full_root_level_four", FULL_ROOT_LEVEL_FOUR_GEOMETRY_UNITS),
        today("two_fifty_six_mebibyte_file", TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY_UNITS),
        Geometry::new("small_fanout_one_tebibyte_file", 128, 128, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS),
        Geometry::new("large_fanout_one_tebibyte_file", 296, 296, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS),
    ]
}

/// 三个旋钮分开扫：每组只动一个旋钮，另两个钉住。
fn separated_knob_geometries(knob: &str) -> Vec<Geometry> {
    let today = |node_levels: u32, root_children: u64| {
        Geometry::with_separated_knobs(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_INTERNAL_FANOUT, node_levels, root_children)
    };
    match knob {
        // 树高钉 4、扇出钉今天的 144/147，只动根下孩子数
        "root_children_knob" => [2u64, 4, 7, 11, 17, 24, 32, 48, 58, 96, 147].iter().map(|&root_children| today(4, root_children)).collect(),
        // 根下孩子数钉 2 / 11 / 58 / 147、扇出钉今天的 144/147，只动树高
        "height_knob" => [2u64, 11, 58, 147]
            .iter()
            .flat_map(|&root_children| [2u32, 3, 4, 5, 6].into_iter().map(move |node_levels| (node_levels, root_children)))
            .map(|(node_levels, root_children)| today(node_levels, root_children))
            .collect(),
        // 树高钉 4、根下孩子数钉 2 / 11，只动扇出（叶罩与扇出一起换，同入库装置的 128/128、296/296 两点）
        "fanout_knob" => [2u64, 11]
            .iter()
            .flat_map(|&root_children| [(128u64, 128u64), (144, 147), (296, 296)].into_iter().map(move |(leaf_units, fanout)| (leaf_units, fanout, root_children)))
            .map(|(leaf_units, fanout, root_children)| Geometry::with_separated_knobs(leaf_units, fanout, 4, root_children))
            .collect(),
        other => panic!("认不出的旋钮 {other}：只有 root_children_knob、height_knob、fanout_knob"),
    }
}

const ARM_PAIRS: [(&str, fn(&CellResult) -> f64); 3] = [
    ("intent_over_write_ahead_log_leaf", CellResult::ratio_intent_over_write_ahead_log_leaf),
    ("intent_over_third_arm_deduplicated", CellResult::ratio_intent_over_third_arm_deduplicated),
    ("intent_over_third_arm_whole_path", CellResult::ratio_intent_over_third_arm_whole_path),
];

/// 跑一组几何点：几何配置行、（可选）逐格行、三种臂对各自的逐行分类。
fn run_geometries(sink: &mut LineSink, label: &str, geometries: &[Geometry], accounting: OverwriteAccounting, emit_cells: bool) {
    for geometry in geometries {
        sink.push(geometry_line(geometry, accounting));
        let rows = compute_rows(geometry, accounting);
        if emit_cells {
            for (_streams, cells) in &rows {
                for cell in cells {
                    sink.push(format!(
                        "name=cell geometry={} streams={} batch={} intent_avg={:.4} write_ahead_log_leaf_avg={:.4} ratio={:.6} \
                         third_arm_deduplicated_avg={:.4} third_arm_whole_path_avg={:.4} difference_per_fsync={:.4} overwrite_seen={}",
                        geometry.name, cell.streams, cell.batch, cell.intent_average, cell.write_ahead_log_leaf_average,
                        cell.ratio_intent_over_write_ahead_log_leaf(), cell.third_arm_deduplicated_average, cell.third_arm_whole_path_average,
                        cell.difference_per_fsync_intent_minus_write_ahead_log_leaf(), cell.overwrite_seen
                    ));
                }
            }
        }
        for (arm_pair, ratio_of) in ARM_PAIRS {
            for (streams, cells) in &rows {
                sink.push(row_summary_line(label, geometry, arm_pair, &classify_row(*streams, cells, ratio_of)));
            }
        }
    }
}

/// 绝对差读法：每行在几个批上的「甲 − 乙」每次 fsync 块数，以及它除以批之后的每操作差。
fn run_difference_mode(sink: &mut LineSink, geometry: &Geometry) {
    sink.push(geometry_line(geometry, OverwriteAccounting::AsInFifthRun));
    for (streams, cells) in compute_rows(geometry, OverwriteAccounting::AsInFifthRun) {
        let at = |batch: u64| cells.iter().find(|cell| cell.batch == batch).expect("批轴上没有这个批");
        let largest_difference_cell = cells.iter().max_by(|left_cell, right_cell| {
            left_cell.difference_per_fsync_intent_minus_write_ahead_log_leaf().total_cmp(&right_cell.difference_per_fsync_intent_minus_write_ahead_log_leaf())
        }).expect("这一行没有格");
        let mut probe_batches = vec![1u64, 2, streams, 4 * streams, 16 * streams, 16384];
        probe_batches.retain(|batch| *batch <= 16384 && batch_axis_for(streams).contains(batch));
        probe_batches.dedup();
        let probes = probe_batches
            .iter()
            .map(|&batch| {
                let cell = at(batch);
                format!("B{batch}:{:.1}/{:.4}/{:.4}", cell.difference_per_fsync_intent_minus_write_ahead_log_leaf(),
                        cell.difference_per_fsync_intent_minus_write_ahead_log_leaf() / batch as f64, cell.ratio_intent_over_write_ahead_log_leaf())
            })
            .collect::<Vec<_>>()
            .join(",");
        sink.push(format!(
            "name=difference_row geometry={} streams={streams} largest_difference_per_fsync={:.1} at_batch={} \
             probes(batch:difference_per_fsync/difference_per_operation/ratio)={probes}",
            geometry.name, largest_difference_cell.difference_per_fsync_intent_minus_write_ahead_log_leaf(), largest_difference_cell.batch
        ));
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mode = arguments.first().map(String::as_str).unwrap_or("");
    let selector = arguments.get(1).map(String::as_str).unwrap_or("");
    let mut sink = LineSink { lines: Vec::new() };
    match mode {
        // 入库装置原样记账，重算第五次跑的几何点，逐格与产物比（阳性对照）；第二个参数只跑同名的那一个几何点
        "reproduce" => {
            let geometries: Vec<Geometry> = fifth_run_geometries().into_iter().filter(|geometry| selector.is_empty() || geometry.name == selector).collect();
            assert!(!geometries.is_empty(), "没有叫 {selector} 的几何点");
            run_geometries(&mut sink, "row_summary", &geometries, OverwriteAccounting::AsInFifthRun, true);
        }
        // 三个旋钮分开扫，覆写按 Corrected 记（大几何上两种记法逐格相同，小几何上只有 Corrected 对）
        "separated" => {
            let geometries = separated_knob_geometries(selector);
            run_geometries(&mut sink, &format!("row_summary_{selector}"), &geometries, OverwriteAccounting::Corrected, false);
        }
        // 256 MiB 那一点按 Corrected 重记，与 reproduce 的 AsInFifthRun 对照
        "overwrite" => {
            let geometries: Vec<Geometry> = fifth_run_geometries().into_iter().filter(|geometry| geometry.name == "two_fifty_six_mebibyte_file").collect();
            run_geometries(&mut sink, "row_summary_corrected", &geometries, OverwriteAccounting::Corrected, true);
        }
        // 绝对差读法：1 TiB 真实基线每行的「甲 − 乙」
        "difference" => {
            let geometry = Geometry::new("one_tebibyte_file", EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_INTERNAL_FANOUT, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
            run_difference_mode(&mut sink, &geometry);
        }
        other => {
            eprintln!("认不出的模式 {other:?}：reproduce [几何名] | separated root_children_knob|height_knob|fanout_knob | overwrite | difference");
            std::process::exit(2);
        }
    }
    sink.finish_and_print();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separated_knob_geometry_pins_height_and_root_children() {
        let geometry = Geometry::with_separated_knobs(144, 147, 4, 11);
        assert_eq!(geometry.units, 11 * 144 * 147 * 147);
        assert_eq!(geometry.node_levels(), 4);
        assert_eq!(geometry.root_children(), 11);
        let taller = Geometry::with_separated_knobs(144, 147, 6, 11);
        assert_eq!(taller.node_levels(), 6);
        assert_eq!(taller.root_children(), 11);
    }

    #[test]
    fn batch_one_difference_is_node_levels_plus_one_and_third_arms_equal_write_ahead_log_leaf() {
        let geometry = Geometry::new("one_tebibyte_file", 144, 147, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
        let cell = simulate_cell(&geometry, 64, 1, OverwriteAccounting::AsInFifthRun);
        assert_eq!(cell.intent_average, 1.0 + 4.0 + 2.0);
        assert_eq!(cell.write_ahead_log_leaf_average, 2.0);
        assert_eq!(cell.third_arm_deduplicated_average, 2.0);
        assert_eq!(cell.third_arm_whole_path_average, 2.0);
    }

    #[test]
    fn two_accountings_differ_only_when_a_unit_is_written_twice_in_a_window() {
        let large = Geometry::new("one_tebibyte_file", 144, 147, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
        let as_in_fifth_run = simulate_cell(&large, 8, 64, OverwriteAccounting::AsInFifthRun);
        let corrected = simulate_cell(&large, 8, 64, OverwriteAccounting::Corrected);
        assert!(!as_in_fifth_run.overwrite_seen);
        assert_eq!(as_in_fifth_run.intent_average, corrected.intent_average);
        assert_eq!(as_in_fifth_run.write_ahead_log_leaf_average, corrected.write_ahead_log_leaf_average);
        let small = Geometry::new("two_fifty_six_mebibyte_file", 144, 147, TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY_UNITS);
        let as_in_fifth_run_small = simulate_cell(&small, 2, 16384, OverwriteAccounting::AsInFifthRun);
        let corrected_small = simulate_cell(&small, 2, 16384, OverwriteAccounting::Corrected);
        assert!(as_in_fifth_run_small.overwrite_seen);
        // 入库装置把一批 16384 次操作记成 16384 个单元块，而这个文件只有 8226 个单元
        assert!(as_in_fifth_run_small.intent_average > 16384.0);
        assert!(corrected_small.intent_average < 8226.0 + 100.0);
    }
}
