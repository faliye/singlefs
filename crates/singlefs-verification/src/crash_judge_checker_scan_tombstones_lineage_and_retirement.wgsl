// ───────────────────────── 墓碑与谱系那一遍（walk.rs judge_tombstones_and_the_lineage_of_the_versions）─────────────────────────
// 判到位里「射程里有一部分没判」的几组（walk.rs TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS、RETIREMENT_SCAN_INVARIANTS 与几条容量），都在 D_SCAN_MASK_WORD 那一个字里
const D_SCAN_MASK_WORD: u32 = INVARIANT_I_9_18 / 32u;
const D_TOMBSTONE_AND_LINEAGE_SCAN_MASK: u32 = (1u << (INVARIANT_I_9_18 % 32u)) | (1u << (INVARIANT_I_9_19 % 32u))
    | (1u << (INVARIANT_I_9_20 % 32u)) | (1u << (INVARIANT_I_9_29 % 32u));
const D_RETIREMENT_SCAN_MASK: u32 = (1u << (INVARIANT_I_9_32 % 32u)) | (1u << (INVARIANT_I_9_33 % 32u))
    | (1u << (INVARIANT_I_9_34 % 32u)) | (1u << (INVARIANT_I_9_35 % 32u));

// 墓碑容量判的是哪一条：没装旋钮 I-9.21、装了 I-9.22；谱系、退役那一棵同理 I-9.36 / I-9.37（walk.rs capacity_invariant_of、lineage_or_retirement_capacity_invariant_of）
fn d_tombstone_capacity_mask() -> u32 {
    return select(1u << (INVARIANT_I_9_21 % 32u), 1u << (INVARIANT_I_9_22 % 32u), D_TOMBSTONE_CONTAINER_CAPACITY_KNOB != 0u);
}

fn d_lineage_or_retirement_capacity_mask(knob_records: u32) -> u32 {
    return select(1u << (INVARIANT_I_9_36 % 32u), 1u << (INVARIANT_I_9_37 % 32u), knob_records != 0u);
}
// 删除态扫描那一份版本清单里每一版重读树表、谱系树、墓碑树、inode 树与 deleted_inodes（只读，不进 w_failures）：判 I-9.18、I-9.19、I-9.20、
// I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29（主走读没沿最新根的实例表链走到末片时不比）。
// CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、谱系树条目不是恰好一条、谱系树或墓碑树读不全、谱系接不成一条链、头 ID 配不上恰好一棵
// inode 树或恰好一棵墓碑树、队列根读不出、队列里的号没有记录、inode 树读不全）在这里一格都不置位；CPU 按 (树表单元, txg) 去重只扫一次，
// 违例位是并集，这里不去重。三棵容器树装没装只供测试的容量旋钮看扫描内核源码前面的三个常量 D_TOMBSTONE_CONTAINER_CAPACITY_KNOB、
// D_LINEAGE_CONTAINER_CAPACITY_KNOB、D_RETIREMENT_CONTAINER_CAPACITY_KNOB（每片记录数，0 是没装；crash_judge_gpu.rs 的
// container_capacity_knob_wgsl_constants 生成）：没装的判产品那一条（I-9.21、I-9.36），装了的判旋钮那一条（I-9.22、I-9.37），
// 同 walk.rs check_pool_image_given_the_container_capacity_knobs。崩溃点重放与放量编的是三个都为 0 的那一份。不按已发布与否滤墓碑（第二节 T4）。
// 表越过上限（一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数）判不了，交 CPU

var<private> d_container_count: u32;

// 这一版谱系树有几片容器（各片的位置抄在 D_LINEAGE_CONTAINERS）、一共几条谱系记录
var<private> d_lineage_container_count: u32;

var<private> d_lineage_record_count: u32;

// d_read_the_lineage_of_a_version 顺带记下的、I-9.35 找父版要的（walk.rs judge_tombstones_and_the_lineage_of_a_version 里的 parent_txg）：谱系记录读得出
// （谱系树恰好一条、容器读得全，不要求接得成链）；叶序里第一条 T_rb 等于这一版 txg 的谱系记录有没有、它的 T_old
var<private> d_lineage_records_are_readable: bool;

var<private> d_lineage_names_the_parent_of_this_version: bool;

var<private> d_parent_txg_named_by_the_lineage: vec2<u32>;

var<private> d_object_birth_count: u32;

// I-9.29 跨版本那张表比不比：走读内核的主走读沿最新根的实例表链走到了末片（交接段 H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE，
// 与候选集的滤法用同一遍走读，同 walk.rs 的 Walk::abandoned_roots_filtering）才比；没走到末片（一片读不出、解不开、身份不对、链指针记录不合法、
// 链成了环）时回退候选集滤不掉被崩溃恢复抛弃的根，被抛弃的线上同号可以是另一个对象，CPU 报部分没判
// （walk.rs 的 AbandonedRootsFilteringOfTheCandidateSet；`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」），
// 这里一个号都不记、不置位。不记也就不会撞上那张表的上限：CPU 那一支一个号都不比，这里没有判不了的理由。
// 不用 r_instance_rows 判：它拒收的比主走读多（链指针记录之前有不是行的记录、载荷 CRC 对不上），候选集已经滤全时会说没滤全
// （判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第二节 K2 N5）
var<private> d_object_births_are_compared: bool;

// I-9.20 不重号那一句：一棵墓碑树的墓碑多于这么多条就判不了（原来两两比、比较次数随条数平方涨时定的门槛；改成排一次序之后门槛照旧，
// 判不判得了不变，排序借的两段按它的大小判装不装得下）
const DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE: u32 = 4096u;

fn d_container_row(container_index: u32) -> u32 {
    return D_CONTAINERS_OF_A_CONTAINER_TREE + container_index * D_CONTAINER_OF_A_CONTAINER_TREE_WORDS;
}

fn d_container_base(container_index: u32) -> u32 {
    return sw(d_container_row(container_index));
}

fn d_container_record_count(container_index: u32) -> u32 {
    return sw(d_container_row(container_index) + 1u);
}

// 第 container_index 片容器里第 record_index 条记录的偏移（记录宽读的时候核过是登记的 record_bytes）
fn d_container_record_offset(container_index: u32, record_index: u32, record_bytes: u32) -> u32 {
    return sw(d_container_row(container_index) + 2u) + record_index * record_bytes;
}

fn d_container_number(container_index: u32) -> vec2<u32> {
    let row = d_container_row(container_index);
    return vec2<u32>(sw(row + 3u), sw(row + 4u));
}

