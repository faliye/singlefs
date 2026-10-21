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

var<private> c_harvested: bool;

var<private> c_exempted: array<u32, 8>;

var<private> c_deferred_per_device: array<u32, 8>;

// 已释放记录那张表（C_RELEASED）按 (盘, 槽) 的桶表，写法同 crash_judge_checker_common.wgsl 的 slot_key_first_bucket 那一段：最新根那一遍收完
// 已释放记录之后建，候选根那几遍里 r_note 按它找。借 C_JOURNAL_GROUP_KEYS 的前 C_RELEASED_BUCKET_COUNT 个字：I-1.8 在这之前用完，
// I-3.10 在这之后自己清桶
const C_RELEASED_BUCKET_COUNT: u32 = 2u * C_MAX_RELEASED;

const C_RELEASED_BUCKETS: u32 = C_JOURNAL_GROUP_KEYS;

// 借的那几段装得下（解析内核时判，不进展开量）：已释放记录的桶表与 I-1.8 的检查字在 C_JOURNAL_GROUP_KEYS 里，I-1.8 的次序在 C_ROW_ORDER 里
const_assert C_RELEASED_BUCKET_COUNT <= C_JOURNAL_GROUP_KEYS_END - C_JOURNAL_GROUP_KEYS;

const_assert C_MAX_SCANNED <= C_JOURNAL_GROUP_KEYS_END - C_JOURNAL_GROUP_KEYS;

const_assert C_MAX_SCANNED <= C_JOURNAL_GROUP_KEYS - C_ROW_ORDER;

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

fn c_append_allocation_record(device: u32, slot: vec2<u32>, span: u32, generation: vec2<u32>, released: u32) -> bool {
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
    sws(row + 7u, 0u);
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
                if (!c_append_allocation_record(rd32(n.base, o), rd48(n.base, o + 4u), span_field & 0x7FFFu, rd64(n.base, o + 12u), span_field >> 15u)) {
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
        // I-3.14 不成立的一片解不开（D18 已定项 11，用户 2026-10-06 补句）：链指针记录之前有不是 kind 0 的记录（kind 不认得、
        // 片中间有链指针记录）或行的 flags 非 0，数不全（walk.rs note_instance_table_pages_after_the_first 同判）
        for (var r = 0u; r + 1u < packed.record_count; r = r + 1u) {
            let o = packed_record_offset(packed, r);
            if (rd8(packed.base, o) != 0u || rd8(packed.base, o + 21u) != 0u) {
                r_complete = false;
                return;
            }
        }
        let last = packed_record_offset(packed, packed.record_count - 1u);
        // 有下一片而位置指针头部不是第一版的取值（I-3.14 第五形，第 10u 批）同样交 2u：这条指针不记、数不全
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
        sws(row, 0u);
        return;
    }
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        if (tt.entry_width < 200u) {
            r_complete = false;
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
    // I-9.14：候选集里的根
    judge_tree_table_birth_txg_across_the_rows(c_candidate_count, INVARIANT_I_9_14);
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
    order_rows_by_their_first_four_words(C_ALLOC_RECORDS, C_ALLOC_RECORD_WORDS, c_alloc_record_count, C_ROW_ORDER);
    for (var position = 1u; position < c_alloc_record_count; position = position + 1u) {
        let earlier = C_ALLOC_RECORDS + sw(C_ROW_ORDER + position - 1u) * C_ALLOC_RECORD_WORDS;
        let later = C_ALLOC_RECORDS + sw(C_ROW_ORDER + position) * C_ALLOC_RECORD_WORDS;
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
    // 最后那一版在前两段判完之后才找（与原先同一时点）
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
// 借 C_JOURNAL_GROUP_KEYS 那一段：那是走读内核逐盘 journal 用的；扫描内核里 I-1.8（检查字）与 I-3.9（已释放记录的桶表）在这之前借过、
// 各自用完，这里开判时从头清
const C_EXAMINED_BUCKET_COUNT: u32 = 2u * C_MAX_EXAMINED;

const C_EXAMINED_BUCKETS: u32 = C_JOURNAL_GROUP_KEYS;

// 借的那一段装得下：越过 C_JOURNAL_GROUP_KEYS_END 就压到它后面的表上
const_assert C_EXAMINED_BUCKET_COUNT <= C_JOURNAL_GROUP_KEYS_END - C_JOURNAL_GROUP_KEYS;

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
        // 有下一片而位置指针头部不是第一版的取值（I-3.14 第五形，第 10u 批）同样交 2u：读不出
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

// user_visible_tree_root_pointers：一条根树表里 inode 树与 extent 树的根指针（字节位置），连同这张树表（码 14、15 那几条快照树的根指针从它读，
// walk.rs 的 UserVisibleTreeRootPointers::snapshot_tree_roots）；判不了写 has = 0
fn c_user_visible_pointers(record_base: u32, out_row: u32) {
    sws(out_row + 1u, 0u);
    sws(out_row + 2u, NONE);
    sws(out_row + 3u, 0u);
    sws(out_row + 4u, NONE);
    sws(out_row + 5u, 0u);
    sws(out_row + 6u, 0u);
    sws(out_row + 7u, 0u);
    sws(out_row + 8u, 0u);
    sws(out_row + 9u, 0u);
    let tree_table_pointer = parse_node_pointer(record_base, 36u);
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

// newest_first_distinct_state_txgs 与 rollback_floor_ceiling_from 合在一起：按 (txg, 实例) 从新到旧计入，交回上限
fn c_rollback_floor_ceiling(non_empty_count: u32, undeterminable_count: u32, newest_on_every_device: vec2<u32>, oldest: vec2<u32>) -> vec2<u32> {
    var counted = 0u;
    var counted_states = 0u;
    let total = non_empty_count + undeterminable_count;
    for (var i = 0u; i < total; i = i + 1u) {
        sws(C_PICKED + i, 0u);
    }
    for (var round = 0u; round < total; round = round + 1u) {
        // 选还没选过的里 (txg, 实例) 最大的
        var best = NONE;
        var best_txg = vec2<u32>(0u, 0u);
        var best_instance = 0u;
        for (var i = 0u; i < total; i = i + 1u) {
            if (sw(C_PICKED + i) != 0u) {
                continue;
            }
            var txg = vec2<u32>(0u, 0u);
            var instance = 0u;
            if (i < non_empty_count) {
                txg = vec2<u32>(sw(C_NON_EMPTY + i * 4u), sw(C_NON_EMPTY + i * 4u + 1u));
                instance = sw(C_NON_EMPTY + i * 4u + 2u);
            } else {
                let u = i - non_empty_count;
                txg = vec2<u32>(sw(C_UNDETERMINABLE + u * 3u), sw(C_UNDETERMINABLE + u * 3u + 1u));
                instance = sw(C_UNDETERMINABLE + u * 3u + 2u);
            }
            if (best == NONE || root_key_less(best_txg, best_instance, txg, instance)) {
                best = i;
                best_txg = txg;
                best_instance = instance;
            }
        }
        sws(C_PICKED + best, 1u);
        if (best >= non_empty_count) {
            sws(C_COUNTED + counted * 2u, best_txg.x);
            sws(C_COUNTED + counted * 2u + 1u, best_txg.y);
            counted = counted + 1u;
            continue;
        }
        let state_row = C_VALID_BEFORE + sw(C_NON_EMPTY + best * 4u + 3u) * C_VALID_BEFORE_WORDS;
        var seen_state = false;
        for (var k = 0u; k < counted_states; k = k + 1u) {
            if (c_pointers_equal(state_row, C_VALID_BEFORE + sw(C_COUNTED_STATES + k) * C_VALID_BEFORE_WORDS)) {
                seen_state = true;
            }
        }
        if (!seen_state) {
            sws(C_COUNTED_STATES + counted_states, sw(C_NON_EMPTY + best * 4u + 3u));
            counted_states = counted_states + 1u;
            sws(C_COUNTED + counted * 2u, best_txg.x);
            sws(C_COUNTED + counted * 2u + 1u, best_txg.y);
            counted = counted + 1u;
        }
    }
    var fourth = oldest;
    if (counted >= 4u) {
        fourth = vec2<u32>(sw(C_COUNTED + 6u), sw(C_COUNTED + 7u));
    }
    if (lt64(newest_on_every_device, fourth)) {
        return newest_on_every_device;
    }
    return fourth;
}

// 一圈窗口（walk.rs 的 OneRevolutionWindow::contains）：txg > 锚 − 3S，写成 txg + 3S > 锚（锚 < 3S 时整个环都在窗口里，不减出负数）
fn in_the_one_revolution_window(txg: vec2<u32>, anchor_txg: vec2<u32>, root_ring_slot_count: u32) -> bool {
    return lt64(anchor_txg, add64_saturating(txg, u64_of(root_ring_slot_count)));
}

// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限。两支的「之前的根」都只取 txg 比它小、落在锚在这张镜像根环里最新那条根的 txg 上的
// 一圈窗口里的根（候选过滤那一个；CPU 一侧 walk.rs 同一个口径）。锚在被判那条根自己的 txg 上的那一个窗口不取：最新那条根的 txg 不小于
// 根表里每一条根的（walk_and_hand_off 择 (txg, 实例) 最大的），锚在它上面的窗口本来就落在另一个里面，两个的交就是它，拿掉不改任何判定
fn judge_rollback_floor_raises_against_their_ceilings() {
    let device_count = hdr(3u);
    let root_ring_slot_count = sw(H_BASE + H_REGIONS) * sw(H_BASE + H_SLOTS_PER_REGION);
    let newest_txg_of_the_image = checker_root_of(c_newest_index).txg;
    for (var i = 0u; i < root_count; i = i + 1u) {
        let raising = checker_root_of(i);
        // 同一实例里 txg 比它小的最新那条根
        var have_previous = false;
        var previous_txg = vec2<u32>(0u, 0u);
        var floor_before = vec2<u32>(0u, 0u);
        for (var k = 0u; k < root_count; k = k + 1u) {
            let root = checker_root_of(k);
            if (root.instance == raising.instance && lt64(root.txg, raising.txg) && (!have_previous || le64(previous_txg, root.txg))) {
                previous_txg = root.txg;
                floor_before = root.floor;
                have_previous = true;
            }
        }
        if (!have_previous || le64(raising.floor, floor_before)) {
            continue;
        }
        if (raising.unmount) {
            // 带卸载记号那一支：F 不高于窗口里 txg 比它小的最新那条根的 txg；窗口里一条都没有 ⇒ 这条根不判
            var have_newest_before = false;
            var newest_before = vec2<u32>(0u, 0u);
            for (var k = 0u; k < root_count; k = k + 1u) {
                let txg = checker_root_of(k).txg;
                if (lt64(txg, raising.txg)
                    && in_the_one_revolution_window(txg, newest_txg_of_the_image, root_ring_slot_count)
                    && (!have_newest_before || lt64(newest_before, txg))) {
                    newest_before = txg;
                    have_newest_before = true;
                }
            }
            if (have_newest_before) {
                judge(INVARIANT_I_7_9, le64(raising.floor, newest_before));
            }
            continue;
        }
        if (!r_instance_rows(raising.base)) {
            continue;
        }
        // 它之前的有效根：txg < 它的、在一圈窗口里、≥ 抬之前的 F、按它自己的实例表不被抛弃；按 (txg, 实例) 升序
        var valid_count = 0u;
        for (var k = 0u; k < root_count; k = k + 1u) {
            let root = checker_root_of(k);
            if (!lt64(root.txg, raising.txg)
                || !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)
                || lt64(root.txg, floor_before) || r_rows_abandon(root.instance, root.txg)) {
                continue;
            }
            var position = 0u;
            while (position < valid_count) {
                let existing = checker_root_of(sw(C_VALID_BEFORE + position * C_VALID_BEFORE_WORDS));
                if (root_key_less(root.txg, root.instance, existing.txg, existing.instance)) {
                    break;
                }
                position = position + 1u;
            }
            var j = valid_count;
            while (j > position) {
                sws(C_VALID_BEFORE + j * C_VALID_BEFORE_WORDS, sw(C_VALID_BEFORE + (j - 1u) * C_VALID_BEFORE_WORDS));
                j = j - 1u;
            }
            sws(C_VALID_BEFORE + position * C_VALID_BEFORE_WORDS, k);
            valid_count = valid_count + 1u;
        }
        if (valid_count == 0u) {
            continue;
        }
        let oldest = checker_root_of(sw(C_VALID_BEFORE)).txg;
        var have_newest_on_every = false;
        var newest_on_every = vec2<u32>(0u, 0u);
        for (var d = 0u; d < device_count; d = d + 1u) {
            var have = false;
            var newest_on_this = vec2<u32>(0u, 0u);
            for (var v = 0u; v < valid_count; v = v + 1u) {
                let root_index = sw(C_VALID_BEFORE + v * C_VALID_BEFORE_WORDS);
                if (handoff_region_device(checker_root_region(root_index)) != hdr(12u + d)) {
                    continue;
                }
                let txg = checker_root_of(root_index).txg;
                if (!have || lt64(newest_on_this, txg)) {
                    newest_on_this = txg;
                    have = true;
                }
            }
            if (have && (!have_newest_on_every || lt64(newest_on_this, newest_on_every))) {
                newest_on_every = newest_on_this;
                have_newest_on_every = true;
            }
        }
        // 非空：跟前一条有效根比两棵用户可见树的根指针；最旧的跟「两棵都没有」比
        var non_empty_count = 0u;
        var undeterminable_count = 0u;
        var previous_row = NONE;      // NONE 表示 ABSENT（两棵都没有、判得了）
        var previous_determinable = true;
        for (var v = 0u; v < valid_count; v = v + 1u) {
            let row = C_VALID_BEFORE + v * C_VALID_BEFORE_WORDS;
            let root = checker_root_of(sw(row));
            c_user_visible_pointers(root.base, row);
            let determinable = sw(row + 1u) == 1u;
            if (determinable && previous_determinable) {
                var differs = false;
                if (previous_row == NONE) {
                    // 跟「两棵都没有、一个快照都没有」比（walk.rs 的 UserVisibleTreeRootPointers::ABSENT）
                    differs = sw(row + 2u) != NONE || sw(row + 4u) != NONE || c_next_snapshot_entry(row, 0u) != NONE;
                } else {
                    differs = !c_pointers_equal(row, previous_row);
                }
                if (differs) {
                    sws(C_NON_EMPTY + non_empty_count * 4u, root.txg.x);
                    sws(C_NON_EMPTY + non_empty_count * 4u + 1u, root.txg.y);
                    sws(C_NON_EMPTY + non_empty_count * 4u + 2u, root.instance);
                    sws(C_NON_EMPTY + non_empty_count * 4u + 3u, v);
                    non_empty_count = non_empty_count + 1u;
                }
            } else {
                sws(C_UNDETERMINABLE + undeterminable_count * 3u, root.txg.x);
                sws(C_UNDETERMINABLE + undeterminable_count * 3u + 1u, root.txg.y);
                sws(C_UNDETERMINABLE + undeterminable_count * 3u + 2u, root.instance);
                undeterminable_count = undeterminable_count + 1u;
            }
            previous_row = row;
            previous_determinable = determinable;
        }
        let lowest = c_rollback_floor_ceiling(non_empty_count, 0u, newest_on_every, oldest);
        let highest = c_rollback_floor_ceiling(non_empty_count, undeterminable_count, newest_on_every, oldest);
        if (lt64(lowest, raising.floor) && le64(raising.floor, highest)) {
            continue;
        }
        judge(INVARIANT_I_7_9, !lt64(lowest, raising.floor));
    }
}

// 位置项里指着 (盘, 槽) 的校验和：走过的版本收到的，查不到时把根环里没走过的根整遍走一次补进来（只补这一次）
fn c_location_entry_checksum_known(device: u32, slot: vec2<u32>, fsid_low: vec2<u32>) -> bool {
    for (var i = 0u; i < w_checksum_count; i = i + 1u) {
        if (sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS) == device && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 2u) == slot.y) {
            return true;
        }
    }
    // 根环里没走过的根那一遍走读内核已经走过（`hand_off_to_the_scan_kernel`）、位置项校验和在 C_HARVEST；
    // 与 CPU 同一个懒法：第一次查不到才开始用它
    if (!c_harvested) {
        c_harvested = true;
    }
    for (var i = 0u; i < w_harvest_count; i = i + 1u) {
        if (sw(C_HARVEST + i * C_CHECKSUM_WORDS) == device && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 2u) == slot.y) {
            return true;
        }
    }
    return false;
}

fn c_location_entry_checksum_contains(device: u32, slot: vec2<u32>, checksum: u32) -> bool {
    for (var i = 0u; i < w_checksum_count; i = i + 1u) {
        if (sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS) == device && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 2u) == slot.y
            && sw(C_CHECKSUMS + i * C_CHECKSUM_WORDS + 3u) == checksum) {
            return true;
        }
    }
    if (!c_harvested) {
        return false;
    }
    for (var i = 0u; i < w_harvest_count; i = i + 1u) {
        if (sw(C_HARVEST + i * C_CHECKSUM_WORDS) == device && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 1u) == slot.x && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 2u) == slot.y
            && sw(C_HARVEST + i * C_CHECKSUM_WORDS + 3u) == checksum) {
            return true;
        }
    }
    return false;
}

// 隔离的记录那一份读回来对不对：读得出、整单元 CRC 在指着它的位置项里；没有位置项指着它时退回自证（check_unit）
fn c_copy_matches_its_location_entry(device: u32, slot: vec2<u32>, span: u32, fsid_low: vec2<u32>) -> bool {
    let s = read_at(device, slot_to_bytes(slot), span << SLOT_SHIFT);
    if (s.version == NONE) {
        return false;
    }
    var checksum = s.c0;
    if (span == 2u) {
        checksum = s.c1;
    }
    if (c_location_entry_checksum_known(device, slot, fsid_low)) {
        return c_location_entry_checksum_contains(device, slot, checksum);
    }
    // check_unit：magic、flags、类 1 / 2 / 3、长度按类、头校验和、载荷 CRC、版本、预留位
    if (rd32(s.base, 0u) != MAGIC_SFSU || rd8(s.base, 7u) != 0u || rd16(s.base, 4u) != 1u) {
        return false;
    }
    let unit_class = rd8(s.base, 6u);
    var expected_span = 2u;
    if (unit_class == 2u) {
        expected_span = 1u;
    } else if (unit_class != 1u && unit_class != 3u) {
        return false;
    }
    if (span != expected_span) {
        return false;
    }
    return (s.c2 & 11u) == 11u;
}

// C_GROUPS 一行第四个字（盘位掩码）里这一位：这一组整组豁免了（I-3.16 收占着的槽读它；盘位只占低 MAX_DEVICES 位）
const C_GROUP_EXEMPTED_AS_QUARANTINED: u32 = 0x80000000u;

// 最近那一次 c_quarantine_exempted 记进 C_GROUPS 的组数
var<private> c_quarantine_group_count: u32;

// quarantined_slots_exempted_per_device：最新根的分配记录树里未释放、没有根引用起在它那一槽的记录，按单元归组；有一份读回来对不上，整组豁免
// （整组豁免的那一行掩码字带上 C_GROUP_EXEMPTED_AS_QUARANTINED：walk.rs 交回的 slots 就是这几组每块盘上罩住的槽）
fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, fsid_low: vec2<u32>) {
    let device_count = hdr(3u);
    for (var d = 0u; d < 8u; d = d + 1u) {
        c_exempted[d] = 0u;
    }
    c_quarantine_group_count = 0u;
    let tree_table_pointer = parse_node_pointer(newest_base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < 200u) {
        return;
    }
    // 分配记录树的节点按最新根自己的中央映射认位置（walk.rs quarantined_slots_exempted_per_device 的 central_mapping）
    s_use_the_central_mapping_named_at(newest_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    var groups = 0u;
    r_noting = false;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION || parse_node_pointer(tt.base, o + 14u).all_zero) {
            continue;
        }
        c_alloc_record_count = 0u;
        if (!r_allocation_tree(tt.base, o + 14u)) {
            continue;
        }
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            let device = sw(a);
            let slot = vec2<u32>(sw(a + 1u), sw(a + 2u));
            let span = sw(a + 3u);
            if (sw(a + 6u) != 0u || c_referenced_start(device, slot, reference_limit)) {
                continue;
            }
            var device_index = NONE;
            for (var d = 0u; d < device_count; d = d + 1u) {
                if (hdr(12u + d) == device) {
                    device_index = d;
                }
            }
            if (device_index == NONE) {
                continue;
            }
            var found = false;
            for (var k = 0u; k < groups; k = k + 1u) {
                let group = C_GROUPS + k * 4u;
                if (sw(group) == slot.x && sw(group + 1u) == slot.y && sw(group + 2u) == span) {
                    sws(group + 3u, sw(group + 3u) | (1u << device_index));
                    found = true;
                }
            }
            if (!found) {
                if (groups >= C_MAX_GROUPS) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (33u << 8u);
                    return;
                }
                sws(C_GROUPS + groups * 4u, slot.x);
                sws(C_GROUPS + groups * 4u + 1u, slot.y);
                sws(C_GROUPS + groups * 4u + 2u, span);
                sws(C_GROUPS + groups * 4u + 3u, 1u << device_index);
                groups = groups + 1u;
            }
        }
    }
    for (var k = 0u; k < groups; k = k + 1u) {
        let group = C_GROUPS + k * 4u;
        let slot = vec2<u32>(sw(group), sw(group + 1u));
        let span = sw(group + 2u);
        let mask = sw(group + 3u);
        var some_copy_fails = false;
        for (var d = 0u; d < device_count; d = d + 1u) {
            if ((mask & (1u << d)) != 0u && !c_copy_matches_its_location_entry(hdr(12u + d), slot, span, fsid_low)) {
                some_copy_fails = true;
            }
        }
        if (some_copy_fails) {
            for (var d = 0u; d < device_count; d = d + 1u) {
                if ((mask & (1u << d)) != 0u) {
                    c_exempted[d] = c_exempted[d] + span;
                }
            }
            sws(group + 3u, mask | C_GROUP_EXEMPTED_AS_QUARANTINED);
        }
    }
    c_quarantine_group_count = groups;
}

// c_deferred_below_the_floor 记进 C_DEFERRED 的行数（I-3.16 收占着的槽读它）
var<private> c_deferred_count: u32;

// deferred_slots_referenced_only_below_the_floor_per_device
fn c_deferred_below_the_floor(newest_base: u32) {
    let device_count = hdr(3u);
    for (var d = 0u; d < 8u; d = d + 1u) {
        c_deferred_per_device[d] = 0u;
    }
    c_deferred_count = 0u;
    let tree_table_pointer = parse_node_pointer(newest_base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < 200u) {
        return;
    }
    // 分配记录树的节点按最新根自己的中央映射认位置（walk.rs deferred_slots_referenced_only_below_the_floor_per_device 的 central_mapping）
    s_use_the_central_mapping_named_at(newest_base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    var released_above = 0u;
    r_noting = false;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION || parse_node_pointer(tt.base, o + 14u).all_zero) {
            continue;
        }
        c_alloc_record_count = 0u;
        if (!r_allocation_tree(tt.base, o + 14u)) {
            continue;
        }
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            if (sw(a + 6u) == 0u || !lt64(c_effective_floor, vec2<u32>(sw(a + 4u), sw(a + 5u)))) {
                continue;
            }
            if (released_above >= C_MAX_RELEASED_ABOVE) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (34u << 8u);
                return;
            }
            let released_above_row = C_RELEASED_ABOVE + released_above * C_RELEASED_ABOVE_WORDS;
            sws(released_above_row, sw(a));
            sws(released_above_row + 1u, sw(a + 1u));
            sws(released_above_row + 2u, sw(a + 2u));
            sws(released_above_row + 3u, sw(a + 3u));
            released_above = released_above + 1u;
        }
    }
    if (released_above == 0u) {
        return;
    }
    // F 之下、没被抛弃的根引用的、候选版本没走到的槽（走读内核交过来的清单，已去重）里，落在释放代高于 F 的已释放记录罩住的槽上的
    var deferred = 0u;
    let below_floor_slot_count = sw(H_BASE + H_BELOW_FLOOR_SLOT_COUNT);
    for (var i = 0u; i < below_floor_slot_count; i = i + 1u) {
        let device = sw(H_BELOW_FLOOR_SLOTS + i * 3u);
        let slot = vec2<u32>(sw(H_BELOW_FLOOR_SLOTS + i * 3u + 1u), sw(H_BELOW_FLOOR_SLOTS + i * 3u + 2u));
        var in_released = false;
        for (var q = 0u; q < released_above; q = q + 1u) {
            let record = C_RELEASED_ABOVE + q * C_RELEASED_ABOVE_WORDS;
            if (sw(record) != device) {
                continue;
            }
            let record_slot = vec2<u32>(sw(record + 1u), sw(record + 2u));
            if (le64(record_slot, slot) && lt64(slot, add64(record_slot, u64_of(sw(record + 3u))))) {
                in_released = true;
            }
        }
        if (!in_released) {
            continue;
        }
        if (deferred >= C_MAX_DEFERRED) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (35u << 8u);
            return;
        }
        sws(C_DEFERRED + deferred * 3u, device);
        sws(C_DEFERRED + deferred * 3u + 1u, slot.x);
        sws(C_DEFERRED + deferred * 3u + 2u, slot.y);
        deferred = deferred + 1u;
    }
    c_deferred_count = deferred;
    for (var q = 0u; q < deferred; q = q + 1u) {
        for (var d = 0u; d < device_count; d = d + 1u) {
            if (hdr(12u + d) == sw(C_DEFERRED + q * 3u)) {
                c_deferred_per_device[d] = c_deferred_per_device[d] + 1u;
            }
        }
    }
}

// 记账行 (统计量, 盘)：找到写进 value 交回 true
fn c_accounting_row(statistic: u32, device: u32, value: ptr<function, vec2<u32>>) -> bool {
    for (var k = 0u; k < w_accounting_row_count; k = k + 1u) {
        if (sw(C_ACCOUNTING_ROWS + k * 4u) == statistic && sw(C_ACCOUNTING_ROWS + k * 4u + 1u) == device) {
            *value = vec2<u32>(sw(C_ACCOUNTING_ROWS + k * 4u + 2u), sw(C_ACCOUNTING_ROWS + k * 4u + 3u));
            return true;
        }
    }
    return false;
}

