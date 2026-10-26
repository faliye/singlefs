//! E97（记账与分配记录的条目编码） 第 5 次重跑（r5）：判据 10 按「每条根只装一代、只写不删」重算。
//!
//! 跑前登记 `research/prompts/e97-r5-prereg.md`，回答问题单 `research/prompts/c693-forks.md` 的 Q3（E97 那一半）。
//! 原装置 `e97_entry_encoding.rs`、它的变异表与复跑行一个字不动；这份是另起的独立手写模型，
//! 不 `use` 任何 `crates/` 下的包，也不 `use` 原装置（各自是一个 bin）。
//!
//! ## 被引条款（登记第二节整段抄的那几段，这里只摘与式子有关的句子）
//!
//! - D5（快照 / 空间记账机制） 已定项 2：「每条根的记账树只装这条根那一代的行，不留多代」；
//!   射程：「每次发布把这一代的行全写一遍，写的行数是实付值，不是上界」。
//! - D5 已定项 3 第 5 条：整棵记账树按 COW 挂在父指针下 ⇒ 一条插入脏一个完整节点（判据 10 的来历）。
//! - D8（核心索引结构） 已定项 2：节点 16 KiB。
//! - D16（发布语义） 已定项 5：`T_dirty` = 2 GiB；原判据 4 / 10 的门槛是它的 1%。
//!
//! ## 这一次唯一改的口径（登记读法第 1 条）
//!
//! | 量 | 旧口径（K 代、一写一删） | 新口径（一代、只写不删） |
//! |---|---|---|
//! | 判据 10 每叶组数 | `leaf_fanout / 25` | `leaf_fanout / 1` |
//! | 判据 4 条目字节 | 组数 × 2 × 条目宽 | 组数 × 1 × 条目宽 |
//! | 判据 4 反解 | 触碰条目 = 组数 × 2 | 触碰条目 = 组数 |
//! | 判据 1 / 5 树高的条目数 | 组数 × 25 | 组数 × 1 |
//!
//! 式子本身（每叶组数 = `max(leaf_fanout / K, 1)`，脏叶 = 组数向上整除每叶组数，脏字节 = 脏叶 × 16384）不改。
//! 十一项统计量表、四条 key 臂、节点头 58、四格 (t, d) × 两种读法照原装置。
//! 旧口径那一列由这份装置的旧口径路径现算，要与原产物 `e97-entry-encoding-2026-09-07.out`
//! 的 `name=acct_budget` 行逐字一致（登记 V1；单元测试 `old_caliber_reproduces_the_archived_budget_lines`）。
//!
//! ## 它答不了的
//!
//! 纯算术几何模型：没有 btree、没有 write buffer、没有 I/O；只数叶、不数内部节点；
//! 不答 inode 号水位的台阶行（它随根留着、不在「这一代的行」里，这里不建模）；
//! 不答今天十二项统计量表下的数（统计量表照 2026-09-03 那张十一项表）；不答挂钟。
//! **确定性模型**：同一个二进制跑 N 遍必然逐字节相同 ⇒ N 轮说明的是没有隐藏状态，
//! 不是统计上稳定；证据强度来自钉绝对值的断言与变异。

use e7_index_bench::Emitter;

/// D8 已定项 2：节点 16 KiB。
const NODE_BYTES: u64 = 16384;
/// D16 已定项 5：`T_dirty` = 2 GiB。
const DIRTY_BYTES_THRESHOLD: u64 = 2 * 1024 * 1024 * 1024;
/// 原判据 4 / 10 的门槛：`T_dirty` 的 1%（整除）。过界 = 严格大于。
const ACCOUNTING_WRITE_BUDGET_BYTES: u64 = DIRTY_BYTES_THRESHOLD / 100;
/// 2026-09-03 那天 D5 已定项 4 的统计量数（登记读法第 3 条：这一次不换表）。
const STATISTIC_COUNT: usize = 11;
/// 记账 value：完整值 8 字节（原装置同一个数）。
const ACCOUNTING_VALUE_BYTES: u64 = 8;
/// 原装置判据 4 / 10 那一段取的节点头：`NODE_HEADERS[0]` = 58（E73 那天的下界）。
const NODE_HEADER_BYTES_AT_E97: u64 = 58;
/// 原装置判据 1 / 5 扫的另两档 E73 节点头下界（58 之外的 67、76）。
const NODE_HEADERS_AT_E73: [u64; 3] = [NODE_HEADER_BYTES_AT_E97, 67, 76];
/// D18 已定项 7 补注的两笔加宽：写序 10，预留位 nonce 代号 12 + MAC 16。
const WRITE_ORDER_BYTES: u64 = 10;
const RESERVED_HEADER_BYTES: u64 = 12 + 16;
/// 原装置 `NODE_HEADERS_TODAY`：三档下界各加写序与预留位。头 96 那一档就是登记第八节的 S1。
const NODE_HEADERS_TODAY: [u64; 3] = [
    NODE_HEADERS_AT_E73[0] + WRITE_ORDER_BYTES + RESERVED_HEADER_BYTES,
    NODE_HEADERS_AT_E73[1] + WRITE_ORDER_BYTES + RESERVED_HEADER_BYTES,
    NODE_HEADERS_AT_E73[2] + WRITE_ORDER_BYTES + RESERVED_HEADER_BYTES,
];
/// 原装置判据 1 / 5 的两种子指针宽：E73 的参数 32、D22 已定项 7 的树表单元指针实宽 53。
const CHILD_POINTER_BYTES: [u64; 2] = [32, 53];
/// 原装置判据 1 / 5 树高那一组的 (t, d)：`accounting_entries(64, 8, 25, …)`。
const TREE_HEIGHT_TREE_COUNT: u64 = 64;
const TREE_HEIGHT_DEVICE_COUNT: u64 = 8;

/// 登记第八节 S2「实现几何」：本地常量，值抄自 `crates/singlefs-format/src/lib.rs`，不 `use`。
/// 节点头不含 key 区间的部分 86 + 29（`ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES` 的注释 `86 + 2 × 22 + 29`）。
const IMPLEMENTATION_NODE_HEADER_WITHOUT_KEY_RANGE_BYTES: u64 = 86 + 29;
/// `ACCOUNTING_KEY_BYTES = 22`（同一份文件）。
const IMPLEMENTATION_ACCOUNTING_KEY_BYTES: u64 = 22;
/// `ACCOUNTING_ENTRY_BYTES = 34`（key 22 + value 8 + seq 4，同一份文件）。
const IMPLEMENTATION_ACCOUNTING_ENTRY_BYTES: u64 = 34;
/// `ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES = 159`（同一份文件，含 key 区间）。只给单元测试回比用。
#[cfg(test)]
const IMPLEMENTATION_NODE_HEADER_WITH_KEY_RANGE_BYTES: u64 = 159;
/// `crates/singlefs-core/src/transaction.rs` `node_capacity_of_the_node_format` 的注释「记账树叶 477」。只给单元测试回比用。
#[cfg(test)]
const IMPLEMENTATION_LEAF_CAPACITY_IN_ENTRIES: u64 = 477;

/// 行数口径：旧的（K 代、每次发布一写一删）与新的（D5 已定项 2 现行定案）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowCountCaliber {
    TwentyFiveGenerationsWriteAndDelete,
    SingleGenerationWriteOnly,
}

impl RowCountCaliber {
    const BOTH: [RowCountCaliber; 2] = [
        RowCountCaliber::TwentyFiveGenerationsWriteAndDelete,
        RowCountCaliber::SingleGenerationWriteOnly,
    ];

    /// 记账树里同一组 (统计量, 树, 设备) 同时有几代的行。
    fn kept_generations(self) -> u64 {
        match self {
            RowCountCaliber::TwentyFiveGenerationsWriteAndDelete => 25, // 原装置调用处写死的 K
            RowCountCaliber::SingleGenerationWriteOnly => 1, // D5 已定项 2：每条根只装这一代
        }
    }

    /// 一次发布每组写几次条目。
    fn writes_per_group_per_publish(self) -> u64 {
        match self {
            RowCountCaliber::TwentyFiveGenerationsWriteAndDelete => 2, // 一写一删
            RowCountCaliber::SingleGenerationWriteOnly => 1, // 只写不删
        }
    }

    fn label(self) -> &'static str {
        match self {
            RowCountCaliber::TwentyFiveGenerationsWriteAndDelete => "old",
            RowCountCaliber::SingleGenerationWriteOnly => "new",
        }
    }
}

/// 窄读：只有第 8、9 项按树分；宽读：十一项都按树分（原装置与 E71 的两种读法）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reading {
    Narrow,
    Wide,
}

impl Reading {
    const BOTH: [Reading; 2] = [Reading::Narrow, Reading::Wide];

    fn label(self) -> &'static str {
        match self {
            Reading::Narrow => "narrow",
            Reading::Wide => "wide",
        }
    }
}

