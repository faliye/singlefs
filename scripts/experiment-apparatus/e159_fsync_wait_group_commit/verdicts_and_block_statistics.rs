//! 分位数、比值与五轮的判定、趋势标签、Mann-Whitney 判别统计、来宾块层读数与单盘字节数。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RatioVerdict {
    CrossedUp,
    CrossedDown,
    NotCrossed,
}
/// J1 / J4 的三档（第六节）：`ratio >= threshold` 越线，`ratio <= 1/threshold` 反越线，其余未越线。
/// M11 的变异目标：把 `>=` 改成 `>`，边界 `ratio == threshold` 的单测就会翻。
pub(super) fn classify_ratio(ratio: f64, threshold: f64) -> RatioVerdict {
    if ratio >= threshold {
        RatioVerdict::CrossedUp
    } else if ratio <= 1.0 / threshold {
        RatioVerdict::CrossedDown
    } else {
        RatioVerdict::NotCrossed
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FiveRoundVerdict {
    Consistent(RatioVerdict),
    Unstable,
}
/// 五轮怎么合（第六节）：五轮判同一档才报那一档，不一致记「不稳定」。
/// M12 的变异目标：改成「多数轮说了算」，四胜一的单测就会翻。
pub(super) fn merge_five_rounds(verdicts: &[RatioVerdict; 5]) -> FiveRoundVerdict {
    if verdicts.iter().all(|&verdict| verdict == verdicts[0]) {
        FiveRoundVerdict::Consistent(verdicts[0])
    } else {
        FiveRoundVerdict::Unstable
    }
}

/// 规则 R（第六节）：给一串按轴排好的值打走势标签，看最后 3 步的 δ。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TrendLabel {
    ExpandingAlongAxis,
    ShrinkingAlongAxis,
    Converging,
    Other,
}
pub(super) fn trend_rule_label(values_along_axis: &[f64]) -> TrendLabel {
    assert!(values_along_axis.len() >= 2, "规则 R 至少要两个点才算得出一步 δ");
    let deltas: Vec<f64> = values_along_axis.windows(2).map(|pair| pair[1] / pair[0] - 1.0).collect();
    let last_three = &deltas[deltas.len().saturating_sub(3)..];
    if last_three.len() == 3 && last_three.iter().all(|&delta| delta >= 0.05) {
        TrendLabel::ExpandingAlongAxis
    } else if last_three.len() == 3 && last_three.iter().all(|&delta| delta <= -0.05) {
        TrendLabel::ShrinkingAlongAxis
    } else if deltas.last().map(|&delta| delta.abs() < 0.05).unwrap_or(false) {
        TrendLabel::Converging
    } else {
        TrendLabel::Other
    }
}

/// Mann–Whitney U 归一化成 `A = P(甲 > WAL) + ½P(相等)`（第五节 5.4 PC3；精确算法，不用直方图近似）。
pub(super) fn mann_whitney_discrimination_statistic(first_sample: &[u64], second_sample: &[u64]) -> f64 {
    assert!(!first_sample.is_empty() && !second_sample.is_empty(), "PC3 的两组样本都不能是空的");
    let mut greater = 0u64;
    let mut equal = 0u64;
    for &value_from_first_sample in first_sample {
        for &value_from_second_sample in second_sample {
            match value_from_first_sample.cmp(&value_from_second_sample) {
                std::cmp::Ordering::Greater => greater += 1,
                std::cmp::Ordering::Equal => equal += 1,
                std::cmp::Ordering::Less => {}
            }
        }
    }
    let total = (first_sample.len() * second_sample.len()) as f64;
    (greater as f64 + 0.5 * equal as f64) / total
}

// ============================================================
// 来宾块层读数（第五节 5.6）、结果行、CLI、main
// ============================================================

/// `/sys/block/<设备>/stat` 的字段 2（写完成扇区，第 7 列，0-based 索引 6）与字段 0（读请求数）。
/// 字段顺序见 Linux `Documentation/ABI/testing/procfs-diskstats`：
/// reads_completed reads_merged sectors_read ms_reading writes_completed writes_merged
/// sectors_written ms_writing ...
pub(super) struct BlockLayerStatistics {
    pub(super) reads_completed: u64,
    pub(super) sectors_read: u64,
    pub(super) writes_completed: u64,
    pub(super) sectors_written: u64,
}
pub(super) fn read_block_layer_statistics(device_name: &str) -> std::io::Result<BlockLayerStatistics> {
    let text = std::fs::read_to_string(format!("/sys/block/{device_name}/stat"))?;
    let fields: Vec<u64> = text.split_whitespace().map(|token| token.parse().unwrap_or(0)).collect();
    Ok(BlockLayerStatistics {
        reads_completed: *fields.first().unwrap_or(&0),
        sectors_read: *fields.get(2).unwrap_or(&0),
        writes_completed: *fields.get(4).unwrap_or(&0),
        sectors_written: *fields.get(6).unwrap_or(&0),
    })
}

pub(super) fn device_name_from_path(device_path: &str) -> String {
    device_path.rsplit('/').next().unwrap_or(device_path).to_string()
}

/// 甲一批的单盘总字节（跨 `print_anchors` 与单测共用）。`JiaPublishOutcome::root_slot_bytes`
/// 本来就是单盘值（第三节 2.3：「根槽只在一块盘」），不能像别的字段一样再除以 `DEVICE_COUNT`——
/// `outcome.total_bytes() / DEVICE_COUNT` 这个近似对根槽字段是错的（会把它也腰斩），B4 钉的
/// 172544 就是这个函数的口径，不是那个近似的口径。
pub(super) fn jia_outcome_single_device_bytes(outcome: &JiaPublishOutcome) -> u64 {
    let both_device_total = outcome.data_bytes + outcome.extent_bytes + outcome.inode_container_bytes + outcome.inode_root_bytes + outcome.allocation_bytes + outcome.accounting_bytes + outcome.mapping_bytes + outcome.tree_table_bytes + outcome.record_bytes;
    both_device_total / DEVICE_COUNT + outcome.root_slot_bytes + outcome.system_configuration_bytes / DEVICE_COUNT
}
