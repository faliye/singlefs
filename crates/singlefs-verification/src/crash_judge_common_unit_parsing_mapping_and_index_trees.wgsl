// ───────────────────────── 单元解析 ─────────────────────────

struct Node {
    ok: bool,
    base: u32,
    tree: vec2<u32>,
    level: u32,
    key_width: u32,
    smallest_offset: u32,
    largest_offset: u32,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    birth_sequence: u32,
    entry_count: u32,
    entry_width: u32,
    entries_start: u32,
};
fn bad_node() -> Node {
    var n: Node;
    n.ok = false;
    n.base = 0u;
    n.tree = vec2<u32>(0u, 0u);
    n.level = 0u;
    n.key_width = 0u;
    n.smallest_offset = 0u;
    n.largest_offset = 0u;
    n.birth_txg = vec2<u32>(0u, 0u);
    n.fsid = vec2<u32>(0u, 0u);
    n.instance = 0u;
    n.birth_sequence = 0u;
    n.entry_count = 0u;
    n.entry_width = 0u;
    n.entries_start = 0u;
    return n;
}
// 码 2 节点头里条目那一段（unit::parse_index_node 的头布局）：key 宽在偏移 51，头到 86 + 2 × key 宽为止，条数与条目宽是头的最后 4 字节，
// 条目从头之后 29 字节起。parse_index_node 与中央映射叶表（mapping_leaf_at）都照这一处读
struct IndexNodeEntries {
    key_width: u32,
    entry_count: u32,
    entry_width: u32,
    entries_start: u32,
};
fn index_node_entries_of_the_header(base: u32) -> IndexNodeEntries {
    let key_width = rd8(base, 51u);
    let header_end = 86u + 2u * key_width;
    return IndexNodeEntries(key_width, rd16(base, header_end - 4u), rd16(base, header_end - 2u), header_end + 29u);
}
// unit::parse_index_node（读 16 KiB）
fn parse_index_node(s: Slice) -> Node {
    if (s.version == NONE || s.length < NODE_BYTES) {
        return bad_node();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 2u || rd8(base, 7u) != 0u) {
        return bad_node();
    }
    if ((s.c2 & 11u) != 11u) {
        return bad_node();
    }
    let entries = index_node_entries_of_the_header(base);
    let key_width = entries.key_width;
    let declared_length = rd16(base, 8u);
    let entry_count = entries.entry_count;
    let entry_width = entries.entry_width;
    if (entry_count * entry_width != declared_length) {
        return bad_node();
    }
    if (entry_width == 0u && entry_count != 0u) {
        return bad_node();
    }
    if (entry_width < key_width) {
        return bad_node();
    }
    let entries_start = entries.entries_start;
    if (entries_start + declared_length > NODE_BYTES) {
        return bad_node();
    }
    var n: Node;
    n.ok = true;
    n.base = base;
    n.tree = rd64(base, 42u);
    n.level = rd8(base, 50u);
    n.key_width = key_width;
    n.smallest_offset = 52u;
    n.largest_offset = 52u + key_width;
    n.birth_txg = rd64(base, 52u + 2u * key_width);
    n.fsid = rd64(base, 60u + 2u * key_width);
    n.instance = rd32(base, 68u + 2u * key_width);
    n.birth_sequence = rd32(base, 72u + 2u * key_width);
    n.entry_count = entry_count;
    n.entry_width = entry_width;
    n.entries_start = entries_start;
    return n;
}
fn entry_offset(n: Node, index: u32) -> u32 {
    return n.entries_start + index * n.entry_width;
}
struct Packed {
    ok: bool,
    base: u32,
    birth_tree: vec2<u32>,
    record_type: u32,
    container: vec2<u32>,
    container_birth: vec2<u32>,
    record_count: u32,
    record_width: u32,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
    birth_sequence: u32,
};
fn bad_packed() -> Packed {
    var p: Packed;
    p.ok = false;
    p.base = 0u;
    p.birth_tree = vec2<u32>(0u, 0u);
    p.record_type = 0u;
    p.container = vec2<u32>(0u, 0u);
    p.container_birth = vec2<u32>(0u, 0u);
    p.record_count = 0u;
    p.record_width = 0u;
    p.birth_txg = vec2<u32>(0u, 0u);
    p.fsid = vec2<u32>(0u, 0u);
    p.instance = 0u;
    p.transaction = vec2<u32>(0u, 0u);
    p.birth_sequence = 0u;
    return p;
}
// unit::parse_packed_unit（读 32 KiB）
fn parse_packed_unit(s: Slice) -> Packed {
    if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
        return bad_packed();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 3u || rd8(base, 7u) != 0u) {
        return bad_packed();
    }
    if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 3u) {
        return bad_packed();
    }
    let declared_length = rd16(base, 8u);
    let record_count = rd16(base, 69u);
    let record_width = rd16(base, 71u);
    if (record_count * record_width != declared_length) {
        return bad_packed();
    }
    if (record_width == 0u && record_count != 0u) {
        return bad_packed();
    }
    if (136u + declared_length > DATA_UNIT_BYTES) {
        return bad_packed();
    }
    var p: Packed;
    p.ok = true;
    p.base = base;
    p.birth_tree = rd64(base, 43u);
    p.record_type = rd16(base, 51u);
    p.container = rd64(base, 53u);
    p.container_birth = rd64(base, 61u);
    p.record_count = record_count;
    p.record_width = record_width;
    p.birth_txg = rd64(base, 73u);
    p.fsid = rd64(base, 81u);
    p.instance = rd32(base, 93u);
    p.transaction = rd48(base, 97u);
    p.birth_sequence = rd32(base, 103u);
    return p;
}
fn packed_record_offset(p: Packed, index: u32) -> u32 {
    return 136u + index * p.record_width;
}
struct DataUnit {
    ok: bool,
    base: u32,
    tree: vec2<u32>,
    object: vec2<u32>,
    object_birth: vec2<u32>,
    anchor_offset: vec2<u32>,
    birth_txg: vec2<u32>,
    fsid: vec2<u32>,
    instance: u32,
    transaction: vec2<u32>,
    declared_length: u32,
    padding_zero: bool,
    payload_crc: u32,
};
fn bad_data_unit() -> DataUnit {
    var d: DataUnit;
    d.ok = false;
    d.base = 0u;
    d.tree = vec2<u32>(0u, 0u);
    d.object = vec2<u32>(0u, 0u);
    d.object_birth = vec2<u32>(0u, 0u);
    d.anchor_offset = vec2<u32>(0u, 0u);
    d.birth_txg = vec2<u32>(0u, 0u);
    d.fsid = vec2<u32>(0u, 0u);
    d.instance = 0u;
    d.transaction = vec2<u32>(0u, 0u);
    d.declared_length = 0u;
    d.padding_zero = false;
    d.payload_crc = 0u;
    return d;
}
// unit::parse_data_unit（读 32 KiB）
fn parse_data_unit(s: Slice) -> DataUnit {
    if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
        return bad_data_unit();
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSU || rd16(base, 4u) != 1u || rd8(base, 6u) != 1u || rd8(base, 7u) != 0u) {
        return bad_data_unit();
    }
    if ((s.c2 & 11u) != 11u || rd8(base, 42u) != 1u) {
        return bad_data_unit();
    }
    let declared_length = rd16(base, 8u);
    if (134u + declared_length > DATA_UNIT_BYTES) {
        return bad_data_unit();
    }
    var d: DataUnit;
    d.ok = true;
    d.base = base;
    d.tree = rd64(base, 43u);
    d.object = rd64(base, 51u);
    d.object_birth = rd64(base, 59u);
    d.anchor_offset = rd64(base, 67u);
    d.birth_txg = rd64(base, 75u);
    d.fsid = rd64(base, 83u);
    d.instance = rd32(base, 91u);
    d.transaction = rd48(base, 95u);
    d.declared_length = declared_length;
    d.padding_zero = (s.c2 & 16u) != 0u;
    d.payload_crc = rd32(base, 101u);
    return d;
}

