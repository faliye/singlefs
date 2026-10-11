// GPU 判器的池级 checker 走读内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：按 singlefs-checker 自己的读法
// 判第一批（系统配置、根环、journal）与第二批（走读那 29 条），连同里程碑三二·1 那一批里不在扫描内核的十五条（I-1.12、I-8.10、I-8.11、
// I-4.9、I-9.25、I-3.13、I-9.26、I-7.16、I-7.17、I-8.12、I-7.15、I-4.1、I-8.1、I-7.5、I-8.5）与里程碑三·3 墓碑树、谱系树条目身份那两条
// （I-9.23、I-9.17），违例位写进每个状态结论的违例位那几个字
// （IMPLEMENTED_INVARIANTS 次序），之后一字是走到的批次、再一字是判不了的原因；走完把扫描内核要的计数与几何写进草稿区的交接段
// （第三批与 I-3.12 在 crash_judge_checker_scan.wgsl）。


// ───────────────────────── checker 读系统配置槽 ─────────────────────────

// 一槽读下来：0 可择、1 带这一版读者不收的值（I-7.13 违例）、2 不自证（读不到、magic 或整槽校验和不对）、
// 3 自证过而不可择（incompat 位不认得、R 或 S 越界；I-8.5 把它数进自证过的槽）
const SLOT_USABLE: u32 = 0u;
const SLOT_CARRIES_A_REFUSED_VALUE: u32 = 1u;
const SLOT_NOT_SELF_VERIFIED: u32 = 2u;
const SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE: u32 = 3u;

struct CheckerSlot {
    reading: u32,
    fsid: vec4<u32>,
    this_device: u32,
    device_count: u32,
    generation: vec2<u32>,
    physical_block_size: u32,
    journal_instance: u32,
    floor: vec2<u32>,
    regions: u32,
    slots_per_region: u32,
    prime_step: u32,
    chunk_bytes: u32,
    base_slot: vec2<u32>,
    region0: u32,
    region1: u32,
    region2: u32,
    unit_area_start_slot: vec2<u32>,
    spacing: u32,
    ring_start_slot: vec2<u32>,
    ring_bytes: vec2<u32>,
    // I-7.5 的判定宽度要的 io_min（偏移 425，0 = 没声明）与 I-8.1 的三个字段：在飞上限 345、最坏占用 349、安全系数 F 357
    minimum_input_output_bytes: u32,
    declared_in_flight_record_limit: u32,
    declared_worst_case_occupancy_bytes: vec2<u32>,
    declared_safety_factor: u32,
};

fn empty_checker_slot(reading: u32) -> CheckerSlot {
    var c: CheckerSlot;
    c.reading = reading;
    c.fsid = vec4<u32>(0u, 0u, 0u, 0u);
    c.this_device = 0u;
    c.device_count = 0u;
    c.generation = vec2<u32>(0u, 0u);
    c.physical_block_size = 0u;
    c.journal_instance = 0u;
    c.floor = vec2<u32>(0u, 0u);
    c.regions = 0u;
    c.slots_per_region = 0u;
    c.prime_step = 0u;
    c.chunk_bytes = 0u;
    c.base_slot = vec2<u32>(0u, 0u);
    c.region0 = 0u;
    c.region1 = 0u;
    c.region2 = 0u;
    c.unit_area_start_slot = vec2<u32>(0u, 0u);
    c.spacing = 0u;
    c.ring_start_slot = vec2<u32>(0u, 0u);
    c.ring_bytes = vec2<u32>(0u, 0u);
    c.minimum_input_output_bytes = 0u;
    c.declared_in_flight_record_limit = 0u;
    c.declared_worst_case_occupancy_bytes = vec2<u32>(0u, 0u);
    c.declared_safety_factor = 0u;
    return c;
}

// image::read_system_configuration_slot：check_system_configuration_slot 再 geometry_of，第一条不成立的规则定这一槽是「拒」还是「不可择」
fn checker_read_system_configuration_slot(device: u32, offset: u32) -> CheckerSlot {
    let s = read_at(device, u64_of(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES) {
        return empty_checker_slot(SLOT_NOT_SELF_VERIFIED);
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
        return empty_checker_slot(SLOT_NOT_SELF_VERIFIED);
    }
    if (rd16(base, 4u) != 1u) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    let incompat0 = rd8(base, 6u);
    if ((incompat0 & 0xFDu) != 0u || !all_zero(base, 7u, 31u) || (incompat0 & 2u) == 0u) {
        return empty_checker_slot(SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE);
    }
    if (rd8(base, 219u) != 0u) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    // geometry_of
    let regions = rd8(base, 361u);
    if (regions > 3u) {
        return empty_checker_slot(SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE);
    }
    let slots_per_region = rd8(base, 362u);
    if (slots_per_region < 4u || slots_per_region > 16u) {
        return empty_checker_slot(SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE);
    }
    let ring_start_slot = rd64(base, 325u);
    let ring_bytes = rd64(base, 333u);
    let base_slot = rd64(base, 371u);
    let unit_area_start_slot = rd64(base, 417u);
    let spacing = rd32(base, 429u);
    if (!eq64(ring_start_slot, u64_of(1024u))) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    if (!eq64(base_slot, u64_of(64u))) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    // 槽距 ≥ 4096 且槽 1 整槽落在根环起点之前（基址乘出来溢出算成立）
    var spacing_holds = spacing >= 4096u;
    if (base_slot.y < (1u << 18u)) {
        let base_bytes = slot_to_bytes(base_slot);
        let slot_one_end = add64(u64_of(spacing), u64_of(SYSTEM_CONFIGURATION_SLOT_BYTES));
        if (lt64(base_bytes, slot_one_end)) {
            spacing_holds = false;
        }
    }
    if (!spacing_holds) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    let physical_block_size = rd32(base, 317u);
    if (physical_block_size < ROOT_RECORD_BYTES || physical_block_size > spacing) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    // 环长：在飞上限 ≥ 1，环末端（起点 × 16384 + 环长，溢出算越界）≤ 单元区起点 × 16384（溢出算成立）
    let in_flight = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    var ring_holds = !is_zero64(in_flight);
    let ring_start_bytes = slot_to_bytes(ring_start_slot);
    if (add64_overflows(ring_start_bytes, ring_bytes)) {
        ring_holds = false;
    } else {
        let ring_end = add64(ring_start_bytes, ring_bytes);
        if (unit_area_start_slot.y < (1u << 18u) && lt64(slot_to_bytes(unit_area_start_slot), ring_end)) {
            ring_holds = false;
        }
    }
    if (!ring_holds) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    let ring_slots = shr64(add64_saturating(ring_bytes, u64_of(16383u)), SLOT_SHIFT);
    if (add64_overflows(ring_start_slot, ring_slots) || !eq64(add64(ring_start_slot, ring_slots), unit_area_start_slot)) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    if ((unit_area_start_slot.x & 63u) != 0u) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    var c = empty_checker_slot(SLOT_USABLE);
    c.fsid = vec4<u32>(rd32(base, 102u), rd32(base, 106u), rd32(base, 110u), rd32(base, 114u));
    c.this_device = rd32(base, 139u);
    c.device_count = rd32(base, 143u);
    c.generation = rd64(base, 147u);
    c.physical_block_size = physical_block_size;
    c.journal_instance = rd32(base, 477u);
    c.floor = rd64(base, 481u);
    c.regions = regions;
    c.slots_per_region = slots_per_region;
    c.prime_step = rd32(base, 363u);
    c.chunk_bytes = rd32(base, 367u);
    c.base_slot = base_slot;
    c.region0 = rd32(base, 379u);
    c.region1 = rd32(base, 383u);
    c.region2 = rd32(base, 387u);
    c.unit_area_start_slot = unit_area_start_slot;
    c.spacing = spacing;
    c.ring_start_slot = ring_start_slot;
    c.ring_bytes = ring_bytes;
    c.minimum_input_output_bytes = rd32(base, 425u);
    c.declared_in_flight_record_limit = rd32(base, 345u);
    c.declared_worst_case_occupancy_bytes = rd64(base, 349u);
    c.declared_safety_factor = rd32(base, 357u);
    return c;
}

// I-8.1 的三个字段等式（image::judge_journal_ring_geometry_fields_agree）：F = JOURNAL_SAFETY_FACTOR（3）、
// 在飞上限 = ⌊环长 ÷ 4096 ÷ F⌋（F 取同一槽里的那个字段；它等于 3 才走到这一格，不会是 0）、最坏占用 = 在飞上限 × 4096
// （在飞上限是 4 字节字段，乘 4096 装得进 u64，不溢出）
fn journal_ring_geometry_fields_agree(slot: CheckerSlot) -> bool {
    if (slot.declared_safety_factor != 3u) {
        return false;
    }
    let in_flight_record_limit_from_the_ring = divmod64(shr64(slot.ring_bytes, 12u), u64_of(slot.declared_safety_factor)).quotient;
    if (!eq64(in_flight_record_limit_from_the_ring, u64_of(slot.declared_in_flight_record_limit))) {
        return false;
    }
    return eq64(shl64(u64_of(slot.declared_in_flight_record_limit), 12u), slot.declared_worst_case_occupancy_bytes);
}

fn same_fsid(a: vec4<u32>, b: vec4<u32>) -> bool {
    return a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w;
}

fn checker_region_device(g: CheckerSlot, region: u32) -> u32 {
    if (region == 0u) {
        return g.region0;
    }
    if (region == 1u) {
        return g.region1;
    }
    return g.region2;
}

// 全池没有一个可用的槽 0 时逐档读一块盘的槽 1（image::slot_one_read_at_every_slot_spacing_the_format_allows）：4096 起、每 512 一档、
// 到根环起点（这一版读者只收 64 × 16384）减 4096 为止；只收可用、且记的槽距等于所在偏移的，取世代号最大的、同号取偏移小的；
// 读到带不收之值的那一档按偏移升序交回第一个；读到可用的一档先判 I-8.1 三个字段等式（实现那张不收之值表的最后三格，代码轮第二轮判决 K3-1），
// 不成立同样当场交回带不收之值、不核偏移不比世代号；一档可用的都没收到时，见过自证过而不可择的就交回那一类（I-8.5 数它），再没有就是不自证
fn checker_slot_one_by_reading_every_spacing(device: u32) -> CheckerSlot {
    var best = empty_checker_slot(2u);
    var have_best = false;
    var some_slot_verifies_itself_but_is_not_choosable = false;
    let largest = ROOT_RING_BASE_BYTES - SYSTEM_CONFIGURATION_SLOT_BYTES;
    for (var candidate = 4096u; candidate <= largest; candidate = candidate + 512u) {
        let s = checker_read_system_configuration_slot(device, candidate);
        if (s.reading == 1u) {
            return s;
        }
        if (s.reading == SLOT_USABLE && !journal_ring_geometry_fields_agree(s)) {
            return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
        }
        if (s.reading == 0u && s.spacing == candidate) {
            if (!have_best || lt64(best.generation, s.generation)) {
                best = s;
                have_best = true;
            }
        }
        if (s.reading == SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE) {
            some_slot_verifies_itself_but_is_not_choosable = true;
        }
    }
    if (!have_best && some_slot_verifies_itself_but_is_not_choosable) {
        return empty_checker_slot(SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE);
    }
    return best;
}

// 每盘两槽的读法（image::system_configuration_slot_readings）：槽 1 的偏移按这块盘槽 0 记的槽距，不可用时按第一块槽 0 可用的盘的，
// 全池都不可用时逐档读（checker_slot_one_by_reading_every_spacing）
var<private> slot_readings: array<CheckerSlot, 16>;

fn read_every_system_configuration_slot() {
    let device_count = hdr(3u);
    var first_spacing = NONE;
    for (var d = 0u; d < device_count; d = d + 1u) {
        slot_readings[d * 2u] = checker_read_system_configuration_slot(hdr(12u + d), 0u);
        if (slot_readings[d * 2u].reading == 0u && first_spacing == NONE) {
            first_spacing = slot_readings[d * 2u].spacing;
        }
    }
    for (var d = 0u; d < device_count; d = d + 1u) {
        var spacing = first_spacing;
        if (slot_readings[d * 2u].reading == 0u) {
            spacing = slot_readings[d * 2u].spacing;
        }
        if (spacing != NONE) {
            slot_readings[d * 2u + 1u] = checker_read_system_configuration_slot(hdr(12u + d), spacing);
        } else {
            slot_readings[d * 2u + 1u] = checker_slot_one_by_reading_every_spacing(hdr(12u + d));
        }
    }
}

// ───────────────────────── 根环（checker 的读法）─────────────────────────

// image::valid_roots：R × S 个槽逐个按 check_root_slot 判（magic、整槽校验和、fsid、flags）
fn checker_collect_roots(g: CheckerSlot) {
    root_count = 0u;
    for (var region = 0u; region < g.regions; region = region + 1u) {
        let device = checker_region_device(g, region);
        for (var slot = 0u; slot < g.slots_per_region; slot = slot + 1u) {
            let offset = add64(slot_to_bytes(g.base_slot), u64_of(region * g.prime_step * g.chunk_bytes + slot * g.spacing));
            let s = read_at(device, offset, g.physical_block_size);
            let candidate = parse_root(s, g.fsid);
            if (!candidate.valid) {
                continue;
            }
            if (root_count >= MAX_ROOTS) {
                undecidable = UNDECIDABLE_TOO_MANY_ROOTS;
                return;
            }
            sws(R_ROOTS + root_count * 2u, s.version);
            sws(R_ROOTS + root_count * 2u + 1u, region | (slot << 8u));
            root_count = root_count + 1u;
        }
    }
}

// ───────────────────────── journal（checker 的读法，逐盘）─────────────────────────

// lib::check_journal_record：magic、整条校验和、类型 1、长度 4096、fsid、载荷校验和、每个点名项的 flags 字节为 0
fn checker_record_valid(s: Slice, fsid_low: vec2<u32>) -> bool {
    if (s.version == NONE || s.length < JOURNAL_RECORD_BYTES) {
        return false;
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSJ || s.c0 != 1u || rd16(base, 4u) != 1u || rd32(base, 8u) != JOURNAL_RECORD_BYTES) {
        return false;
    }
    if (!eq64(rd64(base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET), fsid_low)) {
        return false;
    }
    // 新根段末尾卸载记号那 4 字节位 0 之外有位为 1：读者与恢复当这条记录损坏，checker 同一口径不收（RR1）
    if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 360u) & 0xFFFFFFFEu) != 0u) {
        return false;
    }
    if (s.c1 != 1u) {
        return false;
    }
    let named = rd32(base, 12u);
    for (var i = 0u; i < named; i = i + 1u) {
        if (rd8(base, JOURNAL_HEADER_BYTES + i * JOURNAL_NAMED_ENTRY_BYTES + 55u) != 0u) {
            return false;
        }
    }
    return true;
}

// 一块盘上的记录按计数器排进草稿区：版本、计数器 lo/hi、实例、环槽 lo/hi。插入点用二分找（环按偏移收、计数器升序到达时就是表尾，不挪）
fn checker_insert_record(version: u32, counter: vec2<u32>, instance: u32, ring_slot: vec2<u32>) {
    let position = checker_record_position_not_below(counter);
    if (checker_record_counter_at_is(position, counter)) {
        return;
    }
    if (record_count >= MAX_RECORDS) {
        undecidable = UNDECIDABLE_TOO_MANY_RECORDS;
        return;
    }
    var i = record_count;
    while (i > position) {
        for (var k = 0u; k < RECORD_WORDS; k = k + 1u) {
            sws(R_RECORDS + i * RECORD_WORDS + k, sw(R_RECORDS + (i - 1u) * RECORD_WORDS + k));
        }
        i = i - 1u;
    }
    sws(R_RECORDS + position * RECORD_WORDS, version);
    sws(R_RECORDS + position * RECORD_WORDS + 1u, counter.x);
    sws(R_RECORDS + position * RECORD_WORDS + 2u, counter.y);
    sws(R_RECORDS + position * RECORD_WORDS + 3u, instance);
    sws(R_RECORDS + position * RECORD_WORDS + 4u, ring_slot.x);
    sws(R_RECORDS + position * RECORD_WORDS + 5u, ring_slot.y);
    record_count = record_count + 1u;
}

// 一块盘的环：候选记录槽逐个读，自证过的按计数器排进草稿区
fn checker_scan_journal_of_device(device: u32, fsid_low: vec2<u32>) {
    record_count = 0u;
    let location_count = hdr(6u);
    for (var l = 0u; l < location_count; l = l + 1u) {
        if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
            continue;
        }
        let s = read_location(l);
        if (!checker_record_valid(s, fsid_low)) {
            continue;
        }
        let offset = vec2<u32>(location_word(l, 1u), location_word(l, 2u));
        let ring_slot = shr64(sub64(offset, u64_of(JOURNAL_RING_START_BYTES)), 12u);
        checker_insert_record(s.version, rd48(s.base, 20u), rd32(s.base, 16u), ring_slot);
    }
}

// I-8.6：反向链 = 本实例内逻辑前一条头的链值；计数器 1 与前一条是别的实例 ⇒ 0；计数器 0 违例；前一条不在这块盘上不判
fn judge_journal_back_chain_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        var expected = 0u;
        if (is_zero64(r.counter)) {
            judge(INVARIANT_I_8_6, false);
            continue;
        }
        if (eq64(r.counter, u64_of(1u))) {
            expected = 0u;
        } else {
            let previous_index = checker_find_record_by_counter(sub64(r.counter, u64_of(1u)));
            if (previous_index == NONE) {
                continue;
            }
            let previous = checker_record_at(previous_index);
            if (previous.instance == r.instance) {
                expected = previous.chain_value;
            } else {
                expected = 0u;
            }
        }
        judge(INVARIANT_I_8_6, r.back_chain == expected);
    }
}

// I-8.7：同一实例相邻两条非 0 记录，号相等则中间不许有本实例别的记录，号不等则严格变大
fn judge_transaction_numbers_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        if (is_zero64(r.transaction)) {
            continue;
        }
        // 前一条本实例非 0 的记录
        var previous_index = NONE;
        var j = i;
        while (j > 0u) {
            j = j - 1u;
            let candidate = checker_record_at(j);
            if (candidate.instance == r.instance && !is_zero64(candidate.transaction)) {
                previous_index = j;
                break;
            }
        }
        if (previous_index == NONE) {
            continue;
        }
        let previous = checker_record_at(previous_index);
        if (eq64(r.transaction, previous.transaction)) {
            var separated = false;
            for (var k = previous_index + 1u; k < i; k = k + 1u) {
                if (checker_record_at(k).instance == r.instance) {
                    separated = true;
                    break;
                }
            }
            judge(INVARIANT_I_8_7, !separated);
            continue;
        }
        judge(INVARIANT_I_8_7, lt64(previous.transaction, r.transaction));
    }
}

// 逐盘 journal 那几条按组判（I-8.8 的组是同实例同事务号，I-8.9、I-8.1、I-8.10 的组是同实例同 checkpoint_txg，即同一次发布）：组首、组末、
// 组里各条原来各自从表头扫到 i、从 i 扫到表尾去找，条数平方长。改成每条记录一行组键（实例、键 lo/hi、0）、按 (组键, 表里的下标) 排一次序
// （order_rows_by_their_first_four_words），同组的在次序里连成一段、段里按表里的下标升序：段首就是组里第一条，段尾就是组末那一条
const JOURNAL_GROUP_BY_TRANSACTION: u32 = 0u;

const JOURNAL_GROUP_BY_PUBLISH: u32 = 1u;

fn checker_record_base_at(index: u32) -> u32 {
    return B[T_VERSION_BYTES] + tab(T_VERSIONS, sw(R_RECORDS + index * RECORD_WORDS) * VERSION_WORDS + 1u);
}

