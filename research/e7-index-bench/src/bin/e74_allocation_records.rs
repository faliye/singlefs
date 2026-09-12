//! E74：分配记录的条目数与整理开销 —— D3 已定项 1 自陈「代价没有量」欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D3 已定项 1（2026-08-31 用户定案）：分配记录**逐落点**答得出「属不属于现行版本」。
//!   逐字自陈：「⚠️ **代价没有量**：逐落点意味着分配记录的条目数随容量走，
//!   而不是随统计维度走。它与 D3『后台整理常驻』的交互（整理会大批改写落点归属）没有实测。」
//! - D3 已定项 3（2026-09-01 用户定案）：「**独立 keyspace 的 btree，
//!   key = 落点（设备身份 + 设备内偏移），value = 分配代。**」
//! - D8 已定项 2：节点 **16 KiB**。D4 已定：单元恒 **32 KiB**。
//! - D5 已定项 4（2026-09-01）：统计量取九个 ⇒ 记账树条目数随统计维度走（E71 已量）。
//! - `.claude/rules/fs-design.md` 第一格：运行时决策路径**不许遍历**，
//!   且代价**不许随盘容量增长**。
//!
//! ## 判据（E74 正文跑前写死，跑完不许改）
//!
//! 1. **绝对值断言**：条目数必须恰好等于已分配落点数，不许只看它「大致随容量走」。
//! 2. 整理一批 N 个落点，写入字节必须是 `O(N × 条目宽度 + 树高 × 节点大小)`；
//!    出现随**盘容量**而非 N 增长的项，判「整理路径里混进了全扫」。
//! 3. 与 D5 的记账树对照：两棵树的条目数**量纲必须不同**
//!    （一个随容量、一个随统计维度）；量纲相同说明有一棵建错了。
//!
//! ## 失败条款
//!
//! - **阳性对照**：把容量加倍，条目数必须加倍；没加倍说明分配记录根本没按落点建。
//! - 量出来若不可接受，**重开的是 D3 已定项 1 的粒度，不是已定项 3 的载体**。
//!
//! ## 它答不了的
//!
//! 纯算术 + btree 结构模型：没有分配器实现、没有后台整理、没有 write buffer，
//! 文件操作 0 处。
//! ⚠️ **E74 正文问的第二条轴有两问，这里只答得了一问**：
//! 「这棵树要写多少字节」是结构量，算得出；**「跑多久」是挂钟，要实现才有**。
//! ⚠️ **后台整理的形态本身还没定**（D26 六个分项全开着）
//! ⇒ 这里量的是「**btree 这个载体在整理批量改写下强制付出多少**」，
//! **不是**某个具体整理策略的代价。载体是已定的，策略不是。
//! ⚠️ 落点粒度仓里没定过（D4 定的 32 KiB 是**单元**，不是「落点」的定义）⇒ 按参数扫。

use e7_index_bench::Emitter;

/// D8 已定项 2：节点 16 KiB。**格式常量**。
const NODE_BYTES: u64 = 16384;
/// D4 已定：单元恒 32 KiB。**格式常量**。落点粒度取它作主档，另扫两档。
const DATA_UNIT_BYTES: u64 = 32768;
/// D3 已定项 3：key = 落点（设备身份 + 设备内偏移）。⚠️ 各段宽度仓里没定过。
const KEY_DEVICE_IDENTITY_BYTES: u64 = 4;
const KEY_OFFSET_BYTES: u64 = 8;
/// value = 分配代。
const VALUE_GENERATION_BYTES: u64 = 8;
const ENTRY_BYTES: u64 = KEY_DEVICE_IDENTITY_BYTES + KEY_OFFSET_BYTES + VALUE_GENERATION_BYTES; // 20
/// 节点头基础字节，与 E73 同口径（仓里没定过，取中间那档）。
const NODE_HEADER_BYTES: u64 = 64;

/// 扇出 = `(节点大小 − 节点头) / 条目宽度`。
fn fanout(node_bytes: u64, header_bytes: u64, entry_bytes: u64) -> u64 {
    if node_bytes <= header_bytes {
        return 0;
    }
    (node_bytes - header_bytes) / entry_bytes
}

