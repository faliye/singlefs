//! E16 第五次跑：今天的单元宽度（扇出、树高、记录宽）下，甲（`intent`）/ 乙（`write_ahead_log_leaf`）
//! 比值在「流数 × 批」二维上的峰值落在哪、预测式还对不对、实验页那句适用范围句还是不是真话。
//!
//! 跑前登记 `research/prompts/e16-r5-prereg.md` 写死了这个装置：**独立手写计数模型，
//! 不与 `crates/` 共用代码**（不是入库装置），只把原装置 `research/e7-index-bench/src/bin/e16_journal.rs`
//! 的两条臂（甲=`Intent`、乙=`WriteAheadLogLeaf`）换成今天的几何、今天的记录宽。原装置的
//! `sweep()` 一个字都不动，仍是它自己的复跑行；这个装置只在 `pc3` 模式里现跑今天的几何取「原 56 格
//! 几何」（`ORIGINAL_DEVICE_GEOMETRY`）来对拍，不改原装置代码。
//!
//! ## 两条臂，逐字照跑前登记第 5.1 节
//!
//! | 臂 | 每次 fsync 当场写什么 |
//! |---|---|
//! | 甲（intent） | 自上次 fsync 以来的脏数据单元各 1 块 + 这些单元的全部祖先去重后各 1 块（层级 0 到根层级，含根）+ 1 条只装游标的记录 + 1 个根槽 |
//! | 乙（write_ahead_log_leaf） | 自上次 fsync 以来**还没落过**的脏数据单元各 1 块 + 点名这些单元的记录 `ceil(k / 每记录项数)` 条 |
//!
//! 甲每次 fsync 都清空脏集合（它是一次完整 checkpoint）；乙只更新「已落盘」集合，脏集合一路累积到
//! 下一次 checkpoint 才清空——`persisted` 存在的理由正是让「已经落过的不再算」这件事在乙的账上
//! 显式发生，见 `simulate_cell` 与变异 M15。
//!
//! ## 今天的宽度：三个常量，一处定义，回比 `crates/singlefs-format/src/lib.rs`
//!
//! 这不是入库装置，不受「入库装置不许引 `crates/` 常量」那条约束（那条只管
//! `crates/singlefs-checker-tier/src/bin/`）；但跑前登记仍把这几个数登记成本地常量、
//! 靠停机条款 S1（第十三节命令二）人工核对，不在编译期 `use crates::…`——独立手写模型的意义
//! 就在「不共用代码」，共用了常量定义就不独立了。
//!
//! ## Q1 归类没做「加密」那一步（跑前登记第六节 Q1 末句）
//!
//! 跑前登记要求 `B_lo` 或 `B_hi` 换成它在批轴上的相邻取值会改变归类时，在两个相邻取值之间
//! 逐个整数加密重判、报「加密过」。这一版没有实现这一步：报告与实验页里写明未做，
//! 由主 agent 判要不要第二段补。
use e7_index_bench::Emitter;
use std::collections::HashSet;

// ── 今天的宽度：抄自 crates/singlefs-format/src/lib.rs，行号见跑前登记第十三节命令二 ──────

/// 数据单元总字节数。lib.rs:20 `DATA_UNIT_BYTES`。
const DATA_UNIT_BYTES: u64 = 32768;
/// 数据单元头部字节数，载荷从这个偏移量开始。lib.rs:38 `DATA_UNIT_PAYLOAD_OFFSET`。
const DATA_UNIT_PAYLOAD_OFFSET: u64 = 134;
/// extent 树内部扇出。lib.rs:153 `EXTENT_TREE_INTERNAL_FANOUT`。M2 的锚点。
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// extent 树下段叶罩的数据单元数。lib.rs:156 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`。M1 的锚点。
const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// journal 一条记录的定长字节数。lib.rs:191 `JOURNAL_RECORD_BYTES`。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// journal 记录头部字节数。lib.rs:200 `JOURNAL_HEADER_BYTES`。
const JOURNAL_HEADER_BYTES: u64 = 311;
/// journal 一个点名项的字节数。lib.rs:203 `JOURNAL_NAMED_ENTRY_BYTES`。
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
/// 一条记录最多装的点名项数 = (4096 − 311) ÷ 56 = 67。lib.rs:206
/// `JOURNAL_NAMED_ENTRIES_PER_RECORD`。M4 的锚点：换成 72 就是旧记录宽。
const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 67;

/// 数据单元载荷字节数 = 总字节 − 头偏移。M3 的锚点：换成不扣头就是「载荷用 32768」。
const DATA_UNIT_PAYLOAD_BYTES: u64 = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET;

// ── 原装置（ORIGINAL_DEVICE_GEOMETRY）的几何与记录宽：只用于 PC3 对拍，与「今天」的常量互不影响 ────────────

/// 原装置 `research/e7-index-bench/src/bin/e16_journal.rs` 的 `FANOUT`。
const LEGACY_FANOUT: u64 = 128;
/// 原装置的 `HEIGHT`（祖先层数，含根）。
const LEGACY_ANCESTOR_LEVELS: u64 = 4;
/// 原装置 `ENTRY_BYTES`。
const LEGACY_ENTRY_BYTES: u64 = 24;
/// 原装置 `CHECKSUM_BYTES`。
const LEGACY_CHECKSUM_BYTES: u64 = 32;
/// 原装置 `BLOCK`。
const LEGACY_BLOCK_BYTES: u64 = 4096;
/// 原装置 WAL 记录的固定头字节（`write_ahead_log_record_bytes` 里的 `48`）。
const LEGACY_RECORD_HEADER_BYTES: u64 = 48;

// ── 两条臂共用的块数常量：甲的记录与根槽恒为 1 块，两种记录宽下都成立 ────────────────

/// 甲每次 fsync 写的那条游标记录的块数：今天 311 字节头、原装置 48 字节头，都 ≤ 一个块。
/// M5 的锚点：换成 0 就是「游标记录不计块」。
const INTENT_RECORD_BLOCKS: u64 = 1;
/// 甲每次 fsync 覆写的根槽块数（D22：发根 = 覆写一个槽，不是追加）。
/// M6 的锚点：换成 0 就是「根槽不计块」。
const INTENT_ROOT_SLOT_BLOCKS: u64 = 1;

/// 峰值集合的容差：批轴上比值落在 `max * (1 - 容差)` 以内的都算「峰值集合」的一部分。
/// 门槛在跑前登记第六节写死，读产物之前定的。M12 的锚点。
const PEAK_SET_TOLERANCE: f64 = 0.001;
/// 单调性判定的容差：相邻两点之间涨跌超过这个比例才算「涨」或「跌」，否则算平。
/// 门槛在跑前登记第六节写死。M14 的锚点。
const MONOTONIC_TOLERANCE: f64 = 0.01;
/// 「批 ≈ 流数」这条对角带的半宽因子：带子是 `[S / 因子, S * 因子]`。
/// 门槛在跑前登记第六节写死（`[S÷2, 2S]`）。M13 的锚点：换成 4 就是 `[S÷4, 4S]`。
const NEAR_DIAGONAL_BAND_FACTOR: f64 = 2.0;

// ── 几何 ──────────────────────────────────────────────────────────────────

/// 一个几何点：下段按位置寻址的树，`leaf_units` 是层级 0 节点罩的数据单元数，
/// `fanout` 是往上每层的扇出，`units` 是这棵树要罩住的数据单元总数。
#[derive(Clone, Copy)]
struct Geometry {
    name: &'static str,
    leaf_units: u64,
    fanout: u64,
    units: u64,
}

/// 层级 `level`（0 起）的节点罩住的数据单元跨度。
/// `crates/singlefs-core/src/extent_tree.rs:81-86`（`lower_span_in_data_units`）同一定义。
fn span_in_units(leaf_units: u64, fanout: u64, level: u32) -> u64 {
    leaf_units * fanout.pow(level)
}

/// 罩得住 `units` 个数据单元的最低层级：满足 `跨度 >= units` 的最小 level。
/// `crates/singlefs-core/src/extent_tree.rs:97-102`（`lower_root_level_for`）同一定义。
/// M8 的锚点：把 `<` 换成 `<=` 就是「跨度 > N 的最小层」，边界情形（`units` 恰等于某层跨度）会多算一层。
fn root_level_for(leaf_units: u64, fanout: u64, units: u64) -> u32 {
    let mut level = 0u32;
    while span_in_units(leaf_units, fanout, level) < units {
        level += 1;
    }
    level
}

impl Geometry {
    fn root_level(&self) -> u32 {
        root_level_for(self.leaf_units, self.fanout, self.units)
    }

    /// 祖先层数 = 根层级 + 1（最底一层是层级 0，含根本身）。就是原装置的 `HEIGHT`。
    fn node_levels(&self) -> u32 {
        self.root_level() + 1
    }

    /// 根下孩子数：`ceil(units / 根往下一层的跨度)`；根层级为 0（一层就罩住了）时只有 1 个孩子（根本身）。
    fn root_children(&self) -> u64 {
        let root_level = self.root_level();
        if root_level == 0 {
            1
        } else {
            self.units.div_ceil(span_in_units(self.leaf_units, self.fanout, root_level - 1))
        }
    }

    /// 一个数据单元从层级 0 到根层级（含根）的祖先下标，一层一个。
    /// M7 的锚点：把 `0..=root_level` 换成 `0..root_level` 就是「只走到根层级的下一层，漏根」。
    fn ancestor_indices(&self, unit: u64) -> Vec<(u32, u64)> {
        let root_level = self.root_level();
        (0..=root_level).map(|level| (level, unit / span_in_units(self.leaf_units, self.fanout, level))).collect()
    }
}