// 走读引用的槽数逐盘相加（前 limit 条引用）
fn c_slots_referenced(device: u32, limit: u32) -> vec2<u32> {
    var total = vec2<u32>(0u, 0u);
    for (var i = 0u; i < limit; i = i + 1u) {
        let row = C_REFERENCES0 + i * C_REFERENCE_WORDS;
        if (sw(row) == device) {
            total = add64(total, u64_of(sw(row + 3u)));
        }
    }
    return total;
}

// ── 一版的树表与分配记录，只读不判（walk.rs 的 tree_table_entries_of_a_version_without_judging、allocation_records_of_a_version_without_judging）：
// I-3.16 读最新那一版的，引用与快照那一截的 I-5.6、I-3.15 读版本清单里每一版的

// 一版的树表：树表指针全零、读不出、有条目比字段表短（树表条目 200 字节）的交回 bad_node（那一版判不了）
fn g_tree_table_of_the_version(base: u32, tree_table_offset: u32) -> Node {
    let pointer = parse_node_pointer(base, tree_table_offset);
    if (pointer.all_zero) {
        return bad_node();
    }
    let tt = r_read_index_node(pointer);
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < 200u)) {
        return bad_node();
    }
    return tt;
}

// 一版的分配记录读到了几棵分配记录树（walk.rs 的 AllocationRecordsOfAVersion::allocation_record_trees_read）：g_allocation_records_of_the_version
// 每次从 0 数，读一棵非全零的加一。0 说明这一版的账一棵树都没有，记录表空不等于「没有未释放的记录」（树表缺种类 3 那一条的坏镜像，I-9.16 判它）
var<private> g_allocation_record_trees_read: u32;

// 一版的分配记录记进 C_ALLOC_RECORDS（c_alloc_record_count 从 0 数）：根记录（或记录新根段）直接持有的那一棵（base 里 held_offset 起那条指针，
// 树表 0 条的那一版才有），再是树表 tt 里种类 3 的每一条，非全零的整棵读、记进 g_allocation_record_trees_read；有一棵读不全交回 false。
// 两样同一个 r_allocation_tree 调用点：第 0 步是根记录持有的那一棵，第 k 步是树表第 k − 1 条（GPU 程序把每个调用点展开成一份拷贝）
fn g_allocation_records_of_the_version(base: u32, held_offset: u32, tt: Node) -> bool {
    c_alloc_record_count = 0u;
    g_allocation_record_trees_read = 0u;
    r_noting = false;
    for (var step = 0u; step <= tt.entry_count; step = step + 1u) {
        var tree_base = base;
        var tree_offset = held_offset;
        if (step > 0u) {
            let o = entry_offset(tt, step - 1u);
            if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
                continue;
            }
            tree_base = tt.base;
            tree_offset = o + 14u;
        }
        if (parse_node_pointer(tree_base, tree_offset).all_zero) {
            continue;
        }
        if (!r_allocation_tree(tree_base, tree_offset)) {
            return false;
        }
        g_allocation_record_trees_read = g_allocation_record_trees_read + 1u;
    }
    return true;
}

// ── I-3.16（walk.rs judge_fragmentation_and_empty_segment_statistics，N5 = C413）：最新根记账里每块盘的 runs（第 10 项）与全空聚簇段数（第 11 项）
// 等于全扫重算的值。占着的槽是 I-3.1 那一份并集：全部走过的版本的引用（C_REFERENCES0 前 w_reference_count[0] 条）、第一遍隔离豁免的那几组
// （每块盘上罩住的槽）、F 之下仍在 defer 的槽（C_DEFERRED）。三样记成区间（盘、槽 lo/hi、跨度）排一次序、逐盘按序走一遍：
// 那块盘上占着的槽数（区间并起来的槽数，单元区之外的也算）乘 16 KiB 不等于记账的已分配时这块盘不判（认不出占着哪几槽）；单元区 [起始槽号, 盘末) 里
// 空闲槽的连续段数（walk.rs free_runs_between_the_occupied_slots：区间按起点升序，起点越过「下一个可能空闲的槽」就多一段）比第 10 项；
// 全空段只按记录口径（最新根未释放的分配记录罩住的整段之外的整段数，D3（空间分配） 已定项 10 ①，C372 按记录口径收口）数、比第 11 项，
// 不经占着的槽那一份并集、也不与按占用口径数的比（walk.rs 改成这样之后的写法）；最新根的分配记录读不全、或那一版一棵分配记录树都没有
// （树表里没有种类 3 那一条、根记录也不持有，I-9.16 判它）时第 11 项报部分没判。区间表借引用与快照那一截的 S_KEYED_UNITS 起那一段
// （扫描内核在引用与快照那一截之前跑、那一截从头写它）

const C_OCCUPIED_WORDS: u32 = 4u;            // 盘、槽 lo/hi、跨度

// 区间至多几条：引用集合 C_MAX_REFERENCES、隔离豁免的组 C_MAX_GROUPS 乘每组的盘、F 之下的推迟 C_MAX_DEFERRED，合法镜像上远不到
const C_MAX_OCCUPIED: u32 = 16384u;

const C_OCCUPIED: u32 = S_KEYED_UNITS;

const C_OCCUPIED_ORDER: u32 = C_OCCUPIED + C_OCCUPIED_WORDS * C_MAX_OCCUPIED;

const_assert C_OCCUPIED_ORDER + C_MAX_OCCUPIED <= SCRATCH_END;

// 区间表满了（「条目太多」一类，站点号与别处不撞）
const C_SITE_OCCUPIED_INTERVALS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (100u << 8u);

// 第 10、11 项的统计量标签（walk.rs 的 STATISTIC_FRAGMENTATION_RUNS、STATISTIC_EMPTY_CLUSTER_SEGMENTS）与已分配（STATISTIC_ALLOCATED_BYTES）
const STATISTIC_ALLOCATED_BYTES: u32 = 1u;
const STATISTIC_FRAGMENTATION_RUNS: u32 = 10u;
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u32 = 11u;

// 一个聚簇段几个槽（singlefs_format::CLUSTER_SEGMENT_SLOTS）与它的位数
const CLUSTER_SEGMENT_SLOTS: u32 = 64u;
const CLUSTER_SEGMENT_SLOT_SHIFT: u32 = 6u;

var<private> c_occupied_count: u32;

fn c_note_an_occupied_interval(device: u32, slot: vec2<u32>, span: u32) {
    if (c_occupied_count >= C_MAX_OCCUPIED) {
        undecidable = C_SITE_OCCUPIED_INTERVALS_PAST_THE_TABLE;
        return;
    }
    let row = C_OCCUPIED + c_occupied_count * C_OCCUPIED_WORDS;
    sws(row, device);
    sws(row + 1u, slot.x);
    sws(row + 2u, slot.y);
    sws(row + 3u, span);
    c_occupied_count = c_occupied_count + 1u;
}

// 第一遍隔离豁免整组豁免的那几组：组里每块盘一条区间
fn c_note_the_quarantined_groups_as_occupied() {
    let device_count = hdr(3u);
    for (var k = 0u; k < c_quarantine_group_count; k = k + 1u) {
        let group = C_GROUPS + k * 4u;
        let mask = sw(group + 3u);
        if ((mask & C_GROUP_EXEMPTED_AS_QUARANTINED) == 0u) {
            continue;
        }
        for (var d = 0u; d < device_count; d = d + 1u) {
            if ((mask & (1u << d)) != 0u) {
                c_note_an_occupied_interval(hdr(12u + d), vec2<u32>(sw(group), sw(group + 1u)), sw(group + 2u));
            }
        }
    }
}

// 一块盘上按起点升序走过的几段 [起点, 终点)（单元区里、非空）罩住的段号（(槽 − 单元区起始槽号) / 64，只数小于 segment_limit 的）：
// counted 是数过的段数，next_uncounted 是数过的最大段号 + 1（一段都没数过是 0）。起点升序时，此前各段与 [这一段的起点所在段, ∞) 的交恰是
// [起点所在段, next_uncounted)：之前每一段起点都不大于它，罩到它之后的那一段就连着罩到自己的终点
struct SegmentTally {
    counted: vec2<u32>,
    next_uncounted: vec2<u32>,
};

fn c_tally_the_segments_of_an_interval(tally: SegmentTally, start: vec2<u32>, end: vec2<u32>, unit_area_start: vec2<u32>,
    segment_limit: vec2<u32>) -> SegmentTally {
    var after = tally;
    let first_segment = shr64(sub64(start, unit_area_start), CLUSTER_SEGMENT_SLOT_SHIFT);
    var last_segment = shr64(sub64(sub64(end, u64_of(1u)), unit_area_start), CLUSTER_SEGMENT_SLOT_SHIFT);
    if (is_zero64(segment_limit)) {
        return after;
    }
    let highest_segment = sub64(segment_limit, u64_of(1u));
    if (lt64(highest_segment, last_segment)) {
        last_segment = highest_segment;
    }
    let first_uncounted_segment = select(first_segment, after.next_uncounted, lt64(first_segment, after.next_uncounted));
    if (le64(first_uncounted_segment, last_segment)) {
        after.counted = add64(after.counted, add64(sub64(last_segment, first_uncounted_segment), u64_of(1u)));
        after.next_uncounted = add64(last_segment, u64_of(1u));
    }
    return after;
}

fn judge_fragmentation_and_empty_segment_statistics() {
    for (var i = 0u; i < w_reference_count[0]; i = i + 1u) {
        let row = C_REFERENCES0 + i * C_REFERENCE_WORDS;
        c_note_an_occupied_interval(sw(row), vec2<u32>(sw(row + 1u), sw(row + 2u)), sw(row + 3u));
    }
    for (var q = 0u; q < c_deferred_count; q = q + 1u) {
        c_note_an_occupied_interval(sw(C_DEFERRED + q * 3u), vec2<u32>(sw(C_DEFERRED + q * 3u + 1u), sw(C_DEFERRED + q * 3u + 2u)), 1u);
    }
    if (undecidable != 0u) {
        return;
    }
    // 最新那一版的分配记录（按记录口径数全空段）：树表或哪一棵读不全，记录口径那一半数不出
    let newest_root = checker_root_of(c_newest_index);
    let newest_tree_table = g_tree_table_of_the_version(newest_root.base, 36u);
    var newest_records_read_in_full = false;
    if (newest_tree_table.ok) {
        // 分配记录树的节点按最新根自己的中央映射认位置（walk.rs allocation_records_of_a_version_without_judging 的 central_mapping）
        s_use_the_central_mapping_named_at(newest_root.base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
        newest_records_read_in_full = g_allocation_records_of_the_version(newest_root.base, 342u, newest_tree_table);
    }
    let newest_allocation_record_trees_read = g_allocation_record_trees_read;
    // 区间与记录各排一次序（同一个调用点：第 0 遍排区间、第 1 遍排记录）
    for (var table = 0u; table < 2u; table = table + 1u) {
        if (table == 1u && !newest_records_read_in_full) {
            break;
        }
        order_rows_by_their_first_four_words(select(C_OCCUPIED, C_ALLOC_RECORDS, table == 1u), select(C_OCCUPIED_WORDS, C_ALLOC_RECORD_WORDS, table == 1u),
            select(c_occupied_count, c_alloc_record_count, table == 1u), select(C_OCCUPIED_ORDER, C_ROW_ORDER, table == 1u));
    }
    let unit_area_start = handoff_unit_area_start_slot();
    let device_end_slot = shr64(device_bytes(), SLOT_SHIFT);
    let device_count = hdr(3u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var allocated = vec2<u32>(0u, 0u);
        if (!c_accounting_row(STATISTIC_ALLOCATED_BYTES, device, &allocated)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        let unit_area_has_slots = lt64(unit_area_start, device_end_slot);
        let unit_area_slots = select(u64_of(0u), sub64(device_end_slot, unit_area_start), unit_area_has_slots);
        let full_segments = shr64(unit_area_slots, CLUSTER_SEGMENT_SLOT_SHIFT);
        // 占着的槽：整块盘上并起来的槽数；单元区里的空闲连续段数
        var occupied_slots = vec2<u32>(0u, 0u);
        var covered_end = vec2<u32>(0u, 0u);
        var next_free_candidate = unit_area_start;
        var runs = vec2<u32>(0u, 0u);
        for (var position = 0u; position < c_occupied_count; position = position + 1u) {
            let row = C_OCCUPIED + sw(C_OCCUPIED_ORDER + position) * C_OCCUPIED_WORDS;
            if (sw(row) != device) {
                continue;
            }
            let start = vec2<u32>(sw(row + 1u), sw(row + 2u));
            let end = add64(start, u64_of(sw(row + 3u)));
            let counted_from = select(start, covered_end, lt64(start, covered_end));
            if (lt64(counted_from, end)) {
                occupied_slots = add64(occupied_slots, sub64(end, counted_from));
                covered_end = end;
            }
            let in_area_start = select(start, unit_area_start, lt64(start, unit_area_start));
            let in_area_end = select(end, device_end_slot, lt64(device_end_slot, end));
            if (!lt64(in_area_start, in_area_end)) {
                continue;
            }
            if (lt64(next_free_candidate, in_area_start)) {
                runs = add64(runs, u64_of(1u));
            }
            if (lt64(next_free_candidate, in_area_end)) {
                next_free_candidate = in_area_end;
            }
        }
        if (lt64(next_free_candidate, device_end_slot)) {
            runs = add64(runs, u64_of(1u));
        }
        // 槽数乘 16 KiB 溢出 64 位（槽数 ≥ 2⁵⁰）时与记账对不上，同 walk.rs 的 checked_mul
        if (occupied_slots.y >= (1u << (32u - SLOT_SHIFT)) || !eq64(shl64(occupied_slots, SLOT_SHIFT), allocated)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        if (!unit_area_has_slots) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        var recorded = vec2<u32>(0u, 0u);
        if (c_accounting_row(STATISTIC_FRAGMENTATION_RUNS, device, &recorded)) {
            judge(INVARIANT_I_3_16, eq64(recorded, runs));
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
        }
        if (!newest_records_read_in_full) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        // 最新根那一版一棵分配记录树都没有：记录表空不等于没有未释放的记录，按记录口径数不出（walk.rs 同一句守）
        if (newest_allocation_record_trees_read == 0u) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
            continue;
        }
        // 按记录口径：这块盘上未释放的记录罩住的整段（段号 < 整段数）
        var segments_holding_a_record: SegmentTally;
        segments_holding_a_record.counted = vec2<u32>(0u, 0u);
        segments_holding_a_record.next_uncounted = vec2<u32>(0u, 0u);
        for (var position = 0u; position < c_alloc_record_count; position = position + 1u) {
            let record = C_ALLOC_RECORDS + sw(C_ROW_ORDER + position) * C_ALLOC_RECORD_WORDS;
            if (sw(record) != device || sw(record + 6u) != 0u) {
                continue;
            }
            let start = vec2<u32>(sw(record + 1u), sw(record + 2u));
            let end = add64_saturating(start, u64_of(sw(record + 3u)));
            let in_area_start = select(start, unit_area_start, lt64(start, unit_area_start));
            let in_area_end = select(end, device_end_slot, lt64(device_end_slot, end));
            if (!lt64(in_area_start, in_area_end)) {
                continue;
            }
            segments_holding_a_record = c_tally_the_segments_of_an_interval(segments_holding_a_record, in_area_start, in_area_end, unit_area_start,
                full_segments);
        }
        let empty_segments_by_the_records = sub64(full_segments, segments_holding_a_record.counted);
        if (c_accounting_row(STATISTIC_EMPTY_CLUSTER_SEGMENTS, device, &recorded)) {
            judge(INVARIANT_I_3_16, eq64(recorded, empty_segments_by_the_records));
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_16);
        }
    }
}

fn judge_accounting(fsid_low: vec2<u32>) {
    let device_count = hdr(3u);
    let newest_root = checker_root_of(c_newest_index);
    let written_under_effective_floor = eq64(newest_root.floor, c_effective_floor);
    c_harvested = false;
    // 两遍同一个调用点：第一遍到走读的引用数为止，存下它的豁免数、算完 F 之下的推迟，第二遍到最新根自己的引用数为止。
    // 两遍隔离豁免与 F 之下的推迟照 walk.rs check_pool_image 不看有没有记账都算（根环里没走过的根那一份位置项校验和补没补过，
    // 经 C_HARVESTED_BY_THE_ACCOUNTING_QUARANTINE 交给引用与快照那一截的 I-3.15）；判 I-3.1 那几条才要记账
    var exempted_all: array<u32, 8>;
    for (var exemption_pass = 0u; exemption_pass < 2u; exemption_pass = exemption_pass + 1u) {
        var reference_limit = w_reference_count[0];
        if (exemption_pass == 1u) {
            for (var d = 0u; d < 8u; d = d + 1u) {
                exempted_all[d] = c_exempted[d];
            }
            // I-3.16 占着的槽：第一遍（全部走过的版本）整组豁免的那几组，第二遍改写 C_GROUPS 之前收走
            c_occupied_count = 0u;
            c_note_the_quarantined_groups_as_occupied();
            c_deferred_below_the_floor(newest_root.base);
            reference_limit = c_newest_reference_count;
        }
        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);
    }
    sws(C_HARVESTED_BY_THE_ACCOUNTING_QUARANTINE, select(0u, 1u, c_harvested));
    if (!w_accounting_seen) {
        return;
    }
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var allocated = vec2<u32>(0u, 0u);
        let have_allocated = c_accounting_row(1u, device, &allocated);
        let walked = shl64(add64(add64(c_slots_referenced(device, w_reference_count[0]), u64_of(exempted_all[d])), u64_of(c_deferred_per_device[d])), SLOT_SHIFT);
        if (written_under_effective_floor) {
            judge(INVARIANT_I_3_1, have_allocated && eq64(allocated, walked));
        }
        // I-5.2：空闲 + 已分配 = 单元区容量
        let slots = shr64(device_bytes(), SLOT_SHIFT);
        var free = vec2<u32>(0u, 0u);
        let have_free = c_accounting_row(2u, device, &free);
        var holds = have_free && have_allocated && le64(handoff_unit_area_start_slot(), slots) && !add64_overflows(free, allocated);
        if (holds) {
            holds = eq64(add64(free, allocated), shl64(sub64(slots, handoff_unit_area_start_slot()), SLOT_SHIFT));
        }
        judge(INVARIANT_I_5_2, holds);
        // I-3.11：defer + 最新根走读（含隔离豁免） = 已分配
        var deferred = vec2<u32>(0u, 0u);
        let have_deferred = c_accounting_row(5u, device, &deferred);
        let referenced_by_newest = shl64(add64(c_slots_referenced(device, c_newest_reference_count), u64_of(c_exempted[d])), SLOT_SHIFT);
        var holds_defer = have_allocated && have_deferred && !add64_overflows(deferred, referenced_by_newest);
        if (holds_defer) {
            holds_defer = eq64(add64(deferred, referenced_by_newest), allocated);
        }
        judge(INVARIANT_I_3_11, holds_defer);
    }
    // I-3.16（walk.rs 在 I-3.1 那一格之后判，同一个条件：有记账、最新根的记账按 F 生效值记的）
    if (written_under_effective_floor) {
        judge_fragmentation_and_empty_segment_statistics();
    }
}

// ── I-1.8：扫描方向按已发布谓词过滤的码 1 / 码 3 单元归并成组
// 一个扫描项的归并键：码 1 是 [42, 101)；码 3 是 [42, 89) 与 [93, 107)。原来每一项跟它前面、后面的每一项各比一遍归并键（码 3 的对象 key 那一半
// 还要套两三层），扫描项数的平方乃至更高次长；改成按归并键排一次序、按序走一遍。比的时候码 3 的归并键换一个字节次序：先对象 key [42, 69)，
// 再全序键（诞生代号 [73, 81)、实例与事务号 [93, 103)），再归并键其余的 [69, 73)、[81, 89)、[103, 107)。换的只是同一组字节的次序，
// 两条归并键相等 ⟺ 这样排的字节逐个相等；同一组的成员在次序里连成一段，同对象 key 的组连成一串，串里全序键相同的组又挨在一起
const C_MERGE_KEY_BYTES_OF_A_DATA_UNIT: u32 = 59u;

const C_MERGE_KEY_BYTES_OF_A_PACKED_UNIT: u32 = 61u;

// 码 3 换过次序的归并键里对象 key 占前几个字节、对象 key 连着全序键占前几个字节
const C_OBJECT_KEY_BYTES: u32 = 27u;

const C_OBJECT_KEY_AND_TOTAL_ORDER_KEY_BYTES: u32 = 45u;

// 扫描项按归并键排好的次序借 C_ROW_ORDER（至多 C_MAX_SCANNED 个，那张表 C_MAX_REFERENCES 个字）；每一项读出来那一份切片的检查字（c2）
// 借 C_JOURNAL_GROUP_KEYS 的前 C_MAX_SCANNED 个字（那一段 C_JOURNAL_GROUP_KEY_WORDS × MAX_RECORDS 个字）。I-1.8 是扫描内核第一判，
// 之后 I-5.4 借 C_ROW_ORDER、I-3.9 与 I-3.10 借 C_JOURNAL_GROUP_KEYS 都各自从头写
const C_SCANNED_CHECK_WORDS: u32 = C_JOURNAL_GROUP_KEYS;

fn c_merge_key_bytes(unit_class: u32) -> u32 {
    return select(C_MERGE_KEY_BYTES_OF_A_PACKED_UNIT, C_MERGE_KEY_BYTES_OF_A_DATA_UNIT, unit_class == 1u);
}

// 换过次序的归并键第 position 个字节在单元里的偏移
fn c_merge_key_byte_offset(unit_class: u32, position: u32) -> u32 {
    if (unit_class == 1u || position < C_OBJECT_KEY_BYTES) {
        return 42u + position;
    }
    if (position < 35u) {
        return 73u + position - C_OBJECT_KEY_BYTES;
    }
    if (position < C_OBJECT_KEY_AND_TOTAL_ORDER_KEY_BYTES) {
        return 93u + position - 35u;
    }
    if (position < 49u) {
        return 69u + position - C_OBJECT_KEY_AND_TOTAL_ORDER_KEY_BYTES;
    }
    if (position < 57u) {
        return 81u + position - 49u;
    }
    return 103u + position - 57u;
}

// 扫描项 a 与 b（收的次序里的下标）换过次序的归并键从头起有几个字节相同；两项的类不同交 0
fn c_scanned_units_common_merge_key_prefix(a: u32, b: u32) -> u32 {
    let base_a = sw(C_SCANNED + a);
    let base_b = sw(C_SCANNED + b);
    let unit_class = rd8(base_a, 6u);
    if (rd8(base_b, 6u) != unit_class) {
        return 0u;
    }
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

// 扫描项 a 排在 b 前面：先按类（码 1 在前），再按换过次序的归并键逐字节比，键相同的按收的次序——全序。同一组的段里按收的次序走，
// 段首就是原写法里「这一组的代表」（收的次序里第一个）
fn c_scanned_unit_precedes(a: u32, b: u32) -> bool {
    let base_a = sw(C_SCANNED + a);
    let base_b = sw(C_SCANNED + b);
    let class_a = rd8(base_a, 6u);
    let class_b = rd8(base_b, 6u);
    if (class_a != class_b) {
        return class_a < class_b;
    }
    let prefix = c_scanned_units_common_merge_key_prefix(a, b);
    if (prefix == c_merge_key_bytes(class_a)) {
        return a < b;
    }
    let offset = c_merge_key_byte_offset(class_a, prefix);
    return rd8(base_a, offset) < rd8(base_b, offset);
}

// 前 count 个扫描项按 c_scanned_unit_precedes 排，下标写进 C_ROW_ORDER 起的 count 个字。堆排序，写法同 order_rows_by_their_first_four_words
fn c_order_the_scanned_units_by_merge_key(count: u32) {
    for (var i = 0u; i < count; i = i + 1u) {
        sws(C_ROW_ORDER + i, i);
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
            let largest = sw(C_ROW_ORDER);
            sws(C_ROW_ORDER, sw(C_ROW_ORDER + heap_size));
            sws(C_ROW_ORDER + heap_size, largest);
        }
        loop {
            let left = 2u * node + 1u;
            if (left >= heap_size) {
                break;
            }
            var larger = left;
            if (left + 1u < heap_size && c_scanned_unit_precedes(sw(C_ROW_ORDER + left), sw(C_ROW_ORDER + left + 1u))) {
                larger = left + 1u;
            }
            if (!c_scanned_unit_precedes(sw(C_ROW_ORDER + node), sw(C_ROW_ORDER + larger))) {
                break;
            }
            let parent = sw(C_ROW_ORDER + node);
            sws(C_ROW_ORDER + node, sw(C_ROW_ORDER + larger));
            sws(C_ROW_ORDER + larger, parent);
            node = larger;
        }
    }
}

fn c_payload_checksum(base: u32) -> u32 {
    if (rd8(base, 6u) == 1u) {
        return rd32(base, 101u);
    }
    return rd32(base, 89u);
}