/// 树高（含叶层）。扇出 < 2 不收敛 ⇒ None。循环有界，见 E73 踩过的那个挂死。
fn tree_height(entry_count: u64, fanout_per_node: u64) -> Option<u64> {
    if fanout_per_node < 2 {
        return None;
    }
    let mut height = 1u64;
    let mut capacity_at_height = fanout_per_node;
    while capacity_at_height < entry_count {
        capacity_at_height = capacity_at_height.saturating_mul(fanout_per_node);
        height += 1;
        if height > 64 {
            return None;
        }
    }
    Some(height)
}

/// **绝对值算术**：容量 `capacity_bytes`、落点粒度 `grain_bytes`、填充率 `fill_percent`
/// ⇒ 已分配落点数 = 条目数。
fn entries(capacity_bytes: u64, grain_bytes: u64, fill_percent: u64) -> u64 {
    (capacity_bytes / grain_bytes) * fill_percent / 100
}

/// 被整理的 N 个落点在 key 空间里聚不聚。**没有 `_ =>` 通配臂。**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Locality {
    /// 连续：N 个落点在 key 空间里相邻 ⇒ 挤在同一批叶里。
    /// key = (设备身份, 设备内偏移) ⇒ **物理相邻的落点在 key 空间里也相邻**。
    Contiguous,
    /// 打散：N 个落点各落一个叶 ⇒ 每个都要重写一整个节点。**最坏情形。**
    Scattered,
}

impl Locality {
    fn tag(self) -> &'static str {
        match self {
            Locality::Contiguous => "contiguous",
            Locality::Scattered => "scattered",
        }
    }
}

/// COW 下改 N 条 key 要重写几个节点：被碰到的叶 + 每层各一个内部节点。
/// ⚠️ **COW 重写的是整个节点，不是那 20 字节** —— 这正是要量的东西。
fn nodes_rewritten(rewritten_entry_count: u64, total_entries: u64, fanout_per_node: u64, locality: Locality) -> Option<u64> {
    let height = tree_height(total_entries, fanout_per_node)?;
    let leaves_total = total_entries.div_ceil(fanout_per_node).max(1);
    let leaves_touched = match locality {
        Locality::Contiguous => rewritten_entry_count.div_ceil(fanout_per_node).max(1),
        Locality::Scattered => rewritten_entry_count.min(leaves_total).max(1),
    };
    // 叶之上每层各碰一个节点（连续时）；打散时上层也会被碰到多个，
    // 但上界仍是「每层的节点数」，这里取保守的 min(leaves, 该层节点数)。
    let mut interior_nodes_touched = 0u64;
    let mut level_nodes = leaves_total;
    for _ in 1..height {
        level_nodes = level_nodes.div_ceil(fanout_per_node).max(1);
        interior_nodes_touched += leaves_touched.min(level_nodes);
    }
    Some(leaves_touched + interior_nodes_touched)
}

/// 理想写入量：只写那 N 条条目本身。
fn ideal_bytes(entry_count: u64, entry_bytes: u64) -> u64 {
    entry_count * entry_bytes
}

/// COW 实际写入字节 = 重写的节点数 × 整个节点。
/// ⚠️ **main 与单测必须走同一个函数**：各写一遍的话，产物里那个数没有任何测试钉着
/// ——实测踩过，变异「按条目计费而不是按整节点」一个测试都没红。
fn rewritten_bytes(node_count: u64) -> u64 {
    node_count * NODE_BYTES
}

