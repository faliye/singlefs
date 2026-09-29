// GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
// 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），违例位并进第 8、9 字，
// 第 10 字写成 5、判不了时第 11 字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。


var<private> r_complete: bool;

var<private> r_noting: bool;

var<private> r_candidate: u32;

var<private> r_seen_count: u32;

var<private> c_released_count: u32;

var<private> c_alloc_record_count: u32;

var<private> c_seen_node_count: u32;

var<private> c_examined_count: u32;

var<private> c_harvested: bool;

var<private> c_exempted: array<u32, 8>;

var<private> c_deferred_per_device: array<u32, 8>;

// 引用集合这一遍「记一条指针」：只回答「这条指针指着哪条已释放记录的落点」，位记在 (记录, 候选根) 上
fn r_note(p: Ptr) {
    if (!r_noting) {
        return;
    }
    for (var k = 0u; k < c_released_count; k = k + 1u) {
        let device = sw(C_RELEASED + k * 5u);
        let slot = vec2<u32>(sw(C_RELEASED + k * 5u + 1u), sw(C_RELEASED + k * 5u + 2u));
        if ((p.device0 == device && eq64(p.slot0, slot)) || (p.device1 == device && eq64(p.slot1, slot))) {
            let word = C_REFERENCED_BY + k * 2u + (r_candidate >> 5u);
            sws(word, sw(word) | (1u << (r_candidate & 31u)));
        }
    }
}

// read_index_node_without_judging：第一份校验和对得上的那一份，按 index_node_view 解
fn r_read_index_node(p: Ptr) -> Node {
    let s = read_unit_via_locations(p, NODE_BYTES);
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
        r_note(p);
        let n = r_read_index_node(p);
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
        r_note(p);
        let n = r_read_index_node(p);
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
                    if (tag == 0u) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return false;
                        }
                    } else if (tag == 1u) {
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return false;
                        }
                        if (pending >= C_R_MAX_PENDING) {
                            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (22u << 8u);
                            return false;
                        }
                        let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
                        let entry_inode = extent_key_inode(n.base, o);
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
                    } else if (tag == 2u) {
                        r_note(parse_data_pointer(n.base, o + 25u));
                    } else {
                        return false;
                    }
                }
            } else {
                if (n.entry_width < 112u) {
                    return false;
                }
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    r_note(parse_data_pointer(n.base, entry_offset(n, e) + 24u));
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
        r_note(p);
        let child = r_read_index_node(p);
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
        r_note(p);
        let record_type = rd16(base, o + 16u);
        if (record_type == 2u) {
            continue;
        }
        if (record_type != 0u) {
            r_complete = false;
            continue;
        }
        let child = r_read_index_node(p);
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
    for (var step = 0u; step < 256u; step = step + 1u) {
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
        let last = packed_record_offset(packed, packed.record_count - 1u);
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
        p = next;
        page = add64(page, u64_of(1u));
    }
    r_complete = false;
}