/// 统计量表的一行。次序与 2026-09-03 那天 D5 已定项 4 那张表逐行对应（原装置 `STATISTICS_TABLE`）。
struct Statistic {
    per_device: bool,
    per_tree_narrow: bool,
}
const STATISTICS_TABLE: [Statistic; STATISTIC_COUNT] = [
    Statistic { per_device: true,  per_tree_narrow: false }, // 1 已分配字节
    Statistic { per_device: true,  per_tree_narrow: false }, // 2 空闲字节
    Statistic { per_device: true,  per_tree_narrow: false }, // 3 不可回收字节
    Statistic { per_device: false, per_tree_narrow: false }, // 4 待删占用
    Statistic { per_device: true,  per_tree_narrow: false }, // 5 defer 待释放
    Statistic { per_device: false, per_tree_narrow: false }, // 6 已承诺预留
    Statistic { per_device: false, per_tree_narrow: false }, // 7 扩展点配额已用
    Statistic { per_device: false, per_tree_narrow: true  }, // 8 每树独占
    Statistic { per_device: false, per_tree_narrow: true  }, // 9 每树共享
    Statistic { per_device: false, per_tree_narrow: false }, // 10 碎片度 runs
    Statistic { per_device: true,  per_tree_narrow: false }, // 11 全空聚簇段数
];

/// 一条记账 key 臂：原装置的名字与四段宽（统计量标签、树 ID、设备、代）。登记 V1 按名字对行。
struct KeyArm {
    name: &'static str,
    statistic_tag_bytes: u64,
    tree_identifier_bytes: u64,
    device_bytes: u64,
    generation_bytes: u64,
}

impl KeyArm {
    fn key_bytes(&self) -> u64 {
        self.statistic_tag_bytes + self.tree_identifier_bytes + self.device_bytes + self.generation_bytes
    }

    /// 叶条目 = key + value 8（原装置判据 4 / 10 那一段的写法）。
    fn leaf_entry_bytes(&self) -> u64 {
        self.key_bytes() + ACCOUNTING_VALUE_BYTES
    }

    /// 判据 10 主点的叶几何：节点头 58、区间 2 × key、条目 key + 8。
    fn criterion10_geometry(&self) -> LeafGeometry {
        LeafGeometry {
            label: self.name,
            header_bytes: NODE_HEADER_BYTES_AT_E97,
            key_bytes: self.key_bytes(),
            entry_bytes: self.leaf_entry_bytes(),
        }
    }
}

const KEY_ARMS: [KeyArm; 4] = [
    KeyArm { name: "acct_e73_22B", statistic_tag_bytes: 2, tree_identifier_bytes: 8, device_bytes: 4, generation_bytes: 8 },
    KeyArm { name: "acct_wide_19B", statistic_tag_bytes: 2, tree_identifier_bytes: 8, device_bytes: 1, generation_bytes: 8 },
    KeyArm { name: "acct_tight_14B", statistic_tag_bytes: 1, tree_identifier_bytes: 4, device_bytes: 1, generation_bytes: 8 },
    KeyArm { name: "acct_windowgen_7B", statistic_tag_bytes: 1, tree_identifier_bytes: 4, device_bytes: 1, generation_bytes: 1 },
];
/// 判决臂（登记读法第 6 条）：19 字节 key 那条臂。臂名里的 wide 是 key 宽，不是宽读。
const JUDGED_KEY_ARM_INDEX: usize = 1;

/// 判据 10 的叶几何：节点头（不含 key 区间）、key 宽（定 key 区间 2 × key）、叶条目宽。
struct LeafGeometry {
    label: &'static str,
    header_bytes: u64,
    key_bytes: u64,
    entry_bytes: u64,
}

impl LeafGeometry {
    fn leaf_fanout(&self) -> u64 {
        let key_range_field_bytes = 2 * self.key_bytes;
        node_fanout(self.header_bytes, key_range_field_bytes, self.entry_bytes)
    }
}

/// 登记第八节 S1：19B 臂，头取今天那一档 96。
fn today_header_geometry() -> LeafGeometry {
    LeafGeometry {
        label: "S1_today_header_19B",
        header_bytes: NODE_HEADERS_TODAY[0],
        key_bytes: KEY_ARMS[JUDGED_KEY_ARM_INDEX].key_bytes(),
        entry_bytes: KEY_ARMS[JUDGED_KEY_ARM_INDEX].leaf_entry_bytes(),
    }
}

/// 登记第八节 S2：实现几何（头 115、区间 2 × 22、条目 34）。
fn implementation_geometry() -> LeafGeometry {
    LeafGeometry {
        label: "S2_implementation_22B_entry34",
        header_bytes: IMPLEMENTATION_NODE_HEADER_WITHOUT_KEY_RANGE_BYTES,
        key_bytes: IMPLEMENTATION_ACCOUNTING_KEY_BYTES,
        entry_bytes: IMPLEMENTATION_ACCOUNTING_ENTRY_BYTES,
    }
}

/// 扇出 = (节点 − 节点头 − 区间字段) / 条目宽（原判据 1 的式子）。几何非法（装不下头、条目宽 0）是装置写错，断言。
fn node_fanout(header_bytes: u64, range_field_bytes: u64, entry_bytes: u64) -> u64 {
    assert!(entry_bytes > 0, "条目宽 0 不是一个几何");
    let payload_bytes = NODE_BYTES
        .checked_sub(header_bytes + range_field_bytes)
        .expect("节点头加区间字段超过 16 KiB 不是一个几何");
    payload_bytes / entry_bytes
}

/// 原取样的四格 (树数, 设备数)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Grid {
    tree_count: u64,
    device_count: u64,
}

const GRIDS: [Grid; 4] = [
    Grid { tree_count: 64, device_count: 8 },
    Grid { tree_count: 1024, device_count: 64 },
    Grid { tree_count: 8192, device_count: 8 },
    Grid { tree_count: 8192, device_count: 64 },
];

/// 问题单 Q3 翻面观测逐字点名的两格（都看宽读）：t=1024 / d=64 与 t=8192 / d=8。
const JUDGED_GRID_TREES_1024_DEVICES_64: Grid = Grid { tree_count: 1024, device_count: 64 };
const JUDGED_GRID_TREES_8192_DEVICES_8: Grid = Grid { tree_count: 8192, device_count: 8 };

impl Grid {
    fn label(self) -> String {
        format!("{}x{}", self.tree_count, self.device_count)
    }
}

/// 一次发布触碰的 (统计量, 树, 设备) 组数 = 原装置 `accounting_entries(t, d, 1, wide)`。
fn accounting_groups(tree_count: u64, device_count: u64, reading: Reading) -> u64 {
    STATISTICS_TABLE
        .iter()
        .map(|statistic| {
            let is_split_by_tree = match reading {
                Reading::Wide => true,
                Reading::Narrow => statistic.per_tree_narrow,
            };
            let tree_factor = if is_split_by_tree { tree_count } else { 1 };
            let device_factor = if statistic.per_device { device_count } else { 1 };
            tree_factor * device_factor
        })
        .sum()
}

/// 判据 10 的一格：每叶装几组、脏几片叶、脏多少字节。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirtyLeaves {
    groups_per_leaf: u64,
    leaves: u64,
    dirty_bytes: u64,
}

/// **判据 10**：一次发布脏掉的叶字节。同组的 K 代在 key 空间里连续 ⇒ 每叶装 `leaf_fanout / K` 组，
/// 每组每次发布被触碰一次 ⇒ 组所在的叶全脏（原装置 `dirty_bytes_per_publish` 的式子，不改）。
fn dirty_leaves_per_publish(groups: u64, kept_generations: u64, leaf_fanout: u64) -> DirtyLeaves {
    assert!(leaf_fanout > 0 && kept_generations > 0, "扇出与代数都至少是 1");
    let groups_per_leaf = (leaf_fanout / kept_generations).max(1);
    let leaves = groups.div_ceil(groups_per_leaf);
    DirtyLeaves { groups_per_leaf, leaves, dirty_bytes: leaves * NODE_BYTES }
}

/// 过界 = 严格大于门槛（原判据 4 / 10）。门槛作参数：判别力自证要把它挪到两格之间。
fn exceeds_budget(bytes: u64, budget_bytes: u64) -> bool {
    bytes > budget_bytes
}

/// 占 `T_dirty` 1% 预算的百分比，整除（原单元测试 `占预算 53%` 那一处的写法）。
fn percent_of_budget(bytes: u64) -> u64 {
    bytes * 100 / ACCOUNTING_WRITE_BUDGET_BYTES
}

/// 千分比，整除。
fn permille_of_budget(bytes: u64) -> u64 {
    bytes * 1000 / ACCOUNTING_WRITE_BUDGET_BYTES
}

/// **判据 4（附带）**：一次发布的记账条目字节 = 组数 × 每组写几次 × 条目宽。
fn entry_bytes_per_publish(groups: u64, leaf_entry_bytes: u64, caliber: RowCountCaliber) -> u64 {
    groups * caliber.writes_per_group_per_publish() * leaf_entry_bytes
}

/// **判据 4 的反解（附带）**：条目最宽能到几字节还不撑破门槛。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeyCeiling {
    touched_entries: u64,
    maximum_entry_bytes: u64,
    maximum_key_bytes: u64,
    is_feasible: bool,
}

fn key_ceiling(groups: u64, caliber: RowCountCaliber) -> KeyCeiling {
    let touched_entries = groups * caliber.writes_per_group_per_publish();
    let maximum_entry_bytes = ACCOUNTING_WRITE_BUDGET_BYTES / touched_entries;
    KeyCeiling {
        touched_entries,
        maximum_entry_bytes,
        maximum_key_bytes: maximum_entry_bytes.saturating_sub(ACCOUNTING_VALUE_BYTES),
        is_feasible: maximum_entry_bytes > ACCOUNTING_VALUE_BYTES,
    }
}

