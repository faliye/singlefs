// ── 多层码 2 树（walk_code_two_subtree）：schema 0 记账（I-1.10：叶 34、内部 108）、1 映射（只守走读要的字节：叶 55、内部 113）
// 记账的叶条目记进 R_ACCOUNTING（字节基、偏移、宽）；映射的叶一片一行记进 R_MAPPING_LEAVES（条目留在叶的字节里，条数不设上限；
// 记的叶都是这一遍新走过的单元，行数不过 C_MAX_VISITED）
struct Code2WalkFrame {
    n: Node,
    children_first: u32,
    child_index: u32,
    is_complete: bool,
    separator_base: u32,
    separator_offset: u32,
    parent_valid: bool,
    parent_device: u32,
    parent_slot: vec2<u32>,
};

var<private> code2_walk_frames: array<Code2WalkFrame, 12>;

// 孩子的结局：0 走过（带区间）、1 别的根走过、2 走不下去
fn w_push_child_result(kind: u32, smallest_base: u32, smallest_offset: u32, largest_base: u32, largest_offset: u32, separator_base: u32, separator_offset: u32) {
    if (w_child_count >= C_MAX_CHILDREN) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (10u << 8u);
        return;
    }
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS, kind);
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS + 1u, smallest_base);
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS + 2u, smallest_offset);
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS + 3u, largest_base);
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS + 4u, largest_offset);
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS + 5u, separator_base);
    sws(C_CHILDREN + w_child_count * C_CHILD_WORDS + 6u, separator_offset);
    w_child_count = w_child_count + 1u;
}

// 有父条目时头里的层级是父层级减一（I-1.1，walk.rs judge_the_level_the_parent_entry_names；根的 expected_level 是 NONE、不判）：交回成立与否。
// 第一次走到（不成立另记走读失败、不往下走）与别的版本先走过的节点（C896，只判）共用这一判
fn w_judge_the_level_the_parent_entry_names(n: Node, expected_level: u32) -> bool {
    if (expected_level == NONE) {
        return true;
    }
    w_judge(INVARIANT_I_1_1, n.level == expected_level);
    return n.level == expected_level;
}

// 多层码 2 树一个读回的节点往不往下走（walk.rs walk_code_two_subtree 读回节点之后那三步）：头里自述的 key 宽是这棵树的（不是的记走读失败）；
// 有父条目时层级是父层级减一（I-1.1，不是的记走读失败；根的 expected_level 是 NONE、不判）；条目宽在 I-1.10 射程里的按它判，不在的只守走读要的字节
// （不够的记走读失败）
fn w_code_two_node_is_walkable_under_its_parent_entry(n: Node, expected_level: u32, key_width: u32, registered: bool, leaf_width: u32,
    internal_width: u32) -> bool {
    if (n.key_width != key_width) {
        w_failures = w_failures + 1u;
        return false;
    }
    if (!w_judge_the_level_the_parent_entry_names(n, expected_level)) {
        w_failures = w_failures + 1u;
        return false;
    }
    var width = internal_width;
    if (n.level == 0u) {
        width = leaf_width;
    }
    if (registered) {
        w_judge(INVARIANT_I_1_10, n.entry_width == width);
        return n.entry_width == width;
    }
    if (n.entry_width < width) {
        w_failures = w_failures + 1u;
        return false;
    }
    return true;
}

struct Code2Walk {
    kind: u32,        // 根的结局：0 走过、1 别的根走过、2 走不下去
    is_complete: bool,
    entries: u32,
};

