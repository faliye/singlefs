//! E109：位置权威三臂的运行时代价——搬迁、发布、读三条路径各付多少。
//!
//! **它答的是**同一个岔路在四处的投影：D21（权威态与派生态的分界） 那条「共享之下多引用者住哪」、D26 未定项 5、
//! C34（明文映射层的态别三处打架），以及 D9 已定项 9 取物理键之后那一层的形状。
//! 2026-09-06 两轮三方论证独立判出这四处是同一个决定，而**「哪一条更快」全仓没有数**。
//!
//! ## 三条臂
//!
//! | 臂 | 位置的权威住哪 | 搬一个单元要动什么 | 读一个单元 |
//! |---|---|---|---|
//! | `ptr`    | 指针里的位置条目（D19 已定项 4 的字面形态，今天已定的那条） | 每棵引用树 COW 一条路径 + 每个根改写 | 0 额外跳 |
//! | `map`    | 一层「逻辑身份 → 物理」的中央映射 | 映射树 COW 一条路径 + 1 个根 | 每次多一次映射点查 |
//! | `hybrid` | 位置条目当**提示**，中央映射是权威 | 同 `map` | 提示命中 0 跳，过期才多一跳 |
//!
//! ## 口径（跑前写死，改它就是新实验）
//!
//! 1. **几何全部指得到已定条款**：数据单元 32768（D4 已定项 1 / 5）、索引节点 16384（D8 已定项 2）、
//!    扇出 119（D11 已定项 2，ε = 0.65）、位置条目 14 字节（D19 已定项 4）。
//! 2. **映射条目的 key 取已定的五元组宽度 33 字节**（D18 已定项 3 定数据单元自描述头的五元组 33，
//!    D18 已定项 7 的字段表 `105 = 42 + 33 + 8 + 8 + 10 + 4` 与它一致）。
//!    ⚠️ 2026-09-06 第一版把它写成「仓里没定死」，**那是错的**——已定项 3 定死了。
//!    另外三档（16 / 24 / 40）保留为**灵敏度探针**，不是「因为没定」。
//! 6. ⚠️ **第一版还有一个更硬的毛病，已修**：取样点只有一个，而那一点上
//!    引用树与映射树**恰好同高 4** ⇒ 把映射树的几何整个删掉、换成引用树的高，
//!    11 个单测全绿、产物逐字节不变（2026-09-06 反推攻击腿实跑证明）。
//!    ⇒ 现在按两棵树**不同高**的取样点扫，并加一条必须调用 `map` 臂的判别力断言。
//! 3. **一次发布** = D25 主负载：`N` 个数据叶 + 一条共享脊柱。与 E107 同口径，便于对照。
//! 4. **`K` 是引用同一个单元的树数**（多可写头 / 快照，D6 已定项 1「每头一棵自己的树」）。
//! 5. 读代价只数**节点读次数**，不折算成时间——本实验不测挂钟。
//!
//! ## 它不答什么（跑前写死）
//!
//! 不答挂钟与缓存命中（映射层装不装得进内存是另一个实验）；不答加密开启后的代价；
//! 不答一致性（E96 已量混合形态的静默错读，本实验不重复）；
//! 不答空间效率（`.claude/rules/fs-design.md`「比谁省不构成任何判据」）。

use e7_index_bench::{Emitter, Sample};