// 一棵墓碑树或谱系树（根指针 root_pointer）按叶序读出的容器进 D_CONTAINERS_OF_A_CONTAINER_TREE（walk.rs containers_of_a_container_tree_without_judging）：
// 根指针全零交回空（零根）；码 2 节点要读得出、层级 ≥ 1、条目宽 ≥ 120，类型段 0 的条目指下一层码 2 节点（子指针非零、层级 = 父层级 − 1），
// 类型段 = record_type 的指码 3 容器（打包记录类型 = record_type、记录宽 = 登记的 registered_record_bytes），别的类型段、读不出、形状不对交回 false（判不了）。
// 条目倒着压进 C_R_PENDING、弹出来是叶序（walk.rs 按条目序递归）
fn d_read_containers_of_a_container_tree(root_pointer: Ptr, record_type: u32, registered_record_bytes: u32) -> bool {
    d_container_count = 0u;
    if (root_pointer.all_zero) {
        return true;
    }
    let root = r_read_located_index_node(root_pointer);
    if (!root.ok || root.level == 0u || root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
        return false;
    }
    var pending = 0u;
    if (!d_push_container_tree_entries_in_reverse(root, &pending, 55u)) {
        return false;
    }
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let base = sw(row);
        let o = sw(row + 1u);
        let parent_level = sw(row + 2u);
        let child_pointer = parse_node_pointer(base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
        let entry_type = rd16(base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
        if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
            if (child_pointer.all_zero) {
                return false;
            }
            let child = r_read_located_index_node(child_pointer);
            if (!child.ok || child.level + 1u != parent_level || child.level == 0u || child.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
                return false;
            }
            if (!d_push_container_tree_entries_in_reverse(child, &pending, 56u)) {
                return false;
            }
        } else if (entry_type == record_type) {
            if (child_pointer.all_zero) {
                return false;
            }
            let container = parse_packed_unit(r_read_located_unit(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
            if (!container.ok || container.record_type != record_type || container.record_width != registered_record_bytes) {
                return false;
            }
            if (d_container_count >= D_MAX_CONTAINERS_OF_A_CONTAINER_TREE) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (57u << 8u);
                return false;
            }
            let container_row = d_container_row(d_container_count);
            sws(container_row, container.base);
            sws(container_row + 1u, container.record_count);
            sws(container_row + 2u, packed_record_offset(container, 0u));
            sws(container_row + 3u, container.container.x);
            sws(container_row + 4u, container.container.y);
            d_container_count = d_container_count + 1u;
        } else {
            return false;
        }
    }
    return true;
}

// 一条谱系记录在哪：谱系树叶序里第 container_index 片容器的第 record_index 条
struct LineageRecordPosition {
    container_index: u32,
    record_index: u32,
};

fn d_lineage_container_row(container_index: u32) -> u32 {
    return D_LINEAGE_CONTAINERS + container_index * D_LINEAGE_CONTAINER_WORDS;
}

fn d_lineage_container_record_count(container_index: u32) -> u32 {
    return sw(d_lineage_container_row(container_index) + 1u);
}

// 那条记录的一个 8 字节字段（field_offset 是 DELETION_LINEAGE_RECORD_*_OFFSET 之一），从容器字节里读
fn d_lineage_record_field(position: LineageRecordPosition, field_offset: u32) -> vec2<u32> {
    let row = d_lineage_container_row(position.container_index);
    return rd64(sw(row), sw(row + 2u) + position.record_index * DELETION_LINEAGE_RECORD_BYTES + field_offset);
}

fn d_lineage_rollback_txg(position: LineageRecordPosition) -> vec2<u32> {
    return d_lineage_record_field(position, DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
}

fn d_lineage_parent_txg(position: LineageRecordPosition) -> vec2<u32> {
    return d_lineage_record_field(position, DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET);
}

fn d_lineage_inode_number_watermark_rolled_back_from(position: LineageRecordPosition) -> vec2<u32> {
    return d_lineage_record_field(position, DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET);
}

// 按 T_rb 从大到小走谱系记录的下一步：从 previous 往下（has_previous 为假时从头起）。is_found 为假是走完了
struct NextLineageRecord {
    is_found: bool,
    position: LineageRecordPosition,
};

// 只在谱系接得成链（T_rb 两两不同，d_read_the_lineage_of_a_version 交回 true）时调：在全部记录里找 T_rb 比 previous 小的最大那一条
// （每一步过一遍全部记录，不靠叶序：合法的谱系叶序是升序，坏盘上不一定）
fn d_next_lineage_record_in_falling_rollback_order(has_previous: bool, previous: LineageRecordPosition) -> NextLineageRecord {
    var next: NextLineageRecord;
    next.is_found = false;
    next.position = previous;
    var bound = vec2<u32>(0u, 0u);
    if (has_previous) {
        bound = d_lineage_rollback_txg(previous);
    }
    var largest = vec2<u32>(0u, 0u);
    for (var container_index = 0u; container_index < d_lineage_container_count; container_index = container_index + 1u) {
        for (var record_index = 0u; record_index < d_lineage_container_record_count(container_index); record_index = record_index + 1u) {
            let position = LineageRecordPosition(container_index, record_index);
            let rollback_txg = d_lineage_rollback_txg(position);
            if (has_previous && !lt64(rollback_txg, bound)) {
                continue;
            }
            if (!next.is_found || lt64(largest, rollback_txg)) {
                largest = rollback_txg;
                next.is_found = true;
                next.position = position;
            }
        }
    }
    return next;
}

// ───────────────────────── I-9.30（walk.rs judge_inode_number_watermark_steps_of_the_newest_root）─────────────────────────
// 最新根记账里 inode 号水位那一统计量（标签 12、不带设备维）的行收进 D_INODE_NUMBER_WATERMARK_ROWS；台阶行 (g, H) 照 walk.rs
// inode_number_watermark_steps_of_a_version 认：代段早于最新根 txg 的行都是台阶，代段等于它的那一行值与代段最大的台阶不同（或没有更早的台阶）才是台阶。
// ① 判在收行那一步；② 判在墓碑与谱系那一遍走到最新根那一版时（inode 记录、谱系上的墓碑）；④ 也在那里判（谱系记录带的 H_cur）。
// CPU 记成射程里判不了的（记账树、树表、谱系、inode 树、墓碑树读不全，谱系接不成链）这里一格都不置位；行多于上限判不了
var<private> d_inode_number_watermark_row_count: u32;
// 收到了最新根的 inode 号水位行（CPU 不报不适用）：②④ 只在这时判
var<private> d_inode_number_watermark_steps_are_judged: bool;
var<private> d_newest_root_txg: vec2<u32>;
// 代段早于最新根 txg 的行里代段最大那一行的值（台阶里最后一级），has_an_earlier_step 为假是没有这样的行
var<private> d_latest_earlier_step_watermark: vec2<u32>;
var<private> d_has_an_earlier_step: bool;

fn d_inode_number_watermark_row_generation(row_index: u32) -> vec2<u32> {
    let row = D_INODE_NUMBER_WATERMARK_ROWS + row_index * D_INODE_NUMBER_WATERMARK_ROW_WORDS;
    return vec2<u32>(sw(row), sw(row + 1u));
}

fn d_inode_number_watermark_row_value(row_index: u32) -> vec2<u32> {
    let row = D_INODE_NUMBER_WATERMARK_ROWS + row_index * D_INODE_NUMBER_WATERMARK_ROW_WORDS;
    return vec2<u32>(sw(row + 2u), sw(row + 3u));
}

// 第 row_index 行是不是一级台阶（认法见本节开头）
fn d_inode_number_watermark_row_is_a_step(row_index: u32) -> bool {
    let generation = d_inode_number_watermark_row_generation(row_index);
    if (lt64(generation, d_newest_root_txg)) {
        return true;
    }
    if (!eq64(generation, d_newest_root_txg)) {
        return false;
    }
    return !d_has_an_earlier_step || !eq64(d_inode_number_watermark_row_value(row_index), d_latest_earlier_step_watermark);
}

// ② 那一判：号 inode、对象出生代 object_birth 的对象，有没有一级台阶 (g, H) 使 H > inode 且 object_birth > g
fn d_number_was_issued_before_a_step(inode: vec2<u32>, object_birth: vec2<u32>) -> bool {
    for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
        if (d_inode_number_watermark_row_is_a_step(row_index)
            && lt64(inode, d_inode_number_watermark_row_value(row_index))
            && lt64(d_inode_number_watermark_row_generation(row_index), object_birth)) {
            return true;
        }
    }
    return false;
}

// 最新根一棵记账树（根指针在 (base, offset)，非零）里标签 12、设备段 0xFFFF_FFFF 的行接着收进 D_INODE_NUMBER_WATERMARK_ROWS：key 宽 22；
// 叶条目宽 = 34、内部条目宽 = 108（等于、不是不小于：CPU 的行取自主走读 walk.rs walk_code_two_subtree，条目宽按 I-1.10 判、不等就不往下走、
// 这棵树不算走全；d_judge_pending_deletion_occupancy_of_an_accounting_tree 照的是 collect_accounting_rows_without_judging，那一份是不小于）、
// 子指针非零、孩子层级 = 父层级 − 1；读不全交回 false、这棵树收的行退回去（CPU 只取走全了的记账树的行）
fn d_collect_inode_number_watermark_rows_of_an_accounting_tree(base: u32, offset: u32) -> bool {
    let rows_before_this_tree = d_inode_number_watermark_row_count;
    var pending = 1u;
    sws(C_R_PENDING, base);
    sws(C_R_PENDING + 1u, offset);
    sws(C_R_PENDING + 2u, NONE);   // 根：层级取它自己头里的
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let p = parse_node_pointer(sw(row), sw(row + 1u));
        let expected_level = sw(row + 2u);
        let n = r_read_located_index_node(p);
        if (!n.ok || (expected_level != NONE && n.level != expected_level) || n.key_width != DELETION_ACCOUNTING_KEY_BYTES) {
            d_inode_number_watermark_row_count = rows_before_this_tree;
            return false;
        }
        if (n.level == 0u) {
            if (n.entry_width != DELETION_ACCOUNTING_ENTRY_BYTES) {
                d_inode_number_watermark_row_count = rows_before_this_tree;
                return false;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                let o = entry_offset(n, e);
                if (rd16(n.base, o) != DELETION_STATISTIC_INODE_WATERMARK || rd32(n.base, o + DELETION_ACCOUNTING_ROW_DEVICE_OFFSET) != NONE) {
                    continue;
                }
                if (d_inode_number_watermark_row_count >= D_MAX_INODE_NUMBER_WATERMARK_ROWS) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (63u << 8u);
                    d_inode_number_watermark_row_count = rows_before_this_tree;
                    return false;
                }
                let generation = rd64(n.base, o + DELETION_ACCOUNTING_ROW_GENERATION_OFFSET);
                let value = rd64(n.base, o + DELETION_ACCOUNTING_ROW_VALUE_OFFSET);
                let collected = D_INODE_NUMBER_WATERMARK_ROWS + d_inode_number_watermark_row_count * D_INODE_NUMBER_WATERMARK_ROW_WORDS;
                sws(collected, generation.x);
                sws(collected + 1u, generation.y);
                sws(collected + 2u, value.x);
                sws(collected + 3u, value.y);
                d_inode_number_watermark_row_count = d_inode_number_watermark_row_count + 1u;
            }
            continue;
        }
        if (n.entry_width != DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES) {
            d_inode_number_watermark_row_count = rows_before_this_tree;
            return false;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e) + DELETION_ACCOUNTING_KEY_BYTES;
            if (parse_node_pointer(n.base, o).all_zero) {
                d_inode_number_watermark_row_count = rows_before_this_tree;
                return false;
            }
            if (pending >= C_R_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (64u << 8u);
                d_inode_number_watermark_row_count = rows_before_this_tree;
                return false;
            }
            let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
            sws(child, n.base);
            sws(child + 1u, o);
            sws(child + 2u, n.level - 1u);
            pending = pending + 1u;
        }
    }
    return true;
}

// 收最新根记账里 inode 号水位那一统计量的行（树表里每一棵根指针非零的记账树），定台阶，判 ①：记账里的水位（代段最大的那一行的值）
// 不小于每一级台阶的 H。一行都没收到（没有走全了的记账树、或没有那一行）时 ②④ 不判（CPU 报不适用）
fn judge_inode_number_watermark_steps_against_the_watermark_of_the_newest_root() {
    d_inode_number_watermark_row_count = 0u;
    d_inode_number_watermark_steps_are_judged = false;
    let newest_root = checker_root_of(c_newest_index);
    d_newest_root_txg = newest_root.txg;
    let tree_table_pointer = parse_node_pointer(newest_root.base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {
        return;
    }
    // 记账树的节点按最新根自己的中央映射认位置（walk.rs judge_inode_number_watermark_steps_of_the_newest_root 的 central_mapping）
    s_use_the_central_mapping_named_at(newest_root.base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_ACCOUNTING
            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
            continue;
        }
        if (!d_collect_inode_number_watermark_rows_of_an_accounting_tree(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET)
            && undecidable != 0u) {
            return;
        }
    }
    if (d_inode_number_watermark_row_count == 0u) {
        return;
    }
    d_inode_number_watermark_steps_are_judged = true;
    var watermark_generation = d_inode_number_watermark_row_generation(0u);
    var watermark = d_inode_number_watermark_row_value(0u);
    d_has_an_earlier_step = false;
    var latest_earlier_step_generation = vec2<u32>(0u, 0u);
    d_latest_earlier_step_watermark = vec2<u32>(0u, 0u);
    for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
        let generation = d_inode_number_watermark_row_generation(row_index);
        if (lt64(watermark_generation, generation)) {
            watermark_generation = generation;
            watermark = d_inode_number_watermark_row_value(row_index);
        }
        if (lt64(generation, d_newest_root_txg) && (!d_has_an_earlier_step || lt64(latest_earlier_step_generation, generation))) {
            d_has_an_earlier_step = true;
            latest_earlier_step_generation = generation;
            d_latest_earlier_step_watermark = d_inode_number_watermark_row_value(row_index);
        }
    }
    for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
        if (d_inode_number_watermark_row_is_a_step(row_index)) {
            judge(INVARIANT_I_9_30, !lt64(watermark, d_inode_number_watermark_row_value(row_index)));
        }
    }
}

