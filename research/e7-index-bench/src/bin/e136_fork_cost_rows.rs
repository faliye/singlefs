//! E136：映射 key 岔路的代价行。
//!
//! D19 已定项 6 三轮对抗第三轮（那一格的 ⑫㈤）判：交用户之前岔路缺四样代价数。跑前登记
//! `research/prompts/e136-preregistration.md`（写于本文件之前，补记也在第一次运行之前）：
//! ① C280 取「只装码 1」那一支的配置（指向码 2 / 码 3 的指针只带出生树 + 出生 txg）；
//! ② 码 2 头的增量回灌进扇出——节点头取 D18 已定项 14 的码 2 三档头 96 / 105 / 114 加该臂的增量；
//! ③ 映射树层数按池规模区间报，叶层字节报比值；④ 搬一个码 2 / 码 3 节点的字节，与逻辑身份 + 写序键的派生树码 2 节点搬迁多查的那一次读。
//! ⚠️ **只报数、不判输赢**：`.claude/rules/fs-design.md` 逐字「「比谁省」不构成任何判据」。
//!
//! 计数模型，不是实现：没有 I/O、没有随机源 ⇒ 跑 N 遍必然逐字节一致，证据强度来自变异测试与钉死绝对值的断言。
//! 三道跨装置闸：`all` 配置 × 节点头 64 逐行复现 E134 的留存产物；三档头的扇出式复现 D18 第 308 行登记的数；
//! 搬迁计数复现 E109 的「搬一个单元」表。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384; // D8 已定项 2
/// E133 / E134 扇出式里的引用树节点头（只用来过跨装置闸，与 D18 的码 2 三档头不是同一个口径）。
const NODE_HEADER_BYTES: u64 = 64;
/// E109 的映射节点头。
const MAP_NODE_HEADER_BYTES: u64 = 76;
/// 树表单元每层净字节（22-单元原子性怎么合成.md 的分母）。
const TREE_TABLE_PAYLOAD_BYTES: u64 = 16284;
/// 指针头部：MAC 16 + nonce 12 + 算法类型 1 + extent 偏移 2（D21 正文）。
const POINTER_HEAD_BYTES_TODAY: u64 = 16 + 12 + 1 + 2;
/// 位置条目：设备 4 + 16 KiB 槽号 6 + 密文校验和 4（D19 已定项 4）。
const LOCATION_ENTRY_BYTES: u64 = 4 + 6 + 4;
/// 第一版每单元两份副本（D2 已定项 9）。
const REPLICA_COUNT: u64 = 2;
const CHILD_POINTER_BYTES_TODAY: u64 = POINTER_HEAD_BYTES_TODAY + LOCATION_ENTRY_BYTES * REPLICA_COUNT;
/// 映射 value：每个副本一份位置条目（D19 已定项 5 硬规则 2）。
const MAP_VALUE_BYTES: u64 = LOCATION_ENTRY_BYTES * REPLICA_COUNT;
/// inode 树内部条目的非指针段：分隔 key 8 + 身份引用 26（D8 已定项 6）。
const INODE_INTERNAL_NON_POINTER_BYTES: u64 = 8 + 26;
/// 记账树内部条目的非指针段：key 22（D5 已定项 5 的 key，内部条目 81 − 59）。
const LEDGER_INTERNAL_NON_POINTER_BYTES: u64 = 22;
/// extent 条目的非指针段：(locality_id, inode, offset) 各 8（D8 已定项 3）。
const EXTENT_NON_POINTER_BYTES: u64 = 8 + 8 + 8;
/// 记账叶条目：key 22 + value 8（D5 已定项 5）。
const LEDGER_LEAF_ENTRY_BYTES: u64 = 22 + 8;
/// 树表条目的非指针段：长度 2 + 种类 2 + flags 2 + 树 ID 8 + previous_snapshot_txg 8 + 诞生 txg 8 + 预留 32（D8 已定项 8）。
const TREE_TABLE_NON_POINTER_BYTES: u64 = 2 + 2 + 2 + 8 + 8 + 8 + 32;
/// 根记录的非指针段（D22 已定项 7）。
const ROOT_RECORD_NON_POINTER_BYTES: u64 = 4 + 16 + 4 + 4 + 8 + 8 + 32;
const ROOT_RECORD_POINTER_COUNT: u64 = 2;
const INODE_RECORD_BYTES: u64 = 140; // D8 已定项 6
const DATA_UNIT_BYTES: u64 = 32768; // D4 已定项 1
const CODE1_HEADER_BYTES_TODAY: u64 = 133; // D18 已定项 14
const CODE3_HEADER_BYTES_TODAY: u64 = 131; // D18 已定项 14
/// D18 已定项 14 的码 2 三档头（树 ID 与层级的宽度随实现定，三档是三种取法，不按树分）。
const CODE2_HEADER_TIERS_TODAY: [u64; 3] = [96, 105, 114];
/// E109 的池规模（数据单元数）。
const POOL_DATA_UNITS: u64 = 483_183_820;
/// 「含索引节点」那一档按每 119 个单元一个索引节点估（同 E133）。
const INDEX_FANOUT_FOR_NODE_COUNT: u64 = 119;
/// E109 发布模型里的根槽宽。
const ROOT_SLOT_BYTES: u64 = 512;
/// 搬迁表里的引用树数（同 E109）。
const REFERENCING_TREE_COUNTS: [u64; 5] = [1, 2, 4, 8, 16];
/// 搬一个码 2 节点时报的节点所在层（叶为 0）。
const RELOCATED_NODE_LEVELS: [u64; 3] = [0, 1, 2];
/// 映射树门槛报到几层。
const HIGHEST_REPORTED_LEVEL_COUNT: u32 = 6;
/// 叶层字节比值保留的小数位（按 10 的幂放大，整数除法）。
const RATIO_SCALE: u128 = 100_000;
/// 「只装码 1」时指向码 2 / 码 3 的指针只带出生树 8 + 出生 txg 8。
const CODE1_ONLY_NODE_POINTER_EXTRA_BYTES: u64 = 8 + 8;

/// 一条候选的格式形态，三条臂的字段逐字取自 E134 的臂表（`e134_map_key_slot_baselines.rs`）。
#[derive(Clone, Copy, Debug)]
struct Arm {
    name: &'static str,
    data_pointer_extra_bytes: u64,
    node_pointer_extra_bytes: u64,
    code1_header_extra_bytes: u64,
    code23_header_extra_bytes: u64,
    code1_key_bytes: u64,
    code23_key_bytes: Option<u64>,
    root_record_extra_bytes: u64,
    slot_extra_over_e114_pack_bytes: u64,
    slot_extra_over_five_tuple_bytes: u64,
}

const ARMS: [Arm; 3] = [
    Arm { name: "k1_seq", data_pointer_extra_bytes: 50, node_pointer_extra_bytes: 16, code1_header_extra_bytes: 0, code23_header_extra_bytes: 0, code1_key_bytes: 51, code23_key_bytes: None, root_record_extra_bytes: 0, slot_extra_over_e114_pack_bytes: 8, slot_extra_over_five_tuple_bytes: 18 },
    Arm { name: "k1_seq_class_reuse", data_pointer_extra_bytes: 50, node_pointer_extra_bytes: 24, code1_header_extra_bytes: 0, code23_header_extra_bytes: 4, code1_key_bytes: 51, code23_key_bytes: Some(25), root_record_extra_bytes: 0, slot_extra_over_e114_pack_bytes: 8, slot_extra_over_five_tuple_bytes: 18 },
    Arm { name: "class_reuse_tagged", data_pointer_extra_bytes: 26, node_pointer_extra_bytes: 24, code1_header_extra_bytes: 0, code23_header_extra_bytes: 4, code1_key_bytes: 27, code23_key_bytes: Some(25), root_record_extra_bytes: 0, slot_extra_over_e114_pack_bytes: 9, slot_extra_over_five_tuple_bytes: 19 },
];

