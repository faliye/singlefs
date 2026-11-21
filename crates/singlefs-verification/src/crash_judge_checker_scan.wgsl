// GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
// 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
// I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6、I-9.24，回退候选集每一版）、
// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单；退役记录 I-9.32–I-9.35、
// 谱系与退役容器 I-9.36 / I-9.37 也在这一遍）与 I-9.6 墓碑那一半
// （所选根的墓碑树），再加里程碑四自建 GC 的 I-3.16（记账那一判之后），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。
// 第二个入口（引用与快照那一截）判 I-3.9、I-9.14、只读快照那几条，再是自建 GC 的 I-4.4、I-5.6、I-3.15、I-1.14。
// 只读不判的读法照 walk.rs 的 *_without_judging 各写一份。


// 扫描内核沿实例表链最多读几片：与走读内核 crash_judge_checker.wgsl 的 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 同值（两个内核各自编译，
// 共用段 crash_judge_checker_common.wgsl 里没有它，各写一份；对拍用例 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs 钉两份相同）。
// walk.rs 读链不设片数上限：走满这么多片、链还没完，记判不了交 CPU（判决 `research/prompts/m3-batch27-code-r1-main-verification.md`
// 第二节 K1 J1 那一行 O-1、第三节第 2 条：不当读不出跳过 I-7.9 / I-3.12）。今天走读内核先把根环里每条自证过的根的链按同一个上限走一遍、
// 走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处落在候选根的链途经走读内核已走过的片、链长过上限的那一形（走读内核走到已走过的片
// 就停、不记 61；坏镜像才造得出，判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K1 L2 那一行）
const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;

// r_instance_pages_after_first（候选根的引用集合）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞
const UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (37u << 8u);

// r_instance_rows（一条根自己那张实例表的全部行，I-7.9 与 I-3.12 读它）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞
const UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (38u << 8u);

var<private> r_complete: bool;

var<private> r_noting: bool;

var<private> r_candidate: u32;

var<private> r_seen_count: u32;

var<private> c_released_count: u32;

var<private> c_alloc_record_count: u32;

var<private> c_seen_node_count: u32;
// 去重表 C_SEEN_NODES 里第 i 个分配记录树根有没有整棵读到叶（I-5.4 的判到位：一版的分配记录树走到叶才算这一版判过，walk.rs
// judge_allocation_records_disjoint 的 allocation_roots_walked_to_their_leaves）：位 i 在第 i / 32 字
var<private> c_seen_node_walked_to_its_leaves: array<u32, 8>;

var<private> c_examined_count: u32;

var<private> c_exempted: array<u32, 8>;

// I-3.16 占着的槽里钉带出来的那一截（walk.rs released_slots_the_pin_kept_in_defer）在 C_DEFERRED 里接在 F 之下的推迟后面的行数
var<private> c_kept_in_defer_by_the_pin_count: u32;

// c_quarantine_exempted 第二遍（最新根）把树表里种类 3 的每一棵都读全了：I-3.1（RCW）要最新根的全部分配记录，少一棵就判不了
// （walk.rs allocation_records_of_the_newest_root_read_in_full_without_judging）
var<private> c_newest_records_read_in_full: bool;

// c_quarantine_exempted 第二遍（最新根）读的树表非空而一条种类 3 的条目都没有（根指针全零的种类 3 条目也算一条）：I-3.1 的记录口径无从比，
// 每块盘记射程里判不了（walk.rs AllocationRecordsOfTheNewestRootReadInFull::TreeTableWithEntriesButNoAllocationRecordTreeEntry；那块畸形 I-9.16 ④ 判红）
var<private> c_newest_tree_table_has_entries_but_no_allocation_record_tree_entry: bool;

// c_quarantine_exempted 第二遍（最新根）读的树表有几条条目：树表指针全零记 0，树表读不出记 NONE（条目窄过字段表照记条数）。
// I-3.21 拿它分不适用（0 条而根记录不持有分配记录树）与射程里判不了（walk.rs judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root）
var<private> c_newest_tree_table_entry_count: u32;

// c_quarantine_exempted 第二遍（最新根）的树表指针全零：I-3.21 记射程里判不了（walk.rs TreeTablePointerOfTheRootRecord::AllZero；
// core 照它读盘 0 槽 0、读不出，判决 research/prompts/m3-close-b19-reason-r3-main-verification.md 第二节 Q2 第 1 问那一行）
var<private> c_newest_tree_table_pointer_is_all_zero: bool;

// c_quarantine_exempted 第二遍（最新根）树表 0 条时根记录直接持有的那一棵分配记录树（根记录偏移 342，C963 候选一）读成什么样：
// 没读（根记录那一项全零，或树表不是 0 条）、读全了（记录在 C_ALLOC_RECORDS 前 c_newest_records_held_by_the_root_record_count 行）、读不全
const C_TREE_HELD_BY_THE_ROOT_RECORD_NOT_READ: u32 = 0u;
const C_TREE_HELD_BY_THE_ROOT_RECORD_READ_IN_FULL: u32 = 1u;
const C_TREE_HELD_BY_THE_ROOT_RECORD_NOT_READ_IN_FULL: u32 = 2u;

var<private> c_newest_tree_held_by_the_root_record: u32;

// 根记录直接持有的那一棵读全了时它的记录条数（它们在 C_ALLOC_RECORDS 前这么多行；c_alloc_record_count 照 walk.rs 隔离豁免与 I-3.17 的读法记 0）
var<private> c_newest_records_held_by_the_root_record_count: u32;

// 已释放记录那张表（C_RELEASED）按 (盘, 槽) 的桶表，写法同 crash_judge_checker_common.wgsl 的 slot_key_first_bucket 那一段：最新根那一遍收完
// 已释放记录之后建，候选根那几遍里 r_note 按它找。借 crash_judge_checker_common.wgsl ⑮ 的 C_ALLOCATION_RECORD_BUCKETS 那一段（与 I-3.10 的去重桶表
// 先后借，为什么不同时用见那一处）：C_MAX_RELEASED 随分配记录那张表抬到 8192 条时桶表 16384 个字，原来借的 C_JOURNAL_GROUP_KEYS 那 8192 个字装不下
const C_RELEASED_BUCKET_COUNT: u32 = 2u * C_MAX_RELEASED;

const C_RELEASED_BUCKETS: u32 = C_ALLOCATION_RECORD_BUCKETS;

// 借的那几段装得下（解析内核时判，不进展开量）：已释放记录的桶表在 C_ALLOCATION_RECORD_BUCKETS 里，I-1.8 的检查字在 C_JOURNAL_GROUP_KEYS 里、
// 次序在 C_ROW_ORDER 里
const_assert C_RELEASED_BUCKET_COUNT <= C_ALLOCATION_RECORD_BUCKET_WORDS;

const_assert C_MAX_SCANNED <= C_JOURNAL_GROUP_KEYS_END - C_JOURNAL_GROUP_KEYS;

const_assert C_MAX_SCANNED <= C_ROW_ORDER_END - C_ROW_ORDER;

// 引用集合这一遍「记一条指针」：只回答「这条指针指着哪条已释放记录的落点」，位记在 (记录, 候选根) 上。原来每记一条指针都把已释放记录从头比到尾，
// 改成两条位置条目各按桶表找一遍：同 (盘, 槽) 的几条记录都在从首桶起到第一个空桶那一段里，一条记录两条位置条目都对上时同一位置两次，与原来一样
fn r_note(p: Ptr) {
    if (!r_noting) {
        return;
    }
    for (var copy = 0u; copy < 2u; copy = copy + 1u) {
        let device = select(p.device0, p.device1, copy == 1u);
        let slot = select(p.slot0, p.slot1, copy == 1u);
        var bucket = slot_key_first_bucket(device, slot, C_RELEASED_BUCKET_COUNT);
        for (var probe = 0u; probe < C_RELEASED_BUCKET_COUNT; probe = probe + 1u) {
            let released_index_plus_one = sw(C_RELEASED_BUCKETS + bucket);
            if (released_index_plus_one == 0u) {
                break;
            }
            let released = C_RELEASED + (released_index_plus_one - 1u) * 5u;
            if (sw(released) == device && sw(released + 1u) == slot.x && sw(released + 2u) == slot.y) {
                let word = C_REFERENCED_BY + (released_index_plus_one - 1u) * 2u + (r_candidate >> 5u);
                sws(word, sw(word) | (1u << (r_candidate & 31u)));
            }
            bucket = (bucket + 1u) & (C_RELEASED_BUCKET_COUNT - 1u);
        }
    }
}

// 豁免三类之外的一条指针（walk.rs RootReferences::note_located_by_the_central_mapping）：按这条根自己的中央映射认位置再记（判决
// `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3「引用集合记映射位置」）；这一遍不记的时候（引用集合那一遍之外借 r_extent_tree、
// r_allocation_tree 的几处）不查映射
fn r_note_located(p: Ptr, mapping_class: u32) {
    if (!r_noting) {
        return;
    }
    r_note(located_by_the_central_mapping(p, mapping_class));
}

// I-5.5 那一遍（judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark）开着时，r_extent_tree 读到的每条数据指针
// 按盘表下标记下每块盘上最小的落点槽号（walk.rs 同名函数里的 smallest_data_unit_slot_on_each_device）；不在盘表里的盘不记
var<private> r_collecting_data_unit_slots: bool;

var<private> r_has_data_unit_on_device: array<u32, 8>;

var<private> r_smallest_data_unit_slot_on_device: array<vec2<u32>, 8>;

fn r_note_data_unit_slot(device: u32, slot: vec2<u32>) {
    let device_count = hdr(3u);
    for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
        if (hdr(12u + device_index) != device) {
            continue;
        }
        if (r_has_data_unit_on_device[device_index] == 0u || lt64(slot, r_smallest_data_unit_slot_on_device[device_index])) {
            r_smallest_data_unit_slot_on_device[device_index] = slot;
            r_has_data_unit_on_device[device_index] = 1u;
        }
    }
}

// extent 树里的一条数据指针：照 r_note_located 记（码 1）；I-5.5 那一遍开着时再记两条位置条目的落点（按这一版中央映射认过的，全零指针不记，同 walk.rs 过滤 all_zero）；
// 引用与快照那一截收单元时记它的映射 key
fn r_note_data_pointer(p: Ptr) {
    r_note_located(p, MAPPING_CLASS_DATA_UNIT);
    if (r_unit_keys_go_to != 0u && !p.all_zero) {
        s_note_a_unit_key(mapping_key_of_a_pointer(p, MAPPING_CLASS_DATA_UNIT), r_unit_keys_go_to);
    }
    if (!r_collecting_data_unit_slots || p.all_zero) {
        return;
    }
    // 落点按装着的那一版中央映射认过的位置取（D19（块指针的结构与宽度预算） 已定项 5：位置条目只是提示；walk.rs 同名那一遍里 central_mapping.locate）
    let located = located_by_the_central_mapping(p, MAPPING_CLASS_DATA_UNIT);
    r_note_data_unit_slot(located.device0, located.slot0);
    r_note_data_unit_slot(located.device1, located.slot1);
}

