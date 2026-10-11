//! E112：旧写者遇到不认识的树 —— flags 政策与记账搬运政策的交叉。
//!
//! ## 它要答什么
//!
//! D8（核心索引结构）已定项 8 收口清单剩下两条：「旧写者对不认识的记账维度」与
//! 「flags 其余 15 位取哪一侧」。两条不独立：flags 取「跳过该条目」时，v1 **写者**
//! 也跳过那棵树，于是它的记账行没人维护——正好落进第一条那个洞。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md）
//!
//! - D8（核心索引结构）已定项 1：记账条目取**幂等完整值**，不是增量 Δ。
//!   ⇒ 一条行的值对搬运者不透明也搬得动。
//! - D5（快照 / 空间记账机制）已定项 2：「代取 checkpoint 号，并配一条增量丢弃规则，
//!   **只保留最近 K 代**」⇒ 没人重写的行，过 K 代就消失。
//! - D5（快照 / 空间记账机制）已定项 4 第 12 项：「inode 号水位」，带树维 =
//!   「**带**（只为可写头维护）」；**今天只有这一项带树维**（第 8 / 9 项 2026-09-06 撤回）。
//! - I-9.11（可写克隆头有水位行）：「每个可写克隆头的树 ID 在记账里都有『inode 号水位』行」，
//!   状态 **未实现** ⇒ 「按规范会被抓」与「今天会被抓」是两个数。
//! - D15（格式冻结政策）已定项 2：新增 keyspace 对旧读者「**只是看不到新树**」⇒ compat。
//!
//! ## 模型
//!
//! v1 写者挂上来发布 G 代。池里有 v1 认识的树与不认识的树，各自有一部分是可写头。
//! 只有**可写头**才有带树维的记账行（每头 TREE_DIMENSION_ROWS_PER_HEAD 行）；其余统计量是全池的，与树无关。

use e7_index_bench::Emitter;

