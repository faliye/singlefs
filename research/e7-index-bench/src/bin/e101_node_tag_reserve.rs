//! E101：节点头留位的代价与表达力 —— D26 未定项 4 的格式那一半。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D26 未定项 4 判「是」的逐字依据**：「第一个事务要写索引节点，而这一项**直接改节点布局**
//!   ⇒ **不定它，第一个事务的节点字节写不出来**」。
//! - **D26 未定项 4 的 2026-09-03 复核逐字**：三腿判「翻转为否」不成立，**判定维持「是」**。
//! - **E95 自陈的射程逐字**：「两条格式臂是**候选不是穷举**：别的节点布局形态若被提出，
//!   **要加臂重跑**」；且「组身份与老化代在真格式里**各占几字节**……**模型不算**」。
//! - **E95 的两条臂**：① 节点头带**组身份**（BetrFS 大节点效应的显式化，节点大小仍是 16 KiB）；
//!   ② 节点头带**老化代**。
//! - **D19 已定项 3 的定宽留位政策**：指针按**最坏情形**定宽——MAC（128 位）与 nonce（96 位）
//!   恒定占位。⚠️ 它给得出数，是因为最坏宽度**由密码算法给死**。
//! - **D8 已定项 2**：节点 16 KiB。**D18 已定项 7**：共同明文前缀 42 字节，
//!   索引节点类身份段「**宽度随实现定**」。
//! - **D5 已定项 5**：记账条目 30 字节。**D3 已定项 7**：分配记录条目 20 字节。
//! - **D22 已定项 2**：根环 R = 3、每区槽数 S ∈ 1..16 ⇒ 保留代数 K = 3S + 1 ≤ 49。
//!
//! ## 判据（E101 正文跑前写死，跑完不许改）
//!
//! 1. **代价要算得出绝对值**：留位 0 / 1 / 2 / 4 字节各一档，逐档算三棵已定条目宽的树的
//!    叶扇出、内部扇出、树高。扇出恰好 `(16384 − 节点头 − 区间字段) / 条目宽`。
//! 2. **表达力下界**：把 E95 注册的两条臂形式化成「每节点一个标签」，
//!    **数出各自要多少个互不相同的标签值** ⇒ 标签宽度的下界。**数不出来就报「上界取不到」，不许估。**
//! 3. **翻转判据**：留位之后，第一个事务写出的节点字节在「将来选任何一条臂」下**是否逐字节相同**——
//!    相同则依据消解、判定可翻为否；**不同则不许翻**。
//! 4. **冲突计数**：标签宽度不足时被迫合并的组数；宽度够时必须恰好 0。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照，每条臂都跑**：宽度取 0 时两条臂的冲突数都必须 > 0；取足够宽时都必须恰好 0。
//! - **阴性对照**：只有一个组时，任何宽度（含 0 位）的冲突数都必须为 0。
//! - **反向接受条款**：判据 3 判出「第一个事务的字节会随所选臂而变」
//!   ⇒ 结论是「**留位翻不了那个判定**，D26 未定项 4 照旧挡在第一行代码前面」，如实写。
//! - E95 未跑 ⇒ 两条臂的形态取它注册时写下的那一句，**不许自己扩写**。
//!
//! ## 它答不了的
//!
//! 纯算术：没有老化负载、没有整理实现、没有 E95 的实测。不答「哪一条抗老化形态更好」。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384;
const NODE_HEADERS: [u64; 3] = [58, 67, 76];
/// D19 已定项 4 之后的树表单元指针。
const CHILD_POINTER_BYTES: u64 = 59;
/// 三棵已定条目宽的树。inode 那一档取 E98 的 140。
const TREES: [(&str, u64, u64); 3] = [
    ("acct", 30, 22),  // D5 已定项 5：条目 30、key 22
    ("alloc", 20, 12), // D3 已定项 7：条目 20、key = dev 4 + 槽号 6 + 跨度 2
    ("inode", 140, 8), // E98：记录 140、key = inode 8
];
/// 留位宽度四档。
const TAG_WIDTHS: [u64; 4] = [0, 1, 2, 4];
/// D22 已定项 2：K = 3S + 1，S ≤ 16 ⇒ K ≤ 49。
const MAXIMUM_RETAINED_GENERATIONS: u64 = 49;

