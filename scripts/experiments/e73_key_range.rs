//! E73：节点 key 区间的扇出代价 —— D18 已定项 2 欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D18 已定项 2（2026-09-01 用户定案）：「**记账树的节点带 key 区间**，dirent 树推迟」。
//!   它买到的逐字是「**一个节点单独捡起来能自证它该覆盖哪一段 key**」。
//!   并逐字自陈：「⚠️ **欠一次测量，已立 E73**：节点头多两个 key 字段会挤掉载荷，
//!   扇出下降多少、写放大动没动，一次都没量过。
//!   D8 已定项 2 的 16 KiB 是在**没有**这两个字段的前提下选的。」
//! - D5 已定项 1：记账树的 key 是 `(统计量, 树 ID, 设备, 代)`。
//! - D5 已定项 4（2026-09-01）：统计量取九个。
//! - D8 已定项 2：节点 **16 KiB**。
//! - D19 已定项 1：位置条目带设备身份（dev id）；已定项 3：定宽。
//! - `.claude/rules/fs-design.md`「不为省空间牺牲自包含」：
//!   「遇到『这个字段能不能省掉』这类权衡，默认答案是**不省**——除非有实测证明省下来的那部分是瓶颈。」
//!
//! ## 判据（E73 正文跑前写死，跑完不许改）
//!
//! 1. **绝对值断言**：扇出必须恰好等于 `(节点大小 − 节点头) / 条目宽度`，两条臂各自算得出，
//!    不许只比两条臂的比值。
//! 2. 若树高因此涨一层，判「16 KiB 这个取值要跟着重开」——那是 D8 已定项 2 自己写的重开条件的同类。
//! 3. key 区间买到的那件事（一个节点单独捡起来能自证覆盖哪一段）要有一条**能失败的检查**：
//!    拿一个孤立节点判它该覆盖哪一段，不带区间的那条臂必须判不出来。
//!
//! ## 失败条款
//!
//! - **阳性对照**：把 key 区间字段撑到节点大小的一半，扇出必须塌掉；没塌说明字段根本没进节点头。
//! - 只测记账树不测别的 keyspace 是**合法的**——dirent 树那一格按 D18 已定项 2 推迟；
//!   但结论一律带「只对记账树成立」的口径，**不许外推**。
//!
//! ## 它答不了的
//!
//! 纯算术 + 一个孤立节点的判定模型：没有 btree 实现、没有分裂合并、没有 write buffer，
//! 文件操作 0 处。
//! ⚠️ **节点头的基础字节数仓里一处都没定过** ⇒ 它是本实验的**参数**，按三档各算一次。
//! ⚠️ 「写放大动没动」只按**结构**算（改一条 key 要重写几个节点 = 树高），
//! **不量字节吞吐**——那要真设备。

use e7_index_bench::Emitter;

/// D8 已定项 2：节点 16 KiB。**格式常量**，与 kb 的 format-const 标记绑定。
const NODE_BYTES: u64 = 16384;
/// D5 已定项 4：统计量取九个。
const STATISTIC_COUNT: u64 = 9;

/// 记账树的 key 宽度：`(统计量标签, 树 ID, 设备, 代)`。
/// ⚠️ 各段宽度仓里没定过，取一组显式假设，与 E71 同口径。
const KEY_TAG_BYTES: u64 = 2;
const KEY_TREE_BYTES: u64 = 8;
const KEY_DEVICE_BYTES: u64 = 4;
const KEY_GENERATION_BYTES: u64 = 8;
const KEY_BYTES: u64 = KEY_TAG_BYTES + KEY_TREE_BYTES + KEY_DEVICE_BYTES + KEY_GENERATION_BYTES; // 22
/// 内节点一条条目 = key + 子指针。子指针宽度取 D19 的位置条目口径（E70 用的 32 字节）。
const CHILD_POINTER_BYTES: u64 = 32;

/// 两条臂。**没有 `_ =>` 通配臂**。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// D18 已定项 2 已定：节点头带 `[min_key, max_key]` 两个 key 字段。
    WithRange,
    /// 对照：不带，区间只存在于父节点的下一条 keypointer 里（btrfs 形态）。
    WithoutRange,
}

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::WithRange => "with_range",
            Arm::WithoutRange => "without_range",
        }
    }
    /// 这条臂的节点头字节数 = 基础头 + 区间字段。
    fn header_bytes(self, base_header_bytes: u64, key_bytes: u64) -> u64 {
        match self {
            Arm::WithRange => base_header_bytes + 2 * key_bytes, // min_key + max_key
            Arm::WithoutRange => base_header_bytes,
        }
    }
}