/// 数据单元 32768 含头（D4 已定项 1 / 已定项 5）。
const DATA_UNIT_BYTES: u64 = 32768;
/// 索引节点 16384（D8 已定项 2）。
const NODE_BYTES: u64 = 16384;
/// 扇出 119（D11 已定项 2，ε = 0.65 ⇒ 扇出 119、缓冲 665 条）。
const FANOUT: u64 = 119;
/// 缓冲 665 条（同上，只用来做交叉校验断言）。
const BUFFER_MESSAGES: u64 = 665;
/// 位置条目 14 字节 = dev 4 + 16 KiB 槽号 6 + 校验和 4（D19 已定项 4）。
const LOCATION_ENTRY_BYTES: u64 = 14;
/// journal 记录 4 KiB（D23 已定项 12）。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 根槽 512。
const ROOT_SLOT_BYTES: u64 = 512;
/// D25 主负载：一次 fsync 带 8 叶、落在 1 条共享脊柱上。
const MAIN_LEAVES: u64 = 8;
/// 那条共享脊柱上被 COW 的索引节点数（与 E107 同口径）。
const SPINE_NODES: u64 = 4;
/// 映射树节点头，与容器索引同口径（E106 / E107）。
const MAP_NODE_HEADER_BYTES: u64 = 76;
/// 副本宽度下界，D2 已定项 6 硬下界。
const MINIMUM_REPLICA_WIDTH: u64 = 2;
/// 逻辑身份五元组 33 字节（D18 已定项 3；已定项 7 的 `105 = 42 + 33 + 8 + 8 + 10 + 4` 与它一致）。
const KEY_BYTES: u64 = 33;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Pointer,
    Map,
    Hybrid,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Pointer => "ptr",
            Arm::Map => "map",
            Arm::Hybrid => "hybrid",
        }
    }
}

/// 装得下 `entry_count` 个条目、扇出 `node_fanout` 的树有几层（`n <= 1` 时 1 层）。
fn height(entry_count: u64, node_fanout: u64) -> u64 {
    let mut height_levels = 1u64;
    let mut reachable_entries = node_fanout;
    while reachable_entries < entry_count {
        reachable_entries = reachable_entries.saturating_mul(node_fanout);
        height_levels += 1;
    }
    height_levels
}

/// 映射树一个节点装多少条目：key 宽 `referencing_tree_count`，value 是 `replica_width` 份位置条目。
fn map_fanout(key_bytes: u64, replica_width: u64) -> u64 {
    (NODE_BYTES - MAP_NODE_HEADER_BYTES) / (key_bytes + LOCATION_ENTRY_BYTES * replica_width)
}

/// 一棵引用树的高：`leaf_count` 个叶、扇出 119。
fn reference_tree_height_for_leaves(leaf_count: u64) -> u64 {
    height(leaf_count, FANOUT)
}

/// 搬**一个**单元要写多少字节。`referencing_tree_count` = 引用它的树数（含活头）。
fn move_bytes(arm: Arm, referencing_tree_count: u64, leaves_per_tree: u64, total_units: u64, key_bytes: u64, replica_width: u64) -> u64 {
    match arm {
        // 每棵引用树 COW 一条从叶到根的路径；根记录本身另算一个槽。
        Arm::Pointer => referencing_tree_count * (reference_tree_height_for_leaves(leaves_per_tree) * NODE_BYTES + ROOT_SLOT_BYTES),
        // 只改一条映射条目 ⇒ 映射树 COW 一条路径 + 它自己的一个根槽。与 k 无关。
        Arm::Map | Arm::Hybrid => {
            height(total_units, map_fanout(key_bytes, replica_width)) * NODE_BYTES + ROOT_SLOT_BYTES
        }
    }
}

/// 一次发布要写多少字节（`new_leaf_count` 个新叶）。
fn publish_bytes(arm: Arm, new_leaf_count: u64, total_units: u64, key_bytes: u64, replica_width: u64) -> u64 {
    if new_leaf_count == 0 {
        return 0;
    }
    let baseline_publish_bytes = new_leaf_count * DATA_UNIT_BYTES + SPINE_NODES * NODE_BYTES + JOURNAL_RECORD_BYTES + ROOT_SLOT_BYTES;
    match arm {
        Arm::Pointer => baseline_publish_bytes,
        // 中央映射要为每个新叶插一条条目；一次发布的 n 条落在映射树的少数几条路径上，
        // 攒批之后按「一条路径 + 叶层多占的节点数」算：⌈n / 映射扇出⌉ 个叶节点 + 一条内部路径。
        Arm::Map | Arm::Hybrid => {
            let map_node_fanout = map_fanout(key_bytes, replica_width);
            let leaf_nodes = new_leaf_count.div_ceil(map_node_fanout).max(1);
            let internal_path_nodes = height(total_units, map_node_fanout).saturating_sub(1);
            baseline_publish_bytes + (leaf_nodes + internal_path_nodes) * NODE_BYTES + ROOT_SLOT_BYTES
        }
    }
}

