use super::*;

// ───────────────────────────── 开跑前：常量回比与第七节乙类锚点 ─────────────────────────────

/// 第七节乙类锚点（登记第十三节那条 python 命令的输出原样抄来的数）：（f、K、批数、签发加各批扫描读字节、第一批搬迁副本字节）。
const INTENT_ANCHORS: [(LiveSlotFraction, u64, u64, u64, u64); 6] = [
    (LiveSlotFraction::OneQuarter, 4, 2, 655_360, 262_144),
    (LiveSlotFraction::OneQuarter, 8, 1, 524_288, 524_288),
    (LiveSlotFraction::OneQuarter, 16, 1, 524_288, 524_288),
    (LiveSlotFraction::ThreeQuarters, 4, 6, 3_538_944, 262_144),
    (LiveSlotFraction::ThreeQuarters, 8, 3, 2_359_296, 524_288),
    (LiveSlotFraction::ThreeQuarters, 16, 2, 1_835_008, 1_048_576),
];
/// （f、签发那一步扫 R 读到的字节、一条意图的搬迁副本字节之和）。
const FRACTION_ANCHORS: [(LiveSlotFraction, u64, u64); 2] = [
    (LiveSlotFraction::OneQuarter, 262_144, 524_288),
    (LiveSlotFraction::ThreeQuarters, 786_432, 1_572_864),
];
/// （GPU 段数 n、输入缓冲字节、一次调用建的缓冲字节）。
const GPU_ANCHORS: [(u64, u64, u64); 4] = [
    (1, 32_768, 32_804),
    (1024, 33_554_432, 33_574_928),
    (4096, 134_217_728, 134_299_664),
    (16_384, 536_870_912, 537_198_608),
];
/// R 一块盘的字节。
const REGION_BYTES_ANCHOR: u64 = 1_048_576;
/// PCIe 5.0 x16、x8 的峰值（四舍五入到字节 / 秒）。
const PCIE_ANCHORS: [(u32, u32, u64); 2] = [(5, 16, 63_015_384_615), (5, 8, 31_507_692_308)];

/// 常量回比（停机条款第 1 条）与第七节乙类锚点逐条打出来；交回对不上的条数（常量对不上的另记一行 `name=stop_clause`）。
pub(super) fn check_the_constants_and_the_anchors(emitter: &mut Emitter) -> usize {
    let disagreeing = constants_disagreeing_with_the_crates();
    for (name, local_value, crates_value) in &disagreeing {
        emitter.line(&format!(
            "name=stop_clause clause=1 constant={name} local={local_value} crates={crates_value}"
        ));
    }
    if cfg!(feature = "gpu") {
        emitter.line("name=constant_check item=gpu_buffer_bytes status=compared");
    } else {
        emitter.line("name=constant_check item=gpu_buffer_bytes status=not_run reason=gpu_feature_not_compiled");
    }
    let mut mismatches = disagreeing.len();
    let mut anchor = |item: String, value: u64, expected: u64| {
        let holds = value == expected;
        if !holds {
            mismatches += 1;
        }
        emitter.line(&format!(
            "name=anchor item={item} value={value} expected={expected} holds={holds}"
        ));
    };
    anchor(
        "region_bytes_per_device".to_string(),
        region_bytes_per_device(),
        REGION_BYTES_ANCHOR,
    );
    for (fraction, issue_bytes, intent_copy) in FRACTION_ANCHORS {
        let data_units = fraction.data_units_per_segment();
        anchor(
            format!("issue_scan_bytes_fraction_{}", fraction.label()),
            issue_scan_bytes(data_units),
            issue_bytes,
        );
        anchor(
            format!("intent_copy_bytes_fraction_{}", fraction.label()),
            intent_copy_bytes(data_units),
            intent_copy,
        );
    }
    for (fraction, batch_units, batches, scan_bytes, first_copy) in INTENT_ANCHORS {
        let data_units = fraction.data_units_per_segment();
        let label = format!("fraction_{}_batch_units_{batch_units}", fraction.label());
        anchor(
            format!("batches_{label}"),
            batches_of_one_intent(data_units, batch_units),
            batches,
        );
        anchor(
            format!("intent_scan_bytes_{label}"),
            intent_scan_bytes(data_units, batch_units),
            scan_bytes,
        );
        anchor(
            format!("first_batch_copy_bytes_{label}"),
            first_batch_copy_bytes(data_units, batch_units),
            first_copy,
        );
    }
    for (range_count, input_bytes, buffer_bytes) in GPU_ANCHORS {
        anchor(
            format!("gpu_input_bytes_{range_count}_ranges"),
            range_count * LOCAL_GPU_SLOT_BYTES,
            input_bytes,
        );
        anchor(
            format!("gpu_buffer_bytes_{range_count}_ranges"),
            local_gpu_buffer_bytes(range_count),
            buffer_bytes,
        );
    }
    anchor(
        "cpu_crc32c_of_the_check_string".to_string(),
        u64::from(crc32_castagnoli(CHECK_STRING)),
        u64::from(CHECK_STRING_CRC32_CASTAGNOLI),
    );
    for (generation, lanes, peak) in PCIE_ANCHORS {
        let computed = pcie_link_peak_bytes_per_second(generation, lanes).map_or(0.0, f64::round);
        let computed_text = format!("{computed:.0}");
        anchor(
            format!("pcie_generation_{generation}_by_{lanes}_peak_bytes_per_second"),
            computed_text.parse::<u64>().unwrap_or(0),
            peak,
        );
    }
    mismatches
}