// 两个多项式在 CRC-32C 的生成多项式下相乘取余（反射位序，位 31 是 x 的 0 次）：逐位看乘数，被乘数每过一位乘一次 x。循环恰好 32 圈、没有提前出口
fn crc32c_product_modulo_the_polynomial(multiplier: u32, multiplicand: u32) -> u32 {
    var product = 0u;
    var multiplicand_times_x_to_the_bit = multiplicand;
    for (var bit = 0u; bit < 32u; bit = bit + 1u) {
        if ((multiplier & (0x80000000u >> bit)) != 0u) {
            product = product ^ multiplicand_times_x_to_the_bit;
        }
        let shifted_out_a_one = (multiplicand_times_x_to_the_bit & 1u) != 0u;
        multiplicand_times_x_to_the_bit = multiplicand_times_x_to_the_bit >> 1u;
        if (shifted_out_a_one) {
            multiplicand_times_x_to_the_bit = multiplicand_times_x_to_the_bit ^ CRC32C_REFLECTED_POLYNOMIAL;
        }
    }
    return product;
}
// byte_count 个 0 字节的 CRC-32C（初值与末尾异或都是全 1）：寄存器读进 n 个 0 字节等于乘 x 的 8n 次再取余，x 的 8n 次按 n 的二进制位平方连乘，
// 不逐字节走（洞里一个单元的载荷区三万多字节，每个洞的位置逐字节走太慢）。循环圈数是 byte_count 的二进制位数（至多 32）
fn crc32c_of_zero_bytes(byte_count: u32) -> u32 {
    var x_to_the_eight_times_this_power_of_two = 0x00800000u;
    var x_to_the_eight_times_the_bytes_seen = 0x80000000u;
    var bytes_left = byte_count;
    loop {
        if (bytes_left == 0u) {
            break;
        }
        if ((bytes_left & 1u) != 0u) {
            x_to_the_eight_times_the_bytes_seen = crc32c_product_modulo_the_polynomial(x_to_the_eight_times_this_power_of_two, x_to_the_eight_times_the_bytes_seen);
        }
        x_to_the_eight_times_this_power_of_two = crc32c_product_modulo_the_polynomial(x_to_the_eight_times_this_power_of_two, x_to_the_eight_times_this_power_of_two);
        bytes_left = bytes_left >> 1u;
    }
    return ~crc32c_product_modulo_the_polynomial(x_to_the_eight_times_the_bytes_seen, 0xFFFFFFFFu);
}

// ───────────────────────── 中央映射 ─────────────────────────

