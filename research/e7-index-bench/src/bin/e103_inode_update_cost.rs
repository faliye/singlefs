//! E103：inode 打包路 ① 的更新代价 —— D8 已定项 6 比较表欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D8 已定项 6**（2026-09-04 随 D18 已定项 11 补的那句）：「更新代价要进比较表（第二轮正推腿估算，
//!   未实测：改一条记录 = COW 32 KiB × w + 容器索引一条 ≈ 3 倍于住索引叶；stat 多一次查找；爆炸半径 2 倍）」。
//!   ⚠️ 那个估算**只数了叶子**：住索引叶那一侧算成 1 个 16 KiB 叶，打包那一侧算成 1 个容器 + 1 个索引叶。
//! - **D23 已定项 1 取甲，逐字**：「每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。祖先不延后。」
//!   ⇒ 更新代价必须把**树高**算进去，两条形态的祖先数不同。
//! - **D18 已定项 11**：打包记录单元头 107（2026-09-05 C113 定案加写序 10 之前是 93，2026-09-12 C288 ① 加出生序号 4 之前是 103，叶容量 233 都不变）；容器索引是物理指针唯一持有者，别的树按身份
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
//!    **第三臂「叶即容器」（2026-09-05 第一轮论证后加）**：inode 树的叶就是码 3 容器、内部节点就是容器索引
//!    ⇒ 写 = 容器 + 内部层的节点；读 = 内部层次数 + 1 次 32 KiB 单元读。
//!    内部节点条目 = 分隔 key 8 + 身份引用 26（出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8，D18 已定项 11 给别的树引用容器的那 26 字节）+ 子指针 59 = 93
//!    （第四版：类型段 0 表示子节点是码 2 索引节点、2 表示码 3 容器，读者由此知道读几个槽、AAD 期望值全部来自条目；出生树逐条目携带是因为克隆头共享 origin 的叶）。
//!    基础节点头三档 58 / 67 / 76 是 E73 的**不带 key 区间**下界；inode 树内部节点带 16 字节区间后是 74 / 83 / 92，扇出仍 175（单测钉住 92，以及 109 / 110 那道边）。
//! 4. **爆炸半径**：丢一个容器丢多少条，两条形态各一个数（沿用 E98 的口径）。
//! 5. **判先前估算**：第二轮正推腿写的「≈ 3 倍」按判据 1 的单次更新比核——实测 ≥ 3 则估算成立，< 3 则估算作废并如实写。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照**：把打包形态的容器大小设成 16384 且不计容器索引，它必须与住索引叶形态**逐字节同代价**——
//!   否则两条臂不是在同一把尺子上比，整轮作废。
//! - **阴性对照**：k = 0 时两条形态的结构字节都必须恰好 0。
//! - **饱和对照**：k 远大于层内节点数时，被碰到的节点数必须收敛到该层节点数（差 < 0.5%）。
//! - **反向接受条款**：若单次更新比 < 3，结论是「第二轮正推腿的 3 倍估算作废」，D8 已定项 6 那句要改成实测数。
//!
//! ## 它答不了的
//!
//! 纯算术几何模型：没有 btree 实现、没有 I/O、没有崩溃点重放，文件操作 0 处；不答挂钟，不答缓存命中；
//! 更新均匀散开是选定的场景点，真实负载的局部性只用聚簇格给一个上界。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384;
const UNIT_BYTES: u64 = 32768;
const NODE_HEADERS: [u64; 3] = [58, 67, 76];
/// 今天成立的三档基础节点头。上面那组 58 / 67 / 76 是 E73 跑那天的下界，
/// 此后两笔加宽从来没落到本实验的源码里，2026-09-07 一次补上：
/// ① D18 已定项 7 补注：三档各加**写序 10** 成 68 / 77 / 86；
/// ② D18 已定项 7 逐字「v1 就预留 nonce 代号与 MAC 的字段位」，两个位在字段表**之外**——
///    按 D18 已定项 14 臂甲（nonce 代号 = 完整 96 位 nonce 12）与 D9 已定项 2（MAC 满 128 位 16）
///    共 **28 字节**，再各加一次成 96 / 105 / 114（E117 算的）。
/// **两组都留着并逐格跑**：旧那组是这条结论此前站的地方，删掉就看不出这次补账改了什么。
/// D18 已定项 7 的两个预留位合计（nonce 代号 12 + MAC 16）。E117 的 `resv12` 臂同一个数。
const RESERVED_HEADER_BYTES: u64 = 12 + 16;
/// 写序（C113 定案 P1）。D18 已定项 7 补注：三档下界各加它。
const WRITE_ORDER: u64 = 10;
const NODE_HEADERS_TODAY: [u64; 3] = [
    NODE_HEADERS[0] + WRITE_ORDER + RESERVED_HEADER_BYTES,
    NODE_HEADERS[1] + WRITE_ORDER + RESERVED_HEADER_BYTES,
    NODE_HEADERS[2] + WRITE_ORDER + RESERVED_HEADER_BYTES,
];
/// D19 已定项 4 之后：31 + 14 × 2。
const CHILD_POINTER_BYTES: u64 = 59;
/// E98 的 inode 记录宽。
const INODE_RECORD_BYTES: u64 = 140;
/// D18 已定项 11 打包记录单元头（kb 里 `format-const: PACKED_UNIT_HEADER_BYTES`）。
const PACKED_UNIT_HEADER_BYTES: u64 = 107;
/// D2 已定项 9：第一版 2 盘恒 w = 2。
const COLUMNS_PER_WRITE: u64 = 2;
/// D23 已定项 12 / E79：每次发布的常量部分（一条 journal 记录 + 根槽），两条形态相同。
const JOURNAL_RECORD_BYTES: u64 = 4096;
const ROOT_SLOT: u64 = 512;
/// inode 树按身份引用容器时的 value：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8 + 槽号 1。
const IDENTITY_VALUE_BYTES: u64 = 27;
const INODE_KEY: u64 = 8;
/// 容器索引的 key：出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8。
const CONTAINER_KEY_BYTES: u64 = 26;
/// 第三臂：inode 树内部节点的条目 = 分隔 key 8 + 身份引用 26（与 CONTAINER_KEY_BYTES 同一段）+ 子指针 59 = 93。
/// 写成字面量是给 27 号门禁（format-const）钉的；与推导式的相等由单测守。
const INODE_INTERNAL_ENTRY: u64 = 93;
/// 一个类型 2 容器装几条记录 = ⌊(32768 − 107) / 140⌋ = 233（93、103 时也是 233）；字面量同样为 27 号门禁，与 fanout() 的相等由单测守。
const INODE_LEAF_RECORDS: u64 = 233;

