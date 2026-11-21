// ── 中央映射认位置（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；walk.rs 的 Walk::located 与 copy_at_a_location_hint）
// w_locate 换过位置的最近那一条指针（换过之后的样子）与它自己那两条位置条目（提示）：w_read_unit_to_walk 读的正是它时先按提示读。
// 每版开头清掉；同一条指针（same_pointer 逐段相同）才认
var<private> w_hint_is_noted: bool;
var<private> w_hint_located: Ptr;
var<private> w_hint_device0: u32;
var<private> w_hint_slot0: vec2<u32>;
var<private> w_hint_device1: u32;
var<private> w_hint_slot1: vec2<u32>;

// 按查找路径给的类认一条指针指的单元落在哪（located_by_the_central_mapping），位置换过时把提示记下
fn w_locate(p: Ptr, mapping_class: u32) -> Ptr {
    let located = located_by_the_central_mapping(p, mapping_class);
    if (located.device0 != p.device0 || !eq64(located.slot0, p.slot0) || located.device1 != p.device1 || !eq64(located.slot1, p.slot1)) {
        w_hint_is_noted = true;
        w_hint_located = located;
        w_hint_device0 = p.device0;
        w_hint_slot0 = p.slot0;
        w_hint_device1 = p.device1;
        w_hint_slot1 = p.slot1;
    }
    return located;
}

// 主走读的「中央映射 key → 走读认它作哪个单元」（W_MAPPING_KEY_UNITS，walk.rs 的 unit_of_each_central_mapping_key）：只在主走读（上下文 0）记
struct MappingKeyUnit {
    device: u32,
    slot: vec2<u32>,
};

var<private> w_mapping_key_unit_count: u32;
var<private> w_mapping_key_units_are_indexed: bool;

const UNDECIDABLE_MAPPING_KEY_UNITS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (92u << 8u);

fn w_mapping_key_of_a_unit_row(row_index: u32) -> MappingKey {
    return mapping_key_of_the_words(W_MAPPING_KEY_UNITS + row_index * W_MAPPING_KEY_UNIT_WORDS);
}

// key 第一次记下时记成 first（or_insert），之后交回第一次记下的那一个。条数过了 KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST 才建桶表（同 ⑬）；
// 找与记各一个比键的调用点
fn w_unit_of_the_mapping_key(key: MappingKey, first: MappingKeyUnit) -> MappingKeyUnit {
    var bucket = mapping_key_first_bucket(key, W_MAPPING_KEY_UNIT_BUCKET_COUNT);
    let steps = select(w_mapping_key_unit_count, W_MAPPING_KEY_UNIT_BUCKET_COUNT, w_mapping_key_units_are_indexed);
    var empty_bucket = NONE;
    for (var step = 0u; step < steps; step = step + 1u) {
        var row_index = step;
        if (w_mapping_key_units_are_indexed) {
            let row_plus_one = sw(W_MAPPING_KEY_UNIT_BUCKETS + bucket);
            if (row_plus_one == 0u) {
                empty_bucket = bucket;
                break;
            }
            row_index = row_plus_one - 1u;
            bucket = (bucket + 1u) & (W_MAPPING_KEY_UNIT_BUCKET_COUNT - 1u);
        }
        if (same_mapping_key(w_mapping_key_of_a_unit_row(row_index), key)) {
            let row = W_MAPPING_KEY_UNITS + row_index * W_MAPPING_KEY_UNIT_WORDS;
            return MappingKeyUnit(sw(row + 7u), vec2<u32>(sw(row + 8u), sw(row + 9u)));
        }
    }
    if (w_mapping_key_unit_count >= W_MAX_MAPPING_KEY_UNITS) {
        undecidable = UNDECIDABLE_MAPPING_KEY_UNITS_PAST_THE_TABLE;
        return first;
    }
    let appended = w_mapping_key_unit_count;
    let row = W_MAPPING_KEY_UNITS + appended * W_MAPPING_KEY_UNIT_WORDS;
    put_the_mapping_key_into_the_words(row, key);
    sws(row + 7u, first.device);
    sws(row + 8u, first.slot.x);
    sws(row + 9u, first.slot.y);
    sws(row + 10u, 0u);
    w_mapping_key_unit_count = appended + 1u;
    if (w_mapping_key_units_are_indexed) {
        sws(W_MAPPING_KEY_UNIT_BUCKETS + empty_bucket, appended + 1u);
    } else if (w_mapping_key_unit_count > KEYED_TABLE_ROWS_FOUND_BY_SCANNING_AT_MOST) {
        for (var cleared = 0u; cleared < W_MAPPING_KEY_UNIT_BUCKET_COUNT; cleared = cleared + 1u) {
            sws(W_MAPPING_KEY_UNIT_BUCKETS + cleared, 0u);
        }
        for (var row_to_put = 0u; row_to_put < w_mapping_key_unit_count; row_to_put = row_to_put + 1u) {
            var put_bucket = mapping_key_first_bucket(w_mapping_key_of_a_unit_row(row_to_put), W_MAPPING_KEY_UNIT_BUCKET_COUNT);
            for (var probe = 0u; probe < W_MAPPING_KEY_UNIT_BUCKET_COUNT; probe = probe + 1u) {
                if (sw(W_MAPPING_KEY_UNIT_BUCKETS + put_bucket) == 0u) {
                    sws(W_MAPPING_KEY_UNIT_BUCKETS + put_bucket, row_to_put + 1u);
                    break;
                }
                put_bucket = (put_bucket + 1u) & (W_MAPPING_KEY_UNIT_BUCKET_COUNT - 1u);
            }
        }
        w_mapping_key_units_are_indexed = true;
    }
    return first;
}

// image::read_referenced_unit：两条位置条目都读、都判 I-2.1，交回第一份对得上的。两条位置条目在一个循环里读（一个 read_at 调用点，
// 次序与原来两次调用相同：先第 0 条、后第 1 条，都判、都数对不上的）
fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
    var first = no_slice();
    w_read_back_from_the_second_location = false;
    for (var copy = 0u; copy < 2u; copy = copy + 1u) {
        let is_the_second = copy == 1u;
        let s = read_at(select(p.device0, p.device1, is_the_second), slot_to_bytes(select(p.slot0, p.slot1, is_the_second)), unit_bytes);
        let matches = s.version != NONE && slice_crc_matches(s, unit_bytes, select(p.sum0, p.sum1, is_the_second));
        if (!matches) {
            w_mismatches = w_mismatches + 1u;
        }
        w_judge(INVARIANT_I_2_1, matches);
        if (matches && first.version == NONE) {
            first = s;
            w_read_back_from_the_second_location = is_the_second;
        }
    }
    return first;
}

// walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from：按位置条目读一个走读要往下走的单元（同 w_read_referenced_unit），读回、自证过的那一份的
// 落点记进 w_unit_device_read_back / w_unit_slot_read_back；读回的是第二条那一份、I-3.5 的落点表此刻把那个落点折在第一条落点那个单元上时，
// 改记成它自己（appearance_note_placement 记的是第一条位置条目的落点）。调用方先 w_note_pointer_references 再调它
fn w_read_unit_to_walk(p: Ptr, unit_bytes: u32) -> Slice {
    // 提示与映射位置不同（w_locate 刚换过这条指针的位置）：先按提示读、只读不判（walk.rs copy_at_a_location_hint）——一份按指针里的整单元校验和对得上、
    // 头里的出生身份与查找路径相同就用它，认的落点是映射条目第 0 条位置（这个单元在走读里的身份落在映射位置上，提示那一槽已释放、已复用都不判）；
    // 两份都对不上再按映射位置读，每条照判 I-2.1
    // 提示两份与位置两份在一个循环里读（一个 read_at 调用点，这个函数在走读内核里展开十几份）：前两圈是提示（只读不判），后两圈是 w_read_referenced_unit 那一读
    let hints_apply = w_hint_is_noted && same_pointer(p, w_hint_located);
    var first = no_slice();
    var read_from_a_hint = false;
    w_read_back_from_the_second_location = false;
    for (var copy = 0u; copy < 4u; copy = copy + 1u) {
        let is_a_hint = copy < 2u;
        if (is_a_hint && !hints_apply) {
            continue;
        }
        let is_the_second = (copy % 2u) == 1u;
        var device = select(p.device0, p.device1, is_the_second);
        var slot = select(p.slot0, p.slot1, is_the_second);
        if (is_a_hint) {
            device = select(w_hint_device0, w_hint_device1, is_the_second);
            slot = select(w_hint_slot0, w_hint_slot1, is_the_second);
        }
        let copy_read = read_at(device, slot_to_bytes(slot), unit_bytes);
        let matches = copy_read.version != NONE && slice_crc_matches(copy_read, unit_bytes, select(p.sum0, p.sum1, is_the_second));
        if (is_a_hint) {
            if (matches && unit_birth_identity_matches_the_pointer(copy_read, p)) {
                first = copy_read;
                read_from_a_hint = true;
                break;
            }
            continue;
        }
        if (!matches) {
            w_mismatches = w_mismatches + 1u;
        }
        w_judge(INVARIANT_I_2_1, matches);
        if (matches && first.version == NONE) {
            first = copy_read;
            w_read_back_from_the_second_location = is_the_second;
        }
    }
    let s = first;
    if (read_from_a_hint) {
        w_unit_device_read_back = p.device0;
        w_unit_slot_read_back = p.slot0;
        return s;
    }
    w_unit_device_read_back = p.device0;
    w_unit_slot_read_back = p.slot0;
    if (s.version != NONE && w_read_back_from_the_second_location) {
        w_unit_device_read_back = p.device1;
        w_unit_slot_read_back = p.slot1;
        if (w_context == 0u && !(p.device1 == p.device0 && eq64(p.slot1, p.slot0))) {
            let index = appearance_placement_index(p.device1, p.slot1);
            if (index != NONE) {
                let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
                if (sw(row + 3u) == p.device0 && sw(row + 4u) == p.slot0.x && sw(row + 5u) == p.slot0.y) {
                    sws(row + 3u, p.device1);
                    sws(row + 4u, p.slot1.x);
                    sws(row + 5u, p.slot1.y);
                }
            }
        }
    }
    return s;
}

