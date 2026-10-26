// GPU 判器池级 checker 的扫描重建内核（接在 crash_judge_checker_scan.wgsl 之后、同一份源码里的第三个入口 scan_rebuild_main，另编一条管线、
// 接在引用与快照那一截之后派）：判 I-1.13（扫描重建与遍历方向的索引相同），照 `singlefs_checker::walk` 的
// judge_scan_rebuild_agrees_with_the_walked_index 与 `singlefs_checker::rebuild` 逐条对齐（欠账 C821；用户 2026-10-18 定「补 GPU 扫描重建内核」，
// 两路结论逐位相同）。I-1.13 是池级 checker 最后判的一条：比对之前要看别的不变量有没有违例（walk.rs 的
// INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED），所以接在三个池级 checker 入口都写完违例位之后跑，从结论字读违例位。
// 结局另写进结论的 SCAN_REBUILD_OUTCOME_WORD 那一字（成立、违例、报不适用的哪一种理由，码由 crash_judge_gpu.rs 的
// scan_rebuild_outcome_wgsl_constants 生成、接在扫描内核源码最前面），对拍用例拿它与 CPU 那一句不适用的理由逐个比。
//
// 只读不判的那几遍（树表、inode 树、extent 树、中央映射、deadlist）用扫描内核里现成的读法（walk.rs 的 *_without_judging 各一份）；
// 扫描方向的单元与 I-1.8 那一遍同一个收法。草稿区只用扫描与引用两个入口跑完之后没有谁再读的那几段：本文件自己的表从 C_R_ROWS 起
// （之前的 C_INSTANCE_ROWS 到 W_INSTANCE_ROW_NEXT 是已发布谓词 w_published 读的最新根实例表行，留着），到中央映射那一版的表
// （C_VERSION_MAPPING_ROWS，读最新根的中央映射时写）之前为止；扫描内核的读法各自写的那几段（C_R_PENDING、C_R_SEEN、D_DATA_UNITS、
// S_DEADLIST_ENTRIES、D_LINEAGE_CONTAINERS）照它们自己的用法写。
//
// 每张表的条数上限与扫描内核读同一批东西的那张表相同：扫描项与 I-1.8 那一遍（C_MAX_SCANNED，站点 36）、一棵 inode 树的记录与删除态扫描那一张
// （D_MAX_INODE_RECORDS）、一棵 extent 树的数据单元与删除态扫描那一张（D_MAX_DATA_UNITS）、墓碑与 I-9.20 不重号那一判（DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE）、
// 谱系容器与读一棵容器树那一张（D_MAX_CONTAINERS_OF_A_CONTAINER_TREE）；满了记判不了（站点 110–118），不当成立。

// 本文件写的表的起点与终点（解析内核时判装得下，不进展开量）
const SCAN_REBUILD_TABLES_START: u32 = C_R_ROWS;

// ① 扫描项（walk.rs scanned_content_units，与 I-1.8 那一遍同一个收法）：每项一行——读出来那一份切片的字节基、落点的盘、槽 lo/hi
const SCAN_REBUILD_UNIT_WORDS: u32 = 4u;
const SCAN_REBUILD_MAX_UNITS: u32 = C_MAX_SCANNED;
const SCAN_REBUILD_UNITS: u32 = SCAN_REBUILD_TABLES_START;
// ② 扫描项按（对象身份、择新键、归并键、收的次序）排好的次序
const SCAN_REBUILD_UNIT_ORDER: u32 = SCAN_REBUILD_UNITS + SCAN_REBUILD_UNIT_WORDS * SCAN_REBUILD_MAX_UNITS;
// ③ 要读记录的那几片当前版本码 3 容器（墓碑、退役；谱系的照扫描内核的写法记进 D_LINEAGE_CONTAINERS）：打包记录类型、解得开那一份的字节基、记录数
const SCAN_REBUILD_CONTAINER_WORDS: u32 = 3u;
const SCAN_REBUILD_CONTAINERS: u32 = SCAN_REBUILD_UNIT_ORDER + SCAN_REBUILD_MAX_UNITS;
// ④ 重建出的中央映射（rebuild.rs RebuiltIndexTables::central_mapping）：key 7 字、那一版在 ② 里的起点与副本数、比过没有。同一个 key 后记的盖掉先记的
//    （BTreeMap::insert）；条数过 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST 起建桶表（同 crash_judge_checker_common.wgsl 的 ⑬）
const SCAN_REBUILD_MAPPING_WORDS: u32 = 10u;
const SCAN_REBUILD_MAPPING: u32 = SCAN_REBUILD_CONTAINERS + SCAN_REBUILD_CONTAINER_WORDS * SCAN_REBUILD_MAX_UNITS;
const SCAN_REBUILD_MAPPING_BUCKET_COUNT: u32 = 2u * SCAN_REBUILD_MAX_UNITS;
const SCAN_REBUILD_MAPPING_BUCKETS: u32 = SCAN_REBUILD_MAPPING + SCAN_REBUILD_MAPPING_WORDS * SCAN_REBUILD_MAX_UNITS;
// ⑤ 当前版本的数据单元按 (inode, 对象出生代) 分的组（键 4 字）
const SCAN_REBUILD_DATA_UNIT_GROUP_WORDS: u32 = 4u;
const SCAN_REBUILD_DATA_UNIT_GROUPS: u32 = SCAN_REBUILD_MAPPING_BUCKETS + SCAN_REBUILD_MAPPING_BUCKET_COUNT;
const SCAN_REBUILD_DATA_UNIT_GROUP_BUCKET_COUNT: u32 = 2u * SCAN_REBUILD_MAX_UNITS;
const SCAN_REBUILD_DATA_UNIT_GROUP_BUCKETS: u32 = SCAN_REBUILD_DATA_UNIT_GROUPS + SCAN_REBUILD_DATA_UNIT_GROUP_WORDS * SCAN_REBUILD_MAX_UNITS;
// ⑥ 当前版本的数据单元（rebuild.rs 的 current_data_units，按对象身份的次序记）：组、偏移 lo/hi、0（前四个字是 order_rows_by_their_first_four_words 的键），
//    inode lo/hi、对象出生代 lo/hi、单元头出生 txg lo/hi、那一版在 ② 里的起点与副本数、偏移更小的单元里最大的出生 txg lo/hi、有没有那样的单元
const SCAN_REBUILD_DATA_UNIT_WORDS: u32 = 15u;
const SCAN_REBUILD_DATA_UNITS: u32 = SCAN_REBUILD_DATA_UNIT_GROUP_BUCKETS + SCAN_REBUILD_DATA_UNIT_GROUP_BUCKET_COUNT;
const SCAN_REBUILD_DATA_UNIT_ORDER: u32 = SCAN_REBUILD_DATA_UNITS + SCAN_REBUILD_DATA_UNIT_WORDS * SCAN_REBUILD_MAX_UNITS;
// ⑦ 重建出的 inode 记录集合（键是号）：号 lo/hi、记录所在的字节基、记录的偏移
const SCAN_REBUILD_INODE_RECORD_WORDS: u32 = 4u;
const SCAN_REBUILD_MAX_INODE_RECORDS: u32 = D_MAX_INODE_RECORDS;
const SCAN_REBUILD_INODE_RECORDS: u32 = SCAN_REBUILD_DATA_UNIT_ORDER + SCAN_REBUILD_MAX_UNITS;
const SCAN_REBUILD_INODE_RECORD_BUCKET_COUNT: u32 = 2u * SCAN_REBUILD_MAX_INODE_RECORDS;
const SCAN_REBUILD_INODE_RECORD_BUCKETS: u32 = SCAN_REBUILD_INODE_RECORDS + SCAN_REBUILD_INODE_RECORD_WORDS * SCAN_REBUILD_MAX_INODE_RECORDS;
// ⑧ 重建出的每个 inode 的数据单元集合（键是 (inode, 偏移)）：inode lo/hi、偏移 lo/hi、映射 key 7 字
const SCAN_REBUILD_POSITION_WORDS: u32 = 11u;
const SCAN_REBUILD_POSITIONS: u32 = SCAN_REBUILD_INODE_RECORD_BUCKETS + SCAN_REBUILD_INODE_RECORD_BUCKET_COUNT;
const SCAN_REBUILD_POSITION_BUCKET_COUNT: u32 = 2u * SCAN_REBUILD_MAX_UNITS;
const SCAN_REBUILD_POSITION_BUCKETS: u32 = SCAN_REBUILD_POSITIONS + SCAN_REBUILD_POSITION_WORDS * SCAN_REBUILD_MAX_UNITS;
// ⑨ 墓碑判死的对象（rebuild.rs 的 tombstoned_objects）：号 lo/hi、对象出生代 lo/hi
const SCAN_REBUILD_TOMBSTONED_OBJECT_WORDS: u32 = 4u;
const SCAN_REBUILD_MAX_TOMBSTONED_OBJECTS: u32 = DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE;
const SCAN_REBUILD_TOMBSTONED_OBJECTS: u32 = SCAN_REBUILD_POSITION_BUCKETS + SCAN_REBUILD_POSITION_BUCKET_COUNT;
const SCAN_REBUILD_TOMBSTONED_OBJECT_BUCKET_COUNT: u32 = 2u * SCAN_REBUILD_MAX_TOMBSTONED_OBJECTS;
const SCAN_REBUILD_TOMBSTONED_OBJECT_BUCKETS: u32 = SCAN_REBUILD_TOMBSTONED_OBJECTS + SCAN_REBUILD_TOMBSTONED_OBJECT_WORDS * SCAN_REBUILD_MAX_TOMBSTONED_OBJECTS;
// ⑩ 遍历方向的 inode 记录集合（walk.rs inode_record_bytes_without_judging，同号后读到的盖掉先读到的）：号 lo/hi、字节基、记录的偏移、记录宽、比过没有
const SCAN_REBUILD_WALKED_INODE_RECORD_WORDS: u32 = 6u;
const SCAN_REBUILD_WALKED_INODE_RECORDS: u32 = SCAN_REBUILD_TOMBSTONED_OBJECT_BUCKETS + SCAN_REBUILD_TOMBSTONED_OBJECT_BUCKET_COUNT;
const SCAN_REBUILD_WALKED_INODE_RECORD_BUCKETS: u32 = SCAN_REBUILD_WALKED_INODE_RECORDS + SCAN_REBUILD_WALKED_INODE_RECORD_WORDS * SCAN_REBUILD_MAX_INODE_RECORDS;
// ⑪ 遍历方向的每个 inode 的数据单元集合（键是 (inode, 偏移)）：inode lo/hi、偏移 lo/hi、映射 key 7 字、比过没有
const SCAN_REBUILD_WALKED_POSITION_WORDS: u32 = 12u;
const SCAN_REBUILD_WALKED_POSITIONS: u32 = SCAN_REBUILD_WALKED_INODE_RECORD_BUCKETS + SCAN_REBUILD_INODE_RECORD_BUCKET_COUNT;
const SCAN_REBUILD_WALKED_POSITION_BUCKET_COUNT: u32 = 2u * D_MAX_DATA_UNITS;
const SCAN_REBUILD_WALKED_POSITION_BUCKETS: u32 = SCAN_REBUILD_WALKED_POSITIONS + SCAN_REBUILD_WALKED_POSITION_WORDS * D_MAX_DATA_UNITS;
// ⑫ 最新根中央映射那一版的表（C_VERSION_MAPPING_ROWS）每一行的记号：被 deadlist 点名、size 之外的单元的 key、进遍历方向那一张、比过没有
const SCAN_REBUILD_MAPPING_ROW_FLAGS: u32 = SCAN_REBUILD_WALKED_POSITION_BUCKETS + SCAN_REBUILD_WALKED_POSITION_BUCKET_COUNT;
const SCAN_REBUILD_TABLES_END: u32 = SCAN_REBUILD_MAPPING_ROW_FLAGS + C_MAX_VERSION_MAPPING_ROWS;

const_assert SCAN_REBUILD_TABLES_START == W_INSTANCE_ROW_NEXT + C_MAX_INSTANCE_ROWS;
const_assert SCAN_REBUILD_TABLES_END <= C_VERSION_MAPPING_ROWS;

