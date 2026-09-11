//! E130：每头一份 livelist 过不过 D6 判据 5「有界销毁」。
//!
//! D6 未定项 2 的第一轮三方（2026-09-10）把丙与乙判出局、丁的代价重估，
//! 只剩甲（独立 livelist）存活；而反推腿自己明写「我的结论只到『乙过不了』，
//! **不到『甲过得了』**」——甲有条目数，可「每批最坏空间需求」的算式与
//! 「可 condense」的代价仓里一个数都没有。E130 补的就是这一格。
//!
//! **判据、阈值、作废条款、以及「结果反过来我接不接受」写在
//! `research/prompts/e130-preregistration.md`，写于本文件之前。**
//!
//! ## 这是计数模型，不是实现
//!
//! 没有文件 I/O、没有并发、没有随机源 ⇒ 同一个二进制跑 N 遍必然逐字节一致。
//! 「N 轮一致」说明的是没有隐藏状态，不是统计上稳定
//! （`.claude/singlefs-ai-sop/rules/test-discipline.md`）。
//! 证据强度来自变异测试与钉死绝对值的断言，不来自轮数。
//!
//! ## 跨装置闸：两个常量必须落回 kb
//!
//! 单元恒 32768 字节含头（D4 已定项 7 / 已定项 3），分配记录条目 30 字节
//! （D3 已定项 7 与 D5 已定项 5 同宽口径）。这两个数不在本文件里自己发明，
//! 对不上 kb 就是口径漂了（`show-me-test.md`「立在装置之间」）。

use e7_index_bench::Emitter;

/// 单元字节数，含头。D4（校验和位置） 已定项 3 / 已定项 7。
const UNIT_BYTES: u64 = 32768;
/// 一条分配记录条目的盘上宽度。D3（空间分配） 已定项 7 的字段表逐段相加：
/// 设备身份 4 + 落点（16 KiB 槽号）6 + 跨度段 2 + 分配代 / 释放代 8。
///
/// ⚠️ **第一版写成 30，挂的却是 D3（空间分配） 的名字，2026-09-10 当天改正**：
/// 30 是 D5（快照 / 空间记账机制） 已定项 5 的**记账**条目宽（key 22 + value 8），
/// 不是分配记录的宽。跨装置闸当时确实写了、变异也抓得到、门禁全绿——
/// **它只是钉在了另一条决策的数上**，正是 `show-me-test.md`
/// 「修坑的人最容易在这里收手……不会再想一遍这条检查的射程有多远」说的那个形态。
/// 是三方论证的正推腿逐段相加 D3（空间分配） 的字段表时抓到的，不是任何一条断言。
const ALLOCATION_RECORD_BYTES: u64 = 20;
/// 一条 livelist 条目：类型标签 2 + 物理指针 14（D19 已定项 4 的位置条目宽） + birth txg 8。
const LIVELIST_ENTRY_BYTES: u64 = 2 + 14 + 8;
/// 一批处理多少条 livelist 条目。判据 1 要证 worst_batch_bytes 只随它走。
const ENTRIES_PER_BATCH: u64 = 4096;
/// condense 触发判据：结构膨胀到净活块数的几倍就抵消一次。
///
/// ⚠️ **跑前写死的第一版写反了，改在任何测量之前，理由留档**：原判据是
/// 「FREE 条目占比 ≥ 1/2 就 condense」，而这族负载里
/// `free_event_count / (allocation_event_count + free_event_count) = c / (1 + 2c) < 1/2` 对**任何有限 c** 成立
/// ⇒ 前件恒假、condense 臂一次也不触发，等于 naive 的复制品。
/// 那是 `test-discipline.md`「失败条款的前件可以写反，写反之后它永远不触发」的形态，
/// 也是 `evidence-discipline.md` 点名的稻草人对照臂——一条永不触发的 condense
/// 不是支持 livelist 的人会建的东西。
/// 现判据 `entries > CONDENSE_RATIO × net_allocated_block_count`，在 c ≥ 1 的格上触发、c = 0 不触发。
/// 写反那一版由单测 `the_first_condense_predicate_never_fires` 留档。
const CONDENSE_RATIO: u64 = 2;