// 「走过」按 w_read_unit_to_walk 刚读回、自证过的那一份的落点认（攻方 F-KEY，判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第三节第 2 条）：
// 第一条位置条目指着这一遍先走过的别的单元、第二条那一份完好时，按第一条认会把读回来的这一份当成走过、它的头与格式一个字节都不判
fn w_visit_the_copy_read_back() -> bool {
    return w_visit(w_unit_device_read_back, w_unit_slot_read_back);
}

// I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免
fn w_judge_location_order(p: Ptr) {
    if (p.all_zero) {
        return;
    }
    w_judge(INVARIANT_I_2_5, p.device0 < p.device1);
}

// I-2.4 的指针那一半：头部 MAC / nonce 全 0（image::judge_pointer_mac_and_nonce_are_zero），以及算法类型、压缩算法码、压后长度
// 是第一版的取值 0（image::judge_pointer_encryption_and_compression_fields_take_the_first_version_values；head_ok 是
// crash_judge_common.wgsl 的 pointer_encryption_and_compression_fields_are_the_first_version，连 MAC / nonce 一并判）
fn w_judge_pointer_head(p: Ptr) {
    w_judge(INVARIANT_I_2_4, p.mac_nonce_zero);
    w_judge(INVARIANT_I_2_4, p.head_ok);
}

// I-2.4 的指针那一半另一格：头部 extent 偏移为 0（image::judge_pointer_extent_offset_is_zero；第一版指针指整个单元，与加密开不开无关）
fn w_judge_pointer_extent_offset(p: Ptr) {
    w_judge(INVARIANT_I_2_4, p.extent_offset_zero);
}

// walk.rs judge_unit_header：按期望的类的字段表判 I-1.6、I-2.4（三格）、I-2.3、I-1.4；交回头能不能用
fn w_judge_unit_header(s: Slice, expected_class: u32, fsid_low: vec2<u32>) -> bool {
    let unit_class = rd8(s.base, 6u);
    var copy_matches = true;
    if (unit_class == 1u || unit_class == 3u) {
        copy_matches = rd8(s.base, 42u) == unit_class;
    }
    w_judge(INVARIANT_I_1_6, unit_class >= 1u && unit_class <= 3u && unit_class == expected_class && rd8(s.base, 7u) == 0u && copy_matches);
    var header_layout = 1u;
    var fsid_offset = 60u + 2u * rd8(s.base, 51u);
    if (expected_class == 1u) {
        header_layout = 0u;
        fsid_offset = 83u;
    } else if (expected_class == 3u) {
        header_layout = 2u;
        fsid_offset = 81u;
    }
    let bits = s.c3 >> (4u * header_layout);
    let header_holds = (bits & 1u) != 0u;
    w_judge(INVARIANT_I_2_4, header_holds);
    if (!header_holds || unit_class != expected_class) {
        w_failures = w_failures + 1u;
        return false;
    }
    let version_recognized = rd16(s.base, 4u) == 1u;
    w_judge(INVARIANT_I_2_4, version_recognized);
    if (!version_recognized) {
        w_failures = w_failures + 1u;
        return false;
    }
    let padding_zero = (bits & 8u) != 0u;
    let payload_holds = (bits & 2u) != 0u;
    w_judge(INVARIANT_I_2_3, padding_zero && payload_holds);
    w_judge(INVARIANT_I_1_4, eq64(rd64(s.base, fsid_offset), fsid_low));
    if (!payload_holds) {
        return true;
    }
    let reserved_zero = (bits & 4u) != 0u;
    w_judge(INVARIANT_I_2_4, reserved_zero);
    if (!reserved_zero) {
        w_failures = w_failures + 1u;
        return false;
    }
    return true;
}

// I-1.2 / I-4.2：头里的出生身份与引用它的指针记的相同；按已发布谓词判成已发布。诞生代号记在 (unit_device, unit_slot)：读回那一份的落点
fn w_judge_birth_identity(s: Slice, p: Ptr, unit_device: u32, unit_slot: vec2<u32>) {
    let unit_class = rd8(s.base, 6u);
    var birth_txg = vec2<u32>(0u, 0u);
    var instance = 0u;
    var tail = vec2<u32>(0u, 0u);
    var transaction = vec2<u32>(0u, 0u);
    if (unit_class == 1u) {
        birth_txg = rd64(s.base, 75u);
        instance = rd32(s.base, 91u);
        transaction = rd48(s.base, 95u);
        tail = transaction;
    } else if (unit_class == 2u) {
        let key_span = 2u * rd8(s.base, 51u);
        birth_txg = rd64(s.base, 52u + key_span);
        instance = rd32(s.base, 68u + key_span);
        tail = vec2<u32>(rd32(s.base, 72u + key_span), 0u);
    } else if (unit_class == 3u) {
        birth_txg = rd64(s.base, 73u);
        instance = rd32(s.base, 93u);
        tail = vec2<u32>(rd32(s.base, 103u), 0u);
    } else {
        return;
    }
    // 记下这个单元的诞生代号（按读回那一份的落点认，与走过的单元表同一个键），再判 I-4.9
    if (w_context == 0u) {
        let visited_index = w_visited_index_in_the_main_walk(unit_device, unit_slot);
        if (visited_index != NONE) {
            w_note_the_birth_txg_of_a_visited_row(visited_index, 1u, birth_txg);
        }
    }
    w_judge_unit_birth_against_an_older_candidate_root(birth_txg);
    w_judge(INVARIANT_I_1_2, eq64(birth_txg, p.birth_txg) && instance == p.instance && eq64(tail, p.tail));
    w_judge(INVARIANT_I_4_2, w_published(birth_txg, instance, unit_class == 1u, transaction));
}

// lib::check_index_node_keys：自述 key 宽等于形态宽、条目 key 严格递增、区间贴紧首末条目
fn w_check_index_node_keys(n: Node, schema: u32) -> bool {
    if (schema_key_width(schema) != n.key_width) {
        return false;
    }
    for (var e = 1u; e < n.entry_count; e = e + 1u) {
        if (schema_compare(schema, n.base, entry_offset(n, e - 1u), n.base, entry_offset(n, e)) != 0u) {
            return false;
        }
    }
    if (n.entry_count > 0u) {
        if (schema_compare(schema, n.base, entry_offset(n, 0u), n.base, n.smallest_offset) != 1u
            || schema_compare(schema, n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset) != 1u) {
            return false;
        }
    }
    return true;
}

// lib::check_internal_node_separators：自述 key 宽等于形态宽、节点不空、分隔 key 严格递增
fn w_check_internal_node_separators(n: Node, schema: u32) -> bool {
    if (schema_key_width(schema) != n.key_width) {
        return false;
    }
    if (n.entry_count == 0u) {
        return false;
    }
    for (var e = 1u; e < n.entry_count; e = e + 1u) {
        if (schema_compare(schema, n.base, entry_offset(n, e - 1u), n.base, entry_offset(n, e)) != 0u) {
            return false;
        }
    }
    return true;
}

// lib::check_separators_of_a_node_whose_subtree_coverage_the_walk_judges：自述 key 宽等于形态宽、分隔 key 严格递增（零条目只判 key 宽）。
// 头里的 key 区间是子树覆盖区间（D18 已定项 2），孩子读回来之后由走读那一方判
fn w_check_separators_of_a_node_whose_subtree_coverage_the_walk_judges(n: Node, schema: u32) -> bool {
    if (schema_key_width(schema) != n.key_width) {
        return false;
    }
    for (var e = 1u; e < n.entry_count; e = e + 1u) {
        if (schema_compare(schema, n.base, entry_offset(n, e - 1u), n.base, entry_offset(n, e)) != 0u) {
            return false;
        }
    }
    return true;
}

