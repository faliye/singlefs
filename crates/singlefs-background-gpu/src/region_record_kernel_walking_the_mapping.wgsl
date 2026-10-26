// 区域扫描里的单元头解析与映射反查，两份独立核里按条目表的次序逐条看映射的那一份（D26（后台整理与放置回收） 已定项 13 第 8 条：两份核不共用核函数）。
// 每条记录一个调用：照核心层三个 parse 的判法解单元头（码 2 索引节点 16 KiB；码 1 数据单元 32 KiB，类标签是 3 时改当码 3 打包记录单元），
// 头校验和与载荷校验和按字节查 256 项余式表算 CRC-32C；解出来的映射 key 再从头到尾走一遍条目表，看有哪几条的位置项指着这个落点、这把 key 在不在表里。
// 偏移与宽度是这一份自己照字节表写的（D18（块里携带什么信息） 已定项 7、11、18），不读别处；与 CPU 参照（核心层）的对拍在 GPU 用例里。
// 另一份（region_record_kernel_searching_sorted_indexes.wgsl）逐位移算校验和、在两张排好序的索引上二分查；两份只共用缓冲布局，布局由 src/region_record_kernels.rs 定。

struct ChunkParametersForTheWalk {
    record_count: u32,
    mapping_entry_count: u32,
    location_item_count: u32,
    unused_word: u32,
}

struct RecordDescriptorForTheWalk {
    byte_offset: u32,
    byte_length: u32,
    span_slots: u32,
    bytes_were_read: u32,
    device: u32,
    slot_low: u32,
    slot_high: u32,
}

struct MappingEntryForTheWalk {
    key_length: u32,
    key_words: array<u32, 7>,
    first_device: u32,
    first_slot_low: u32,
    first_slot_high: u32,
    second_device: u32,
    second_slot_low: u32,
    second_slot_high: u32,
}

struct RecordOutcomeOfTheWalk {
    outcome_kind: u32,
    detail_word: u32,
    second_detail_word: u32,
    birth_txg_low: u32,
    birth_txg_high: u32,
    key_length: u32,
    key_words: array<u32, 7>,
}

@group(0) @binding(0) var<uniform> parameters_of_the_chunk: ChunkParametersForTheWalk;
@group(0) @binding(1) var<storage, read> unit_words_of_the_chunk: array<u32>;
@group(0) @binding(2) var<storage, read> descriptors_of_the_records: array<RecordDescriptorForTheWalk>;
@group(0) @binding(3) var<storage, read> entries_of_the_mapping: array<MappingEntryForTheWalk>;
@group(0) @binding(6) var<storage, read_write> outcomes_of_the_walk: array<RecordOutcomeOfTheWalk>;

const WALK_CASTAGNOLI_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const WALK_INVOCATIONS_PER_WORKGROUP: u32 = 64u;

// 结果的类、解析拒收的码与结构缺陷的码：与 src/region_record_kernels.rs 同值。
const KIND_MOVABLE: u32 = 0u;
const KIND_DEAD: u32 = 1u;
const KIND_UNIT_CLASS_DISAGREES: u32 = 2u;
const KIND_HEADER_KEY_DISAGREES_WITH_THE_LOCATION: u32 = 3u;
const KIND_DEVICE_READ_FAILED: u32 = 4u;
const KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT: u32 = 5u;
const KIND_REFUSED_BY_THE_PARSER: u32 = 6u;
const NOT_REFUSED: u32 = 0xFFFFFFFFu;
const REFUSED_TOO_SHORT: u32 = 0u;
const REFUSED_BAD_MAGIC: u32 = 1u;
const REFUSED_UNKNOWN_FORMAT_VERSION: u32 = 2u;
const REFUSED_NON_ZERO_FLAGS: u32 = 3u;
const REFUSED_WRONG_UNIT_CLASS: u32 = 4u;
const REFUSED_HEADER_CHECKSUM_MISMATCH: u32 = 5u;
const REFUSED_PAYLOAD_CHECKSUM_MISMATCH: u32 = 6u;
const REFUSED_ENCRYPTION_RESERVED_BYTES_NOT_ZERO: u32 = 7u;
const REFUSED_STRUCTURE: u32 = 8u;
const DEFECT_INDEX_NODE_DECLARED_LENGTH: u32 = 0u;
const DEFECT_INDEX_NODE_ENTRY_WIDTH_ZERO_WITH_ENTRIES: u32 = 1u;
const DEFECT_INDEX_NODE_ENTRY_WIDTH_BELOW_THE_KEY_WIDTH: u32 = 2u;
const DEFECT_INDEX_NODE_ENTRY_AREA_PAST_THE_END: u32 = 3u;
const DEFECT_PACKED_CLASS_TAG_COPY: u32 = 4u;
const DEFECT_PACKED_DECLARED_LENGTH: u32 = 5u;
const DEFECT_PACKED_RECORD_WIDTH_ZERO_WITH_RECORDS: u32 = 6u;
const DEFECT_PACKED_RECORD_AREA_PAST_THE_END: u32 = 7u;
const DEFECT_DATA_CLASS_TAG_COPY: u32 = 8u;
const DEFECT_DATA_DECLARED_LENGTH_PAST_THE_END: u32 = 9u;

