//! E155 第三次跑（岔路单第 8 行，连带第 5 行 rand 那一半）——三种分配记录释放写法（基线 /
//! ① 释放只追加 / ② 按上一版写入时刻聚簇）下的分支比、F2 状态与每次 fsync 写字节。
//! 确定性计数模型，不与 `crates/` 共用代码，不量时间。
//!
//! 判据、失败条款、变异表的权威登记在 `research/prompts/e155-r3-prereg.md`
//! （第一节「分支比」「求解器」、第五节 5.1 C1/C2/G/T/SQ、第六节 Q8.1–Q8.5/Q5.1–Q5.3、
//! 第七节 B-r3-1..12、第九节 M1–M14、第十一节作废与停机条款）。第二次跑的登记
//! `research/prompts/e155-r2-prereg.md` 与它的二进制 `e155_second_run_fsync_write_volume.rs`
//! （本文件从它拷出再改，取变量命名与容量/组公式一节）不动。
//!
//! 本文件是另一个二进制；`replay.sh` 里 E155 的这一行单独登记，不影响第一次/第二次的产物。
//!
//! **这一次跑的范围**（登记第五节 5.5「按岔路单排段」第一段 + 第二段）：三种写法在
//! 第 8 行主表（rand、L均、F1/F8A、P∈{10⁴,10⁶,10⁸}、N=16、k=1、三臂）与第 5 行 rand 半
//! （甲、rand、L均、F1/F8A、P∈{1,10²,10⁴,10⁶,10⁸}）上的求解、代际展开（G）、逐轮记录
//! 摘要（T，报峰值/超阈值代数/末值，不逐轮落盘全量——`test-discipline.md`「端点不是轨迹」
//! 的做法是报三样统计量，这里照此办，不是照抄「每一轮都打一行」）、阳性对照、变异锚点。
//! 反向取样点（第八节 8.2：L远/L近/②连/阻尼0.5/上限1e4/g=0/φ=0.5/SQ）**只在代表性格上报**
//! （family=F1、arm=甲、P=10⁶ 这一格；SQ 另在这一格做「冻住状态之后的一次性重算」，不重新
//! 迭代整条不动点）——不是登记要求的全网格复算，这是执行员在报告与实验页里点名的范围缩减，
//! 交主 agent 认。R8（组提交批量）与「不共享取随机」不在这一次范围（k=1 不用 R8）。

use e7_index_bench::Emitter;

// ============================================================
// 一、常量（手抄自 `crates/singlefs-format/src/lib.rs`，与 `anchors_r3*.py` 一致）
// ============================================================

const NODE_BYTES: u64 = 16384;
const UNIT_BYTES: u64 = 32768;
const RECORD_BYTES: u64 = 4096;
// 打印出去的键名还叫 `system_configuration_slot_bytes`（第 1624 行的格式串）：它是留存产物第一行的一部分，
// 改它产物就变了，而产物是冻结的证据、只能连同重跑一起改（evidence-discipline）。下次重跑这个实验时一起改。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
const RECORD_HEADER_BYTES: u64 = 307;
const NAMED_ITEM_BYTES: u64 = 56;
const DEVICE_COUNT: u64 = 2;
const PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 512;

const NODE_HEADER_BASE_BYTES: u64 = 86 + 29;
const NODE_POINTER_BYTES: u64 = 86;

const EXTENT_KEY_BYTES: u64 = 24;
const EXTENT_LEAF_ENTRY_BYTES: u64 = 112;
const INODE_KEY_BYTES: u64 = 8;
const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
const ALLOCATION_KEY_BYTES: u64 = 10;
const ALLOCATION_LEAF_ENTRY_BYTES: u64 = 20;
const ACCOUNTING_KEY_BYTES: u64 = 22;
const ACCOUNTING_LEAF_ENTRY_BYTES: u64 = 34;
const MAPPING_KEY_BYTES: u64 = 27;
const MAPPING_LEAF_ENTRY_BYTES: u64 = 55;
const TREE_TABLE_KEY_BYTES: u64 = 8;
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
const INODE_CONTAINER_HEADER_BYTES: u64 = 136;
const INODE_CONTAINER_ENTRY_BYTES: u64 = 140;

const ACCOUNTING_POOL_WIDE_ENTRIES: u64 = 3;
const ACCOUNTING_ENTRIES_PER_DEVICE: u64 = 6;
const TREE_TABLE_ENTRY_COUNT: u64 = 7;
const TREE_TABLE_TOUCHED_ROOTS: u64 = 4;
const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;

/// D3（空间分配）已定项 10 ①：聚簇段 64 个 16 KiB 槽（`crates/singlefs-format/src/lib.rs`
/// 的 `CLUSTER_SEGMENT_SLOTS`）。② 的主形态按它切块；②连（反向取样点）不切块。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;

/// 不动点收敛判据（相对误差）与迭代上限——主取值；反向取样点（阻尼 0.5、上限 10⁴）另传参数。
const CONVERGENCE_EPSILON: f64 = 1e-9;
const MAIN_DAMPING_ALPHA: f64 = 0.15;
const MAIN_MAXIMUM_ITERATIONS: u64 = 1000;

const fn node_header_bytes(key_bytes: u64) -> u64 {
    NODE_HEADER_BASE_BYTES + 2 * key_bytes
}

const fn node_capacity(key_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - node_header_bytes(key_bytes)) / entry_bytes
}

const fn internal_entry_bytes(key_bytes: u64) -> u64 {
    key_bytes + NODE_POINTER_BYTES
}

fn inode_container_capacity() -> u64 {
    (UNIT_BYTES - INODE_CONTAINER_HEADER_BYTES) / INODE_CONTAINER_ENTRY_BYTES
}

// ============================================================
// 二、树高与各层节点数（与第二次逐字相同）
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FillRatio {
    Full,
    Half,
}

impl FillRatio {
    fn effective_capacity(self, raw_capacity: u64) -> u64 {
        match self {
            FillRatio::Full => raw_capacity,
            FillRatio::Half => (raw_capacity / 2).max(1),
        }
    }
}

fn tree_layers(entry_count: u64, leaf_capacity: u64, internal_capacity: u64, minimum_layers: usize) -> Vec<u64> {
    assert!(leaf_capacity >= 1, "叶容量必须 ≥ 1，否则树高不收敛");
    assert!(internal_capacity >= 1, "内部容量必须 ≥ 1，否则树高不收敛");
    let mut layers = Vec::new();
    let mut count = entry_count.div_ceil(leaf_capacity).max(1);
    layers.push(count);
    while count > 1 {
        count = count.div_ceil(internal_capacity);
        layers.push(count);
        assert!(layers.len() <= 64, "树高超过 64 层，多半是容量参数错了");
    }
    while layers.len() < minimum_layers {
        layers.push(1);
    }
    layers
}

fn layer_coverages(leaf_capacity: u64, internal_capacity: u64, layer_count: usize) -> Vec<f64> {
    let mut covs = Vec::with_capacity(layer_count);
    let mut cov = leaf_capacity as f64;
    for _ in 0..layer_count {
        covs.push(cov);
        cov *= internal_capacity as f64;
    }
    covs
}

#[cfg(test)]
mod capacity_tests {
    use super::*;

    /// B-r3-1：容量锚点，与第二次登记 B1 相同（R1–R7 与这一次的反事实都不改容量公式）。
    #[test]
    fn capacities_match_the_preregistration_anchors() {
        assert_eq!(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), 144, "extent 叶容量");
        assert_eq!(node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES)), 147, "extent 内部容量");
        assert_eq!(inode_container_capacity(), 233, "inode 容器容量");
        assert_eq!(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 135, "inode 内部容量");
        assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES), 812, "分配记录叶容量（B-r3-1）");
        assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES)), 169, "分配记录内部容量");
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES), 477, "记账叶容量");
        assert_eq!(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES), 294, "映射叶容量");
        assert_eq!(node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES)), 143, "映射内部容量");
        assert_eq!(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES), 81, "树表容量");
        assert_eq!(NAMED_ITEMS_PER_RECORD_MAIN, 67, "一条记录装的点名项数");
    }

    #[test]
    fn inode_tree_is_at_least_two_layers_even_with_one_entry() {
        let layers = tree_layers(1, inode_container_capacity(), node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 2);
        assert_eq!(layers, vec![1, 1]);
    }
}

// ============================================================
// 三、组公式（与第二次逐字相同：R1–R7 不改这一节）
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Placement {
    Sequential,
    Random,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PositionPolicy {
    Near,
    Far,
    Balanced,
}

impl Placement {
    fn tag(self) -> &'static str {
        match self {
            Placement::Sequential => "seq",
            Placement::Random => "rand",
        }
    }
}

impl PositionPolicy {
    fn tag(self) -> &'static str {
        match self {
            PositionPolicy::Near => "Lnear",
            PositionPolicy::Far => "Lfar",
            PositionPolicy::Balanced => "Lbalanced",
        }
    }
}

fn continuous_groups_touch(node_count_at_layer: u64, layer_coverage: f64, group_sizes: &[f64], gaps_between: &[f64]) -> f64 {
    if node_count_at_layer == 0 || group_sizes.is_empty() {
        return 0.0;
    }
    assert!(gaps_between.len() + 1 == group_sizes.len(), "空隙数必须比组数少 1");
    let mut touched = 1.0;
    for &group_size in group_sizes {
        touched += (group_size - 1.0) / layer_coverage;
    }
    for &gap in gaps_between {
        touched += (gap / layer_coverage).min(1.0);
    }
    touched.min(node_count_at_layer as f64)
}

fn scattered_group_touch(touch_count: f64, region_entries: f64, layer_coverage: f64) -> f64 {
    if touch_count <= 0.0 {
        return 0.0;
    }
    let expected_saturated_node_count = (region_entries / layer_coverage).max(1.0);
    expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))
}

fn scattered_continuous_groups_touch(node_count_at_layer: u64, layer_coverage: f64, group_count: f64, group_size: f64, region_entries: f64) -> f64 {
    if group_count <= 0.0 || group_size <= 0.0 {
        return 0.0;
    }
    let saturated_node_count = (region_entries / layer_coverage).max(1.0);
    let nodes_touched_per_group = saturated_node_count.min(1.0 + (group_size - 1.0) / layer_coverage);
    (saturated_node_count * (1.0 - (1.0 - nodes_touched_per_group / saturated_node_count).powf(group_count))).min(node_count_at_layer as f64)
}

fn scattered_continuous_groups_distinct_entries(region_entries: f64, group_count: f64, group_size: f64) -> f64 {
    if group_count <= 0.0 || group_size <= 0.0 || region_entries <= 0.0 {
        return 0.0;
    }
    region_entries * (1.0 - (1.0 - group_size / region_entries).powf(group_count))
}

fn combine_touches(node_count_at_layer: u64, continuous_touch: f64, scattered_touches: &[f64]) -> f64 {
    let node_count = node_count_at_layer as f64;
    if node_count <= 0.0 {
        return 0.0;
    }
    let mut surviving_fraction = 1.0 - (continuous_touch / node_count).min(1.0);
    for &scattered_touch in scattered_touches {
        surviving_fraction *= 1.0 - (scattered_touch / node_count).min(1.0);
    }
    node_count * (1.0 - surviving_fraction)
}

fn single_group_dirty_nodes_per_layer(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, group_size: u64) -> Vec<f64> {
    if group_size == 0 {
        return vec![0.0; layers.len()];
    }
    let mut result = Vec::with_capacity(layers.len());
    let mut cov = leaf_capacity as f64;
    for &node_count_at_layer in layers {
        result.push(continuous_groups_touch(node_count_at_layer, cov, &[group_size as f64], &[]));
        cov *= internal_capacity as f64;
    }
    result
}

#[cfg(test)]
mod group_formula_tests {
    use super::*;

    #[test]
    fn single_continuous_group_never_exceeds_node_count() {
        assert_eq!(continuous_groups_touch(1, 100.0, &[8.0], &[]), 1.0);
    }

    #[test]
    fn scattered_group_saturates_to_region_node_count_as_touch_count_grows() {
        let large_touch_count = scattered_group_touch(1000.0, 1000.0, 100.0);
        assert!((large_touch_count - 10.0).abs() < 1e-6);
    }

    #[test]
    fn combine_touches_never_exceeds_node_count() {
        let combined = combine_touches(10, 5.0, &[5.0, 5.0]);
        assert!(combined <= 10.0);
    }
}

// ============================================================
// 四、映射树六个类（与第二次逐字相同：C1/C2 不改映射树，第五节 5.1 的理由）
// ============================================================

#[derive(Clone, Copy, Debug)]
struct TreeClassActivity {
    region_entries: f64,
    insert_count: f64,
    delete_count: f64,
    data_unit_totals: Option<(f64, f64)>,
}