// ④ 那一判（谱系读得出、接得成链）：每条谱系记录 (T_rb, T_old, H_cur)，代段 < T_rb 的台阶里最大的 H 等于 H_cur；
// 没有这样的台阶判红；H_cur 为 0 时等式不成立（台阶的水位至少是 1），同 walk.rs 不另写一判。逐片、逐条从容器里读，判的次序不影响置的位
fn d_judge_the_lineage_against_the_inode_number_watermark_steps() {
    for (var container_index = 0u; container_index < d_lineage_container_count; container_index = container_index + 1u) {
        for (var record_index = 0u; record_index < d_lineage_container_record_count(container_index); record_index = record_index + 1u) {
            let position = LineageRecordPosition(container_index, record_index);
            let rollback_txg = d_lineage_rollback_txg(position);
            let watermark_rolled_back_from = d_lineage_inode_number_watermark_rolled_back_from(position);
            var has_a_step_before_the_rollback = false;
            var largest_step_before_the_rollback = vec2<u32>(0u, 0u);
            for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
                if (!d_inode_number_watermark_row_is_a_step(row_index) || !lt64(d_inode_number_watermark_row_generation(row_index), rollback_txg)) {
                    continue;
                }
                let step_watermark = d_inode_number_watermark_row_value(row_index);
                if (!has_a_step_before_the_rollback || lt64(largest_step_before_the_rollback, step_watermark)) {
                    largest_step_before_the_rollback = step_watermark;
                }
                has_a_step_before_the_rollback = true;
            }
            judge(INVARIANT_I_9_30, has_a_step_before_the_rollback && eq64(largest_step_before_the_rollback, watermark_rolled_back_from));
        }
    }
}

// 这一版的谱系（walk.rs judge_tombstones_and_the_lineage_of_a_version 开头与 lineage_chain_of_a_version 判链之前那两步）：树表 tt 里谱系树（种类 11）
// 恰好一条、整棵读得出、记录接得成一条链（每条 T_old < T_rb、没有两条同一个 T_rb）才交回 true；谱系记录留在容器里，每片容器的位置抄进
// D_LINEAGE_CONTAINERS。否则交回 false（这一版要谱系的那几判判不了）
fn d_read_the_lineage_of_a_version(tt: Node, version_txg: vec2<u32>) -> bool {
    d_lineage_record_count = 0u;
    d_lineage_container_count = 0u;
    d_lineage_records_are_readable = false;
    d_lineage_names_the_parent_of_this_version = false;
    var lineage_entry_count = 0u;
    var lineage_entry = 0u;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LINEAGE) {
            lineage_entry_count = lineage_entry_count + 1u;
            lineage_entry = entry;
        }
    }
    // 谱系树不是恰好一条、容器读不全：谱系容量那一条记射程里有一部分没判（walk.rs judge_tombstones_and_the_lineage_of_a_version 开头）
    if (lineage_entry_count != 1u
        || !d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, lineage_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
            PACKED_TYPE_LINEAGE, DELETION_LINEAGE_RECORD_BYTES)) {
        mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, d_lineage_or_retirement_capacity_mask(D_LINEAGE_CONTAINER_CAPACITY_KNOB));
        return false;
    }
    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS, D_LINEAGE_CONTAINER_CAPACITY_KNOB);
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let row = d_lineage_container_row(container_index);
        sws(row, d_container_base(container_index));
        sws(row + 1u, d_container_record_count(container_index));
        sws(row + 2u, d_container_record_offset(container_index, 0u, DELETION_LINEAGE_RECORD_BYTES));
    }
    d_lineage_container_count = d_container_count;
    // 叶序过一遍：数条数、找叶序里第一条 T_rb 等于这一版 txg 的（I-9.35 的父版）、核每条 T_old < T_rb、看 T_rb 是不是严格升序
    var every_record_is_younger_than_its_parent = true;
    // 叶序里 T_rb 严格升序（写路径只往最右那片追加，回退那次发布的 txg 一次比一次大）就没有两条同一个 T_rb
    var records_rise_in_leaf_order = true;
    var has_a_previous_record = false;
    var previous_rollback_txg = vec2<u32>(0u, 0u);
    for (var container_index = 0u; container_index < d_lineage_container_count; container_index = container_index + 1u) {
        for (var record_index = 0u; record_index < d_lineage_container_record_count(container_index); record_index = record_index + 1u) {
            let position = LineageRecordPosition(container_index, record_index);
            let rollback_txg = d_lineage_rollback_txg(position);
            let parent_txg = d_lineage_parent_txg(position);
            d_lineage_record_count = d_lineage_record_count + 1u;
            if (!d_lineage_names_the_parent_of_this_version && eq64(rollback_txg, version_txg)) {
                d_lineage_names_the_parent_of_this_version = true;
                d_parent_txg_named_by_the_lineage = parent_txg;
            }
            if (!lt64(parent_txg, rollback_txg)) {
                every_record_is_younger_than_its_parent = false;
            }
            if (has_a_previous_record && !lt64(previous_rollback_txg, rollback_txg)) {
                records_rise_in_leaf_order = false;
            }
            has_a_previous_record = true;
            previous_rollback_txg = rollback_txg;
        }
    }
    d_lineage_records_are_readable = true;
    if (!every_record_is_younger_than_its_parent) {
        return false;
    }
    // 叶序严格升序就没有两条同一个 T_rb；不是时两两比（条数平方次：写路径追加的谱系都是升序，只有坏盘走到这里）
    if (records_rise_in_leaf_order) {
        return true;
    }
    for (var container_index = 0u; container_index < d_lineage_container_count; container_index = container_index + 1u) {
        for (var record_index = 0u; record_index < d_lineage_container_record_count(container_index); record_index = record_index + 1u) {
            let rollback_txg = d_lineage_rollback_txg(LineageRecordPosition(container_index, record_index));
            for (var later_container_index = container_index; later_container_index < d_lineage_container_count; later_container_index = later_container_index + 1u) {
                var later_record_index = 0u;
                if (later_container_index == container_index) {
                    later_record_index = record_index + 1u;
                }
                for (; later_record_index < d_lineage_container_record_count(later_container_index); later_record_index = later_record_index + 1u) {
                    if (eq64(d_lineage_rollback_txg(LineageRecordPosition(later_container_index, later_record_index)), rollback_txg)) {
                        return false;
                    }
                }
            }
        }
    }
    return true;
}

// 「在谱系上」（walk.rs LineageChainOfAVersion::holds_txg 与 lineage_chain_of_a_version 的按段走）：从这一版自己的 txg 起，谱系记录按 T_rb 从大到小过一遍，
// T_rb 大于此刻走到的 txg 的跳过，否则 [T_rb, 此刻] 整段在链上、跳到 T_old；最后 [0, 此刻] 也在链上。此刻走到的 txg 只减不增，
// 一旦小于 txg，往后每一段都在它下面，txg 不在链上，不再往下走。只在谱系接得成链时调（d_read_the_lineage_of_a_version 交回 true）
fn d_txg_is_on_the_lineage(txg: vec2<u32>, version_txg: vec2<u32>) -> bool {
    var txg_walked_to = version_txg;
    var has_previous = false;
    var previous = LineageRecordPosition(0u, 0u);
    // 迭代上界：谱系记录条数（每一轮走过一条，按 T_rb 从大到小、不回头）
    loop {
        if (lt64(txg_walked_to, txg)) {
            return false;
        }
        let next = d_next_lineage_record_in_falling_rollback_order(has_previous, previous);
        if (!next.is_found) {
            break;
        }
        has_previous = true;
        previous = next.position;
        let rollback_txg = d_lineage_rollback_txg(next.position);
        if (lt64(txg_walked_to, rollback_txg)) {
            continue;
        }
        if (le64(rollback_txg, txg)) {
            return true;
        }
        txg_walked_to = d_lineage_parent_txg(next.position);
    }
    return le64(txg, txg_walked_to);
}

// I-9.29 跨版本那张表记一笔：(inode 树, 号) 第一次见到时记下对象出生代，再见到而出生代不同就判红（walk.rs 收齐之后每个号判一次
// births.len() == 1，与这里见到第二个不同的出生代就红是同一判）。表满判不了。主走读没沿最新根的实例表链走到末片时整张表不比（d_object_births_are_compared）
fn d_note_object_birth(inode_tree: vec2<u32>, inode: vec2<u32>, object_birth: vec2<u32>) {
    if (!d_object_births_are_compared) {
        return;
    }
    // 表里有一个号就判过（walk.rs judge_one_object_birth_per_inode_number 每个号判一次）
    judge(INVARIANT_I_9_29, true);
    let existing = keyed_first_row_of(d_object_birth_table(), d_object_birth_count, d_object_births_are_indexed,
        vec4<u32>(inode_tree.x, inode_tree.y, inode.x, inode.y));
    if (existing != NONE) {
        let row = D_OBJECT_BIRTHS + existing * D_OBJECT_BIRTH_WORDS;
        if (!eq64(vec2<u32>(sw(row + 4u), sw(row + 5u)), object_birth)) {
            violate(INVARIANT_I_9_29);
        }
        return;
    }
    if (d_object_birth_count >= D_MAX_OBJECT_BIRTHS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (59u << 8u);
        return;
    }
    let row = D_OBJECT_BIRTHS + d_object_birth_count * D_OBJECT_BIRTH_WORDS;
    sws(row, inode_tree.x);
    sws(row + 1u, inode_tree.y);
    sws(row + 2u, inode.x);
    sws(row + 3u, inode.y);
    sws(row + 4u, object_birth.x);
    sws(row + 5u, object_birth.y);
    let appended = d_object_birth_count;
    d_object_birth_count = appended + 1u;
    d_object_births_are_indexed = keyed_table_after_a_row_was_appended(d_object_birth_table(), d_object_birth_count,
        d_object_births_are_indexed, appended);
}