// 单元格式（照字节表）。
const INDEX_NODE_BYTES_OF_THE_WALK: u32 = 16384u;
const TWO_SLOT_UNIT_BYTES_OF_THE_WALK: u32 = 32768u;
const MAGIC_WORD_SFSU: u32 = 0x55534653u;
const FORMAT_VERSION_ONE: u32 = 1u;
const CLASS_TAG_DATA: u32 = 1u;
const CLASS_TAG_INDEX_NODE: u32 = 2u;
const CLASS_TAG_PACKED: u32 = 3u;
const HEADER_CHECKSUM_FIELD_START: u32 = 10u;
const HEADER_CHECKSUM_FIELD_END: u32 = 42u;
const RESERVED_BYTES_AFTER_THE_HEADER: u32 = 29u;
const INDEX_NODE_HEADER_WITHOUT_THE_KEY_RANGE: u32 = 86u;
const DATA_UNIT_HEADER_END: u32 = 105u;
const PACKED_UNIT_HEADER_END: u32 = 107u;
const HEADER_KEY_BYTES: u32 = 27u;

var<workgroup> remainder_table_of_the_walk: array<u32, 256>;

struct RefusalOfTheWalk {
    refusal_code: u32,
    refusal_detail: u32,
}

struct ParsedHeaderOfTheWalk {
    refusal: RefusalOfTheWalk,
    class_tag: u32,
    birth_txg_low: u32,
    birth_txg_high: u32,
    key_words: array<u32, 7>,
}

fn unit_byte_at(byte_address: u32) -> u32 {
    return (unit_words_of_the_chunk[byte_address >> 2u] >> ((byte_address & 3u) * 8u)) & 0xFFu;
}

fn unit_u16_at(byte_address: u32) -> u32 {
    return unit_byte_at(byte_address) | (unit_byte_at(byte_address + 1u) << 8u);
}

fn unit_u32_at(byte_address: u32) -> u32 {
    return unit_u16_at(byte_address) | (unit_u16_at(byte_address + 2u) << 16u);
}

// CRC-32C over [first, past_the_end)，[zeroed_first, zeroed_past_the_end) 那几个字节按 0 参与。
fn crc32c_over_the_unit_by_table(first: u32, past_the_end: u32, zeroed_first: u32, zeroed_past_the_end: u32) -> u32 {
    var crc_register = 0xFFFFFFFFu;
    for (var byte_address = first; byte_address < past_the_end; byte_address = byte_address + 1u) {
        var byte_value = unit_byte_at(byte_address);
        if (byte_address >= zeroed_first && byte_address < zeroed_past_the_end) {
            byte_value = 0u;
        }
        crc_register = (crc_register >> 8u) ^ remainder_table_of_the_walk[(crc_register ^ byte_value) & 0xFFu];
    }
    return crc_register ^ 0xFFFFFFFFu;
}

fn not_refused() -> RefusalOfTheWalk {
    return RefusalOfTheWalk(NOT_REFUSED, 0u);
}

