//! E159：fsync 等待时间随并发数——组提交（甲）与 WAL 两臂（wal_full-K10、乙-M-K10）。
//!
//! 判据、失败条款、变异表、停机条款的权威登记在 `research/prompts/e159-preregistration.md`；
//! 这一份只实现登记写死的东西，不重新论证。**这一次只做到正式计时之前**：装置、单测、钉绝对值
//! 的断言、变异表、停机条款 S1–S3、冒烟跑（跑前登记第五节 5.7 第一段的主网格、PC1–PC3、AA、
//! G8、G1、G6 五轮正式计时不在这一次范围内，机器上并发编译会污染计时，见交回报告）。
//!
//! ## 从 E155 拷来的部分（到「二、」小节末尾为止，一个字不改）
//!
//! 从 `research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs` 第 36–2058
//! 行原样拷来：常量、K1–K4 容量与树高、5.3 组公式、`solve_fixed_point_core`、甲 / WAL 两臂的单发
//! 与批量函数（含 `solve_jia_batch_publish`、`solve_write_ahead_log_batch_fsync`、
//! `solve_write_ahead_log_batch_checkpoint`），以及它们依赖的全部类型与函数。这些函数算得对不对
//! 归 E155 的锚点与变异表管；跑前登记停机条款 S2 要求这一段与源文件逐个 diff 为空。
//!
//! ## 这一次新写的部分
//!
//! 客体设备区域布局与 O_DIRECT I/O（`RealBlockIo`）、录制假设备（`RecordingBlockIo`，供
//! 不碰真 I/O 的结构性单测用）、睡眠假设备（`SleepingBlockIo`，供不需要真磁盘的时序结构单测
//! 用）、随机取整（`round_stochastic`）、三条臂的批计划构造（`plan_jia_batch` /
//! `plan_write_ahead_log_fsync_batch` / `plan_write_ahead_log_checkpoint`）、批的执行顺序
//! （`execute_batch_persist` / `execute_batch_finalize_system_configuration`）、组提交队列与
//! 发布者（`publisher_loop`）、WAL checkpoint 后台线程（`checkpoint_loop`）、闭环调用方
//! （`caller_loop`）、统计（分位数、规则 R、J1 / J2a / J4、Mann–Whitney A、PC1 的 Δ）、来宾块层
//! 读数、CLI 与 `main`。
//!
//! ## 已知的范围缺口（交回报告里也会写，供续派时看）
//!
//! - 开环到达（第五节 5.5 G8）只实现了 `t_issue` 取排定时刻这一条纯语义（`open_loop_t_issue`），
//!   没有接上完整的开环调用方池；G1–G8 的其余反向取样点、Q1–Q12 的完整统计与结果行、五轮合并
//!   都还没有实现——这些都在「正式计时」的范围内，这一次不做。
//! - journal 环记录数到上限时批次要等 checkpoint 做完（第五节 5.3、第八节表格「环里在飞记录数」）
//!   没有接上背压；这一次的取样量级碰不到 65536 条的上限，接上背压是续做时的欠账。
//! - 在飞 checkpoint 深度实现成单个顺序线程（峰值恒为 1），满足 A2「≤ 2」的上界但不是「最多
//!   两个」的字面实现；续做正式计时前要不要做成真并发由主 agent 定。

#![allow(dead_code)]

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
/// K11 反向取样点：内部条目再加身份引用 26 字节。
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

/// 内部节点条目宽：分隔 key + 子指针（K3 主几何）；K11 反向取样点再加身份引用 26 字节。
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
        assert_eq!(node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, true)), 119, "K11 extent");
        assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, true)), 133, "K11 分配记录");
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, true)), 121, "K11 记账");
        assert_eq!(node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, true)), 116, "K11 映射");
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

/// 分配记录树一块盘一层的期望脏节点数（R1/R3/R4 合并处理；K10 的中间版释放记录 R7(a) 只进尾部插入）。
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
    // + K10 中间版释放记录（R7(a)，恒在尾部，不参与远近判定）。
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
// 四、族与几何：一格的输入（与第一次逐字相同，加 K10 开关的实际接线）
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
/// `intermediate_version_policy`（K9 / K10）不放在这里——它只对 WAL checkpoint 有意义，
/// 是 `solve_write_ahead_log_checkpoint` 的独立参数（见 `IntermediateVersionPolicy`）。
#[derive(Clone, Copy, Debug)]
struct Geometry {
    fill_ratio: FillRatio,
    identity_reference: bool, // K3（false）/ K11（true）
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

/// K9 / K10（岔路单第 6 行，登记 5.1 R7）：间隔内被换掉的中间版要不要各自补一条已释放的分配记录、
/// 进 K6 积压。只对 WAL checkpoint 有意义——甲每次 fsync 就是一次发布，没有中间版。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum IntermediateVersionPolicy {
    /// K9（第一次登记）：中间版不写分配记录、不进映射、不进积压。
    NotTracked,
    /// K10（这一次登记）：每个中间版每盘写一条已释放的分配记录，(a) 并进分配记录树区间尾部、
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
/// `extra_allocation_release_insertions`：K10（R7）每个中间版每盘写一条已释放的分配记录，
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

        // R1（第三节 3.2 ①）：K6 积压的「每次持久化释放的单元数」要含数据单元；K10（R7(b)）再加中间版 I。
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
        0.0, // 甲没有中间版（K10 只对 WAL 两臂有意义）。
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

/// 第 7 行：每次 fsync 摊到的写字节 / 写请求（甲 = 一批的写 ÷ concurrent_fsync_count；M16 的反例：不除以 concurrent_fsync_count）。
fn amortize_batch_per_fsync(batch_value: u64, concurrent_fsync_count: u64) -> f64 {
    batch_value as f64 / concurrent_fsync_count as f64
}

/// 第 7 行：checkpoint 间隔 N 按 fsync 计——一个间隔 N 次 fsync = N/concurrent_fsync_count 批
/// （第一节「读法写死」；M13 的反例：间隔按批计，直接把 N 当批数）。
fn batches_in_interval_by_fsync_count(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    interval_fsyncs / concurrent_fsync_count
}

/// R13（第四次跑登记第一节「N_b 在 k > N 时怎么算」）：k ≤ N 时 N_b = N/concurrent_fsync_count；
/// k > N 时一批已经把间隔撑满，N_b = 1（B-r4-9）。M1 的变异目标。
fn batches_in_interval_for_concurrent_fsync_count(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    if concurrent_fsync_count > interval_fsyncs {
        1
    } else {
        interval_fsyncs / concurrent_fsync_count
    }
}

/// R13：k ≤ N 时间隔实际 fsync 数 = N（摊销分母不变）；k > N 时间隔实际 fsync 数 = concurrent_fsync_count，
/// 摊销分母跟着从 N 换成 concurrent_fsync_count（B-r4-9）。M6 的变异目标。
fn effective_fsyncs_in_interval(interval_fsyncs: u64, concurrent_fsync_count: u64) -> u64 {
    if concurrent_fsync_count > interval_fsyncs {
        concurrent_fsync_count
    } else {
        interval_fsyncs
    }
}

/// Q7.16：某一层「不共享脏节点 = n_l」即饱和，返回饱和的层号（从 0 开始，叶层 = 0）。
/// `node_count == 1` 的层（多数树的根）排除在外：单节点层不管碰没碰都必然「脏节点 = n_l」，
/// 这是树结构本身的平凡事实，跟不共享路径数够不够多没有关系，不算这一条要判的「饱和」
/// （实测：P=10⁴、F1、k=1024，共享与不共享的 extent 根层都会命中这个平凡情形，
/// 若不排除，「共享不该饱和」这条阳性对照会被平凡命中一起打红）。
fn saturated_layer_indices(layers: &[u64], dirty: &[f64]) -> Vec<usize> {
    layers
        .iter()
        .zip(dirty.iter())
        .enumerate()
        .filter_map(|(layer_index, (&node_count, &dirty_value))| if node_count > 1 && (dirty_value - node_count as f64).abs() < 1e-9 { Some(layer_index) } else { None })
        .collect()
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
// 六、5.4 wal_full 与乙-M（K10，第 6 行三臂）
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
/// + K10（R7）中间版释放记录的支撑量（Q6.6/Q6.9/B11）。
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

/// WAL checkpoint 结果上的 F2 判定（第 6 行 wal_full / 乙-M 两臂，K9 与 K10 都要看）。
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
// ============================================================
// E159 新增（跑前登记 research/prompts/e159-preregistration.md 第五节）：
// 客体设备布局、随机取整、批发布引擎、组提交与 WAL checkpoint、计时统计、CLI。
// 上面拷自 E155 的部分到此为止，一个字不改（登记第十一节停机条款 S2）。
// ============================================================

use e7_index_bench::Emitter;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const O_DIRECT: i32 = 0o40000; // naming-lint:external Linux open(2) 标志名
const ALIGNMENT_BYTES: usize = 4096;

// ---- 5.3 区域布局 ----
const CONFIGURATION_REGION_OFFSET_BYTES: u64 = 0;
const SYSTEM_CONFIGURATION_SLOT_ZERO_OFFSET_BYTES: u64 = 0;
const SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET_BYTES: u64 = SYSTEM_CONFIGURATION_SLOT_BYTES;
const ROOT_SLOT_BASE_OFFSET_BYTES: u64 = 65536; // 64 KiB，留在 [0, 1 MiB) 区内、避开两个系统配置槽
const ROOT_SLOT_STRIDE_BYTES: u64 = 16384;
const ROOT_SLOT_COUNT: u64 = 24;
const JOURNAL_REGION_OFFSET_BYTES: u64 = 16 * 1024 * 1024;
const JOURNAL_REGION_BYTES: u64 = RING_DEFAULT_BYTES;
const UNIT_REGION_OFFSET_BYTES: u64 = 1024 * 1024 * 1024;
const UNIT_REGION_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// 只回放一块盘（第五节 5.3「回放哪块盘」）：模型算出的是两盘合计，除以这个数拿单盘量。
const REPLAY_DEVICE_COUNT: u64 = 1;

/// journal 环里在飞记录数上限：768 MiB ÷ 4096 ÷ F（F=3，A1 锚点）。
const JOURNAL_RING_RECORD_LIMIT: u64 = RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN;

// ============================================================
// 对齐缓冲与 O_DIRECT（模式照 e53_ring_failure.rs 的 AlignedBuffer / O_DIRECT 写法）
// ============================================================

struct AlignedBuffer {
    pointer: *mut u8,
    length_in_bytes: usize,
    allocation_layout: Layout,
}
impl AlignedBuffer {
    fn new_with_seed(length_in_bytes: usize, seed: u64) -> Self {
        let allocation_layout = Layout::from_size_align(length_in_bytes, ALIGNMENT_BYTES).unwrap();
        let pointer = unsafe { alloc(allocation_layout) };
        let mut state = seed;
        let mut written = 0usize;
        let slice = unsafe { std::slice::from_raw_parts_mut(pointer, length_in_bytes) };
        while written < length_in_bytes {
            let word = splitmix64(&mut state).to_le_bytes();
            let take = word.len().min(length_in_bytes - written);
            slice[written..written + take].copy_from_slice(&word[..take]);
            written += take;
        }
        AlignedBuffer { pointer, length_in_bytes, allocation_layout }
    }
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.length_in_bytes) }
    }
    /// O_DIRECT 的读必须把数据读进一块按 `ALIGNMENT_BYTES` 对齐的缓冲区，
    /// 不能读进普通的 `Vec`（`RealBlockDevice::read_at` 的 M9/S1 之外那个真实 bug：
    /// 2026-09-24 冒烟跑在虚机里第一次跑到 `read_at` 时炸出 EINVAL，
    /// 因为当时读进的是 `vec![0u8; length]`——那不是对齐缓冲区）。
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.pointer, self.length_in_bytes) }
    }
}
impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer, self.allocation_layout) }
    }
}
unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// [0, 1) 上的均匀变量，用来做「整数部分 + 以小数部分为概率再加 1」的随机取整（第五节 5.3）。
fn uniform_unit_interval(state: &mut u64) -> f64 {
    (splitmix64(state) >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}


// ============================================================
// 块设备抽象：真设备（O_DIRECT）与录制假设备（单测用，不碰 I/O）
// ============================================================

/// 一步操作的记号，供 S1（次序对拍）与 M1/M2/M9/M10 的单测直接断言。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepMark {
    Write { kind: UnitKind },
    Barrier,
    Read { kind: UnitKind },
}