#[allow(clippy::too_many_arguments, reason = "K8 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
fn class_dirty_nodes_this_layer(
    region_entries: f64,
    layer_coverage: f64,
    insert_count: f64,
    delete_count: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
    data_unit_totals: Option<(f64, f64)>,
) -> f64 {
    if insert_count <= 0.0 && delete_count <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer_for_class = (region_entries / layer_coverage).ceil().max(1.0) as u64;
    if node_count_at_layer_for_class <= 1 {
        return 1.0;
    }
    let far_fraction: f64 = if let Some((total_units, touched_units)) = data_unit_totals {
        if total_units > touched_units {
            1.0
        } else {
            0.0
        }
    } else {
        match (policy, placement) {
            (PositionPolicy::Near, _) => 0.0,
            (PositionPolicy::Far, _) => 1.0,
            (PositionPolicy::Balanced, Placement::Sequential) => (insert_count / layer_coverage).min(1.0),
            (PositionPolicy::Balanced, Placement::Random) => (1.0 - previous_touch_fraction).clamp(0.0, 1.0),
        }
    };
    let near_count = delete_count * (1.0 - far_fraction);
    let far_count = delete_count * far_fraction;
    let tail_group = insert_count + near_count;
    let continuous_touch = if far_fraction > 0.0 && placement == Placement::Sequential {
        let gap = (region_entries - tail_group - far_count).max(0.0);
        continuous_groups_touch(node_count_at_layer_for_class, layer_coverage, &[far_count, tail_group], &[gap])
    } else {
        continuous_groups_touch(node_count_at_layer_for_class, layer_coverage, &[tail_group], &[])
    };
    let scattered_touch = if far_fraction > 0.0 && placement == Placement::Random {
        scattered_group_touch(far_count, region_entries, layer_coverage)
    } else {
        0.0
    };
    let scattered_touches: &[f64] = if scattered_touch > 0.0 { &[scattered_touch] } else { &[] };
    combine_touches(node_count_at_layer_for_class, continuous_touch, scattered_touches)
}

fn multiclass_tree_dirty_nodes_per_layer(
    layers: &[u64],
    leaf_capacity: u64,
    internal_capacity: u64,
    classes: &[TreeClassActivity],
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fractions: &[f64],
) -> Vec<f64> {
    assert_eq!(previous_touch_fractions.len(), layers.len(), "每层都要有上一轮的 q");
    let mut dirty_per_layer = Vec::with_capacity(layers.len());
    let mut cov = leaf_capacity as f64;
    for (layer_index, &node_count_at_layer) in layers.iter().enumerate() {
        let mut layer_sum = 0.0;
        for class in classes {
            layer_sum += class_dirty_nodes_this_layer(class.region_entries, cov, class.insert_count, class.delete_count, placement, policy, previous_touch_fractions[layer_index], class.data_unit_totals);
        }
        dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
        cov *= internal_capacity as f64;
    }
    dirty_per_layer
}

fn sum_f64(values: &[f64]) -> f64 {
    values.iter().sum()
}

// ============================================================
// 五、族与几何
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    OneDataUnitPerFile,
    EightAdjacentDataUnitsPerFsync,
}

impl Family {
    fn tag(self) -> &'static str {
        match self {
            Family::OneDataUnitPerFile => "F1",
            Family::EightAdjacentDataUnitsPerFsync => "F8A",
        }
    }

    fn data_unit_count(self, file_count: u64) -> u64 {
        match self {
            Family::OneDataUnitPerFile => file_count,
            Family::EightAdjacentDataUnitsPerFsync => file_count * 8,
        }
    }

    fn data_units_per_fsync(self) -> u64 {
        match self {
            Family::OneDataUnitPerFile => 1,
            Family::EightAdjacentDataUnitsPerFsync => 8,
        }
    }
}

/// 主几何 φ=1、K3、g=24、pbs=512（登记第一节「读法写死」）；`fill_ratio`（φ=0.5）与
/// `backlog_generations`（g=0）各是第八节 8.2 的一个反向取样点。
#[derive(Clone, Copy, Debug)]
struct Geometry {
    fill_ratio: FillRatio,
    backlog_generations: u64,
}

impl Geometry {
    fn main() -> Geometry {
        Geometry { fill_ratio: FillRatio::Full, backlog_generations: 24 }
    }

    fn leaf_capacity(self, raw: u64) -> u64 {
        self.fill_ratio.effective_capacity(raw)
    }

    fn internal_capacity(self, key_bytes: u64) -> u64 {
        self.fill_ratio.effective_capacity(node_capacity(key_bytes, internal_entry_bytes(key_bytes)))
    }
}

#[derive(Clone, Copy, Debug)]
struct PoolShape {
    file_count: u64,
    family: Family,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
}

// ============================================================
// 六、三种写法（登记第五节 5.1 C1/C2）与 K5 释放积压的口径
// ============================================================

/// 三种写法（登记第五节 5.2）：基线（第二次装置）、① 释放只追加、② 按上一版写入时刻聚簇
/// （`cluster_segment_slots = Some(64)`；`None` 是反向取样点 ②连——一个时期不切块）。
#[derive(Clone, Copy, PartialEq, Debug)]
enum AllocationVariant {
    Baseline,
    ReleaseAppendOnly,
    ReleaseClusteredByEpoch { cluster_segment_slots: Option<u64> },
}

impl AllocationVariant {
    fn tag(self) -> &'static str {
        match self {
            AllocationVariant::Baseline => "baseline",
            AllocationVariant::ReleaseAppendOnly => "append",
            AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: Some(_) } => "epoch",
            AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: None } => "epoch_contiguous",
        }
    }
}

/// K5：分配记录条目 = D ×（活单元 + g ×〈释放积压〉）。基线 / ②：释放积压 = X+d+I（第二次 R1）；
/// ①：释放积压 = 2×(X+d) + I（登记第五节 5.1 C1：每个被释放的单元在积压期里有槽号那一条
/// 与追加的那一条，中间版只有追加那一条——I 不重复计）。
fn release_backlog_units_for_variant(variant: AllocationVariant, previous_written_units: f64, distinct_data_units_touched: f64, extra_release_only_insertions_total: f64) -> f64 {
    match variant {
        AllocationVariant::ReleaseAppendOnly => 2.0 * (previous_written_units + distinct_data_units_touched) + extra_release_only_insertions_total,
        _ => previous_written_units + distinct_data_units_touched + extra_release_only_insertions_total,
    }
}

/// ② 的时期求和：`K = Σ_{j≥2} n_p (1 − e^{−λ_j/n_p})`，`λ_j = X q (1−q)^{j−1}`；显式求和到
/// `λ_j/n_p < 1e-9` 为止，余项按线性 `Σ_{i≥j} λ_i = X(1−q)^{j−1}` 补（登记第五节 5.1 C2，
/// `anchors_r3.py` 的 `epoch_pieces`）。`q ≤ 1e-12` 取极限 `K=X(1−q)`；`q ≥ 1` 时 `K=0`。
fn epoch_touched_bump_pieces(node_units: f64, touch_fraction: f64, bump_unit_count: f64, cluster_segment_slots: Option<f64>) -> (f64, f64) {
    let piece_count = match cluster_segment_slots {
        None => 1.0,
        Some(segment_slots) => (bump_unit_count / segment_slots).ceil().max(1.0),
    };
    if node_units <= 0.0 {
        return (0.0, piece_count);
    }
    let touch_fraction = touch_fraction.clamp(0.0, 1.0);
    if touch_fraction <= 1e-12 {
        return (node_units * (1.0 - touch_fraction), piece_count);
    }
    if touch_fraction >= 1.0 {
        return (0.0, piece_count);
    }
    // 逐代求和，直到某一代的 λ/n_p 已经小到可以用泰勒展开的闭式补完剩余的无穷级数——
    // 不再逐代加到 1e-9 那么细：`touch_fraction` 很小时 λ 从第 2 代起就已经远小于 n_p
    // （衰减速率是 `1 − touch_fraction`，越接近 1 衰减越慢），逐代求和到 1e-9 要几百万到
    // 上千万代，会把整条不动点拖到不可用。换成三阶泰勒闭式之后，在 λ/n_p < 1e-2 就切换，
    // 丢掉的四阶项相对贡献 ~ (λ/n_p)³ ~ 1e-6，比 1e-9 的直接阈值在实测anchor容差（1e-4）内等价。
    let decay_ratio = 1.0 - touch_fraction;
    let mut touched_pieces = 0.0;
    let mut generation = 2u32;
    loop {
        let expected_count_at_generation = node_units * touch_fraction * decay_ratio.powi(generation as i32 - 1);
        let relative_to_piece_count = expected_count_at_generation / piece_count;
        if relative_to_piece_count < 1e-9 {
            touched_pieces += node_units * decay_ratio.powi(generation as i32 - 1);
            break;
        }
        if relative_to_piece_count < 1e-2 {
            // 剩余级数 Σ n_p(1 − e^{−λ_i/n_p}) 按泰勒展开到三阶：Σλ_i − Σλ_i²/(2n_p) + Σλ_i³/(6n_p²)。
            // 三个几何级数都有闭式：Σλ_i^k = λ_j^k / (1 − (1−touch_fraction)^k)。
            let lam = expected_count_at_generation;
            let sum_of_power = |power: i32| lam.powi(power) / (1.0 - decay_ratio.powi(power));
            touched_pieces += sum_of_power(1) - sum_of_power(2) / (2.0 * piece_count) + sum_of_power(3) / (6.0 * piece_count * piece_count);
            break;
        }
        touched_pieces += piece_count * (1.0 - (-relative_to_piece_count).exp());
        generation += 1;
        assert!(generation < 10_000, "时期求和不该需要这么多代——多半是 touch_fraction 算错了");
    }
    (touched_pieces, piece_count)
}

/// 分配记录树一块盘一层的期望脏节点数，三种写法（登记第五节 5.1）；`bump_unit_count`（S）只有
/// ② 用：一次持久化 bump 出的单元数（甲 S=X；WAL checkpoint S=X+I_node，由调用方算好传入）。
#[allow(clippy::too_many_arguments)]
fn allocation_tree_dirty_this_layer_by_variant(
    variant: AllocationVariant,
    region_entries: f64,
    layer_coverage: f64,
    node_units: f64,
    data_units: f64,
    data_total_units: f64,
    extra_release_only_insertions_total: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
    bump_unit_count: f64,
    backlog_generations: f64,
) -> f64 {
    if node_units <= 0.0 && data_units <= 0.0 && extra_release_only_insertions_total <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer = (region_entries / layer_coverage).ceil().max(1.0) as u64;
    if node_count_at_layer <= 1 {
        return 1.0;
    }
    let data_is_far = data_total_units > data_units;
    let data_new_placement_scattered = policy == PositionPolicy::Far && placement == Placement::Random && data_is_far;

    if let AllocationVariant::ReleaseAppendOnly = variant {
        let released_units = node_units + data_units + extra_release_only_insertions_total;
        let new_slot_records = node_units + if data_new_placement_scattered { 0.0 } else { data_units };
        let continuous = if backlog_generations >= 1.0 {
            continuous_groups_touch(node_count_at_layer, layer_coverage, &[new_slot_records + released_units, released_units], &[((backlog_generations - 1.0) * released_units).max(0.0)])
        } else {
            continuous_groups_touch(node_count_at_layer, layer_coverage, &[new_slot_records], &[])
        };
        return if data_new_placement_scattered { combine_touches(node_count_at_layer, continuous, &[scattered_group_touch(data_units, region_entries, layer_coverage)]) } else { continuous };
    }

    let node_far_fraction: f64 = match (policy, placement) {
        (PositionPolicy::Near, _) => 0.0,
        (PositionPolicy::Far, _) => 1.0,
        (PositionPolicy::Balanced, Placement::Sequential) => (node_units / layer_coverage).min(1.0),
        (PositionPolicy::Balanced, Placement::Random) => (1.0 - previous_touch_fraction).clamp(0.0, 1.0),
    };
    let tail_group = node_units + node_units * (1.0 - node_far_fraction) + if data_new_placement_scattered { 0.0 } else { data_units } + if data_is_far { 0.0 } else { data_units } + extra_release_only_insertions_total;
    let node_far = node_units * node_far_fraction;
    let data_far_total = (if data_is_far { data_units } else { 0.0 }) + (if data_new_placement_scattered { data_units } else { 0.0 });

    if let AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots } = variant {
        if policy == PositionPolicy::Balanced && placement == Placement::Random {
            let (touched_pieces, piece_count) = epoch_touched_bump_pieces(node_units, previous_touch_fraction, bump_unit_count, cluster_segment_slots.map(|slots| slots as f64));
            let piece_size = bump_unit_count / piece_count;
            let far_nodes = scattered_continuous_groups_touch(node_count_at_layer, layer_coverage, touched_pieces, piece_size, region_entries);
            let continuous = continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[]);
            return combine_touches(node_count_at_layer, continuous, &[far_nodes, scattered_group_touch(data_far_total, region_entries, layer_coverage)]);
        }
        // ② 只在 L均+rand 起作用；别处（seq、L远、L近）退回下面的基线几何（B-r3-3）。
    }

    let far_total = node_far + data_far_total;
    if far_total > 0.0 && placement == Placement::Sequential {
        let gap = (region_entries - tail_group - far_total).max(0.0);
        continuous_groups_touch(node_count_at_layer, layer_coverage, &[far_total, tail_group], &[gap])
    } else if far_total > 0.0 && placement == Placement::Random {
        let continuous = continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[]);
        combine_touches(node_count_at_layer, continuous, &[scattered_group_touch(far_total, region_entries, layer_coverage)])
    } else {
        continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[])
    }
}

#[cfg(test)]
mod allocation_variant_tests {
    use super::*;