// 删除态扫描那一遍开着时，r_extent_tree 读到的每个号的上段叶条目声明与每条非全零数据指针（walk.rs data_units_of_each_inode_without_judging：
// 上段叶内联的那一个属于条目 key 里的号、偏移 0，下段叶的每条记录属于引它那条上段条目的号、偏移取记录 key 的第三段；全零的数据指针不指单元，不记；
// 标签 0 的号也记，单元是空的）：号记进 D_DATA_UNITS_OF_EACH_INODE（单元数、声明、单元链表头），单元记进 D_DATA_UNITS（偏移、数据指针在打包表里的位置、
// 同号的下一个单元）。两张表借引用与快照那一截的 G_KEY_GROUPS 起那一段（见那两个常量）
var<private> r_counting_data_units_of_each_inode: bool;

var<private> d_data_unit_inode_count: u32;

var<private> d_data_unit_count: u32;

// D_DATA_UNITS_OF_EACH_INODE 里号 inode 那一行；没有交回 NONE
fn d_data_units_row_of_inode(inode: vec2<u32>) -> u32 {
    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
        let row = D_DATA_UNITS_OF_EACH_INODE + i * D_DATA_UNITS_OF_AN_INODE_WORDS;
        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
            return row;
        }
    }
    return NONE;
}

// 同上，没有就记一行（单元数 0、没有上段叶条目、没有单元）；表满记判不了、交回 NONE
fn d_data_units_row_of_inode_noting_it(inode: vec2<u32>) -> u32 {
    let existing = d_data_units_row_of_inode(inode);
    if (existing != NONE) {
        return existing;
    }
    if (d_data_unit_inode_count >= D_MAX_DATA_UNIT_INODES) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (48u << 8u);
        return NONE;
    }
    let row = D_DATA_UNITS_OF_EACH_INODE + d_data_unit_inode_count * D_DATA_UNITS_OF_AN_INODE_WORDS;
    sws(row, inode.x);
    sws(row + 1u, inode.y);
    sws(row + 2u, 0u);
    sws(row + 3u, D_DECLARED_BY_NO_UPPER_LEAF_ENTRY);
    sws(row + 4u, 0u);
    d_data_unit_inode_count = d_data_unit_inode_count + 1u;
    return row;
}

// 一条上段叶条目对号 inode 的声明（walk.rs DataUnitsDeclaredByTheUpperLeafEntry）：同号两条时后读到的盖掉先读到的
fn r_note_the_declaration_of_an_upper_leaf_entry(inode: vec2<u32>, tag: u32) {
    if (!r_counting_data_units_of_each_inode) {
        return;
    }
    let row = d_data_units_row_of_inode_noting_it(inode);
    if (row == NONE) {
        return;
    }
    var declaration = D_DECLARED_DATA_UNITS_WITHOUT_A_HOLE_AFTER_THEM;
    if (tag == EXTENT_UPPER_LEAF_TAG_NO_DATA_UNIT) {
        declaration = D_DECLARED_NO_DATA_UNIT;
    } else if (tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE || tag == EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE) {
        declaration = D_DECLARED_A_HOLE_UP_TO_THE_SIZE_AFTER_THE_LAST_DATA_UNIT;
    }
    sws(row + 3u, declaration);
}

// 打包表 base 起 pointer_offset 处那条数据指针 p 记成号 inode、文件字节偏移 offset_in_bytes 的一个单元（全零的不记）
fn r_note_data_unit_of_inode(inode: vec2<u32>, p: Ptr, base: u32, pointer_offset: u32, offset_in_bytes: vec2<u32>) {
    if (!r_counting_data_units_of_each_inode || p.all_zero) {
        return;
    }
    let row = d_data_units_row_of_inode_noting_it(inode);
    if (row == NONE) {
        return;
    }
    if (d_data_unit_count >= D_MAX_DATA_UNITS) {
        undecidable = D_SITE_DATA_UNITS_PAST_THE_TABLE;
        return;
    }
    let unit = D_DATA_UNITS + d_data_unit_count * D_DATA_UNIT_WORDS;
    sws(unit, offset_in_bytes.x);
    sws(unit + 1u, offset_in_bytes.y);
    sws(unit + 2u, base);
    sws(unit + 3u, pointer_offset);
    sws(unit + 4u, sw(row + 4u));
    d_data_unit_count = d_data_unit_count + 1u;
    sws(row + 4u, d_data_unit_count);
    sws(row + 2u, sw(row + 2u) + 1u);
}

// read_index_node_without_judging 按指针本身读的那一支（walk.rs 的 PositionOfASecondaryRead::ThePointerItself：自举豁免的三类——
// 树表单元、映射树自己的节点；实例表单元另经 read_unit_via_locations 读）：第一份校验和对得上的那一份，按 index_node_view 解
fn r_read_index_node(p: Ptr) -> Node {
    let s = read_unit_via_locations(p, NODE_BYTES);
    if (s.version == NONE) {
        return bad_node();
    }
    return parse_index_node(s);
}

// ── 次级读法按这一版的中央映射认位置（D19（块指针的结构与宽度预算） 已定项 5、8；walk.rs 的 PositionOfASecondaryRead::TheCentralMappingOfTheVersion）
// 只读不判的各遍逐版读：进一版之前装这一版的中央映射（C_VERSION_MAPPING_ROWS，read_the_central_mapping_without_judging），豁免三类之外的
// 码 1 / 2 / 3 经 r_read_located_unit 读。装着的是哪一条映射根指针记在这里：同一条根指针（逐段相同，指的是同一个单元、读出同一张表）不重读
var<private> s_mapping_root_loaded: Ptr;
var<private> s_mapping_root_is_loaded: bool;

// 装 mapping_root 指的那一版中央映射（读不全时是空表，照提示读；根指针全零是那一版没有映射，同样是空表）
fn s_use_the_central_mapping_of(mapping_root: Ptr) {
    if (s_mapping_root_is_loaded && mapping_root.all_zero == s_mapping_root_loaded.all_zero && same_pointer(mapping_root, s_mapping_root_loaded)) {
        return;
    }
    read_the_central_mapping_without_judging(mapping_root);
    s_mapping_root_loaded = mapping_root;
    s_mapping_root_is_loaded = true;
    d_forget_the_inode_records_read_in_full();
}

// 打包表 base 起 offset 处那条映射根指针指的那一版中央映射（根环里的根取根记录偏移 256，由记录施加出来的那一版取新根段里的那一条）
fn s_use_the_central_mapping_named_at(base: u32, offset: u32) {
    s_use_the_central_mapping_of(parse_node_pointer(base, offset));
}

// 读一个豁免三类之外的单元、不判（walk.rs read_located_unit_without_judging）：按查找路径给的类在装着的那一版映射里认位置
// （located_by_the_central_mapping）；认到的位置与提示（指针自己的两条位置条目）不同时先按提示读，校验和对得上、头里出生身份与指针相同
// 才用（copy_at_a_location_hint）；读不出再按映射位置两条位置条目读、校验和过滤；两者相同时照位置条目读。提示那一槽已释放、被 discard、
// 被别的单元复用都不判。提示两份与位置两份在一个循环里读（一个 read_at 调用点）
fn r_read_located_unit(p: Ptr, mapping_class: u32, unit_bytes: u32) -> Slice {
    let located = located_by_the_central_mapping(p, mapping_class);
    let hints_differ = located.device0 != p.device0 || !eq64(located.slot0, p.slot0) || located.device1 != p.device1 || !eq64(located.slot1, p.slot1);
    for (var copy = 0u; copy < 4u; copy = copy + 1u) {
        let is_a_hint = copy < 2u;
        if (is_a_hint && !hints_differ) {
            continue;
        }
        let is_the_second = (copy % 2u) == 1u;
        var device = select(located.device0, located.device1, is_the_second);
        var slot = select(located.slot0, located.slot1, is_the_second);
        if (is_a_hint) {
            device = select(p.device0, p.device1, is_the_second);
            slot = select(p.slot0, p.slot1, is_the_second);
        }
        let s = read_at(device, slot_to_bytes(slot), unit_bytes);
        if (s.version != NONE && slice_crc_matches(s, unit_bytes, select(p.sum0, p.sum1, is_the_second))
            && (!is_a_hint || unit_birth_identity_matches_the_pointer(s, p))) {
            return s;
        }
    }
    return no_slice();
}

// read_index_node_without_judging 按这一版中央映射认位置的那一支（豁免三类之外的码 2 节点）
fn r_read_located_index_node(p: Ptr) -> Node {
    let s = r_read_located_unit(p, MAPPING_CLASS_INDEX_NODE, NODE_BYTES);
    if (s.version == NONE) {
        return bad_node();
    }
    return parse_index_node(s);
}

fn r_seen_insert(device: u32, slot: vec2<u32>) -> bool {
    for (var i = 0u; i < r_seen_count; i = i + 1u) {
        if (sw(C_R_SEEN + i * 3u) == device && sw(C_R_SEEN + i * 3u + 1u) == slot.x && sw(C_R_SEEN + i * 3u + 2u) == slot.y) {
            return false;
        }
    }
    if (r_seen_count >= C_R_MAX_SEEN) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (19u << 8u);
        return false;
    }
    sws(C_R_SEEN + r_seen_count * 3u, device);
    sws(C_R_SEEN + r_seen_count * 3u + 1u, slot.x);
    sws(C_R_SEEN + r_seen_count * 3u + 2u, slot.y);
    r_seen_count = r_seen_count + 1u;
    return true;
}

// C_ALLOC_RECORDS 一行第八个字里这一位：跨度段带隔离记号（0x4000，D19（块指针的结构与宽度预算） 已定项 5；walk.rs AllocationRecordView 的
// quarantine_mark）。同一个字的低两位是 I-3.15 那一遍的记号（G_RECORD_OFF_THE_CENTRAL_MAPPING 那两个值），两样各占各的位
const C_ALLOC_RECORD_QUARANTINE_MARK_BIT: u32 = 0x80000000u;