// I-9.29 跨版本那一张（键是 (inode 树, 号) 四个字，crash_judge_checker_common.wgsl 的 ⑬）与它的桶表建过没有
fn d_object_birth_table() -> KeyedTable {
    return KeyedTable(D_OBJECT_BIRTHS, D_OBJECT_BIRTH_WORDS, 4u, D_OBJECT_BIRTH_BUCKETS, D_OBJECT_BIRTH_BUCKET_COUNT);
}

var<private> d_object_births_are_indexed: bool;

// 一条墓碑（容器字节基 base、记录偏移 o）是不是整对象 [0, 2⁶⁴ − 1]（walk.rs TombstoneRecordOfTheScan::covers_the_whole_object：区间起 < 区间止照条款原文合取）
fn d_tombstone_covers_the_whole_object(base: u32, o: u32) -> bool {
    let range_start = rd64(base, o + DELETION_TOMBSTONE_RECORD_RANGE_START_OFFSET);
    let range_end = rd64(base, o + DELETION_TOMBSTONE_RECORD_RANGE_END_OFFSET);
    return lt64(range_start, range_end) && is_zero64(range_start) && eq64(range_end, vec2<u32>(NONE, NONE));
}

// 容器号与记录数的三句（walk.rs judge_tombstone_container_capacity_of_a_version、judge_lineage_or_retirement_container_capacity_of_a_version
// 有对象之后那三判）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的容器（读容器头，不沿根的条目读）沿叶序第一片容器号是 1、
// 相邻两片容器号加一（左片号加一不进位越过 2⁶⁴ − 1：同 walk.rs 的 checked_add）、每片（含最右那片）至少一条；不成立置 invariant 那一位。
// 有没有对象（没装旋钮至少两片、装了至少一片）由调用方先判
fn d_judge_container_numbers_from_one_by_one_and_no_empty_container(invariant: u32) {
    judge(invariant, eq64(d_container_number(0u), u64_of(D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE)));
    for (var container_index = 1u; container_index < d_container_count; container_index = container_index + 1u) {
        let left_container = d_container_number(container_index - 1u);
        let right_container = d_container_number(container_index);
        if (!(lt64(left_container, right_container) && eq64(add64(left_container, u64_of(1u)), right_container))) {
            violate(invariant);
        }
    }
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        if (d_container_record_count(container_index) == 0u) {
            violate(invariant);
        }
    }
}

// 装了只供测试的容量旋钮（knob_records 条一片）的那一棵（walk.rs 那两个判法装了旋钮那一支，I-9.22 / I-9.37）：至少一片时判容器号与记录数的三句，
// 每片记录数 ≤ knob_records，最右那片之外每片记录数 == knob_records（开放容器满了才关，D18 已定项 24 第 3 条在旋钮下的那一半）；零根没有对象
fn d_judge_containers_under_the_capacity_knob(knob_records: u32, invariant: u32) {
    if (d_container_count == 0u) {
        return;
    }
    d_judge_container_numbers_from_one_by_one_and_no_empty_container(invariant);
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        if (d_container_record_count(container_index) > knob_records) {
            violate(invariant);
        }
    }
    for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
        if (d_container_record_count(container_index) != knob_records) {
            violate(invariant);
        }
    }
}

// I-9.21 / I-9.22（walk.rs judge_tombstone_container_capacity_of_a_version）：墓碑旋钮 D_TOMBSTONE_CONTAINER_CAPACITY_KNOB 不为 0 时判 I-9.22
// （d_judge_containers_under_the_capacity_knob）；为 0（没装）时读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的墓碑容器至少两片才判 I-9.21：
// 容器号与记录数的三句，最右那片之外每片记录数 == 815；零根或只一片没有对象
fn d_judge_tombstone_container_capacity() {
    if (D_TOMBSTONE_CONTAINER_CAPACITY_KNOB != 0u) {
        d_judge_containers_under_the_capacity_knob(D_TOMBSTONE_CONTAINER_CAPACITY_KNOB, INVARIANT_I_9_22);
        return;
    }
    if (d_container_count < 2u) {
        return;
    }
    d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_21);
    for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
        if (d_container_record_count(container_index) != DELETION_TOMBSTONE_CONTAINER_RECORDS) {
            violate(INVARIANT_I_9_21);
        }
    }
}

// I-9.36 / I-9.37（walk.rs judge_lineage_or_retirement_container_capacity_of_a_version，与 I-9.21 / I-9.22 同形）：knob_records 不为 0
// （那一棵装了旋钮）时调 d_judge_containers_under_the_capacity_knob 判 I-9.37（测试镜像里谱系与退役容器不超旋钮值）；为 0 时读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的谱系或退役容器
// 至少两片才判 I-9.36：容器号与记录数的三句，最右那片之外每片记录数 == 产品容量 product_records（谱系 1359、退役 959）；零根或只一片没有对象。
// 谱系树在 d_read_the_lineage_of_a_version 读完容器那一处调，退役树在墓碑与谱系那一遍读完退役容器那一处调（与 walk.rs 一样：谱系树要恰好一条条目、
// 容器读得全才判）
fn d_judge_lineage_or_retirement_container_capacity(product_records: u32, knob_records: u32) {
    if (knob_records != 0u) {
        d_judge_containers_under_the_capacity_knob(knob_records, INVARIANT_I_9_37);
        return;
    }
    if (d_container_count < 2u) {
        return;
    }
    d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_36);
    for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
        if (d_container_record_count(container_index) != product_records) {
            violate(INVARIANT_I_9_36);
        }
    }
}

// I-9.20 管全部墓碑的那几句（walk.rs judge_every_tombstone_of_a_version_is_well_formed）：每片容器记录数 ≤ 815（与 I-1.7 重合，留作合取）；
// 每条整对象、对象出生代 ≤ 死亡 txg ≤ 这一版的 txg
fn d_judge_every_tombstone_is_well_formed(version_txg: vec2<u32>) {
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        let record_count = d_container_record_count(container_index);
        judge(INVARIANT_I_9_20, record_count <= DELETION_TOMBSTONE_CONTAINER_RECORDS);
        for (var record_index = 0u; record_index < record_count; record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
            if (!d_tombstone_covers_the_whole_object(base, o)) {
                violate(INVARIANT_I_9_20);
            }
            let death_txg = rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET);
            if (!le64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), death_txg) || !le64(death_txg, version_txg)) {
                violate(INVARIANT_I_9_20);
            }
        }
    }
}

// I-9.20 不重号那一句：在谱系上的墓碑的 (号 lo、号 hi、对象出生代 lo、对象出生代 hi) 每条一行、4 个字，按 order_rows_by_their_first_four_words 排、
// 行号次序借 C_ROW_ORDER。借 C_ALLOC_RECORDS 那一段（C_MAX_ALLOC_RECORDS × C_ALLOC_RECORD_WORDS 个字）：读它的几处（I-5.4、I-3.10、I-5.5、记账那几条的
// r_allocation_tree）都把记录数复位、从头写过才读，墓碑与谱系那一遍里没有谁读它；C_ROW_ORDER 在扫描内核里 I-1.8 之后就没有谁读
// （I-5.4、I-3.16 排分配记录借的是 crash_judge_checker_common.wgsl ⑮ 的 C_ALLOCATION_RECORD_ROW_ORDER）
const D_TOMBSTONE_KEY_WORDS: u32 = 4u;

const D_TOMBSTONE_KEYS: u32 = C_ALLOC_RECORDS;

// 借的两段装得下墓碑总数的上限（解析内核时判，不进展开量）
const_assert DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE * D_TOMBSTONE_KEY_WORDS <= C_MAX_ALLOC_RECORDS * C_ALLOC_RECORD_WORDS;

const_assert DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE <= C_ROW_ORDER_END - C_ROW_ORDER;

// I-9.20 不重号那一句（walk.rs judge_tombstones_on_the_lineage_of_a_version_are_distinct，只判在谱系上的）：这一版在谱系上的墓碑按 (号, 对象出生代)
// 不重复。原来每条在谱系上的墓碑跟叶序在它之前的每一条比（每版 T² / 2）；改成每条在谱系上的墓碑记一行键、按键排一次序，同键的在次序里挨着，
// 比相邻两行就够——有两条同键 ⟺ 次序里有相邻两行同键；原写法每见一条重的置一次违例位，这里每对相邻同键置一次，置的是同一位。
// 墓碑总数多于 DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE 照旧判不了（站点 60，与原写法同一个门槛，判不判得了不变）
fn d_judge_tombstones_on_the_lineage_are_distinct(version_txg: vec2<u32>) {
    var tombstone_count = 0u;
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        tombstone_count = tombstone_count + d_container_record_count(container_index);
    }
    if (tombstone_count > DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (60u << 8u);
        return;
    }
    var keys_on_the_lineage = 0u;
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
            if (!d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                continue;
            }
            judge(INVARIANT_I_9_20, true);
            let object = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
            let object_birth = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
            let key_row = D_TOMBSTONE_KEYS + keys_on_the_lineage * D_TOMBSTONE_KEY_WORDS;
            sws(key_row, object.x);
            sws(key_row + 1u, object.y);
            sws(key_row + 2u, object_birth.x);
            sws(key_row + 3u, object_birth.y);
            keys_on_the_lineage = keys_on_the_lineage + 1u;
        }
    }
    order_rows_by_their_first_four_words(D_TOMBSTONE_KEYS, D_TOMBSTONE_KEY_WORDS, keys_on_the_lineage, C_ROW_ORDER);
    for (var position = 1u; position < keys_on_the_lineage; position = position + 1u) {
        let earlier = D_TOMBSTONE_KEYS + sw(C_ROW_ORDER + position - 1u) * D_TOMBSTONE_KEY_WORDS;
        let later = D_TOMBSTONE_KEYS + sw(C_ROW_ORDER + position) * D_TOMBSTONE_KEY_WORDS;
        if (sw(earlier) == sw(later) && sw(earlier + 1u) == sw(later + 1u) && sw(earlier + 2u) == sw(later + 2u) && sw(earlier + 3u) == sw(later + 3u)) {
            violate(INVARIANT_I_9_20);
        }
    }
}

