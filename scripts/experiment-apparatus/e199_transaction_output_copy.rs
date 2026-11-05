//! E199（发布照抄上一版的代价）第一段：纯算术计数模型。
//!
//! 跑前登记：`research/prompts/e199-preregistration-r4.md`（第四份；它没写到的照第三份 `e199-preregistration-r3.md`，
//! 第三份写「照留」的第二份、第一份各节，按它点名的那几节读）。第四份写死的五样：乙2 与乙1 阳性对照的退化形态、乙1 的照抄字节、
//! 甲丙丁三式常驻峰值（丁加残余式）、10⁶ 次 unlink 累计的次序（第四份第五节）。
//! 臂：甲-B2后（真实基线）、甲-今天（对照）、乙2 持久结构、乙1 每片引用计数（附带，只报数）、丙 按值交进来加撤回日志、丁 只记差量加压实。
//! 每次发布的照抄字节、新装字节、现算重建字节照登记第五节的式子算；判据照第六节；锚点照第七节，写成单测。
//! 独立手写，`crates/` 一行都不引；第三节的文件:行只当照抄点清单。
//!
//! 用法：不带参数，产物就是标准输出。线程数：取登记的重活默认（`.claude/heavy-thread-default`），
//! 只用在「10⁶ 次 unlink 累计」那一段：按发布序号切片并行、按片的次序相加，产物与线程数无关
//! （单测 `cumulative_unlinks_do_not_depend_on_the_thread_count`）。别的取样点每点只做 O(层数) 次算术，单线程。

use std::io::Write;
use std::mem::size_of;

// 写成 `#[path] mod`：单元闭包的键不收条目级 `include!`（同 E178（叶与记录的死亡计数））。
#[path = "heavy_threads_default/threads.rs"]
mod heavy_threads_default;
use heavy_threads_default::default_heavy_threads;

// ── 出自被测条款的常量（登记第五节「模型常量」第一份那张表；第七节「出自被测条款本身的」） ──

/// 容器单元字节（D18（块里携带什么信息） 已定项 24 第 2 条「一个 32 KiB 容器」）。
const UNIT_BYTES: u64 = 32768;
/// 节点字节（D8（核心索引结构） 已定项 2）。
const NODE_BYTES: u64 = 16384;
/// 码 2 头 115 加两个 key 宽 8，条目 120（D18（块里携带什么信息） 已定项 24 第 1 条）。
const CODE_TWO_HEADER_BYTES: u64 = 115;
const CODE_TWO_KEY_BYTES: u64 = 8;
const CODE_TWO_ENTRY_BYTES: u64 = 120;
/// 码 2 节点扇出：⌊(16384 − (115 + 2 × 8)) ÷ 120⌋。
const CODE_TWO_FANOUT: u64 = 135;
/// 码 3 单元的记录区起点（D18（块里携带什么信息） 第 336 行 cap(136, 140) = 233 那一式的 136）。
const CODE_THREE_RECORD_AREA_START: u64 = 136;
/// 一片墓碑、谱系、退役容器与一片 inode 叶装几条（D18（块里携带什么信息） 已定项 24 第 2、5 条，第 303 行）。
const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
const LINEAGE_CONTAINER_RECORDS: u64 = 1359;
const RETIREMENT_CONTAINER_RECORDS: u64 = 959;
const INODE_LEAF_RECORDS: u64 = 233;
/// 盘上记录宽：墓碑 40、谱系 24、退役 34、inode 140（只用在第七节那四条单元装满的加法断言里）。
const TOMBSTONE_RECORD_DISK_BYTES: u64 = 40;
const LINEAGE_RECORD_DISK_BYTES: u64 = 24;
const RETIREMENT_RECORD_DISK_BYTES: u64 = 34;
const INODE_RECORD_DISK_BYTES: u64 = 140;

// ── 取实现值的模型常量（不出自被测条款；登记第五节第二份那张表与第三份加的两行） ──

/// 多层码 2 树的叶容量、内部容量、key 字节与 key 字段数（`transaction.rs:2679` 注释；`code_two_tree.rs:37`、`40`、`44`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct MultiLevelTreeKind {
    leaf_entries: u64,
    internal_entries: u64,
    key_bytes: u64,
    key_fields: u64,
}
const ACCOUNTING_TREE: MultiLevelTreeKind = MultiLevelTreeKind { leaf_entries: 477, internal_entries: 150, key_bytes: 22, key_fields: 4 };
const DEADLIST_TREE: MultiLevelTreeKind = MultiLevelTreeKind { leaf_entries: 376, internal_entries: 125, key_bytes: 43, key_fields: 7 };
const CENTRAL_MAPPING_TREE: MultiLevelTreeKind = MultiLevelTreeKind { leaf_entries: 294, internal_entries: 143, key_bytes: 27, key_fields: 5 };
/// 一条映射条目的字节（`transaction.rs` 注释「映射条目 55」）。
const MAPPING_ENTRY_BYTES: u64 = 55;
/// 分配记录树：叶罩 812 槽；内部节点头 135、内部条目 96，扇出 169（`singlefs-format/src/lib.rs`）。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
const ALLOCATION_RECORD_TREE_INTERNAL_HEADER_BYTES: u64 = 135;
const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 记账条目数：池级 3 行、两块盘各 6 行、台阶行 1（`transaction.rs:7915`、`7955` 的 `ALL`）。
const POOL_ACCOUNTING_ROWS: u64 = 3;
const PER_DEVICE_ACCOUNTING_ROWS: u64 = 6;
const DEVICE_COUNT: u64 = 2;
const STEP_ACCOUNTING_ROWS: u64 = 1;
const ACCOUNTING_ENTRIES: u64 = POOL_ACCOUNTING_ROWS + DEVICE_COUNT * PER_DEVICE_ACCOUNTING_ROWS + STEP_ACCOUNTING_ROWS;
/// 实例表片数（模型选择，一片 32 KiB 单元，照抄）与固定点单元数（16 个 16 KiB 节点，每次发布全部重写）。
const INSTANCE_TABLE_UNITS: u64 = 1;
const FIXED_POINT_NODES: u64 = 16;
/// 指向码 1 数据单元的指针 88：头部 50 + 位置条目 14 × 2 + 写序 10（`crates/singlefs-core/src/pointer.rs:828` 注释）。
const DATA_POINTER_HEADER_BYTES: u64 = 50;
const POSITION_ENTRY_BYTES: u64 = 14;
const DATA_POINTER_WRITE_ORDER_BYTES: u64 = 10;
const DATA_POINTER_BYTES: u64 = 88;

// ── 内存宽度：装置自己的类型，`size_of` 回比（登记第五节；读法第 3 条：不是 `crates/` 里 Rust 类型的真宽） ──

const VECTOR_HEADER_BYTES: u64 = size_of::<Vec<u8>>() as u64;
/// 墓碑记录 5 个 u64；谱系记录 3 个 u64；退役记录 u64 + u16 + 三个 u64（按 8 对齐）；inode 记录 8 个 u64；分配记录 5 个 u64。
const TOMBSTONE_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 5]>() as u64;
const LINEAGE_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 3]>() as u64;
const RETIREMENT_RECORD_MEMORY_BYTES: u64 = size_of::<(u64, u16, [u64; 3])>() as u64;
const INODE_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 8]>() as u64;
const ALLOCATION_RECORD_MEMORY_BYTES: u64 = size_of::<[u64; 5]>() as u64;
/// 容器外壳 = 身份 32 + `Vec` 头 24；单元外壳 = 槽号 8 + 角色 16 + `Vec` 头 24。
const CONTAINER_SHELL_BYTES: u64 = size_of::<([u64; 4], Vec<u8>)>() as u64;
const UNIT_SHELL_BYTES: u64 = size_of::<(u64, [u64; 2], Vec<u8>)>() as u64;
/// 内存里一个容器单元、一个节点单元（字节加外壳）。
const CONTAINER_UNIT_MEMORY_BYTES: u64 = UNIT_BYTES + UNIT_SHELL_BYTES;
const NODE_UNIT_MEMORY_BYTES: u64 = NODE_BYTES + UNIT_SHELL_BYTES;
/// 容器树形状：每个节点 8（子数），每层一个 `Vec` 头 24。
const SHAPE_NODE_BYTES: u64 = size_of::<u64>() as u64;
const SHAPE_LEVEL_BYTES: u64 = size_of::<Vec<u64>>() as u64;
/// `mapped_units` 一项：身份 16 + `Vec` 头 24 + 映射 key；带位置项一项再加两条位置项。
const UNIT_IDENTITY_BYTES: u64 = size_of::<[u64; 2]>() as u64;
const MAPPED_UNIT_ENTRY_BYTES: u64 = UNIT_IDENTITY_BYTES + VECTOR_HEADER_BYTES + CENTRAL_MAPPING_TREE.key_bytes;
const MAPPED_UNIT_WITH_LOCATIONS_BYTES: u64 = MAPPED_UNIT_ENTRY_BYTES + 2 * POSITION_ENTRY_BYTES;
/// key 字节一项、叶条目一项（`Vec` 头加字节）。
const MAPPING_KEY_BYTES_ENTRY_BYTES: u64 = VECTOR_HEADER_BYTES + CENTRAL_MAPPING_TREE.key_bytes;
const MAPPING_LEAF_ENTRY_BYTES: u64 = VECTOR_HEADER_BYTES + MAPPING_ENTRY_BYTES;
/// 多层码 2 树形状：每个节点 = 位置 8 + 孩子表的 `Vec` 头 24；每个孩子 = key 结构 + 位置 8。
const MULTI_LEVEL_SHAPE_NODE_BYTES: u64 = size_of::<(u64, Vec<u64>)>() as u64;
const MULTI_LEVEL_CHILD_POSITION_BYTES: u64 = size_of::<u64>() as u64;
/// 分配记录树节点身份 16；`deleted_inodes` 一个号 32（`Vec` 头 24 + 号 8）。
const ALLOCATION_TREE_NODE_IDENTITY_BYTES: u64 = size_of::<[u64; 2]>() as u64;
const DELETED_INODE_NUMBER_BYTES: u64 = VECTOR_HEADER_BYTES + size_of::<u64>() as u64;
/// 第一个文件照抄的数据单元抄进 `data_units` 那一份是裸 `Vec<u8>`：字节加 `Vec` 头，没有单元外壳。
const CARRIED_DATA_UNIT_BYTES: u64 = UNIT_BYTES + VECTOR_HEADER_BYTES;
/// 装置自己的数据指针类型 `[u8; 88]`。
const DATA_POINTER_MEMORY_BYTES: u64 = size_of::<[u8; 88]>() as u64;
/// 乙1 换成引用计数之后照抄的一个指针（第二份第五节「× 8（只抄指针）」）。
const REFERENCE_COUNTED_POINTER_BYTES: u64 = size_of::<std::sync::Arc<[u8; 32768]>>() as u64;

/// key 结构的内存宽 W_ck = 字段表 `Vec` 头 24 + 8 × 字段数 + 字节 `Vec` 头 24 + key 字节（映射 115、记账 102、deadlist 147）。
const fn key_structure_bytes(kind: MultiLevelTreeKind) -> u64 {
    size_of::<Vec<u64>>() as u64 + 8 * kind.key_fields + VECTOR_HEADER_BYTES + kind.key_bytes
}
/// 映射条目表一项：key 结构 + 条目 `Vec` 头 + 条目字节（194）。
const MAPPING_ENTRY_TABLE_BYTES: u64 = key_structure_bytes(CENTRAL_MAPPING_TREE) + VECTOR_HEADER_BYTES + MAPPING_ENTRY_BYTES;

// ── 几何（登记第五节「几何与派生量」；第一段与第二段同一套） ──

/// 映射树与分配记录树被碰到的叶取哪一档（读法第 6 条）：分散 = 改动的 key 依次落在不同的叶，集中 = 连号落在最右那几片。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TouchedLeaves {
    Spread,
    Focused,
}

impl TouchedLeaves {
    fn label(self) -> &'static str {
        match self {
            TouchedLeaves::Spread => "spread",
            TouchedLeaves::Focused => "focused",
        }
    }
}

/// 一组旋钮 G = (u, a_T, R_X, a_X, R_L, a_L, I, D, s, φ, 碰叶档)，另带乙2 的 b、g。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Knobs {
    tombstone_records: u64,
    tombstones_added: u64,
    retirement_records: u64,
    retirements_added: u64,
    lineage_records: u64,
    lineage_records_added: u64,
    live_inodes: u64,
    file_data_units: u64,
    settle_rounds: u64,
    mapping_leaves_half_full: bool,
    touched_leaves: TouchedLeaves,
    persistent_fanout: u64,
    persistent_leaf_block: u64,
}

impl Knobs {
    /// G0：u = 10⁶、a_T = 1、R_X = 10³、a_X = 0、R_L = 16、a_L = 0、I = 10³、D = 0、s = 4、φ = 1、分散档；乙2 b = 32、g = 64。
    fn main_geometry() -> Knobs {
        Knobs {
            tombstone_records: 1_000_000,
            tombstones_added: 1,
            retirement_records: 1000,
            retirements_added: 0,
            lineage_records: 16,
            lineage_records_added: 0,
            live_inodes: 1000,
            file_data_units: 0,
            settle_rounds: 4,
            mapping_leaves_half_full: false,
            touched_leaves: TouchedLeaves::Spread,
            persistent_fanout: 32,
            persistent_leaf_block: 64,
        }
    }

    /// G-small（方向相反的点）：u = 10、R_X = 0、R_L = 0、I = 1、s = 1，其余取 G0。
    fn small_geometry() -> Knobs {
        Knobs { tombstone_records: 10, retirement_records: 0, lineage_records: 0, live_inodes: 1, settle_rounds: 1, ..Knobs::main_geometry() }
    }
}

/// 一棵容器树（墓碑、退役、谱系）这一版与下一版的形状：左满右开，只有最右那条路不满。
#[derive(Clone, PartialEq, Eq, Debug)]
struct ContainerTree {
    records: u64,
    added: u64,
    containers: u64,
    levels: Vec<u64>,
    nodes: u64,
    height: u64,
    fill: u64,
    open_container_rewritten: u64,
    new_containers: u64,
    rewritten_index_nodes: u64,
    appends: bool,
    records_after: u64,
    containers_after: u64,
    levels_after: Vec<u64>,
    nodes_after: u64,
    height_after: u64,
}

/// 按扇出往上建的每层节点数，从下往上；count = 0 时没有层。
fn levels_by_fanout(count: u64, bottom_capacity: u64, fanout: u64) -> Vec<u64> {
    if count == 0 {
        return Vec::new();
    }
    let mut levels = vec![count.div_ceil(bottom_capacity)];
    while let Some(&top) = levels.last() {
        if top <= 1 {
            break;
        }
        levels.push(top.div_ceil(fanout));
    }
    levels
}

/// 按碰到的叶数往上数重写的节点：x_1 = 碰到的叶，x_ℓ = min(n_ℓ, x_{ℓ−1})。
fn rewritten_nodes_by_level(levels: &[u64], touched_leaves: u64) -> u64 {
    let mut reaching = touched_leaves;
    let mut rewritten = 0;
    for &nodes_at_level in levels {
        reaching = reaching.min(nodes_at_level);
        rewritten += reaching;
    }
    rewritten
}

impl ContainerTree {
    fn new(records: u64, added: u64, capacity: u64, appends_every_publish: bool) -> ContainerTree {
        let containers = records.div_ceil(capacity);
        let levels = levels_by_fanout(containers, CODE_TWO_FANOUT, CODE_TWO_FANOUT);
        let fill = if containers > 0 { records - (containers - 1) * capacity } else { 0 };
        let room = if containers > 0 { capacity - fill } else { 0 };
        let open_container_rewritten = u64::from(added > 0 && containers > 0 && room > 0);
        let new_containers = if added > 0 && added > room { (added - room).div_ceil(capacity) } else { 0 };
        let leftmost_touched_container = if open_container_rewritten == 1 { containers - 1 } else { containers };
        let mut rewritten_index_nodes = 0;
        if added > 0 {
            let mut span = CODE_TWO_FANOUT;
            for &nodes_at_level in &levels {
                rewritten_index_nodes += nodes_at_level.saturating_sub(leftmost_touched_container / span);
                span = span.saturating_mul(CODE_TWO_FANOUT);
            }
        }
        let records_after = records + added;
        let containers_after = records_after.div_ceil(capacity);
        let levels_after = levels_by_fanout(containers_after, CODE_TWO_FANOUT, CODE_TWO_FANOUT);
        ContainerTree {
            records,
            added,
            containers,
            nodes: levels.iter().sum(),
            height: levels.len() as u64,
            levels,
            fill,
            open_container_rewritten,
            new_containers,
            rewritten_index_nodes,
            appends: appends_every_publish || added > 0,
            records_after,
            containers_after,
            nodes_after: levels_after.iter().sum(),
            height_after: levels_after.len() as u64,
            levels_after,
        }
    }

    fn shape_bytes(&self) -> u64 {
        shape_bytes(self.nodes, self.height)
    }
}

/// 容器树形状 shape(N, h) = 8N + 24h。
fn shape_bytes(nodes: u64, height: u64) -> u64 {
    SHAPE_NODE_BYTES * nodes + SHAPE_LEVEL_BYTES * height
}

/// 多层码 2 树形状 shape_ml(E, N) = 32N + E·W_ck + (N − 1)(W_ck + 8)；N = 0 时 0。
fn multi_level_shape_bytes(entries: u64, nodes: u64, kind: MultiLevelTreeKind) -> u64 {
    if nodes == 0 {
        return 0;
    }
    let key_structure = key_structure_bytes(kind);
    MULTI_LEVEL_SHAPE_NODE_BYTES * nodes + entries * key_structure + (nodes - 1) * (key_structure + MULTI_LEVEL_CHILD_POSITION_BYTES)
}

/// 一组旋钮上这一版与下一版的全部派生量（登记第五节「几何与派生量」逐条；「_after」= 这一版之后）。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Geometry {
    knobs: Knobs,
    tombstones: ContainerTree,
    retirements: ContainerTree,
    lineage: ContainerTree,
    inode_leaves: u64,
    core_rewritten_units: u64,
    core_new_units: u64,
    slots_per_device: u64,
    allocation_levels: Vec<u64>,
    allocation_nodes: u64,
    allocation_levels_after: Vec<u64>,
    allocation_nodes_after: u64,
    allocation_changed_records: u64,
    allocation_touched_leaves: u64,
    allocation_rewritten_nodes: u64,
    allocation_carried_nodes: u64,
    mapping_leaf_entries: u64,
    mapping_entries: u64,
    mapping_entries_after: u64,
    mapping_rewritten_units: u64,
    mapping_new_entries: u64,
    mapping_changed_entries: u64,
    mapping_levels: Vec<u64>,
    mapping_nodes: u64,
    mapping_levels_after: Vec<u64>,
    mapping_nodes_after: u64,
    mapping_touched_leaves: u64,
    mapping_rewritten_nodes: u64,
    mapping_carried_nodes: u64,
    units: u64,
    units_after: u64,
    rewritten_units: u64,
    new_units: u64,
    allocation_records: u64,
    allocation_records_after: u64,
    deleted_inodes_after: u64,
}

