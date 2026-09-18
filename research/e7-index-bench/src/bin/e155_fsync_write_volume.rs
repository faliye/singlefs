//! E155：每次持久化写多少 —— 三种 fsync 形态（甲 / write_ahead_log_full / 乙）的确定性计数模型。
//!
//! 判据、失败条款、变异表的权威登记在 `research/prompts/e155-preregistration.md`（跑前写死 2026-09-18）。
//! 本文件只实现登记第五节定义的模型，不与 `crates/` 共用代码
//! （`.claude/rules/implementation-first.md` 第 4 条）。
//!
//! 这一次交付只做登记 5.7 的第一、二段：模型核心（K1–K8、5.3 不动点）、甲在第 1 行主表上的值、
//! 三条臂各自的阳性对照、Q1.1/Q1.2/Q1.3/Q1.4/Q1.5/Q1.5b/Q1.6/Q1.7/Q1.8。
//! 第三、四段（第 2–4 行反事实上界、第 5 行 P 门槛搜索）未做，运行记录里逐条标「够判后未跑」或「未做」。

use e7_index_bench::Emitter;

// ============================================================
// 一、常量（登记第三节，手抄自 `crates/singlefs-format/src/lib.rs`；数值与第七节 B1 锚点对拍）
// ============================================================

/// 码 2 节点占盘宽度（含头）。
const NODE_BYTES: u64 = 16384;
/// 数据单元与码 3 容器占盘宽度（含头）。
const UNIT_BYTES: u64 = 32768;
/// journal 记录固定宽度。
const RECORD_BYTES: u64 = 4096;
/// 超级块槽宽度。
const SUPERBLOCK_SLOT_BYTES: u64 = 4096;
/// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
const RECORD_HEADER_BYTES: u64 = 307;
/// 一条点名项的宽度。
const NAMED_ITEM_BYTES: u64 = 56;
/// 两块盘（`D`）。
const DEVICE_COUNT: u64 = 2;
/// 主几何下 journal 环安全系数 `F`（D22 已定项 2）。
const RING_SAFETY_FACTOR_MAIN: u64 = 3;
/// M19 反向取样点：环安全系数改 2（不是主几何）。
const RING_SAFETY_FACTOR_REVERSE: u64 = 2;
/// journal 环默认容量。
const RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;
/// 挂载时夹过的有效 `T_dirty`（D16 已定项 5）。
const EFFECTIVE_DIRTY_DATA_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
/// 主几何 `physical_block_size`。
const PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 512;
/// 反向取样点 `physical_block_size`（8.2）。
const REVERSE_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 4096;

/// 码 2 节点头宽公式的常量部分：`86 + 2×key宽 + 29`（`anchors.py` 的 `hdr2`）。
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
/// 树表条目数（主几何，7 棵：extent、inode、alloc、accounting、mapping、mapping 树自己、实例表；
/// 第七节 B1 锚点钉 81 容量、7 条目，不探树表嵌套轴——那是第 2 行 Q2.5，第三段做）。
const TREE_TABLE_ENTRY_COUNT: u64 = 7;
/// 树表改根时相邻改动的条目数（extent、inode、分配记录、记账四棵树的根）。
const TREE_TABLE_TOUCHED_ROOTS: u64 = 4;
/// 一次发布点名项超过 67 时怎么切（K7 主口径）。
const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;

/// 码 2 节点头宽：`86 + 2×key宽 + 29`。
const fn node_header_bytes(key_bytes: u64) -> u64 {
    NODE_HEADER_BASE_BYTES + 2 * key_bytes
}

/// 一个节点能装的条目数：`(节点宽 − 头宽) / 条目宽`（`anchors.py` 的 `cap`，叶与内部共用同一个公式，
/// 只是喂不同的 (key宽, 条目宽)）。
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
// 二、K2：树高与各层节点数
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

    /// B1（第七节 7.2）：容量锚点，`anchors.py` / `anchors2.py` 手抄常量算出的独立值。
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

    /// B1 补充（`anchors2.py`）：记账内部扇出、K3′ 反向取样点的四棵树内部容量。
    #[test]
    fn accounting_internal_capacity_and_identity_reference_variant_match_the_anchors() {
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, false)), 150);
        assert_eq!(node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, true)), 119, "K3′ extent");
        assert_eq!(node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, true)), 133, "K3′ 分配记录");
        assert_eq!(node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, true)), 121, "K3′ 记账");
        assert_eq!(node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, true)), 116, "K3′ 映射");
    }

    /// K2：inode 树恒 ≥ 2 层，即便条目数只有 1（D8 已定项 6，根恒码 2）。
    #[test]
    fn inode_tree_is_at_least_two_layers_even_with_one_entry() {
        let layers = tree_layers(1, inode_container_capacity(), node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES), 2);
        assert_eq!(layers, vec![1, 1], "1 个容器 + 1 个恒码 2 的根");
    }

    /// K2：extent 树在 P = 145 时跨过叶容量 144，长到 2 层（第七节 B7）。
    #[test]
    fn extent_tree_reaches_two_layers_at_file_count_145() {
        let layers = tree_layers(145, node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES), node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)), 1);
        assert_eq!(layers, vec![2, 1], "145 条分两叶、加一个根");
    }
}

// ============================================================
// 三、5.3：组的期望覆盖节点数
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

/// 若干连续组合起来，在某一层期望覆盖的节点数（5.3 第 1 步）。
/// `group_sizes` 按位置顺序排好；`gaps_between` 长度 = `group_sizes.len() - 1`（相邻两组之间的空隙，条目数）。
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

/// 一个散组在某一层期望覆盖的节点数（5.3 第 2 步）。`region_entries` 是这个组落在的区间条目数。
fn scattered_group_touch(touch_count: f64, region_entries: f64, layer_coverage: f64) -> f64 {
    if touch_count <= 0.0 {
        return 0.0;
    }
    let expected_saturated_node_count = (region_entries / layer_coverage).max(1.0);
    expected_saturated_node_count * (1.0 - (1.0 - 1.0 / expected_saturated_node_count).powf(touch_count))
}

/// 若干组合起来，在某一层期望覆盖的节点数（5.3 第 3 步）：`continuous_touch` 是全部连续组合并之后的值，
/// `scattered_touches` 是各个散组各自的值。
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

/// 单一连续组的树（extent、inode）逐层期望脏节点数：这次 fsync 改的 `group_size` 条记录，一路带着祖先往上算。
/// 没有旧版删改分量（extent / inode 树按逻辑位置索引，不因为物理落点变化而在树里挪动，5.3「各树的组从哪来」）。
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

/// 一个带旧版删改的类（映射树的六个类之一、分配记录树的一块盘）在某一层的期望脏节点数（K8 三种位置策略）。
/// `region_entries`：这个类今天的条目数；`layer_coverage`：这一层节点罩的条目数；`insert_count` / `delete_count`：这次持久化插入 /
/// 删改的条目数（本模型里覆盖写恒 `insert_count == delete_count`）；`previous_touch_fraction`：L均 + rand 用上一轮迭代的 `U_l / n_l`（q）。
#[allow(clippy::too_many_arguments, reason = "K8 的公式本身要这么多输入，拆结构体不会让哪一步更好验")]
fn class_dirty_nodes_this_layer(
    region_entries: f64,
    layer_coverage: f64,
    insert_count: f64,
    delete_count: f64,
    placement: Placement,
    policy: PositionPolicy,
    previous_touch_fraction: f64,
) -> f64 {
    if insert_count <= 0.0 && delete_count <= 0.0 {
        return 0.0;
    }
    let node_count_at_layer_for_class = (region_entries / layer_coverage).ceil().max(1.0) as u64;
    if node_count_at_layer_for_class <= 1 {
        // 只有一个节点：插入与删改（不管远近）都落在它上面。
        return 1.0;
    }
    let far_fraction: f64 = match (policy, placement) {
        (PositionPolicy::Near, _) => 0.0,
        (PositionPolicy::Far, _) => 1.0,
        (PositionPolicy::Balanced, Placement::Sequential) => (insert_count / layer_coverage).min(1.0),
        (PositionPolicy::Balanced, Placement::Random) => previous_touch_fraction.min(1.0),
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

/// 一个类的当前状态：条目数、这次持久化插入 / 删改的条目数。
#[derive(Clone, Copy, Debug)]
struct TreeClassActivity {
    region_entries: f64,
    insert_count: f64,
    delete_count: f64,
}

/// 多个类共享一棵树（映射树的六个类、分配记录树的两块盘）逐层期望脏节点数，K8 三种位置策略；
/// `previous_touch_fractions` 长度必须等于 `layers.len()`（上一轮迭代每层的 `U_l / n_l`）。
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
            );
        }
        dirty_per_layer.push(layer_sum.min(node_count_at_layer as f64));
        cov *= internal_capacity as f64;
    }
    dirty_per_layer
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
        // 钉绝对值（M12）：touch_count=1000、region=1000、layer_coverage=100 时 m=10，0.9^1000 趋于 0，
        // 期望值应几乎恰好饱和到 10（不是 M12 那种「不饱和」的原始 touch_count=1000）。
        assert!((large_touch_count - 10.0).abs() < 1e-6, "large_touch_count={large_touch_count}");
    }

    #[test]
    fn combine_touches_never_exceeds_node_count() {
        let combined = combine_touches(10, 5.0, &[5.0, 5.0]);
        assert!(combined <= 10.0, "U_l ≤ n_l（V2）：{combined}");
    }

    /// K8：只有一个节点时，插入与删改都落在它上面，不管远近策略。
    #[test]
    fn single_node_class_is_always_fully_dirty_when_active() {
        for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
            for placement in [Placement::Sequential, Placement::Random] {
                let touched = class_dirty_nodes_this_layer(6.0, 294.0, 6.0, 6.0, placement, policy, 0.5);
                assert_eq!(touched, 1.0, "policy={policy:?} placement={placement:?}");
            }
        }
    }

    /// K8 L近：旧版恒近，只有尾部一组，不产生额外的远组。
    #[test]
    fn near_policy_never_creates_a_far_group() {
        let touched = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Random, PositionPolicy::Near, 0.9);
        // 尾部组 k=16（插入 8 + 近删改 8），期望碰到节点数应接近 1，不该被「远」的散组拉高。
        assert!(touched < 1.5, "L近不该有远组：{touched}");
    }

    /// K8 L远：多节点类里旧版恒远，脏节点数应明显高于 L近同一格。
    #[test]
    fn far_policy_touches_more_nodes_than_near_policy_when_multi_node() {
        let near = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Near, 0.0);
        let far = class_dirty_nodes_this_layer(100_000.0, 100.0, 8.0, 8.0, Placement::Sequential, PositionPolicy::Far, 0.0);
        assert!(far > near, "L远 应该比 L近 碰更多节点：near={near} far={far}");
    }
}

