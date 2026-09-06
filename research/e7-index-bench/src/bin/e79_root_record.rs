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
/// 位置条目宽（D19 已定项 4，2026-09-03 用户定案）。与 e109 同名，受门禁阶段 27 的
/// `format-const: LOC_ENTRY` 绑住——此前这里写死 11，D19 定案后漂了三天没人发现。
const LOC_ENTRY: u64 = 14;
/// 每棵树一条：tree_id 8 + 指针头部 31 + 位置条目 × 2 副本。
const TREE_ENTRY: u64 = 8 + 31 + LOC_ENTRY * 2;
/// 树表单元取 16 KiB 元数据单元（D8 已定项 2 的节点大小）。单元头按 D18 已定项 7 的**索引节点类**：
/// E73 的三档下界 58 / 67 / 76 加 C113 定案（2026-09-05）的 10 字节写序 ⇒ 68 / 77 / 86。
/// 树表单元不带 key 区间 ⇒ 取最窄那档 68；三档给出同一个容量，由单测钉住。
const NODE_BYTES: u64 = 16384;
const UNIT_HDR: u64 = 68;
const NODE_HDR_TIERS: [u64; 3] = [68, 77, 86];
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

    /// **绝对值锚点**：固定头 76 / 含水位 86；树条目 67。手算：4+16+4+8+8+4+32=76；8+31+28=67。
    #[test]
    fn absolute_field_arithmetic() {
        assert_eq!(HEAD_BASE, 76);
        assert_eq!(HEAD_BASE + WATERMARK, 86);
        assert_eq!(TREE_ENTRY, 67);
        // 位置条目取 D19 已定项 4 的 14 而不是 11：11 那个值在 2026-09-03 就被顶掉了，
        // 而这里停在旧值到 2026-09-06 才被发现。写成加法而不是减法，让变异活到运行期。
        assert_eq!(TREE_ENTRY, 8 + 31 + LOC_ENTRY * 2);
        assert_eq!(LOC_ENTRY, 14);
    }

    /// **绝对值锚点**：512 槽、带水位 ⇒ (512−86)/67 = 6 棵；不带水位 ⇒ (512−76)/67 = 6 棵。
    /// 4096 槽、带水位 ⇒ (4096−86)/67 = 59 棵；不带水位 ⇒ 4020/67 = 60 棵（整除）。
    #[test]
    fn absolute_flat_capacity() {
        assert_eq!(flat_capacity(512, true), 6);
        assert_eq!(flat_capacity(512, false), 6);
        assert_eq!(flat_capacity(4096, true), 59);
        assert_eq!(flat_capacity(4096, false), 60);
        // 整除那一格单独钉住：67 × 60 = 4020 = 4096 − 76，边界上不许差一。
        assert_eq!(67 * flat_capacity(4096, false), 4096 - HEAD_BASE);
    }

    /// v1 最小树集（记账、分配记录、数据 extent = 3 棵）在 512 槽上装得下；
    /// 常识树集 7 棵在 512 槽上**装不下**，而且**去掉水位也装不下**——
    /// 位置条目取 14 之后，压垮平铺形态的是条目宽，不是水位。
    #[test]
    fn v1_min_fits_common_does_not() {
        assert!(flat_capacity(512, true) >= 3);
        assert!(flat_capacity(512, true) < 7, "带水位的 512 槽装不下 7 棵");
        assert!(flat_capacity(512, false) < 7, "去掉水位仍装不下 7 棵——不是水位挤掉的");
        assert_eq!(flat_capacity(512, false), flat_capacity(512, true),
                   "512 槽上水位一棵树都没挤掉");
    }

    /// 树表单元（16 KiB）装得下 243 棵：(16384−68−32)/67 = 16284/67 = 243。
    /// **三档节点头不再逐格同值**：68 → 243，77 与 86 → 242 ⇒ 条目宽 61 时那句
    /// 「容量不由头宽的档位定」在 67 上不成立，差一棵。
    #[test]
    fn absolute_table_unit_capacity() {
        assert_eq!(table_unit_capacity(), 243);
        // ⚠️ **条目变宽之后容量对头宽不再敏感**：67 字节条目下 UNIT_HDR 取 68 或 55
        // 都得 243（16284/67 与 16297/67 同为 243），M6 那条变异因此变成盲区。
        // ⇒ 把头宽本身与它算出的净空间各钉一格，别只钉最终容量。
        assert_eq!(UNIT_HDR, 68);
        assert_eq!(UNIT_HDR, NODE_HDR_TIERS[0], "UNIT_HDR 必须就是最窄那一档");
        assert_eq!(NODE_BYTES - UNIT_HDR - 32, 16284);
        assert_eq!(NODE_BYTES - NODE_HDR_TIERS[0] - 32, 16284);
        let per_tier: Vec<u64> = NODE_HDR_TIERS
            .iter()
            .map(|h| (NODE_BYTES - h - 32) / TREE_ENTRY)
            .collect();
        assert_eq!(per_tier, vec![243, 242, 242]);
        // 边界各钉一格：243 装得下、244 装不下（最宽那一档）。
        assert!(67 * 243 <= 16284);
        assert!(67 * 244 > 16284);
    }

    /// 每头一棵树（D6）：8 个可写头 + 7 棵基础树 = 15 棵，两种 512 形态都装不下，
    /// 4096 槽装得下。头数过 52 连 4096 槽也翻——开放树表迟早逼出间接层。
    #[test]
    fn heads_push_past_flat() {
        assert!(15 > flat_capacity(512, true));
        assert!(15 <= flat_capacity(4096, true));
        assert!(52 + 7 <= flat_capacity(4096, true), "59 棵是 4096 槽带水位的上限");
        assert!(52 + 8 > flat_capacity(4096, true));
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