/// 分配记录树每块盘用到的槽 Sl = 2·(Σ C_t + L_I + D + P_IT) + (Σ N_t + F)；叶 n_1 = 2·⌈Sl ÷ 812⌉，往上扇出 169。
fn allocation_record_tree_levels(containers: u64, index_nodes: u64, inode_leaves: u64, file_data_units: u64) -> (u64, Vec<u64>) {
    let slots = 2 * (containers + inode_leaves + file_data_units + INSTANCE_TABLE_UNITS) + (index_nodes + FIXED_POINT_NODES);
    let leaves = DEVICE_COUNT * slots.div_ceil(ALLOCATION_RECORD_TREE_LEAF_SLOTS);
    let mut levels = vec![leaves];
    while let Some(&top) = levels.last() {
        if top <= 1 {
            break;
        }
        levels.push(top.div_ceil(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT));
    }
    (slots, levels)
}

impl Geometry {
    fn new(knobs: Knobs) -> Geometry {
        let tombstones = ContainerTree::new(knobs.tombstone_records, knobs.tombstones_added, TOMBSTONE_CONTAINER_RECORDS, false);
        let retirements = ContainerTree::new(knobs.retirement_records, knobs.retirements_added, RETIREMENT_CONTAINER_RECORDS, true);
        let lineage = ContainerTree::new(knobs.lineage_records, knobs.lineage_records_added, LINEAGE_CONTAINER_RECORDS, false);
        let trees = [&tombstones, &retirements, &lineage];
        let inode_leaves = knobs.live_inodes.div_ceil(INODE_LEAF_RECORDS).max(1);
        let containers: u64 = trees.iter().map(|tree| tree.containers).sum();
        let index_nodes: u64 = trees.iter().map(|tree| tree.nodes).sum();
        let containers_after: u64 = trees.iter().map(|tree| tree.containers_after).sum();
        let index_nodes_after: u64 = trees.iter().map(|tree| tree.nodes_after).sum();
        let core_units = containers + index_nodes + inode_leaves + knobs.file_data_units + FIXED_POINT_NODES;
        let core_units_after = containers_after + index_nodes_after + inode_leaves + knobs.file_data_units + FIXED_POINT_NODES;
        let core_rewritten_units =
            trees.iter().map(|tree| tree.open_container_rewritten + tree.rewritten_index_nodes).sum::<u64>() + 1 + FIXED_POINT_NODES;
        let core_new_units = core_units_after - (core_units - core_rewritten_units);
        let (slots_per_device, allocation_levels) = allocation_record_tree_levels(containers, index_nodes, inode_leaves, knobs.file_data_units);
        let (_, allocation_levels_after) = allocation_record_tree_levels(containers_after, index_nodes_after, inode_leaves, knobs.file_data_units);
        let allocation_nodes: u64 = allocation_levels.iter().sum();
        let allocation_nodes_after: u64 = allocation_levels_after.iter().sum();
        let allocation_changed_records = 2 * (core_rewritten_units + core_new_units);
        let allocation_touched_leaves = match knobs.touched_leaves {
            TouchedLeaves::Spread => allocation_levels_after[0].min(allocation_changed_records),
            TouchedLeaves::Focused => allocation_levels_after[0].min(2),
        };
        let allocation_rewritten_nodes = rewritten_nodes_by_level(&allocation_levels_after, allocation_touched_leaves);
        let (mapping_leaf_entries, mapping_internal_entries) = if knobs.mapping_leaves_half_full {
            (CENTRAL_MAPPING_TREE.leaf_entries / 2, CENTRAL_MAPPING_TREE.internal_entries / 2)
        } else {
            (CENTRAL_MAPPING_TREE.leaf_entries, CENTRAL_MAPPING_TREE.internal_entries)
        };
        // 映射条目：树表、映射树、实例表豁免，分配记录树每个节点一条（`transaction.rs:15226-15227`）。
        let mapping_entries = core_units - 1 + allocation_nodes;
        let mapping_entries_after = core_units_after - 1 + allocation_nodes_after;
        let mapping_rewritten_units = core_rewritten_units - 1 + allocation_rewritten_nodes;
        let mapping_new_entries = mapping_entries_after - (mapping_entries - mapping_rewritten_units);
        let mapping_changed_entries = mapping_rewritten_units + mapping_new_entries;
        let mapping_levels = levels_by_fanout(mapping_entries, mapping_leaf_entries, mapping_internal_entries);
        let mapping_levels_after = levels_by_fanout(mapping_entries_after, mapping_leaf_entries, mapping_internal_entries);
        let mapping_nodes: u64 = mapping_levels.iter().sum();
        let mapping_nodes_after: u64 = mapping_levels_after.iter().sum();
        let mapping_touched_leaves = match knobs.touched_leaves {
            TouchedLeaves::Spread => mapping_levels_after[0].min(mapping_changed_entries),
            TouchedLeaves::Focused => mapping_levels_after[0].min(mapping_changed_entries.div_ceil(mapping_leaf_entries)),
        };
        let mapping_rewritten_nodes = rewritten_nodes_by_level(&mapping_levels_after, mapping_touched_leaves);
        let units = core_units + allocation_nodes + mapping_nodes + INSTANCE_TABLE_UNITS;
        let units_after = core_units_after + allocation_nodes_after + mapping_nodes_after + INSTANCE_TABLE_UNITS;
        let rewritten_units = core_rewritten_units + allocation_rewritten_nodes + mapping_rewritten_nodes;
        let new_units = units_after - (units - rewritten_units);
        Geometry {
            knobs,
            inode_leaves,
            core_rewritten_units,
            core_new_units,
            slots_per_device,
            allocation_carried_nodes: allocation_nodes_after.saturating_sub(allocation_rewritten_nodes),
            allocation_levels,
            allocation_nodes,
            allocation_levels_after,
            allocation_nodes_after,
            allocation_changed_records,
            allocation_touched_leaves,
            allocation_rewritten_nodes,
            mapping_leaf_entries,
            mapping_entries,
            mapping_entries_after,
            mapping_rewritten_units,
            mapping_new_entries,
            mapping_changed_entries,
            mapping_carried_nodes: mapping_nodes_after.saturating_sub(mapping_rewritten_nodes),
            mapping_levels,
            mapping_nodes,
            mapping_levels_after,
            mapping_nodes_after,
            mapping_touched_leaves,
            mapping_rewritten_nodes,
            units,
            units_after,
            rewritten_units,
            new_units,
            allocation_records: 2 * units,
            allocation_records_after: 2 * units_after,
            deleted_inodes_after: u64::from(knobs.tombstones_added > 0),
            tombstones,
            retirements,
            lineage,
        }
    }

    fn container_trees(&self) -> [&ContainerTree; 3] {
        [&self.tombstones, &self.retirements, &self.lineage]
    }

    /// 墓碑树与退役树（O 里逐树的那几项只管这两棵）。
    fn tombstone_and_retirement_trees(&self) -> [(&ContainerTree, u64); 2] {
        [(&self.tombstones, TOMBSTONE_RECORD_MEMORY_BYTES), (&self.retirements, RETIREMENT_RECORD_MEMORY_BYTES)]
    }

    fn trees_with_record_widths(&self) -> [(&ContainerTree, u64); 3] {
        [
            (&self.tombstones, TOMBSTONE_RECORD_MEMORY_BYTES),
            (&self.retirements, RETIREMENT_RECORD_MEMORY_BYTES),
            (&self.lineage, LINEAGE_RECORD_MEMORY_BYTES),
        ]
    }
}

// ── 甲：每次发布的照抄字节（登记第五节「每次发布的照抄字节」；第三份的式子） ──

/// 甲-B2后准入前的三部分：O（`ordinary_allocation_part` 一遍，跑 s + 1 遍）、Rl（`resolve` 一轮，跑 s 遍）、Once（循环外一次）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct CopiedBeforeAdmission {
    ordinary_allocation_pass: u64,
    settle_round: u64,
    outside_the_rounds: u64,
}

impl CopiedBeforeAdmission {
    fn total(&self, settle_rounds: u64) -> u64 {
        (settle_rounds + 1) * self.ordinary_allocation_pass + settle_rounds * self.settle_round + self.outside_the_rounds
    }
}

/// 三棵多层码 2 树（记账、deadlist、中央映射）这一版的（条目数、节点数、种类）。记账树 16 条一片叶；deadlist 零根（没有快照）。
fn multi_level_trees(geometry: &Geometry) -> [(u64, u64, MultiLevelTreeKind); 3] {
    [(ACCOUNTING_ENTRIES, 1, ACCOUNTING_TREE), (0, 0, DEADLIST_TREE), (geometry.mapping_entries, geometry.mapping_nodes, CENTRAL_MAPPING_TREE)]
}

/// 同上，这一版之后。
fn multi_level_trees_after(geometry: &Geometry) -> [(u64, u64, MultiLevelTreeKind); 3] {
    [(ACCOUNTING_ENTRIES, 1, ACCOUNTING_TREE), (0, 0, DEADLIST_TREE), (geometry.mapping_entries_after, geometry.mapping_nodes_after, CENTRAL_MAPPING_TREE)]
}

/// 不追加那一支每遍由调用方抄的那一份上一版形状（`transaction.rs:8640` 墓碑树、`8965` 谱系树；退役树每次都追加，这一项是 0）。
fn shape_copied_when_not_appending(tree: &ContainerTree) -> u64 {
    if tree.appends { 0 } else { tree.shape_bytes() }
}

fn jia_after_patch_copied_before_admission(geometry: &Geometry) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let mut ordinary_allocation_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        ordinary_allocation_pass += tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers;
        if tree.appends {
            ordinary_allocation_pass += tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers + tree.shape_bytes();
        }
        // `transaction.rs:8640`：墓碑树不追加那一支照抄上一版形状（第三份补进）。
        ordinary_allocation_pass += shape_copied_when_not_appending(tree);
    }
    // inode 叶照抄与它的第二份（`transaction.rs:4208`、`inode_tree.rs:1093` / `1970`）。
    ordinary_allocation_pass += 2 * (INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves);
    let lineage = &geometry.lineage;
    let mut settle_round = lineage.records * LINEAGE_RECORD_MEMORY_BYTES + CONTAINER_SHELL_BYTES * lineage.containers;
    if lineage.appends {
        settle_round += lineage.records * LINEAGE_RECORD_MEMORY_BYTES + CONTAINER_SHELL_BYTES * lineage.containers + lineage.shape_bytes();
    }
    // `transaction.rs:8965`：谱系树不追加那一支照抄上一版形状（第三份补进）。
    settle_round += shape_copied_when_not_appending(lineage);
    settle_round += (geometry.mapping_entries - geometry.mapping_rewritten_units) * MAPPED_UNIT_ENTRY_BYTES;
    for (entries, nodes, kind) in multi_level_trees(geometry) {
        settle_round += entries * key_structure_bytes(kind);
        if nodes >= 1 {
            settle_round += (entries + nodes - 1) * key_structure_bytes(kind);
        }
    }
    settle_round += 2 * geometry.allocation_records * ALLOCATION_RECORD_MEMORY_BYTES;
    CopiedBeforeAdmission {
        ordinary_allocation_pass,
        settle_round,
        outside_the_rounds: 2 * geometry.allocation_nodes * ALLOCATION_TREE_NODE_IDENTITY_BYTES,
    }
}

/// 甲-B2后准入后照抄的分项：(a) 新版 inode 叶内容再抄一次；(b) 中央映射树照抄节点（两份各一）；(c) 三棵多层码 2 树形状；
/// (d) 上一版分配记录树节点集两份（在准入前，Once）；其余照式子合在 `rest` 里。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct CopiedAfterAdmission {
    inode_leaves_again: u64,
    mapping_carried_nodes_each_copy: u64,
    multi_level_shapes: u64,
    carried_file_data_units: u64,
    carried_file_data_pointers: u64,
    rest: u64,
}

impl CopiedAfterAdmission {
    fn total(&self) -> u64 {
        self.inode_leaves_again
            + 2 * self.mapping_carried_nodes_each_copy
            + self.multi_level_shapes
            + self.carried_file_data_units
            + self.carried_file_data_pointers
            + self.rest
    }
}

fn jia_after_patch_copied_after_admission(geometry: &Geometry) -> CopiedAfterAdmission {
    let knobs = geometry.knobs;
    let mut rest = 0;
    for (tree, record_width) in geometry.trees_with_record_widths() {
        rest += tree.records_after * record_width
            + CONTAINER_SHELL_BYTES * tree.containers_after
            + shape_bytes(tree.nodes_after, tree.height_after)
            + (tree.containers - tree.open_container_rewritten) * CONTAINER_UNIT_MEMORY_BYTES
            + (tree.nodes - tree.rewritten_index_nodes) * NODE_UNIT_MEMORY_BYTES;
    }
    // 第一个文件之外的照抄单元（inode 叶 L_I − 1 片与数据单元 D 个）走 `carried_unit`。
    rest += (geometry.inode_leaves - 1 + knobs.file_data_units) * CONTAINER_UNIT_MEMORY_BYTES;
    rest += ALLOCATION_RECORD_MEMORY_BYTES * (geometry.allocation_records_after - 2 * geometry.new_units);
    rest += geometry.allocation_carried_nodes * NODE_UNIT_MEMORY_BYTES;
    rest += INSTANCE_TABLE_UNITS * CONTAINER_UNIT_MEMORY_BYTES;
    rest += geometry.allocation_records * ALLOCATION_RECORD_MEMORY_BYTES;
    CopiedAfterAdmission {
        inode_leaves_again: INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves,
        mapping_carried_nodes_each_copy: geometry.mapping_carried_nodes * NODE_UNIT_MEMORY_BYTES,
        multi_level_shapes: multi_level_trees_after(geometry)
            .iter()
            .map(|&(entries, nodes, kind)| multi_level_shape_bytes(entries, nodes, kind))
            .sum(),
        // `transaction.rs:14640`、`14643`（第三份全扫补进）。
        carried_file_data_units: CARRIED_DATA_UNIT_BYTES * knobs.file_data_units,
        carried_file_data_pointers: DATA_POINTER_MEMORY_BYTES * knobs.file_data_units,
        rest,
    }
}

/// 甲-今天准入前 = 甲-B2后 + (s + 1)·Σ_{T,X} (2 + k_new_t − [t 不追加])·shape + s·(2 + k_new_L − [a_L = 0])·shape_L。
fn jia_today_copied_before_admission(geometry: &Geometry) -> u64 {
    let settle_rounds = geometry.knobs.settle_rounds;
    let shapes_dropped_by_the_patch = |tree: &ContainerTree| (2 + tree.new_containers - u64::from(!tree.appends)) * tree.shape_bytes();
    let mut today = jia_after_patch_copied_before_admission(geometry).total(settle_rounds);
    for (tree, _) in geometry.tombstone_and_retirement_trees() {
        today += (settle_rounds + 1) * shapes_dropped_by_the_patch(tree);
    }
    today + settle_rounds * shapes_dropped_by_the_patch(&geometry.lineage)
}

/// 第三份补进的四项逐项（甲-B2后，每次发布）：`8640`、`8965`、`14640`、`14643`。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ItemsAddedByTheThirdRegistration {
    tombstone_shape_when_not_appending: u64,
    lineage_shape_when_not_appending: u64,
    carried_file_data_units: u64,
    carried_file_data_pointers: u64,
}

fn items_added_by_the_third_registration(geometry: &Geometry) -> ItemsAddedByTheThirdRegistration {
    let settle_rounds = geometry.knobs.settle_rounds;
    let tombstones = &geometry.tombstones;
    let lineage = &geometry.lineage;
    ItemsAddedByTheThirdRegistration {
        tombstone_shape_when_not_appending: (settle_rounds + 1) * shape_copied_when_not_appending(tombstones),
        lineage_shape_when_not_appending: settle_rounds * shape_copied_when_not_appending(lineage),
        carried_file_data_units: CARRIED_DATA_UNIT_BYTES * geometry.knobs.file_data_units,
        carried_file_data_pointers: DATA_POINTER_MEMORY_BYTES * geometry.knobs.file_data_units,
    }
}

// ── 乙2、丙、丁的照抄字节（登记第五节；第三份新补几样上三臂多付 0） ──

/// depth(n) = 使 b^d ≥ n 的最小 d（n ≤ 1 时 0）。
fn persistent_depth(count: u64, fanout: u64) -> u64 {
    let mut depth = 0;
    let mut reach = 1u64;
    while reach < count {
        reach = reach.saturating_mul(fanout);
        depth += 1;
    }
    depth
}

/// 乙2 的持久集合照抄 PV(n, w, x, 块) = T·(min(块, n)·w + depth(⌈n ÷ 块⌉)·P)，T = min(⌈n ÷ 块⌉, x)；n = 0 或 x = 0 时 0。
fn persistent_vector_copy(count: u64, width: u64, touched: u64, block: u64, fanout: u64) -> u64 {
    if count == 0 || touched == 0 {
        return 0;
    }
    let blocks = count.div_ceil(block);
    blocks.min(touched) * (block.min(count) * width + persistent_depth(blocks, fanout) * persistent_internal_node_bytes(fanout))
}

/// 乙2 一个内部节点 P = 8b + 16。
fn persistent_internal_node_bytes(fanout: u64) -> u64 {
    8 * fanout + 16
}

/// 乙2 一棵容器树一遍：开放容器的末块、容器表末块、最右路径形状（第一份第五节乙2 那一式里逐树那一段）。
fn yi2_container_tree_part(tree: &ContainerTree, record_width: u64, knobs: Knobs) -> u64 {
    let fanout = knobs.persistent_fanout;
    let block = knobs.persistent_leaf_block;
    let node = persistent_internal_node_bytes(fanout);
    let mut part = tree.open_container_rewritten * ((tree.fill % block) * record_width + persistent_depth(tree.fill.div_ceil(block), fanout) * node);
    if tree.containers > 0 {
        part += (((tree.containers - 1) % fanout) + 1) * CONTAINER_SHELL_BYTES + persistent_depth(tree.containers.div_ceil(fanout), fanout) * node;
    }
    part + SHAPE_NODE_BYTES * tree.height + SHAPE_LEVEL_BYTES
}

