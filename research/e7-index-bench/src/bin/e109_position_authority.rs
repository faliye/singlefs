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
const DATA_UNIT: u64 = 32768;
/// 索引节点 16384（D8 已定项 2）。
const NODE: u64 = 16384;
/// 扇出 119（D11 已定项 2，ε = 0.65 ⇒ 扇出 119、缓冲 665 条）。
const FANOUT: u64 = 119;
/// 缓冲 665 条（同上，只用来做交叉校验断言）。
const BUFFER_MSGS: u64 = 665;
/// 位置条目 14 字节 = dev 4 + 16 KiB 槽号 6 + 校验和 4（D19 已定项 4）。
const LOC_ENTRY: u64 = 14;
/// journal 记录 4 KiB（D23 已定项 12）。
const JOURNAL_REC: u64 = 4096;
/// 根槽 512。
const ROOT_SLOT: u64 = 512;
/// D25 主负载：一次 fsync 带 8 叶、落在 1 条共享脊柱上。
const MAIN_LEAVES: u64 = 8;
/// 那条共享脊柱上被 COW 的索引节点数（与 E107 同口径）。
const SPINE_NODES: u64 = 4;
/// 映射树节点头，与容器索引同口径（E106 / E107）。
const MAP_HDR: u64 = 76;
/// 副本宽度下界，D2 已定项 6 硬下界。
const W_MIN: u64 = 2;
/// 逻辑身份五元组 33 字节（D18 已定项 3；已定项 7 的 `105 = 42 + 33 + 8 + 8 + 10 + 4` 与它一致）。
const KEY_BYTES: u64 = 33;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Ptr,
    Map,
    Hybrid,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Ptr => "ptr",
            Arm::Map => "map",
            Arm::Hybrid => "hybrid",
        }
    }
}

/// 装得下 `n` 个条目、扇出 `f` 的树有几层（`n <= 1` 时 1 层）。
fn height(n: u64, f: u64) -> u64 {
    let mut h = 1u64;
    let mut cap = f;
    while cap < n {
        cap = cap.saturating_mul(f);
        h += 1;
    }
    h
}

/// 映射树一个节点装多少条目：key 宽 `k`，value 是 `w` 份位置条目。
fn map_fanout(key_bytes: u64, w: u64) -> u64 {
    (NODE - MAP_HDR) / (key_bytes + LOC_ENTRY * w)
}

/// 一棵引用树的高：`leaves` 个叶、扇出 119。
fn ref_height(leaves: u64) -> u64 {
    height(leaves, FANOUT)
}

/// 搬**一个**单元要写多少字节。`k` = 引用它的树数（含活头）。
fn move_bytes(arm: Arm, k: u64, leaves_per_tree: u64, total_units: u64, key_bytes: u64, w: u64) -> u64 {
    match arm {
        // 每棵引用树 COW 一条从叶到根的路径；根记录本身另算一个槽。
        Arm::Ptr => k * (ref_height(leaves_per_tree) * NODE + ROOT_SLOT),
        // 只改一条映射条目 ⇒ 映射树 COW 一条路径 + 它自己的一个根槽。与 k 无关。
        Arm::Map | Arm::Hybrid => {
            height(total_units, map_fanout(key_bytes, w)) * NODE + ROOT_SLOT
        }
    }
}

/// 一次发布要写多少字节（`n` 个新叶）。
fn publish_bytes(arm: Arm, n: u64, total_units: u64, key_bytes: u64, w: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let base = n * DATA_UNIT + SPINE_NODES * NODE + JOURNAL_REC + ROOT_SLOT;
    match arm {
        Arm::Ptr => base,
        // 中央映射要为每个新叶插一条条目；一次发布的 n 条落在映射树的少数几条路径上，
        // 攒批之后按「一条路径 + 叶层多占的节点数」算：⌈n / 映射扇出⌉ 个叶节点 + 一条内部路径。
        Arm::Map | Arm::Hybrid => {
            let f = map_fanout(key_bytes, w);
            let leaf_nodes = n.div_ceil(f).max(1);
            let inner = height(total_units, f).saturating_sub(1);
            base + (leaf_nodes + inner) * NODE + ROOT_SLOT
        }
    }
}

