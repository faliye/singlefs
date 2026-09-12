//! E133：映射 key 各候选的格式代价。
//!
//! D19 已定项 6 要把「中央映射条目的 key 取哪一族」交给用户定；交之前每条路要有代价数。
//! 仓里此前没有一份装置同时算过这几条候选：E128 只算了甲（指针加 16），E109 只扫了 key 33 / 40 两档。
//!
//! **判据、口径、作废条款、以及「结果反过来我接不接受」写在
//! `research/prompts/e133-preregistration.md`，写于本文件之前；开跑之前补的臂记在它的「补记」一节。**
//! ⚠️ **只报数、不判输赢**：`.claude/rules/fs-design.md` 逐字「「比谁省」不构成任何判据」。
//! 判据全是装置对账——跨装置闸（与 E128 / E109 / D19 未定项 7 那一格对同一个量）与阳性对照。
//!
//! 计数模型，不是实现：没有 I/O、没有随机源 ⇒ 跑 N 遍必然逐字节一致，
//! 证据强度来自变异测试与钉死绝对值的断言。
//!
//! ⚠️ **指针增量分两个量**：extent 叶里指向码 1 数据单元的那条，与内部节点、树表条目、根记录里
//! 指向码 2 / 码 3 的那几条。逻辑身份族只在前一种上带对象 ID / 对象出生代 / 锚点偏移；按类复用写序键两种宽度不同。
//! ⚠️ 码 2 头的增量**不回灌进扇出**：扇出式里的节点头 64 与 D18 的码 2 三档头不是同一个口径，
//! 把增量加进 64 等于自己挑一个口径（`.claude/rules/mutation-sampling.md` 第五类）。头宽只照报。
//! ⚠️ D27 的槽只报增量不报容量：槽今天带哪些字段 D27 已定项 3 自陈「仍未写死」。

use e7_index_bench::Emitter;

