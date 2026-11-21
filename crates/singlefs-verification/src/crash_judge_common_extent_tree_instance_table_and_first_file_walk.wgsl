// ───────────────────────── extent 树 ─────────────────────────

fn extent_upper_span(level: u32) -> vec2<u32> {
    var span = u64_of(EXTENT_UPPER_LEAF_INODES);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
    }
    return span;
}
fn extent_lower_span(level: u32) -> vec2<u32> {
    var span = u64_of(EXTENT_LOWER_LEAF_DATA_UNITS);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
    }
    return span;
}
fn mul64_64_saturating(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    if (b.y != 0u) {
        if (is_zero64(a)) {
            return a;
        }
        return vec2<u32>(NONE, NONE);
    }
    return mul64_32_saturating(a, b.x);
}
// extent key 的三段
fn extent_key_locality(base: u32, o: u32) -> vec2<u32> { return rd64(base, o); }
fn extent_key_inode(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 8u); }
fn extent_key_offset(base: u32, o: u32) -> vec2<u32> { return rd64(base, o + 16u); }
fn extent_key_equals(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>) -> bool {
    return is_zero64(rd64(base, o)) && eq64(rd64(base, o + 8u), inode) && eq64(rd64(base, o + 16u), offset);
}
// extent_tree 的 internal_entries：一个内部节点的每条子指针（条目 key 24 之后）头部加密、压缩那几段是第一版的取值、extent 偏移为 0
// （NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from）；
// core 在往下读第一个孩子之前就把整个节点的条目解完，有一条不是这个节点就损坏
fn extent_internal_child_pointer_heads_are_the_first_version(n: Node) -> bool {
    for (var e = 0u; e < n.entry_count; e = e + 1u) {
        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, entry_offset(n, e) + 24u)) {
            return false;
        }
        if (!pointer_extent_offset_is_zero(n.base, entry_offset(n, e) + 24u)) {
            return false;
        }
    }
    return true;
}
struct ExtentFrame {
    n: Node,
    level: u32,
    index: vec2<u32>,
    child_index: u32,
    has_previous_child: bool,
    previous_child_index: vec2<u32>,
};
var<private> extent_frames: array<ExtentFrame, 12>;