/// 乙2（b、g 取旋钮）：准入前 (s + 1)·O₂ + s·Rl₂ + Once₂，准入后一次。
fn yi2_copied(geometry: &Geometry) -> (u64, u64) {
    let knobs = geometry.knobs;
    let fanout = knobs.persistent_fanout;
    let block = knobs.persistent_leaf_block;
    let node = persistent_internal_node_bytes(fanout);
    let inode_part = INODE_LEAF_RECORDS.min(block) * INODE_RECORD_MEMORY_BYTES
        + persistent_depth(INODE_LEAF_RECORDS.div_ceil(block), fanout) * node
        + fanout.min(geometry.inode_leaves) * CONTAINER_SHELL_BYTES
        + persistent_depth(geometry.inode_leaves.div_ceil(fanout), fanout) * node;
    let mut ordinary_allocation_pass = inode_part;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        if tree.added > 0 {
            ordinary_allocation_pass += yi2_container_tree_part(tree, record_width, knobs);
        }
    }
    let changed = |kind: MultiLevelTreeKind| -> u64 {
        if kind == CENTRAL_MAPPING_TREE {
            geometry.mapping_changed_entries
        } else if kind == ACCOUNTING_TREE {
            ACCOUNTING_ENTRIES
        } else {
            0
        }
    };
    let mut settle_round = 0;
    if geometry.lineage.added > 0 {
        settle_round += yi2_container_tree_part(&geometry.lineage, LINEAGE_RECORD_MEMORY_BYTES, knobs);
    }
    settle_round += persistent_vector_copy(geometry.mapping_entries, MAPPED_UNIT_ENTRY_BYTES, geometry.mapping_changed_entries, block, fanout);
    for (entries, _, kind) in multi_level_trees(geometry) {
        settle_round += persistent_vector_copy(entries, key_structure_bytes(kind), changed(kind), block, fanout);
    }
    settle_round += 2 * persistent_vector_copy(geometry.allocation_records, ALLOCATION_RECORD_MEMORY_BYTES, geometry.allocation_changed_records, block, fanout);
    let outside_the_rounds =
        persistent_vector_copy(geometry.allocation_nodes, ALLOCATION_TREE_NODE_IDENTITY_BYTES, geometry.allocation_touched_leaves, fanout, fanout);
    let before = CopiedBeforeAdmission { ordinary_allocation_pass, settle_round, outside_the_rounds }.total(knobs.settle_rounds);
    let touched_unit_segments = geometry.container_trees().iter().filter(|tree| tree.added > 0).count() as u64 + 4;
    let mut after = touched_unit_segments
        * (fanout.min(geometry.units) * UNIT_SHELL_BYTES + persistent_depth(geometry.units.div_ceil(fanout), fanout) * node)
        + 2 * geometry.rewritten_units
            * (fanout.min(geometry.allocation_records) * ALLOCATION_RECORD_MEMORY_BYTES
                + persistent_depth(geometry.allocation_records.div_ceil(fanout), fanout) * node)
        + persistent_vector_copy(knobs.live_inodes, INODE_RECORD_MEMORY_BYTES, 1, block, fanout)
        + persistent_vector_copy(geometry.inode_leaves, CONTAINER_SHELL_BYTES, 1, fanout, fanout)
        + persistent_vector_copy(geometry.allocation_records, ALLOCATION_RECORD_MEMORY_BYTES, geometry.allocation_changed_records, block, fanout);
    for (entries, _, kind) in multi_level_trees_after(geometry) {
        after += persistent_vector_copy(entries, key_structure_bytes(kind), changed(kind), block, fanout);
    }
    (before, after)
}

// ── 第四份写死的乙2、乙1 两样（第四份第五节「阳性对照」「乙1 的照抄字节」） ──

/// 一棵容器树整条：记录、容器外壳与形状 R_t·W_t + 56·C_t + shape(N_t, h_t)（乙2 共享关掉那一读里每条集合整条复制）。
fn whole_container_tree_bytes(tree: &ContainerTree, record_width: u64) -> u64 {
    tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers + tree.shape_bytes()
}

/// 乙2 阳性对照的退化形态（第四份第五节写死）：乙2 自己的持久分块向量把共享关掉，这一版每一条集合每次造版都整条复制，
/// 碰到的与没碰到的都复制、每一片容器的记录都复制、a_t = 0 的树也复制；遍数照乙2 准入前（O 部分 s + 1 遍、Rl 部分 s 遍、节点集一次）。
/// 只计元素字节，分块向量的内部节点与末块凑整不计（第二段的计数分配器照量）。
fn yi2_sharing_turned_off_copied_before_admission(geometry: &Geometry) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let mut sharing_off_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        // 没碰到的树（a_t = 0，G0 的退役树）也整条复制。
        sharing_off_pass += whole_container_tree_bytes(tree, record_width);
    }
    sharing_off_pass += INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    let mut sharing_off_round = whole_container_tree_bytes(&geometry.lineage, LINEAGE_RECORD_MEMORY_BYTES);
    sharing_off_round += MAPPED_UNIT_ENTRY_BYTES * geometry.mapping_entries;
    for (entries, _, kind) in multi_level_trees(geometry) {
        sharing_off_round += key_structure_bytes(kind) * entries;
    }
    sharing_off_round += 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    CopiedBeforeAdmission {
        ordinary_allocation_pass: sharing_off_pass,
        settle_round: sharing_off_round,
        outside_the_rounds: ALLOCATION_TREE_NODE_IDENTITY_BYTES * geometry.allocation_nodes,
    }
}

/// 附带（第四份第六节另加那一行，报数、不判、不触发作废条款第 3 条）：执行员头一种读法，只把乙2 式子里碰到的集合整条复制，
/// 墓碑树只复制开放那一片的记录、没碰到的树照旧共享。
fn yi2_touched_collections_whole_copied_before_admission(geometry: &Geometry) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let open_container_and_table = |tree: &ContainerTree, record_width: u64| -> u64 {
        if tree.added > 0 { tree.fill * record_width + CONTAINER_SHELL_BYTES * tree.containers + tree.shape_bytes() } else { 0 }
    };
    let mut touched_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        touched_pass += open_container_and_table(tree, record_width);
    }
    touched_pass += INODE_LEAF_RECORDS.min(knobs.live_inodes) * INODE_RECORD_MEMORY_BYTES + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    let mut touched_round = open_container_and_table(&geometry.lineage, LINEAGE_RECORD_MEMORY_BYTES);
    touched_round += MAPPED_UNIT_ENTRY_BYTES * geometry.mapping_entries;
    for (entries, _, kind) in multi_level_trees(geometry) {
        touched_round += key_structure_bytes(kind) * entries;
    }
    touched_round += 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    CopiedBeforeAdmission {
        ordinary_allocation_pass: touched_pass,
        settle_round: touched_round,
        outside_the_rounds: ALLOCATION_TREE_NODE_IDENTITY_BYTES * geometry.allocation_nodes,
    }
}

/// 乙1 的引用计数照常共享，或阳性对照那一读把共享关掉（克隆内容、不克隆指针；第四份第五节「乙1：字面推得出」：
/// 乙1 的每一项是在甲-B2后那一项上换出来的，关掉共享就逐项换回甲-B2后那一项）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ReferenceSharing {
    Shared,
    TurnedOff,
}

/// 乙1 准入前（第四份第五节「乙1 的照抄字节」三条规则与逐项对照表）：规则 1 第一份乙1 写法，规则 3 其余照甲。
fn yi1_copied_before_admission(geometry: &Geometry, sharing: ReferenceSharing) -> CopiedBeforeAdmission {
    let knobs = geometry.knobs;
    let is_shared = sharing == ReferenceSharing::Shared;
    // 容器表：共享时顶层只抄指针 8·C_t（规则 1），关掉时照甲抄记录与外壳 R_t·W_t + 56·C_t。
    let container_table = |tree: &ContainerTree, record_width: u64| -> u64 {
        if is_shared { REFERENCE_COUNTED_POINTER_BYTES * tree.containers } else { tree.records * record_width + CONTAINER_SHELL_BYTES * tree.containers }
    };
    // 被改那片写时复制 fill_t·W_t（a_t > 0、k_old_t = 1；规则 1）；甲没有单列这一项，含在整条记录里，关掉时是 0。
    let copy_on_write_of_the_rewritten_container = |tree: &ContainerTree, record_width: u64| -> u64 {
        if is_shared { tree.open_container_rewritten * tree.fill * record_width } else { 0 }
    };
    let mut yi1_ordinary_allocation_pass = 0;
    for (tree, record_width) in geometry.tombstone_and_retirement_trees() {
        yi1_ordinary_allocation_pass += container_table(tree, record_width);
        if tree.appends {
            yi1_ordinary_allocation_pass += container_table(tree, record_width) + tree.shape_bytes();
        }
        yi1_ordinary_allocation_pass += copy_on_write_of_the_rewritten_container(tree, record_width);
        // `8640`：不追加那一支的形状照甲（规则 3）。
        yi1_ordinary_allocation_pass += shape_copied_when_not_appending(tree);
    }
    // inode 叶第一份：共享时照第一份乙1 写法 8·L_I + min(I, 233)·64（规则 1）。
    let inode_leaves_first_copy = if is_shared {
        REFERENCE_COUNTED_POINTER_BYTES * geometry.inode_leaves + INODE_LEAF_RECORDS.min(knobs.live_inodes) * INODE_RECORD_MEMORY_BYTES
    } else {
        INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves
    };
    // inode 叶第二份是第二份登记新补的项，照甲（规则 3），共享开不开都一样。
    let inode_leaves_second_copy = INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    yi1_ordinary_allocation_pass += inode_leaves_first_copy + inode_leaves_second_copy;
    let lineage = &geometry.lineage;
    let mut yi1_settle_round = container_table(lineage, LINEAGE_RECORD_MEMORY_BYTES);
    if lineage.appends {
        yi1_settle_round += container_table(lineage, LINEAGE_RECORD_MEMORY_BYTES) + lineage.shape_bytes();
    }
    yi1_settle_round += copy_on_write_of_the_rewritten_container(lineage, LINEAGE_RECORD_MEMORY_BYTES);
    // `8965`：谱系树不追加那一支的形状照甲（规则 3）。
    yi1_settle_round += shape_copied_when_not_appending(lineage);
    // 映射 key、多层码 2 树的 key 与规划、分配记录按叶分组照甲（规则 3）。
    yi1_settle_round += MAPPED_UNIT_ENTRY_BYTES * (geometry.mapping_entries - geometry.mapping_rewritten_units);
    for (entries, nodes, kind) in multi_level_trees(geometry) {
        yi1_settle_round += key_structure_bytes(kind) * entries;
        if nodes >= 1 {
            yi1_settle_round += key_structure_bytes(kind) * (entries + nodes - 1);
        }
    }
    yi1_settle_round += 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    CopiedBeforeAdmission {
        ordinary_allocation_pass: yi1_ordinary_allocation_pass,
        settle_round: yi1_settle_round,
        outside_the_rounds: 2 * ALLOCATION_TREE_NODE_IDENTITY_BYTES * geometry.allocation_nodes,
    }
}

/// 乙1 准入后一次（第四份第五节合起来那一式）：容器内容与形状照第一份乙1 写法（规则 1），单元字节项换成 × 8（规则 2），其余照甲（规则 3）。
fn yi1_copied_after_admission(geometry: &Geometry, sharing: ReferenceSharing) -> u64 {
    let knobs = geometry.knobs;
    let is_shared = sharing == ReferenceSharing::Shared;
    let carried_container_units = |units: u64| -> u64 { if is_shared { REFERENCE_COUNTED_POINTER_BYTES * units } else { CONTAINER_UNIT_MEMORY_BYTES * units } };
    let carried_node_units = |units: u64| -> u64 { if is_shared { REFERENCE_COUNTED_POINTER_BYTES * units } else { NODE_UNIT_MEMORY_BYTES * units } };
    let mut yi1_after = 0;
    for (tree, record_width) in geometry.trees_with_record_widths() {
        let new_version_contents =
            if is_shared { REFERENCE_COUNTED_POINTER_BYTES * tree.containers_after } else { tree.records_after * record_width + CONTAINER_SHELL_BYTES * tree.containers_after };
        yi1_after += new_version_contents
            + shape_bytes(tree.nodes_after, tree.height_after)
            + carried_container_units(tree.containers - tree.open_container_rewritten)
            + carried_node_units(tree.nodes - tree.rewritten_index_nodes);
    }
    yi1_after += carried_container_units(geometry.inode_leaves - 1 + knobs.file_data_units);
    yi1_after += ALLOCATION_RECORD_MEMORY_BYTES * (geometry.allocation_records_after - 2 * geometry.new_units);
    // (a) 新版 inode 叶内容再抄一次（规则 3）。
    yi1_after += INODE_RECORD_MEMORY_BYTES * knobs.live_inodes + CONTAINER_SHELL_BYTES * geometry.inode_leaves;
    // (b) 中央映射树照抄节点与它的第二份、分配记录树照抄节点（规则 2）。
    yi1_after += 2 * carried_node_units(geometry.mapping_carried_nodes) + carried_node_units(geometry.allocation_carried_nodes);
    // (c) 三棵多层码 2 树形状（规则 3）。
    yi1_after += multi_level_trees_after(geometry).iter().map(|&(entries, nodes, kind)| multi_level_shape_bytes(entries, nodes, kind)).sum::<u64>();
    yi1_after += carried_container_units(INSTANCE_TABLE_UNITS);
    // 分配记录按叶分组 A·40（规则 3）。
    yi1_after += ALLOCATION_RECORD_MEMORY_BYTES * geometry.allocation_records;
    // `14640` 第一个文件照抄的数据单元换成指针（规则 2）；`14643` 数据指针照甲（规则 3）。
    let yi1_carried_file_data_units = if is_shared { REFERENCE_COUNTED_POINTER_BYTES * knobs.file_data_units } else { CARRIED_DATA_UNIT_BYTES * knobs.file_data_units };
    let yi1_carried_file_data_pointers = knobs.file_data_units * DATA_POINTER_MEMORY_BYTES;
    yi1_after + yi1_carried_file_data_units + yi1_carried_file_data_pointers
}

/// 丙准入后：撤回日志 = 48·r + Σ_{T,X,L,M,AR} (8 + 8·h_t) + 64 + 80·r；准入前 0。
fn bing_copied_after_admission(geometry: &Geometry) -> u64 {
    let heights = [
        geometry.tombstones.height,
        geometry.retirements.height,
        geometry.lineage.height,
        geometry.mapping_levels.len() as u64,
        geometry.allocation_levels.len() as u64,
    ];
    let rightmost_paths: u64 = heights.iter().map(|height| 8 + 8 * height).sum();
    UNIT_SHELL_BYTES * geometry.rewritten_units
        + rightmost_paths
        + INODE_RECORD_MEMORY_BYTES
        + 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.rewritten_units
}

/// 丁的压实方式：整份新基（照抄 |V'|）与就地折叠（照抄 0）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Compaction {
    WholeNewBase,
    FoldInPlace,
}

impl Compaction {
    fn label(self) -> &'static str {
        match self {
            Compaction::WholeNewBase => "whole_new_base",
            Compaction::FoldInPlace => "fold_in_place",
        }
    }
}

/// 丁准入后：非压实那次 0；压实那次整份新基 |V'|、就地折叠 0；准入前 0。
fn ding_copied_after_admission(geometry: &Geometry, compacts_now: bool, compaction: Compaction) -> u64 {
    if compacts_now && compaction == Compaction::WholeNewBase {
        version_bytes_after(geometry)
    } else {
        0
    }
}

// ── 新装、新装第二份、现算重建、一版大小（四条臂相同的那几样；登记第五节） ──

/// 新装字节（四条臂相同）：准入前、准入后。
fn installed_bytes(geometry: &Geometry) -> (u64, u64) {
    let knobs = geometry.knobs;
    let appended_records: u64 = geometry.trees_with_record_widths().iter().map(|(tree, width)| tree.added * width).sum();
    let before = (knobs.settle_rounds + 1) * (geometry.tombstones.added * TOMBSTONE_RECORD_MEMORY_BYTES + geometry.retirements.added * RETIREMENT_RECORD_MEMORY_BYTES)
        + knobs.settle_rounds
            * (geometry.lineage.added * LINEAGE_RECORD_MEMORY_BYTES
                + geometry.mapping_new_entries * MAPPED_UNIT_ENTRY_BYTES
                + ACCOUNTING_ENTRIES * key_structure_bytes(ACCOUNTING_TREE));
    let after = appended_records + installed_unit_bytes(geometry) + 2 * ALLOCATION_RECORD_MEMORY_BYTES * geometry.new_units;
    (before, after)
}

/// 准入后新装里的单元字节那几项：容器、索引节点、inode 叶、固定点、映射树与分配记录树重写的节点。
fn installed_unit_bytes(geometry: &Geometry) -> u64 {
    let mut unit_bytes = 0;
    for tree in geometry.container_trees() {
        unit_bytes += (tree.open_container_rewritten + tree.new_containers) * CONTAINER_UNIT_MEMORY_BYTES
            + (tree.nodes_after - tree.nodes + tree.rewritten_index_nodes) * NODE_UNIT_MEMORY_BYTES;
    }
    unit_bytes
        + CONTAINER_UNIT_MEMORY_BYTES
        + FIXED_POINT_NODES * NODE_UNIT_MEMORY_BYTES
        + (geometry.mapping_rewritten_nodes + geometry.allocation_rewritten_nodes) * NODE_UNIT_MEMORY_BYTES
}

/// 现算重建字节（甲-B2后、甲-今天、乙1；丁压实那次的准入后同此）：准入前、准入后。
fn recomputed_in_full(geometry: &Geometry) -> (u64, u64) {
    let knobs = geometry.knobs;
    let deadlist_entries_after = 0;
    let before = knobs.settle_rounds
        * (geometry.mapping_entries_after * key_structure_bytes(CENTRAL_MAPPING_TREE)
            + deadlist_entries_after * key_structure_bytes(DEADLIST_TREE)
            + geometry.allocation_nodes_after * ALLOCATION_TREE_NODE_IDENTITY_BYTES
            + 3 * geometry.allocation_records_after * ALLOCATION_RECORD_MEMORY_BYTES);
    let after = geometry.mapping_entries_after * mapping_unit_recompute_bytes()
        + rewritten_mapping_leaf_entries(geometry, geometry.mapping_entries_after) * MAPPING_LEAF_ENTRY_BYTES
        + deadlist_entries_after * MAPPED_UNIT_ENTRY_BYTES
        + geometry.deleted_inodes_after * DELETED_INODE_NUMBER_BYTES
        + 3 * geometry.allocation_records_after * ALLOCATION_RECORD_MEMORY_BYTES
        + 2 * geometry.allocation_nodes_after * ALLOCATION_TREE_NODE_IDENTITY_BYTES;
    (before, after)
}

/// 准入后每条映射现算的四样：带位置项 95 + `mapped_units` 67 + 映射条目表 194 + key 字节 51 = 407。
fn mapping_unit_recompute_bytes() -> u64 {
    MAPPED_UNIT_WITH_LOCATIONS_BYTES + MAPPED_UNIT_ENTRY_BYTES + MAPPING_ENTRY_TABLE_BYTES + MAPPING_KEY_BYTES_ENTRY_BYTES
}

/// 重写的映射树叶的条目数 min(M', τ_M·c)。
fn rewritten_mapping_leaf_entries(geometry: &Geometry, mapping_entries_after: u64) -> u64 {
    mapping_entries_after.min(geometry.mapping_touched_leaves * geometry.mapping_leaf_entries)
}

/// 残余式（乙2、丙、丁非压实那次）：x = min(d_M, M')、y = min(d_AR, A')。
fn recomputed_residual(geometry: &Geometry) -> (u64, u64) {
    let changed_mappings = geometry.mapping_changed_entries.min(geometry.mapping_entries_after);
    let changed_allocations = geometry.allocation_changed_records.min(geometry.allocation_records_after);
    let before = geometry.knobs.settle_rounds
        * (changed_mappings * key_structure_bytes(CENTRAL_MAPPING_TREE) + 3 * changed_allocations * ALLOCATION_RECORD_MEMORY_BYTES);
    let after = changed_mappings * mapping_unit_recompute_bytes()
        + rewritten_mapping_leaf_entries(geometry, geometry.mapping_entries_after) * MAPPING_LEAF_ENTRY_BYTES
        + geometry.deleted_inodes_after * DELETED_INODE_NUMBER_BYTES
        + 3 * changed_allocations * ALLOCATION_RECORD_MEMORY_BYTES;
    (before, after)
}

