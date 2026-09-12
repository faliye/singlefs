//! E131：livelist 的两条载体，数值与代价。
//!
//! 用户 2026-09-10 对「每头一棵自己的树」逐字判「**这个有点狂暴了吧。你看看数值和性能呢。
//! 好的话也不是不能接受**」⇒ 这一格先有数再拍板，形态照 D3 已定项 7 的先例。
//!
//! **判据、口径、作废条款、以及「结果反过来我接不接受」写在
//! `research/prompts/e131-preregistration.md`，写于本文件之前。**
//!
//! ## 计数模型，不是实现
//!
//! 没有 I/O、没有并发、没有随机源 ⇒ 同一个二进制跑 N 遍必然逐字节一致。
//! 「N 轮一致」说明没有隐藏状态，不是统计上稳定。证据强度来自变异测试与钉死绝对值的断言。
//!
//! ## 跨装置口径：分母不自己挑
//!
//! 「树表单元每层装几棵」的分母，仓里有两个口径（16284 与 16320），差异记在 C157。
//! 本装置**按仓里那个（16284）算并注明出处**，不自己挑——
//! `.claude/rules/mutation-sampling.md` 第五类正是 2026-09-10 从 E128 那次立的。

use e7_index_bench::Emitter;

/// 索引节点字节数。D8 已定项 2。
const NODE_BYTES: u64 = 16384;
/// 树表单元装条目的净字节。`22-单元原子性怎么合成.md` 的口径，C148 用的也是它。
/// ⚠️ 仓里另有一个口径 16320（C157），本装置不采用。
const TREE_TABLE_PAYLOAD: u64 = 16284;
/// 一条树表条目。D8 已定项 8：长度 2 + 种类 2 + flags 2 + 树 ID 8 + 根指针 59
/// + `previous_snapshot_txg` 8 + 诞生 txg 8 + 预留 32。
const TREE_TABLE_ENTRY: u64 = 121;
/// 索引节点头。E73 给的基础节点头下界那一档，与 E128 同口径。
const NODE_HEADER_BYTES: u64 = 64;
/// livelist 一条事件记录：类型标签 2 + 位置条目 14（D19 已定项 4）+ birth txg 8。
///
/// ⚠️ **写成分解式不写总数**：2026-09-10 E130 的 `ALLOC_REC_BYTES` 就是只钉了一个总数（30），
/// 于是它挂着 D3 已定项 7 的名字、实际是 D5 已定项 5 的数，而断言与变异全绿。落 C261。
const EVENT_ENTRY_BYTES: u64 = 2 + 14 + 8;
/// 头的树 ID，进 key 时加的那一段。D8 已定项 8。
const TREE_IDENTIFIER_BYTES: u64 = 8;
/// E97 口径下每字节条目宽对应的占容量，16 TiB / 90% 填充、483 183 820 条。
/// 逐字取自 E97 那张表：415/15、442/16、553/20、387/14、193/7 都落在这个值上。
const CAPACITY_PARTS_PER_MILLION_PER_ENTRY_BYTE_AT_E97: f64 = 27.65;
/// E97 那张表的条目数基准。
const E97_ENTRIES: f64 = 483_183_820.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Reading {
    /// 这条臂一共要几棵树（不含 livelist 之外的树）。
    livelist_trees: u64,
    /// 树表里一共几条条目（每头的数据树 + 这条臂要的 livelist 树）。
    tree_table_entries: u64,
    /// 树表要几层间接。
    tree_table_levels: u32,
    /// livelist 条目总数。
    entries_total: u64,
    /// 一条条目多宽。
    entry_bytes: u64,
    /// 点查走几次树下降。
    descents: u32,
    /// 销毁一个头要触及多少条目。
    destroy_touch: u64,
    /// 批前预留算得出算不出：0 = 读那棵树自己的计数，1 = 要范围计数。
    pre_reserve_needs_range_scan: u64,
}

/// 一层能装几个 entry_bytes 宽的条目。
fn fanout(entry_bytes: u64) -> u64 {
    ((NODE_BYTES - NODE_HEADER_BYTES) / entry_bytes).max(2)
}