/// 读一个单元要读几个节点。`stale` 是提示过期率（只对 hybrid 有意义），千分数。
fn read_nodes(arm: Arm, leaves_per_tree: u64, total_units: u64, key_bytes: u64, replica_width: u64, stale_permille: u64) -> u64 {
    let reference_tree_height = reference_tree_height_for_leaves(leaves_per_tree);
    let map_tree_height = height(total_units, map_fanout(key_bytes, replica_width));
    match arm {
        Arm::Pointer => reference_tree_height,
        Arm::Map => reference_tree_height + map_tree_height,
        // 千分之 stale 的读要再走一趟映射；这里报的是**千次读**的节点数，避免整数除法把它抹成 0。
        Arm::Hybrid => reference_tree_height * 1000 + map_tree_height * stale_permille,
    }
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    let arms = [Arm::Pointer, Arm::Map, Arm::Hybrid];
    // 16 TiB / 32768 的数据单元数量级，与 E106 / E107 同口径。
    let total_units: u64 = 483_183_820;
    let leaves_per_tree: u64 = 4_000_000;

    // ── 阴性对照（作废条款 1）：零操作 ⇒ 三臂全 0 ──────────────────────
    let mut negative_control_passed = true;
    for arm in arms {
        if publish_bytes(arm, 0, total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH) != 0 {
            negative_control_passed = false;
        }
    }
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=negative_control publish_leaves=0 verdict={}",
        if negative_control_passed { "pass" } else { "VOID" }
    )));
    output_text.push('\n');

    output_text.push_str(&emitter.emit_raw(&format!(
        "name=geometry fanout={FANOUT} buffer_msgs={BUFFER_MESSAGES} loc_entry={LOCATION_ENTRY_BYTES} \
         ref_height={} total_units={total_units}",
        reference_tree_height_for_leaves(leaves_per_tree)
    )));
    output_text.push('\n');

    // ── 判别力：两棵树不同高的取样点，map 臂必须跟着动而 ptr 臂不动 ───────
    // 引用树扇出 119：叶 1e5 ⇒ 高 3、4e6 ⇒ 高 4、3e8 ⇒ 高 5。
    // 映射树扇出 267（key 33 + 位置条目 14 × 2）：池 1e7 ⇒ 高 3、4.83e8 ⇒ 高 4、1e10 ⇒ 高 5。
    for &(sampled_leaves_per_tree, sampled_total_units) in &[
        (100_000u64, 10_000_000u64),
        (100_000, 483_183_820),
        (100_000, 10_000_000_000),
        (4_000_000, 10_000_000),
        (4_000_000, 483_183_820),
        (300_000_000, 10_000_000),
    ] {
        let map_move_bytes = move_bytes(Arm::Map, 1, sampled_leaves_per_tree, sampled_total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH);
        let pointer_move_bytes = move_bytes(Arm::Pointer, 1, sampled_leaves_per_tree, sampled_total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH);
        output_text.push_str(&emitter.emit_raw(&format!(
            "name=discriminating leaves={sampled_leaves_per_tree} pool={sampled_total_units} ref_height={} map_height={} \
             ptr_bytes={pointer_move_bytes} map_bytes={map_move_bytes} same_height={}",
            reference_tree_height_for_leaves(sampled_leaves_per_tree),
            height(sampled_total_units, map_fanout(KEY_BYTES, MINIMUM_REPLICA_WIDTH)),
            reference_tree_height_for_leaves(sampled_leaves_per_tree) == height(sampled_total_units, map_fanout(KEY_BYTES, MINIMUM_REPLICA_WIDTH))
        )));
        output_text.push('\n');
    }

    // ── 搬迁：扫 K 与 key 宽度 ───────────────────────────────────────────
    for &key_bytes in &[16u64, 24, KEY_BYTES, 40] {
        for &referencing_tree_count in &[1u64, 2, 4, 8, 16] {
            let pointer_move_bytes = move_bytes(Arm::Pointer, referencing_tree_count, leaves_per_tree, total_units, key_bytes, MINIMUM_REPLICA_WIDTH);
            let map_move_bytes = move_bytes(Arm::Map, referencing_tree_count, leaves_per_tree, total_units, key_bytes, MINIMUM_REPLICA_WIDTH);
            output_text.push_str(&emitter.emit_raw(&format!(
                "name=move key_bytes={key_bytes} k={referencing_tree_count} map_fanout={} map_height={} \
                 ptr_bytes={pointer_move_bytes} map_bytes={map_move_bytes} ratio_ptr_over_map={:.4}",
                map_fanout(key_bytes, MINIMUM_REPLICA_WIDTH),
                height(total_units, map_fanout(key_bytes, MINIMUM_REPLICA_WIDTH)),
                pointer_move_bytes as f64 / map_move_bytes as f64
            )));
            output_text.push('\n');
        }
    }

    // ── 发布：每次发布中央映射多付多少 ───────────────────────────────────
    for &key_bytes in &[16u64, 24, KEY_BYTES, 40] {
        for &new_leaf_count in &[1u64, MAIN_LEAVES, 64] {
            let pointer_publish_bytes = publish_bytes(Arm::Pointer, new_leaf_count, total_units, key_bytes, MINIMUM_REPLICA_WIDTH);
            let map_publish_bytes = publish_bytes(Arm::Map, new_leaf_count, total_units, key_bytes, MINIMUM_REPLICA_WIDTH);
            output_text.push_str(&emitter.emit_raw(&format!(
                "name=publish key_bytes={key_bytes} leaves={new_leaf_count} ptr_bytes={pointer_publish_bytes} map_bytes={map_publish_bytes} \
                 extra_bytes={} ratio_map_over_ptr={:.4}",
                map_publish_bytes - pointer_publish_bytes,
                map_publish_bytes as f64 / pointer_publish_bytes as f64
            )));
            output_text.push('\n');
        }
    }

    // ── 交叉点：每次发布搬多少个单元，map 臂才回本 ───────────────────────
    for &key_bytes in &[16u64, 24, KEY_BYTES, 40] {
        for &referencing_tree_count in &[1u64, 2, 4, 8, 16] {
            let map_publish_extra_bytes = publish_bytes(Arm::Map, MAIN_LEAVES, total_units, key_bytes, MINIMUM_REPLICA_WIDTH)
                - publish_bytes(Arm::Pointer, MAIN_LEAVES, total_units, key_bytes, MINIMUM_REPLICA_WIDTH);
            let bytes_saved_per_move = move_bytes(Arm::Pointer, referencing_tree_count, leaves_per_tree, total_units, key_bytes, MINIMUM_REPLICA_WIDTH)
                .saturating_sub(move_bytes(Arm::Map, referencing_tree_count, leaves_per_tree, total_units, key_bytes, MINIMUM_REPLICA_WIDTH));
            let moves_to_break_even = if bytes_saved_per_move == 0 { -1i64 } else { map_publish_extra_bytes.div_ceil(bytes_saved_per_move) as i64 };
            output_text.push_str(&emitter.emit_raw(&format!(
                "name=crossover key_bytes={key_bytes} k={referencing_tree_count} publish_extra={map_publish_extra_bytes} \
                 saved_per_move={bytes_saved_per_move} moves_per_publish_to_break_even={moves_to_break_even}"
            )));
            output_text.push('\n');
        }
    }

    // ── 读路径：hybrid 按提示过期率扫（E96 量到轮末 19–37%）───────────────
    for &stale_permille in &[0u64, 190, 370, 1000] {
        let pointer_read_nodes = read_nodes(Arm::Pointer, leaves_per_tree, total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH, stale_permille) * 1000;
        let map_read_nodes = read_nodes(Arm::Map, leaves_per_tree, total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH, stale_permille) * 1000;
        let hybrid_read_nodes = read_nodes(Arm::Hybrid, leaves_per_tree, total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH, stale_permille);
        output_text.push_str(&emitter.emit_raw(&format!(
            "name=read stale_permille={stale_permille} nodes_per_1000_reads_ptr={pointer_read_nodes} \
             nodes_per_1000_reads_map={map_read_nodes} nodes_per_1000_reads_hybrid={hybrid_read_nodes}"
        )));
        output_text.push('\n');
    }

    // ── 阳性对照：必须**调用 map 臂本身**，而不是手写一个同形的表达式 ──────
    // 第一版这里写成 `height(LEAVES_PER_TREE, FANOUT) * NODE_BYTES + ROOT_SLOT_BYTES == move_bytes(Ptr, 1, ..)`，
    // 两边是同一个表达式、一次都没碰 `Arm::Map` ⇒ `X == X`，一点判别力都没有。
    // 现在改成：换一个映射树几何，`map` 臂必须跟着变，而 `ptr` 臂必须纹丝不动。
    let pool_with_map_height_three = 10_000_000u64;
    let pool_with_map_height_five = 10_000_000_000u64;
    let map_move_bytes_at_height_three = move_bytes(Arm::Map, 1, leaves_per_tree, pool_with_map_height_three, KEY_BYTES, MINIMUM_REPLICA_WIDTH);
    let map_move_bytes_at_height_five = move_bytes(Arm::Map, 1, leaves_per_tree, pool_with_map_height_five, KEY_BYTES, MINIMUM_REPLICA_WIDTH);
    let pointer_move_bytes_at_height_three = move_bytes(Arm::Pointer, 1, leaves_per_tree, pool_with_map_height_three, KEY_BYTES, MINIMUM_REPLICA_WIDTH);
    let pointer_move_bytes_at_height_five = move_bytes(Arm::Pointer, 1, leaves_per_tree, pool_with_map_height_five, KEY_BYTES, MINIMUM_REPLICA_WIDTH);
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=positive_control map_at_pool_h3={map_move_bytes_at_height_three} map_at_pool_h5={map_move_bytes_at_height_five} \
         ptr_at_pool_h3={pointer_move_bytes_at_height_three} ptr_at_pool_h5={pointer_move_bytes_at_height_five} verdict={}",
        if map_move_bytes_at_height_three != map_move_bytes_at_height_five && pointer_move_bytes_at_height_three == pointer_move_bytes_at_height_five {
            "pass"
        } else {
            "VOID"
        }
    )));
    output_text.push('\n');

    output_text.push_str(&emitter.emit(
        "main_workload_publish_ptr",
        &Sample {
            operation_count: 1,
            bytes_per_operation: publish_bytes(Arm::Pointer, MAIN_LEAVES, total_units, KEY_BYTES, MINIMUM_REPLICA_WIDTH),
            elapsed_nanoseconds: 1,
        },
    ));
    output_text.push('\n');
    output_text.push_str(&emitter.finish());
    println!("{output_text}");
}