/// 一版大小的各项：三棵容器树、inode、单元（含固定点、实例表、映射树与分配记录树节点）、分配记录、映射、多层码 2 树形状、数据指针。
fn version_size_parts(
    trees: [(u64, u64, u64, u64, u64); 3],
    geometry: &Geometry,
    mapping_entries: u64,
    mapping_nodes: u64,
    allocation_nodes: u64,
    allocation_records: u64,
) -> (u64, u64) {
    let knobs = geometry.knobs;
    let mut without_pointers = 0;
    for (records, record_width, containers, nodes, height) in trees {
        without_pointers += records * record_width
            + CONTAINER_SHELL_BYTES * containers
            + shape_bytes(nodes, height)
            + containers * CONTAINER_UNIT_MEMORY_BYTES
            + nodes * NODE_UNIT_MEMORY_BYTES;
    }
    without_pointers += INODE_RECORD_MEMORY_BYTES * knobs.live_inodes
        + CONTAINER_SHELL_BYTES * geometry.inode_leaves
        + (geometry.inode_leaves + knobs.file_data_units) * CONTAINER_UNIT_MEMORY_BYTES
        + FIXED_POINT_NODES * NODE_UNIT_MEMORY_BYTES
        + INSTANCE_TABLE_UNITS * CONTAINER_UNIT_MEMORY_BYTES
        + (mapping_nodes + allocation_nodes) * NODE_UNIT_MEMORY_BYTES
        + ALLOCATION_RECORD_MEMORY_BYTES * allocation_records
        + MAPPED_UNIT_ENTRY_BYTES * mapping_entries
        + ALLOCATION_TREE_NODE_IDENTITY_BYTES * allocation_nodes
        + multi_level_shape_bytes(ACCOUNTING_ENTRIES, 1, ACCOUNTING_TREE)
        + multi_level_shape_bytes(mapping_entries, mapping_nodes, CENTRAL_MAPPING_TREE);
    (without_pointers, DATA_POINTER_MEMORY_BYTES * knobs.file_data_units)
}

/// |V|：这一版的大小（第三份另加这一版的 `data_pointers` 88·D）。
fn version_bytes(geometry: &Geometry) -> u64 {
    let trees = geometry
        .trees_with_record_widths()
        .map(|(tree, width)| (tree.records, width, tree.containers, tree.nodes, tree.height));
    let (without_pointers, pointers) =
        version_size_parts(trees, geometry, geometry.mapping_entries, geometry.mapping_nodes, geometry.allocation_nodes, geometry.allocation_records);
    without_pointers + pointers
}

/// |V'|：这一版之后的大小，另加 `deleted_inodes` 的号 32·Q'。
fn version_bytes_after(geometry: &Geometry) -> u64 {
    let trees = geometry
        .trees_with_record_widths()
        .map(|(tree, width)| (tree.records_after, width, tree.containers_after, tree.nodes_after, tree.height_after));
    let (without_pointers, pointers) = version_size_parts(
        trees,
        geometry,
        geometry.mapping_entries_after,
        geometry.mapping_nodes_after,
        geometry.allocation_nodes_after,
        geometry.allocation_records_after,
    );
    without_pointers + pointers + DELETED_INODE_NUMBER_BYTES * geometry.deleted_inodes_after
}

// ── 各臂每次发布的四样字节、比值与判定（登记第六节） ──

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    JiaAfterPatch,
    JiaToday,
    Yi2,
    Bing,
    DingWithoutCompaction,
    DingWholeNewBaseCompaction,
}

impl Arm {
    const EVERY_ARM: [Arm; 6] = [Arm::JiaAfterPatch, Arm::JiaToday, Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction, Arm::DingWholeNewBaseCompaction];

    fn label(self) -> &'static str {
        match self {
            Arm::JiaAfterPatch => "jia_after_patch",
            Arm::JiaToday => "jia_today",
            Arm::Yi2 => "yi2_persistent",
            Arm::Bing => "bing_by_value",
            Arm::DingWithoutCompaction => "ding_without_compaction",
            Arm::DingWholeNewBaseCompaction => "ding_whole_new_base_compaction",
        }
    }
}

/// 一条臂一次发布：照抄、现算重建，准入前与准入后各一格（读法第 1 条：两段各报各的）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct ArmBytes {
    copied_before: u64,
    copied_after: u64,
    recomputed_before: u64,
    recomputed_after: u64,
}

fn arm_bytes(geometry: &Geometry, arm: Arm) -> ArmBytes {
    let in_full = recomputed_in_full(geometry);
    let residual = recomputed_residual(geometry);
    let jia_after = jia_after_patch_copied_after_admission(geometry).total();
    let (copied_before, copied_after, recomputed) = match arm {
        Arm::JiaAfterPatch => (jia_after_patch_copied_before_admission(geometry).total(geometry.knobs.settle_rounds), jia_after, in_full),
        Arm::JiaToday => (jia_today_copied_before_admission(geometry), jia_after, in_full),
        Arm::Yi2 => {
            let (before, after) = yi2_copied(geometry);
            (before, after, residual)
        }
        Arm::Bing => (0, bing_copied_after_admission(geometry), residual),
        Arm::DingWithoutCompaction => (0, ding_copied_after_admission(geometry, false, Compaction::WholeNewBase), residual),
        Arm::DingWholeNewBaseCompaction => (0, ding_copied_after_admission(geometry, true, Compaction::WholeNewBase), (residual.0, in_full.1)),
    };
    ArmBytes { copied_before, copied_after, recomputed_before: recomputed.0, recomputed_after: recomputed.1 }
}

/// 一格判定落在门槛的哪一侧；分母 0 判「不低于门槛」（读法第 9 条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    BelowThreshold,
    AtOrAboveThreshold,
}

/// 门槛写成分数：分子、分母（判据门槛 2 = (2, 1)）。
const JUDGED_THRESHOLD: (u64, u64) = (2, 1);

fn side_of(numerator: u64, denominator: u64, threshold: (u64, u64)) -> Side {
    if denominator == 0 || u128::from(numerator) * u128::from(threshold.1) >= u128::from(threshold.0) * u128::from(denominator) {
        Side::AtOrAboveThreshold
    } else {
        Side::BelowThreshold
    }
}

fn ratio_text(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        "zero_denominator".to_string()
    } else {
        format!("{:.4}", numerator as f64 / denominator as f64)
    }
}

/// 字节比那几行的判定：两种形式都 < 2 倾向留甲、都 ≥ 2 不倾向、分到两侧不稳定（读法第 9 条）。
fn verdict_of_sides(sides: &[Side]) -> &'static str {
    if sides.iter().all(|side| *side == Side::BelowThreshold) {
        "leans_keep_jia"
    } else if sides.iter().all(|side| *side == Side::AtOrAboveThreshold) {
        "does_not_lean_keep_jia"
    } else {
        "unstable"
    }
}

/// 比较的三对：甲-B2后 ÷ 乙2、÷ 丙、÷ 丁（丁取非压实那次；读法第 7 条逐对各算）。
const COMPARED_ARMS: [Arm; 3] = [Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction];

/// 判定那几格的一格：哪一对、哪一段、照抄还是现算重建。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct JudgedCell {
    compared_arm: Arm,
    after_admission: bool,
    recomputed: bool,
}

fn judged_cells() -> Vec<JudgedCell> {
    let mut judged = Vec::new();
    for recomputed in [false, true] {
        for after_admission in [false, true] {
            for compared_arm in COMPARED_ARMS {
                judged.push(JudgedCell { compared_arm, after_admission, recomputed });
            }
        }
    }
    judged
}

impl JudgedCell {
    fn label(&self) -> String {
        let kind = if self.recomputed { "recomputed" } else { "copied" };
        let segment = if self.after_admission { "after_admission" } else { "before_admission" };
        format!("jia_after_patch_over_{}_{kind}_{segment}", self.compared_arm.label())
    }

    /// 这一格的 (分子, 分母) 两种形式：照抄格给形式 A（只照抄）与形式 B（照抄 + 新装）；现算重建格只有一种形式。
    fn forms(&self, geometry: &Geometry) -> Vec<(u64, u64)> {
        let jia = arm_bytes(geometry, Arm::JiaAfterPatch);
        let other = arm_bytes(geometry, self.compared_arm);
        let (installed_before, installed_after) = installed_bytes(geometry);
        let pick = |bytes: &ArmBytes| match (self.recomputed, self.after_admission) {
            (false, false) => bytes.copied_before,
            (false, true) => bytes.copied_after,
            (true, false) => bytes.recomputed_before,
            (true, true) => bytes.recomputed_after,
        };
        if self.recomputed {
            vec![(pick(&jia), pick(&other))]
        } else {
            let installed = if self.after_admission { installed_after } else { installed_before };
            vec![(pick(&jia), pick(&other)), (pick(&jia) + installed, pick(&other) + installed)]
        }
    }

    fn verdict(&self, geometry: &Geometry, threshold: (u64, u64)) -> &'static str {
        let sides: Vec<Side> = self.forms(geometry).iter().map(|&(numerator, denominator)| side_of(numerator, denominator, threshold)).collect();
        verdict_of_sides(&sides)
    }
}

// ── 读路径跳数与常驻峰值（第一份第五节「读路径与常驻峰值」；第二份加现算重建与残余式） ──

/// 乙2 读一条墓碑记录、读一个单元比甲多走几跳。
fn yi2_extra_hops(geometry: &Geometry) -> (u64, u64) {
    let fanout = geometry.knobs.persistent_fanout;
    let block = geometry.knobs.persistent_leaf_block;
    let tombstones = &geometry.tombstones;
    let record = persistent_depth(tombstones.containers.div_ceil(fanout), fanout) + 1 + persistent_depth(tombstones.fill.div_ceil(block), fanout) + 1;
    let unit = persistent_depth(geometry.units.div_ceil(fanout), fanout) + 1;
    (record, unit)
}

/// 甲的常驻峰值 = |V| + |V'| + 一轮准入前照抄（O + Rl）+ 一轮现算重建（准入前那一份 ÷ s）+ 准入后现算重建。
fn jia_resident_peak(geometry: &Geometry) -> u64 {
    let rounds = jia_after_patch_copied_before_admission(geometry);
    let (recomputed_before, recomputed_after) = recomputed_in_full(geometry);
    version_bytes(geometry)
        + version_bytes_after(geometry)
        + rounds.ordinary_allocation_pass
        + rounds.settle_round
        + recomputed_before / geometry.knobs.settle_rounds
        + recomputed_after
}

/// 丙的常驻峰值 = |V| + 新装 + 撤回日志 + 被换下的 k 个容器与 ρ 个节点的旧字节 + 残余式（准入前 ÷ s 与准入后）。
fn bing_resident_peak(geometry: &Geometry) -> u64 {
    let replaced_old_bytes: u64 =
        geometry.container_trees().iter().map(|tree| tree.open_container_rewritten * UNIT_BYTES + tree.rewritten_index_nodes * NODE_BYTES).sum();
    let (residual_before, residual_after) = recomputed_residual(geometry);
    version_bytes(geometry)
        + installed_bytes(geometry).1
        + bing_copied_after_admission(geometry)
        + replaced_old_bytes
        + residual_before / geometry.knobs.settle_rounds
        + residual_after
}

// ── 两段历史：丁的 256 次连发与 10⁶ 次 unlink 累计（第六节；第十四节：串内不切，串与串之间独立） ──

/// 丁的压实周期取样点与主值（第八节逐一扫；G0 取 p = 64）。
const DING_PERIODS: [u64; 7] = [1, 2, 3, 4, 8, 64, 1024];
const DING_MAIN_PERIOD: u64 = 64;
/// 256 次连发（G0 起，a_T = 1、u 逐次加一）。
const DING_HISTORY_PUBLISHES: u64 = 256;
/// 10⁶ 次 unlink（每次 unlink 发两次：意图 a_T = 1、最后一批 a_T = 0）。
const CUMULATIVE_UNLINKS: u64 = 1_000_000;

fn add_arm_bytes(totals: &mut ArmBytes, bytes: ArmBytes) {
    totals.copied_before += bytes.copied_before;
    totals.copied_after += bytes.copied_after;
    totals.recomputed_before += bytes.recomputed_before;
    totals.recomputed_after += bytes.recomputed_after;
}

/// 丁这一次发布：压实那次照压实方式照抄，现算重建准入后取全量；非压实那次照抄 0、残余式。
fn ding_bytes(geometry: &Geometry, compacts_now: bool, compaction: Compaction) -> ArmBytes {
    let residual = recomputed_residual(geometry);
    ArmBytes {
        copied_before: 0,
        copied_after: ding_copied_after_admission(geometry, compacts_now, compaction),
        recomputed_before: residual.0,
        recomputed_after: if compacts_now { recomputed_in_full(geometry).1 } else { residual.1 },
    }
}

/// 一段历史上每臂的合计。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct HistoryTotals {
    publishes: u64,
    jia: ArmBytes,
    yi2: ArmBytes,
    bing: ArmBytes,
    ding: ArmBytes,
    installed_before: u64,
    installed_after: u64,
}

impl HistoryTotals {
    fn add_publish(&mut self, geometry: &Geometry, ding: ArmBytes) {
        self.publishes += 1;
        add_arm_bytes(&mut self.jia, arm_bytes(geometry, Arm::JiaAfterPatch));
        add_arm_bytes(&mut self.yi2, arm_bytes(geometry, Arm::Yi2));
        add_arm_bytes(&mut self.bing, arm_bytes(geometry, Arm::Bing));
        add_arm_bytes(&mut self.ding, ding);
        let (installed_before, installed_after) = installed_bytes(geometry);
        self.installed_before += installed_before;
        self.installed_after += installed_after;
    }

    fn merge(&mut self, other: &HistoryTotals) {
        self.publishes += other.publishes;
        add_arm_bytes(&mut self.jia, other.jia);
        add_arm_bytes(&mut self.yi2, other.yi2);
        add_arm_bytes(&mut self.bing, other.bing);
        add_arm_bytes(&mut self.ding, other.ding);
        self.installed_before += other.installed_before;
        self.installed_after += other.installed_after;
    }

    fn arm(&self, arm: Arm) -> ArmBytes {
        match arm {
            Arm::Yi2 => self.yi2,
            Arm::Bing => self.bing,
            Arm::DingWithoutCompaction | Arm::DingWholeNewBaseCompaction => self.ding,
            Arm::JiaAfterPatch | Arm::JiaToday => self.jia,
        }
    }
}

/// 序号落在 `unlinks` 里的那几次 unlink（第 k 次：u = k 上的意图，u = k + 1 上的最后一批）；丁照 p 在第 p、2p、… 次发布压实。
fn cumulative_unlinks_between(unlinks: std::ops::Range<u64>, ding_period: u64) -> HistoryTotals {
    let mut totals = HistoryTotals::default();
    for unlink in unlinks {
        let intent = Knobs { tombstone_records: unlink, tombstones_added: 1, ..Knobs::main_geometry() };
        let final_batch = Knobs { tombstone_records: unlink + 1, tombstones_added: 0, ..Knobs::main_geometry() };
        for (publish_number, knobs) in [(2 * unlink + 1, intent), (2 * unlink + 2, final_batch)] {
            let geometry = Geometry::new(knobs);
            let ding = ding_bytes(&geometry, publish_number % ding_period == 0, Compaction::WholeNewBase);
            totals.add_publish(&geometry, ding);
        }
    }
    totals
}

/// 把 [0, unlinks) 按序号切成 `threads` 片并行算，按片的次序合并（合计是整数加法，与线程数无关）。
fn cumulative_unlinks(unlinks: u64, threads: usize) -> HistoryTotals {
    let slices = (threads.max(1) as u64).min(unlinks.max(1));
    let slice_length = unlinks.div_ceil(slices);
    let ranges: Vec<std::ops::Range<u64>> =
        (0..slices).map(|slice| (slice * slice_length).min(unlinks)..((slice + 1) * slice_length).min(unlinks)).collect();
    let partials: Vec<HistoryTotals> = std::thread::scope(|scope| {
        let handles: Vec<_> =
            ranges.iter().cloned().map(|range| scope.spawn(move || cumulative_unlinks_between(range, DING_MAIN_PERIOD))).collect();
        handles.into_iter().map(|handle| handle.join().expect("累计那一片的线程不该 panic")).collect()
    });
    let mut totals = HistoryTotals::default();
    for partial in &partials {
        totals.merge(partial);
    }
    totals
}

/// 丁一个 p、一种压实方式在 256 次连发上的轨迹与合计。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DingHistory {
    period: u64,
    compaction: Compaction,
    compaction_copied_total: u64,
    ding: ArmBytes,
    layer_peak: u64,
    layer_positive_publishes: u64,
    layer_final: u64,
    resident_peak: u64,
    resident_final: u64,
}

/// 256 次连发里第 n 次（从 1 数）的几何：u = 10⁶ + n − 1、a_T = 1，其余取 G0。
fn ding_history_geometries() -> Vec<Geometry> {
    let main = Knobs::main_geometry();
    (0..DING_HISTORY_PUBLISHES)
        .map(|offset| Geometry::new(Knobs { tombstone_records: main.tombstone_records + offset, ..main }))
        .collect()
}

/// 丁第 n 次发布的常驻字节（第四份第五节「常驻峰值」丁那一式）：基版 + 已有各层 + 这次准入后新装 + [压实且整份新基]·|V'|
/// + 残余式准入前 ÷ s + 准入后那一份（压实那次取甲-B2后的准入后现算重建，两种压实方式都取它；不压实取残余式）。
fn ding_resident_bytes(geometry: &Geometry, base_version: u64, layer_bytes: u64, compacts_now: bool, compaction: Compaction) -> u64 {
    let (residual_before, residual_after) = recomputed_residual(geometry);
    let new_base_copy = if compacts_now && compaction == Compaction::WholeNewBase { version_bytes_after(geometry) } else { 0 };
    let recomputed_after_in_the_peak = if compacts_now { recomputed_in_full(geometry).1 } else { residual_after };
    base_version + layer_bytes + installed_bytes(geometry).1 + new_base_copy + residual_before / geometry.knobs.settle_rounds + recomputed_after_in_the_peak
}

