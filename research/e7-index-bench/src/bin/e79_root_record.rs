//! E79：根记录的容量 —— 一个根槽（宽 = `physical_block_size`）装不装得下开放树表。
//!
//! ## 为什么要有这个实验
//!
//! 第一个事务就要写根槽，而**根记录里有什么字段，全仓没有一处定过**（2026-09-02 grep 证实：
//! 只有 I-7.2（最新根可完整遍历）一句「checkpoint 树根指针出发」提过它的内容）。
//! 三条已定条款合起来把这个空白顶成了容量问题：
//!
//! - D22（单元原子性怎么合成）已定项 2：「槽宽 = 挂载时探测的 `physical_block_size`」——
//!   本机实测这个值是 **512**（D20（承重面：单元的原子性与自包含）推论三第一问）；
//! - D15（格式冻结政策）已定项 2：「超级块树表 day-1 做成开放列表」——树的数量不封顶；
//! - D6（快照实现模型）已定项 1：「每头一棵自己的树」——可写头越多树越多。
//!
//! 512 字节的槽 × 不封顶的树表 ⇒ 装不下只是时间问题；**什么时候装不下、
//! 装不下之后间接层每次发布多付多少**，要算出来才能定根记录的形态。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D21（权威态与派生态的分界）逐字节表：「指针头部（MAC 16 + nonce 12 + 算法类型 1 +
//!   extent 偏移 2）31 + 位置条目 ×2（dev 1 + 物理偏移 6 + 密文校验和 4）22」。
//! - D19（块指针的结构与宽度预算）已定项 3：定宽，不加密时 MAC/nonce 留位不省。
//! - D23（journal 的角色与格式）已定项 9：`jsn` = 实例代号 32 位 + 计数器 48 位 = 10 字节
//!   （E78（重放的起点）判定根记录要带 jsn 水位时，就是这 10 字节）。
//! - E23（journal 几何）字段表：`header_csum` 32 字节——根记录的自证校验和按同宽计。
//! - D25（目标负载优先级）：一次 fsync = 8 叶 + 4 祖先 + 1 根槽 + 1 记录 = 14 块——
//!   间接层的每发布代价拿它当分母。
//!
//! ## 模型（纯算术，字段宽度是输入不是结论）
//!
//! 固定头：magic 4 + fsid 16 + flags 4 + 发布代号 8 + checkpoint_txg 8 + 树数 4 +
//! 自证校验和 32 = **76 字节**；带 jsn 水位（E78 出路 B）再 +10 = **86 字节**。
//! 每棵树一条：tree_id 8 + 指针头部 31 + 位置条目 22（2 副本，D2（RAID 条带策略）已定项 9
//! 第一版 2 盘 ⇒ w=2）= **61 字节**。
//!
//! 两种形态：**flat**（树表直接住根槽）与 **indirect**（根槽只放一条指向「树表单元」的指针，
//! 树表单元是一个 16 KiB 元数据单元，头 55 字节——D21 的提议值）。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 全部数字是闭式算术，必须被独立的手算锚点钉住（见单测）；对不上整轮作废。
//! 2. 报出：每档槽宽下 flat 装得下的树数上限；v1 最小树集（3 棵：记账、分配记录、数据 extent）
//!    与常识树集（7 棵）各自装不装得下；间接层每发布多付的块数与字节数（分母 = D25 的 14 块）。
//! 3. 不判「选 flat 还是 indirect」——那是决策，本实验只交数字。
//!
//! ## 它答不了的
//!
//! tree_id 的宽度仓里没定（取 8 字节是假设，变窄只会让 flat 多装一两棵，不改变量级）；
//! 树表单元自身的崩溃一致性归 E77（发布的持久顺序）的屏障结论管，这里只算字节。

use e7_index_bench::Emitter;

/// 固定头（不含 jsn 水位）：magic 4 + fsid 16 + flags 4 + 发布代号 8 + checkpoint_txg 8 +
/// 树数 4 + 自证校验和 32。
const HEAD_BASE: u64 = 4 + 16 + 4 + 8 + 8 + 4 + 32;
/// E78 出路 B 要的 jsn 水位（D23 已定项 9 的宽度）。
const WATERMARK: u64 = 10;
/// 每棵树一条：tree_id 8 + 指针头部 31 + 位置条目 11 × 2 副本。
const TREE_ENTRY: u64 = 8 + 31 + 11 * 2;
/// 树表单元取 16 KiB 元数据单元（D8 已定项 2 的节点大小），单元头 55（D21 提议值）。
const NODE_BYTES: u64 = 16384;
const UNIT_HDR: u64 = 55;
/// D25 目标负载一次 fsync 的块数（8 叶 + 4 祖先 + 1 根槽 + 1 记录）。
const FSYNC_BLOCKS: u64 = 14;

/// flat 形态：槽里装得下几棵树。
fn flat_capacity(slot: u64, with_watermark: bool) -> u64 {
    let head = HEAD_BASE + if with_watermark { WATERMARK } else { 0 };
    slot.saturating_sub(head) / TREE_ENTRY
}