struct Lookup {
    found: bool,
    failed: bool,
    p: Ptr,
};
// 中央映射叶表（R_MAPPING_LEAVES）第 leaf_index 行那片叶：字节基与头里条目那一段
struct MappingLeaf {
    base: u32,
    entries: IndexNodeEntries,
};
fn mapping_leaf_at(leaf_index: u32) -> MappingLeaf {
    let base = sw(R_MAPPING_LEAVES + leaf_index);
    return MappingLeaf(base, index_node_entries_of_the_header(base));
}
// 中央映射的一把 27 字节 key 拆成五段（类标签 1、出生树 8、出生 txg 8、实例 4、尾段 6，各段小端），次序同 compare_keys 的 schema 1：逐段按值比
struct MappingKeyFields {
    unit_class: u32,
    birth_tree: vec2<u32>,
    birth_txg: vec2<u32>,
    instance: u32,
    tail: vec2<u32>,
};
// (base, key_at) 起那 27 字节按映射 key 的布局拆开（映射叶条目与 deadlist 条目偏移 16 起的那一段都是这个布局）
fn mapping_key_fields_at(base: u32, key_at: u32) -> MappingKeyFields {
    return MappingKeyFields(rd8(base, key_at), rd64(base, key_at + 1u), rd64(base, key_at + 9u), rd32(base, key_at + 17u), rd48(base, key_at + 21u));
}
// 映射叶里 (base, entry_at) 起那条条目的 key 与 probe 比，次序同 compare_keys(1, …)：0 条目在前、1 相等、2 条目在后
fn mapping_entry_key_order_against_the_probe(base: u32, entry_at: u32, probe: MappingKeyFields) -> u32 {
    let entry = mapping_key_fields_at(base, entry_at);
    if (entry.unit_class != probe.unit_class) {
        return select(2u, 0u, entry.unit_class < probe.unit_class);
    }
    if (!eq64(entry.birth_tree, probe.birth_tree)) {
        return select(2u, 0u, lt64(entry.birth_tree, probe.birth_tree));
    }
    if (!eq64(entry.birth_txg, probe.birth_txg)) {
        return select(2u, 0u, lt64(entry.birth_txg, probe.birth_txg));
    }
    if (entry.instance != probe.instance) {
        return select(2u, 0u, entry.instance < probe.instance);
    }
    if (!eq64(entry.tail, probe.tail)) {
        return select(2u, 0u, lt64(entry.tail, probe.tail));
    }
    return 1u;
}
// 中央映射叶表里 key 等于 probe 的那条条目在叶字节里的位置（found 为假是没有）
struct MappingEntryPosition {
    found: bool,
    base: u32,
    offset: u32,
};
// 前提：这一份叶表是一次读通的映射树记下的（mapping_tree_failure_point 为 NONE；只有恢复内核调）。读通就说明叶内条目按 compare_keys(1, …) 严格递增
// （read_code_two_tree 叶那一判），跨叶由分隔 key 不大于孩子最小 key、分隔 key 严格递增且大于前一个孩子的最大 key 连成严格递增（内部节点那几判），
// 叶按深度优先次序记进 R_MAPPING_LEAVES：全表按 key 严格递增、不重，所以先在叶之间二分（比每片的末条，取第一片末条不在 probe 之前的叶）、再在那片叶里二分。
// 读树时零条目的叶不收，每片至少一条。圈数：叶之间至多 ⌈log2(叶数 + 1)⌉，叶内至多 ⌈log2(条数 + 1)⌉
fn mapping_entry_holding_the_key(probe: MappingKeyFields) -> MappingEntryPosition {
    var out = MappingEntryPosition(false, 0u, 0u);
    var low_leaf = 0u;
    var high_leaf = mapping_leaf_count;
    while (low_leaf < high_leaf) {
        let middle_leaf = (low_leaf + high_leaf) >> 1u;
        let leaf = mapping_leaf_at(middle_leaf);
        let last_entry = leaf.entries.entries_start + (leaf.entries.entry_count - 1u) * leaf.entries.entry_width;
        if (mapping_entry_key_order_against_the_probe(leaf.base, last_entry, probe) == 0u) {
            low_leaf = middle_leaf + 1u;
        } else {
            high_leaf = middle_leaf;
        }
    }
    if (low_leaf >= mapping_leaf_count) {
        return out;
    }
    let leaf = mapping_leaf_at(low_leaf);
    var low_entry = 0u;
    var high_entry = leaf.entries.entry_count;
    while (low_entry < high_entry) {
        let middle_entry = (low_entry + high_entry) >> 1u;
        let entry_at = leaf.entries.entries_start + middle_entry * leaf.entries.entry_width;
        let order = mapping_entry_key_order_against_the_probe(leaf.base, entry_at, probe);
        if (order == 1u) {
            out = MappingEntryPosition(true, leaf.base, entry_at);
            return out;
        }
        if (order == 0u) {
            low_entry = middle_entry + 1u;
        } else {
            high_entry = middle_entry;
        }
    }
    return out;
}
// 在映射叶条目里找一个 27 字节 key：key 的类标签是 key_class，出生树与出生 txg 是 (key_base, key_offset + 34) 起 16 字节，实例是 key_offset + 78 起 4 字节，
// 尾段是 key_tail（事务那种 48 位全比；别的取低 32 位、高 16 位为 0）。叶表有一片条目窄于 55 就判红（不论查的 key 在哪）。
// 前提同 mapping_entry_holding_the_key：只在映射树读通之后调（调用点都先判 mapping_tree_failure_point 为 NONE）
fn mapping_lookup(key_base: u32, key_offset: u32, key_class: u32, key_tail: vec2<u32>, key_tail_is_transaction: bool) -> Lookup {
    var out: Lookup;
    out.found = false;
    out.failed = false;
    out.p = zero_pointer();
    if (some_mapping_leaf_has_entries_narrower_than_a_mapping_entry) {
        out.failed = true;
        return out;
    }
    let probe = MappingKeyFields(key_class, rd64(key_base, key_offset + 34u), rd64(key_base, key_offset + 42u), rd32(key_base, key_offset + 78u),
        select(vec2<u32>(key_tail.x, 0u), key_tail, key_tail_is_transaction));
    let position = mapping_entry_holding_the_key(probe);
    if (!position.found) {
        return out;
    }
    let base = position.base;
    let entry_at = position.offset;
    var pointer = zero_pointer();
    pointer.all_zero = false;
    pointer.device0 = rd32(base, entry_at + 27u);
    pointer.slot0 = rd48(base, entry_at + 31u);
    pointer.sum0 = rd32(base, entry_at + 37u);
    pointer.device1 = rd32(base, entry_at + 41u);
    pointer.slot1 = rd48(base, entry_at + 45u);
    pointer.sum1 = rd32(base, entry_at + 51u);
    out.p = pointer;
    out.found = true;
    return out;
}
struct MappedRead {
    s: Slice,
    failed: bool,
};
// 按位置提示读，读不出经中央映射回退（recovery::read_mapped_tree_node_via_hint_then_central_mapping / read_data_unit_via_hint_then_central_mapping）
// pointer_base / pointer_offset：这条指针在字节里的位置（映射 key 从它的出生树、出生 txg、实例、尾段拼）
fn read_via_hint_then_mapping(p: Ptr, pointer_base: u32, pointer_offset: u32, unit_class: u32, unit_bytes: u32) -> MappedRead {
    var out: MappedRead;
    out.failed = false;
    // 建上一版那一样读者读的每个进映射的单元：映射里有它的 key 时，重建之后它的指针换成映射那两条位置条目，再按 I-2.5 判一遍
    // （transaction::take_the_mapping_positions_over_the_location_hints 之后的 pointers_of_the_rebuilt_version_ascend_by_device）。
    // 查映射本身判红（映射树读不出、条目窄于字段表）的这里不记：那一判在读者里先于这一判报
    if (rebuild_judges_the_pointers_of_the_version && mapping_tree_failure_point == FAILURE_POINT_NONE) {
        let position = mapping_lookup(pointer_base, pointer_offset, unit_class, p.tail, unit_class == 1u);
        if (!position.failed && position.found && position.p.device0 >= position.p.device1) {
            some_mapping_position_of_the_version_does_not_ascend_by_device = true;
        }
        if (!position.failed && !position.found) {
            some_mapped_unit_key_is_missing_from_the_central_mapping = true;
        }
    }
    out.s = read_unit_via_locations(p, unit_bytes);
    if (out.s.version != NONE) {
        return out;
    }
    // 要经映射回退：映射树读不出就报它读不出的那一处；映射条目窄于字段表报 EntryWidthDiffersFromItsFieldTable（central_mapping_locations_among_entries）；
    // 映射里没有报 MappingMiss；映射落点也读不出报 MappingStillUnreadable
    out.failed = true;
    if (mapping_tree_failure_point != FAILURE_POINT_NONE) {
        walk_failure_point = mapping_tree_failure_point;
        return out;
    }
    let looked = mapping_lookup(pointer_base, pointer_offset, unit_class, p.tail, unit_class == 1u);
    if (looked.failed) {
        walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
        return out;
    }
    if (!looked.found) {
        walk_failure_point = FAILURE_POINT_MAPPING_MISS;
        return out;
    }
    out.s = read_unit_via_locations(looked.p, unit_bytes);
    if (out.s.version == NONE) {
        walk_failure_point = FAILURE_POINT_MAPPING_STILL_UNREADABLE;
        return out;
    }
    out.failed = false;
    return out;
}