const SCAN_REBUILD_MAPPING_ROW_PINNED_BY_THE_DEADLIST: u32 = 1u;
const SCAN_REBUILD_MAPPING_ROW_OF_A_DATA_UNIT_BEYOND_THE_SIZE: u32 = 2u;
const SCAN_REBUILD_MAPPING_ROW_IN_THE_WALKED_TABLE: u32 = 4u;
const SCAN_REBUILD_MAPPING_ROW_COMPARED: u32 = 8u;

// 判不了的站点（「条目太多」一类，与别处不撞）
const SCAN_REBUILD_SITE_UNITS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (110u << 8u);
const SCAN_REBUILD_SITE_INODE_RECORDS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (111u << 8u);
const SCAN_REBUILD_SITE_WALKED_INODE_RECORDS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (112u << 8u);
const SCAN_REBUILD_SITE_TOMBSTONED_OBJECTS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (113u << 8u);
const SCAN_REBUILD_SITE_LINEAGE_CONTAINERS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (114u << 8u);
// 经 d_push_container_tree_entries_in_reverse 的 site 参数拼成原因字。原来取 115，与扫描内核释放代高于 F 那张表两头碰上的那一处
// （crash_judge_checker_scan.wgsl 的 c_deferred_below_the_floor）拼出同一个原因字 0x7303，看原因字分不出是哪一张满了
const SCAN_REBUILD_SITE_INODE_TREE_PENDING_PAST_THE_STACK: u32 = 118u;

// 码 1 与码 3 的头里几段的偏移（singlefs_checker 的 ContentUnitClass::header_offsets）：明文头末尾、对象身份末尾、诞生代号、写序、载荷校验和
const SCAN_REBUILD_OBJECT_IDENTITY_START: u32 = 42u;
const SCAN_REBUILD_PACKED_RECORD_TYPE_OFFSET: u32 = 51u;
const SCAN_REBUILD_PACKED_BIRTH_SEQUENCE_OFFSET: u32 = 103u;
// 退役记录点名的身份（26 字节）与码 3 对象身份里类标签之后那一段同序同宽（rebuild.rs RETIREMENT_RECORD_RETIRED_IDENTITY_BYTES）
const SCAN_REBUILD_RETIRED_IDENTITY_BYTES: u32 = 26u;
// 实例表的片（码 3 打包记录类型 4）不进中央映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）
const SCAN_REBUILD_INSTANCE_TABLE_RECORD_TYPE: u32 = 4u;
// 码 3 容器里第一条记录的偏移（头 107 加预留 29，同 crash_judge_common.wgsl 的 packed_record_offset）
const SCAN_REBUILD_PACKED_RECORDS_START: u32 = 136u;

var<private> scan_rebuild_unit_count: u32;
var<private> scan_rebuild_container_count: u32;
var<private> scan_rebuild_mapping_count: u32;
var<private> scan_rebuild_mapping_is_indexed: bool;
var<private> scan_rebuild_data_unit_group_count: u32;
var<private> scan_rebuild_data_unit_groups_are_indexed: bool;
var<private> scan_rebuild_data_unit_count: u32;
var<private> scan_rebuild_inode_record_count: u32;
var<private> scan_rebuild_inode_records_are_indexed: bool;
var<private> scan_rebuild_position_count: u32;
var<private> scan_rebuild_positions_are_indexed: bool;
var<private> scan_rebuild_tombstoned_object_count: u32;
var<private> scan_rebuild_tombstoned_objects_are_indexed: bool;
var<private> scan_rebuild_walked_inode_record_count: u32;
var<private> scan_rebuild_walked_inode_records_are_indexed: bool;
var<private> scan_rebuild_walked_position_count: u32;
var<private> scan_rebuild_walked_positions_are_indexed: bool;

fn scan_rebuild_inode_record_table() -> KeyedTable {
    return KeyedTable(SCAN_REBUILD_INODE_RECORDS, SCAN_REBUILD_INODE_RECORD_WORDS, 2u, SCAN_REBUILD_INODE_RECORD_BUCKETS,
        SCAN_REBUILD_INODE_RECORD_BUCKET_COUNT);
}

fn scan_rebuild_walked_inode_record_table() -> KeyedTable {
    return KeyedTable(SCAN_REBUILD_WALKED_INODE_RECORDS, SCAN_REBUILD_WALKED_INODE_RECORD_WORDS, 2u, SCAN_REBUILD_WALKED_INODE_RECORD_BUCKETS,
        SCAN_REBUILD_INODE_RECORD_BUCKET_COUNT);
}

fn scan_rebuild_position_table() -> KeyedTable {
    return KeyedTable(SCAN_REBUILD_POSITIONS, SCAN_REBUILD_POSITION_WORDS, 4u, SCAN_REBUILD_POSITION_BUCKETS, SCAN_REBUILD_POSITION_BUCKET_COUNT);
}

fn scan_rebuild_walked_position_table() -> KeyedTable {
    return KeyedTable(SCAN_REBUILD_WALKED_POSITIONS, SCAN_REBUILD_WALKED_POSITION_WORDS, 4u, SCAN_REBUILD_WALKED_POSITION_BUCKETS,
        SCAN_REBUILD_WALKED_POSITION_BUCKET_COUNT);
}

fn scan_rebuild_tombstoned_object_table() -> KeyedTable {
    return KeyedTable(SCAN_REBUILD_TOMBSTONED_OBJECTS, SCAN_REBUILD_TOMBSTONED_OBJECT_WORDS, 4u, SCAN_REBUILD_TOMBSTONED_OBJECT_BUCKETS,
        SCAN_REBUILD_TOMBSTONED_OBJECT_BUCKET_COUNT);
}

fn scan_rebuild_data_unit_group_table() -> KeyedTable {
    return KeyedTable(SCAN_REBUILD_DATA_UNIT_GROUPS, SCAN_REBUILD_DATA_UNIT_GROUP_WORDS, 4u, SCAN_REBUILD_DATA_UNIT_GROUP_BUCKETS,
        SCAN_REBUILD_DATA_UNIT_GROUP_BUCKET_COUNT);
}

// ───────────────────────── 遍历方向那三张表（walk.rs index_tables_walked_from_the_newest_root）─────────────────────────