// 共同前缀四关（magic、版本、类标签、flags）加头校验和，次序同核心层。
fn refusal_of_the_common_prefix(unit_start: u32, expected_class_tag: u32, header_end: u32) -> RefusalOfTheWalk {
    if (unit_u32_at(unit_start) != MAGIC_WORD_SFSU) {
        return RefusalOfTheWalk(REFUSED_BAD_MAGIC, 0u);
    }
    if (unit_u16_at(unit_start + 4u) != FORMAT_VERSION_ONE) {
        return RefusalOfTheWalk(REFUSED_UNKNOWN_FORMAT_VERSION, 0u);
    }
    let found_class_tag = unit_byte_at(unit_start + 6u);
    if (found_class_tag != expected_class_tag) {
        return RefusalOfTheWalk(REFUSED_WRONG_UNIT_CLASS, expected_class_tag | (found_class_tag << 8u));
    }
    if (unit_byte_at(unit_start + 7u) != 0u) {
        return RefusalOfTheWalk(REFUSED_NON_ZERO_FLAGS, 0u);
    }
    let computed_header_crc = crc32c_over_the_unit_by_table(
        unit_start,
        unit_start + header_end,
        unit_start + HEADER_CHECKSUM_FIELD_START,
        unit_start + HEADER_CHECKSUM_FIELD_END,
    );
    var header_checksum_holds = unit_u32_at(unit_start + HEADER_CHECKSUM_FIELD_START) == computed_header_crc;
    for (var field_byte = HEADER_CHECKSUM_FIELD_START + 4u; field_byte < HEADER_CHECKSUM_FIELD_END; field_byte = field_byte + 1u) {
        if (unit_byte_at(unit_start + field_byte) != 0u) {
            header_checksum_holds = false;
        }
    }
    if (!header_checksum_holds) {
        return RefusalOfTheWalk(REFUSED_HEADER_CHECKSUM_MISMATCH, 0u);
    }
    return not_refused();
}

// 载荷 CRC 罩 [header_end, 单元末尾)，存在 payload_crc_offset；之后是 29 字节预留位，全 0 才过。
fn refusal_of_the_payload_and_reserved_bytes(unit_start: u32, unit_bytes: u32, header_end: u32, payload_crc_offset: u32) -> RefusalOfTheWalk {
    let computed_payload_crc = crc32c_over_the_unit_by_table(unit_start + header_end, unit_start + unit_bytes, 0u, 0u);
    if (unit_u32_at(unit_start + payload_crc_offset) != computed_payload_crc) {
        return RefusalOfTheWalk(REFUSED_PAYLOAD_CHECKSUM_MISMATCH, 0u);
    }
    for (var reserved_byte = 0u; reserved_byte < RESERVED_BYTES_AFTER_THE_HEADER; reserved_byte = reserved_byte + 1u) {
        if (unit_byte_at(unit_start + header_end + reserved_byte) != 0u) {
            return RefusalOfTheWalk(REFUSED_ENCRYPTION_RESERVED_BYTES_NOT_ZERO, 0u);
        }
    }
    return not_refused();
}

fn refused_header(refusal: RefusalOfTheWalk) -> ParsedHeaderOfTheWalk {
    var parsed: ParsedHeaderOfTheWalk;
    parsed.refusal = refusal;
    return parsed;
}

// 把单元里从 unit_address 起的 byte_count 个字节放进 key 的第 key_position 个字节起（key 先是全 0）。
fn key_with_unit_bytes(key_before: array<u32, 7>, key_position: u32, unit_address: u32, byte_count: u32) -> array<u32, 7> {
    var key_after = key_before;
    for (var copied = 0u; copied < byte_count; copied = copied + 1u) {
        let position_in_the_key = key_position + copied;
        key_after[position_in_the_key >> 2u] = key_after[position_in_the_key >> 2u]
            | (unit_byte_at(unit_address + copied) << ((position_in_the_key & 3u) * 8u));
    }
    return key_after;
}