fn judge_merged_version_total_order(fsid_low: vec2<u32>) {
    let location_count = hdr(6u);
    var scanned = 0u;
    for (var l = 0u; l < location_count; l = l + 1u) {
        if (location_word(l, 4u) != LOCATION_UNIT_SLOT) {
            continue;
        }
        let s = read_location(l);
        if (s.version == NONE || s.length < 4096u || rd32(s.base, 0u) != MAGIC_SFSU) {
            continue;
        }
        let unit_class = rd8(s.base, 6u);
        var fsid_offset = 83u;
        var birth_offset = 75u;
        var write_order_offset = 91u;
        if (unit_class == 3u) {
            fsid_offset = 81u;
            birth_offset = 73u;
            write_order_offset = 93u;
        } else if (unit_class != 1u) {
            continue;
        }
        if ((s.c2 & 1u) == 0u || !eq64(rd64(s.base, fsid_offset), fsid_low)) {
            continue;
        }
        let birth = rd64(s.base, birth_offset);
        let instance = rd32(s.base, write_order_offset);
        let transaction = rd48(s.base, write_order_offset + 4u);
        if (!w_published(birth, instance, unit_class == 1u, transaction)) {
            continue;
        }
        if (scanned >= C_MAX_SCANNED) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (36u << 8u);
            return;
        }
        // 原写法存落点下标、每比一次重读一遍（同一个状态里同一个落点读出同一份切片）：这里读一次，存字节基与检查字
        sws(C_SCANNED + scanned, s.base);
        sws(C_SCANNED_CHECK_WORDS + scanned, s.c2);
        scanned = scanned + 1u;
    }
    c_order_the_scanned_units_by_merge_key(scanned);
    // ② 那一串（同对象 key 的码 3 各组）：上一组的代表、这一串几组、串里有没有两组全序键相同
    var previous_packed_representative = NONE;
    var representatives_of_the_object_key = 0u;
    var two_representatives_share_the_total_order_key = false;
    var group_start = 0u;
    while (group_start < scanned) {
        let representative = sw(C_ROW_ORDER + group_start);
        let unit_class = rd8(sw(C_SCANNED + representative), 6u);
        let key_bytes = c_merge_key_bytes(unit_class);
        var group_end = group_start + 1u;
        while (group_end < scanned && c_scanned_units_common_merge_key_prefix(representative, sw(C_ROW_ORDER + group_end)) == key_bytes) {
            group_end = group_end + 1u;
        }
        // ① 同一组的成员载荷相同：可读的（载荷 CRC 在盘上对得上）成员的载荷校验和不超过一种。原写法先看校验和字段全相同就判成立，
        // 不全相同才看可读的那几个——全相同时可读的那几个也只有一种，两种写法同判；两个以上成员的组各判一次（walk.rs judge_merged_version_total_order 的 judged）
        var readable_checksum = 0u;
        var have_readable = false;
        var more_than_one = false;
        for (var position = group_start; position < group_end; position = position + 1u) {
            let member = sw(C_ROW_ORDER + position);
            if ((sw(C_SCANNED_CHECK_WORDS + member) & 2u) == 0u) {
                continue;
            }
            let checksum = c_payload_checksum(sw(C_SCANNED + member));
            if (!have_readable) {
                readable_checksum = checksum;
                have_readable = true;
            } else if (checksum != readable_checksum) {
                more_than_one = true;
            }
        }
        if (group_end - group_start >= 2u) {
            judge(INVARIANT_I_1_8, !more_than_one);
        }
        // ② 同对象 key 的各组两两全序键不等（只判码 3，码 3 排在码 1 后面）：同对象 key 的组连成一串、串里全序键相同的组挨在一起，
        // 比相邻两组的代表就够；一串两组以上判一次
        if (unit_class == 3u) {
            var same_object_key = false;
            if (previous_packed_representative != NONE) {
                let prefix = c_scanned_units_common_merge_key_prefix(previous_packed_representative, representative);
                same_object_key = prefix >= C_OBJECT_KEY_BYTES;
                if (prefix >= C_OBJECT_KEY_AND_TOTAL_ORDER_KEY_BYTES) {
                    two_representatives_share_the_total_order_key = true;
                }
            }
            if (!same_object_key) {
                if (representatives_of_the_object_key >= 2u) {
                    judge(INVARIANT_I_1_8, !two_representatives_share_the_total_order_key);
                }
                representatives_of_the_object_key = 0u;
                two_representatives_share_the_total_order_key = false;
            }
            representatives_of_the_object_key = representatives_of_the_object_key + 1u;
            previous_packed_representative = representative;
        }
        group_start = group_end;
    }
    if (representatives_of_the_object_key >= 2u) {
        judge(INVARIANT_I_1_8, !two_representatives_share_the_total_order_key);
    }
}

// I-3.12：根环里每条自证过的根，它自己指着的实例表（沿链读，只读不判）每一行 (i, T, W) 满足 T < 这条根的 checkpoint_txg；
// 链上某一片读不出的根不判（walk.rs 的 judge_instance_table_rows_against_the_roots_referencing_them）
fn judge_instance_table_rows_against_the_roots_referencing_them() {
    for (var i = 0u; i < root_count; i = i + 1u) {
        let root = checker_root_of(i);
        if (!r_instance_rows(root.base)) {
            continue;
        }
        for (var k = 0u; k < c_r_row_count; k = k + 1u) {
            let row_txg = vec2<u32>(sw(C_R_ROWS + k * 3u + 1u), sw(C_R_ROWS + k * 3u + 2u));
            judge(INVARIANT_I_3_12, lt64(row_txg, root.txg));
        }
    }
}

// 第三批的入口
// I-5.5 那一判用的两个量（walk.rs 的 CLUSTER_SEGMENT_SLOTS 取自 singlefs_format、DATA_UNIT_SLOTS）：聚簇段 64 槽，一个数据单元占 2 槽
const FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS: u32 = 64u;
const FIRST_DATA_UNIT_DATA_UNIT_SLOTS: u32 = 2u;

// C_ALLOC_RECORDS 里装着这一版的分配记录时，device 上的槽 slot 算不算被占：被「已释放、或分配代早于这一版的 txg」的记录罩着
// （walk.rs first_data_unit_slot_the_allocation_records_leave_on_a_device 的 occupied_slots；这一版自己分配的记录当空）
fn c_slot_is_occupied_before_the_version(device: u32, slot: vec2<u32>, txg: vec2<u32>) -> bool {
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        if (sw(a) != device) {
            continue;
        }
        let generation = vec2<u32>(sw(a + 4u), sw(a + 5u));
        if (sw(a + 6u) == 0u && !lt64(generation, txg)) {
            continue;
        }
        let start = vec2<u32>(sw(a + 1u), sw(a + 2u));
        if (le64(start, slot) && lt64(slot, add64(start, u64_of(sw(a + 3u))))) {
            return true;
        }
    }
    return false;
}

// 起点 candidate 的那一对合不合 I-5.5 的落点：不在开放段 [open_first, open_first + 64) 里（has_open 为假时不排除），两槽都不被占
fn c_first_data_unit_pair_is_free(device: u32, candidate: vec2<u32>, txg: vec2<u32>, has_open: bool, open_first: vec2<u32>) -> bool {
    if (has_open && le64(open_first, candidate)
        && lt64(candidate, add64(open_first, u64_of(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS)))) {
        return false;
    }
    return !c_slot_is_occupied_before_the_version(device, candidate, txg)
        && !c_slot_is_occupied_before_the_version(device, add64(candidate, u64_of(1u)), txg);
}

// I-5.5 的判据（walk.rs first_data_unit_slot_the_allocation_records_leave_on_a_device，近似在那里写明）：从 U 起，device 上起点最小、
// 起点是 2 的整数倍、两槽都空、不在开放段里的那一对的起点。walk.rs 从 U 起一对一对往后扫；这里不扫，只试几个候选取合格里最小的：
// 最小的合格起点要么是 U，要么它前面那一对不合格——那一对里有被占的槽（罩住它的那条记录的末端向上取到 2 的整数倍就是它），
// 或在开放段里（开放段的末端就是它）。比每个被占的槽、比开放段都靠后的那个候选总合格，所以一定找得到，与 walk.rs 交回同一个起点
fn c_first_data_unit_slot_the_allocation_records_leave_on_a_device(device: u32, txg: vec2<u32>, has_open: bool, open_first: vec2<u32>) -> vec2<u32> {
    let unit_area_start = handoff_unit_area_start_slot();
    var expected = unit_area_start;
    var found = c_first_data_unit_pair_is_free(device, unit_area_start, txg, has_open, open_first);
    if (has_open) {
        let past_the_open_segment = add64(open_first, u64_of(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS));
        if (le64(unit_area_start, past_the_open_segment) && (!found || lt64(past_the_open_segment, expected))
            && c_first_data_unit_pair_is_free(device, past_the_open_segment, txg, has_open, open_first)) {
            expected = past_the_open_segment;
            found = true;
        }
    }
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        let generation = vec2<u32>(sw(a + 4u), sw(a + 5u));
        if (sw(a) != device || (sw(a + 6u) == 0u && !lt64(generation, txg))) {
            continue;
        }
        var past_the_record = add64(vec2<u32>(sw(a + 1u), sw(a + 2u)), u64_of(sw(a + 3u)));
        if ((past_the_record.x % FIRST_DATA_UNIT_DATA_UNIT_SLOTS) != 0u) {
            past_the_record = add64(past_the_record, u64_of(FIRST_DATA_UNIT_DATA_UNIT_SLOTS - past_the_record.x % FIRST_DATA_UNIT_DATA_UNIT_SLOTS));
        }
        if (le64(unit_area_start, past_the_record) && (!found || lt64(past_the_record, expected))
            && c_first_data_unit_pair_is_free(device, past_the_record, txg, has_open, open_first)) {
            expected = past_the_record;
            found = true;
        }
    }
    return expected;
}

// I-5.5（walk.rs judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark）的一版：树表指针在 (base, offset)、它的 txg。
// 认法 ① 树表非空、种类 1 条目恰好一条且树 ID 是 mkfs 水位 65536；② 每条诞生 txg（条目偏移 108）都等于这一版的 txg。认出来、extent 树读得全、
// 种类 3 条目恰好一条且那棵分配记录树读得全时，盘表里每块有它数据单元的盘上最小落点要等于这一版的分配记录在那块盘上算出的落点
// （c_first_data_unit_slot_the_allocation_records_leave_on_a_device；U 取交接段里所选系统配置的单元区起始槽号，开放段取这一版树表单元
// 在那块盘上那一份所在的段）。CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、种类 1 不止一条、extent 树读不全、
// 某块盘上没有数据单元、种类 3 不是恰好一条、分配记录树读不全）在这里一格都不置位
// mapping_root_offset 起是这一版的中央映射根指针（候选根取根记录偏移 256，由记录施加出来的那一版取记录新根段偏移 86）：extent 树、
// 分配记录树的节点与数据单元按它认位置
fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32, txg: vec2<u32>, mapping_root_offset: u32) {
    // 判到位：CPU 记进 versions_not_judged（部分没判）的那几处在这里记射程里有一部分没判——树表指针全零、读不出、条目比字段表短、
    // 种类 1 不止一条、extent 树读不全、种类 3 不是恰好一条、分配记录树读不全；树表 0 条、没有种类 1、不是从 mkfs 水位建树、extent 树根全零、
    // 没有数据单元只是这一版不判
    let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
    if (tree_table_pointer.all_zero) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    if (tt.entry_count == 0u) {
        return;
    }
    if (tt.entry_width < 200u) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    var extent_entry_count = 0u;
    var extent_entry = 0u;
    var allocation_entry_count = 0u;
    var allocation_entry = 0u;
    var every_birth_is_this_version = true;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + 10u) == TREE_KIND_EXTENT) {
            extent_entry_count = extent_entry_count + 1u;
            extent_entry = entry;
        }
        if (rd16(tt.base, entry + 10u) == TREE_KIND_ALLOCATION) {
            allocation_entry_count = allocation_entry_count + 1u;
            allocation_entry = entry;
        }
        if (!eq64(rd64(tt.base, entry + 108u), txg)) {
            every_birth_is_this_version = false;
        }
    }
    if (extent_entry_count == 0u) {
        return;
    }
    if (extent_entry_count > 1u) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    if (!every_birth_is_this_version || !eq64(rd64(tt.base, extent_entry), u64_of(TREE_IDENTIFIER_WATERMARK_AT_MKFS))) {
        return;
    }
    if (parse_node_pointer(tt.base, extent_entry + 14u).all_zero) {
        return;
    }
    for (var device_index = 0u; device_index < MAX_DEVICES; device_index = device_index + 1u) {
        r_has_data_unit_on_device[device_index] = 0u;
    }
    s_use_the_central_mapping_named_at(base, mapping_root_offset);
    r_noting = false;
    r_collecting_data_unit_slots = true;
    let read_the_whole_extent_tree = r_extent_tree(tt.base, extent_entry + 14u);
    r_collecting_data_unit_slots = false;
    if (!read_the_whole_extent_tree) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    let device_count = hdr(3u);
    var has_any_data_unit = false;
    for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
        if (r_has_data_unit_on_device[device_index] == 1u) {
            has_any_data_unit = true;
        }
    }
    if (!has_any_data_unit) {
        return;
    }
    if (allocation_entry_count != 1u) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    c_alloc_record_count = 0u;
    if (!r_allocation_tree(tt.base, allocation_entry + 14u)) {
        mark_part_of_the_range_not_judged(INVARIANT_I_5_5);
        return;
    }
    for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
        if (r_has_data_unit_on_device[device_index] == 0u) {
            continue;
        }
        let device = hdr(12u + device_index);
        // 开放段：这一版树表单元在这块盘上那一份所在的 64 槽段（walk.rs open_cluster_segment_named_by_the_tree_table_on_a_device，先看第一条位置条目）
        var has_open = false;
        var open_first = vec2<u32>(0u, 0u);
        if (tree_table_pointer.device0 == device) {
            has_open = true;
            open_first = vec2<u32>(tree_table_pointer.slot0.x & ~(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS - 1u), tree_table_pointer.slot0.y);
        } else if (tree_table_pointer.device1 == device) {
            has_open = true;
            open_first = vec2<u32>(tree_table_pointer.slot1.x & ~(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS - 1u), tree_table_pointer.slot1.y);
        }
        let expected_slot = c_first_data_unit_slot_the_allocation_records_leave_on_a_device(device, txg, has_open, open_first);
        judge(INVARIANT_I_5_5, eq64(r_smallest_data_unit_slot_on_device[device_index], expected_slot));
    }
}

// I-5.5：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针）
fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark() {
    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
        var base = 0u;
        var tree_table_offset = 36u;
        var mapping_root_offset = G_ROOT_RECORD_MAPPING_ROOT_POINTER;
        var txg = vec2<u32>(0u, 0u);
        if (version_index < c_candidate_count) {
            let root = checker_root_of(sw(C_CANDIDATES + version_index));
            base = root.base;
            txg = root.txg;
        } else {
            let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
            base = record.base;
            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
            mapping_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER;
            txg = record.txg;
        }
        judge_first_data_unit_of_a_version(base, tree_table_offset, txg, mapping_root_offset);
    }
}

// ───────────────────────── 删除态扫描（walk.rs judge_deletion_state_of_the_versions）与 I-9.6 墓碑那一半 ─────────────────────────
// I-9.5、I-3.4、I-4.7、I-4.6、I-4.10、I-9.38、I-9.24 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。CPU 记成射程里判不了的那几格
// （树表读不出、条目宽不到 200、deleted_inodes 的头 ID 配不上恰好一条 inode 树、队列根读不出或形状不对、inode 树 / extent 树 / 记账树读不全、
// 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
// I-4.4（logged_ops 条目，整理与截断两种）在引用与快照那一截判（g_judge_the_compaction_intents_of_the_version）。

// inode 记录宽（类型 2 登记 140）；对象出生代、nlink、size 的偏移用共用那一半的 INODE_RECORD_OBJECT_BIRTH_OFFSET、INODE_RECORD_NLINK_OFFSET、
// INODE_RECORD_SIZE_OFFSET（crash_judge_common.wgsl 拼在三个内核前面），不另定义
const DELETION_PACKED_TYPE_INODE: u32 = 2u;
const DELETION_INODE_RECORD_BYTES: u32 = 140u;
// inode 树与墓碑树码 2 节点条目里的类型段（偏移 16，2 字节）：0 指下一层码 2 节点，2 指 inode 叶容器（墓碑树里 1 指墓碑容器）
const DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET: u32 = 16u;
const DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u32 = 0u;
// 墓碑记录 40 字节（类型 1 登记 40）：对象 ID 0、对象出生代 8、区间起 16、区间止 24、死亡 txg 32（walk.rs TOMBSTONE_RECORD_*_OFFSET，
// D18（块里携带什么信息） 已定项 24 第 2 条）；第一版只许整对象 [0, 2⁶⁴ − 1]；一片墓碑容器最多 815 条（⌊(32768 − 136) ÷ 40⌋，walk.rs TOMBSTONE_CONTAINER_RECORDS）
const DELETION_TOMBSTONE_RECORD_BYTES: u32 = 40u;
const DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: u32 = 0u;
const DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;
const DELETION_TOMBSTONE_RECORD_RANGE_START_OFFSET: u32 = 16u;
const DELETION_TOMBSTONE_RECORD_RANGE_END_OFFSET: u32 = 24u;
const DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET: u32 = 32u;
const DELETION_TOMBSTONE_CONTAINER_RECORDS: u32 = 815u;
// 墓碑树、谱系树、退役树沿叶序第一片容器的容器号（walk.rs FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE，D18（块里携带什么信息） 已定项 24 第 1、5 条：
// 容器号从 1 起，下一个号 = 最右号 + 1）
const D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE: u32 = 1u;
// 谱系记录 24 字节（类型 5 登记 24）：T_rb 0、T_old 8、H_cur 16（回退之前的现行那一版的 inode 号水位），读作「T_rb 的父是 T_old」
// （walk.rs LINEAGE_RECORD_*_OFFSET，D18（块里携带什么信息） 已定项 24 第 5 条）
const DELETION_LINEAGE_RECORD_BYTES: u32 = 24u;

// 一片谱系容器的产品容量 ⌊(32768 − 136) ÷ 24⌋ = 1359（walk.rs 的 LINEAGE_CONTAINER_RECORDS；I-9.36 判非最右那几片装满）
const DELETION_LINEAGE_CONTAINER_RECORDS: u32 = 1359u;
const DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: u32 = 0u;
const DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET: u32 = 8u;
const DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: u32 = 16u;
// 记账树：key 22（统计量 2、树 8、设备 4、代 8），叶条目 34，内部条目 key + 子指针 86；行里设备段在 10、值在 22
const DELETION_ACCOUNTING_KEY_BYTES: u32 = 22u;
const DELETION_ACCOUNTING_ENTRY_BYTES: u32 = 34u;
const DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES: u32 = 108u;
const DELETION_ACCOUNTING_ROW_DEVICE_OFFSET: u32 = 10u;
const DELETION_ACCOUNTING_ROW_GENERATION_OFFSET: u32 = 14u;
const DELETION_ACCOUNTING_ROW_VALUE_OFFSET: u32 = 22u;
// 记账统计量的行号：4 待删除但意图未完成占用（walk.rs STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES）、12 inode 号水位（STATISTIC_INODE_WATERMARK）
const DELETION_STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES: u32 = 4u;
const DELETION_STATISTIC_INODE_WATERMARK: u32 = 12u;
// 树表条目宽（字段表 200）、条目里种类与根指针的偏移
const DELETION_TREE_TABLE_ENTRY_BYTES: u32 = 200u;
const DELETION_TREE_TABLE_ENTRY_KIND_OFFSET: u32 = 10u;
const DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: u32 = 14u;

var<private> d_inode_record_count: u32;

// 这一版 inode 记录那一张（键是 inode 号，crash_judge_checker_common.wgsl 的 ⑬）与它的桶表建过没有
fn d_inode_record_table() -> KeyedTable {
    return KeyedTable(D_INODE_RECORDS, D_INODE_RECORD_WORDS, 2u, D_INODE_RECORD_BUCKETS, D_INODE_RECORD_BUCKET_COUNT);
}

var<private> d_inode_records_are_indexed: bool;

// 这一版的一条 inode 记录记进 D_INODE_RECORDS：同一个号已有一行就盖掉（walk.rs 是按号插入的 BTreeMap，后读到的盖掉先读到的）；表满交回 false。
// 截断字段 T 与 N_w 两段照记（walk.rs InodeRecordOfTheDeletionScan 的 truncation_field_txg、truncation_field_size_in_bytes，I-9.39 读）
fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>, object_birth: vec2<u32>,
    truncation_field_txg: vec2<u32>, truncation_field_size: vec2<u32>) -> bool {
    let existing = d_inode_record_row_of(inode);
    if (existing != NONE) {
        sws(existing + 2u, nlink);
        sws(existing + 3u, size_in_bytes.x);
        sws(existing + 4u, size_in_bytes.y);
        sws(existing + 5u, object_birth.x);
        sws(existing + 6u, object_birth.y);
        sws(existing + 7u, truncation_field_txg.x);
        sws(existing + 8u, truncation_field_txg.y);
        sws(existing + 9u, truncation_field_size.x);
        sws(existing + 10u, truncation_field_size.y);
        return true;
    }
    if (d_inode_record_count >= D_MAX_INODE_RECORDS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (49u << 8u);
        return false;
    }
    let row = D_INODE_RECORDS + d_inode_record_count * D_INODE_RECORD_WORDS;
    sws(row, inode.x);
    sws(row + 1u, inode.y);
    sws(row + 2u, nlink);
    sws(row + 3u, size_in_bytes.x);
    sws(row + 4u, size_in_bytes.y);
    sws(row + 5u, object_birth.x);
    sws(row + 6u, object_birth.y);
    sws(row + 7u, truncation_field_txg.x);
    sws(row + 8u, truncation_field_txg.y);
    sws(row + 9u, truncation_field_size.x);
    sws(row + 10u, truncation_field_size.y);
    let appended = d_inode_record_count;
    d_inode_record_count = appended + 1u;
    d_inode_records_are_indexed = keyed_table_after_a_row_was_appended(d_inode_record_table(), d_inode_record_count,
        d_inode_records_are_indexed, appended);
    return true;
}

// D_INODE_RECORDS 里一行的 nlink 与对象出生代
fn d_inode_record_nlink(row: u32) -> u32 {
    return sw(row + 2u);
}

fn d_inode_record_object_birth(row: u32) -> vec2<u32> {
    return vec2<u32>(sw(row + 5u), sw(row + 6u));
}

// D_INODE_RECORDS 里一行的截断字段 T 与 N_w
fn d_inode_record_truncation_field_txg(row: u32) -> vec2<u32> {
    return vec2<u32>(sw(row + 7u), sw(row + 8u));
}

fn d_inode_record_truncation_field_size(row: u32) -> vec2<u32> {
    return vec2<u32>(sw(row + 9u), sw(row + 10u));
}

// 这一版 inode 记录表里号 inode 的那一行（没有交回 NONE）
fn d_inode_record_row_of(inode: vec2<u32>) -> u32 {
    let row_index = keyed_first_row_of(d_inode_record_table(), d_inode_record_count, d_inode_records_are_indexed,
        vec4<u32>(inode.x, inode.y, 0u, 0u));
    if (row_index == NONE) {
        return NONE;
    }
    return D_INODE_RECORDS + row_index * D_INODE_RECORD_WORDS;
}

// 这一版 extent 树里每个号的数据单元那两张表（r_note_data_unit_of_inode 记）。借引用与快照那一截的 I-1.14 跨版本那张 key 表（G_KEY_GROUPS 起）那一段：
// 扫描内核主入口里删除态扫描在 I-3.16 之前跑，I-3.16 借 S_KEYED_UNITS 起的那一段也从头写，引用与快照那一截的几张表都从头写、写过才读，
// 不读这里留下的字；主入口里 G_KEY_GROUPS 之前那几张（logged_ops 叶条目、读树的栈）截断条目那一判照它们自己的用法借。
// ① 每个号一行：号 lo/hi、单元数、上段叶条目的声明（D_DECLARED_*）、它最后记进的那个单元在 D_DATA_UNITS 里的下标 + 1（0 是没有单元）
const D_DATA_UNITS_OF_AN_INODE_WORDS: u32 = 5u;
const D_DATA_UNITS_OF_EACH_INODE: u32 = G_KEY_GROUPS;
// ② 每个单元一行：文件字节偏移 lo/hi、数据指针所在的打包表字节基与偏移、同号前一个记进的单元的下标 + 1（0 是没有）
const D_DATA_UNIT_WORDS: u32 = 5u;
const D_MAX_DATA_UNITS: u32 = 8192u;
const D_DATA_UNITS: u32 = D_DATA_UNITS_OF_EACH_INODE + D_DATA_UNITS_OF_AN_INODE_WORDS * D_MAX_DATA_UNIT_INODES;
// ③ I-4.6 ① 判一个号的单元连号时，单元序号 k（偏移 ÷ 净荷）见过没有的位表：位 k 在第 k / 32 字的第 k % 32 位，一个号一个号地清
const D_DATA_UNIT_POSITIONS_SEEN: u32 = D_DATA_UNITS + D_DATA_UNIT_WORDS * D_MAX_DATA_UNITS;
const_assert D_DATA_UNIT_POSITIONS_SEEN + D_MAX_DATA_UNITS / 32u <= SCRATCH_END;
const_assert G_PENDING + G_PENDING_WORDS * G_MAX_PENDING <= D_DATA_UNITS_OF_EACH_INODE;
// 单元多于 D_MAX_DATA_UNITS 个判不了（「条目太多」一类的站点号，与别处不撞）
const D_SITE_DATA_UNITS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (105u << 8u);

// 上段叶条目对一个号的声明（walk.rs DataUnitsDeclaredByTheUpperLeafEntry 加「没有上段叶条目」）
const D_DECLARED_BY_NO_UPPER_LEAF_ENTRY: u32 = 0u;
const D_DECLARED_NO_DATA_UNIT: u32 = 1u;
const D_DECLARED_DATA_UNITS_WITHOUT_A_HOLE_AFTER_THEM: u32 = 2u;
const D_DECLARED_A_HOLE_UP_TO_THE_SIZE_AFTER_THE_LAST_DATA_UNIT: u32 = 3u;

// 截断条目的种类码与 value 里各段的偏移（walk.rs LOGGED_OPERATION_KIND_TRUNCATION 与 TRUNCATION_INTENT_*_OFFSET_IN_THE_VALUE；判决 m4-trunc-r1 K1：
// value 34 = inode 号 8、对象出生代 8、目标 size 8、补零 10）
const G_LOGGED_OPERATION_KIND_TRUNCATION: u32 = 2u;
const G_TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE: u32 = 0u;
const G_TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE: u32 = 8u;
const G_TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE: u32 = 16u;
const G_TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE: u32 = 24u;
const G_TRUNCATION_INTENT_PADDING_BYTES: u32 = 10u;

// 数据单元头里声明长度那一段的偏移（walk.rs DATA_UNIT_DECLARED_LENGTH_OFFSET，D18（块里携带什么信息） 已定项 7 字段表：码 1 偏移 8，2 字节）
const D_DATA_UNIT_DECLARED_LENGTH_OFFSET: u32 = 8u;