// walk.rs inode_record_bytes_without_judging：最新根那棵 inode 树的全部记录（号 → 记录原样）进 ⑩，同号后读到的盖掉先读到的。
// 走法与扫描内核的 d_read_inode_records 相同（根指针全零是零条记录；根层级 0 只许是零条目根；类型段 2 的叶容器读每条记录，类型段 0 的内部节点往下走），
// 只是每条记录记它在哪、不拆段（I-1.13 整条比）。读不全交回 false
fn scan_rebuild_read_the_walked_inode_records(root_pointer: Ptr) -> bool {
    scan_rebuild_walked_inode_record_count = 0u;
    scan_rebuild_walked_inode_records_are_indexed = false;
    if (root_pointer.all_zero) {
        return true;
    }
    let root = r_read_located_index_node(root_pointer);
    if (!root.ok) {
        return false;
    }
    if (root.level == 0u) {
        return root.entry_count == 0u;
    }
    if (root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
        return false;
    }
    var pending = 0u;
    if (!d_push_container_tree_entries_in_reverse(root, &pending, SCAN_REBUILD_SITE_INODE_TREE_PENDING_PAST_THE_STACK)) {
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
        if (child_pointer.all_zero
            || (entry_type != DELETION_PACKED_TYPE_INODE && entry_type != DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE)) {
            return false;
        }
        if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
            let child = r_read_located_index_node(child_pointer);
            if (!child.ok || child.level + 1u != parent_level || child.level == 0u
                || child.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
                return false;
            }
            if (!d_push_container_tree_entries_in_reverse(child, &pending, SCAN_REBUILD_SITE_INODE_TREE_PENDING_PAST_THE_STACK)) {
                return false;
            }
            continue;
        }
        let container = parse_packed_unit(r_read_located_unit(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
        if (!container.ok || container.record_type != DELETION_PACKED_TYPE_INODE
            || container.record_width < DELETION_INODE_RECORD_BYTES) {
            return false;
        }
        for (var record_index = 0u; record_index < container.record_count; record_index = record_index + 1u) {
            let record = packed_record_offset(container, record_index);
            let inode = rd64(container.base, record);
            var row_index = keyed_first_row_of(scan_rebuild_walked_inode_record_table(), scan_rebuild_walked_inode_record_count,
                scan_rebuild_walked_inode_records_are_indexed, vec4<u32>(inode.x, inode.y, 0u, 0u));
            if (row_index == NONE) {
                if (scan_rebuild_walked_inode_record_count >= SCAN_REBUILD_MAX_INODE_RECORDS) {
                    undecidable = SCAN_REBUILD_SITE_WALKED_INODE_RECORDS_PAST_THE_TABLE;
                    return false;
                }
                row_index = scan_rebuild_walked_inode_record_count;
                let appended_row = SCAN_REBUILD_WALKED_INODE_RECORDS + row_index * SCAN_REBUILD_WALKED_INODE_RECORD_WORDS;
                sws(appended_row, inode.x);
                sws(appended_row + 1u, inode.y);
                scan_rebuild_walked_inode_record_count = row_index + 1u;
                scan_rebuild_walked_inode_records_are_indexed = keyed_table_after_a_row_was_appended(scan_rebuild_walked_inode_record_table(),
                    scan_rebuild_walked_inode_record_count, scan_rebuild_walked_inode_records_are_indexed, row_index);
            }
            let walked_row = SCAN_REBUILD_WALKED_INODE_RECORDS + row_index * SCAN_REBUILD_WALKED_INODE_RECORD_WORDS;
            sws(walked_row + 2u, container.base);
            sws(walked_row + 3u, record);
            sws(walked_row + 4u, container.record_width);
            sws(walked_row + 5u, 0u);
        }
    }
    return true;
}

// 遍历方向的 inode 记录集合里号 inode 那一行（没有交回 NONE）
fn scan_rebuild_walked_inode_record_row_of(inode: vec2<u32>) -> u32 {
    let row_index = keyed_first_row_of(scan_rebuild_walked_inode_record_table(), scan_rebuild_walked_inode_record_count,
        scan_rebuild_walked_inode_records_are_indexed, vec4<u32>(inode.x, inode.y, 0u, 0u));
    if (row_index == NONE) {
        return NONE;
    }
    return SCAN_REBUILD_WALKED_INODE_RECORDS + row_index * SCAN_REBUILD_WALKED_INODE_RECORD_WORDS;
}

// 偏移 offset_in_bytes 的数据单元在不在 size 为 size_in_bytes 的记录罩的单元之外：偏移 ÷ 净荷容量不小于 ⌈size ÷ 净荷⌉.max(1)
// （walk.rs 与 rebuild.rs 各写一份的同一个式子）
fn scan_rebuild_offset_lies_beyond_the_data_units_the_size_asks_for(offset_in_bytes: vec2<u32>, size_in_bytes: vec2<u32>) -> bool {
    let data_unit_index = divide_by_a_divisor_below_65536(offset_in_bytes, DATA_UNIT_PAYLOAD_CAPACITY).quotient;
    return !lt64(data_unit_index, d_data_units_the_size_asks_for(size_in_bytes));
}

// 最新根中央映射那一版的表里 key 那一行的记号字（找不到交回 NONE；同一个 key 在叶里出现两次的，只第一行有记号）
fn scan_rebuild_mapping_row_flags_word_of(key: MappingKey) -> u32 {
    let row_index = version_mapping_first_row_of(key);
    if (row_index == NONE) {
        return NONE;
    }
    return SCAN_REBUILD_MAPPING_ROW_FLAGS + row_index;
}

fn scan_rebuild_mark_the_mapping_row_of(key: MappingKey, flag: u32) {
    let flags_word = scan_rebuild_mapping_row_flags_word_of(key);
    if (flags_word != NONE) {
        sws(flags_word, sw(flags_word) | flag);
    }
}

// walk.rs index_tables_walked_from_the_newest_root：最新根下面的三张表，只读不判。读不全时交回不适用的那一种结局码（walk.rs
// IndexTableTheWalkDoesNotReadInFull 的次序：树表、inode 树、extent 树、同一个 (inode, 偏移) 两条、中央映射树、树表里 deadlist 不止一条、
// deadlist 树、映射里同一个 key 两条），读全了交回 SCAN_REBUILD_OUTCOME_HOLDS（只当「读全了」用）。表在 ⑩、⑪ 与 ⑫ 的记号里
fn scan_rebuild_read_the_walked_tables(newest_root_base: u32) -> u32 {
    // inode 树、extent 树与 deadlist 树里豁免三类之外的单元按最新根自己的中央映射认位置（读不全时是空表，照提示）
    s_use_the_central_mapping_named_at(newest_root_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    let mapping_read_in_full = version_mapping_read_in_full;
    for (var row_index = 0u; row_index < version_mapping_count; row_index = row_index + 1u) {
        sws(SCAN_REBUILD_MAPPING_ROW_FLAGS + row_index, 0u);
    }
    // walk.rs user_visible_tree_root_pointers：树表指针全零、读不出、条目窄于字段表、inode 树或 extent 树出现两条
    let tree_table_pointer = parse_node_pointer(newest_root_base, G_ROOT_RECORD_TREE_TABLE_POINTER);
    if (tree_table_pointer.all_zero) {
        return SCAN_REBUILD_OUTCOME_TREE_TABLE_NOT_READ_IN_FULL;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES)) {
        return SCAN_REBUILD_OUTCOME_TREE_TABLE_NOT_READ_IN_FULL;
    }
    var inode_entries = 0u;
    var inode_entry = 0u;
    var extent_entries = 0u;
    var extent_entry = 0u;
    var deadlist_entries = 0u;
    var deadlist_entry = 0u;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
        if (kind == TREE_KIND_INODE) {
            inode_entries = inode_entries + 1u;
            inode_entry = entry;
        } else if (kind == TREE_KIND_EXTENT) {
            extent_entries = extent_entries + 1u;
            extent_entry = entry;
        } else if (kind == TREE_KIND_DEADLIST) {
            deadlist_entries = deadlist_entries + 1u;
            deadlist_entry = entry;
        }
    }
    if (inode_entries > 1u || extent_entries > 1u) {
        return SCAN_REBUILD_OUTCOME_TREE_TABLE_NOT_READ_IN_FULL;
    }
    scan_rebuild_walked_inode_record_count = 0u;
    scan_rebuild_walked_inode_records_are_indexed = false;
    if (inode_entries == 1u) {
        let read_the_whole_inode_tree =
            scan_rebuild_read_the_walked_inode_records(parse_node_pointer(tt.base, inode_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET));
        if (undecidable != 0u) {
            return SCAN_REBUILD_OUTCOME_NOT_REACHED;
        }
        if (!read_the_whole_inode_tree) {
            return SCAN_REBUILD_OUTCOME_INODE_TREE_NOT_READ_IN_FULL;
        }
    }
    // walk.rs extent_tree_with_the_inode_of_each_data_pointer_without_judging：每个号的数据单元进 D_DATA_UNITS（扫描内核删除态扫描那一份读法）
    scan_rebuild_walked_position_count = 0u;
    scan_rebuild_walked_positions_are_indexed = false;
    d_data_unit_inode_count = 0u;
    d_data_unit_count = 0u;
    let extent_root_offset = extent_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    if (extent_entries == 1u && !parse_node_pointer(tt.base, extent_root_offset).all_zero) {
        r_noting = false;
        r_collecting_data_unit_slots = false;
        r_unit_keys_go_to = 0u;
        r_counting_data_units_of_each_inode = true;
        let read_the_whole_extent_tree = r_extent_tree(tt.base, extent_root_offset);
        r_counting_data_units_of_each_inode = false;
        if (undecidable != 0u) {
            return SCAN_REBUILD_OUTCOME_NOT_REACHED;
        }
        if (!read_the_whole_extent_tree) {
            return SCAN_REBUILD_OUTCOME_EXTENT_TREE_NOT_READ_IN_FULL;
        }
    }
    // 每个单元：size 之外的（inode 树里有它那个号的记录、偏移在 size 罩的单元之外）不收、它的映射 key 记下（映射那一张不收它）；
    // 其余按 (inode, 偏移) 收，同一个位置两条交回不适用
    for (var inode_index = 0u; inode_index < d_data_unit_inode_count; inode_index = inode_index + 1u) {
        let inode_row = D_DATA_UNITS_OF_EACH_INODE + inode_index * D_DATA_UNITS_OF_AN_INODE_WORDS;
        let inode = vec2<u32>(sw(inode_row), sw(inode_row + 1u));
        let record_row = scan_rebuild_walked_inode_record_row_of(inode);
        var unit_plus_one = sw(inode_row + 4u);
        // 迭代上界：这个号记进的单元数（链表每一步往前一个，记的时候下标只增）
        while (unit_plus_one != 0u) {
            let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
            unit_plus_one = sw(unit + 4u);
            let offset_in_bytes = vec2<u32>(sw(unit), sw(unit + 1u));
            let key = mapping_key_of_a_pointer(parse_data_pointer(sw(unit + 2u), sw(unit + 3u)), MAPPING_CLASS_DATA_UNIT);
            if (record_row != NONE && scan_rebuild_offset_lies_beyond_the_data_units_the_size_asks_for(offset_in_bytes,
                rd64(sw(record_row + 2u), sw(record_row + 3u) + INODE_RECORD_SIZE_OFFSET))) {
                scan_rebuild_mark_the_mapping_row_of(key, SCAN_REBUILD_MAPPING_ROW_OF_A_DATA_UNIT_BEYOND_THE_SIZE);
                continue;
            }
            let position = vec4<u32>(inode.x, inode.y, offset_in_bytes.x, offset_in_bytes.y);
            if (keyed_first_row_of(scan_rebuild_walked_position_table(), scan_rebuild_walked_position_count,
                scan_rebuild_walked_positions_are_indexed, position) != NONE) {
                return SCAN_REBUILD_OUTCOME_EXTENT_TREE_NAMES_ONE_POSITION_TWICE;
            }
            let row_index = scan_rebuild_walked_position_count;
            let row = SCAN_REBUILD_WALKED_POSITIONS + row_index * SCAN_REBUILD_WALKED_POSITION_WORDS;
            sws(row, position.x);
            sws(row + 1u, position.y);
            sws(row + 2u, position.z);
            sws(row + 3u, position.w);
            put_the_mapping_key_into_the_words(row + 4u, key);
            sws(row + 11u, 0u);
            scan_rebuild_walked_position_count = row_index + 1u;
            scan_rebuild_walked_positions_are_indexed = keyed_table_after_a_row_was_appended(scan_rebuild_walked_position_table(),
                scan_rebuild_walked_position_count, scan_rebuild_walked_positions_are_indexed, row_index);
        }
    }
    // 中央映射树：根指针非零而读不全
    if (!mapping_read_in_full) {
        return SCAN_REBUILD_OUTCOME_CENTRAL_MAPPING_TREE_NOT_READ_IN_FULL;
    }
    // walk.rs mapping_keys_named_by_the_deadlist_of_the_newest_root：树表里 deadlist 树条目不止一条、deadlist 树读不全
    if (deadlist_entries > 1u) {
        return SCAN_REBUILD_OUTCOME_TREE_TABLE_NOT_READ_IN_FULL;
    }
    if (deadlist_entries == 1u) {
        let deadlist_root = parse_node_pointer(tt.base, deadlist_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
        if (!deadlist_root.all_zero) {
            let deadlist_read = s_read_the_deadlist_leaf_entries(deadlist_root);
            if (undecidable != 0u) {
                return SCAN_REBUILD_OUTCOME_NOT_REACHED;
            }
            if (!deadlist_read) {
                return SCAN_REBUILD_OUTCOME_DEADLIST_TREE_NOT_READ_IN_FULL;
            }
            for (var entry_index = 0u; entry_index < s_deadlist_entry_count; entry_index = entry_index + 1u) {
                let deadlist_row = S_DEADLIST_ENTRIES + entry_index * 2u;
                scan_rebuild_mark_the_mapping_row_of(mapping_key_of_the_bytes(sw(deadlist_row), sw(deadlist_row + 1u) + DEADLIST_ENTRY_MAPPING_KEY_OFFSET),
                    SCAN_REBUILD_MAPPING_ROW_PINNED_BY_THE_DEADLIST);
            }
        }
    }
    // 遍历方向那一张中央映射：码 1 / 码 3 的条目，不收 deadlist 点名的与 size 之外的单元的；同一个 key 两条交回不适用
    for (var row_index = 0u; row_index < version_mapping_count; row_index = row_index + 1u) {
        let key = version_mapping_key_of_row(row_index);
        let unit_class = key.class_and_tail_high & 0xFFu;
        let flags = sw(SCAN_REBUILD_MAPPING_ROW_FLAGS + row_index);
        if ((unit_class != 1u && unit_class != 3u) || version_mapping_first_row_of(key) != row_index
            || (flags & (SCAN_REBUILD_MAPPING_ROW_PINNED_BY_THE_DEADLIST | SCAN_REBUILD_MAPPING_ROW_OF_A_DATA_UNIT_BEYOND_THE_SIZE)) != 0u) {
            continue;
        }
        if (sw(C_VERSION_MAPPING_ROWS + row_index * C_VERSION_MAPPING_ROW_WORDS + 7u) != 0u) {
            return SCAN_REBUILD_OUTCOME_CENTRAL_MAPPING_TREE_NAMES_ONE_KEY_TWICE;
        }
        sws(SCAN_REBUILD_MAPPING_ROW_FLAGS + row_index, flags | SCAN_REBUILD_MAPPING_ROW_IN_THE_WALKED_TABLE);
    }
    return SCAN_REBUILD_OUTCOME_HOLDS;
}

// ───────────────────────── 扫描方向：收单元、择当前版本（rebuild.rs choose_current_versions）─────────────────────────

// walk.rs scanned_content_units：单元槽上的码 1 / 码 3 单元，头校验和过、fsid 对、按已发布谓词判成已发布（收法与扫描内核 I-1.8 那一遍逐条相同）。
// 每项记读出来那一份切片的字节基与落点（盘、槽）。多于 SCAN_REBUILD_MAX_UNITS 项判不了、交回 false
fn scan_rebuild_collect_the_published_units(fsid_low: vec2<u32>) -> bool {
    scan_rebuild_unit_count = 0u;
    let location_count = hdr(6u);
    for (var location = 0u; location < location_count; location = location + 1u) {
        if (location_word(location, 4u) != LOCATION_UNIT_SLOT) {
            continue;
        }
        let s = read_location(location);
        if (s.version == NONE || s.length < 4096u || rd32(s.base, 0u) != MAGIC_SFSU) {
            continue;
        }
        let unit_class = rd8(s.base, 6u);
        if (unit_class != 1u && unit_class != 3u) {
            continue;
        }
        let is_a_data_unit = unit_class == 1u;
        let fsid_offset = select(81u, 83u, is_a_data_unit);
        let write_order_offset = select(93u, 91u, is_a_data_unit);
        if ((s.c2 & 1u) == 0u || !eq64(rd64(s.base, fsid_offset), fsid_low)) {
            continue;
        }
        if (!w_published(rd64(s.base, scan_rebuild_birth_txg_offset(unit_class)), rd32(s.base, write_order_offset), is_a_data_unit,
            rd48(s.base, write_order_offset + 4u))) {
            continue;
        }
        if (scan_rebuild_unit_count >= SCAN_REBUILD_MAX_UNITS) {
            undecidable = SCAN_REBUILD_SITE_UNITS_PAST_THE_TABLE;
            return false;
        }
        let row = SCAN_REBUILD_UNITS + scan_rebuild_unit_count * SCAN_REBUILD_UNIT_WORDS;
        let slot = shr64(vec2<u32>(location_word(location, 1u), location_word(location, 2u)), SLOT_SHIFT);
        sws(row, s.base);
        sws(row + 1u, location_word(location, 0u));
        sws(row + 2u, slot.x);
        sws(row + 3u, slot.y);
        scan_rebuild_unit_count = scan_rebuild_unit_count + 1u;
    }
    return true;
}

// 单元头里诞生代号的偏移：码 1 是 75、码 3 是 73
fn scan_rebuild_birth_txg_offset(unit_class: u32) -> u32 {
    return select(73u, 75u, unit_class == 1u);
}

// 次序里第 position 项的切片字节基
fn scan_rebuild_base_at(position: u32) -> u32 {
    return sw(SCAN_REBUILD_UNITS + sw(SCAN_REBUILD_UNIT_ORDER + position) * SCAN_REBUILD_UNIT_WORDS);
}

// 对象身份（rebuild.rs 的 ObjectIdentity：类身份段里与版本无关的那一段，从偏移 42 起）的字节数：码 1 的五元组 33、码 3 的 27
fn scan_rebuild_object_identity_bytes(unit_class: u32) -> u32 {
    return select(27u, 33u, unit_class == 1u);
}

// C113 定案 P3 的择新键（rebuild.rs CurrentVersionOrderKey）：码 1 与码 3 同形，(诞生代号, 实例代号, 事务号)，诞生代号打头
// （码 1 那一半是 C464 用户 2026-10-26 定的「① 加诞生代号打头」：一个事务跨两个 checkpoint 时两版写序逐字节相同，只有诞生代号分得开）
struct ScanRebuildOrderKey {
    birth_txg: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
};

fn scan_rebuild_order_key_of(base: u32) -> ScanRebuildOrderKey {
    var key: ScanRebuildOrderKey;
    if (rd8(base, 6u) == 1u) {
        key.birth_txg = rd64(base, 75u);
        key.instance = rd32(base, 91u);
        key.transaction = rd48(base, 95u);
    } else {
        key.birth_txg = rd64(base, 73u);
        key.instance = rd32(base, 93u);
        key.transaction = rd48(base, 97u);
    }
    return key;
}

fn scan_rebuild_order_keys_are_equal(a: ScanRebuildOrderKey, b: ScanRebuildOrderKey) -> bool {
    return eq64(a.birth_txg, b.birth_txg) && a.instance == b.instance && eq64(a.transaction, b.transaction);
}

fn scan_rebuild_order_key_is_less(a: ScanRebuildOrderKey, b: ScanRebuildOrderKey) -> bool {
    if (!eq64(a.birth_txg, b.birth_txg)) {
        return lt64(a.birth_txg, b.birth_txg);
    }
    if (a.instance != b.instance) {
        return a.instance < b.instance;
    }
    return lt64(a.transaction, b.transaction);
}

// 两份头从偏移 42 起的对象身份有几个字节相同（至多 bytes 个）
fn scan_rebuild_common_object_identity_prefix(base_a: u32, base_b: u32, bytes: u32) -> u32 {
    var position = 0u;
    while (position < bytes && rd8(base_a, SCAN_REBUILD_OBJECT_IDENTITY_START + position) == rd8(base_b, SCAN_REBUILD_OBJECT_IDENTITY_START + position)) {
        position = position + 1u;
    }
    return position;
}

// 两份同类的头的归并键（扫描内核 I-1.8 那一份换过次序的写法 c_merge_key_byte_offset）从头起有几个字节相同
fn scan_rebuild_common_merge_key_prefix(base_a: u32, base_b: u32, unit_class: u32) -> u32 {
    let key_bytes = c_merge_key_bytes(unit_class);
    var position = 0u;
    while (position < key_bytes) {
        let offset = c_merge_key_byte_offset(unit_class, position);
        if (rd8(base_a, offset) != rd8(base_b, offset)) {
            break;
        }
        position = position + 1u;
    }
    return position;
}

// 扫描项 a 排在 b 前面：对象身份按字节比（rebuild.rs 的 BTreeMap<ObjectIdentity, …> 次序：逐字节比、短的在前），同一个对象身份按择新键升序，
// 择新键相同的按归并键（同一版的副本挨在一起），再按收的次序（同一版里头一份就是 rebuild.rs 认的那一份明文头，副本按落点次序）——全序
fn scan_rebuild_unit_precedes(a: u32, b: u32) -> bool {
    let base_a = sw(SCAN_REBUILD_UNITS + a * SCAN_REBUILD_UNIT_WORDS);
    let base_b = sw(SCAN_REBUILD_UNITS + b * SCAN_REBUILD_UNIT_WORDS);
    let class_a = rd8(base_a, 6u);
    let identity_bytes_a = scan_rebuild_object_identity_bytes(class_a);
    let identity_bytes_b = scan_rebuild_object_identity_bytes(rd8(base_b, 6u));
    let shorter = min(identity_bytes_a, identity_bytes_b);
    let identity_prefix = scan_rebuild_common_object_identity_prefix(base_a, base_b, shorter);
    if (identity_prefix < shorter) {
        return rd8(base_a, SCAN_REBUILD_OBJECT_IDENTITY_START + identity_prefix) < rd8(base_b, SCAN_REBUILD_OBJECT_IDENTITY_START + identity_prefix);
    }
    if (identity_bytes_a != identity_bytes_b) {
        return identity_bytes_a < identity_bytes_b;
    }
    let order_a = scan_rebuild_order_key_of(base_a);
    let order_b = scan_rebuild_order_key_of(base_b);
    if (!scan_rebuild_order_keys_are_equal(order_a, order_b)) {
        return scan_rebuild_order_key_is_less(order_a, order_b);
    }
    let merge_prefix = scan_rebuild_common_merge_key_prefix(base_a, base_b, class_a);
    if (merge_prefix < c_merge_key_bytes(class_a)) {
        let offset = c_merge_key_byte_offset(class_a, merge_prefix);
        return rd8(base_a, offset) < rd8(base_b, offset);
    }
    return a < b;
}

// 扫描项按 scan_rebuild_unit_precedes 排进 SCAN_REBUILD_UNIT_ORDER。堆排序，写法同 order_rows_by_their_first_four_words
fn scan_rebuild_order_the_units() {
    let count = scan_rebuild_unit_count;
    for (var i = 0u; i < count; i = i + 1u) {
        sws(SCAN_REBUILD_UNIT_ORDER + i, i);
    }
    if (count < 2u) {
        return;
    }
    let heap_building_steps = count / 2u;
    var heap_size = count;
    for (var step = 0u; step < heap_building_steps + count - 1u; step = step + 1u) {
        var node = 0u;
        if (step < heap_building_steps) {
            node = heap_building_steps - 1u - step;
        } else {
            heap_size = heap_size - 1u;
            let largest = sw(SCAN_REBUILD_UNIT_ORDER);
            sws(SCAN_REBUILD_UNIT_ORDER, sw(SCAN_REBUILD_UNIT_ORDER + heap_size));
            sws(SCAN_REBUILD_UNIT_ORDER + heap_size, largest);
        }
        loop {
            let left = 2u * node + 1u;
            if (left >= heap_size) {
                break;
            }
            var larger = left;
            if (left + 1u < heap_size && scan_rebuild_unit_precedes(sw(SCAN_REBUILD_UNIT_ORDER + left), sw(SCAN_REBUILD_UNIT_ORDER + left + 1u))) {
                larger = left + 1u;
            }
            if (!scan_rebuild_unit_precedes(sw(SCAN_REBUILD_UNIT_ORDER + node), sw(SCAN_REBUILD_UNIT_ORDER + larger))) {
                break;
            }
            let parent = sw(SCAN_REBUILD_UNIT_ORDER + node);
            sws(SCAN_REBUILD_UNIT_ORDER + node, sw(SCAN_REBUILD_UNIT_ORDER + larger));
            sws(SCAN_REBUILD_UNIT_ORDER + larger, parent);
            node = larger;
        }
    }
}

// 从次序里第 start 项起、同一个对象身份的那一段到哪里为止
fn scan_rebuild_object_end(start: u32) -> u32 {
    let base = scan_rebuild_base_at(start);
    let unit_class = rd8(base, 6u);
    let identity_bytes = scan_rebuild_object_identity_bytes(unit_class);
    var end = start + 1u;
    while (end < scan_rebuild_unit_count) {
        let other = scan_rebuild_base_at(end);
        if (rd8(other, 6u) != unit_class || scan_rebuild_common_object_identity_prefix(base, other, identity_bytes) != identity_bytes) {
            break;
        }
        end = end + 1u;
    }
    return end;
}

// 从次序里第 start 项起、同一版（归并键相同，归并键罩着对象身份）的那一段到哪里为止
fn scan_rebuild_version_end(start: u32) -> u32 {
    let base = scan_rebuild_base_at(start);
    let unit_class = rd8(base, 6u);
    let key_bytes = c_merge_key_bytes(unit_class);
    var end = start + 1u;
    while (end < scan_rebuild_unit_count) {
        let other = scan_rebuild_base_at(end);
        if (rd8(other, 6u) != unit_class || scan_rebuild_common_merge_key_prefix(base, other, unit_class) != key_bytes) {
            break;
        }
        end = end + 1u;
    }
    return end;
}

// 一个对象身份择当前版本走哪一步（rebuild.rs current_version_is_left_for_the_lineage）：码 1 与 inode 叶容器（码 3 打包记录类型 2）随回退走
fn scan_rebuild_is_left_for_the_lineage(base: u32) -> bool {
    return rd8(base, 6u) == 1u || rd16(base, SCAN_REBUILD_PACKED_RECORD_TYPE_OFFSET) == DELETION_PACKED_TYPE_INODE;
}

// rebuild.rs choose_current_versions 判的两格：同一版的副本头里载荷校验和不同（不论在哪个对象上都先报它），随现行那一版走的容器最大的两版择新键相同
fn scan_rebuild_outcome_of_choosing_among_every_published_version() -> u32 {
    var tied = false;
    var object_start = 0u;
    while (object_start < scan_rebuild_unit_count) {
        let object_end = scan_rebuild_object_end(object_start);
        var previous_version_start = NONE;
        var last_version_start = object_start;
        var version_start = object_start;
        while (version_start < object_end) {
            let version_end = scan_rebuild_version_end(version_start);
            let payload_checksum = c_payload_checksum(scan_rebuild_base_at(version_start));
            for (var position = version_start + 1u; position < version_end; position = position + 1u) {
                if (c_payload_checksum(scan_rebuild_base_at(position)) != payload_checksum) {
                    return SCAN_REBUILD_OUTCOME_COPIES_OF_ONE_VERSION_CARRY_DIFFERENT_PAYLOADS;
                }
            }
            previous_version_start = select(NONE, last_version_start, version_start != object_start);
            last_version_start = version_start;
            version_start = version_end;
        }
        if (!scan_rebuild_is_left_for_the_lineage(scan_rebuild_base_at(object_start)) && previous_version_start != NONE
            && scan_rebuild_order_keys_are_equal(scan_rebuild_order_key_of(scan_rebuild_base_at(previous_version_start)),
                scan_rebuild_order_key_of(scan_rebuild_base_at(last_version_start)))) {
            tied = true;
        }
        object_start = object_end;
    }
    return select(SCAN_REBUILD_OUTCOME_HOLDS, SCAN_REBUILD_OUTCOME_PACKED_RECORD_VERSIONS_TIED_ON_THE_TOTAL_ORDER_KEY, tied);
}

// ───────────────────────── 重建三张表（rebuild.rs rebuild_index_tables）─────────────────────────

// 一个当前版本的映射 key，从它的明文头现算（rebuild.rs central_mapping_key_of_the_header）：类标签取偏移 6，出生树、诞生代号、写序实例代号各取头里那一段，
// 尾段码 1 是写序的事务号 6 字节、码 3 是出生序号 4 字节加补零 2
fn scan_rebuild_mapping_key_of_the_header(base: u32) -> MappingKey {
    let unit_class = rd8(base, 6u);
    var key: MappingKey;
    key.birth_tree = rd64(base, 43u);
    key.birth_txg = rd64(base, scan_rebuild_birth_txg_offset(unit_class));
    if (unit_class == 1u) {
        let transaction = rd48(base, 95u);
        key.class_and_tail_high = unit_class | ((transaction.y & 0xFFFFu) << 16u);
        key.instance = rd32(base, 91u);
        key.tail_low = transaction.x;
    } else {
        key.class_and_tail_high = unit_class;
        key.instance = rd32(base, 93u);
        key.tail_low = rd32(base, SCAN_REBUILD_PACKED_BIRTH_SEQUENCE_OFFSET);
    }
    return key;
}

// 重建出的中央映射里键是 key 的那一行的下标（没有交回 NONE）：条数没过界时从表头比，过了界按桶表找
fn scan_rebuild_rebuilt_mapping_row_of(key: MappingKey) -> u32 {
    var bucket = mapping_key_first_bucket(key, SCAN_REBUILD_MAPPING_BUCKET_COUNT);
    let steps = select(scan_rebuild_mapping_count, SCAN_REBUILD_MAPPING_BUCKET_COUNT, scan_rebuild_mapping_is_indexed);
    for (var step = 0u; step < steps; step = step + 1u) {
        var row_index = step;
        if (scan_rebuild_mapping_is_indexed) {
            let row_plus_one = sw(SCAN_REBUILD_MAPPING_BUCKETS + bucket);
            if (row_plus_one == 0u) {
                return NONE;
            }
            row_index = row_plus_one - 1u;
            bucket = (bucket + 1u) & (SCAN_REBUILD_MAPPING_BUCKET_COUNT - 1u);
        }
        if (same_mapping_key(mapping_key_of_the_words(SCAN_REBUILD_MAPPING + row_index * SCAN_REBUILD_MAPPING_WORDS), key)) {
            return row_index;
        }
    }
    return NONE;
}

// 重建出的中央映射记一条（rebuild.rs 的 central_mapping.insert：同一个 key 后记的盖掉先记的）：key → 一版的副本（次序里从 version_start 起 member_count 项）
fn scan_rebuild_put_into_the_rebuilt_mapping(key: MappingKey, version_start: u32, member_count: u32) {
    var row_index = scan_rebuild_rebuilt_mapping_row_of(key);
    if (row_index == NONE) {
        // 一版至多一行，版本数不过扫描项数：表装得下
        row_index = scan_rebuild_mapping_count;
        put_the_mapping_key_into_the_words(SCAN_REBUILD_MAPPING + row_index * SCAN_REBUILD_MAPPING_WORDS, key);
        scan_rebuild_mapping_count = row_index + 1u;
        let indexed = scan_rebuild_mapping_is_indexed || scan_rebuild_mapping_count > KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST;
        var first_row_to_put = row_index;
        if (indexed && !scan_rebuild_mapping_is_indexed) {
            for (var bucket = 0u; bucket < SCAN_REBUILD_MAPPING_BUCKET_COUNT; bucket = bucket + 1u) {
                sws(SCAN_REBUILD_MAPPING_BUCKETS + bucket, 0u);
            }
            first_row_to_put = 0u;
        }
        scan_rebuild_mapping_is_indexed = indexed;
        for (var row_to_put = first_row_to_put; indexed && row_to_put < scan_rebuild_mapping_count; row_to_put = row_to_put + 1u) {
            var bucket = mapping_key_first_bucket(mapping_key_of_the_words(SCAN_REBUILD_MAPPING + row_to_put * SCAN_REBUILD_MAPPING_WORDS),
                SCAN_REBUILD_MAPPING_BUCKET_COUNT);
            for (var probe = 0u; probe < SCAN_REBUILD_MAPPING_BUCKET_COUNT; probe = probe + 1u) {
                if (sw(SCAN_REBUILD_MAPPING_BUCKETS + bucket) == 0u) {
                    sws(SCAN_REBUILD_MAPPING_BUCKETS + bucket, row_to_put + 1u);
                    break;
                }
                bucket = (bucket + 1u) & (SCAN_REBUILD_MAPPING_BUCKET_COUNT - 1u);
            }
        }
    }
    let row = SCAN_REBUILD_MAPPING + row_index * SCAN_REBUILD_MAPPING_WORDS;
    sws(row + 7u, version_start);
    sws(row + 8u, member_count);
    sws(row + 9u, 0u);
}

// 一版码 3 容器解得开的那一份（rebuild.rs records_of_a_current_container）：按落点次序读这一版的每一份副本（32 KiB），取第一份明文头与这一版
// 头一份逐字节相同、解得开、记录类型与记录宽对得上的；一份都没有交回 NONE，有就交回它的字节基
fn scan_rebuild_decoded_copy_of_a_container_version(version_start: u32, version_end: u32, record_type: u32, record_width: u32) -> u32 {
    let header = scan_rebuild_base_at(version_start);
    for (var position = version_start; position < version_end; position = position + 1u) {
        let row = SCAN_REBUILD_UNITS + sw(SCAN_REBUILD_UNIT_ORDER + position) * SCAN_REBUILD_UNIT_WORDS;
        let s = read_at(sw(row + 1u), slot_to_bytes(vec2<u32>(sw(row + 2u), sw(row + 3u))), DATA_UNIT_BYTES);
        if (s.version == NONE || !bytes_equal(s.base, 0u, header, 0u, 107u)) {
            continue;
        }
        let container = parse_packed_unit(s);
        if (container.ok && container.record_type == record_type && container.record_width == record_width) {
            return container.base;
        }
    }
    return NONE;
}

// 一个对象身份的最后一版（择新键最大）在次序里从哪起；对象只有一版时就是 object_start
fn scan_rebuild_last_version_start(object_start: u32, object_end: u32) -> u32 {
    var last_version_start = object_start;
    var version_start = object_start;
    while (version_start < object_end) {
        last_version_start = version_start;
        version_start = scan_rebuild_version_end(version_start);
    }
    return last_version_start;
}

// 随现行那一版走的树的容器（rebuild.rs rebuild_index_tables 第一个循环）：最大的那一版，实例表的片之外进中央映射；墓碑、退役、谱系三种读记录——
// 谱系容器照扫描内核的写法记进 D_LINEAGE_CONTAINERS（字节基、记录数、第一条的偏移），墓碑与退役记进 ③。解不开交回不适用
fn scan_rebuild_outcome_of_the_containers_chosen_among_every_published_version() -> u32 {
    scan_rebuild_container_count = 0u;
    d_lineage_container_count = 0u;
    var object_start = 0u;
    while (object_start < scan_rebuild_unit_count) {
        let object_end = scan_rebuild_object_end(object_start);
        let base = scan_rebuild_base_at(object_start);
        if (scan_rebuild_is_left_for_the_lineage(base)) {
            object_start = object_end;
            continue;
        }
        let version_start = scan_rebuild_last_version_start(object_start, object_end);
        let record_type = rd16(base, SCAN_REBUILD_PACKED_RECORD_TYPE_OFFSET);
        if (record_type != SCAN_REBUILD_INSTANCE_TABLE_RECORD_TYPE) {
            scan_rebuild_put_into_the_rebuilt_mapping(scan_rebuild_mapping_key_of_the_header(scan_rebuild_base_at(version_start)), version_start,
                object_end - version_start);
        }
        var record_width = 0u;
        if (record_type == PACKED_TYPE_TOMBSTONE) {
            record_width = DELETION_TOMBSTONE_RECORD_BYTES;
        } else if (record_type == PACKED_TYPE_RETIREMENT) {
            record_width = DELETION_RETIREMENT_RECORD_BYTES;
        } else if (record_type == PACKED_TYPE_LINEAGE) {
            record_width = DELETION_LINEAGE_RECORD_BYTES;
        }
        if (record_width != 0u) {
            let container = scan_rebuild_decoded_copy_of_a_container_version(version_start, object_end, record_type, record_width);
            if (container == NONE) {
                return SCAN_REBUILD_OUTCOME_CURRENT_PACKED_CONTAINER_DECODES_FROM_NO_COPY;
            }
            let record_count = rd16(container, 69u);
            if (record_type == PACKED_TYPE_LINEAGE) {
                if (d_lineage_container_count >= D_MAX_CONTAINERS_OF_A_CONTAINER_TREE) {
                    undecidable = SCAN_REBUILD_SITE_LINEAGE_CONTAINERS_PAST_THE_TABLE;
                    return SCAN_REBUILD_OUTCOME_NOT_REACHED;
                }
                let lineage_row = d_lineage_container_row(d_lineage_container_count);
                sws(lineage_row, container);
                sws(lineage_row + 1u, record_count);
                sws(lineage_row + 2u, SCAN_REBUILD_PACKED_RECORDS_START);
                d_lineage_container_count = d_lineage_container_count + 1u;
            } else {
                let row = SCAN_REBUILD_CONTAINERS + scan_rebuild_container_count * SCAN_REBUILD_CONTAINER_WORDS;
                sws(row, record_type);
                sws(row + 1u, container);
                sws(row + 2u, record_count);
                scan_rebuild_container_count = scan_rebuild_container_count + 1u;
            }
        }
        object_start = object_end;
    }
    return SCAN_REBUILD_OUTCOME_HOLDS;
}

// 谱系记录接得成一条父链（rebuild.rs txgs_on_the_lineage_of_the_newest_version 开头的两格）：每条 T_old < T_rb、没有两条同一个 T_rb。
// 叶序（这里是容器在次序里的先后、容器里的记录序）里 T_rb 严格升序就没有两条相同；不是时两两比
fn scan_rebuild_lineage_records_form_a_parent_chain() -> bool {
    var records_rise = true;
    var has_a_previous_record = false;
    var previous_rollback_txg = vec2<u32>(0u, 0u);
    for (var container_index = 0u; container_index < d_lineage_container_count; container_index = container_index + 1u) {
        for (var record_index = 0u; record_index < d_lineage_container_record_count(container_index); record_index = record_index + 1u) {
            let position = LineageRecordPosition(container_index, record_index);
            let rollback_txg = d_lineage_rollback_txg(position);
            if (!lt64(d_lineage_parent_txg(position), rollback_txg)) {
                return false;
            }
            if (has_a_previous_record && !lt64(previous_rollback_txg, rollback_txg)) {
                records_rise = false;
            }
            has_a_previous_record = true;
            previous_rollback_txg = rollback_txg;
        }
    }
    if (records_rise) {
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

// 「在谱系上」（rebuild.rs TxgsOnTheLineageOfTheNewestVersion::holds，父链的起点取 2⁶⁴ − 1，重建不读根）：扫描内核的 d_txg_is_on_the_lineage 走同一条链
fn scan_rebuild_txg_is_on_the_lineage(txg: vec2<u32>) -> bool {
    return d_txg_is_on_the_lineage(txg, vec2<u32>(0xFFFFFFFFu, 0xFFFFFFFFu));
}

// 墓碑（rebuild.rs rebuild_index_tables 里墓碑那一个循环）：区间不是整对象交回不适用；死亡 txg 在谱系上的墓碑判 (号, 对象出生代) 死，记进 ⑨
fn scan_rebuild_outcome_of_the_tombstones() -> u32 {
    scan_rebuild_tombstoned_object_count = 0u;
    scan_rebuild_tombstoned_objects_are_indexed = false;
    for (var container_index = 0u; container_index < scan_rebuild_container_count; container_index = container_index + 1u) {
        let row = SCAN_REBUILD_CONTAINERS + container_index * SCAN_REBUILD_CONTAINER_WORDS;
        if (sw(row) != PACKED_TYPE_TOMBSTONE) {
            continue;
        }
        let base = sw(row + 1u);
        for (var record_index = 0u; record_index < sw(row + 2u); record_index = record_index + 1u) {
            let o = SCAN_REBUILD_PACKED_RECORDS_START + record_index * DELETION_TOMBSTONE_RECORD_BYTES;
            if (!d_tombstone_covers_the_whole_object(base, o)) {
                return SCAN_REBUILD_OUTCOME_TOMBSTONE_RANGE_OTHER_THAN_THE_WHOLE_OBJECT;
            }
            if (!scan_rebuild_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET))) {
                continue;
            }
            let object = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
            let object_birth = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
            let key = vec4<u32>(object.x, object.y, object_birth.x, object_birth.y);
            if (keyed_first_row_of(scan_rebuild_tombstoned_object_table(), scan_rebuild_tombstoned_object_count,
                scan_rebuild_tombstoned_objects_are_indexed, key) != NONE) {
                continue;
            }
            if (scan_rebuild_tombstoned_object_count >= SCAN_REBUILD_MAX_TOMBSTONED_OBJECTS) {
                undecidable = SCAN_REBUILD_SITE_TOMBSTONED_OBJECTS_PAST_THE_TABLE;
                return SCAN_REBUILD_OUTCOME_NOT_REACHED;
            }
            let row_index = scan_rebuild_tombstoned_object_count;
            let object_row = SCAN_REBUILD_TOMBSTONED_OBJECTS + row_index * SCAN_REBUILD_TOMBSTONED_OBJECT_WORDS;
            sws(object_row, key.x);
            sws(object_row + 1u, key.y);
            sws(object_row + 2u, key.z);
            sws(object_row + 3u, key.w);
            scan_rebuild_tombstoned_object_count = row_index + 1u;
            scan_rebuild_tombstoned_objects_are_indexed = keyed_table_after_a_row_was_appended(scan_rebuild_tombstoned_object_table(),
                scan_rebuild_tombstoned_object_count, scan_rebuild_tombstoned_objects_are_indexed, row_index);
        }
    }
    return SCAN_REBUILD_OUTCOME_HOLDS;
}

// 一片 inode 叶容器（对象身份的头在 base）被谱系上的退役记录点名时最早的那个死亡 txg（rebuild.rs 的 retired_leaf_containers：被退役身份的打包记录类型是 2、
// 死亡 txg 在谱系上、身份与码 3 对象身份里类标签之后那 26 字节逐字节相同）；没被点名 is_retired 为假
struct ScanRebuildRetirement {
    is_retired: bool,
    earliest_death_txg: vec2<u32>,
};

fn scan_rebuild_retirement_of_the_leaf_container(base: u32) -> ScanRebuildRetirement {
    var retirement: ScanRebuildRetirement;
    retirement.is_retired = false;
    retirement.earliest_death_txg = vec2<u32>(0u, 0u);
    for (var container_index = 0u; container_index < scan_rebuild_container_count; container_index = container_index + 1u) {
        let row = SCAN_REBUILD_CONTAINERS + container_index * SCAN_REBUILD_CONTAINER_WORDS;
        if (sw(row) != PACKED_TYPE_RETIREMENT) {
            continue;
        }
        let container = sw(row + 1u);
        for (var record_index = 0u; record_index < sw(row + 2u); record_index = record_index + 1u) {
            let o = SCAN_REBUILD_PACKED_RECORDS_START + record_index * DELETION_RETIREMENT_RECORD_BYTES;
            if (rd16(container, o + DELETION_RETIREMENT_RECORD_RECORD_TYPE_OFFSET) != DELETION_PACKED_TYPE_INODE
                || !bytes_equal(container, o, base, SCAN_REBUILD_OBJECT_IDENTITY_START + 1u, SCAN_REBUILD_RETIRED_IDENTITY_BYTES)) {
                continue;
            }
            let death_txg = rd64(container, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET);
            if (!scan_rebuild_txg_is_on_the_lineage(death_txg)) {
                continue;
            }
            if (!retirement.is_retired || lt64(death_txg, retirement.earliest_death_txg)) {
                retirement.earliest_death_txg = death_txg;
            }
            retirement.is_retired = true;
        }
    }
    return retirement;
}

// 随回退走的那几类（rebuild.rs rebuild_index_tables 第二个循环，按对象身份的次序）：码 1 与 inode 叶容器只在诞生代号落在谱系上的版本里择最大的一版，
// 最大的两版择新键相同交回不适用；码 1 记进 ⑥ 留给数据单元那一步；inode 叶容器被谱系上的退役记录点名的不认领（择出的那一版诞生代号晚于死亡 txg 交回不适用），
// 否则进中央映射、读记录进 ⑦（解不开、同一个号在两片当前容器里交回不适用）
fn scan_rebuild_outcome_of_the_versions_left_for_the_lineage() -> u32 {
    scan_rebuild_data_unit_count = 0u;
    scan_rebuild_data_unit_group_count = 0u;
    scan_rebuild_data_unit_groups_are_indexed = false;
    scan_rebuild_inode_record_count = 0u;
    scan_rebuild_inode_records_are_indexed = false;
    var object_start = 0u;
    while (object_start < scan_rebuild_unit_count) {
        let object_end = scan_rebuild_object_end(object_start);
        let object_base = scan_rebuild_base_at(object_start);
        if (!scan_rebuild_is_left_for_the_lineage(object_base)) {
            object_start = object_end;
            continue;
        }
        let unit_class = rd8(object_base, 6u);
        let birth_txg_offset = scan_rebuild_birth_txg_offset(unit_class);
        // 诞生代号在谱系上的最大两版（各版按择新键升序：最后两个在谱系上的就是）
        var current_start = NONE;
        var current_end = NONE;
        var next_older_start = NONE;
        var version_start = object_start;
        while (version_start < object_end) {
            let version_end = scan_rebuild_version_end(version_start);
            if (scan_rebuild_txg_is_on_the_lineage(rd64(scan_rebuild_base_at(version_start), birth_txg_offset))) {
                next_older_start = current_start;
                current_start = version_start;
                current_end = version_end;
            }
            version_start = version_end;
        }
        object_start = object_end;
        if (current_start == NONE) {
            continue;
        }
        let current_base = scan_rebuild_base_at(current_start);
        if (next_older_start != NONE
            && scan_rebuild_order_keys_are_equal(scan_rebuild_order_key_of(scan_rebuild_base_at(next_older_start)), scan_rebuild_order_key_of(current_base))) {
            return select(SCAN_REBUILD_OUTCOME_PACKED_RECORD_VERSIONS_TIED_ON_THE_TOTAL_ORDER_KEY,
                SCAN_REBUILD_OUTCOME_DATA_UNIT_VERSIONS_TIED_ON_THE_TOTAL_ORDER_KEY, unit_class == 1u);
        }
        if (unit_class == 1u) {
            let inode = rd64(current_base, 51u);
            let object_birth = rd64(current_base, 59u);
            let group_key = vec4<u32>(inode.x, inode.y, object_birth.x, object_birth.y);
            var group = keyed_first_row_of(scan_rebuild_data_unit_group_table(), scan_rebuild_data_unit_group_count,
                scan_rebuild_data_unit_groups_are_indexed, group_key);
            if (group == NONE) {
                // 一个对象身份至多一组，对象身份不多于扫描项：表装得下
                group = scan_rebuild_data_unit_group_count;
                let group_row = SCAN_REBUILD_DATA_UNIT_GROUPS + group * SCAN_REBUILD_DATA_UNIT_GROUP_WORDS;
                sws(group_row, group_key.x);
                sws(group_row + 1u, group_key.y);
                sws(group_row + 2u, group_key.z);
                sws(group_row + 3u, group_key.w);
                scan_rebuild_data_unit_group_count = group + 1u;
                scan_rebuild_data_unit_groups_are_indexed = keyed_table_after_a_row_was_appended(scan_rebuild_data_unit_group_table(),
                    scan_rebuild_data_unit_group_count, scan_rebuild_data_unit_groups_are_indexed, group);
            }
            let offset_in_bytes = rd64(current_base, 67u);
            let birth_txg = rd64(current_base, 75u);
            let row = SCAN_REBUILD_DATA_UNITS + scan_rebuild_data_unit_count * SCAN_REBUILD_DATA_UNIT_WORDS;
            sws(row, group);
            sws(row + 1u, offset_in_bytes.x);
            sws(row + 2u, offset_in_bytes.y);
            sws(row + 3u, 0u);
            sws(row + 4u, inode.x);
            sws(row + 5u, inode.y);
            sws(row + 6u, object_birth.x);
            sws(row + 7u, object_birth.y);
            sws(row + 8u, birth_txg.x);
            sws(row + 9u, birth_txg.y);
            sws(row + 10u, current_start);
            sws(row + 11u, current_end - current_start);
            sws(row + 12u, 0u);
            sws(row + 13u, 0u);
            sws(row + 14u, 0u);
            scan_rebuild_data_unit_count = scan_rebuild_data_unit_count + 1u;
            continue;
        }
        let retirement = scan_rebuild_retirement_of_the_leaf_container(current_base);
        if (retirement.is_retired) {
            if (lt64(retirement.earliest_death_txg, rd64(current_base, birth_txg_offset))) {
                return SCAN_REBUILD_OUTCOME_RETIRED_LEAF_CONTAINER_HAS_A_VERSION_BORN_AFTER_ITS_DEATH;
            }
            continue;
        }
        scan_rebuild_put_into_the_rebuilt_mapping(scan_rebuild_mapping_key_of_the_header(current_base), current_start, current_end - current_start);
        let container = scan_rebuild_decoded_copy_of_a_container_version(current_start, current_end, DELETION_PACKED_TYPE_INODE,
            DELETION_INODE_RECORD_BYTES);
        if (container == NONE) {
            return SCAN_REBUILD_OUTCOME_CURRENT_PACKED_CONTAINER_DECODES_FROM_NO_COPY;
        }
        for (var record_index = 0u; record_index < rd16(container, 69u); record_index = record_index + 1u) {
            let record = SCAN_REBUILD_PACKED_RECORDS_START + record_index * DELETION_INODE_RECORD_BYTES;
            let inode = rd64(container, record);
            let key = vec4<u32>(inode.x, inode.y, 0u, 0u);
            if (keyed_first_row_of(scan_rebuild_inode_record_table(), scan_rebuild_inode_record_count, scan_rebuild_inode_records_are_indexed,
                key) != NONE) {
                return SCAN_REBUILD_OUTCOME_INODE_NUMBER_IN_TWO_CURRENT_CONTAINERS;
            }
            if (scan_rebuild_inode_record_count >= SCAN_REBUILD_MAX_INODE_RECORDS) {
                undecidable = SCAN_REBUILD_SITE_INODE_RECORDS_PAST_THE_TABLE;
                return SCAN_REBUILD_OUTCOME_NOT_REACHED;
            }
            let row_index = scan_rebuild_inode_record_count;
            let row = SCAN_REBUILD_INODE_RECORDS + row_index * SCAN_REBUILD_INODE_RECORD_WORDS;
            sws(row, inode.x);
            sws(row + 1u, inode.y);
            sws(row + 2u, container);
            sws(row + 3u, record);
            scan_rebuild_inode_record_count = row_index + 1u;
            scan_rebuild_inode_records_are_indexed = keyed_table_after_a_row_was_appended(scan_rebuild_inode_record_table(),
                scan_rebuild_inode_record_count, scan_rebuild_inode_records_are_indexed, row_index);
        }
    }
    return SCAN_REBUILD_OUTCOME_HOLDS;
}

// 读法 Z 加写序第三条要的那张表（rebuild.rs newest_birth_txg_below_each_data_unit）：⑥ 按（组、偏移）排一次序，同一组里按偏移升序走，
// 每个单元记下偏移严格更小的那些单元里最大的出生 txg（同一个偏移上有几版的先都记下、再并进「之下」那一份）
fn scan_rebuild_note_the_newest_birth_txg_below_each_data_unit() {
    let count = scan_rebuild_data_unit_count;
    order_rows_by_their_first_four_words(SCAN_REBUILD_DATA_UNITS, SCAN_REBUILD_DATA_UNIT_WORDS, count, SCAN_REBUILD_DATA_UNIT_ORDER);
    var position = 0u;
    var has_newest_so_far = false;
    var newest_so_far = vec2<u32>(0u, 0u);
    // 迭代上界：单元数（每一轮至少走过一个单元）；跨轮带的是这一组里此前各偏移的最大出生 txg，换组时清掉
    while (position < count) {
        let first_row = SCAN_REBUILD_DATA_UNITS + sw(SCAN_REBUILD_DATA_UNIT_ORDER + position) * SCAN_REBUILD_DATA_UNIT_WORDS;
        let group = sw(first_row);
        let offset_in_bytes = vec2<u32>(sw(first_row + 1u), sw(first_row + 2u));
        if (position == 0u || sw(SCAN_REBUILD_DATA_UNITS + sw(SCAN_REBUILD_DATA_UNIT_ORDER + position - 1u) * SCAN_REBUILD_DATA_UNIT_WORDS) != group) {
            has_newest_so_far = false;
        }
        var end = position;
        var newest_at_the_offset = vec2<u32>(0u, 0u);
        while (end < count) {
            let row = SCAN_REBUILD_DATA_UNITS + sw(SCAN_REBUILD_DATA_UNIT_ORDER + end) * SCAN_REBUILD_DATA_UNIT_WORDS;
            if (sw(row) != group || !eq64(vec2<u32>(sw(row + 1u), sw(row + 2u)), offset_in_bytes)) {
                break;
            }
            let birth_txg = vec2<u32>(sw(row + 8u), sw(row + 9u));
            if (lt64(newest_at_the_offset, birth_txg)) {
                newest_at_the_offset = birth_txg;
            }
            sws(row + 12u, newest_so_far.x);
            sws(row + 13u, newest_so_far.y);
            sws(row + 14u, select(0u, 1u, has_newest_so_far));
            end = end + 1u;
        }
        if (!has_newest_so_far || lt64(newest_so_far, newest_at_the_offset)) {
            newest_so_far = newest_at_the_offset;
        }
        has_newest_so_far = true;
        position = end;
    }
}

// 数据单元（rebuild.rs rebuild_index_tables 最后一个循环，按对象身份的次序）：墓碑压着的不认领；有同一对象出生代的记录时
// 读法 Z 四条（size 之内、偏移更小的单元里没有出生更晚的、截断字段之外不早于 T）都满足才认领；认领的进每个 inode 的数据单元集合（同一个位置两个交回不适用）
// 与中央映射
fn scan_rebuild_outcome_of_the_data_units() -> u32 {
    scan_rebuild_position_count = 0u;
    scan_rebuild_positions_are_indexed = false;
    scan_rebuild_note_the_newest_birth_txg_below_each_data_unit();
    for (var data_unit_index = 0u; data_unit_index < scan_rebuild_data_unit_count; data_unit_index = data_unit_index + 1u) {
        let row = SCAN_REBUILD_DATA_UNITS + data_unit_index * SCAN_REBUILD_DATA_UNIT_WORDS;
        let offset_in_bytes = vec2<u32>(sw(row + 1u), sw(row + 2u));
        let inode = vec2<u32>(sw(row + 4u), sw(row + 5u));
        let object_birth = vec2<u32>(sw(row + 6u), sw(row + 7u));
        let birth_txg = vec2<u32>(sw(row + 8u), sw(row + 9u));
        let record_index = keyed_first_row_of(scan_rebuild_inode_record_table(), scan_rebuild_inode_record_count,
            scan_rebuild_inode_records_are_indexed, vec4<u32>(inode.x, inode.y, 0u, 0u));
        var record_base = 0u;
        var record = 0u;
        var has_the_record_of_this_object = false;
        if (record_index != NONE) {
            let record_row = SCAN_REBUILD_INODE_RECORDS + record_index * SCAN_REBUILD_INODE_RECORD_WORDS;
            record_base = sw(record_row + 2u);
            record = sw(record_row + 3u);
            has_the_record_of_this_object = eq64(rd64(record_base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET), object_birth);
        }
        // 墓碑压着的不认领：删完的、删除中的与墓碑压着而不算删除中的（记录还在）都是（删除中的对象两侧都不比，判决 m3-multi-file-unlink-r1
        // 第三节 M5，比对之前由 scan_rebuild_leave_the_objects_being_deleted_out_of_the_walked_tables 从遍历方向那一侧去掉；不算删除中的两侧照比，
        // 判决 m4-c860-r1 第三节 C2）
        if (keyed_first_row_of(scan_rebuild_tombstoned_object_table(), scan_rebuild_tombstoned_object_count,
            scan_rebuild_tombstoned_objects_are_indexed, vec4<u32>(inode.x, inode.y, object_birth.x, object_birth.y)) != NONE) {
            continue;
        }
        if (has_the_record_of_this_object) {
            if (scan_rebuild_offset_lies_beyond_the_data_units_the_size_asks_for(offset_in_bytes, rd64(record_base, record + INODE_RECORD_SIZE_OFFSET))) {
                continue;
            }
            if (sw(row + 14u) != 0u && lt64(birth_txg, vec2<u32>(sw(row + 12u), sw(row + 13u)))) {
                continue;
            }
            let truncation_field_txg = rd64(record_base, record + INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET);
            if (!is_zero64(truncation_field_txg)
                && scan_rebuild_offset_lies_beyond_the_data_units_the_size_asks_for(offset_in_bytes,
                    rd64(record_base, record + INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET))
                && lt64(birth_txg, truncation_field_txg)) {
                continue;
            }
        }
        let version_start = sw(row + 10u);
        let key = scan_rebuild_mapping_key_of_the_header(scan_rebuild_base_at(version_start));
        let position = vec4<u32>(inode.x, inode.y, offset_in_bytes.x, offset_in_bytes.y);
        if (keyed_first_row_of(scan_rebuild_position_table(), scan_rebuild_position_count, scan_rebuild_positions_are_indexed, position) != NONE) {
            return SCAN_REBUILD_OUTCOME_TWO_OBJECTS_CLAIM_ONE_DATA_UNIT_POSITION;
        }
        // 一个当前版本至多一行，当前版本不多于扫描项：表装得下
        let position_index = scan_rebuild_position_count;
        let position_row = SCAN_REBUILD_POSITIONS + position_index * SCAN_REBUILD_POSITION_WORDS;
        sws(position_row, position.x);
        sws(position_row + 1u, position.y);
        sws(position_row + 2u, position.z);
        sws(position_row + 3u, position.w);
        put_the_mapping_key_into_the_words(position_row + 4u, key);
        scan_rebuild_position_count = position_index + 1u;
        scan_rebuild_positions_are_indexed = keyed_table_after_a_row_was_appended(scan_rebuild_position_table(), scan_rebuild_position_count,
            scan_rebuild_positions_are_indexed, position_index);
        scan_rebuild_put_into_the_rebuilt_mapping(key, version_start, sw(row + 11u));
    }
    return SCAN_REBUILD_OUTCOME_HOLDS;
}

// ───────────────────────── 比对（walk.rs judge_scan_rebuild_agrees_with_the_walked_index 的三张表）─────────────────────────

// 中央映射同一个 key 两边都有时比落点（walk.rs differences_in_the_central_mapping 的那一支）：遍历方向那两条位置（walked_row 是最新根中央映射那一版的表的行）
// 都是扫描重建认领的这一版的副本就对得上；不是时两边各只留读得出的再比——扫描重建那几份按头里的载荷校验和读回（切片的检查字位 1：按自述的类载荷 CRC 过，
// 落点有 32 KiB），遍历方向那两份按位置条目里的整单元校验和读回。两条位置与这一版的副本在一个循环里读（一个 read_at 调用点）
fn scan_rebuild_placements_agree(walked_row: u32, version_start: u32, member_count: u32) -> bool {
    let walked = C_VERSION_MAPPING_ROWS + walked_row * C_VERSION_MAPPING_ROW_WORDS;
    var walked_location_is_a_rebuilt_copy = array<bool, 2>(false, false);
    for (var position = version_start; position < version_start + member_count; position = position + 1u) {
        let row = SCAN_REBUILD_UNITS + sw(SCAN_REBUILD_UNIT_ORDER + position) * SCAN_REBUILD_UNIT_WORDS;
        for (var location = 0u; location < 2u; location = location + 1u) {
            let location_row = walked + 8u + location * 4u;
            if (sw(row + 1u) == sw(location_row) && sw(row + 2u) == sw(location_row + 1u) && sw(row + 3u) == sw(location_row + 2u)) {
                walked_location_is_a_rebuilt_copy[location] = true;
            }
        }
    }
    if (walked_location_is_a_rebuilt_copy[0] && walked_location_is_a_rebuilt_copy[1]) {
        return true;
    }
    var walked_location_is_readable = array<bool, 2>(false, false);
    var walked_location_is_a_readable_rebuilt_copy = array<bool, 2>(false, false);
    for (var read_index = 0u; read_index < 2u + member_count; read_index = read_index + 1u) {
        let is_a_walked_location = read_index < 2u;
        var device = 0u;
        var slot = vec2<u32>(0u, 0u);
        if (is_a_walked_location) {
            let location_row = walked + 8u + read_index * 4u;
            device = sw(location_row);
            slot = vec2<u32>(sw(location_row + 1u), sw(location_row + 2u));
        } else {
            let row = SCAN_REBUILD_UNITS + sw(SCAN_REBUILD_UNIT_ORDER + version_start + read_index - 2u) * SCAN_REBUILD_UNIT_WORDS;
            device = sw(row + 1u);
            slot = vec2<u32>(sw(row + 2u), sw(row + 3u));
        }
        let s = read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES);
        if (is_a_walked_location) {
            walked_location_is_readable[read_index] = s.version != NONE
                && slice_crc_matches(s, DATA_UNIT_BYTES, sw(walked + 8u + read_index * 4u + 3u));
            continue;
        }
        if (s.version == NONE || (s.c2 & 6u) != 6u) {
            continue;
        }
        // 扫描重建这一份读得出：要等于遍历方向某一条读得出的位置
        var matches_a_readable_walked_location = false;
        for (var location = 0u; location < 2u; location = location + 1u) {
            let location_row = walked + 8u + location * 4u;
            if (device == sw(location_row) && eq64(slot, vec2<u32>(sw(location_row + 1u), sw(location_row + 2u)))) {
                walked_location_is_a_readable_rebuilt_copy[location] = true;
                if (walked_location_is_readable[location]) {
                    matches_a_readable_walked_location = true;
                }
            }
        }
        if (!matches_a_readable_walked_location) {
            return false;
        }
    }
    for (var location = 0u; location < 2u; location = location + 1u) {
        if (walked_location_is_readable[location] && !walked_location_is_a_readable_rebuilt_copy[location]) {
            return false;
        }
    }
    return true;
}