/// 映射装哪几类单元（C280 那个开关）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapScope {
    /// 全部指针进映射（D19 已定项 5 的字面）：E134 的配置。
    AllPointers,
    /// 映射只装码 1：指向码 2 / 码 3 的指针只带出生树 + 出生 txg，码 2 / 码 3 头不再加出生序号。
    Code1Only,
}

const MAP_SCOPES: [MapScope; 2] = [MapScope::AllPointers, MapScope::Code1Only];

fn map_scope_name(map_scope: MapScope) -> &'static str {
    match map_scope {
        MapScope::AllPointers => "all",
        MapScope::Code1Only => "code1_only",
    }
}

/// 一条臂在某个映射范围下的实际形态：`Code1Only` 时指向码 2 / 3 的指针只带 16 字节，码 2 / 3 头与 key 不再需要。
fn arm_under_scope(arm: &Arm, map_scope: MapScope) -> Arm {
    match map_scope {
        MapScope::AllPointers => *arm,
        MapScope::Code1Only => Arm {
            name: arm.name,
            data_pointer_extra_bytes: arm.data_pointer_extra_bytes,
            node_pointer_extra_bytes: CODE1_ONLY_NODE_POINTER_EXTRA_BYTES,
            code1_header_extra_bytes: arm.code1_header_extra_bytes,
            code23_header_extra_bytes: 0,
            code1_key_bytes: arm.code1_key_bytes,
            code23_key_bytes: None,
            root_record_extra_bytes: arm.root_record_extra_bytes,
            slot_extra_over_e114_pack_bytes: arm.slot_extra_over_e114_pack_bytes,
            slot_extra_over_five_tuple_bytes: arm.slot_extra_over_five_tuple_bytes,
        },
    }
}

fn data_child_pointer_bytes(arm: &Arm) -> u64 {
    CHILD_POINTER_BYTES_TODAY + arm.data_pointer_extra_bytes
}
fn node_child_pointer_bytes(arm: &Arm) -> u64 {
    CHILD_POINTER_BYTES_TODAY + arm.node_pointer_extra_bytes
}
fn extent_leaf_entry_bytes(arm: &Arm) -> u64 {
    EXTENT_NON_POINTER_BYTES + data_child_pointer_bytes(arm)
}
fn extent_internal_entry_bytes(arm: &Arm) -> u64 {
    EXTENT_NON_POINTER_BYTES + node_child_pointer_bytes(arm)
}
fn inode_internal_entry_bytes(arm: &Arm) -> u64 {
    INODE_INTERNAL_NON_POINTER_BYTES + node_child_pointer_bytes(arm)
}
fn ledger_internal_entry_bytes(arm: &Arm) -> u64 {
    LEDGER_INTERNAL_NON_POINTER_BYTES + node_child_pointer_bytes(arm)
}
fn tree_table_entry_bytes(arm: &Arm) -> u64 {
    TREE_TABLE_NON_POINTER_BYTES + node_child_pointer_bytes(arm)
}
fn root_record_bytes(arm: &Arm) -> u64 {
    ROOT_RECORD_NON_POINTER_BYTES + node_child_pointer_bytes(arm) * ROOT_RECORD_POINTER_COUNT + arm.root_record_extra_bytes
}
fn map_entry_bytes(key_bytes: u64) -> u64 {
    key_bytes + MAP_VALUE_BYTES
}

/// 节点头 `node_header_bytes` 的节点装几条宽 `entry_bytes` 的条目。
fn fanout_with_header(node_header_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - node_header_bytes) / entry_bytes
}
/// E133 / E134 口径：引用树节点头 64。
fn node_fanout(entry_bytes: u64) -> u64 {
    fanout_with_header(NODE_HEADER_BYTES, entry_bytes)
}
fn tree_table_entries_per_level(entry_bytes: u64) -> u64 {
    TREE_TABLE_PAYLOAD_BYTES / entry_bytes
}
/// 映射树叶：key + value。
fn map_leaf_fanout_with_header(node_header_bytes: u64, key_bytes: u64) -> u64 {
    fanout_with_header(node_header_bytes, map_entry_bytes(key_bytes))
}
/// 映射树内部节点：key + 一条指向映射树自己节点（码 2）的子指针。
fn map_internal_fanout_with_header(node_header_bytes: u64, key_bytes: u64, child_pointer: u64) -> u64 {
    fanout_with_header(node_header_bytes, key_bytes + child_pointer)
}
fn map_leaf_fanout(key_bytes: u64) -> u64 {
    map_leaf_fanout_with_header(MAP_NODE_HEADER_BYTES, key_bytes)
}
fn map_internal_fanout(key_bytes: u64, child_pointer: u64) -> u64 {
    map_internal_fanout_with_header(MAP_NODE_HEADER_BYTES, key_bytes, child_pointer)
}
/// inode 树的叶是码 3 容器：头之后按 140 字节一条记录。
fn inode_leaf_capacity(arm: &Arm) -> u64 {
    (DATA_UNIT_BYTES - (CODE3_HEADER_BYTES_TODAY + arm.code23_header_extra_bytes)) / INODE_RECORD_BYTES
}

fn capacity_at_levels(leaf_capacity: u64, internal_fanout: u64, level_count: u32) -> u64 {
    let mut capacity_in_entries = leaf_capacity;
    for _ in 1..level_count {
        capacity_in_entries = capacity_in_entries.saturating_mul(internal_fanout);
    }
    capacity_in_entries
}
/// 最少多少条条目时这棵树要 `level_count` 层（`level_count` ≥ 2）。
fn first_entry_count_needing_levels(leaf_capacity: u64, internal_fanout: u64, level_count: u32) -> u64 {
    capacity_at_levels(leaf_capacity, internal_fanout, level_count - 1).saturating_add(1)
}
fn level_count_for(entry_count: u64, leaf_capacity: u64, internal_fanout: u64) -> u32 {
    let mut node_count = entry_count.div_ceil(leaf_capacity).max(1);
    let mut level_count = 1u32;
    while node_count > 1 {
        node_count = node_count.div_ceil(internal_fanout);
        level_count += 1;
    }
    level_count
}