fn parsed_index_node_header(unit_start: u32, unit_bytes: u32) -> ParsedHeaderOfTheWalk {
    if (unit_bytes != INDEX_NODE_BYTES_OF_THE_WALK) {
        return refused_header(RefusalOfTheWalk(REFUSED_TOO_SHORT, 0u));
    }
    let key_width = unit_byte_at(unit_start + 51u);
    let header_end = INDEX_NODE_HEADER_WITHOUT_THE_KEY_RANGE + 2u * key_width;
    let prefix_refusal = refusal_of_the_common_prefix(unit_start, CLASS_TAG_INDEX_NODE, header_end);
    if (prefix_refusal.refusal_code != NOT_REFUSED) {
        return refused_header(prefix_refusal);
    }
    let payload_refusal = refusal_of_the_payload_and_reserved_bytes(unit_start, unit_bytes, header_end, header_end - 10u);
    if (payload_refusal.refusal_code != NOT_REFUSED) {
        return refused_header(payload_refusal);
    }
    let declared_length = unit_u16_at(unit_start + 8u);
    let after_the_key_range = unit_start + 52u + 2u * key_width;
    let entry_count = unit_u16_at(after_the_key_range + 30u);
    let entry_width = unit_u16_at(after_the_key_range + 32u);
    if (entry_count * entry_width != declared_length) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_INDEX_NODE_DECLARED_LENGTH));
    }
    if (entry_width == 0u && entry_count != 0u) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_INDEX_NODE_ENTRY_WIDTH_ZERO_WITH_ENTRIES));
    }
    if (entry_width < key_width) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_INDEX_NODE_ENTRY_WIDTH_BELOW_THE_KEY_WIDTH));
    }
    if (header_end + RESERVED_BYTES_AFTER_THE_HEADER + declared_length > unit_bytes) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_INDEX_NODE_ENTRY_AREA_PAST_THE_END));
    }
    var parsed: ParsedHeaderOfTheWalk;
    parsed.refusal = not_refused();
    parsed.class_tag = CLASS_TAG_INDEX_NODE;
    parsed.birth_txg_low = unit_u32_at(after_the_key_range);
    parsed.birth_txg_high = unit_u32_at(after_the_key_range + 4u);
    // key：类标签 2、树 ID（偏移 42 的 8 字节）、诞生代号 8、实例代号 4 与出生序号 4（诞生代号之后隔 fsid 8），末尾补 0 两字节
    var key: array<u32, 7>;
    key[0] = CLASS_TAG_INDEX_NODE;
    key = key_with_unit_bytes(key, 1u, unit_start + 42u, 8u);
    key = key_with_unit_bytes(key, 9u, after_the_key_range, 8u);
    key = key_with_unit_bytes(key, 17u, after_the_key_range + 16u, 8u);
    parsed.key_words = key;
    return parsed;
}

fn parsed_packed_unit_header(unit_start: u32, unit_bytes: u32) -> ParsedHeaderOfTheWalk {
    let prefix_refusal = refusal_of_the_common_prefix(unit_start, CLASS_TAG_PACKED, PACKED_UNIT_HEADER_END);
    if (prefix_refusal.refusal_code != NOT_REFUSED) {
        return refused_header(prefix_refusal);
    }
    let payload_refusal = refusal_of_the_payload_and_reserved_bytes(unit_start, unit_bytes, PACKED_UNIT_HEADER_END, 89u);
    if (payload_refusal.refusal_code != NOT_REFUSED) {
        return refused_header(payload_refusal);
    }
    if (unit_byte_at(unit_start + 42u) != CLASS_TAG_PACKED) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_PACKED_CLASS_TAG_COPY));
    }
    let declared_length = unit_u16_at(unit_start + 8u);
    let record_count = unit_u16_at(unit_start + 69u);
    let record_width = unit_u16_at(unit_start + 71u);
    if (record_count * record_width != declared_length) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_PACKED_DECLARED_LENGTH));
    }
    if (record_width == 0u && record_count != 0u) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_PACKED_RECORD_WIDTH_ZERO_WITH_RECORDS));
    }
    if (PACKED_UNIT_HEADER_END + RESERVED_BYTES_AFTER_THE_HEADER + declared_length > unit_bytes) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_PACKED_RECORD_AREA_PAST_THE_END));
    }
    var parsed: ParsedHeaderOfTheWalk;
    parsed.refusal = not_refused();
    parsed.class_tag = CLASS_TAG_PACKED;
    parsed.birth_txg_low = unit_u32_at(unit_start + 73u);
    parsed.birth_txg_high = unit_u32_at(unit_start + 77u);
    // key：类标签 3、出生树 8（偏移 43）、诞生代号 8（偏移 73）、写序里的实例代号 4（偏移 93）、出生序号 4（偏移 103），末尾补 0 两字节
    var key: array<u32, 7>;
    key[0] = CLASS_TAG_PACKED;
    key = key_with_unit_bytes(key, 1u, unit_start + 43u, 8u);
    key = key_with_unit_bytes(key, 9u, unit_start + 73u, 8u);
    key = key_with_unit_bytes(key, 17u, unit_start + 93u, 4u);
    key = key_with_unit_bytes(key, 21u, unit_start + 103u, 4u);
    parsed.key_words = key;
    return parsed;
}