trait BlockDevice: Send + Sync {
    fn write_at(&self, offset: u64, buffer: &[u8], kind: UnitKind);
    fn read_at(&self, offset: u64, length: usize, kind: UnitKind) -> Vec<u8>;
    fn barrier(&self);
}

struct RealBlockDevice {
    file: File,
}
impl RealBlockDevice {
    fn open(path: &str) -> std::io::Result<RealBlockDevice> {
        let file = OpenOptions::new().read(true).write(true).custom_flags(O_DIRECT).open(path)?;
        Ok(RealBlockDevice { file })
    }
}
impl BlockDevice for RealBlockDevice {
    fn write_at(&self, offset: u64, buffer: &[u8], _kind: UnitKind) {
        self.file.write_at(buffer, offset).expect("write_at 失败：设备或对齐有问题");
    }
    fn read_at(&self, offset: u64, length: usize, _kind: UnitKind) -> Vec<u8> {
        let mut buffer = AlignedBuffer::new_with_seed(length, 0);
        self.file.read_exact_at(buffer.as_mut(), offset).expect("read_at 失败：设备或对齐有问题");
        buffer.as_ref().to_vec()
    }
    fn barrier(&self) {
        self.file.sync_data().expect("sync_data 失败");
    }
}

/// 单测用：不碰真 I/O，只把每一步记下来，供 S1、M1、M2、M9、M10 断言用（`mutation-sampling.md`
/// 第四类要求单测取样点覆盖实验会跑到的取值——这份记录只覆盖控制流与调用顺序，不覆盖真实时延，
/// 时延相关的 M3/M4/M6 由 `SleepingBlockDevice`（见下）在真线程真时钟上跑，不需要真设备）。
struct RecordingBlockDevice {
    log: Mutex<Vec<(Instant, StepMark)>>,
}
impl RecordingBlockDevice {
    fn new() -> RecordingBlockDevice {
        RecordingBlockDevice { log: Mutex::new(Vec::new()) }
    }
    fn drain(&self) -> Vec<(Instant, StepMark)> {
        std::mem::take(&mut self.log.lock().unwrap())
    }
    fn drain_marks(&self) -> Vec<StepMark> {
        self.drain().into_iter().map(|(_, mark)| mark).collect()
    }
}
impl BlockDevice for RecordingBlockDevice {
    fn write_at(&self, _offset: u64, _buffer: &[u8], kind: UnitKind) {
        self.log.lock().unwrap().push((Instant::now(), StepMark::Write { kind }));
    }
    fn read_at(&self, _offset: u64, length: usize, kind: UnitKind) -> Vec<u8> {
        self.log.lock().unwrap().push((Instant::now(), StepMark::Read { kind }));
        vec![0u8; length]
    }
    fn barrier(&self) {
        self.log.lock().unwrap().push((Instant::now(), StepMark::Barrier));
    }
}

/// 单测用：不碰真设备，但用真线程 + 真 `Instant`；每次 `barrier()` 睡一段固定时长模拟持久化延迟，
/// 让 PC1/PC2/PC3 那一类「结构对不对」的检查能在宿主上跑，不必起虚机（M3/M4/M5/M6 的结构半用它测；
/// 真实设备上的量级仍旧只能在虚机里量，见跑前登记第九节）。
struct SleepingBlockDevice {
    barrier_delay: Duration,
}
impl BlockDevice for SleepingBlockDevice {
    fn write_at(&self, _offset: u64, _buffer: &[u8], _kind: UnitKind) {}
    fn read_at(&self, _offset: u64, length: usize, _kind: UnitKind) -> Vec<u8> {
        vec![0u8; length]
    }
    fn barrier(&self) {
        std::thread::sleep(self.barrier_delay);
    }
}

// ============================================================
// 单元种类、随机取整、批计划
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum UnitKind {
    Data,
    ExtentNode,
    InodeContainer,
    InodeRootNode,
    AllocationNode,
    AccountingNode,
    MappingNode,
    TreeTableNode,
    Record,
    RootSlot,
    SystemConfigurationSlot,
}
impl UnitKind {
    fn width_bytes(self) -> u64 {
        match self {
            UnitKind::Data | UnitKind::InodeContainer => UNIT_BYTES,
            UnitKind::ExtentNode | UnitKind::InodeRootNode | UnitKind::AllocationNode | UnitKind::AccountingNode | UnitKind::MappingNode | UnitKind::TreeTableNode => NODE_BYTES,
            UnitKind::Record => RECORD_BYTES,
            UnitKind::RootSlot => PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES,
            UnitKind::SystemConfigurationSlot => SYSTEM_CONFIGURATION_SLOT_BYTES,
        }
    }
    /// 落在哪个区域（5.3）：单元区还是 journal 环。根槽与系统配置槽走各自专用的槽位，不走游标。
    fn region(self) -> Region {
        match self {
            UnitKind::Record => Region::Journal,
            UnitKind::RootSlot | UnitKind::SystemConfigurationSlot => Region::Configuration,
            _ => Region::Unit,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Region {
    Unit,
    Journal,
    Configuration,
}

/// 期望个数 = 这一种的字节（两盘合计）÷ REPLAY_DEVICE_COUNT ÷ 单元宽（第五节 5.3）。
fn expected_count_from_both_device_bytes(both_device_bytes: u64, unit_width_bytes: u64) -> f64 {
    let single_device_bytes = both_device_bytes / DEVICE_COUNT; // 整除：模型自己乘过 DEVICE_COUNT
    (single_device_bytes as f64) / (unit_width_bytes as f64) * (REPLAY_DEVICE_COUNT as f64)
}

/// 随机取整：整数部分 + 以小数部分为概率再加 1（第五节 5.3；M7 的变异目标）。
fn round_stochastic(expected: f64, random_state: &mut u64) -> u64 {
    assert!(expected >= 0.0, "期望个数不该是负的：{expected}");
    let floor_value = expected.floor();
    let fractional_part = expected - floor_value;
    let draw = uniform_unit_interval(random_state);
    floor_value as u64 + u64::from(draw < fractional_part)
}


/// 一批要写的东西：每种单元的实际个数（已随机取整）+ 记录数 + 写不写根槽 / 系统配置槽。
#[derive(Clone, Debug, Default)]
struct BatchPlan {
    unit_counts: Vec<(UnitKind, u64)>,
    record_count: u64,
    writes_root_slot: bool,
    writes_system_configuration_slot: bool,
}
impl BatchPlan {
    /// W：这一批实际写出、且被记录点名的单元总数（不含记录、根槽、系统配置槽自己）。
    fn named_unit_total(&self) -> u64 {
        self.unit_counts.iter().map(|&(_, count)| count).sum()
    }
    fn total_write_calls(&self) -> u64 {
        self.unit_counts.iter().map(|&(_, count)| count).sum::<u64>() + self.record_count + u64::from(self.writes_root_slot) + u64::from(self.writes_system_configuration_slot)
    }
}

/// 三条候选臂（第二节 2.3、第五节 5.2）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Jia,
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}
impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::Jia => "jia",
            Arm::WriteAheadLogFull => "wal_full-K10",
            Arm::WriteAheadLogLeaf => "yi-M-K10",
        }
    }
    fn write_ahead_log_arm(self) -> Option<WriteAheadLogArm> {
        match self {
            Arm::Jia => None,
            Arm::WriteAheadLogFull => Some(WriteAheadLogArm::WriteAheadLogFull),
            Arm::WriteAheadLogLeaf => Some(WriteAheadLogArm::WriteAheadLogLeaf),
        }
    }
}

/// 主几何（第五节 5.1）：P=10⁴、F8A、共享脊柱、seq、L均、φ=1、K3、g=24、pbs=512。
fn main_shape(file_count: u64, family: Family) -> PoolShape {
    PoolShape { file_count, family, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }
}

/// 甲一批的计划（第五节 5.2「怎么做」+ 5.3「取整」）。
fn plan_jia_batch(shape: PoolShape, concurrent_fsync_count: u64, shared: bool, random_state: &mut u64) -> BatchPlan {
    let outcome = solve_jia_batch_publish(shape, concurrent_fsync_count, shared);
    let fields: [(UnitKind, u64); 8] = [
        (UnitKind::Data, outcome.data_bytes),
        (UnitKind::ExtentNode, outcome.extent_bytes),
        (UnitKind::InodeContainer, outcome.inode_container_bytes),
        (UnitKind::InodeRootNode, outcome.inode_root_bytes),
        (UnitKind::AllocationNode, outcome.allocation_bytes),
        (UnitKind::AccountingNode, outcome.accounting_bytes),
        (UnitKind::MappingNode, outcome.mapping_bytes),
        (UnitKind::TreeTableNode, outcome.tree_table_bytes),
    ];
    let mut unit_counts = Vec::with_capacity(8);
    for (kind, both_device_bytes) in fields {
        let expected = expected_count_from_both_device_bytes(both_device_bytes, kind.width_bytes());
        unit_counts.push((kind, round_stochastic(expected, random_state)));
    }
    let named_unit_total: u64 = unit_counts.iter().map(|&(_, count)| count).sum();
    let record_count = named_unit_total.max(1).div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    BatchPlan { unit_counts, record_count, writes_root_slot: true, writes_system_configuration_slot: true }
}

/// WAL 两臂一批 fsync 的计划（不发根、不写固定点；第五节 5.2）。
fn plan_write_ahead_log_fsync_batch(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool, random_state: &mut u64) -> BatchPlan {
    let outcome = solve_write_ahead_log_batch_fsync(shape, arm, concurrent_fsync_count, shared);
    let fields: [(UnitKind, u64); 3] = [(UnitKind::Data, outcome.data_bytes), (UnitKind::ExtentNode, outcome.extent_bytes), (UnitKind::InodeContainer, outcome.inode_container_bytes)];
    let mut unit_counts = Vec::with_capacity(4);
    for (kind, both_device_bytes) in fields {
        let expected = expected_count_from_both_device_bytes(both_device_bytes, kind.width_bytes());
        unit_counts.push((kind, round_stochastic(expected, random_state)));
    }
    let record_count = match arm {
        // wal_full 恒点名两棵新树根，不随 k 变（第五节 5.3；M13 之外另一条不变量，未在变异表登记，
        // 因为它是臂定义本身，见第五节 5.2「wal_full-K10」）。
        WriteAheadLogArm::WriteAheadLogFull => {
            let inode_root_expected = expected_count_from_both_device_bytes(outcome.inode_root_bytes, UnitKind::InodeRootNode.width_bytes());
            let inode_root_actual = round_stochastic(inode_root_expected, random_state);
            if inode_root_actual > 0 {
                unit_counts.push((UnitKind::InodeRootNode, inode_root_actual));
            }
            1
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let named_unit_total: u64 = unit_counts.iter().map(|&(_, count)| count).sum();
            named_unit_total.max(1).div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1)
        }
    };
    BatchPlan { unit_counts, record_count, writes_root_slot: false, writes_system_configuration_slot: false }
}