// 数据单元头里诞生代号那 8 字节的偏移（walk.rs 经 ContentUnitClass::Data.header_offsets().birth_txg，同一张字段表：码 1 偏移 75）
const D_DATA_UNIT_BIRTH_TXG_OFFSET: u32 = 75u;

// 一条记录的 size 要几个数据单元：⌈size ÷ 净荷容量⌉.max(1)（walk.rs data_units_the_size_asks_for）
fn d_data_units_the_size_asks_for(size_in_bytes: vec2<u32>) -> vec2<u32> {
    let division = divmod64(size_in_bytes, u64_of(DATA_UNIT_PAYLOAD_CAPACITY));
    var asked_for = division.quotient;
    if (!is_zero64(division.remainder)) {
        asked_for = add64(asked_for, u64_of(1u));
    }
    if (is_zero64(asked_for)) {
        asked_for = u64_of(1u);
    }
    return asked_for;
}

// g_read_the_logged_operation_entries 收进 G_LOGGED_OPERATION_ENTRIES 的条目里有没有一条截断条目对得上这条记录（walk.rs a_truncation_intent_names_the_record）：
// inode 号、对象出生代相同，目标 = 记录的 size
fn d_a_truncation_intent_names_the_record(inode: vec2<u32>, object_birth: vec2<u32>, size_in_bytes: vec2<u32>) -> bool {
    for (var entry_index = 0u; entry_index < g_logged_operation_entry_count; entry_index = entry_index + 1u) {
        let base = sw(G_LOGGED_OPERATION_ENTRIES + entry_index * 2u);
        let o = sw(G_LOGGED_OPERATION_ENTRIES + entry_index * 2u + 1u);
        let value = o + LOGGED_OPERATIONS_KEY_BYTES;
        if (rd16(base, o) == G_LOGGED_OPERATION_KIND_TRUNCATION
            && eq64(rd64(base, value + G_TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE), inode)
            && eq64(rd64(base, value + G_TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE), object_birth)
            && eq64(rd64(base, value + G_TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE), size_in_bytes)) {
            return true;
        }
    }
    return false;
}

// 一个号存在的单元从 0 起连号（walk.rs judge_data_unit_counts_of_the_inodes_against_their_size ①：按偏移升序第 i 个的偏移是 i × 净荷容量）：
// 等价于每个单元的偏移都是净荷容量的整数倍 k、k 小于单元数、两两不同——单元的偏移集合恰是 {0, 净荷, …, (单元数 − 1) × 净荷}
fn d_data_units_of_the_inode_are_in_sequence_from_zero(first_unit_plus_one: u32, data_unit_count: u32) -> bool {
    for (var word = 0u; word < (data_unit_count + 31u) / 32u; word = word + 1u) {
        sws(D_DATA_UNIT_POSITIONS_SEEN + word, 0u);
    }
    var unit_plus_one = first_unit_plus_one;
    // 迭代上界：链表每个单元只记进一次（r_note_data_unit_of_inode 只往表头接），至多 D_MAX_DATA_UNITS 步
    for (var step = 0u; step < D_MAX_DATA_UNITS && unit_plus_one != 0u; step = step + 1u) {
        let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
        let division = divmod64(vec2<u32>(sw(unit), sw(unit + 1u)), u64_of(DATA_UNIT_PAYLOAD_CAPACITY));
        if (!is_zero64(division.remainder) || !lt64(division.quotient, u64_of(data_unit_count))) {
            return false;
        }
        let position = division.quotient.x;
        let word = D_DATA_UNIT_POSITIONS_SEEN + position / 32u;
        let bit = 1u << (position % 32u);
        if ((sw(word) & bit) != 0u) {
            return false;
        }
        sws(word, sw(word) | bit);
        unit_plus_one = sw(unit + 4u);
    }
    return true;
}

// I-4.6（walk.rs judge_data_unit_counts_of_the_inodes_against_their_size，判决 m4-trunc-r1 K7 T-3 与第六节 S1）、I-4.10（judge_data_units_beyond_the_size_against_the_truncation_intents，
// T-2，只判「多出来就要有对得上的截断条目」那一个方向，第六节 S2）、I-9.38（judge_declared_lengths_of_the_data_units_within_the_size，T-4）、
// I-9.39（judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field，判决 m4-trunc-r2 第三节 R6）：
// 这一版 inode 树的每条记录（nlink = 0 与 ≥ 1 都判）。tt 是这一版的树表，读截断条目用；读不全的 I-4.6、I-4.10 判不了，I-9.38、I-9.39 照判
fn d_judge_the_data_units_of_each_record(tt: Node) {
    // 这一版 logged_ops 里的条目（walk.rs truncation_intents_of_a_version_without_judging：树表里种类 12 那几条指着的根逐棵读到叶）
    g_logged_operation_entry_count = 0u;
    var truncation_intents_read_in_full = true;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let root = parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LOGGED_OPERATIONS && !root.all_zero
            && !g_read_the_logged_operation_entries(root)) {
            truncation_intents_read_in_full = false;
        }
    }
    if (!truncation_intents_read_in_full) {
        mark_part_of_the_range_not_judged(INVARIANT_I_4_6);
        mark_part_of_the_range_not_judged(INVARIANT_I_4_10);
    }
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
        let record = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
        let inode = vec2<u32>(sw(record), sw(record + 1u));
        let size_in_bytes = vec2<u32>(sw(record + 3u), sw(record + 4u));
        let asked_for = d_data_units_the_size_asks_for(size_in_bytes);
        let row = d_data_units_row_of_inode(inode);
        var data_unit_count = 0u;
        var declaration = D_DECLARED_BY_NO_UPPER_LEAF_ENTRY;
        var first_unit_plus_one = 0u;
        if (row != NONE) {
            data_unit_count = sw(row + 2u);
            declaration = sw(row + 3u);
            first_unit_plus_one = sw(row + 4u);
        }
        if (truncation_intents_read_in_full) {
            judge(INVARIANT_I_4_6, d_data_units_of_the_inode_are_in_sequence_from_zero(first_unit_plus_one, data_unit_count));
            let names_a_truncation_intent = d_a_truncation_intent_names_the_record(inode, d_inode_record_object_birth(record), size_in_bytes);
            let existing = u64_of(data_unit_count);
            // 单元数对标签：标签 1、2 等于 size 要的个数，有一条对得上的截断条目时 size 之外可以多；标签 3、4 至少一个、少于它（至少一个洞单元）；
            // 标签 0 与没有上段叶条目的号一个单元都没有
            var count_matches_the_declaration = data_unit_count == 0u;
            if (declaration == D_DECLARED_DATA_UNITS_WITHOUT_A_HOLE_AFTER_THEM) {
                count_matches_the_declaration = eq64(existing, asked_for) || (names_a_truncation_intent && le64(asked_for, existing));
            } else if (declaration == D_DECLARED_A_HOLE_UP_TO_THE_SIZE_AFTER_THE_LAST_DATA_UNIT) {
                count_matches_the_declaration = data_unit_count >= 1u && lt64(existing, asked_for);
            }
            judge(INVARIANT_I_4_6, count_matches_the_declaration);
            judge(INVARIANT_I_4_10, le64(existing, asked_for) || names_a_truncation_intent);
        }
        // I-9.38：size 之内（偏移 ÷ 净荷 < size 要的个数）的每个单元，单元头偏移 8 的声明长度 = min(size − 起点, 净荷容量)；单元读不出的判不了
        var unit_plus_one = first_unit_plus_one;
        for (var step = 0u; step < D_MAX_DATA_UNITS && unit_plus_one != 0u; step = step + 1u) {
            let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
            unit_plus_one = sw(unit + 4u);
            let offset_in_bytes = vec2<u32>(sw(unit), sw(unit + 1u));
            if (!lt64(divmod64(offset_in_bytes, payload).quotient, asked_for)) {
                continue;
            }
            let s = r_read_located_unit(parse_data_pointer(sw(unit + 2u), sw(unit + 3u)), MAPPING_CLASS_DATA_UNIT, DATA_UNIT_BYTES);
            if (s.version == NONE) {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_38);
                continue;
            }
            var expected_length = sub64_saturating(size_in_bytes, offset_in_bytes);
            if (lt64(payload, expected_length)) {
                expected_length = payload;
            }
            judge(INVARIANT_I_9_38, eq64(u64_of(rd16(s.base, D_DATA_UNIT_DECLARED_LENGTH_OFFSET)), expected_length));
        }
        d_judge_the_birth_txgs_of_the_live_data_units_beyond_the_truncation_field(record, first_unit_plus_one, asked_for);
    }
}

// I-9.39（walk.rs judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field）：一条记录判一次。截断字段 T = 0 的照判成立；
// T ≠ 0 时，extent 树里存在、在 size 之内（单元号 < size 要的个数 asked_for）、单元号不小于 ⌈N_w ÷ 净荷⌉.max(1) 的数据单元，单元头诞生代号
// 都不早于 T。单元两份都读不出的记射程里判不了、不拿它判（读不出归 I-2.1）
fn d_judge_the_birth_txgs_of_the_live_data_units_beyond_the_truncation_field(record: u32, first_unit_plus_one: u32, asked_for: vec2<u32>) {
    let truncation_field_txg = d_inode_record_truncation_field_txg(record);
    var every_live_unit_is_born_no_earlier = true;
    if (!is_zero64(truncation_field_txg)) {
        let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
        let kept_by_the_truncation_field = d_data_units_the_size_asks_for(d_inode_record_truncation_field_size(record));
        var unit_plus_one = first_unit_plus_one;
        // 迭代上界：同 I-9.38 那一圈，链表每个单元只记进一次，至多 D_MAX_DATA_UNITS 步
        for (var step = 0u; step < D_MAX_DATA_UNITS && unit_plus_one != 0u; step = step + 1u) {
            let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
            unit_plus_one = sw(unit + 4u);
            let unit_number = divmod64(vec2<u32>(sw(unit), sw(unit + 1u)), payload).quotient;
            if (lt64(unit_number, kept_by_the_truncation_field) || !lt64(unit_number, asked_for)) {
                continue;
            }
            let s = r_read_located_unit(parse_data_pointer(sw(unit + 2u), sw(unit + 3u)), MAPPING_CLASS_DATA_UNIT, DATA_UNIT_BYTES);
            if (s.version == NONE) {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_39);
                continue;
            }
            if (lt64(rd64(s.base, D_DATA_UNIT_BIRTH_TXG_OFFSET), truncation_field_txg)) {
                every_live_unit_is_born_no_earlier = false;
            }
        }
    }
    judge(INVARIANT_I_9_39, every_live_unit_is_born_no_earlier);
}

// 码 2 节点 node 的条目逐个压进 C_R_PENDING（倒着压、弹出来是条目序：walk.rs collect_inode_records_below 按条目序往下走，同号后读到的盖掉先读到的，
// 次序要一样）：每行是条目所在的字节基、条目偏移、节点层级。压满交回 false（判不了，交 CPU）
fn d_push_container_tree_entries_in_reverse(node: Node, pending: ptr<function, u32>, site: u32) -> bool {
    for (var k = 0u; k < node.entry_count; k = k + 1u) {
        if (*pending >= C_R_MAX_PENDING) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (site << 8u);
            return false;
        }
        let row = C_R_PENDING + *pending * C_R_PENDING_WORDS;
        sws(row, node.base);
        sws(row + 1u, entry_offset(node, node.entry_count - 1u - k));
        sws(row + 2u, node.level);
        *pending = *pending + 1u;
    }
    return true;
}

// walk.rs inode_records_without_judging：一版 inode 树的全部记录读进 D_INODE_RECORDS。根指针全零是零条记录；根层级 0 只许是零条目根；
// 层级 ≥ 1 的往下走：类型段 2 的叶容器（码 3、记录类型 2、记录宽 ≥ 140）读每条记录，类型段 0 的内部节点（子指针非零、层级 = 父层级 − 1 且 ≥ 1、
// 条目宽 ≥ 120）往下走，别的类型段、读不出、形状不对都交回 false（判不了）
fn d_read_inode_records(root_pointer: Ptr) -> bool {
    d_inode_record_count = 0u;
    d_inode_records_are_indexed = false;
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
    if (!d_push_container_tree_entries_in_reverse(root, &pending, 50u)) {
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
        if (entry_type == DELETION_PACKED_TYPE_INODE) {
            if (child_pointer.all_zero) {
                return false;
            }
            let container = parse_packed_unit(r_read_located_unit(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
            if (!container.ok || container.record_type != DELETION_PACKED_TYPE_INODE
                || container.record_width < DELETION_INODE_RECORD_BYTES) {
                return false;
            }
            for (var r = 0u; r < container.record_count; r = r + 1u) {
                let record = packed_record_offset(container, r);
                if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + INODE_RECORD_NLINK_OFFSET),
                    rd64(container.base, record + INODE_RECORD_SIZE_OFFSET), rd64(container.base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET),
                    rd64(container.base, record + INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET),
                    rd64(container.base, record + INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET))) {
                    return false;
                }
            }
        } else if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
            if (child_pointer.all_zero) {
                return false;
            }
            let child = r_read_located_index_node(child_pointer);
            if (!child.ok || child.level + 1u != parent_level || child.level == 0u
                || child.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
                return false;
            }
            if (!d_push_container_tree_entries_in_reverse(child, &pending, 51u)) {
                return false;
            }
        } else {
            return false;
        }
    }
    return true;
}

// 队列根 queue（层级 0、key 宽 8 的码 2 根，条目前 8 字节是号）里有没有号 inode；queue_present 为假是根指针全零的空队列
fn d_queue_contains(queue: Node, queue_present: bool, inode: vec2<u32>) -> bool {
    if (!queue_present) {
        return false;
    }
    for (var e = 0u; e < queue.entry_count; e = e + 1u) {
        if (eq64(rd64(queue.base, entry_offset(queue, e)), inode)) {
            return true;
        }
    }
    return false;
}

// walk.rs judge_pending_deletion_occupancy_of_a_version 的一棵记账树（根指针在 (base, offset)，非零）：整棵读（key 宽 22；叶条目宽 ≥ 34；
// 内部条目宽 ≥ 108、子指针非零、孩子层级 = 父层级 − 1），走全了才判——要有标签 4、设备段 0xFFFF_FFFF 的那一行（缺行判红），
// 每一行标签 4 的值都是 0；读不全的不判
fn d_judge_pending_deletion_occupancy_of_an_accounting_tree(base: u32, offset: u32) {
    var has_the_pending_deletion_row = false;
    var some_pending_deletion_value_is_not_zero = false;
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
            mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
            return;
        }
        if (n.level == 0u) {
            if (n.entry_width < DELETION_ACCOUNTING_ENTRY_BYTES) {
                mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
                return;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                let o = entry_offset(n, e);
                if (rd16(n.base, o) != DELETION_STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES) {
                    continue;
                }
                if (rd32(n.base, o + DELETION_ACCOUNTING_ROW_DEVICE_OFFSET) == NONE) {
                    has_the_pending_deletion_row = true;
                }
                if (!is_zero64(rd64(n.base, o + DELETION_ACCOUNTING_ROW_VALUE_OFFSET))) {
                    some_pending_deletion_value_is_not_zero = true;
                }
            }
            continue;
        }
        if (n.entry_width < DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
            return;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e) + DELETION_ACCOUNTING_KEY_BYTES;
            if (parse_node_pointer(n.base, o).all_zero) {
                mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
                return;
            }
            if (pending >= C_R_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (52u << 8u);
                return;
            }
            let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
            sws(child, n.base);
            sws(child + 1u, o);
            sws(child + 2u, n.level - 1u);
            pending = pending + 1u;
        }
    }
    // 读全了：判一次缺不缺那一行、每一行标签 4 的值各判一次（判到位只看判过没有，并成一次）
    judge(INVARIANT_I_3_4, has_the_pending_deletion_row && !some_pending_deletion_value_is_not_zero);
}

// walk.rs deleted_inodes_queue_without_judging 读回的一个队列根：readable 为假是读不出或形状不对（判不了）；present 为假是根指针全零的空队列
struct DeletionQueueRead {
    readable: bool,
    present: bool,
    queue: Node,
};

// 树表 tt 的条目偏移 deleted_inodes_entry 那条 deleted_inodes 条目的队列：根指针全零是空队列；否则要读得出、层级 0、key 宽 8、条目宽 ≥ 8
fn d_read_deletion_queue(tt: Node, deleted_inodes_entry: u32) -> DeletionQueueRead {
    var read: DeletionQueueRead;
    read.readable = true;
    read.queue = bad_node();
    let queue_pointer = parse_node_pointer(tt.base, deleted_inodes_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
    read.present = !queue_pointer.all_zero;
    if (read.present) {
        read.queue = r_read_located_index_node(queue_pointer);
        if (!read.queue.ok || read.queue.level != 0u || read.queue.key_width != DELETED_INODES_ENTRY_BYTES
            || read.queue.entry_width < DELETED_INODES_ENTRY_BYTES) {
            read.readable = false;
        }
    }
    return read;
}

// I-9.24：读得出的队列里不同的号至多一个（walk.rs 把队列读成按号的集合，同一个号的几条算一条）
fn d_deletion_queue_holds_at_most_one_key(read: DeletionQueueRead) -> bool {
    if (!read.present) {
        return true;
    }
    for (var e = 1u; e < read.queue.entry_count; e = e + 1u) {
        if (!eq64(rd64(read.queue.base, entry_offset(read.queue, e)), rd64(read.queue.base, entry_offset(read.queue, 0u)))) {
            return false;
        }
    }
    return true;
}

// walk.rs judge_deletion_queue_of_a_version 里一条 deleted_inodes 条目（树表 tt 的条目偏移 deleted_inodes_entry）：先读队列判 I-9.24（每一条
// deleted_inodes 条目都判、在配对之前，判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T8 取 a）；再经偏移 116 的头 ID
// 配同一张表里树 ID 等于它的恰好一条 inode 树条目，读记录判 I-9.5 两个方向；再配头 ID 等于这棵 inode 树的恰好一条 extent 树条目，
// 读每个号的数据单元判 I-4.7（对象取丙）与 I-4.6、I-4.10、I-9.38（每条记录，d_judge_the_data_units_of_each_record）
fn d_judge_deletion_queue_of_an_entry(tt: Node, deleted_inodes_entry: u32) {
    let queue_read = d_read_deletion_queue(tt, deleted_inodes_entry);
    if (queue_read.readable) {
        judge(INVARIANT_I_9_24, d_deletion_queue_holds_at_most_one_key(queue_read));
    } else {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_24);
    }
    let inode_tree = rd64(tt.base, deleted_inodes_entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
    var inode_tree_entry_count = 0u;
    var inode_tree_entry = 0u;
    var extent_tree_entry_count = 0u;
    var extent_tree_entry = 0u;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
        if (kind == TREE_KIND_INODE && eq64(rd64(tt.base, entry), inode_tree)) {
            inode_tree_entry_count = inode_tree_entry_count + 1u;
            inode_tree_entry = entry;
        }
        if (kind == TREE_KIND_EXTENT && eq64(rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), inode_tree)) {
            extent_tree_entry_count = extent_tree_entry_count + 1u;
            extent_tree_entry = entry;
        }
    }
    // 配不上恰好一棵 inode 树、队列读不出、inode 树的记录读不全：I-9.5、I-4.7、I-4.6、I-4.10、I-9.38 记射程里有一部分没判（walk.rs DELETION_QUEUE_SCAN_INVARIANTS）
    if (inode_tree_entry_count != 1u || !queue_read.readable) {
        d_mark_the_deletion_queue_scan_not_judged();
        return;
    }
    let queue_present = queue_read.present;
    let queue = queue_read.queue;
    if (!d_read_inode_records(parse_node_pointer(tt.base, inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
        d_mark_the_deletion_queue_scan_not_judged();
        return;
    }
    // I-9.5 ①：队列里的每个号在这棵 inode 树里有记录且 nlink = 0
    var every_queued_inode_has_a_record_of_nlink_zero = true;
    if (queue_present) {
        for (var e = 0u; e < queue.entry_count; e = e + 1u) {
            let row = d_inode_record_row_of(rd64(queue.base, entry_offset(queue, e)));
            if (row == NONE || sw(row + 2u) != 0u) {
                every_queued_inode_has_a_record_of_nlink_zero = false;
            }
        }
    }
    judge(INVARIANT_I_9_5, every_queued_inode_has_a_record_of_nlink_zero);
    // I-9.5 ②：nlink = 0 的每条记录，号在队列里
    var every_record_of_nlink_zero_is_queued = true;
    // I-4.7 判过没有：处于删除中的号（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）有一个就判过
    var some_inode_is_being_deleted = false;
    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
        let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
        if (sw(row + 2u) == 0u) {
            some_inode_is_being_deleted = true;
            if (!d_queue_contains(queue, queue_present, vec2<u32>(sw(row), sw(row + 1u)))) {
                every_record_of_nlink_zero_is_queued = false;
            }
        }
    }
    judge(INVARIANT_I_9_5, every_record_of_nlink_zero_is_queued);
    if (extent_tree_entry_count != 1u) {
        d_mark_the_inodes_being_deleted_not_judged();
        return;
    }
    // 每个号的数据单元（walk.rs data_units_of_each_inode_without_judging）：根指针全零的树一个单元都没有
    d_data_unit_inode_count = 0u;
    d_data_unit_count = 0u;
    let extent_root_offset = extent_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    if (!parse_node_pointer(tt.base, extent_root_offset).all_zero) {
        r_noting = false;
        r_collecting_data_unit_slots = false;
        r_counting_data_units_of_each_inode = true;
        let read_the_whole_extent_tree = r_extent_tree(tt.base, extent_root_offset);
        r_counting_data_units_of_each_inode = false;
        if (!read_the_whole_extent_tree || undecidable != 0u) {
            d_mark_the_inodes_being_deleted_not_judged();
            return;
        }
    }
    // I-4.7：处于删除中的号（nlink = 0 的记录的号，与有数据单元而没有 nlink ≥ 1 记录的号）要么在队列里，要么一个数据单元都不剩——
    // 单元数为 0 的号自己就成立，所以只看有单元的号
    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
        let count_row = D_DATA_UNITS_OF_EACH_INODE + i * D_DATA_UNITS_OF_AN_INODE_WORDS;
        let inode = vec2<u32>(sw(count_row), sw(count_row + 1u));
        let record_row = d_inode_record_row_of(inode);
        let has_a_record_of_nlink_one_or_more = record_row != NONE && sw(record_row + 2u) >= 1u;
        if (sw(count_row + 2u) > 0u && !has_a_record_of_nlink_one_or_more) {
            some_inode_is_being_deleted = true;
            if (!d_queue_contains(queue, queue_present, inode)) {
                violate(INVARIANT_I_4_7);
            }
        }
    }
    if (some_inode_is_being_deleted) {
        judge(INVARIANT_I_4_7, true);
    }
    // I-4.6、I-4.10、I-9.38、I-9.39：每条记录（截断那一批起 I-4.6 不只判 nlink = 0 的记录）
    d_judge_the_data_units_of_each_record(tt);
}

// 删除态扫描判不全时记射程里有一部分没判的三组（walk.rs judge_deletion_state_of_the_versions 与 judge_deletion_queue_of_a_version）：
// 整张树表扫不了是 DELETION_QUEUE_SCAN_INVARIANTS 那六条加 I-3.4、I-9.24；队列配不上 inode 树、读不全是 DELETION_QUEUE_SCAN_INVARIANTS 那六条；
// extent 树配不上、读不全是 DELETION_SCAN_INVARIANTS_READING_THE_EXTENT_TREE 那五条（I-4.7、I-4.6、I-4.10、I-9.38、I-9.39）
fn d_mark_the_inodes_being_deleted_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_4_7);
    mark_part_of_the_range_not_judged(INVARIANT_I_4_6);
    mark_part_of_the_range_not_judged(INVARIANT_I_4_10);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_38);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_39);
}

fn d_mark_the_deletion_queue_scan_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_9_5);
    d_mark_the_inodes_being_deleted_not_judged();
}

fn d_mark_the_deletion_scan_not_judged() {
    d_mark_the_deletion_queue_scan_not_judged();
    mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_24);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_40);
}

// ── I-9.40：extent 上段的规范形（walk.rs 的 extent_upper_segment_shape_without_judging、canonical_extent_upper_segment_of、
// departure_of_the_extent_upper_segment_from_its_canonical_shape、judge_extent_upper_segment_shape_of_a_version）
//
// CPU 读一版上段（不进下段）记三样：根层级、读到的节点位置集合 P、叶条目的 inode 号集合 S，再与 S 定的规范形比：根层级是罩得住 max(S) 的最低一层 R
// （S 空时是 0），节点是 S 里每个号在 0..=R 每一层上罩着它的那一格。GPU 不把这两个集合抄进表（按块分批读、不设上限，用户 2026-10-19 定），按下面两条比：
// 读到的位置都是从根 (根层级, 0) 沿条目一层层往下定的，每个孩子的序号整除扇出就是父的序号，P 对祖先封闭；所以根层级等于 R 时
// ① P ⊆ 规范形 ⟺ 每个读到的节点罩着 S 里至少一个号；② 规范形 ⊆ P ⟺ S 里每个号那片叶的位置 (0, 号 ÷ 143) 读到了（它的祖先跟着都在）。
// 一遍深度优先走全上段，每个节点按后序记「子树里有没有落在它那片叶自己区间里的条目」：有的节点 ① 成立，落在自己那片叶区间里的号 ② 成立；
// 剩下的节点与号（写路径写不出，改过字节的镜像才有）逐个再走上段：找罩着那个节点的号、找那个号的那片叶。每一遍都是同一个读法、同一份字节。
const UPPER_SHAPE_WALK_TAKES_THE_SHAPE: u32 = 0u;
const UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES: u32 = 1u;
const UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION: u32 = 2u;
const UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF: u32 = 3u;
const UPPER_SHAPE_WALK_FINDS_THE_LEAF: u32 = 4u;
// 走上段的节点栈越过 C_R_PENDING 那张表（层级至多 255；写路径写出的上段不过十来层）
const UPPER_SHAPE_SITE_FRAMES_PAST_THE_STACK: u32 = UNDECIDABLE_TOO_DEEP | (116u << 8u);

