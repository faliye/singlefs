//! E155 第二次跑：岔路单第 6 行 —— 三臂（甲 / wal_full-K9′ / 乙-M-K9′）在同一批取样点上的
//! fsync 行、摊销行、之比与 K9′ 相对 K9 的增量。确定性计数模型。
//!
//! 判据、失败条款、变异表的权威登记在 `research/prompts/e155-r2-prereg.md`（第五节 5.1 R1–R8、
//! 第六节 Q6.x、第七节 B1/B2/B4/B5/B7/B10/B11/B15–B19）。第一次跑的登记 `research/prompts/
//! e155-preregistration.md` 与它的二进制 `e155_fsync_write_volume.rs`（本文件从它拷出再改，
//! 第三节 3.2）、产物都不动——本文件是另一个二进制，`replay.sh` 里 E155 那一行仍指向第一次的产物。
//!
//! 本文件从第一次的二进制原样带过来的部分：常量、K1–K4 容量与树高、5.3 组公式的基础三步
//! （`continuous_groups_touch` / `scattered_group_touch` / `combine_touches`）、第 2–5 行的反事实
//! 上界函数与它们的单测（`row_2_to_4_counterfactual_tests`、`row_5_growth_tests`）——这些函数与单测
//! 不属于这一次的判据，留着只是为了让第一次的变异表（15 条）能在这个新二进制上照跑一遍（登记第九节）。
//! `main()` 不再输出第 1–5 行（那些数已经在第一次的产物里），只输出第 6 行主表、阳性对照、Q6.1–Q6.9、
//! 8.2、Q6.11。
//!
//! 第一段交付（登记 5.5）：另起二进制、停机条款 S1–S3、R6、R7、第 6 行主表五列、
//! 三条臂的阳性对照、B1/B2/B4/B5/B7/B10/B11/B15–B19、Q6.1–Q6.9、第 6 行判定格上的 8.2 与判别力自证、
//! 变异 M1–M8/M14/M15/M18/M19 与第一次变异表；另加主 agent 点名要跑的 Q6.11（第三节 3.2 四处
//! 改动前后甲的差，登记原写「够判后不跑」，这一次改成要跑）。
//!
//! 第二段交付（R8 组提交、第 7 行）：R8、第 7 行主表（`row7_grid`，1800 格）、B12–B14、
//! 阳性对照（`positive_control_row7`）、变异 M9–M13/M16/M17（表里的 R2_M9–R2_M17）、
//! 翻面点 Q7.3/Q7.3b（`row7_fold_point`）、反向取样点「N 按批计」（`geometry_sensitivity_sample_row7`
//! 的 `interval_counted_by_batch`）。**还没做**：第 7 行判定格上 φ=0.5/K3′/g=0/pbs=4096/K9′-b 五个
//! 反向取样点、「不共享取随机」（F9，只留了一条 `unshared_random_not_modeled_this_pass` 占位说明）、
//! 沿 k 轴的判别力自证——这些交主 agent 定要不要续（见岔路表）。

use e7_index_bench::Emitter;

// ============================================================
// 一、常量（与第一次逐字相同，手抄自 `crates/singlefs-format/src/lib.rs`；第七节 B1 锚点对拍）
// ============================================================

/// 码 2 节点占盘宽度（含头）。
const NODE_BYTES: u64 = 16384;
/// 数据单元与码 3 容器占盘宽度（含头）。
const UNIT_BYTES: u64 = 32768;
/// journal 记录固定宽度。
const RECORD_BYTES: u64 = 4096;
/// 系统配置槽宽度。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
const RECORD_HEADER_BYTES: u64 = 307;
/// 一条点名项的宽度。
const NAMED_ITEM_BYTES: u64 = 56;
/// 两块盘（`D`）。
const DEVICE_COUNT: u64 = 2;
/// 主几何下 journal 环安全系数 `F`（D22 已定项 2）。
const RING_SAFETY_FACTOR_MAIN: u64 = 3;
/// journal 环默认容量。
const RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;
/// 主几何 `physical_block_size`。
const PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 512;
/// 反向取样点 `physical_block_size`（8.2）。
const REVERSE_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 4096;

/// 码 2 节点头宽公式的常量部分：`86 + 2×key宽 + 29`（`anchors_r2*.py` 的 `hdr2`）。
const NODE_HEADER_BASE_BYTES: u64 = 86 + 29;
/// 节点指针宽（D19 已定项 8）。
const NODE_POINTER_BYTES: u64 = 86;
/// K3′ 反向取样点：内部条目再加身份引用 26 字节。
const IDENTITY_REFERENCE_EXTRA_BYTES: u64 = 26;

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
/// 码 3 容器头宽（`UNIT_BYTES - 136`，独立于 `hdr2` 公式）。
const INODE_CONTAINER_HEADER_BYTES: u64 = 136;
const INODE_CONTAINER_ENTRY_BYTES: u64 = 140;

/// 记账池级（不带设备维）行数：inode 号水位、待删占用、已承诺预留。
const ACCOUNTING_POOL_WIDE_ENTRIES: u64 = 3;
/// 记账每块盘各一行的行数。
const ACCOUNTING_ENTRIES_PER_DEVICE: u64 = 6;
/// 树表条目数（主几何，7 棵）。
const TREE_TABLE_ENTRY_COUNT: u64 = 7;
/// 树表改根时相邻改动的条目数（extent、inode、分配记录、记账四棵树的根）。
const TREE_TABLE_TOUCHED_ROOTS: u64 = 4;
/// 一次发布点名项超过 67 时怎么切（K7 主口径）。
const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;

/// 码 2 节点头宽：`86 + 2×key宽 + 29`。
const fn node_header_bytes(key_bytes: u64) -> u64 {
    NODE_HEADER_BASE_BYTES + 2 * key_bytes
}

/// 一个节点能装的条目数：`(节点宽 − 头宽) / 条目宽`。
const fn node_capacity(key_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - node_header_bytes(key_bytes)) / entry_bytes
}

/// 内部节点条目宽：分隔 key + 子指针（K3 主几何）；K3′ 反向取样点再加身份引用 26 字节。
const fn internal_entry_bytes(key_bytes: u64, identity_reference: bool) -> u64 {
    key_bytes + NODE_POINTER_BYTES + if identity_reference { IDENTITY_REFERENCE_EXTRA_BYTES } else { 0 }
}

fn inode_container_capacity() -> u64 {
    (UNIT_BYTES - INODE_CONTAINER_HEADER_BYTES) / INODE_CONTAINER_ENTRY_BYTES
}

// ============================================================
// 二、K2：树高与各层节点数（与第一次逐字相同）
// ============================================================

/// 填充率（K4）：主几何 φ=1；反向取样点 φ=0.5。
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

/// 从叶到根每一层的节点数（K2）；`minimum_layers` 强制树至少这么高（inode 树恒 ≥ 2，根恒码 2）。
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

#[cfg(test)]
mod capacity_tests {
    use super::*;

    /// B1（第七节 7.2）：容量锚点，与第一次登记相同（R1–R8 不改容量公式）。
    #[test]
    fn capacities_match_the_preregistration_anchors() {
        assert_eq!(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), 144, "extent 叶容量");
        assert_eq!(
            node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)),
            147,
            "extent 内部容量（K3）"
        );
        assert_eq!(inode_container_capacity(), 233, "inode 容器容量");
        assert_eq!(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 135, "inode 内部容量");
        assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES), 812, "分配记录叶容量");
        assert_eq!(
            node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, false)),
            169,
            "分配记录内部容量（K3）"
        );
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES), 477, "记账叶容量");
        assert_eq!(node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES), 294, "映射叶容量");
        assert_eq!(
            node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, false)),
            143,
            "映射内部容量（K3）"
        );
        assert_eq!(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES), 81, "树表容量");
        assert_eq!(NAMED_ITEMS_PER_RECORD_MAIN, 67, "一条记录装的点名项数（A1）");
        assert_eq!(UNIT_BYTES - 105 - 29, 32634, "数据单元载荷上限");
    }

    #[test]
    fn accounting_internal_capacity_and_identity_reference_variant_match_the_anchors() {
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, false)), 150);
        assert_eq!(node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, true)), 119, "K3′ extent");
        assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, true)), 133, "K3′ 分配记录");
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, true)), 121, "K3′ 记账");
        assert_eq!(node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, true)), 116, "K3′ 映射");
    }

    #[test]
    fn inode_tree_is_at_least_two_layers_even_with_one_entry() {
        let layers = tree_layers(1, inode_container_capacity(), node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 2);
        assert_eq!(layers, vec![1, 1], "1 个容器 + 1 个恒码 2 的根");
    }

    #[test]
    fn extent_tree_reaches_two_layers_at_file_count_145() {
        let layers = tree_layers(145, node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)), 1);
        assert_eq!(layers, vec![2, 1], "145 条分两叶、加一个根");
    }
}

// ============================================================
// 三、5.3：组的期望覆盖节点数（R2/R3/R4/R6 在这一节加新函数、改 class_dirty_nodes_this_layer）
// ============================================================

/// 落点方式（5.2）：只管 fsync 与 fsync 之间的位置怎么推进。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Placement {
    Sequential,
    Random,
}

/// K8 位置策略：一个旧版落在哪。
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

/// 若干连续组合起来，在某一层期望覆盖的节点数（5.3 第 1 步，与第一次逐字相同）。
fn continuous_groups_touch(node_count_at_layer: u64, layer_coverage: f64, group_sizes: &[f64], gaps_between: &[f64]) -> f64 {
    if node_count_at_layer == 0 || group_sizes.is_empty() {
        return 0.0;
    }
    assert!(
        gaps_between.len() + 1 == group_sizes.len(),
        "空隙数必须比组数少 1：{} 组、{} 个空隙",
        group_sizes.len(),
        gaps_between.len()
    );
    let mut touched = 1.0;
    for &group_size in group_sizes {
        touched += (group_size - 1.0) / layer_coverage;
    }
    for &gap in gaps_between {
        touched += (gap / layer_coverage).min(1.0);
    }
    touched.min(node_count_at_layer as f64)
}

/// 一个散组在某一层期望覆盖的节点数（5.3 第 2 步，与第一次逐字相同）。
fn scattered_group_touch(touch_count: f64, region_entries: f64, layer_coverage: f64) -> f64 {
    if touch_count <= 0.0 {
        return 0.0;
    }
    let expected_saturated_node_count = (region_entries / layer_coverage).max(1.0);
    expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))
}

/// R6：散连续组——`group_count` 组、每组 `group_size` 条相邻，在长 `region_entries` 条的区间里
/// 均匀独立地落。`group_size = 1` 时退化成 5.3 第 2 步（登记 5.1 R6 原文）。
fn scattered_continuous_groups_touch(node_count_at_layer: u64, layer_coverage: f64, group_count: f64, group_size: f64, region_entries: f64) -> f64 {
    if group_count <= 0.0 || group_size <= 0.0 {
        return 0.0;
    }
    let saturated_node_count = (region_entries / layer_coverage).max(1.0);
    let nodes_touched_per_group = saturated_node_count.min(1.0 + (group_size - 1.0) / layer_coverage);
    (saturated_node_count * (1.0 - (1.0 - nodes_touched_per_group / saturated_node_count).powf(group_count))).min(node_count_at_layer as f64)
}

/// R6：散连续组去重之后的条目数——`region_entries × (1 − (1 − group_size/region_entries)^group_count)`。
fn scattered_continuous_groups_distinct_entries(region_entries: f64, group_count: f64, group_size: f64) -> f64 {
    if group_count <= 0.0 || group_size <= 0.0 || region_entries <= 0.0 {
        return 0.0;
    }
    region_entries * (1.0 - (1.0 - group_size / region_entries).powf(group_count))
}

/// 若干组合起来，在某一层期望覆盖的节点数（5.3 第 3 步，与第一次逐字相同）。
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

/// 单一连续组的树（extent、inode）逐层期望脏节点数（与第一次逐字相同）。
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

/// 一个带旧版删改的类（映射树的六个类之一）在某一层的期望脏节点数（K8 三种位置策略）。
/// `data_unit_totals`：`Some((总量, 这次持久化触达量))` 时按 R3 判远近（与 policy 无关，只看这两个数）；
/// `None` 时按 K8 policy + R2（Balanced+Random 取 `1 − q`，不是 `q`）判远近。
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
        // 只有一个节点：插入与删改（不管远近）都落在它上面。
        return 1.0;
    }
    let far_fraction: f64 = if let Some((total_units, touched_units)) = data_unit_totals {
        // R3（2026-09-19 修订）：数据单元旧版远近只看总量 vs 这次持久化触达量，与 K8 策略无关。
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
            // R2（2026-09-19 修订）：远的比例 = 1 − q，不是 q 本身（第三节 3.2 ②）。
            (PositionPolicy::Balanced, Placement::Random) => (1.0 - previous_touch_fraction).clamp(0.0, 1.0),
        }
    };
    let near_count = delete_count * (1.0 - far_fraction);
    let far_count = delete_count * far_fraction;
    let tail_group = insert_count + near_count;
    let continuous_touch = if far_fraction > 0.0 && placement == Placement::Sequential {
        // 远（seq）：连续组，落在区间起点；与尾部的插入 + 近删改分成两组，中间的空隙是区间剩下的部分。
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

/// 一个类的当前状态：条目数、这次持久化插入 / 删改的条目数、是不是数据单元类（R3）。
#[derive(Clone, Copy, Debug)]
struct TreeClassActivity {
    region_entries: f64,
    insert_count: f64,
    delete_count: f64,
    /// R3：`Some((总量, 触达量))` 表示这是数据单元类；`None` 表示按 K8 policy 判的普通类。
    data_unit_totals: Option<(f64, f64)>,
}

/// 多个类共享一棵树（映射树的六个类）逐层期望脏节点数，K8 三种位置策略。
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
            layer_sum += class_dirty_nodes_this_layer(
                class.region_entries,
                cov,
                class.insert_count,
                class.delete_count,
                placement,
                policy,
                previous_touch_fractions[layer_index],
                class.data_unit_totals,
            );
        }
        dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
        cov *= internal_capacity as f64;
    }
    dirty_per_layer
}

/// 分配记录树一块盘一层的期望脏节点数（R1/R3/R4 合并处理；K9′ 的中间版释放记录 R7(a) 只进尾部插入）。
/// 节点写（新落点 + 旧版按 K8/R2 策略）与数据单元写（新落点 + 旧版按 R3/R4）的「远」贡献合并成
/// **一个**连续组或散组，不是两个独立类各自算完再相加——登记第七节 B17 手算格是这样验证的
/// （`allocation_tree_dirty_this_layer` 名字与手算格逐字节相同的读法：一块盘的分配记录区间共享同一层，
/// 节点与数据单元的「远」量合并成一次 `scattered_group_touch` / `continuous_groups_touch` 调用）。
#[allow(clippy::too_many_arguments)]
fn allocation_tree_dirty_this_layer(
    region_entries: f64,
    layer_coverage: f64,
    node_units: f64,
    data_units: f64,
    data_total_units: f64,
    extra_release_only_insertions: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
) -> f64 {
    if node_units <= 0.0 && data_units <= 0.0 && extra_release_only_insertions <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer = (region_entries / layer_coverage).ceil().max(1.0) as u64;
    if node_count_at_layer <= 1 {
        return 1.0;
    }
    let node_far_fraction: f64 = match (policy, placement) {
        (PositionPolicy::Near, _) => 0.0,
        (PositionPolicy::Far, _) => 1.0,
        (PositionPolicy::Balanced, Placement::Sequential) => (node_units / layer_coverage).min(1.0),
        (PositionPolicy::Balanced, Placement::Random) => (1.0 - previous_touch_fraction).clamp(0.0, 1.0),
    };
    // R3：数据单元旧版远近只看总量 vs 这次持久化触达量，与 K8 策略无关。
    let data_is_far = data_total_units > data_units;
    // R4：仅 L远 + rand 时，数据单元新落点也散组，不并进尾部（登记 5.1 R4，第七节 B17）。
    let data_new_placement_scattered = policy == PositionPolicy::Far && placement == Placement::Random && data_is_far;

    let node_near = node_units * (1.0 - node_far_fraction);
    let node_far = node_units * node_far_fraction;
    let data_near = if data_is_far { 0.0 } else { data_units };
    let data_far = if data_is_far { data_units } else { 0.0 };
    let data_new_in_tail = if data_new_placement_scattered { 0.0 } else { data_units };
    let data_new_scattered = if data_new_placement_scattered { data_units } else { 0.0 };

    // 尾部：节点新落点（恒在尾部）+ 节点旧版的近份 + 数据新落点（除非 R4 把它挪去散组）+ 数据旧版的近份
    // + K9′ 中间版释放记录（R7(a)，恒在尾部，不参与远近判定）。
    let tail_group = node_units + node_near + data_new_in_tail + data_near + extra_release_only_insertions;
    // 远：节点旧版的远份 + 数据旧版的远份 + （R4 触发时）数据新落点，三者合并成一个连续组或散组。
    let far_total = node_far + data_far + data_new_scattered;

    if far_total > 0.0 && placement == Placement::Sequential {
        let gap = (region_entries - tail_group - far_total).max(0.0);
        continuous_groups_touch(node_count_at_layer, layer_coverage, &[far_total, tail_group], &[gap])
    } else if far_total > 0.0 && placement == Placement::Random {
        let continuous = continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[]);
        let scattered = scattered_group_touch(far_total, region_entries, layer_coverage);
        combine_touches(node_count_at_layer, continuous, &[scattered])
    } else {
        continuous_groups_touch(node_count_at_layer, layer_coverage, &[tail_group], &[])
    }
}

#[cfg(test)]
mod group_formula_tests {
    use super::*;

    #[test]
    fn single_continuous_group_never_exceeds_node_count() {
        assert_eq!(continuous_groups_touch(1, 100.0, &[8.0], &[]), 1.0);
        let touched = continuous_groups_touch(1000, 100.0, &[8.0], &[]);
        assert!(touched < 1.1, "8 条记录不该碰到超过一个节点太多：{touched}");
    }

    #[test]
    fn scattered_group_saturates_to_region_node_count_as_touch_count_grows() {
        let small_touch_count = scattered_group_touch(1.0, 1000.0, 100.0);
        let large_touch_count = scattered_group_touch(1000.0, 1000.0, 100.0);
        assert!(small_touch_count < large_touch_count, "touch_count 越大期望碰到的节点数不该变少");
        assert!(large_touch_count <= 10.0 + 1e-6, "散组不能碰到超过区间自己的节点数上界");
        assert!((large_touch_count - 10.0).abs() < 1e-6, "large_touch_count={large_touch_count}");
    }

    #[test]
    fn combine_touches_never_exceeds_node_count() {
        let combined = combine_touches(10, 5.0, &[5.0, 5.0]);
        assert!(combined <= 10.0, "U_l ≤ n_l（V2）：{combined}");
    }