/// 一条臂在一个负载格上的全部读数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Reading {
    /// 结构里现存的条目数。
    entries: u64,
    /// 净活块数（头自己出生、还没被释放的那批）。
    net_allocated_block_count: u64,
    /// 处理一批最坏要写的字节数。
    worst_batch_bytes: u64,
    /// 写意图那一刻算得出的待删占用上界。
    pre_reserve_bytes: u64,
    /// 真实待删字节。
    true_pending_bytes: u64,
    /// 销毁时要读的条目数（乙 读的是活树节点数）。
    destroy_reads: u64,
    /// 头里释放一个块，额外要预付几字节。
    free_path_extra_bytes: u64,
}

/// 一个负载格。`inherited_block_count` 是从 origin 继承的块数，不进 livelist。
#[derive(Debug, Clone, Copy)]
struct Load {
    live_block_count: u64,
    churn: u64,
    inherited_block_count: u64,
}

impl Load {
    fn new(live_block_count: u64, churn: u64, share: f64) -> Self {
        let inherited_block_count = (live_block_count as f64 * share).round() as u64;
        Load { live_block_count, churn, inherited_block_count }
    }
    /// 头自己出生的块数（`birth > origin.txg`）。只有这些进 livelist。
    fn self_born_block_count(&self) -> u64 {
        self.live_block_count - self.inherited_block_count
    }
    /// ALLOC 事件数：初始写 self_born_block_count() 次，每轮覆写再写 self_born_block_count() 次。
    fn allocation_event_count(&self) -> u64 {
        self.self_born_block_count() * (1 + self.churn)
    }
    /// FREE 事件数：每次覆写释放一个旧块。
    fn free_event_count(&self) -> u64 {
        self.self_born_block_count() * self.churn
    }
}

/// 活树节点数：乙 销毁时要扫的对象。含继承来的那部分——乙 没有索引，分不出谁是谁。
fn live_tree_nodes(load: &Load) -> u64 {
    load.live_block_count
}

fn naive(load: &Load) -> Reading {
    let entries = load.allocation_event_count() + load.free_event_count();
    let net_allocated_block_count = load.allocation_event_count() - load.free_event_count();
    Reading {
        entries,
        net_allocated_block_count,
        worst_batch_bytes: ENTRIES_PER_BATCH * (ALLOCATION_RECORD_BYTES + LIVELIST_ENTRY_BYTES),
        // 未 condense 时只数得出条目总数里的 ALLOC 那一半，抵消关系还没算过。
        pre_reserve_bytes: load.allocation_event_count() * UNIT_BYTES,
        true_pending_bytes: net_allocated_block_count * UNIT_BYTES,
        destroy_reads: entries,
        free_path_extra_bytes: LIVELIST_ENTRY_BYTES,
    }
}

/// condense 判据本身，单独拿出来是为了能在负载族**取不到**的取样点上验它。
///
/// ⚠️ 这族负载里 `raw_entry_count / net_allocated_block_count = 1 + 2c`，只取得到 1、3、9、33，
/// **取不到 `(1, 2]` 这一段**——而「判据恒触发」那条变异恰好只在那一段上
/// 才与原式不同，于是它在全部负载格上都被判绿。
/// 按 `.claude/rules/mutation-sampling.md` 第三类「取样点不敏感」，
/// 处置是**补一个敏感的取样点**，不是记成等价变异留档。
fn condense_entries(raw_entry_count: u64, net_allocated_block_count: u64) -> u64 {
    // 结构膨胀到净活块数的 CONDENSE_RATIO 倍就抵消一次，抵消后只剩净 ALLOC。
    if raw_entry_count > CONDENSE_RATIO * net_allocated_block_count {
        net_allocated_block_count
    } else {
        raw_entry_count
    }
}