// 走一遍上段记下的（u_walk_the_extent_upper_segment 写）：根层级、有没有叶条目、最大的号、子树里没有落在自己那片叶区间里的条目的节点数、
// 落在所在那片叶区间之外的条目数，与这一遍要找的那一个找到了没有、是哪一格、哪个号
var<private> u_root_level: u32;
var<private> u_has_an_entry: bool;
var<private> u_largest_inode: vec2<u32>;
var<private> u_nodes_without_an_entry_of_their_own_leaves: u32;
var<private> u_entries_outside_their_leaf: u32;
var<private> u_found: bool;
var<private> u_found_level: u32;
var<private> u_found_index: vec2<u32>;
var<private> u_found_inode: vec2<u32>;

// 上段一个节点走完（叶读完条目、内部节点的孩子都走完）：frames_left 是走完之后栈里还有几帧（父是最上面那一帧）。子树里有没有落在自己那片叶区间里的
// 条目并进父那一帧、父往下一条条目；没有的数一个，这一遍找第 wanted_ordinal 个这样的节点时记下它的位置
fn u_finish_a_node(frames_left: u32, level: u32, index: vec2<u32>, has_an_entry_of_its_own_leaves: bool, mode: u32, wanted_ordinal: u32) {
    if (!has_an_entry_of_its_own_leaves) {
        if (mode == UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES && u_nodes_without_an_entry_of_their_own_leaves == wanted_ordinal) {
            u_found = true;
            u_found_level = level;
            u_found_index = index;
        }
        u_nodes_without_an_entry_of_their_own_leaves = u_nodes_without_an_entry_of_their_own_leaves + 1u;
    }
    if (frames_left > 0u) {
        let parent = C_R_PENDING + (frames_left - 1u) * C_R_PENDING_WORDS;
        sws(parent + 4u, sw(parent + 4u) + 1u);
        if (has_an_entry_of_its_own_leaves) {
            sws(parent + 5u, 1u);
        }
    }
}

// 走一版 extent 树的上段（根指针在 base 起 root_offset 处），照 walk.rs 的 extent_upper_segment_shape_without_judging 只读不判：读不出、层级或 key 宽
// 对不上、叶条目解不开（extent_upper_leaf_entry_view）、内部条目的 key 不在父那一格里（extent_upper_child_of_entry_key）、同一个落点读到两次，交回 false
// （CPU 交 None）。mode 说这一遍另找什么（UPPER_SHAPE_WALK_*）：找第 wanted_ordinal 个；罩着哪一格的号按 wanted_span 除、比 wanted_index；
// 找那片叶时比 wanted_index
fn u_walk_the_extent_upper_segment(base: u32, root_offset: u32, mode: u32, wanted_ordinal: u32, wanted_span: vec2<u32>, wanted_index: vec2<u32>) -> bool {
    u_root_level = 0u;
    u_has_an_entry = false;
    u_largest_inode = vec2<u32>(0u, 0u);
    u_nodes_without_an_entry_of_their_own_leaves = 0u;
    u_entries_outside_their_leaf = 0u;
    u_found = false;
    r_seen_count = 0u;
    var frames = 0u;
    var pending_base = base;
    var pending_offset = root_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    // 迭代上界：每个节点进一次（同一落点读两次交 false，见过的表 C_R_MAX_SEEN 个落点）、每条内部条目往下一次；跨轮带着的是节点栈
    // （C_R_PENDING 的前 frames 行：字节基、层级、序号 lo/hi、下一条条目、子树里有没有落在自己那片叶区间里的条目）
    loop {
        if (enter) {
            let p = parse_node_pointer(pending_base, pending_offset);
            if (p.all_zero || !r_seen_insert(p.device0, p.slot0)) {
                return false;
            }
            let n = r_read_located_index_node(p);
            if (!n.ok) {
                return false;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
                u_root_level = n.level;
            }
            if (n.level != level || n.key_width != 24u) {
                return false;
            }
            if (level == 0u) {
                if (n.entry_width < EXTENT_UPPER_LEAF_ENTRY_BYTES) {
                    return false;
                }
                var has_an_entry_of_its_own_range = false;
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
                        return false;
                    }
                    let tag = rd8(n.base, o + 24u);
                    // 标签 3 同 1（指针之后两字节要是 0）、4 同 2（判决 m4-trunc-r1 K5）
                    let names_a_lower_segment_root = tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE
                        || tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT;
                    if (tag == EXTENT_UPPER_LEAF_TAG_NO_DATA_UNIT) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return false;
                        }
                    } else if (names_a_lower_segment_root) {
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return false;
                        }
                    } else if (tag != EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT && tag != EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE) {
                        return false;
                    }
                    let inode = extent_key_inode(n.base, o);
                    if (!u_has_an_entry || lt64(u_largest_inode, inode)) {
                        u_largest_inode = inode;
                    }
                    u_has_an_entry = true;
                    if (eq64(divide_by_a_divisor_below_65536(inode, EXTENT_UPPER_LEAF_INODES).quotient, pending_index)) {
                        has_an_entry_of_its_own_range = true;
                    } else {
                        if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF && u_entries_outside_their_leaf == wanted_ordinal) {
                            u_found = true;
                            u_found_inode = inode;
                        }
                        u_entries_outside_their_leaf = u_entries_outside_their_leaf + 1u;
                    }
                    if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION && eq64(divmod64(inode, wanted_span).quotient, wanted_index)) {
                        u_found = true;
                    }
                }
                if (mode == UPPER_SHAPE_WALK_FINDS_THE_LEAF && eq64(pending_index, wanted_index)) {
                    u_found = true;
                }
                u_finish_a_node(frames, level, pending_index, has_an_entry_of_its_own_range, mode, wanted_ordinal);
                if (frames == 0u) {
                    break;
                }
                enter = false;
                continue;
            }
            if (n.entry_width < 110u) {
                return false;
            }
            if (frames >= C_R_MAX_PENDING) {
                undecidable = UPPER_SHAPE_SITE_FRAMES_PAST_THE_STACK;
                return false;
            }
            let pushed = C_R_PENDING + frames * C_R_PENDING_WORDS;
            sws(pushed, n.base);
            sws(pushed + 1u, level);
            sws(pushed + 2u, pending_index.x);
            sws(pushed + 3u, pending_index.y);
            sws(pushed + 4u, 0u);
            sws(pushed + 5u, 0u);
            frames = frames + 1u;
            enter = false;
            continue;
        }
        let top = C_R_PENDING + (frames - 1u) * C_R_PENDING_WORDS;
        let node_base = sw(top);
        let level = sw(top + 1u);
        let index = vec2<u32>(sw(top + 2u), sw(top + 3u));
        let entries = index_node_entries_of_the_header(node_base);
        let next_entry = sw(top + 4u);
        if (next_entry >= entries.entry_count) {
            frames = frames - 1u;
            u_finish_a_node(frames, level, index, sw(top + 5u) == 1u, mode, wanted_ordinal);
            if (frames == 0u) {
                break;
            }
            continue;
        }
        // position_addressed::extent_upper_child_of_entry_key：key 是 (0, 孩子那一格的第一个号, 0)，号整除孩子那一层的跨度，孩子的序号整除扇出是父的序号
        let o = entries.entries_start + next_entry * entries.entry_width;
        let child_level = level - 1u;
        let child = divmod64(extent_key_inode(node_base, o), extent_upper_span(child_level));
        if (!is_zero64(extent_key_locality(node_base, o)) || !is_zero64(extent_key_offset(node_base, o)) || !is_zero64(child.remainder)) {
            return false;
        }
        if (!eq64(divide_by_a_divisor_below_65536(child.quotient, EXTENT_INTERNAL_FANOUT).quotient, index)) {
            return false;
        }
        pending_base = node_base;
        pending_offset = o + 24u;
        pending_level = child_level;
        pending_index = child.quotient;
        enter = true;
    }
    return true;
}

// I-9.40 判一棵 extent 树（根指针在 base 起 root_offset 处）：先走一遍取形状，读不全报射程里判不了（walk.rs 同）；根层级不是 R 判违例；
// 再逐个核没有落在自己那片叶区间里的条目的节点（找罩着它的号）与落在所在那片叶区间之外的号（找它那片叶）。几遍走法在一个循环里、一个调用点
fn u_judge_the_extent_upper_segment_shape(base: u32, root_offset: u32) {
    var mode = UPPER_SHAPE_WALK_TAKES_THE_SHAPE;
    var ordinal = 0u;
    var wanted_span = vec2<u32>(0u, 0u);
    var wanted_index = vec2<u32>(0u, 0u);
    var nodes_without_an_entry_of_their_own_leaves = 0u;
    var entries_outside_their_leaf = 0u;
    var holds = true;
    // 迭代上界：取形状一遍，加上那几个节点与那几个号各两遍；跨轮带着的是这一遍找什么（mode）与第几个（ordinal）
    loop {
        let readable = u_walk_the_extent_upper_segment(base, root_offset, mode, ordinal, wanted_span, wanted_index);
        if (!readable) {
            // 取形状那一遍读不全是 CPU 的 None；之后几遍读同一份字节、同一个读法，只有见过的表或节点栈满了（判不了已记）才走不完
            if (mode == UPPER_SHAPE_WALK_TAKES_THE_SHAPE) {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_40);
            }
            return;
        }
        if (mode == UPPER_SHAPE_WALK_TAKES_THE_SHAPE) {
            // canonical_extent_upper_segment_of：罩得住最大号的最低一层（罩的号数大于它），层级至多 255；没有叶条目是层级 0
            var canonical_root_level = 0u;
            if (u_has_an_entry) {
                var span = u64_of(EXTENT_UPPER_LEAF_INODES);
                loop {
                    if (lt64(u_largest_inode, span) || canonical_root_level >= 255u) {
                        break;
                    }
                    canonical_root_level = canonical_root_level + 1u;
                    span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
                }
            }
            holds = u_root_level == canonical_root_level;
            if (!holds || !u_has_an_entry) {
                break;
            }
            nodes_without_an_entry_of_their_own_leaves = u_nodes_without_an_entry_of_their_own_leaves;
            entries_outside_their_leaf = u_entries_outside_their_leaf;
            ordinal = 0u;
            if (nodes_without_an_entry_of_their_own_leaves > 0u) {
                mode = UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES;
                continue;
            }
            if (entries_outside_their_leaf > 0u) {
                mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF;
                continue;
            }
            break;
        }
        if (mode == UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES) {
            wanted_span = extent_upper_span(u_found_level);
            wanted_index = u_found_index;
            mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION;
            continue;
        }
        if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION) {
            // ①：那个节点的位置不罩 S 里任何一个号，是规范形之外多出来的节点
            if (!u_found) {
                holds = false;
                break;
            }
            ordinal = ordinal + 1u;
            if (ordinal < nodes_without_an_entry_of_their_own_leaves) {
                mode = UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES;
                continue;
            }
            ordinal = 0u;
            if (entries_outside_their_leaf > 0u) {
                mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF;
                continue;
            }
            break;
        }
        if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF) {
            wanted_index = divide_by_a_divisor_below_65536(u_found_inode, EXTENT_UPPER_LEAF_INODES).quotient;
            mode = UPPER_SHAPE_WALK_FINDS_THE_LEAF;
            continue;
        }
        // ②：那个号那片叶的位置没读到，规范形里的这一格少了
        if (!u_found) {
            holds = false;
            break;
        }
        ordinal = ordinal + 1u;
        if (ordinal < entries_outside_their_leaf) {
            mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF;
            continue;
        }
        break;
    }
    judge(INVARIANT_I_9_40, holds);
}

// walk.rs judge_deletion_state_of_the_versions 的一版：树表指针在 (base, offset)。树表 0 条的不判；指针全零、树表读不出、条目宽不到 200 的
// 判不了（记射程里有一部分没判）
// mapping_root_offset 起是这一版的中央映射根指针（候选根取根记录偏移 256，由记录施加出来的那一版取记录新根段偏移 86）：几棵树里豁免三类之外的
// 节点、码 3 容器与数据单元按它认位置（walk.rs judge_deletion_state_of_the_versions 每一版的 central_mapping）
fn d_judge_deletion_state_of_a_version(base: u32, tree_table_pointer_offset: u32, mapping_root_offset: u32) {
    let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
    if (tree_table_pointer.all_zero) {
        d_mark_the_deletion_scan_not_judged();
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok) {
        d_mark_the_deletion_scan_not_judged();
        return;
    }
    if (tt.entry_count == 0u) {
        return;
    }
    if (tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {
        d_mark_the_deletion_scan_not_judged();
        return;
    }
    s_use_the_central_mapping_named_at(base, mapping_root_offset);
    // inode 树不是恰好被一条 deleted_inodes 条目点名、extent 树的头 ID 不是恰好配上一棵 inode 树：判不全（judge_deletion_queue_of_a_version 末尾两段）
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
        if (kind != TREE_KIND_INODE && kind != TREE_KIND_EXTENT) {
            continue;
        }
        var named = 0u;
        for (var other_index = 0u; other_index < tt.entry_count; other_index = other_index + 1u) {
            let other = entry_offset(tt, other_index);
            let other_kind = rd16(tt.base, other + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
            if (kind == TREE_KIND_INODE && other_kind == TREE_KIND_DELETED_INODES
                && eq64(rd64(tt.base, other + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), rd64(tt.base, entry))) {
                named = named + 1u;
            }
            if (kind == TREE_KIND_EXTENT && other_kind == TREE_KIND_INODE
                && eq64(rd64(tt.base, other), rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET))) {
                named = named + 1u;
            }
        }
        if (named != 1u) {
            if (kind == TREE_KIND_INODE) {
                d_mark_the_deletion_queue_scan_not_judged();
            } else {
                d_mark_the_inodes_being_deleted_not_judged();
            }
        }
    }
    // I-3.4（walk.rs judge_pending_deletion_occupancy_of_a_version）：每一棵根指针非零的记账树
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_ACCOUNTING
            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
            continue;
        }
        d_judge_pending_deletion_occupancy_of_an_accounting_tree(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
    }
    // I-9.24、I-9.5、I-4.7、I-4.6（walk.rs judge_deletion_queue_of_a_version）：每一条 deleted_inodes 条目
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_DELETED_INODES) {
            d_judge_deletion_queue_of_an_entry(tt, entry);
        }
    }
    // I-9.40（walk.rs judge_extent_upper_segment_shape_of_a_version）：每一棵根指针非零的 extent 树（种类 1），按这一版的中央映射认位置
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_EXTENT
            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
            continue;
        }
        u_judge_the_extent_upper_segment_shape(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
    }
}

// 删除态扫描：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针），同 I-5.5 那一份清单
fn judge_deletion_state_of_the_versions() {
    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
        let version = d_version_of_the_scan(version_index);
        d_judge_deletion_state_of_a_version(version.base, version.tree_table_offset, version.mapping_root_offset);
    }
}

// I-9.6 墓碑那一半（walk.rs largest_published_tombstone_object_identifier 与 check_pool_image 里接它的那一处）：记账里有 inode 号水位那一行时，
// 所选根（最新的根）的树表里种类 10 的墓碑树整棵读——类型段 0 的条目指下一层码 2 节点（层级 = 父层级 − 1 且 ≥ 1），类型段 1 的指墓碑容器
// （码 3、记录类型 1、记录宽 ≥ 40），按已发布谓词滤容器（码 3：写序实例代号与诞生代号），取已发布墓碑记录对象 ID 的最大值，要小于水位。
// 一条墓碑都没有不判；树表或墓碑树读不全不判
fn judge_inode_number_watermark_above_every_published_tombstone() {
    var watermark = vec2<u32>(0u, 0u);
    if (!c_accounting_row(DELETION_STATISTIC_INODE_WATERMARK, NONE, &watermark)) {
        return;
    }
    let newest_root = checker_root_of(c_newest_index);
    let tree_table_pointer = parse_node_pointer(newest_root.base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    // 树表或墓碑树读不全（walk.rs largest_published_tombstone_object_identifier 交 Err）：check_pool_image 记 I-9.6 射程里有一部分没判
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES)) {
        mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
        return;
    }
    // 墓碑树的节点与墓碑容器按所选根自己的中央映射认位置（walk.rs 同处的 central_mapping）
    s_use_the_central_mapping_named_at(newest_root.base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
    var pending = 0u;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_TOMBSTONE
            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
            continue;
        }
        if (pending >= C_R_MAX_PENDING) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (53u << 8u);
            return;
        }
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        sws(row, tt.base);
        sws(row + 1u, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
        pending = pending + 1u;
    }
    var has_a_published_tombstone = false;
    var largest_object_identifier = vec2<u32>(0u, 0u);
    while (pending > 0u) {
        pending = pending - 1u;
        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
        let n = r_read_located_index_node(parse_node_pointer(sw(row), sw(row + 1u)));
        if (!n.ok || n.level == 0u || n.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
            return;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            let child_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
            let entry_type = rd16(n.base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
            if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
                let child = r_read_located_index_node(child_pointer);
                if (!child.ok || child.level + 1u != n.level || child.level == 0u) {
                    mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
                    return;
                }
                if (pending >= C_R_MAX_PENDING) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (54u << 8u);
                    return;
                }
                let child_row = C_R_PENDING + pending * C_R_PENDING_WORDS;
                sws(child_row, n.base);
                sws(child_row + 1u, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
                pending = pending + 1u;
            } else if (entry_type == PACKED_TYPE_TOMBSTONE) {
                if (child_pointer.all_zero) {
                    mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
                    return;
                }
                let container = parse_packed_unit(r_read_located_unit(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
                if (!container.ok || container.record_type != PACKED_TYPE_TOMBSTONE
                    || container.record_width < DELETION_TOMBSTONE_RECORD_BYTES) {
                    mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
                    return;
                }
                if (!w_published(container.birth_txg, container.instance, false, vec2<u32>(0u, 0u))) {
                    continue;
                }
                for (var r = 0u; r < container.record_count; r = r + 1u) {
                    let object_identifier = rd64(container.base, packed_record_offset(container, r) + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
                    if (!has_a_published_tombstone || lt64(largest_object_identifier, object_identifier)) {
                        largest_object_identifier = object_identifier;
                    }
                    has_a_published_tombstone = true;
                }
            } else {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
                return;
            }
        }
    }
    if (has_a_published_tombstone) {
        judge(INVARIANT_I_9_6, lt64(largest_object_identifier, watermark));
    }
}

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
// container_capacity_knob_wgsl_constants 生成，第 56 批 G6）：没装的判产品那一条（I-9.21、I-9.36），装了的判旋钮那一条（I-9.22、I-9.37），
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
// 有对象之后那三判，第 56 批 G2）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的容器（读容器头，不沿根的条目读）沿叶序第一片容器号是 1、
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
// 每片记录数 ≤ knob_records，最右那片之外每片记录数 == knob_records（开放容器满了才关，D18 已定项 24 第 3 条在旋钮下的那一半，第 57 批 H1）；零根没有对象
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

// I-9.36 / I-9.37（walk.rs judge_lineage_or_retirement_container_capacity_of_a_version，第 53 批，与 I-9.21 / I-9.22 同形）：knob_records 不为 0
// （那一棵装了旋钮，第 56 批 G6）时判 I-9.37（d_judge_containers_under_the_capacity_knob）；为 0 时读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的谱系或退役容器
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
// r_allocation_tree）都把记录数复位、从头写过才读，墓碑与谱系那一遍里没有谁读它；C_ROW_ORDER 在扫描内核里 I-1.8、I-5.4 之后就没有谁读
const D_TOMBSTONE_KEY_WORDS: u32 = 4u;

const D_TOMBSTONE_KEYS: u32 = C_ALLOC_RECORDS;

// 借的两段装得下墓碑总数的上限（解析内核时判，不进展开量）
const_assert DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE * D_TOMBSTONE_KEY_WORDS <= C_MAX_ALLOC_RECORDS * C_ALLOC_RECORD_WORDS;

const_assert DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE <= C_JOURNAL_GROUP_KEYS - C_ROW_ORDER;

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
// 判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条原写「GPU 判器同判 I-9.31，其余回落 CPU」；用户 2026-09-29 定「配了 GPU 就一定要
// GPU 判，不退回 CPU」，GPU 对拍文件头「一个状态都不许判不了」，第 50 批补做把这四条也搬上 GPU，不记判不了。
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
        d_judge_tombstones_and_the_lineage_of_a_version(version.base, version.tree_table_offset, version.txg, is_the_newest_root,
            version.mapping_root_offset);
    }
}

// ───────────────────────── 引用与快照那一截：只读快照与 deadlist（I-3.5 ③ ④ 与逆向、I-3.6、I-3.7）─────────────────────────
// walk.rs judge_snapshots_and_deadlists_of_the_versions（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）：版本清单（与删除态扫描同一份，
// 回退候选集里每条根与由记录施加出来的那几版）里每一版重读自己的树表、只读不判。树表读不了的一版 I-3.6、I-3.7 记射程里判不了；读得了的一版判 I-3.7 的
// 「每个活快照都是这一版之前的一次捕获」；树表单元（第 0 条位置条目的盘、槽、校验和）相同的几版内容相同，I-3.5 ③ ④ 与逆向、I-3.6 只判一次；
// 走完再在 txg 相接的两版之间判 I-3.7 的「快集合只照条款变」。一版的快照与 deadlist 读不全、配不上（deadlist 树条目不止一条、活头两棵树不是恰好一对、
// 快照的码 15 配不上、哪棵树读不全）：I-3.5、I-3.6 记射程里判不了，不判红。读法照 walk.rs 那几个 *_without_judging，判红的那几格由主走读与扫描内核的别处判

// 一版里引用一个单元的是谁（S_KEYED_UNITS 那一字的位）：位 0..S_MAX_LIVE_SNAPSHOTS − 1 是按（S.txg, 树 ID）排的第几个活快照，位 31 是活头
const S_MAX_LIVE_SNAPSHOTS: u32 = 16u;
const S_REFERENCED_BY_THE_HEAD: u32 = 0x80000000u;
const S_MAX_SNAPSHOTS_BEING_DESTROYED: u32 = 16u;
const S_SITE_KEYED_UNITS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (95u << 8u);
const S_SITE_DEADLIST_ENTRIES_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (96u << 8u);
const S_SITE_PENDING_PAST_THE_STACK: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (97u << 8u);
const S_SITE_SNAPSHOTS_PAST_THE_BITS: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (98u << 8u);
const S_SITE_VERSIONS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (99u << 8u);

// r_extent_tree 读到的每个节点与每条非全零数据指针的映射 key 记进 S_KEYED_UNITS 的那一位（walk.rs units_below_the_user_visible_roots_without_judging 的
// extent 树那一半）；0 是不记（别处借 r_extent_tree 的几遍）
var<private> r_unit_keys_go_to: u32;

var<private> s_keyed_unit_count: u32;
var<private> s_keyed_units_are_indexed: bool;
var<private> s_deadlist_entry_count: u32;

// 一版的活快照（按（S.txg, 树 ID）升序）与销毁中的快照：树 ID、S.txg（码 14 条目的诞生 txg）、前驱
var<private> s_live_snapshot_tree: array<vec2<u32>, 16>;
var<private> s_live_snapshot_txg: array<vec2<u32>, 16>;
var<private> s_live_snapshot_previous_txg: array<vec2<u32>, 16>;
var<private> s_snapshot_being_destroyed_tree: array<vec2<u32>, 16>;
var<private> s_snapshot_being_destroyed_txg: array<vec2<u32>, 16>;
var<private> s_snapshot_being_destroyed_previous_txg: array<vec2<u32>, 16>;

fn s_keyed_unit_row(row_index: u32) -> u32 {
    return S_KEYED_UNITS + row_index * S_KEYED_UNIT_WORDS;
}

// S_KEYED_UNITS 里键是 key 的那一行的下标，没有是 NONE；条数过了 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST 才按桶表找（同 ⑬）
fn s_keyed_unit_row_of(key: MappingKey) -> u32 {
    var bucket = mapping_key_first_bucket(key, S_KEYED_UNIT_BUCKET_COUNT);
    let steps = select(s_keyed_unit_count, S_KEYED_UNIT_BUCKET_COUNT, s_keyed_units_are_indexed);
    for (var step = 0u; step < steps; step = step + 1u) {
        var row_index = step;
        if (s_keyed_units_are_indexed) {
            let row_plus_one = sw(S_KEYED_UNIT_BUCKETS + bucket);
            if (row_plus_one == 0u) {
                return NONE;
            }
            row_index = row_plus_one - 1u;
            bucket = (bucket + 1u) & (S_KEYED_UNIT_BUCKET_COUNT - 1u);
        }
        if (same_mapping_key(mapping_key_of_the_words(s_keyed_unit_row(row_index)), key)) {
            return row_index;
        }
    }
    return NONE;
}

fn s_put_the_keyed_unit_into_the_buckets(row_index: u32) {
    var bucket = mapping_key_first_bucket(mapping_key_of_the_words(s_keyed_unit_row(row_index)), S_KEYED_UNIT_BUCKET_COUNT);
    for (var probe = 0u; probe < S_KEYED_UNIT_BUCKET_COUNT; probe = probe + 1u) {
        if (sw(S_KEYED_UNIT_BUCKETS + bucket) == 0u) {
            sws(S_KEYED_UNIT_BUCKETS + bucket, row_index + 1u);
            return;
        }
        bucket = (bucket + 1u) & (S_KEYED_UNIT_BUCKET_COUNT - 1u);
    }
}

// 一个单元（它的映射 key）被 bit 那一方引用：已有这把 key 就并上那一位，没有就记一行
fn s_note_a_unit_key(key: MappingKey, bit: u32) {
    let found = s_keyed_unit_row_of(key);
    if (found != NONE) {
        let row = s_keyed_unit_row(found);
        sws(row + 7u, sw(row + 7u) | bit);
        return;
    }
    if (s_keyed_unit_count >= S_MAX_KEYED_UNITS) {
        undecidable = S_SITE_KEYED_UNITS_PAST_THE_TABLE;
        return;
    }
    let appended = s_keyed_unit_count;
    let row = s_keyed_unit_row(appended);
    put_the_mapping_key_into_the_words(row, key);
    sws(row + 7u, bit);
    sws(row + 8u, 0u);
    sws(row + 9u, NONE);
    s_keyed_unit_count = appended + 1u;
    if (s_keyed_units_are_indexed) {
        s_put_the_keyed_unit_into_the_buckets(appended);
    } else if (s_keyed_unit_count > KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST) {
        for (var bucket = 0u; bucket < S_KEYED_UNIT_BUCKET_COUNT; bucket = bucket + 1u) {
            sws(S_KEYED_UNIT_BUCKETS + bucket, 0u);
        }
        for (var row_to_put = 0u; row_to_put < s_keyed_unit_count; row_to_put = row_to_put + 1u) {
            s_put_the_keyed_unit_into_the_buckets(row_to_put);
        }
        s_keyed_units_are_indexed = true;
    }
}