fn trees(arm: &Arm) -> [(&'static str, u64, u64); 4] {
    [
        ("extent", node_fanout(extent_leaf_entry_bytes(arm)), node_fanout(extent_internal_entry_bytes(arm))),
        ("inode", inode_leaf_capacity(arm), node_fanout(inode_internal_entry_bytes(arm))),
        ("ledger", node_fanout(LEDGER_LEAF_ENTRY_BYTES), node_fanout(ledger_internal_entry_bytes(arm))),
        ("map", map_leaf_fanout(arm.code1_key_bytes), map_internal_fanout(arm.code1_key_bytes, node_child_pointer_bytes(arm))),
    ]
}

fn optional_width(width: Option<u64>) -> String {
    match width {
        Some(bytes) => bytes.to_string(),
        None => "n/a".to_string(),
    }
}

/// 与 E134 同名函数逐字相同：`all` 配置下它的输出要逐行等于 E134 的留存产物（跨装置闸 1）。
fn arm_result_lines(arm: &Arm) -> Vec<String> {
    let mut lines = vec![
        format!(
            "name=widths arm={} data_pointer={} node_pointer={} extent_leaf_entry={} extent_internal_entry={} inode_internal={} ledger_internal={} tree_table_entry={} root_record={} map_entry_code1={} map_entry_code23={}",
            arm.name,
            data_child_pointer_bytes(arm),
            node_child_pointer_bytes(arm),
            extent_leaf_entry_bytes(arm),
            extent_internal_entry_bytes(arm),
            inode_internal_entry_bytes(arm),
            ledger_internal_entry_bytes(arm),
            tree_table_entry_bytes(arm),
            root_record_bytes(arm),
            map_entry_bytes(arm.code1_key_bytes),
            optional_width(arm.code23_key_bytes.map(map_entry_bytes))
        ),
        format!(
            "name=headers arm={} code1={} code3={} code2={}/{}/{} slot_extra_over_e114_pack={} slot_extra_over_five_tuple={} livelist_identity_code1={}",
            arm.name,
            CODE1_HEADER_BYTES_TODAY + arm.code1_header_extra_bytes,
            CODE3_HEADER_BYTES_TODAY + arm.code23_header_extra_bytes,
            CODE2_HEADER_TIERS_TODAY[0] + arm.code23_header_extra_bytes,
            CODE2_HEADER_TIERS_TODAY[1] + arm.code23_header_extra_bytes,
            CODE2_HEADER_TIERS_TODAY[2] + arm.code23_header_extra_bytes,
            arm.slot_extra_over_e114_pack_bytes,
            arm.slot_extra_over_five_tuple_bytes,
            arm.code1_key_bytes
        ),
        format!(
            "name=fanouts arm={} extent_leaf={} extent_internal={} inode_internal={} ledger_internal={} tree_table_per_level={} map_leaf={} map_internal={} inode_leaf={}",
            arm.name,
            node_fanout(extent_leaf_entry_bytes(arm)),
            node_fanout(extent_internal_entry_bytes(arm)),
            node_fanout(inode_internal_entry_bytes(arm)),
            node_fanout(ledger_internal_entry_bytes(arm)),
            tree_table_entries_per_level(tree_table_entry_bytes(arm)),
            map_leaf_fanout(arm.code1_key_bytes),
            map_internal_fanout(arm.code1_key_bytes, node_child_pointer_bytes(arm)),
            inode_leaf_capacity(arm)
        ),
    ];
    for (tree_name, leaf_capacity, internal_fanout) in trees(arm) {
        let thresholds: Vec<String> = (2u32..=5)
            .map(|level_count| format!("l{level_count}={}", first_entry_count_needing_levels(leaf_capacity, internal_fanout, level_count)))
            .collect();
        lines.push(format!("name=thresholds arm={} tree={tree_name} {}", arm.name, thresholds.join(" ")));
    }
    let map_leaf = map_leaf_fanout(arm.code1_key_bytes);
    let map_internal = map_internal_fanout(arm.code1_key_bytes, node_child_pointer_bytes(arm));
    let with_index_nodes = POOL_DATA_UNITS + POOL_DATA_UNITS.div_ceil(INDEX_FANOUT_FOR_NODE_COUNT);
    lines.push(format!(
        "name=map_at_pool arm={} data_units_levels={} data_units_leaf_bytes={} with_index_levels={} with_index_leaf_bytes={}",
        arm.name,
        level_count_for(POOL_DATA_UNITS, map_leaf, map_internal),
        POOL_DATA_UNITS.div_ceil(map_leaf) * NODE_BYTES,
        level_count_for(with_index_nodes, map_leaf, map_internal),
        with_index_nodes.div_ceil(map_leaf) * NODE_BYTES
    ));
    lines
}

/// `Code1Only` 配置下的宽度、头、64 口径扇出、映射树与 extent 树门槛、映射树在池规模上的层数。
fn code1_only_result_lines(arm: &Arm) -> Vec<String> {
    let scoped = arm_under_scope(arm, MapScope::Code1Only);
    let (_, extent_leaf, extent_internal) = trees(&scoped)[0];
    let (_, map_leaf, map_internal) = trees(&scoped)[3];
    let threshold_text = |leaf_capacity: u64, internal_fanout: u64| -> String {
        (2u32..=HIGHEST_REPORTED_LEVEL_COUNT)
            .map(|level_count| format!("l{level_count}={}", first_entry_count_needing_levels(leaf_capacity, internal_fanout, level_count)))
            .collect::<Vec<String>>()
            .join(" ")
    };
    vec![
        format!(
            "name=code1_only_widths arm={} data_pointer={} node_pointer={} extent_leaf_entry={} extent_internal_entry={} inode_internal={} ledger_internal={} tree_table_entry={} root_record={} map_entry_code1={}",
            scoped.name,
            data_child_pointer_bytes(&scoped),
            node_child_pointer_bytes(&scoped),
            extent_leaf_entry_bytes(&scoped),
            extent_internal_entry_bytes(&scoped),
            inode_internal_entry_bytes(&scoped),
            ledger_internal_entry_bytes(&scoped),
            tree_table_entry_bytes(&scoped),
            root_record_bytes(&scoped),
            map_entry_bytes(scoped.code1_key_bytes)
        ),
        format!(
            "name=code1_only_headers arm={} code1={} code3={} code2={}/{}/{}",
            scoped.name,
            CODE1_HEADER_BYTES_TODAY + scoped.code1_header_extra_bytes,
            CODE3_HEADER_BYTES_TODAY + scoped.code23_header_extra_bytes,
            CODE2_HEADER_TIERS_TODAY[0] + scoped.code23_header_extra_bytes,
            CODE2_HEADER_TIERS_TODAY[1] + scoped.code23_header_extra_bytes,
            CODE2_HEADER_TIERS_TODAY[2] + scoped.code23_header_extra_bytes
        ),
        format!(
            "name=code1_only_fanouts arm={} extent_leaf={} extent_internal={} inode_internal={} ledger_internal={} tree_table_per_level={} map_leaf={} map_internal={} inode_leaf={}",
            scoped.name,
            extent_leaf,
            extent_internal,
            node_fanout(inode_internal_entry_bytes(&scoped)),
            node_fanout(ledger_internal_entry_bytes(&scoped)),
            tree_table_entries_per_level(tree_table_entry_bytes(&scoped)),
            map_leaf,
            map_internal,
            inode_leaf_capacity(&scoped)
        ),
        format!("name=code1_only_thresholds arm={} tree=extent {}", scoped.name, threshold_text(extent_leaf, extent_internal)),
        format!("name=code1_only_thresholds arm={} tree=map {}", scoped.name, threshold_text(map_leaf, map_internal)),
        format!(
            "name=code1_only_map_at_pool arm={} data_units_levels={} data_units_leaf_bytes={}",
            scoped.name,
            level_count_for(POOL_DATA_UNITS, map_leaf, map_internal),
            POOL_DATA_UNITS.div_ceil(map_leaf) * NODE_BYTES
        ),
    ]
}

/// 一条臂在某个映射范围下，码 2 节点头取三档之一时的实际头宽：档 + 该配置下的码 2 头增量。
fn code2_node_header_bytes(arm: &Arm, map_scope: MapScope, tier_bytes: u64) -> u64 {
    tier_bytes + arm_under_scope(arm, map_scope).code23_header_extra_bytes
}

/// 用码 2 三档头（加增量）当节点头时的扇出：(extent 叶, extent 内部, inode 内部, 记账内部, 映射叶, 映射内部)。
fn tier_fanouts(arm: &Arm, map_scope: MapScope, tier_bytes: u64) -> [u64; 6] {
    let scoped = arm_under_scope(arm, map_scope);
    let node_header_bytes = code2_node_header_bytes(arm, map_scope, tier_bytes);
    [
        fanout_with_header(node_header_bytes, extent_leaf_entry_bytes(&scoped)),
        fanout_with_header(node_header_bytes, extent_internal_entry_bytes(&scoped)),
        fanout_with_header(node_header_bytes, inode_internal_entry_bytes(&scoped)),
        fanout_with_header(node_header_bytes, ledger_internal_entry_bytes(&scoped)),
        map_leaf_fanout_with_header(node_header_bytes, scoped.code1_key_bytes),
        map_internal_fanout_with_header(node_header_bytes, scoped.code1_key_bytes, node_child_pointer_bytes(&scoped)),
    ]
}

/// 同一组条目宽，节点头取 `node_header_bytes` 与再加 `header_increment_bytes` 时各装几条，以及掉了几格。
fn fanout_drop_count(node_header_bytes: u64, header_increment_bytes: u64, entry_widths: &[u64]) -> (Vec<u64>, Vec<u64>, usize) {
    let without_increment: Vec<u64> = entry_widths.iter().map(|entry_bytes| fanout_with_header(node_header_bytes, *entry_bytes)).collect();
    let with_increment: Vec<u64> = entry_widths.iter().map(|entry_bytes| fanout_with_header(node_header_bytes + header_increment_bytes, *entry_bytes)).collect();
    let dropped_cell_count = without_increment.iter().zip(with_increment.iter()).filter(|(before, after)| before != after).count();
    (without_increment, with_increment, dropped_cell_count)
}

/// 一条臂在全部指针进映射时码 2 节点里的六种条目宽：(extent 叶, extent 内部, inode 内部, 记账内部, 映射叶, 映射内部)。
fn code2_node_entry_widths(arm: &Arm) -> [u64; 6] {
    [
        extent_leaf_entry_bytes(arm),
        extent_internal_entry_bytes(arm),
        inode_internal_entry_bytes(arm),
        ledger_internal_entry_bytes(arm),
        map_entry_bytes(arm.code1_key_bytes),
        arm.code1_key_bytes + node_child_pointer_bytes(arm),
    ]
}

fn slash_joined(values: &[u64]) -> String {
    values.iter().map(|value| value.to_string()).collect::<Vec<String>>().join("/")
}

/// 映射树扇出的两套口径。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FanoutBasis {
    /// E133 / E134 的映射节点头 76。
    MapHeader76,
    /// D18 的码 2 三档头之一（加该配置下的增量）。
    Code2Tier(u64),
}

