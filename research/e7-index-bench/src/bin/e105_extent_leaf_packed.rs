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
//!    （D18 已定项 7 三档加 C113 的写序 10）逐格；容器头取 107（93 + 写序 10 + 出生序号 4）。
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
/// D18 已定项 11 打包记录单元头（kb 里 `format-const: PACKED_UNIT_HEADER_BYTES`；C113 定案 2026-09-05 加写序 10；C288 ① 2026-09-12 加出生序号 4）。
const PACKED_UNIT_HEADER_BYTES: u64 = 107;
const CHILD_POINTER_BYTES: u64 = 59;
const EXTENT_KEY: u64 = 24;
/// extent 记录 = key 24 + 位置指针 59。
const EXTENT_RECORD_BYTES: u64 = 83;
/// 第四轮反推腿 6.4 估的窄记录：对象 ID 8 + 出生代 8 + 锚点偏移 8 + 位置条目 14 × 2。
const EXTENT_RECORD_NARROW_BYTES: u64 = 52;
const STRIPE_WIDTH: u64 = 2;
const EXTENT_RECORD_COUNTS: [u64; 3] = [10_000, 1_000_000, 100_000_000];
const SCATTERED_UPDATE_COUNTS: [u64; 4] = [1, 10, 100, 1000];

fn fanout(container_bytes: u64, header_bytes: u64, entry_bytes: u64) -> u64 {
    if entry_bytes == 0 || container_bytes <= header_bytes { return 0; }
    (container_bytes - header_bytes) / entry_bytes
}

fn tree_levels(record_count: u64, leaf_fanout: u64, inner_fanout: u64) -> Vec<u64> {
    let mut levels = Vec::new();
    if leaf_fanout == 0 || inner_fanout < 2 || record_count == 0 { return levels; }
    let mut level_node_count = record_count.div_ceil(leaf_fanout);
    levels.push(level_node_count);
    while level_node_count > 1 {
        level_node_count = level_node_count.div_ceil(inner_fanout);
        levels.push(level_node_count);
        if levels.len() > 64 { break; }
    }
    levels
}

fn touched(level_node_count: u64, update_count: u64) -> f64 {
    if level_node_count == 0 || update_count == 0 { return 0.0; }
    if update_count == 1 { return 1.0; }
    let level_node_count_float = level_node_count as f64;
    level_node_count_float * (1.0 - (1.0 - 1.0 / level_node_count_float).powi(update_count as i32))
}

fn copy_on_write_nodes(levels: &[u64], update_count: u64) -> f64 {
    levels.iter().map(|&level_node_count| touched(level_node_count, update_count)).sum()
}

#[derive(Clone, Debug)]
struct Geometry {
    /// 码 2 叶形态：各层节点数（叶层在前）
    index_leaf_tree_levels: Vec<u64>,
    /// 码 3 叶形态：容器数 + 内部层各层节点数
    container_count: u64,
    packed_internal_levels: Vec<u64>,
    index_leaf_fanout: u64,
    records_per_container: u64,
}

fn geometry(record_count: u64, node_header_bytes: u64, record_bytes: u64, unit_bytes: u64, unit_header_bytes: u64) -> Geometry {
    let index_leaf_fanout = fanout(NODE_BYTES, node_header_bytes, record_bytes);
    let inner_fanout = fanout(NODE_BYTES, node_header_bytes, EXTENT_KEY + CHILD_POINTER_BYTES);
    let index_leaf_tree_levels = tree_levels(record_count, index_leaf_fanout, inner_fanout);
    let records_per_container = fanout(unit_bytes, unit_header_bytes, record_bytes);
    let container_count = record_count.div_ceil(records_per_container);
    let packed_internal_levels = tree_levels(container_count, inner_fanout, inner_fanout);
    Geometry { index_leaf_tree_levels, container_count: container_count, packed_internal_levels, index_leaf_fanout: index_leaf_fanout, records_per_container: records_per_container }
}

fn index_leaf_form_update_bytes(tree_geometry: &Geometry, update_count: u64) -> f64 {
    copy_on_write_nodes(&tree_geometry.index_leaf_tree_levels, update_count) * (NODE_BYTES * STRIPE_WIDTH) as f64
}

fn packed_unit_form_update_bytes(tree_geometry: &Geometry, update_count: u64, unit_bytes: u64) -> f64 {
    touched(tree_geometry.container_count, update_count) * (unit_bytes * STRIPE_WIDTH) as f64 + copy_on_write_nodes(&tree_geometry.packed_internal_levels, update_count) * (NODE_BYTES * STRIPE_WIDTH) as f64
}

fn index_leaf_form_point_lookup(tree_geometry: &Geometry) -> (u64, u64) {
    let index_leaf_height = tree_geometry.index_leaf_tree_levels.len() as u64;
    (index_leaf_height, index_leaf_height * NODE_BYTES)
}