fn ding_history(geometries: &[Geometry], period: u64, compaction: Compaction) -> DingHistory {
    let mut history = DingHistory {
        period,
        compaction,
        compaction_copied_total: 0,
        ding: ArmBytes::default(),
        layer_peak: 0,
        layer_positive_publishes: 0,
        layer_final: 0,
        resident_peak: 0,
        resident_final: 0,
    };
    let mut base_version = geometries.first().map(version_bytes).unwrap_or(0);
    let mut layer_bytes = 0;
    for (offset, geometry) in geometries.iter().enumerate() {
        let publish_number = offset as u64 + 1;
        let compacts_now = publish_number % period == 0;
        let bytes = ding_bytes(geometry, compacts_now, compaction);
        history.compaction_copied_total += bytes.copied_after;
        add_arm_bytes(&mut history.ding, bytes);
        let installed_after = installed_bytes(geometry).1;
        let resident = ding_resident_bytes(geometry, base_version, layer_bytes, compacts_now, compaction);
        history.resident_peak = history.resident_peak.max(resident);
        history.resident_final = resident;
        if compacts_now {
            base_version = version_bytes_after(geometry);
            layer_bytes = 0;
        } else {
            layer_bytes += installed_after;
        }
        let layers = publish_number % period;
        history.layer_peak = history.layer_peak.max(layers);
        history.layer_positive_publishes += u64::from(layers > 0);
        history.layer_final = layers;
    }
    history
}

// ── 取样点（第八节：G0、逐一扫、事件点、等高点、G-small） ──

fn sampling_points() -> Vec<(String, Knobs)> {
    let main = Knobs::main_geometry();
    let mut points = vec![("main_geometry".to_string(), main)];
    for tombstone_records in [0, 10, 1000, 10_000, 100_000, 10_000_000, 100_000_000] {
        points.push((format!("tombstone_records_{tombstone_records}"), Knobs { tombstone_records, ..main }));
    }
    for tombstones_added in [0, 8150, 81_500, 815_000] {
        points.push((format!("tombstones_added_{tombstones_added}"), Knobs { tombstones_added, ..main }));
    }
    for settle_rounds in [1, 3, 64] {
        points.push((format!("settle_rounds_{settle_rounds}"), Knobs { settle_rounds, ..main }));
    }
    for retirement_records in [0, 1_000_000] {
        points.push((format!("retirement_records_{retirement_records}"), Knobs { retirement_records, ..main }));
    }
    for retirements_added in [1, 959] {
        points.push((format!("retirements_added_{retirements_added}"), Knobs { retirements_added, ..main }));
    }
    points.push(("lineage_records_0".to_string(), Knobs { lineage_records: 0, ..main }));
    points.push(("lineage_records_added_1".to_string(), Knobs { lineage_records_added: 1, ..main }));
    for live_inodes in [1, 1_000_000] {
        points.push((format!("live_inodes_{live_inodes}"), Knobs { live_inodes, ..main }));
    }
    points.push(("file_data_units_32768".to_string(), Knobs { file_data_units: 32768, ..main }));
    points.push(("mapping_leaves_half_full".to_string(), Knobs { mapping_leaves_half_full: true, ..main }));
    points.push(("touched_leaves_focused".to_string(), Knobs { touched_leaves: TouchedLeaves::Focused, ..main }));
    for persistent_fanout in [16, 32, 135] {
        for persistent_leaf_block in [8, 64, 815] {
            if (persistent_fanout, persistent_leaf_block) != (main.persistent_fanout, main.persistent_leaf_block) {
                let label = format!("persistent_fanout_{persistent_fanout}_leaf_block_{persistent_leaf_block}");
                points.push((label, Knobs { persistent_fanout, persistent_leaf_block, ..main }));
            }
        }
    }
    // 事件点：开新片（815 × 1227 = 1000005，fill = 815、room = 0）与长高（815 × 135 = 110025，追加 1 条 C' = 136）。
    points.push(("open_new_container".to_string(), Knobs { tombstone_records: 1_000_005, ..main }));
    points.push(("tree_grows_a_level".to_string(), Knobs { tombstone_records: 110_025, ..main }));
    // 等高点（集中档）：C = 6401 与 C = 16001。
    for tombstone_records in [5_216_400, 13_040_400] {
        points.push((format!("equal_height_{tombstone_records}"), Knobs { tombstone_records, touched_leaves: TouchedLeaves::Focused, ..main }));
    }
    points.push(("small_geometry".to_string(), Knobs::small_geometry()));
    // 第三份第七节「新补两处逐项」那一点：u = 10⁸、a_T = 0。
    points.push(("tombstone_records_100000000_not_appending".to_string(), Knobs { tombstone_records: 100_000_000, tombstones_added: 0, ..main }));
    points
}

/// 判定那几格在 G0 之外重跑的点（第八节「几何敏感性」）与只报数的两点（第三份：a_T = 0、D = 32768）。
const SENSITIVITY_POINTS: [&str; 6] =
    ["small_geometry", "settle_rounds_1", "tombstones_added_815000", "live_inodes_1", "mapping_leaves_half_full", "touched_leaves_focused"];
const REPORTED_ONLY_POINTS: [&str; 2] = ["tombstones_added_0", "file_data_units_32768"];

fn levels_text(levels: &[u64]) -> String {
    levels.iter().map(u64::to_string).collect::<Vec<_>>().join("/")
}

/// 一个取样点的全部产物行：几何、各臂四样字节、共有的几样、读路径跳数、常驻峰值、每格比值。
fn point_lines(label: &str, geometry: &Geometry) -> Vec<String> {
    let tombstones = &geometry.tombstones;
    let mut lines = vec![format!(
        "name=geometry point={label} tombstone_containers={} tombstone_levels={} tombstone_fill={} open_container_rewritten={} new_containers={} \
         rewritten_index_nodes={} tombstone_containers_after={} tombstone_levels_after={} inode_leaves={} core_rewritten_units={} core_new_units={} \
         slots_per_device={} allocation_levels={} allocation_levels_after={} allocation_changed_records={} allocation_touched_leaves={} \
         allocation_rewritten_nodes={} allocation_carried_nodes={} mapping_leaf_entries={} mapping_entries={} mapping_entries_after={} \
         mapping_levels={} mapping_levels_after={} mapping_rewritten_units={} mapping_new_entries={} mapping_changed_entries={} \
         mapping_touched_leaves={} mapping_rewritten_nodes={} mapping_carried_nodes={} units={} units_after={} rewritten_units={} new_units={} \
         touched_leaves={}",
        tombstones.containers,
        levels_text(&tombstones.levels),
        tombstones.fill,
        tombstones.open_container_rewritten,
        tombstones.new_containers,
        tombstones.rewritten_index_nodes,
        tombstones.containers_after,
        levels_text(&tombstones.levels_after),
        geometry.inode_leaves,
        geometry.core_rewritten_units,
        geometry.core_new_units,
        geometry.slots_per_device,
        levels_text(&geometry.allocation_levels),
        levels_text(&geometry.allocation_levels_after),
        geometry.allocation_changed_records,
        geometry.allocation_touched_leaves,
        geometry.allocation_rewritten_nodes,
        geometry.allocation_carried_nodes,
        geometry.mapping_leaf_entries,
        geometry.mapping_entries,
        geometry.mapping_entries_after,
        levels_text(&geometry.mapping_levels),
        levels_text(&geometry.mapping_levels_after),
        geometry.mapping_rewritten_units,
        geometry.mapping_new_entries,
        geometry.mapping_changed_entries,
        geometry.mapping_touched_leaves,
        geometry.mapping_rewritten_nodes,
        geometry.mapping_carried_nodes,
        geometry.units,
        geometry.units_after,
        geometry.rewritten_units,
        geometry.new_units,
        geometry.knobs.touched_leaves.label(),
    )];
    for arm in Arm::EVERY_ARM {
        let bytes = arm_bytes(geometry, arm);
        lines.push(format!(
            "name=arm_bytes point={label} arm={} copied_before={} copied_after={} recomputed_before={} recomputed_after={}",
            arm.label(),
            bytes.copied_before,
            bytes.copied_after,
            bytes.recomputed_before,
            bytes.recomputed_after
        ));
    }
    lines.extend(shared_lines(label, geometry));
    for cell in judged_cells() {
        let forms: Vec<String> = cell.forms(geometry).iter().map(|&(numerator, denominator)| ratio_text(numerator, denominator)).collect();
        lines.push(format!("name=ratio point={label} cell={} forms={} side_at_two={}", cell.label(), forms.join("/"), cell.verdict(geometry, JUDGED_THRESHOLD)));
    }
    lines
}

/// 一个取样点上四条臂共有的几样，与只给甲报的附带量、甲的分项、第三份补进的四项、读路径跳数、常驻峰值。
fn shared_lines(label: &str, geometry: &Geometry) -> Vec<String> {
    let settle_rounds = geometry.knobs.settle_rounds;
    let (installed_before, installed_after) = installed_bytes(geometry);
    let rounds = jia_after_patch_copied_before_admission(geometry);
    let after = jia_after_patch_copied_after_admission(geometry);
    let added = items_added_by_the_third_registration(geometry);
    let jia_after_patch_before = rounds.total(settle_rounds);
    let (yi2_record_hops, yi2_unit_hops) = yi2_extra_hops(geometry);
    let jia_peak = jia_resident_peak(geometry);
    let bing_peak = bing_resident_peak(geometry);
    let yi1_rounds = yi1_copied_before_admission(geometry, ReferenceSharing::Shared);
    vec![
        // 乙1（附带，只报数、不判；第四份第五节「乙1 的照抄字节」拼法）。
        format!(
            "name=yi1_copied point={label} arm=yi1_reference_counted copied_before={} copied_after={} ordinary_allocation_pass={} settle_round={} outside_the_rounds={} judged=no",
            yi1_rounds.total(settle_rounds),
            yi1_copied_after_admission(geometry, ReferenceSharing::Shared),
            yi1_rounds.ordinary_allocation_pass,
            yi1_rounds.settle_round,
            yi1_rounds.outside_the_rounds,
        ),
        format!(
            "name=shared_bytes point={label} installed_before={installed_before} installed_after={installed_after} \
             installed_unit_bytes_second_copy={} installed_unit_bytes_publish_writes_copy={} version_bytes={} version_bytes_after={} \
             jia_today_minus_jia_after_patch={}",
            installed_unit_bytes(geometry),
            installed_unit_bytes(geometry),
            version_bytes(geometry),
            version_bytes_after(geometry),
            jia_today_copied_before_admission(geometry) - jia_after_patch_before,
        ),
        format!(
            "name=jia_after_patch_parts point={label} ordinary_allocation_pass={} settle_round={} outside_the_rounds={} inode_leaves_again={} \
             mapping_carried_nodes_each_copy={} multi_level_shapes={} item_8640={} item_8965={} item_14640={} item_14643={}",
            rounds.ordinary_allocation_pass,
            rounds.settle_round,
            rounds.outside_the_rounds,
            after.inode_leaves_again,
            after.mapping_carried_nodes_each_copy,
            after.multi_level_shapes,
            added.tombstone_shape_when_not_appending,
            added.lineage_shape_when_not_appending,
            added.carried_file_data_units,
            added.carried_file_data_pointers,
        ),
        format!(
            "name=read_hops point={label} jia_extra=0 bing_extra=0 yi2_record_extra={yi2_record_hops} yi2_unit_extra={yi2_unit_hops} \
             yi1_record_extra=1 yi1_unit_extra=1 ding_extra=layers_since_compaction_0_to_{}",
            DING_MAIN_PERIOD - 1
        ),
        format!(
            "name=resident_peak point={label} jia_after_patch={jia_peak} bing_by_value={bing_peak} bing_over_jia={} bing_side_at_one={}",
            ratio_text(bing_peak, jia_peak),
            resident_peak_side(bing_peak, jia_peak)
        ),
    ]
}

/// 常驻峰值那一行：臂 ÷ 甲 ≥ 1 判「不比甲省」，< 1 判「比甲省」（门槛 1）。
fn resident_peak_side(arm_peak: u64, jia_peak: u64) -> &'static str {
    match side_of(arm_peak, jia_peak, (1, 1)) {
        Side::AtOrAboveThreshold => "not_less_than_jia",
        Side::BelowThreshold => "less_than_jia",
    }
}

fn geometry_of_point(points: &[(String, Knobs)], label: &str) -> Geometry {
    let knobs = points.iter().find(|(name, _)| name == label).map(|(_, knobs)| *knobs).expect("点名的取样点在取样点表里");
    Geometry::new(knobs)
}

/// 甲的增长阶：u = 10⁵→10⁶、10⁶→10⁷ 两段上的 log-log 斜率（照抄准入前；现算重建准入前与准入后）。
fn growth_order_lines(points: &[(String, Knobs)]) -> Vec<String> {
    let geometries = [
        ("1e5", geometry_of_point(points, "tombstone_records_100000")),
        ("1e6", geometry_of_point(points, "main_geometry")),
        ("1e7", geometry_of_point(points, "tombstone_records_10000000")),
    ];
    let quantities: [(&str, fn(&Geometry) -> u64); 3] = [
        ("jia_after_patch_copied_before", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).copied_before),
        ("jia_after_patch_recomputed_before", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).recomputed_before),
        ("jia_after_patch_recomputed_after", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).recomputed_after),
    ];
    let mut lines = Vec::new();
    for (quantity, measure) in quantities {
        for pair in geometries.windows(2) {
            let (lower, upper) = (measure(&pair[0].1), measure(&pair[1].1));
            lines.push(format!(
                "name=growth_order quantity={quantity} from_tombstone_records={} to_tombstone_records={} lower={lower} upper={upper} slope={:.5}",
                pair[0].0,
                pair[1].0,
                (upper as f64 / lower as f64).log10()
            ));
        }
    }
    lines
}

/// 阳性对照（每条臂都跑）：u = 10⁶ 对 u = 10⁵ 的比；照抄 ≥ 9，现算重建 ≥ 8（第一份第五节那张表，第二份另加一条；
/// 乙2、乙1、丙、丁的退化形态照第四份第五节写死）。
fn positive_control_lines(points: &[(String, Knobs)]) -> (Vec<String>, Vec<(String, String)>) {
    let lower = geometry_of_point(points, "tombstone_records_100000");
    let upper = geometry_of_point(points, "main_geometry");
    let mut lines = Vec::new();
    let mut verdict_fields = Vec::new();
    let degenerate_forms: [(&str, fn(&Geometry) -> u64); 5] = [
        ("jia_after_patch_copied_before", |geometry| arm_bytes(geometry, Arm::JiaAfterPatch).copied_before),
        ("yi2_persistent_sharing_turned_off", |geometry| yi2_sharing_turned_off_copied_before_admission(geometry).total(geometry.knobs.settle_rounds)),
        ("yi1_reference_counted_sharing_turned_off", |geometry| {
            yi1_copied_before_admission(geometry, ReferenceSharing::TurnedOff).total(geometry.knobs.settle_rounds)
        }),
        ("bing_cloned_before_handing_in", |geometry| bing_copied_after_admission(geometry) + version_bytes(geometry)),
        ("ding_period_1_whole_new_base", |geometry| ding_copied_after_admission(geometry, true, Compaction::WholeNewBase)),
    ];
    for (arm, measure) in degenerate_forms {
        let (lower_bytes, upper_bytes) = (measure(&lower), measure(&upper));
        let holds = side_of(upper_bytes, lower_bytes, (9, 1)) == Side::AtOrAboveThreshold && lower_bytes > 0;
        lines.push(format!("name=positive_control arm={arm} tombstone_records_1e5={lower_bytes} tombstone_records_1e6={upper_bytes} ratio={} threshold=9 holds={holds}", ratio_text(upper_bytes, lower_bytes)));
        verdict_fields.push((format!("positive_control_{arm}"), holds.to_string()));
    }
    // 现算重建的阳性对照：乙2、丙、丁的残余式关掉（改成按全部单元现算），准入后比 ≥ 8。
    for arm in [Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction] {
        let (lower_bytes, upper_bytes) = (recomputed_in_full(&lower).1, recomputed_in_full(&upper).1);
        let holds = side_of(upper_bytes, lower_bytes, (8, 1)) == Side::AtOrAboveThreshold && lower_bytes > 0;
        lines.push(format!(
            "name=recompute_positive_control arm={} tombstone_records_1e5={lower_bytes} tombstone_records_1e6={upper_bytes} ratio={} threshold=8 holds={holds}",
            arm.label(),
            ratio_text(upper_bytes, lower_bytes)
        ));
        verdict_fields.push((format!("recompute_positive_control_{}", arm.label()), holds.to_string()));
    }
    // 附带（第四份第六节另加那一行）：执行员头一种读法那一比，报数、不判，不进判决行、不触发作废条款第 3 条。
    let touched_whole = |geometry: &Geometry| yi2_touched_collections_whole_copied_before_admission(geometry).total(geometry.knobs.settle_rounds);
    let (lower_bytes, upper_bytes) = (touched_whole(&lower), touched_whole(&upper));
    lines.push(format!(
        "name=positive_control_reported_only arm=yi2_persistent_touched_collections_whole tombstone_records_1e5={lower_bytes} tombstone_records_1e6={upper_bytes} ratio={} judged=no reason=registered_reading_is_sharing_turned_off",
        ratio_text(upper_bytes, lower_bytes)
    ));
    (lines, verdict_fields)
}

/// G0 上判定那几格，与几何敏感性：同一格在敏感性各点上重跑，翻面记「不稳定」并写明哪一点翻；a_T = 0、D = 32768 只报数。
fn judgement_lines(points: &[(String, Knobs)]) -> (Vec<String>, bool) {
    let main = geometry_of_point(points, "main_geometry");
    let mut lines = Vec::new();
    for cell in judged_cells() {
        let main_verdict = cell.verdict(&main, JUDGED_THRESHOLD);
        let mut flipped = Vec::new();
        let mut at_points = Vec::new();
        for point in SENSITIVITY_POINTS {
            let verdict = cell.verdict(&geometry_of_point(points, point), JUDGED_THRESHOLD);
            if verdict != main_verdict {
                flipped.push(point);
            }
            at_points.push(format!("{point}:{verdict}"));
        }
        let reported_only: Vec<String> =
            REPORTED_ONLY_POINTS.iter().map(|point| format!("{point}:{}", cell.verdict(&geometry_of_point(points, point), JUDGED_THRESHOLD))).collect();
        let stability = if flipped.is_empty() { "stable".to_string() } else { format!("unstable_flipped_at_{}", flipped.join("+")) };
        lines.push(format!(
            "name=judgement point=main_geometry cell={} verdict={main_verdict} sensitivity={stability} at_points={} reported_only={}",
            cell.label(),
            at_points.join(","),
            reported_only.join(",")
        ));
    }
    // 判别力自证：门槛挪到 G0 与 G-small 两个值之间，那一格的检查必须由一侧转到另一侧（登记给的两例：照抄门槛 1，现算重建门槛 5）。
    let small = geometry_of_point(points, "small_geometry");
    let mut every_example_flips = true;
    let examples = [
        (JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: false }, (1, 1)),
        (JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: true }, (5, 1)),
        (JudgedCell { compared_arm: Arm::Yi2, after_admission: true, recomputed: true }, (5, 1)),
    ];
    for (cell, threshold) in examples {
        let (main_side, small_side) = (cell.verdict(&main, threshold), cell.verdict(&small, threshold));
        let flips = main_side != small_side;
        every_example_flips &= flips;
        lines.push(format!(
            "name=discriminative_power cell={} threshold={}/{} main_geometry={main_side} small_geometry={small_side} flips={flips}",
            cell.label(),
            threshold.0,
            threshold.1
        ));
    }
    for cell in judged_cells() {
        let forms = cell.forms(&main).into_iter().zip(cell.forms(&small));
        for (form_index, ((main_numerator, main_denominator), (small_numerator, small_denominator))) in forms.enumerate() {
            let form = if form_index == 0 { "a" } else { "b" };
            let outcome = if main_denominator == 0 || small_denominator == 0 {
                "zero_denominator_no_threshold_between".to_string()
            } else {
                // 门槛取两点比值的中点：(p₀/q₀ + p₁/q₁) ÷ 2。
                let threshold = (main_numerator * small_denominator + small_numerator * main_denominator, 2 * main_denominator * small_denominator);
                let flips = side_of(main_numerator, main_denominator, threshold) != side_of(small_numerator, small_denominator, threshold);
                format!("midpoint_flips_{flips}")
            };
            lines.push(format!("name=discriminative_power_midpoint cell={} form={form} outcome={outcome}", cell.label()));
        }
    }
    (lines, every_example_flips)
}