const NODE_BYTES: u64 = 16384; // D8 已定项 2
/// 引用树的节点头，E128 的 `NODE_HDR`：它复现 D8 的 inode 树 175 与 D18 的记账树 201。
const NODE_HEADER_BYTES: u64 = 64;
/// 映射树的节点头，E109 的 `MAP_HDR`。
const MAP_NODE_HEADER_BYTES: u64 = 76;
/// 树表单元装条目的净字节，`22-单元原子性怎么合成.md` 的口径（C157 另记 16320，不采用）。
const TREE_TABLE_PAYLOAD_BYTES: u64 = 16284;
/// 指针头部：MAC 16 + nonce 12 + 算法类型 1 + extent 偏移 2（D21 正文，D19 未定项 7 那一格整句引）。
const POINTER_HEAD_BYTES_TODAY: u64 = 16 + 12 + 1 + 2;
/// 位置条目：设备 4 + 16 KiB 槽号 6 + 密文校验和 4（D19 已定项 4）。
const LOCATION_ENTRY_BYTES: u64 = 4 + 6 + 4;
/// 第一版两块盘，每个单元两盘各一份（D2 已定项 9）。
const REPLICA_COUNT: u64 = 2;
const CHILD_POINTER_BYTES_TODAY: u64 = POINTER_HEAD_BYTES_TODAY + LOCATION_ENTRY_BYTES * REPLICA_COUNT;
/// 映射条目的 value：每个副本一份位置条目（D19 已定项 5）。
const MAP_VALUE_BYTES: u64 = LOCATION_ENTRY_BYTES * REPLICA_COUNT;
/// inode 树内部条目除子指针之外：分隔 key 8 + 身份引用 26（E128 `derived()`）。
const INODE_INTERNAL_NON_POINTER_BYTES: u64 = 8 + 26;
/// 记账树内部条目除子指针之外：记账 key 22（E128 `derived()`）。
const LEDGER_INTERNAL_NON_POINTER_BYTES: u64 = 22;
/// extent 树条目除子指针之外：locality 8 + inode 8 + offset 8（D8 已定项 3）；叶与内部同一个 key。
const EXTENT_NON_POINTER_BYTES: u64 = 8 + 8 + 8;
/// 记账树叶条目：key 22 + 完整值 8（D5 已定项 5 逐字「条目 30 字节」）。它不含指针，各臂相同。
/// ⚠️ D8 已定项 7 定 write buffer 条目的 seq 4 字节住 value，按那个口径是 34；两个口径各臂同值，
/// 只影响记账树门槛的绝对值、不影响臂间比较，这里取 D5 那一处并注明（不自己挑第三个）。
const LEDGER_LEAF_ENTRY_BYTES: u64 = 22 + 8;
/// 树表条目除根指针之外：长度 2 + 种类 2 + flags 2 + 树 ID 8 + previous_snapshot_txg 8 + 诞生 txg 8 + 预留 32（D8 已定项 8）。
const TREE_TABLE_NON_POINTER_BYTES: u64 = 2 + 2 + 2 + 8 + 8 + 8 + 32;
/// 根记录除两条指针之外：magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树 ID 水位 8 + 自证校验和 32（D22 已定项 7）。
const ROOT_RECORD_NON_POINTER_BYTES: u64 = 4 + 16 + 4 + 4 + 8 + 8 + 32;
/// 根记录里的指针条数：树表单元指针 + 实例表单元指针（D22 已定项 7）。两条都按指向码 2 / 3 那一种算。
const ROOT_RECORD_POINTER_COUNT: u64 = 2;
/// inode 记录定长（D8 已定项 6）；inode 树的叶是码 3 容器。
const INODE_RECORD_BYTES: u64 = 140;
const DATA_UNIT_BYTES: u64 = 32768; // D4 已定项 1
/// 单元头今天的宽度（D18 已定项 14 的头宽表，含 nonce / MAC 预留位）。
const CODE1_HEADER_BYTES_TODAY: u64 = 133;
const CODE3_HEADER_BYTES_TODAY: u64 = 131;
const CODE2_HEADER_TIERS_TODAY: [u64; 3] = [96, 105, 114];
/// E109 的池规模：16 TiB 量级的数据单元数（它自陈只数数据单元）。
const POOL_DATA_UNITS: u64 = 483_183_820;
/// E109 的引用树扇出，用来估「含索引节点」那一档：每 119 个单元多一个索引节点。
const INDEX_FANOUT_FOR_NODE_COUNT: u64 = 119;
/// E109 的 key 宽档，跨装置闸只在这一档上对 267。
const E109_POSITION_AUTHORITY_KEY_BYTES: u64 = 33;

/// 一条候选（或基准）的格式形态。字段含义见 `research/prompts/e133-preregistration.md` 的臂表与补记。
#[derive(Clone, Copy, Debug)]
struct Arm {
    name: &'static str,
    /// extent 叶里指向码 1 数据单元的那条指针要多带几字节。
    data_pointer_extra_bytes: u64,
    /// 内部节点、树表条目、根记录里指向码 2 / 码 3 的指针要多带几字节。
    node_pointer_extra_bytes: u64,
    code1_header_extra_bytes: u64,
    code23_header_extra_bytes: u64,
    /// 码 1 单元的映射 key 宽。
    code1_key_bytes: u64,
    /// 码 2 / 码 3 单元的映射 key 宽；逻辑身份族给不出定宽（码 2 的 key 带 key 区间下界，宽随树变）⇒ None。
    code23_key_bytes: Option<u64>,
    root_record_extra_bytes: u64,
    /// D27 的槽要多带几字节才装得下这条候选的 key，基线取 E114 `pack` 臂的槽：五元组 33（其树 ID 段按 I-1.3 是出生树）+ 写序 10。
    /// D27 正文里槽只写了五元组；按那个基线的数在 E134（映射 key 格式代价·槽基线与两条新臂）；本产物被三方论证逐行引用，不原地改。
    packed_slot_extra_bytes: u64,
}