fn c_append_allocation_record(device: u32, slot: vec2<u32>, span: u32, generation: vec2<u32>, released: u32, quarantine_mark: u32) -> bool {
    if (c_alloc_record_count >= C_MAX_ALLOC_RECORDS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (20u << 8u);
        return false;
    }
    let row = C_ALLOC_RECORDS + c_alloc_record_count * C_ALLOC_RECORD_WORDS;
    sws(row, device);
    sws(row + 1u, slot.x);
    sws(row + 2u, slot.y);
    sws(row + 3u, span);
    sws(row + 4u, generation.x);
    sws(row + 5u, generation.y);
    sws(row + 6u, released);
    sws(row + 7u, select(0u, C_ALLOC_RECORD_QUARANTINE_MARK_BIT, quarantine_mark == 1u));
    c_alloc_record_count = c_alloc_record_count + 1u;
    return true;
}

// allocation_record_tree_without_judging：整棵读，记录追加进 C_ALLOC_RECORDS；哪一步读不下去交回 false（记录数复位）
fn r_allocation_tree(base: u32, offset: u32) -> bool {
    let records_before = c_alloc_record_count;
    let root_level = allocation_root_level();
    if (root_level == 255u) {
        return false;
    }
    r_seen_count = 0u;
    var pending = 1u;
    sws(C_R_PENDING, base);
    sws(C_R_PENDING + 1u, offset);
    sws(C_R_PENDING + 2u, 1u);
    sws(C_R_PENDING + 3u, root_level);
    sws(C_R_PENDING + 4u, NONE);
    sws(C_R_PENDING + 5u, 0u);
    sws(C_R_PENDING + 6u, 0u);
    var ok = true;
    while (pending > 0u && ok) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let p = parse_node_pointer(sw(row), sw(row + 1u));
        let is_root = sw(row + 2u) == 1u;
        let level = sw(row + 3u);
        let cell_device = sw(row + 4u);
        let cell_index = vec2<u32>(sw(row + 5u), sw(row + 6u));
        if (p.all_zero || !r_seen_insert(p.device0, p.slot0)) {
            ok = false;
            break;
        }
        r_note_located(p, MAPPING_CLASS_INDEX_NODE);
        let n = r_read_located_index_node(p);
        if (!n.ok || n.level != level || n.key_width != 10u) {
            ok = false;
            break;
        }
        if (level == 0u) {
            if (n.entry_width < 20u) {
                ok = false;
                break;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                let o = entry_offset(n, e);
                let span_field = rd16(n.base, o + 10u);
                // 跨度去掉已释放标志与隔离记号；两个标志各记一个字（walk.rs parse_allocation_record：同时置的也照读，由 I-3.17 判红）
                if (!c_append_allocation_record(rd32(n.base, o), rd48(n.base, o + 4u), span_field & 0x3FFFu, rd64(n.base, o + 12u), span_field >> 15u,
                    (span_field >> 14u) & 1u)) {
                    ok = false;
                    break;
                }
            }
            continue;
        }
        if (n.entry_width < 96u) {
            ok = false;
            break;
        }
        let child_span = allocation_span(level - 1u);
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            let device = rd32(n.base, o);
            let slot = rd48(n.base, o + 4u);
            let qr = divmod64(slot, child_span);
            if (!is_zero64(qr.remainder) || !device_is_in_the_pool(device)) {
                ok = false;
                break;
            }
            if (!is_root && (device != cell_device || !eq64(divmod64(qr.quotient, u64_of(ALLOCATION_INTERNAL_FANOUT)).quotient, cell_index))) {
                ok = false;
                break;
            }
            if (pending >= C_R_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (21u << 8u);
                ok = false;
                break;
            }
            let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
            sws(child, n.base);
            sws(child + 1u, o + 10u);
            sws(child + 2u, 0u);
            sws(child + 3u, level - 1u);
            sws(child + 4u, device);
            sws(child + 5u, qr.quotient.x);
            sws(child + 6u, qr.quotient.y);
            pending = pending + 1u;
        }
    }
    if (!ok) {
        c_alloc_record_count = records_before;
    }
    return ok;
}

// extent_tree_without_judging：整棵读（上段与每个文件的下段），节点指针与数据指针都记进引用集合
fn r_extent_tree(base: u32, offset: u32) -> bool {
    r_seen_count = 0u;
    var pending = 1u;
    sws(C_R_PENDING, base);
    sws(C_R_PENDING + 1u, offset);
    sws(C_R_PENDING + 2u, 0u);   // 下段
    sws(C_R_PENDING + 3u, 0u);   // 有位置
    sws(C_R_PENDING + 4u, 0u);   // 层级
    sws(C_R_PENDING + 5u, 0u);   // 序号 lo
    sws(C_R_PENDING + 6u, 0u);   // 序号 hi
    sws(C_R_PENDING + 7u, 0u);   // inode lo
    sws(C_R_PENDING + 8u, 0u);   // inode hi
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let p = parse_node_pointer(sw(row), sw(row + 1u));
        let is_lower = sw(row + 2u) == 1u;
        let has_position = sw(row + 3u) == 1u;
        var level = sw(row + 4u);
        let index = vec2<u32>(sw(row + 5u), sw(row + 6u));
        let inode = vec2<u32>(sw(row + 7u), sw(row + 8u));
        if (p.all_zero || !r_seen_insert(p.device0, p.slot0)) {
            return false;
        }
        r_note_located(p, MAPPING_CLASS_INDEX_NODE);
        if (r_unit_keys_go_to != 0u) {
            s_note_a_unit_key(mapping_key_of_a_pointer(p, MAPPING_CLASS_INDEX_NODE), r_unit_keys_go_to);
        }
        let n = r_read_located_index_node(p);
        if (!n.ok) {
            return false;
        }
        if (!has_position) {
            level = n.level;
        }
        if (n.level != level || n.key_width != 24u) {
            return false;
        }
        if (level == 0u) {
            if (!is_lower) {
                if (n.entry_width < 113u) {
                    return false;
                }
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
                        return false;
                    }
                    let tag = rd8(n.base, o + 24u);
                    let entry_inode = extent_key_inode(n.base, o);
                    // 标签 3 同 1（下段根）、4 同 2（内联的数据指针）：判决 m4-trunc-r1 K5 的「之后到 size 是洞」（walk.rs extent_upper_leaf_entry_view）
                    if (tag == EXTENT_UPPER_LEAF_TAG_NO_DATA_UNIT) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return false;
                        }
                        r_note_the_declaration_of_an_upper_leaf_entry(entry_inode, tag);
                    } else if (tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT || tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE) {
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return false;
                        }
                        r_note_the_declaration_of_an_upper_leaf_entry(entry_inode, tag);
                        if (pending >= C_R_MAX_PENDING) {
                            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (22u << 8u);
                            return false;
                        }
                        let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
                        sws(child, n.base);
                        sws(child + 1u, o + 25u);
                        sws(child + 2u, 1u);
                        sws(child + 3u, 0u);
                        sws(child + 4u, 0u);
                        sws(child + 5u, 0u);
                        sws(child + 6u, 0u);
                        sws(child + 7u, entry_inode.x);
                        sws(child + 8u, entry_inode.y);
                        pending = pending + 1u;
                    } else if (tag == EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT || tag == EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE) {
                        r_note_the_declaration_of_an_upper_leaf_entry(entry_inode, tag);
                        let inline_data_pointer = parse_data_pointer(n.base, o + 25u);
                        r_note_data_pointer(inline_data_pointer);
                        r_note_data_unit_of_inode(entry_inode, inline_data_pointer, n.base, o + 25u, vec2<u32>(0u, 0u));
                    } else {
                        return false;
                    }
                }
            } else {
                if (n.entry_width < 112u) {
                    return false;
                }
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let record_offset = entry_offset(n, e);
                    let lower_data_pointer = parse_data_pointer(n.base, record_offset + 24u);
                    r_note_data_pointer(lower_data_pointer);
                    r_note_data_unit_of_inode(inode, lower_data_pointer, n.base, record_offset + 24u, extent_key_offset(n.base, record_offset));
                }
            }
            continue;
        }
        if (n.entry_width < 110u) {
            return false;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            var child_index = vec2<u32>(0u, 0u);
            if (is_lower) {
                let child_offset_span = mul64_64_saturating(extent_lower_span(level - 1u), payload);
                let qr = divmod64(extent_key_offset(n.base, o), child_offset_span);
                if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode) || !is_zero64(qr.remainder)) {
                    return false;
                }
                child_index = qr.quotient;
            } else {
                let qr = divmod64(extent_key_inode(n.base, o), extent_upper_span(level - 1u));
                if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o)) || !is_zero64(qr.remainder)) {
                    return false;
                }
                child_index = qr.quotient;
            }
            if (!eq64(divmod64(child_index, u64_of(EXTENT_INTERNAL_FANOUT)).quotient, index)) {
                return false;
            }
            if (pending >= C_R_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (23u << 8u);
                return false;
            }
            let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
            sws(child, n.base);
            sws(child + 1u, o + 24u);
            sws(child + 2u, select(0u, 1u, is_lower));
            sws(child + 3u, 1u);
            sws(child + 4u, level - 1u);
            sws(child + 5u, child_index.x);
            sws(child + 6u, child_index.y);
            sws(child + 7u, inode.x);
            sws(child + 8u, inode.y);
            pending = pending + 1u;
        }
    }
    return true;
}

// note_every_node_below：多层码 2 树根之下的全部节点记进引用集合
fn r_note_every_node_below(root: Node, schema: u32) {
    if (root.level == 0u) {
        return;
    }
    let key_width = schema_key_width(schema);
    if (root.key_width != key_width || root.entry_width < key_width + 86u) {
        r_complete = false;
        return;
    }
    var pending = 0u;
    for (var e = 0u; e < root.entry_count; e = e + 1u) {
        if (pending >= C_R_MAX_PENDING) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (24u << 8u);
            return;
        }
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        sws(row, root.base);
        sws(row + 1u, entry_offset(root, e) + key_width);
        sws(row + 2u, root.level - 1u);
        pending = pending + 1u;
    }
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let p = parse_node_pointer(sw(row), sw(row + 1u));
        let expected_level = sw(row + 2u);
        r_note_located(p, MAPPING_CLASS_INDEX_NODE);
        let child = r_read_located_index_node(p);
        if (!child.ok || child.level != expected_level) {
            r_complete = false;
            continue;
        }
        if (child.level == 0u) {
            continue;
        }
        if (child.key_width != key_width || child.entry_width < key_width + 86u) {
            r_complete = false;
            continue;
        }
        for (var e = 0u; e < child.entry_count; e = e + 1u) {
            if (pending >= C_R_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (25u << 8u);
                return;
            }
            let grandchild = C_R_PENDING + pending * C_R_PENDING_WORDS;
            sws(grandchild, child.base);
            sws(grandchild + 1u, entry_offset(child, e) + key_width);
            sws(grandchild + 2u, child.level - 1u);
            pending = pending + 1u;
        }
    }
}