fn w_walk_code_two_tree(root_pointer: Ptr, tree: vec2<u32>, schema: u32, fsid_low: vec2<u32>) -> Code2Walk {
    var out: Code2Walk;
    out.kind = 2u;
    out.is_complete = false;
    out.entries = 0u;
    let key_width = schema_key_width(schema);
    var leaf_width = 34u;
    var internal_width = 108u;
    // 记账树（schema 0）、deadlist 树（5）与 logged_ops 树（6）的条目宽按 I-1.10 判（walk.rs 的 EntryWidthInTheWalk::JudgedAsInvariantOneTen；deadlist 叶 43、
    // 内部 43 + 86，判决 `research/prompts/m3-close-w13-code-r1-main-verification.md` 第二节 K5 那一行），中央映射树（1）只守走读要几个字节。
    // 走到这里的 schema 只有 0、1、5、6 四种（两个调用点），「不小于 deadlist 的编号」罩的就是 deadlist 与 logged_ops 两种；写成比大小、不再并一个等号，
    // 这个函数展开之后的表达式个数不变（crash_judge_dispatch.rs 的 each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget 钉着走读内核的展开大小）。
    // 以后再有种类走这条路，按它在不在 I-1.10 射程改这一判
    let registered = schema == 0u || schema >= KEY_SCHEMA_DEADLIST;
    // deadlist 树与 logged_ops 树的叶条目不记（条目在引用与快照那一截每一版各读各判），不占那两张表、不受记账那张表的条数上限
    let records_each_leaf_entry = schema == 0u;
    if (schema == 1u) {
        leaf_width = 55u;
        internal_width = 113u;
        mapping_leaf_count = 0u;
    } else if (schema == KEY_SCHEMA_DEADLIST) {
        leaf_width = DEADLIST_ENTRY_BYTES;
        internal_width = DEADLIST_ENTRY_BYTES + 86u;
    } else if (schema == KEY_SCHEMA_LOGGED_OPERATIONS) {
        leaf_width = LOGGED_OPERATIONS_ENTRY_BYTES;
        internal_width = LOGGED_OPERATIONS_KEY_BYTES + 86u;
    }
    var entries = 0u;
    w_child_count = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_expected_level = NONE;
    var pending_separator_base = 0u;
    var pending_separator_offset = 0u;
    var enter = true;
    // 根指针全零：根那一格不记走读失败（只有父条目里的子指针全零才记）
    if (root_pointer.all_zero) {
        return out;
    }
    loop {
        if (enter) {
            var result_kind = 2u;
            var result_smallest_base = 0u;
            var result_smallest_offset = 0u;
            var result_largest_base = 0u;
            var result_largest_offset = 0u;
            var result_complete = false;
            var descended = false;
            if (pending_pointer.all_zero && pending_expected_level != NONE) {
                w_failures = w_failures + 1u;
            } else {
                // 中央映射树自己的节点是自举豁免（按指针自己的位置读），记账树与 deadlist 树的节点按中央映射认位置
                let read = w_read_index_node(pending_pointer, tree, schema, 1u, registered,
                    select(MAPPING_CLASS_INDEX_NODE, MAPPING_CLASS_THE_POINTER_ITSELF, schema == 1u), fsid_low);
                if (!read.ok) {
                    if (read.already_walked) {
                        result_kind = 1u;
                        // C896：根之下的孩子别的版本先走过——这一条父条目定的层级照判（I-1.1；头里 key 宽不是这棵树的不判，同第一次走到时），
                        // 层级对得上的按「走过」带头里的区间交给父节点比 ② ③；都不往下走，不记走读失败、条目宽不重判，完整与否照旧不受它影响
                        // （叶条目不收）。层级对不上、key 宽不对的照旧按「别的根走过」交，② ③ 不比它（walk.rs SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot
                        // 带不带头里的区间）
                        if (pending_expected_level != NONE) {
                            let walked_earlier = w_index_node_an_earlier_walk_parsed();
                            if (walked_earlier.ok && walked_earlier.key_width == key_width
                                && w_judge_the_level_the_parent_entry_names(walked_earlier, pending_expected_level)) {
                                result_kind = 0u;
                                result_smallest_base = walked_earlier.base;
                                result_smallest_offset = walked_earlier.smallest_offset;
                                result_largest_base = walked_earlier.base;
                                result_largest_offset = walked_earlier.largest_offset;
                                result_complete = true;
                            }
                        }
                    }
                } else {
                    let n = read.n;
                    let walkable = w_code_two_node_is_walkable_under_its_parent_entry(n, pending_expected_level, key_width, registered,
                        leaf_width, internal_width);
                    if (walkable) {
                        if (n.level == 0u) {
                            if (schema == 1u) {
                                sws(R_MAPPING_LEAVES + mapping_leaf_count, n.base);
                                mapping_leaf_count = mapping_leaf_count + 1u;
                                entries = entries + n.entry_count;
                            }
                            for (var e = 0u; e < n.entry_count && records_each_leaf_entry; e = e + 1u) {
                                if (entries >= MAX_ACCOUNTING_ENTRIES) {
                                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (11u << 8u);
                                    break;
                                }
                                sws(R_ACCOUNTING + entries * 3u, n.base);
                                sws(R_ACCOUNTING + entries * 3u + 1u, entry_offset(n, e));
                                sws(R_ACCOUNTING + entries * 3u + 2u, n.entry_width);
                                entries = entries + 1u;
                            }
                            result_kind = 0u;
                            result_smallest_base = n.base;
                            result_smallest_offset = n.smallest_offset;
                            result_largest_base = n.base;
                            result_largest_offset = n.largest_offset;
                            result_complete = true;
                        } else {
                            if (depth >= MAX_FRAMES) {
                                undecidable = UNDECIDABLE_TOO_DEEP;
                                return out;
                            }
                            code2_walk_frames[depth].n = n;
                            code2_walk_frames[depth].children_first = w_child_count;
                            code2_walk_frames[depth].child_index = 0u;
                            code2_walk_frames[depth].is_complete = true;
                            code2_walk_frames[depth].separator_base = pending_separator_base;
                            code2_walk_frames[depth].separator_offset = pending_separator_offset;
                            code2_walk_frames[depth].parent_valid = w_parent_valid;
                            code2_walk_frames[depth].parent_device = w_parent_device;
                            code2_walk_frames[depth].parent_slot = w_parent_slot;
                            w_parent_valid = true;
                            w_parent_device = read.unit_device;
                            w_parent_slot = read.unit_slot;
                            depth = depth + 1u;
                            descended = true;
                        }
                    }
                }
            }
            if (descended) {
                enter = false;
                continue;
            }
            // 这个孩子（或根）走完了：交给父节点
            if (depth == 0u) {
                out.kind = result_kind;
                out.is_complete = result_complete;
                break;
            }
            w_push_child_result(result_kind, result_smallest_base, result_smallest_offset, result_largest_base, result_largest_offset, pending_separator_base, pending_separator_offset);
            if (result_kind == 2u || (result_kind == 0u && !result_complete)) {
                code2_walk_frames[depth - 1u].is_complete = false;
            }
            code2_walk_frames[depth - 1u].child_index = code2_walk_frames[depth - 1u].child_index + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = code2_walk_frames[top].n;
        if (code2_walk_frames[top].child_index >= n.entry_count) {
            // ② 两条不等式、③ 覆盖区间
            let first_child = code2_walk_frames[top].children_first;
            let child_count = n.entry_count;
            for (var c = 0u; c < child_count; c = c + 1u) {
                let row = C_CHILDREN + (first_child + c) * C_CHILD_WORDS;
                if (sw(row) != 0u) {
                    continue;
                }
                w_judge(INVARIANT_I_1_1, schema_compare(schema, sw(row + 5u), sw(row + 6u), sw(row + 1u), sw(row + 2u)) != 2u);
                if (c == 0u) {
                    continue;
                }
                let previous = C_CHILDREN + (first_child + c - 1u) * C_CHILD_WORDS;
                if (sw(previous) == 0u) {
                    w_judge(INVARIANT_I_1_1, schema_compare(schema, sw(row + 5u), sw(row + 6u), sw(previous + 3u), sw(previous + 4u)) == 2u);
                }
            }
            let first_row = C_CHILDREN + first_child * C_CHILD_WORDS;
            let last_row = C_CHILDREN + (first_child + child_count - 1u) * C_CHILD_WORDS;
            if (child_count > 0u && sw(first_row) == 0u && sw(last_row) == 0u) {
                let covers = bytes_equal(sw(first_row + 1u), sw(first_row + 2u), n.base, n.smallest_offset, key_width)
                    && bytes_equal(sw(last_row + 3u), sw(last_row + 4u), n.base, n.largest_offset, key_width);
                w_judge(INVARIANT_I_1_1, covers);
            }
            let complete = code2_walk_frames[top].is_complete;
            w_parent_valid = code2_walk_frames[top].parent_valid;
            w_parent_device = code2_walk_frames[top].parent_device;
            w_parent_slot = code2_walk_frames[top].parent_slot;
            w_child_count = first_child;
            depth = depth - 1u;
            if (depth == 0u) {
                out.kind = 0u;
                out.is_complete = complete;
                break;
            }
            w_push_child_result(0u, n.base, n.smallest_offset, n.base, n.largest_offset, code2_walk_frames[depth].separator_base, code2_walk_frames[depth].separator_offset);
            if (!complete) {
                code2_walk_frames[depth - 1u].is_complete = false;
            }
            code2_walk_frames[depth - 1u].child_index = code2_walk_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = code2_walk_frames[top].child_index;
        let separator_offset = entry_offset(n, e);
        pending_pointer = parse_node_pointer(n.base, separator_offset + key_width);
        w_judge_location_order(pending_pointer);
        pending_separator_base = n.base;
        pending_separator_offset = separator_offset;
        pending_expected_level = n.level - 1u;
        enter = true;
    }
    out.entries = entries;
    return out;
}

// walk_central_mapping_entries：I-2.5、I-1.11、I-2.1；映射条目指的单元记进引用集合（不判同一版第二次引用）。
// 条目逐片从 w_walk_code_two_tree 刚记下的映射叶里读（R_MAPPING_LEAVES，叶序、片里逐条）
fn w_walk_central_mapping_entries(fsid_low: vec2<u32>) {
    for (var leaf_index = 0u; leaf_index < mapping_leaf_count; leaf_index = leaf_index + 1u) {
        let leaf = mapping_leaf_at(leaf_index);
        let base = leaf.base;
        for (var entry_index = 0u; entry_index < leaf.entries.entry_count; entry_index = entry_index + 1u) {
            let o = leaf.entries.entries_start + entry_index * leaf.entries.entry_width;
            var p = zero_pointer();
            p.all_zero = all_zero(base, o + 27u, 28u);
            p.device0 = rd32(base, o + 27u);
            p.slot0 = rd48(base, o + 31u);
            p.sum0 = rd32(base, o + 37u);
            p.device1 = rd32(base, o + 41u);
            p.slot1 = rd48(base, o + 45u);
            p.sum1 = rd32(base, o + 51u);
            w_judge_location_order(p);
            let key_class = rd8(base, o);
            var unit_bytes = 0u;
            if (key_class == 1u || key_class == 3u) {
                unit_bytes = DATA_UNIT_BYTES;
            } else if (key_class == 2u) {
                unit_bytes = NODE_BYTES;
            } else {
                w_judge(INVARIANT_I_1_11, false);
                continue;
            }
            let span = unit_bytes >> SLOT_SHIFT;
            w_note_reference(p.device0, p.slot0, span);
            w_note_reference(p.device1, p.slot1, span);
            let s = w_read_referenced_unit(p, unit_bytes);
            if (s.version == NONE) {
                w_failures = w_failures + 1u;
                continue;
            }
            let header_class = rd8(s.base, 6u);
            w_judge(INVARIANT_I_1_11, header_class == key_class);
            if (header_class != key_class) {
                continue;
            }
            var birth_txg = vec2<u32>(0u, 0u);
            var instance = 0u;
            var tail = vec2<u32>(0u, 0u);
            var key_tail = vec2<u32>(0u, 0u);
            if (header_class == 1u) {
                birth_txg = rd64(s.base, 75u);
                instance = rd32(s.base, 91u);
                tail = rd48(s.base, 95u);
                key_tail = rd48(base, o + 21u);
            } else if (header_class == 2u) {
                let key_span = 2u * rd8(s.base, 51u);
                birth_txg = rd64(s.base, 52u + key_span);
                instance = rd32(s.base, 68u + key_span);
                tail = vec2<u32>(rd32(s.base, 72u + key_span), 0u);
                key_tail = vec2<u32>(rd32(base, o + 21u), 0u);
            } else {
                birth_txg = rd64(s.base, 73u);
                instance = rd32(s.base, 93u);
                tail = vec2<u32>(rd32(s.base, 103u), 0u);
                key_tail = vec2<u32>(rd32(base, o + 21u), 0u);
            }
            w_judge(INVARIANT_I_1_11, eq64(birth_txg, rd64(base, o + 9u)) && instance == rd32(base, o + 17u) && eq64(tail, key_tail));
            if (header_class != 1u) {
                w_judge(INVARIANT_I_1_11, rd16(base, o + 25u) == 0u);
            }
            if (header_class != 2u) {
                w_judge(INVARIANT_I_1_11, eq64(rd64(s.base, 43u), rd64(base, o + 1u)));
            }
        }
    }
}

// ── inode 树（walk_inode_root / walk_inode_internal_node / walk_inode_leaf_container）：根层级 1，孩子只接叶容器；类型段 0 而孩子不是别的一类（w_child_unit_class_other_than_the_entry_type_names）的这一版判不了
fn w_cached_leaves_below(device: u32, slot: vec2<u32>, into_from: u32) -> u32 {
    // 把父单元 (device, slot) 下面缓存的叶容器接进叶序（从 into_from 起追加），交回追加了几条
    var appended = 0u;
    let count = w_leaf_count;
    for (var i = 0u; i < count; i = i + 1u) {
        let row = C_LEAVES + i * C_LEAF_WORDS;
        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y) {
            if (w_leaf_count >= C_MAX_LEAVES) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (12u << 8u);
                return appended;
            }
            let target_row = C_LEAVES + w_leaf_count * C_LEAF_WORDS;
            sws(target_row, NONE);
            sws(target_row + 1u, into_from);
            sws(target_row + 2u, 0u);
            for (var k = 3u; k < C_LEAF_WORDS; k = k + 1u) {
                sws(target_row + k, sw(row + k));
            }
            w_leaf_count = w_leaf_count + 1u;
            appended = appended + 1u;
        }
    }
    return appended;
}

// I-9.12 的一行 (分隔 key, 孩子最小, 孩子最大)，按条目次序记进 C_SEPARATORS（walk.rs walk_inode_internal_node 的 separator_and_child_range）
fn w_note_the_separator_of_an_inode_child(separator: vec2<u32>, smallest: vec2<u32>, largest: vec2<u32>) {
    if (w_separator_count >= C_MAX_SEPARATORS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (15u << 8u);
        return;
    }
    let srow = C_SEPARATORS + w_separator_count * C_SEPARATOR_WORDS;
    sws(srow, separator.x);
    sws(srow + 1u, separator.y);
    sws(srow + 2u, smallest.x);
    sws(srow + 3u, smallest.y);
    sws(srow + 4u, largest.x);
    sws(srow + 5u, largest.y);
    w_separator_count = w_separator_count + 1u;
}