// 条目宽 0 而条目数非 0（lib::index_node_view 的 EntryWidthZeroWithEntries）
fn entry_width_zero_with_entries(s: Slice) -> bool {
    if (s.version == NONE || s.length < NODE_BYTES) {
        return false;
    }
    if (rd32(s.base, 0u) != MAGIC_SFSU || rd16(s.base, 4u) != 1u || rd8(s.base, 6u) != 2u || rd8(s.base, 7u) != 0u) {
        return false;
    }
    if ((s.c2 & 11u) != 11u) {
        return false;
    }
    let key_width = rd8(s.base, 51u);
    let header_end = 86u + 2u * key_width;
    let declared = rd16(s.base, 8u);
    let entry_count = rd16(s.base, header_end - 4u);
    let entry_width = rd16(s.base, header_end - 2u);
    return declared == entry_count * entry_width && entry_width >= key_width && entry_width == 0u && entry_count != 0u;
}

// C896（`.claude/kb/checks-owed.md`）：w_read_index_node 命中（这个码 2 节点别的版本先走过、这一次不往下走）时读回的那一份，与先走那一遍
// 解没解得开那个节点（主走读走过的单元表那一行的 C_VISITED_INDEX_NODE_PARSES：头用得了、条目区解得开）。调用方在这一次 w_read_index_node
// 交回之后、下一次之前经 w_index_node_an_earlier_walk_parsed 取节点，照判引用它的父条目定的层级、key 区间与子树覆盖（I-1.1），判完照旧不往下走；
// 命中这一支只判、不记走读失败（walk.rs 的 view_of_a_node_an_earlier_version_walked 与它的调用方）。另起的那几遍（上下文 1）不记走过的单元表
// 那一行、判定本来就丢掉，不取（同 C01 命中照判那几格，w_judge_the_cells_the_reference_decides_on_a_unit_walked_earlier）
var<private> w_index_node_read_back_on_the_last_hit: Slice;

var<private> w_an_earlier_walk_parsed_the_index_node_hit_last: bool;

fn w_note_the_index_node_an_earlier_walk_parsed(s: Slice) {
    w_index_node_read_back_on_the_last_hit = s;
    w_an_earlier_walk_parsed_the_index_node_hit_last = w_visited_row_of_the_unit_asked_last != NONE
        && (sw(C_VISITED_BIRTHS0 + w_visited_row_of_the_unit_asked_last * C_VISITED_BIRTH_WORDS) & C_VISITED_INDEX_NODE_PARSES) != 0u;
}

// 最近一次命中的那个码 2 节点：先走那一遍解得开时同一份字节再解一遍，没解开（或上下文 1）交 bad_node（n.ok 为假）。
// 只在刚交回的 ReadNode.already_walked 为真时取：没命中的那几次 w_read_index_node 不改这两个变量
fn w_index_node_an_earlier_walk_parsed() -> Node {
    if (!w_an_earlier_walk_parsed_the_index_node_hit_last) {
        return bad_node();
    }
    return parse_index_node(w_index_node_read_back_on_the_last_hit);
}

struct ReadNode {
    ok: bool,
    already_walked: bool,
    n: Node,
    // 读回那一份的落点（这个节点在走读里的身份；往下走时记成父单元）。读不出时是第一条位置条目的落点
    unit_device: u32,
    unit_slot: vec2<u32>,
};

// walk.rs read_index_node：读一个码 2 节点并判头、出生身份、树 ID（I-1.3）与 key（I-1.1）
// key_range_reading：0 贴紧首末条目、1 多层树的子树覆盖（叶贴紧、内部只判分隔 key）、2 按位置（只判分隔 key）、
// 3 按位置读 extent 树上段的根（层级 0 的零条目根只判 key 宽，其余同 2；walk.rs 的 PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller）、
// 4 inode 树（层级 0 贴紧，层级 ≥ 1 只判分隔 key、零条目只判 key 宽，区间按叶里的记录号由 w_walk_inode_root 判；walk.rs 的 SubtreeCoverageOfTheInodeTreeJudgedByTheCaller）、
// 5 墓碑树、谱系树、退役树（层级 ≤ 1 贴紧，层级 ≥ 2 同 4 的读法，区间按孩子头由 w_walk_container_tree_root 判；walk.rs 的
// SubtreeCoverageOfAContainerTreeAboveLevelOneJudgedByTheCaller）
// mapping_class：树表单元与映射树自己的节点（自举豁免）是 0、按指针自己的位置读；别的码 2 节点是 MAPPING_CLASS_INDEX_NODE，按这一版中央映射认的位置
// （w_locate）。I-2.5 与指针头部那几格判的是指针自己的字节
fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_reading: u32, registered: bool, mapping_class: u32, fsid_low: vec2<u32>) -> ReadNode {
    var out: ReadNode;
    out.ok = false;
    out.already_walked = false;
    out.n = bad_node();
    out.unit_device = p.device0;
    out.unit_slot = p.slot0;
    if (p.all_zero) {
        return out;
    }
    w_judge_location_order(p);
    w_judge_pointer_head(p);
    w_judge_pointer_extent_offset(p);
    let located = w_locate(p, mapping_class);
    out.unit_device = located.device0;
    out.unit_slot = located.slot0;
    w_note_pointer_references_of_a_located_pointer(located, 1u, w_unit_of_a_located_pointer(located, mapping_class), mapping_class != MAPPING_CLASS_THE_POINTER_ITSELF);
    let s = w_read_unit_to_walk(located, NODE_BYTES);
    if (s.version == NONE) {
        w_failures = w_failures + 1u;
        return out;
    }
    // 「走过」、并进前一版、诞生代号按读回那一份的落点认，「先走过」交回调用方看（walk.rs IndexNodeReading）
    out.unit_device = w_unit_device_read_back;
    out.unit_slot = w_unit_slot_read_back;
    if (!w_visit(out.unit_device, out.unit_slot)) {
        out.already_walked = true;
        w_note_the_index_node_an_earlier_walk_parsed(s);
        w_note_a_unit_an_earlier_version_walked_referenced_again(s, located, out.unit_device, out.unit_slot,
            reference_to_an_index_node_of_a_tree(expected_tree));
        return out;
    }
    if (!w_judge_unit_header(s, 2u, fsid_low)) {
        return out;
    }
    w_note_the_header_the_walk_could_use();
    w_judge_birth_identity(s, located, out.unit_device, out.unit_slot);
    let n = parse_index_node(s);
    if (!n.ok) {
        if (registered && entry_width_zero_with_entries(s)) {
            w_judge(INVARIANT_I_1_10, false);
        }
        w_failures = w_failures + 1u;
        return out;
    }
    w_note_on_the_visited_row_what_the_first_walk_found(C_VISITED_INDEX_NODE_PARSES);
    w_judge(INVARIANT_I_1_3, eq64(n.tree, expected_tree));
    var keys_hold = false;
    if (key_range_reading == 0u || (key_range_reading == 1u && n.level == 0u) || (key_range_reading == 4u && n.level == 0u)
        || (key_range_reading == 5u && n.level <= 1u)) {
        keys_hold = w_check_index_node_keys(n, schema);
    } else if (key_range_reading == 4u || key_range_reading == 5u) {
        keys_hold = w_check_separators_of_a_node_whose_subtree_coverage_the_walk_judges(n, schema);
    } else if (key_range_reading == 3u && n.level == 0u && n.entry_count == 0u) {
        // extent 树上段的根兼叶删完第一个文件之后可以一条条目都没有（D8 已定项 16 第 5 条）：只判 key 宽，不判节点不空
        keys_hold = schema_key_width(schema) == n.key_width;
    } else {
        keys_hold = w_check_internal_node_separators(n, schema);
    }
    w_judge(INVARIANT_I_1_1, keys_hold);
    out.ok = true;
    out.n = n;
    return out;
}

struct PackedRead {
    ok: bool,
    p: Packed,
};