/// `entry_count` 条条目在扇出 `fanout_per_node` 的树上要几层（叶算一层）。
fn tree_level_count(entry_count: u64, fanout_per_node: u64) -> u32 {
    if entry_count <= 1 {
        return 1;
    }
    let mut level_count = 1u32;
    let mut capacity_in_entries = fanout_per_node;
    while capacity_in_entries < entry_count {
        capacity_in_entries = capacity_in_entries.saturating_mul(fanout_per_node);
        level_count += 1;
    }
    level_count
}

/// 树表每层装几棵树。
fn tree_table_entries_per_level() -> u64 {
    TREE_TABLE_PAYLOAD / TREE_TABLE_ENTRY
}

fn per_head(heads: u64, blocks_per_head: u64) -> Reading {
    let entry_bytes = EVENT_ENTRY_BYTES;
    // 每头两棵：它的数据树（D6 已定项 1 本来就要）+ 它自己的 livelist。
    let tree_table_entry_count = heads * 2;
    Reading {
        livelist_trees: heads,
        tree_table_entries: tree_table_entry_count,
        tree_table_levels: tree_level_count(tree_table_entry_count, tree_table_entries_per_level()),
        entries_total: heads * blocks_per_head,
        entry_bytes,
        // 点查只在自己那棵树里走。
        descents: tree_level_count(blocks_per_head, fanout(entry_bytes)),
        destroy_touch: blocks_per_head,
        pre_reserve_needs_range_scan: 0,
    }
}

fn shared(heads: u64, blocks_per_head: u64) -> Reading {
    let entry_bytes = EVENT_ENTRY_BYTES + TREE_IDENTIFIER_BYTES;
    // 每头一棵数据树，livelist 共用一棵。
    let tree_table_entry_count = heads + 1;
    Reading {
        livelist_trees: 1,
        tree_table_entries: tree_table_entry_count,
        tree_table_levels: tree_level_count(tree_table_entry_count, tree_table_entries_per_level()),
        entries_total: heads * blocks_per_head,
        entry_bytes,
        // 点查要在装着所有头的那棵树里走。
        descents: tree_level_count(heads * blocks_per_head, fanout(entry_bytes)),
        destroy_touch: blocks_per_head,
        // 「这个头有多少条」在共享树里只能按前缀数。
        pre_reserve_needs_range_scan: 1,
    }
}

fn capacity_parts_per_million(reading: &Reading) -> f64 {
    // 按 E97 口径线性缩放：条目宽 × 每字节 ppm × (本臂条目数 / E97 基准条目数)。
    CAPACITY_PARTS_PER_MILLION_PER_ENTRY_BYTE_AT_E97 * reading.entry_bytes as f64 * (reading.entries_total as f64 / E97_ENTRIES)
}

const ARMS: [(&str, fn(u64, u64) -> Reading); 2] = [("per_head", per_head), ("shared", shared)];
const HEADS_SWEPT: [u64; 7] = [1, 4, 16, 64, 256, 1024, 4096];
const BLOCKS_PER_HEAD_SWEPT: [u64; 3] = [1024, 65536, 1048576];