/// 读一个单元要读几个节点。`stale` 是提示过期率（只对 hybrid 有意义），千分数。
fn read_nodes(arm: Arm, leaves_per_tree: u64, total_units: u64, key_bytes: u64, w: u64, stale_permille: u64) -> u64 {
    let refh = ref_height(leaves_per_tree);
    let maph = height(total_units, map_fanout(key_bytes, w));
    match arm {
        Arm::Ptr => refh,
        Arm::Map => refh + maph,
        // 千分之 stale 的读要再走一趟映射；这里报的是**千次读**的节点数，避免整数除法把它抹成 0。
        Arm::Hybrid => refh * 1000 + maph * stale_permille,
    }
}

fn main() {
    let mut em = Emitter::new();
    let mut out = String::new();
    let arms = [Arm::Ptr, Arm::Map, Arm::Hybrid];
    // 16 TiB / 32768 的数据单元数量级，与 E106 / E107 同口径。
    let total_units: u64 = 483_183_820;
    let leaves_per_tree: u64 = 4_000_000;

    // ── 阴性对照（作废条款 1）：零操作 ⇒ 三臂全 0 ──────────────────────
    let mut neg_ok = true;
    for a in arms {
        if publish_bytes(a, 0, total_units, KEY_BYTES, W_MIN) != 0 {
            neg_ok = false;
        }
    }
    out.push_str(&em.emit_raw(&format!(
        "name=negative_control publish_leaves=0 verdict={}",
        if neg_ok { "pass" } else { "VOID" }
    )));
    out.push('\n');

    out.push_str(&em.emit_raw(&format!(
        "name=geometry fanout={FANOUT} buffer_msgs={BUFFER_MSGS} loc_entry={LOC_ENTRY} \
         ref_height={} total_units={total_units}",
        ref_height(leaves_per_tree)
    )));
    out.push('\n');

    // ── 判别力：两棵树不同高的取样点，map 臂必须跟着动而 ptr 臂不动 ───────
    // 引用树扇出 119：叶 1e5 ⇒ 高 3、4e6 ⇒ 高 4、3e8 ⇒ 高 5。
    // 映射树扇出 267（key 33 + 位置条目 14 × 2）：池 1e7 ⇒ 高 3、4.83e8 ⇒ 高 4、1e10 ⇒ 高 5。
    for &(leaves, pool) in &[
        (100_000u64, 10_000_000u64),
        (100_000, 483_183_820),
        (100_000, 10_000_000_000),
        (4_000_000, 10_000_000),
        (4_000_000, 483_183_820),
        (300_000_000, 10_000_000),
    ] {
        let mp = move_bytes(Arm::Map, 1, leaves, pool, KEY_BYTES, W_MIN);
        let pp = move_bytes(Arm::Ptr, 1, leaves, pool, KEY_BYTES, W_MIN);
        out.push_str(&em.emit_raw(&format!(
            "name=discriminating leaves={leaves} pool={pool} ref_height={} map_height={} \
             ptr_bytes={pp} map_bytes={mp} same_height={}",
            ref_height(leaves),
            height(pool, map_fanout(KEY_BYTES, W_MIN)),
            ref_height(leaves) == height(pool, map_fanout(KEY_BYTES, W_MIN))
        )));
        out.push('\n');
    }

    // ── 搬迁：扫 K 与 key 宽度 ───────────────────────────────────────────
    for &key_bytes in &[16u64, 24, KEY_BYTES, 40] {
        for &k in &[1u64, 2, 4, 8, 16] {
            let p = move_bytes(Arm::Ptr, k, leaves_per_tree, total_units, key_bytes, W_MIN);
            let m = move_bytes(Arm::Map, k, leaves_per_tree, total_units, key_bytes, W_MIN);
            out.push_str(&em.emit_raw(&format!(
                "name=move key_bytes={key_bytes} k={k} map_fanout={} map_height={} \
                 ptr_bytes={p} map_bytes={m} ratio_ptr_over_map={:.4}",
                map_fanout(key_bytes, W_MIN),
                height(total_units, map_fanout(key_bytes, W_MIN)),
                p as f64 / m as f64
            )));
            out.push('\n');
        }
    }

    // ── 发布：每次发布中央映射多付多少 ───────────────────────────────────
    for &key_bytes in &[16u64, 24, KEY_BYTES, 40] {
        for &n in &[1u64, MAIN_LEAVES, 64] {
            let p = publish_bytes(Arm::Ptr, n, total_units, key_bytes, W_MIN);
            let m = publish_bytes(Arm::Map, n, total_units, key_bytes, W_MIN);
            out.push_str(&em.emit_raw(&format!(
                "name=publish key_bytes={key_bytes} leaves={n} ptr_bytes={p} map_bytes={m} \
                 extra_bytes={} ratio_map_over_ptr={:.4}",
                m - p,
                m as f64 / p as f64
            )));
            out.push('\n');
        }
    }

    // ── 交叉点：每次发布搬多少个单元，map 臂才回本 ───────────────────────
    for &key_bytes in &[16u64, 24, KEY_BYTES, 40] {
        for &k in &[1u64, 2, 4, 8, 16] {
            let extra = publish_bytes(Arm::Map, MAIN_LEAVES, total_units, key_bytes, W_MIN)
                - publish_bytes(Arm::Ptr, MAIN_LEAVES, total_units, key_bytes, W_MIN);
            let saved = move_bytes(Arm::Ptr, k, leaves_per_tree, total_units, key_bytes, W_MIN)
                .saturating_sub(move_bytes(Arm::Map, k, leaves_per_tree, total_units, key_bytes, W_MIN));
            let cross = if saved == 0 { -1i64 } else { extra.div_ceil(saved) as i64 };
            out.push_str(&em.emit_raw(&format!(
                "name=crossover key_bytes={key_bytes} k={k} publish_extra={extra} \
                 saved_per_move={saved} moves_per_publish_to_break_even={cross}"
            )));
            out.push('\n');
        }
    }

    // ── 读路径：hybrid 按提示过期率扫（E96 量到轮末 19–37%）───────────────
    for &stale in &[0u64, 190, 370, 1000] {
        let p = read_nodes(Arm::Ptr, leaves_per_tree, total_units, KEY_BYTES, W_MIN, stale) * 1000;
        let m = read_nodes(Arm::Map, leaves_per_tree, total_units, KEY_BYTES, W_MIN, stale) * 1000;
        let h = read_nodes(Arm::Hybrid, leaves_per_tree, total_units, KEY_BYTES, W_MIN, stale);
        out.push_str(&em.emit_raw(&format!(
            "name=read stale_permille={stale} nodes_per_1000_reads_ptr={p} \
             nodes_per_1000_reads_map={m} nodes_per_1000_reads_hybrid={h}"
        )));
        out.push('\n');
    }

    // ── 阳性对照：必须**调用 map 臂本身**，而不是手写一个同形的表达式 ──────
    // 第一版这里写成 `height(LEAVES, FANOUT) * NODE + ROOT_SLOT == move_bytes(Ptr, 1, ..)`，
    // 两边是同一个表达式、一次都没碰 `Arm::Map` ⇒ `X == X`，一点判别力都没有。
    // 现在改成：换一个映射树几何，`map` 臂必须跟着变，而 `ptr` 臂必须纹丝不动。
    let pool_h3 = 10_000_000u64;
    let pool_h5 = 10_000_000_000u64;
    let map_h3 = move_bytes(Arm::Map, 1, leaves_per_tree, pool_h3, KEY_BYTES, W_MIN);
    let map_h5 = move_bytes(Arm::Map, 1, leaves_per_tree, pool_h5, KEY_BYTES, W_MIN);
    let ptr_h3 = move_bytes(Arm::Ptr, 1, leaves_per_tree, pool_h3, KEY_BYTES, W_MIN);
    let ptr_h5 = move_bytes(Arm::Ptr, 1, leaves_per_tree, pool_h5, KEY_BYTES, W_MIN);
    out.push_str(&em.emit_raw(&format!(
        "name=positive_control map_at_pool_h3={map_h3} map_at_pool_h5={map_h5} \
         ptr_at_pool_h3={ptr_h3} ptr_at_pool_h5={ptr_h5} verdict={}",
        if map_h3 != map_h5 && ptr_h3 == ptr_h5 {
            "pass"
        } else {
            "VOID"
        }
    )));
    out.push('\n');

    out.push_str(&em.emit(
        "main_workload_publish_ptr",
        &Sample {
            ops: 1,
            bytes_per_op: publish_bytes(Arm::Ptr, MAIN_LEAVES, total_units, KEY_BYTES, W_MIN),
            elapsed_ns: 1,
        },
    ));
    out.push('\n');
    out.push_str(&em.finish());
    println!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;
    const TOTAL: u64 = 483_183_820;
    const LEAVES: u64 = 4_000_000;
    /// 映射树扇出 267 之下高分别为 3 / 5 的两个池规模，用来证明 `map` 臂真的在看映射几何。
    const POOL_H3: u64 = 10_000_000;
    const POOL_H5: u64 = 10_000_000_000;

    /// 几何常量交叉校验：扇出与缓冲抄 D11 已定项 2，位置条目抄 D19 已定项 4。
    /// 写成加法不写减法（test-discipline：减法会让变异编译期溢出、被记成无效变异）。
    #[test]
    fn geometry_constants_match_settled_clauses() {
        assert_eq!(FANOUT, 119);
        assert_eq!(BUFFER_MSGS, 665);
        assert_eq!(LOC_ENTRY, 4 + 6 + 4);
        assert_eq!(DATA_UNIT, 32768);
        assert_eq!(NODE, 16384);
    }

    /// 引用树的高：400 万叶、扇出 119 ⇒ 119^3 = 1 685 159 < 4e6 ≤ 119^4，恰为 4。
    #[test]
    fn reference_tree_height_is_pinned() {
        assert_eq!(FANOUT.pow(3), 1_685_159);
        assert!(FANOUT.pow(3) < LEAVES);
        assert!(LEAVES <= FANOUT.pow(4));
        assert_eq!(ref_height(LEAVES), 4);
    }

    /// 映射树扇出：key 33（D18 已定项 3 的五元组宽度）、w = 2 ⇒ 每条 33 + 28 = 61 字节，(16384 − 76) / 61 = 267。
    #[test]
    fn map_fanout_is_pinned() {
        assert_eq!(KEY_BYTES, 33);
        assert_eq!(KEY_BYTES + LOC_ENTRY * 2, 61);
        assert_eq!(map_fanout(KEY_BYTES, 2), 267);
        assert_eq!(height(TOTAL, 267), 4);
    }

    /// **这个实验最要紧的那条结构结论**：ptr 臂的搬迁字节随 k 线性，map 臂与 k 无关。
    /// 只写「随 k 涨」不够（三条臂一起错时互比仍成立），所以把两端的绝对值都钉住。
    #[test]
    fn move_cost_is_linear_in_k_for_ptr_and_flat_for_map() {
        let one = move_bytes(Arm::Ptr, 1, LEAVES, TOTAL, KEY_BYTES, 2);
        assert_eq!(one, 4 * 16384 + 512);
        assert_eq!(one, 66048);
        for k in [1u64, 2, 4, 8, 16] {
            assert_eq!(move_bytes(Arm::Ptr, k, LEAVES, TOTAL, KEY_BYTES, 2), k * 66048);
            // map 臂：映射树高 4 ⇒ 与 ptr 的 k = 1 恰好同值，且不随 k 动。
            assert_eq!(move_bytes(Arm::Map, k, LEAVES, TOTAL, KEY_BYTES, 2), 66048);
        }
    }

    /// 一次发布中央映射多付多少：8 叶、key 32 ⇒ 叶节点 1 + 内部 3 = 4 个节点，加一个根槽。
    #[test]
    fn publish_extra_for_map_is_pinned() {
        let p = publish_bytes(Arm::Ptr, MAIN_LEAVES, TOTAL, KEY_BYTES, 2);
        let m = publish_bytes(Arm::Map, MAIN_LEAVES, TOTAL, KEY_BYTES, 2);
        assert_eq!(p, 332288); // 与 E107 的基线逐字节相同，两个实验同尺
        assert_eq!(m, p + 4 * 16384 + 512);
        assert_eq!(m, 398336);
    }

    /// 交叉点：主负载下每次发布搬几个单元，map 臂才回本。
    /// k = 1 时 map 一分不省 ⇒ 永远回不了本，报 −1。
    #[test]
    fn breakeven_moves_per_publish_are_pinned() {
        let extra = publish_bytes(Arm::Map, MAIN_LEAVES, TOTAL, KEY_BYTES, 2)
            - publish_bytes(Arm::Ptr, MAIN_LEAVES, TOTAL, KEY_BYTES, 2);
        assert_eq!(extra, 66048);
        let saved_k1 = move_bytes(Arm::Ptr, 1, LEAVES, TOTAL, KEY_BYTES, 2)
            - move_bytes(Arm::Map, 1, LEAVES, TOTAL, KEY_BYTES, 2);
        assert_eq!(saved_k1, 0);
        let saved_k2 = move_bytes(Arm::Ptr, 2, LEAVES, TOTAL, KEY_BYTES, 2)
            - move_bytes(Arm::Map, 2, LEAVES, TOTAL, KEY_BYTES, 2);
        assert_eq!(saved_k2, 66048);
        assert_eq!(extra.div_ceil(saved_k2), 1);
        let saved_k16 = move_bytes(Arm::Ptr, 16, LEAVES, TOTAL, KEY_BYTES, 2)
            - move_bytes(Arm::Map, 16, LEAVES, TOTAL, KEY_BYTES, 2);
        assert_eq!(saved_k16, 15 * 66048);
        assert_eq!(extra.div_ceil(saved_k16), 1);
    }

    /// **判别力自证（作废条款 3）**：提示过期率为 0 时 hybrid 必须与 ptr 逐格相等，
    /// 为 1000‰ 时必须与 map 逐格相等。这条不成立，说明 hybrid 根本没建成一条中间臂。
    #[test]
    fn hybrid_collapses_to_ptr_at_zero_stale_and_to_map_at_full_stale() {
        let p = read_nodes(Arm::Ptr, LEAVES, TOTAL, 32, 2, 0) * 1000;
        let m = read_nodes(Arm::Map, LEAVES, TOTAL, 32, 2, 0) * 1000;
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES, TOTAL, 32, 2, 0), p);
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES, TOTAL, 32, 2, 1000), m);
        // 中间值必须严格落在两端之间，否则那条曲线是平的、这一维没有信息
        let mid = read_nodes(Arm::Hybrid, LEAVES, TOTAL, 32, 2, 370);
        assert!(p < mid && mid < m, "p={p} mid={mid} m={m}");
    }

    /// 读路径的绝对值：引用树高 4、映射树高 4 ⇒ 千次读 ptr 4000、map 8000。
    #[test]
    fn read_nodes_absolute_values_are_pinned() {
        assert_eq!(read_nodes(Arm::Ptr, LEAVES, TOTAL, 32, 2, 0) * 1000, 4000);
        assert_eq!(read_nodes(Arm::Map, LEAVES, TOTAL, 32, 2, 0) * 1000, 8000);
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES, TOTAL, 32, 2, 370), 4000 + 4 * 370);
        assert_eq!(read_nodes(Arm::Hybrid, LEAVES, TOTAL, 32, 2, 370), 5480);
    }

    /// 作废条款 3 的另一半：key 宽度四档上结论方向必须一致。
    #[test]
    fn direction_is_stable_across_key_widths() {
        for kb in [16u64, 24, 32, 40] {
            // 搬迁：map 恒不随 k 涨，ptr 恒随 k 涨
            let a = move_bytes(Arm::Map, 1, LEAVES, TOTAL, kb, 2);
            let b = move_bytes(Arm::Map, 16, LEAVES, TOTAL, kb, 2);
            assert_eq!(a, b, "key_bytes={kb}");
            assert!(
                move_bytes(Arm::Ptr, 16, LEAVES, TOTAL, kb, 2)
                    > move_bytes(Arm::Ptr, 1, LEAVES, TOTAL, kb, 2)
            );
            // 发布：map 恒比 ptr 贵
            assert!(
                publish_bytes(Arm::Map, MAIN_LEAVES, TOTAL, kb, 2)
                    > publish_bytes(Arm::Ptr, MAIN_LEAVES, TOTAL, kb, 2),
                "key_bytes={kb}"
            );
        }
    }

    /// 阴性对照（作废条款 1）。
    #[test]
    fn zero_publish_is_zero_on_all_arms() {
        for a in [Arm::Ptr, Arm::Map, Arm::Hybrid] {
            assert_eq!(publish_bytes(a, 0, TOTAL, KEY_BYTES, 2), 0, "{}", a.name());
        }
    }

    /// **阳性对照（2026-09-06 重写）**：换一个映射树几何，`map` 臂必须跟着变，
    /// 而 `ptr` 臂必须纹丝不动。
    ///
    /// ⚠️ **第一版这条是 `X == X`**：写成
    /// `height(LEAVES, FANOUT) * NODE + ROOT_SLOT == move_bytes(Ptr, 1, ..)`，
    /// 两边是同一个表达式，**一次都没调用 `Arm::Map`** ⇒ 把映射树的几何整个删掉、
    /// 换成引用树的高，11 个单测全绿、产物逐字节不变（2026-09-06 反推攻击腿实跑证明）。
    #[test]
    fn positive_control_must_actually_exercise_the_map_arm() {
        let map_h3 = move_bytes(Arm::Map, 1, LEAVES, POOL_H3, KEY_BYTES, 2);
        let map_h5 = move_bytes(Arm::Map, 1, LEAVES, POOL_H5, KEY_BYTES, 2);
        let ptr_h3 = move_bytes(Arm::Ptr, 1, LEAVES, POOL_H3, KEY_BYTES, 2);
        let ptr_h5 = move_bytes(Arm::Ptr, 1, LEAVES, POOL_H5, KEY_BYTES, 2);
        assert_eq!(height(POOL_H3, map_fanout(KEY_BYTES, 2)), 3);
        assert_eq!(height(POOL_H5, map_fanout(KEY_BYTES, 2)), 5);
        assert_eq!(map_h3, 3 * NODE + ROOT_SLOT);
        assert_eq!(map_h5, 5 * NODE + ROOT_SLOT);
        assert_ne!(map_h3, map_h5);
        assert_eq!(ptr_h3, ptr_h5);
        assert_eq!(ptr_h3, 4 * NODE + ROOT_SLOT);
    }

    /// 读路径也必须真的在看映射树的几何——主取样点上两棵树同高 4，
    /// 只在那一点上测等于没测（2026-09-06 变异 M16 抓到的第二个同型盲区）。
    #[test]
    fn read_nodes_depend_on_map_geometry() {
        assert_eq!(read_nodes(Arm::Map, LEAVES, POOL_H3, KEY_BYTES, 2, 0), 4 + 3);
        assert_eq!(read_nodes(Arm::Map, LEAVES, POOL_H5, KEY_BYTES, 2, 0), 4 + 5);
        assert_ne!(
            read_nodes(Arm::Map, LEAVES, POOL_H3, KEY_BYTES, 2, 0),
            read_nodes(Arm::Map, LEAVES, POOL_H5, KEY_BYTES, 2, 0)
        );
        // ptr 臂对池规模不敏感
        assert_eq!(
            read_nodes(Arm::Ptr, LEAVES, POOL_H3, KEY_BYTES, 2, 0),
            read_nodes(Arm::Ptr, LEAVES, POOL_H5, KEY_BYTES, 2, 0)
        );
        // hybrid 的多跳那一项也必须跟着映射树走
        assert_eq!(
            read_nodes(Arm::Hybrid, LEAVES, POOL_H5, KEY_BYTES, 2, 370),
            4 * 1000 + 5 * 370
        );
    }

    /// **主取样点是一个退化点，必须显式标出来**：那一点上 `map` 与 `ptr`（`K = 1`）
    /// 数值相同，是因为两棵树恰好都是高 4，不是因为它们本质相同。
    #[test]
    fn the_main_sampling_point_is_a_degenerate_one() {
        assert_eq!(ref_height(LEAVES), 4);
        assert_eq!(height(TOTAL, map_fanout(KEY_BYTES, 2)), 4);
        assert_eq!(
            move_bytes(Arm::Map, 1, LEAVES, TOTAL, KEY_BYTES, 2),
            move_bytes(Arm::Ptr, 1, LEAVES, TOTAL, KEY_BYTES, 2)
        );
        assert_ne!(
            move_bytes(Arm::Map, 1, LEAVES, POOL_H5, KEY_BYTES, 2),
            move_bytes(Arm::Ptr, 1, LEAVES, POOL_H5, KEY_BYTES, 2)
        );
    }
}