// ───────────────────────── 走读用的判定 ─────────────────────────

fn root_header_checks(n: Node, tree: vec2<u32>, key_width: u32, p: Ptr, root: RootFields, fsid_low: vec2<u32>) -> bool {
    if (!eq64(n.tree, tree)) {
        return false;
    }
    if (n.key_width != key_width) {
        return false;
    }
    if (lt64(root.txg, n.birth_txg) || n.instance > root.instance) {
        return false;
    }
    if (!eq64(n.fsid, fsid_low)) {
        return false;
    }
    if (n.birth_sequence != p.tail.x) {
        return false;
    }
    return true;
}
// recovery::tree_root_checked_against_its_pointer
// 判不过时记失败点：解不开 UnitMalformed，头与指针、层级 0 的 key 区间对不上 InvariantViolated。层级 ≥ 1 的节点头里的 key 区间是子树覆盖区间
// （D18 已定项 2），由读孩子的一方判（inode 树根读完叶容器判 inode_tree_root_key_range_is_the_subtree_coverage，容器树在 read_container_tree_index_nodes 里判）
fn tree_root_checked(n: Node, tree: vec2<u32>, key_width: u32, p: Ptr, root: RootFields, fsid_low: vec2<u32>) -> bool {
    if (!n.ok) {
        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
        return false;
    }
    walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
    if (!root_header_checks(n, tree, key_width, p, root, fsid_low)) {
        return false;
    }
    if (n.level == 0u && !key_range_is_tight_over_the_entries(n, key_width)) {
        return false;
    }
    return true;
}
// recovery::key_range_is_tight_over_the_entries：头里的 key 区间是首末两条条目的 key（零条目的节点不判）
fn key_range_is_tight_over_the_entries(n: Node, key_width: u32) -> bool {
    if (n.entry_count == 0u) {
        return true;
    }
    return bytes_equal(n.base, entry_offset(n, 0u), n.base, n.smallest_offset, key_width)
        && bytes_equal(n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset, key_width);
}
// recovery::inode_tree_root_key_range_is_the_subtree_coverage 的累加那一半：读回来的记录号里最小、最大的两个（have 为 false 是还一条都没有）
struct InodeNumbersReadBack {
    have: bool,
    smallest: vec2<u32>,
    largest: vec2<u32>,
};
fn inode_numbers_read_back_with(range: InodeNumbersReadBack, inode: vec2<u32>) -> InodeNumbersReadBack {
    var out = range;
    if (!range.have) {
        out.have = true;
        out.smallest = inode;
        out.largest = inode;
        return out;
    }
    if (lt64(inode, range.smallest)) { out.smallest = inode; }
    if (lt64(range.largest, inode)) { out.largest = inode; }
    return out;
}
// recovery::inode_records_of_the_leaf_ascend_after_the_leaves_before（写者的前提：全部 inode 叶的记录按叶序连起来严格升序）：一片 inode 叶容器的第一条
// 大于叶序在它前面那几片的最大记录号（I-9.12，两片叶交叠），指着它的那条内部条目的分隔 key 大于那个最大记录号（I-9.12 第三句），片内相邻两条
// 严格升序（I-9.4 第四句）。leaves_before 是读这片之前读回的记录号区间（头一片 have 为假；前面各片过了这一判，最大的就是上一片的末条，同 CPU 交回的）；
// 没有记录的叶交真（三个读者各自照 CPU 另判或不判）。不成立是 InvariantViolated，调用方照它的读者种类折算
fn inode_records_of_the_leaf_ascend_after_the_leaves_before(leaf: Packed, separator: vec2<u32>, leaves_before: InodeNumbersReadBack) -> bool {
    if (leaf.record_count == 0u) {
        return true;
    }
    if (leaves_before.have
        && (le64(rd64(leaf.base, packed_record_offset(leaf, 0u)), leaves_before.largest) || le64(separator, leaves_before.largest))) {
        return false;
    }
    for (var record_index = 1u; record_index < leaf.record_count; record_index = record_index + 1u) {
        if (le64(rd64(leaf.base, packed_record_offset(leaf, record_index)), rd64(leaf.base, packed_record_offset(leaf, record_index - 1u)))) {
            return false;
        }
    }
    return true;
}
// recovery::inode_tree_root_key_range_is_the_subtree_coverage：inode 树根头里的 key 区间 = [读回来的最小记录号, 最大记录号]；
// 零条目的根与一条记录都没读回来的不判。不成立是 InvariantViolated（调用方照它的读者种类折算）
fn inode_tree_root_key_range_is_the_subtree_coverage(inode_root: Node, read_back: InodeNumbersReadBack) -> bool {
    if (inode_root.entry_count == 0u || !read_back.have) {
        return true;
    }
    return eq64(rd64(inode_root.base, inode_root.smallest_offset), read_back.smallest)
        && eq64(rd64(inode_root.base, inode_root.largest_offset), read_back.largest);
}
// 见过的单元（按第一条位置条目的槽号）：重复交回 false
fn seen_insert(slot: vec2<u32>) -> bool {
    for (var i = 0u; i < seen_count; i = i + 1u) {
        if (sw(R_SEEN + i * 2u) == slot.x && sw(R_SEEN + i * 2u + 1u) == slot.y) {
            return false;
        }
    }
    if (seen_count >= MAX_SEEN) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
        return false;
    }
    sws(R_SEEN + seen_count * 2u, slot.x);
    sws(R_SEEN + seen_count * 2u + 1u, slot.y);
    seen_count = seen_count + 1u;
    return true;
}