// note_every_inode_child_below：inode 树内部节点下面的孩子都记进引用集合
fn r_note_every_inode_child_below(root: Node) {
    var pending = 0u;
    for (var e = 0u; e < root.entry_count; e = e + 1u) {
        if (pending >= C_R_MAX_PENDING) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (26u << 8u);
            return;
        }
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        sws(row, root.base);
        sws(row + 1u, entry_offset(root, e));
        sws(row + 2u, root.level);
        pending = pending + 1u;
    }
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let base = sw(row);
        let o = sw(row + 1u);
        let parent_level = sw(row + 2u);
        let p = parse_node_pointer(base, o + 34u);
        let record_type = rd16(base, o + 16u);
        // 叶容器按码 3、内部节点按码 2 查这条根的中央映射认位置；类型段不认得的按指针自己的位置记、数不全（walk.rs note_every_inode_child_below）
        if (record_type == 2u) {
            r_note_located(p, MAPPING_CLASS_PACKED_RECORD_UNIT);
            continue;
        }
        if (record_type != 0u) {
            r_note(p);
            r_complete = false;
            continue;
        }
        r_note_located(p, MAPPING_CLASS_INDEX_NODE);
        let child = r_read_located_index_node(p);
        if (!child.ok || parent_level == 0u || child.level != parent_level - 1u || child.level == 0u || child.entry_width < 120u) {
            r_complete = false;
            continue;
        }
        for (var e = 0u; e < child.entry_count; e = e + 1u) {
            if (pending >= C_R_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (27u << 8u);
                return;
            }
            let grandchild = C_R_PENDING + pending * C_R_PENDING_WORDS;
            sws(grandchild, child.base);
            sws(grandchild + 1u, entry_offset(child, e));
            sws(grandchild + 2u, child.level);
            pending = pending + 1u;
        }
    }
}

// note_instance_table_pages_after_the_first
fn r_instance_pages_after_first(first: Ptr) {
    var p = first;
    var page = vec2<u32>(0u, 0u);
    for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {
        let s = read_unit_via_locations(p, DATA_UNIT_BYTES);
        let packed = parse_packed_unit(s);
        if (!packed.ok) {
            r_complete = false;
            return;
        }
        if (!is_zero64(packed.birth_tree) || packed.record_type != 4u || !eq64(packed.container, page) || !is_zero64(packed.container_birth)) {
            r_complete = false;
            return;
        }
        if (packed.record_count == 0u) {
            r_complete = false;
            return;
        }
        // I-3.14 不成立的一片解不开（D18 已定项 11 的补句）：链指针记录之前有不是 kind 0 的记录（kind 不认得、
        // 片中间有链指针记录）或行的 flags 非 0，数不全（walk.rs note_instance_table_pages_after_the_first 同判）
        for (var r = 0u; r + 1u < packed.record_count; r = r + 1u) {
            let o = packed_record_offset(packed, r);
            if (rd8(packed.base, o) != 0u || rd8(packed.base, o + 21u) != 0u) {
                r_complete = false;
                return;
            }
        }
        let last = packed_record_offset(packed, packed.record_count - 1u);
        // 有下一片而位置指针头部不是第一版的取值（I-3.14 第五形）同样交 2u：这条指针不记、数不全
        let chain = w_chain_record_kind(packed.base, last, packed.record_width);
        if (chain == 0u) {
            return;
        }
        if (chain == 2u) {
            r_complete = false;
            return;
        }
        let next = parse_node_pointer(packed.base, last + 2u);
        r_note(next);
        if (g_noting_the_instance_table_pages) {
            g_note_a_pointer_named_directly(next);
        }
        p = next;
        page = add64(page, u64_of(1u));
    }
    r_complete = false;
    undecidable = UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED;
}

// 树表一条条目（偏移 o 起，宽 ≥ 200）的树 ID（偏移 0）与诞生 txg（偏移 108）记进信息行 row：同一棵树在这一片里再出现时后一条盖前一条
// （walk.rs 的 tree_table_birth_txg 是按树 ID 插入的 BTreeMap）
fn r_note_tree_table_entry_birth_txg(row: u32, tree_table_base: u32, o: u32) {
    let tree = rd64(tree_table_base, o);
    let birth = rd64(tree_table_base, o + 108u);
    let tree_count = sw(row + 5u);
    var found = false;
    for (var t = 0u; t < tree_count; t = t + 1u) {
        if (sw(row + 6u + t * 4u) == tree.x && sw(row + 7u + t * 4u) == tree.y) {
            sws(row + 8u + t * 4u, birth.x);
            sws(row + 9u + t * 4u, birth.y);
            found = true;
        }
    }
    if (!found) {
        if (tree_count >= C_MAX_TREES_PER_TABLE) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (28u << 8u);
        } else {
            sws(row + 6u + tree_count * 4u, tree.x);
            sws(row + 7u + tree_count * 4u, tree.y);
            sws(row + 8u + tree_count * 4u, birth.x);
            sws(row + 9u + tree_count * 4u, birth.y);
            sws(row + 5u, tree_count + 1u);
        }
    }
}

// references_of_root：一条根引用了什么（只读不判），走全没走全、树表落点与树表条目的诞生 txg 记进候选根的信息行。
// 走到叶那个深度上先读这条根自己的中央映射：豁免三类之外的指针按它认位置（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；
// 读不全时是空表，照提示数）。销毁中的快照（码 14 带销毁中记号、连同头 ID 配它的码 15）不经它们的树引用任何落点（D5（快照 / 空间记账机制）
// 已定项 20 的例外句）；活快照的两棵树照 inode 树、extent 树数；deadlist 树照多层码 2 树数节点；映射叶里每条映射条目指的单元也数
// （被快照钉住、映射条目留存的那几个，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 的 α）
// 这一遍有一条候选根的树表没读全（指针全零、读不出、有条目比字段表短）：I-9.14 记射程里判不了（C701，walk.rs judge_tree_table_birth_txg
// 报部分没判、逐条列出那几条根），judge_release_generation_and_tree_table_birth 开头清掉
var<private> r_some_candidate_tree_table_is_not_read_whole: bool;

fn r_references_of_root(record_base: u32, every: bool, info: u32) {
    let row = C_CANDIDATE_INFO + info * C_CANDIDATE_WORDS;
    r_complete = true;
    sws(row + 1u, 0u);
    sws(row + 5u, 0u);
    if (every) {
        s_use_the_central_mapping_named_at(record_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    } else {
        version_mapping_count = 0u;
        version_mapping_is_indexed = false;
        s_mapping_root_is_loaded = false;
        d_forget_the_inode_records_read_in_full();
    }
    let instance_table = parse_node_pointer(record_base, 170u);
    r_note(instance_table);
    if (every) {
        r_instance_pages_after_first(instance_table);
    }
    let mapping_root = parse_node_pointer(record_base, 256u);
    r_note(mapping_root);
    let allocation_root = parse_node_pointer(record_base, 342u);
    r_note(allocation_root);
    if (every && !allocation_root.all_zero) {
        if (!r_allocation_tree(record_base, 342u)) {
            r_complete = false;
        }
    }
    let tree_table_pointer = parse_node_pointer(record_base, 36u);
    r_note(tree_table_pointer);
    if (tree_table_pointer.all_zero) {
        r_complete = false;
        r_some_candidate_tree_table_is_not_read_whole = true;
        sws(row, 0u);
        return;
    }
    sws(row + 1u, 1u);
    sws(row + 2u, tree_table_pointer.device0);
    sws(row + 3u, tree_table_pointer.slot0.x);
    sws(row + 4u, tree_table_pointer.slot0.y);
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok) {
        r_complete = false;
        r_some_candidate_tree_table_is_not_read_whole = true;
        sws(row, 0u);
        return;
    }
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        if (tt.entry_width < 200u) {
            r_complete = false;
            r_some_candidate_tree_table_is_not_read_whole = true;
            continue;
        }
        let o = entry_offset(tt, e);
        r_note_tree_table_entry_birth_txg(row, tt.base, o);
        let tree_root = parse_node_pointer(tt.base, o + 14u);
        if (tree_root.all_zero) {
            continue;
        }
        let kind = rd16(tt.base, o + 10u);
        if ((kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) && snapshot_tree_is_being_destroyed(tt, e)) {
            continue;
        }
        r_note_located(tree_root, MAPPING_CLASS_INDEX_NODE);
        if (!every) {
            continue;
        }
        let node = r_read_located_index_node(tree_root);
        if (!node.ok) {
            r_complete = false;
            continue;
        }
        if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            if (!r_extent_tree(tt.base, o + 14u)) {
                r_complete = false;
            }
        } else if (kind == TREE_KIND_INODE || kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
            if (node.level > 0u && node.entry_width >= 120u) {
                r_note_every_inode_child_below(node);
            } else {
                r_complete = false;
            }
        } else if (kind == TREE_KIND_ALLOCATION) {
            if (!r_allocation_tree(tt.base, o + 14u)) {
                r_complete = false;
            }
        } else if (kind == TREE_KIND_ACCOUNTING || kind == TREE_KIND_DEADLIST || kind == TREE_KIND_LOGGED_OPERATIONS) {
            // deadlist 的条目只是 key（被钉单元的中央映射 key），不引用别的单元；被钉的单元由映射叶里留着的那条映射条目记进来（下面映射那一段）。
            // logged_ops 的条目是意图，也不引用别的单元（walk.rs 的 TreeKindForReferenceScan::LoggedOperations）：三棵都只数根之下的节点
            r_note_every_node_below(node, select(select(0u, KEY_SCHEMA_DEADLIST, kind == TREE_KIND_DEADLIST), KEY_SCHEMA_LOGGED_OPERATIONS,
                kind == TREE_KIND_LOGGED_OPERATIONS));
        } else if (kind == TREE_KIND_DELETED_INODES) {
            // deleted_inodes 根的条目只是号，不引用别的单元（根自己上面记过）
        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
            // 墓碑树、谱系树、退役树（walk.rs 的 TreeKindForReferenceScan::ContainerTree）：根是层级 1 时每条条目的子指针（条目偏移 34 起）指一片容器；
            // 根长高之后（层级 > 1）这一遍不往下数，记成数不全
            if (node.level == 1u && node.entry_width >= CONTAINER_TREE_ENTRY_BYTES) {
                for (var container = 0u; container < node.entry_count; container = container + 1u) {
                    r_note_located(parse_node_pointer(node.base, entry_offset(node, container) + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET),
                        MAPPING_CLASS_PACKED_RECORD_UNIT);
                }
            } else {
                r_complete = false;
            }
        } else {
            r_complete = false;
        }
    }
    if (every && !mapping_root.all_zero) {
        let mapping = r_read_index_node(mapping_root);
        if (mapping.ok) {
            r_note_every_node_and_every_mapped_unit_below_the_mapping_root(mapping);
        } else {
            r_complete = false;
        }
    }
    sws(row, select(0u, 1u, r_complete));
}

