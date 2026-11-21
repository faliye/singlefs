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

// walk.rs entry_is_pending_release：这条条目（头、纪元、birth）在某个未完成销毁意图的待放集里——头是活头、纪元 ∈ [D.S.txg, D 之后第一个活快照的 S.txg，
// 下一侧是活头时取被判这一版的 txg version_txg)、birth > D 的前驱（判决 m4-c145-destruction-bound-r1 第二节 B2）
fn s_entry_is_pending_release(head: vec2<u32>, epoch: vec2<u32>, birth: vec2<u32>, head_inode_tree: vec2<u32>, live_count: u32, destroyed_count: u32,
    version_txg: vec2<u32>) -> bool {
    if (!eq64(head, head_inode_tree)) {
        return false;
    }
    for (var destroyed = 0u; destroyed < destroyed_count; destroyed = destroyed + 1u) {
        let bound = s_next_live_side_upper_bound_after(live_count, s_snapshot_being_destroyed_txg[destroyed], s_snapshot_being_destroyed_tree[destroyed]);
        let upper_bound = select(version_txg, bound.txg, bound.found);
        if (le64(s_snapshot_being_destroyed_txg[destroyed], epoch) && lt64(epoch, upper_bound)
            && lt64(s_snapshot_being_destroyed_previous_txg[destroyed], birth)) {
            return true;
        }
    }
    return false;
}

// walk.rs previous_snapshot_chain_of_the_sides_is_contiguous：活快照（按（S.txg, 树 ID）升序）与活头的前驱链首尾相接——每个活快照的前驱等于前一个活快照的
// S.txg，活头的前驱等于最后一个活快照的 S.txg（一个活快照都没有时只有活头一侧，恒相接）
fn s_previous_snapshot_chain_of_the_sides_is_contiguous(head_previous_snapshot_txg: vec2<u32>, live_count: u32) -> bool {
    for (var live = 1u; live < live_count; live = live + 1u) {
        if (!eq64(s_live_snapshot_previous_txg[live], s_live_snapshot_txg[live - 1u])) {
            return false;
        }
    }
    return live_count == 0u || eq64(head_previous_snapshot_txg, s_live_snapshot_txg[live_count - 1u]);
}

fn s_mark_the_deadlist_of_a_version_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_3_5);
    mark_part_of_the_range_not_judged(INVARIANT_I_3_6);
}