fn fanout_basis_name(fanout_basis: FanoutBasis) -> String {
    match fanout_basis {
        FanoutBasis::MapHeader76 => "map_header_76".to_string(),
        FanoutBasis::Code2Tier(tier_bytes) => format!("code2_tier_{tier_bytes}"),
    }
}

/// (映射叶扇出, 映射内部扇出)。
fn map_fanouts_under(arm: &Arm, map_scope: MapScope, fanout_basis: FanoutBasis) -> (u64, u64) {
    let scoped = arm_under_scope(arm, map_scope);
    match fanout_basis {
        FanoutBasis::MapHeader76 => (map_leaf_fanout(scoped.code1_key_bytes), map_internal_fanout(scoped.code1_key_bytes, node_child_pointer_bytes(&scoped))),
        FanoutBasis::Code2Tier(tier_bytes) => {
            let fanouts = tier_fanouts(arm, map_scope, tier_bytes);
            (fanouts[4], fanouts[5])
        }
    }
}

/// 映射树 l2…l6 的门槛（只数数据单元）。
fn map_thresholds_under(arm: &Arm, map_scope: MapScope, fanout_basis: FanoutBasis) -> Vec<u64> {
    let (map_leaf, map_internal) = map_fanouts_under(arm, map_scope, fanout_basis);
    (2u32..=HIGHEST_REPORTED_LEVEL_COUNT)
        .map(|level_count| first_entry_count_needing_levels(map_leaf, map_internal, level_count))
        .collect()
}

/// 两条臂映射树层数不同的规模区间：每个层数一段 [下沿, 上沿)，门槛相同的层不报。
fn map_level_bands(first_arm: &Arm, second_arm: &Arm, map_scope: MapScope, fanout_basis: FanoutBasis) -> Vec<(u32, u64, u64)> {
    let first_thresholds = map_thresholds_under(first_arm, map_scope, fanout_basis);
    let second_thresholds = map_thresholds_under(second_arm, map_scope, fanout_basis);
    (2u32..=HIGHEST_REPORTED_LEVEL_COUNT)
        .zip(first_thresholds.iter().zip(second_thresholds.iter()))
        .filter(|(_, (first_threshold, second_threshold))| first_threshold != second_threshold)
        .map(|(level_count, (first_threshold, second_threshold))| (level_count, *first_threshold.min(second_threshold), *first_threshold.max(second_threshold)))
        .collect()
}

/// 哪几段区间把 `entry_count` 包在里面（半开区间 [下沿, 上沿)）。
fn level_counts_whose_band_contains(bands: &[(u32, u64, u64)], entry_count: u64) -> Vec<u32> {
    bands.iter().filter(|(_, lower, upper)| *lower <= entry_count && entry_count < *upper).map(|(level_count, _, _)| *level_count).collect()
}

/// 映射叶层字节（只数数据单元）在池规模上的比值 first / second，按 `RATIO_SCALE` 放大后向下取整。
fn map_leaf_bytes_ratio_scaled(first_arm: &Arm, second_arm: &Arm, map_scope: MapScope, fanout_basis: FanoutBasis) -> u128 {
    let first_leaf_bytes = u128::from(POOL_DATA_UNITS.div_ceil(map_fanouts_under(first_arm, map_scope, fanout_basis).0) * NODE_BYTES);
    let second_leaf_bytes = u128::from(POOL_DATA_UNITS.div_ceil(map_fanouts_under(second_arm, map_scope, fanout_basis).0) * NODE_BYTES);
    first_leaf_bytes * RATIO_SCALE / second_leaf_bytes
}

fn scaled_ratio_text(ratio_scaled: u128) -> String {
    format!("{}.{:05}", ratio_scaled / RATIO_SCALE, ratio_scaled % RATIO_SCALE)
}

/// extent 树在池规模上的高（64 口径，同 E133 / E134 的层数口径）。
fn extent_height_at_pool(arm: &Arm, map_scope: MapScope) -> u32 {
    let (_, extent_leaf, extent_internal) = trees(&arm_under_scope(arm, map_scope))[0];
    level_count_for(POOL_DATA_UNITS, extent_leaf, extent_internal)
}
/// 映射树在池规模上的高（只数数据单元，76 口径）。
fn map_height_at_pool(arm: &Arm, map_scope: MapScope) -> u32 {
    let (map_leaf, map_internal) = map_fanouts_under(arm, map_scope, FanoutBasis::MapHeader76);
    level_count_for(POOL_DATA_UNITS, map_leaf, map_internal)
}