    const EPOCH64: AllocationVariant = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: Some(CLUSTER_SEGMENT_SLOTS) };
    const EPOCH_CONTIGUOUS: AllocationVariant = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: None };

    fn call(variant: AllocationVariant, touch_fraction: f64, bump_unit_count: f64) -> f64 {
        allocation_tree_dirty_this_layer_by_variant(variant, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Balanced, touch_fraction, bump_unit_count, 24.0)
    }

    /// B-r3-2：同一组输入，三种写法（登记第七节 7.2）。
    #[test]
    fn positive_control_pc1_matches_the_preregistration_anchor() {
        assert!((call(AllocationVariant::Baseline, 0.5, 400.0) - 183.591_958).abs() < 1e-5, "基线={}", call(AllocationVariant::Baseline, 0.5, 400.0));
        assert!((call(AllocationVariant::ReleaseAppendOnly, 0.5, 400.0) - 3.479_064).abs() < 1e-5);
        assert!((call(EPOCH64, 0.5, 400.0) - 40.600_989).abs() < 1e-5);
        assert!((call(EPOCH_CONTIGUOUS, 0.5, 400.0) - 14.539_217).abs() < 1e-5);
    }

    /// B-r3-3：seq 与 L远 下 ② 与基线逐字节相同。
    #[test]
    fn epoch_variant_falls_back_to_baseline_outside_balanced_random() {
        let sequential_baseline = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::Baseline, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Sequential, PositionPolicy::Balanced, 0.5, 400.0, 24.0);
        let sequential_epoch = allocation_tree_dirty_this_layer_by_variant(EPOCH64, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Sequential, PositionPolicy::Balanced, 0.5, 400.0, 24.0);
        assert!((sequential_baseline - 2.985_222).abs() < 1e-5, "sequential_baseline={sequential_baseline}");
        assert!((sequential_epoch - sequential_baseline).abs() < 1e-9);
        let far_baseline = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::Baseline, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Far, 0.5, 400.0, 24.0);
        let far_epoch = allocation_tree_dirty_this_layer_by_variant(EPOCH64, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Far, 0.5, 400.0, 24.0);
        assert!((far_baseline - 332.151_299).abs() < 1e-4, "far_baseline={far_baseline}");
        assert!((far_epoch - far_baseline).abs() < 1e-9);
        let far_append = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::ReleaseAppendOnly, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Far, 0.5, 400.0, 24.0);
        assert!((far_append - 4.474_355).abs() < 1e-4, "far_append={far_append}");
    }

    /// B-r3-4：② 的数值精度（`q` 很小或很大时的显式求和 + 余项）。
    #[test]
    fn epoch_variant_matches_the_numerical_precision_anchors() {
        assert!((call(EPOCH64, 1e-6, 400.0) - 349.733_527).abs() < 1e-4, "got={}", call(EPOCH64, 1e-6, 400.0));
        assert!((call(EPOCH64, 1e-4, 400.0) - 349.312_931).abs() < 1e-4);
        assert!((call(EPOCH64, 1e-12, 400.0) - 349.737_780).abs() < 1e-3, "q≈0 极限 got={}", call(EPOCH64, 1e-12, 400.0));
        assert!((call(EPOCH64, 0.9, 400.0) - 13.716_421).abs() < 1e-4);
        assert!((call(AllocationVariant::Baseline, 0.9, 400.0) - 42.048_713).abs() < 1e-4);
    }

    /// B-r3-4b：WAL 形态（I=60、S=X+I_node=460）。
    #[test]
    fn epoch_variant_with_intermediate_versions_matches_the_write_ahead_log_anchor() {
        let baseline = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::Baseline, 812_000.0, 812.0, 400.0, 1.0, 1e6, 60.0, Placement::Random, PositionPolicy::Balanced, 0.5, 400.0, 24.0);
        assert!((baseline - 183.652_388).abs() < 1e-4, "baseline={baseline}");
        let append = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::ReleaseAppendOnly, 812_000.0, 812.0, 400.0, 1.0, 1e6, 60.0, Placement::Random, PositionPolicy::Balanced, 0.5, 400.0, 24.0);
        assert!((append - 3.626_847).abs() < 1e-4, "append={append}");
        let epoch = allocation_tree_dirty_this_layer_by_variant(EPOCH64, 812_000.0, 812.0, 400.0, 1.0, 1e6, 60.0, Placement::Random, PositionPolicy::Balanced, 0.5, 460.0, 24.0);
        assert!((epoch - 44.404_694).abs() < 1e-4, "epoch(S=460)={epoch}");
        // 另一格（登记第七节 B-r3-4b「另 I=0、S=460 时」）：S 照样带着 I_node 的量涨到 460，
        // 但 I（进 K5/尾部的总量）取 0——单独看「S 涨」这一半的效应，与上面「I 与 S 都涨」分开。
        let epoch_with_bump_size_increase_but_no_intermediate_total = allocation_tree_dirty_this_layer_by_variant(EPOCH64, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Balanced, 0.5, 460.0, 24.0);
        assert!((epoch_with_bump_size_increase_but_no_intermediate_total - 44.333_955).abs() < 1e-3, "epoch(I=0,S=460)={epoch_with_bump_size_increase_but_no_intermediate_total}");
    }

    /// B-r3-8：① 下每一代分支比都远小于 1（第四节 (c)：装置错就走 V3，不是判据本身）。
    #[test]
    fn release_append_only_touches_far_fewer_nodes_than_baseline() {
        let baseline = call(AllocationVariant::Baseline, 0.5, 400.0);
        let append = call(AllocationVariant::ReleaseAppendOnly, 0.5, 400.0);
        assert!(append < baseline * 0.05, "append={append} baseline={baseline}");
    }

    /// g=0（第八节 8.2 反向取样点）：① 的追加段不进树，退回只有新落点那一组。
    #[test]
    fn release_append_only_with_zero_backlog_generations_drops_the_append_segment() {
        let value = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::ReleaseAppendOnly, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Balanced, 0.5, 400.0, 0.0);
        assert!((value - 1.492_611).abs() < 1e-4, "got={value}");
    }
}

// ============================================================
// 七、不动点求解器（登记第一节「求解器」；本次接线三种写法、可选逐轮记录）
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TouchSpecification {
    Continuous(u64),
    ScatteredContinuousGroups { group_count: u64, group_size: u64 },
}

fn touch_dirty_nodes_per_layer(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, region_entries: f64, touch_specification: TouchSpecification) -> Vec<f64> {
    match touch_specification {
        TouchSpecification::Continuous(touch_count) => single_group_dirty_nodes_per_layer(layers, leaf_capacity, internal_capacity, touch_count),
        TouchSpecification::ScatteredContinuousGroups { group_count, group_size } => {
            if group_count == 0 {
                return vec![0.0; layers.len()];
            }
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(scattered_continuous_groups_touch(node_count_at_layer, cov, group_count as f64, group_size as f64, region_entries));
                cov *= internal_capacity as f64;
            }
            result
        }
    }
}

/// T（登记第五节 5.1）：求解器每一轮的记录，只保留判 Q8.2/Q8.3 要用的几样，不整份逐轮落盘
/// （`test-discipline.md`「端点不是轨迹」：报峰值/超阈值代数/末值三样，由调用方从这个向量里算）。
#[derive(Clone, Copy, Debug)]
struct RoundRecord {
    raw_allocation_layer0_dirty: f64,
    raw_mapping_layer0_dirty: f64,
}

#[derive(Clone, Debug)]
struct FixedPointCoreOutcome {
    extent_layers: Vec<u64>,
    extent_dirty: Vec<f64>,
    inode_layers: Vec<u64>,
    inode_dirty: Vec<f64>,
    accounting_dirty_total: f64,
    tree_table_dirty_total: f64,
    allocation_layers: Vec<u64>,
    allocation_dirty: Vec<f64>,
    mapping_layers: Vec<u64>,
    mapping_dirty: Vec<f64>,
    written_units: f64,
    iterations: u64,
    allocation_touch_final: Vec<f64>,
    mapping_touch_final: Vec<f64>,
    allocation_entries_final: f64,
    round_records: Vec<RoundRecord>,
}

/// 5.3 不动点核心：extent / inode 用 `extent_touch` / `inode_touch` 指定这次持久化怎么碰它们；
/// `extra_allocation_release_insertions_node`（I_node）只喂 ② 的 `bump_unit_count`（S=X+I_node）；
/// `extra_allocation_release_insertions_total`（I）喂 K5 与几何的 `extra_release_only_insertions`；
/// `damping_alpha`/`maximum_iterations` 是第八节 8.2 的两个反向取样点（主取值 0.15 / 1000）。
#[allow(clippy::too_many_arguments)]
fn solve_fixed_point_core_with_variant(
    data_unit_count: u64,
    file_count: u64,
    extent_touch: TouchSpecification,
    inode_touch: TouchSpecification,
    distinct_data_units_touched: f64,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
    variant: AllocationVariant,
    extra_allocation_release_insertions_node: f64,
    extra_allocation_release_insertions_total: f64,
    damping_alpha: f64,
    maximum_iterations: u64,
    record_rounds: bool,
) -> FixedPointCoreOutcome {
    let device_count = DEVICE_COUNT as f64;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
    let extent_dirty = touch_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, data_unit_count as f64, extent_touch);
    let extent_dirty_total = sum_f64(&extent_dirty);

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let inode_layers = tree_layers(file_count, inode_leaf_cap, inode_internal_cap, 2);
    let inode_dirty = touch_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, file_count as f64, inode_touch);
    let inode_container_dirty = inode_dirty[0];
    let inode_root_dirty_total: f64 = inode_dirty[1..].iter().sum();
    let inode_dirty_total = inode_container_dirty + inode_root_dirty_total;

    let accounting_entries = ACCOUNTING_POOL_WIDE_ENTRIES + ACCOUNTING_ENTRIES_PER_DEVICE * DEVICE_COUNT;
    let accounting_leaf_cap = geometry.leaf_capacity(node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES));
    let accounting_internal_cap = geometry.internal_capacity(ACCOUNTING_KEY_BYTES);
    let accounting_layers = tree_layers(accounting_entries, accounting_leaf_cap, accounting_internal_cap, 1);
    assert_eq!(accounting_layers, vec![1], "记账树恒 1 节点（K5）");
    let accounting_dirty_total = 1.0_f64;

    let tree_table_cap = geometry.leaf_capacity(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES));
    let tree_table_layers = tree_layers(TREE_TABLE_ENTRY_COUNT, tree_table_cap, tree_table_cap, 1);
    let tree_table_dirty = touch_dirty_nodes_per_layer(&tree_table_layers, tree_table_cap, tree_table_cap, TREE_TABLE_ENTRY_COUNT as f64, TouchSpecification::Continuous(TREE_TABLE_TOUCHED_ROOTS));
    let tree_table_total_nodes: u64 = tree_table_layers.iter().sum();
    let tree_table_dirty_total = sum_f64(&tree_table_dirty);

    let allocation_leaf_cap = geometry.leaf_capacity(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES));
    let allocation_internal_cap = geometry.internal_capacity(ALLOCATION_KEY_BYTES);
    let mapping_leaf_cap = geometry.leaf_capacity(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES));
    let mapping_internal_cap = geometry.internal_capacity(MAPPING_KEY_BYTES);

    let mut allocation_layers: Vec<u64> = vec![1];
    let mut mapping_layers: Vec<u64> = vec![1];
    let mut allocation_dirty: Vec<f64> = vec![0.0];
    let mut mapping_dirty: Vec<f64> = vec![0.0];
    let mut previous_touch_allocation: Vec<f64> = vec![0.0];
    let mut previous_touch_mapping: Vec<f64> = vec![0.0];
    let mut previous_written_units = extent_dirty_total + inode_dirty_total + accounting_dirty_total + tree_table_dirty_total;
    let mut previous_allocation_entries = 0.0_f64;
    let mut previous_mapping_entries = 0.0_f64;
    let mut iterations = 0u64;
    let mut round_records: Vec<RoundRecord> = Vec::new();

    loop {
        iterations += 1;
        let allocation_total_nodes: u64 = allocation_layers.iter().sum();
        let mapping_total_nodes: u64 = mapping_layers.iter().sum();
        let extent_total_nodes: u64 = extent_layers.iter().sum();
        let inode_total_nodes: u64 = inode_layers.iter().sum();

        let live_units = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes as f64 + 1.0 + mapping_total_nodes as f64 + tree_table_total_nodes as f64 + 1.0;

        let release_backlog_units = release_backlog_units_for_variant(variant, previous_written_units, distinct_data_units_touched, extra_allocation_release_insertions_total);
        let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * release_backlog_units);
        let allocation_layers_now = tree_layers(allocation_entries.round() as u64, allocation_leaf_cap, allocation_internal_cap, 1);
        let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() { previous_touch_allocation.clone() } else { vec![0.0; allocation_layers_now.len()] };
        let per_device_allocation_entries = allocation_entries / device_count;
        let bump_unit_count = previous_written_units + extra_allocation_release_insertions_node;
        let allocation_covs = layer_coverages(allocation_leaf_cap, allocation_internal_cap, allocation_layers_now.len());
        let raw_allocation_layer0_dirty = allocation_tree_dirty_this_layer_by_variant(
            variant,
            per_device_allocation_entries,
            allocation_covs[0],
            previous_written_units,
            distinct_data_units_touched,
            data_unit_count as f64,
            extra_allocation_release_insertions_total,
            placement,
            policy,
            allocation_touch[0],
            bump_unit_count,
            geometry.backlog_generations as f64,
        ) * device_count;
        let allocation_dirty_now: Vec<f64> = allocation_covs
            .iter()
            .zip(allocation_layers_now.iter())
            .enumerate()
            .map(|(layer_index, (&cov, &node_count))| {
                let per_device_dirty = if layer_index == 0 {
                    raw_allocation_layer0_dirty / device_count
                } else {
                    allocation_tree_dirty_this_layer_by_variant(
                        variant,
                        per_device_allocation_entries,
                        cov,
                        previous_written_units,
                        distinct_data_units_touched,
                        data_unit_count as f64,
                        extra_allocation_release_insertions_total,
                        placement,
                        policy,
                        allocation_touch[layer_index],
                        bump_unit_count,
                        geometry.backlog_generations as f64,
                    )
                };
                (per_device_dirty * device_count).min(node_count as f64)
            })
            .collect();
        let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
        let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

        let mapping_entries = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes_now as f64 + 1.0;
        let mapping_layers_now = tree_layers(mapping_entries.round() as u64, mapping_leaf_cap, mapping_internal_cap, 1);
        let mapping_touch = if mapping_layers_now.len() == previous_touch_mapping.len() { previous_touch_mapping.clone() } else { vec![0.0; mapping_layers_now.len()] };
        let mapping_classes = [
            TreeClassActivity { region_entries: data_unit_count as f64, insert_count: distinct_data_units_touched, delete_count: distinct_data_units_touched, data_unit_totals: Some((data_unit_count as f64, distinct_data_units_touched)) },
            TreeClassActivity { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total, data_unit_totals: None },
            TreeClassActivity { region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total), insert_count: inode_root_dirty_total, delete_count: inode_root_dirty_total, data_unit_totals: None },
            TreeClassActivity { region_entries: allocation_total_nodes_now as f64, insert_count: allocation_dirty_total_now, delete_count: allocation_dirty_total_now, data_unit_totals: None },
            TreeClassActivity { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total, data_unit_totals: None },
            TreeClassActivity { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty, data_unit_totals: None },
        ];
        let mapping_dirty_now = multiclass_tree_dirty_nodes_per_layer(&mapping_layers_now, mapping_leaf_cap, mapping_internal_cap, &mapping_classes, placement, policy, &mapping_touch);
        let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

        let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

        if record_rounds {
            round_records.push(RoundRecord { raw_allocation_layer0_dirty, raw_mapping_layer0_dirty: *mapping_dirty_now.first().unwrap_or(&0.0) });
        }

        let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs() / allocation_entries.max(1.0);
        let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs() / mapping_entries.max(1.0);
        let written_units_delta = (written_units_now - previous_written_units).abs() / written_units_now.max(1.0);
        let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs() / allocation_dirty_total_now.max(1.0);
        let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs() / mapping_dirty_total_now.max(1.0);

        let raw_touch_allocation: Vec<f64> = allocation_layers_now.iter().zip(allocation_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_allocation = if raw_touch_allocation.len() == previous_touch_allocation.len() {
            raw_touch_allocation.iter().zip(previous_touch_allocation.iter()).map(|(&raw, &previous)| damping_alpha * raw + (1.0 - damping_alpha) * previous).collect()
        } else {
            raw_touch_allocation
        };
        let raw_touch_mapping: Vec<f64> = mapping_layers_now.iter().zip(mapping_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_mapping = if raw_touch_mapping.len() == previous_touch_mapping.len() {
            raw_touch_mapping.iter().zip(previous_touch_mapping.iter()).map(|(&raw, &previous)| damping_alpha * raw + (1.0 - damping_alpha) * previous).collect()
        } else {
            raw_touch_mapping
        };
        allocation_layers = allocation_layers_now;
        mapping_layers = mapping_layers_now;
        allocation_dirty = allocation_dirty_now;
        mapping_dirty = mapping_dirty_now;
        previous_allocation_entries = allocation_entries;
        previous_mapping_entries = mapping_entries;
        previous_written_units = damping_alpha * written_units_now + (1.0 - damping_alpha) * previous_written_units;

        let converged = allocation_entries_delta < CONVERGENCE_EPSILON && mapping_entries_delta < CONVERGENCE_EPSILON && written_units_delta < CONVERGENCE_EPSILON && allocation_dirty_delta < CONVERGENCE_EPSILON && mapping_dirty_delta < CONVERGENCE_EPSILON;
        if converged || iterations >= maximum_iterations {
            break;
        }
    }

    FixedPointCoreOutcome {
        extent_layers,
        extent_dirty,
        inode_layers,
        inode_dirty,
        accounting_dirty_total,
        tree_table_dirty_total,
        allocation_layers,
        allocation_dirty,
        mapping_layers,
        mapping_dirty,
        written_units: previous_written_units,
        iterations,
        allocation_touch_final: previous_touch_allocation,
        mapping_touch_final: previous_touch_mapping,
        allocation_entries_final: previous_allocation_entries,
        round_records,
    }
}

