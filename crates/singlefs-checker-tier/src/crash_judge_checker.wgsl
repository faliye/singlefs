// GPU 判器的池级 checker 走读内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：按 singlefs-checker 自己的读法
// 判第一批（系统配置、根环、journal）与第二批（走读那 29 条），违例位写进每个状态结论的第 8、9 字（IMPLEMENTED_INVARIANTS 次序），
// 第 10 字是走到的批次、第 11 字是判不了的原因；走完把扫描内核要的计数与几何写进草稿区的交接段（第三批在 crash_judge_checker_scan.wgsl）。


// ───────────────────────── checker 读系统配置槽 ─────────────────────────

// 一槽读下来：0 可择、1 带这一版读者不收的值（I-7.13 违例）、2 不可择
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
    return c;
}

// image::read_system_configuration_slot：check_system_configuration_slot 再 geometry_of，第一条不成立的规则定这一槽是「拒」还是「不可择」
fn checker_read_system_configuration_slot(device: u32, offset: u32) -> CheckerSlot {
    let s = read_at(device, u64_of(offset), SYSTEM_CONFIGURATION_SLOT_BYTES);
    if (s.version == NONE || s.length < SYSTEM_CONFIGURATION_SLOT_BYTES) {
        return empty_checker_slot(2u);
    }
    let base = s.base;
    if (rd32(base, 0u) != MAGIC_SFSB || s.c0 != 1u) {
        return empty_checker_slot(2u);
    }
    if (rd16(base, 4u) != 1u) {
        return empty_checker_slot(1u);
    }
    let incompat0 = rd8(base, 6u);
    if ((incompat0 & 0xFDu) != 0u || !all_zero(base, 7u, 31u) || (incompat0 & 2u) == 0u) {
        return empty_checker_slot(2u);
    }
    if (rd8(base, 219u) != 0u) {
        return empty_checker_slot(1u);
    }
    // geometry_of
    let regions = rd8(base, 361u);
    if (regions > 3u) {
        return empty_checker_slot(2u);
    }
    let slots_per_region = rd8(base, 362u);
    if (slots_per_region < 4u || slots_per_region > 16u) {
        return empty_checker_slot(2u);
    }
    let ring_start_slot = rd64(base, 325u);
    let ring_bytes = rd64(base, 333u);
    let base_slot = rd64(base, 371u);
    let unit_area_start_slot = rd64(base, 417u);
    let spacing = rd32(base, 429u);
    if (!eq64(ring_start_slot, u64_of(1024u))) {
        return empty_checker_slot(1u);
    }
    if (!eq64(base_slot, u64_of(64u))) {
        return empty_checker_slot(1u);
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
        return empty_checker_slot(1u);
    }
    let physical_block_size = rd32(base, 317u);
    if (physical_block_size < ROOT_RECORD_BYTES || physical_block_size > spacing) {
        return empty_checker_slot(1u);
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
        return empty_checker_slot(1u);
    }
    let ring_slots = shr64(add64_saturating(ring_bytes, u64_of(16383u)), SLOT_SHIFT);
    if (add64_overflows(ring_start_slot, ring_slots) || !eq64(add64(ring_start_slot, ring_slots), unit_area_start_slot)) {
        return empty_checker_slot(1u);
    }
    if ((unit_area_start_slot.x & 63u) != 0u) {
        return empty_checker_slot(1u);
    }
    var c = empty_checker_slot(0u);
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
    return c;
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

// 每盘两槽的读法（image::system_configuration_slot_readings）：槽 1 的偏移按这块盘槽 0 记的槽距，不可用时按第一块槽 0 可用的盘的，
// 全池都不可用时按 4096
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
        var spacing = 4096u;
        if (slot_readings[d * 2u].reading == 0u) {
            spacing = slot_readings[d * 2u].spacing;
        } else if (first_spacing != NONE) {
            spacing = first_spacing;
        }
        slot_readings[d * 2u + 1u] = checker_read_system_configuration_slot(hdr(12u + d), spacing);
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
            sws(R_ROOTS + root_count * 2u + 1u, region);
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
    if (!eq64(rd64(base, 287u), fsid_low)) {
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

// 一块盘上的记录按计数器排进草稿区：版本、计数器 lo/hi、实例、环槽 lo/hi
fn checker_insert_record(version: u32, counter: vec2<u32>, instance: u32, ring_slot: vec2<u32>) {
    var position = 0u;
    while (position < record_count) {
        let existing = vec2<u32>(sw(R_RECORDS + position * RECORD_WORDS + 1u), sw(R_RECORDS + position * RECORD_WORDS + 2u));
        if (eq64(existing, counter)) {
            return;
        }
        if (lt64(counter, existing)) {
            break;
        }
        position = position + 1u;
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
            violate(INVARIANT_I_8_6);
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
        if (r.back_chain != expected) {
            violate(INVARIANT_I_8_6);
        }
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
            for (var k = previous_index + 1u; k < i; k = k + 1u) {
                if (checker_record_at(k).instance == r.instance) {
                    violate(INVARIANT_I_8_7);
                    break;
                }
            }
            continue;
        }
        if (!lt64(previous.transaction, r.transaction)) {
            violate(INVARIANT_I_8_7);
        }
    }
}

// I-8.8：提交标记字节只许 0 / 1；一组（同实例同非 0 事务号）带标记的至多一条且在组末；一条都不带时本实例之后不许再有记录
fn judge_commit_markers_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        if (r.commit_byte > 1u) {
            violate(INVARIANT_I_8_8);
        }
        if (is_zero64(r.transaction)) {
            continue;
        }
        // 只在组的第一条上判一次
        var is_first = true;
        for (var j = 0u; j < i; j = j + 1u) {
            let earlier = checker_record_at(j);
            if (earlier.instance == r.instance && eq64(earlier.transaction, r.transaction)) {
                is_first = false;
                break;
            }
        }
        if (!is_first) {
            continue;
        }
        var last_index = i;
        var marked = 0u;
        var marked_at_last = false;
        for (var j = i; j < record_count; j = j + 1u) {
            let member = checker_record_at(j);
            if (member.instance == r.instance && eq64(member.transaction, r.transaction)) {
                last_index = j;
            }
        }
        for (var j = i; j <= last_index; j = j + 1u) {
            let member = checker_record_at(j);
            if (member.instance == r.instance && eq64(member.transaction, r.transaction) && member.commit_byte == 1u) {
                marked = marked + 1u;
                marked_at_last = j == last_index;
            }
        }
        if (marked >= 2u || (marked == 1u && !marked_at_last)) {
            violate(INVARIANT_I_8_8);
        }
        if (marked == 0u) {
            for (var j = last_index + 1u; j < record_count; j = j + 1u) {
                if (checker_record_at(j).instance == r.instance) {
                    violate(INVARIANT_I_8_8);
                    break;
                }
            }
        }
    }
}