// ───────────────────────── 多层码 2 树（记账、中央映射）─────────────────────────

// key 按字段比：field_widths 0 = 记账 [2,8,4,8]、1 = 映射 [1,8,8,4,6]、5 = deadlist [8,8,1,8,8,4,6]（头 8、纪元 8，后五段是中央映射 key）、
// 6 = logged_ops [2,8]
fn field_width(schema: u32, field: u32) -> u32 {
    if (schema == 0u) {
        if (field == 0u) { return 2u; }
        if (field == 1u) { return 8u; }
        if (field == 2u) { return 4u; }
        return 8u;
    }
    if (schema == KEY_SCHEMA_LOGGED_OPERATIONS) {
        return select(8u, 2u, field == 0u);
    }
    // 映射那五段 [1,8,8,4,6] 从第 field_in_the_mapping_key 段数起：映射 key 从第 0 字段起，deadlist 的 key 前面多头与纪元两段 8 字节
    var field_in_the_mapping_key = field;
    if (schema == KEY_SCHEMA_DEADLIST) {
        if (field < 2u) { return 8u; }
        field_in_the_mapping_key = field - 2u;
    }
    if (field_in_the_mapping_key == 0u) { return 1u; }
    if (field_in_the_mapping_key == 1u) { return 8u; }
    if (field_in_the_mapping_key == 2u) { return 8u; }
    if (field_in_the_mapping_key == 3u) { return 4u; }
    return 6u;
}
fn field_count(schema: u32) -> u32 {
    if (schema == 0u) {
        return 4u;
    }
    if (schema == KEY_SCHEMA_DEADLIST) {
        return 7u;
    }
    if (schema == KEY_SCHEMA_LOGGED_OPERATIONS) {
        return 2u;
    }
    return 5u;
}
// 一种 key 形态的宽：各字段宽之和（记账 22、映射 27、deadlist 43）
fn key_width_of_the_schema(schema: u32) -> u32 {
    var width = 0u;
    for (var field = 0u; field < field_count(schema); field = field + 1u) {
        width = width + field_width(schema, field);
    }
    return width;
}
fn read_field(base: u32, o: u32, width: u32) -> vec2<u32> {
    var v = vec2<u32>(0u, 0u);
    for (var i = 0u; i < width; i = i + 1u) {
        let b = rd8(base, o + i);
        if (i < 4u) {
            v.x = v.x | (b << (i * 8u));
        } else {
            v.y = v.y | (b << ((i - 4u) * 8u));
        }
    }
    return v;
}
// -1 / 0 / 1 → 0 / 1 / 2
fn compare_keys(schema: u32, base_a: u32, oa: u32, base_b: u32, ob: u32) -> u32 {
    var o1 = oa;
    var o2 = ob;
    let n = field_count(schema);
    for (var f = 0u; f < n; f = f + 1u) {
        let w = field_width(schema, f);
        let a = read_field(base_a, o1, w);
        let b = read_field(base_b, o2, w);
        if (lt64(a, b)) {
            return 0u;
        }
        if (lt64(b, a)) {
            return 2u;
        }
        o1 = o1 + w;
        o2 = o2 + w;
    }
    return 1u;
}
struct Code2Frame {
    n: Node,
    child_index: u32,
    first_child_smallest_base: u32,
    first_child_smallest_offset: u32,
    previous_child_largest_base: u32,
    previous_child_largest_offset: u32,
    has_previous_child: bool,
    separator_base: u32,
    separator_offset: u32,
    previous_separator_offset: u32,
    has_previous_separator: bool,
    expected_level: u32,
    is_root: bool,
};
var<private> code2_frames: array<Code2Frame, 12>;