// 两槽的单元先当码 1 数据单元解；共同前缀判出类标签是 3 时改当码 3 打包记录单元解（同核心层的区域扫描）。
fn parsed_two_slot_unit_header(unit_start: u32, unit_bytes: u32) -> ParsedHeaderOfTheWalk {
    if (unit_bytes != TWO_SLOT_UNIT_BYTES_OF_THE_WALK) {
        return refused_header(RefusalOfTheWalk(REFUSED_TOO_SHORT, 0u));
    }
    let prefix_refusal = refusal_of_the_common_prefix(unit_start, CLASS_TAG_DATA, DATA_UNIT_HEADER_END);
    if (prefix_refusal.refusal_code == REFUSED_WRONG_UNIT_CLASS && unit_byte_at(unit_start + 6u) == CLASS_TAG_PACKED) {
        return parsed_packed_unit_header(unit_start, unit_bytes);
    }
    if (prefix_refusal.refusal_code != NOT_REFUSED) {
        return refused_header(prefix_refusal);
    }
    let payload_refusal = refusal_of_the_payload_and_reserved_bytes(unit_start, unit_bytes, DATA_UNIT_HEADER_END, 101u);
    if (payload_refusal.refusal_code != NOT_REFUSED) {
        return refused_header(payload_refusal);
    }
    if (unit_byte_at(unit_start + 42u) != CLASS_TAG_DATA) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_DATA_CLASS_TAG_COPY));
    }
    let declared_length = unit_u16_at(unit_start + 8u);
    if (DATA_UNIT_HEADER_END + RESERVED_BYTES_AFTER_THE_HEADER + declared_length > unit_bytes) {
        return refused_header(RefusalOfTheWalk(REFUSED_STRUCTURE, DEFECT_DATA_DECLARED_LENGTH_PAST_THE_END));
    }
    var parsed: ParsedHeaderOfTheWalk;
    parsed.refusal = not_refused();
    parsed.class_tag = CLASS_TAG_DATA;
    parsed.birth_txg_low = unit_u32_at(unit_start + 75u);
    parsed.birth_txg_high = unit_u32_at(unit_start + 79u);
    // key：类标签 1、树 ID 8（偏移 43）、诞生代号 8（偏移 75）、写序 10（实例代号 4 加事务号 6，偏移 91）
    var key: array<u32, 7>;
    key[0] = CLASS_TAG_DATA;
    key = key_with_unit_bytes(key, 1u, unit_start + 43u, 8u);
    key = key_with_unit_bytes(key, 9u, unit_start + 75u, 8u);
    key = key_with_unit_bytes(key, 17u, unit_start + 91u, 10u);
    parsed.key_words = key;
    return parsed;
}

fn entry_carries_the_header_key(entry_position: u32, header_key: array<u32, 7>) -> bool {
    if (entries_of_the_mapping[entry_position].key_length != HEADER_KEY_BYTES) {
        return false;
    }
    var key_to_compare = header_key;
    for (var word = 0u; word < 7u; word = word + 1u) {
        if (entries_of_the_mapping[entry_position].key_words[word] != key_to_compare[word]) {
            return false;
        }
    }
    return true;
}

fn entry_points_at_the_record(entry_position: u32, descriptor: RecordDescriptorForTheWalk) -> bool {
    let mapping_entry = entries_of_the_mapping[entry_position];
    let first_points_here = mapping_entry.first_device == descriptor.device
        && mapping_entry.first_slot_low == descriptor.slot_low
        && mapping_entry.first_slot_high == descriptor.slot_high;
    let second_points_here = mapping_entry.second_device == descriptor.device
        && mapping_entry.second_slot_low == descriptor.slot_low
        && mapping_entry.second_slot_high == descriptor.slot_high;
    return first_points_here || second_points_here;
}