// walk.rs collect_inode_tree_units_below：inode 树一个内部节点下面每个单元的映射 key 记进 bit 那一位——类型段 2 的叶容器（码 3）、类型段 0 的内部节点
// （码 2，层级 = 父层级 − 1 且 ≥ 1）再往下。条目宽不到 120、子指针全零、孩子读不出、层级不对、类型段不认得：交回 false（判不了）。
// 递归写成显式栈（S_PENDING：字节基、子指针偏移、该有的层级）：上界是节点数
fn s_collect_the_inode_tree_units_below(root: Node, bit: u32) -> bool {
    var pending = 0u;
    var n = root;
    loop {
        if (n.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
            return false;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            let child = parse_node_pointer(n.base, o + 34u);
            if (child.all_zero) {
                return false;
            }
            let entry_type = rd16(n.base, o + 16u);
            if (entry_type == 2u) {
                s_note_a_unit_key(mapping_key_of_a_pointer(child, MAPPING_CLASS_PACKED_RECORD_UNIT), bit);
            } else if (entry_type == 0u) {
                s_note_a_unit_key(mapping_key_of_a_pointer(child, MAPPING_CLASS_INDEX_NODE), bit);
                if (pending >= S_MAX_PENDING) {
                    undecidable = S_SITE_PENDING_PAST_THE_STACK;
                    return false;
                }
                let pending_row = S_PENDING + pending * S_PENDING_WORDS;
                sws(pending_row, n.base);
                sws(pending_row + 1u, o + 34u);
                sws(pending_row + 2u, n.level - 1u);
                pending = pending + 1u;
            } else {
                return false;
            }
        }
        if (pending == 0u) {
            break;
        }
        pending = pending - 1u;
        let pending_row = S_PENDING + pending * S_PENDING_WORDS;
        let child = r_read_located_index_node(parse_node_pointer(sw(pending_row), sw(pending_row + 1u)));
        let expected_level = sw(pending_row + 2u);
        if (!child.ok || child.level != expected_level || child.level == 0u) {
            return false;
        }
        n = child;
    }
    return true;
}

// walk.rs units_below_the_user_visible_roots_without_judging：一对用户可见树的根（inode 树根在 (inode_base, inode_offset)、extent 树根在
// (extent_base, extent_offset) 的那两条指针）下面每个单元的映射 key 记进 bit 那一位：inode 树的根与每个内部节点（码 2）、每片叶容器（码 3），
// extent 树的每个节点（码 2）与每个非全零的数据指针（码 1）。读不出、层级不对、类型段不认得、条目窄于字段表：交回 false
fn s_collect_the_units_below_the_user_visible_roots(inode_base: u32, inode_offset: u32, extent_base: u32, extent_offset: u32, bit: u32) -> bool {
    let inode_root = parse_node_pointer(inode_base, inode_offset);
    if (!inode_root.all_zero) {
        s_note_a_unit_key(mapping_key_of_a_pointer(inode_root, MAPPING_CLASS_INDEX_NODE), bit);
        let root = r_read_located_index_node(inode_root);
        if (!root.ok) {
            return false;
        }
        if (root.level == 0u) {
            if (root.entry_count != 0u) {
                return false;
            }
        } else if (!s_collect_the_inode_tree_units_below(root, bit)) {
            return false;
        }
    }
    if (!parse_node_pointer(extent_base, extent_offset).all_zero) {
        r_unit_keys_go_to = bit;
        let extent_tree_read = r_extent_tree(extent_base, extent_offset);
        r_unit_keys_go_to = 0u;
        if (!extent_tree_read) {
            return false;
        }
    }
    return true;
}

// walk.rs leaf_entries_of_a_multi_level_tree_without_judging（deadlist 树）：key 宽 43、叶条目宽 ≥ 43、内部条目宽 ≥ 43 + 86、孩子层级 = 父层级 − 1、
// 子指针非全零；叶条目（字节基、偏移）记进 S_DEADLIST_ENTRIES。有一处不成立交回 false
fn s_read_the_deadlist_leaf_entries(root: Ptr) -> bool {
    s_deadlist_entry_count = 0u;
    var pending = 0u;
    var current = root;
    var expected_level = NONE;
    loop {
        let n = r_read_located_index_node(current);
        if (!n.ok || n.key_width != DEADLIST_ENTRY_BYTES || (expected_level != NONE && n.level != expected_level)) {
            return false;
        }
        if (n.level == 0u) {
            if (n.entry_width < DEADLIST_ENTRY_BYTES) {
                return false;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                if (s_deadlist_entry_count >= S_MAX_DEADLIST_ENTRIES) {
                    undecidable = S_SITE_DEADLIST_ENTRIES_PAST_THE_TABLE;
                    return false;
                }
                sws(S_DEADLIST_ENTRIES + s_deadlist_entry_count * 2u, n.base);
                sws(S_DEADLIST_ENTRIES + s_deadlist_entry_count * 2u + 1u, entry_offset(n, e));
                s_deadlist_entry_count = s_deadlist_entry_count + 1u;
            }
        } else {
            if (n.entry_width < DEADLIST_ENTRY_BYTES + 86u) {
                return false;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                let child_offset = entry_offset(n, e) + DEADLIST_ENTRY_BYTES;
                if (parse_node_pointer(n.base, child_offset).all_zero) {
                    return false;
                }
                if (pending >= S_MAX_PENDING) {
                    undecidable = S_SITE_PENDING_PAST_THE_STACK;
                    return false;
                }
                let pending_row = S_PENDING + pending * S_PENDING_WORDS;
                sws(pending_row, n.base);
                sws(pending_row + 1u, child_offset);
                sws(pending_row + 2u, n.level - 1u);
                pending = pending + 1u;
            }
        }
        if (pending == 0u) {
            break;
        }
        pending = pending - 1u;
        let pending_row = S_PENDING + pending * S_PENDING_WORDS;
        current = parse_node_pointer(sw(pending_row), sw(pending_row + 1u));
        expected_level = sw(pending_row + 2u);
    }
    return true;
}

// （txg_a, tree_a）严格小于（txg_b, tree_b）
fn s_snapshot_key_less(txg_a: vec2<u32>, tree_a: vec2<u32>, txg_b: vec2<u32>, tree_b: vec2<u32>) -> bool {
    return lt64(txg_a, txg_b) || (eq64(txg_a, txg_b) && lt64(tree_a, tree_b));
}

// walk.rs next_live_side_upper_bound_after：活快照链上排在（txg, tree）之后的第一个活快照的 S.txg（found 为假时是活头那一侧，上界 ∞）
fn s_next_live_side_upper_bound_after(live_count: u32, txg: vec2<u32>, tree: vec2<u32>) -> NearestLiveSnapshotBound {
    var bound: NearestLiveSnapshotBound;
    bound.found = false;
    bound.txg = vec2<u32>(0u, 0u);
    for (var live = 0u; live < live_count; live = live + 1u) {
        if (s_snapshot_key_less(txg, tree, s_live_snapshot_txg[live], s_live_snapshot_tree[live])) {
            bound.found = true;
            bound.txg = s_live_snapshot_txg[live];
            return bound;
        }
    }
    return bound;
}

struct NearestLiveSnapshotBound {
    found: bool,
    txg: vec2<u32>,
};

// walk.rs entry_is_pending_release：这条条目（头、纪元、birth）在某个未完成销毁意图的待放集里——头是活头、纪元 ∈ [D.S.txg, D 之后第一个活快照的 S.txg 或 ∞)、
// birth > D 的前驱
fn s_entry_is_pending_release(head: vec2<u32>, epoch: vec2<u32>, birth: vec2<u32>, head_inode_tree: vec2<u32>, live_count: u32, destroyed_count: u32) -> bool {
    if (!eq64(head, head_inode_tree)) {
        return false;
    }
    for (var destroyed = 0u; destroyed < destroyed_count; destroyed = destroyed + 1u) {
        let bound = s_next_live_side_upper_bound_after(live_count, s_snapshot_being_destroyed_txg[destroyed], s_snapshot_being_destroyed_tree[destroyed]);
        if (le64(s_snapshot_being_destroyed_txg[destroyed], epoch) && (!bound.found || lt64(epoch, bound.txg))
            && lt64(s_snapshot_being_destroyed_previous_txg[destroyed], birth)) {
            return true;
        }
    }
    return false;
}

fn s_mark_the_deadlist_of_a_version_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_3_5);
    mark_part_of_the_range_not_judged(INVARIANT_I_3_6);
}

// walk.rs judge_the_deadlist_against_the_head_and_the_live_snapshots：
// ④ deadlist 每一条钉的单元活头都不引用；逆向：每一条钉的单元有一个活快照引用、或在待放集里；I-3.6 每一条的 birth 不大于它归属那一侧的前驱（待放的豁免，
// 头不是活头、纪元落不进恰好一侧的那一条判不了）；③ 活快照引用而活头不引用的每个单元，deadlist 里恰好一条钉它、头是活头、纪元落在引用它的最晚那个
// 活快照的 S.txg 与链上下一侧的上界之间。一条条目都没有时 I-3.6 判一次、成立
fn s_judge_the_deadlist_against_the_head_and_the_live_snapshots(head_inode_tree: vec2<u32>, head_previous_snapshot_txg: vec2<u32>, live_count: u32,
    destroyed_count: u32) {
    // deadlist 里钉每个单元的有几条、最后一条是哪一条（③ 要「恰好一条」）
    for (var entry_index = 0u; entry_index < s_deadlist_entry_count; entry_index = entry_index + 1u) {
        let base = sw(S_DEADLIST_ENTRIES + entry_index * 2u);
        let o = sw(S_DEADLIST_ENTRIES + entry_index * 2u + 1u);
        let found = s_keyed_unit_row_of(mapping_key_of_the_bytes(base, o + DEADLIST_ENTRY_MAPPING_KEY_OFFSET));
        if (found != NONE) {
            let row = s_keyed_unit_row(found);
            sws(row + 8u, sw(row + 8u) + 1u);
            sws(row + 9u, entry_index);
        }
    }
    if (s_deadlist_entry_count == 0u) {
        judge(INVARIANT_I_3_6, true);
    }
    for (var entry_index = 0u; entry_index < s_deadlist_entry_count; entry_index = entry_index + 1u) {
        let base = sw(S_DEADLIST_ENTRIES + entry_index * 2u);
        let o = sw(S_DEADLIST_ENTRIES + entry_index * 2u + 1u);
        let key = mapping_key_of_the_bytes(base, o + DEADLIST_ENTRY_MAPPING_KEY_OFFSET);
        let head = rd64(base, o + DEADLIST_ENTRY_HEAD_OFFSET);
        let epoch = rd64(base, o + DEADLIST_ENTRY_EPOCH_OFFSET);
        let found = s_keyed_unit_row_of(key);
        var referenced_by = 0u;
        if (found != NONE) {
            referenced_by = sw(s_keyed_unit_row(found) + 7u);
        }
        let is_pending_release = s_entry_is_pending_release(head, epoch, key.birth_txg, head_inode_tree, live_count, destroyed_count);
        judge(INVARIANT_I_3_5, (referenced_by & S_REFERENCED_BY_THE_HEAD) == 0u);
        judge(INVARIANT_I_3_5, (referenced_by & ~S_REFERENCED_BY_THE_HEAD) != 0u || is_pending_release);
        if (!eq64(head, head_inode_tree)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_6);
            continue;
        }
        // 归哪一侧：活快照 [前驱, S.txg)、活头 [前驱, ∞)，要恰好一侧
        var owning_sides = 0u;
        var owning_side_previous_snapshot_txg = vec2<u32>(0u, 0u);
        for (var live = 0u; live < live_count; live = live + 1u) {
            if (le64(s_live_snapshot_previous_txg[live], epoch) && lt64(epoch, s_live_snapshot_txg[live])) {
                owning_sides = owning_sides + 1u;
                owning_side_previous_snapshot_txg = s_live_snapshot_previous_txg[live];
            }
        }
        if (le64(head_previous_snapshot_txg, epoch)) {
            owning_sides = owning_sides + 1u;
            owning_side_previous_snapshot_txg = head_previous_snapshot_txg;
        }
        if (owning_sides != 1u) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_6);
            continue;
        }
        judge(INVARIANT_I_3_6, is_pending_release || le64(key.birth_txg, owning_side_previous_snapshot_txg));
    }
    // ③：活快照引用而活头不引用的每个单元
    for (var row_index = 0u; row_index < s_keyed_unit_count; row_index = row_index + 1u) {
        let row = s_keyed_unit_row(row_index);
        let referenced_by = sw(row + 7u);
        let live_snapshots_referencing_it = referenced_by & ~S_REFERENCED_BY_THE_HEAD;
        if (live_snapshots_referencing_it == 0u || (referenced_by & S_REFERENCED_BY_THE_HEAD) != 0u) {
            continue;
        }
        let latest = firstLeadingBit(live_snapshots_referencing_it);
        let bound = s_next_live_side_upper_bound_after(live_count, s_live_snapshot_txg[latest], s_live_snapshot_tree[latest]);
        var is_in_exactly_its_deadlist = false;
        if (sw(row + 8u) == 1u) {
            let entry_index = sw(row + 9u);
            let base = sw(S_DEADLIST_ENTRIES + entry_index * 2u);
            let o = sw(S_DEADLIST_ENTRIES + entry_index * 2u + 1u);
            let epoch = rd64(base, o + DEADLIST_ENTRY_EPOCH_OFFSET);
            is_in_exactly_its_deadlist = eq64(rd64(base, o + DEADLIST_ENTRY_HEAD_OFFSET), head_inode_tree)
                && le64(s_live_snapshot_txg[latest], epoch) && (!bound.found || lt64(epoch, bound.txg));
        }
        judge(INVARIANT_I_3_5, is_in_exactly_its_deadlist);
    }
}

// walk.rs deadlist_and_snapshot_reach_of_a_version 连同判那一步：一版（树表读得出、条目不窄）的快照、活头两棵树与 deadlist 读出来再判
fn s_judge_the_snapshots_and_the_deadlist_of_a_tree_table(tt: Node) {
    var deadlist_tree_entries = 0u;
    var deadlist_tree_entry = NONE;
    var snapshot_entries = 0u;
    var head_inode_tree_entries = 0u;
    var head_inode_tree_entry = NONE;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let kind = rd16(tt.base, entry_offset(tt, e) + 10u);
        if (kind == TREE_KIND_DEADLIST) {
            deadlist_tree_entries = deadlist_tree_entries + 1u;
            deadlist_tree_entry = e;
        } else if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
            snapshot_entries = snapshot_entries + 1u;
        } else if (kind == TREE_KIND_INODE) {
            head_inode_tree_entries = head_inode_tree_entries + 1u;
            head_inode_tree_entry = e;
        }
    }
    if (deadlist_tree_entries > 1u) {
        s_mark_the_deadlist_of_a_version_not_judged();
        return;
    }
    var deadlist_root = zero_pointer();
    var has_a_deadlist_root = false;
    if (deadlist_tree_entries == 1u) {
        deadlist_root = parse_node_pointer(tt.base, entry_offset(tt, deadlist_tree_entry) + 14u);
        has_a_deadlist_root = !deadlist_root.all_zero;
    }
    // 一个快照都没有、deadlist 一条都没有：I-3.6 判一次、成立（空 deadlist，同 I-9.24 空队列报成立）；I-3.5 ③ ④ 没有对象
    if (snapshot_entries == 0u && !has_a_deadlist_root) {
        judge(INVARIANT_I_3_6, true);
        return;
    }
    if (head_inode_tree_entries != 1u) {
        s_mark_the_deadlist_of_a_version_not_judged();
        return;
    }
    let head_o = entry_offset(tt, head_inode_tree_entry);
    let head_inode_tree = rd64(tt.base, head_o);
    var head_extent_tree_entries = 0u;
    var head_extent_tree_entry = NONE;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) == TREE_KIND_EXTENT && eq64(rd64(tt.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), head_inode_tree)) {
            head_extent_tree_entries = head_extent_tree_entries + 1u;
            head_extent_tree_entry = e;
        }
    }
    if (head_extent_tree_entries != 1u) {
        s_mark_the_deadlist_of_a_version_not_judged();
        return;
    }
    s_deadlist_entry_count = 0u;
    if (has_a_deadlist_root && !s_read_the_deadlist_leaf_entries(deadlist_root)) {
        s_mark_the_deadlist_of_a_version_not_judged();
        return;
    }
    s_keyed_unit_count = 0u;
    s_keyed_units_are_indexed = false;
    if (!s_collect_the_units_below_the_user_visible_roots(tt.base, head_o + 14u, tt.base, entry_offset(tt, head_extent_tree_entry) + 14u,
        S_REFERENCED_BY_THE_HEAD)) {
        s_mark_the_deadlist_of_a_version_not_judged();
        return;
    }
    // 快照按（S.txg, 树 ID）升序（同键的按树表次序）：每一圈选比上一圈选中的大的里最小的一条
    var live_count = 0u;
    var destroyed_count = 0u;
    var have_previous_pick = false;
    var previous_txg = vec2<u32>(0u, 0u);
    var previous_tree = vec2<u32>(0u, 0u);
    var previous_entry = 0u;
    for (var round = 0u; round < snapshot_entries; round = round + 1u) {
        var pick = NONE;
        var pick_txg = vec2<u32>(0u, 0u);
        var pick_tree = vec2<u32>(0u, 0u);
        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
            let o = entry_offset(tt, e);
            if (rd16(tt.base, o + 10u) != TREE_KIND_SNAPSHOT_INODE_ROOT) {
                continue;
            }
            let txg = rd64(tt.base, o + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET);
            let tree = rd64(tt.base, o);
            let after_the_previous_pick = !have_previous_pick || s_snapshot_key_less(previous_txg, previous_tree, txg, tree)
                || (eq64(previous_txg, txg) && eq64(previous_tree, tree) && previous_entry < e);
            let before_the_pick = pick == NONE || s_snapshot_key_less(txg, tree, pick_txg, pick_tree);
            if (after_the_previous_pick && before_the_pick) {
                pick = e;
                pick_txg = txg;
                pick_tree = tree;
            }
        }
        have_previous_pick = true;
        previous_txg = pick_txg;
        previous_tree = pick_tree;
        previous_entry = pick;
        let o = entry_offset(tt, pick);
        let previous_snapshot_txg = rd64(tt.base, o + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET);
        if ((rd16(tt.base, o + TREE_TABLE_ENTRY_FLAGS_OFFSET) & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED) != 0u) {
            if (destroyed_count >= S_MAX_SNAPSHOTS_BEING_DESTROYED) {
                undecidable = S_SITE_SNAPSHOTS_PAST_THE_BITS;
                return;
            }
            s_snapshot_being_destroyed_tree[destroyed_count] = pick_tree;
            s_snapshot_being_destroyed_txg[destroyed_count] = pick_txg;
            s_snapshot_being_destroyed_previous_txg[destroyed_count] = previous_snapshot_txg;
            destroyed_count = destroyed_count + 1u;
            continue;
        }
        // 活快照：头 ID 配它的码 15 恰好一条
        var paired_extent_roots = 0u;
        var paired_extent_root = NONE;
        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
            let extent_o = entry_offset(tt, e);
            if (rd16(tt.base, extent_o + 10u) == TREE_KIND_SNAPSHOT_EXTENT_ROOT && eq64(rd64(tt.base, extent_o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), pick_tree)) {
                paired_extent_roots = paired_extent_roots + 1u;
                paired_extent_root = e;
            }
        }
        if (paired_extent_roots != 1u) {
            s_mark_the_deadlist_of_a_version_not_judged();
            return;
        }
        if (live_count >= S_MAX_LIVE_SNAPSHOTS) {
            undecidable = S_SITE_SNAPSHOTS_PAST_THE_BITS;
            return;
        }
        s_live_snapshot_tree[live_count] = pick_tree;
        s_live_snapshot_txg[live_count] = pick_txg;
        s_live_snapshot_previous_txg[live_count] = previous_snapshot_txg;
        if (!s_collect_the_units_below_the_user_visible_roots(tt.base, o + 14u, tt.base, entry_offset(tt, paired_extent_root) + 14u, 1u << live_count)) {
            s_mark_the_deadlist_of_a_version_not_judged();
            return;
        }
        live_count = live_count + 1u;
    }
    s_judge_the_deadlist_against_the_head_and_the_live_snapshots(head_inode_tree, rd64(tt.base, head_o + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET),
        live_count, destroyed_count);
}

// walk.rs judge_live_snapshots_are_captures_before_their_version：这一版每一个活快照诞生 txg 不大于这一版的 txg、两条根指针（非全零的；码 15 恰好一条配它时
// 才算它那一条）头里的出生 txg 不大于诞生 txg。一个活快照都没有也成立
fn s_live_snapshots_are_captures_before_the_version(tt: Node, version_txg: vec2<u32>) -> bool {
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) != TREE_KIND_SNAPSHOT_INODE_ROOT
            || (rd16(tt.base, o + TREE_TABLE_ENTRY_FLAGS_OFFSET) & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED) != 0u) {
            continue;
        }
        let snapshot_txg = rd64(tt.base, o + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET);
        let tree = rd64(tt.base, o);
        var has_a_root = false;
        var latest_root_birth_txg = vec2<u32>(0u, 0u);
        let inode_root = parse_node_pointer(tt.base, o + 14u);
        if (!inode_root.all_zero) {
            has_a_root = true;
            latest_root_birth_txg = inode_root.birth_txg;
        }
        var paired_extent_roots = 0u;
        var paired_extent_root = NONE;
        for (var other = 0u; other < tt.entry_count; other = other + 1u) {
            let other_o = entry_offset(tt, other);
            if (rd16(tt.base, other_o + 10u) == TREE_KIND_SNAPSHOT_EXTENT_ROOT && eq64(rd64(tt.base, other_o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), tree)) {
                paired_extent_roots = paired_extent_roots + 1u;
                paired_extent_root = other;
            }
        }
        if (paired_extent_roots == 1u) {
            let extent_root = parse_node_pointer(tt.base, entry_offset(tt, paired_extent_root) + 14u);
            if (!extent_root.all_zero && (!has_a_root || lt64(latest_root_birth_txg, extent_root.birth_txg))) {
                has_a_root = true;
                latest_root_birth_txg = extent_root.birth_txg;
            }
        }
        if (!le64(snapshot_txg, version_txg) || (has_a_root && !le64(latest_root_birth_txg, snapshot_txg))) {
            return false;
        }
    }
    return true;
}

// 版本表一行记下的树表里树号是 tree 的码 14 条目里代表那一个（walk.rs 按（S.txg, 树 ID）升序收进以树号为键的表，同一树号后收的盖先收的）：
// 诞生 txg 最大的那一条，同 txg 的取树表里靠后的；没有是 NONE
fn s_snapshot_entry_of_the_tree(version_row: u32, tree: vec2<u32>) -> u32 {
    var picked = NONE;
    var picked_txg = vec2<u32>(0u, 0u);
    for (var e = 0u; e < sw(version_row + 6u); e = e + 1u) {
        let o = sw(version_row + 4u) + e * sw(version_row + 5u);
        if (rd16(sw(version_row + 3u), o + 10u) != TREE_KIND_SNAPSHOT_INODE_ROOT || !eq64(rd64(sw(version_row + 3u), o), tree)) {
            continue;
        }
        let txg = rd64(sw(version_row + 3u), o + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET);
        if (picked == NONE || le64(picked_txg, txg)) {
            picked = e;
            picked_txg = txg;
        }
    }
    return picked;
}

fn s_snapshot_entry_txg(version_row: u32, entry: u32) -> vec2<u32> {
    return rd64(sw(version_row + 3u), sw(version_row + 4u) + entry * sw(version_row + 5u) + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET);
}

fn s_snapshot_entry_is_live(version_row: u32, entry: u32) -> bool {
    return (rd16(sw(version_row + 3u), sw(version_row + 4u) + entry * sw(version_row + 5u) + TREE_TABLE_ENTRY_FLAGS_OFFSET)
        & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED) == 0u;
}

fn s_snapshot_entry_tree(version_row: u32, entry: u32) -> vec2<u32> {
    return rd64(sw(version_row + 3u), sw(version_row + 4u) + entry * sw(version_row + 5u));
}

// walk.rs snapshot_changes_against_the_clauses_between：txg 相接的两版之间快照集合只照条款变——前一版的快照在后一版要么还在且诞生 txg 不变（活的可以带上
// 销毁中记号，销毁中的不许又活回来），要么没了（只有销毁中的才许没）；后一版多出来的只许是诞生 txg 等于后一版 txg 的活快照
fn s_snapshot_changes_follow_the_clauses(earlier_row: u32, later_row: u32, later_txg: vec2<u32>) -> bool {
    for (var e = 0u; e < sw(earlier_row + 6u); e = e + 1u) {
        let o = sw(earlier_row + 4u) + e * sw(earlier_row + 5u);
        if (rd16(sw(earlier_row + 3u), o + 10u) != TREE_KIND_SNAPSHOT_INODE_ROOT) {
            continue;
        }
        let tree = s_snapshot_entry_tree(earlier_row, e);
        if (s_snapshot_entry_of_the_tree(earlier_row, tree) != e) {
            continue;
        }
        let after = s_snapshot_entry_of_the_tree(later_row, tree);
        if (after == NONE) {
            if (s_snapshot_entry_is_live(earlier_row, e)) {
                return false;
            }
            continue;
        }
        if (!eq64(s_snapshot_entry_txg(later_row, after), s_snapshot_entry_txg(earlier_row, e))) {
            return false;
        }
        if (!s_snapshot_entry_is_live(earlier_row, e) && s_snapshot_entry_is_live(later_row, after)) {
            return false;
        }
    }
    for (var e = 0u; e < sw(later_row + 6u); e = e + 1u) {
        let o = sw(later_row + 4u) + e * sw(later_row + 5u);
        if (rd16(sw(later_row + 3u), o + 10u) != TREE_KIND_SNAPSHOT_INODE_ROOT) {
            continue;
        }
        let tree = s_snapshot_entry_tree(later_row, e);
        if (s_snapshot_entry_of_the_tree(later_row, tree) != e || s_snapshot_entry_of_the_tree(earlier_row, tree) != NONE) {
            continue;
        }
        if (!s_snapshot_entry_is_live(later_row, e) || !eq64(s_snapshot_entry_txg(later_row, e), later_txg)) {
            return false;
        }
    }
    return true;
}