struct ExtentRead {
    ok: bool,
    node_count: u32,
    // 上段叶条目一共几条（read_upper_segment 交回；记进 R_UPPER 的只是从 batch_first 起的那一批）
    upper_entry_total: u32,
};
// 判一个 extent 树节点（extent_tree::ExtentTreeReading::read_and_judge_node，Every）；is_the_root_of_the_tree 说它是不是整棵树的根
// （上段的根，树表条目指着它；下段的根不算）
fn judge_extent_node(n: Node, tree: vec2<u32>, p: Ptr, root: RootFields, fsid_low: vec2<u32>, expected_level: u32, is_the_root_of_the_tree: bool,
    range_inode_first: vec2<u32>, range_offset_first: vec2<u32>, range_inode_last: vec2<u32>, range_offset_last: vec2<u32>) -> bool {
    // 失败点：解不开 UnitMalformed；头与引用、层级、key 区间、空节点 InvariantViolated（extent_tree::read_and_judge_node）
    if (!n.ok) {
        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
        return false;
    }
    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
    if (!root_header_checks(n, tree, 24u, p, root, fsid_low)) {
        return false;
    }
    if (n.level != expected_level) {
        return false;
    }
    if (!extent_key_equals(n.base, n.smallest_offset, range_inode_first, range_offset_first)
        || !extent_key_equals(n.base, n.largest_offset, range_inode_last, range_offset_last)) {
        return false;
    }
    // 零条目只许上段的根兼叶：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），与 read_and_judge_node 同一判
    if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {
        return false;
    }
    return true;
}
// 上段：按走读次序第 batch_first 条起的叶条目写进 R_UPPER（字节基、条目偏移），至多 MAX_UPPER_ENTRIES 条，upper_count 是这一批记了几条。
// 按块分批读、不设上限（与 R_DATA 那一段同一规则）：is_the_judging_pass 那一遍（batch_first 是 0）照 extent_tree::read_upper_segment
// 逐节点判、记见过、数全部叶条目，记满一批之后的条目照判、不记；后面几批是同一棵已判过的上段从根再走一遍，只按次序记下一批，不再记见过
// （同一个读法、同一份字节，判得过的一遍读不出别的结局）
fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>,
    is_the_judging_pass: bool, batch_first: u32) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    out.upper_entry_total = 0u;
    upper_count = 0u;
    var entry_ordinal = 0u;
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            // 段的根先读、先解出层级，再记「见过」（extent_tree::read_upper_segment / read_lower_segment 先 read_node、parse_index_node，
            // read_and_judge_node 才插 units_seen）；段里别的节点先记见过再读。记下一批的那几遍不记见过（判的那一遍记过了）
            let is_the_root_of_the_segment = depth == 0u;
            if (is_the_judging_pass && !is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (is_the_judging_pass && is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let span = extent_upper_span(level);
            let first_inode = mul64_64_saturating(pending_index, span);
            let last_inode = add64_saturating(first_inode, sub64(span, u64_of(1u)));
            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, depth == 0u, first_inode, vec2<u32>(0u, 0u), last_inode, vec2<u32>(NONE, NONE))) {
                return out;
            }
            nodes = nodes + 1u;
            if (level == 0u) {
                // 条目宽只在有条目时核：extent_tree.rs 的 upper_leaf_entries 逐条 ExtentUpperLeafEntry::parse 时才核宽，
                // 零条目的上段根（第一个文件删完之后）条目宽是什么都不拒；有条目时宽不等于 113 就拒，宽了的同样拒（ExtentUpperLeafEntry::parse，I-1.10 写「等于」）
                if (n.entry_count > 0u && n.entry_width != EXTENT_UPPER_LEAF_ENTRY_BYTES) {
                    walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                    return out;
                }
                var previous_inode = vec2<u32>(0u, 0u);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    // 失败点（extent_tree::upper_leaf_entries）：标签不认识、字段表写零的字节不是零、不在叶罩的那一段、次序 InvariantViolated；
                    // 指针头部 UnitMalformed
                    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                    if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
                        return out;
                    }
                    let inode = extent_key_inode(n.base, o);
                    let tag = rd8(n.base, o + 24u);
                    if (tag == 0u) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return out;
                        }
                    } else if (tag == 1u || tag == 3u) {
                        // 标签 3 的载荷与标签 1 同一个字段表（ExtentUpperLeafEntry::parse 换成没有洞的标签照同一段解，判决 m4-trunc-r1 第三节 K5），
                        // 下面几判同 1；4 同 2
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return out;
                        }
                        // ExtentUpperLeafEntry::parse：下段根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏
                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                        // ExtentUpperLeafEntry::parse：下段根指针头部 extent 偏移非 0，这条条目损坏
                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                    } else if (tag == 2u || tag == 4u) {
                        // 同上：内联的数据指针（加密、压缩那几段与 extent 偏移各判一次）
                        if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                        // 同上：内联的数据指针
                        if (!pointer_extent_offset_is_zero(n.base, o + 25u)) {
                            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                            return out;
                        }
                    } else {
                        return out;
                    }
                    if (lt64(inode, first_inode) || lt64(last_inode, inode)) {
                        return out;
                    }
                    if (e > 0u && !lt64(previous_inode, inode)) {
                        return out;
                    }
                    previous_inode = inode;
                    // 这一批从第 batch_first 条起、至多 MAX_UPPER_ENTRIES 条；这一批之外的照判、照数，不记
                    if (entry_ordinal >= batch_first && upper_count < MAX_UPPER_ENTRIES) {
                        sws(R_UPPER + upper_count * 2u, n.base);
                        sws(R_UPPER + upper_count * 2u + 1u, o);
                        upper_count = upper_count + 1u;
                    }
                    entry_ordinal = entry_ordinal + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            // extent_tree::internal_entries：条目宽不等于 110（窄了宽了都算，C867）EntryWidthDiffersFromItsFieldTable，再逐条判子指针头部 UnitMalformed，都在读任何一个孩子之前
            if (n.entry_width != 110u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (!extent_internal_child_pointer_heads_are_the_first_version(n)) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            extent_frames[depth].n = n;
            extent_frames[depth].level = level;
            extent_frames[depth].index = pending_index;
            extent_frames[depth].child_index = 0u;
            extent_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_frames[top].n;
        if (extent_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_frames[top].child_index;
        let o = entry_offset(n, e);
        let child_level = extent_frames[top].level - 1u;
        let child_span = extent_upper_span(child_level);
        let first_inode = extent_key_inode(n.base, o);
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
            return out;
        }
        let qr = divmod64(first_inode, child_span);
        if (!is_zero64(qr.remainder)) {
            return out;
        }
        let child_index = qr.quotient;
        // child.parent() == parent：index / 147
        let parent_index = divmod64(child_index, u64_of(EXTENT_INTERNAL_FANOUT)).quotient;
        if (!eq64(parent_index, extent_frames[top].index)) {
            return out;
        }
        if (extent_frames[top].has_previous_child && !lt64(extent_frames[top].previous_child_index, child_index)) {
            return out;
        }
        extent_frames[top].has_previous_child = true;
        extent_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 24u;
        pending_level = child_level;
        pending_index = child_index;
        enter = true;
    }
    out.node_count = nodes;
    out.upper_entry_total = entry_ordinal;
    return out;
}
// 下段：数一个文件的每一个数据单元（units_of_the_file_being_read），是第一个文件的下段时头一块的数据指针写进 R_DATA（字节基、指针偏移），
// 按单元号升序；单元号要从 0 起连号由调用方判（读到的单元号与它在这个文件里的位置不同就记进
// units_of_the_file_being_read_are_numbered_from_zero_without_a_gap）
fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, inode: vec2<u32>, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>,
    is_the_first_file: bool) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    out.upper_entry_total = 0u;
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    loop {
        if (enter) {
            // 段的根先读、先解出层级，再记「见过」（extent_tree::read_upper_segment / read_lower_segment 先 read_node、parse_index_node，
            // read_and_judge_node 才插 units_seen）；段里别的节点先记见过再读
            let is_the_root_of_the_segment = depth == 0u;
            if (!is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (is_the_root_of_the_segment && !seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
            }
            let span = extent_lower_span(level);
            let first_unit = mul64_64_saturating(pending_index, span);
            let last_unit = add64_saturating(first_unit, sub64(span, u64_of(1u)));
            let first_offset = mul64_64_saturating(first_unit, payload);
            let last_offset = sub64_saturating(mul64_64_saturating(add64_saturating(last_unit, u64_of(1u)), payload), u64_of(1u));
            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, false, inode, first_offset, inode, last_offset)) {
                return out;
            }
            nodes = nodes + 1u;
            if (level == 0u) {
                // extent 叶记录宽不等于 112 就拒，宽了的同样拒（records.rs parse_extent_record，C867）
                if (n.entry_width != 112u) {
                    walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                    return out;
                }
                var previous_unit = vec2<u32>(0u, 0u);
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    // records::parse_extent_record 先判数据指针头部（UnitMalformed），read_lower_node_and_its_subtree 再判 key、区间与次序（InvariantViolated）
                    // records::parse_extent_record：数据指针头部加密、压缩那几段不是第一版的取值，这条叶记录损坏
                    if (!pointer_encryption_and_compression_fields_are_the_first_version(n.base, o + 24u)) {
                        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                        return out;
                    }
                    // records::parse_extent_record：数据指针头部 extent 偏移非 0，这条叶记录损坏
                    if (!pointer_extent_offset_is_zero(n.base, o + 24u)) {
                        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                        return out;
                    }
                    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                    let offset = extent_key_offset(n.base, o);
                    let qr = divmod64(offset, payload);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode) || !is_zero64(qr.remainder)) {
                        return out;
                    }
                    let unit = qr.quotient;
                    if (lt64(unit, first_unit) || lt64(last_unit, unit)) {
                        return out;
                    }
                    if (e > 0u && !lt64(previous_unit, unit)) {
                        return out;
                    }
                    previous_unit = unit;
                    if (is_the_first_file && units_of_the_file_being_read < DATA_POINTERS_PER_BLOCK) {
                        sws(R_DATA + units_of_the_file_being_read * 2u, n.base);
                        sws(R_DATA + units_of_the_file_being_read * 2u + 1u, o + 24u);
                    }
                    // 单元号要等于它在整个文件里的位置（extents_of_the_first_file 的「没有洞」，第一个文件与别的文件同一条）：这里记下来，调用方判
                    if (!eq64(unit, u64_of(units_of_the_file_being_read))) {
                        units_of_the_file_being_read_are_numbered_from_zero_without_a_gap = false;
                    }
                    units_of_the_file_being_read = units_of_the_file_being_read + 1u;
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            // extent_tree::internal_entries：条目宽不等于 110（窄了宽了都算，C867）EntryWidthDiffersFromItsFieldTable，再逐条判子指针头部 UnitMalformed，都在读任何一个孩子之前
            if (n.entry_width != 110u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (!extent_internal_child_pointer_heads_are_the_first_version(n)) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            extent_frames[depth].n = n;
            extent_frames[depth].level = level;
            extent_frames[depth].index = pending_index;
            extent_frames[depth].child_index = 0u;
            extent_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = extent_frames[top].n;
        if (extent_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            extent_frames[depth - 1u].child_index = extent_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = extent_frames[top].child_index;
        let o = entry_offset(n, e);
        let level = extent_frames[top].level;
        let child_span = extent_lower_span(level - 1u);
        let child_offset_span = mul64_64_saturating(child_span, payload);
        let offset = extent_key_offset(n.base, o);
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (!is_zero64(extent_key_locality(n.base, o)) || !eq64(extent_key_inode(n.base, o), inode)) {
            return out;
        }
        let qr = divmod64(offset, child_offset_span);
        if (!is_zero64(qr.remainder)) {
            return out;
        }
        let child_index = qr.quotient;
        // child.data_unit_range().0 / span(level) == index
        let child_first_unit = mul64_64_saturating(child_index, child_span);
        let parent_index = divmod64(child_first_unit, extent_lower_span(level)).quotient;
        if (!eq64(parent_index, extent_frames[top].index)) {
            return out;
        }
        if (extent_frames[top].has_previous_child && !lt64(extent_frames[top].previous_child_index, child_index)) {
            return out;
        }
        extent_frames[top].has_previous_child = true;
        extent_frames[top].previous_child_index = child_index;
        pending_pointer = parse_node_pointer(n.base, o + 24u);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 24u;
        pending_level = level - 1u;
        pending_index = child_index;
        enter = true;
    }
    out.node_count = nodes;
    return out;
}
// 整棵 extent 树 + extents_of_the_first_file：上段整段（按块分批记叶条目），再按每条叶条目读那个文件的下段。第一个文件之外有数据单元的文件
// （按 inode 号写数据写出来的）与 CPU 恢复同一套收法（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M6）：
// 至少一个单元、单元从 0 起连号、尾部没有洞（标签 1、2）；它们的单元数并进映射条目数（recovery 的 data_units_of_every_file），
// 冷走读不读它们的数据单元、不找它们的记录（recovery.rs walk_to_file 同）
fn read_extent_tree_of_the_first_file(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> ExtentRead {
    var out: ExtentRead;
    out.ok = false;
    out.node_count = 0u;
    out.upper_entry_total = 0u;
    seen_count = 0u;
    data_count = 0u;
    files_of_the_extent_tree = 0u;
    data_units_of_the_files_other_than_the_first = 0u;
    data_units_are_numbered_from_zero_without_a_gap = true;
    first_file_upper_leaf_tag = NONE;
    first_file_upper_entry_base = NONE;
    first_file_upper_entry_offset = 0u;
    // 第一个文件之外的文件有一个没有单元、单元有洞或尾部有洞（recovery::extents_of_the_first_file 在读完整棵树之后才判，读的失败点先报）
    var files_other_than_the_first_are_well_formed = true;
    var nodes = 0u;
    var upper_entry_total = 0u;
    var batch_first = 0u;
    // 迭代上界：每一批至少记一条叶条目（记不到就判不了、交回），批数不过叶条目数 ÷ MAX_UPPER_ENTRIES 向上取整；跨轮带着的是下一批从第几条起。
    // 上段与下段各一个调用点（GPU 程序把每个调用点展开成一份拷贝）
    loop {
        let is_the_judging_pass = batch_first == 0u;
        let upper = read_upper_segment(root_pointer, root_pointer_base, root_pointer_offset, tree, root, fsid_low, is_the_judging_pass, batch_first);
        if (!upper.ok) {
            return out;
        }
        if (is_the_judging_pass) {
            nodes = upper.node_count;
            upper_entry_total = upper.upper_entry_total;
        }
        if (upper_count == 0u && batch_first < upper_entry_total) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
            return out;
        }
        for (var i = 0u; i < upper_count; i = i + 1u) {
            let base = sw(R_UPPER + i * 2u);
            let o = sw(R_UPPER + i * 2u + 1u);
            let inode = extent_key_inode(base, o);
            let tag = rd8(base, o + 24u);
            let is_the_first_file = eq64(inode, u64_of(FIRST_INODE_NUMBER));
            // extent_tree::read_extent_tree：标签 1、3 读下段，2、4 是内联的那一个单元（3、4 之后到 size 是洞，判决 m4-trunc-r1 第三节 K5）；标签 0 一个单元都没有
            let names_a_lower_segment_root = tag == 1u || tag == 3u;
            let names_an_inline_data_unit = tag == 2u || tag == 4u;
            units_of_the_file_being_read = 0u;
            units_of_the_file_being_read_are_numbered_from_zero_without_a_gap = true;
            if (names_a_lower_segment_root) {
                let lower_root = parse_node_pointer(base, o + 25u);
                let lower = read_lower_segment(lower_root, base, o + 25u, inode, tree, root, fsid_low, is_the_first_file);
                if (!lower.ok) {
                    return out;
                }
                nodes = nodes + lower.node_count;
            } else if (names_an_inline_data_unit) {
                if (is_the_first_file) {
                    sws(R_DATA, base);
                    sws(R_DATA + 1u, o + 25u);
                }
                units_of_the_file_being_read = 1u;
            }
            // 建上一版那一样读者要每个文件的单元：上段叶条目在哪、标签、单元数记一行（按 inode 号升序，下段读完才记，单元数是数全的）
            if (extent_tree_records_its_files) {
                if (files_of_the_extent_tree >= MAX_FILES_OF_THE_EXTENT_TREE) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                    return out;
                }
                let row = R_FILES + files_of_the_extent_tree * FILE_ROW_WORDS;
                sws(row, inode.x);
                sws(row + 1u, inode.y);
                sws(row + 2u, base);
                sws(row + 3u, o);
                sws(row + 4u, tag);
                sws(row + 5u, units_of_the_file_being_read);
                sws(row + 6u, 0u);
                files_of_the_extent_tree = files_of_the_extent_tree + 1u;
            }
            if (is_the_first_file) {
                first_file_upper_leaf_tag = tag;
                first_file_upper_entry_base = base;
                first_file_upper_entry_offset = o;
                data_count = units_of_the_file_being_read;
                data_units_are_numbered_from_zero_without_a_gap = units_of_the_file_being_read_are_numbered_from_zero_without_a_gap;
            } else {
                data_units_of_the_files_other_than_the_first = data_units_of_the_files_other_than_the_first + units_of_the_file_being_read;
                // 第一个文件之外的文件与第一个文件同一读法（判决 m4-multi-inode-size-r1 第二节 S2，recovery::extents_of_the_first_file）：
                // 标签 0 一个单元都没有（粘住的标签 0）、标签 3、4 尾部是洞，都收；只拒单元中间有洞（从 0 起不连号）
                if (!units_of_the_file_being_read_are_numbered_from_zero_without_a_gap) {
                    files_other_than_the_first_are_well_formed = false;
                }
            }
        }
        batch_first = batch_first + upper_count;
        if (batch_first >= upper_entry_total) {
            break;
        }
    }
    data_block_first = 0u;
    data_block_count = min(data_count, DATA_POINTERS_PER_BLOCK);
    // recovery::extents_of_the_first_file：哪个文件的单元中间有洞都是 UnitMalformed
    walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
    if (!files_other_than_the_first_are_well_formed) {
        return out;
    }
    // 上段里没有第一个文件那一条：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），或删完之后只有别的文件有条目，
    // 第一个文件没有数据单元、没有下段，读得出（同 recovery.rs extents_of_the_first_file 那一支）；结局由后面 inode 树里第一个文件那条记录定
    if (first_file_upper_leaf_tag == NONE) {
        out.ok = true;
        out.node_count = nodes;
        return out;
    }
    // 标签 0 配 size > 0 是整个文件都是洞（判决 m4-trunc-r1 第三节 K5、第六节 S1），一个单元都没有也读得出；别的标签至少一个单元
    if (data_count == 0u && first_file_upper_leaf_tag != 0u) {
        return out;
    }
    if (!data_units_are_numbered_from_zero_without_a_gap) {
        return out;
    }
    out.ok = true;
    out.node_count = nodes;
    return out;
}
// 找下一块数据指针时往下走的一步（walk_to_file 读了下段里一个节点交给它）：is_the_block_found 为真时 R_DATA 换成下一块；
// 否则 pointer_base / pointer_offset 是下一层要读的那条子指针在哪
struct StepTowardTheNextBlockOfDataPointers {
    is_the_block_found: bool,
    pointer_base: u32,
    pointer_offset: u32,
};
// 第一个文件的第 unit 个单元在下段的哪片叶里：节点 read 是从下段根往下读回的一层。内部节点取子区间起点不超过 unit 的最后一条条目往下；
// 叶里从 unit 那一条起、到叶末尾的数据指针记成下一块（叶不过 EXTENT_LOWER_LEAF_DATA_UNITS 条，装得进一块）。
// 这几层在 extent 树那一遍都读过、核过（read_lower_segment：键升序、子区间对得上父区间、单元从 0 起连号、没有零条目的下段节点），
// 这里不再判，只认路；读回的不是那一形记判不了（UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN）
fn find_the_next_block_of_data_pointers(read: MappedRead, unit: u32) -> StepTowardTheNextBlockOfDataPointers {
    var step: StepTowardTheNextBlockOfDataPointers;
    step.is_the_block_found = false;
    step.pointer_base = NONE;
    step.pointer_offset = 0u;
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    let n = parse_index_node(read.s);
    if (read.failed || !n.ok || n.entry_count == 0u) {
        undecidable = UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN;
        return step;
    }
    if (n.level > 0u) {
        var chosen = 0u;
        for (var e = 1u; e < n.entry_count; e = e + 1u) {
            let first_unit_of_the_child = divmod64(extent_key_offset(n.base, entry_offset(n, e)), payload).quotient;
            if (lt64(u64_of(unit), first_unit_of_the_child)) {
                break;
            }
            chosen = e;
        }
        step.pointer_base = n.base;
        step.pointer_offset = entry_offset(n, chosen) + 24u;
        return step;
    }
    let first_unit_of_the_leaf = divmod64(extent_key_offset(n.base, entry_offset(n, 0u)), payload).quotient;
    let entry_of_the_unit = unit - first_unit_of_the_leaf.x;
    if (first_unit_of_the_leaf.y != 0u || first_unit_of_the_leaf.x > unit || entry_of_the_unit >= n.entry_count) {
        undecidable = UNDECIDABLE_DATA_POINTER_BLOCK_NOT_FOUND_AGAIN;
        return step;
    }
    data_block_first = unit;
    data_block_count = min(n.entry_count - entry_of_the_unit, DATA_POINTERS_PER_BLOCK);
    for (var k = 0u; k < data_block_count; k = k + 1u) {
        sws(R_DATA + k * 2u, n.base);
        sws(R_DATA + k * 2u + 1u, entry_offset(n, entry_of_the_unit + k) + 24u);
    }
    step.is_the_block_found = true;
    return step;
}

