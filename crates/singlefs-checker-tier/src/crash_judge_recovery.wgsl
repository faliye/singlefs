// ───────────────────────── 恢复与 oracle ─────────────────────────

// 走读失败在 F2 判退里是不是字节丢了（recovery::RecoveryFailure::failure_kind_for_the_fall_back）：只有读不出与经映射仍读不出两个失败点是，别的都是内容拒收
fn failure_point_is_bytes_lost(failure: u32) -> bool {
    return failure == FAILURE_POINT_UNIT_UNREADABLE || failure == FAILURE_POINT_MAPPING_STILL_UNREADABLE;
}

struct Recovery {
    outcome: u32,            // 走读结局；择不到根、重放停下也算 FAILED
    has_effective: bool,
    effective_instance: u32,
    effective_txg: vec2<u32>,
    content_matches: bool,
    failure: u32,
};
fn recover(consult: bool, cfg_choice: SysChoice) -> Recovery {
    var out: Recovery;
    out.outcome = OUTCOME_FAILED;
    out.has_effective = false;
    out.effective_instance = 0u;
    out.effective_txg = vec2<u32>(0u, 0u);
    out.content_matches = false;
    out.failure = FAILURE_POINT_NONE;
    if (!cfg_choice.ok) {
        out.failure = cfg_choice.failure;
        return out;
    }
    let chosen = choose_root(cfg_choice.cfg);
    if (!chosen.found) {
        out.failure = FAILURE_POINT_NO_VALID_ROOT;
        return out;
    }
    var effective = chosen.root;
    if (consult) {
        // 看 journal 那一遍：先扫环（recovery::scan_journal），再从所选根接着重放（此前漏了这一句，重放的记录表恒空，
        // 两遍恢复落到的是同一条根；红位上看不出来，对拍比落到的根与结局才抓到）
        scan_journal(cfg_choice.cfg);
        let replayed = replay_journal(chosen.root, cfg_choice.cfg.ring_bytes);
        if (!replayed.ok) {
            out.failure = replayed.failure;
            return out;
        }
        effective = replayed.root;
    }
    out.has_effective = true;
    out.effective_instance = effective.instance;
    out.effective_txg = effective.txg;
    // 判退（recovery::recover 调 fall_back_judged_on_the_three_readers）：所选那一版先走一遍；看 journal 的那一遍再判退不退，退就走退到的那一版，
    // 一个调用点走两遍，内核展开只多一份判法、不多一份走读。退到的那一版走得通就落在它，走不通留在所选的那一版、照它自己的走读报
    var root_to_walk = effective;
    for (var walk_round = 0u; walk_round < 2u; walk_round = walk_round + 1u) {
        let walked = walk_to_file(root_to_walk, cfg_choice.cfg, expected_version_index(root_to_walk.instance, root_to_walk.txg));
        // 第一遍照报；退回的那一版走不通时照报第一遍的失败（落到的根也不换）
        if (walk_round == 0u || walked.outcome != OUTCOME_FAILED) {
            out.effective_instance = root_to_walk.instance;
            out.effective_txg = root_to_walk.txg;
            out.outcome = walked.outcome;
            out.failure = walked.failure;
            out.content_matches = walked.content_matches;
        }
        if (walk_round == 1u || !consult) {
            break;
        }
        // ① 系统配置没见证所选那一版、交得出一条可退的根（施加过记录的那一版不退，函数里第一句判）；不退就不必读三样读者
        let fallback = newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg_choice.cfg, chosen.root, effective);
        if (!fallback.found) {
            break;
        }
        // ② 三样读者的并集（C755，用户 2026-10-20 定「三句都补」；C866 改法甲：并集里有内容拒收不退）。冷走读那一样就是这一轮的这一遍。
        // ③ 触发之后重扫一遍环再问 F2 判法：GPU 判的崩溃态是一张定了的镜像，两次扫环读回同一份记录，① 那一问的结局就是 ③ 的，不另扫
        let union_of_the_three_readers = selected_version_under_the_three_readers(effective, cfg_choice.cfg, walked);
        if (union_of_the_three_readers == READER_UNDECIDABLE) {
            undecidable = UNDECIDABLE_A_READER_OF_THE_FALL_BACK_UNION;
            break;
        }
        if (union_of_the_three_readers != READER_LOST_BYTES) {
            break;
        }
        root_to_walk = fallback.root;
    }
    return out;
}

// ───────────────────────── 判退看三样读者的并集 ─────────────────────────

// 一样读者在所选那一版上读一遍的结局（recovery::FailureKindForTheFallBack 两种，加读通，加 GPU 判不了）
const READER_GETS_THROUGH: u32 = 0u;
const READER_LOST_BYTES: u32 = 1u;
const READER_REFUSED_CONTENT: u32 = 2u;
const READER_UNDECIDABLE: u32 = 3u;

// 读者停在一个记了失败点码（walk_failure_point 那一张）的失败上：那一步是字节丢了还是内容拒收（RecoveryFailure::failure_kind_for_the_fall_back）
fn reader_kind_of_the_failure_point(failure: u32) -> u32 {
    if (failure_point_is_bytes_lost(failure)) {
        return READER_LOST_BYTES;
    }
    return READER_REFUSED_CONTENT;
}

// 一条节点指针按 I-2.5 判（NodePointer::location_entries_ascend_by_device）：全零的空指针豁免，别的两条位置条目按设备身份严格升序
fn node_pointer_ascends_by_device(pointer: Ptr) -> bool {
    return pointer.all_zero || pointer.device0 < pointer.device1;
}

// recovery::selected_version_under_the_three_readers：所选那一版 selected 在三样读者——恢复的冷走读、只读挂载的打开、可写挂载的建上一版连同读它的
// 实例表——下各读一遍，每样停在它第一个失败上：有一样拒收内容交内容拒收（不退），否则有一样字节丢了交字节丢了（F2 判退的那一个触发），都读通交读通。
// 冷走读那一样就是恢复第一轮走的那一遍 walk_of_the_selected_version。一样读者 GPU 判不了（表越过内核容量）时，别的读者有内容拒收照样不退，
// 否则交判不了（整个状态记判不了，同走读判不了）。各样读者自己的判不了只算进它那一样，不漏进别的状态位
fn selected_version_under_the_three_readers(selected: RootFields, cfg: SysConfig, walk_of_the_selected_version: WalkResult) -> u32 {
    var kinds: array<u32, 4>;
    kinds[0] = READER_GETS_THROUGH;
    if (walk_of_the_selected_version.outcome == OUTCOME_FAILED) {
        kinds[0] = reader_kind_of_the_failure_point(walk_of_the_selected_version.failure);
    }
    let undecidable_before_the_readers = undecidable;
    // 打开与建上一版两样读者读同一版的同一棵中央映射树：打开那一样整棵先读（读不出就停在那里），建上一版到第一次要经映射回退、或走到映射树那一步时才读，
    // 两处读的是同一份字节、同一个结局。这里先读一遍，读不出的那一处记在 mapping_tree_failure_point，两样读者照各自的时点报。
    // 映射树每个节点的指针是建上一版最后按 I-2.5 判的那几条之一：读的时候开着判指针那一记，读完存下来交给建上一版那一样
    undecidable = 0u;
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    rebuild_judges_the_pointers_of_the_version = true;
    some_pointer_of_the_version_does_not_ascend_by_device = false;
    mapping_tree_failure_point = FAILURE_POINT_NONE;
    let mapping = read_code_two_tree(selected.mapping_root, 0u, 0u, selected.mapping_root.birth_tree, 1u, selected, fsid_low);
    if (!mapping.ok) {
        mapping_tree_failure_point = walk_failure_point;
    }
    rebuild_judges_the_pointers_of_the_version = false;
    let mapping_tree_pointers_ascend_by_device = !some_pointer_of_the_version_does_not_ascend_by_device;
    let mapping_tree_read_is_undecidable = undecidable != 0u;
    for (var reader = 1u; reader < 4u; reader = reader + 1u) {
        undecidable = 0u;
        var kind = READER_UNDECIDABLE;
        if (reader == 1u) {
            if (!mapping_tree_read_is_undecidable) {
                kind = open_reader_kind(selected, cfg);
            }
        } else if (reader == 2u) {
            if (!mapping_tree_read_is_undecidable) {
                kind = rebuild_reader_kind(selected, cfg, mapping_tree_pointers_ascend_by_device);
            }
        } else {
            // recovery::instance_table_chain_of_root：沿链读整张实例表，一片读不出 UnitUnreadable、解不开 UnitMalformed
            kind = READER_GETS_THROUGH;
            if (!instance_table_chain_readable(selected)) {
                kind = reader_kind_of_the_failure_point(walk_failure_point);
            }
        }
        if (undecidable != 0u) {
            kind = READER_UNDECIDABLE;
        }
        kinds[reader] = kind;
    }
    undecidable = undecidable_before_the_readers;
    var some_reader_refused_content = false;
    var some_reader_lost_bytes = false;
    var some_reader_is_undecidable = false;
    for (var reader = 0u; reader < 4u; reader = reader + 1u) {
        some_reader_refused_content = some_reader_refused_content || kinds[reader] == READER_REFUSED_CONTENT;
        some_reader_lost_bytes = some_reader_lost_bytes || kinds[reader] == READER_LOST_BYTES;
        some_reader_is_undecidable = some_reader_is_undecidable || kinds[reader] == READER_UNDECIDABLE;
    }
    if (some_reader_refused_content) {
        return READER_REFUSED_CONTENT;
    }
    if (some_reader_is_undecidable) {
        return READER_UNDECIDABLE;
    }
    if (some_reader_lost_bytes) {
        return READER_LOST_BYTES;
    }
    return READER_GETS_THROUGH;
}

// 树表里种类是 kind 的那一条条目在表里的偏移（条目逐条判过、每种至多一条之后调）；没有交 NONE
fn tree_table_entry_offset_of_kind(tree_table: Node, kind: u32) -> u32 {
    for (var entry_index = 0u; entry_index < tree_table.entry_count; entry_index = entry_index + 1u) {
        let entry_at = entry_offset(tree_table, entry_index);
        if (rd16(tree_table.base, entry_at + 10u) == kind) {
            return entry_at;
        }
    }
    return NONE;
}

// ───────────────────────── 只读挂载的打开（判退的第二样读者）─────────────────────────