const ARMS: [Arm; 12] = [
    // 今天：指针里没有出生身份；映射 key 按第一轮之前的预想取五元组 33。只当基准。
    Arm { name: "bing", data_pointer_extra_bytes: 0, node_pointer_extra_bytes: 0, code1_header_extra_bytes: 0, code23_header_extra_bytes: 0, code1_key_bytes: 33, code23_key_bytes: None, root_record_extra_bytes: 0, packed_slot_extra_bytes: 0 },
    // 逻辑身份键按背景材料的字面：每条指针只加出生树 + 出生 txg（= D19 未定项 7 的甲）；槽缺诞生代号 8。
    Arm { name: "k1", data_pointer_extra_bytes: 16, node_pointer_extra_bytes: 16, code1_header_extra_bytes: 0, code23_header_extra_bytes: 0, code1_key_bytes: 41, code23_key_bytes: None, root_record_extra_bytes: 0, packed_slot_extra_bytes: 8 },
    // 逻辑身份键最强形态：指向码 1 的指针另带对象 ID 8 + 对象出生代 8 + 锚点偏移 8；指向码 2 / 3 的只带甲的 16（层级与 key 区间下界由查找路径给）。
    Arm { name: "k1_full", data_pointer_extra_bytes: 40, node_pointer_extra_bytes: 16, code1_header_extra_bytes: 0, code23_header_extra_bytes: 0, code1_key_bytes: 41, code23_key_bytes: None, root_record_extra_bytes: 0, packed_slot_extra_bytes: 8 },
    // 逻辑身份 + 写序键：在最强形态上再带写序 10；码 1 头里本来就有写序。码 2 的 key 取 K1a 形态、宽随树变，给不出定宽。
    Arm { name: "k1_seq", data_pointer_extra_bytes: 50, node_pointer_extra_bytes: 16, code1_header_extra_bytes: 0, code23_header_extra_bytes: 0, code1_key_bytes: 51, code23_key_bytes: None, root_record_extra_bytes: 0, packed_slot_extra_bytes: 8 },
    // 代理流水号：号 8 + 出生树 8 + 出生 txg 8；三类头各加号 8；根记录加水位 8。
    Arm { name: "k2", data_pointer_extra_bytes: 24, node_pointer_extra_bytes: 24, code1_header_extra_bytes: 8, code23_header_extra_bytes: 8, code1_key_bytes: 8, code23_key_bytes: Some(8), root_record_extra_bytes: 8, packed_slot_extra_bytes: 8 },
    // 实例内计数键：实例代号 4 + 实例内计数 8，另加出生树 8 + 出生 txg 8；头各加计数 8（实例代号取写序里现成的）；计数随实例从头发，不要水位。
    Arm { name: "k2_instance", data_pointer_extra_bytes: 28, node_pointer_extra_bytes: 28, code1_header_extra_bytes: 8, code23_header_extra_bytes: 8, code1_key_bytes: 12, code23_key_bytes: Some(12), root_record_extra_bytes: 0, packed_slot_extra_bytes: 8 },
    // 出生身份键：出生树 8 + 出生 txg 8 + 出生序号 4；三类头各加序号 4；槽加诞生代号 8 + 序号 4。
    Arm { name: "k3", data_pointer_extra_bytes: 20, node_pointer_extra_bytes: 20, code1_header_extra_bytes: 4, code23_header_extra_bytes: 4, code1_key_bytes: 20, code23_key_bytes: Some(20), root_record_extra_bytes: 0, packed_slot_extra_bytes: 12 },
    // 出生身份 + 实例键：再加实例代号 4；头只加序号 4（实例代号取写序里现成的）。
    Arm { name: "k3_instance", data_pointer_extra_bytes: 24, node_pointer_extra_bytes: 24, code1_header_extra_bytes: 4, code23_header_extra_bytes: 4, code1_key_bytes: 24, code23_key_bytes: Some(24), root_record_extra_bytes: 0, packed_slot_extra_bytes: 12 },
    // 按类复用写序键：码 1 的 key = 出生树 + 出生 txg + 写序 10，码 2 / 3 = 出生树 + 出生 txg + 实例代号 + 出生序号；只有码 2 / 3 头加序号 4。
    Arm { name: "class_reuse", data_pointer_extra_bytes: 26, node_pointer_extra_bytes: 24, code1_header_extra_bytes: 0, code23_header_extra_bytes: 4, code1_key_bytes: 26, code23_key_bytes: Some(24), root_record_extra_bytes: 0, packed_slot_extra_bytes: 8 },
    // 出生落点键：位置条目冻结当 key 前两段，另加出生 txg 8 + 出生树 8（后者给 E90）；三类头各加出生落点 10。
    Arm { name: "k4", data_pointer_extra_bytes: 16, node_pointer_extra_bytes: 16, code1_header_extra_bytes: 10, code23_header_extra_bytes: 10, code1_key_bytes: 18, code23_key_bytes: Some(18), root_record_extra_bytes: 0, packed_slot_extra_bytes: 18 },
    // 本地腿 s1：随机 16 字节标识 + 出生树 8 + 出生 txg 8；三类头各加标识 16。
    Arm { name: "uuid16", data_pointer_extra_bytes: 32, node_pointer_extra_bytes: 32, code1_header_extra_bytes: 16, code23_header_extra_bytes: 16, code1_key_bytes: 16, code23_key_bytes: Some(16), root_record_extra_bytes: 0, packed_slot_extra_bytes: 16 },
    // 本地腿 s2：出生落点 10 + 每槽版本号 2 当 key，指针另加版本号 2 + 出生树 8 + 出生 txg 8；每槽版本号那张持久表的代价不建模。
    Arm { name: "slotgen12", data_pointer_extra_bytes: 18, node_pointer_extra_bytes: 18, code1_header_extra_bytes: 12, code23_header_extra_bytes: 12, code1_key_bytes: 12, code23_key_bytes: Some(12), root_record_extra_bytes: 0, packed_slot_extra_bytes: 12 },
];

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

