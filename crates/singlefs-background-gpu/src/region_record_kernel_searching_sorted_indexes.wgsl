// 区域扫描里的单元头解析与映射反查，两份独立核里在两张排好序的索引上二分查映射的那一份（D26（后台整理与放置回收） 已定项 13 第 8 条：两份核不共用核函数）。
// 每条记录一个调用：先按单元的类定下几处偏移（码 2 索引节点的头宽随 key 宽变），再逐关核（共同前缀、头校验和、载荷校验和、预留位、结构字段），
// 校验和逐位移算 CRC-32C、不查表；映射反查在落点索引上二分找这个落点的第一项、往后读完同一落点的那几项（按条目序号升序，即条目表的次序），
// 在按 key 排的序号上二分找头里那把 key。偏移与宽度是这一份自己照字节表写的（D18（块里携带什么信息） 已定项 7、11、18），不读别处。
// 另一份（region_record_kernel_walking_the_mapping.wgsl）查表算校验和、逐条走条目表；两份只共用缓冲布局，布局由 src/region_record_kernels.rs 定。

struct SearchChunkParameters {
    number_of_records: u32,
    number_of_mapping_entries: u32,
    number_of_location_items: u32,
    word_left_unused: u32,
}

struct SearchRecordDescriptor {
    first_byte: u32,
    length_in_bytes: u32,
    span_in_slots: u32,
    read_succeeded: u32,
    device_of_the_record: u32,
    slot_low_word: u32,
    slot_high_word: u32,
}

struct SearchMappingEntry {
    length_of_the_key: u32,
    words_of_the_key: array<u32, 7>,
    device_of_the_first_location: u32,
    slot_low_word_of_the_first_location: u32,
    slot_high_word_of_the_first_location: u32,
    device_of_the_second_location: u32,
    slot_low_word_of_the_second_location: u32,
    slot_high_word_of_the_second_location: u32,
}

struct SearchLocationItem {
    device_of_the_location: u32,
    slot_low_word: u32,
    slot_high_word: u32,
    position_of_the_entry: u32,
}

struct SearchRecordOutcome {
    kind_of_the_outcome: u32,
    first_detail: u32,
    second_detail: u32,
    birth_generation_low_word: u32,
    birth_generation_high_word: u32,
    length_of_the_key: u32,
    words_of_the_key: array<u32, 7>,
}

@group(0) @binding(0) var<uniform> search_parameters: SearchChunkParameters;
@group(0) @binding(1) var<storage, read> bytes_of_the_units: array<u32>;
@group(0) @binding(2) var<storage, read> record_descriptors_to_search: array<SearchRecordDescriptor>;
@group(0) @binding(3) var<storage, read> mapping_entry_table: array<SearchMappingEntry>;
@group(0) @binding(4) var<storage, read> location_index_items: array<SearchLocationItem>;
@group(0) @binding(5) var<storage, read> entry_positions_sorted_by_key: array<u32>;
@group(0) @binding(6) var<storage, read_write> outcomes_of_the_search: array<SearchRecordOutcome>;

// 结果的类、解析拒收的码与结构缺陷的码：与 src/region_record_kernels.rs 同值。
const OUTCOME_MOVABLE: u32 = 0u;
const OUTCOME_DEAD: u32 = 1u;
const OUTCOME_CLASS_TAG_DISAGREES: u32 = 2u;
const OUTCOME_KEY_DISAGREES_WITH_THE_LOCATION: u32 = 3u;
const OUTCOME_READ_FAILED: u32 = 4u;
const OUTCOME_SPAN_NOT_A_UNIT: u32 = 5u;
const OUTCOME_PARSER_REFUSED: u32 = 6u;
const PARSER_ACCEPTED: u32 = 0xFFFFFFFFu;
const PARSER_TOO_SHORT: u32 = 0u;
const PARSER_BAD_MAGIC: u32 = 1u;
const PARSER_UNKNOWN_VERSION: u32 = 2u;
const PARSER_NON_ZERO_FLAGS: u32 = 3u;
const PARSER_WRONG_CLASS: u32 = 4u;
const PARSER_HEADER_CRC_MISMATCH: u32 = 5u;
const PARSER_PAYLOAD_CRC_MISMATCH: u32 = 6u;
const PARSER_RESERVED_BYTES_NOT_ZERO: u32 = 7u;
const PARSER_STRUCTURE: u32 = 8u;