// mounted_read::open_pool_for_read_judging_the_truncation_fields_as 在 root 上读一遍，停在第一个失败上交它的种类
// （OpenPoolForReadFailure::failure_kind_for_the_fall_back：沿根读单元那一步的失败照 RecoveryFailure 那一张，别的都是内容拒收；树表 0 条算读通）。
// 中央映射树调用方已读过（selected_version_under_the_three_readers），读不出的那一处在 mapping_tree_failure_point；打开在读映射树那一步就报它。
// 读序：映射根是空指针时先读树表、0 条就读通（C695 甲）；整棵读映射树、逐条解映射条目；读树表；条目逐条解、每种至多一条；取 extent 与 inode 两条有根的条目；
// 读 inode 树根（提示读不出经映射回退）、核自描述与层级 1；逐条内部条目解开、读叶容器（码 3）、核叶头与条目身份、记录宽 140、fsid，逐条解记录、判截断字段
fn open_reader_kind(root: RootFields, cfg: SysConfig) -> u32 {
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    // 树表在映射根是空指针时先读一次、读完映射树再读一次：同一个单元同一份字节，一个调用点在两轮里读
    var tree_table = bad_node();
    for (var tree_table_read = 0u; tree_table_read < 2u; tree_table_read = tree_table_read + 1u) {
        if (tree_table_read == 0u && !root.mapping_root.all_zero) {
            continue;
        }
        if (tree_table_read == 1u) {
            if (mapping_tree_failure_point != FAILURE_POINT_NONE) {
                return reader_kind_of_the_failure_point(mapping_tree_failure_point);
            }
            // 映射条目逐条解（records::parse_mapping_entry 窄于 55 交 None）：RecordMalformed
            for (var leaf_index = 0u; leaf_index < mapping_leaf_count; leaf_index = leaf_index + 1u) {
                if (mapping_leaf_at(leaf_index).entries.entry_width < 55u) {
                    return READER_REFUSED_CONTENT;
                }
            }
        }
        let tree_table_slice = read_unit_via_locations(root.tree_table, NODE_BYTES);
        if (tree_table_slice.version == NONE) {
            return READER_LOST_BYTES;
        }
        tree_table = parse_index_node(tree_table_slice);
        if (!tree_table.ok) {
            return READER_REFUSED_CONTENT;
        }
        if (tree_table.entry_count == 0u) {
            return READER_GETS_THROUGH;
        }
    }
    if (!tree_table_entries_each_kind_at_most_once(tree_table)) {
        return READER_REFUSED_CONTENT;
    }
    // entry_of_kind：这个种类的条目、根指针不是全零（还没根的算没有这棵树，TreeTableHasNoTreeOfKind）
    let extent_entry = tree_table_entry_offset_of_kind(tree_table, TREE_KIND_EXTENT);
    if (extent_entry == NONE || all_zero(tree_table.base, extent_entry + 14u, 86u)) {
        return READER_REFUSED_CONTENT;
    }
    let inode_entry = tree_table_entry_offset_of_kind(tree_table, TREE_KIND_INODE);
    if (inode_entry == NONE || all_zero(tree_table.base, inode_entry + 14u, 86u)) {
        return READER_REFUSED_CONTENT;
    }
    // recovery::read_mapped_tree_root：读树根（提示读不出经映射回退）、核自描述（树 ID、key 宽 8、出生身份、fsid、key 区间）
    let inode_root_pointer = parse_node_pointer(tree_table.base, inode_entry + 14u);
    let inode_root_read = read_via_hint_then_mapping(inode_root_pointer, tree_table.base, inode_entry + 14u, 2u, NODE_BYTES);
    if (inode_root_read.failed) {
        return reader_kind_of_the_failure_point(walk_failure_point);
    }
    let inode_root = parse_index_node(inode_root_read.s);
    if (!tree_root_checked(inode_root, rd64(tree_table.base, inode_entry), 8u, inode_root_pointer, root, fsid_low)) {
        return READER_REFUSED_CONTENT;
    }
    if (inode_root.level != 1u) {
        return READER_REFUSED_CONTENT;
    }
    var inode_numbers_read_back = InodeNumbersReadBack(false, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u));
    for (var entry_index = 0u; entry_index < inode_root.entry_count; entry_index = entry_index + 1u) {
        // records::parse_inode_internal_entry_or_refusal：条目宽不等于 120、子指针头部有一段不是第一版的取值（RecordMalformed、PointerInTheRecordCannotBeDereferenced）
        if (inode_root.entry_width != CONTAINER_TREE_ENTRY_BYTES) {
            return READER_REFUSED_CONTENT;
        }
        let entry_at = entry_offset(inode_root, entry_index);
        let child = parse_node_pointer(inode_root.base, entry_at + 34u);
        if (!child.head_ok || !child.extent_offset_zero) {
            return READER_REFUSED_CONTENT;
        }
        let leaf_read = read_via_hint_then_mapping(child, inode_root.base, entry_at + 34u, 3u, DATA_UNIT_BYTES);
        if (leaf_read.failed) {
            return reader_kind_of_the_failure_point(walk_failure_point);
        }
        let leaf = parse_packed_unit(leaf_read.s);
        if (!leaf.ok) {
            return READER_REFUSED_CONTENT;
        }
        // 叶头的四元组身份等于条目里的身份引用、类型段 2、记录宽 140、fsid 是本池（InodeLeafContainerDisagreesWithTheInternalEntry）
        let identity_matches_the_entry = eq64(leaf.birth_tree, rd64(inode_root.base, entry_at + 8u)) && leaf.record_type == rd16(inode_root.base, entry_at + 16u)
            && eq64(leaf.container, rd64(inode_root.base, entry_at + 18u)) && eq64(leaf.container_birth, rd64(inode_root.base, entry_at + 26u));
        if (!identity_matches_the_entry || rd16(inode_root.base, entry_at + 16u) != 2u || leaf.record_width != 140u || !eq64(leaf.fsid, fsid_low)) {
            return READER_REFUSED_CONTENT;
        }
        // InodeRecord::parse（InodeRecordMalformed）与截断字段在这个池里收不收（InodeRecordCarriesATruncationFieldInAPoolWithoutIncompatBitTwo）
        for (var record_index = 0u; record_index < leaf.record_count; record_index = record_index + 1u) {
            let record_offset = packed_record_offset(leaf, record_index);
            if (!inode_record_parses_by_the_core_reader(leaf.base, record_offset)
                || !inode_record_truncation_field_fits_the_pool(leaf.base, record_offset, cfg.carries_the_inode_record_truncation_field)) {
                return READER_REFUSED_CONTENT;
            }
            inode_numbers_read_back = inode_numbers_read_back_with(inode_numbers_read_back, rd64(leaf.base, record_offset));
        }
    }
    // mounted_read：inode 树根头里的 key 区间是子树覆盖区间，读完叶容器判（read_mapped_tree_root 对层级 ≥ 1 的根不判区间）
    if (!inode_tree_root_key_range_is_the_subtree_coverage(inode_root, inode_numbers_read_back)) {
        return READER_REFUSED_CONTENT;
    }
    return READER_GETS_THROUGH;
}

// ───────────────────────── 可写挂载的建上一版连同实例表（判退的第三样读者）─────────────────────────

