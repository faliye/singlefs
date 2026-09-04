//! E103：inode 打包路 ① 的更新代价 —— D8 未定项 6 比较表欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D8 未定项 6**（2026-09-04 随 D18 已定项 11 补的那句）：「更新代价要进比较表（第二轮正推腿估算，
//!   未实测：改一条记录 = COW 32 KiB × w + 容器索引一条 ≈ 3 倍于住索引叶；stat 多一次查找；爆炸半径 2 倍）」。
//!   ⚠️ 那个估算**只数了叶子**：住索引叶那一侧算成 1 个 16 KiB 叶，打包那一侧算成 1 个容器 + 1 个索引叶。
//! - **D23 已定项 1 取甲，逐字**：「每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。祖先不延后。」
//!   ⇒ 更新代价必须把**树高**算进去，两条形态的祖先数不同。
//! - **D18 已定项 11**：打包记录单元头 93；容器索引是物理指针唯一持有者，别的树按身份
//!   （出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8）引用容器；更新一个容器 = COW 它 + 改容器索引一条。
//! - **D8 已定项 2**：节点 16384。**D4 已定项 7**：单元 32768。**D2 已定项 9**：第一版 2 盘恒 w = 2。
//! - **D19 已定项 4**：子指针 59。**E98**：inode 记录 140、住索引叶时叶扇出 116（头 58）、内部扇出 243。
//! - **D23 已定项 12**：journal 记录 4 KiB。**E79**：根槽 512（本机 `physical_block_size`）。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. **单次更新的绝对值**（一次发布只改一条记录）：两条形态各自的「写出的节点 / 单元数 × 大小 × w」，
//!    按 N ∈ {1e4, 1e6, 1e8}、节点头 58 / 67 / 76 逐格算；树高由叶扇出与内部扇出两级各自算出。
//! 2. **批量更新**：一次发布改 k 条（k ∈ {1, 10, 100, 1000}），更新均匀散开，
//!    每层被碰到的节点数按 L × (1 − (1 − 1/L)^k) 算；另报「全部落在同一容器 / 同一叶」的聚簇格。
//! 3. **stat 的读代价**：住索引叶 = 树高次节点读；打包 = inode 树高 + 容器索引树高次节点读 + 1 次 32 KiB 单元读。
//! 4. **爆炸半径**：丢一个容器丢多少条，两条形态各一个数（沿用 E98 的口径）。
//! 5. **判先前估算**：第二轮正推腿写的「≈ 3 倍」按判据 1 的单次更新比核——实测 ≥ 3 则估算成立，< 3 则估算作废并如实写。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照**：把打包形态的容器大小设成 16384 且不计容器索引，它必须与住索引叶形态**逐字节同代价**——
//!   否则两条臂不是在同一把尺子上比，整轮作废。
//! - **阴性对照**：k = 0 时两条形态的结构字节都必须恰好 0。
//! - **饱和对照**：k 远大于层内节点数时，被碰到的节点数必须收敛到该层节点数（差 < 0.5%）。
//! - **反向接受条款**：若单次更新比 < 3，结论是「第二轮正推腿的 3 倍估算作废」，D8 未定项 6 那句要改成实测数。
//!
//! ## 它答不了的
//!
//! 纯算术几何模型：没有 btree 实现、没有 I/O、没有崩溃点重放，文件操作 0 处；不答挂钟，不答缓存命中；
//! 更新均匀散开是选定的场景点，真实负载的局部性只用聚簇格给一个上界。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384;
const UNIT_BYTES: u64 = 32768;
const NODE_HEADERS: [u64; 3] = [58, 67, 76];
/// D19 已定项 4 之后：31 + 14 × 2。
const CHILD_PTR: u64 = 59;
/// E98 的 inode 记录宽。
const INODE_REC: u64 = 140;
/// D18 已定项 11 打包记录单元头（kb 里 `format-const: UNIT_HDR_PACKED`）。
const UNIT_HDR_PACKED: u64 = 93;
/// D2 已定项 9：第一版 2 盘恒 w = 2。
const W: u64 = 2;
/// D23 已定项 12 / E79：每次发布的常量部分（一条 journal 记录 + 根槽），两条形态相同。
const JOURNAL_REC: u64 = 4096;
const ROOT_SLOT: u64 = 512;
/// inode 树按身份引用容器时的 value：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8 + 槽号 1。
const IDENT_VALUE: u64 = 27;
const INODE_KEY: u64 = 8;
/// 容器索引的 key：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。
const CONT_KEY: u64 = 26;

