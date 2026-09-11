//! E132：livelist 载体·按真实树数与内部扇出重算。
//!
//! E131 的数被用户拿去表了倾向，而三轮对抗第一轮的反推腿打中它两处建模错误，主 agent 现查坐实：
//! ① 树数——E131 按「每头一棵数据树、没有池级树」数树表条目；已提交的 `first-txn-layout.md`
//!   预想「extent 树 1、inode 树 2、分配记录树 3、记账树 4、中央映射树 5」⇒ 每头至少两棵数据树，外加三棵池级树；
//! ② 内部扇出——E131 对所有层都用叶条目宽，而内部节点条目是 key 加一条 59 字节的子指针。
//!
//! **判据、口径、作废条款、以及「结果反过来我接不接受」写在
//! `research/prompts/e132-preregistration.md`，写于本文件之前。**
//! E131 原样保留（判据已写死、产物已入库），按 E95→E127、E20→E128 的先例另立新号。
//!
//! 计数模型，不是实现：没有 I/O、没有随机源 ⇒ 跑 N 遍必然逐字节一致，
//! 证据强度来自变异测试与钉死绝对值的断言。
//!
//! ⚠️ **快照不计**：D8 已定项 8 逐字「快照进树表」，一个快照占几条条目今天没定
//! ⇒ 树表层数门槛只会比这里报的更早。
//! ⚠️ **条目宽只能扫**：条目身份要改用中央映射的 key（位置条目只是提示，D19 已定项 5），
//! 而那个 key 是 D19 未定项 6、今天没定。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384; // D8 已定项 2
const NODE_HEADER_BYTES: u64 = 64;
/// 树表单元装条目的净字节，`22-单元原子性怎么合成.md` 的口径（C157 记着另一个口径 16320，不采用）。
const TREE_TABLE_PAYLOAD: u64 = 16284;
/// 树表条目：长度 2 + 种类 2 + flags 2 + 树 ID 8 + 根指针 59 + prev_snap_txg 8 + 诞生 txg 8 + 预留 32。
const TREE_TABLE_ENTRY: u64 = 2 + 2 + 2 + 8 + 59 + 8 + 8 + 32;
/// 内部节点里一条子指针：头部 31 + 位置条目 14 × 2（D22 已定项 7）。
const CHILD_POINTER_BYTES: u64 = 31 + 14 * 2;
/// 每个头的数据树棵数（extent 树 + inode 树），已提交 first-txn-layout 第 47 行预想。
const DATA_TREES_PER_HEAD: u64 = 2;
/// 池级树棵数（分配记录树 + 记账树 + 中央映射树），同一行预想。
const POOL_TREES: u64 = 3;
/// 共享臂的 key 多出的头的树 ID。D8 已定项 8。
const TREE_IDENTIFIER_BYTES: u64 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    OneLivelistPerHead,
    OneLivelistPerDataTree,
    SharedLivelist,
}

const ARMS: [(&str, Arm); 3] = [
    ("per_head_1", Arm::OneLivelistPerHead),
    ("per_head_t", Arm::OneLivelistPerDataTree),
    ("shared", Arm::SharedLivelist),
];
const HEADS_SWEPT: [u64; 7] = [1, 4, 16, 64, 256, 1024, 4096];
const BLOCKS_PER_HEAD_SWEPT: [u64; 3] = [1024, 65536, 1048576];
const ENTRY_BYTES_SWEPT: [u64; 5] = [24, 32, 40, 48, 56];

fn tree_table_entries_per_level() -> u64 {
    TREE_TABLE_PAYLOAD / TREE_TABLE_ENTRY
}

/// 这条臂在 H 个头时一共占几条树表条目。
fn tree_table_entries(arm: Arm, heads: u64) -> u64 {
    match arm {
        Arm::OneLivelistPerHead => heads * (DATA_TREES_PER_HEAD + 1) + POOL_TREES,
        Arm::OneLivelistPerDataTree => heads * (DATA_TREES_PER_HEAD + DATA_TREES_PER_HEAD) + POOL_TREES,
        Arm::SharedLivelist => heads * DATA_TREES_PER_HEAD + POOL_TREES + 1,
    }
}