// walk.rs judge_the_deadlist_against_the_head_and_the_live_snapshots：
// ④ deadlist 每一条钉的单元活头都不引用；逆向：每一条钉的单元有一个活快照引用、或在待放集里；I-3.6 每一条的 birth 不大于它归属那一侧的前驱（待放的豁免，
// 头不是活头、纪元落不进恰好一侧而前驱链不首尾相接的那一条判不了；前驱链首尾相接而落不进任何一侧的判红，待放的豁免）；I-3.6 每一条的映射 key 在这一版
// 装着的中央映射里查得到（C300，映射读不全时这一格判不了）；③ 活快照引用而活头不引用的每个单元，deadlist 里恰好一条钉它、头是活头、纪元落在引用它的最晚那个
// 活快照的 S.txg 与链上下一侧的上界之间。一条条目都没有时 I-3.6 判一次、成立
fn s_judge_the_deadlist_against_the_head_and_the_live_snapshots(head_inode_tree: vec2<u32>, head_previous_snapshot_txg: vec2<u32>, live_count: u32,
    destroyed_count: u32, version_txg: vec2<u32>) {
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
    } else if (!version_mapping_read_in_full) {
        mark_part_of_the_range_not_judged(INVARIANT_I_3_6);
    }
    let chain_is_contiguous = s_previous_snapshot_chain_of_the_sides_is_contiguous(head_previous_snapshot_txg, live_count);
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
        let is_pending_release = s_entry_is_pending_release(head, epoch, key.birth_txg, head_inode_tree, live_count, destroyed_count, version_txg);
        if (version_mapping_read_in_full) {
            judge(INVARIANT_I_3_6, version_mapping_first_row_of(key) != NONE);
        }
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
        if (owning_sides == 0u && chain_is_contiguous) {
            judge(INVARIANT_I_3_6, is_pending_release);
            continue;
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
fn s_judge_the_snapshots_and_the_deadlist_of_a_tree_table(tt: Node, version_txg: vec2<u32>) {
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
        live_count, destroyed_count, version_txg);
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
            s_judge_the_snapshots_and_the_deadlist_of_a_tree_table(tt, version.txg);
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

// 借的那一段装得下（解析内核时判，不进展开量）；上界是 ⑯ 的起点（crash_judge_checker_common.wgsl：⑯ 那三张扫描内核要读，借的这一段不越进去）
const_assert G_KEY_GROUP_BUCKETS + G_KEY_GROUP_BUCKET_COUNT <= R_RECORDS;

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

// C_ALLOC_RECORDS 一行第八个字低两位在 I-3.15 里的记号：对不上映射的、它那一组已判过的（高位那一位是隔离记号，C_ALLOC_RECORD_QUARANTINE_MARK_BIT）
const G_RECORD_OFF_THE_CENTRAL_MAPPING: u32 = 1u;
const G_RECORD_OFF_THE_CENTRAL_MAPPING_JUDGED: u32 = 2u;
const G_RECORD_MARKER_BITS: u32 = 3u;

// 一行分配记录在 I-3.15 那一遍里的记号（第八个字的低两位）
fn g_record_marker(record: u32) -> u32 {
    return sw(record + 7u) & G_RECORD_MARKER_BITS;
}

// 换掉一行分配记录在 I-3.15 那一遍里的记号，隔离记号那一位不动
fn g_set_the_record_marker(record: u32, marker: u32) {
    sws(record + 7u, (sw(record + 7u) & ~G_RECORD_MARKER_BITS) | marker);
}

// 一版（它的分配记录在 C_ALLOC_RECORDS，中央映射在 C_VERSION_MAPPING_ROWS，都读全了）上的 I-3.15。对不上的记录按单元（起点槽、跨度）归组，
// 组按 (起点槽, 跨度) 升序判（walk.rs 那张 BTreeMap 的次序）；这一版这一单元每块盘上的未释放记录都带隔离记号的那一组不判
// （walk.rs every_copy_of_the_unit_carries_the_quarantine_mark：只认记号、不读副本，D19（块指针的结构与宽度预算） 已定项 5），否则组里每条判红
fn g_judge_the_unreleased_records_of_the_version_against_its_central_mapping(version: VersionWithItsRootPointersOfTheScan) {
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
        g_set_the_record_marker(record, 0u);
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
        g_set_the_record_marker(record, G_RECORD_OFF_THE_CENTRAL_MAPPING);
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
            if (g_record_marker(record) != G_RECORD_OFF_THE_CENTRAL_MAPPING) {
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
        // 隔离的单元只认记号：这一版这一单元（起点槽、跨度）每块盘上的未释放记录都带隔离记号（至少一条）就不判，不读副本
        var unreleased_copies = 0u;
        var every_unreleased_copy_is_marked = true;
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            if (sw(record + 6u) == 0u && eq64(vec2<u32>(sw(record + 1u), sw(record + 2u)), group_slot) && sw(record + 3u) == group_span) {
                unreleased_copies = unreleased_copies + 1u;
                if ((sw(record + 7u) & C_ALLOC_RECORD_QUARANTINE_MARK_BIT) == 0u) {
                    every_unreleased_copy_is_marked = false;
                }
            }
        }
        let quarantined = unreleased_copies > 0u && every_unreleased_copy_is_marked;
        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
            if (g_record_marker(record) == G_RECORD_OFF_THE_CENTRAL_MAPPING && eq64(vec2<u32>(sw(record + 1u), sw(record + 2u)), group_slot)
                && sw(record + 3u) == group_span) {
                g_set_the_record_marker(record, G_RECORD_OFF_THE_CENTRAL_MAPPING_JUDGED);
                if (!quarantined) {
                    judge(INVARIANT_I_3_15, false);
                }
            }
        }
    }
}

// ── I-3.18（walk.rs judge_allocation_record_spans_against_the_classes_of_the_units_they_cover）

// 一版（它的分配记录在 C_ALLOC_RECORDS，读全了）上每条未释放的记录：起点槽上的单元头用得了（同 I-3.15 那一判的读法：magic、类 1 / 2 / 3、
// 头校验和过）时，跨度要等于那一类占的槽数（码 1、码 3 两槽，码 2 一槽；walk.rs slots_spanned_by_a_unit_of_the_class）；头用不了的没有对象。
// 不要这一版的中央映射读全（walk.rs 那一判不读映射）
fn g_judge_the_spans_of_the_unreleased_records_of_the_version_against_the_classes_of_their_units() {
    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
        let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
        if (sw(record + 6u) != 0u) {
            continue;
        }
        let slots_of_the_class = c_slots_of_the_class_of_the_unit_whose_usable_header_starts_at(sw(record),
            vec2<u32>(sw(record + 1u), sw(record + 2u)));
        if (slots_of_the_class == 0u) {
            continue;
        }
        judge(INVARIANT_I_3_18, sw(record + 3u) == slots_of_the_class);
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
            mark_part_of_the_range_not_judged(INVARIANT_I_3_18);
            continue;
        }
        let records_read_in_full = g_allocation_records_of_the_version(version.base, version.allocation_root_offset, tt);
        g_judge_the_compaction_intents_of_the_version(version, tt, records_read_in_full);
        if (g_root_pointers_seen_in_an_earlier_version(version_index)) {
            continue;
        }
        // I-3.18 只要这一版的分配记录读全（walk.rs 那一判不读映射）；几版照抄同一棵分配记录树时 walk.rs 只判一次，这里按 I-3.15 那四样指针去重、
        // 照抄的那几版可能再判一遍：读的是同一批记录、同一批单元头，判到与违例都一样
        if (!records_read_in_full) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_15);
            mark_part_of_the_range_not_judged(INVARIANT_I_3_18);
            continue;
        }
        g_judge_the_spans_of_the_unreleased_records_of_the_version_against_the_classes_of_their_units();
        if (!mapping_read_in_full) {
            mark_part_of_the_range_not_judged(INVARIANT_I_3_15);
            continue;
        }
        g_judge_the_unreleased_records_of_the_version_against_its_central_mapping(version);
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