fn condense(load: &Load) -> Reading {
    let raw_entry_count = load.allocation_event_count() + load.free_event_count();
    let net_allocated_block_count = load.allocation_event_count() - load.free_event_count();
    let entries = condense_entries(raw_entry_count, net_allocated_block_count);
    Reading {
        entries,
        net_allocated_block_count,
        worst_batch_bytes: ENTRIES_PER_BATCH * (ALLOCATION_RECORD_BYTES + LIVELIST_ENTRY_BYTES),
        pre_reserve_bytes: entries * UNIT_BYTES,
        true_pending_bytes: net_allocated_block_count * UNIT_BYTES,
        destroy_reads: entries,
        free_path_extra_bytes: LIVELIST_ENTRY_BYTES,
    }
}

fn no_structure(load: &Load) -> Reading {
    let net_allocated_block_count = load.allocation_event_count() - load.free_event_count();
    Reading {
        entries: 0,
        net_allocated_block_count,
        worst_batch_bytes: ENTRIES_PER_BATCH * ALLOCATION_RECORD_BYTES,
        // 乙 在写意图那一刻给不出上界：能当上界用的每树字节总量
        // 是 D5 已定项 4 第 8 / 9 项，2026-09-06 已撤回 ⇒ 报 0 表示「算不出」。
        pre_reserve_bytes: 0,
        true_pending_bytes: net_allocated_block_count * UNIT_BYTES,
        destroy_reads: live_tree_nodes(load),
        free_path_extra_bytes: 0,
    }
}

const ARMS: [(&str, fn(&Load) -> Reading); 3] =
    [("naive", naive), ("condense", condense), ("no_structure", no_structure)];

const LIVE_BLOCK_COUNTS: [u64; 3] = [1024, 8192, 65536];
const CHURNS: [u64; 4] = [0, 1, 4, 16];
const SHARES: [f64; 2] = [0.0, 0.7];

fn overestimate(reading: &Reading) -> f64 {
    if reading.true_pending_bytes == 0 {
        return f64::NAN;
    }
    reading.pre_reserve_bytes as f64 / reading.true_pending_bytes as f64
}

