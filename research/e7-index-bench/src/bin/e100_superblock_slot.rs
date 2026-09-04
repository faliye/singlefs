//! E100：超级块的三段几何 —— D22 未定项 9 欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里
//!
//! - **D22 已定项 8**：超级块**每盘一份**，更新走 **≥2 槽轮换**。
//! - **D22 已定三**：原地覆写的结构要带**整单元校验和**与**被实际检查的世代号**。
//! - **D20 推论三**：撕裂判定宽度 = 运行时探测的 `physical_block_size`
//!   ⇒ **一个槽要原子，就不能跨扇区**。本机 512（E79 用的就是它），另一档 4096。
//! - **D15 逐字**：feature bit「**位数**：三个 bitmap 各 **256 位**（`u64[4]`），存超级块」
//!   ⇒ 光 feature bit 就是 **96 字节**。
//! - **D15 止损规则逐字**：「**第 1、2 层冻结前必须已包含 D9 全部四项预留**」，而超级块是第 1 层。
//! - **D9 已定项 8 逐字**：「nonce 水位**不许住在裸明文超级块里**，必须落在一个被主密钥派生
//!   MAC 覆盖的字段上」⇒ 超级块要给 MAC 与 nonce 水位留位。
//! - **D12 已定项 3**：设备级几何量住超级块里的**设备描述符表**；**异构池要支持**。
//! - **E72**：表的字节数恰好 `设备数 × 条目宽度`；**条目宽度仓里没定过**，按 24 / 40 / 64 三档算。
//! - **D23**：「**journal 几何进超级块**（环大小、最大记录字节、在飞记录数上限），**不许是代码常量**」。
//! - **D16 已定项 5**：「`T_dirty` 是可调值，不是格式常量：超级块里那个字段是格式，
//!   字段里放什么数不是」。
//! - **C78 / E79 的先例**：512 字节槽带 jsn 水位只装 **6 棵树**，7 棵常识树集恰好 **513 字节爆 1 字节**
//!   ⇒ 根记录**恒走间接层**（D22 已定项 7）。**超级块面对的是同一个形状。**
//!
//! ## 判据（E100 正文跑前写死，跑完不许改）
//!
//! 1. **装得下装不下要算得出绝对值**：三段字节数之和 vs 槽宽（512 / 4096），
//!    逐格报「装得下 / 爆多少字节」。
//! 2. **设备表是唯一随规模长的东西**：扫设备数 1 / 2 / 8 / 64 / 256 × 条目宽 24 / 40 / 64，
//!    **数出第一个爆槽的格子**。
//! 3. **出路臂**：设备表移出槽（超级块只存指向设备表单元的指针，形态同 D22 已定项 7 的间接层）
//!    ⇒ 三段字节数必须变成**与设备数无关的常数**，且两档槽宽上恒装得下。
//! 4. **原子性判据**：一个槽必须恰好一个扇区。超过时**数出跨扇区的槽在撕裂点上
//!    有几种可读但不一致的状态**。
//! 5. **自举顺序**：段一里的字段必须在读到设备表之前解得出来——
//!    报出「能判 fsid / 能判 dev id / 能判 feature bits」各要读到第几字节。
//!
//! ## 失败条款（跑前写死）
//!
//! - **阳性对照**：设备表留在槽里、设备数 64，512 槽必须判爆；移出去之后必须判不爆。
//! - **阴性对照**：设备数 1、条目宽 24、4096 槽，必须判装得下。
//! - **反向接受条款**：出路臂在两档槽宽上仍装不下 ⇒ 结论是「D22 已定项 8 的槽轮换形态要重开」。
//! - 条目宽度三档不同向 ⇒ 写成「依赖条目宽度」并立成新的空白。
//!
//! ## 它答不了的
//!
//! 纯算术：没有 mkfs、没有挂载、没有真设备。不答「加盘时设备表怎么改」的事务形态。