/// 码 2 节点在第 `node_level` 层（叶为 0）时，它的父节点到根的 COW 路径有几个节点；数据单元的路径是整棵引用树（补记第 2 条）。
fn copy_on_write_path_node_count_for_node(tree_height: u64, node_level: u64) -> u64 {
    tree_height - 1 - node_level
}
/// 指针权威：每棵引用树 COW 一条路径 + 改写一个根槽（E109 的 `ptr` 臂）。
fn pointer_authority_relocation_bytes(referencing_tree_count: u64, copy_on_write_path_node_count: u64) -> u64 {
    referencing_tree_count * (copy_on_write_path_node_count * NODE_BYTES + ROOT_SLOT_BYTES)
}
/// 映射权威：映射树 COW 一条路径 + 一个根槽，与引用树数、节点所在层无关（E109 的 `map` 臂）。
fn map_authority_relocation_bytes(map_tree_height: u64) -> u64 {
    map_tree_height * NODE_BYTES + ROOT_SLOT_BYTES
}
/// 逻辑身份 + 写序键在 `AllPointers` 下搬一个派生树码 2 节点多读几个节点：从根下到父节点拿分隔 key；别的臂 key 各段都在头里，0。
fn derived_relocation_extra_reads(arm: &Arm, tree_height: u64, node_level: u64) -> u64 {
    match arm.code23_key_bytes {
        None => copy_on_write_path_node_count_for_node(tree_height, node_level),
        Some(_) => 0,
    }
}