// walk.rs judge_packed_container：头按码 3 判，再 I-1.7（类型、登记的记录宽、记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136）
fn w_judge_packed_container(s: Slice, expected_type: u32, fsid_low: vec2<u32>) -> PackedRead {
    var out: PackedRead;
    out.ok = false;
    out.p = bad_packed();
    if (!w_judge_unit_header(s, 3u, fsid_low)) {
        return out;
    }
    let record_type = rd16(s.base, 51u);
    let count = rd16(s.base, 69u);
    let width = rd16(s.base, 71u);
    let declared = rd16(s.base, 8u);
    var registered_width = NONE;
    if (record_type == 2u) {
        registered_width = 140u;
    } else if (record_type == 4u) {
        registered_width = 88u;
    } else if (record_type == PACKED_TYPE_TOMBSTONE) {
        registered_width = 40u;
    } else if (record_type == PACKED_TYPE_LINEAGE) {
        registered_width = 24u;
    } else if (record_type == PACKED_TYPE_RETIREMENT) {
        registered_width = 34u;
    }
    let legal = record_type == expected_type && registered_width == width && count * width <= declared && declared <= DATA_UNIT_BYTES - 136u;
    w_judge(INVARIANT_I_1_7, legal);
    if (!legal) {
        return out;
    }
    var p = bad_packed();
    p.ok = true;
    p.base = s.base;
    p.birth_tree = rd64(s.base, 43u);
    p.record_type = record_type;
    p.container = rd64(s.base, 53u);
    p.container_birth = rd64(s.base, 61u);
    p.record_count = count;
    p.record_width = width;
    p.birth_txg = rd64(s.base, 73u);
    p.fsid = rd64(s.base, 81u);
    p.instance = rd32(s.base, 93u);
    p.transaction = rd48(s.base, 97u);
    p.birth_sequence = rd32(s.base, 103u);
    out.ok = true;
    out.p = p;
    return out;
}

// 主走读沿最新根的实例表链走没走到末片（链指针记录说「没有下一片」）：walk.rs 的 Walk::abandoned_roots_filtering。走到了，C_INSTANCE_ROWS
// 就是整张表的行、候选集按它滤掉了被崩溃恢复抛弃的根；没走到就滤不全。经交接段交给扫描内核，I-9.29 跨版本那张表比不比读它（判决
// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条：门与候选集的滤法用同一遍走读，不另读一遍）。择根之后、走最新根之前清掉
var<private> w_newest_instance_table_chain_reached_its_last_page: bool;

// 实例表链走读循环最多走几片（walk.rs 的链不设片数上限，走过的单元不再走）
const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;

// 走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、最后那一片的链指针记录还说有下一片：这一张表 GPU 走不完，记判不了交 CPU
// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第二节 H-c 那一行与第三节第 1 条：不当「没走到末片」，
// 否则门关、I-9.29 那张表不比，CPU 走到末片照比，两边分叉）。「条目太多」一类，站点号与别处不撞
const UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (61u << 8u);

// walk.rs walk_instance_table_chain（每一片判 I-3.14 四格，不成立的一片解不开：行不收、链不往下走；挂载根那一张另判行：
// I-3.8 的行那一半，行记进 C_INSTANCE_ROWS；走到末片记进 w_newest_instance_table_chain_reached_its_last_page）
fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>) {
    var pointer = first;
    var page_index = vec2<u32>(0u, 0u);
    let parent_valid = w_parent_valid;
    let parent_device = w_parent_device;
    let parent_slot = w_parent_slot;
    var rows_unique = true;
    var rows_below = true;
    var pages = 0u;
    // 循环里每个提前出口都在 page 加一之前 break：走满上限才落到 page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED
    var page = 0u;
    for (; page < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; page = page + 1u) {
        w_judge_location_order(pointer);
        w_judge_pointer_head(pointer);
        w_judge_pointer_extent_offset(pointer);
        w_note_pointer_references(pointer, 2u);
        let s = w_read_unit_to_walk(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            w_failures = w_failures + 1u;
            break;
        }
        // 「走过」按读回、自证过的那一份的落点认（walk.rs placement_it_was_read_from，攻方 H-b）；并进前一版、父单元、这一片的诞生代号同一个落点
        // （攻方 allrbmap）
        let device_read_from = w_unit_device_read_back;
        let slot_read_from = w_unit_slot_read_back;
        if (!w_visit(device_read_from, slot_read_from)) {
            w_note_a_unit_an_earlier_version_walked_referenced_again(s, pointer, device_read_from, slot_read_from,
                reference_to_an_instance_table_page(page_index));
            break;
        }
        w_parent_valid = true;
        w_parent_device = device_read_from;
        w_parent_slot = slot_read_from;
        let packed = w_judge_packed_container(s, 4u, fsid_low);
        if (!packed.ok) {
            break;
        }
        w_note_the_header_the_walk_could_use();
        if (w_context == 0u) {
            appearance_note_the_birth_txg_of_an_instance_table_page(device_read_from, slot_read_from, rd64(s.base, 73u));
        }
        let p = packed.p;
        // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 3）
        w_judge(INVARIANT_I_1_12, eq64(pointer.birth_tree, p.birth_tree));
        // I-1.2 遍历方向那一半（walk.rs walk_instance_table_chain 每一片都调 judge_birth_identity_against_the_pointer）：头里的诞生代号（码 3 偏移 73）、
        // 写序实例（93）与出生序号（103）与指着这一片的指针记的相同，偏移同 w_judge_birth_identity 码 3 那一支。只判 I-1.2、不判 I-4.2：
        // 打包记录类型 4 的实例表单元按 I-1.2 那一行的例外只免谓词那一半（判决 `research/prompts/m3-batch28-code-r1-main-verification.md`
        // 第二节「实例表链上的 I-1.2」那一行、第三节第 2 条，攻方改法 F-IT）
        w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance
            && eq64(vec2<u32>(rd32(s.base, 103u), 0u), pointer.tail));
        let identity_holds = is_zero64(p.birth_tree) && p.record_type == 4u && eq64(p.container, page_index) && is_zero64(p.container_birth);
        w_judge(INVARIANT_I_1_1, identity_holds);
        if (!identity_holds) {
            w_failures = w_failures + 1u;
            break;
        }
        var chain = 2u;
        if (p.record_count > 0u) {
            chain = w_chain_record_kind(p.base, packed_record_offset(p, p.record_count - 1u), p.record_width);
        }
        // I-3.14 四格（walk.rs judge_instance_table_page_record_forms 三格，加最后一条是合法的链指针记录）：kind 只有 0 或 1、
        // kind 1 不在最后一条之前、kind 0 行偏移 21 的 flags 为 0；走到的每一片都判（不只挂载根那一张）
        var kinds_are_registered = true;
        var chain_records_only_last = true;
        var row_flags_are_zero = true;
        for (var r = 0u; r < p.record_count; r = r + 1u) {
            let o = packed_record_offset(p, r);
            let kind = rd8(p.base, o);
            if (kind == 0u) {
                if (rd8(p.base, o + 21u) != 0u) {
                    row_flags_are_zero = false;
                }
            } else if (kind == 1u) {
                if (r + 1u < p.record_count) {
                    chain_records_only_last = false;
                }
            } else {
                kinds_are_registered = false;
            }
        }
        w_judge(INVARIANT_I_3_14, kinds_are_registered);
        w_judge(INVARIANT_I_3_14, chain_records_only_last);
        w_judge(INVARIANT_I_3_14, row_flags_are_zero);
        // 最后一条是合法的链指针记录那一格：有下一片而位置指针头部不是第一版的取值（第五形）同样 chain == 2u（w_chain_record_kind）
        w_judge(INVARIANT_I_3_14, chain != 2u);
        // I-3.14 不成立的一片解不开（给 D18 已定项 11 补的那一句）：行不收、链不往下走、不记走读失败、没走到末片
        if (!(kinds_are_registered && chain_records_only_last && row_flags_are_zero)) {
            break;
        }
        if (chain == 2u) {
            break;
        }
        // 行那一半的账
        if (judge_rows) {
            for (var r = 0u; r < p.record_count; r = r + 1u) {
                let o = packed_record_offset(p, r);
                let kind = rd8(p.base, o);
                if (kind == 0u) {
                    let instance = rd32(p.base, o + 1u);
                    // 这个实例之前已有一行：I-3.8 行唯一不成立（表满时照判，行不记、站点 8 判不了）
                    if (w_append_an_instance_row(instance, rd64(p.base, o + 5u), rd64(p.base, o + 13u))) {
                        rows_unique = false;
                    }
                    if (instance >= w_mount_instance) {
                        rows_below = false;
                    }
                }
            }
            pages = pages + 1u;
            if (chain == 0u) {
                w_newest_instance_table_chain_reached_its_last_page = true;
            }
        }
        if (chain != 1u) {
            break;
        }
        pointer = parse_node_pointer(p.base, packed_record_offset(p, p.record_count - 1u) + 2u);
        page_index = add64(page_index, u64_of(1u));
    }
    if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {
        undecidable = UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED;
    }
    w_parent_valid = parent_valid;
    w_parent_device = parent_device;
    w_parent_slot = parent_slot;
    // I-3.8 只在走到末片时判（C742「任一片解不开就报不适用」）：没走到末片时 CPU 报不适用、不置违例位也不置判到位，
    // 不照已收到的前几片的行判（walk.rs walk_instance_table_chain 末尾那一处 match）
    if (judge_rows && pages > 0u && w_newest_instance_table_chain_reached_its_last_page) {
        w_judge(INVARIANT_I_3_8, rows_unique && rows_below);
    }
}