const INODE_COUNTS: [u64; 3] = [10_000, 1_000_000, 100_000_000];
const UPDATE_COUNTS: [u64; 4] = [1, 10, 100, 1000];

fn fanout(node: u64, header: u64, entry: u64) -> u64 {
    if entry == 0 || node <= header {
        return 0;
    }
    (node - header) / entry
}

/// 从叶到根每一层的节点数；长度就是树高。
fn tree_levels(item_count: u64, leaf_fanout: u64, internal_fanout: u64) -> Vec<u64> {
    let mut levels = Vec::new();
    if leaf_fanout == 0 || internal_fanout < 2 || item_count == 0 {
        return levels;
    }
    let mut count = item_count.div_ceil(leaf_fanout);
    levels.push(count);
    while count > 1 {
        count = count.div_ceil(internal_fanout);
        levels.push(count);
        if levels.len() > 64 {
            break;
        }
    }
    levels
}

/// k 次均匀散开的更新，期望碰到某一层 `nodes_in_level` 个节点里的几个。
fn touched(nodes_in_level: u64, update_count: u64) -> f64 {
    if nodes_in_level == 0 || update_count == 0 {
        return 0.0;
    }
    if update_count == 1 {
        return 1.0; // 一次更新恰碰一个节点：精确值，不走浮点
    }
    let nodes_in_level_as_float = nodes_in_level as f64;
    nodes_in_level_as_float * (1.0 - (1.0 - 1.0 / nodes_in_level_as_float).powi(update_count as i32))
}

/// 一棵 COW 树里 k 次散开更新写出的节点数（脏叶 + 全部祖先，D23 已定项 1 甲）。
fn copy_on_write_nodes_written(levels: &[u64], update_count: u64) -> f64 {
    levels.iter().map(|&nodes_in_level| touched(nodes_in_level, update_count)).sum()
}

#[derive(Clone, Debug)]
struct Geometry {
    /// 住索引叶形态：inode 树各层节点数。
    in_leaf_tree_levels: Vec<u64>,
    /// 打包形态：容器数、容器索引各层节点数、inode 树各层节点数（stat 用）。
    packed_container_count: u64,
    packed_container_index_levels: Vec<u64>,
    packed_inode_tree_levels: Vec<u64>,
    in_leaf_records_per_leaf: u64,
    packed_records_per_container: u64,
    /// 第三臂：叶即容器时内部层各层节点数（叶层 = 容器数，不在此列）。
    leaf_as_container_internal_levels: Vec<u64>,
}