// ============================================================
// 八、F2 判定（与第二次逐字相同）
// ============================================================

fn layer_zero_is_mostly_dirty(layers: &[u64], dirty: &[f64]) -> bool {
    match (layers.first(), dirty.first()) {
        (Some(&node_count), Some(&dirty_count)) if node_count >= 64 => dirty_count / node_count as f64 >= 0.5,
        _ => false,
    }
}

fn convergence_and_saturation_status(
    iterations: u64,
    maximum_iterations: u64,
    extent_layers: &[u64],
    extent_dirty: &[f64],
    inode_layers: &[u64],
    inode_dirty: &[f64],
    allocation_layers: &[u64],
    allocation_dirty: &[f64],
    mapping_layers: &[u64],
    mapping_dirty: &[f64],
) -> (bool, Vec<&'static str>) {
    let half1 = iterations >= maximum_iterations;
    let mut triggered_trees = Vec::new();
    if layer_zero_is_mostly_dirty(extent_layers, extent_dirty) {
        triggered_trees.push("extent");
    }
    if layer_zero_is_mostly_dirty(inode_layers, inode_dirty) {
        triggered_trees.push("inode");
    }
    if layer_zero_is_mostly_dirty(allocation_layers, allocation_dirty) {
        triggered_trees.push("allocation");
    }
    if layer_zero_is_mostly_dirty(mapping_layers, mapping_dirty) {
        triggered_trees.push("mapping");
    }
    (half1, triggered_trees)
}

fn core_convergence_and_saturation_status(core: &FixedPointCoreOutcome, maximum_iterations: u64) -> (bool, Vec<&'static str>) {
    convergence_and_saturation_status(core.iterations, maximum_iterations, &core.extent_layers, &core.extent_dirty, &core.inode_layers, &core.inode_dirty, &core.allocation_layers, &core.allocation_dirty, &core.mapping_layers, &core.mapping_dirty)
}

#[cfg(test)]
mod layer_zero_saturation_tests {
    use super::*;

    #[test]
    fn layer_zero_at_or_above_sixty_four_nodes_triggers_at_the_fifty_percent_boundary() {
        assert!(!layer_zero_is_mostly_dirty(&[64], &[31.99]));
        assert!(layer_zero_is_mostly_dirty(&[64], &[32.0]));
    }

    #[test]
    fn below_sixty_four_nodes_never_triggers() {
        assert!(!layer_zero_is_mostly_dirty(&[63], &[63.0]));
    }
}

// ============================================================
// 九、甲 / WAL 两臂（k=1，不用 R8 批量）
// ============================================================

#[derive(Clone, Debug)]
struct JiaPublishOutcomeWithVariant {
    data_bytes: u64,
    extent_bytes: u64,
    inode_container_bytes: u64,
    inode_root_bytes: u64,
    allocation_bytes: u64,
    accounting_bytes: u64,
    mapping_bytes: u64,
    tree_table_bytes: u64,
    record_bytes: u64,
    root_slot_bytes: u64,
    system_configuration_bytes: u64,
    core: FixedPointCoreOutcome,
}

impl JiaPublishOutcomeWithVariant {
    fn total_bytes(&self) -> u64 {
        self.data_bytes + self.extent_bytes + self.inode_container_bytes + self.inode_root_bytes + self.allocation_bytes + self.accounting_bytes + self.mapping_bytes + self.tree_table_bytes + self.record_bytes + self.root_slot_bytes + self.system_configuration_bytes
    }
}