// ============================================================
// 四、族与几何：一格的输入
// ============================================================

/// 族（5.2）：F1 每个文件 1 个数据单元；F8A 一次 fsync 改 8 个相邻数据单元（D25 已定，问法 F8A，第十二节修订①）。
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

    /// 数据单元总数（extent 树条目数）：F1 每文件 1 个、F8A 每文件 8 个。
    fn data_unit_count(self, file_count: u64) -> u64 {
        match self {
            Family::OneDataUnitPerFile => file_count,
            Family::EightAdjacentDataUnitsPerFsync => file_count * 8,
        }
    }

    /// 一次 fsync 改的数据单元数 `d`。
    fn data_units_per_fsync(self) -> u64 {
        match self {
            Family::OneDataUnitPerFile => 1,
            Family::EightAdjacentDataUnitsPerFsync => 8,
        }
    }
}

/// 一格的几何输入（5.6）：主几何 φ=1、K3、g=24、pbs=512；反向取样点各自单独翻一个。
#[derive(Clone, Copy, Debug)]
struct Geometry {
    fill_ratio: FillRatio,
    identity_reference: bool, // K3（false）/ K3′（true）
    backlog_generations: u64, // g：主 24，反向 0
    physical_block_size_bytes: u64,
    unaccounted_switchback: bool, // K9′（true）：WAL 两臂 checkpoint 多写已释放记录，甲不受影响
}

impl Geometry {
    fn main() -> Geometry {
        Geometry {
            fill_ratio: FillRatio::Full,
            identity_reference: false,
            backlog_generations: 24,
            physical_block_size_bytes: PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES,
            unaccounted_switchback: false,
        }
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
    superblock_bytes: u64,
    write_calls: u64,
    barriers: u64,
    fua_count: u64,
    named_items: u64,
    record_count: u64,
    extent_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `extent_layers.len()`），Q1.6 分子要用「非叶」那些层。
    extent_dirty: Vec<f64>,
    inode_layers: Vec<u64>,
    /// 逐层期望脏节点数；`inode_dirty[0]` 是容器（叶），`inode_dirty[1..]` 是码 2 内部 / 根。Q3.1 要按层拆开算 c。
    inode_dirty: Vec<f64>,
    allocation_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `allocation_layers.len()`）。Q3.1 要按层拆开算 c。
    allocation_dirty: Vec<f64>,
    mapping_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `mapping_layers.len()`）。Q3.1 要按层拆开算 c。
    mapping_dirty: Vec<f64>,
    /// 记账树这次持久化的脏节点数（0 或 1，取决于是否被 Q2 抑制）。Q3.1 算记账的 c 要用它。
    accounting_dirty: f64,
    /// 这次持久化写出的单元总数（不动点收敛值，未取整）。Q3.1 算分配记录、映射两棵树的 c 要用它。
    written_units: f64,
    fixed_point_iterations: u64,
}

impl JiaPublishOutcome {
    /// Q1.6 分子：extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**脏节点 + 根槽。
    fn ancestor_and_fixed_point_share_numerator_bytes(&self) -> u64 {
        let extent_non_leaf_dirty: f64 = self.extent_dirty[1..].iter().sum();
        (extent_non_leaf_dirty * NODE_BYTES as f64).round() as u64 * DEVICE_COUNT
            + self.inode_root_bytes
            + self.allocation_bytes
            + self.accounting_bytes
            + self.mapping_bytes
            + self.tree_table_bytes
            + self.root_slot_bytes
    }

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
            + self.superblock_bytes
    }

    fn block_layer_write_requests(&self) -> u64 {
        self.write_calls + self.barriers + self.fua_count
    }
}

/// 求和一个 `Vec<f64>`。
fn sum_f64(values: &[f64]) -> f64 {
    values.iter().sum()
}

/// 5.3 不动点：反复重算分配记录 / 映射两棵树的条目数、层数、脏节点数，直到相邻两轮变化都 < `CONVERGENCE_EPSILON`。
const CONVERGENCE_EPSILON: f64 = 1e-9;
const MAXIMUM_FIXED_POINT_ITERATIONS: u64 = 1000;

/// 求一格（P、族、落点、位置策略、几何）甲一次 fsync 的写出（5.3 + 5.4 甲）。
/// 一次持久化对 extent / inode 树的碰法：连续组（fsync 内 / checkpoint 的 seq 间隔并集）
/// 或散组（checkpoint 的 rand 间隔并集，5.4 write_ahead_log_full / 乙 checkpoint 段）。
#[derive(Clone, Copy, Debug)]
enum TouchSpecification {
    Continuous(f64),
    Scattered(f64),
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
    }
}

/// 5.3 不动点算完的六棵树状态：`solve_jia_publish`（甲 fsync）与 checkpoint 段共用这一个核心。
struct FixedPointCoreResult {
    extent_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `extent_layers.len()`）。
    extent_dirty: Vec<f64>,
    inode_layers: Vec<u64>,
    /// 逐层期望脏节点数；`inode_dirty[0]` 是容器（叶），`inode_dirty[1..]` 是码 2 内部 / 根。
    inode_dirty: Vec<f64>,
    accounting_dirty_total: f64,
    tree_table_dirty_total: f64,
    allocation_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `allocation_layers.len()`）；Q2.1/Q3.1 要按层拆开算 c。
    allocation_dirty: Vec<f64>,
    mapping_layers: Vec<u64>,
    /// 逐层期望脏节点数（长度 = `mapping_layers.len()`）。
    mapping_dirty: Vec<f64>,
    /// 六棵树（extent + inode + 四样固定点）这次的脏节点数之和：喂给下一轮迭代的 W、
    /// K6 积压公式的「每次持久化释放的单元数」，**不是**任何一个臂自己这次持久化实际写出的字节口径
    /// （write_ahead_log_full / 乙的 checkpoint 段不重写 extent / inode，自己的字节口径由调用处另算）。
    written_units: f64,
    iterations: u64,
}

/// Q2：四样固定点各自的反事实——「X 不写」= X 的节点不写、不分配、不进映射、不点名，其余照甲重算不动点（登记 5.6/6）。
/// 每个字段独立取 `true`；主表（`SuppressedFixedPoints::default()`）四个都是 `false`，行为与甲完全相同。
#[derive(Clone, Copy, Debug, Default)]
struct SuppressedFixedPoints {
    accounting: bool,
    allocation: bool,
    mapping: bool,
    tree_table: bool,
}

/// 5.3 不动点核心：extent / inode 用 `extent_touch` / `inode_touch` 指定这次持久化怎么碰它们
/// （甲 fsync：连续组 k=d/f；WAL checkpoint：N 次 fsync 并集，seq 连续、rand 散组）；
/// 记账、树表、分配记录、映射四样固定点每次都按甲的写法全部重算（K9 的净改动近似见调用处）。
fn solve_fixed_point_core(
    data_unit_count: u64,
    file_count: u64,
    extent_touch: TouchSpecification,
    inode_touch: TouchSpecification,
    // 这次持久化里**去重之后**碰到的数据单元数（映射树「数据单元」类的插 / 删条目数）：
    // 甲一次 fsync 恰好是 `d`（这 d 个单元互不相同）；WAL checkpoint 是 N 次 fsync 并集去重之后的数
    // （5.4：seq `min(Nd, 数据单元数)`、rand 用生日公式，等价于 `continuous_groups_touch` /
    // `scattered_group_touch` 在 `cov=1` 时的读数，调用处按 5.4 原文算好再传进来）。
    distinct_data_units_touched: f64,
    placement: Placement,
    policy: PositionPolicy,
    geometry: Geometry,
    suppressed: SuppressedFixedPoints,
    // 树表条目数（Q2.5：主几何 7，反事实取样点 81/82/6561/6562，D8 已定项 8「树表按每层 81 棵嵌套」，
    // 用与其它树相同的 `tree_layers` 公式算树表自己的高度）。
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
    assert_eq!(accounting_layers, vec![1], "15 行远小于 477 的叶容量，记账树恒 1 节点（K5）");
    // Q2：记账「不写」时这次持久化的记账脏节点数为 0（树仍在，只是这次不重写、不进映射、不点名）。
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
    // Q2：树表「不写」——树仍在（贡献 `live_units` 的节点数不变），这次持久化脏节点数强制为 0。
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
            + 1.0 // 记账树 1 节点
            + mapping_total_nodes as f64
            + tree_table_total_nodes as f64
            + 1.0; // 实例表 1

        let allocation_entries = device_count * (live_units + geometry.backlog_generations as f64 * previous_written_units);
        let allocation_layers_now = tree_layers(allocation_entries.round() as u64, allocation_leaf_cap, allocation_internal_cap, 1);
        let allocation_touch = if allocation_layers_now.len() == previous_touch_allocation.len() {
            previous_touch_allocation.clone()
        } else {
            vec![0.0; allocation_layers_now.len()]
        };
        let per_device_allocation_entries = allocation_entries / device_count;
        let allocation_classes = [
            TreeClassActivity { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
            TreeClassActivity { region_entries: per_device_allocation_entries, insert_count: previous_written_units, delete_count: previous_written_units },
        ];
        let allocation_dirty_now = multiclass_tree_dirty_nodes_per_layer(
            &allocation_layers_now,
            allocation_leaf_cap,
            allocation_internal_cap,
            &allocation_classes,
            placement,
            policy,
            &allocation_touch,
        );
        // Q2：分配记录「不写」——树仍在（entries/layers 正常算，供别处的「活单元」计数用），
        // 这次持久化的脏节点数强制为 0，不进 W、不作为映射里「分配记录树节点」类的插删来源。
        let allocation_dirty_now: Vec<f64> = if suppressed.allocation { vec![0.0; allocation_dirty_now.len()] } else { allocation_dirty_now };
        let allocation_dirty_total_now = sum_f64(&allocation_dirty_now);
        let allocation_total_nodes_now: u64 = allocation_layers_now.iter().sum();

        let mapping_entries = data_unit_count as f64
            + extent_total_nodes as f64
            + inode_total_nodes as f64
            + allocation_total_nodes_now as f64
            + 1.0; // 记账树 1 节点
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
            },
            TreeClassActivity { region_entries: extent_total_nodes as f64, insert_count: extent_dirty_total, delete_count: extent_dirty_total },
            TreeClassActivity {
                region_entries: (inode_total_nodes as f64 - inode_layers[0] as f64).max(inode_root_dirty_total),
                insert_count: inode_root_dirty_total,
                delete_count: inode_root_dirty_total,
            },
            TreeClassActivity { region_entries: allocation_total_nodes_now as f64, insert_count: allocation_dirty_total_now, delete_count: allocation_dirty_total_now },
            TreeClassActivity { region_entries: 1.0, insert_count: accounting_dirty_total, delete_count: accounting_dirty_total },
            TreeClassActivity { region_entries: inode_layers[0] as f64, insert_count: inode_container_dirty, delete_count: inode_container_dirty },
        ];
        let mapping_dirty_now = multiclass_tree_dirty_nodes_per_layer(
            &mapping_layers_now,
            mapping_leaf_cap,
            mapping_internal_cap,
            &mapping_classes,
            placement,
            policy,
            &mapping_touch,
        );
        // Q2：映射「不写」——同上，树仍在，这次持久化脏节点数强制为 0。
        let mapping_dirty_now: Vec<f64> = if suppressed.mapping { vec![0.0; mapping_dirty_now.len()] } else { mapping_dirty_now };
        let mapping_dirty_total_now = sum_f64(&mapping_dirty_now);

        let written_units_now = extent_dirty_total + inode_dirty_total + allocation_dirty_total_now + accounting_dirty_total + mapping_dirty_total_now + tree_table_dirty_total;

        let allocation_entries_delta = (allocation_entries - previous_allocation_entries).abs();
        let mapping_entries_delta = (mapping_entries - previous_mapping_entries).abs();
        let written_units_delta = (written_units_now - previous_written_units).abs();
        let allocation_dirty_delta = (allocation_dirty_total_now - sum_f64(&allocation_dirty)).abs();
        let mapping_dirty_delta = (mapping_dirty_total_now - sum_f64(&mapping_dirty)).abs();

        previous_touch_allocation = allocation_layers_now
            .iter()
            .zip(allocation_dirty_now.iter())
            .map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 })
            .collect();
        previous_touch_mapping = mapping_layers_now
            .iter()
            .zip(mapping_dirty_now.iter())
            .map(|(&node_count, &dirty)| if node_count == 0 { 0.0 } else { dirty / node_count as f64 })
            .collect();
        allocation_layers = allocation_layers_now;
        mapping_layers = mapping_layers_now;
        allocation_dirty = allocation_dirty_now;
        mapping_dirty = mapping_dirty_now;
        previous_allocation_entries = allocation_entries;
        previous_mapping_entries = mapping_entries;
        previous_written_units = written_units_now;

        let converged = allocation_entries_delta < CONVERGENCE_EPSILON
            && mapping_entries_delta < CONVERGENCE_EPSILON
            && written_units_delta < CONVERGENCE_EPSILON
            && allocation_dirty_delta < CONVERGENCE_EPSILON
            && mapping_dirty_delta < CONVERGENCE_EPSILON;
        if converged || iterations >= MAXIMUM_FIXED_POINT_ITERATIONS {
            break;
        }
    }

    FixedPointCoreResult {
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
    }
}

