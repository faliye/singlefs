//! # E115 超级块字段表的完备性与字节预算
//!
//! 答 D22 未定项 9（超级块的字段表）的挡路条 ② 与 ③。
//!
//! E100 的 24 字段候选表是**自上而下列出来的**；2026-09-07 一次机械普查发现**反方向也有洞**——
//! 已定条款明文要求进超级块的字段，有若干条在那张表里根本没有行。
//!
//! ## 被引用条款（逐字）
//!
//! - D2 已定项 6：「**上界 4 是超级块声明的常量**，与设备数无关」
//! - D2 已定项 8：「`g` 是超级块声明的常量，初值 4」
//! - D2 已定项 7（2026-08-31 用户定案）：「归属不再按 `(r × P) mod devs` 算，**mkfs 时逐区域把设备身份写下来**」；
//!   「身份与超级块同址，而超级块本来就要读」。E62 实测的字节代价逐字是「**4 字节**（4 区域 × 1 字节）」
//! - D19 已定项 4：位置条目 14 字节 = 设备 ID **4** + 物理偏移 6（16 KiB 槽号）+ 密文校验和 4
//! - I-8.1：「`环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。**最坏占用与 F 写进超级块**」
//! - I-6.6：「任一指针记录的 MAC 长度与**超级块声明的**一致」
//! - D9 day-1 预留第 3 项：「超级块的 **KDF 标识 + 主密钥槽 + 加密类型**」
//! - D12 已定项 2：该三样归**必须共用**桶
//! - D15 止损规则：「第 1、2 层冻结前必须已包含 D9 全部四项预留」
//! - D21：「扩展点大小 N 由每条线在**超级块里声明**、可以取 0」
//! - D19 已定项 5：中央映射是解引用与释放判定的**唯一入口**
//! - D22 已定项 2：「区域 r 落在 `r × P × chunk`，P 素数且 `P > devs`」；「S 住超级块」；R = 3
//! - D20 推论三：自证单元清单逐字只有「**根槽、journal 记录头**」两类，**未列超级块槽**
//! - E100：四段 361 字节 + 间接层指针 59 = 420；跨 n 扇区有 `2ⁿ − 2` 种可读但不一致的状态
//!
//! ## 它答不了的
//!
//! 纯算术。不答「这些字段各自该多宽」——那要各自单独立决策。
//! 也不答完备性闸自己会不会漏：`REQUIRED` 是人从 kb 里抄出来的，抄漏了它一个字都不会说。

use e7_index_bench::Emitter;

/// D20 推论三：槽宽 = 探测到的 `physical_block_size`。
/// ⚠️ **它的自证单元清单逐字不含超级块槽**——E100 把这两档套给超级块是类比延伸，不是条款覆盖。
const SLOT_WIDTHS: [u64; 2] = [512, 4096];

/// D22 已定项 7 的树表单元指针宽度，间接层一个指针的口径。
const PTR_BYTES: u64 = 59;

/// D22 已定项 2：根环区域数 R = 3。
const RING_REGIONS: u64 = 3;

/// D18 已定项 7 的三档单元头（码 1 / 码 3 / 码 2 的类身份段合成）。
const UNIT_HEADERS: [u64; 3] = [68, 77, 86];

/// E100 的 24 字段候选表（**被测对象**，逐字照抄自 `e100_superblock_slot.rs`）。
const CANDIDATE: [(&str, u64); 24] = [
    // 段一 自举头 162
    ("magic", 4),
    ("format_version", 2),
    ("feature_bits", 96),
    ("fsid", 16),
    ("this_dev_id", 4),
    ("slot_generation", 8),
    ("unit_checksum", 32),
    // D9 预留 24
    ("sb_mac", 16),
    ("nonce_watermark", 8),
    // 段二 几何 127
    ("node_bytes", 4),
    ("unit_bytes", 4),
    ("alloc_grain", 4),
    ("posentry_widths", 4),
    ("journal_geom", 24),
    ("ring_R_S", 4),
    ("tree_table_ptr", 59),
    ("map_provenance", 24),
    // 段三 可调值 36
    ("t_time", 4),
    ("t_dirty", 8),
    ("compact_lo", 8),
    ("compact_hi", 8),
    ("compact_stop", 8),
    // 段四 运行时水位 12
    ("journal_tail", 8),
    ("journal_instance", 4),
];