#[cfg(test)]
mod tests {
    use super::*;
    const TOTAL_UNITS: u64 = 483_183_820;
    const LEAVES_PER_TREE: u64 = 4_000_000;
    /// 映射树扇出 267 之下高分别为 3 / 5 的两个池规模，用来证明 `map` 臂真的在看映射几何。
    const POOL_WITH_MAP_HEIGHT_THREE: u64 = 10_000_000;
    const POOL_WITH_MAP_HEIGHT_FIVE: u64 = 10_000_000_000;

    /// 几何常量交叉校验：扇出与缓冲抄 D11 已定项 2，位置条目抄 D19 已定项 4。
    /// 写成加法不写减法（test-discipline：减法会让变异编译期溢出、被记成无效变异）。
    #[test]
    fn geometry_constants_match_settled_clauses() {
        assert_eq!(FANOUT, 119);
        assert_eq!(BUFFER_MESSAGES, 665);
        assert_eq!(LOCATION_ENTRY_BYTES, 4 + 6 + 4);
        assert_eq!(DATA_UNIT_BYTES, 32768);
        assert_eq!(NODE_BYTES, 16384);
    }

    /// 引用树的高：400 万叶、扇出 119 ⇒ 119^3 = 1 685 159 < 4e6 ≤ 119^4，恰为 4。
    #[test]
    fn reference_tree_height_is_pinned() {
        assert_eq!(FANOUT.pow(3), 1_685_159);
        assert!(FANOUT.pow(3) < LEAVES_PER_TREE);
        assert!(LEAVES_PER_TREE <= FANOUT.pow(4));
        assert_eq!(reference_tree_height_for_leaves(LEAVES_PER_TREE), 4);
    }

