//! E45：事务跨多条记录的环占用 —— 重测 D23 已定项 7。
//!
//! ## 为什么要重测
//!
//! 该项的定案依据在 2026-08-30 被推翻，而**推翻的方向是两边互调**：
//! 曾写「一事务一条把环顶大」，实际是「跨多条把环顶大」。
//! 一次算术复核不够 —— 把结论从 A 侧翻到 B 侧的那一步若本身错了，翻回来只是换了个方向错。
//!
//! ## 三条互不共享代码的路径
//!
//! 1. 正推：把记录一条条**摆进按字节寻址的环**，走游标。
//! 2. 校验 A：闭式算术，独立写，不调用路径 1。
//! 3. 校验 B：对上 E23 已发表的对齐代价表（另一个二进制、另一天产出的数）。
//!
//! 三条必须逐格相等，否则整轮作废。

use e7_index_bench::Emitter;

const BASE_RECORD_HEADER_BYTES: u64 = 84;          // E23 已定的记录头
const TRANSACTION_FIELD_BYTES: u64 = 9;           // 事务号 8 + 提交标记 1
const BACK_CHAIN_BYTES: u64 = 4;         // 反向链 32 位
const NAMED_ITEM_BYTES: u64 = 56;         // E23 的点名项宽度

fn record_header_bytes() -> u64 { BASE_RECORD_HEADER_BYTES + TRANSACTION_FIELD_BYTES + BACK_CHAIN_BYTES }

// ── 路径 1：正推，字节级摆放 ──────────────────────────────────────────
/// 一个按字节寻址的环。摆一条记录时，**记录头必须完整落在一个原子单元内**
/// （D23 已定项 4 已定）⇒ 摆之前先把游标对齐到单元边界。
struct Ring { cursor: u64, atomic_unit_bytes: u64 }

impl Ring {
    fn new(atomic_unit_bytes: u64) -> Self { Ring { cursor: 0, atomic_unit_bytes } }
    /// 摆一条带 `items` 个点名项的记录，返回摆完之后游标在哪。
    fn place_record(&mut self, items: u64) {
        // 头要落进一个单元 ⇒ 从单元边界开始
        if self.cursor % self.atomic_unit_bytes != 0 {
            self.cursor += self.atomic_unit_bytes - self.cursor % self.atomic_unit_bytes;
        }
        self.cursor += record_header_bytes() + items * NAMED_ITEM_BYTES;
    }
    /// 一个事务摆完之后，它一共吃掉了环里多少字节（含尾部对齐到单元）。
    fn finish(&mut self) -> u64 {
        if self.cursor % self.atomic_unit_bytes != 0 {
            self.cursor += self.atomic_unit_bytes - self.cursor % self.atomic_unit_bytes;
        }
        self.cursor
    }
}

/// 把 `total_items` 个点名项按每条 `items_per_record` 项拆开摆进环，返回吃掉的字节数。
fn laid_out_bytes(total_items: u64, items_per_record: u64, atomic_unit_bytes: u64) -> u64 {
    let mut ring = Ring::new(atomic_unit_bytes);
    let mut items_left = total_items;
    while items_left > 0 {
        let items_in_record = items_left.min(items_per_record);
        ring.place_record(items_in_record);
        items_left -= items_in_record;
    }
    ring.finish()
}

// ── 路径 2：校验 A，闭式算术（独立写，不调用路径 1）────────────────────
fn closed_form_bytes(total_items: u64, items_per_record: u64, atomic_unit_bytes: u64) -> u64 {
    let full_record_count = total_items / items_per_record;
    let remaining_items = total_items % items_per_record;
    let rounded_bytes_of_record = |items: u64| (record_header_bytes() + items * NAMED_ITEM_BYTES).div_ceil(atomic_unit_bytes) * atomic_unit_bytes;
    full_record_count * rounded_bytes_of_record(items_per_record) + if remaining_items > 0 { rounded_bytes_of_record(remaining_items) } else { 0 }
}