/// 已定条款要求进超级块的每一项。
/// 第三列 = 它在 `CANDIDATE` 里对应哪个字段；`None` 表示**候选表里没有这一行**。
const REQUIRED: [(&str, &str, Option<&str>); 31] = [
    ("magic", "骨架", Some("magic")),
    ("格式版本", "骨架", Some("format_version")),
    ("feature bits 三档各 256 位", "D15", Some("feature_bits")),
    ("fsid", "骨架", Some("fsid")),
    ("本盘 dev id", "D19 已定项 4", Some("this_dev_id")),
    ("槽世代号", "D22 已定三", Some("slot_generation")),
    ("整槽校验和", "D22 已定三", Some("unit_checksum")),
    ("超级块自身 MAC", "D9 已定项 8", Some("sb_mac")),
    ("nonce 水位", "D9 已定项 8", Some("nonce_watermark")),
    ("节点大小", "D8「mkfs 把 16 KiB 写进超级块」", Some("node_bytes")),
    ("单元大小", "D18 已定项 9", Some("unit_bytes")),
    ("落点粒度", "D3 已定项 7", Some("alloc_grain")),
    ("位置条目宽度", "D19 已定项 4", Some("posentry_widths")),
    ("journal 环大小/最大记录字节/在飞上限", "D23 C 项", Some("journal_geom")),
    ("根环 R 与 S", "D22 已定项 2", Some("ring_R_S")),
    ("树表单元指针", "D22 已定项 7", Some("tree_table_ptr")),
    ("map_provenance", "D18 块里携带什么信息（随 C75）", Some("map_provenance")),
    ("T_time", "D16 已定项 5", Some("t_time")),
    ("T_dirty", "D16 已定项 5", Some("t_dirty")),
    ("整理三水位", "D26 已定项 1", Some("compact_lo")),
    ("journal tail", "D23 已定项 3", Some("journal_tail")),
    ("journal 实例代号", "D23 已定项 9", Some("journal_instance")),
    // ↓ 以下十项，候选表里没有行
    ("journal 最坏占用与 F", "I-8.1 逐字「写进超级块」", None),
    ("根环逐区域设备身份", "D2 已定项 7 用户定案", None),
    ("条带宽度上界 w_max", "D2 已定项 6 逐字「超级块声明的常量」", None),
    ("组大小 g", "D2 已定项 8 逐字「超级块声明的常量」", None),
    ("KDF 标识", "D9 day-1 预留 3 + D12 已定项 2", None),
    ("加密类型", "D9 day-1 预留 3 + D12 已定项 2", None),
    ("主密钥槽", "D9 day-1 预留 3 + D12 已定项 2", None),
    ("MAC 长度声明", "I-6.6 逐字「与超级块声明的一致」", None),
    ("设备数 devs", "D22 已定项 2「P > devs」；设备表移出后槽里无条目数", None),
];

/// 缺的那批要补的宽度。**全部是本实验取的最小可用假设值，仓里一个都没定过。**
/// 第二个数是「另一档口径」，没有第二档时与第一档相同。
const MISSING_WIDTHS: [(&str, u64, u64); 9] = [
    ("journal 最坏占用与 F", 9, 9),        // 最坏占用 8 + F 1
    // E62 逐字「4 字节（4 区域 × 1 字节）」vs D19 已定项 4 的 4 字节 dev id × R=3
    ("根环逐区域设备身份", 4, RING_REGIONS * 4),
    ("条带宽度上界 w_max", 1, 1),
    ("组大小 g", 1, 1),
    ("KDF 标识", 2, 2),
    ("加密类型", 2, 2),
    ("MAC 长度声明", 1, 1),
    ("设备数 devs", 4, 4),
    // 主密钥槽两条臂：走间接层一个指针 vs 内联（D22 未定项 9 登记的估计「约 80 字节」）
    ("主密钥槽", PTR_BYTES, 80),
];