/// 一张每层装 `fanout_per_node` 条的表，`entry_count` 条要几层（叶算一层）。
fn tree_level_count(entry_count: u64, fanout_per_node: u64) -> u32 {
    let mut level_count = 1u32;
    let mut capacity_in_entries = fanout_per_node;
    while capacity_in_entries < entry_count {
        capacity_in_entries = capacity_in_entries.saturating_mul(fanout_per_node);
        level_count += 1;
    }
    level_count
}

/// 这条臂最早在多少个头时进第 `level` 层树表。
fn first_heads_at_level(arm: Arm, level: u32) -> u64 {
    let mut heads = 1u64;
    while tree_level_count(tree_table_entries(arm, heads), tree_table_entries_per_level()) < level {
        heads += 1;
    }
    heads
}

fn leaf_fanout(entry_bytes: u64) -> u64 {
    ((NODE_BYTES - NODE_HEADER_BYTES) / entry_bytes).max(2)
}

fn internal_fanout(entry_bytes: u64) -> u64 {
    ((NODE_BYTES - NODE_HEADER_BYTES) / (entry_bytes + CHILD_POINTER_BYTES)).max(2)
}

/// `entry_count` 条 key 宽 `entry_bytes` 的条目，一次点查走几层（叶按叶扇出、往上按内部扇出）。
fn descents(entry_count: u64, entry_bytes: u64) -> u32 {
    let mut node_count = entry_count.div_ceil(leaf_fanout(entry_bytes)).max(1);
    let mut level_count = 1u32;
    while node_count > 1 {
        node_count = node_count.div_ceil(internal_fanout(entry_bytes));
        level_count += 1;
    }
    level_count
}

/// 点查下降次数：每头臂只在自己那棵 livelist 里走，共享臂在装着全部头的那棵里走。
fn arm_descents(arm: Arm, heads: u64, blocks_per_head: u64, entry_bytes: u64) -> u32 {
    match arm {
        Arm::OneLivelistPerHead | Arm::OneLivelistPerDataTree => descents(blocks_per_head, entry_bytes),
        Arm::SharedLivelist => descents(heads * blocks_per_head, entry_bytes + TREE_IDENTIFIER_BYTES),
    }
}