/// 树高（含叶层）：叶层按叶扇出装，上面各层按内部扇出装（原装置 `tree_height`）。
fn tree_height(entry_count: u64, leaf_fanout: u64, inner_fanout: u64) -> Option<u64> {
    if leaf_fanout == 0 || inner_fanout < 2 {
        return None;
    }
    let mut height = 1u64;
    let mut covered_entries = u128::from(leaf_fanout);
    while covered_entries < u128::from(entry_count) {
        covered_entries = covered_entries.saturating_mul(u128::from(inner_fanout));
        height += 1;
        if height > 64 {
            return None; // 不收敛报 None，不挂死
        }
    }
    Some(height)
}

/// 一格（几何 × (t, d) × 读法）在两种口径下的判据 10。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellEvaluation {
    grid: Grid,
    reading: Reading,
    leaf_fanout: u64,
    groups: u64,
    old: DirtyLeaves,
    new: DirtyLeaves,
}

fn evaluate_cell(geometry: &LeafGeometry, grid: Grid, reading: Reading) -> CellEvaluation {
    let leaf_fanout = geometry.leaf_fanout();
    let groups = accounting_groups(grid.tree_count, grid.device_count, reading);
    let dirty_leaves_of = |caliber: RowCountCaliber| {
        dirty_leaves_per_publish(groups, caliber.kept_generations(), leaf_fanout)
    };
    CellEvaluation {
        grid,
        reading,
        leaf_fanout,
        groups,
        old: dirty_leaves_of(RowCountCaliber::TwentyFiveGenerationsWriteAndDelete),
        new: dirty_leaves_of(RowCountCaliber::SingleGenerationWriteOnly),
    }
}

/// 一个几何在四格 × 两种读法上的八格，次序照原装置（格在外、读法在里）。
fn evaluate_geometry(geometry: &LeafGeometry) -> Vec<CellEvaluation> {
    GRIDS
        .iter()
        .flat_map(|grid| Reading::BOTH.iter().map(move |reading| evaluate_cell(geometry, *grid, *reading)))
        .collect()
}

fn over_bit(bytes: u64) -> u8 {
    u8::from(exceeds_budget(bytes, ACCOUNTING_WRITE_BUDGET_BYTES))
}

/// 一组格里过界的格（按给定口径的脏字节）：格数与格名单（没有写 none）。
fn over_cells(cells: &[CellEvaluation], pick: fn(&CellEvaluation) -> u64) -> (usize, String) {
    let over: Vec<String> = cells
        .iter()
        .filter(|cell| exceeds_budget(pick(cell), ACCOUNTING_WRITE_BUDGET_BYTES))
        .map(|cell| cell.grid.label())
        .collect();
    let names = if over.is_empty() { "none".to_string() } else { over.join(",") };
    (over.len(), names)
}

/// 一组格里脏字节最大的那几格（并列的都列出来）：最大值与格名单。
fn tightest_cells(cells: &[CellEvaluation], pick: fn(&CellEvaluation) -> u64) -> (u64, String) {
    let maximum = cells.iter().map(pick).max().expect("至少一格");
    let names: Vec<String> = cells
        .iter()
        .filter(|cell| pick(cell) == maximum)
        .map(|cell| cell.grid.label())
        .collect();
    (maximum, names.join(","))
}

fn old_dirty_bytes(cell: &CellEvaluation) -> u64 {
    cell.old.dirty_bytes
}

fn new_dirty_bytes(cell: &CellEvaluation) -> u64 {
    cell.new.dirty_bytes
}

fn dirty_leaves_in(cell: &CellEvaluation, caliber: RowCountCaliber) -> DirtyLeaves {
    match caliber {
        RowCountCaliber::TwentyFiveGenerationsWriteAndDelete => cell.old,
        RowCountCaliber::SingleGenerationWriteOnly => cell.new,
    }
}

/// 原产物 `name=acct_budget` 行 `name=` 之后的那一截，字段与次序照原装置；旧口径这一截要与原产物逐字一致（登记 V1）。
fn budget_line_body(arm: &KeyArm, cell: &CellEvaluation, caliber: RowCountCaliber) -> String {
    let entry_bytes = entry_bytes_per_publish(cell.groups, arm.leaf_entry_bytes(), caliber);
    let node_bytes = dirty_leaves_in(cell, caliber).dirty_bytes;
    format!(
        "arm={} trees={} devs={} reading={} entry={} leaf_fanout={} groups={} entry_bytes={entry_bytes} \
         over_by_entry={} node_bytes={node_bytes} over_by_node={}",
        arm.name,
        cell.grid.tree_count,
        cell.grid.device_count,
        cell.reading.label(),
        arm.leaf_entry_bytes(),
        cell.leaf_fanout,
        cell.groups,
        over_bit(entry_bytes),
        over_bit(node_bytes),
    )
}

/// 原产物 `name=acct_key_ceiling` 行 `name=` 之后的那一截。
fn key_ceiling_line_body(grid: Grid, reading: Reading, caliber: RowCountCaliber) -> String {
    let ceiling = key_ceiling(accounting_groups(grid.tree_count, grid.device_count, reading), caliber);
    format!(
        "trees={} devs={} reading={} touched_entries={} budget={ACCOUNTING_WRITE_BUDGET_BYTES} \
         max_entry_bytes={} max_key_bytes={} feasible={}",
        grid.tree_count,
        grid.device_count,
        reading.label(),
        ceiling.touched_entries,
        ceiling.maximum_entry_bytes,
        ceiling.maximum_key_bytes,
        u8::from(ceiling.is_feasible),
    )
}

/// 原产物 `name=acct_geom` 行 `name=` 之后的那一截：条目数 = 组数 × 这一口径的代数。
fn tree_height_line_body(arm: &KeyArm, header_bytes: u64, child_pointer_bytes: u64, caliber: RowCountCaliber) -> String {
    let key_byte_count = arm.key_bytes();
    let inner_entry_bytes = key_byte_count + child_pointer_bytes;
    let inner_key_range_field_bytes = 2 * arm.key_bytes();
    let leaf_fanout = node_fanout(header_bytes, inner_key_range_field_bytes, arm.leaf_entry_bytes());
    let inner_fanout = node_fanout(header_bytes, inner_key_range_field_bytes, inner_entry_bytes);
    let entries_of = |reading: Reading| {
        accounting_groups(TREE_HEIGHT_TREE_COUNT, TREE_HEIGHT_DEVICE_COUNT, reading) * caliber.kept_generations()
    };
    let height_label = |entry_count: u64| {
        tree_height(entry_count, leaf_fanout, inner_fanout).map_or_else(|| "NA".to_string(), |height| height.to_string())
    };
    let (entries_narrow, entries_wide) = (entries_of(Reading::Narrow), entries_of(Reading::Wide));
    format!(
        "arm={} key_bytes={key_byte_count} leaf_entry={} inner_entry={inner_entry_bytes} header={header_bytes} \
         child_ptr={child_pointer_bytes} leaf_fanout={leaf_fanout} inner_fanout={inner_fanout} \
         entries_narrow={entries_narrow} height_narrow={} entries_wide={entries_wide} height_wide={}",
        arm.name,
        arm.leaf_entry_bytes(),
        height_label(entries_narrow),
        height_label(entries_wide),
    )
}

/// 登记 Q-E97-6 的逐格并列行（`name=` 之后那一截）。
fn side_by_side_line_body(geometry: &LeafGeometry, cell: &CellEvaluation) -> String {
    format!(
        "geometry={} header={} key_bytes={} entry={} leaf_fanout={} trees={} devs={} reading={} groups={} \
         old_groups_per_leaf={} old_leaves={} old_node_bytes={} old_over={} old_percent={} old_permille={} \
         new_groups_per_leaf={} new_leaves={} new_node_bytes={} new_over={} new_percent={} new_permille={}",
        geometry.label,
        geometry.header_bytes,
        geometry.key_bytes,
        geometry.entry_bytes,
        cell.leaf_fanout,
        cell.grid.tree_count,
        cell.grid.device_count,
        cell.reading.label(),
        cell.groups,
        cell.old.groups_per_leaf,
        cell.old.leaves,
        cell.old.dirty_bytes,
        over_bit(cell.old.dirty_bytes),
        percent_of_budget(cell.old.dirty_bytes),
        permille_of_budget(cell.old.dirty_bytes),
        cell.new.groups_per_leaf,
        cell.new.leaves,
        cell.new.dirty_bytes,
        over_bit(cell.new.dirty_bytes),
        percent_of_budget(cell.new.dirty_bytes),
        permille_of_budget(cell.new.dirty_bytes),
    )
}

/// 阳性对照甲（每条臂、每一格）：K = 25 的脏字节严格大于 K = 1 的，且 K = 1 的恰好等于 ⌈组数 / 扇出⌉ × 16384。
/// 返回（格数、严格大于的格数、恰好等于的格数）。
fn generations_in_formula_control(cells: &[CellEvaluation]) -> (usize, usize, usize) {
    let strictly_greater = cells.iter().filter(|cell| cell.old.dirty_bytes > cell.new.dirty_bytes).count();
    let equals_ceiling = cells
        .iter()
        .filter(|cell| cell.new.dirty_bytes == cell.groups.div_ceil(cell.leaf_fanout) * NODE_BYTES)
        .count();
    (cells.len(), strictly_greater, equals_ceiling)
}