/// WAL 两臂一次 checkpoint 的计划（第五节 5.2；发根、写四样固定点）。
#[allow(clippy::too_many_arguments)]
fn plan_write_ahead_log_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool, batches_in_interval: u64, policy: IntermediateVersionPolicy, random_state: &mut u64) -> BatchPlan {
    let outcome = solve_write_ahead_log_batch_checkpoint(shape, arm, concurrent_fsync_count, shared, batches_in_interval, policy);
    let fields: [(UnitKind, u64); 6] = [
        (UnitKind::ExtentNode, outcome.extent_catch_up_bytes),
        (UnitKind::InodeRootNode, outcome.inode_catch_up_bytes),
        (UnitKind::AllocationNode, outcome.allocation_bytes),
        (UnitKind::AccountingNode, outcome.accounting_bytes),
        (UnitKind::MappingNode, outcome.mapping_bytes),
        (UnitKind::TreeTableNode, outcome.tree_table_bytes),
    ];
    let mut unit_counts = Vec::with_capacity(6);
    for (kind, both_device_bytes) in fields {
        let expected = expected_count_from_both_device_bytes(both_device_bytes, kind.width_bytes());
        unit_counts.push((kind, round_stochastic(expected, random_state)));
    }
    let named_unit_total: u64 = unit_counts.iter().map(|&(_, count)| count).sum();
    let record_count = named_unit_total.max(1).div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    BatchPlan { unit_counts, record_count, writes_root_slot: true, writes_system_configuration_slot: true }
}

// ============================================================
// 预生成缓冲、区域游标、批的执行（单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → [放行] → 系统配置槽）
// ============================================================

/// 每种单元宽度各预生成一块固定伪随机内容的缓冲，写的时候共享只读，不逐次现生成
/// （第五节 5.3「内容是固定种子生成的伪随机字节」；内容不需要逐次唯一）。
struct UnitBuffers {
    data: AlignedBuffer,
    node: AlignedBuffer,
    record: AlignedBuffer,
    root_slot: AlignedBuffer,
    system_configuration_slot: AlignedBuffer,
}
impl UnitBuffers {
    fn new(seed: u64) -> UnitBuffers {
        UnitBuffers {
            data: AlignedBuffer::new_with_seed(UNIT_BYTES as usize, seed ^ 1),
            node: AlignedBuffer::new_with_seed(NODE_BYTES as usize, seed ^ 2),
            record: AlignedBuffer::new_with_seed(RECORD_BYTES as usize, seed ^ 3),
            root_slot: AlignedBuffer::new_with_seed(PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES as usize, seed ^ 4),
            system_configuration_slot: AlignedBuffer::new_with_seed(SYSTEM_CONFIGURATION_SLOT_BYTES as usize, seed ^ 5),
        }
    }
    fn buffer_for(&self, kind: UnitKind) -> &[u8] {
        match kind {
            UnitKind::Data | UnitKind::InodeContainer => self.data.as_ref(),
            UnitKind::ExtentNode | UnitKind::InodeRootNode | UnitKind::AllocationNode | UnitKind::AccountingNode | UnitKind::MappingNode | UnitKind::TreeTableNode => self.node.as_ref(),
            UnitKind::Record => self.record.as_ref(),
            UnitKind::RootSlot => self.root_slot.as_ref(),
            UnitKind::SystemConfigurationSlot => self.system_configuration_slot.as_ref(),
        }
    }
}

/// 单元区、journal 环各一个顺序游标，写满回绕（第五节 5.3）；根槽按发布序号轮着用 24 个位置；
/// 系统配置槽在两个物理槽之间交替。
struct RegionCursors {
    unit_cursor_bytes: AtomicU64,
    journal_cursor_bytes: AtomicU64,
    publish_counter: AtomicU64,
    system_configuration_next_slot: AtomicU64,
}
impl RegionCursors {
    fn new() -> RegionCursors {
        RegionCursors { unit_cursor_bytes: AtomicU64::new(0), journal_cursor_bytes: AtomicU64::new(0), publish_counter: AtomicU64::new(0), system_configuration_next_slot: AtomicU64::new(0) }
    }
    fn claim_unit_offset(&self, width_bytes: u64) -> u64 {
        let raw = self.unit_cursor_bytes.fetch_add(width_bytes, Ordering::Relaxed);
        UNIT_REGION_OFFSET_BYTES + raw % UNIT_REGION_BYTES
    }
    fn claim_journal_offset(&self, width_bytes: u64) -> u64 {
        let raw = self.journal_cursor_bytes.fetch_add(width_bytes, Ordering::Relaxed);
        JOURNAL_REGION_OFFSET_BYTES + raw % JOURNAL_REGION_BYTES
    }
    fn claim_root_slot_offset(&self) -> u64 {
        let index = self.publish_counter.fetch_add(1, Ordering::Relaxed) % ROOT_SLOT_COUNT;
        ROOT_SLOT_BASE_OFFSET_BYTES + index * ROOT_SLOT_STRIDE_BYTES
    }
    /// 系统配置槽：先读两个槽（生成号语义不需要，只为读次数与次序对拍），再写「下一个」那个。
    fn system_configuration_offsets(&self) -> (u64, u64, u64) {
        let next = self.system_configuration_next_slot.fetch_add(1, Ordering::Relaxed) % 2;
        let write_offset = CONFIGURATION_REGION_OFFSET_BYTES + if next == 0 { SYSTEM_CONFIGURATION_SLOT_ZERO_OFFSET_BYTES } else { SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET_BYTES };
        (CONFIGURATION_REGION_OFFSET_BYTES + SYSTEM_CONFIGURATION_SLOT_ZERO_OFFSET_BYTES, CONFIGURATION_REGION_OFFSET_BYTES + SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET_BYTES, write_offset)
    }
}

/// 一批展开成的写任务：offset 已经从游标领到，kind 决定用哪块缓冲、算不算「持久点」的一部分。
struct WriteJob {
    offset: u64,
    kind: UnitKind,
}

fn expand_jobs(plan: &BatchPlan, cursors: &RegionCursors) -> Vec<WriteJob> {
    let mut jobs = Vec::with_capacity(plan.unit_counts.iter().map(|&(_, count)| count).sum::<u64>() as usize);
    for &(kind, count) in &plan.unit_counts {
        for _ in 0..count {
            jobs.push(WriteJob { offset: cursors.claim_unit_offset(kind.width_bytes()), kind });
        }
    }
    jobs
}

fn expand_record_jobs(plan: &BatchPlan, cursors: &RegionCursors) -> Vec<WriteJob> {
    (0..plan.record_count).map(|_| WriteJob { offset: cursors.claim_journal_offset(RECORD_BYTES), kind: UnitKind::Record }).collect()
}

/// 把一批写任务并发交给最多 `worker_count` 个线程，全部完成才返回（第五节 5.3「同一阶段的写并发交出去」）。
fn execute_phase_parallel(device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, jobs: &[WriteJob], worker_count: usize) {
    if jobs.is_empty() {
        return;
    }
    let chunk_size = jobs.len().div_ceil(worker_count.max(1));
    std::thread::scope(|scope| {
        for chunk in jobs.chunks(chunk_size) {
            scope.spawn(move || {
                for job in chunk {
                    device.write_at(job.offset, buffers.buffer_for(job.kind), job.kind);
                }
            });
        }
    });
}

/// 持久点之前的全部步骤：单元 → 屏障 → 记录 → 屏障 → （甲/checkpoint）根槽写 + FUA。
/// 返回 made_durable_at。WAL 一批 fsync（`writes_root_slot=false`）在第二道屏障之后就是 made_durable_at。
fn execute_batch_persist(device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, cursors: &RegionCursors, plan: &BatchPlan, worker_count: usize) -> Instant {
    let unit_jobs = expand_jobs(plan, cursors);
    execute_phase_parallel(device, buffers, &unit_jobs, worker_count);
    device.barrier();
    // M2 的变异目标：把返回值换成这个早期时刻，就是「在第一道屏障之后就放行，不等根槽」。
    let after_first_barrier = Instant::now();
    let record_jobs = expand_record_jobs(plan, cursors);
    execute_phase_parallel(device, buffers, &record_jobs, worker_count);
    device.barrier();
    if plan.writes_root_slot {
        let root_offset = cursors.claim_root_slot_offset();
        device.write_at(root_offset, buffers.buffer_for(UnitKind::RootSlot), UnitKind::RootSlot);
        device.barrier(); // FUA = 写完立刻 sync_data（`block_device.rs:519-520`，第三节抄）
    }
    let made_durable_at = Instant::now();
    debug_assert!(made_durable_at >= after_first_barrier);
    made_durable_at
}

/// 持久点、放行调用方之后才做的一步：系统配置槽先读两个槽、再写一个（第五节 5.2）。
fn execute_batch_finalize_system_configuration(device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, cursors: &RegionCursors, plan: &BatchPlan) {
    if !plan.writes_system_configuration_slot {
        return;
    }
    let (slot_zero, slot_one, write_offset) = cursors.system_configuration_offsets();
    let _ = device.read_at(slot_zero, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, UnitKind::SystemConfigurationSlot);
    let _ = device.read_at(slot_one, SYSTEM_CONFIGURATION_SLOT_BYTES as usize, UnitKind::SystemConfigurationSlot);
    device.write_at(write_offset, buffers.buffer_for(UnitKind::SystemConfigurationSlot), UnitKind::SystemConfigurationSlot);
}


impl BatchPlan {
    /// 这一批实际会往回放盘上写的字节（单盘；供 S4 写量预算与 Q9 的 journal 字节计数用）。
    fn approximate_bytes_single_device(&self) -> u64 {
        let unit_bytes: u64 = self.unit_counts.iter().map(|&(kind, count)| kind.width_bytes() * count).sum();
        let record_bytes = self.record_count * RECORD_BYTES;
        let root_bytes = if self.writes_root_slot { PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES } else { 0 };
        let system_configuration_bytes = if self.writes_system_configuration_slot { SYSTEM_CONFIGURATION_SLOT_BYTES } else { 0 };
        unit_bytes + record_bytes + root_bytes + system_configuration_bytes
    }
}

fn busy_wait(duration: Duration) {
    let start = Instant::now();
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}

// ============================================================
// 组提交队列、WAL checkpoint 触发、闭环调用方与发布者（第五节 5.2、5.5、5.6）
// ============================================================

struct QueueEntry {
    reply: mpsc::Sender<(Instant, Instant, Instant)>, // (batch_started_at, made_durable_at, released_at)
}

struct SharedQueue {
    entries: Mutex<VecDeque<QueueEntry>>,
    condvar: Condvar,
    stop: std::sync::atomic::AtomicBool,
}
impl SharedQueue {
    fn new() -> SharedQueue {
        SharedQueue { entries: Mutex::new(VecDeque::new()), condvar: Condvar::new(), stop: std::sync::atomic::AtomicBool::new(false) }
    }
}