// 扫描重建那一侧当前版本 inode 叶容器里的这条记录（record_base 起 record 处）是不是删除中的对象的（walk.rs inodes_being_deleted_on_both_sides，
// 判决 `research/prompts/m4-c860-r1-main-verification.md` 第三节 C2，用户 2026-10-20 选丙）：它的 (号, 对象出生代) 被谱系上的墓碑判死
// （rebuild.rs 的 records_of_the_tombstoned_objects：墓碑压着、记录还在），遍历方向也有同号、同对象出生代的记录，且两侧那条记录 nlink 都是
// NLINK_OF_AN_INODE_BEING_DELETED（D8（核心索引结构） 已定项 16 第 1、3 条）。不是的照比：墓碑压着而不算删除中的，记录两侧照比，数据单元
// 扫描重建那一侧按墓碑不认领、遍历方向照收
fn scan_rebuild_record_is_of_an_object_being_deleted(record_base: u32, record: u32) -> bool {
    let inode = rd64(record_base, record);
    let object_birth = rd64(record_base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET);
    if (keyed_first_row_of(scan_rebuild_tombstoned_object_table(), scan_rebuild_tombstoned_object_count,
        scan_rebuild_tombstoned_objects_are_indexed, vec4<u32>(inode.x, inode.y, object_birth.x, object_birth.y)) == NONE) {
        return false;
    }
    let walked_row = scan_rebuild_walked_inode_record_row_of(inode);
    if (walked_row == NONE) {
        return false;
    }
    let walked_record_base = sw(walked_row + 2u);
    let walked_record = sw(walked_row + 3u);
    if (!eq64(rd64(walked_record_base, walked_record + INODE_RECORD_OBJECT_BIRTH_OFFSET), object_birth)) {
        return false;
    }
    return rd32(record_base, record + INODE_RECORD_NLINK_OFFSET) == NLINK_OF_AN_INODE_BEING_DELETED
        && rd32(walked_record_base, walked_record + INODE_RECORD_NLINK_OFFSET) == NLINK_OF_AN_INODE_BEING_DELETED;
}

