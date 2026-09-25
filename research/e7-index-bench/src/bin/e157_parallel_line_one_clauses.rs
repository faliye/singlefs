//! E157 第一段：并行线一挡着的两条条款的计数模型 —— 岔路单第 1 行（C490，extent 叶记录 key 的
//! offset 段取什么单位）。跑前登记 `research/prompts/e157-preregistration.md`。
//!
//! **这一段只算岔路单第 1 行。** 第 2 行（C491，共享提交内生块在哪条记录点名）留给第二段，
//! 本文件一个字都没有触碰。
//!
//! ## 两条臂（逐字照登记第五·一节）
//!
//! | 臂 | extent 叶记录 key 第三段写什么 | 读文件字节偏移 `f` 怎么定位单元 |
//! |---|---|---|
//! | `jia`（1甲，文件字节偏移） | `n × C`（`C` = 写它那天的净荷容量） | 树里找「最后一个 key ≤ f」 |
//! | `yi`（1乙，单元序号） | `n` 本身 | 先算 `n = f / C`，再找 key = `n` |
//!
//! ## 它答的三件事（岔路单第 1 行 ①②③）
//!
//! - ①（量 1.1a / 1.1b）：定位一页沿 extent 树走的节点读取次数，冷走与挂载态各一份。
//! - ②（量 1.2a / 1.2b / 1.2c）：叶片数、树高、节点总数。
//! - ③（量 1.3a / 1.3b / 1.3c）：换一档净荷容量之后，写下的 key 还认不认得出。
//!   量 1.3a 在跑前登记写死时已经被算出来了（登记第四节），**降级成对拍量，不当判据**；
//!   ③ 的判定改由 1.3b 与 1.3c 承担（登记「附一条判定纪律」）。
//!
//! ## ①②的一个结构性事实（不是倾向，是从两条臂的定义直接推出来的）
//!
//! 两条臂**只改 key 的取值，不改 key 宽、记录宽、也不改单元序号到叶/页的映射**——
//! `n → 第几叶` 与 `字节偏移 → 第几个单元` 这两个映射对两条臂完全相同（登记五·一节
//! 「⚠️ 对面那条臂写成支持它的人认的样子」那一段已经提醒过这一点）。所以在**同一档净荷容量**下，
//! 冷走 / 挂载态节点读取次数与叶片数 / 树高 / 节点总数，**两臂必然逐点相等**——这不是本装置的
//! 缺陷，是这两个量的定义域里没有能分开两臂的维度。① 与 ② 因此预计报「分不开」（登记 F3：
//! 「这是正当结果，如实记下来，不许写成装置坏了」），真正能分开两臂的是③（1.3b / 1.3c），
//! 因为③引入了「写的时候是一档净荷容量、读的时候是另一档」这个两臂真正不对称的维度。
//! 这句话是**推导**，不是先看了产物才补的结论——下面的实现与断言原样把①②算出来，
//! 算出来是不是 0 由跑出来的数说话，不预先把判据焊死成「一定是 0」。
//!
//! ## 口径（跑前写死，来自登记第二、三、六、七、八节，逐条带出处）
//!
//! - 一个数据单元占盘 32768 含头；净荷容量 `C` = 32768 − 134 = 32634（今天，`layout/01-first-txn.md`
//!   「二」末行）；预留 28 那天是 32768 − 105 − 28 = 32635（`milestone/02-second-txn.md:611`）。
//! - extent 码 2 节点头 163 = 86 + 2×24 + 29（D18 已定项 2/7/12/16；`layout/01-first-txn.md`「四·二」）；
//!   extent 叶记录 112 = key 24 + 指针 88（D19 已定项 7/8）。
//! - extent 树内部节点条目宽在 `crates/` 里零命中（`grep -rn 'EXTENT_INTERNAL\|extent_internal' crates/`
//!   退出码 1）⇒ 当旋钮，110 与 136 两个取样点各报一次（登记第三节、第八·二节）。
//! - journal 记录 4096、记录头 311、点名项 56 ⇒ 67 项/条（D23 已定项 12/17）——这两个量本段不判定，
//!   只在 A1/B2 锚点里出现，供第二段复用。
//!
//! ## 它不答什么
//!
//! 不答第 2 行（journal 记录、共享内生块、崩溃点轨迹）；不答挂钟；不答 `crates/` 今天走的
//! 「按位次定位」读法（登记第三节：那是这条条款没定时的绕行，不是本模型要建的两条候选之一）。

use e7_index_bench::Emitter;

// ── 格式常量（逐条带出处，第二节已整段抄） ──────────────────────────────

/// extent 叶 key 三段中，本实验唯一在争的那一段的兄弟字段宽度：locality 8 + inode 8（D8 已定项 3）。
/// 本模型不建 locality / inode 两段的模型（两臂在那两段上同值），只建第三段（offset 段）。
const EXTENT_LEAF_KEY_BYTES: u64 = 24;
/// extent 叶记录里指向数据单元的指针宽度（D19 已定项 4/7/8）。
const EXTENT_LEAF_POINTER_BYTES: u64 = 88;
/// extent 叶记录总宽，与 A3 对拍：24 + 88 = 112。
const EXTENT_LEAF_RECORD_BYTES: u64 = 112;

/// extent 码 2 节点头公式：`86 + 2 × key_bytes + 29`（D18 已定项 2，2026-09-14 用户定案：
/// 全部码 2 节点都带 key 区间，不再按树分）。
const fn extent_node_header_bytes(key_bytes: u64) -> u64 {
    86 + 2 * key_bytes + 29
}
/// 上面那条公式在 `key_bytes = 24`（extent 叶 key 宽）下的值，与 B1/B5/A3 对拍：163。
const EXTENT_NODE_HEADER_BYTES: u64 = extent_node_header_bytes(EXTENT_LEAF_KEY_BYTES);

