//! E75：记录尺寸与环几何 —— D23 已定项 12（记录尺寸取 4 KiB）欠的那次测量。
//!
//! ## 被引用条款逐字贴在这里（verify-before-claiming.md「把定义句原样贴进实验注释」）
//!
//! - D23 已定项 12（2026-09-01 用户定案）：「定为 **4 KiB，写进超级块**」，
//!   依据逐字是「E45 按『塞满一个定长记录』算出 4096 字节的记录装 `(4096 − 99) / 56 = 71` 项，
//!   而 D25 已定的目标负载是一次 fsync 带 8 叶、落在 1 条共享脊柱上，E45 记作 12 项事务
//!   ⇒ **5.9 倍余量**」。并自陈「**那是算术，不是实测**」。
//! - D23 已定项 4：「记录头能不能被约束在单个原子单元内：**能。头 78 字节。**」
//!   并逐字附一句：「**两笔已定、待落地的增量不在 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、
//!   已定项 8 的反向链 **4 字节** ⇒ 两者落地后是 **91 字节**。引用 78 时要连同这一句一起引。」
//! - D23 已定项 13（2026-09-01 用户定案）：`header_csum` 仍只覆盖头，**另加一个覆盖点名项数组的校验和**，
//!   算法 CRC32C ⇒ 这是**第三笔**已定待落地的增量，4 字节。
//! - I-8.1（环几何够大）逐字：「`环大小 ≥ F × 任一事务的最坏 journal 占用`，F ≥ 2。」
//! - I-8.2（记录头不跨原子单元）逐字：「任一 journal 记录的头完整落在一个 `physical_block_size` 单元内。
//!   该宽度**运行时探测**，不许硬编码。」
//! - E23：点名项宽 **56 字节**；对齐代价表（变长记录）pbs=4096 时点名 1 项 140 B → 4096 B。
//! - D25（目标负载优先级）：一次 fsync 带 **8 叶**、落在 **1 条共享脊柱**上 ⇒ E45 记作 **12 项**。
//!
//! ## ⚠️ 不许再犯的那个错（E45 结论作废的病灶）
//!
//! E45 把 I-8.2 实现成了「**整条记录起于单元边界**」，而 I-8.2 字面只要求**头**不跨单元。
//! 那条自造的严规则单独造出了一个 12 倍，结论作废。
//! **本实验按字面实现**：起始偏移 `o` 合法 ⟺ `o mod unit ≤ unit − hdr`。
//!
//! ## 判据（E75 正文跑前写死，跑完不许改）
//!
//! 1. **绝对值断言**：每条记录装得下的项数必须恰好等于 `(记录尺寸 − 记录头) / 56`，
//!    与 E45 的算术逐格相等；对不上说明两边有一边算错了。
//! 2. 4 KiB 那一档，目标负载的事务必须**恰好占一条记录**（跨记录数 = 0）。
//! 3. 记录头必须完整落在一个 `physical_block_size` 内（I-8.2），对探测到的 512 与 4096 各验一次。
//!
//! ## 失败条款
//!
//! - **阳性对照**：把负载放大到 200 项，4 KiB 那一档必须出现跨记录；
//!   不出现说明跨记录路径根本没走到。
//! - 若目标负载后来长大一个量级，这个数要重推，那时另立实验，不是给 E75 加一轮。
//!
//! ## 它答不了的
//!
//! 纯算术模型：没有 journal 实现、没有事务层，文件操作 0 处。
//! 「一个点名项恰好 56 字节」与「一个 12 项事务恰好点名 12 项」都是**沿用 E23/E45 的口径**，
//! 本实验不重新论证它们。环回绕、内存里先缓冲整条记录这两条代价（E45 攻击轮点名的）**没建模**。

use e7_index_bench::Emitter;