// 这块盘的记录表（checker_scan_journal_of_device 收的）按组排进 C_ROW_ORDER：组键写进 C_JOURNAL_GROUP_KEYS（事务号取记录偏移 78，checkpoint_txg 取 26）
fn order_the_records_of_device_by_group(group_kind: u32) {
    let key_offset = select(26u, 78u, group_kind == JOURNAL_GROUP_BY_TRANSACTION);
    for (var i = 0u; i < record_count; i = i + 1u) {
        let key = rd64(checker_record_base_at(i), key_offset);
        let row = C_JOURNAL_GROUP_KEYS + i * C_JOURNAL_GROUP_KEY_WORDS;
        sws(row, sw(R_RECORDS + i * RECORD_WORDS + 3u));
        sws(row + 1u, key.x);
        sws(row + 2u, key.y);
        sws(row + 3u, 0u);
    }
    order_rows_by_their_first_four_words(C_JOURNAL_GROUP_KEYS, C_JOURNAL_GROUP_KEY_WORDS, record_count, C_ROW_ORDER);
}

// C_ROW_ORDER 里从 position 起那一组段的末端（段之后第一个位置）：组键前三个字与段首那条相同的连着走完
fn journal_group_end(position: u32) -> u32 {
    let first = C_JOURNAL_GROUP_KEYS + sw(C_ROW_ORDER + position) * C_JOURNAL_GROUP_KEY_WORDS;
    var end = position + 1u;
    while (end < record_count) {
        let member = C_JOURNAL_GROUP_KEYS + sw(C_ROW_ORDER + end) * C_JOURNAL_GROUP_KEY_WORDS;
        if (sw(member) != sw(first) || sw(member + 1u) != sw(first + 1u) || sw(member + 2u) != sw(first + 2u)) {
            break;
        }
        end = end + 1u;
    }
    return end;
}

// I-8.8：提交标记字节只许 0 / 1；一组（同实例同非 0 事务号）带标记的至多一条且在组末；一条都不带时本实例之后不许再有记录。
// 要 C_ROW_ORDER 里是按 (实例, 事务号) 排好的次序（order_the_records_of_device_by_group(JOURNAL_GROUP_BY_TRANSACTION)）。
// 「本实例之后还有记录」⟺ 本实例在表里最靠后的那一条的下标 > 组末的下标：同实例的各组在次序里连成一段，进这一段时走一遍取最大下标
fn judge_commit_markers_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        if (rd8(checker_record_base_at(i), 86u) > 1u) {
            violate(INVARIANT_I_8_8);
        }
    }
    var position = 0u;
    var instance_span_end = 0u;
    var latest_index_of_the_instance = 0u;
    while (position < record_count) {
        let first = sw(C_ROW_ORDER + position);
        let instance = sw(C_JOURNAL_GROUP_KEYS + first * C_JOURNAL_GROUP_KEY_WORDS);
        if (position >= instance_span_end) {
            instance_span_end = position;
            latest_index_of_the_instance = first;
            while (instance_span_end < record_count
                && sw(C_JOURNAL_GROUP_KEYS + sw(C_ROW_ORDER + instance_span_end) * C_JOURNAL_GROUP_KEY_WORDS) == instance) {
                latest_index_of_the_instance = max(latest_index_of_the_instance, sw(C_ROW_ORDER + instance_span_end));
                instance_span_end = instance_span_end + 1u;
            }
        }
        let group_end = journal_group_end(position);
        let group_start = position;
        position = group_end;
        let transaction = vec2<u32>(sw(C_JOURNAL_GROUP_KEYS + first * C_JOURNAL_GROUP_KEY_WORDS + 1u),
            sw(C_JOURNAL_GROUP_KEYS + first * C_JOURNAL_GROUP_KEY_WORDS + 2u));
        if (is_zero64(transaction)) {
            continue;
        }
        let last_index = sw(C_ROW_ORDER + group_end - 1u);
        var marked = 0u;
        var marked_at_last = false;
        for (var member_position = group_start; member_position < group_end; member_position = member_position + 1u) {
            let member = sw(C_ROW_ORDER + member_position);
            if (rd8(checker_record_base_at(member), 86u) == 1u) {
                marked = marked + 1u;
                marked_at_last = member == last_index;
            }
        }
        if (marked >= 2u || (marked == 1u && !marked_at_last)) {
            violate(INVARIANT_I_8_8);
        }
        // image::judge_commit_markers_per_transaction 的 judged_transactions：两条以上的组、一条提交标记都不带的组各算判过一组
        if (group_end - group_start >= 2u || marked == 0u) {
            judge(INVARIANT_I_8_8, true);
        }
        if (marked == 0u && latest_index_of_the_instance > last_index) {
            violate(INVARIANT_I_8_8);
        }
    }
}

// I-8.9：记录标志其余位、序号 0、组内跳号、不从 1 起、多于一条末条、末条之后还有记录、没有末条。
// 要 C_ROW_ORDER 里是按 (实例, checkpoint_txg) 排好的次序（order_the_records_of_device_by_group(JOURNAL_GROUP_BY_PUBLISH)）
fn judge_publish_ordinals_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let base = checker_record_base_at(i);
        // image::judge_publish_ordinals_and_last_record_flags：盘上有一条记录就算判过（judged_records）
        judge(INVARIANT_I_8_9, (rd8(base, 7u) & 0xFEu) == 0u);
        if (rd32(base, 87u) == 0u) {
            violate(INVARIANT_I_8_9);
        }
    }
    var position = 0u;
    while (position < record_count) {
        let group_start = position;
        let group_end = journal_group_end(position);
        position = group_end;
        let r = checker_record_at(sw(C_ROW_ORDER + group_start));
        let last_index = sw(C_ROW_ORDER + group_end - 1u);
        var flagged = 0u;
        var flagged_at_last = false;
        for (var member_position = group_start; member_position < group_end; member_position = member_position + 1u) {
            let member_index = sw(C_ROW_ORDER + member_position);
            let member = checker_record_at(member_index);
            if (member_position > group_start) {
                // 序号之差要等于计数器之差（序号是 u32，差可以为负：按 64 位有符号比）
                let counter_distance = sub64(member.counter, r.counter);
                if (member.ordinal >= r.ordinal) {
                    if (!eq64(u64_of(member.ordinal - r.ordinal), counter_distance)) {
                        violate(INVARIANT_I_8_9);
                    }
                } else {
                    violate(INVARIANT_I_8_9);
                }
            }
            if ((member.flags & 1u) != 0u) {
                flagged = flagged + 1u;
                flagged_at_last = member_index == last_index;
            }
        }
        if (!is_zero64(r.counter)) {
            let before = checker_find_record_by_counter(sub64(r.counter, u64_of(1u)));
            if (before != NONE && r.ordinal != 1u) {
                violate(INVARIANT_I_8_9);
            }
        }
        if (flagged == 0u) {
            let last = checker_record_at(last_index);
            let next = checker_find_record_by_counter(add64(last.counter, u64_of(1u)));
            if (next != NONE && checker_record_at(next).instance == r.instance) {
                violate(INVARIANT_I_8_9);
            }
        } else if (flagged == 1u) {
            if (!flagged_at_last) {
                violate(INVARIANT_I_8_9);
            }
        } else {
            violate(INVARIANT_I_8_9);
        }
    }
}

// I-2.5 的 journal 点名项那一半：每个点名项的两条位置条目设备身份严格升序
fn judge_named_entry_location_order_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        for (var n = 0u; n < r.named_count; n = n + 1u) {
            let o = JOURNAL_HEADER_BYTES + n * JOURNAL_NAMED_ENTRY_BYTES;
            judge(INVARIANT_I_2_5, rd32(r.base, o) < rd32(r.base, o + 14u));
        }
    }
}

// I-8.12：同一实例的记录按计数器从小到大 checkpoint_txg 不减——每条只与同实例的前一条比，中间夹着别的实例的不管
fn judge_journal_record_txg_per_instance_of_device() {
    for (var i = 1u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        var j = i;
        while (j > 0u) {
            j = j - 1u;
            let previous = checker_record_at(j);
            if (previous.instance == r.instance) {
                judge(INVARIANT_I_8_12, !lt64(r.txg, previous.txg));
                break;
            }
        }
    }
}

// I-8.1 读环的那一格：同一实例、同一 checkpoint_txg 的记录（同一次发布）不多于系统配置记的在飞上限条，每组数一次。
// 要 C_ROW_ORDER 里是按 (实例, checkpoint_txg) 排好的次序：组里几条就是那一段多长
fn judge_records_of_one_publish_within_the_in_flight_record_limit_of_device(declared_in_flight_record_limit: u32) {
    var position = 0u;
    while (position < record_count) {
        let group_end = journal_group_end(position);
        judge(INVARIANT_I_8_1, group_end - position <= declared_in_flight_record_limit);
        position = group_end;
    }
}

// I-8.11：每条记录每个点名项的每条位置条目，按类读那一份（码 2 一个节点、码 1 / 码 3 一个数据单元宽，类标签不在登记表里的不读），
// 整单元校验和相符时，单元头的类标签、诞生代号、写序实例代号与尾段（码 1 事务号 6 字节；码 2 / 码 3 出生序号 4 + 补零 2）、
// 码 1 / 码 3 的出生树（头偏移 43）与点名项逐字段相同；校验和不相符的不判（walk.rs judge_named_entries_against_the_unit_headers）
fn judge_named_entries_against_the_unit_headers_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        for (var n = 0u; n < r.named_count; n = n + 1u) {
            let o = JOURNAL_HEADER_BYTES + n * JOURNAL_NAMED_ENTRY_BYTES;
            let named_class = rd8(r.base, o + 28u);
            var unit_bytes = 0u;
            if (named_class == 1u || named_class == 3u) {
                unit_bytes = DATA_UNIT_BYTES;
            } else if (named_class == 2u) {
                unit_bytes = NODE_BYTES;
            } else {
                continue;
            }
            let named_birth_tree = rd64(r.base, o + 29u);
            let named_birth_txg = rd64(r.base, o + 37u);
            let named_instance = rd32(r.base, o + 45u);
            let named_tail = rd48(r.base, o + 49u);
            for (var copy = 0u; copy < 2u; copy = copy + 1u) {
                let location = o + copy * 14u;
                let s = read_at(rd32(r.base, location), slot_to_bytes(rd48(r.base, location + 4u)), unit_bytes);
                if (s.version == NONE || !slice_crc_matches(s, unit_bytes, rd32(r.base, location + 10u))) {
                    continue;
                }
                let header_class = rd8(s.base, 6u);
                var identity_matches = false;
                var birth_tree_matches = true;
                if (header_class == 1u) {
                    identity_matches = eq64(rd64(s.base, 75u), named_birth_txg) && rd32(s.base, 91u) == named_instance && eq64(rd48(s.base, 95u), named_tail);
                    birth_tree_matches = eq64(rd64(s.base, 43u), named_birth_tree);
                } else if (header_class == 2u) {
                    let key_span = 2u * rd8(s.base, 51u);
                    identity_matches = eq64(rd64(s.base, 52u + key_span), named_birth_txg) && rd32(s.base, 68u + key_span) == named_instance
                        && eq64(vec2<u32>(rd32(s.base, 72u + key_span), 0u), named_tail);
                } else if (header_class == 3u) {
                    identity_matches = eq64(rd64(s.base, 73u), named_birth_txg) && rd32(s.base, 93u) == named_instance
                        && eq64(vec2<u32>(rd32(s.base, 103u), 0u), named_tail);
                    birth_tree_matches = eq64(rd64(s.base, 43u), named_birth_tree);
                }
                judge(INVARIANT_I_8_11, header_class == named_class && identity_matches && birth_tree_matches);
            }
        }
    }
}

// I-8.10：同 (实例代号, checkpoint_txg) 带末条标志的记录恰一条时，它的新根段七段（记录偏移 99 起：树表指针 86、映射根指针 86、
// 树 ID 水位 8、回退下界 F 8、实例表指针 86、分配记录树根指针 86、卸载记号 4）与根环里同 (实例代号, checkpoint_txg) 的每条自证过的根的
// 同名七段（根记录 36、256、122、130、170、342，卸载记号对根记录 flags 20 那 4 字节）逐字节相同（用户 2026-10-03 定从四段扩到七段）。
// 要 C_ROW_ORDER 里是按 (实例, checkpoint_txg) 排好的次序：一组里带末条标志的数一遍，恰一条才比
fn judge_new_root_segment_of_the_last_record_against_the_roots_of_device() {
    var position = 0u;
    while (position < record_count) {
        let group_end = journal_group_end(position);
        var last_records_of_this_publish = 0u;
        var the_last_record = NONE;
        for (var member_position = position; member_position < group_end; member_position = member_position + 1u) {
            let member = sw(C_ROW_ORDER + member_position);
            if ((rd8(checker_record_base_at(member), 7u) & 1u) != 0u) {
                last_records_of_this_publish = last_records_of_this_publish + 1u;
                the_last_record = member;
            }
        }
        position = group_end;
        if (last_records_of_this_publish != 1u) {
            continue;
        }
        let r = checker_record_at(the_last_record);
        for (var k = 0u; k < root_count; k = k + 1u) {
            let root = checker_root_of(k);
            if (root.instance != r.instance || !eq64(root.txg, r.txg)) {
                continue;
            }
            let segment = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
            let same = bytes_equal(r.base, segment, root.base, 36u, 86u)
                && bytes_equal(r.base, segment + 86u, root.base, 256u, 86u)
                && bytes_equal(r.base, segment + 172u, root.base, 122u, 8u)
                && bytes_equal(r.base, segment + 180u, root.base, 130u, 8u)
                && bytes_equal(r.base, segment + 188u, root.base, 170u, 86u)
                && bytes_equal(r.base, segment + 274u, root.base, 342u, 86u)
                && bytes_equal(r.base, segment + 360u, root.base, 20u, 4u);
            judge(INVARIANT_I_8_10, same);
        }
    }
}

// ───────────────────────── I-7.7：盘上带实例代号的东西 ─────────────────────────

// walk::unit_write_order_instance：头校验和过、fsid 与本池相同的单元头里写序的实例代号；不是就 NONE
fn unit_write_order_instance(s: Slice, fsid_low: vec2<u32>) -> u32 {
    if (s.version == NONE || s.length < 52u || rd32(s.base, 0u) != MAGIC_SFSU) {
        return NONE;
    }
    let unit_class = rd8(s.base, 6u);
    var fsid_offset = 0u;
    var instance_offset = 0u;
    if (unit_class == 1u) {
        fsid_offset = 83u;
        instance_offset = 91u;
    } else if (unit_class == 2u) {
        let key_span = 2u * rd8(s.base, 51u);
        fsid_offset = 60u + key_span;
        instance_offset = 68u + key_span;
    } else if (unit_class == 3u) {
        fsid_offset = 81u;
        instance_offset = 93u;
    } else {
        return NONE;
    }
    if ((s.c2 & 1u) == 0u) {
        return NONE;
    }
    if (!eq64(rd64(s.base, fsid_offset), fsid_low)) {
        return NONE;
    }
    return rd32(s.base, instance_offset);
}

// walk::slots_claimed_by_a_unit_header（D18（块里携带什么信息） 已定项 7 认领规则）：起点槽上解得出单元头——magic 对、类标签 1 / 2 / 3、
// 头校验和按自述的类过（检查字 c2 位 0）——就认领它：码 1、码 3 两槽，码 2 一槽；解不出 ⇒ 0，这一槽不认领。不看 fsid
fn slots_claimed_by_a_unit_header(s: Slice) -> u32 {
    if (s.version == NONE || s.length < NODE_BYTES || rd32(s.base, 0u) != MAGIC_SFSU || (s.c2 & 1u) == 0u) {
        return 0u;
    }
    let unit_class = rd8(s.base, 6u);
    if (unit_class == 1u || unit_class == 3u) {
        return DATA_UNIT_BYTES >> SLOT_SHIFT;
    }
    if (unit_class == 2u) {
        return NODE_BYTES >> SLOT_SHIFT;
    }
    return 0u;
}

// 是不是有载体的实例代号高于 threshold（根、每块盘的 journal 记录、每块盘的单元头写序）
fn some_instance_carrier_exceeds(threshold: u32, g: CheckerSlot) -> bool {
    for (var i = 0u; i < root_count; i = i + 1u) {
        if (checker_root_of(i).instance > threshold) {
            return true;
        }
    }
    let fsid_low = vec2<u32>(g.fsid.x, g.fsid.y);
    let location_count = hdr(6u);
    for (var l = 0u; l < location_count; l = l + 1u) {
        let kind = location_word(l, 4u);
        if (kind == LOCATION_JOURNAL_RECORD) {
            let s = read_location(l);
            if (checker_record_valid(s, fsid_low) && rd32(s.base, 16u) > threshold) {
                return true;
            }
        } else if (kind == LOCATION_UNIT_SLOT) {
            let instance = unit_write_order_instance(read_location(l), fsid_low);
            if (instance != NONE && instance > threshold) {
                return true;
            }
        }
    }
    return false;
}

// walk.rs 走读里 `self.judgements.judge(…)` 的对应：只在判的那一趟（w_judging）记评估过与违例
fn w_judge(invariant: u32, holds: bool) {
    if (w_judging) {
        judge(invariant, holds);
    }
}

// walk.rs 走读里 `self.judgements.part_of_the_range_not_judged(…)` 的对应：同只在判的那一趟记
fn w_mark_part_of_the_range_not_judged(invariant: u32) {
    if (w_judging) {
        mark_part_of_the_range_not_judged(invariant);
    }
}

// 正在走的那一版（walk.rs 的 version_being_walked）：它的 checkpoint_txg（I-9.25 与 I-3.13 按它判），
// 与它是不是回退候选集里不是最新的那几条根之一（I-4.9 只在这一类上判）。walk_and_hand_off 在唯一那个调用点之前设。
var<private> w_version_txg: vec2<u32>;

var<private> w_version_is_an_older_candidate_root: bool;

// I-4.9：正在走的是不是最新的候选根时，它引用的一个单元的诞生代号不大于这条根的 txg
fn w_judge_unit_birth_against_an_older_candidate_root(birth_txg: vec2<u32>) {
    if (w_version_is_an_older_candidate_root) {
        w_judge(INVARIANT_I_4_9, le64(birth_txg, w_version_txg));
    }
}

// ───────────────────────── I-3.5（引用区间的精确性）的出现规则：走读时记下的东西 ─────────────────────────
// 四张表在 crash_judge_checker_common.wgsl 的 C_APPEARANCE_*，只记主走读（上下文 0）走的那几版；这几个数是各表记到第几行，w_start(0) 清零。
var<private> appearance_placement_count: u32;

var<private> appearance_version_count: u32;

var<private> appearance_version_placement_count: u32;

var<private> appearance_walked_tree_table_count: u32;

// 正在走的这一版树表里用户可见树的根记了几棵：记在版本表第 appearance_version_count 行（这一版走完才算进那一行）
var<private> appearance_root_count_of_this_version: u32;

// 走读此刻在哪棵用户可见树里往下走（walk.rs 的 user_visible_tree_being_walked）：在不在、种类、树 ID
var<private> appearance_user_visible_tree_is_being_walked: bool;

var<private> appearance_user_visible_tree_kind: u32;

var<private> appearance_user_visible_tree: vec2<u32>;

// 用户可见树的种类（walk.rs 的 USER_VISIBLE_TREE_KINDS：extent、inode、deleted_inodes）
fn appearance_tree_kind_is_user_visible(kind: u32) -> bool {
    return kind == TREE_KIND_EXTENT || kind == TREE_KIND_INODE || kind == TREE_KIND_DELETED_INODES;
}

// 落点表里 (盘, 槽) 那一行的下标，不在就是 NONE（按桶表 W_PLACEMENT_BUCKETS 找）
fn appearance_placement_index(device: u32, slot: vec2<u32>) -> u32 {
    return hashed_row_of(W_PLACEMENT_BUCKETS, W_PLACEMENT_BUCKET_COUNT, C_APPEARANCE_PLACEMENTS, C_APPEARANCE_PLACEMENT_WORDS, device, slot, 0u, false).row;
}