// 读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的墓碑树对 D_INODE_RECORDS 里配上的那棵 inode 树（头 ID inode_tree）：
// I-9.10 墓碑那一个等号（walk.rs judge_tombstones_against_the_object_births_of_the_records_by_number：按号认、不看谱系，有记录时对象出生代相等）；
// 谱系接得成链时（lineage_holds）再判 I-9.19（walk.rs judge_tombstones_on_the_lineage_name_no_live_object：谱系上的墓碑有号与出生代都相同的记录时
// 那条记录 nlink = 0），并把谱系上每条墓碑的 (号, 对象出生代) 记进 I-9.29 那张表
fn d_judge_tombstones_against_the_records(inode_tree: vec2<u32>, version_txg: vec2<u32>, lineage_holds: bool) {
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
            let object = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
            let object_birth = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
            let record_row = d_inode_record_row_of(object);
            if (record_row != NONE) {
                judge(INVARIANT_I_9_10, eq64(d_inode_record_object_birth(record_row), object_birth));
            }
            if (!lineage_holds || !d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                continue;
            }
            if (record_row != NONE && eq64(d_inode_record_object_birth(record_row), object_birth)) {
                judge(INVARIANT_I_9_19, d_inode_record_nlink(record_row) == 0u);
            }
            d_note_object_birth(inode_tree, object, object_birth);
        }
    }
}

// I-9.18（walk.rs judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage）在树表 tt 的一棵 inode 树（条目偏移 inode_tree_entry）上：
// 经头 ID 配这棵树的 deleted_inodes（恰好一条）与墓碑树（恰好一条），队列里每个号 N 要在墓碑树里有 (N, N 的记录的对象出生代, 整对象)、
// 死亡 txg 在这一版的谱系上；N 没有记录的判不了、不判红（那一形只让 I-9.5 红）。队列空的没有对象
fn d_judge_queued_inodes_have_tombstones_on_the_lineage(tt: Node, inode_tree_entry: u32, version_txg: vec2<u32>, lineage_holds: bool) {
    let inode_tree = rd64(tt.base, inode_tree_entry);
    var deleted_inodes_entry_count = 0u;
    var deleted_inodes_entry = 0u;
    var tombstone_tree_entry_count = 0u;
    var tombstone_tree_entry = 0u;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
        let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        if (kind == TREE_KIND_DELETED_INODES && eq64(head_identifier, inode_tree)) {
            deleted_inodes_entry_count = deleted_inodes_entry_count + 1u;
            deleted_inodes_entry = entry;
        }
        if (kind == TREE_KIND_TOMBSTONE && eq64(head_identifier, inode_tree)) {
            tombstone_tree_entry_count = tombstone_tree_entry_count + 1u;
            tombstone_tree_entry = entry;
        }
    }
    // walk.rs 那一判记射程里有一部分没判的几处：deleted_inodes 不是恰好一条、队列读不出、记录读不全、墓碑树不是恰好一条或读不全、谱系接不成链、
    // 队列里的号没有记录；队列空（含根指针全零）不判
    if (deleted_inodes_entry_count != 1u) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_18);
        return;
    }
    let queue_read = d_read_deletion_queue(tt, deleted_inodes_entry);
    if (!queue_read.readable) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_18);
        return;
    }
    if (!queue_read.present || queue_read.queue.entry_count == 0u) {
        return;
    }
    if (!d_read_inode_records(parse_node_pointer(tt.base, inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))
        || tombstone_tree_entry_count != 1u
        || !d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, tombstone_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
            PACKED_TYPE_TOMBSTONE, DELETION_TOMBSTONE_RECORD_BYTES)
        || !lineage_holds) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_18);
        return;
    }
    let queue = queue_read.queue;
    for (var queue_index = 0u; queue_index < queue.entry_count; queue_index = queue_index + 1u) {
        let inode = rd64(queue.base, entry_offset(queue, queue_index));
        let record_row = d_inode_record_row_of(inode);
        if (record_row == NONE) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_18);
            continue;
        }
        let object_birth = d_inode_record_object_birth(record_row);
        var has_a_whole_object_tombstone_on_the_lineage = false;
        for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
            let base = d_container_base(container_index);
            for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
                let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
                if (eq64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET), inode)
                    && eq64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), object_birth)
                    && d_tombstone_covers_the_whole_object(base, o)
                    && d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                    has_a_whole_object_tombstone_on_the_lineage = true;
                }
            }
        }
        judge(INVARIANT_I_9_18, has_a_whole_object_tombstone_on_the_lineage);
    }
}

// ───────────────────────── 退役记录那四条（walk.rs 的 RETIREMENT_*：I-9.32–I-9.35）─────────────────────────
// 判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条原写「GPU 判器同判 I-9.31，其余回落 CPU」「配了 GPU 就一定要
// GPU 判，不退回 CPU」，GPU 对拍文件头「一个状态都不许判不了」，这四条也在 GPU 上判，不记判不了。
// 都在墓碑与谱系那一遍的每一版上判（d_judge_tombstones_and_the_lineage_of_a_version）：退役容器借墓碑树那一处读容器的调用点读，
// 每版一处 d_judge_the_retirement_records_and_their_completeness 判 I-9.32、I-9.33、I-9.34，并就地跨到父版判 I-9.35；叶身份只有一个读的调用点
// （被判那一版与父版轮流走它）。展开预算（crash_judge_dispatch.rs 的 LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL）只许这四条多占两万来个表达式，
// 重的读法都只留一个调用点。
// I-9.32 每条都判；I-9.33、I-9.34 只判死亡 txg 在这一版谱系上的（「在谱系上」同墓碑那几判，d_txg_is_on_the_lineage）。
// CPU 记成射程里判不了的那几格（退役树、inode 树、谱系树读不全，谱系接不成链，头 ID 配不上，父版读不全或不止一版）这里一格都不置位；
// 表越过上限（一版 inode 树的叶身份、两两比不重复的退役记录总数、同一个 txg 的版本数）判不了，交 CPU。

// 退役记录 34 字节（类型 3 登记 34）：被退役身份的出生树 0、打包记录类型 8（2 字节）、容器号 10、容器出生代 18、死亡 txg 26
// （walk.rs RETIREMENT_RECORD_*_OFFSET；身份四段与 inode 树内部条目里的身份引用同序同宽）；一片退役容器最多 959 条
// （⌊(32768 − 136) ÷ 34⌋，walk.rs RETIREMENT_CONTAINER_RECORDS）
const DELETION_RETIREMENT_RECORD_BYTES: u32 = 34u;
const DELETION_RETIREMENT_RECORD_BIRTH_TREE_OFFSET: u32 = 0u;
const DELETION_RETIREMENT_RECORD_RECORD_TYPE_OFFSET: u32 = 8u;
const DELETION_RETIREMENT_RECORD_CONTAINER_OFFSET: u32 = 10u;
const DELETION_RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: u32 = 18u;
const DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET: u32 = 26u;
const DELETION_RETIREMENT_CONTAINER_RECORDS: u32 = 959u;
// inode 树码 2 节点条目里身份引用的起点（分隔 key 8 字节之后；类型段就是身份里的打包记录类型，在条目偏移 16）
const DELETION_INODE_INTERNAL_ENTRY_IDENTITY_OFFSET: u32 = 8u;
// I-9.33 两两比：一棵退役树的退役记录多于这么多条就判不了（比较次数随条数平方涨，同墓碑那一判的上限）
const DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE: u32 = 4096u;
// I-9.35 找父版：同一个 txg 的版本（按 walk.rs 的去重算不同的）最多记几份树表指针，多于它判不了（合法镜像上同一个 txg 至多两三份）
const DELETION_MAX_VERSIONS_AT_ONE_TXG: u32 = 8u;
// 这一节判不了的站点号（「条目太多」一类，与别处不撞）：读叶身份时压 C_R_PENDING 压满、叶身份表满、两两比的退役记录太多、同一个 txg 的版本太多
const DELETION_SITE_LEAF_IDENTITIES_PENDING: u32 = 65u;
const DELETION_SITE_LEAF_IDENTITY_TABLE_FULL: u32 = 66u;
const DELETION_SITE_RETIREMENT_RECORDS_COMPARED_PAIRWISE: u32 = 67u;
const DELETION_SITE_VERSIONS_AT_ONE_TXG: u32 = 68u;

// 一片打包记录容器的身份四段（walk.rs ContainerIdentityOfTheScan）
struct ContainerIdentityOfTheScan {
    birth_tree: vec2<u32>,
    record_type: u32,
    container: vec2<u32>,
    container_birth: vec2<u32>,
};

// 从 (base, o) 起按「出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8」读一个身份
fn d_container_identity_at(base: u32, o: u32) -> ContainerIdentityOfTheScan {
    var identity: ContainerIdentityOfTheScan;
    identity.birth_tree = rd64(base, o + DELETION_RETIREMENT_RECORD_BIRTH_TREE_OFFSET);
    identity.record_type = rd16(base, o + DELETION_RETIREMENT_RECORD_RECORD_TYPE_OFFSET);
    identity.container = rd64(base, o + DELETION_RETIREMENT_RECORD_CONTAINER_OFFSET);
    identity.container_birth = rd64(base, o + DELETION_RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET);
    return identity;
}

fn d_same_container_identity(left: ContainerIdentityOfTheScan, right: ContainerIdentityOfTheScan) -> bool {
    return eq64(left.birth_tree, right.birth_tree) && left.record_type == right.record_type
        && eq64(left.container, right.container) && eq64(left.container_birth, right.container_birth);
}