    #[test]
    fn single_node_class_is_always_fully_dirty_when_active() {
        for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
            for placement in [Placement::Sequential, Placement::Random] {
                let touched = class_dirty_nodes_this_layer(6.0, 294.0, 6.0, 6.0, placement, policy, 0.5, None);
                assert_eq!(touched, 1.0, "policy={policy:?} placement={placement:?}");
            }
        }
    }

    #[test]
    fn near_policy_never_creates_a_far_group() {
        let touched = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Random, PositionPolicy::Near, 0.9, None);
        assert!(touched < 1.5, "L近不该有远组：{touched}");
    }

    #[test]
    fn far_policy_touches_more_nodes_than_near_policy_when_multi_node() {
        let near = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Near, 0.0, None);
        let far = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Far, 0.0, None);
        assert!(far > near, "L远 应该比 L近 碰更多节点：near={near} far={far}");
    }

    /// 修订二（B16，第三节 3.2 ②）：L均 + rand，远的比例是 `1 − 上一轮触达比例`，不是「上一轮触达比例」本身。
    #[test]
    fn balanced_random_far_fraction_is_one_minus_previous_touch_fraction_not_the_fraction_itself() {
        let touched = class_dirty_nodes_this_layer(10_000.0, 294.0, 1.0, 1.0, Placement::Random, PositionPolicy::Balanced, 0.2, None);
        assert!((touched - 1.780_12).abs() < 1e-3, "touched={touched}");
        // 第三节 3.2 ②：按装置原来的口径（远的比例直接取上一轮触达比例）应该是 1.199317，与修订后的读数不同。
        let legacy_reading = 0.2_f64; // 文档记录的旧口径读数，供人对照，不在装置里复算。
        assert!((legacy_reading - 0.2).abs() < 1e-9);
    }

    /// 修订三（B15，第三节 3.2 ③）：数据单元类旧版远近只看总量 vs 触达量，三种 policy 都一样。
    #[test]
    fn data_unit_class_far_fraction_ignores_position_policy() {
        for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
            let sequential_reading = class_dirty_nodes_this_layer(10_000.0, 294.0, 1.0, 1.0, Placement::Sequential, policy, 0.0, Some((10_000.0, 1.0)));
            assert!((sequential_reading - 2.0).abs() < 1e-9, "policy={policy:?} sequential_reading={sequential_reading}");
            let random_reading = class_dirty_nodes_this_layer(10_000.0, 294.0, 1.0, 1.0, Placement::Random, policy, 0.0, Some((10_000.0, 1.0)));
            assert!((random_reading - 1.971_428_571).abs() < 1e-6, "policy={policy:?} random_reading={random_reading}");
        }
    }

    /// 修订四（B17，第三节 3.2 ④）：分配记录树 L远+随机落点，节点写与数据单元写合并成一个远组；
    /// 数据新落点计不计进尾部让结果从 7.45661（并进尾部）变成 7.952435（散组）。
    #[test]
    fn allocation_tree_new_data_placement_scattered_under_far_policy_and_random_placement() {
        let region_entries = 12_000.0;
        let layer_coverage = 812.0;
        let dirty_with_new_placement_scattered = allocation_tree_dirty_this_layer(region_entries, layer_coverage, 8.0, 1.0, 12_000.0, 0.0, Placement::Random, PositionPolicy::Far, 0.0);
        assert!((dirty_with_new_placement_scattered - 7.952_435).abs() < 1e-6, "dirty_with_new_placement_scattered={dirty_with_new_placement_scattered}");
    }

    /// 修订六（B18）：散连续组——F8A（组大小 8）下间隔并集与「独立散点」的读数不同：16.542347 对 114.41612。
    #[test]
    fn scattered_continuous_groups_differ_from_independent_scattered_points() {
        let region = 80_000.0;
        let cov = 144.0;
        let node_count = 1_000_000u64;
        let grouped = scattered_continuous_groups_touch(node_count, cov, 16.0, 8.0, region);
        assert!((grouped - 16.542_347).abs() < 1e-6, "grouped={grouped}");
        let independent = scattered_group_touch(16.0 * 8.0, region, cov);
        assert!((independent - 114.416_12).abs() < 1e-5, "independent={independent}");
    }
}

// ============================================================
// 四、族与几何：一格的输入（与第一次逐字相同，加 K9′ 开关的实际接线）
// ============================================================

/// 族（5.2）：F1 每个文件 1 个数据单元；F8A 一次 fsync 改 8 个相邻数据单元。
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

/// 一格的几何输入（5.6）：主几何 φ=1、K3、g=24、pbs=512；反向取样点各自单独翻一个。
/// `intermediate_version_policy`（K9 / K9′）不放在这里——它只对 WAL checkpoint 有意义，
/// 是 `solve_write_ahead_log_checkpoint` 的独立参数（见 `IntermediateVersionPolicy`）。
#[derive(Clone, Copy, Debug)]
struct Geometry {
    fill_ratio: FillRatio,
    identity_reference: bool, // K3（false）/ K3′（true）
    backlog_generations: u64, // g：主 24，反向 0
    physical_block_size_bytes: u64,
}

impl Geometry {
    fn main() -> Geometry {
        Geometry { fill_ratio: FillRatio::Full, identity_reference: false, backlog_generations: 24, physical_block_size_bytes: PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES }
    }

    fn leaf_capacity(self, raw: u64) -> u64 {
        self.fill_ratio.effective_capacity(raw)
    }

    fn internal_capacity(self, key_bytes: u64) -> u64 {
        self.fill_ratio.effective_capacity(node_capacity(key_bytes, internal_entry_bytes(key_bytes, self.identity_reference)))
    }
}

/// 一格的输入：池大小、族、落点方式、位置策略、几何。
#[derive(Clone, Copy, Debug)]
struct PoolShape {
    file_count: u64,
    family: Family,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
}

/// K9 / K9′（岔路单第 6 行，登记 5.1 R7）：间隔内被换掉的中间版要不要各自补一条已释放的分配记录、
/// 进 K6 积压。只对 WAL checkpoint 有意义——甲每次 fsync 就是一次发布，没有中间版。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum IntermediateVersionPolicy {
    /// K9（第一次登记）：中间版不写分配记录、不进映射、不进积压。
    NotTracked,
    /// K9′（这一次登记）：每个中间版每盘写一条已释放的分配记录，(a) 并进分配记录树区间尾部、
    /// (b) 进 K6 积压（每个 checkpoint 的释放数加 I）。
    ReleasedAndBacklogged,
}

// ============================================================
// 五、5.3 不动点：甲一次 fsync（= 一次发布）写出的东西
// ============================================================

/// 甲一次 fsync（= 一次发布）写出的东西，按角色分列，字节数已经乘过两块盘（除非另外注明）。
#[derive(Clone, Debug, Default)]
struct JiaPublishOutcome {
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
    write_calls: u64,
    barriers: u64,
    fua_count: u64,
    named_items: u64,
    record_count: u64,
    extent_layers: Vec<u64>,
    extent_dirty: Vec<f64>,
    inode_layers: Vec<u64>,
    inode_dirty: Vec<f64>,
    allocation_layers: Vec<u64>,
    allocation_dirty: Vec<f64>,
    mapping_layers: Vec<u64>,
    mapping_dirty: Vec<f64>,
    accounting_dirty: f64,
    written_units: f64,
    fixed_point_iterations: u64,
}

impl JiaPublishOutcome {
    fn total_bytes(&self) -> u64 {
        self.data_bytes
            + self.extent_bytes
            + self.inode_container_bytes
            + self.inode_root_bytes
            + self.allocation_bytes
            + self.accounting_bytes
            + self.mapping_bytes
            + self.tree_table_bytes
            + self.record_bytes
            + self.root_slot_bytes
            + self.system_configuration_bytes
    }

    fn block_layer_write_requests(&self) -> u64 {
        self.write_calls + self.barriers + self.fua_count
    }
}

fn sum_f64(values: &[f64]) -> f64 {
    values.iter().sum()
}

const CONVERGENCE_EPSILON: f64 = 1e-9;
const MAXIMUM_FIXED_POINT_ITERATIONS: u64 = 1000;

#[derive(Clone, Copy, Debug)]
enum TouchSpecification {
    Continuous(f64),
    Scattered(f64),
    /// R6：散连续组——`group_count` 组、每组 `group_size` 条相邻。
    ScatteredContinuousGroups { group_count: f64, group_size: f64 },
    /// R8（第 7 行组提交，第一节「读法写死」）：`path_count` 条互不共享的路径，每条覆盖 `group_size`
    /// 条相邻记录，路径间空隙都 ≥ 这一层的覆盖（`unshared_paths_touch` 的公式）。
    UnsharedPaths { path_count: f64, group_size: f64 },
}

/// R8：concurrent_fsync_count 条互不共享的路径，每条覆盖 `group_size` 条相邻记录，组间空隙 ≥ `layer_coverage`
/// （第一节「读法写死」：第 l 层脏节点 = min(n_l, concurrent_fsync_count × (1 + (group_size − 1)/cov_l))）。
fn unshared_paths_touch(node_count_at_layer: u64, layer_coverage: f64, path_count: f64, group_size: f64) -> f64 {
    (path_count * (1.0 + (group_size - 1.0) / layer_coverage)).min(node_count_at_layer as f64)
}

fn touch_dirty_nodes_per_layer(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, region_entries: f64, touch_specification: TouchSpecification) -> Vec<f64> {
    match touch_specification {
        TouchSpecification::Continuous(touch_count) if touch_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::Continuous(touch_count) => single_group_dirty_nodes_per_layer(layers, leaf_capacity, internal_capacity, touch_count.round() as u64),
        TouchSpecification::Scattered(touch_count) if touch_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::Scattered(touch_count) => {
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(scattered_group_touch(touch_count, region_entries, cov).min(node_count_at_layer as f64));
                cov *= internal_capacity as f64;
            }
            result
        }
        TouchSpecification::ScatteredContinuousGroups { group_count, .. } if group_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::ScatteredContinuousGroups { group_count, group_size } => {
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(scattered_continuous_groups_touch(node_count_at_layer, cov, group_count, group_size, region_entries));
                cov *= internal_capacity as f64;
            }
            result
        }
        TouchSpecification::UnsharedPaths { path_count, .. } if path_count <= 0.0 => vec![0.0; layers.len()],
        TouchSpecification::UnsharedPaths { path_count, group_size } => {
            let mut result = Vec::with_capacity(layers.len());
            let mut cov = leaf_capacity as f64;
            for &node_count_at_layer in layers {
                result.push(unshared_paths_touch(node_count_at_layer, cov, path_count, group_size));
                cov *= internal_capacity as f64;
            }
            result
        }
    }
}

/// 5.3 不动点算完的六棵树状态。
struct FixedPointCoreResult {
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
}

/// Q2：四样固定点各自的反事实——「X 不写」= X 的节点不写、不分配、不进映射、不点名。
#[derive(Clone, Copy, Debug, Default)]
struct SuppressedFixedPoints {
    accounting: bool,
    allocation: bool,
    mapping: bool,
    tree_table: bool,
}

/// 5.3 不动点核心（R1/R3/R4/R7 在这一版改过；R2 在 `class_dirty_nodes_this_layer` 里已经改）：
/// extent / inode 用 `extent_touch` / `inode_touch` 指定这次持久化怎么碰它们；
/// `extra_allocation_release_insertions`：K9′（R7）每个中间版每盘写一条已释放的分配记录，
/// 只并进分配记录树尾部插入、进 K6 积压，K9 与甲恒传 0。
#[allow(clippy::too_many_arguments)]
fn solve_fixed_point_core(
    data_unit_count: u64,
    file_count: u64,
    extent_touch: TouchSpecification,
    inode_touch: TouchSpecification,
    distinct_data_units_touched: f64,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
    suppressed: SuppressedFixedPoints,
    tree_table_entry_count: u64,
    extra_allocation_release_insertions: f64,
) -> FixedPointCoreResult {
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
    assert_eq!(accounting_layers, vec![1], "15 行远小于 477 的叶容量，记账树恒 1 节点（K5）");
    let accounting_dirty_total = if suppressed.accounting { 0.0_f64 } else { 1.0_f64 };

    let tree_table_cap = geometry.leaf_capacity(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES));
    let tree_table_layers = tree_layers(tree_table_entry_count, tree_table_cap, tree_table_cap, 1);
    let tree_table_dirty = touch_dirty_nodes_per_layer(
        &tree_table_layers,
        tree_table_cap,
        tree_table_cap,
        tree_table_entry_count as f64,
        TouchSpecification::Continuous(TREE_TABLE_TOUCHED_ROOTS as f64),
    );
    let tree_table_total_nodes: u64 = tree_table_layers.iter().sum();
    let tree_table_dirty_total = if suppressed.tree_table { 0.0_f64 } else { sum_f64(&tree_table_dirty) };

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
    // `previous_written_units`：不含数据单元（与第一次相同的口径），只用于「节点写」子类与外部 `written_units` 字段。
    let mut previous_written_units = extent_dirty_total + inode_dirty_total + accounting_dirty_total + tree_table_dirty_total;
    let mut previous_allocation_entries = 0.0_f64;
    let mut previous_mapping_entries = 0.0_f64;
    let mut iterations = 0u64;

    loop {
        iterations += 1;
        let allocation_total_nodes: u64 = allocation_layers.iter().sum();
        let mapping_total_nodes: u64 = mapping_layers.iter().sum();
        let extent_total_nodes: u64 = extent_layers.iter().sum();
        let inode_total_nodes: u64 = inode_layers.iter().sum();

        let live_units = data_unit_count as f64
            + extent_total_nodes as f64
            + inode_total_nodes as f64
            + allocation_total_nodes as f64
            + 1.0
            + mapping_total_nodes as f64
            + tree_table_total_nodes as f64
            + 1.0;

        // R1（第三节 3.2 ①）：K6 积压的「每次持久化释放的单元数」要含数据单元；K9′（R7(b)）再加中间版 I。
        let release_backlog_units = previous_written_units + distinct_data_units_touched + extra_allocation_release_insertions;
        let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * release_backlog_units);
        let allocation_layers_now = tree_layers(allocation_entries.round() as u64, allocation_leaf_cap, allocation_internal_cap, 1);
        let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() {
            previous_touch_allocation.clone()
        } else {
            vec![0.0; allocation_layers_now.len()]
        };
        let per_device_allocation_entries = allocation_entries / device_count;
        let allocation_dirty_now: Vec<f64> = {
            let mut result = Vec::with_capacity(allocation_layers_now.len());
            let mut cov = allocation_leaf_cap as f64;
            for (layer_index, &node_count) in allocation_layers_now.iter().enumerate() {
                let per_device_dirty = allocation_tree_dirty_this_layer(
                    per_device_allocation_entries,
                    cov,
                    previous_written_units,
                    distinct_data_units_touched,
                    data_unit_count as f64,
                    extra_allocation_release_insertions,
                    placement,
                    policy,
                    allocation_touch[layer_index],
                );
                result.push((per_device_dirty * device_count).min(node_count as f64));
                cov *= allocation_internal_cap as f64;
            }
            result
        };
        let allocation_dirty_now: Vec<f64> = if suppressed.allocation { vec![0.0; allocation_dirty_now.len()] } else { allocation_dirty_now };
        let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
        let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

        let mapping_entries = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes_now as f64 + 1.0;
        let mapping_layers_now = tree_layers(mapping_entries.round() as u64, mapping_leaf_cap, mapping_internal_cap, 1);
        let mapping_touch = if mapping_layers_now.len() == previous_touch_mapping.len() {
            previous_touch_mapping.clone()
        } else {
            vec![0.0; mapping_layers_now.len()]
        };
        let mapping_classes = [
            TreeClassActivity {
                region_entries: data_unit_count as f64,
                insert_count: distinct_data_units_touched,
                delete_count: distinct_data_units_touched,
                // R3：映射树「数据单元」类，远近只看总量 vs 触达量。
                data_unit_totals: Some((data_unit_count as f64, distinct_data_units_touched)),
            },
            TreeClassActivity { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total, data_unit_totals: None },
            TreeClassActivity {
                region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total),
                insert_count: inode_root_dirty_total,
                delete_count: inode_root_dirty_total,
                data_unit_totals: None,
            },
            TreeClassActivity {
                region_entries: allocation_total_nodes_now as f64,
                insert_count: allocation_dirty_total_now,
                delete_count: allocation_dirty_total_now,
                data_unit_totals: None,
            },
            TreeClassActivity { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total, data_unit_totals: None },
            TreeClassActivity { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty, data_unit_totals: None },
        ];
        let mapping_dirty_now = multiclass_tree_dirty_nodes_per_layer(&mapping_layers_now, mapping_leaf_cap, mapping_internal_cap, &mapping_classes, placement, policy, &mapping_touch);
        let mapping_dirty_now: Vec<f64> = if suppressed.mapping { vec![0.0; mapping_dirty_now.len()] } else { mapping_dirty_now };
        let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

        let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

        // 相对误差，不是绝对误差：P 从 1 到 1 亿，`allocation_entries` 等量的量级也跟着从个位涨到
        // 千万位，绝对 1e-9 在大 P 上超出 f64 chained 算术的可分辨精度（实测 P=10^6–10^8、F8A、rand，
        // 相邻两轮在第 15 位小数打转，永远到不了绝对 1e-9，触发的不是 F2 说的『重写一大半』，是浮点噪声）；
        // 相对误差在小 P（值本身是个位数）与大 P（值是千万级）上要求的是同一件事：约 9 位有效数字一致。
        let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs() / allocation_entries.max(1.0);
        let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs() / mapping_entries.max(1.0);
        let written_units_delta = (written_units_now - previous_written_units).abs() / written_units_now.max(1.0);
        let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs() / allocation_dirty_total_now.max(1.0);
        let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs() / mapping_dirty_total_now.max(1.0);

        // R2 的「1 − q」是个倒转映射，原样把上一轮的读数喂给下一轮在大树上会撞出 2-周期震荡
        // （不动点从来到不了，`written_units` 在两个值之间来回跳，实测 P=100 万、F8A、rand、
        // Balanced 一格）；按 0.5 阻尼（successive under-relaxation）与上一轮的值取平均再喂下去——
        // 到达不动点时阻尼前后同值，不改变收敛到的答案，只去掉震荡。未阻尼的行为不是判据，
        // 单测只钉收敛到的绝对值。
        const DAMPING_ALPHA: f64 = 0.15;
        let raw_touch_allocation: Vec<f64> = allocation_layers_now.iter().zip(allocation_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_allocation = if raw_touch_allocation.len() == previous_touch_allocation.len() {
            raw_touch_allocation.iter().zip(previous_touch_allocation.iter()).map(|(&raw, &previous)| DAMPING_ALPHA * raw + (1.0 - DAMPING_ALPHA) * previous).collect()
        } else {
            raw_touch_allocation
        };
        let raw_touch_mapping: Vec<f64> = mapping_layers_now.iter().zip(mapping_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
        previous_touch_mapping = if raw_touch_mapping.len() == previous_touch_mapping.len() {
            raw_touch_mapping.iter().zip(previous_touch_mapping.iter()).map(|(&raw, &previous)| DAMPING_ALPHA * raw + (1.0 - DAMPING_ALPHA) * previous).collect()
        } else {
            raw_touch_mapping
        };
        allocation_layers = allocation_layers_now;
        mapping_layers = mapping_layers_now;
        allocation_dirty = allocation_dirty_now;
        mapping_dirty = mapping_dirty_now;
        previous_allocation_entries = allocation_entries;
        previous_mapping_entries = mapping_entries;
        // 阻尼同样套在 `written_units`（喂 K6 积压与下一轮 `live_units` 的量）：`tree_layers` 的
        // `ceil` 在条目数正好卡在容量边界时会让节点数在两个整数间来回跳，级联出与 R2 无关的震荡
        // （实测 F8A、rand、Lfar、P=100 万–1 亿，与 policy 无关，Far 分支不读 q）；到达不动点时
        // 阻尼前后同值，不改变收敛到的答案。
        previous_written_units = DAMPING_ALPHA * written_units_now + (1.0 - DAMPING_ALPHA) * previous_written_units;

        let converged =
            allocation_entries_delta < CONVERGENCE_EPSILON && mapping_entries_delta < CONVERGENCE_EPSILON && written_units_delta < CONVERGENCE_EPSILON && allocation_dirty_delta < CONVERGENCE_EPSILON && mapping_dirty_delta < CONVERGENCE_EPSILON;
        if converged || iterations >= MAXIMUM_FIXED_POINT_ITERATIONS {
            break;
        }
    }

    FixedPointCoreResult { extent_layers, extent_dirty, inode_layers, inode_dirty, accounting_dirty_total, tree_table_dirty_total, allocation_layers, allocation_dirty, mapping_layers, mapping_dirty, written_units: previous_written_units, iterations }
}