// walk.rs note_every_node_and_every_mapped_unit_below_the_mapping_root：中央映射树根之下的每个节点记进引用集合（映射树自己的节点按指针自己的位置），
// 叶里每条映射条目的两条位置条目也记进去。key 宽不是 27、条目宽窄于字段表、孩子读不出、孩子的层级不是父层级减一，都如实记成数不全、不往下走。
// 递归写成显式栈（C_R_PENDING：字节基、子指针偏移、该有的层级），每次从栈里取一个孩子：上界是节点数
fn r_note_every_node_and_every_mapped_unit_below_the_mapping_root(root: Node) {
    var pending = 0u;
    var n = root;
    loop {
        if (n.key_width != MAPPING_KEY_BYTES) {
            r_complete = false;
        } else if (n.level == 0u) {
            if (n.entry_width < MAPPING_ENTRY_BYTES) {
                r_complete = false;
            } else {
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e) + MAPPING_KEY_BYTES;
                    var locations = zero_pointer();
                    locations.all_zero = all_zero(n.base, o, 28u);
                    locations.device0 = rd32(n.base, o);
                    locations.slot0 = rd48(n.base, o + 4u);
                    locations.sum0 = rd32(n.base, o + 10u);
                    locations.device1 = rd32(n.base, o + 14u);
                    locations.slot1 = rd48(n.base, o + 18u);
                    locations.sum1 = rd32(n.base, o + 24u);
                    if (!locations.all_zero) {
                        r_note(locations);
                    }
                }
            }
        } else if (n.entry_width < MAPPING_KEY_BYTES + 86u) {
            r_complete = false;
        } else {
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                if (pending >= C_R_MAX_PENDING) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (94u << 8u);
                    return;
                }
                let child_row = C_R_PENDING + pending * C_R_PENDING_WORDS;
                sws(child_row, n.base);
                sws(child_row + 1u, entry_offset(n, e) + MAPPING_KEY_BYTES);
                sws(child_row + 2u, n.level - 1u);
                pending = pending + 1u;
            }
        }
        var next_node_found = false;
        while (pending > 0u) {
            pending = pending - 1u;
            let child_row = C_R_PENDING + pending * C_R_PENDING_WORDS;
            let child_pointer = parse_node_pointer(sw(child_row), sw(child_row + 1u));
            let expected_level = sw(child_row + 2u);
            r_note(child_pointer);
            let child = r_read_index_node(child_pointer);
            if (!child.ok || child.level != expected_level) {
                r_complete = false;
                continue;
            }
            n = child;
            next_node_found = true;
            break;
        }
        if (!next_node_found) {
            break;
        }
    }
}

// I-3.9 与 I-9.14：候选集里每条根各走一遍取引用集合与树表
fn judge_release_generation_and_tree_table_birth() {
    if (c_candidate_count < 2u) {
        return;
    }
    // 先走最新根一遍取它的账：已释放记录定这一遍走多深
    var newest_slot = 0u;
    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
        if (sw(C_CANDIDATES + c) == c_newest_index) {
            newest_slot = c;
        }
    }
    let newest_root = checker_root_of(c_newest_index);
    r_some_candidate_tree_table_is_not_read_whole = false;
    // 第 0 圈走最新根一遍取它的账，之后每圈走一条候选根：同一个调用点（GPU 程序把每个调用点展开成一份拷贝）
    var every = false;
    for (var walk_index = 0u; walk_index <= c_candidate_count; walk_index = walk_index + 1u) {
        var record_base = newest_root.base;
        var walks_every_node = true;
        var candidate_slot = newest_slot;
        if (walk_index == 0u) {
            r_noting = false;
        } else {
            let candidate = walk_index - 1u;
            let index = sw(C_CANDIDATES + candidate);
            record_base = checker_root_of(index).base;
            walks_every_node = every || index == c_newest_index;
            candidate_slot = candidate;
            r_candidate = candidate;
            r_noting = every;
        }
        c_alloc_record_count = 0u;
        r_references_of_root(record_base, walks_every_node, candidate_slot);
        if (walk_index != 0u) {
            continue;
        }
        c_released_count = 0u;
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            if (sw(record + 6u) == 0u) {
                continue;
            }
            if (c_released_count >= C_MAX_RELEASED) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (29u << 8u);
                return;
            }
            sws(C_RELEASED + c_released_count * 5u, sw(record));
            sws(C_RELEASED + c_released_count * 5u + 1u, sw(record + 1u));
            sws(C_RELEASED + c_released_count * 5u + 2u, sw(record + 2u));
            sws(C_RELEASED + c_released_count * 5u + 3u, sw(record + 4u));
            sws(C_RELEASED + c_released_count * 5u + 4u, sw(record + 5u));
            sws(C_REFERENCED_BY + c_released_count * 2u, 0u);
            sws(C_REFERENCED_BY + c_released_count * 2u + 1u, 0u);
            c_released_count = c_released_count + 1u;
        }
        // r_note 读的桶表：表至多 C_MAX_RELEASED 条、桶数是它的两倍，往后挪总找得到空桶
        for (var bucket = 0u; bucket < C_RELEASED_BUCKET_COUNT; bucket = bucket + 1u) {
            sws(C_RELEASED_BUCKETS + bucket, 0u);
        }
        for (var released_index = 0u; released_index < c_released_count; released_index = released_index + 1u) {
            let released = C_RELEASED + released_index * 5u;
            var bucket = slot_key_first_bucket(sw(released), vec2<u32>(sw(released + 1u), sw(released + 2u)), C_RELEASED_BUCKET_COUNT);
            for (var probe = 0u; probe < C_RELEASED_BUCKET_COUNT; probe = probe + 1u) {
                if (sw(C_RELEASED_BUCKETS + bucket) == 0u) {
                    sws(C_RELEASED_BUCKETS + bucket, released_index + 1u);
                    break;
                }
                bucket = (bucket + 1u) & (C_RELEASED_BUCKET_COUNT - 1u);
            }
        }
        every = c_released_count > 0u;
    }
    r_noting = false;
    // I-3.9（walk.rs judge_release_generations）：最新根下面每条已释放记录，取引用它的候选根里 txg 最大的那一条；比它晚的候选根有一条引用集合不全时
    // 这一条跳过、记射程里有一部分没判，否则判一次（已释放代在 (最后引用它的那一版, 头一个不引用它的那一版] 里）
    if (every) {
        for (var k = 0u; k < c_released_count; k = k + 1u) {
            let bits_lo = sw(C_REFERENCED_BY + k * 2u);
            let bits_hi = sw(C_REFERENCED_BY + k * 2u + 1u);
            var have_last = false;
            var last = vec2<u32>(0u, 0u);
            for (var c = 0u; c < c_candidate_count; c = c + 1u) {
                var bit = false;
                if (c < 32u) {
                    bit = (bits_lo & (1u << c)) != 0u;
                } else {
                    bit = (bits_hi & (1u << (c - 32u))) != 0u;
                }
                if (!bit) {
                    continue;
                }
                let txg = checker_root_of(sw(C_CANDIDATES + c)).txg;
                if (!have_last || lt64(last, txg)) {
                    last = txg;
                    have_last = true;
                }
            }
            if (!have_last) {
                continue;
            }
            var a_later_root_is_incomplete = false;
            var have_first = false;
            var first_without = vec2<u32>(0u, 0u);
            for (var c = 0u; c < c_candidate_count; c = c + 1u) {
                let txg = checker_root_of(sw(C_CANDIDATES + c)).txg;
                if (lt64(last, txg)) {
                    if (sw(C_CANDIDATE_INFO + c * C_CANDIDATE_WORDS) == 0u) {
                        a_later_root_is_incomplete = true;
                    }
                    if (!have_first || lt64(txg, first_without)) {
                        first_without = txg;
                        have_first = true;
                    }
                }
            }
            if (a_later_root_is_incomplete) {
                mark_part_of_the_range_not_judged(INVARIANT_I_3_9);
                continue;
            }
            let generation = vec2<u32>(sw(C_RELEASED + k * 5u + 3u), sw(C_RELEASED + k * 5u + 4u));
            judge(INVARIANT_I_3_9, lt64(last, generation) && have_first && le64(generation, first_without));
        }
    }
    // I-9.14：候选集里的根；有一条的树表没读全时记射程里判不了（CPU 报部分没判，C701）
    judge_tree_table_birth_txg_across_the_rows(c_candidate_count, INVARIANT_I_9_14);
    if (r_some_candidate_tree_table_is_not_read_whole) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_14);
    }
}