// 删除中的对象只对它自己两侧都不比（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5；walk.rs
// judge_scan_rebuild_agrees_with_the_walked_index 读完两侧之后那一段）：删除中由两侧一起认（scan_rebuild_record_is_of_an_object_being_deleted），
// 算的，遍历方向去掉它的记录（记成比过）、它在每个 inode 的数据单元集合里的每一项（记成比过）与那几项的映射条目（摘掉「在遍历方向那张表里」的记号）。
// 不算的两侧照比；别的对象照比。迭代上界：扫描重建那一侧的记录数乘遍历方向的数据单元数（删除中的对象至多几个）
fn scan_rebuild_leave_the_objects_being_deleted_out_of_the_walked_tables() {
    for (var row_index = 0u; row_index < scan_rebuild_inode_record_count; row_index = row_index + 1u) {
        let row = SCAN_REBUILD_INODE_RECORDS + row_index * SCAN_REBUILD_INODE_RECORD_WORDS;
        let record_base = sw(row + 2u);
        let record = sw(row + 3u);
        if (!scan_rebuild_record_is_of_an_object_being_deleted(record_base, record)) {
            continue;
        }
        let inode = vec2<u32>(sw(row), sw(row + 1u));
        // 认删除中那一判已核过遍历方向有这一行；这里再核一次只为不往 NONE 那一行写
        let walked_row = scan_rebuild_walked_inode_record_row_of(inode);
        if (walked_row == NONE) {
            continue;
        }
        sws(walked_row + 5u, 1u);
        for (var walked_index = 0u; walked_index < scan_rebuild_walked_position_count; walked_index = walked_index + 1u) {
            let walked_position = SCAN_REBUILD_WALKED_POSITIONS + walked_index * SCAN_REBUILD_WALKED_POSITION_WORDS;
            if (!eq64(vec2<u32>(sw(walked_position), sw(walked_position + 1u)), inode)) {
                continue;
            }
            sws(walked_position + 11u, 1u);
            let flags_word = scan_rebuild_mapping_row_flags_word_of(mapping_key_of_the_words(walked_position + 4u));
            if (flags_word != NONE) {
                sws(flags_word, sw(flags_word) & ~SCAN_REBUILD_MAPPING_ROW_IN_THE_WALKED_TABLE);
            }
        }
    }
}

