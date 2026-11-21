// ───────────────────────────── GPU 一侧的算术（第三段；纯函数，不编 gpu 特性也有单测） ─────────────────────────────

/// PCIe 链路一个方向的峰值（字节 / 秒）：每条 lane 的传输率 × lane 数 × 编码效率 ÷ 8。1、2 代 8b/10b，3 到 5 代 128b/130b；别的代数交回 `None`
/// （读不到，那张卡作废，登记第十一节「读不到」）。5 代 x16 是 63015384615.38…，x8 是 31507692307.69…（登记第七节乙类按四舍五入写）。
pub(super) fn pcie_link_peak_bytes_per_second(generation: u32, lanes: u32) -> Option<f64> {
    let (giga_transfers_per_lane, payload_bits, line_bits) = match generation {
        1 => (2.5, 8.0, 10.0),
        2 => (5.0, 8.0, 10.0),
        3 => (8.0, 128.0, 130.0),
        4 => (16.0, 128.0, 130.0),
        5 => (32.0, 128.0, 130.0),
        _ => return None,
    };
    Some(giga_transfers_per_lane * 1e9 * f64::from(lanes) * payload_bits / line_bits / 8.0)
}

/// 一次调用没快过物理下限：`t ≥ 输入字节 ÷ 链路峰值`（快过就是没等到提交做完，那张卡作废）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn respects_the_physical_floor(
    seconds: f64,
    input_bytes: u64,
    link_peak_bytes_per_second: f64,
) -> bool {
    seconds >= input_bytes as f64 / link_peak_bytes_per_second
}

/// 最小二乘拟合 `t = a + b·B`：交回 (a 截距秒, b 斜率秒 / 字节)；少于两个不同的 B 交回 `None`。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn least_squares_intercept_and_slope(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let count = points.len() as f64;
    let mean_bytes = points.iter().map(|(bytes, _seconds)| bytes).sum::<f64>() / count;
    let mean_seconds = points.iter().map(|(_bytes, seconds)| seconds).sum::<f64>() / count;
    let covariance: f64 = points
        .iter()
        .map(|(bytes, seconds)| (bytes - mean_bytes) * (seconds - mean_seconds))
        .sum();
    let variance: f64 = points
        .iter()
        .map(|(bytes, _seconds)| (bytes - mean_bytes).powi(2))
        .sum();
    if points.len() < 2 || variance == 0.0 {
        return None;
    }
    let slope = covariance / variance;
    let intercept = mean_seconds - slope * mean_bytes;
    Some((intercept, slope))
}

/// Q13 的 φ = a ÷ (a + b·B_批)，B_批 按字节代。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn fixed_overhead_share(
    intercept_seconds: f64,
    slope_seconds_per_byte: f64,
    batch_bytes: u64,
) -> f64 {
    let batch_bytes_wide = batch_bytes as f64;
    intercept_seconds / (intercept_seconds + slope_seconds_per_byte * batch_bytes_wide)
}

/// Q13：φ > 10% 时要合几批 n* = ⌈9a ÷ (b·B_批)⌉（合 n* 批之后 φ ≤ 10%）。交回浮点数，打印时按整数写。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn batches_to_merge(
    intercept_seconds: f64,
    slope_seconds_per_byte: f64,
    batch_bytes: u64,
) -> f64 {
    (9.0 * intercept_seconds / (slope_seconds_per_byte * batch_bytes as f64)).ceil()
}

/// GPU 交回的 CRC 与 CPU 逐段相同（有一段不同整张卡作废，登记第五节「GPU 一侧怎么量」）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "第三段（gpu 特性）才用；没编 gpu 特性时只在单测里用"
    )
)]
pub(super) fn gpu_crcs_agree_with_the_cpu(gpu_results: &[u32], cpu_results: &[u32]) -> bool {
    gpu_results.len() == cpu_results.len()
        && gpu_results
            .iter()
            .zip(cpu_results)
            .all(|(gpu_value, cpu_value)| gpu_value == cpu_value)
}

/// 公开校验值：CRC-32C("123456789") = 0xE3069283。
pub(super) const CHECK_STRING: &[u8] = b"123456789";
pub(super) const CHECK_STRING_CRC32_CASTAGNOLI: u32 = 0xE306_9283;

#[cfg(test)]
mod tests;