// 树表条目里 I-9.16 四道（walk.rs judge_tree_table_entries_ordering）：① 盘上相邻两条树 ID 严格升序；条目宽读得出种类时
// ③ ④（walk.rs judge_tree_table_entry_kind_counts）与 ② 发号次序。只有违例位，几道的先后不影响结论
fn w_judge_tree_table_entries_ordering(n: Node) {
    for (var e = 1u; e < n.entry_count; e = e + 1u) {
        if (!lt64(rd64(n.base, entry_offset(n, e - 1u)), rd64(n.base, entry_offset(n, e)))) {
            w_judge(INVARIANT_I_9_16, false);
            return;
        }
    }
    if (n.entry_width < 12u) {
        // 条目短到读不出种类：② ③ ④ 判不了，CPU 记射程里有一部分没判、再判一次成立（judge_tree_table_entries_ordering）；0 条的表 all() 为真、不记
        if (n.entry_count > 0u) {
            w_mark_part_of_the_range_not_judged(INVARIANT_I_9_16);
        }
        w_judge(INVARIANT_I_9_16, true);
        return;
    }
    // ③ ④：0 条的表不判
    if (n.entry_count > 0u) {
        // ③：每一种种类码（登记过的与没登记的都算）至多一条；码 14、15 一个快照一对、可以有多条，不按至多一条判，改按头 ID 一一配对判
        // （walk.rs judge_tree_table_entry_kind_counts 与 judge_snapshot_tree_table_entries_paired_one_to_one）
        for (var later_entry = 1u; later_entry < n.entry_count; later_entry = later_entry + 1u) {
            let later_kind = rd16(n.base, entry_offset(n, later_entry) + 10u);
            if (later_kind == TREE_KIND_SNAPSHOT_INODE_ROOT || later_kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
                continue;
            }
            for (var earlier_entry = 0u; earlier_entry < later_entry; earlier_entry = earlier_entry + 1u) {
                if (rd16(n.base, entry_offset(n, earlier_entry) + 10u) == later_kind) {
                    w_judge(INVARIANT_I_9_16, false);
                    return;
                }
            }
        }
        w_judge_snapshot_tree_table_entries_paired_one_to_one(n);
        // ④：树表非空时进树表的十二种（发号次序 1,2,3,4,6,7,8,9,10,11,12,13）各至少一条，与 ③ 合起来各恰好一条
        // （判决 m3-formal-code-b26-r1 第二节 J1 C1：core 重建一版时十二种各要一条）
        for (var position = 0u; position < TREE_KINDS_OF_THE_TREE_TABLE; position = position + 1u) {
            let required_kind = tree_kind_in_the_issuing_order(position);
            var required_kind_is_present = false;
            for (var entry_index = 0u; entry_index < n.entry_count; entry_index = entry_index + 1u) {
                if (rd16(n.base, entry_offset(n, entry_index) + 10u) == required_kind) {
                    required_kind_is_present = true;
                }
            }
            if (!required_kind_is_present) {
                w_judge(INVARIANT_I_9_16, false);
                return;
            }
        }
    }
    // ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12,13（walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）
    var previous_tree = vec2<u32>(0u, 0u);
    var have_previous = false;
    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
        let kind = tree_kind_in_the_issuing_order(k);
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            if (rd16(n.base, o + 10u) == kind) {
                let tree = rd64(n.base, o);
                if (have_previous && !lt64(previous_tree, tree)) {
                    w_judge(INVARIANT_I_9_16, false);
                    return;
                }
                previous_tree = tree;
                have_previous = true;
                break;
            }
        }
    }
    w_judge(INVARIANT_I_9_16, true);
}