/// 阳性对照丙：宽读 1024 × 64 一格，设备维去掉（设备数当 1）之后新口径脏字节严格下降。返回（带设备维、去掉之后）。
fn device_dimension_control(geometry: &LeafGeometry) -> (u64, u64) {
    let with_device = evaluate_cell(geometry, JUDGED_GRID_TREES_1024_DEVICES_64, Reading::Wide).new.dirty_bytes;
    let groups_without_device = accounting_groups(JUDGED_GRID_TREES_1024_DEVICES_64.tree_count, 1, Reading::Wide);
    let without_device = dirty_leaves_per_publish(
        groups_without_device,
        RowCountCaliber::SingleGenerationWriteOnly.kept_generations(),
        geometry.leaf_fanout(),
    )
    .dirty_bytes;
    (with_device, without_device)
}

fn cell_at(cells: &[CellEvaluation], grid: Grid, reading: Reading) -> CellEvaluation {
    *cells
        .iter()
        .find(|cell| cell.grid == grid && cell.reading == reading)
        .expect("一个几何的八格里每个 (格, 读法) 恰好一格")
}

fn cells_of_reading(cells: &[CellEvaluation], reading: Reading) -> Vec<CellEvaluation> {
    cells.iter().filter(|cell| cell.reading == reading).copied().collect()
}

/// 判决行 Q-E97-1 到 Q-E97-5（登记第六节），各报各的，不合成。`judged_cells` 是判决臂的八格。
fn question_line_bodies(arm_name: &str, judged_cells: &[CellEvaluation]) -> Vec<String> {
    let mut bodies = Vec::new();
    for (question, grid) in [("Q-E97-1", JUDGED_GRID_TREES_1024_DEVICES_64), ("Q-E97-2", JUDGED_GRID_TREES_8192_DEVICES_8)] {
        let cell = cell_at(judged_cells, grid, Reading::Wide);
        bodies.push(format!(
            "name=judged_cell question={question} arm={arm_name} trees={} devs={} reading=wide old_node_bytes={} old_over={} \
             new_node_bytes={} new_over={} new_percent={} new_permille={}",
            grid.tree_count,
            grid.device_count,
            cell.old.dirty_bytes,
            over_bit(cell.old.dirty_bytes),
            cell.new.dirty_bytes,
            over_bit(cell.new.dirty_bytes),
            percent_of_budget(cell.new.dirty_bytes),
            permille_of_budget(cell.new.dirty_bytes),
        ));
    }
    let wide_cells = cells_of_reading(judged_cells, Reading::Wide);
    let (old_wide_over_count, old_wide_over_grids) = over_cells(&wide_cells, old_dirty_bytes);
    let (new_wide_over_count, new_wide_over_grids) = over_cells(&wide_cells, new_dirty_bytes);
    bodies.push(format!(
        "name=wide_over_cells question=Q-E97-3 arm={arm_name} old_over_cells={old_wide_over_count} old_over_grids={old_wide_over_grids} \
         new_over_cells={new_wide_over_count} new_over_grids={new_wide_over_grids}"
    ));
    let narrow_cells = cells_of_reading(judged_cells, Reading::Narrow);
    let (old_tightest_bytes, old_tightest_grids) = tightest_cells(&narrow_cells, old_dirty_bytes);
    let (new_tightest_bytes, new_tightest_grids) = tightest_cells(&narrow_cells, new_dirty_bytes);
    bodies.push(format!(
        "name=narrow_tightest question=Q-E97-4 arm={arm_name} old_node_bytes={old_tightest_bytes} old_grids={old_tightest_grids} \
         old_percent={} old_permille={} old_over={} new_node_bytes={new_tightest_bytes} new_grids={new_tightest_grids} \
         new_percent={} new_permille={} new_over={}",
        percent_of_budget(old_tightest_bytes),
        permille_of_budget(old_tightest_bytes),
        over_bit(old_tightest_bytes),
        percent_of_budget(new_tightest_bytes),
        permille_of_budget(new_tightest_bytes),
        over_bit(new_tightest_bytes),
    ));
    let (old_narrow_over_count, old_narrow_over_grids) = over_cells(&narrow_cells, old_dirty_bytes);
    let (new_narrow_over_count, new_narrow_over_grids) = over_cells(&narrow_cells, new_dirty_bytes);
    bodies.push(format!(
        "name=narrow_over_cells question=Q-E97-5 arm={arm_name} old_over_cells={old_narrow_over_count} old_over_grids={old_narrow_over_grids} \
         new_over_cells={new_narrow_over_count} new_over_grids={new_narrow_over_grids}"
    ));
    bodies
}

/// 几何敏感性（登记第八节）：一个几何上两格判决格的新口径判定与窄读最紧格占比。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeometrySensitivity {
    leaf_fanout: u64,
    is_wide_1024_by_64_over: bool,
    is_wide_8192_by_8_over: bool,
    narrow_tightest_percent: u64,
    narrow_tightest_permille: u64,
}

fn geometry_sensitivity(geometry: &LeafGeometry) -> GeometrySensitivity {
    let cells = evaluate_geometry(geometry);
    let new_over_at = |grid: Grid| {
        exceeds_budget(cell_at(&cells, grid, Reading::Wide).new.dirty_bytes, ACCOUNTING_WRITE_BUDGET_BYTES)
    };
    let (narrow_tightest_bytes, _) = tightest_cells(&cells_of_reading(&cells, Reading::Narrow), new_dirty_bytes);
    GeometrySensitivity {
        leaf_fanout: geometry.leaf_fanout(),
        is_wide_1024_by_64_over: new_over_at(JUDGED_GRID_TREES_1024_DEVICES_64),
        is_wide_8192_by_8_over: new_over_at(JUDGED_GRID_TREES_8192_DEVICES_8),
        narrow_tightest_percent: percent_of_budget(narrow_tightest_bytes),
        narrow_tightest_permille: permille_of_budget(narrow_tightest_bytes),
    }
}

/// 判据 10 敏感性要扫的全部几何：四条 key 臂的主点（判决臂在其中）加 S1、S2。
fn sensitivity_geometries() -> Vec<LeafGeometry> {
    let mut geometries: Vec<LeafGeometry> = KEY_ARMS.iter().map(KeyArm::criterion10_geometry).collect();
    geometries.push(today_header_geometry());
    geometries.push(implementation_geometry());
    geometries
}