const REFLECTED_POLYNOMIAL_OF_CASTAGNOLI: u32 = 0x82F63B78u;
// 单元格式（照字节表）。
const BYTES_OF_AN_INDEX_NODE: u32 = 16384u;
const BYTES_OF_A_DATA_OR_PACKED_UNIT: u32 = 32768u;
const UNIT_MAGIC_BYTES_AS_A_WORD: u32 = 0x55534653u;
const TAG_OF_A_DATA_UNIT: u32 = 1u;
const TAG_OF_AN_INDEX_NODE: u32 = 2u;
const TAG_OF_A_PACKED_UNIT: u32 = 3u;
const WIDE_CHECKSUM_FIELD_OFFSET: u32 = 10u;
const WIDE_CHECKSUM_FIELD_WIDTH: u32 = 32u;
const NONCE_MAC_ALGORITHM_RESERVED_WIDTH: u32 = 29u;
const LENGTH_OF_A_MAPPING_KEY: u32 = 27u;

// 一种单元的解法：头宽、载荷 CRC 在哪、类身份段里的类标签副本在哪（没有副本的码 2 填 0xFFFFFFFF）。
struct UnitLayoutToSearch {
    expected_tag: u32,
    header_end: u32,
    payload_crc_at: u32,
    tag_copy_at: u32,
}

struct HeaderVerdictOfTheSearch {
    refusal_code: u32,
    refusal_detail: u32,
    tag_of_the_header: u32,
    birth_generation_low_word: u32,
    birth_generation_high_word: u32,
    words_of_the_header_key: array<u32, 7>,
}

fn unit_byte_numbered(byte_number: u32) -> u32 {
    return extractBits(bytes_of_the_units[byte_number / 4u], (byte_number % 4u) * 8u, 8u);
}

fn little_endian_value_of_bytes(first_byte_number: u32, byte_count: u32) -> u32 {
    var value = 0u;
    for (var counted = byte_count; counted > 0u; counted = counted - 1u) {
        value = (value << 8u) | unit_byte_numbered(first_byte_number + counted - 1u);
    }
    return value;
}

fn register_after_one_byte(register_before: u32, absorbed_byte: u32) -> u32 {
    var crc_register = register_before ^ absorbed_byte;
    for (var bit = 0u; bit < 8u; bit = bit + 1u) {
        let mask_of_the_lowest_bit = 0u - (crc_register & 1u);
        crc_register = (crc_register >> 1u) ^ (REFLECTED_POLYNOMIAL_OF_CASTAGNOLI & mask_of_the_lowest_bit);
    }
    return crc_register;
}

fn register_after_unit_bytes(register_before: u32, first_byte_number: u32, past_the_last_byte_number: u32) -> u32 {
    var crc_register = register_before;
    var byte_number = first_byte_number;
    while (byte_number < past_the_last_byte_number) {
        crc_register = register_after_one_byte(crc_register, unit_byte_numbered(byte_number));
        byte_number = byte_number + 1u;
    }
    return crc_register;
}

fn register_after_zero_bytes(register_before: u32, zero_byte_count: u32) -> u32 {
    var crc_register = register_before;
    for (var absorbed = 0u; absorbed < zero_byte_count; absorbed = absorbed + 1u) {
        crc_register = register_after_one_byte(crc_register, 0u);
    }
    return crc_register;
}

fn rejected_header(code: u32, detail: u32) -> HeaderVerdictOfTheSearch {
    var verdict: HeaderVerdictOfTheSearch;
    verdict.refusal_code = code;
    verdict.refusal_detail = detail;
    return verdict;
}

// 校验和字段：前 4 字节小端是 CRC，其余 28 字节必须是 0；算 CRC 时整个字段按 0 参与。
fn header_checksum_field_holds(start: u32, header_end: u32) -> bool {
    var crc_register = register_after_unit_bytes(0xFFFFFFFFu, start, start + WIDE_CHECKSUM_FIELD_OFFSET);
    crc_register = register_after_zero_bytes(crc_register, WIDE_CHECKSUM_FIELD_WIDTH);
    crc_register = register_after_unit_bytes(
        crc_register,
        start + WIDE_CHECKSUM_FIELD_OFFSET + WIDE_CHECKSUM_FIELD_WIDTH,
        start + header_end,
    );
    if (little_endian_value_of_bytes(start + WIDE_CHECKSUM_FIELD_OFFSET, 4u) != ~crc_register) {
        return false;
    }
    var zero_tail_holds = true;
    for (var tail_byte = 4u; tail_byte < WIDE_CHECKSUM_FIELD_WIDTH; tail_byte = tail_byte + 1u) {
        zero_tail_holds = zero_tail_holds && unit_byte_numbered(start + WIDE_CHECKSUM_FIELD_OFFSET + tail_byte) == 0u;
    }
    return zero_tail_holds;
}