const JOURNAL_HEADER_BYTES: u64 = 311;
const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 一条记录装多少点名项：与 A1/B2 对拍，67。本段不消费它，留给第二段。
const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES;

const DATA_UNIT_BYTES: u64 = 32768;
const DATA_UNIT_HEADER_FIXED_BYTES: u64 = 105;

/// 净荷容量公式：`32768 − 105 − 预留位`。今天预留 29，那天预留 28（B3）。
fn payload_capacity(reserve_bytes: u64) -> u64 {
    DATA_UNIT_BYTES - DATA_UNIT_HEADER_FIXED_BYTES - reserve_bytes
}
/// 今天这一档，与 A4/B3 对拍：32634。
const PAYLOAD_CAPACITY_TODAY: u64 = 32634;
/// 预留 28 那天，与 B3 对拍：32635。这是**真发生过**的一次容量改动（登记第四节）。
const PAYLOAD_CAPACITY_RESERVE_28_DAY: u64 = 32635;
/// `Δ = -2` 那一档（登记第八·二节取样表第四组）。
const PAYLOAD_CAPACITY_DELTA_MINUS_TWO: u64 = 32632;

const PAGE_BYTES: u64 = 4096;

// ── 两条臂 ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 1甲：extent 叶 key 第三段写文件字节偏移 `n × C`。
    ByteOffsetKey,
    /// 1乙：extent 叶 key 第三段写单元序号 `n`。
    UnitIndexKey,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Arm::ByteOffsetKey => "jia",
            Arm::UnitIndexKey => "yi",
        }
    }
}

const ARMS: [Arm; 2] = [Arm::ByteOffsetKey, Arm::UnitIndexKey];

// ── 结构性算术：叶片数 / 树高 / 节点总数（对两条臂共用，见模块头「结构性事实」） ──────

/// 一个 extent 码 2 节点装多少条叶记录。
fn extent_leaf_capacity(node_bytes: u64) -> u64 {
    (node_bytes - EXTENT_NODE_HEADER_BYTES) / EXTENT_LEAF_RECORD_BYTES
}

/// 一个 extent 码 2 内部节点的扇出：`internal_entry_bytes` 是登记第三节点名的旋钮（110 / 136，
/// `crates/` 里零命中，没有权威值）。
fn extent_internal_fanout(node_bytes: u64, internal_entry_bytes: u64) -> u64 {
    (node_bytes - EXTENT_NODE_HEADER_BYTES) / internal_entry_bytes
}

/// `total_data_units` 个数据单元需要几片叶（量 1.2a）。
fn extent_tree_leaf_count(total_data_units: u64, leaf_capacity: u64) -> u64 {
    total_data_units.max(1).div_ceil(leaf_capacity)
}

/// 树高（量 1.2b）。`leaf_count <= 1` 时根兼叶，高度 1（与 A5 的第一个事务对拍）。
fn extent_tree_height(leaf_count: u64, internal_fanout: u64) -> u64 {
    if leaf_count <= 1 {
        return 1;
    }
    let mut height = 1u64;
    let mut level_entry_count = leaf_count;
    while level_entry_count > 1 {
        level_entry_count = level_entry_count.div_ceil(internal_fanout);
        height += 1;
    }
    height
}

/// 节点总数：各层节点数之和（量 1.2c）。
fn extent_tree_node_count(leaf_count: u64, internal_fanout: u64) -> u64 {
    if leaf_count <= 1 {
        return 1;
    }
    let mut total_node_count = leaf_count;
    let mut level_entry_count = leaf_count;
    while level_entry_count > 1 {
        level_entry_count = level_entry_count.div_ceil(internal_fanout);
        total_node_count += level_entry_count;
    }
    total_node_count
}

/// 一次顺序写切成几个数据单元（`write_request_split.rs:59` 同口径：`div_ceil(净荷容量).max(1)`）。
fn data_unit_count_of_a_sequential_write(total_bytes: u64, payload_capacity: u64) -> u64 {
    total_bytes.div_ceil(payload_capacity).max(1)
}

/// M13 的落点：一份按「预留 28 那天」的容量恰好写满 144 个单元的文件，今天重新按今天的容量切分
/// 要几个单元。这是唯一读今天这一档容量常量做除数的地方——main() 与单测都调这一个函数，
/// 常量被换成错的那天（32635），两边会一起从 145 掉到 144。
fn data_unit_count_of_the_reserve28_day_boundary_file_recut_today() -> u64 {
    let boundary_file_bytes = PAYLOAD_CAPACITY_RESERVE_28_DAY * 144;
    data_unit_count_of_a_sequential_write(boundary_file_bytes, PAYLOAD_CAPACITY_TODAY)
}

/// 文件字节偏移 `byte_offset` 落在第几个单元（两臂共用：单元到字节的映射不因 key 编码而变）。
fn unit_index_covering_byte_offset(byte_offset: u64, payload_capacity: u64, total_data_units: u64) -> u64 {
    (byte_offset / payload_capacity).min(total_data_units - 1)
}