// 三张表逐项相同（walk.rs judge_scan_rebuild_agrees_with_the_walked_index 判一次：三张都没有差异才成立）。删除中的对象两侧都已去掉
// （扫描重建那一侧的记录在比记录时跳过，遍历方向那一侧由 scan_rebuild_leave_the_objects_being_deleted_out_of_the_walked_tables 去掉）
fn scan_rebuild_every_table_agrees() -> bool {
    scan_rebuild_leave_the_objects_being_deleted_out_of_the_walked_tables();
    // 中央映射：扫描重建的每一条在遍历方向那一张里、落点对得上；遍历方向那一张里每一条都被比过
    for (var row_index = 0u; row_index < scan_rebuild_mapping_count; row_index = row_index + 1u) {
        let row = SCAN_REBUILD_MAPPING + row_index * SCAN_REBUILD_MAPPING_WORDS;
        let walked_row = version_mapping_first_row_of(mapping_key_of_the_words(row));
        if (walked_row == NONE) {
            return false;
        }
        let flags = sw(SCAN_REBUILD_MAPPING_ROW_FLAGS + walked_row);
        if ((flags & SCAN_REBUILD_MAPPING_ROW_IN_THE_WALKED_TABLE) == 0u) {
            return false;
        }
        sws(SCAN_REBUILD_MAPPING_ROW_FLAGS + walked_row, flags | SCAN_REBUILD_MAPPING_ROW_COMPARED);
        if (!scan_rebuild_placements_agree(walked_row, sw(row + 7u), sw(row + 8u))) {
            return false;
        }
    }
    for (var row_index = 0u; row_index < version_mapping_count; row_index = row_index + 1u) {
        let flags = sw(SCAN_REBUILD_MAPPING_ROW_FLAGS + row_index);
        if ((flags & SCAN_REBUILD_MAPPING_ROW_IN_THE_WALKED_TABLE) != 0u && (flags & SCAN_REBUILD_MAPPING_ROW_COMPARED) == 0u) {
            return false;
        }
    }
    // 每个 inode 的数据单元集合：值比映射 key（文件头「读法」：落点归中央映射那张表）
    for (var row_index = 0u; row_index < scan_rebuild_position_count; row_index = row_index + 1u) {
        let row = SCAN_REBUILD_POSITIONS + row_index * SCAN_REBUILD_POSITION_WORDS;
        let walked_index = keyed_first_row_of(scan_rebuild_walked_position_table(), scan_rebuild_walked_position_count,
            scan_rebuild_walked_positions_are_indexed, vec4<u32>(sw(row), sw(row + 1u), sw(row + 2u), sw(row + 3u)));
        if (walked_index == NONE) {
            return false;
        }
        let walked_row = SCAN_REBUILD_WALKED_POSITIONS + walked_index * SCAN_REBUILD_WALKED_POSITION_WORDS;
        if (!same_mapping_key(mapping_key_of_the_words(row + 4u), mapping_key_of_the_words(walked_row + 4u))) {
            return false;
        }
        sws(walked_row + 11u, 1u);
    }
    for (var walked_index = 0u; walked_index < scan_rebuild_walked_position_count; walked_index = walked_index + 1u) {
        if (sw(SCAN_REBUILD_WALKED_POSITIONS + walked_index * SCAN_REBUILD_WALKED_POSITION_WORDS + 11u) == 0u) {
            return false;
        }
    }
    // inode 记录集合：记录原样（宽与字节）相同。跳过的只有两侧一起认的删除中对象；墓碑压着而不算删除中的那条记录照比
    // （walk.rs 把它放回扫描重建那一侧的 inode 记录集合）
    for (var row_index = 0u; row_index < scan_rebuild_inode_record_count; row_index = row_index + 1u) {
        let row = SCAN_REBUILD_INODE_RECORDS + row_index * SCAN_REBUILD_INODE_RECORD_WORDS;
        if (scan_rebuild_record_is_of_an_object_being_deleted(sw(row + 2u), sw(row + 3u))) {
            continue;
        }
        let walked_row = scan_rebuild_walked_inode_record_row_of(vec2<u32>(sw(row), sw(row + 1u)));
        if (walked_row == NONE || sw(walked_row + 4u) != DELETION_INODE_RECORD_BYTES
            || !bytes_equal(sw(row + 2u), sw(row + 3u), sw(walked_row + 2u), sw(walked_row + 3u), DELETION_INODE_RECORD_BYTES)) {
            return false;
        }
        sws(walked_row + 5u, 1u);
    }
    for (var walked_index = 0u; walked_index < scan_rebuild_walked_inode_record_count; walked_index = walked_index + 1u) {
        if (sw(SCAN_REBUILD_WALKED_INODE_RECORDS + walked_index * SCAN_REBUILD_WALKED_INODE_RECORD_WORDS + 5u) == 0u) {
            return false;
        }
    }
    return true;
}