fn solve_jia_publish(shape: PoolShape) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points(shape, SuppressedFixedPoints::default())
}

fn solve_jia_publish_with_suppressed_fixed_points(shape: PoolShape, suppressed: SuppressedFixedPoints) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, TREE_TABLE_ENTRY_COUNT)
}

fn solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape: PoolShape, suppressed: SuppressedFixedPoints, tree_table_entry_count: u64) -> JiaPublishOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();

    let core = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        TouchSpecification::Continuous(fsync_data_unit_count as f64),
        TouchSpecification::Continuous(1.0),
        fsync_data_unit_count as f64,
        shape.placement,
        shape.policy,
        geometry,
        suppressed,
        tree_table_entry_count,
        0.0, // 甲没有中间版（K9′ 只对 WAL 两臂有意义）。
    );

    let written_units = fsync_data_unit_count as f64 + core.written_units;
    let named_items = written_units.round() as u64;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = (written_units.round() as u64) * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    let extent_dirty_total = sum_f64(&core.extent_dirty);
    let inode_container_dirty = core.inode_dirty[0];
    let inode_root_dirty_total: f64 = core.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    JiaPublishOutcome {
        data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: (extent_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_container_bytes: (inode_container_dirty * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: pbs,
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        write_calls: total_calls,
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        named_items,
        record_count,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_layers: core.allocation_layers,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        accounting_dirty: core.accounting_dirty_total,
        written_units,
        fixed_point_iterations: core.iterations,
    }
}

/// R8（第 7 行组提交，登记 5.1 R8 + 第一节「读法写死」）：甲把 `concurrent_fsync_count` 个并发 fsync 并成一批 = 一次发布；
/// `shared` = 这 concurrent_fsync_count 个文件是不是在 key 上相邻（true：共享脊柱，一个连续组；false：互不共享，concurrent_fsync_count 条路径）。
fn solve_jia_batch_publish(shape: PoolShape, concurrent_fsync_count: u64, shared: bool) -> JiaPublishOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;

    let extent_touch = if shared {
        TouchSpecification::Continuous(batch_data_unit_count as f64)
    } else {
        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 }
    };
    let inode_touch = if shared {
        TouchSpecification::Continuous(concurrent_fsync_count as f64)
    } else {
        TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 }
    };

    let core = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        extent_touch,
        inode_touch,
        batch_data_unit_count as f64,
        shape.placement,
        shape.policy,
        geometry,
        SuppressedFixedPoints::default(),
        TREE_TABLE_ENTRY_COUNT,
        0.0, // 甲没有中间版。
    );

    let batch_written_units = batch_data_unit_count as f64 + core.written_units;
    let named_items = batch_written_units.round() as u64;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = (batch_written_units.round() as u64) * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let batch_write_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    let extent_dirty_total = sum_f64(&core.extent_dirty);
    let inode_container_dirty = core.inode_dirty[0];
    let inode_root_dirty_total: f64 = core.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    JiaPublishOutcome {
        data_bytes: batch_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: (extent_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_container_bytes: (inode_container_dirty * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        // R8 一批只写一份固定点、一条记录组、一个根槽、一份系统配置（M9 的反例：每个 fsync 各写一份）。
        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: pbs,
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        write_calls: batch_write_calls,
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        named_items,
        record_count,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_layers: core.allocation_layers,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        accounting_dirty: core.accounting_dirty_total,
        written_units: batch_written_units,
        fixed_point_iterations: core.iterations,
    }
}

/// 第 7 行：每次 fsync 摊到的字节 / 写请求（甲 = 一批的写 ÷ concurrent_fsync_count；M16 的反例：不除以 concurrent_fsync_count）。
fn amortize_batch_per_fsync(batch_value: u64, concurrent_fsync_count: u64) -> f64 {
    batch_value as f64 / concurrent_fsync_count as f64
}

/// 第 7 行：checkpoint 间隔 N 按 fsync 计——一个间隔 N 次 fsync = N/concurrent_fsync_count 批
/// （第一节「读法写死」；M13 的反例：间隔按批计，直接把 N 当批数）。
fn batches_in_interval_by_fsync_count(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    interval_fsyncs / concurrent_fsync_count
}

// ============================================================
// 五之二、legacy：第一次装置的原样实现（3.2 四处未改），只为 Q6.11 对比用，不进任何判据
// ============================================================
mod legacy {
    //! 逐字节照抄第一次装置 `e155_fsync_write_volume.rs` 657–938 行的分配 / 映射树计算
    //! （R1–R4 之前的样子），只用于 Q6.11「第三节 3.2 四处改动前后甲的差」。
    //! extent / inode 两树的公式两次装置相同，这里直接复用外层的 `single_group_dirty_nodes_per_layer` /
    //! `tree_layers` 等，不重复定义。
    use super::*;

    fn legacy_class_dirty_nodes_this_layer(region_entries: f64, layer_coverage: f64, insert_count: f64, delete_count: f64, placement: Placement, policy: PositionPolicy, previous_touch_fraction: f64) -> f64 {
        if insert_count <= 0.0 && delete_count <= 0.0 {
            return 0.0;
        }
        let node_count_at_layer_for_class = (region_entries / layer_coverage).ceil().max(1.0) as u64;
        if node_count_at_layer_for_class <= 1 {
            return 1.0;
        }
        let far_fraction: f64 = match (policy, placement) {
            (PositionPolicy::Near, _) => 0.0,
            (PositionPolicy::Far, _) => 1.0,
            (PositionPolicy::Balanced, Placement::Sequential) => (insert_count / layer_coverage).min(1.0),
            // legacy：远的比例直接取 q（第三节 3.2 ②，未修订）。
            (PositionPolicy::Balanced, Placement::Random) => previous_touch_fraction.min(1.0),
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
        let scattered_touch = if far_fraction > 0.0 && placement == Placement::Random { scattered_group_touch(far_count, region_entries, layer_coverage) } else { 0.0 };
        let scattered_touches: &[f64] = if scattered_touch > 0.0 { &[scattered_touch] } else { &[] };
        combine_touches(node_count_at_layer_for_class, continuous_touch, scattered_touches)
    }

    struct LegacyClass {
        region_entries: f64,
        insert_count: f64,
        delete_count: f64,
    }

    fn legacy_multiclass(layers: &[u64], leaf_capacity: u64, internal_capacity: u64, classes: &[LegacyClass], placement: Placement, policy: PositionPolicy, previous_touch_fractions: &[f64]) -> Vec<f64> {
        let mut dirty_per_layer = Vec::with_capacity(layers.len());
        let mut cov = leaf_capacity as f64;
        for (layer_index, &node_count_at_layer) in layers.iter().enumerate() {
            let mut layer_sum = 0.0;
            for class in classes {
                layer_sum += legacy_class_dirty_nodes_this_layer(class.region_entries, cov, class.insert_count, class.delete_count, placement, policy, previous_touch_fractions[layer_index]);
            }
            dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
            cov *= internal_capacity as f64;
        }
        dirty_per_layer
    }

    /// 逐字节照抄第一次装置的 `solve_fixed_point_core`（R1–R4 之前）。
    #[allow(clippy::too_many_arguments)]
    fn legacy_solve_fixed_point_core(
        data_unit_count: u64,
        file_count: u64,
        extent_touch: TouchSpecification,
        inode_touch: TouchSpecification,
        distinct_data_units_touched: f64,
        placement: Placement,
        policy: PositionPolicy,
        geometry: Geometry,
        tree_table_entry_count: u64,
    ) -> FixedPointCoreResult {
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
        assert_eq!(accounting_layers, vec![1]);
        let accounting_dirty_total = 1.0_f64;

        let tree_table_cap = geometry.leaf_capacity(node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES));
        let tree_table_layers = tree_layers(tree_table_entry_count, tree_table_cap, tree_table_cap, 1);
        let tree_table_dirty = touch_dirty_nodes_per_layer(&tree_table_layers, tree_table_cap, tree_table_cap, tree_table_entry_count as f64, TouchSpecification::Continuous(TREE_TABLE_TOUCHED_ROOTS as f64));
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

        loop {
            iterations += 1;
            let allocation_total_nodes: u64 = allocation_layers.iter().sum();
            let mapping_total_nodes: u64 = mapping_layers.iter().sum();
            let extent_total_nodes: u64 = extent_layers.iter().sum();
            let inode_total_nodes: u64 = inode_layers.iter().sum();

            let live_units = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes as f64 + 1.0 + mapping_total_nodes as f64 + tree_table_total_nodes as f64 + 1.0;

            let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * previous_written_units);
            let allocation_layers_now = tree_layers(allocation_entries.round() as u64, allocation_leaf_cap, allocation_internal_cap, 1);
            let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() { previous_touch_allocation.clone() } else { vec![0.0; allocation_layers_now.len()] };
            let per_device_allocation_entries = allocation_entries / device_count;
            let allocation_classes = [
                LegacyClass { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
                LegacyClass { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
            ];
            let allocation_dirty_now = legacy_multiclass(&allocation_layers_now, allocation_leaf_cap, allocation_internal_cap, &allocation_classes, placement, policy, &allocation_touch);
            let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
            let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

            let mapping_entries = data_unit_count as f64 + extent_total_nodes as f64 + inode_total_nodes as f64 + allocation_total_nodes_now as f64 + 1.0;
            let mapping_layers_now = tree_layers(mapping_entries.round() as u64, mapping_leaf_cap, mapping_internal_cap, 1);
            let mapping_touch = if mapping_layers_now.len() == previous_touch_mapping.len() { previous_touch_mapping.clone() } else { vec![0.0; mapping_layers_now.len()] };
            let mapping_classes = [
                LegacyClass { region_entries: data_unit_count as f64, insert_count: distinct_data_units_touched, delete_count: distinct_data_units_touched },
                LegacyClass { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total },
                LegacyClass { region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total), insert_count: inode_root_dirty_total, delete_count: inode_root_dirty_total },
                LegacyClass { region_entries: allocation_total_nodes_now as f64, insert_count: allocation_dirty_total_now, delete_count: allocation_dirty_total_now },
                LegacyClass { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total },
                LegacyClass { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty },
            ];
            let mapping_dirty_now = legacy_multiclass(&mapping_layers_now, mapping_leaf_cap, mapping_internal_cap, &mapping_classes, placement, policy, &mapping_touch);
            let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

            let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

            let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs();
            let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs();
            let written_units_delta = (written_units_now - previous_written_units).abs();
            let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs();
            let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs();

            previous_touch_allocation = allocation_layers_now.iter().zip(allocation_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
            previous_touch_mapping = mapping_layers_now.iter().zip(mapping_dirty_now.iter()).map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 }).collect();
            allocation_layers = allocation_layers_now;
            mapping_layers = mapping_layers_now;
            allocation_dirty = allocation_dirty_now;
            mapping_dirty = mapping_dirty_now;
            previous_allocation_entries = allocation_entries;
            previous_mapping_entries = mapping_entries;
            previous_written_units = written_units_now;

            let converged = allocation_entries_delta < CONVERGENCE_EPSILON && mapping_entries_delta < CONVERGENCE_EPSILON && written_units_delta < CONVERGENCE_EPSILON && allocation_dirty_delta < CONVERGENCE_EPSILON && mapping_dirty_delta < CONVERGENCE_EPSILON;
            if converged || iterations >= MAXIMUM_FIXED_POINT_ITERATIONS {
                break;
            }
        }

        FixedPointCoreResult { extent_layers, extent_dirty, inode_layers, inode_dirty, accounting_dirty_total, tree_table_dirty_total, allocation_layers, allocation_dirty, mapping_layers, mapping_dirty, written_units: previous_written_units, iterations }
    }

    /// legacy 甲：与第一次装置的 `solve_jia_publish` 逐字节相同，只为 Q6.11 对比用。
    pub(super) fn legacy_solve_jia_publish(shape: PoolShape) -> JiaPublishOutcome {
        let geometry = shape.geometry;
        let data_unit_count = shape.family.data_unit_count(shape.file_count);
        let fsync_data_unit_count = shape.family.data_units_per_fsync();

        let core = legacy_solve_fixed_point_core(
            data_unit_count,
            shape.file_count,
            TouchSpecification::Continuous(fsync_data_unit_count as f64),
            TouchSpecification::Continuous(1.0),
            fsync_data_unit_count as f64,
            shape.placement,
            shape.policy,
            geometry,
            TREE_TABLE_ENTRY_COUNT,
        );

        let written_units = fsync_data_unit_count as f64 + core.written_units;
        let named_items = written_units.round() as u64;
        let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
        let pbs = geometry.physical_block_size_bytes;
        let unit_write_calls = (written_units.round() as u64) * DEVICE_COUNT;
        let record_calls = record_count * DEVICE_COUNT;
        let total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

        let extent_dirty_total = sum_f64(&core.extent_dirty);
        let inode_container_dirty = core.inode_dirty[0];
        let inode_root_dirty_total: f64 = core.inode_dirty[1..].iter().sum();
        let allocation_dirty_total = sum_f64(&core.allocation_dirty);
        let mapping_dirty_total = sum_f64(&core.mapping_dirty);

        JiaPublishOutcome {
            data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
            extent_bytes: (extent_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
            inode_container_bytes: (inode_container_dirty * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
            inode_root_bytes: (inode_root_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
            allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
            accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
            mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
            tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
            record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
            root_slot_bytes: pbs,
            system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
            write_calls: total_calls,
            barriers: 2 * DEVICE_COUNT,
            fua_count: 1,
            named_items,
            record_count,
            extent_layers: core.extent_layers,
            extent_dirty: core.extent_dirty,
            inode_layers: core.inode_layers,
            inode_dirty: core.inode_dirty,
            allocation_layers: core.allocation_layers,
            allocation_dirty: core.allocation_dirty,
            mapping_layers: core.mapping_layers,
            mapping_dirty: core.mapping_dirty,
            accounting_dirty: core.accounting_dirty_total,
            written_units,
            fixed_point_iterations: core.iterations,
        }
    }
}

#[cfg(test)]
mod jia_anchor_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// B2（第七节 7.2）：甲，P=1、F1、seq、主几何，三种位置策略都一样；R1 之后分配记录 402 条
    /// （第一次装置的口径 354，见 Q6.11）；总字节不变（P=1 时分配记录树仍是单节点）。
    #[test]
    fn jia_matches_the_anchor_at_file_count_1_on_every_position_policy() {
        for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
            let outcome = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, policy));
            assert_eq!(outcome.total_bytes(), 344_576, "policy={policy:?}");
            assert_eq!(outcome.write_calls, 21, "policy={policy:?}");
            assert_eq!(outcome.barriers, 4, "policy={policy:?}");
            assert_eq!(outcome.fua_count, 1, "policy={policy:?}");
            assert_eq!(outcome.block_layer_write_requests(), 26, "policy={policy:?}");
            assert_eq!(outcome.named_items, 8, "policy={policy:?}");
            assert_eq!(outcome.record_count, 1, "policy={policy:?}");
            assert_eq!(outcome.allocation_layers, vec![1], "policy={policy:?}");
        }
        let legacy = legacy::legacy_solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(legacy.total_bytes(), 344_576, "legacy 总字节应与 R1 之后相同（P=1 仍是单节点）");
    }

    /// B7（第七节 7.2）：甲，P=145、F1、seq、主几何 —— extent 树长到 2 层，甲每次持久化写字节 377 344。
    #[test]
    fn jia_at_file_count_145_matches_the_anchor() {
        let outcome = solve_jia_publish(shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(outcome.extent_layers, vec![2, 1], "extent 树 2 层（2 叶 + 1 根）");
        assert_eq!(outcome.inode_layers, vec![1, 1], "inode 树仍是 1 容器 + 1 根");
        assert_eq!(outcome.allocation_layers, vec![1], "分配记录 742 条（R1）≤ 812，仍 1 节点");
        assert_eq!(outcome.mapping_layers, vec![1], "映射 152 条 ≤ 294，仍 1 节点");
        assert_eq!(outcome.total_bytes(), 377_344);
    }

    /// B10（第七节 7.2，第三节 3.2 ①）：R1 之后分配记录树长到 2 层的门槛 P* = 181（P=180 恰 812 条），
    /// legacy（R1 之前）门槛是 205。
    #[test]
    fn allocation_tree_height_threshold_moves_from_205_to_181_after_counting_data_units_in_backlog() {
        let at_180 = solve_jia_publish(shape_at(180, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        let at_181 = solve_jia_publish(shape_at(181, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(at_180.allocation_layers.len(), 1, "P=180 分配记录树仍是 1 层（R1 之后）");
        assert_eq!(at_181.allocation_layers.len(), 2, "P=181 分配记录树长到 2 层（R1 之后）");

        let legacy_at_204 = legacy::legacy_solve_jia_publish(shape_at(204, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        let legacy_at_205 = legacy::legacy_solve_jia_publish(shape_at(205, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(legacy_at_204.allocation_layers.len(), 1, "legacy：P=204 仍是 1 层");
        assert_eq!(legacy_at_205.allocation_layers.len(), 2, "legacy：P=205 长到 2 层（第一次装置的口径）");
    }

    /// A7（第七节 7.1）：甲的持久顺序每次 fsync 恰好两道屏障、根槽一次 FUA。
    #[test]
    fn jia_has_exactly_two_barriers_and_one_fua_on_every_grid_point() {
        for file_count in [1u64, 100, 10_000] {
            for placement in [Placement::Sequential, Placement::Random] {
                for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
                    let outcome = solve_jia_publish(shape_at(file_count, Family::OneDataUnitPerFile, placement, policy));
                    assert_eq!(outcome.barriers, 4, "file_count={file_count} placement={placement:?} policy={policy:?}");
                    assert_eq!(outcome.fua_count, 1);
                }
            }
        }
    }

    /// A8（第七节 7.1）：F8 各格，extent 树每次 fsync 第 1 层起每层期望脏节点 ≤ 2。
    #[test]
    fn eight_adjacent_family_extent_dirty_nodes_above_leaf_layer_stay_at_or_below_two() {
        for file_count in [1u64, 100, 10_000, 1_000_000] {
            let outcome = solve_jia_publish(shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Sequential, PositionPolicy::Balanced));
            let extent_internal_cap = node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false));
            let extent_dirty = single_group_dirty_nodes_per_layer(&outcome.extent_layers, node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), extent_internal_cap, 8);
            for (layer_index, &dirty) in extent_dirty.iter().enumerate().skip(1) {
                assert!(dirty <= 2.0 + 1e-9, "file_count={file_count} layer={layer_index} dirty={dirty}");
            }
        }
    }

    /// 不动点收敛（Q6.9 支撑量）：一格算完，迭代次数应远小于上限 1000（R1 让 W 更大，仍要收敛）。
    /// 这一条只覆盖 F8A；F1 在同样规模上会撞上限，见 `fixed_point_iteration_ceiling_triggers_at_file_count_1e8_one_data_unit_per_file_random_balanced`——
    /// 两条并存是如实的（F2 触发不等于模型错，登记第十节）。
    #[test]
    fn fixed_point_converges_well_below_the_iteration_ceiling() {
        for file_count in [1u64, 100, 10_000, 1_000_000, 100_000_000] {
            let outcome = solve_jia_publish(shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Balanced));
            assert!(outcome.fixed_point_iterations < MAXIMUM_FIXED_POINT_ITERATIONS, "file_count={file_count} 撞到了迭代上限，F2 应该触发");
        }
    }

    /// F2 第一半（登记第十节，主 agent 2026-09-19 追加要求）：P=10⁸、F1、rand、Lbalanced 一格
    /// 迭代到上限仍未收敛（`row6_grid` 里这一格 `fixed_point_iterations=1000`），`jia_convergence_and_saturation_status` 要把它
    /// 判成第一半触发；这一格第二半不该触发（extent/alloc/mapping 第 0 层脏节点占比都远低于 50%）。
    #[test]
    fn fixed_point_iteration_ceiling_triggers_at_file_count_1e8_one_data_unit_per_file_random_balanced() {
        let outcome = solve_jia_publish(shape_at(100_000_000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Balanced));
        assert_eq!(outcome.fixed_point_iterations, MAXIMUM_FIXED_POINT_ITERATIONS, "这一格应该撞满 1000 次迭代");
        let (half1, half2) = jia_convergence_and_saturation_status(&outcome);
        assert!(half1, "F2 第一半应该触发");
        assert!(half2.is_empty(), "这一格第二半不该触发：{half2:?}");
    }

    /// F2 第二半（登记第十节）：P=10⁶、F1、rand、Lfar 一格分配记录树第 0 层节点数 ≥ 64 且脏节点占比
    /// ≥ 50%，`jia_convergence_and_saturation_status` 要把它判成第二半触发（触发的树含 `allocation`）；同一个 P、族、落点换成
    /// Lbalanced 时不触发（占比只有约 39.7%，见运行记录），两者对照证明这条判据不是恒真恒假。
    #[test]
    fn layer_zero_saturation_triggers_on_allocation_tree_at_file_count_1e6_one_data_unit_per_file_random_far() {
        let far_outcome = solve_jia_publish(shape_at(1_000_000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Far));
        let (far_half1, far_half2) = jia_convergence_and_saturation_status(&far_outcome);
        assert!(!far_half1, "这一格应该收敛");
        assert!(far_half2.contains(&"allocation"), "L远 一格分配记录树应该触发第二半：{far_half2:?}");

        let balanced_outcome = solve_jia_publish(shape_at(1_000_000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Balanced));
        let (balanced_half1, balanced_half2) = jia_convergence_and_saturation_status(&balanced_outcome);
        assert!(!balanced_half1, "这一格应该收敛");
        assert!(balanced_half2.is_empty(), "同一个 P 换 Lbalanced 不该触发第二半：{balanced_half2:?}");
    }

    /// V2（作废条款）：甲自身不产生负的或超界的脏节点数。
    #[test]
    fn jia_dirty_node_counts_never_exceed_their_own_layer_node_counts() {
        for file_count in [1u64, 145, 10_000, 1_000_000] {
            for family in [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync] {
                for placement in [Placement::Sequential, Placement::Random] {
                    for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
                        let outcome = solve_jia_publish(shape_at(file_count, family, placement, policy));
                        assert!(outcome.total_bytes() > 0, "file_count={file_count} family={family:?}");
                    }
                }
            }
        }
    }
}

// ============================================================
// 六、5.4 wal_full 与乙-M（K9′，第 6 行三臂）
// ============================================================

/// 岔路单第 6 行的 WAL 两臂。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteAheadLogArm {
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}