fn emit_line(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    emit_line(
        &mut emitter,
        &format!(
            "name=config node_bytes={NODE_BYTES} t_dirty={DIRTY_BYTES_THRESHOLD} budget={ACCOUNTING_WRITE_BUDGET_BYTES} \
             stats={STATISTIC_COUNT} old_kept_generations={} old_writes_per_group={} new_kept_generations={} \
             new_writes_per_group={} criterion10_header={NODE_HEADER_BYTES_AT_E97} threads=1",
            RowCountCaliber::TwentyFiveGenerationsWriteAndDelete.kept_generations(),
            RowCountCaliber::TwentyFiveGenerationsWriteAndDelete.writes_per_group_per_publish(),
            RowCountCaliber::SingleGenerationWriteOnly.kept_generations(),
            RowCountCaliber::SingleGenerationWriteOnly.writes_per_group_per_publish(),
        ),
    );
    let cells_by_arm: Vec<Vec<CellEvaluation>> =
        KEY_ARMS.iter().map(|arm| evaluate_geometry(&arm.criterion10_geometry())).collect();

    // 判据 10 与判据 4（附带）：原产物同形的行，两种口径各 32 行；旧口径那 32 行登记 V1 拿去与原产物逐字比
    for caliber in RowCountCaliber::BOTH {
        for (arm, cells) in KEY_ARMS.iter().zip(&cells_by_arm) {
            for cell in cells {
                let body = budget_line_body(arm, cell, caliber);
                emit_line(&mut emitter, &format!("name=acct_budget_{}_caliber {body}", caliber.label()));
            }
        }
    }
    // Q-E97-6：32 格逐格新旧并列
    for (arm, cells) in KEY_ARMS.iter().zip(&cells_by_arm) {
        for cell in cells {
            let body = side_by_side_line_body(&arm.criterion10_geometry(), cell);
            emit_line(&mut emitter, &format!("name=cell_side_by_side {body}"));
        }
    }
    // 判据 4 的反解（附带）
    for caliber in RowCountCaliber::BOTH {
        for grid in GRIDS {
            for reading in Reading::BOTH {
                let body = key_ceiling_line_body(grid, reading, caliber);
                emit_line(&mut emitter, &format!("name=acct_key_ceiling_{}_caliber {body}", caliber.label()));
            }
        }
    }
    // 判据 1 / 5 的树高（附带）：四臂 × 六档头 × 两种子指针，条目数按这一口径的代数
    for caliber in RowCountCaliber::BOTH {
        for arm in &KEY_ARMS {
            for header_bytes in NODE_HEADERS_AT_E73.iter().chain(NODE_HEADERS_TODAY.iter()) {
                for child_pointer_bytes in CHILD_POINTER_BYTES {
                    let body = tree_height_line_body(arm, *header_bytes, child_pointer_bytes, caliber);
                    emit_line(&mut emitter, &format!("name=acct_geom_{}_caliber {body}", caliber.label()));
                }
            }
        }
    }
    // 几何敏感性 S1、S2：两种读法 × 四格
    for geometry in [today_header_geometry(), implementation_geometry()] {
        for cell in evaluate_geometry(&geometry) {
            let body = side_by_side_line_body(&geometry, &cell);
            emit_line(&mut emitter, &format!("name=sensitivity_cell {body}"));
        }
    }
    // 判决行
    let judged_arm = &KEY_ARMS[JUDGED_KEY_ARM_INDEX];
    for body in question_line_bodies(judged_arm.name, &cells_by_arm[JUDGED_KEY_ARM_INDEX]) {
        emit_line(&mut emitter, &body);
    }
    // 几何敏感性一览与依赖计数（登记第十节 FC5）
    let judged_sensitivity = geometry_sensitivity(&judged_arm.criterion10_geometry());
    let mut judged_verdict_differing_geometries = 0usize;
    let mut narrow_percent_differing_geometries = 0usize;
    for geometry in sensitivity_geometries() {
        let sensitivity = geometry_sensitivity(&geometry);
        let judged_verdicts_differ = sensitivity.is_wide_1024_by_64_over != judged_sensitivity.is_wide_1024_by_64_over
            || sensitivity.is_wide_8192_by_8_over != judged_sensitivity.is_wide_8192_by_8_over;
        judged_verdict_differing_geometries += usize::from(judged_verdicts_differ);
        narrow_percent_differing_geometries +=
            usize::from(sensitivity.narrow_tightest_percent != judged_sensitivity.narrow_tightest_percent);
        emit_line(
            &mut emitter,
            &format!(
                "name=geometry_sensitivity geometry={} leaf_fanout={} wide_1024x64_new_over={} wide_8192x8_new_over={} \
                 narrow_tightest_new_percent={} narrow_tightest_new_permille={}",
                geometry.label,
                sensitivity.leaf_fanout,
                u8::from(sensitivity.is_wide_1024_by_64_over),
                u8::from(sensitivity.is_wide_8192_by_8_over),
                sensitivity.narrow_tightest_percent,
                sensitivity.narrow_tightest_permille,
            ),
        );
    }
    emit_line(
        &mut emitter,
        &format!(
            "name=geometry_dependence geometries={} judged_cell_verdict_differing_geometries={judged_verdict_differing_geometries} \
             narrow_tightest_percent_differing_geometries={narrow_percent_differing_geometries}",
            sensitivity_geometries().len(),
        ),
    );
    // 阳性对照甲与丙，每条臂都跑
    let mut is_generations_control_passing = true;
    let mut is_device_control_passing = true;
    for (arm, cells) in KEY_ARMS.iter().zip(&cells_by_arm) {
        let (cell_count, strictly_greater, equals_ceiling) = generations_in_formula_control(cells);
        is_generations_control_passing &= strictly_greater == cell_count && equals_ceiling == cell_count;
        emit_line(
            &mut emitter,
            &format!(
                "name=control_generations_in_formula arm={} cells={cell_count} old_strictly_greater={strictly_greater} \
                 new_equals_ceiling_times_node={equals_ceiling}",
                arm.name
            ),
        );
    }
    for arm in &KEY_ARMS {
        let (with_device, without_device) = device_dimension_control(&arm.criterion10_geometry());
        is_device_control_passing &= without_device < with_device;
        emit_line(
            &mut emitter,
            &format!(
                "name=control_device_dimension arm={} trees=1024 devs=64 reading=wide new_with_device={with_device} \
                 new_without_device={without_device} strictly_lower={}",
                arm.name,
                u8::from(without_device < with_device)
            ),
        );
    }
    emit_line(
        &mut emitter,
        &format!(
            "name=verdict control_generations_in_formula_ok={is_generations_control_passing} \
             control_device_dimension_ok={is_device_control_passing}"
        ),
    );
    println!("{}", emitter.finish());
    assert!(is_generations_control_passing, "阳性对照甲没过：登记 V3，整轮作废");
    assert!(is_device_control_passing, "阳性对照丙没过：登记 V3，整轮作废");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn judged_arm_cell(grid: Grid, reading: Reading) -> CellEvaluation {
        evaluate_cell(&KEY_ARMS[JUDGED_KEY_ARM_INDEX].criterion10_geometry(), grid, reading)
    }

    #[test]
    fn budget_is_one_percent_of_two_gibibytes_and_a_node_is_sixteen_kibibytes() {
        assert_eq!(ACCOUNTING_WRITE_BUDGET_BYTES, 21_474_836, "登记甲 1：T_dirty 2 GiB 的 1% 整除");
        assert_eq!(NODE_BYTES, 16 * 1024, "登记甲 3：每片脏叶 16384 字节");
    }

    #[test]
    fn new_caliber_keeps_one_generation_and_writes_each_group_once() {
        assert_eq!(RowCountCaliber::SingleGenerationWriteOnly.kept_generations(), 1, "登记甲 2");
        assert_eq!(RowCountCaliber::SingleGenerationWriteOnly.writes_per_group_per_publish(), 1, "登记甲 2");
        assert_eq!(RowCountCaliber::TwentyFiveGenerationsWriteAndDelete.kept_generations(), 25, "原装置调用处的 K");
        assert_eq!(RowCountCaliber::TwentyFiveGenerationsWriteAndDelete.writes_per_group_per_publish(), 2, "一写一删");
    }

    #[test]
    fn leaf_fanouts_of_the_four_key_arms_match_the_original_apparatus() {
        let fanouts: Vec<u64> = KEY_ARMS.iter().map(|arm| arm.criterion10_geometry().leaf_fanout()).collect();
        assert_eq!(fanouts, vec![542, 603, 740, 1087], "原产物 acct_budget 行的 leaf_fanout");
    }

    #[test]
    fn today_header_geometry_has_header_96_and_fanout_601() {
        assert_eq!(NODE_HEADERS_TODAY[0], 58 + 10 + 28, "登记 S1：58 + 写序 10 + 预留位 28");
        assert_eq!(today_header_geometry().leaf_fanout(), 601);
    }

    #[test]
    fn implementation_geometry_matches_the_format_constants_and_the_capacity_comment() {
        assert_eq!(
            IMPLEMENTATION_NODE_HEADER_WITHOUT_KEY_RANGE_BYTES + 2 * IMPLEMENTATION_ACCOUNTING_KEY_BYTES,
            IMPLEMENTATION_NODE_HEADER_WITH_KEY_RANGE_BYTES,
            "86 + 2 × 22 + 29 = 159"
        );
        assert_eq!(implementation_geometry().leaf_fanout(), IMPLEMENTATION_LEAF_CAPACITY_IN_ENTRIES, "记账树叶 477");
    }

    #[test]
    fn judged_cell_wide_1024_by_64_drops_from_13910_to_554_leaves() {
        let cell = judged_arm_cell(JUDGED_GRID_TREES_1024_DEVICES_64, Reading::Wide);
        assert_eq!(cell.groups, 5 * 1024 * 64 + 6 * 1024, "宽读组数 5td + 6t");
        assert_eq!(cell.new.groups_per_leaf, 603);
        assert_eq!(cell.new.dirty_bytes, 554 * NODE_BYTES);
        assert!(!exceeds_budget(cell.new.dirty_bytes, ACCOUNTING_WRITE_BUDGET_BYTES), "新口径不过界");
        assert_eq!(cell.old.groups_per_leaf, 24);
        assert_eq!(cell.old.dirty_bytes, 13_910 * NODE_BYTES);
        assert!(exceeds_budget(cell.old.dirty_bytes, ACCOUNTING_WRITE_BUDGET_BYTES), "旧口径过界");
        assert_eq!(percent_of_budget(cell.new.dirty_bytes), 42);
        assert_eq!(permille_of_budget(cell.new.dirty_bytes), 422);
    }

    #[test]
    fn judged_cell_wide_8192_by_8_drops_from_15702_to_625_leaves() {
        let cell = judged_arm_cell(JUDGED_GRID_TREES_8192_DEVICES_8, Reading::Wide);
        assert_eq!(cell.groups, 5 * 8192 * 8 + 6 * 8192);
        assert_eq!(cell.new.dirty_bytes, 625 * NODE_BYTES);
        assert!(!exceeds_budget(cell.new.dirty_bytes, ACCOUNTING_WRITE_BUDGET_BYTES));
        assert_eq!(cell.old.dirty_bytes, 15_702 * NODE_BYTES);
        assert_eq!(percent_of_budget(cell.new.dirty_bytes), 47);
        assert_eq!(permille_of_budget(cell.new.dirty_bytes), 476);
    }

    #[test]
    fn widest_cell_8192_by_64_stays_over_budget_under_the_new_caliber() {
        let cell = judged_arm_cell(Grid { tree_count: 8192, device_count: 64 }, Reading::Wide);
        assert_eq!(cell.new.dirty_bytes, 4429 * NODE_BYTES);
        assert!(exceeds_budget(cell.new.dirty_bytes, ACCOUNTING_WRITE_BUDGET_BYTES), "72 564 736 > 21 474 836");
        assert_eq!(permille_of_budget(cell.new.dirty_bytes), 3379);
    }

    #[test]
    fn judged_arm_question_lines_report_counts_and_tightest_narrow_cells() {
        let cells = evaluate_geometry(&KEY_ARMS[JUDGED_KEY_ARM_INDEX].criterion10_geometry());
        let wide_cells = cells_of_reading(&cells, Reading::Wide);
        let narrow_cells = cells_of_reading(&cells, Reading::Narrow);
        assert_eq!(over_cells(&wide_cells, old_dirty_bytes), (3, "1024x64,8192x8,8192x64".to_string()));
        assert_eq!(over_cells(&wide_cells, new_dirty_bytes), (1, "8192x64".to_string()));
        assert_eq!(tightest_cells(&narrow_cells, old_dirty_bytes), (697 * NODE_BYTES, "8192x64".to_string()));
        assert_eq!(tightest_cells(&narrow_cells, new_dirty_bytes), (28 * NODE_BYTES, "8192x8,8192x64".to_string()));
        assert_eq!(percent_of_budget(697 * NODE_BYTES), 53, "原单元测试钉的 53%");
        assert_eq!(percent_of_budget(28 * NODE_BYTES), 2);
        assert_eq!(permille_of_budget(28 * NODE_BYTES), 21);
        assert_eq!(over_cells(&narrow_cells, new_dirty_bytes), (0, "none".to_string()));
        let bodies = question_line_bodies(KEY_ARMS[JUDGED_KEY_ARM_INDEX].name, &cells);
        assert_eq!(bodies.len(), 5, "Q-E97-1 到 Q-E97-5 各一行");
    }

    #[test]
    fn a_leaf_narrower_than_the_generation_count_still_holds_one_group() {
        let dirty = dirty_leaves_per_publish(30, 25, 10);
        assert_eq!(dirty.groups_per_leaf, 1, "10 / 25 整除是 0，下限 1");
        assert_eq!(dirty.dirty_bytes, 30 * NODE_BYTES);
    }

    #[test]
    fn budget_judgement_is_strict_and_flips_when_the_threshold_moves_between_two_cells() {
        assert!(!exceeds_budget(ACCOUNTING_WRITE_BUDGET_BYTES, ACCOUNTING_WRITE_BUDGET_BYTES), "等于门槛不算过界");
        assert!(exceeds_budget(ACCOUNTING_WRITE_BUDGET_BYTES + 1, ACCOUNTING_WRITE_BUDGET_BYTES));
        assert!(!exceeds_budget(554 * NODE_BYTES, 9_076_736), "门槛取 9 076 736 时判决格一不过界");
        assert!(exceeds_budget(554 * NODE_BYTES, 9_076_735), "门槛取 9 076 735 时判决格一过界");
    }

    #[test]
    fn positive_control_generations_in_formula_holds_on_every_arm() {
        for arm in &KEY_ARMS {
            let cells = evaluate_geometry(&arm.criterion10_geometry());
            assert_eq!(generations_in_formula_control(&cells), (8, 8, 8), "臂 {}", arm.name);
        }
    }

    #[test]
    fn positive_control_device_dimension_holds_on_every_arm() {
        for arm in &KEY_ARMS {
            let (with_device, without_device) = device_dimension_control(&arm.criterion10_geometry());
            assert!(without_device < with_device, "臂 {}", arm.name);
        }
        let judged = device_dimension_control(&KEY_ARMS[JUDGED_KEY_ARM_INDEX].criterion10_geometry());
        assert_eq!(judged, (554 * NODE_BYTES, 19 * NODE_BYTES), "去掉设备维：11 × 1024 组 / 603 向上整除 = 19 片");
    }

    #[test]
    fn no_geometry_flips_the_judged_cells_under_the_new_caliber() {
        for geometry in sensitivity_geometries() {
            let sensitivity = geometry_sensitivity(&geometry);
            assert!(!sensitivity.is_wide_1024_by_64_over && !sensitivity.is_wide_8192_by_8_over, "几何 {}", geometry.label);
        }
        assert_eq!(sensitivity_geometries().len(), 6, "四条 key 臂加 S1、S2");
    }

    /// 登记第七节乙类：`anchors.py`（只按登记读法第 1–3 条的式子现算，不读装置代码）的 `anchors.out` 里 48 行 e97 锚点，
    /// 逐行生成一条用例（生成器 `gen_anchor_tests.py` 在草稿目录，读 `anchors.out` 原样行）。脏字节写成「片数 × 16384」。
    macro_rules! anchor_test {
        ($test_name:ident, $geometry:expr, $fanout:expr, $trees:expr, $devices:expr, $reading:expr, $groups:expr,
         $new_leaves:expr, $new_over:expr, $new_permille:expr, $old_leaves:expr, $old_over:expr) => {
            #[test]
            fn $test_name() {
                let cell = evaluate_cell(&$geometry, Grid { tree_count: $trees, device_count: $devices }, $reading);
                assert_eq!(cell.leaf_fanout, $fanout, "叶扇出");
                assert_eq!(cell.groups, $groups, "组数");
                assert_eq!(cell.new.dirty_bytes, $new_leaves * NODE_BYTES, "新口径脏字节");
                assert_eq!(over_bit(cell.new.dirty_bytes), $new_over, "新口径过界");
                assert_eq!(permille_of_budget(cell.new.dirty_bytes), $new_permille, "新口径千分比");
                assert_eq!(cell.old.dirty_bytes, $old_leaves * NODE_BYTES, "旧口径脏字节");
                assert_eq!(over_bit(cell.old.dirty_bytes), $old_over, "旧口径过界");
            }
        };
    }

    anchor_test!(anchor_key22_header58_trees_64_devices_8_narrow, KEY_ARMS[0].criterion10_geometry(), 542, 64, 8, Reading::Narrow, 172, 1, 0, 0, 9, 0);
    anchor_test!(anchor_key22_header58_trees_64_devices_8_wide, KEY_ARMS[0].criterion10_geometry(), 542, 64, 8, Reading::Wide, 2944, 6, 0, 4, 141, 0);
    anchor_test!(anchor_key22_header58_trees_1024_devices_64_narrow, KEY_ARMS[0].criterion10_geometry(), 542, 1024, 64, Reading::Narrow, 2372, 5, 0, 3, 113, 0);
    anchor_test!(anchor_key22_header58_trees_1024_devices_64_wide, KEY_ARMS[0].criterion10_geometry(), 542, 1024, 64, Reading::Wide, 333824, 616, 0, 469, 15897, 1);
    anchor_test!(anchor_key22_header58_trees_8192_devices_8_narrow, KEY_ARMS[0].criterion10_geometry(), 542, 8192, 8, Reading::Narrow, 16428, 31, 0, 23, 783, 0);
    anchor_test!(anchor_key22_header58_trees_8192_devices_8_wide, KEY_ARMS[0].criterion10_geometry(), 542, 8192, 8, Reading::Wide, 376832, 696, 0, 531, 17945, 1);
    anchor_test!(anchor_key22_header58_trees_8192_devices_64_narrow, KEY_ARMS[0].criterion10_geometry(), 542, 8192, 64, Reading::Narrow, 16708, 31, 0, 23, 796, 0);
    anchor_test!(anchor_key22_header58_trees_8192_devices_64_wide, KEY_ARMS[0].criterion10_geometry(), 542, 8192, 64, Reading::Wide, 2670592, 4928, 1, 3759, 127172, 1);
    anchor_test!(anchor_key19_header58_trees_64_devices_8_narrow, KEY_ARMS[1].criterion10_geometry(), 603, 64, 8, Reading::Narrow, 172, 1, 0, 0, 8, 0);
    anchor_test!(anchor_key19_header58_trees_64_devices_8_wide, KEY_ARMS[1].criterion10_geometry(), 603, 64, 8, Reading::Wide, 2944, 5, 0, 3, 123, 0);
    anchor_test!(anchor_key19_header58_trees_1024_devices_64_narrow, KEY_ARMS[1].criterion10_geometry(), 603, 1024, 64, Reading::Narrow, 2372, 4, 0, 3, 99, 0);
    anchor_test!(anchor_key19_header58_trees_1024_devices_64_wide, KEY_ARMS[1].criterion10_geometry(), 603, 1024, 64, Reading::Wide, 333824, 554, 0, 422, 13910, 1);
    anchor_test!(anchor_key19_header58_trees_8192_devices_8_narrow, KEY_ARMS[1].criterion10_geometry(), 603, 8192, 8, Reading::Narrow, 16428, 28, 0, 21, 685, 0);
    anchor_test!(anchor_key19_header58_trees_8192_devices_8_wide, KEY_ARMS[1].criterion10_geometry(), 603, 8192, 8, Reading::Wide, 376832, 625, 0, 476, 15702, 1);
    anchor_test!(anchor_key19_header58_trees_8192_devices_64_narrow, KEY_ARMS[1].criterion10_geometry(), 603, 8192, 64, Reading::Narrow, 16708, 28, 0, 21, 697, 0);
    anchor_test!(anchor_key19_header58_trees_8192_devices_64_wide, KEY_ARMS[1].criterion10_geometry(), 603, 8192, 64, Reading::Wide, 2670592, 4429, 1, 3379, 111275, 1);
    anchor_test!(anchor_key14_header58_trees_64_devices_8_narrow, KEY_ARMS[2].criterion10_geometry(), 740, 64, 8, Reading::Narrow, 172, 1, 0, 0, 6, 0);
    anchor_test!(anchor_key14_header58_trees_64_devices_8_wide, KEY_ARMS[2].criterion10_geometry(), 740, 64, 8, Reading::Wide, 2944, 4, 0, 3, 102, 0);
    anchor_test!(anchor_key14_header58_trees_1024_devices_64_narrow, KEY_ARMS[2].criterion10_geometry(), 740, 1024, 64, Reading::Narrow, 2372, 4, 0, 3, 82, 0);
    anchor_test!(anchor_key14_header58_trees_1024_devices_64_wide, KEY_ARMS[2].criterion10_geometry(), 740, 1024, 64, Reading::Wide, 333824, 452, 0, 344, 11512, 1);
    anchor_test!(anchor_key14_header58_trees_8192_devices_8_narrow, KEY_ARMS[2].criterion10_geometry(), 740, 8192, 8, Reading::Narrow, 16428, 23, 0, 17, 567, 0);
    anchor_test!(anchor_key14_header58_trees_8192_devices_8_wide, KEY_ARMS[2].criterion10_geometry(), 740, 8192, 8, Reading::Wide, 376832, 510, 0, 389, 12995, 1);
    anchor_test!(anchor_key14_header58_trees_8192_devices_64_narrow, KEY_ARMS[2].criterion10_geometry(), 740, 8192, 64, Reading::Narrow, 16708, 23, 0, 17, 577, 0);
    anchor_test!(anchor_key14_header58_trees_8192_devices_64_wide, KEY_ARMS[2].criterion10_geometry(), 740, 8192, 64, Reading::Wide, 2670592, 3609, 1, 2753, 92090, 1);
    anchor_test!(anchor_key7_header58_trees_64_devices_8_narrow, KEY_ARMS[3].criterion10_geometry(), 1087, 64, 8, Reading::Narrow, 172, 1, 0, 0, 4, 0);
    anchor_test!(anchor_key7_header58_trees_64_devices_8_wide, KEY_ARMS[3].criterion10_geometry(), 1087, 64, 8, Reading::Wide, 2944, 3, 0, 2, 69, 0);
    anchor_test!(anchor_key7_header58_trees_1024_devices_64_narrow, KEY_ARMS[3].criterion10_geometry(), 1087, 1024, 64, Reading::Narrow, 2372, 3, 0, 2, 56, 0);
    anchor_test!(anchor_key7_header58_trees_1024_devices_64_wide, KEY_ARMS[3].criterion10_geometry(), 1087, 1024, 64, Reading::Wide, 333824, 308, 0, 234, 7764, 1);
    anchor_test!(anchor_key7_header58_trees_8192_devices_8_narrow, KEY_ARMS[3].criterion10_geometry(), 1087, 8192, 8, Reading::Narrow, 16428, 16, 0, 12, 383, 0);
    anchor_test!(anchor_key7_header58_trees_8192_devices_8_wide, KEY_ARMS[3].criterion10_geometry(), 1087, 8192, 8, Reading::Wide, 376832, 347, 0, 264, 8764, 1);
    anchor_test!(anchor_key7_header58_trees_8192_devices_64_narrow, KEY_ARMS[3].criterion10_geometry(), 1087, 8192, 64, Reading::Narrow, 16708, 16, 0, 12, 389, 0);
    anchor_test!(anchor_key7_header58_trees_8192_devices_64_wide, KEY_ARMS[3].criterion10_geometry(), 1087, 8192, 64, Reading::Wide, 2670592, 2457, 1, 1874, 62107, 1);
    anchor_test!(anchor_key19_header96_trees_64_devices_8_narrow, today_header_geometry(), 601, 64, 8, Reading::Narrow, 172, 1, 0, 0, 8, 0);
    anchor_test!(anchor_key19_header96_trees_64_devices_8_wide, today_header_geometry(), 601, 64, 8, Reading::Wide, 2944, 5, 0, 3, 123, 0);
    anchor_test!(anchor_key19_header96_trees_1024_devices_64_narrow, today_header_geometry(), 601, 1024, 64, Reading::Narrow, 2372, 4, 0, 3, 99, 0);
    anchor_test!(anchor_key19_header96_trees_1024_devices_64_wide, today_header_geometry(), 601, 1024, 64, Reading::Wide, 333824, 556, 0, 424, 13910, 1);
    anchor_test!(anchor_key19_header96_trees_8192_devices_8_narrow, today_header_geometry(), 601, 8192, 8, Reading::Narrow, 16428, 28, 0, 21, 685, 0);
    anchor_test!(anchor_key19_header96_trees_8192_devices_8_wide, today_header_geometry(), 601, 8192, 8, Reading::Wide, 376832, 628, 0, 479, 15702, 1);
    anchor_test!(anchor_key19_header96_trees_8192_devices_64_narrow, today_header_geometry(), 601, 8192, 64, Reading::Narrow, 16708, 28, 0, 21, 697, 0);
    anchor_test!(anchor_key19_header96_trees_8192_devices_64_wide, today_header_geometry(), 601, 8192, 64, Reading::Wide, 2670592, 4444, 1, 3390, 111275, 1);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_64_devices_8_narrow, implementation_geometry(), 477, 64, 8, Reading::Narrow, 172, 1, 0, 0, 10, 0);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_64_devices_8_wide, implementation_geometry(), 477, 64, 8, Reading::Wide, 2944, 7, 0, 5, 155, 0);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_1024_devices_64_narrow, implementation_geometry(), 477, 1024, 64, Reading::Narrow, 2372, 5, 0, 3, 125, 0);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_1024_devices_64_wide, implementation_geometry(), 477, 1024, 64, Reading::Wide, 333824, 700, 0, 534, 17570, 1);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_8192_devices_8_narrow, implementation_geometry(), 477, 8192, 8, Reading::Narrow, 16428, 35, 0, 26, 865, 0);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_8192_devices_8_wide, implementation_geometry(), 477, 8192, 8, Reading::Wide, 376832, 791, 0, 603, 19834, 1);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_8192_devices_64_narrow, implementation_geometry(), 477, 8192, 64, Reading::Narrow, 16708, 36, 0, 27, 880, 0);
    anchor_test!(anchor_implementation_key22_header115_entry34_trees_8192_devices_64_wide, implementation_geometry(), 477, 8192, 64, Reading::Wide, 2670592, 5599, 1, 4271, 140558, 1);

    /// 原产物 `research/results/e97-entry-encoding-2026-09-07.out`（已归档：`git show 3cff909b0^:research/results/e97-entry-encoding-2026-09-07.out`）
    /// 第 66–97 行 `name=acct_budget` 的 `name=` 之后那一截，原样抄。登记 V1 要旧口径 32 格与它逐字一致。
    const ARCHIVED_BUDGET_LINE_BODIES: [&str; 32] = [
        "arm=acct_e73_22B trees=64 devs=8 reading=narrow entry=30 leaf_fanout=542 groups=172 entry_bytes=10320 over_by_entry=0 node_bytes=147456 over_by_node=0",
        "arm=acct_e73_22B trees=64 devs=8 reading=wide entry=30 leaf_fanout=542 groups=2944 entry_bytes=176640 over_by_entry=0 node_bytes=2310144 over_by_node=0",
        "arm=acct_e73_22B trees=1024 devs=64 reading=narrow entry=30 leaf_fanout=542 groups=2372 entry_bytes=142320 over_by_entry=0 node_bytes=1851392 over_by_node=0",
        "arm=acct_e73_22B trees=1024 devs=64 reading=wide entry=30 leaf_fanout=542 groups=333824 entry_bytes=20029440 over_by_entry=0 node_bytes=260456448 over_by_node=1",
        "arm=acct_e73_22B trees=8192 devs=8 reading=narrow entry=30 leaf_fanout=542 groups=16428 entry_bytes=985680 over_by_entry=0 node_bytes=12828672 over_by_node=0",
        "arm=acct_e73_22B trees=8192 devs=8 reading=wide entry=30 leaf_fanout=542 groups=376832 entry_bytes=22609920 over_by_entry=1 node_bytes=294010880 over_by_node=1",
        "arm=acct_e73_22B trees=8192 devs=64 reading=narrow entry=30 leaf_fanout=542 groups=16708 entry_bytes=1002480 over_by_entry=0 node_bytes=13041664 over_by_node=0",
        "arm=acct_e73_22B trees=8192 devs=64 reading=wide entry=30 leaf_fanout=542 groups=2670592 entry_bytes=160235520 over_by_entry=1 node_bytes=2083586048 over_by_node=1",
        "arm=acct_wide_19B trees=64 devs=8 reading=narrow entry=27 leaf_fanout=603 groups=172 entry_bytes=9288 over_by_entry=0 node_bytes=131072 over_by_node=0",
        "arm=acct_wide_19B trees=64 devs=8 reading=wide entry=27 leaf_fanout=603 groups=2944 entry_bytes=158976 over_by_entry=0 node_bytes=2015232 over_by_node=0",
        "arm=acct_wide_19B trees=1024 devs=64 reading=narrow entry=27 leaf_fanout=603 groups=2372 entry_bytes=128088 over_by_entry=0 node_bytes=1622016 over_by_node=0",
        "arm=acct_wide_19B trees=1024 devs=64 reading=wide entry=27 leaf_fanout=603 groups=333824 entry_bytes=18026496 over_by_entry=0 node_bytes=227901440 over_by_node=1",
        "arm=acct_wide_19B trees=8192 devs=8 reading=narrow entry=27 leaf_fanout=603 groups=16428 entry_bytes=887112 over_by_entry=0 node_bytes=11223040 over_by_node=0",
        "arm=acct_wide_19B trees=8192 devs=8 reading=wide entry=27 leaf_fanout=603 groups=376832 entry_bytes=20348928 over_by_entry=0 node_bytes=257261568 over_by_node=1",
        "arm=acct_wide_19B trees=8192 devs=64 reading=narrow entry=27 leaf_fanout=603 groups=16708 entry_bytes=902232 over_by_entry=0 node_bytes=11419648 over_by_node=0",
        "arm=acct_wide_19B trees=8192 devs=64 reading=wide entry=27 leaf_fanout=603 groups=2670592 entry_bytes=144211968 over_by_entry=1 node_bytes=1823129600 over_by_node=1",
        "arm=acct_tight_14B trees=64 devs=8 reading=narrow entry=22 leaf_fanout=740 groups=172 entry_bytes=7568 over_by_entry=0 node_bytes=98304 over_by_node=0",
        "arm=acct_tight_14B trees=64 devs=8 reading=wide entry=22 leaf_fanout=740 groups=2944 entry_bytes=129536 over_by_entry=0 node_bytes=1671168 over_by_node=0",
        "arm=acct_tight_14B trees=1024 devs=64 reading=narrow entry=22 leaf_fanout=740 groups=2372 entry_bytes=104368 over_by_entry=0 node_bytes=1343488 over_by_node=0",
        "arm=acct_tight_14B trees=1024 devs=64 reading=wide entry=22 leaf_fanout=740 groups=333824 entry_bytes=14688256 over_by_entry=0 node_bytes=188612608 over_by_node=1",
        "arm=acct_tight_14B trees=8192 devs=8 reading=narrow entry=22 leaf_fanout=740 groups=16428 entry_bytes=722832 over_by_entry=0 node_bytes=9289728 over_by_node=0",
        "arm=acct_tight_14B trees=8192 devs=8 reading=wide entry=22 leaf_fanout=740 groups=376832 entry_bytes=16580608 over_by_entry=0 node_bytes=212910080 over_by_node=1",
        "arm=acct_tight_14B trees=8192 devs=64 reading=narrow entry=22 leaf_fanout=740 groups=16708 entry_bytes=735152 over_by_entry=0 node_bytes=9453568 over_by_node=0",
        "arm=acct_tight_14B trees=8192 devs=64 reading=wide entry=22 leaf_fanout=740 groups=2670592 entry_bytes=117506048 over_by_entry=1 node_bytes=1508802560 over_by_node=1",
        "arm=acct_windowgen_7B trees=64 devs=8 reading=narrow entry=15 leaf_fanout=1087 groups=172 entry_bytes=5160 over_by_entry=0 node_bytes=65536 over_by_node=0",
        "arm=acct_windowgen_7B trees=64 devs=8 reading=wide entry=15 leaf_fanout=1087 groups=2944 entry_bytes=88320 over_by_entry=0 node_bytes=1130496 over_by_node=0",
        "arm=acct_windowgen_7B trees=1024 devs=64 reading=narrow entry=15 leaf_fanout=1087 groups=2372 entry_bytes=71160 over_by_entry=0 node_bytes=917504 over_by_node=0",
        "arm=acct_windowgen_7B trees=1024 devs=64 reading=wide entry=15 leaf_fanout=1087 groups=333824 entry_bytes=10014720 over_by_entry=0 node_bytes=127205376 over_by_node=1",
        "arm=acct_windowgen_7B trees=8192 devs=8 reading=narrow entry=15 leaf_fanout=1087 groups=16428 entry_bytes=492840 over_by_entry=0 node_bytes=6275072 over_by_node=0",
        "arm=acct_windowgen_7B trees=8192 devs=8 reading=wide entry=15 leaf_fanout=1087 groups=376832 entry_bytes=11304960 over_by_entry=0 node_bytes=143589376 over_by_node=1",
        "arm=acct_windowgen_7B trees=8192 devs=64 reading=narrow entry=15 leaf_fanout=1087 groups=16708 entry_bytes=501240 over_by_entry=0 node_bytes=6373376 over_by_node=0",
        "arm=acct_windowgen_7B trees=8192 devs=64 reading=wide entry=15 leaf_fanout=1087 groups=2670592 entry_bytes=80117760 over_by_entry=1 node_bytes=1017561088 over_by_node=1",
    ];

    /// 同一份原产物第 98–105 行 `name=acct_key_ceiling` 的 `name=` 之后那一截。
    const ARCHIVED_KEY_CEILING_LINE_BODIES: [&str; 8] = [
        "trees=64 devs=8 reading=narrow touched_entries=344 budget=21474836 max_entry_bytes=62426 max_key_bytes=62418 feasible=1",
        "trees=64 devs=8 reading=wide touched_entries=5888 budget=21474836 max_entry_bytes=3647 max_key_bytes=3639 feasible=1",
        "trees=1024 devs=64 reading=narrow touched_entries=4744 budget=21474836 max_entry_bytes=4526 max_key_bytes=4518 feasible=1",
        "trees=1024 devs=64 reading=wide touched_entries=667648 budget=21474836 max_entry_bytes=32 max_key_bytes=24 feasible=1",
        "trees=8192 devs=8 reading=narrow touched_entries=32856 budget=21474836 max_entry_bytes=653 max_key_bytes=645 feasible=1",
        "trees=8192 devs=8 reading=wide touched_entries=753664 budget=21474836 max_entry_bytes=28 max_key_bytes=20 feasible=1",
        "trees=8192 devs=64 reading=narrow touched_entries=33416 budget=21474836 max_entry_bytes=642 max_key_bytes=634 feasible=1",
        "trees=8192 devs=64 reading=wide touched_entries=5341184 budget=21474836 max_entry_bytes=4 max_key_bytes=0 feasible=0",
    ];

    #[test]
    fn old_caliber_reproduces_the_archived_budget_lines() {
        let reproduced: Vec<String> = KEY_ARMS
            .iter()
            .flat_map(|arm| {
                evaluate_geometry(&arm.criterion10_geometry())
                    .into_iter()
                    .map(move |cell| budget_line_body(arm, &cell, RowCountCaliber::TwentyFiveGenerationsWriteAndDelete))
            })
            .collect();
        assert_eq!(reproduced, ARCHIVED_BUDGET_LINE_BODIES.to_vec(), "登记 V1：旧口径 32 格与原产物逐字一致");
    }

    #[test]
    fn old_caliber_reproduces_the_archived_key_ceiling_lines() {
        let reproduced: Vec<String> = GRIDS
            .iter()
            .flat_map(|grid| {
                Reading::BOTH
                    .iter()
                    .map(move |reading| key_ceiling_line_body(*grid, *reading, RowCountCaliber::TwentyFiveGenerationsWriteAndDelete))
            })
            .collect();
        assert_eq!(reproduced, ARCHIVED_KEY_CEILING_LINE_BODIES.to_vec());
    }

    #[test]
    fn new_caliber_budget_line_halves_the_entry_bytes_of_the_judged_cell() {
        let arm = &KEY_ARMS[JUDGED_KEY_ARM_INDEX];
        let cell = judged_arm_cell(JUDGED_GRID_TREES_1024_DEVICES_64, Reading::Wide);
        assert_eq!(
            budget_line_body(arm, &cell, RowCountCaliber::SingleGenerationWriteOnly),
            format!(
                "arm=acct_wide_19B trees=1024 devs=64 reading=wide entry=27 leaf_fanout=603 groups=333824 entry_bytes={} \
                 over_by_entry=0 node_bytes={} over_by_node=0",
                333_824 * 27,
                554 * NODE_BYTES
            )
        );
    }

    #[test]
    fn new_caliber_key_ceiling_at_8192_by_64_wide_is_infeasible_and_at_1024_by_64_wide_is_56_bytes() {
        let widest = key_ceiling(accounting_groups(8192, 64, Reading::Wide), RowCountCaliber::SingleGenerationWriteOnly);
        assert_eq!(widest.touched_entries, 2_670_592);
        assert_eq!(widest.maximum_entry_bytes, 8);
        assert!(!widest.is_feasible, "条目只够装 value 8 字节，key 0 字节");
        let judged = key_ceiling(accounting_groups(1024, 64, Reading::Wide), RowCountCaliber::SingleGenerationWriteOnly);
        assert_eq!(judged.maximum_entry_bytes, 64);
        assert_eq!(judged.maximum_key_bytes, 56);
    }

    #[test]
    fn old_caliber_reproduces_the_archived_tree_height_line_of_the_22_byte_arm() {
        assert_eq!(
            tree_height_line_body(&KEY_ARMS[0], 58, 32, RowCountCaliber::TwentyFiveGenerationsWriteAndDelete),
            "arm=acct_e73_22B key_bytes=22 leaf_entry=30 inner_entry=54 header=58 child_ptr=32 leaf_fanout=542 \
             inner_fanout=301 entries_narrow=4300 height_narrow=2 entries_wide=73600 height_wide=2",
            "原产物第 18 行"
        );
    }

    #[test]
    fn new_caliber_narrow_tree_fits_in_one_leaf_and_wide_tree_keeps_height_two() {
        assert_eq!(
            tree_height_line_body(&KEY_ARMS[JUDGED_KEY_ARM_INDEX], 58, 32, RowCountCaliber::SingleGenerationWriteOnly),
            "arm=acct_wide_19B key_bytes=19 leaf_entry=27 inner_entry=51 header=58 child_ptr=32 leaf_fanout=603 \
             inner_fanout=319 entries_narrow=172 height_narrow=1 entries_wide=2944 height_wide=2",
        );
    }
}