// 这一条不变量在违例位里置了没有（走读、扫描、引用与快照三个入口写进结论字的那几个字）
fn scan_rebuild_is_violated(invariant: u32) -> bool {
    return (violated[invariant / 32u] & (1u << (invariant % 32u))) != 0u;
}

// I-1.13（walk.rs judge_scan_rebuild_agrees_with_the_walked_index）：最新根的实例表沿链读不全、遍历方向的索引已被别的不变量判损坏、遍历方向那三张表读不全、
// 扫描重建停在条款没写之处都报不适用（不判、交回那一种理由的结局码）；否则重建三张表逐项比，判一次
fn scan_rebuild_outcome_of_the_state() -> u32 {
    if (sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) != 1u) {
        return SCAN_REBUILD_OUTCOME_INSTANCE_TABLE_NOT_READ_IN_FULL;
    }
    // walk.rs INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED 的次序，取第一条违例了的
    if (scan_rebuild_is_violated(INVARIANT_I_1_11)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_1_11;
    }
    if (scan_rebuild_is_violated(INVARIANT_I_1_12)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_1_12;
    }
    if (scan_rebuild_is_violated(INVARIANT_I_4_2)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_4_2;
    }
    if (scan_rebuild_is_violated(INVARIANT_I_5_1)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_5_1;
    }
    if (scan_rebuild_is_violated(INVARIANT_I_9_34)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_34;
    }
    if (scan_rebuild_is_violated(INVARIANT_I_9_32)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_32;
    }
    if (scan_rebuild_is_violated(INVARIANT_I_9_39)) {
        return SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_39;
    }
    let walked = scan_rebuild_read_the_walked_tables(checker_root_of(c_newest_index).base);
    if (walked != SCAN_REBUILD_OUTCOME_HOLDS) {
        return walked;
    }
    if (!scan_rebuild_collect_the_published_units(vec2<u32>(sw(H_BASE + H_FSID0), sw(H_BASE + H_FSID1)))) {
        return SCAN_REBUILD_OUTCOME_NOT_REACHED;
    }
    scan_rebuild_order_the_units();
    scan_rebuild_mapping_count = 0u;
    scan_rebuild_mapping_is_indexed = false;
    let chosen = scan_rebuild_outcome_of_choosing_among_every_published_version();
    if (chosen != SCAN_REBUILD_OUTCOME_HOLDS) {
        return chosen;
    }
    let containers = scan_rebuild_outcome_of_the_containers_chosen_among_every_published_version();
    if (containers != SCAN_REBUILD_OUTCOME_HOLDS || undecidable != 0u) {
        return containers;
    }
    if (!scan_rebuild_lineage_records_form_a_parent_chain()) {
        return SCAN_REBUILD_OUTCOME_LINEAGE_RECORDS_FORM_NO_PARENT_CHAIN;
    }
    let tombstones = scan_rebuild_outcome_of_the_tombstones();
    if (tombstones != SCAN_REBUILD_OUTCOME_HOLDS || undecidable != 0u) {
        return tombstones;
    }
    let versions_left_for_the_lineage = scan_rebuild_outcome_of_the_versions_left_for_the_lineage();
    if (versions_left_for_the_lineage != SCAN_REBUILD_OUTCOME_HOLDS || undecidable != 0u) {
        return versions_left_for_the_lineage;
    }
    let data_units = scan_rebuild_outcome_of_the_data_units();
    if (data_units != SCAN_REBUILD_OUTCOME_HOLDS || undecidable != 0u) {
        return data_units;
    }
    let every_table_agrees = scan_rebuild_every_table_agrees();
    if (undecidable != 0u) {
        return SCAN_REBUILD_OUTCOME_NOT_REACHED;
    }
    judge(INVARIANT_I_1_13, every_table_agrees);
    return select(SCAN_REBUILD_OUTCOME_VIOLATED, SCAN_REBUILD_OUTCOME_HOLDS, every_table_agrees);
}

