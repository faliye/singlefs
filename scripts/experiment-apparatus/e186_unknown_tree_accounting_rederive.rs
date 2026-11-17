//! E186：不认识的树·记账维重推 —— 旧写者遇到不认识的树时，记账那一维（`Carry` / `NoCarry` / 随根留着）与 flags 那一维
//! （五条臂）在今天的记账条款下怎么排；E113（不认识的树·补齐臂集） 的字典序在「每代写行」之前补一位「要改几条已定条款」。
//!
//! 跑前登记：`research/prompts/e186-preregistration.md`。读法写死：读法 B 取 B1（D8 已定项 8 的 ㊀ ㊁ 不进条款清单），
//! 读法 W 取 W2（每代写行 = 这一版记账树里的全部行）；B2、W1 只作附带量照报、不判。
//! 独立手写模型，不与 `crates/` 共用代码；与 `crates/` 逐项对拍只做登记第十一节停机条款的三项（行数、台阶行认法、I-3.13 的语法认法）。
//!
//! ## 谓词直接引的条款句子（整段原文在跑前登记第二节）
//!
//! - D5（快照 / 空间记账机制） 已定项 2：「每条根的记账树只装这条根那一代的行，不留多代」；「例外是 inode 号水位那一统计量的台阶行：随根一直留着」。
//! - D5（快照 / 空间记账机制） 已定项 2 射程：「记账在 inode 号水位变了的每次发布给这一统计量留一行台阶行 (g, H)」「克隆头的台阶行按 (统计量, 树) 分」
//!   ⇒ 台阶行按 (统计量, 树) 认、看的是那次发布那个头的水位变没变（`WatermarkChangeHistory`），不按代段语法认。
//! - D5（快照 / 空间记账机制） 已定项 4 射程：「读一个统计量的现行值 = 取该维度元组下代号最大的那一行，空表示从未维护过、不是 0」
//!   ⇒ 丢行按维度元组数（元组里还剩任一行，现行值读得出）。
//! - I-3.13（记账行代段等于这一版的 txg）：「每一行的代段……等于这一版的 `checkpoint_txg`……inode 号水位那一统计量的台阶行不判」。
//! - I-9.11（可写克隆头有水位行）：「每个可写克隆头的树 ID 在记账里都有「inode 号水位」行」。
//!
//! ## 选法（跑前写死，字典序，越小越好）
//!
//! 1 误释放 → 2 违反语义 → 3 丢行（元组） → 4 静默 → 5 不可写代数 → 6 要改几条已定条款 → 7 每代写行。
//! 不设「挂得上才比」的筛子：十五格每一格都进比较。
//!
//! ## 第二次跑（重跑登记 `research/prompts/e186-preregistration.md` 的「第二次跑（重跑登记）」一节）
//!
//! 同一个 bin 加参数 `--judgement <vacuous-u5|dangling-u5>`：不带参数等于 `vacuous-u5`，逐行照第一次跑的判据算，
//! 产物逐字节等于第一次跑的产物；`dangling-u5` 是第二次跑的判据（新 U5 谓词与新的反向接受条款）。
//! 与第一次跑比只改这两处，其余照旧。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

/// U5 的判据（登记第二次跑第三节「装置写在哪」）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Judgement {
    /// 第一次跑的判据：写者这次为不认识的树写进去的行，代段 = 这次的 txg；没写就空真。
    VacuousCarriedRowsClause,
    /// 第二次跑的判据：那半句所说的行在这条臂下不存在 ⇒ 要改；`Carry` 下照原谓词判。
    DanglingCarriedRowsClause,
}

/// 不带参数时的判据：第一次跑的那一种，产物逐字节等于第一次跑的产物（登记第二次跑第十一节停机条款第 5 项）。
const DEFAULT_JUDGEMENT: Judgement = Judgement::VacuousCarriedRowsClause;

impl Judgement {
    fn label(self) -> &'static str {
        match self {
            Judgement::VacuousCarriedRowsClause => "vacuous-u5",
            Judgement::DanglingCarriedRowsClause => "dangling-u5",
        }
    }

    /// 命令行参数（不含程序名）：空 ⇒ 默认；`--judgement <值>` ⇒ 那一种；别的一律拒。
    fn from_arguments(arguments: &[String]) -> Result<Judgement, String> {
        match arguments {
            [] => Ok(DEFAULT_JUDGEMENT),
            [flag, value] if flag == "--judgement" => match value.as_str() {
                "vacuous-u5" => Ok(Judgement::VacuousCarriedRowsClause),
                "dangling-u5" => Ok(Judgement::DanglingCarriedRowsClause),
                other => Err(format!("--judgement 只认 vacuous-u5 或 dangling-u5，这里是 {other}")),
            },
            other => Err(format!("参数只认 --judgement <vacuous-u5|dangling-u5>，这里是 {other:?}")),
        }
    }
}

/// v1 不认识的树（E113 原值）。
const UNKNOWN_TREE_COUNT: u64 = 4;
/// 其中那个未知位取「只读」语义的。
const UNKNOWN_READ_ONLY_TREE_COUNT: u64 = 1;
/// 其中那个未知位取「半删」语义的。
const UNKNOWN_HALF_DELETED_TREE_COUNT: u64 = 1;
/// 每代出生多少块 —— 误释放数的量纲（E113 原值）。
const BLOCKS_BORN_PER_GENERATION: u64 = 64;
/// v1 认识的树（两种几何同为 12，只进配置行）。
const KNOWN_TREE_COUNT: u64 = 12;

/// E113 几何：不带树维的行 11 行（E113 的 `r_pool`）、认识的可写头 6 个、认识的头没有台阶行（E113 口径）。
const E113_ROWS_WITHOUT_TREE_DIMENSION: u64 = 11;
const E113_KNOWN_WRITABLE_HEAD_COUNT: u64 = 6;
const E113_KNOWN_HEAD_STEP_ROWS: u64 = 0;

/// 实现两盘几何，值抄自 `crates/singlefs-core/src/transaction.rs` 的 `accounting_entries_of_this_publish`：
/// 池级三行（含 inode 号水位那一行）加每块盘六行，两块盘；只有 inode 树一个可写头，它的水位有一级台阶行。
const CRATES_POOL_WIDE_ROWS: u64 = 3;
const CRATES_PER_DEVICE_ROWS: u64 = 6;
const CRATES_DEVICE_COUNT: u64 = 2;
const CRATES_KNOWN_WRITABLE_HEAD_COUNT: u64 = 1;
const CRATES_KNOWN_HEAD_STEP_ROWS: u64 = 1;
/// 池级三行里带树维的只有 inode 号水位那一行，它在模型里按「认识的可写头 × 带树维的统计量」另算。
const CRATES_POOL_WIDE_ROWS_WITH_TREE_DIMENSION: u64 = 1;

/// R 那一版（认识全部树的新写者发布）的 txg；旧写者此后发布的第 g 版 txg 是它加 g（D16 已定项 6）。
const INITIAL_VERSION_TXG: u64 = 1000;
/// 不认识的头更早的台阶行之间相隔几个 txg（只要互不相同、都早于 R 就行）。
const STEP_TXG_SPACING: u64 = 10;
/// 实现两盘几何里认识的那个头那一级台阶行的代段。
const KNOWN_HEAD_STEP_TXG: u64 = INITIAL_VERSION_TXG - 5;
const KNOWN_HEAD_WATERMARK: u64 = 500;
const UNKNOWN_HEAD_FIRST_STEP_WATERMARK: u64 = 100;
/// 不带树维的统计量与第二个带树维的统计量的值：对搬运者不透明，取什么都不影响判定。
const OPAQUE_STATISTIC_VALUE: u64 = 7;

/// Q2 读「计」时 `RejectEntry` 每一格多记的条数（要新定义的「EIO 但位置还在」语义）。
const REJECT_ENTRY_NEW_SEMANTICS_CLAUSE_UNITS: u64 = 1;
/// 第 4 位「静默」的口径（E113 那一位加第二个统计量那一项）：误释放有 I-3.1 盖，水位元组丢失有 I-9.11 盖，
/// 违反只读、复活半删、第二个统计量的元组丢失没有不变量盖。
const WRONG_FREE_HAS_AN_INVARIANT: bool = true;
const WATERMARK_TUPLE_LOSS_HAS_AN_INVARIANT: bool = true;
const SECOND_TREE_STATISTIC_LOSS_HAS_AN_INVARIANT: bool = false;

/// 登记第一节第 8 条的取样维度。`k_keep` 今天没有对象（K 代已撤回），模型不读它，只为覆盖 E113 的点。
const RETIRED_KEPT_GENERATION_COUNT_VALUES: [u64; 5] = [1, 2, 4, 8, 20];
const INVISIBLE_WINDOW_GENERATION_VALUES: [u64; 4] = [0, 1, 3, 8];
const UNKNOWN_WRITABLE_HEAD_COUNT_VALUES: [u64; 3] = [0, 2, 4];
const REGISTERED_TREE_COUNT_VALUES: [u64; 2] = [0, UNKNOWN_TREE_COUNT];
const PUBLISHED_GENERATION_VALUES: [u64; 4] = [1, 4, 5, 10];
/// (`s` 更早的台阶行数, `lpc` R 那次发布本身改没改这个头的水位)。
const STEP_HISTORY_VALUES: [(u64, bool); 6] = [(0, false), (0, true), (1, false), (1, true), (3, false), (3, true)];
const TREE_DIMENSION_STATISTIC_COUNT_VALUES: [u64; 2] = [1, 2];
const GEOMETRIES: [Geometry; 2] = [Geometry::UnknownTreeFullArms, Geometry::ImplementationTwoDevices];
const EXPECTED_POINT_COUNT: u64 = 11520;
const CELLS_PER_POINT: u64 = 15;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Geometry {
    /// E113 的几何。
    UnknownTreeFullArms,
    /// `crates/` 今天的两盘池。
    ImplementationTwoDevices,
}

