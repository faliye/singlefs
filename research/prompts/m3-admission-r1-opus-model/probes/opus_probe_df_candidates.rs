//! 攻方腿 m3-admission-r1-opus，A1（C370、C371）与 A4（C571）：`df` 的几种候选写法，在同一段历史的每一个状态上与「这一刻写得进的最大字节数」对照。
//! 写得进的最大字节数：在池的拷贝上顺序写 n 个数据单元（整份重写，载荷 n × 32634 字节），二分找做得成的最大 n（会话照常推抬 F）。
//! 一个候选报的 > 真值，就有一个写 s 字节（真值 < s ≤ 候选）在 `df` ≥ s 时被拒：D3（空间分配） 已定项 9 第 1 条的假性 ENOSPC。
//! 只在冻结副本的拷贝上跑，不进仓库的 crates/。
mod common_admission;
#[path = "opus_probe/common.rs"]
mod probe_common;

use probe_common::{clone_pool, counts_of, line_of, start_pool};
use singlefs_core::admission::{
    checkpoint_cost_of_the_version_to_build_on, space_budget_of_role, SpaceBudgetOfARole,
};
use singlefs_core::transaction::{journal_record_limit_of_one_publish, TransactionUnit};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::SparseBlockDevice;

use common_admission::PoolUnderTest;

fn bytes_of_units(units: usize) -> usize {
    units * data_unit_payload_capacity()
}

fn write_succeeds(pool: &PoolUnderTest<SparseBlockDevice>, units: usize) -> bool {
    if units == 0 {
        return true;
    }
    let mut probe = clone_pool(pool);
    probe.sequential_write(bytes_of_units(units)).is_ok()
}

