//! E113：不认识的树·补齐臂集 —— E112 的重做，臂集与选法都改了。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D18（块里携带什么信息）：「偏移 7 是 8 个 flag 位，第一版恒 0，**读者遇到非 0 位一律拒收**
//!   ……本工程**不给**这 8 位留「忽略陌生位」的通道」⇒ `RejectEntry` 臂的出处。
//! - D8（核心索引结构）：「flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ **该记录 EIO**
//!   （对象级，不拒整个容器）」，落成 I-9.7（记录三段恒零）。
//! - D15（格式冻结政策）：「新增记录类型（新 key 类型）| 只读遍历可能误判；整理 / 写可能误删看不懂的记录
//!   | **compat_ro** | 是 | **「只读安全」与「可写安全」是两条独立保证**，必须分开判」⇒ `ReadOnly` 臂的出处。
//! - D5（快照 / 空间记账机制）：「销毁快照时把它的 deadlist **合并到下一个更新的那一侧**，
//!   并按 `birth > prev(S).txg` 过滤出可释放的条目」⇒ `blocks_freed_while_referenced` 的机制推导。
//! - D5（快照 / 空间记账机制）已定项 2：「只保留最近 K 代」；**K 的值至今未定**
//!   （E54 逐字「真正要定的只有 K」）⇒ K 必须扫，不许钉死。
//! - D5（快照 / 空间记账机制）已定项 4 第 12 项：inode 号水位，带树维「**带**（只为可写头维护）」，
//!   **今天只有这一项带树维**。
//!
//! ## 选法（跑前写死，字典序）
//!
//! 1 不许丢数据 → 2 不许违反语义 → 3 不许丢账 → 4 静默最少 → 5 可写代数最多 → 6 写行数最少。
//! **不设 `mount_ok` 筛子**：只读挂载的代价由第 5 关定价，不是把它请出场。

use e7_index_bench::Emitter;