/// v1 认识的树总数。
const KNOWN_TREE_COUNT: u64 = 12;
/// 其中是可写头的。
const KNOWN_WRITABLE_HEAD_COUNT: u64 = 6;
/// v1 不认识的树总数（新种类，或已知种类但置了 v1 不认识的 flags 位）。
const UNKNOWN_TREE_COUNT: u64 = 4;
/// 其中是可写头的 —— **只有这些才有记账行可丢**。
const UNKNOWN_WRITABLE_HEAD_COUNT: u64 = 2;
/// 带树维的统计量条数：今天恰好 1（D5 已定项 4 第 12 项）。
const TREE_DIMENSION_ROWS_PER_HEAD: u64 = 1;
/// 不带树维的统计量条数：在用十二个减去带树维的那一个。
const POOL_LEVEL_ROWS: u64 = 11;
/// 只保留最近 K 代。
const KEPT_GENERATIONS: u64 = 4;
/// 发布代数，取 > KEPT_GENERATIONS 才看得见丢弃。
const PUBLISHED_GENERATIONS: u64 = 10;
/// 每棵不认识的树里的记录数（误读的量纲）。
const RECORDS_PER_NEW_TREE: u64 = 1000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FlagsPolicy {
    /// 忽略未知位，按已知语义继续解释这条条目。
    Ignore,
    /// 按「不认识这棵树」跳过该条目。
    Skip,
    /// 拒绝挂载。
    Refuse,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CarryPolicy {
    /// 把不认识的记账行原样带到新代。
    Carry,
    /// 不带。
    NoCarry,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Outcome {
    mount_ok: bool,
    misread_records: u64,
    rows_lost: u64,
    rows_written_per_generation: u64,
    /// 按 invariants.md 的规范，这个丢失会不会被某条不变量抓到。
    caught_by_specification: bool,
    /// 今天（按各条不变量的实现状态）会不会被抓到。
    caught_today: bool,
}

impl Outcome {
    /// 「静默」= 出了错而没有任何东西会报警。**逐格报它，这是判据 3。**
    fn silent(&self) -> u64 {
        let wrong = self.misread_records > 0 || self.rows_lost > 0;
        u64::from(wrong && !self.caught_today)
    }
    /// 按规范（不变量都实现之后）还静不静默。
    fn silent_by_specification(&self) -> u64 {
        let wrong = self.misread_records > 0 || self.rows_lost > 0;
        u64::from(wrong && !self.caught_by_specification)
    }
}

/// 一代写多少行：全池统计量 + 每个**自己维护的**可写头一行。
fn rows_per_generation(flags_policy: FlagsPolicy, carry_policy: CarryPolicy) -> u64 {
    match flags_policy {
        // 挂不上，一行都不写。
        FlagsPolicy::Refuse => 0,
        // 自以为认识 ⇒ 连不认识的头也一起维护。
        FlagsPolicy::Ignore => POOL_LEVEL_ROWS + (KNOWN_WRITABLE_HEAD_COUNT + UNKNOWN_WRITABLE_HEAD_COUNT) * TREE_DIMENSION_ROWS_PER_HEAD,
        // 跳过 ⇒ 只维护自己认识的；搬运政策决定要不要把别人的原样带上。
        FlagsPolicy::Skip => match carry_policy {
            CarryPolicy::Carry => POOL_LEVEL_ROWS + (KNOWN_WRITABLE_HEAD_COUNT + UNKNOWN_WRITABLE_HEAD_COUNT) * TREE_DIMENSION_ROWS_PER_HEAD,
            CarryPolicy::NoCarry => POOL_LEVEL_ROWS + KNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD,
        },
    }
}

fn run(flags_policy: FlagsPolicy, carry_policy: CarryPolicy, generation_count: u64) -> Outcome {
    let aged_out = generation_count > KEPT_GENERATIONS;
    match flags_policy {
        FlagsPolicy::Refuse => Outcome {
            mount_ok: false,
            misread_records: 0,
            rows_lost: 0,
            rows_written_per_generation: 0,
            caught_by_specification: false,
            caught_today: false,
        },
        FlagsPolicy::Ignore => Outcome {
            mount_ok: true,
            // 按错编码解释了不认识的树里的每一条记录。
            misread_records: UNKNOWN_TREE_COUNT * RECORDS_PER_NEW_TREE,
            // 它自以为认识 ⇒ 照样维护那些行 ⇒ 一行不丢。
            rows_lost: 0,
            rows_written_per_generation: rows_per_generation(flags_policy, carry_policy),
            // 误读发生在记录编码这一层，树 ID 是对的 ⇒ I-1.3 够不着；
            // 记账行在册且值由错误语义算出 ⇒ I-9.11 判「有行」也过。
            caught_by_specification: false,
            caught_today: false,
        },
        FlagsPolicy::Skip => {
            let lost = match carry_policy {
                CarryPolicy::Carry => 0,
                CarryPolicy::NoCarry => {
                    if aged_out {
                        UNKNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD
                    } else {
                        0
                    }
                }
            };
            Outcome {
                mount_ok: true,
                misread_records: 0,
                rows_lost: lost,
                rows_written_per_generation: rows_per_generation(flags_policy, carry_policy),
                // 丢的是可写头的「inode 号水位」行 ⇒ I-9.11 正对着它。
                caught_by_specification: lost > 0,
                // 而 I-9.11 未实现 ⇒ 今天没有任何东西会红。
                caught_today: false,
            }
        }
    }
}

/// 判据 2 要的那条**独立闭式**：不由模拟计数得来。
fn expected_rows_lost_closed_form(flags_policy: FlagsPolicy, carry_policy: CarryPolicy, generation_count: u64) -> u64 {
    if flags_policy != FlagsPolicy::Skip || carry_policy != CarryPolicy::NoCarry || generation_count <= KEPT_GENERATIONS {
        return 0;
    }
    UNKNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD
}

fn name_of(flags_policy: FlagsPolicy, carry_policy: CarryPolicy) -> String {
    format!("{flags_policy:?}_{carry_policy:?}")
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config t_known={KNOWN_TREE_COUNT} h_known={KNOWN_WRITABLE_HEAD_COUNT} t_new={UNKNOWN_TREE_COUNT} h_new={UNKNOWN_WRITABLE_HEAD_COUNT} \
             r_tree_dim={TREE_DIMENSION_ROWS_PER_HEAD} r_pool={POOL_LEVEL_ROWS} k_keep={KEPT_GENERATIONS} g_gens={PUBLISHED_GENERATIONS} \
             records_per_new_tree={RECORDS_PER_NEW_TREE} model=arithmetic file_ops=0"
        ))
    );

    let mut fewest_silent_outcomes = u64::MAX;
    let mut best: Vec<String> = Vec::new();

    for flags_policy in [FlagsPolicy::Ignore, FlagsPolicy::Skip, FlagsPolicy::Refuse] {
        for carry_policy in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
            let outcome = run(flags_policy, carry_policy, PUBLISHED_GENERATIONS);
            let closed_form_rows_lost = expected_rows_lost_closed_form(flags_policy, carry_policy, PUBLISHED_GENERATIONS);
            let cell_name = name_of(flags_policy, carry_policy);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=cell arm={cell_name} mount_ok={} misread_records={} rows_lost={} \
                     rows_lost_closed_form={} rows_written_per_gen={} \
                     caught_by_spec={} caught_today={} silent_today={} silent_by_spec={}",
                    u8::from(outcome.mount_ok),
                    outcome.misread_records,
                    outcome.rows_lost,
                    closed_form_rows_lost,
                    outcome.rows_written_per_generation,
                    u8::from(outcome.caught_by_specification),
                    u8::from(outcome.caught_today),
                    outcome.silent(),
                    outcome.silent_by_specification(),
                ))
            );
            if outcome.mount_ok {
                match outcome.silent().cmp(&fewest_silent_outcomes) {
                    std::cmp::Ordering::Less => {
                        fewest_silent_outcomes = outcome.silent();
                        best = vec![cell_name];
                    }
                    std::cmp::Ordering::Equal => best.push(cell_name),
                    std::cmp::Ordering::Greater => {}
                }
            }
        }
    }

    // 判据 4：搬运的绝对写代价，按行数报，不报百分比。
    let carry_extra = rows_per_generation(FlagsPolicy::Skip, CarryPolicy::Carry)
        - rows_per_generation(FlagsPolicy::Skip, CarryPolicy::NoCarry);
    let own_rows = POOL_LEVEL_ROWS + KNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=carry_cost extra_rows_per_gen={carry_extra} own_rows_per_gen={own_rows} \
             threshold_rows={} over_threshold={}",
            KNOWN_TREE_COUNT * (TREE_DIMENSION_ROWS_PER_HEAD + POOL_LEVEL_ROWS),
            u8::from(carry_extra > KNOWN_TREE_COUNT * (TREE_DIMENSION_ROWS_PER_HEAD + POOL_LEVEL_ROWS))
        ))
    );

    // 判据 5：Refuse 的代价。
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=refuse_cost unmountable_gens={PUBLISHED_GENERATIONS} compat_with_d15_item2=0"
        ))
    );

    // 结论行：静默结局最少的那些格子（只在挂得上的格子里比）。
    best.sort();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=verdict min_silent_today={fewest_silent_outcomes} winners={}",
            best.join("+")
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：模型的每个输入常量都钉住，改一个就红。
    /// 手算：带树维的统计量今天 1 项（D5 已定项 4 第 12 项），全池 11 项，合计在用十二个。
    #[test]
    fn absolute_model_constants() {
        assert_eq!(TREE_DIMENSION_ROWS_PER_HEAD, 1);
        assert_eq!(POOL_LEVEL_ROWS, 11);
        assert_eq!(TREE_DIMENSION_ROWS_PER_HEAD + POOL_LEVEL_ROWS, 12, "在用十二个统计量");
        assert!(KNOWN_WRITABLE_HEAD_COUNT <= KNOWN_TREE_COUNT);
        assert!(UNKNOWN_WRITABLE_HEAD_COUNT <= UNKNOWN_TREE_COUNT);
        assert!(PUBLISHED_GENERATIONS > KEPT_GENERATIONS, "代数要大过保留代数，否则丢弃看不见");
    }

    /// **阳性对照甲**：Skip + NoCarry 且 G > K，必须丢行。
    #[test]
    fn positive_control_skip_nocarry_loses_rows() {
        let outcome = run(FlagsPolicy::Skip, CarryPolicy::NoCarry, PUBLISHED_GENERATIONS);
        assert!(outcome.rows_lost > 0, "代际丢弃这一维没进模型");
        // 绝对值：写成加法不写减法，让变异活到运行期。
        assert_eq!(outcome.rows_lost, UNKNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD);
        assert_eq!(outcome.rows_lost, 2);
    }

    /// **阳性对照乙**：Ignore 臂下写者自以为认识，行不丢而记录被误读。
    #[test]
    fn positive_control_ignore_misreads_but_keeps_rows() {
        for carry_policy in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
            let outcome = run(FlagsPolicy::Ignore, carry_policy, PUBLISHED_GENERATIONS);
            assert_eq!(outcome.rows_lost, 0, "自以为认识就会照样维护那些行");
            assert!(outcome.misread_records > 0, "「自以为认识」这一维没进模型");
            assert_eq!(outcome.misread_records, UNKNOWN_TREE_COUNT * RECORDS_PER_NEW_TREE);
            assert_eq!(outcome.misread_records, 4000);
        }
    }

    /// **阳性对照丙**：Refuse 臂挂不上，且不产生任何错误。
    #[test]
    fn positive_control_refuse_mounts_nothing() {
        for carry_policy in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
            let outcome = run(FlagsPolicy::Refuse, carry_policy, PUBLISHED_GENERATIONS);
            assert!(!outcome.mount_ok);
            assert_eq!(outcome.rows_lost, 0);
            assert_eq!(outcome.misread_records, 0);
            assert_eq!(outcome.rows_written_per_generation, 0);
        }
    }

    /// **阴性对照**：Skip + Carry 且 G > K，一行不丢。
    #[test]
    fn negative_control_skip_carry_loses_nothing() {
        let outcome = run(FlagsPolicy::Skip, CarryPolicy::Carry, PUBLISHED_GENERATIONS);
        assert_eq!(outcome.rows_lost, 0, "搬运这一维没进模型");
    }

    /// **判据 2**：模拟出的丢失数与独立闭式逐格相等。
    #[test]
    fn closed_form_matches_every_cell() {
        for flags_policy in [FlagsPolicy::Ignore, FlagsPolicy::Skip, FlagsPolicy::Refuse] {
            for carry_policy in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
                for generation_count in [1u64, KEPT_GENERATIONS, KEPT_GENERATIONS + 1, PUBLISHED_GENERATIONS] {
                    let outcome = run(flags_policy, carry_policy, generation_count);
                    assert_eq!(
                        outcome.rows_lost,
                        expected_rows_lost_closed_form(flags_policy, carry_policy, generation_count),
                        "{flags_policy:?}/{carry_policy:?}/g={generation_count}"
                    );
                }
            }
        }
    }

    /// **代数边界**：恰好 K 代不丢，K+1 代才丢——off-by-one 钉死。
    #[test]
    fn absolute_generation_boundary() {
        assert_eq!(run(FlagsPolicy::Skip, CarryPolicy::NoCarry, KEPT_GENERATIONS).rows_lost, 0);
        assert_eq!(
            run(FlagsPolicy::Skip, CarryPolicy::NoCarry, KEPT_GENERATIONS + 1).rows_lost,
            2
        );
    }

    /// **判据 4**：搬运的每代多写行数是绝对值，且远低于作废阈值。
    #[test]
    fn absolute_carry_cost() {
        let extra = rows_per_generation(FlagsPolicy::Skip, CarryPolicy::Carry)
            - rows_per_generation(FlagsPolicy::Skip, CarryPolicy::NoCarry);
        assert_eq!(extra, UNKNOWN_WRITABLE_HEAD_COUNT * TREE_DIMENSION_ROWS_PER_HEAD);
        assert_eq!(extra, 2);
        // 写成加法：阈值 = KNOWN_TREE_COUNT × 12 = 144，2 + 142 = 144。
        assert_eq!(KNOWN_TREE_COUNT * (TREE_DIMENSION_ROWS_PER_HEAD + POOL_LEVEL_ROWS), extra + 142);
        assert!(extra < KNOWN_TREE_COUNT * (TREE_DIMENSION_ROWS_PER_HEAD + POOL_LEVEL_ROWS), "搬运比自己的账还贵就要改结论");
    }

    /// **每代写多少行**：三条臂各钉一个绝对值。
    #[test]
    fn absolute_rows_per_generation() {
        assert_eq!(rows_per_generation(FlagsPolicy::Refuse, CarryPolicy::Carry), 0);
        assert_eq!(rows_per_generation(FlagsPolicy::Ignore, CarryPolicy::NoCarry), 19);
        assert_eq!(rows_per_generation(FlagsPolicy::Skip, CarryPolicy::Carry), 19);
        assert_eq!(rows_per_generation(FlagsPolicy::Skip, CarryPolicy::NoCarry), 17);
    }

    /// **判据 3**：静默结局逐格可数，且今天与「不变量都实现之后」不是同一个数。
    #[test]
    fn absolute_silent_counts() {
        let ignore_carry_outcome = run(FlagsPolicy::Ignore, CarryPolicy::Carry, PUBLISHED_GENERATIONS);
        assert_eq!(ignore_carry_outcome.silent(), 1, "误读没人抓");
        assert_eq!(ignore_carry_outcome.silent_by_specification(), 1, "规范里也没有抓它的那一条");

        let skip_no_carry_outcome = run(FlagsPolicy::Skip, CarryPolicy::NoCarry, PUBLISHED_GENERATIONS);
        assert_eq!(skip_no_carry_outcome.silent(), 1, "I-9.11 未实现 ⇒ 今天静默");
        assert_eq!(skip_no_carry_outcome.silent_by_specification(), 0, "I-9.11 实现之后就不静默了");

        let skip_carry_outcome = run(FlagsPolicy::Skip, CarryPolicy::Carry, PUBLISHED_GENERATIONS);
        assert_eq!(skip_carry_outcome.silent(), 0);
        assert_eq!(skip_carry_outcome.silent_by_specification(), 0);
    }

    /// **不许只做臂间互比**：把「谁最少」这个结论本身钉成绝对值。
    #[test]
    fn absolute_verdict() {
        let mut fewest_silent = u64::MAX;
        let mut winners = Vec::new();
        for flags_policy in [FlagsPolicy::Ignore, FlagsPolicy::Skip, FlagsPolicy::Refuse] {
            for carry_policy in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
                let outcome = run(flags_policy, carry_policy, PUBLISHED_GENERATIONS);
                if !outcome.mount_ok {
                    continue;
                }
                let silent_count = outcome.silent();
                if silent_count < fewest_silent {
                    fewest_silent = silent_count;
                    winners = vec![name_of(flags_policy, carry_policy)];
                } else if silent_count == fewest_silent {
                    winners.push(name_of(flags_policy, carry_policy));
                }
            }
        }
        assert_eq!(fewest_silent, 0, "静默结局最少的格子必须是 0，不是「相对更少」");
        winners.sort();
        assert_eq!(winners, vec!["Skip_Carry".to_string()]);
    }
}