// mount::record_standing_for_the_version：顶着 version 的那条记录——同实例、同 checkpoint_txg、带末条标志的记录里计数器最大的；一条都没有就拿环里
// 计数器最大的那条（同计数器几条取记录表次序上最后那条，同 max_by_key）。环里一条记录都没有交 NONE
fn record_standing_for_the_version(version: RootFields) -> u32 {
    var standing = NONE;
    var standing_counter = vec2<u32>(0u, 0u);
    var newest = NONE;
    var newest_counter = vec2<u32>(0u, 0u);
    for (var record_index = 0u; record_index < record_count; record_index = record_index + 1u) {
        let record = record_at(record_index);
        if (record.instance == version.instance && eq64(record.txg, version.txg) && record.last
            && (standing == NONE || !lt64(record.counter, standing_counter))) {
            standing = record_index;
            standing_counter = record.counter;
        }
        if (newest == NONE || !lt64(record.counter, newest_counter)) {
            newest = record_index;
            newest_counter = record.counter;
        }
    }
    if (standing != NONE) {
        return standing;
    }
    return newest;
}
// recovery::location_entries_of_the_record_ascend_by_device（I-2.5）：新根段四条指针（全零的豁免）与每个点名项的两条位置条目（不豁免）按设备身份严格升序。
// 新根段四条指针在记录里的偏移同 parse_record：树表指针在新根段起点、中央映射树根指针接在它之后（185）、实例表与分配记录树根两条照新根段里的偏移
fn record_location_entries_ascend_by_device(record: Rec) -> bool {
    var pointer_offsets = array<u32, 4>(
        JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
        185u,
        JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER,
        JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER);
    for (var pointer = 0u; pointer < 4u; pointer = pointer + 1u) {
        if (!node_pointer_ascends_by_device(parse_node_pointer(record.base, pointer_offsets[pointer]))) {
            return false;
        }
    }
    for (var named = 0u; named < record.named_count; named = named + 1u) {
        let entry_at = JOURNAL_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES;
        if (rd32(record.base, entry_at) >= rd32(record.base, entry_at + 14u)) {
            return false;
        }
    }
    return true;
}
// R_FILES 里 inode 号是 inode 的那一行（行按 inode 号严格升序，二分）；没有交 NONE
fn file_row_of_the_inode(inode: vec2<u32>) -> u32 {
    var low = 0u;
    var high = files_of_the_extent_tree;
    while (low < high) {
        let middle = (low + high) / 2u;
        let row = R_FILES + middle * FILE_ROW_WORDS;
        let row_inode = vec2<u32>(sw(row), sw(row + 1u));
        if (eq64(row_inode, inode)) {
            return middle;
        }
        if (lt64(row_inode, inode)) {
            low = middle + 1u;
        } else {
            high = middle;
        }
    }
    return NONE;
}
// write_request_split::data_units_read_within_the_file_size（recovery::data_unit_count_matches_the_inode_size 调它）：size 按净荷容量算出的单元数
// max(1, ⌈size ÷ 32634⌉)；最后一个单元之后不是洞的，存在的单元不少于它、交它；是洞的，存在的单元少于它、交存在的单元数；不成立交 NONE
fn data_units_within_the_file_size(size: vec2<u32>, units_on_disk: u32, tail_is_a_hole: bool) -> u32 {
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    var units_implied_by_the_size = divmod64(add64_saturating(size, sub64(payload, u64_of(1u))), payload).quotient;
    if (is_zero64(units_implied_by_the_size)) {
        units_implied_by_the_size = u64_of(1u);
    }
    if (tail_is_a_hole) {
        if (lt64(u64_of(units_on_disk), units_implied_by_the_size)) {
            return units_on_disk;
        }
        return NONE;
    }
    if (lt64(u64_of(units_on_disk), units_implied_by_the_size)) {
        return NONE;
    }
    return units_implied_by_the_size.x;
}
// recovery::data_unit_judged_against_its_pointer_and_the_inode_record：读到手的一个数据单元按查找路径与 inode 记录核（五元组、对象出生代、诞生代号、写序、
// fsid、声明长度、补齐字节为 0）。不成立交 false（UnitMalformed 或 InvariantViolated，都是内容拒收）
fn data_unit_judged_against_its_pointer_and_the_inode_record(unit_slice: Slice, pointer: Ptr, position: u32, extent_tree: vec2<u32>, inode: vec2<u32>,
    size: vec2<u32>, object_birth: vec2<u32>, fsid_low: vec2<u32>) -> bool {
    let unit = parse_data_unit(unit_slice);
    if (!unit.ok) {
        return false;
    }
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    let first_file_byte = mul64_64_saturating(u64_of(position), payload);
    if (!eq64(unit.tree, extent_tree) || !eq64(unit.object, inode) || !eq64(unit.anchor_offset, first_file_byte)) {
        return false;
    }
    if (!eq64(unit.object_birth, object_birth)) {
        return false;
    }
    if (!eq64(unit.birth_txg, pointer.birth_txg) || unit.instance != pointer.instance || !eq64(unit.transaction, pointer.tail) || !eq64(unit.fsid, fsid_low)) {
        return false;
    }
    var expected_length = sub64_saturating(size, first_file_byte);
    if (lt64(payload, expected_length)) {
        expected_length = payload;
    }
    return eq64(u64_of(unit.declared_length), expected_length) && unit.padding_zero;
}
// 建上一版读每个文件的每个数据单元（recovery::rebuild_version 读完 extent 树之后那两圈）：R_FILES 里每个文件按单元号读，提示读不出经映射回退；
// 映射里没有它、映射落点也读不出的照抄位置项、不读内容，不报错；查映射本身判红（映射树读不出、映射条目窄于字段表）就停在这里交它的种类。
// judges 为真时（inode 树那一遍之后的第二遍）再核：第一个文件那条记录没有而 extent 树里有它的单元、别的文件没有记录、单元数对不上 size
// （每个文件都按标签认尾部的洞），读到手的 size 之内的单元按查找路径与它那条记录核（recovery::rebuild_version 判第一个文件与
// 别的文件那两圈），不成立交内容拒收。数据指针按块读，一块用完从下段根往下找含下一个单元的那片叶（同走读的 find_the_next_block_of_data_pointers）；
// 读下段节点与读数据单元同一个调用点
fn rebuild_reader_data_units_kind(extent_tree: vec2<u32>, judges: bool, first_file_record_found: bool, fsid_low: vec2<u32>) -> u32 {
    if (judges && !first_file_record_found && file_row_of_the_inode(u64_of(FIRST_INODE_NUMBER)) != NONE
        && sw(R_FILES + file_row_of_the_inode(u64_of(FIRST_INODE_NUMBER)) * FILE_ROW_WORDS + 5u) > 0u) {
        return READER_REFUSED_CONTENT;
    }
    for (var file = 0u; file < files_of_the_extent_tree; file = file + 1u) {
        let row = R_FILES + file * FILE_ROW_WORDS;
        let inode = vec2<u32>(sw(row), sw(row + 1u));
        let upper_entry_base = sw(row + 2u);
        let upper_entry_offset = sw(row + 3u);
        let tag = sw(row + 4u);
        let units = sw(row + 5u);
        let is_the_first_file = eq64(inode, u64_of(FIRST_INODE_NUMBER));
        let record_found = sw(row + 6u) != 0u;
        let size = vec2<u32>(sw(row + 7u), sw(row + 8u));
        let object_birth = vec2<u32>(sw(row + 9u), sw(row + 10u));
        var units_within_the_size = units;
        if (judges) {
            if (!record_found) {
                // 第一个文件没有记录、有单元那一形上面判过；别的文件要有记录
                if (!is_the_first_file) {
                    return READER_REFUSED_CONTENT;
                }
                continue;
            }
            // 尾部照标签声明的判，第一个文件与别的文件同一判（判决 m4-multi-inode-size-r1 第二节 S2，recovery.rs 那一圈的 file.tail）
            let tail_is_a_hole = tag == 0u || tag == 3u || tag == 4u;
            units_within_the_size = data_units_within_the_file_size(size, units, tail_is_a_hole);
            if (units_within_the_size == NONE) {
                return READER_REFUSED_CONTENT;
            }
        }
        // 标签 2、4 内联的那一个单元是这一块唯一的一条；1、3 从下段根往下找头一块
        let names_an_inline_data_unit = tag == 2u || tag == 4u;
        data_block_first = 0u;
        data_block_count = 0u;
        if (names_an_inline_data_unit) {
            sws(R_DATA, upper_entry_base);
            sws(R_DATA + 1u, upper_entry_offset + 25u);
            data_block_count = 1u;
        }
        var next_block_pointer_base = NONE;
        var next_block_pointer_offset = 0u;
        var position = 0u;
        // 迭代上界：每个单元读一次，加上每找一块往下读的层数（extent 树那一遍核过的下段，层数不过 MAX_FRAMES）。读第一遍的全部单元；判的那一遍只读 size 之内的
        loop {
            if (position >= units || (judges && position >= units_within_the_size)) {
                break;
            }
            let finding_the_next_block = position >= data_block_first + data_block_count;
            if (finding_the_next_block && next_block_pointer_base == NONE) {
                next_block_pointer_base = upper_entry_base;
                next_block_pointer_offset = upper_entry_offset + 25u;
            }
            var base = next_block_pointer_base;
            var pointer_offset = next_block_pointer_offset;
            var pointer: Ptr;
            var unit_class = 2u;
            var unit_bytes = NODE_BYTES;
            if (finding_the_next_block) {
                pointer = parse_node_pointer(base, pointer_offset);
            } else {
                base = sw(R_DATA + (position - data_block_first) * 2u);
                pointer_offset = sw(R_DATA + (position - data_block_first) * 2u + 1u);
                pointer = parse_data_pointer(base, pointer_offset);
                unit_class = 1u;
                unit_bytes = DATA_UNIT_BYTES;
            }
            let unit_read = read_via_hint_then_mapping(pointer, base, pointer_offset, unit_class, unit_bytes);
            if (finding_the_next_block) {
                let step = find_the_next_block_of_data_pointers(unit_read, position);
                if (undecidable != 0u) {
                    return READER_UNDECIDABLE;
                }
                next_block_pointer_base = step.pointer_base;
                next_block_pointer_offset = step.pointer_offset;
                if (step.is_the_block_found) {
                    next_block_pointer_base = NONE;
                }
                continue;
            }
            if (unit_read.failed) {
                // DataUnitReadThroughTheCentralMapping 的两种读不出照抄、不报；查映射本身判红的才报
                if (walk_failure_point != FAILURE_POINT_MAPPING_MISS && walk_failure_point != FAILURE_POINT_MAPPING_STILL_UNREADABLE) {
                    return reader_kind_of_the_failure_point(walk_failure_point);
                }
            } else if (judges && !data_unit_judged_against_its_pointer_and_the_inode_record(unit_read.s, pointer, position, extent_tree, inode, size, object_birth, fsid_low)) {
                return READER_REFUSED_CONTENT;
            }
            position = position + 1u;
        }
    }
    return READER_GETS_THROUGH;
}
// recovery::rebuild_version_with_its_snapshot_entries_judging_the_truncation_fields_as 在 root 上建一遍（只读、不写盘），停在第一个失败上交它的种类
// （RebuildVersionFailure::failure_kind_for_the_fall_back：走读同款的失败照 RecoveryFailure 那一张；树表有条目而环里一条记录都没有算字节丢了）。
// mapping_tree_pointers_ascend_by_device：调用方读中央映射树时判的那一记（这一版每条指针按 I-2.5 判，映射树的节点指针也在里面）。
// 判这一版每条指针那一记在读这一版的各棵树时开着，判完关上（rebuild_reader_kind_with_the_pointers_judged）
fn rebuild_reader_kind(root: RootFields, cfg: SysConfig, mapping_tree_pointers_ascend_by_device: bool) -> u32 {
    let kind = rebuild_reader_kind_with_the_pointers_judged(root, cfg, mapping_tree_pointers_ascend_by_device);
    rebuild_judges_the_pointers_of_the_version = false;
    extent_tree_records_its_files = false;
    return kind;
}
fn rebuild_reader_kind_with_the_pointers_judged(root: RootFields, cfg: SysConfig, mapping_tree_pointers_ascend_by_device: bool) -> u32 {
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    // root_record_pointers_ascend_by_device：根记录四条指针（I-2.5，全零的豁免）
    if (!node_pointer_ascends_by_device(root.tree_table) || !node_pointer_ascends_by_device(root.instance_table)
        || !node_pointer_ascends_by_device(root.mapping_root) || !node_pointer_ascends_by_device(root.allocation_root)) {
        return READER_REFUSED_CONTENT;
    }
    // 实例表第 0 片按位置条目读（读得出就照抄，不解）；树表读、解、key 宽 8
    if (read_unit_via_locations(root.instance_table, DATA_UNIT_BYTES).version == NONE) {
        return READER_LOST_BYTES;
    }
    let tree_table_slice = read_unit_via_locations(root.tree_table, NODE_BYTES);
    if (tree_table_slice.version == NONE) {
        return READER_LOST_BYTES;
    }
    let tree_table = parse_index_node(tree_table_slice);
    if (!tree_table.ok || tree_table.key_width != 8u) {
        return READER_REFUSED_CONTENT;
    }
    // 树表 0 条的一版建得出（RebuiltVersion::WithoutFile）
    if (tree_table.entry_count == 0u) {
        return READER_GETS_THROUGH;
    }
    // 顶着这一版的那条记录：环里一条都读不出是 NoRecordStandingForFileVersion（字节丢了）
    let record_index = record_standing_for_the_version(root);
    if (record_index == NONE) {
        return READER_LOST_BYTES;
    }
    if (!record_location_entries_ascend_by_device(record_at(record_index))) {
        return READER_REFUSED_CONTENT;
    }
    if (!tree_table_entries_each_kind_at_most_once(tree_table) || !tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(tree_table)) {
        return READER_REFUSED_CONTENT;
    }
    // tree_identifier_of_the_entry_is_below_the_watermark 逐条（I-7.8）
    for (var entry_index = 0u; entry_index < tree_table.entry_count; entry_index = entry_index + 1u) {
        if (!lt64(rd64(tree_table.base, entry_offset(tree_table, entry_index)), root.watermark)) {
            return READER_REFUSED_CONTENT;
        }
    }
    // pointer_of / tree_of：十二种树每种都要有条目（根指针全零也算有），缺一种 UnitMalformed
    for (var issuing_position = 0u; issuing_position < TREE_KINDS_OF_THE_TREE_TABLE; issuing_position = issuing_position + 1u) {
        if (tree_table_entry_offset_of_kind(tree_table, tree_kind_in_the_issuing_order(issuing_position)) == NONE) {
            return READER_REFUSED_CONTENT;
        }
    }
    // 从这里起读到的这一版的每条指针按 I-2.5 记下（parse_node_pointer / parse_data_pointer、read_via_hint_then_mapping 里那两处），最后判。
    // 树表条目的根指针（码 14、15 与 livelist、稀疏侧表那几条读者不往下读的也在里面）先逐条记
    rebuild_judges_the_pointers_of_the_version = true;
    some_pointer_of_the_version_does_not_ascend_by_device = false;
    some_mapping_position_of_the_version_does_not_ascend_by_device = false;
    for (var entry_index = 0u; entry_index < tree_table.entry_count; entry_index = entry_index + 1u) {
        let root_pointer = parse_node_pointer(tree_table.base, entry_offset(tree_table, entry_index) + 14u);
        // 重建之后树表条目的根指针也换成映射位置（码 2 key）再判：读者不往下读的那几棵树，映射里有它们的根也在这里查
        if (!root_pointer.all_zero && mapping_tree_failure_point == FAILURE_POINT_NONE) {
            let position = mapping_lookup(tree_table.base, entry_offset(tree_table, entry_index) + 14u, 2u, root_pointer.tail, false);
            if (!position.failed && position.found && position.p.device0 >= position.p.device1) {
                some_mapping_position_of_the_version_does_not_ascend_by_device = true;
            }
        }
    }
    // extent 树整棵读回来、extents_of_the_first_file（同走读那一遍），每个有上段叶条目的文件记进 R_FILES
    let extent_entry = tree_table_entry_offset_of_kind(tree_table, TREE_KIND_EXTENT);
    let extent_tree = rd64(tree_table.base, extent_entry);
    extent_tree_records_its_files = true;
    let extent = read_extent_tree_of_the_first_file(parse_node_pointer(tree_table.base, extent_entry + 14u), tree_table.base, extent_entry + 14u,
        extent_tree, root, fsid_low);
    extent_tree_records_its_files = false;
    if (undecidable != 0u) {
        return READER_UNDECIDABLE;
    }
    if (!extent.ok) {
        return reader_kind_of_the_failure_point(walk_failure_point);
    }
    let extent_nodes = extent.node_count;
    let data_units_of_every_file = data_count + data_units_of_the_files_other_than_the_first;
    // 数据单元两遍、中间夹 inode 树（recovery::rebuild_version 的次序：读每个文件的每个单元，读 inode 树根与叶容器，再核第一个文件与别的文件的单元）：
    // 两遍是同一个调用点
    let inode_entry = tree_table_entry_offset_of_kind(tree_table, TREE_KIND_INODE);
    let inode_tree = rd64(tree_table.base, inode_entry);
    var inode_leaf_containers = 0u;
    var first_file_record_found = false;
    // 有记录、上段里没有它的条目而 size 不是 0 的第一个文件之外的号（recovery::data_units_judged_against_the_inode_records 最后那一判，
    // 判决 m4-multi-inode-size-r1 第二节 S2：RECORD_WITHOUT_AN_EXTENT_ENTRY_HAS_A_SIZE）：读记录那一遍记下，单元核完之后拒，次序同 CPU
    var a_record_without_an_entry_has_a_size = false;
    for (var data_pass = 0u; data_pass < 2u; data_pass = data_pass + 1u) {
        if (data_pass == 1u) {
            // inode 树根照冷走读核自描述与层级 1（零条目的根不拒）；逐条内部条目解开、判（宽 120、子指针头部、类型段 2），读叶容器（码 3）、
            // 核叶头与条目（inode_leaf_container_judged_against_its_entry）、逐条解记录、判记录号与截断字段，叶容器没有记录 UnitMalformed。
            // 第一个文件那条记录与 R_FILES 里各文件的记录取叶序上第一条（同 find）
            let inode_root_pointer = parse_node_pointer(tree_table.base, inode_entry + 14u);
            let inode_root_read = read_via_hint_then_mapping(inode_root_pointer, tree_table.base, inode_entry + 14u, 2u, NODE_BYTES);
            if (inode_root_read.failed) {
                return reader_kind_of_the_failure_point(walk_failure_point);
            }
            let inode_root = parse_index_node(inode_root_read.s);
            if (!tree_root_checked(inode_root, inode_tree, 8u, inode_root_pointer, root, fsid_low) || inode_root.level != 1u) {
                return READER_REFUSED_CONTENT;
            }
            inode_leaf_containers = inode_root.entry_count;
            var inode_numbers_read_back = InodeNumbersReadBack(false, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u));
            for (var entry_index = 0u; entry_index < inode_root.entry_count; entry_index = entry_index + 1u) {
                if (inode_root.entry_width != CONTAINER_TREE_ENTRY_BYTES) {
                    return READER_REFUSED_CONTENT;
                }
                let entry_at = entry_offset(inode_root, entry_index);
                let separator = rd64(inode_root.base, entry_at);
                let identity_birth_tree = rd64(inode_root.base, entry_at + 8u);
                let identity_type = rd16(inode_root.base, entry_at + 16u);
                let identity_container = rd64(inode_root.base, entry_at + 18u);
                let identity_birth = rd64(inode_root.base, entry_at + 26u);
                let child = parse_node_pointer(inode_root.base, entry_at + 34u);
                if (!child.head_ok || !child.extent_offset_zero || identity_type != 2u) {
                    return READER_REFUSED_CONTENT;
                }
                let leaf_read = read_via_hint_then_mapping(child, inode_root.base, entry_at + 34u, 3u, DATA_UNIT_BYTES);
                if (leaf_read.failed) {
                    return reader_kind_of_the_failure_point(walk_failure_point);
                }
                let leaf = parse_packed_unit(leaf_read.s);
                if (!leaf.ok) {
                    return READER_REFUSED_CONTENT;
                }
                if (!eq64(leaf.birth_tree, identity_birth_tree) || leaf.record_type != identity_type || !eq64(leaf.container, identity_container)
                    || !eq64(leaf.container_birth, identity_birth) || leaf.record_width != 140u || !eq64(leaf.fsid, fsid_low)) {
                    return READER_REFUSED_CONTENT;
                }
                if (!eq64(child.birth_tree, identity_birth_tree) || leaf.birth_sequence != child.tail.x) {
                    return READER_REFUSED_CONTENT;
                }
                if (lt64(root.txg, leaf.birth_txg) || leaf.instance != child.instance) {
                    return READER_REFUSED_CONTENT;
                }
                for (var record_index = 0u; record_index < leaf.record_count; record_index = record_index + 1u) {
                    let record_offset = packed_record_offset(leaf, record_index);
                    if (!inode_record_parses_by_the_core_reader(leaf.base, record_offset)) {
                        return READER_REFUSED_CONTENT;
                    }
                    let inode = rd64(leaf.base, record_offset);
                    if (lt64(inode, separator) || lt64(inode, identity_container)
                        || !inode_record_truncation_field_fits_the_pool(leaf.base, record_offset, cfg.carries_the_inode_record_truncation_field)) {
                        return READER_REFUSED_CONTENT;
                    }
                    inode_numbers_read_back = inode_numbers_read_back_with(inode_numbers_read_back, inode);
                    if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {
                        first_file_record_found = true;
                    }
                    let file_row = file_row_of_the_inode(inode);
                    if (file_row == NONE && !eq64(inode, u64_of(FIRST_INODE_NUMBER))
                        && !is_zero64(rd64(leaf.base, record_offset + INODE_RECORD_SIZE_OFFSET))) {
                        a_record_without_an_entry_has_a_size = true;
                    }
                    if (file_row != NONE && sw(R_FILES + file_row * FILE_ROW_WORDS + 6u) == 0u) {
                        let row = R_FILES + file_row * FILE_ROW_WORDS;
                        let size = rd64(leaf.base, record_offset + INODE_RECORD_SIZE_OFFSET);
                        let object_birth = rd64(leaf.base, record_offset + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                        sws(row + 6u, 1u);
                        sws(row + 7u, size.x);
                        sws(row + 8u, size.y);
                        sws(row + 9u, object_birth.x);
                        sws(row + 10u, object_birth.y);
                    }
                }
                if (leaf.record_count == 0u) {
                    return READER_REFUSED_CONTENT;
                }
            }
            // recovery::inode_tree_read_and_judged：读完叶容器判 inode 树根头里的 key 区间是子树覆盖区间
            if (!inode_tree_root_key_range_is_the_subtree_coverage(inode_root, inode_numbers_read_back)) {
                return READER_REFUSED_CONTENT;
            }
            // 第一个文件那条记录在而 extent 树里没有它那一条上段叶条目：一个单元都没有，尾部没有洞，size 算出至少一个单元，单元数对不上（UnitMalformed 之前那一判）
            if (first_file_record_found && file_row_of_the_inode(u64_of(FIRST_INODE_NUMBER)) == NONE) {
                return READER_REFUSED_CONTENT;
            }
        }
        let data_units = rebuild_reader_data_units_kind(extent_tree, data_pass == 1u, first_file_record_found, fsid_low);
        if (data_units != READER_GETS_THROUGH) {
            return data_units;
        }
        if (data_pass == 1u && a_record_without_an_entry_has_a_size) {
            return READER_REFUSED_CONTENT;
        }
    }
    // 删除那两棵树的根（deleted_inodes 码 9、logged_ops 码 12）：根指针全零就是零根；否则读根（提示读不出经映射回退）、核自描述，
    // deleted_inodes 要层级 0、非空、条目宽 8，logged_ops 要层级 0、非空、条目逐条照字段表解（logged_operation_entries_of_the_root_entries）
    var deletion_tree_nodes = 0u;
    for (var deletion_tree = 0u; deletion_tree < 2u; deletion_tree = deletion_tree + 1u) {
        let kind = select(TREE_KIND_DELETED_INODES, TREE_KIND_LOGGED_OPERATIONS, deletion_tree == 1u);
        let entry = tree_table_entry_offset_of_kind(tree_table, kind);
        let deletion_root_pointer = parse_node_pointer(tree_table.base, entry + 14u);
        if (deletion_root_pointer.all_zero) {
            continue;
        }
        let deletion_root_read = read_via_hint_then_mapping(deletion_root_pointer, tree_table.base, entry + 14u, 2u, NODE_BYTES);
        if (deletion_root_read.failed) {
            return reader_kind_of_the_failure_point(walk_failure_point);
        }
        let deletion_root = parse_index_node(deletion_root_read.s);
        if (!tree_root_checked(deletion_root, rd64(tree_table.base, entry), tree_kind_key_width(kind), deletion_root_pointer, root, fsid_low)
            || deletion_root.level != 0u || deletion_root.entry_count == 0u) {
            return READER_REFUSED_CONTENT;
        }
        deletion_tree_nodes = deletion_tree_nodes + 1u;
        if (kind == TREE_KIND_DELETED_INODES) {
            if (deletion_root.entry_width != DELETED_INODES_KEY_WIDTH) {
                return READER_REFUSED_CONTENT;
            }
            continue;
        }
        if (!logged_operation_entries_parse(deletion_root)) {
            return READER_REFUSED_CONTENT;
        }
    }
    // 墓碑树、谱系树、退役树（recovery::read_container_tree）：零根跳过；先读完全部码 2 节点，再按叶序读每片容器、核容器与条目
    for (var container_tree = 0u; container_tree < 3u; container_tree = container_tree + 1u) {
        var kind = TREE_KIND_TOMBSTONE;
        var record_type = 1u;
        var record_width = 40u;
        if (container_tree == 1u) {
            kind = TREE_KIND_LINEAGE;
            record_type = 5u;
            record_width = 24u;
        } else if (container_tree == 2u) {
            kind = TREE_KIND_RETIREMENT;
            record_type = 3u;
            record_width = 34u;
        }
        let entry = tree_table_entry_offset_of_kind(tree_table, kind);
        let container_root_pointer = parse_node_pointer(tree_table.base, entry + 14u);
        if (container_root_pointer.all_zero) {
            continue;
        }
        for (var container_pass = 0u; container_pass < 2u; container_pass = container_pass + 1u) {
            let read = read_container_tree_index_nodes(container_root_pointer, tree_table.base, entry + 14u, rd64(tree_table.base, entry), root, fsid_low,
                select(NONE, record_type, container_pass == 1u), record_width);
            if (undecidable != 0u) {
                return READER_UNDECIDABLE;
            }
            if (!read.ok) {
                return reader_kind_of_the_failure_point(walk_failure_point);
            }
            if (container_pass == 0u) {
                deletion_tree_nodes = deletion_tree_nodes + read.mapped_units;
            }
        }
    }
    // 分配记录树整棵读回来，照三判逐条判
    let allocation_entry = tree_table_entry_offset_of_kind(tree_table, TREE_KIND_ALLOCATION);
    let allocation = read_allocation_tree(parse_node_pointer(tree_table.base, allocation_entry + 14u), tree_table.base, allocation_entry + 14u,
        rd64(tree_table.base, allocation_entry), root, fsid_low);
    if (undecidable != 0u) {
        return READER_UNDECIDABLE;
    }
    if (!allocation.ok) {
        return reader_kind_of_the_failure_point(walk_failure_point);
    }
    if (allocation_records_judged_against_the_pool_and_the_root(root, cfg) != FAILURE_POINT_NONE) {
        return READER_REFUSED_CONTENT;
    }
    // 记账树（照三判，再判这一版自己写的 inode 号水位那一行在：标签 12、代段 = 根的 txg，InodeNumberWatermarkRowMissingFromTheAccountingTree），
    // 再 deadlist 树（零根时没有条目；条目逐条宽 43，deadlist_entries_parsed）：同一个读法，一个调用点
    var accounting_nodes = 0u;
    var deadlist_nodes = 0u;
    var deadlist_entries = 0u;
    for (var code_two_tree = 0u; code_two_tree < 2u; code_two_tree = code_two_tree + 1u) {
        let is_the_deadlist = code_two_tree == 1u;
        let entry = tree_table_entry_offset_of_kind(tree_table, select(TREE_KIND_ACCOUNTING, TREE_KIND_DEADLIST, is_the_deadlist));
        let tree_root_pointer = parse_node_pointer(tree_table.base, entry + 14u);
        if (is_the_deadlist && tree_root_pointer.all_zero) {
            // 零根：一条 deadlist 条目都没有（叶表清空，下面查映射 key 那一圈不读别的树留下的叶）
            deadlist_leaf_count = 0u;
            continue;
        }
        let read = read_code_two_tree(tree_root_pointer, tree_table.base, entry + 14u, rd64(tree_table.base, entry),
            select(0u, KEY_SCHEMA_DEADLIST, is_the_deadlist), root, fsid_low);
        if (undecidable != 0u) {
            return READER_UNDECIDABLE;
        }
        if (!read.ok) {
            return reader_kind_of_the_failure_point(walk_failure_point);
        }
        if (is_the_deadlist) {
            deadlist_nodes = read.node_count;
            deadlist_entries = read.entry_count;
            for (var leaf_index = 0u; leaf_index < deadlist_leaf_count; leaf_index = leaf_index + 1u) {
                if (index_node_entries_of_the_header(sw(R_DEADLIST_LEAVES + leaf_index)).entry_width != DEADLIST_ENTRY_BYTES) {
                    return READER_REFUSED_CONTENT;
                }
            }
            continue;
        }
        accounting_nodes = read.node_count;
        if (accounting_entries_judged_against_the_root(root) != FAILURE_POINT_NONE) {
            return READER_REFUSED_CONTENT;
        }
        var the_inode_number_watermark_row_of_this_version_is_there = false;
        for (var accounting_index = 0u; accounting_index < accounting_count; accounting_index = accounting_index + 1u) {
            let base = sw(R_ACCOUNTING + accounting_index * 3u);
            let entry_at = sw(R_ACCOUNTING + accounting_index * 3u + 1u);
            if (rd16(base, entry_at) == 12u && eq64(rd64(base, entry_at + 14u), root.txg)) {
                the_inode_number_watermark_row_of_this_version_is_there = true;
            }
        }
        if (!the_inode_number_watermark_row_of_this_version_is_there) {
            return READER_REFUSED_CONTENT;
        }
    }
    // 中央映射树（CentralMappingTreeWithBytesReadOnFirstUse::into_tree）：前面没要过它的，走到这一步要它；读不出就停在这里
    if (mapping_tree_failure_point != FAILURE_POINT_NONE) {
        return reader_kind_of_the_failure_point(mapping_tree_failure_point);
    }
    // mapping_entry_count_is_one_per_mapped_unit
    if (mapping_count != 1u + extent_nodes + allocation.node_count + accounting_nodes + deadlist_nodes + deadlist_entries + inode_leaf_containers
        + data_units_of_every_file + deletion_tree_nodes) {
        return READER_REFUSED_CONTENT;
    }
    // 映射条目逐条解（parse_mapping_entry 窄于 55，EntryWidthDiffersFromItsFieldTable）；每条 deadlist 条目的映射 key（条目偏移 16 起 27 字节）在映射里有、
    // 两条 deadlist 条目不同一把映射 key（映射 key 在映射树里严格升序、不重）
    for (var leaf_index = 0u; leaf_index < mapping_leaf_count; leaf_index = leaf_index + 1u) {
        if (mapping_leaf_at(leaf_index).entries.entry_width < 55u) {
            return READER_REFUSED_CONTENT;
        }
    }
    for (var deadlist_leaf = 0u; deadlist_leaf < deadlist_leaf_count; deadlist_leaf = deadlist_leaf + 1u) {
        let base = sw(R_DEADLIST_LEAVES + deadlist_leaf);
        let entries = index_node_entries_of_the_header(base);
        for (var deadlist_entry = 0u; deadlist_entry < entries.entry_count; deadlist_entry = deadlist_entry + 1u) {
            let key_offset = entries.entries_start + deadlist_entry * entries.entry_width + 16u;
            if (!mapping_carries_the_key(base, key_offset)) {
                return READER_REFUSED_CONTENT;
            }
            if (an_earlier_deadlist_entry_carries_the_mapping_key(base, key_offset, deadlist_leaf, deadlist_entry)) {
                return READER_REFUSED_CONTENT;
            }
        }
    }
    // pointers_of_the_rebuilt_version_ascend_by_device 两遍（I-2.5）：盘上读来的指针，与换成映射位置之后的
    if (some_pointer_of_the_version_does_not_ascend_by_device || !mapping_tree_pointers_ascend_by_device
        || some_mapping_position_of_the_version_does_not_ascend_by_device) {
        return READER_REFUSED_CONTENT;
    }
    return READER_GETS_THROUGH;
}
// logged_operations::logged_operation_entries_of_the_root_entries：每条照 LoggedOperationEntry::parse 解（宽 44、种类码 1 整理或 2 截断；整理意图 R 槽数 64、
// 补零 6 字节为 0、起始槽 64 对齐；截断意图补零 10 字节为 0），几条按 (种类码, 签发 txg) 严格升序，整理意图、截断意图各至多一条
fn logged_operation_entries_parse(logged_operations_root: Node) -> bool {
    if (logged_operations_root.entry_width != LOGGED_OPERATIONS_ENTRY_BYTES) {
        return false;
    }
    var compaction_intents = 0u;
    var truncation_intents = 0u;
    for (var entry_index = 0u; entry_index < logged_operations_root.entry_count; entry_index = entry_index + 1u) {
        let base = logged_operations_root.base;
        let entry_at = entry_offset(logged_operations_root, entry_index);
        let kind = rd16(base, entry_at);
        if (kind == 1u) {
            let start_slot = rd48(base, entry_at + 14u);
            if (rd16(base, entry_at + 20u) != 64u || !all_zero(base, entry_at + 38u, 6u) || (start_slot.x & 63u) != 0u) {
                return false;
            }
            compaction_intents = compaction_intents + 1u;
        } else if (kind == 2u) {
            if (!all_zero(base, entry_at + 34u, 10u)) {
                return false;
            }
            truncation_intents = truncation_intents + 1u;
        } else {
            return false;
        }
        if (entry_index > 0u) {
            let previous = entry_offset(logged_operations_root, entry_index - 1u);
            let previous_kind = rd16(base, previous);
            if (!(previous_kind < kind || (previous_kind == kind && lt64(rd64(base, previous + 2u), rd64(base, entry_at + 2u))))) {
                return false;
            }
        }
    }
    return compaction_intents <= 1u && truncation_intents <= 1u;
}
// 中央映射里有没有一条条目的 key 与 (key_base, key_offset) 起的 27 字节逐字节相等（映射叶表 R_MAPPING_LEAVES，条目宽调用方判过不窄于 55）
fn mapping_carries_the_key(key_base: u32, key_offset: u32) -> bool {
    for (var leaf_index = 0u; leaf_index < mapping_leaf_count; leaf_index = leaf_index + 1u) {
        let leaf = mapping_leaf_at(leaf_index);
        for (var entry_index = 0u; entry_index < leaf.entries.entry_count; entry_index = entry_index + 1u) {
            if (bytes_equal(leaf.base, leaf.entries.entries_start + entry_index * leaf.entries.entry_width, key_base, key_offset, 27u)) {
                return true;
            }
        }
    }
    return false;
}
// deadlist 叶表里排在 (这片叶, 这一条) 之前的条目有没有同一把映射 key（条目偏移 16 起 27 字节）
fn an_earlier_deadlist_entry_carries_the_mapping_key(key_base: u32, key_offset: u32, this_leaf: u32, this_entry: u32) -> bool {
    for (var deadlist_leaf = 0u; deadlist_leaf <= this_leaf; deadlist_leaf = deadlist_leaf + 1u) {
        let base = sw(R_DEADLIST_LEAVES + deadlist_leaf);
        let entries = index_node_entries_of_the_header(base);
        let entries_before = select(entries.entry_count, this_entry, deadlist_leaf == this_leaf);
        for (var deadlist_entry = 0u; deadlist_entry < entries_before; deadlist_entry = deadlist_entry + 1u) {
            if (bytes_equal(base, entries.entries_start + deadlist_entry * entries.entry_width + 16u, key_base, key_offset, 27u)) {
                return true;
            }
        }
    }
    return false;
}


