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
//! 只有**可写头**才有带树维的记账行（每头 R_TREE_DIM 行）；其余统计量是全池的，与树无关。

use e7_index_bench::Emitter;

/// v1 认识的树总数。
const T_KNOWN: u64 = 12;
/// 其中是可写头的。
const H_KNOWN: u64 = 6;
/// v1 不认识的树总数（新种类，或已知种类但置了 v1 不认识的 flags 位）。
const T_NEW: u64 = 4;
/// 其中是可写头的 —— **只有这些才有记账行可丢**。
const H_NEW: u64 = 2;
/// 带树维的统计量条数：今天恰好 1（D5 已定项 4 第 12 项）。
const R_TREE_DIM: u64 = 1;
/// 不带树维的统计量条数：在用十二个减去带树维的那一个。
const R_POOL: u64 = 11;
/// 只保留最近 K 代。
const K_KEEP: u64 = 4;
/// 发布代数，取 > K_KEEP 才看得见丢弃。
const G_GENS: u64 = 10;
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
    rows_written_per_gen: u64,
    /// 按 invariants.md 的规范，这个丢失会不会被某条不变量抓到。
    caught_by_spec: bool,
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
    fn silent_by_spec(&self) -> u64 {
        let wrong = self.misread_records > 0 || self.rows_lost > 0;
        u64::from(wrong && !self.caught_by_spec)
    }
}

/// 一代写多少行：全池统计量 + 每个**自己维护的**可写头一行。
fn rows_per_gen(f: FlagsPolicy, c: CarryPolicy) -> u64 {
    match f {
        // 挂不上，一行都不写。
        FlagsPolicy::Refuse => 0,
        // 自以为认识 ⇒ 连不认识的头也一起维护。
        FlagsPolicy::Ignore => R_POOL + (H_KNOWN + H_NEW) * R_TREE_DIM,
        // 跳过 ⇒ 只维护自己认识的；搬运政策决定要不要把别人的原样带上。
        FlagsPolicy::Skip => match c {
            CarryPolicy::Carry => R_POOL + (H_KNOWN + H_NEW) * R_TREE_DIM,
            CarryPolicy::NoCarry => R_POOL + H_KNOWN * R_TREE_DIM,
        },
    }
}

fn run(f: FlagsPolicy, c: CarryPolicy, gens: u64) -> Outcome {
    let aged_out = gens > K_KEEP;
    match f {
        FlagsPolicy::Refuse => Outcome {
            mount_ok: false,
            misread_records: 0,
            rows_lost: 0,
            rows_written_per_gen: 0,
            caught_by_spec: false,
            caught_today: false,
        },
        FlagsPolicy::Ignore => Outcome {
            mount_ok: true,
            // 按错编码解释了不认识的树里的每一条记录。
            misread_records: T_NEW * RECORDS_PER_NEW_TREE,
            // 它自以为认识 ⇒ 照样维护那些行 ⇒ 一行不丢。
            rows_lost: 0,
            rows_written_per_gen: rows_per_gen(f, c),
            // 误读发生在记录编码这一层，树 ID 是对的 ⇒ I-1.3 够不着；
            // 记账行在册且值由错误语义算出 ⇒ I-9.11 判「有行」也过。
            caught_by_spec: false,
            caught_today: false,
        },
        FlagsPolicy::Skip => {
            let lost = match c {
                CarryPolicy::Carry => 0,
                CarryPolicy::NoCarry => {
                    if aged_out {
                        H_NEW * R_TREE_DIM
                    } else {
                        0
                    }
                }
            };
            Outcome {
                mount_ok: true,
                misread_records: 0,
                rows_lost: lost,
                rows_written_per_gen: rows_per_gen(f, c),
                // 丢的是可写头的「inode 号水位」行 ⇒ I-9.11 正对着它。
                caught_by_spec: lost > 0,
                // 而 I-9.11 未实现 ⇒ 今天没有任何东西会红。
                caught_today: false,
            }
        }
    }
}

/// 判据 2 要的那条**独立闭式**：不由模拟计数得来。
fn expected_rows_lost_closed_form(f: FlagsPolicy, c: CarryPolicy, gens: u64) -> u64 {
    if f != FlagsPolicy::Skip || c != CarryPolicy::NoCarry || gens <= K_KEEP {
        return 0;
    }
    H_NEW * R_TREE_DIM
}