fn main() {
    let mut emitter = Emitter::new();
    println!("E132 livelist 载体·按真实树数与内部扇出重算");
    println!("判据写死在 research/prompts/e132-preregistration.md（写于本装置之前）");
    println!(
        "口径：树表每层={} 条目={TREE_TABLE_ENTRY} 子指针={CHILD_POINTER_BYTES} 每头数据树={DATA_TREES_PER_HEAD} 池级树={POOL_TREES}",
        tree_table_entries_per_level()
    );

    for (arm_name, arm) in ARMS {
        for &heads in HEADS_SWEPT.iter() {
            let tree_table_entry_count = tree_table_entries(arm, heads);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=tree_table arm={arm_name} heads={heads} entries={tree_table_entry_count} levels={}",
                    tree_level_count(tree_table_entry_count, tree_table_entries_per_level())
                ))
            );
        }
        for level in [2u32, 3u32] {
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=criterion1 arm={arm_name} level={level} first_heads={}",
                    first_heads_at_level(arm, level)
                ))
            );
        }
    }

    for &entry_bytes in ENTRY_BYTES_SWEPT.iter() {
        let mut shallower_cell_count = 0u32;
        let mut deeper_cell_count = 0u32;
        let mut cell_count = 0u32;
        for &heads in HEADS_SWEPT.iter() {
            for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                let one_livelist_per_head_descents = arm_descents(Arm::OneLivelistPerHead, heads, blocks_per_head, entry_bytes);
                let shared_livelist_descents = arm_descents(Arm::SharedLivelist, heads, blocks_per_head, entry_bytes);
                cell_count += 1;
                if one_livelist_per_head_descents < shared_livelist_descents {
                    shallower_cell_count += 1;
                }
                if one_livelist_per_head_descents > shared_livelist_descents {
                    deeper_cell_count += 1;
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=criterion2 width={entry_bytes} heads={heads} blocks_per_head={blocks_per_head} per_head_1={one_livelist_per_head_descents} shared={shared_livelist_descents}"
                    ))
                );
            }
        }
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=criterion2_summary width={entry_bytes} cells={cell_count} per_head_1_shallower={shallower_cell_count} per_head_1_deeper={deeper_cell_count}"
            ))
        );
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=criterion3_capacity_ratio width={entry_bytes} shared_over_per_head_1={:.4} judged=no",
                (entry_bytes + TREE_IDENTIFIER_BYTES) as f64 / entry_bytes as f64
            ))
        );
    }

    for (arm_name, arm) in ARMS {
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=positive_control arm={arm_name} entries_h1={} entries_h4096={}",
                tree_table_entries(arm, 1),
                tree_table_entries(arm, 4096)
            ))
        );
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_the_knowledge_base_field_tables_segment_by_segment() {
        assert_eq!(TREE_TABLE_ENTRY, 121);
        assert_eq!(CHILD_POINTER_BYTES, 59);
        assert_eq!(TREE_TABLE_PAYLOAD, 16284);
        assert_eq!(tree_table_entries_per_level(), 134);
    }

    #[test]
    fn tree_table_entry_counts_are_pinned() {
        assert_eq!(tree_table_entries(Arm::OneLivelistPerHead, 1024), 3 * 1024 + 3);
        assert_eq!(tree_table_entries(Arm::OneLivelistPerDataTree, 1024), 4 * 1024 + 3);
        assert_eq!(tree_table_entries(Arm::SharedLivelist, 1024), 2 * 1024 + 4);
        assert_eq!(tree_table_entries(Arm::OneLivelistPerHead, 1024), 3075);
        assert_eq!(tree_table_entries(Arm::SharedLivelist, 1024), 2052);
    }

    #[test]
    fn thresholds_match_the_preregistered_hand_arithmetic() {
        // 作废条款 2：3H+3、4H+3、2H+4 越过 134 与 134²。
        assert_eq!(first_heads_at_level(Arm::OneLivelistPerHead, 2), 44);
        assert_eq!(first_heads_at_level(Arm::OneLivelistPerHead, 3), 5985);
        assert_eq!(first_heads_at_level(Arm::OneLivelistPerDataTree, 2), 33);
        assert_eq!(first_heads_at_level(Arm::OneLivelistPerDataTree, 3), 4489);
        assert_eq!(first_heads_at_level(Arm::SharedLivelist, 2), 66);
        assert_eq!(first_heads_at_level(Arm::SharedLivelist, 3), 8977);
    }

    #[test]
    fn the_e131_count_was_one_data_tree_and_no_pool_trees() {
        // 留档：E131 的 `heads * 2` 只数了一棵数据树 + 一棵 livelist，没有池级树，
        // 于是它给的第二层门槛是 68，而按真实树数是 44。
        let e131_tree_table_entry_count = |heads: u64| heads * 2;
        let e131_first_heads_at_second_level = (1u64..)
            .find(|&heads| tree_level_count(e131_tree_table_entry_count(heads), 134) >= 2)
            .unwrap();
        assert_eq!(e131_first_heads_at_second_level, 68);
        assert_ne!(e131_first_heads_at_second_level, first_heads_at_level(Arm::OneLivelistPerHead, 2));
    }

    #[test]
    fn per_head_never_reaches_a_third_level_within_1024_heads() {
        // 判据 1 的判红条件：任一条 per_head 臂在 ≤ 1024 头进第三层 ⇒ per_head 输。
        for arm in [Arm::OneLivelistPerHead, Arm::OneLivelistPerDataTree] {
            assert!(first_heads_at_level(arm, 3) > 1024, "{arm:?} 在 1024 头内就进第三层");
        }
    }

    #[test]
    fn internal_fanout_is_not_the_leaf_fanout() {
        // 留档 E131 的另一处：它对内部层也用了叶扇出。
        assert_eq!(leaf_fanout(24), 680);
        assert_eq!(internal_fanout(24), 196);
        assert_eq!(leaf_fanout(32), 510);
        assert_eq!(internal_fanout(32), 179);
    }

    #[test]
    fn descents_are_pinned_to_absolute_values() {
        // per_head_1，宽 24，65536 条：⌈65536/680⌉ = 97 个叶，⌈97/196⌉ = 1 ⇒ 2 层。
        assert_eq!(arm_descents(Arm::OneLivelistPerHead, 4096, 65536, 24), 2);
        // shared，宽 24 + 8 = 32，4096 × 65536 = 268435456 条：
        // ⌈…/510⌉ = 526345 叶，⌈/179⌉ = 2941，⌈/179⌉ = 17，⌈/179⌉ = 1 ⇒ 4 层。
        assert_eq!(arm_descents(Arm::SharedLivelist, 4096, 65536, 24), 4);
        assert_eq!(268435456u64.div_ceil(510), 526345);
        assert_eq!(526345u64.div_ceil(179), 2941);
        assert_eq!(2941u64.div_ceil(179), 17);
    }

    #[test]
    fn per_head_1_is_never_deeper_and_is_shallower_somewhere_at_every_width() {
        for &entry_bytes in ENTRY_BYTES_SWEPT.iter() {
            let mut is_shallower_somewhere = false;
            for &heads in HEADS_SWEPT.iter() {
                for &blocks_per_head in BLOCKS_PER_HEAD_SWEPT.iter() {
                    let one_livelist_per_head_descents = arm_descents(Arm::OneLivelistPerHead, heads, blocks_per_head, entry_bytes);
                    let shared_livelist_descents = arm_descents(Arm::SharedLivelist, heads, blocks_per_head, entry_bytes);
                    assert!(
                        one_livelist_per_head_descents <= shared_livelist_descents,
                        "宽 {entry_bytes} H={heads} N={blocks_per_head}：per_head_1 比 shared 深"
                    );
                    is_shallower_somewhere |= one_livelist_per_head_descents < shared_livelist_descents;
                }
            }
            assert!(is_shallower_somewhere, "宽 {entry_bytes}：per_head_1 逐格与 shared 相等 ⇒ 判据 2 触发");
        }
    }

    #[test]
    fn shared_key_includes_the_tree_identifier_at_a_sensitive_point() {
        // 第三类取样点不敏感的补丁：`descents_are_pinned_to_absolute_values` 那一格
        // 在宽 24 与宽 32 下恰好都是 4 层，于是「共享臂的 key 漏加树 ID」那条变异在那里同值。
        // 两层能装的条数：宽 24 是 680 × 196 = 133280，宽 32 是 510 × 179 = 91290
        // ⇒ 取 10 万条，宽 24 两层、宽 32 三层。
        assert_eq!(leaf_fanout(24) * internal_fanout(24), 133_280);
        assert_eq!(leaf_fanout(32) * internal_fanout(32), 91_290);
        assert_eq!(descents(100_000, 24), 2);
        assert_eq!(descents(100_000, 32), 3);
        assert_eq!(arm_descents(Arm::SharedLivelist, 1, 100_000, 24), 3);
    }

    #[test]
    fn single_head_descents_match_at_the_hint_width() {
        // 单头时共享树里只有一个头的条目；宽 24 与 32 在 1024 条时都是一层叶就装完。
        assert_eq!(arm_descents(Arm::OneLivelistPerHead, 1, 1024, 24), 2);
        assert_eq!(arm_descents(Arm::SharedLivelist, 1, 1024, 24), 2);
    }

    #[test]
    fn capacity_ratio_is_reported_not_judged() {
        // 判据 3 不判输赢（fs-design.md「比谁省」不构成判据）；这里只钉算式。
        assert_eq!((24 + TREE_IDENTIFIER_BYTES) * 3, 24 * 4); // 4/3
        assert_eq!((48 + TREE_IDENTIFIER_BYTES) * 6, 48 * 7); // 7/6
    }

    #[test]
    fn positive_control_runs_on_every_arm() {
        for (_, arm) in ARMS {
            assert!(tree_table_entries(arm, 4096) > tree_table_entries(arm, 1));
        }
        assert_eq!(tree_table_entries(Arm::SharedLivelist, 1), 6);
        assert_eq!(tree_table_entries(Arm::OneLivelistPerHead, 1), 6);
        assert_eq!(tree_table_entries(Arm::OneLivelistPerDataTree, 1), 7);
    }

    #[test]
    fn tree_level_count_counts_the_leaf_as_one() {
        assert_eq!(tree_level_count(134, 134), 1);
        assert_eq!(tree_level_count(135, 134), 2);
        assert_eq!(tree_level_count(17956, 134), 2);
        assert_eq!(tree_level_count(17957, 134), 3);
    }
}