use e7_index_bench::Emitter;

/// D20 推论三：槽宽 = 探测到的 `physical_block_size`。两档。
const SLOT_WIDTHS: [u64; 2] = [512, 4096];
/// D15 逐字：三个 bitmap 各 256 位 ⇒ 96 字节。
const FEATURE_BITS_BYTES: u64 = 3 * 256 / 8;
/// E72：设备描述符表的条目宽度**仓里没定过**，三档。
const DEV_ENTRY_WIDTHS: [u64; 3] = [24, 40, 64];

/// 段一：自举头。**每个字段都指得到一条已定条款**（见文件头）。
const SEC1: [(&str, u64); 7] = [
    ("magic", 4),
    ("format_version", 2),
    ("feature_bits", FEATURE_BITS_BYTES), // D15：三个 256 位 bitmap
    ("fsid", 16),
    ("this_dev_id", 4),   // D19 已定项 4 的设备 ID 同宽
    ("slot_generation", 8), // D22 已定三：被实际检查的世代号
    ("unit_checksum", 32),  // D22 已定三：整单元校验和
];

/// D9 的 day-1 预留（D15 止损规则：第 1、2 层冻结前必须已包含 D9 全部四项预留）。
const SEC1_CRYPTO: [(&str, u64); 2] = [("sb_mac", 16), ("nonce_watermark", 8)];

/// 段二：几何。设备表另算（它是唯一随规模长的）。
const SEC2_FIXED: [(&str, u64); 8] = [
    ("node_bytes", 4),
    ("unit_bytes", 4),
    ("alloc_grain", 4),      // D3 已定项 7
    ("posentry_widths", 4),  // D19 已定项 4
    ("journal_geom", 24),    // D23：环大小 + 最大记录字节 + 在飞上限
    ("ring_R_S", 4),         // D22 已定项 2
    ("tree_table_ptr", 59),  // D22 已定项 7 的树表单元指针（间接层）
    ("map_provenance", 24),  // D18：重建纪元 + 权威态摘要，day-1 留位
];

/// 段三：可调值。D16 已定项 5 + D26 未定项 1 的三个水位。
const SEC3: [(&str, u64); 5] = [
    ("t_time", 4),
    ("t_dirty", 8),
    ("compact_lo", 8),
    ("compact_hi", 8),
    ("compact_stop", 8),
];

/// 第四类：**运行时由系统改的水位 / 代号 / 纪元**。
/// 第一版提案的三段轴收不了它们——tail 槽每个 checkpoint 改一次，
/// 既非永不改、非 mkfs 只读、也非人调的可调值。
const SEC4_RUNTIME: [(&str, u64); 2] = [
    ("journal_tail", 8),     // D23 已定项 3：tail 住超级块槽
    ("journal_instance", 4), // D23 已定项 9 的实例代号
];

fn sum(t: &[(&str, u64)]) -> u64 {
    t.iter().map(|x| x.1).sum()
}

/// 三段 + 第四类 + 设备表 的总字节。`inline_devtable` = 设备表在不在槽里。
fn total_bytes(devs: u64, dev_entry: u64, inline_devtable: bool, with_crypto: bool) -> u64 {
    let mut b = sum(&SEC1) + sum(&SEC2_FIXED) + sum(&SEC3) + sum(&SEC4_RUNTIME);
    if with_crypto {
        b += sum(&SEC1_CRYPTO);
    }
    if inline_devtable {
        b += devs * dev_entry;
    } else {
        b += 59; // 指向设备表单元的指针，同 D22 已定项 7 的树表单元指针宽度
    }
    b
}

/// **判据 4**：一个槽跨 `n` 个扇区时，撕裂点上有几种可读但不一致的状态。
/// 每个扇区各自要么是新值要么是旧值 ⇒ `2^n` 种组合，去掉「全新」与「全旧」两种一致态。
fn torn_states(total: u64, slot: u64) -> u64 {
    if slot == 0 {
        return 0;
    }
    let sectors = total.div_ceil(slot);
    if sectors <= 1 {
        return 0;
    }
    (1u64 << sectors).saturating_sub(2)
}

