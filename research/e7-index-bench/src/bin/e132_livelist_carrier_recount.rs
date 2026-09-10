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
const NODE_HDR: u64 = 64;
/// 树表单元装条目的净字节，`22-单元原子性怎么合成.md` 的口径（C157 记着另一个口径 16320，不采用）。
const TREE_TABLE_PAYLOAD: u64 = 16284;
/// 树表条目：长度 2 + 种类 2 + flags 2 + 树 ID 8 + 根指针 59 + prev_snap_txg 8 + 诞生 txg 8 + 预留 32。
const TREE_TABLE_ENTRY: u64 = 2 + 2 + 2 + 8 + 59 + 8 + 8 + 32;
/// 内部节点里一条子指针：头部 31 + 位置条目 14 × 2（D22 已定项 7）。
const CHILD_PTR: u64 = 31 + 14 * 2;
/// 每个头的数据树棵数（extent 树 + inode 树），已提交 first-txn-layout 第 47 行预想。
const DATA_TREES_PER_HEAD: u64 = 2;
/// 池级树棵数（分配记录树 + 记账树 + 中央映射树），同一行预想。
const POOL_TREES: u64 = 3;
/// 共享臂的 key 多出的头的树 ID。D8 已定项 8。
const TREE_ID_BYTES: u64 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    PerHead1,
    PerHeadT,
    Shared,
}

const ARMS: [(&str, Arm); 3] = [
    ("per_head_1", Arm::PerHead1),
    ("per_head_t", Arm::PerHeadT),
    ("shared", Arm::Shared),
];
const HEADS: [u64; 7] = [1, 4, 16, 64, 256, 1024, 4096];
const BLOCKS: [u64; 3] = [1024, 65536, 1048576];
const WIDTHS: [u64; 5] = [24, 32, 40, 48, 56];

fn tree_table_per_level() -> u64 {
    TREE_TABLE_PAYLOAD / TREE_TABLE_ENTRY
}

/// 这条臂在 H 个头时一共占几条树表条目。
fn tree_table_entries(arm: Arm, heads: u64) -> u64 {
    match arm {
        Arm::PerHead1 => heads * (DATA_TREES_PER_HEAD + 1) + POOL_TREES,
        Arm::PerHeadT => heads * (DATA_TREES_PER_HEAD + DATA_TREES_PER_HEAD) + POOL_TREES,
        Arm::Shared => heads * DATA_TREES_PER_HEAD + POOL_TREES + 1,
    }
}

/// 一张每层装 `per_level` 条的表，n 条要几层（叶算一层）。
fn table_levels(n: u64, per_level: u64) -> u32 {
    let mut lv = 1u32;
    let mut cap = per_level;
    while cap < n {
        cap = cap.saturating_mul(per_level);
        lv += 1;
    }
    lv
}

/// 这条臂最早在多少个头时进第 `lv` 层树表。
fn first_heads_at_level(arm: Arm, lv: u32) -> u64 {
    let mut h = 1u64;
    while table_levels(tree_table_entries(arm, h), tree_table_per_level()) < lv {
        h += 1;
    }
    h
}

fn leaf_fanout(key: u64) -> u64 {
    ((NODE_BYTES - NODE_HDR) / key).max(2)
}

fn internal_fanout(key: u64) -> u64 {
    ((NODE_BYTES - NODE_HDR) / (key + CHILD_PTR)).max(2)
}

/// n 条 key 宽 `key` 的条目，一次点查走几层（叶按叶扇出、往上按内部扇出）。
fn descents(n: u64, key: u64) -> u32 {
    let mut nodes = n.div_ceil(leaf_fanout(key)).max(1);
    let mut lv = 1u32;
    while nodes > 1 {
        nodes = nodes.div_ceil(internal_fanout(key));
        lv += 1;
    }
    lv
}

/// 点查下降次数：每头臂只在自己那棵 livelist 里走，共享臂在装着全部头的那棵里走。
fn arm_descents(arm: Arm, heads: u64, blocks: u64, width: u64) -> u32 {
    match arm {
        Arm::PerHead1 | Arm::PerHeadT => descents(blocks, width),
        Arm::Shared => descents(heads * blocks, width + TREE_ID_BYTES),
    }
}