const NS: [u64; 3] = [10_000, 1_000_000, 100_000_000];
const KS: [u64; 4] = [1, 10, 100, 1000];

fn fanout(node: u64, header: u64, entry: u64) -> u64 {
    if entry == 0 || node <= header {
        return 0;
    }
    (node - header) / entry
}

/// 从叶到根每一层的节点数；长度就是树高。
fn tree_levels(n: u64, leaf_f: u64, inner_f: u64) -> Vec<u64> {
    let mut levels = Vec::new();
    if leaf_f == 0 || inner_f < 2 || n == 0 {
        return levels;
    }
    let mut count = n.div_ceil(leaf_f);
    levels.push(count);
    while count > 1 {
        count = count.div_ceil(inner_f);
        levels.push(count);
        if levels.len() > 64 {
            break;
        }
    }
    levels
}

/// k 次均匀散开的更新，期望碰到某一层 `l` 个节点里的几个。
fn touched(l: u64, k: u64) -> f64 {
    if l == 0 || k == 0 {
        return 0.0;
    }
    if k == 1 {
        return 1.0; // 一次更新恰碰一个节点：精确值，不走浮点
    }
    let lf = l as f64;
    lf * (1.0 - (1.0 - 1.0 / lf).powi(k as i32))
}

/// 一棵 COW 树里 k 次散开更新写出的节点数（脏叶 + 全部祖先，D23 已定项 1 甲）。
fn cow_nodes(levels: &[u64], k: u64) -> f64 {
    levels.iter().map(|&l| touched(l, k)).sum()
}

#[derive(Clone, Debug)]
struct Geometry {
    /// 住索引叶形态：inode 树各层节点数。
    a_levels: Vec<u64>,
    /// 打包形态：容器数、容器索引各层节点数、inode 树各层节点数（stat 用）。
    b_containers: u64,
    b_index_levels: Vec<u64>,
    b_inode_levels: Vec<u64>,
    a_leaf_fanout: u64,
    b_per_container: u64,
}

fn geometry(n: u64, hdr: u64) -> Geometry {
    let a_leaf_f = fanout(NODE_BYTES, hdr, INODE_REC);
    let inner_f = fanout(NODE_BYTES, hdr, INODE_KEY + CHILD_PTR);
    let a_levels = tree_levels(n, a_leaf_f, inner_f);
    let per_container = fanout(UNIT_BYTES, UNIT_HDR_PACKED, INODE_REC);
    let containers = n.div_ceil(per_container);
    let idx_f = fanout(NODE_BYTES, hdr, CONT_KEY + CHILD_PTR);
    let b_index_levels = tree_levels(containers, idx_f, idx_f);
    let b_leaf_f = fanout(NODE_BYTES, hdr, INODE_KEY + IDENT_VALUE);
    let b_inode_levels = tree_levels(n, b_leaf_f, inner_f);
    Geometry { a_levels, b_containers: containers, b_index_levels, b_inode_levels, a_leaf_fanout: a_leaf_f, b_per_container: per_container }
}

/// 住索引叶：k 次散开更新写出的结构字节。
fn write_a(g: &Geometry, k: u64) -> f64 {
    cow_nodes(&g.a_levels, k) * (NODE_BYTES * W) as f64
}

/// 打包 + 容器索引：k 次散开更新写出的结构字节（inode 树不动）。
fn write_b(g: &Geometry, k: u64, container_bytes: u64, count_index: bool) -> f64 {
    let c = touched(g.b_containers, k) * (container_bytes * W) as f64;
    let i = if count_index { cow_nodes(&g.b_index_levels, k) * (NODE_BYTES * W) as f64 } else { 0.0 };
    c + i
}