// 按树 ID 归并 C_CANDIDATE_INFO 前 row_count 行记下的树表条目：一棵树的条目出现在两个不同的树表单元里、诞生 txg 又不一，置 invariant 的违例位。
// 没比到两个不同树表单元的树不判；树表读不出、条目比字段表短的那几行没有条目，只是少一份证据（不适用、部分没判与成立都不置位）。
// I-9.14 交候选根那几行，I-9.28 交根表里每一条根那几行
fn judge_tree_table_birth_txg_across_the_rows(row_count: u32, invariant: u32) {
    for (var a = 0u; a < row_count; a = a + 1u) {
        let row_a = C_CANDIDATE_INFO + a * C_CANDIDATE_WORDS;
        if (sw(row_a + 1u) == 0u) {
            continue;
        }
        let tree_count_a = sw(row_a + 5u);
        for (var t = 0u; t < tree_count_a; t = t + 1u) {
            let tree = vec2<u32>(sw(row_a + 6u + t * 4u), sw(row_a + 7u + t * 4u));
            let birth_a = vec2<u32>(sw(row_a + 8u + t * 4u), sw(row_a + 9u + t * 4u));
            // 更早的候选根已经带着这棵树时那一轮判过了
            var handled = false;
            for (var b = 0u; b < a; b = b + 1u) {
                let row_b = C_CANDIDATE_INFO + b * C_CANDIDATE_WORDS;
                if (sw(row_b + 1u) == 0u) {
                    continue;
                }
                for (var u = 0u; u < sw(row_b + 5u); u = u + 1u) {
                    if (sw(row_b + 6u + u * 4u) == tree.x && sw(row_b + 7u + u * 4u) == tree.y) {
                        handled = true;
                    }
                }
            }
            if (handled) {
                continue;
            }
            // 这棵树的条目出现在几个不同的树表单元里、诞生 txg 是否都相同
            var distinct_tables = 0u;
            var same_birth = true;
            for (var c = a; c < row_count; c = c + 1u) {
                let row_c = C_CANDIDATE_INFO + c * C_CANDIDATE_WORDS;
                if (sw(row_c + 1u) == 0u) {
                    continue;
                }
                var has_tree = false;
                for (var u = 0u; u < sw(row_c + 5u); u = u + 1u) {
                    if (sw(row_c + 6u + u * 4u) == tree.x && sw(row_c + 7u + u * 4u) == tree.y) {
                        has_tree = true;
                        if (sw(row_c + 8u + u * 4u) != birth_a.x || sw(row_c + 9u + u * 4u) != birth_a.y) {
                            same_birth = false;
                        }
                    }
                }
                if (!has_tree) {
                    continue;
                }
                var placement_seen = false;
                for (var d = a; d < c; d = d + 1u) {
                    let row_d = C_CANDIDATE_INFO + d * C_CANDIDATE_WORDS;
                    if (sw(row_d + 1u) == 0u) {
                        continue;
                    }
                    var d_has_tree = false;
                    for (var u = 0u; u < sw(row_d + 5u); u = u + 1u) {
                        if (sw(row_d + 6u + u * 4u) == tree.x && sw(row_d + 7u + u * 4u) == tree.y) {
                            d_has_tree = true;
                        }
                    }
                    if (d_has_tree && sw(row_d + 2u) == sw(row_c + 2u) && sw(row_d + 3u) == sw(row_c + 3u) && sw(row_d + 4u) == sw(row_c + 4u)) {
                        placement_seen = true;
                    }
                }
                if (!placement_seen) {
                    distinct_tables = distinct_tables + 1u;
                }
            }
            // 比到两个以上不同的树表单元的树各判一次（walk.rs judge_tree_table_birth_txg 与 judge_tree_identifier_birth_txg_across_every_readable_root）
            if (distinct_tables >= 2u) {
                judge(invariant, same_birth);
            }
        }
    }
}

// references_of_root 按「只要树表」那个深度（walk.rs 的 ReferenceScanDepth::TreeTableOnly）：一条根的树表落点与条目的诞生 txg 记进信息行 row。
// 树表指针全零 ⇒ 没有落点；树表读不出、条目比字段表短 ⇒ 有落点、没有条目。交回树表的每一条条目读全了没有（walk.rs 的 TreeTableReading::EveryEntryRead）
fn r_tree_table_entries_of_root(record_base: u32, row: u32) -> bool {
    sws(row + 1u, 0u);
    sws(row + 5u, 0u);
    let tree_table_pointer = parse_node_pointer(record_base, 36u);
    if (tree_table_pointer.all_zero) {
        return false;
    }
    sws(row + 1u, 1u);
    sws(row + 2u, tree_table_pointer.device0);
    sws(row + 3u, tree_table_pointer.slot0.x);
    sws(row + 4u, tree_table_pointer.slot0.y);
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < 200u)) {
        return false;
    }
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        r_note_tree_table_entry_birth_txg(row, tt.base, entry_offset(tt, e));
    }
    return true;
}

// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，walk.rs 的 judge_tree_identifier_birth_txg_across_every_readable_root）：根环里每一条读得出、
// 自证过的根（根表：不按实例表判有效与否、不按 F 与窗口滤，含被崩溃恢复抛弃的旧线上的根）各读一遍树表，按树 ID 归并，诞生 txg 不一就违例。
// 不适用（没比到两个不同的树表单元）与部分没判（有根的树表指针非零而读不出、或条目比字段表短）都不置位，与成立一样
fn judge_tree_identifier_birth_txg_across_every_readable_root() {
    if (root_count > C_MAX_CANDIDATES) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (47u << 8u);
        return;
    }
    // 有一条根的树表没读全（指针全零、读不出、条目比字段表短）：CPU 报部分没判（没有违例时压过成立与不适用）
    for (var index = 0u; index < root_count; index = index + 1u) {
        if (!r_tree_table_entries_of_root(checker_root_of(index).base, C_CANDIDATE_INFO + index * C_CANDIDATE_WORDS)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_28);
        }
    }
    judge_tree_table_birth_txg_across_the_rows(root_count, INVARIANT_I_9_28);
}

// 去重表里这个根节点的下标，没见过是 NONE
fn c_seen_node_index(p: Ptr) -> u32 {
    for (var i = 0u; i < c_seen_node_count; i = i + 1u) {
        if (sw(C_SEEN_NODES + i * 4u) == p.device0 && sw(C_SEEN_NODES + i * 4u + 1u) == p.slot0.x
            && sw(C_SEEN_NODES + i * 4u + 2u) == p.slot0.y && sw(C_SEEN_NODES + i * 4u + 3u) == p.sum0) {
            return i;
        }
    }
    return NONE;
}

// I-5.4 的一版里一个分配记录树根：见过的交回它那一次读没读到叶；没见过的记进去表、整棵读，读到叶就判这一棵（judge_allocation_record_ranges）。
// 「同一棵树」按这一版中央映射认过的落点认（walk.rs judge_allocation_records_disjoint 的 located_allocation_root）：同一条过期提示在搬迁前后两版里
// 指的是同一个单元，认过之后落在同一处
fn c_allocation_root_of_a_version_walked_to_its_leaves(base: u32, offset: u32) -> bool {
    let p = located_by_the_central_mapping(parse_node_pointer(base, offset), MAPPING_CLASS_INDEX_NODE);
    let seen_index = c_seen_node_index(p);
    if (seen_index != NONE) {
        return (c_seen_node_walked_to_its_leaves[seen_index / 32u] & (1u << (seen_index % 32u))) != 0u;
    }
    if (!c_seen_node_insert(p)) {
        return false;
    }
    c_alloc_record_count = 0u;
    if (!r_allocation_tree(base, offset)) {
        return false;
    }
    judge_allocation_record_ranges();
    let inserted = c_seen_node_count - 1u;
    c_seen_node_walked_to_its_leaves[inserted / 32u] = c_seen_node_walked_to_its_leaves[inserted / 32u] | (1u << (inserted % 32u));
    return true;
}

// 分配记录树根节点（按第一条位置条目）去重：见过交回 false
fn c_seen_node_insert(p: Ptr) -> bool {
    for (var i = 0u; i < c_seen_node_count; i = i + 1u) {
        if (sw(C_SEEN_NODES + i * 4u) == p.device0 && sw(C_SEEN_NODES + i * 4u + 1u) == p.slot0.x
            && sw(C_SEEN_NODES + i * 4u + 2u) == p.slot0.y && sw(C_SEEN_NODES + i * 4u + 3u) == p.sum0) {
            return false;
        }
    }
    if (c_seen_node_count >= C_MAX_SEEN_NODES) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (30u << 8u);
        return false;
    }
    sws(C_SEEN_NODES + c_seen_node_count * 4u, p.device0);
    sws(C_SEEN_NODES + c_seen_node_count * 4u + 1u, p.slot0.x);
    sws(C_SEEN_NODES + c_seen_node_count * 4u + 2u, p.slot0.y);
    sws(C_SEEN_NODES + c_seen_node_count * 4u + 3u, p.sum0);
    c_seen_node_count = c_seen_node_count + 1u;
    return true;
}

// I-5.4 的一棵树：同一块盘上任意两条记录罩住的槽区间互不相交。两两比的写法是「同盘的 a、b，(槽, 跨度, 行号) 字典序 a 在前、b 的槽 < a 的末端 ⇒ 违例」；
// 按 (盘, 槽, 跨度, 行号) 排好之后只比相邻两条（同盘、后一条的槽 < 前一条的末端）与它等价：有违例的一对 (a, b) 时，a 后面那一条 k 排在 b 之前或就是 b、
// 同盘，k 的槽 ≤ b 的槽 < a 的末端；反过来相邻两条违例就是一对（同 (槽, 跨度) 的两条照两两比的写法按行号分先后，跨度不为 0 就算相交）
fn judge_allocation_record_ranges() {
    // walk.rs 每块盘有记录就判一次（相邻两条都不相交那一次成立）：读到的树里有记录就算判过
    if (c_alloc_record_count > 0u) {
        judge(INVARIANT_I_5_4, true);
    }
    order_rows_by_their_first_four_words(C_ALLOC_RECORDS, C_ALLOC_RECORD_WORDS, c_alloc_record_count, C_ALLOCATION_RECORD_ROW_ORDER);
    for (var position = 1u; position < c_alloc_record_count; position = position + 1u) {
        let earlier = C_ALLOC_RECORDS + sw(C_ALLOCATION_RECORD_ROW_ORDER + position - 1u) * C_ALLOC_RECORD_WORDS;
        let later = C_ALLOC_RECORDS + sw(C_ALLOCATION_RECORD_ROW_ORDER + position) * C_ALLOC_RECORD_WORDS;
        if (sw(earlier) == sw(later)
            && lt64(vec2<u32>(sw(later + 1u), sw(later + 2u)), add64(vec2<u32>(sw(earlier + 1u), sw(earlier + 2u)), u64_of(sw(earlier + 3u))))) {
            violate(INVARIANT_I_5_4);
        }
    }
}