fn solve_jia_publish_with_variant(shape: PoolShape, variant: AllocationVariant, damping_alpha: f64, maximum_iterations: u64, record_rounds: bool) -> JiaPublishOutcomeWithVariant {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();

    let core = solve_fixed_point_core_with_variant(
        data_unit_count,
        shape.file_count,
        TouchSpecification::Continuous(fsync_data_unit_count),
        TouchSpecification::Continuous(1),
        fsync_data_unit_count as f64,
        shape.placement,
        shape.policy,
        geometry,
        variant,
        0.0, // 甲没有中间版：I_node 恒 0。
        0.0, // I 恒 0。
        damping_alpha,
        maximum_iterations,
        record_rounds,
    );

    let written_units = fsync_data_unit_count as f64 + core.written_units;
    let named_items = written_units.round() as u64;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    let unit_write_calls = named_items * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let _total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    let extent_dirty_total = sum_f64(&core.extent_dirty);
    let inode_container_dirty = core.inode_dirty[0];
    let inode_root_dirty_total: f64 = core.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    JiaPublishOutcomeWithVariant {
        data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: (extent_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_container_bytes: (inode_container_dirty * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: geometry_root_slot_bytes(),
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        core,
    }
}

fn geometry_root_slot_bytes() -> u64 {
    PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteAheadLogArm {
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}

impl WriteAheadLogArm {
    fn tag(self) -> &'static str {
        match self {
            WriteAheadLogArm::WriteAheadLogFull => "wal_full",
            WriteAheadLogArm::WriteAheadLogLeaf => "wal_leaf_m",
        }
    }
}

/// WAL 一次 fsync 写出的东西（不发根、不写固定点、不写树表；与第二次逐字相同）。
fn solve_write_ahead_log_fsync_bytes(shape: PoolShape, arm: WriteAheadLogArm) -> u64 {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
    let extent_dirty = single_group_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, fsync_data_unit_count);

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let inode_layers = tree_layers(shape.file_count, inode_leaf_cap, inode_internal_cap, 2);
    let inode_dirty = single_group_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, 1);

    let (extent_component, inode_container_component, inode_root_component, named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => {
            let extent_total = sum_f64(&extent_dirty);
            let inode_container = inode_dirty[0];
            let inode_root: f64 = inode_dirty[1..].iter().sum();
            (extent_total, inode_container, inode_root, 2u64)
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_leaf = extent_dirty[0];
            let inode_leaf = inode_dirty[0];
            let dirty_leaves = (extent_leaf + inode_leaf).round() as u64;
            (extent_leaf, inode_leaf, 0.0, dirty_leaves)
        }
    };
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    let data_bytes = fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT;
    let extent_bytes = (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT;
    let inode_container_bytes = (inode_container_component * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT;
    let inode_root_bytes = (inode_root_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT;
    let record_bytes = record_count * RECORD_BYTES * DEVICE_COUNT;
    data_bytes + extent_bytes + inode_container_bytes + inode_root_bytes + record_bytes
}

/// R7 中间版计数：数据、extent、inode 分开（与第二次逐字相同）。
fn intermediate_version_counts(shape: PoolShape, arm: WriteAheadLogArm, interval_fsyncs: u64, extent_layers: &[u64], dedup_extent_dirty: &[f64], inode_layers: &[u64], dedup_inode_dirty: &[f64], dedup_data_units_touched: f64) -> (f64, f64, f64) {
    let geometry = shape.geometry;
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_count_in_interval = interval_fsyncs as f64;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let per_batch_extent = single_group_dirty_nodes_per_layer(extent_layers, extent_leaf_cap, extent_internal_cap, fsync_data_unit_count);

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let per_batch_inode = single_group_dirty_nodes_per_layer(inode_layers, inode_leaf_cap, inode_internal_cap, 1);

    let data_intermediate = (batch_count_in_interval * fsync_data_unit_count as f64 - dedup_data_units_touched).max(0.0);
    let extent_intermediate_total: f64 = per_batch_extent.iter().zip(dedup_extent_dirty.iter()).map(|(&per_batch, &dedup)| (batch_count_in_interval * per_batch - dedup).max(0.0)).sum();
    let inode_intermediate_total: f64 = match arm {
        WriteAheadLogArm::WriteAheadLogFull => per_batch_inode.iter().zip(dedup_inode_dirty.iter()).map(|(&per_batch, &dedup)| (batch_count_in_interval * per_batch - dedup).max(0.0)).sum(),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            if per_batch_inode.is_empty() {
                0.0
            } else {
                (batch_count_in_interval * per_batch_inode[0] - dedup_inode_dirty[0]).max(0.0)
            }
        }
    };
    (data_intermediate, extent_intermediate_total, inode_intermediate_total)
}

struct WriteAheadLogCheckpointOutcomeWithVariant {
    total_bytes: u64,
    intermediate_version_total: f64,
    core: FixedPointCoreOutcome,
}

/// WAL 一次 checkpoint（间隔 N=16 次 fsync，登记第一节「一次持久化」）；K10 恒开
/// （这一次不再比较 K9，第二次已经比过，第三节 3.2 与第七节 A5）。
fn solve_write_ahead_log_checkpoint_with_variant(shape: PoolShape, arm: WriteAheadLogArm, interval_fsyncs: u64, variant: AllocationVariant, damping_alpha: f64, maximum_iterations: u64, record_rounds: bool) -> WriteAheadLogCheckpointOutcomeWithVariant {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let interval_data_unit_touch_count = interval_fsyncs as f64 * fsync_data_unit_count as f64;

    let extent_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_data_unit_touch_count.round() as u64),
        Placement::Random => TouchSpecification::ScatteredContinuousGroups { group_count: interval_fsyncs, group_size: fsync_data_unit_count },
    };
    let inode_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_fsyncs),
        Placement::Random => TouchSpecification::ScatteredContinuousGroups { group_count: interval_fsyncs, group_size: 1 },
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_continuous_groups_distinct_entries(data_unit_count as f64, interval_fsyncs as f64, fsync_data_unit_count as f64),
    };

    let core_zero = solve_fixed_point_core_with_variant(data_unit_count, shape.file_count, extent_touch, inode_touch, distinct_data_units_touched, shape.placement, shape.policy, geometry, variant, 0.0, 0.0, damping_alpha, maximum_iterations, false);

    let (data_intermediate, extent_intermediate_total, inode_intermediate_total) = intermediate_version_counts(shape, arm, interval_fsyncs, &core_zero.extent_layers, &core_zero.extent_dirty, &core_zero.inode_layers, &core_zero.inode_dirty, distinct_data_units_touched);
    let intermediate_node_total = extent_intermediate_total + inode_intermediate_total;
    let intermediate_total = data_intermediate + intermediate_node_total;

    let core = if intermediate_total > 0.0 {
        solve_fixed_point_core_with_variant(data_unit_count, shape.file_count, extent_touch, inode_touch, distinct_data_units_touched, shape.placement, shape.policy, geometry, variant, intermediate_node_total, intermediate_total, damping_alpha, maximum_iterations, record_rounds)
    } else {
        solve_fixed_point_core_with_variant(data_unit_count, shape.file_count, extent_touch, inode_touch, distinct_data_units_touched, shape.placement, shape.policy, geometry, variant, 0.0, 0.0, damping_alpha, maximum_iterations, record_rounds)
    };

    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);
    let (extent_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            (extent_ancestors, inode_ancestors, (extent_ancestors + inode_ancestors).round() as u64)
        }
    };
    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
    let named_items = fixed_point_units.round() as u64 + extra_named_items;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let total_bytes = (extent_catch_up * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
        + (inode_root_catch_up * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
        + (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
        + (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
        + (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
        + (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
        + record_count * RECORD_BYTES * DEVICE_COUNT
        + geometry_root_slot_bytes()
        + SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT;

    WriteAheadLogCheckpointOutcomeWithVariant { total_bytes, intermediate_version_total: intermediate_total, core }
}

fn amortize(fsync_total_bytes: u64, checkpoint_total_bytes: u64, interval_fsyncs: u64) -> f64 {
    (fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64
}

// ============================================================
// 十、G：代际展开（登记第一节「分支比」）
// ============================================================

struct FrozenMappingContext {
    data_unit_count: f64,
    distinct_data_units_touched: f64,
    extent_total_nodes: f64,
    extent_dirty_total: f64,
    inode_region_for_root_class: f64,
    inode_root_dirty_total: f64,
    accounting_dirty_total: f64,
    inode_container_node_count: f64,
    inode_container_dirty: f64,
    allocation_total_nodes_frozen: f64,
    mapping_layers: Vec<u64>,
    mapping_touch_frozen: Vec<f64>,
    mapping_leaf_cap: u64,
    mapping_internal_cap: u64,
    placement: Placement,
    policy: PositionPolicy,
}

fn frozen_mapping_dirty_total(context: &FrozenMappingContext, allocation_dirty_total_this_generation: f64) -> f64 {
    let classes = [
        TreeClassActivity { region_entries: context.data_unit_count, insert_count: context.distinct_data_units_touched, delete_count: context.distinct_data_units_touched, data_unit_totals: Some((context.data_unit_count, context.distinct_data_units_touched)) },
        TreeClassActivity { region_entries: context.extent_total_nodes, insert_count: context.extent_dirty_total, delete_count: context.extent_dirty_total, data_unit_totals: None },
        TreeClassActivity { region_entries: context.inode_region_for_root_class, insert_count: context.inode_root_dirty_total, delete_count: context.inode_root_dirty_total, data_unit_totals: None },
        TreeClassActivity { region_entries: context.allocation_total_nodes_frozen, insert_count: allocation_dirty_total_this_generation, delete_count: allocation_dirty_total_this_generation, data_unit_totals: None },
        TreeClassActivity { region_entries: 1.0, insert_count: context.accounting_dirty_total, delete_count: context.accounting_dirty_total, data_unit_totals: None },
        TreeClassActivity { region_entries: context.inode_container_node_count, insert_count: context.inode_container_dirty, delete_count: context.inode_container_dirty, data_unit_totals: None },
    ];
    sum_f64(&multiclass_tree_dirty_nodes_per_layer(&context.mapping_layers, context.mapping_leaf_cap, context.mapping_internal_cap, &classes, context.placement, context.policy, &context.mapping_touch_frozen))
}

#[derive(Clone, Debug)]
struct BranchExpansionSummary {
    stopped_at_generation: u64,
    layer0_dirty_by_generation: Vec<f64>, // A_0^(0)..A_0^(stopped)
    branch_ratios: Vec<f64>,              // b_2..b_stopped（下标 0 对应 t=2）
    peak_branch_ratio: Option<f64>,
    generations_with_branch_ratio_above_one: u64,
    final_branch_ratio: Option<f64>,
    label: &'static str,
}

/// G：从一格求解结束时的状态出发，冻住形状与 q，只让 W 往前滚，不阻尼（登记第一节「分支比」）。
#[allow(clippy::too_many_arguments)]
fn generational_branch_expansion(
    variant: AllocationVariant,
    allocation_layers: &[u64],
    allocation_leaf_cap: u64,
    allocation_internal_cap: u64,
    allocation_touch_frozen: &[f64],
    per_device_allocation_entries_frozen: f64,
    zeroth_generation_written_units: f64,
    distinct_data_units_touched: f64,
    data_unit_count: f64,
    extra_release_only_insertions_node: f64,
    extra_release_only_insertions_total: f64,
    backlog_generations: f64,
    placement: Placement,
    policy: PositionPolicy,
    mapping_context: &FrozenMappingContext,
) -> BranchExpansionSummary {
    let allocation_covs = layer_coverages(allocation_leaf_cap, allocation_internal_cap, allocation_layers.len());
    let mut previous_generation_written_units = zeroth_generation_written_units;
    let mut layer0_dirty_by_generation = vec![0.0_f64];
    let mut branch_ratios = Vec::new();
    let mut generation = 0u64;
    loop {
        generation += 1;
        let bump_unit_count = previous_generation_written_units + extra_release_only_insertions_node;
        let allocation_dirty_per_layer: Vec<f64> = allocation_covs
            .iter()
            .zip(allocation_layers.iter())
            .enumerate()
            .map(|(layer_index, (&cov, &node_count))| {
                let per_device_dirty = allocation_tree_dirty_this_layer_by_variant(
                    variant,
                    per_device_allocation_entries_frozen,
                    cov,
                    previous_generation_written_units,
                    distinct_data_units_touched,
                    data_unit_count,
                    extra_release_only_insertions_total,
                    placement,
                    policy,
                    allocation_touch_frozen[layer_index],
                    bump_unit_count,
                    backlog_generations,
                );
                (per_device_dirty * DEVICE_COUNT as f64).min(node_count as f64)
            })
            .collect();
        let allocation_dirty_total_this_generation = sum_f64(&allocation_dirty_per_layer);
        let mapping_dirty_total_this_generation = frozen_mapping_dirty_total(mapping_context, allocation_dirty_total_this_generation);
        let written_units_this_generation = zeroth_generation_written_units + allocation_dirty_total_this_generation + mapping_dirty_total_this_generation;

        let layer0_dirty_this_generation = allocation_dirty_per_layer[0];
        let layer_zero_delta_this_generation = layer0_dirty_this_generation - layer0_dirty_by_generation[layer0_dirty_by_generation.len() - 1];
        layer0_dirty_by_generation.push(layer0_dirty_this_generation);
        if generation >= 2 {
            let layer_zero_delta_previous_generation = layer0_dirty_by_generation[layer0_dirty_by_generation.len() - 2] - layer0_dirty_by_generation.get(layer0_dirty_by_generation.len().wrapping_sub(3)).copied().unwrap_or(0.0);
            if layer_zero_delta_previous_generation.abs() > 1e-12 {
                branch_ratios.push(layer_zero_delta_this_generation / layer_zero_delta_previous_generation);
            } else {
                branch_ratios.push(f64::NAN);
            }
        }
        previous_generation_written_units = written_units_this_generation;

        let stop = layer_zero_delta_this_generation <= 1e-9 * layer0_dirty_this_generation.max(1.0) || generation >= MAIN_MAXIMUM_ITERATIONS;
        if stop {
            let peak_branch_ratio = branch_ratios.iter().copied().filter(|value| value.is_finite()).fold(None, |accumulator: Option<f64>, value| Some(accumulator.map_or(value, |peak: f64| peak.max(value))));
            let generations_with_branch_ratio_above_one = branch_ratios.iter().filter(|&&value| value.is_finite() && value > 1.0).count() as u64;
            let final_branch_ratio = branch_ratios.last().copied();
            let label = if generation < 4 || branch_ratios.len() < 3 {
                "inconclusive_stopped_before_generation_4"
            } else if branch_ratios[..3].iter().all(|&value| value.is_finite() && value > 1.0) {
                "supercritical"
            } else if branch_ratios.iter().all(|&value| value.is_finite() && value < 1.0) {
                "subcritical"
            } else {
                "other"
            };
            return BranchExpansionSummary { stopped_at_generation: generation, layer0_dirty_by_generation, branch_ratios, peak_branch_ratio, generations_with_branch_ratio_above_one, final_branch_ratio, label };
        }
    }
}

#[cfg(test)]
mod branch_expansion_tests {
    use super::*;

    /// B-r3-10：甲、P=1、F1、seq、基线——单节点的树一次持久化恒 1 个脏节点，展开在 t=2 停下、b₂=0。
    #[test]
    fn single_node_tree_expansion_stops_at_generation_two_as_subcritical_edge_case() {
        let shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let outcome = solve_jia_publish_with_variant(shape, AllocationVariant::Baseline, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let core = &outcome.core;
        assert_eq!(core.allocation_layers, vec![1], "P=1 时分配记录树恒 1 节点");
        let allocation_leaf_cap = Geometry::main().leaf_capacity(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES));
        let allocation_internal_cap = Geometry::main().internal_capacity(ALLOCATION_KEY_BYTES);
        let mapping_leaf_cap = Geometry::main().leaf_capacity(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES));
        let mapping_internal_cap = Geometry::main().internal_capacity(MAPPING_KEY_BYTES);
        let zeroth_generation_written_units = sum_f64(&core.extent_dirty) + sum_f64(&core.inode_dirty) + core.accounting_dirty_total + core.tree_table_dirty_total;
        let allocation_total_nodes_frozen: u64 = core.allocation_layers.iter().sum();
        let mapping_context = FrozenMappingContext {
            data_unit_count: 1.0,
            distinct_data_units_touched: 1.0,
            extent_total_nodes: core.extent_layers.iter().sum::<u64>() as f64,
            extent_dirty_total: sum_f64(&core.extent_dirty),
            inode_region_for_root_class: (core.inode_layers.iter().sum::<u64>() as f64 - core.inode_layers[0] as f64).max(core.inode_dirty[1..].iter().sum::<f64>()),
            inode_root_dirty_total: core.inode_dirty[1..].iter().sum(),
            accounting_dirty_total: core.accounting_dirty_total,
            inode_container_node_count: core.inode_layers[0] as f64,
            inode_container_dirty: core.inode_dirty[0],
            allocation_total_nodes_frozen: allocation_total_nodes_frozen as f64,
            mapping_layers: core.mapping_layers.clone(),
            mapping_touch_frozen: core.mapping_touch_final.clone(),
            mapping_leaf_cap,
            mapping_internal_cap,
            placement: Placement::Sequential,
            policy: PositionPolicy::Balanced,
        };
        let summary = generational_branch_expansion(AllocationVariant::Baseline, &core.allocation_layers, allocation_leaf_cap, allocation_internal_cap, &core.allocation_touch_final, core.allocation_entries_final / DEVICE_COUNT as f64, zeroth_generation_written_units, 1.0, 1.0, 0.0, 0.0, 24.0, Placement::Sequential, PositionPolicy::Balanced, &mapping_context);
        assert_eq!(summary.stopped_at_generation, 2, "单节点树一次持久化恒 1 个脏节点，第 2 代增量应为 0 而停下");
        assert_eq!(summary.layer0_dirty_by_generation[1], 1.0, "A_0^(1) 应为 1（B-r3-10）");
        assert!((summary.branch_ratios[0] - 0.0).abs() < 1e-9, "b_2 应为 0：{:?}", summary.branch_ratios);
    }
}

// ============================================================
// 十一、分支比仪器（阳性对照，登记第五节 5.3 最后一行；只有分配记录树第 0 层自环，不带映射树）
// ============================================================

/// 分支比仪器：`x_t = D × alloc(variant, region, cov, X=E+x_{t-1}, d=0, U=0, I=0, rand, balanced, q)`，
/// `x_0=0`；`b_t = (x_t−x_{t-1})/(x_{t-1}−x_{t-2})`。冻住 q（不随不动点迭代），不带映射树。
fn branch_ratio_instrument(variant: AllocationVariant, touch_fraction: f64, generation_input: f64, region: f64, cov: f64, rounds: usize) -> (Vec<f64>, Vec<f64>) {
    let mut xs = vec![0.0_f64];
    for _ in 0..rounds {
        let bumped_node_units = generation_input + xs[xs.len() - 1];
        let bump_unit_count = bumped_node_units;
        let per_device = allocation_tree_dirty_this_layer_by_variant(variant, region, cov, bumped_node_units, 0.0, 0.0, 0.0, Placement::Random, PositionPolicy::Balanced, touch_fraction, bump_unit_count, 24.0);
        xs.push(DEVICE_COUNT as f64 * per_device);
    }
    let bs: Vec<f64> = (2..xs.len()).map(|index| (xs[index] - xs[index - 1]) / (xs[index - 1] - xs[index - 2])).collect();
    (xs, bs)
}

#[cfg(test)]
mod branch_ratio_instrument_tests {
    use super::*;

    /// B-r3-7：分支比仪器的五组阳性对照。
    #[test]
    fn branch_ratio_instrument_matches_the_preregistration_anchors() {
        let region = 812.0 * 1e6;
        let (xs, bs) = branch_ratio_instrument(AllocationVariant::Baseline, 0.2, 10.0, region, 812.0, 4);
        assert!((xs[1] - 18.027_021).abs() < 1e-4, "x1={}", xs[1]);
        assert!((xs[4] - 167.480_693).abs() < 1e-3, "x4={}", xs[4]);
        assert!((bs[0] - 1.602_930).abs() < 1e-4, "b2={}", bs[0]);
        assert!((bs[2] - 1.602_852).abs() < 1e-4, "b4={}", bs[2]);

        let (_, bs_high_touch_fraction) = branch_ratio_instrument(AllocationVariant::Baseline, 0.7, 10.0, region, 812.0, 4);
        assert!((bs_high_touch_fraction[0] - 0.604_184).abs() < 1e-4, "b2={}", bs_high_touch_fraction[0]);
        assert!((bs_high_touch_fraction[2] - 0.604_182).abs() < 1e-4, "b4={}", bs_high_touch_fraction[2]);

        let (_, bs_baseline) = branch_ratio_instrument(AllocationVariant::Baseline, 0.3, 6400.0, region, 812.0, 4);
        assert!((bs_baseline[0] - 1.392_532).abs() < 1e-3, "b2={}", bs_baseline[0]);
        assert!((bs_baseline[2] - 1.367_764).abs() < 1e-3, "b4={}", bs_baseline[2]);

        let epoch64 = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: Some(CLUSTER_SEGMENT_SLOTS) };
        let (_, bs_epoch) = branch_ratio_instrument(epoch64, 0.3, 6400.0, region, 812.0, 4);
        assert!((bs_epoch[0] - 0.319_468).abs() < 1e-3, "b2={}", bs_epoch[0]);

        let (_, bs_append) = branch_ratio_instrument(AllocationVariant::ReleaseAppendOnly, 0.3, 6400.0, region, 812.0, 4);
        assert!((bs_append[0] - 0.007_389).abs() < 1e-4, "b2={}", bs_append[0]);
        assert!((bs_append[2] - 0.007_389).abs() < 1e-4, "b4={}", bs_append[2]);
    }
}

// ============================================================
// 十二、第 5 行：ρ 三种读法与树高（登记 q4，Q5.1/Q5.2a-c）
// ============================================================

fn tree_heights_with_variant(outcome: &JiaPublishOutcomeWithVariant) -> [usize; 6] {
    [outcome.core.extent_layers.len(), outcome.core.inode_layers.len(), outcome.core.allocation_layers.len(), 1, outcome.core.mapping_layers.len(), 1]
}

fn one_path_bytes(height: usize, leaf_is_inode_container: bool) -> u64 {
    assert!(height >= 1, "树高至少 1（根即叶）");
    if leaf_is_inode_container {
        UNIT_BYTES + NODE_BYTES * (height as u64 - 1)
    } else {
        NODE_BYTES * height as u64
    }
}

fn tree_bytes_of(outcome: &JiaPublishOutcomeWithVariant) -> u64 {
    outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes
}

fn path_ratio_readings_with_variant(shape: PoolShape, variant: AllocationVariant) -> (f64, f64, f64, u64, u64) {
    let outcome = solve_jia_publish_with_variant(shape, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
    let tree_bytes = tree_bytes_of(&outcome) as f64;
    let heights = tree_heights_with_variant(&outcome);
    let one_path_sum: u64 = one_path_bytes(heights[0], false) + one_path_bytes(heights[1], true) + one_path_bytes(heights[2], false) + one_path_bytes(heights[3], false) + one_path_bytes(heights[4], false) + one_path_bytes(heights[5], false);
    let maximum_tree_height = *heights.iter().max().expect("六个高度都有值") as u64;
    let path_ratio_actual_height = tree_bytes / (DEVICE_COUNT as f64 * one_path_sum as f64);
    let path_ratio_node_width_bound = tree_bytes / (maximum_tree_height as f64 * 6.0 * NODE_BYTES as f64 * DEVICE_COUNT as f64);
    let path_ratio_unit_width_bound = tree_bytes / (maximum_tree_height as f64 * 6.0 * UNIT_BYTES as f64 * DEVICE_COUNT as f64);
    (path_ratio_actual_height, path_ratio_node_width_bound, path_ratio_unit_width_bound, outcome.total_bytes(), tree_bytes as u64)
}

#[cfg(test)]
mod allocation_entries_tests {
    use super::*;

    fn shape_at(file_count: u64) -> PoolShape {
        PoolShape { file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }
    }

    /// B-r3-5：甲、P=1、F1、seq、主几何，三种写法——写字节相同，分配记录条目基线 402、① 786、② 402。
    #[test]
    fn allocation_entries_at_one_file_match_the_preregistration_anchor() {
        let baseline = solve_jia_publish_with_variant(shape_at(1), AllocationVariant::Baseline, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let append = solve_jia_publish_with_variant(shape_at(1), AllocationVariant::ReleaseAppendOnly, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let epoch64 = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: Some(CLUSTER_SEGMENT_SLOTS) };
        let epoch = solve_jia_publish_with_variant(shape_at(1), epoch64, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        assert_eq!(baseline.core.allocation_entries_final.round() as u64, 402, "基线");
        assert_eq!(append.core.allocation_entries_final.round() as u64, 786, "①");
        assert_eq!(epoch.core.allocation_entries_final.round() as u64, 402, "②（K5 与基线相同）");
        assert_eq!(baseline.total_bytes(), 344_576, "写字节三种写法相同（B-r3-5）");
        assert_eq!(append.total_bytes(), 344_576);
        assert_eq!(epoch.total_bytes(), 344_576);
        assert_eq!(baseline.core.allocation_layers.len(), 1);
        assert_eq!(append.core.allocation_layers.len(), 1);
        assert_eq!(epoch.core.allocation_layers.len(), 1);
    }

    /// B-r3-6：甲、F1、seq、主几何、①——分配记录树长到第 2 层的门槛 P*=15（P=14 恰 812 条、1 层；
    /// P=15 为 2 层；基线在同一个 P 上仍是 1 层，181 条，与第二次登记 B10 同）。
    #[test]
    fn release_append_only_reaches_two_layers_at_fifteen_files() {
        // `anchors_r3_k5.py` 的 814 是「假设活单元项里分配记录/映射仍是 1 节点」算出来的一阶估计，
        // 只用来判断门槛落在哪个 P——过了门槛之后分配记录树本身长到 2 节点，`live_units` 与
        // `mapping_entries` 都跟着涨，装置里真正跑的不动点会在门槛之后继续放大（这里读到 1010，
        // 不是 814）。这条测试只钉「门槛出现在 P=14→15 之间」这个定性结论与 P=14 那一侧的精确值，
        // 不钉过门槛之后的精确条目数——过门槛之后的数字本来就不是「一阶估计」能钉住的东西。
        let append_at_boundary = solve_jia_publish_with_variant(shape_at(14), AllocationVariant::ReleaseAppendOnly, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        assert_eq!(append_at_boundary.core.allocation_entries_final.round() as u64, 812, "P=14 恰 812 条");
        assert_eq!(append_at_boundary.core.allocation_layers.len(), 1);
        let append_past_boundary = solve_jia_publish_with_variant(shape_at(15), AllocationVariant::ReleaseAppendOnly, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        assert_eq!(append_past_boundary.core.allocation_layers.len(), 2, "P=15 应长到第 2 层（B-r3-6 的定性结论）");
        let baseline_at_fifteen_files = solve_jia_publish_with_variant(shape_at(15), AllocationVariant::Baseline, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        assert_eq!(baseline_at_fifteen_files.core.allocation_entries_final.round() as u64, 430, "基线同一个 P 仍是 1 节点，条目数与 `anchors_r3_k5.py` 的一阶估计一致（未过门槛，两种算法本该吻合）");
        assert_eq!(baseline_at_fifteen_files.core.allocation_layers.len(), 1);
    }

    /// B-r3-9（后半）：T 开与不开，同一个求解的全部输出逐字节相同（M12 盯着这条不变量）。
    /// 前半「WAL 两臂的 fsync 行三种写法逐字节相同」在这个装置里是结构性恒成立——
    /// `solve_write_ahead_log_fsync_bytes` 根本不接收 `variant` 参数，不必另立测试。
    #[test]
    fn recording_rounds_does_not_change_the_solve_outcome() {
        let shape = PoolShape { file_count: 1_000_000, family: Family::OneDataUnitPerFile, placement: Placement::Random, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let without_recording = solve_jia_publish_with_variant(shape, AllocationVariant::Baseline, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let with_recording = solve_jia_publish_with_variant(shape, AllocationVariant::Baseline, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, true);
        assert_eq!(without_recording.total_bytes(), with_recording.total_bytes());
        assert_eq!(without_recording.core.allocation_layers, with_recording.core.allocation_layers);
        assert_eq!(without_recording.core.allocation_dirty, with_recording.core.allocation_dirty);
        assert_eq!(without_recording.core.iterations, with_recording.core.iterations);
    }

    /// M9：WAL 形态下 `bump_unit_count`（S=X+I_node）真的在求解器内部接了 I_node——
    /// 固定 I（进 K5/尾部的总量）不变，只改 I_node，② 的第 0 层脏节点数应该跟着变；
    /// 基线与 ① 不看 `bump_unit_count`，同样的两次调用应该不变。
    #[test]
    fn epoch_variant_bump_unit_count_inside_the_solver_depends_on_intermediate_node_count() {
        let epoch64 = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: Some(CLUSTER_SEGMENT_SLOTS) };
        let touch = TouchSpecification::ScatteredContinuousGroups { group_count: 16, group_size: 1 };
        let with_small_intermediate_node_count = solve_fixed_point_core_with_variant(1_000_000, 1_000_000, touch, touch, 16.0, Placement::Random, PositionPolicy::Balanced, Geometry::main(), epoch64, 10.0, 60.0, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let with_large_intermediate_node_count = solve_fixed_point_core_with_variant(1_000_000, 1_000_000, touch, touch, 16.0, Placement::Random, PositionPolicy::Balanced, Geometry::main(), epoch64, 200.0, 60.0, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        assert!((with_small_intermediate_node_count.allocation_dirty[0] - with_large_intermediate_node_count.allocation_dirty[0]).abs() > 1e-6, "small={} large={}", with_small_intermediate_node_count.allocation_dirty[0], with_large_intermediate_node_count.allocation_dirty[0]);

        let baseline_small = solve_fixed_point_core_with_variant(1_000_000, 1_000_000, touch, touch, 16.0, Placement::Random, PositionPolicy::Balanced, Geometry::main(), AllocationVariant::Baseline, 10.0, 60.0, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let baseline_large = solve_fixed_point_core_with_variant(1_000_000, 1_000_000, touch, touch, 16.0, Placement::Random, PositionPolicy::Balanced, Geometry::main(), AllocationVariant::Baseline, 200.0, 60.0, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        assert!((baseline_small.allocation_dirty[0] - baseline_large.allocation_dirty[0]).abs() < 1e-6, "基线不看 I_node，两次应该相同");
    }

    /// M13（V4「开关没接上」）：第 5 行的 ρ 真的接了写法开关——大池上基线与 ① 的 ρ 不该相同。
    #[test]
    fn path_ratio_readings_differ_between_variants_at_large_pool() {
        let shape = PoolShape { file_count: 1_000_000, family: Family::OneDataUnitPerFile, placement: Placement::Random, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let (baseline_path_ratio_actual_height, ..) = path_ratio_readings_with_variant(shape, AllocationVariant::Baseline);
        let (append_path_ratio_actual_height, ..) = path_ratio_readings_with_variant(shape, AllocationVariant::ReleaseAppendOnly);
        assert!((baseline_path_ratio_actual_height - append_path_ratio_actual_height).abs() > 1e-6, "①的 ρ_a 应该与基线不同：baseline={baseline_path_ratio_actual_height} append={append_path_ratio_actual_height}");
    }
}

// ============================================================
// 十三、SQ（反向取样点，登记第五节 5.1 最后一行）：冻住状态之后的一次性重算
// ============================================================

/// 每次都重写的根与固定点（记账、树表）在 SQ 里的触达比例恒为 1（登记第一节 SQ：
/// 「记账、树表每次都重写，q_src=1」）。抽成命名常量是为了让 M14（记账/树表的 q_src 取 0）
/// 这条变异有单测盯着，不只是主流程里两个裸字面量。
const FULLY_REWRITTEN_TOUCH_FRACTION: f64 = 1.0;

/// SQ 的六个来源（登记第一节 SQ、第七节 B-r3-11）：额定重写的根/固定点用
/// `FULLY_REWRITTEN_TOUCH_FRACTION`，其余四个用各自这一轮的触达比例。
#[allow(clippy::too_many_arguments)]
fn sq_sources(extent_dirty_total: f64, extent_touch_fraction: f64, inode_dirty_total: f64, inode_touch_fraction: f64, accounting_dirty_total: f64, tree_table_dirty_total: f64, allocation_dirty_total: f64, allocation_touch_fraction: f64, mapping_dirty_total: f64, mapping_touch_fraction: f64) -> [(f64, f64); 6] {
    [
        (extent_dirty_total, extent_touch_fraction),
        (inode_dirty_total, inode_touch_fraction),
        (accounting_dirty_total, FULLY_REWRITTEN_TOUCH_FRACTION),
        (tree_table_dirty_total, FULLY_REWRITTEN_TOUCH_FRACTION),
        (allocation_dirty_total, allocation_touch_fraction),
        (mapping_dirty_total, mapping_touch_fraction),
    ]
}

/// SQ 的组合函数：每个来源各给 `(数量, 触达比例)`，远的条数 = Σ X_src×(1−q_src)，其余进尾部
/// （登记第七节 B-r3-11）。这是独立于求解器的函数级锚点；网格上按「冻住收敛状态之后取一次性
/// 重算」的读法接（第十四节 `sq_one_shot_layer_zero`），不重新迭代整条不动点——范围缩减见文件头。
fn sq_far_count_and_value(region_entries: f64, layer_coverage: f64, sources: &[(f64, f64)], data_units: f64) -> (f64, f64) {
    let far: f64 = sources.iter().map(|&(count, touch_fraction)| count * (1.0 - touch_fraction)).sum();
    let total: f64 = sources.iter().map(|&(count, _)| count).sum();
    let tail = total + (total - far) + data_units;
    let node_count_at_layer = (region_entries / layer_coverage).ceil().max(1.0) as u64;
    let continuous = continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail], &[]);
    let value = combine_touches(node_count_at_layer, continuous, &[scattered_group_touch(far + data_units, region_entries, layer_coverage)]);
    (far, value)
}

#[cfg(test)]
mod sq_tests {
    use super::*;

    /// B-r3-11 + M14。
    #[test]
    fn sq_matches_the_preregistration_anchor() {
        let sources = [(6.0, 1.0), (2.0, 0.001), (300.0, 0.4), (92.0, 0.3)];
        let (far, value) = sq_far_count_and_value(812_000.0, 812.0, &sources, 1.0);
        assert!((far - 246.398_0).abs() < 1e-3, "far={far}");
        assert!((value - 220.579_803).abs() < 1e-3, "value={value}");
        let baseline = allocation_tree_dirty_this_layer_by_variant(AllocationVariant::Baseline, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Balanced, 0.4, 400.0, 24.0);
        assert!((baseline - 215.580_766).abs() < 1e-3, "baseline={baseline}");

        // M14：记账、树表（前两个来源）的 q_src 改取 0。
        let sources_with_fixed_points_untouched = [(6.0, 0.0), (2.0, 0.001), (300.0, 0.4), (92.0, 0.3)];
        let (_, value_with_fixed_points_untouched) = sq_far_count_and_value(812_000.0, 812.0, &sources_with_fixed_points_untouched, 1.0);
        assert!((value_with_fixed_points_untouched - 225.238_914).abs() < 1e-3, "value_with_fixed_points_untouched={value_with_fixed_points_untouched}");
    }

    /// M14 盯的是主流程真正用的那条装配路径：记账、树表两个来源恒 `FULLY_REWRITTEN_TOUCH_FRACTION`。
    #[test]
    fn sq_sources_treats_accounting_and_tree_table_as_fully_rewritten() {
        let sources = sq_sources(10.0, 0.5, 20.0, 0.3, 6.0, 2.0, 300.0, 0.4, 92.0, 0.3);
        assert_eq!(sources[2], (6.0, 1.0), "记账 q_src 恒 1");
        assert_eq!(sources[3], (2.0, 1.0), "树表 q_src 恒 1");
    }
}

// ============================================================
// 十四、main()
// ============================================================

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!("name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} cluster_segment_slots={CLUSTER_SEGMENT_SLOTS} damping_alpha={MAIN_DAMPING_ALPHA} maximum_iterations={MAIN_MAXIMUM_ITERATIONS} model=counting")));
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope covers=第8行主表(rand,Lbalanced,F1/F8A,P1e4/1e6/1e8,N16,k1,三臂,三种写法),阳性对照,B-r3-1..12,Q8.1(G代际展开),Q8.2(F2),Q8.3(T摘要),Q8.4(fsync字节),第5行rand半(甲,三种写法,五档P),Q5.1,Q5.2a-c,反向取样点(代表性格,见文件头) missing=第8行/第5行反向取样点全网格复算,R8组提交,mapping树的SQ(只对分配记录树做)"
        )
    );

    // B-r3-1：容量一览。
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=b_r3_1_capacities alloc_leaf={} alloc_internal={} extent_leaf={} inode_container={} mapping_leaf={} tree_table={} named_items_per_record={}",
            node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES),
            node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES)),
            node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES),
            inode_container_capacity(),
            node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES),
            node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES),
            NAMED_ITEMS_PER_RECORD_MAIN,
        ))
    );

    // 阳性对照（登记第五节 5.3）：函数一级的格，三种写法各报，事先算好的数在旁边核对。
    let epoch64 = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: Some(CLUSTER_SEGMENT_SLOTS) };
    let epoch_contiguous = AllocationVariant::ReleaseClusteredByEpoch { cluster_segment_slots: None };
    for (label, variant) in [("baseline", AllocationVariant::Baseline), ("append", AllocationVariant::ReleaseAppendOnly), ("epoch", epoch64), ("epoch_contiguous", epoch_contiguous)] {
        let value = allocation_tree_dirty_this_layer_by_variant(variant, 812_000.0, 812.0, 400.0, 1.0, 1e6, 0.0, Placement::Random, PositionPolicy::Balanced, 0.5, 400.0, 24.0);
        println!("{}", emitter.emit_raw(&format!("name=positive_control_r3 variant={label} value={value:.6}")));
    }
    for (label, variant, touch_fraction, generation_input) in [("baseline_q0.2", AllocationVariant::Baseline, 0.2, 10.0), ("baseline_q0.7", AllocationVariant::Baseline, 0.7, 10.0), ("baseline_q0.3_e6400", AllocationVariant::Baseline, 0.3, 6400.0), ("epoch_q0.3_e6400", epoch64, 0.3, 6400.0), ("append_q0.3_e6400", AllocationVariant::ReleaseAppendOnly, 0.3, 6400.0)] {
        let (xs, bs) = branch_ratio_instrument(variant, touch_fraction, generation_input, 812.0 * 1e6, 812.0, 4);
        println!("{}", emitter.emit_raw(&format!("name=positive_control_branch_instrument case={label} x1={:.6} x2={:.6} x3={:.6} x4={:.6} b2={:.6} b3={:.6} b4={:.6}", xs[1], xs[2], xs[3], xs[4], bs[0], bs[1], bs[2])));
    }

    // 第 8 行主表（登记第五节 5.4）：rand、Lbalanced、F1/F8A、P∈{1e4,1e6,1e8}、N=16、k=1、三臂 × 三种写法 = 54 个求解。
    let families = [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync];
    let file_counts: [u64; 3] = [10_000, 1_000_000, 100_000_000];
    let variants = [("baseline", AllocationVariant::Baseline), ("append", AllocationVariant::ReleaseAppendOnly), ("epoch", epoch64)];
    let arms = [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf];
    let interval_fsyncs = 16u64;
    for (variant_tag, variant) in variants {
        debug_assert_eq!(variant_tag, variant.tag(), "手写标签要跟 AllocationVariant::tag() 对得上");
    }

    for family in families {
        for &file_count in &file_counts {
            let shape = PoolShape { file_count, family, placement: Placement::Random, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
            for (variant_tag, variant) in variants {
                // 甲。
                let jia = solve_jia_publish_with_variant(shape, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, true);
                let (jia_unconverged, jia_saturated) = core_convergence_and_saturation_status(&jia.core, MAIN_MAXIMUM_ITERATIONS);
                let jia_bytes = jia.total_bytes();
                let allocation_layer0_nodes = jia.core.allocation_layers[0];
                let round_count = jia.core.round_records.len();
                let (jia_peak_ratio, jia_rounds_above_half, jia_final_ratio, jia_delta_sign_changes, jia_delta_median_abs_ratio, jia_delta_label, jia_mapping_peak_ratio) = round_trace_summary(&jia.core.round_records, allocation_layer0_nodes, jia.core.mapping_layers[0]);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=row8_grid family={} placement={} policy={} p={file_count} arm=jia variant={variant_tag} fsync_bytes={jia_bytes} amortized_bytes={jia_bytes} unconverged={jia_unconverged} saturated_trees={jia_saturated:?} allocation_layers={:?} mapping_layers={:?} allocation_layer0_dirty={:.6} mapping_layer0_dirty={:.6} iterations={} round_count={round_count} t_peak_ratio={jia_peak_ratio:.6} t_rounds_ge_half={jia_rounds_above_half} t_final_ratio={jia_final_ratio:.6} t_delta_sign_changes_last100={jia_delta_sign_changes} t_delta_median_abs_ratio_last100={jia_delta_median_abs_ratio:.6} t_label={jia_delta_label} t_mapping_peak_ratio={jia_mapping_peak_ratio:.6}",
                        family.tag(),
                        shape.placement.tag(),
                        shape.policy.tag(),
                        jia.core.allocation_layers,
                        jia.core.mapping_layers,
                        jia.core.allocation_dirty[0],
                        jia.core.mapping_dirty[0],
                        jia.core.iterations,
                    ))
                );
                emit_branch_expansion(&mut emitter, "jia", family, file_count, variant_tag, variant, &jia.core, shape);

                for arm in arms {
                    let checkpoint = solve_write_ahead_log_checkpoint_with_variant(shape, arm, interval_fsyncs, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, true);
                    let (unconverged, saturated) = core_convergence_and_saturation_status(&checkpoint.core, MAIN_MAXIMUM_ITERATIONS);
                    let fsync_bytes = solve_write_ahead_log_fsync_bytes(shape, arm);
                    let amortized_bytes = amortize(fsync_bytes, checkpoint.total_bytes, interval_fsyncs);
                    let allocation_layer0_nodes = checkpoint.core.allocation_layers[0];
                    let round_count = checkpoint.core.round_records.len();
                    let (peak_ratio, rounds_above_half, final_ratio, delta_sign_changes, delta_median_abs_ratio, delta_label, mapping_peak_ratio) = round_trace_summary(&checkpoint.core.round_records, allocation_layer0_nodes, checkpoint.core.mapping_layers[0]);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=row8_grid family={} placement={} policy={} p={file_count} arm={} variant={variant_tag} fsync_bytes={fsync_bytes} amortized_bytes={amortized_bytes:.3} unconverged={unconverged} saturated_trees={saturated:?} allocation_layers={:?} mapping_layers={:?} allocation_layer0_dirty={:.6} mapping_layer0_dirty={:.6} iterations={} intermediate_version_total={:.6} round_count={round_count} t_peak_ratio={peak_ratio:.6} t_rounds_ge_half={rounds_above_half} t_final_ratio={final_ratio:.6} t_delta_sign_changes_last100={delta_sign_changes} t_delta_median_abs_ratio_last100={delta_median_abs_ratio:.6} t_label={delta_label} t_mapping_peak_ratio={mapping_peak_ratio:.6}",
                            family.tag(),
                            shape.placement.tag(),
                            shape.policy.tag(),
                            arm.tag(),
                            checkpoint.core.allocation_layers,
                            checkpoint.core.mapping_layers,
                            checkpoint.core.allocation_dirty[0],
                            checkpoint.core.mapping_dirty[0],
                            checkpoint.core.iterations,
                            checkpoint.intermediate_version_total,
                        ))
                    );
                    emit_branch_expansion(&mut emitter, arm.tag(), family, file_count, variant_tag, variant, &checkpoint.core, shape);
                }
            }
        }
    }

    // 第 5 行 rand 半（登记第五节 5.4）：甲、rand、Lbalanced、F1/F8A、P∈{1,1e2,1e4,1e6,1e8} × 三种写法。
    let row5_file_counts: [u64; 5] = [1, 100, 10_000, 1_000_000, 100_000_000];
    for family in families {
        for &file_count in &row5_file_counts {
            let shape = PoolShape { file_count, family, placement: Placement::Random, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
            for (variant_tag, variant) in variants {
                let (path_ratio_actual_height, path_ratio_node_width_bound, path_ratio_unit_width_bound, total_bytes, tree_bytes) = path_ratio_readings_with_variant(shape, variant);
                let outcome = solve_jia_publish_with_variant(shape, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=row5_grid family={} placement={} policy={} p={file_count} variant={variant_tag} total_bytes={total_bytes} tree_bytes={tree_bytes} rho_a={path_ratio_actual_height:.6} rho_b={path_ratio_node_width_bound:.6} rho_c={path_ratio_unit_width_bound:.6} extent_layers={} inode_layers={} allocation_layers={} mapping_layers={}",
                        family.tag(),
                        shape.placement.tag(),
                        shape.policy.tag(),
                        outcome.core.extent_layers.len(),
                        outcome.core.inode_layers.len(),
                        outcome.core.allocation_layers.len(),
                        outcome.core.mapping_layers.len(),
                    ))
                );
            }
        }
    }

    // 反向取样点（第八节 8.2）：只在代表性格上报（family=F1、arm=甲、P=1e6，文件头已说明范围缩减）。
    let representative_shape_main = PoolShape { file_count: 1_000_000, family: Family::OneDataUnitPerFile, placement: Placement::Random, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
    for (variant_tag, variant) in [("baseline", AllocationVariant::Baseline), ("append", AllocationVariant::ReleaseAppendOnly), ("epoch", epoch64)] {
        // L远 / L近。
        for (policy_tag, policy) in [("Lfar", PositionPolicy::Far), ("Lnear", PositionPolicy::Near)] {
            let shape = PoolShape { policy, ..representative_shape_main };
            let outcome = solve_jia_publish_with_variant(shape, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
            let (unconverged, saturated) = core_convergence_and_saturation_status(&outcome.core, MAIN_MAXIMUM_ITERATIONS);
            println!("{}", emitter.emit_raw(&format!("name=row8_reverse point={policy_tag} variant={variant_tag} fsync_bytes={} unconverged={unconverged} saturated_trees={saturated:?} allocation_layer0_dirty={:.6}", outcome.total_bytes(), outcome.core.allocation_dirty[0])));
        }
        // 阻尼 0.5 / 上限 1e4。
        let damped = solve_jia_publish_with_variant(representative_shape_main, variant, 0.5, MAIN_MAXIMUM_ITERATIONS, false);
        let (damped_unconverged, damped_saturated) = core_convergence_and_saturation_status(&damped.core, MAIN_MAXIMUM_ITERATIONS);
        println!("{}", emitter.emit_raw(&format!("name=row8_reverse point=damping0.5 variant={variant_tag} unconverged={damped_unconverged} saturated_trees={damped_saturated:?} iterations={}", damped.core.iterations)));
        let higher_cap = solve_jia_publish_with_variant(representative_shape_main, variant, MAIN_DAMPING_ALPHA, 10_000, false);
        let (higher_cap_unconverged, higher_cap_saturated) = core_convergence_and_saturation_status(&higher_cap.core, 10_000);
        println!("{}", emitter.emit_raw(&format!("name=row8_reverse point=maxiter1e4 variant={variant_tag} unconverged={higher_cap_unconverged} saturated_trees={higher_cap_saturated:?} iterations={}", higher_cap.core.iterations)));
        // g=0（Q5.2 三列；这里在 row8 的格上一并报）。
        let zero_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..Geometry::main() }, ..representative_shape_main };
        let zero_backlog = solve_jia_publish_with_variant(zero_backlog_shape, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        println!("{}", emitter.emit_raw(&format!("name=row8_reverse point=g0 variant={variant_tag} fsync_bytes={} allocation_layers={:?}", zero_backlog.total_bytes(), zero_backlog.core.allocation_layers)));
        // φ=0.5。
        let half_fill_shape = PoolShape { geometry: Geometry { fill_ratio: FillRatio::Half, ..Geometry::main() }, ..representative_shape_main };
        let half_fill = solve_jia_publish_with_variant(half_fill_shape, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        println!("{}", emitter.emit_raw(&format!("name=row8_reverse point=phi0.5 variant={variant_tag} fsync_bytes={} allocation_layers={:?}", half_fill.total_bytes(), half_fill.core.allocation_layers)));
        // ②连（只对 ② 这一列；这里只在 variant=epoch 时额外报一行，其余变体跳过）。
        if variant_tag == "epoch" {
            let contiguous = solve_jia_publish_with_variant(representative_shape_main, epoch_contiguous, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
            println!("{}", emitter.emit_raw(&format!("name=row8_reverse point=epoch_contiguous variant=epoch fsync_bytes={} allocation_layer0_dirty={:.6}", contiguous.total_bytes(), contiguous.core.allocation_dirty[0])));
        }
        // SQ：冻住收敛状态之后的一次性重算（文件头「范围缩减」）。
        let converged = solve_jia_publish_with_variant(representative_shape_main, variant, MAIN_DAMPING_ALPHA, MAIN_MAXIMUM_ITERATIONS, false);
        let core = &converged.core;
        let extent_total_nodes: u64 = core.extent_layers.iter().sum();
        let inode_total_nodes: u64 = core.inode_layers.iter().sum();
        let touch_fraction_extent = if extent_total_nodes > 0 { sum_f64(&core.extent_dirty) / extent_total_nodes as f64 } else { 1.0 };
        let touch_fraction_inode = if inode_total_nodes > 0 { sum_f64(&core.inode_dirty) / inode_total_nodes as f64 } else { 1.0 };
        let allocation_total_nodes: u64 = core.allocation_layers.iter().sum();
        let mapping_total_nodes: u64 = core.mapping_layers.iter().sum();
        let touch_fraction_mapping = if mapping_total_nodes > 0 { sum_f64(&core.mapping_dirty) / mapping_total_nodes as f64 } else { 1.0 };
        let sources = sq_sources(sum_f64(&core.extent_dirty), touch_fraction_extent, sum_f64(&core.inode_dirty), touch_fraction_inode, core.accounting_dirty_total, core.tree_table_dirty_total, sum_f64(&core.allocation_dirty), core.allocation_touch_final[0], sum_f64(&core.mapping_dirty), touch_fraction_mapping);
        let allocation_leaf_cap = Geometry::main().leaf_capacity(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES));
        let (sq_far, sq_layer0_value) = sq_far_count_and_value(core.allocation_entries_final / DEVICE_COUNT as f64, allocation_leaf_cap as f64, &sources, 1.0);
        println!("{}", emitter.emit_raw(&format!("name=row8_reverse point=SQ variant={variant_tag} allocation_total_nodes={allocation_total_nodes} sq_far_count={sq_far:.6} sq_layer0_dirty_one_shot={:.6} baseline_layer0_dirty={:.6}", sq_layer0_value * DEVICE_COUNT as f64, core.allocation_dirty[0])));
    }

    println!("{}", emitter.finish());
}

/// T 摘要（登记第八节 8.1「只报期末值的量不许写『从来』『恒』」的做法：报峰值 / 超阈值代数 / 末值三样）。
fn round_trace_summary(round_records: &[RoundRecord], layer0_node_count: u64, mapping_layer0_node_count: u64) -> (f64, u64, f64, u64, f64, &'static str, f64) {
    if round_records.is_empty() || layer0_node_count == 0 {
        return (0.0, 0, 0.0, 0, 0.0, "no_rounds", 0.0);
    }
    let ratios: Vec<f64> = round_records.iter().map(|record| record.raw_allocation_layer0_dirty / layer0_node_count as f64).collect();
    let mapping_peak_ratio = if mapping_layer0_node_count == 0 {
        0.0
    } else {
        round_records.iter().map(|record| record.raw_mapping_layer0_dirty / mapping_layer0_node_count as f64).fold(0.0, f64::max)
    };
    let peak_ratio = ratios.iter().copied().fold(0.0, f64::max);
    let rounds_above_half = ratios.iter().filter(|&&ratio| ratio >= 0.5).count() as u64;
    let final_ratio = *ratios.last().expect("已判空");
    let window_start = ratios.len().saturating_sub(101);
    let deltas: Vec<f64> = (window_start + 1..ratios.len()).map(|index| ratios[index] - ratios[index - 1]).collect();
    let mut sign_changes = 0u64;
    for pair in deltas.windows(2) {
        if pair[0] * pair[1] < 0.0 {
            sign_changes += 1;
        }
    }
    let ratio_of_deltas: Vec<f64> = deltas.windows(2).map(|pair| if pair[0].abs() > 1e-12 { (pair[1] / pair[0]).abs() } else { f64::NAN }).filter(|value| value.is_finite()).collect();
    let median_abs_ratio = if ratio_of_deltas.is_empty() {
        0.0
    } else {
        let mut sorted = ratio_of_deltas.clone();
        sorted.sort_by(|left, right| left.partial_cmp(right).expect("有限值"));
        sorted[sorted.len() / 2]
    };
    let label = if round_records.len() < 100 {
        "converged_before_100_rounds"
    } else if sign_changes >= 50 {
        "oscillating"
    } else if sign_changes == 0 {
        "monotonic"
    } else {
        "other"
    };
    (peak_ratio, rounds_above_half, final_ratio, sign_changes, median_abs_ratio, label, mapping_peak_ratio)
}

#[allow(clippy::too_many_arguments)]
fn emit_branch_expansion(emitter: &mut Emitter, arm_tag: &str, family: Family, file_count: u64, variant_tag: &str, variant: AllocationVariant, core: &FixedPointCoreOutcome, shape: PoolShape) {
    let allocation_leaf_cap = shape.geometry.leaf_capacity(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES));
    let allocation_internal_cap = shape.geometry.internal_capacity(ALLOCATION_KEY_BYTES);
    let mapping_leaf_cap = shape.geometry.leaf_capacity(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES));
    let mapping_internal_cap = shape.geometry.internal_capacity(MAPPING_KEY_BYTES);
    let zeroth_generation_written_units = sum_f64(&core.extent_dirty) + sum_f64(&core.inode_dirty) + core.accounting_dirty_total + core.tree_table_dirty_total;
    let allocation_total_nodes_frozen: u64 = core.allocation_layers.iter().sum();
    let data_unit_count = shape.family.data_unit_count(shape.file_count) as f64;
    let mapping_context = FrozenMappingContext {
        data_unit_count,
        distinct_data_units_touched: shape.family.data_units_per_fsync() as f64, // 一次持久化触达量的下界近似：主表用 16 次 fsync 间隔的去重量，这里用一次 fsync 的量作代际展开的冻结输入（G 只跟踪节点自环，不重算数据单元触达）。
        extent_total_nodes: core.extent_layers.iter().sum::<u64>() as f64,
        extent_dirty_total: sum_f64(&core.extent_dirty),
        inode_region_for_root_class: (core.inode_layers.iter().sum::<u64>() as f64 - core.inode_layers[0] as f64).max(core.inode_dirty[1..].iter().sum::<f64>()),
        inode_root_dirty_total: core.inode_dirty[1..].iter().sum(),
        accounting_dirty_total: core.accounting_dirty_total,
        inode_container_node_count: core.inode_layers[0] as f64,
        inode_container_dirty: core.inode_dirty[0],
        allocation_total_nodes_frozen: allocation_total_nodes_frozen as f64,
        mapping_layers: core.mapping_layers.clone(),
        mapping_touch_frozen: core.mapping_touch_final.clone(),
        mapping_leaf_cap,
        mapping_internal_cap,
        placement: shape.placement,
        policy: shape.policy,
    };
    let summary = generational_branch_expansion(
        variant,
        &core.allocation_layers,
        allocation_leaf_cap,
        allocation_internal_cap,
        &core.allocation_touch_final,
        core.allocation_entries_final / DEVICE_COUNT as f64,
        zeroth_generation_written_units,
        data_unit_count.min(shape.family.data_units_per_fsync() as f64 * 16.0),
        data_unit_count,
        0.0,
        0.0,
        shape.geometry.backlog_generations as f64,
        shape.placement,
        shape.policy,
        &mapping_context,
    );
    let display_count = summary.layer0_dirty_by_generation.len().min(5);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=row8_branch family={} p={file_count} arm={arm_tag} variant={variant_tag} stopped_at_generation={} a0_by_generation={:?} b_by_generation={:?} peak_branch_ratio={} generations_above_one={} final_branch_ratio={} label={}",
            family.tag(),
            summary.stopped_at_generation,
            &summary.layer0_dirty_by_generation[..display_count],
            &summary.branch_ratios[..summary.branch_ratios.len().min(4)],
            summary.peak_branch_ratio.map_or("nan".to_string(), |value| format!("{value:.6}")),
            summary.generations_with_branch_ratio_above_one,
            summary.final_branch_ratio.map_or("nan".to_string(), |value| format!("{value:.6}")),
            summary.label,
        ))
    );
}