struct WriteAheadLogAccounting {
    bytes_since_checkpoint: AtomicU64,
    fsync_count_since_cut: AtomicU64,
    batch_count_since_cut: AtomicU64,
    last_checkpoint_instant: Mutex<Instant>,
    checkpoint_count: AtomicU64,
    checkpoint_bytes_total: AtomicU64,
    stop: std::sync::atomic::AtomicBool,
}
impl WriteAheadLogAccounting {
    fn new() -> WriteAheadLogAccounting {
        WriteAheadLogAccounting {
            bytes_since_checkpoint: AtomicU64::new(0),
            fsync_count_since_cut: AtomicU64::new(0),
            batch_count_since_cut: AtomicU64::new(0),
            last_checkpoint_instant: Mutex::new(Instant::now()),
            checkpoint_count: AtomicU64::new(0),
            checkpoint_bytes_total: AtomicU64::new(0),
            stop: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// `T_time` 默认值（D16 已定项 5，第二节 2.2 第二块）：5 s。装置里唯一的权威落点——
/// `run_cell_subcommand` 与真格测试都读这个常量，不各自写一份字面量。
/// M8 的变异目标：读错成 50 s 会让主网格 10 s 窗口里 0 次触发（跑前登记第十一节 V10）。
const CHECKPOINT_TIME_THRESHOLD: Duration = Duration::from_millis(5000);

/// WAL checkpoint 的触发判据（D16 已定项 2 抄，第二节 2.2 第一块）。
fn should_trigger_checkpoint(now: Instant, last_checkpoint: Instant, bytes_since_checkpoint: u64, checkpoint_time_threshold: Duration, effective_dirty_bytes_threshold: u64) -> bool {
    now.duration_since(last_checkpoint) >= checkpoint_time_threshold || bytes_since_checkpoint >= effective_dirty_bytes_threshold
}

#[derive(Clone, Copy, Debug)]
struct SampleRecord {
    issued_at: Instant,
    batch_started_at: Instant,
    made_durable_at: Instant,
    released_at: Instant,
    returned_at: Instant,
    batch_size: u64,
}
impl SampleRecord {
    fn wait_nanos(&self) -> u64 {
        self.returned_at.duration_since(self.issued_at).as_nanos() as u64
    }
    /// 「放行到醒」这一段（M14 的变异目标：`returned_at` 若在发布者那边、持久点时刻取，这一段恒为 0）。
    fn release_to_wake_nanos(&self) -> u64 {
        self.returned_at.saturating_duration_since(self.released_at).as_nanos() as u64
    }
    /// V5 包含自检：`issued_at <= batch_started_at <= made_durable_at <= released_at <= returned_at`。
    fn violates_containment(&self) -> bool {
        !(self.issued_at <= self.batch_started_at && self.batch_started_at <= self.made_durable_at && self.made_durable_at <= self.released_at && self.released_at <= self.returned_at)
    }
}

/// 一批从队列里怎么收（第五节 5.2「组提交」）：组提交开 ⇒ 把队列现有的全部请求收成一批；
/// 组提交关 ⇒ 恰好收 1 个。M4 的变异目标：把「开」那一支也换成只取 1 个（组提交名存实亡）。
fn drain_batch_from_queue(queue: &mut VecDeque<QueueEntry>, group_commit_enabled: bool) -> Vec<QueueEntry> {
    if group_commit_enabled {
        queue.drain(..).collect()
    } else {
        let mut one = VecDeque::new();
        if let Some(entry) = queue.pop_front() {
            one.push_back(entry);
        }
        one.into()
    }
}

/// 一批交给哪个臂的规划函数（第五节 5.2「怎么做」）。M3 的变异目标：把某个臂的分支换成
/// 别的臂的规划函数（「臂写混」）——三条臂各自的批必须只由自己的规划函数产出。
fn plan_batch_for_arm(arm: Arm, shape: PoolShape, batch_size: u64, shared: bool, random_state: &mut u64) -> BatchPlan {
    match arm {
        Arm::Jia => plan_jia_batch(shape, batch_size, shared, random_state),
        Arm::WriteAheadLogFull => plan_write_ahead_log_fsync_batch(shape, WriteAheadLogArm::WriteAheadLogFull, batch_size, shared, random_state),
        Arm::WriteAheadLogLeaf => plan_write_ahead_log_fsync_batch(shape, WriteAheadLogArm::WriteAheadLogLeaf, batch_size, shared, random_state),
    }
}

#[allow(clippy::too_many_arguments)]
fn publisher_loop(
    queue: &SharedQueue,
    group_commit_enabled: bool,
    device: &(dyn BlockDevice + Sync),
    buffers: &UnitBuffers,
    cursors: &RegionCursors,
    shape: PoolShape,
    arm: Arm,
    shared: bool,
    worker_count: usize,
    pc1_inject: Option<Duration>,
    random_state: &Mutex<u64>,
    write_ahead_log_accounting: Option<&WriteAheadLogAccounting>,
    batch_index_counter: &AtomicU64,
    bytes_written_total: &AtomicU64,
) {
    loop {
        let batch: Vec<QueueEntry> = {
            let mut guard = queue.entries.lock().unwrap();
            loop {
                if !guard.is_empty() {
                    break;
                }
                if queue.stop.load(Ordering::SeqCst) {
                    return;
                }
                let (next_guard, _timeout) = queue.condvar.wait_timeout(guard, Duration::from_millis(5)).unwrap();
                guard = next_guard;
            }
            drain_batch_from_queue(&mut guard, group_commit_enabled)
        };
        if batch.is_empty() {
            continue;
        }
        let batch_started_at = Instant::now();
        let batch_size = batch.len() as u64;
        let plan = {
            let mut state = random_state.lock().unwrap();
            plan_batch_for_arm(arm, shape, batch_size, shared, &mut state)
        };
        let batch_index = batch_index_counter.fetch_add(1, Ordering::Relaxed);
        let made_durable_at = execute_batch_persist(device, buffers, cursors, &plan, worker_count);
        if let Some(inject) = pc1_inject {
            if batch_index % 2 == 0 {
                busy_wait(inject);
            }
        }
        let released_at = Instant::now();
        for entry in &batch {
            let _ = entry.reply.send((batch_started_at, made_durable_at, released_at));
        }
        execute_batch_finalize_system_configuration(device, buffers, cursors, &plan);
        bytes_written_total.fetch_add(plan.approximate_bytes_single_device(), Ordering::Relaxed);
        if let Some(accounting) = write_ahead_log_accounting {
            accounting.fsync_count_since_cut.fetch_add(batch_size, Ordering::Relaxed);
            accounting.batch_count_since_cut.fetch_add(1, Ordering::Relaxed);
            accounting.bytes_since_checkpoint.fetch_add(plan.approximate_bytes_single_device(), Ordering::Relaxed);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_loop(accounting: &WriteAheadLogAccounting, device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, cursors: &RegionCursors, shape: PoolShape, write_ahead_log_arm: WriteAheadLogArm, shared: bool, worker_count: usize, checkpoint_time_threshold: Duration, effective_dirty_bytes_threshold: u64, random_state: &Mutex<u64>) {
    loop {
        if accounting.stop.load(Ordering::SeqCst) {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
        let now = Instant::now();
        let last = *accounting.last_checkpoint_instant.lock().unwrap();
        if !should_trigger_checkpoint(now, last, accounting.bytes_since_checkpoint.load(Ordering::Relaxed), checkpoint_time_threshold, effective_dirty_bytes_threshold) {
            continue;
        }
        let fsync_count = accounting.fsync_count_since_cut.swap(0, Ordering::SeqCst);
        let batch_count = accounting.batch_count_since_cut.swap(0, Ordering::SeqCst).max(1);
        accounting.bytes_since_checkpoint.store(0, Ordering::SeqCst);
        *accounting.last_checkpoint_instant.lock().unwrap() = Instant::now();
        if fsync_count == 0 {
            continue; // 这一格窗口内还没有新 fsync，不必做空 checkpoint
        }
        let concurrent_fsync_count = ((fsync_count as f64 / batch_count as f64).round() as u64).max(1);
        let plan = {
            let mut state = random_state.lock().unwrap();
            plan_write_ahead_log_checkpoint(shape, write_ahead_log_arm, concurrent_fsync_count, shared, batch_count, IntermediateVersionPolicy::ReleasedAndBacklogged, &mut state)
        };
        let _made_durable_at = execute_batch_persist(device, buffers, cursors, &plan, worker_count);
        execute_batch_finalize_system_configuration(device, buffers, cursors, &plan);
        accounting.checkpoint_count.fetch_add(1, Ordering::SeqCst);
        accounting.checkpoint_bytes_total.fetch_add(plan.approximate_bytes_single_device(), Ordering::SeqCst);
    }
}

fn caller_loop(queue: &SharedQueue, caller_stop: &std::sync::atomic::AtomicBool, samples_out: &Mutex<Vec<SampleRecord>>) {
    loop {
        if caller_stop.load(Ordering::SeqCst) {
            return;
        }
        let issued_at = Instant::now();
        let (reply_sender, rx) = mpsc::channel();
        {
            let mut guard = queue.entries.lock().unwrap();
            guard.push_back(QueueEntry { reply: reply_sender });
            queue.condvar.notify_all();
        }
        match rx.recv() {
            Ok((batch_started_at, made_durable_at, released_at)) => {
                let returned_at = Instant::now();
                samples_out.lock().unwrap().push(SampleRecord { issued_at, batch_started_at, made_durable_at, released_at, returned_at, batch_size: 0 });
            }
            Err(_) => return,
        }
    }
}

/// 开环里 `issued_at` 的语义（第一节「读法写死」）：取排定的到达时刻，不取实际发出的时刻——
/// 发晚了的那一段也算等待。M13 的反例是直接返回 `actual_issue_instant`。
fn open_loop_issued_at(scheduled_instant: Instant, _actual_issue_instant: Instant) -> Instant {
    scheduled_instant
}

struct CellConfiguration {
    arm: Arm,
    concurrent_fsync_count: u64,
    shared: bool,
    group_commit_enabled: bool,
    warmup: Duration,
    duration: Duration,
    worker_count: usize,
    pc1_inject_duration: Option<Duration>,
    checkpoint_time_threshold: Duration,
    effective_dirty_bytes_threshold: u64,
    seed: u64,
    shape: PoolShape,
}

struct CellReport {
    samples_in_window: Vec<SampleRecord>,
    total_samples: usize,
    containment_violations: usize,
    checkpoint_count: u64,
    checkpoint_bytes_total: u64,
    bytes_written_estimate: u64,
}

/// 跑一个闭环格（第五节 5.5）：k 个调用方、零思考时间，跑 `warmup + duration`，只收
/// `issued_at` 落在 `[warmup, warmup+duration)` 里的样本。
fn run_closed_loop_cell(device: Arc<dyn BlockDevice>, configuration: CellConfiguration) -> CellReport {
    let buffers = Arc::new(UnitBuffers::new(configuration.seed));
    let cursors = Arc::new(RegionCursors::new());
    let queue = Arc::new(SharedQueue::new());
    let random_state = Arc::new(Mutex::new(configuration.seed | 1));
    let caller_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let samples_out: Arc<Mutex<Vec<SampleRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let batch_index_counter = Arc::new(AtomicU64::new(0));

    let write_ahead_log_arm = configuration.arm.write_ahead_log_arm();
    let write_ahead_log_accounting = write_ahead_log_arm.map(|_| Arc::new(WriteAheadLogAccounting::new()));
    let publisher_bytes_total = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    {
        let queue = Arc::clone(&queue);
        let device = Arc::clone(&device);
        let buffers = Arc::clone(&buffers);
        let cursors = Arc::clone(&cursors);
        let random_state = Arc::clone(&random_state);
        let batch_index_counter = Arc::clone(&batch_index_counter);
        let write_ahead_log_accounting_for_publisher = write_ahead_log_accounting.clone();
        let shape = configuration.shape;
        let arm = configuration.arm;
        let shared = configuration.shared;
        let worker_count = configuration.worker_count;
        let group_commit_enabled = configuration.group_commit_enabled;
        let pc1_inject = configuration.pc1_inject_duration;
        let bytes_written_total = Arc::clone(&publisher_bytes_total);
        handles.push(std::thread::spawn(move || {
            publisher_loop(&queue, group_commit_enabled, &*device, &buffers, &cursors, shape, arm, shared, worker_count, pc1_inject, &random_state, write_ahead_log_accounting_for_publisher.as_deref(), &batch_index_counter, &bytes_written_total);
        }));
    }
    if let (Some(write_ahead_log_arm), Some(accounting)) = (write_ahead_log_arm, write_ahead_log_accounting.clone()) {
        let device = Arc::clone(&device);
        let buffers = Arc::clone(&buffers);
        let cursors = Arc::clone(&cursors);
        let random_state = Arc::clone(&random_state);
        let shape = configuration.shape;
        let shared = configuration.shared;
        let worker_count = configuration.worker_count;
        let checkpoint_time_threshold = configuration.checkpoint_time_threshold;
        let effective_dirty_bytes_threshold = configuration.effective_dirty_bytes_threshold;
        handles.push(std::thread::spawn(move || {
            checkpoint_loop(&accounting, &*device, &buffers, &cursors, shape, write_ahead_log_arm, shared, worker_count, checkpoint_time_threshold, effective_dirty_bytes_threshold, &random_state);
        }));
    }

    let mut caller_handles = Vec::new();
    for _ in 0..configuration.concurrent_fsync_count {
        let queue = Arc::clone(&queue);
        let caller_stop = Arc::clone(&caller_stop);
        let samples_out = Arc::clone(&samples_out);
        caller_handles.push(
            std::thread::Builder::new()
                .stack_size(64 * 1024)
                .spawn(move || caller_loop(&queue, &caller_stop, &samples_out))
                .expect("起调用方线程失败"),
        );
    }

    let run_start = Instant::now();
    let deadline = run_start + configuration.warmup + configuration.duration;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10).min(deadline.saturating_duration_since(Instant::now())));
    }

    caller_stop.store(true, Ordering::SeqCst);
    for handle in caller_handles {
        let _ = handle.join();
    }
    queue.stop.store(true, Ordering::SeqCst);
    queue.condvar.notify_all();
    if let Some(accounting) = &write_ahead_log_accounting {
        accounting.stop.store(true, Ordering::SeqCst);
    }
    for handle in handles {
        let _ = handle.join();
    }

    let all_samples = std::mem::take(&mut *samples_out.lock().unwrap());
    let warmup_cutoff = run_start + configuration.warmup;
    let samples_in_window: Vec<SampleRecord> = all_samples.iter().copied().filter(|sample| sample.issued_at >= warmup_cutoff).collect();
    let containment_violations = samples_in_window.iter().filter(|sample| sample.violates_containment()).count();

    let publisher_bytes = publisher_bytes_total.load(Ordering::SeqCst);
    let checkpoint_bytes = write_ahead_log_accounting.as_ref().map(|accounting| accounting.checkpoint_bytes_total.load(Ordering::SeqCst)).unwrap_or(0);

    CellReport {
        total_samples: samples_in_window.len(),
        containment_violations,
        checkpoint_count: write_ahead_log_accounting.as_ref().map(|accounting| accounting.checkpoint_count.load(Ordering::SeqCst)).unwrap_or(0),
        checkpoint_bytes_total: checkpoint_bytes,
        bytes_written_estimate: publisher_bytes + checkpoint_bytes,
        samples_in_window,
    }
}

// ============================================================
// 统计：分位数、规则 R、J1 / J2a / J4 判定、Mann–Whitney A（第六节）
// ============================================================

/// 最近秩分位数（p ∈ [0,1]），输入必须已排序。
fn percentile_nanos(sorted_values: &[u64], percentile_fraction: f64) -> u64 {
    assert!(!sorted_values.is_empty(), "分位数不能在空样本上算——读不到 ≠ 读到 0");
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    let rank = (percentile_fraction * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[rank.min(sorted_values.len() - 1)]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RatioVerdict {
    CrossedUp,
    CrossedDown,
    NotCrossed,
}
/// J1 / J4 的三档（第六节）：`ratio >= threshold` 越线，`ratio <= 1/threshold` 反越线，其余未越线。
/// M11 的变异目标：把 `>=` 改成 `>`，边界 `ratio == threshold` 的单测就会翻。
fn classify_ratio(ratio: f64, threshold: f64) -> RatioVerdict {
    if ratio >= threshold {
        RatioVerdict::CrossedUp
    } else if ratio <= 1.0 / threshold {
        RatioVerdict::CrossedDown
    } else {
        RatioVerdict::NotCrossed
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FiveRoundVerdict {
    Consistent(RatioVerdict),
    Unstable,
}
/// 五轮怎么合（第六节）：五轮判同一档才报那一档，不一致记「不稳定」。
/// M12 的变异目标：改成「多数轮说了算」，四胜一的单测就会翻。
fn merge_five_rounds(verdicts: &[RatioVerdict; 5]) -> FiveRoundVerdict {
    if verdicts.iter().all(|&verdict| verdict == verdicts[0]) {
        FiveRoundVerdict::Consistent(verdicts[0])
    } else {
        FiveRoundVerdict::Unstable
    }
}

/// 规则 R（第六节）：给一串按轴排好的值打走势标签，看最后 3 步的 δ。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TrendLabel {
    ExpandingAlongAxis,
    ShrinkingAlongAxis,
    Converging,
    Other,
}
fn trend_rule_label(values_along_axis: &[f64]) -> TrendLabel {
    assert!(values_along_axis.len() >= 2, "规则 R 至少要两个点才算得出一步 δ");
    let deltas: Vec<f64> = values_along_axis.windows(2).map(|pair| pair[1] / pair[0] - 1.0).collect();
    let last_three = &deltas[deltas.len().saturating_sub(3)..];
    if last_three.len() == 3 && last_three.iter().all(|&delta| delta >= 0.05) {
        TrendLabel::ExpandingAlongAxis
    } else if last_three.len() == 3 && last_three.iter().all(|&delta| delta <= -0.05) {
        TrendLabel::ShrinkingAlongAxis
    } else if deltas.last().map(|&delta| delta.abs() < 0.05).unwrap_or(false) {
        TrendLabel::Converging
    } else {
        TrendLabel::Other
    }
}

/// Mann–Whitney U 归一化成 `A = P(甲 > WAL) + ½P(相等)`（第五节 5.4 PC3；精确算法，不用直方图近似）。
fn mann_whitney_discrimination_statistic(first_sample: &[u64], second_sample: &[u64]) -> f64 {
    assert!(!first_sample.is_empty() && !second_sample.is_empty(), "PC3 的两组样本都不能是空的");
    let mut greater = 0u64;
    let mut equal = 0u64;
    for &value_from_first_sample in first_sample {
        for &value_from_second_sample in second_sample {
            match value_from_first_sample.cmp(&value_from_second_sample) {
                std::cmp::Ordering::Greater => greater += 1,
                std::cmp::Ordering::Equal => equal += 1,
                std::cmp::Ordering::Less => {}
            }
        }
    }
    let total = (first_sample.len() * second_sample.len()) as f64;
    (greater as f64 + 0.5 * equal as f64) / total
}

/// PC1 的 Δ（第五节 5.4）：注入批（下标偶数，见 `publisher_loop`）与未注入批各自的 W 中位之差。
fn pc1_delta_milliseconds(injected_wait_nanos: &[u64], baseline_wait_nanos: &[u64]) -> f64 {
    let mut injected_sorted = injected_wait_nanos.to_vec();
    let mut baseline_sorted = baseline_wait_nanos.to_vec();
    injected_sorted.sort_unstable();
    baseline_sorted.sort_unstable();
    let injected_median = percentile_nanos(&injected_sorted, 0.5) as f64;
    let baseline_median = percentile_nanos(&baseline_sorted, 0.5) as f64;
    (injected_median - baseline_median) / 1_000_000.0
}

// ============================================================
// 来宾块层读数（第五节 5.6）、结果行、CLI、main
// ============================================================

/// `/sys/block/<设备>/stat` 的字段 2（写完成扇区，第 7 列，0-based 索引 6）与字段 0（读请求数）。
/// 字段顺序见 Linux `Documentation/ABI/testing/procfs-diskstats`：
/// reads_completed reads_merged sectors_read ms_reading writes_completed writes_merged
/// sectors_written ms_writing ...
struct BlockLayerStatistics {
    reads_completed: u64,
    sectors_read: u64,
    writes_completed: u64,
    sectors_written: u64,
}
fn read_block_layer_statistics(device_name: &str) -> std::io::Result<BlockLayerStatistics> {
    let text = std::fs::read_to_string(format!("/sys/block/{device_name}/stat"))?;
    let fields: Vec<u64> = text.split_whitespace().map(|token| token.parse().unwrap_or(0)).collect();
    Ok(BlockLayerStatistics {
        reads_completed: *fields.first().unwrap_or(&0),
        sectors_read: *fields.get(2).unwrap_or(&0),
        writes_completed: *fields.get(4).unwrap_or(&0),
        sectors_written: *fields.get(6).unwrap_or(&0),
    })
}

fn device_name_from_path(device_path: &str) -> String {
    device_path.rsplit('/').next().unwrap_or(device_path).to_string()
}

/// 甲一批的单盘总字节（跨 `print_anchors` 与单测共用）。`JiaPublishOutcome::root_slot_bytes`
/// 本来就是单盘值（第三节 2.3：「根槽只在一块盘」），不能像别的字段一样再除以 `DEVICE_COUNT`——
/// `outcome.total_bytes() / DEVICE_COUNT` 这个近似对根槽字段是错的（会把它也腰斩），B4 钉的
/// 172544 就是这个函数的口径，不是那个近似的口径。
fn jia_outcome_single_device_bytes(outcome: &JiaPublishOutcome) -> u64 {
    let both_device_total = outcome.data_bytes + outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes + outcome.record_bytes;
    both_device_total / DEVICE_COUNT + outcome.root_slot_bytes + outcome.system_configuration_bytes / DEVICE_COUNT
}

/// 诊断用：把 B1–B4 用到的几格期望值打出来，供人眼核对；判据本身钉在单测里
/// （`anchors_match_prereg_section_seven` 等，第七节 7.2），不靠这条命令的输出。
/// 纯算术、不碰真设备、不含任何计时字段——`research/scripts/replay.sh` 拿它逐字节复跑
/// （E159 装置还没有正式计时产物时，这是唯一能稳定逐字节复现的一份）。
fn print_anchors() {
    let mut emitter = Emitter::new();
    let shape_file_count_one_family_one_data_unit = main_shape(1, Family::OneDataUnitPerFile);
    let jia_outcome_at_file_count_one = solve_jia_batch_publish(shape_file_count_one_family_one_data_unit, 1, true);
    let write_ahead_log_full_outcome_at_file_count_one = solve_write_ahead_log_batch_fsync(shape_file_count_one_family_one_data_unit, WriteAheadLogArm::WriteAheadLogFull, 1, true);
    let write_ahead_log_leaf_outcome_at_file_count_one = solve_write_ahead_log_batch_fsync(shape_file_count_one_family_one_data_unit, WriteAheadLogArm::WriteAheadLogLeaf, 1, true);
    println!("{}", emitter.emit_raw(&format!("name=anchor label=jia_p1_f1_k1_single_device_bytes value={}", jia_outcome_single_device_bytes(&jia_outcome_at_file_count_one))));
    println!("{}", emitter.emit_raw(&format!("name=anchor label=write_ahead_log_full_k10_p1_f1_k1_single_device_bytes value={}", write_ahead_log_full_outcome_at_file_count_one.total_bytes() / DEVICE_COUNT)));
    println!("{}", emitter.emit_raw(&format!("name=anchor label=yi_m_k10_p1_f1_k1_single_device_bytes value={}", write_ahead_log_leaf_outcome_at_file_count_one.total_bytes() / DEVICE_COUNT)));
    let main_geometry_shape = main_shape(10_000, Family::EightAdjacentDataUnitsPerFsync);
    let write_ahead_log_full_outcome_at_main_geometry_batch_size_one = solve_write_ahead_log_batch_fsync(main_geometry_shape, WriteAheadLogArm::WriteAheadLogFull, 1, true);
    let write_ahead_log_full_outcome_at_main_geometry_batch_size_sixteen = solve_write_ahead_log_batch_fsync(main_geometry_shape, WriteAheadLogArm::WriteAheadLogFull, 16, true);
    println!("{}", emitter.emit_raw(&format!("name=anchor label=write_ahead_log_full_k10_f8a_p10000_k1_single_device_expected_bytes value={:.3}", write_ahead_log_full_outcome_at_main_geometry_batch_size_one.total_bytes() as f64 / DEVICE_COUNT as f64)));
    println!("{}", emitter.emit_raw(&format!("name=anchor label=write_ahead_log_full_k10_f8a_p10000_k16_single_device_expected_bytes value={:.3}", write_ahead_log_full_outcome_at_main_geometry_batch_size_sixteen.total_bytes() as f64 / DEVICE_COUNT as f64)));
    println!("{}", emitter.finish());
}

struct CliArguments {
    subcommand: String,
    device_path: Option<String>,
    arm: Arm,
    concurrent_fsync_count: u64,
    group_commit_enabled: bool,
    warmup_seconds: f64,
    duration_seconds: f64,
    pc1_inject_milliseconds: Option<f64>,
    seed: u64,
    file_count: u64,
    family: Family,
}
const KNOWN_SUBCOMMANDS: [&str; 3] = ["anchors", "cell", "probe-device"];

/// `research/scripts/vm-bench.sh` 按 `.claude/kb/vm-harness.md`「挂几块盘」的约定，
/// 把设备路径按顺序当成前缀参数塞在最前面、我们自己的参数跟在后面——
/// 所以在找到第一个认得出的子命令之前，把前面的位置参数都当成设备路径，
/// 第一个当默认 `--device`（后面出现的 `--device` 可以覆盖它）。
fn parse_arguments(raw_arguments: &[String]) -> CliArguments {
    let subcommand_index = raw_arguments.iter().position(|argument| KNOWN_SUBCOMMANDS.contains(&argument.as_str()));
    let leading_device_path = match subcommand_index {
        Some(0) | None => None,
        Some(_) => raw_arguments.first().cloned(),
    };
    let subcommand = subcommand_index.map(|found_index| raw_arguments[found_index].clone()).unwrap_or_else(|| "anchors".to_string());
    let remaining_arguments: &[String] = match subcommand_index {
        Some(found_index) => &raw_arguments[found_index + 1..],
        None => &[],
    };

    let mut arguments = CliArguments {
        subcommand,
        device_path: leading_device_path,
        arm: Arm::Jia,
        concurrent_fsync_count: 1,
        group_commit_enabled: true,
        warmup_seconds: 1.0,
        duration_seconds: 2.0,
        pc1_inject_milliseconds: None,
        seed: 1,
        file_count: 10_000,
        family: Family::EightAdjacentDataUnitsPerFsync,
    };
    let raw_arguments = remaining_arguments;
    let mut index = 0usize;
    while index < raw_arguments.len() {
        let flag = raw_arguments[index].as_str();
        let mut take_value = || {
            index += 1;
            raw_arguments.get(index).cloned().unwrap_or_default()
        };
        match flag {
            "--device" => arguments.device_path = Some(take_value()),
            "--arm" => {
                arguments.arm = match take_value().as_str() {
                    "jia" => Arm::Jia,
                    "wal-full" => Arm::WriteAheadLogFull,
                    "yi-m" => Arm::WriteAheadLogLeaf,
                    other => panic!("未知 --arm：{other}"),
                }
            }
            "--k" => arguments.concurrent_fsync_count = take_value().parse().expect("--k 要是整数"),
            "--group-commit" => arguments.group_commit_enabled = take_value() == "on",
            "--warmup-seconds" => arguments.warmup_seconds = take_value().parse().expect("--warmup-seconds 要是数字"),
            "--duration-seconds" => arguments.duration_seconds = take_value().parse().expect("--duration-seconds 要是数字"),
            "--pc1-inject-ms" => arguments.pc1_inject_milliseconds = Some(take_value().parse().expect("--pc1-inject-ms 要是数字")),
            "--seed" => arguments.seed = take_value().parse().expect("--seed 要是整数"),
            "--file-count" => arguments.file_count = take_value().parse().expect("--file-count 要是整数"),
            "--family" => {
                arguments.family = match take_value().as_str() {
                    "f1" => Family::OneDataUnitPerFile,
                    "f8a" => Family::EightAdjacentDataUnitsPerFsync,
                    other => panic!("未知 --family：{other}"),
                }
            }
            other => panic!("未知参数：{other}"),
        }
        index += 1;
    }
    arguments
}

fn run_cell_subcommand(arguments: &CliArguments) {
    let device_path = arguments.device_path.as_deref().expect("--device 必须给");
    let device_name = device_name_from_path(device_path);
    let device: Arc<dyn BlockDevice> = Arc::new(RealBlockDevice::open(device_path).expect("打开设备失败"));
    let before = read_block_layer_statistics(&device_name).ok();
    let shape = main_shape(arguments.file_count, arguments.family);
    let configuration = CellConfiguration {
        arm: arguments.arm,
        concurrent_fsync_count: arguments.concurrent_fsync_count,
        shared: true,
        group_commit_enabled: arguments.group_commit_enabled,
        warmup: Duration::from_secs_f64(arguments.warmup_seconds),
        duration: Duration::from_secs_f64(arguments.duration_seconds),
        worker_count: 32,
        pc1_inject_duration: arguments.pc1_inject_milliseconds.map(Duration::from_secs_f64),
        checkpoint_time_threshold: CHECKPOINT_TIME_THRESHOLD,
        effective_dirty_bytes_threshold: 256 * 1024 * 1024,
        seed: arguments.seed,
        shape,
    };
    let report = run_closed_loop_cell(device, configuration);
    let after = read_block_layer_statistics(&device_name).ok();
    let mut waits: Vec<u64> = report.samples_in_window.iter().map(|sample| sample.wait_nanos()).collect();
    waits.sort_unstable();
    let median = if waits.is_empty() { 0 } else { percentile_nanos(&waits, 0.5) };
    let ninety_ninth_percentile_nanoseconds = if waits.is_empty() { 0 } else { percentile_nanos(&waits, 0.99) };
    // 结果抓取要有完整性闸（`command-safety.md`）：借用 `e7_index_bench::Emitter`，
    // 收尾打一行 `E7RESULT name=done emitted=N`，`vm-bench.sh` 拿它对账（`.claude/kb/vm-harness.md`「结果抓取有一道完整性闸」）。
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "E159RESULT arm={} k={} group_commit={} samples={} median_ns={} p99_ns={} containment_violations={} checkpoint_count={} checkpoint_bytes={} bytes_written_estimate={}",
            arguments.arm.tag(),
            arguments.concurrent_fsync_count,
            arguments.group_commit_enabled,
            report.total_samples,
            median,
            ninety_ninth_percentile_nanoseconds,
            report.containment_violations,
            report.checkpoint_count,
            report.checkpoint_bytes_total,
            report.bytes_written_estimate,
        ))
    );
    if let (Some(before), Some(after)) = (before, after) {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "E159BLOCKSTAT device={} sectors_written_diff={} sectors_read_diff={} writes_completed_diff={} reads_completed_diff={}",
                device_name,
                after.sectors_written.saturating_sub(before.sectors_written),
                after.sectors_read.saturating_sub(before.sectors_read),
                after.writes_completed.saturating_sub(before.writes_completed),
                after.reads_completed.saturating_sub(before.reads_completed),
            ))
        );
    }
    println!("{}", emitter.finish());
}

/// 停机条款 S3：来宾回放盘的 `logical_block_size` 必须是 512（根槽宽度依赖这个值，第七节 B9）。
fn run_probe_device_subcommand(arguments: &CliArguments) {
    let device_path = arguments.device_path.as_deref().expect("--device 必须给");
    let device_name = device_name_from_path(device_path);
    let logical_block_size_path = format!("/sys/block/{device_name}/queue/logical_block_size");
    let logical_block_size_text = std::fs::read_to_string(&logical_block_size_path).unwrap_or_else(|error| panic!("读不到 {logical_block_size_path}：{error}"));
    let logical_block_size: u64 = logical_block_size_text.trim().parse().unwrap_or_else(|error| panic!("{logical_block_size_path} 里的内容 {logical_block_size_text:?} 解析不出整数：{error}"));
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!("E159PROBE device={device_name} logical_block_size={logical_block_size}")));
    println!("{}", emitter.finish());
}