@compute @workgroup_size(64)
fn classify_records_walking_the_mapping_in_table_order(
    @builtin(global_invocation_id) global_invocation: vec3<u32>,
    @builtin(local_invocation_index) invocation_in_the_workgroup: u32,
) {
    for (var table_slot = invocation_in_the_workgroup; table_slot < 256u; table_slot = table_slot + WALK_INVOCATIONS_PER_WORKGROUP) {
        var remainder = table_slot;
        for (var bit_round = 0u; bit_round < 8u; bit_round = bit_round + 1u) {
            if ((remainder & 1u) == 1u) {
                remainder = (remainder >> 1u) ^ WALK_CASTAGNOLI_REFLECTED_POLYNOMIAL;
            } else {
                remainder = remainder >> 1u;
            }
        }
        remainder_table_of_the_walk[table_slot] = remainder;
    }
    workgroupBarrier();

    let record_position = global_invocation.x;
    if (record_position >= parameters_of_the_chunk.record_count) {
        return;
    }
    let descriptor = descriptors_of_the_records[record_position];
    var outcome: RecordOutcomeOfTheWalk;
    if (descriptor.span_slots != 1u && descriptor.span_slots != 2u) {
        outcome.outcome_kind = KIND_SPAN_IS_NOT_A_UNIT_FOOTPRINT;
        outcome.detail_word = descriptor.span_slots;
        outcomes_of_the_walk[record_position] = outcome;
        return;
    }
    if (descriptor.bytes_were_read == 0u) {
        outcome.outcome_kind = KIND_DEVICE_READ_FAILED;
        outcomes_of_the_walk[record_position] = outcome;
        return;
    }
    var parsed: ParsedHeaderOfTheWalk;
    if (descriptor.span_slots == 1u) {
        parsed = parsed_index_node_header(descriptor.byte_offset, descriptor.byte_length);
    } else {
        parsed = parsed_two_slot_unit_header(descriptor.byte_offset, descriptor.byte_length);
    }
    if (parsed.refusal.refusal_code != NOT_REFUSED) {
        outcome.outcome_kind = KIND_REFUSED_BY_THE_PARSER;
        outcome.detail_word = parsed.refusal.refusal_code;
        outcome.second_detail_word = parsed.refusal.refusal_detail;
        outcomes_of_the_walk[record_position] = outcome;
        return;
    }

    // 按条目表的次序走一遍：指着这个落点的有几条、头一条类标签不符的是哪个标签、指着的是不是都带头里那把 key、这把 key 在不在表里
    var entries_pointing_here = 0u;
    var first_disagreeing_class_tag_found = false;
    var first_disagreeing_class_tag = 0u;
    var every_entry_pointing_here_carries_the_header_key = true;
    var header_key_is_mapped = false;
    for (var entry_position = 0u; entry_position < parameters_of_the_chunk.mapping_entry_count; entry_position = entry_position + 1u) {
        let carries_the_header_key = entry_carries_the_header_key(entry_position, parsed.key_words);
        if (carries_the_header_key) {
            header_key_is_mapped = true;
        }
        if (!entry_points_at_the_record(entry_position, descriptor)) {
            continue;
        }
        entries_pointing_here = entries_pointing_here + 1u;
        if (!carries_the_header_key) {
            every_entry_pointing_here_carries_the_header_key = false;
        }
        // 空 key 的第一个字节按 0 报（核心层 `unwrap_or_default`），且总与头里的类标签不符
        let entry_has_a_key_byte = entries_of_the_mapping[entry_position].key_length != 0u;
        let entry_class_tag = select(0u, entries_of_the_mapping[entry_position].key_words[0] & 0xFFu, entry_has_a_key_byte);
        if (!first_disagreeing_class_tag_found && (!entry_has_a_key_byte || entry_class_tag != parsed.class_tag)) {
            first_disagreeing_class_tag_found = true;
            first_disagreeing_class_tag = entry_class_tag;
        }
    }
    if (first_disagreeing_class_tag_found) {
        outcome.outcome_kind = KIND_UNIT_CLASS_DISAGREES;
        outcome.detail_word = parsed.class_tag;
        outcome.second_detail_word = first_disagreeing_class_tag;
    } else if (entries_pointing_here == 0u && !header_key_is_mapped) {
        outcome.outcome_kind = KIND_DEAD;
    } else if (entries_pointing_here != 0u && every_entry_pointing_here_carries_the_header_key) {
        outcome.outcome_kind = KIND_MOVABLE;
        outcome.detail_word = parsed.class_tag;
        outcome.birth_txg_low = parsed.birth_txg_low;
        outcome.birth_txg_high = parsed.birth_txg_high;
        outcome.key_length = HEADER_KEY_BYTES;
        outcome.key_words = parsed.key_words;
    } else {
        outcome.outcome_kind = KIND_HEADER_KEY_DISAGREES_WITH_THE_LOCATION;
    }
    outcomes_of_the_walk[record_position] = outcome;
}