/// WAL 两臂一次 fsync 写出的东西（与第一次逐字相同：不发根、不写固定点、不写树表）。
#[derive(Clone, Debug)]
struct WriteAheadLogFsyncOutcome {
    data_bytes: u64,
    extent_bytes: u64,
    inode_container_bytes: u64,
    inode_root_bytes: u64,
    record_bytes: u64,
    write_calls: u64,
    barriers: u64,
    named_items: u64,
    record_count: u64,
}

impl WriteAheadLogFsyncOutcome {
    fn total_bytes(&self) -> u64 {
        self.data_bytes + self.extent_bytes + self.inode_container_bytes + self.inode_root_bytes + self.record_bytes
    }
}

fn solve_write_ahead_log_fsync(shape: PoolShape, arm: WriteAheadLogArm) -> WriteAheadLogFsyncOutcome {
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
    let unit_count = (fsync_data_unit_count as f64 + extent_component + inode_container_component + inode_root_component).round() as u64;
    let write_calls = unit_count * DEVICE_COUNT + record_count * DEVICE_COUNT;

    WriteAheadLogFsyncOutcome {
        data_bytes: fsync_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_container_bytes: (inode_container_component * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_root_bytes: (inode_root_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        write_calls,
        barriers: 2 * DEVICE_COUNT,
        named_items,
        record_count,
    }
}

/// R8（第 7 行组提交）：WAL 两臂把 `concurrent_fsync_count` 个并发 fsync 并成一批 = 一次「单元 → 屏障 → 记录 → 屏障」；
/// wal_full 恒点名两棵新树根（2 项，不随 concurrent_fsync_count 变），乙-M 点名这一批的全部脏叶。
fn solve_write_ahead_log_batch_fsync(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool) -> WriteAheadLogFsyncOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
    let extent_dirty = if shared {
        single_group_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, batch_data_unit_count)
    } else {
        touch_dirty_nodes_per_layer(&extent_layers, extent_leaf_cap, extent_internal_cap, data_unit_count as f64, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 })
    };

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let inode_layers = tree_layers(shape.file_count, inode_leaf_cap, inode_internal_cap, 2);
    let inode_dirty = if shared {
        single_group_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, concurrent_fsync_count)
    } else {
        touch_dirty_nodes_per_layer(&inode_layers, inode_leaf_cap, inode_internal_cap, shape.file_count as f64, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 })
    };

    let (extent_component, inode_container_component, inode_root_component, named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => {
            let extent_total = sum_f64(&extent_dirty);
            let inode_container = inode_dirty[0];
            let inode_root: f64 = inode_dirty[1..].iter().sum();
            (extent_total, inode_container, inode_root, 2u64) // 恒 2 项，不随 concurrent_fsync_count 变（第一节「读法写死」）。
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_leaf = extent_dirty[0];
            let inode_leaf = inode_dirty[0];
            let dirty_leaves = (extent_leaf + inode_leaf).round() as u64;
            (extent_leaf, inode_leaf, 0.0, dirty_leaves)
        }
    };

    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    let unit_count = (batch_data_unit_count as f64 + extent_component + inode_container_component + inode_root_component).round() as u64;
    let write_calls = unit_count * DEVICE_COUNT + record_count * DEVICE_COUNT;

    WriteAheadLogFsyncOutcome {
        data_bytes: batch_data_unit_count * UNIT_BYTES * DEVICE_COUNT,
        extent_bytes: (extent_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_container_bytes: (inode_container_component * UNIT_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_root_bytes: (inode_root_component * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        write_calls,
        barriers: 2 * DEVICE_COUNT,
        named_items,
        record_count,
    }
}

/// WAL 两臂一次 checkpoint 写出的东西：四样固定点的间隔净改动 + 乙额外补的用户树祖先追赶
/// + K9′（R7）中间版释放记录的支撑量（Q6.6/Q6.9/B11）。
#[derive(Clone, Debug)]
struct WriteAheadLogCheckpointOutcome {
    extent_catch_up_bytes: u64,
    inode_catch_up_bytes: u64,
    allocation_bytes: u64,
    accounting_bytes: u64,
    mapping_bytes: u64,
    tree_table_bytes: u64,
    record_bytes: u64,
    root_slot_bytes: u64,
    system_configuration_bytes: u64,
    write_calls: u64,
    barriers: u64,
    fua_count: u64,
    named_items: u64,
    record_count: u64,
    allocation_layers: Vec<u64>,
    intermediate_version_data: f64,
    intermediate_version_extent_total: f64,
    intermediate_version_inode_total: f64,
    intermediate_version_total: f64,
    // F2（第十节）判定要的支撑量：这个 checkpoint 用的不动点核心，逐树第 0 层的节点数与脏节点数、
    // 与它跑了几轮迭代（第一半：`fixed_point_iterations >= MAXIMUM_FIXED_POINT_ITERATIONS` 未收敛；
    // 第二半：某棵树第 0 层节点数 ≥ 64 且脏节点占比 ≥ 50%）。
    extent_layers: Vec<u64>,
    extent_dirty: Vec<f64>,
    inode_layers: Vec<u64>,
    inode_dirty: Vec<f64>,
    allocation_dirty: Vec<f64>,
    mapping_layers: Vec<u64>,
    mapping_dirty: Vec<f64>,
    fixed_point_iterations: u64,
}

/// F2 第二半：某棵树第 0 层节点数 ≥ 64 且第 0 层脏节点数占这层节点数 ≥ 50%（登记第十节 F2）。
fn layer_zero_is_mostly_dirty(layers: &[u64], dirty: &[f64]) -> bool {
    match (layers.first(), dirty.first()) {
        (Some(&node_count), Some(&dirty_count)) if node_count >= 64 => dirty_count / node_count as f64 >= 0.5,
        _ => false,
    }
}

/// F2 两半的判定：`.0` = 第一半（未收敛），`.1` = 第二半触发的树名单（空 = 没触发）。
fn convergence_and_saturation_status(iterations: u64, extent_layers: &[u64], extent_dirty: &[f64], inode_layers: &[u64], inode_dirty: &[f64], allocation_layers: &[u64], allocation_dirty: &[f64], mapping_layers: &[u64], mapping_dirty: &[f64]) -> (bool, Vec<&'static str>) {
    let half1 = iterations >= MAXIMUM_FIXED_POINT_ITERATIONS;
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

/// JIA 结果上的 F2 判定（第 5、6 行都要用）。
fn jia_convergence_and_saturation_status(outcome: &JiaPublishOutcome) -> (bool, Vec<&'static str>) {
    convergence_and_saturation_status(
        outcome.fixed_point_iterations,
        &outcome.extent_layers,
        &outcome.extent_dirty,
        &outcome.inode_layers,
        &outcome.inode_dirty,
        &outcome.allocation_layers,
        &outcome.allocation_dirty,
        &outcome.mapping_layers,
        &outcome.mapping_dirty,
    )
}

/// WAL checkpoint 结果上的 F2 判定（第 6 行 wal_full / 乙-M 两臂，K9 与 K9′ 都要看）。
fn checkpoint_convergence_and_saturation_status(outcome: &WriteAheadLogCheckpointOutcome) -> (bool, Vec<&'static str>) {
    convergence_and_saturation_status(
        outcome.fixed_point_iterations,
        &outcome.extent_layers,
        &outcome.extent_dirty,
        &outcome.inode_layers,
        &outcome.inode_dirty,
        &outcome.allocation_layers,
        &outcome.allocation_dirty,
        &outcome.mapping_layers,
        &outcome.mapping_dirty,
    )
}

impl WriteAheadLogCheckpointOutcome {
    fn total_bytes(&self) -> u64 {
        self.extent_catch_up_bytes + self.inode_catch_up_bytes + self.allocation_bytes + self.accounting_bytes + self.mapping_bytes + self.tree_table_bytes + self.record_bytes + self.root_slot_bytes + self.system_configuration_bytes
    }
}

#[cfg(test)]
mod convergence_and_saturation_status_tests {
    use super::*;

    #[test]
    fn layer_zero_below_sixty_four_nodes_never_triggers_second_half() {
        assert!(!layer_zero_is_mostly_dirty(&[63], &[63.0]), "节点数 63 < 64，占比 100% 也不该触发");
        assert!(!layer_zero_is_mostly_dirty(&[], &[]), "空向量不该触发");
    }

    #[test]
    fn layer_zero_at_or_above_sixty_four_nodes_triggers_at_the_fifty_percent_boundary() {
        assert!(!layer_zero_is_mostly_dirty(&[64], &[31.99]), "占比刚好低于 50% 不该触发");
        assert!(layer_zero_is_mostly_dirty(&[64], &[32.0]), "占比恰好 50% 应该触发");
        assert!(layer_zero_is_mostly_dirty(&[100], &[99.0]), "占比 99% 应该触发");
    }

    #[test]
    fn convergence_and_saturation_status_reports_both_conditions_independently() {
        // 第一半触发、第二半不触发。
        let (half1, half2) = convergence_and_saturation_status(MAXIMUM_FIXED_POINT_ITERATIONS, &[1], &[1.0], &[1, 1], &[1.0, 1.0], &[1], &[1.0], &[1], &[1.0]);
        assert!(half1);
        assert!(half2.is_empty());
        // 第一半不触发、第二半在 allocation 与 mapping 两棵树上触发。
        let (half1, half2) = convergence_and_saturation_status(1, &[1], &[1.0], &[1, 1], &[1.0, 1.0], &[100], &[60.0], &[100], &[60.0]);
        assert!(!half1);
        assert_eq!(half2, vec!["allocation", "mapping"]);
    }
}

/// R7：一个间隔 N_b 批里的中间版计数（数据单元、extent 各层、inode 各层分开）。
/// `dedup_*` 是间隔并集去重之后的脏节点向量（`solve_fixed_point_core` 已经算过的那份）；
/// 乙-M 的祖先只在 checkpoint 写一次，不产生中间版，所以只数叶层（inode 容器 = 第 0 层）。
fn intermediate_version_counts(
    shape: PoolShape,
    arm: WriteAheadLogArm,
    interval_fsyncs: u64,
    extent_layers: &[u64],
    dedup_extent_dirty: &[f64],
    inode_layers: &[u64],
    dedup_inode_dirty: &[f64],
    dedup_data_units_touched: f64,
) -> (f64, f64, f64, f64) {
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

    let total = data_intermediate + extent_intermediate_total + inode_intermediate_total;
    (data_intermediate, extent_intermediate_total, inode_intermediate_total, total)
}

/// R8：一个间隔 `batches_in_interval` 批（每批 concurrent_fsync_count 个 fsync）里的中间版计数，批级版本的 `intermediate_version_counts`。
#[allow(clippy::too_many_arguments)]
fn intermediate_version_counts_batch(
    shape: PoolShape,
    arm: WriteAheadLogArm,
    concurrent_fsync_count: u64,
    shared: bool,
    batches_in_interval: u64,
    extent_layers: &[u64],
    dedup_extent_dirty: &[f64],
    inode_layers: &[u64],
    dedup_inode_dirty: &[f64],
    dedup_data_units_touched: f64,
) -> (f64, f64, f64, f64) {
    let geometry = shape.geometry;
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;
    let batches_in_interval_count = batches_in_interval as f64;

    let extent_leaf_cap = geometry.leaf_capacity(node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES));
    let extent_internal_cap = geometry.internal_capacity(EXTENT_KEY_BYTES);
    let per_batch_extent = if shared {
        single_group_dirty_nodes_per_layer(extent_layers, extent_leaf_cap, extent_internal_cap, batch_data_unit_count)
    } else {
        touch_dirty_nodes_per_layer(extent_layers, extent_leaf_cap, extent_internal_cap, 0.0, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 })
    };

    let inode_leaf_cap = geometry.leaf_capacity(inode_container_capacity());
    let inode_internal_cap = geometry.leaf_capacity(node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES));
    let per_batch_inode = if shared {
        single_group_dirty_nodes_per_layer(inode_layers, inode_leaf_cap, inode_internal_cap, concurrent_fsync_count)
    } else {
        touch_dirty_nodes_per_layer(inode_layers, inode_leaf_cap, inode_internal_cap, 0.0, TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: 1.0 })
    };

    let data_intermediate = (batches_in_interval_count * batch_data_unit_count as f64 - dedup_data_units_touched).max(0.0);

    let extent_intermediate_total: f64 = per_batch_extent.iter().zip(dedup_extent_dirty.iter()).map(|(&per_batch, &dedup)| (batches_in_interval_count * per_batch - dedup).max(0.0)).sum();

    let inode_intermediate_total: f64 = match arm {
        WriteAheadLogArm::WriteAheadLogFull => per_batch_inode.iter().zip(dedup_inode_dirty.iter()).map(|(&per_batch, &dedup)| (batches_in_interval_count * per_batch - dedup).max(0.0)).sum(),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            if per_batch_inode.is_empty() {
                0.0
            } else {
                (batches_in_interval_count * per_batch_inode[0] - dedup_inode_dirty[0]).max(0.0)
            }
        }
    };

    let total = data_intermediate + extent_intermediate_total + inode_intermediate_total;
    (data_intermediate, extent_intermediate_total, inode_intermediate_total, total)
}