// 别的版本先走过的孩子：它的 key 区间取刚接进叶序的缓存那几格（叶序第 first_leaf 格起到表尾）里最小、最大的记录号，
// 与 walk.rs 命中那一支交回的缓存叶序同取；接进来的至少一格（调用方判过）
fn w_note_the_separator_of_an_inode_child_over_the_leaves_from(separator: vec2<u32>, first_leaf: u32) {
    let first_row = C_LEAVES + first_leaf * C_LEAF_WORDS;
    var smallest = vec2<u32>(sw(first_row + 5u), sw(first_row + 6u));
    var largest = vec2<u32>(sw(first_row + 7u), sw(first_row + 8u));
    for (var i = first_leaf + 1u; i < w_leaf_count; i = i + 1u) {
        let row = C_LEAVES + i * C_LEAF_WORDS;
        let leaf_smallest = vec2<u32>(sw(row + 5u), sw(row + 6u));
        let leaf_largest = vec2<u32>(sw(row + 7u), sw(row + 8u));
        if (lt64(leaf_smallest, smallest)) { smallest = leaf_smallest; }
        if (lt64(largest, leaf_largest)) { largest = leaf_largest; }
    }
    w_note_the_separator_of_an_inode_child(separator, smallest, largest);
}

fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, fsid_low: vec2<u32>) {
    if (root.level == 0u) {
        return;
    }
    if (w_context == 0u && !w_walking_the_trees_of_a_snapshot) {
        w_inode_tree_walked = true;
    }
    // 叶序：这一趟走出来的叶容器依次记进 C_LEAVES（父 = 根），供 I-9.4 第二、三句沿叶序比
    let leaves_before = w_leaf_count;
    w_separator_count = 0u;
    // 每条条目的孩子都到得了叶（读得出、类是类型段说的那一类、叶容器解得开且至少一条记录、记进叶序）：判根的子树覆盖区间要它
    // （walk.rs walk_inode_internal_node 的 EveryLeafReached）
    var every_leaf_reached = true;
    let parent_valid = w_parent_valid;
    let parent_device = w_parent_device;
    let parent_slot = w_parent_slot;
    w_parent_valid = true;
    w_parent_device = root_device;
    w_parent_slot = root_slot;
    for (var e = 0u; e < root.entry_count; e = e + 1u) {
        let o = entry_offset(root, e);
        let record_type = rd16(root.base, o + 16u);
        let identity_birth_tree = rd64(root.base, o + 8u);
        let identity_container = rd64(root.base, o + 18u);
        let identity_birth = rd64(root.base, o + 26u);
        var child = parse_node_pointer(root.base, o + 34u);
        w_judge_location_order(child);
        w_judge_pointer_head(child);
        w_judge_pointer_extent_offset(child);
        var length = NODE_BYTES;
        var span = 1u;
        if (record_type == 2u) {
            length = DATA_UNIT_BYTES;
            span = 2u;
        }
        // I-9.2 判孩子的类按孩子头里的类标签认、不按类型段定读多长（同墓碑树、谱系树那两条，walk.rs walk_inode_internal_node；判决
        // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行「inode 树 I-9.2 同形」）：孩子自证过的类与类型段说的不符
        // （类型段 2 ⇒ 码 3、类型段 0 ⇒ 码 2；类型段不认得的，孩子读得出就算），判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败
        var class_the_inode_entry_type_names = NONE;
        if (record_type == 2u) {
            class_the_inode_entry_type_names = 3u;
        } else if (record_type == 0u) {
            class_the_inode_entry_type_names = 2u;
        }
        // 孩子落在哪按类型段说的那一类查中央映射（walk.rs walk_inode_internal_node；类型段不认得的没有类、照提示）。上面 I-2.5 与指针头部那几格判的是指针自己的字节
        let mapping_class_of_the_inode_child = select(class_the_inode_entry_type_names, MAPPING_CLASS_THE_POINTER_ITSELF, class_the_inode_entry_type_names == NONE);
        child = w_locate(child, mapping_class_of_the_inode_child);
        var class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);
        // 认出另一类、另有一份头里就是类型段那一类且按那一类的长度自证得过（walk.rs walk_inode_internal_node 的 F-W，判决
        // `research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1 条）：I-9.2 照判红，再照类型段那一支往下走那一份（槽数按类型段那一类登记）
        if (class_of_the_inode_child != NONE && class_the_inode_entry_type_names != NONE
            && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, class_the_inode_entry_type_names)) {
            // 身份成立要孩子没有一份自证成别类：这里恒不成立
            w_judge(INVARIANT_I_9_2, class_of_the_inode_child == NONE);
            class_of_the_inode_child = NONE;
        }
        if (class_of_the_inode_child != NONE) {
            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_inode_child));
            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
            w_judge(INVARIANT_I_9_2, class_of_the_inode_child == class_the_inode_entry_type_names);
            every_leaf_reached = false;
            continue;
        }
        // 类型段不认得、上面也没认出孩子自证过的类：不按哪一种长度去读孩子（walk.rs walk_inode_internal_node 的 Unregistered 那一支，
        // 与墓碑树、谱系树类型段不认得那一支同判）——孩子按这一层定的形状登记引用（层级 1 下面是叶容器两槽，层级 ≥ 2 下面是码 2 节点一槽），
        // 判 I-9.2 红；每一份副本照判 I-2.1、两份都对不上记走读失败（H-J3a）
        if (class_the_inode_entry_type_names == NONE) {
            var child_span_by_the_level = 2u;
            if (root.level >= 2u) {
                child_span_by_the_level = 1u;
            }
            w_note_pointer_references(child, child_span_by_the_level);
            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
            w_judge(INVARIANT_I_9_2, false);
            every_leaf_reached = false;
            continue;
        }
        w_note_pointer_references_of_a_located_pointer(child, span, w_unit_of_a_located_pointer(child, mapping_class_of_the_inode_child),
            mapping_class_of_the_inode_child != MAPPING_CLASS_THE_POINTER_ITSELF);
        let s = w_read_unit_to_walk(child, length);
        if (s.version == NONE) {
            // 孩子两份都读不出：I-9.2 里不看孩子就判得了的那几小句照判（walk.rs judge_an_inode_entry_whose_child_does_not_read_back，
            // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段 2 判子指针头部的出生树 == 身份引用的出生树
            // （判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条），比孩子头的那几小句不判（类型段不认得的上面已经判完）
            if (record_type == 0u) {
                w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));
            }
            if (record_type == 2u) {
                w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));
            }
            w_failures = w_failures + 1u;
            every_leaf_reached = false;
            continue;
        }
        if (record_type == 2u) {
            let header_identity_holds = rd8(s.base, 6u) == 3u && eq64(rd64(s.base, 43u), identity_birth_tree) && rd16(s.base, 51u) == 2u
                && eq64(rd64(s.base, 53u), identity_container) && eq64(rd64(s.base, 61u), identity_birth) && eq64(child.birth_tree, identity_birth_tree);
            w_judge(INVARIANT_I_9_2, header_identity_holds);
            // 叶容器（走过、并进前一版、叶序、诞生代号都按读回那一份的落点认，F-KEY 与攻方 allrbmap）
            let leaf_device = w_unit_device_read_back;
            let leaf_slot = w_unit_slot_read_back;
            if (!w_visit_the_copy_read_back()) {
                w_note_a_unit_an_earlier_version_walked_referenced_again(s, child, leaf_device, leaf_slot,
                    reference_to_a_packed_container_of_a_tree(tree));
                if (w_context == 0u) {
                    // 别的版本先走过的叶：缓存里有它那一格才算到得了（walk.rs 缓存的那一枝 every_leaf_reached）。到得了的照判 I-9.12：
                    // 分隔 key 由这一版的父条目定，别的版本按它自己的父条目判过、证明不了这一版也写对了（C896 的口径，walk.rs
                    // walk_inode_internal_node 对命中的孩子同判，孩子的区间取缓存的叶序）
                    let cached_leaves_from = w_leaf_count;
                    if (w_cached_leaves_below(leaf_device, leaf_slot, leaves_before) == 0u) {
                        every_leaf_reached = false;
                    } else {
                        w_note_the_separator_of_an_inode_child_over_the_leaves_from(rd64(root.base, o), cached_leaves_from);
                    }
                }
                continue;
            }
            w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
            w_judge_birth_identity(s, child, leaf_device, leaf_slot);
            let packed = w_judge_packed_container(s, 2u, fsid_low);
            if (!packed.ok) {
                every_leaf_reached = false;
                continue;
            }
            w_note_the_header_the_walk_could_use();
            let p = packed.p;
            w_judge(INVARIANT_I_9_13, p.record_count > 0u);
            var have_range = false;
            var smallest = vec2<u32>(0u, 0u);
            var largest = vec2<u32>(0u, 0u);
            // 前一条记录的号（跨轮带着，不回头再读前一条记录）：r 为 0 时不读它
            var inode_of_the_previous_record = vec2<u32>(0u, 0u);
            for (var r = 0u; r < p.record_count; r = r + 1u) {
                let ro = packed_record_offset(p, r);
                let inode = rd64(p.base, ro);
                // I-9.4 第四句：同一片打包记录类型 2 容器里记录号沿记录序严格递增（判决 `research/prompts/m3-delete-checker-r2-main-verification.md`
                // 第二节 T11 取甲；walk.rs walk_inode_leaf_container 逐对判）
                if (r > 0u) {
                    w_judge(INVARIANT_I_9_4, lt64(inode_of_the_previous_record, inode));
                }
                inode_of_the_previous_record = inode;
                w_judge_reserved_bytes_and_the_truncation_field_of_an_inode_record(p.base, ro);
                w_judge(INVARIANT_I_9_4, le64(p.container, inode));
                let size = rd64(p.base, ro + INODE_RECORD_SIZE_OFFSET);
                let blocks = rd64(p.base, ro + 48u);
                let logical_blocks = shr64(add64_saturating(size, u64_of(511u)), 9u);
                w_judge(INVARIANT_I_9_15, eq64(blocks, logical_blocks));
                // I-9.26：对象出生代（记录偏移 INODE_RECORD_OBJECT_BIRTH_OFFSET）≤ 改动计数（偏移 88）≤ 这片容器头的诞生代号（偏移 73）
                let change_count = rd64(p.base, ro + 88u);
                let birth = rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                w_judge(INVARIANT_I_9_26, le64(birth, change_count) && le64(change_count, p.birth_txg));
                // inode → 对象出生代（I-9.10）与 inode 树内最大号（I-9.6）只记活头的树：快照里的记录是建快照那一刻的样子，同号可以已是别的对象
                if (w_context == 0u && !w_walking_the_trees_of_a_snapshot) {
                    // inode → 对象出生代（后写的覆盖先写的）
                    let birth_row_index = w_inode_birth_row_of(inode);
                    if (birth_row_index != NONE) {
                        sws(C_INODE_BIRTHS + birth_row_index * 4u + 2u, birth.x);
                        sws(C_INODE_BIRTHS + birth_row_index * 4u + 3u, birth.y);
                    } else if (w_inode_birth_count >= C_MAX_INODE_BIRTHS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (13u << 8u);
                    } else {
                        let appended = w_inode_birth_count;
                        sws(C_INODE_BIRTHS + appended * 4u, inode.x);
                        sws(C_INODE_BIRTHS + appended * 4u + 1u, inode.y);
                        sws(C_INODE_BIRTHS + appended * 4u + 2u, birth.x);
                        sws(C_INODE_BIRTHS + appended * 4u + 3u, birth.y);
                        w_inode_birth_count = appended + 1u;
                        w_inode_births_are_indexed = keyed_table_after_a_row_was_appended(w_inode_birth_table(), w_inode_birth_count,
                            w_inode_births_are_indexed, appended);
                    }
                    if (!w_have_largest_inode || lt64(w_largest_inode, inode)) {
                        w_largest_inode = inode;
                        w_have_largest_inode = true;
                    }
                }
                if (!have_range) {
                    smallest = inode;
                    largest = inode;
                    have_range = true;
                } else {
                    if (lt64(inode, smallest)) { smallest = inode; }
                    if (lt64(largest, inode)) { largest = inode; }
                }
            }
            if (have_range && w_context == 0u) {
                if (w_leaf_count >= C_MAX_LEAVES) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (14u << 8u);
                } else {
                    // 叶序里的一格，父 = 这片叶自己（别的版本再引用它时按父找）；这一趟的叶序按写入次序读
                    let row = C_LEAVES + w_leaf_count * C_LEAF_WORDS;
                    sws(row, leaf_device);
                    sws(row + 1u, leaf_slot.x);
                    sws(row + 2u, leaf_slot.y);
                    sws(row + 3u, p.container.x);
                    sws(row + 4u, p.container.y);
                    sws(row + 5u, smallest.x);
                    sws(row + 6u, smallest.y);
                    sws(row + 7u, largest.x);
                    sws(row + 8u, largest.y);
                    w_leaf_count = w_leaf_count + 1u;
                }
                w_note_the_separator_of_an_inode_child(rd64(root.base, o), smallest, largest);
            }
            if (!have_range) {
                every_leaf_reached = false;
            }
        } else {
            // 首次走到与别的版本先走过（walk.rs walk_inode_internal_child 命中时照判父层级减一，C896）两种都在下面交判不了
            // 类型段 0 的内部节点（类型段不认得的上面已经判完）：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了
            undecidable = UNDECIDABLE_TOO_DEEP;
            return;
        }
    }
    w_parent_valid = parent_valid;
    w_parent_device = parent_device;
    w_parent_slot = parent_slot;
    if (w_context != 0u) {
        return;
    }
    // I-9.12：分隔 key ≤ 孩子最小；严格递增；> 上一个孩子的最大
    for (var i = 0u; i < w_separator_count; i = i + 1u) {
        let row = C_SEPARATORS + i * C_SEPARATOR_WORDS;
        let separator = vec2<u32>(sw(row), sw(row + 1u));
        let smallest = vec2<u32>(sw(row + 2u), sw(row + 3u));
        w_judge(INVARIANT_I_9_12, le64(separator, smallest));
        if (i > 0u) {
            let previous = C_SEPARATORS + (i - 1u) * C_SEPARATOR_WORDS;
            let previous_separator = vec2<u32>(sw(previous), sw(previous + 1u));
            let previous_largest = vec2<u32>(sw(previous + 4u), sw(previous + 5u));
            w_judge(INVARIANT_I_9_12, lt64(previous_separator, separator));
            w_judge(INVARIANT_I_9_12, lt64(previous_largest, separator));
        }
    }
    // I-1.1 的 inode 树那一半（walk.rs judge_the_key_range_of_an_inode_internal_node_against_its_leaves，D18 已定项 2「区间取子树覆盖区间」）：
    // 根头里的 key 区间 = [叶序里最小的记录号, 最大的记录号]（这一趟根下面记进叶序的那几格，别的版本先走过的叶连缓存的一起）；
    // 叶没到全、零条目的根、自述 key 宽不是 8 的不判
    if (every_leaf_reached && root.entry_count > 0u && root.key_width == 8u && w_leaf_count > leaves_before) {
        var subtree_smallest = vec2<u32>(sw(C_LEAVES + leaves_before * C_LEAF_WORDS + 5u), sw(C_LEAVES + leaves_before * C_LEAF_WORDS + 6u));
        var subtree_largest = vec2<u32>(sw(C_LEAVES + leaves_before * C_LEAF_WORDS + 7u), sw(C_LEAVES + leaves_before * C_LEAF_WORDS + 8u));
        for (var i = leaves_before + 1u; i < w_leaf_count; i = i + 1u) {
            let row = C_LEAVES + i * C_LEAF_WORDS;
            let leaf_smallest = vec2<u32>(sw(row + 5u), sw(row + 6u));
            let leaf_largest = vec2<u32>(sw(row + 7u), sw(row + 8u));
            if (lt64(leaf_smallest, subtree_smallest)) { subtree_smallest = leaf_smallest; }
            if (lt64(subtree_largest, leaf_largest)) { subtree_largest = leaf_largest; }
        }
        w_judge(INVARIANT_I_1_1, eq64(rd64(root.base, root.smallest_offset), subtree_smallest) && eq64(rd64(root.base, root.largest_offset), subtree_largest));
    }
    // I-9.4 第二句：沿叶序相邻两片容器号严格递增（第三句「左容器的最大 key < 右容器号」随 N1 删掉，X4，
    // 左从右借之后右片保留身份，它的容器号可以不大于左片的最大 key；walk.rs judge_container_numbers_along_the_leaf_order 同改）
    for (var i = leaves_before + 1u; i < w_leaf_count; i = i + 1u) {
        let left = C_LEAVES + (i - 1u) * C_LEAF_WORDS;
        let right = C_LEAVES + i * C_LEAF_WORDS;
        let left_container = vec2<u32>(sw(left + 3u), sw(left + 4u));
        let right_container = vec2<u32>(sw(right + 3u), sw(right + 4u));
        w_judge(INVARIANT_I_9_4, lt64(left_container, right_container));
    }
}

