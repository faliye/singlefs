// GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
// 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
// I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6、I-9.24，回退候选集每一版）、
// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单；退役记录 I-9.32–I-9.35、
// 谱系与退役容器 I-9.36 / I-9.37 也在这一遍）与 I-9.6 墓碑那一半
// （所选根的墓碑树），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。


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

// extent 树里的一条数据指针：照 r_note 记；I-5.5 那一遍开着时再记两条位置条目的落点（全零指针不记，同 walk.rs 过滤 all_zero）
fn r_note_data_pointer(p: Ptr) {
    r_note(p);
    if (!r_collecting_data_unit_slots || p.all_zero) {
        return;
    }
    r_note_data_unit_slot(p.device0, p.slot0);
    r_note_data_unit_slot(p.device1, p.slot1);
}

// 删除态扫描那一遍开着时，r_extent_tree 读到的每条数据指针按它属于的 inode 号数一个（walk.rs data_unit_count_of_each_inode_without_judging：
// 上段叶内联的那一个属于条目 key 里的号，下段叶的每条记录属于引它那条上段条目的号；全零的数据指针不指单元，不数），记进 D_DATA_UNIT_COUNTS
var<private> r_counting_data_units_of_each_inode: bool;

var<private> d_data_unit_inode_count: u32;

fn r_count_data_unit_of_inode(inode: vec2<u32>, p: Ptr) {
    if (!r_counting_data_units_of_each_inode || p.all_zero) {
        return;
    }
    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
        let row = D_DATA_UNIT_COUNTS + i * D_DATA_UNIT_COUNT_WORDS;
        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
            sws(row + 2u, sw(row + 2u) + 1u);
            return;
        }
    }
    if (d_data_unit_inode_count >= D_MAX_DATA_UNIT_INODES) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (48u << 8u);
        return;
    }
    let row = D_DATA_UNIT_COUNTS + d_data_unit_inode_count * D_DATA_UNIT_COUNT_WORDS;
    sws(row, inode.x);
    sws(row + 1u, inode.y);
    sws(row + 2u, 1u);
    d_data_unit_inode_count = d_data_unit_inode_count + 1u;
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
                        let inline_data_pointer = parse_data_pointer(n.base, o + 25u);
                        r_note_data_pointer(inline_data_pointer);
                        r_count_data_unit_of_inode(extent_key_inode(n.base, o), inline_data_pointer);
                    } else {
                        return false;
                    }
                }
            } else {
                if (n.entry_width < 112u) {
                    return false;
                }
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let lower_data_pointer = parse_data_pointer(n.base, entry_offset(n, e) + 24u);
                    r_note_data_pointer(lower_data_pointer);
                    r_count_data_unit_of_inode(inode, lower_data_pointer);
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
        r_note_tree_table_entry_birth_txg(row, tt.base, o);
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
        } else if (kind == TREE_KIND_DELETED_INODES) {
            // deleted_inodes 根的条目只是号，不引用别的单元（根自己上面记过）
        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
            // 墓碑树、谱系树、退役树（walk.rs 的 TreeKindForReferenceScan::ContainerTree）：根是层级 1 时每条条目的子指针（条目偏移 34 起）指一片容器；
            // 根长高之后（层级 > 1）这一遍不往下数，记成数不全
            if (node.level == 1u && node.entry_width >= CONTAINER_TREE_ENTRY_BYTES) {
                for (var container = 0u; container < node.entry_count; container = container + 1u) {
                    r_note(parse_node_pointer(node.base, entry_offset(node, container) + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET));
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

// I-5.4 的一版里一个分配记录树根：见过的交回它那一次读没读到叶；没见过的记进去表、整棵读，读到叶就判这一棵（judge_allocation_record_ranges）
fn c_allocation_root_of_a_version_walked_to_its_leaves(base: u32, offset: u32) -> bool {
    let p = parse_node_pointer(base, offset);
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
// 第二节 Y1 allocation_record_tree_root 那一行）。条目宽不到 200 的表也判，所以放在跳过窄表那一句之前。读不出的那几格 CPU 报部分没判，这里不置位
fn judge_allocation_records_disjoint_of_a_version(base: u32, tree_table_offset: u32, held_offset: u32) {
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
        if (version_index < c_candidate_count) {
            base = checker_root_of(sw(C_CANDIDATES + version_index)).base;
            tree_table_offset = 36u;
            allocation_offset = 342u;
        } else if (version_index < c_candidate_count + c_version_count) {
            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
        } else {
            let next_version = record_row_of_the_version_the_next_mount_applies_first();
            if (next_version == NONE) {
                break;
            }
            base = checker_record_of_row(next_version).base;
        }
        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);
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
    for (var bucket = 0u; bucket < C_EXAMINED_BUCKET_COUNT; bucket = bucket + 1u) {
        sws(C_EXAMINED_BUCKETS + bucket, 0u);
    }
    r_noting = false;
    // 候选集里每条根（先判它持有的那棵分配记录树）、由记录施加出来的那几版、下一次挂载先施加的那一版（在前两段判完之后才找），
    // 次序照旧，树表那一判同一个调用点
    for (var version_index = 0u; version_index <= c_candidate_count + c_version_count; version_index = version_index + 1u) {
        var base = 0u;
        var tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        if (version_index < c_candidate_count) {
            let root = checker_root_of(sw(C_CANDIDATES + version_index));
            base = root.base;
            tree_table_offset = 36u;
            let held = parse_node_pointer(root.base, 342u);
            if (!held.all_zero && c_seen_node_insert(held)) {
                c_alloc_record_count = 0u;
                if (r_allocation_tree(root.base, 342u)) {
                    judge_allocation_generations_of_the_records();
                }
            }
        } else if (version_index < c_candidate_count + c_version_count) {
            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
        } else {
            let next_version = record_row_of_the_version_the_next_mount_applies_first();
            if (next_version == NONE) {
                break;
            }
            base = checker_record_of_row(next_version).base;
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
    // 两遍同一个调用点：第一遍到走读的引用数为止，存下它的豁免数、算完 F 之下的推迟，第二遍到最新根自己的引用数为止
    var exempted_all: array<u32, 8>;
    for (var exemption_pass = 0u; exemption_pass < 2u; exemption_pass = exemption_pass + 1u) {
        var reference_limit = w_reference_count[0];
        if (exemption_pass == 1u) {
            for (var d = 0u; d < 8u; d = d + 1u) {
                exempted_all[d] = c_exempted[d];
            }
            c_deferred_below_the_floor(newest_root.base);
            reference_limit = c_newest_reference_count;
        }
        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);
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
fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32, txg: vec2<u32>) {
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
        var txg = vec2<u32>(0u, 0u);
        if (version_index < c_candidate_count) {
            let root = checker_root_of(sw(C_CANDIDATES + version_index));
            base = root.base;
            txg = root.txg;
        } else {
            let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
            base = record.base;
            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
            txg = record.txg;
        }
        judge_first_data_unit_of_a_version(base, tree_table_offset, txg);
    }
}

// ───────────────────────── 删除态扫描（walk.rs judge_deletion_state_of_the_versions）与 I-9.6 墓碑那一半 ─────────────────────────
// I-9.5、I-3.4、I-4.7、I-4.6、I-9.24 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。CPU 记成射程里判不了的那几格
// （树表读不出、条目宽不到 200、deleted_inodes 的头 ID 配不上恰好一条 inode 树、队列根读不出或形状不对、inode 树 / extent 树 / 记账树读不全、
// 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
// I-4.4 第一版报不适用（check_pool_image 恒报），内核不置它那一位。

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

// 这一版的一条 inode 记录记进 D_INODE_RECORDS：同一个号已有一行就盖掉（walk.rs 是按号插入的 BTreeMap，后读到的盖掉先读到的）；表满交回 false
fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>, object_birth: vec2<u32>) -> bool {
    let existing = d_inode_record_row_of(inode);
    if (existing != NONE) {
        sws(existing + 2u, nlink);
        sws(existing + 3u, size_in_bytes.x);
        sws(existing + 4u, size_in_bytes.y);
        sws(existing + 5u, object_birth.x);
        sws(existing + 6u, object_birth.y);
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

// 这一版 inode 记录表里号 inode 的那一行（没有交回 NONE）
fn d_inode_record_row_of(inode: vec2<u32>) -> u32 {
    let row_index = keyed_first_row_of(d_inode_record_table(), d_inode_record_count, d_inode_records_are_indexed,
        vec4<u32>(inode.x, inode.y, 0u, 0u));
    if (row_index == NONE) {
        return NONE;
    }
    return D_INODE_RECORDS + row_index * D_INODE_RECORD_WORDS;
}

// 这一版 extent 树里号 inode 的数据单元数（没有的号是 0）
fn d_data_units_of_inode(inode: vec2<u32>) -> u32 {
    for (var i = 0u; i < d_data_unit_inode_count; i = i + 1u) {
        let row = D_DATA_UNIT_COUNTS + i * D_DATA_UNIT_COUNT_WORDS;
        if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
            return sw(row + 2u);
        }
    }
    return 0u;
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
    let root = r_read_index_node(root_pointer);
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
            let container = parse_packed_unit(read_unit_via_locations(child_pointer, DATA_UNIT_BYTES));
            if (!container.ok || container.record_type != DELETION_PACKED_TYPE_INODE
                || container.record_width < DELETION_INODE_RECORD_BYTES) {
                return false;
            }
            for (var r = 0u; r < container.record_count; r = r + 1u) {
                let record = packed_record_offset(container, r);
                if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + INODE_RECORD_NLINK_OFFSET),
                    rd64(container.base, record + INODE_RECORD_SIZE_OFFSET), rd64(container.base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET))) {
                    return false;
                }
            }
        } else if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
            if (child_pointer.all_zero) {
                return false;
            }
            let child = r_read_index_node(child_pointer);
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
        let n = r_read_index_node(p);
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
        read.queue = r_read_index_node(queue_pointer);
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
// 数每个号的数据单元判 I-4.7（对象取丙）、I-4.6（取乙）
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
    // 配不上恰好一棵 inode 树、队列读不出、inode 树的记录读不全：I-9.5、I-4.7、I-4.6 记射程里有一部分没判（walk.rs DELETION_QUEUE_SCAN_INVARIANTS）
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
    // 每个号的数据单元数（walk.rs data_unit_count_of_each_inode_without_judging）：根指针全零的树一个单元都没有
    d_data_unit_inode_count = 0u;
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
        let count_row = D_DATA_UNIT_COUNTS + i * D_DATA_UNIT_COUNT_WORDS;
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
    // I-4.6：每条 nlink = 0 的记录，size 为 0 时数据单元 0 或 1 个，size > 0 时 ⌈size ÷ 净荷容量⌉ 个
    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
        let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
        if (sw(row + 2u) != 0u) {
            continue;
        }
        let data_units = d_data_units_of_inode(vec2<u32>(sw(row), sw(row + 1u)));
        let size_in_bytes = vec2<u32>(sw(row + 3u), sw(row + 4u));
        var matches_the_size = data_units <= 1u;
        if (!is_zero64(size_in_bytes)) {
            let division = divmod64(size_in_bytes, u64_of(DATA_UNIT_PAYLOAD_CAPACITY));
            var data_units_the_size_asks_for = division.quotient;
            if (!is_zero64(division.remainder)) {
                data_units_the_size_asks_for = add64(data_units_the_size_asks_for, u64_of(1u));
            }
            matches_the_size = eq64(u64_of(data_units), data_units_the_size_asks_for);
        }
        judge(INVARIANT_I_4_6, matches_the_size);
    }
}