fn main() {
    let mut e = Emitter::new();
    println!("E132 livelist 载体·按真实树数与内部扇出重算");
    println!("判据写死在 research/prompts/e132-preregistration.md（写于本装置之前）");
    println!(
        "口径：树表每层={} 条目={TREE_TABLE_ENTRY} 子指针={CHILD_PTR} 每头数据树={DATA_TREES_PER_HEAD} 池级树={POOL_TREES}",
        tree_table_per_level()
    );

    for (name, arm) in ARMS {
        for &h in HEADS.iter() {
            let tt = tree_table_entries(arm, h);
            println!(
                "{}",
                e.emit_raw(&format!(
                    "name=tree_table arm={name} heads={h} entries={tt} levels={}",
                    table_levels(tt, tree_table_per_level())
                ))
            );
        }
        for lv in [2u32, 3u32] {
            println!(
                "{}",
                e.emit_raw(&format!(
                    "name=criterion1 arm={name} level={lv} first_heads={}",
                    first_heads_at_level(arm, lv)
                ))
            );
        }
    }

    for &w in WIDTHS.iter() {
        let mut shallower = 0u32;
        let mut deeper = 0u32;
        let mut cells = 0u32;
        for &h in HEADS.iter() {
            for &n in BLOCKS.iter() {
                let a = arm_descents(Arm::PerHead1, h, n, w);
                let b = arm_descents(Arm::Shared, h, n, w);
                cells += 1;
                if a < b {
                    shallower += 1;
                }
                if a > b {
                    deeper += 1;
                }
                println!(
                    "{}",
                    e.emit_raw(&format!(
                        "name=criterion2 width={w} heads={h} blocks_per_head={n} per_head_1={a} shared={b}"
                    ))
                );
            }
        }
        println!(
            "{}",
            e.emit_raw(&format!(
                "name=criterion2_summary width={w} cells={cells} per_head_1_shallower={shallower} per_head_1_deeper={deeper}"
            ))
        );
        println!(
            "{}",
            e.emit_raw(&format!(
                "name=criterion3_capacity_ratio width={w} shared_over_per_head_1={:.4} judged=no",
                (w + TREE_ID_BYTES) as f64 / w as f64
            ))
        );
    }

    for (name, arm) in ARMS {
        println!(
            "{}",
            e.emit_raw(&format!(
                "name=positive_control arm={name} entries_h1={} entries_h4096={}",
                tree_table_entries(arm, 1),
                tree_table_entries(arm, 4096)
            ))
        );
    }
    println!("{}", e.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_the_kb_field_tables_segment_by_segment() {
        assert_eq!(TREE_TABLE_ENTRY, 121);
        assert_eq!(CHILD_PTR, 59);
        assert_eq!(TREE_TABLE_PAYLOAD, 16284);
        assert_eq!(tree_table_per_level(), 134);
    }

    #[test]
    fn tree_table_entry_counts_are_pinned() {
        assert_eq!(tree_table_entries(Arm::PerHead1, 1024), 3 * 1024 + 3);
        assert_eq!(tree_table_entries(Arm::PerHeadT, 1024), 4 * 1024 + 3);
        assert_eq!(tree_table_entries(Arm::Shared, 1024), 2 * 1024 + 4);
        assert_eq!(tree_table_entries(Arm::PerHead1, 1024), 3075);
        assert_eq!(tree_table_entries(Arm::Shared, 1024), 2052);
    }

    #[test]
    fn thresholds_match_the_preregistered_hand_arithmetic() {
        // 作废条款 2：3H+3、4H+3、2H+4 越过 134 与 134²。
        assert_eq!(first_heads_at_level(Arm::PerHead1, 2), 44);
        assert_eq!(first_heads_at_level(Arm::PerHead1, 3), 5985);
        assert_eq!(first_heads_at_level(Arm::PerHeadT, 2), 33);
        assert_eq!(first_heads_at_level(Arm::PerHeadT, 3), 4489);
        assert_eq!(first_heads_at_level(Arm::Shared, 2), 66);
        assert_eq!(first_heads_at_level(Arm::Shared, 3), 8977);
    }

    #[test]
    fn the_e131_count_was_one_data_tree_and_no_pool_trees() {
        // 留档：E131 的 `heads * 2` 只数了一棵数据树 + 一棵 livelist，没有池级树，
        // 于是它给的第二层门槛是 68，而按真实树数是 44。
        let e131 = |h: u64| h * 2;
        let first = (1u64..).find(|&h| table_levels(e131(h), 134) >= 2).unwrap();
        assert_eq!(first, 68);
        assert_ne!(first, first_heads_at_level(Arm::PerHead1, 2));
    }

    #[test]
    fn per_head_never_reaches_a_third_level_within_1024_heads() {
        // 判据 1 的判红条件：任一条 per_head 臂在 ≤ 1024 头进第三层 ⇒ per_head 输。
        for arm in [Arm::PerHead1, Arm::PerHeadT] {
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
        assert_eq!(arm_descents(Arm::PerHead1, 4096, 65536, 24), 2);
        // shared，宽 24 + 8 = 32，4096 × 65536 = 268435456 条：
        // ⌈…/510⌉ = 526345 叶，⌈/179⌉ = 2941，⌈/179⌉ = 17，⌈/179⌉ = 1 ⇒ 4 层。
        assert_eq!(arm_descents(Arm::Shared, 4096, 65536, 24), 4);
        assert_eq!(268435456u64.div_ceil(510), 526345);
        assert_eq!(526345u64.div_ceil(179), 2941);
        assert_eq!(2941u64.div_ceil(179), 17);
    }

    #[test]
    fn per_head_1_is_never_deeper_and_is_shallower_somewhere_at_every_width() {
        for &w in WIDTHS.iter() {
            let mut shallower = false;
            for &h in HEADS.iter() {
                for &n in BLOCKS.iter() {
                    let a = arm_descents(Arm::PerHead1, h, n, w);
                    let b = arm_descents(Arm::Shared, h, n, w);
                    assert!(a <= b, "宽 {w} H={h} N={n}：per_head_1 比 shared 深");
                    shallower |= a < b;
                }
            }
            assert!(shallower, "宽 {w}：per_head_1 逐格与 shared 相等 ⇒ 判据 2 触发");
        }
    }

    #[test]
    fn shared_key_includes_the_tree_id_at_a_sensitive_point() {
        // 第三类取样点不敏感的补丁：`descents_are_pinned_to_absolute_values` 那一格
        // 在宽 24 与宽 32 下恰好都是 4 层，于是「共享臂的 key 漏加树 ID」那条变异在那里同值。
        // 两层能装的条数：宽 24 是 680 × 196 = 133280，宽 32 是 510 × 179 = 91290
        // ⇒ 取 10 万条，宽 24 两层、宽 32 三层。
        assert_eq!(leaf_fanout(24) * internal_fanout(24), 133_280);
        assert_eq!(leaf_fanout(32) * internal_fanout(32), 91_290);
        assert_eq!(descents(100_000, 24), 2);
        assert_eq!(descents(100_000, 32), 3);
        assert_eq!(arm_descents(Arm::Shared, 1, 100_000, 24), 3);
    }

    #[test]
    fn single_head_descents_match_at_the_hint_width() {
        // 单头时共享树里只有一个头的条目；宽 24 与 32 在 1024 条时都是一层叶就装完。
        assert_eq!(arm_descents(Arm::PerHead1, 1, 1024, 24), 2);
        assert_eq!(arm_descents(Arm::Shared, 1, 1024, 24), 2);
    }

    #[test]
    fn capacity_ratio_is_reported_not_judged() {
        // 判据 3 不判输赢（fs-design.md「比谁省」不构成判据）；这里只钉算式。
        assert_eq!((24 + TREE_ID_BYTES) * 3, 24 * 4); // 4/3
        assert_eq!((48 + TREE_ID_BYTES) * 6, 48 * 7); // 7/6
    }

    #[test]
    fn positive_control_runs_on_every_arm() {
        for (_, arm) in ARMS {
            assert!(tree_table_entries(arm, 4096) > tree_table_entries(arm, 1));
        }
        assert_eq!(tree_table_entries(Arm::Shared, 1), 6);
        assert_eq!(tree_table_entries(Arm::PerHead1, 1), 6);
        assert_eq!(tree_table_entries(Arm::PerHeadT, 1), 7);
    }

    #[test]
    fn table_levels_counts_the_leaf_as_one() {
        assert_eq!(table_levels(134, 134), 1);
        assert_eq!(table_levels(135, 134), 2);
        assert_eq!(table_levels(17956, 134), 2);
        assert_eq!(table_levels(17957, 134), 3);
    }
}