// 前驱链上排在一条码 14 前面、不带销毁中记号的最近那一条（walk.rs 的 birth_txg_of_the_nearest_live_snapshot_among）：链按（诞生 txg, 树 ID）
// 升序；before_entry 是那条码 14 在树表里的下标，NONE 是整条链（找链上最后一条不带记号的，活头 inode 树那条的前驱比它）。
// 「排在前面」是（诞生 txg, 树 ID）严格小于那一条的：同键的几条里链上第一条之前的都严格更小（walk.rs 按键找到链上第一个同键的位置）
struct NearestLiveSnapshot {
    found: bool,
    birth_txg: vec2<u32>,
};

fn w_nearest_live_snapshot_before(tt: Node, before_entry: u32) -> NearestLiveSnapshot {
    var nearest: NearestLiveSnapshot;
    nearest.found = false;
    nearest.birth_txg = vec2<u32>(0u, 0u);
    var nearest_tree = vec2<u32>(0u, 0u);
    var limit_birth = vec2<u32>(0u, 0u);
    var limit_tree = vec2<u32>(0u, 0u);
    if (before_entry != NONE) {
        let limit_o = entry_offset(tt, before_entry);
        limit_birth = rd64(tt.base, limit_o + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET);
        limit_tree = rd64(tt.base, limit_o);
    }
    for (var other = 0u; other < tt.entry_count; other = other + 1u) {
        let other_o = entry_offset(tt, other);
        if (rd16(tt.base, other_o + 10u) != TREE_KIND_SNAPSHOT_INODE_ROOT
            || (rd16(tt.base, other_o + TREE_TABLE_ENTRY_FLAGS_OFFSET) & TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED) != 0u) {
            continue;
        }
        let birth = rd64(tt.base, other_o + TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET);
        let tree = rd64(tt.base, other_o);
        if (before_entry != NONE && !(lt64(birth, limit_birth) || (eq64(birth, limit_birth) && lt64(tree, limit_tree)))) {
            continue;
        }
        if (!nearest.found || lt64(nearest.birth_txg, birth) || (eq64(nearest.birth_txg, birth) && lt64(nearest_tree, tree))) {
            nearest.found = true;
            nearest.birth_txg = birth;
            nearest_tree = tree;
        }
    }
    return nearest;
}