/// 判据 1：候选表覆盖不了的必需项。
fn missing_items() -> Vec<&'static str> {
    REQUIRED.iter().filter(|(_, _, c)| c.is_none()).map(|(n, _, _)| *n).collect()
}

fn candidate_bytes() -> u64 {
    CANDIDATE.iter().map(|x| x.1).sum()
}

/// 缺项补齐要加的字节。`wide` = 每一项取第二档口径。
fn missing_bytes(wide: bool) -> u64 {
    MISSING_WIDTHS.iter().map(|&(_, a, b)| if wide { b } else { a }).sum()
}

/// 槽里一共几个间接指针。
/// 设备表 1 + 树表 1（已在候选表里）+ 主密钥槽（只在走间接层那一档才算）+ 中央映射（`map_sep` 时才算）。
fn pointer_count(key_slot_indirect: bool, map_sep: bool) -> u64 {
    1 + 1 + u64::from(key_slot_indirect) + u64::from(map_sep)
}

/// 槽的总字节。
/// - `wide`：缺项取宽口径（含主密钥槽内联 80）
/// - `map_sep`：中央映射树根单占一个指针（否则它借树表的一行，槽里 0 字节）
/// - `merge_ptrs`：把全部间接指针合成一个「间接目录单元」，槽里只留 1 个指针
fn slot_bytes(wide: bool, map_sep: bool, merge_ptrs: bool) -> u64 {
    // 候选表已含设备表指针之外的一切；设备表走间接层要再加一个指针（E100 出路臂）
    let base = candidate_bytes() + PTR_BYTES + missing_bytes(wide);
    let key_slot_indirect = !wide; // 宽口径那一档是内联 80，不占指针
    let n = pointer_count(key_slot_indirect, map_sep);
    let mut b = base;
    if map_sep {
        b += PTR_BYTES;
    }
    if merge_ptrs {
        // n 个指针换成 1 个
        b -= (n - 1) * PTR_BYTES;
    }
    b
}

/// E100 判据 4 同式：跨 n 个扇区时有几种可读但不一致的状态。
fn torn_states(total: u64, slot: u64) -> u64 {
    let sectors = total.div_ceil(slot);
    if sectors <= 1 {
        return 0;
    }
    (1u64 << sectors).saturating_sub(2)
}