/// 点名项宽度。E23 字段表口径，E45 沿用。
const ITEM: u64 = 56;
/// D23 已定项 4 的现行头部字节数。**格式常量**，与 kb 的 format-const 标记绑定。
const JOURNAL_HDR: u64 = 78;
/// 已定项 7 的事务号 + 提交标记，9 字节。
const INC_TXN_BOUNDARY: u64 = 9;
/// 已定项 8 的反向链，32 位 = 4 字节。
const INC_BACK_CHAIN: u64 = 4;
/// 已定项 13 的载荷校验和，CRC32C 32 位 = 4 字节。
const INC_PAYLOAD_CSUM: u64 = 4;
/// D23 已定项 12 的算术里用的那个头部数（含已被已定项 3 删掉的 `tail_lsn` 8 字节）。
const HDR_AS_CITED_BY_D23_ITEM12: u64 = 99;
/// D25 目标负载：8 叶 + 4 祖先。E45 口径。
const TARGET_ITEMS: u64 = 12;
/// I-8.1 的安全系数下界。
const F: u64 = 2;

/// 一条 `rec` 字节的定长记录，扣掉 `hdr` 字节头之后装得下几个点名项。
/// 头装不下时返回 0——**不是「装得下 0 项」，是这个尺寸根本不合法**，调用方要分开处理。
fn items_per_record(rec: u64, hdr: u64) -> u64 {
    if rec <= hdr {
        return 0;
    }
    (rec - hdr) / ITEM
}

/// `items` 个点名项要几条记录。cap 为 0 时返回 None——**读不到 ≠ 读到 0**。
fn records_for(items: u64, cap: u64) -> Option<u64> {
    if cap == 0 {
        return None;
    }
    Some(items.div_ceil(cap).max(1))
}

/// I-8.2 **字面**判据：起始偏移 `o` 上放一条头宽 `hdr` 的记录，头跨不跨 `unit` 边界。
fn header_straddles(o: u64, unit: u64, hdr: u64) -> bool {
    o % unit + hdr > unit
}

/// 把 `n` 条定长记录背靠背摆进按字节寻址的环，只在**头会跨单元**时才推到下一个单元边界。
/// 返回游标推进的总字节数 = 这批记录的环占用。
fn laid_out_bytes(n: u64, rec: u64, unit: u64, hdr: u64) -> u64 {
    let mut cur = 0u64;
    for _ in 0..n {
        if header_straddles(cur, unit, hdr) {
            cur = cur.div_ceil(unit) * unit;
        }
        cur += rec;
    }
    cur
}

/// I-8.1 反解的环大小下界：`F × 最坏事务占用`。
fn ring_floor(items: u64, rec: u64, unit: u64, hdr: u64) -> Option<u64> {
    let cap = items_per_record(rec, hdr);
    let n = records_for(items, cap)?;
    Some(F * laid_out_bytes(n, rec, unit, hdr))
}

