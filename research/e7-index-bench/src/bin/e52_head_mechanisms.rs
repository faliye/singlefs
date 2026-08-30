//! E52：两种多可写头机制的运营代价 —— 答 D6 已定项 1。
//!
//! 做不做多可写头已定（做，付查找）；**取哪条机制没定**，而 E35 自陈它只量了两个
//! 构造给出的常数、**不足以在两条之间做选择**。本实验把那两个换成可扫的量，再补三个。
//!
//! ⚠️ **本实验不替人做定案**，它的产出是一组数，让定案落在数字上。
//!
//! ## 两条臂（逐字取自 D6 已定项 1）
//!
//! ① `key_snapshot_dim`：单树，快照 ID 进 key 低位，查找按祖先关系过滤。
//! ② `tree_per_head`：每头一棵自己的树，各持一份 `prev_snap_txg` 与 deadlist。
//!
//! ## 判据（跑前写死）
//!
//! 1. 「分得开」= 至少一个指标上两条臂差 ≥ 2 倍。达不到 ⇒ 维持 E35 的判定（分不出来）。
//! 2. 每个指标各有一条独立算术给出的绝对值断言，不许只做臂间互比。
//! 3. 五个指标分别给，**不合成一个总代价**——量纲不同，加权就是替人定案。
//!
//! ## 失败条款
//!
//! - 阳性对照 A：头数翻倍 ⇒ ① 的「删一个头触及的 key 数」翻倍，② 不变。不满足 ⇒ 整轮作废。
//! - 阳性对照 B：头数 1 且共享率 0 ⇒ 两条臂的旁表条目与固定元数据条数相同。不同 ⇒ 整轮作废。
//! - 阴性对照：共享率 0 ⇒ 旁表条目必须为 0。
//! - 两条臂全部指标逐格相同 ⇒ **如实记录「仍分不出来」**，不许调参数凑差别。
//!
//! ## 它答不了的
//!
//! 这是**计数模型不是实现**（文件操作 0 处）；祖先过滤按最朴素的「沿链走」建模
//! ⇒ ① 那一侧是**上界**不是它的最好形态；不含崩溃语义；
//! 不建模删快照的 O(N) 反向索引查找（两条臂共有，E26 已量）。

use e7_index_bench::Emitter;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// ①-低位：快照 ID 进 key **低位**（D6 已定项 1 逐字，bcachefs 形态）。
    KeySnapshotSuffix,
    /// ①-前缀：快照 ID 进 key **前缀**（D8 已定项 3 的布局规则逐字：
    /// 「同一快照的 key 带同一 snapshot 前缀 → 删快照有机会变成范围操作」）。
    /// ⚠️ **本臂是开跑后、写任何结论之前补的**：现查发现仓里两处各说一种位置，
    /// 而位置决定删除是全扫还是范围操作。**判据一个字没改。**
    KeySnapshotPrefix,
    /// ②：每头一棵自己的树。
    TreePerHead,
}
impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::KeySnapshotSuffix => "key_snapshot_suffix",
            Arm::KeySnapshotPrefix => "key_snapshot_prefix",
            Arm::TreePerHead => "tree_per_head",
        }
    }
}

/// key 的字节数：三段各 8 字节；① 多一维快照 4 字节。
fn key_bytes(arm: Arm) -> u64 {
    match arm {
        Arm::KeySnapshotSuffix | Arm::KeySnapshotPrefix => 8 * 3 + 4,
        Arm::TreePerHead => 8 * 3,
    }
}

/// 一个节点装几条 key。
fn fanout(node_bytes: u64, arm: Arm) -> u64 {
    node_bytes / key_bytes(arm)
}

/// **指标 1**：删一个头要触及多少条 key。
/// ① 快照 ID 在 key 低位 ⇒ 同一个头的 key **不连续** ⇒ 要扫全树的每一条（H × N）。
/// ② 整棵树丢弃 ⇒ 触及的是**节点**不是 key：`ceil(N / fanout)`。
fn delete_head_touched(arm: Arm, heads: u64, keys_per_head: u64, node_bytes: u64) -> u64 {
    match arm {
        // 低位：同一个头的 key **不连续** ⇒ 要扫全树的每一条
        Arm::KeySnapshotSuffix => heads * keys_per_head,
        // 前缀：同一个头的 key 连续 ⇒ 范围操作，触及的是节点
        Arm::KeySnapshotPrefix => keys_per_head.div_ceil(fanout(node_bytes, arm)),
        Arm::TreePerHead => keys_per_head.div_ceil(fanout(node_bytes, arm)),
    }
}