// ── 树表的一条条目（walk_tree_table_entry）
fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
    w_walking_the_trees_of_a_snapshot = false;
    if (tt.entry_width < 200u) {
        w_failures = w_failures + 1u;
        return;
    }
    let o = entry_offset(tt, e);
    let tree = rd64(tt.base, o);
    let kind = rd16(tt.base, o + 10u);
    // I-7.18、I-7.19 认建账的发布：走最新根那一遍（上下文 0、记账照记）第一条记账树条目的根指针出生 txg（walk.rs walk_tree_table_entry 同记）
    if (kind == TREE_KIND_ACCOUNTING && w_context == 0u && w_accounting_recording && !w_newest_accounting_tree_root_birth_known) {
        let accounting_tree_root = parse_node_pointer(tt.base, o + 14u);
        if (!accounting_tree_root.all_zero) {
            w_newest_accounting_tree_root_birth_known = true;
            w_newest_accounting_tree_root_birth_txg = accounting_tree_root.birth_txg;
        }
    }
    // I-9.25：previous_snapshot_txg（条目偏移 100）只写在 inode 树那条与码 14 条目里，别的种类恒 0；inode 树那条不大于这一版的 txg。
    // 树表里有码 14 时按前驱链判（walk.rs judge_previous_snapshot_txg_of_the_tree_table_entries，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    // 第三节 S1、S4）：码 14 按（诞生 txg, 树 ID）排成链，带销毁中记号的那几条在链上被跳过——每条码 14 的前驱是排在它前面、不带记号的最近那一条的
    // 诞生 txg（没有是 0），inode 树那条是链上最后一条不带记号的那一条的诞生 txg。树表被几版共享时只在第一个走到它的那一版判（走过的单元不重走）
    let previous_snapshot_txg = rd64(tt.base, o + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET);
    if (kind == TREE_KIND_INODE) {
        let chain = w_nearest_live_snapshot_before(tt, NONE);
        w_judge(INVARIANT_I_9_25, le64(previous_snapshot_txg, w_version_txg) && (!chain.found || eq64(previous_snapshot_txg, chain.birth_txg)));
    } else if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
        let chain = w_nearest_live_snapshot_before(tt, e);
        w_judge(INVARIANT_I_9_25, eq64(previous_snapshot_txg, select(vec2<u32>(0u, 0u), chain.birth_txg, chain.found)));
    } else {
        w_judge(INVARIANT_I_9_25, is_zero64(previous_snapshot_txg));
    }
    if (w_context == 0u) {
        var known = false;
        for (var i = 0u; i < w_tree_id_count; i = i + 1u) {
            if (sw(C_TREE_IDS + i * 2u) == tree.x && sw(C_TREE_IDS + i * 2u + 1u) == tree.y) {
                known = true;
            }
        }
        if (!known) {
            if (w_tree_id_count >= C_MAX_TREE_IDS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (16u << 8u);
            } else {
                sws(C_TREE_IDS + w_tree_id_count * 2u, tree.x);
                sws(C_TREE_IDS + w_tree_id_count * 2u + 1u, tree.y);
                w_tree_id_count = w_tree_id_count + 1u;
            }
        }
    }
    // key 形态（lib.rs 的 key_schema_for_tree_kind）：1 extent、2 inode 与 deleted_inodes、3 分配记录、4 记账、5 映射、墓碑树与谱系树的容器号 [8]
    // 与 inode 同形（形态 2）；条目字段表宽（walk.rs 的 field_table_width_of_tree_kind）：deleted_inodes 8，墓碑树、谱系树 120；别的种类不走
    var schema = NONE;
    var registered = false;
    var field_table_width = 0u;
    // 只读快照的两种条目（walk.rs walk_the_tree_of_a_snapshot_entry）：码 14 照 inode 树往下走、码 15 照 extent 树往下走，与活头那两种同一处调用
    if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) { schema = 3u; registered = true; field_table_width = 112u; }
    else if (kind == TREE_KIND_INODE || kind == TREE_KIND_SNAPSHOT_INODE_ROOT) { schema = 2u; registered = true; field_table_width = 120u; }
    else if (kind == TREE_KIND_ALLOCATION) { schema = 4u; registered = true; field_table_width = 20u; }
    else if (kind == TREE_KIND_ACCOUNTING) { schema = 0u; registered = true; field_table_width = 34u; }
    else if (kind == 5u) { schema = 1u; }
    else if (kind == TREE_KIND_DELETED_INODES) { schema = 2u; registered = true; field_table_width = DELETED_INODES_ENTRY_BYTES; }
    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { schema = 2u; registered = true; field_table_width = CONTAINER_TREE_ENTRY_BYTES; }
    // deadlist 树（第三棵多层码 2 树，walk.rs walk_tree_table_entry 的 deadlist 那一支，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：
    // 叶 43、内部节点 key 43 + 子指针 86 按 I-1.10 判（w_walk_code_two_tree 的 registered）
    else if (kind == TREE_KIND_DEADLIST) { schema = KEY_SCHEMA_DEADLIST; }
    // logged_ops 树（第四棵多层码 2 树，walk.rs walk_tree_table_entry 的 logged_ops 那一支，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：
    // 叶 44、内部节点 key 10 + 子指针 86 按 I-1.10 判，走到的节点记进引用；条目（意图）在引用与快照那一截每一版各判
    else if (kind == TREE_KIND_LOGGED_OPERATIONS) { schema = KEY_SCHEMA_LOGGED_OPERATIONS; }
    // 没有 key 形态的条目走读走不下去：这一版的引用没收全在走到树表这一层时就记了（appearance_note_the_user_visible_roots_of_a_tree_table_walked_first）
    if (schema == NONE) {
        return;
    }
    let pointer = parse_node_pointer(tt.base, o + 14u);
    // 节点头里的树 ID 归哪棵树（I-1.3 的「实际引用它的树」）：活头的树是条目自己的树号；快照那两种是根指针头部的出生树（建快照那一刻活头那棵树的号，
    // 判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。带销毁中记号的码 14 与头 ID 配它的码 15 不走：从写意图那一刻起它们不再钉住块，
    // 删快照的批可以已经放掉、复用了它们的单元（D5（快照 / 空间记账机制） 已定项 20 的例外句）；根指针全零的不走
    var tree_the_nodes_belong_to = tree;
    if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
        if (pointer.all_zero || snapshot_tree_is_being_destroyed(tt, e)) {
            return;
        }
        tree_the_nodes_belong_to = pointer.birth_tree;
        w_walking_the_trees_of_a_snapshot = true;
    }
    if (kind == TREE_KIND_INODE && !pointer.all_zero) {
        let located_root = located_by_the_central_mapping(pointer, MAPPING_CLASS_INDEX_NODE);
        let s = read_at(located_root.device0, slot_to_bytes(located_root.slot0), NODE_BYTES);
        if (s.version != NONE) {
            w_judge(INVARIANT_I_9_1, rd8(s.base, 6u) == 2u);
        }
    }
    if (kind == TREE_KIND_ACCOUNTING || kind == TREE_KIND_DEADLIST || kind == TREE_KIND_LOGGED_OPERATIONS) {
        // 记账树、deadlist 树与 logged_ops 树同一个调用点：每个调用点各展开一份；后两棵走完就收（条目在引用与快照那一截每一版各判）
        let walked = w_walk_code_two_tree(pointer, tree, schema, fsid_low);
        if (kind != TREE_KIND_ACCOUNTING) {
            return;
        }
        // I-3.13：走全了的记账树里每一行的代段（key 偏移 14）等于这一版的 txg；inode 号水位那一统计量（标签 12）、不带设备维、
        // 代段早于这一版 txg 的行是台阶行，跳过（walk.rs walk_tree_table_entry 同改，D5（快照 / 空间记账机制） 已定项 2 射程）
        if (walked.kind == 0u && walked.is_complete) {
            for (var i = 0u; i < walked.entries; i = i + 1u) {
                let row_base = sw(R_ACCOUNTING + i * 3u);
                let row_offset = sw(R_ACCOUNTING + i * 3u + 1u);
                let row_generation = rd64(row_base, row_offset + 14u);
                let is_an_inode_number_watermark_step = rd16(row_base, row_offset) == 12u
                    && rd32(row_base, row_offset + 10u) == NONE
                    && lt64(row_generation, w_version_txg);
                if (is_an_inode_number_watermark_step) {
                    continue;
                }
                w_judge(INVARIANT_I_3_13, eq64(row_generation, w_version_txg));
            }
        }
        // 记账行只取最新根那一遍的（check_pool_image 在走最新根之后就把 accounting 拷走了，候选根的账不覆盖它）
        if (walked.kind == 0u && walked.is_complete && w_context == 0u && w_accounting_recording) {
            w_accounting_seen = true;
            for (var i = 0u; i < walked.entries; i = i + 1u) {
                let base = sw(R_ACCOUNTING + i * 3u);
                let ro = sw(R_ACCOUNTING + i * 3u + 1u);
                let statistic = rd16(base, ro);
                let device = rd32(base, ro + 10u);
                let value = rd64(base, ro + 22u);
                var found = false;
                for (var k = 0u; k < w_accounting_row_count; k = k + 1u) {
                    if (sw(C_ACCOUNTING_ROWS + k * 4u) == statistic && sw(C_ACCOUNTING_ROWS + k * 4u + 1u) == device) {
                        sws(C_ACCOUNTING_ROWS + k * 4u + 2u, value.x);
                        sws(C_ACCOUNTING_ROWS + k * 4u + 3u, value.y);
                        found = true;
                    }
                }
                if (!found) {
                    if (w_accounting_row_count >= C_MAX_ACCOUNTING_ROWS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (17u << 8u);
                    } else {
                        sws(C_ACCOUNTING_ROWS + w_accounting_row_count * 4u, statistic);
                        sws(C_ACCOUNTING_ROWS + w_accounting_row_count * 4u + 1u, device);
                        sws(C_ACCOUNTING_ROWS + w_accounting_row_count * 4u + 2u, value.x);
                        sws(C_ACCOUNTING_ROWS + w_accounting_row_count * 4u + 3u, value.y);
                        w_accounting_row_count = w_accounting_row_count + 1u;
                    }
                }
            }
        }
        return;
    }
    if (kind == TREE_KIND_ALLOCATION) {
        if (!pointer.all_zero) {
            w_walk_allocation_record_tree(pointer, tree, fsid_low);
        }
        return;
    }
    if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
        if (!pointer.all_zero) {
            w_walk_extent_tree(pointer, tree_the_nodes_belong_to, fsid_low);
        }
        return;
    }
    // 根头里的 key 区间怎么判（walk.rs 的 key_range_reading_of_a_tree_walked_from_its_tree_table_entry）：inode 树与快照 inode 根 4，
    // 墓碑树、谱系树、退役树 5，别的走到这里的（deleted_inodes）贴紧首末条目 0
    var key_range_reading_of_the_root = 0u;
    if (kind == TREE_KIND_INODE || kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
        key_range_reading_of_the_root = 4u;
    } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
        key_range_reading_of_the_root = 5u;
    }
    let read = w_read_index_node(pointer, tree_the_nodes_belong_to, schema, key_range_reading_of_the_root, registered, MAPPING_CLASS_INDEX_NODE, fsid_low);
    if (!read.ok) {
        return;
    }
    if (registered) {
        w_judge(INVARIANT_I_1_10, read.n.entry_width == field_table_width);
        if (read.n.entry_width != field_table_width) {
            return;
        }
    }
    if (kind == TREE_KIND_INODE || kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
        w_walk_inode_root(read.n, tree_the_nodes_belong_to, read.unit_device, read.unit_slot, fsid_low);
    }
    // 墓碑树、谱系树、退役树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}、退役 {0, 3}）；走到每片容器、判它的头与容器格式
    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
        var container_record_type = PACKED_TYPE_TOMBSTONE;
        if (kind == TREE_KIND_LINEAGE) {
            container_record_type = PACKED_TYPE_LINEAGE;
        } else if (kind == TREE_KIND_RETIREMENT) {
            container_record_type = PACKED_TYPE_RETIREMENT;
        }
        w_walk_container_tree_root(read.n, tree, read.unit_device, read.unit_slot, container_record_type, fsid_low);
    }
}

