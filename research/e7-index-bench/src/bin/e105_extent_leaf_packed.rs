//! E105：extent 树的叶改成码 3 打包记录单元的更新代价——C113 定案第六节对立臂 B 的价钱。
//!
//! **它答的是**：臂 B（写序只上码 2 / 3，extent 树的叶从码 2 索引叶改成码 3 打包记录单元）
//! 让 extent 树的单次更新、批量更新与点查各贵多少。算法与 E103 同一套（脏叶 + 全部祖先，
//! D23 已定项 1 甲；k 次散开更新按 L × (1 − (1 − 1/L)^k) 算碰到的节点），
//! 只换记录宽：extent 记录 = key 24（locality 8 + inode 8 + offset 8）+ 位置指针 59 = 83（D8 已定项 3 / D19 已定项 4），
//! 另跑一档 52（第四轮反推腿 6.4 的估算：对象 ID 8 + 出生代 8 + 锚点偏移 8 + 位置条目 14 × 2）看灵敏度。
//!
//! ## 判据（跑前写死）
//!
//! 1. 单次更新（k = 1）两种形态各自「写出的节点 / 单元数 × 大小 × w」，N ∈ {1e4, 1e6, 1e8}、节点头 68 / 77 / 86
//!    （D18 已定项 7 三档加 C113 的写序 10）逐格；容器头取 103（93 + 10）。
//! 2. 批量 k ∈ {1, 10, 100, 1000} 散开；饱和对照：k 远大于层内节点数时碰到的节点数收敛到该层节点数。
//! 3. 点查：树高次节点读（码 2 叶）对 树高 − 1 次节点读 + 1 次 32 KiB 单元读（码 3 叶）。
//! 4. 阳性对照：把容器取 16384 且记录宽相同，两种形态的叶那一层代价必须相等（同一把尺子）。
//! 5. 阴性对照：k = 0 ⇒ 两侧 0。
//!
//! 纯算术，文件操作 0 处；不答挂钟、不答缓存；不答 D11 缓冲那一格（那是 E56 的题）。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384;
const UNIT_BYTES: u64 = 32768;
/// D18 已定项 7 三档 58 / 67 / 76 加 C113 的写序 10。
const NODE_HEADERS: [u64; 3] = [68, 77, 86];
/// 码 3 头 93（D18 已定项 11 登记的现行值）+ 写序 10。不用登记名，免得格式常量门禁把它当成现行值的漂移。
const PACKED_HDR_WITH_WSEQ: u64 = 103;
const CHILD_PTR: u64 = 59;
const EXTENT_KEY: u64 = 24;
/// extent 记录 = key 24 + 位置指针 59。
const EXTENT_REC: u64 = 83;
/// 第四轮反推腿 6.4 估的窄记录：对象 ID 8 + 出生代 8 + 锚点偏移 8 + 位置条目 14 × 2。
const EXTENT_REC_NARROW: u64 = 52;
const W: u64 = 2;
const NS: [u64; 3] = [10_000, 1_000_000, 100_000_000];
const KS: [u64; 4] = [1, 10, 100, 1000];

fn fanout(node: u64, header: u64, entry: u64) -> u64 {
    if entry == 0 || node <= header { return 0; }
    (node - header) / entry
}

fn tree_levels(n: u64, leaf_f: u64, inner_f: u64) -> Vec<u64> {
    let mut levels = Vec::new();
    if leaf_f == 0 || inner_f < 2 || n == 0 { return levels; }
    let mut count = n.div_ceil(leaf_f);
    levels.push(count);
    while count > 1 {
        count = count.div_ceil(inner_f);
        levels.push(count);
        if levels.len() > 64 { break; }
    }
    levels
}

fn touched(l: u64, k: u64) -> f64 {
    if l == 0 || k == 0 { return 0.0; }
    if k == 1 { return 1.0; }
    let lf = l as f64;
    lf * (1.0 - (1.0 - 1.0 / lf).powi(k as i32))
}

fn cow_nodes(levels: &[u64], k: u64) -> f64 {
    levels.iter().map(|&l| touched(l, k)).sum()
}