/// **绝对值算术**：扇出 = `(节点大小 − 节点头) / 条目宽度`。
/// 头装不下时返回 0——**不是「扇出 0」，是这组参数不合法**。
fn fanout(node_bytes: u64, header_bytes: u64, entry_bytes: u64) -> u64 {
    if node_bytes <= header_bytes {
        return 0;
    }
    (node_bytes - header_bytes) / entry_bytes
}

/// 装 `entry_count` 条记录、扇出 `fanout_per_node` 的 btree 有多高（含叶层）。扇出 < 2 时返回 None——
/// 扇出 1 的树不收敛，报成一个高度会静默给出错误结论。
///
/// ⚠️ **循环必须有界，光靠上面那道 guard 不够**：`fanout_per_node == 1` 时 `capacity_at_height` 永远不增长，
/// 循环转不出去——而**挂住不是判红**。实测踩过：变异测试把 guard 改成 `fanout_per_node < 1`
/// 之后整个变异轮挂死，既没有红也没有绿，只能靠 `kill` 收场。
/// 扇出 ≥ 2 时 u64 装得下的树最高 64 层，所以 64 是个撞不到的上界；
/// 撞到了就说明扇出不合法，返回 None。
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

/// **判据 3**：拿一个孤立节点（没有父）问它该覆盖哪一段 key。
/// 带区间的臂从自己的头里读得出来；不带的臂只能返回 None——那正是 key 区间买到的东西。
fn range_of_orphan_node(arm: Arm, stored: Option<(u64, u64)>) -> Option<(u64, u64)> {
    match arm {
        Arm::WithRange => stored,
        // 区间只在父节点的 keypointer 里，孤立节点拿不到 ⇒ 判不出来。
        Arm::WithoutRange => None,
    }
}