// 叶身份表（D_LEAF_IDENTITIES_OF_THE_VERSION 或 D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION）的第 row_index 行
fn d_leaf_identity_at(table: u32, row_index: u32) -> ContainerIdentityOfTheScan {
    let row = table + row_index * D_LEAF_IDENTITY_WORDS;
    var identity: ContainerIdentityOfTheScan;
    identity.birth_tree = vec2<u32>(sw(row), sw(row + 1u));
    identity.record_type = sw(row + 2u);
    identity.container = vec2<u32>(sw(row + 3u), sw(row + 4u));
    identity.container_birth = vec2<u32>(sw(row + 5u), sw(row + 6u));
    return identity;
}

fn d_put_leaf_identity(table: u32, row_index: u32, identity: ContainerIdentityOfTheScan) {
    let row = table + row_index * D_LEAF_IDENTITY_WORDS;
    sws(row, identity.birth_tree.x);
    sws(row + 1u, identity.birth_tree.y);
    sws(row + 2u, identity.record_type);
    sws(row + 3u, identity.container.x);
    sws(row + 4u, identity.container.y);
    sws(row + 5u, identity.container_birth.x);
    sws(row + 6u, identity.container_birth.y);
}

// 叶身份表前 identity_count 行里有没有这个身份
fn d_leaf_identity_table_holds(table: u32, identity_count: u32, identity: ContainerIdentityOfTheScan) -> bool {
    for (var row_index = 0u; row_index < identity_count; row_index = row_index + 1u) {
        if (d_same_container_identity(d_leaf_identity_at(table, row_index), identity)) {
            return true;
        }
    }
    return false;
}

// walk.rs inode_leaf_identities_without_judging：一版 inode 树（根指针在 (base, root_pointer_offset)）里每片叶容器的身份——码 2 节点里类型段 2
// 那几条条目的身份引用，不开叶——收进叶身份表 table（集合：同一个身份只记一行），交回条数。根指针全零、零条目根交回 0；根读不出、根层级 0 而有条目、
// 码 2 节点条目宽不到 120、类型段既不是 0 也不是 2、类型段 0 的子指针全零或读不出、子层级不是父层级 − 1 或是 0，都交回 NONE（判不了）；
// 表满、C_R_PENDING 压满记判不了、交回 NONE。根与每个孩子都经同一处读（C_R_PENDING 每行：指针所在的字节基、指针偏移、父层级，根的父层级记 NONE），
// 次序不影响集合
fn d_read_inode_leaf_identities(base: u32, root_pointer_offset: u32, table: u32) -> u32 {
    var identity_count = 0u;
    sws(C_R_PENDING, base);
    sws(C_R_PENDING + 1u, root_pointer_offset);
    sws(C_R_PENDING + 2u, NONE);
    var pending = 1u;
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let parent_level = sw(row + 2u);
        let pointer = parse_node_pointer(sw(row), sw(row + 1u));
        if (pointer.all_zero) {
            if (parent_level == NONE) {
                continue;
            }
            return NONE;
        }
        let node = r_read_located_index_node(pointer);
        if (!node.ok) {
            return NONE;
        }
        if (parent_level == NONE && node.level == 0u) {
            if (node.entry_count == 0u) {
                continue;
            }
            return NONE;
        }
        if ((parent_level != NONE && (node.level + 1u != parent_level || node.level == 0u)) || node.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
            return NONE;
        }
        for (var entry_index = 0u; entry_index < node.entry_count; entry_index = entry_index + 1u) {
            let entry = entry_offset(node, entry_index);
            let entry_type = rd16(node.base, entry + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
            if (entry_type == DELETION_PACKED_TYPE_INODE) {
                let identity = d_container_identity_at(node.base, entry + DELETION_INODE_INTERNAL_ENTRY_IDENTITY_OFFSET);
                if (d_leaf_identity_table_holds(table, identity_count, identity)) {
                    continue;
                }
                if (identity_count >= D_MAX_LEAF_IDENTITIES) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_LEAF_IDENTITY_TABLE_FULL << 8u);
                    return NONE;
                }
                d_put_leaf_identity(table, identity_count, identity);
                identity_count = identity_count + 1u;
            } else if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
                if (pending >= C_R_MAX_PENDING) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_LEAF_IDENTITIES_PENDING << 8u);
                    return NONE;
                }
                let pushed = C_R_PENDING + pending * C_R_PENDING_WORDS;
                sws(pushed, node.base);
                sws(pushed + 1u, entry + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
                sws(pushed + 2u, node.level);
                pending = pending + 1u;
            } else {
                return NONE;
            }
        }
    }
    return identity_count;
}

// 树表 tt 里种类 kind、条目偏移 identifier_offset 处的 8 字节（树 ID 在 0，头 ID 在 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET）等于 identifier 的最后一条条目的偏移
// （walk.rs 按树 ID / 头 ID 收进 BTreeMap，同一个键的后一条盖掉前一条）；没有交回 NONE
// 树表 tt 里最后一条种类是 kind 的条目（不看树 ID），没有是 NONE
fn d_last_tree_table_entry_of_kind_keyed_by_kind(tt: Node, kind: u32) -> u32 {
    var last_entry = NONE;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == kind) {
            last_entry = entry;
        }
    }
    return last_entry;
}

fn d_last_tree_table_entry_of_kind_keyed_by(tt: Node, kind: u32, identifier_offset: u32, identifier: vec2<u32>) -> u32 {
    var last_entry = NONE;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == kind && eq64(rd64(tt.base, entry + identifier_offset), identifier)) {
            last_entry = entry;
        }
    }
    return last_entry;
}

// I-9.32（walk.rs judge_every_retirement_record_of_a_version_is_well_formed）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的退役容器每片记录数 ≤ 959；
// 每条被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg。不按谱系滤
fn d_judge_every_retirement_record_is_well_formed(version_txg: vec2<u32>) {
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        let record_count = d_container_record_count(container_index);
        judge(INVARIANT_I_9_32, record_count <= DELETION_RETIREMENT_CONTAINER_RECORDS);
        for (var record_index = 0u; record_index < record_count; record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
            let retired = d_container_identity_at(base, o);
            let death_txg = rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET);
            if (retired.record_type != DELETION_PACKED_TYPE_INODE || is_zero64(retired.birth_tree)
                || !lt64(retired.container_birth, death_txg) || !le64(death_txg, version_txg)) {
                violate(INVARIANT_I_9_32);
            }
        }
    }
}

// I-9.33（walk.rs judge_retirement_records_on_the_lineage_of_a_version_are_distinct）：读进来的退役容器里死亡 txg 在这一版谱系上的退役记录按身份不重复。
// 两两比，退役记录总数多于 DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE 判不了
fn d_judge_retirement_records_on_the_lineage_are_distinct(version_txg: vec2<u32>) {
    var record_total = 0u;
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        record_total = record_total + d_container_record_count(container_index);
    }
    if (record_total > DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_RETIREMENT_RECORDS_COMPARED_PAIRWISE << 8u);
        return;
    }
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
            if (!d_txg_is_on_the_lineage(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                continue;
            }
            judge(INVARIANT_I_9_33, true);
            let retired = d_container_identity_at(base, o);
            // 叶序在它之前的退役记录：前几片容器的全部、同一片里它之前的
            for (var earlier_container = 0u; earlier_container <= container_index; earlier_container = earlier_container + 1u) {
                let earlier_base = d_container_base(earlier_container);
                var earlier_limit = d_container_record_count(earlier_container);
                if (earlier_container == container_index) {
                    earlier_limit = record_index;
                }
                for (var earlier_record = 0u; earlier_record < earlier_limit; earlier_record = earlier_record + 1u) {
                    let earlier = d_container_record_offset(earlier_container, earlier_record, DELETION_RETIREMENT_RECORD_BYTES);
                    if (d_same_container_identity(d_container_identity_at(earlier_base, earlier), retired)
                        && d_txg_is_on_the_lineage(rd64(earlier_base, earlier + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                        violate(INVARIANT_I_9_33);
                    }
                }
            }
        }
    }
}

// 读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的退役容器里有没有一条死亡 txg 等于 death_txg、身份是 identity 的退役记录
fn d_retired_at(identity: ContainerIdentityOfTheScan, death_txg: vec2<u32>) -> bool {
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
            if (eq64(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), death_txg) && d_same_container_identity(d_container_identity_at(base, o), identity)) {
                return true;
            }
        }
    }
    return false;
}

// 删除态扫描那一份版本清单里的一版：树表指针所在的字节基与偏移、txg（先候选集里每条根、再由记录施加出来的那几版）
struct VersionOfTheDeletionScan {
    base: u32,
    tree_table_offset: u32,
    // 这一版的中央映射根指针（候选根在根记录偏移 256，由记录施加出来的那一版在记录新根段偏移 86）
    mapping_root_offset: u32,
    txg: vec2<u32>,
};

fn d_version_of_the_scan(version_index: u32) -> VersionOfTheDeletionScan {
    var version: VersionOfTheDeletionScan;
    if (version_index < c_candidate_count) {
        let root = checker_root_of(sw(C_CANDIDATES + version_index));
        version.base = root.base;
        version.tree_table_offset = 36u;
        version.mapping_root_offset = G_ROOT_RECORD_MAPPING_ROOT_POINTER;
        version.txg = root.txg;
    } else {
        let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
        version.base = record.base;
        version.tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        version.mapping_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER;
        version.txg = record.txg;
    }
    return version;
}

// I-9.35 的父版在版本清单里是哪一份：walk.rs 按 (树表指针第一条位置条目的盘、槽、校验和, txg) 去重、只记第一次（树表指针全零的版不去重，每次都记一版），
// 去过重之后 txg 等于 txg 的恰好一份才比（没有不判、不止一份报判不了）。is_single 为真时 pointer 是那一份的树表指针，mapping_root 是它的中央映射根指针
struct SingleVersionAtATxg {
    is_single: bool,
    // 去过重之后几份（0：没有父版，不判）
    version_count: u32,
    pointer: Ptr,
    mapping_root: Ptr,
};