// ───────────────────────── 入口 ─────────────────────────

// 扫描重建的入口（与扫描内核同一份源码、各编一条管线：扫描主入口离展开预算只剩四万多个表达式，装不下这一判）。走读内核判完、交接段有效时，
// 接着另三个池级 checker 入口的违例位与判到位记，判不了的原因照扫描内核那一套只在还没记过时写；结局写进 SCAN_REBUILD_OUTCOME_WORD。
// 交接段无效时不写那一字，留着走读内核写的：池级 checker 走到扫描重建之前整池交回的四种（码 27 到 30）是走读内核在那几处早退记的码，
// 走读内核判不了、没判完的状态是 SCAN_REBUILD_OUTCOME_NOT_REACHED（池级 checker 在那种状态上没走到 I-1.13）
@compute @workgroup_size(64)
fn scan_rebuild_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    let out = local * VERDICT_WORDS;
    if (sw(H_BASE + H_VALID) == 0u) {
        return;
    }
    load_violated(out);
    load_judged();
    handoff_load();
    let outcome = scan_rebuild_outcome_of_the_state();
    store_violated(out);
    store_judged(out);
    verdicts[out + SCAN_REBUILD_OUTCOME_WORD] = outcome;
    if (undecidable != 0u && verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] == 0u) {
        verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] = undecidable;
    }
}