// recovery::system_configuration_slots_of_one_device_read_once 收进 self_verified 的那一判：SystemConfiguration::parse_slot 交 Ok
// （magic、整槽校验和、incompat 位认得、每区槽数在 [4, 16]；读者收不收那几个池级值不在这一判里）且 fsid 与所选系统配置的相同
fn system_configuration_slot_self_verified_with_the_fsid(s: Slice, fsid: vec4<u32>) -> bool {
    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES || rd32(s.base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
        return false;
    }
    if (!incompat_bits_are_recognized(s.base)) {
        return false;
    }
    let slots_per_region = rd8(s.base, 362u);
    return slots_per_region >= 4u && slots_per_region <= 16u && fsid_matches(s.base, 102u, fsid);
}
// 系统配置槽里 journal tail 的偏移（system_configuration.rs 的 TAIL_OFFSET）
const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: u32 = 469u;
// 内存池按扇区对齐判读请求（memory_pool 的 check_request_against_the_device_contract）：槽距不按 512 对齐的槽 1 读不到、算设备报错
const SECTOR_BYTES_OF_THE_MEMORY_POOL: u32 = 512u;

struct FallbackVersion {
    found: bool,
    root: RootFields,
};
// 择根的序（recovery 里 order_of_the_root_choice：checkpoint_txg 为主、实例代号破平局）上 (earlier_txg, earlier_instance) 排在 (later_txg, later_instance) 之前
fn precedes_in_the_root_choice_order(earlier_txg: vec2<u32>, earlier_instance: u32, later_txg: vec2<u32>, later_instance: u32) -> bool {
    return lt64(earlier_txg, later_txg) || (eq64(earlier_txg, later_txg) && earlier_instance < later_instance);
}
// recovery::instance_table_chain_of_root 读出的整张表的行，写进 R_ROWS（abandoned_by_rows 照它判）：沿链读法与走读判表读不读得出的
// instance_table_chain_readable 同一份（unit::parse_packed_unit、InstanceTablePage::parse，链指针头部按走读那一种判），只多把每片的行记下来。
// 一片读不出、解不开交 false；行多过 R_ROWS 装得下的、链长过 256 片的记判不了（CPU 不设这两个上限，那几个状态交 CPU 判）
fn load_rows_of_the_instance_table_chain(root: RootFields) -> bool {
    row_count = 0u;
    var pointer = root.instance_table;
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        let page_read = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (page_read.version == NONE) {
            return false;
        }
        let page_unit = parse_packed_unit(page_read);
        let parsed_page = parse_instance_table_page(page_unit, page_index);
        if (parsed_page.verdict == 2u) {
            return false;
        }
        for (var row_index = 0u; row_index + 1u < page_unit.record_count; row_index = row_index + 1u) {
            if (row_count >= MAX_ROWS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                return false;
            }
            let row_offset = packed_record_offset(page_unit, row_index);
            sws(R_ROWS + row_count * 3u, rd32(page_unit.base, row_offset + 1u));
            let row_txg = rd64(page_unit.base, row_offset + 5u);
            sws(R_ROWS + row_count * 3u + 1u, row_txg.x);
            sws(R_ROWS + row_count * 3u + 2u, row_txg.y);
            row_count = row_count + 1u;
        }
        if (parsed_page.verdict == 0u) {
            return true;
        }
        pointer = parsed_page.next;
        page_index = add64(page_index, u64_of(1u));
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}
// recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to，F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md`
// 第二节 K1 / K2 W4 H4 行与第三节第 2 条，用户 2026-10-08 定「四形都修」）：所选那一版（一条记录都没施加的所选根本身）走读失败、系统配置没见证它时，
// 退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（R_ROOTS 按根环次序记着 choose_root 自证过的根；同为最新的几条取根环次序上
// 最后那条，同 max_by_key）；所选根的实例表读不出、解不开时只认 (txg, 实例) 与见证到的那一版恰好相等的那条。「见证到的那一版」是计数器 = c_见证 的那条记录
// 所属那一版。见证判不出、见证过、那条记录读不出、c_见证 = 0 都不退：c_见证 = 0 时环里读得出计数器 0 的记录（写者的计数器从 1 起，那一条只能是
// 坏盘或伪造的）也不退，与 CPU 同一个守卫（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）。退回的那一版不重放、直接走。
// 「见证过」：所选那一版带末条标志的记录读得出 ⇒ 它的计数器 ≤ c_见证；读不出而 c_见证 那条记录读得出 ⇒ 那条记录的 (txg, 实例) 不比所选那一版旧；
// 两条都读不出当见证过。c_见证 落在所选那次发布自己的某条记录上时下界就是所选那一版、区间是空的，同样不退
fn newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(cfg: SysConfig, chosen: RootFields, selected: RootFields) -> FallbackVersion {
    var out: FallbackVersion;
    out.found = false;
    out.root = invalid_root();
    if (selected.instance != chosen.instance || !eq64(selected.txg, chosen.txg)) {
        return out;
    }
    // c_见证：每块盘两个系统配置槽（偏移 0 与所选系统配置的槽距）里自证过的 journal tail 取最大；见证判不出（R2）：有槽读不到、又没有一块盘两槽都自证过
    let device_count = hdr(3u);
    var witnessed_counter = vec2<u32>(0u, 0u);
    var some_slot_reports_a_read_error = false;
    var some_device_has_both_slots_self_verified = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        var self_verified_slots = 0u;
        for (var slot = 0u; slot < 2u; slot = slot + 1u) {
            let offset = select(0u, cfg.spacing, slot == 1u);
            // 内存池另把越过盘尾的读报成读错（check_request_against_the_device_contract 的 OutOfRange），这里不另判那一支：走到这一步时所选根已经
            // 从根环里读出来了（根环起点 ROOT_RING_BASE_BYTES = 1 MiB，CPU 读得出就说明盘至少这么大，池里每块盘一样大），读者收下的槽距不超过
            // 1 MiB − 4096（parse_system_configuration 的 largest_spacing），两槽 [0, 4096)、[槽距, 槽距 + 4096) 恒在盘内（判决
            // `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之一：在走得到的状态上两边同判）
            if ((offset % SECTOR_BYTES_OF_THE_MEMORY_POOL) != 0u) {
                some_slot_reports_a_read_error = true;
                continue;
            }
            let s = read_at(hdr(12u + d), u64_of(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
            if (!system_configuration_slot_self_verified_with_the_fsid(s, cfg.fsid)) {
                continue;
            }
            self_verified_slots = self_verified_slots + 1u;
            let tail = rd64(s.base, SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET);
            if (lt64(witnessed_counter, tail)) {
                witnessed_counter = tail;
            }
        }
        if (self_verified_slots == 2u) {
            some_device_has_both_slots_self_verified = true;
        }
    }
    if (some_slot_reports_a_read_error && !some_device_has_both_slots_self_verified) {
        return out;
    }
    var has_last_record_of_the_selected_version = false;
    var last_record_counter = vec2<u32>(0u, 0u);
    var has_witnessed_record = false;
    var witnessed_record_txg = vec2<u32>(0u, 0u);
    var witnessed_record_instance = 0u;
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = record_at(i);
        if (r.instance == selected.instance && eq64(r.txg, selected.txg) && r.last
            && (!has_last_record_of_the_selected_version || lt64(last_record_counter, r.counter))) {
            has_last_record_of_the_selected_version = true;
            last_record_counter = r.counter;
        }
        if (eq64(r.counter, witnessed_counter)
            && (!has_witnessed_record || lt64(witnessed_record_txg, r.txg)
                || (eq64(witnessed_record_txg, r.txg) && witnessed_record_instance < r.instance))) {
            has_witnessed_record = true;
            witnessed_record_txg = r.txg;
            witnessed_record_instance = r.instance;
        }
    }
    let witnessed_record_is_older_than_the_selected_version = lt64(witnessed_record_txg, selected.txg)
        || (eq64(witnessed_record_txg, selected.txg) && witnessed_record_instance < selected.instance);
    var is_witnessed = true;
    if (has_last_record_of_the_selected_version) {
        is_witnessed = le64(last_record_counter, witnessed_counter);
    } else if (has_witnessed_record) {
        is_witnessed = !witnessed_record_is_older_than_the_selected_version;
    }
    if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {
        return out;
    }
    let chosen_instance_table_is_readable = load_rows_of_the_instance_table_chain(chosen);
    // 所选根的实例表读不出时，见证到的那一版（R_ROOTS 里读得出的那条根）与所选根的实例表指针（根记录偏移 170 起 86 字节）相同，才另收区间里与所选根
    // 同实例的根（C751 甲，用户 2026-10-20 定；CPU 同名函数里那一判）。见证到的那一版的根读不出就比不了，不另收
    // 根记录里实例代号在偏移 24、checkpoint_txg 在 28、实例表单元指针在 170 起 86 字节（D22 已定项 7 字段表，parse_root 同一组偏移）
    var has_chosen_base = false;
    var chosen_base = 0u;
    var has_witnessed_base = false;
    var witnessed_base = 0u;
    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
        let root_base = B[T_VERSION_BYTES] + tab(T_VERSIONS, sw(R_ROOTS + root_index * 2u) * VERSION_WORDS + 1u);
        if (rd32(root_base, 24u) == chosen.instance && eq64(rd64(root_base, 28u), chosen.txg)) {
            has_chosen_base = true;
            chosen_base = root_base;
        }
        if (rd32(root_base, 24u) == witnessed_record_instance && eq64(rd64(root_base, 28u), witnessed_record_txg)) {
            has_witnessed_base = true;
            witnessed_base = root_base;
        }
    }
    let witnessed_version_shares_the_instance_table_pointer_of_the_chosen_root = has_chosen_base && has_witnessed_base
        && bytes_equal(witnessed_base, 170u, chosen_base, 170u, 86u);
    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
        var root_read: Slice = no_slice();
        root_read.version = sw(R_ROOTS + root_index * 2u);
        root_read.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, root_read.version * VERSION_WORDS + 1u);
        root_read.length = cfg.physical_block_size;
        root_read.c0 = tab(T_VERSIONS, root_read.version * VERSION_WORDS + 2u);
        let root = parse_root(root_read, cfg.fsid);
        let is_the_witnessed_version = root.instance == witnessed_record_instance && eq64(root.txg, witnessed_record_txg);
        var is_admissible = is_the_witnessed_version
            || (witnessed_version_shares_the_instance_table_pointer_of_the_chosen_root && root.instance == chosen.instance);
        if (chosen_instance_table_is_readable) {
            is_admissible = !abandoned_by_rows(root.instance, root.txg);
        }
        let is_in_the_range = !precedes_in_the_root_choice_order(root.txg, root.instance, witnessed_record_txg, witnessed_record_instance)
            && precedes_in_the_root_choice_order(root.txg, root.instance, chosen.txg, chosen.instance);
        if (is_in_the_range && is_admissible
            && (!out.found || !precedes_in_the_root_choice_order(root.txg, root.instance, out.root.txg, out.root.instance))) {
            out.found = true;
            out.root = root;
        }
    }
    // 不跳过一次末条读得出、根读不出、不被所选根的实例表抛弃的发布（CPU 同名函数里那一判，攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md`
    // 第二节 Y4）：退到的那一条与所选根之间（择根的序，两头都不含）有一条带末条标志的记录，R_ROOTS 里没有 (txg, 实例) 与它相等的根，
    // 所选根的实例表读得出时它的那次发布不被表抛弃（读不出时算不被抛弃）——就不退。R_ROOTS 里的根 choose_root 都自证过，只取实例代号与 txg 那两段比
    if (out.found) {
        for (var i = 0u; i < record_count; i = i + 1u) {
            let r = record_at(i);
            if (!r.last
                || !precedes_in_the_root_choice_order(out.root.txg, out.root.instance, r.txg, r.instance)
                || !precedes_in_the_root_choice_order(r.txg, r.instance, chosen.txg, chosen.instance)
                || (chosen_instance_table_is_readable && abandoned_by_rows(r.instance, r.txg))) {
                continue;
            }
            var root_of_the_publish_reads = false;
            for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
                let root_base = B[T_VERSION_BYTES] + tab(T_VERSIONS, sw(R_ROOTS + root_index * 2u) * VERSION_WORDS + 1u);
                if (rd32(root_base, 24u) == r.instance && eq64(rd64(root_base, 28u), r.txg)) {
                    root_of_the_publish_reads = true;
                }
            }
            if (!root_of_the_publish_reads) {
                out.found = false;
                out.root = invalid_root();
                return out;
            }
        }
    }
    return out;
}
struct NewestRoot {
    found: bool,
    txg: vec2<u32>,
    instance: u32,
};
// memory_pool::newest_persisted_root：持久的根槽写里 (txg, 实例) 最大的
fn newest_persisted_root() -> NewestRoot {
    var out: NewestRoot;
    out.found = false;
    out.txg = vec2<u32>(0u, 0u);
    out.instance = 0u;
    let count = hdr(7u);
    for (var i = 0u; i < count; i = i + 1u) {
        let root_write = tab(T_PUBLISHES, i * PUBLISH_WORDS);
        if (!is_persisted(root_write)) {
            continue;
        }
        let instance = tab(T_PUBLISHES, i * PUBLISH_WORDS + 1u);
        let txg = vec2<u32>(tab(T_PUBLISHES, i * PUBLISH_WORDS + 2u), tab(T_PUBLISHES, i * PUBLISH_WORDS + 3u));
        if (!out.found || lt64(out.txg, txg) || (eq64(out.txg, txg) && out.instance < instance)) {
            out.found = true;
            out.txg = txg;
            out.instance = instance;
        }
    }
    return out;
}
// 期望版本表第 5 字（crash_judge_tables::EXPECTED_VERSION_WITH_A_FILE / EXPECTED_VERSION_WITHOUT_A_FILE）：这一版有文件，还是文件从这一版起没有了
const EXPECTED_VERSION_WITH_A_FILE: u32 = 0u;
const EXPECTED_VERSION_WITHOUT_A_FILE: u32 = 1u;