struct TreeRead {
    ok: bool,
    node_count: u32,
    entry_count: u32,
};
fn failed_tree_read() -> TreeRead {
    var t: TreeRead;
    t.ok = false;
    t.node_count = 0u;
    t.entry_count = 0u;
    return t;
}
// code_two_tree::read_code_two_tree（EveryHeaderAgainstItsReference）：schema 0 记账（提示读不出经映射回退）、1 映射（只按位置条目读）、
// 5 deadlist（同记账：提示读不出经映射回退；recovery.rs walk_to_file 的 deadlist 树那一支，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
// 第三节 S3）。记账的叶条目写进 R_ACCOUNTING（字节基、条目偏移、条目宽）；映射的叶一片一行记进 R_MAPPING_LEAVES（条目留在叶的字节里，条数不设上限）；
// deadlist 的叶条目只数不记（冷走读只拿它的条数数映射条目，recovery.rs 的 mapping_entries_kept_for_the_snapshots），不占那两张表、不受记账那张表的条数上限
fn read_code_two_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, schema: u32, root: RootFields, fsid_low: vec2<u32>) -> TreeRead {
    var out = failed_tree_read();
    seen_count = 0u;
    let kw = key_width_of_the_schema(schema);
    let records_each_leaf_entry = schema == 0u;
    if (schema == 1u) {
        mapping_count = 0u;
        mapping_leaf_count = 0u;
        some_mapping_leaf_has_entries_narrower_than_a_mapping_entry = false;
    } else if (records_each_leaf_entry) {
        accounting_count = 0u;
    } else if (schema == KEY_SCHEMA_DEADLIST) {
        deadlist_leaf_count = 0u;
    }
    var entries = 0u;
    var nodes = 0u;
    var depth = 0u;
    // 压根
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_expected_level = NONE;
    var pending_is_root = true;
    var enter = true;
    loop {
        if (enter) {
            // 进一个节点
            // 失败点（code_two_tree::read_node_and_its_subtree）：同一个节点两次引用、头与引用对不上、层级、空节点、key 次序与区间都是
            // InvariantViolated；映射树的节点只按位置条目读，读不出 UnitUnreadable；解不开 UnitMalformed；内部条目窄 EntryWidthDiffersFromItsFieldTable；
            // 子指针头部 UnitMalformed；记账树的节点读不出照 read_via_hint_then_mapping 记的
            if (!seen_insert(pending_pointer.slot0)) {
                walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                return out;
            }
            var s: Slice;
            if (schema == 1u) {
                s = read_unit_via_locations(pending_pointer, NODE_BYTES);
                if (s.version == NONE) {
                    walk_failure_point = FAILURE_POINT_UNIT_UNREADABLE;
                    return out;
                }
            } else {
                let m = read_via_hint_then_mapping(pending_pointer, pending_pointer_base, pending_pointer_offset, 2u, NODE_BYTES);
                if (m.failed) {
                    return out;
                }
                s = m.s;
            }
            let n = parse_index_node(s);
            if (!n.ok) {
                walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                return out;
            }
            walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
            if (!root_header_checks(n, tree, kw, pending_pointer, root, fsid_low)) {
                return out;
            }
            if (n.level == 0u && n.entry_count > 0u) {
                if (!bytes_equal(n.base, entry_offset(n, 0u), n.base, n.smallest_offset, kw)
                    || !bytes_equal(n.base, entry_offset(n, n.entry_count - 1u), n.base, n.largest_offset, kw)) {
                    return out;
                }
            }
            if (pending_expected_level != NONE && n.level != pending_expected_level) {
                return out;
            }
            if (n.entry_count == 0u) {
                return out;
            }
            nodes = nodes + 1u;
            if (n.level == 0u) {
                // 叶：条目按 key 严格递增，记条目（映射记叶：这片叶已经进了见过的节点表，行数不过 MAX_SEEN）
                if (schema == 1u) {
                    sws(R_MAPPING_LEAVES + mapping_leaf_count, n.base);
                    mapping_leaf_count = mapping_leaf_count + 1u;
                    if (n.entry_width < 55u) {
                        some_mapping_leaf_has_entries_narrower_than_a_mapping_entry = true;
                    }
                } else if (schema == KEY_SCHEMA_DEADLIST) {
                    sws(R_DEADLIST_LEAVES + deadlist_leaf_count, n.base);
                    deadlist_leaf_count = deadlist_leaf_count + 1u;
                }
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    if (e > 0u && compare_keys(schema, n.base, entry_offset(n, e - 1u), n.base, entry_offset(n, e)) != 0u) {
                        return out;
                    }
                    if (records_each_leaf_entry) {
                        if (entries >= MAX_ACCOUNTING_ENTRIES) {
                            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                            return out;
                        }
                        sws(R_ACCOUNTING + entries * 3u, n.base);
                        sws(R_ACCOUNTING + entries * 3u + 1u, entry_offset(n, e));
                        sws(R_ACCOUNTING + entries * 3u + 2u, n.entry_width);
                    }
                    entries = entries + 1u;
                }
                // 把叶交给父节点当孩子
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                let parent = depth - 1u;
                // 分隔 key ≤ 孩子最小 key
                if (compare_keys(schema, code2_frames[parent].separator_base, code2_frames[parent].separator_offset, n.base, n.smallest_offset) == 2u) {
                    return out;
                }
                if (code2_frames[parent].child_index == 0u) {
                    code2_frames[parent].first_child_smallest_base = n.base;
                    code2_frames[parent].first_child_smallest_offset = n.smallest_offset;
                }
                code2_frames[parent].previous_child_largest_base = n.base;
                code2_frames[parent].previous_child_largest_offset = n.largest_offset;
                code2_frames[parent].has_previous_child = true;
                code2_frames[parent].child_index = code2_frames[parent].child_index + 1u;
                enter = false;
                continue;
            }
            // 内部条目宽不等于 key 宽 + 86 就拒，宽了的同样拒（code_two_tree::read_code_two_tree 读内部节点那一判与 parse_internal_entry，
            // C780「宽于字段表也拒」）；失败点沿用 EntryWidthDiffersFromItsFieldTable 那一个码，与 core 报的成员相同
            if (n.entry_width != kw + 86u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            code2_frames[depth].n = n;
            code2_frames[depth].child_index = 0u;
            code2_frames[depth].has_previous_child = false;
            code2_frames[depth].has_previous_separator = false;
            code2_frames[depth].expected_level = pending_expected_level;
            code2_frames[depth].is_root = pending_is_root;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        // 栈顶内部节点：下一个孩子，或收尾
        let top = depth - 1u;
        let n = code2_frames[top].n;
        if (code2_frames[top].child_index >= n.entry_count) {
            // 区间 = 第一个孩子的最小 key 到最后一个孩子的最大 key
            if (!bytes_equal(code2_frames[top].first_child_smallest_base, code2_frames[top].first_child_smallest_offset, n.base, n.smallest_offset, kw)
                || !bytes_equal(code2_frames[top].previous_child_largest_base, code2_frames[top].previous_child_largest_offset, n.base, n.largest_offset, kw)) {
                return out;
            }
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            let parent = depth - 1u;
            if (compare_keys(schema, code2_frames[parent].separator_base, code2_frames[parent].separator_offset, n.base, n.smallest_offset) == 2u) {
                return out;
            }
            if (code2_frames[parent].child_index == 0u) {
                code2_frames[parent].first_child_smallest_base = n.base;
                code2_frames[parent].first_child_smallest_offset = n.smallest_offset;
            }
            code2_frames[parent].previous_child_largest_base = n.base;
            code2_frames[parent].previous_child_largest_offset = n.largest_offset;
            code2_frames[parent].has_previous_child = true;
            code2_frames[parent].child_index = code2_frames[parent].child_index + 1u;
            continue;
        }
        let e = code2_frames[top].child_index;
        let separator_offset = entry_offset(n, e);
        let child_pointer = parse_node_pointer(n.base, separator_offset + kw);
        // code_two_tree::parse_internal_entry：子指针头部加密、压缩那几段与 extent 偏移都要是第一版的取值
        if (!child_pointer.head_ok || !child_pointer.extent_offset_zero) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return out;
        }
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (code2_frames[top].has_previous_separator) {
            if (compare_keys(schema, n.base, separator_offset, n.base, code2_frames[top].previous_separator_offset) != 2u) {
                return out;
            }
            if (compare_keys(schema, n.base, separator_offset, code2_frames[top].previous_child_largest_base, code2_frames[top].previous_child_largest_offset) != 2u) {
                return out;
            }
        }
        code2_frames[top].separator_base = n.base;
        code2_frames[top].separator_offset = separator_offset;
        code2_frames[top].previous_separator_offset = separator_offset;
        code2_frames[top].has_previous_separator = true;
        pending_pointer = child_pointer;
        pending_pointer_base = n.base;
        pending_pointer_offset = separator_offset + kw;
        pending_expected_level = n.level - 1u;
        pending_is_root = false;
        enter = true;
    }
    out.node_count = nodes;
    out.entry_count = entries;
    if (schema == 1u) {
        mapping_count = entries;
    } else if (records_each_leaf_entry) {
        accounting_count = entries;
    }
    return out;
}