/// indirect 形态：树表单元装得下几棵树（一层）。
fn table_unit_capacity() -> u64 {
    (NODE_BYTES - UNIT_HDR - 32) / TREE_ENTRY // 32 = 树表单元自己的校验和从父（根槽）拿，此处按保守再留一份长度与保留字段
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config head_base={HEAD_BASE} watermark={WATERMARK} tree_entry={TREE_ENTRY} model=arithmetic file_ops=0"
        ))
    );
    for slot in [512u64, 4096] {
        for wm in [false, true] {
            let cap = flat_capacity(slot, wm);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=flat slot={slot} watermark={} capacity_trees={cap} fits_v1_min3={} fits_v1_common7={}",
                    u8::from(wm),
                    u8::from(cap >= 3),
                    u8::from(cap >= 7)
                ))
            );
        }
    }
    // 间接层：根槽 = 固定头 + 水位 + 1 条树表指针（当一棵树计宽）；树表单元另写。
    let tbl = table_unit_capacity();
    let extra_blocks = 1u64; // 每次发布树表单元必然被 COW（任何树根变了它都变）
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=indirect table_unit_trees={tbl} extra_units_per_publish={extra_blocks} \
             publish_overhead_blocks_pct={:.1} publish_overhead_bytes={} ",
            100.0 * extra_blocks as f64 / FSYNC_BLOCKS as f64,
            NODE_BYTES
        ))
    );
    // 头有多少净余量可以留给以后的字段（512 槽、带水位、v1 常识 7 棵直住时）：
    let used = HEAD_BASE + WATERMARK + 7 * TREE_ENTRY;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=headroom slot512_used_if_7_trees={used} overflows_512={}",
            u8::from(used > 512)
        ))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：固定头 76 / 含水位 86；树条目 61。手算：4+16+4+8+8+4+32=76；8+31+22=61。
    #[test]
    fn absolute_field_arithmetic() {
        assert_eq!(HEAD_BASE, 76);
        assert_eq!(HEAD_BASE + WATERMARK, 86);
        assert_eq!(TREE_ENTRY, 61);
    }

    /// **绝对值锚点**：512 槽、带水位 ⇒ (512−86)/61 = 6 棵；不带水位 ⇒ 7 棵。
    /// 4096 槽、带水位 ⇒ (4096−86)/61 = 65 棵。
    #[test]
    fn absolute_flat_capacity() {
        assert_eq!(flat_capacity(512, true), 6);
        assert_eq!(flat_capacity(512, false), 7);
        assert_eq!(flat_capacity(4096, true), 65);
        assert_eq!(flat_capacity(4096, false), 65);
    }

    /// v1 最小树集（记账、分配记录、数据 extent = 3 棵）在 512 槽上装得下；
    /// 常识树集 7 棵在带水位的 512 槽上**装不下**——这是判决性的那一格。
    #[test]
    fn v1_min_fits_common_does_not() {
        assert!(flat_capacity(512, true) >= 3);
        assert!(flat_capacity(512, true) < 7, "带水位的 512 槽装不下 7 棵");
        assert!(flat_capacity(512, false) >= 7, "不带水位恰好装下 7 棵——水位挤掉了第 7 棵");
    }

    /// 树表单元（16 KiB）装得下 267 棵：(16384−55−32)/61 = 267。
    #[test]
    fn absolute_table_unit_capacity() {
        assert_eq!(table_unit_capacity(), 267);
    }

    /// 每头一棵树（D6）：8 个可写头 + 7 棵基础树 = 15 棵，两种 512 形态都装不下，
    /// 4096 槽装得下。头数过 58 连 4096 槽也翻——开放树表迟早逼出间接层。
    #[test]
    fn heads_push_past_flat() {
        assert!(15 > flat_capacity(512, true));
        assert!(15 <= flat_capacity(4096, true));
        assert!(58 + 7 <= flat_capacity(4096, true), "65 棵是 4096 槽的上限");
        assert!(58 + 8 > flat_capacity(4096, true));
    }

    /// 间接层的每发布代价：+1 个 16 KiB 单元 = 目标负载 14 块上的 +7.1%。
    #[test]
    fn indirect_overhead_on_target_load() {
        let pct = 100.0 * 1.0 / FSYNC_BLOCKS as f64;
        assert!((pct - 7.142857).abs() < 1e-3);
    }

    /// 容量对槽宽单调、对水位反单调——算术自检。
    #[test]
    fn monotonicity() {
        assert!(flat_capacity(4096, true) > flat_capacity(512, true));
        assert!(flat_capacity(512, false) >= flat_capacity(512, true));
    }

    /// 固定头必须装得进最窄的槽——装不进说明字段表本身写爆了。
    #[test]
    fn head_fits_smallest_slot() {
        assert!(HEAD_BASE + WATERMARK < 512);
    }
}