// crash::classified_oracle_violation_for_the_version_table：落到的根是带文件的一版，读回的得是它的内容；是「文件从这一版起没有了」的一版，
// 只许报没有文件；不在表里，报没有文件时看比它旧的最新那一项（按 (txg, 实例)）——那一项有文件就是违例，没有文件、或没有更旧的项不是
// （文件从不消失的表上与「任一更旧版有文件」相同）；不在表里却读出了内容一律是违例
fn classify_oracle(recovery: Recovery, newest: NewestRoot) -> u32 {
    if (!recovery.has_effective) {
        return ORACLE_NO_ROOT_CHOSEN;
    }
    if (newest.found) {
        if (lt64(recovery.effective_txg, newest.txg) || (eq64(recovery.effective_txg, newest.txg) && recovery.effective_instance < newest.instance)) {
            return ORACLE_RECOVERED_TO_AN_OLDER_ROOT;
        }
    }
    let version = expected_version_index(recovery.effective_instance, recovery.effective_txg);
    var version_has_a_file = false;
    if (version != NONE) {
        version_has_a_file = tab(T_EXPECTED, version * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE;
    }
    if (recovery.outcome == OUTCOME_FAILED) {
        return ORACLE_READ_FAILED;
    }
    if (recovery.outcome == OUTCOME_NO_FILE) {
        if (version == NONE) {
            // 比它旧的最新那一项下面有文件？
            let count = hdr(8u);
            var newest_older = NONE;
            var newest_older_txg = vec2<u32>(0u, 0u);
            var newest_older_instance = 0u;
            for (var i = 0u; i < count; i = i + 1u) {
                let instance = tab(T_EXPECTED, i * EXPECTED_WORDS);
                let txg = vec2<u32>(tab(T_EXPECTED, i * EXPECTED_WORDS + 1u), tab(T_EXPECTED, i * EXPECTED_WORDS + 2u));
                let is_older = lt64(txg, recovery.effective_txg) || (eq64(txg, recovery.effective_txg) && instance < recovery.effective_instance);
                let is_newer_than_the_newest_older = newest_older == NONE || lt64(newest_older_txg, txg)
                    || (eq64(newest_older_txg, txg) && newest_older_instance < instance);
                if (is_older && is_newer_than_the_newest_older) {
                    newest_older = i;
                    newest_older_txg = txg;
                    newest_older_instance = instance;
                }
            }
            if (newest_older != NONE && tab(T_EXPECTED, newest_older * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE) {
                return ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE;
            }
            return ORACLE_NONE;
        }
        if (!version_has_a_file) {
            return ORACLE_NONE;
        }
        return ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE;
    }
    if (version == NONE || !version_has_a_file) {
        return ORACLE_FILE_READ_WHERE_THIS_ROOT_HAS_NONE;
    }
    if (!recovery.content_matches) {
        return ORACLE_WRONG_CONTENT;
    }
    return ORACLE_NONE;
}

// ───────────────────────── 记录核对器 ─────────────────────────

// 一次写还在盘上：它落点的版本在它的每个扇区上都是它的字节
fn write_in_place(write_index: u32) -> bool {
    let location = tab(T_WRITES, write_index * WRITE_WORDS + 5u);
    if (location == NONE) {
        return false;
    }
    let version = version_of_location(location);
    if (version == NONE) {
        return false;
    }
    let writes_first = location_word(location, 5u);
    let writes_count = location_word(location, 6u);
    var position = NONE;
    for (var i = 0u; i < writes_count; i = i + 1u) {
        if (tab(T_LOCATION_WRITES, writes_first + i) == write_index) {
            position = i;
        }
    }
    if (position == NONE) {
        return false;
    }
    let equal_first = tab(T_VERSIONS, version * VERSION_WORDS + 6u);
    let mask = vec2<u32>(tab(T_EQUAL_MASKS, (equal_first + position) * 2u), tab(T_EQUAL_MASKS, (equal_first + position) * 2u + 1u));
    let sectors = tab(T_WRITES, write_index * WRITE_WORDS + 3u) >> 9u;
    for (var s = 0u; s < sectors; s = s + 1u) {
        var bit = 0u;
        if (s < 32u) {
            bit = (mask.x >> s) & 1u;
        } else {
            bit = (mask.y >> (s - 32u)) & 1u;
        }
        if (bit == 0u) {
            return false;
        }
    }
    return true;
}
// crash::unit_copy_is_missing_under_the_persisted_set
fn unit_copy_is_missing(copy: u32) -> bool {
    let location = tab(T_WRITES, copy * WRITE_WORDS + 5u);
    let version = version_of_location(location);
    if (version == NONE) {
        return true;
    }
    let writes_first = location_word(location, 5u);
    let writes_count = location_word(location, 6u);
    var position = NONE;
    for (var i = 0u; i < writes_count; i = i + 1u) {
        if (tab(T_LOCATION_WRITES, writes_first + i) == copy) {
            position = i;
        }
    }
    if (position == NONE) {
        return true;
    }
    let equal_first = tab(T_VERSIONS, version * VERSION_WORDS + 6u);
    let mask = vec2<u32>(tab(T_EQUAL_MASKS, (equal_first + position) * 2u), tab(T_EQUAL_MASKS, (equal_first + position) * 2u + 1u));
    let sectors = tab(T_WRITES, copy * WRITE_WORDS + 3u) >> 9u;
    let laters_first = tab(T_WRITES, copy * WRITE_WORDS + 10u);
    let laters_count = tab(T_WRITES, copy * WRITE_WORDS + 11u);
    for (var s = 0u; s < sectors; s = s + 1u) {
        var bit = 0u;
        if (s < 32u) {
            bit = (mask.x >> s) & 1u;
        } else {
            bit = (mask.y >> (s - 32u)) & 1u;
        }
        if (bit == 1u) {
            continue;
        }
        var explained = false;
        for (var l = 0u; l < laters_count; l = l + 1u) {
            let later = tab(T_LATERS, (laters_first + l) * LATER_WORDS);
            let legal = tab(T_LATERS, (laters_first + l) * LATER_WORDS + 1u);
            let covered = vec2<u32>(tab(T_LATERS, (laters_first + l) * LATER_WORDS + 2u), tab(T_LATERS, (laters_first + l) * LATER_WORDS + 3u));
            var covers = 0u;
            if (s < 32u) {
                covers = (covered.x >> s) & 1u;
            } else {
                covers = (covered.y >> (s - 32u)) & 1u;
            }
            if (is_persisted(later) && covers == 1u && legal == 1u) {
                explained = true;
                break;
            }
        }
        if (!explained) {
            return true;
        }
    }
    return false;
}
// 恢复落到的那一版的实例表（写表里最后一次写出这个根身份的根槽写 → 它的实例表指针 → 沿链读，只读不判的读法）；行写进 R_ROWS
fn load_instance_table_of_the_effective_root(instance: u32, txg: vec2<u32>) -> bool {
    row_count = 0u;
    let write_count = hdr(1u);
    var root_bytes = NONE;
    for (var w = 0u; w < write_count; w = w + 1u) {
        let i = write_count - 1u - w;
        if (tab(T_WRITES, i * WRITE_WORDS + 4u) != WRITE_ROOT) {
            continue;
        }
        if (tab(T_WRITES, i * WRITE_WORDS + 6u) == instance && tab(T_WRITES, i * WRITE_WORDS + 7u) == txg.x && tab(T_WRITES, i * WRITE_WORDS + 8u) == txg.y) {
            root_bytes = tab(T_WRITES, i * WRITE_WORDS + 9u);
            break;
        }
    }
    if (root_bytes == NONE) {
        return false;
    }
    var pointer = parse_node_pointer(B[T_ROOT_BYTES] + root_bytes, 170u);
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        // read_unit_without_judging：整单元 CRC 与位置条目相符
        let s = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            return false;
        }
        // check_unit == Ok(3)：magic、flags、类 3、两道校验和、版本、预留位
        let base = s.base;
        if (rd32(base, 0u) != MAGIC_SFSU || rd8(base, 7u) != 0u || rd8(base, 6u) != 3u || (s.c2 & 11u) != 11u || rd16(base, 4u) != 1u) {
            return false;
        }
        if (!is_zero64(rd64(base, 43u)) || rd16(base, 51u) != 4u || !eq64(rd64(base, 53u), page_index) || !is_zero64(rd64(base, 61u))) {
            return false;
        }
        let count = rd16(base, 69u);
        let width = rd16(base, 71u);
        let declared = rd16(base, 8u);
        if (width != 88u || count * width > declared || declared > DATA_UNIT_BYTES - 136u) {
            return false;
        }
        if (count == 0u) {
            return false;
        }
        for (var r = 0u; r + 1u < count; r = r + 1u) {
            let o = 136u + r * 88u;
            // 行的 flags（偏移 21）非 0 同样解不开（InstanceTableOfRootRecord::read 同判；D18 已定项 11「第一版恒 0、非 0 拒收」）
            if (rd8(base, o) != 0u || rd8(base, o + 21u) != 0u) {
                return false;
            }
            if (row_count >= MAX_ROWS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                return false;
            }
            sws(R_ROWS + row_count * 3u, rd32(base, o + 1u));
            let row_txg = rd64(base, o + 5u);
            sws(R_ROWS + row_count * 3u + 1u, row_txg.x);
            sws(R_ROWS + row_count * 3u + 2u, row_txg.y);
            row_count = row_count + 1u;
        }
        let last = 136u + (count - 1u) * 88u;
        if (rd8(base, last) != 1u) {
            return false;
        }
        let next = parse_node_pointer(base, last + 2u);
        let flag = rd8(base, last + 1u);
        if (flag == 0u && next.all_zero) {
            return true;
        }
        // 有下一片还要位置指针头部 extent 偏移及之前那几段都是第一版的取值（Ptr 的 head_ok 与 extent_offset_zero；walk.rs chain_record_view 同判）：
        // 有一段不过是 I-3.14 第五形，这一片解不开、读不出整张表（用户 2026-10-06 定 K2-1 甲，第 10u 批）
        if (flag == 1u && !next.all_zero && next.head_ok && next.extent_offset_zero) {
            pointer = next;
            page_index = add64(page_index, u64_of(1u));
            continue;
        }
        return false;
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}
fn abandoned_by_rows(instance: u32, txg: vec2<u32>) -> bool {
    for (var i = 0u; i < row_count; i = i + 1u) {
        if (sw(R_ROWS + i * 3u) == instance) {
            let row_txg = vec2<u32>(sw(R_ROWS + i * 3u + 1u), sw(R_ROWS + i * 3u + 2u));
            if (lt64(row_txg, txg)) {
                return true;
            }
        }
    }
    return false;
}
struct RecordVerdict {
    root_without_record: bool,
    claimed_state_missing_unit: bool,
    claimed_version_not_landed_on_by_the_recovery: bool,
};
// crash::check_records（崩溃点重放：两份镜像同一份，一条录制流）。恢复自称的那一版：给定状态表带了那一栏（T_CLAIMED_VERSIONS 不空，
// crash_judge_tables::JudgeStateSource::GivenPersistedSets 的 claimed_versions）而这个状态写了的，照写的那一版（崩溃注入第二截挂载之后的池：
// 挂载与那次发布写出的最新那条根，crash_given_states::judge_a_given_state_on_the_cpu 同取法），写的那一版与看 journal 那一遍落到的不同另判红；
// 没写的照看 journal 那一遍落到的根
fn check_records(consulted: Recovery) -> RecordVerdict {
    var out: RecordVerdict;
    out.root_without_record = false;
    out.claimed_state_missing_unit = false;
    out.claimed_version_not_landed_on_by_the_recovery = false;
    var claimed_has_a_root = consulted.has_effective;
    var claimed_instance = consulted.effective_instance;
    var claimed_txg = consulted.effective_txg;
    if (table_length(T_CLAIMED_VERSIONS) != 0u) {
        let claimed_at = given_state_ordinal * CLAIMED_VERSION_WORDS;
        let claimed_code = tab(T_CLAIMED_VERSIONS, claimed_at);
        if (claimed_code != CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY) {
            claimed_has_a_root = claimed_code == CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT;
            claimed_instance = tab(T_CLAIMED_VERSIONS, claimed_at + 1u);
            claimed_txg = vec2<u32>(tab(T_CLAIMED_VERSIONS, claimed_at + 2u), tab(T_CLAIMED_VERSIONS, claimed_at + 3u));
            if (claimed_has_a_root != consulted.has_effective) {
                out.claimed_version_not_landed_on_by_the_recovery = true;
            } else if (claimed_has_a_root
                && (claimed_instance != consulted.effective_instance
                    || claimed_txg.x != consulted.effective_txg.x
                    || claimed_txg.y != consulted.effective_txg.y)) {
                out.claimed_version_not_landed_on_by_the_recovery = true;
            }
        }
    }
    var have_table = false;
    if (claimed_has_a_root) {
        have_table = load_instance_table_of_the_effective_root(claimed_instance, claimed_txg);
    }
    let count = hdr(7u);
    for (var i = 0u; i < count; i = i + 1u) {
        let root_write = tab(T_PUBLISHES, i * PUBLISH_WORDS);
        let instance = tab(T_PUBLISHES, i * PUBLISH_WORDS + 1u);
        let txg = vec2<u32>(tab(T_PUBLISHES, i * PUBLISH_WORDS + 2u), tab(T_PUBLISHES, i * PUBLISH_WORDS + 3u));
        let records_first = tab(T_PUBLISHES, i * PUBLISH_WORDS + 4u);
        let records_count = tab(T_PUBLISHES, i * PUBLISH_WORDS + 5u);
        let groups_first = tab(T_PUBLISHES, i * PUBLISH_WORDS + 6u);
        let groups_count = tab(T_PUBLISHES, i * PUBLISH_WORDS + 7u);
        if (records_count > 0u && write_in_place(root_write)) {
            var any_record = false;
            for (var r = 0u; r < records_count; r = r + 1u) {
                if (write_in_place(tab(T_PUBLISH_RECORDS, records_first + r))) {
                    any_record = true;
                    break;
                }
            }
            if (!any_record) {
                out.root_without_record = true;
            }
        }
        if (claimed_has_a_root && le64(txg, claimed_txg)) {
            var abandoned = false;
            if (have_table) {
                abandoned = abandoned_by_rows(instance, txg);
            }
            if (!abandoned) {
                for (var g = 0u; g < groups_count; g = g + 1u) {
                    let copies_first = tab(T_COPY_GROUPS, (groups_first + g) * 2u);
                    let copies_count = tab(T_COPY_GROUPS, (groups_first + g) * 2u + 1u);
                    var all_missing = true;
                    for (var c = 0u; c < copies_count; c = c + 1u) {
                        if (!unit_copy_is_missing(tab(T_COPIES, copies_first + c))) {
                            all_missing = false;
                            break;
                        }
                    }
                    if (all_missing) {
                        out.claimed_state_missing_unit = true;
                    }
                }
            }
        }
    }
    return out;
}

// ───────────────────────── 入口 ─────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    // 判
    let cfg_choice = choose_system_configuration();
    // 看 journal 与不看 journal 两遍：一个调用点在两轮里走（GPU 程序把每个调用点展开成一份拷贝，恢复连同判退的三样读者只展开一份）
    var consulted: Recovery;
    var ignored: Recovery;
    for (var recovery_pass = 0u; recovery_pass < 2u; recovery_pass = recovery_pass + 1u) {
        let recovered = recover(recovery_pass == 0u, cfg_choice);
        if (recovery_pass == 0u) {
            consulted = recovered;
        } else {
            ignored = recovered;
        }
    }
    let newest = newest_persisted_root();
    let consulted_oracle = classify_oracle(consulted, newest);
    let ignored_oracle = classify_oracle(ignored, newest);
    let records = check_records(consulted);
    var bits = 0u;
    if (consulted_oracle != ORACLE_NONE) {
        bits = bits | (1u << (BIT_CONSULTED_ORACLE + consulted_oracle));
    }
    if (ignored_oracle != ORACLE_NONE) {
        bits = bits | (1u << (BIT_IGNORED_ORACLE + ignored_oracle));
    }
    if (records.root_without_record) {
        bits = bits | (1u << BIT_ROOT_WITHOUT_RECORD);
    }
    if (records.claimed_state_missing_unit) {
        bits = bits | (1u << BIT_CLAIMED_STATE_MISSING_UNIT);
    }
    if (records.claimed_version_not_landed_on_by_the_recovery) {
        bits = bits | (1u << BIT_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY);
    }
    if (undecidable != 0u) {
        bits = bits | (1u << BIT_UNDECIDABLE);
    }
    let out = local * VERDICT_WORDS;
    verdicts[out] = bits;
    // 判得了时第 1 字本来恒 0：借它带出看 journal 那一遍重放停在哪一道（诊断，`replay_stop_reason`；高 16 位给 record_count）
    if (undecidable != 0u) {
        verdicts[out + 1u] = undecidable;
    } else {
        verdicts[out + 1u] = replay_stop_reason | (record_count << 16u);
    }
    // 两遍各自落到的根：落没落到根的标记、实例代号、checkpoint_txg 低 32 位（x）与高 32 位（y），完整的 u64 都写出去（用户 2026-10-24 定
    // 「存完整 u64」，审计复核 T01）。没落到根（择不到系统配置或根、重放在施加任何记录之前停下）时标记是 PASS_LANDED_ON_NO_ROOT、
    // 另三个字都是 NONE（CPU 的 effective_root 为 None，crash_judge_conclusions.rs 的 LandedRootWords::NOT_LANDED）
    var consulted_marker = PASS_LANDED_ON_NO_ROOT;
    var consulted_instance = NONE;
    var consulted_txg = vec2<u32>(NONE, NONE);
    if (consulted.has_effective) {
        consulted_marker = PASS_LANDED_ON_A_ROOT;
        consulted_instance = consulted.effective_instance;
        consulted_txg = consulted.effective_txg;
    }
    var ignored_marker = PASS_LANDED_ON_NO_ROOT;
    var ignored_instance = NONE;
    var ignored_txg = vec2<u32>(NONE, NONE);
    if (ignored.has_effective) {
        ignored_marker = PASS_LANDED_ON_A_ROOT;
        ignored_instance = ignored.effective_instance;
        ignored_txg = ignored.effective_txg;
    }
    verdicts[out + CONSULTED_PASS_LANDED_MARKER_WORD] = consulted_marker;
    verdicts[out + CONSULTED_PASS_LANDED_INSTANCE_WORD] = consulted_instance;
    verdicts[out + CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = consulted_txg.x;
    verdicts[out + CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = consulted_txg.y;
    verdicts[out + CONSULTED_PASS_OUTCOME_WORD] = consulted.outcome;
    verdicts[out + CONSULTED_PASS_FAILURE_POINT_WORD] = consulted.failure;
    verdicts[out + IGNORED_PASS_OUTCOME_WORD] = ignored.outcome;
    verdicts[out + IGNORED_PASS_FAILURE_POINT_WORD] = ignored.failure;
    verdicts[out + IGNORED_PASS_LANDED_MARKER_WORD] = ignored_marker;
    verdicts[out + IGNORED_PASS_LANDED_INSTANCE_WORD] = ignored_instance;
    verdicts[out + IGNORED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD] = ignored_txg.x;
    verdicts[out + IGNORED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD] = ignored_txg.y;
}