fn main() {
    let raw_arguments: Vec<String> = std::env::args().collect();
    let arguments = parse_arguments(&raw_arguments[1..].to_vec());
    match arguments.subcommand.as_str() {
        "anchors" => print_anchors(),
        "cell" => run_cell_subcommand(&arguments),
        "probe-device" => run_probe_device_subcommand(&arguments),
        other => panic!("未知子命令：{other}（用 anchors、cell 或 probe-device）"),
    }
}

// ============================================================
// 单测（跑前登记第七节钉绝对值的断言、第九节变异表 M1–M14 的目标）
// ============================================================

#[cfg(test)]
mod e159_anchor_and_rule_tests {
    use super::*;

    /// B4：甲、F1、P=1、k=1（锚点格），带根槽那块盘。与第三节 `crates/` 测试表逐项比
    /// （跑前登记停机条款 S1 的一部分）：写 172544 字节、写调用 11 次。
    /// （`jia_outcome_single_device_bytes` 现在是模块顶层的共用函数，`print_anchors` 也用它。）
    #[test]
    fn jia_single_fsync_batch_matches_crates_test_table() {
        let shape = main_shape(1, Family::OneDataUnitPerFile);
        let outcome = solve_jia_batch_publish(shape, 1, true);
        assert_eq!(jia_outcome_single_device_bytes(&outcome), 172_544, "B4：单盘总字节要等于 crates 测试表逐项加总");
        // `outcome.write_calls` 里根槽只算了 1 次（`+ 1`，不是 `+ DEVICE_COUNT`），两盘合计是奇数，
        // 直接除以 2 会截断丢 1；单盘写调用数照 `BatchPlan::total_write_calls` 同样的口径重算。
        let mut random_state = 99u64;
        let plan = plan_jia_batch(shape, 1, true, &mut random_state);
        assert_eq!(plan.total_write_calls(), 11, "单盘写调用应是 8 单元 + 1 记录 + 根槽 + 系统配置槽 = 11 次");
    }

