//! 记账行与版本、水位变化史、每代写行的构造与推进、发布历史。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Statistic {
    /// 不带树维的统计量（含按盘展开的那几行），下标只为区分元组。
    WithoutTreeDimension(u64),
    /// D5 已定项 4 第 12 项，今天唯一带树维的统计量。
    InodeNumberWatermark,
    /// `r_tree_dim = 2` 时假设的第二个带树维的统计量：只有这一代的行、没有台阶行。
    SecondTreeDimension,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum TreeOwner {
    PoolWide,
    KnownHead(u64),
    UnknownHead(u64),
}

/// 字段次序就是 key 次序：(统计量, 树, 代段)，再按值。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) struct AccountingRow {
    pub(super) statistic: Statistic,
    pub(super) tree: TreeOwner,
    pub(super) generation_txg: u64,
    pub(super) value: u64,
}

impl AccountingRow {
    pub(super) fn tuple(&self) -> (Statistic, TreeOwner) {
        (self.statistic, self.tree)
    }

    pub(super) fn belongs_to_an_unknown_tree(&self) -> bool {
        matches!(self.tree, TreeOwner::UnknownHead(_))
    }

    fn recoded_to(self, generation_txg: u64) -> AccountingRow {
        AccountingRow { statistic: self.statistic, tree: self.tree, generation_txg, value: self.value }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct AccountingVersion {
    pub(super) txg: u64,
    /// 按 key 升序。
    pub(super) rows: Vec<AccountingRow>,
    /// 写者这一次为不认识的树新写进去的行（照抄的台阶行与原样留着的行不算），U5 判它。
    pub(super) rows_written_for_unknown_trees: Vec<AccountingRow>,
}

/// D5 意义的台阶行：按 (统计量, 树) 分，那次发布那个头的水位变了才留。模型知道每个头水位在哪几次发布变过，
/// 台阶行由此认，不看代段的语法（`crates/` 今天的语法认法另写在 `steps_by_the_crates_reading`，只作附带量）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct WatermarkChangeHistory {
    pub(super) changed_at: BTreeSet<(TreeOwner, u64)>,
}

impl WatermarkChangeHistory {
    pub(super) fn is_a_step(&self, row: &AccountingRow) -> bool {
        row.statistic == Statistic::InodeNumberWatermark && self.changed_at.contains(&(row.tree, row.generation_txg))
    }
}

pub(super) fn known_writable_heads(point: &SamplePoint) -> Vec<TreeOwner> {
    (0..point.geometry.known_writable_head_count()).map(TreeOwner::KnownHead).collect()
}

pub(super) fn unknown_writable_heads(point: &SamplePoint) -> Vec<TreeOwner> {
    (0..point.unknown_writable_head_count).map(TreeOwner::UnknownHead).collect()
}

fn tree_dimension_statistics(point: &SamplePoint) -> Vec<Statistic> {
    match point.tree_dimension_statistic_count {
        1 => vec![Statistic::InodeNumberWatermark],
        2 => vec![Statistic::InodeNumberWatermark, Statistic::SecondTreeDimension],
        other => panic!("r_tree_dim 只取 1 或 2（登记第一节第 8 条），这里是 {other}"),
    }
}

/// 不认识的头更早的第 `step_index` 级台阶（从 0 起，越往后越新）的代段。
fn unknown_head_step_txg(earlier_step_count: u64, step_index: u64) -> u64 {
    INITIAL_VERSION_TXG - STEP_TXG_SPACING * (earlier_step_count - step_index)
}

pub(super) fn watermark_change_history_of(point: &SamplePoint) -> WatermarkChangeHistory {
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
pub(super) fn initial_version(point: &SamplePoint) -> AccountingVersion {
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
                Statistic::SecondTreeDimension | Statistic::WithoutTreeDimension(_) => OPAQUE_STATISTIC_VALUE,
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
                Statistic::SecondTreeDimension | Statistic::WithoutTreeDimension(_) => OPAQUE_STATISTIC_VALUE,
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

pub(super) fn effective_flags_policy_of(flags_policy: UnknownFlagsPolicy, point: &SamplePoint) -> UnknownFlagsPolicy {
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
pub(super) struct PublishedHistory {
    pub(super) initial: AccountingVersion,
    pub(super) versions: Vec<AccountingVersion>,
    pub(super) writable_generations: u64,
    pub(super) effective_flags_policy: UnknownFlagsPolicy,
}

pub(super) fn published_history_of(point: &SamplePoint, history: &WatermarkChangeHistory, flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy) -> PublishedHistory {
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