fn name_of(f: FlagsPolicy, c: CarryPolicy) -> String {
    format!("{f:?}_{c:?}")
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config t_known={T_KNOWN} h_known={H_KNOWN} t_new={T_NEW} h_new={H_NEW} \
             r_tree_dim={R_TREE_DIM} r_pool={R_POOL} k_keep={K_KEEP} g_gens={G_GENS} \
             records_per_new_tree={RECORDS_PER_NEW_TREE} model=arithmetic file_ops=0"
        ))
    );

    let mut min_silent = u64::MAX;
    let mut best: Vec<String> = Vec::new();

    for f in [FlagsPolicy::Ignore, FlagsPolicy::Skip, FlagsPolicy::Refuse] {
        for c in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
            let o = run(f, c, G_GENS);
            let closed = expected_rows_lost_closed_form(f, c, G_GENS);
            let n = name_of(f, c);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=cell arm={n} mount_ok={} misread_records={} rows_lost={} \
                     rows_lost_closed_form={} rows_written_per_gen={} \
                     caught_by_spec={} caught_today={} silent_today={} silent_by_spec={}",
                    u8::from(o.mount_ok),
                    o.misread_records,
                    o.rows_lost,
                    closed,
                    o.rows_written_per_gen,
                    u8::from(o.caught_by_spec),
                    u8::from(o.caught_today),
                    o.silent(),
                    o.silent_by_spec(),
                ))
            );
            if o.mount_ok {
                match o.silent().cmp(&min_silent) {
                    std::cmp::Ordering::Less => {
                        min_silent = o.silent();
                        best = vec![n];
                    }
                    std::cmp::Ordering::Equal => best.push(n),
                    std::cmp::Ordering::Greater => {}
                }
            }
        }
    }

    // 判据 4：搬运的绝对写代价，按行数报，不报百分比。
    let carry_extra = rows_per_gen(FlagsPolicy::Skip, CarryPolicy::Carry)
        - rows_per_gen(FlagsPolicy::Skip, CarryPolicy::NoCarry);
    let own_rows = R_POOL + H_KNOWN * R_TREE_DIM;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=carry_cost extra_rows_per_gen={carry_extra} own_rows_per_gen={own_rows} \
             threshold_rows={} over_threshold={}",
            T_KNOWN * (R_TREE_DIM + R_POOL),
            u8::from(carry_extra > T_KNOWN * (R_TREE_DIM + R_POOL))
        ))
    );

    // 判据 5：Refuse 的代价。
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=refuse_cost unmountable_gens={G_GENS} compat_with_d15_item2=0"
        ))
    );

    // 结论行：静默结局最少的那些格子（只在挂得上的格子里比）。
    best.sort();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=verdict min_silent_today={min_silent} winners={}",
            best.join("+")
        ))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **绝对值锚点**：模型的每个输入常量都钉住，改一个就红。
    /// 手算：带树维的统计量今天 1 项（D5 已定项 4 第 12 项），全池 11 项，合计在用十二个。
    #[test]
    fn absolute_model_constants() {
        assert_eq!(R_TREE_DIM, 1);
        assert_eq!(R_POOL, 11);
        assert_eq!(R_TREE_DIM + R_POOL, 12, "在用十二个统计量");
        assert!(H_KNOWN <= T_KNOWN);
        assert!(H_NEW <= T_NEW);
        assert!(G_GENS > K_KEEP, "代数要大过保留代数，否则丢弃看不见");
    }

    /// **阳性对照甲**：Skip + NoCarry 且 G > K，必须丢行。
    #[test]
    fn positive_control_skip_nocarry_loses_rows() {
        let o = run(FlagsPolicy::Skip, CarryPolicy::NoCarry, G_GENS);
        assert!(o.rows_lost > 0, "代际丢弃这一维没进模型");
        // 绝对值：写成加法不写减法，让变异活到运行期。
        assert_eq!(o.rows_lost, H_NEW * R_TREE_DIM);
        assert_eq!(o.rows_lost, 2);
    }

    /// **阳性对照乙**：Ignore 臂下写者自以为认识，行不丢而记录被误读。
    #[test]
    fn positive_control_ignore_misreads_but_keeps_rows() {
        for c in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
            let o = run(FlagsPolicy::Ignore, c, G_GENS);
            assert_eq!(o.rows_lost, 0, "自以为认识就会照样维护那些行");
            assert!(o.misread_records > 0, "「自以为认识」这一维没进模型");
            assert_eq!(o.misread_records, T_NEW * RECORDS_PER_NEW_TREE);
            assert_eq!(o.misread_records, 4000);
        }
    }

    /// **阳性对照丙**：Refuse 臂挂不上，且不产生任何错误。
    #[test]
    fn positive_control_refuse_mounts_nothing() {
        for c in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
            let o = run(FlagsPolicy::Refuse, c, G_GENS);
            assert!(!o.mount_ok);
            assert_eq!(o.rows_lost, 0);
            assert_eq!(o.misread_records, 0);
            assert_eq!(o.rows_written_per_gen, 0);
        }
    }

    /// **阴性对照**：Skip + Carry 且 G > K，一行不丢。
    #[test]
    fn negative_control_skip_carry_loses_nothing() {
        let o = run(FlagsPolicy::Skip, CarryPolicy::Carry, G_GENS);
        assert_eq!(o.rows_lost, 0, "搬运这一维没进模型");
    }

    /// **判据 2**：模拟出的丢失数与独立闭式逐格相等。
    #[test]
    fn closed_form_matches_every_cell() {
        for f in [FlagsPolicy::Ignore, FlagsPolicy::Skip, FlagsPolicy::Refuse] {
            for c in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
                for g in [1u64, K_KEEP, K_KEEP + 1, G_GENS] {
                    let o = run(f, c, g);
                    assert_eq!(
                        o.rows_lost,
                        expected_rows_lost_closed_form(f, c, g),
                        "{f:?}/{c:?}/g={g}"
                    );
                }
            }
        }
    }

    /// **代数边界**：恰好 K 代不丢，K+1 代才丢——off-by-one 钉死。
    #[test]
    fn absolute_generation_boundary() {
        assert_eq!(run(FlagsPolicy::Skip, CarryPolicy::NoCarry, K_KEEP).rows_lost, 0);
        assert_eq!(
            run(FlagsPolicy::Skip, CarryPolicy::NoCarry, K_KEEP + 1).rows_lost,
            2
        );
    }

    /// **判据 4**：搬运的每代多写行数是绝对值，且远低于作废阈值。
    #[test]
    fn absolute_carry_cost() {
        let extra = rows_per_gen(FlagsPolicy::Skip, CarryPolicy::Carry)
            - rows_per_gen(FlagsPolicy::Skip, CarryPolicy::NoCarry);
        assert_eq!(extra, H_NEW * R_TREE_DIM);
        assert_eq!(extra, 2);
        // 写成加法：阈值 = T_KNOWN × 12 = 144，2 + 142 = 144。
        assert_eq!(T_KNOWN * (R_TREE_DIM + R_POOL), extra + 142);
        assert!(extra < T_KNOWN * (R_TREE_DIM + R_POOL), "搬运比自己的账还贵就要改结论");
    }

    /// **每代写多少行**：三条臂各钉一个绝对值。
    #[test]
    fn absolute_rows_per_gen() {
        assert_eq!(rows_per_gen(FlagsPolicy::Refuse, CarryPolicy::Carry), 0);
        assert_eq!(rows_per_gen(FlagsPolicy::Ignore, CarryPolicy::NoCarry), 19);
        assert_eq!(rows_per_gen(FlagsPolicy::Skip, CarryPolicy::Carry), 19);
        assert_eq!(rows_per_gen(FlagsPolicy::Skip, CarryPolicy::NoCarry), 17);
    }

    /// **判据 3**：静默结局逐格可数，且今天与「不变量都实现之后」不是同一个数。
    #[test]
    fn absolute_silent_counts() {
        let ig = run(FlagsPolicy::Ignore, CarryPolicy::Carry, G_GENS);
        assert_eq!(ig.silent(), 1, "误读没人抓");
        assert_eq!(ig.silent_by_spec(), 1, "规范里也没有抓它的那一条");

        let sk_nc = run(FlagsPolicy::Skip, CarryPolicy::NoCarry, G_GENS);
        assert_eq!(sk_nc.silent(), 1, "I-9.11 未实现 ⇒ 今天静默");
        assert_eq!(sk_nc.silent_by_spec(), 0, "I-9.11 实现之后就不静默了");

        let sk_c = run(FlagsPolicy::Skip, CarryPolicy::Carry, G_GENS);
        assert_eq!(sk_c.silent(), 0);
        assert_eq!(sk_c.silent_by_spec(), 0);
    }

    /// **不许只做臂间互比**：把「谁最少」这个结论本身钉成绝对值。
    #[test]
    fn absolute_verdict() {
        let mut min = u64::MAX;
        let mut winners = Vec::new();
        for f in [FlagsPolicy::Ignore, FlagsPolicy::Skip, FlagsPolicy::Refuse] {
            for c in [CarryPolicy::Carry, CarryPolicy::NoCarry] {
                let o = run(f, c, G_GENS);
                if !o.mount_ok {
                    continue;
                }
                let s = o.silent();
                if s < min {
                    min = s;
                    winners = vec![name_of(f, c)];
                } else if s == min {
                    winners.push(name_of(f, c));
                }
            }
        }
        assert_eq!(min, 0, "静默结局最少的格子必须是 0，不是「相对更少」");
        winners.sort();
        assert_eq!(winners, vec!["Skip_Carry".to_string()]);
    }
}