fn main() {
    let mut em = Emitter::new();
    let hdr_landed = JOURNAL_HDR + INC_TXN_BOUNDARY + INC_BACK_CHAIN;
    let hdr_full = hdr_landed + INC_PAYLOAD_CSUM;
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config item={ITEM} hdr_now={JOURNAL_HDR} hdr_landed={hdr_landed} \
             hdr_full={hdr_full} hdr_cited_by_d23_item12={HDR_AS_CITED_BY_D23_ITEM12} \
             target_items={TARGET_ITEMS} f={F} model=arithmetic file_ops=0"
        ))
    );

    // ── 主扫：记录尺寸 × 四个头部口径 ────────────────────────────────────
    // 四个口径同时报，是因为 D23 已定项 12 的算术用的是 99，而现行值是 78，
    // 三笔已定待落地的增量把它抬到 95。**哪一个才是权威，正是本实验要暴露的。**
    for rec_kib in [1u64, 2, 4, 8, 32] {
        let rec = rec_kib * 1024;
        for (tag, hdr) in [
            ("now78", JOURNAL_HDR),
            ("landed91", hdr_landed),
            ("full95", hdr_full),
            ("d23item12_99", HDR_AS_CITED_BY_D23_ITEM12),
        ] {
            let cap = items_per_record(rec, hdr);
            let n_target = records_for(TARGET_ITEMS, cap);
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=capacity rec_kib={rec_kib} hdr_tag={tag} hdr={hdr} \
                     items_per_record={cap} target_records={} target_spans={}",
                    n_target.map_or("NA".into(), |v| v.to_string()),
                    n_target.map_or("NA".into(), |v| u8::from(v > 1).to_string())
                ))
            );
        }
    }

    // ── 判据 3：记录头完整落在一个 physical_block_size 内，512 与 4096 各验一次 ──
    for pbs in [512u64, 4096] {
        for (tag, hdr) in [
            ("now78", JOURNAL_HDR),
            ("landed91", hdr_landed),
            ("full95", hdr_full),
        ] {
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=i82_header_fits pbs={pbs} hdr_tag={tag} hdr={hdr} fits={}",
                    u8::from(hdr <= pbs)
                ))
            );
        }
    }

    // ── 最坏事务占用与环下界（I-8.1）──────────────────────────────────
    // 事务尺寸取 E45 扫过的那几档，好与它已发表的表逐格对照。
    for items in [1u64, 4, TARGET_ITEMS, 64, 200, 1024, 62_000] {
        for rec_kib in [4u64, 32] {
            let rec = rec_kib * 1024;
            for unit in [512u64, 4096] {
                let cap = items_per_record(rec, hdr_full);
                let n = records_for(items, cap);
                let occ = n.map(|n| laid_out_bytes(n, rec, unit, hdr_full));
                let floor = ring_floor(items, rec, unit, hdr_full);
                println!(
                    "{}",
                    em.emit_raw(&format!(
                        "name=worst_txn items={items} rec_kib={rec_kib} unit={unit} \
                         records={} occupancy={} ring_floor={} ring_floor_kib={}",
                        n.map_or("NA".into(), |v| v.to_string()),
                        occ.map_or("NA".into(), |v| v.to_string()),
                        floor.map_or("NA".into(), |v| v.to_string()),
                        floor.map_or("NA".into(), |v| (v / 1024).to_string())
                    ))
                );
            }
        }
    }

    // ── 跨记录事务占比：1..=200 项的事务里有多少条装不进一条记录 ──────────
    for rec_kib in [1u64, 2, 4, 8, 32] {
        let rec = rec_kib * 1024;
        let cap = items_per_record(rec, hdr_full);
        let spanning = (1..=200u64)
            .filter(|&i| records_for(i, cap).is_some_and(|n| n > 1))
            .count() as u64;
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=span_share rec_kib={rec_kib} cap={cap} \
                 spanning_of_200={spanning} share_ppm={}",
                spanning * 1_000_000 / 200
            ))
        );
    }

    // ── 阳性对照：200 项在 4 KiB 上必须跨记录 ────────────────────────────
    let cap4k = items_per_record(4096, hdr_full);
    let n200 = records_for(200, cap4k);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=positive_control_200_spans cap={cap4k} records={} spans={}",
            n200.map_or("NA".into(), |v| v.to_string()),
            n200.map_or("NA".into(), |v| u8::from(v > 1).to_string())
        ))
    );

    // ── 阳性对照 2：对齐逻辑本身有没有判别力 ─────────────────────────────
    // 2 的幂记录尺寸下 I-8.2 恒满足 ⇒ 主扫里一个字节的对齐浪费都看不到。
    // 拿一个非 2 的幂尺寸逼它出现，否则「对齐浪费恒 0」分不清是结论还是死代码。
    let packed = laid_out_bytes(6, 100, 512, JOURNAL_HDR);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=positive_control_alignment rec=100 unit=512 n=6 \
             bytes={packed} naive={} padded={}",
            6 * 100,
            u8::from(packed > 600)
        ))
    );
    // ── 阴性对照：单元 = 1 字节 ⇒ 对齐浪费必须恰好为 0 ───────────────────
    let no_align = laid_out_bytes(6, 100, 1, JOURNAL_HDR);
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=negative_control_unit1 bytes={no_align} expect=600"
        ))
    );

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 的绝对值断言**：D23 已定项 12 逐字写的 `(4096 − 99) / 56 = 71`。
    /// 这一格是那条定案的全部算术依据，独立复算一次。
    #[test]
    fn absolute_d23_item12_arithmetic_is_71() {
        assert_eq!((4096 - 99) / 56, 71, "手算：D23 已定项 12 的式子");
        assert_eq!(items_per_record(4096, HDR_AS_CITED_BY_D23_ITEM12), 71);
    }

    /// **绝对值断言**：4 KiB 那一档，四个头部口径**全部**给出 71 项。
    /// ⇒ D23 已定项 12 引的那个 99 虽然是旧口径，**它的结论不受影响**。
    #[test]
    fn absolute_4kib_capacity_is_71_under_all_four_header_readings() {
        for hdr in [78u64, 91, 95, 99] {
            assert_eq!(items_per_record(4096, hdr), 71, "hdr={hdr}");
        }
    }

    /// **绝对值断言**：2 KiB 那一档口径就分岔了——现行的 78 给 35，
    /// 而把三笔已定待落地的增量算进去（91 / 95）与 D23 已定项 12 引的 99 都给 34。
    /// ⇒ **78 在 2 KiB 上多报一个点名项**；「哪个头部口径是权威」在别的记录尺寸上
    /// 是有后果的，不是纯文字问题。
    #[test]
    fn absolute_2kib_capacity_splits_by_header_reading() {
        assert_eq!(items_per_record(2048, 78), 35);
        assert_eq!(items_per_record(2048, 91), 34);
        assert_eq!(items_per_record(2048, 95), 34);
        assert_eq!(items_per_record(2048, 99), 34);
        assert_eq!((2048 - 78) / 56, 35, "手算");
        assert_eq!((2048 - 91) / 56, 34, "手算");
        assert_eq!((2048 - 99) / 56, 34, "手算");
    }

    /// **绝对值断言**：全尺寸容量表，逐格手算钉死。
    #[test]
    fn absolute_capacity_table_full_header() {
        // hdr = 95：(1024−95)/56=16、(2048−95)/56=34（余 49）、(4096−95)/56=71、
        //           (8192−95)/56=144、(32768−95)/56=583
        assert_eq!(items_per_record(1024, 95), 16);
        assert_eq!(items_per_record(2048, 95), 34);
        assert_eq!(items_per_record(4096, 95), 71);
        assert_eq!(items_per_record(8192, 95), 144);
        assert_eq!(items_per_record(32768, 95), 583);
    }

    /// **判据 2**：4 KiB 上目标负载（12 项）恰好占一条记录，跨记录数 0。
    #[test]
    fn criterion2_target_load_fits_one_record_at_4kib() {
        for hdr in [78u64, 91, 95, 99] {
            let cap = items_per_record(4096, hdr);
            assert_eq!(records_for(TARGET_ITEMS, cap), Some(1), "hdr={hdr}");
        }
        // 余量：71 / 12 = 5.9 倍，D23 已定项 12 逐字写的那个数。
        assert_eq!(items_per_record(4096, 95) / TARGET_ITEMS, 5);
        assert!(items_per_record(4096, 95) as f64 / TARGET_ITEMS as f64 > 5.9);
    }

    /// **判据 3**：记录头完整落在一个 pbs 内，512 与 4096 各验一次。
    #[test]
    fn criterion3_header_fits_in_one_atomic_unit() {
        for pbs in [512u64, 4096] {
            for hdr in [78u64, 91, 95] {
                assert!(hdr <= pbs, "hdr={hdr} pbs={pbs}");
                assert!(!header_straddles(0, pbs, hdr));
            }
        }
        // 反向：头比单元还宽时必须判跨。**这一条让上面那组不是恒真式。**
        assert!(header_straddles(0, 64, 95));
    }

    /// **阳性对照**：200 项在 4 KiB 上必须跨记录，且恰好 3 条。
    #[test]
    fn positive_control_200_items_span_three_records() {
        let cap = items_per_record(4096, 95);
        assert_eq!(cap, 71);
        assert_eq!(records_for(200, cap), Some(3), "ceil(200/71)=3");
        assert!(records_for(200, cap).unwrap() > 1);
    }

    /// **与 E45 已发表的表逐格对照**：12 项事务、一事务一条、4096 单元
    /// ⇒ 占用 4096 B、I-8.1 下环至少 8 KiB。E45 那张表的第一行。
    #[test]
    fn absolute_matches_e45_published_ring_floor() {
        assert_eq!(ring_floor(12, 4096, 4096, 95), Some(8192));
        assert_eq!(ring_floor(12, 4096, 512, 95), Some(8192));
        assert_eq!(laid_out_bytes(1, 4096, 4096, 95), 4096);
    }

    /// **I-8.2 按字面实现**：起始偏移只要头装得下就合法，不要求整条记录对齐。
    /// 这一条正是 E45 结论作废的病灶，钉死免得再犯。
    #[test]
    fn i82_is_literal_not_whole_record_alignment() {
        // 偏移 400、单元 512、头 95：400 + 95 = 495 ≤ 512 ⇒ 合法，不许推到 512。
        assert!(!header_straddles(400, 512, 95));
        // 偏移 500：500 + 95 = 595 > 512 ⇒ 跨了，必须推。
        assert!(header_straddles(500, 512, 95));
        // E45 的严规则会把 12 条 153 字节记录摆成 12×512=6144；按字面只要 1836。
        assert_eq!(laid_out_bytes(12, 153, 4096, 97), 1836);
    }

    /// **阳性对照 2**：对齐逻辑有判别力——非 2 的幂尺寸上必须真的padding。
    /// 6 条 100 字节记录、单元 512、头 78：第 6 条起于 500，500+78=578>512 ⇒ 推到 512，
    /// 总占用 512 + 100 = 612，比裸的 600 多 12。
    #[test]
    fn positive_control_alignment_actually_pads() {
        assert_eq!(laid_out_bytes(6, 100, 512, JOURNAL_HDR), 612);
        assert!(laid_out_bytes(6, 100, 512, JOURNAL_HDR) > 6 * 100);
    }

    /// **阴性对照**：单元 = 1 字节 ⇒ 对齐浪费恰好 0。
    #[test]
    fn negative_control_unit_one_has_zero_padding() {
        assert_eq!(laid_out_bytes(6, 100, 1, JOURNAL_HDR), 600);
        assert_eq!(laid_out_bytes(71, 4096, 1, 95), 71 * 4096);
    }

    /// 2 的幂记录尺寸 + 2 的幂单元 ⇒ 对齐浪费恒 0。**这是本实验的一个结论，钉死它。**
    #[test]
    fn power_of_two_record_sizes_never_pad() {
        for rec_kib in [1u64, 2, 4, 8, 32] {
            for unit in [512u64, 4096] {
                let rec = rec_kib * 1024;
                assert_eq!(
                    laid_out_bytes(7, rec, unit, 95),
                    7 * rec,
                    "rec={rec} unit={unit}"
                );
            }
        }
    }

    /// 头装不下时返回 0，而 `records_for` 必须把它报成 None，不许退化成「1 条」。
    #[test]
    fn zero_capacity_is_not_a_measurement() {
        assert_eq!(items_per_record(64, 95), 0);
        assert_eq!(records_for(12, 0), None);
        assert_eq!(ring_floor(12, 64, 512, 95), None);
    }

    /// 空事务（0 项）仍要占一条记录——记录头本身要落盘。
    #[test]
    fn empty_transaction_still_takes_one_record() {
        assert_eq!(records_for(0, 71), Some(1));
    }

    /// 三笔已定待落地的增量加起来是 17 字节，78 + 17 = 95。
    #[test]
    fn absolute_pending_header_increments_sum_to_17() {
        assert_eq!(INC_TXN_BOUNDARY + INC_BACK_CHAIN + INC_PAYLOAD_CSUM, 17);
        assert_eq!(JOURNAL_HDR + INC_TXN_BOUNDARY + INC_BACK_CHAIN, 91);
        assert_eq!(
            JOURNAL_HDR + INC_TXN_BOUNDARY + INC_BACK_CHAIN + INC_PAYLOAD_CSUM,
            95
        );
    }

    /// 格式常量必须与 kb 的 format-const 标记一致。
    #[test]
    fn format_constants_match_kb() {
        assert_eq!(JOURNAL_HDR, 78, "D23 已定项 4 的 format-const 标记");
        assert_eq!(ITEM, 56, "E23 字段表的点名项宽度");
    }
}