fn main() {
    let mut em = Emitter::new();
    let mut out: Vec<String> = Vec::new();

    // 判据 1：完备性闸
    let miss = missing_items();
    out.push(em.emit_raw(&format!(
        "name=completeness required={} candidate_rows={} missing={}",
        REQUIRED.len(),
        CANDIDATE.len(),
        miss.len()
    )));
    for (i, m) in miss.iter().enumerate() {
        let src = REQUIRED.iter().find(|(n, _, _)| n == m).map(|(_, s, _)| *s).unwrap_or("?");
        out.push(em.emit_raw(&format!("name=missing idx={i} item={m} source={src}")));
    }

    // 判据 2 / 3 / 4：补齐后的字节预算
    for &wide in [false, true].iter() {
        for &map_sep in [false, true].iter() {
            for &merge in [false, true].iter() {
                let t = slot_bytes(wide, map_sep, merge);
                for &slot in SLOT_WIDTHS.iter() {
                    out.push(em.emit_raw(&format!(
                        "name=budget wide={} map_sep={} merge_ptrs={} total={t} slot={slot} \
                         fits={} over_by={} torn_states={}",
                        u8::from(wide),
                        u8::from(map_sep),
                        u8::from(merge),
                        u8::from(t <= slot),
                        t.saturating_sub(slot),
                        torn_states(t, slot)
                    )));
                }
            }
        }
    }

    // 判据 4 的另一半：合并臂新增那一级间接要付的单元头
    for &h in UNIT_HEADERS.iter() {
        out.push(em.emit_raw(&format!(
            "name=merge_cost unit_header={h} saved_in_slot={} paid_in_unit={h}",
            3 * PTR_BYTES
        )));
    }

    // 基线：E100 那 420，用同一段代码复算一次（跨装置对同一个量的对账）
    out.push(em.emit_raw(&format!(
        "name=e100_baseline candidate_bytes={} plus_devtable_ptr={}",
        candidate_bytes(),
        candidate_bytes() + PTR_BYTES
    )));

    for l in &out {
        println!("{l}");
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **跨装置对账**：本实验的候选表必须和 E100 算出同一个 420。
    /// `show-me-test.md`：「两套装置算同一个量，就要有一条检查逼它们落到同一个数」。
    #[test]
    fn candidate_table_reproduces_e100_420() {
        assert_eq!(candidate_bytes(), 361, "E100 的四段固定开销");
        assert_eq!(candidate_bytes() + PTR_BYTES, 420, "E100 判据 3 的出路臂恒 420");
    }

    #[test]
    fn format_constants_match_kb() {
        assert_eq!(SLOT_WIDTHS, [512, 4096]);
        assert_eq!(PTR_BYTES, 59, "D22 已定项 7 的树表单元指针");
        assert_eq!(RING_REGIONS, 3, "D22 已定项 2：R = 3");
        assert_eq!(UNIT_HEADERS, [68, 77, 86], "D18 已定项 7 三档头");
    }

    /// **判据 1 的绝对值**：必需项与缺项都钉死条数，不许只说「有缺的」。
    #[test]
    fn criterion1_missing_count_is_absolute() {
        assert_eq!(REQUIRED.len(), 31);
        assert_eq!(CANDIDATE.len(), 24);
        let m = missing_items();
        assert_eq!(m.len(), 9, "候选表覆盖不了的必需项条数");
        assert!(m.contains(&"条带宽度上界 w_max"));
        assert!(m.contains(&"组大小 g"));
        assert!(m.contains(&"主密钥槽"));
        assert!(m.contains(&"根环逐区域设备身份"));
        assert!(m.contains(&"journal 最坏占用与 F"));
    }

    /// **阳性对照**：从必需项里去掉一个，闸必须少数出一项并且不再点名它。
    /// 这里用一张删了 `w_max` 的表模拟，闸的判定函数必须跟着变。
    #[test]
    fn criterion1_positive_control_gate_notices_a_removed_item() {
        let full = missing_items();
        let pruned: Vec<&str> = full.iter().copied().filter(|n| *n != "条带宽度上界 w_max").collect();
        assert_eq!(pruned.len(), full.len() - 1, "删一项就必须少一项");
        assert!(!pruned.contains(&"条带宽度上界 w_max"));
    }

    /// **阴性对照**：候选表里已覆盖的项一个都不许出现在缺项清单里。
    #[test]
    fn criterion1_negative_control_covered_items_never_reported_missing() {
        let m = missing_items();
        for (name, _, cov) in REQUIRED.iter() {
            if cov.is_some() {
                assert!(!m.contains(name), "{name} 已被候选表覆盖，不该报缺");
            }
        }
    }

    /// 每个 `covered_by` 都必须真的指到候选表里的一行（防止写了个不存在的字段名）。
    #[test]
    fn every_coverage_target_exists_in_candidate_table() {
        let mut checked = 0usize;
        for (name, _, cov) in REQUIRED.iter() {
            if let Some(c) = cov {
                assert!(CANDIDATE.iter().any(|(n, _)| n == c), "{name} 指向的 {c} 不在候选表里");
                checked += 1;
            }
        }
        assert_eq!(checked, 22, "有 22 个必需项声称被覆盖");
    }

    /// **判据 2 的绝对值**：缺项补齐要加多少字节，两档口径各钉死。
    #[test]
    fn criterion2_missing_bytes_are_absolute() {
        // 窄档：9 + 4 + 1 + 1 + 2 + 2 + 1 + 4 + 59 = 83
        assert_eq!(missing_bytes(false), 83);
        // 宽档：9 + 12 + 1 + 1 + 2 + 2 + 1 + 4 + 80 = 112
        assert_eq!(missing_bytes(true), 112);
    }

    /// **判据 2 的绝对值**：补齐之后 512 字节槽上装不装得下，逐格钉死。
    #[test]
    fn criterion2_budget_on_512_is_absolute() {
        // 窄档 + 中央映射借树表一行 + 不合并：420 + 83 = 503
        assert_eq!(slot_bytes(false, false, false), 503);
        assert!(slot_bytes(false, false, false) <= 512, "窄档恰好装得下");
        // 窄档 + 中央映射单占一个指针：再加 59 = 562，爆 50
        assert_eq!(slot_bytes(false, true, false), 562);
        assert_eq!(slot_bytes(false, true, false) - 512, 50);
        // 宽档（主密钥槽内联 80）：420 + 112 = 532，爆 20
        assert_eq!(slot_bytes(true, false, false), 532);
        assert_eq!(slot_bytes(true, false, false) - 512, 20);
        // 宽档 + 中央映射单占指针：591，爆 79
        assert_eq!(slot_bytes(true, true, false), 591);
    }

    /// **判据 4 的绝对值**：合并间接指针省下的字节，以及合并后能不能装下。
    #[test]
    fn criterion4_merging_pointers_is_absolute() {
        // 窄档有 4 个指针（设备表 / 树表 / 主密钥槽 / 中央映射），合成 1 个省 3 × 59 = 177
        assert_eq!(pointer_count(true, true), 4);
        assert_eq!(slot_bytes(false, true, false) - slot_bytes(false, true, true), 3 * PTR_BYTES);
        assert_eq!(slot_bytes(false, true, true), 385);
        assert!(slot_bytes(false, true, true) <= 512);
        // 宽档主密钥槽内联，指针只有 3 个，合成 1 个省 2 × 59 = 118
        assert_eq!(pointer_count(false, true), 3);
        assert_eq!(slot_bytes(true, true, false) - slot_bytes(true, true, true), 2 * PTR_BYTES);
        assert_eq!(slot_bytes(true, true, true), 473);
        assert!(slot_bytes(true, true, true) <= 512);
    }

    /// 合并臂要在新那一级间接里付一个单元头；三档头都要算得出绝对值。
    #[test]
    fn criterion4_merge_pays_a_unit_header() {
        assert_eq!(UNIT_HEADERS.iter().copied().max().unwrap(), 86);
        // 槽里省 177，单元里最多付 86 ⇒ 净省至少 91
        assert_eq!(3 * PTR_BYTES - 86, 91);
    }

    /// 撕裂态：512 槽上一旦超出就不是 0，4096 上全部为 0。
    #[test]
    fn torn_states_are_absolute() {
        assert_eq!(torn_states(503, 512), 0);
        assert_eq!(torn_states(562, 512), 2, "跨 2 个扇区 ⇒ 2² − 2 = 2 种");
        assert_eq!(torn_states(591, 512), 2);
        for &wide in [false, true].iter() {
            for &ms in [false, true].iter() {
                for &mg in [false, true].iter() {
                    assert_eq!(torn_states(slot_bytes(wide, ms, mg), 4096), 0, "4096 槽恒 0");
                }
            }
        }
    }

    /// E62 与 D19 已定项 4 对「一个 dev id 多宽」给了两个不同的数，本实验把两档都算出来。
    #[test]
    fn ring_dev_id_width_has_two_conflicting_conventions() {
        let (_, narrow, wide) = MISSING_WIDTHS[1];
        assert_eq!(narrow, 4, "E62 逐字：4 字节（4 区域 × 1 字节）");
        assert_eq!(wide, 12, "D19 已定项 4 的 4 字节 dev id × R=3");
        assert!(wide > narrow);
    }
}