    /// B1：wal_full-K10、F1、P=10⁴、k=1：单盘 118784 字节、6 次写调用。
    #[test]
    fn write_ahead_log_full_single_fsync_batch_anchor() {
        let shape = main_shape(10_000, Family::OneDataUnitPerFile);
        let outcome = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 1, true);
        assert_eq!(outcome.total_bytes() / DEVICE_COUNT, 118_784);
        assert_eq!(outcome.write_calls / DEVICE_COUNT, 6);
    }

    /// B2：yi-M-K10、F1、P=10⁴、k=1：单盘 86016 字节、4 次写调用。
    #[test]
    fn write_ahead_log_leaf_single_fsync_batch_anchor() {
        let shape = main_shape(10_000, Family::OneDataUnitPerFile);
        let outcome = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 1, true);
        assert_eq!(outcome.total_bytes() / DEVICE_COUNT, 86_016);
        assert_eq!(outcome.write_calls / DEVICE_COUNT, 4);
    }

    /// B3：F8A、P=10⁴（主几何），WAL 两臂；b=1 与 b=16 的批，单盘写字节期望值
    /// （跑前登记第十三节 13.5 python 脚本原样输出：365345.862 / 4313361.603 / 316188.444 / 4264111.306）。
    #[test]
    fn main_geometry_write_ahead_log_batch_expected_bytes() {
        let shape = main_shape(10_000, Family::EightAdjacentDataUnitsPerFsync);
        let write_ahead_log_full_outcome_batch_size_one = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 1, true);
        let write_ahead_log_full_outcome_batch_size_sixteen = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 16, true);
        let write_ahead_log_leaf_outcome_batch_size_one = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 1, true);
        let write_ahead_log_leaf_outcome_batch_size_sixteen = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 16, true);
        // 单盘容差放宽到 2.0：模型内部按 (两盘合计的整数字节).round() 逐字段取整，
        // 8 个字段各自的取整误差累加、再除以 2，理论上界正是 8 × 0.5 / 2 = 2.0。
        let close = |actual: f64, expected: f64| (actual - expected).abs() < 2.0;
        assert!(close(write_ahead_log_full_outcome_batch_size_one.total_bytes() as f64 / DEVICE_COUNT as f64, 365_345.862));
        assert!(close(write_ahead_log_full_outcome_batch_size_sixteen.total_bytes() as f64 / DEVICE_COUNT as f64, 4_313_361.603));
        assert!(close(write_ahead_log_leaf_outcome_batch_size_one.total_bytes() as f64 / DEVICE_COUNT as f64, 316_188.444));
        assert!(close(write_ahead_log_leaf_outcome_batch_size_sixteen.total_bytes() as f64 / DEVICE_COUNT as f64, 4_264_111.306));
    }

    /// B9：来宾 `logical_block_size` 由跑真设备探针核（跑前登记第十一节 S3），这里只钉主几何的
    /// `physical_block_size_bytes` 常量本身取的是 512（根槽宽度依赖这个值）。
    #[test]
    fn main_geometry_primary_physical_block_size_is_512() {
        assert_eq!(Geometry::main().physical_block_size_bytes, 512);
    }

    /// M7 的正确性锚点（第九节）：随机取整长期不偏，期望值的小数部分就是「多写一个」的频率。
    #[test]
    fn round_stochastic_is_unbiased_over_many_draws() {
        let expected = 3.0489; // 与跑前登记第十三节 13.5 的 extent 期望个数同量级
        let mut state = 42u64;
        let draws = 20_000u64;
        let total: u64 = (0..draws).map(|_| round_stochastic(expected, &mut state)).sum();
        let observed_mean = total as f64 / draws as f64;
        assert!((observed_mean - expected).abs() < 0.02, "20000 次取整的均值 {observed_mean} 应逼近期望 {expected}");
    }

    #[test]
    fn round_stochastic_never_returns_negative_or_absurd_values_for_zero() {
        let mut state = 1u64;
        for _ in 0..1000 {
            assert_eq!(round_stochastic(0.0, &mut state), 0);
        }
    }

    /// M11 的变异目标：边界 `ratio == 2.0` 必须判「越线」（`>=`，不是 `>`）。
    #[test]
    fn classify_ratio_boundary_at_threshold_counts_as_crossed() {
        assert_eq!(classify_ratio(2.0, 2.0), RatioVerdict::CrossedUp);
        assert_eq!(classify_ratio(1.999, 2.0), RatioVerdict::NotCrossed);
        assert_eq!(classify_ratio(0.5, 2.0), RatioVerdict::CrossedDown);
    }

    /// M12 的变异目标：四胜一不该算「一致」，五轮不同档只能记「不稳定」。
    #[test]
    fn merge_five_rounds_four_against_one_is_unstable_not_majority() {
        let verdicts = [RatioVerdict::CrossedUp, RatioVerdict::CrossedUp, RatioVerdict::CrossedUp, RatioVerdict::CrossedUp, RatioVerdict::NotCrossed];
        assert_eq!(merge_five_rounds(&verdicts), FiveRoundVerdict::Unstable);
    }

    #[test]
    fn merge_five_rounds_unanimous_reports_that_verdict() {
        let verdicts = [RatioVerdict::NotCrossed; 5];
        assert_eq!(merge_five_rounds(&verdicts), FiveRoundVerdict::Consistent(RatioVerdict::NotCrossed));
    }

    #[test]
    fn trend_rule_expanding_requires_three_consecutive_five_percent_steps() {
        assert_eq!(trend_rule_label(&[1.0, 1.06, 1.13, 1.21]), TrendLabel::ExpandingAlongAxis);
        assert_eq!(trend_rule_label(&[1.0, 1.06, 1.061, 1.062]), TrendLabel::Converging);
    }

    /// M8 的变异目标：`T_time` 若读成 50 s，10 s 窗口里应有 0 次触发；正确的 5 s 应至少触发 1 次。
    #[test]
    fn checkpoint_trigger_fires_on_time_threshold_with_correct_constant_but_not_with_mutated_one() {
        let last_checkpoint = Instant::now() - Duration::from_secs(6);
        assert!(should_trigger_checkpoint(Instant::now(), last_checkpoint, 0, Duration::from_millis(5000), 256 * 1024 * 1024), "5 s 阈值下，距上次 6 s 应当触发");
        assert!(!should_trigger_checkpoint(Instant::now(), last_checkpoint, 0, Duration::from_millis(50_000), 256 * 1024 * 1024), "M8：读成 50 s 时同样的间隔不该触发");
    }

    #[test]
    fn checkpoint_trigger_fires_on_dirty_bytes_threshold_even_before_time_threshold() {
        let last_checkpoint = Instant::now();
        assert!(should_trigger_checkpoint(Instant::now(), last_checkpoint, 256 * 1024 * 1024, Duration::from_millis(5000), 256 * 1024 * 1024));
        assert!(!should_trigger_checkpoint(Instant::now(), last_checkpoint, 1, Duration::from_millis(5000), 256 * 1024 * 1024));
    }

    /// M8 的变异目标（直接测常量，不是 `should_trigger_checkpoint` 本身——那个函数的阈值是参数，
    /// 真正会被读错的是 `run_cell_subcommand` 用的这个常量）：`T_time` 必须是 5000 ms，不是 50000 ms。
    #[test]
    fn checkpoint_time_threshold_constant_is_five_seconds() {
        assert_eq!(CHECKPOINT_TIME_THRESHOLD, Duration::from_millis(5000));
    }

    /// M13 的变异目标：开环 `issued_at` 必须取排定时刻，不取实际发出时刻——发晚的那一段也算等待。
    #[test]
    fn open_loop_issued_at_uses_scheduled_instant_not_actual() {
        let scheduled = Instant::now();
        std::thread::sleep(Duration::from_millis(5));
        let actual = Instant::now();
        let issued_at = open_loop_issued_at(scheduled, actual);
        assert_eq!(issued_at, scheduled, "M13：issued_at 必须等于排定时刻，不能悄悄换成实际发出时刻");
        assert!(actual.duration_since(issued_at) >= Duration::from_millis(4), "晚发的那几毫秒要能被 W 吃进去");
    }

    /// M14 的变异目标：「放行到醒」这一段必须能为正——真线程的唤醒不可能恰好为 0。
    #[test]
    fn release_to_wake_is_positive_when_release_precedes_return() {
        let issued_at = Instant::now();
        let batch_started_at = issued_at;
        let made_durable_at = batch_started_at + Duration::from_micros(10);
        let released_at = made_durable_at;
        let returned_at = released_at + Duration::from_micros(3);
        let sample = SampleRecord { issued_at, batch_started_at, made_durable_at, released_at, returned_at, batch_size: 1 };
        assert!(sample.release_to_wake_nanos() > 0);
        assert!(!sample.violates_containment());
    }

    #[test]
    fn containment_violation_detected_when_order_broken() {
        let now = Instant::now();
        let sample = SampleRecord { issued_at: now + Duration::from_micros(5), batch_started_at: now, made_durable_at: now, released_at: now, returned_at: now, batch_size: 1 };
        assert!(sample.violates_containment(), "issued_at 晚于 batch_started_at 时必须判违反");
    }

    #[test]
    fn mann_whitney_a_is_one_half_when_samples_are_identical_distributions() {
        let first_sample = vec![10u64, 20, 30, 40];
        let second_sample = vec![10u64, 20, 30, 40];
        let discrimination = mann_whitney_discrimination_statistic(&first_sample, &second_sample);
        assert!((discrimination - 0.5).abs() < 1e-9);
    }

    #[test]
    fn mann_whitney_a_is_near_one_when_a_always_greater() {
        let first_sample = vec![100u64, 200, 300];
        let second_sample = vec![1u64, 2, 3];
        assert!(mann_whitney_discrimination_statistic(&first_sample, &second_sample) > 0.99);
    }

    fn run_recorded_jia_batch(seed: u64) -> (BatchPlan, Instant, Vec<(Instant, StepMark)>) {
        let shape = main_shape(1, Family::OneDataUnitPerFile);
        let mut state = seed;
        let plan = plan_jia_batch(shape, 1, true, &mut state);
        let buffers = UnitBuffers::new(seed);
        let cursors = RegionCursors::new();
        let device = RecordingBlockDevice::new();
        let made_durable_at = execute_batch_persist(&device, &buffers, &cursors, &plan, 4);
        execute_batch_finalize_system_configuration(&device, &buffers, &cursors, &plan);
        let log = device.drain();
        (plan, made_durable_at, log)
    }

    fn split_by_barrier(marks: &[StepMark]) -> Vec<Vec<StepMark>> {
        let mut phases = vec![Vec::new()];
        for &mark in marks {
            if mark == StepMark::Barrier {
                phases.push(Vec::new());
            } else {
                phases.last_mut().unwrap().push(mark);
            }
        }
        phases
    }

    /// S1（跑前登记停机条款 S1 的次序部分）与 A4：甲一次发布的次序 = 单元 → 屏障 → 记录 → 屏障 →
    /// 根槽 FUA → 系统配置槽（先读两个、再写一个）；A5：两道屏障 + 1 次 FUA = 3 次 FLUSH。
    #[test]
    fn jia_single_fsync_batch_step_sequence_matches_crates_order() {
        let (plan, _made_durable_at, log) = run_recorded_jia_batch(7);
        let marks: Vec<StepMark> = log.iter().map(|&(_, mark)| mark).collect();
        let barrier_count = marks.iter().filter(|&&mark| mark == StepMark::Barrier).count();
        assert_eq!(barrier_count, 3, "A5：两道屏障 + 1 次 FUA 应等于 3 次 FLUSH（M1 去掉第二道屏障会让这里变 2）");
        let phases = split_by_barrier(&marks);
        assert_eq!(phases.len(), 4, "3 次屏障应把日志切成 4 段：单元段、记录段、根槽段、系统配置段");
        assert_eq!(phases[0].len(), plan.named_unit_total() as usize, "第 1 段只应是单元写，条数等于 W");
        assert!(phases[0].iter().all(|mark| matches!(mark, StepMark::Write { kind } if *kind != UnitKind::Record && *kind != UnitKind::RootSlot && *kind != UnitKind::SystemConfigurationSlot)));
        assert_eq!(phases[1].len(), plan.record_count as usize, "第 2 段只应是记录写");
        assert!(phases[1].iter().all(|mark| matches!(mark, StepMark::Write { kind: UnitKind::Record })));
        assert_eq!(phases[2], vec![StepMark::Write { kind: UnitKind::RootSlot }], "第 3 段（FUA 之前）只应是根槽写");
        assert_eq!(phases[3], vec![StepMark::Read { kind: UnitKind::SystemConfigurationSlot }, StepMark::Read { kind: UnitKind::SystemConfigurationSlot }, StepMark::Write { kind: UnitKind::SystemConfigurationSlot }], "M9 的反例：系统配置槽写之前必须先读两次");
    }

    /// A3：`made_durable_at` 必须是根槽那次 FUA（写 + 第 3 道屏障）返回之后的时刻，不能早于它
    /// （M2 的变异目标：在第一道屏障之后就把这个时刻报出去）。
    #[test]
    fn jia_durable_instant_is_not_earlier_than_the_root_slot_fua() {
        let (_plan, made_durable_at, log) = run_recorded_jia_batch(11);
        let last_root_related_timestamp = log
            .iter()
            .filter(|&&(_, mark)| mark == StepMark::Barrier || mark == StepMark::Write { kind: UnitKind::RootSlot })
            .map(|&(instant, _)| instant)
            .take(3) // 前 3 次屏障/根槽写：第 3 次就是 FUA 的 barrier
            .last()
            .expect("至少要有 3 次屏障/根槽写事件");
        assert!(made_durable_at >= last_root_related_timestamp, "M2：made_durable_at 不能早于根槽 FUA 完成的时刻");
    }

    /// M10 的变异目标：根槽写宽度必须是主几何的 `physical_block_size_bytes`（512），不是 4096。
    #[test]
    fn root_slot_width_is_primary_physical_block_size() {
        assert_eq!(UnitKind::RootSlot.width_bytes(), PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES);
        assert_ne!(UnitKind::RootSlot.width_bytes(), SYSTEM_CONFIGURATION_SLOT_BYTES);
    }

    /// M9 的变异目标（直接测系统配置槽读次数）：一次甲批必须读两次系统配置槽。
    #[test]
    fn system_configuration_slot_is_read_twice_before_being_written() {
        let (_plan, _made_durable_at, log) = run_recorded_jia_batch(23);
        let marks = log.into_iter().map(|(_, mark)| mark).collect::<Vec<_>>();
        let read_count = marks.iter().filter(|&&mark| mark == StepMark::Read { kind: UnitKind::SystemConfigurationSlot }).count();
        assert_eq!(read_count, 2);
    }

    /// M3 的变异目标：把某个臂的分支换成别的臂的规划函数（「臂写混」）。三条臂在同一个几何、
    /// 同一个批大小上必须给出三个不同的字节数，且各自与直接调用对应 `solve_*` 函数（B1/B2 已锚定
    /// 的值）一致——这样任何一支被换成别的分支都会被下面两个 `assert_eq!` 抓到。
    #[test]
    fn plan_batch_for_arm_routes_each_arm_to_its_own_solver_not_a_different_one() {
        let shape = main_shape(10_000, Family::OneDataUnitPerFile);
        let mut state_full = 5u64;
        let write_ahead_log_full_plan = plan_batch_for_arm(Arm::WriteAheadLogFull, shape, 1, true, &mut state_full);
        assert_eq!(write_ahead_log_full_plan.total_write_calls(), 6, "M3：wal_full-K10 那一支必须调用 WAL 的规划函数（B1：6 次写调用）");
        let mut state_leaf = 5u64;
        let write_ahead_log_leaf_plan = plan_batch_for_arm(Arm::WriteAheadLogLeaf, shape, 1, true, &mut state_leaf);
        assert_eq!(write_ahead_log_leaf_plan.total_write_calls(), 4, "M3：乙-M-K10 那一支必须调用乙的规划函数（B2：4 次写调用），不是 wal_full 的");
        let mut state_jia = 5u64;
        let jia_plan = plan_batch_for_arm(Arm::Jia, shape, 1, true, &mut state_jia);
        assert_ne!(jia_plan.total_write_calls(), write_ahead_log_full_plan.total_write_calls(), "M3：甲那一支不许换成 WAL 的规划函数");
    }

    /// M4 的变异目标：「组提交开」也把批封顶为 1（组提交名存实亡）。组提交开时必须把队列里
    /// 现有的全部请求收成一批，不是只取 1 个。
    #[test]
    fn drain_batch_from_queue_collects_everything_when_group_commit_is_on() {
        let (sender, _receiver) = mpsc::channel();
        let mut queue: VecDeque<QueueEntry> = VecDeque::new();
        queue.push_back(QueueEntry { reply: sender.clone() });
        queue.push_back(QueueEntry { reply: sender.clone() });
        queue.push_back(QueueEntry { reply: sender });
        let batch = drain_batch_from_queue(&mut queue, true);
        assert_eq!(batch.len(), 3, "M4：组提交开时应当把队列里全部 3 个请求收成一批");
        assert!(queue.is_empty(), "收完之后队列应当空");
    }

    #[test]
    fn drain_batch_from_queue_collects_only_one_when_group_commit_is_off() {
        let (sender, _receiver) = mpsc::channel();
        let mut queue: VecDeque<QueueEntry> = VecDeque::new();
        queue.push_back(QueueEntry { reply: sender.clone() });
        queue.push_back(QueueEntry { reply: sender });
        let batch = drain_batch_from_queue(&mut queue, false);
        assert_eq!(batch.len(), 1, "组提交关时一批只能有 1 个（第一节「组提交关」）");
        assert_eq!(queue.len(), 1, "剩下那个必须留在队列里，等下一批");
    }

    /// M5 的变异目标：`t_issue` 改成在收批时刻取，漏掉排队那一段。用 `SleepingBlockDevice`
    /// （固定屏障延迟，不碰真磁盘）逼出排队：4 个闭环调用方、组提交关（一批只服务 1 个），
    /// 发布者一批的服务时间约 2×`barrier_delay`，跑的时间窗口够让后到的调用方排上队。
    /// 正确实现下，至少要有一个样本的 `batch_started_at − issued_at`（排队段）明显为正；
    /// 若 `issued_at` 被换成收批时刻，这一段恒为 0，断言必红。
    #[test]
    fn closed_loop_queueing_delay_is_counted_when_group_commit_is_off() {
        let device: Arc<dyn BlockDevice> = Arc::new(SleepingBlockDevice { barrier_delay: Duration::from_millis(15) });
        let configuration = CellConfiguration {
            arm: Arm::WriteAheadLogFull,
            concurrent_fsync_count: 4,
            shared: true,
            group_commit_enabled: false,
            warmup: Duration::from_millis(0),
            duration: Duration::from_millis(400),
            worker_count: 1,
            pc1_inject_duration: None,
            checkpoint_time_threshold: Duration::from_secs(5),
            effective_dirty_bytes_threshold: 256 * 1024 * 1024,
            seed: 17,
            shape: main_shape(10_000, Family::OneDataUnitPerFile),
        };
        let report = run_closed_loop_cell(device, configuration);
        assert!(report.total_samples >= 4, "M5：窗口太短，样本不够（读不到 ≠ 读到 0）");
        let maximum_queueing_nanos = report
            .samples_in_window
            .iter()
            .map(|sample| sample.batch_started_at.saturating_duration_since(sample.issued_at).as_nanos())
            .max()
            .unwrap();
        assert!(maximum_queueing_nanos as u64 >= Duration::from_millis(10).as_nanos() as u64, "M5：4 个调用方抢一个串行发布者，至少要有一个样本排过队（观测到 {maximum_queueing_nanos} ns）");
    }

    /// M6 的变异目标：PC1 注入的忙等被挪到放行之后（`released_at` 算出来之后才等）。
    /// 正确实现下，`released_at − made_durable_at` 在被注入的那些批上应当明显偏大
    /// （约等于注入时长），在没被注入的批上应当接近 0；两者都要出现，否则说明注入没起作用
    /// 或者起作用的时机不对。
    #[test]
    fn pc1_injected_wait_lands_before_release_not_after() {
        let inject = Duration::from_millis(30);
        let device: Arc<dyn BlockDevice> = Arc::new(SleepingBlockDevice { barrier_delay: Duration::from_millis(1) });
        let configuration = CellConfiguration {
            arm: Arm::Jia,
            concurrent_fsync_count: 1,
            shared: true,
            group_commit_enabled: true,
            warmup: Duration::from_millis(0),
            duration: Duration::from_millis(300),
            worker_count: 1,
            pc1_inject_duration: Some(inject),
            checkpoint_time_threshold: Duration::from_secs(5),
            effective_dirty_bytes_threshold: 256 * 1024 * 1024,
            seed: 29,
            shape: main_shape(10_000, Family::OneDataUnitPerFile),
        };
        let report = run_closed_loop_cell(device, configuration);
        assert!(report.total_samples >= 4, "M6：窗口太短，样本不够（读不到 ≠ 读到 0）");
        let gaps_nanos: Vec<u128> = report.samples_in_window.iter().map(|sample| sample.released_at.saturating_duration_since(sample.made_durable_at).as_nanos()).collect();
        let half_inject_nanos = inject.as_nanos() / 2;
        let has_injected_gap = gaps_nanos.iter().any(|&gap| gap >= half_inject_nanos);
        let has_uninjected_gap = gaps_nanos.iter().any(|&gap| gap < half_inject_nanos);
        assert!(has_injected_gap, "M6：偶数号批次应当在 made_durable_at 之后、released_at 之前吃到那 30 ms（观测 {gaps_nanos:?}）");
        assert!(has_uninjected_gap, "M6：奇数号批次不该被注入，released_at 应当紧跟 made_durable_at（观测 {gaps_nanos:?}）");
    }
}