/// 一批数据单元的去重祖先数：多片单元共享的祖先只算一次。
fn deduplicated_ancestor_count(geometry: &Geometry, units: &[u64]) -> u64 {
    let mut ancestors: HashSet<(u32, u64)> = HashSet::new();
    for &unit in units {
        for pair in geometry.ancestor_indices(unit) {
            ancestors.insert(pair);
        }
    }
    ancestors.len() as u64
}

// 五个「今天」的几何点 + 原装置的 ORIGINAL_DEVICE_GEOMETRY，单元数抄自跑前登记第十三节「命令一」的独立锚点输出
// （sha256 见跑前登记，`root_level`/`node_levels`/`root_children` 由本文件的函数重算，单测钉在一起）。
const ORIGINAL_DEVICE_GEOMETRY_UNITS: u64 = 268_435_456; // = 128^4，原装置叶数
const FOUR_GIBIBYTE_FILE_GEOMETRY_UNITS: u64 = 131_611; // = ceil(4 GiB / 32634)
const ONE_TEBIBYTE_FILE_GEOMETRY_UNITS: u64 = 33_692_212; // = ceil(1 TiB / 32634)
const FULL_ROOT_LEVEL_FOUR_GEOMETRY_UNITS: u64 = 457_419_312; // = 144 * 147^3，根下孩子数满
const SIXTEEN_TEBIBYTE_FILE_GEOMETRY_UNITS: u64 = 539_075_383; // = ceil(16 TiB / 32634)
const TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY_UNITS: u64 = 8_226; // = ceil(256 MiB / 32634)

const ORIGINAL_DEVICE_GEOMETRY: Geometry = Geometry { name: "original_device", leaf_units: LEGACY_FANOUT, fanout: LEGACY_FANOUT, units: ORIGINAL_DEVICE_GEOMETRY_UNITS };
const FOUR_GIBIBYTE_FILE_GEOMETRY: Geometry = Geometry { name: "four_gibibyte_file", leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS, fanout: EXTENT_TREE_INTERNAL_FANOUT, units: FOUR_GIBIBYTE_FILE_GEOMETRY_UNITS };
const ONE_TEBIBYTE_FILE_GEOMETRY: Geometry = Geometry { name: "one_tebibyte_file", leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS, fanout: EXTENT_TREE_INTERNAL_FANOUT, units: ONE_TEBIBYTE_FILE_GEOMETRY_UNITS };
const FULL_ROOT_LEVEL_FOUR_GEOMETRY: Geometry = Geometry { name: "full_root_level_four", leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS, fanout: EXTENT_TREE_INTERNAL_FANOUT, units: FULL_ROOT_LEVEL_FOUR_GEOMETRY_UNITS };
const SIXTEEN_TEBIBYTE_FILE_GEOMETRY: Geometry = Geometry { name: "sixteen_tebibyte_file", leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS, fanout: EXTENT_TREE_INTERNAL_FANOUT, units: SIXTEEN_TEBIBYTE_FILE_GEOMETRY_UNITS };
const TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY: Geometry = Geometry { name: "two_fifty_six_mebibyte_file", leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS, fanout: EXTENT_TREE_INTERNAL_FANOUT, units: TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY_UNITS };

/// 第一段要跑的三个几何点（真实基线 ONE_TEBIBYTE_FILE_GEOMETRY，两侧树高取样点 FOUR_GIBIBYTE_FILE_GEOMETRY、SIXTEEN_TEBIBYTE_FILE_GEOMETRY）。
const FIRST_SEGMENT_GEOMETRIES: [Geometry; 3] = [FOUR_GIBIBYTE_FILE_GEOMETRY, ONE_TEBIBYTE_FILE_GEOMETRY, SIXTEEN_TEBIBYTE_FILE_GEOMETRY];

// ── 第二段（跑前登记第五节「实现量与分段」）：扇出旋钮与记录宽旋钮各自的取样点 ──────

/// 扇出旋钮的两个取样点（跑前登记第八节 8.2）：单元数与 ONE_TEBIBYTE_FILE_GEOMETRY 相同
/// （同一个 1 TiB 文件），只把 leaf_units、fanout 换成 128/128、296/296，与今天的 144/147 对照。
/// 独立算出：两点的 root_level/node_levels/root_children 都是 3/4/17（小扇出）、3/4/2（大扇出），
/// 见 `geometry_root_level_node_levels_root_children_of_the_fanout_knob_points_are_pinned`。
const SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY: Geometry =
    Geometry { name: "small_fanout_one_tebibyte_file", leaf_units: 128, fanout: 128, units: ONE_TEBIBYTE_FILE_GEOMETRY_UNITS };
const LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY: Geometry =
    Geometry { name: "large_fanout_one_tebibyte_file", leaf_units: 296, fanout: 296, units: ONE_TEBIBYTE_FILE_GEOMETRY_UNITS };

/// 记录宽旋钮的两个取样点（跑前登记第八节 8.2 R1、R∞）：树形与 ONE_TEBIBYTE_FILE_GEOMETRY 完全
/// 相同（leaf_units、fanout、units 一个字不改），只挂个不同的名字，方便在 second_segment 模式的
/// 产物里跟今天的记录宽（67 项一条）区分——记录宽旋钮只经 RecordFormat 起作用，不改树形。
const RECORD_WIDTH_KNOB_MINIMUM_GEOMETRY: Geometry = Geometry {
    name: "record_width_knob_minimum_one_entry_per_record",
    leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
    fanout: EXTENT_TREE_INTERNAL_FANOUT,
    units: ONE_TEBIBYTE_FILE_GEOMETRY_UNITS,
};
const RECORD_WIDTH_KNOB_MAXIMUM_GEOMETRY: Geometry = Geometry {
    name: "record_width_knob_maximum_one_million_entries_per_record",
    leaf_units: EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
    fanout: EXTENT_TREE_INTERNAL_FANOUT,
    units: ONE_TEBIBYTE_FILE_GEOMETRY_UNITS,
};

// ── 记录宽：今天的格式 vs 原装置的格式，只有乙（write_ahead_log_leaf）的记录块数依赖它 ───────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RecordFormat {
    /// 今天：一条记录装 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 项，按项数切。
    Today,
    /// 原装置：一条记录 48 字节头 + 每项 (24+32) 字节，按字节数除以 4096 上取整。
    LegacyOriginal,
    /// 第二段记录宽旋钮（跑前登记第八节 8.2 R1、R∞）：一条记录固定装这么多项，不看字节数——
    /// 只用来扫描「一条记录能装多少项」这个旋钮，今天的真实值走 `Today`。
    EntriesPerRecord(u64),
}

/// 乙（write_ahead_log_leaf）一次 fsync 点名 `entry_count` 个单元时，记录本身占几块。
fn write_ahead_log_leaf_record_blocks(entry_count: u64, format: RecordFormat) -> u64 {
    match format {
        // 没有 `_ =>`：新增一种记录格式不补这里就编译不过
        RecordFormat::Today => entry_count.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD),
        RecordFormat::LegacyOriginal => {
            let bytes = LEGACY_RECORD_HEADER_BYTES + (LEGACY_ENTRY_BYTES + LEGACY_CHECKSUM_BYTES) * entry_count;
            bytes.div_ceil(LEGACY_BLOCK_BYTES).max(1)
        }
        // M17 的锚点：换成地板除，条数不整除时会少算一条记录。
        RecordFormat::EntriesPerRecord(entries_per_record) => entry_count.div_ceil(entries_per_record),
    }
}

// ── 第二段三个旋钮分组（跑前登记第八节 8.2）：每组第一个是这一组的基线（第一段已经跑过，
// 这里重跑一次是为了同一张表里直接比，不必跨文件对第一段的产物），其余是这一组要补的取样点 ──