fn main() {
    let mut emitter = Emitter::new();
    println!("E136 映射 key 岔路的代价行");
    println!("判据写死在 research/prompts/e136-preregistration.md（写于本装置之前，补记在第一次运行之前）；只报数、不判输赢");
    println!(
        "口径：节点={NODE_BYTES} 节点头(64 口径)={NODE_HEADER_BYTES} 映射节点头={MAP_NODE_HEADER_BYTES} 码2三档头={}/{}/{} 子指针今天={CHILD_POINTER_BYTES_TODAY} 根槽={ROOT_SLOT_BYTES} 池={POOL_DATA_UNITS}",
        CODE2_HEADER_TIERS_TODAY[0], CODE2_HEADER_TIERS_TODAY[1], CODE2_HEADER_TIERS_TODAY[2]
    );
    println!("—— all 配置 × 64 口径（跨装置闸 1：逐行等于 E134 的留存产物）");
    for arm in ARMS.iter() {
        for line in arm_result_lines(arm) {
            println!("{}", emitter.emit_raw(&line));
        }
    }
    println!("—— code1_only 配置 × 64 口径");
    for arm in ARMS.iter() {
        for line in code1_only_result_lines(arm) {
            println!("{}", emitter.emit_raw(&line));
        }
    }
    println!("—— 码 2 三档头（加该配置下的增量）当节点头");
    for arm in ARMS.iter() {
        for map_scope in MAP_SCOPES {
            for tier_bytes in CODE2_HEADER_TIERS_TODAY {
                let fanouts = tier_fanouts(arm, map_scope, tier_bytes);
                let thresholds: Vec<String> = map_thresholds_under(arm, map_scope, FanoutBasis::Code2Tier(tier_bytes))
                    .iter()
                    .zip(2u32..)
                    .map(|(threshold, level_count)| format!("map_l{level_count}={threshold}"))
                    .collect();
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=tier_fanouts arm={} config={} tier={tier_bytes} node_header={} extent_leaf={} extent_internal={} inode_internal={} ledger_internal={} map_leaf={} map_internal={} {}",
                        arm.name,
                        map_scope_name(map_scope),
                        code2_node_header_bytes(arm, map_scope, tier_bytes),
                        fanouts[0],
                        fanouts[1],
                        fanouts[2],
                        fanouts[3],
                        fanouts[4],
                        fanouts[5],
                        thresholds.join(" ")
                    ))
                );
            }
        }
    }
    println!("—— 码 2 头的增量单独拿出来看：同一组条目宽，头取档本身与档 + 增量时的扇出（all 配置，只列加了出生序号的臂）");
    for arm in ARMS.iter().filter(|arm| arm.code23_header_extra_bytes > 0) {
        for tier_bytes in CODE2_HEADER_TIERS_TODAY {
            let (without_increment, with_increment, dropped_cell_count) = fanout_drop_count(tier_bytes, arm.code23_header_extra_bytes, &code2_node_entry_widths(arm));
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=code2_header_increment_effect arm={} tier={tier_bytes} increment={} without={} with={} dropped_cells={dropped_cell_count}",
                    arm.name,
                    arm.code23_header_extra_bytes,
                    slash_joined(&without_increment),
                    slash_joined(&with_increment)
                ))
            );
        }
    }
    println!("—— 映射树层数不同的规模区间 [下沿, 上沿) 与叶层字节比值（池规模 {POOL_DATA_UNITS} 落在哪一段）");
    let pairs = [(&ARMS[0], &ARMS[2]), (&ARMS[1], &ARMS[2]), (&ARMS[0], &ARMS[1])];
    let mut fanout_bases = vec![FanoutBasis::MapHeader76];
    fanout_bases.extend(CODE2_HEADER_TIERS_TODAY.iter().map(|tier_bytes| FanoutBasis::Code2Tier(*tier_bytes)));
    for map_scope in MAP_SCOPES {
        for fanout_basis in fanout_bases.iter() {
            for (first_arm, second_arm) in pairs {
                let bands = map_level_bands(first_arm, second_arm, map_scope, *fanout_basis);
                let bands_text = if bands.is_empty() {
                    "none".to_string()
                } else {
                    bands.iter().map(|(level_count, lower, upper)| format!("{level_count}:{lower}..{upper}")).collect::<Vec<String>>().join(",")
                };
                let pool_levels: Vec<String> = level_counts_whose_band_contains(&bands, POOL_DATA_UNITS).iter().map(|level_count| level_count.to_string()).collect();
                let pool_text = if pool_levels.is_empty() { "none".to_string() } else { pool_levels.join(",") };
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=map_level_bands config={} basis={} pair={}/{} bands={bands_text} pool_in={pool_text} leaf_bytes_ratio={}",
                        map_scope_name(map_scope),
                        fanout_basis_name(*fanout_basis),
                        first_arm.name,
                        second_arm.name,
                        scaled_ratio_text(map_leaf_bytes_ratio_scaled(first_arm, second_arm, map_scope, *fanout_basis))
                    ))
                );
            }
        }
    }
    println!("—— 搬一个码 2 / 码 3 节点（all：映射权威；code1_only：节点的位置条目回到权威）");
    for arm in ARMS.iter() {
        let map_tree_height = u64::from(map_height_at_pool(arm, MapScope::AllPointers));
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=relocate_node config=all arm={} map_height={map_tree_height} bytes={}",
                arm.name,
                map_authority_relocation_bytes(map_tree_height)
            ))
        );
    }
    for arm in ARMS.iter() {
        let extent_height = u64::from(extent_height_at_pool(arm, MapScope::Code1Only));
        for node_level in RELOCATED_NODE_LEVELS {
            let copy_on_write_path_node_count = copy_on_write_path_node_count_for_node(extent_height, node_level);
            for referencing_tree_count in REFERENCING_TREE_COUNTS {
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=relocate_node config=code1_only arm={} extent_height={extent_height} level={node_level} k={referencing_tree_count} bytes={} parent_reads={}",
                        arm.name,
                        pointer_authority_relocation_bytes(referencing_tree_count, copy_on_write_path_node_count),
                        referencing_tree_count * copy_on_write_path_node_count
                    ))
                );
            }
        }
    }
    println!("—— 派生树码 2 节点搬迁多读的节点数（all 配置，extent 树在池规模上的高）");
    for arm in ARMS.iter() {
        let extent_height = u64::from(extent_height_at_pool(arm, MapScope::AllPointers));
        for node_level in RELOCATED_NODE_LEVELS {
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=derived_relocation_extra_reads config=all arm={} extent_height={extent_height} level={node_level} reads={}",
                    arm.name,
                    derived_relocation_extra_reads(arm, extent_height, node_level)
                ))
            );
        }
    }
    let registered_row_entry54_fanouts: Vec<String> = CODE2_HEADER_TIERS_TODAY.iter().map(|tier_bytes| fanout_with_header(*tier_bytes, 54).to_string()).collect();
    let registered_row_entry81_fanouts: Vec<String> = CODE2_HEADER_TIERS_TODAY.iter().map(|tier_bytes| fanout_with_header(*tier_bytes, 81).to_string()).collect();
    let e109_pointer_authority: Vec<String> = REFERENCING_TREE_COUNTS.iter().map(|referencing_tree_count| pointer_authority_relocation_bytes(*referencing_tree_count, 4).to_string()).collect();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=cross_apparatus d18_308_entry54={} d18_308_entry81={} e109_move_ptr={} e109_move_map={}",
            registered_row_entry54_fanouts.join("/"),
            registered_row_entry81_fanouts.join("/"),
            e109_pointer_authority.join("/"),
            map_authority_relocation_bytes(4)
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm_named(name: &str) -> Arm {
        *ARMS.iter().find(|arm| arm.name == name).expect("臂名写错了：ARMS 里没有这一条")
    }

    /// 跨装置闸 1：`all` 配置 × 64 口径的每一行，逐字节出现在 E134 的留存产物里。
    #[test]
    fn all_pointers_lines_reproduce_the_e134_product_line_by_line() {
        let e134_product = include_str!("../../../results/e134-map-key-slot-baselines-2026-09-11.out");
        let product_lines: Vec<&str> = e134_product.lines().collect();
        let mut checked_line_count = 0;
        for arm in ARMS.iter() {
            for line in arm_result_lines(arm) {
                let expected = format!("E7RESULT {line}");
                assert!(product_lines.contains(&expected.as_str()), "E134 的留存产物里没有这一行：{expected}");
                checked_line_count += 1;
            }
        }
        assert_eq!(checked_line_count, 3 * 8, "三条臂各 8 行（widths / headers / fanouts / 四棵树的门槛 / map_at_pool）都要对上");
    }

    /// 跨装置闸 2：三档头的扇出式复现 D18 第 308 行登记的「扇出（条目 54）301 / 301 / 301」「扇出（条目 81）201 / 200 / 200」。
    #[test]
    fn code2_tier_fanout_formula_reproduces_the_registered_header_row() {
        let entry54: Vec<u64> = CODE2_HEADER_TIERS_TODAY.iter().map(|tier_bytes| fanout_with_header(*tier_bytes, 54)).collect();
        let entry81: Vec<u64> = CODE2_HEADER_TIERS_TODAY.iter().map(|tier_bytes| fanout_with_header(*tier_bytes, 81)).collect();
        assert_eq!(entry54, vec![301, 301, 301], "D18 第 308 行：条目 54 在三档头下扇出 301 / 301 / 301");
        assert_eq!(entry81, vec![201, 200, 200], "D18 第 308 行：条目 81 在三档头下扇出 201 / 200 / 200");
    }

    /// 跨装置闸 3：数据单元（路径 = 引用树高 4）、映射树高 4 时，搬迁字节逐格等于 E109 留存产物里 key 33 那五行。
    #[test]
    fn relocation_count_reproduces_the_e109_move_table() {
        let e109_product = include_str!("../../../results/e109-position-authority-2026-09-06.out");
        let mut checked_row_count = 0;
        for referencing_tree_count in REFERENCING_TREE_COUNTS {
            let expected_prefix = format!("E7RESULT name=move key_bytes=33 k={referencing_tree_count} map_fanout=267 map_height=4 ");
            let row = e109_product.lines().find(|line| line.starts_with(&expected_prefix)).expect("E109 的留存产物里找不到这一行");
            let expected_fields = format!(
                "ptr_bytes={} map_bytes={}",
                pointer_authority_relocation_bytes(referencing_tree_count, 4),
                map_authority_relocation_bytes(4)
            );
            assert!(row.contains(&expected_fields), "E109 那一行是 {row}，本装置算出 {expected_fields}");
            checked_row_count += 1;
        }
        assert_eq!(checked_row_count, 5);
        assert_eq!(pointer_authority_relocation_bytes(1, 4), 66_048, "E109：K = 1 时指针权威 66 048 字节");
        assert_eq!(map_authority_relocation_bytes(4), 66_048, "E109：映射权威 66 048 字节，与 K 无关");
    }

    #[test]
    fn code1_only_widths_are_pinned() {
        let expectations = [
            ("k1_seq", [109, 75, 133, 99, 109, 97, 137, 226, 79]),
            ("k1_seq_class_reuse", [109, 75, 133, 99, 109, 97, 137, 226, 79]),
            ("class_reuse_tagged", [85, 75, 109, 99, 109, 97, 137, 226, 55]),
        ];
        for (name, expected) in expectations {
            let scoped = arm_under_scope(&arm_named(name), MapScope::Code1Only);
            let actual = [
                data_child_pointer_bytes(&scoped),
                node_child_pointer_bytes(&scoped),
                extent_leaf_entry_bytes(&scoped),
                extent_internal_entry_bytes(&scoped),
                inode_internal_entry_bytes(&scoped),
                ledger_internal_entry_bytes(&scoped),
                tree_table_entry_bytes(&scoped),
                root_record_bytes(&scoped),
                map_entry_bytes(scoped.code1_key_bytes),
            ];
            assert_eq!(actual, expected, "{name} 在只装码 1 时的宽度");
        }
    }

    #[test]
    fn code1_only_fanouts_and_map_levels_are_pinned() {
        let expectations = [
            ("k1_seq", [122, 164, 149, 168, 118, 206, 129, 233], 5, 38_429_540_352),
            ("k1_seq_class_reuse", [122, 164, 149, 168, 118, 206, 129, 233], 5, 38_429_540_352),
            ("class_reuse_tagged", [149, 164, 149, 168, 118, 296, 159, 233], 4, 26_744_881_152),
        ];
        for (name, expected_fanouts, expected_levels, expected_leaf_bytes) in expectations {
            let scoped = arm_under_scope(&arm_named(name), MapScope::Code1Only);
            let actual = [
                node_fanout(extent_leaf_entry_bytes(&scoped)),
                node_fanout(extent_internal_entry_bytes(&scoped)),
                node_fanout(inode_internal_entry_bytes(&scoped)),
                node_fanout(ledger_internal_entry_bytes(&scoped)),
                tree_table_entries_per_level(tree_table_entry_bytes(&scoped)),
                map_leaf_fanout(scoped.code1_key_bytes),
                map_internal_fanout(scoped.code1_key_bytes, node_child_pointer_bytes(&scoped)),
                inode_leaf_capacity(&scoped),
            ];
            assert_eq!(actual, expected_fanouts, "{name} 在只装码 1 时的 64 口径扇出");
            assert_eq!(map_height_at_pool(&arm_named(name), MapScope::Code1Only), expected_levels, "{name} 只装码 1 时映射树在池规模上的层数");
            assert_eq!(POOL_DATA_UNITS.div_ceil(scoped_map_leaf(name)) * NODE_BYTES, expected_leaf_bytes, "{name} 只装码 1 时映射叶层字节");
        }
    }

    fn scoped_map_leaf(name: &str) -> u64 {
        map_leaf_fanout(arm_under_scope(&arm_named(name), MapScope::Code1Only).code1_key_bytes)
    }

    /// 码 2 头的增量只在全部指针进映射、且那条臂加出生序号时才加；只装码 1 时三条都是档本身。
    #[test]
    fn code2_header_increment_is_applied_only_where_the_birth_sequence_exists() {
        for tier_bytes in CODE2_HEADER_TIERS_TODAY {
            assert_eq!(code2_node_header_bytes(&arm_named("k1_seq"), MapScope::AllPointers, tier_bytes), tier_bytes);
            assert_eq!(code2_node_header_bytes(&arm_named("k1_seq_class_reuse"), MapScope::AllPointers, tier_bytes), tier_bytes + 4);
            assert_eq!(code2_node_header_bytes(&arm_named("class_reuse_tagged"), MapScope::AllPointers, tier_bytes), tier_bytes + 4);
            for arm in ARMS.iter() {
                assert_eq!(code2_node_header_bytes(arm, MapScope::Code1Only, tier_bytes), tier_bytes, "{} 只装码 1 时码 2 头不加出生序号", arm.name);
            }
        }
    }

    /// 三档头下的扇出：(extent 叶, extent 内部, inode 内部, 记账内部, 映射叶, 映射内部)。期望值在装置之前另算。
    #[test]
    fn tier_fanouts_are_pinned() {
        let expectations: [(&str, MapScope, [[u64; 6]; 3]); 6] = [
            ("k1_seq", MapScope::AllPointers, [[122, 164, 149, 167, 206, 129], [122, 164, 149, 167, 206, 129], [122, 164, 149, 167, 205, 129]]),
            ("k1_seq", MapScope::Code1Only, [[122, 164, 149, 167, 206, 129], [122, 164, 149, 167, 206, 129], [122, 164, 149, 167, 205, 129]]),
            ("k1_seq_class_reuse", MapScope::AllPointers, [[122, 152, 139, 155, 206, 121], [122, 152, 139, 155, 206, 121], [122, 152, 139, 154, 205, 121]]),
            ("k1_seq_class_reuse", MapScope::Code1Only, [[122, 164, 149, 167, 206, 129], [122, 164, 149, 167, 206, 129], [122, 164, 149, 167, 205, 129]]),
            ("class_reuse_tagged", MapScope::AllPointers, [[149, 152, 139, 155, 296, 148], [149, 152, 139, 155, 295, 147], [149, 152, 139, 154, 295, 147]]),
            ("class_reuse_tagged", MapScope::Code1Only, [[149, 164, 149, 167, 296, 159], [149, 164, 149, 167, 295, 159], [149, 164, 149, 167, 295, 159]]),
        ];
        for (name, map_scope, per_tier) in expectations {
            for (tier_index, tier_bytes) in CODE2_HEADER_TIERS_TODAY.iter().enumerate() {
                assert_eq!(tier_fanouts(&arm_named(name), map_scope, *tier_bytes), per_tier[tier_index], "{name} / {map_scope:?} / 档 {tier_bytes}");
            }
        }
    }

    /// 码 2 头 +4：两条加出生序号的臂、三档头、六种条目宽，一格都不掉（期望值在装置之前另算）。
    #[test]
    fn the_code2_header_increment_drops_no_fanout_cell_at_any_tier() {
        let expectations = [
            ("k1_seq_class_reuse", [[122, 152, 139, 155, 206, 121], [122, 152, 139, 155, 206, 121], [122, 152, 139, 154, 205, 121]]),
            ("class_reuse_tagged", [[149, 152, 139, 155, 296, 148], [149, 152, 139, 155, 295, 147], [149, 152, 139, 154, 295, 147]]),
        ];
        for (name, per_tier) in expectations {
            let arm = arm_named(name);
            for (tier_index, tier_bytes) in CODE2_HEADER_TIERS_TODAY.iter().enumerate() {
                let (without_increment, with_increment, dropped_cell_count) = fanout_drop_count(*tier_bytes, 4, &code2_node_entry_widths(&arm));
                assert_eq!(without_increment, per_tier[tier_index].to_vec(), "{name} 档 {tier_bytes} 不加增量");
                assert_eq!(with_increment, per_tier[tier_index].to_vec(), "{name} 档 {tier_bytes} 加 4");
                assert_eq!(dropped_cell_count, 0, "{name} 档 {tier_bytes}：加 4 不该掉格");
            }
        }
    }

    /// 阳性对照：增量跨过条目边界时，计数器必须报出掉格（16288 / 8144 = 2，16284 / 8144 = 1）。
    #[test]
    fn the_drop_counter_sees_a_drop_when_the_increment_crosses_an_entry_boundary() {
        assert_eq!(fanout_drop_count(96, 4, &[8144]), (vec![2], vec![1], 1));
        assert_eq!(fanout_drop_count(96, 0, &[8144]), (vec![2], vec![2], 0));
    }

    #[test]
    fn map_thresholds_are_pinned_under_both_bases() {
        assert_eq!(map_thresholds_under(&arm_named("k1_seq"), MapScope::AllPointers, FanoutBasis::MapHeader76), vec![207, 26_575, 3_428_047, 442_217_935, 57_046_113_487]);
        assert_eq!(map_thresholds_under(&arm_named("k1_seq_class_reuse"), MapScope::AllPointers, FanoutBasis::MapHeader76), vec![207, 24_927, 3_016_047, 364_941_567, 44_157_929_487]);
        assert_eq!(map_thresholds_under(&arm_named("class_reuse_tagged"), MapScope::AllPointers, FanoutBasis::MapHeader76), vec![297, 43_809, 6_483_585, 959_570_433, 142_016_423_937]);
        assert_eq!(map_thresholds_under(&arm_named("class_reuse_tagged"), MapScope::Code1Only, FanoutBasis::MapHeader76), vec![297, 47_065, 7_483_177, 1_189_824_985, 189_182_172_457]);
        assert_eq!(map_thresholds_under(&arm_named("k1_seq"), MapScope::AllPointers, FanoutBasis::Code2Tier(114)), vec![206, 26_446, 3_411_406, 440_071_246, 56_769_190_606]);
        assert_eq!(map_thresholds_under(&arm_named("class_reuse_tagged"), MapScope::AllPointers, FanoutBasis::Code2Tier(105)), vec![296, 43_366, 6_374_656, 937_074_286, 137_749_919_896]);
        assert_eq!(map_thresholds_under(&arm_named("class_reuse_tagged"), MapScope::Code1Only, FanoutBasis::Code2Tier(105)), vec![296, 46_906, 7_457_896, 1_185_805_306, 188_543_043_496]);
    }

    #[test]
    fn map_level_bands_and_leaf_ratios_are_pinned() {
        let logical = arm_named("k1_seq");
        let mixed = arm_named("k1_seq_class_reuse");
        let class_reuse = arm_named("class_reuse_tagged");
        assert_eq!(
            map_level_bands(&logical, &class_reuse, MapScope::AllPointers, FanoutBasis::MapHeader76),
            vec![(2, 207, 297), (3, 26_575, 43_809), (4, 3_428_047, 6_483_585), (5, 442_217_935, 959_570_433), (6, 57_046_113_487, 142_016_423_937)]
        );
        assert_eq!(
            map_level_bands(&mixed, &class_reuse, MapScope::AllPointers, FanoutBasis::MapHeader76),
            vec![(2, 207, 297), (3, 24_927, 43_809), (4, 3_016_047, 6_483_585), (5, 364_941_567, 959_570_433), (6, 44_157_929_487, 142_016_423_937)]
        );
        assert_eq!(
            map_level_bands(&logical, &mixed, MapScope::AllPointers, FanoutBasis::MapHeader76),
            vec![(3, 24_927, 26_575), (4, 3_016_047, 3_428_047), (5, 364_941_567, 442_217_935), (6, 44_157_929_487, 57_046_113_487)]
        );
        assert_eq!(
            map_level_bands(&logical, &class_reuse, MapScope::Code1Only, FanoutBasis::MapHeader76),
            vec![(2, 207, 297), (3, 26_575, 47_065), (4, 3_428_047, 7_483_177), (5, 442_217_935, 1_189_824_985), (6, 57_046_113_487, 189_182_172_457)]
        );
        assert!(map_level_bands(&logical, &mixed, MapScope::Code1Only, FanoutBasis::MapHeader76).is_empty(), "只装码 1 时混合路就是逻辑身份 + 写序键，层数处处相同");
        assert_eq!(
            map_level_bands(&mixed, &class_reuse, MapScope::AllPointers, FanoutBasis::Code2Tier(114)),
            vec![(2, 206, 296), (3, 24_806, 43_366), (4, 3_001_406, 6_374_656), (5, 363_170_006, 937_074_286), (6, 43_943_570_606, 137_749_919_896)]
        );
        assert_eq!(scaled_ratio_text(map_leaf_bytes_ratio_scaled(&logical, &class_reuse, MapScope::AllPointers, FanoutBasis::MapHeader76)), "1.43689");
        assert_eq!(scaled_ratio_text(map_leaf_bytes_ratio_scaled(&logical, &class_reuse, MapScope::AllPointers, FanoutBasis::Code2Tier(105))), "1.43203");
        assert_eq!(scaled_ratio_text(map_leaf_bytes_ratio_scaled(&logical, &class_reuse, MapScope::AllPointers, FanoutBasis::Code2Tier(114))), "1.43902");
        assert_eq!(scaled_ratio_text(map_leaf_bytes_ratio_scaled(&logical, &mixed, MapScope::AllPointers, FanoutBasis::MapHeader76)), "1.00000");
    }

    #[test]
    fn the_pool_falls_in_a_band_only_where_the_two_thresholds_straddle_it() {
        let logical = arm_named("k1_seq");
        let mixed = arm_named("k1_seq_class_reuse");
        let class_reuse = arm_named("class_reuse_tagged");
        assert_eq!(level_counts_whose_band_contains(&map_level_bands(&logical, &class_reuse, MapScope::AllPointers, FanoutBasis::MapHeader76), POOL_DATA_UNITS), vec![5]);
        assert!(
            level_counts_whose_band_contains(&map_level_bands(&logical, &mixed, MapScope::AllPointers, FanoutBasis::MapHeader76), POOL_DATA_UNITS).is_empty(),
            "池规模 483 183 820 在 364 941 567..442 217 935 之外，两条都是 5 层"
        );
        assert_eq!(level_counts_whose_band_contains(&[(5, POOL_DATA_UNITS, POOL_DATA_UNITS + 1)], POOL_DATA_UNITS), vec![5], "下沿是闭的");
        assert!(level_counts_whose_band_contains(&[(5, POOL_DATA_UNITS - 1, POOL_DATA_UNITS)], POOL_DATA_UNITS).is_empty(), "上沿是开的");
    }

    #[test]
    fn relocation_bytes_and_heights_are_pinned() {
        assert_eq!(extent_height_at_pool(&arm_named("k1_seq"), MapScope::AllPointers), 4);
        assert_eq!(extent_height_at_pool(&arm_named("k1_seq_class_reuse"), MapScope::AllPointers), 5, "混合路的 extent 内部扇出 152，池规模上比另两条高一层");
        assert_eq!(extent_height_at_pool(&arm_named("class_reuse_tagged"), MapScope::AllPointers), 4);
        for arm in ARMS.iter() {
            assert_eq!(extent_height_at_pool(arm, MapScope::Code1Only), 4, "{} 只装码 1 时 extent 树高", arm.name);
        }
        assert_eq!(map_authority_relocation_bytes(u64::from(map_height_at_pool(&arm_named("k1_seq"), MapScope::AllPointers))), 82_432);
        assert_eq!(map_authority_relocation_bytes(u64::from(map_height_at_pool(&arm_named("k1_seq_class_reuse"), MapScope::AllPointers))), 82_432);
        assert_eq!(map_authority_relocation_bytes(u64::from(map_height_at_pool(&arm_named("class_reuse_tagged"), MapScope::AllPointers))), 66_048);
        assert_eq!(pointer_authority_relocation_bytes(1, copy_on_write_path_node_count_for_node(4, 0)), 49_664, "叶层节点、树高 4：父到根 3 个节点 + 根槽");
        assert_eq!(pointer_authority_relocation_bytes(16, copy_on_write_path_node_count_for_node(4, 0)), 794_624);
        assert_eq!(pointer_authority_relocation_bytes(1, copy_on_write_path_node_count_for_node(4, 1)), 33_280);
        assert_eq!(pointer_authority_relocation_bytes(1, copy_on_write_path_node_count_for_node(4, 2)), 16_896);
    }

    #[test]
    fn derived_relocation_extra_reads_fall_only_on_the_logical_identity_arm() {
        assert_eq!(derived_relocation_extra_reads(&arm_named("k1_seq"), 4, 0), 3);
        assert_eq!(derived_relocation_extra_reads(&arm_named("k1_seq"), 4, 1), 2);
        assert_eq!(derived_relocation_extra_reads(&arm_named("k1_seq"), 4, 2), 1);
        assert_eq!(derived_relocation_extra_reads(&arm_named("k1_seq_class_reuse"), 5, 0), 0);
        assert_eq!(derived_relocation_extra_reads(&arm_named("class_reuse_tagged"), 4, 0), 0);
    }

    /// 只装码 1 时混合路逐格等于逻辑身份 + 写序键；全部指针进映射时两者的节点指针不同——配置真的在切换。
    #[test]
    fn the_map_scope_switch_changes_the_mixed_arm_and_nothing_else_changes_the_logical_arm() {
        let mixed_code1 = code1_only_result_lines(&arm_named("k1_seq_class_reuse"));
        let logical_code1 = code1_only_result_lines(&arm_named("k1_seq"));
        let strip_name = |lines: Vec<String>, name: &str| -> Vec<String> { lines.iter().map(|line| line.replace(&format!("arm={name} "), "arm=_ ")).collect() };
        assert_eq!(strip_name(mixed_code1, "k1_seq_class_reuse"), strip_name(logical_code1, "k1_seq"));
        assert_eq!(node_child_pointer_bytes(&arm_under_scope(&arm_named("k1_seq_class_reuse"), MapScope::AllPointers)), 83);
        assert_eq!(node_child_pointer_bytes(&arm_under_scope(&arm_named("k1_seq_class_reuse"), MapScope::Code1Only)), 75);
        assert_eq!(node_child_pointer_bytes(&arm_under_scope(&arm_named("k1_seq"), MapScope::AllPointers)), 75);
    }
}