// 码 2 的布局随 key 宽：头宽 86 + 2 × key 宽，载荷 CRC 在头末尾前 10 字节，没有类标签副本。
fn layout_of_an_index_node(start: u32) -> UnitLayoutToSearch {
    let header_end = 86u + 2u * unit_byte_numbered(start + 51u);
    return UnitLayoutToSearch(TAG_OF_AN_INDEX_NODE, header_end, header_end - 10u, 0xFFFFFFFFu);
}

fn layout_of_a_data_unit() -> UnitLayoutToSearch {
    return UnitLayoutToSearch(TAG_OF_A_DATA_UNIT, 105u, 101u, 42u);
}

fn layout_of_a_packed_unit() -> UnitLayoutToSearch {
    return UnitLayoutToSearch(TAG_OF_A_PACKED_UNIT, 107u, 89u, 42u);
}

// 逐关核：magic、版本、类标签、flags、头校验和、载荷校验和、预留位、类标签副本。全过交回 PARSER_ACCEPTED。
fn checks_shared_by_every_unit_class(start: u32, unit_length: u32, unit_layout: UnitLayoutToSearch) -> HeaderVerdictOfTheSearch {
    if (little_endian_value_of_bytes(start, 4u) != UNIT_MAGIC_BYTES_AS_A_WORD) {
        return rejected_header(PARSER_BAD_MAGIC, 0u);
    }
    if (little_endian_value_of_bytes(start + 4u, 2u) != 1u) {
        return rejected_header(PARSER_UNKNOWN_VERSION, 0u);
    }
    let tag_in_the_prefix = unit_byte_numbered(start + 6u);
    if (tag_in_the_prefix != unit_layout.expected_tag) {
        return rejected_header(PARSER_WRONG_CLASS, insertBits(unit_layout.expected_tag, tag_in_the_prefix, 8u, 8u));
    }
    if (unit_byte_numbered(start + 7u) != 0u) {
        return rejected_header(PARSER_NON_ZERO_FLAGS, 0u);
    }
    if (!header_checksum_field_holds(start, unit_layout.header_end)) {
        return rejected_header(PARSER_HEADER_CRC_MISMATCH, 0u);
    }
    let payload_register = register_after_unit_bytes(0xFFFFFFFFu, start + unit_layout.header_end, start + unit_length);
    if (little_endian_value_of_bytes(start + unit_layout.payload_crc_at, 4u) != ~payload_register) {
        return rejected_header(PARSER_PAYLOAD_CRC_MISMATCH, 0u);
    }
    var reserved_bytes_are_zero = true;
    for (var reserved = 0u; reserved < NONCE_MAC_ALGORITHM_RESERVED_WIDTH; reserved = reserved + 1u) {
        reserved_bytes_are_zero = reserved_bytes_are_zero && unit_byte_numbered(start + unit_layout.header_end + reserved) == 0u;
    }
    if (!reserved_bytes_are_zero) {
        return rejected_header(PARSER_RESERVED_BYTES_NOT_ZERO, 0u);
    }
    var verdict: HeaderVerdictOfTheSearch;
    verdict.refusal_code = PARSER_ACCEPTED;
    verdict.tag_of_the_header = unit_layout.expected_tag;
    return verdict;
}

// key 的第 position 个字节放进字里（字先是 0）。
fn key_words_with_byte(words_before: array<u32, 7>, position: u32, byte_value: u32) -> array<u32, 7> {
    var words_after = words_before;
    let word_number = position / 4u;
    words_after[word_number] = insertBits(words_after[word_number], byte_value, (position % 4u) * 8u, 8u);
    return words_after;
}

// 依次把几段单元字节接成 key：类标签之后接 field_starts 里每段 field_widths 个字节。
fn key_words_of_the_fields(tag: u32, field_starts: vec4<u32>, field_widths: vec4<u32>) -> array<u32, 7> {
    var words = array<u32, 7>(tag, 0u, 0u, 0u, 0u, 0u, 0u);
    var position_in_the_key = 1u;
    for (var field = 0u; field < 4u; field = field + 1u) {
        for (var byte_of_the_field = 0u; byte_of_the_field < field_widths[field]; byte_of_the_field = byte_of_the_field + 1u) {
            words = key_words_with_byte(words, position_in_the_key, unit_byte_numbered(field_starts[field] + byte_of_the_field));
            position_in_the_key = position_in_the_key + 1u;
        }
    }
    return words;
}