// ── 墓碑树或谱系树的码 2 根往下走（walk.rs 的 walk_container_tree_root 与 walk_the_entries_of_a_container_tree_node）
// 条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86）：类型段 = record_type 的走到码 3 容器（判容器头、出生树、身份、出生身份、容器格式，
// 单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点（层级 = 父层级 − 1、条目宽 120 才往下走；层级 ≤ 1 的节点里出现类型段 0 记走读失败），
// 别的类型段先登记引用（层级 1 的节点按码 3 容器两槽、层级 ≥ 2 的按码 2 节点一槽）再判该条目损坏、不往下走。每条条目的身份单独判定
// （墓碑树 I-9.23、谱系树 I-9.17），身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）：
// 类型段 0 判身份引用三段为 0 且孩子（读得出时）是码 2；类型段 = record_type 判容器头四段与身份引用逐字相等、子指针出生树相等、类标签 3。
// 孩子是哪一类按它头里的类标签认、不按类型段定读多长（w_child_unit_class_other_than_the_entry_type_names，判决
// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行：类型段 0 指着码 3 容器、类型段 = record_type 指着码 2 节点时，
// 按类型段的长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）：头里是另一类、按那一类的长度自证得过的，
// 判身份红、按它自己那一类占的槽登记引用（码 2 一槽、码 1 / 码 3 两槽）、不往下走、不记走读失败；另有一份按类型段那一类自证得过的，
// 判身份红之后照类型段走那一份（F-W，w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names）；别的照旧按类型段走。
// 往下记的落点都记在正在走的那个码 2 节点下面（w_parent_*）。walk.rs 是递归，这里是同一个先序的显式栈，
// 深度超过 w_container_tree_frames 的 12 层判不了；两个种类只经这一处调用点
struct WalkContainerTreeFrame {
    n: Node,
    next_entry: u32,
    device: u32,
    slot: vec2<u32>,
};

var<private> w_container_tree_frames: array<WalkContainerTreeFrame, 12>;

// 容器树层级 ≥ 2 的节点 parent 第 entry_index 条条目（类型段 0）的孩子码 2 节点 child，按这一条父条目判（walk.rs
// walk_the_internal_node_below_a_container_tree_entry）：父节点头里的 key 区间是子树覆盖区间（D18 已定项 2）——第一条条目的孩子头里的最小 key =
// 父头里的最小 key，最后一条的孩子头里的最大 key = 父头里的最大 key，父或孩子的 key 宽不是 8 时不比；孩子的层级是父层级减一（I-1.1）。
// 交回层级对不对（对了才往下走）
fn w_judge_a_container_tree_child_node_under_its_parent_entry(parent: Node, entry_index: u32, child: Node) -> bool {
    if (parent.key_width == CONTAINER_NUMBER_KEY_WIDTH && child.key_width == CONTAINER_NUMBER_KEY_WIDTH) {
        if (entry_index == 0u) {
            w_judge(INVARIANT_I_1_1, eq64(rd64(parent.base, parent.smallest_offset), rd64(child.base, child.smallest_offset)));
        }
        if (entry_index + 1u == parent.entry_count) {
            w_judge(INVARIANT_I_1_1, eq64(rd64(parent.base, parent.largest_offset), rd64(child.base, child.largest_offset)));
        }
    }
    w_judge(INVARIANT_I_1_1, child.level + 1u == parent.level);
    return child.level + 1u == parent.level;
}

// 一份落点上读回的单元按 unit_class 那一类的长度（码 2 16 KiB，码 1 / 码 3 32 KiB）算的整单元校验和对得上 checksum；别的类没有长度，恒不成立
fn w_unit_verifies_at_the_length_of_its_class(s: Slice, unit_class: u32, checksum: u32) -> bool {
    if (unit_class == 2u) {
        return slice_crc_matches(s, NODE_BYTES, checksum);
    }
    if (unit_class == 1u || unit_class == 3u) {
        return slice_crc_matches(s, DATA_UNIT_BYTES, checksum);
    }
    return false;
}

// 树里一条内部条目的孩子是不是条目类型段说的那一类（walk.rs 的 child_unit_class_other_than_the_entry_type_names）：只读孩子的单元头取类标签，
// 不按类型段定读多长。两条位置条目都看：头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码、且按那一类的长度读回的
// 整单元校验和对得上，这一份才算自证过的孩子；有一份自证过的是 class_the_entry_type_names 之外的一类，交回那一类（两份各自证成两个另一类时
// 交回第一份的）；自证过的都是类型段那一类、或哪一份都自证不过（读不出、类标签不在登记表里、校验和对不上：陈旧副本、坏副本），交回 NONE
// （照类型段那一支走，读不出照旧归 I-2.1）。不在第一份自证过的副本处停（判决 `research/prompts/m3-batch26-code-r1-main-verification.md`
// 第二节 K2 H1 H-V 那一行、第三节第 2 条：两份都自证得过、却是两类时，只看第一份的话结论随那一份在哪块盘翻面，walk.rs 同改）。
// class_the_entry_type_names 是 NONE（类型段不认得）时交回孩子自证过的那一类
fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> u32 {
    var class_other_than_the_entry_type_names = NONE;
    let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
    if (first_copy.version != NONE) {
        let class_in_the_first_header = rd8(first_copy.base, 6u);
        if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {
            if (class_in_the_first_header != class_the_entry_type_names) {
                class_other_than_the_entry_type_names = class_in_the_first_header;
            }
        }
    }
    let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
    if (second_copy.version != NONE) {
        let class_in_the_second_header = rd8(second_copy.base, 6u);
        if (w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {
            if (class_in_the_second_header != class_the_entry_type_names && class_other_than_the_entry_type_names == NONE) {
                class_other_than_the_entry_type_names = class_in_the_second_header;
            }
        }
    }
    return class_other_than_the_entry_type_names;
}

// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本头里的类标签（walk.rs 的 class_of_the_first_copy_verifying_at_the_length_of_its_class）：
// 头里的类标签是登记过的码、按那一类的长度读回的整单元校验和对得上；哪一份都没自证过交 NONE
fn w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child: Ptr) -> u32 {
    let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
    if (first_copy.version != NONE && w_unit_verifies_at_the_length_of_its_class(first_copy, rd8(first_copy.base, 6u), child.sum0)) {
        return rd8(first_copy.base, 6u);
    }
    let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
    if (second_copy.version != NONE && w_unit_verifies_at_the_length_of_its_class(second_copy, rd8(second_copy.base, 6u), child.sum1)) {
        return rd8(second_copy.base, 6u);
    }
    return NONE;
}