// 删除态扫描判不全时记射程里有一部分没判的三组（walk.rs judge_deletion_state_of_the_versions 与 judge_deletion_queue_of_a_version）：
// 整张树表扫不了是五条；队列配不上 inode 树、读不全是 DELETION_QUEUE_SCAN_INVARIANTS 那三条；extent 树配不上、读不全是 I-4.7 与 I-4.6
fn d_mark_the_inodes_being_deleted_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_4_7);
    mark_part_of_the_range_not_judged(INVARIANT_I_4_6);
}

fn d_mark_the_deletion_queue_scan_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_9_5);
    d_mark_the_inodes_being_deleted_not_judged();
}

fn d_mark_the_deletion_scan_not_judged() {
    d_mark_the_deletion_queue_scan_not_judged();
    mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_24);
}

// walk.rs judge_deletion_state_of_the_versions 的一版：树表指针在 (base, offset)。树表 0 条的不判；指针全零、树表读不出、条目宽不到 200 的
// 判不了（记射程里有一部分没判）
fn d_judge_deletion_state_of_a_version(base: u32, tree_table_pointer_offset: u32) {
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
}

// 删除态扫描：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针），同 I-5.5 那一份清单
fn judge_deletion_state_of_the_versions() {
    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
        var base = 0u;
        var tree_table_offset = 36u;
        if (version_index < c_candidate_count) {
            base = checker_root_of(sw(C_CANDIDATES + version_index)).base;
        } else {
            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        }
        d_judge_deletion_state_of_a_version(base, tree_table_offset);
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
        let n = r_read_index_node(parse_node_pointer(sw(row), sw(row + 1u)));
        if (!n.ok || n.level == 0u || n.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
            return;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            let child_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
            let entry_type = rd16(n.base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
            if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
                let child = r_read_index_node(child_pointer);
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
                let container = parse_packed_unit(read_unit_via_locations(child_pointer, DATA_UNIT_BYTES));
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
    let root = r_read_index_node(root_pointer);
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
            let child = r_read_index_node(child_pointer);
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
            let container = parse_packed_unit(read_unit_via_locations(child_pointer, DATA_UNIT_BYTES));
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

fn d_lineage_rollback_txg(lineage_index: u32) -> vec2<u32> {
    let row = D_LINEAGE_RECORDS + lineage_index * D_LINEAGE_RECORD_WORDS;
    return vec2<u32>(sw(row), sw(row + 1u));
}

fn d_lineage_parent_txg(lineage_index: u32) -> vec2<u32> {
    let row = D_LINEAGE_RECORDS + lineage_index * D_LINEAGE_RECORD_WORDS;
    return vec2<u32>(sw(row + 2u), sw(row + 3u));
}

fn d_lineage_inode_number_watermark_rolled_back_from(lineage_index: u32) -> vec2<u32> {
    let row = D_LINEAGE_RECORDS + lineage_index * D_LINEAGE_RECORD_WORDS;
    return vec2<u32>(sw(row + 4u), sw(row + 5u));
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
        let n = r_read_index_node(p);
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

// ④ 那一判（谱系已读进 D_LINEAGE_RECORDS、接得成链）：每条谱系记录 (T_rb, T_old, H_cur)，代段 < T_rb 的台阶里最大的 H 等于 H_cur；
// 没有这样的台阶判红；H_cur 为 0 时等式不成立（台阶的水位至少是 1），同 walk.rs 不另写一判
fn d_judge_the_lineage_against_the_inode_number_watermark_steps() {
    for (var lineage_index = 0u; lineage_index < d_lineage_record_count; lineage_index = lineage_index + 1u) {
        let rollback_txg = d_lineage_rollback_txg(lineage_index);
        let watermark_rolled_back_from = d_lineage_inode_number_watermark_rolled_back_from(lineage_index);
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

// 这一版的谱系（walk.rs judge_tombstones_and_the_lineage_of_a_version 开头与 lineage_chain_of_a_version 判链之前那两步）：树表 tt 里谱系树（种类 11）
// 恰好一条、整棵读得出、记录接得成一条链（每条 T_old < T_rb、没有两条同一个 T_rb）才交回 true，谱系记录按 T_rb 从大到小排进 D_LINEAGE_RECORDS；
// 否则交回 false（这一版要谱系的那几判判不了）
fn d_read_the_lineage_of_a_version(tt: Node, version_txg: vec2<u32>) -> bool {
    d_lineage_record_count = 0u;
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
        let base = d_container_base(container_index);
        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
            if (d_lineage_record_count >= D_MAX_LINEAGE_RECORDS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (58u << 8u);
                return false;
            }
            let record = d_container_record_offset(container_index, record_index, DELETION_LINEAGE_RECORD_BYTES);
            let rollback_txg = rd64(base, record + DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
            let parent_txg = rd64(base, record + DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET);
            let watermark_rolled_back_from = rd64(base, record + DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET);
            let row = D_LINEAGE_RECORDS + d_lineage_record_count * D_LINEAGE_RECORD_WORDS;
            sws(row, rollback_txg.x);
            sws(row + 1u, rollback_txg.y);
            sws(row + 2u, parent_txg.x);
            sws(row + 3u, parent_txg.y);
            sws(row + 4u, watermark_rolled_back_from.x);
            sws(row + 5u, watermark_rolled_back_from.y);
            d_lineage_record_count = d_lineage_record_count + 1u;
            if (!d_lineage_names_the_parent_of_this_version && eq64(rollback_txg, version_txg)) {
                d_lineage_names_the_parent_of_this_version = true;
                d_parent_txg_named_by_the_lineage = parent_txg;
            }
        }
    }
    d_lineage_records_are_readable = true;
    // 按 T_rb 从大到小排（选择排序：条数不过 D_MAX_LINEAGE_RECORDS）
    for (var sorted = 0u; sorted < d_lineage_record_count; sorted = sorted + 1u) {
        var latest = sorted;
        for (var candidate = sorted + 1u; candidate < d_lineage_record_count; candidate = candidate + 1u) {
            if (lt64(d_lineage_rollback_txg(latest), d_lineage_rollback_txg(candidate))) {
                latest = candidate;
            }
        }
        if (latest != sorted) {
            let sorted_row = D_LINEAGE_RECORDS + sorted * D_LINEAGE_RECORD_WORDS;
            let latest_row = D_LINEAGE_RECORDS + latest * D_LINEAGE_RECORD_WORDS;
            for (var word = 0u; word < D_LINEAGE_RECORD_WORDS; word = word + 1u) {
                let kept = sw(sorted_row + word);
                sws(sorted_row + word, sw(latest_row + word));
                sws(latest_row + word, kept);
            }
        }
    }
    for (var lineage_index = 0u; lineage_index < d_lineage_record_count; lineage_index = lineage_index + 1u) {
        if (!lt64(d_lineage_parent_txg(lineage_index), d_lineage_rollback_txg(lineage_index))) {
            return false;
        }
        if (lineage_index > 0u && eq64(d_lineage_rollback_txg(lineage_index - 1u), d_lineage_rollback_txg(lineage_index))) {
            return false;
        }
    }
    return true;
}

// 「在谱系上」（walk.rs LineageChainOfAVersion::holds_txg 与 lineage_chain_of_a_version 的按段走）：从这一版自己的 txg 起，谱系记录按 T_rb 从大到小过一遍，
// T_rb 大于此刻走到的 txg 的跳过，否则 [T_rb, 此刻] 整段在链上、跳到 T_old；最后 [0, 此刻] 也在链上
fn d_txg_is_on_the_lineage(txg: vec2<u32>, version_txg: vec2<u32>) -> bool {
    var txg_walked_to = version_txg;
    for (var lineage_index = 0u; lineage_index < d_lineage_record_count; lineage_index = lineage_index + 1u) {
        let rollback_txg = d_lineage_rollback_txg(lineage_index);
        if (lt64(txg_walked_to, rollback_txg)) {
            continue;
        }
        if (le64(rollback_txg, txg) && le64(txg, txg_walked_to)) {
            return true;
        }
        txg_walked_to = d_lineage_parent_txg(lineage_index);
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
        let node = r_read_index_node(pointer);
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
    txg: vec2<u32>,
};

fn d_version_of_the_scan(version_index: u32) -> VersionOfTheDeletionScan {
    var version: VersionOfTheDeletionScan;
    if (version_index < c_candidate_count) {
        let root = checker_root_of(sw(C_CANDIDATES + version_index));
        version.base = root.base;
        version.tree_table_offset = 36u;
        version.txg = root.txg;
    } else {
        let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
        version.base = record.base;
        version.tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
        version.txg = record.txg;
    }
    return version;
}

// I-9.35 的父版在版本清单里是哪一份：walk.rs 按 (树表指针第一条位置条目的盘、槽、校验和, txg) 去重、只记第一次（树表指针全零的版不去重，每次都记一版），
// 去过重之后 txg 等于 txg 的恰好一份才比（没有不判、不止一份报判不了）。is_single 为真时 pointer 是那一份的树表指针
struct SingleVersionAtATxg {
    is_single: bool,
    // 去过重之后几份（0：没有父版，不判）
    version_count: u32,
    pointer: Ptr,
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
fn d_judge_the_retirement_records_and_their_completeness(tt: Node, entry: u32, version_txg: vec2<u32>, lineage_holds: bool,
    parent: ParentVersionToCompare) {
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
    for (var reading = 0u; reading < readings; reading = reading + 1u) {
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
fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer_offset: u32, version_txg: vec2<u32>, is_the_newest_root: bool) {
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
            d_judge_the_retirement_records_and_their_completeness(tt, entry, version_txg, lineage_holds, parent);
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
        var base = 0u;
        var tree_table_offset = 36u;
        var txg = vec2<u32>(0u, 0u);
        var is_the_newest_root = false;
        if (version_index < c_candidate_count) {
            let root = checker_root_of(sw(C_CANDIDATES + version_index));
            base = root.base;
            txg = root.txg;
            is_the_newest_root = sw(C_CANDIDATES + version_index) == c_newest_index;
        } else {
            let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
            base = record.base;
            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
            txg = record.txg;
        }
        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg, is_the_newest_root);
    }
}

fn scan_stage(fsid_low: vec2<u32>) {
    judge_merged_version_total_order(fsid_low);
    judge_release_generation_and_tree_table_birth();
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