fn verdict_on_an_index_node(start: u32, unit_length: u32) -> HeaderVerdictOfTheSearch {
    if (unit_length != BYTES_OF_AN_INDEX_NODE) {
        return rejected_header(PARSER_TOO_SHORT, 0u);
    }
    let unit_layout = layout_of_an_index_node(start);
    var verdict = checks_shared_by_every_unit_class(start, unit_length, unit_layout);
    if (verdict.refusal_code != PARSER_ACCEPTED) {
        return verdict;
    }
    let width_of_a_key = unit_byte_numbered(start + 51u);
    let birth_generation_at = start + 52u + 2u * width_of_a_key;
    let declared = little_endian_value_of_bytes(start + 8u, 2u);
    let count_of_entries = little_endian_value_of_bytes(unit_layout.header_end - 4u + start, 2u);
    let width_of_an_entry = little_endian_value_of_bytes(unit_layout.header_end - 2u + start, 2u);
    if (count_of_entries * width_of_an_entry != declared) {
        return rejected_header(PARSER_STRUCTURE, 0u);
    }
    if (width_of_an_entry == 0u && count_of_entries != 0u) {
        return rejected_header(PARSER_STRUCTURE, 1u);
    }
    if (width_of_an_entry < width_of_a_key) {
        return rejected_header(PARSER_STRUCTURE, 2u);
    }
    if (unit_layout.header_end + NONCE_MAC_ALGORITHM_RESERVED_WIDTH + declared > unit_length) {
        return rejected_header(PARSER_STRUCTURE, 3u);
    }
    verdict.birth_generation_low_word = little_endian_value_of_bytes(birth_generation_at, 4u);
    verdict.birth_generation_high_word = little_endian_value_of_bytes(birth_generation_at + 4u, 4u);
    // 树 ID 在偏移 42；诞生代号之后隔 fsid 8 字节是实例代号与出生序号；末尾两字节留 0
    verdict.words_of_the_header_key = key_words_of_the_fields(
        TAG_OF_AN_INDEX_NODE,
        vec4<u32>(start + 42u, birth_generation_at, birth_generation_at + 16u, 0u),
        vec4<u32>(8u, 8u, 8u, 0u),
    );
    return verdict;
}

fn verdict_on_a_packed_unit(start: u32, unit_length: u32) -> HeaderVerdictOfTheSearch {
    let unit_layout = layout_of_a_packed_unit();
    var verdict = checks_shared_by_every_unit_class(start, unit_length, unit_layout);
    if (verdict.refusal_code != PARSER_ACCEPTED) {
        return verdict;
    }
    if (unit_byte_numbered(start + unit_layout.tag_copy_at) != TAG_OF_A_PACKED_UNIT) {
        return rejected_header(PARSER_STRUCTURE, 4u);
    }
    let declared = little_endian_value_of_bytes(start + 8u, 2u);
    let count_of_records = little_endian_value_of_bytes(start + 69u, 2u);
    let width_of_a_record = little_endian_value_of_bytes(start + 71u, 2u);
    if (count_of_records * width_of_a_record != declared) {
        return rejected_header(PARSER_STRUCTURE, 5u);
    }
    if (width_of_a_record == 0u && count_of_records != 0u) {
        return rejected_header(PARSER_STRUCTURE, 6u);
    }
    if (unit_layout.header_end + NONCE_MAC_ALGORITHM_RESERVED_WIDTH + declared > unit_length) {
        return rejected_header(PARSER_STRUCTURE, 7u);
    }
    verdict.birth_generation_low_word = little_endian_value_of_bytes(start + 73u, 4u);
    verdict.birth_generation_high_word = little_endian_value_of_bytes(start + 77u, 4u);
    // 出生树（偏移 43）、诞生代号（73）、写序里的实例代号（93）、出生序号（103）；末尾两字节留 0
    verdict.words_of_the_header_key = key_words_of_the_fields(
        TAG_OF_A_PACKED_UNIT,
        vec4<u32>(start + 43u, start + 73u, start + 93u, start + 103u),
        vec4<u32>(8u, 8u, 4u, 4u),
    );
    return verdict;
}