// ───────────────────────── 分配记录树 ─────────────────────────

fn allocation_span(level: u32) -> vec2<u32> {
    var span = u64_of(ALLOCATION_LEAF_SLOTS);
    for (var i = 0u; i < level; i = i + 1u) {
        span = mul64_32_saturating(span, ALLOCATION_INTERNAL_FANOUT);
    }
    return span;
}
fn allocation_root_level() -> u32 {
    let slots = shr64(device_bytes(), SLOT_SHIFT);
    let device_count = hdr(3u);
    for (var level = 1u; level < 256u; level = level + 1u) {
        let child_span = allocation_span(level - 1u);
        var cells = vec2<u32>(0u, 0u);
        for (var d = 0u; d < device_count; d = d + 1u) {
            let q = divmod64(add64_saturating(slots, sub64(child_span, u64_of(1u))), child_span).quotient;
            cells = add64_saturating(cells, q);
        }
        if (le64(cells, u64_of(ALLOCATION_INTERNAL_FANOUT))) {
            return level;
        }
    }
    return 255u;
}
// 根之下的节点位置：level、device、index；根 level = root_level、device = NONE
struct AllocationNode {
    is_root: bool,
    level: u32,
    device: u32,
    index: vec2<u32>,
};
fn allocation_key_bytes_equal(base: u32, o: u32, device: u32, slot: vec2<u32>) -> bool {
    return rd32(base, o) == device && eq64(rd48(base, o + 4u), slot);
}
struct AllocationFrame {
    n: Node,
    node: AllocationNode,
    child_index: u32,
    has_previous_child: bool,
    previous_child_device: u32,
    previous_child_index: vec2<u32>,
};
var<private> allocation_frames: array<AllocationFrame, 12>;