// I-5.4 的一版（walk.rs judge_allocation_records_disjoint 循环里的一轮）：base 里 tree_table_offset 起是树表指针，held_offset 起是
// 「树表 0 条那一版的分配记录树根」（候选根取根记录偏移 36 与 342，由记录造出来的版本取记录新根段打头与新根段偏移 274）；
// 同一条下另判一格「一条根只有一棵分配记录树」：树表读得出、条目非空（带树）的那一版，那条分配记录树根必须全零（判决 m3-formal-c655-c687-r1
// 第二节 Y1 allocation_record_tree_root 那一行）。条目宽不到 200 的表也判，所以放在跳过窄表那一句之前。读不出的那几格 CPU 报部分没判，这里不置位。
// mapping_root_offset 起是这一版的中央映射根指针（候选根取根记录偏移 256，另两类取记录新根段偏移 86）：分配记录树的节点按这一版的映射认位置
fn judge_allocation_records_disjoint_of_a_version(base: u32, tree_table_offset: u32, held_offset: u32, mapping_root_offset: u32) {
    s_use_the_central_mapping_named_at(base, mapping_root_offset);
    // 判到位照 walk.rs judge_allocation_records_disjoint：树表指针全零、树表读不出、带树而根记录那条为零的一版分配记录树没走到叶、
    // 树表 0 条而根记录持有的那棵没走到叶，都记射程里有一部分没判；带树而根记录那条为零、这一版的树走到了叶的，判一次成立
    let held = parse_node_pointer(base, held_offset);
    var this_version_walked_to_its_leaves = false;
    if (!held.all_zero && c_allocation_root_of_a_version_walked_to_its_leaves(base, held_offset)) {
        this_version_walked_to_its_leaves = true;
    }
    let tree_table_pointer = parse_node_pointer(base, tree_table_offset);
    if (tree_table_pointer.all_zero) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_4);
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_4);
        return;
    }
    let holds_its_allocation_record_tree_in_the_root_record = tt.entry_count == 0u && !held.all_zero;
    var carries_trees_and_a_zero_root_record_allocation_tree_root = false;
    if (tt.entry_count > 0u) {
        if (held.all_zero) {
            carries_trees_and_a_zero_root_record_allocation_tree_root = true;
        } else {
            violate(INVARIANT_I_5_4);
        }
    }
    if (tt.entry_width >= 200u) {
        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
            let o = entry_offset(tt, e);
            if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
                continue;
            }
            if (parse_node_pointer(tt.base, o + 14u).all_zero) {
                continue;
            }
            if (c_allocation_root_of_a_version_walked_to_its_leaves(tt.base, o + 14u)) {
                this_version_walked_to_its_leaves = true;
            }
        }
    }
    if (carries_trees_and_a_zero_root_record_allocation_tree_root) {
        if (this_version_walked_to_its_leaves) {
            judge(INVARIANT_I_5_4, true);
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_5_4);
        }
    }
    if (holds_its_allocation_record_tree_in_the_root_record && !this_version_walked_to_its_leaves) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_4);
    }
}

// I-5.4：候选集里每条根，加由记录施加出来的那几版与下一次挂载先施加的那一版（walk.rs judge_allocation_records_disjoint 的改法 RV，
// 判决 m3-formal-code-b33-r1 第二节 P2）。后两类的两条指针在记录新根段里
fn judge_allocation_records_disjoint() {
    c_seen_node_count = 0u;
    for (var word_index = 0u; word_index < 8u; word_index = word_index + 1u) {
        c_seen_node_walked_to_its_leaves[word_index] = 0u;
    }
    r_noting = false;
    // 候选集里每条根、由记录施加出来的那几版、下一次挂载先施加的那一版，次序照旧，同一个调用点；
    // 最后那一版在前两段判完之后才找
    for (var version_index = 0u; version_index <= c_candidate_count + c_version_count; version_index = version_index + 1u) {
        var base = 0u;
        var tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        var allocation_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
        var mapping_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER;
        if (version_index < c_candidate_count) {
            base = checker_root_of(sw(C_CANDIDATES + version_index)).base;
            tree_table_offset = 36u;
            allocation_offset = 342u;
            mapping_root_offset = G_ROOT_RECORD_MAPPING_ROOT_POINTER;
        } else if (version_index < c_candidate_count + c_version_count) {
            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
        } else {
            let next_version = record_row_of_the_version_the_next_mount_applies_first();
            if (next_version == NONE) {
                break;
            }
            base = checker_record_of_row(next_version).base;
        }
        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset, mapping_root_offset);
    }
}

// I-3.10 去重表 C_EXAMINED（每行 5 字：盘、槽 lo/hi、代 lo/hi）的桶：每个桶记一行的下标 + 1，0 是空桶，按 (盘, 槽, 代) 散列、撞了往后挪一个桶。
// 原来每条记录从头比到表尾，各版的记录多半已判过，条数平方长。桶数取表的两倍，表满（C_MAX_EXAMINED 行）时也至少一半是空桶，往后挪总停得下；
// 表满之后新来的记录不进表、照判（judge_allocation_generations_of_the_records），去重只省重读单元头。
// 借 crash_judge_checker_common.wgsl ⑮ 的 C_ALLOCATION_RECORD_BUCKETS 那一段：I-3.9 已释放记录的桶表在引用与快照那一截（这一趟之后的那一趟）也借它，
// 两处各自开判时从头清（原来借 C_JOURNAL_GROUP_KEYS 那 8192 个字，C_MAX_EXAMINED 随分配记录那张表抬到 8192 条时桶表 16384 个字装不下）
const C_EXAMINED_BUCKET_COUNT: u32 = 2u * C_MAX_EXAMINED;

const C_EXAMINED_BUCKETS: u32 = C_ALLOCATION_RECORD_BUCKETS;

// 借的那一段装得下：越过 C_ALLOCATION_RECORD_BUCKET_WORDS 就越过草稿区末端（SCRATCH_END）
const_assert C_EXAMINED_BUCKET_COUNT <= C_ALLOCATION_RECORD_BUCKET_WORDS;

fn c_examined_first_bucket(device: u32, slot: vec2<u32>, generation: vec2<u32>) -> u32 {
    var hash = device * 0x9E3779B1u;
    hash = (hash ^ slot.x) * 0x85EBCA77u;
    hash = (hash ^ slot.y) * 0xC2B2AE3Du;
    hash = (hash ^ generation.x) * 0x27D4EB2Fu;
    hash = (hash ^ generation.y) * 0x165667B1u;
    return (hash ^ (hash >> 15u)) & (C_EXAMINED_BUCKET_COUNT - 1u);
}

// I-3.10 的一棵树：未释放的记录（按盘、槽、代去重）的分配代等于它罩住的单元头里的诞生代号
fn judge_allocation_generations_of_the_records() {
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        if (sw(a + 6u) != 0u) {
            continue;
        }
        let device = sw(a);
        let slot = vec2<u32>(sw(a + 1u), sw(a + 2u));
        let generation = vec2<u32>(sw(a + 4u), sw(a + 5u));
        var bucket = c_examined_first_bucket(device, slot, generation);
        var examined = false;
        loop {
            let examined_row_plus_one = sw(C_EXAMINED_BUCKETS + bucket);
            if (examined_row_plus_one == 0u) {
                break;
            }
            let row = C_EXAMINED + (examined_row_plus_one - 1u) * 5u;
            if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y
                && sw(row + 3u) == generation.x && sw(row + 4u) == generation.y) {
                examined = true;
                break;
            }
            bucket = (bucket + 1u) & (C_EXAMINED_BUCKET_COUNT - 1u);
        }
        if (examined) {
            continue;
        }
        // 表满之后不再往表里记、照判（原来满了记判不了，站点 31）：judge 只置评估位与违例位，同一条记录（同盘、同槽、同代）读的是同一个单元头，
        // 判两遍结论相同；CPU judge_allocation_generations_against_unit_births 去重只为不重复判，into_report 只看评估过没有、有没有违例。
        // 表满时桶里至多一半有行，上面找的那一圈照旧停得下
        if (c_examined_count < C_MAX_EXAMINED) {
            sws(C_EXAMINED + c_examined_count * 5u, device);
            sws(C_EXAMINED + c_examined_count * 5u + 1u, slot.x);
            sws(C_EXAMINED + c_examined_count * 5u + 2u, slot.y);
            sws(C_EXAMINED + c_examined_count * 5u + 3u, generation.x);
            sws(C_EXAMINED + c_examined_count * 5u + 4u, generation.y);
            sws(C_EXAMINED_BUCKETS + bucket, c_examined_count + 1u);
            c_examined_count = c_examined_count + 1u;
        }
        // 起点槽上可用的单元头：magic、类 1 / 2 / 3、头校验和过
        let s = read_at(device, slot_to_bytes(slot), 4096u);
        if (s.version == NONE || rd32(s.base, 0u) != MAGIC_SFSU || (s.c2 & 1u) == 0u) {
            continue;
        }
        let unit_class = rd8(s.base, 6u);
        var birth = vec2<u32>(0u, 0u);
        if (unit_class == 1u) {
            birth = rd64(s.base, 75u);
        } else if (unit_class == 2u) {
            birth = rd64(s.base, 52u + 2u * rd8(s.base, 51u));
        } else if (unit_class == 3u) {
            birth = rd64(s.base, 73u);
        } else {
            continue;
        }
        judge(INVARIANT_I_3_10, eq64(generation, birth));
    }
}

// 一棵分配记录树（base 起 offset 处的根指针）：按装着的那一版中央映射认过的落点去重（walk.rs judge_allocation_generations_against_unit_births 的
// located_root：同一条过期提示在搬迁前后两版里指的是同一个单元），之后整棵读、判 I-3.10；读不全的那一棵记射程里判不了
// （walk.rs 的 ALLOCATION_GENERATIONS_OF_AN_UNREADABLE_ALLOCATION_RECORD_TREE_ARE_NOT_JUDGED：那棵树里的记录一条都没判，别的树判到了也不报成立）
fn judge_allocation_generations_of_a_tree(base: u32, offset: u32) {
    let p = parse_node_pointer(base, offset);
    if (p.all_zero || !c_seen_node_insert(located_by_the_central_mapping(p, MAPPING_CLASS_INDEX_NODE))) {
        return;
    }
    c_alloc_record_count = 0u;
    if (r_allocation_tree(base, offset)) {
        judge_allocation_generations_of_the_records();
    } else {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_10);
    }
}

// 一张树表里种类 3 那一条指着的分配记录树：去重之后整棵读、判 I-3.10
fn judge_allocation_generations_under_tree_table(tree_table_pointer: Ptr) {
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < 200u) {
        return;
    }
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
            continue;
        }
        judge_allocation_generations_of_a_tree(tt.base, o + 14u);
    }
}

// I-3.10：候选根、由记录施加出来的版本、下一次挂载会先施加的那一版。每一版的分配记录树按那一版的中央映射认位置（候选根取根记录偏移 256，
// 另两类取记录新根段偏移 86；walk.rs 的 AllocationRecordTreeOfAVersion）
fn judge_allocation_generations_against_unit_births() {
    c_seen_node_count = 0u;
    c_examined_count = 0u;
    for (var bucket = 0u; bucket < C_EXAMINED_BUCKET_COUNT; bucket = bucket + 1u) {
        sws(C_EXAMINED_BUCKETS + bucket, 0u);
    }
    r_noting = false;
    // 候选集里每条根（先判它持有的那棵分配记录树）、由记录施加出来的那几版、下一次挂载先施加的那一版（在前两段判完之后才找），
    // 次序照旧，树表那一判同一个调用点
    for (var version_index = 0u; version_index <= c_candidate_count + c_version_count; version_index = version_index + 1u) {
        var base = 0u;
        var tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        var mapping_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER;
        if (version_index < c_candidate_count) {
            let root = checker_root_of(sw(C_CANDIDATES + version_index));
            base = root.base;
            tree_table_offset = 36u;
            mapping_root_offset = G_ROOT_RECORD_MAPPING_ROOT_POINTER;
        } else if (version_index < c_candidate_count + c_version_count) {
            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
        } else {
            let next_version = record_row_of_the_version_the_next_mount_applies_first();
            if (next_version == NONE) {
                break;
            }
            base = checker_record_of_row(next_version).base;
        }
        s_use_the_central_mapping_named_at(base, mapping_root_offset);
        // 候选根先判它持有的那棵（树表 0 条那一版的分配记录树）
        if (version_index < c_candidate_count) {
            judge_allocation_generations_of_a_tree(base, 342u);
        }
        judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));
    }
}