fn verdict_on_a_two_slot_unit(start: u32, unit_length: u32) -> HeaderVerdictOfTheSearch {
    if (unit_length != BYTES_OF_A_DATA_OR_PACKED_UNIT) {
        return rejected_header(PARSER_TOO_SHORT, 0u);
    }
    let unit_layout = layout_of_a_data_unit();
    var verdict = checks_shared_by_every_unit_class(start, unit_length, unit_layout);
    // 类标签是 3：当码 3 重解（核心层区域扫描的读法）
    if (verdict.refusal_code == PARSER_WRONG_CLASS && extractBits(verdict.refusal_detail, 8u, 8u) == TAG_OF_A_PACKED_UNIT) {
        return verdict_on_a_packed_unit(start, unit_length);
    }
    if (verdict.refusal_code != PARSER_ACCEPTED) {
        return verdict;
    }
    if (unit_byte_numbered(start + unit_layout.tag_copy_at) != TAG_OF_A_DATA_UNIT) {
        return rejected_header(PARSER_STRUCTURE, 8u);
    }
    let declared = little_endian_value_of_bytes(start + 8u, 2u);
    if (unit_layout.header_end + NONCE_MAC_ALGORITHM_RESERVED_WIDTH + declared > unit_length) {
        return rejected_header(PARSER_STRUCTURE, 9u);
    }
    verdict.birth_generation_low_word = little_endian_value_of_bytes(start + 75u, 4u);
    verdict.birth_generation_high_word = little_endian_value_of_bytes(start + 79u, 4u);
    // 树 ID（偏移 43）、诞生代号（75）、写序 10 字节（91 起：实例代号 4、事务号 6）
    verdict.words_of_the_header_key = key_words_of_the_fields(
        TAG_OF_A_DATA_UNIT,
        vec4<u32>(start + 43u, start + 75u, start + 91u, 0u),
        vec4<u32>(8u, 8u, 10u, 0u),
    );
    return verdict;
}

// 落点索引里排在（盘、槽号）之前的项：先比盘，再比槽号高 32 位、低 32 位。
fn location_item_comes_before(item_number: u32, device: u32, slot_high_word: u32, slot_low_word: u32) -> bool {
    let item = location_index_items[item_number];
    if (item.device_of_the_location != device) {
        return item.device_of_the_location < device;
    }
    if (item.slot_high_word != slot_high_word) {
        return item.slot_high_word < slot_high_word;
    }
    return item.slot_low_word < slot_low_word;
}

fn location_item_is_at(item_number: u32, device: u32, slot_high_word: u32, slot_low_word: u32) -> bool {
    let item = location_index_items[item_number];
    return item.device_of_the_location == device && item.slot_high_word == slot_high_word && item.slot_low_word == slot_low_word;
}

// 条目的 key 比头里那把 key：先比字节数，再逐字比；小于交 -1、相等交 0、大于交 1（与 CPU 排序用的次序相同）。
fn ordering_of_the_entry_key(entry_number: u32, header_key_words: array<u32, 7>) -> i32 {
    let entry_key_length = mapping_entry_table[entry_number].length_of_the_key;
    if (entry_key_length != LENGTH_OF_A_MAPPING_KEY) {
        return select(1i, -1i, entry_key_length < LENGTH_OF_A_MAPPING_KEY);
    }
    var header_words = header_key_words;
    for (var word_number = 0u; word_number < 7u; word_number = word_number + 1u) {
        let entry_word = mapping_entry_table[entry_number].words_of_the_key[word_number];
        if (entry_word != header_words[word_number]) {
            return select(1i, -1i, entry_word < header_words[word_number]);
        }
    }
    return 0i;
}