// walk.rs judge_snapshots_and_deadlists_of_the_versions（见这一节开头）
fn judge_snapshots_and_deadlists_of_the_versions() {
    let version_total = c_candidate_count + c_version_count;
    if (version_total > S_MAX_VERSIONS) {
        undecidable = S_SITE_VERSIONS_PAST_THE_TABLE;
        return;
    }
    for (var version_index = 0u; version_index < version_total; version_index = version_index + 1u) {
        let version = d_version_of_the_scan(version_index);
        let row = S_VERSIONS + version_index * S_VERSION_WORDS;
        sws(row, 0u);
        sws(row + 1u, version.txg.x);
        sws(row + 2u, version.txg.y);
        let pointer = parse_node_pointer(version.base, version.tree_table_offset);
        var tt = bad_node();
        if (!pointer.all_zero) {
            tt = r_read_index_node(pointer);
        }
        if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES)) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_6);
            mark_part_of_the_range_not_judged(INVARIANT_I_3_7);
            continue;
        }
        sws(row, 1u);
        sws(row + 3u, tt.base);
        sws(row + 4u, tt.entries_start);
        sws(row + 5u, tt.entry_width);
        sws(row + 6u, tt.entry_count);
        sws(row + 7u, pointer.device0);
        sws(row + 8u, pointer.slot0.x);
        sws(row + 9u, pointer.slot0.y);
        sws(row + 10u, pointer.sum0);
        judge(INVARIANT_I_3_7, s_live_snapshots_are_captures_before_the_version(tt, version.txg));
        // 树表单元相同的几版内容相同：deadlist 那几条只判第一次读到它的那一版
        var judged_before = false;
        for (var earlier = 0u; earlier < version_index; earlier = earlier + 1u) {
            let earlier_row = S_VERSIONS + earlier * S_VERSION_WORDS;
            if (sw(earlier_row) == 1u && sw(earlier_row + 7u) == pointer.device0 && sw(earlier_row + 8u) == pointer.slot0.x
                && sw(earlier_row + 9u) == pointer.slot0.y && sw(earlier_row + 10u) == pointer.sum0) {
                judged_before = true;
            }
        }
        if (!judged_before) {
            // 活头、快照与 deadlist 几棵树里豁免三类之外的单元按这一版自己的中央映射认位置（walk.rs judge_snapshots_and_deadlists_of_the_versions
            // 的 central_mapping；树表单元相同的几版只判第一版，用的是第一版的映射）
            s_use_the_central_mapping_named_at(version.base, version.mapping_root_offset);
            s_judge_the_snapshots_and_the_deadlist_of_a_tree_table(tt);
        }
    }
    // I-3.7 跨版本那一格：txg 相接（后一版 = 前一版 + 1）、各恰好一版树表读得出的两版之间
    for (var later = 0u; later < version_total; later = later + 1u) {
        let later_row = S_VERSIONS + later * S_VERSION_WORDS;
        if (sw(later_row) != 1u) {
            continue;
        }
        let later_txg = vec2<u32>(sw(later_row + 1u), sw(later_row + 2u));
        if (is_zero64(later_txg)) {
            continue;
        }
        let earlier_txg = sub64(later_txg, u64_of(1u));
        var versions_at_the_later_txg = 0u;
        var versions_at_the_earlier_txg = 0u;
        var earlier_version = NONE;
        for (var other = 0u; other < version_total; other = other + 1u) {
            let other_row = S_VERSIONS + other * S_VERSION_WORDS;
            if (sw(other_row) != 1u) {
                continue;
            }
            let other_txg = vec2<u32>(sw(other_row + 1u), sw(other_row + 2u));
            if (eq64(other_txg, later_txg)) {
                versions_at_the_later_txg = versions_at_the_later_txg + 1u;
            } else if (eq64(other_txg, earlier_txg)) {
                versions_at_the_earlier_txg = versions_at_the_earlier_txg + 1u;
                earlier_version = other;
            }
        }
        if (versions_at_the_later_txg != 1u || versions_at_the_earlier_txg != 1u) {
            continue;
        }
        judge(INVARIANT_I_3_7, s_snapshot_changes_follow_the_clauses(S_VERSIONS + earlier_version * S_VERSION_WORDS, later_row, later_txg));
    }
}

// ───────────────────────── 引用与快照那一截之后：整理意图与搬迁（I-4.4、I-5.6、I-3.15、I-1.14）─────────────────────────
// walk.rs check_pool_image 末尾那几遍（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3 的 N2、N3、N4、N6）：版本清单与删除态扫描同一份
// （回退候选集里每条根，再是由记录施加出来的那几版），每一版重读自己的树表、分配记录、中央映射与 logged_ops，只读不判（判红的那几格由主走读与扫描内核别处判）。
// I-4.4 / I-5.6：每一版 logged_ops 树（树表种类 12）的每条条目（walk.rs judge_compaction_intents_of_the_versions）；I-1.14：每一版中央映射里每把 key 落在哪一组
// 位置条目，同一把 key 落在不止一组时各组读第一份对得上校验和的副本逐字节比（walk.rs judge_unit_bytes_of_each_mapping_key_across_the_versions）；
// I-3.15：每一版每条未释放记录罩住的单元，由根记录或父指针直接指着、或这一版映射里有一把与单元头相符的 key 指着这一槽，对不上的按单元归组、
// 隔离的单元不判（walk.rs judge_unreleased_allocation_records_against_the_central_mapping；四条根指针逐字节相同的几版只判第一版）。
// 几张表借 S_KEYED_UNITS 起那一段（引用与快照那一截判完之后才用，那一截不再读它们；扫描内核主入口的 I-3.16 也借过这一段、在这一截之前用完）

// 根记录里四条根指针的偏移（walk.rs 的 ROOT_RECORD_TREE_TABLE_POINTER、ROOT_RECORD_INSTANCE_TABLE_POINTER、ROOT_RECORD_MAPPING_ROOT_POINTER、
// ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT）与新根段里映射根指针的段内偏移（NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER；树表指针在段首）
const G_ROOT_RECORD_TREE_TABLE_POINTER: u32 = 36u;
const G_ROOT_RECORD_INSTANCE_TABLE_POINTER: u32 = 170u;
const G_ROOT_RECORD_MAPPING_ROOT_POINTER: u32 = 256u;
const G_ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: u32 = 342u;
const G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: u32 = 86u;
// 一条节点指针 86 字节；第 0 条位置条目的整单元校验和在指针偏移 60（版本去重先比这四个字节）
const G_NODE_POINTER_BYTES: u32 = 86u;
const G_NODE_POINTER_FIRST_LOCATION_CHECKSUM_OFFSET: u32 = 60u;

// I-3.15 一版的落点表：每行 盘、槽 lo/hi、名目（这一版映射叶的第几行，或 G_NAMED_DIRECTLY：根记录或父指针直接指着）、它记在哪个桶；
// 映射叶每条条目的两个落点与直接指着的每条指针的两个落点都记，条数上限取一版中央映射那张表（C_MAX_VERSION_MAPPING_ROWS）两份再加同样多的直接指针
const G_PLACEMENT_WORDS: u32 = 5u;
const G_MAX_PLACEMENTS: u32 = 4u * C_MAX_VERSION_MAPPING_ROWS;
const G_NAMED_DIRECTLY: u32 = 0xFFFFFFFFu;
const G_PLACEMENTS: u32 = S_KEYED_UNITS;
const G_PLACEMENT_BUCKET_COUNT: u32 = 2u * G_MAX_PLACEMENTS;
const G_PLACEMENT_BUCKETS: u32 = G_PLACEMENTS + G_PLACEMENT_WORDS * G_MAX_PLACEMENTS;
// 一版 logged_ops 树的叶条目（字节基、条目偏移）：一个码 2 节点装 369 条，第一版全池至多一条整理意图
const G_MAX_LOGGED_OPERATION_ENTRIES: u32 = 512u;
const G_LOGGED_OPERATION_ENTRIES: u32 = G_PLACEMENT_BUCKETS + G_PLACEMENT_BUCKET_COUNT;
// 读多层码 2 树（logged_ops 树、中央映射树与根记录持有的分配记录树的节点）的显式栈：字节基、子指针偏移、该有的层级
const G_PENDING_WORDS: u32 = 3u;
const G_MAX_PENDING: u32 = 1024u;
const G_PENDING: u32 = G_LOGGED_OPERATION_ENTRIES + 2u * G_MAX_LOGGED_OPERATION_ENTRIES;
// I-1.14 跨版本那张表：每行一把 key（7 字）落在的一组位置条目（盘、槽 lo/hi、校验和各两份，8 字），再三字是那一组读过没有（0 没读、1 读不出、
// 2 读出来了）、读出来那一份的字节基与版本号；(key, 位置条目) 相同的只记一行，桶表按 key 散列
const G_KEY_GROUP_WORDS: u32 = 18u;
const G_MAX_KEY_GROUPS: u32 = 4096u;
const G_KEY_GROUPS: u32 = G_PENDING + G_PENDING_WORDS * G_MAX_PENDING;
const G_KEY_GROUP_BUCKET_COUNT: u32 = 2u * G_MAX_KEY_GROUPS;
const G_KEY_GROUP_BUCKETS: u32 = G_KEY_GROUPS + G_KEY_GROUP_WORDS * G_MAX_KEY_GROUPS;
const G_KEY_GROUP_NOT_READ_YET: u32 = 0u;
const G_KEY_GROUP_UNREADABLE: u32 = 1u;
const G_KEY_GROUP_READ: u32 = 2u;

// 借的那一段装得下（解析内核时判，不进展开量）
const_assert G_KEY_GROUP_BUCKETS + G_KEY_GROUP_BUCKET_COUNT <= SCRATCH_END;

// 这一节判不了的站点号（「条目太多」一类，与别处不撞）
const G_SITE_PLACEMENTS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (101u << 8u);
const G_SITE_LOGGED_OPERATION_ENTRIES_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (102u << 8u);
const G_SITE_PENDING_PAST_THE_STACK: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (103u << 8u);
const G_SITE_KEY_GROUPS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (104u << 8u);

// logged_ops 条目（key 10 = 种类 2 + 签发 txg 8；value 34 = R 的设备 4、起始槽 6、槽数 2、c0 8、segs0 8、补零 6）里各段的偏移（walk.rs 的
// LOGGED_OPERATION_ISSUED_TXG_OFFSET 与 COMPACTION_INTENT_*_OFFSET_IN_THE_VALUE；判决 G1 字段表）与整理那一种的种类码
const G_LOGGED_OPERATION_ISSUED_TXG_OFFSET: u32 = 2u;
const G_COMPACTION_INTENT_REGION_DEVICE_OFFSET_IN_THE_VALUE: u32 = 0u;
const G_COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_VALUE: u32 = 4u;
const G_COMPACTION_INTENT_REGION_SLOT_COUNT_OFFSET_IN_THE_VALUE: u32 = 10u;
const G_COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE: u32 = 12u;
const G_COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE: u32 = 28u;
const G_COMPACTION_INTENT_PADDING_BYTES: u32 = 6u;
const G_LOGGED_OPERATION_KIND_COMPACTION: u32 = 1u;

var<private> g_placement_count: u32;
var<private> g_noting_the_instance_table_pages: bool;
var<private> g_logged_operation_entry_count: u32;
var<private> g_key_group_count: u32;

// 版本清单里的一版：根记录（或记录新根段）所在的字节基、四条根指针的偏移、txg
struct VersionWithItsRootPointersOfTheScan {
    base: u32,
    tree_table_offset: u32,
    instance_table_offset: u32,
    mapping_root_offset: u32,
    allocation_root_offset: u32,
    txg: vec2<u32>,
};

fn g_version_with_its_root_pointers(version_index: u32) -> VersionWithItsRootPointersOfTheScan {
    var version: VersionWithItsRootPointersOfTheScan;
    if (version_index < c_candidate_count) {
        let root = checker_root_of(sw(C_CANDIDATES + version_index));
        version.base = root.base;
        version.tree_table_offset = G_ROOT_RECORD_TREE_TABLE_POINTER;
        version.instance_table_offset = G_ROOT_RECORD_INSTANCE_TABLE_POINTER;
        version.mapping_root_offset = G_ROOT_RECORD_MAPPING_ROOT_POINTER;
        version.allocation_root_offset = G_ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT;
        version.txg = root.txg;
    } else {
        let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
        version.base = record.base;
        version.tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        version.instance_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
        version.mapping_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER;
        version.allocation_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
        version.txg = record.txg;
    }
    return version;
}

// 一版的第 which 条根指针（0 树表、1 实例表第 0 片、2 映射根、3 根记录持有的分配记录树根）的偏移
fn g_root_pointer_offset(version: VersionWithItsRootPointersOfTheScan, which: u32) -> u32 {
    return select(select(select(version.tree_table_offset, version.instance_table_offset, which == 1u), version.mapping_root_offset, which == 2u),
        version.allocation_root_offset, which == 3u);
}

// 第 version_index 版的四条根指针与更早某一版逐字节相同（walk.rs 按四条根指针的字节去重，I-3.15 只判第一版）：先比四条指针第 0 条位置条目的校验和，
// 都相同再逐字节比
fn g_root_pointers_seen_in_an_earlier_version(version_index: u32) -> bool {
    let version = g_version_with_its_root_pointers(version_index);
    for (var earlier_index = 0u; earlier_index < version_index; earlier_index = earlier_index + 1u) {
        let earlier = g_version_with_its_root_pointers(earlier_index);
        var same = true;
        for (var which = 0u; which < 4u && same; which = which + 1u) {
            same = rd32(version.base, g_root_pointer_offset(version, which) + G_NODE_POINTER_FIRST_LOCATION_CHECKSUM_OFFSET)
                == rd32(earlier.base, g_root_pointer_offset(earlier, which) + G_NODE_POINTER_FIRST_LOCATION_CHECKSUM_OFFSET);
        }
        for (var which = 0u; which < 4u && same; which = which + 1u) {
            same = bytes_equal(version.base, g_root_pointer_offset(version, which), earlier.base, g_root_pointer_offset(earlier, which),
                G_NODE_POINTER_BYTES);
        }
        if (same) {
            return true;
        }
    }
    return false;
}

// ── I-4.4 / I-5.6（walk.rs judge_compaction_intents_of_the_versions）

// walk.rs leaf_entries_of_a_multi_level_tree_without_judging（logged_ops 树）：key 宽 10、叶条目宽 ≥ 44、内部条目宽 ≥ 10 + 86、孩子层级 = 父层级 − 1、
// 子指针非全零，有一处不成立交回 false。照 CPU 递归的次序走（先序、条目次序）：孩子按条目倒序压栈、出栈就是正序，子指针全零走到它那一刻才拒
// （它之前的兄弟整棵先走完），读不全时已收的叶条目与 CPU 那一份相同。叶条目（字节基、偏移）接着记进 G_LOGGED_OPERATION_ENTRIES
fn g_read_the_logged_operation_entries(root: Ptr) -> bool {
    var pending = 0u;
    var current = root;
    var expected_level = NONE;
    loop {
        if (current.all_zero) {
            return false;
        }
        let n = r_read_located_index_node(current);
        if (!n.ok || n.key_width != LOGGED_OPERATIONS_KEY_BYTES || (expected_level != NONE && n.level != expected_level)) {
            return false;
        }
        if (n.level == 0u) {
            if (n.entry_width < LOGGED_OPERATIONS_ENTRY_BYTES) {
                return false;
            }
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                if (g_logged_operation_entry_count >= G_MAX_LOGGED_OPERATION_ENTRIES) {
                    undecidable = G_SITE_LOGGED_OPERATION_ENTRIES_PAST_THE_TABLE;
                    return false;
                }
                sws(G_LOGGED_OPERATION_ENTRIES + g_logged_operation_entry_count * 2u, n.base);
                sws(G_LOGGED_OPERATION_ENTRIES + g_logged_operation_entry_count * 2u + 1u, entry_offset(n, e));
                g_logged_operation_entry_count = g_logged_operation_entry_count + 1u;
            }
        } else {
            if (n.entry_width < LOGGED_OPERATIONS_KEY_BYTES + 86u) {
                return false;
            }
            for (var remaining = n.entry_count; remaining > 0u; remaining = remaining - 1u) {
                if (pending >= G_MAX_PENDING) {
                    undecidable = G_SITE_PENDING_PAST_THE_STACK;
                    return false;
                }
                let pending_row = G_PENDING + pending * G_PENDING_WORDS;
                sws(pending_row, n.base);
                sws(pending_row + 1u, entry_offset(n, remaining - 1u) + LOGGED_OPERATIONS_KEY_BYTES);
                sws(pending_row + 2u, n.level - 1u);
                pending = pending + 1u;
            }
        }
        if (pending == 0u) {
            break;
        }
        pending = pending - 1u;
        let pending_row = G_PENDING + pending * G_PENDING_WORDS;
        current = parse_node_pointer(sw(pending_row), sw(pending_row + 1u));
        expected_level = sw(pending_row + 2u);
    }
    return true;
}

// 这一版树表里那一棵 inode 树（种类 2，第一版一版只有一棵）的全部记录读进 D_INODE_RECORDS（walk.rs inode_records_of_the_single_inode_tree_without_judging）：
// 不是恰好一条 inode 树条目、读不全交回 false
fn g_read_the_inode_records_of_the_single_inode_tree(tt: Node) -> bool {
    var inode_tree_entry_count = 0u;
    var inode_tree_entry = 0u;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_INODE) {
            inode_tree_entry_count = inode_tree_entry_count + 1u;
            inode_tree_entry = o;
        }
    }
    return inode_tree_entry_count == 1u
        && d_read_inode_records(parse_node_pointer(tt.base, inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET));
}

// 一版 logged_ops 树的每条条目：I-4.4 判种类码登记过（1 = 整理、2 = 截断）。截断条目（walk.rs judge_truncation_intent_against_its_record，
// 判决 m4-trunc-r1 K7 T-1）判补零 10 字节全 0、签发 txg 不晚于这一版的 txg、点名的记录在这一版那一棵 inode 树里、nlink ≥ 1、对象出生代相等、
// 目标 = size（记录在第一条截断条目那一刻读，读不出的那几条判不了），有截断条目的那一版另判至多一条。整理条目判补零全 0、R 对齐（起始槽 64 对齐、槽数 = 64）
// 且在单元区内（设备是池里的盘、[起始槽, 起始槽 + 槽数) 在 [单元区起始槽号, 盘末) 里）、c0 与签发 txg 都不晚于这一版的 txg；R 合法的整理意图再按 I-5.6 判这一版分配记录
// （records_read_in_full 时在 C_ALLOC_RECORDS 里）里罩进 R（任一块盘上的同号槽）的未释放记录，分配代都不大于 c0，判完再判一次成立
fn g_judge_the_compaction_intents_of_the_version(version: VersionWithItsRootPointersOfTheScan, tt: Node, records_read_in_full: bool) {
    g_logged_operation_entry_count = 0u;
    var read_in_full = true;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) != TREE_KIND_LOGGED_OPERATIONS) {
            continue;
        }
        let root = parse_node_pointer(tt.base, o + 14u);
        if (root.all_zero) {
            continue;
        }
        if (!g_read_the_logged_operation_entries(root)) {
            read_in_full = false;
        }
    }
    if (!read_in_full) {
        mark_part_of_the_range_not_judged(INVARIANT_I_4_4);
        mark_part_of_the_range_not_judged(INVARIANT_I_5_6);
    }
    let unit_area_start = handoff_unit_area_start_slot();
    let device_end_slot = shr64(device_bytes(), SLOT_SHIFT);
    var truncation_intents_of_this_version = 0u;
    var inode_records_were_read = false;
    var inode_records_read_in_full = false;
    for (var entry_index = 0u; entry_index < g_logged_operation_entry_count; entry_index = entry_index + 1u) {
        let base = sw(G_LOGGED_OPERATION_ENTRIES + entry_index * 2u);
        let o = sw(G_LOGGED_OPERATION_ENTRIES + entry_index * 2u + 1u);
        let kind = rd16(base, o);
        if (kind == G_LOGGED_OPERATION_KIND_TRUNCATION) {
            truncation_intents_of_this_version = truncation_intents_of_this_version + 1u;
            let truncation_value = o + LOGGED_OPERATIONS_KEY_BYTES;
            judge(INVARIANT_I_4_4, all_zero(base, truncation_value + G_TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE, G_TRUNCATION_INTENT_PADDING_BYTES));
            judge(INVARIANT_I_4_4, le64(rd64(base, o + G_LOGGED_OPERATION_ISSUED_TXG_OFFSET), version.txg));
            if (!inode_records_were_read) {
                inode_records_were_read = true;
                inode_records_read_in_full = g_read_the_inode_records_of_the_single_inode_tree(tt);
            }
            if (!inode_records_read_in_full) {
                mark_part_of_the_range_not_judged(INVARIANT_I_4_4);
                continue;
            }
            let named_record = d_inode_record_row_of(rd64(base, truncation_value + G_TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE));
            judge(INVARIANT_I_4_4, named_record != NONE && d_inode_record_nlink(named_record) >= 1u
                && eq64(d_inode_record_object_birth(named_record), rd64(base, truncation_value + G_TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE))
                && eq64(vec2<u32>(sw(named_record + 3u), sw(named_record + 4u)), rd64(base, truncation_value + G_TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE)));
            continue;
        }
        if (kind != G_LOGGED_OPERATION_KIND_COMPACTION) {
            judge(INVARIANT_I_4_4, false);
            continue;
        }
        let value = o + LOGGED_OPERATIONS_KEY_BYTES;
        let issued_txg = rd64(base, o + G_LOGGED_OPERATION_ISSUED_TXG_OFFSET);
        let region_device = rd32(base, value + G_COMPACTION_INTENT_REGION_DEVICE_OFFSET_IN_THE_VALUE);
        let region_start_slot = rd48(base, value + G_COMPACTION_INTENT_REGION_START_SLOT_OFFSET_IN_THE_VALUE);
        let region_slot_count = rd16(base, value + G_COMPACTION_INTENT_REGION_SLOT_COUNT_OFFSET_IN_THE_VALUE);
        let allocation_generation_at_issue = rd64(base, value + G_COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE);
        judge(INVARIANT_I_4_4, all_zero(base, value + G_COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE, G_COMPACTION_INTENT_PADDING_BYTES));
        let region_is_aligned = (region_start_slot.x & (CLUSTER_SEGMENT_SLOTS - 1u)) == 0u && region_slot_count == CLUSTER_SEGMENT_SLOTS;
        judge(INVARIANT_I_4_4, region_is_aligned);
        let region_lies_in_the_unit_area = device_is_in_the_pool(region_device) && le64(unit_area_start, region_start_slot)
            && le64(add64(region_start_slot, u64_of(region_slot_count)), device_end_slot);
        judge(INVARIANT_I_4_4, region_lies_in_the_unit_area);
        judge(INVARIANT_I_4_4, le64(allocation_generation_at_issue, version.txg) && le64(issued_txg, version.txg));
        if (!(region_is_aligned && region_lies_in_the_unit_area)) {
            continue;
        }
        if (!records_read_in_full) {
            mark_part_of_the_range_not_judged(INVARIANT_I_5_6);
            continue;
        }
        let region_end_slot = add64(region_start_slot, u64_of(CLUSTER_SEGMENT_SLOTS));
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            let slot = vec2<u32>(sw(record + 1u), sw(record + 2u));
            if (sw(record + 6u) != 0u || !lt64(slot, region_end_slot) || !lt64(region_start_slot, add64(slot, u64_of(sw(record + 3u))))) {
                continue;
            }
            judge(INVARIANT_I_5_6, le64(vec2<u32>(sw(record + 4u), sw(record + 5u)), allocation_generation_at_issue));
        }
        // R 里一条未释放记录都没有（整理已把它搬空、删意图那一刻之前）也是判过
        judge(INVARIANT_I_5_6, true);
    }
    // 全池至多一条截断条目（判决 m4-trunc-r1 K1）：只在有截断条目的那一版判，没有的版本 I-4.4 照旧没有对象
    if (truncation_intents_of_this_version > 0u) {
        judge(INVARIANT_I_4_4, truncation_intents_of_this_version <= 1u);
    }
}

// ── I-3.15（walk.rs judge_unreleased_allocation_records_against_the_central_mapping）

// 落点表记一行 (盘, 槽, 名目)：表满记判不了
fn g_note_a_placement(device: u32, slot: vec2<u32>, naming: u32) {
    if (g_placement_count >= G_MAX_PLACEMENTS) {
        undecidable = G_SITE_PLACEMENTS_PAST_THE_TABLE;
        return;
    }
    var bucket = slot_key_first_bucket(device, slot, G_PLACEMENT_BUCKET_COUNT);
    for (var probe = 0u; probe < G_PLACEMENT_BUCKET_COUNT && sw(G_PLACEMENT_BUCKETS + bucket) != 0u; probe = probe + 1u) {
        bucket = (bucket + 1u) & (G_PLACEMENT_BUCKET_COUNT - 1u);
    }
    let row = G_PLACEMENTS + g_placement_count * G_PLACEMENT_WORDS;
    sws(row, device);
    sws(row + 1u, slot.x);
    sws(row + 2u, slot.y);
    sws(row + 3u, naming);
    sws(row + 4u, bucket);
    g_placement_count = g_placement_count + 1u;
    sws(G_PLACEMENT_BUCKETS + bucket, g_placement_count);
}