// 扫描那三组判各一趟（主入口 main 判前两判，allocation_records_disjoint_main 判 I-5.4，allocation_generations_through_the_rollback_floor_raises_main
// 判之后那七判），记账那一判拆成头、分片、决断、收尾四趟（accounting_*_main），各一次提交：一个状态上单次派活不越过驱动掐通道的时限。
// 几趟之间只靠草稿区交接：违例位与判到位按位或、各趟接着前一趟记；判不了的原因存在交接段（C_ACCOUNTING_CARRY_UNDECIDABLE），后写的盖前写的，
// 结论里「走到第几步」那一字与判不了的原因只在收尾那一趟写一次（中间的趟一个都不写，不然半截就记成走完）

fn scan_stage_of_the_version_order_and_the_tree_identifiers(fsid_low: vec2<u32>) {
    judge_merged_version_total_order(fsid_low);
    judge_tree_identifier_birth_txg_across_every_readable_root();
}

fn scan_stage_from_the_allocation_generations_through_the_rollback_floor_raises() {
    judge_allocation_generations_against_unit_births();
    judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
    judge_deletion_state_of_the_versions();
    judge_inode_number_watermark_steps_against_the_watermark_of_the_newest_root();
    judge_tombstones_and_the_lineage_of_the_versions();
    judge_inode_number_watermark_above_every_published_tombstone();
    judge_rollback_floor_raises_against_their_ceilings();
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
    scan_stage_of_the_version_order_and_the_tree_identifiers(fsid_low);
    store_violated(out);
    store_judged(out);
    sws(C_ACCOUNTING_CARRY_UNDECIDABLE, undecidable);
}

// 扫描那一截 I-5.4 那一趟（接着主入口那一趟记）
@compute @workgroup_size(64)
fn allocation_records_disjoint_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    undecidable = sw(C_ACCOUNTING_CARRY_UNDECIDABLE);
    judge_allocation_records_disjoint();
    store_violated(out);
    store_judged(out);
    sws(C_ACCOUNTING_CARRY_UNDECIDABLE, undecidable);
}

// 扫描那一截 I-5.4 之后那七判那一趟
@compute @workgroup_size(64)
fn allocation_generations_through_the_rollback_floor_raises_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    undecidable = sw(C_ACCOUNTING_CARRY_UNDECIDABLE);
    scan_stage_from_the_allocation_generations_through_the_rollback_floor_raises();
    store_violated(out);
    store_judged(out);
    sws(C_ACCOUNTING_CARRY_UNDECIDABLE, undecidable);
}

// 记账头那一趟
@compute @workgroup_size(64)
fn accounting_head_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    undecidable = sw(C_ACCOUNTING_CARRY_UNDECIDABLE);
    accounting_head();
    store_violated(out);
    store_judged(out);
    sws(C_ACCOUNTING_CARRY_UNDECIDABLE, undecidable);
}

// 记账分片那一趟（每一片一次派活，派活参数在第 4 号绑定）
@compute @workgroup_size(64)
fn accounting_slice_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    undecidable = sw(C_ACCOUNTING_CARRY_UNDECIDABLE);
    accounting_carry_load();
    accounting_slice();
    store_violated(out);
    store_judged(out);
    sws(C_ACCOUNTING_CARRY_UNDECIDABLE, undecidable);
}

// 记账决断那一趟（按记录自述那一轮分片罩满之后）
@compute @workgroup_size(64)
fn accounting_decide_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    undecidable = sw(C_ACCOUNTING_CARRY_UNDECIDABLE);
    accounting_carry_load();
    accounting_decide();
    store_violated(out);
    store_judged(out);
    sws(C_ACCOUNTING_CARRY_UNDECIDABLE, undecidable);
}

// 记账收尾那一趟：判完写结论里「走到第几步」那一字（5），判不了时写原因（还没记过时）
@compute @workgroup_size(64)
fn accounting_final_main(@builtin(global_invocation_id) id: vec3<u32>) {
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
    undecidable = sw(C_ACCOUNTING_CARRY_UNDECIDABLE);
    accounting_carry_load();
    accounting_final();
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