@compute @workgroup_size(64)
fn classify_records_searching_the_sorted_mapping_indexes(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let record_number = invocation.x;
    if (record_number >= search_parameters.number_of_records) {
        return;
    }
    let record = record_descriptors_to_search[record_number];
    var outcome: SearchRecordOutcome;
    var header: HeaderVerdictOfTheSearch;
    switch (record.span_in_slots) {
        case 1u: {
            if (record.read_succeeded == 0u) {
                outcome.kind_of_the_outcome = OUTCOME_READ_FAILED;
                outcomes_of_the_search[record_number] = outcome;
                return;
            }
            header = verdict_on_an_index_node(record.first_byte, record.length_in_bytes);
        }
        case 2u: {
            if (record.read_succeeded == 0u) {
                outcome.kind_of_the_outcome = OUTCOME_READ_FAILED;
                outcomes_of_the_search[record_number] = outcome;
                return;
            }
            header = verdict_on_a_two_slot_unit(record.first_byte, record.length_in_bytes);
        }
        default: {
            outcome.kind_of_the_outcome = OUTCOME_SPAN_NOT_A_UNIT;
            outcome.first_detail = record.span_in_slots;
            outcomes_of_the_search[record_number] = outcome;
            return;
        }
    }
    if (header.refusal_code != PARSER_ACCEPTED) {
        outcome.kind_of_the_outcome = OUTCOME_PARSER_REFUSED;
        outcome.first_detail = header.refusal_code;
        outcome.second_detail = header.refusal_detail;
        outcomes_of_the_search[record_number] = outcome;
        return;
    }

    // 落点索引上二分：第一项不排在这个落点之前的位置
    var lower = 0u;
    var upper = search_parameters.number_of_location_items;
    while (lower < upper) {
        let middle = lower + (upper - lower) / 2u;
        if (location_item_comes_before(middle, record.device_of_the_record, record.slot_high_word, record.slot_low_word)) {
            lower = middle + 1u;
        } else {
            upper = middle;
        }
    }
    var number_pointing_here = 0u;
    var disagreeing_tag_seen = false;
    var disagreeing_tag = 0u;
    var all_pointing_here_carry_the_header_key = true;
    var item_number = lower;
    while (item_number < search_parameters.number_of_location_items
        && location_item_is_at(item_number, record.device_of_the_record, record.slot_high_word, record.slot_low_word)) {
        let entry_number = location_index_items[item_number].position_of_the_entry;
        number_pointing_here = number_pointing_here + 1u;
        if (ordering_of_the_entry_key(entry_number, header.words_of_the_header_key) != 0) {
            all_pointing_here_carry_the_header_key = false;
        }
        let key_length_of_the_entry = mapping_entry_table[entry_number].length_of_the_key;
        var tag_of_the_entry = 0u;
        if (key_length_of_the_entry > 0u) {
            tag_of_the_entry = extractBits(mapping_entry_table[entry_number].words_of_the_key[0], 0u, 8u);
        }
        if (!disagreeing_tag_seen && (key_length_of_the_entry == 0u || tag_of_the_entry != header.tag_of_the_header)) {
            disagreeing_tag_seen = true;
            disagreeing_tag = tag_of_the_entry;
        }
        item_number = item_number + 1u;
    }

    // 按 key 排的序号上二分找头里那把 key
    var key_lower = 0u;
    var key_upper = search_parameters.number_of_mapping_entries;
    var header_key_found = false;
    while (key_lower < key_upper && !header_key_found) {
        let key_middle = key_lower + (key_upper - key_lower) / 2u;
        let ordering = ordering_of_the_entry_key(entry_positions_sorted_by_key[key_middle], header.words_of_the_header_key);
        if (ordering == 0) {
            header_key_found = true;
        } else if (ordering < 0) {
            key_lower = key_middle + 1u;
        } else {
            key_upper = key_middle;
        }
    }

    if (disagreeing_tag_seen) {
        outcome.kind_of_the_outcome = OUTCOME_CLASS_TAG_DISAGREES;
        outcome.first_detail = header.tag_of_the_header;
        outcome.second_detail = disagreeing_tag;
    } else if (number_pointing_here == 0u) {
        if (header_key_found) {
            outcome.kind_of_the_outcome = OUTCOME_KEY_DISAGREES_WITH_THE_LOCATION;
        } else {
            outcome.kind_of_the_outcome = OUTCOME_DEAD;
        }
    } else if (all_pointing_here_carry_the_header_key) {
        outcome.kind_of_the_outcome = OUTCOME_MOVABLE;
        outcome.first_detail = header.tag_of_the_header;
        outcome.birth_generation_low_word = header.birth_generation_low_word;
        outcome.birth_generation_high_word = header.birth_generation_high_word;
        outcome.length_of_the_key = LENGTH_OF_A_MAPPING_KEY;
        outcome.words_of_the_key = header.words_of_the_header_key;
    } else {
        outcome.kind_of_the_outcome = OUTCOME_KEY_DISAGREES_WITH_THE_LOCATION;
    }
    outcomes_of_the_search[record_number] = outcome;
}