fn allocation_first_slot(level: u32, index: vec2<u32>) -> vec2<u32> {
    let span = allocation_span(level);
    let p = mul64_32(index, span.x);
    var first = p.value;
    if (p.overflow || span.y != 0u) {
        first = vec2<u32>(NONE, NONE);
    }
    // 槽号最大 2^48 − 1
    let largest = vec2<u32>(NONE, 0xFFFFu);
    if (lt64(largest, first)) {
        first = largest;
    }
    return first;
}
fn allocation_last_slot(level: u32, index: vec2<u32>) -> vec2<u32> {
    let span = allocation_span(level);
    let first = allocation_first_slot(level, index);
    var last = add64_saturating(first, sub64(span, u64_of(1u)));
    let largest = vec2<u32>(NONE, 0xFFFFu);
    if (lt64(largest, last)) {
        last = largest;
    }
    return last;
}
struct AllocationRead {
    ok: bool,
    node_count: u32,
    record_count: u32,
};
// allocation_record_tree::read_allocation_record_tree（EveryHeaderAgainstItsReference，提示读不出经映射回退）；记录写进 R_ALLOCATION
fn read_allocation_tree(root_pointer: Ptr, root_pointer_base: u32, root_pointer_offset: u32, tree: vec2<u32>, root: RootFields, fsid_low: vec2<u32>) -> AllocationRead {
    var out: AllocationRead;
    out.ok = false;
    out.node_count = 0u;
    out.record_count = 0u;
    seen_count = 0u;
    allocation_count = 0u;
    let root_level = allocation_root_level();
    var nodes = 0u;
    var depth = 0u;
    var pending_pointer = root_pointer;
    var pending_pointer_base = root_pointer_base;
    var pending_pointer_offset = root_pointer_offset;
    var pending_node: AllocationNode;
    pending_node.is_root = true;
    pending_node.level = root_level;
    pending_node.device = NONE;
    pending_node.index = vec2<u32>(0u, 0u);
    var enter = true;
    loop {
        if (enter) {
            // 失败点（allocation_record_tree::read_node_and_its_subtree）：两次引用、头与引用、层级、key 区间、空节点、叶里记录次序、
            // 内部条目点名的孩子与次序是 InvariantViolated；解不开 UnitMalformed；条目窄 EntryWidthDiffersFromItsFieldTable；叶里记录不在叶罩的那一段
            // AllocationRecordOutsideThePoolGeometry；子指针头部 UnitMalformed；读不出照 read_via_hint_then_mapping 记的
            if (!seen_insert(pending_pointer.slot0)) {
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
            walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
            if (!root_header_checks(n, tree, 10u, pending_pointer, root, fsid_low)) {
                return out;
            }
            if (n.level != pending_node.level) {
                return out;
            }
            // key 区间是它的位置规定罩的那一段
            if (pending_node.is_root) {
                if (!all_zero(n.base, n.smallest_offset, 10u)) {
                    return out;
                }
                for (var i = 0u; i < 10u; i = i + 1u) {
                    if (rd8(n.base, n.largest_offset + i) != 0xFFu) {
                        return out;
                    }
                }
            } else {
                let first = allocation_first_slot(pending_node.level, pending_node.index);
                let last = allocation_last_slot(pending_node.level, pending_node.index);
                if (!allocation_key_bytes_equal(n.base, n.smallest_offset, pending_node.device, first)
                    || !allocation_key_bytes_equal(n.base, n.largest_offset, pending_node.device, last)) {
                    return out;
                }
            }
            if (n.entry_count == 0u) {
                return out;
            }
            nodes = nodes + 1u;
            if (n.level == 0u) {
                // 分配记录宽不等于 20 就拒，宽了的同样拒（allocation_record_tree.rs 逐条 AllocationRecord::parse，C867，I-1.10 写「等于」）
                if (n.entry_width != 20u) {
                    walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                    return out;
                }
                let leaf_last = allocation_last_slot(0u, pending_node.index);
                let first_record_of_the_leaf = allocation_count;
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    let device = rd32(n.base, o);
                    let slot = rd48(n.base, o + 4u);
                    let span_field = rd16(n.base, o + 10u);
                    // 隔离记号与已释放标志同时置（0x4000 与 0x8000）：allocator::AllocationRecord::parse 拒成
                    // QuarantineMarkOnAReleasedRecord，分配记录树读者报 UnitMalformed（D19（块指针的结构与宽度预算） 已定项 5），先于落叶判
                    if ((span_field & 0xC000u) == 0xC000u) {
                        walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
                        return out;
                    }
                    // 跨度取去掉两个标志位的低位；记号恢复用不上（各盘那几条比的是 (槽, 跨度, 代, 已释放)，
                    // recovery::allocation_records_are_one_per_device），存进表时去掉，下面几处照旧按 0x7FFF 取跨度
                    let span = span_field & 0x3FFFu;
                    let generation = rd64(n.base, o + 12u);
                    // 落在这片叶里：同盘、叶序号 = 槽 / 812、末槽不越过叶末槽
                    let leaf_index = divmod64(slot, u64_of(ALLOCATION_LEAF_SLOTS)).quotient;
                    if (pending_node.is_root || device != pending_node.device || !eq64(leaf_index, pending_node.index)) {
                        walk_failure_point = FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
                        return out;
                    }
                    if (span == 0u || lt64(leaf_last, add64(slot, u64_of(span - 1u)))) {
                        walk_failure_point = FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY;
                        return out;
                    }
                    if (allocation_count >= MAX_ALLOCATION_RECORDS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;
                        return out;
                    }
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS, device);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 1u, slot.x);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 2u, slot.y);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 3u, span_field & 0xBFFFu);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 4u, generation.x);
                    sws(R_ALLOCATION + allocation_count * ALLOCATION_RECORD_WORDS + 5u, generation.y);
                    allocation_count = allocation_count + 1u;
                }
                // 叶里的记录按 key 严格递增：CPU 逐条解完、判完落在叶里之后才判（read_node_and_its_subtree 叶那一支的 keys_ascend）
                for (var r = first_record_of_the_leaf + 1u; r < allocation_count; r = r + 1u) {
                    let previous_device = sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS);
                    let previous_slot = vec2<u32>(sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + (r - 1u) * ALLOCATION_RECORD_WORDS + 2u));
                    let device = sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS);
                    let slot = vec2<u32>(sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 1u), sw(R_ALLOCATION + r * ALLOCATION_RECORD_WORDS + 2u));
                    if (!(previous_device < device || (previous_device == device && lt64(previous_slot, slot)))) {
                        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
                        return out;
                    }
                }
                if (depth == 0u) {
                    out.ok = true;
                    break;
                }
                allocation_frames[depth - 1u].child_index = allocation_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            // 分配记录树内部条目宽不等于 96 就拒，宽了的同样拒（allocation_record_tree.rs 那一判，C867）
            if (n.entry_width != 96u) {
                walk_failure_point = FAILURE_POINT_ENTRY_WIDTH_DIFFERS_FROM_ITS_FIELD_TABLE;
                return out;
            }
            if (depth >= MAX_FRAMES) {
                undecidable = UNDECIDABLE_TOO_DEEP;
                return out;
            }
            allocation_frames[depth].n = n;
            allocation_frames[depth].node = pending_node;
            allocation_frames[depth].child_index = 0u;
            allocation_frames[depth].has_previous_child = false;
            depth = depth + 1u;
            enter = false;
            continue;
        }
        let top = depth - 1u;
        let n = allocation_frames[top].n;
        let parent = allocation_frames[top].node;
        if (allocation_frames[top].child_index >= n.entry_count) {
            depth = depth - 1u;
            if (depth == 0u) {
                out.ok = true;
                break;
            }
            allocation_frames[depth - 1u].child_index = allocation_frames[depth - 1u].child_index + 1u;
            continue;
        }
        let e = allocation_frames[top].child_index;
        let o = entry_offset(n, e);
        // 孩子由条目 key 点名：槽落在孩子那一层的格点上、盘在池里、孩子的父就是这个节点
        let child_level = parent.level - 1u;
        let device = rd32(n.base, o);
        let slot = rd48(n.base, o + 4u);
        let child_span = allocation_span(child_level);
        let qr = divmod64(slot, child_span);
        walk_failure_point = FAILURE_POINT_INVARIANT_VIOLATED;
        if (!is_zero64(qr.remainder) || !device_is_in_the_pool(device)) {
            return out;
        }
        var child: AllocationNode;
        child.is_root = false;
        child.level = child_level;
        child.device = device;
        child.index = qr.quotient;
        // parent_of(child) == parent
        if (child_level + 1u == root_level) {
            if (!parent.is_root) {
                return out;
            }
        } else {
            if (parent.is_root) {
                return out;
            }
            let parent_index = divmod64(child.index, u64_of(ALLOCATION_INTERNAL_FANOUT)).quotient;
            if (parent.device != device || !eq64(parent_index, parent.index)) {
                return out;
            }
        }
        if (allocation_frames[top].has_previous_child) {
            let previous_device = allocation_frames[top].previous_child_device;
            let previous_index = allocation_frames[top].previous_child_index;
            if (!(previous_device < device || (previous_device == device && lt64(previous_index, child.index)))) {
                return out;
            }
        }
        allocation_frames[top].has_previous_child = true;
        allocation_frames[top].previous_child_device = device;
        allocation_frames[top].previous_child_index = child.index;
        pending_pointer = parse_node_pointer(n.base, o + 10u);
        // allocation_record_tree 读内部条目的子指针：头部加密、压缩那几段不是第一版的取值，这个节点损坏
        if (!pending_pointer.head_ok) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return out;
        }
        // allocation_record_tree 读内部条目的子指针：头部 extent 偏移非 0，这个节点损坏
        if (!pending_pointer.extent_offset_zero) {
            walk_failure_point = FAILURE_POINT_UNIT_MALFORMED;
            return out;
        }
        pending_pointer_base = n.base;
        pending_pointer_offset = o + 10u;
        pending_node = child;
        enter = true;
    }
    out.node_count = nodes;
    out.record_count = allocation_count;
    return out;
}