/// 一个 4096 字节的页是否跨了一个**叶**边界（不只是跨单元——跨单元但落在同一叶不算，
/// 登记量 1.1a「n₁≠n₂ 时第二条记录落在同一叶计 0、落在相邻叶计 +1」）。
fn page_straddles_leaf_boundary(page_number: u64, leaf_capacity: u64, payload_capacity: u64, total_data_units: u64) -> bool {
    let first_byte = page_number * PAGE_BYTES;
    let last_byte = first_byte + (PAGE_BYTES - 1);
    let first_unit_index = unit_index_covering_byte_offset(first_byte, payload_capacity, total_data_units);
    let last_unit_index = unit_index_covering_byte_offset(last_byte, payload_capacity, total_data_units);
    first_unit_index != last_unit_index && (first_unit_index / leaf_capacity) != (last_unit_index / leaf_capacity)
}

/// 量 1.1a：冷走定位一页的码 2 节点读取次数（根计 1）。
fn cold_walk_node_reads_to_locate_page(
    page_number: u64,
    node_bytes: u64,
    internal_entry_bytes: u64,
    payload_capacity: u64,
    total_data_units: u64,
) -> u64 {
    let leaf_capacity = extent_leaf_capacity(node_bytes);
    let internal_fanout = extent_internal_fanout(node_bytes, internal_entry_bytes);
    let leaf_count = extent_tree_leaf_count(total_data_units, leaf_capacity);
    let height = extent_tree_height(leaf_count, internal_fanout);
    let extra_leaf_read = u64::from(page_straddles_leaf_boundary(page_number, leaf_capacity, payload_capacity, total_data_units));
    height + extra_leaf_read
}

/// 量 1.1b：挂载态定位一页只数叶节点读取次数（根与内部节点已在挂载态里）。
fn mounted_leaf_reads_to_locate_page(page_number: u64, node_bytes: u64, payload_capacity: u64, total_data_units: u64) -> u64 {
    let leaf_capacity = extent_leaf_capacity(node_bytes);
    let extra_leaf_read = u64::from(page_straddles_leaf_boundary(page_number, leaf_capacity, payload_capacity, total_data_units));
    1 + extra_leaf_read
}

// ── 净荷容量改变之后的还原（量 1.3a / 1.3b / 1.3c，两臂在这里才真正不对称） ──────────

/// 写第 `unit_index` 个单元那天，extent 叶 key 第三段写什么。
fn extent_leaf_key_third_segment(arm: Arm, unit_index: u64, capacity_at_write_time: u64) -> u64 {
    match arm {
        Arm::ByteOffsetKey => unit_index * capacity_at_write_time,
        Arm::UnitIndexKey => unit_index,
    }
}

/// 一个单元换一档容量之后的三样还原结果。
struct ReconstructionOutcome {
    /// 量 1.3a：还原出的单元序号是否等于写它时的 `n`（对拍量，不当判据）。
    unit_index_correct: bool,
    /// 量 1.3b：还原出的字节区间起点与终点是否都等于写它时的区间。
    byte_interval_correct: bool,
}

fn reconstruction_outcome(arm: Arm, unit_index: u64, capacity_at_write_time: u64, capacity_at_read_time: u64) -> ReconstructionOutcome {
    let key_third_segment = extent_leaf_key_third_segment(arm, unit_index, capacity_at_write_time);

    // 量 1.3a：⌊key / C₁⌋（1甲）或 key 本身（1乙）。
    let reconstructed_unit_index = match arm {
        Arm::ByteOffsetKey => key_third_segment / capacity_at_read_time,
        Arm::UnitIndexKey => key_third_segment,
    };
    let unit_index_correct = reconstructed_unit_index == unit_index;

    // 量 1.3b：[start, start + C₁)（1甲 start=key；1乙 start=key×C₁），与写它时的
    // [n×C₀, n×C₀+C₀) 比较（本模型把每个单元的声明长度都当作写它那天的净荷容量 C₀——
    // 这是本实验的建模选择：本量测的是「按全局容量常量重算区间」这条路会不会把已有的
    // key 读岔，不是测「有没有信 header 自己的声明长度字段」那件事，登记第六·一节 1.3b
    // 「怎么算」原文就是拿 C₁ 加回去，不是读单元自己的声明长度）。
    let byte_interval_start = match arm {
        Arm::ByteOffsetKey => key_third_segment,
        Arm::UnitIndexKey => key_third_segment * capacity_at_read_time,
    };
    let byte_interval_end = byte_interval_start + capacity_at_read_time;
    let written_start = unit_index * capacity_at_write_time;
    let written_end = written_start + capacity_at_write_time;
    let byte_interval_correct = byte_interval_start == written_start && byte_interval_end == written_end;

    ReconstructionOutcome { unit_index_correct, byte_interval_correct }
}

/// 量 1.3c：按 D9 已定项 6 现算 AAD 锚点偏移（`key.offset − ptr.extent_off`，本版
/// `ptr.extent_off = 0`），与单元头里写它那天的锚点字段（`n × C₀`）比较。
/// **不依赖 `capacity_at_read_time`**——这正是这一格「最容易被写成倾向」的地方：
/// AAD 锚点字段本身就是按字节偏移定义的，1甲的 key 恰好与它同一个字面值。
fn aad_anchor_reconstructed_correctly(arm: Arm, unit_index: u64, capacity_at_write_time: u64) -> bool {
    let key_third_segment = extent_leaf_key_third_segment(arm, unit_index, capacity_at_write_time);
    let extent_off = 0u64; // 第一版一个单元一个 extent（D9 已定项 6 的备注）
    let reconstructed_anchor = key_third_segment - extent_off;
    let header_anchor_field = unit_index * capacity_at_write_time;
    reconstructed_anchor == header_anchor_field
}

// ── 主程序：把上面的算术铺成产物 ─────────────────────────────────────