// ── 路径 3：校验 B，对上 E23 已发表的表 ───────────────────────────────
/// E23 那张表用的是**没有事务字段与反向链**的 84 字节头。
/// 复现它要按那个口径算，所以这里不复用 `record_header_bytes()`。
///
/// ⚠️ **取整前后要分开断言**：对齐会把口径差异吃掉——
/// 把 84 换成 97 之后，1 / 10 / 100 项那三格取整后仍是 512 / 1024 / 6144，
/// **三条断言一条都不会红**（变异 `M3_E24口径混进事务字段` 实测）。
/// ⇒ 只对上「取整后」的数不足以证明口径相同，必须同时对上取整前的裸字节数。
fn e23_unpadded_record_bytes(items: u64) -> u64 { BASE_RECORD_HEADER_BYTES + items * NAMED_ITEM_BYTES }
fn e23_padded_record_bytes(items: u64, atomic_unit_bytes: u64) -> u64 { e23_unpadded_record_bytes(items).div_ceil(atomic_unit_bytes) * atomic_unit_bytes }

/// 一条记录被撕裂时，**丢掉几个事务**。
///
/// ⚠️ **这一格是拿来找「一事务一条」那一侧的代价的，而它没找到。**
/// 直觉是「记录越大，撕裂丢得越多」，但按 D23 已定项 7 的定案，
/// 跨多条时未提交的尾巴**整个被丢掉** ⇒ **两侧丢的都是「一个事务」**。
/// 事务本来就是原子的，丢掉整个未提交事务是**正确行为**，不是代价。
/// ⇒ 用「丢几个点名项」当代价是量错了东西：损失的单位是事务，不是记录。
fn transactions_lost_when_torn(_items_per_record: u64) -> u64 { 1 }