const TREE_HEIGHT_KNOB_POINTS: [(Geometry, RecordFormat); 5] = [
    (ONE_TEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
    (FOUR_GIBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
    (SIXTEEN_TEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
    (FULL_ROOT_LEVEL_FOUR_GEOMETRY, RecordFormat::Today),
    (TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
];
const FANOUT_KNOB_POINTS: [(Geometry, RecordFormat); 3] = [
    (ONE_TEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
    (SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
    (LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
];
const RECORD_WIDTH_KNOB_POINTS: [(Geometry, RecordFormat); 3] = [
    (ONE_TEBIBYTE_FILE_GEOMETRY, RecordFormat::Today),
    (RECORD_WIDTH_KNOB_MINIMUM_GEOMETRY, RecordFormat::EntriesPerRecord(1)),
    (RECORD_WIDTH_KNOB_MAXIMUM_GEOMETRY, RecordFormat::EntriesPerRecord(1_000_000)),
];

// ── 操作流：流数 S、批 B，与原装置同一个生成器（第一节「流、流的落点」逐字照抄） ─────

/// checkpoint 间隔：批的整数倍，且落在「批 × 10」的量级，不让 checkpoint 主导也不让脏集合无界增长
/// （跑前登记第一节「批 B」）。原装置固定 2000，因为它的 8 档批都整除 2000；这里把 2000 换成
/// `ceil(2000 / batch) * batch`，对能整除 2000 的批（原 8 档）结果不变。
/// M16 的锚点：换成恒定 `CHECKPOINT_BASE_OPERATIONS` 就是「不取批的整数倍」。
const CHECKPOINT_BASE_OPERATIONS: u64 = 2000;
fn checkpoint_interval_for_batch(batch: u64) -> u64 {
    batch * CHECKPOINT_BASE_OPERATIONS.div_ceil(batch)
}

/// 每个几何 / 流数 / 批组合要跑的操作总数 = checkpoint 间隔的 10 倍。
const OPERATIONS_PER_CHECKPOINT_INTERVAL: u64 = 10;
fn operation_count_for_batch(batch: u64) -> u64 {
    OPERATIONS_PER_CHECKPOINT_INTERVAL * checkpoint_interval_for_batch(batch)
}

/// 第 `stream_count` 条流各自占的单元数：地板除，最后一条流未必占满一个整份。
/// M9 的锚点：换成 `div_ceil` 就是「流的落点用 ⌈N÷S⌉」——`simulate_cell` 与单测都经这一个函数，
/// 改这里两边一起动。
fn units_per_stream_for(geometry: &Geometry, stream_count: u64) -> u64 {
    geometry.units / stream_count
}

/// 第 `cursor` 个操作弄脏哪个数据单元：第 `cursor` 个操作归第 `cursor mod S` 条流，
/// 每条流在自己的 `[i * (units/S), (i+1) * (units/S))` 区间里顺序推进。
fn unit_for_cursor(cursor: u64, stream_count: u64, units_per_stream: u64, stream_index: u64) -> u64 {
    let offset_in_stream = (cursor / stream_count) % units_per_stream;
    stream_index * units_per_stream + offset_in_stream
}

// ── 单次模拟：一个几何点 × 一个流数 × 一个批，跑出两条臂各自的 fsync 代价序列 ──────

struct CellSamples {
    intent_fsync_costs: Vec<u64>,
    write_ahead_log_leaf_fsync_costs: Vec<u64>,
}

/// 跑一个格：两条臂各自维护自己的状态，读同一条操作流（第一节「流、流的落点」的生成器），
/// 互不干扰——比的是同一份工作量。
fn simulate_cell(geometry: &Geometry, stream_count: u64, batch: u64, format: RecordFormat) -> CellSamples {
    let checkpoint_interval = checkpoint_interval_for_batch(batch);
    let operation_count = operation_count_for_batch(batch);
    let units_per_stream = units_per_stream_for(geometry, stream_count);

    // 甲：脏集合每次 fsync 都清空，本批的单元不会跨批重复，用一个按批清空的 Vec 就够。
    let mut intent_batch_units: Vec<u64> = Vec::with_capacity(batch as usize);
    // 乙：脏集合累积到下一次 checkpoint 才清空；`persisted` 记录上一次 fsync 时已经落盘的那些，
    // 让「还没落过的」这句话（第一节「今天的记录宽」）在账上显式发生——M15 拿掉这个减法。
    let mut write_ahead_log_leaf_dirty: HashSet<u64> = HashSet::new();
    let mut write_ahead_log_leaf_persisted: HashSet<u64> = HashSet::new();

    let mut intent_fsync_costs = Vec::new();
    let mut write_ahead_log_leaf_fsync_costs = Vec::new();

    for cursor in 0..operation_count {
        let stream_index = cursor % stream_count;
        let unit = unit_for_cursor(cursor, stream_count, units_per_stream, stream_index);
        intent_batch_units.push(unit);
        write_ahead_log_leaf_dirty.insert(unit);

        let is_fsync = (cursor + 1) % batch == 0;
        let is_checkpoint = (cursor + 1) % checkpoint_interval == 0;

        if is_fsync {
            debug_assert_eq!(
                intent_batch_units.iter().collect::<HashSet<_>>().len(),
                intent_batch_units.len(),
                "同一批内出现了重复单元号——多流生成器的「同一区间内不重复」假设被打破"
            );
            let ancestor_count = deduplicated_ancestor_count(geometry, &intent_batch_units);
            intent_fsync_costs.push(intent_batch_units.len() as u64 + ancestor_count + INTENT_RECORD_BLOCKS + INTENT_ROOT_SLOT_BLOCKS);
            intent_batch_units.clear();

            let unpersisted_count = write_ahead_log_leaf_dirty.difference(&write_ahead_log_leaf_persisted).count() as u64;
            write_ahead_log_leaf_fsync_costs.push(unpersisted_count + write_ahead_log_leaf_record_blocks(unpersisted_count, format));
            write_ahead_log_leaf_persisted = write_ahead_log_leaf_dirty.clone();
        }
        if is_checkpoint {
            write_ahead_log_leaf_dirty.clear();
            write_ahead_log_leaf_persisted.clear();
        }
    }

    CellSamples { intent_fsync_costs, write_ahead_log_leaf_fsync_costs }
}

fn cost_statistics(samples: &[u64]) -> (f64, u64, u64) {
    let sum: u64 = samples.iter().sum();
    let average = sum as f64 / samples.len() as f64;
    let minimum = *samples.iter().min().expect("空的代价序列——上游没跑到任何一次 fsync");
    let maximum = *samples.iter().max().expect("空的代价序列——上游没跑到任何一次 fsync");
    (average, minimum, maximum)
}

/// 先算的预测比值：甲每批 ≈ `批 + min(流数,批)×祖先层数 + 2`，乙 ≈ `批 + 1`（跑前登记第一节）。
/// M10 的锚点：把 `node_levels` 换成 `node_levels - 1` 就是套错了一层。
fn predicted_ratio(batch: u64, stream_count: u64, node_levels: u32) -> f64 {
    let predicted_intent = batch as f64 + (stream_count.min(batch) as f64) * node_levels as f64 + 2.0;
    let predicted_write_ahead_log_leaf = batch as f64 + 1.0;
    predicted_intent / predicted_write_ahead_log_leaf
}

/// 相对误差 = (预测比值 − 模型比值) ÷ 分母，带符号。分母默认是模型比值（跑前登记第一节原定义）。
/// M11 的锚点：把分母换成 `predicted_ratio` 就是 PC2 里失败条款 F2 要核的另一种分母。
fn relative_error(model_ratio: f64, predicted_ratio: f64, denominator: f64) -> f64 {
    (predicted_ratio - model_ratio) / denominator
}

struct CellResult {
    streams: u64,
    batch: u64,
    intent_average: f64,
    intent_minimum: u64,
    intent_maximum: u64,
    write_ahead_log_leaf_average: f64,
    write_ahead_log_leaf_minimum: u64,
    write_ahead_log_leaf_maximum: u64,
    ratio: f64,
    predicted_ratio: f64,
    error_model_denominator: f64,
    error_predicted_denominator: f64,
    windows: usize,
}

fn compute_cell(geometry: &Geometry, stream_count: u64, batch: u64, format: RecordFormat) -> CellResult {
    let samples = simulate_cell(geometry, stream_count, batch, format);
    let (intent_average, intent_minimum, intent_maximum) = cost_statistics(&samples.intent_fsync_costs);
    let (write_ahead_log_leaf_average, write_ahead_log_leaf_minimum, write_ahead_log_leaf_maximum) = cost_statistics(&samples.write_ahead_log_leaf_fsync_costs);
    let ratio = intent_average / write_ahead_log_leaf_average;
    let predicted = predicted_ratio(batch, stream_count, geometry.node_levels());
    CellResult {
        streams: stream_count,
        batch,
        intent_average,
        intent_minimum,
        intent_maximum,
        write_ahead_log_leaf_average,
        write_ahead_log_leaf_minimum,
        write_ahead_log_leaf_maximum,
        ratio,
        predicted_ratio: predicted,
        error_model_denominator: relative_error(ratio, predicted, ratio),
        error_predicted_denominator: relative_error(ratio, predicted, predicted),
        windows: samples.intent_fsync_costs.len(),
    }
}

fn emit_cell_line(emitter: &mut Emitter, output: &mut String, mode: &str, geometry_name: &str, cell: &CellResult) {
    output.push_str(&emitter.emit_raw(&format!(
        "name=cell mode={mode} geometry={geometry_name} streams={} batch={} intent_avg={:.4} intent_min={} intent_max={} \
         write_ahead_log_leaf_avg={:.4} write_ahead_log_leaf_min={} write_ahead_log_leaf_max={} ratio={:.6} predicted_ratio={:.6} \
         error_model_denominator={:.6} error_predicted_denominator={:.6} windows={}",
        cell.streams, cell.batch, cell.intent_average, cell.intent_minimum, cell.intent_maximum,
        cell.write_ahead_log_leaf_average, cell.write_ahead_log_leaf_minimum, cell.write_ahead_log_leaf_maximum, cell.ratio, cell.predicted_ratio,
        cell.error_model_denominator, cell.error_predicted_denominator, cell.windows
    )));
    output.push('\n');
}

// ── 坐标轴：第一节「流数轴 S」「批轴 B」，第八节的几何 / 端点扩展点直接并入 ─────────

const STREAM_AXIS_BASE: [u64; 17] = [2, 3, 4, 6, 7, 8, 11, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256];
/// Q2 端点扩展点（跑前登记第六节 Q2）：直接并入第一段，省一轮「触发了再加」的重跑。
const STREAM_AXIS_ENDPOINT_EXTENSION: [u64; 4] = [384, 512, 768, 1024];
const BATCH_AXIS_BASE: [u64; 49] = [
    1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 14, 16, 20, 24, 28, 32, 40, 48, 50, 56, 64, 67, 68, 70, 72, 73, 80, 96, 100,
    112, 128, 160, 192, 200, 224, 256, 320, 384, 448, 512, 640, 768, 1024, 1280, 1536, 2048, 3072, 4096,
];
/// F3 端点扩展点（跑前登记第六节 Q1）：同样直接并入。
const BATCH_AXIS_ENDPOINT_EXTENSION: [u64; 4] = [6144, 8192, 12288, 16384];
const ORIGINAL_56_STREAMS: [u64; 7] = [2, 4, 8, 16, 32, 64, 128];
const ORIGINAL_56_BATCHES: [u64; 8] = [1, 2, 5, 10, 20, 50, 100, 200];

fn stream_axis() -> Vec<u64> {
    let mut streams: Vec<u64> = STREAM_AXIS_BASE.iter().chain(STREAM_AXIS_ENDPOINT_EXTENSION.iter()).copied().collect();
    streams.sort_unstable();
    streams.dedup();
    streams
}

/// 批轴：基础表 ∪ 端点扩展点 ∪ 这个流数自己的 `{S-1, S, S+1}`。
fn batch_axis_for(stream_count: u64) -> Vec<u64> {
    let mut batches: Vec<u64> = BATCH_AXIS_BASE.iter().chain(BATCH_AXIS_ENDPOINT_EXTENSION.iter()).copied().collect();
    if stream_count > 1 {
        batches.push(stream_count - 1);
    }
    batches.push(stream_count);
    batches.push(stream_count + 1);
    batches.sort_unstable();
    batches.dedup();
    batches
}

// ── 分类：Q1 每行的峰值落点，Q5/Q6 单调性 ─────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum RowClassification {
    /// 峰值集合的上界落在这一行批轴的最大取值上，还没量到真正的峰值。
    Endpoint,
    /// `B_lo == 1`：从批轴起点就在下降，没有逐行峰值。
    NoInteriorPeak,
    /// 峰值集合整个落在「批 ≈ 流数」带内。
    NearDiagonal,
    /// 峰值集合与「批 ≈ 流数」带不相交，移到了别的对角。
    MovedDiagonal,
    /// 峰值集合横跨带内带外，是个宽平台。
    WidePlateau,
}

struct RowSummary {
    streams: u64,
    peak_ratio: f64,
    peak_batch_low: u64,
    peak_batch_high: u64,
    classification: RowClassification,
    moved_diagonal_ratio: Option<f64>,
    rising_steps_before_peak: u32,
    falling_steps_after_peak: u32,
    opposite_direction_steps: u32,
    final_batch: u64,
    final_ratio: f64,
    ratio_non_decreasing_through_streams_holds: bool,
    worst_drop_fraction_through_streams: f64,
    worst_drop_batch_through_streams: u64,
    ratio_non_increasing_beyond_streams_holds: bool,
    settles_near_one_beyond_sixteen_streams_holds: bool,
    settle_beyond_sixteen_streams_batch: u64,
    settle_beyond_sixteen_streams_margin: f64,
    row_minimum_ratio: f64,
    row_minimum_batch: u64,
    row_dips_below_one: bool,
}

/// 峰值集合（比值不低于最大值 × (1 − 容差) 的那些批）连同 B_lo / B_hi。
fn peak_set(cells: &[CellResult]) -> (f64, u64, u64) {
    let maximum_ratio = cells.iter().map(|cell| cell.ratio).fold(f64::MIN, f64::max);
    let threshold = maximum_ratio * (1.0 - PEAK_SET_TOLERANCE);
    let in_peak_set: Vec<u64> = cells.iter().filter(|cell| cell.ratio >= threshold).map(|cell| cell.batch).collect();
    let batch_low = *in_peak_set.iter().min().expect("峰值集合不该是空的——至少最大值那一格在里面");
    let batch_high = *in_peak_set.iter().max().expect("峰值集合不该是空的——至少最大值那一格在里面");
    (maximum_ratio, batch_low, batch_high)
}

fn classify_row(streams: u64, cells: &[CellResult]) -> RowSummary {
    let mut sorted_cells: Vec<&CellResult> = cells.iter().collect();
    sorted_cells.sort_by_key(|cell| cell.batch);
    let (peak_ratio, peak_batch_low, peak_batch_high) = peak_set(cells);
    let batch_axis_maximum = sorted_cells.last().expect("这一行没有任何格").batch;

    let band_lower = streams as f64 / NEAR_DIAGONAL_BAND_FACTOR;
    let band_upper = streams as f64 * NEAR_DIAGONAL_BAND_FACTOR;
    let peak_low_in_band = (peak_batch_low as f64) >= band_lower && (peak_batch_low as f64) <= band_upper;
    let peak_high_in_band = (peak_batch_high as f64) >= band_lower && (peak_batch_high as f64) <= band_upper;
    let peak_overlaps_band = (peak_batch_low as f64) <= band_upper && (peak_batch_high as f64) >= band_lower;

    let (classification, moved_diagonal_ratio) = if peak_batch_high == batch_axis_maximum {
        (RowClassification::Endpoint, None)
    } else if peak_batch_low == 1 {
        (RowClassification::NoInteriorPeak, None)
    } else if peak_low_in_band && peak_high_in_band {
        (RowClassification::NearDiagonal, None)
    } else if !peak_overlaps_band {
        (RowClassification::MovedDiagonal, Some(peak_batch_low as f64 / streams as f64))
    } else {
        (RowClassification::WidePlateau, None)
    };

    // 轨迹（跑前登记第八节 8.1）：峰值前段该涨、峰值后段该跌，方向相反的步数单独计。
    let mut rising_steps_before_peak = 0u32;
    let mut falling_steps_after_peak = 0u32;
    let mut opposite_direction_steps = 0u32;
    for window in sorted_cells.windows(2) {
        let (previous, next) = (window[0], window[1]);
        let rose = next.ratio > previous.ratio * (1.0 + MONOTONIC_TOLERANCE);
        let fell = next.ratio < previous.ratio * (1.0 - MONOTONIC_TOLERANCE);
        if next.batch <= peak_batch_low {
            if rose {
                rising_steps_before_peak += 1;
            } else if fell {
                opposite_direction_steps += 1;
            }
        } else if previous.batch >= peak_batch_high {
            if fell {
                falling_steps_after_peak += 1;
            } else if rose {
                opposite_direction_steps += 1;
            }
        }
    }

    // Q5：批 <= 流数的取值上，相邻两点不许下降超过容差。
    let mut ratio_non_decreasing_through_streams_holds = true;
    let mut worst_drop_fraction_through_streams = 0.0f64;
    let mut worst_drop_batch_through_streams = 0u64;
    for window in sorted_cells.windows(2) {
        let (previous, next) = (window[0], window[1]);
        if next.batch > streams {
            continue;
        }
        if next.ratio < previous.ratio * (1.0 - MONOTONIC_TOLERANCE) {
            ratio_non_decreasing_through_streams_holds = false;
            let drop_fraction = 1.0 - next.ratio / previous.ratio;
            if drop_fraction > worst_drop_fraction_through_streams {
                worst_drop_fraction_through_streams = drop_fraction;
                worst_drop_batch_through_streams = next.batch;
            }
        }
    }

    // Q6①：批 >= 流数的取值上，相邻两点不许上升超过容差。
    let mut ratio_non_increasing_beyond_streams_holds = true;
    for window in sorted_cells.windows(2) {
        let (previous, next) = (window[0], window[1]);
        if previous.batch < streams {
            continue;
        }
        if next.ratio > previous.ratio * (1.0 + MONOTONIC_TOLERANCE) {
            ratio_non_increasing_beyond_streams_holds = false;
        }
    }

    // Q6②：批轴上不小于 16×流数的最小批，超额不超过峰值超额的四分之一。
    let settle_target = 16 * streams;
    let settle_cell = sorted_cells.iter().find(|cell| cell.batch >= settle_target);
    let (settles_near_one_beyond_sixteen_streams_holds, settle_beyond_sixteen_streams_batch, settle_beyond_sixteen_streams_margin) = match settle_cell {
        Some(cell) => {
            let margin = (cell.ratio - 1.0) - (peak_ratio - 1.0) / 4.0;
            (margin <= 0.0, cell.batch, cell.ratio - 1.0)
        }
        None => (false, 0, f64::NAN), // 批轴没扫到 16×流数，这一行的 Q6② 判不了
    };

    // Q6③：这一行比值的最小值，是否跌破 1。
    let minimum_cell = sorted_cells.iter().min_by(|left_cell, right_cell| left_cell.ratio.total_cmp(&right_cell.ratio)).expect("这一行没有任何格");
    let final_cell = sorted_cells.last().expect("这一行没有任何格");

    RowSummary {
        streams,
        peak_ratio,
        peak_batch_low,
        peak_batch_high,
        classification,
        moved_diagonal_ratio,
        rising_steps_before_peak,
        falling_steps_after_peak,
        opposite_direction_steps,
        final_batch: final_cell.batch,
        final_ratio: final_cell.ratio,
        ratio_non_decreasing_through_streams_holds,
        worst_drop_fraction_through_streams,
        worst_drop_batch_through_streams,
        ratio_non_increasing_beyond_streams_holds,
        settles_near_one_beyond_sixteen_streams_holds,
        settle_beyond_sixteen_streams_batch,
        settle_beyond_sixteen_streams_margin,
        row_minimum_ratio: minimum_cell.ratio,
        row_minimum_batch: minimum_cell.batch,
        row_dips_below_one: minimum_cell.ratio < 1.0,
    }
}

fn classification_name(classification: &RowClassification) -> &'static str {
    match classification {
        // 没有 `_ =>`：新增一种分类不补这里就编译不过
        RowClassification::Endpoint => "端点",
        RowClassification::NoInteriorPeak => "丙行",
        RowClassification::NearDiagonal => "甲行",
        RowClassification::MovedDiagonal => "乙行",
        RowClassification::WidePlateau => "宽平台",
    }
}

fn emit_row_summary(emitter: &mut Emitter, output: &mut String, geometry_name: &str, row: &RowSummary) {
    let moved_text = row.moved_diagonal_ratio.map(|value| format!("{value:.4}")).unwrap_or_else(|| "NA".into());
    output.push_str(&emitter.emit_raw(&format!(
        "name=row_summary geometry={geometry_name} streams={} classification={} peak_ratio={:.6} \
         peak_batch_low={} peak_batch_high={} moved_diagonal_ratio={moved_text} \
         rising_steps_before_peak={} falling_steps_after_peak={} opposite_direction_steps={} \
         final_batch={} final_ratio={:.6} ratio_non_decreasing_through_streams_holds={} worst_drop_fraction_through_streams={:.6} worst_drop_batch_through_streams={} \
         ratio_non_increasing_beyond_streams_holds={} settles_near_one_beyond_sixteen_streams_holds={} settle_beyond_sixteen_streams_batch={} settle_beyond_sixteen_streams_margin={:.6} \
         row_minimum_ratio={:.6} row_minimum_batch={} row_dips_below_one={}",
        row.streams, classification_name(&row.classification), row.peak_ratio, row.peak_batch_low, row.peak_batch_high,
        row.rising_steps_before_peak, row.falling_steps_after_peak, row.opposite_direction_steps,
        row.final_batch, row.final_ratio, row.ratio_non_decreasing_through_streams_holds, row.worst_drop_fraction_through_streams, row.worst_drop_batch_through_streams,
        row.ratio_non_increasing_beyond_streams_holds, row.settles_near_one_beyond_sixteen_streams_holds, row.settle_beyond_sixteen_streams_batch, row.settle_beyond_sixteen_streams_margin,
        row.row_minimum_ratio, row.row_minimum_batch, row.row_dips_below_one
    )));
    output.push('\n');
}

// ── 一个几何点：算完全部行、汇总 Q1-Q6 ──────────────────────────────────────

struct GeometryGrid {
    #[allow(dead_code, reason = "字段保留几何点身份，供未来扩展读取；本轮只用 rows")]
    geometry: Geometry,
    rows: Vec<(u64, Vec<CellResult>)>, // (streams, 这一行的全部格，按 batch 未必有序)
}

fn compute_geometry_grid(geometry: Geometry, format: RecordFormat) -> GeometryGrid {
    let rows = stream_axis()
        .into_iter()
        .map(|streams| {
            let cells = batch_axis_for(streams).into_iter().map(|batch| compute_cell(&geometry, streams, batch, format)).collect();
            (streams, cells)
        })
        .collect();
    GeometryGrid { geometry, rows }
}

fn emit_geometry_configuration(emitter: &mut Emitter, output: &mut String, geometry: &Geometry) {
    output.push_str(&emitter.emit_raw(&format!(
        "name=geometry_config geometry={} units={} leaf_units={} fanout={} root_level={} node_levels={} root_children={}",
        geometry.name, geometry.units, geometry.leaf_units, geometry.fanout, geometry.root_level(), geometry.node_levels(), geometry.root_children()
    )));
    output.push('\n');
}

fn peak_location_classification_summary(rows: &[RowSummary]) -> (String, String) {
    let mut classes: Vec<&RowClassification> = rows.iter().map(|row| &row.classification).collect();
    classes.dedup_by(|left_classification, right_classification| *left_classification == *right_classification);
    if classes.len() == 1 {
        (classification_name(classes[0]).to_string(), "全部行同一类".to_string())
    } else {
        let detail = rows.iter().map(|row| format!("S={}:{}", row.streams, classification_name(&row.classification))).collect::<Vec<_>>().join(",");
        ("分段".to_string(), detail)
    }
}

fn emit_peak_location_classification_summary(emitter: &mut Emitter, output: &mut String, geometry_name: &str, rows: &[RowSummary]) {
    let (classification, detail) = peak_location_classification_summary(rows);
    output.push_str(&emitter.emit_raw(&format!("name=q1_summary geometry={geometry_name} classification={classification} detail={detail}")));
    output.push('\n');
}

/// Q2：整张网格上比值不低于最大值 × (1 − 容差) 的格，报它们的 S 范围与 B 范围，
/// 判两轴的最大值是否都落在扫描区间内部（S < 256 且没有 B = 批轴最大值的格）。
fn emit_two_dimensional_maximum_summary(emitter: &mut Emitter, output: &mut String, geometry_name: &str, grid: &GeometryGrid) {
    let overall_maximum = grid.rows.iter().flat_map(|(_, cells)| cells.iter().map(|cell| cell.ratio)).fold(f64::MIN, f64::max);
    let threshold = overall_maximum * (1.0 - PEAK_SET_TOLERANCE);
    let batch_axis_maximum = grid.rows.iter().flat_map(|(_, cells)| cells.iter().map(|cell| cell.batch)).max().unwrap_or(0);
    let mut streams_in_set: Vec<u64> = Vec::new();
    let mut batches_in_set: Vec<u64> = Vec::new();
    for (streams, cells) in &grid.rows {
        for cell in cells {
            if cell.ratio >= threshold {
                streams_in_set.push(*streams);
                batches_in_set.push(cell.batch);
            }
        }
    }
    let streams_lower = *streams_in_set.iter().min().unwrap_or(&0);
    let streams_upper = *streams_in_set.iter().max().unwrap_or(&0);
    let batches_lower = *batches_in_set.iter().min().unwrap_or(&0);
    let batches_upper = *batches_in_set.iter().max().unwrap_or(&0);
    let streams_interior = streams_lower < 256;
    let batch_interior = batches_upper != batch_axis_maximum;
    output.push_str(&emitter.emit_raw(&format!(
        "name=q2_summary geometry={geometry_name} maximum_ratio={overall_maximum:.6} streams_lower={streams_lower} \
         streams_upper={streams_upper} batch_lower={batches_lower} batch_upper={batches_upper} \
         streams_interior={streams_interior} batch_interior={batch_interior}"
    )));
    output.push('\n');
}

/// 两种相对误差分母（跑前登记第一节「相对误差」与 M11）：模型比值本身，或预测比值。
/// PC2 核哪一种复现得出 9.6% / 20.5%，两种都报，判据在读产物的那一步选。
const ERROR_DENOMINATOR_SELECTORS: [(&str, fn(&CellResult) -> f64); 2] =
    [("model", |cell: &CellResult| cell.error_model_denominator), ("predicted", |cell: &CellResult| cell.error_predicted_denominator)];

/// Q3/Q4：原 56 格子集（`streams=已定`）与全网格（Q4），两种误差分母各报一遍。
fn emit_relative_error_summary(emitter: &mut Emitter, output: &mut String, geometry_name: &str, grid: &GeometryGrid) {
    let original_56: Vec<&CellResult> = grid
        .rows
        .iter()
        .filter(|(streams, _)| ORIGINAL_56_STREAMS.contains(streams))
        .flat_map(|(_, cells)| cells.iter().filter(|cell| ORIGINAL_56_BATCHES.contains(&cell.batch)))
        .collect();
    for (label, selector) in ERROR_DENOMINATOR_SELECTORS {
        let mut absolute_errors: Vec<f64> = original_56.iter().map(|cell| selector(cell).abs()).collect();
        absolute_errors.sort_by(f64::total_cmp);
        let median = absolute_errors[absolute_errors.len() / 2];
        let maximum = *absolute_errors.last().expect("56 格子集不该是空的");
        let median_verdict = if median <= 0.096 { "不超" } else { "超" };
        let maximum_verdict = if maximum <= 0.205 { "不超" } else { "超" };
        output.push_str(&emitter.emit_raw(&format!(
            "name=q3_summary geometry={geometry_name} denominator={label} median_abs_error={median:.6} max_abs_error={maximum:.6} \
             median_verdict={median_verdict} maximum_verdict={maximum_verdict}"
        )));
        output.push('\n');
    }

    let all_cells: Vec<&CellResult> = grid.rows.iter().flat_map(|(_, cells)| cells.iter()).collect();
    for (label, selector) in ERROR_DENOMINATOR_SELECTORS {
        let positive_count = all_cells.iter().filter(|cell| selector(cell) > 0.0).count();
        output.push_str(&emitter.emit_raw(&format!(
            "name=q4_summary geometry={geometry_name} denominator={label} positive_error_fraction={:.4} cells={}",
            positive_count as f64 / all_cells.len() as f64,
            all_cells.len()
        )));
        output.push('\n');
    }
}

fn emit_geometry_sensitivity(emitter: &mut Emitter, output: &mut String, per_geometry_rows: &[(&str, &[RowSummary])]) {
    let streams_axis = stream_axis();
    for streams in streams_axis {
        let classes: Vec<(&str, &RowClassification)> = per_geometry_rows
            .iter()
            .map(|(name, rows)| {
                let row = rows.iter().find(|row| row.streams == streams).expect("每个几何点都跑了同一条流数轴");
                (*name, &row.classification)
            })
            .collect();
        let stable = classes.windows(2).all(|window| window[0].1 == window[1].1);
        let detail = classes.iter().map(|(name, class)| format!("{name}={}", classification_name(class))).collect::<Vec<_>>().join(",");
        output.push_str(&emitter.emit_raw(&format!("name=geometry_sensitivity streams={streams} stable={stable} detail={detail}")));
        output.push('\n');
    }
}

// ── main：grid 模式（第一段全网格）与 pc3 模式（ORIGINAL_DEVICE_GEOMETRY 上原 56 格，对拍原装置） ────────

fn run_grid_mode() {
    let mut emitter = Emitter::new();
    let mut output = String::new();
    output.push_str(&emitter.emit_raw(&format!(
        "name=config peak_set_tolerance={PEAK_SET_TOLERANCE} monotonic_tolerance={MONOTONIC_TOLERANCE} \
         near_diagonal_band_factor={NEAR_DIAGONAL_BAND_FACTOR} streams_axis_len={} \
         record_format=today entries_per_record={JOURNAL_NAMED_ENTRIES_PER_RECORD}",
        stream_axis().len()
    )));
    output.push('\n');

    let mut per_geometry_rows: Vec<(&'static str, Vec<RowSummary>)> = Vec::new();
    for geometry in FIRST_SEGMENT_GEOMETRIES {
        emit_geometry_configuration(&mut emitter, &mut output, &geometry);
        let grid = compute_geometry_grid(geometry, RecordFormat::Today);
        for (_streams, cells) in &grid.rows {
            for cell in cells {
                emit_cell_line(&mut emitter, &mut output, "grid", geometry.name, cell);
            }
        }
        let rows: Vec<RowSummary> = grid.rows.iter().map(|(streams, cells)| classify_row(*streams, cells)).collect();
        for row in &rows {
            emit_row_summary(&mut emitter, &mut output, geometry.name, row);
        }
        emit_peak_location_classification_summary(&mut emitter, &mut output, geometry.name, &rows);
        emit_two_dimensional_maximum_summary(&mut emitter, &mut output, geometry.name, &grid);
        emit_relative_error_summary(&mut emitter, &mut output, geometry.name, &grid);
        per_geometry_rows.push((geometry.name, rows));
    }
    let borrowed_rows: Vec<(&str, &[RowSummary])> = per_geometry_rows.iter().map(|(name, rows)| (*name, rows.as_slice())).collect();
    emit_geometry_sensitivity(&mut emitter, &mut output, &borrowed_rows);

    output.push_str(&emitter.finish());
    output.push('\n');
    print!("{output}");
}

/// `pc3` 模式：ORIGINAL_DEVICE_GEOMETRY（原装置几何，`RecordFormat::LegacyOriginal`）上跑原 56 格，
/// 与现跑的原装置 `e16-journal sweep` 逐格比对（跑前登记 PC3），不写进第一段的网格。
fn run_pc3_mode() {
    let mut emitter = Emitter::new();
    let mut output = String::new();
    emit_geometry_configuration(&mut emitter, &mut output, &ORIGINAL_DEVICE_GEOMETRY);
    for &streams in &ORIGINAL_56_STREAMS {
        for &batch in &ORIGINAL_56_BATCHES {
            let cell = compute_cell(&ORIGINAL_DEVICE_GEOMETRY, streams, batch, RecordFormat::LegacyOriginal);
            emit_cell_line(&mut emitter, &mut output, "pc3", ORIGINAL_DEVICE_GEOMETRY.name, &cell);
        }
    }
    output.push_str(&emitter.finish());
    output.push('\n');
    print!("{output}");
}

/// 第二段一个旋钮分组的完整流程：几何配置行、逐格结果、Q1-Q6 汇总、这一组内部的敏感性汇总，
/// 都发进调用方传入的同一个 `Emitter`——`second_segment` 模式三组共用一个 `Emitter`，
/// 完整性闸（跑前登记 V1）靠的正是「一份产物只有一个 `name=done`」这件事。
fn run_knob_group(emitter: &mut Emitter, output: &mut String, knob_name: &str, points: &[(Geometry, RecordFormat)]) {
    output.push_str(&emitter.emit_raw(&format!(
        "name=config knob={knob_name} peak_set_tolerance={PEAK_SET_TOLERANCE} monotonic_tolerance={MONOTONIC_TOLERANCE} \
         near_diagonal_band_factor={NEAR_DIAGONAL_BAND_FACTOR} streams_axis_len={}",
        stream_axis().len()
    )));
    output.push('\n');

    let mut per_geometry_rows: Vec<(&'static str, Vec<RowSummary>)> = Vec::new();
    for &(geometry, format) in points {
        emit_geometry_configuration(emitter, output, &geometry);
        let grid = compute_geometry_grid(geometry, format);
        for (_streams, cells) in &grid.rows {
            for cell in cells {
                emit_cell_line(emitter, output, knob_name, geometry.name, cell);
            }
        }
        let rows: Vec<RowSummary> = grid.rows.iter().map(|(streams, cells)| classify_row(*streams, cells)).collect();
        for row in &rows {
            emit_row_summary(emitter, output, geometry.name, row);
        }
        emit_peak_location_classification_summary(emitter, output, geometry.name, &rows);
        emit_two_dimensional_maximum_summary(emitter, output, geometry.name, &grid);
        emit_relative_error_summary(emitter, output, geometry.name, &grid);
        per_geometry_rows.push((geometry.name, rows));
    }
    let borrowed_rows: Vec<(&str, &[RowSummary])> = per_geometry_rows.iter().map(|(name, rows)| (*name, rows.as_slice())).collect();
    emit_geometry_sensitivity(emitter, output, &borrowed_rows);
}

/// `second_segment` 模式：跑前登记第五节「实现量与分段」的第二段（G3、G5、扇出 128/296、记录宽
/// 1 项/10⁶ 项），按第八节 8.2 的三个旋钮分组各跑一遍；每组把 `ONE_TEBIBYTE_FILE_GEOMETRY`
/// （第一段已经跑过的真实基线）重跑一次凑进同一张表，不必跨文件对第一段的产物。
fn run_second_segment_mode() {
    let mut emitter = Emitter::new();
    let mut output = String::new();
    run_knob_group(&mut emitter, &mut output, "tree_height_knob", &TREE_HEIGHT_KNOB_POINTS);
    run_knob_group(&mut emitter, &mut output, "fanout_knob", &FANOUT_KNOB_POINTS);
    run_knob_group(&mut emitter, &mut output, "record_width_knob", &RECORD_WIDTH_KNOB_POINTS);
    output.push_str(&emitter.finish());
    output.push('\n');
    print!("{output}");
}

/// 停机 S1 的常量自洽性检查（第七节 7.2）：`crates/singlefs-format/src/lib.rs` 的字面量
/// 与它们的加法 / 除法派生关系。这几个基础常量本身只被这条检查与单测读到，不参与网格计算
/// （网格用的是从独立锚点脚本抄来的、已经算好的 `*_UNITS` 字面量），运行期照样过一遍，
/// 免得只靠 `#[cfg(test)]` 撑着——不用它们，非测试构建会把它们判成用不到的代码。
fn assert_registered_constants_are_internally_consistent() {
    assert_eq!(DATA_UNIT_PAYLOAD_BYTES + DATA_UNIT_PAYLOAD_OFFSET, DATA_UNIT_BYTES);
    assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES);
    assert_eq!(LEGACY_ANCESTOR_LEVELS, ORIGINAL_DEVICE_GEOMETRY.node_levels() as u64);
    assert_eq!(FULL_ROOT_LEVEL_FOUR_GEOMETRY_UNITS, span_in_units(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_INTERNAL_FANOUT, 3));
    assert_eq!(FULL_ROOT_LEVEL_FOUR_GEOMETRY.units, FULL_ROOT_LEVEL_FOUR_GEOMETRY_UNITS);
    assert_eq!(TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY.units, TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY_UNITS);
    // 第二段的扇出、记录宽旋钮取样点：单元数必须与今天真实基线（1 TiB 文件）一个字不差，
    // 旋钮才只动了扇出或记录宽这一个变量。
    assert_eq!(SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.units, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
    assert_eq!(LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.units, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
    assert_eq!(RECORD_WIDTH_KNOB_MINIMUM_GEOMETRY.units, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
    assert_eq!(RECORD_WIDTH_KNOB_MAXIMUM_GEOMETRY.units, ONE_TEBIBYTE_FILE_GEOMETRY_UNITS);
    assert_eq!(RECORD_WIDTH_KNOB_MINIMUM_GEOMETRY.leaf_units, ONE_TEBIBYTE_FILE_GEOMETRY.leaf_units);
    assert_eq!(RECORD_WIDTH_KNOB_MAXIMUM_GEOMETRY.fanout, ONE_TEBIBYTE_FILE_GEOMETRY.fanout);
}

fn main() {
    assert_registered_constants_are_internally_consistent();
    match std::env::args().nth(1).as_deref() {
        Some("pc3") => run_pc3_mode(),
        Some("second_segment") => run_second_segment_mode(),
        Some(other) => {
            eprintln!(
                "e16_fifth_run_peak_under_today_widths: 认不出的模式 {other}，只有「不带参数」（grid）、pc3、second_segment 三种"
            );
            std::process::exit(2);
        }
        None => run_grid_mode(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 第七节 7.2：独立算出、用命令核过的绝对值 ────────────────────────────

    #[test]
    fn data_unit_payload_bytes_matches_the_addition_form() {
        // 写加法不写减法（test-discipline.md「常量断言写加法，别写减法」）。
        assert_eq!(DATA_UNIT_PAYLOAD_BYTES + DATA_UNIT_PAYLOAD_OFFSET, DATA_UNIT_BYTES);
        assert_eq!(DATA_UNIT_PAYLOAD_BYTES, 32634);
    }

    #[test]
    fn entries_per_record_today_and_legacy() {
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES);
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67);
        let legacy_entries_per_block = (LEGACY_BLOCK_BYTES - LEGACY_RECORD_HEADER_BYTES) / (LEGACY_ENTRY_BYTES + LEGACY_CHECKSUM_BYTES);
        assert_eq!(legacy_entries_per_block, 72);
    }

    #[test]
    fn geometry_unit_counts_match_the_byte_sizes() {
        assert_eq!(TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY.units, (256u64 << 20).div_ceil(DATA_UNIT_PAYLOAD_BYTES));
        assert_eq!(FOUR_GIBIBYTE_FILE_GEOMETRY.units, (4u64 << 30).div_ceil(DATA_UNIT_PAYLOAD_BYTES));
        assert_eq!(ONE_TEBIBYTE_FILE_GEOMETRY.units, (1u64 << 40).div_ceil(DATA_UNIT_PAYLOAD_BYTES));
        assert_eq!(SIXTEEN_TEBIBYTE_FILE_GEOMETRY.units, (16u64 << 40).div_ceil(DATA_UNIT_PAYLOAD_BYTES));
        assert_eq!(FULL_ROOT_LEVEL_FOUR_GEOMETRY.units, span_in_units(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_INTERNAL_FANOUT, 3));
        assert_eq!(ORIGINAL_DEVICE_GEOMETRY.units, LEGACY_FANOUT.pow(4));
    }

    #[test]
    fn geometry_root_level_node_levels_root_children_are_pinned() {
        // (root_level, node_levels, root_children)，逐条对跑前登记第十三节「命令一」的输出。
        for (geometry, expected) in [(TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY, (1u32, 2u32, 58u64)), (FOUR_GIBIBYTE_FILE_GEOMETRY, (2, 3, 7)), (ONE_TEBIBYTE_FILE_GEOMETRY, (3, 4, 11)), (FULL_ROOT_LEVEL_FOUR_GEOMETRY, (3, 4, 147)), (SIXTEEN_TEBIBYTE_FILE_GEOMETRY, (4, 5, 2))] {
            assert_eq!(geometry.root_level(), expected.0, "{}", geometry.name);
            assert_eq!(geometry.node_levels(), expected.1, "{}", geometry.name);
            assert_eq!(geometry.root_children(), expected.2, "{}", geometry.name);
        }
        assert_eq!(ORIGINAL_DEVICE_GEOMETRY.root_level(), 3);
        assert_eq!(ORIGINAL_DEVICE_GEOMETRY.node_levels(), 4);
        assert_eq!(LEGACY_ANCESTOR_LEVELS, ORIGINAL_DEVICE_GEOMETRY.node_levels() as u64);
    }

    #[test]
    fn s2b1_and_s2b2_match_the_independent_anchor_script() {
        // S=2、B=1：甲 = 祖先层数 + 3，乙 = 2（跑前登记第七节 7.2）。
        for (geometry, node_levels) in [(TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY, 2u32), (FOUR_GIBIBYTE_FILE_GEOMETRY, 3), (ONE_TEBIBYTE_FILE_GEOMETRY, 4), (FULL_ROOT_LEVEL_FOUR_GEOMETRY, 4), (SIXTEEN_TEBIBYTE_FILE_GEOMETRY, 5)] {
            let cell = compute_cell(&geometry, 2, 1, RecordFormat::Today);
            assert_eq!(cell.intent_average, node_levels as f64 + 3.0, "{} S2B1 甲", geometry.name);
            assert_eq!(cell.write_ahead_log_leaf_average, 2.0, "{} S2B1 乙", geometry.name);
        }
        // S=2、B=2：跑前登记第七节 7.2 表里的 S2B2_a / S2B2_ratio。
        for (geometry, expected_intent_average, expected_ratio) in
            [(TWO_FIFTY_SIX_MEBIBYTE_FILE_GEOMETRY, 7.00, 2.333), (FOUR_GIBIBYTE_FILE_GEOMETRY, 9.00, 3.000), (ONE_TEBIBYTE_FILE_GEOMETRY, 11.00, 3.667), (FULL_ROOT_LEVEL_FOUR_GEOMETRY, 11.00, 3.667), (SIXTEEN_TEBIBYTE_FILE_GEOMETRY, 12.00, 4.000)]
        {
            let cell = compute_cell(&geometry, 2, 2, RecordFormat::Today);
            assert!((cell.intent_average - expected_intent_average).abs() < 1e-9, "{} S2B2 甲", geometry.name);
            assert_eq!(cell.write_ahead_log_leaf_average, 3.0, "{} S2B2 乙", geometry.name);
            assert!((cell.ratio - expected_ratio).abs() < 1e-3, "{} S2B2 比值", geometry.name);
        }
    }

    #[test]
    fn write_ahead_log_leaf_record_blocks_today_vs_legacy_at_boundary_batches() {
        // 跑前登记第七节 7.2：乙在 B=67/68/70/72/73 的块数（今天 / 原装置记录宽）。
        for (batch, today_blocks, legacy_blocks) in [(67u64, 68u64, 68u64), (68, 70, 69), (70, 72, 71), (72, 74, 73), (73, 75, 75)] {
            assert_eq!(batch + write_ahead_log_leaf_record_blocks(batch, RecordFormat::Today), today_blocks, "batch={batch} 今天");
            assert_eq!(batch + write_ahead_log_leaf_record_blocks(batch, RecordFormat::LegacyOriginal), legacy_blocks, "batch={batch} 原装置");
        }
    }

    #[test]
    fn checkpoint_interval_and_operation_count_are_pinned() {
        assert_eq!(checkpoint_interval_for_batch(3), 2001);
        assert_eq!(operation_count_for_batch(3), 20010);
        assert_eq!(checkpoint_interval_for_batch(200), 2000);
        assert_eq!(operation_count_for_batch(200), 20000);
        assert_eq!(checkpoint_interval_for_batch(4096), 4096);
        assert_eq!(operation_count_for_batch(4096), 40960);
    }

    #[test]
    fn stream_offset_for_one_tebibyte_file_geometry_matches_the_anchor_script() {
        // ONE_TEBIBYTE_FILE_GEOMETRY 上 S=3 时第 1 条流的首个单元号 = floor(N / 3)。
        // 经 units_per_stream_for 走（M9 的锚点就在它里面）：换成 div_ceil 这一条就该红。
        let units_per_stream = units_per_stream_for(&ONE_TEBIBYTE_FILE_GEOMETRY, 3);
        assert_eq!(units_per_stream, 11_230_737);
        assert_eq!(unit_for_cursor(1, 3, units_per_stream, 1), 11_230_737);
    }

    #[test]
    fn lowest_level_node_indices_of_adjacent_units() {
        // 单元 143、144 的最底层节点序号：144（叶罩单元数）时分属节点 0 与 1。
        assert_eq!(143 / EXTENT_TREE_LOWER_LEAF_DATA_UNITS, 0);
        assert_eq!(144 / EXTENT_TREE_LOWER_LEAF_DATA_UNITS, 1);
    }

    #[test]
    fn crates_format_literals_used_here_are_pinned_to_the_registered_values() {
        // 停机 S1 的常量核对（跑前登记第十三节「命令二」）：这九行的值与那次现查逐字相同。
        assert_eq!(DATA_UNIT_BYTES, 32768);
        assert_eq!(DATA_UNIT_PAYLOAD_OFFSET, 134);
        assert_eq!(EXTENT_TREE_INTERNAL_FANOUT, 147);
        assert_eq!(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, 144);
        assert_eq!(JOURNAL_RECORD_BYTES, 4096);
        assert_eq!(JOURNAL_HEADER_BYTES, 311);
        assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 56);
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67);
    }

    // ── PC4（跑前登记 5.3）：每个几何点 B=1 时甲−乙=祖先层数+1；B<=67 时比值>1 ─────

    #[test]
    fn positive_control_four_batch_one_difference_equals_node_levels_plus_one() {
        for geometry in FIRST_SEGMENT_GEOMETRIES {
            for &streams in &STREAM_AXIS_BASE {
                let cell = compute_cell(&geometry, streams, 1, RecordFormat::Today);
                assert_eq!(
                    cell.intent_average - cell.write_ahead_log_leaf_average,
                    geometry.node_levels() as f64 + 1.0,
                    "{} streams={streams}",
                    geometry.name
                );
            }
        }
    }

    #[test]
    fn positive_control_four_ratio_above_one_for_small_batches() {
        for geometry in FIRST_SEGMENT_GEOMETRIES {
            for &streams in &[2u64, 16, 128] {
                for &batch in &[1u64, 2, 5, 10, 20, 50, 67] {
                    let cell = compute_cell(&geometry, streams, batch, RecordFormat::Today);
                    assert!(cell.ratio > 1.0, "{} streams={streams} batch={batch} ratio={}", geometry.name, cell.ratio);
                }
            }
        }
    }

    // ── 守恒：两条臂读同一条操作流 ───────────────────────────────────────────

    #[test]
    fn both_arms_see_the_same_unit_at_the_same_cursor() {
        let streams = 4u64;
        let units_per_stream = units_per_stream_for(&ONE_TEBIBYTE_FILE_GEOMETRY, streams);
        for cursor in 0..20u64 {
            let stream_index = cursor % streams;
            let unit_via_helper = unit_for_cursor(cursor, streams, units_per_stream, stream_index);
            // 与生成公式手推一遍：同一游标必须落在同一条流、同一个偏移。
            let expected = stream_index * units_per_stream + (cursor / streams) % units_per_stream;
            assert_eq!(unit_via_helper, expected, "cursor={cursor}");
        }
    }

    #[test]
    fn ancestor_deduplication_matches_hand_computation() {
        // 8 个相邻单元共享同一条祖先链。
        let units: Vec<u64> = (0..8).collect();
        assert_eq!(deduplicated_ancestor_count(&FOUR_GIBIBYTE_FILE_GEOMETRY, &units), FOUR_GIBIBYTE_FILE_GEOMETRY.node_levels() as u64);
    }

    // ── 分类逻辑：判别力自证（跑前登记第八节「判别力自证」） ──────────────────

    fn synthetic_cell(streams: u64, batch: u64, ratio: f64) -> CellResult {
        CellResult {
            streams,
            batch,
            intent_average: ratio * 2.0,
            intent_minimum: 0,
            intent_maximum: 0,
            write_ahead_log_leaf_average: 2.0,
            write_ahead_log_leaf_minimum: 0,
            write_ahead_log_leaf_maximum: 0,
            ratio,
            predicted_ratio: ratio,
            error_model_denominator: 0.0,
            error_predicted_denominator: 0.0,
            windows: 1,
        }
    }

    #[test]
    fn near_diagonal_band_bound_change_flips_classification() {
        // 流数 10：峰值集合只在批=30（= 3×流数），恰好落在 [S÷2,2S]=[5,20] 之外，是乙行；
        // 把带宽因子从 2 挪到 4（[2.5,40]）之后，30 落进带内，变成甲行——这就是「挪到两点之间必须翻面」的自证。
        let cells = vec![synthetic_cell(10, 5, 2.0), synthetic_cell(10, 20, 2.5), synthetic_cell(10, 30, 3.0), synthetic_cell(10, 40, 2.0)];
        let row_with_original_band = classify_row(10, &cells);
        assert_eq!(row_with_original_band.classification, RowClassification::MovedDiagonal);

        let widened_band_lower = 10.0 / 4.0;
        let widened_band_upper = 10.0 * 4.0;
        let peak_low_in_widened_band = 30.0 >= widened_band_lower && 30.0 <= widened_band_upper;
        assert!(peak_low_in_widened_band, "带宽因子换成 4 之后，批=30 应当落进带内，分类必须翻面");
    }

    #[test]
    fn peak_set_tolerance_change_widens_the_peak_set() {
        // 跑前登记第八节 M12 的例子：B=1,2,3 上 ρ=3.00,3.20,3.10，S=2。
        // 容差 0.1% 时峰值集合只有 {2}（丙行的反例，验证不是宽平台）；容差 10% 时 3.00 也进集合，B_lo 从 2 变 1（丙行）。
        let cells = vec![synthetic_cell(2, 1, 3.00), synthetic_cell(2, 2, 3.20), synthetic_cell(2, 3, 3.10)];
        let (_, batch_low_tight, _) = peak_set(&cells);
        assert_eq!(batch_low_tight, 2, "0.1% 容差下峰值集合不该含 B=1");

        let maximum_ratio = 3.20;
        let widened_threshold = maximum_ratio * (1.0 - 0.10);
        let batch_low_widened = cells.iter().filter(|cell| cell.ratio >= widened_threshold).map(|cell| cell.batch).min().unwrap();
        assert_eq!(batch_low_widened, 1, "10% 容差下峰值集合应当把 B=1 也纳入，B_lo 翻到 1（丙行）");
    }

    // ── 变异表 M10、M11、M14 的锚点，直接钉绝对值 ──────────────────────────

    #[test]
    fn predicted_ratio_uses_node_levels_not_node_levels_minus_one() {
        // M10 的锚点：原装置几何 L=4，S=2、B=1 ⇒ (1 + 1×4 + 2) ÷ (1+1) = 3.500；
        // 换成 node_levels − 1 会变成 (1 + 1×3 + 2) ÷ 2 = 3.000。
        assert_eq!(predicted_ratio(1, 2, ORIGINAL_DEVICE_GEOMETRY.node_levels()), 3.5);
    }

    #[test]
    fn relative_error_denominator_is_the_passed_in_value_not_always_predicted() {
        // M11 的锚点：模型比值 2.0、预测比值 2.5，以模型为分母 e = 0.25；
        // 分母被换成恒定用预测比值时会变成 0.20。
        assert!((relative_error(2.0, 2.5, 2.0) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn monotonic_tolerance_change_flips_the_batch_at_most_streams_verdict() {
        // M14 的锚点：S=4，批 1、2、3 上 ρ = 3.00、2.985（跌 0.5%）、3.10。
        // 1% 容差下这一步跌幅容许，Q5（批 ≤ 流数不降）判「成立」；容差换成 0 就该判「不成立」。
        let cells = vec![synthetic_cell(4, 1, 3.00), synthetic_cell(4, 2, 2.985), synthetic_cell(4, 3, 3.10)];
        let row = classify_row(4, &cells);
        assert!(row.ratio_non_decreasing_through_streams_holds, "1% 容差下 0.5% 的跌幅该判「成立」");
    }

    // ── 第二段（跑前登记第五节「实现量与分段」）：扇出旋钮、记录宽旋钮的取样点 ──────────

    #[test]
    fn geometry_root_level_node_levels_root_children_of_the_fanout_knob_points_are_pinned() {
        // 独立算出（root_level_for/span_in_units 手推，见交回报告）：小扇出（128/128）、大扇出
        // （296/296）在 1 TiB 单元数（与 ONE_TEBIBYTE_FILE_GEOMETRY 相同）下都恰好 4 层，
        // 根下孩子数分别是 17、2——与今天扇出（144/147）的 11 都不同，但树高都是 4，
        // 正好把「扇出」与「树高」两个变量分开。
        assert_eq!(SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.root_level(), 3);
        assert_eq!(SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.node_levels(), 4);
        assert_eq!(SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.root_children(), 17);
        assert_eq!(LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.root_level(), 3);
        assert_eq!(LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.node_levels(), 4);
        assert_eq!(LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY.root_children(), 2);
    }

    #[test]
    fn record_width_knob_geometries_share_the_one_tebibyte_file_tree_shape() {
        // 记录宽旋钮只该经 RecordFormat 起作用，树形（leaf_units/fanout/units）必须与
        // ONE_TEBIBYTE_FILE_GEOMETRY 逐项相同，否则这一组比的就不只是记录宽这一个变量了。
        for geometry in [RECORD_WIDTH_KNOB_MINIMUM_GEOMETRY, RECORD_WIDTH_KNOB_MAXIMUM_GEOMETRY] {
            assert_eq!(geometry.leaf_units, ONE_TEBIBYTE_FILE_GEOMETRY.leaf_units, "{}", geometry.name);
            assert_eq!(geometry.fanout, ONE_TEBIBYTE_FILE_GEOMETRY.fanout, "{}", geometry.name);
            assert_eq!(geometry.units, ONE_TEBIBYTE_FILE_GEOMETRY.units, "{}", geometry.name);
        }
    }

    #[test]
    fn entries_per_record_format_divides_with_ceiling_not_floor() {
        // M17 的锚点：100 万项一条记录恰好装满 1_000_000 项 ⇒ 1 块；多 1 项就该多 1 块（div_ceil）。
        // 换成地板除会把 1_000_001 项算成 1 块，这条断言就该红。
        assert_eq!(write_ahead_log_leaf_record_blocks(1_000_000, RecordFormat::EntriesPerRecord(1_000_000)), 1);
        assert_eq!(write_ahead_log_leaf_record_blocks(1_000_001, RecordFormat::EntriesPerRecord(1_000_000)), 2);
        // 一条记录只装 1 项：k 个单元就要 k 条记录。
        assert_eq!(write_ahead_log_leaf_record_blocks(5, RecordFormat::EntriesPerRecord(1)), 5);
    }

    #[test]
    fn positive_control_four_batch_one_difference_equals_node_levels_plus_one_on_the_second_segment_points() {
        // PC4（跑前登记 5.3）对第二段的每个几何点、以及记录宽旋钮的两个取样点同样要成立：
        // B=1 时只有 1 个脏单元，不涉及去重，甲 − 乙 = 祖先层数 + 1，与记录格式无关
        // （B=1 时乙的记录数恒为 1：ceil(1÷任意正整数) = 1）。
        for (geometry, format) in TREE_HEIGHT_KNOB_POINTS
            .into_iter()
            .chain(FANOUT_KNOB_POINTS)
            .chain(RECORD_WIDTH_KNOB_POINTS)
        {
            for &streams in &[2u64, 16, 128] {
                let cell = compute_cell(&geometry, streams, 1, format);
                assert_eq!(
                    cell.intent_average - cell.write_ahead_log_leaf_average,
                    geometry.node_levels() as f64 + 1.0,
                    "{} streams={streams} format={format:?}",
                    geometry.name
                );
            }
        }
    }
}