/// **指标 2b**：一次点查要走几次树下降。
/// 低位：一次下降，祖先过滤在同一次里做完。
/// **前缀：每个祖先快照各是一段独立的 key 空间 ⇒ 最坏要走 H 次下降。**
/// 每头一棵树：一次（只查本头的树）。
fn lookup_descents(arm: Arm, heads: u64) -> u64 {
    match arm {
        Arm::KeySnapshotSuffix => 1,
        Arm::KeySnapshotPrefix => heads,
        Arm::TreePerHead => 1,
    }
}

/// **指标 2**：一次点查比多少个 key 分量。
/// ① 四段 + 沿快照树走祖先链（深度 = 头数 − 1，最朴素的形态）。② 三段。
fn lookup_components(arm: Arm, heads: u64) -> u64 {
    match arm {
        Arm::KeySnapshotSuffix => 4 + heads.saturating_sub(1),
        Arm::KeySnapshotPrefix => 4,
        Arm::TreePerHead => 3,
    }
}

/// **指标 3**：旁表条目数。
/// ② 每头一棵树 ⇒ 跨头共享的块出现**第二个活引用**，破 D5 前提 3 ⇒ 要引用计数旁表。
/// ① 共享靠 birth txg + 祖先关系判定，**无旁表**。
fn sidetable_entries(arm: Arm, heads: u64, keys_per_head: u64, shared_permille: u64) -> u64 {
    match arm {
        Arm::KeySnapshotSuffix | Arm::KeySnapshotPrefix => 0,
        Arm::TreePerHead => {
            if heads <= 1 {
                0
            } else {
                keys_per_head * shared_permille / 1000
            }
        }
    }
}

/// **指标 4**：全部 key 的总字节。
fn total_key_bytes(arm: Arm, heads: u64, keys_per_head: u64) -> u64 {
    heads * keys_per_head * key_bytes(arm)
}

/// **指标 5**：固定元数据条数（树根 + 记账 + deadlist）。
/// ② 每头各一份；① 全局一份。
fn fixed_metadata(arm: Arm, heads: u64) -> u64 {
    match arm {
        Arm::KeySnapshotSuffix | Arm::KeySnapshotPrefix => 3,
        Arm::TreePerHead => 3 * heads,
    }
}