fn d_single_version_of_the_scan_at_txg(txg: vec2<u32>) -> SingleVersionAtATxg {
    var found: SingleVersionAtATxg;
    found.is_single = false;
    found.version_count = 0u;
    var distinct_versions = 0u;
    var distinct_locations: array<vec4<u32>, 8>;
    var distinct_location_count = 0u;
    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
        let version = d_version_of_the_scan(version_index);
        if (!eq64(version.txg, txg)) {
            continue;
        }
        let pointer = parse_node_pointer(version.base, version.tree_table_offset);
        if (!pointer.all_zero) {
            let location = vec4<u32>(pointer.device0, pointer.slot0.x, pointer.slot0.y, pointer.sum0);
            var repeats_an_earlier_one = false;
            for (var location_index = 0u; location_index < distinct_location_count; location_index = location_index + 1u) {
                if (all(distinct_locations[location_index] == location)) {
                    repeats_an_earlier_one = true;
                }
            }
            if (repeats_an_earlier_one) {
                continue;
            }
            if (distinct_location_count >= DELETION_MAX_VERSIONS_AT_ONE_TXG) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_VERSIONS_AT_ONE_TXG << 8u);
                return found;
            }
            distinct_locations[distinct_location_count] = location;
            distinct_location_count = distinct_location_count + 1u;
        }
        distinct_versions = distinct_versions + 1u;
        found.pointer = pointer;
        found.mapping_root = parse_node_pointer(version.base, version.mapping_root_offset);
    }
    found.is_single = distinct_versions == 1u;
    found.version_count = distinct_versions;
    return found;
}

// 一版的 I-9.35 要比的父版（walk.rs 里的 parent_txg 与 judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version 的门）：
// 主走读沿最新根的实例表链走到了末片（候选集滤得掉被崩溃恢复抛弃的根，同 I-9.29 那张表的门）、这一版谱系记录读得出、父版 txg 有
// （d_read_the_lineage_of_a_version 叶序第一条 T_rb 等于这一版 txg 的 T_old，否则 txg − 1，txg 0 没有），版本清单里那一份恰好一版、
// 树表读得出（0 条也算：每棵树是空集）、条目不窄，这一版又没有一棵退役树的头 ID 配不上这一版的任何一棵 inode 树：is_judged 为真，tree_table 是父版的树表
struct ParentVersionToCompare {
    is_judged: bool,
    // 父版的中央映射根指针（父版那棵 inode 树的叶身份按父版自己的映射认位置，walk.rs 里父版是版本清单里的另一版、按它自己那一版的映射读）
    mapping_root: Ptr,
    // 这一版要比而比不了（CPU 记 I-9.35 射程里有一部分没判）：谱系记录读不出、父版不止一份、父版树表指针全零或读不全、有一棵退役树配不上 inode 树
    is_part_of_the_range_not_judged: bool,
    tree_table: Node,
};

fn d_parent_version_to_compare(tt: Node, version_txg: vec2<u32>) -> ParentVersionToCompare {
    var parent: ParentVersionToCompare;
    parent.is_judged = false;
    parent.is_part_of_the_range_not_judged = false;
    parent.tree_table = bad_node();
    if (!d_object_births_are_compared) {
        return parent;
    }
    if (!d_lineage_records_are_readable) {
        parent.is_part_of_the_range_not_judged = true;
        return parent;
    }
    var parent_txg = d_parent_txg_named_by_the_lineage;
    if (!d_lineage_names_the_parent_of_this_version) {
        if (is_zero64(version_txg)) {
            return parent;
        }
        parent_txg = sub64(version_txg, vec2<u32>(1u, 0u));
    }
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_RETIREMENT
            && d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET)) == NONE) {
            parent.is_part_of_the_range_not_judged = true;
            return parent;
        }
    }
    let single = d_single_version_of_the_scan_at_txg(parent_txg);
    if (single.version_count == 0u) {
        return parent;
    }
    if (!single.is_single || single.pointer.all_zero) {
        parent.is_part_of_the_range_not_judged = true;
        return parent;
    }
    parent.tree_table = r_read_index_node(single.pointer);
    parent.mapping_root = single.mapping_root;
    parent.is_judged = parent.tree_table.ok
        && (parent.tree_table.entry_count == 0u || parent.tree_table.entry_width >= DELETION_TREE_TABLE_ENTRY_BYTES);
    parent.is_part_of_the_range_not_judged = !parent.is_judged;
    return parent;
}

// 这一版树表 tt 里条目 entry（一棵退役树、容器已读进 D_CONTAINERS_OF_A_CONTAINER_TREE；或一棵没有退役树配它的 inode 树、d_container_count 已置 0）：
// - 退役树：I-9.32 每条判；谱系接得成链时判 I-9.33，再经头 ID 配这张表里树 ID 等于它的最后一棵 inode 树判 I-9.34（walk.rs judge_tombstones_and_the_lineage_of_a_version 末段）；
// - I-9.35（walk.rs judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version）：父版要比（parent.is_judged）时，对一棵 inode 树 T 比一次——
//   退役树那一支只在它是头 ID 为 T 的最后一棵时比（walk.rs 按头 ID 收进 BTreeMap，后一棵盖掉前一棵），T 是它头 ID 配上的那棵；
//   inode 树那一支是表里同 ID 的最后一条、又没有一棵退役树的头 ID 是它（死于这一版的退役集合是空集）。
//   父版那棵树的叶身份集 − 这一版的叶身份集 = 死于这一版（死亡 txg 等于这一版 txg）的退役记录的身份集，漏记、多记都置位；父版没有 T 那一棵是空集。
// 叶身份按被判那一版、父版的次序轮流经同一处读（读不全的那一版那一判不判）
// version_mapping_root 是这一版的中央映射根指针（叶身份读完父版那一棵要装回它：这一版后面几棵树还按它读）
fn d_judge_the_retirement_records_and_their_completeness(tt: Node, entry: u32, version_txg: vec2<u32>, lineage_holds: bool,
    parent: ParentVersionToCompare, version_mapping_root: Ptr) {
    var inode_tree_entry = entry;
    var judges_no_live_leaf = false;
    var judges_completeness = parent.is_judged;
    if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_RETIREMENT) {
        d_judge_every_retirement_record_is_well_formed(version_txg);
        if (lineage_holds) {
            d_judge_retirement_records_on_the_lineage_are_distinct(version_txg);
        } else {
            mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, (1u << (INVARIANT_I_9_33 % 32u)) | (1u << (INVARIANT_I_9_34 % 32u)));
        }
        let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        inode_tree_entry = d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, head_identifier);
        judges_no_live_leaf = lineage_holds && inode_tree_entry != NONE;
        // 头 ID 配不上 inode 树：I-9.34 射程里有一部分没判（walk.rs 那一段 paired_leaf_identities 为空）
        if (lineage_holds && inode_tree_entry == NONE) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_34);
        }
        judges_completeness = judges_completeness && inode_tree_entry != NONE
            && d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_RETIREMENT, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET, head_identifier) == entry;
    }
    if (!judges_no_live_leaf && !judges_completeness) {
        return;
    }
    let inode_tree = rd64(tt.base, inode_tree_entry);
    var parent_inode_tree_entry = NONE;
    if (judges_completeness) {
        parent_inode_tree_entry = d_last_tree_table_entry_of_kind_keyed_by(parent.tree_table, TREE_KIND_INODE, 0u, inode_tree);
    }
    // 第 0 轮读这一版那棵树进 D_LEAF_IDENTITIES_OF_THE_VERSION，第 1 轮读父版那棵进 D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION（父版没有那一棵是空集，不读）
    var leaf_identity_counts = vec2<u32>(0u, 0u);
    var readings = 1u;
    if (judges_completeness && parent_inode_tree_entry != NONE) {
        readings = 2u;
    }
    // 每一轮先装那一版的中央映射（父版那一轮装父版的）；多一轮只把这一版的装回去（同一条根指针装着的不重读）
    for (var reading = 0u; reading <= readings; reading = reading + 1u) {
        var mapping_root = version_mapping_root;
        if (reading == 1u && reading < readings) {
            mapping_root = parent.mapping_root;
        }
        s_use_the_central_mapping_of(mapping_root);
        if (reading == readings) {
            break;
        }
        var tree_table_base = tt.base;
        var root_pointer_offset = inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
        var table = D_LEAF_IDENTITIES_OF_THE_VERSION;
        if (reading == 1u) {
            tree_table_base = parent.tree_table.base;
            root_pointer_offset = parent_inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
            table = D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION;
        }
        leaf_identity_counts[reading] = d_read_inode_leaf_identities(tree_table_base, root_pointer_offset, table);
    }
    let leaf_identity_count = leaf_identity_counts.x;
    let parent_leaf_identity_count = leaf_identity_counts.y;
    // 叶身份读不全：要判的那几条射程里有一部分没判
    if (leaf_identity_count == NONE) {
        if (judges_no_live_leaf) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_34);
        }
        if (judges_completeness) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_35);
        }
        return;
    }
    // I-9.34（walk.rs judge_retirement_records_on_the_lineage_name_no_live_leaf）：谱系上的退役记录指认的身份不在这一版那棵树的叶身份里
    if (judges_no_live_leaf) {
        for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
            let base = d_container_base(container_index);
            for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
                let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
                if (d_txg_is_on_the_lineage(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                    judge(INVARIANT_I_9_34, !d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_VERSION, leaf_identity_count, d_container_identity_at(base, o)));
                }
            }
        }
    }
    if (!judges_completeness) {
        return;
    }
    if (parent_leaf_identity_count == NONE) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_35);
        return;
    }
    // 两份叶身份都读全了：这一棵树判一次（walk.rs 每棵 inode 树一次 judge），漏记、多记再置违例位
    judge(INVARIANT_I_9_35, true);
    // 漏记：父版有、这一版没有，又没有死于这一版的退役记录
    for (var parent_row = 0u; parent_row < parent_leaf_identity_count; parent_row = parent_row + 1u) {
        let gone = d_leaf_identity_at(D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION, parent_row);
        if (!d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_VERSION, leaf_identity_count, gone)
            && !d_retired_at(gone, version_txg)) {
            violate(INVARIANT_I_9_35);
        }
    }
    // 多记：死于这一版的退役记录指认的身份要在父版里、不在这一版里
    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
        let base = d_container_base(container_index);
        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
            if (!eq64(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                continue;
            }
            let retired = d_container_identity_at(base, o);
            if (!d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION, parent_leaf_identity_count, retired)
                || d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_VERSION, leaf_identity_count, retired)) {
                violate(INVARIANT_I_9_35);
            }
        }
    }
}

