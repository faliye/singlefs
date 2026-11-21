//! 判据的两种取法、登记的各个计数与取值表、默认判据。

use super::*;

/// U5 的判据（登记第二次跑第三节「装置写在哪」）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Judgement {
    /// 第一次跑的判据：写者这次为不认识的树写进去的行，代段 = 这次的 txg；没写就空真。
    VacuousCarriedRowsClause,
    /// 第二次跑的判据：那半句所说的行在这条臂下不存在 ⇒ 要改；`Carry` 下照原谓词判。
    DanglingCarriedRowsClause,
}

/// 不带参数时的判据：第一次跑的那一种，产物逐字节等于第一次跑的产物（登记第二次跑第十一节停机条款第 5 项）。
pub(super) const DEFAULT_JUDGEMENT: Judgement = Judgement::VacuousCarriedRowsClause;

impl Judgement {
    pub(super) fn label(self) -> &'static str {
        match self {
            Judgement::VacuousCarriedRowsClause => "vacuous-u5",
            Judgement::DanglingCarriedRowsClause => "dangling-u5",
        }
    }

    /// 命令行参数（不含程序名）：空 ⇒ 默认；`--judgement <值>` ⇒ 那一种；别的一律拒。
    pub(super) fn from_arguments(arguments: &[String]) -> Result<Judgement, String> {
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
pub(super) const UNKNOWN_TREE_COUNT: u64 = 4;
/// 其中那个未知位取「只读」语义的。
pub(super) const UNKNOWN_READ_ONLY_TREE_COUNT: u64 = 1;
/// 其中那个未知位取「半删」语义的。
pub(super) const UNKNOWN_HALF_DELETED_TREE_COUNT: u64 = 1;
/// 每代出生多少块 —— 误释放数的量纲（E113 原值）。
pub(super) const BLOCKS_BORN_PER_GENERATION: u64 = 64;
/// v1 认识的树（两种几何同为 12，只进配置行）。
pub(super) const KNOWN_TREE_COUNT: u64 = 12;

/// E113 几何：不带树维的行 11 行（E113 的 `r_pool`）、认识的可写头 6 个、认识的头没有台阶行（E113 口径）。
pub(super) const E113_ROWS_WITHOUT_TREE_DIMENSION: u64 = 11;
pub(super) const E113_KNOWN_WRITABLE_HEAD_COUNT: u64 = 6;
pub(super) const E113_KNOWN_HEAD_STEP_ROWS: u64 = 0;

/// 实现两盘几何，值抄自 `crates/singlefs-core/src/transaction.rs` 的 `accounting_entries_of_this_publish`：
/// 池级三行（含 inode 号水位那一行）加每块盘六行，两块盘；只有 inode 树一个可写头，它的水位有一级台阶行。
pub(super) const CRATES_POOL_WIDE_ROWS: u64 = 3;
pub(super) const CRATES_PER_DEVICE_ROWS: u64 = 6;
pub(super) const CRATES_DEVICE_COUNT: u64 = 2;
pub(super) const CRATES_KNOWN_WRITABLE_HEAD_COUNT: u64 = 1;
pub(super) const CRATES_KNOWN_HEAD_STEP_ROWS: u64 = 1;
/// 池级三行里带树维的只有 inode 号水位那一行，它在模型里按「认识的可写头 × 带树维的统计量」另算。
pub(super) const CRATES_POOL_WIDE_ROWS_WITH_TREE_DIMENSION: u64 = 1;

/// R 那一版（认识全部树的新写者发布）的 txg；旧写者此后发布的第 g 版 txg 是它加 g（D16 已定项 6）。
pub(super) const INITIAL_VERSION_TXG: u64 = 1000;
/// 不认识的头更早的台阶行之间相隔几个 txg（只要互不相同、都早于 R 就行）。
pub(super) const STEP_TXG_SPACING: u64 = 10;
/// 实现两盘几何里认识的那个头那一级台阶行的代段。
pub(super) const KNOWN_HEAD_STEP_TXG: u64 = INITIAL_VERSION_TXG - 5;
pub(super) const KNOWN_HEAD_WATERMARK: u64 = 500;
pub(super) const UNKNOWN_HEAD_FIRST_STEP_WATERMARK: u64 = 100;
/// 不带树维的统计量与第二个带树维的统计量的值：对搬运者不透明，取什么都不影响判定。
pub(super) const OPAQUE_STATISTIC_VALUE: u64 = 7;

/// Q2 读「计」时 `RejectEntry` 每一格多记的条数（要新定义的「EIO 但位置还在」语义）。
pub(super) const REJECT_ENTRY_NEW_SEMANTICS_CLAUSE_UNITS: u64 = 1;
/// 第 4 位「静默」的口径（E113 那一位加第二个统计量那一项）：误释放有 I-3.1 盖，水位元组丢失有 I-9.11 盖，
/// 违反只读、复活半删、第二个统计量的元组丢失没有不变量盖。
pub(super) const WRONG_FREE_HAS_AN_INVARIANT: bool = true;
pub(super) const WATERMARK_TUPLE_LOSS_HAS_AN_INVARIANT: bool = true;
pub(super) const SECOND_TREE_STATISTIC_LOSS_HAS_AN_INVARIANT: bool = false;

/// 登记第一节第 8 条的取样维度。`k_keep` 今天没有对象（K 代已撤回），模型不读它，只为覆盖 E113 的点。
pub(super) const RETIRED_KEPT_GENERATION_COUNT_VALUES: [u64; 5] = [1, 2, 4, 8, 20];
pub(super) const INVISIBLE_WINDOW_GENERATION_VALUES: [u64; 4] = [0, 1, 3, 8];
pub(super) const UNKNOWN_WRITABLE_HEAD_COUNT_VALUES: [u64; 3] = [0, 2, 4];
pub(super) const REGISTERED_TREE_COUNT_VALUES: [u64; 2] = [0, UNKNOWN_TREE_COUNT];
pub(super) const PUBLISHED_GENERATION_VALUES: [u64; 4] = [1, 4, 5, 10];
/// (`s` 更早的台阶行数, `lpc` R 那次发布本身改没改这个头的水位)。
pub(super) const STEP_HISTORY_VALUES: [(u64, bool); 6] = [(0, false), (0, true), (1, false), (1, true), (3, false), (3, true)];
pub(super) const TREE_DIMENSION_STATISTIC_COUNT_VALUES: [u64; 2] = [1, 2];
pub(super) const GEOMETRIES: [Geometry; 2] = [Geometry::UnknownTreeFullArms, Geometry::ImplementationTwoDevices];
pub(super) const EXPECTED_POINT_COUNT: u64 = 11520;
pub(super) const CELLS_PER_POINT: u64 = 15;
