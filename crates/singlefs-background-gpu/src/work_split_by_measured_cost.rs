//! 按每张卡挂载时量出的代价分活（D26（后台整理与放置回收） 已定项 13 第 2 条；用户 2026-10-26 定「全部独显都用，按每张卡挂载时量的代价分」）。
//! 纯函数，输入每张卡一次提交的主机 CPU 时间与一批的条目数，交回每张卡分几条。
//!
//! 分法：每张卡分到的条数与它的代价成反比（份额 ∝ 1 / 代价），同一批里每张卡上「条数 × 一次提交的代价」大致拉平；
//! 整数部分向下取，余下的几条按余数从大到小补，余数相同的给排在前面的卡。条目按卡的次序切成相连的几段（[`entry_ranges_per_gpu`]），
//! 合并时按卡的次序接回，同一组输入每次交回同一种切法。

/// 一次提交在主机上占的 CPU 时间（这条线程的 CPU 钟，不是挂钟），纳秒。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostCpuNanosecondsPerSubmission(pub u64);

/// 量到 0 纳秒（钟的粒度之下）按 1 纳秒算：份额取代价的倒数，0 没有倒数。
const SMALLEST_COST_IN_NANOSECONDS: u64 = 1;

/// 份额的分子：代价的倒数写成 `2^64 / 代价` 的整数，代价不超过 `u64::MAX` 时它至少是 1，各卡的比照样分得开。
const RECIPROCAL_COST_NUMERATOR: u128 = 1 << 64;

/// 每张卡分几条，按卡的次序；加起来等于 `entry_count`。
///
/// # Panics
/// `measured_costs` 为空：参与的卡至少一张（挂载那一刻的校验拒了「GPU 不在」）。
#[must_use]
pub fn entries_per_gpu_by_measured_cost(
    measured_costs: &[HostCpuNanosecondsPerSubmission],
    entry_count: usize,
) -> Vec<usize> {
    assert!(
        !measured_costs.is_empty(),
        "参与的卡至少一张：挂载那一刻一张独显都没有就拒了"
    );
    let reciprocal_weights: Vec<u128> = measured_costs
        .iter()
        .map(|cost| {
            RECIPROCAL_COST_NUMERATOR / u128::from(cost.0.max(SMALLEST_COST_IN_NANOSECONDS))
        })
        .collect();
    let total_weight: u128 = reciprocal_weights.iter().sum();
    let entry_count_wide = u128::try_from(entry_count).expect("usize 装得进 u128");
    let mut entries_per_gpu = Vec::with_capacity(reciprocal_weights.len());
    let mut remainders = Vec::with_capacity(reciprocal_weights.len());
    for weight in &reciprocal_weights {
        let scaled = entry_count_wide
            .checked_mul(*weight)
            .expect("条目数不超过 2^64、每张卡的权不超过 2^64，乘积装得进 u128");
        entries_per_gpu
            .push(usize::try_from(scaled / total_weight).expect("一张卡分到的条数不超过总条数"));
        remainders.push(scaled % total_weight);
    }
    let assigned: usize = entries_per_gpu.iter().sum();
    let leftover = entry_count - assigned;
    assert!(
        leftover < measured_costs.len(),
        "向下取整每张卡少给不到一条，补的条数少于卡数"
    );
    let mut gpus_by_remainder: Vec<usize> = (0..measured_costs.len()).collect();
    gpus_by_remainder.sort_by(|left, right| {
        remainders[*right]
            .cmp(&remainders[*left])
            .then(left.cmp(right))
    });
    for gpu_position in gpus_by_remainder.into_iter().take(leftover) {
        entries_per_gpu[gpu_position] += 1;
    }
    assert_eq!(
        entries_per_gpu.iter().sum::<usize>(),
        entry_count,
        "每张卡分到的条数加起来等于这一批的条目数"
    );
    entries_per_gpu
}

/// 把每张卡分到的条数按卡的次序切成相连的几段：排在最前的卡拿最前面那几条，依次往后。
#[must_use]
pub fn entry_ranges_per_gpu(entries_per_gpu: &[usize]) -> Vec<std::ops::Range<usize>> {
    let mut start = 0;
    entries_per_gpu
        .iter()
        .map(|entry_count| {
            let range = start..start + entry_count;
            start = range.end;
            range
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn costs(nanoseconds: &[u64]) -> Vec<HostCpuNanosecondsPerSubmission> {
        nanoseconds
            .iter()
            .map(|cost| HostCpuNanosecondsPerSubmission(*cost))
            .collect()
    }

    #[test]
    fn a_single_gpu_takes_every_entry() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[12_000]), 37),
            vec![37]
        );
    }

    #[test]
    fn gpus_with_the_same_cost_split_evenly_and_the_earlier_gpu_takes_the_leftover() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[5_000, 5_000, 5_000]), 10),
            vec![4, 3, 3],
            "代价相同平分，除不尽的那一条给排在前面的卡"
        );
    }

    #[test]
    fn a_gpu_three_times_as_costly_takes_a_third_as_many_entries() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[1_000, 3_000]), 8),
            vec![6, 2],
            "份额与代价成反比"
        );
    }

    #[test]
    fn a_vastly_cheaper_gpu_takes_nearly_every_entry() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[1_000_000, 1_000]), 100),
            vec![0, 100],
            "代价差一千倍：贵的那张一条都分不到整数份"
        );
    }

    #[test]
    fn fewer_entries_than_gpus_go_to_the_cheapest_gpus_first() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[4_000, 1_000, 2_000]), 2),
            vec![0, 1, 1],
            "两条三张卡：给最便宜的两张"
        );
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[7_000, 7_000, 7_000]), 2),
            vec![1, 1, 0],
            "代价相同：给排在前面的两张"
        );
    }

    #[test]
    fn an_empty_batch_gives_every_gpu_nothing() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[1, 2]), 0),
            vec![0, 0]
        );
    }

    #[test]
    fn a_cost_measured_as_zero_counts_as_the_smallest_cost() {
        assert_eq!(
            entries_per_gpu_by_measured_cost(&costs(&[0, 1]), 4),
            vec![2, 2],
            "0 纳秒按 1 纳秒算"
        );
    }

    #[test]
    fn the_shares_always_add_up_to_the_entry_count() {
        let cost_sets = [
            costs(&[1]),
            costs(&[3, 7]),
            costs(&[u64::MAX, 1, 999]),
            costs(&[10, 10, 10, 10, 11]),
        ];
        for cost_set in &cost_sets {
            for entry_count in [0_usize, 1, 2, 5, 97, 4096] {
                let shares = entries_per_gpu_by_measured_cost(cost_set, entry_count);
                assert_eq!(shares.len(), cost_set.len(), "每张卡一个份额");
                assert_eq!(
                    shares.iter().sum::<usize>(),
                    entry_count,
                    "代价 {cost_set:?}、{entry_count} 条：份额加起来等于条目数"
                );
            }
        }
    }

    #[test]
    fn entry_ranges_follow_the_gpu_order_and_cover_the_batch_contiguously() {
        assert_eq!(entry_ranges_per_gpu(&[2, 0, 3]), vec![0..2, 2..2, 2..5]);
    }
}