// references_of_root：一条根引用了什么（只读不判），走全没走全、树表落点与树表条目的诞生 txg 记进候选根的信息行
fn r_references_of_root(record_base: u32, every: bool, info: u32) {
    let row = C_CANDIDATE_INFO + info * C_CANDIDATE_WORDS;
    r_complete = true;
    sws(row + 1u, 0u);
    sws(row + 5u, 0u);
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
        let tree = rd64(tt.base, o);
        let birth = rd64(tt.base, o + 108u);
        var tree_count = sw(row + 5u);
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
        let tree_root = parse_node_pointer(tt.base, o + 14u);
        if (tree_root.all_zero) {
            continue;
        }
        r_note(tree_root);
        if (!every) {
            continue;
        }
        let node = r_read_index_node(tree_root);
        if (!node.ok) {
            r_complete = false;
            continue;
        }
        let kind = rd16(tt.base, o + 10u);
        if (kind == TREE_KIND_EXTENT) {
            if (!r_extent_tree(tt.base, o + 14u)) {
                r_complete = false;
            }
        } else if (kind == TREE_KIND_INODE) {
            if (node.level > 0u && node.entry_width >= 120u) {
                r_note_every_inode_child_below(node);
            } else {
                r_complete = false;
            }
        } else if (kind == TREE_KIND_ALLOCATION) {
            if (!r_allocation_tree(tt.base, o + 14u)) {
                r_complete = false;
            }
        } else if (kind == TREE_KIND_ACCOUNTING) {
            r_note_every_node_below(node, 0u);
        } else {
            r_complete = false;
        }
    }
    if (every && !mapping_root.all_zero) {
        let mapping = r_read_index_node(mapping_root);
        if (mapping.ok) {
            r_note_every_node_below(mapping, 1u);
        } else {
            r_complete = false;
        }
    }
    sws(row, select(0u, 1u, r_complete));
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
    r_noting = false;
    c_alloc_record_count = 0u;
    r_references_of_root(newest_root.base, true, newest_slot);
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
    let every = c_released_count > 0u;
    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
        let index = sw(C_CANDIDATES + c);
        let root = checker_root_of(index);
        r_candidate = c;
        r_noting = every;
        c_alloc_record_count = 0u;
        r_references_of_root(root.base, every || index == c_newest_index, c);
    }
    r_noting = false;
    // I-3.9
    if (every) {
        var every_complete = true;
        for (var c = 0u; c < c_candidate_count; c = c + 1u) {
            if (sw(C_CANDIDATE_INFO + c * C_CANDIDATE_WORDS) == 0u) {
                every_complete = false;
            }
        }
        if (every_complete) {
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
                var have_first = false;
                var first_without = vec2<u32>(0u, 0u);
                for (var c = 0u; c < c_candidate_count; c = c + 1u) {
                    let txg = checker_root_of(sw(C_CANDIDATES + c)).txg;
                    if (lt64(last, txg) && (!have_first || lt64(txg, first_without))) {
                        first_without = txg;
                        have_first = true;
                    }
                }
                let generation = vec2<u32>(sw(C_RELEASED + k * 5u + 3u), sw(C_RELEASED + k * 5u + 4u));
                if (!(lt64(last, generation) && have_first && le64(generation, first_without))) {
                    violate(INVARIANT_I_3_9);
                }
            }
        }
    }
    // I-9.14
    for (var a = 0u; a < c_candidate_count; a = a + 1u) {
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
            for (var c = a; c < c_candidate_count; c = c + 1u) {
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
            if (distinct_tables >= 2u && !same_birth) {
                violate(INVARIANT_I_9_14);
            }
        }
    }
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

// I-5.4 的一棵树：同一块盘上任意两条记录罩住的槽区间互不相交（按 (槽, 跨度) 排序后相邻比 ⟺ 任两条比）
fn judge_allocation_record_ranges() {
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        let device = sw(a);
        let slot = vec2<u32>(sw(a + 1u), sw(a + 2u));
        let span = sw(a + 3u);
        let end = add64(slot, u64_of(span));
        for (var j = 0u; j < c_alloc_record_count; j = j + 1u) {
            if (i == j) {
                continue;
            }
            let b = C_ALLOC_RECORDS + j * C_ALLOC_RECORD_WORDS;
            if (sw(b) != device) {
                continue;
            }
            let other_slot = vec2<u32>(sw(b + 1u), sw(b + 2u));
            let other_span = sw(b + 3u);
            let a_first = lt64(slot, other_slot) || (eq64(slot, other_slot) && (span < other_span || (span == other_span && i < j)));
            if (a_first && lt64(other_slot, end)) {
                violate(INVARIANT_I_5_4);
            }
        }
    }
}