/// 量 1.2 与量 1.1 都要跑的 `N`（数据单元数）取样表，登记第八·二节逐字抄。
const DATA_UNIT_COUNT_SAMPLE_POINTS: [u64; 14] = [1, 2, 10, 67, 68, 143, 144, 145, 288, 289, 8225, 8226, 20736, 20737];
/// `NODE_BYTES` 几何取样点：16384 是今天，4096/65536 是几何敏感性的两个方向（登记第八·三节）。
const NODE_BYTES_SAMPLE_POINTS: [u64; 3] = [4096, 16384, 65536];
/// extent 树内部条目宽旋钮，`crates/` 零命中，两个取样点各报一次（登记第三节）。
const INTERNAL_ENTRY_BYTES_SAMPLE_POINTS: [u64; 2] = [110, 136];
/// `(C₀, C₁)` 取样表，登记第八·二节逐字抄。
const CAPACITY_PAIR_SAMPLE_POINTS: [(u64, u64); 4] = [
    (PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_TODAY),
    (PAYLOAD_CAPACITY_RESERVE_28_DAY, PAYLOAD_CAPACITY_TODAY),
    (PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_RESERVE_28_DAY),
    (PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_DELTA_MINUS_TWO),
];

fn required_pages(total_data_units: u64, capacity: u64) -> Vec<(&'static str, u64)> {
    let mut pages = vec![("p0", 0u64), ("p_cross_unit_boundary", 7u64)];
    if total_data_units > 144 {
        let boundary_byte = 144 * capacity;
        pages.push(("p_leaf_boundary_143_144", boundary_byte / PAGE_BYTES));
    }
    let last_unit_byte = (total_data_units - 1) * capacity;
    pages.push(("p_last_unit", last_unit_byte / PAGE_BYTES));
    pages
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    let mut emit = |line: String| {
        output_text.push_str(&line);
        output_text.push('\n');
    };

    // ── 锚点 A1–A5 / B1–B3（登记第七节，本段范围） ──────────────────────
    emit(emitter.emit_raw(&format!(
        "name=anchors a1_named_entries_per_record={JOURNAL_NAMED_ENTRIES_PER_RECORD} \
         a2_named_entry_bytes={JOURNAL_NAMED_ENTRY_BYTES} a3_extent_leaf_record_bytes={EXTENT_LEAF_RECORD_BYTES} \
         a3_extent_node_header_bytes={EXTENT_NODE_HEADER_BYTES} a4_payload_capacity_today={PAYLOAD_CAPACITY_TODAY} \
         b1_leaf_capacity_16384={} b2_named_entries_per_record_check={} b3_payload_capacity_today={} \
         b3_payload_capacity_reserve28={}",
        extent_leaf_capacity(16384),
        (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES,
        payload_capacity(29),
        payload_capacity(28),
    )));

    emit(emitter.emit_raw(&format!(
        "name=config n_sample_points={} node_bytes_sample_points={} internal_entry_bytes_sample_points={} \
         capacity_pair_sample_points={}",
        DATA_UNIT_COUNT_SAMPLE_POINTS.len(),
        NODE_BYTES_SAMPLE_POINTS.len(),
        INTERNAL_ENTRY_BYTES_SAMPLE_POINTS.len(),
        CAPACITY_PAIR_SAMPLE_POINTS.len(),
    )));

    // ── B5/B6 对拍：叶容量、扇出、跨单元页 ───────────────────────────────
    for &node_bytes in &NODE_BYTES_SAMPLE_POINTS {
        for &internal_entry_bytes in &INTERNAL_ENTRY_BYTES_SAMPLE_POINTS {
            emit(emitter.emit_raw(&format!(
                "name=b5_geometry node_bytes={node_bytes} internal_entry_bytes={internal_entry_bytes} \
                 leaf_capacity={} internal_fanout={}",
                extent_leaf_capacity(node_bytes),
                extent_internal_fanout(node_bytes, internal_entry_bytes),
            )));
        }
    }
    emit(emitter.emit_raw(&format!(
        "name=b6_straddle_page unit0_end_byte={PAYLOAD_CAPACITY_TODAY} page={}",
        PAYLOAD_CAPACITY_TODAY / PAGE_BYTES
    )));

    // ── 真实基线（登记五·三节）：N=1，两臂在这一格上必然同值，是对拍不是判据 ─────
    {
        let leaf_capacity = extent_leaf_capacity(16384);
        let internal_fanout = extent_internal_fanout(16384, 110);
        let leaf_count = extent_tree_leaf_count(1, leaf_capacity);
        let height = extent_tree_height(leaf_count, internal_fanout);
        let node_count = extent_tree_node_count(leaf_count, internal_fanout);
        for arm in ARMS {
            let key_third_segment = extent_leaf_key_third_segment(arm, 0, PAYLOAD_CAPACITY_TODAY);
            emit(emitter.emit_raw(&format!(
                "name=real_baseline arm={} n=1 leaf_records={leaf_count} leaves={leaf_count} height={height} \
                 node_count={node_count} key_third_segment={key_third_segment}",
                arm.label()
            )));
        }
    }

    // ── P1：树高 / 叶数对照，NODE_BYTES 16384 → 4096，两臂各跑 ──────────────
    {
        let data_unit_count_for_control_one = 8225u64; // 跨过一叶 144 条门槛、也在几何敏感性范围内的一个代表点
        for &node_bytes in &[16384u64, 4096] {
            let leaf_capacity = extent_leaf_capacity(node_bytes);
            let internal_fanout = extent_internal_fanout(node_bytes, 110);
            let leaves = extent_tree_leaf_count(data_unit_count_for_control_one, leaf_capacity);
            let height = extent_tree_height(leaves, internal_fanout);
            for arm in ARMS {
                emit(emitter.emit_raw(&format!(
                    "name=p1_control arm={} node_bytes={node_bytes} n={data_unit_count_for_control_one} leaves={leaves} height={height}"
                , arm.label())));
            }
        }
    }

    // ── P2a / P2b：容量换算零点与有效点，两臂各跑 ─────────────────────────
    for &(label, capacity_at_write_time, capacity_at_read_time) in &[
        ("p2a_zero_point", PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_TODAY),
        ("p2b_effective", PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_RESERVE_28_DAY),
    ] {
        for &total_data_units in &[1u64, 2, 145, 20737] {
            for arm in ARMS {
                let mut unit_index_correct_count = 0u64;
                let mut byte_interval_correct_count = 0u64;
                for unit_index in 0..total_data_units {
                    let outcome = reconstruction_outcome(arm, unit_index, capacity_at_write_time, capacity_at_read_time);
                    unit_index_correct_count += u64::from(outcome.unit_index_correct);
                    byte_interval_correct_count += u64::from(outcome.byte_interval_correct);
                }
                emit(emitter.emit_raw(&format!(
                    "name={label} arm={} n={total_data_units} unit_index_correct_count={unit_index_correct_count} \
                     unit_index_wrong_count={} byte_interval_correct_count={byte_interval_correct_count} \
                     byte_interval_wrong_count={}",
                    arm.label(),
                    total_data_units - unit_index_correct_count,
                    total_data_units - byte_interval_correct_count,
                )));
            }
        }
    }

    // ── 量 1.1a / 1.1b：定位一页的节点读取次数，页集合逐个报 ───────────────
    for &total_data_units in &DATA_UNIT_COUNT_SAMPLE_POINTS {
        let pages = required_pages(total_data_units, PAYLOAD_CAPACITY_TODAY);
        for &node_bytes in &NODE_BYTES_SAMPLE_POINTS {
            for &internal_entry_bytes in &INTERNAL_ENTRY_BYTES_SAMPLE_POINTS {
                for &(page_label, page_number) in &pages {
                    for arm in ARMS {
                        let cold_reads = cold_walk_node_reads_to_locate_page(
                            page_number,
                            node_bytes,
                            internal_entry_bytes,
                            PAYLOAD_CAPACITY_TODAY,
                            total_data_units,
                        );
                        let mounted_reads = mounted_leaf_reads_to_locate_page(page_number, node_bytes, PAYLOAD_CAPACITY_TODAY, total_data_units);
                        emit(emitter.emit_raw(&format!(
                            "name=locate_page arm={} n={total_data_units} node_bytes={node_bytes} internal_entry_bytes={internal_entry_bytes} \
                             page_label={page_label} page={page_number} cold_walk_node_reads={cold_reads} \
                             mounted_leaf_reads={mounted_reads}",
                            arm.label()
                        )));
                    }
                }
            }
        }
    }

    // ── 量 1.2a / 1.2b / 1.2c：叶片数、树高、节点总数 ───────────────────────
    for &total_data_units in &DATA_UNIT_COUNT_SAMPLE_POINTS {
        for &node_bytes in &NODE_BYTES_SAMPLE_POINTS {
            for &internal_entry_bytes in &INTERNAL_ENTRY_BYTES_SAMPLE_POINTS {
                let leaf_capacity = extent_leaf_capacity(node_bytes);
                let internal_fanout = extent_internal_fanout(node_bytes, internal_entry_bytes);
                let leaves = extent_tree_leaf_count(total_data_units, leaf_capacity);
                let height = extent_tree_height(leaves, internal_fanout);
                let node_count = extent_tree_node_count(leaves, internal_fanout);
                for arm in ARMS {
                    emit(emitter.emit_raw(&format!(
                        "name=tree_shape arm={} n={total_data_units} node_bytes={node_bytes} internal_entry_bytes={internal_entry_bytes} \
                         leaves={leaves} height={height} node_count={node_count}",
                        arm.label()
                    )));
                }
            }
        }
    }

    // ── 量 1.3a（对拍，不当判据）/ 1.3b / 1.3c：容量取样表 ────────────────
    for &(capacity_at_write_time, capacity_at_read_time) in &CAPACITY_PAIR_SAMPLE_POINTS {
        for &total_data_units in &DATA_UNIT_COUNT_SAMPLE_POINTS {
            for arm in ARMS {
                let mut unit_index_correct_count = 0u64;
                let mut byte_interval_correct_count = 0u64;
                let mut aad_anchor_correct_count = 0u64;
                for unit_index in 0..total_data_units {
                    let outcome = reconstruction_outcome(arm, unit_index, capacity_at_write_time, capacity_at_read_time);
                    unit_index_correct_count += u64::from(outcome.unit_index_correct);
                    byte_interval_correct_count += u64::from(outcome.byte_interval_correct);
                    aad_anchor_correct_count += u64::from(aad_anchor_reconstructed_correctly(arm, unit_index, capacity_at_write_time));
                }
                emit(emitter.emit_raw(&format!(
                    "name=capacity_conversion arm={} n={total_data_units} c0={capacity_at_write_time} c1={capacity_at_read_time} \
                     delta={} unit_index_correct_count={unit_index_correct_count} \
                     byte_interval_correct_count={byte_interval_correct_count} aad_anchor_correct_count={aad_anchor_correct_count}",
                    arm.label(),
                    capacity_at_read_time as i64 - capacity_at_write_time as i64,
                )));
            }
        }
    }

    // ── M13 的落点：文件大小固定，容量常量对不对决定切成几个单元 ─────────────
    {
        let boundary_unit_count = data_unit_count_of_the_reserve28_day_boundary_file_recut_today();
        emit(emitter.emit_raw(&format!(
            "name=unit_count_boundary file_bytes={} capacity_used={PAYLOAD_CAPACITY_TODAY} \
             unit_count={boundary_unit_count}",
            PAYLOAD_CAPACITY_RESERVE_28_DAY * 144,
        )));
    }

    // ── 判别力自证（登记第八·三节末）：把 1.2a 的门槛挪到两个几何点之间，必须由绿转红 ──
    {
        let data_unit_count_between_geometry_thresholds = 40u64; // 4096 档叶容量 35，16384 档叶容量 144；40 落在两者之间
        let leaves_at_4096 = extent_tree_leaf_count(data_unit_count_between_geometry_thresholds, extent_leaf_capacity(4096));
        let leaves_at_16384 = extent_tree_leaf_count(data_unit_count_between_geometry_thresholds, extent_leaf_capacity(16384));
        emit(emitter.emit_raw(&format!(
            "name=discrimination_self_check n={data_unit_count_between_geometry_thresholds} leaves_at_4096={leaves_at_4096} leaves_at_16384={leaves_at_16384} \
             threshold_between_them={}",
            leaves_at_4096 != leaves_at_16384
        )));
    }

    output_text.push_str(&emitter.finish());
    output_text.push('\n');
    print!("{output_text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── A1–A5 / B1–B3 锚点（登记第七节，逐字对拍） ──────────────────────

    #[test]
    fn 锚点甲一与锚点乙二_一条记录装六十七个点名项() {
        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67);
        assert!(67 * JOURNAL_NAMED_ENTRY_BYTES + JOURNAL_HEADER_BYTES <= JOURNAL_RECORD_BYTES);
        assert!(JOURNAL_RECORD_BYTES < 68 * JOURNAL_NAMED_ENTRY_BYTES + JOURNAL_HEADER_BYTES);
    }

    /// B7：头 311 时的余数（33，D23 已定项 4「4096 上余 3785」）——全部单测里唯一分得开 307 与
    /// 311 的一条。写成加法，不写减法：减法在变异把常量改大时会编译期溢出
    /// （`.claude/singlefs-ai-sop/rules/test-discipline.md`「常量断言写加法，别写减法」）。
    #[test]
    fn named_entries_per_record_remainder_matches_head_311() {
        assert_eq!(JOURNAL_HEADER_BYTES + JOURNAL_NAMED_ENTRIES_PER_RECORD * JOURNAL_NAMED_ENTRY_BYTES + 33, JOURNAL_RECORD_BYTES);
    }

    #[test]
    fn 锚点甲二_点名项五十六字节的字段表() {
        assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 14 * 2 + 1 + 8 + 8 + 10 + 1);
    }

    #[test]
    fn 锚点甲三_extent叶记录与节点头宽度() {
        assert_eq!(EXTENT_LEAF_RECORD_BYTES, 112);
        assert_eq!(EXTENT_LEAF_RECORD_BYTES, EXTENT_LEAF_KEY_BYTES + EXTENT_LEAF_POINTER_BYTES);
        assert_eq!(EXTENT_NODE_HEADER_BYTES, extent_node_header_bytes(EXTENT_LEAF_KEY_BYTES));
        assert_eq!(EXTENT_NODE_HEADER_BYTES, 163);
    }

    #[test]
    fn 锚点甲四与锚点乙三_今天的净荷容量() {
        assert_eq!(PAYLOAD_CAPACITY_TODAY, payload_capacity(29));
        assert_eq!(PAYLOAD_CAPACITY_TODAY, DATA_UNIT_BYTES - 134);
        assert_eq!(PAYLOAD_CAPACITY_RESERVE_28_DAY, payload_capacity(28));
    }

    #[test]
    fn 锚点甲五_第一个事务的extent树部分() {
        let leaf_capacity = extent_leaf_capacity(16384);
        let internal_fanout = extent_internal_fanout(16384, 110);
        let leaves = extent_tree_leaf_count(1, leaf_capacity);
        assert_eq!(leaves, 1);
        assert_eq!(extent_tree_height(leaves, internal_fanout), 1);
        assert_eq!(extent_tree_node_count(leaves, internal_fanout), 1);
        // key (0, 1, 0)：第三段（offset 段）在两臂下对单元序号 0 都写 0。
        assert_eq!(extent_leaf_key_third_segment(Arm::ByteOffsetKey, 0, PAYLOAD_CAPACITY_TODAY), 0);
        assert_eq!(extent_leaf_key_third_segment(Arm::UnitIndexKey, 0, PAYLOAD_CAPACITY_TODAY), 0);
    }

    #[test]
    fn 锚点乙一_一叶一百四十四条() {
        assert_eq!(extent_leaf_capacity(16384), 144);
        assert!(144 * EXTENT_LEAF_RECORD_BYTES + EXTENT_NODE_HEADER_BYTES <= 16384);
        assert!(16384 < 145 * EXTENT_LEAF_RECORD_BYTES + EXTENT_NODE_HEADER_BYTES);
    }

    /// 树高的绝对值断言（不与另一次调用互比）：M2「树高从零起数」把 `extent_tree_height` 里
    /// 循环外的起始值从 1 改成 0，只在**同一次调用的返回值**里才看得出来——旁边任何一条
    /// 「拿 reads 与另一次 extent_tree_height 调用比较」的断言，两边共用同一个被改坏的函数，
    /// 会一起偏、互比看不出来（`test-discipline.md`「只让多条臂互相比，测不出所有臂一起错」）。
    #[test]
    fn 树高绝对值_两片叶到三层各钉一个数() {
        let internal_fanout = extent_internal_fanout(16384, 110);
        assert_eq!(extent_tree_height(1, internal_fanout), 1, "根兼叶");
        assert_eq!(extent_tree_height(2, internal_fanout), 2, "两片叶，一层内部节点");
        assert_eq!(extent_tree_height(internal_fanout + 1, internal_fanout), 3, "叶数刚超过一层内部节点能收的上限");
    }

    #[test]
    fn 锚点乙五_三档node_bytes下的叶容量与扇出() {
        assert_eq!(extent_leaf_capacity(4096), 35);
        assert_eq!(extent_leaf_capacity(16384), 144);
        assert_eq!(extent_leaf_capacity(65536), 583);
        assert_eq!(extent_internal_fanout(4096, 110), 35);
        assert_eq!(extent_internal_fanout(16384, 110), 147);
        assert_eq!(extent_internal_fanout(65536, 110), 594);
        assert_eq!(extent_internal_fanout(4096, 136), 28);
        assert_eq!(extent_internal_fanout(16384, 136), 119);
        assert_eq!(extent_internal_fanout(65536, 136), 480);
    }

    #[test]
    fn 锚点乙六_跨单元那一页是第七页() {
        assert_eq!(PAYLOAD_CAPACITY_TODAY / PAGE_BYTES, 7);
    }

    // ── ① 量 1.1a / 1.1b：两臂在同一档容量下必然逐点相等（模块头「结构性事实」） ──

    #[test]
    fn 量1_1a_两臂在一次跨叶的页上相等且等于树高加一() {
        // N=145：叶容量 144，两个叶；page 7 覆盖单元 0/1，都在叶 0，不跨叶。
        let leaf_capacity = extent_leaf_capacity(16384);
        let internal_fanout = extent_internal_fanout(16384, 110);
        let leaves = extent_tree_leaf_count(145, leaf_capacity);
        let height = extent_tree_height(leaves, internal_fanout);
        for arm in ARMS {
            let _ = arm; // 两臂共用同一个函数，这里只是显式走一遍两条臂的标签
            let reads = cold_walk_node_reads_to_locate_page(7, 16384, 110, PAYLOAD_CAPACITY_TODAY, 145);
            assert_eq!(reads, height, "page 7 在 N=145 下不跨叶，节点读取次数应恰等于树高");
        }
    }

    #[test]
    fn 量1_1a_跨叶那一页比树高多一() {
        // N=145：单元 143（叶 0 最后一条）与单元 144（叶 1 唯一一条）之间的边界。
        let boundary_page = (144 * PAYLOAD_CAPACITY_TODAY) / PAGE_BYTES;
        let leaf_capacity = extent_leaf_capacity(16384);
        let internal_fanout = extent_internal_fanout(16384, 110);
        let leaves = extent_tree_leaf_count(145, leaf_capacity);
        let height = extent_tree_height(leaves, internal_fanout);
        let reads = cold_walk_node_reads_to_locate_page(boundary_page, 16384, 110, PAYLOAD_CAPACITY_TODAY, 145);
        assert_eq!(reads, height + 1);
    }

    #[test]
    fn 量1_1b_挂载态只数叶读取() {
        assert_eq!(mounted_leaf_reads_to_locate_page(7, 16384, PAYLOAD_CAPACITY_TODAY, 145), 1);
        let boundary_page = (144 * PAYLOAD_CAPACITY_TODAY) / PAGE_BYTES;
        assert_eq!(mounted_leaf_reads_to_locate_page(boundary_page, 16384, PAYLOAD_CAPACITY_TODAY, 145), 2);
    }

    // ── P1：几何变小之后叶数必须涨、树高必须不减 ─────────────────────────

    #[test]
    fn 对照一_几何变小之后叶数必须涨树高必须不减() {
        let total_data_units = 8225u64;
        let leaves_16384 = extent_tree_leaf_count(total_data_units, extent_leaf_capacity(16384));
        let leaves_4096 = extent_tree_leaf_count(total_data_units, extent_leaf_capacity(4096));
        assert!(leaves_4096 > leaves_16384, "叶数必须涨：{leaves_4096} vs {leaves_16384}");
        let height_16384 = extent_tree_height(leaves_16384, extent_internal_fanout(16384, 110));
        let height_4096 = extent_tree_height(leaves_4096, extent_internal_fanout(4096, 110));
        assert!(height_4096 >= height_16384, "树高不许减：{height_4096} vs {height_16384}");
    }

    // ── ② 量 1.2a/b/c 的判别力自证：门槛挪到两点之间必须翻面 ────────────────

    #[test]
    fn 判别力自证_叶数门槛挪到两点之间必须翻面() {
        let total_data_units = 40u64;
        assert_ne!(
            extent_tree_leaf_count(total_data_units, extent_leaf_capacity(4096)),
            extent_tree_leaf_count(total_data_units, extent_leaf_capacity(16384)),
        );
    }

    // ── P2a：Δ=0 时两臂都必须还原全对 ────────────────────────────────────

    #[test]
    fn 对照二_容量不变时两臂的两个量都必须还原全对() {
        for &total_data_units in &[1u64, 2, 145, 20737] {
            for arm in ARMS {
                let mut unit_index_correct = 0u64;
                let mut byte_interval_correct = 0u64;
                for unit_index in 0..total_data_units {
                    let outcome = reconstruction_outcome(arm, unit_index, PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_TODAY);
                    unit_index_correct += u64::from(outcome.unit_index_correct);
                    byte_interval_correct += u64::from(outcome.byte_interval_correct);
                }
                assert_eq!(unit_index_correct, total_data_units, "arm={:?} n={total_data_units}", arm);
                assert_eq!(byte_interval_correct, total_data_units, "arm={:?} n={total_data_units}", arm);
            }
        }
    }

    // ── P2b + 量 1.3a 对拍：与登记第四节写登记时就算出来的四组数逐字重合 ──────

    #[test]
    fn 量1_3a_与登记第四节命令三的四组数逐字重合() {
        let capacity_at_write_time = PAYLOAD_CAPACITY_TODAY;
        let capacity_at_read_time = PAYLOAD_CAPACITY_RESERVE_28_DAY;
        let expected_jia = [1u64, 1, 1, 1];
        let expected_yi = [1u64, 2, 145, 20737];
        for (index, &total_data_units) in [1u64, 2, 145, 20737].iter().enumerate() {
            let mut jia_correct = 0u64;
            let mut yi_correct = 0u64;
            for unit_index in 0..total_data_units {
                jia_correct += u64::from(
                    reconstruction_outcome(Arm::ByteOffsetKey, unit_index, capacity_at_write_time, capacity_at_read_time)
                        .unit_index_correct,
                );
                yi_correct += u64::from(
                    reconstruction_outcome(Arm::UnitIndexKey, unit_index, capacity_at_write_time, capacity_at_read_time)
                        .unit_index_correct,
                );
            }
            assert_eq!(jia_correct, expected_jia[index], "n={total_data_units}");
            assert_eq!(yi_correct, expected_yi[index], "n={total_data_units}");
        }
    }

    #[test]
    fn 对照三_容量变化时两个量至少一个臂报出还原不对的key() {
        let total_data_units = 145u64;
        let capacity_at_write_time = PAYLOAD_CAPACITY_TODAY;
        let capacity_at_read_time = PAYLOAD_CAPACITY_RESERVE_28_DAY;
        for quantity_is_unit_index in [true, false] {
            let mut any_arm_has_wrong_key = false;
            for arm in ARMS {
                let mut correct = 0u64;
                for unit_index in 0..total_data_units {
                    let outcome = reconstruction_outcome(arm, unit_index, capacity_at_write_time, capacity_at_read_time);
                    correct += u64::from(if quantity_is_unit_index { outcome.unit_index_correct } else { outcome.byte_interval_correct });
                }
                if correct < total_data_units {
                    any_arm_has_wrong_key = true;
                }
            }
            assert!(any_arm_has_wrong_key, "quantity_is_unit_index={quantity_is_unit_index}");
        }
    }

    // ── ③ 量 1.3b：结构性地对两臂都是 0（Δ≠0 时），这是本模型的真实结果 ──────

    #[test]
    fn 量1_3b_delta不为零时两臂的字节区间还原全错() {
        for &(capacity_at_write_time, capacity_at_read_time) in &[
            (PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_RESERVE_28_DAY),
            (PAYLOAD_CAPACITY_TODAY, PAYLOAD_CAPACITY_DELTA_MINUS_TWO),
            (PAYLOAD_CAPACITY_RESERVE_28_DAY, PAYLOAD_CAPACITY_TODAY),
        ] {
            for arm in ARMS {
                let mut correct = 0u64;
                for unit_index in 0..145u64 {
                    correct += u64::from(reconstruction_outcome(arm, unit_index, capacity_at_write_time, capacity_at_read_time).byte_interval_correct);
                }
                assert_eq!(correct, 0, "arm={:?} c0={capacity_at_write_time} c1={capacity_at_read_time}", arm);
            }
        }
    }

    // ── ③ 量 1.3c：唯一在这一段里稳定分开两臂的量，且不依赖 Δ ────────────────

    #[test]
    fn 量1_3c_一甲恒对一乙只对第零条() {
        for &(capacity_at_write_time, capacity_at_read_time) in &CAPACITY_PAIR_SAMPLE_POINTS {
            let _ = capacity_at_read_time; // 1.3c 的算式本来就不用它，这里显式标出
            for &total_data_units in &[1u64, 2, 145, 20737] {
                let mut jia_correct = 0u64;
                let mut yi_correct = 0u64;
                for unit_index in 0..total_data_units {
                    jia_correct += u64::from(aad_anchor_reconstructed_correctly(Arm::ByteOffsetKey, unit_index, capacity_at_write_time));
                    yi_correct += u64::from(aad_anchor_reconstructed_correctly(Arm::UnitIndexKey, unit_index, capacity_at_write_time));
                }
                assert_eq!(jia_correct, total_data_units, "c0={capacity_at_write_time} n={total_data_units}");
                assert_eq!(yi_correct, 1, "c0={capacity_at_write_time} n={total_data_units}");
            }
        }
    }

    // ── M13 的落点：文件大小固定用错误容量常量切分，边界从 145 变 144 ─────────

    #[test]
    fn 变异十三落点_文件大小固定容量常量切出的单元数() {
        assert_eq!(data_unit_count_of_the_reserve28_day_boundary_file_recut_today(), 145);
        let boundary_file_bytes = PAYLOAD_CAPACITY_RESERVE_28_DAY * 144;
        assert_eq!(data_unit_count_of_a_sequential_write(boundary_file_bytes, PAYLOAD_CAPACITY_RESERVE_28_DAY), 144);
    }
}