// I-9.16 ③ 的快照那一格（walk.rs judge_snapshot_tree_table_entries_paired_one_to_one，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
// 第三节 S1）：码 14 与码 15 条数相等，按盘上次序每条码 15 的头 ID 恰是一条还没被前面的码 15 配上的码 14 的树 ID。一条码 14、15 都没有时不判；
// 条目宽不到头 ID 那 8 字节时射程里有一部分判不了。树表条目装不过 128 条（16 KiB 的码 2 节点、条目至少 124 字节），配上没有记在四个字的位图里
fn w_judge_snapshot_tree_table_entries_paired_one_to_one(n: Node) {
    var snapshot_inode_roots = 0u;
    var snapshot_extent_roots = 0u;
    for (var e = 0u; e < n.entry_count; e = e + 1u) {
        let kind = rd16(n.base, entry_offset(n, e) + 10u);
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
            snapshot_inode_roots = snapshot_inode_roots + 1u;
        } else if (kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            snapshot_extent_roots = snapshot_extent_roots + 1u;
        }
    }
    if (snapshot_inode_roots == 0u && snapshot_extent_roots == 0u) {
        return;
    }
    if (n.entry_width < TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8u) {
        w_mark_part_of_the_range_not_judged(INVARIANT_I_9_16);
        return;
    }
    if (snapshot_inode_roots != snapshot_extent_roots) {
        w_judge(INVARIANT_I_9_16, false);
        return;
    }
    var paired: array<u32, 4>;
    for (var word = 0u; word < 4u; word = word + 1u) {
        paired[word] = 0u;
    }
    var every_extent_root_has_its_own_inode_root = true;
    for (var e = 0u; e < n.entry_count; e = e + 1u) {
        let o = entry_offset(n, e);
        if (rd16(n.base, o + 10u) != TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        let head = rd64(n.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        var paired_position = NONE;
        for (var candidate = 0u; candidate < n.entry_count && candidate < 128u; candidate = candidate + 1u) {
            let candidate_o = entry_offset(n, candidate);
            if (rd16(n.base, candidate_o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT && eq64(rd64(n.base, candidate_o), head)
                && (paired[candidate / 32u] & (1u << (candidate % 32u))) == 0u) {
                paired_position = candidate;
                break;
            }
        }
        if (paired_position == NONE) {
            every_extent_root_has_its_own_inode_root = false;
            break;
        }
        paired[paired_position / 32u] = paired[paired_position / 32u] | (1u << (paired_position % 32u));
    }
    w_judge(INVARIANT_I_9_16, every_extent_root_has_its_own_inode_root);
}

// 树表条目的头 ID（walk.rs judge_tree_table_entry_head_identifiers，I-9.27）：inode 树等于自己的树 ID；extent 树、deleted_inodes、墓碑树、退役树、
// 快照 inode 根（码 14）等于同一张表里某一条 inode 树条目的树 ID（一条都没有就红）；快照 extent 根（码 15）等于同一张表里某一条码 14 条目的树 ID；
// 3、4、6、7、8、11、12 等于 0；别的种类不判。条目宽不到头 ID 那 8 字节时整张表不判
fn w_judge_tree_table_entry_head_identifiers(n: Node) {
    if (n.entry_width < TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8u) {
        if (n.entry_count > 0u) {
            w_mark_part_of_the_range_not_judged(INVARIANT_I_9_27);
        }
        return;
    }
    for (var entry_index = 0u; entry_index < n.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(n, entry_index);
        let tree = rd64(n.base, entry);
        let kind = rd16(n.base, entry + 10u);
        let head = rd64(n.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        if (kind == TREE_KIND_INODE) {
            w_judge(INVARIANT_I_9_27, eq64(head, tree));
        } else if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_DELETED_INODES || kind == TREE_KIND_TOMBSTONE
            || kind == TREE_KIND_RETIREMENT || kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            // 码 15 的头 ID 指的是一条码 14（配对的那一条），别的这几种指一条 inode 树条目
            var kind_the_head_names = TREE_KIND_INODE;
            if (kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
                kind_the_head_names = TREE_KIND_SNAPSHOT_INODE_ROOT;
            }
            var names_an_inode_tree_of_this_table = false;
            for (var other_index = 0u; other_index < n.entry_count; other_index = other_index + 1u) {
                let other = entry_offset(n, other_index);
                if (rd16(n.base, other + 10u) == kind_the_head_names && eq64(rd64(n.base, other), head)) {
                    names_an_inode_tree_of_this_table = true;
                }
            }
            w_judge(INVARIANT_I_9_27, names_an_inode_tree_of_this_table);
        } else if (kind == TREE_KIND_ALLOCATION || kind == TREE_KIND_ACCOUNTING || kind == TREE_KIND_LIVELIST
            || kind == TREE_KIND_SPARSE_SIDE_TABLE || kind == TREE_KIND_DEADLIST || kind == TREE_KIND_LINEAGE
            || kind == TREE_KIND_LOGGED_OPERATIONS) {
            w_judge(INVARIANT_I_9_27, is_zero64(head));
        }
    }
}

// 数据单元（walk.rs walk_extent_data_pointer）
fn w_walk_extent_data_pointer(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>, tree: vec2<u32>, fsid_low: vec2<u32>) {
    let raw = parse_data_pointer(base, o);
    w_judge_location_order(raw);
    w_judge_pointer_head(raw);
    w_judge_pointer_extent_offset(raw);
    // 数据单元落在哪按这一版中央映射里码 1 的那把 key 认（I-2.5 与指针头部那几格判的是指针自己的字节）
    let p = w_locate(raw, MAPPING_CLASS_DATA_UNIT);
    w_note_pointer_references_of_a_located_pointer(p, 2u, w_unit_of_a_located_pointer(p, MAPPING_CLASS_DATA_UNIT), true);
    let s = w_read_unit_to_walk(p, DATA_UNIT_BYTES);
    if (s.version == NONE) {
        w_failures = w_failures + 1u;
        return;
    }
    let unit_device = w_unit_device_read_back;
    let unit_slot = w_unit_slot_read_back;
    if (!w_visit_the_copy_read_back()) {
        w_note_a_unit_an_earlier_version_walked_referenced_again(s, p, unit_device, unit_slot,
            reference_to_a_data_unit_of_an_extent_key(tree, inode, offset));
        return;
    }
    if (!w_judge_unit_header(s, 1u, fsid_low)) {
        return;
    }
    w_note_the_header_the_walk_could_use();
    w_judge_birth_identity(s, p, unit_device, unit_slot);
    // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 1 五元组里的树）
    w_judge(INVARIANT_I_1_12, eq64(p.birth_tree, rd64(s.base, 43u)));
    w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
    w_judge(INVARIANT_I_1_1, eq64(rd64(s.base, 51u), inode) && eq64(rd64(s.base, 67u), offset) && eq64(rd64(s.base, 43u), tree));
    // I-9.10 只拿活头的数据单元对活头的 inode 记录：快照的树里同一个 inode 号可以是删过、重发之后的另一个对象
    if (w_context == 0u && !w_walking_the_trees_of_a_snapshot) {
        if (w_data_object_count >= C_MAX_DATA_OBJECTS) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (9u << 8u);
            return;
        }
        let birth = rd64(s.base, 59u);
        sws(C_DATA_OBJECTS + w_data_object_count * 4u, inode.x);
        sws(C_DATA_OBJECTS + w_data_object_count * 4u + 1u, inode.y);
        sws(C_DATA_OBJECTS + w_data_object_count * 4u + 2u, birth.x);
        sws(C_DATA_OBJECTS + w_data_object_count * 4u + 3u, birth.y);
        w_data_object_count = w_data_object_count + 1u;
    }
}

// 按位置寻址的节点（分配记录树、extent 树上段与下段）头里的层级是它的位置要的那一层、头里的 key 区间是它的位置规定罩的那一段（range_ok）：
// I-1.1，层级不对时区间不比；交回层级对不对（walk.rs judge_the_level_and_the_key_range_the_position_prescribes）。第一次走到（w_position_and_entry_width_hold）
// 与别的版本先走过的节点（C896：位置由这一版的父条目定，别的版本按它自己的父条目走过它，证明不了这一版也把它放对了）共用这一判
fn w_judge_the_level_and_the_key_range_the_position_prescribes(n: Node, level: u32, range_ok: bool) -> bool {
    w_judge(INVARIANT_I_1_1, n.level == level);
    if (n.level != level) {
        return false;
    }
    w_judge(INVARIANT_I_1_1, range_ok);
    return true;
}

// 按位置寻址的节点：层级、区间、条目宽（walk.rs position_and_entry_width_hold）；层级不对记走读失败；交回还往不往下走
fn w_position_and_entry_width_hold(n: Node, level: u32, range_ok: bool, entry_width: u32) -> bool {
    if (!w_judge_the_level_and_the_key_range_the_position_prescribes(n, level, range_ok)) {
        w_failures = w_failures + 1u;
        return false;
    }
    w_judge(INVARIANT_I_1_10, n.entry_width == entry_width);
    return n.entry_width == entry_width;
}

// ── 分配记录树（walk_allocation_record_tree）：显式栈
struct AllocationWalkFrame {
    n: Node,
    node: AllocationNode,
    child_index: u32,
    has_previous_child: bool,
    previous_child_device: u32,
    previous_child_index: vec2<u32>,
    parent_valid: bool,
    parent_device: u32,
    parent_slot: vec2<u32>,
};

var<private> allocation_walk_frames: array<AllocationWalkFrame, 12>;

// 分配记录树一个节点头里的 key 区间是不是它的位置规定罩的那一段（walk.rs allocation_record_tree_cell_range）：根罩全零到全 0xFF，
// 根之下 (层级, 盘, 序号) 罩那一格的首槽到末槽
fn allocation_node_range_is_its_cell(n: Node, cell: AllocationNode) -> bool {
    if (cell.is_root) {
        var range_is_the_whole_key_space = all_zero(n.base, n.smallest_offset, 10u);
        for (var i = 0u; i < 10u; i = i + 1u) {
            if (rd8(n.base, n.largest_offset + i) != 0xFFu) {
                range_is_the_whole_key_space = false;
            }
        }
        return range_is_the_whole_key_space;
    }
    return allocation_key_bytes_equal(n.base, n.smallest_offset, cell.device, allocation_first_slot(cell.level, cell.index))
        && allocation_key_bytes_equal(n.base, n.largest_offset, cell.device, allocation_last_slot(cell.level, cell.index));
}

// 分配记录树一层的条目宽：叶 20（walk.rs allocation_record_bytes）、内部节点 96（ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES）
fn allocation_record_tree_entry_width(level: u32) -> u32 {
    if (level == 0u) {
        return 20u;
    }
    return 96u;
}

// C896：别的版本先走过的分配记录树节点（w_read_index_node 命中）又被这一版的父条目引用：按这一条父条目给的位置照判层级与区间（I-1.1），
// 只判、不往下走：不记走读失败、条目宽（只按单元自己定）不重判
fn w_judge_the_position_of_an_allocation_node_an_earlier_version_walked(cell: AllocationNode) {
    let n = w_index_node_an_earlier_walk_parsed();
    if (n.ok) {
        _ = w_judge_the_level_and_the_key_range_the_position_prescribes(n, cell.level, allocation_node_range_is_its_cell(n, cell));
    }
}

fn w_walk_allocation_record_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: vec2<u32>) {
    let root_level = allocation_root_level();
    if (root_level == 255u) {
        w_failures = w_failures + 1u;
        return;
    }
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_node: AllocationNode;
    pending_node.is_root = true;
    pending_node.level = root_level;
    pending_node.device = NONE;
    pending_node.index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            let read = w_read_index_node(pending_pointer, tree, 4u, 2u, true, MAPPING_CLASS_INDEX_NODE, fsid_low);
            if (!read.ok) {
                if (read.already_walked) {
                    w_judge_the_position_of_an_allocation_node_an_earlier_version_walked(pending_node);
                }
                if (depth == 0u) { break; }
                allocation_walk_frames[depth - 1u].child_index = allocation_walk_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            let n = read.n;
            if (!w_position_and_entry_width_hold(n, pending_node.level, allocation_node_range_is_its_cell(n, pending_node),
                allocation_record_tree_entry_width(pending_node.level))) {
                if (depth == 0u) { break; }
                allocation_walk_frames[depth - 1u].child_index = allocation_walk_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (pending_node.level == 0u) {
                let leaf_first = allocation_first_slot(0u, pending_node.index);
                let leaf_last = allocation_last_slot(0u, pending_node.index);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let device = rd32(n.base, o);
                    let slot = rd48(n.base, o + 4u);
                    // 跨度段去掉已释放标志与隔离记号（0x8000、0x4000，D19（块指针的结构与宽度预算） 已定项 5）：walk.rs parse_allocation_record
                    let span = rd16(n.base, o + 10u) & 0x3FFFu;
                    var fits = !pending_node.is_root && device == pending_node.device && span > 0u && le64(leaf_first, slot);
                    if (fits) {
                        fits = le64(add64(slot, u64_of(span - 1u)), leaf_last);
                    }
                    w_judge(INVARIANT_I_1_1, fits);
                }
                if (depth == 0u) { break; }
                allocation_walk_frames[depth - 1u].child_index = allocation_walk_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return;
            }
            allocation_walk_frames[depth].n = n;
            allocation_walk_frames[depth].node = pending_node;
            allocation_walk_frames[depth].child_index = 0u;
            allocation_walk_frames[depth].has_previous_child = false;
            allocation_walk_frames[depth].parent_valid = w_parent_valid;
            allocation_walk_frames[depth].parent_device = w_parent_device;
            allocation_walk_frames[depth].parent_slot = w_parent_slot;
            w_parent_valid = true;
            w_parent_device = read.unit_device;
            w_parent_slot = read.unit_slot;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = allocation_walk_frames[top].n;
        let parent = allocation_walk_frames[top].node;
        if (allocation_walk_frames[top].child_index >= n.entry_count) {
            w_parent_valid = allocation_walk_frames[top].parent_valid;
            w_parent_device = allocation_walk_frames[top].parent_device;
            w_parent_slot = allocation_walk_frames[top].parent_slot;
            depth = depth - 1u;
            if (depth == 0u) { break; }
            allocation_walk_frames[depth - 1u].child_index = allocation_walk_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = allocation_walk_frames[top].child_index;
        let o = entry_offset(n, e);
        let child_level = parent.level - 1u;
        let device = rd32(n.base, o);
        let slot = rd48(n.base, o + 4u);
        let child_span = allocation_span(child_level);
        let qr = divmod64(slot, child_span);
        var child_ok = is_zero64(qr.remainder) && device_is_in_the_pool(device);
        var child: AllocationNode;
        child.is_root = false;
        child.level = child_level;
        child.device = device;
        child.index = qr.quotient;
        if (child_ok && !parent.is_root) {
            let parent_index = divmod64(child.index, u64_of(ALLOCATION_INTERNAL_FANOUT)).quotient;
            if (parent.device != device || !eq64(parent_index, parent.index)) {
                child_ok = false;
            }
        }
        if (child_ok && allocation_walk_frames[top].has_previous_child) {
            let previous_device = allocation_walk_frames[top].previous_child_device;
            let previous_index = allocation_walk_frames[top].previous_child_index;
            if (!(previous_device < device || (previous_device == device && lt64(previous_index, child.index)))) {
                child_ok = false;
            }
        }
        w_judge(INVARIANT_I_1_1, child_ok);
        if (!child_ok) {
            w_failures = w_failures + 1u;
            allocation_walk_frames[top].child_index = e + 1u;
            continue;
        }
        allocation_walk_frames[top].has_previous_child = true;
        allocation_walk_frames[top].previous_child_device = device;
        allocation_walk_frames[top].previous_child_index = child.index;
        pending_pointer = parse_node_pointer(n.base, o + 10u);
        w_judge_location_order(pending_pointer);
        pending_node = child;
        enter = true;
    }
}

// ── extent 树（walk_extent_upper_node / walk_extent_lower_node）：上段按 inode 号、下段每个文件一棵按数据单元号，各一套帧；
// 下段从上段叶条目处就地走（与 CPU 同一个次序：visited 谁先到谁判）
struct ExtentWalkFrame {
    n: Node,
    level: u32,
    index: vec2<u32>,
    child_index: u32,
    has_previous_child: bool,
    previous_child_index: vec2<u32>,
    parent_valid: bool,
    parent_device: u32,
    parent_slot: vec2<u32>,
};

var<private> extent_upper_frames: array<ExtentWalkFrame, 12>;

var<private> extent_lower_frames: array<ExtentWalkFrame, 12>;

fn w_extent_range_ok(n: Node, inode_first: vec2<u32>, offset_first: vec2<u32>, inode_last: vec2<u32>, offset_last: vec2<u32>) -> bool {
    return extent_key_equals(n.base, n.smallest_offset, inode_first, offset_first) && extent_key_equals(n.base, n.largest_offset, inode_last, offset_last);
}

// extent 树一个节点的位置 (层级, 序号) 规定罩的那一段首末（下段按数据单元号，上段按 inode 号）
struct ExtentCell {
    first: vec2<u32>,
    last: vec2<u32>,
};

fn extent_lower_cell(level: u32, index: vec2<u32>) -> ExtentCell {
    let span = extent_lower_span(level);
    let first = mul64_64_saturating(index, span);
    return ExtentCell(first, add64_saturating(first, sub64(span, u64_of(1u))));
}

// 下段节点头里的 key 区间是不是位置规定罩的那一段：(这个文件, 首个数据单元的起点偏移) 到 (这个文件, 末个数据单元的末字节偏移)
fn extent_lower_node_range_is_its_cell(n: Node, inode: vec2<u32>, cell: ExtentCell) -> bool {
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    let first_offset = mul64_64_saturating(cell.first, payload);
    let last_offset = sub64_saturating(mul64_64_saturating(add64_saturating(cell.last, u64_of(1u)), payload), u64_of(1u));
    return w_extent_range_ok(n, inode, first_offset, inode, last_offset);
}

// 下段一层的条目宽：叶 112、内部节点 110
fn extent_lower_entry_width(level: u32) -> u32 {
    if (level == 0u) {
        return 112u;
    }
    return 110u;
}

// C896：别的版本先走过的下段节点（w_read_index_node 命中）又被这一版引用：按这一条父条目给的位置（文件取上段叶条目的、层级与序号取父节点条目的，
// 根那一个的层级取它自己头里的）照判层级与区间（I-1.1），只判、不往下走：不记走读失败、条目宽不重判
fn w_judge_the_position_of_an_extent_lower_node_an_earlier_version_walked(inode: vec2<u32>, level_given_by_the_parent: u32, index: vec2<u32>) {
    let n = w_index_node_an_earlier_walk_parsed();
    if (!n.ok) {
        return;
    }
    var level = level_given_by_the_parent;
    if (level == NONE) {
        level = n.level;
    }
    _ = w_judge_the_level_and_the_key_range_the_position_prescribes(n, level, extent_lower_node_range_is_its_cell(n, inode, extent_lower_cell(level, index)));
}

// 一个文件的下段（walk_extent_lower_node）
fn w_walk_extent_lower(root_pointer: Ptr, inode: vec2<u32>, tree: vec2<u32>, fsid_low: vec2<u32>) {
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    loop {
        if (enter) {
            let read = w_read_index_node(pending_pointer, tree, 3u, 2u, true, MAPPING_CLASS_INDEX_NODE, fsid_low);
            if (!read.ok) {
                if (read.already_walked) {
                    w_judge_the_position_of_an_extent_lower_node_an_earlier_version_walked(inode, pending_level, pending_index);
                }
                if (depth == 0u) { break; }
                extent_lower_frames[depth - 1u].child_index = extent_lower_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            let n = read.n;
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let cell = extent_lower_cell(level, pending_index);
            let first = cell.first;
            let last = cell.last;
            if (!w_position_and_entry_width_hold(n, level, extent_lower_node_range_is_its_cell(n, inode, cell), extent_lower_entry_width(level))) {
                if (depth == 0u) { break; }
                extent_lower_frames[depth - 1u].child_index = extent_lower_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (level == 0u) {
                let parent_valid = w_parent_valid;
                let parent_device = w_parent_device;
                let parent_slot = w_parent_slot;
                w_parent_valid = true;
                w_parent_device = read.unit_device;
                w_parent_slot = read.unit_slot;
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let record_inode = extent_key_inode(n.base, o);
                    let offset = extent_key_offset(n.base, o);
                    let qr = divmod64(offset, payload);
                    let key_ok = is_zero64(extent_key_locality(n.base, o)) && eq64(record_inode, inode) && is_zero64(qr.remainder)
                        && le64(first, qr.quotient) && le64(qr.quotient, last);
                    w_judge(INVARIANT_I_1_1, key_ok);
                    w_walk_extent_data_pointer(n.base, o + 24u, record_inode, offset, tree, fsid_low);
                }
                w_parent_valid = parent_valid;
                w_parent_device = parent_device;
                w_parent_slot = parent_slot;
                if (depth == 0u) { break; }
                extent_lower_frames[depth - 1u].child_index = extent_lower_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return;
            }
            extent_lower_frames[depth].n = n;
            extent_lower_frames[depth].level = level;
            extent_lower_frames[depth].index = pending_index;
            extent_lower_frames[depth].child_index = 0u;
            extent_lower_frames[depth].has_previous_child = false;
            extent_lower_frames[depth].parent_valid = w_parent_valid;
            extent_lower_frames[depth].parent_device = w_parent_device;
            extent_lower_frames[depth].parent_slot = w_parent_slot;
            w_parent_valid = true;
            w_parent_device = read.unit_device;
            w_parent_slot = read.unit_slot;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_lower_frames[top].n;
        if (extent_lower_frames[top].child_index >= n.entry_count) {
            w_parent_valid = extent_lower_frames[top].parent_valid;
            w_parent_device = extent_lower_frames[top].parent_device;
            w_parent_slot = extent_lower_frames[top].parent_slot;
            depth = depth - 1u;
            if (depth == 0u) { break; }
            extent_lower_frames[depth - 1u].child_index = extent_lower_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_lower_frames[top].child_index;
        let o = entry_offset(n, e);
        let level = extent_lower_frames[top].level;
        let child_span = extent_lower_span(level - 1u);
        let child_offset_span = mul64_64_saturating(child_span, payload);
        let offset = extent_key_offset(n.base, o);
        let qr = divmod64(offset, child_offset_span);
        var child_ok = is_zero64(extent_key_locality(n.base, o)) && eq64(extent_key_inode(n.base, o), inode) && is_zero64(qr.remainder);
        let child_index = qr.quotient;
        if (child_ok && !eq64(divmod64(child_index, u64_of(EXTENT_INTERNAL_FANOUT)).quotient, extent_lower_frames[top].index)) {
            child_ok = false;
        }
        if (child_ok && extent_lower_frames[top].has_previous_child && !lt64(extent_lower_frames[top].previous_child_index, child_index)) {
            child_ok = false;
        }
        w_judge(INVARIANT_I_1_1, child_ok);
        if (!child_ok) {
            w_failures = w_failures + 1u;
            extent_lower_frames[top].child_index = e + 1u;
            continue;
        }
        extent_lower_frames[top].has_previous_child = true;
        extent_lower_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        w_judge_location_order(pending_pointer);
        pending_level = level - 1u;
        pending_index = child_index;
        enter = true;
    }
}

fn extent_upper_cell(level: u32, index: vec2<u32>) -> ExtentCell {
    let span = extent_upper_span(level);
    let first = mul64_64_saturating(index, span);
    return ExtentCell(first, add64_saturating(first, sub64(span, u64_of(1u))));
}

// 上段节点头里的 key 区间是不是位置规定罩的那一段：(首个 inode 号, 偏移 0) 到 (末个 inode 号, 偏移全 1)
fn extent_upper_node_range_is_its_cell(n: Node, cell: ExtentCell) -> bool {
    return w_extent_range_ok(n, cell.first, vec2<u32>(0u, 0u), cell.last, vec2<u32>(NONE, NONE));
}

// 上段一层的条目宽：叶 113、内部节点 110
fn extent_upper_entry_width(level: u32) -> u32 {
    if (level == 0u) {
        return 113u;
    }
    return 110u;
}

// C896：别的版本先走过的上段节点（w_read_index_node 命中）又被这一版引用：按这一条父条目给的位置（根那一个的层级取它自己头里的）照判层级与区间
// （I-1.1），只判、不往下走：不记走读失败、条目宽不重判
fn w_judge_the_position_of_an_extent_upper_node_an_earlier_version_walked(level_given_by_the_parent: u32, index: vec2<u32>) {
    let n = w_index_node_an_earlier_walk_parsed();
    if (!n.ok) {
        return;
    }
    var level = level_given_by_the_parent;
    if (level == NONE) {
        level = n.level;
    }
    _ = w_judge_the_level_and_the_key_range_the_position_prescribes(n, level, extent_upper_node_range_is_its_cell(n, extent_upper_cell(level, index)));
}

// 上段（walk_extent_upper_node）
fn w_walk_extent_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: vec2<u32>) {
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            // 读根（pending_level 还是 NONE）照 walk.rs walk_extent_upper_node 读根那一种读法（key_range_reading 3），读孩子照按位置（2）
            var key_range_reading_of_this_node = 2u;
            if (pending_level == NONE) {
                key_range_reading_of_this_node = 3u;
            }
            let read = w_read_index_node(pending_pointer, tree, 3u, key_range_reading_of_this_node, true, MAPPING_CLASS_INDEX_NODE, fsid_low);
            if (!read.ok) {
                if (read.already_walked) {
                    w_judge_the_position_of_an_extent_upper_node_an_earlier_version_walked(pending_level, pending_index);
                }
                if (depth == 0u) { break; }
                extent_upper_frames[depth - 1u].child_index = extent_upper_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            let n = read.n;
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let cell = extent_upper_cell(level, pending_index);
            let first = cell.first;
            let last = cell.last;
            if (!w_position_and_entry_width_hold(n, level, extent_upper_node_range_is_its_cell(n, cell), extent_upper_entry_width(level))) {
                if (depth == 0u) { break; }
                extent_upper_frames[depth - 1u].child_index = extent_upper_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (level == 0u) {
                let parent_valid = w_parent_valid;
                let parent_device = w_parent_device;
                let parent_slot = w_parent_slot;
                w_parent_valid = true;
                w_parent_device = read.unit_device;
                w_parent_slot = read.unit_slot;
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let entry_inode = extent_key_inode(n.base, o);
                    let tag = rd8(n.base, o + 24u);
                    var well_formed = is_zero64(extent_key_locality(n.base, o)) && is_zero64(extent_key_offset(n.base, o));
                    // 标签 3、4 是判决 m4-trunc-r1 K5 的「之后到 size 是洞」：3 同 1 走进下段，4 同 2 走内联的数据单元；洞与单元数对不对得上 size
                    // 归扫描内核的删除态扫描（I-4.6）
                    let names_a_lower_segment_root = tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT
                        || tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE;
                    let names_an_inline_data_unit = tag == EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT
                        || tag == EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE;
                    if (tag == EXTENT_UPPER_LEAF_TAG_NO_DATA_UNIT) {
                        well_formed = well_formed && all_zero(n.base, o + 25u, 88u);
                    } else if (names_a_lower_segment_root) {
                        well_formed = well_formed && all_zero(n.base, o + 25u + 86u, 2u);
                    } else if (!names_an_inline_data_unit) {
                        well_formed = false;
                    }
                    w_judge(INVARIANT_I_1_1, well_formed && le64(first, entry_inode) && le64(entry_inode, last));
                    if (well_formed && names_a_lower_segment_root) {
                        let lower_root = parse_node_pointer(n.base, o + 25u);
                        w_judge_location_order(lower_root);
                        w_walk_extent_lower(lower_root, entry_inode, tree, fsid_low);
                        // 下段走完把父单元换回这片叶
                        w_parent_valid = true;
                        w_parent_device = read.unit_device;
                        w_parent_slot = read.unit_slot;
                    } else if (well_formed && names_an_inline_data_unit) {
                        w_walk_extent_data_pointer(n.base, o + 25u, entry_inode, vec2<u32>(0u, 0u), tree, fsid_low);
                    }
                }
                w_parent_valid = parent_valid;
                w_parent_device = parent_device;
                w_parent_slot = parent_slot;
                if (depth == 0u) { break; }
                extent_upper_frames[depth - 1u].child_index = extent_upper_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return;
            }
            extent_upper_frames[depth].n = n;
            extent_upper_frames[depth].level = level;
            extent_upper_frames[depth].index = pending_index;
            extent_upper_frames[depth].child_index = 0u;
            extent_upper_frames[depth].has_previous_child = false;
            extent_upper_frames[depth].parent_valid = w_parent_valid;
            extent_upper_frames[depth].parent_device = w_parent_device;
            extent_upper_frames[depth].parent_slot = w_parent_slot;
            w_parent_valid = true;
            w_parent_device = read.unit_device;
            w_parent_slot = read.unit_slot;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_upper_frames[top].n;
        if (extent_upper_frames[top].child_index >= n.entry_count) {
            w_parent_valid = extent_upper_frames[top].parent_valid;
            w_parent_device = extent_upper_frames[top].parent_device;
            w_parent_slot = extent_upper_frames[top].parent_slot;
            depth = depth - 1u;
            if (depth == 0u) { break; }
            extent_upper_frames[depth - 1u].child_index = extent_upper_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_upper_frames[top].child_index;
        let o = entry_offset(n, e);
        let level = extent_upper_frames[top].level;
        let child_span = extent_upper_span(level - 1u);
        let first_inode = extent_key_inode(n.base, o);
        let qr = divmod64(first_inode, child_span);
        var child_ok = is_zero64(extent_key_locality(n.base, o)) && is_zero64(extent_key_offset(n.base, o)) && is_zero64(qr.remainder);
        let child_index = qr.quotient;
        if (child_ok && !eq64(divmod64(child_index, u64_of(EXTENT_INTERNAL_FANOUT)).quotient, extent_upper_frames[top].index)) {
            child_ok = false;
        }
        if (child_ok && extent_upper_frames[top].has_previous_child && !lt64(extent_upper_frames[top].previous_child_index, child_index)) {
            child_ok = false;
        }
        w_judge(INVARIANT_I_1_1, child_ok);
        if (!child_ok) {
            w_failures = w_failures + 1u;
            extent_upper_frames[top].child_index = e + 1u;
            continue;
        }
        extent_upper_frames[top].has_previous_child = true;
        extent_upper_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        w_judge_location_order(pending_pointer);
        pending_level = level - 1u;
        pending_index = child_index;
        enter = true;
    }
}