/// **判据 5**：解析器读到第几字节才判得出某个字段。段一按声明次序排布。
fn bytes_to_reach(field: &str) -> Option<u64> {
    let mut off = 0u64;
    for (n, w) in SEC1.iter() {
        off += w;
        if *n == field {
            return Some(off);
        }
    }
    None
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();

    out.push(em.emit_raw(&format!(
        "name=config sec1={} sec1_crypto={} sec2_fixed={} sec3={} sec4_runtime={} feature_bits={}",
        sum(&SEC1),
        sum(&SEC1_CRYPTO),
        sum(&SEC2_FIXED),
        sum(&SEC3),
        sum(&SEC4_RUNTIME),
        FEATURE_BITS_BYTES
    )));

    // 判据 1 / 2 / 3：装得下装不下
    for &inline in [true, false].iter() {
        for &crypto in [false, true].iter() {
            for &dw in DEV_ENTRY_WIDTHS.iter() {
                for &devs in [1u64, 2, 8, 64, 256].iter() {
                    let t = total_bytes(devs, dw, inline, crypto);
                    for &slot in SLOT_WIDTHS.iter() {
                        let over = t.saturating_sub(slot);
                        out.push(em.emit_raw(&format!(
                            "name=fit inline_devtable={} with_crypto={} dev_entry={dw} devs={devs} \
                             total={t} slot={slot} fits={} over_by={over} torn_states={}",
                            u8::from(inline),
                            u8::from(crypto),
                            u8::from(t <= slot),
                            torn_states(t, slot)
                        )));
                    }
                }
            }
        }
    }

    // 判据 5：自举顺序
    for f in ["magic", "format_version", "feature_bits", "fsid", "this_dev_id"].iter() {
        out.push(em.emit_raw(&format!(
            "name=bootstrap field={f} bytes_to_reach={}",
            bytes_to_reach(f).map(|v| v.to_string()).unwrap_or_else(|| "NA".into())
        )));
    }

    for l in &out {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_match_kb() {
        assert_eq!(FEATURE_BITS_BYTES, 96, "D15：三个 256 位 bitmap");
        assert_eq!(SLOT_WIDTHS, [512, 4096], "D20 推论三：探测到的 physical_block_size");
        assert_eq!(DEV_ENTRY_WIDTHS, [24, 40, 64], "E72 的三档假设");
    }

    /// **判据 1 的绝对值**：段一光 feature bit 就 96 字节，整段 162。
    #[test]
    fn criterion1_section_sizes_are_absolute() {
        // 手算：4 + 2 + 96 + 16 + 4 + 8 + 32 = 162
        assert_eq!(sum(&SEC1), 162);
        assert_eq!(sum(&SEC1_CRYPTO), 24, "D9 的 MAC 16 + nonce 水位 8");
        // 手算：4+4+4+4+24+4+59+24 = 127
        assert_eq!(sum(&SEC2_FIXED), 127);
        assert_eq!(sum(&SEC3), 36);
        assert_eq!(sum(&SEC4_RUNTIME), 12);
        // 不含设备表的固定开销：162+24+127+36+12 = 361
        assert_eq!(
            sum(&SEC1) + sum(&SEC1_CRYPTO) + sum(&SEC2_FIXED) + sum(&SEC3) + sum(&SEC4_RUNTIME),
            361
        );
    }

    /// **判据 2 的绝对值 + 阳性对照**：设备表留在槽里时，512 字节槽在很小的设备数上就爆。
    #[test]
    fn criterion2_inline_device_table_overflows_a_512_byte_slot_early() {
        // 361 + devs × 24 ≤ 512 ⇒ devs ≤ 6（6×24 = 144，361+144 = 505）
        assert_eq!(total_bytes(6, 24, true, true), 505);
        assert!(total_bytes(6, 24, true, true) <= 512);
        assert_eq!(total_bytes(7, 24, true, true), 529, "7 台就爆 17 字节");
        assert!(total_bytes(7, 24, true, true) > 512);
        // 阳性对照：64 台必须爆
        assert!(total_bytes(64, 24, true, true) > 512);
        assert_eq!(total_bytes(64, 24, true, true), 1897);
        // 最宽的条目更早爆：64 字节一条 ⇒ 512 槽只装得下 2 台
        assert_eq!(total_bytes(2, 64, true, true), 489);
        assert_eq!(total_bytes(3, 64, true, true), 553);
    }

    /// **判据 3 的绝对值**：移出去之后与设备数无关，两档槽宽都装得下。
    #[test]
    fn criterion3_indirection_makes_it_constant_and_it_fits() {
        let a = total_bytes(1, 24, false, true);
        let b = total_bytes(256, 64, false, true);
        assert_eq!(a, b, "移出槽之后与设备数、条目宽都无关");
        // 手算：361 + 59（设备表单元指针）= 420
        assert_eq!(a, 420);
        assert!(a <= 512, "512 字节槽装得下");
        assert!(a <= 4096);
        // 阳性对照的另一半：inline 时同样的格子必须爆
        assert!(total_bytes(256, 64, true, true) > 512);
    }

    /// **阴性对照**：最宽松的格子必须判装得下。
    #[test]
    fn negative_control_smallest_case_fits() {
        assert!(total_bytes(1, 24, true, false) <= 4096);
        assert!(total_bytes(1, 24, true, true) <= 512, "361 + 24 = 385");
        assert_eq!(total_bytes(1, 24, true, true), 385);
    }

    /// **判据 4 的绝对值**：跨扇区的槽在撕裂点上有 2^n − 2 种不一致状态。
    #[test]
    fn criterion4_a_slot_that_spans_sectors_is_not_atomic() {
        assert_eq!(torn_states(420, 512), 0, "一个扇区内 ⇒ 原子，零撕裂态");
        assert_eq!(torn_states(513, 512), 2, "跨 2 扇区 ⇒ 2^2 − 2 = 2");
        assert_eq!(torn_states(1897, 512), 14, "跨 4 扇区 ⇒ 2^4 − 2 = 14");
        // 阳性对照：把槽宽设成 0 之外的极小值，撕裂态必须爆炸式上升
        assert!(torn_states(1897, 128) > torn_states(1897, 512));
        assert_eq!(torn_states(0, 512), 0);
    }

    /// **判据 5 的绝对值**：解析器要读到第几字节才判得出各字段。
    #[test]
    fn criterion5_bootstrap_order_is_absolute() {
        assert_eq!(bytes_to_reach("magic"), Some(4));
        assert_eq!(bytes_to_reach("format_version"), Some(6));
        assert_eq!(bytes_to_reach("feature_bits"), Some(102), "6 + 96");
        assert_eq!(bytes_to_reach("fsid"), Some(118));
        assert_eq!(bytes_to_reach("this_dev_id"), Some(122));
        assert_eq!(bytes_to_reach("not_a_field"), None, "不存在的字段报 None，不许返回 0");
    }

    /// 第四类不是可有可无的：三段轴收不了 tail 槽这种「每个 checkpoint 改一次」的字段。
    #[test]
    fn a_fourth_class_is_needed_for_runtime_watermarks() {
        assert_eq!(SEC4_RUNTIME.len(), 2);
        assert_eq!(sum(&SEC4_RUNTIME), 12);
        // 把它们塞进段一（「永不改」）会让段一每个 checkpoint 都变 ⇒ 段一的定义作废
        assert!(sum(&SEC4_RUNTIME) > 0, "有对象，不是空类");
    }
}