// w_child_unit_class_other_than_the_entry_type_names 认出另一类之后再问一句（walk.rs 的 some_copy_of_the_child_verifies_as_the_class_the_entry_type_names，
// 判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1 条，攻方改法 F-W）：有没有一份副本头里的类标签就是 class_the_entry_type_names、
// 按那一类的长度读回的整单元校验和对得上。有的话读者用的就是这一份，走读照类型段那一支走它的子树。只在认出另一类之后调
fn w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> bool {
    let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
    if (first_copy.version != NONE && rd8(first_copy.base, 6u) == class_the_entry_type_names
        && w_unit_verifies_at_the_length_of_its_class(first_copy, class_the_entry_type_names, child.sum0)) {
        return true;
    }
    let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
    return second_copy.version != NONE && rd8(second_copy.base, 6u) == class_the_entry_type_names
        && w_unit_verifies_at_the_length_of_its_class(second_copy, class_the_entry_type_names, child.sum1);
}

// 走读不往下读的孩子的一份副本（盘 device、槽 slot）与位置条目里的整单元校验和对不对得上（walk.rs 的 copy_matches_its_location_checksum_at_the_length_its_header_names）：
// 头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上
// 才算对不上（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）。比 32 KiB 时另按 32 KiB 读一次
// （CPU 那一边按每种长度各读一次）：没写过的槽 read_at 按 16 KiB 交回的全零单元只按它自己那一槽的长度用，下一槽有落点时不拿它的 32 KiB 检查字去比
// （判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 F-零槽那一行、第三节第 2 条，攻方改法 Z3）。
// 空段最后一槽那一读是拼读（read_at）：这里只比整段 CRC、不往下读字节，拼出的 CRC 照比（slice_whole_read_crc_matches，不记判不了；
// 判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条，正推说的「空槽加下一已写槽前 16 KiB」那一形）
fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(device: u32, slot: vec2<u32>, checksum: u32) -> bool {
    let copy = read_at(device, slot_to_bytes(slot), NODE_BYTES);
    if (copy.version == NONE) {
        return false;
    }
    let class_in_the_header = rd8(copy.base, 6u);
    if (class_in_the_header >= 1u && class_in_the_header <= 3u) {
        return w_unit_verifies_at_the_length_of_its_class(copy, class_in_the_header, checksum);
    }
    return slice_crc_matches(copy, NODE_BYTES, checksum)
        || slice_whole_read_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), checksum);
}

// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的）：每一份副本照判 I-2.1，
// 两份都对不上记走读失败（walk.rs 的 Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into，H-J3a、H-J3b）。不往下走，引用由调用方先登记
fn w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child: Ptr) {
    let first_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(child.device0, child.slot0, child.sum0);
    if (!first_matches) {
        w_mismatches = w_mismatches + 1u;
    }
    w_judge(INVARIANT_I_2_1, first_matches);
    let second_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(child.device1, child.slot1, child.sum1);
    if (!second_matches) {
        w_mismatches = w_mismatches + 1u;
    }
    w_judge(INVARIANT_I_2_1, second_matches);
    if (!first_matches && !second_matches) {
        w_failures = w_failures + 1u;
    }
}

// 一个单元按它的类占几槽：码 2 节点一槽，码 1 数据单元与码 3 打包记录单元两槽
fn w_slots_of_a_unit_of_class(unit_class: u32) -> u32 {
    if (unit_class == 2u) {
        return 1u;
    }
    return 2u;
}

fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, record_type: u32, fsid_low: vec2<u32>) {
    var identity_invariant = INVARIANT_I_9_23;
    if (record_type == PACKED_TYPE_LINEAGE) {
        identity_invariant = INVARIANT_I_9_17;
    } else if (record_type == PACKED_TYPE_RETIREMENT) {
        identity_invariant = INVARIANT_I_9_31;
    }
    let parent_valid = w_parent_valid;
    let parent_device = w_parent_device;
    let parent_slot = w_parent_slot;
    w_container_tree_frames[0].n = root;
    w_container_tree_frames[0].next_entry = 0u;
    w_container_tree_frames[0].device = root_device;
    w_container_tree_frames[0].slot = root_slot;
    var depth = 1u;
    w_parent_valid = true;
    w_parent_device = root_device;
    w_parent_slot = root_slot;
    loop {
        if (depth == 0u) {
            break;
        }
        let top = depth - 1u;
        let n = w_container_tree_frames[top].n;
        let e = w_container_tree_frames[top].next_entry;
        if (e >= n.entry_count) {
            depth = depth - 1u;
            if (depth > 0u) {
                w_parent_device = w_container_tree_frames[depth - 1u].device;
                w_parent_slot = w_container_tree_frames[depth - 1u].slot;
            }
            continue;
        }
        w_container_tree_frames[top].next_entry = e + 1u;
        let o = entry_offset(n, e);
        let entry_type = rd16(n.base, o + 16u);
        var child = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
        w_judge_location_order(child);
        w_judge_pointer_head(child);
        w_judge_pointer_extent_offset(child);
        // 孩子落在哪按类型段说的那一类查中央映射（walk.rs walk_the_entries_of_a_container_tree_node：类型段 0 是码 2 节点，= 记录类型是码 3 容器，
        // 不认得的没有类、照提示）；上面三格判的是指针自己的字节
        var mapping_class_of_the_container_tree_child = MAPPING_CLASS_THE_POINTER_ITSELF;
        if (entry_type == 0u) {
            mapping_class_of_the_container_tree_child = MAPPING_CLASS_INDEX_NODE;
        } else if (entry_type == record_type) {
            mapping_class_of_the_container_tree_child = MAPPING_CLASS_PACKED_RECORD_UNIT;
        }
        child = w_locate(child, mapping_class_of_the_container_tree_child);
        if (entry_type == 0u) {
            // 孩子头里是码 2 之外的一类、按那一类的长度自证得过：身份不符，按它自己那一类占的槽登记引用，不往下走、不记走读失败
            var class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);
            // 另有一份码 2、按 16 KiB 自证得过（walk.rs walk_the_entries_of_a_container_tree_node 的 F-W）：身份照判红，再照类型段 0 那一支往下走那一份
            if (class_of_the_child_of_an_internal_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 2u)) {
                w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == NONE);
                class_of_the_child_of_an_internal_entry = NONE;
            }
            if (class_of_the_child_of_an_internal_entry != NONE) {
                w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_an_internal_entry));
                w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
                w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == 2u);
                continue;
            }
            // 类型段 0：身份引用的出生树、容器号、出生代三段为 0；「子单元是码 2」与 I-1.6 重合、照 I-9.2 的先例留作合取，读不出的孩子这一小句不判。
            // 孩子的类只认按它自己那一类的长度整份自证过的副本（walk.rs 的 class_of_the_first_copy_verifying_at_the_length_of_its_class，
            // F-ID-b）；两份都没自证过的当读不出、这一小句不判
            let class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);
            let child_is_a_code_two_node_or_unreadable = class_of_the_child_of_a_type_zero_entry == NONE || class_of_the_child_of_a_type_zero_entry == 2u;
            w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u)) && is_zero64(rd64(n.base, o + 18u)) && is_zero64(rd64(n.base, o + 26u))
                && child_is_a_code_two_node_or_unreadable);
            if (n.level <= 1u) {
                // 孩子不往下走、记走读失败；每一份副本照判 I-2.1（walk.rs walk_the_internal_node_below_a_container_tree_entry 的 F-L0，
                // 判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 1 条），引用照旧不登记
                w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
                w_failures = w_failures + 1u;
                continue;
            }
            // 读孩子节点照读码 2 节点那一种读法：交的是条目里指针自己的字节（I-2.5 与头部几格照判、位置在里面按中央映射认，walk.rs 交 &entry[34..120]）
            let read = w_read_index_node(parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET), tree, 2u, 5u, true, MAPPING_CLASS_INDEX_NODE, fsid_low);
            if (!read.ok) {
                if (read.already_walked) {
                    // C896：别的版本先走过的孩子节点：这一条父条目定的子树覆盖与层级照判，不往下走
                    let walked_earlier = w_index_node_an_earlier_walk_parsed();
                    if (walked_earlier.ok) {
                        _ = w_judge_a_container_tree_child_node_under_its_parent_entry(n, e, walked_earlier);
                    }
                }
                continue;
            }
            if (w_judge_a_container_tree_child_node_under_its_parent_entry(n, e, read.n) && read.n.entry_width == CONTAINER_TREE_ENTRY_BYTES) {
                if (depth >= MAX_FRAMES) {
                    undecidable = UNDECIDABLE_TOO_DEEP;
                    break;
                }
                w_container_tree_frames[depth].n = read.n;
                w_container_tree_frames[depth].next_entry = 0u;
                w_container_tree_frames[depth].device = read.unit_device;
                w_container_tree_frames[depth].slot = read.unit_slot;
                depth = depth + 1u;
                w_parent_device = read.unit_device;
                w_parent_slot = read.unit_slot;
            }
            continue;
        }
        if (entry_type != record_type) {
            // 类型段不在 {0, record_type} 里：先登记引用（不登记的话这个孩子占的槽不进引用并集，I-3.1 连带红），孩子的形状由这一层定
            var span_of_the_unregistered_child = 2u;
            if (n.level >= 2u) {
                span_of_the_unregistered_child = 1u;
            }
            w_note_pointer_references(child, span_of_the_unregistered_child);
            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
            w_judge(identity_invariant, false);
            continue;
        }
        // 类型段 = record_type：孩子头里是码 3 之外的一类、按那一类的长度自证得过，身份不符（「类标签 3」那一小句），不按数据单元长度去读、不往下走
        var class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);
        // 另有一份码 3、按 32 KiB 自证得过（walk.rs walk_the_entries_of_a_container_tree_node 的 F-W）：身份照判红，再照这一支往下走那一份（两槽登记）
        if (class_of_the_child_of_a_container_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 3u)) {
            w_judge(identity_invariant, class_of_the_child_of_a_container_entry == NONE);
            class_of_the_child_of_a_container_entry = NONE;
        }
        if (class_of_the_child_of_a_container_entry != NONE) {
            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_a_container_entry));
            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
            w_judge(identity_invariant, class_of_the_child_of_a_container_entry == 3u);
            continue;
        }
        w_note_pointer_references_of_a_located_pointer(child, 2u, w_unit_of_a_located_pointer(child, MAPPING_CLASS_PACKED_RECORD_UNIT), true);
        let s = w_read_unit_to_walk(child, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            // 容器两份都读不出：子指针头部出生树 == 身份引用出生树只比父条目里的两段，照判（walk.rs walk_the_container_below_a_container_tree_entry，
            // 判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条）；比容器头的那几小句不判
            w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));
            w_failures = w_failures + 1u;
            continue;
        }
        // 条目里的身份引用（出生树、类型段、容器号、容器出生代）与容器头逐段相同、子指针出生树相同、类标签 3，在查这一遍走没走过之前判
        // （别的版本共享这片容器时也按这一版的条目判）
        let identity_holds = rd8(s.base, 6u) == 3u && eq64(rd64(s.base, 43u), rd64(n.base, o + 8u)) && rd16(s.base, 51u) == entry_type
            && eq64(rd64(s.base, 53u), rd64(n.base, o + 18u)) && eq64(rd64(s.base, 61u), rd64(n.base, o + 26u))
            && eq64(child.birth_tree, rd64(n.base, o + 8u));
        w_judge(identity_invariant, identity_holds);
        // 走过、并进前一版、诞生代号都按读回那一份的落点认（F-KEY 与攻方 allrbmap）
        let container_device = w_unit_device_read_back;
        let container_slot = w_unit_slot_read_back;
        if (!w_visit_the_copy_read_back()) {
            w_note_a_unit_an_earlier_version_walked_referenced_again(s, child, container_device, container_slot,
                reference_to_a_packed_container_of_a_tree(tree));
            continue;
        }
        w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
        w_judge_birth_identity(s, child, container_device, container_slot);
        // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 判；记录本身的不变量归 I-9.17 起那几条，走读不判
        if (w_judge_packed_container(s, record_type, fsid_low).ok) {
            w_note_the_header_the_walk_could_use();
        }
    }
    w_parent_valid = parent_valid;
    w_parent_device = parent_device;
    w_parent_slot = parent_slot;
}