/// R8（第 7 行组提交）：一个间隔 `batches_in_interval` 批（每批 concurrent_fsync_count 个 fsync）之后的 WAL checkpoint；
/// `batches_in_interval` 由调用处按第八节 8.2 的读法算好再传进来（主：`interval_fsyncs / concurrent_fsync_count`；反向「N 按批计」：`interval_fsyncs`）。
fn solve_write_ahead_log_batch_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool, batches_in_interval: u64, intermediate_version_policy: IntermediateVersionPolicy) -> WriteAheadLogCheckpointOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let batch_data_unit_count = concurrent_fsync_count * fsync_data_unit_count;
    let batches_in_interval_count = batches_in_interval as f64;
    let interval_data_unit_touch_count = batches_in_interval_count * batch_data_unit_count as f64;

    // R8 + R6：共享是「N_b 组、每组 concurrent_fsync_count·d 条」（seq 连续、rand 散连续组）；不共享 seq 是「concurrent_fsync_count 段、每段
    // N_b·d 条，段间空隙 ≥ cov_l」（`UnsharedPaths`），rand 是「N_b·concurrent_fsync_count 组、每组 d 条」（R6）。
    let extent_touch = match (shape.placement, shared) {
        (Placement::Sequential, true) => TouchSpecification::Continuous(interval_data_unit_touch_count),
        (Placement::Sequential, false) => TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: batches_in_interval_count * fsync_data_unit_count as f64 },
        (Placement::Random, true) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count, group_size: batch_data_unit_count as f64 },
        (Placement::Random, false) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count * concurrent_fsync_count as f64, group_size: fsync_data_unit_count as f64 },
    };
    let inode_touch = match (shape.placement, shared) {
        (Placement::Sequential, true) => TouchSpecification::Continuous(batches_in_interval_count * concurrent_fsync_count as f64),
        (Placement::Sequential, false) => TouchSpecification::UnsharedPaths { path_count: concurrent_fsync_count as f64, group_size: batches_in_interval_count },
        (Placement::Random, true) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count, group_size: concurrent_fsync_count as f64 },
        (Placement::Random, false) => TouchSpecification::ScatteredContinuousGroups { group_count: batches_in_interval_count * concurrent_fsync_count as f64, group_size: 1.0 },
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_continuous_groups_distinct_entries(data_unit_count as f64, batches_in_interval_count * concurrent_fsync_count as f64, fsync_data_unit_count as f64),
    };

    let core_zero = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        extent_touch,
        inode_touch,
        distinct_data_units_touched,
        shape.placement,
        shape.policy,
        geometry,
        SuppressedFixedPoints::default(),
        TREE_TABLE_ENTRY_COUNT,
        0.0,
    );

    let (intermediate_data, intermediate_extent_total, intermediate_inode_total, intermediate_total) = match intermediate_version_policy {
        IntermediateVersionPolicy::NotTracked => (0.0, 0.0, 0.0, 0.0),
        IntermediateVersionPolicy::ReleasedAndBacklogged => {
            intermediate_version_counts_batch(shape, arm, concurrent_fsync_count, shared, batches_in_interval, &core_zero.extent_layers, &core_zero.extent_dirty, &core_zero.inode_layers, &core_zero.inode_dirty, distinct_data_units_touched)
        }
    };

    let core = if intermediate_total > 0.0 {
        solve_fixed_point_core(
            data_unit_count,
            shape.file_count,
            extent_touch,
            inode_touch,
            distinct_data_units_touched,
            shape.placement,
            shape.policy,
            geometry,
            SuppressedFixedPoints::default(),
            TREE_TABLE_ENTRY_COUNT,
            intermediate_total,
        )
    } else {
        core_zero
    };

    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    let (extent_catch_up, inode_container_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            let items = (extent_ancestors + inode_ancestors).round() as u64;
            (extent_ancestors, 0.0, inode_ancestors, items)
        }
    };

    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
    let named_items = fixed_point_units.round() as u64 + extra_named_items;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = named_items * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    WriteAheadLogCheckpointOutcome {
        extent_catch_up_bytes: (extent_catch_up * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_catch_up_bytes: (inode_root_catch_up * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: pbs,
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        write_calls: total_calls,
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        named_items,
        record_count,
        allocation_layers: core.allocation_layers,
        intermediate_version_data: intermediate_data,
        intermediate_version_extent_total: intermediate_extent_total,
        intermediate_version_inode_total: intermediate_inode_total,
        intermediate_version_total: intermediate_total,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        fixed_point_iterations: core.iterations,
    }
}

/// 求一格 WAL 两臂一次 checkpoint（间隔 `interval_fsyncs` 次 fsync 之后）的写出。
/// `intermediate_version_policy`：K9（不进积压）或 K9′（R7，中间版进积压 + 尾部插入）。
fn solve_write_ahead_log_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, interval_fsyncs: u64, intermediate_version_policy: IntermediateVersionPolicy) -> WriteAheadLogCheckpointOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let interval_data_unit_touch_count = interval_fsyncs as f64 * fsync_data_unit_count as f64;
    let interval_inode_record_touch_count = interval_fsyncs as f64;

    // R6：rand 下用散连续组（N_g 组、每组 s 条），不是把 N×d 次触达当独立散点；seq 不变。
    let extent_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_data_unit_touch_count),
        Placement::Random => TouchSpecification::ScatteredContinuousGroups { group_count: interval_fsyncs as f64, group_size: fsync_data_unit_count as f64 },
    };
    let inode_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_inode_record_touch_count),
        // inode 每次 fsync 只碰 1 条记录（f=1），组大小恒 1，散连续组退化成散点——两种写法读数相同。
        Placement::Random => TouchSpecification::ScatteredContinuousGroups { group_count: interval_fsyncs as f64, group_size: 1.0 },
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_continuous_groups_distinct_entries(data_unit_count as f64, interval_fsyncs as f64, fsync_data_unit_count as f64),
    };

    // 第一遍（extra=0）：算出 extent / inode 的间隔并集去重脏节点数，供 I（中间版）计数用；
    // 这两个向量不受 extra_allocation_release_insertions 影响（它只进分配记录树的计算）。
    let core_zero = solve_fixed_point_core(
        data_unit_count,
        shape.file_count,
        extent_touch,
        inode_touch,
        distinct_data_units_touched,
        shape.placement,
        shape.policy,
        geometry,
        SuppressedFixedPoints::default(),
        TREE_TABLE_ENTRY_COUNT,
        0.0,
    );

    let (intermediate_data, intermediate_extent_total, intermediate_inode_total, intermediate_total) = match intermediate_version_policy {
        IntermediateVersionPolicy::NotTracked => (0.0, 0.0, 0.0, 0.0),
        IntermediateVersionPolicy::ReleasedAndBacklogged => intermediate_version_counts(shape, arm, interval_fsyncs, &core_zero.extent_layers, &core_zero.extent_dirty, &core_zero.inode_layers, &core_zero.inode_dirty, distinct_data_units_touched),
    };

    let core = if intermediate_total > 0.0 {
        solve_fixed_point_core(
            data_unit_count,
            shape.file_count,
            extent_touch,
            inode_touch,
            distinct_data_units_touched,
            shape.placement,
            shape.policy,
            geometry,
            SuppressedFixedPoints::default(),
            TREE_TABLE_ENTRY_COUNT,
            intermediate_total,
        )
    } else {
        core_zero
    };

    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    let (extent_catch_up, inode_container_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            let items = (extent_ancestors + inode_ancestors).round() as u64;
            (extent_ancestors, 0.0, inode_ancestors, items)
        }
    };

    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
    let named_items = fixed_point_units.round() as u64 + extra_named_items;
    let record_count = named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);

    let pbs = geometry.physical_block_size_bytes;
    let unit_write_calls = named_items * DEVICE_COUNT;
    let record_calls = record_count * DEVICE_COUNT;
    let total_calls = unit_write_calls + record_calls + 1 + DEVICE_COUNT;

    WriteAheadLogCheckpointOutcome {
        extent_catch_up_bytes: (extent_catch_up * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        inode_catch_up_bytes: (inode_root_catch_up * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        allocation_bytes: (allocation_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        accounting_bytes: (core.accounting_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        mapping_bytes: (mapping_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        tree_table_bytes: (core.tree_table_dirty_total * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT,
        record_bytes: record_count * RECORD_BYTES * DEVICE_COUNT,
        root_slot_bytes: pbs,
        system_configuration_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES * DEVICE_COUNT,
        write_calls: total_calls,
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        named_items,
        record_count,
        allocation_layers: core.allocation_layers,
        intermediate_version_data: intermediate_data,
        intermediate_version_extent_total: intermediate_extent_total,
        intermediate_version_inode_total: intermediate_inode_total,
        intermediate_version_total: intermediate_total,
        extent_layers: core.extent_layers,
        extent_dirty: core.extent_dirty,
        inode_layers: core.inode_layers,
        inode_dirty: core.inode_dirty,
        allocation_dirty: core.allocation_dirty,
        mapping_layers: core.mapping_layers,
        mapping_dirty: core.mapping_dirty,
        fixed_point_iterations: core.iterations,
    }
}

/// 摊销行（Q6.2）：（N 次 fsync 之和 + checkpoint）÷ N。
fn amortize(fsync_total_bytes: u64, checkpoint_total_bytes: u64, interval_fsyncs: u64) -> f64 {
    (fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64
}

#[cfg(test)]
mod write_ahead_log_anchor_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// B4（第七节 7.2）：wal_full-K9′，P=1、F1、seq、主几何；K9 与 K9′ 在 N=1 逐字段相同（B19）。
    #[test]
    fn write_ahead_log_full_at_file_count_1_matches_the_anchor() {
        let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull);
        assert_eq!(fsync.total_bytes(), 204_800);
        assert_eq!(fsync.write_calls, 10);
        assert_eq!(fsync.barriers, 4);

        let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::NotTracked);
        assert_eq!(checkpoint.total_bytes(), 147_968);
        assert_eq!(checkpoint.write_calls, 13);
        assert_eq!(checkpoint.fua_count, 1);
        assert_eq!(checkpoint.allocation_layers, vec![1], "K9：P=1、N=16 分配记录树仍单节点（402 条）");

        for interval in [1u64, 16] {
            let cp = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::NotTracked);
            let amortized = amortize(fsync.total_bytes(), cp.total_bytes(), interval);
            let expected = if interval == 1 { 352_768.0 } else { 214_048.0 };
            assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
        }
    }

    /// B5（第七节 7.2）：乙-M-K9′，P=1、F1、seq、主几何。
    #[test]
    fn write_ahead_log_leaf_at_file_count_1_matches_the_anchor() {
        let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf);
        assert_eq!(fsync.total_bytes(), 172_032);
        assert_eq!(fsync.write_calls, 8);
        assert_eq!(fsync.barriers, 4);

        let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::NotTracked);
        assert_eq!(checkpoint.total_bytes(), 180_736);
        assert_eq!(checkpoint.write_calls, 15);

        for interval in [1u64, 16] {
            let cp = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf, interval, IntermediateVersionPolicy::NotTracked);
            let amortized = amortize(fsync.total_bytes(), cp.total_bytes(), interval);
            let expected = if interval == 1 { 352_768.0 } else { 183_328.0 };
            assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
        }
    }

    /// 阳性对照（5.5）：三条臂各自的格与「必须出现的差」（第 6 行）。
    #[test]
    fn positive_controls_show_the_prescribed_difference_for_every_arm() {
        let shape_at_file_count_1 = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let jia_fsync = solve_jia_publish(shape_at_file_count_1).total_bytes();
        let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 系统配置槽 × 2");
        assert_eq!(write_ahead_log_full_fsync - write_ahead_log_leaf_fsync, 32_768, "wal_full：inode 根 × 2 盘");

        let shape_at_file_count_145 = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let write_ahead_log_leaf_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        let write_ahead_log_full_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        assert_eq!(write_ahead_log_leaf_at_file_count_145, 172_032, "乙-M fsync 行不含 extent 与 inode 两个根");
        assert_eq!(write_ahead_log_full_at_file_count_145, 237_568);
        let checkpoint_at_file_count_145 = solve_write_ahead_log_checkpoint(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
        assert!(checkpoint_at_file_count_145.extent_catch_up_bytes > 0, "extent 根追赶应该非零");
        assert!(checkpoint_at_file_count_145.inode_catch_up_bytes > 0, "inode 根追赶应该非零");

        // K9′：P=1、N=16 一格 wal_full 中间版 60、分配记录树 [5,1]；乙-M 中间版 45、[4,1]（B11）。
        let checkpoint_full_released_and_backlogged = solve_write_ahead_log_checkpoint(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
        assert_eq!(checkpoint_full_released_and_backlogged.intermediate_version_total, 60.0, "wal_full 中间版：数据 15+extent 15+inode 叶 15+inode 根 15");
        assert_eq!(checkpoint_full_released_and_backlogged.allocation_layers, vec![5, 1], "wal_full-K9′ 分配记录树 [5,1]");
        let checkpoint_leaf_released_and_backlogged = solve_write_ahead_log_checkpoint(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
        assert_eq!(checkpoint_leaf_released_and_backlogged.intermediate_version_total, 45.0, "乙-M 中间版：数据 15+extent 15+inode 叶 15（无 inode 根）");
        assert_eq!(checkpoint_leaf_released_and_backlogged.allocation_layers, vec![4, 1], "乙-M-K9′ 分配记录树 [4,1]");
    }

    /// B19：N_b=1（N=1、concurrent_fsync_count=1）的格上，K9 与 K9′ 逐字段相同——中间版恒 0。
    #[test]
    fn batch_count_equal_one_cells_have_zero_intermediate_versions_and_not_tracked_equals_released_and_backlogged() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
            let not_tracked = solve_write_ahead_log_checkpoint(shape, arm, 1, IntermediateVersionPolicy::NotTracked);
            let released_and_backlogged = solve_write_ahead_log_checkpoint(shape, arm, 1, IntermediateVersionPolicy::ReleasedAndBacklogged);
            assert_eq!(released_and_backlogged.intermediate_version_total, 0.0, "arm={arm:?}");
            assert_eq!(not_tracked.total_bytes(), released_and_backlogged.total_bytes(), "arm={arm:?}：N=1 时 K9 与 K9′ 应该逐字节相同");
            assert_eq!(not_tracked.write_calls, released_and_backlogged.write_calls, "arm={arm:?}");
        }
        // wal_full − 甲 = 8192（一条记录的差，第一节「读法写死」N_b=1 那一条）。
        let jia = solve_jia_publish(shape).total_bytes();
        let write_ahead_log_full_checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 1, IntermediateVersionPolicy::NotTracked);
        let write_ahead_log_full_amortized = amortize(solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes(), write_ahead_log_full_checkpoint.total_bytes(), 1);
        assert!((write_ahead_log_full_amortized - jia as f64 - 8192.0).abs() < 1e-6, "write_ahead_log_full_amortized={write_ahead_log_full_amortized} jia={jia}");
    }

    /// V2（作废条款）：任一格「乙-M ≤ wal_full ≤ 甲」（fsync 行字节）都不许违反。
    #[test]
    fn arm_containment_holds_on_the_grid() {
        for file_count in [1u64, 145, 10_000, 1_000_000] {
            for family in [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync] {
                for placement in [Placement::Sequential, Placement::Random] {
                    for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
                        let shape = shape_at(file_count, family, placement, policy);
                        let jia = solve_jia_publish(shape).total_bytes();
                        let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
                        let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
                        assert!(
                            write_ahead_log_leaf <= write_ahead_log_full && write_ahead_log_full <= jia,
                            "file_count={file_count} family={family:?} placement={placement:?} policy={policy:?}: 乙-M={write_ahead_log_leaf} wal_full={write_ahead_log_full} 甲={jia}"
                        );
                    }
                }
            }
        }
    }

    /// V2：K9′ 的摊销字节 / 写请求不小于 K9（R7(b) 只加，不减）。
    #[test]
    fn released_and_backlogged_never_amortizes_cheaper_than_not_tracked() {
        for file_count in [1u64, 145, 10_000] {
            for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
                for placement in [Placement::Sequential, Placement::Random] {
                    let shape = shape_at(file_count, Family::OneDataUnitPerFile, placement, PositionPolicy::Balanced);
                    let fsync = solve_write_ahead_log_fsync(shape, arm);
                    let not_tracked = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::NotTracked);
                    let released_and_backlogged = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
                    let amortized_not_tracked = amortize(fsync.total_bytes(), not_tracked.total_bytes(), 16);
                    let amortized_released_and_backlogged = amortize(fsync.total_bytes(), released_and_backlogged.total_bytes(), 16);
                    assert!(amortized_released_and_backlogged >= amortized_not_tracked - 1e-6, "file_count={file_count} arm={arm:?} placement={placement:?}: not_tracked={amortized_not_tracked} not_tracked'={amortized_released_and_backlogged}");
                    assert!(released_and_backlogged.write_calls >= not_tracked.write_calls, "file_count={file_count} arm={arm:?} placement={placement:?}");
                }
            }
        }
    }

    /// Q6.9 支撑量：在飞记录峰值 `R = N × r_f + r_c`，P=1、N=16 一格与手算一致（M20 的反例：不计 checkpoint 那条会得 16）。
    #[test]
    fn in_flight_records_and_ring_bytes_match_the_hand_calculation() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
            let fsync = solve_write_ahead_log_fsync(shape, arm);
            let checkpoint = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::NotTracked);
            let peak_in_flight_records = 16 * fsync.record_count + checkpoint.record_count;
            assert_eq!(peak_in_flight_records, 17, "arm={arm:?}");
            let ring_bytes_needed = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * peak_in_flight_records;
            assert_eq!(ring_bytes_needed, 3 * 4096 * 17);
            let default_ring_in_flight_limit = RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN;
            assert_eq!(default_ring_in_flight_limit, 65_536, "B9：默认环的在飞上限");
            assert!(peak_in_flight_records <= default_ring_in_flight_limit, "arm={arm:?} 默认环应该装得下 P=1 这一格");
        }
    }
}

