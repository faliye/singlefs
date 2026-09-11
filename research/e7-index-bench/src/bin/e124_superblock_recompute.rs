//! # E124 超级块字段表按 D18 已定项 14 重算
//!
//! 答 D22 未定项 9 那条 2026-09-08 复核留下的账。该段在「D18 十四条全部定案」之后逐字写：
//! 「**已定项 14（nonce 代号就是完整的 96 位 nonce，住单元明文头）改了挡路条 ② 那一格的输入**
//! ——E115 给 D9 那三项取的是最小可用假设值，要按它复算。」
//!
//! 外加一条 E115 没问过的：它必需项表里「设备数 `devs`」的出处逐字是
//! 「D22 已定项 2「P > devs」；设备表移出槽后槽里没有条目数」——
//! 而设备表单元若取码 3 打包记录容器，容器头里本来就有记录数。
//!
//! ## 被引用条款（逐字）
//!
//! - D18 已定项 14：「**取臂甲**：nonce 代号就是完整的 96 位 nonce（12 字节），住单元明文头，指针里那份保留」
//! - D9 已定项 8：「nonce 水位**不许住在裸明文超级块里**，必须落在一个被主密钥派生 MAC 覆盖的字段上」
//! - I-6.5：「记录在案的 nonce 水位 > 全盘出现过的最大 nonce（崩溃后不重用的可判定形式）」
//! - D18 已定项 11 码 3 字段表：容器头偏移 69 那一行逐字 `| 69 | 记录数 | 2 | ≤ 583 |`
//! - D22 已定项 7：树表单元指针 **59** = 指针头部 31 + 位置条目 14 × 2
//! - D22 已定项 2：R = 3；「区域 r 落在 `r × P × chunk`，P 素数且 `P > devs`」
//! - D20 推论三：撕裂判定宽度 = `physical_block_size` ⇒ 一个槽就是一个扇区
//! - E115 产物首行逐字：`name=completeness required=31 candidate_rows=24 missing=9`；
//!   末行逐字：`name=e100_baseline candidate_bytes=361 plus_devtable_ptr=420`
//!
//! ## 它答不了的
//!
//! 纯算术。不答「这些字段各自该多宽」，不答完备性闸自己会不会漏（C178），
//! 也不答设备表单元该取哪个码——判据 2 只把两条臂的字节各算一遍。

use e7_index_bench::Emitter;

/// D20 推论三：槽宽 = 探测到的 `physical_block_size`。
const SLOT_WIDTHS: [u64; 2] = [512, 4096];

/// D22 已定项 7 的树表单元指针宽度。
const POINTER_BYTES: u64 = 59;

/// D22 已定项 2：根环区域数 R = 3。
const RING_REGIONS: u64 = 3;

/// D18 已定项 7 的三档单元头。
const UNIT_HEADERS: [u64; 3] = [68, 77, 86];

/// D18 已定项 14 逐字：完整的 96 位 nonce。
const NONCE_BITS: u64 = 96;

/// E100 / E115 两套装置里 `nonce_watermark` 用的宽度（**被测对象**，逐字照抄）。
const E100_WATERMARK_BYTES: u64 = 8;

/// D18 已定项 11 码 3 打包记录容器头里那一行：偏移 69，宽 2。
const CODE3_RECORD_COUNT_BYTES: u64 = 2;

/// E100 的 24 字段候选表（**被测对象**，逐字照抄自 `e100_superblock_slot.rs`），
/// 但 `nonce_watermark` 那一行**不在这里**——它是判据 1 的被测量，单独作参数传进来。
/// 这样 353 是**算出来的**，不是写死的：跨装置对账才不是自证。
const CANDIDATE_WITHOUT_WATERMARK_ROWS: [(&str, u64); 23] = [
    // 段一 自举头
    ("magic", 4),
    ("format_version", 2),
    ("feature_bits", 96),
    ("fsid", 16),
    ("this_dev_id", 4),
    ("slot_generation", 8),
    ("unit_checksum", 32),
    ("sb_mac", 16),
    // ("nonce_watermark", ?) —— 判据 1 的被测量，见 `slot_bytes` 的 watermark_bytes 参数
    // 段二 几何
    ("node_bytes", 4),
    ("unit_bytes", 4),
    ("alloc_grain", 4),
    ("posentry_widths", 4),
    ("journal_geom", 24),
    ("ring_R_S", 4),
    ("tree_table_ptr", 59),
    ("map_provenance", 24),
    // 段三 可调值
    ("t_time", 4),
    ("t_dirty", 8),
    ("compact_lo", 8),
    ("compact_hi", 8),
    ("compact_stop", 8),
    // 段四 运行时水位
    ("journal_tail", 8),
    ("journal_instance", 4),
];