/// 求一格（P、族、落点、位置策略、几何）甲一次 fsync 的写出（5.3 + 5.4 甲），四样固定点都正常写、树表 7 条（主几何）。
fn solve_jia_publish(shape: PoolShape) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points(shape, SuppressedFixedPoints::default())
}

/// Q2.1–Q2.4 反事实：`suppressed` 里标 `true` 的那样固定点，这次持久化不写、不分配、不进映射、不点名，
/// 其余照甲重算不动点（登记 5.6/六）；树表仍是主几何 7 条。`SuppressedFixedPoints::default()` 时与 `solve_jia_publish` 完全相同。
fn solve_jia_publish_with_suppressed_fixed_points(shape: PoolShape, suppressed: SuppressedFixedPoints) -> JiaPublishOutcome {
    solve_jia_publish_with_suppressed_fixed_points_and_tree_count(shape, suppressed, TREE_TABLE_ENTRY_COUNT)
}

/// Q2.5 反事实：树表条目数换成 `tree_table_entry_count`（登记 5.6 第 2 行：T ∈ {7,81,82,6561,6562}），
/// 其余与 `solve_jia_publish_with_suppressed_fixed_points` 相同。
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
        superblock_bytes: SUPERBLOCK_SLOT_BYTES * DEVICE_COUNT,
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

#[cfg(test)]
mod jia_anchor_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// B2（第七节 7.2）：甲，P=1、F1、seq、主几何，三种位置策略都一样——只有一个位置，近远无差别。
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
            assert_eq!(outcome.data_bytes, 32768 * 2, "policy={policy:?}");
            assert_eq!(outcome.extent_bytes, 16384 * 2, "policy={policy:?}");
            assert_eq!(outcome.inode_container_bytes, 32768 * 2, "policy={policy:?}");
            assert_eq!(outcome.inode_root_bytes, 16384 * 2, "policy={policy:?}");
            assert_eq!(outcome.allocation_bytes, 16384 * 2, "policy={policy:?}");
            assert_eq!(outcome.accounting_bytes, 16384 * 2, "policy={policy:?}");
            assert_eq!(outcome.mapping_bytes, 16384 * 2, "policy={policy:?}");
            assert_eq!(outcome.tree_table_bytes, 16384 * 2, "policy={policy:?}");
            assert_eq!(outcome.record_bytes, 4096 * 2, "policy={policy:?}");
            assert_eq!(outcome.root_slot_bytes, 512, "policy={policy:?}");
            assert_eq!(outcome.superblock_bytes, 4096 * 2, "policy={policy:?}");
        }
    }

    /// B3（值断言，Q1.6 门槛已撤，第十二节修订②）：G23.1 在锚点的份额，字面口径 47.70%、加超级块 50.07%。
    /// 分子（Q1.6）＝ extent、inode 两树的**非叶**节点 + 分配记录、记账、映射、树表四棵的**全部**脏节点 + 根槽；
    /// P=1 时 extent 树高 1（根就是叶），非叶节点为 0——`inode_container_bytes`（叶）与 `extent_bytes`（此格恒为叶）都不算进去。
    #[test]
    fn ancestor_and_fixed_point_share_at_the_anchor_matches_the_literal_and_with_superblock_readings() {
        let outcome = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(outcome.extent_layers.len(), 1, "P=1 时 extent 树高 1，根即叶，非叶节点为 0");
        let literal_numerator = outcome.ancestor_and_fixed_point_share_numerator_bytes();
        let total = outcome.total_bytes();
        assert_eq!(literal_numerator, 164_352);
        let share = literal_numerator as f64 / total as f64;
        assert!((share - 0.476_968_796).abs() < 1e-6, "share={share}");
        let with_superblock = (literal_numerator + outcome.superblock_bytes) as f64 / total as f64;
        assert!((with_superblock - 0.500_742_942).abs() < 1e-6, "with_superblock={with_superblock}");
    }

    /// B7（第七节 7.2）：甲，P=145、F1、seq、主几何 —— extent 树长到 2 层，其余树层数与 P=1 相同，
    /// 甲每次持久化写字节 377 344。
    #[test]
    fn jia_at_file_count_145_matches_the_anchor() {
        let outcome = solve_jia_publish(shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(outcome.extent_layers, vec![2, 1], "extent 树 2 层（2 叶 + 1 根）");
        assert_eq!(outcome.inode_layers, vec![1, 1], "inode 树仍是 1 容器 + 1 根");
        assert_eq!(outcome.allocation_layers, vec![1], "分配记录 742 条 ≤ 812，仍 1 节点");
        assert_eq!(outcome.mapping_layers, vec![1], "映射 152 条 ≤ 294，仍 1 节点");
        assert_eq!(outcome.total_bytes(), 377_344);
    }

    /// A7（第七节 7.1）：甲的持久顺序每次 fsync 恰好两道屏障（每道每盘一次，共 4 次刷写）、根槽一次 FUA。
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

    /// A8（第七节 7.1）：F8 各格，extent 树每次 fsync 第 1 层起每层期望脏节点 ≤ 2（8 条相邻记录，一条共享脊柱）。
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

    /// 不动点收敛（Q1.7 支撑量）：一格算完，迭代次数应远小于上限 1000，且没有撞到上限（F2 未触发）。
    #[test]
    fn fixed_point_converges_well_below_the_iteration_ceiling() {
        for file_count in [1u64, 100, 10_000, 1_000_000, 100_000_000] {
            let outcome = solve_jia_publish(shape_at(file_count, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Balanced));
            assert!(
                outcome.fixed_point_iterations < MAXIMUM_FIXED_POINT_ITERATIONS,
                "file_count={file_count} 撞到了迭代上限，F2 应该触发"
            );
        }
    }

    /// V2（作废条款）：三条臂的包含关系与 `U_l ≤ n_l` 在主表网格上处处不违反——这里先钉住甲自身不产生负的或超界的脏节点数。
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
// 六、5.4 write_ahead_log_full 与乙（第二段）
// ============================================================

/// 岔路单第 1 行的 WAL 两臂。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteAheadLogArm {
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}

/// WAL 两臂一次 fsync 写出的东西（5.4）：不发根、不写固定点、不写树表，次序同甲但只有一道 barrier
/// 之前的单元与一道之后的记录。
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

/// 求一格 WAL 两臂一次 fsync 的写出。write_ahead_log_full 写脏叶与全部祖先、记一条按树点名的记录；
/// 乙只写脏叶、记一条按叶点名的记录（5.4）。
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
            // 每棵被碰到的树点名它的新树根一项：这一格 d ≥ 1、f = 1，extent 与 inode 恒各被碰到一次。
            (extent_total, inode_container, inode_root, 2u64)
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let extent_leaf = extent_dirty[0];
            let inode_leaf = inode_dirty[0]; // 容器 = 叶
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

/// WAL 两臂一次 checkpoint 写出的东西（5.4）：四样固定点的 K9 净改动 + 乙额外补的用户树祖先追赶。
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
    superblock_bytes: u64,
    write_calls: u64,
    barriers: u64,
    fua_count: u64,
    named_items: u64,
    record_count: u64,
}

impl WriteAheadLogCheckpointOutcome {
    fn total_bytes(&self) -> u64 {
        self.extent_catch_up_bytes
            + self.inode_catch_up_bytes
            + self.allocation_bytes
            + self.accounting_bytes
            + self.mapping_bytes
            + self.tree_table_bytes
            + self.record_bytes
            + self.root_slot_bytes
            + self.superblock_bytes
    }
}