fn packed_unit_form_point_lookup(tree_geometry: &Geometry) -> (u64, u64) {
    let packed_internal_height = tree_geometry.packed_internal_levels.len() as u64;
    (packed_internal_height + 1, packed_internal_height * NODE_BYTES + UNIT_BYTES)
}

/// 树高：甲 = 码 2 叶形态的层数，乙 = 码 3 叶形态的层数（容器算一层）。
fn heights(record_count: u64, node_header_bytes: u64, record_bytes: u64) -> (usize, usize) {
    let tree_geometry = geometry(record_count, node_header_bytes, record_bytes, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
    (tree_geometry.index_leaf_tree_levels.len(), tree_geometry.packed_internal_levels.len() + 1)
}

/// 连续 N 上乙比甲矮一层的区段（第五轮反推腿 6.4）：这些 N 上单次更新的字节比恒 1.000、点查还少读一个节点。
/// 两侧树高都是阶跃函数，阶跃点只会落在 leaf_f × inner_f^k（甲）与 b_f × inner_f^k（乙）上；
/// 把这些点排好序，逐段用模型自己的 `heights` 判两侧高度，矮一层的段收进来。
fn equal_cost_segments(node_header_bytes: u64, record_bytes: u64, record_count_limit: u64) -> Vec<(u64, u64)> {
    let index_leaf_fanout = fanout(NODE_BYTES, node_header_bytes, record_bytes);
    let inner_fanout = fanout(NODE_BYTES, node_header_bytes, EXTENT_KEY + CHILD_POINTER_BYTES);
    let records_per_container = fanout(UNIT_BYTES, PACKED_UNIT_HEADER_BYTES, record_bytes);
    let mut edges: Vec<u64> = vec![0];
    let (mut index_leaf_height_step, mut packed_height_step) = (index_leaf_fanout, records_per_container);
    while index_leaf_height_step < record_count_limit || packed_height_step < record_count_limit {
        edges.push(index_leaf_height_step); edges.push(packed_height_step);
        index_leaf_height_step *= inner_fanout; packed_height_step *= inner_fanout;
    }
    edges.push(record_count_limit);
    edges.sort_unstable(); edges.dedup();
    let mut segments = Vec::new();
    for adjacent_edges in edges.windows(2) {
        let (segment_start, segment_end) = (adjacent_edges[0] + 1, adjacent_edges[1].min(record_count_limit));
        if segment_start > segment_end { continue; }
        let (index_leaf_height, packed_height) = heights(segment_start, node_header_bytes, record_bytes);
        if packed_height < index_leaf_height { segments.push((segment_start, segment_end)); }
    }
    segments
}

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!(
        "name=config note=extent 叶改码 3 的更新代价 node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} unit_hdr={PACKED_UNIT_HEADER_BYTES} \
         extent_rec={EXTENT_RECORD_BYTES} extent_rec_narrow={EXTENT_RECORD_NARROW_BYTES} child_ptr={CHILD_POINTER_BYTES} w={STRIPE_WIDTH} model=arithmetic file_ops=0")));
    for (record_width_name, record_bytes) in [("wide", EXTENT_RECORD_BYTES), ("narrow", EXTENT_RECORD_NARROW_BYTES)] {
        for &record_count in &EXTENT_RECORD_COUNTS {
            for &node_header_bytes in &NODE_HEADERS {
                let tree_geometry = geometry(record_count, node_header_bytes, record_bytes, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
                println!("{}", emitter.emit_raw(&format!(
                    "name=geom rec={record_width_name} n={record_count} hdr={node_header_bytes} a_leaf_fanout={} a_height={} b_per_container={} b_containers={} b_height={}",
                    tree_geometry.index_leaf_fanout, tree_geometry.index_leaf_tree_levels.len(), tree_geometry.records_per_container, tree_geometry.container_count, tree_geometry.packed_internal_levels.len() + 1)));
                for &update_count in &SCATTERED_UPDATE_COUNTS {
                    let index_leaf_update_bytes = index_leaf_form_update_bytes(&tree_geometry, update_count);
                    let packed_update_bytes = packed_unit_form_update_bytes(&tree_geometry, update_count, UNIT_BYTES);
                    println!("{}", emitter.emit_raw(&format!(
                        "name=write rec={record_width_name} n={record_count} hdr={node_header_bytes} k={update_count} a_bytes={index_leaf_update_bytes:.0} b_bytes={packed_update_bytes:.0} b_over_a={:.3}", packed_update_bytes / index_leaf_update_bytes)));
                }
                let (index_leaf_reads, index_leaf_read_bytes) = index_leaf_form_point_lookup(&tree_geometry);
                let (packed_reads, packed_read_bytes) = packed_unit_form_point_lookup(&tree_geometry);
                println!("{}", emitter.emit_raw(&format!(
                    "name=read rec={record_width_name} n={record_count} hdr={node_header_bytes} a_reads={index_leaf_reads} a_bytes={index_leaf_read_bytes} b_reads={packed_reads} b_bytes={packed_read_bytes} b_over_a={:.3}", packed_read_bytes as f64 / index_leaf_read_bytes as f64)));
            }
        }
    }
    for (record_width_name, record_bytes) in [("wide", EXTENT_RECORD_BYTES), ("narrow", EXTENT_RECORD_NARROW_BYTES)] {
        for &node_header_bytes in &NODE_HEADERS {
            let segments = equal_cost_segments(node_header_bytes, record_bytes, 200_000_000);
            let segment_labels: Vec<String> = segments.iter().map(|(segment_start, segment_end)| format!("{segment_start}-{segment_end}")).collect();
            // 对数轴上落进这些区段的比例：ln(hi/lo) / ln(inner_f)
            let inner_fanout = fanout(NODE_BYTES, node_header_bytes, EXTENT_KEY + CHILD_POINTER_BYTES) as f64;
            let log_share = segments.first().map_or(0.0, |(segment_start, segment_end)| ((*segment_end as f64) / (*segment_start as f64)).ln() / inner_fanout.ln());
            println!("{}", emitter.emit_raw(&format!(
                "name=equal_height rec={record_width_name} hdr={node_header_bytes} segments={} log_share={log_share:.3}", segment_labels.join(","))));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 绝对值：extent 记录 83 时码 2 叶（头 68）装 196 条，码 3 容器（头 107）装 393 条。
    #[test]
    fn fanouts_are_absolute() {
        assert_eq!(fanout(NODE_BYTES, 68, EXTENT_RECORD_BYTES), 196);
        assert_eq!(fanout(NODE_BYTES, 86, EXTENT_RECORD_BYTES), 196);
        assert_eq!(fanout(UNIT_BYTES, PACKED_UNIT_HEADER_BYTES, EXTENT_RECORD_BYTES), 393);
        assert_eq!(fanout(UNIT_BYTES, PACKED_UNIT_HEADER_BYTES, EXTENT_RECORD_NARROW_BYTES), 628);
        assert_eq!(fanout(NODE_BYTES, 68, EXTENT_KEY + CHILD_POINTER_BYTES), 196);
    }

    /// 判据 1 的绝对值：1e6 条、头 68、k = 1：码 2 叶写 3 层 × 16 KiB × 2 = 98304；
    /// 码 3 叶写 1 个 32 KiB 单元 × 2 + 2 层内部节点 × 16 KiB × 2 = 131072 ⇒ 1.333。
    #[test]
    fn single_update_is_pinned() {
        let tree_geometry = geometry(1_000_000, 68, EXTENT_RECORD_BYTES, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
        assert_eq!(tree_geometry.index_leaf_tree_levels.len(), 3);
        assert_eq!(tree_geometry.packed_internal_levels.len(), 2);
        // 容器扇出与容器数要单独钉：单次更新的字节数不随容器数变，变异「扇出按节点算容器」曾一个测试都不红
        assert_eq!(tree_geometry.records_per_container, 393);
        assert_eq!(tree_geometry.container_count, 2545);
        assert_eq!(tree_geometry.index_leaf_tree_levels[0], 5103);
        assert_eq!(index_leaf_form_update_bytes(&tree_geometry, 1), 98304.0);
        assert_eq!(packed_unit_form_update_bytes(&tree_geometry, 1, UNIT_BYTES), 131072.0);
    }

    /// 判据 4 阳性对照：容器取 16384、记录宽相同 ⇒ 叶那一层与码 2 叶同代价（内部层各自算）。
    #[test]
    fn positive_control_same_ruler() {
        for &node_header_bytes in &NODE_HEADERS {
            let tree_geometry = geometry(1_000_000, node_header_bytes, EXTENT_RECORD_BYTES, NODE_BYTES, node_header_bytes);
            assert_eq!(tree_geometry.records_per_container, tree_geometry.index_leaf_fanout);
            assert_eq!(tree_geometry.container_count, tree_geometry.index_leaf_tree_levels[0]);
            assert_eq!(touched(tree_geometry.container_count, 1) * (NODE_BYTES * STRIPE_WIDTH) as f64, touched(tree_geometry.index_leaf_tree_levels[0], 1) * (NODE_BYTES * STRIPE_WIDTH) as f64);
        }
    }

    /// 判据 5 阴性对照 + 判据 2 饱和对照。
    #[test]
    fn zero_and_saturation() {
        let tree_geometry = geometry(1_000_000, 68, EXTENT_RECORD_BYTES, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
        assert_eq!(index_leaf_form_update_bytes(&tree_geometry, 0), 0.0);
        assert_eq!(packed_unit_form_update_bytes(&tree_geometry, 0, UNIT_BYTES), 0.0);
        let container_count = tree_geometry.container_count;
        let touched_containers = touched(container_count, container_count * 50);
        let container_count_float = container_count as f64;
        assert!((touched_containers - container_count_float).abs() / container_count_float < 0.005, "饱和：{touched_containers} 对 {container_count}");
    }

    /// 判据 3：点查字节比在三档头、三个 N 上都落在 1.25–2.0 之间（叶从 16 KiB 涨到 32 KiB，树高不增）。
    /// 第五轮反推腿 6.4：三个取样点上的 1.25–1.5 是取样的性质。连续 N 上存在乙比甲矮一层的区段，
    /// 那里单次更新字节比恒 1.000、点查字节相同且少读一个节点；区段边界钉绝对值。
    #[test]
    fn equal_height_segments_are_pinned() {
        let segments = equal_cost_segments(68, EXTENT_RECORD_BYTES, 200_000_000);
        assert_eq!(segments, vec![(38417, 77028), (7529537, 15097488)], "{segments:?}");
        for (segment_start, segment_end) in [(38417u64, 77028u64), (7529537, 15097488)] {
            for record_count in [segment_start, segment_end] {
                let tree_geometry = geometry(record_count, 68, EXTENT_RECORD_BYTES, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
                let (index_leaf_height, packed_height) = heights(record_count, 68, EXTENT_RECORD_BYTES);
                assert_eq!(packed_height + 1, index_leaf_height, "n={record_count}");
                assert_eq!(packed_unit_form_update_bytes(&tree_geometry, 1, UNIT_BYTES), index_leaf_form_update_bytes(&tree_geometry, 1), "n={record_count}");
                let (index_leaf_reads, index_leaf_read_bytes) = index_leaf_form_point_lookup(&tree_geometry);
                let (packed_reads, packed_read_bytes) = packed_unit_form_point_lookup(&tree_geometry);
                assert_eq!(packed_read_bytes, index_leaf_read_bytes, "n={record_count}");
                assert_eq!(packed_reads + 1, index_leaf_reads, "n={record_count}：乙少读一个节点");
            }
            // 区段两端之外恰好回到 1 + 1/h_A
            for record_count in [segment_start - 1, segment_end + 1] {
                let tree_geometry = geometry(record_count, 68, EXTENT_RECORD_BYTES, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
                let (index_leaf_height, packed_height) = heights(record_count, 68, EXTENT_RECORD_BYTES);
                assert_eq!(packed_height, index_leaf_height, "n={record_count}");
                assert!(packed_unit_form_update_bytes(&tree_geometry, 1, UNIT_BYTES) > index_leaf_form_update_bytes(&tree_geometry, 1), "n={record_count}");
            }
        }
        // 窄记录同形，边界不同
        assert_eq!(equal_cost_segments(68, EXTENT_RECORD_NARROW_BYTES, 200_000_000)[0], (61349, 123088));
        // 单容器的树上面仍有一个码 2 根节点：n ≤ 196 时甲一层、乙两层，比值 2.0，不是等高段
        assert_eq!(heights(196, 68, EXTENT_RECORD_BYTES), (1, 2));
        // 三档节点头不改边界：内部扇出 (16384 − hdr) / 83 在三档上都是 196
        for &node_header_bytes in &NODE_HEADERS { assert_eq!(equal_cost_segments(node_header_bytes, EXTENT_RECORD_BYTES, 200_000_000), segments); }
    }

    #[test]
    fn read_ratio_bounds() {
        for &record_count in &EXTENT_RECORD_COUNTS {
            for &node_header_bytes in &NODE_HEADERS {
                let tree_geometry = geometry(record_count, node_header_bytes, EXTENT_RECORD_BYTES, UNIT_BYTES, PACKED_UNIT_HEADER_BYTES);
                let (_, index_leaf_read_bytes) = index_leaf_form_point_lookup(&tree_geometry);
                let (_, packed_read_bytes) = packed_unit_form_point_lookup(&tree_geometry);
                let read_bytes_ratio = packed_read_bytes as f64 / index_leaf_read_bytes as f64;
                assert!((1.25..=2.0).contains(&read_bytes_ratio), "n={record_count} hdr={node_header_bytes} r={read_bytes_ratio}");
                assert!(read_bytes_ratio >= 1.0, "解析下界：容器扇出 ≥ 叶扇出 ⇒ 乙不比甲高，也至多矮一层 ⇒ 比值 ≥ 1");
                assert!(tree_geometry.packed_internal_levels.len() + 1 <= tree_geometry.index_leaf_tree_levels.len(), "码 3 叶的树高不许比码 2 叶高");
            }
        }
    }
}