fn main() {
    let mut emitter = Emitter::new();
    let entry_bytes = KEY_BYTES + CHILD_POINTER_BYTES;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config node={NODE_BYTES} key_bytes={KEY_BYTES} child_ptr={CHILD_POINTER_BYTES} \
             entry={entry_bytes} stats={STATISTIC_COUNT} model=arithmetic file_ops=0"
        ))
    );

    // ── 主扫：三档基础节点头 × 两条臂 ──────────────────────────────────
    // 基础头仓里没定过，扫三档：紧凑 / 中等 / 宽松。
    // 32 / 64 / 128 是跑那天的参数扫描；96 / 105 / 114 是**今天成立的那三档**——
    // E73 自己的下界 58 / 67 / 76 各加写序 10（D18 已定项 7 补注）与预留位 28
    // （D18 已定项 7「v1 就预留 nonce 代号与 MAC 的字段位」，按已定项 14 臂甲 12 + D9 已定项 2 的 16）。
    // 2026-09-07 补进扫描，两组都跑：旧那组是结论此前站的地方。
    for base_header_bytes in [32u64, 64, 128, 96, 105, 114] {
        for arm in [Arm::WithRange, Arm::WithoutRange] {
            let arm_header_bytes = arm.header_bytes(base_header_bytes, KEY_BYTES);
            let fanout_per_node = fanout(NODE_BYTES, arm_header_bytes, entry_bytes);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=fanout base_header={base_header_bytes} arm={} header={arm_header_bytes} fanout={fanout_per_node}",
                    arm.tag()
                ))
            );
        }
    }

    // ── 判据 2：树高涨没涨一层 ────────────────────────────────────────
    // 条目数取 E71 量过的那几档记账树规模。
    for entry_count in [4_075u64, 59_200, 115_200, 1_000_000] {
        for base_header_bytes in [32u64, 64, 128] {
            let fanout_with_range = fanout(NODE_BYTES, Arm::WithRange.header_bytes(base_header_bytes, KEY_BYTES), entry_bytes);
            let fanout_without_range = fanout(NODE_BYTES, Arm::WithoutRange.header_bytes(base_header_bytes, KEY_BYTES), entry_bytes);
            let height_with_range = tree_height(entry_count, fanout_with_range);
            let height_without_range = tree_height(entry_count, fanout_without_range);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=height entries={entry_count} base_header={base_header_bytes} fanout_with={fanout_with_range} \
                     fanout_without={fanout_without_range} height_with={} height_without={} grew={}",
                    height_with_range.map_or("NA".into(), |value| value.to_string()),
                    height_without_range.map_or("NA".into(), |value| value.to_string()),
                    match (height_with_range, height_without_range) {
                        (Some(with_range_height), Some(without_range_height)) => u8::from(with_range_height > without_range_height).to_string(),
                        _ => "NA".into(),
                    }
                ))
            );
        }
    }

    // ── 判据 3：孤立节点自证覆盖区间 ──────────────────────────────────
    for arm in [Arm::WithRange, Arm::WithoutRange] {
        let orphan_range = range_of_orphan_node(arm, Some((100, 200)));
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=orphan_self_describes arm={} answered={} range={}",
                arm.tag(),
                u8::from(orphan_range.is_some()),
                orphan_range.map_or("NA".into(), |(range_start, range_end)| format!("{range_start}..{range_end}"))
            ))
        );
    }

    // ── 阳性对照：key 区间字段撑到节点大小的一半，扇出必须塌掉 ────────────
    let huge_key = NODE_BYTES / 4; // 两个 key 字段合计就是节点的一半
    let arm_header_bytes = Arm::WithRange.header_bytes(64, huge_key);
    let fanout_per_node = fanout(NODE_BYTES, arm_header_bytes, entry_bytes);
    let normal_fanout = fanout(NODE_BYTES, Arm::WithRange.header_bytes(64, KEY_BYTES), entry_bytes);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control_huge_key key_bytes={huge_key} header={arm_header_bytes} \
             fanout={fanout_per_node} fanout_normal={normal_fanout} collapsed={}",
            u8::from(fanout_per_node * 2 <= normal_fanout)
        ))
    );

    // ── 写放大：改一条 key 要重写几个节点 = 树高（COW） ──────────────────
    for entry_count in [4_075u64, 115_200, 1_000_000] {
        let fanout_with_range = fanout(NODE_BYTES, Arm::WithRange.header_bytes(64, KEY_BYTES), entry_bytes);
        let fanout_without_range = fanout(NODE_BYTES, Arm::WithoutRange.header_bytes(64, KEY_BYTES), entry_bytes);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=write_amp entries={entry_count} nodes_rewritten_with={} nodes_rewritten_without={}",
                tree_height(entry_count, fanout_with_range).map_or("NA".into(), |value| value.to_string()),
                tree_height(entry_count, fanout_without_range).map_or("NA".into(), |value| value.to_string())
            ))
        );
    }

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 的绝对值断言**：两条臂的扇出各自由 `(节点大小 − 节点头) / 条目宽度` 算出。
    /// 基础头 64、key 22、条目 54：
    ///   带区间 → 头 64 + 44 = 108，(16384 − 108) / 54 = 301
    ///   不带   → 头 64，        (16384 − 64)  / 54 = 302
    #[test]
    fn criterion1_absolute_fanout_both_arms() {
        let entry_bytes = KEY_BYTES + CHILD_POINTER_BYTES;
        assert_eq!(KEY_BYTES, 22);
        assert_eq!(entry_bytes, 54);
        assert_eq!(Arm::WithRange.header_bytes(64, KEY_BYTES), 108);
        assert_eq!(Arm::WithoutRange.header_bytes(64, KEY_BYTES), 64);
        assert_eq!(fanout(NODE_BYTES, 108, entry_bytes), 301);
        assert_eq!(fanout(NODE_BYTES, 64, entry_bytes), 302);
        assert_eq!((16384 - 108) / 54, 301, "手算");
        assert_eq!((16384 - 64) / 54, 302, "手算");
    }

    /// **绝对值断言**：扇出只掉 1，相对代价 0.33%。
    #[test]
    fn absolute_fanout_cost_is_one_slot() {
        let entry_bytes = KEY_BYTES + CHILD_POINTER_BYTES;
        for base_header_bytes in [32u64, 64, 128] {
            let fanout_with_range = fanout(NODE_BYTES, Arm::WithRange.header_bytes(base_header_bytes, KEY_BYTES), entry_bytes);
            let fanout_without_range = fanout(NODE_BYTES, Arm::WithoutRange.header_bytes(base_header_bytes, KEY_BYTES), entry_bytes);
            assert!(fanout_without_range - fanout_with_range <= 1, "base={base_header_bytes} 掉了 {} 个", fanout_without_range - fanout_with_range);
        }
    }

    /// **判据 2**：树高一层都没涨——在 E71 量过的全部记账树规模上。
    #[test]
    fn criterion2_tree_height_does_not_grow() {
        let entry_bytes = KEY_BYTES + CHILD_POINTER_BYTES;
        for base_header_bytes in [32u64, 64, 128] {
            let fanout_with_range = fanout(NODE_BYTES, Arm::WithRange.header_bytes(base_header_bytes, KEY_BYTES), entry_bytes);
            let fanout_without_range = fanout(NODE_BYTES, Arm::WithoutRange.header_bytes(base_header_bytes, KEY_BYTES), entry_bytes);
            for entry_count in [4_075u64, 59_200, 115_200, 1_000_000] {
                assert_eq!(
                    tree_height(entry_count, fanout_with_range),
                    tree_height(entry_count, fanout_without_range),
                    "base={base_header_bytes} n={entry_count}：树高涨了 ⇒ D8 已定项 2 的 16 KiB 要跟着重开"
                );
            }
        }
        // 绝对值：301 扇出下 100 万条是 3 层（301² = 90601 < 10⁶ ≤ 301³）。
        assert_eq!(tree_height(1_000_000, 301), Some(3));
        assert_eq!(tree_height(4_075, 301), Some(2));
    }

    /// **判据 3**：孤立节点自证覆盖区间——带区间答得出，不带的必须答不出。
    /// 这一条就是 D18 已定项 2 逐字说它买到的那件事。
    #[test]
    fn criterion3_orphan_node_self_describes_only_with_range() {
        assert_eq!(
            range_of_orphan_node(Arm::WithRange, Some((100, 200))),
            Some((100, 200))
        );
        assert_eq!(range_of_orphan_node(Arm::WithoutRange, Some((100, 200))), None);
    }

    /// **阳性对照**：key 区间撑到节点的一半，扇出必须塌掉。
    /// 没塌说明字段根本没进节点头，整轮作废。
    #[test]
    fn positive_control_huge_key_collapses_fanout() {
        let entry_bytes = KEY_BYTES + CHILD_POINTER_BYTES;
        let huge_key_bytes = NODE_BYTES / 4;
        let collapsed_fanout = fanout(NODE_BYTES, Arm::WithRange.header_bytes(64, huge_key_bytes), entry_bytes);
        let normal_fanout = fanout(NODE_BYTES, Arm::WithRange.header_bytes(64, KEY_BYTES), entry_bytes);
        assert_eq!(normal_fanout, 301);
        assert_eq!(collapsed_fanout, 150, "(16384 − 64 − 8192) / 54 = 150");
        assert!(collapsed_fanout * 2 <= normal_fanout, "扇出必须塌掉一半：150 × 2 = 300 ≤ 301");
    }

    /// 扇出 < 2 的树不收敛，必须报 None 而不是一个高度；
    /// 而扇出恰好 2 是**合法**的，必须算得出高度——两边都钉住，
    /// 否则把 guard 收紧成 `fanout_per_node < 3` 这种错误没有任何测试看得见。
    #[test]
    fn fanout_below_two_is_not_a_tree() {
        assert_eq!(tree_height(1000, 1), None);
        assert_eq!(tree_height(1000, 0), None);
        assert_eq!(tree_height(4, 2), Some(2), "扇出 2 是合法的：2 → 4");
        assert_eq!(tree_height(1024, 2), Some(10), "2¹⁰ = 1024");
        // 头比节点还大 ⇒ 扇出 0 ⇒ 不是一棵树
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES + 1, 54), 0);
        assert_eq!(tree_height(1000, fanout(NODE_BYTES, NODE_BYTES + 1, 54)), None);
    }

    /// 写放大按结构算：COW 下改一条 key 重写的节点数就是树高，两条臂相同。
    #[test]
    fn write_amplification_is_unchanged() {
        let entry_bytes = KEY_BYTES + CHILD_POINTER_BYTES;
        let fanout_with_range = fanout(NODE_BYTES, Arm::WithRange.header_bytes(64, KEY_BYTES), entry_bytes);
        let fanout_without_range = fanout(NODE_BYTES, Arm::WithoutRange.header_bytes(64, KEY_BYTES), entry_bytes);
        for entry_count in [4_075u64, 115_200, 1_000_000] {
            assert_eq!(tree_height(entry_count, fanout_with_range), tree_height(entry_count, fanout_without_range), "n={entry_count}");
        }
    }

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2 的 format-const 标记");
        assert_eq!(STATISTIC_COUNT, 9, "D5 已定项 4");
    }
}