// 一条由根记录或父指针直接指着的指针（D19（块指针的结构与宽度预算） 已定项 8 的自举豁免）：全零的不记，两条位置条目各记一行
fn g_note_a_pointer_named_directly(p: Ptr) {
    if (p.all_zero) {
        return;
    }
    for (var copy = 0u; copy < 2u; copy = copy + 1u) {
        g_note_a_placement(select(p.device0, p.device1, copy == 1u), select(p.slot0, p.slot1, copy == 1u), G_NAMED_DIRECTLY);
    }
}

// 中央映射树根之下各层的节点指针（walk.rs central_mapping_tree_of_a_version_without_judging 的 node_pointers_below_the_root）：映射这一版读全了才走，
// 每个内部节点每条条目的子指针都记
fn g_note_the_central_mapping_nodes_below_the_root(mapping_root: Ptr) {
    if (mapping_root.all_zero) {
        return;
    }
    var pending = 0u;
    var current = mapping_root;
    loop {
        let n = r_read_index_node(current);
        if (n.ok && n.level > 0u) {
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                if (pending >= G_MAX_PENDING) {
                    undecidable = G_SITE_PENDING_PAST_THE_STACK;
                    return;
                }
                let child_offset = entry_offset(n, e) + MAPPING_KEY_BYTES;
                g_note_a_pointer_named_directly(parse_node_pointer(n.base, child_offset));
                let pending_row = G_PENDING + pending * G_PENDING_WORDS;
                sws(pending_row, n.base);
                sws(pending_row + 1u, child_offset);
                pending = pending + 1u;
            }
        }
        if (pending == 0u) {
            break;
        }
        pending = pending - 1u;
        let pending_row = G_PENDING + pending * G_PENDING_WORDS;
        current = parse_node_pointer(sw(pending_row), sw(pending_row + 1u));
    }
}

// 树表 0 条那一版根记录（或记录新根段）直接持有的那棵分配记录树根之下的节点（walk.rs allocation_records_of_a_version_without_judging 的
// nodes_of_the_tree_the_root_record_holds；树 ID 0，不进映射，只有这棵树里的父节点指着）：这一版的分配记录读全了才走，内部条目偏移 10 起是子指针
fn g_note_the_nodes_below_the_allocation_record_tree_root_held_by_the_root(base: u32, held_offset: u32) {
    let held = parse_node_pointer(base, held_offset);
    if (held.all_zero) {
        return;
    }
    var pending = 0u;
    var current = held;
    loop {
        let n = r_read_located_index_node(current);
        if (n.ok && n.level > 0u) {
            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                if (pending >= G_MAX_PENDING) {
                    undecidable = G_SITE_PENDING_PAST_THE_STACK;
                    return;
                }
                let child_offset = entry_offset(n, e) + 10u;
                g_note_a_pointer_named_directly(parse_node_pointer(n.base, child_offset));
                let pending_row = G_PENDING + pending * G_PENDING_WORDS;
                sws(pending_row, n.base);
                sws(pending_row + 1u, child_offset);
                pending = pending + 1u;
            }
        }
        if (pending == 0u) {
            break;
        }
        pending = pending - 1u;
        let pending_row = G_PENDING + pending * G_PENDING_WORDS;
        current = parse_node_pointer(sw(pending_row), sw(pending_row + 1u));
    }
}

// walk.rs mapping_key_matches_the_unit_header：类标签相同；头里的诞生代号、写序实例与尾段（码 1 事务号 6 字节、码 2 / 码 3 出生序号 4 字节，
// 与 key 尾段的低 4 字节比）与 key 里的相同；码 1 / 码 3 头里偏移 43 的出生树与 key 里的出生树相同，码 2 不比（C289）。s 是一份可用的单元头
fn g_mapping_key_matches_the_unit_header(key: MappingKey, s: Slice) -> bool {
    let unit_class = rd8(s.base, 6u);
    if ((key.class_and_tail_high & 0xFFu) != unit_class) {
        return false;
    }
    var birth_txg = vec2<u32>(0u, 0u);
    var instance = 0u;
    var tail_matches = false;
    var birth_tree_matches = true;
    if (unit_class == 1u) {
        birth_txg = rd64(s.base, 75u);
        instance = rd32(s.base, 91u);
        let transaction = rd48(s.base, 95u);
        tail_matches = key.tail_low == transaction.x && (key.class_and_tail_high >> 16u) == transaction.y;
        birth_tree_matches = eq64(rd64(s.base, 43u), key.birth_tree);
    } else if (unit_class == 2u) {
        let key_span = 2u * rd8(s.base, 51u);
        birth_txg = rd64(s.base, 52u + key_span);
        instance = rd32(s.base, 68u + key_span);
        tail_matches = key.tail_low == rd32(s.base, 72u + key_span);
    } else if (unit_class == 3u) {
        birth_txg = rd64(s.base, 73u);
        instance = rd32(s.base, 93u);
        tail_matches = key.tail_low == rd32(s.base, 103u);
        birth_tree_matches = eq64(rd64(s.base, 43u), key.birth_tree);
    } else {
        return false;
    }
    return eq64(birth_txg, key.birth_txg) && instance == key.instance && tail_matches && birth_tree_matches;
}

// 落点表里 (盘, 槽) 那几行：有一行是直接指着的（looking_for_a_key 为假时），或有一行的映射 key 与单元头 s 相符（looking_for_a_key 为真时）。
// 同 (盘, 槽) 的几行都落在从它的首桶起、到第一个空桶为止的那一段里
fn g_placement_is_accounted_for(device: u32, slot: vec2<u32>, looking_for_a_key: bool, s: Slice) -> bool {
    var bucket = slot_key_first_bucket(device, slot, G_PLACEMENT_BUCKET_COUNT);
    for (var probe = 0u; probe < G_PLACEMENT_BUCKET_COUNT; probe = probe + 1u) {
        let row_plus_one = sw(G_PLACEMENT_BUCKETS + bucket);
        if (row_plus_one == 0u) {
            return false;
        }
        let row = G_PLACEMENTS + (row_plus_one - 1u) * G_PLACEMENT_WORDS;
        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y) {
            let naming = sw(row + 3u);
            if (!looking_for_a_key && naming == G_NAMED_DIRECTLY) {
                return true;
            }
            if (looking_for_a_key && naming != G_NAMED_DIRECTLY && g_mapping_key_matches_the_unit_header(version_mapping_key_of_row(naming), s)) {
                return true;
            }
        }
        bucket = (bucket + 1u) & (G_PLACEMENT_BUCKET_COUNT - 1u);
    }
    return false;
}

// 落点表里 (盘, 槽) 那几行有一行是这一版映射叶的位置条目（不是直接指着的那一种）：walk.rs keys_at_each_placement 里有 (盘, 槽) 这一格
fn g_some_mapping_location_entry_names_the_placement(device: u32, slot: vec2<u32>) -> bool {
    var bucket = slot_key_first_bucket(device, slot, G_PLACEMENT_BUCKET_COUNT);
    for (var probe = 0u; probe < G_PLACEMENT_BUCKET_COUNT; probe = probe + 1u) {
        let row_plus_one = sw(G_PLACEMENT_BUCKETS + bucket);
        if (row_plus_one == 0u) {
            return false;
        }
        let row = G_PLACEMENTS + (row_plus_one - 1u) * G_PLACEMENT_WORDS;
        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y && sw(row + 3u) != G_NAMED_DIRECTLY) {
            return true;
        }
        bucket = (bucket + 1u) & (G_PLACEMENT_BUCKET_COUNT - 1u);
    }
    return false;
}

// 这一版映射里没有一条位置条目指着 slot（这一版分配记录里这一槽有记录的每块盘上都没有）：walk.rs 的
// no_mapping_key_of_the_version_names_the_slot（判决 m4-i315-r1 第三节 C1「乙二」）
fn g_no_mapping_location_entry_of_the_version_names_the_slot(slot: vec2<u32>) -> bool {
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        if (eq64(vec2<u32>(sw(record + 1u), sw(record + 2u)), slot) && g_some_mapping_location_entry_names_the_placement(sw(record), slot)) {
            return false;
        }
    }
    return true;
}

// C_ALLOC_RECORDS 一行第八个字（c_append_allocation_record 写 0）在 I-3.15 里的记号：对不上映射的、它那一组已判过的
const G_RECORD_OFF_THE_CENTRAL_MAPPING: u32 = 1u;
const G_RECORD_OFF_THE_CENTRAL_MAPPING_JUDGED: u32 = 2u;

// 一版（它的分配记录在 C_ALLOC_RECORDS，中央映射在 C_VERSION_MAPPING_ROWS，都读全了）上的 I-3.15。对不上的记录按单元（起点槽、跨度）归组，
// 组按 (起点槽, 跨度) 升序判（walk.rs 那张 BTreeMap 的次序：隔离判据查位置项校验和时补不补根环里没走过的根，次序影响得到）；组里按记录次序逐块盘
// 读回来核位置项，有一份对不上就停（同 walk.rs 的 any），这一组不判，否则组里每条判红
fn g_judge_the_unreleased_records_of_the_version_against_its_central_mapping(version: VersionWithItsRootPointersOfTheScan, fsid_low: vec2<u32>) {
    // 上一版记进桶表的那几行，按各自记的桶清掉（桶表在这一截开头整张清过）
    for (var placement = 0u; placement < g_placement_count; placement = placement + 1u) {
        sws(G_PLACEMENT_BUCKETS + sw(G_PLACEMENTS + placement * G_PLACEMENT_WORDS + 4u), 0u);
    }
    g_placement_count = 0u;
    let first_page = parse_node_pointer(version.base, version.instance_table_offset);
    if (!first_page.all_zero) {
        r_noting = false;
        r_complete = true;
        g_noting_the_instance_table_pages = true;
        r_instance_pages_after_first(first_page);
        g_noting_the_instance_table_pages = false;
        if (!r_complete) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_15);
            return;
        }
    }
    for (var which = 0u; which < 4u; which = which + 1u) {
        g_note_a_pointer_named_directly(parse_node_pointer(version.base, g_root_pointer_offset(version, which)));
    }
    g_note_the_central_mapping_nodes_below_the_root(parse_node_pointer(version.base, version.mapping_root_offset));
    g_note_the_nodes_below_the_allocation_record_tree_root_held_by_the_root(version.base, version.allocation_root_offset);
    for (var row_index = 0u; row_index < version_mapping_count; row_index = row_index + 1u) {
        let row = C_VERSION_MAPPING_ROWS + row_index * C_VERSION_MAPPING_ROW_WORDS;
        for (var location = 0u; location < 2u; location = location + 1u) {
            g_note_a_placement(sw(row + 8u + location * 4u), vec2<u32>(sw(row + 9u + location * 4u), sw(row + 10u + location * 4u)), row_index);
        }
    }
    if (undecidable != 0u) {
        return;
    }
    var nothing_off_the_mapping = true;
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        sws(record + 7u, 0u);
        if (sw(record + 6u) != 0u) {
            continue;
        }
        let device = sw(record);
        let slot = vec2<u32>(sw(record + 1u), sw(record + 2u));
        let no_header = no_slice();
        if (g_placement_is_accounted_for(device, slot, false, no_header)) {
            judge(INVARIANT_I_3_15, true);
            continue;
        }
        // 起点槽上可用的单元头（walk.rs birth_txg_in_a_usable_unit_header：magic、类 1 / 2 / 3、头校验和过）：用不了的没有对象
        let s = read_at(device, slot_to_bytes(slot), 4096u);
        if (s.version == NONE || rd32(s.base, 0u) != MAGIC_SFSU || (s.c2 & 1u) == 0u || rd8(s.base, 6u) == 0u || rd8(s.base, 6u) > 3u) {
            continue;
        }
        if (g_placement_is_accounted_for(device, slot, true, s)) {
            judge(INVARIANT_I_3_15, true);
            continue;
        }
        sws(record + 7u, G_RECORD_OFF_THE_CENTRAL_MAPPING);
        nothing_off_the_mapping = false;
    }
    if (nothing_off_the_mapping) {
        return;
    }
    loop {
        // 还没判的组里 (起点槽, 跨度) 最小的那一组的第一条
        var first_of_the_group = NONE;
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            if (sw(record + 7u) != G_RECORD_OFF_THE_CENTRAL_MAPPING) {
                continue;
            }
            if (first_of_the_group == NONE) {
                first_of_the_group = i;
                continue;
            }
            let first = C_ALLOC_RECORDS + first_of_the_group * C_ALLOC_RECORD_WORDS;
            let slot = vec2<u32>(sw(record + 1u), sw(record + 2u));
            let first_slot = vec2<u32>(sw(first + 1u), sw(first + 2u));
            if (lt64(slot, first_slot) || (eq64(slot, first_slot) && sw(record + 3u) < sw(first + 3u))) {
                first_of_the_group = i;
            }
        }
        if (first_of_the_group == NONE) {
            break;
        }
        let first = C_ALLOC_RECORDS + first_of_the_group * C_ALLOC_RECORD_WORDS;
        let group_slot = vec2<u32>(sw(first + 1u), sw(first + 2u));
        let group_span = sw(first + 3u);
        // 隔离判据读哪几份（walk.rs devices_of_the_unit，判决 m4-i315-r1 第三节 C1「乙二」）：这一版映射里没有位置条目指着这一槽时，
        // 这一单元每块盘上未释放的记录（头用不了、没打记号的那几条也在里面），否则只是打了对不上映射记号的那几条；都按记录次序
        let every_unreleased_copy_of_the_unit = g_no_mapping_location_entry_of_the_version_names_the_slot(group_slot);
        var some_copy_fails = false;
        for (var i = 0u; i < c_alloc_record_count && !some_copy_fails; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            let read_for_the_quarantine = select(sw(record + 7u) == G_RECORD_OFF_THE_CENTRAL_MAPPING, sw(record + 6u) == 0u,
                every_unreleased_copy_of_the_unit);
            if (read_for_the_quarantine && eq64(vec2<u32>(sw(record + 1u), sw(record + 2u)), group_slot)
                && sw(record + 3u) == group_span) {
                some_copy_fails = !c_copy_matches_its_location_entry(sw(record), group_slot, group_span, fsid_low);
            }
        }
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            if (sw(record + 7u) == G_RECORD_OFF_THE_CENTRAL_MAPPING && eq64(vec2<u32>(sw(record + 1u), sw(record + 2u)), group_slot)
                && sw(record + 3u) == group_span) {
                sws(record + 7u, G_RECORD_OFF_THE_CENTRAL_MAPPING_JUDGED);
                if (!some_copy_fails) {
                    judge(INVARIANT_I_3_15, false);
                }
            }
        }
    }
}

// ── I-1.14（walk.rs judge_unit_bytes_of_each_mapping_key_across_the_versions）

// 跨版本那张表第 group_index 行是不是这一版映射叶第 mapping_row 起那一行的 (key, 两条位置条目)
fn g_key_group_is_the_mapping_row(group_index: u32, key: MappingKey, mapping_row: u32) -> bool {
    let group = G_KEY_GROUPS + group_index * G_KEY_GROUP_WORDS;
    if (!same_mapping_key(mapping_key_of_the_words(group), key)) {
        return false;
    }
    for (var word = 0u; word < 8u; word = word + 1u) {
        if (sw(group + 7u + word) != sw(mapping_row + 8u + word)) {
            return false;
        }
    }
    return true;
}

// 这一版映射叶（C_VERSION_MAPPING_ROWS，叶里出现两次的 key 两行都在）每一行的 (key, 两条位置条目) 没记过的记进跨版本那张表
fn g_note_the_key_groups_of_the_version() {
    for (var row_index = 0u; row_index < version_mapping_count; row_index = row_index + 1u) {
        let mapping_row = C_VERSION_MAPPING_ROWS + row_index * C_VERSION_MAPPING_ROW_WORDS;
        let key = version_mapping_key_of_row(row_index);
        var bucket = mapping_key_first_bucket(key, G_KEY_GROUP_BUCKET_COUNT);
        var noted_before = false;
        for (var probe = 0u; probe < G_KEY_GROUP_BUCKET_COUNT; probe = probe + 1u) {
            let group_plus_one = sw(G_KEY_GROUP_BUCKETS + bucket);
            if (group_plus_one == 0u) {
                break;
            }
            if (g_key_group_is_the_mapping_row(group_plus_one - 1u, key, mapping_row)) {
                noted_before = true;
                break;
            }
            bucket = (bucket + 1u) & (G_KEY_GROUP_BUCKET_COUNT - 1u);
        }
        if (noted_before) {
            continue;
        }
        if (g_key_group_count >= G_MAX_KEY_GROUPS) {
            undecidable = G_SITE_KEY_GROUPS_PAST_THE_TABLE;
            return;
        }
        let group = G_KEY_GROUPS + g_key_group_count * G_KEY_GROUP_WORDS;
        put_the_mapping_key_into_the_words(group, key);
        for (var word = 0u; word < 8u; word = word + 1u) {
            sws(group + 7u + word, sw(mapping_row + 8u + word));
        }
        sws(group + 15u, G_KEY_GROUP_NOT_READ_YET);
        g_key_group_count = g_key_group_count + 1u;
        sws(G_KEY_GROUP_BUCKETS + bucket, g_key_group_count);
    }
}

// 一把 key 的类标签说的单元多大（walk.rs unit_bytes_of_the_class）：码 1、码 3 是 32 KiB，码 2 是 16 KiB，别的码 0（不比）
fn g_unit_bytes_of_the_class(unit_class: u32) -> u32 {
    if (unit_class == 1u || unit_class == 3u) {
        return DATA_UNIT_BYTES;
    }
    if (unit_class == 2u) {
        return NODE_BYTES;
    }
    return 0u;
}

// 第 group_index 组读得出没有：第一份按自己的校验和对得上的副本（两条位置条目依次，同 walk.rs 的 find_map），读过就记下、不读第二次
fn g_key_group_is_readable(group_index: u32, unit_bytes: u32) -> bool {
    let group = G_KEY_GROUPS + group_index * G_KEY_GROUP_WORDS;
    if (sw(group + 15u) == G_KEY_GROUP_NOT_READ_YET) {
        var locations = zero_pointer();
        locations.all_zero = false;
        locations.device0 = sw(group + 7u);
        locations.slot0 = vec2<u32>(sw(group + 8u), sw(group + 9u));
        locations.sum0 = sw(group + 10u);
        locations.device1 = sw(group + 11u);
        locations.slot1 = vec2<u32>(sw(group + 12u), sw(group + 13u));
        locations.sum1 = sw(group + 14u);
        let s = read_unit_via_locations(locations, unit_bytes);
        if (s.version == NONE) {
            sws(group + 15u, G_KEY_GROUP_UNREADABLE);
        } else {
            sws(group + 15u, G_KEY_GROUP_READ);
            sws(group + 16u, s.base);
            sws(group + 17u, s.version);
        }
    }
    return sw(group + 15u) == G_KEY_GROUP_READ;
}

// 两组读出来的那两份 unit_bytes 个字节逐个相同（同一个版本号就是同一份字节）
fn g_key_groups_read_the_same_bytes(first_group_index: u32, group_index: u32, unit_bytes: u32) -> bool {
    let first = G_KEY_GROUPS + first_group_index * G_KEY_GROUP_WORDS;
    let group = G_KEY_GROUPS + group_index * G_KEY_GROUP_WORDS;
    if (sw(first + 17u) == sw(group + 17u)) {
        return true;
    }
    for (var word = 0u; word < unit_bytes / 4u; word = word + 1u) {
        if (rd32(sw(first + 16u), word * 4u) != rd32(sw(group + 16u), word * 4u)) {
            return false;
        }
    }
    return true;
}

// 同一把 key 落在不止一组位置条目上时，每一组读出来的那一份与这把 key 最早记下、读得出的那一组比（读得出的各组两两相同 ⟺ 都与最早那一组相同，
// 比的对数同 walk.rs：读得出的组数减一）；只有一组的 key 不读
fn g_judge_the_unit_bytes_of_each_mapping_key_across_the_versions() {
    for (var group_index = 0u; group_index < g_key_group_count; group_index = group_index + 1u) {
        let key = mapping_key_of_the_words(G_KEY_GROUPS + group_index * G_KEY_GROUP_WORDS);
        let unit_bytes = g_unit_bytes_of_the_class(key.class_and_tail_high & 0xFFu);
        if (unit_bytes == 0u) {
            continue;
        }
        var key_has_another_group = false;
        var first_readable_group = NONE;
        var bucket = mapping_key_first_bucket(key, G_KEY_GROUP_BUCKET_COUNT);
        for (var probe = 0u; probe < G_KEY_GROUP_BUCKET_COUNT; probe = probe + 1u) {
            let other_plus_one = sw(G_KEY_GROUP_BUCKETS + bucket);
            if (other_plus_one == 0u) {
                break;
            }
            let other = other_plus_one - 1u;
            if (other != group_index && same_mapping_key(mapping_key_of_the_words(G_KEY_GROUPS + other * G_KEY_GROUP_WORDS), key)) {
                key_has_another_group = true;
                if (other < group_index && (first_readable_group == NONE || other < first_readable_group)
                    && g_key_group_is_readable(other, unit_bytes)) {
                    first_readable_group = other;
                }
            }
            bucket = (bucket + 1u) & (G_KEY_GROUP_BUCKET_COUNT - 1u);
        }
        if (!key_has_another_group || !g_key_group_is_readable(group_index, unit_bytes) || first_readable_group == NONE) {
            continue;
        }
        judge(INVARIANT_I_1_14, g_key_groups_read_the_same_bytes(first_readable_group, group_index, unit_bytes));
    }
}

// 这一节的入口：版本清单里每一版各判 I-4.4 / I-5.6、记 I-1.14 那张表、判 I-3.15，最后跨版本判 I-1.14
fn judge_compaction_intents_relocations_and_the_unit_bytes_of_the_versions() {
    let fsid_low = vec2<u32>(sw(H_BASE + H_FSID0), sw(H_BASE + H_FSID1));
    // I-3.15 的隔离判据接着扫描内核记账那一判两遍隔离豁免之后的状态（walk.rs 的位置项校验和在 I-3.1 与 I-3.15 之间是同一份）
    c_harvested = sw(C_HARVESTED_BY_THE_ACCOUNTING_QUARANTINE) == 1u;
    for (var bucket = 0u; bucket < G_PLACEMENT_BUCKET_COUNT; bucket = bucket + 1u) {
        sws(G_PLACEMENT_BUCKETS + bucket, 0u);
    }
    for (var bucket = 0u; bucket < G_KEY_GROUP_BUCKET_COUNT; bucket = bucket + 1u) {
        sws(G_KEY_GROUP_BUCKETS + bucket, 0u);
    }
    g_placement_count = 0u;
    g_key_group_count = 0u;
    let version_total = c_candidate_count + c_version_count;
    for (var version_index = 0u; version_index < version_total; version_index = version_index + 1u) {
        let version = g_version_with_its_root_pointers(version_index);
        s_use_the_central_mapping_named_at(version.base, version.mapping_root_offset);
        let mapping_read_in_full = version_mapping_read_in_full;
        if (mapping_read_in_full) {
            g_note_the_key_groups_of_the_version();
        } else {
            mark_part_of_the_range_not_judged(INVARIANT_I_1_14);
        }
        let tt = g_tree_table_of_the_version(version.base, version.tree_table_offset);
        if (!tt.ok) {
            mark_part_of_the_range_not_judged(INVARIANT_I_4_4);
            mark_part_of_the_range_not_judged(INVARIANT_I_5_6);
            mark_part_of_the_range_not_judged(INVARIANT_I_3_15);
            continue;
        }
        let records_read_in_full = g_allocation_records_of_the_version(version.base, version.allocation_root_offset, tt);
        g_judge_the_compaction_intents_of_the_version(version, tt, records_read_in_full);
        if (g_root_pointers_seen_in_an_earlier_version(version_index)) {
            continue;
        }
        if (!records_read_in_full || !mapping_read_in_full) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_15);
            continue;
        }
        g_judge_the_unreleased_records_of_the_version_against_its_central_mapping(version, fsid_low);
    }
    g_judge_the_unit_bytes_of_each_mapping_key_across_the_versions();
}

// 引用与快照那一截（扫描内核源码里的第二个入口 references_and_snapshots_main 跑它）：I-3.9 与 I-9.14（每条候选根引用了什么），连同只读快照与 deadlist，
// 再是整理意图与搬迁那几条
fn references_and_snapshots_stage() {
    judge_release_generation_and_tree_table_birth();
    judge_snapshots_and_deadlists_of_the_versions();
    judge_compaction_intents_relocations_and_the_unit_bytes_of_the_versions();
}

fn scan_stage(fsid_low: vec2<u32>) {
    judge_merged_version_total_order(fsid_low);
    judge_tree_identifier_birth_txg_across_every_readable_root();
    judge_allocation_records_disjoint();
    judge_allocation_generations_against_unit_births();
    judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
    judge_deletion_state_of_the_versions();
    judge_inode_number_watermark_steps_against_the_watermark_of_the_newest_root();
    judge_tombstones_and_the_lineage_of_the_versions();
    judge_inode_number_watermark_above_every_published_tombstone();
    judge_rollback_floor_raises_against_their_ceilings();
    judge_accounting(fsid_low);
    judge_instance_table_rows_against_the_roots_referencing_them();
}

// ───────────────────────── 入口 ─────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    let fsid_low = vec2<u32>(sw(H_BASE + H_FSID0), sw(H_BASE + H_FSID1));
    scan_stage(fsid_low);
    store_violated(out);
    store_judged(out);
    if (undecidable != 0u) {
        if (verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] == 0u) {
            verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] = undecidable;
        }
    } else {
        verdicts[out + POOL_CHECKER_REACHED_WORD] = 5u;
    }
}

// 引用与快照那一截的入口（与扫描内核同一份源码、各编一条管线：扫描那一截展开之后离预算只剩几千个表达式，I-3.9 跟着判决 G3 认映射位置、数快照的树之后
// 装不下，连同只读快照那几条挪到这里）。走读内核判完、交接段有效时，接着扫描内核的违例位与判到位记，判不了的原因照扫描内核那一套只在还没记过时写；
// 「走到第几步」那一字仍由扫描内核写
@compute @workgroup_size(64)
fn references_and_snapshots_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    references_and_snapshots_stage();
    store_violated(out);
    store_judged(out);
    if (undecidable != 0u && verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] == 0u) {
        verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] = undecidable;
    }
}