impl Geometry {
    fn label(self) -> &'static str {
        match self {
            Geometry::UnknownTreeFullArms => "e113",
            Geometry::ImplementationTwoDevices => "implementation_two_devices",
        }
    }

    fn rows_without_tree_dimension(self) -> u64 {
        match self {
            Geometry::UnknownTreeFullArms => E113_ROWS_WITHOUT_TREE_DIMENSION,
            Geometry::ImplementationTwoDevices => {
                CRATES_POOL_WIDE_ROWS - CRATES_POOL_WIDE_ROWS_WITH_TREE_DIMENSION + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT
            }
        }
    }

    fn known_writable_head_count(self) -> u64 {
        match self {
            Geometry::UnknownTreeFullArms => E113_KNOWN_WRITABLE_HEAD_COUNT,
            Geometry::ImplementationTwoDevices => CRATES_KNOWN_WRITABLE_HEAD_COUNT,
        }
    }

    fn known_head_step_rows(self) -> u64 {
        match self {
            Geometry::UnknownTreeFullArms => E113_KNOWN_HEAD_STEP_ROWS,
            Geometry::ImplementationTwoDevices => CRATES_KNOWN_HEAD_STEP_ROWS,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum UnknownFlagsPolicy {
    Ignore,
    Skip,
    RejectEntry,
    ReadOnly,
    PerBit,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum AccountingPolicy {
    Carry,
    NoCarry,
    /// 随根留着：不认识的树的行原样留在记账树里，不重写、不换代号。
    KeepWithRoot,
}

const FLAGS_ARMS: [UnknownFlagsPolicy; 5] = [
    UnknownFlagsPolicy::Ignore,
    UnknownFlagsPolicy::Skip,
    UnknownFlagsPolicy::RejectEntry,
    UnknownFlagsPolicy::ReadOnly,
    UnknownFlagsPolicy::PerBit,
];
const ACCOUNTING_ARMS: [AccountingPolicy; 3] = [AccountingPolicy::Carry, AccountingPolicy::NoCarry, AccountingPolicy::KeepWithRoot];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SamplePoint {
    geometry: Geometry,
    /// 已撤回的「只留最近 K 代」的 K：模型不读它（阴性对照丙），只进产物。
    retired_kept_generation_count: u64,
    invisible_window_generations: u64,
    unknown_writable_head_count: u64,
    trees_with_registered_bit: u64,
    published_generations: u64,
    earlier_step_count: u64,
    initial_publish_changed_the_watermark: bool,
    tree_dimension_statistic_count: u64,
}

impl SamplePoint {
    /// (`s`, `lpc`) = (0, 0) 是 E113 口径：头一级台阶行都没有，按今天 D5 已定项 2 射程「建克隆那次给新头留一行」不合法。
    fn is_legal_under_todays_step_rows(&self) -> bool {
        self.earlier_step_count > 0 || self.initial_publish_changed_the_watermark
    }

    fn describe(&self) -> String {
        format!(
            "geometry={} k_keep={} w_gens={} h_new={} registered={} g_gens={} s={} lpc={} r_tree_dim={}",
            self.geometry.label(),
            self.retired_kept_generation_count,
            self.invisible_window_generations,
            self.unknown_writable_head_count,
            self.trees_with_registered_bit,
            self.published_generations,
            self.earlier_step_count,
            u8::from(self.initial_publish_changed_the_watermark),
            self.tree_dimension_statistic_count
        )
    }
}

/// 全部取样点，次序固定（几何、k_keep、w_gens、h_new、registered、G、(s, lpc)、r_tree_dim 由外到内）。
fn all_sample_points() -> Vec<SamplePoint> {
    let mut points = Vec::new();
    for geometry in GEOMETRIES {
        for retired_kept_generation_count in RETIRED_KEPT_GENERATION_COUNT_VALUES {
            for invisible_window_generations in INVISIBLE_WINDOW_GENERATION_VALUES {
                for unknown_writable_head_count in UNKNOWN_WRITABLE_HEAD_COUNT_VALUES {
                    for trees_with_registered_bit in REGISTERED_TREE_COUNT_VALUES {
                        for published_generations in PUBLISHED_GENERATION_VALUES {
                            for (earlier_step_count, initial_publish_changed_the_watermark) in STEP_HISTORY_VALUES {
                                for tree_dimension_statistic_count in TREE_DIMENSION_STATISTIC_COUNT_VALUES {
                                    points.push(SamplePoint {
                                        geometry,
                                        retired_kept_generation_count,
                                        invisible_window_generations,
                                        unknown_writable_head_count,
                                        trees_with_registered_bit,
                                        published_generations,
                                        earlier_step_count,
                                        initial_publish_changed_the_watermark,
                                        tree_dimension_statistic_count,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    points
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Statistic {
    /// 不带树维的统计量（含按盘展开的那几行），下标只为区分元组。
    WithoutTreeDimension(u64),
    /// D5 已定项 4 第 12 项，今天唯一带树维的统计量。
    InodeNumberWatermark,
    /// `r_tree_dim = 2` 时假设的第二个带树维的统计量：只有这一代的行、没有台阶行。
    SecondTreeDimensionStatistic,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TreeOwner {
    PoolWide,
    KnownHead(u64),
    UnknownHead(u64),
}

/// 字段次序就是 key 次序：(统计量, 树, 代段)，再按值。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct AccountingRow {
    statistic: Statistic,
    tree: TreeOwner,
    generation_txg: u64,
    value: u64,
}

impl AccountingRow {
    fn tuple(&self) -> (Statistic, TreeOwner) {
        (self.statistic, self.tree)
    }

    fn belongs_to_an_unknown_tree(&self) -> bool {
        matches!(self.tree, TreeOwner::UnknownHead(_))
    }

    fn recoded_to(self, generation_txg: u64) -> AccountingRow {
        AccountingRow { statistic: self.statistic, tree: self.tree, generation_txg, value: self.value }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct AccountingVersion {
    txg: u64,
    /// 按 key 升序。
    rows: Vec<AccountingRow>,
    /// 写者这一次为不认识的树新写进去的行（照抄的台阶行与原样留着的行不算），U5 判它。
    rows_written_for_unknown_trees: Vec<AccountingRow>,
}

/// D5 意义的台阶行：按 (统计量, 树) 分，那次发布那个头的水位变了才留。模型知道每个头水位在哪几次发布变过，
/// 台阶行由此认，不看代段的语法（`crates/` 今天的语法认法另写在 `steps_by_the_crates_reading`，只作附带量）。
#[derive(Clone, PartialEq, Eq, Debug)]
struct WatermarkChangeHistory {
    changed_at: BTreeSet<(TreeOwner, u64)>,
}

impl WatermarkChangeHistory {
    fn is_a_step(&self, row: &AccountingRow) -> bool {
        row.statistic == Statistic::InodeNumberWatermark && self.changed_at.contains(&(row.tree, row.generation_txg))
    }
}

fn known_writable_heads(point: &SamplePoint) -> Vec<TreeOwner> {
    (0..point.geometry.known_writable_head_count()).map(TreeOwner::KnownHead).collect()
}

fn unknown_writable_heads(point: &SamplePoint) -> Vec<TreeOwner> {
    (0..point.unknown_writable_head_count).map(TreeOwner::UnknownHead).collect()
}

fn tree_dimension_statistics(point: &SamplePoint) -> Vec<Statistic> {
    match point.tree_dimension_statistic_count {
        1 => vec![Statistic::InodeNumberWatermark],
        2 => vec![Statistic::InodeNumberWatermark, Statistic::SecondTreeDimensionStatistic],
        other => panic!("r_tree_dim 只取 1 或 2（登记第一节第 8 条），这里是 {other}"),
    }
}

/// 不认识的头更早的第 `step_index` 级台阶（从 0 起，越往后越新）的代段。
fn unknown_head_step_txg(earlier_step_count: u64, step_index: u64) -> u64 {
    INITIAL_VERSION_TXG - STEP_TXG_SPACING * (earlier_step_count - step_index)
}

fn watermark_change_history_of(point: &SamplePoint) -> WatermarkChangeHistory {
    let mut changed_at = BTreeSet::new();
    for known_head in known_writable_heads(point) {
        if point.geometry.known_head_step_rows() > 0 {
            changed_at.insert((known_head, KNOWN_HEAD_STEP_TXG));
        }
    }
    let the_initial_publish_is_a_step = point.initial_publish_changed_the_watermark;
    for unknown_head in unknown_writable_heads(point) {
        for step_index in 0..point.earlier_step_count {
            changed_at.insert((unknown_head, unknown_head_step_txg(point.earlier_step_count, step_index)));
        }
        if the_initial_publish_is_a_step {
            changed_at.insert((unknown_head, INITIAL_VERSION_TXG));
        }
    }
    WatermarkChangeHistory { changed_at }
}

/// R：认识全部树的新写者发布的那一版。v1 认识的树与不认识的树的行都在。
fn initial_version(point: &SamplePoint) -> AccountingVersion {
    let mut rows = Vec::new();
    for statistic_index in 0..point.geometry.rows_without_tree_dimension() {
        rows.push(AccountingRow {
            statistic: Statistic::WithoutTreeDimension(statistic_index),
            tree: TreeOwner::PoolWide,
            generation_txg: INITIAL_VERSION_TXG,
            value: OPAQUE_STATISTIC_VALUE,
        });
    }
    for known_head in known_writable_heads(point) {
        rows.extend(rows_of_this_generation_for_a_known_head(point, known_head, INITIAL_VERSION_TXG));
        if point.geometry.known_head_step_rows() > 0 {
            rows.push(AccountingRow {
                statistic: Statistic::InodeNumberWatermark,
                tree: known_head,
                generation_txg: KNOWN_HEAD_STEP_TXG,
                value: KNOWN_HEAD_WATERMARK,
            });
        }
    }
    for unknown_head in unknown_writable_heads(point) {
        for step_index in 0..point.earlier_step_count {
            rows.push(AccountingRow {
                statistic: Statistic::InodeNumberWatermark,
                tree: unknown_head,
                generation_txg: unknown_head_step_txg(point.earlier_step_count, step_index),
                value: UNKNOWN_HEAD_FIRST_STEP_WATERMARK + step_index,
            });
        }
        // R 那次发布改了水位，这一行的值就比最后一级更早的台阶大；没改就等于它（s = 0 时没有更早的台阶，取一个基准值）。
        let latest_earlier_watermark = match point.earlier_step_count {
            0 => UNKNOWN_HEAD_FIRST_STEP_WATERMARK,
            earlier_step_count => UNKNOWN_HEAD_FIRST_STEP_WATERMARK + (earlier_step_count - 1),
        };
        let watermark_of_the_initial_publish = latest_earlier_watermark + u64::from(point.initial_publish_changed_the_watermark);
        for statistic in tree_dimension_statistics(point) {
            let value = match statistic {
                Statistic::InodeNumberWatermark => watermark_of_the_initial_publish,
                Statistic::SecondTreeDimensionStatistic | Statistic::WithoutTreeDimension(_) => OPAQUE_STATISTIC_VALUE,
            };
            rows.push(AccountingRow { statistic, tree: unknown_head, generation_txg: INITIAL_VERSION_TXG, value });
        }
    }
    rows.sort();
    AccountingVersion { txg: INITIAL_VERSION_TXG, rows, rows_written_for_unknown_trees: Vec::new() }
}

/// 认识的头每一版写的「这一代的行」：带树维的每个统计量一行。旧写者不改认识的头的水位（与被判的政策无关，取最简的那一种）。
fn rows_of_this_generation_for_a_known_head(point: &SamplePoint, known_head: TreeOwner, txg: u64) -> Vec<AccountingRow> {
    tree_dimension_statistics(point)
        .into_iter()
        .map(|statistic| {
            let value = match statistic {
                Statistic::InodeNumberWatermark => KNOWN_HEAD_WATERMARK,
                Statistic::SecondTreeDimensionStatistic | Statistic::WithoutTreeDimension(_) => OPAQUE_STATISTIC_VALUE,
            };
            AccountingRow { statistic, tree: known_head, generation_txg: txg, value }
        })
        .collect()
}

/// `PerBit` 是条件政策：还有一棵树的位没登记就整次挂载退到只读；全部登记过才退化成跳过。
fn per_bit_policy_reduces_to(point: &SamplePoint) -> UnknownFlagsPolicy {
    if point.trees_with_registered_bit >= UNKNOWN_TREE_COUNT {
        UnknownFlagsPolicy::Skip
    } else {
        UnknownFlagsPolicy::ReadOnly
    }
}

fn effective_flags_policy_of(flags_policy: UnknownFlagsPolicy, point: &SamplePoint) -> UnknownFlagsPolicy {
    match flags_policy {
        UnknownFlagsPolicy::PerBit => per_bit_policy_reduces_to(point),
        UnknownFlagsPolicy::Ignore | UnknownFlagsPolicy::Skip | UnknownFlagsPolicy::RejectEntry | UnknownFlagsPolicy::ReadOnly => flags_policy,
    }
}

/// 记账臂真正起作用的那一条：`Ignore` 下写者自以为认识那几棵树，把它们的行当自己的行维护（与 `Carry` 同一种写法），
/// 记账臂没有作用点；别的 flags 臂下写者知道自己不认识它们，照记账臂办。
fn accounting_policy_in_effect(effective_flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> AccountingPolicy {
    match effective_flags_policy {
        UnknownFlagsPolicy::Ignore => AccountingPolicy::Carry,
        UnknownFlagsPolicy::Skip | UnknownFlagsPolicy::RejectEntry | UnknownFlagsPolicy::ReadOnly | UnknownFlagsPolicy::PerBit => accounting_policy,
    }
}

/// `Carry`：对上一版里每个不认识的树的维度元组写一行（代段 = 这次的 txg，值 = 那个元组代段最大那一行的值），
/// 台阶行照 D5 已定项 2 射程的例外原样照抄。返回 (这一版里不认识的树的行, 这一次新写的行)。
fn carried_unknown_rows(previous: &AccountingVersion, txg: u64, watermark_steps_of_unknown_heads: &[AccountingRow]) -> (Vec<AccountingRow>, Vec<AccountingRow>) {
    let mut latest_row_by_tuple: BTreeMap<(Statistic, TreeOwner), AccountingRow> = BTreeMap::new();
    for row in previous.rows.iter().filter(|row| row.belongs_to_an_unknown_tree()) {
        let keeps_the_later_row = latest_row_by_tuple.get(&row.tuple()).is_none_or(|kept| kept.generation_txg < row.generation_txg);
        if keeps_the_later_row {
            latest_row_by_tuple.insert(row.tuple(), *row);
        }
    }
    let rewritten: Vec<AccountingRow> = latest_row_by_tuple.values().map(|latest_row| latest_row.recoded_to(txg)).collect();
    let mut rows = rewritten.clone();
    rows.extend(watermark_steps_of_unknown_heads.iter().copied());
    (rows, rewritten)
}

fn rows_of_unknown_trees_left_as_they_were(previous: &AccountingVersion) -> Vec<AccountingRow> {
    previous.rows.iter().filter(|row| row.belongs_to_an_unknown_tree()).copied().collect()
}

/// 旧写者的一次发布：自己那些行全写一遍（代段 = 这次的 txg），认识的头的台阶行照抄，不认识的树的行照记账臂办。
fn next_version(
    previous: &AccountingVersion,
    point: &SamplePoint,
    history: &WatermarkChangeHistory,
    effective_flags_policy: UnknownFlagsPolicy,
    accounting_policy: AccountingPolicy,
) -> AccountingVersion {
    let txg = previous.txg + 1;
    let mut rows = Vec::new();
    for statistic_index in 0..point.geometry.rows_without_tree_dimension() {
        rows.push(AccountingRow {
            statistic: Statistic::WithoutTreeDimension(statistic_index),
            tree: TreeOwner::PoolWide,
            generation_txg: txg,
            value: OPAQUE_STATISTIC_VALUE,
        });
    }
    for known_head in known_writable_heads(point) {
        rows.extend(rows_of_this_generation_for_a_known_head(point, known_head, txg));
    }
    rows.extend(previous.rows.iter().filter(|row| matches!(row.tree, TreeOwner::KnownHead(_)) && history.is_a_step(row)).copied());

    let watermark_steps_of_unknown_heads: Vec<AccountingRow> =
        previous.rows.iter().filter(|row| row.belongs_to_an_unknown_tree() && history.is_a_step(row)).copied().collect();
    let (rows_of_unknown_trees, rows_written_for_unknown_trees) = match accounting_policy_in_effect(effective_flags_policy, accounting_policy) {
        AccountingPolicy::Carry => carried_unknown_rows(previous, txg, &watermark_steps_of_unknown_heads),
        AccountingPolicy::NoCarry => (watermark_steps_of_unknown_heads.clone(), Vec::new()),
        AccountingPolicy::KeepWithRoot => (rows_of_unknown_trees_left_as_they_were(previous), Vec::new()),
    };
    rows.extend(rows_of_unknown_trees);
    rows.sort();
    AccountingVersion { txg, rows, rows_written_for_unknown_trees }
}

/// 一格（flags 臂 × 记账臂）在一个取样点上：R 与旧写者发布的 R_1 … R_G（只读挂载时一版都没有）。
struct PublishedHistory {
    initial: AccountingVersion,
    versions: Vec<AccountingVersion>,
    writable_generations: u64,
    effective_flags_policy: UnknownFlagsPolicy,
}

fn published_history_of(point: &SamplePoint, history: &WatermarkChangeHistory, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> PublishedHistory {
    let effective_flags_policy = effective_flags_policy_of(flags_policy, point);
    let writable_generations = if effective_flags_policy == UnknownFlagsPolicy::ReadOnly { 0 } else { point.published_generations };
    let initial = initial_version(point);
    let mut versions: Vec<AccountingVersion> = Vec::new();
    for _generation_index in 0..writable_generations {
        let previous = versions.last().unwrap_or(&initial);
        let published = next_version(previous, point, history, effective_flags_policy, accounting_policy);
        versions.push(published);
    }
    PublishedHistory { initial, versions, writable_generations, effective_flags_policy }
}

/// 认台阶行的两种认法：D5 意义（按 (统计量, 树) 分、水位变了的那次发布才留），与池级 checker 今天判 I-3.13 时的语法认法
/// （`crates/singlefs-checker/src/walk.rs` 判 I-3.13 的那一段：水位那一统计量、不带设备维、代段早于这一版 txg 的行就跳过）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StepRecognition {
    ByWatermarkChangeHistory,
    ByCheckerSyntax,
}

/// U7（I-3.13）认台阶行用哪一种：登记第六节「台阶行按 D5 意义认，不按语法认」。
const STEP_RECOGNITION_OF_THE_ROW_GENERATION_INVARIANT: StepRecognition = StepRecognition::ByWatermarkChangeHistory;

fn row_counts_as_a_step(row: &AccountingRow, version_txg: u64, history: &WatermarkChangeHistory, recognition: StepRecognition) -> bool {
    match recognition {
        StepRecognition::ByWatermarkChangeHistory => history.is_a_step(row),
        StepRecognition::ByCheckerSyntax => row.statistic == Statistic::InodeNumberWatermark && row.generation_txg < version_txg,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowsJudged {
    AllRows,
    WatermarkRowsOnly,
}

/// P-gen：这一版里每一行的代段等于这一版的 txg，或者它是一级台阶行。
fn every_row_is_this_generation_or_a_step(version: &AccountingVersion, history: &WatermarkChangeHistory, recognition: StepRecognition, judged: RowsJudged) -> bool {
    version
        .rows
        .iter()
        .filter(|row| match judged {
            RowsJudged::AllRows => true,
            RowsJudged::WatermarkRowsOnly => row.statistic == Statistic::InodeNumberWatermark,
        })
        .all(|row| row.generation_txg == version.txg || row_counts_as_a_step(row, version.txg, history, recognition))
}

/// P-step：上一版的每一级台阶行原样在这一版里。
fn every_step_of_the_previous_version_is_kept(previous: &AccountingVersion, version: &AccountingVersion, history: &WatermarkChangeHistory) -> bool {
    previous.rows.iter().filter(|row| history.is_a_step(row)).all(|step| version.rows.contains(step))
}

/// 登记第六节条款清单 U1–U13（一条 = 一个已定项或不变量清单的一行）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ClauseUnit {
    /// U1：D5 已定项 2（条文与射程「inode 号水位的台阶行」）。
    DecisionFiveItemTwoGenerationAndSteps,
    /// U2：D5 已定项 4 射程第 12 项那一条。
    DecisionFiveItemFourWatermarkRows,
    /// U3：D8 已定项 1。
    DecisionEightItemOneWholeBatch,
    /// U4：D8 已定项 6「水位行……照台阶行的规矩写」。
    DecisionEightItemSixWatermarkRowRule,
    /// U5：D8 已定项 12 条文「含已定项 8 ㊀ 那类旧写者搬运的记账行」。
    DecisionEightItemTwelveCarriedRows,
    /// U6：D16 已定项 9「每次发布把这一代的行全写一遍」。
    DecisionSixteenItemNineRewriteEachPublish,
    /// U7：I-3.13。
    InvariantGenerationEqualsTheVersionTxg,
    /// U8：I-9.11。
    InvariantWritableHeadHasAWatermarkRow,
    /// U9：I-9.30 ①。
    InvariantWatermarkNotBelowItsSteps,
    /// U10：D18 已定项 24 第 5 条。
    DecisionEighteenItemTwentyFourLineageSteps,
    /// U11：D23 已定项 14「不点删旧代，跳号不留要删的行」。
    DecisionTwentyThreeItemFourteenNoPointDeletion,
    /// U12：D8 已定项 8 ㊀（只在读法 B2 下计）。
    DecisionEightItemEightCarryUnknownRows,
    /// U13：D8 已定项 8 ㊁（只在读法 B2 下计）。
    DecisionEightItemEightRejectUnknownBits,
}

const ALL_CLAUSE_UNITS: [ClauseUnit; 13] = [
    ClauseUnit::DecisionFiveItemTwoGenerationAndSteps,
    ClauseUnit::DecisionFiveItemFourWatermarkRows,
    ClauseUnit::DecisionEightItemOneWholeBatch,
    ClauseUnit::DecisionEightItemSixWatermarkRowRule,
    ClauseUnit::DecisionEightItemTwelveCarriedRows,
    ClauseUnit::DecisionSixteenItemNineRewriteEachPublish,
    ClauseUnit::InvariantGenerationEqualsTheVersionTxg,
    ClauseUnit::InvariantWritableHeadHasAWatermarkRow,
    ClauseUnit::InvariantWatermarkNotBelowItsSteps,
    ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps,
    ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion,
    ClauseUnit::DecisionEightItemEightCarryUnknownRows,
    ClauseUnit::DecisionEightItemEightRejectUnknownBits,
];

impl ClauseUnit {
    fn registered_label(self) -> &'static str {
        match self {
            ClauseUnit::DecisionFiveItemTwoGenerationAndSteps => "U1",
            ClauseUnit::DecisionFiveItemFourWatermarkRows => "U2",
            ClauseUnit::DecisionEightItemOneWholeBatch => "U3",
            ClauseUnit::DecisionEightItemSixWatermarkRowRule => "U4",
            ClauseUnit::DecisionEightItemTwelveCarriedRows => "U5",
            ClauseUnit::DecisionSixteenItemNineRewriteEachPublish => "U6",
            ClauseUnit::InvariantGenerationEqualsTheVersionTxg => "U7",
            ClauseUnit::InvariantWritableHeadHasAWatermarkRow => "U8",
            ClauseUnit::InvariantWatermarkNotBelowItsSteps => "U9",
            ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps => "U10",
            ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion => "U11",
            ClauseUnit::DecisionEightItemEightCarryUnknownRows => "U12",
            ClauseUnit::DecisionEightItemEightRejectUnknownBits => "U13",
        }
    }

    /// 读法 B1 计不计它：㊀ ㊁ 两条不计（每条非现任臂都要改它，计进去等于先给现任臂一票）。
    fn counts_when_the_judged_items_are_excluded(self) -> bool {
        !matches!(self, ClauseUnit::DecisionEightItemEightCarryUnknownRows | ClauseUnit::DecisionEightItemEightRejectUnknownBits)
    }
}

fn latest_watermark_row_of(version: &AccountingVersion, head: TreeOwner) -> Option<AccountingRow> {
    version.rows.iter().filter(|row| row.statistic == Statistic::InodeNumberWatermark && row.tree == head).max_by_key(|row| row.generation_txg).copied()
}

/// 一个头代段不晚于 `generation_limit` 的台阶里最大的水位（D18 已定项 24 第 5 条「代段 ≤ T_rb − 1 的台阶里最大的 H」）。
fn largest_step_watermark_up_to(version: &AccountingVersion, head: TreeOwner, history: &WatermarkChangeHistory, generation_limit: u64) -> Option<u64> {
    version.rows.iter().filter(|row| row.tree == head && history.is_a_step(row) && row.generation_txg <= generation_limit).map(|row| row.value).max()
}

fn all_writable_heads(point: &SamplePoint) -> Vec<TreeOwner> {
    let mut heads = known_writable_heads(point);
    heads.extend(unknown_writable_heads(point));
    heads
}

/// R 里每个不认识的树的维度元组代段最大那一行的值。
fn latest_value_of_each_unknown_tuple(initial: &AccountingVersion) -> BTreeMap<(Statistic, TreeOwner), u64> {
    let mut latest: BTreeMap<(Statistic, TreeOwner), (u64, u64)> = BTreeMap::new();
    for row in initial.rows.iter().filter(|row| row.belongs_to_an_unknown_tree()) {
        let entry = latest.entry(row.tuple()).or_insert((row.generation_txg, row.value));
        if row.generation_txg >= entry.0 {
            *entry = (row.generation_txg, row.value);
        }
    }
    latest.into_iter().map(|(tuple, (_generation_txg, value))| (tuple, value)).collect()
}

/// R 里属于不认识的树的行数（第二次跑新 U5 的 N）。
fn unknown_tree_row_count_of(initial: &AccountingVersion) -> u64 {
    initial.rows.iter().filter(|row| row.belongs_to_an_unknown_tree()).count().try_into().expect("计数装得进 u64")
}

/// 新 U5 的 W_g：写者在 R_g 这一次为不认识的树换了代号写进去的行（读法第 10 条；照抄的台阶行、原样留着的行都不算）。
fn rows_the_writer_wrote_for_unknown_trees(version: &AccountingVersion) -> Vec<AccountingRow> {
    version.rows_written_for_unknown_trees.clone()
}

/// 第二次跑的新 U5（登记第二次跑第六节）：N = 0 ⇒ 真（没有对象）；N > 0 且 W_g 为空 ⇒ 假（条款成了空指称）；N > 0 且 W_g 非空 ⇒ W_g 每一行代段 = R_g 的 txg 才真。
fn dangling_carried_rows_clause_holds(initial: &AccountingVersion, version: &AccountingVersion) -> bool {
    let unknown_tree_row_count = unknown_tree_row_count_of(initial);
    let written = rows_the_writer_wrote_for_unknown_trees(version);
    if unknown_tree_row_count == 0 { return true; }
    if written.is_empty() { return false; }
    written.iter().all(|row| row.generation_txg == version.txg)
}

/// 第六节表 U 列每一条的谓词，在 R_g（`version`）上判，`previous` 是 R_{g−1}，`initial` 是 R。
fn clause_holds_on_version(
    clause: ClauseUnit,
    initial: &AccountingVersion,
    previous: &AccountingVersion,
    version: &AccountingVersion,
    point: &SamplePoint,
    history: &WatermarkChangeHistory,
    flags_policy: UnknownFlagsPolicy,
    judgement: Judgement,
) -> bool {
    let generation_rule = |recognition: StepRecognition, judged: RowsJudged| every_row_is_this_generation_or_a_step(version, history, recognition, judged);
    match clause {
        ClauseUnit::DecisionFiveItemTwoGenerationAndSteps => {
            generation_rule(StepRecognition::ByWatermarkChangeHistory, RowsJudged::AllRows) && every_step_of_the_previous_version_is_kept(previous, version, history)
        }
        ClauseUnit::DecisionFiveItemFourWatermarkRows | ClauseUnit::DecisionEightItemSixWatermarkRowRule => {
            generation_rule(StepRecognition::ByWatermarkChangeHistory, RowsJudged::WatermarkRowsOnly)
        }
        ClauseUnit::DecisionEightItemOneWholeBatch | ClauseUnit::DecisionSixteenItemNineRewriteEachPublish => {
            generation_rule(StepRecognition::ByWatermarkChangeHistory, RowsJudged::AllRows)
        }
        ClauseUnit::DecisionEightItemTwelveCarriedRows => match judgement {
            Judgement::VacuousCarriedRowsClause => version.rows_written_for_unknown_trees.iter().all(|row| row.generation_txg == version.txg),
            Judgement::DanglingCarriedRowsClause => dangling_carried_rows_clause_holds(initial, version),
        },
        ClauseUnit::InvariantGenerationEqualsTheVersionTxg => generation_rule(STEP_RECOGNITION_OF_THE_ROW_GENERATION_INVARIANT, RowsJudged::AllRows),
        ClauseUnit::InvariantWritableHeadHasAWatermarkRow => {
            let heads_that_must_have_a_watermark_row: Vec<TreeOwner> = known_writable_heads(point).into_iter().chain(unknown_writable_heads(point)).collect();
            heads_that_must_have_a_watermark_row
                .iter()
                .all(|head| version.rows.iter().any(|row| row.statistic == Statistic::InodeNumberWatermark && row.tree == *head))
        }
        ClauseUnit::InvariantWatermarkNotBelowItsSteps => all_writable_heads(point).into_iter().all(|head| match latest_watermark_row_of(version, head) {
            None => true,
            Some(latest_row) => version.rows.iter().filter(|row| row.tree == head && history.is_a_step(row)).all(|step| step.value <= latest_row.value),
        }),
        ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps => all_writable_heads(point).into_iter().all(|head| {
            largest_step_watermark_up_to(previous, head, history, previous.txg) == largest_step_watermark_up_to(version, head, history, version.txg - 1)
        }),
        // 模型每一版整棵重建，没有点删这一步：空真。
        ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion => true,
        ClauseUnit::DecisionEightItemEightCarryUnknownRows => latest_value_of_each_unknown_tuple(initial)
            .into_iter()
            .all(|(tuple, value)| version.rows.iter().any(|row| row.tuple() == tuple && row.generation_txg == version.txg && row.value == value)),
        ClauseUnit::DecisionEightItemEightRejectUnknownBits => flags_policy == UnknownFlagsPolicy::RejectEntry,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ClauseReading {
    /// 判的那一种：D8 已定项 8 的 ㊀ ㊁ 不计。
    ExcludesTheJudgedItems,
    /// 附带量：㊀ ㊁ 各算一条。
    CountsTheJudgedItems,
}

/// 第七节「条款账本（B1）」与「B2 增项」逐格照抄，与装置的谓词是两条路（第十节「账本两条路」）。
/// `dangling-u5` 下的账本是第二次跑登记第七节那张新表：`NoCarry`、随根留着在 `h_new > 0` 的各列各比第一次跑的表多 1（新 U5）。
fn clause_units_from_the_hand_ledger(
    point: &SamplePoint,
    flags_policy: UnknownFlagsPolicy,
    accounting_policy: AccountingPolicy,
    reading: ClauseReading,
    judgement: Judgement,
) -> u64 {
    let accounting_arm_acts = matches!(effective_flags_policy_of(flags_policy, point), UnknownFlagsPolicy::Skip | UnknownFlagsPolicy::RejectEntry);
    let has_unknown_heads = point.unknown_writable_head_count > 0;
    let no_step_at_all = point.earlier_step_count == 0 && !point.initial_publish_changed_the_watermark;
    let ledger_excluding_the_judged_items = if !accounting_arm_acts || !has_unknown_heads {
        0
    } else {
        match (judgement, accounting_policy) {
            (_, AccountingPolicy::Carry) => 0,
            (Judgement::VacuousCarriedRowsClause, AccountingPolicy::NoCarry) => u64::from(no_step_at_all),
            (Judgement::DanglingCarriedRowsClause, AccountingPolicy::NoCarry) => {
                if no_step_at_all {
                    2
                } else {
                    1
                }
            }
            (Judgement::VacuousCarriedRowsClause, AccountingPolicy::KeepWithRoot) => match (point.initial_publish_changed_the_watermark, point.tree_dimension_statistic_count) {
                (false, _) => 6,
                (true, 1) => 0,
                (true, _) => 4,
            },
            (Judgement::DanglingCarriedRowsClause, AccountingPolicy::KeepWithRoot) => match (point.initial_publish_changed_the_watermark, point.tree_dimension_statistic_count) {
                (false, _) => 7,
                (true, 1) => 1,
                (true, _) => 5,
            },
        }
    };
    match reading {
        ClauseReading::ExcludesTheJudgedItems => ledger_excluding_the_judged_items,
        ClauseReading::CountsTheJudgedItems => {
            let carry_item_extra = u64::from(accounting_arm_acts && has_unknown_heads && accounting_policy != AccountingPolicy::Carry);
            let reject_item_extra = u64::from(flags_policy != UnknownFlagsPolicy::RejectEntry);
            ledger_excluding_the_judged_items + carry_item_extra + reject_item_extra
        }
    }
}

/// 一个量在 R_1 … R_G 上的三个数：峰值、为正的版数、期末值（R 那一版是初始供给、不算）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Trajectory {
    peak: u64,
    positive_versions: u64,
    final_value: u64,
}

/// 没有新版本（只读挂载）时没有轨迹，交回 `None`，产物里写 `no_new_version`，不当 0。
fn trajectory_of(values_by_version: &[u64]) -> Option<Trajectory> {
    let final_value = *values_by_version.last()?;
    Some(Trajectory {
        peak: values_by_version.iter().copied().max().unwrap_or(0),
        positive_versions: values_by_version.iter().filter(|value| **value > 0).count().try_into().expect("计数装得进 u64"),
        final_value,
    })
}

fn render_trajectory(trajectory: Option<Trajectory>) -> String {
    match trajectory {
        Some(trajectory) => format!("{}/{}/{}", trajectory.peak, trajectory.positive_versions, trajectory.final_value),
        None => "no_new_version".to_string(),
    }
}

/// `crates/` 今天的台阶认法（`TransactionOutput::inode_number_watermark_steps` 与 checker 的 `inode_number_watermark_steps_of_a_version`
/// 同一条，按字段表另写一份）：水位那一统计量代段早于这一版 txg 的行都是台阶，按代段升序；代段等于这一版 txg 的第一行（key 次序），
/// 值与代段最大的台阶不同（或没有更早的台阶）也是台阶。不按树分。
fn steps_by_the_crates_reading(version: &AccountingVersion) -> Vec<AccountingRow> {
    let mut steps: Vec<AccountingRow> =
        version.rows.iter().filter(|row| row.statistic == Statistic::InodeNumberWatermark && row.generation_txg < version.txg).copied().collect();
    steps.sort_by_key(|row| row.generation_txg);
    let row_of_this_version = version.rows.iter().find(|row| row.statistic == Statistic::InodeNumberWatermark && row.generation_txg == version.txg);
    if let Some(row) = row_of_this_version {
        if steps.last().is_none_or(|latest_step| latest_step.value != row.value) {
            steps.push(*row);
        }
    }
    steps
}

fn steps_by_the_watermark_change_history(version: &AccountingVersion, history: &WatermarkChangeHistory) -> Vec<AccountingRow> {
    let mut steps: Vec<AccountingRow> = version.rows.iter().filter(|row| history.is_a_step(row)).copied().collect();
    steps.sort_by_key(|row| row.generation_txg);
    steps
}

/// I-3.13 按池级 checker 今天的语法认法判出的违例行数（附带量，停机条款第 3 项照它写）。
fn row_generation_invariant_violations_under_the_checker_syntax(version: &AccountingVersion, history: &WatermarkChangeHistory) -> u64 {
    version
        .rows
        .iter()
        .filter(|row| !row_counts_as_a_step(row, version.txg, history, StepRecognition::ByCheckerSyntax) && row.generation_txg != version.txg)
        .count().try_into().expect("计数装得进 u64")
}

fn rows_of_tuple(version: &AccountingVersion, tuple: (Statistic, TreeOwner)) -> u64 {
    version.rows.iter().filter(|row| row.tuple() == tuple).count().try_into().expect("计数装得进 u64")
}

#[derive(Clone, Debug)]
struct CellOutcome {
    flags_policy: UnknownFlagsPolicy,
    accounting_policy: AccountingPolicy,
    writable_generations: u64,
    blocks_freed_while_referenced: u64,
    read_only_violations: u64,
    revived_dead_trees: u64,
    /// 元组口径（第一节第 4 条）：R 里有行、R_G 里一行都没有的不认识的树的维度元组数。
    rows_lost: u64,
    /// 行数口径的附带量 `per_generation_rows_dropped` 的总和：每一版里不认识的树每个元组比上一版少了几行，逐版相加。
    rows_dropped_over_all_generations: u64,
    silent_harm_count: u64,
    /// U1–U13 每一条在每一版上为假（1）还是真（0）的轨迹。
    clause_false_trajectories: Vec<(ClauseUnit, Option<Trajectory>)>,
    clause_units_excluding_the_judged_items: u64,
    clause_units_counting_the_judged_items: u64,
    /// W2：R_g 记账树里的全部行数，R_1 … R_G 上取最大值；没有新版本记 0（一行都没写）。
    rows_written_all_rows: u64,
    /// W1：代段等于 R_g 的 txg 的行数（E113 口径），同上取最大值。
    rows_written_this_generation: u64,
    trajectory_unknown_tuples_remaining: Option<Trajectory>,
    trajectory_rows_lost: Option<Trajectory>,
    trajectory_rows_written_this_generation: Option<Trajectory>,
    trajectory_rows_written_all_rows: Option<Trajectory>,
    trajectory_unknown_rows_off_this_txg: Option<Trajectory>,
    /// 不认识的树代段等于 R 的 txg 的行数（随根留着的阳性对照：每一版都有一行，为正版数 = G）。
    trajectory_unknown_rows_at_the_initial_txg: Option<Trajectory>,
    trajectory_rows_dropped_per_generation: Option<Trajectory>,
    trajectory_row_generation_invariant_syntactic_violations: Option<Trajectory>,
    trajectory_crates_step_reading_differences: Option<Trajectory>,
}

fn blocks_wrongly_freed(invisible_window_generations: u64) -> u64 {
    BLOCKS_BORN_PER_GENERATION * invisible_window_generations
}

fn evaluate_cell(point: &SamplePoint, history: &WatermarkChangeHistory, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, judgement: Judgement) -> CellOutcome {
    let published = published_history_of(point, history, flags_policy, accounting_policy);
    let effective_flags_policy = published.effective_flags_policy;
    let writable_generations = published.writable_generations;

    // 只有「树对写者隐形」才会认错 deadlist 的邻居；RejectEntry 下条目在、只是不可用 ⇒ 写者不做跨它的合并。
    let blocks_freed_while_referenced = if effective_flags_policy == UnknownFlagsPolicy::Skip && writable_generations > 0 {
        blocks_wrongly_freed(point.invisible_window_generations)
    } else {
        0
    };
    // 只有「自以为认识」才会把只读树当可写、把半删树当活的。
    let (read_only_violations, revived_dead_trees) = if effective_flags_policy == UnknownFlagsPolicy::Ignore {
        (UNKNOWN_READ_ONLY_TREE_COUNT * writable_generations, UNKNOWN_HALF_DELETED_TREE_COUNT)
    } else {
        (0, 0)
    };

    let unknown_tuples: Vec<(Statistic, TreeOwner)> = latest_value_of_each_unknown_tuple(&published.initial).into_keys().collect();
    let mut unknown_tuples_remaining_by_version = Vec::new();
    let mut rows_lost_by_version = Vec::new();
    let mut rows_written_this_generation_by_version = Vec::new();
    let mut rows_written_all_rows_by_version = Vec::new();
    let mut unknown_rows_off_this_txg_by_version = Vec::new();
    let mut unknown_rows_at_the_initial_txg_by_version = Vec::new();
    let mut rows_dropped_by_version = Vec::new();
    let mut row_generation_invariant_syntactic_violations_by_version = Vec::new();
    let mut crates_step_reading_differences_by_version = Vec::new();
    let mut clause_false_by_version: BTreeMap<ClauseUnit, Vec<u64>> = ALL_CLAUSE_UNITS.iter().map(|clause| (*clause, Vec::new())).collect();
    for (version_index, version) in published.versions.iter().enumerate() {
        let previous = if version_index == 0 { &published.initial } else { &published.versions[version_index - 1] };
        let remaining: u64 = unknown_tuples.iter().filter(|tuple| rows_of_tuple(version, **tuple) > 0).count().try_into().expect("计数装得进 u64");
        unknown_tuples_remaining_by_version.push(remaining);
        let unknown_tuple_count: u64 = unknown_tuples.len().try_into().expect("计数装得进 u64");
        rows_lost_by_version.push(unknown_tuple_count - remaining);
        rows_written_this_generation_by_version.push(version.rows.iter().filter(|row| row.generation_txg == version.txg).count().try_into().expect("计数装得进 u64"));
        rows_written_all_rows_by_version.push(version.rows.len().try_into().expect("计数装得进 u64"));
        unknown_rows_off_this_txg_by_version
            .push(version.rows.iter().filter(|row| row.belongs_to_an_unknown_tree() && row.generation_txg != version.txg).count().try_into().expect("计数装得进 u64"));
        unknown_rows_at_the_initial_txg_by_version
            .push(version.rows.iter().filter(|row| row.belongs_to_an_unknown_tree() && row.generation_txg == INITIAL_VERSION_TXG).count().try_into().expect("计数装得进 u64"));
        rows_dropped_by_version.push(unknown_tuples.iter().map(|tuple| rows_of_tuple(previous, *tuple).saturating_sub(rows_of_tuple(version, *tuple))).sum::<u64>());
        row_generation_invariant_syntactic_violations_by_version.push(row_generation_invariant_violations_under_the_checker_syntax(version, history));
        let crates_steps: BTreeSet<AccountingRow> = steps_by_the_crates_reading(version).into_iter().collect();
        let change_history_steps: BTreeSet<AccountingRow> = steps_by_the_watermark_change_history(version, history).into_iter().collect();
        crates_step_reading_differences_by_version.push(crates_steps.symmetric_difference(&change_history_steps).count().try_into().expect("计数装得进 u64"));
        for clause in ALL_CLAUSE_UNITS {
            let holds = clause_holds_on_version(clause, &published.initial, previous, version, point, history, flags_policy, judgement);
            clause_false_by_version.get_mut(&clause).expect("每一条都在表里").push(u64::from(!holds));
        }
    }

    let tuples_lost_by_the_final_version = rows_lost_by_version.last().copied().unwrap_or(0);
    let rows_dropped_over_all_generations: u64 = rows_dropped_by_version.iter().sum();
    let rows_lost = tuples_lost_by_the_final_version;
    let final_version = published.versions.last().unwrap_or(&published.initial);
    let lost_tuples_of = |statistic: Statistic| -> u64 {
        unknown_tuples.iter().filter(|tuple| tuple.0 == statistic && rows_of_tuple(final_version, **tuple) == 0).count().try_into().expect("计数装得进 u64")
    };
    let watermark_tuples_lost = lost_tuples_of(Statistic::InodeNumberWatermark);
    let second_statistic_tuples_lost = lost_tuples_of(Statistic::SecondTreeDimensionStatistic);

    let silent_if_uncovered = |harm: u64, has_invariant: bool| -> u64 { u64::from(harm > 0 && !has_invariant) };
    let silent_harm_count = silent_if_uncovered(blocks_freed_while_referenced, WRONG_FREE_HAS_AN_INVARIANT)
        + silent_if_uncovered(read_only_violations, false)
        + silent_if_uncovered(revived_dead_trees, false)
        + silent_if_uncovered(watermark_tuples_lost, WATERMARK_TUPLE_LOSS_HAS_AN_INVARIANT)
        + silent_if_uncovered(second_statistic_tuples_lost, SECOND_TREE_STATISTIC_LOSS_HAS_AN_INVARIANT);

    let clause_false_trajectories: Vec<(ClauseUnit, Option<Trajectory>)> =
        ALL_CLAUSE_UNITS.iter().map(|clause| (*clause, trajectory_of(&clause_false_by_version[clause]))).collect();
    // U13 按臂判（flags 臂对未知位的处置是不是「该条目 EIO」），不看有没有新版本；别的条在任一版上为假记 1，没有新版本空真。
    let clause_counts = |clause: ClauseUnit, trajectory: Option<Trajectory>| -> bool {
        if clause == ClauseUnit::DecisionEightItemEightRejectUnknownBits {
            flags_policy != UnknownFlagsPolicy::RejectEntry
        } else {
            trajectory.is_some_and(|trajectory| trajectory.peak > 0)
        }
    };
    let clause_units_excluding_the_judged_items: u64 =
        clause_false_trajectories.iter().filter(|(clause, trajectory)| clause.counts_when_the_judged_items_are_excluded() && clause_counts(*clause, *trajectory)).count().try_into().expect("计数装得进 u64");
    let clause_units_counting_the_judged_items: u64 = clause_false_trajectories.iter().filter(|(clause, trajectory)| clause_counts(*clause, *trajectory)).count().try_into().expect("计数装得进 u64");

    CellOutcome {
        flags_policy,
        accounting_policy,
        writable_generations,
        blocks_freed_while_referenced,
        read_only_violations,
        revived_dead_trees,
        rows_lost,
        rows_dropped_over_all_generations,
        silent_harm_count,
        clause_false_trajectories,
        clause_units_excluding_the_judged_items,
        clause_units_counting_the_judged_items,
        rows_written_all_rows: rows_written_all_rows_by_version.iter().copied().max().unwrap_or(0),
        rows_written_this_generation: rows_written_this_generation_by_version.iter().copied().max().unwrap_or(0),
        trajectory_unknown_tuples_remaining: trajectory_of(&unknown_tuples_remaining_by_version),
        trajectory_rows_lost: trajectory_of(&rows_lost_by_version),
        trajectory_rows_written_this_generation: trajectory_of(&rows_written_this_generation_by_version),
        trajectory_rows_written_all_rows: trajectory_of(&rows_written_all_rows_by_version),
        trajectory_unknown_rows_off_this_txg: trajectory_of(&unknown_rows_off_this_txg_by_version),
        trajectory_unknown_rows_at_the_initial_txg: trajectory_of(&unknown_rows_at_the_initial_txg_by_version),
        trajectory_rows_dropped_per_generation: trajectory_of(&rows_dropped_by_version),
        trajectory_row_generation_invariant_syntactic_violations: trajectory_of(&row_generation_invariant_syntactic_violations_by_version),
        trajectory_crates_step_reading_differences: trajectory_of(&crates_step_reading_differences_by_version),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowsWrittenReading {
    /// 判的那一种：R_g 记账树里的全部行数。
    AllRowsOfTheVersion,
    /// 附带量：代段等于 R_g 的 txg 的行数（E113 口径）。
    ThisGenerationRowsOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RejectEntrySemanticsReading {
    /// Q2「计」：`RejectEntry` 每一格多记一条要新定义的语义。
    Counted,
    /// Q2「不计」：维持 E113 原判。
    NotCounted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Reading {
    clause: ClauseReading,
    rows_written: RowsWrittenReading,
    reject_entry_semantics: RejectEntrySemanticsReading,
}

impl Reading {
    fn label(self) -> String {
        let clause = match self.clause {
            ClauseReading::ExcludesTheJudgedItems => "b1",
            ClauseReading::CountsTheJudgedItems => "b2",
        };
        let rows_written = match self.rows_written {
            RowsWrittenReading::AllRowsOfTheVersion => "w2",
            RowsWrittenReading::ThisGenerationRowsOnly => "w1",
        };
        let reject_entry_semantics = match self.reject_entry_semantics {
            RejectEntrySemanticsReading::Counted => "q2_counted",
            RejectEntrySemanticsReading::NotCounted => "q2_not_counted",
        };
        format!("{clause}_{rows_written}_{reject_entry_semantics}")
    }

    fn with_reject_entry_semantics(self, reject_entry_semantics: RejectEntrySemanticsReading) -> Reading {
        Reading { clause: self.clause, rows_written: self.rows_written, reject_entry_semantics }
    }
}

const JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED: Reading =
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::Counted };
const JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED: Reading =
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::NotCounted };
const INCIDENTAL_READINGS: [Reading; 4] = [
    Reading { clause: ClauseReading::CountsTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::Counted },
    Reading { clause: ClauseReading::CountsTheJudgedItems, rows_written: RowsWrittenReading::AllRowsOfTheVersion, reject_entry_semantics: RejectEntrySemanticsReading::NotCounted },
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::ThisGenerationRowsOnly, reject_entry_semantics: RejectEntrySemanticsReading::Counted },
    Reading { clause: ClauseReading::ExcludesTheJudgedItems, rows_written: RowsWrittenReading::ThisGenerationRowsOnly, reject_entry_semantics: RejectEntrySemanticsReading::NotCounted },
];

type LexicographicVector = [u64; 7];

fn reject_entry_semantics_extra_clause_units(flags_policy: UnknownFlagsPolicy, reject_entry_semantics: RejectEntrySemanticsReading) -> u64 {
    if reject_entry_semantics == RejectEntrySemanticsReading::Counted && flags_policy == UnknownFlagsPolicy::RejectEntry {
        REJECT_ENTRY_NEW_SEMANTICS_CLAUSE_UNITS
    } else {
        0
    }
}

/// 跑前写死的字典序，七位逐位比、越小越好。
fn lexicographic_vector(outcome: &CellOutcome, published_generations: u64, clause_units_to_change: u64, rows_written_per_generation: u64) -> LexicographicVector {
    [
        outcome.blocks_freed_while_referenced,
        outcome.read_only_violations + outcome.revived_dead_trees,
        outcome.rows_lost,
        outcome.silent_harm_count,
        published_generations - outcome.writable_generations,
        clause_units_to_change, rows_written_per_generation,
    ]
}

fn vector_under(outcome: &CellOutcome, point: &SamplePoint, reading: Reading) -> LexicographicVector {
    let clause_units_of_the_list = match reading.clause {
        ClauseReading::ExcludesTheJudgedItems => outcome.clause_units_excluding_the_judged_items,
        ClauseReading::CountsTheJudgedItems => outcome.clause_units_counting_the_judged_items,
    };
    let clause_units_to_change = clause_units_of_the_list + reject_entry_semantics_extra_clause_units(outcome.flags_policy, reading.reject_entry_semantics);
    let rows_written_per_generation = match reading.rows_written {
        RowsWrittenReading::AllRowsOfTheVersion => outcome.rows_written_all_rows,
        RowsWrittenReading::ThisGenerationRowsOnly => outcome.rows_written_this_generation,
    };
    lexicographic_vector(outcome, point.published_generations, clause_units_to_change, rows_written_per_generation)
}

struct EvaluatedPoint {
    point: SamplePoint,
    /// 这些格是按哪一种 U5 判据算的（手写账本要按同一种对）。
    judgement: Judgement,
    /// 进字典序比较的格，次序是 flags 臂在外、记账臂在内。
    cells: Vec<CellOutcome>,
}

impl EvaluatedPoint {
    fn outcome_under_the_arms(&self, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> &CellOutcome {
        self.cells
            .iter()
            .find(|cell| cell.flags_policy == flags_policy && cell.accounting_policy == accounting_policy)
            .expect("十五格每一格都进比较（登记第五节：不许有挂得上才比的筛子）")
    }

    fn vector(&self, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, reading: Reading) -> LexicographicVector {
        vector_under(self.outcome_under_the_arms(flags_policy, accounting_policy), &self.point, reading)
    }
}

fn evaluate_point(point: SamplePoint, judgement: Judgement) -> EvaluatedPoint {
    let history = watermark_change_history_of(&point);
    let mut cells = Vec::new();
    for flags_policy in FLAGS_ARMS {
        for accounting_policy in ACCOUNTING_ARMS {
            let outcome = evaluate_cell(&point, &history, flags_policy, accounting_policy, judgement);
            cells.push(outcome);
        }
    }
    EvaluatedPoint { point, judgement, cells }
}

fn flags_label(flags_policy: UnknownFlagsPolicy) -> &'static str {
    match flags_policy {
        UnknownFlagsPolicy::Ignore => "Ignore",
        UnknownFlagsPolicy::Skip => "Skip",
        UnknownFlagsPolicy::RejectEntry => "RejectEntry",
        UnknownFlagsPolicy::ReadOnly => "ReadOnly",
        UnknownFlagsPolicy::PerBit => "PerBit",
    }
}

fn accounting_label(accounting_policy: AccountingPolicy) -> &'static str {
    match accounting_policy {
        AccountingPolicy::Carry => "Carry",
        AccountingPolicy::NoCarry => "NoCarry",
        AccountingPolicy::KeepWithRoot => "KeepWithRoot",
    }
}

/// 第一个不全相等的位号（从 1 数）；七位全等交回 `None`。
fn first_separating_position(vectors: &[LexicographicVector]) -> Option<usize> {
    let reference = vectors.first()?;
    (0..reference.len()).find(|position| vectors.iter().any(|vector| vector[*position] != reference[*position])).map(|position| position + 1)
}

fn render_position(position: Option<usize>) -> String {
    match position {
        Some(position) => position.to_string(),
        None => "none".to_string(),
    }
}

/// 七位最小的那几个的下标（并列全报）。
fn winner_indices(vectors: &[LexicographicVector]) -> Vec<usize> {
    let Some(best) = vectors.iter().min() else {
        return Vec::new();
    };
    (0..vectors.len()).filter(|index| vectors[*index] == *best).collect()
}

const ACCOUNTING_PAIRS: [(AccountingPolicy, AccountingPolicy); 3] = [
    (AccountingPolicy::Carry, AccountingPolicy::NoCarry),
    (AccountingPolicy::Carry, AccountingPolicy::KeepWithRoot),
    (AccountingPolicy::NoCarry, AccountingPolicy::KeepWithRoot),
];

#[derive(Clone, PartialEq, Eq, Debug)]
struct AccountingWinnersUnderFlagsArm {
    flags_policy: UnknownFlagsPolicy,
    first_separating_position: Option<usize>,
    pair_first_separating_positions: Vec<Option<usize>>,
    winners: Vec<AccountingPolicy>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct FlagsWinnersUnderAccountingArm {
    accounting_policy: AccountingPolicy,
    reject_entry_versus_skip_first_separating_position: Option<usize>,
    /// `RejectEntry` 那一格第 6 位对 `Skip` 那一格第 6 位。
    position_six_reject_entry_versus_skip: std::cmp::Ordering,
    /// 第 1–5 位全平而第 6 位分开。
    separable_at_position_six: bool,
    flags_winners: Vec<UnknownFlagsPolicy>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct PointAnalysis {
    accounting_winners_by_flags_arm: Vec<AccountingWinnersUnderFlagsArm>,
    overall_winners: Vec<(UnknownFlagsPolicy, AccountingPolicy)>,
    flags_winners_by_accounting_arm: Vec<FlagsWinnersUnderAccountingArm>,
    all_fifteen_cells_equal: bool,
}

fn analyse_point(evaluated: &EvaluatedPoint, reading: Reading) -> PointAnalysis {
    let accounting_winners_by_flags_arm = FLAGS_ARMS
        .iter()
        .map(|flags_policy| {
            let vectors: Vec<LexicographicVector> = ACCOUNTING_ARMS.iter().map(|accounting_policy| evaluated.vector(*flags_policy, *accounting_policy, reading)).collect();
            let pair_first_separating_positions = ACCOUNTING_PAIRS
                .iter()
                .map(|(left, right)| first_separating_position(&[evaluated.vector(*flags_policy, *left, reading), evaluated.vector(*flags_policy, *right, reading)]))
                .collect();
            AccountingWinnersUnderFlagsArm {
                flags_policy: *flags_policy,
                first_separating_position: first_separating_position(&vectors),
                pair_first_separating_positions,
                winners: winner_indices(&vectors).into_iter().map(|index| ACCOUNTING_ARMS[index]).collect(),
            }
        })
        .collect();
    let all_cells: Vec<(UnknownFlagsPolicy, AccountingPolicy)> =
        FLAGS_ARMS.iter().flat_map(|flags_policy| ACCOUNTING_ARMS.iter().map(move |accounting_policy| (*flags_policy, *accounting_policy))).collect();
    let all_vectors: Vec<LexicographicVector> = all_cells.iter().map(|(flags_policy, accounting_policy)| evaluated.vector(*flags_policy, *accounting_policy, reading)).collect();
    let overall_winners = winner_indices(&all_vectors).into_iter().map(|index| all_cells[index]).collect();
    let flags_winners_by_accounting_arm = ACCOUNTING_ARMS
        .iter()
        .map(|accounting_policy| {
            let reject_entry = evaluated.vector(UnknownFlagsPolicy::RejectEntry, *accounting_policy, reading);
            let skip = evaluated.vector(UnknownFlagsPolicy::Skip, *accounting_policy, reading);
            let flags_vectors: Vec<LexicographicVector> = FLAGS_ARMS.iter().map(|flags_policy| evaluated.vector(*flags_policy, *accounting_policy, reading)).collect();
            FlagsWinnersUnderAccountingArm {
                accounting_policy: *accounting_policy,
                reject_entry_versus_skip_first_separating_position: first_separating_position(&[reject_entry, skip]),
                position_six_reject_entry_versus_skip: reject_entry[5].cmp(&skip[5]),
                separable_at_position_six: reject_entry[..5] == skip[..5] && reject_entry[5] != skip[5],
                flags_winners: winner_indices(&flags_vectors).into_iter().map(|index| FLAGS_ARMS[index]).collect(),
            }
        })
        .collect();
    PointAnalysis { accounting_winners_by_flags_arm, overall_winners, flags_winners_by_accounting_arm, all_fifteen_cells_equal: first_separating_position(&all_vectors).is_none() }
}

fn cell_label(flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> String {
    format!("{}_{}", flags_label(flags_policy), accounting_label(accounting_policy))
}

fn render_vector(vector: &LexicographicVector) -> String {
    vector.iter().map(u64::to_string).collect::<Vec<String>>().join(",")
}

fn join_labels<Item>(items: &[Item], label_of: impl Fn(&Item) -> String) -> String {
    if items.is_empty() {
        return "none".to_string();
    }
    items.iter().map(label_of).collect::<Vec<String>>().join("+")
}

/// 为假过的条（U13 按臂判，记 `U13:arm`），其余写 `U<n>:峰值/为正版数/期末值`；一条都没有写 `none`。没列出的条三数都是 0。
fn render_clauses_false(outcome: &CellOutcome) -> String {
    let mut parts = Vec::new();
    for (clause, trajectory) in &outcome.clause_false_trajectories {
        if *clause == ClauseUnit::DecisionEightItemEightRejectUnknownBits {
            if outcome.flags_policy != UnknownFlagsPolicy::RejectEntry {
                parts.push(format!("{}:arm", clause.registered_label()));
            }
        } else if let Some(trajectory) = trajectory {
            if trajectory.peak > 0 {
                parts.push(format!("{}:{}", clause.registered_label(), render_trajectory(Some(*trajectory))));
            }
        }
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join("+")
    }
}

fn render_cell_line(point_index: usize, evaluated: &EvaluatedPoint, outcome: &CellOutcome) -> String {
    let point = &evaluated.point;
    format!(
        "name=cell point={point_index} arm={} writable_gens={} rdonly_violations={} dead_revived={} vector_q2_counted={} vector_q2_not_counted={} \
         clause_units_b1={} ledger_b1={} clause_units_b2={} ledger_b2={} rows_written_w2={} rows_written_w1={} clauses_false={} \
         unknown_tuples_remaining={} rows_lost_trajectory={} w1_trajectory={} w2_trajectory={} unknown_rows_off_this_txg={} unknown_rows_at_initial_txg={} \
         per_generation_rows_dropped={} per_generation_rows_dropped_total={} i313_syntactic_violations={} crates_step_reading_differences={}",
        cell_label(outcome.flags_policy, outcome.accounting_policy),
        outcome.writable_generations,
        outcome.read_only_violations,
        outcome.revived_dead_trees,
        render_vector(&vector_under(outcome, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)),
        render_vector(&vector_under(outcome, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED)),
        outcome.clause_units_excluding_the_judged_items,
        clause_units_from_the_hand_ledger(point, outcome.flags_policy, outcome.accounting_policy, ClauseReading::ExcludesTheJudgedItems, evaluated.judgement),
        outcome.clause_units_counting_the_judged_items,
        clause_units_from_the_hand_ledger(point, outcome.flags_policy, outcome.accounting_policy, ClauseReading::CountsTheJudgedItems, evaluated.judgement),
        outcome.rows_written_all_rows,
        outcome.rows_written_this_generation,
        render_clauses_false(outcome),
        render_trajectory(outcome.trajectory_unknown_tuples_remaining),
        render_trajectory(outcome.trajectory_rows_lost),
        render_trajectory(outcome.trajectory_rows_written_this_generation),
        render_trajectory(outcome.trajectory_rows_written_all_rows),
        render_trajectory(outcome.trajectory_unknown_rows_off_this_txg),
        render_trajectory(outcome.trajectory_unknown_rows_at_the_initial_txg),
        render_trajectory(outcome.trajectory_rows_dropped_per_generation),
        outcome.rows_dropped_over_all_generations,
        render_trajectory(outcome.trajectory_row_generation_invariant_syntactic_violations),
        render_trajectory(outcome.trajectory_crates_step_reading_differences),
    )
}

fn render_ordering_of_reject_entry_against_skip(ordering: std::cmp::Ordering) -> &'static str {
    match ordering {
        std::cmp::Ordering::Less => "reject_entry_smaller",
        std::cmp::Ordering::Equal => "equal",
        std::cmp::Ordering::Greater => "skip_smaller",
    }
}

fn render_point_verdict_line(point_index: usize, evaluated: &EvaluatedPoint, counted: &PointAnalysis, not_counted: &PointAnalysis) -> String {
    let accounting_winners = &counted.accounting_winners_by_flags_arm;
    let accounting_winners_first_separating = accounting_winners.iter().map(|arm| format!("{}:{}", flags_label(arm.flags_policy), render_position(arm.first_separating_position))).collect::<Vec<_>>().join(",");
    let accounting_winner_pairs = accounting_winners
        .iter()
        .map(|arm| {
            let pairs = ACCOUNTING_PAIRS
                .iter()
                .zip(&arm.pair_first_separating_positions)
                .map(|((left, right), position)| format!("{}~{}={}", accounting_label(*left), accounting_label(*right), render_position(*position)))
                .collect::<Vec<_>>()
                .join("/");
            format!("{}:{pairs}", flags_label(arm.flags_policy))
        })
        .collect::<Vec<_>>()
        .join(",");
    let accounting_winner_sets =
        accounting_winners.iter().map(|arm| format!("{}:{}", flags_label(arm.flags_policy), join_labels(&arm.winners, |policy| accounting_label(*policy).to_string()))).collect::<Vec<_>>().join(",");
    let overall = |analysis: &PointAnalysis| join_labels(&analysis.overall_winners, |(flags_policy, accounting_policy)| cell_label(*flags_policy, *accounting_policy));
    let flags_winners_field = |analysis: &PointAnalysis, render: &dyn Fn(&FlagsWinnersUnderAccountingArm) -> String| {
        analysis.flags_winners_by_accounting_arm.iter().map(|arm| format!("{}:{}", accounting_label(arm.accounting_policy), render(arm))).collect::<Vec<_>>().join(",")
    };
    let first = |arm: &FlagsWinnersUnderAccountingArm| render_position(arm.reject_entry_versus_skip_first_separating_position);
    let six = |arm: &FlagsWinnersUnderAccountingArm| render_ordering_of_reject_entry_against_skip(arm.position_six_reject_entry_versus_skip).to_string();
    let separable = |arm: &FlagsWinnersUnderAccountingArm| u8::from(arm.separable_at_position_six).to_string();
    let flags_winners = |arm: &FlagsWinnersUnderAccountingArm| join_labels(&arm.flags_winners, |policy| flags_label(*policy).to_string());
    format!(
        "name=point_verdict point={point_index} {} legal={} q1_first_separating={accounting_winners_first_separating} q1_pair_first_separating={accounting_winner_pairs} q1_winners={accounting_winner_sets} \
         overall_winners_q2_counted={} overall_winners_q2_not_counted={} \
         q2_reject_vs_skip_first_separating_counted={} q2_reject_vs_skip_first_separating_not_counted={} \
         q2_position_six_counted={} q2_position_six_not_counted={} q2_separable_counted={} q2_separable_not_counted={} \
         q2_flags_winners_counted={} q2_flags_winners_not_counted={} q2_flags_winners_changed={} all_fifteen_cells_equal_counted={} all_fifteen_cells_equal_not_counted={}",
        evaluated.point.describe(),
        u8::from(evaluated.point.is_legal_under_todays_step_rows()),
        overall(counted),
        overall(not_counted),
        flags_winners_field(counted, &first),
        flags_winners_field(not_counted, &first),
        flags_winners_field(counted, &six),
        flags_winners_field(not_counted, &six),
        flags_winners_field(counted, &separable),
        flags_winners_field(not_counted, &separable),
        flags_winners_field(counted, &flags_winners),
        flags_winners_field(not_counted, &flags_winners),
        u8::from(flags_winners_signature(counted) != flags_winners_signature(not_counted)),
        u8::from(counted.all_fifteen_cells_equal),
        u8::from(not_counted.all_fifteen_cells_equal),
    )
}

/// Q2 后半「胜者是否换档」比的东西：每条记账臂下 flags 五格的胜者，加十五格总胜者里的 flags 臂。
fn flags_winners_signature(analysis: &PointAnalysis) -> (Vec<Vec<UnknownFlagsPolicy>>, BTreeSet<UnknownFlagsPolicy>) {
    (
        analysis.flags_winners_by_accounting_arm.iter().map(|arm| arm.flags_winners.clone()).collect(),
        analysis.overall_winners.iter().map(|(flags_policy, _accounting_policy)| *flags_policy).collect(),
    )
}

fn accounting_winners_signature(analysis: &PointAnalysis) -> Vec<Vec<AccountingPolicy>> {
    analysis.accounting_winners_by_flags_arm.iter().map(|arm| arm.winners.clone()).collect()
}

/// 一组取样点的描述：几个点，每一维取到了哪些值（报「依赖取样点」时列翻转点用）。
#[derive(Default)]
struct PointSetDescription {
    point_count: u64,
    values_by_coordinate: BTreeMap<&'static str, BTreeSet<String>>,
}

impl PointSetDescription {
    fn add(&mut self, point: &SamplePoint) {
        self.point_count += 1;
        let coordinates: [(&'static str, String); 9] = [
            ("geometry", point.geometry.label().to_string()),
            ("k_keep", point.retired_kept_generation_count.to_string()),
            ("w_gens", point.invisible_window_generations.to_string()),
            ("h_new", point.unknown_writable_head_count.to_string()),
            ("registered", point.trees_with_registered_bit.to_string()),
            ("g_gens", point.published_generations.to_string()),
            ("s", point.earlier_step_count.to_string()),
            ("lpc", u8::from(point.initial_publish_changed_the_watermark).to_string()),
            ("r_tree_dim", point.tree_dimension_statistic_count.to_string()),
        ];
        for (coordinate, value) in coordinates {
            self.values_by_coordinate.entry(coordinate).or_default().insert(value);
        }
    }

    /// 每个字段名前加 `prefix`（几何敏感性那一行写 `flipped_`，免得与这一行自己的 `pairs=` 混淆）。
    fn render(&self, prefix: &str) -> String {
        let mut text = format!("{prefix}points={}", self.point_count);
        for coordinate in ["geometry", "k_keep", "w_gens", "h_new", "registered", "g_gens", "s", "lpc", "r_tree_dim"] {
            let values = self.values_by_coordinate.get(coordinate).map(|values| values.iter().cloned().collect::<Vec<String>>().join("+")).unwrap_or_else(|| "none".to_string());
            text.push_str(&format!(" {prefix}{coordinate}={values}"));
        }
        text
    }
}

/// 判决分组：合法点与 E113 口径点（(s, lpc) = (0, 0)）分开，`r_tree_dim` 两档分开（登记第一节第 9 条）。
fn group_of(point: &SamplePoint) -> String {
    let legality = if point.is_legal_under_todays_step_rows() { "legal" } else { "e113_style" };
    format!("{legality}_r_tree_dim_{}", point.tree_dimension_statistic_count)
}

/// 记账臂起作用的 flags 格（第五节阳性对照表「flags 臂同上」那一组）：`Skip`、`RejectEntry`，`PerBit` 只在全部登记时。
fn accounting_arm_acts_under(flags_policy: UnknownFlagsPolicy, point: &SamplePoint) -> bool {
    matches!(effective_flags_policy_of(flags_policy, point), UnknownFlagsPolicy::Skip | UnknownFlagsPolicy::RejectEntry)
}

/// 第八节判别力自证·Q2：钉住选定读法（Q2「计」）下同一记账臂 `RejectEntry` 格与 `Skip` 格第 6 位的比较，翻 Q2 那一位之后比较变了才算由绿转红。
fn flags_question_self_proof_turns_red(evaluated: &EvaluatedPoint, accounting_policy: AccountingPolicy) -> bool {
    let pinned_reading = JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED;
    let flipped_reading = pinned_reading.with_reject_entry_semantics(RejectEntrySemanticsReading::NotCounted);
    let pinned = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, pinned_reading)[5].cmp(&evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, pinned_reading)[5]);
    let after_flip =
        evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, flipped_reading)[5].cmp(&evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, flipped_reading)[5]);
    pinned != after_flip
}

/// 第八节判别力自证·Q1 第 6 位：钉住 `Carry` 与随根留着第 6 位的比较，把随根留着的第 6 位改成等于 `Carry` 的值之后比较变了才算由绿转红。
fn accounting_question_position_six_self_proof_turns_red(evaluated: &EvaluatedPoint, flags_policy: UnknownFlagsPolicy) -> bool {
    let carry = evaluated.vector(flags_policy, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let keep_with_root = evaluated.vector(flags_policy, AccountingPolicy::KeepWithRoot, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let pinned = carry[5].cmp(&keep_with_root[5]);
    let corrupted_keep_with_root_position_six = carry[5];
    pinned != carry[5].cmp(&corrupted_keep_with_root_position_six)
}

/// 第八节判别力自证·Q1 第 7 位：钉住两格第 7 位的比较，把 `Carry` 的第 7 位减去 `h_new` × `r_tree_dim` 之后比较变了才算由绿转红。
fn accounting_question_position_seven_self_proof_turns_red(evaluated: &EvaluatedPoint, flags_policy: UnknownFlagsPolicy) -> bool {
    let carry = evaluated.vector(flags_policy, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let keep_with_root = evaluated.vector(flags_policy, AccountingPolicy::KeepWithRoot, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let pinned = carry[6].cmp(&keep_with_root[6]);
    let corrupted_carry_position_seven = carry[6] - evaluated.point.unknown_writable_head_count * evaluated.point.tree_dimension_statistic_count;
    pinned != corrupted_carry_position_seven.cmp(&keep_with_root[6])
}

/// 第二次跑第八节新自证（`dangling-u5`）：钉住 `Carry` 与 `NoCarry` 第 6 位的比较，把 `NoCarry` 的 U5 换回空真读法（第 6 位减 1）之后比较变了才算由绿转红。
/// 跑在 `h_new = 2`、`lpc = 1`、`r_tree_dim = 1` 的点上：第一次跑的判据下 `Carry` 与 `NoCarry` 在这里第 6 位相等。
fn accounting_question_dangling_carried_rows_self_proof_turns_red(evaluated: &EvaluatedPoint, flags_policy: UnknownFlagsPolicy) -> bool {
    let carry = evaluated.vector(flags_policy, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let no_carry = evaluated.vector(flags_policy, AccountingPolicy::NoCarry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
    let pinned = carry[5].cmp(&no_carry[5]);
    let no_carry_position_six_under_the_vacuous_reading = no_carry[5].saturating_sub(1);
    pinned != carry[5].cmp(&no_carry_position_six_under_the_vacuous_reading)
}

/// 第二次跑第八节新自证的取样点：`h_new = 2`、`lpc = 1`、`r_tree_dim = 1`。
fn is_a_point_of_the_dangling_carried_rows_self_proof(point: &SamplePoint) -> bool {
    point.unknown_writable_head_count == 2 && point.initial_publish_changed_the_watermark && point.tree_dimension_statistic_count == 1
}

/// 第五节阳性对照表的每一行（`RejectEntry` 那一行有两条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PositiveControl {
    IgnoreViolatesSemantics,
    SkipFreesReferencedBlocks,
    RejectEntryCountedSemanticsAddsOne,
    RejectEntryNoCarryLosesTuples,
    ReadOnlyNeverPublishes,
    PerBitEqualsTheArmItReducesTo,
    CarryWritesTheUnknownRows,
    NoCarryLosesTuples,
    KeepWithRootKeepsTheInitialRow,
    /// 第二次跑第五节补的三条（新 U5，三条记账臂各一条），只在 `dangling-u5` 下跑。
    DanglingCarriedRowsCarryHolds,
    DanglingCarriedRowsNoCarryDangles,
    DanglingCarriedRowsKeepWithRootDangles,
}

/// 新 U5 的三条阳性对照（`dangling-u5` 下接在 `POSITIVE_CONTROLS` 后面跑）。
const DANGLING_CARRIED_ROWS_POSITIVE_CONTROLS: [PositiveControl; 3] =
    [PositiveControl::DanglingCarriedRowsCarryHolds, PositiveControl::DanglingCarriedRowsNoCarryDangles, PositiveControl::DanglingCarriedRowsKeepWithRootDangles];

fn positive_controls_of(judgement: Judgement) -> Vec<PositiveControl> {
    let mut controls = POSITIVE_CONTROLS.to_vec();
    if judgement == Judgement::DanglingCarriedRowsClause {
        controls.extend(DANGLING_CARRIED_ROWS_POSITIVE_CONTROLS);
    }
    controls
}

const POSITIVE_CONTROLS: [PositiveControl; 9] = [
    PositiveControl::IgnoreViolatesSemantics,
    PositiveControl::SkipFreesReferencedBlocks,
    PositiveControl::RejectEntryCountedSemanticsAddsOne,
    PositiveControl::RejectEntryNoCarryLosesTuples,
    PositiveControl::ReadOnlyNeverPublishes,
    PositiveControl::PerBitEqualsTheArmItReducesTo,
    PositiveControl::CarryWritesTheUnknownRows,
    PositiveControl::NoCarryLosesTuples,
    PositiveControl::KeepWithRootKeepsTheInitialRow,
];

impl PositiveControl {
    fn label(self) -> &'static str {
        match self {
            PositiveControl::IgnoreViolatesSemantics => "Ignore",
            PositiveControl::SkipFreesReferencedBlocks => "Skip",
            PositiveControl::RejectEntryCountedSemanticsAddsOne => "RejectEntry_counted_semantics_adds_one",
            PositiveControl::RejectEntryNoCarryLosesTuples => "RejectEntry_no_carry_loses_tuples",
            PositiveControl::ReadOnlyNeverPublishes => "ReadOnly",
            PositiveControl::PerBitEqualsTheArmItReducesTo => "PerBit",
            PositiveControl::CarryWritesTheUnknownRows => "Carry",
            PositiveControl::NoCarryLosesTuples => "NoCarry",
            PositiveControl::KeepWithRootKeepsTheInitialRow => "KeepWithRoot",
            PositiveControl::DanglingCarriedRowsCarryHolds => "DanglingU5_Carry",
            PositiveControl::DanglingCarriedRowsNoCarryDangles => "DanglingU5_NoCarry",
            PositiveControl::DanglingCarriedRowsKeepWithRootDangles => "DanglingU5_KeepWithRoot",
        }
    }
}

/// 第五节阳性对照，每条臂一条；交回 (这一点上查了几格, 其中不成立的几格)。
fn positive_control_failures(evaluated: &EvaluatedPoint, control: PositiveControl) -> (u64, u64) {
    let point = &evaluated.point;
    let mut checked = 0u64;
    let mut failures = 0u64;
    let mut judge = |holds: bool| {
        checked += 1;
        failures += u64::from(!holds);
    };
    let has_unknown_heads = point.unknown_writable_head_count > 0;
    let acting_flags: Vec<UnknownFlagsPolicy> = FLAGS_ARMS.iter().copied().filter(|flags_policy| accounting_arm_acts_under(*flags_policy, point)).collect();
    match control {
        PositiveControl::IgnoreViolatesSemantics => {
            for accounting_policy in ACCOUNTING_ARMS {
                let cell = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Ignore, accounting_policy);
                judge(cell.read_only_violations > 0 && cell.revived_dead_trees == 1);
            }
        }
        PositiveControl::SkipFreesReferencedBlocks if point.invisible_window_generations > 0 => {
            for accounting_policy in ACCOUNTING_ARMS {
                judge(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, accounting_policy).blocks_freed_while_referenced > 0);
            }
        }
        PositiveControl::RejectEntryCountedSemanticsAddsOne => {
            for accounting_policy in ACCOUNTING_ARMS {
                let counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
                let not_counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
                judge(counted[5] == not_counted[5] + 1);
            }
        }
        PositiveControl::RejectEntryNoCarryLosesTuples if has_unknown_heads && point.tree_dimension_statistic_count == 2 => {
            judge(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::NoCarry).rows_lost > 0);
        }
        PositiveControl::ReadOnlyNeverPublishes => {
            for accounting_policy in ACCOUNTING_ARMS {
                judge(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::ReadOnly, accounting_policy).writable_generations == 0);
            }
        }
        PositiveControl::PerBitEqualsTheArmItReducesTo => {
            let reduces_to = if point.trees_with_registered_bit == 0 { UnknownFlagsPolicy::ReadOnly } else { UnknownFlagsPolicy::Skip };
            for accounting_policy in ACCOUNTING_ARMS {
                for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
                    judge(evaluated.vector(UnknownFlagsPolicy::PerBit, accounting_policy, reading) == evaluated.vector(reduces_to, accounting_policy, reading));
                }
            }
        }
        PositiveControl::CarryWritesTheUnknownRows if has_unknown_heads => {
            for flags_policy in &acting_flags {
                let extra = point.unknown_writable_head_count * point.tree_dimension_statistic_count;
                let carry = evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::Carry).rows_written_this_generation;
                let no_carry = evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::NoCarry).rows_written_this_generation;
                judge(carry == no_carry + extra && extra > 0);
            }
        }
        PositiveControl::NoCarryLosesTuples if has_unknown_heads && (point.tree_dimension_statistic_count == 2 || !point.is_legal_under_todays_step_rows()) => {
            for flags_policy in &acting_flags {
                judge(evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::NoCarry).rows_lost > 0);
            }
        }
        PositiveControl::KeepWithRootKeepsTheInitialRow if has_unknown_heads && !point.initial_publish_changed_the_watermark => {
            for flags_policy in &acting_flags {
                let cell = evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::KeepWithRoot);
                let every_version_keeps_an_initial_row = cell.trajectory_unknown_rows_at_the_initial_txg.is_some_and(|trajectory| trajectory.positive_versions == point.published_generations);
                let row_generation_invariant_false = cell
                    .clause_false_trajectories
                    .iter()
                    .any(|(clause, trajectory)| *clause == ClauseUnit::InvariantGenerationEqualsTheVersionTxg && trajectory.is_some_and(|trajectory| trajectory.peak > 0));
                judge(every_version_keeps_an_initial_row && row_generation_invariant_false);
            }
        }
        // 新 U5 的三条（第二次跑第五节补的表）：`Carry` 每一版都为真，另两条每一版都为假（为正版数 = G）。
        PositiveControl::DanglingCarriedRowsCarryHolds if has_unknown_heads => {
            for flags_policy in &acting_flags {
                let trajectory = carried_rows_clause_false_trajectory_of(evaluated.outcome_under_the_arms(*flags_policy, AccountingPolicy::Carry));
                judge(trajectory.is_some_and(|trajectory| trajectory.peak == 0 && trajectory.positive_versions == 0 && trajectory.final_value == 0));
            }
        }
        PositiveControl::DanglingCarriedRowsNoCarryDangles | PositiveControl::DanglingCarriedRowsKeepWithRootDangles if has_unknown_heads => {
            let accounting_policy = if control == PositiveControl::DanglingCarriedRowsNoCarryDangles { AccountingPolicy::NoCarry } else { AccountingPolicy::KeepWithRoot };
            for flags_policy in &acting_flags {
                let trajectory = carried_rows_clause_false_trajectory_of(evaluated.outcome_under_the_arms(*flags_policy, accounting_policy));
                judge(trajectory.is_some_and(|trajectory| trajectory.peak == 1 && trajectory.positive_versions == point.published_generations && trajectory.final_value == 1));
            }
        }
        // 带条件的那几行在条件不成立的点上不查（第五节「跑在哪些点」）。
        PositiveControl::SkipFreesReferencedBlocks
        | PositiveControl::RejectEntryNoCarryLosesTuples
        | PositiveControl::CarryWritesTheUnknownRows
        | PositiveControl::NoCarryLosesTuples
        | PositiveControl::KeepWithRootKeepsTheInitialRow
        | PositiveControl::DanglingCarriedRowsCarryHolds
        | PositiveControl::DanglingCarriedRowsNoCarryDangles
        | PositiveControl::DanglingCarriedRowsKeepWithRootDangles => {}
    }
    (checked, failures)
}

/// U5 在 R_1 … R_G 上的为假轨迹；没有新版本（只读挂载）交回 `None`。
fn carried_rows_clause_false_trajectory_of(cell: &CellOutcome) -> Option<Trajectory> {
    cell.clause_false_trajectories
        .iter()
        .find(|(clause, _trajectory)| *clause == ClauseUnit::DecisionEightItemTwelveCarriedRows)
        .and_then(|(_clause, trajectory)| *trajectory)
}

/// 阴性对照甲：`w_gens = 0` 的点上任一格误释放 > 0 ⇒ 误释放是硬编码的。交回 (查了几格, 不成立几格)。
fn negative_control_no_window_failures(evaluated: &EvaluatedPoint) -> (u64, u64) {
    if evaluated.point.invisible_window_generations > 0 {
        return (0, 0);
    }
    let failures: u64 = evaluated.cells.iter().filter(|cell| cell.blocks_freed_while_referenced > 0).count().try_into().expect("计数装得进 u64");
    (evaluated.cells.len().try_into().expect("计数装得进 u64"), failures)
}

/// 阴性对照乙：`h_new = 0` 的点上，同一 flags 臂下三条记账格七位（判的两种读法）有一位不同，或某条 U 为假 ⇒ 记账效应是硬编码的。
/// 「某条 U」只取读法 B1 下计的那几条（U1–U11，登记第十二节修订）：U12、U13 只在 B2 下计，是附带量，
/// 由 `negative_control_no_unknown_heads_incidental_clauses_counting_the_judged_items` 照报、不判。
fn negative_control_no_unknown_heads_failures(evaluated: &EvaluatedPoint) -> (u64, u64) {
    if evaluated.point.unknown_writable_head_count > 0 {
        return (0, 0);
    }
    let mut checked = 0u64;
    let mut failures = 0u64;
    for flags_policy in FLAGS_ARMS {
        for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
            let vectors: Vec<LexicographicVector> = ACCOUNTING_ARMS.iter().map(|accounting_policy| evaluated.vector(flags_policy, *accounting_policy, reading)).collect();
            checked += 1;
            failures += u64::from(first_separating_position(&vectors).is_some());
        }
    }
    for cell in &evaluated.cells {
        for (clause, trajectory) in &cell.clause_false_trajectories {
            if !clause.counts_when_the_judged_items_are_excluded() {
                continue;
            }
            checked += 1;
            failures += u64::from(trajectory.is_some_and(|trajectory| trajectory.peak > 0));
        }
    }
    (checked, failures)
}

/// 阴性对照乙的附带量：`h_new = 0` 的点上 U12（任一版为假）与 U13（按臂判）各有几格为假。照报、不判。
/// 交回 (查了几格, U12 为假几格, U13 为假几格)。
fn negative_control_no_unknown_heads_incidental_clauses_counting_the_judged_items(evaluated: &EvaluatedPoint) -> (u64, u64, u64) {
    if evaluated.point.unknown_writable_head_count > 0 {
        return (0, 0, 0);
    }
    let mut checked = 0u64;
    let mut carry_item_false = 0u64;
    let mut reject_item_false = 0u64;
    for cell in &evaluated.cells {
        checked += 1;
        for (clause, trajectory) in &cell.clause_false_trajectories {
            match clause {
                ClauseUnit::DecisionEightItemEightCarryUnknownRows => carry_item_false += u64::from(trajectory.is_some_and(|trajectory| trajectory.peak > 0)),
                ClauseUnit::DecisionEightItemEightRejectUnknownBits => reject_item_false += u64::from(cell.flags_policy != UnknownFlagsPolicy::RejectEntry),
                ClauseUnit::DecisionFiveItemTwoGenerationAndSteps
                | ClauseUnit::DecisionFiveItemFourWatermarkRows
                | ClauseUnit::DecisionEightItemOneWholeBatch
                | ClauseUnit::DecisionEightItemSixWatermarkRowRule
                | ClauseUnit::DecisionEightItemTwelveCarriedRows
                | ClauseUnit::DecisionSixteenItemNineRewriteEachPublish
                | ClauseUnit::InvariantGenerationEqualsTheVersionTxg
                | ClauseUnit::InvariantWritableHeadHasAWatermarkRow
                | ClauseUnit::InvariantWatermarkNotBelowItsSteps
                | ClauseUnit::DecisionEighteenItemTwentyFourLineageSteps
                | ClauseUnit::DecisionTwentyThreeItemFourteenNoPointDeletion => {}
            }
        }
    }
    (checked, carry_item_false, reject_item_false)
}

/// 取样点只改一维（几何敏感性、阴性对照丙、`G` 的甲类锚点找配对的点用）。
#[derive(Clone, Copy, Debug)]
enum PointChange {
    Geometry(Geometry),
    RetiredKeptGenerationCount(u64),
    InvisibleWindowGenerations(u64),
    UnknownWritableHeads(u64),
    RegisteredTrees(u64),
    PublishedGenerations(u64),
    StepHistory(u64, bool),
    TreeDimensionStatistics(u64),
}

fn point_with(point: &SamplePoint, change: PointChange) -> SamplePoint {
    let mut changed = *point;
    match change {
        PointChange::Geometry(geometry) => changed.geometry = geometry,
        PointChange::RetiredKeptGenerationCount(retired_kept_generation_count) => changed.retired_kept_generation_count = retired_kept_generation_count,
        PointChange::InvisibleWindowGenerations(generations) => changed.invisible_window_generations = generations,
        PointChange::UnknownWritableHeads(head_count) => changed.unknown_writable_head_count = head_count,
        PointChange::RegisteredTrees(tree_count) => changed.trees_with_registered_bit = tree_count,
        PointChange::PublishedGenerations(generations) => changed.published_generations = generations,
        PointChange::StepHistory(earlier_step_count, initial_publish_changed_the_watermark) => {
            changed.earlier_step_count = earlier_step_count;
            changed.initial_publish_changed_the_watermark = initial_publish_changed_the_watermark;
        }
        PointChange::TreeDimensionStatistics(statistic_count) => changed.tree_dimension_statistic_count = statistic_count,
    }
    changed
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SensitivityVerdict {
    AccountingWinners,
    FlagsWinnersAndRejectEntryVersusSkip,
}

/// 第八节「几何敏感性」表：每对两点方向相反，判决那一格在两点上都报。
struct SensitivityPair {
    name: &'static str,
    verdict: SensitivityVerdict,
    applies_to: fn(&SamplePoint) -> bool,
    change: PointChange,
}

fn sensitivity_pairs() -> Vec<SensitivityPair> {
    let mut pairs = vec![
        SensitivityPair {
            name: "lpc_0_to_1_where_h_new_positive_and_r_tree_dim_1",
            verdict: SensitivityVerdict::AccountingWinners,
            applies_to: |point| point.unknown_writable_head_count > 0 && point.tree_dimension_statistic_count == 1 && !point.initial_publish_changed_the_watermark,
            change: PointChange::StepHistory(0, true),
        },
        SensitivityPair { name: "r_tree_dim_1_to_2", verdict: SensitivityVerdict::AccountingWinners, applies_to: |point| point.tree_dimension_statistic_count == 1, change: PointChange::TreeDimensionStatistics(2) },
        SensitivityPair { name: "registered_0_to_4", verdict: SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip, applies_to: |point| point.trees_with_registered_bit == 0, change: PointChange::RegisteredTrees(UNKNOWN_TREE_COUNT) },
    ];
    // lpc 0 → 1 保留各点自己的 s：StepHistory 的 s 在用时按那一点改写（见 `partner_of`）。
    for (earlier_step_count, initial_publish_changed_the_watermark) in STEP_HISTORY_VALUES.into_iter().skip(1) {
        pairs.push(SensitivityPair {
            name: match (earlier_step_count, initial_publish_changed_the_watermark) {
                (0, true) => "s_lpc_0_0_to_0_1",
                (1, false) => "s_lpc_0_0_to_1_0",
                (1, true) => "s_lpc_0_0_to_1_1",
                (3, false) => "s_lpc_0_0_to_3_0",
                _ => "s_lpc_0_0_to_3_1",
            },
            verdict: SensitivityVerdict::AccountingWinners,
            applies_to: |point| !point.is_legal_under_todays_step_rows(),
            change: PointChange::StepHistory(earlier_step_count, initial_publish_changed_the_watermark),
        });
    }
    for (name, generations) in [("w_gens_0_to_1", 1), ("w_gens_0_to_3", 3), ("w_gens_0_to_8", 8)] {
        pairs.push(SensitivityPair { name, verdict: SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip, applies_to: |point| point.invisible_window_generations == 0, change: PointChange::InvisibleWindowGenerations(generations) });
    }
    for (name, head_count) in [("h_new_0_to_2", 2), ("h_new_0_to_4", 4)] {
        pairs.push(SensitivityPair { name, verdict: SensitivityVerdict::AccountingWinners, applies_to: |point| point.unknown_writable_head_count == 0, change: PointChange::UnknownWritableHeads(head_count) });
    }
    for verdict in [SensitivityVerdict::AccountingWinners, SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip] {
        pairs.push(SensitivityPair {
            name: "geometry_e113_to_implementation_two_devices",
            verdict,
            applies_to: |point| point.geometry == Geometry::UnknownTreeFullArms,
            change: PointChange::Geometry(Geometry::ImplementationTwoDevices),
        });
        pairs.push(SensitivityPair { name: "g_gens_1_to_10", verdict, applies_to: |point| point.published_generations == 1, change: PointChange::PublishedGenerations(10) });
        pairs.push(SensitivityPair { name: "k_keep_1_to_20", verdict, applies_to: |point| point.retired_kept_generation_count == 1, change: PointChange::RetiredKeptGenerationCount(20) });
    }
    pairs
}

/// 配对的那一点：lpc 0 → 1 那一对只翻 lpc、保留这一点自己的 s；别的照 `change` 改。
fn partner_of(pair: &SensitivityPair, point: &SamplePoint) -> Option<SamplePoint> {
    if !(pair.applies_to)(point) {
        return None;
    }
    let change = match (pair.name, pair.change) {
        ("lpc_0_to_1_where_h_new_positive_and_r_tree_dim_1", PointChange::StepHistory(_, _)) => PointChange::StepHistory(point.earlier_step_count, true),
        (_, change) => change,
    };
    Some(point_with(point, change))
}

/// 第七节乙类（独立算出、用命令核过）与甲类的绝对值，逐条在全部取样点里对；交回 (名字, 查了几格, 对不上几格)。
fn anchor_failures(evaluated_points: &[EvaluatedPoint]) -> Vec<(String, u64, u64)> {
    let mut results = Vec::new();
    let mut check = |name: String, select: &dyn Fn(&SamplePoint) -> bool, value_of: &dyn Fn(&EvaluatedPoint) -> u64, expected: &dyn Fn(&SamplePoint) -> u64| {
        let mut checked = 0u64;
        let mut mismatches = 0u64;
        for evaluated in evaluated_points.iter().filter(|evaluated| select(&evaluated.point)) {
            checked += 1;
            mismatches += u64::from(value_of(evaluated) != expected(&evaluated.point));
        }
        results.push((name, checked, mismatches));
    };
    check(
        "own_rows_r_tree_dim_1_e113_17_implementation_15".to_string(),
        &|point| point.tree_dimension_statistic_count == 1,
        &|evaluated| evaluated.point.geometry.rows_without_tree_dimension() + evaluated.point.geometry.known_writable_head_count(),
        &|point| match point.geometry {
            Geometry::UnknownTreeFullArms => 17,
            Geometry::ImplementationTwoDevices => 15,
        },
    );
    check(
        "skip_blocks_freed_w_gens_0_1_3_8_is_0_64_192_512".to_string(),
        &|_point| true,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).blocks_freed_while_referenced,
        &|point| match point.invisible_window_generations {
            0 => 0,
            1 => 64,
            3 => 192,
            _ => 512,
        },
    );
    check(
        "ignore_rdonly_violations_g_gens_1_4_5_10_is_1_4_5_10".to_string(),
        &|_point| true,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Ignore, AccountingPolicy::Carry).read_only_violations,
        &|point| match point.published_generations {
            1 => 1,
            4 => 4,
            5 => 5,
            _ => 10,
        },
    );
    for (head_count, statistic_count, extra) in [(2u64, 1u64, 2u64), (4, 2, 8)] {
        // 互比写成加法（`carry == no_carry + extra`），成立记 1、钉成 1。
        check(
            format!("w1_carry_equals_no_carry_plus_{extra}_at_h_new_{head_count}_r_tree_dim_{statistic_count}"),
            &move |point| point.unknown_writable_head_count == head_count && point.tree_dimension_statistic_count == statistic_count,
            &move |evaluated| {
                let carry = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).rows_written_this_generation;
                let no_carry = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry).rows_written_this_generation;
                u64::from(carry == no_carry + extra)
            },
            &|_point| 1,
        );
    }
    check(
        "baseline_reject_entry_carry_w1_e113_h_new_2_r_tree_dim_1_is_19".to_string(),
        &|point| point.geometry == Geometry::UnknownTreeFullArms && point.unknown_writable_head_count == 2 && point.tree_dimension_statistic_count == 1,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry).rows_written_this_generation,
        &|_point| 19,
    );
    let closed_form_points: [(Geometry, u64, u64, bool, u64, RowsWrittenReading, [u64; 3]); 5] = [
        (Geometry::UnknownTreeFullArms, 2, 1, true, 1, RowsWrittenReading::AllRowsOfTheVersion, [23, 21, 21]),
        (Geometry::ImplementationTwoDevices, 4, 3, false, 2, RowsWrittenReading::AllRowsOfTheVersion, [37, 29, 37]),
        (Geometry::ImplementationTwoDevices, 4, 3, false, 2, RowsWrittenReading::ThisGenerationRowsOnly, [24, 16, 16]),
        (Geometry::ImplementationTwoDevices, 2, 0, false, 1, RowsWrittenReading::AllRowsOfTheVersion, [18, 16, 18]),
        (Geometry::ImplementationTwoDevices, 2, 0, false, 1, RowsWrittenReading::ThisGenerationRowsOnly, [17, 15, 15]),
    ];
    for (geometry, head_count, earlier_step_count, initial_publish_changed_the_watermark, statistic_count, rows_written, expected_by_accounting_arm) in closed_form_points {
        for (accounting_policy, expected) in ACCOUNTING_ARMS.into_iter().zip(expected_by_accounting_arm) {
            check(
                format!(
                    "closed_form_{}_{}_h_new_{head_count}_s_{earlier_step_count}_lpc_{}_r_tree_dim_{statistic_count}_{}_is_{expected}",
                    match rows_written {
                        RowsWrittenReading::AllRowsOfTheVersion => "w2",
                        RowsWrittenReading::ThisGenerationRowsOnly => "w1",
                    },
                    geometry.label(),
                    u8::from(initial_publish_changed_the_watermark),
                    accounting_label(accounting_policy)
                ),
                &move |point| {
                    point.geometry == geometry
                        && point.unknown_writable_head_count == head_count
                        && point.earlier_step_count == earlier_step_count
                        && point.initial_publish_changed_the_watermark == initial_publish_changed_the_watermark
                        && point.tree_dimension_statistic_count == statistic_count
                },
                &move |evaluated| {
                    let cell = evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, accounting_policy);
                    match rows_written {
                        RowsWrittenReading::AllRowsOfTheVersion => cell.rows_written_all_rows,
                        RowsWrittenReading::ThisGenerationRowsOnly => cell.rows_written_this_generation,
                    }
                },
                &move |_point| expected,
            );
        }
    }
    check(
        "clause_anchor_implementation_h_new_0_r_tree_dim_1_w1_is_15".to_string(),
        &|point| point.geometry == Geometry::ImplementationTwoDevices && point.unknown_writable_head_count == 0 && point.tree_dimension_statistic_count == 1,
        &|evaluated| evaluated.outcome_under_the_arms(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).rows_written_this_generation,
        &|_point| 15,
    );
    results
}

/// 第二次跑第十节新表的形态（`form=`）：从被判点上的胜者集合列表机械推出。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AcceptanceForm {
    UniqueCarry,
    UniqueOtherArm,
    TieWithCarry,
    TieWithoutCarry,
    DependsOnPointCarryEverywhere,
    DependsOnPointCarryNowhere,
    DependsOnPointMixed,
    NoJudgedPoints,
}

impl AcceptanceForm {
    fn label(self) -> &'static str {
        match self {
            AcceptanceForm::UniqueCarry => "unique_carry",
            AcceptanceForm::UniqueOtherArm => "unique_other_arm",
            AcceptanceForm::TieWithCarry => "tie_with_carry",
            AcceptanceForm::TieWithoutCarry => "tie_without_carry",
            AcceptanceForm::DependsOnPointCarryEverywhere => "depends_on_point_carry_everywhere",
            AcceptanceForm::DependsOnPointCarryNowhere => "depends_on_point_carry_nowhere",
            AcceptanceForm::DependsOnPointMixed => "depends_on_point_mixed",
            AcceptanceForm::NoJudgedPoints => "no_judged_points",
        }
    }
}

/// 第十节新表里 ㊀ 的后果（`consequence=`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Consequence {
    /// ㊀ 不翻面。
    CarryStands,
    /// ㊀ 不翻面，另几条与 `Carry` 在第 1–7 位都相等，作为「打平的另一条路」交用户知情。
    CarryStandsTieKnownToUser,
    /// ㊀ 翻面成这一条臂。
    FlipsTo(AccountingPolicy),
    /// ㊀ 翻面，换成哪一条交用户判。
    FlipChoiceToUser,
    /// 不判翻面也不判站住，交用户。
    LeftToUser,
    /// 没有被判的点，读不到，作废。
    NoJudgedPoints,
}

impl Consequence {
    fn label(self) -> String {
        match self {
            Consequence::CarryStands => "carry_stands".to_string(),
            Consequence::CarryStandsTieKnownToUser => "carry_stands_tie_known_to_user".to_string(),
            Consequence::FlipsTo(accounting_policy) => format!("flips_to_{}", accounting_label(accounting_policy)),
            Consequence::FlipChoiceToUser => "flips_choice_to_user".to_string(),
            Consequence::LeftToUser => "left_to_user".to_string(),
            Consequence::NoJudgedPoints => "none".to_string(),
        }
    }
}

/// 一组取样点上 Q1 反向接受条款的判定（被判点数、不进判的 `h_new = 0` 点数、形态、后果、各胜者集合的点数）。
struct AcceptanceTally {
    judged_points: u64,
    excluded_points_without_unknown_writable_heads: u64,
    form: AcceptanceForm,
    consequence: Consequence,
    winner_sets: BTreeMap<Vec<AccountingPolicy>, u64>,
}

/// 被判点上至少有一个胜者集合时，按集合有几种、含不含 `Carry` 归形态。
fn form_of_the_winner_sets(sets: &BTreeMap<Vec<AccountingPolicy>, u64>) -> (AcceptanceForm, Consequence) {
    if sets.len() == 1 {
        let only = sets.keys().next().expect("恰有一种集合");
        if only.as_slice() == [AccountingPolicy::Carry] {
            (AcceptanceForm::UniqueCarry, Consequence::CarryStands)
        } else if only.len() == 1 {
            (AcceptanceForm::UniqueOtherArm, Consequence::FlipsTo(only[0]))
        } else if only.contains(&AccountingPolicy::Carry) {
            (AcceptanceForm::TieWithCarry, Consequence::CarryStandsTieKnownToUser)
        } else {
            (AcceptanceForm::TieWithoutCarry, Consequence::FlipChoiceToUser)
        }
    } else {
        let sets_with_carry = sets.keys().filter(|set| set.contains(&AccountingPolicy::Carry)).count();
        if sets_with_carry == sets.len() {
            (AcceptanceForm::DependsOnPointCarryEverywhere, Consequence::CarryStands)
        } else if sets_with_carry == 0 {
            (AcceptanceForm::DependsOnPointCarryNowhere, Consequence::FlipChoiceToUser)
        } else {
            (AcceptanceForm::DependsOnPointMixed, Consequence::LeftToUser)
        }
    }
}

/// 第二次跑第十节：只判 `h_new > 0` 的点（输入每个点一对：`h_new > 0` 与这个点十五格总胜者里的记账臂集合），`h_new = 0` 的点照数、不进判。
fn carry_acceptance_of(points: &[(bool, Vec<AccountingPolicy>)]) -> AcceptanceTally {
    let judged: Vec<&Vec<AccountingPolicy>> = points.iter().filter(|(has_unknown_writable_heads, _winners)| *has_unknown_writable_heads).map(|(_has_unknown_writable_heads, winners)| winners).collect();
    let mut winner_sets: BTreeMap<Vec<AccountingPolicy>, u64> = BTreeMap::new();
    for winners in &judged {
        *winner_sets.entry((*winners).clone()).or_default() += 1;
    }
    let judged_points: u64 = judged.len().try_into().expect("计数装得进 u64");
    let total_points: u64 = points.len().try_into().expect("计数装得进 u64");
    let (form, consequence) = if judged.is_empty() { (AcceptanceForm::NoJudgedPoints, Consequence::NoJudgedPoints) } else { form_of_the_winner_sets(&winner_sets) };
    AcceptanceTally { judged_points, excluded_points_without_unknown_writable_heads: total_points - judged_points, form, consequence, winner_sets }
}

/// 一个点十五格总胜者里的记账臂（去重、按臂次序）。
fn overall_winner_accounting_arms(analysis: &PointAnalysis) -> Vec<AccountingPolicy> {
    analysis.overall_winners.iter().map(|(_flags_policy, accounting_policy)| *accounting_policy).collect::<BTreeSet<AccountingPolicy>>().into_iter().collect()
}

fn render_acceptance_line(judgement: Judgement, group: &str, judged: bool, tally: &AcceptanceTally) -> String {
    let winner_sets =
        tally.winner_sets.iter().map(|(set, count)| format!("{}:{count}", join_labels(set, |policy| accounting_label(*policy).to_string()))).collect::<Vec<String>>().join("|");
    format!(
        "name=verdict check=q1_carry_acceptance_form judgement={} reading={} group={group} judged={} judged_points={} excluded_points_h_new_0={} form={} consequence={} winner_sets={winner_sets}",
        judgement.label(),
        JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED.label(),
        if judged { "yes" } else { "no" },
        tally.judged_points,
        tally.excluded_points_without_unknown_writable_heads,
        tally.form.label(),
        tally.consequence.label(),
    )
}

/// 打哪几组、判不判：判 ㊀ 只看 `legal_pooled`，两个 `r_tree_dim` 分组照报；E113 口径两组单报、标不判（登记第二次跑第一节读法第 11 条）。
const ACCEPTANCE_GROUPS: [(&str, bool); 5] =
    [("legal_r_tree_dim_1", true), ("legal_r_tree_dim_2", true), ("legal_pooled", true), ("e113_style_r_tree_dim_1", false), ("e113_style_r_tree_dim_2", false)];

/// 每组的反向接受条款判定：每个点取 Q2「计」读法下十五格总胜者里的记账臂（第一次跑产物判 Q1 的就是这一类，读法 B1、W2）。
fn acceptance_tallies(evaluated_points: &[EvaluatedPoint]) -> BTreeMap<&'static str, AcceptanceTally> {
    let mut points_by_group: BTreeMap<String, Vec<(bool, Vec<AccountingPolicy>)>> = BTreeMap::new();
    for evaluated in evaluated_points {
        let point = &evaluated.point;
        let acceptance_point = (point.unknown_writable_head_count > 0, overall_winner_accounting_arms(&analyse_point(evaluated, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)));
        points_by_group.entry(group_of(point)).or_default().push(acceptance_point.clone());
        if point.is_legal_under_todays_step_rows() {
            points_by_group.entry("legal_pooled".to_string()).or_default().push(acceptance_point);
        }
    }
    ACCEPTANCE_GROUPS.iter().map(|(group, _judged)| (*group, carry_acceptance_of(points_by_group.get(*group).map(Vec::as_slice).unwrap_or(&[])))).collect()
}

/// 登记第二次跑第七节乙类：每个 `r_tree_dim` 分组 (被判点, 不进判的点) 是 (3200, 1600)，合起来 (6400, 3200)。
const EXPECTED_ACCEPTANCE_POINT_COUNTS: [(&str, u64, u64); 3] = [("legal_r_tree_dim_1", 3200, 1600), ("legal_r_tree_dim_2", 3200, 1600), ("legal_pooled", 6400, 3200)];

fn all_readings() -> Vec<Reading> {
    [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED].into_iter().chain(INCIDENTAL_READINGS).collect()
}

/// 停机条款第 6 项的格一侧：两种判据下 U5 以外的条款轨迹相同、七位向量除第 6 位外相同、第 6 位之差等于新旧 U5 记数之差（六种读法都比）。
fn cells_differ_only_in_the_carried_rows_clause(vacuous: &CellOutcome, dangling: &CellOutcome, point: &SamplePoint) -> bool {
    let other_clause_trajectories_agree = vacuous
        .clause_false_trajectories
        .iter()
        .zip(&dangling.clause_false_trajectories)
        .all(|(before, after)| before.0 == after.0 && (before.0 == ClauseUnit::DecisionEightItemTwelveCarriedRows || before.1 == after.1));
    let carried_rows_clause_count_of = |cell: &CellOutcome| u64::from(carried_rows_clause_false_trajectory_of(cell).is_some_and(|trajectory| trajectory.peak > 0));
    let (vacuous_carried_rows_clause_count, dangling_carried_rows_clause_count) = (carried_rows_clause_count_of(vacuous), carried_rows_clause_count_of(dangling));
    let vectors_agree = all_readings().into_iter().all(|reading| {
        let before = vector_under(vacuous, point, reading);
        let after = vector_under(dangling, point, reading);
        [0usize, 1, 2, 3, 4, 6].iter().all(|position| before[*position] == after[*position]) && after[5] + vacuous_carried_rows_clause_count == before[5] + dangling_carried_rows_clause_count
    });
    other_clause_trajectories_agree && vectors_agree
}

/// 停机条款第 6 项的 Q2 一侧：六种读法下，每条记账臂下 flags 臂的比较与十五格总胜者里的 flags 臂集合在两种判据下相同。
fn flags_question_judgements_agree(vacuous: &EvaluatedPoint, dangling: &EvaluatedPoint) -> bool {
    all_readings().into_iter().all(|reading| {
        let before = analyse_point(vacuous, reading);
        let after = analyse_point(dangling, reading);
        before.flags_winners_by_accounting_arm == after.flags_winners_by_accounting_arm && flags_winners_signature(&before).1 == flags_winners_signature(&after).1
    })
}

/// 停机条款第 7 项：新 U5 照 N（R 里不认识的树的行数）判与照 `h_new > 0` 判，在这一点上一致。
fn carried_rows_object_count_matches_the_unknown_writable_heads(point: &SamplePoint) -> bool {
    (unknown_tree_row_count_of(&initial_version(point)) > 0) == (point.unknown_writable_head_count > 0)
}

struct ProductWriter<Output: std::io::Write> {
    emitter: Emitter,
    output: Output,
}

impl<Output: std::io::Write> ProductWriter<Output> {
    fn line(&mut self, body: &str) {
        writeln!(self.output, "{}", self.emitter.emit_raw(body)).expect("产物写得进标准输出");
    }

    fn finish(mut self) {
        let done_line = self.emitter.finish();
        writeln!(self.output, "{done_line}").expect("产物写得进标准输出");
        self.output.flush().expect("产物写得进标准输出");
    }
}

/// (种类, 读法, 分组, 臂, 取值) → 落在这一格的取样点。
type SummaryKey = (&'static str, String, String, String, String);

fn add_analysis_to_summaries(summaries: &mut BTreeMap<SummaryKey, PointSetDescription>, point: &SamplePoint, reading: Reading, analysis: &PointAnalysis) {
    let reading_label = reading.label();
    let group = group_of(point);
    let mut add = |kind: &'static str, arm: String, value: String| {
        summaries.entry((kind, reading_label.clone(), group.clone(), arm, value)).or_default().add(point);
    };
    for arm in &analysis.accounting_winners_by_flags_arm {
        let flags = flags_label(arm.flags_policy);
        add("q1_first_separating", flags.to_string(), render_position(arm.first_separating_position));
        add("q1_winners", flags.to_string(), join_labels(&arm.winners, |policy| accounting_label(*policy).to_string()));
        for ((left, right), position) in ACCOUNTING_PAIRS.iter().zip(&arm.pair_first_separating_positions) {
            add("q1_pair_first_separating", format!("{flags}:{}~{}", accounting_label(*left), accounting_label(*right)), render_position(*position));
        }
    }
    let overall_accounting_arms: Vec<AccountingPolicy> =
        analysis.overall_winners.iter().map(|(_flags_policy, accounting_policy)| *accounting_policy).collect::<BTreeSet<AccountingPolicy>>().into_iter().collect();
    add("q1_overall_winner_accounting_arms", "all".to_string(), join_labels(&overall_accounting_arms, |policy| accounting_label(*policy).to_string()));
    for arm in &analysis.flags_winners_by_accounting_arm {
        let accounting = accounting_label(arm.accounting_policy).to_string();
        add("q2_reject_vs_skip_first_separating", accounting.clone(), render_position(arm.reject_entry_versus_skip_first_separating_position));
        add("q2_position_six", accounting.clone(), render_ordering_of_reject_entry_against_skip(arm.position_six_reject_entry_versus_skip).to_string());
        add("q2_separable", accounting.clone(), u8::from(arm.separable_at_position_six).to_string());
        add("q2_flags_winners", accounting, join_labels(&arm.flags_winners, |policy| flags_label(*policy).to_string()));
    }
    let overall_flags_arms: Vec<UnknownFlagsPolicy> =
        analysis.overall_winners.iter().map(|(flags_policy, _accounting_policy)| *flags_policy).collect::<BTreeSet<UnknownFlagsPolicy>>().into_iter().collect();
    add("q2_overall_winner_flags_arms", "all".to_string(), join_labels(&overall_flags_arms, |policy| flags_label(*policy).to_string()));
}

/// 每个分组的 Q2 计数：几个点，几个点上有一条记账臂 `RejectEntry` 对 `Skip` 在第 6 位分得开（两种读法各数），几个点上 flags 胜者随读法换档。
#[derive(Default)]
struct FlagsQuestionGroupTally {
    points: u64,
    separable_points_counted: u64,
    separable_points_not_counted: u64,
    changed_points_judged: u64,
    changed_points_incidental_counting_the_judged_items: u64,
}

fn render_verdict_holds(failures: u64) -> &'static str {
    if failures == 0 {
        "true"
    } else {
        "false"
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let judgement = match Judgement::from_arguments(&arguments) {
        Ok(judgement) => judgement,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let standard_output = std::io::stdout();
    let mut writer = ProductWriter { emitter: Emitter::new(), output: std::io::BufWriter::new(standard_output.lock()) };
    let evaluated_points: Vec<EvaluatedPoint> = all_sample_points().into_iter().map(|point| evaluate_point(point, judgement)).collect();
    let point_count: u64 = evaluated_points.len().try_into().expect("计数装得进 u64");
    // 不带参数（第一次跑的判据）时配置行一个字不加，产物逐字节等于第一次跑的产物；第二次跑的判据另报这两个字段。
    let judgement_fields = match judgement {
        Judgement::VacuousCarriedRowsClause => String::new(),
        Judgement::DanglingCarriedRowsClause => format!(
            " judgement={} u5_predicate=n_zero_true_n_positive_nothing_written_false_else_every_written_row_at_this_txg acceptance_judged_points=h_new_positive_legal_only",
            judgement.label()
        ),
    };
    writer.line(&format!(
        "name=config experiment=e186_unknown_tree_accounting_rederive model=arithmetic threads=1 points={point_count} cells_per_point={CELLS_PER_POINT} \
         initial_txg={INITIAL_VERSION_TXG} t_known={KNOWN_TREE_COUNT} t_new={UNKNOWN_TREE_COUNT} t_new_rdonly={UNKNOWN_READ_ONLY_TREE_COUNT} t_new_dead={UNKNOWN_HALF_DELETED_TREE_COUNT} \
         blocks_per_gen={BLOCKS_BORN_PER_GENERATION} e113_rows_without_tree_dimension={E113_ROWS_WITHOUT_TREE_DIMENSION} e113_h_known={E113_KNOWN_WRITABLE_HEAD_COUNT} \
         e113_known_head_steps={E113_KNOWN_HEAD_STEP_ROWS} crates_pool_wide_rows={CRATES_POOL_WIDE_ROWS} crates_per_device_rows={CRATES_PER_DEVICE_ROWS} \
         crates_devices={CRATES_DEVICE_COUNT} crates_h_known={CRATES_KNOWN_WRITABLE_HEAD_COUNT} crates_known_head_steps={CRATES_KNOWN_HEAD_STEP_ROWS} \
         judged_readings={}+{} incidental_readings={} \
         vector_positions=blocks_freed_while_referenced,rdonly_violations+dead_revived,rows_lost_tuples,silent,g_gens_minus_writable_gens,clause_units_to_change,rows_written_per_gen \
         trajectory_fields=peak/positive_versions/final_over_r1_to_rg clauses_false=listed_when_peak_is_1_others_are_0/0/0_u13_judged_per_arm \
         step_recognition=watermark_change_history_by_statistic_and_tree{judgement_fields}",
        JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED.label(),
        JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED.label(),
        INCIDENTAL_READINGS.iter().map(|reading| reading.label()).collect::<Vec<String>>().join("+"),
    ));

    let mut summaries: BTreeMap<SummaryKey, PointSetDescription> = BTreeMap::new();
    let mut flags_question_tallies: BTreeMap<String, FlagsQuestionGroupTally> = BTreeMap::new();
    let mut ranked_cell_count = 0u64;
    let mut ledger_mismatches_excluding_the_judged_items = 0u64;
    let mut ledger_mismatches_counting_the_judged_items = 0u64;
    let mut points_where_the_accounting_winners_depend_on_the_reject_entry_reading = 0u64;
    let mut non_distinguishing_points_counted = 0u64;
    let mut non_distinguishing_points_not_counted = 0u64;
    let mut analyses: Vec<(PointAnalysis, PointAnalysis)> = Vec::with_capacity(evaluated_points.len());    for (point_index, evaluated) in evaluated_points.iter().enumerate() {
        let point = &evaluated.point;
        let cell_count: u64 = evaluated.cells.len().try_into().expect("计数装得进 u64");
        ranked_cell_count += cell_count;
        for cell in &evaluated.cells {
            writer.line(&render_cell_line(point_index, evaluated, cell));
            ledger_mismatches_excluding_the_judged_items += u64::from(cell.clause_units_excluding_the_judged_items != clause_units_from_the_hand_ledger(point, cell.flags_policy, cell.accounting_policy, ClauseReading::ExcludesTheJudgedItems, judgement));
            ledger_mismatches_counting_the_judged_items += u64::from(cell.clause_units_counting_the_judged_items != clause_units_from_the_hand_ledger(point, cell.flags_policy, cell.accounting_policy, ClauseReading::CountsTheJudgedItems, judgement));
        }
        let counted = analyse_point(evaluated, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
        let not_counted = analyse_point(evaluated, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
        points_where_the_accounting_winners_depend_on_the_reject_entry_reading += u64::from(counted.accounting_winners_by_flags_arm != not_counted.accounting_winners_by_flags_arm);
        non_distinguishing_points_counted += u64::from(counted.all_fifteen_cells_equal);
        non_distinguishing_points_not_counted += u64::from(not_counted.all_fifteen_cells_equal);
        writer.line(&render_point_verdict_line(point_index, evaluated, &counted, &not_counted));
        add_analysis_to_summaries(&mut summaries, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, &counted);
        add_analysis_to_summaries(&mut summaries, point, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED, &not_counted);
        let incidental_analyses: Vec<PointAnalysis> = INCIDENTAL_READINGS.iter().map(|reading| analyse_point(evaluated, *reading)).collect();
        for (reading, analysis) in INCIDENTAL_READINGS.iter().zip(&incidental_analyses) {
            add_analysis_to_summaries(&mut summaries, point, *reading, analysis);
        }
        let changed_judged = flags_winners_signature(&counted) != flags_winners_signature(&not_counted);
        let changed_incidental_counting_the_judged_items = flags_winners_signature(&incidental_analyses[0]) != flags_winners_signature(&incidental_analyses[1]);
        summaries.entry(("q2_flags_winners_changed", "b1_w2".to_string(), group_of(point), "all".to_string(), u8::from(changed_judged).to_string())).or_default().add(point);
        summaries.entry(("q2_flags_winners_changed", "b2_w2".to_string(), group_of(point), "all".to_string(), u8::from(changed_incidental_counting_the_judged_items).to_string())).or_default().add(point);
        let tally = flags_question_tallies.entry(group_of(point)).or_default();
        tally.points += 1;
        tally.separable_points_counted += u64::from(counted.flags_winners_by_accounting_arm.iter().any(|arm| arm.separable_at_position_six));
        tally.separable_points_not_counted += u64::from(not_counted.flags_winners_by_accounting_arm.iter().any(|arm| arm.separable_at_position_six));
        tally.changed_points_judged += u64::from(changed_judged);
        tally.changed_points_incidental_counting_the_judged_items += u64::from(changed_incidental_counting_the_judged_items);
        analyses.push((counted, not_counted));
    }

    for ((kind, reading_label, group, arm, value), description) in &summaries {
        writer.line(&format!("name=summary kind={kind} reading={reading_label} group={group} arm={arm} value={value} {}", description.render("")));
    }

    let expected_cell_count = EXPECTED_POINT_COUNT * CELLS_PER_POINT;
    let count_holds = point_count == EXPECTED_POINT_COUNT && ranked_cell_count == expected_cell_count;
    writer.line(&format!(
        "name=verdict check=point_and_cell_count points={point_count} expected_points={EXPECTED_POINT_COUNT} ranked_cells={ranked_cell_count} expected_cells={expected_cell_count} count_ok={count_holds}"
    ));

    for control in positive_controls_of(judgement) {
        let (checked, failures) = evaluated_points.iter().map(|evaluated| positive_control_failures(evaluated, control)).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
        writer.line(&format!("name=verdict check=positive_control control={} checked={checked} failures={failures} holds={}", control.label(), render_verdict_holds(failures)));
    }
    let (checked, failures) = evaluated_points.iter().map(negative_control_no_window_failures).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
    writer.line(&format!("name=verdict check=negative_control_no_window_no_wrong_free checked={checked} failures={failures} holds={}", render_verdict_holds(failures)));
    let (checked, failures) = evaluated_points.iter().map(negative_control_no_unknown_heads_failures).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
    writer.line(&format!("name=verdict check=negative_control_no_unknown_heads_no_accounting_effect clauses_judged=b1_only checked={checked} failures={failures} holds={}", render_verdict_holds(failures)));
    let (incidental_checked, carry_item_false, reject_item_false) = evaluated_points
        .iter()
        .map(negative_control_no_unknown_heads_incidental_clauses_counting_the_judged_items)
        .fold((0, 0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1, sum.2 + part.2));
    writer.line(&format!(
        "name=incidental check=negative_control_no_unknown_heads_b2_only_clauses judged=no cells={incidental_checked} u12_false_cells={carry_item_false} u13_false_cells={reject_item_false}"
    ));
    writer.line("name=incidental check=incidental_quantities judged=no fields=per_generation_rows_dropped,per_generation_rows_dropped_total,i313_syntactic_violations,crates_step_reading_differences source=cell_lines");

    let index_by_description: BTreeMap<String, usize> = evaluated_points.iter().enumerate().map(|(index, evaluated)| (evaluated.point.describe(), index)).collect();
    let partner_index = |point: &SamplePoint| -> usize { index_by_description[&point.describe()] };
    let all_readings: Vec<Reading> = [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED].into_iter().chain(INCIDENTAL_READINGS).collect();
    let mut kept_generation_count_checked = 0u64;
    let mut kept_generation_count_failures = 0u64;
    let mut published_generations_checked = 0u64;
    let mut published_generations_mismatches = 0u64;
    for evaluated in &evaluated_points {
        let point = &evaluated.point;
        if point.retired_kept_generation_count != RETIRED_KEPT_GENERATION_COUNT_VALUES[0] {
            let partner = &evaluated_points[partner_index(&point_with(point, PointChange::RetiredKeptGenerationCount(RETIRED_KEPT_GENERATION_COUNT_VALUES[0])))];
            for cell in &evaluated.cells {
                for reading in &all_readings {
                    kept_generation_count_checked += 1;
                    kept_generation_count_failures += u64::from(vector_under(cell, point, *reading) != partner.vector(cell.flags_policy, cell.accounting_policy, *reading));
                }
            }
        }
        if point.published_generations == 1 {
            let partner = &evaluated_points[partner_index(&point_with(point, PointChange::PublishedGenerations(10)))];
            for flags_policy in FLAGS_ARMS {
                published_generations_checked += 1;
                published_generations_mismatches += u64::from(evaluated.outcome_under_the_arms(flags_policy, AccountingPolicy::NoCarry).rows_lost != partner.outcome_under_the_arms(flags_policy, AccountingPolicy::NoCarry).rows_lost);
            }
        }
    }
    writer.line(&format!(
        "name=verdict check=negative_control_k_keep_not_read checked={kept_generation_count_checked} failures={kept_generation_count_failures} holds={}",
        render_verdict_holds(kept_generation_count_failures)
    ));
    writer.line(&format!(
        "name=verdict check=ledger_two_paths cells={ranked_cell_count} mismatches_b1={ledger_mismatches_excluding_the_judged_items} mismatches_b2={ledger_mismatches_counting_the_judged_items} holds={}",
        render_verdict_holds(ledger_mismatches_excluding_the_judged_items + ledger_mismatches_counting_the_judged_items)
    ));
    for (name, checked, mismatches) in anchor_failures(&evaluated_points) {
        writer.line(&format!("name=verdict check=anchor anchor={name} checked={checked} mismatches={mismatches} holds={}", render_verdict_holds(mismatches)));
    }
    writer.line(&format!(
        "name=verdict check=anchor anchor=no_carry_rows_lost_same_at_g_gens_1_and_10 checked={published_generations_checked} mismatches={published_generations_mismatches} holds={}",
        render_verdict_holds(published_generations_mismatches)
    ));
    writer.line(&format!(
        "name=verdict check=anchor anchor=today_tree_dimension_statistics_is_1 checked=1 mismatches={} holds={}",
        u64::from(TREE_DIMENSION_STATISTIC_COUNT_VALUES[0] != 1),
        render_verdict_holds(u64::from(TREE_DIMENSION_STATISTIC_COUNT_VALUES[0] != 1))
    ));

    // 停机条款第 1、2 项的模型一侧：实现两盘几何、只有认识的那一个头时，行数与台阶认法要与 crates/ 那两段读出的相同。
    let crates_rows_with_one_step = CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT + CRATES_KNOWN_HEAD_STEP_ROWS;
    let mut stop_rows_checked = 0u64;
    let mut stop_rows_mismatches = 0u64;
    let mut stop_steps_checked = 0u64;
    let mut stop_steps_mismatches = 0u64;
    for evaluated in evaluated_points.iter().filter(|evaluated| evaluated.point.geometry == Geometry::ImplementationTwoDevices && evaluated.point.unknown_writable_head_count == 0) {
        for cell in evaluated.cells.iter().filter(|cell| cell.writable_generations > 0) {
            if evaluated.point.tree_dimension_statistic_count == 1 {
                stop_rows_checked += 1;
                stop_rows_mismatches += u64::from(cell.rows_written_this_generation != CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT || cell.rows_written_all_rows != crates_rows_with_one_step);
            }
            stop_steps_checked += 1;
            stop_steps_mismatches += u64::from(cell.trajectory_crates_step_reading_differences.is_none_or(|trajectory| trajectory.peak > 0));
        }
    }
    writer.line(&format!(
        "name=verdict check=stop_clause_rows_match_crates checked={stop_rows_checked} mismatches={stop_rows_mismatches} w1_expected={} w2_expected={crates_rows_with_one_step} holds={}",
        CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT,
        render_verdict_holds(stop_rows_mismatches)
    ));
    writer.line(&format!(
        "name=verdict check=stop_clause_step_reading_matches_crates checked={stop_steps_checked} mismatches={stop_steps_mismatches} holds={}",
        render_verdict_holds(stop_steps_mismatches)
    ));

    // 第八节判别力自证：弄坏之后还绿的格数（not_turning_red）不为 0，那一位在那一对上没有判别力（第十节）。
    let mut flags_question_checked = 0u64;
    let mut flags_question_not_turning_red = 0u64;
    let mut accounting_question_position_six_checked = 0u64;
    let mut accounting_question_position_six_not_turning_red = 0u64;
    let mut accounting_question_position_seven_checked = 0u64;
    let mut accounting_question_dangling_carried_rows_checked = 0u64;
    let mut accounting_question_dangling_carried_rows_not_turning_red = 0u64;
    let mut accounting_question_position_seven_not_turning_red = 0u64;
    for evaluated in &evaluated_points {
        let point = &evaluated.point;
        if point.invisible_window_generations == 0 {
            for accounting_policy in ACCOUNTING_ARMS {
                flags_question_checked += 1;
                flags_question_not_turning_red += u64::from(!flags_question_self_proof_turns_red(evaluated, accounting_policy));
            }
        }
        let acting_flags = FLAGS_ARMS.iter().copied().filter(|flags_policy| accounting_arm_acts_under(*flags_policy, point));
        if point.tree_dimension_statistic_count == 1 && point.unknown_writable_head_count > 0 {
            for flags_policy in acting_flags {
                if point.unknown_writable_head_count == 2 && !point.initial_publish_changed_the_watermark {
                    accounting_question_position_six_checked += 1;
                    accounting_question_position_six_not_turning_red += u64::from(!accounting_question_position_six_self_proof_turns_red(evaluated, flags_policy));
                }
                if point.initial_publish_changed_the_watermark {
                    accounting_question_position_seven_checked += 1;
                    accounting_question_position_seven_not_turning_red += u64::from(!accounting_question_position_seven_self_proof_turns_red(evaluated, flags_policy));
                }
                if is_a_point_of_the_dangling_carried_rows_self_proof(point) {
                    accounting_question_dangling_carried_rows_checked += 1;
                    accounting_question_dangling_carried_rows_not_turning_red += u64::from(!accounting_question_dangling_carried_rows_self_proof_turns_red(evaluated, flags_policy));
                }
            }
        }
    }
    let mut self_proofs = vec![
        ("self_proof_q2_reject_entry_versus_skip_position_six_where_w_gens_0", flags_question_checked, flags_question_not_turning_red),
        ("self_proof_q1_carry_versus_keep_with_root_position_six_where_h_new_2_lpc_0_r_tree_dim_1", accounting_question_position_six_checked, accounting_question_position_six_not_turning_red),
        ("self_proof_q1_carry_versus_keep_with_root_position_seven_where_h_new_positive_lpc_1_r_tree_dim_1", accounting_question_position_seven_checked, accounting_question_position_seven_not_turning_red),
    ];
    if judgement == Judgement::DanglingCarriedRowsClause {
        self_proofs.push(("self_proof_q1_carry_versus_no_carry_position_six_where_h_new_2_lpc_1_r_tree_dim_1", accounting_question_dangling_carried_rows_checked, accounting_question_dangling_carried_rows_not_turning_red));
    }
    for (name, checked, not_turning_red) in self_proofs {
        writer.line(&format!("name=verdict check={name} checked={checked} not_turning_red={not_turning_red} turns_red={}", render_verdict_holds(not_turning_red)));
    }
    writer.line(&format!(
        "name=verdict check=non_distinguishing_points points_all_fifteen_cells_equal_q2_counted={non_distinguishing_points_counted} points_all_fifteen_cells_equal_q2_not_counted={non_distinguishing_points_not_counted}"
    ));
    writer.line(&format!("name=verdict check=q1_independent_of_q2_reading points_where_q1_differs={points_where_the_accounting_winners_depend_on_the_reject_entry_reading} holds={}", render_verdict_holds(points_where_the_accounting_winners_depend_on_the_reject_entry_reading)));

    // Q1 判定：一个分组、一条 flags 臂下，各点的记账胜者是同一个集合 ⇒ 记这个集合；随点变 ⇒ 依赖取样点（翻转点在 summary 行里按集合列）。
    let mut winner_sets: BTreeMap<(&'static str, String, String, String), Vec<(String, u64)>> = BTreeMap::new();
    for ((kind, reading_label, group, arm, value), description) in &summaries {
        if *kind == "q1_winners" || *kind == "q1_overall_winner_accounting_arms" || *kind == "q2_flags_winners" || *kind == "q2_overall_winner_flags_arms" {
            winner_sets.entry((*kind, reading_label.clone(), group.clone(), arm.clone())).or_default().push((value.clone(), description.point_count));
        }
    }
    for ((kind, reading_label, group, arm), sets) in &winner_sets {
        let judgement = if sets.len() == 1 { "same_set_everywhere" } else { "depends_on_point" };
        let rendered_sets = sets.iter().map(|(set, count)| format!("{set}:{count}")).collect::<Vec<String>>().join("|");
        writer.line(&format!("name=verdict check={kind} reading={reading_label} group={group} arm={arm} distinct_winner_sets={} judgement={judgement} winner_sets={rendered_sets}", sets.len()));
    }
    for (group, tally) in &flags_question_tallies {
        writer.line(&format!(
            "name=verdict check=q2_reject_entry_versus_skip group={group} points={} separable_points_q2_counted={} separable_points_q2_not_counted={} \
             flags_winners_changed_points_b1_w2={} flags_winners_changed_points_b2_w2={} judgement_b1_w2={}",
            tally.points,
            tally.separable_points_counted,
            tally.separable_points_not_counted,
            tally.changed_points_judged,
            tally.changed_points_incidental_counting_the_judged_items,
            if tally.changed_points_judged > 0 { "changes_tier" } else { "same_tier" }
        ));
    }

    for pair in sensitivity_pairs() {
        let mut pair_count = 0u64;
        let mut flipped = PointSetDescription::default();
        for (index, evaluated) in evaluated_points.iter().enumerate() {
            let Some(partner) = partner_of(&pair, &evaluated.point) else {
                continue;
            };
            let partner_analyses = &analyses[partner_index(&partner)];
            let own_analyses = &analyses[index];
            pair_count += 1;
            let differs = match pair.verdict {
                SensitivityVerdict::AccountingWinners => accounting_winners_signature(&own_analyses.0) != accounting_winners_signature(&partner_analyses.0),
                SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip => {
                    let signature = |analyses: &(PointAnalysis, PointAnalysis)| {
                        (
                            flags_winners_signature(&analyses.0),
                            flags_winners_signature(&analyses.1),
                            analyses.0.flags_winners_by_accounting_arm.iter().map(|arm| (arm.position_six_reject_entry_versus_skip, arm.separable_at_position_six)).collect::<Vec<_>>(),
                            analyses.1.flags_winners_by_accounting_arm.iter().map(|arm| (arm.position_six_reject_entry_versus_skip, arm.separable_at_position_six)).collect::<Vec<_>>(),
                        )
                    };
                    signature(own_analyses) != signature(partner_analyses)
                }
            };
            if differs {
                flipped.add(&evaluated.point);
            }
        }
        let verdict_label = match pair.verdict {
            SensitivityVerdict::AccountingWinners => "q1_accounting_winners",
            SensitivityVerdict::FlagsWinnersAndRejectEntryVersusSkip => "q2_flags_winners_and_reject_entry_versus_skip",
        };
        let judgement = if flipped.point_count > 0 { "depends_on_point" } else { "two_points_same" };
        writer.line(&format!(
            "name=verdict check=sensitivity pair={} verdict={verdict_label} pairs={pair_count} judgement={judgement} {}",
            pair.name,
            flipped.render("flipped_")
        ));
    }
    if judgement == Judgement::DanglingCarriedRowsClause {
        // 第二次跑第六节「加一个量」：判 ㊀ 只看 `legal_pooled` 那一行，两个 `r_tree_dim` 分组照报；E113 口径两组单报、标不判。
        let tallies_by_group = acceptance_tallies(&evaluated_points);
        for (group, judged) in ACCEPTANCE_GROUPS {
            writer.line(&render_acceptance_line(judgement, group, judged, &tallies_by_group[group]));
        }
        let acceptance_count_mismatches: u64 = EXPECTED_ACCEPTANCE_POINT_COUNTS
            .iter()
            .map(|(group, judged_points, excluded_points)| {
                let tally = &tallies_by_group[group];
                u64::from(tally.judged_points != *judged_points || tally.excluded_points_without_unknown_writable_heads != *excluded_points || tally.form == AcceptanceForm::NoJudgedPoints)
            })
            .sum();
        writer.line(&format!(
            "name=verdict check=acceptance_point_counts_and_form_present checked={} mismatches={acceptance_count_mismatches} holds={}",
            EXPECTED_ACCEPTANCE_POINT_COUNTS.len(),
            render_verdict_holds(acceptance_count_mismatches)
        ));

        // 停机条款第 6 项：两种判据逐（点, 格）比，U5 以外一格都不许不同；Q2 判定一点都不许不同。
        let mut cells_compared = 0u64;
        let mut cell_mismatches = 0u64;
        let mut flags_question_mismatches = 0u64;
        for evaluated in &evaluated_points {
            let vacuous = evaluate_point(evaluated.point, Judgement::VacuousCarriedRowsClause);
            for (dangling_cell, vacuous_cell) in evaluated.cells.iter().zip(&vacuous.cells) {
                cells_compared += 1;
                cell_mismatches += u64::from(!cells_differ_only_in_the_carried_rows_clause(vacuous_cell, dangling_cell, &evaluated.point));
            }
            flags_question_mismatches += u64::from(!flags_question_judgements_agree(&vacuous, evaluated));
        }
        writer.line(&format!(
            "name=verdict check=judgements_differ_only_in_u5 cells={cells_compared} mismatches={cell_mismatches} holds={}",
            render_verdict_holds(cell_mismatches)
        ));
        writer.line(&format!(
            "name=verdict check=judgements_q2_identical points={point_count} mismatches={flags_question_mismatches} holds={}",
            render_verdict_holds(flags_question_mismatches)
        ));

        // 停机条款第 7 项：新 U5 照 N 判与照 `h_new > 0` 判在每个点上一致。
        let object_mismatches: u64 = evaluated_points.iter().map(|evaluated| u64::from(!carried_rows_object_count_matches_the_unknown_writable_heads(&evaluated.point))).sum();
        writer.line(&format!(
            "name=verdict check=u5_object_count_matches_h_new points={point_count} mismatches={object_mismatches} holds={}",
            render_verdict_holds(object_mismatches)
        ));
    }
    writer.finish();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point_at(geometry: Geometry, unknown_writable_head_count: u64, earlier_step_count: u64, initial_publish_changed_the_watermark: bool, tree_dimension_statistic_count: u64) -> SamplePoint {
        SamplePoint {
            geometry,
            retired_kept_generation_count: 4,
            invisible_window_generations: 3,
            unknown_writable_head_count,
            trees_with_registered_bit: 0,
            published_generations: 10,
            earlier_step_count,
            initial_publish_changed_the_watermark,
            tree_dimension_statistic_count,
        }
    }

    /// 第一次跑的单测钉的是不带参数时的判据（`DEFAULT_JUDGEMENT`）：下面三个助手不写判据时都取它，
    /// 第二次跑的单测用 `_under` 结尾的写法点名判据。
    fn cell_at(point: SamplePoint, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> CellOutcome {
        cell_at_under(point, flags_policy, accounting_policy, DEFAULT_JUDGEMENT)
    }

    fn cell_at_under(point: SamplePoint, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, judgement: Judgement) -> CellOutcome {
        evaluate_cell(&point, &watermark_change_history_of(&point), flags_policy, accounting_policy, judgement)
    }

    fn evaluate_point(point: SamplePoint) -> EvaluatedPoint {
        super::evaluate_point(point, DEFAULT_JUDGEMENT)
    }

    fn all_evaluated_points() -> &'static [EvaluatedPoint] {
        all_evaluated_points_under(DEFAULT_JUDGEMENT)
    }

    /// 全部 11520 点按给定判据算一遍，同一个测试进程里每种判据只算一次（并行的几个测试共用）。
    fn all_evaluated_points_under(judgement: Judgement) -> &'static [EvaluatedPoint] {
        static VACUOUS: std::sync::OnceLock<Vec<EvaluatedPoint>> = std::sync::OnceLock::new();
        static DANGLING: std::sync::OnceLock<Vec<EvaluatedPoint>> = std::sync::OnceLock::new();
        let slot = match judgement {
            Judgement::VacuousCarriedRowsClause => &VACUOUS,
            Judgement::DanglingCarriedRowsClause => &DANGLING,
        };
        slot.get_or_init(|| all_sample_points().into_iter().map(|point| super::evaluate_point(point, judgement)).collect())
    }

    #[test]
    fn sample_points_number_11520_and_every_point_ranks_all_fifteen_cells() {
        let points = all_sample_points();
        assert_eq!(u64::try_from(points.len()).expect("计数装得进 u64"), EXPECTED_POINT_COUNT);
        assert_eq!(EXPECTED_POINT_COUNT, 5 * 4 * 3 * 2 * 4 * 6 * 2 * 2);
        // 只读挂载那几格（`ReadOnly`、未登记时的 `PerBit`）可写代数为 0，照样进比较。
        let evaluated = evaluate_point(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1));
        assert_eq!(u64::try_from(evaluated.cells.len()).expect("计数装得进 u64"), CELLS_PER_POINT);
        assert_eq!(evaluated.outcome_under_the_arms(UnknownFlagsPolicy::ReadOnly, AccountingPolicy::Carry).writable_generations, 0);
    }

    #[test]
    fn ignore_violates_read_only_once_per_generation_and_revives_the_half_deleted_tree() {
        for (published_generations, expected_violations) in [(1u64, 1u64), (4, 4), (5, 5), (10, 10)] {
            let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1);
            point.published_generations = published_generations;
            for accounting_policy in ACCOUNTING_ARMS {
                let cell = cell_at(point, UnknownFlagsPolicy::Ignore, accounting_policy);
                assert_eq!(cell.read_only_violations, expected_violations, "G = {published_generations}");
                assert_eq!(cell.revived_dead_trees, 1);
            }
        }
    }

    #[test]
    fn skip_wrongly_frees_sixty_four_blocks_per_invisible_generation_and_nothing_without_a_window() {
        for (invisible_window_generations, expected_blocks) in [(0u64, 0u64), (1, 64), (3, 192), (8, 512)] {
            let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1);
            point.invisible_window_generations = invisible_window_generations;
            assert_eq!(cell_at(point, UnknownFlagsPolicy::Skip, AccountingPolicy::Carry).blocks_freed_while_referenced, expected_blocks);
            assert_eq!(cell_at(point, UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry).blocks_freed_while_referenced, 0);
        }
    }

    #[test]
    fn reject_entry_counted_semantics_adds_exactly_one_clause_unit_and_skip_gets_none() {
        let evaluated = evaluate_point(point_at(Geometry::ImplementationTwoDevices, 2, 1, false, 1));
        for accounting_policy in ACCOUNTING_ARMS {
            let counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED);
            let not_counted = evaluated.vector(UnknownFlagsPolicy::RejectEntry, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
            assert_eq!(counted[5], not_counted[5] + 1);
            assert_eq!(
                evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED),
                evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED)
            );
        }
        assert_eq!(evaluated.vector(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5], 1);
    }

    #[test]
    fn read_only_never_publishes_and_loses_nothing() {
        for accounting_policy in ACCOUNTING_ARMS {
            let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 0, false, 2), UnknownFlagsPolicy::ReadOnly, accounting_policy);
            assert_eq!(cell.writable_generations, 0);
            assert_eq!(cell.rows_lost, 0);
            assert_eq!(cell.rows_written_all_rows, 0);
            assert_eq!(cell.clause_units_excluding_the_judged_items, 0);
            assert_eq!(cell.clause_units_counting_the_judged_items, 1, "U13 按臂判：只读挂载不是「该条目 EIO」");
        }
    }

    #[test]
    fn per_bit_equals_read_only_when_unregistered_and_skip_when_all_registered() {
        for (trees_with_registered_bit, reduces_to) in [(0u64, UnknownFlagsPolicy::ReadOnly), (UNKNOWN_TREE_COUNT, UnknownFlagsPolicy::Skip)] {
            let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 2);
            point.trees_with_registered_bit = trees_with_registered_bit;
            let evaluated = evaluate_point(point);
            for accounting_policy in ACCOUNTING_ARMS {
                for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
                    assert_eq!(evaluated.vector(UnknownFlagsPolicy::PerBit, accounting_policy, reading), evaluated.vector(reduces_to, accounting_policy, reading));
                }
            }
        }
    }

    #[test]
    fn no_carry_loses_the_second_statistic_tuple_at_the_first_publish_whatever_the_kept_generation_count_says() {
        for retired_kept_generation_count in [1u64, 20] {
            for published_generations in [1u64, 10] {
                let mut point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2);
                point.retired_kept_generation_count = retired_kept_generation_count;
                point.published_generations = published_generations;
                let cell = cell_at(point, UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry);
                assert_eq!(cell.rows_lost, 2, "k_keep = {retired_kept_generation_count}, G = {published_generations}：两个不认识的头各丢第二个统计量那一元组");
                assert_eq!(cell.silent_harm_count, 1, "第二个统计量的元组丢失没有不变量盖");
            }
        }
    }

    #[test]
    fn no_carry_keeps_the_watermark_tuple_through_its_step_rows() {
        let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry);
        assert_eq!(cell.rows_lost, 0);
        assert_eq!(cell.clause_units_excluding_the_judged_items, 0);
        assert_eq!(cell.rows_dropped_over_all_generations, 2, "行数口径：R 那一行（不是台阶）在 R_1 里没了，两个头各一行");
    }

    #[test]
    fn keep_with_root_breaks_six_clauses_when_the_initial_row_is_not_a_step_and_none_when_it_is() {
        let not_a_step = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
        assert_eq!(not_a_step.clause_units_excluding_the_judged_items, 6, "U1 U2 U3 U4 U6 U7");
        let is_a_step = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
        assert_eq!(is_a_step.clause_units_excluding_the_judged_items, 0);
        let second_statistic = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
        assert_eq!(second_statistic.clause_units_excluding_the_judged_items, 4, "U1 U3 U6 U7");
        assert_eq!(not_a_step.trajectory_unknown_rows_at_the_initial_txg, Some(Trajectory { peak: 2, positive_versions: 10, final_value: 2 }));
    }

    #[test]
    fn carry_breaks_no_clause_and_keeps_every_step_unchanged() {
        for (earlier_step_count, initial_publish_changed_the_watermark) in STEP_HISTORY_VALUES {
            for statistic_count in [1u64, 2] {
                let cell = cell_at(point_at(Geometry::ImplementationTwoDevices, 2, earlier_step_count, initial_publish_changed_the_watermark, statistic_count), UnknownFlagsPolicy::Skip, AccountingPolicy::Carry);
                assert_eq!(cell.clause_units_excluding_the_judged_items, 0, "s = {earlier_step_count}, lpc = {initial_publish_changed_the_watermark}, r = {statistic_count}");
                assert_eq!(cell.rows_lost, 0);
            }
        }
    }

    #[test]
    fn no_carry_without_any_step_loses_the_unknown_heads_watermark_rows_under_the_watermark_row_invariant() {
        let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 0, false, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry);
        assert_eq!(cell.clause_units_excluding_the_judged_items, 1, "U8");
        assert_eq!(cell.rows_lost, 2);
        assert_eq!(cell.silent_harm_count, 0, "水位元组丢失有 I-9.11 盖");
    }

    #[test]
    fn rows_written_match_the_registered_closed_forms() {
        let expectations: [(Geometry, u64, u64, bool, u64, [u64; 3], [u64; 3]); 3] = [
            (Geometry::UnknownTreeFullArms, 2, 1, true, 1, [23, 21, 21], [19, 17, 17]),
            (Geometry::ImplementationTwoDevices, 4, 3, false, 2, [37, 29, 37], [24, 16, 16]),
            (Geometry::ImplementationTwoDevices, 2, 0, false, 1, [18, 16, 18], [17, 15, 15]),
        ];
        for (geometry, head_count, earlier_step_count, initial_publish_changed_the_watermark, statistic_count, all_rows, this_generation_rows) in expectations {
            let point = point_at(geometry, head_count, earlier_step_count, initial_publish_changed_the_watermark, statistic_count);
            for ((accounting_policy, expected_all_rows), expected_this_generation_rows) in ACCOUNTING_ARMS.into_iter().zip(all_rows).zip(this_generation_rows) {
                let cell = cell_at(point, UnknownFlagsPolicy::Skip, accounting_policy);
                assert_eq!(cell.rows_written_all_rows, expected_all_rows, "{point:?} {accounting_policy:?} W2");
                assert_eq!(cell.rows_written_this_generation, expected_this_generation_rows, "{point:?} {accounting_policy:?} W1");
            }
        }
        // 甲类：实现两盘几何、没有不认识的头时，W1 = 15（D5 已定项 8 的 15 行），W2 再加认识的头那一级台阶。
        let implementation = cell_at(point_at(Geometry::ImplementationTwoDevices, 0, 1, true, 1), UnknownFlagsPolicy::Skip, AccountingPolicy::Carry);
        assert_eq!(implementation.rows_written_this_generation, CRATES_POOL_WIDE_ROWS + CRATES_PER_DEVICE_ROWS * CRATES_DEVICE_COUNT);
        assert_eq!(implementation.rows_written_this_generation, 15);
        assert_eq!(implementation.rows_written_all_rows, 15 + CRATES_KNOWN_HEAD_STEP_ROWS);
    }

    #[test]
    fn this_generation_rows_ranking_puts_carry_ahead_of_keep_with_root_on_clause_units_despite_more_rows() {
        let evaluated = evaluate_point(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1));
        let this_generation_rows_reading = INCIDENTAL_READINGS[2];
        assert_eq!(this_generation_rows_reading.rows_written, RowsWrittenReading::ThisGenerationRowsOnly);
        let carry = evaluated.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, this_generation_rows_reading);
        let keep_with_root = evaluated.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot, this_generation_rows_reading);
        assert_eq!(carry, [192, 0, 0, 0, 0, 0, 19]);
        assert_eq!(keep_with_root, [192, 0, 0, 0, 0, 6, 17]);
        assert!(carry < keep_with_root);
    }

    #[test]
    fn all_rows_ranking_puts_carry_ahead_of_keep_with_root_when_the_second_statistic_is_not_a_step() {
        let evaluated = evaluate_point(point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2));
        let carry = evaluated.vector(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
        let keep_with_root = evaluated.vector(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::KeepWithRoot, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED);
        // 自己那些行 11 + 6 × 2 = 23；Carry 每头 s + lpc + r = 4 行，随根留着每头 s + r = 3 行。
        assert_eq!(carry, [0, 0, 0, 0, 0, 0, 23 + 2 * 4]);
        assert_eq!(keep_with_root, [0, 0, 0, 0, 0, 4, 23 + 2 * 3]);
        assert!(carry < keep_with_root);
    }

    #[test]
    fn crates_step_reading_matches_the_change_history_on_a_single_head_history() {
        let head = TreeOwner::KnownHead(0);
        let watermark_row = |generation_txg: u64, value: u64| AccountingRow { statistic: Statistic::InodeNumberWatermark, tree: head, generation_txg, value };
        let history = WatermarkChangeHistory { changed_at: [(head, 990), (head, 995), (head, 1001)].into_iter().collect() };
        let changed_this_publish = AccountingVersion { txg: 1001, rows: vec![watermark_row(990, 5), watermark_row(995, 6), watermark_row(1001, 7)], rows_written_for_unknown_trees: Vec::new() };
        assert_eq!(steps_by_the_crates_reading(&changed_this_publish), steps_by_the_watermark_change_history(&changed_this_publish, &history));
        assert_eq!(steps_by_the_crates_reading(&changed_this_publish).len(), 3);
        let unchanged_this_publish = AccountingVersion { txg: 1002, rows: vec![watermark_row(990, 5), watermark_row(995, 6), watermark_row(1001, 7), watermark_row(1002, 7)], rows_written_for_unknown_trees: Vec::new() };
        assert_eq!(steps_by_the_crates_reading(&unchanged_this_publish), steps_by_the_watermark_change_history(&unchanged_this_publish, &history));
        assert_eq!(steps_by_the_crates_reading(&unchanged_this_publish).len(), 3);
    }

    #[test]
    fn checker_syntax_counts_the_second_statistic_but_skips_the_kept_watermark_row() {
        let cell = cell_at(point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 2), UnknownFlagsPolicy::Skip, AccountingPolicy::KeepWithRoot);
        assert_eq!(cell.trajectory_row_generation_invariant_syntactic_violations, Some(Trajectory { peak: 2, positive_versions: 10, final_value: 2 }));
    }

    #[test]
    fn positive_and_negative_controls_hold_on_every_point() {
        let evaluated_points = all_evaluated_points();
        for control in POSITIVE_CONTROLS {
            let (checked, failures) = evaluated_points.iter().map(|evaluated| positive_control_failures(evaluated, control)).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
            assert!(checked > 0, "{control:?} 一格都没查");
            assert_eq!(failures, 0, "{control:?}");
        }
        for negative_control in [negative_control_no_window_failures, negative_control_no_unknown_heads_failures] {
            let (checked, failures) = evaluated_points.iter().map(negative_control).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
            assert!(checked > 0);
            assert_eq!(failures, 0);
        }
    }

    #[test]
    fn device_predicates_and_the_hand_ledger_agree_on_every_cell_under_both_readings() {
        for judgement in [Judgement::VacuousCarriedRowsClause, Judgement::DanglingCarriedRowsClause] {
            let mut compared = 0u64;
            for evaluated in all_evaluated_points_under(judgement) {
                for cell in &evaluated.cells {
                    assert_eq!(cell.clause_units_excluding_the_judged_items, clause_units_from_the_hand_ledger(&evaluated.point, cell.flags_policy, cell.accounting_policy, ClauseReading::ExcludesTheJudgedItems, judgement), "{judgement:?} {:?} {cell:?}", evaluated.point);
                    assert_eq!(cell.clause_units_counting_the_judged_items, clause_units_from_the_hand_ledger(&evaluated.point, cell.flags_policy, cell.accounting_policy, ClauseReading::CountsTheJudgedItems, judgement), "{judgement:?} {:?} {cell:?}", evaluated.point);
                    compared += 1;
                }
            }
            assert_eq!(compared, EXPECTED_POINT_COUNT * CELLS_PER_POINT);
        }
    }

    #[test]
    fn registered_anchors_hold_on_every_selected_point() {
        for (name, checked, mismatches) in anchor_failures(&all_evaluated_points()) {
            assert!(checked > 0, "{name} 一格都没查");
            assert_eq!(mismatches, 0, "{name}");
        }
    }

    #[test]
    fn kept_generation_count_is_not_read_by_the_model() {
        for point in all_sample_points().into_iter().filter(|point| point.retired_kept_generation_count == 20 && point.invisible_window_generations == 3) {
            let with_largest = evaluate_point(point);
            let with_smallest = evaluate_point(point_with(&point, PointChange::RetiredKeptGenerationCount(1)));
            for cell in &with_largest.cells {
                for reading in [JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED, JUDGED_READING_REJECT_ENTRY_SEMANTICS_NOT_COUNTED] {
                    assert_eq!(vector_under(cell, &point, reading), with_smallest.vector(cell.flags_policy, cell.accounting_policy, reading), "{point:?}");
                }
            }
        }
    }

    #[test]
    fn self_proofs_turn_red_when_the_discriminating_position_is_broken() {
        let mut no_window = point_at(Geometry::ImplementationTwoDevices, 2, 1, false, 1);
        no_window.invisible_window_generations = 0;
        let evaluated = evaluate_point(no_window);
        for accounting_policy in ACCOUNTING_ARMS {
            assert!(flags_question_self_proof_turns_red(&evaluated, accounting_policy));
        }
        assert!(accounting_question_position_six_self_proof_turns_red(&evaluated, UnknownFlagsPolicy::Skip));
        let initial_is_a_step = evaluate_point(point_at(Geometry::ImplementationTwoDevices, 2, 1, true, 1));
        assert!(accounting_question_position_seven_self_proof_turns_red(&initial_is_a_step, UnknownFlagsPolicy::RejectEntry));
        // 记账臂没有作用点的格（只读挂载）七位全平，弄坏也不变：自证在那里不成立，只在记账臂起作用的格上查。
        assert!(!accounting_question_position_six_self_proof_turns_red(&evaluated, UnknownFlagsPolicy::ReadOnly));
    }

    #[test]
    fn first_separating_position_and_winners_follow_the_lexicographic_order() {
        let vectors = [[0, 0, 0, 0, 0, 6, 17], [0, 0, 0, 0, 0, 0, 19], [0, 0, 0, 0, 0, 0, 17]];
        assert_eq!(first_separating_position(&vectors), Some(6));
        assert_eq!(winner_indices(&vectors), vec![2]);
        assert_eq!(first_separating_position(&[[1, 2, 3, 4, 5, 6, 7], [1, 2, 3, 4, 5, 6, 7]]), None);
        assert_eq!(winner_indices(&[[1, 2, 3, 4, 5, 6, 7], [1, 2, 3, 4, 5, 6, 7]]), vec![0, 1]);
    }

    // ---- 第二次跑（`dangling-u5`）的单测：登记第二次跑第七、八、九、十一节 ----

    fn carried_rows_clause_false_units(cell: &CellOutcome) -> u64 {
        u64::from(carried_rows_clause_false_trajectory_of(cell).is_some_and(|trajectory| trajectory.peak > 0))
    }

    fn arguments_of(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    #[test]
    fn the_default_judgement_is_the_first_runs_and_the_parameter_picks_either() {
        assert_eq!(DEFAULT_JUDGEMENT, Judgement::VacuousCarriedRowsClause);
        assert_eq!(Judgement::from_arguments(&[]), Ok(Judgement::VacuousCarriedRowsClause));
        assert_eq!(Judgement::from_arguments(&arguments_of(&["--judgement", "vacuous-u5"])), Ok(Judgement::VacuousCarriedRowsClause));
        assert_eq!(Judgement::from_arguments(&arguments_of(&["--judgement", "dangling-u5"])), Ok(Judgement::DanglingCarriedRowsClause));
        assert!(Judgement::from_arguments(&arguments_of(&["--judgement", "other"])).is_err());
        assert!(Judgement::from_arguments(&arguments_of(&["--judgement"])).is_err());
        assert!(Judgement::from_arguments(&arguments_of(&["dangling-u5"])).is_err());
        assert_eq!(Judgement::VacuousCarriedRowsClause.label(), "vacuous-u5");
        assert_eq!(Judgement::DanglingCarriedRowsClause.label(), "dangling-u5");
    }

    #[test]
    fn dangling_carried_rows_clause_on_the_registered_point_counts_zero_one_one_across_the_accounting_arms() {
        let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1);
        let never_false = Trajectory { peak: 0, positive_versions: 0, final_value: 0 };
        let false_in_every_version = Trajectory { peak: 1, positive_versions: 10, final_value: 1 };
        // (记账臂, 新 U5 记几, 新 U5 为假的轨迹, 新判据下 B1 第 6 位)：登记第七节「单测的期望」第一张表。
        let expectations = [
            (AccountingPolicy::Carry, 0u64, never_false, 0u64),
            (AccountingPolicy::NoCarry, 1, false_in_every_version, 1),
            (AccountingPolicy::KeepWithRoot, 1, false_in_every_version, 7),
        ];
        let dangling = super::evaluate_point(point, Judgement::DanglingCarriedRowsClause);
        for (accounting_policy, expected_carried_rows_clause_units, expected_trajectory, expected_position_six) in expectations {
            let cell = cell_at_under(point, UnknownFlagsPolicy::Skip, accounting_policy, Judgement::DanglingCarriedRowsClause);
            assert_eq!(carried_rows_clause_false_units(&cell), expected_carried_rows_clause_units, "{accounting_policy:?}");
            assert_eq!(carried_rows_clause_false_trajectory_of(&cell), Some(expected_trajectory), "{accounting_policy:?}");
            assert_eq!(dangling.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5], expected_position_six, "{accounting_policy:?}");
            // 第一次跑的判据下三条臂的 U5 都是 0；新判据比它多的正是 U5（写加法）。
            let vacuous_cell = cell_at_under(point, UnknownFlagsPolicy::Skip, accounting_policy, Judgement::VacuousCarriedRowsClause);
            assert_eq!(carried_rows_clause_false_units(&vacuous_cell), 0, "{accounting_policy:?}");
            assert_eq!(cell.clause_units_excluding_the_judged_items, vacuous_cell.clause_units_excluding_the_judged_items + expected_carried_rows_clause_units, "{accounting_policy:?}");
        }
    }

    #[test]
    fn dangling_carried_rows_clause_counts_nothing_where_the_accounting_arm_does_not_act_or_there_is_no_unknown_head() {
        let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1);
        for flags_policy in [UnknownFlagsPolicy::Ignore, UnknownFlagsPolicy::ReadOnly] {
            for accounting_policy in ACCOUNTING_ARMS {
                let cell = cell_at_under(point, flags_policy, accounting_policy, Judgement::DanglingCarriedRowsClause);
                assert_eq!(carried_rows_clause_false_units(&cell), 0, "{flags_policy:?} {accounting_policy:?}");
            }
        }
        let without_unknown_head = point_at(Geometry::UnknownTreeFullArms, 0, 1, false, 1);
        for accounting_policy in ACCOUNTING_ARMS {
            let cell = cell_at_under(without_unknown_head, UnknownFlagsPolicy::Skip, accounting_policy, Judgement::DanglingCarriedRowsClause);
            assert_eq!(carried_rows_clause_false_units(&cell), 0, "{accounting_policy:?}");
            assert_eq!(carried_rows_clause_false_trajectory_of(&cell), Some(Trajectory { peak: 0, positive_versions: 0, final_value: 0 }), "{accounting_policy:?}");
        }
    }

    #[test]
    fn dangling_carried_rows_clause_ledger_columns_follow_the_registered_second_run_table() {
        // (几何点: h_new, s, lpc, r_tree_dim)，三条记账臂的 B1 第 6 位（Skip 下）：登记第七节「另三格钉账本别的列」。
        let columns = [((2u64, 1u64, true, 1u64), [0u64, 1, 1]), ((2, 0, false, 1), [0, 2, 7]), ((2, 1, true, 2), [0, 1, 5])];
        for ((head_count, earlier_step_count, lpc, statistic_count), expected) in columns {
            let evaluated = super::evaluate_point(point_at(Geometry::UnknownTreeFullArms, head_count, earlier_step_count, lpc, statistic_count), Judgement::DanglingCarriedRowsClause);
            for (accounting_policy, expected_position_six) in ACCOUNTING_ARMS.into_iter().zip(expected) {
                assert_eq!(evaluated.vector(UnknownFlagsPolicy::Skip, accounting_policy, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5], expected_position_six, "{head_count} {earlier_step_count} {lpc} {statistic_count} {accounting_policy:?}");
            }
        }
    }

    #[test]
    fn dangling_carried_rows_clause_self_proof_turns_red_where_the_vacuous_reading_cannot_tell_carry_from_no_carry() {
        let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 1);
        assert!(is_a_point_of_the_dangling_carried_rows_self_proof(&point));
        let dangling = super::evaluate_point(point, Judgement::DanglingCarriedRowsClause);
        for flags_policy in [UnknownFlagsPolicy::Skip, UnknownFlagsPolicy::RejectEntry] {
            assert!(accounting_question_dangling_carried_rows_self_proof_turns_red(&dangling, flags_policy), "{flags_policy:?}");
        }
        // 记账臂没有作用点的格（只读挂载）七位全平，弄坏也不变：自证只在记账臂起作用的格上查。
        assert!(!accounting_question_dangling_carried_rows_self_proof_turns_red(&dangling, UnknownFlagsPolicy::ReadOnly));
        // 第一次跑的判据下 Carry 与 NoCarry 在这一点第 6 位相等（第一次跑判据错在这里），这条自证在那里本来就红不起来。
        let vacuous = super::evaluate_point(point, Judgement::VacuousCarriedRowsClause);
        assert_eq!(
            vacuous.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5],
            vacuous.vector(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, JUDGED_READING_REJECT_ENTRY_SEMANTICS_COUNTED)[5]
        );
        assert!(!accounting_question_dangling_carried_rows_self_proof_turns_red(&vacuous, UnknownFlagsPolicy::Skip));
        assert!(!is_a_point_of_the_dangling_carried_rows_self_proof(&point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1)));
        assert!(!is_a_point_of_the_dangling_carried_rows_self_proof(&point_at(Geometry::UnknownTreeFullArms, 2, 1, true, 2)));
    }

    #[test]
    fn positive_and_negative_controls_hold_on_every_point_under_the_dangling_carried_rows_clause() {
        let evaluated_points = all_evaluated_points_under(Judgement::DanglingCarriedRowsClause);
        let controls = positive_controls_of(Judgement::DanglingCarriedRowsClause);
        assert_eq!(controls.len(), POSITIVE_CONTROLS.len() + 3);
        assert_eq!(positive_controls_of(Judgement::VacuousCarriedRowsClause), POSITIVE_CONTROLS.to_vec());
        for control in controls {
            let (checked, failures) = evaluated_points.iter().map(|evaluated| positive_control_failures(evaluated, control)).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
            assert!(checked > 0, "{control:?} 一格都没查");
            assert_eq!(failures, 0, "{control:?}");
        }
        for negative_control in [negative_control_no_window_failures, negative_control_no_unknown_heads_failures] {
            let (checked, failures) = evaluated_points.iter().map(negative_control).fold((0, 0), |sum, part| (sum.0 + part.0, sum.1 + part.1));
            assert!(checked > 0);
            assert_eq!(failures, 0);
        }
    }

    #[test]
    fn the_new_carried_rows_clause_controls_fail_on_the_first_runs_cells() {
        // 判别力：新 U5 的阳性对照在第一次跑的判据算出的格上必须红，否则这三条对照分不出两种判据。
        let vacuous_points = all_evaluated_points_under(Judgement::VacuousCarriedRowsClause);
        for control in DANGLING_CARRIED_ROWS_POSITIVE_CONTROLS {
            let failures: u64 = vacuous_points.iter().map(|evaluated| positive_control_failures(evaluated, control).1).sum();
            let expected_to_fail = control != PositiveControl::DanglingCarriedRowsCarryHolds;
            assert_eq!(failures > 0, expected_to_fail, "{control:?}");
        }
    }

    fn pt(has_unknown_writable_heads: bool, winners: &[AccountingPolicy]) -> (bool, Vec<AccountingPolicy>) {
        (has_unknown_writable_heads, winners.to_vec())
    }

    #[test]
    fn every_winner_form_gets_its_registered_consequence() {
        use AccountingPolicy::{Carry, KeepWithRoot, NoCarry};
        let form_and_consequence = |points: &[(bool, Vec<AccountingPolicy>)]| {
            let tally = carry_acceptance_of(points);
            (tally.form, tally.consequence)
        };
        assert_eq!(form_and_consequence(&[pt(true, &[Carry]), pt(true, &[Carry])]), (AcceptanceForm::UniqueCarry, Consequence::CarryStands));
        assert_eq!(form_and_consequence(&[pt(true, &[NoCarry]), pt(true, &[NoCarry])]), (AcceptanceForm::UniqueOtherArm, Consequence::FlipsTo(NoCarry)));
        assert_eq!(form_and_consequence(&[pt(true, &[KeepWithRoot])]), (AcceptanceForm::UniqueOtherArm, Consequence::FlipsTo(KeepWithRoot)));
        assert_eq!(form_and_consequence(&[pt(true, &[Carry, NoCarry]), pt(true, &[Carry, NoCarry])]), (AcceptanceForm::TieWithCarry, Consequence::CarryStandsTieKnownToUser));
        assert_eq!(form_and_consequence(&[pt(true, &[NoCarry, KeepWithRoot]), pt(true, &[NoCarry, KeepWithRoot])]), (AcceptanceForm::TieWithoutCarry, Consequence::FlipChoiceToUser));
        assert_eq!(form_and_consequence(&[pt(true, &[Carry]), pt(true, &[Carry, NoCarry])]), (AcceptanceForm::DependsOnPointCarryEverywhere, Consequence::CarryStands));
        assert_eq!(form_and_consequence(&[pt(true, &[NoCarry]), pt(true, &[NoCarry, KeepWithRoot])]), (AcceptanceForm::DependsOnPointCarryNowhere, Consequence::FlipChoiceToUser));
        assert_eq!(form_and_consequence(&[pt(true, &[Carry]), pt(true, &[NoCarry])]), (AcceptanceForm::DependsOnPointMixed, Consequence::LeftToUser));
        assert_eq!(form_and_consequence(&[]), (AcceptanceForm::NoJudgedPoints, Consequence::NoJudgedPoints));
        // 全是 h_new = 0 的点：没有被判点，同样读不到。
        assert_eq!(form_and_consequence(&[pt(false, &[Carry, NoCarry, KeepWithRoot])]), (AcceptanceForm::NoJudgedPoints, Consequence::NoJudgedPoints));
    }

    #[test]
    fn points_with_no_unknown_writable_head_are_reported_but_not_judged() {
        use AccountingPolicy::{Carry, KeepWithRoot, NoCarry};
        let tally = carry_acceptance_of(&[pt(true, &[Carry]), pt(false, &[Carry, NoCarry, KeepWithRoot]), pt(true, &[Carry]), pt(false, &[Carry, NoCarry, KeepWithRoot])]);
        assert_eq!((tally.judged_points, tally.excluded_points_without_unknown_writable_heads), (2, 2));
        assert_eq!((tally.form, tally.consequence), (AcceptanceForm::UniqueCarry, Consequence::CarryStands));
        assert_eq!(tally.winner_sets.len(), 1);
        assert_eq!(render_acceptance_line(Judgement::DanglingCarriedRowsClause, "legal_pooled", true, &tally),
            "name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_pooled judged=yes judged_points=2 excluded_points_h_new_0=2 form=unique_carry consequence=carry_stands winner_sets=Carry:2");
    }

    #[test]
    fn judged_and_excluded_point_counts_match_the_registered_anchors() {
        let tallies = acceptance_tallies(all_evaluated_points_under(Judgement::DanglingCarriedRowsClause));
        for (group, judged_points, excluded_points) in EXPECTED_ACCEPTANCE_POINT_COUNTS {
            assert_eq!((tallies[group].judged_points, tallies[group].excluded_points_without_unknown_writable_heads), (judged_points, excluded_points), "{group}");
        }
        assert_eq!(EXPECTED_ACCEPTANCE_POINT_COUNTS.map(|(_, judged_points, _)| judged_points), [3200, 3200, 6400]);
        // 命令独立数出来的 E113 口径点数 1920 与全部点数（登记第七节乙类）。
        let e113_style = all_sample_points().iter().filter(|point| !point.is_legal_under_todays_step_rows()).count();
        assert_eq!(e113_style, 1920);
        assert_eq!(tallies["e113_style_r_tree_dim_1"].judged_points + tallies["e113_style_r_tree_dim_1"].excluded_points_without_unknown_writable_heads, 960);
        assert_eq!(tallies["e113_style_r_tree_dim_2"].judged_points + tallies["e113_style_r_tree_dim_2"].excluded_points_without_unknown_writable_heads, 960);
    }

    #[test]
    fn the_two_judgements_differ_only_in_the_carried_rows_clause_on_every_point_and_cell() {
        let vacuous = all_evaluated_points_under(Judgement::VacuousCarriedRowsClause);
        let dangling = all_evaluated_points_under(Judgement::DanglingCarriedRowsClause);
        assert_eq!(vacuous.len(), dangling.len());
        let mut cells_compared = 0u64;
        let mut cells_where_the_carried_rows_clause_differs = 0u64;
        for (before, after) in vacuous.iter().zip(dangling) {
            assert_eq!(before.point, after.point);
            for (vacuous_cell, dangling_cell) in before.cells.iter().zip(&after.cells) {
                cells_compared += 1;
                cells_where_the_carried_rows_clause_differs += u64::from(carried_rows_clause_false_units(vacuous_cell) != carried_rows_clause_false_units(dangling_cell));
                assert!(cells_differ_only_in_the_carried_rows_clause(vacuous_cell, dangling_cell, &after.point), "{:?} {:?} {:?}", after.point, dangling_cell.flags_policy, dangling_cell.accounting_policy);
            }
            assert!(flags_question_judgements_agree(before, after), "{:?}", after.point);
        }
        assert_eq!(cells_compared, EXPECTED_POINT_COUNT * CELLS_PER_POINT);
        assert!(cells_where_the_carried_rows_clause_differs > 0, "两种判据真的不同，这条对拍才有分辨力");
    }

    #[test]
    fn the_judgement_comparison_turns_red_when_something_other_than_the_carried_rows_clause_moves() {
        let point = point_at(Geometry::UnknownTreeFullArms, 2, 1, false, 1);
        let vacuous = cell_at_under(point, UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, Judgement::VacuousCarriedRowsClause);
        let dangling = cell_at_under(point, UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, Judgement::DanglingCarriedRowsClause);
        assert!(cells_differ_only_in_the_carried_rows_clause(&vacuous, &dangling, &point));
        let mut wrongly_freed = dangling.clone();
        wrongly_freed.blocks_freed_while_referenced += 1;
        assert!(!cells_differ_only_in_the_carried_rows_clause(&vacuous, &wrongly_freed, &point));
        let mut extra_clause_unit = dangling.clone();
        extra_clause_unit.clause_units_excluding_the_judged_items += 1;
        assert!(!cells_differ_only_in_the_carried_rows_clause(&vacuous, &extra_clause_unit, &point));
        let mut other_clause_moved = dangling.clone();
        for (clause, trajectory) in &mut other_clause_moved.clause_false_trajectories {
            if *clause == ClauseUnit::DecisionSixteenItemNineRewriteEachPublish {
                *trajectory = Some(Trajectory { peak: 1, positive_versions: 1, final_value: 1 });
            }
        }
        assert!(!cells_differ_only_in_the_carried_rows_clause(&vacuous, &other_clause_moved, &point));
    }

    #[test]
    fn carried_rows_object_count_equals_unknown_writable_heads_positive_on_every_point() {
        let mut checked = 0u64;
        for point in all_sample_points() {
            assert!(carried_rows_object_count_matches_the_unknown_writable_heads(&point), "{point:?}");
            checked += 1;
        }
        assert_eq!(checked, EXPECTED_POINT_COUNT);
    }
}