/// E115 那九项缺项的宽度（**不含**设备数 devs，它在 E124 里由判据 2 单列）。
/// 逐字照抄自 `e115_superblock_completeness.rs` 的 `MISSING_WIDTHS`，全部是假设值。
const MISSING_ROWS_WITHOUT_DEVICE_COUNT: [(&str, u64, u64); 8] = [
    ("journal 最坏占用与 F", 9, 9),
    ("根环逐区域设备身份", 4, RING_REGIONS * 4),
    ("条带宽度上界 w_max", 1, 1),
    ("组大小 g", 1, 1),
    ("KDF 标识", 2, 2),
    ("加密类型", 2, 2),
    ("MAC 长度声明", 1, 1),
    ("主密钥槽", POINTER_BYTES, 80),
];

/// E115 必需项表里「设备数 devs」取的宽度。
const DEVICE_COUNT_BYTES: u64 = 4;

fn candidate_without_watermark() -> u64 {
    CANDIDATE_WITHOUT_WATERMARK_ROWS.iter().map(|candidate_row| candidate_row.1).sum()
}

fn missing_without_device_count(is_wide_estimate: bool) -> u64 {
    MISSING_ROWS_WITHOUT_DEVICE_COUNT.iter().map(|&(_, narrow_bytes, wide_bytes)| if is_wide_estimate { wide_bytes } else { narrow_bytes }).sum()
}

/// 判据 1：I-6.5 的水位要能给 96 位 nonce 排全序 ⇒ 至少 96 位。
fn required_watermark_bytes() -> u64 {
    NONCE_BITS / 8
}

/// 判据 1 的重算闸：E100 / E115 用的宽度与 D18 已定项 14 一致吗？
/// 返回 (是否一致, 差几字节)。
fn watermark_gate(used_watermark_bytes: u64) -> (bool, u64) {
    let required_bytes = required_watermark_bytes();
    (used_watermark_bytes == required_bytes, required_bytes.saturating_sub(used_watermark_bytes))
}

/// 设备表单元取哪个码，决定槽里的 `devs` 省不省得掉。
#[derive(Clone, Copy, PartialEq, Eq)]
enum DeviceTableUnit {
    /// 码 3 打包记录容器：头里偏移 69 有记录数 ⇒ 槽里那 4 字节可省。
    Code3Packed,
    /// 码 2 索引节点类：类身份段里没有记录数 ⇒ 省不掉。
    Code2Index,
}

impl DeviceTableUnit {
    fn carries_record_count(self) -> bool {
        match self {
            DeviceTableUnit::Code3Packed => true,
            DeviceTableUnit::Code2Index => false,
        }
    }
    fn name(self) -> &'static str {
        match self {
            DeviceTableUnit::Code3Packed => "code3_packed",
            DeviceTableUnit::Code2Index => "code2_index",
        }
    }
    /// 槽里 `devs` 那一项要付几个字节。
    fn device_count_bytes_in_slot(self) -> u64 {
        if self.carries_record_count() { 0 } else { DEVICE_COUNT_BYTES }
    }
}

/// 槽里一共几个间接指针：设备表 1 + 树表 1 + 主密钥槽（走间接层才算）+ 中央映射（`map_sep` 才算）。
fn pointer_count(key_slot_indirect: bool, map_sep: bool) -> u64 {
    1 + 1 + u64::from(key_slot_indirect) + u64::from(map_sep)
}

/// 槽的总字节。
/// - `wide`：缺项取宽口径（含主密钥槽内联 80）
/// - `map_sep`：中央映射树根单占一个指针
/// - `merge_pointers`：全部间接指针合成一个「间接目录单元」，槽里只留 1 个
/// - `watermark_bytes`：nonce 水位那一行的宽度（判据 1 的被测量）
/// - `device_table_unit`：设备表单元取哪个码（判据 2 的被测量）
fn slot_bytes(
    wide: bool,
    map_sep: bool,
    merge_pointers: bool,
    watermark_bytes: u64,
    device_table_unit: DeviceTableUnit,
) -> u64 {
    let missing_bytes = missing_without_device_count(wide);
    let base_bytes = candidate_without_watermark()
        + watermark_bytes
        + POINTER_BYTES
        + missing_bytes
        + device_table_unit.device_count_bytes_in_slot();
    let key_slot_indirect = !wide;
    let indirect_pointer_count = pointer_count(key_slot_indirect, map_sep);
    let mut slot_total_bytes = base_bytes;
    if map_sep {
        slot_total_bytes += POINTER_BYTES;
    }
    if merge_pointers {
        slot_total_bytes -= (indirect_pointer_count - 1) * POINTER_BYTES;
    }
    slot_total_bytes
}