/// E95 注册的两条臂。`distinct_values` = 它要多少个互不相同的标签值；
/// `None` = **上界指不到任何已定条款**，不许估。
struct Arm {
    name: &'static str,
    distinct_values: Option<u64>,
    why: &'static str,
}
const ARMS: [Arm; 2] = [
    Arm {
        name: "generation_split",
        // 代际分离：要分开的代数就是保留代数 K，由 D22 已定项 2 给死。
        distinct_values: Some(MAXIMUM_RETAINED_GENERATIONS),
        why: "D22-item2: K = 3S + 1, S<=16 => K<=49",
    },
    Arm {
        name: "node_group",
        // 节点组：组数是策略参数，**没有任何已定条款给它上界**。
        distinct_values: None,
        why: "E95 self-declared: widths not modelled; no settled clause bounds group count",
    },
];

fn fanout(node: u64, header: u64, range_field: u64, entry: u64) -> u64 {
    if entry == 0 {
        return 0;
    }
    let occupied_header_bytes = header.saturating_add(range_field);
    if node <= occupied_header_bytes {
        return 0;
    }
    (node - occupied_header_bytes) / entry
}

fn tree_height(record_count: u64, leaf_fanout: u64, inner_fanout: u64) -> Option<u64> {
    if leaf_fanout == 0 || inner_fanout < 2 {
        return None;
    }
    let mut height = 1u64;
    let mut covered_records = leaf_fanout as u128;
    while covered_records < record_count as u128 {
        covered_records = covered_records.saturating_mul(inner_fanout as u128);
        height += 1;
        if height > 64 {
            return None;
        }
    }
    Some(height)
}

/// **判据 2**：`w` 字节的标签装得下几个互不相同的值。
fn tag_capacity(tag_bytes: u64) -> u128 {
    if tag_bytes == 0 {
        return 1; // 0 字节只表达得了一个值 = 所有节点同组
    }
    if tag_bytes >= 16 {
        return u128::MAX;
    }
    1u128 << (8 * tag_bytes as u32)
}

/// **判据 4**：`groups` 个组塞进 `w` 字节的标签，被迫合并几个组。
/// `groups` 未知（`None`）时返回 `None`——**读不到 ≠ 读到 0**。
fn forced_merges(groups: Option<u64>, tag_bytes: u64) -> Option<u64> {
    let group_count = groups?;
    let capacity = tag_capacity(tag_bytes);
    if (group_count as u128) <= capacity {
        Some(0)
    } else {
        Some(group_count - capacity as u64)
    }
}