const KNOWN_TREE_COUNT: u64 = 12;
const KNOWN_WRITABLE_HEAD_COUNT: u64 = 6;
/// v1 不认识的树（新种类，或已知种类但置了 v1 不认识的 flags 位）。
const UNKNOWN_TREE_COUNT: u64 = 4;
/// 其中是可写头的 —— 只有这些才有带树维的记账行。
const UNKNOWN_WRITABLE_HEAD_COUNT: u64 = 2;
/// 其中那个未知位取「只读」语义的（现役先例：btrfs 的 `BTRFS_ROOT_SUBVOL_RDONLY`）。
const UNKNOWN_READ_ONLY_TREE_COUNT: u64 = 1;
/// 其中那个未知位取「半删」语义的（现役先例：`BTRFS_ROOT_SUBVOL_DEAD`）。
const UNKNOWN_HALF_DELETED_TREE_COUNT: u64 = 1;
const TREE_DIMENSION_ROWS_PER_HEAD: u64 = 1;
const POOL_LEVEL_ROWS: u64 = 11;
const PUBLISHED_GENERATIONS: u64 = 10;
/// 每代出生多少块 —— 误释放数的量纲。
const BLOCKS_BORN_PER_GENERATION: u64 = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnknownFlagsPolicy {
    Ignore,
    Skip,
    RejectEntry,
    ReadOnly,
    PerBit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AccountingPolicy {
    Carry,
    NoCarry,
    Exempt,
}

#[derive(Clone, Copy, Debug)]
struct ModelParameters {
    /// 只保留最近几代（D5 已定项 2，值未定 ⇒ 扫）。
    retained_generations: u64,
    /// 隐形窗口跨几代（0 = 没有隐形窗口，阴性对照）。
    invisible_window_generations: u64,
    /// 不认识的树里有几棵是可写头。
    unknown_writable_head_count: u64,
    /// 不认识的位里有几棵树的位是**已登记为 compat** 的（`PerBit` 才用得上）。
    trees_with_registered_bit: u64,
}

impl ModelParameters {
    fn base() -> Self {
        ModelParameters { retained_generations: 4, invisible_window_generations: 3, unknown_writable_head_count: UNKNOWN_WRITABLE_HEAD_COUNT, trees_with_registered_bit: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Outcome {
    writable_generations: u64,
    blocks_freed_while_referenced: u64,
    read_only_violations: u64,
    revived_dead_trees: u64,
    rows_lost: u64,
    rows_written_per_generation: u64,
    /// 出了错而**不变量清单里没有任何一条**盖得住它。
    silent_harm_count: u64,
}

/// `PerBit` 是个条件政策：只要还有一棵树的位没登记，整次挂载就退到只读；
/// 全部登记过（登记时已逐项证明写侧安全）才退化成跳过。
fn per_bit_policy_reduces_to(parameters: ModelParameters) -> UnknownFlagsPolicy {
    if parameters.trees_with_registered_bit >= UNKNOWN_TREE_COUNT {
        UnknownFlagsPolicy::Skip
    } else {
        UnknownFlagsPolicy::ReadOnly
    }
}

/// 误释放的机制推导：一棵对写者隐形的快照使「下一个更新的那一侧」认成更新的那个
/// ⇒ 过滤用的 `prev` 更大 ⇒ 隐形窗口里出生的块全部通过「可直接释放」。
fn blocks_wrongly_freed(invisible_window_generations: u64) -> u64 {
    BLOCKS_BORN_PER_GENERATION * invisible_window_generations
}

fn accounting_rows(flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, parameters: ModelParameters, writable_generations: u64) -> (u64, u64) {
    // 一代都没发布 ⇒ 代号不前进 ⇒ 既不写也不丢。
    if writable_generations == 0 {
        return (0, 0);
    }
    let own_rows = POOL_LEVEL_ROWS + KNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD;
    let unknown_tree_rows = parameters.unknown_writable_head_count * TREE_DIMENSION_ROWS_PER_HEAD;
    // 自以为认识 ⇒ 那些行本来就在它的维护范围里，搬运政策没有作用点。
    if flags_policy == UnknownFlagsPolicy::Ignore {
        return (0, own_rows + unknown_tree_rows);
    }
    match accounting_policy {
        AccountingPolicy::Carry => (0, own_rows + unknown_tree_rows),
        // 豁免点删：不写，也不丢。
        AccountingPolicy::Exempt => (0, own_rows),
        AccountingPolicy::NoCarry => {
            let lost_rows = if writable_generations > parameters.retained_generations { unknown_tree_rows } else { 0 };
            (lost_rows, own_rows)
        }
    }
}

fn run(flags_policy: UnknownFlagsPolicy, accounting_policy: AccountingPolicy, parameters: ModelParameters) -> Outcome {
    let effective_flags_policy = if flags_policy == UnknownFlagsPolicy::PerBit { per_bit_policy_reduces_to(parameters) } else { flags_policy };
    let writable_generations = if effective_flags_policy == UnknownFlagsPolicy::ReadOnly { 0 } else { PUBLISHED_GENERATIONS };

    // 只有「树对写者隐形」才会认错 deadlist 的邻居。
    // RejectEntry 下条目在、只是不可用 ⇒ 写者知道那儿有一棵树 ⇒ 不做跨它的合并。
    let blocks_freed_while_referenced = if effective_flags_policy == UnknownFlagsPolicy::Skip && writable_generations > 0 {
        blocks_wrongly_freed(parameters.invisible_window_generations)
    } else {
        0
    };

    // 只有「自以为认识」才会把只读树当可写、把半删树当活的。
    let (read_only_violations, revived_dead_trees) = if effective_flags_policy == UnknownFlagsPolicy::Ignore {
        (UNKNOWN_READ_ONLY_TREE_COUNT * writable_generations, UNKNOWN_HALF_DELETED_TREE_COUNT)
    } else {
        (0, 0)
    };

    let (rows_lost, rows_written_per_generation) = accounting_rows(effective_flags_policy, accounting_policy, parameters, writable_generations);

    // 谁盖得住：丢账有 I-9.11（可写克隆头有水位行）；误释放有 I-3.1（已分配统计对得上），
    // 但只在 checker 不跟着跳过时成立（C158）；违反只读 / 复活半删树**一条都没有**。
    let silent_if_uncovered = |harm: u64, has_invariant: bool| -> u64 {
        u64::from(harm > 0 && !has_invariant)
    };
    let silent_harm_count = silent_if_uncovered(blocks_freed_while_referenced, true) + silent_if_uncovered(read_only_violations, false) + silent_if_uncovered(revived_dead_trees, false)
        + silent_if_uncovered(rows_lost, true);

    Outcome {
        writable_generations,
        blocks_freed_while_referenced,
        read_only_violations,
        revived_dead_trees,
        rows_lost,
        rows_written_per_generation,
        silent_harm_count,
    }
}

/// 跑前写死的字典序。返回值越小越好，逐位比较。
fn rank_key(outcome: &Outcome) -> (u64, u64, u64, u64, u64, u64) {
    (
        outcome.blocks_freed_while_referenced,             // 1 不许丢数据
        outcome.read_only_violations + outcome.revived_dead_trees,        // 2 不许违反语义
        outcome.rows_lost,                                 // 3 不许丢账
        outcome.silent_harm_count,                                    // 4 静默最少
        PUBLISHED_GENERATIONS - outcome.writable_generations,                    // 5 可写代数最多
        outcome.rows_written_per_generation,                      // 6 写行数最少
    )
}

const FLAGS_ARMS: [UnknownFlagsPolicy; 5] = [
    UnknownFlagsPolicy::Ignore,
    UnknownFlagsPolicy::Skip,
    UnknownFlagsPolicy::RejectEntry,
    UnknownFlagsPolicy::ReadOnly,
    UnknownFlagsPolicy::PerBit,
];
const ACCOUNTING_ARMS: [AccountingPolicy; 3] = [AccountingPolicy::Carry, AccountingPolicy::NoCarry, AccountingPolicy::Exempt];

fn winners(parameters: ModelParameters) -> Vec<String> {
    let mut best: Option<(u64, u64, u64, u64, u64, u64)> = None;
    let mut winning_arms: Vec<String> = Vec::new();
    for flags_policy in FLAGS_ARMS {
        for accounting_policy in ACCOUNTING_ARMS {
            let candidate_key = rank_key(&run(flags_policy, accounting_policy, parameters));
            match best {
                None => {
                    best = Some(candidate_key);
                    winning_arms = vec![format!("{flags_policy:?}_{accounting_policy:?}")];
                }
                Some(best_key) if candidate_key < best_key => {
                    best = Some(candidate_key);
                    winning_arms = vec![format!("{flags_policy:?}_{accounting_policy:?}")];
                }
                Some(best_key) if candidate_key == best_key => winning_arms.push(format!("{flags_policy:?}_{accounting_policy:?}")),
                _ => {}
            }
        }
    }
    winning_arms.sort();
    winning_arms
}

fn main() {
    let mut emitter = Emitter::new();
    let base_parameters = ModelParameters::base();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config t_known={KNOWN_TREE_COUNT} h_known={KNOWN_WRITABLE_HEAD_COUNT} t_new={UNKNOWN_TREE_COUNT} h_new={UNKNOWN_WRITABLE_HEAD_COUNT} \
             t_new_rdonly={UNKNOWN_READ_ONLY_TREE_COUNT} t_new_dead={UNKNOWN_HALF_DELETED_TREE_COUNT} r_tree_dim={TREE_DIMENSION_ROWS_PER_HEAD} \
             r_pool={POOL_LEVEL_ROWS} g_gens={PUBLISHED_GENERATIONS} blocks_per_gen={BLOCKS_BORN_PER_GENERATION} \
             k_keep={} w_gens={} model=arithmetic file_ops=0",
            base_parameters.retained_generations, base_parameters.invisible_window_generations
        ))
    );

    for flags_policy in FLAGS_ARMS {
        for accounting_policy in ACCOUNTING_ARMS {
            let outcome = run(flags_policy, accounting_policy, base_parameters);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=cell arm={flags_policy:?}_{accounting_policy:?} writable_gens={} blocks_freed_while_referenced={} \
                     rdonly_violations={} dead_revived={} rows_lost={} rows_written_per_gen={} silent={}",
                    outcome.writable_generations,
                    outcome.blocks_freed_while_referenced,
                    outcome.read_only_violations,
                    outcome.revived_dead_trees,
                    outcome.rows_lost,
                    outcome.rows_written_per_generation,
                    outcome.silent_harm_count
                ))
            );
        }
    }

    println!(
        "{}",
        emitter.emit_raw(&format!("name=verdict_base winners={}", winners(base_parameters).join("+")))
    );

    // 参数扫：跑前写死要报「排名翻不翻」。
    let base_winners = winners(base_parameters);
    let mut flips = 0u64;
    for retained_generations in [1u64, 2, 4, 8, 20] {
        for invisible_window_generations in [0u64, 1, 3, 8] {
            for unknown_writable_head_count in [0u64, 2, 4] {
                for registered_trees in [0u64, UNKNOWN_TREE_COUNT] {
                    let sweep_parameters = ModelParameters { retained_generations, invisible_window_generations, unknown_writable_head_count, trees_with_registered_bit: registered_trees };
                    let sweep_winners = winners(sweep_parameters);
                    let flipped = sweep_winners != base_winners;
                    if flipped {
                        flips += 1;
                    }
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=sweep k_keep={retained_generations} w_gens={invisible_window_generations} h_new={unknown_writable_head_count} registered={registered_trees} \
                             winners={} flipped={}",
                            sweep_winners.join("+"),
                            u8::from(flipped)
                        ))
                    );
                }
            }
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=sweep_summary points={} flipped={flips}",
            5 * 4 * 3 * 2
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：模型常量逐个钉住。
    #[test]
    fn absolute_constants() {
        assert_eq!(TREE_DIMENSION_ROWS_PER_HEAD + POOL_LEVEL_ROWS, 12, "在用十二个统计量");
        assert_eq!(FLAGS_ARMS.len(), 5);
        assert_eq!(ACCOUNTING_ARMS.len(), 3);
        assert_eq!(BLOCKS_BORN_PER_GENERATION, 64);
        assert_eq!(UNKNOWN_READ_ONLY_TREE_COUNT + UNKNOWN_HALF_DELETED_TREE_COUNT + 2, UNKNOWN_TREE_COUNT, "四棵里两棵带语义位");
    }

    /// **阳性对照·Ignore**：必须违反只读语义并复活半删树。
    #[test]
    fn positive_ignore() {
        let outcome = run(UnknownFlagsPolicy::Ignore, AccountingPolicy::Carry, ModelParameters::base());
        assert_eq!(outcome.read_only_violations, UNKNOWN_READ_ONLY_TREE_COUNT * PUBLISHED_GENERATIONS);
        assert_eq!(outcome.read_only_violations, 10);
        assert_eq!(outcome.revived_dead_trees, 1);
        assert_eq!(outcome.blocks_freed_while_referenced, 0, "树可见 ⇒ 邻居认得对");
    }

    /// **阳性对照·Skip**：必须误释放仍被引用的块，且数是闭式算出来的。
    #[test]
    fn positive_skip() {
        let parameters = ModelParameters::base();
        let outcome = run(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, parameters);
        assert!(outcome.blocks_freed_while_referenced > 0, "误释放这一维没进模型");
        assert_eq!(outcome.blocks_freed_while_referenced, BLOCKS_BORN_PER_GENERATION * parameters.invisible_window_generations);
        assert_eq!(outcome.blocks_freed_while_referenced, 192);
    }

    /// **阳性对照·RejectEntry**：不丢数据、不违反语义，但账照样会丢（NoCarry 下）。
    #[test]
    fn positive_reject_entry() {
        let outcome = run(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::NoCarry, ModelParameters::base());
        assert_eq!(outcome.blocks_freed_while_referenced, 0, "条目在 ⇒ 写者知道那儿有树");
        assert_eq!(outcome.read_only_violations + outcome.revived_dead_trees, 0);
        assert!(outcome.rows_lost > 0, "RejectEntry 这一维的代价没进模型");
        assert_eq!(outcome.rows_lost, 2);
    }

    /// **阳性对照·ReadOnly**：一代都发不出去，其余伤害全零。
    #[test]
    fn positive_read_only() {
        for accounting_policy in ACCOUNTING_ARMS {
            let outcome = run(UnknownFlagsPolicy::ReadOnly, accounting_policy, ModelParameters::base());
            assert_eq!(outcome.writable_generations, 0, "ReadOnly 这一维没进模型");
            assert_eq!(outcome.blocks_freed_while_referenced, 0);
            assert_eq!(outcome.rows_lost, 0);
            assert_eq!(outcome.rows_written_per_generation, 0);
        }
    }

    /// **阳性对照·PerBit**：它是条件政策，两档各退化到一条已有的臂。
    #[test]
    fn positive_per_bit_is_conditional() {
        let unregistered_parameters = ModelParameters { trees_with_registered_bit: 0, ..ModelParameters::base() };
        assert_eq!(per_bit_policy_reduces_to(unregistered_parameters), UnknownFlagsPolicy::ReadOnly);
        assert_eq!(run(UnknownFlagsPolicy::PerBit, AccountingPolicy::Carry, unregistered_parameters), run(UnknownFlagsPolicy::ReadOnly, AccountingPolicy::Carry, unregistered_parameters));

        let registered_parameters = ModelParameters { trees_with_registered_bit: UNKNOWN_TREE_COUNT, ..ModelParameters::base() };
        assert_eq!(per_bit_policy_reduces_to(registered_parameters), UnknownFlagsPolicy::Skip);
        assert_eq!(run(UnknownFlagsPolicy::PerBit, AccountingPolicy::Carry, registered_parameters), run(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, registered_parameters));
    }

    /// **阴性对照**：没有隐形窗口就不该有误释放。
    #[test]
    fn negative_no_window_no_loss() {
        let parameters = ModelParameters { invisible_window_generations: 0, ..ModelParameters::base() };
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, parameters).blocks_freed_while_referenced, 0);
    }

    /// **不许用筛子请人出场**：五条 flags 臂都必须出现在排名里。
    #[test]
    fn every_arm_is_ranked() {
        let parameters = ModelParameters::base();
        let mut ranked_cells = 0;
        for flags_policy in FLAGS_ARMS {
            for accounting_policy in ACCOUNTING_ARMS {
                let _ = rank_key(&run(flags_policy, accounting_policy, parameters));
                ranked_cells += 1;
            }
        }
        assert_eq!(ranked_cells, 15, "15 个格子一个都不许被跳过");
    }

    /// **字典序本身**：丢数据必须压过一切，哪怕对方一代都发不出去。
    #[test]
    fn lexicographic_order_puts_data_loss_first() {
        let parameters = ModelParameters::base();
        let skip_rank = rank_key(&run(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, parameters));
        let read_only_rank = rank_key(&run(UnknownFlagsPolicy::ReadOnly, AccountingPolicy::Carry, parameters));
        assert!(read_only_rank < skip_rank, "丢数据要输给「只读但不丢」");
    }

    /// **基准点的赢家**：钉成绝对值，不是「相对更好」。
    /// ⚠️ 跑之前写下的接受条款倾向 `PerBit`，实算赢家是 `RejectEntry_Exempt`
    /// ⇒ 按那条条款改结论。这里连**它凭哪一位赢**一起钉住。
    #[test]
    fn absolute_winner_at_base() {
        let parameters = ModelParameters::base();
        assert_eq!(winners(parameters), vec!["RejectEntry_Exempt".to_string()]);
        // 凭哪一位赢：六位全零到第五位，第六位 17。
        assert_eq!(rank_key(&run(UnknownFlagsPolicy::RejectEntry, AccountingPolicy::Exempt, parameters)), (0, 0, 0, 0, 0, 17));
        // ReadOnly 在第五位（可写代数）输掉 —— 它不丢任何东西，但一代都发不出去。
        assert_eq!(rank_key(&run(UnknownFlagsPolicy::ReadOnly, AccountingPolicy::Exempt, parameters)), (0, 0, 0, 0, 10, 0));
        // Skip 在第一位（丢数据）就输了。
        // ⚠️ 它的第四位（静默）是 0 而不是 1：误释放**按规范**有 I-3.1（已分配统计对得上） 盖着。
        // 但那要 checker 不跟着跳过 —— 跟着跳过的话两边一起少一项照样平，落点 C158。
        // ⇒ 这一位在这里不构成 Skip 的辩护，它已经在第一位输掉了。
        assert_eq!(
            rank_key(&run(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, parameters)),
            (192, 0, 0, 0, 0, 19)
        );
        // Ignore 在第二位（违反语义）输，第一位与赢家平。
        assert_eq!(rank_key(&run(UnknownFlagsPolicy::Ignore, AccountingPolicy::Carry, parameters)), (0, 11, 0, 2, 0, 19));
    }

    /// **记账三臂的写代价**：各钉一个绝对值。
    #[test]
    fn absolute_rows_written() {
        let parameters = ModelParameters::base();
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::Carry, parameters).rows_written_per_generation, 19);
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, parameters).rows_written_per_generation, 17);
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::Exempt, parameters).rows_written_per_generation, 17);
        // 豁免点删与不搬一样便宜，而且不丢账 —— 它的代价在别处（要改 D5 已定项 2）。
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::Exempt, parameters).rows_lost, 0);
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, parameters).rows_lost, 2);
    }

    /// **K 的边界**：K 大到 ≥ 发布代数就不丢账 —— K 未定，这条边界要钉住。
    #[test]
    fn absolute_retained_generations_boundary() {
        let below_boundary = ModelParameters { retained_generations: PUBLISHED_GENERATIONS - 1, ..ModelParameters::base() };
        let at_boundary = ModelParameters { retained_generations: PUBLISHED_GENERATIONS, ..ModelParameters::base() };
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, below_boundary).rows_lost, 2);
        assert_eq!(run(UnknownFlagsPolicy::Skip, AccountingPolicy::NoCarry, at_boundary).rows_lost, 0);
    }
}