fn geometry(inode_count: u64, node_header: u64) -> Geometry {
    let in_leaf_leaf_fanout = fanout(NODE_BYTES, node_header, INODE_RECORD_BYTES);
    let internal_fanout = fanout(NODE_BYTES, node_header, INODE_KEY + CHILD_POINTER_BYTES);
    let in_leaf_tree_levels = tree_levels(inode_count, in_leaf_leaf_fanout, internal_fanout);
    let records_per_container = fanout(UNIT_BYTES, PACKED_UNIT_HEADER_BYTES, INODE_RECORD_BYTES);
    let container_count = inode_count.div_ceil(records_per_container);
    let container_index_fanout = fanout(NODE_BYTES, node_header, CONTAINER_KEY_BYTES + CHILD_POINTER_BYTES);
    let packed_container_index_levels = tree_levels(container_count, container_index_fanout, container_index_fanout);
    let packed_inode_leaf_fanout = fanout(NODE_BYTES, node_header, INODE_KEY + IDENTITY_VALUE_BYTES);
    let packed_inode_tree_levels = tree_levels(inode_count, packed_inode_leaf_fanout, internal_fanout);
    let leaf_as_container_internal_fanout = fanout(NODE_BYTES, node_header, INODE_INTERNAL_ENTRY);
    let leaf_as_container_internal_levels = tree_levels(container_count, leaf_as_container_internal_fanout, leaf_as_container_internal_fanout);
    Geometry { in_leaf_tree_levels, packed_container_count: container_count, packed_container_index_levels, packed_inode_tree_levels, in_leaf_records_per_leaf: in_leaf_leaf_fanout, packed_records_per_container: records_per_container, leaf_as_container_internal_levels }
}

/// 住索引叶：k 次散开更新写出的结构字节。
fn write_in_leaf_bytes(tree_geometry: &Geometry, update_count: u64) -> f64 {
    copy_on_write_nodes_written(&tree_geometry.in_leaf_tree_levels, update_count) * (NODE_BYTES * COLUMNS_PER_WRITE) as f64
}

/// 打包 + 容器索引：k 次散开更新写出的结构字节（inode 树不动）。
fn write_packed_bytes(tree_geometry: &Geometry, update_count: u64, container_bytes: u64, count_index: bool) -> f64 {
    let containers_written_bytes = touched(tree_geometry.packed_container_count, update_count) * (container_bytes * COLUMNS_PER_WRITE) as f64;
    let container_index_written_bytes = if count_index { copy_on_write_nodes_written(&tree_geometry.packed_container_index_levels, update_count) * (NODE_BYTES * COLUMNS_PER_WRITE) as f64 } else { 0.0 };
    containers_written_bytes + container_index_written_bytes
}

/// 第三臂：叶即容器。k 次散开更新 = 碰到的容器 + 内部层碰到的节点。
fn write_leaf_as_container_bytes(tree_geometry: &Geometry, update_count: u64) -> f64 {
    touched(tree_geometry.packed_container_count, update_count) * (UNIT_BYTES * COLUMNS_PER_WRITE) as f64 + copy_on_write_nodes_written(&tree_geometry.leaf_as_container_internal_levels, update_count) * (NODE_BYTES * COLUMNS_PER_WRITE) as f64
}
fn file_status_reads_leaf_as_container(tree_geometry: &Geometry) -> (u64, u64) {
    let internal_height = tree_geometry.leaf_as_container_internal_levels.len() as u64;
    (internal_height + 1, internal_height * NODE_BYTES + UNIT_BYTES)
}

/// 聚簇格：k 条更新全落在同一个容器 / 同一片叶里（k ≤ 容器容量）。
fn write_in_leaf_clustered_bytes(tree_geometry: &Geometry) -> f64 {
    tree_geometry.in_leaf_tree_levels.len() as f64 * (NODE_BYTES * COLUMNS_PER_WRITE) as f64
}
fn write_packed_clustered_bytes(tree_geometry: &Geometry) -> f64 {
    (UNIT_BYTES * COLUMNS_PER_WRITE) as f64 + tree_geometry.packed_container_index_levels.len() as f64 * (NODE_BYTES * COLUMNS_PER_WRITE) as f64
}

fn per_publish_constant_bytes() -> u64 {
    (JOURNAL_RECORD_BYTES + ROOT_SLOT) * COLUMNS_PER_WRITE
}

/// stat 的读代价（节点读次数, 字节）。
fn file_status_reads_in_leaf(tree_geometry: &Geometry) -> (u64, u64) {
    let tree_height = tree_geometry.in_leaf_tree_levels.len() as u64;
    (tree_height, tree_height * NODE_BYTES)
}
fn file_status_reads_packed(tree_geometry: &Geometry) -> (u64, u64) {
    let combined_height = (tree_geometry.packed_inode_tree_levels.len() + tree_geometry.packed_container_index_levels.len()) as u64;
    (combined_height + 1, combined_height * NODE_BYTES + UNIT_BYTES)
}