// ── 一版的树表与中央映射树根（walk_tree_table_and_central_mapping_root）
// 树表别的版本先走过（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：条目不重走，这一版的用户可见树的根照先走那一版记下的取，
// 映射根照样往下走（共享的就并进这一版，I-3.5 比相邻两版的引用集合要它）；树表读不出、头用不了时映射根也不走
fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer: Ptr, fsid_low: vec2<u32>) {
    // 树表指针整条全零（walk.rs 的 follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core）：core 的根记录读者照两条位置条目
    // 读盘 0 槽 0、读不出。照 CPU 跟过去：两条位置条目照 I-2.1 判、这一版记一次走读失败，映射根不走；不判 I-2.5（整条全零的指针豁免）、
    // 不记引用。根环里的根、由记录施加出来的那几版、重放施加出来的那一版都经这里
    if (tree_table_pointer.all_zero) {
        _ = w_read_referenced_unit(tree_table_pointer, NODE_BYTES);
        w_failures = w_failures + 1u;
        return;
    }
    // 这一版的中央映射先只读读出来（walk.rs walk_tree_table_and_central_mapping_root）：下面走树表条目时，豁免三类之外的每条指针按它认位置；
    // 映射树本身的判定在这一版树表条目走完之后（下面 w_walk_code_two_tree 那一遍）
    read_the_central_mapping_without_judging(mapping_root_pointer);
    let read = w_read_index_node(tree_table_pointer, vec2<u32>(0u, 0u), 2u, 0u, false, MAPPING_CLASS_THE_POINTER_ITSELF, fsid_low);
    if (read.ok) {
        let tt = read.n;
        w_judge_tree_table_entries_ordering(tt);
        w_judge_tree_table_entry_head_identifiers(tt);
        if (w_context == 0u) {
            appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt, read.unit_device, read.unit_slot);
        }
        let parent_valid = w_parent_valid;
        let parent_device = w_parent_device;
        let parent_slot = w_parent_slot;
        w_parent_valid = true;
        w_parent_device = read.unit_device;
        w_parent_slot = read.unit_slot;
        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
            appearance_begin_tree_table_entry(tt, e);
            w_walk_tree_table_entry(tt, e, fsid_low);
            appearance_user_visible_tree_is_being_walked = false;
        }
        w_walking_the_trees_of_a_snapshot = false;
        w_parent_valid = parent_valid;
        w_parent_device = parent_device;
        w_parent_slot = parent_slot;
    } else if (read.already_walked) {
        if (w_context == 0u) {
            appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(read.unit_device, read.unit_slot);
        }
    } else {
        return;
    }
    _ = w_walk_code_two_tree(mapping_root_pointer, mapping_root_pointer.birth_tree, 1u, fsid_low);
    w_walk_central_mapping_entries(fsid_low);
}

// ── 一条根（walk_root）
// 走一遍：一条根（WALK_KIND_ROOT：实例表与分配记录树、树表与映射都取自这条根记录），或一版由记录施加出来、根槽从没落盘的版本
// （WALK_KIND_VERSION：四条指针都取自那条记录的新根段——施加一条记录 = 把所选根的七个字段换成记录里的，D23（journal 的角色与格式）
// 已定项 15「回到甲」；walk.rs 的 walk_version_applied_only_by_records 同一个取法）。
// 全内核只许有一处调它（walk_and_hand_off 的循环里）：显卡上的程序没有真的函数调用，驱动的编译器把每个调用点各展开一份，
// 这一整棵展开之后三十多万记号。原来五处走根、一处走版本各展开一份时，走读内核展开后 242 万记号，本机（驱动 595 版）冷编 523 秒、
// 常驻 4.1 GiB，第二台（驱动 580 版）在 48G 里编不完；只留一处之后本机 18 秒、0.8 GiB。
// 往这个内核里加走读的地方时，加一个阶段、不加调用点。
const WALK_KIND_ROOT: u32 = 0u;
const WALK_KIND_VERSION: u32 = 1u;

fn w_walk_one(kind: u32, record_base: u32, is_newest: bool, fsid_low: vec2<u32>) {
    w_begin_version();
    w_accounting_recording = is_newest;
    var tree_table_pointer_offset = 36u;
    var mapping_root_pointer_offset = 256u;
    var instance_table_pointer_offset = 170u;
    var allocation_root_pointer_offset = 342u;
    if (kind == WALK_KIND_VERSION) {
        tree_table_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        mapping_root_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86u;
        instance_table_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
        allocation_root_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
    }
    // I-3.5 取号跳过的第四条件读它（walk.rs instance_table_pointer_birth_txg_of）：这一版的实例表指针（码 2 指针 86 字节）非全零、
    // 指针头里的出生 txg（偏移 42，parse_pointer_common 的同一个偏移）等于这一版的 txg
    appearance_instance_table_pointer_is_born_with_the_version = !all_zero(record_base, instance_table_pointer_offset, 86u)
        && eq64(rd64(record_base, instance_table_pointer_offset + 42u), w_version_txg);
    // I-3.5 (b) 的谱系那一半：这一版走完时按这条树表指针读它的谱系（appearance_note_the_lineage_of_the_version_just_walked）
    appearance_tree_table_pointer_of_the_version = parse_node_pointer(record_base, tree_table_pointer_offset);
    w_walk_instance_table_chain(parse_node_pointer(record_base, instance_table_pointer_offset), is_newest, fsid_low);
    let allocation_root = parse_node_pointer(record_base, allocation_root_pointer_offset);
    if (!allocation_root.all_zero) {
        w_walk_allocation_record_tree(allocation_root, vec2<u32>(0u, 0u), fsid_low);
    }
    w_walk_tree_table_and_mapping(parse_node_pointer(record_base, tree_table_pointer_offset), parse_node_pointer(record_base, mapping_root_pointer_offset), fsid_low);
}