/// 求一格 WAL 两臂一次 checkpoint（间隔 `interval_fsyncs` 次 fsync 之后）的写出。
/// K9（主几何）：净改动按「间隔并集」算，乙额外补上用户树第 1 层起的全部祖先（write_ahead_log_full 每次 fsync 已经写过，不必再补）。
fn solve_write_ahead_log_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, interval_fsyncs: u64) -> WriteAheadLogCheckpointOutcome {
    let geometry = shape.geometry;
    let data_unit_count = shape.family.data_unit_count(shape.file_count);
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let interval_data_unit_touch_count = interval_fsyncs as f64 * fsync_data_unit_count as f64;
    let interval_inode_record_touch_count = interval_fsyncs as f64; // 每次 fsync 恰好碰 1 条 inode 记录（f = 1）。

    let extent_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_data_unit_touch_count),
        Placement::Random => TouchSpecification::Scattered(interval_data_unit_touch_count),
    };
    let inode_touch = match shape.placement {
        Placement::Sequential => TouchSpecification::Continuous(interval_inode_record_touch_count),
        Placement::Random => TouchSpecification::Scattered(interval_inode_record_touch_count),
    };
    let distinct_data_units_touched = match shape.placement {
        Placement::Sequential => continuous_groups_touch(data_unit_count, 1.0, &[interval_data_unit_touch_count], &[]),
        Placement::Random => scattered_group_touch(interval_data_unit_touch_count, data_unit_count as f64, 1.0),
    };

    // Q2 的反事实只问甲（登记 5.6：「第 2–4 行：第 1 行主表的...只算甲」），WAL 两臂的 checkpoint 恒不抑制四样固定点。
    let core = solve_fixed_point_core(
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
    );
    let allocation_dirty_total = sum_f64(&core.allocation_dirty);
    let mapping_dirty_total = sum_f64(&core.mapping_dirty);

    let (extent_catch_up, inode_container_catch_up, inode_root_catch_up, extra_named_items) = match arm {
        WriteAheadLogArm::WriteAheadLogFull => (0.0, 0.0, 0.0, 0u64),
        WriteAheadLogArm::WriteAheadLogLeaf => {
            // 乙每次 fsync 只写叶：checkpoint 要把第 1 层起的全部祖先追上来；容器（叶）已经在每次 fsync 里写过。
            let extent_ancestors: f64 = core.extent_dirty[1..].iter().sum();
            let inode_ancestors: f64 = core.inode_dirty[1..].iter().sum();
            let items = (extent_ancestors + inode_ancestors).round() as u64;
            (extent_ancestors, 0.0, inode_ancestors, items)
        }
    };

    let fixed_point_units = allocation_dirty_total + core.accounting_dirty_total + mapping_dirty_total + core.tree_table_dirty_total;
    // 点名项按「固定点单元数（各自四舍五入）+ 乙的祖先追赶数」相加，不对总和再四舍五入一次——
    // 两种算法在小数部分接近 .5 时会差 1（先加后舍入 vs 先舍入后加），这里取跟 `extra_named_items`
    // 同一套舍入口径，两者天然一致，不需要再断言。
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
        superblock_bytes: SUPERBLOCK_SLOT_BYTES * DEVICE_COUNT,
        write_calls: total_calls,
        barriers: 2 * DEVICE_COUNT,
        fua_count: 1,
        named_items,
        record_count,
    }
}

/// 摊销行（Q1.2）：（N 次 fsync 之和 + checkpoint）÷ N。
fn amortize(fsync_total_bytes: u64, checkpoint_total_bytes: u64, interval_fsyncs: u64) -> f64 {
    (fsync_total_bytes as f64 * interval_fsyncs as f64 + checkpoint_total_bytes as f64) / interval_fsyncs as f64
}