/// 引用树一个节点装几条宽 `entry_bytes` 的条目。
fn node_fanout(entry_bytes: u64) -> u64 {
    (NODE_BYTES - NODE_HEADER_BYTES) / entry_bytes
}
fn tree_table_entries_per_level(entry_bytes: u64) -> u64 {
    TREE_TABLE_PAYLOAD_BYTES / entry_bytes
}
/// 映射树叶：key + value，E109 的口径。
fn map_leaf_fanout(key_bytes: u64) -> u64 {
    (NODE_BYTES - MAP_NODE_HEADER_BYTES) / map_entry_bytes(key_bytes)
}
/// 映射树内部节点：key + 一条指向映射树自己节点（码 2）的子指针（E132 改正 E131 的那一处）。
fn map_internal_fanout(key_bytes: u64, child_pointer: u64) -> u64 {
    (NODE_BYTES - MAP_NODE_HEADER_BYTES) / (key_bytes + child_pointer)
}
/// inode 树的叶是码 3 容器：头之后按 140 字节一条记录。
fn inode_leaf_capacity(arm: &Arm) -> u64 {
    (DATA_UNIT_BYTES - (CODE3_HEADER_BYTES_TODAY + arm.code23_header_extra_bytes)) / INODE_RECORD_BYTES
}

/// 叶装 `leaf_capacity` 条、往上每层扇出 `internal_fanout` 的树，`level_count` 层最多装几条。
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
/// `entry_count` 条条目要几层。
fn level_count_for(entry_count: u64, leaf_capacity: u64, internal_fanout: u64) -> u32 {
    let mut node_count = entry_count.div_ceil(leaf_capacity).max(1);
    let mut level_count = 1u32;
    while node_count > 1 {
        node_count = node_count.div_ceil(internal_fanout);
        level_count += 1;
    }
    level_count
}

/// 四棵树各自的（叶容量，内部扇出）。映射树按码 1 的 key 算（池里绝大多数条目是数据单元）。
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