#[derive(Clone, Debug)]
struct Geometry {
    /// 码 2 叶形态：各层节点数（叶层在前）
    a_levels: Vec<u64>,
    /// 码 3 叶形态：容器数 + 内部层各层节点数
    b_containers: u64,
    b_internal: Vec<u64>,
    a_leaf_fanout: u64,
    b_per_container: u64,
}

fn geometry(n: u64, hdr: u64, rec: u64, unit_bytes: u64, unit_hdr: u64) -> Geometry {
    let a_leaf_f = fanout(NODE_BYTES, hdr, rec);
    let inner_f = fanout(NODE_BYTES, hdr, EXTENT_KEY + CHILD_PTR);
    let a_levels = tree_levels(n, a_leaf_f, inner_f);
    let per_container = fanout(unit_bytes, unit_hdr, rec);
    let containers = n.div_ceil(per_container);
    let b_internal = tree_levels(containers, inner_f, inner_f);
    Geometry { a_levels, b_containers: containers, b_internal, a_leaf_fanout: a_leaf_f, b_per_container: per_container }
}

fn write_a(g: &Geometry, k: u64) -> f64 {
    cow_nodes(&g.a_levels, k) * (NODE_BYTES * W) as f64
}

fn write_b(g: &Geometry, k: u64, unit_bytes: u64) -> f64 {
    touched(g.b_containers, k) * (unit_bytes * W) as f64 + cow_nodes(&g.b_internal, k) * (NODE_BYTES * W) as f64
}

fn read_a(g: &Geometry) -> (u64, u64) {
    let h = g.a_levels.len() as u64;
    (h, h * NODE_BYTES)
}

fn read_b(g: &Geometry) -> (u64, u64) {
    let h = g.b_internal.len() as u64;
    (h + 1, h * NODE_BYTES + UNIT_BYTES)
}