// walk.rs note_references_of_a_pointer_followed_by_the_walk 里记 I-3.5 那两张表的两句：落点第一次记下时记它是哪个单元
// （unit_of_each_placement.entry(..).or_insert），走读此刻在一棵用户可见树里、这个落点还没打标签时打上那棵树（or_insert）
fn appearance_note_placement(device: u32, slot: vec2<u32>, unit_device: u32, unit_slot: vec2<u32>) {
    let found = hashed_row_of(W_PLACEMENT_BUCKETS, W_PLACEMENT_BUCKET_COUNT, C_APPEARANCE_PLACEMENTS, C_APPEARANCE_PLACEMENT_WORDS, device, slot, 0u, false);
    var index = found.row;
    if (index == NONE) {
        if (appearance_placement_count >= C_APPEARANCE_MAX_PLACEMENTS) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (40u << 8u);
            return;
        }
        index = appearance_placement_count;
        let new_row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
        sws(new_row, device);
        sws(new_row + 1u, slot.x);
        sws(new_row + 2u, slot.y);
        sws(new_row + 3u, unit_device);
        sws(new_row + 4u, unit_slot.x);
        sws(new_row + 5u, unit_slot.y);
        sws(new_row + 6u, 0u);
        sws(new_row + 7u, 0u);
        sws(new_row + 8u, 0u);
        sws(new_row + 9u, 0u);
        if (found.empty_bucket != NONE) {
            sws(W_PLACEMENT_BUCKETS + found.empty_bucket, index + 1u);
        }
        appearance_placement_count = appearance_placement_count + 1u;
    }
    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
    if (appearance_user_visible_tree_is_being_walked && sw(row + 6u) == 0u) {
        sws(row + 6u, 1u);
        sws(row + 7u, appearance_user_visible_tree_kind);
        sws(row + 8u, appearance_user_visible_tree.x);
        sws(row + 9u, appearance_user_visible_tree.y);
    }
}

// 实例表一片的诞生代号（walk.rs 的 birth_txg_of_walked_instance_table_pages）：记进走过的单元那一行，标记写 2——
// I-4.9 只认标记 1（实例表单元按 I-1.2 那一行的例外不进 I-4.9），I-3.5 两种都认
fn appearance_note_the_birth_txg_of_an_instance_table_page(device: u32, slot: vec2<u32>, birth_txg: vec2<u32>) {
    let visited_index = w_visited_index_in_the_main_walk(device, slot);
    if (visited_index == NONE || sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS) != 0u) {
        return;
    }
    sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS, 2u);
    sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u, birth_txg.x);
    sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u, birth_txg.y);
}

// 正在走的这一版树表里一棵用户可见树的根记进版本表的待定那一行（版本表满了不记：这一版走完记它时判不了）
fn appearance_note_a_user_visible_root_of_this_version(kind: u32, tree: vec2<u32>, root_pointer_birth_txg: vec2<u32>) {
    if (appearance_version_count >= C_APPEARANCE_MAX_VERSIONS) {
        return;
    }
    if (appearance_root_count_of_this_version >= C_APPEARANCE_MAX_USER_VISIBLE_ROOTS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (41u << 8u);
        return;
    }
    let row = C_APPEARANCE_VERSIONS + appearance_version_count * C_APPEARANCE_VERSION_WORDS
        + C_APPEARANCE_VERSION_ROOTS_OFFSET + appearance_root_count_of_this_version * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
    sws(row, kind);
    sws(row + 1u, tree.x);
    sws(row + 2u, tree.y);
    sws(row + 3u, root_pointer_birth_txg.x);
    sws(row + 4u, root_pointer_birth_txg.y);
    appearance_root_count_of_this_version = appearance_root_count_of_this_version + 1u;
}

// 这一版第一个读得出的树表（walk_the_entries_of_a_tree_table_walked_first_by_this_version）：树表里每棵根指针非全零的用户可见树
// （条目窄于 200 字节的不算，tree_table_entry_of_the_walk 交 None）记成这一版的根，连同树表落点记进走过的树表那张表
fn appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt: Node, tree_table_device: u32, tree_table_slot: vec2<u32>) {
    if (appearance_walked_tree_table_count >= C_APPEARANCE_MAX_WALKED_TREE_TABLES) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (42u << 8u);
        return;
    }
    let table_row = C_APPEARANCE_WALKED_TREE_TABLES + appearance_walked_tree_table_count * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
    sws(table_row, tree_table_device);
    sws(table_row + 1u, tree_table_slot.x);
    sws(table_row + 2u, tree_table_slot.y);
    var roots = 0u;
    if (tt.entry_width >= 200u) {
        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
            let o = entry_offset(tt, e);
            let kind = rd16(tt.base, o + 10u);
            let root_pointer = parse_node_pointer(tt.base, o + 14u);
            if (!appearance_tree_kind_is_user_visible(kind) || root_pointer.all_zero) {
                continue;
            }
            if (roots >= C_APPEARANCE_MAX_USER_VISIBLE_ROOTS) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (46u << 8u);
                return;
            }
            let root_row = table_row + C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET + roots * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
            let tree = rd64(tt.base, o);
            sws(root_row, kind);
            sws(root_row + 1u, tree.x);
            sws(root_row + 2u, tree.y);
            sws(root_row + 3u, root_pointer.birth_txg.x);
            sws(root_row + 4u, root_pointer.birth_txg.y);
            roots = roots + 1u;
            appearance_note_a_user_visible_root_of_this_version(kind, tree, root_pointer.birth_txg);
        }
    }
    sws(table_row + 3u, roots);
    appearance_walked_tree_table_count = appearance_walked_tree_table_count + 1u;
}

// 这一版的树表别的版本先走过（judge_the_tree_table_entries_an_earlier_version_walked）：这一版的用户可见树的根照先走那一版记下的取；
// 先走那一版头用不了、没记下的，这里没有对象
fn appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_device: u32, tree_table_slot: vec2<u32>) {
    for (var t = 0u; t < appearance_walked_tree_table_count; t = t + 1u) {
        let table_row = C_APPEARANCE_WALKED_TREE_TABLES + t * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
        if (sw(table_row) != tree_table_device || sw(table_row + 1u) != tree_table_slot.x || sw(table_row + 2u) != tree_table_slot.y) {
            continue;
        }
        let roots = sw(table_row + 3u);
        for (var r = 0u; r < roots; r = r + 1u) {
            let root_row = table_row + C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET + r * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
            appearance_note_a_user_visible_root_of_this_version(sw(root_row), vec2<u32>(sw(root_row + 1u), sw(root_row + 2u)), vec2<u32>(sw(root_row + 3u), sw(root_row + 4u)));
        }
        return;
    }
}

// 走树表的第 e 条之前（walk_the_entries_of_a_tree_table_walked_first_by_this_version 的循环）：条目是一棵根指针非全零的用户可见树时，
// 之后跟着指针记下的落点打上那棵树；不是就不打
fn appearance_begin_tree_table_entry(tt: Node, e: u32) {
    appearance_user_visible_tree_is_being_walked = false;
    if (tt.entry_width < 200u) {
        return;
    }
    let o = entry_offset(tt, e);
    let kind = rd16(tt.base, o + 10u);
    if (!appearance_tree_kind_is_user_visible(kind) || parse_node_pointer(tt.base, o + 14u).all_zero) {
        return;
    }
    appearance_user_visible_tree_is_being_walked = true;
    appearance_user_visible_tree_kind = kind;
    appearance_user_visible_tree = rd64(tt.base, o);
}

// note_the_version_just_walked_for_the_appearance_rule：一版走完，把它此刻引用过的落点（这一版的集合，w_judge_placement_in_version 记的）
// 换成落点表的下标记进版本表，连同实例、txg、走读这一版有没有记失败
fn appearance_note_the_version_just_walked(instance: u32, txg: vec2<u32>, references_are_complete: bool) {
    if (appearance_version_count >= C_APPEARANCE_MAX_VERSIONS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (43u << 8u);
        return;
    }
    let row = C_APPEARANCE_VERSIONS + appearance_version_count * C_APPEARANCE_VERSION_WORDS;
    sws(row, instance);
    sws(row + 1u, txg.x);
    sws(row + 2u, txg.y);
    sws(row + 3u, select(0u, 1u, references_are_complete));
    sws(row + 4u, appearance_version_placement_count);
    let count = w_in_version_count[0];
    for (var i = 0u; i < count; i = i + 1u) {
        let placement = appearance_placement_index(sw(C_IN_VERSION0 + i * 3u), vec2<u32>(sw(C_IN_VERSION0 + i * 3u + 1u), sw(C_IN_VERSION0 + i * 3u + 2u)));
        // 这一版引用过的每个落点都经 w_note_pointer_references 记过（这一版或先走的那一版）：不在说明落点表与这一版的集合脱节了
        if (placement == NONE || appearance_version_placement_count >= C_APPEARANCE_MAX_VERSION_PLACEMENTS) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (44u << 8u);
            return;
        }
        sws(C_APPEARANCE_VERSION_PLACEMENTS + appearance_version_placement_count, placement);
        appearance_version_placement_count = appearance_version_placement_count + 1u;
    }
    sws(row + 5u, count);
    sws(row + 6u, appearance_root_count_of_this_version);
    appearance_version_count = appearance_version_count + 1u;
}

// 主走读里 (盘, 槽) 在走过的单元表里的下标，不在就是 NONE（按桶表 W_VISITED_BUCKETS 找）
fn w_visited_index_in_the_main_walk(device: u32, slot: vec2<u32>) -> u32 {
    return hashed_row_of(W_VISITED_BUCKETS, W_VISITED_BUCKET_COUNT, C_VISITED0, 3u, device, slot, 0u, false).row;
}

fn c_references_base() -> u32 {
    if (w_context == 0u) { return C_REFERENCES0; }
    return C_REFERENCES1;
}

fn c_visited_base() -> u32 {
    if (w_context == 0u) { return C_VISITED0; }
    return C_VISITED1;
}

fn c_in_version_base() -> u32 {
    if (w_context == 0u) { return C_IN_VERSION0; }
    return C_IN_VERSION1;
}

fn c_below_base() -> u32 {
    if (w_context == 0u) { return C_BELOW0; }
    return C_BELOW1;
}

// 开一份新的 Walk（上下文 c）：全部集合清空；判定只在主走读那一份进违例位
fn w_start(context: u32, judging: bool) {
    w_context = context;
    w_judging = judging;
    w_reference_count[context] = 0u;
    w_visited_count[context] = 0u;
    w_in_version_count[context] = 0u;
    w_below_count[context] = 0u;
    w_failures = 0u;
    w_mismatches = 0u;
    w_parent_valid = false;
    if (context == 0u) {
        appearance_placement_count = 0u;
        appearance_version_count = 0u;
        appearance_version_placement_count = 0u;
        appearance_walked_tree_table_count = 0u;
        appearance_root_count_of_this_version = 0u;
        w_leaf_count = 0u;
        w_checksum_count = 0u;
        w_tree_id_count = 0u;
        w_inode_birth_count = 0u;
        w_data_object_count = 0u;
        w_accounting_row_count = 0u;
        w_accounting_seen = false;
        w_instance_row_count = 0u;
        w_inode_tree_walked = false;
        w_have_largest_inode = false;
        w_largest_inode = vec2<u32>(0u, 0u);
        for (var bucket = W_REFERENCE_BUCKETS; bucket < W_BUCKETS_END; bucket = bucket + 1u) {
            sws(bucket, 0u);
        }
    }
}

fn w_begin_version() {
    w_in_version_count[w_context] = 0u;
    w_parent_valid = false;
    appearance_user_visible_tree_is_being_walked = false;
    if (w_context == 0u) {
        appearance_root_count_of_this_version = 0u;
    }
}

// 引用集合：(盘, 起点槽, 跨度) 去重记进；位置项校验和记进主走读的表或收割表。两张表各一轮、同一个找的调用点（这个函数在走读内核里展开二十来份）：
// 第 0 轮引用集合（第四个字比跨度），第 1 轮位置项校验和（第四个字比校验和）。主走读那两张按桶表找（W_REFERENCE_BUCKETS、W_CHECKSUM_BUCKETS），
// 另起的那几遍（上下文 1）的引用集合与收割表照旧从头比：那几遍各走一条根、表小。引用集合满了记站点 1、照旧接着记校验和；校验和那张满了记站点 2
fn w_note_reference(device: u32, slot: vec2<u32>, checksum: u32, span: u32) {
    for (var table = 0u; table < 2u; table = table + 1u) {
        var rows = c_references_base();
        var stride = C_REFERENCE_WORDS;
        var count = w_reference_count[w_context];
        var capacity = C_MAX_REFERENCES;
        var fourth_word = span;
        var buckets = W_REFERENCE_BUCKETS;
        var bucket_count = W_REFERENCE_BUCKET_COUNT;
        if (table == 1u) {
            if (w_context != 0u && !w_harvesting) {
                return;
            }
            rows = select(C_HARVEST, C_CHECKSUMS, w_context == 0u);
            stride = C_CHECKSUM_WORDS;
            count = select(w_harvest_count, w_checksum_count, w_context == 0u);
            capacity = C_MAX_CHECKSUMS;
            fourth_word = checksum;
            buckets = W_CHECKSUM_BUCKETS;
            bucket_count = W_CHECKSUM_BUCKET_COUNT;
        }
        var present = false;
        var empty_bucket = NONE;
        if (w_context == 0u) {
            let found = hashed_row_of(buckets, bucket_count, rows, stride, device, slot, fourth_word, true);
            present = found.row != NONE;
            empty_bucket = found.empty_bucket;
        } else {
            for (var i = 0u; i < count; i = i + 1u) {
                if (sw(rows + i * stride) == device && sw(rows + i * stride + 1u) == slot.x
                    && sw(rows + i * stride + 2u) == slot.y && sw(rows + i * stride + 3u) == fourth_word) {
                    present = true;
                    break;
                }
            }
        }
        if (present) {
            continue;
        }
        if (count >= capacity) {
            if (table == 0u) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (1u << 8u);
                continue;
            }
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (2u << 8u);
            return;
        }
        sws(rows + count * stride, device);
        sws(rows + count * stride + 1u, slot.x);
        sws(rows + count * stride + 2u, slot.y);
        sws(rows + count * stride + 3u, fourth_word);
        if (empty_bucket != NONE) {
            sws(buckets + empty_bucket, count + 1u);
        }
        if (table == 0u) {
            sws(rows + count * stride + 4u, 0u);
            w_reference_count[w_context] = count + 1u;
        } else if (w_context == 0u) {
            w_checksum_count = count + 1u;
        } else {
            w_harvest_count = count + 1u;
        }
    }
}

fn w_visited_contains(device: u32, slot: vec2<u32>) -> bool {
    let base = c_visited_base();
    for (var i = 0u; i < w_visited_count[w_context]; i = i + 1u) {
        if (sw(base + i * 3u) == device && sw(base + i * 3u + 1u) == slot.x && sw(base + i * 3u + 2u) == slot.y) {
            return true;
        }
    }
    return false;
}

// visited_units.insert：已有交回 false。主走读那一张按桶表 W_VISITED_BUCKETS 找，另起的那几遍照旧从头比
fn w_visit(device: u32, slot: vec2<u32>) -> bool {
    var visited_bucket = NONE;
    if (w_context == 0u) {
        let found = hashed_row_of(W_VISITED_BUCKETS, W_VISITED_BUCKET_COUNT, C_VISITED0, 3u, device, slot, 0u, false);
        if (found.row != NONE) {
            return false;
        }
        visited_bucket = found.empty_bucket;
    } else if (w_visited_contains(device, slot)) {
        return false;
    }
    let count = w_visited_count[w_context];
    if (count >= C_MAX_VISITED) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (3u << 8u);
        return false;
    }
    let base = c_visited_base();
    sws(base + count * 3u, device);
    sws(base + count * 3u + 1u, slot.x);
    sws(base + count * 3u + 2u, slot.y);
    if (w_context == 0u) {
        sws(C_VISITED_BIRTHS0 + count * C_VISITED_BIRTH_WORDS, 0u);
    }
    if (visited_bucket != NONE) {
        sws(W_VISITED_BUCKETS + visited_bucket, count + 1u);
    }
    w_visited_count[w_context] = count + 1u;
    return true;
}

// I-5.1「同一版第二次引用同一个落点」：这一版引用过就红，没有就记下
fn w_judge_placement_in_version(device: u32, slot: vec2<u32>) {
    let base = c_in_version_base();
    let count = w_in_version_count[w_context];
    for (var i = 0u; i < count; i = i + 1u) {
        if (sw(base + i * 3u) == device && sw(base + i * 3u + 1u) == slot.x && sw(base + i * 3u + 2u) == slot.y) {
            w_judge(INVARIANT_I_5_1, false);
            return;
        }
    }
    if (count >= C_MAX_IN_VERSION) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (4u << 8u);
        return;
    }
    sws(base + count * 3u, device);
    sws(base + count * 3u + 1u, slot.x);
    sws(base + count * 3u + 2u, slot.y);
    w_in_version_count[w_context] = count + 1u;
}

// 一个别的版本已经走过的单元又被这一版引用：把它下面的落点（孩子、孩子的孩子……）逐个并进这一版的集合
fn w_merge_below(device: u32, slot: vec2<u32>) {
    var pending_count = 1u;
    sws(C_PENDING, device);
    sws(C_PENDING + 1u, slot.x);
    sws(C_PENDING + 2u, slot.y);
    var expanded_count = 0u;
    let below = c_below_base();
    let below_count = w_below_count[w_context];
    while (pending_count > 0u) {
        pending_count = pending_count - 1u;
        let unit_device = sw(C_PENDING + pending_count * 3u);
        let unit_slot = vec2<u32>(sw(C_PENDING + pending_count * 3u + 1u), sw(C_PENDING + pending_count * 3u + 2u));
        var already = false;
        for (var i = 0u; i < expanded_count; i = i + 1u) {
            if (sw(C_EXPANDED + i * 3u) == unit_device && sw(C_EXPANDED + i * 3u + 1u) == unit_slot.x && sw(C_EXPANDED + i * 3u + 2u) == unit_slot.y) {
                already = true;
                break;
            }
        }
        if (already) {
            continue;
        }
        if (expanded_count >= C_MAX_PENDING) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (5u << 8u);
            return;
        }
        sws(C_EXPANDED + expanded_count * 3u, unit_device);
        sws(C_EXPANDED + expanded_count * 3u + 1u, unit_slot.x);
        sws(C_EXPANDED + expanded_count * 3u + 2u, unit_slot.y);
        expanded_count = expanded_count + 1u;
        // I-4.9：别的版本先走过的单元按记下的诞生代号判（merge_placements_below_a_unit_walked_by_an_earlier_version）
        if (w_version_is_an_older_candidate_root && w_context == 0u) {
            let visited_index = w_visited_index_in_the_main_walk(unit_device, unit_slot);
            if (visited_index != NONE && sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS) == 1u) {
                w_judge_unit_birth_against_an_older_candidate_root(vec2<u32>(
                    sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u),
                    sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u)));
            }
        }
        for (var i = 0u; i < below_count; i = i + 1u) {
            if (sw(below + i * C_BELOW_WORDS) != unit_device || sw(below + i * C_BELOW_WORDS + 1u) != unit_slot.x || sw(below + i * C_BELOW_WORDS + 2u) != unit_slot.y) {
                continue;
            }
            let child_device = sw(below + i * C_BELOW_WORDS + 3u);
            let child_slot = vec2<u32>(sw(below + i * C_BELOW_WORDS + 4u), sw(below + i * C_BELOW_WORDS + 5u));
            w_judge_placement_in_version(child_device, child_slot);
            if (pending_count >= C_MAX_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (6u << 8u);
                return;
            }
            sws(C_PENDING + pending_count * 3u, child_device);
            sws(C_PENDING + pending_count * 3u + 1u, child_slot.x);
            sws(C_PENDING + pending_count * 3u + 2u, child_slot.y);
            pending_count = pending_count + 1u;
        }
    }
}