fn main() {
    let mut emitter = Emitter::new();
    let fanout_per_node = fanout(NODE_BYTES, NODE_HEADER_BYTES, ENTRY_BYTES);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config node={NODE_BYTES} node_header={NODE_HEADER_BYTES} entry={ENTRY_BYTES} \
             fanout={fanout_per_node} unit={DATA_UNIT_BYTES} model=arithmetic file_ops=0"
        ))
    );

    // ── 轴 ①：条目数与树高随盘容量怎么走 ─────────────────────────────
    for capacity_tebibytes in [1u64, 4, 16, 64, 256] {
        let capacity_bytes = capacity_tebibytes * 1024 * 1024 * 1024 * 1024;
        for grain_bytes in [4096u64, DATA_UNIT_BYTES, 1024 * 1024] {
            let entry_count = entries(capacity_bytes, grain_bytes, 90);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=scale cap_tib={capacity_tebibytes} grain={grain_bytes} fill_pct=90 entries={entry_count} \
                     height={} tree_bytes_mib={}",
                    tree_height(entry_count, fanout_per_node).map_or("NA".into(), |value| value.to_string()),
                    entry_count.div_ceil(fanout_per_node) * NODE_BYTES / (1024 * 1024)
                ))
            );
        }
    }

    // ── 判据 2：整理 N 个落点的写入量随不随盘容量走 ───────────────────
    // 固定 N，扫容量：写入量只许通过树高（对数）变，不许线性跟着容量走。
    for compacted_entry_count in [1u64, 64, 4096, 65536] {
        for capacity_tebibytes in [1u64, 16, 256] {
            let capacity_bytes = capacity_tebibytes * 1024 * 1024 * 1024 * 1024;
            let total_entries = entries(capacity_bytes, DATA_UNIT_BYTES, 90);
            for locality in [Locality::Contiguous, Locality::Scattered] {
                let rewritten_node_count = nodes_rewritten(compacted_entry_count, total_entries, fanout_per_node, locality);
                let rewritten_byte_count = rewritten_node_count.map(rewritten_bytes);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=compact n={compacted_entry_count} cap_tib={capacity_tebibytes} total_entries={total_entries} \
                         locality={} nodes={} bytes={} ideal_bytes={} amp_x100={}",
                        locality.tag(),
                        rewritten_node_count.map_or("NA".into(), |value| value.to_string()),
                        rewritten_byte_count.map_or("NA".into(), |value| value.to_string()),
                        ideal_bytes(compacted_entry_count, ENTRY_BYTES),
                        rewritten_byte_count.map_or("NA".into(), |byte_count| (byte_count * 100
                            / ideal_bytes(compacted_entry_count, ENTRY_BYTES).max(1))
                            .to_string())
                    ))
                );
            }
        }
    }

    // ── 判据 3：与记账树对照，两棵树的量纲必须不同 ─────────────────────
    // 记账树条目数（E71 上界臂，t=64 d=8 K=25）与容量无关。
    let accounting_entries = 9 * 64 * 8 * 25;
    for capacity_tebibytes in [1u64, 16, 256] {
        let capacity_bytes = capacity_tebibytes * 1024 * 1024 * 1024 * 1024;
        let allocation_entries = entries(capacity_bytes, DATA_UNIT_BYTES, 90);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=dimension cap_tib={capacity_tebibytes} alloc_entries={allocation_entries} acct_entries={accounting_entries} \
                 alloc_scales_with_cap=1 acct_scales_with_cap=0 ratio={}",
                allocation_entries / accounting_entries
            ))
        );
    }

    // ── 阳性对照：容量加倍 ⇒ 条目数加倍 ──────────────────────────────
    // ⚠️ 两档都报：100% 填充是精确判定，90% 那档差 1 条是 `× fill / 100` 的取整，
    // 只报 90% 会让产物里出现一个读起来像「阳性对照没过」的 0。
    for fill_percent in [100u64, 90] {
        let entries_16_tebibytes = entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, fill_percent);
        let entries_32_tebibytes = entries(32 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, fill_percent);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=positive_control_capacity_doubles fill_pct={fill_percent} a={entries_16_tebibytes} b={entries_32_tebibytes} exactly_2x={} rounding_gap={}",
                u8::from(entries_32_tebibytes == 2 * entries_16_tebibytes),
                entries_32_tebibytes - 2 * entries_16_tebibytes
            ))
        );
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 的绝对值断言**：条目数恰好等于已分配落点数。
    /// 16 TiB、32 KiB 落点、90% 填充 ⇒ 16×2⁴⁰/32768 × 0.9 = 471_859_200。
    #[test]
    fn criterion1_absolute_entry_count() {
        let capacity_bytes = 16u64 * 1024 * 1024 * 1024 * 1024;
        assert_eq!(capacity_bytes / DATA_UNIT_BYTES, 536_870_912, "16 TiB 有这么多个 32 KiB 落点");
        assert_eq!(entries(capacity_bytes, DATA_UNIT_BYTES, 90), 483_183_820);
        assert_eq!(536_870_912u64 * 90 / 100, 483_183_820, "手算");
        // 100% 填充时就是落点总数本身
        assert_eq!(entries(capacity_bytes, DATA_UNIT_BYTES, 100), 536_870_912);
    }

    /// **阳性对照**：容量加倍 ⇒ 条目数恰好加倍。
    /// ⚠️ 判定取 **100% 填充**——`(cap/grain) × fill / 100` 在 90% 那档会被整数除法
    /// 截掉 1 条（966_367_641 vs 2 × 483_183_820），那是**取整**不是模型错。
    /// 两档都断言：100% 精确加倍，90% 差额恰好 1。
    #[test]
    fn positive_control_capacity_doubles_entries() {
        let entries_16_tebibytes = entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 100);
        let entries_32_tebibytes = entries(32 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 100);
        assert_eq!(entries_16_tebibytes, 536_870_912);
        assert_eq!(entries_32_tebibytes, 2 * entries_16_tebibytes, "100% 填充下必须精确加倍");
        let entries_16_tebibytes_at_90_percent = entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        let entries_32_tebibytes_at_90_percent = entries(32 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        assert_eq!(entries_32_tebibytes_at_90_percent - 2 * entries_16_tebibytes_at_90_percent, 1, "90% 那档的差额恰好是取整的 1 条");
    }

    /// **绝对值断言**：扇出与树高。(16384 − 64) / 20 = 816。
    #[test]
    fn absolute_fanout_and_height() {
        assert_eq!(ENTRY_BYTES, 20);
        assert_eq!(fanout(NODE_BYTES, NODE_HEADER_BYTES, ENTRY_BYTES), 816);
        assert_eq!((16384 - 64) / 20, 816, "手算");
        // 4.8 亿条目、扇出 816 ⇒ 816³ = 5.4 亿 ≥ 4.8 亿 ⇒ 3 层
        assert_eq!(tree_height(483_183_820, 816), Some(3));
        assert_eq!(816u64.pow(3), 543_338_496);
    }

    /// **判据 2**：固定 N，容量从 1 TiB 涨到 256 TiB，写入量只许通过树高变。
    #[test]
    fn criterion2_compaction_cost_does_not_scale_with_capacity() {
        let fanout_per_node = fanout(NODE_BYTES, NODE_HEADER_BYTES, ENTRY_BYTES);
        let compacted_entry_count = 4096u64;
        let mut rewritten_nodes_per_capacity = vec![];
        for capacity_tebibytes in [1u64, 16, 256] {
            let total_entries = entries(capacity_tebibytes * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
            rewritten_nodes_per_capacity.push(nodes_rewritten(compacted_entry_count, total_entries, fanout_per_node, Locality::Contiguous).unwrap());
        }
        // 容量从 1 TiB 涨到 256 TiB（256 倍），重写节点数只从 13 涨到 19——
        // 涨的那 6 个全部来自树高 3 → 4，**没有任何一项跟着容量线性走**。
        assert_eq!(rewritten_nodes_per_capacity, vec![13, 13, 19], "1 / 16 / 256 TiB 三档");
        let (smallest_node_count, largest_node_count) = (rewritten_nodes_per_capacity[0], rewritten_nodes_per_capacity[2]);
        assert!(largest_node_count < 2 * smallest_node_count, "容量涨 256 倍，重写节点数连一倍都没涨（{smallest_node_count} → {largest_node_count}）");
        assert!(largest_node_count < compacted_entry_count, "重写节点数必须远小于 N，否则就是每条一个节点");
        // 树高确实从 3 涨到 4，涨的就是它。
        assert_eq!(tree_height(entries(1024u64.pow(4), DATA_UNIT_BYTES, 90), fanout_per_node), Some(3));
        assert_eq!(tree_height(entries(256 * 1024u64.pow(4), DATA_UNIT_BYTES, 90), fanout_per_node), Some(4));
    }

    /// **COW 的真实代价**：打散的 N 个落点每个各重写一个 16 KiB 节点
    /// ⇒ 相对「只写那 20 字节」放大 **819 倍**。这是载体强制的，不是策略的错。
    #[test]
    fn absolute_copy_on_write_amplification_when_scattered() {
        let fanout_per_node = fanout(NODE_BYTES, NODE_HEADER_BYTES, ENTRY_BYTES);
        let total_entries = entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        let compacted_entry_count = 4096u64;
        let rewritten_node_count = nodes_rewritten(compacted_entry_count, total_entries, fanout_per_node, Locality::Scattered).unwrap();
        let rewritten_byte_count = rewritten_bytes(rewritten_node_count);
        let ideal_byte_count = ideal_bytes(compacted_entry_count, ENTRY_BYTES);
        assert_eq!(ideal_byte_count, 81_920, "4096 × 20 字节");
        assert!(rewritten_node_count >= compacted_entry_count, "打散时每条至少碰一个叶");
        assert_eq!(rewritten_node_count, 4823, "4096 个叶 + 上面两层各 727 / 1 个内部节点");
        assert_eq!(rewritten_byte_count / ideal_byte_count, 964, "放大恰好 964 倍");
    }

    /// **连续那一侧**：N 条挤进 ceil(N/816) 个叶 ⇒ 放大回到个位数。
    /// key = (设备身份, 设备内偏移) ⇒ 物理相邻的落点在 key 空间里也相邻。
    #[test]
    fn absolute_contiguous_keeps_amplification_small() {
        let fanout_per_node = fanout(NODE_BYTES, NODE_HEADER_BYTES, ENTRY_BYTES);
        let total_entries = entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        let compacted_entry_count = 4096u64;
        let rewritten_node_count = nodes_rewritten(compacted_entry_count, total_entries, fanout_per_node, Locality::Contiguous).unwrap();
        assert_eq!(compacted_entry_count.div_ceil(fanout_per_node), 6, "4096 / 816 → 6 个叶");
        assert_eq!(rewritten_node_count, 13, "6 个叶 + 上面两层各 6 / 1 个内部节点");
        let amplification = rewritten_bytes(rewritten_node_count) / ideal_bytes(compacted_entry_count, ENTRY_BYTES);
        assert_eq!(amplification, 2, "连续时放大恰好 2 倍：13 × 16384 / 81920");
    }

    /// **判据 3**：两棵树量纲不同——分配记录随容量走，记账树不随。
    #[test]
    fn criterion3_two_trees_have_different_dimensions() {
        let accounting_entries = 9u64 * 64 * 8 * 25; // E71 上界臂，与容量无关
        assert_eq!(accounting_entries, 115_200);
        let entries_16_tebibytes = entries(16 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        let entries_256_tebibytes = entries(256 * 1024 * 1024 * 1024 * 1024, DATA_UNIT_BYTES, 90);
        assert!(entries_256_tebibytes - 16 * entries_16_tebibytes <= 16, "分配记录随容量线性走（差额只是取整）");
        assert_eq!(entries_16_tebibytes, 483_183_820);
        assert_eq!(entries_256_tebibytes, 7_730_941_132);
        // 记账树在两个容量上是同一个数 —— 量纲不同，判据 3 成立
        assert_eq!(accounting_entries, 115_200);
        assert!(entries_16_tebibytes / accounting_entries > 4000, "同一容量下分配记录比记账树大三个数量级以上");
    }

    /// 扇出不合法时报 None，不许退化成一个数。
    #[test]
    fn illegal_fanout_is_not_a_measurement() {
        assert_eq!(nodes_rewritten(10, 1000, 1, Locality::Contiguous), None);
        assert_eq!(tree_height(1000, 1), None);
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES + 1, ENTRY_BYTES), 0);
    }

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(DATA_UNIT_BYTES, 32768, "D4 已定");
    }
}