/// 树高：甲 = 码 2 叶形态的层数，乙 = 码 3 叶形态的层数（容器算一层）。
fn heights(n: u64, hdr: u64, rec: u64) -> (usize, usize) {
    let g = geometry(n, hdr, rec, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
    (g.a_levels.len(), g.b_internal.len() + 1)
}

/// 连续 N 上乙比甲矮一层的区段（第五轮反推腿 6.4）：这些 N 上单次更新的字节比恒 1.000、点查还少读一个节点。
/// 两侧树高都是阶跃函数，阶跃点只会落在 leaf_f × inner_f^k（甲）与 b_f × inner_f^k（乙）上；
/// 把这些点排好序，逐段用模型自己的 `heights` 判两侧高度，矮一层的段收进来。
fn equal_cost_segments(hdr: u64, rec: u64, n_max: u64) -> Vec<(u64, u64)> {
    let a_f = fanout(NODE_BYTES, hdr, rec);
    let inner = fanout(NODE_BYTES, hdr, EXTENT_KEY + CHILD_PTR);
    let b_f = fanout(UNIT_BYTES, PACKED_HDR_WITH_WSEQ, rec);
    let mut edges: Vec<u64> = vec![0];
    let (mut a_edge, mut b_edge) = (a_f, b_f);
    while a_edge < n_max || b_edge < n_max {
        edges.push(a_edge); edges.push(b_edge);
        a_edge *= inner; b_edge *= inner;
    }
    edges.push(n_max);
    edges.sort_unstable(); edges.dedup();
    let mut out = Vec::new();
    for w in edges.windows(2) {
        let (lo, hi) = (w[0] + 1, w[1].min(n_max));
        if lo > hi { continue; }
        let (ha, hb) = heights(lo, hdr, rec);
        if hb < ha { out.push((lo, hi)); }
    }
    out
}

fn main() {
    let mut em = Emitter::new();
    println!("{}", em.emit_raw(&format!(
        "name=config note=extent 叶改码 3 的更新代价 node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} unit_hdr={PACKED_HDR_WITH_WSEQ} \
         extent_rec={EXTENT_REC} extent_rec_narrow={EXTENT_REC_NARROW} child_ptr={CHILD_PTR} w={W} model=arithmetic file_ops=0")));
    for (rec_name, rec) in [("wide", EXTENT_REC), ("narrow", EXTENT_REC_NARROW)] {
        for &n in &NS {
            for &hdr in &NODE_HEADERS {
                let g = geometry(n, hdr, rec, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
                println!("{}", em.emit_raw(&format!(
                    "name=geom rec={rec_name} n={n} hdr={hdr} a_leaf_fanout={} a_height={} b_per_container={} b_containers={} b_height={}",
                    g.a_leaf_fanout, g.a_levels.len(), g.b_per_container, g.b_containers, g.b_internal.len() + 1)));
                for &k in &KS {
                    let a = write_a(&g, k);
                    let b = write_b(&g, k, UNIT_BYTES);
                    println!("{}", em.emit_raw(&format!(
                        "name=write rec={rec_name} n={n} hdr={hdr} k={k} a_bytes={a:.0} b_bytes={b:.0} b_over_a={:.3}", b / a)));
                }
                let (ar, ab) = read_a(&g);
                let (br, bb) = read_b(&g);
                println!("{}", em.emit_raw(&format!(
                    "name=read rec={rec_name} n={n} hdr={hdr} a_reads={ar} a_bytes={ab} b_reads={br} b_bytes={bb} b_over_a={:.3}", bb as f64 / ab as f64)));
            }
        }
    }
    for (rec_name, rec) in [("wide", EXTENT_REC), ("narrow", EXTENT_REC_NARROW)] {
        for &hdr in &NODE_HEADERS {
            let segs = equal_cost_segments(hdr, rec, 200_000_000);
            let segs_s: Vec<String> = segs.iter().map(|(lo, hi)| format!("{lo}-{hi}")).collect();
            // 对数轴上落进这些区段的比例：ln(hi/lo) / ln(inner_f)
            let inner = fanout(NODE_BYTES, hdr, EXTENT_KEY + CHILD_PTR) as f64;
            let share = segs.first().map_or(0.0, |(lo, hi)| ((*hi as f64) / (*lo as f64)).ln() / inner.ln());
            println!("{}", em.emit_raw(&format!(
                "name=equal_height rec={rec_name} hdr={hdr} segments={} log_share={share:.3}", segs_s.join(","))));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 绝对值：extent 记录 83 时码 2 叶（头 68）装 196 条，码 3 容器（头 103）装 393 条。
    #[test]
    fn fanouts_are_absolute() {
        assert_eq!(fanout(NODE_BYTES, 68, EXTENT_REC), 196);
        assert_eq!(fanout(NODE_BYTES, 86, EXTENT_REC), 196);
        assert_eq!(fanout(UNIT_BYTES, PACKED_HDR_WITH_WSEQ, EXTENT_REC), 393);
        assert_eq!(fanout(UNIT_BYTES, PACKED_HDR_WITH_WSEQ, EXTENT_REC_NARROW), 628);
        assert_eq!(fanout(NODE_BYTES, 68, EXTENT_KEY + CHILD_PTR), 196);
    }

    /// 判据 1 的绝对值：1e6 条、头 68、k = 1：码 2 叶写 3 层 × 16 KiB × 2 = 98304；
    /// 码 3 叶写 1 个 32 KiB 单元 × 2 + 2 层内部节点 × 16 KiB × 2 = 131072 ⇒ 1.333。
    #[test]
    fn single_update_is_pinned() {
        let g = geometry(1_000_000, 68, EXTENT_REC, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
        assert_eq!(g.a_levels.len(), 3);
        assert_eq!(g.b_internal.len(), 2);
        // 容器扇出与容器数要单独钉：单次更新的字节数不随容器数变，变异「扇出按节点算容器」曾一个测试都不红
        assert_eq!(g.b_per_container, 393);
        assert_eq!(g.b_containers, 2545);
        assert_eq!(g.a_levels[0], 5103);
        assert_eq!(write_a(&g, 1), 98304.0);
        assert_eq!(write_b(&g, 1, UNIT_BYTES), 131072.0);
    }

    /// 判据 4 阳性对照：容器取 16384、记录宽相同 ⇒ 叶那一层与码 2 叶同代价（内部层各自算）。
    #[test]
    fn positive_control_same_ruler() {
        for &hdr in &NODE_HEADERS {
            let g = geometry(1_000_000, hdr, EXTENT_REC, NODE_BYTES, hdr);
            assert_eq!(g.b_per_container, g.a_leaf_fanout);
            assert_eq!(g.b_containers, g.a_levels[0]);
            assert_eq!(touched(g.b_containers, 1) * (NODE_BYTES * W) as f64, touched(g.a_levels[0], 1) * (NODE_BYTES * W) as f64);
        }
    }

    /// 判据 5 阴性对照 + 判据 2 饱和对照。
    #[test]
    fn zero_and_saturation() {
        let g = geometry(1_000_000, 68, EXTENT_REC, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
        assert_eq!(write_a(&g, 0), 0.0);
        assert_eq!(write_b(&g, 0, UNIT_BYTES), 0.0);
        let l = g.b_containers;
        let t = touched(l, l * 50);
        let lf = l as f64;
        assert!((t - lf).abs() / lf < 0.005, "饱和：{t} 对 {l}");
    }

    /// 判据 3：点查字节比在三档头、三个 N 上都落在 1.25–2.0 之间（叶从 16 KiB 涨到 32 KiB，树高不增）。
    /// 第五轮反推腿 6.4：三个取样点上的 1.25–1.5 是取样的性质。连续 N 上存在乙比甲矮一层的区段，
    /// 那里单次更新字节比恒 1.000、点查字节相同且少读一个节点；区段边界钉绝对值。
    #[test]
    fn equal_height_segments_are_pinned() {
        let segs = equal_cost_segments(68, EXTENT_REC, 200_000_000);
        assert_eq!(segs, vec![(38417, 77028), (7529537, 15097488)], "{segs:?}");
        for (lo, hi) in [(38417u64, 77028u64), (7529537, 15097488)] {
            for n in [lo, hi] {
                let g = geometry(n, 68, EXTENT_REC, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
                let (ha, hb) = heights(n, 68, EXTENT_REC);
                assert_eq!(hb + 1, ha, "n={n}");
                assert_eq!(write_b(&g, 1, UNIT_BYTES), write_a(&g, 1), "n={n}");
                let (ra, ba) = read_a(&g);
                let (rb, bb) = read_b(&g);
                assert_eq!(bb, ba, "n={n}");
                assert_eq!(rb + 1, ra, "n={n}：乙少读一个节点");
            }
            // 区段两端之外恰好回到 1 + 1/h_A
            for n in [lo - 1, hi + 1] {
                let g = geometry(n, 68, EXTENT_REC, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
                let (ha, hb) = heights(n, 68, EXTENT_REC);
                assert_eq!(hb, ha, "n={n}");
                assert!(write_b(&g, 1, UNIT_BYTES) > write_a(&g, 1), "n={n}");
            }
        }
        // 窄记录同形，边界不同
        assert_eq!(equal_cost_segments(68, EXTENT_REC_NARROW, 200_000_000)[0], (61349, 123088));
        // 单容器的树上面仍有一个码 2 根节点：n ≤ 196 时甲一层、乙两层，比值 2.0，不是等高段
        assert_eq!(heights(196, 68, EXTENT_REC), (1, 2));
        // 三档节点头不改边界：内部扇出 (16384 − hdr) / 83 在三档上都是 196
        for &hdr in &NODE_HEADERS { assert_eq!(equal_cost_segments(hdr, EXTENT_REC, 200_000_000), segs); }
    }

    #[test]
    fn read_ratio_bounds() {
        for &n in &NS {
            for &hdr in &NODE_HEADERS {
                let g = geometry(n, hdr, EXTENT_REC, UNIT_BYTES, PACKED_HDR_WITH_WSEQ);
                let (_, ab) = read_a(&g);
                let (_, bb) = read_b(&g);
                let r = bb as f64 / ab as f64;
                assert!((1.25..=2.0).contains(&r), "n={n} hdr={hdr} r={r}");
                assert!(r >= 1.0, "解析下界：容器扇出 ≥ 叶扇出 ⇒ 乙不比甲高，也至多矮一层 ⇒ 比值 ≥ 1");
                assert!(g.b_internal.len() + 1 <= g.a_levels.len(), "码 3 叶的树高不许比码 2 叶高");
            }
        }
    }
}