// 下一次挂载会先施加的那一版（walk.rs version_the_next_mount_applies_first，改法 R1：判决 m3-formal-code-b35-r1 第三节第 1 条）：
// 走读内核照 I-4.1 那一份重放（replay_the_next_mount_would_do）施加的第一次发布的末条（core 施加末条那一份新根段），经交接段交来；
// 交回它在草稿区里的那一行（重放读的那一份环里，交给 checker_record_of_row 解），没有交回 NONE。core 的施加单位是一次发布：
// 断号、反向链对不上、序号或 txg 接不上、末条之后同一次发布还有记录、点名单元验不过的那次发布都不施加，这里同一份重放、同样不认；
// 一次都不施加、锚点认不定、所选根 txg 没有后继时没有这一版；
// 前缀停在后面某次发布的点名单元上时，停之前整次收下的第一次发布照认。
fn record_row_of_the_version_the_next_mount_applies_first() -> u32 {
    let replay_index = sw(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED);
    if (replay_index == NONE) {
        return NONE;
    }
    return C_REPLAY_RECORDS + replay_index * C_REPLAY_RECORD_WORDS;
}

// instance_table_rows_of_root_without_judging：一条根自己的实例表沿链读出全部行，读不出交回 false
fn r_instance_rows(record_base: u32) -> bool {
    var p = parse_node_pointer(record_base, 170u);
    var page = vec2<u32>(0u, 0u);
    var rows = 0u;
    for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {
        let s = read_unit_via_locations(p, DATA_UNIT_BYTES);
        if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
            return false;
        }
        if (rd32(s.base, 0u) != MAGIC_SFSU || rd8(s.base, 7u) != 0u || rd8(s.base, 6u) != 3u || rd16(s.base, 4u) != 1u || (s.c2 & 11u) != 11u) {
            return false;
        }
        let count = rd16(s.base, 69u);
        let width = rd16(s.base, 71u);
        let declared = rd16(s.base, 8u);
        if (!is_zero64(rd64(s.base, 43u)) || rd16(s.base, 51u) != 4u || !eq64(rd64(s.base, 53u), page) || !is_zero64(rd64(s.base, 61u))
            || width != 88u || count * width > declared || declared > DATA_UNIT_BYTES - 136u) {
            return false;
        }
        if (count == 0u) {
            return false;
        }
        for (var r = 0u; r + 1u < count; r = r + 1u) {
            let o = 136u + r * width;
            // 链指针记录之前每一条都是 kind 0 的行、偏移 21 的 flags 为 0（D18 已定项 11「第一版恒 0、非 0 拒收」；
            // walk.rs instance_table_rows_of_root_without_judging 同判，I-3.14 不成立的一片解不开）
            if (rd8(s.base, o) != 0u || rd8(s.base, o + 21u) != 0u) {
                return false;
            }
            if (rows >= C_R_MAX_ROWS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (32u << 8u);
                return false;
            }
            let txg = rd64(s.base, o + 5u);
            sws(C_R_ROWS + rows * 3u, rd32(s.base, o + 1u));
            sws(C_R_ROWS + rows * 3u + 1u, txg.x);
            sws(C_R_ROWS + rows * 3u + 2u, txg.y);
            rows = rows + 1u;
        }
        let last = 136u + (count - 1u) * width;
        // 有下一片而位置指针头部不是第一版的取值（I-3.14 第五形）同样交 2u：读不出
        let chain = w_chain_record_kind(s.base, last, width);
        if (chain == 0u) {
            c_r_row_count = rows;
            return true;
        }
        if (chain == 2u) {
            return false;
        }
        p = parse_node_pointer(s.base, last + 2u);
        page = add64(page, u64_of(1u));
    }
    undecidable = UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED;
    return false;
}

var<private> c_r_row_count: u32;

fn r_rows_abandon(instance: u32, txg: vec2<u32>) -> bool {
    for (var i = 0u; i < c_r_row_count; i = i + 1u) {
        if (sw(C_R_ROWS + i * 3u) == instance && lt64(vec2<u32>(sw(C_R_ROWS + i * 3u + 1u), sw(C_R_ROWS + i * 3u + 2u)), txg)) {
            return true;
        }
    }
    return false;
}

// user_visible_tree_root_pointers_through_the_tree_table_pointer：树表指针在 unit_base 那一份字节的 tree_table_pointer_offset 处（根记录 36；
// I-7.9 按 journal 重建出的那一版是 journal 记录新根段的树表指针，JOURNAL_NEW_ROOT_SEGMENT_OFFSET），那张树表里 inode 树与 extent 树的根指针
// （字节位置），连同这张树表（码 14、15 那几条快照树的根指针从它读，walk.rs 的 UserVisibleTreeRootPointers::snapshot_tree_roots）；判不了写 has = 0
fn c_user_visible_pointers(unit_base: u32, tree_table_pointer_offset: u32, out_row: u32) {
    sws(out_row + 1u, 0u);
    sws(out_row + 2u, NONE);
    sws(out_row + 3u, 0u);
    sws(out_row + 4u, NONE);
    sws(out_row + 5u, 0u);
    sws(out_row + 6u, 0u);
    sws(out_row + 7u, 0u);
    sws(out_row + 8u, 0u);
    sws(out_row + 9u, 0u);
    // 基是 NONE：按 journal 重建抬 F 那一刻的根环时已被盖掉的第 0 代根（c_root_in_the_ring_slot_at_the_raise）。它的树表是 mkfs 种的第 0 版
    // （0 条条目，.claude/kb/layout/01-first-txn.md 的 m2）：判得了，两棵用户可见树都没有、一个快照都没有（walk.rs UserVisibleTreeRootPointers::ABSENT；
    // 判决 research/prompts/m3-close-b19-reason-r2-main-verification.md 第二节 Q3 第 3 问「定」）
    if (unit_base == NONE) {
        sws(out_row + 1u, 1u);
        return;
    }
    let tree_table_pointer = parse_node_pointer(unit_base, tree_table_pointer_offset);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok) {
        return;
    }
    var inode_base = NONE;
    var inode_offset = 0u;
    var extent_base = NONE;
    var extent_offset = 0u;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        if (tt.entry_width < 200u) {
            return;
        }
        let o = entry_offset(tt, e);
        let kind = rd16(tt.base, o + 10u);
        if (kind == TREE_KIND_INODE) {
            if (inode_base != NONE) {
                return;
            }
            inode_base = tt.base;
            inode_offset = o + 14u;
        } else if (kind == TREE_KIND_EXTENT) {
            if (extent_base != NONE) {
                return;
            }
            extent_base = tt.base;
            extent_offset = o + 14u;
        }
    }
    sws(out_row + 1u, 1u);
    sws(out_row + 2u, inode_base);
    sws(out_row + 3u, inode_offset);
    sws(out_row + 4u, extent_base);
    sws(out_row + 5u, extent_offset);
    sws(out_row + 6u, tt.base);
    sws(out_row + 7u, tt.entries_start);
    sws(out_row + 8u, tt.entry_width);
    sws(out_row + 9u, tt.entry_count);
}

// 一行记下的树表里从第 first_entry 条起（按树表次序）的第一条快照条目（码 14、15）的下标；没有是 NONE
fn c_next_snapshot_entry(row: u32, first_entry: u32) -> u32 {
    for (var entry_index = first_entry; entry_index < sw(row + 9u); entry_index = entry_index + 1u) {
        let kind = rd16(sw(row + 6u), sw(row + 7u) + entry_index * sw(row + 8u) + 10u);
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            return entry_index;
        }
    }
    return NONE;
}

// 两条根的用户可见树根指针相同（两棵各自都没有、或 86 字节逐字相同），快照树的根指针按树表次序一串也相同（条数相同、逐条 86 字节相同）
fn c_pointers_equal(row_a: u32, row_b: u32) -> bool {
    for (var k = 0u; k < 2u; k = k + 1u) {
        let base_a = sw(row_a + 2u + k * 2u);
        let base_b = sw(row_b + 2u + k * 2u);
        if ((base_a == NONE) != (base_b == NONE)) {
            return false;
        }
        if (base_a != NONE && !bytes_equal(base_a, sw(row_a + 3u + k * 2u), base_b, sw(row_b + 3u + k * 2u), 86u)) {
            return false;
        }
    }
    var entry_a = c_next_snapshot_entry(row_a, 0u);
    var entry_b = c_next_snapshot_entry(row_b, 0u);
    // 两串一一比：每一圈各往后挪到下一条快照条目（至多树表条数那么多圈）
    for (var step = 0u; step < sw(row_a + 9u) + 1u; step = step + 1u) {
        if (entry_a == NONE || entry_b == NONE) {
            break;
        }
        let o_a = sw(row_a + 7u) + entry_a * sw(row_a + 8u) + 14u;
        let o_b = sw(row_b + 7u) + entry_b * sw(row_b + 8u) + 14u;
        if (!bytes_equal(sw(row_a + 6u), o_a, sw(row_b + 6u), o_b, 86u)) {
            return false;
        }
        entry_a = c_next_snapshot_entry(row_a, entry_a + 1u);
        entry_b = c_next_snapshot_entry(row_b, entry_b + 1u);
    }
    return entry_a == NONE && entry_b == NONE;
}

// D16（发布语义） 已定项 1 的抬 F 上限数到第几新的非空有效根（walk.rs 的 NON_EMPTY_VALID_ROOT_RANK_THE_CEILING_TAKES；I-7.9 判窗口里的根
// 够不够定住上限时用）
const C_NON_EMPTY_VALID_ROOT_RANK_THE_CEILING_TAKES: u32 = 4u;

// 最近那一次 c_rollback_floor_ceiling 数下的：非空有效根里 (txg, 实例) 不是最旧有效根的那几条去重之后计入几条（walk.rs 的
// RollbackFloorCeilingBeforeTheRaise::distinct_non_empty_states_newer_than_the_oldest_valid_root；空不空判不了的不进这个数）。
// 最旧那条按 (txg, 实例) 排在最后才选到，它前面那几条计入与否不受它影响，与 CPU 先滤掉它再去重数同一个数
var<private> c_distinct_non_empty_states_newer_than_the_oldest: u32;