fn main() {
    let mut emitter = Emitter::new();
    println!("E130 每头一份 livelist 过不过 D6 判据 5「有界销毁」");
    println!("判据写死在 research/prompts/e130-preregistration.md（写于本装置之前）");
    println!(
        "常量：UNIT_BYTES={UNIT_BYTES} ALLOC_REC_BYTES={ALLOCATION_RECORD_BYTES} \
         LIVELIST_ENTRY_BYTES={LIVELIST_ENTRY_BYTES} BATCH_K={ENTRIES_PER_BATCH}"
    );

    for (arm_name, arm_function) in ARMS {
        for &share in SHARES.iter() {
            for &live_block_count in LIVE_BLOCK_COUNTS.iter() {
                for &churn in CHURNS.iter() {
                    let load = Load::new(live_block_count, churn, share);
                    let reading = arm_function(&load);
                    let overestimate_ratio = overestimate(&reading);
                    let overestimate_text = if overestimate_ratio.is_nan() { "NA".to_string() } else { format!("{overestimate_ratio:.4}") };
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name={arm_name} n={live_block_count} churn={churn} share={share:.1} \
                             entries={} net_alloc={} worst_batch_bytes={} \
                             pre_reserve_bytes={} true_pending_bytes={} \
                             overestimate={overestimate_text} destroy_reads={} free_path_extra_bytes={}",
                            reading.entries,
                            reading.net_allocated_block_count,
                            reading.worst_batch_bytes,
                            reading.pre_reserve_bytes,
                            reading.true_pending_bytes,
                            reading.destroy_reads,
                            reading.free_path_extra_bytes
                        ))
                    );
                }
            }
        }
    }

    // 判据 1：worst_batch_bytes 只随 K 走。扫遍全部格，取值集合大小必须是 1。
    for (arm_name, arm_function) in ARMS {
        let mut distinct_worst_batch_bytes_values: Vec<u64> = Vec::new();
        for &share in SHARES.iter() {
            for &live_block_count in LIVE_BLOCK_COUNTS.iter() {
                for &churn in CHURNS.iter() {
                    let worst_batch_bytes = arm_function(&Load::new(live_block_count, churn, share)).worst_batch_bytes;
                    if !distinct_worst_batch_bytes_values.contains(&worst_batch_bytes) {
                        distinct_worst_batch_bytes_values.push(worst_batch_bytes);
                    }
                }
            }
        }
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=criterion1 arm={arm_name} distinct_worst_batch_bytes={} value={}",
                distinct_worst_batch_bytes_values.len(),
                distinct_worst_batch_bytes_values[0]
            ))
        );
    }

    // 判据 4 的阳性对照：naive 在 churn 上必须随 C 线性长。
    let entries_at_churn_zero = naive(&Load::new(8192, 0, 0.0)).entries;
    let entries_at_churn_sixteen = naive(&Load::new(8192, 16, 0.0)).entries;
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control arm=naive n=8192 entries_c0={entries_at_churn_zero} entries_c16={entries_at_churn_sixteen} ratio={}",
            entries_at_churn_sixteen / entries_at_churn_zero
        ))
    );

    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 钉绝对值的断言（防「所有臂一起错」）────────────────────────────

    #[test]
    fn naive_entries_at_zero_churn_without_inheritance_equal_the_live_block_count() {
        // 跑前写死的作废条款 3：N 次 ALLOC、0 次 FREE。
        assert_eq!(naive(&Load::new(1024, 0, 0.0)).entries, 1024);
        assert_eq!(naive(&Load::new(65536, 0, 0.0)).entries, 65536);
    }

    #[test]
    fn naive_entries_at_zero_churn_with_share_excludes_inherited() {
        // 0.7 继承率下只有 self_born_block_count() 进 livelist：1024 − round(0.7×1024) = 1024 − 717 = 307。
        assert_eq!(Load::new(1024, 0, 0.7).inherited_block_count, 717);
        assert_eq!(naive(&Load::new(1024, 0, 0.7)).entries, 307);
    }

    #[test]
    fn livelist_entry_is_twenty_four_bytes() {
        // 2 + 14 + 8。14 逐字是 D19 已定项 4 的位置条目宽。
        assert_eq!(LIVELIST_ENTRY_BYTES, 2 + 14 + 8);
        assert_eq!(LIVELIST_ENTRY_BYTES, 24);
    }

    #[test]
    fn worst_batch_bytes_for_livelist_is_batch_times_two_records() {
        // 写成加法不是减法：变异把常量改大时不会编译期溢出。
        assert_eq!(
            naive(&Load::new(8192, 4, 0.0)).worst_batch_bytes,
            ENTRIES_PER_BATCH * (ALLOCATION_RECORD_BYTES + LIVELIST_ENTRY_BYTES)
        );
        assert_eq!(naive(&Load::new(8192, 4, 0.0)).worst_batch_bytes, 4096 * 44);
        assert_eq!(4096 * 44, 180_224);
    }

    // ── 判据 1：每批最坏空间需求只随 K 走 ──────────────────────────────

    #[test]
    fn worst_batch_bytes_is_constant_across_every_load_cell() {
        for (_, arm_function) in ARMS {
            let mut first_worst_batch_bytes_seen: Option<u64> = None;
            for &share in SHARES.iter() {
                for &live_block_count in LIVE_BLOCK_COUNTS.iter() {
                    for &churn in CHURNS.iter() {
                        let worst_batch_bytes = arm_function(&Load::new(live_block_count, churn, share)).worst_batch_bytes;
                        match first_worst_batch_bytes_seen {
                            None => first_worst_batch_bytes_seen = Some(worst_batch_bytes),
                            Some(first_worst_batch_bytes) => assert_eq!(
                                worst_batch_bytes, first_worst_batch_bytes,
                                "worst_batch_bytes 随负载变了：n={live_block_count} churn={churn} share={share}"
                            ),
                        }
                    }
                }
            }
        }
    }

    // ── 判据 4：阳性对照必须分得出差别 ─────────────────────────────────

    #[test]
    fn positive_control_naive_grows_linearly_with_churn() {
        // 作废条款 1 的触发观测：C=0 与 C=16 相等 ⇒ 装置分不出差别。
        let entries_at_churn_zero = naive(&Load::new(8192, 0, 0.0)).entries;
        let entries_at_churn_sixteen = naive(&Load::new(8192, 16, 0.0)).entries;
        assert_ne!(entries_at_churn_zero, entries_at_churn_sixteen, "阳性对照：naive 在 churn 上没长 ⇒ 整轮作废");
        // 1 + 2×16 = 33 倍。
        assert_eq!(entries_at_churn_zero, 8192);
        assert_eq!(entries_at_churn_sixteen, 8192 * 33);
    }

    #[test]
    fn positive_control_runs_on_every_arm_not_just_the_first() {
        // 「阳性对照必须对每一条被测的臂都跑」——三条臂各自问一次
        // 「C 从 0 到 16，这条臂的 entries 变不变」，并把各自的答案钉死。
        assert_ne!(
            naive(&Load::new(8192, 0, 0.0)).entries,
            naive(&Load::new(8192, 16, 0.0)).entries
        );
        // condense 的判别力在「触发之后塌不塌得回净值」上：
        // c=0 不触发 ⇒ entries 等于 raw_entry_count；c=1 触发 ⇒ entries 等于净值。
        assert_eq!(condense(&Load::new(8192, 0, 0.0)).entries, 8192);
        assert_eq!(condense(&Load::new(8192, 1, 0.0)).entries, 8192);
        assert_eq!(naive(&Load::new(8192, 1, 0.0)).entries, 8192 * 3);
        // no_structure 恒 0 条目，它的判别力在 destroy_reads 上。
        assert_eq!(no_structure(&Load::new(8192, 0, 0.0)).entries, 0);
        assert_eq!(no_structure(&Load::new(8192, 16, 0.0)).entries, 0);
        assert_eq!(no_structure(&Load::new(8192, 16, 0.0)).destroy_reads, 8192);
    }

    // ── 判据 5：condense 之后回不回得到 O(净活块数) ────────────────────

    #[test]
    fn condense_predicate_leaves_the_structure_alone_below_the_ratio() {
        // 取样点不敏感的补丁：负载族取不到 raw_entry_count/net_allocated_block_count ∈ (1, 2]，而那正是
        // 「判据恒触发」与原式唯一不同的那一段。这三个点直接钉住它。
        assert_eq!(condense_entries(150, 100), 150); // 1.5× ⇒ 不动
        assert_eq!(condense_entries(200, 100), 200); // 恰好 2× ⇒ 不动（严格大于才动）
        assert_eq!(condense_entries(201, 100), 100); // 越过 2× ⇒ 塌回净值
    }

    #[test]
    fn the_load_family_cannot_reach_the_sensitive_ratio() {
        // 留档：上面那条为什么非要绕开 Load 直接调判据。
        for &churn in CHURNS.iter() {
            let load = Load::new(8192, churn, 0.0);
            let raw_entry_count = load.allocation_event_count() + load.free_event_count();
            let net_allocated_block_count = load.allocation_event_count() - load.free_event_count();
            assert!(
                raw_entry_count == net_allocated_block_count || raw_entry_count > CONDENSE_RATIO * net_allocated_block_count,
                "负载族竟然取到了 (1, 2] 这一段：churn={churn} raw={raw_entry_count} net={net_allocated_block_count}"
            );
        }
    }

    #[test]
    fn the_first_condense_predicate_never_fires() {
        // 留档：跑前写死的第一版判据是「FREE 占比 ≥ 1/2」，而这族负载里
        // free_event_count/(allocation_event_count+free_event_count) = c/(1+2c)，对任何有限 c 都严格小于 1/2
        // ⇒ 前件恒假，condense 臂会退化成 naive 的复制品。
        for &churn in CHURNS.iter() {
            let load = Load::new(8192, churn, 0.0);
            let raw_entry_count = load.allocation_event_count() + load.free_event_count();
            assert!(
                load.free_event_count() * 2 < raw_entry_count,
                "第一版判据在 churn={churn} 上竟然触发了 —— 那这条留档写错了"
            );
        }
    }

    #[test]
    fn condense_collapses_to_net_allocated_block_count_from_churn_one_upward() {
        // 现判据 raw_entry_count > 2 × net_allocated_block_count ⇔ 1 + 2c > 2 ⇔ c ≥ 1。
        assert_eq!(condense(&Load::new(1024, 0, 0.0)).entries, 1024); // c=0 不触发
        assert_eq!(condense(&Load::new(1024, 1, 0.0)).entries, 1024); // c=1 触发，塌回净值
        assert_eq!(condense(&Load::new(1024, 16, 0.0)).entries, 1024);
        // 而 naive 在同两格上是 3072 与 33792 —— 两条臂必须分得开。
        assert_eq!(naive(&Load::new(1024, 1, 0.0)).entries, 3072);
        assert_eq!(naive(&Load::new(1024, 16, 0.0)).entries, 33792);
    }

    #[test]
    fn condense_never_reports_fewer_entries_than_net_allocated_block_count() {
        // 作废条款 2 的触发观测：entries < allocation_event_count − free_event_count ⇒ 结构装不下它该装的。
        for &share in SHARES.iter() {
            for &live_block_count in LIVE_BLOCK_COUNTS.iter() {
                for &churn in CHURNS.iter() {
                    let load = Load::new(live_block_count, churn, share);
                    for (arm_name, arm_function) in [("naive", naive as fn(&Load) -> Reading), ("condense", condense)] {
                        let reading = arm_function(&load);
                        assert!(
                            reading.entries >= reading.net_allocated_block_count,
                            "{arm_name} 报出的条目数少于净活块数：n={live_block_count} churn={churn} share={share}"
                        );
                    }
                }
            }
        }
    }

    // ── 判据 3：高估比 ────────────────────────────────────────────────

    #[test]
    fn naive_overestimates_pending_bytes_without_bound_in_churn() {
        // naive 数不出抵消关系 ⇒ 上界 = 全部 ALLOC，随 C 线性发散。
        let overestimate_at_churn_zero = overestimate(&naive(&Load::new(8192, 0, 0.0)));
        let overestimate_at_churn_sixteen = overestimate(&naive(&Load::new(8192, 16, 0.0)));
        assert_eq!(overestimate_at_churn_zero, 1.0);
        assert_eq!(overestimate_at_churn_sixteen, 17.0);
        assert!(overestimate_at_churn_sixteen > overestimate_at_churn_zero);
    }

    #[test]
    fn condense_overestimate_equals_entries_over_net() {
        let load = Load::new(8192, 4, 0.0);
        let reading = condense(&load);
        assert_eq!(reading.pre_reserve_bytes, reading.entries * UNIT_BYTES);
        assert_eq!(overestimate(&reading), reading.entries as f64 / reading.net_allocated_block_count as f64);
    }

    // ── 判据 2：上界算式里不许出现活树节点数 ───────────────────────────

    #[test]
    fn livelist_pre_reserve_does_not_read_the_live_tree() {
        // 同一个净活块数、不同的活树规模（继承部分不同）下，
        // livelist 两臂的 pre_reserve_bytes 必须只随自己的条目走。
        let load_with_small_live_tree = Load::new(1000, 0, 0.0); // self_born_block_count=1000, 活树 1000
        let load_with_large_live_tree = Load::new(2000, 0, 0.5); // self_born_block_count=1000, 活树 2000
        assert_eq!(load_with_small_live_tree.self_born_block_count(), load_with_large_live_tree.self_born_block_count());
        assert_eq!(naive(&load_with_small_live_tree).pre_reserve_bytes, naive(&load_with_large_live_tree).pre_reserve_bytes);
        assert_eq!(condense(&load_with_small_live_tree).pre_reserve_bytes, condense(&load_with_large_live_tree).pre_reserve_bytes);
        // 而乙 的 destroy_reads 随活树走 —— 这就是两者的分界。
        assert_ne!(no_structure(&load_with_small_live_tree).destroy_reads, no_structure(&load_with_large_live_tree).destroy_reads);
    }

    #[test]
    fn no_structure_cannot_produce_a_bound_at_intent_time() {
        // 报 0 表示「算不出」，不是「不需要空间」——真实待删字节同时报出来做对照。
        let reading = no_structure(&Load::new(8192, 4, 0.0));
        assert_eq!(reading.pre_reserve_bytes, 0);
        assert!(reading.true_pending_bytes > 0);
    }

    // ── 判据 6：释放路径的额外预付 ────────────────────────────────────

    #[test]
    fn livelist_arms_charge_the_free_path_and_no_structure_does_not() {
        assert_eq!(naive(&Load::new(1024, 1, 0.0)).free_path_extra_bytes, 24);
        assert_eq!(condense(&Load::new(1024, 1, 0.0)).free_path_extra_bytes, 24);
        assert_eq!(no_structure(&Load::new(1024, 1, 0.0)).free_path_extra_bytes, 0);
    }

    // ── 负载口径自身的断言 ────────────────────────────────────────────

    #[test]
    fn churn_counts_one_allocation_and_one_free_each_round() {
        let load = Load::new(100, 3, 0.0);
        assert_eq!(load.allocation_event_count(), 400);
        assert_eq!(load.free_event_count(), 300);
        assert_eq!(load.allocation_event_count() - load.free_event_count(), 100);
    }

    #[test]
    fn inherited_blocks_never_enter_the_livelist() {
        let load = Load::new(1000, 5, 0.7);
        assert_eq!(load.inherited_block_count, 700);
        assert_eq!(load.self_born_block_count(), 300);
        assert_eq!(load.allocation_event_count(), 300 * 6);
        assert_eq!(naive(&load).net_allocated_block_count, 300);
    }

    #[test]
    fn unit_bytes_matches_the_knowledge_base_constant() {
        // 跨装置闸：D4 已定项 3 / 已定项 7 钉死单元恒 32768 含头。
        assert_eq!(UNIT_BYTES, 32768);
        assert_eq!(UNIT_BYTES, 32 * 1024);
    }

    #[test]
    fn allocation_record_matches_the_knowledge_base_field_table_segment_by_segment() {
        // 跨装置闸：钉的是 D3 已定项 7 那张字段表的**逐段**，不是一个总数——
        // 只钉总数时，一个来自别条决策的同量级数字照样能让断言全绿（实测踩过）。
        let device_identity_bytes = 4u64; // 设备身份，D19 已定项 4「同一个坐标系」
        let placement_slot_bytes = 6u64; // 落点，编码为 16 KiB 槽号
        let span_segment_bytes = 2u64; // 跨度段（含已释放标志位），初值
        let allocation_and_free_generation_bytes = 8u64; // 分配代 / 释放代，与 checkpoint_txg 同宽
        assert_eq!(ALLOCATION_RECORD_BYTES, device_identity_bytes + placement_slot_bytes + span_segment_bytes + allocation_and_free_generation_bytes);
        assert_eq!(ALLOCATION_RECORD_BYTES, 20);
    }

    #[test]
    fn the_allocation_record_width_is_not_the_ledger_entry_width() {
        // 留档：第一版把这两个数搞混了。记账条目是 D5 已定项 5 的 key 22 + value 8 = 30，
        // 与分配记录的 20 是两棵树上的两个量，差 10 字节。
        let ledger_entry_bytes = 22u64 + 8;
        assert_eq!(ledger_entry_bytes, 30);
        assert_ne!(ALLOCATION_RECORD_BYTES, ledger_entry_bytes);
    }
}