// I-5.4：候选集里每条根的分配记录树（树表种类 3 那一条与根记录直接持有的那一棵）
fn judge_allocation_records_disjoint() {
    c_seen_node_count = 0u;
    r_noting = false;
    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
        let root = checker_root_of(sw(C_CANDIDATES + c));
        let held = parse_node_pointer(root.base, 342u);
        if (!held.all_zero && c_seen_node_insert(held)) {
            c_alloc_record_count = 0u;
            if (r_allocation_tree(root.base, 342u)) {
                judge_allocation_record_ranges();
            }
        }
        let tree_table_pointer = parse_node_pointer(root.base, 36u);
        if (tree_table_pointer.all_zero) {
            continue;
        }
        let tt = r_read_index_node(tree_table_pointer);
        if (!tt.ok || tt.entry_width < 200u) {
            continue;
        }
        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
            let o = entry_offset(tt, e);
            if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
                continue;
            }
            let p = parse_node_pointer(tt.base, o + 14u);
            if (p.all_zero || !c_seen_node_insert(p)) {
                continue;
            }
            c_alloc_record_count = 0u;
            if (r_allocation_tree(tt.base, o + 14u)) {
                judge_allocation_record_ranges();
            }
        }
    }
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
        var examined = false;
        for (var k = 0u; k < c_examined_count; k = k + 1u) {
            if (sw(C_EXAMINED + k * 5u) == device && sw(C_EXAMINED + k * 5u + 1u) == slot.x && sw(C_EXAMINED + k * 5u + 2u) == slot.y
                && sw(C_EXAMINED + k * 5u + 3u) == generation.x && sw(C_EXAMINED + k * 5u + 4u) == generation.y) {
                examined = true;
            }
        }
        if (examined) {
            continue;
        }
        if (c_examined_count >= C_MAX_EXAMINED) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (31u << 8u);
            return;
        }
        sws(C_EXAMINED + c_examined_count * 5u, device);
        sws(C_EXAMINED + c_examined_count * 5u + 1u, slot.x);
        sws(C_EXAMINED + c_examined_count * 5u + 2u, slot.y);
        sws(C_EXAMINED + c_examined_count * 5u + 3u, generation.x);
        sws(C_EXAMINED + c_examined_count * 5u + 4u, generation.y);
        c_examined_count = c_examined_count + 1u;
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
        if (!eq64(generation, birth)) {
            violate(INVARIANT_I_3_10);
        }
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
        let p = parse_node_pointer(tt.base, o + 14u);
        if (p.all_zero || !c_seen_node_insert(p)) {
            continue;
        }
        c_alloc_record_count = 0u;
        if (r_allocation_tree(tt.base, o + 14u)) {
            judge_allocation_generations_of_the_records();
        }
    }
}

// I-3.10：候选根、由记录施加出来的版本、下一次挂载会先施加的那一版
fn judge_allocation_generations_against_unit_births() {
    c_seen_node_count = 0u;
    c_examined_count = 0u;
    r_noting = false;
    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
        let root = checker_root_of(sw(C_CANDIDATES + c));
        let held = parse_node_pointer(root.base, 342u);
        if (!held.all_zero && c_seen_node_insert(held)) {
            c_alloc_record_count = 0u;
            if (r_allocation_tree(root.base, 342u)) {
                judge_allocation_generations_of_the_records();
            }
        }
        judge_allocation_generations_under_tree_table(parse_node_pointer(root.base, 36u));
    }
    for (var v = 0u; v < c_version_count; v = v + 1u) {
        let r = checker_record_at(sw(R_DATA + v));
        judge_allocation_generations_under_tree_table(parse_node_pointer(r.base, 99u));
    }
    // 下一次挂载会先施加的那一版：与最新根同实例、txg = 最新根 txg + 1、带提交标记、进得了前缀、根槽读不出
    let newest_root = checker_root_of(c_newest_index);
    if (!add64_overflows(newest_root.txg, u64_of(1u))) {
        let next_txg = add64(newest_root.txg, u64_of(1u));
        var root_slot_is_readable = false;
        for (var i = 0u; i < root_count; i = i + 1u) {
            let root = checker_root_of(i);
            if (root.instance == newest_root.instance && eq64(root.txg, next_txg)) {
                root_slot_is_readable = true;
            }
        }
        if (!root_slot_is_readable) {
            for (var i = 0u; i < record_count; i = i + 1u) {
                let r = checker_record_at(i);
                if (r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {
                    judge_allocation_generations_under_tree_table(parse_node_pointer(r.base, 99u));
                    break;
                }
            }
        }
    }
}