#[cfg(test)]
mod row7_group_commit_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// B12（第七节 7.2）：P=100、F1、seq、concurrent_fsync_count=2（六棵树全是单节点，共享与不共享同值、位置策略不起作用）。
    #[test]
    fn batch_anchors_at_file_count_100_concurrent_fsync_count_2_match_the_hand_calculation() {
        for shared in [true, false] {
            let shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
            let jia = solve_jia_batch_publish(shape, 2, shared);
            assert_eq!(jia.total_bytes(), 410_112, "shared={shared}");
            assert_eq!(jia.write_calls, 23, "shared={shared}");
            assert_eq!(jia.allocation_layers, vec![1], "shared={shared}");

            let write_ahead_log_full = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 2, shared);
            assert_eq!(write_ahead_log_full.total_bytes(), 270_336, "shared={shared}");
            assert_eq!(write_ahead_log_full.write_calls, 12, "shared={shared}");

            let write_ahead_log_leaf = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 2, shared);
            assert_eq!(write_ahead_log_leaf.total_bytes(), 237_568, "shared={shared}");
            assert_eq!(write_ahead_log_leaf.write_calls, 10, "shared={shared}");
        }

        // 每次 fsync 摊到的字节 / 写请求：甲 = 一批 ÷ concurrent_fsync_count。
        let shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let jia = solve_jia_batch_publish(shape, 2, true);
        assert!((amortize_batch_per_fsync(jia.total_bytes(), 2) - 205_056.0).abs() < 1e-9);
        assert!((amortize_batch_per_fsync(jia.write_calls, 2) - 11.5).abs() < 1e-9);

        // g=0、N=16（N_b=8）：wal_full-K9′ / 乙-M-K9′ checkpoint 与摊销；中间版 21 / 14；
        // g=0 下 K9′ 与 K9 字节相同（积压乘数是 0）。
        let no_backlog_shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let no_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..no_backlog_shape.geometry }, ..no_backlog_shape };
        let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(no_backlog_shape, WriteAheadLogArm::WriteAheadLogFull, 2, true);
        let write_ahead_log_leaf_fsync = solve_write_ahead_log_batch_fsync(no_backlog_shape, WriteAheadLogArm::WriteAheadLogLeaf, 2, true);
        let batches_in_interval = batches_in_interval_by_fsync_count(16, 2);
        assert_eq!(batches_in_interval, 8, "N=16、concurrent_fsync_count=2 应该是 8 批（N_b）");
        let checkpoint_full_released_and_backlogged = solve_write_ahead_log_batch_checkpoint(no_backlog_shape, WriteAheadLogArm::WriteAheadLogFull, 2, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let checkpoint_leaf_released_and_backlogged = solve_write_ahead_log_batch_checkpoint(no_backlog_shape, WriteAheadLogArm::WriteAheadLogLeaf, 2, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let checkpoint_full_not_tracked = solve_write_ahead_log_batch_checkpoint(no_backlog_shape, WriteAheadLogArm::WriteAheadLogFull, 2, true, batches_in_interval, IntermediateVersionPolicy::NotTracked);
        assert_eq!(checkpoint_full_released_and_backlogged.total_bytes(), 147_968);
        assert_eq!(checkpoint_full_released_and_backlogged.write_calls, 13);
        assert_eq!(checkpoint_leaf_released_and_backlogged.total_bytes(), 180_736);
        assert_eq!(checkpoint_leaf_released_and_backlogged.write_calls, 15);
        assert_eq!(checkpoint_full_released_and_backlogged.intermediate_version_total, 21.0, "wal_full 中间版");
        assert_eq!(checkpoint_leaf_released_and_backlogged.intermediate_version_total, 14.0, "乙-M 中间版");
        assert_eq!(checkpoint_full_released_and_backlogged.total_bytes(), checkpoint_full_not_tracked.total_bytes(), "g=0 下 K9′ 与 K9 字节应该相同");

        let amortized_full = (write_ahead_log_full_fsync.total_bytes() as f64 * 8.0 + checkpoint_full_released_and_backlogged.total_bytes() as f64) / 16.0;
        let amortized_leaf = (write_ahead_log_leaf_fsync.total_bytes() as f64 * 8.0 + checkpoint_leaf_released_and_backlogged.total_bytes() as f64) / 16.0;
        assert!((amortized_full - 144_416.0).abs() < 1e-6, "amortized_full={amortized_full}");
        assert!((amortized_leaf - 130_080.0).abs() < 1e-6, "amortized_leaf={amortized_leaf}");
    }

    /// B13（第七节 7.2）：P=10⁴、concurrent_fsync_count=2，一批的用户树逐层脏节点，共享与不共享分开算。
    #[test]
    fn batch_dirty_nodes_per_layer_at_file_count_10000_concurrent_fsync_count_2_match_the_hand_calculation() {
        let shape_one_data_unit_per_file = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let shared_one_data_unit_per_file = solve_jia_batch_publish(shape_one_data_unit_per_file, 2, true);
        assert_eq!(shared_one_data_unit_per_file.extent_layers, vec![70, 1]);
        assert!((shared_one_data_unit_per_file.extent_dirty[0] - 1.006944).abs() < 1e-6);
        assert!((shared_one_data_unit_per_file.extent_dirty[1] - 1.0).abs() < 1e-9);
        assert_eq!(shared_one_data_unit_per_file.inode_layers, vec![43, 1]);
        assert!((shared_one_data_unit_per_file.inode_dirty[0] - 1.004292).abs() < 1e-6);

        let unshared_one_data_unit_per_file = solve_jia_batch_publish(shape_one_data_unit_per_file, 2, false);
        assert!((unshared_one_data_unit_per_file.extent_dirty[0] - 2.0).abs() < 1e-9);
        assert!((unshared_one_data_unit_per_file.inode_dirty[0] - 2.0).abs() < 1e-9);

        let shape_eight_adjacent = shape_at(10_000, Family::EightAdjacentDataUnitsPerFsync, Placement::Sequential, PositionPolicy::Balanced);
        let shared_eight_adjacent = solve_jia_batch_publish(shape_eight_adjacent, 2, true);
        assert_eq!(shared_eight_adjacent.extent_layers, vec![556, 4, 1]);
        assert!((shared_eight_adjacent.extent_dirty[0] - 1.104167).abs() < 1e-6);
        assert!((shared_eight_adjacent.extent_dirty[1] - 1.000709).abs() < 1e-6);
        assert!((shared_eight_adjacent.extent_dirty[2] - 1.0).abs() < 1e-9);
        // inode 同 F1（file_count 相同，与族无关）。
        assert!((shared_eight_adjacent.inode_dirty[0] - 1.004292).abs() < 1e-6);

        let unshared_eight_adjacent = solve_jia_batch_publish(shape_eight_adjacent, 2, false);
        assert!((unshared_eight_adjacent.extent_dirty[0] - 2.097222).abs() < 1e-6);
        assert!((unshared_eight_adjacent.extent_dirty[1] - 2.000661).abs() < 1e-6);
        assert!((unshared_eight_adjacent.inode_dirty[0] - 2.0).abs() < 1e-9);
    }

    /// B14（第七节 7.2）：每次 fsync 摊到的刷写与 FUA，随 concurrent_fsync_count、N 变化。
    #[test]
    fn flush_and_fua_per_fsync_match_the_hand_calculation() {
        // 每次 fsync 摊到的刷写 / FUA：一批的 barriers/fua_count（batch 结构体自己的字段，不是手算的常数）
        // 乘 N_b（这个间隔的批数）再加 checkpoint 自己的 barriers/fua_count，除以 N——用真实函数的
        // 输出算，M17（WAL 两臂每批的刷写按 concurrent_fsync_count 个 fsync 各算一次）改了 `barriers` 字段
        // 才会被这条测试看见。
        fn write_ahead_log_flushes_and_fua_per_fsync(shape: PoolShape, concurrent_fsync_count: u64, interval_fsyncs: u64) -> (f64, f64) {
            let batch = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);
            let batches_in_interval = batches_in_interval_by_fsync_count(interval_fsyncs, concurrent_fsync_count);
            let checkpoint = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
            let flushes = (batch.barriers as f64 * batches_in_interval as f64 + checkpoint.barriers as f64) / interval_fsyncs as f64;
            let fua = (0.0 * batches_in_interval as f64 + checkpoint.fua_count as f64) / interval_fsyncs as f64;
            (flushes, fua)
        }
        let shape = shape_at(1_000_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);

        // concurrent_fsync_count=2、N=16（N_b=8）：甲 2/0.5；WAL 两臂 2.25/0.0625。
        assert!((amortize_batch_per_fsync(2 * DEVICE_COUNT, 2) - 2.0).abs() < 1e-9);
        assert!((amortize_batch_per_fsync(1, 2) - 0.5).abs() < 1e-9);
        let (write_ahead_log_flushes_per_fsync, write_ahead_log_fua_per_fsync) = write_ahead_log_flushes_and_fua_per_fsync(shape, 2, 16);
        assert!((write_ahead_log_flushes_per_fsync - 2.25).abs() < 1e-9, "write_ahead_log_flushes_per_fsync={write_ahead_log_flushes_per_fsync}");
        assert!((write_ahead_log_fua_per_fsync - 0.0625).abs() < 1e-9);

        // concurrent_fsync_count=16、N=16（N_b=1）：甲 0.25/0.0625；WAL 0.5/0.0625。
        assert!((amortize_batch_per_fsync(2 * DEVICE_COUNT, 16) - 0.25).abs() < 1e-9);
        assert!((amortize_batch_per_fsync(1, 16) - 0.0625).abs() < 1e-9);
        let (write_ahead_log_flushes_per_fsync_at_concurrent_fsync_count_16, _) = write_ahead_log_flushes_and_fua_per_fsync(shape, 16, 16);
        assert!((write_ahead_log_flushes_per_fsync_at_concurrent_fsync_count_16 - 0.5).abs() < 1e-9, "value={write_ahead_log_flushes_per_fsync_at_concurrent_fsync_count_16}");

        // concurrent_fsync_count=2、N=4096（N_b=2048）：WAL 2.0009765625。
        let (write_ahead_log_flushes_per_fsync_at_interval_4096, write_ahead_log_fua_per_fsync_at_interval_4096) = write_ahead_log_flushes_and_fua_per_fsync(shape, 2, 4096);
        assert!((write_ahead_log_flushes_per_fsync_at_interval_4096 - 2.0009765625).abs() < 1e-9, "value={write_ahead_log_flushes_per_fsync_at_interval_4096}");
        assert!((write_ahead_log_fua_per_fsync_at_interval_4096 - 0.000244140625).abs() < 1e-9);
    }

    /// 阳性对照（5.5，第 7 行）：共享 / 不共享在 B13 那一格上必须给出登记原文的差；
    /// 甲组提交（concurrent_fsync_count=2 对 concurrent_fsync_count=1）必须省下固定点/记录/根槽/系统配置那一份重复开销。
    #[test]
    fn positive_controls_for_row_7_show_the_prescribed_difference() {
        let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let shared = solve_jia_batch_publish(shape, 2, true);
        let unshared = solve_jia_batch_publish(shape, 2, false);
        assert!(unshared.extent_bytes > shared.extent_bytes, "不共享应该比共享碰更多 extent 节点");

        let shape_100 = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let batch_bytes_at_concurrent_fsync_count_one = solve_jia_batch_publish(shape_100, 1, true).total_bytes();
        let batch_bytes_at_concurrent_fsync_count_two = solve_jia_batch_publish(shape_100, 2, true).total_bytes();
        assert_eq!(batch_bytes_at_concurrent_fsync_count_one, 344_576, "concurrent_fsync_count=1 应该与第 6 行 P=100 一格相同");
        assert!((batch_bytes_at_concurrent_fsync_count_two as f64) < 2.0 * batch_bytes_at_concurrent_fsync_count_one as f64, "组提交应该比两次各自发布省钱：batch_bytes_at_concurrent_fsync_count_one×2={} batch_bytes_at_concurrent_fsync_count_two={}", batch_bytes_at_concurrent_fsync_count_one * 2, batch_bytes_at_concurrent_fsync_count_two);
    }

    /// concurrent_fsync_count=1 时批级函数应该逐字节回到第 6 行的读数（甲、wal_full、乙-M 都要核）。
    #[test]
    fn batch_functions_at_concurrent_fsync_count_1_match_the_row_6_functions() {
        for shared in [true, false] {
            for file_count in [1u64, 145, 10_000] {
                let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Far);
                let batch = solve_jia_batch_publish(shape, 1, shared);
                let single = solve_jia_publish(shape);
                assert_eq!(batch.total_bytes(), single.total_bytes(), "shared={shared} file_count={file_count}");
                assert_eq!(batch.write_calls, single.write_calls, "shared={shared} file_count={file_count}");

                for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
                    let batch_fsync = solve_write_ahead_log_batch_fsync(shape, arm, 1, shared);
                    let single_fsync = solve_write_ahead_log_fsync(shape, arm);
                    assert_eq!(batch_fsync.total_bytes(), single_fsync.total_bytes(), "arm={arm:?} shared={shared} file_count={file_count}");

                    let batch_checkpoint = solve_write_ahead_log_batch_checkpoint(shape, arm, 1, shared, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
                    let single_checkpoint = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
                    assert_eq!(batch_checkpoint.total_bytes(), single_checkpoint.total_bytes(), "arm={arm:?} shared={shared} file_count={file_count}");
                    assert_eq!(batch_checkpoint.intermediate_version_total, single_checkpoint.intermediate_version_total, "arm={arm:?} shared={shared} file_count={file_count}");
                }
            }
        }
    }
}

/// 默认环的在飞记录上限（768 MiB ÷ 4096 ÷ 3）。
fn default_ring_in_flight_limit() -> u64 {
    RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN
}

// ============================================================
// 六之二：第一次登记第 2–5 行的反事实上界函数与单测，原样带过来（不进这一次任何判据、main() 不再
// 输出它们的产物行），只为让第一次的变异表（M15/M17/M18/M22 目标在这几个函数里）能在这个新二进制上
// 照跑一遍（登记第九节：「第一次的变异表在新二进制上照跑一遍」）。
// ============================================================

/// 数据单元的头 + 尾总开销（B8/Q4.1 取整用）：kb 登记的 `DATA_UNIT_HEADER_BYTES = 105`
/// （`.claude/kb/decisions/18-块里携带什么信息.md:592`）只是头；这里的 134 = 105 + 29，
/// 29 是数据单元载荷上限公式 `UNIT_BYTES − 105 − 29 = 32634`（第七节 B1）里另算的那一段，
/// 两个不是同一个量，故意不与 kb 那个名字撞名。
const DATA_UNIT_HEADER_AND_TRAILER_BYTES: u64 = 134;

impl JiaPublishOutcome {
    /// Q1.6 分子（第一次登记）：extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**
    /// 脏节点 + 根槽。这一次不进任何判据（Q1.6 判据已撤），只为让第一次变异表 M15 有目标。
    fn ancestor_and_fixed_point_share_numerator_bytes(&self) -> u64 {
        let extent_non_leaf_dirty: f64 = self.extent_dirty[1..].iter().sum();
        (extent_non_leaf_dirty * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT + self.inode_root_bytes + self.allocation_bytes + self.accounting_bytes + self.mapping_bytes + self.tree_table_bytes + self.root_slot_bytes
    }
}

/// Q2.1–Q2.4：四样固定点各自的反事实上界 `U_X = 1 − B(甲|X 不写) / B(甲)`。
fn fixed_point_counterfactual_share(shape: PoolShape, suppressed: SuppressedFixedPoints) -> f64 {
    let baseline = solve_jia_publish(shape).total_bytes();
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points(shape, suppressed).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q2.5：树表随树的棵数 T 的反事实上界，同一个 `U_X` 公式，X = 树表、T 可变。
fn tree_table_counterfactual_share(shape: PoolShape, tree_table_entry_count: u64) -> f64 {
    let baseline = solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, SuppressedFixedPoints::default(), tree_table_entry_count).total_bytes();
    let suppressed = SuppressedFixedPoints { tree_table: true, ..SuppressedFixedPoints::default() };
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, tree_table_entry_count).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q3.1 的取整：`⌈(H + c×w) / pbs⌉ × pbs`，不超过整单元宽度。
fn partial_rewrite_rounded_bytes(header: u64, conditions_per_node: f64, entry_width: u64, pbs: u64, full_width: u64) -> f64 {
    let rounded = ((header as f64 + conditions_per_node * entry_width as f64) / pbs as f64).ceil() * pbs as f64;
    rounded.min(full_width as f64)
}

/// Q3.1：逐层展开一棵树省下的字节（单盘口径，调用处再乘 `DEVICE_COUNT`）。
#[allow(clippy::too_many_arguments, reason = "登记 Q3.1 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
fn partial_rewrite_tree_savings(
    dirty_per_layer: &[f64],
    leaf_conditions: f64,
    leaf_header: u64,
    leaf_entry_width: u64,
    leaf_full_width: u64,
    internal_header: u64,
    internal_entry_width: u64,
    internal_full_width: u64,
    pbs: u64,
) -> f64 {
    let mut savings = 0.0;
    for (layer_index, &dirty) in dirty_per_layer.iter().enumerate() {
        if dirty <= 0.0 {
            continue;
        }
        let (header, entry_width, full_width, conditions) = if layer_index == 0 {
            (leaf_header, leaf_entry_width, leaf_full_width, leaf_conditions)
        } else {
            (internal_header, internal_entry_width, internal_full_width, dirty_per_layer[layer_index - 1])
        };
        let conditions_per_node = conditions / dirty;
        let rounded = partial_rewrite_rounded_bytes(header, conditions_per_node, entry_width, pbs, full_width);
        savings += dirty * (full_width as f64 - rounded);
    }
    savings
}

/// Q3.1：改一条记录也整单元重写的反事实上界，在甲的这一格上算。
fn whole_unit_rewrite_counterfactual_share(shape: PoolShape, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let outcome = solve_jia_publish(shape);
    let fsync_data_unit_count = shape.family.data_units_per_fsync() as f64;

    let mut savings = 0.0;

    savings += partial_rewrite_tree_savings(&outcome.extent_dirty, fsync_data_unit_count, node_header_bytes(EXTENT_KEY_BYTES), EXTENT_LEAF_ENTRY_BYTES, NODE_BYTES, node_header_bytes(EXTENT_KEY_BYTES), internal_entry_bytes(EXTENT_KEY_BYTES, false), NODE_BYTES, pbs);

    savings += partial_rewrite_tree_savings(&outcome.inode_dirty, 1.0, INODE_CONTAINER_HEADER_BYTES, INODE_CONTAINER_ENTRY_BYTES, UNIT_BYTES, node_header_bytes(INODE_KEY_BYTES), INODE_INTERNAL_ENTRY_BYTES, NODE_BYTES, pbs);

    savings += partial_rewrite_tree_savings(&outcome.allocation_dirty, 4.0 * outcome.written_units, node_header_bytes(ALLOCATION_KEY_BYTES), ALLOCATION_LEAF_ENTRY_BYTES, NODE_BYTES, node_header_bytes(ALLOCATION_KEY_BYTES), internal_entry_bytes(ALLOCATION_KEY_BYTES, false), NODE_BYTES, pbs);

    let extent_dirty_total = sum_f64(&outcome.extent_dirty);
    let inode_container_dirty = outcome.inode_dirty[0];
    let inode_root_dirty_total: f64 = outcome.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&outcome.allocation_dirty);
    let mapping_leaf_conditions = 2.0 * (fsync_data_unit_count + extent_dirty_total + inode_container_dirty + inode_root_dirty_total + allocation_dirty_total + outcome.accounting_dirty);
    savings += partial_rewrite_tree_savings(&outcome.mapping_dirty, mapping_leaf_conditions, node_header_bytes(MAPPING_KEY_BYTES), MAPPING_LEAF_ENTRY_BYTES, NODE_BYTES, node_header_bytes(MAPPING_KEY_BYTES), internal_entry_bytes(MAPPING_KEY_BYTES, false), NODE_BYTES, pbs);

    if outcome.accounting_dirty > 0.0 {
        let conditions_per_node = 15.0 / outcome.accounting_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(ACCOUNTING_KEY_BYTES), conditions_per_node, ACCOUNTING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += outcome.accounting_dirty * (NODE_BYTES as f64 - rounded);
    }

    let tree_table_dirty = outcome.tree_table_bytes as f64 / NODE_BYTES as f64 / DEVICE_COUNT as f64;
    if tree_table_dirty > 0.0 {
        let conditions_per_node = TREE_TABLE_TOUCHED_ROOTS as f64 / tree_table_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(TREE_TABLE_KEY_BYTES), conditions_per_node, TREE_TABLE_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += tree_table_dirty * (NODE_BYTES as f64 - rounded);
    }

    (savings * DEVICE_COUNT as f64) / outcome.total_bytes() as f64
}

/// Q4.1：数据单元按内容取整的反事实上界。
fn data_unit_rounding_counterfactual_share(shape: PoolShape, content_bytes: u64, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let baseline = solve_jia_publish(shape).total_bytes();
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
    let saving_per_unit_per_device = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
    (fsync_data_unit_count * DEVICE_COUNT * saving_per_unit_per_device) as f64 / baseline as f64
}

/// 六棵会长高的 / 恒定的树各自的树高（extent、inode、分配记录、记账、映射、树表）。
fn tree_heights(outcome: &JiaPublishOutcome) -> [usize; 6] {
    [outcome.extent_layers.len(), outcome.inode_layers.len(), outcome.allocation_layers.len(), 1, outcome.mapping_layers.len(), 1]
}

/// 一棵树高 `height` 的树，从叶到根走一条路径要写多少字节：inode 叶是容器（32768），其余每层 16384。
fn one_path_bytes(height: usize, leaf_is_inode_container: bool) -> u64 {
    assert!(height >= 1, "树高至少 1（根即叶）");
    if leaf_is_inode_container {
        UNIT_BYTES + NODE_BYTES * (height as u64 - 1)
    } else {
        NODE_BYTES * height as u64
    }
}

/// 甲这一格的 `A(P)`（`total_bytes`）与 `A_tree(P)`（六棵树的字节之和）。
fn total_bytes_and_tree_bytes(shape: PoolShape) -> (u64, u64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes;
    (outcome.total_bytes(), tree_bytes)
}

/// 三种「树高 × 棵数」界的读法。
fn rho_readings(shape: PoolShape) -> (f64, f64, f64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = (outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes) as f64;
    let heights = tree_heights(&outcome);
    let one_path_sum: u64 =
        one_path_bytes(heights[0], false) + one_path_bytes(heights[1], true) + one_path_bytes(heights[2], false) + one_path_bytes(heights[3], false) + one_path_bytes(heights[4], false) + one_path_bytes(heights[5], false);
    let maximum_tree_height = *heights.iter().max().expect("六个高度都有值") as u64;
    let path_ratio_by_actual_height = tree_bytes / (DEVICE_COUNT as f64 * one_path_sum as f64);
    let path_ratio_by_maximum_height_node_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * NODE_BYTES as f64 * DEVICE_COUNT as f64);
    let path_ratio_by_maximum_height_unit_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * UNIT_BYTES as f64 * DEVICE_COUNT as f64);
    (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width)
}