/// 丁的 256 次连发：每个 p、每种压实方式一行（轨迹、合计、摊销、甲 ÷ 丁），另报甲的合计与「比值 ≥ 2 的最小 p」。
fn ding_history_lines() -> Vec<String> {
    let geometries = ding_history_geometries();
    let mut jia = HistoryTotals::default();
    for geometry in &geometries {
        jia.add_publish(geometry, ArmBytes::default());
    }
    let mut lines = vec![format!(
        "name=ding_history_jia publishes={} jia_copied_before_total={} jia_copied_after_total={} jia_recomputed_before_total={} jia_recomputed_after_total={} \
         installed_before_total={} installed_after_total={}",
        jia.publishes,
        jia.jia.copied_before,
        jia.jia.copied_after,
        jia.jia.recomputed_before,
        jia.jia.recomputed_after,
        jia.installed_before,
        jia.installed_after
    )];
    let jia_peaks: Vec<u64> = geometries.iter().map(jia_resident_peak).collect();
    let jia_peak = jia_peaks.iter().copied().max().unwrap_or(0);
    for compaction in [Compaction::WholeNewBase, Compaction::FoldInPlace] {
        let mut smallest_period_at_least_two = None;
        for period in DING_PERIODS {
            let history = ding_history(&geometries, period, compaction);
            let copied_without_installed = side_of(jia.jia.copied_after, history.ding.copied_after, JUDGED_THRESHOLD);
            if copied_without_installed == Side::AtOrAboveThreshold && smallest_period_at_least_two.is_none() {
                smallest_period_at_least_two = Some(period);
            }
            lines.push(format!(
                "name=ding_history period={} compaction={} compaction_copied_total={} amortized_per_publish={} layer_peak={} layer_positive_publishes={} \
                 layer_final={} jia_over_ding_copied_after_without_installed={} jia_over_ding_copied_after_with_installed={} jia_over_ding_copied_before={} \
                 jia_over_ding_recomputed_before={} jia_over_ding_recomputed_after={} ding_resident_peak={} ding_resident_final={} \
                 ding_resident_peak_over_jia={} ding_resident_side_at_one={}",
                history.period,
                history.compaction.label(),
                history.compaction_copied_total,
                ratio_text(history.compaction_copied_total, DING_HISTORY_PUBLISHES),
                history.layer_peak,
                history.layer_positive_publishes,
                history.layer_final,
                ratio_text(jia.jia.copied_after, history.ding.copied_after),
                ratio_text(jia.jia.copied_after + jia.installed_after, history.ding.copied_after + jia.installed_after),
                ratio_text(jia.jia.copied_before, history.ding.copied_before),
                ratio_text(jia.jia.recomputed_before, history.ding.recomputed_before),
                ratio_text(jia.jia.recomputed_after, history.ding.recomputed_after),
                history.resident_peak,
                history.resident_final,
                ratio_text(history.resident_peak, jia_peak),
                resident_peak_side(history.resident_peak, jia_peak)
            ));
        }
        // 落在扫描端点照「扫描出来的极值落在端点」写成 ≤ / ≥ 端点。
        let smallest = match smallest_period_at_least_two {
            Some(period) if period == DING_PERIODS[0] => format!("at_most_{period}"),
            Some(period) => period.to_string(),
            None => format!("above_{}", DING_PERIODS[DING_PERIODS.len() - 1]),
        };
        lines.push(format!("name=ding_smallest_period compaction={} jia_resident_peak={jia_peak} smallest_period_with_copied_after_ratio_at_least_two={smallest}", compaction.label()));
    }
    lines
}

/// 累计里每臂在产物行里的名字：累计里的丁取每 `DING_MAIN_PERIOD` 次发布整份新基压实的那一条臂，名字说出这一点；
/// 每臂一行与三对比值的 `pair=` 都取它，分母用的是哪一条臂从名字就看得出。
fn cumulative_arm_name(arm: Arm) -> &'static str {
    if arm == Arm::DingWithoutCompaction { "ding_period_64_whole_new_base" } else { arm.label() }
}

/// 10⁶ 次 unlink 累计：每臂一行，及三对比值（照抄两种形式；现算重建一种形式）。
fn cumulative_lines(totals: &HistoryTotals) -> Vec<String> {
    let mut lines = vec![format!(
        "name=cumulative_unlinks unlinks={CUMULATIVE_UNLINKS} publishes={} ding_period={DING_MAIN_PERIOD} installed_before={} installed_after={}",
        totals.publishes, totals.installed_before, totals.installed_after
    )];
    for arm in [Arm::JiaAfterPatch, Arm::Yi2, Arm::Bing, Arm::DingWithoutCompaction] {
        let bytes = totals.arm(arm);
        let label = cumulative_arm_name(arm);
        lines.push(format!(
            "name=cumulative_arm arm={label} copied_before={} copied_after={} recomputed_before={} recomputed_after={}",
            bytes.copied_before, bytes.copied_after, bytes.recomputed_before, bytes.recomputed_after
        ));
    }
    for arm in COMPARED_ARMS {
        let (jia, other) = (totals.jia, totals.arm(arm));
        for (segment, jia_copied, other_copied, installed, jia_recomputed, other_recomputed) in [
            ("before_admission", jia.copied_before, other.copied_before, totals.installed_before, jia.recomputed_before, other.recomputed_before),
            ("after_admission", jia.copied_after, other.copied_after, totals.installed_after, jia.recomputed_after, other.recomputed_after),
        ] {
            let sides = [side_of(jia_copied, other_copied, JUDGED_THRESHOLD), side_of(jia_copied + installed, other_copied + installed, JUDGED_THRESHOLD)];
            lines.push(format!(
                "name=cumulative_ratio pair=jia_after_patch_over_{} segment={segment} copied_without_installed={} copied_with_installed={} copied_verdict={} recomputed={} recomputed_verdict={}",
                cumulative_arm_name(arm),
                ratio_text(jia_copied, other_copied),
                ratio_text(jia_copied + installed, other_copied + installed),
                verdict_of_sides(&sides),
                ratio_text(jia_recomputed, other_recomputed),
                verdict_of_sides(&[side_of(jia_recomputed, other_recomputed, JUDGED_THRESHOLD)])
            ));
        }
    }
    lines
}

/// 第七节「出自被测条款本身」「取实现值的模型常量」两组与内存宽度，在产物里现算一遍（单测另逐条钉）。
fn constant_check_lines() -> (Vec<String>, [bool; 3]) {
    let clause = [
        CODE_TWO_HEADER_BYTES + 2 * CODE_TWO_KEY_BYTES + CODE_TWO_FANOUT * CODE_TWO_ENTRY_BYTES + 53 == NODE_BYTES,
        CODE_THREE_RECORD_AREA_START + TOMBSTONE_CONTAINER_RECORDS * TOMBSTONE_RECORD_DISK_BYTES + 32 == UNIT_BYTES,
        CODE_THREE_RECORD_AREA_START + LINEAGE_CONTAINER_RECORDS * LINEAGE_RECORD_DISK_BYTES + 16 == UNIT_BYTES,
        CODE_THREE_RECORD_AREA_START + RETIREMENT_CONTAINER_RECORDS * RETIREMENT_RECORD_DISK_BYTES + 26 == UNIT_BYTES,
        CODE_THREE_RECORD_AREA_START + INODE_LEAF_RECORDS * INODE_RECORD_DISK_BYTES + 12 == UNIT_BYTES,
    ];
    let implementation = [
        ALLOCATION_RECORD_TREE_INTERNAL_HEADER_BYTES + ALLOCATION_RECORD_TREE_INTERNAL_FANOUT * ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES + 25 == NODE_BYTES,
        ACCOUNTING_ENTRIES == 16,
        DATA_POINTER_HEADER_BYTES + POSITION_ENTRY_BYTES * 2 + DATA_POINTER_WRITE_ORDER_BYTES == DATA_POINTER_BYTES,
        DATA_POINTER_MEMORY_BYTES == DATA_POINTER_BYTES,
    ];
    let registered_widths: [(&str, u64, u64); 23] = [
        ("reference_counted_pointer", REFERENCE_COUNTED_POINTER_BYTES, 8),
        ("tombstone_record", TOMBSTONE_RECORD_MEMORY_BYTES, 40),
        ("lineage_record", LINEAGE_RECORD_MEMORY_BYTES, 24),
        ("retirement_record", RETIREMENT_RECORD_MEMORY_BYTES, 40),
        ("inode_record", INODE_RECORD_MEMORY_BYTES, 64),
        ("allocation_record", ALLOCATION_RECORD_MEMORY_BYTES, 40),
        ("container_shell", CONTAINER_SHELL_BYTES, 56),
        ("unit_shell", UNIT_SHELL_BYTES, 48),
        ("shape_node", SHAPE_NODE_BYTES, 8),
        ("shape_level", SHAPE_LEVEL_BYTES, 24),
        ("mapping_key_structure", key_structure_bytes(CENTRAL_MAPPING_TREE), 115),
        ("accounting_key_structure", key_structure_bytes(ACCOUNTING_TREE), 102),
        ("deadlist_key_structure", key_structure_bytes(DEADLIST_TREE), 147),
        ("mapped_unit", MAPPED_UNIT_ENTRY_BYTES, 67),
        ("mapped_unit_with_locations", MAPPED_UNIT_WITH_LOCATIONS_BYTES, 95),
        ("mapping_entry_table", MAPPING_ENTRY_TABLE_BYTES, 194),
        ("mapping_key_bytes", MAPPING_KEY_BYTES_ENTRY_BYTES, 51),
        ("mapping_leaf_entry", MAPPING_LEAF_ENTRY_BYTES, 79),
        ("multi_level_shape_node", MULTI_LEVEL_SHAPE_NODE_BYTES, 32),
        ("allocation_tree_node_identity", ALLOCATION_TREE_NODE_IDENTITY_BYTES, 16),
        ("deleted_inode_number", DELETED_INODE_NUMBER_BYTES, 32),
        ("data_pointer", DATA_POINTER_MEMORY_BYTES, 88),
        ("carried_data_unit", CARRIED_DATA_UNIT_BYTES, 32792),
    ];
    let widths_hold = registered_widths.iter().all(|&(_, measured, registered)| measured == registered);
    let widths: Vec<String> = registered_widths.iter().map(|(name, measured, _)| format!("{name}={measured}")).collect();
    let holds = [clause.iter().all(|holds| *holds), implementation.iter().all(|holds| *holds), widths_hold];
    let lines = vec![
        format!("name=constant_checks clause_constants_hold={} implementation_constants_hold={} memory_widths_hold={}", holds[0], holds[1], holds[2]),
        format!("name=memory_widths {}", widths.join(" ")),
    ];
    (lines, holds)
}

/// 这一段没量、归第二段或不量的几样（第五节「实现量与分段」、第十一节「够判停机」），各写一行，不当 0。
fn not_in_this_segment_lines() -> Vec<String> {
    [
        "execution_clock_ratio reason=segment_two_measures_it",
        "estimated_clock reason=segment_two_measures_the_copy_bandwidth",
        "yi2_grows_with_containers reason=segment_two_counting_allocator_at_the_equal_height_points",
        "read_path_nanoseconds reason=segment_two",
        "resident_peak_of_yi1_and_yi2 reason=segment_two_counting_allocator",
        "whole_allocator_copy_per_round reason=out_of_range_not_measured",
        "extent_tree_nodes_and_other_files reason=zero_in_the_model_not_measured",
        "compaction_batch_cloning_the_previous_version reason=not_reached_by_the_main_geometry_not_measured",
        "implementation_comparison_segment reason=main_agent_decided_not_to_run_it",
    ]
    .iter()
    .map(|quantity| format!("name=not_in_this_segment quantity={quantity}"))
    .collect()
}

/// 整份产物（不含收尾行）：配置、常量、各取样点、增长阶、阳性对照、判定与敏感性、丁的历史、10⁶ 次 unlink 累计、这一段不量的、判决行。
fn counting_model_lines(threads: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config experiment=e199_transaction_output_copy segment=counting_model registration=research/prompts/e199-preregistration-r4.md \
         threads_cumulative_unlinks={threads} threads_sampling_points=1 threads_ding_history=1 unit_bytes={UNIT_BYTES} node_bytes={NODE_BYTES} \
         fanout={CODE_TWO_FANOUT} tombstone_capacity={TOMBSTONE_CONTAINER_RECORDS} lineage_capacity={LINEAGE_CONTAINER_RECORDS} \
         retirement_capacity={RETIREMENT_CONTAINER_RECORDS} inode_leaf_capacity={INODE_LEAF_RECORDS} allocation_leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} \
         allocation_fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} accounting_entries={ACCOUNTING_ENTRIES} fixed_point_nodes={FIXED_POINT_NODES} \
         instance_table_units={INSTANCE_TABLE_UNITS} mapping_entry_bytes={MAPPING_ENTRY_BYTES} data_pointer_bytes={DATA_POINTER_BYTES} \
         judged_threshold={}/{} judged_arm=jia_after_patch baseline_patch=b_cluster_item_two_in_the_main_worktree",
        JUDGED_THRESHOLD.0, JUDGED_THRESHOLD.1
    )];
    let (constant_lines, constant_holds) = constant_check_lines();
    lines.extend(constant_lines);
    let points = sampling_points();
    for (label, knobs) in &points {
        lines.extend(point_lines(label, &Geometry::new(*knobs)));
    }
    lines.extend(growth_order_lines(&points));
    let (positive_lines, positive_fields) = positive_control_lines(&points);
    lines.extend(positive_lines);
    let (judgement, every_example_flips) = judgement_lines(&points);
    lines.extend(judgement);
    lines.extend(ding_history_lines());
    lines.extend(cumulative_lines(&cumulative_unlinks(CUMULATIVE_UNLINKS, threads)));
    lines.extend(not_in_this_segment_lines());
    let positive: Vec<String> = positive_fields.iter().map(|(name, holds)| format!("{name}={holds}")).collect();
    lines.push(format!(
        "name=verdict segment=counting_model clause_constants_hold={} implementation_constants_hold={} memory_widths_hold={} \
         discriminative_power_examples_flip={every_example_flips} {} execution_clock=not_run",
        constant_holds[0],
        constant_holds[1],
        constant_holds[2],
        positive.join(" ")
    ));
    lines
}