// ───────────────────────── 实例表 ─────────────────────────

// InstanceTablePage::parse 的每一片：身份 (0, 4, 片号, 0)、行 kind 0 且 flags 0、末条是合法链指针记录；交回 0 最后一片、1 有下一片（写进 next）、2 坏
struct PageParse {
    verdict: u32,
    next: Ptr,
};
fn parse_instance_table_page(p: Packed, page_index: vec2<u32>) -> PageParse {
    var out: PageParse;
    out.verdict = 2u;
    out.next = zero_pointer();
    if (!p.ok) {
        return out;
    }
    if (!is_zero64(p.birth_tree) || p.record_type != 4u || !eq64(p.container, page_index) || !is_zero64(p.container_birth)) {
        return out;
    }
    if (p.record_count == 0u || p.record_width != 88u) {
        return out;
    }
    let last = packed_record_offset(p, p.record_count - 1u);
    if (rd8(p.base, last) != 1u) {
        return out;
    }
    let flag = rd8(p.base, last + 1u);
    let pointer_zero = all_zero(p.base, last + 2u, 86u);
    if (flag == 0u) {
        if (!pointer_zero) {
            return out;
        }
        out.verdict = 0u;
    } else if (flag == 1u) {
        // InstanceTableChainRecord::parse（走读那一种判）：链指针头部加密、压缩那几段不是第一版的取值，或 extent 偏移非 0，这一片损坏
        if (pointer_zero || !pointer_encryption_and_compression_fields_are_the_first_version(p.base, last + 2u)) {
            return out;
        }
        if (!pointer_extent_offset_is_zero(p.base, last + 2u)) {
            return out;
        }
        out.verdict = 1u;
        out.next = parse_node_pointer(p.base, last + 2u);
    } else {
        return out;
    }
    for (var r = 0u; r + 1u < p.record_count; r = r + 1u) {
        let o = packed_record_offset(p, r);
        if (rd8(p.base, o) != 0u || rd8(p.base, o + 21u) != 0u) {
            out.verdict = 2u;
            return out;
        }
    }
    return out;
}
// recovery::read_instance_table_chain：沿链读整张表；true = 读得出解得开
fn instance_table_chain_readable(root: RootFields) -> bool {
    var pointer = root.instance_table;
    var page_index = vec2<u32>(0u, 0u);
    for (var page = 0u; page < 256u; page = page + 1u) {
        // recovery::read_instance_table_chain：一片读不出 UnitUnreadable、解不开 UnitMalformed
        let s = read_unit_via_locations(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            walk_failure_point = FAILURE_POINT_UNIT_UNREADABLE;
            return false;
        }
        let parsed = parse_instance_table_page(parse_packed_unit(s), page_index);
        if (parsed.verdict == 2u) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return false;
        }
        if (parsed.verdict == 0u) {
            return true;
        }
        pointer = parsed.next;
        page_index = add64(page_index, u64_of(1u));
    }
    undecidable = UNDECIDABLE_TOO_DEEP;
    return false;
}

// ───────────────────────── 走到第一个文件 ─────────────────────────