    /// 映射树扇出：key 33（D18 已定项 3 的五元组宽度）、w = 2 ⇒ 每条 33 + 28 = 61 字节，(16384 − 76) / 61 = 267。
    #[test]
    fn map_fanout_is_pinned() {
        assert_eq!(KEY_BYTES, 33);
        assert_eq!(KEY_BYTES + LOCATION_ENTRY_BYTES * 2, 61);
        assert_eq!(map_fanout(KEY_BYTES, 2), 267);
        assert_eq!(height(TOTAL_UNITS, 267), 4);
    }

    /// **这个实验最要紧的那条结构结论**：ptr 臂的搬迁字节随 k 线性，map 臂与 k 无关。
    /// 只写「随 k 涨」不够（三条臂一起错时互比仍成立），所以把两端的绝对值都钉住。
    #[test]
    fn move_cost_is_linear_in_referencing_tree_count_for_pointer_and_flat_for_map() {
        let single_tree_pointer_move_bytes = move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2);
        assert_eq!(single_tree_pointer_move_bytes, 4 * 16384 + 512);
        assert_eq!(single_tree_pointer_move_bytes, 66048);
        for referencing_tree_count in [1u64, 2, 4, 8, 16] {
            assert_eq!(move_bytes(Arm::Pointer, referencing_tree_count, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2), referencing_tree_count * 66048);
            // map 臂：映射树高 4 ⇒ 与 ptr 的 k = 1 恰好同值，且不随 k 动。
            assert_eq!(move_bytes(Arm::Map, referencing_tree_count, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2), 66048);
        }
    }

    /// 一次发布中央映射多付多少：8 叶、key 32 ⇒ 叶节点 1 + 内部 3 = 4 个节点，加一个根槽。
    #[test]
    fn publish_extra_for_map_is_pinned() {
        let pointer_publish_bytes = publish_bytes(Arm::Pointer, MAIN_LEAVES, TOTAL_UNITS, KEY_BYTES, 2);
        let map_publish_bytes = publish_bytes(Arm::Map, MAIN_LEAVES, TOTAL_UNITS, KEY_BYTES, 2);
        assert_eq!(pointer_publish_bytes, 332288); // 与 E107 的基线逐字节相同，两个实验同尺
        assert_eq!(map_publish_bytes, pointer_publish_bytes + 4 * 16384 + 512);
        assert_eq!(map_publish_bytes, 398336);
    }

    /// 交叉点：主负载下每次发布搬几个单元，map 臂才回本。
    /// k = 1 时 map 一分不省 ⇒ 永远回不了本，报 −1。
    #[test]
    fn breakeven_moves_per_publish_are_pinned() {
        let map_publish_extra_bytes = publish_bytes(Arm::Map, MAIN_LEAVES, TOTAL_UNITS, KEY_BYTES, 2)
            - publish_bytes(Arm::Pointer, MAIN_LEAVES, TOTAL_UNITS, KEY_BYTES, 2);
        assert_eq!(map_publish_extra_bytes, 66048);
        let saved_with_one_tree = move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2)
            - move_bytes(Arm::Map, 1, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2);
        assert_eq!(saved_with_one_tree, 0);
        let saved_with_two_trees = move_bytes(Arm::Pointer, 2, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2)
            - move_bytes(Arm::Map, 2, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2);
        assert_eq!(saved_with_two_trees, 66048);
        assert_eq!(map_publish_extra_bytes.div_ceil(saved_with_two_trees), 1);
        let saved_with_sixteen_trees = move_bytes(Arm::Pointer, 16, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2)
            - move_bytes(Arm::Map, 16, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2);
        assert_eq!(saved_with_sixteen_trees, 15 * 66048);
        assert_eq!(map_publish_extra_bytes.div_ceil(saved_with_sixteen_trees), 1);
    }

    /// **判别力自证（作废条款 3）**：提示过期率为 0 时 hybrid 必须与 ptr 逐格相等，
    /// 为 1000‰ 时必须与 map 逐格相等。这条不成立，说明 hybrid 根本没建成一条中间臂。
    #[test]
    fn hybrid_collapses_to_pointer_at_zero_stale_and_to_map_at_full_stale() {
        let pointer_read_nodes_per_thousand = read_nodes(Arm::Pointer, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 0) * 1000;
        let map_read_nodes_per_thousand = read_nodes(Arm::Map, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 0) * 1000;
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 0), pointer_read_nodes_per_thousand);
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 1000), map_read_nodes_per_thousand);
        // 中间值必须严格落在两端之间，否则那条曲线是平的、这一维没有信息
        let hybrid_read_nodes_per_thousand_at_370_permille = read_nodes(Arm::Hybrid, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 370);
        assert!(pointer_read_nodes_per_thousand < hybrid_read_nodes_per_thousand_at_370_permille && hybrid_read_nodes_per_thousand_at_370_permille < map_read_nodes_per_thousand, "p={pointer_read_nodes_per_thousand} mid={hybrid_read_nodes_per_thousand_at_370_permille} m={map_read_nodes_per_thousand}");
    }

    /// 读路径的绝对值：引用树高 4、映射树高 4 ⇒ 千次读 ptr 4000、map 8000。
    #[test]
    fn read_nodes_absolute_values_are_pinned() {
        assert_eq!(read_nodes(Arm::Pointer, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 0) * 1000, 4000);
        assert_eq!(read_nodes(Arm::Map, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 0) * 1000, 8000);
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 370), 4000 + 4 * 370);
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES_PER_TREE, TOTAL_UNITS, 32, 2, 370), 5480);
    }

    /// 作废条款 3 的另一半：key 宽度四档上结论方向必须一致。
    #[test]
    fn direction_is_stable_across_key_widths() {
        for key_width_bytes in [16u64, 24, 32, 40] {
            // 搬迁：map 恒不随 k 涨，ptr 恒随 k 涨
            let map_move_one_tree = move_bytes(Arm::Map, 1, LEAVES_PER_TREE, TOTAL_UNITS, key_width_bytes, 2);
            let map_move_sixteen_trees = move_bytes(Arm::Map, 16, LEAVES_PER_TREE, TOTAL_UNITS, key_width_bytes, 2);
            assert_eq!(map_move_one_tree, map_move_sixteen_trees, "key_bytes={key_width_bytes}");
            assert!(
                move_bytes(Arm::Pointer, 16, LEAVES_PER_TREE, TOTAL_UNITS, key_width_bytes, 2)
                    > move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, TOTAL_UNITS, key_width_bytes, 2)
            );
            // 发布：map 恒比 ptr 贵
            assert!(
                publish_bytes(Arm::Map, MAIN_LEAVES, TOTAL_UNITS, key_width_bytes, 2)
                    > publish_bytes(Arm::Pointer, MAIN_LEAVES, TOTAL_UNITS, key_width_bytes, 2),
                "key_bytes={key_width_bytes}"
            );
        }
    }

    /// 阴性对照（作废条款 1）。
    #[test]
    fn zero_publish_is_zero_on_all_arms() {
        for arm in [Arm::Pointer, Arm::Map, Arm::Hybrid] {
            assert_eq!(publish_bytes(arm, 0, TOTAL_UNITS, KEY_BYTES, 2), 0, "{}", arm.name());
        }
    }

    /// **阳性对照（2026-09-06 重写）**：换一个映射树几何，`map` 臂必须跟着变，
    /// 而 `ptr` 臂必须纹丝不动。
    ///
    /// ⚠️ **第一版这条是 `X == X`**：写成
    /// `height(LEAVES_PER_TREE, FANOUT) * NODE_BYTES + ROOT_SLOT_BYTES == move_bytes(Ptr, 1, ..)`，
    /// 两边是同一个表达式，**一次都没调用 `Arm::Map`** ⇒ 把映射树的几何整个删掉、
    /// 换成引用树的高，11 个单测全绿、产物逐字节不变（2026-09-06 反推攻击腿实跑证明）。
    #[test]
    fn positive_control_must_actually_exercise_the_map_arm() {
        let map_move_bytes_at_height_three = move_bytes(Arm::Map, 1, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_THREE, KEY_BYTES, 2);
        let map_move_bytes_at_height_five = move_bytes(Arm::Map, 1, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2);
        let pointer_move_bytes_at_height_three = move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_THREE, KEY_BYTES, 2);
        let pointer_move_bytes_at_height_five = move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2);
        assert_eq!(height(POOL_WITH_MAP_HEIGHT_THREE, map_fanout(KEY_BYTES, 2)), 3);
        assert_eq!(height(POOL_WITH_MAP_HEIGHT_FIVE, map_fanout(KEY_BYTES, 2)), 5);
        assert_eq!(map_move_bytes_at_height_three, 3 * NODE_BYTES + ROOT_SLOT_BYTES);
        assert_eq!(map_move_bytes_at_height_five, 5 * NODE_BYTES + ROOT_SLOT_BYTES);
        assert_ne!(map_move_bytes_at_height_three, map_move_bytes_at_height_five);
        assert_eq!(pointer_move_bytes_at_height_three, pointer_move_bytes_at_height_five);
        assert_eq!(pointer_move_bytes_at_height_three, 4 * NODE_BYTES + ROOT_SLOT_BYTES);
    }

    /// 读路径也必须真的在看映射树的几何——主取样点上两棵树同高 4，
    /// 只在那一点上测等于没测（2026-09-06 变异 M16 抓到的第二个同型盲区）。
    #[test]
    fn read_nodes_depend_on_map_geometry() {
        assert_eq!(read_nodes(Arm::Map, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_THREE, KEY_BYTES, 2, 0), 4 + 3);
        assert_eq!(read_nodes(Arm::Map, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2, 0), 4 + 5);
        assert_ne!(
            read_nodes(Arm::Map, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_THREE, KEY_BYTES, 2, 0),
            read_nodes(Arm::Map, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2, 0)
        );
        // ptr 臂对池规模不敏感
        assert_eq!(
            read_nodes(Arm::Pointer, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_THREE, KEY_BYTES, 2, 0),
            read_nodes(Arm::Pointer, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2, 0)
        );
        // hybrid 的多跳那一项也必须跟着映射树走
        assert_eq!(
            read_nodes(Arm::Hybrid, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2, 370),
            4 * 1000 + 5 * 370
        );
    }

    /// **主取样点是一个退化点，必须显式标出来**：那一点上 `map` 与 `ptr`（`K = 1`）
    /// 数值相同，是因为两棵树恰好都是高 4，不是因为它们本质相同。
    #[test]
    fn the_main_sampling_point_is_a_degenerate_one() {
        assert_eq!(reference_tree_height_for_leaves(LEAVES_PER_TREE), 4);
        assert_eq!(height(TOTAL_UNITS, map_fanout(KEY_BYTES, 2)), 4);
        assert_eq!(
            move_bytes(Arm::Map, 1, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2),
            move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, TOTAL_UNITS, KEY_BYTES, 2)
        );
        assert_ne!(
            move_bytes(Arm::Map, 1, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2),
            move_bytes(Arm::Pointer, 1, LEAVES_PER_TREE, POOL_WITH_MAP_HEIGHT_FIVE, KEY_BYTES, 2)
        );
    }
}