fn main() {
    let mut emitter = Emitter::new();
    println!("E131 livelist 的两条载体，数值与代价");
    println!("判据写死在 research/prompts/e131-preregistration.md（写于本装置之前）");
    println!(
        "口径：NODE_BYTES={NODE_BYTES} TREE_TABLE_PAYLOAD={TREE_TABLE_PAYLOAD} \
         TREE_TABLE_ENTRY={TREE_TABLE_ENTRY} 每层={} EVENT_ENTRY={EVENT_ENTRY_BYTES}",
        tree_table_entries_per_level()
    );

    for (arm_name, arm_reading) in ARMS {
        for &heads in HEADS_SWEPT.iter() {
            for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                let reading = arm_reading(heads, blocks_per_head);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name={arm_name} heads={heads} blocks_per_head={blocks_per_head} livelist_trees={} \
                         tree_table_entries={} tree_table_levels={} entries_total={} \
                         entry_bytes={} descents={} destroy_touch={} \
                         pre_reserve_range_scan={} ppm={:.1}",
                        reading.livelist_trees,
                        reading.tree_table_entries,
                        reading.tree_table_levels,
                        reading.entries_total,
                        reading.entry_bytes,
                        reading.descents,
                        reading.destroy_touch,
                        reading.pre_reserve_needs_range_scan,
                        capacity_parts_per_million(&reading)
                    ))
                );
            }
        }
    }

    // 判据 1：per_head 在哪个头数上压出第二层、第三层。
    for level in [2u32, 3u32] {
        let first_heads = HEADS_SWEPT
            .iter()
            .find(|&&heads| per_head(heads, 1024).tree_table_levels >= level)
            .copied();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=criterion1 arm=per_head level={level} first_heads={}",
                first_heads.map(|heads| heads.to_string()).unwrap_or_else(|| "none_in_sweep".into())
            ))
        );
    }

    // 判据 2：点查下降次数的逐格差。
    for &heads in HEADS_SWEPT.iter() {
        for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
            let per_head_descents = per_head(heads, blocks_per_head).descents;
            let shared_descents = shared(heads, blocks_per_head).descents;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=criterion2 heads={heads} blocks_per_head={blocks_per_head} \
                     per_head_descents={per_head_descents} shared_descents={shared_descents} delta={}",
                    shared_descents as i64 - per_head_descents as i64
                ))
            );
        }
    }

    // 判据 6 的阳性对照。
    let tree_table_entries_at_1_head = per_head(1, 1024).tree_table_entries;
    let tree_table_entries_at_4096_heads = per_head(4096, 1024).tree_table_entries;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control arm=per_head tt_entries_h1={tree_table_entries_at_1_head} tt_entries_h4096={tree_table_entries_at_4096_heads} ratio={}",
            tree_table_entries_at_4096_heads / tree_table_entries_at_1_head
        ))
    );

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 钉绝对值 ──────────────────────────────────────────────────────

    #[test]
    fn tree_table_holds_exactly_one_hundred_thirty_four_per_level() {
        // 作废条款 3：16284 / 121 = 134。分母按仓里那个取，不自己挑（C157）。
        assert_eq!(TREE_TABLE_PAYLOAD, 16284);
        assert_eq!(TREE_TABLE_ENTRY, 121);
        assert_eq!(tree_table_entries_per_level(), 134);
    }

    #[test]
    fn the_other_tree_table_denominator_in_the_repo_agrees_at_121_bytes_and_diverges_at_137() {
        // 留档：仓里另一个口径是 16320（C157）。它在 121 这一档上给同一个数，
        // 所以「基线对上了」在这里不构成证据——第五类那条纪律的原样形态。
        assert_eq!(16320u64 / TREE_TABLE_ENTRY, 134);
        assert_eq!(TREE_TABLE_PAYLOAD / TREE_TABLE_ENTRY, 134);
        // 而在 137 这一档上两者分道（E128 那次就是栽在这里）。
        assert_eq!(16320u64 / 137, 119);
        assert_eq!(TREE_TABLE_PAYLOAD / 137, 118);
    }

    #[test]
    fn per_head_tree_table_entries_are_exactly_two_per_head() {
        // 作废条款 3：H=1024 时正好 2048。
        assert_eq!(per_head(1024, 1024).tree_table_entries, 2048);
        assert_eq!(per_head(1, 1024).tree_table_entries, 2);
        assert_eq!(shared(1024, 1024).tree_table_entries, 1025);
    }

    #[test]
    fn entries_total_is_heads_times_blocks_on_both_arms() {
        // 作废条款 2。
        for &heads in HEADS_SWEPT.iter() {
            for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                assert_eq!(per_head(heads, blocks_per_head).entries_total, heads * blocks_per_head);
                assert_eq!(shared(heads, blocks_per_head).entries_total, heads * blocks_per_head);
            }
        }
    }

    #[test]
    fn entry_widths_come_from_the_knowledge_base_coordinate_system() {
        // 24 = 类型标签 2 + 位置条目 14（D19 已定项 4）+ birth txg 8。
        assert_eq!(EVENT_ENTRY_BYTES, 2 + 14 + 8);
        // 共享臂多的正是树 ID 那 8 字节（D8 已定项 8）。
        assert_eq!(shared(4, 1024).entry_bytes, EVENT_ENTRY_BYTES + TREE_IDENTIFIER_BYTES);
        assert_eq!(shared(4, 1024).entry_bytes, 32);
        assert_eq!(per_head(4, 1024).entry_bytes, 24);
    }

    // ── 判据 1：树表层数 ──────────────────────────────────────────────

    #[test]
    fn per_head_needs_a_second_tree_table_level_only_past_sixty_seven_heads() {
        // 每头两条 ⇒ 134 条装得下 67 个头。
        assert_eq!(per_head(67, 1024).tree_table_levels, 1);
        assert_eq!(per_head(68, 1024).tree_table_levels, 2);
    }

    #[test]
    fn per_head_stays_at_two_tree_table_levels_through_the_whole_sweep() {
        // 判据 1 的判红条件：H ≤ 64 就压到三层 ⇒ per_head 输。
        assert!(per_head(64, 1024).tree_table_levels < 3, "H=64 就三层 ⇒ per_head 输");
        for &heads in HEADS_SWEPT.iter() {
            assert!(
                per_head(heads, 1024).tree_table_levels <= 2,
                "H={heads} 时 per_head 已经要三层"
            );
        }
        // 三层的门槛：134² = 17956 条 ⇒ 8978 个头。
        assert_eq!(134u64 * 134, 17956);
        assert_eq!(per_head(8978, 1024).tree_table_levels, 2);
        assert_eq!(per_head(8979, 1024).tree_table_levels, 3);
    }

    // ── 判据 2：点查 ──────────────────────────────────────────────────

    #[test]
    fn shared_is_never_shallower_than_per_head() {
        for &heads in HEADS_SWEPT.iter() {
            for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                assert!(
                    shared(heads, blocks_per_head).descents >= per_head(heads, blocks_per_head).descents,
                    "H={heads} N={blocks_per_head}：共享树竟然比每头一棵浅"
                );
            }
        }
    }

    #[test]
    fn the_two_arms_differ_in_descents_somewhere_in_the_sweep() {
        // 判据 2 的判红条件：逐格相等 ⇒ per_head 换不到读上的好处。
        let any_cell_differs = HEADS_SWEPT.iter().any(|&heads| {
            BLOCKS_PER_HEAD_SWEPT
                .iter()
                .any(|&blocks_per_head| shared(heads, blocks_per_head).descents != per_head(heads, blocks_per_head).descents)
        });
        assert!(any_cell_differs, "两臂点查逐格相等 ⇒ per_head 输这一格");
    }

    #[test]
    fn descents_are_pinned_to_absolute_values_not_only_compared() {
        // 真盲区的补丁：此前两条断言都只做臂间互比（「共享不比每头浅」「某处不同」），
        // 把 per_head 改成按全池算下降次数，两条都照样绿——所有臂一起错时互比看不见。
        // per_head：1024 条 > 扇出 680 ⇒ 2 层。
        assert_eq!(per_head(4096, 1024).descents, 2);
        // shared：4096 × 1024 = 4194304 条，扇出 510，510² = 260100 < 4194304 ≤ 510³ ⇒ 3 层。
        assert_eq!(shared(4096, 1024).descents, 3);
        assert_eq!(4096u64 * 1024, 4_194_304);
        // 单头时两臂同深（共享树里只有一个头的条目）。
        assert_eq!(per_head(1, 1024).descents, 2);
        assert_eq!(shared(1, 1024).descents, 2);
    }

    #[test]
    fn fanout_at_twenty_four_and_thirty_two_bytes() {
        assert_eq!(fanout(24), (16384 - 64) / 24);
        assert_eq!(fanout(24), 680);
        assert_eq!(fanout(32), 510);
    }

    #[test]
    fn tree_level_count_counts_the_leaf_as_one() {
        assert_eq!(tree_level_count(1, 680), 1);
        assert_eq!(tree_level_count(680, 680), 1);
        assert_eq!(tree_level_count(681, 680), 2);
        assert_eq!(tree_level_count(680 * 680, 680), 2);
        assert_eq!(tree_level_count(680 * 680 + 1, 680), 3);
    }

    // ── 判据 3 / 5 ────────────────────────────────────────────────────

    #[test]
    fn destroy_touch_is_the_same_on_both_arms() {
        // 判据 3 跑前就预计两臂相等：销毁触及的是同一批条目，只是住法不同。
        // 写下来是为了让「这一格没判别力」成为产物里的事实，而不是事后的说法。
        for &heads in HEADS_SWEPT.iter() {
            for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                assert_eq!(per_head(heads, blocks_per_head).destroy_touch, shared(heads, blocks_per_head).destroy_touch);
            }
        }
    }

    #[test]
    fn only_the_shared_arm_needs_a_range_scan_for_pre_reserve() {
        assert_eq!(per_head(16, 1024).pre_reserve_needs_range_scan, 0);
        assert_eq!(shared(16, 1024).pre_reserve_needs_range_scan, 1);
    }

    // ── 判据 4：占容量 ────────────────────────────────────────────────

    #[test]
    fn shared_costs_exactly_four_thirds_of_per_head_in_capacity() {
        // 32 / 24 = 4/3，条目数两臂相同 ⇒ ppm 比值恒 4/3。
        for &heads in HEADS_SWEPT.iter() {
            for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                let per_head_parts_per_million = capacity_parts_per_million(&per_head(heads, blocks_per_head));
                let shared_parts_per_million = capacity_parts_per_million(&shared(heads, blocks_per_head));
                assert!((shared_parts_per_million / per_head_parts_per_million - 4.0 / 3.0).abs() < 1e-9, "H={heads} N={blocks_per_head}");
            }
        }
    }

    #[test]
    fn capacity_parts_per_million_scales_with_entry_count_away_from_the_e97_baseline() {
        // 第三类取样点不敏感的补丁：下面那条断言钉在 E97 基准条目数上，
        // 缩放因子在那一点正好是 1 ⇒「不随条目数缩放」的变异在那里与原式同值。
        // 换一个离开基准的取样点：1024 头 × 1048576 块 = 1073741824 条，是基准的约 2.22 倍。
        let reading = per_head(1024, 1048576);
        assert_eq!(reading.entries_total, 1_073_741_824);
        let expected_parts_per_million = CAPACITY_PARTS_PER_MILLION_PER_ENTRY_BYTE_AT_E97 * 24.0 * (1_073_741_824.0 / E97_ENTRIES);
        assert!((capacity_parts_per_million(&reading) - expected_parts_per_million).abs() < 1e-6);
        assert!(capacity_parts_per_million(&reading) > 1400.0 && capacity_parts_per_million(&reading) < 1500.0, "算出 {:.1}", capacity_parts_per_million(&reading));
    }

    #[test]
    fn capacity_parts_per_million_matches_the_e97_scale_at_its_own_baseline() {
        // 跨装置闸：把条目数喂成 E97 的基准、宽度喂成 20，必须落回 E97 表里的 553 ppm。
        let reading = Reading {
            livelist_trees: 1,
            tree_table_entries: 1,
            tree_table_levels: 1,
            entries_total: E97_ENTRIES as u64,
            entry_bytes: 20,
            descents: 1,
            destroy_touch: 0,
            pre_reserve_needs_range_scan: 0,
        };
        assert!((capacity_parts_per_million(&reading) - 553.0).abs() < 1.0, "算出来 {:.1}，E97 表里是 553", capacity_parts_per_million(&reading));
    }

    // ── 判据 6：阳性对照 ──────────────────────────────────────────────

    #[test]
    fn positive_control_tree_table_entries_grow_with_heads() {
        let tree_table_entries_at_1_head = per_head(1, 1024).tree_table_entries;
        let tree_table_entries_at_4096_heads = per_head(4096, 1024).tree_table_entries;
        assert_ne!(tree_table_entries_at_1_head, tree_table_entries_at_4096_heads, "阳性对照：树表条目数没随头数长 ⇒ 整轮作废");
        assert_eq!(tree_table_entries_at_4096_heads / tree_table_entries_at_1_head, 4096);
    }

    #[test]
    fn positive_control_runs_on_both_arms_not_just_the_first() {
        // 「阳性对照必须对每一条被测的臂都跑」。
        assert_ne!(
            shared(1, 1024).tree_table_entries,
            shared(4096, 1024).tree_table_entries
        );
        assert_eq!(shared(4096, 1024).tree_table_entries, 4097);
    }
}