struct WalkResult {
    outcome: u32,
    failure: u32,
    content_matches: bool,
};
fn walk_failed(code: u32) -> WalkResult {
    var w: WalkResult;
    w.outcome = OUTCOME_FAILED;
    w.failure = code;
    w.content_matches = false;
    return w;
}
fn walk_no_file() -> WalkResult {
    var w: WalkResult;
    w.outcome = OUTCOME_NO_FILE;
    w.failure = 0u;
    w.content_matches = false;
    return w;
}
fn tree_kind_key_width(kind: u32) -> u32 {
    if (kind == TREE_KIND_EXTENT) { return 24u; }
    if (kind == TREE_KIND_INODE) { return 8u; }
    if (kind == TREE_KIND_ALLOCATION) { return 10u; }
    if (kind == TREE_KIND_ACCOUNTING) { return 22u; }
    if (kind == TREE_KIND_DELETED_INODES) { return DELETED_INODES_KEY_WIDTH; }
    // logged_ops 根（recovery.rs key_width_for_kind 的码 12：种类 2 + 签发 txg 8，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）
    if (kind == TREE_KIND_LOGGED_OPERATIONS) { return LOGGED_OPERATIONS_KEY_BYTES; }
    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { return CONTAINER_NUMBER_KEY_WIDTH; }
    return NONE;
}
// 进树表的第 position 个种类（发号次序，从 0 数）：1、2、3、4，跳过不进树表的 5，再 6..13
fn tree_kind_in_the_issuing_order(position: u32) -> u32 {
    if (position < 4u) {
        return position + 1u;
    }
    return position + 2u;
}
struct ContainerTreeFrame {
    n: Node,
    next_entry: u32,
};
var<private> container_tree_frames: array<ContainerTreeFrame, 12>;
struct ContainerTreeRead {
    ok: bool,
    mapped_units: u32,
};
// recovery::read_container_tree_index_nodes（墓碑树、谱系树）：从根往下读码 2 节点（提示读不出经映射回退），每个节点照树根同一组核自描述
// （tree_root_checked：树 ID、key 宽 8、出生身份、fsid、key 区间与首末条目相符）；根层级至少 1、每个节点非空；每条条目宽不窄于 120、
// 子指针头部是第一版的取值；层级 ℓ > 1 的条目身份引用全 0、指一个码 2 孩子：层级 ℓ − 1、首 key = 条目的分隔 key；层级 1 的条目指容器，只数不读。
// 交回进映射的单元数：每个码 2 节点（根在内）加每片容器。深度超过 container_tree_frames 的 12 层判不了。
// 只有一处读节点（根与孩子都经 pending 那一组进去）：每个调用点各展开一份。
// container_record_type 不是 NONE 时（建上一版那一样读者读整棵容器树，recovery::read_container_tree）层级 1 的条目连容器一起读：提示读不出经映射回退
// （码 3），解开、核容器头的身份与条目里的身份引用逐字相等、类型 = container_record_type、出生树 = 这棵树、容器号 = 分隔 key、记录宽 = container_record_width、
// 非空（不成立 UnitMalformed；记录宽等于字段表宽之后逐条解记录不会再不成）。CPU 先读完全部码 2 节点再按叶序读容器：调用方先照 NONE 走一遍、
// 过了再照记录类型走一遍（码 2 节点那几判在头一遍都过了，第二遍只会停在容器上）
fn read_container_tree_index_nodes(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>,
    container_record_type: u32, container_record_width: u32) -> ContainerTreeRead {
    var out: ContainerTreeRead;
    out.ok = false;
    out.mapped_units = 0u;
    var nodes = 0u;
    var containers = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_separator_base = 0u;
    var pending_separator_offset = 0u;
    // 交进来的孩子是不是父节点的第一条、最后一条条目（判父节点的子树覆盖区间要它们）
    var pending_is_the_first_entry = false;
    var pending_is_the_last_entry = false;
    var enter = true;
    loop {
        if (enter) {
            enter = false;
            let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
            if (m.failed) {
                return out;
            }
            let n = parse_index_node(m.s);
            if (!tree_root_checked(n, tree, CONTAINER_NUMBER_KEY_WIDTH, pending_pointer, root, fsid_low)) {
                return out;
            }
            // recovery::container_tree_node_of_level_one_is_tight_over_its_entries：层级 1 的节点（孩子是容器、key 是容器号）区间贴紧首末两条条目，
            // 与 tree_root_checked 对叶的那一判同一个失败点
            walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
            if (n.level == 1u && !key_range_is_tight_over_the_entries(n, CONTAINER_NUMBER_KEY_WIDTH)) {
                return out;
            }
            // recovery::read_container_tree_index_nodes：根层级 0 或空、孩子层级不合父层级减一、空、首 key 不是分隔 key、
            // 类型段 0 的身份引用不全 0 都是 UnitMalformed；条目窄 EntryWidthDiffersFromItsFieldTable；子指针头部 UnitMalformed
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            if (n.entry_count == 0u) {
                return out;
            }
            if (depth == 0u) {
                if (n.level == 0u) {
                    return out;
                }
            } else {
                let parent_level = container_tree_frames[depth - 1u].n.level;
                if (n.level + 1u != parent_level
                    || !bytes_equal(n.base, n.smallest_offset, pending_separator_base, pending_separator_offset, CONTAINER_NUMBER_KEY_WIDTH)) {
                    return out;
                }
                // 父节点（层级 ≥ 2）头里的 key 区间是子树覆盖区间（D18 已定项 2）：第一个孩子头里的最小 key = 父头里的最小 key，
                // 最后一个孩子头里的最大 key = 父头里的最大 key（InvariantViolated）
                let parent = container_tree_frames[depth - 1u].n;
                if ((pending_is_the_first_entry && !bytes_equal(parent.base, parent.smallest_offset, n.base, n.smallest_offset, CONTAINER_NUMBER_KEY_WIDTH))
                    || (pending_is_the_last_entry && !bytes_equal(parent.base, parent.largest_offset, n.base, n.largest_offset, CONTAINER_NUMBER_KEY_WIDTH))) {
                    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                    return out;
                }
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            container_tree_frames[depth].n = n;
            container_tree_frames[depth].next_entry = 0u;
            depth = depth + 1u;
            nodes = nodes + 1u;
            continue;
        }
        if (depth == 0u) {
            break;
        }
        let top = depth - 1u;
        let n = container_tree_frames[top].n;
        let e = container_tree_frames[top].next_entry;
        if (e >= n.entry_count) {
            depth = depth - 1u;
            continue;
        }
        container_tree_frames[top].next_entry = e + 1u;
        // inode 树内部条目宽不等于 120 就拒，宽了的同样拒（records.rs parse_inode_internal_entry，C867）
        if (n.entry_width != CONTAINER_TREE_ENTRY_BYTES) {
            walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
            return out;
        }
        let o = entry_offset(n, e);
        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
        if (!record_pointer_head_is_the_first_version(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET)) {
            return out;
        }
        if (n.level == 1u) {
            containers = containers + 1u;
            if (container_record_type != NONE) {
                let container_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
                let container_read = read_via_hint_then_mapping(container_pointer, n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET, 3u, DATA_UNIT_BYTES);
                if (container_read.failed) {
                    return out;
                }
                let container = parse_packed_unit(container_read.s);
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                if (!container.ok) {
                    return out;
                }
                // 条目里的身份引用：出生树 8、打包记录类型 2、容器号 8、容器出生代 8（偏移 8 起）；分隔 key 在偏移 0
                let identity_matches_the_entry = eq64(container.birth_tree, rd64(n.base, o + 8u)) && container.record_type == rd16(n.base, o + 16u)
                    && eq64(container.container, rd64(n.base, o + 18u)) && eq64(container.container_birth, rd64(n.base, o + 26u));
                if (!identity_matches_the_entry || container.record_type != container_record_type || !eq64(container.birth_tree, tree)
                    || !eq64(container.container, rd64(n.base, o)) || container.record_width != container_record_width || container.record_count == 0u) {
                    return out;
                }
            }
            continue;
        }
        // 类型段 0：身份引用（出生树 8、类型 2、容器号 8、容器出生代 8）全 0
        if (!all_zero(n.base, o + 8u, 26u)) {
            return out;
        }
        pending_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
        pending_pointer_base = n.base;
        pending_pointer_offset = o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET;
        pending_separator_base = n.base;
        pending_separator_offset = o;
        pending_is_the_first_entry = e == 0u;
        pending_is_the_last_entry = e + 1u == n.entry_count;
        enter = true;
    }
    out.ok = true;
    out.mapped_units = nodes + containers;
    return out;
}
fn expected_version_index(instance: u32, txg: vec2<u32>) -> u32 {
    let count = hdr(8u);
    for (var i = 0u; i < count; i = i + 1u) {
        if (tab(T_EXPECTED, i * EXPECTED_WORDS) == instance
            && tab(T_EXPECTED, i * EXPECTED_WORDS + 1u) == txg.x
            && tab(T_EXPECTED, i * EXPECTED_WORDS + 2u) == txg.y) {
            return i;
        }
    }
    return NONE;
}
// recovery::walk_to_file_in_the_unit_area_starting_at
// recovery::snapshot_tree_table_entries_paired_one_to_one（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 与第七节岔路 1）：
// 码 14 与码 15 条数相等；按盘上次序每条码 15 的头 ID 恰是一条还没被前面的码 15 配上的码 14 的树 ID；码 14 至多 SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION 条；
// 每条码 14 的头 ID 等于表里第一条码 2 条目的树 ID（表里没有码 2 也不成立）。没有快照条目的表恒成立。四样不成立都是 UnitMalformed，先后不改结论。
// 条目宽调用方已判过是 200；一张树表的条目装不过 128 条（16 KiB 的码 2 节点、200 字节的条目），配上没有记在四个字的位图里
fn snapshot_tree_table_entries_are_paired_one_to_one(tree_table: Node) -> bool {
    var snapshot_inode_roots = 0u;
    var snapshot_extent_roots = 0u;
    var live_inode_tree = vec2<u32>(0u, 0u);
    var has_a_live_inode_tree = false;
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        let kind = rd16(tree_table.base, o + 10u);
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT) {
            snapshot_inode_roots = snapshot_inode_roots + 1u;
        } else if (kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            snapshot_extent_roots = snapshot_extent_roots + 1u;
        } else if (kind == TREE_KIND_INODE && !has_a_live_inode_tree) {
            live_inode_tree = rd64(tree_table.base, o);
            has_a_live_inode_tree = true;
        }
    }
    if (snapshot_inode_roots != snapshot_extent_roots) {
        return false;
    }
    var paired: array<u32, 4>;
    for (var word = 0u; word < 4u; word = word + 1u) {
        paired[word] = 0u;
    }
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        if (rd16(tree_table.base, o + 10u) != TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        let head = rd64(tree_table.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
        var paired_position = NONE;
        for (var candidate = 0u; candidate < tree_table.entry_count && candidate < 128u; candidate = candidate + 1u) {
            let candidate_o = entry_offset(tree_table, candidate);
            if (rd16(tree_table.base, candidate_o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT && eq64(rd64(tree_table.base, candidate_o), head)
                && (paired[candidate / 32u] & (1u << (candidate % 32u))) == 0u) {
                paired_position = candidate;
                break;
            }
        }
        if (paired_position == NONE) {
            return false;
        }
        paired[paired_position / 32u] = paired[paired_position / 32u] | (1u << (paired_position % 32u));
    }
    if (snapshot_inode_roots > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION) {
        return false;
    }
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        if (rd16(tree_table.base, o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT
            && (!has_a_live_inode_tree || !eq64(rd64(tree_table.base, o + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET), live_inode_tree))) {
            return false;
        }
    }
    return true;
}
// 读回来的分配记录（R_ALLOCATION）照 recovery 那三判逐条判：allocation_records_fit_the_pool_geometry、
// allocation_records_are_one_placement_per_device_on_every_device、allocation_record_generations_and_spans_are_judged，次序照它们；交回第一处判不过的失败点码，
// 都过交 FAILURE_POINT_NONE。冷走读与建上一版各调一处
fn allocation_records_judged_against_the_pool_and_the_root(root: RootFields, cfg: SysConfig) -> u32 {
    // 分配记录落在池几何里：盘在池里、槽不低于单元区起点、跨度不越过单元区末尾、同盘不相交（记录按 (盘, 槽) 升序，相邻比即可）
    // recovery::allocation_records_fit_the_pool_geometry 逐条：盘不在池里、槽在单元区起点之下 AllocationRecordOutsideThePoolGeometry，
    // 盘末尾在单元区起点之前 DeviceEndsBeforeTheUnitAreaStart（placement_lies_in_the_unit_area_of_its_device 判到这一条时才判），
    // 跨度越过单元区末尾、同盘两条罩住同一个槽 AllocationRecordOutsideThePoolGeometry
    let device_slots = shr64(device_bytes(), SLOT_SHIFT);
    for (var r = 0u; r < allocation_count; r = r + 1u) {
        let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
        let slot = vec2<u32>(sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 2u));
        let span = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
        if (!device_is_in_the_pool(device)) {
            return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
        }
        if (lt64(slot, cfg.unit_area_start_slot)) {
            return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
        }
        if (lt64(device_slots, cfg.unit_area_start_slot)) {
            return FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START;
        }
        if (lt64(device_slots, add64(slot, u64_of(span)))) {
            return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
        }
        if (r > 0u) {
            let previous_device = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS);
            let previous_slot = vec2<u32>(sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 2u));
            let previous_span = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
            if (previous_device == device && lt64(slot, add64(previous_slot, u64_of(previous_span)))) {
                return FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
            }
        }
    }
    // 每个落点每盘各一条：各盘的 (槽, 跨度, 代, 已释放) 集合相同、同盘槽号唯一（升序已判过）、不少于 10 个落点
    let device_count = hdr(3u);
    var first_device_start = NONE;
    var first_device_count = 0u;
    var devices_with_records = 0u;
    var r = 0u;
    while (r < allocation_count) {
        let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
        var end = r;
        while (end < allocation_count && sw(R_ALLOCATION + end * ALLOCATION_RECORD_WORDS) == device) {
            end = end + 1u;
        }
        devices_with_records = devices_with_records + 1u;
        if (first_device_start == NONE) {
            first_device_start = r;
            first_device_count = end - r;
        } else {
            if (end - r != first_device_count) {
                return FAILURE_POINT_INVARIANT_VIOLATED;
            }
            for (var i = 0u; i < first_device_count; i = i + 1u) {
                let a = R_ALLOCATION + (first_device_start + i) * ALLOCATION_RECORD_WORDS;
                let b = R_ALLOCATION + (r + i) * ALLOCATION_RECORD_WORDS;
                if (sw(a + 1u) != sw(b + 1u) || sw(a + 2u) != sw(b + 2u) || sw(a + 3u) != sw(b + 3u) || sw(a + 4u) != sw(b + 4u) || sw(a + 5u) != sw(b + 5u)) {
                    return FAILURE_POINT_INVARIANT_VIOLATED;
                }
            }
        }
        r = end;
    }
    if (devices_with_records != device_count || first_device_count < 10u) {
        return FAILURE_POINT_INVARIANT_VIOLATED;
    }
    // 分配记录跨度非 0、代不晚于根
    for (var r2 = 0u; r2 < allocation_count; r2 = r2 + 1u) {
        let span = sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 3u) & 0x7FFFu;
        let generation = vec2<u32>(sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 4u), sw(R_ALLOCATION + r2 * ALLOCATION_RECORD_WORDS + 5u));
        if (span == 0u || lt64(root.txg, generation)) {
            return FAILURE_POINT_INVARIANT_VIOLATED;
        }
    }
    return FAILURE_POINT_NONE;
}
// 读回来的记账条目（R_ACCOUNTING）照 recovery 那三判：every_accounting_entry_is_as_wide_as_its_field_table、
// accounting_entry_parsed_and_judged_against_the_root、accounting_entry_count_is_three_plus_six_per_device，次序照它们；交回第一处判不过的失败点码，
// 都过交 FAILURE_POINT_NONE。冷走读与建上一版各调一处
fn accounting_entries_judged_against_the_root(root: RootFields) -> u32 {
    let device_count = hdr(3u);
    // 记账条目：先逐条判宽 = 34，窄了宽了都拒（recovery::every_accounting_entry_is_as_wide_as_its_field_table，EntryWidthDiffersFromItsFieldTable；
    // records::AccountingEntry::parse 不等于 34 交 None，C780「宽于字段表也拒」），
    // 再逐条判代不晚于根、seq ≠ 0（accounting_entry_parsed_and_judged_against_the_root，InvariantViolated）；
    // 次序同 recovery::walk_to_file_in_the_unit_area_starting_at：分配记录的代与跨度、记账条目宽、逐条判记账条目，之后才数记账条目与映射条目
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        if (sw(R_ACCOUNTING + a * 3u + 2u) != 34u) {
            return FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
        }
    }
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        let base = sw(R_ACCOUNTING + a * 3u);
        let o = sw(R_ACCOUNTING + a * 3u + 1u);
        if (lt64(root.txg, rd64(base, o + 14u)) || rd32(base, o + 30u) == 0u) {
            return FAILURE_POINT_INVARIANT_VIOLATED;
        }
    }
    // 记账条目数 3 + 6 × 盘数，inode 号水位的台阶行不算（recovery.rs accounting_entry_count_is_three_plus_six_per_device：
    // 标签 12、设备段 0xFFFF_FFFF、代段早于根的 txg 的行；条目宽不等于 34 的解不开、照算，与 AccountingEntry::parse 交 None 那一支相同；
    // 走到这里的条目都过了上面判宽那一道，宽都是 34）
    var accounting_entries_other_than_the_inode_number_watermark_steps = 0u;
    for (var a = 0u; a < accounting_count; a = a + 1u) {
        let base = sw(R_ACCOUNTING + a * 3u);
        let o = sw(R_ACCOUNTING + a * 3u + 1u);
        let width = sw(R_ACCOUNTING + a * 3u + 2u);
        let is_an_inode_number_watermark_step = width == 34u && rd16(base, o) == 12u && rd32(base, o + 10u) == NONE
            && lt64(rd64(base, o + 14u), root.txg);
        if (!is_an_inode_number_watermark_step) {
            accounting_entries_other_than_the_inode_number_watermark_steps = accounting_entries_other_than_the_inode_number_watermark_steps + 1u;
        }
    }
    if (accounting_entries_other_than_the_inode_number_watermark_steps != 3u + 6u * device_count) {
        return FAILURE_POINT_INVARIANT_VIOLATED;
    }
    return FAILURE_POINT_NONE;
}
// recovery::tree_table_entries_each_kind_at_most_once（连同它判的 snapshot_tree_table_entries_paired_one_to_one）：条目宽 200、每条照
// TreeTableEntry::parse_with_destruction_mark 解得开（条目长度、flags、预留 76、根指针头部加密压缩那几段与 extent 偏移）、码 14 / 15 之外每种至多一条、
// 快照条目一一配对。不成立交 false：冷走读、建上一版报 UnitMalformed，只读挂载打开报 TreeTableEntryMalformed 或同一个 UnitMalformed，都是内容拒收。
// 走读、只读挂载打开、建上一版三样读者各调一处
fn tree_table_entries_each_kind_at_most_once(tree_table: Node) -> bool {
    if (tree_table.entry_width != 200u) {
        return false;
    }
    var kinds_seen = 0u;
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        // TreeTableEntry::parse_with_destruction_mark：flags 是 0，或码 14 条目上恰是位 0（销毁中）；别的位、别的种类上的位 0 拒收
        let flags = rd16(tree_table.base, o + TREE_TABLE_ENTRY_FLAGS_OFFSET);
        let flags_are_accepted = flags == 0u
            || (flags == TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED && rd16(tree_table.base, o + 10u) == TREE_KIND_SNAPSHOT_INODE_ROOT);
        if (rd16(tree_table.base, o + 8u) != 200u || !flags_are_accepted || !all_zero(tree_table.base, o + 124u, 76u)) {
            return false;
        }
        // TreeTableEntry::parse：树根指针头部加密、压缩那几段不是第一版的取值，这条条目损坏
        if (!pointer_encryption_and_compression_fields_are_the_first_version(tree_table.base, o + 14u)) {
            return false;
        }
        // TreeTableEntry::parse：树根指针头部 extent 偏移非 0，这条条目损坏（与条目长度、flags、预留 76 同在读任何一棵树之前拒）
        if (!pointer_extent_offset_is_zero(tree_table.base, o + 14u)) {
            return false;
        }
        let kind = rd16(tree_table.base, o + 10u);
        // 码 14、15 两种快照条目一个快照一对、可以有多条：不进「每种至多一条」，下面按头 ID 一一配对判
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        // tree_table_entries_each_kind_at_most_once 的 kinds_seen 是整个 u16 的集合：种类 < 32 的记进位图，
        // 种类 ≥ 32 的（第一版写者写不出、坏盘上才有）逐条与前面各条比（树表条目数不超过一个码 2 节点装得下的，与走读内核 ③ 的两两比同一量级）。
        // 走读内核的 ③（crash_judge_checker.wgsl 判 I-9.16 那一段）同样比整个 u16，两个内核在种类 ≥ 32 的重复上都拦得到
        if (kind < 32u) {
            if ((kinds_seen & (1u << kind)) != 0u) {
                return false;
            }
            kinds_seen = kinds_seen | (1u << kind);
        } else {
            for (var earlier = 0u; earlier < e; earlier = earlier + 1u) {
                if (rd16(tree_table.base, entry_offset(tree_table, earlier) + 10u) == kind) {
                    return false;
                }
            }
        }
    }
    if (!snapshot_tree_table_entries_are_paired_one_to_one(tree_table)) {
        return false;
    }
    return true;
}
// recovery::tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order（I-9.16 ① ②）：不成立交 false（InvariantViolated）
fn tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(tree_table: Node) -> bool {
    // I-9.16 ①：树 ID 严格升序
    for (var e = 1u; e < tree_table.entry_count; e = e + 1u) {
        let previous = rd64(tree_table.base, entry_offset(tree_table, e - 1u));
        let current = rd64(tree_table.base, entry_offset(tree_table, e));
        if (!lt64(previous, current)) {
            return false;
        }
    }
    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12,13（recovery.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）相邻两棵严格升序
    var previous_tree = vec2<u32>(0u, 0u);
    var have_previous = false;
    for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
        let kind = tree_kind_in_the_issuing_order(k);
        for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
            let o = entry_offset(tree_table, e);
            if (rd16(tree_table.base, o + 10u) == kind) {
                let tree = rd64(tree_table.base, o);
                if (have_previous && !lt64(previous_tree, tree)) {
                    return false;
                }
                previous_tree = tree;
                have_previous = true;
                break;
            }
        }
    }
    return true;
}
fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
    let fsid_low = vec2<u32>(cfg.fsid.x, cfg.fsid.y);
    // 实例表：根直接持有第 0 片
    let instance_table_slice = read_unit_via_locations(root.instance_table, DATA_UNIT_BYTES);
    if (instance_table_slice.version == NONE) {
        return walk_failed(FAILURE_POINT_UNIT_UNREADABLE);
    }
    let instance_table = parse_packed_unit(instance_table_slice);
    if (!instance_table.ok) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (instance_table.record_type != 4u || instance_table.record_count == 0u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    let first_page = parse_instance_table_page(instance_table, vec2<u32>(0u, 0u));
    if (first_page.verdict == 2u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    if (first_page.verdict == 1u) {
        if (!instance_table_chain_readable(root)) {
            return walk_failed(walk_failure_point);
        }
    }
    // 树表
    let tree_table_slice = read_unit_via_locations(root.tree_table, NODE_BYTES);
    if (tree_table_slice.version == NONE) {
        return walk_failed(FAILURE_POINT_UNIT_UNREADABLE);
    }
    let tree_table = parse_index_node(tree_table_slice);
    if (!tree_table.ok) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (tree_table.key_width != 8u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    if (tree_table.entry_count == 0u) {
        return walk_no_file();
    }
    // 条目逐条解开、每种至多一条
    if (!tree_table_entries_each_kind_at_most_once(tree_table)) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (!tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(tree_table)) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // 中央映射树：走读到 TreeRoots 那一步总要读它；先读也罢（读得出读不出与次序无关，见 crash_judge_gpu.rs 的说明）
    // 映射树读不出不在这里报：CPU 到第一次有提示读不出、要经映射回退时才读它（那一刻报它读不出的那一处），走完各棵树、缺不缺树判完之后再要它一次
    // （recovery 的 CentralMappingTreeReadOnFirstUse::locations_of_key 与 into_tree）；读不出的那一处记在 mapping_tree_failure_point
    mapping_tree_failure_point = FAILURE_POINT_NONE;
    let mapping = read_code_two_tree(root.mapping_root, 0u, 0u, root.mapping_root.birth_tree, 1u, root, fsid_low);
    if (!mapping.ok) {
        if (undecidable != 0u) {
            return walk_failed(FAILURE_POINT_NONE);
        }
        mapping_tree_failure_point = walk_failure_point;
    }
    var have_extent = false;
    var have_inode = false;
    var have_allocation = false;
    var have_accounting = false;
    var extent_tree = vec2<u32>(0u, 0u);
    var extent_nodes = 0u;
    var allocation_nodes = 0u;
    var accounting_nodes = 0u;
    var deadlist_nodes = 0u;
    var deadlist_entries = 0u;
    var have_deleted_inodes = false;
    var have_logged_operations = false;
    var container_tree_units = 0u;
    var inode_root = bad_node();
    var inode_root_pointer = zero_pointer();
    for (var e = 0u; e < tree_table.entry_count; e = e + 1u) {
        let o = entry_offset(tree_table, e);
        let tree = rd64(tree_table.base, o);
        // I-7.8：树 ID 低于水位
        if (!lt64(tree, root.watermark)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        let kind = rd16(tree_table.base, o + 10u);
        // 只读快照的两种条目：冷走读读的是活头那一版的文件，跳过它们（recovery.rs walk_to_file 在判过树 ID 水位之后 continue）
        if (kind == TREE_KIND_SNAPSHOT_INODE_ROOT || kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT) {
            continue;
        }
        let pointer = parse_node_pointer(tree_table.base, o + 14u);
        if (pointer.all_zero) {
            continue;
        }
        if (kind == TREE_KIND_ALLOCATION) {
            let read = read_allocation_tree(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            allocation_nodes = read.node_count;
            have_allocation = true;
            continue;
        }
        if (kind == TREE_KIND_EXTENT) {
            let read = read_extent_tree_of_the_first_file(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            extent_nodes = read.node_count;
            extent_tree = tree;
            have_extent = true;
            continue;
        }
        // 记账树与 deadlist 树（第三棵多层码 2 树，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：同一个读法整棵读回来，
        // 一个调用点（GPU 程序把每个调用点展开成一份拷贝）；deadlist 的节点与叶条目各占映射条目（条目留存的那几条，判决 S3 的 α），数映射条目时要它们
        if (kind == TREE_KIND_ACCOUNTING || kind == TREE_KIND_DEADLIST) {
            let read = read_code_two_tree(pointer, tree_table.base, o + 14u, tree, select(0u, KEY_SCHEMA_DEADLIST, kind == TREE_KIND_DEADLIST), root, fsid_low);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            if (kind == TREE_KIND_DEADLIST) {
                deadlist_nodes = read.node_count;
                deadlist_entries = read.entry_count;
                continue;
            }
            accounting_nodes = read.node_count;
            have_accounting = true;
            continue;
        }
        // 墓碑树、谱系树与退役树：从根往下读码 2 节点、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）
        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
            let read = read_container_tree_index_nodes(pointer, tree_table.base, o + 14u, tree, root, fsid_low, NONE, 0u);
            if (!read.ok) {
                return walk_failed(walk_failure_point);
            }
            container_tree_units = container_tree_units + read.mapped_units;
            continue;
        }
        let key_width = tree_kind_key_width(kind);
        if (key_width == NONE) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        // inode 树根、deleted_inodes 根与 logged_ops 根（进映射的树根：提示读不出经映射回退，再核自描述；logged_ops 根只数进映射条目数，
        // 条目不在冷走读里解，recovery.rs walk_to_file 同）
        let m = read_via_hint_then_mapping(pointer, tree_table.base, o + 14u, 2u, NODE_BYTES);
        if (m.failed) {
            return walk_failed(walk_failure_point);
        }
        let n = parse_index_node(m.s);
        if (!tree_root_checked(n, tree, key_width, pointer, root, fsid_low)) {
            return walk_failed(walk_failure_point);
        }
        if (kind == TREE_KIND_INODE) {
            inode_root = n;
            inode_root_pointer = pointer;
            have_inode = true;
        }
        if (kind == TREE_KIND_DELETED_INODES) {
            have_deleted_inodes = true;
        }
        if (kind == TREE_KIND_LOGGED_OPERATIONS) {
            have_logged_operations = true;
        }
    }
    if (!have_extent || !have_inode || !have_allocation || !have_accounting) {
        return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
    }
    if (mapping_tree_failure_point != FAILURE_POINT_NONE) {
        return walk_failed(mapping_tree_failure_point);
    }
    let allocation_failure = allocation_records_judged_against_the_pool_and_the_root(root, cfg);
    if (allocation_failure != FAILURE_POINT_NONE) {
        return walk_failed(allocation_failure);
    }
    let accounting_failure = accounting_entries_judged_against_the_root(root);
    if (accounting_failure != FAILURE_POINT_NONE) {
        return walk_failed(accounting_failure);
    }
    // 映射条目数 = 1 + extent 节点 + 分配记录树节点 + 记账树节点 + deadlist 树节点 + deadlist 条目数 + inode 叶容器数 + 数据单元数
    // + 删除那两棵树与谱系树的节点（recovery.rs 的 deletion_tree_nodes：deleted_inodes 根有就 1、logged_ops 根有就 1，墓碑树、谱系树各自每个码 2 节点加每片容器；
    // deadlist 那两项是 mapping_entry_count_is_one_per_mapped_unit 的 deadlist_tree_nodes 与 mapping_entries_kept_for_the_snapshots）
    let deletion_tree_nodes = select(0u, 1u, have_deleted_inodes) + select(0u, 1u, have_logged_operations) + container_tree_units;
    // 数据单元数是每个文件的（第一个文件与第一个文件之外各文件，recovery 的 data_units_of_every_file）
    if (mapping_count != 1u + extent_nodes + allocation_nodes + accounting_nodes + deadlist_nodes + deadlist_entries + inode_root.entry_count + data_count
        + data_units_of_the_files_other_than_the_first + deletion_tree_nodes) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // inode 树：根层级 1，每条内部条目 → 叶容器 → 记录
    if (inode_root.level != 1u) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // 条目宽只在有条目时核：recovery.rs 逐条 inode_internal_entry_parsed_and_judged 时才核宽，零条目的根（unlink 删掉树里最后一条记录之后）
    // 条目宽是什么都不拒；有条目时宽不等于 120 就拒，宽了的同样拒（C867）
    if (inode_root.entry_count > 0u && inode_root.entry_width != CONTAINER_TREE_ENTRY_BYTES) {
        return walk_failed(FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE);
    }
    var found_inode = false;
    var inode_nlink = 0u;
    var inode_size = vec2<u32>(0u, 0u);
    var inode_object_birth = vec2<u32>(0u, 0u);
    var inode_numbers_read_back = InodeNumbersReadBack(false, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u));
    for (var e = 0u; e < inode_root.entry_count; e = e + 1u) {
        let o = entry_offset(inode_root, e);
        let separator = rd64(inode_root.base, o);
        let inode_numbers_of_the_leaves_before = inode_numbers_read_back;
        let identity_birth_tree = rd64(inode_root.base, o + 8u);
        let identity_type = rd16(inode_root.base, o + 16u);
        let identity_container = rd64(inode_root.base, o + 18u);
        let identity_birth = rd64(inode_root.base, o + 26u);
        // recovery::inode_internal_entry_parsed_and_judged：先解条目（子指针头部 UnitMalformed），再判类型段（InvariantViolated）
        let child = parse_node_pointer(inode_root.base, o + 34u);
        // records::parse_inode_internal_entry_or_refusal：子指针头部加密、压缩那几段不是第一版的取值，这条内部条目损坏
        if (!child.head_ok) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        // records::parse_inode_internal_entry_or_refusal：子指针头部 extent 偏移非 0，这条内部条目损坏
        if (!child.extent_offset_zero) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        if (identity_type != 2u) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        let m = read_via_hint_then_mapping(child, inode_root.base, o + 34u, 3u, DATA_UNIT_BYTES);
        if (m.failed) {
            return walk_failed(walk_failure_point);
        }
        let leaf = parse_packed_unit(m.s);
        if (!leaf.ok) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        if (!eq64(leaf.birth_tree, identity_birth_tree) || leaf.record_type != identity_type || !eq64(leaf.container, identity_container)
            || !eq64(leaf.container_birth, identity_birth) || leaf.record_width != 140u || !eq64(leaf.fsid, fsid_low)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!eq64(child.birth_tree, identity_birth_tree) || leaf.birth_sequence != child.tail.x) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (lt64(root.txg, leaf.birth_txg) || leaf.instance != child.instance) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        for (var rec = 0u; rec < leaf.record_count; rec = rec + 1u) {
            let ro = packed_record_offset(leaf, rec);
            if (!inode_record_parses_by_the_core_reader(leaf.base, ro)) {
                return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
            }
            let inode = rd64(leaf.base, ro);
            if (lt64(inode, separator) || lt64(inode, identity_container)) {
                return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
            }
            inode_numbers_read_back = inode_numbers_read_back_with(inode_numbers_read_back, inode);
            if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {
                found_inode = true;
                inode_object_birth = rd64(leaf.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                inode_nlink = rd32(leaf.base, ro + INODE_RECORD_NLINK_OFFSET);
                inode_size = rd64(leaf.base, ro + INODE_RECORD_SIZE_OFFSET);
            }
        }
        // recovery.rs 冷走读与可写挂载重建同一判：全部叶的记录按叶序严格升序、分隔 key 大于前面那几片的最大记录号（片里逐条判过、读完这一片判）
        if (!inode_records_of_the_leaf_ascend_after_the_leaves_before(leaf, separator, inode_numbers_of_the_leaves_before)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
    }
    // inode 树根头里的 key 区间是子树覆盖区间（recovery.rs walk_to_file：读完叶容器、取第一个文件那条记录之前判）
    if (!inode_tree_root_key_range_is_the_subtree_coverage(inode_root, inode_numbers_read_back)) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    if (!found_inode) {
        return walk_no_file();
    }
    // 删除中的 inode（nlink = 0）：从意图那次发布起冷走读答「没有文件」，与 recovery.rs 冷走读同一判、同一处（取到第一个文件那条记录之后、
    // 核单元数之前；D8（核心索引结构） 已定项 16 第 4 条）
    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {
        return walk_no_file();
    }
    // 数据单元数按上段叶条目的标签对 size（core 冷走读的 data_unit_count_matches_the_inode_size，即 write_request_split::data_units_read_within_the_file_size）：
    // size 按净荷容量算出的单元数是 max(1, ⌈size ÷ 32634⌉)；标签 1、2 存在的单元不少于它，多出来的是截断中间态里 size 之外还挂着的单元，
    // 不读、不判声明长度（判决 m4-trunc-r1 第三节 K2）；标签 3、4、0 少于它，存在的单元之后到 size 是洞、读出 0、不读盘（K5）
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    var units_implied_by_the_size = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;
    if (is_zero64(units_implied_by_the_size)) {
        units_implied_by_the_size = u64_of(1u);
    }
    let tail_is_a_hole = first_file_upper_leaf_tag == 0u || first_file_upper_leaf_tag == 3u || first_file_upper_leaf_tag == 4u;
    if (tail_is_a_hole) {
        if (!lt64(u64_of(data_count), units_implied_by_the_size)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
    } else if (lt64(u64_of(data_count), units_implied_by_the_size)) {
        return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
    }
    // size 之内要读、要核的单元数：有洞时是存在的全部，没有洞时是 size 算出的那几个（不多于 data_count，装得进 32 位）
    let data_units_within_the_size = select(units_implied_by_the_size.x, data_count, tail_is_a_hole);
    var content_matches = true;
    var chunk_first = 0u;
    var chunk_count = 0u;
    if (expected != NONE) {
        chunk_first = tab(T_EXPECTED, expected * EXPECTED_WORDS + 3u);
        chunk_count = tab(T_EXPECTED, expected * EXPECTED_WORDS + 4u);
        // 期望那一版的内容按写侧的切法切成 max(1, ⌈内容长度 ÷ 净荷⌉) 个单元（crash_judge_tables 的 expected_version_chunks），读出的文件是 size 那么长：
        // 两边单元数不同，内容就不同
        if (!eq64(u64_of(chunk_count), units_implied_by_the_size)) {
            content_matches = false;
        }
    }
    // 数据指针按块读（R_DATA 里是 data_block_first 起的 data_block_count 条）：读到这一块末尾，从下段根一层层往下找含下一个单元的那片叶，
    // 每一步读一个节点；找下一块时此刻要读的那条节点指针在哪（字节基、偏移），不在找是 NONE
    var next_block_pointer_base = NONE;
    var next_block_pointer_offset = 0u;
    var i = 0u;
    // 迭代上界：每个单元读一次，加上每找一块往下读的层数（每块至少一个单元，层数不过 MAX_FRAMES，内部节点每一步往下一层）。
    // 跨轮带着的：i 只增，找块时 next_block_pointer_* 一层层往下、找到就回 NONE；data_block_first ≤ i < data_block_first + data_block_count 时读单元
    loop {
        if (i >= data_units_within_the_size) {
            break;
        }
        let finding_the_next_block = i >= data_block_first + data_block_count;
        if (finding_the_next_block && next_block_pointer_base == NONE) {
            // 从下段根起：第一个文件那一条上段叶条目偏移 25 起是下段根指针（一块装不下的文件有下段，内联的只有一个单元）
            next_block_pointer_base = first_file_upper_entry_base;
            next_block_pointer_offset = first_file_upper_entry_offset + 25u;
        }
        var base = next_block_pointer_base;
        var o = next_block_pointer_offset;
        var pointer: Ptr;
        var unit_class = 2u;
        var unit_bytes = NODE_BYTES;
        if (finding_the_next_block) {
            pointer = parse_node_pointer(base, o);
        } else {
            base = sw(R_DATA + (i - data_block_first) * 2u);
            o = sw(R_DATA + (i - data_block_first) * 2u + 1u);
            pointer = parse_data_pointer(base, o);
            unit_class = 1u;
            unit_bytes = DATA_UNIT_BYTES;
        }
        // 读下段节点与读数据单元同一个调用点（GPU 程序把每个调用点展开成一份拷贝）
        let m = read_via_hint_then_mapping(pointer, base, o, unit_class, unit_bytes);
        if (finding_the_next_block) {
            let step = find_the_next_block_of_data_pointers(m, i);
            if (undecidable != 0u) {
                return walk_failed(FAILURE_POINT_NONE);
            }
            next_block_pointer_base = step.pointer_base;
            next_block_pointer_offset = step.pointer_offset;
            if (step.is_the_block_found) {
                next_block_pointer_base = NONE;
            }
            continue;
        }
        if (m.failed) {
            return walk_failed(walk_failure_point);
        }
        let unit = parse_data_unit(m.s);
        if (!unit.ok) {
            return walk_failed(FAILURE_POINT_UNIT_MALFORMED);
        }
        let first_file_byte = mul64_64_saturating(u64_of(i), payload);
        if (!eq64(unit.tree, extent_tree) || !eq64(unit.object, u64_of(FIRST_INODE_NUMBER)) || !eq64(unit.anchor_offset, first_file_byte)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!eq64(unit.object_birth, inode_object_birth)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!eq64(unit.birth_txg, pointer.birth_txg) || unit.instance != pointer.instance || !eq64(unit.transaction, pointer.tail) || !eq64(unit.fsid, fsid_low)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        var expected_length = sub64_saturating(inode_size, first_file_byte);
        if (lt64(payload, expected_length)) {
            expected_length = payload;
        }
        if (!eq64(u64_of(unit.declared_length), expected_length)) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (!unit.padding_zero) {
            return walk_failed(FAILURE_POINT_INVARIANT_VIOLATED);
        }
        if (expected != NONE && i < chunk_count) {
            if (tab(T_CHUNKS, (chunk_first + i) * 2u) != unit.payload_crc || tab(T_CHUNKS, (chunk_first + i) * 2u + 1u) != unit.declared_length) {
                content_matches = false;
            }
        }
        i = i + 1u;
    }
    // 洞（存在的单元之后到 size）不读盘，读出 0（core 的 content_padded_with_the_hole_up_to_the_size）。每个洞的位置照期望那一版的那个单元比：
    // 声明长度是 size 切到这一段的长度，载荷区 CRC 是内核现算的全零那一份——载荷区罩预留位、载荷与补到单元末尾的 0，洞的载荷全是 0，
    // 所以每个洞的位置都是同一个数（判决 m4-trunc-r1 第六节 S5：判表不加「这一块全零」位、不加从镜像算出的列，期望只来自版本表）。
    // 只比到期望那一版的单元数为止：两边单元数不同上面已判内容不同，再往后比不出别的，也不让一个很大的 size 拖长这一圈
    if (expected != NONE && data_units_within_the_size < chunk_count) {
        let payload_region_crc_of_a_hole = crc32c_of_zero_bytes(DATA_UNIT_PAYLOAD_REGION_BYTES);
        for (var position = data_units_within_the_size; position < chunk_count; position = position + 1u) {
            let first_file_byte = mul64_64_saturating(u64_of(position), payload);
            var length_of_the_hole_here = sub64_saturating(inode_size, first_file_byte);
            if (lt64(payload, length_of_the_hole_here)) {
                length_of_the_hole_here = payload;
            }
            if (tab(T_CHUNKS, (chunk_first + position) * 2u) != payload_region_crc_of_a_hole
                || !eq64(u64_of(tab(T_CHUNKS, (chunk_first + position) * 2u + 1u)), length_of_the_hole_here)) {
                content_matches = false;
            }
        }
    }
    var w: WalkResult;
    w.outcome = OUTCOME_FILE_READ;
    w.failure = 0u;
    w.content_matches = content_matches;
    return w;
}


// ───────────────────────── 每个状态的开场：表的目录、草稿区、持久掩码 ─────────────────────────

fn begin_state(local: u32) {
    for (var t = 0u; t < TABLE_COUNT; t = t + 1u) {
        B[t] = tables[t * 2u];
    }
    scratch_base = local * dispatch.scratch_words;
    undecidable = 0u;
    record_count = 0u;
    root_count = 0u;
    mapping_count = 0u;
    mapping_leaf_count = 0u;
    accounting_count = 0u;
    allocation_count = 0u;
    seen_count = 0u;
    data_count = 0u;
    data_units_of_the_files_other_than_the_first = 0u;
    data_block_first = 0u;
    data_block_count = 0u;
    upper_count = 0u;
    row_count = 0u;
    deadlist_leaf_count = 0u;
    extent_tree_records_its_files = false;
    files_of_the_extent_tree = 0u;
    rebuild_judges_the_pointers_of_the_version = false;
    some_pointer_of_the_version_does_not_ascend_by_device = false;
    some_mapping_position_of_the_version_does_not_ascend_by_device = false;
    var ordinal_lo = dispatch.block_start_lo + local;
    var ordinal_hi = dispatch.block_start_hi;
    if (ordinal_lo < dispatch.block_start_lo) {
        ordinal_hi = ordinal_hi + 1u;
    }
    for (var word = 0u; word < PERSISTED_MASK_WORDS; word = word + 1u) {
        persisted[word] = 0u;
    }
    // 给定状态表（crash_judge_tables::JudgeStateSource::GivenPersistedSets）：这张表不空，第 k 个状态就是表里第 k 个持久掩码
    // （每个 ⌈枚举写数 ÷ 32⌉ 个字），撕裂镜像与重放已在掩码里，不再按段拆
    given_state_ordinal = ordinal_lo;
    if (table_length(T_GIVEN_PERSISTED_SETS) != 0u) {
        let words_per_state = (hdr(1u) + 31u) / 32u;
        for (var word = 0u; word < words_per_state; word = word + 1u) {
            persisted[word] = tab(T_GIVEN_PERSISTED_SETS, ordinal_lo * words_per_state + word);
        }
        return;
    }
    // 序号 → 持久掩码（与 crash_facts::persisted_writes_of_state_by_the_segment_expansions 同一套数字次序）
    let segment_count = hdr(2u);
    var segment_of_state = segment_count;
    for (var s = 0u; s < segment_count; s = s + 1u) {
        let end = vec2<u32>(tab(T_SEGMENTS, s * SEGMENT_WORDS + 4u), tab(T_SEGMENTS, s * SEGMENT_WORDS + 5u));
        if (lt64(vec2<u32>(ordinal_lo, ordinal_hi), end)) {
            segment_of_state = s;
            break;
        }
    }
    // 之前的段整段持久：数位那一组与整组落的单元写那一组合起来就是段里全部的写
    for (var s = 0u; s < segment_of_state; s = s + 1u) {
        let digit_first = tab(T_SEGMENTS, s * SEGMENT_WORDS);
        let digit_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 1u);
        for (var k = 0u; k < digit_count; k = k + 1u) {
            set_persisted(tab(T_SEGMENT_WRITES, digit_first + k));
        }
        let group_first = tab(T_SEGMENTS, s * SEGMENT_WORDS + 7u);
        let group_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 8u);
        for (var k = 0u; k < group_count; k = k + 1u) {
            set_persisted(tab(T_SEGMENT_WRITES, group_first + k));
        }
    }
    var torn: array<u32, PERSISTED_MASK_WORDS>;
    for (var word = 0u; word < PERSISTED_MASK_WORDS; word = word + 1u) {
        torn[word] = 0u;
    }
    if (segment_of_state < segment_count) {
        let s = segment_of_state;
        // 段内序号满 64 位（判决 m3-crash-enumeration-merge-r2 第二节 L1-7：只用低 32 位会让 33–64 写的段拆错）
        var remaining = sub64(vec2<u32>(ordinal_lo, ordinal_hi), vec2<u32>(tab(T_SEGMENTS, s * SEGMENT_WORDS + 2u), tab(T_SEGMENTS, s * SEGMENT_WORDS + 3u)));
        let expansion = tab(T_SEGMENTS, s * SEGMENT_WORDS + 6u);
        // 不展开的段序号区间是空的，落不到这里；落到了就是表与序号对不上，判不了
        if (expansion == SEGMENT_EXPANSION_CODE_NOT_EXPANDED) {
            undecidable = UNDECIDABLE_STATE_IN_A_SEGMENT_NOT_EXPANDED;
        } else {
            let digit_first = tab(T_SEGMENTS, s * SEGMENT_WORDS);
            let digit_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 1u);
            let group_first = tab(T_SEGMENTS, s * SEGMENT_WORDS + 7u);
            let group_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 8u);
            let slice_digit_first = tab(T_SEGMENTS, s * SEGMENT_WORDS + 9u);
            let slice_digit_count = tab(T_SEGMENTS, s * SEGMENT_WORDS + 10u);
            // 分片的段（C790）：段内编号 = 片起点 + 片内序号。片起点拆成的各位（段写表里从 slice_digit_first 起，单元写那一组那一位在前）
            // 加上片内序号、逐位进位，每一步只用 u32（片长不超过 2^32，片内序号装得进低 32 位；crash::add_to_the_mixed_radix_digits 同一个算法）
            if (slice_digit_count > 0u) {
                if (remaining.y != 0u) {
                    undecidable = UNDECIDABLE_SLICE_PAST_ITS_DOMAIN;
                } else {
                    var carry = remaining.x;
                    var position = 0u;
                    if (group_count > 0u) {
                        let sum = tab(T_SEGMENT_WRITES, slice_digit_first) + carry % 2u;
                        carry = carry / 2u + sum / 2u;
                        if (sum % 2u == 1u) {
                            for (var k = 0u; k < group_count; k = k + 1u) {
                                set_persisted(tab(T_SEGMENT_WRITES, group_first + k));
                            }
                        }
                        position = 1u;
                    }
                    for (var k = 0u; k < digit_count; k = k + 1u) {
                        let write = tab(T_SEGMENT_WRITES, digit_first + k);
                        let choices = tab(T_RECORDED, write * 4u);
                        let sum = tab(T_SEGMENT_WRITES, slice_digit_first + position + k) + carry % choices;
                        carry = carry / choices + sum / choices;
                        let digit = sum % choices;
                        if (digit == choices - 1u) {
                            set_persisted(write);
                        } else if (digit == 1u) {
                            torn[write >> 5u] = torn[write >> 5u] | (1u << (write & 31u));
                        }
                    }
                    if (carry != 0u) {
                        undecidable = UNDECIDABLE_SLICE_PAST_ITS_DOMAIN;
                    }
                }
            } else {
                // 甲二有单元写：段内序号的最低一位是「单元写全落」，其余拆到原地写上（crash::CrashStatePlan 同一次序）
                if (group_count > 0u) {
                    if ((remaining.x & 1u) == 1u) {
                        for (var k = 0u; k < group_count; k = k + 1u) {
                            set_persisted(tab(T_SEGMENT_WRITES, group_first + k));
                        }
                    }
                    remaining = shr64(remaining, 1u);
                }
                for (var k = 0u; k < digit_count; k = k + 1u) {
                    let write = tab(T_SEGMENT_WRITES, digit_first + k);
                    let choices = tab(T_RECORDED, write * 4u);
                    let divided = divide_by_a_divisor_below_65536(remaining, choices);
                    let digit = divided.remainder;
                    remaining = divided.quotient;
                    if (digit == choices - 1u) {
                        set_persisted(write);
                    } else if (digit == 1u) {
                        torn[write >> 5u] = torn[write >> 5u] | (1u << (write & 31u));
                    }
                }
            }
        }
    }
    let recorded_count = hdr(0u);
    for (var w = 0u; w < recorded_count; w = w + 1u) {
        if ((torn[w >> 5u] & (1u << (w & 31u))) == 0u) {
            continue;
        }
        let torn_index = tab(T_RECORDED, w * 4u + 1u);
        if (torn_index == NONE) {
            continue;
        }
        set_persisted(torn_index);
        let replay_first = tab(T_RECORDED, w * 4u + 2u);
        let replay_count = tab(T_RECORDED, w * 4u + 3u);
        for (var r = 0u; r < replay_count; r = r + 1u) {
            let replay_index = tab(T_REPLAYS, (replay_first + r) * 2u);
            let replayed = tab(T_REPLAYS, (replay_first + r) * 2u + 1u);
            if (is_persisted(replayed)) {
                set_persisted(replay_index);
            }
        }
    }
}