/// **判据 3**：第一个事务写出的节点字节，在「将来选任何一条臂」下是否逐字节相同。
/// 只有当**所有臂都装得进同一个已经定死的宽度**时才相同。
/// 有一条臂的宽度需求取不到 ⇒ 定不出那个宽度 ⇒ 字节写不出来 ⇒ **翻转不成立**。
fn first_transaction_bytes_stable(tag_bytes: u64) -> bool {
    ARMS.iter().all(|arm| matches!(forced_merges(arm.distinct_values, tag_bytes), Some(0)))
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_lines: Vec<String> = Vec::new();

    output_lines.push(emitter.emit_raw(&format!(
        "name=config node_bytes={NODE_BYTES} child_ptr={CHILD_POINTER_BYTES} k_max={MAXIMUM_RETAINED_GENERATIONS} arms={}",
        ARMS.len()
    )));

    // 判据 1：代价
    for &tag_bytes in TAG_WIDTHS.iter() {
        for &node_header_bytes in NODE_HEADERS.iter() {
            for &(tree_name, entry, key) in TREES.iter() {
                let header_with_tag = node_header_bytes + tag_bytes;
                let leaf_fanout = fanout(NODE_BYTES, header_with_tag, 0, entry);
                let inner_fanout = fanout(NODE_BYTES, header_with_tag, 0, key + CHILD_POINTER_BYTES);
                let leaf_fanout_without_tag = fanout(NODE_BYTES, node_header_bytes, 0, entry);
                output_lines.push(emitter.emit_raw(&format!(
                    "name=cost tag_bytes={tag_bytes} header={node_header_bytes} tree={tree_name} entry={entry} \
                     leaf_fanout={leaf_fanout} leaf_fanout_no_tag={leaf_fanout_without_tag} lost={} \
                     inner_fanout={inner_fanout} height_1e8={}",
                    leaf_fanout_without_tag.saturating_sub(leaf_fanout),
                    tree_height(100_000_000, leaf_fanout, inner_fanout)
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "NA".into())
                )));
            }
        }
    }

    // 判据 2 / 4：表达力与冲突
    for arm in ARMS.iter() {
        for &tag_bytes in TAG_WIDTHS.iter() {
            output_lines.push(emitter.emit_raw(&format!(
                "name=expressive arm={} tag_bytes={tag_bytes} distinct_needed={} capacity={} \
                 forced_merges={} why={}",
                arm.name,
                arm.distinct_values
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "UPPER_UNKNOWN".into()),
                tag_capacity(tag_bytes),
                forced_merges(arm.distinct_values, tag_bytes)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "UNDECIDABLE".into()),
                arm.why
            )));
        }
    }

    // 判据 3：翻转
    for &tag_bytes in TAG_WIDTHS.iter() {
        output_lines.push(emitter.emit_raw(&format!(
            "name=flip tag_bytes={tag_bytes} first_txn_bytes_stable={} verdict_can_flip_to_no={}",
            u8::from(first_transaction_bytes_stable(tag_bytes)),
            u8::from(first_transaction_bytes_stable(tag_bytes))
        )));
    }

    for output_line in &output_lines {
        println!("{output_line}");
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(CHILD_POINTER_BYTES, 59, "D19 已定项 4 之后");
        assert_eq!(MAXIMUM_RETAINED_GENERATIONS, 49, "D22 已定项 2：3 × 16 + 1");
        assert_eq!(TREES[0].1, 30, "D5 已定项 5");
        assert_eq!(TREES[1].1, 20, "D3 已定项 7");
    }

    /// **判据 1 的绝对值**：留位的扇出代价小到可以忽略，三棵树逐档钉死。
    #[test]
    fn criterion1_the_cost_of_reserving_is_small_and_absolute() {
        // 记账树条目 30、头 58：(16384−58)/30 = 544.2 ⇒ 544
        assert_eq!(fanout(NODE_BYTES, 58, 0, 30), 544);
        // 留 4 字节：(16384−62)/30 = 544.06 ⇒ 544，**一格都没掉**
        assert_eq!(fanout(NODE_BYTES, 62, 0, 30), 544);
        // 分配记录条目 20：(16384−58)/20 = 816.3 ⇒ 816；留 4 字节 ⇒ (16384−62)/20 = 816.1 ⇒ 816
        assert_eq!(fanout(NODE_BYTES, 58, 0, 20), 816);
        assert_eq!(fanout(NODE_BYTES, 62, 0, 20), 816);
        // inode 记录 140：116 → 116
        assert_eq!(fanout(NODE_BYTES, 58, 0, 140), 116);
        assert_eq!(fanout(NODE_BYTES, 62, 0, 140), 116);
        // ⇒ 代价不是反对留位的理由；反对它的是宽度定不出来（判据 2/3）
    }

    /// **判据 2 的绝对值**：代际分离要 49 个值（6 位够）；节点组的上界**指不到任何已定条款**。
    #[test]
    fn criterion2_one_arm_has_a_bound_and_the_other_does_not() {
        assert_eq!(ARMS[0].distinct_values, Some(49), "K = 3×16+1，D22 已定项 2");
        assert_eq!(ARMS[1].distinct_values, None, "节点组的组数没有任何已定条款给上界");
        // 1 字节装 256 个值 ⇒ 代际分离够用
        assert_eq!(tag_capacity(1), 256);
        assert_eq!(forced_merges(Some(49), 1), Some(0));
        // 0 字节只装 1 个值 ⇒ 49 个代必须挤成 1 个 ⇒ 被迫合并 48 个
        assert_eq!(tag_capacity(0), 1);
        assert_eq!(forced_merges(Some(49), 0), Some(48));
        // 节点组那条臂：任何宽度都判不出来，**不许退化成 0**
        for &tag_bytes in TAG_WIDTHS.iter() {
            assert_eq!(forced_merges(None, tag_bytes), None, "宽度 {tag_bytes} 上仍然判不出来");
        }
    }

    /// **判据 3 + 反向接受条款**：没有任何一档宽度能让第一个事务的字节稳定
    /// ⇒ **留位翻不了 D26 未定项 4 的「是」**。
    #[test]
    fn criterion3_reserving_cannot_flip_the_blocking_verdict() {
        for &tag_bytes in TAG_WIDTHS.iter() {
            assert!(!first_transaction_bytes_stable(tag_bytes), "宽度 {tag_bytes} 不该判成稳定");
        }
        // 阳性对照：把两条臂都换成有上界的，翻转就成立——证明这一维真的进了模型
        assert!(matches!(forced_merges(Some(49), 1), Some(0)));
        assert!(matches!(forced_merges(Some(200), 1), Some(0)));
        // ⇒ 翻不了的原因**只有一个**：节点组那条臂的宽度需求取不到
        assert_eq!(ARMS.iter().filter(|arm| arm.distinct_values.is_none()).count(), 1);
    }

    /// **判据 4 + 阳性对照 / 阴性对照**。
    #[test]
    fn criterion4_conflicts_are_counted_not_estimated() {
        // 阳性对照：0 字节时有上界的那条臂必须冲突
        assert!(forced_merges(ARMS[0].distinct_values, 0).unwrap() > 0);
        // 够宽时恰好 0
        assert_eq!(forced_merges(ARMS[0].distinct_values, 1), Some(0));
        assert_eq!(forced_merges(ARMS[0].distinct_values, 4), Some(0));
        // 阴性对照：只有一个组时任何宽度都不冲突，含 0 位
        assert_eq!(forced_merges(Some(1), 0), Some(0));
        assert_eq!(forced_merges(Some(1), 4), Some(0));
        // 边界：恰好装满不冲突，多一个就冲突一个
        assert_eq!(forced_merges(Some(256), 1), Some(0));
        assert_eq!(forced_merges(Some(257), 1), Some(1));
    }

    /// 树高在四档留位上都不动——代价这一维不构成反对理由。
    #[test]
    fn tag_width_does_not_move_tree_height() {
        for &(_, entry, key) in TREES.iter() {
            let mut previous_height: Option<u64> = None;
            for &tag_bytes in TAG_WIDTHS.iter() {
                let leaf_fanout = fanout(NODE_BYTES, 58 + tag_bytes, 0, entry);
                let inner_fanout = fanout(NODE_BYTES, 58 + tag_bytes, 0, key + CHILD_POINTER_BYTES);
                let height = tree_height(100_000_000, leaf_fanout, inner_fanout).unwrap();
                if let Some(previous) = previous_height {
                    assert_eq!(height, previous, "留位不该改树高");
                }
                previous_height = Some(height);
            }
        }
    }

    /// 不合法几何一律 None / 0。
    #[test]
    fn illegal_geometry_is_not_a_measurement() {
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES, 0, 30), 0);
        assert_eq!(tree_height(100, 0, 10), None);
        assert_eq!(forced_merges(None, 1), None);
    }
}