fn main() {
    if std::env::args().len() > 1 {
        eprintln!("用法：e199-transaction-output-copy（不带参数；产物写到标准输出）");
        std::process::exit(2);
    }
    let threads = default_heavy_threads(include_str!("../../.claude/heavy-thread-default"));
    let lines = counting_model_lines(threads);
    let standard_output = std::io::stdout();
    let mut output = std::io::BufWriter::new(standard_output.lock());
    for line in &lines {
        writeln!(output, "E7RESULT {line}").expect("写得进标准输出");
    }
    writeln!(output, "E7RESULT name=done emitted={}", lines.len() + 1).expect("写得进标准输出");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn main_with(change: impl FnOnce(&mut Knobs)) -> Geometry {
        let mut knobs = Knobs::main_geometry();
        change(&mut knobs);
        Geometry::new(knobs)
    }

    fn jia_before(geometry: &Geometry) -> u64 {
        arm_bytes(geometry, Arm::JiaAfterPatch).copied_before
    }

    fn jia_after(geometry: &Geometry) -> u64 {
        arm_bytes(geometry, Arm::JiaAfterPatch).copied_after
    }

    // ── 第七节「出自被测条款本身的」（加法写，不写减法） ──

    #[test]
    fn unit_and_node_bytes_are_the_clause_values() {
        assert_eq!(UNIT_BYTES, 32768);
        assert_eq!(NODE_BYTES, 16384);
    }

    #[test]
    fn code_two_fanout_fills_a_node_with_53_bytes_left() {
        assert_eq!(CODE_TWO_HEADER_BYTES + 2 * CODE_TWO_KEY_BYTES + CODE_TWO_FANOUT * CODE_TWO_ENTRY_BYTES + 53, 16384);
        assert_eq!(CODE_TWO_FANOUT, 135);
    }

    #[test]
    fn code_three_containers_fill_a_unit() {
        assert_eq!(136 + TOMBSTONE_CONTAINER_RECORDS * TOMBSTONE_RECORD_DISK_BYTES + 32, 32768);
        assert_eq!(136 + LINEAGE_CONTAINER_RECORDS * LINEAGE_RECORD_DISK_BYTES + 16, 32768);
        assert_eq!(136 + RETIREMENT_CONTAINER_RECORDS * RETIREMENT_RECORD_DISK_BYTES + 26, 32768);
        assert_eq!(136 + INODE_LEAF_RECORDS * INODE_RECORD_DISK_BYTES + 12, 32768);
        assert_eq!((TOMBSTONE_CONTAINER_RECORDS, LINEAGE_CONTAINER_RECORDS, RETIREMENT_CONTAINER_RECORDS, INODE_LEAF_RECORDS), (815, 1359, 959, 233));
        assert_eq!(CODE_THREE_RECORD_AREA_START, 136);
    }

    // ── 第七节「取实现值的模型常量」 ──

    #[test]
    fn allocation_record_tree_fanout_and_leaf_slots_are_the_implementation_values() {
        assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_HEADER_BYTES + ALLOCATION_RECORD_TREE_INTERNAL_FANOUT * ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES + 25, 16384);
        assert_eq!(ALLOCATION_RECORD_TREE_LEAF_SLOTS, 812);
        assert_eq!(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, 169);
    }

    #[test]
    fn multi_level_tree_capacities_are_the_implementation_values() {
        assert_eq!((ACCOUNTING_TREE.leaf_entries, ACCOUNTING_TREE.internal_entries), (477, 150));
        assert_eq!((DEADLIST_TREE.leaf_entries, DEADLIST_TREE.internal_entries), (376, 125));
        assert_eq!((CENTRAL_MAPPING_TREE.leaf_entries, CENTRAL_MAPPING_TREE.internal_entries), (294, 143));
        assert_eq!((CENTRAL_MAPPING_TREE.key_bytes, ACCOUNTING_TREE.key_bytes, DEADLIST_TREE.key_bytes), (27, 22, 43));
        assert_eq!((CENTRAL_MAPPING_TREE.key_fields, ACCOUNTING_TREE.key_fields, DEADLIST_TREE.key_fields), (5, 4, 7));
        assert_eq!(MAPPING_ENTRY_BYTES, 55);
    }

    #[test]
    fn accounting_entries_are_pool_rows_device_rows_and_the_step_row() {
        assert_eq!(3 + 2 * 6 + 1, ACCOUNTING_ENTRIES);
        assert_eq!(ACCOUNTING_ENTRIES, 16);
    }

    #[test]
    fn data_pointer_is_header_two_positions_and_write_order() {
        assert_eq!(50 + 14 * 2 + 10, DATA_POINTER_BYTES);
        assert_eq!(DATA_POINTER_HEADER_BYTES + POSITION_ENTRY_BYTES * 2 + DATA_POINTER_WRITE_ORDER_BYTES, 88);
        assert_eq!(DATA_POINTER_MEMORY_BYTES, 88);
    }

    #[test]
    fn carried_data_unit_is_unit_bytes_and_a_vector_header() {
        assert_eq!(32768 + 24, CARRIED_DATA_UNIT_BYTES);
    }

    #[test]
    fn memory_widths_are_the_registered_values() {
        assert_eq!(
            [TOMBSTONE_RECORD_MEMORY_BYTES, LINEAGE_RECORD_MEMORY_BYTES, RETIREMENT_RECORD_MEMORY_BYTES, INODE_RECORD_MEMORY_BYTES, ALLOCATION_RECORD_MEMORY_BYTES],
            [40, 24, 40, 64, 40]
        );
        assert_eq!([CONTAINER_SHELL_BYTES, UNIT_SHELL_BYTES, SHAPE_NODE_BYTES, SHAPE_LEVEL_BYTES], [56, 48, 8, 24]);
        assert_eq!(
            [key_structure_bytes(CENTRAL_MAPPING_TREE), key_structure_bytes(ACCOUNTING_TREE), key_structure_bytes(DEADLIST_TREE)],
            [115, 102, 147]
        );
        assert_eq!(
            [MAPPED_UNIT_ENTRY_BYTES, MAPPED_UNIT_WITH_LOCATIONS_BYTES, MAPPING_ENTRY_TABLE_BYTES, MAPPING_KEY_BYTES_ENTRY_BYTES, MAPPING_LEAF_ENTRY_BYTES],
            [67, 95, 194, 51, 79]
        );
        assert_eq!([MULTI_LEVEL_SHAPE_NODE_BYTES, ALLOCATION_TREE_NODE_IDENTITY_BYTES, DELETED_INODE_NUMBER_BYTES], [32, 16, 32]);
        assert_eq!(REFERENCE_COUNTED_POINTER_BYTES, 8);
        assert_eq!((CONTAINER_UNIT_MEMORY_BYTES, NODE_UNIT_MEMORY_BYTES), (32816, 16432));
        assert_eq!(mapping_unit_recompute_bytes(), 407);
    }

    #[test]
    fn the_artifact_constant_checks_all_hold() {
        let (_, holds) = constant_check_lines();
        assert_eq!(holds, [true, true, true]);
    }

    // ── 第七节「独立算出」：容器树形状 ──

    #[test]
    fn tombstone_tree_shape_at_a_million_records() {
        let tree = &main_with(|_| {}).tombstones;
        assert_eq!((tree.containers, tree.levels.clone(), tree.nodes, tree.fill), (1227, vec![10, 1], 11, 810));
    }

    #[test]
    fn tombstone_tree_shape_at_ten_and_a_hundred_million_records() {
        let ten_million = main_with(|knobs| knobs.tombstone_records = 10_000_000);
        assert_eq!((ten_million.tombstones.containers, ten_million.tombstones.levels.clone()), (12270, vec![91, 1]));
        let hundred_million = main_with(|knobs| knobs.tombstone_records = 100_000_000);
        assert_eq!((hundred_million.tombstones.containers, hundred_million.tombstones.levels.clone()), (122700, vec![909, 7, 1]));
    }

    #[test]
    fn tombstone_tree_grows_a_level_when_the_container_count_passes_the_fanout() {
        let tree = &main_with(|knobs| knobs.tombstone_records = 110_025).tombstones;
        assert_eq!((tree.containers, tree.levels.clone()), (135, vec![1]));
        assert_eq!((tree.containers_after, tree.levels_after.clone()), (136, vec![2, 1]));
    }

    // ── 第七节「独立算出」：G0 两棵树与甲 ──

    #[test]
    fn main_geometry_mapping_and_allocation_record_trees() {
        let geometry = main_with(|_| {});
        assert_eq!((geometry.mapping_entries, geometry.mapping_levels.clone()), (1272, vec![5, 1]));
        assert_eq!((geometry.slots_per_device, geometry.allocation_levels.clone()), (2501, vec![8, 1]));
        assert_eq!(geometry.mapping_changed_entries, 56);
        assert_eq!((geometry.mapping_rewritten_nodes, geometry.mapping_carried_nodes), (6, 0));
        assert_eq!((geometry.allocation_rewritten_nodes, geometry.allocation_carried_nodes), (9, 0));
        assert_eq!((geometry.units, geometry.rewritten_units, geometry.new_units), (1280, 35, 35));
    }

    #[test]
    fn main_geometry_jia_after_patch_copies_before_admission() {
        let geometry = main_with(|_| {});
        let rounds = jia_after_patch_copied_before_admission(&geometry);
        assert_eq!((rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds), (80346376, 585019, 288));
        assert_eq!(jia_before(&geometry), 404072244);
    }

    #[test]
    fn main_geometry_jia_after_patch_copies_after_admission() {
        assert_eq!(jia_after(&main_with(|_| {})), 81200231);
    }

    #[test]
    fn main_geometry_jia_today_copies_1808_more_before_admission() {
        let geometry = main_with(|_| {});
        assert_eq!(jia_today_copied_before_admission(&geometry), 404074052);
        assert_eq!(jia_before(&geometry) + 1808, jia_today_copied_before_admission(&geometry));
    }

    #[test]
    fn jia_today_is_the_same_under_the_second_and_third_registration_formulas() {
        for (_, knobs) in sampling_points() {
            let geometry = Geometry::new(knobs);
            let added = items_added_by_the_third_registration(&geometry);
            let settle_rounds = knobs.settle_rounds;
            let second_registration_jia = jia_before(&geometry) - added.tombstone_shape_when_not_appending - added.lineage_shape_when_not_appending;
            let tombstone_and_retirement: u64 =
                geometry.tombstone_and_retirement_trees().iter().map(|(tree, _)| (2 + tree.new_containers) * tree.shape_bytes()).sum();
            let second_registration_today = second_registration_jia
                + (settle_rounds + 1) * tombstone_and_retirement
                + settle_rounds * (2 + geometry.lineage.new_containers) * geometry.lineage.shape_bytes();
            assert_eq!(second_registration_today, jia_today_copied_before_admission(&geometry), "{knobs:?}");
        }
    }

    #[test]
    fn main_geometry_installed_bytes_version_sizes_and_recomputation() {
        let geometry = main_with(|_| {});
        assert_eq!(installed_bytes(&geometry), (14232, 610728));
        assert_eq!((version_bytes(&geometry), version_bytes_after(&geometry)), (81793847, 81793919));
        assert_eq!(recomputed_in_full(&geometry), (1814496, 925712));
    }

    #[test]
    fn after_admission_parts_at_the_main_geometry() {
        let parts = jia_after_patch_copied_after_admission(&main_with(|_| {}));
        assert_eq!(parts.inode_leaves_again, 64280);
        assert_eq!(parts.mapping_carried_nodes_each_copy, 0);
        assert_eq!(parts.multi_level_shapes, 1664 + 147087);
    }

    #[test]
    fn after_admission_parts_at_the_focused_and_hundred_million_points() {
        let focused = jia_after_patch_copied_after_admission(&main_with(|knobs| knobs.touched_leaves = TouchedLeaves::Focused));
        assert_eq!(focused.mapping_carried_nodes_each_copy, 65728);
        let large = main_with(|knobs| knobs.tombstone_records = 100_000_000);
        let parts = jia_after_patch_copied_after_admission(&large);
        assert_eq!((parts.inode_leaves_again, parts.mapping_carried_nodes_each_copy, parts.multi_level_shapes), (64280, 3368560, 14357051));
        assert_eq!(jia_after_patch_copied_before_admission(&large).outside_the_rounds, 19616);
        assert_eq!((large.mapping_levels_after.clone(), large.mapping_carried_nodes), (vec![423, 3, 1], 205));
        assert_eq!((large.allocation_levels_after.clone(), large.allocation_carried_nodes), (vec![608, 4, 1], 524));
    }

    #[test]
    fn after_admission_parts_at_the_small_geometry() {
        let small = Geometry::new(Knobs::small_geometry());
        let parts = jia_after_patch_copied_after_admission(&small);
        assert_eq!((parts.inode_leaves_again, parts.mapping_carried_nodes_each_copy, parts.multi_level_shapes), (120, 0, 4111));
        assert_eq!(jia_after_patch_copied_before_admission(&small).outside_the_rounds, 96);
    }

    #[test]
    fn main_geometry_yi2_bing_and_ding() {
        let geometry = main_with(|_| {});
        assert_eq!(yi2_copied(&geometry), (2014312, 421160));
        assert_eq!(recomputed_residual(&geometry), (64160, 132912));
        assert_eq!(arm_bytes(&geometry, Arm::Bing), ArmBytes { copied_before: 0, copied_after: 4648, recomputed_before: 64160, recomputed_after: 132912 });
        let compaction = arm_bytes(&geometry, Arm::DingWholeNewBaseCompaction);
        assert_eq!((compaction.copied_after, compaction.recomputed_after), (81793919, 925712));
        assert_eq!(arm_bytes(&geometry, Arm::DingWithoutCompaction).copied_after, 0);
    }

    #[test]
    fn small_geometry_every_arm() {
        let small = Geometry::new(Knobs::small_geometry());
        assert_eq!((jia_before(&small), jia_after(&small)), (14398, 39575));
        assert_eq!(yi2_copied(&small), (19166, 83239));
        assert_eq!(bing_copied_after_admission(&small), 3080);
        assert_eq!(installed_bytes(&small), (3119, 412584));
        assert_eq!(recomputed_in_full(&small), (8223, 16094));
        assert_eq!(recomputed_residual(&small), (8175, 15998));
    }

    #[test]
    fn jia_before_admission_along_the_tombstone_records_sweep() {
        let expected = [(0, 1107132), (10, 1114972), (1000, 1513360), (10_000, 5139628), (100_000, 41402308), (10_000_000, 4030722004), (100_000_000, 40297407128)];
        for (tombstone_records, copied) in expected {
            assert_eq!(jia_before(&main_with(|knobs| knobs.tombstone_records = tombstone_records)), copied, "u = {tombstone_records}");
        }
        assert_eq!(jia_after(&main_with(|knobs| knobs.tombstone_records = 10_000_000)), 808615927);
    }

    #[test]
    fn jia_before_admission_at_one_and_sixty_four_settle_rounds() {
        assert_eq!(jia_before(&main_with(|knobs| knobs.settle_rounds = 1)), 161278059);
        assert_eq!(jia_before(&main_with(|knobs| knobs.settle_rounds = 64)), 5259955944);
    }

    #[test]
    fn publish_that_appends_no_tombstone() {
        let geometry = main_with(|knobs| knobs.tombstones_added = 0);
        let rounds = jia_after_patch_copied_before_admission(&geometry);
        assert_eq!((rounds.ordinary_allocation_pass, rounds.settle_round), (40277664, 585220));
        assert_eq!((jia_before(&geometry), jia_after(&geometry)), (203729488, 81266111));
    }

    #[test]
    fn publish_that_opens_a_new_tombstone_container() {
        let geometry = main_with(|knobs| knobs.tombstone_records = 1_000_005);
        assert_eq!((geometry.tombstones.open_container_rewritten, geometry.tombstones.new_containers), (0, 1));
        assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404074512, 81233498));
    }

    #[test]
    fn publish_that_appends_815000_tombstones() {
        let geometry = main_with(|knobs| knobs.tombstones_added = 815_000);
        assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404071172, 113972417));
        assert_eq!(installed_bytes(&geometry), (163286052, 66338416));
    }

    #[test]
    fn half_full_mapping_leaves() {
        let geometry = main_with(|knobs| knobs.mapping_leaves_half_full = true);
        assert_eq!(geometry.mapping_levels.clone(), vec![9, 1]);
        assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404076644, 81201171));
    }

    #[test]
    fn focused_touched_leaves() {
        let geometry = main_with(|knobs| knobs.touched_leaves = TouchedLeaves::Focused);
        assert_eq!((geometry.mapping_rewritten_nodes, geometry.mapping_carried_nodes), (2, 4));
        assert_eq!((geometry.allocation_rewritten_nodes, geometry.allocation_carried_nodes, geometry.rewritten_units), (3, 6, 25));
        assert_eq!((jia_before(&geometry), jia_after(&geometry)), (404073852, 81431079));
        assert_eq!(recomputed_in_full(&geometry).1, 848450);
        assert_eq!(recomputed_residual(&geometry), (58640, 50766));
    }

    #[test]
    fn the_two_equal_height_points_give_yi2_bing_and_the_residual_the_same_bytes() {
        let lower = main_with(|knobs| (knobs.tombstone_records, knobs.touched_leaves) = (5_216_400, TouchedLeaves::Focused));
        let upper = main_with(|knobs| (knobs.tombstone_records, knobs.touched_leaves) = (13_040_400, TouchedLeaves::Focused));
        assert_eq!((lower.tombstones.containers, upper.tombstones.containers), (6401, 16001));
        for geometry in [&lower, &upper] {
            assert_eq!(yi2_copied(geometry), (4267192, 703976));
            assert_eq!(bing_copied_after_admission(geometry), 3368);
            assert_eq!(recomputed_residual(geometry), (58640, 50766));
            assert_eq!((geometry.rewritten_units, geometry.mapping_changed_entries), (25, 44));
        }
        assert_eq!((jia_before(&lower), jia_before(&upper)), (2103124788, 5255907796));
    }

    #[test]
    fn positive_control_of_jia_compares_a_million_with_a_hundred_thousand_records() {
        let lower = main_with(|knobs| knobs.tombstone_records = 100_000);
        assert_eq!((jia_before(&lower), jia_after(&lower)), (41402308, 8452936));
        let (lines, fields) = positive_control_lines(&sampling_points());
        assert!(lines.iter().any(|line| line.starts_with("name=positive_control arm=jia_after_patch_copied_before tombstone_records_1e5=41402308 tombstone_records_1e6=404072244 ratio=9.7597")));
        let holding: Vec<&str> = fields.iter().filter(|(_, holds)| holds == "true").map(|(name, _)| name.as_str()).collect();
        assert_eq!(
            holding,
            [
                "positive_control_jia_after_patch_copied_before",
                "positive_control_yi2_persistent_sharing_turned_off",
                "positive_control_yi1_reference_counted_sharing_turned_off",
                "positive_control_bing_cloned_before_handing_in",
                "positive_control_ding_period_1_whole_new_base",
                "recompute_positive_control_yi2_persistent",
                "recompute_positive_control_bing_by_value",
                "recompute_positive_control_ding_without_compaction",
            ]
        );
    }

    #[test]
    fn ding_history_over_256_publishes() {
        let geometries = ding_history_geometries();
        let mut jia = HistoryTotals::default();
        for geometry in &geometries {
            jia.add_publish(geometry, ArmBytes::default());
        }
        assert_eq!((jia.jia.copied_before, jia.jia.copied_after), (103456147732, 20796884553));
        let expected = [
            (1, 20948865498, (0, 0, 0)),
            (2, 10474451876, (1, 128, 0)),
            (3, 6955701571, (2, 171, 1)),
            (4, 5237228498, (3, 192, 0)),
            (8, 2618633376, (7, 224, 0)),
            (64, 327333652, (63, 252, 0)),
            (1024, 0, (256, 256, 256)),
        ];
        for (period, total, layers) in expected {
            let history = ding_history(&geometries, period, Compaction::WholeNewBase);
            assert_eq!(history.compaction_copied_total, total, "p = {period}");
            assert_eq!((history.layer_peak, history.layer_positive_publishes, history.layer_final), layers, "p = {period}");
            assert_eq!(ding_history(&geometries, period, Compaction::FoldInPlace).compaction_copied_total, 0);
        }
    }

    // ── 第三份补进的四项 ──

    #[test]
    fn items_added_by_the_third_registration_at_the_registered_points() {
        let pairs = |geometry: &Geometry| {
            let added = items_added_by_the_third_registration(geometry);
            (added.tombstone_shape_when_not_appending, added.lineage_shape_when_not_appending)
        };
        assert_eq!(pairs(&main_with(|_| {})), (0, 128));
        assert_eq!(pairs(&main_with(|knobs| knobs.tombstones_added = 0)), (680, 128));
        assert_eq!(pairs(&main_with(|knobs| knobs.settle_rounds = 64)), (0, 2048));
        assert_eq!(pairs(&main_with(|knobs| knobs.lineage_records_added = 1)), (0, 0));
        assert_eq!(pairs(&Geometry::new(Knobs::small_geometry())), (0, 0));
        assert_eq!(pairs(&main_with(|knobs| (knobs.tombstone_records, knobs.tombstones_added) = (100_000_000, 0))), (37040, 128));
    }

    #[test]
    fn carried_file_data_units_and_pointers_at_32768_data_units() {
        let geometry = main_with(|knobs| knobs.file_data_units = 32768);
        let added = items_added_by_the_third_registration(&geometry);
        assert_eq!((added.carried_file_data_units, added.carried_file_data_pointers), (1074528256, 2883584));
        assert_eq!(added.carried_file_data_units + added.carried_file_data_pointers, 1077411840);
        let parts = jia_after_patch_copied_after_admission(&geometry);
        assert_eq!((parts.carried_file_data_units, parts.carried_file_data_pointers), (1074528256, 2883584));
        let trees = geometry.trees_with_record_widths().map(|(tree, width)| (tree.records, width, tree.containers, tree.nodes, tree.height));
        let (without_pointers, pointers) =
            version_size_parts(trees, &geometry, geometry.mapping_entries, geometry.mapping_nodes, geometry.allocation_nodes, geometry.allocation_records);
        assert_eq!(without_pointers + 2883584, version_bytes(&geometry));
        assert_eq!(pointers, 2883584);
        assert_eq!((jia_before(&geometry), jia_after(&geometry), version_bytes(&geometry)), (464373652, 2244448535, 1173117639));
        let main = items_added_by_the_third_registration(&main_with(|_| {}));
        assert_eq!((main.carried_file_data_units, main.carried_file_data_pointers), (0, 0));
    }

    // ── 10⁶ 次 unlink 累计：每次 unlink 两次发布，与线程数无关 ──

    #[test]
    fn cumulative_unlinks_count_the_intent_and_the_final_batch() {
        let totals = cumulative_unlinks_between(0..3, DING_MAIN_PERIOD);
        assert_eq!(totals.publishes, 6);
        assert_eq!((totals.jia.copied_before, totals.jia.copied_after), (6665160, 2602316));
        let intent_at_zero = main_with(|knobs| knobs.tombstone_records = 0);
        assert_eq!((jia_before(&intent_at_zero), jia_after(&intent_at_zero)), (1107132, 408842));
    }

    #[test]
    fn cumulative_unlinks_compact_ding_every_period_publishes() {
        // 0..64 这 64 次 unlink 共 128 次发布，第 64 次是 u = 32 上的最后一批，第 128 次是 u = 64 上的最后一批。
        let totals = cumulative_unlinks_between(0..64, DING_MAIN_PERIOD);
        let compacted: u64 = [32, 64]
            .iter()
            .map(|&records| version_bytes_after(&main_with(|knobs| (knobs.tombstone_records, knobs.tombstones_added) = (records, 0))))
            .sum();
        assert_eq!(totals.ding.copied_after, compacted);
        assert!(compacted > 0);
    }

    #[test]
    fn cumulative_unlinks_do_not_depend_on_the_thread_count() {
        assert_eq!(cumulative_unlinks(300, 1), cumulative_unlinks(300, 7));
        assert_eq!(cumulative_unlinks(300, 1), cumulative_unlinks_between(0..300, DING_MAIN_PERIOD));
    }

    #[test]
    fn cumulative_ratio_pair_names_the_arm_whose_bytes_it_divides_by() {
        // 0..64 这 64 次 unlink 共 128 次发布，丁在第 64、128 次压实，整份新基准入后的照抄不为 0，与丙的不同。
        assert_eq!(DING_MAIN_PERIOD, 64);
        let lines = cumulative_lines(&cumulative_unlinks_between(0..64, DING_MAIN_PERIOD));
        let field = |line: &str, key: &str| -> String {
            line.split(' ').find_map(|word| word.strip_prefix(&format!("{key}=")).map(str::to_string)).unwrap_or_else(|| panic!("{key} is not in {line}"))
        };
        let arm_line = |name: &str| -> String {
            let found: Vec<&String> = lines.iter().filter(|line| line.starts_with("name=cumulative_arm ") && field(line.as_str(), "arm") == name).collect();
            assert_eq!(found.len(), 1, "one cumulative_arm line named {name}");
            found[0].clone()
        };
        let bytes = |name: &str, key: &str| -> u64 { field(arm_line(name).as_str(), key).parse().unwrap() };
        let ratio_lines: Vec<&String> = lines.iter().filter(|line| line.starts_with("name=cumulative_ratio ")).collect();
        let pairs: Vec<String> = ratio_lines.iter().map(|line| field(line.as_str(), "pair")).collect();
        assert_eq!(
            pairs,
            [
                "jia_after_patch_over_yi2_persistent",
                "jia_after_patch_over_yi2_persistent",
                "jia_after_patch_over_bing_by_value",
                "jia_after_patch_over_bing_by_value",
                "jia_after_patch_over_ding_period_64_whole_new_base",
                "jia_after_patch_over_ding_period_64_whole_new_base",
            ]
        );
        assert!(bytes("ding_period_64_whole_new_base", "copied_after") > 0);
        assert_ne!(bytes("ding_period_64_whole_new_base", "copied_after"), bytes("bing_by_value", "copied_after"));
        for line in ratio_lines {
            let pair = field(line.as_str(), "pair");
            let denominator = pair.strip_prefix("jia_after_patch_over_").unwrap();
            let (copied_key, recomputed_key) = match field(line.as_str(), "segment").as_str() {
                "before_admission" => ("copied_before", "recomputed_before"),
                "after_admission" => ("copied_after", "recomputed_after"),
                other => panic!("segment {other}"),
            };
            let copied = ratio_text(bytes("jia_after_patch", copied_key), bytes(denominator, copied_key));
            let recomputed = ratio_text(bytes("jia_after_patch", recomputed_key), bytes(denominator, recomputed_key));
            assert_eq!(field(line.as_str(), "copied_without_installed"), copied, "{line}");
            assert_eq!(field(line.as_str(), "recomputed"), recomputed, "{line}");
        }
    }

    // ── 取样点覆盖实验会跑到的全部取值；判定那几格 ──

    #[test]
    fn every_sampling_point_yields_its_lines() {
        let points = sampling_points();
        assert_eq!(points.len(), 40);
        for (label, knobs) in &points {
            let lines = point_lines(label, &Geometry::new(*knobs));
            assert_eq!(lines.len(), 1 + Arm::EVERY_ARM.len() + 5 + judged_cells().len(), "{label}");
        }
    }

    #[test]
    fn main_geometry_judged_cells_and_their_ratios() {
        let geometry = main_with(|_| {});
        let yi2_before = JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: false };
        let forms: Vec<String> = yi2_before.forms(&geometry).iter().map(|&(numerator, denominator)| ratio_text(numerator, denominator)).collect();
        assert_eq!(forms, ["200.6006", "199.2003"]);
        let bing_before = JudgedCell { compared_arm: Arm::Bing, ..yi2_before };
        assert_eq!(ratio_text(bing_before.forms(&geometry)[0].0, bing_before.forms(&geometry)[0].1), "zero_denominator");
        for cell in judged_cells() {
            assert_eq!(cell.verdict(&geometry, JUDGED_THRESHOLD), "does_not_lean_keep_jia", "{}", cell.label());
        }
        assert_eq!(judged_cells().len(), 12);
    }

    #[test]
    fn small_geometry_turns_jia_over_yi2_before_admission_below_two() {
        let small = Geometry::new(Knobs::small_geometry());
        let yi2_before = JudgedCell { compared_arm: Arm::Yi2, after_admission: false, recomputed: false };
        assert_eq!(yi2_before.verdict(&small, JUDGED_THRESHOLD), "leans_keep_jia");
        assert_eq!(ratio_text(yi2_before.forms(&small)[0].0, yi2_before.forms(&small)[0].1), "0.7512");
    }

    #[test]
    fn discriminative_power_examples_flip_between_the_main_and_small_geometries() {
        let (lines, every_example_flips) = judgement_lines(&sampling_points());
        assert!(every_example_flips);
        assert_eq!(lines.iter().filter(|line| line.starts_with("name=discriminative_power ")).count(), 3);
    }

    #[test]
    fn verdicts_name_a_side_only_from_the_threshold() {
        assert_eq!(side_of(399, 200, JUDGED_THRESHOLD), Side::BelowThreshold);
        assert_eq!(side_of(400, 200, JUDGED_THRESHOLD), Side::AtOrAboveThreshold);
        assert_eq!(side_of(1, 0, JUDGED_THRESHOLD), Side::AtOrAboveThreshold);
        assert_eq!(verdict_of_sides(&[Side::BelowThreshold, Side::AtOrAboveThreshold]), "unstable");
        assert_eq!(verdict_of_sides(&[Side::BelowThreshold, Side::BelowThreshold]), "leans_keep_jia");
    }

    #[test]
    fn read_hops_at_the_main_geometry() {
        assert_eq!(yi2_extra_hops(&main_with(|_| {})), (5, 3));
    }

    #[test]
    fn resident_peaks_of_jia_and_bing_at_the_main_geometry() {
        let geometry = main_with(|_| {});
        assert_eq!(jia_resident_peak(&geometry), 245898497);
        assert_eq!(bing_resident_peak(&geometry), 82623711);
        assert_eq!(resident_peak_side(82623711, 245898497), "less_than_jia");
    }

    #[test]
    fn ding_resident_peak_without_compaction_is_the_base_every_layer_and_the_residual() {
        // 第四份第五节：丁加残余式；p = 1024 在 256 次里不压实，两种压实方式峰值相同（互比旁边钉绝对值）。
        let geometries = ding_history_geometries();
        let whole = ding_history(&geometries, 1024, Compaction::WholeNewBase);
        assert_eq!((whole.resident_peak, whole.resident_final), (238289246, 238289246));
        let fold = ding_history(&geometries, 1024, Compaction::FoldInPlace);
        assert_eq!((fold.resident_peak, fold.resident_final), (238289246, 238289246));
    }

    #[test]
    fn ding_resident_peak_at_period_sixty_four_for_both_compactions() {
        let geometries = ding_history_geometries();
        let whole = ding_history(&geometries, 64, Compaction::WholeNewBase);
        assert_eq!((whole.resident_peak, whole.resident_final), (203701016, 203701016));
        let fold = ding_history(&geometries, 64, Compaction::FoldInPlace);
        assert_eq!((fold.resident_peak, fold.resident_final), (121863763, 121863763));
    }

    #[test]
    fn ding_resident_peak_at_period_one_for_both_compactions() {
        let geometries = ding_history_geometries();
        let whole = ding_history(&geometries, 1, Compaction::WholeNewBase);
        assert_eq!((whole.resident_peak, whole.resident_final), (165227672, 165227672));
        let fold = ding_history(&geometries, 1, Compaction::FoldInPlace);
        assert_eq!((fold.resident_peak, fold.resident_final), (83390419, 83390419));
    }

    #[test]
    fn jia_growth_order_before_admission() {
        let lines = growth_order_lines(&sampling_points());
        assert!(lines.iter().any(|line| line.contains("quantity=jia_after_patch_copied_before from_tombstone_records=1e5 to_tombstone_records=1e6 lower=41402308 upper=404072244 slope=0.98943")));
        assert!(lines.iter().any(|line| line.contains("from_tombstone_records=1e6 to_tombstone_records=1e7 lower=404072244 upper=4030722004 slope=0.99892")));
    }

    #[test]
    fn ding_smallest_period_with_a_ratio_of_two() {
        let lines = ding_history_lines();
        assert!(lines.iter().any(|line| line.contains("compaction=whole_new_base") && line.ends_with("smallest_period_with_copied_after_ratio_at_least_two=3")));
        assert!(lines.iter().any(|line| line.contains("compaction=fold_in_place") && line.ends_with("smallest_period_with_copied_after_ratio_at_least_two=at_most_1")));
    }

    #[test]
    fn geometry_sensitivity_names_the_point_that_flips_a_judged_cell() {
        let (lines, _) = judgement_lines(&sampling_points());
        let yi2_before = lines
            .iter()
            .find(|line| line.starts_with("name=judgement point=main_geometry cell=jia_after_patch_over_yi2_persistent_copied_before_admission "))
            .expect("甲 ÷ 乙2 准入前照抄那一格有一行");
        assert!(yi2_before.contains("verdict=does_not_lean_keep_jia sensitivity=unstable_flipped_at_small_geometry "), "{yi2_before}");
    }

    // ── 第四份第七节新钉的锚点（数取设计员草稿 calc/anchors_r4.out、ding_peak_r4.out、sequence_r4.out、mutations_r4.out） ──

    fn yi2_sharing_off_parts(geometry: &Geometry) -> (u64, u64, u64, u64) {
        let rounds = yi2_sharing_turned_off_copied_before_admission(geometry);
        (rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds, rounds.total(geometry.knobs.settle_rounds))
    }

    #[test]
    fn yi2_positive_control_copies_every_collection_whole_with_sharing_turned_off() {
        assert_eq!(yi2_sharing_off_parts(&main_with(|_| {})), (40173272, 438408, 144, 202620136));
        assert_eq!(yi2_sharing_off_parts(&main_with(|knobs| knobs.tombstone_records = 100_000)), (4111344, 54568, 48, 20775040));
        assert_eq!(side_of(202620136, 20775040, (9, 1)), Side::AtOrAboveThreshold);
        let (lines, _) = positive_control_lines(&sampling_points());
        assert!(lines.iter().any(|line| line
            == "name=positive_control arm=yi2_persistent_sharing_turned_off tombstone_records_1e5=20775040 tombstone_records_1e6=202620136 ratio=9.7531 threshold=9 holds=true"));
    }

    #[test]
    fn yi2_sharing_turned_off_copies_the_untouched_retirement_tree_and_every_record() {
        // a_X = 1 的点上退役树碰到了，第四份第九节「漏掉没碰到的树」那一条在这里不变；G-small 一片、fill = R。
        assert_eq!(yi2_sharing_off_parts(&main_with(|knobs| knobs.retirements_added = 1)).3, 202620136);
        assert_eq!(yi2_sharing_off_parts(&Geometry::new(Knobs::small_geometry())).3, 10558);
        let retirements = &main_with(|_| {}).retirements;
        assert_eq!(whole_container_tree_bytes(retirements, RETIREMENT_RECORD_MEMORY_BYTES), 40000 + 112 + 32);
    }

    #[test]
    fn yi2_positive_control_reported_only_reading_copies_only_the_touched_collections() {
        let parts = |geometry: &Geometry| {
            let rounds = yi2_touched_collections_whole_copied_before_admission(geometry);
            (rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds, rounds.total(geometry.knobs.settle_rounds))
        };
        assert_eq!(parts(&main_with(|_| {})), (116440, 437936, 144, 2334088));
        assert_eq!(parts(&main_with(|knobs| knobs.tombstone_records = 100_000)), (44912, 54096, 48, 440992));
        let (lines, fields) = positive_control_lines(&sampling_points());
        assert!(lines.iter().any(|line| line.starts_with(
            "name=positive_control_reported_only arm=yi2_persistent_touched_collections_whole tombstone_records_1e5=440992 tombstone_records_1e6=2334088 ratio=5.2928 judged=no"
        )));
        assert!(fields.iter().all(|(name, _)| !name.contains("touched_collections_whole")));
    }

    #[test]
    fn yi1_positive_control_with_sharing_turned_off_is_jia_after_patch() {
        let total = |geometry: &Geometry| yi1_copied_before_admission(geometry, ReferenceSharing::TurnedOff).total(geometry.knobs.settle_rounds);
        let main = main_with(|_| {});
        let lower = main_with(|knobs| knobs.tombstone_records = 100_000);
        assert_eq!((total(&main), total(&lower)), (404072244, 41402308));
        assert_eq!(yi1_copied_after_admission(&main, ReferenceSharing::TurnedOff), 81200231);
        for (label, knobs) in sampling_points() {
            let geometry = Geometry::new(knobs);
            assert_eq!(total(&geometry), jia_before(&geometry), "{label}");
            assert_eq!(yi1_copied_after_admission(&geometry, ReferenceSharing::TurnedOff), jia_after(&geometry), "{label}");
        }
        let (lines, _) = positive_control_lines(&sampling_points());
        assert!(lines.iter().any(|line| line
            == "name=positive_control arm=yi1_reference_counted_sharing_turned_off tombstone_records_1e5=41402308 tombstone_records_1e6=404072244 ratio=9.7597 threshold=9 holds=true"));
    }

    #[test]
    fn yi1_copied_bytes_at_the_registered_points() {
        let yi1 = |geometry: &Geometry| {
            let rounds = yi1_copied_before_admission(geometry, ReferenceSharing::Shared);
            (rounds.total(geometry.knobs.settle_rounds), yi1_copied_after_admission(geometry, ReferenceSharing::Shared))
        };
        let main = main_with(|_| {});
        let rounds = yi1_copied_before_admission(&main, ReferenceSharing::Shared);
        assert_eq!((rounds.ordinary_allocation_pass, rounds.settle_round, rounds.outside_the_rounds), (131464, 584587, 288));
        assert_eq!(yi1(&main), (2995956, 435031));
        let small = Geometry::new(Knobs::small_geometry());
        let small_rounds = yi1_copied_before_admission(&small, ReferenceSharing::Shared);
        assert_eq!((small_rounds.ordinary_allocation_pass, small_rounds.settle_round, small_rounds.outside_the_rounds), (640, 11934, 96));
        assert_eq!(yi1(&small), (13310, 6279));
        assert_eq!(yi1(&main_with(|knobs| knobs.tombstone_records = 100_000)), (807940, 108576));
        assert_eq!(yi1(&main_with(|knobs| knobs.tombstones_added = 0)), (2785680, 435295));
        assert_eq!(yi1(&main_with(|knobs| knobs.lineage_records_added = 1)), (2996988, 434855));
        assert_eq!(yi1(&main_with(|knobs| knobs.file_data_units = 32768)), (63297364, 12919367));
        assert_eq!(yi1(&main_with(|knobs| knobs.touched_leaves = TouchedLeaves::Focused)), (2997564, 435943));
        assert_eq!(yi1(&main_with(|knobs| knobs.tombstone_records = 1_000_005)).0, 2834224);
    }

    #[test]
    fn bing_and_ding_positive_controls_at_the_two_registered_points() {
        let main = main_with(|_| {});
        let lower = main_with(|knobs| knobs.tombstone_records = 100_000);
        assert_eq!(bing_copied_after_admission(&main) + version_bytes(&main), 4648 + 81793847);
        assert_eq!(bing_copied_after_admission(&lower) + version_bytes(&lower), 3096 + 8863272);
        assert_eq!(ding_copied_after_admission(&main, true, Compaction::WholeNewBase), 81793919);
        assert_eq!(ding_copied_after_admission(&lower, true, Compaction::WholeNewBase), 8863344);
        let (lines, _) = positive_control_lines(&sampling_points());
        assert!(lines.iter().any(|line| line
            == "name=positive_control arm=bing_cloned_before_handing_in tombstone_records_1e5=8866368 tombstone_records_1e6=81798495 ratio=9.2257 threshold=9 holds=true"));
        assert!(lines.iter().any(|line| line
            == "name=positive_control arm=ding_period_1_whole_new_base tombstone_records_1e5=8863344 tombstone_records_1e6=81793919 ratio=9.2283 threshold=9 holds=true"));
    }

    #[test]
    fn bing_resident_peak_counts_only_the_replaced_containers_at_815000_tombstones() {
        // 第四份第九节：被换下的旧字节取 k_old，不取 k_old + k_new（a_T = 815000 上 k_new = 1000，G0 上 k_new = 0）。
        let geometry = main_with(|knobs| knobs.tombstones_added = 815_000);
        assert_eq!(geometry.tombstones.new_containers, 1000);
        assert_eq!(bing_resident_peak(&geometry), 149447442);
        // G0 上被换下的是墓碑树开放那一片与两个索引节点：32768 + 2 × 16384 = 65536。
        let main = main_with(|_| {});
        let replaced: Vec<(u64, u64)> = main.container_trees().iter().map(|tree| (tree.open_container_rewritten, tree.rewritten_index_nodes)).collect();
        assert_eq!(replaced, [(1, 2), (0, 0), (0, 0)]);
        assert_eq!(bing_resident_peak(&main), 82623711);
    }

    #[test]
    fn cumulative_unlinks_prefix_of_thirty_two_unlinks_starts_at_zero_records() {
        // 第四份第五节：第 j 次 unlink（j 从 0 起）发布序号 2j + 1、2j + 2；丁在序号 64（j = 31 的最后一批，u = 32）压实一次。
        let totals = cumulative_unlinks_between(0..32, DING_MAIN_PERIOD);
        assert_eq!(totals.publishes, 64);
        assert_eq!((totals.jia.copied_before, totals.jia.copied_after), (71410560, 27796704));
        assert_eq!((totals.yi2.copied_before, totals.yi2.copied_after), (6291984, 5865360));
        assert_eq!(totals.bing.copied_after, 189688);
        assert_eq!(totals.ding.copied_after, 822204);
        assert_eq!(version_bytes_after(&main_with(|knobs| (knobs.tombstone_records, knobs.tombstones_added) = (32, 0))), 822204);
    }
}