fn ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_lines: Vec<String> = Vec::new();
    output_lines.push(emitter.emit_raw(&format!(
        "name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} packed_hdr={PACKED_UNIT_HEADER_BYTES} inode_rec={INODE_RECORD_BYTES} \
         child_ptr={CHILD_POINTER_BYTES} w={COLUMNS_PER_WRITE} per_publish_const={} ident_value={IDENTITY_VALUE_BYTES} cont_key={CONTAINER_KEY_BYTES} internal_entry={INODE_INTERNAL_ENTRY} leaf_records={INODE_LEAF_RECORDS} model=arithmetic file_ops=0",
        per_publish_constant_bytes()
    )));

    for &node_header in NODE_HEADERS.iter().chain(NODE_HEADERS_TODAY.iter()) {
        for &inode_count in INODE_COUNTS.iter() {
            let tree_geometry = geometry(inode_count, node_header);
            output_lines.push(emitter.emit_raw(&format!(
                "name=geom header={node_header} inodes={inode_count} a_leaf_fanout={} a_height={} a_levels={:?} \
                 b_per_container={} b_containers={} b_index_height={} b_index_levels={:?} b_inode_height={}",
                tree_geometry.in_leaf_records_per_leaf,
                tree_geometry.in_leaf_tree_levels.len(),
                tree_geometry.in_leaf_tree_levels,
                tree_geometry.packed_records_per_container,
                tree_geometry.packed_container_count,
                tree_geometry.packed_container_index_levels.len(),
                tree_geometry.packed_container_index_levels,
                tree_geometry.packed_inode_tree_levels.len()
            )));
            // 判据 1 / 2：单次与批量
            for &update_count in UPDATE_COUNTS.iter() {
                let in_leaf_bytes = write_in_leaf_bytes(&tree_geometry, update_count);
                let packed_bytes = write_packed_bytes(&tree_geometry, update_count, UNIT_BYTES, true);
                let leaf_as_container_bytes = write_leaf_as_container_bytes(&tree_geometry, update_count);
                output_lines.push(emitter.emit_raw(&format!(
                    "name=write header={node_header} inodes={inode_count} k={update_count} a_bytes={in_leaf_bytes:.0} b_bytes={packed_bytes:.0} b_over_a={:.3} \
                     a_per_update={:.0} b_per_update={:.0} c_bytes={leaf_as_container_bytes:.0} c_over_a={:.3} const_per_publish={}",
                    ratio(packed_bytes, in_leaf_bytes),
                    in_leaf_bytes / update_count as f64,
                    packed_bytes / update_count as f64,
                    ratio(leaf_as_container_bytes, in_leaf_bytes),
                    per_publish_constant_bytes()
                )));
            }
            // 聚簇格
            let in_leaf_clustered_bytes = write_in_leaf_clustered_bytes(&tree_geometry);
            let packed_clustered_bytes = write_packed_clustered_bytes(&tree_geometry);
            output_lines.push(emitter.emit_raw(&format!(
                "name=write_clustered header={node_header} inodes={inode_count} a_bytes={in_leaf_clustered_bytes:.0} b_bytes={packed_clustered_bytes:.0} b_over_a={:.3}",
                ratio(packed_clustered_bytes, in_leaf_clustered_bytes)
            )));
            // 判据 3：读
            let (in_leaf_reads, in_leaf_read_bytes) = file_status_reads_in_leaf(&tree_geometry);
            let (packed_reads, packed_read_bytes) = file_status_reads_packed(&tree_geometry);
            let (leaf_as_container_reads, leaf_as_container_read_bytes) = file_status_reads_leaf_as_container(&tree_geometry);
            output_lines.push(emitter.emit_raw(&format!(
                "name=stat header={node_header} inodes={inode_count} a_reads={in_leaf_reads} a_bytes={in_leaf_read_bytes} b_reads={packed_reads} b_bytes={packed_read_bytes} b_over_a={:.3} c_reads={leaf_as_container_reads} c_bytes={leaf_as_container_read_bytes} c_over_a={:.3} c_internal_height={}",
                ratio(packed_read_bytes as f64, in_leaf_read_bytes as f64),
                ratio(leaf_as_container_read_bytes as f64, in_leaf_read_bytes as f64),
                tree_geometry.leaf_as_container_internal_levels.len()
            )));
        }
    }
    // 判据 4：爆炸半径（与节点头无关的那一半按 58 报）
    let tree_geometry = geometry(1_000_000, 58);
    output_lines.push(emitter.emit_raw(&format!(
        "name=blast a_records_per_leaf={} b_records_per_container={} b_over_a={:.3}",
        tree_geometry.in_leaf_records_per_leaf,
        tree_geometry.packed_records_per_container,
        ratio(tree_geometry.packed_records_per_container as f64, tree_geometry.in_leaf_records_per_leaf as f64)
    )));
    // 判据 5：先前估算
    let tree_geometry = geometry(1_000_000, 58);
    let measured_single_update_ratio = ratio(write_packed_bytes(&tree_geometry, 1, UNIT_BYTES, true), write_in_leaf_bytes(&tree_geometry, 1));
    output_lines.push(emitter.emit_raw(&format!(
        "name=prior_estimate claimed=3.0 measured_single_update_ratio_1e6={measured_single_update_ratio:.3} claim_holds={}",
        u8::from(measured_single_update_ratio >= 3.0)
    )));
    // 对照
    let is_positive_control_same_ruler = write_packed_bytes(&tree_geometry, 1, NODE_BYTES, false) == (NODE_BYTES * COLUMNS_PER_WRITE) as f64;
    output_lines.push(emitter.emit_raw(&format!(
        "name=controls positive_same_ruler={} negative_k0_a={:.0} negative_k0_b={:.0} saturation_1e6_leaf_k_huge={:.1} leaf_count={}",
        u8::from(is_positive_control_same_ruler),
        write_in_leaf_bytes(&tree_geometry, 0),
        write_packed_bytes(&tree_geometry, 0, UNIT_BYTES, true),
        touched(tree_geometry.in_leaf_tree_levels[0], tree_geometry.in_leaf_tree_levels[0] * 100),
        tree_geometry.in_leaf_tree_levels[0]
    )));

    for output_line in &output_lines {
        println!("{output_line}");
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(NODE_BYTES, 16384, "D8 已定项 2");
        assert_eq!(UNIT_BYTES, 32768, "D4 已定项 7");
        assert_eq!(PACKED_UNIT_HEADER_BYTES, 107, "D18 已定项 11（C113 定案 2026-09-05 加写序 10；C288 ① 2026-09-12 加出生序号 4）");
        assert_eq!(CHILD_POINTER_BYTES, 59, "D19 已定项 4");
        assert_eq!(COLUMNS_PER_WRITE, 2, "D2 已定项 9");
        assert_eq!(JOURNAL_RECORD_BYTES, 4096, "D23 已定项 12");
        assert_eq!(IDENTITY_VALUE_BYTES, 8 + 2 + 8 + 8 + 1);
        assert_eq!(CONTAINER_KEY_BYTES, 8 + 2 + 8 + 8);
    }

    /// 两笔补账的算术钉死：58 / 67 / 76 各加写序 10 与预留 28 = 96 / 105 / 114。
    /// **写成加法不写减法**——变异把常量改大时减法会编译期溢出、被记成无效变异。
    #[test]
    fn today_headers_are_absolute() {
        assert_eq!(RESERVED_HEADER_BYTES, 28);
        assert_eq!(WRITE_ORDER, 10);
        assert_eq!(NODE_HEADERS_TODAY, [96, 105, 114]);
        assert_eq!(NODE_HEADERS_TODAY[0], NODE_HEADERS[0] + 38);
    }

    /// 补账之后那一档（头 114）的绝对值：内部扇出确实掉一格，而结论一格不动。
    /// ⚠️ **这不是「头宽不影响结论」**：内部扇出 243 → 242 是变了的，
    /// 只是被树高的向上取整吸收掉（⌈8621 / 243⌉ = ⌈8621 / 242⌉ = 36）。
    /// 按 `.claude/rules/mutation-sampling.md`，把它记成「不敏感」要有这条把中间量钉住的断言，
    /// 否则下次有人改了扇出公式也不会有任何东西报警。
    #[test]
    fn geometry_at_today_header_is_absolute() {
        let geometry_header_58 = geometry(1_000_000, NODE_HEADERS[0]);
        let geometry_header_114 = geometry(1_000_000, NODE_HEADERS_TODAY[2]);
        assert_eq!(fanout(NODE_BYTES, 58, INODE_KEY + CHILD_POINTER_BYTES), 243);
        assert_eq!(fanout(NODE_BYTES, 114, INODE_KEY + CHILD_POINTER_BYTES), 242, "中间量掉一格");
        assert_eq!(fanout(NODE_BYTES, 114, CONTAINER_KEY_BYTES + CHILD_POINTER_BYTES), 191, "(16384 − 114) / 85");
        assert_eq!(geometry_header_58.in_leaf_tree_levels, geometry_header_114.in_leaf_tree_levels, "树高被向上取整吸收");
        assert_eq!(geometry_header_114.in_leaf_tree_levels, vec![8621, 36, 1]);
        assert_eq!(geometry_header_114.packed_container_count, 4292);
        assert_eq!(geometry_header_114.packed_records_per_container, 233);
    }

    /// 几何的绝对值（头 58）：与 E98 的 116 / 243 / 233 对得上，容器索引扇出 192。
    #[test]
    fn geometry_is_absolute() {
        let tree_geometry = geometry(1_000_000, 58);
        assert_eq!(tree_geometry.in_leaf_records_per_leaf, 116, "E98");
        assert_eq!(tree_geometry.packed_records_per_container, 233, "E98 / E102");
        assert_eq!(fanout(NODE_BYTES, 58, INODE_KEY + CHILD_POINTER_BYTES), 243, "E98 内部扇出");
        assert_eq!(fanout(NODE_BYTES, 58, CONTAINER_KEY_BYTES + CHILD_POINTER_BYTES), 192, "(16384 − 58) / 85");
        assert_eq!(fanout(NODE_BYTES, 58, INODE_KEY + IDENTITY_VALUE_BYTES), 466, "(16384 − 58) / 35");
        // 1e6：住索引叶 8621 叶 → 36 → 1（高 3）；打包 4292 容器，索引 23 叶 → 1（高 2）
        assert_eq!(tree_geometry.in_leaf_tree_levels, vec![8621, 36, 1]);
        assert_eq!(tree_geometry.packed_container_count, 4292);
        assert_eq!(tree_geometry.packed_container_index_levels, vec![23, 1]);
        assert_eq!(tree_geometry.packed_inode_tree_levels, vec![2146, 9, 1]);
    }

    /// **判据 1 的绝对值**：单次更新，三个 N 逐格钉死（头 58），比值 1.5 / 1.333 / 1.25。
    #[test]
    fn criterion1_single_update_bytes_are_pinned() {
        let geometry_ten_thousand_inodes = geometry(10_000, 58);
        assert_eq!(write_in_leaf_bytes(&geometry_ten_thousand_inodes, 1), 65536.0, "2 个节点 × 16384 × 2");
        assert_eq!(write_packed_bytes(&geometry_ten_thousand_inodes, 1, UNIT_BYTES, true), 98304.0, "容器 65536 + 索引 1 节点 32768");
        let geometry_million_inodes = geometry(1_000_000, 58);
        assert_eq!(write_in_leaf_bytes(&geometry_million_inodes, 1), 98304.0, "3 个节点");
        assert_eq!(write_packed_bytes(&geometry_million_inodes, 1, UNIT_BYTES, true), 131072.0, "容器 65536 + 索引 2 节点 65536");
        let geometry_hundred_million_inodes = geometry(100_000_000, 58);
        assert_eq!(write_in_leaf_bytes(&geometry_hundred_million_inodes, 1), 131072.0, "4 个节点");
        assert_eq!(write_packed_bytes(&geometry_hundred_million_inodes, 1, UNIT_BYTES, true), 163840.0, "容器 65536 + 索引 3 节点 98304");
        assert!((ratio(write_packed_bytes(&geometry_million_inodes, 1, UNIT_BYTES, true), write_in_leaf_bytes(&geometry_million_inodes, 1)) - 4.0 / 3.0).abs() < 1e-9);
    }

    /// **判据 2**：批量散开时打包侧每条更新的边际代价趋向一个容器（32 KiB × w），
    /// 住索引叶趋向一片叶（16 KiB × w）；全量重写时两者相同（同一批 140 字节记录）。
    #[test]
    fn criterion2_batch_amortisation_and_saturation() {
        let tree_geometry = geometry(1_000_000, 58);
        let in_leaf_bytes_at_1000_updates = write_in_leaf_bytes(&tree_geometry, 1000);
        let packed_bytes_at_1000_updates = write_packed_bytes(&tree_geometry, 1000, UNIT_BYTES, true);
        let packed_over_in_leaf_ratio = ratio(packed_bytes_at_1000_updates, in_leaf_bytes_at_1000_updates);
        assert!(packed_over_in_leaf_ratio > 1.5 && packed_over_in_leaf_ratio < 2.0, "k=1000 时比值在 (1.5, 2)：{packed_over_in_leaf_ratio}");
        // 饱和：k 远大于层内节点数 ⇒ 碰到的节点数收敛到该层节点数
        let saturated_touched_nodes = touched(tree_geometry.in_leaf_tree_levels[0], tree_geometry.in_leaf_tree_levels[0] * 100);
        let saturation_relative_error = (saturated_touched_nodes - tree_geometry.in_leaf_tree_levels[0] as f64).abs() / (tree_geometry.in_leaf_tree_levels[0] as f64);
        assert!(saturation_relative_error < 0.005, "饱和相对误差 {saturation_relative_error}");
        // 全量重写：两条形态都写出全部记录所在的容器 ⇒ 字节数量级相同（都 ≈ 1e6 × 140 × w）
        let full_rewrite_update_count = 100_000_000;
        let full_rewrite_in_leaf_bytes = write_in_leaf_bytes(&tree_geometry, full_rewrite_update_count);
        let full_rewrite_packed_bytes = write_packed_bytes(&tree_geometry, full_rewrite_update_count, UNIT_BYTES, true);
        assert!((full_rewrite_packed_bytes / full_rewrite_in_leaf_bytes - 1.0).abs() < 0.02, "全量重写比 ≈ 1：{}", full_rewrite_packed_bytes / full_rewrite_in_leaf_bytes);
        // 聚簇格：1e6 下 3 节点 vs 容器 + 2 节点
        assert_eq!(write_in_leaf_clustered_bytes(&tree_geometry), 98304.0);
        assert_eq!(write_packed_clustered_bytes(&tree_geometry), 131072.0);
    }

    /// **判据 3 的绝对值**：stat 读，1e6 头 58：3 次 vs 6 次，49152 vs 114688 字节。
    #[test]
    fn criterion3_file_status_reads_are_pinned() {
        let tree_geometry = geometry(1_000_000, 58);
        assert_eq!(file_status_reads_in_leaf(&tree_geometry), (3, 49152));
        assert_eq!(file_status_reads_packed(&tree_geometry), (6, 5 * 16384 + 32768));
    }

    /// **判据 4**：爆炸半径 116 vs 233（E98 口径）。
    #[test]
    fn criterion4_blast_radius_matches_e98() {
        let tree_geometry = geometry(1_000_000, 58);
        assert_eq!(tree_geometry.in_leaf_records_per_leaf, 116);
        assert_eq!(tree_geometry.packed_records_per_container, 233);
    }

    /// **判据 5 + 反向接受条款**：第二轮正推腿的「≈ 3 倍」不成立——它只数了叶子。
    #[test]
    fn criterion5_prior_three_times_estimate_does_not_hold() {
        for &inode_count in INODE_COUNTS.iter() {
            let tree_geometry = geometry(inode_count, 58);
            let single_update_ratio = ratio(write_packed_bytes(&tree_geometry, 1, UNIT_BYTES, true), write_in_leaf_bytes(&tree_geometry, 1));
            assert!(single_update_ratio < 3.0, "N={inode_count} 单次更新比 {single_update_ratio} 应 < 3");
            assert!(single_update_ratio >= 1.25 - 1e-9, "N={inode_count} 单次更新比 {single_update_ratio} 应 ≥ 1.25");
        }
        // 只数叶子的那个算法：96 KiB / 32 KiB = 3——正是估算的来源
        assert_eq!(((UNIT_BYTES + NODE_BYTES) * COLUMNS_PER_WRITE) as f64 / (NODE_BYTES * COLUMNS_PER_WRITE) as f64, 3.0);
    }

    /// 阳性对照：容器取 16384 且不计索引 ⇒ 与住索引叶的**叶那一层**同代价（同一把尺子）。
    #[test]
    fn positive_control_same_ruler() {
        let tree_geometry = geometry(1_000_000, 58);
        assert_eq!(write_packed_bytes(&tree_geometry, 1, NODE_BYTES, false), (NODE_BYTES * COLUMNS_PER_WRITE) as f64);
        assert_eq!(touched(tree_geometry.in_leaf_tree_levels[0], 1) * (NODE_BYTES * COLUMNS_PER_WRITE) as f64, (NODE_BYTES * COLUMNS_PER_WRITE) as f64);
    }

    /// 阴性对照：k = 0 ⇒ 0；不合法几何 ⇒ 空。
    #[test]
    fn negative_controls() {
        let tree_geometry = geometry(1_000_000, 58);
        assert_eq!(write_in_leaf_bytes(&tree_geometry, 0), 0.0);
        assert_eq!(write_packed_bytes(&tree_geometry, 0, UNIT_BYTES, true), 0.0);
        assert!(tree_levels(0, 116, 243).is_empty());
        assert_eq!(fanout(NODE_BYTES, NODE_BYTES, 140), 0);
    }

    /// **第三臂「叶即容器」的绝对值**（头 58）：写与路 ① 逐格相同（内部层数 = 容器索引树高），
    /// 读少了 inode 树那一趟：1e6 是 2 次内部 + 1 次单元 = 65536 字节（1.333 倍），1e8 是 81920（1.25 倍）。
    #[test]
    fn third_arm_leaf_as_container_is_pinned() {
        assert_eq!(INODE_INTERNAL_ENTRY, 93);
        assert_eq!(INODE_INTERNAL_ENTRY, INODE_KEY + CONTAINER_KEY_BYTES + CHILD_POINTER_BYTES, "93 = 8 + 26 + 59");
        assert_eq!(INODE_LEAF_RECORDS, fanout(UNIT_BYTES, PACKED_UNIT_HEADER_BYTES, INODE_RECORD_BYTES), "233 = ⌊(32768 − 107) / 140⌋");
        assert_eq!(fanout(NODE_BYTES, 58, INODE_INTERNAL_ENTRY), 175, "(16384 − 58) / 93");
        assert_eq!(fanout(NODE_BYTES, 76, INODE_INTERNAL_ENTRY), 175, "基础头三档（E73 的不带区间下界）里最大的 76");
        assert_eq!(fanout(NODE_BYTES, 92, INODE_INTERNAL_ENTRY), 175, "基础头 76 加 16 字节 key 区间 = 92，扇出仍不掉格");
        assert_eq!(fanout(NODE_BYTES, 109, INODE_INTERNAL_ENTRY), 175, "头到 109 仍是 175");
        assert_eq!(fanout(NODE_BYTES, 110, INODE_INTERNAL_ENTRY), 174, "头 110 才掉一格");
        let geometry_million_inodes = geometry(1_000_000, 58);
        assert_eq!(geometry_million_inodes.leaf_as_container_internal_levels, vec![25, 1]);
        assert_eq!(write_leaf_as_container_bytes(&geometry_million_inodes, 1), 131072.0);
        assert_eq!(file_status_reads_leaf_as_container(&geometry_million_inodes), (3, 65536));
        // k ≥ 10 时第三臂比路 ① 贵的正好是内部层多出来的节点：[25, 1] 对容器索引的 [23, 1]，多 2 个节点 × 16384 × 2
        let extra_bytes_of_leaf_as_container = write_leaf_as_container_bytes(&geometry_million_inodes, 1000) - write_packed_bytes(&geometry_million_inodes, 1000, UNIT_BYTES, true);
        assert!((extra_bytes_of_leaf_as_container - 65536.0).abs() < 1.0, "第三臂多出的字节应恰为 2 个节点，实得 {extra_bytes_of_leaf_as_container}");
        let geometry_ten_thousand_inodes = geometry(10_000, 58);
        assert_eq!(geometry_ten_thousand_inodes.leaf_as_container_internal_levels, vec![1]);
        assert_eq!(write_leaf_as_container_bytes(&geometry_ten_thousand_inodes, 1), 98304.0);
        assert_eq!(file_status_reads_leaf_as_container(&geometry_ten_thousand_inodes), (2, 49152));
        let geometry_hundred_million_inodes = geometry(100_000_000, 58);
        assert_eq!(geometry_hundred_million_inodes.leaf_as_container_internal_levels, vec![2453, 15, 1]);
        assert_eq!(write_leaf_as_container_bytes(&geometry_hundred_million_inodes, 1), 163840.0);
        assert_eq!(file_status_reads_leaf_as_container(&geometry_hundred_million_inodes), (4, 81920));
        // 与路 ① 的写逐格相同，读少一趟
        for &inode_count in INODE_COUNTS.iter() {
            let tree_geometry = geometry(inode_count, 58);
            assert_eq!(write_leaf_as_container_bytes(&tree_geometry, 1), write_packed_bytes(&tree_geometry, 1, UNIT_BYTES, true));
            assert!(file_status_reads_leaf_as_container(&tree_geometry).1 < file_status_reads_packed(&tree_geometry).1);
        }
    }

    /// 三档节点头同向：比值都落在 [1.25, 2)。
    #[test]
    fn header_tiers_do_not_flip_the_direction() {
        for &node_header in NODE_HEADERS.iter() {
            for &inode_count in INODE_COUNTS.iter() {
                let tree_geometry = geometry(inode_count, node_header);
                let single_update_ratio = ratio(write_packed_bytes(&tree_geometry, 1, UNIT_BYTES, true), write_in_leaf_bytes(&tree_geometry, 1));
                assert!(single_update_ratio >= 1.25 - 1e-9 && single_update_ratio < 2.0, "头 {node_header} N={inode_count} 比值 {single_update_ratio}");
            }
        }
    }
}