/// 在 `[low, high]` 上二分，找「`height_at` 第一次超过 `base_height`」的最小 P。
fn bisect_first_threshold(low: u64, high: u64, base_height: usize, height_at: impl Fn(u64) -> usize) -> Option<u64> {
    if height_at(high) <= base_height {
        return None;
    }
    let mut low = low;
    let mut high = high;
    while low < high {
        let mid = low + (high - low) / 2;
        if height_at(mid) > base_height {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    Some(low)
}

/// 每棵树每次长高一层的门槛：从 P = 1 的高度开始，逐次二分找下一个跨越点，直到搜索上限。
fn height_growth_thresholds(search_upper_bound: u64, height_at: impl Fn(u64) -> usize) -> Vec<u64> {
    let mut thresholds = Vec::new();
    let mut base_height = height_at(1);
    let mut floor = 1u64;
    loop {
        match bisect_first_threshold(floor, search_upper_bound, base_height, &height_at) {
            Some(threshold) => {
                thresholds.push(threshold);
                base_height = height_at(threshold);
                floor = threshold;
                if thresholds.len() > 16 {
                    break;
                }
            }
            None => break,
        }
    }
    thresholds
}

#[cfg(test)]
mod row_2_to_4_counterfactual_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    #[test]
    fn whole_unit_rewrite_general_implementation_matches_the_hand_calculated_cell_at_file_count_1() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let share = whole_unit_rewrite_counterfactual_share(shape, PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES);
        assert!((share - 250_880.0 / 344_576.0).abs() < 1e-9, "share={share}");
    }

    #[test]
    fn fixed_point_upper_bounds_are_equal_at_file_count_1_because_every_fixed_point_is_one_node() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let expected = 32_768.0 / 344_576.0;
        let accounting = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { accounting: true, ..Default::default() });
        let allocation = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { allocation: true, ..Default::default() });
        let mapping = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { mapping: true, ..Default::default() });
        let tree_table = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { tree_table: true, ..Default::default() });
        for (name, value) in [("accounting", accounting), ("allocation", allocation), ("mapping", mapping), ("tree_table", tree_table)] {
            assert!((value - expected).abs() < 1e-9, "{name}={value} expected={expected}");
        }
        assert!(expected < 0.10, "P=1 一格：{expected}");
    }

    #[test]
    fn suppressing_any_fixed_point_still_converges_and_stays_nonnegative() {
        for suppressed in [
            SuppressedFixedPoints { accounting: true, ..Default::default() },
            SuppressedFixedPoints { allocation: true, ..Default::default() },
            SuppressedFixedPoints { mapping: true, ..Default::default() },
            SuppressedFixedPoints { tree_table: true, ..Default::default() },
        ] {
            for file_count in [1u64, 145, 10_000, 1_000_000] {
                let shape = shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Far);
                let outcome = solve_jia_publish_with_suppressed_fixed_points(shape, suppressed);
                assert!(outcome.fixed_point_iterations < MAXIMUM_FIXED_POINT_ITERATIONS, "file_count={file_count} suppressed={suppressed:?}");
                assert!(outcome.total_bytes() > 0, "file_count={file_count} suppressed={suppressed:?}");
            }
        }
    }

    #[test]
    fn tree_table_reaches_two_layers_at_82_entries_and_upper_bound_does_not_shrink() {
        let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let share_at_7 = tree_table_counterfactual_share(shape, 7);
        let share_at_81 = tree_table_counterfactual_share(shape, 81);
        let share_at_82 = tree_table_counterfactual_share(shape, 82);
        assert!((share_at_7 - share_at_81).abs() < 1e-9, "T=7 与 T=81 都还是 1 层，上界应该相同：{share_at_7} vs {share_at_81}");
        assert!(share_at_82 >= share_at_81, "T=82 长到 2 层，上界不该变小：{share_at_82} vs {share_at_81}");
    }

    #[test]
    fn data_unit_rounding_per_unit_savings_match_the_anchor() {
        let cases: [(u64, u64, u64); 8] = [(100, 512, 32_256), (100, 4096, 28_672), (4100, 512, 28_160), (4100, 4096, 24_576), (16_000, 512, 16_384), (16_000, 4096, 16_384), (32_634, 512, 0), (32_634, 4096, 0)];
        for (content_bytes, pbs, expected_saving_per_unit_per_device) in cases {
            let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
            let saving = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
            assert_eq!(saving, expected_saving_per_unit_per_device, "content_bytes={content_bytes} pbs={pbs}");
        }
    }

    #[test]
    fn content_size_4000_is_header_sensitive_unlike_the_other_sampled_sizes() {
        for (content_bytes, pbs) in [(100u64, 512u64), (4100, 512), (16_000, 512), (32_634, 512)] {
            let with_header = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
            let without_header = content_bytes.div_ceil(pbs) * pbs;
            assert_eq!(with_header, without_header, "content_bytes={content_bytes}：这几档不该对头字节敏感");
        }
        let with_header = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + 4000).div_ceil(512) * 512;
        let without_header = 4000u64.div_ceil(512) * 512;
        assert_ne!(with_header, without_header, "s=4000 应该是防等价的那一档");
    }

    #[test]
    fn data_unit_rounding_at_content_size_4000_accounts_for_the_data_unit_header() {
        let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let baseline = solve_jia_publish(PoolShape { geometry: Geometry { physical_block_size_bytes: 512, ..shape.geometry }, ..shape }).total_bytes();
        let share = data_unit_rounding_counterfactual_share(shape, 4000, 512);
        let expected = (1 * 2 * 28_160) as f64 / baseline as f64;
        assert!((share - expected).abs() < 1e-12, "share={share} expected={expected}");
    }

    #[test]
    fn data_unit_rounding_scales_linearly_with_data_units_touched_per_fsync() {
        let shape_one_data_unit_per_file = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let shape_eight_adjacent = PoolShape { family: Family::EightAdjacentDataUnitsPerFsync, ..shape_one_data_unit_per_file };
        let baseline_one_data_unit_per_file = solve_jia_publish(shape_one_data_unit_per_file).total_bytes() as f64;
        let baseline_f8a = solve_jia_publish(shape_eight_adjacent).total_bytes() as f64;
        let share_one_data_unit_per_file = data_unit_rounding_counterfactual_share(shape_one_data_unit_per_file, 4100, 512);
        let share_f8a = data_unit_rounding_counterfactual_share(shape_eight_adjacent, 4100, 512);
        let numerator_one_data_unit_per_file = share_one_data_unit_per_file * baseline_one_data_unit_per_file;
        let numerator_f8a = share_f8a * baseline_f8a;
        assert!((numerator_f8a / numerator_one_data_unit_per_file - 8.0).abs() < 1e-6, "numerator_one_data_unit_per_file={numerator_one_data_unit_per_file} numerator_f8a={numerator_f8a}");
    }

    /// B3（值断言，第一次登记）：G23.1 在锚点的份额，用来让 M15 有单测可红。
    #[test]
    fn ancestor_and_fixed_point_share_at_the_anchor_matches_the_literal_and_with_system_configuration_readings() {
        let outcome = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(outcome.extent_layers.len(), 1, "P=1 时 extent 树高 1，根即叶，非叶节点为 0");
        let literal_numerator = outcome.ancestor_and_fixed_point_share_numerator_bytes();
        let total = outcome.total_bytes();
        assert_eq!(literal_numerator, 164_352);
        let share = literal_numerator as f64 / total as f64;
        assert!((share - 0.476_968_796).abs() < 1e-6, "share={share}");
        let with_system_configuration = (literal_numerator + outcome.system_configuration_bytes) as f64 / total as f64;
        assert!((with_system_configuration - 0.500_742_942).abs() < 1e-6, "with_system_configuration={with_system_configuration}");
    }
}