// walk.rs judge_tombstones_and_the_lineage_of_a_version 的一版：树表指针在 (base, tree_table_pointer_offset)，这一版的 txg 是 version_txg。
// 指针全零、树表 0 条的不判；树表读不出、条目宽不到 200 的不判。is_the_newest_root：这一版是最新的根，顺带判 I-9.30 的 ② ④
// （walk.rs 在 judge_inode_number_watermark_steps_of_the_newest_root 里另读一遍最新根；这里借这一遍已经读进来的记录、墓碑与谱系，不多一个调用点）
// mapping_root_offset 起是这一版的中央映射根指针：谱系、墓碑、inode、退役几棵树里豁免三类之外的单元按它认位置
// （walk.rs judge_tombstones_and_the_lineage_of_the_versions 每一版的 central_mapping）
fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer_offset: u32, version_txg: vec2<u32>, is_the_newest_root: bool,
    mapping_root_offset: u32) {
    let judges_the_inode_number_watermark_steps = is_the_newest_root && d_inode_number_watermark_steps_are_judged;
    // I-9.30 ② ④ 读不全时记射程里有一部分没判（只在最新根那一版、收到了水位行时）
    let inode_number_watermark_mask = select(0u, 1u << (INVARIANT_I_9_30 % 32u), judges_the_inode_number_watermark_steps);
    // 树表指针全零、读不出、条目比字段表短：这一版要判的墓碑、谱系、退役那几条与三条容量都判不全（walk.rs 同处）
    let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
    var tt = bad_node();
    if (!tree_table_pointer.all_zero) {
        tt = r_read_index_node(tree_table_pointer);
    }
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES)) {
        mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, D_TOMBSTONE_AND_LINEAGE_SCAN_MASK | D_RETIREMENT_SCAN_MASK | d_tombstone_capacity_mask()
            | d_lineage_or_retirement_capacity_mask(D_LINEAGE_CONTAINER_CAPACITY_KNOB)
            | d_lineage_or_retirement_capacity_mask(D_RETIREMENT_CONTAINER_CAPACITY_KNOB) | inode_number_watermark_mask);
        return;
    }
    if (tt.entry_count == 0u) {
        return;
    }
    let version_mapping_root = parse_node_pointer(base, mapping_root_offset);
    s_use_the_central_mapping_of(version_mapping_root);
    let lineage_holds = d_read_the_lineage_of_a_version(tt, version_txg);
    let parent = d_parent_version_to_compare(tt, version_txg);
    if (parent.is_part_of_the_range_not_judged && d_last_tree_table_entry_of_kind_keyed_by_kind(tt, TREE_KIND_INODE) != NONE) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_35);
    }
    if (judges_the_inode_number_watermark_steps && lineage_holds) {
        d_judge_the_lineage_against_the_inode_number_watermark_steps();
    }
    if (!lineage_holds) {
        mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, inode_number_watermark_mask);
    }
    // 每棵读得出的 inode 树的每条记录记进 I-9.29 那张表（最新根那一版另判 I-9.30 ②）
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_INODE) {
            continue;
        }
        if (!d_read_inode_records(parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
            mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, (1u << (INVARIANT_I_9_29 % 32u)) | inode_number_watermark_mask);
            continue;
        }
        let inode_tree = rd64(tt.base, entry);
        for (var record_index = 0u; record_index < d_inode_record_count; record_index = record_index + 1u) {
            let row = D_INODE_RECORDS + record_index * D_INODE_RECORD_WORDS;
            d_note_object_birth(inode_tree, vec2<u32>(sw(row), sw(row + 1u)), d_inode_record_object_birth(row));
            if (judges_the_inode_number_watermark_steps) {
                judge(INVARIANT_I_9_30, !d_number_was_issued_before_a_step(vec2<u32>(sw(row), sw(row + 1u)), d_inode_record_object_birth(row)));
            }
        }
    }
    // 每棵墓碑树与退役树（容器经同一处读）：读不全的不判。墓碑树只要墓碑的那几判先判，再经头 ID 配恰好一棵 inode 树判墓碑对记录的那几判；
    // 退役树与「父版要比、表里同 ID 的最后一条、又没有退役树配它」的每棵 inode 树（死于这一版的退役集合是空集）交给
    // d_judge_the_retirement_records_and_their_completeness（一个调用点）
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
        let is_a_retirement_tree = kind == TREE_KIND_RETIREMENT;
        if (kind == TREE_KIND_INODE) {
            let inode_tree = rd64(tt.base, entry);
            if (!parent.is_judged
                || d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, inode_tree) != entry
                || d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_RETIREMENT, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET, inode_tree) != NONE) {
                continue;
            }
            d_container_count = 0u;
        } else if (kind != TREE_KIND_TOMBSTONE && !is_a_retirement_tree) {
            continue;
        } else if (!d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
                select(PACKED_TYPE_TOMBSTONE, PACKED_TYPE_RETIREMENT, is_a_retirement_tree),
                select(DELETION_TOMBSTONE_RECORD_BYTES, DELETION_RETIREMENT_RECORD_BYTES, is_a_retirement_tree))) {
            // 容器读不全：墓碑树是 TOMBSTONE_AND_LINEAGE_SCAN 与墓碑容量（最新根那一版另有 I-9.30），退役树是 RETIREMENT_SCAN 与退役容量
            mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, select(D_TOMBSTONE_AND_LINEAGE_SCAN_MASK | d_tombstone_capacity_mask() | inode_number_watermark_mask,
                D_RETIREMENT_SCAN_MASK | d_lineage_or_retirement_capacity_mask(D_RETIREMENT_CONTAINER_CAPACITY_KNOB), is_a_retirement_tree));
            continue;
        }
        if (kind != TREE_KIND_TOMBSTONE) {
            if (is_a_retirement_tree) {
                d_judge_lineage_or_retirement_container_capacity(DELETION_RETIREMENT_CONTAINER_RECORDS, D_RETIREMENT_CONTAINER_CAPACITY_KNOB);
            }
            d_judge_the_retirement_records_and_their_completeness(tt, entry, version_txg, lineage_holds, parent, version_mapping_root);
            continue;
        }
        d_judge_tombstone_container_capacity();
        d_judge_every_tombstone_is_well_formed(version_txg);
        if (lineage_holds) {
            d_judge_tombstones_on_the_lineage_are_distinct(version_txg);
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_20);
        }
        // I-9.30 ②：最新根谱系上的墓碑
        if (judges_the_inode_number_watermark_steps && lineage_holds) {
            for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
                let container_base = d_container_base(container_index);
                for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
                    let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
                    if (d_txg_is_on_the_lineage(rd64(container_base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
                        judge(INVARIANT_I_9_30, !d_number_was_issued_before_a_step(rd64(container_base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
                            rd64(container_base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET)));
                    }
                }
            }
        }
        let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        var paired_inode_tree_count = 0u;
        var paired_inode_tree_entry = 0u;
        for (var other_index = 0u; other_index < tt.entry_count; other_index = other_index + 1u) {
            let other = entry_offset(tt, other_index);
            if (rd16(tt.base, other + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_INODE && eq64(rd64(tt.base, other), head_identifier)) {
                paired_inode_tree_count = paired_inode_tree_count + 1u;
                paired_inode_tree_entry = other;
            }
        }
        // 配不上恰好一棵 inode 树、那棵读不全、谱系接不成链：I-9.19 与 I-9.29 射程里有一部分没判
        let records_of_the_paired_inode_tree = paired_inode_tree_count == 1u
            && d_read_inode_records(parse_node_pointer(tt.base, paired_inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET));
        if (!records_of_the_paired_inode_tree || !lineage_holds) {
            mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, (1u << (INVARIANT_I_9_19 % 32u)) | (1u << (INVARIANT_I_9_29 % 32u)));
        }
        if (!records_of_the_paired_inode_tree) {
            continue;
        }
        d_judge_tombstones_against_the_records(head_identifier, version_txg, lineage_holds);
    }
    // I-9.18：每棵 inode 树
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_INODE) {
            d_judge_queued_inodes_have_tombstones_on_the_lineage(tt, entry, version_txg, lineage_holds);
        }
    }
}

// 墓碑与谱系那一遍：删除态扫描那一份版本清单（回退候选集里每条根，与由记录施加出来的那几版），每一版从自己的 txg 与自己的谱系树起算；
// 全部扫完时 I-9.29 那张表已经判完（d_note_object_birth 见到第二个出生代当场判）；主走读没沿最新根的实例表链走到末片时那张表不比
fn judge_tombstones_and_the_lineage_of_the_versions() {
    d_object_birth_count = 0u;
    d_object_births_are_indexed = false;
    d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;
    // 候选集滤不掉被崩溃恢复抛弃的根：I-9.29 与 I-9.35 报部分没判（walk.rs judge_tombstones_and_the_lineage_of_the_versions 末段）
    if (!d_object_births_are_compared) {
        mark_parts_of_the_range_not_judged(D_SCAN_MASK_WORD, (1u << (INVARIANT_I_9_29 % 32u)) | (1u << (INVARIANT_I_9_35 % 32u)));
    }
    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
        let version = d_version_of_the_scan(version_index);
        let is_the_newest_root = version_index < c_candidate_count && sw(C_CANDIDATES + version_index) == c_newest_index;
        d_forget_the_inode_records_read_in_full();
        d_judge_tombstones_and_the_lineage_of_a_version(version.base, version.tree_table_offset, version.txg, is_the_newest_root,
            version.mapping_root_offset);
    }
}