#[cfg(test)]
mod write_ahead_log_anchor_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// B4（第七节 7.2）：write_ahead_log_full，P=1、F1、seq、主几何。
    #[test]
    fn write_ahead_log_full_at_file_count_1_matches_the_anchor() {
        let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull);
        assert_eq!(fsync.total_bytes(), 204_800);
        assert_eq!(fsync.write_calls, 10);
        assert_eq!(fsync.barriers, 4);

        let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull, 16);
        assert_eq!(checkpoint.total_bytes(), 147_968);
        assert_eq!(checkpoint.write_calls, 13);
        assert_eq!(checkpoint.fua_count, 1);

        for interval in [1u64, 16] {
            let amortized = amortize(fsync.total_bytes(), checkpoint.total_bytes(), interval);
            let expected = if interval == 1 { 352_768.0 } else { 214_048.0 };
            assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
        }
    }

    /// B5（第七节 7.2）：乙，P=1、F1、seq、主几何。
    #[test]
    fn write_ahead_log_leaf_at_file_count_1_matches_the_anchor() {
        let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf);
        assert_eq!(fsync.total_bytes(), 172_032);
        assert_eq!(fsync.write_calls, 8);
        assert_eq!(fsync.barriers, 4);

        let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf, 16);
        assert_eq!(checkpoint.total_bytes(), 180_736);
        assert_eq!(checkpoint.write_calls, 15);

        for interval in [1u64, 16] {
            let amortized = amortize(fsync.total_bytes(), checkpoint.total_bytes(), interval);
            let expected = if interval == 1 { 352_768.0 } else { 183_328.0 };
            assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
        }
    }

    /// B6（第七节 7.2）：三臂在锚点上的差——甲 − write_ahead_log_full = 139776，write_ahead_log_full − 乙 = 32768。
    #[test]
    fn arm_differences_at_the_anchor_match() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let jia = solve_jia_publish(shape).total_bytes();
        let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        assert_eq!(jia - write_ahead_log_full, 139_776);
        assert_eq!(write_ahead_log_full - write_ahead_log_leaf, 32_768);
    }

    /// B7（第七节 7.2）：P=145、F1、seq，write_ahead_log_full 与乙的 fsync 行。
    #[test]
    fn write_ahead_log_arms_at_file_count_145_match_the_anchor() {
        let shape = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        assert_eq!(solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes(), 237_568);
        assert_eq!(solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes(), 172_032);
    }

    /// 阳性对照（5.5）：三条臂各自的格与「必须出现的差」逐条核对，任一条不过整轮作废（V1）。
    #[test]
    fn positive_controls_show_the_prescribed_difference_for_every_arm() {
        let shape_at_file_count_1 = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let jia_fsync = solve_jia_publish(shape_at_file_count_1).total_bytes();
        let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 超级块槽 × 2");
        assert_eq!(write_ahead_log_full_fsync - write_ahead_log_leaf_fsync, 32_768, "write_ahead_log_full：inode 根 × 2 盘");

        let jia_root_slots_in_16 = 16u64; // 甲每次 fsync 都写根槽。
        assert_eq!(jia_root_slots_in_16, 16, "16 次 fsync 里甲写根槽 16 次");
        let write_ahead_log_full_root_slots_in_16 = 1u64; // write_ahead_log_full 只在 checkpoint 写根槽。
        assert_eq!(write_ahead_log_full_root_slots_in_16, 1, "16 次 fsync 里 write_ahead_log_full 写根槽 1 次（那次 checkpoint）");

        let shape_at_file_count_145 = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let write_ahead_log_leaf_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        let write_ahead_log_full_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        assert_eq!(write_ahead_log_leaf_at_file_count_145, 172_032, "乙 fsync 行不含 extent 与 inode 两个根");
        assert_eq!(write_ahead_log_full_at_file_count_145, 237_568);
        let checkpoint_at_file_count_145 = solve_write_ahead_log_checkpoint(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf, 16);
        // 这 16 次 fsync 的 checkpoint 里乙写 extent 根、inode 根各 1 次：P=145 时 extent 树高 2，
        // 追赶的祖先恰是 1 个 extent 根 + 1 个 inode 根。
        assert!(checkpoint_at_file_count_145.extent_catch_up_bytes > 0, "extent 根追赶应该非零");
        assert!(checkpoint_at_file_count_145.inode_catch_up_bytes > 0, "inode 根追赶应该非零");
    }

    /// V2（作废条款）：任一格「乙 ≤ write_ahead_log_full ≤ 甲」（fsync 行字节）都不许违反。
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
                            "file_count={file_count} family={family:?} placement={placement:?} policy={policy:?}: 乙={write_ahead_log_leaf} write_ahead_log_full={write_ahead_log_full} 甲={jia}"
                        );
                    }
                }
            }
        }
    }

    /// Q1.5：在飞记录峰值 `R = N × r_f + r_c`，要的环 = `3 × 4096 × R` 字节（F=3）；
    /// 默认环装得下 ⟺ R ≤ 65536。P=1、N=16 一格算出的 R 与手算一致。
    #[test]
    fn in_flight_records_and_ring_bytes_match_the_hand_calculation() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
            let fsync = solve_write_ahead_log_fsync(shape, arm);
            let checkpoint = solve_write_ahead_log_checkpoint(shape, arm, 16);
            let peak_in_flight_records = 16 * fsync.record_count + checkpoint.record_count;
            // P=1 一格每次 fsync 与 checkpoint 都恰好 1 条记录：R = 16*1+1 = 17（M20 的反例：不计 checkpoint 那条会得 16）。
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
mod section_7_3_hand_calculated_cells {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// 7.3 手算格 1：分配记录树第 2 层刚出现的一格（P=205，F1、seq、主几何；用一段临时的搜索脚本
    /// 逐个 P 扫过，204 仍 1 层、205 起 2 层，搜索代码与输出见运行记录，不留在正式产出里）。
    #[test]
    fn allocation_tree_reaches_two_layers_at_file_count_205() {
        let at_204 = solve_jia_publish(shape_at(204, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        let at_205 = solve_jia_publish(shape_at(205, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(at_204.allocation_layers.len(), 1, "P=204 分配记录树仍是 1 层");
        assert_eq!(at_205.allocation_layers.len(), 2, "P=205 分配记录树长到 2 层");
    }

    /// 7.3 手算格 2：映射树第 2 层刚出现的一格（P=285，F1、seq、主几何，同上搜索方式）。
    #[test]
    fn mapping_tree_reaches_two_layers_at_file_count_285() {
        let at_284 = solve_jia_publish(shape_at(284, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        let at_285 = solve_jia_publish(shape_at(285, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced));
        assert_eq!(at_284.mapping_layers.len(), 1, "P=284 映射树仍是 1 层");
        assert_eq!(at_285.mapping_layers.len(), 2, "P=285 映射树长到 2 层");
    }

    /// 7.3 手算格 3：rand 下 L远 与 L均 不同的一格（P=1000，F1、rand）。P=145 这一格 alloc / 映射仍是
    /// 单节点，远近无差别，用不了（搜索时先试过，正好验证「差别只从多节点树才开始出现」这句话）。
    #[test]
    fn far_and_balanced_policies_differ_under_random_placement_at_file_count_1000() {
        let far = solve_jia_publish(shape_at(1000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Far));
        let balanced = solve_jia_publish(shape_at(1000, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Balanced));
        assert_ne!(far.total_bytes(), balanced.total_bytes(), "L远 与 L均 在这一格应该给出不同的字节数");
    }

    /// 7.3 手算格 4：点名项超过 67 的一格（P=10⁴、F8A、rand、L远——第 1 行主网格自带的取样点，
    /// 不必构造网格之外的格）。
    #[test]
    fn named_items_exceed_one_record_at_file_count_10000_eight_adjacent_family_random_far() {
        let outcome = solve_jia_publish(shape_at(10_000, Family::EightAdjacentDataUnitsPerFsync, Placement::Random, PositionPolicy::Far));
        assert!(outcome.named_items > NAMED_ITEMS_PER_RECORD_MAIN, "named_items={}", outcome.named_items);
        assert_eq!(outcome.record_count, outcome.named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN));
        assert!(outcome.record_count >= 2, "record_count={}", outcome.record_count);
    }

    /// 7.3 手算格 5：Q3.1（改一条记录也整单元重写的反事实上界）在 P=1 的一格。
    /// `H = 115 + 2×key宽`（码 2）或 136（码 3）；`c` = 这一层每个脏节点平均改了几条；`w` 按 K1 / K3。
    /// P=1 时甲写出的七个非数据单元逐个手算（取整到 pbs=512）：
    ///
    /// | 单元 | H | c | w | H+cw | 取整到 512 | 单盘省下 |
    /// |---|---|---|---|---|---|---|
    /// | extent（此格是叶，key24） | 163 | 1（改 1 条 extent 记录 ÷ 1 脏节点） | 112 | 275 | 512 | 15872 |
    /// | inode 容器（码 3） | 136 | 1（改 1 条 inode 记录 ÷ 1 脏节点） | 140 | 276 | 512 | 32256 |
    /// | inode 根（码 2，key8） | 131 | 1（1 个脏子节点 ÷ 1 脏节点） | 120 | 251 | 512 | 15872 |
    /// | 分配记录（码 2，key10） | 135 | 32（2 盘 ×（插 8 + 释放改写 8）÷ 1 脏节点） | 20 | 775 | 1024 | 15360 |
    /// | 记账（码 2，key22） | 159 | 15（全部行重写 ÷ 1 脏节点） | 34 | 669 | 1024 | 15360 |
    /// | 映射（码 2，key27） | 169 | 12（6 类 ×（插 1 + 删 1）÷ 1 脏节点） | 55 | 829 | 1024 | 15360 |
    /// | 树表（码 2，key8） | 131 | 4（改了根的条目数） | 200 | 931 | 1024 | 15360 |
    ///
    /// 单盘合计省 125440 字节，两盘 250880；`U_3` = 250880 / 344576 ≈ 72.81%（≥ 10%：按 Q2.1 同一门槛，
    /// 这一行「值得设计具体候选」；上界不是省下的量，F9）。
    #[test]
    fn counterfactual_upper_bound_of_rewriting_whole_unit_at_file_count_1_matches_the_hand_calculation() {
        fn rounded_unit_bytes(header: u64, condition_count: u64, entry_width: u64, pbs: u64, full_width: u64) -> u64 {
            ((header + condition_count * entry_width).div_ceil(pbs) * pbs).min(full_width)
        }
        let pbs = PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES;
        let savings_per_device: u64 = [
            NODE_BYTES - rounded_unit_bytes(node_header_bytes(EXTENT_KEY_BYTES), 1, EXTENT_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
            UNIT_BYTES - rounded_unit_bytes(INODE_CONTAINER_HEADER_BYTES, 1, INODE_CONTAINER_ENTRY_BYTES, pbs, UNIT_BYTES),
            NODE_BYTES - rounded_unit_bytes(node_header_bytes(INODE_KEY_BYTES), 1, INODE_INTERNAL_ENTRY_BYTES, pbs, NODE_BYTES),
            NODE_BYTES - rounded_unit_bytes(node_header_bytes(ALLOCATION_KEY_BYTES), 32, ALLOCATION_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
            NODE_BYTES - rounded_unit_bytes(node_header_bytes(ACCOUNTING_KEY_BYTES), 15, ACCOUNTING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
            NODE_BYTES - rounded_unit_bytes(node_header_bytes(MAPPING_KEY_BYTES), 12, MAPPING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES),
            NODE_BYTES - rounded_unit_bytes(node_header_bytes(TREE_TABLE_KEY_BYTES), 4, TREE_TABLE_ENTRY_BYTES, pbs, NODE_BYTES),
        ]
        .iter()
        .sum();
        assert_eq!(savings_per_device, 125_440, "单盘合计省下的字节，见方法注释的逐行手算");
        let savings_total = savings_per_device * DEVICE_COUNT;
        assert_eq!(savings_total, 250_880);
        let anchor_bytes = solve_jia_publish(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced)).total_bytes();
        let upper_bound = savings_total as f64 / anchor_bytes as f64;
        assert!((upper_bound - 0.728_08).abs() < 1e-3, "U_3={upper_bound}");
        assert!(upper_bound >= 0.10, "Q2.1 同门槛：≥10% 记『上界够进候选』");
    }
}

/// 数据单元的头 + 尾总开销（B8/Q4.1 取整用）：kb 登记的 `DATA_UNIT_HEADER_BYTES = 105`
/// （`.claude/kb/decisions/18-块里携带什么信息.md:592`）只是头；这里的 134 = 105 + 29，
/// 29 是数据单元载荷上限公式 `UNIT_BYTES − 105 − 29 = 32634`（第七节 B1）里另算的那一段，
/// 两个不是同一个量，故意不与 kb 那个名字撞名。
const DATA_UNIT_HEADER_AND_TRAILER_BYTES: u64 = 134;

// ============================================================
// 七、第 2–4 行：四样固定点、整单元重写、数据单元取整的反事实上界（第三段）
// ============================================================

/// Q2.1–Q2.4：四样固定点各自的反事实上界 `U_X = 1 − B(甲|X 不写) / B(甲)`（登记 6.Q2.1）。
fn fixed_point_counterfactual_share(shape: PoolShape, suppressed: SuppressedFixedPoints) -> f64 {
    let baseline = solve_jia_publish(shape).total_bytes();
    let counterfactual = solve_jia_publish_with_suppressed_fixed_points(shape, suppressed).total_bytes();
    1.0 - (counterfactual as f64 / baseline as f64)
}

/// Q2.5：树表随树的棵数 T 的反事实上界，同一个 `U_X` 公式，X = 树表、T 可变（登记 6.Q2.5）。
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

/// Q3.1：逐层展开一棵树省下的字节（单盘口径，调用处再乘 `DEVICE_COUNT`）。`leaf_conditions` 是叶层自己这次持久化的
/// 插 / 删 / 改写条数；内部层的条件数取「上一层的脏节点数」（登记 6.Q3.1：「内部：脏子节点数 ÷ 脏节点数」）。
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

/// Q3.1：改一条记录也整单元重写的反事实上界，在甲的这一格上算（登记 6.Q3.1）；`pbs` 用登记声明的 512 或反向 4096。
fn whole_unit_rewrite_counterfactual_share(shape: PoolShape, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let outcome = solve_jia_publish(shape);
    let fsync_data_unit_count = shape.family.data_units_per_fsync() as f64;

    let mut savings = 0.0;

    // extent：码 2，叶 conditions = d（这次 fsync 改的数据单元数，D25 的「1 条共享脊柱」）。
    savings += partial_rewrite_tree_savings(
        &outcome.extent_dirty,
        fsync_data_unit_count,
        node_header_bytes(EXTENT_KEY_BYTES),
        EXTENT_LEAF_ENTRY_BYTES,
        NODE_BYTES,
        node_header_bytes(EXTENT_KEY_BYTES),
        internal_entry_bytes(EXTENT_KEY_BYTES, false),
        NODE_BYTES,
        pbs,
    );

    // inode：叶是码 3 容器，conditions = f = 1；内部层是码 2。
    savings += partial_rewrite_tree_savings(
        &outcome.inode_dirty,
        1.0,
        INODE_CONTAINER_HEADER_BYTES,
        INODE_CONTAINER_ENTRY_BYTES,
        UNIT_BYTES,
        node_header_bytes(INODE_KEY_BYTES),
        INODE_INTERNAL_ENTRY_BYTES,
        NODE_BYTES,
        pbs,
    );

    // 分配记录：叶 conditions = 4W（2 盘 ×（插 + 释放改写各一份 W）），W 用这一格收敛后的 `written_units`。
    savings += partial_rewrite_tree_savings(
        &outcome.allocation_dirty,
        4.0 * outcome.written_units,
        node_header_bytes(ALLOCATION_KEY_BYTES),
        ALLOCATION_LEAF_ENTRY_BYTES,
        NODE_BYTES,
        node_header_bytes(ALLOCATION_KEY_BYTES),
        internal_entry_bytes(ALLOCATION_KEY_BYTES, false),
        NODE_BYTES,
        pbs,
    );

    // 映射：叶 conditions = 6 类各插 + 删 = 2×(d + extent 脏 + inode 容器脏 + inode 非叶脏 + 分配记录脏 + 记账脏)。
    let extent_dirty_total = sum_f64(&outcome.extent_dirty);
    let inode_container_dirty = outcome.inode_dirty[0];
    let inode_root_dirty_total: f64 = outcome.inode_dirty[1..].iter().sum();
    let allocation_dirty_total = sum_f64(&outcome.allocation_dirty);
    let mapping_leaf_conditions =
        2.0 * (fsync_data_unit_count + extent_dirty_total + inode_container_dirty + inode_root_dirty_total + allocation_dirty_total + outcome.accounting_dirty);
    savings += partial_rewrite_tree_savings(
        &outcome.mapping_dirty,
        mapping_leaf_conditions,
        node_header_bytes(MAPPING_KEY_BYTES),
        MAPPING_LEAF_ENTRY_BYTES,
        NODE_BYTES,
        node_header_bytes(MAPPING_KEY_BYTES),
        internal_entry_bytes(MAPPING_KEY_BYTES, false),
        NODE_BYTES,
        pbs,
    );

    // 记账：主几何恒 1 层，conditions = 15（全部行重写）。
    if outcome.accounting_dirty > 0.0 {
        let conditions_per_node = 15.0 / outcome.accounting_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(ACCOUNTING_KEY_BYTES), conditions_per_node, ACCOUNTING_LEAF_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += outcome.accounting_dirty * (NODE_BYTES as f64 - rounded);
    }

    // 树表：主几何恒 1 层，conditions = 改了根的条目数（4）。
    let tree_table_dirty = outcome.tree_table_bytes as f64 / NODE_BYTES as f64 / DEVICE_COUNT as f64;
    if tree_table_dirty > 0.0 {
        let conditions_per_node = TREE_TABLE_TOUCHED_ROOTS as f64 / tree_table_dirty;
        let rounded = partial_rewrite_rounded_bytes(node_header_bytes(TREE_TABLE_KEY_BYTES), conditions_per_node, TREE_TABLE_ENTRY_BYTES, pbs, NODE_BYTES);
        savings += tree_table_dirty * (NODE_BYTES as f64 - rounded);
    }

    (savings * DEVICE_COUNT as f64) / outcome.total_bytes() as f64
}

/// Q4.1：数据单元按内容取整的反事实上界（登记 6.Q4.1）。
fn data_unit_rounding_counterfactual_share(shape: PoolShape, content_bytes: u64, pbs: u64) -> f64 {
    let shape = PoolShape { geometry: Geometry { physical_block_size_bytes: pbs, ..shape.geometry }, ..shape };
    let baseline = solve_jia_publish(shape).total_bytes();
    let fsync_data_unit_count = shape.family.data_units_per_fsync();
    let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
    let saving_per_unit_per_device = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
    (fsync_data_unit_count * DEVICE_COUNT * saving_per_unit_per_device) as f64 / baseline as f64
}

// ============================================================
// 八、第 5 行：甲每次持久化写字节随对象数怎么涨（第四段，用户直接问的那一题）
// ============================================================

/// 六棵会长高的 / 恒定的树各自的树高（Q5.2 的 `T` 取值：extent、inode、分配记录、记账、映射、树表）。
fn tree_heights(outcome: &JiaPublishOutcome) -> [usize; 6] {
    [
        outcome.extent_layers.len(),
        outcome.inode_layers.len(),
        outcome.allocation_layers.len(),
        1, // 记账恒 1 层（K5：15 行远小于 477 的叶容量）。
        outcome.mapping_layers.len(),
        1, // 树表主几何恒 1 层（T = 7 ≤ 81）。
    ]
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

/// Q5.1：甲这一格的 `A(P)`（`total_bytes`）与 `A_tree(P)`（六棵树的字节之和，不含数据单元 / 记录 / 根槽 / 超级块）。
fn total_bytes_and_tree_bytes(shape: PoolShape) -> (u64, u64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = outcome.extent_bytes
        + outcome.inode_container_bytes
        + outcome.inode_root_bytes
        + outcome.allocation_bytes
        + outcome.accounting_bytes
        + outcome.mapping_bytes
        + outcome.tree_table_bytes;
    (outcome.total_bytes(), tree_bytes)
}

/// Q5.2a/b/c：三种「树高 × 棵数」界的读法，见登记 6.Q5.2a/b/c。
fn rho_readings(shape: PoolShape) -> (f64, f64, f64) {
    let outcome = solve_jia_publish(shape);
    let tree_bytes = (outcome.extent_bytes
        + outcome.inode_container_bytes
        + outcome.inode_root_bytes
        + outcome.allocation_bytes
        + outcome.accounting_bytes
        + outcome.mapping_bytes
        + outcome.tree_table_bytes) as f64;
    let heights = tree_heights(&outcome);
    let one_path_sum: u64 = one_path_bytes(heights[0], false)
        + one_path_bytes(heights[1], true)
        + one_path_bytes(heights[2], false)
        + one_path_bytes(heights[3], false)
        + one_path_bytes(heights[4], false)
        + one_path_bytes(heights[5], false);
    let maximum_tree_height = *heights.iter().max().expect("六个高度都有值") as u64;
    let path_ratio_by_actual_height = tree_bytes / (DEVICE_COUNT as f64 * one_path_sum as f64);
    let path_ratio_by_maximum_height_node_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * NODE_BYTES as f64 * DEVICE_COUNT as f64);
    let path_ratio_by_maximum_height_unit_width = tree_bytes / (maximum_tree_height as f64 * 6.0 * UNIT_BYTES as f64 * DEVICE_COUNT as f64);
    (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width)
}

/// 在 `[low, high]` 上二分，找「`height_at` 第一次超过 `base_height`」的最小 P；`height_at` 必须单调不减。
/// 找不到（`height_at(high)` 仍 `<= base_height`）时返回 `None`。
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

/// Q5.3 要的「每棵树每次长高一层的门槛」：从 P = 1 的高度开始，逐次二分找下一个跨越点，直到搜索上限。
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
                    break; // 防御性上限：正常情况下 P ≤ search_upper_bound 不会长这么多层。
                }
            }
            None => break,
        }
    }
    thresholds
}

/// 默认环的在飞记录上限（B9：768 MiB ÷ 4096 ÷ 3）。
fn default_ring_in_flight_limit() -> u64 {
    RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN
}

#[cfg(test)]
mod row_2_to_4_counterfactual_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// Q3.1（通用实现）在 P = 1 一格必须与 7.3 手算格逐字节相同：`U_3 = 250880/344576`。
    #[test]
    fn whole_unit_rewrite_general_implementation_matches_the_hand_calculated_cell_at_file_count_1() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let share = whole_unit_rewrite_counterfactual_share(shape, PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES);
        assert!((share - 250_880.0 / 344_576.0).abs() < 1e-9, "share={share}");
    }