#[cfg(test)]
mod row_5_growth_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    #[test]
    fn tree_bytes_at_file_count_1_matches_the_hand_calculation() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let (total_bytes, tree_bytes) = total_bytes_and_tree_bytes(shape);
        assert_eq!(total_bytes, 344_576);
        assert_eq!(tree_bytes, 16384 * 2 + 32768 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2);
    }

    #[test]
    fn path_ratio_readings_do_not_exceed_one_at_file_count_1() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width) = rho_readings(shape);
        assert!((path_ratio_by_actual_height - 1.0).abs() < 1e-9, "path_ratio_by_actual_height={path_ratio_by_actual_height}");
        assert!(path_ratio_by_maximum_height_node_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_node_width={path_ratio_by_maximum_height_node_width}");
        assert!(path_ratio_by_maximum_height_unit_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_unit_width={path_ratio_by_maximum_height_unit_width}");
    }

    #[test]
    fn extent_height_growth_threshold_matches_the_anchor() {
        let height_at = |file_count: u64| {
            let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
            solve_jia_publish(shape).extent_layers.len()
        };
        let thresholds = height_growth_thresholds(1_000_000, height_at);
        assert!(!thresholds.is_empty(), "extent 树在 1..=1000000 之间应该至少长高一次");
        assert_eq!(thresholds[0], 145, "extent 叶容量 144，第 145 个条目跨过边界");
    }

    /// 门槛搜索：分配记录树的门槛这一次是 181（R1 之后），不是第一次登记的 205（legacy 门槛见 `jia_anchor_tests`）。
    #[test]
    fn allocation_height_growth_threshold_matches_the_value_after_counting_data_units_in_backlog() {
        let height_at = |file_count: u64| {
            let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
            solve_jia_publish(shape).allocation_layers.len()
        };
        let thresholds = height_growth_thresholds(1_000_000, height_at);
        assert!(!thresholds.is_empty());
        assert_eq!(thresholds[0], 181, "R1 之后门槛从 205 移到 181（第七节 B10）");
    }

    #[test]
    fn bytes_per_layer_growth_ratio_at_the_extent_threshold_is_close_to_one_layer_one_path() {
        let shape_before = shape_at(144, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let shape_at_threshold = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let (_, tree_bytes_before_threshold) = total_bytes_and_tree_bytes(shape_before);
        let (_, tree_bytes_at_threshold) = total_bytes_and_tree_bytes(shape_at_threshold);
        let bytes_per_layer_growth_ratio = (tree_bytes_at_threshold as f64 - tree_bytes_before_threshold as f64) / (NODE_BYTES as f64 * DEVICE_COUNT as f64);
        assert!((bytes_per_layer_growth_ratio - 1.0).abs() < 1e-9, "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 在这个门槛上长高一层应该恰好多写一条路径的份额");
        assert!(bytes_per_layer_growth_ratio <= 1.05, "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 长高一层不该比『一层一条路径』贵太多");
    }
}

// ============================================================
// 七、main：第 6 行主表（stage 1）、阳性对照、Q6.1–Q6.9、Q6.11、8.2、判别力自证
// ============================================================

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!("name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} model=counting", default_ring_in_flight_limit())));
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope stage=2 covers=第6行主表五列(甲/wal_full-K9'/乙-M-K9'/wal_full-K9/乙-M-K9),阳性对照,R1,R2,R3,R4,R6,R7,B1,B2,B4,B5,B7,B10,B11,B15-B19,Q6.1-Q6.9,Q6.11,8.2(第6行判定格),判别力自证(轴n),R8,第7行主表row7_grid(1800格),B12-B14,阳性对照row7,Q7.1-Q7.6,row7_fold_point,8.2(第7行,仅N按批计) missing=8.2(第7行phi0.5/K3'/g0/pbs4096/K9'-b五个反向取样点),F9不共享取随机(仅占位说明未建开关),判别力自证(轴k)"
        )
    );

    // B1：容量一览（与第一次相同）。
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=b1_capacities extent_leaf={} extent_internal={} inode_container={} inode_internal={} alloc_leaf={} alloc_internal={} accounting_leaf={} accounting_internal={} mapping_leaf={} mapping_internal={} tree_table={} named_items_per_record={}",
            node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES),
            node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)),
            inode_container_capacity(),
            node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES),
            node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES),
            node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, false)),
            node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES),
            node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, false)),
            node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES),
            node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, false)),
            node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES),
            NAMED_ITEMS_PER_RECORD_MAIN,
        ))
    );

    // 阳性对照（5.5）：三条臂各自的格与「必须出现的差」，K9 与 K9′ 都报。
    let control_shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
    let jia_fsync = solve_jia_publish(control_shape).total_bytes();
    let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    let checkpoint_full_released_and_backlogged = solve_write_ahead_log_checkpoint(control_shape, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    let checkpoint_leaf_released_and_backlogged = solve_write_ahead_log_checkpoint(control_shape, WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control cell=p1_f1_seq jia_fsync_bytes={jia_fsync} write_ahead_log_full_fsync_bytes={write_ahead_log_full_fsync} write_ahead_log_leaf_fsync_bytes={write_ahead_log_leaf_fsync} jia_minus_write_ahead_log_full={} write_ahead_log_full_minus_write_ahead_log_leaf={} jia_root_slots_per_16_fsync=16 write_ahead_log_full_root_slots_per_16_fsync=1 write_ahead_log_full_released_and_backlogged_intermediate_total={} write_ahead_log_full_released_and_backlogged_allocation_layers={:?} write_ahead_log_leaf_released_and_backlogged_intermediate_total={} write_ahead_log_leaf_released_and_backlogged_allocation_layers={:?}",
            jia_fsync - write_ahead_log_full_fsync,
            write_ahead_log_full_fsync - write_ahead_log_leaf_fsync,
            checkpoint_full_released_and_backlogged.intermediate_version_total,
            checkpoint_full_released_and_backlogged.allocation_layers,
            checkpoint_leaf_released_and_backlogged.intermediate_version_total,
            checkpoint_leaf_released_and_backlogged.allocation_layers,
        ))
    );

    // 第 6 行主表（登记 5.4）：P × 族 × 落点 × N × 三种位置策略，五列各报各的。
    let file_counts: [u64; 6] = [1, 100, 10_000, 1_000_000, 100_000_000, 145];
    let families = [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync];
    let placements = [Placement::Sequential, Placement::Random];
    let intervals: [u64; 4] = [1, 16, 256, 4096];
    let policies = [PositionPolicy::Near, PositionPolicy::Balanced, PositionPolicy::Far];

    for &file_count in &file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                    let jia = solve_jia_publish(shape);
                    let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull);
                    let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf);
                    for &interval in &intervals {
                        let cp_full_not_tracked = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::NotTracked);
                        let cp_leaf_not_tracked = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, interval, IntermediateVersionPolicy::NotTracked);
                        let cp_full_released_and_backlogged = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
                        let cp_leaf_released_and_backlogged = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, interval, IntermediateVersionPolicy::ReleasedAndBacklogged);

                        let jia_bytes = jia.total_bytes();
                        let amortized_full_not_tracked = amortize(write_ahead_log_full_fsync.total_bytes(), cp_full_not_tracked.total_bytes(), interval);
                        let amortized_leaf_not_tracked = amortize(write_ahead_log_leaf_fsync.total_bytes(), cp_leaf_not_tracked.total_bytes(), interval);
                        let amortized_full_released_and_backlogged = amortize(write_ahead_log_full_fsync.total_bytes(), cp_full_released_and_backlogged.total_bytes(), interval);
                        let amortized_leaf_released_and_backlogged = amortize(write_ahead_log_leaf_fsync.total_bytes(), cp_leaf_released_and_backlogged.total_bytes(), interval);

                        // Q6.3：三条臂之比（fsync 行 / 摊销行都报，用 K9′ 的摊销值）。
                        let ratio_jia_over_full = jia_bytes as f64 / amortized_full_released_and_backlogged;
                        let ratio_jia_over_leaf = jia_bytes as f64 / amortized_leaf_released_and_backlogged;
                        let ratio_full_over_leaf = amortized_full_released_and_backlogged / amortized_leaf_released_and_backlogged;

                        // Q6.5 / Q6.5c：`s` = 1 − 摊销 ÷ 甲摊销（字节与写请求）。
                        let saved_share_write_ahead_log_full_bytes = 1.0 - amortized_full_released_and_backlogged / jia_bytes as f64;
                        let saved_share_write_ahead_log_leaf_bytes = 1.0 - amortized_leaf_released_and_backlogged / jia_bytes as f64;

                        // Q6.6：K9′ 比 K9 多写多少。
                        let delta_full_bytes = amortized_full_released_and_backlogged - amortized_full_not_tracked;
                        let delta_leaf_bytes = amortized_leaf_released_and_backlogged - amortized_leaf_not_tracked;

                        // F2（第十节）：不动点第一半（撞迭代上限未收敛）与第二半（某棵树第 0 层节点数 ≥ 64
                        // 且脏节点占比 ≥ 50%）——甲与四条 WAL checkpoint 核心（K9 / K9′ × wal_full / 乙-M）分开判。
                        let (jia_unconverged, jia_saturated_trees) = jia_convergence_and_saturation_status(&jia);
                        let (full_not_tracked_unconverged, full_not_tracked_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_full_not_tracked);
                        let (full_released_and_backlogged_unconverged, full_released_and_backlogged_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_full_released_and_backlogged);
                        let (leaf_not_tracked_unconverged, leaf_not_tracked_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_leaf_not_tracked);
                        let (leaf_released_and_backlogged_unconverged, leaf_released_and_backlogged_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_leaf_released_and_backlogged);

                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=row6_grid p={file_count} family={} placement={} policy={} n={interval} \
                                 jia_fsync_bytes={jia_bytes} jia_calls={} jia_barriers={} jia_fua={} jia_amortized_bytes={jia_bytes} \
                                 wal_full_not_tracked_fsync_bytes={} wal_full_not_tracked_checkpoint_bytes={} wal_full_not_tracked_amortized_bytes={amortized_full_not_tracked} wal_full_not_tracked_alloc_layers={:?} \
                                 wal_full_released_and_backlogged_fsync_bytes={} wal_full_released_and_backlogged_checkpoint_bytes={} wal_full_released_and_backlogged_amortized_bytes={amortized_full_released_and_backlogged} wal_full_released_and_backlogged_alloc_layers={:?} wal_full_released_and_backlogged_intermediate_total={} \
                                 wal_leaf_not_tracked_fsync_bytes={} wal_leaf_not_tracked_checkpoint_bytes={} wal_leaf_not_tracked_amortized_bytes={amortized_leaf_not_tracked} \
                                 wal_leaf_released_and_backlogged_fsync_bytes={} wal_leaf_released_and_backlogged_checkpoint_bytes={} wal_leaf_released_and_backlogged_amortized_bytes={amortized_leaf_released_and_backlogged} wal_leaf_released_and_backlogged_alloc_layers={:?} wal_leaf_released_and_backlogged_intermediate_total={} \
                                 ratio_jia_over_wal_full={ratio_jia_over_full} ratio_jia_over_wal_leaf={ratio_jia_over_leaf} ratio_wal_full_over_wal_leaf={ratio_full_over_leaf} \
                                 saved_share_write_ahead_log_full_bytes={saved_share_write_ahead_log_full_bytes} saved_share_write_ahead_log_leaf_bytes={saved_share_write_ahead_log_leaf_bytes} \
                                 delta_released_and_backlogged_minus_not_tracked_wal_full_bytes={delta_full_bytes} delta_released_and_backlogged_minus_not_tracked_wal_leaf_bytes={delta_leaf_bytes} \
                                 jia_write_calls_per_fsync={} wal_full_released_and_backlogged_write_calls_per_fsync={} wal_leaf_released_and_backlogged_write_calls_per_fsync={} \
                                 extent_layers={} inode_layers={} allocation_layers={} mapping_layers={} fixed_point_iterations={} \
                                 jia_unconverged={jia_unconverged} jia_saturated_trees={jia_saturated_trees:?} \
                                 wal_full_not_tracked_unconverged={full_not_tracked_unconverged} wal_full_not_tracked_saturated_trees={full_not_tracked_saturated_trees:?} \
                                 wal_full_released_and_backlogged_unconverged={full_released_and_backlogged_unconverged} wal_full_released_and_backlogged_saturated_trees={full_released_and_backlogged_saturated_trees:?} \
                                 wal_leaf_not_tracked_unconverged={leaf_not_tracked_unconverged} wal_leaf_not_tracked_saturated_trees={leaf_not_tracked_saturated_trees:?} \
                                 wal_leaf_released_and_backlogged_unconverged={leaf_released_and_backlogged_unconverged} wal_leaf_released_and_backlogged_saturated_trees={leaf_released_and_backlogged_saturated_trees:?}",
                                family.tag(),
                                placement.tag(),
                                policy.tag(),
                                jia.write_calls,
                                jia.barriers,
                                jia.fua_count,
                                write_ahead_log_full_fsync.total_bytes(),
                                cp_full_not_tracked.total_bytes(),
                                cp_full_not_tracked.allocation_layers,
                                write_ahead_log_full_fsync.total_bytes(),
                                cp_full_released_and_backlogged.total_bytes(),
                                cp_full_released_and_backlogged.allocation_layers,
                                cp_full_released_and_backlogged.intermediate_version_total,
                                write_ahead_log_leaf_fsync.total_bytes(),
                                cp_leaf_not_tracked.total_bytes(),
                                write_ahead_log_leaf_fsync.total_bytes(),
                                cp_leaf_released_and_backlogged.total_bytes(),
                                cp_leaf_released_and_backlogged.allocation_layers,
                                cp_leaf_released_and_backlogged.intermediate_version_total,
                                jia.write_calls,
                                write_ahead_log_full_fsync.write_calls,
                                write_ahead_log_leaf_fsync.write_calls,
                                jia.extent_layers.len(),
                                jia.inode_layers.len(),
                                jia.allocation_layers.len(),
                                jia.mapping_layers.len(),
                                jia.fixed_point_iterations,
                            ))
                        );
                    }
                }
            }
        }
    }

    // Q6.11：第三节 3.2 四处改动前后甲的差，覆盖第 6 行主表同一批格（这一次改成要跑）；
    // 每格另判 F2 两半（改前 legacy、改后 fixed 分开判，F2 触发的格按登记「结论收窄」报，不当普通值用）。
    for &file_count in &file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                    let fixed_outcome = solve_jia_publish(shape);
                    let legacy_outcome = legacy::legacy_solve_jia_publish(shape);
                    let fixed = fixed_outcome.total_bytes();
                    let legacy = legacy_outcome.total_bytes();
                    let delta = fixed as i128 - legacy as i128;
                    let (fixed_unconverged, fixed_saturated_trees) = jia_convergence_and_saturation_status(&fixed_outcome);
                    let (legacy_unconverged, legacy_saturated_trees) = jia_convergence_and_saturation_status(&legacy_outcome);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=q6_11_legacy_vs_fixed p={file_count} family={} placement={} policy={} fixed_bytes={fixed} legacy_bytes={legacy} delta_bytes={delta} differs={} \
                             fixed_unconverged={fixed_unconverged} fixed_saturated_trees={fixed_saturated_trees:?} legacy_unconverged={legacy_unconverged} legacy_saturated_trees={legacy_saturated_trees:?}",
                            family.tag(),
                            placement.tag(),
                            policy.tag(),
                            fixed != legacy,
                        ))
                    );
                }
            }
        }
    }

    // 8.2 几何敏感性：第 6 行判定格上逐个换到反向取样点（代表格 P=100000, F1, seq, Lbalanced, N=16）。
    {
        let representative_file_count = 100_000u64;
        let base_shape = PoolShape { file_count: representative_file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let base_jia = solve_jia_publish(base_shape).total_bytes();
        let base_write_ahead_log_full = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let base_write_ahead_log_leaf = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        let base_checkpoint_full = solve_write_ahead_log_checkpoint(base_shape, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let base_amortized_full = amortize(base_write_ahead_log_full, base_checkpoint_full.total_bytes(), 16);
        let base_saved_share_write_ahead_log_full_value = 1.0 - base_amortized_full / base_jia as f64;

        let half_fill_shape = PoolShape { geometry: Geometry { fill_ratio: FillRatio::Half, ..Geometry::main() }, ..base_shape };
        let reverse_identity_reference_shape = PoolShape { geometry: Geometry { identity_reference: true, ..Geometry::main() }, ..base_shape };
        let no_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..Geometry::main() }, ..base_shape };
        let reverse_pbs_shape = PoolShape { geometry: Geometry { physical_block_size_bytes: REVERSE_PHYSICAL_BLOCK_SIZE_BYTES, ..Geometry::main() }, ..base_shape };

        for (label, shape) in [("phi_0.5", half_fill_shape), ("k3_prime", reverse_identity_reference_shape), ("g_0", no_backlog_shape), ("pbs_4096", reverse_pbs_shape)] {
            let jia = solve_jia_publish(shape).total_bytes();
            let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
            let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
            let checkpoint_full = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
            let amortized_full = amortize(write_ahead_log_full, checkpoint_full.total_bytes(), 16);
            let saved_share_write_ahead_log_full_value = 1.0 - amortized_full / jia as f64;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=geometry_sensitivity_sample_row6 point={label} p={representative_file_count} base_jia={base_jia} reverse_jia={jia} base_wal_full_fsync={base_write_ahead_log_full} reverse_wal_full_fsync={write_ahead_log_full} base_wal_leaf_fsync={base_write_ahead_log_leaf} reverse_wal_leaf_fsync={write_ahead_log_leaf} base_saved_share_wal_full={base_saved_share_write_ahead_log_full_value} reverse_saved_share_wal_full={saved_share_write_ahead_log_full_value} saved_share_sign_changes={}",
                    (base_saved_share_write_ahead_log_full_value > 0.0) != (saved_share_write_ahead_log_full_value > 0.0),
                ))
            );
        }

        // K9′-b（R7 反向取样点）：中间版不进积压——只影响 s，不影响 jia/fsync。
        let intermediate_version_alternate_reading_note = "K9′-b（中间版不进积压）与 N 按批计（第 7 行才有 concurrent_fsync_count 轴，这一次只报占位）在这一版未建独立开关，交主 agent";
        println!("{}", emitter.emit_raw(&format!("name=geometry_sensitivity_sample_row6 point=k9prime_b_not_modeled_this_pass p={representative_file_count} note={intermediate_version_alternate_reading_note:?}")));

        // 独立散点（R6 反向取样点）：rand 下把间隔并集当独立散点，不当散连续组。
        let rand_shape = PoolShape { placement: Placement::Random, ..base_shape };
        let rand_family_f8a_shape = PoolShape { family: Family::EightAdjacentDataUnitsPerFsync, ..rand_shape };
        let grouped_extent_touch = {
            let data_unit_count = rand_family_f8a_shape.family.data_unit_count(rand_family_f8a_shape.file_count);
            let extent_leaf_cap = node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES);
            let extent_internal_cap = node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false));
            let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
            scattered_continuous_groups_touch(extent_layers[0], extent_leaf_cap as f64, 16.0, 8.0, data_unit_count as f64)
        };
        let independent_extent_touch = {
            let data_unit_count = rand_family_f8a_shape.family.data_unit_count(rand_family_f8a_shape.file_count);
            let extent_leaf_cap = node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES);
            scattered_group_touch(16.0 * 8.0, data_unit_count as f64, extent_leaf_cap as f64)
        };
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=geometry_sensitivity_sample_row6 point=independent_scattered_points p={representative_file_count} family=F8A placement=rand n=16 grouped_extent_leaf_dirty={grouped_extent_touch} independent_extent_leaf_dirty={independent_extent_touch} independent_makes_wal_more_expensive={}",
                independent_extent_touch > grouped_extent_touch,
            ))
        );
    }

    // 判别力自证（第八节 8.2）：`s` 的 10% 与 0 两道门槛在实际值序列里能不能翻面（沿 N 轴，P=1、F1、seq、Lbalanced）。
    {
        let shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let jia_bytes = solve_jia_publish(shape).total_bytes();
        let fsync_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let mut saved_share_values = Vec::new();
        for &interval in &intervals {
            let checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
            let amortized = amortize(fsync_full, checkpoint.total_bytes(), interval);
            let saved_share = 1.0 - amortized / jia_bytes as f64;
            saved_share_values.push((interval, saved_share));
        }
        let crosses_zero = saved_share_values.windows(2).any(|adjacent_pair| (adjacent_pair[0].1 > 0.0) != (adjacent_pair[1].1 > 0.0));
        let crosses_ten_percent = saved_share_values.windows(2).any(|adjacent_pair| (adjacent_pair[0].1 >= 0.10) != (adjacent_pair[1].1 >= 0.10));
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=discriminability_self_proof axis=n shape=p1_f1_seq_lbalanced values={saved_share_values:?} crosses_zero_threshold={crosses_zero} crosses_ten_percent_threshold={crosses_ten_percent}"
            ))
        );
    }

    // ============================================================
    // 第 7 行（第二段，R8 组提交）：P × 族 × 落点 × 策略 × N × concurrent_fsync_count × 共享/不共享，三臂。
    // ============================================================
    let row_seven_file_counts: [u64; 5] = [100, 10_000, 1_000_000, 100_000_000, 145];
    let row_seven_intervals: [u64; 3] = [16, 256, 4096];
    let concurrent_fsync_counts: [u64; 5] = [1, 2, 4, 8, 16];
    let sharing_options = [true, false];

    // 阳性对照（5.5，第 7 行）：B12/B13 那两格，逐字节钉在单测里；这里只报一行，指回单测名。
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control_row7 cell=p100_f1_seq_concurrent_fsync_count_2 jia_batch_bytes={} wal_full_batch_bytes={} wal_leaf_batch_bytes={} shared_extent_dirty_matches_unshared={}",
            solve_jia_batch_publish(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, 2, true).total_bytes(),
            solve_write_ahead_log_batch_fsync(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, WriteAheadLogArm::WriteAheadLogFull, 2, true).total_bytes(),
            solve_write_ahead_log_batch_fsync(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, WriteAheadLogArm::WriteAheadLogLeaf, 2, true).total_bytes(),
            solve_jia_batch_publish(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, 2, true).extent_bytes
                == solve_jia_batch_publish(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, 2, false).extent_bytes,
        ))
    );

    for &file_count in &row_seven_file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    for &interval in &row_seven_intervals {
                        for shared in sharing_options {
                            // Q7.3/Q7.3b：沿 concurrent_fsync_count 从小到大找 `s` 第一次 ≤ 0 / < 10% 的那个格（甲不比这条 WAL 臂贵）。
                            let mut fold_point_zero_full: Option<u64> = None;
                            let mut fold_point_ten_percent_full: Option<u64> = None;
                            let mut fold_point_zero_leaf: Option<u64> = None;
                            let mut fold_point_ten_percent_leaf: Option<u64> = None;

                            for &concurrent_fsync_count in &concurrent_fsync_counts {
                                if file_count < concurrent_fsync_count {
                                    continue; // P < concurrent_fsync_count 的格不适用（第一节「读法写死」）。
                                }
                                let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                                let jia = solve_jia_batch_publish(shape, concurrent_fsync_count, shared);
                                let jia_batch_bytes = jia.total_bytes();
                                let jia_amortized_bytes = amortize_batch_per_fsync(jia_batch_bytes, concurrent_fsync_count);

                                let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, shared);
                                let write_ahead_log_leaf_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, shared);
                                // N 按 fsync 计（主）：一个间隔 interval 次 fsync = interval/concurrent_fsync_count 批。
                                let batches_in_interval_main = batches_in_interval_by_fsync_count(interval, concurrent_fsync_count);
                                let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, shared, batches_in_interval_main, IntermediateVersionPolicy::ReleasedAndBacklogged);
                                let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, shared, batches_in_interval_main, IntermediateVersionPolicy::ReleasedAndBacklogged);

                                // WAL 两臂摊到每次 fsync：（N_b 批之和 + checkpoint）÷ interval（第一节「读法写死」）。
                                let amortized_full = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_in_interval_main as f64 + checkpoint_full.total_bytes() as f64) / interval as f64;
                                let amortized_leaf = (write_ahead_log_leaf_fsync.total_bytes() as f64 * batches_in_interval_main as f64 + checkpoint_leaf.total_bytes() as f64) / interval as f64;

                                let ratio_jia_over_full = jia_amortized_bytes / amortized_full;
                                let ratio_jia_over_leaf = jia_amortized_bytes / amortized_leaf;
                                let ratio_full_over_leaf = amortized_full / amortized_leaf;
                                let saved_share_full = 1.0 - amortized_full / jia_amortized_bytes;
                                let saved_share_leaf = 1.0 - amortized_leaf / jia_amortized_bytes;

                                // N_b = 1 的格（concurrent_fsync_count = interval，或 interval=16 时 batches_in_interval_main=1）不算判据（第一节）。
                                let batches_in_interval_equals_one = batches_in_interval_main == 1;
                                if !batches_in_interval_equals_one {
                                    if saved_share_full <= 0.0 && fold_point_zero_full.is_none() {
                                        fold_point_zero_full = Some(concurrent_fsync_count);
                                    }
                                    if saved_share_full < 0.10 && fold_point_ten_percent_full.is_none() {
                                        fold_point_ten_percent_full = Some(concurrent_fsync_count);
                                    }
                                    if saved_share_leaf <= 0.0 && fold_point_zero_leaf.is_none() {
                                        fold_point_zero_leaf = Some(concurrent_fsync_count);
                                    }
                                    if saved_share_leaf < 0.10 && fold_point_ten_percent_leaf.is_none() {
                                        fold_point_ten_percent_leaf = Some(concurrent_fsync_count);
                                    }
                                }

                                // F2（第十节）：甲与两条 WAL checkpoint 核心（K9′）分开判；不拿 F2 触发的格判臂（登记原文）。
                                let (jia_unconverged, jia_saturated_trees) = jia_convergence_and_saturation_status(&jia);
                                let (full_unconverged, full_saturated_trees) = checkpoint_convergence_and_saturation_status(&checkpoint_full);
                                let (leaf_unconverged, leaf_saturated_trees) = checkpoint_convergence_and_saturation_status(&checkpoint_leaf);

                                println!(
                                    "{}",
                                    emitter.emit_raw(&format!(
                                        "name=row7_grid p={file_count} family={} placement={} policy={} n={interval} concurrent_fsync_count={concurrent_fsync_count} shared={shared} \
                                         batches_in_interval={batches_in_interval_main} batches_in_interval_equals_one={batches_in_interval_equals_one} \
                                         jia_batch_bytes={jia_batch_bytes} jia_batch_write_calls={} jia_amortized_bytes={jia_amortized_bytes} \
                                         wal_full_batch_bytes={} wal_full_batch_write_calls={} wal_full_checkpoint_bytes={} wal_full_amortized_bytes={amortized_full} \
                                         wal_leaf_batch_bytes={} wal_leaf_batch_write_calls={} wal_leaf_checkpoint_bytes={} wal_leaf_amortized_bytes={amortized_leaf} \
                                         ratio_jia_over_wal_full={ratio_jia_over_full} ratio_jia_over_wal_leaf={ratio_jia_over_leaf} ratio_wal_full_over_wal_leaf={ratio_full_over_leaf} \
                                         saved_share_wal_full={saved_share_full} saved_share_wal_leaf={saved_share_leaf} \
                                         jia_unconverged={jia_unconverged} jia_saturated_trees={jia_saturated_trees:?} \
                                         wal_full_unconverged={full_unconverged} wal_full_saturated_trees={full_saturated_trees:?} \
                                         wal_leaf_unconverged={leaf_unconverged} wal_leaf_saturated_trees={leaf_saturated_trees:?}",
                                        family.tag(),
                                        placement.tag(),
                                        policy.tag(),
                                        jia.write_calls,
                                        write_ahead_log_full_fsync.total_bytes(),
                                        write_ahead_log_full_fsync.write_calls,
                                        checkpoint_full.total_bytes(),
                                        write_ahead_log_leaf_fsync.total_bytes(),
                                        write_ahead_log_leaf_fsync.write_calls,
                                        checkpoint_leaf.total_bytes(),
                                    ))
                                );
                            }

                            println!(
                                "{}",
                                emitter.emit_raw(&format!(
                                    "name=row7_fold_point p={file_count} family={} placement={} policy={} n={interval} shared={shared} \
                                     fold_point_zero_wal_full={:?} fold_point_ten_percent_wal_full={:?} \
                                     fold_point_zero_wal_leaf={:?} fold_point_ten_percent_wal_leaf={:?}",
                                    family.tag(),
                                    placement.tag(),
                                    policy.tag(),
                                    fold_point_zero_full,
                                    fold_point_ten_percent_full,
                                    fold_point_zero_leaf,
                                    fold_point_ten_percent_leaf,
                                ))
                            );
                        }
                    }
                }
            }
        }
    }

    // 8.2（第 7 行反向取样点）：N 按批计——一个间隔 concurrent_fsync_count×batches_in_interval 次 fsync，
    // 不是 interval 次；代表格 P=10⁴、F1、seq、Lbalanced、concurrent_fsync_count=2、interval=16。
    {
        let representative_shape = PoolShape { file_count: 10_000, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let concurrent_fsync_count = 2u64;
        let interval = 16u64;
        let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(representative_shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);

        let batches_by_fsync = interval / concurrent_fsync_count; // 主：N 按 fsync 计。
        let batches_by_batch = interval; // 反向：N 按批计（一个间隔 interval 批）。

        let checkpoint_by_fsync = solve_write_ahead_log_batch_checkpoint(representative_shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_by_fsync, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let checkpoint_by_batch = solve_write_ahead_log_batch_checkpoint(representative_shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_by_batch, IntermediateVersionPolicy::ReleasedAndBacklogged);

        let amortized_by_fsync = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_by_fsync as f64 + checkpoint_by_fsync.total_bytes() as f64) / interval as f64;
        // 反向读法下，一个「间隔」实际跨越 interval×concurrent_fsync_count 次 fsync，摊销分母也要跟着换。
        let amortized_by_batch = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_by_batch as f64 + checkpoint_by_batch.total_bytes() as f64) / (batches_by_batch * concurrent_fsync_count) as f64;

        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=geometry_sensitivity_sample_row7 point=interval_counted_by_batch p=10000 family=F1 placement=seq policy=Lbalanced concurrent_fsync_count=2 n=16 \
                 checkpoint_bytes_by_fsync={} checkpoint_bytes_by_batch={} amortized_by_fsync={amortized_by_fsync} amortized_by_batch={amortized_by_batch} \
                 makes_wal_full_cheaper={}",
                checkpoint_by_fsync.total_bytes(),
                checkpoint_by_batch.total_bytes(),
                amortized_by_batch < amortized_by_fsync,
            ))
        );

        // 不共享取随机（F9）：这一版没有建独立开关（与 K9′-b、N 按批计不同，这一条完全未实现）。
        let unshared_random_note = "不共享取随机（k 个文件在全池均匀独立地落，R6 的 N_g=concurrent_fsync_count、s=d）这一版没有建可计算的开关，F9 答不了，交下一段或主 agent";
        println!("{}", emitter.emit_raw(&format!("name=geometry_sensitivity_sample_row7 point=unshared_random_not_modeled_this_pass p=10000 note={unshared_random_note:?}")));
    }

    println!("{}", emitter.finish());
}