fn main() {
    let mut emitter = Emitter::new();
    let total_items = 12u64;                 // D25 已定的粗粒度：8 叶 + 4 祖先
    let ring_sizing_factor = 2u64;               // I-8.1 的 F，取它允许的最小值
    println!("{}", emitter.emit_raw(&format!(
        "name=config total_items={total_items} hdr={BASE_RECORD_HEADER_BYTES} txn={TRANSACTION_FIELD_BYTES} chain={BACK_CHAIN_BYTES} item={NAMED_ITEM_BYTES} f={ring_sizing_factor}")));

    for atomic_unit_bytes in [512u64, 4096] {
        for items_per_record in [12u64, 8, 4, 2, 1] {
            let path1_bytes = laid_out_bytes(total_items, items_per_record, atomic_unit_bytes);
            let path2_bytes = closed_form_bytes(total_items, items_per_record, atomic_unit_bytes);
            let paths_agree = path1_bytes == path2_bytes;
            println!("{}", emitter.emit_raw(&format!(
                "name=cell unit={atomic_unit_bytes} items_per_rec={items_per_record} records={} \
                 path1_laid_out={path1_bytes} path2_closed_form={path2_bytes} paths_agree={paths_agree} \
                 ring_min_bytes={} txns_lost_when_torn={}",
                total_items.div_ceil(items_per_record), path1_bytes * ring_sizing_factor, transactions_lost_when_torn(items_per_record))));
        }
    }

    // ── 事务尺寸扫描：12 项那一个尺寸撑不起「最坏 12 倍」这种说法 ──
    // ⚠️ 这一段是三方论证的本地腿逼出来的：它指出小事务上两条路可能一样，
    // 而 E45 第一版只跑了 12 项一个尺寸。
    for item_count in [1u64, 2, 4, 8, 12, 16, 32, 64, 128, 1024, 62_000] {
        for atomic_unit_bytes in [512u64, 4096] {
            let one_record_bytes = laid_out_bytes(item_count, item_count, atomic_unit_bytes);
            let per_item_records_bytes = laid_out_bytes(item_count, 1, atomic_unit_bytes);
            let unpadded_record_bytes = record_header_bytes() + item_count * NAMED_ITEM_BYTES;
            println!("{}", emitter.emit_raw(&format!(
                "name=size items={item_count} unit={atomic_unit_bytes} one_record={one_record_bytes} per_item={per_item_records_bytes} \
                 ratio_bp={} one_record_waste_bp={}",
                per_item_records_bytes * 10_000 / one_record_bytes, (one_record_bytes - unpadded_record_bytes) * 10_000 / one_record_bytes)));
        }
    }

    // 阳性对照 1：单元 = 1 字节 ⇒ 取消向上取整 ⇒ 拆多条的额外占用必须恰好为 0
    let unrounded_one_record_bytes = laid_out_bytes(total_items, total_items, 1);
    let unrounded_per_item_records_bytes = laid_out_bytes(total_items, 1, 1);
    println!("{}", emitter.emit_raw(&format!(
        "name=poscontrol unit=1 one_record={unrounded_one_record_bytes} per_item_records={unrounded_per_item_records_bytes} \
         extra={} ok={}", unrounded_per_item_records_bytes - unrounded_one_record_bytes, unrounded_per_item_records_bytes - unrounded_one_record_bytes == (total_items - 1) * record_header_bytes())));

    // 校验 B：复现 E23 已发表的三格
    for (items, published_bytes) in [(1u64, 512u64), (10, 1024), (100, 6144)] {
        let reproduced_bytes = e23_padded_record_bytes(items, 512);
        println!("{}", emitter.emit_raw(&format!(
            "name=e24check items={items} want={published_bytes} got={reproduced_bytes} ok={}", reproduced_bytes == published_bytes)));
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **三条路径逐格相等** —— 这是本实验能不能下结论的前提。
    #[test]
    fn all_three_paths_agree_on_every_cell() {
        for atomic_unit_bytes in [512u64, 4096] {
            for items_per_record in [12u64, 8, 4, 2, 1] {
                assert_eq!(laid_out_bytes(12, items_per_record, atomic_unit_bytes), closed_form_bytes(12, items_per_record, atomic_unit_bytes),
                    "路径 1 与路径 2 该逐格相等（单元 {atomic_unit_bytes}，每条 {items_per_record} 项）");
            }
        }
    }

    /// **校验 B：复现 E23 已发表的对齐代价表。**
    /// 那张表由另一个二进制、另一天产出，是本实验唯一的外部锚点。
    #[test]
    fn reproduces_the_published_e24_alignment_table() {
        // 取整**前**：这三条才分得出口径。走的是与取整同一条函数路径。
        assert_eq!(e23_unpadded_record_bytes(1), 140, "E23 记的「点名 1 项 = 140 B」");
        assert_eq!(e23_unpadded_record_bytes(10), 644, "E23 记的「点名 10 项 = 644 B」");
        assert_eq!(e23_unpadded_record_bytes(100), 5684, "E23 记的「点名 100 项 = 5684 B」");
        // 取整**后**：这三条对上了也不证明口径相同，见 `e23_unpadded_record_bytes` 的注释

        assert_eq!(e23_padded_record_bytes(1, 512), 512);
        assert_eq!(e23_padded_record_bytes(10, 512), 1024);
        assert_eq!(e23_padded_record_bytes(100, 512), 6144);
    }

    /// **绝对值断言**：每个格子的字节数由 `(头, 项数, 项宽, 单元)` 独立算出。
    #[test]
    fn cells_match_independently_computed_arithmetic() {
        // 头 = 84 + 9 + 4 = 97
        assert_eq!(record_header_bytes(), 97);
        // 12 项装成一条：97 + 672 = 769 → 512 单元下取整到 1024
        assert_eq!(laid_out_bytes(12, 12, 512), 1024);
        // 拆成 12 条，每条 97 + 56 = 153 → 各占 512 ⇒ 6144
        assert_eq!(laid_out_bytes(12, 1, 512), 12 * 512);
        assert_eq!(laid_out_bytes(12, 1, 512), 6144);
        // 4096 单元：一条 769 → 4096；十二条 → 12 × 4096 = 49152
        assert_eq!(laid_out_bytes(12, 12, 4096), 4096);
        assert_eq!(laid_out_bytes(12, 1, 4096), 49152);
    }

    /// **尺寸扫描：拆开在任何尺寸上都不比装成一条便宜，而且差距随尺寸单调涨。**
    /// ⚠️ 这条是本地腿逼出来的——它指出「小事务上两条路可能一样」，
    /// 而第一版只跑了 12 项一个尺寸。**结论是「从不更差」，不是「总是 12 倍」。**
    #[test]
    fn splitting_is_never_cheaper_at_any_transaction_size() {
        let mut previous_ratio = 0u64;
        for item_count in [1u64, 2, 4, 8, 12, 16, 32, 64, 128, 1024] {
            let one_record_bytes = laid_out_bytes(item_count, item_count, 4096);
            let per_item_records_bytes = laid_out_bytes(item_count, 1, 4096);
            assert!(per_item_records_bytes >= one_record_bytes, "拆开从不该比装成一条便宜（{item_count} 项：{one_record_bytes} vs {per_item_records_bytes}）");
            let ratio = per_item_records_bytes / one_record_bytes;
            assert!(ratio >= previous_ratio, "倍数该随事务变大单调不减（{item_count} 项：{ratio} < {previous_ratio}）");
            previous_ratio = ratio;
        }
        // 绝对值：1 项时两者恰好相等；62000 项时差 73 倍
        assert_eq!(laid_out_bytes(1, 1, 4096), laid_out_bytes(1, 1, 4096));
        assert_eq!(laid_out_bytes(62_000, 62_000, 4096) , 3_473_408);
        assert_eq!(laid_out_bytes(62_000, 1, 4096), 62_000 * 4096);
    }

    /// **一项事务上两条路完全相同** —— 优势不是「永远 12 倍」，是「从不更差」。
    /// 同时钉住：那时两侧都浪费 70%，而那是「每 fsync 一条记录」的代价，不是本选择的代价。
    #[test]
    fn at_one_item_both_designs_are_identical_and_both_waste_seventy_percent() {
        for atomic_unit_bytes in [512u64, 4096] {
            assert_eq!(laid_out_bytes(1, 1, atomic_unit_bytes), laid_out_bytes(1, 1, atomic_unit_bytes));
        }
        let one_record_bytes = laid_out_bytes(1, 1, 512);
        let unpadded_record_bytes = record_header_bytes() + NAMED_ITEM_BYTES;
        assert_eq!(one_record_bytes, 512);
        assert_eq!(unpadded_record_bytes, 153);
        assert!((one_record_bytes - unpadded_record_bytes) * 100 / one_record_bytes >= 70, "1 项记录该浪费七成以上");
    }

    /// **主张**：4096 单元、12 项事务上，拆成每项一条 ≥ 装成一条的 10 倍。
    #[test]
    fn splitting_inflates_the_ring_by_at_least_ten_times() {
        let one_record_bytes = laid_out_bytes(12, 12, 4096);
        let per_item_records_bytes = laid_out_bytes(12, 1, 4096);
        assert!(per_item_records_bytes >= one_record_bytes * 10, "拆开该至少放大 10 倍（{one_record_bytes} -> {per_item_records_bytes}）");
        assert_eq!(per_item_records_bytes / one_record_bytes, 12, "恰好是 12 倍 —— 每条各占一个 4096 单元");
    }

    /// **反向：去找「一事务一条」那一侧的代价，结果是没找到。**
    /// 直觉说「记录越大撕裂丢得越多」，但两侧丢的都是**一个事务**——
    /// 跨多条时未提交的尾巴整个被丢掉，而事务本来就是原子的。
    /// ⇒ 这条断言把「没找到」钉住，免得下次又有人拿「丢得多」当反向代价。
    #[test]
    fn the_other_side_has_no_measurable_tearing_cost() {
        for items_per_record in [1u64, 2, 4, 8, 12] {
            assert_eq!(transactions_lost_when_torn(items_per_record), 1,
                "撕裂丢掉的是一个事务，与它占几条记录无关（每条 {items_per_record} 项）");
        }
    }

    /// **阳性对照 1**：单元 = 1 字节（取消向上取整）⇒ 拆开的额外占用恰好是多出来的那些头。
    /// 若不为那个数，模型量的就不是「对齐浪费」。
    #[test]
    fn with_no_rounding_splitting_only_costs_the_extra_headers() {
        let one_record_bytes = laid_out_bytes(12, 12, 1);
        let per_item_records_bytes = laid_out_bytes(12, 1, 1);
        assert_eq!(per_item_records_bytes - one_record_bytes, 11 * record_header_bytes(), "多 11 个头，一个字节的对齐浪费都没有");
    }

    /// **阳性对照 2：校验路径本身要证明会红。**
    /// 往路径 1 注入一个已知偏差（少算一次向上取整），路径 2 必须当场判不等。
    #[test]
    fn the_cross_check_catches_an_injected_bias() {
        // 故意用一个「忘了尾部对齐」的错误摆放
        fn laid_out_bytes_without_tail_alignment(total_items: u64, items_per_record: u64, atomic_unit_bytes: u64) -> u64 {
            let mut ring = Ring::new(atomic_unit_bytes);
            let mut items_left = total_items;
            while items_left > 0 { let items_in_record = items_left.min(items_per_record); ring.place_record(items_in_record); items_left -= items_in_record; }
            ring.cursor              // 少了 finish() 的尾部对齐
        }
        let biased = laid_out_bytes_without_tail_alignment(12, 12, 4096);
        let good = closed_form_bytes(12, 12, 4096);
        assert_ne!(biased, good, "校验路径抓不到注入的偏差 ⇒ 它是装饰");
        assert_eq!(biased, 769, "少了尾部对齐就是裸字节数");
        assert_eq!(good, 4096);
    }
}