/// E100 判据 4 同式：跨 n 个扇区时有几种可读但不一致的状态。
fn torn_states(total_bytes: u64, slot_width_bytes: u64) -> u64 {
    let sectors = total_bytes.div_ceil(slot_width_bytes);
    if sectors <= 1 {
        return 0;
    }
    (1u64 << sectors).saturating_sub(2)
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_lines: Vec<String> = Vec::new();

    // 判据 1：nonce 水位重算闸
    let required_bytes = required_watermark_bytes();
    let (is_consistent, short_by_bytes) = watermark_gate(E100_WATERMARK_BYTES);
    output_lines.push(emitter.emit_raw(&format!(
        "name=watermark_gate decided_nonce_bits={NONCE_BITS} required_bytes={required_bytes} \
         e100_used={E100_WATERMARK_BYTES} consistent={} short_by={short_by_bytes}",
        u8::from(is_consistent)
    )));
    // 阳性对照的另一侧：按正确宽度喂进去必须判一致
    let (control_is_consistent, control_short_by_bytes) = watermark_gate(required_bytes);
    output_lines.push(emitter.emit_raw(&format!(
        "name=watermark_gate_control fed={required_bytes} consistent={} short_by={control_short_by_bytes}",
        u8::from(control_is_consistent)
    )));

    // 判据 1 + 2 + 3：全格重算
    for &device_table_unit in [DeviceTableUnit::Code3Packed, DeviceTableUnit::Code2Index].iter() {
        for &wide in [false, true].iter() {
            for &map_sep in [false, true].iter() {
                for &merge_pointers in [false, true].iter() {
                    let total_bytes = slot_bytes(wide, map_sep, merge_pointers, required_bytes, device_table_unit);
                    let total_bytes_with_e100_watermark = slot_bytes(wide, map_sep, merge_pointers, E100_WATERMARK_BYTES, device_table_unit);
                    for &slot_width_bytes in SLOT_WIDTHS.iter() {
                        output_lines.push(emitter.emit_raw(&format!(
                            "name=budget dev_unit={} wide={} map_sep={} merge_ptrs={} \
                             total={total_bytes} was={total_bytes_with_e100_watermark} slot={slot_width_bytes} fits={} over_by={} torn_states={}",
                            device_table_unit.name(),
                            u8::from(wide),
                            u8::from(map_sep),
                            u8::from(merge_pointers),
                            u8::from(total_bytes <= slot_width_bytes),
                            total_bytes.saturating_sub(slot_width_bytes),
                            torn_states(total_bytes, slot_width_bytes)
                        )));
                    }
                }
            }
        }
    }

    // 判据 2 的差额：两条臂在同一格上差几字节
    for &wide in [false, true].iter() {
        let code3_packed_bytes = slot_bytes(wide, true, true, required_bytes, DeviceTableUnit::Code3Packed);
        let code2_index_bytes = slot_bytes(wide, true, true, required_bytes, DeviceTableUnit::Code2Index);
        output_lines.push(emitter.emit_raw(&format!(
            "name=devs_arm wide={} code3={code3_packed_bytes} code2={code2_index_bytes} saved={} record_count_bytes={CODE3_RECORD_COUNT_BYTES}",
            u8::from(wide),
            code2_index_bytes - code3_packed_bytes
        )));
    }

    // 判据 3 的另一半：合并臂新增那一级间接要付的单元头
    for &unit_header_bytes in UNIT_HEADERS.iter() {
        output_lines.push(emitter.emit_raw(&format!(
            "name=merge_cost unit_header={unit_header_bytes} saved_in_slot={} paid_in_unit={unit_header_bytes}",
            3 * POINTER_BYTES
        )));
    }

    // 判据 4：跨装置对账——本实验必须复算出 E115 / E100 的 361 与 420
    output_lines.push(emitter.emit_raw(&format!(
        "name=e115_reconcile rows={} candidate_bytes={} plus_devtable_ptr={} \
         missing_narrow={} missing_wide={}",
        CANDIDATE_WITHOUT_WATERMARK_ROWS.len() + 1,
        candidate_without_watermark() + E100_WATERMARK_BYTES,
        candidate_without_watermark() + E100_WATERMARK_BYTES + POINTER_BYTES,
        missing_without_device_count(false) + DEVICE_COUNT_BYTES,
        missing_without_device_count(true) + DEVICE_COUNT_BYTES
    )));

    for output_line in &output_lines {
        println!("{output_line}");
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **跨装置对账**：本实验拆开的候选表必须和 E115 / E100 算出同一个 361 / 420。
    /// `show-me-test.md`：「两套装置算同一个量，就要有一条检查逼它们落到同一个数」。
    #[test]
    fn reconciles_with_e115_and_e100() {
        // 23 行 + 抽出去的 nonce 水位 = E100 / E115 的 24 行
        assert_eq!(CANDIDATE_WITHOUT_WATERMARK_ROWS.len() + 1, 24, "候选表 24 行");
        assert_eq!(candidate_without_watermark(), 353);
        assert_eq!(candidate_without_watermark() + E100_WATERMARK_BYTES, 361, "E100 四段固定开销");
        assert_eq!(
            candidate_without_watermark() + E100_WATERMARK_BYTES + POINTER_BYTES,
            420,
            "E100 判据 3 的出路臂恒 420"
        );
    }

    /// E115 的两档缺项字节，扣掉 devs 之后必须还原得回去。
    #[test]
    fn missing_bytes_reconcile_with_e115() {
        assert_eq!(MISSING_ROWS_WITHOUT_DEVICE_COUNT.len() + 1, 9, "E115 缺 9 项");
        assert_eq!(missing_without_device_count(false) + DEVICE_COUNT_BYTES, 83, "E115 窄档 83");
        assert_eq!(missing_without_device_count(true) + DEVICE_COUNT_BYTES, 112, "E115 宽档 112");
    }

    #[test]
    fn format_constants_match_knowledge_base() {
        assert_eq!(SLOT_WIDTHS, [512, 4096]);
        assert_eq!(POINTER_BYTES, 59, "D22 已定项 7 的树表单元指针");
        assert_eq!(RING_REGIONS, 3, "D22 已定项 2：R = 3");
        assert_eq!(UNIT_HEADERS, [68, 77, 86], "D18 已定项 7 三档头");
        assert_eq!(NONCE_BITS, 96, "D18 已定项 14 逐字：完整的 96 位 nonce");
        assert_eq!(CODE3_RECORD_COUNT_BYTES, 2, "D18 已定项 11 码 3 头偏移 69 那一行");
    }

    /// **判据 1 的绝对值**：水位要 12 字节，E100 用的 8 少 4。
    #[test]
    fn criterion1_watermark_is_absolute() {
        assert_eq!(required_watermark_bytes(), 12);
        assert_eq!(required_watermark_bytes(), E100_WATERMARK_BYTES + 4);
        let (is_consistent, short_by_bytes) = watermark_gate(E100_WATERMARK_BYTES);
        assert!(!is_consistent, "8 字节水位与 D18 已定项 14 不一致");
        assert_eq!(short_by_bytes, 4);
    }

    /// **阳性对照**：闸喂正确宽度时必须判一致、差 0；喂 8 必须判不一致、差 4。
    /// 两侧都要跑，否则分不清闸是「会判」还是「恒判红」。
    #[test]
    fn criterion1_positive_control_gate_has_both_verdicts() {
        assert_eq!(watermark_gate(12), (true, 0));
        assert_eq!(watermark_gate(8), (false, 4));
        assert_eq!(watermark_gate(4), (false, 8));
    }

    /// **判据 2 的绝对值**：两条臂在同一格上的槽字节各钉死，不许只比差值。
    #[test]
    fn criterion2_device_count_arms_are_absolute() {
        // 窄档 + 中央映射单占指针 + 合并：353 + 12 + 59 + 79 + 0 = 503 - 118 = 385 + 4 - 4
        assert_eq!(slot_bytes(false, true, true, 12, DeviceTableUnit::Code3Packed), 385);
        assert_eq!(slot_bytes(false, true, true, 12, DeviceTableUnit::Code2Index), 389);
        assert_eq!(slot_bytes(true, true, true, 12, DeviceTableUnit::Code3Packed), 473);
        assert_eq!(slot_bytes(true, true, true, 12, DeviceTableUnit::Code2Index), 477);
        // 两条臂只差 devs 那 4 字节
        assert_eq!(
            slot_bytes(false, true, true, 12, DeviceTableUnit::Code2Index)
                - slot_bytes(false, true, true, 12, DeviceTableUnit::Code3Packed),
            DEVICE_COUNT_BYTES
        );
    }

    /// 码 3 头里有记录数、码 2 头里没有——两条臂的判据本身要钉住。
    #[test]
    fn criterion2_arm_predicate_is_pinned() {
        assert!(DeviceTableUnit::Code3Packed.carries_record_count());
        assert!(!DeviceTableUnit::Code2Index.carries_record_count());
        assert_eq!(DeviceTableUnit::Code3Packed.device_count_bytes_in_slot(), 0);
        assert_eq!(DeviceTableUnit::Code2Index.device_count_bytes_in_slot(), DEVICE_COUNT_BYTES);
    }

    /// **判据 1 的下游绝对值**：水位从 8 抬到 12 之后，512 档上原本那三格爆得更多。
    /// E115 逐字：爆 20 / 50 / 79。抬 4 字节之后是 24 / 54 / 83。
    #[test]
    fn criterion1_downstream_budget_on_512_is_absolute() {
        let code2_index_unit = DeviceTableUnit::Code2Index; // 与 E115 同口径（槽里带 devs）
        assert_eq!(slot_bytes(false, false, false, 12, code2_index_unit), 507, "E115 的 503 + 4");
        assert!(slot_bytes(false, false, false, 12, code2_index_unit) <= 512, "窄档仍装得下，余 5");
        assert_eq!(slot_bytes(false, true, false, 12, code2_index_unit), 566);
        assert_eq!(slot_bytes(false, true, false, 12, code2_index_unit) - 512, 54, "E115 是 50");
        assert_eq!(slot_bytes(true, false, false, 12, code2_index_unit), 536);
        assert_eq!(slot_bytes(true, false, false, 12, code2_index_unit) - 512, 24, "E115 是 20");
        assert_eq!(slot_bytes(true, true, false, 12, code2_index_unit), 595);
        assert_eq!(slot_bytes(true, true, false, 12, code2_index_unit) - 512, 83, "E115 是 79");
    }

    /// **判据 3 的绝对值**：合并臂在新水位下四格全部仍装得下 512。
    #[test]
    fn criterion3_merge_arm_still_fits_512() {
        for &device_table_unit in [DeviceTableUnit::Code3Packed, DeviceTableUnit::Code2Index].iter() {
            for &wide in [false, true].iter() {
                for &map_sep in [false, true].iter() {
                    let total_bytes = slot_bytes(wide, map_sep, true, 12, device_table_unit);
                    assert!(total_bytes <= 512, "合并臂 {total_bytes} 应装得下 512");
                    assert_eq!(torn_states(total_bytes, 512), 0);
                }
            }
        }
        // 四个角的绝对值
        assert_eq!(slot_bytes(false, false, true, 12, DeviceTableUnit::Code3Packed), 385);
        assert_eq!(slot_bytes(true, true, true, 12, DeviceTableUnit::Code2Index), 477);
    }

    /// 合并臂省的字节与要付的单元头，绝对值。
    #[test]
    fn criterion3_merge_cost_is_absolute() {
        assert_eq!(3 * POINTER_BYTES, 177);
        assert_eq!(UNIT_HEADERS.iter().copied().max().unwrap(), 86);
        assert_eq!(3 * POINTER_BYTES - 86, 91, "槽里省 177、单元里最多付 86 ⇒ 净省 91");
    }

    /// **阴性对照**：4096 槽上全部格装得下、撕裂态恒 0。
    #[test]
    fn negative_control_4096_always_fits() {
        let mut checked = 0usize;
        for &device_table_unit in [DeviceTableUnit::Code3Packed, DeviceTableUnit::Code2Index].iter() {
            for &wide in [false, true].iter() {
                for &map_sep in [false, true].iter() {
                    for &merge_pointers in [false, true].iter() {
                        let total_bytes = slot_bytes(wide, map_sep, merge_pointers, 12, device_table_unit);
                        assert!(total_bytes <= 4096, "4096 槽上 {total_bytes} 必须装得下");
                        assert_eq!(torn_states(total_bytes, 4096), 0);
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 16, "共检查 16 格");
    }

    /// 撕裂态的绝对值：512 上一超就是 2 种（跨 2 扇区 ⇒ 2² − 2）。
    #[test]
    fn torn_states_are_absolute() {
        assert_eq!(torn_states(507, 512), 0);
        assert_eq!(torn_states(566, 512), 2);
        assert_eq!(torn_states(595, 512), 2);
        assert_eq!(torn_states(1025, 512), 6, "跨 3 扇区 ⇒ 2³ − 2");
    }
}