const HEADS: [u64; 4] = [1, 2, 4, 8];
const KEYS_PER_HEAD: [u64; 3] = [1_000, 10_000, 100_000];
const SHARED_PERMILLE: [u64; 3] = [0, 100, 500];
const NODE_BYTES: [u64; 2] = [4096, 16384];

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw("name=config key_bytes_snapshot=28 key_bytes_perhead=24 metadata_per_tree=3")
    );

    let arms = [Arm::KeySnapshotSuffix, Arm::KeySnapshotPrefix, Arm::TreePerHead];
    let mut max_ratio_touched = 0.0f64;
    let mut max_ratio_lookup = 0.0f64;

    for heads in HEADS {
        for keys in KEYS_PER_HEAD {
            for node in NODE_BYTES {
                for sp in SHARED_PERMILLE {
                    let mut touched = [0u64; 3];
                    for (i, arm) in arms.into_iter().enumerate() {
                        touched[i] = delete_head_touched(arm, heads, keys, node);
                        println!(
                            "{}",
                            em.emit_raw(&format!(
                                "name=metrics arm={} heads={heads} keys_per_head={keys} \
                                 node_bytes={node} shared_permille={sp} \
                                 delete_touched={} lookup_components={} lookup_descents={} \
                                 sidetable={} key_bytes_total={} fixed_metadata={}",
                                arm.name(),
                                touched[i],
                                lookup_components(arm, heads),
                                lookup_descents(arm, heads),
                                sidetable_entries(arm, heads, keys, sp),
                                total_key_bytes(arm, heads, keys),
                                fixed_metadata(arm, heads),
                            ))
                        );
                    }
                    let r = touched[0] as f64 / touched[2].max(1) as f64;
                    if r > max_ratio_touched {
                        max_ratio_touched = r;
                    }
                }
            }
        }
        let rl = lookup_components(Arm::KeySnapshotSuffix, heads) as f64
            / lookup_components(Arm::TreePerHead, heads) as f64;
        if rl > max_ratio_lookup {
            max_ratio_lookup = rl;
        }
    }

    println!(
        "{}",
        em.emit_raw(&format!(
            "name=separation max_ratio_delete_touched={max_ratio_touched:.1} \
             max_ratio_lookup={max_ratio_lookup:.4} separable={}",
            u8::from(max_ratio_touched >= 2.0 || max_ratio_lookup >= 2.0),
        ))
    );

    // ── 阳性对照 A：头数翻倍 ⇒ ① 翻倍、② 不变 ──
    let a1 = delete_head_touched(Arm::KeySnapshotSuffix, 2, 1000, 4096);
    let a2 = delete_head_touched(Arm::KeySnapshotSuffix, 4, 1000, 4096);
    let b1 = delete_head_touched(Arm::TreePerHead, 2, 1000, 4096);
    let b2 = delete_head_touched(Arm::TreePerHead, 4, 1000, 4096);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_a snapshot_h2={a1} snapshot_h4={a2} doubled={} \
             perhead_h2={b1} perhead_h4={b2} unchanged={}",
            u8::from(a2 == 2 * a1),
            u8::from(b1 == b2),
        ))
    );
    // ── 阳性对照 B：单头 + 零共享 ⇒ 旁表与固定元数据相同 ──
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=poscontrol_b sidetable_equal={} metadata_equal={}",
            u8::from(
                sidetable_entries(Arm::KeySnapshotSuffix, 1, 1000, 0)
                    == sidetable_entries(Arm::TreePerHead, 1, 1000, 0)
            ),
            u8::from(fixed_metadata(Arm::KeySnapshotSuffix, 1) == fixed_metadata(Arm::TreePerHead, 1)),
        ))
    );
    // ── 阴性对照：共享率 0 ⇒ 旁表 0 ──
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=negcontrol_zero_sharing perhead={} expect=0",
            sidetable_entries(Arm::TreePerHead, 8, 100_000, 0)
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **key 字节与扇出的绝对值**：① 28 字节 ⇒ 4096 节点装 146 条；② 24 字节 ⇒ 装 170 条。
    #[test]
    fn key_bytes_and_fanout_absolute() {
        assert_eq!(key_bytes(Arm::KeySnapshotSuffix), 28);
        assert_eq!(key_bytes(Arm::TreePerHead), 24);
        assert_eq!(fanout(4096, Arm::KeySnapshotSuffix), 146);
        assert_eq!(fanout(4096, Arm::TreePerHead), 170);
        assert_eq!(fanout(16384, Arm::TreePerHead), 682);
    }

    /// **指标 1 的绝对值**：4 头 × 每头 1000 条、4 KiB 节点。
    /// ① 要扫全树 = 4 × 1000 = **4000**；② 丢自己那棵树 = ceil(1000/170) = **6**。
    /// **比值 666.7**——这就是本实验的判决性数字。
    #[test]
    fn delete_touched_absolute_and_ratio() {
        assert_eq!(delete_head_touched(Arm::KeySnapshotSuffix, 4, 1000, 4096), 4000);
        assert_eq!(delete_head_touched(Arm::TreePerHead, 4, 1000, 4096), 6);
        let r = 4000f64 / 6f64;
        assert!((r - 666.666).abs() < 0.01);
        // 16 KiB 节点：ceil(1000/682) = 2
        assert_eq!(delete_head_touched(Arm::TreePerHead, 4, 1000, 16384), 2);
        // 十万条 / 8 头：① 800000；② ceil(100000/170) = 589
        assert_eq!(delete_head_touched(Arm::KeySnapshotSuffix, 8, 100_000, 4096), 800_000);
        assert_eq!(delete_head_touched(Arm::TreePerHead, 8, 100_000, 4096), 589);
    }

    /// **指标 2 的绝对值**：① 4 + (H−1)；② 恒 3。H=8 时 11 vs 3。
    #[test]
    fn lookup_components_absolute() {
        assert_eq!(lookup_components(Arm::KeySnapshotSuffix, 1), 4);
        assert_eq!(lookup_components(Arm::KeySnapshotSuffix, 2), 5);
        assert_eq!(lookup_components(Arm::KeySnapshotSuffix, 8), 11);
        for h in HEADS {
            assert_eq!(lookup_components(Arm::TreePerHead, h), 3);
        }
        // H=8 时比值 3.667 —— 超过「分得开」那条 2 倍线
        assert!((11f64 / 3f64 - 3.6667).abs() < 1e-3);
    }

    /// **指标 3 的绝对值**：② 在 8 头、每头 10 万条、共享率 500‰ 下是 **50 000** 条旁表；① 恒 0。
    #[test]
    fn sidetable_absolute() {
        assert_eq!(sidetable_entries(Arm::TreePerHead, 8, 100_000, 500), 50_000);
        assert_eq!(sidetable_entries(Arm::TreePerHead, 8, 100_000, 100), 10_000);
        assert_eq!(sidetable_entries(Arm::KeySnapshotSuffix, 8, 100_000, 500), 0);
        // 单头 ⇒ 没有跨头共享
        assert_eq!(sidetable_entries(Arm::TreePerHead, 1, 100_000, 500), 0);
    }

    /// **指标 4 与 5 的绝对值**：8 头 × 1000 条 ⇒ ① 224 000 字节、② 192 000（差 16.7%）；
    /// 固定元数据 ① 恒 3、② 8 头时 24。
    #[test]
    fn bytes_and_metadata_absolute() {
        assert_eq!(total_key_bytes(Arm::KeySnapshotSuffix, 8, 1000), 224_000);
        assert_eq!(total_key_bytes(Arm::TreePerHead, 8, 1000), 192_000);
        assert!((224_000f64 / 192_000f64 - 1.1667).abs() < 1e-3);
        assert_eq!(fixed_metadata(Arm::KeySnapshotSuffix, 8), 3);
        assert_eq!(fixed_metadata(Arm::TreePerHead, 8), 24);
    }

    /// **阳性对照 A**：头数翻倍 ⇒ ① 翻倍、② 不变。
    #[test]
    fn positive_control_a_doubling_heads() {
        for keys in KEYS_PER_HEAD {
            for node in NODE_BYTES {
                let a1 = delete_head_touched(Arm::KeySnapshotSuffix, 2, keys, node);
                let a2 = delete_head_touched(Arm::KeySnapshotSuffix, 4, keys, node);
                assert_eq!(a2, 2 * a1, "① 必须翻倍");
                let b1 = delete_head_touched(Arm::TreePerHead, 2, keys, node);
                let b2 = delete_head_touched(Arm::TreePerHead, 4, keys, node);
                assert_eq!(b1, b2, "② 必须不变");
            }
        }
    }

    /// **阳性对照 B**：单头 + 零共享 ⇒ 旁表与固定元数据相同。
    #[test]
    fn positive_control_b_single_head_degenerates() {
        assert_eq!(
            sidetable_entries(Arm::KeySnapshotSuffix, 1, 1000, 0),
            sidetable_entries(Arm::TreePerHead, 1, 1000, 0)
        );
        assert_eq!(fixed_metadata(Arm::KeySnapshotSuffix, 1), fixed_metadata(Arm::TreePerHead, 1));
    }

    /// **阴性对照**：共享率 0 ⇒ 旁表恒 0，两条臂都是。
    #[test]
    fn negative_control_zero_sharing_has_no_sidetable() {
        for h in HEADS {
            for k in KEYS_PER_HEAD {
                assert_eq!(sidetable_entries(Arm::TreePerHead, h, k, 0), 0);
                assert_eq!(sidetable_entries(Arm::KeySnapshotSuffix, h, k, 0), 0);
            }
        }
    }

    /// **前缀那条臂的绝对值**（开跑后补的第三条臂）：删头塌到与「每头一棵树」同量级，
    /// 而点查要走 **H 次下降**——两项对调。
    #[test]
    fn the_prefix_arm_trades_delete_cost_for_read_cost() {
        // 删头：前缀 = ceil(1000/146) = 7（key 28 字节，4 KiB 节点）；低位 = 4000
        assert_eq!(delete_head_touched(Arm::KeySnapshotPrefix, 4, 1000, 4096), 7);
        assert_eq!(delete_head_touched(Arm::KeySnapshotSuffix, 4, 1000, 4096), 4000);
        // 点查下降次数：前缀 = 头数；另外两条恒 1
        for h in HEADS {
            assert_eq!(lookup_descents(Arm::KeySnapshotPrefix, h), h);
            assert_eq!(lookup_descents(Arm::KeySnapshotSuffix, h), 1);
            assert_eq!(lookup_descents(Arm::TreePerHead, h), 1);
        }
        // 前缀的分量数不随头数涨（过滤挪到了下降次数上）
        for h in HEADS {
            assert_eq!(lookup_components(Arm::KeySnapshotPrefix, h), 4);
        }
        // 前缀仍然不需要旁表——共享靠祖先关系判定
        assert_eq!(sidetable_entries(Arm::KeySnapshotPrefix, 8, 100_000, 500), 0);
    }

    /// **「分得开」这条判据本身**：2 倍线两侧各取一点。
    #[test]
    fn the_separation_threshold_is_two_x() {
        let sep = |r: f64| r >= 2.0;
        assert!(!sep(1.9999));
        assert!(sep(2.0));
        // 实际最大比值远超它
        assert!(sep(4000f64 / 6f64));
        assert!(sep(11f64 / 3f64));
        // 而字节那一项**分不开**（1.167 < 2）——不许拿它当判决
        assert!(!sep(224_000f64 / 192_000f64));
    }
}