// 走读跟着一条指针往下：两条位置条目记进引用集合，判同一版第二次引用，记到父单元下面。
// 八步照原来的次序走：两条位置条目的落点（I-3.5，主走读）、第 0 条的引用、同一版第二次引用、父单元，第 1 条的同样三样（与第 0 条同落点时后两样不做）。
// 次序不改（几张表同时满时 undecidable 留最后写的那个站点）；每样只有一个调用点：这个函数在走读内核里展开二十来份，原来每份里每样两份拷贝
fn w_note_pointer_references(p: Ptr, span: u32) {
    let second_location_is_the_first = p.device1 == p.device0 && eq64(p.slot1, p.slot0);
    for (var step = 0u; step < 8u; step = step + 1u) {
        let on_the_second_location = step == 1u || step >= 5u;
        let device = select(p.device0, p.device1, on_the_second_location);
        let slot = select(p.slot0, p.slot1, on_the_second_location);
        if (step < 2u) {
            if (w_context == 0u) {
                appearance_note_placement(device, slot, p.device0, p.slot0);
            }
        } else if (step == 2u || step == 5u) {
            w_note_reference(device, slot, select(p.sum0, p.sum1, on_the_second_location), span);
        } else if (on_the_second_location && second_location_is_the_first) {
            continue;
        } else if (step == 3u || step == 6u) {
            w_judge_placement_in_version(device, slot);
        } else {
            w_record_below(device, slot);
        }
    }
}

fn w_record_below(device: u32, slot: vec2<u32>) {
    if (!w_parent_valid) {
        return;
    }
    let count = w_below_count[w_context];
    if (count >= C_MAX_BELOW) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (7u << 8u);
        return;
    }
    let base = c_below_base();
    sws(base + count * C_BELOW_WORDS, w_parent_device);
    sws(base + count * C_BELOW_WORDS + 1u, w_parent_slot.x);
    sws(base + count * C_BELOW_WORDS + 2u, w_parent_slot.y);
    sws(base + count * C_BELOW_WORDS + 3u, device);
    sws(base + count * C_BELOW_WORDS + 4u, slot.x);
    sws(base + count * C_BELOW_WORDS + 5u, slot.y);
    w_below_count[w_context] = count + 1u;
}

// w_read_referenced_unit 最近一次交回的是第二条位置条目那一份（第一条那一份对不上、第二条对得上）：「走过」按读回那一份的落点认的几处读它
// （w_visit_the_copy_read_back）
var<private> w_read_back_from_the_second_location: bool;

// w_read_unit_to_walk 最近一次读回、自证过的那一份的落点：这个单元在走读里的身份（walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from
// 交回的落点；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，攻方 allrbmap）。走过、诞生代号、并进前一版、
// 父单元、叶序与 I-3.5 的折叠表都按它认。两份都读不出时是第一条位置条目的落点（调用方不用它）
var<private> w_unit_device_read_back: u32;
var<private> w_unit_slot_read_back: vec2<u32>;

// image::read_referenced_unit：两条位置条目都读、都判 I-2.1，交回第一份对得上的
fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
    var first = no_slice();
    w_read_back_from_the_second_location = false;
    let s0 = read_at(p.device0, slot_to_bytes(p.slot0), unit_bytes);
    let matches0 = s0.version != NONE && slice_crc_matches(s0, unit_bytes, p.sum0);
    if (!matches0) {
        w_mismatches = w_mismatches + 1u;
    }
    w_judge(INVARIANT_I_2_1, matches0);
    if (matches0) {
        first = s0;
    }
    let s1 = read_at(p.device1, slot_to_bytes(p.slot1), unit_bytes);
    let matches1 = s1.version != NONE && slice_crc_matches(s1, unit_bytes, p.sum1);
    if (!matches1) {
        w_mismatches = w_mismatches + 1u;
    }
    w_judge(INVARIANT_I_2_1, matches1);
    if (matches1 && first.version == NONE) {
        first = s1;
        w_read_back_from_the_second_location = true;
    }
    return first;
}

// walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from：按位置条目读一个走读要往下走的单元（同 w_read_referenced_unit），读回、自证过的那一份的
// 落点记进 w_unit_device_read_back / w_unit_slot_read_back；读回的是第二条那一份、I-3.5 的落点表此刻把那个落点折在第一条落点那个单元上时，
// 改记成它自己（appearance_note_placement 记的是第一条位置条目的落点）。调用方先 w_note_pointer_references 再调它
fn w_read_unit_to_walk(p: Ptr, unit_bytes: u32) -> Slice {
    let s = w_read_referenced_unit(p, unit_bytes);
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
            sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS, 1u);
            sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u, birth_txg.x);
            sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u, birth_txg.y);
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
// 3 按位置读 extent 树上段的根（层级 0 的零条目根只判 key 宽，其余同 2；walk.rs 的 PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller）
fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_reading: u32, registered: bool, fsid_low: vec2<u32>) -> ReadNode {
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
    w_note_pointer_references(p, 1u);
    let s = w_read_unit_to_walk(p, NODE_BYTES);
    if (s.version == NONE) {
        w_failures = w_failures + 1u;
        return out;
    }
    // 「走过」、并进前一版、诞生代号按读回那一份的落点认，「先走过」交回调用方看（walk.rs IndexNodeReading）
    out.unit_device = w_unit_device_read_back;
    out.unit_slot = w_unit_slot_read_back;
    if (!w_visit(out.unit_device, out.unit_slot)) {
        out.already_walked = true;
        w_merge_below(out.unit_device, out.unit_slot);
        return out;
    }
    if (!w_judge_unit_header(s, 2u, fsid_low)) {
        return out;
    }
    w_judge_birth_identity(s, p, out.unit_device, out.unit_slot);
    let n = parse_index_node(s);
    if (!n.ok) {
        if (registered && entry_width_zero_with_entries(s)) {
            w_judge(INVARIANT_I_1_10, false);
        }
        w_failures = w_failures + 1u;
        return out;
    }
    w_judge(INVARIANT_I_1_3, eq64(n.tree, expected_tree));
    var keys_hold = false;
    if (key_range_reading == 0u || (key_range_reading == 1u && n.level == 0u)) {
        keys_hold = w_check_index_node_keys(n, schema);
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
            w_merge_below(device_read_from, slot_read_from);
            break;
        }
        w_parent_valid = true;
        w_parent_device = device_read_from;
        w_parent_slot = slot_read_from;
        let packed = w_judge_packed_container(s, 4u, fsid_low);
        if (!packed.ok) {
            break;
        }
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
        // 最后一条是合法的链指针记录那一格：有下一片而位置指针头部不是第一版的取值（第五形，第 10u 批）同样 chain == 2u（w_chain_record_kind）
        w_judge(INVARIANT_I_3_14, chain != 2u);
        // I-3.14 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补的那一句）：行不收、链不往下走、不记走读失败、没走到末片
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
                    for (var i = 0u; i < w_instance_row_count; i = i + 1u) {
                        if (sw(C_INSTANCE_ROWS + i * C_ROW_WORDS) == instance) {
                            rows_unique = false;
                        }
                    }
                    if (w_instance_row_count >= C_MAX_INSTANCE_ROWS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (8u << 8u);
                    } else {
                        sws(C_INSTANCE_ROWS + w_instance_row_count * C_ROW_WORDS, instance);
                        let row_txg = rd64(p.base, o + 5u);
                        let row_high_water = rd64(p.base, o + 13u);
                        sws(C_INSTANCE_ROWS + w_instance_row_count * C_ROW_WORDS + 1u, row_txg.x);
                        sws(C_INSTANCE_ROWS + w_instance_row_count * C_ROW_WORDS + 2u, row_txg.y);
                        sws(C_INSTANCE_ROWS + w_instance_row_count * C_ROW_WORDS + 3u, row_high_water.x);
                        sws(C_INSTANCE_ROWS + w_instance_row_count * C_ROW_WORDS + 4u, row_high_water.y);
                        w_instance_row_count = w_instance_row_count + 1u;
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
    if (judge_rows && pages > 0u) {
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
        // ③：每一种种类码（登记过的与没登记的都算）至多一条
        for (var later_entry = 1u; later_entry < n.entry_count; later_entry = later_entry + 1u) {
            let later_kind = rd16(n.base, entry_offset(n, later_entry) + 10u);
            for (var earlier_entry = 0u; earlier_entry < later_entry; earlier_entry = earlier_entry + 1u) {
                if (rd16(n.base, entry_offset(n, earlier_entry) + 10u) == later_kind) {
                    w_judge(INVARIANT_I_9_16, false);
                    return;
                }
            }
        }
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

// 树表条目的头 ID（walk.rs judge_tree_table_entry_head_identifiers，I-9.27）：inode 树等于自己的树 ID；extent 树、deleted_inodes、墓碑树、退役树
// 等于同一张表里某一条 inode 树条目的树 ID（一条都没有就红）；3、4、6、7、8、11、12 等于 0；别的种类不判。条目宽不到头 ID 那 8 字节时整张表不判
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
            || kind == TREE_KIND_RETIREMENT) {
            var names_an_inode_tree_of_this_table = false;
            for (var other_index = 0u; other_index < n.entry_count; other_index = other_index + 1u) {
                let other = entry_offset(n, other_index);
                if (rd16(n.base, other + 10u) == TREE_KIND_INODE && eq64(rd64(n.base, other), head)) {
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
    let p = parse_data_pointer(base, o);
    w_judge_location_order(p);
    w_judge_pointer_head(p);
    w_judge_pointer_extent_offset(p);
    w_note_pointer_references(p, 2u);
    let s = w_read_unit_to_walk(p, DATA_UNIT_BYTES);
    if (s.version == NONE) {
        w_failures = w_failures + 1u;
        return;
    }
    let unit_device = w_unit_device_read_back;
    let unit_slot = w_unit_slot_read_back;
    if (!w_visit_the_copy_read_back()) {
        w_merge_below(unit_device, unit_slot);
        return;
    }
    if (!w_judge_unit_header(s, 1u, fsid_low)) {
        return;
    }
    w_judge_birth_identity(s, p, unit_device, unit_slot);
    // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 1 五元组里的树）
    w_judge(INVARIANT_I_1_12, eq64(p.birth_tree, rd64(s.base, 43u)));
    w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
    w_judge(INVARIANT_I_1_1, eq64(rd64(s.base, 51u), inode) && eq64(rd64(s.base, 67u), offset) && eq64(rd64(s.base, 43u), tree));
    if (w_context == 0u) {
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

// 按位置寻址的节点：层级、区间、条目宽（walk.rs position_and_entry_width_hold）；交回还往不往下走
fn w_position_and_entry_width_hold(n: Node, level: u32, range_ok: bool, entry_width: u32) -> bool {
    w_judge(INVARIANT_I_1_1, n.level == level);
    if (n.level != level) {
        w_failures = w_failures + 1u;
        return false;
    }
    w_judge(INVARIANT_I_1_1, range_ok);
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
            let read = w_read_index_node(pending_pointer, tree, 4u, 2u, true, fsid_low);
            if (!read.ok) {
                if (depth == 0u) { break; }
                allocation_walk_frames[depth - 1u].child_index = allocation_walk_frames[depth - 1u].child_index + 1u;
                enter = false;
                continue;
            }
            let n = read.n;
            var range_ok = false;
            if (pending_node.is_root) {
                range_ok = all_zero(n.base, n.smallest_offset, 10u);
                for (var i = 0u; i < 10u; i = i + 1u) {
                    if (rd8(n.base, n.largest_offset + i) != 0xFFu) {
                        range_ok = false;
                    }
                }
            } else {
                range_ok = allocation_key_bytes_equal(n.base, n.smallest_offset, pending_node.device, allocation_first_slot(pending_node.level, pending_node.index))
                    && allocation_key_bytes_equal(n.base, n.largest_offset, pending_node.device, allocation_last_slot(pending_node.level, pending_node.index));
            }
            var entry_width = 96u;
            if (pending_node.level == 0u) {
                entry_width = 20u;
            }
            if (!w_position_and_entry_width_hold(n, pending_node.level, range_ok, entry_width)) {
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
                    let span = rd16(n.base, o + 10u) & 0x7FFFu;
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
            let read = w_read_index_node(pending_pointer, tree, 3u, 2u, true, fsid_low);
            if (!read.ok) {
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
            let span = extent_lower_span(level);
            let first = mul64_64_saturating(pending_index, span);
            let last = add64_saturating(first, sub64(span, u64_of(1u)));
            let first_offset = mul64_64_saturating(first, payload);
            let last_offset = sub64_saturating(mul64_64_saturating(add64_saturating(last, u64_of(1u)), payload), u64_of(1u));
            let range_ok = w_extent_range_ok(n, inode, first_offset, inode, last_offset);
            var entry_width = 110u;
            if (level == 0u) {
                entry_width = 112u;
            }
            if (!w_position_and_entry_width_hold(n, level, range_ok, entry_width)) {
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
            let read = w_read_index_node(pending_pointer, tree, 3u, key_range_reading_of_this_node, true, fsid_low);
            if (!read.ok) {
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
            let span = extent_upper_span(level);
            let first = mul64_64_saturating(pending_index, span);
            let last = add64_saturating(first, sub64(span, u64_of(1u)));
            let range_ok = w_extent_range_ok(n, first, vec2<u32>(0u, 0u), last, vec2<u32>(NONE, NONE));
            var entry_width = 110u;
            if (level == 0u) {
                entry_width = 113u;
            }
            if (!w_position_and_entry_width_hold(n, level, range_ok, entry_width)) {
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
                    if (tag == 0u) {
                        well_formed = well_formed && all_zero(n.base, o + 25u, 88u);
                    } else if (tag == 1u) {
                        well_formed = well_formed && all_zero(n.base, o + 25u + 86u, 2u);
                    } else if (tag != 2u) {
                        well_formed = false;
                    }
                    w_judge(INVARIANT_I_1_1, well_formed && le64(first, entry_inode) && le64(entry_inode, last));
                    if (well_formed && tag == 1u) {
                        let lower_root = parse_node_pointer(n.base, o + 25u);
                        w_judge_location_order(lower_root);
                        w_walk_extent_lower(lower_root, entry_inode, tree, fsid_low);
                        // 下段走完把父单元换回这片叶
                        w_parent_valid = true;
                        w_parent_device = read.unit_device;
                        w_parent_slot = read.unit_slot;
                    } else if (well_formed && tag == 2u) {
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

// ── 多层码 2 树（walk_code_two_subtree）：schema 0 记账（I-1.10：叶 34、内部 108）、1 映射（只守走读要的字节：叶 55、内部 113）
// 叶条目记进 R_ACCOUNTING / R_MAPPING（字节基、偏移、宽）
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
    var region = R_ACCOUNTING;
    var capacity = MAX_ACCOUNTING_ENTRIES;
    let registered = schema == 0u;
    if (schema == 1u) {
        leaf_width = 55u;
        internal_width = 113u;
        region = R_MAPPING;
        capacity = MAX_MAPPING_ENTRIES;
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
                let read = w_read_index_node(pending_pointer, tree, schema, 1u, registered, fsid_low);
                if (!read.ok) {
                    if (read.already_walked) {
                        result_kind = 1u;
                    }
                } else {
                    let n = read.n;
                    var walkable = true;
                    if (n.key_width != key_width) {
                        w_failures = w_failures + 1u;
                        walkable = false;
                    }
                    if (walkable && pending_expected_level != NONE) {
                        w_judge(INVARIANT_I_1_1, n.level == pending_expected_level);
                        if (n.level != pending_expected_level) {
                            w_failures = w_failures + 1u;
                            walkable = false;
                        }
                    }
                    if (walkable) {
                        var width = internal_width;
                        if (n.level == 0u) {
                            width = leaf_width;
                        }
                        if (registered) {
                            w_judge(INVARIANT_I_1_10, n.entry_width == width);
                            walkable = n.entry_width == width;
                        } else if (n.entry_width < width) {
                            w_failures = w_failures + 1u;
                            walkable = false;
                        }
                    }
                    if (walkable) {
                        if (n.level == 0u) {
                            for (var e = 0u; e < n.entry_count; e = e + 1u) {
                                if (entries >= capacity) {
                                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (11u << 8u);
                                    break;
                                }
                                sws(region + entries * 3u, n.base);
                                sws(region + entries * 3u + 1u, entry_offset(n, e));
                                sws(region + entries * 3u + 2u, n.entry_width);
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

// walk_central_mapping_entries：I-2.5、I-1.11、I-2.1；映射条目指的单元记进引用集合（不判同一版第二次引用）
fn w_walk_central_mapping_entries(entries: u32, fsid_low: vec2<u32>) {
    for (var i = 0u; i < entries; i = i + 1u) {
        let base = sw(R_MAPPING + i * 3u);
        let o = sw(R_MAPPING + i * 3u + 1u);
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
        w_note_reference(p.device0, p.slot0, p.sum0, span);
        w_note_reference(p.device1, p.slot1, p.sum1, span);
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

fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, fsid_low: vec2<u32>) {
    if (root.level == 0u) {
        return;
    }
    if (w_context == 0u) {
        w_inode_tree_walked = true;
    }
    // 叶序：这一趟走出来的叶容器依次记进 C_LEAVES（父 = 根），供 I-9.4 第二、三句沿叶序比
    let leaves_before = w_leaf_count;
    w_separator_count = 0u;
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
        let child = parse_node_pointer(root.base, o + 34u);
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
            continue;
        }
        // 类型段不认得、上面也没认出孩子自证过的类：不按哪一种长度去读孩子（walk.rs walk_inode_internal_node 的 Unregistered 那一支，
        // 与墓碑树、谱系树类型段不认得那一支同判；第 27 批）——孩子按这一层定的形状登记引用（层级 1 下面是叶容器两槽，层级 ≥ 2 下面是码 2 节点一槽），
        // 判 I-9.2 红；每一份副本照判 I-2.1、两份都对不上记走读失败（第 29 批，H-J3a）
        if (class_the_inode_entry_type_names == NONE) {
            var child_span_by_the_level = 2u;
            if (root.level >= 2u) {
                child_span_by_the_level = 1u;
            }
            w_note_pointer_references(child, child_span_by_the_level);
            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
            w_judge(INVARIANT_I_9_2, false);
            continue;
        }
        w_note_pointer_references(child, span);
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
                w_merge_below(leaf_device, leaf_slot);
                if (w_context == 0u) {
                    w_cached_leaves_below(leaf_device, leaf_slot, leaves_before);
                }
                continue;
            }
            w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
            w_judge_birth_identity(s, child, leaf_device, leaf_slot);
            let packed = w_judge_packed_container(s, 2u, fsid_low);
            if (!packed.ok) {
                continue;
            }
            let p = packed.p;
            w_judge(INVARIANT_I_9_13, p.record_count > 0u);
            var have_range = false;
            var smallest = vec2<u32>(0u, 0u);
            var largest = vec2<u32>(0u, 0u);
            for (var r = 0u; r < p.record_count; r = r + 1u) {
                let ro = packed_record_offset(p, r);
                let inode = rd64(p.base, ro);
                // I-9.4 第四句：同一片打包记录类型 2 容器里记录号沿记录序严格递增（判决 `research/prompts/m3-delete-checker-r2-main-verification.md`
                // 第二节 T11 取甲；walk.rs walk_inode_leaf_container 逐对判）
                if (r > 0u) {
                    w_judge(INVARIANT_I_9_4, lt64(rd64(p.base, packed_record_offset(p, r - 1u)), inode));
                }
                w_judge(INVARIANT_I_9_7, all_zero(p.base, ro + 108u, 32u));
                w_judge(INVARIANT_I_9_4, le64(p.container, inode));
                let size = rd64(p.base, ro + INODE_RECORD_SIZE_OFFSET);
                let blocks = rd64(p.base, ro + 48u);
                let logical_blocks = shr64(add64_saturating(size, u64_of(511u)), 9u);
                w_judge(INVARIANT_I_9_15, eq64(blocks, logical_blocks));
                // I-9.26：对象出生代（记录偏移 INODE_RECORD_OBJECT_BIRTH_OFFSET）≤ 改动计数（偏移 88）≤ 这片容器头的诞生代号（偏移 73）
                let change_count = rd64(p.base, ro + 88u);
                w_judge(INVARIANT_I_9_26, le64(rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET), change_count) && le64(change_count, p.birth_txg));
                if (w_context == 0u) {
                    // inode → 对象出生代（后写的覆盖先写的）
                    let birth = rd64(p.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                    var found = false;
                    for (var i = 0u; i < w_inode_birth_count; i = i + 1u) {
                        if (sw(C_INODE_BIRTHS + i * 4u) == inode.x && sw(C_INODE_BIRTHS + i * 4u + 1u) == inode.y) {
                            sws(C_INODE_BIRTHS + i * 4u + 2u, birth.x);
                            sws(C_INODE_BIRTHS + i * 4u + 3u, birth.y);
                            found = true;
                        }
                    }
                    if (!found) {
                        if (w_inode_birth_count >= C_MAX_INODE_BIRTHS) {
                            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (13u << 8u);
                        } else {
                            sws(C_INODE_BIRTHS + w_inode_birth_count * 4u, inode.x);
                            sws(C_INODE_BIRTHS + w_inode_birth_count * 4u + 1u, inode.y);
                            sws(C_INODE_BIRTHS + w_inode_birth_count * 4u + 2u, birth.x);
                            sws(C_INODE_BIRTHS + w_inode_birth_count * 4u + 3u, birth.y);
                            w_inode_birth_count = w_inode_birth_count + 1u;
                        }
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
                // I-9.12 的 (分隔 key, 孩子最小, 孩子最大)
                if (w_separator_count >= C_MAX_SEPARATORS) {
                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (15u << 8u);
                } else {
                    let separator = rd64(root.base, o);
                    let srow = C_SEPARATORS + w_separator_count * C_SEPARATOR_WORDS;
                    sws(srow, separator.x);
                    sws(srow + 1u, separator.y);
                    sws(srow + 2u, smallest.x);
                    sws(srow + 3u, smallest.y);
                    sws(srow + 4u, largest.x);
                    sws(srow + 5u, largest.y);
                    w_separator_count = w_separator_count + 1u;
                }
            }
        } else {
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
    // I-9.4 第二句：沿叶序相邻两片容器号严格递增（第三句「左容器的最大 key < 右容器号」随 N1 删掉：用户 2026-10-08 定案 X4，
    // 左从右借之后右片保留身份，它的容器号可以不大于左片的最大 key；walk.rs judge_container_numbers_along_the_leaf_order 同改）
    for (var i = leaves_before + 1u; i < w_leaf_count; i = i + 1u) {
        let left = C_LEAVES + (i - 1u) * C_LEAF_WORDS;
        let right = C_LEAVES + i * C_LEAF_WORDS;
        let left_container = vec2<u32>(sw(left + 3u), sw(left + 4u));
        let right_container = vec2<u32>(sw(right + 3u), sw(right + 4u));
        w_judge(INVARIANT_I_9_4, lt64(left_container, right_container));
    }
}

// ── 树表的一条条目（walk_tree_table_entry）
fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
    if (tt.entry_width < 200u) {
        w_failures = w_failures + 1u;
        return;
    }
    let o = entry_offset(tt, e);
    let tree = rd64(tt.base, o);
    let kind = rd16(tt.base, o + 10u);
    // I-9.25：previous_snapshot_txg（条目偏移 100）只写在 inode 树那条里、不大于这一版的 txg；别的种类恒 0。
    // 树表被几版共享时只在第一个走到它的那一版判（走过的单元不重走）
    let previous_snapshot_txg = rd64(tt.base, o + 100u);
    if (kind == TREE_KIND_INODE) {
        w_judge(INVARIANT_I_9_25, le64(previous_snapshot_txg, w_version_txg));
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
    if (kind == TREE_KIND_EXTENT) { schema = 3u; registered = true; field_table_width = 112u; }
    else if (kind == TREE_KIND_INODE) { schema = 2u; registered = true; field_table_width = 120u; }
    else if (kind == TREE_KIND_ALLOCATION) { schema = 4u; registered = true; field_table_width = 20u; }
    else if (kind == TREE_KIND_ACCOUNTING) { schema = 0u; registered = true; field_table_width = 34u; }
    else if (kind == 5u) { schema = 1u; }
    else if (kind == TREE_KIND_DELETED_INODES) { schema = 2u; registered = true; field_table_width = DELETED_INODES_ENTRY_BYTES; }
    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { schema = 2u; registered = true; field_table_width = CONTAINER_TREE_ENTRY_BYTES; }
    if (schema == NONE) {
        return;
    }
    let pointer = parse_node_pointer(tt.base, o + 14u);
    if (kind == TREE_KIND_INODE && !pointer.all_zero) {
        let s = read_at(pointer.device0, slot_to_bytes(pointer.slot0), NODE_BYTES);
        if (s.version != NONE) {
            w_judge(INVARIANT_I_9_1, rd8(s.base, 6u) == 2u);
        }
    }
    if (kind == TREE_KIND_ACCOUNTING) {
        let walked = w_walk_code_two_tree(pointer, tree, 0u, fsid_low);
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
    if (kind == TREE_KIND_EXTENT) {
        if (!pointer.all_zero) {
            w_walk_extent_tree(pointer, tree, fsid_low);
        }
        return;
    }
    let read = w_read_index_node(pointer, tree, schema, 0u, registered, fsid_low);
    if (!read.ok) {
        return;
    }
    if (registered) {
        w_judge(INVARIANT_I_1_10, read.n.entry_width == field_table_width);
        if (read.n.entry_width != field_table_width) {
            return;
        }
    }
    if (kind == TREE_KIND_INODE) {
        w_walk_inode_root(read.n, tree, read.unit_device, read.unit_slot, fsid_low);
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
// 两份都对不上记走读失败（walk.rs 的 Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into，第 29 批 H-J3a、H-J3b）。不往下走，引用由调用方先登记
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
        let child = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
        w_judge_location_order(child);
        w_judge_pointer_head(child);
        w_judge_pointer_extent_offset(child);
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
            // F-ID-b，用户 2026-10-07 定）；两份都没自证过的当读不出、这一小句不判（用户 2026-10-07 定）
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
            let read = w_read_index_node(child, tree, 2u, 0u, true, fsid_low);
            if (!read.ok) {
                continue;
            }
            w_judge(INVARIANT_I_1_1, read.n.level + 1u == n.level);
            if (read.n.level + 1u == n.level && read.n.entry_width == CONTAINER_TREE_ENTRY_BYTES) {
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
        w_note_pointer_references(child, 2u);
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
            w_merge_below(container_device, container_slot);
            continue;
        }
        w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
        w_judge_birth_identity(s, child, container_device, container_slot);
        // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 判；记录本身的不变量归 I-9.17 起那几条，走读不判
        _ = w_judge_packed_container(s, record_type, fsid_low);
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
    // 不记引用。根环里的根、由记录施加出来的那几版、重放施加出来的那一版都经这里（用户 2026-10-06 定 Q1、Q3「甲」）
    if (tree_table_pointer.all_zero) {
        _ = w_read_referenced_unit(tree_table_pointer, NODE_BYTES);
        w_failures = w_failures + 1u;
        return;
    }
    let read = w_read_index_node(tree_table_pointer, vec2<u32>(0u, 0u), 2u, 0u, false, fsid_low);
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
    let mapping = w_walk_code_two_tree(mapping_root_pointer, mapping_root_pointer.birth_tree, 1u, fsid_low);
    w_walk_central_mapping_entries(mapping.entries, fsid_low);
}

// ── 一条根（walk_root）
// 走一遍：一条根（WALK_KIND_ROOT：实例表与分配记录树、树表与映射都取自这条根记录），或一版由记录施加出来、根槽从没落盘的版本
// （WALK_KIND_VERSION：四条指针都取自那条记录的新根段——施加一条记录 = 把所选根的七个字段换成记录里的，D23（journal 的角色与格式）
// 已定项 15，用户 2026-10-02「回到甲」；walk.rs 的 walk_version_applied_only_by_records 同一个取法）。
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
    w_walk_instance_table_chain(parse_node_pointer(record_base, instance_table_pointer_offset), is_newest, fsid_low);
    let allocation_root = parse_node_pointer(record_base, allocation_root_pointer_offset);
    if (!allocation_root.all_zero) {
        w_walk_allocation_record_tree(allocation_root, vec2<u32>(0u, 0u), fsid_low);
    }
    w_walk_tree_table_and_mapping(parse_node_pointer(record_base, tree_table_pointer_offset), parse_node_pointer(record_base, mapping_root_pointer_offset), fsid_low);
}

// ───────────────────────── I-3.5（引用区间的精确性）的出现规则：版本都走完之后逐对判 ─────────────────────────
// walk.rs 的 HoleInTheCandidateSet
const APPEARANCE_NO_HOLE: u32 = 0u;
const APPEARANCE_HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: u32 = 1u;
const APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: u32 = 2u;
// 判一版时它去重之后的单元（版本表一行里的落点折成单元）摊在这里：每个单元 2 个字（落点表下标、走过的单元表下标或 NONE）。
// 借 w_merge_below 的两块（C_PENDING 与紧跟其后的 C_EXPANDED），判的时候主走读那几版已经走完、不再并入
const APPEARANCE_UNIT_WORDS: u32 = 2u;
const APPEARANCE_UNITS: u32 = C_PENDING;
const APPEARANCE_MAX_UNITS: u32 = (C_SCRATCH_END - C_PENDING) / APPEARANCE_UNIT_WORDS;

fn appearance_version_row(version_index: u32) -> u32 {
    return C_APPEARANCE_VERSIONS + version_index * C_APPEARANCE_VERSION_WORDS;
}

fn appearance_version_txg(version_index: u32) -> vec2<u32> {
    let row = appearance_version_row(version_index);
    return vec2<u32>(sw(row + 1u), sw(row + 2u));
}

// 落点表一行的 (盘, 槽) 比另一行的大（BTreeMap 的键序：先盘、后槽）
fn appearance_placement_is_greater(index: u32, other: u32) -> bool {
    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
    let other_row = C_APPEARANCE_PLACEMENTS + other * C_APPEARANCE_PLACEMENT_WORDS;
    if (sw(row) != sw(other_row)) {
        return sw(row) > sw(other_row);
    }
    return lt64(vec2<u32>(sw(other_row + 1u), sw(other_row + 2u)), vec2<u32>(sw(row + 1u), sw(row + 2u)));
}

// 两行落点折成的是同一个单元
fn appearance_same_unit(index: u32, other: u32) -> bool {
    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
    let other_row = C_APPEARANCE_PLACEMENTS + other * C_APPEARANCE_PLACEMENT_WORDS;
    return sw(row + 3u) == sw(other_row + 3u) && sw(row + 4u) == sw(other_row + 4u) && sw(row + 5u) == sw(other_row + 5u);
}

// 落点表一行打的用户可见树标签是不是 (kind, tree)
fn appearance_placement_is_in_tree(index: u32, kind: u32, tree: vec2<u32>) -> bool {
    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
    return sw(row + 6u) == 1u && sw(row + 7u) == kind && sw(row + 8u) == tree.x && sw(row + 9u) == tree.y;
}

// 一版引用的落点里第 k 个排在第 j 个前面（k、j 是版本表那一段里的序号，first 是那一段的起点）：先按折成的单元 (盘, 槽)，同一个单元的
// 按落点自己的 (盘, 槽)（appearance_placement_is_greater 的序），都相同的按序号——全序；同一个单元的落点连成一段、段尾是它 (盘, 槽) 最大的那个
fn appearance_version_placement_precedes(first: u32, k: u32, j: u32) -> bool {
    let placement_k = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k);
    let placement_j = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + j);
    let row_k = C_APPEARANCE_PLACEMENTS + placement_k * C_APPEARANCE_PLACEMENT_WORDS;
    let row_j = C_APPEARANCE_PLACEMENTS + placement_j * C_APPEARANCE_PLACEMENT_WORDS;
    if (sw(row_k + 3u) != sw(row_j + 3u)) {
        return sw(row_k + 3u) < sw(row_j + 3u);
    }
    let unit_slot_k = vec2<u32>(sw(row_k + 4u), sw(row_k + 5u));
    let unit_slot_j = vec2<u32>(sw(row_j + 4u), sw(row_j + 5u));
    if (!eq64(unit_slot_k, unit_slot_j)) {
        return lt64(unit_slot_k, unit_slot_j);
    }
    if (appearance_placement_is_greater(placement_j, placement_k)) {
        return true;
    }
    if (appearance_placement_is_greater(placement_k, placement_j)) {
        return false;
    }
    return k < j;
}

// C_ROW_ORDER 装得下一版引用的落点（解析内核时判，不进展开量）
const_assert C_MAX_IN_VERSION <= C_JOURNAL_GROUP_KEYS - C_ROW_ORDER;

// 一版引用的 count 个落点（版本表那一段从 first 起）按 appearance_version_placement_precedes 排，序号写进 C_ROW_ORDER 起的 count 个字
// （至多 C_MAX_IN_VERSION 个，那张表 C_MAX_REFERENCES 个字；I-3.5 在逐盘 journal 之后、I-5.1 之前判，那两处借它的时段不相交）。
// 堆排序，写法同 order_rows_by_their_first_four_words
fn appearance_order_the_placements_of_a_version_by_unit(first: u32, count: u32) {
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
            if (left + 1u < heap_size && appearance_version_placement_precedes(first, sw(C_ROW_ORDER + left), sw(C_ROW_ORDER + left + 1u))) {
                larger = left + 1u;
            }
            if (!appearance_version_placement_precedes(first, sw(C_ROW_ORDER + node), sw(C_ROW_ORDER + larger))) {
                break;
            }
            let parent = sw(C_ROW_ORDER + node);
            sws(C_ROW_ORDER + node, sw(C_ROW_ORDER + larger));
            sws(C_ROW_ORDER + larger, parent);
            node = larger;
        }
    }
}

// 一版去重之后的单元摊进 APPEARANCE_UNITS，交回几个（判不了交回 NONE）。note_the_version_just_walked_for_the_appearance_rule 把这一版的落点
// 按 (盘, 槽) 升序折成单元收进 BTreeMap，同一个单元后收的标签盖掉先收的：这里每个单元留它 (盘, 槽) 最大的那个落点（标签取那一个的）。
// 原来每个落点跟这一版全部落点两两比「有没有同单元、更大的」，条数平方长；改成按 (单元, 落点) 排一次序、每段留段尾——
// 同单元的落点连成一段，段尾就是那个没有同单元更大落点的（一版的落点两两不同：w_judge_placement_in_version 去过重）。
// 摊开的次序换成按单元排，读它的两处（appearance_tree_is_inherited_whole、逐个单元判 I-3.5）每个单元各判各的、与次序无关
fn appearance_spread_the_units_of_a_version(version_index: u32) -> u32 {
    let row = appearance_version_row(version_index);
    let first = sw(row + 4u);
    let count = sw(row + 5u);
    appearance_order_the_placements_of_a_version_by_unit(first, count);
    var units = 0u;
    for (var position = 0u; position < count; position = position + 1u) {
        let placement = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + sw(C_ROW_ORDER + position));
        if (position + 1u < count
            && appearance_same_unit(placement, sw(C_APPEARANCE_VERSION_PLACEMENTS + first + sw(C_ROW_ORDER + position + 1u)))) {
            continue;
        }
        if (units >= APPEARANCE_MAX_UNITS) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (45u << 8u);
            return NONE;
        }
        let placement_row = C_APPEARANCE_PLACEMENTS + placement * C_APPEARANCE_PLACEMENT_WORDS;
        sws(APPEARANCE_UNITS + units * APPEARANCE_UNIT_WORDS, placement);
        sws(APPEARANCE_UNITS + units * APPEARANCE_UNIT_WORDS + 1u,
            w_visited_index_in_the_main_walk(sw(placement_row + 3u), vec2<u32>(sw(placement_row + 4u), sw(placement_row + 5u))));
        units = units + 1u;
    }
    return units;
}

// 摊开的第 u 个单元有没有记下诞生代号（被引用单元那一支标记 1、实例表一片标记 2；头读不出的没有）
fn appearance_unit_birth_is_known(u: u32) -> bool {
    let visited_index = sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS + 1u);
    return visited_index != NONE && sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS) != 0u;
}

fn appearance_unit_birth_txg(u: u32) -> vec2<u32> {
    let visited_index = sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS + 1u);
    return vec2<u32>(sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u), sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u));
}

// txg 为 previous_txg 的那几版（R⁻ 那一组）里有没有一版引用了与落点表第 placement 行同一个单元
fn appearance_previous_group_references_the_unit_of(previous_txg: vec2<u32>, placement: u32) -> bool {
    for (var other = 0u; other < appearance_version_count; other = other + 1u) {
        if (!eq64(appearance_version_txg(other), previous_txg)) {
            continue;
        }
        let row = appearance_version_row(other);
        let first = sw(row + 4u);
        let count = sw(row + 5u);
        for (var k = 0u; k < count; k = k + 1u) {
            if (appearance_same_unit(sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k), placement)) {
                return true;
            }
        }
    }
    return false;
}

// walk.rs hole_in_the_candidate_set_between：R⁻ 那一组（txg previous_txg 的那几版）与这一版 (version_instance, version_txg) 之间的洞；
// 接得上 ⟺ R⁻ 的某个实例 i：最新根的实例表里有一行 (i, T = R⁻.txg)，这一版的实例 v > i，i 与 v 之间每个实例都有一行、每一行都是 (j, 0, 0)
fn appearance_hole_between(previous_txg: vec2<u32>, version_txg: vec2<u32>, version_instance: u32) -> u32 {
    let previous_txg_has_a_successor = !(previous_txg.x == NONE && previous_txg.y == NONE);
    if (previous_txg_has_a_successor && eq64(add64(previous_txg, u64_of(1u)), version_txg)) {
        return APPEARANCE_NO_HOLE;
    }
    for (var other = 0u; other < appearance_version_count; other = other + 1u) {
        if (!eq64(appearance_version_txg(other), previous_txg)) {
            continue;
        }
        let previous_instance = sw(appearance_version_row(other));
        if (version_instance <= previous_instance) {
            continue;
        }
        var row_of_the_previous_version = false;
        var instances_between = 0u;
        var every_row_between_published_nothing = true;
        for (var k = 0u; k < w_instance_row_count; k = k + 1u) {
            let row = C_INSTANCE_ROWS + k * C_ROW_WORDS;
            let instance = sw(row);
            let row_txg = vec2<u32>(sw(row + 1u), sw(row + 2u));
            let row_high_water = vec2<u32>(sw(row + 3u), sw(row + 4u));
            if (instance == previous_instance && eq64(row_txg, previous_txg)) {
                row_of_the_previous_version = true;
            }
            if (instance <= previous_instance || instance >= version_instance) {
                continue;
            }
            if (!is_zero64(row_txg) || !is_zero64(row_high_water)) {
                every_row_between_published_nothing = false;
            }
            // 中间的实例按不同的代号数（同一个实例有两行只算一个）
            var counted_before = false;
            for (var j = 0u; j < k; j = j + 1u) {
                if (sw(C_INSTANCE_ROWS + j * C_ROW_WORDS) == instance) {
                    counted_before = true;
                    break;
                }
            }
            if (!counted_before) {
                instances_between = instances_between + 1u;
            }
        }
        if (row_of_the_previous_version && instances_between == version_instance - previous_instance - 1u && every_row_between_published_nothing) {
            return APPEARANCE_HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS;
        }
    }
    return APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS;
}

// 这一版（摊开在 APPEARANCE_UNITS 里，units 个）树表里第 root_index 棵用户可见树是不是整棵继承下来的：根指针出生 txg 严格小于 R⁻.txg，
// 且这一版里那棵树下读得出的诞生代号都不大于它
fn appearance_tree_is_inherited_whole(version_index: u32, root_index: u32, units: u32, previous_txg: vec2<u32>) -> bool {
    let root_row = appearance_version_row(version_index) + C_APPEARANCE_VERSION_ROOTS_OFFSET + root_index * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
    let kind = sw(root_row);
    let tree = vec2<u32>(sw(root_row + 1u), sw(root_row + 2u));
    let root_pointer_birth_txg = vec2<u32>(sw(root_row + 3u), sw(root_row + 4u));
    if (!lt64(root_pointer_birth_txg, previous_txg)) {
        return false;
    }
    for (var u = 0u; u < units; u = u + 1u) {
        if (!appearance_placement_is_in_tree(sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS), kind, tree) || !appearance_unit_birth_is_known(u)) {
            continue;
        }
        if (!le64(appearance_unit_birth_txg(u), root_pointer_birth_txg)) {
            return false;
        }
    }
    return true;
}

// walk.rs judge_the_appearance_rule_of_adjacent_candidate_versions：回退候选集（走过的每一版）按 txg 升序、同 txg 并成一组，
// 每一版 R 与 txg 比它小的那几版里 txg 最大的那一组 R⁻ 比：R 引用而 R⁻ 不引用、诞生代号读得出的每个单元 b 满足
// (a) birth(b) > R⁻.txg，或 (b) b 打着一棵整棵继承下来的用户可见树的标签。有洞而实例表行接不上时打着用户可见树标签的不判（N3）；
// R 或 R⁻ 那一组有一版走读记过失败的那一对不判。一个单元都没判到时 CPU 报不适用，违例位上与成立一样是 0
fn judge_the_appearance_rule_of_adjacent_candidate_versions() {
    for (var version_index = 0u; version_index < appearance_version_count; version_index = version_index + 1u) {
        let row = appearance_version_row(version_index);
        let version_txg = appearance_version_txg(version_index);
        var has_previous = false;
        var previous_txg = vec2<u32>(0u, 0u);
        for (var other = 0u; other < appearance_version_count; other = other + 1u) {
            let other_txg = appearance_version_txg(other);
            if (lt64(other_txg, version_txg) && (!has_previous || lt64(previous_txg, other_txg))) {
                previous_txg = other_txg;
                has_previous = true;
            }
        }
        if (!has_previous || sw(row + 3u) != 1u) {
            continue;
        }
        var previous_references_are_complete = true;
        for (var other = 0u; other < appearance_version_count; other = other + 1u) {
            if (eq64(appearance_version_txg(other), previous_txg) && sw(appearance_version_row(other) + 3u) != 1u) {
                previous_references_are_complete = false;
            }
        }
        if (!previous_references_are_complete) {
            continue;
        }
        let hole = appearance_hole_between(previous_txg, version_txg, sw(row));
        let units = appearance_spread_the_units_of_a_version(version_index);
        if (units == NONE) {
            return;
        }
        // 这一版里整棵继承下来的用户可见树（第 r 棵是就置位 r）
        var trees_inherited_whole = 0u;
        let roots = sw(row + 6u);
        for (var r = 0u; r < roots; r = r + 1u) {
            if (appearance_tree_is_inherited_whole(version_index, r, units, previous_txg)) {
                trees_inherited_whole = trees_inherited_whole | (1u << r);
            }
        }
        for (var u = 0u; u < units; u = u + 1u) {
            let placement = sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS);
            if (appearance_previous_group_references_the_unit_of(previous_txg, placement)) {
                continue;
            }
            let placement_row = C_APPEARANCE_PLACEMENTS + placement * C_APPEARANCE_PLACEMENT_WORDS;
            let in_a_user_visible_tree = sw(placement_row + 6u) == 1u;
            if (hole == APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS && in_a_user_visible_tree) {
                continue;
            }
            if (!appearance_unit_birth_is_known(u)) {
                continue;
            }
            let born_after_the_previous_version = lt64(previous_txg, appearance_unit_birth_txg(u));
            var in_a_tree_inherited_whole = false;
            for (var r = 0u; r < roots; r = r + 1u) {
                let root_row = row + C_APPEARANCE_VERSION_ROOTS_OFFSET + r * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
                if ((trees_inherited_whole & (1u << r)) != 0u
                    && appearance_placement_is_in_tree(placement, sw(root_row), vec2<u32>(sw(root_row + 1u), sw(root_row + 2u)))) {
                    in_a_tree_inherited_whole = true;
                }
            }
            judge(INVARIANT_I_3_5, born_after_the_previous_version || in_a_tree_inherited_whole);
        }
    }
}

// ───────────────────────── 第二批：走读与走读之后（check_pool_image 择根之后那一段）─────────────────────────

// 恢复眼里的 journal 环：每个计数器取按盘次序第一块盘上自证过的那一份（records_as_the_recovery_reads_them）
fn checker_scan_journal_as_the_recovery_reads(fsid_low: vec2<u32>) {
    record_count = 0u;
    let device_count = hdr(3u);
    let location_count = hdr(6u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        for (var l = 0u; l < location_count; l = l + 1u) {
            if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
                continue;
            }
            let s = read_location(l);
            if (!checker_record_valid(s, fsid_low)) {
                continue;
            }
            let offset = vec2<u32>(location_word(l, 1u), location_word(l, 2u));
            let ring_slot = shr64(sub64(offset, u64_of(JOURNAL_RING_START_BYTES)), 12u);
            checker_insert_record(s.version, rd48(s.base, 20u), rd32(s.base, 16u), ring_slot);
        }
    }
}

// ───────────────────────── I-4.1：checker 自己复刻的只读重放（walk.rs 的 replay_the_next_mount_would_do）─────────────────────────
// 扫描内核认下一次挂载先施加的那一版（I-3.10、I-5.4）读同一份重放的结局：第一次施加的发布的末条经交接段交过去，不另造一份重放。

// 重放前缀读的那一份环（records_the_replay_reads）：每块盘上自证过、恢复读者读得出的记录（提交标记 0 / 1、记录标志位 0 之外全 0、
// 本次发布内序号不是 0），按 (实例代号, 计数器) 建键、按盘的次序先到先得，升序排在 C_REPLAY_RECORDS。
// 同一块盘上两条记录撞同一个计数器在环里不出现（计数器与环槽一一对应，walk.rs 的 scanned_journal_records_of_device），这里不另去重。
// 交回条数；多于 MAX_RECORDS 条判不了。
fn collect_the_records_the_replay_reads(fsid_low: vec2<u32>) -> u32 {
    var count = 0u;
    let device_count = hdr(3u);
    let location_count = hdr(6u);
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        for (var l = 0u; l < location_count; l = l + 1u) {
            if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
                continue;
            }
            let s = read_location(l);
            if (!checker_record_valid(s, fsid_low)) {
                continue;
            }
            if (rd8(s.base, 86u) > 1u || (rd8(s.base, 7u) & 0xFEu) != 0u || rd32(s.base, 87u) == 0u) {
                continue;
            }
            let instance = rd32(s.base, 16u);
            let counter = rd48(s.base, 20u);
            let position = replay_record_position_not_below(count, instance, counter);
            if (replay_record_key_at_is(count, position, instance, counter)) {
                continue;
            }
            if (count >= MAX_RECORDS) {
                undecidable = UNDECIDABLE_TOO_MANY_RECORDS;
                return count;
            }
            var i = count;
            while (i > position) {
                for (var k = 0u; k < C_REPLAY_RECORD_WORDS; k = k + 1u) {
                    sws(C_REPLAY_RECORDS + i * C_REPLAY_RECORD_WORDS + k, sw(C_REPLAY_RECORDS + (i - 1u) * C_REPLAY_RECORD_WORDS + k));
                }
                i = i - 1u;
            }
            let row = C_REPLAY_RECORDS + position * C_REPLAY_RECORD_WORDS;
            sws(row, s.version);
            sws(row + 1u, counter.x);
            sws(row + 2u, counter.y);
            sws(row + 3u, instance);
            count = count + 1u;
        }
    }
    return count;
}

// 重放读的那一份环（前 count 条，按 (实例, 计数器) 严格升序）里第一个键不小于 (instance, counter) 的位置，都小交回 count。
// 二分：每一轮 [low, high) 之前的键都小于它、从 high 起的都不小于它，至多 ⌈log2(MAX_RECORDS + 1)⌉ = 12 轮
fn replay_record_position_not_below(count: u32, instance: u32, counter: vec2<u32>) -> u32 {
    var low = 0u;
    var high = count;
    while (low < high) {
        let middle = (low + high) / 2u;
        let row = C_REPLAY_RECORDS + middle * C_REPLAY_RECORD_WORDS;
        let middle_instance = sw(row + 3u);
        if (middle_instance < instance || (middle_instance == instance && lt64(vec2<u32>(sw(row + 1u), sw(row + 2u)), counter))) {
            low = middle + 1u;
        } else {
            high = middle;
        }
    }
    return low;
}

fn replay_record_key_at_is(count: u32, position: u32, instance: u32, counter: vec2<u32>) -> bool {
    let row = C_REPLAY_RECORDS + position * C_REPLAY_RECORD_WORDS;
    return position < count && sw(row + 3u) == instance && sw(row + 1u) == counter.x && sw(row + 2u) == counter.y;
}

// 重放读的那一份环里 (实例, 计数器) 那一条的下标，没有就是 NONE
fn replay_record_index(count: u32, instance: u32, counter: vec2<u32>) -> u32 {
    let position = replay_record_position_not_below(count, instance, counter);
    if (replay_record_key_at_is(count, position, instance, counter)) {
        return position;
    }
    return NONE;
}

// 带末条标志的第 index 条之后，同一实例里计数器更大的记录有没有同一个 checkpoint_txg 的：按计数器往后找，先遇到 txg 更大的就停
// （readable_record_of_the_same_publish_follows；表按 (实例, 计数器) 升序，同实例的记录连着）
fn readable_record_of_the_same_publish_follows_in_the_replay(count: u32, index: u32, r: CheckerRecord) -> bool {
    for (var j = index + 1u; j < count; j = j + 1u) {
        let later = replay_record_at(j);
        if (later.instance != r.instance) {
            return false;
        }
        if (lt64(r.txg, later.txg)) {
            return false;
        }
        if (eq64(later.txg, r.txg)) {
            return true;
        }
    }
    return false;
}

// 一条记录的点名单元是不是每一个都两份都读得出、整单元校验和都对（前缀第四条）；类标签不在登记表里的当验不过
fn named_units_of_the_record_verify_on_both_copies(r: CheckerRecord) -> bool {
    for (var n = 0u; n < r.named_count; n = n + 1u) {
        let o = JOURNAL_HEADER_BYTES + n * JOURNAL_NAMED_ENTRY_BYTES;
        let named_class = rd8(r.base, o + 28u);
        var unit_bytes = 0u;
        if (named_class == 1u || named_class == 3u) {
            unit_bytes = DATA_UNIT_BYTES;
        } else if (named_class == 2u) {
            unit_bytes = NODE_BYTES;
        } else {
            return false;
        }
        for (var copy = 0u; copy < 2u; copy = copy + 1u) {
            let location = o + copy * 14u;
            let s = read_at(rd32(r.base, location), slot_to_bytes(rd48(r.base, location + 4u)), unit_bytes);
            if (s.version == NONE || !slice_crc_matches(s, unit_bytes, rd32(r.base, location + 10u))) {
                return false;
            }
        }
    }
    return true;
}

// 下一次挂载在所选根之上施加的那一段记录的结局（walk.rs 的 ReplayTheNextMountWouldDo），两个都是重放读的那一份环里的下标：
// last_record_of_the_last_publish_applied —— 最后一次施加的发布的末条，施加出来的那一版 = 所选根换上它的新根段（I-4.1 走它）；
//   一次发布都不施加、锚点认不定（所选根那次发布带末条标志的多于一条）、所选根 txg 是 u64 的最大值、前缀停在点名单元两份没都验过的记录上，
//   是 NONE（I-4.1 报不适用）。
// last_record_of_the_first_publish_applied —— 第一次施加的发布的末条，下一次挂载先施加的那一版照它的新根段（walk.rs 的
//   version_the_next_mount_applies_first，改法 R1）。前缀停在后面某次发布的点名单元上时，停之前已整次收下的发布 core 照施加
//   （crates/singlefs-core/src/recovery.rs 的 replay_journal），这一个照交回；一次都没收下、锚点认不定、所选根 txg 没有后继时是 NONE。
struct ReplayTheNextMountWouldDo {
    last_record_of_the_last_publish_applied: u32,
    last_record_of_the_first_publish_applied: u32,
};

// 照 D23（journal 的角色与格式） 已定项 14 的全部规则复刻下一次挂载在所选根（chosen）之上施加哪一段记录。
// 在飞上限按环长与格式常量 F（3）现算：⌊环长 ÷ 4096 ÷ 3⌋。
fn replay_the_next_mount_would_do(chosen: CheckerRoot, ring_bytes: vec2<u32>, fsid_low: vec2<u32>) -> ReplayTheNextMountWouldDo {
    let nothing_applied = ReplayTheNextMountWouldDo(NONE, NONE);
    let count = collect_the_records_the_replay_reads(fsid_low);
    if (undecidable != 0u) {
        return nothing_applied;
    }
    var anchors = 0u;
    var anchor_counter = vec2<u32>(0u, 0u);
    for (var i = 0u; i < count; i = i + 1u) {
        let r = replay_record_at(i);
        if (r.instance == chosen.instance && eq64(r.txg, chosen.txg) && (r.flags & 1u) != 0u) {
            anchors = anchors + 1u;
            anchor_counter = r.counter;
        }
    }
    if (anchors >= 2u) {
        return nothing_applied;
    }
    var have_expected_next_counter = anchors == 1u;
    var expected_next_counter = add64(anchor_counter, u64_of(1u));
    if (chosen.txg.x == NONE && chosen.txg.y == NONE) {
        return nothing_applied;
    }
    let chain_start_txg_without_anchor = add64(chosen.txg, u64_of(1u));
    let in_flight_record_limit = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    var records_taken_before_this_one = 0u;
    var open_publish_records = 0u;
    // 开着的那次发布的上一条（open_publish_records 为 0 时不读它）
    var previous: CheckerRecord;
    var last_applied = NONE;
    var last_record_of_the_first_publish_applied = NONE;
    for (var i = 0u; i < count; i = i + 1u) {
        let r = replay_record_at(i);
        if (r.instance != chosen.instance || !lt64(chosen.txg, r.txg)) {
            continue;
        }
        if (!lt64(u64_of(records_taken_before_this_one), in_flight_record_limit)) {
            break;
        }
        records_taken_before_this_one = records_taken_before_this_one + 1u;
        var continues_the_chain = false;
        if (have_expected_next_counter) {
            continues_the_chain = eq64(r.counter, expected_next_counter);
        } else {
            continues_the_chain = eq64(r.txg, chain_start_txg_without_anchor) && r.ordinal == 1u;
        }
        // 反向链（record_back_chain_lets_it_into_the_replay_prefix）：计数器 0 进不了；同实例计数器小 1 的那一条在、链值不等就进不了
        var back_chain_lets_it_in = !is_zero64(r.counter);
        if (back_chain_lets_it_in) {
            let previous_index = replay_record_index(count, r.instance, sub64(r.counter, u64_of(1u)));
            if (previous_index != NONE) {
                back_chain_lets_it_in = r.back_chain == replay_record_at(previous_index).chain_value;
            }
        }
        if (!continues_the_chain || !back_chain_lets_it_in) {
            break;
        }
        var fits_the_open_publish = false;
        if (open_publish_records == 0u) {
            fits_the_open_publish = r.ordinal == 1u;
        } else {
            fits_the_open_publish = eq64(r.txg, previous.txg) && previous.ordinal != NONE && r.ordinal == previous.ordinal + 1u
                && (previous.commit_byte == 1u || eq64(r.transaction, previous.transaction));
        }
        if (!fits_the_open_publish) {
            break;
        }
        let ends_its_publish = (r.flags & 1u) != 0u;
        if (ends_its_publish && (r.commit_byte != 1u || readable_record_of_the_same_publish_follows_in_the_replay(count, i, r))) {
            break;
        }
        if (!named_units_of_the_record_verify_on_both_copies(r)) {
            return ReplayTheNextMountWouldDo(NONE, last_record_of_the_first_publish_applied);
        }
        expected_next_counter = add64(r.counter, u64_of(1u));
        have_expected_next_counter = true;
        open_publish_records = open_publish_records + 1u;
        previous = r;
        if (ends_its_publish) {
            // 第一次发布的那一版同取它的末条（已定项 15：core 施加末条那一份新根段，改法 FL 的那一取法）
            if (last_record_of_the_first_publish_applied == NONE) {
                last_record_of_the_first_publish_applied = i;
            }
            open_publish_records = 0u;
            last_applied = i;
        }
    }
    return ReplayTheNextMountWouldDo(last_applied, last_record_of_the_first_publish_applied);
}

// 走读的七个阶段，按次序；每个阶段里要走的每一处都经 walk_and_hand_off 循环里那一个调用点走
const WALK_PHASE_NEWEST_ROOT: u32 = 0u;
const WALK_PHASE_CANDIDATE_ROOTS: u32 = 1u;
const WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS: u32 = 2u;
const WALK_PHASE_ABANDONED_ROOTS: u32 = 3u;
const WALK_PHASE_HARVEST_ROOTS_OUTSIDE_THE_CANDIDATES: u32 = 4u;
const WALK_PHASE_ROOTS_BELOW_THE_FLOOR: u32 = 5u;
const WALK_PHASE_REPLAYED_VERSION: u32 = 6u;
const WALK_PHASE_DONE: u32 = 7u;
// 交回的值是结论里「走到了第几步」：3 走读那一批没判完（中途判不了），4 走读那一批判完、进了收尾
const WALK_REACHED_THE_ROOT_RING: u32 = 3u;
const WALK_REACHED_THE_HAND_OFF: u32 = 4u;

// 走读那一批连同收尾：择最新根、候选根、由记录施加出来的版本、被抛弃根，判走读之后那几条；
// 收尾是候选集之外的根整遍走一次收位置项校验和（第三批判隔离的记录要），F 之下、没被抛弃的根各走一遍，
// 它们引用的、候选版本没走到的槽记进交接段（第三批算 defer 那一格要），再把计数与几何写进交接段；
// 最后照下一次挂载的重放施加出那一版、另起一份走读走一遍（I-4.1），它只动上下文 1 的集合；同一份重放第一次施加的发布的末条写进交接段
// （H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED），扫描内核照它读重放读的那一份环里那一行（I-3.10、I-5.4）。
// 写成按阶段推进的一个循环：每一圈先找这一阶段下一处要走的（找不到就做完这一阶段收尾的事、转下一阶段），走，再做走完之后的事。
fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
    let device_count = hdr(3u);
    w_harvesting = false;
    c_version_count = 0u;
    c_candidate_count = 0u;
    // 择根：(txg, 实例) 最大，相等取读根环次序里最前的（只在严格更大时才换），与 CPU 的池级 checker 择根那一处
    // （`crates/singlefs-checker/src/walk.rs` 的 fold，代码轮第二轮判决辩方 D3）同一个择法（代码轮第三轮判决辩方 Z1）
    var newest = 0u;
    var newest_root = checker_root_of(0u);
    for (var i = 1u; i < root_count; i = i + 1u) {
        let r = checker_root_of(i);
        if (root_key_less(newest_root.txg, newest_root.instance, r.txg, r.instance)) {
            newest = i;
            newest_root = r;
        }
    }
    c_newest_index = newest;
    w_mount_instance = newest_root.instance;
    w_mount_txg = newest_root.txg;
    w_newest_instance_table_chain_reached_its_last_page = false;
    w_start(0u, true);
    var phase = WALK_PHASE_NEWEST_ROOT;
    var cursor = 0u;
    var newest_failures = 0u;
    var effective_floor = vec2<u32>(0u, 0u);
    var version_count = 0u;
    var below_floor_slots = 0u;
    loop {
        if (phase == WALK_PHASE_DONE) {
            break;
        }
        // ① 这一阶段下一处要走的
        var found = false;
        var walk_kind = WALK_KIND_ROOT;
        var walk_base = NONE;
        var walk_is_newest = false;
        if (phase == WALK_PHASE_NEWEST_ROOT) {
            walk_base = newest_root.base;
            walk_is_newest = true;
            found = true;
        } else if (phase == WALK_PHASE_CANDIDATE_ROOTS) {
            // 候选根：最新根，或没被抛弃且 txg ≥ F 生效值的
            while (cursor < root_count && !found) {
                let i = cursor;
                cursor = cursor + 1u;
                if (!is_candidate_root(i)) {
                    continue;
                }
                if (c_candidate_count >= C_MAX_CANDIDATES) {
                    undecidable = UNDECIDABLE_TOO_MANY_ROOTS;
                    return WALK_REACHED_THE_ROOT_RING;
                }
                sws(C_CANDIDATES + c_candidate_count, i);
                c_candidate_count = c_candidate_count + 1u;
                if (i == newest) {
                    continue;
                }
                walk_base = checker_root_of(i).base;
                found = true;
            }
            if (!found) {
                // 由记录施加出来、根槽从没落盘的那一版（versions_applied_only_by_records）
                var have_oldest_valid = false;
                var oldest_valid_txg = vec2<u32>(0u, 0u);
                for (var i = 0u; i < root_count; i = i + 1u) {
                    let r = checker_root_of(i);
                    if (w_root_abandoned(r.instance, r.txg)) {
                        continue;
                    }
                    if (!have_oldest_valid || lt64(r.txg, oldest_valid_txg)) {
                        oldest_valid_txg = r.txg;
                        have_oldest_valid = true;
                    }
                }
                version_count = 0u;
                checker_scan_journal_as_the_recovery_reads(fsid_low);
                if (have_oldest_valid) {
                    for (var i = 0u; i < record_count; i = i + 1u) {
                        let r = checker_record_at(i);
                        if (r.commit_byte != 1u) {
                            continue;
                        }
                        if (!record_can_enter_a_replay_prefix(i)) {
                            continue;
                        }
                        var applied = false;
                        for (var k = 0u; k < w_instance_row_count; k = k + 1u) {
                            if (sw(C_INSTANCE_ROWS + k * C_ROW_WORDS) == r.instance) {
                                let row_txg = vec2<u32>(sw(C_INSTANCE_ROWS + k * C_ROW_WORDS + 1u), sw(C_INSTANCE_ROWS + k * C_ROW_WORDS + 2u));
                                if (le64(r.txg, row_txg)) {
                                    applied = true;
                                }
                            }
                        }
                        if (!applied) {
                            continue;
                        }
                        var root_slot_is_readable = false;
                        for (var k = 0u; k < root_count; k = k + 1u) {
                            let root = checker_root_of(k);
                            if (root.instance == r.instance && eq64(root.txg, r.txg)) {
                                root_slot_is_readable = true;
                            }
                        }
                        if (root_slot_is_readable || lt64(r.txg, effective_floor) || !lt64(oldest_valid_txg, r.txg)) {
                            continue;
                        }
                        // 按 (实例, txg) 去重、升序插入；同键取带末条标志的那一条（改法 FL：core 施加末条那一份新根段），
                        // 一条带标志的都还没遇到时先占着计数器小的那一条（先到的），之后遇到带标志的换上它（walk.rs 的 SegmentTakenForTheVersion）
                        var position = 0u;
                        var duplicate = false;
                        while (position < version_count) {
                            let existing = checker_record_at(sw(R_DATA + position));
                            if (existing.instance == r.instance && eq64(existing.txg, r.txg)) {
                                duplicate = true;
                                break;
                            }
                            if (r.instance < existing.instance || (r.instance == existing.instance && lt64(r.txg, existing.txg))) {
                                break;
                            }
                            position = position + 1u;
                        }
                        if (duplicate) {
                            let existing = checker_record_at(sw(R_DATA + position));
                            if ((existing.flags & 1u) == 0u && (r.flags & 1u) != 0u) {
                                sws(R_DATA + position, i);
                            }
                            continue;
                        }
                        if (version_count >= MAX_DATA_POINTERS * 2u) {
                            undecidable = UNDECIDABLE_TOO_MANY_RECORDS;
                            return WALK_REACHED_THE_ROOT_RING;
                        }
                        var j = version_count;
                        while (j > position) {
                            sws(R_DATA + j, sw(R_DATA + j - 1u));
                            j = j - 1u;
                        }
                        sws(R_DATA + position, i);
                        version_count = version_count + 1u;
                    }
                }
                c_version_count = version_count;
                phase = WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS;
                cursor = 0u;
            }
        } else if (phase == WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS) {
            if (cursor < version_count) {
                let r = checker_record_at(sw(R_DATA + cursor));
                cursor = cursor + 1u;
                walk_kind = WALK_KIND_VERSION;
                walk_base = r.base;
                found = true;
            } else {
                // I-3.5：回退候选集的每一版（最新根、候选根、由记录施加出来的版本）都走完了，逐对判（check_pool_image 走完它们之后那一处）
                if (undecidable == 0u) {
                    judge_the_appearance_rule_of_adjacent_candidate_versions();
                }
                phase = WALK_PHASE_ABANDONED_ROOTS;
                cursor = 0u;
            }
        } else if (phase == WALK_PHASE_ABANDONED_ROOTS) {
            // I-7.4 被抛弃根那一半：另起一份走读，只看它引用的块读回来对不对
            while (w_instance_row_count > 0u && cursor < root_count && !found) {
                let i = cursor;
                cursor = cursor + 1u;
                if (i == newest) {
                    continue;
                }
                let r = checker_root_of(i);
                if (!w_root_abandoned(r.instance, r.txg)) {
                    continue;
                }
                w_start(1u, false);
                w_harvesting = false;
                walk_base = r.base;
                found = true;
            }
            if (!found) {
                // I-7.2
                judge(INVARIANT_I_7_2, newest_failures == 0u);
                // I-9.6：记账里的 inode 号水位 > inode 树内最大 key
                if (w_have_largest_inode) {
                    for (var k = 0u; k < w_accounting_row_count; k = k + 1u) {
                        if (sw(C_ACCOUNTING_ROWS + k * 4u) == 12u && sw(C_ACCOUNTING_ROWS + k * 4u + 1u) == NONE) {
                            let watermark = vec2<u32>(sw(C_ACCOUNTING_ROWS + k * 4u + 2u), sw(C_ACCOUNTING_ROWS + k * 4u + 3u));
                            judge(INVARIANT_I_9_6, lt64(w_largest_inode, watermark));
                        }
                    }
                }
                // I-9.10：数据单元的对象出生代与 inode 记录的相同
                for (var i = 0u; i < w_data_object_count; i = i + 1u) {
                    let inode = vec2<u32>(sw(C_DATA_OBJECTS + i * 4u), sw(C_DATA_OBJECTS + i * 4u + 1u));
                    let unit_birth = vec2<u32>(sw(C_DATA_OBJECTS + i * 4u + 2u), sw(C_DATA_OBJECTS + i * 4u + 3u));
                    for (var k = 0u; k < w_inode_birth_count; k = k + 1u) {
                        if (sw(C_INODE_BIRTHS + k * 4u) == inode.x && sw(C_INODE_BIRTHS + k * 4u + 1u) == inode.y) {
                            let record_birth = vec2<u32>(sw(C_INODE_BIRTHS + k * 4u + 2u), sw(C_INODE_BIRTHS + k * 4u + 3u));
                            judge(INVARIANT_I_9_10, eq64(record_birth, unit_birth));
                        }
                    }
                }
                // I-5.1：同一块盘上不同的引用不许占重叠的槽（按 (槽, 跨度) 排序后相邻两条比 ⟺ 任两条比）
                let reference_count = w_reference_count[0];
                // walk.rs 每块盘按 (槽, 跨度) 排好之后相邻两条各判一次：有一块盘上有两条以上的引用就算判过
                // 两两比的写法是「同盘的 i、j，(槽, 跨度) 字典序 i 严格在前、j 的槽 < i 的末端 ⇒ 违例」；有两条同盘的就算判过。引用集合按 (盘, 槽, 跨度)
                // 去重（w_note_reference），没有两条 (盘, 槽, 跨度) 都相同。按 (盘, 槽, 跨度, 行号) 排好之后只比相邻两条（同盘、后一条的槽 < 前一条的末端）
                // 与它等价：有违例的一对 (i, j) 时，i 后面那一条 k 排在 j 之前或就是 j、同盘，k 的槽 ≤ j 的槽 < i 的末端；反过来相邻两条违例就是一对；
                // 同盘的在次序里连成一段，有两条同盘就有相邻两条同盘
                order_rows_by_their_first_four_words(C_REFERENCES0, C_REFERENCE_WORDS, reference_count, C_ROW_ORDER);
                var some_device_has_two_references = false;
                for (var position = 1u; position < reference_count; position = position + 1u) {
                    let earlier = C_REFERENCES0 + sw(C_ROW_ORDER + position - 1u) * C_REFERENCE_WORDS;
                    let later = C_REFERENCES0 + sw(C_ROW_ORDER + position) * C_REFERENCE_WORDS;
                    if (sw(earlier) != sw(later)) {
                        continue;
                    }
                    some_device_has_two_references = true;
                    if (lt64(vec2<u32>(sw(later + 1u), sw(later + 2u)), add64(vec2<u32>(sw(earlier + 1u), sw(earlier + 2u)), u64_of(sw(earlier + 3u))))) {
                        violate(INVARIANT_I_5_1);
                    }
                }
                if (some_device_has_two_references) {
                    judge(INVARIANT_I_5_1, true);
                }
                // I-7.8：根环全部根的水位取 max > 盘上出现过的最大树 ID（码 2 单元头 ∪ 走过的树表条目）
                var watermark = vec2<u32>(0u, 0u);
                var newest_published_txg = vec2<u32>(0u, 0u);
                for (var i = 0u; i < root_count; i = i + 1u) {
                    let r = checker_root_of(i);
                    if (lt64(watermark, r.watermark)) {
                        watermark = r.watermark;
                    }
                    if (lt64(newest_published_txg, r.txg)) {
                        newest_published_txg = r.txg;
                    }
                }
                var highest_seen = vec2<u32>(0u, 0u);
                for (var i = 0u; i < w_tree_id_count; i = i + 1u) {
                    let tree = vec2<u32>(sw(C_TREE_IDS + i * 2u), sw(C_TREE_IDS + i * 2u + 1u));
                    if (lt64(highest_seen, tree)) {
                        highest_seen = tree;
                    }
                }
                // 扫描方向照认领规则按整单元跳（walk.rs 的 scanned_tree_identifiers）：落点表按 (盘, 偏移) 升序，每块盘跨轮带
                // 「下一个没被认领的槽」、只增不减；起点槽解得出单元头就认领它占的几槽，被认领的后几槽是它的载荷，不再读头
                var claiming_device = NONE;
                var first_slot_not_claimed = vec2<u32>(0u, 0u);
                let location_count = hdr(6u);
                for (var l = 0u; l < location_count; l = l + 1u) {
                    if (location_word(l, 4u) != LOCATION_UNIT_SLOT) {
                        continue;
                    }
                    let device = location_word(l, 0u);
                    let slot = shr64(vec2<u32>(location_word(l, 1u), location_word(l, 2u)), SLOT_SHIFT);
                    if (device != claiming_device) {
                        claiming_device = device;
                        first_slot_not_claimed = vec2<u32>(0u, 0u);
                    }
                    if (lt64(slot, first_slot_not_claimed)) {
                        continue;
                    }
                    let s = read_location(l);
                    let claimed_slots = slots_claimed_by_a_unit_header(s);
                    if (claimed_slots == 0u) {
                        continue;
                    }
                    first_slot_not_claimed = add64(slot, u64_of(claimed_slots));
                    if (rd8(s.base, 6u) != 2u) {
                        continue;
                    }
                    let key_span = 2u * rd8(s.base, 51u);
                    let birth_txg = rd64(s.base, 52u + key_span);
                    if (!le64(birth_txg, newest_published_txg)) {
                        continue;
                    }
                    let write_order_instance = unit_write_order_instance(s, fsid_low);
                    var written_after_applied = false;
                    if (write_order_instance != NONE) {
                        for (var k = 0u; k < w_instance_row_count; k = k + 1u) {
                            if (sw(C_INSTANCE_ROWS + k * C_ROW_WORDS) == write_order_instance) {
                                let row_txg = vec2<u32>(sw(C_INSTANCE_ROWS + k * C_ROW_WORDS + 1u), sw(C_INSTANCE_ROWS + k * C_ROW_WORDS + 2u));
                                if (!is_zero64(row_txg) && lt64(row_txg, birth_txg)) {
                                    written_after_applied = true;
                                }
                            }
                        }
                    }
                    if (written_after_applied) {
                        continue;
                    }
                    let tree = rd64(s.base, 42u);
                    if (lt64(highest_seen, tree)) {
                        highest_seen = tree;
                    }
                }
                judge(INVARIANT_I_7_8, lt64(highest_seen, watermark));
                // I-7.8 第二格：最新那条根自己的水位 > 盘上出现过的最大树 ID，与 CPU 的池级 checker 同判
                // （`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image` 末尾取 max 那一格之后那一格，C674）：
                // 只把最新那条根的水位压低时取 max 那一格被更早的根盖住，而恢复按所选根的水位往下发号
                judge(INVARIANT_I_7_8, lt64(highest_seen, newest_root.watermark));
                if (undecidable != 0u) {
                    return WALK_REACHED_THE_ROOT_RING;
                }
                w_start(1u, false);
                w_harvesting = true;
                phase = WALK_PHASE_HARVEST_ROOTS_OUTSIDE_THE_CANDIDATES;
                cursor = 0u;
            }
        } else if (phase == WALK_PHASE_HARVEST_ROOTS_OUTSIDE_THE_CANDIDATES) {
            while (cursor < root_count && !found) {
                let i = cursor;
                cursor = cursor + 1u;
                if (is_candidate_root(i)) {
                    continue;
                }
                walk_base = checker_root_of(i).base;
                found = true;
            }
            if (!found) {
                w_harvesting = false;
                below_floor_slots = 0u;
                phase = WALK_PHASE_ROOTS_BELOW_THE_FLOOR;
                cursor = 0u;
            }
        } else if (phase == WALK_PHASE_ROOTS_BELOW_THE_FLOOR) {
            while (cursor < root_count && !found) {
                let root = checker_root_of(cursor);
                cursor = cursor + 1u;
                if (w_root_abandoned(root.instance, root.txg) || !lt64(root.txg, c_effective_floor)) {
                    continue;
                }
                w_start(1u, false);
                walk_base = root.base;
                found = true;
            }
            if (!found) {
                sws(H_BASE + H_BELOW_FLOOR_SLOT_COUNT, below_floor_slots);
                sws(H_BASE + H_ROW_COUNT, w_instance_row_count);
                sws(H_BASE + H_REFERENCE_COUNT, w_reference_count[0]);
                sws(H_BASE + H_NEWEST_REFERENCE_COUNT, c_newest_reference_count);
                sws(H_BASE + H_CHECKSUM_COUNT, w_checksum_count);
                sws(H_BASE + H_HARVEST_COUNT, w_harvest_count);
                sws(H_BASE + H_ACCOUNTING_ROW_COUNT, w_accounting_row_count);
                sws(H_BASE + H_ACCOUNTING_SEEN, select(0u, 1u, w_accounting_seen));
                sws(H_BASE + H_CANDIDATE_COUNT, c_candidate_count);
                sws(H_BASE + H_NEWEST_INDEX, c_newest_index);
                sws(H_BASE + H_FLOOR_LO, c_effective_floor.x);
                sws(H_BASE + H_FLOOR_HI, c_effective_floor.y);
                sws(H_BASE + H_VERSION_COUNT, c_version_count);
                sws(H_BASE + H_RECORD_COUNT, record_count);
                sws(H_BASE + H_MOUNT_INSTANCE, w_mount_instance);
                sws(H_BASE + H_MOUNT_TXG_LO, w_mount_txg.x);
                sws(H_BASE + H_MOUNT_TXG_HI, w_mount_txg.y);
                sws(H_BASE + H_ROOT_COUNT, root_count);
                sws(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE, select(0u, 1u, w_newest_instance_table_chain_reached_its_last_page));
                sws(H_BASE + H_UNIT_AREA_START_LO, g.unit_area_start_slot.x);
                sws(H_BASE + H_UNIT_AREA_START_HI, g.unit_area_start_slot.y);
                sws(H_BASE + H_REGIONS, g.regions);
                sws(H_BASE + H_SLOTS_PER_REGION, g.slots_per_region);
                sws(H_BASE + H_REGION0, g.region0);
                sws(H_BASE + H_REGION1, g.region1);
                sws(H_BASE + H_REGION2, g.region2);
                sws(H_BASE + H_FSID0, g.fsid.x);
                sws(H_BASE + H_FSID1, g.fsid.y);
                sws(H_BASE + H_FSID2, g.fsid.z);
                sws(H_BASE + H_FSID3, g.fsid.w);
                phase = WALK_PHASE_REPLAYED_VERSION;
            }
        } else if (phase == WALK_PHASE_REPLAYED_VERSION) {
            // I-4.1：一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过，都报不适用（不走）
            let replay = replay_the_next_mount_would_do(newest_root, g.ring_bytes, fsid_low);
            // 下一次挂载先施加的那一版（I-3.10、I-5.4）交给扫描内核：H_VALID 只在走过这一阶段之后才置 1
            sws(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED, replay.last_record_of_the_first_publish_applied);
            let last_applied = replay.last_record_of_the_last_publish_applied;
            if (last_applied == NONE || undecidable != 0u) {
                phase = WALK_PHASE_DONE;
            } else {
                w_start(1u, false);
                w_harvesting = false;
                walk_kind = WALK_KIND_VERSION;
                walk_base = checker_record_of_row(C_REPLAY_RECORDS + last_applied * C_REPLAY_RECORD_WORDS).base;
                found = true;
            }
        }
        if (!found) {
            continue;
        }
        // ② 走：全内核唯一的调用点。这一版的 txg：根记录偏移 28，记录偏移 26
        w_version_is_an_older_candidate_root = phase == WALK_PHASE_CANDIDATE_ROOTS;
        if (walk_kind == WALK_KIND_ROOT) {
            w_version_txg = rd64(walk_base, 28u);
        } else {
            w_version_txg = rd64(walk_base, 26u);
        }
        let mismatches_before = w_mismatches;
        let failures_before = w_failures;
        w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);
        w_version_is_an_older_candidate_root = false;
        // I-3.5：主走读走的那几版（最新根、候选根、由记录施加出来的版本）各记一份它引用了哪些单元；这一版的实例与 txg：根记录偏移 24 / 28，记录偏移 16 / 26
        if (phase == WALK_PHASE_NEWEST_ROOT || phase == WALK_PHASE_CANDIDATE_ROOTS || phase == WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS) {
            var walked_instance = rd32(walk_base, 24u);
            if (walk_kind == WALK_KIND_VERSION) {
                walked_instance = rd32(walk_base, 16u);
            }
            appearance_note_the_version_just_walked(walked_instance, w_version_txg, w_failures == failures_before);
        }
        // ③ 走完之后这一阶段要做的
        if (phase == WALK_PHASE_NEWEST_ROOT) {
            c_newest_reference_count = w_reference_count[0];
            newest_failures = w_failures;
            let newest_walked_into_reused_or_erased_unit = w_failures > 0u || w_mismatches > 0u;
            w_judge(INVARIANT_I_4_8, !newest_walked_into_reused_or_erased_unit);
            w_judge(INVARIANT_I_7_4, !newest_walked_into_reused_or_erased_unit);
            // F 生效值：每盘最新有效根的 F 的最大值、被抛弃根的 F、自证过属于本池的系统配置槽的 F，取大
            effective_floor = vec2<u32>(0u, 0u);
            for (var d = 0u; d < device_count; d = d + 1u) {
                let device = hdr(12u + d);
                var have = false;
                var best = checker_root_of(0u);
                for (var i = 0u; i < root_count; i = i + 1u) {
                    let r = checker_root_of(i);
                    if (w_root_abandoned(r.instance, r.txg)) {
                        continue;
                    }
                    if (checker_region_device(g, checker_root_region(i)) != device) {
                        continue;
                    }
                    if (!have || root_key_less(best.txg, best.instance, r.txg, r.instance)) {
                        best = r;
                        have = true;
                    }
                }
                if (have && lt64(effective_floor, best.floor)) {
                    effective_floor = best.floor;
                }
            }
            for (var i = 0u; i < root_count; i = i + 1u) {
                let r = checker_root_of(i);
                if (w_root_abandoned(r.instance, r.txg) && lt64(effective_floor, r.floor)) {
                    effective_floor = r.floor;
                }
            }
            for (var d = 0u; d < device_count; d = d + 1u) {
                for (var k = 0u; k < 2u; k = k + 1u) {
                    let slot = slot_readings[d * 2u + k];
                    if (slot.reading == 0u && same_fsid(slot.fsid, g.fsid) && lt64(effective_floor, slot.floor)) {
                        effective_floor = slot.floor;
                    }
                }
            }
            c_effective_floor = effective_floor;
            phase = WALK_PHASE_CANDIDATE_ROOTS;
            cursor = 0u;
        } else if (phase == WALK_PHASE_CANDIDATE_ROOTS || phase == WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS) {
            let walked_into = w_mismatches > mismatches_before || w_failures > failures_before;
            w_judge(INVARIANT_I_7_4, !walked_into);
            w_judge(INVARIANT_I_4_8, !walked_into);
        } else if (phase == WALK_PHASE_ABANDONED_ROOTS) {
            judge(INVARIANT_I_7_4, !(w_failures > 0u || w_mismatches > 0u));
        } else if (phase == WALK_PHASE_REPLAYED_VERSION) {
            // I-4.1：施加出来的那一版走不完（任一单元两份都读不对、头用不了、条目解不开）
            judge(INVARIANT_I_4_1, w_failures == 0u);
            phase = WALK_PHASE_DONE;
        } else if (phase == WALK_PHASE_ROOTS_BELOW_THE_FLOOR) {
            for (var r = 0u; r < w_reference_count[1]; r = r + 1u) {
                let row = C_REFERENCES1 + r * C_REFERENCE_WORDS;
                let device = sw(row);
                let start = vec2<u32>(sw(row + 1u), sw(row + 2u));
                let span = sw(row + 3u);
                for (var k = 0u; k < span; k = k + 1u) {
                    let slot = add64(start, u64_of(k));
                    if (c_walked_slot(device, slot)) {
                        continue;
                    }
                    var present = false;
                    for (var q = 0u; q < below_floor_slots; q = q + 1u) {
                        if (sw(H_BELOW_FLOOR_SLOTS + q * 3u) == device && sw(H_BELOW_FLOOR_SLOTS + q * 3u + 1u) == slot.x && sw(H_BELOW_FLOOR_SLOTS + q * 3u + 2u) == slot.y) {
                            present = true;
                        }
                    }
                    if (present) {
                        continue;
                    }
                    if (below_floor_slots >= H_MAX_BELOW_FLOOR_SLOTS) {
                        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (18u << 8u);
                        return WALK_REACHED_THE_HAND_OFF;
                    }
                    sws(H_BELOW_FLOOR_SLOTS + below_floor_slots * 3u, device);
                    sws(H_BELOW_FLOOR_SLOTS + below_floor_slots * 3u + 1u, slot.x);
                    sws(H_BELOW_FLOOR_SLOTS + below_floor_slots * 3u + 2u, slot.y);
                    below_floor_slots = below_floor_slots + 1u;
                }
            }
        }
    }
    return WALK_REACHED_THE_HAND_OFF;
}


// ───────────────────────── 择根之前（check_pool_image 读系统配置与根环、逐盘 journal 那一段）─────────────────────────

// I-7.16：每块盘上可择的系统配置槽 i，世代号 ≥ 2 时世代号 mod 2 == i；两槽都可择时世代号相差恰为 1，或两槽都是 mkfs 写下的 1
// （image::judge_system_configuration_slot_generations_follow_the_rotation；带越界值的槽归 I-7.13，走不到这里）
fn judge_system_configuration_slot_generations_follow_the_rotation(device_count: u32) {
    for (var d = 0u; d < device_count; d = d + 1u) {
        let zero = slot_readings[d * 2u];
        let one = slot_readings[d * 2u + 1u];
        if (zero.reading == SLOT_USABLE && lt64(u64_of(1u), zero.generation)) {
            judge(INVARIANT_I_7_16, (zero.generation.x & 1u) == 0u);
        }
        if (one.reading == SLOT_USABLE && lt64(u64_of(1u), one.generation)) {
            judge(INVARIANT_I_7_16, (one.generation.x & 1u) == 1u);
        }
        if (zero.reading == SLOT_USABLE && one.reading == SLOT_USABLE) {
            let both_written_by_mkfs = eq64(zero.generation, u64_of(1u)) && eq64(one.generation, u64_of(1u));
            var one_apart = false;
            if (lt64(zero.generation, one.generation)) {
                one_apart = eq64(sub64(one.generation, zero.generation), u64_of(1u));
            } else {
                one_apart = eq64(sub64(zero.generation, one.generation), u64_of(1u));
            }
            judge(INVARIANT_I_7_16, both_written_by_mkfs || one_apart);
        }
    }
}

// I-7.5：根环 R × S 个槽逐个判，起始偏移（设备内字节）是判定宽度的整数倍（image::judge_root_slots_aligned_to_the_decision_width）。
// 偏移照 image::root_slot_positions 按 64 位算：根环起点 × 16384 + 区域 × P × chunk + 槽 × 槽距。
// 判定宽度 0 不会出现（择到的槽 physical_block_size ≥ 457），出现了这一格判红、不除。
fn judge_root_slots_aligned_to_the_decision_width(g: CheckerSlot, decision_width: u32) {
    let root_ring_base_bytes = slot_to_bytes(g.base_slot);
    for (var region = 0u; region < g.regions; region = region + 1u) {
        let region_bytes = mul64_32(mul32x32(region, g.prime_step), g.chunk_bytes).value;
        for (var slot = 0u; slot < g.slots_per_region; slot = slot + 1u) {
            let offset = add64(add64(root_ring_base_bytes, region_bytes), mul32x32(slot, g.spacing));
            judge(INVARIANT_I_7_5, decision_width != 0u && is_zero64(divmod64(offset, u64_of(decision_width)).remainder));
        }
    }
}

// I-7.17：根环里每条自证过的根住 (区域 txg mod R, 槽 (txg div R) mod S)；第 0 代的根只判槽 0
// （image::judge_root_records_live_in_the_slot_their_txg_names；根环里读出了根 ⇒ R ≥ 1、S ≥ 4）
fn judge_root_records_live_in_the_slot_their_txg_names(g: CheckerSlot) {
    for (var i = 0u; i < root_count; i = i + 1u) {
        let txg = checker_root_of(i).txg;
        let region = checker_root_region(i);
        let slot = checker_root_slot_in_its_region(i);
        if (is_zero64(txg)) {
            judge(INVARIANT_I_7_17, slot == 0u);
            continue;
        }
        let by_region = divmod64(txg, u64_of(g.regions));
        let slot_of_the_txg = divmod64(by_region.quotient, u64_of(g.slots_per_region)).remainder;
        judge(INVARIANT_I_7_17, by_region.remainder.x == region && slot_of_the_txg.x == slot);
    }
}

// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节的起点与字节数（D22（单元原子性怎么合成） 已定项 7 的字段表：偏移 428、429、441，
// 合计到 457；image::ROOT_RECORD_ALGORITHM_NONCE_AND_MAC 是 428..457）
const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET: u32 = 428u;
const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES: u32 = 29u;

// I-2.4 根记录那一处：根环里每一条自证过的根（checker_collect_roots，不按候选集滤），根槽 428..457 全 0
// （image::judge_root_record_algorithm_nonce_and_mac_are_zero；这一版 checker 只收加密类型 0 的系统配置，加密关着时恒 0）。只判不断
fn judge_root_record_algorithm_nonce_and_mac_are_zero() {
    for (var root_index = 0u; root_index < root_count; root_index = root_index + 1u) {
        judge(INVARIANT_I_2_4, all_zero(checker_root_of(root_index).base, ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_OFFSET, ROOT_RECORD_ALGORITHM_NONCE_AND_MAC_BYTES));
    }
}

// check_pool_image 从读系统配置槽到走读那一段，交回结论里「走到了第几步」。整池不作保的两支先判：任一盘任一槽带这一版读者不收的值
// （I-7.13）、任一盘任一可择槽 I-8.1 的三个字段有一格不等（两支都中时走 I-7.13 那一支，image 与 walk 的次序）——中了就只报那一条、
// 别的不判；再 I-8.5（全池没有一个自证过的槽）、择系统配置、系统配置与根环那几条、逐盘 journal，根环里有根才进走读。
// 判到位跟 CPU 的三支走：带越界值那一支只交 I-7.13 那一份判定；I-8.1 三个字段不等那一支 CPU 另起一份判定、只交它（这里清掉之前记的评估过）；
// 一块盘都择不到时 I-8.5 只有判红才算判到（那一支 CPU 判过而成立报不适用）
fn judge_the_pool_from_its_system_configuration_slots(device_count: u32) -> u32 {
    var any_slot_carries_a_refused_value = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        for (var k = 0u; k < 2u; k = k + 1u) {
            let reading = slot_readings[d * 2u + k].reading;
            if (reading == SLOT_USABLE) {
                judge(INVARIANT_I_7_13, true);
            } else if (reading == SLOT_CARRIES_A_REFUSED_VALUE) {
                judge(INVARIANT_I_7_13, false);
                any_slot_carries_a_refused_value = true;
            }
        }
    }
    if (any_slot_carries_a_refused_value) {
        return 1u;
    }
    for (var d = 0u; d < device_count; d = d + 1u) {
        for (var k = 0u; k < 2u; k = k + 1u) {
            let slot = slot_readings[d * 2u + k];
            if (slot.reading == SLOT_USABLE && !journal_ring_geometry_fields_agree(slot)) {
                clear_judged();
                judge(INVARIANT_I_8_1, false);
                return 1u;
            }
        }
    }
    for (var d = 0u; d < device_count; d = d + 1u) {
        for (var k = 0u; k < 2u; k = k + 1u) {
            if (slot_readings[d * 2u + k].reading == SLOT_USABLE) {
                judge(INVARIANT_I_8_1, true);
            }
        }
    }
    var some_slot_verifies_itself = false;
    for (var d = 0u; d < device_count; d = d + 1u) {
        for (var k = 0u; k < 2u; k = k + 1u) {
            let reading = slot_readings[d * 2u + k].reading;
            if (reading == SLOT_USABLE || reading == SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE) {
                some_slot_verifies_itself = true;
            }
        }
    }
    if (!some_slot_verifies_itself) {
        violate(INVARIANT_I_8_5);
    }
    // 每盘择世代号大的那一槽（相等取槽 0），几何取第一块择到的盘的；I-7.5 的判定宽度取各盘所选那一槽的 max(physical_block_size, io_min) 里最大的
    var chosen_valid = false;
    var g = empty_checker_slot(SLOT_NOT_SELF_VERIFIED);
    var decision_width = 0u;
    for (var d = 0u; d < device_count; d = d + 1u) {
        let zero = slot_readings[d * 2u];
        let one = slot_readings[d * 2u + 1u];
        var chosen = empty_checker_slot(SLOT_NOT_SELF_VERIFIED);
        if (zero.reading == SLOT_USABLE && one.reading == SLOT_USABLE) {
            if (lt64(zero.generation, one.generation)) {
                chosen = one;
            } else {
                chosen = zero;
            }
        } else if (zero.reading == SLOT_USABLE) {
            chosen = zero;
        } else if (one.reading == SLOT_USABLE) {
            chosen = one;
        }
        if (chosen.reading != SLOT_USABLE) {
            continue;
        }
        decision_width = max(decision_width, max(chosen.physical_block_size, chosen.minimum_input_output_bytes));
        if (!chosen_valid) {
            g = chosen;
            chosen_valid = true;
        }
        // 每块择到的盘（第一块也算）各判一次 I-1.4 与 I-7.15（各盘所选的系统配置槽设备数相同，不要求等于盘表）
        judge(INVARIANT_I_1_4, same_fsid(chosen.fsid, g.fsid));
        judge(INVARIANT_I_7_15, chosen.device_count == g.device_count);
    }
    if (!chosen_valid) {
        return 0u;
    }
    judge(INVARIANT_I_8_5, some_slot_verifies_itself);
    judge_system_configuration_slot_generations_follow_the_rotation(device_count);
    judge_root_slots_aligned_to_the_decision_width(g, decision_width);
    // I-7.6：根环区域的设备
    var distinct = 0u;
    var heaviest = 0u;
    var all_in_pool = true;
    let region_count = min(g.regions, 3u);
    for (var r = 0u; r < region_count; r = r + 1u) {
        let device = checker_region_device(g, r);
        var seen_before = false;
        for (var q = 0u; q < r; q = q + 1u) {
            if (checker_region_device(g, q) == device) {
                seen_before = true;
            }
        }
        if (!seen_before) {
            distinct = distinct + 1u;
        }
        if (!device_is_in_the_pool(device)) {
            all_in_pool = false;
        }
    }
    for (var d = 0u; d < device_count; d = d + 1u) {
        var load = 0u;
        for (var r = 0u; r < region_count; r = r + 1u) {
            if (checker_region_device(g, r) == hdr(12u + d)) {
                load = load + 1u;
            }
        }
        heaviest = max(heaviest, load);
    }
    let pigeonhole = (g.regions + device_count - 1u) / device_count;
    judge(INVARIANT_I_7_6, distinct == min(g.regions, device_count) && heaviest <= pigeonhole && all_in_pool);
    let fsid_low = vec2<u32>(g.fsid.x, g.fsid.y);
    checker_collect_roots(g);
    judge_root_record_algorithm_nonce_and_mac_are_zero();
    judge_root_records_live_in_the_slot_their_txg_names(g);
    // 自证过、属于本池的槽：I-7.14 本盘设备号；I-7.12 每盘的 F
    for (var d = 0u; d < device_count; d = d + 1u) {
        let device = hdr(12u + d);
        var highest_floor = vec2<u32>(0u, 0u);
        var have_slot = false;
        for (var k = 0u; k < 2u; k = k + 1u) {
            let slot = slot_readings[d * 2u + k];
            if (slot.reading != SLOT_USABLE || !same_fsid(slot.fsid, g.fsid)) {
                continue;
            }
            judge(INVARIANT_I_7_14, slot.this_device == device);
            if (!have_slot || lt64(highest_floor, slot.floor)) {
                highest_floor = slot.floor;
            }
            have_slot = true;
        }
        if (!have_slot) {
            continue;
        }
        // image::judge_system_configuration_floor_against_the_roots_on_each_device：有本池的槽的盘各判一次（这块盘上 F 最大的那条根不超过槽里的 F）
        var floors_hold = true;
        for (var i = 0u; i < root_count; i = i + 1u) {
            if (checker_region_device(g, checker_root_region(i)) != device) {
                continue;
            }
            if (lt64(highest_floor, checker_root_of(i).floor)) {
                floors_hold = false;
            }
        }
        judge(INVARIANT_I_7_12, floors_hold);
    }
    // I-7.7：每盘两槽里本池的槽中 journal 实例代号的最大者；有一块盘没有本池的槽就不判
    var every_device_has_a_slot = true;
    var pool_highest = 0u;
    var pool_lowest = NONE;
    for (var d = 0u; d < device_count; d = d + 1u) {
        var highest = NONE;
        for (var k = 0u; k < 2u; k = k + 1u) {
            let slot = slot_readings[d * 2u + k];
            if (slot.reading == SLOT_USABLE && same_fsid(slot.fsid, g.fsid)) {
                if (highest == NONE || slot.journal_instance > highest) {
                    highest = slot.journal_instance;
                }
            }
        }
        if (highest == NONE) {
            every_device_has_a_slot = false;
        } else {
            pool_highest = max(pool_highest, highest);
            pool_lowest = min(pool_lowest, highest);
        }
    }
    // image::judge_instance_carriers：GPU 的表里每个落点都读得出（没有 some_slot_is_unreadable 那一支），两句各判一次
    if (every_device_has_a_slot) {
        judge(INVARIANT_I_7_7, !some_instance_carrier_exceeds(pool_highest, g));
        judge(INVARIANT_I_7_7, !(pool_lowest < pool_highest && some_instance_carrier_exceeds(pool_lowest, g)));
    }
    // journal 那几条与点名项的位置条目次序，逐盘
    for (var d = 0u; d < device_count; d = d + 1u) {
        checker_scan_journal_of_device(hdr(12u + d), fsid_low);
        judge_journal_back_chain_of_device();
        judge_named_entry_location_order_of_device();
        judge_transaction_numbers_of_device();
        // 按组判的那几条读 C_ROW_ORDER：I-8.8 按 (实例, 事务号) 排；之后 I-8.9、I-8.1、I-8.10 按 (实例, checkpoint_txg) 排一次、三条共用（中间不改记录表）
        order_the_records_of_device_by_group(JOURNAL_GROUP_BY_TRANSACTION);
        judge_commit_markers_of_device();
        order_the_records_of_device_by_group(JOURNAL_GROUP_BY_PUBLISH);
        judge_publish_ordinals_of_device();
        judge_journal_record_txg_per_instance_of_device();
        judge_records_of_one_publish_within_the_in_flight_record_limit_of_device(g.declared_in_flight_record_limit);
        judge_named_entries_against_the_unit_headers_of_device();
        judge_new_root_segment_of_the_last_record_against_the_roots_of_device();
    }
    // I-7.1 / I-7.3
    judge(INVARIANT_I_7_1, root_count != 0u);
    if (root_count == 0u) {
        return 2u;
    }
    var newest = vec2<u32>(0u, 0u);
    for (var i = 0u; i < root_count; i = i + 1u) {
        let txg = checker_root_of(i).txg;
        if (lt64(newest, txg)) {
            newest = txg;
        }
    }
    var earlier_exists = false;
    var every_genesis = true;
    for (var i = 0u; i < root_count; i = i + 1u) {
        let txg = checker_root_of(i).txg;
        if (lt64(txg, newest)) {
            earlier_exists = true;
        }
        if (!is_zero64(txg)) {
            every_genesis = false;
        }
    }
    judge(INVARIANT_I_7_3, earlier_exists || every_genesis);
    let reached = walk_and_hand_off(g, fsid_low);
    if (reached == WALK_REACHED_THE_HAND_OFF && undecidable == 0u) {
        sws(H_BASE + H_VALID, 1u);
    }
    return reached;
}

// ───────────────────────── 入口 ─────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    clear_violated();
    clear_judged();
    let out = local * VERDICT_WORDS;
    sws(H_BASE + H_VALID, 0u);
    let device_count = hdr(3u);
    var reached = 0u;
    if (device_count > MAX_DEVICES) {
        undecidable = UNDECIDABLE_TOO_MANY_DEVICES;
    } else {
        read_every_system_configuration_slot();
        reached = judge_the_pool_from_its_system_configuration_slots(device_count);
    }
    store_violated(out);
    store_judged(out);
    verdicts[out + POOL_CHECKER_REACHED_WORD] = reached;
    verdicts[out + POOL_CHECKER_UNDECIDABLE_WORD] = undecidable;
}