// instance_table_rows_of_root_without_judging：一条根自己的实例表沿链读出全部行，读不出交回 false
fn r_instance_rows(record_base: u32) -> bool {
    var p = parse_node_pointer(record_base, 170u);
    var page = vec2<u32>(0u, 0u);
    var rows = 0u;
    for (var step = 0u; step < 256u; step = step + 1u) {
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
            if (rd8(s.base, o) != 0u) {
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

// user_visible_tree_root_pointers：一条根树表里 inode 树与 extent 树的根指针（字节位置）；判不了写 has = 0
fn c_user_visible_pointers(record_base: u32, out_row: u32) {
    sws(out_row + 1u, 0u);
    sws(out_row + 2u, NONE);
    sws(out_row + 3u, 0u);
    sws(out_row + 4u, NONE);
    sws(out_row + 5u, 0u);
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
}

// 两条根的用户可见树根指针相同（两棵各自都没有、或 86 字节逐字相同）
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
    return true;
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

// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限
fn judge_rollback_floor_raises_against_their_ceilings() {
    let device_count = hdr(3u);
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
            var newest_before = vec2<u32>(0u, 0u);
            for (var k = 0u; k < root_count; k = k + 1u) {
                let txg = checker_root_of(k).txg;
                if (lt64(txg, raising.txg) && lt64(newest_before, txg)) {
                    newest_before = txg;
                }
            }
            if (lt64(newest_before, raising.floor)) {
                violate(INVARIANT_I_7_9);
            }
            continue;
        }
        if (!r_instance_rows(raising.base)) {
            continue;
        }
        // 它之前的有效根：txg < 它的、≥ 抬之前的 F、按它自己的实例表不被抛弃；按 (txg, 实例) 升序
        var valid_count = 0u;
        for (var k = 0u; k < root_count; k = k + 1u) {
            let root = checker_root_of(k);
            if (!lt64(root.txg, raising.txg) || lt64(root.txg, floor_before) || r_rows_abandon(root.instance, root.txg)) {
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
                    differs = sw(row + 2u) != NONE || sw(row + 4u) != NONE;
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
        if (lt64(lowest, raising.floor)) {
            violate(INVARIANT_I_7_9);
        }
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

// quarantined_slots_exempted_per_device：最新根的分配记录树里未释放、没有根引用起在它那一槽的记录，按单元归组；有一份读回来对不上，整组豁免
fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, fsid_low: vec2<u32>) {
    let device_count = hdr(3u);
    for (var d = 0u; d < 8u; d = d + 1u) {
        c_exempted[d] = 0u;
    }
    let tree_table_pointer = parse_node_pointer(newest_base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < 200u) {
        return;
    }
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
        }
    }
}

// deferred_slots_referenced_only_below_the_floor_per_device
fn c_deferred_below_the_floor(newest_base: u32) {
    let device_count = hdr(3u);
    for (var d = 0u; d < 8u; d = d + 1u) {
        c_deferred_per_device[d] = 0u;
    }
    let tree_table_pointer = parse_node_pointer(newest_base, 36u);
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = r_read_index_node(tree_table_pointer);
    if (!tt.ok || tt.entry_width < 200u) {
        return;
    }
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
            sws(C_RELEASED_ABOVE + released_above * 4u, sw(a));
            sws(C_RELEASED_ABOVE + released_above * 4u + 1u, sw(a + 1u));
            sws(C_RELEASED_ABOVE + released_above * 4u + 2u, sw(a + 2u));
            sws(C_RELEASED_ABOVE + released_above * 4u + 3u, sw(a + 3u));
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
            let record = C_RELEASED_ABOVE + q * 4u;
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

fn judge_accounting(fsid_low: vec2<u32>) {
    if (!w_accounting_seen) {
        return;
    }
    let device_count = hdr(3u);
    let newest_root = checker_root_of(c_newest_index);
    let written_under_effective_floor = eq64(newest_root.floor, c_effective_floor);
    c_harvested = false;
    c_quarantine_exempted(newest_root.base, w_reference_count[0], fsid_low);
    var exempted_all: array<u32, 8>;
    for (var d = 0u; d < 8u; d = d + 1u) {
        exempted_all[d] = c_exempted[d];
    }
    c_deferred_below_the_floor(newest_root.base);
    c_quarantine_exempted(newest_root.base, c_newest_reference_count, fsid_low);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var allocated = vec2<u32>(0u, 0u);
        let have_allocated = c_accounting_row(1u, device, &allocated);
        let walked = shl64(add64(add64(c_slots_referenced(device, w_reference_count[0]), u64_of(exempted_all[d])), u64_of(c_deferred_per_device[d])), SLOT_SHIFT);
        if (written_under_effective_floor && !(have_allocated && eq64(allocated, walked))) {
            violate(INVARIANT_I_3_1);
        }
        // I-5.2：空闲 + 已分配 = 单元区容量
        let slots = shr64(device_bytes(), SLOT_SHIFT);
        var free = vec2<u32>(0u, 0u);
        let have_free = c_accounting_row(2u, device, &free);
        var holds = have_free && have_allocated && le64(handoff_unit_area_start_slot(), slots) && !add64_overflows(free, allocated);
        if (holds) {
            holds = eq64(add64(free, allocated), shl64(sub64(slots, handoff_unit_area_start_slot()), SLOT_SHIFT));
        }
        if (!holds) {
            violate(INVARIANT_I_5_2);
        }
        // I-3.11：defer + 最新根走读（含隔离豁免） = 已分配
        var deferred = vec2<u32>(0u, 0u);
        let have_deferred = c_accounting_row(5u, device, &deferred);
        let referenced_by_newest = shl64(add64(c_slots_referenced(device, c_newest_reference_count), u64_of(c_exempted[d])), SLOT_SHIFT);
        var holds_defer = have_allocated && have_deferred && !add64_overflows(deferred, referenced_by_newest);
        if (holds_defer) {
            holds_defer = eq64(add64(deferred, referenced_by_newest), allocated);
        }
        if (!holds_defer) {
            violate(INVARIANT_I_3_11);
        }
    }
}

// ── I-1.8：扫描方向按已发布谓词过滤的码 1 / 码 3 单元归并成组
// 一个扫描项的归并键：码 1 是 [42, 101)；码 3 是 [42, 89) 与 [93, 107)
fn c_merge_key_equal(base_a: u32, base_b: u32) -> bool {
    let unit_class = rd8(base_a, 6u);
    if (rd8(base_b, 6u) != unit_class) {
        return false;
    }
    if (unit_class == 1u) {
        return bytes_equal(base_a, 42u, base_b, 42u, 59u);
    }
    return bytes_equal(base_a, 42u, base_b, 42u, 47u) && bytes_equal(base_a, 93u, base_b, 93u, 14u);
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
        sws(C_SCANNED + scanned, l);
        scanned = scanned + 1u;
    }
    for (var i = 0u; i < scanned; i = i + 1u) {
        let si = read_location(sw(C_SCANNED + i));
        // 更早的成员已经代表这一组
        var represented = false;
        for (var j = 0u; j < i; j = j + 1u) {
            if (c_merge_key_equal(si.base, read_location(sw(C_SCANNED + j)).base)) {
                represented = true;
            }
        }
        if (represented) {
            continue;
        }
        // ① 同一组的成员载荷相同
        var members = 0u;
        var every_same_checksum = true;
        let first_checksum = c_payload_checksum(si.base);
        for (var k = i; k < scanned; k = k + 1u) {
            let sk = read_location(sw(C_SCANNED + k));
            if (!c_merge_key_equal(si.base, sk.base)) {
                continue;
            }
            members = members + 1u;
            if (c_payload_checksum(sk.base) != first_checksum) {
                every_same_checksum = false;
            }
        }
        if (members >= 2u && !every_same_checksum) {
            // 可读的（载荷 CRC 在盘上对得上）成员的载荷校验和是不是不超过一种
            var readable_checksum = 0u;
            var have_readable = false;
            var more_than_one = false;
            for (var k = i; k < scanned; k = k + 1u) {
                let sk = read_location(sw(C_SCANNED + k));
                if (!c_merge_key_equal(si.base, sk.base) || (sk.c2 & 2u) == 0u) {
                    continue;
                }
                let checksum = c_payload_checksum(sk.base);
                if (!have_readable) {
                    readable_checksum = checksum;
                    have_readable = true;
                } else if (checksum != readable_checksum) {
                    more_than_one = true;
                }
            }
            if (more_than_one) {
                violate(INVARIANT_I_1_8);
            }
        }
        // ② 同 key 的各组两两全序键不等（只判码 3）
        if (rd8(si.base, 6u) != 3u) {
            continue;
        }
        var key_represented = false;
        for (var j = 0u; j < i; j = j + 1u) {
            let sj = read_location(sw(C_SCANNED + j));
            if (rd8(sj.base, 6u) != 3u || !bytes_equal(si.base, 42u, sj.base, 42u, 27u)) {
                continue;
            }
            // j 是不是自己那一组的代表
            var j_represented = false;
            for (var q = 0u; q < j; q = q + 1u) {
                if (c_merge_key_equal(sj.base, read_location(sw(C_SCANNED + q)).base)) {
                    j_represented = true;
                }
            }
            if (!j_represented) {
                key_represented = true;
            }
        }
        if (key_represented) {
            continue;
        }
        var representatives = 0u;
        var duplicate_total_order = false;
        for (var k = i; k < scanned; k = k + 1u) {
            let sk = read_location(sw(C_SCANNED + k));
            if (rd8(sk.base, 6u) != 3u || !bytes_equal(si.base, 42u, sk.base, 42u, 27u)) {
                continue;
            }
            var k_represented = false;
            for (var q = 0u; q < k; q = q + 1u) {
                if (c_merge_key_equal(sk.base, read_location(sw(C_SCANNED + q)).base)) {
                    k_represented = true;
                }
            }
            if (k_represented) {
                continue;
            }
            representatives = representatives + 1u;
            // 与更早的代表比全序键 (诞生代号, 实例, 事务号)
            for (var m = i; m < k; m = m + 1u) {
                let sm = read_location(sw(C_SCANNED + m));
                if (rd8(sm.base, 6u) != 3u || !bytes_equal(si.base, 42u, sm.base, 42u, 27u)) {
                    continue;
                }
                var m_represented = false;
                for (var q = 0u; q < m; q = q + 1u) {
                    if (c_merge_key_equal(sm.base, read_location(sw(C_SCANNED + q)).base)) {
                        m_represented = true;
                    }
                }
                if (m_represented) {
                    continue;
                }
                if (eq64(rd64(sm.base, 73u), rd64(sk.base, 73u)) && rd32(sm.base, 93u) == rd32(sk.base, 93u) && eq64(rd48(sm.base, 97u), rd48(sk.base, 97u))) {
                    duplicate_total_order = true;
                }
            }
        }
        if (representatives >= 2u && duplicate_total_order) {
            violate(INVARIANT_I_1_8);
        }
    }
}

// 第三批的入口
fn scan_stage(fsid_low: vec2<u32>) {
    judge_merged_version_total_order(fsid_low);
    judge_release_generation_and_tree_table_birth();
    judge_allocation_records_disjoint();
    judge_allocation_generations_against_unit_births();
    judge_rollback_floor_raises_against_their_ceilings();
    judge_accounting(fsid_low);
}

// ───────────────────────── 入口 ─────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    let out = local * 12u;
    if (sw(H_BASE + H_VALID) == 0u) {
        return;
    }
    violated_lo = verdicts[out + 8u];
    violated_hi = verdicts[out + 9u];
    handoff_load();
    let fsid_low = vec2<u32>(sw(H_BASE + H_FSID0), sw(H_BASE + H_FSID1));
    scan_stage(fsid_low);
    verdicts[out + 8u] = violated_lo;
    verdicts[out + 9u] = violated_hi;
    if (undecidable != 0u) {
        if (verdicts[out + 11u] == 0u) {
            verdicts[out + 11u] = undecidable;
        }
    } else {
        verdicts[out + 10u] = 5u;
    }
}
