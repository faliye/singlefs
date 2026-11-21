use super::super::*;
use super::*;

#[test]
fn cpu_crc32c_of_the_check_string_is_e3069283() {
    assert_eq!(crc32_castagnoli(CHECK_STRING), 0xE306_9283);
    assert_eq!(CHECK_STRING_CRC32_CASTAGNOLI, 0xE306_9283);
}

#[test]
fn pcie_peaks_of_generation_five_by_sixteen_and_by_eight() {
    let by_sixteen = pcie_link_peak_bytes_per_second(5, 16).expect("5 代");
    let by_eight = pcie_link_peak_bytes_per_second(5, 8).expect("5 代");
    assert!(
        (by_sixteen - 63_015_384_615.38).abs() < 1.0,
        "5 代 x16：{by_sixteen}"
    );
    assert!(
        (by_eight.round() - 31_507_692_308.0).abs() < 0.5,
        "5 代 x8：{by_eight}"
    );
    assert!(
        pcie_link_peak_bytes_per_second(7, 16).is_none(),
        "认不得的代数读不到"
    );
}

#[test]
fn submission_faster_than_the_link_allows_breaks_the_physical_floor() {
    let peak = pcie_link_peak_bytes_per_second(5, 16).expect("5 代");
    let bytes = 128 << 20;
    assert!(
        !respects_the_physical_floor(bytes as f64 / (2.0 * peak), bytes, peak),
        "快过下限一倍"
    );
    assert!(
        respects_the_physical_floor(bytes as f64 / peak * 1.01, bytes, peak),
        "比下限慢一点"
    );
}

#[test]
fn least_squares_recovers_a_one_millisecond_intercept_and_a_one_nanosecond_slope() {
    let points: Vec<(f64, f64)> = [32_768_u64, 131_072, 1_048_576, 8_388_608]
        .iter()
        .map(|bytes| (*bytes as f64, 1e-3 + 1e-9 * *bytes as f64))
        .collect();
    let (intercept, slope) = least_squares_intercept_and_slope(&points).expect("四个点");
    assert!((intercept - 1e-3).abs() < 1e-5, "截距 {intercept}");
    assert!((slope - 1e-9).abs() < 1e-11, "斜率 {slope}");
    assert!(
        least_squares_intercept_and_slope(&points[..1]).is_none(),
        "一个点拟不出"
    );
}

#[test]
fn fixed_overhead_share_of_one_mebibyte_is_about_0_488() {
    let share = fixed_overhead_share(1e-3, 1e-9, 1_048_576);
    assert!((share - 0.488_14).abs() < 1e-4, "φ = {share}");
}

#[test]
fn batches_to_merge_round_nine_intercepts_over_the_batch_time_up() {
    let merge = batches_to_merge(1e-3, 1e-9, 1_048_576);
    assert!(
        (merge - 9.0).abs() < 1e-9,
        "⌈9 × 1 ms ÷ 1.048576 ms⌉ = 9，拿到 {merge}"
    );
    assert!(
        fixed_overhead_share(1e-3, 1e-9, 1_048_576 * 9) <= FIXED_SHARE_AT_MOST_FOR_ONE_BATCH,
        "合 9 批之后 φ ≤ 10%"
    );
}

#[test]
fn gpu_results_that_differ_in_one_range_are_refused() {
    let cpu = vec![1_u32, 2, 3, 4];
    assert!(gpu_crcs_agree_with_the_cpu(&cpu, &cpu));
    let mut gpu = cpu.clone();
    gpu[2] ^= 1;
    assert!(
        !gpu_crcs_agree_with_the_cpu(&gpu, &cpu),
        "换掉一段回读值，对拍要红"
    );
    assert!(!gpu_crcs_agree_with_the_cpu(&cpu[..3], &cpu), "少一段也红");
}