fn main() {
    let mut emitter = Emitter::new();
    println!("E133 映射 key 各候选的格式代价");
    println!("判据写死在 research/prompts/e133-preregistration.md（写于本装置之前）；只报数、不判输赢");
    println!(
        "口径：节点={NODE_BYTES} 节点头={NODE_HEADER_BYTES} 映射节点头={MAP_NODE_HEADER_BYTES} 树表净字节={TREE_TABLE_PAYLOAD_BYTES} 子指针今天={CHILD_POINTER_BYTES_TODAY} 池={POOL_DATA_UNITS}"
    );
    for arm in ARMS.iter() {
        println!(
            "{}",
            emitter.emit_raw(&format!(
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
            ))
        );
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=headers arm={} code1={} code3={} code2={}/{}/{} packed_slot_extra={} livelist_identity_code1={}",
                arm.name,
                CODE1_HEADER_BYTES_TODAY + arm.code1_header_extra_bytes,
                CODE3_HEADER_BYTES_TODAY + arm.code23_header_extra_bytes,
                CODE2_HEADER_TIERS_TODAY[0] + arm.code23_header_extra_bytes,
                CODE2_HEADER_TIERS_TODAY[1] + arm.code23_header_extra_bytes,
                CODE2_HEADER_TIERS_TODAY[2] + arm.code23_header_extra_bytes,
                arm.packed_slot_extra_bytes,
                arm.code1_key_bytes
            ))
        );
        println!(
            "{}",
            emitter.emit_raw(&format!(
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
            ))
        );
        for (tree_name, leaf_capacity, internal_fanout) in trees(arm) {
            let thresholds: Vec<String> = (2u32..=5)
                .map(|level_count| format!("l{level_count}={}", first_entry_count_needing_levels(leaf_capacity, internal_fanout, level_count)))
                .collect();
            println!(
                "{}",
                emitter.emit_raw(&format!("name=thresholds arm={} tree={tree_name} {}", arm.name, thresholds.join(" ")))
            );
        }
        let map_leaf = map_leaf_fanout(arm.code1_key_bytes);
        let map_internal = map_internal_fanout(arm.code1_key_bytes, node_child_pointer_bytes(arm));
        let with_index_nodes = POOL_DATA_UNITS + POOL_DATA_UNITS.div_ceil(INDEX_FANOUT_FOR_NODE_COUNT);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=map_at_pool arm={} data_units_levels={} data_units_leaf_bytes={} with_index_levels={} with_index_leaf_bytes={}",
                arm.name,
                level_count_for(POOL_DATA_UNITS, map_leaf, map_internal),
                POOL_DATA_UNITS.div_ceil(map_leaf) * NODE_BYTES,
                level_count_for(with_index_nodes, map_leaf, map_internal),
                with_index_nodes.div_ceil(map_leaf) * NODE_BYTES
            ))
        );
    }
    let baseline = &ARMS[0];
    let mut positive_control_violations = 0u32;
    for arm in ARMS.iter() {
        // 阳性对照只管引用侧三棵树：key 比基准窄的臂，映射树本来就可以更宽。
        let reference_trees_not_wider = trees(arm)[..3].iter().zip(trees(baseline)[..3].iter()).all(|(candidate, base)| candidate.1 <= base.1 && candidate.2 <= base.2);
        if !reference_trees_not_wider {
            positive_control_violations += 1;
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!("name=positive_control arms={} violations={positive_control_violations}", ARMS.len()))
    );
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=cross_apparatus e109_map_leaf_at_33={} e128_child_pointer_at_16={}",
            map_leaf_fanout(E109_POSITION_AUTHORITY_KEY_BYTES),
            CHILD_POINTER_BYTES_TODAY + 16
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

    /// 判据 1：基准那一臂必须复现今天的已登记数。只让臂互相比，测不出「全部一起错」。
    #[test]
    fn baseline_arm_reproduces_todays_registered_widths() {
        let bing = arm_named("bing");
        assert_eq!(data_child_pointer_bytes(&bing), 59, "子指针 59：D22 已定项 7");
        assert_eq!(node_child_pointer_bytes(&bing), 59, "子指针 59：D22 已定项 7");
        assert_eq!(inode_internal_entry_bytes(&bing), 93, "inode 内部条目 93：D19 未定项 7");
        assert_eq!(ledger_internal_entry_bytes(&bing), 81, "记账内部条目 81：D19 未定项 7");
        assert_eq!(extent_leaf_entry_bytes(&bing), 83, "extent 叶记录 83：D19 未定项 7");
        assert_eq!(tree_table_entry_bytes(&bing), 121, "树表条目 121：D8 已定项 8");
        assert_eq!(root_record_bytes(&bing), 194, "根记录 194：D22 已定项 7");
    }

    /// 判据 2（跨装置闸）：两种指针都加 16 的两臂必须落到 E128 与 D19 未定项 7 甲那一格的同一组数。
    #[test]
    fn pointer_plus_sixteen_arms_match_e128_and_option_jia_segment_by_segment() {
        for name in ["k1", "k4"] {
            let arm = arm_named(name);
            assert_eq!(data_child_pointer_bytes(&arm), 75, "{name} 指向码 1 的子指针");
            assert_eq!(node_child_pointer_bytes(&arm), 75, "{name} 指向码 2 / 3 的子指针");
            assert_eq!(inode_internal_entry_bytes(&arm), 109, "{name} inode 内部条目");
            assert_eq!(ledger_internal_entry_bytes(&arm), 97, "{name} 记账内部条目");
            assert_eq!(extent_leaf_entry_bytes(&arm), 99, "{name} extent 叶记录");
            assert_eq!(tree_table_entry_bytes(&arm), 137, "{name} 树表条目");
            assert_eq!(root_record_bytes(&arm), 226, "{name} 根记录");
        }
    }

    /// 判据 3：映射叶扇出在 key 33 与 41 上复现 E109 / D27 已定项 6 的 267 与 D19 已定项 6 的 236。
    #[test]
    fn map_leaf_fanout_matches_e109_and_the_first_round_numbers() {
        assert_eq!(map_leaf_fanout(33), 267);
        assert_eq!(map_leaf_fanout(41), 236);
    }

    /// 判据 4：extent 叶扇出在 83 与 99 上复现 D19 未定项 7 那一格的 196 与 164。
    #[test]
    fn extent_leaf_fanout_matches_the_block_pointer_open_item_seven_numbers() {
        assert_eq!(node_fanout(83), 196);
        assert_eq!(node_fanout(99), 164);
    }

    /// 判据 5：树表每层条数在 121 与 137 上复现 C157 那个口径的 134 与 118。
    #[test]
    fn tree_table_per_level_matches_the_tree_table_payload_denominator() {
        assert_eq!(tree_table_entries_per_level(121), 134);
        assert_eq!(tree_table_entries_per_level(137), 118);
    }

    /// 本工程两处已登记的扇出必须由同一个节点头常量复现：D8 的 inode 树 175、D18 的记账树 201。
    #[test]
    fn registered_fanouts_come_out_of_the_same_node_header() {
        assert_eq!(node_fanout(93), 175);
        assert_eq!(node_fanout(81), 201);
    }

    /// 每条臂的两种指针宽、根记录、码 1 映射条目、码 1 / 码 3 头宽逐个钉死。期望值来自一份独立的 Python 算术。
    #[test]
    fn every_arm_width_is_pinned() {
        let expected: [(&str, u64, u64, u64, u64, u64, u64); 12] = [
            ("bing", 59, 59, 194, 61, 133, 131),
            ("k1", 75, 75, 226, 69, 133, 131),
            ("k1_full", 99, 75, 226, 69, 133, 131),
            ("k1_seq", 109, 75, 226, 79, 133, 131),
            ("k2", 83, 83, 250, 36, 141, 139),
            ("k2_instance", 87, 87, 250, 40, 141, 139),
            ("k3", 79, 79, 234, 48, 137, 135),
            ("k3_instance", 83, 83, 242, 52, 137, 135),
            ("class_reuse", 85, 83, 242, 54, 133, 135),
            ("k4", 75, 75, 226, 46, 143, 141),
            ("uuid16", 91, 91, 258, 44, 149, 147),
            ("slotgen12", 77, 77, 230, 40, 145, 143),
        ];
        for (name, data_pointer, node_pointer, root_record, map_entry_code1, code1_header, code3_header) in expected {
            let arm = arm_named(name);
            assert_eq!(data_child_pointer_bytes(&arm), data_pointer, "{name} 指向码 1 的子指针");
            assert_eq!(node_child_pointer_bytes(&arm), node_pointer, "{name} 指向码 2 / 3 的子指针");
            assert_eq!(root_record_bytes(&arm), root_record, "{name} 根记录");
            assert_eq!(map_entry_bytes(arm.code1_key_bytes), map_entry_code1, "{name} 码 1 映射条目");
            assert_eq!(CODE1_HEADER_BYTES_TODAY + arm.code1_header_extra_bytes, code1_header, "{name} 码 1 头");
            assert_eq!(CODE3_HEADER_BYTES_TODAY + arm.code23_header_extra_bytes, code3_header, "{name} 码 3 头");
        }
    }

    /// 每条臂的七个扇出逐个钉死。期望值来自一份独立的 Python 算术（只用登记的原始量，不读本文件的式子）。
    /// 节点头 64 与 76 在已登记的 196 / 164 / 175 / 201 / 267 / 236 上恰好同值（变异 M1 / M2，`.claude/rules/mutation-sampling.md` 第三类），
    /// 敏感取样点落在别的臂上；映射内部扇出与 extent 内部扇出此前没有断言（变异 M10 / M17）。
    #[test]
    fn every_arm_fanout_is_pinned_including_the_header_sensitive_cells() {
        let expected: [(&str, u64, u64, u64, u64, u64, u64, u64); 12] = [
            ("bing", 196, 196, 175, 201, 134, 267, 177),
            ("k1", 164, 164, 149, 168, 118, 236, 140),
            ("k1_full", 132, 164, 149, 168, 118, 236, 140),
            ("k1_seq", 122, 164, 149, 168, 118, 206, 129),
            ("k2", 152, 152, 139, 155, 112, 453, 179),
            ("k2_instance", 147, 147, 134, 149, 109, 407, 164),
            ("k3", 158, 158, 144, 161, 115, 339, 164),
            ("k3_instance", 152, 152, 139, 155, 112, 313, 152),
            ("class_reuse", 149, 152, 139, 155, 112, 302, 149),
            ("k4", 164, 164, 149, 168, 118, 354, 175),
            ("uuid16", 141, 141, 130, 144, 106, 370, 152),
            ("slotgen12", 161, 161, 147, 164, 117, 407, 183),
        ];
        for (name, extent_leaf, extent_internal, inode_internal, ledger_internal, tree_table, map_leaf, map_internal) in expected {
            let arm = arm_named(name);
            assert_eq!(node_fanout(extent_leaf_entry_bytes(&arm)), extent_leaf, "{name} extent 叶扇出");
            assert_eq!(node_fanout(extent_internal_entry_bytes(&arm)), extent_internal, "{name} extent 内部扇出");
            assert_eq!(node_fanout(inode_internal_entry_bytes(&arm)), inode_internal, "{name} inode 内部扇出");
            assert_eq!(node_fanout(ledger_internal_entry_bytes(&arm)), ledger_internal, "{name} 记账内部扇出");
            assert_eq!(tree_table_entries_per_level(tree_table_entry_bytes(&arm)), tree_table, "{name} 树表每层");
            assert_eq!(map_leaf_fanout(arm.code1_key_bytes), map_leaf, "{name} 映射叶扇出");
            assert_eq!(map_internal_fanout(arm.code1_key_bytes, node_child_pointer_bytes(&arm)), map_internal, "{name} 映射内部扇出");
        }
    }

    /// 映射树在 E109 那个池规模上的层数逐臂钉死（两档：只数数据单元 / 含索引节点）。期望值同样来自独立算术。
    #[test]
    fn every_arm_map_tree_level_count_at_the_e109_pool_is_pinned() {
        let expected: [(&str, u32, u32); 12] = [
            ("bing", 4, 4),
            ("k1", 4, 4),
            ("k1_full", 4, 4),
            ("k1_seq", 5, 5),
            ("k2", 4, 4),
            ("k2_instance", 4, 4),
            ("k3", 4, 4),
            ("k3_instance", 4, 4),
            ("class_reuse", 4, 4),
            ("k4", 4, 4),
            ("uuid16", 4, 4),
            ("slotgen12", 4, 4),
        ];
        let with_index_nodes = POOL_DATA_UNITS + POOL_DATA_UNITS.div_ceil(INDEX_FANOUT_FOR_NODE_COUNT);
        for (name, data_only, with_index) in expected {
            let arm = arm_named(name);
            let leaf = map_leaf_fanout(arm.code1_key_bytes);
            let internal = map_internal_fanout(arm.code1_key_bytes, node_child_pointer_bytes(&arm));
            assert_eq!(level_count_for(POOL_DATA_UNITS, leaf, internal), data_only, "{name} 只数数据单元");
            assert_eq!(level_count_for(with_index_nodes, leaf, internal), with_index, "{name} 含索引节点");
        }
    }

    /// 只有代理流水号那条臂的根记录多一条水位；实例内计数键的计数随实例从头发，不要水位。
    #[test]
    fn only_the_global_surrogate_arm_adds_a_watermark_to_the_root_record() {
        assert_eq!(root_record_bytes(&arm_named("k2")), 76 + 83 * 2 + 8);
        assert_eq!(root_record_bytes(&arm_named("k2_instance")), 76 + 87 * 2);
        assert_eq!(root_record_bytes(&arm_named("k3")), 76 + 79 * 2);
    }

    /// 按类复用写序键只在码 2 / 码 3 的头上加字节，码 1 头不动。
    #[test]
    fn class_reuse_arm_touches_only_code_two_and_three_headers() {
        let arm = arm_named("class_reuse");
        assert_eq!(CODE1_HEADER_BYTES_TODAY + arm.code1_header_extra_bytes, 133);
        assert_eq!(CODE3_HEADER_BYTES_TODAY + arm.code23_header_extra_bytes, 135);
        assert_eq!(data_child_pointer_bytes(&arm), 85);
        assert_eq!(node_child_pointer_bytes(&arm), 83);
    }

    /// 逻辑身份族给不出码 2 / 码 3 的定宽 key，其余臂都给得出。
    #[test]
    fn only_the_logical_identity_family_lacks_a_fixed_code_two_key() {
        for arm in ARMS.iter() {
            let logical_identity_family = matches!(arm.name, "bing" | "k1" | "k1_full" | "k1_seq");
            assert_eq!(arm.code23_key_bytes.is_none(), logical_identity_family, "{}", arm.name);
        }
    }

    /// inode 树的叶容量：码 3 头加到 147 字节仍是 233 条（32621 / 140），头增量在这几档上不改叶容量。
    #[test]
    fn inode_leaf_capacity_stays_233_for_every_arm() {
        for arm in ARMS.iter() {
            assert_eq!(inode_leaf_capacity(arm), 233, "{}", arm.name);
        }
    }

    /// 门槛的式子：两层的门槛 = 叶容量 + 1；三层 = 叶容量 × 内部扇出 + 1。
    #[test]
    fn level_thresholds_follow_leaf_capacity_times_internal_fanout() {
        assert_eq!(first_entry_count_needing_levels(196, 196, 2), 197);
        assert_eq!(first_entry_count_needing_levels(196, 196, 3), 196 * 196 + 1);
        assert_eq!(level_count_for(196, 196, 196), 1);
        assert_eq!(level_count_for(197, 196, 196), 2);
        assert_eq!(level_count_for(196 * 196 + 1, 196, 196), 3);
    }

    /// 判据 6（阳性对照）：每条候选的引用侧三棵树都不比基准宽——指针只增不减。
    #[test]
    fn no_candidate_widens_a_reference_tree_beyond_the_baseline() {
        let baseline = trees(&arm_named("bing"));
        for arm in ARMS.iter() {
            for (candidate, base) in trees(arm)[..3].iter().zip(baseline[..3].iter()) {
                assert!(candidate.1 <= base.1 && candidate.2 <= base.2, "{} 的 {} 树比基准还宽", arm.name, candidate.0);
            }
        }
    }

    /// 指向码 1 的指针最宽的那条臂（逻辑身份 + 写序键），extent 叶最早满，两层门槛必须比出生身份键早。
    #[test]
    fn the_widest_data_pointer_reaches_the_second_extent_level_first() {
        let widest_pointer_extent_tree = trees(&arm_named("k1_seq"))[0];
        let birth_identity_extent_tree = trees(&arm_named("k3"))[0];
        assert!(
            first_entry_count_needing_levels(widest_pointer_extent_tree.1, widest_pointer_extent_tree.2, 2)
                < first_entry_count_needing_levels(birth_identity_extent_tree.1, birth_identity_extent_tree.2, 2)
        );
    }
}