/// 这一刻顺序写做得成的最大单元数（二分；再核 n 做得成、n + 1 做不成）。
fn largest_write_in_units(pool: &PoolUnderTest<SparseBlockDevice>, upper: usize) -> (usize, bool) {
    let (mut low, mut high) = (0usize, upper + 1);
    while high - low > 1 {
        let middle = (low + high) / 2;
        if write_succeeds(pool, middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    let checked = write_succeeds(pool, low) && (low == upper || !write_succeeds(pool, low + 1));
    (low, checked)
}

/// 一段历史的驱动：记下非空（改用户可见状态的）发布的 txg，`df` 的 D16 读法要第 4 新的非空根。
struct Driven {
    pool: PoolUnderTest<SparseBlockDevice>,
    non_empty_txgs: Vec<u64>,
    file_units: usize,
    record_limit: usize,
    states: usize,
}

impl Driven {
    fn sequential_write(&mut self, units: usize) -> bool {
        match self.pool.sequential_write(bytes_of_units(units)) {
            Ok(published) => {
                self.non_empty_txgs
                    .extend(published.versions_published.iter().map(|v| v.root.checkpoint_txg.0));
                self.file_units = units;
                true
            }
            Err(_) => false,
        }
    }

    fn overwrite_small(&mut self) -> bool {
        match self.pool.overwrite(2999) {
            Ok(published) => {
                self.non_empty_txgs
                    .extend(published.versions_published.iter().map(|v| v.root.checkpoint_txg.0));
                self.file_units = 1;
                true
            }
            Err(_) => false,
        }
    }

    /// 这一刻每个候选 `df` 报的字节数与真值（写得进的最大载荷字节）对照，一行。
    fn evaluate(&mut self, tag: &str) {
        self.states += 1;
        let session = self.pool.session();
        let counts = counts_of(&session.allocator, &session.current);
        let payload = data_unit_payload_capacity() as i128;
        let unit = 32768i128;
        let slot = 16384i128;
        let available: Vec<i128> = counts.iter().map(|c| c.formula_available_slots * slot).collect();
        let min_available = *available.iter().min().expect("两块盘");
        let sum_available: i128 = available.iter().sum();
        let df_min_disk_bytes = min_available;
        let df_sum_over_replicas = sum_available / 2;
        let df_sum = sum_available;
        let df_min_whole_units_payload = (min_available.max(0) / unit) * payload;
        let pairs_out = counts.iter().map(|c| c.pairs_outside_segments).min().expect("两块盘") as i128;
        let df_min_units_and_pairs_outside = (min_available.max(0) / unit).min(pairs_out) * payload;
        // D16（发布语义） 已定项 1 的 `df` 字面：没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量，
        // 保留池 = 10 + 7 c_max，残留 = 5 + 6 c_max，c_max 取现算的 ckpt_cost；「已释放的」只算释放代 ≤ max(第 4 新的非空根, 环里最旧有效根)
        // 的（其余是滞后量）；环里最旧有效根取 txg − 3S + 1（S = 8，环里全是有效根）。活元数据两种读法：0，与「全部活着的元数据槽」。
        let current_txg = session.current.root().checkpoint_txg.0;
        let mut non_empty = self.non_empty_txgs.clone();
        non_empty.sort_unstable();
        let fourth_non_empty = if non_empty.len() >= 4 { non_empty[non_empty.len() - 4] } else { 0 };
        let oldest_in_ring = current_txg.saturating_sub(3 * 8 - 1);
        let lag_threshold = fourth_non_empty.max(oldest_in_ring);
        let c_max = i128::from(checkpoint_cost_of_the_version_to_build_on(session.current.file_version(), &session.allocator).0);
        let mut df_d16_zero = i128::MAX;
        let mut df_d16_live = i128::MAX;
        for (index, map) in session.allocator.devices.iter().enumerate() {
            let unused_or_reclaimed = i128::from(map.unit_area_slots()) - i128::from(map.allocated_slots());
            let released_below: i128 = session
                .allocator
                .records()
                .iter()
                .filter(|record| record.device == map.device && record.is_released && record.generation.0 <= lag_threshold)
                .filter(|record| map.is_blocked_for_commit_generated(record.slot) && !map.is_isolated(record.slot))
                .map(|record| i128::from(record.span_slots))
                .sum();
            let live_metadata = i128::from(map.allocated_slots()) - i128::from(map.deferred_slots())
                - 2 * self.file_units as i128;
            let base = unused_or_reclaimed + released_below - (10 + 7 * c_max) - (5 + 6 * c_max);
            let _ = index;
            df_d16_zero = df_d16_zero.min(base * slot);
            df_d16_live = df_d16_live.min((base + live_metadata) * slot);
        }
        let upper = self.record_limit.min(usize::try_from(counts.iter().map(|c| c.pairs_total).min().unwrap_or(0)).unwrap_or(0) + 1);
        let (truth_units, checked) = largest_write_in_units(&self.pool, upper);
        let truth = truth_units as i128 * payload;
        // 做得成的最大那个写的需求拆开：数据单元的槽、普通分配里非数据的槽、固定点的槽，与它推了几串抬 F。
        let (raises, data_slots, ordinary_other_slots, fixed_point_slots) = if truth_units > 0 {
            let mut probe = clone_pool(&self.pool);
            let published = probe.sequential_write(bytes_of_units(truth_units)).expect("真值那个写在拷贝上做得成");
            let roles = &probe.session().current.file_version().expect("带文件").rewritten;
            let mut data = 0u64;
            let mut other = 0u64;
            let mut fixed = 0u64;
            for role in roles {
                match (role, space_budget_of_role(*role)) {
                    (TransactionUnit::Data(_), _) => data += role.span_slots(),
                    (_, SpaceBudgetOfARole::OrdinaryAllocation) => other += role.span_slots(),
                    _ => fixed += role.span_slots(),
                }
            }
            (published.floor_raises.len(), data, other, fixed)
        } else {
            (0, 0, 0, 0)
        };
        // 攻方自己提的一种写法（只在这个模型上量过、被攻过零轮）：df = 最大的 n，使 2n + ⌈n/144⌉ + ⌈n/(144·147)⌉ + 4 + c_max ≤ 各盘可用槽数的最小值，再乘载荷。
        let min_available_slots = min_available / slot;
        let mut h_units: i128 = 0;
        let (mut lo, mut hi) = (0i128, min_available_slots.max(0) / 2 + 1);
        while hi - lo > 1 {
            let n = (lo + hi) / 2;
            let need = 2 * n + (n + 143) / 144 + (n + 144 * 147 - 1) / (144 * 147) + 4 + c_max;
            if need <= min_available_slots { lo = n; } else { hi = n; }
        }
        h_units = h_units.max(lo);
        let df_attack_proposal = h_units * payload;
        let verdict = |df: i128| -> String {
            if df > truth {
                format!("over+{}", df - truth)
            } else {
                format!("under-{}", truth - df)
            }
        };
        println!("{}", line_of(&format!("df_state {tag}"), &counts));
        println!(
            "DF {tag} txg={current_txg} truth_units={truth_units} truth_bytes={truth} monotone_check={checked} c_max={c_max} lag_threshold={lag_threshold} \
             | A_min_disk={df_min_disk_bytes}({}) B_sum_div2={df_sum_over_replicas}({}) C_sum={df_sum}({}) D_min_units_payload={df_min_whole_units_payload}({}) \
             E_min_units_pairs_outside={df_min_units_and_pairs_outside}({}) F_d16_live0={df_d16_zero}({}) G_d16_live_all={df_d16_live}({}) H_attack_proposal={df_attack_proposal}({}) \
             | truth_write raises={raises} data_slots={data_slots} ordinary_other_slots={ordinary_other_slots} fixed_point_slots={fixed_point_slots}",
            verdict(df_min_disk_bytes),
            verdict(df_sum_over_replicas),
            verdict(df_sum),
            verdict(df_min_whole_units_payload),
            verdict(df_min_units_and_pairs_outside),
            verdict(df_d16_zero),
            verdict(df_d16_live),
            verdict(df_attack_proposal),
        );
    }
}

fn driven(unit_area_slots: [u64; 2], journal_ring_bytes: u64) -> Driven {
    let pool = start_pool(unit_area_slots, journal_ring_bytes);
    let record_limit = usize::try_from(journal_record_limit_of_one_publish(journal_ring_bytes)).expect("条数");
    Driven { pool, non_empty_txgs: vec![3], file_units: 1, record_limit, states: 0 }
}

fn geometry_from_env() -> ([u64; 2], u64, String) {
    let slots: Vec<u64> = std::env::var("OPUS_DF_UNIT_AREA_SLOTS")
        .ok()
        .map(|text| text.split(',').map(|n| n.parse().expect("槽数")).collect())
        .unwrap_or_else(|| vec![1391, 1391]);
    let ring: u64 = std::env::var("OPUS_DF_JOURNAL_RING_BYTES").ok().map_or(6 << 20, |t| t.parse().expect("字节"));
    let name = format!("ua={}+{} ring={}", slots[0], slots[1], ring);
    ([slots[0], slots[1]], ring, name)
}

/// 历史一：每次顺序写 n 个单元（n 由用户定，取几档），每一步之后评一次，写到被拒为止。
#[test]
fn opus_probe_df_history_repeated_sequential_writes() {
    let (slots, ring, name) = geometry_from_env();
    let sizes: Vec<usize> = std::env::var("OPUS_DF_SIZES")
        .ok()
        .map(|t| t.split(',').map(|n| n.parse().expect("单元数")).collect())
        .unwrap_or_else(|| vec![16, 64, 200]);
    for size in sizes {
        let mut d = driven(slots, ring);
        println!("==== DF history=repeated geometry={name} size={size}");
        d.evaluate(&format!("repeated size={size} step=0"));
        for step in 1..=60 {
            if !d.sequential_write(size) {
                println!("DF repeated size={size} refused_at_step={step}");
                d.evaluate(&format!("repeated size={size} step={step} after_refusal"));
                break;
            }
            d.evaluate(&format!("repeated size={size} step={step}"));
        }
        println!("DF states_evaluated={}", d.states);
    }
}

/// 历史二：大文件 → 截成 1 单元 → 1 单元覆盖写若干次（释放的大文件过 3 个状态才回来），每一步之后评一次。
#[test]
fn opus_probe_df_history_big_then_small() {
    let (slots, ring, name) = geometry_from_env();
    let bigs: Vec<usize> = std::env::var("OPUS_DF_BIG")
        .ok()
        .map(|t| t.split(',').map(|n| n.parse().expect("单元数")).collect())
        .unwrap_or_else(|| vec![100, 200]);
    for big in bigs {
        let mut d = driven(slots, ring);
        println!("==== DF history=big_then_small geometry={name} big={big}");
        let ok = d.sequential_write(big);
        d.evaluate(&format!("big_then_small big={big} after_big ok={ok}"));
        let small_steps: usize = std::env::var("OPUS_DF_SMALL_STEPS").ok().map_or(6, |t| t.parse().expect("次数"));
        for step in 1..=small_steps {
            let ok = d.overwrite_small();
            d.evaluate(&format!("big_then_small big={big} small_step={step} ok={ok}"));
        }
        println!("DF states_evaluated={}", d.states);
    }
}

/// 历史三（A4）：长时间挂着，提交内生块不停换段：1 单元覆盖写与 8 单元顺序写交替 `rounds` 轮，每 `every` 轮评一次；
/// 看这次挂载开过的聚簇段（对用户数据关着）攒了多少空槽对、它们会不会让「数式子的可用」报的比写得进的多。
#[test]
fn opus_probe_df_history_long_mount() {
    let (slots, ring, name) = geometry_from_env();
    let rounds: usize = std::env::var("OPUS_DF_ROUNDS").ok().map_or(300, |t| t.parse().expect("轮"));
    let every: usize = std::env::var("OPUS_DF_EVERY").ok().map_or(50, |t| t.parse().expect("轮"));
    let long_sizes: Vec<usize> = std::env::var("OPUS_DF_LONG_SIZES")
        .ok()
        .map(|t| t.split(',').map(|n| n.parse().expect("单元数")).collect())
        .unwrap_or_else(|| vec![8]);
    let mut d = driven(slots, ring);
    println!("==== DF history=long_mount geometry={name} rounds={rounds} sizes={long_sizes:?}");
    for round in 1..=rounds {
        let ok = if round % 2 == 0 {
            d.sequential_write(long_sizes[(round / 2) % long_sizes.len()])
        } else {
            d.overwrite_small()
        };
        if !ok {
            println!("DF long_mount refused_at_round={round}");
        }
        if round % every == 0 {
            d.evaluate(&format!("long_mount round={round}"));
        }
    }
    println!("DF states_evaluated={}", d.states);
}