/// 聚簇格：k 条更新全落在同一个容器 / 同一片叶里（k ≤ 容器容量）。
fn write_a_clustered(g: &Geometry) -> f64 {
    g.a_levels.len() as f64 * (NODE_BYTES * W) as f64
}
fn write_b_clustered(g: &Geometry) -> f64 {
    (UNIT_BYTES * W) as f64 + g.b_index_levels.len() as f64 * (NODE_BYTES * W) as f64
}

fn per_publish_const() -> u64 {
    (JOURNAL_REC + ROOT_SLOT) * W
}

/// stat 的读代价（节点读次数, 字节）。
fn read_a(g: &Geometry) -> (u64, u64) {
    let h = g.a_levels.len() as u64;
    (h, h * NODE_BYTES)
}
fn read_b(g: &Geometry) -> (u64, u64) {
    let h = (g.b_inode_levels.len() + g.b_index_levels.len()) as u64;
    (h + 1, h * NODE_BYTES + UNIT_BYTES)
}

fn ratio(b: f64, a: f64) -> f64 {
    if a == 0.0 {
        0.0
    } else {
        b / a
    }
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();
    out.push(em.emit_raw(&format!(
        "name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} packed_hdr={UNIT_HDR_PACKED} inode_rec={INODE_REC} \
         child_ptr={CHILD_PTR} w={W} per_publish_const={} ident_value={IDENT_VALUE} cont_key={CONT_KEY} model=arithmetic file_ops=0",
        per_publish_const()
    )));

    for &hdr in NODE_HEADERS.iter() {
        for &n in NS.iter() {
            let g = geometry(n, hdr);
            out.push(em.emit_raw(&format!(
                "name=geom header={hdr} inodes={n} a_leaf_fanout={} a_height={} a_levels={:?} \
                 b_per_container={} b_containers={} b_index_height={} b_index_levels={:?} b_inode_height={}",
                g.a_leaf_fanout,
                g.a_levels.len(),
                g.a_levels,
                g.b_per_container,
                g.b_containers,
                g.b_index_levels.len(),
                g.b_index_levels,
                g.b_inode_levels.len()
            )));
            // 判据 1 / 2：单次与批量
            for &k in KS.iter() {
                let a = write_a(&g, k);
                let b = write_b(&g, k, UNIT_BYTES, true);
                out.push(em.emit_raw(&format!(
                    "name=write header={hdr} inodes={n} k={k} a_bytes={a:.0} b_bytes={b:.0} b_over_a={:.3} \
                     a_per_update={:.0} b_per_update={:.0} const_per_publish={}",
                    ratio(b, a),
                    a / k as f64,
                    b / k as f64,
                    per_publish_const()
                )));
            }
            // 聚簇格
            let ac = write_a_clustered(&g);
            let bc = write_b_clustered(&g);
            out.push(em.emit_raw(&format!(
                "name=write_clustered header={hdr} inodes={n} a_bytes={ac:.0} b_bytes={bc:.0} b_over_a={:.3}",
                ratio(bc, ac)
            )));
            // 判据 3：读
            let (ra, rab) = read_a(&g);
            let (rb, rbb) = read_b(&g);
            out.push(em.emit_raw(&format!(
                "name=stat header={hdr} inodes={n} a_reads={ra} a_bytes={rab} b_reads={rb} b_bytes={rbb} b_over_a={:.3}",
                ratio(rbb as f64, rab as f64)
            )));
        }
    }
    // 判据 4：爆炸半径（与节点头无关的那一半按 58 报）
    let g = geometry(1_000_000, 58);
    out.push(em.emit_raw(&format!(
        "name=blast a_records_per_leaf={} b_records_per_container={} b_over_a={:.3}",
        g.a_leaf_fanout,
        g.b_per_container,
        ratio(g.b_per_container as f64, g.a_leaf_fanout as f64)
    )));
    // 判据 5：先前估算
    let g = geometry(1_000_000, 58);
    let r = ratio(write_b(&g, 1, UNIT_BYTES, true), write_a(&g, 1));
    out.push(em.emit_raw(&format!(
        "name=prior_estimate claimed=3.0 measured_single_update_ratio_1e6={r:.3} claim_holds={}",
        u8::from(r >= 3.0)
    )));
    // 对照
    let pos = write_b(&g, 1, NODE_BYTES, false) == (NODE_BYTES * W) as f64;
    out.push(em.emit_raw(&format!(
        "name=controls positive_same_ruler={} negative_k0_a={:.0} negative_k0_b={:.0} saturation_1e6_leaf_k_huge={:.1} leaf_count={}",
        u8::from(pos),
        write_a(&g, 0),
        write_b(&g, 0, UNIT_BYTES, true),
        touched(g.a_levels[0], g.a_levels[0] * 100),
        g.a_levels[0]
    )));

    for l in &out {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_kb() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(UNIT_BYTES, 32768, "D4 已定项 7");
        assert_eq!(UNIT_HDR_PACKED, 93, "D18 已定项 11");
        assert_eq!(CHILD_PTR, 59, "D19 已定项 4");
        assert_eq!(W, 2, "D2 已定项 9");
        assert_eq!(JOURNAL_REC, 4096, "D23 已定项 12");
        assert_eq!(IDENT_VALUE, 8 + 2 + 8 + 8 + 1);
        assert_eq!(CONT_KEY, 8 + 2 + 8 + 8);
    }

    /// 几何的绝对值（头 58）：与 E98 的 116 / 243 / 233 对得上，容器索引扇出 192。
    #[test]
    fn geometry_is_absolute() {
        let g = geometry(1_000_000, 58);
        assert_eq!(g.a_leaf_fanout, 116, "E98");
        assert_eq!(g.b_per_container, 233, "E98 / E102");
        assert_eq!(fanout(NODE_BYTES, 58, INODE_KEY + CHILD_PTR), 243, "E98 内部扇出");
        assert_eq!(fanout(NODE_BYTES, 58, CONT_KEY + CHILD_PTR), 192, "(16384 − 58) / 85");
        assert_eq!(fanout(NODE_BYTES, 58, INODE_KEY + IDENT_VALUE), 466, "(16384 − 58) / 35");
        // 1e6：住索引叶 8621 叶 → 36 → 1（高 3）；打包 4292 容器，索引 23 叶 → 1（高 2）
        assert_eq!(g.a_levels, vec![8621, 36, 1]);
        assert_eq!(g.b_containers, 4292);
        assert_eq!(g.b_index_levels, vec![23, 1]);
        assert_eq!(g.b_inode_levels, vec![2146, 9, 1]);
    }

    /// **判据 1 的绝对值**：单次更新，三个 N 逐格钉死（头 58），比值 1.5 / 1.333 / 1.25。
    #[test]
    fn criterion1_single_update_bytes_are_pinned() {
        let g4 = geometry(10_000, 58);
        assert_eq!(write_a(&g4, 1), 65536.0, "2 个节点 × 16384 × 2");
        assert_eq!(write_b(&g4, 1, UNIT_BYTES, true), 98304.0, "容器 65536 + 索引 1 节点 32768");
        let g6 = geometry(1_000_000, 58);
        assert_eq!(write_a(&g6, 1), 98304.0, "3 个节点");
        assert_eq!(write_b(&g6, 1, UNIT_BYTES, true), 131072.0, "容器 65536 + 索引 2 节点 65536");
        let g8 = geometry(100_000_000, 58);
        assert_eq!(write_a(&g8, 1), 131072.0, "4 个节点");
        assert_eq!(write_b(&g8, 1, UNIT_BYTES, true), 163840.0, "容器 65536 + 索引 3 节点 98304");
        assert!((ratio(write_b(&g6, 1, UNIT_BYTES, true), write_a(&g6, 1)) - 4.0 / 3.0).abs() < 1e-9);
    }

    /// **判据 2**：批量散开时打包侧每条更新的边际代价趋向一个容器（32 KiB × w），
    /// 住索引叶趋向一片叶（16 KiB × w）；全量重写时两者相同（同一批 140 字节记录）。
    #[test]
    fn criterion2_batch_amortisation_and_saturation() {
        let g = geometry(1_000_000, 58);
        let a1000 = write_a(&g, 1000);
        let b1000 = write_b(&g, 1000, UNIT_BYTES, true);
        let r = ratio(b1000, a1000);
        assert!(r > 1.5 && r < 2.0, "k=1000 时比值在 (1.5, 2)：{r}");
        // 饱和：k 远大于层内节点数 ⇒ 碰到的节点数收敛到该层节点数
        let sat = touched(g.a_levels[0], g.a_levels[0] * 100);
        let rel = (sat - g.a_levels[0] as f64).abs() / (g.a_levels[0] as f64);
        assert!(rel < 0.005, "饱和相对误差 {rel}");
        // 全量重写：两条形态都写出全部记录所在的容器 ⇒ 字节数量级相同（都 ≈ 1e6 × 140 × w）
        let k = 100_000_000;
        let fa = write_a(&g, k);
        let fb = write_b(&g, k, UNIT_BYTES, true);
        assert!((fb / fa - 1.0).abs() < 0.02, "全量重写比 ≈ 1：{}", fb / fa);
        // 聚簇格：1e6 下 3 节点 vs 容器 + 2 节点
        assert_eq!(write_a_clustered(&g), 98304.0);
        assert_eq!(write_b_clustered(&g), 131072.0);
    }

    /// **判据 3 的绝对值**：stat 读，1e6 头 58：3 次 vs 6 次，49152 vs 114688 字节。
    #[test]
    fn criterion3_stat_reads_are_pinned() {
        let g = geometry(1_000_000, 58);
        assert_eq!(read_a(&g), (3, 49152));
        assert_eq!(read_b(&g), (6, 5 * 16384 + 32768));
    }

    /// **判据 4**：爆炸半径 116 vs 233（E98 口径）。
    #[test]
    fn criterion4_blast_radius_matches_e98() {
        let g = geometry(1_000_000, 58);
        assert_eq!(g.a_leaf_fanout, 116);
        assert_eq!(g.b_per_container, 233);
    }

    /// **判据 5 + 反向接受条款**：第二轮正推腿的「≈ 3 倍」不成立——它只数了叶子。
    #[test]
    fn criterion5_prior_three_times_estimate_does_not_hold() {
        for &n in NS.iter() {
            let g = geometry(n, 58);
            let r = ratio(write_b(&g, 1, UNIT_BYTES, true), write_a(&g, 1));
            assert!(r < 3.0, "N={n} 单次更新比 {r} 应 < 3");
            assert!(r >= 1.25 - 1e-9, "N={n} 单次更新比 {r} 应 ≥ 1.25");
        }
        // 只数叶子的那个算法：96 KiB / 32 KiB = 3——正是估算的来源
        assert_eq!(((UNIT_BYTES + NODE_BYTES) * W) as f64 / (NODE_BYTES * W) as f64, 3.0);
    }

    /// 阳性对照：容器取 16384 且不计索引 ⇒ 与住索引叶的**叶那一层**同代价（同一把尺子）。
    #[test]
    fn positive_control_same_ruler() {
        let g = geometry(1_000_000, 58);
        assert_eq!(write_b(&g, 1, NODE_BYTES, false), (NODE_BYTES * W) as f64);
        assert_eq!(touched(g.a_levels[0], 1) * (NODE_BYTES * W) as f64, (NODE_BYTES * W) as f64);
    }

    /// 阴性对照：k = 0 ⇒ 0；不合法几何 ⇒ 空。
    #[test]
    fn negative_controls() {
        let g = geometry(1_000_000, 58);
        assert_eq!(write_a(&g, 0), 0.0);
        assert_eq!(write_b(&g, 0, UNIT_BYTES, true), 0.0);
        assert!(tree_levels(0, 116, 243).is_empty());
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES, 140), 0);
    }

    /// 三档节点头同向：比值都落在 [1.25, 2)。
    #[test]
    fn header_tiers_do_not_flip_the_direction() {
        for &hdr in NODE_HEADERS.iter() {
            for &n in NS.iter() {
                let g = geometry(n, hdr);
                let r = ratio(write_b(&g, 1, UNIT_BYTES, true), write_a(&g, 1));
                assert!(r >= 1.25 - 1e-9 && r < 2.0, "头 {hdr} N={n} 比值 {r}");
            }
        }
    }
}