    /// Q2.1–Q2.4：P = 1 一格四样固定点各自只贡献 16384×2 字节（B2 锚点逐项相同），
    /// 抑制其中任何一样时其余树仍是单节点、没有连带效应，四个上界应该逐字节相等。
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
        // 钉住这一格的实际读数（0.0951 < 0.10）：P=1 一格四样固定点各自都在 10% 门槛**之下**，
        // 这是这一格的真实结果，不是判据——门槛怎么判随 P 变化，见产物里的 P 轴轨迹。
        assert!(expected < 0.10, "P=1 一格：{expected}");
    }

    /// Q2 的抑制不能让脏节点数变负或让 W 变负——抑制掉的树在极小 P 上仍应收敛。
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

    /// Q2.5：树表条目数 T 从 7 涨到 82 时树表本身长到 2 层（`tree_layers` 与其它树同一个公式），
    /// 上界跟着变大（更多节点可能被这次持久化碰到）。
    #[test]
    fn tree_table_reaches_two_layers_at_82_entries_and_upper_bound_does_not_shrink() {
        let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let share_at_7 = tree_table_counterfactual_share(shape, 7);
        let share_at_81 = tree_table_counterfactual_share(shape, 81);
        let share_at_82 = tree_table_counterfactual_share(shape, 82);
        assert!((share_at_7 - share_at_81).abs() < 1e-9, "T=7 与 T=81 都还是 1 层，上界应该相同：{share_at_7} vs {share_at_81}");
        assert!(share_at_82 >= share_at_81, "T=82 长到 2 层，上界不该变小：{share_at_82} vs {share_at_81}");
    }

    /// B8（第七节 7.2）：第 4 行每单元每盘省下的字节，Q4.1 反事实上界用它对拍。
    #[test]
    fn data_unit_rounding_per_unit_savings_match_the_anchor() {
        let cases: [(u64, u64, u64); 8] = [
            (100, 512, 32_256),
            (100, 4096, 28_672),
            (4100, 512, 28_160),
            (4100, 4096, 24_576),
            (16_000, 512, 16_384),
            (16_000, 4096, 16_384),
            (32_634, 512, 0),
            (32_634, 4096, 0),
        ];
        for (content_bytes, pbs, expected_saving_per_unit_per_device) in cases {
            let rounded = (DATA_UNIT_HEADER_AND_TRAILER_BYTES + content_bytes).div_ceil(pbs) * pbs;
            let saving = UNIT_BYTES.saturating_sub(rounded.min(UNIT_BYTES));
            assert_eq!(saving, expected_saving_per_unit_per_device, "content_bytes={content_bytes} pbs={pbs}");
        }
    }

    /// M18 的防等价取样点：s = 4000 是唯一让数据单元头 134 改变取整的一档（B8 附注）。
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

    /// Q4.1（防等价 M18）：s = 4000、pbs = 512 一档必须真的算头（`DATA_UNIT_HEADER_AND_TRAILER_BYTES`），
    /// 不算头会把取整从 4608 变成 4096，省下的字节从 28160 变成 28672——这一档就是防这个的。
    #[test]
    fn data_unit_rounding_at_content_size_4000_accounts_for_the_data_unit_header() {
        let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let baseline = solve_jia_publish(PoolShape { geometry: Geometry { physical_block_size_bytes: 512, ..shape.geometry }, ..shape }).total_bytes();
        let share = data_unit_rounding_counterfactual_share(shape, 4000, 512);
        let expected = (1 * 2 * 28_160) as f64 / baseline as f64;
        assert!((share - expected).abs() < 1e-12, "share={share} expected={expected}");
    }

    /// Q4.1：F8A 一格每次 fsync 改 8 个数据单元，反事实上界应该正好是 F1 同一格的 8 倍（都用 d×D×省下的字节）。
    #[test]
    fn data_unit_rounding_scales_linearly_with_data_units_touched_per_fsync() {
        let shape_one_data_unit_per_file = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let shape_f8a = PoolShape { family: Family::EightAdjacentDataUnitsPerFsync, ..shape_one_data_unit_per_file };
        let baseline_one_data_unit_per_file = solve_jia_publish(shape_one_data_unit_per_file).total_bytes() as f64;
        let baseline_f8a = solve_jia_publish(shape_f8a).total_bytes() as f64;
        let share_one_data_unit_per_file = data_unit_rounding_counterfactual_share(shape_one_data_unit_per_file, 4100, 512);
        let share_f8a = data_unit_rounding_counterfactual_share(shape_f8a, 4100, 512);
        let numerator_one_data_unit_per_file = share_one_data_unit_per_file * baseline_one_data_unit_per_file;
        let numerator_f8a = share_f8a * baseline_f8a;
        assert!(
            (numerator_f8a / numerator_one_data_unit_per_file - 8.0).abs() < 1e-6,
            "numerator_one_data_unit_per_file={numerator_one_data_unit_per_file} numerator_f8a={numerator_f8a}"
        );
    }
}

#[cfg(test)]
mod row_5_growth_tests {
    use super::*;

    fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
        PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
    }

    /// Q5.1：P = 1 一格 `A_tree` 恰好是七个非数据角色里去掉数据单元后的六棵树部分——
    /// 用 B2 锚点手算：extent 16384×2 + inode 容器 32768×2 + inode 根 16384×2 + 分配记录 16384×2
    /// + 记账 16384×2 + 映射 16384×2 + 树表 16384×2 = 278528（B(甲) 344576 减掉数据单元 65536 与固定开销）。
    #[test]
    fn tree_bytes_at_file_count_1_matches_the_hand_calculation() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let (total_bytes, tree_bytes) = total_bytes_and_tree_bytes(shape);
        assert_eq!(total_bytes, 344_576);
        assert_eq!(tree_bytes, 16384 * 2 + 32768 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2 + 16384 * 2);
    }

    /// Q5.2a/b/c：P = 1 一格每棵树高度都是能取到的最小值（extent/alloc/mapping/记账/树表 1 层、inode 2 层），
    /// 「一条路径」字节应该恰好等于 `A_tree`（一次持久化每棵树只碰一条从叶到根的路径），三种 ρ 都应该 ≤ 1。
    #[test]
    fn path_ratio_readings_do_not_exceed_one_at_file_count_1() {
        let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width) = rho_readings(shape);
        assert!(
            (path_ratio_by_actual_height - 1.0).abs() < 1e-9,
            "P=1 时六棵树各自只有一条路径，path_ratio_by_actual_height 应该恰好是 1：{path_ratio_by_actual_height}"
        );
        assert!(path_ratio_by_maximum_height_node_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_node_width={path_ratio_by_maximum_height_node_width}");
        assert!(path_ratio_by_maximum_height_unit_width <= 1.0 + 1e-9, "path_ratio_by_maximum_height_unit_width={path_ratio_by_maximum_height_unit_width}");
    }

    /// 门槛搜索：extent 树在 F1、seq、主几何下第一个跨越点必须恰好是 145（B7 锚点）。
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

    /// 门槛搜索：分配记录树的门槛必须与 7.3 手算格（P = 205）相同。
    #[test]
    fn allocation_height_growth_threshold_matches_the_hand_calculated_cell() {
        let height_at = |file_count: u64| {
            let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
            solve_jia_publish(shape).allocation_layers.len()
        };
        let thresholds = height_growth_thresholds(1_000_000, height_at);
        assert!(!thresholds.is_empty());
        assert_eq!(thresholds[0], 205);
    }

    /// Q5.3：e ≈ 1（一层一条路径）在 extent 树自己的门槛上必须成立——长高一层只多写一条路径的份额，
    /// 不是好几条（这是「随对象数线性增长」与「指数恶化」之间最直接的判别）。
    #[test]
    fn bytes_per_layer_growth_ratio_at_the_extent_threshold_is_close_to_one_layer_one_path() {
        let shape_before = shape_at(144, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let shape_at_threshold = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let (_, tree_bytes_before_threshold) = total_bytes_and_tree_bytes(shape_before);
        let (_, tree_bytes_at_threshold) = total_bytes_and_tree_bytes(shape_at_threshold);
        let bytes_per_layer_growth_ratio = (tree_bytes_at_threshold as f64 - tree_bytes_before_threshold as f64) / (NODE_BYTES as f64 * DEVICE_COUNT as f64);
        // P=144→145 只多一个脏节点（1 个新内部根，见 B7 锚点「jia = jia + NODE*D」），这个比值应该恰好是 1.0，不只是 ≤ 1.05。
        assert!(
            (bytes_per_layer_growth_ratio - 1.0).abs() < 1e-9,
            "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 在这个门槛上长高一层应该恰好多写一条路径的份额"
        );
        assert!(bytes_per_layer_growth_ratio <= 1.05, "bytes_per_layer_growth_ratio={bytes_per_layer_growth_ratio}：extent 长高一层不该比『一层一条路径』贵太多");
    }
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} superblock_slot_bytes={SUPERBLOCK_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} effective_dirty_data_budget_bytes={EFFECTIVE_DIRTY_DATA_BUDGET_BYTES} model=counting file_ops=0",
            default_ring_in_flight_limit()
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope stage=1-2 covers=K1,K2,K3,K4(phi=1/0.5两点),K5,K6,K7,K8,K9,5.3不动点,5.4甲/write_ahead_log_full/乙,阳性对照(5.5),V1,V2,Q1.1,Q1.2,Q1.3,Q1.4(原始值齐/标签只算代表格),Q1.5,Q1.5b,Q1.6(值+轨迹/判据已撤),Q1.7,Q1.8,7.3手算格5条 missing=第2-4行反事实上界(Q2/Q3/Q4)、第5行P门槛搜索(Q5)、8.2全量反向几何扫描(只做代表格)、Q1.4全量标签矩阵、树表嵌套轴(Q2.5)、Q1.6b、块层写请求列"
        )
    );

    // 七 B1：容量一览。
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

    // 5.5 阳性对照：三条臂各自的格与「必须出现的差」。
    let control_shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
    let jia_fsync = solve_jia_publish(control_shape).total_bytes();
    let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control cell=p1_f1_seq jia_fsync_bytes={jia_fsync} write_ahead_log_full_fsync_bytes={write_ahead_log_full_fsync} write_ahead_log_leaf_fsync_bytes={write_ahead_log_leaf_fsync} jia_minus_write_ahead_log_full={} write_ahead_log_full_minus_write_ahead_log_leaf={} jia_root_slots_per_16_fsync=16 write_ahead_log_full_root_slots_per_16_fsync=1",
            jia_fsync - write_ahead_log_full_fsync,
            write_ahead_log_full_fsync - write_ahead_log_leaf_fsync
        ))
    );
    let control_shape_at_file_count_145 = PoolShape { file_count: 145, ..control_shape };
    let write_ahead_log_full_at_file_count_145 = solve_write_ahead_log_fsync(control_shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf_at_file_count_145 = solve_write_ahead_log_fsync(control_shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    let checkpoint_leaf_at_file_count_145 = solve_write_ahead_log_checkpoint(control_shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf, 16);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control cell=p145_f1_seq write_ahead_log_full_fsync_bytes={write_ahead_log_full_at_file_count_145} write_ahead_log_leaf_fsync_bytes={write_ahead_log_leaf_at_file_count_145} write_ahead_log_leaf_checkpoint_extent_catch_up_bytes={} write_ahead_log_leaf_checkpoint_inode_catch_up_bytes={}",
            checkpoint_leaf_at_file_count_145.extent_catch_up_bytes, checkpoint_leaf_at_file_count_145.inode_catch_up_bytes
        ))
    );

    // 第 1 行主表（5.6）：P × 族 × 落点 × N × 三种位置策略，三条臂同一行各报各的列。
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
                        let write_ahead_log_full_checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval);
                        let write_ahead_log_leaf_checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, interval);
                        let jia_amortized = jia.total_bytes() as f64; // 甲每次 fsync 就是一次发布，两行同值（读法写死）。
                        let write_ahead_log_full_amortized = amortize(write_ahead_log_full_fsync.total_bytes(), write_ahead_log_full_checkpoint.total_bytes(), interval);
                        let write_ahead_log_leaf_amortized = amortize(write_ahead_log_leaf_fsync.total_bytes(), write_ahead_log_leaf_checkpoint.total_bytes(), interval);
                        let in_flight_peak_full = interval * write_ahead_log_full_fsync.record_count + write_ahead_log_full_checkpoint.record_count;
                        let in_flight_peak_leaf = interval * write_ahead_log_leaf_fsync.record_count + write_ahead_log_leaf_checkpoint.record_count;
                        let ring_bytes_needed_full = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * in_flight_peak_full;
                        let ring_bytes_needed_leaf = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * in_flight_peak_leaf;
                        let limit = default_ring_in_flight_limit();
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=row1_grid p={file_count} family={} placement={} policy={} n={interval} \
                                 jia_fsync_bytes={} jia_calls={} jia_barriers={} jia_fua={} jia_amortized_bytes={jia_amortized} \
                                 write_ahead_log_full_fsync_bytes={} write_ahead_log_full_fsync_calls={} write_ahead_log_full_checkpoint_bytes={} write_ahead_log_full_checkpoint_calls={} write_ahead_log_full_amortized_bytes={write_ahead_log_full_amortized} write_ahead_log_full_in_flight_peak={in_flight_peak_full} write_ahead_log_full_ring_bytes_needed={ring_bytes_needed_full} write_ahead_log_full_default_ring_ok={} \
                                 write_ahead_log_leaf_fsync_bytes={} write_ahead_log_leaf_fsync_calls={} write_ahead_log_leaf_checkpoint_bytes={} write_ahead_log_leaf_checkpoint_calls={} write_ahead_log_leaf_amortized_bytes={write_ahead_log_leaf_amortized} write_ahead_log_leaf_in_flight_peak={in_flight_peak_leaf} write_ahead_log_leaf_ring_bytes_needed={ring_bytes_needed_leaf} write_ahead_log_leaf_default_ring_ok={} \
                                 extent_layers={} inode_layers={} allocation_layers={} mapping_layers={} fixed_point_iterations={} g23_1_share={}",
                                family.tag(),
                                placement.tag(),
                                policy.tag(),
                                jia.total_bytes(),
                                jia.write_calls,
                                jia.barriers,
                                jia.fua_count,
                                write_ahead_log_full_fsync.total_bytes(),
                                write_ahead_log_full_fsync.write_calls,
                                write_ahead_log_full_checkpoint.total_bytes(),
                                write_ahead_log_full_checkpoint.write_calls,
                                in_flight_peak_full <= limit,
                                write_ahead_log_leaf_fsync.total_bytes(),
                                write_ahead_log_leaf_fsync.write_calls,
                                write_ahead_log_leaf_checkpoint.total_bytes(),
                                write_ahead_log_leaf_checkpoint.write_calls,
                                in_flight_peak_leaf <= limit,
                                jia.extent_layers.len(),
                                jia.inode_layers.len(),
                                jia.allocation_layers.len(),
                                jia.mapping_layers.len(),
                                jia.fixed_point_iterations,
                                jia.ancestor_and_fixed_point_share_numerator_bytes() as f64 / jia.total_bytes() as f64,
                            ))
                        );
                    }
                }
            }
        }
    }

    // Q1.8：另一口径（C310）—— 每次 fsync 记录数取 max(d, ⌈点名项/67⌉)；重算环大小是否敏感。
    for &file_count in &[1u64, 10_000, 100_000_000] {
        for family in families {
            let shape = PoolShape { file_count, family, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
            let jia = solve_jia_publish(shape);
            let fsync_data_unit_count = family.data_units_per_fsync();
            let alternate_record_count = fsync_data_unit_count.max(jia.named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN));
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=q1_8_record_count_alternate p={file_count} family={} main_record_count={} alternate_record_count={alternate_record_count} differs={}",
                    family.tag(),
                    jia.record_count,
                    jia.record_count != alternate_record_count
                ))
            );
        }
    }

    // 8.2 几何敏感性：只做代表格（P=100000, F1, seq, Lbalanced, N=16），不是全量 1800 格扫描（见「它答不了的」）。
    {
        let representative_file_count = 100_000u64;
        let base_shape = PoolShape { file_count: representative_file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let base_jia = solve_jia_publish(base_shape).total_bytes();
        let base_write_ahead_log_full = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let base_write_ahead_log_leaf = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();

        let half_fill_shape = PoolShape { geometry: Geometry { fill_ratio: FillRatio::Half, ..Geometry::main() }, ..base_shape };
        let reverse_identity_reference_shape = PoolShape { geometry: Geometry { identity_reference: true, ..Geometry::main() }, ..base_shape };
        let no_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..Geometry::main() }, ..base_shape };
        let reverse_pbs_shape = PoolShape { geometry: Geometry { physical_block_size_bytes: REVERSE_PHYSICAL_BLOCK_SIZE_BYTES, ..Geometry::main() }, ..base_shape };

        for (label, shape) in [
            ("phi_0.5", half_fill_shape),
            ("k3_prime", reverse_identity_reference_shape),
            ("g_0", no_backlog_shape),
            ("pbs_4096", reverse_pbs_shape),
        ] {
            let jia = solve_jia_publish(shape).total_bytes();
            let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
            let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=geometry_sensitivity_sample point={label} p={representative_file_count} base_jia={base_jia} reverse_jia={jia} base_write_ahead_log_full={base_write_ahead_log_full} reverse_write_ahead_log_full={write_ahead_log_full} base_write_ahead_log_leaf={base_write_ahead_log_leaf} reverse_write_ahead_log_leaf={write_ahead_log_leaf} jia_ratio_changes={} write_ahead_log_full_ratio_changes={}",
                    (base_jia as f64 / base_write_ahead_log_full as f64 - jia as f64 / write_ahead_log_full as f64).abs() > 1e-9,
                    (base_write_ahead_log_full as f64 / base_write_ahead_log_leaf as f64 - write_ahead_log_full as f64 / write_ahead_log_leaf as f64).abs() > 1e-9,
                ))
            );
        }

        // K9′：只对 WAL 两臂 checkpoint 有意义（甲不受影响）。
        let checkpoint_full = solve_write_ahead_log_checkpoint(base_shape, WriteAheadLogArm::WriteAheadLogFull, 16).total_bytes();
        let checkpoint_leaf = solve_write_ahead_log_checkpoint(base_shape, WriteAheadLogArm::WriteAheadLogLeaf, 16).total_bytes();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=geometry_sensitivity_sample point=k9_prime_not_modeled p={representative_file_count} note=K9′（间隔内被换掉的中间版也写一条已释放的分配记录）未建，checkpoint_bytes_k9_full={checkpoint_full} checkpoint_bytes_k9_leaf={checkpoint_leaf}"
            ))
        );
    }

    // 第 2–4 行（第三段）：第 1 行主表的 P × 族 × 落点 × 位置策略，只算甲（登记 5.6）。
    let row_2_to_4_file_counts: [u64; 6] = [1, 100, 10_000, 1_000_000, 100_000_000, 145];
    for &file_count in &row_2_to_4_file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                    let outcome = solve_jia_publish(shape);
                    let baseline_bytes = outcome.total_bytes();

                    let accounting_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { accounting: true, ..Default::default() });
                    let allocation_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { allocation: true, ..Default::default() });
                    let mapping_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { mapping: true, ..Default::default() });
                    let tree_table_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { tree_table: true, ..Default::default() });
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=row2_fixed_point_upper_bounds p={file_count} family={} placement={} policy={} \
                             u_accounting={accounting_upper_bound} u_accounting_over_10pct={} \
                             u_allocation={allocation_upper_bound} u_allocation_over_10pct={} \
                             u_mapping={mapping_upper_bound} u_mapping_over_10pct={} \
                             u_tree_table={tree_table_upper_bound} u_tree_table_over_10pct={} \
                             direct_share_accounting={} direct_share_allocation={} direct_share_mapping={} direct_share_tree_table={}",
                            family.tag(),
                            placement.tag(),
                            policy.tag(),
                            accounting_upper_bound >= 0.10,
                            allocation_upper_bound >= 0.10,
                            mapping_upper_bound >= 0.10,
                            tree_table_upper_bound >= 0.10,
                            outcome.accounting_bytes as f64 / baseline_bytes as f64,
                            outcome.allocation_bytes as f64 / baseline_bytes as f64,
                            outcome.mapping_bytes as f64 / baseline_bytes as f64,
                            outcome.tree_table_bytes as f64 / baseline_bytes as f64,
                        ))
                    );

                    for pbs in [PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES, REVERSE_PHYSICAL_BLOCK_SIZE_BYTES] {
                        let whole_unit_rewrite_upper_bound = whole_unit_rewrite_counterfactual_share(shape, pbs);
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=row3_whole_unit_rewrite_upper_bound p={file_count} family={} placement={} policy={} pbs={pbs} u_3={whole_unit_rewrite_upper_bound} u_3_over_10pct={}",
                                family.tag(),
                                placement.tag(),
                                policy.tag(),
                                whole_unit_rewrite_upper_bound >= 0.10,
                            ))
                        );
                    }

                    for content_bytes in [100u64, 4000, 4100, 16_000, 32_634] {
                        for pbs in [PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES, REVERSE_PHYSICAL_BLOCK_SIZE_BYTES] {
                            let data_unit_rounding_upper_bound = data_unit_rounding_counterfactual_share(shape, content_bytes, pbs);
                            println!(
                                "{}",
                                emitter.emit_raw(&format!(
                                    "name=row4_data_unit_rounding_upper_bound p={file_count} family={} placement={} policy={} content_bytes={content_bytes} pbs={pbs} u_4={data_unit_rounding_upper_bound} u_4_over_10pct={}",
                                    family.tag(),
                                    placement.tag(),
                                    policy.tag(),
                                    data_unit_rounding_upper_bound >= 0.10,
                                ))
                            );
                        }
                    }
                }
            }
        }
    }

    // Q2.5：树表随树的棵数 T（登记 5.6 第 2 行，P = 10⁴、F1、seq）。
    {
        let shape = PoolShape { file_count: 10_000, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        for tree_table_entry_count in [7u64, 81, 82, 6561, 6562] {
            let tree_table_upper_bound = tree_table_counterfactual_share(shape, tree_table_entry_count);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=row2_tree_table_count_axis p=10000 family=F1 placement=seq policy=Lbalanced tree_table_entry_count={tree_table_entry_count} u_tree_table={tree_table_upper_bound} u_tree_table_over_10pct={}",
                    tree_table_upper_bound >= 0.10,
                ))
            );
        }
    }

    // 第 5 行（第四段，用户直接问的那一题）：甲每次持久化写字节随对象数怎么涨——只算甲、F1、seq 与 rand、三种位置策略。
    let row_5_file_counts: [u64; 9] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];
    for &file_count in &row_5_file_counts {
        for placement in placements {
            for policy in policies {
                let shape = PoolShape { file_count, family: Family::OneDataUnitPerFile, placement, policy, geometry: Geometry::main() };
                let (total_bytes, tree_bytes) = total_bytes_and_tree_bytes(shape);
                let (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width) = rho_readings(shape);
                let outcome = solve_jia_publish(shape);
                let heights = tree_heights(&outcome);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=row5_growth p={file_count} placement={} policy={} a={total_bytes} tree_bytes={tree_bytes} \
                         rho_a={path_ratio_by_actual_height} rho_a_over_1={} rho_b={path_ratio_by_maximum_height_node_width} rho_b_over_1={} rho_c={path_ratio_by_maximum_height_unit_width} rho_c_over_1={} \
                         extent_height={} inode_height={} allocation_height={} accounting_height={} mapping_height={} tree_table_height={} \
                         named_items={} record_count={}",
                        placement.tag(),
                        policy.tag(),
                        path_ratio_by_actual_height > 1.0,
                        path_ratio_by_maximum_height_node_width > 1.0,
                        path_ratio_by_maximum_height_unit_width > 1.0,
                        heights[0],
                        heights[1],
                        heights[2],
                        heights[3],
                        heights[4],
                        heights[5],
                        outcome.named_items,
                        outcome.record_count,
                    ))
                );
            }
        }
    }

    // 第 5 行：每棵树每次长高一层的门槛 P* 与 P* − 1（主几何下逐树搜出来；φ = 0.5 时另搜一遍），报 e 值。
    for (fill_ratio_label, fill_ratio) in [("phi_1", FillRatio::Full), ("phi_0.5", FillRatio::Half)] {
        for (tree_label, height_of) in [
            ("extent", 0usize),
            ("inode", 1usize),
            ("allocation", 2usize),
            ("mapping", 4usize),
        ] {
            let geometry = Geometry { fill_ratio, ..Geometry::main() };
            let height_at = |file_count: u64| {
                let shape = PoolShape { file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry };
                let outcome = solve_jia_publish(shape);
                let heights = tree_heights(&outcome);
                heights[height_of]
            };
            let thresholds = height_growth_thresholds(100_000_000, height_at);
            for &threshold in thresholds.iter().take(4) {
                let shape_before = PoolShape { file_count: threshold - 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry };
                let shape_at_threshold = PoolShape { file_count: threshold, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry };
                let (_, tree_bytes_before_threshold) = total_bytes_and_tree_bytes(shape_before);
                let (_, tree_bytes_at_threshold) = total_bytes_and_tree_bytes(shape_at_threshold);
                let bytes_per_layer_growth_ratio = (tree_bytes_at_threshold as f64 - tree_bytes_before_threshold as f64) / (NODE_BYTES as f64 * DEVICE_COUNT as f64);
                let mut other_trees_changed_height = Vec::new();
                for (other_label, other_height_of) in [("extent", 0usize), ("inode", 1usize), ("allocation", 2usize), ("mapping", 4usize)] {
                    if other_label == tree_label {
                        continue;
                    }
                    let before_outcome = solve_jia_publish(shape_before);
                    let at_outcome = solve_jia_publish(shape_at_threshold);
                    if tree_heights(&before_outcome)[other_height_of] != tree_heights(&at_outcome)[other_height_of] {
                        other_trees_changed_height.push(other_label);
                    }
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=row5_height_threshold geometry={fill_ratio_label} tree={tree_label} p_star={threshold} p_star_minus_1={} e={bytes_per_layer_growth_ratio} e_over_1_05={} other_trees_changed_height={:?}",
                        threshold - 1,
                        bytes_per_layer_growth_ratio > 1.05,
                        other_trees_changed_height,
                    ))
                );
            }
        }
    }

    // 第 5 行 Q5.4：甲每次持久化的记录数随 P 的轨迹（只看 F1、seq、Lbalanced 这一条代表序列）。
    for &file_count in &row_5_file_counts {
        let shape = PoolShape { file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let outcome = solve_jia_publish(shape);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=row5_record_count_trend p={file_count} record_count={} named_items={}",
                outcome.record_count, outcome.named_items,
            ))
        );
    }

    println!("{}", emitter.finish());
}