// I-8.9：记录标志其余位、序号 0、组内跳号、不从 1 起、多于一条末条、末条之后还有记录、没有末条
fn judge_publish_ordinals_of_device() {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        if ((r.flags & 0xFEu) != 0u) {
            violate(INVARIANT_I_8_9);
        }
        if (r.ordinal == 0u) {
            violate(INVARIANT_I_8_9);
        }
        // 组 = 同实例同 txg：只在组的第一条上判
        var is_first = true;
        for (var j = 0u; j < i; j = j + 1u) {
            let earlier = checker_record_at(j);
            if (earlier.instance == r.instance && eq64(earlier.txg, r.txg)) {
                is_first = false;
                break;
            }
        }
        if (!is_first) {
            continue;
        }
        var last_index = i;
        var flagged = 0u;
        var flagged_at_last = false;
        for (var j = i; j < record_count; j = j + 1u) {
            let member = checker_record_at(j);
            if (member.instance == r.instance && eq64(member.txg, r.txg)) {
                last_index = j;
            }
        }
        for (var j = i + 1u; j <= last_index; j = j + 1u) {
            let member = checker_record_at(j);
            if (!(member.instance == r.instance && eq64(member.txg, r.txg))) {
                continue;
            }
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
        if (!is_zero64(r.counter)) {
            let before = checker_find_record_by_counter(sub64(r.counter, u64_of(1u)));
            if (before != NONE && r.ordinal != 1u) {
                violate(INVARIANT_I_8_9);
            }
        }
        for (var j = i; j <= last_index; j = j + 1u) {
            let member = checker_record_at(j);
            if (member.instance == r.instance && eq64(member.txg, r.txg) && (member.flags & 1u) != 0u) {
                flagged = flagged + 1u;
                flagged_at_last = j == last_index;
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
            if (!(rd32(r.base, o) < rd32(r.base, o + 14u))) {
                violate(INVARIANT_I_2_5);
            }
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

fn w_judge(invariant: u32, holds: bool) {
    if (w_judging && !holds) {
        violate(invariant);
    }
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
    }
}

fn w_begin_version() {
    w_in_version_count[w_context] = 0u;
    w_parent_valid = false;
}

// 引用集合：(盘, 起点槽, 跨度) 去重记进；位置项校验和记进主走读的表或收割表
fn w_note_reference(device: u32, slot: vec2<u32>, checksum: u32, span: u32) {
    let base = c_references_base();
    let count = w_reference_count[w_context];
    var present = false;
    for (var i = 0u; i < count; i = i + 1u) {
        if (sw(base + i * C_REFERENCE_WORDS) == device && sw(base + i * C_REFERENCE_WORDS + 1u) == slot.x
            && sw(base + i * C_REFERENCE_WORDS + 2u) == slot.y && sw(base + i * C_REFERENCE_WORDS + 3u) == span) {
            present = true;
            break;
        }
    }
    if (!present) {
        if (count >= C_MAX_REFERENCES) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (1u << 8u);
        } else {
            sws(base + count * C_REFERENCE_WORDS, device);
            sws(base + count * C_REFERENCE_WORDS + 1u, slot.x);
            sws(base + count * C_REFERENCE_WORDS + 2u, slot.y);
            sws(base + count * C_REFERENCE_WORDS + 3u, span);
            sws(base + count * C_REFERENCE_WORDS + 4u, 0u);
            w_reference_count[w_context] = count + 1u;
        }
    }
    var checksum_base = C_CHECKSUMS;
    var checksum_count = w_checksum_count;
    if (w_context != 0u) {
        if (!w_harvesting) {
            return;
        }
        checksum_base = C_HARVEST;
        checksum_count = w_harvest_count;
    }
    var known = false;
    for (var i = 0u; i < checksum_count; i = i + 1u) {
        if (sw(checksum_base + i * C_CHECKSUM_WORDS) == device && sw(checksum_base + i * C_CHECKSUM_WORDS + 1u) == slot.x
            && sw(checksum_base + i * C_CHECKSUM_WORDS + 2u) == slot.y && sw(checksum_base + i * C_CHECKSUM_WORDS + 3u) == checksum) {
            known = true;
            break;
        }
    }
    if (!known) {
        if (checksum_count >= C_MAX_CHECKSUMS) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (2u << 8u);
            return;
        }
        sws(checksum_base + checksum_count * C_CHECKSUM_WORDS, device);
        sws(checksum_base + checksum_count * C_CHECKSUM_WORDS + 1u, slot.x);
        sws(checksum_base + checksum_count * C_CHECKSUM_WORDS + 2u, slot.y);
        sws(checksum_base + checksum_count * C_CHECKSUM_WORDS + 3u, checksum);
        if (w_context == 0u) {
            w_checksum_count = checksum_count + 1u;
        } else {
            w_harvest_count = checksum_count + 1u;
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

// visited_units.insert：已有交回 false
fn w_visit(device: u32, slot: vec2<u32>) -> bool {
    if (w_visited_contains(device, slot)) {
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

// 走读跟着一条指针往下：两条位置条目记进引用集合，判同一版第二次引用，记到父单元下面
fn w_note_pointer_references(p: Ptr, span: u32) {
    w_note_reference(p.device0, p.slot0, p.sum0, span);
    w_judge_placement_in_version(p.device0, p.slot0);
    w_record_below(p.device0, p.slot0);
    w_note_reference(p.device1, p.slot1, p.sum1, span);
    if (!(p.device1 == p.device0 && eq64(p.slot1, p.slot0))) {
        w_judge_placement_in_version(p.device1, p.slot1);
        w_record_below(p.device1, p.slot1);
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

// image::read_referenced_unit：两条位置条目都读、都判 I-2.1，交回第一份对得上的
fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
    var first = no_slice();
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
    }
    return first;
}

// I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免
fn w_judge_location_order(p: Ptr) {
    if (p.all_zero) {
        return;
    }
    w_judge(INVARIANT_I_2_5, p.device0 < p.device1);
}

// I-2.4 的指针那一半：头部 MAC / nonce 全 0
fn w_judge_pointer_head(p: Ptr) {
    w_judge(INVARIANT_I_2_4, p.mac_nonce_zero);
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

// I-1.2 / I-4.2：头里的出生身份与引用它的指针记的相同；按已发布谓词判成已发布
fn w_judge_birth_identity(s: Slice, p: Ptr) {
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
};

// walk.rs read_index_node：读一个码 2 节点并判头、出生身份、树 ID（I-1.3）与 key（I-1.1）
// key_range_reading：0 贴紧首末条目、1 多层树的子树覆盖（叶贴紧、内部只判分隔 key）、2 按位置（只判分隔 key）
fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_reading: u32, registered: bool, fsid_low: vec2<u32>) -> ReadNode {
    var out: ReadNode;
    out.ok = false;
    out.already_walked = false;
    out.n = bad_node();
    if (p.all_zero) {
        return out;
    }
    w_judge_location_order(p);
    w_judge_pointer_head(p);
    w_note_pointer_references(p, 1u);
    let s = w_read_referenced_unit(p, NODE_BYTES);
    if (s.version == NONE) {
        w_failures = w_failures + 1u;
        return out;
    }
    if (!w_visit(p.device0, p.slot0)) {
        out.already_walked = true;
        w_merge_below(p.device0, p.slot0);
        return out;
    }
    if (!w_judge_unit_header(s, 2u, fsid_low)) {
        return out;
    }
    w_judge_birth_identity(s, p);
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

// walk.rs walk_instance_table_chain（挂载根那一张另判行：I-3.8 的行那一半，行记进 C_INSTANCE_ROWS）
fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>) {
    var pointer = first;
    var page_index = vec2<u32>(0u, 0u);
    let parent_valid = w_parent_valid;
    let parent_device = w_parent_device;
    let parent_slot = w_parent_slot;
    var rows_unique = true;
    var rows_below = true;
    var chain_last = true;
    var pages = 0u;
    for (var page = 0u; page < 256u; page = page + 1u) {
        w_judge_location_order(pointer);
        w_judge_pointer_head(pointer);
        w_note_pointer_references(pointer, 2u);
        let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);
        if (s.version == NONE) {
            w_failures = w_failures + 1u;
            break;
        }
        if (!w_visit(pointer.device0, pointer.slot0)) {
            w_merge_below(pointer.device0, pointer.slot0);
            break;
        }
        w_parent_valid = true;
        w_parent_device = pointer.device0;
        w_parent_slot = pointer.slot0;
        let packed = w_judge_packed_container(s, 4u, fsid_low);
        if (!packed.ok) {
            break;
        }
        let p = packed.p;
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
        w_judge(INVARIANT_I_3_8, chain != 2u);
        // 行那一半的账
        if (judge_rows) {
            var chain_last_in_this_page = false;
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
                    chain_last_in_this_page = false;
                } else if (kind == 1u) {
                    chain_last_in_this_page = r + 1u == p.record_count;
                } else {
                    chain_last_in_this_page = false;
                }
            }
            chain_last = chain_last && chain_last_in_this_page;
            pages = pages + 1u;
        }
        if (chain != 1u) {
            break;
        }
        pointer = parse_node_pointer(p.base, packed_record_offset(p, p.record_count - 1u) + 2u);
        page_index = add64(page_index, u64_of(1u));
    }
    w_parent_valid = parent_valid;
    w_parent_device = parent_device;
    w_parent_slot = parent_slot;
    if (judge_rows && pages > 0u) {
        w_judge(INVARIANT_I_3_8, rows_unique && rows_below && chain_last);
    }
}

// 树表条目里 I-9.16 两道（walk.rs judge_tree_table_entries_ordering）
fn w_judge_tree_table_entries_ordering(n: Node) {
    for (var e = 1u; e < n.entry_count; e = e + 1u) {
        if (!lt64(rd64(n.base, entry_offset(n, e - 1u)), rd64(n.base, entry_offset(n, e)))) {
            w_judge(INVARIANT_I_9_16, false);
            return;
        }
    }
    if (n.entry_width < 12u) {
        w_judge(INVARIANT_I_9_16, true);
        return;
    }
    var previous_tree = vec2<u32>(0u, 0u);
    var have_previous = false;
    for (var k = 0u; k < 7u; k = k + 1u) {
        var kind = k + 1u;
        if (k >= 4u) {
            kind = k + 2u;
        }
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

// 数据单元（walk.rs walk_extent_data_pointer）
fn w_walk_extent_data_pointer(base: u32, o: u32, inode: vec2<u32>, offset: vec2<u32>, tree: vec2<u32>, fsid_low: vec2<u32>) {
    let p = parse_data_pointer(base, o);
    w_judge_location_order(p);
    w_judge_pointer_head(p);
    w_note_pointer_references(p, 2u);
    let s = w_read_referenced_unit(p, DATA_UNIT_BYTES);
    if (s.version == NONE) {
        w_failures = w_failures + 1u;
        return;
    }
    if (!w_visit(p.device0, p.slot0)) {
        w_merge_below(p.device0, p.slot0);
        return;
    }
    if (!w_judge_unit_header(s, 1u, fsid_low)) {
        return;
    }
    w_judge_birth_identity(s, p);
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
            w_parent_device = pending_pointer.device0;
            w_parent_slot = pending_pointer.slot0;
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
                w_parent_device = pending_pointer.device0;
                w_parent_slot = pending_pointer.slot0;
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
            w_parent_device = pending_pointer.device0;
            w_parent_slot = pending_pointer.slot0;
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
            let read = w_read_index_node(pending_pointer, tree, 3u, 2u, true, fsid_low);
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
                w_parent_device = pending_pointer.device0;
                w_parent_slot = pending_pointer.slot0;
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
                        w_parent_device = pending_pointer.device0;
                        w_parent_slot = pending_pointer.slot0;
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
            w_parent_device = pending_pointer.device0;
            w_parent_slot = pending_pointer.slot0;
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
                let already = w_visited_contains(pending_pointer.device0, pending_pointer.slot0);
                let read = w_read_index_node(pending_pointer, tree, schema, 1u, registered, fsid_low);
                if (!read.ok) {
                    if (already) {
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
                            w_parent_device = pending_pointer.device0;
                            w_parent_slot = pending_pointer.slot0;
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

// ── inode 树（walk_inode_root / walk_inode_internal_node / walk_inode_leaf_container）：根层级 1，孩子只接叶容器；类型段 0 的孩子这一版判不了
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
        var length = NODE_BYTES;
        var span = 1u;
        if (record_type == 2u) {
            length = DATA_UNIT_BYTES;
            span = 2u;
        }
        w_note_pointer_references(child, span);
        let s = w_read_referenced_unit(child, length);
        if (s.version == NONE) {
            w_failures = w_failures + 1u;
            continue;
        }
        if (record_type == 2u) {
            let header_identity_holds = rd8(s.base, 6u) == 3u && eq64(rd64(s.base, 43u), identity_birth_tree) && rd16(s.base, 51u) == 2u
                && eq64(rd64(s.base, 53u), identity_container) && eq64(rd64(s.base, 61u), identity_birth) && eq64(child.birth_tree, identity_birth_tree);
            w_judge(INVARIANT_I_9_2, header_identity_holds);
            // 叶容器
            if (!w_visit(child.device0, child.slot0)) {
                w_merge_below(child.device0, child.slot0);
                if (w_context == 0u) {
                    w_cached_leaves_below(child.device0, child.slot0, leaves_before);
                }
                continue;
            }
            w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
            w_judge_birth_identity(s, child);
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
                w_judge(INVARIANT_I_9_7, all_zero(p.base, ro + 108u, 32u));
                w_judge(INVARIANT_I_9_4, le64(p.container, inode));
                let size = rd64(p.base, ro + 40u);
                let blocks = rd64(p.base, ro + 48u);
                let logical_blocks = shr64(add64_saturating(size, u64_of(511u)), 9u);
                w_judge(INVARIANT_I_9_15, eq64(blocks, logical_blocks));
                if (w_context == 0u) {
                    // inode → 对象出生代（后写的覆盖先写的）
                    let birth = rd64(p.base, ro + 8u);
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
                    sws(row, child.device0);
                    sws(row + 1u, child.slot0.x);
                    sws(row + 2u, child.slot0.y);
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
        } else if (record_type == 0u) {
            // 类型段 0 的内部节点：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了
            undecidable = UNDECIDABLE_TOO_DEEP;
            return;
        } else {
            w_judge(INVARIANT_I_9_2, false);
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
    // I-9.4 第二、三句：沿叶序相邻两片
    for (var i = leaves_before + 1u; i < w_leaf_count; i = i + 1u) {
        let left = C_LEAVES + (i - 1u) * C_LEAF_WORDS;
        let right = C_LEAVES + i * C_LEAF_WORDS;
        let left_container = vec2<u32>(sw(left + 3u), sw(left + 4u));
        let left_largest = vec2<u32>(sw(left + 7u), sw(left + 8u));
        let right_container = vec2<u32>(sw(right + 3u), sw(right + 4u));
        w_judge(INVARIANT_I_9_4, lt64(left_container, right_container));
        w_judge(INVARIANT_I_9_4, lt64(left_largest, right_container));
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
    // key 形态：1 extent、2 inode、3 分配记录、4 记账、5 映射；别的种类不走
    var schema = NONE;
    var registered = false;
    var field_table_width = 0u;
    if (kind == TREE_KIND_EXTENT) { schema = 3u; registered = true; field_table_width = 112u; }
    else if (kind == TREE_KIND_INODE) { schema = 2u; registered = true; field_table_width = 120u; }
    else if (kind == TREE_KIND_ALLOCATION) { schema = 4u; registered = true; field_table_width = 20u; }
    else if (kind == TREE_KIND_ACCOUNTING) { schema = 0u; registered = true; field_table_width = 34u; }
    else if (kind == 5u) { schema = 1u; }
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
        w_walk_inode_root(read.n, tree, pointer.device0, pointer.slot0, fsid_low);
    }
}

// ── 一版的树表与中央映射树根（walk_tree_table_and_central_mapping_root）
fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer: Ptr, fsid_low: vec2<u32>) {
    let read = w_read_index_node(tree_table_pointer, vec2<u32>(0u, 0u), 2u, 0u, false, fsid_low);
    if (!read.ok) {
        return;
    }
    let tt = read.n;
    w_judge_tree_table_entries_ordering(tt);
    let parent_valid = w_parent_valid;
    let parent_device = w_parent_device;
    let parent_slot = w_parent_slot;
    w_parent_valid = true;
    w_parent_device = tree_table_pointer.device0;
    w_parent_slot = tree_table_pointer.slot0;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        w_walk_tree_table_entry(tt, e, fsid_low);
    }
    w_parent_valid = parent_valid;
    w_parent_device = parent_device;
    w_parent_slot = parent_slot;
    let mapping = w_walk_code_two_tree(mapping_root_pointer, mapping_root_pointer.birth_tree, 1u, fsid_low);
    w_walk_central_mapping_entries(mapping.entries, fsid_low);
}

// ── 一条根（walk_root）
// 走一遍：一条根（WALK_KIND_ROOT：实例表与分配记录树、树表与映射都取自这条根记录），或一版由记录施加出来、根槽从没落盘的版本
// （WALK_KIND_VERSION：树表与映射取自那条记录，实例表与分配记录树取自它施加在其上的那条根；没有那条根就不走这两样）。
// 全内核只许有一处调它（walk_and_hand_off 的循环里）：显卡上的程序没有真的函数调用，驱动的编译器把每个调用点各展开一份，
// 这一整棵展开之后三十多万记号。原来五处走根、一处走版本各展开一份时，走读内核展开后 242 万记号，本机（驱动 595 版）冷编 523 秒、
// 常驻 4.1 GiB，第二台（驱动 580 版）在 48G 里编不完；只留一处之后本机 18 秒、0.8 GiB。
// 往这个内核里加走读的地方时，加一个阶段、不加调用点。
const WALK_KIND_ROOT: u32 = 0u;
const WALK_KIND_VERSION: u32 = 1u;

fn w_walk_one(kind: u32, record_base: u32, applied_on_base: u32, is_newest: bool, fsid_low: vec2<u32>) {
    w_begin_version();
    w_accounting_recording = is_newest;
    var carrier_base = record_base;
    var tree_table_pointer_offset = 36u;
    var mapping_root_pointer_offset = 256u;
    if (kind == WALK_KIND_VERSION) {
        carrier_base = applied_on_base;
        tree_table_pointer_offset = 99u;
        mapping_root_pointer_offset = 185u;
    }
    if (carrier_base != NONE) {
        w_walk_instance_table_chain(parse_node_pointer(carrier_base, 170u), is_newest, fsid_low);
        let allocation_root = parse_node_pointer(carrier_base, 342u);
        if (!allocation_root.all_zero) {
            w_walk_allocation_record_tree(allocation_root, vec2<u32>(0u, 0u), fsid_low);
        }
    }
    w_walk_tree_table_and_mapping(parse_node_pointer(record_base, tree_table_pointer_offset), parse_node_pointer(record_base, mapping_root_pointer_offset), fsid_low);
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

// 走读的六个阶段，按次序；每个阶段里要走的每一处都经 walk_and_hand_off 循环里那一个调用点走
const WALK_PHASE_NEWEST_ROOT: u32 = 0u;
const WALK_PHASE_CANDIDATE_ROOTS: u32 = 1u;
const WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS: u32 = 2u;
const WALK_PHASE_ABANDONED_ROOTS: u32 = 3u;
const WALK_PHASE_HARVEST_ROOTS_OUTSIDE_THE_CANDIDATES: u32 = 4u;
const WALK_PHASE_ROOTS_BELOW_THE_FLOOR: u32 = 5u;
const WALK_PHASE_DONE: u32 = 6u;
// 交回的值是结论里「走到了第几步」：3 走读那一批没判完（中途判不了），4 走读那一批判完、进了收尾
const WALK_REACHED_THE_ROOT_RING: u32 = 3u;
const WALK_REACHED_THE_HAND_OFF: u32 = 4u;

// 走读那一批连同收尾：择最新根、候选根、由记录施加出来的版本、被抛弃根，判走读之后那几条；
// 收尾是候选集之外的根整遍走一次收位置项校验和（第三批判隔离的记录要），F 之下、没被抛弃的根各走一遍，
// 它们引用的、候选版本没走到的槽记进交接段（第三批算 defer 那一格要），再把计数与几何写进交接段。
// 写成按阶段推进的一个循环：每一圈先找这一阶段下一处要走的（找不到就做完这一阶段收尾的事、转下一阶段），走，再做走完之后的事。
fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
    let device_count = hdr(3u);
    w_harvesting = false;
    c_version_count = 0u;
    c_candidate_count = 0u;
    // 择根：(txg, 实例) 最大，相等取后面的（max_by_key 取最后一个最大者）
    var newest = 0u;
    var newest_root = checker_root_of(0u);
    for (var i = 1u; i < root_count; i = i + 1u) {
        let r = checker_root_of(i);
        if (!root_key_less(r.txg, r.instance, newest_root.txg, newest_root.instance)) {
            newest = i;
            newest_root = r;
        }
    }
    c_newest_index = newest;
    w_mount_instance = newest_root.instance;
    w_mount_txg = newest_root.txg;
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
        var walk_applied_on = NONE;
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
                        // 按 (实例, txg) 去重、升序插入；同键取计数器小的那一条（先到的）
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
                // 它施加在其上那一版的根：同实例、txg 比它小、txg 最大的那一条
                var applied_on = NONE;
                var applied_on_txg = vec2<u32>(0u, 0u);
                for (var k = 0u; k < root_count; k = k + 1u) {
                    let root = checker_root_of(k);
                    if (root.instance == r.instance && lt64(root.txg, r.txg)) {
                        if (applied_on == NONE || lt64(applied_on_txg, root.txg)) {
                            applied_on = root.base;
                            applied_on_txg = root.txg;
                        }
                    }
                }
                walk_kind = WALK_KIND_VERSION;
                walk_base = r.base;
                walk_applied_on = applied_on;
                found = true;
            } else {
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
                if (newest_failures > 0u) {
                    violate(INVARIANT_I_7_2);
                }
                // I-9.6：记账里的 inode 号水位 > inode 树内最大 key
                if (w_have_largest_inode) {
                    for (var k = 0u; k < w_accounting_row_count; k = k + 1u) {
                        if (sw(C_ACCOUNTING_ROWS + k * 4u) == 12u && sw(C_ACCOUNTING_ROWS + k * 4u + 1u) == NONE) {
                            let watermark = vec2<u32>(sw(C_ACCOUNTING_ROWS + k * 4u + 2u), sw(C_ACCOUNTING_ROWS + k * 4u + 3u));
                            if (!lt64(w_largest_inode, watermark)) {
                                violate(INVARIANT_I_9_6);
                            }
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
                            if (!eq64(record_birth, unit_birth)) {
                                violate(INVARIANT_I_9_10);
                            }
                        }
                    }
                }
                // I-5.1：同一块盘上不同的引用不许占重叠的槽（按 (槽, 跨度) 排序后相邻两条比 ⟺ 任两条比）
                let reference_count = w_reference_count[0];
                for (var i = 0u; i < reference_count; i = i + 1u) {
                    let device = sw(C_REFERENCES0 + i * C_REFERENCE_WORDS);
                    let slot = vec2<u32>(sw(C_REFERENCES0 + i * C_REFERENCE_WORDS + 1u), sw(C_REFERENCES0 + i * C_REFERENCE_WORDS + 2u));
                    let span = sw(C_REFERENCES0 + i * C_REFERENCE_WORDS + 3u);
                    let end = add64(slot, u64_of(span));
                    for (var j = 0u; j < reference_count; j = j + 1u) {
                        if (j == i || sw(C_REFERENCES0 + j * C_REFERENCE_WORDS) != device) {
                            continue;
                        }
                        let other_slot = vec2<u32>(sw(C_REFERENCES0 + j * C_REFERENCE_WORDS + 1u), sw(C_REFERENCES0 + j * C_REFERENCE_WORDS + 2u));
                        let other_span = sw(C_REFERENCES0 + j * C_REFERENCE_WORDS + 3u);
                        // i 排在 j 前面：(slot, span) 字典序小
                        let i_first = lt64(slot, other_slot) || (eq64(slot, other_slot) && span < other_span);
                        if (i_first && lt64(other_slot, end)) {
                            violate(INVARIANT_I_5_1);
                        }
                    }
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
                let location_count = hdr(6u);
                for (var l = 0u; l < location_count; l = l + 1u) {
                    if (location_word(l, 4u) != LOCATION_UNIT_SLOT) {
                        continue;
                    }
                    let s = read_location(l);
                    if (s.version == NONE || s.length < NODE_BYTES || rd32(s.base, 0u) != MAGIC_SFSU || rd8(s.base, 6u) != 2u) {
                        continue;
                    }
                    if (((s.c3 >> 4u) & 1u) == 0u) {
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
                if (!lt64(highest_seen, watermark)) {
                    violate(INVARIANT_I_7_8);
                }
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
        } else {
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
                sws(H_BASE + H_UNIT_AREA_START_LO, g.unit_area_start_slot.x);
                sws(H_BASE + H_UNIT_AREA_START_HI, g.unit_area_start_slot.y);
                sws(H_BASE + H_REGIONS, g.regions);
                sws(H_BASE + H_REGION0, g.region0);
                sws(H_BASE + H_REGION1, g.region1);
                sws(H_BASE + H_REGION2, g.region2);
                sws(H_BASE + H_FSID0, g.fsid.x);
                sws(H_BASE + H_FSID1, g.fsid.y);
                sws(H_BASE + H_FSID2, g.fsid.z);
                sws(H_BASE + H_FSID3, g.fsid.w);
                phase = WALK_PHASE_DONE;
            }
        }
        if (!found) {
            continue;
        }
        // ② 走：全内核唯一的调用点
        let mismatches_before = w_mismatches;
        let failures_before = w_failures;
        w_walk_one(walk_kind, walk_base, walk_applied_on, walk_is_newest, fsid_low);
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
            if (w_failures > 0u || w_mismatches > 0u) {
                violate(INVARIANT_I_7_4);
            }
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


// ───────────────────────── 入口 ─────────────────────────

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    violated_lo = 0u;
    violated_hi = 0u;
    let out = local * 12u;
    sws(H_BASE + H_VALID, 0u);
    let device_count = hdr(3u);
    var reached = 0u;
    if (device_count > MAX_DEVICES) {
        undecidable = UNDECIDABLE_TOO_MANY_DEVICES;
    } else {
        read_every_system_configuration_slot();
        // 任一槽带这一版读者不收的值：I-7.13 违例，别的不变量在这份镜像上不判
        var any_refused = false;
        var chosen_valid = false;
        var g = empty_checker_slot(2u);
        for (var d = 0u; d < device_count; d = d + 1u) {
            for (var k = 0u; k < 2u; k = k + 1u) {
                if (slot_readings[d * 2u + k].reading == 1u) {
                    any_refused = true;
                }
            }
        }
        if (any_refused) {
            violate(INVARIANT_I_7_13);
            reached = 1u;
        } else {
            // 每盘择世代号大的那一槽（相等取槽 0），几何取第一块择到的盘的
            for (var d = 0u; d < device_count; d = d + 1u) {
                let zero = slot_readings[d * 2u];
                let one = slot_readings[d * 2u + 1u];
                var chosen = empty_checker_slot(2u);
                if (zero.reading == 0u && one.reading == 0u) {
                    if (lt64(zero.generation, one.generation)) {
                        chosen = one;
                    } else {
                        chosen = zero;
                    }
                } else if (zero.reading == 0u) {
                    chosen = zero;
                } else if (one.reading == 0u) {
                    chosen = one;
                }
                if (chosen.reading != 0u) {
                    continue;
                }
                if (!chosen_valid) {
                    g = chosen;
                    chosen_valid = true;
                } else if (!same_fsid(chosen.fsid, g.fsid)) {
                    violate(INVARIANT_I_1_4);
                }
            }
            if (chosen_valid) {
                reached = 2u;
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
                if (!(distinct == min(g.regions, device_count) && heaviest <= pigeonhole && all_in_pool)) {
                    violate(INVARIANT_I_7_6);
                }
                let fsid_low = vec2<u32>(g.fsid.x, g.fsid.y);
                // 自证过、属于本池的槽：I-7.14 本盘设备号；I-7.12 每盘的 F
                checker_collect_roots(g);
                for (var d = 0u; d < device_count; d = d + 1u) {
                    let device = hdr(12u + d);
                    var highest_floor = vec2<u32>(0u, 0u);
                    var have_slot = false;
                    for (var k = 0u; k < 2u; k = k + 1u) {
                        let slot = slot_readings[d * 2u + k];
                        if (slot.reading != 0u || !same_fsid(slot.fsid, g.fsid)) {
                            continue;
                        }
                        if (slot.this_device != device) {
                            violate(INVARIANT_I_7_14);
                        }
                        if (!have_slot || lt64(highest_floor, slot.floor)) {
                            highest_floor = slot.floor;
                        }
                        have_slot = true;
                    }
                    if (!have_slot) {
                        continue;
                    }
                    for (var i = 0u; i < root_count; i = i + 1u) {
                        if (checker_region_device(g, checker_root_region(i)) != device) {
                            continue;
                        }
                        if (lt64(highest_floor, checker_root_of(i).floor)) {
                            violate(INVARIANT_I_7_12);
                        }
                    }
                }
                // I-7.7：每盘两槽里本池的槽中 journal 实例代号的最大者；有一块盘没有本池的槽就不判
                var every_device_has_a_slot = true;
                var pool_highest = 0u;
                var pool_lowest = NONE;
                for (var d = 0u; d < device_count; d = d + 1u) {
                    var highest = NONE;
                    for (var k = 0u; k < 2u; k = k + 1u) {
                        let slot = slot_readings[d * 2u + k];
                        if (slot.reading == 0u && same_fsid(slot.fsid, g.fsid)) {
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
                if (every_device_has_a_slot) {
                    if (some_instance_carrier_exceeds(pool_highest, g)) {
                        violate(INVARIANT_I_7_7);
                    }
                    if (pool_lowest < pool_highest && some_instance_carrier_exceeds(pool_lowest, g)) {
                        violate(INVARIANT_I_7_7);
                    }
                }
                // journal 四条与点名项的位置条目次序，逐盘
                for (var d = 0u; d < device_count; d = d + 1u) {
                    checker_scan_journal_of_device(hdr(12u + d), fsid_low);
                    judge_journal_back_chain_of_device();
                    judge_named_entry_location_order_of_device();
                    judge_transaction_numbers_of_device();
                    judge_commit_markers_of_device();
                    judge_publish_ordinals_of_device();
                }
                // I-7.1 / I-7.3
                if (root_count == 0u) {
                    violate(INVARIANT_I_7_1);
                } else {
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
                    if (!(earlier_exists || every_genesis)) {
                        violate(INVARIANT_I_7_3);
                    }
                    reached = walk_and_hand_off(g, fsid_low);
                    if (reached == WALK_REACHED_THE_HAND_OFF && undecidable == 0u) {
                        sws(H_BASE + H_VALID, 1u);
                    }
                }
            }
        }
    }
    verdicts[out + 8u] = violated_lo;
    verdicts[out + 9u] = violated_hi;
    verdicts[out + 10u] = reached;
    verdicts[out + 11u] = undecidable;
}
