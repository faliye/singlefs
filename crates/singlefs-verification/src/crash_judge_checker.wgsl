// GPU 判器的池级 checker 走读内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：按 singlefs-checker 自己的读法
// 判第一批（系统配置、根环、journal）与第二批（走读那 29 条），连同里程碑三二·1 那一批里不在扫描内核的十五条（I-1.12、I-8.10、I-8.11、
// I-4.9、I-9.25、I-3.13、I-9.26、I-7.16、I-7.17、I-8.12、I-7.15、I-4.1、I-8.1、I-7.5、I-8.5）与里程碑三·3 墓碑树、谱系树条目身份那两条
// （I-9.23、I-9.17）、里程碑三收尾择系统配置时逐盘判的 I-1.5，违例位写进每个状态结论的违例位那几个字
// （IMPLEMENTED_INVARIANTS 次序），之后一字是走到的批次、再一字是判不了的原因；走完把扫描内核要的计数与几何写进草稿区的交接段
// （第三批与 I-3.12 在 crash_judge_checker_scan.wgsl）。


// ───────────────────────── checker 读系统配置槽 ─────────────────────────

// 一槽读下来：0 可择、1 带这一版读者不收的值（I-7.13 违例；R 不等于 3、S 越界也在这里，C663）、2 不自证（读不到、magic 或整槽校验和不对）、
// 3 自证过而不可择（incompat 位不认得；I-8.5 把它数进自证过的槽）
const SLOT_USABLE: u32 = 0u;
const SLOT_CARRIES_A_REFUSED_VALUE: u32 = 1u;
const SLOT_NOT_SELF_VERIFIED: u32 = 2u;
const SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE: u32 = 3u;

// 自举头「写入者身份 20」里实现标识那 16 字节：紧跟 fsid（偏移 102、16 字节）。lib.rs 的 SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER_OFFSET
// 与 _BYTES，D22（单元原子性怎么合成） 已定项 9 的字段表
const SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER_OFFSET: u32 = 118u;
const SYSTEM_CONFIGURATION_WRITER_IMPLEMENTATION_IDENTIFIER_BYTES: u32 = 16u;

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
    // 这一槽 4096 字节在 tables 里的起点（字下标）：I-2.4 判字段表之后那一段全 0 时读（可择的槽才有，别的是 0）
    slot_bytes_base: u32,
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
    c.slot_bytes_base = 0u;
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
    if (!incompat_bits_are_recognized(base)) {
        return empty_checker_slot(SLOT_SELF_VERIFIED_BUT_NOT_CHOOSABLE);
    }
    if (rd8(base, 219u) != 0u) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    // geometry_of：R 与第一版常量 3 不等（0、1、2 与大于 3 都算）、S 不在 [4, 16] 里都是读者不收的值
    let regions = rd8(base, 361u);
    if (regions != 3u) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    let slots_per_region = rd8(base, 362u);
    if (slots_per_region < 4u || slots_per_region > 16u) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
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
    // 根环区域步长 P（363）、chunk（367）等于第一版常量，S × 槽距 ≤ P × chunk（geometry_of 的 RootRingPrimeStepNotTheFirstVersionConstant、
    // RootRingChunkBytesNotTheFirstVersionConstant、RootRingRegionLongerThanTheRegionStride；D22（单元原子性怎么合成） 已定项 16 第 1 句）。
    // S × 槽距在 u32 里会溢出，按 槽距 ≤ ⌊P × chunk ÷ S⌋ 判（S 在 [4, 16] 里，上面判过）。区域按常量放（checker_collect_roots）
    if (rd32(base, 363u) != ROOT_RING_PRIME_STEP) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    if (rd32(base, 367u) != ROOT_RING_CHUNK_BYTES) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    if (spacing > ROOT_RING_REGION_STRIDE / slots_per_region) {
        return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
    }
    // 环长（image::journal_ring_bytes_lie_in_the_supported_range）：在飞上限 ⌊环长 ÷ 4096 ÷ 3⌋ ≥ 1 且装得进系统配置里它那 4 字节一格
    // （高 32 位为 0），环末端（起点 × 16384 + 环长，溢出算越界）≤ 单元区起点 × 16384（溢出算成立）
    let in_flight = divmod64(shr64(ring_bytes, 12u), u64_of(3u)).quotient;
    var ring_holds = !is_zero64(in_flight) && in_flight.y == 0u;
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
    c.slot_bytes_base = base;
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
// 读到带不收之值的那一档按偏移升序交回第一个；读到可用的一档先判 I-8.1 三个字段等式（实现那张不收之值表的最后三格），
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

// 逐盘自证过的系统配置槽数（image::system_configuration_redundancy_report 数的那一样：magic 与整槽校验和都对的槽，不分 fsid，自证过而不可择的、
// 带这一版读者不收之值的也算），第 d 块盘占 SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE × d 起那几位：两槽全废盘面观察者在判段拿它对基镜像比。
// 盘数越过 MAX_DEVICES（判不了，槽没读）交 0
const_assert MAX_DEVICES * SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE <= 32u;
fn self_verified_system_configuration_slots_of_each_device(device_count: u32) -> u32 {
    if (device_count > MAX_DEVICES) {
        return 0u;
    }
    var slots_of_each_device = 0u;
    for (var device_position = 0u; device_position < device_count; device_position = device_position + 1u) {
        var self_verified_slots = 0u;
        for (var slot = 0u; slot < 2u; slot = slot + 1u) {
            if (slot_readings[device_position * 2u + slot].reading != SLOT_NOT_SELF_VERIFIED) {
                self_verified_slots = self_verified_slots + 1u;
            }
        }
        slots_of_each_device = slots_of_each_device | (self_verified_slots << (SELF_VERIFIED_SLOT_COUNT_BITS_OF_A_DEVICE * device_position));
    }
    return slots_of_each_device;
}

// ───────────────────────── 根环（checker 的读法）─────────────────────────

// 根环每一槽的三种样子（ROOT_RING_SLOT_SELF_VERIFIED、ROOT_RING_SLOT_ALL_ZERO、ROOT_RING_SLOT_DOES_NOT_READ_BACK）定义在池级 checker 共用那一半
// （crash_judge_checker_common.wgsl）：扫描内核 I-7.9 读交接过去的那一份（H_ROOT_RING_SLOT_READINGS）也认它们。
// 下标 区域 × S + 槽：(样子、自证过的根的 txg lo、hi)。checker_collect_roots 读槽时逐槽记下；R ≤ 3、S ≤ 16（geometry_of 判过），至多 MAX_ROOTS 槽
var<private> root_ring_slot_readings: array<vec3<u32>, MAX_ROOTS>;

// image::valid_roots：R × S 个槽逐个按 check_root_slot 判（magic、整槽校验和、fsid、flags）。每一槽的样子另记进 root_ring_slot_readings
// （walk.rs 的 root_ring_slot_readings：CPU 的读者交 None 的槽——盘不在池里、越过盘尾——GPU 判表里交没有版本的切片，照
// checker_some_root_slot_is_unreadable 的条件现判；没有版本的切片在池里、盘内是从没写过的槽，CPU 读回全零）
fn checker_collect_roots(g: CheckerSlot) {
    root_count = 0u;
    for (var region = 0u; region < g.regions; region = region + 1u) {
        let device = checker_region_device(g, region);
        for (var slot = 0u; slot < g.slots_per_region; slot = slot + 1u) {
            let offset = add64(slot_to_bytes(g.base_slot), u64_of(region * ROOT_RING_REGION_STRIDE + slot * g.spacing));
            let s = read_at(device, offset, g.physical_block_size);
            let candidate = parse_root(s, g.fsid);
            let root_slot_bytes = u64_of(g.physical_block_size);
            var reading = ROOT_RING_SLOT_ALL_ZERO;
            if (!device_is_in_the_pool(device) || add64_overflows(offset, root_slot_bytes) || lt64(device_bytes(), add64(offset, root_slot_bytes))) {
                reading = ROOT_RING_SLOT_DOES_NOT_READ_BACK;
            } else if (candidate.valid) {
                reading = ROOT_RING_SLOT_SELF_VERIFIED;
            } else if (s.version != NONE && !all_zero(s.base, 0u, s.length)) {
                reading = ROOT_RING_SLOT_DOES_NOT_READ_BACK;
            }
            root_ring_slot_readings[region * g.slots_per_region + slot] = vec3<u32>(reading, candidate.txg.x, candidate.txg.y);
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
    return checker_record_valid_past_the_named_entry_flags(s, fsid_low) && journal_record_named_entry_flags_are_zero(s.base);
}

// 每个点名项的 flags 那 1 字节（点名项 56 字节里的偏移 55）是 0：池级 checker 的记录读者判 NonZeroFlags 拒的那一关，恢复的记录读者
// （`crates/singlefs-core/src/journal.rs` 的 NamedUnit::read_from）跳过这一字节不看
fn journal_record_named_entry_flags_are_zero(base: u32) -> bool {
    let named = rd32(base, 12u);
    for (var i = 0u; i < named; i = i + 1u) {
        if (rd8(base, JOURNAL_HEADER_BYTES + i * JOURNAL_NAMED_ENTRY_BYTES + 55u) != 0u) {
            return false;
        }
    }
    return true;
}

// lib::check_journal_record 判点名项 flags 之前的那几关：magic、整条校验和、类型 1、长度 4096、fsid、卸载记号位 0 之外全 0、载荷校验和。
// 过了这几关、只因点名项 flags 非 0 被池级 checker 的记录读者拒的记录，恢复的读者照收（walk.rs 的
// self_verified_journal_record_read_past_the_named_entry_flags，C897 那一行后半收录口径反方向的那一处）：I-8.13 判它的落点、恢复视图与
// 重放读的那一份环收它，判不变量那一份逐盘表不收（checker_scan_journal_of_device）
fn checker_record_valid_past_the_named_entry_flags(s: Slice, fsid_low: vec2<u32>) -> bool {
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
    return s.c1 == 1u;
}

// 新根段里映射根指针的段内偏移：树表指针在段首、86 字节，映射根指针紧跟其后（walk.rs 的 NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER；
// 扫描内核那一份是 G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER）
const W_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: u32 = 86u;

// 新根段里偏移 o 起那条指针的头部是第一版的取值（walk.rs 的 pointer_head_fields_take_the_first_version_values）：MAC / nonce 全 0、
// 算法类型 0、压缩算法码 0、压后长度 0（pointer_encryption_and_compression_fields_are_the_first_version），extent 偏移 0
fn journal_new_root_segment_pointer_head_is_the_first_version(base: u32, o: u32) -> bool {
    return pointer_encryption_and_compression_fields_are_the_first_version(base, o) && pointer_extent_offset_is_zero(base, o);
}

// 恢复的记录读者收不收一条 checker 的读法（checker_record_valid）已经收下的记录（walk.rs 的 journal_record_is_read_by_the_recovery_reader；
// core 解记录头那一段，`crates/singlefs-core/src/journal.rs` 的 JournalRecord::parse，D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 17、
// D19（块指针的结构与宽度预算） 已定项 3）：提交标记只许 0 / 1（偏移 86）、记录标志位 0 之外全 0（偏移 7）、本次发布内序号不是 0（偏移 87）、
// 新根段四条指针（树表、映射根、实例表、分配记录树根）头部都是第一版的取值。checker 的读法收下读不出的这几样，好让 I-8.8、I-8.9 判红；
// 恢复视图与重放读的那一份环只收读得出的
fn journal_record_reads_as_the_recovery_reader_reads_it(base: u32) -> bool {
    return rd8(base, 86u) <= 1u && (rd8(base, 7u) & 0xFEu) == 0u && rd32(base, 87u) != 0u
        && journal_new_root_segment_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET)
        && journal_new_root_segment_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + W_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER)
        && journal_new_root_segment_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER)
        && journal_new_root_segment_pointer_head_is_the_first_version(base,
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER);
}

// 一块盘上的记录按键 (计数器, 实例代号) 排进草稿区：版本、计数器 lo/hi、实例、环槽 lo/hi。插入点用二分找（环按偏移收、计数器升序到达时就是表尾，不挪）。
// 同一个键按收的次序（扫环次序）先到先留，同一计数器别的实例的记录照收（walk.rs scanned_journal_ring_of_device 判不变量读的那一份表，
// 与 core 扫环 scan_journal 同一个键；`records/2026-10-24-singlefs-checker-审计复核.md` 的 C02）。恢复的读者读不出的记录也照收、照先到先留：
// 逐盘判的那几条（I-8.8、I-8.9）要看见它们；恢复视图另只收读得出的（checker_scan_journal_as_the_recovery_reads）
fn checker_insert_record(version: u32, counter: vec2<u32>, instance: u32, ring_slot: vec2<u32>) {
    let position = checker_record_position_not_below(counter, instance);
    if (checker_record_key_at_is(position, counter, instance)) {
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

// D23（journal 的角色与格式） 已定项 18：记录 n 落在环内偏移 (计数器 − 1) mod 槽数 × 4096，计数器从 1 起（这里只交计数器 ≥ 1 的）。
// ring_slots 是环槽数（环长 ÷ 4096，几何判过在飞上限 ≥ 1，不是 0）
fn journal_record_sits_in_the_ring_slot_its_counter_names(counter: vec2<u32>, ring_slot: vec2<u32>, ring_slots: vec2<u32>) -> bool {
    return eq64(divmod64(sub64(counter, u64_of(1u)), ring_slots).remainder, ring_slot);
}

// 一块盘的环：候选记录槽逐个读，自证过的按计数器排进草稿区。每条自证过、计数器 ≥ 1 的记录判一次 I-8.13（C897，walk.rs 的
// judge_journal_records_live_in_the_ring_slot_their_counter_names）：判在先到先留之前，同一块盘上撞键时被挤掉的那一条也判（合法写者每个计数器
// 只写它那一槽，同一块盘上两条同键的记录至少一条不在它该落的槽）；计数器 0 没有该落的槽、不判（归 I-8.6）。一条都没判到时这一条不算判到。
// 只因点名项 flags 非 0 被池级 checker 的记录读者拒的也判 I-8.13（恢复的读者收它），不进判不变量那一份表
// 逐盘扫环时有没有一槽读不出（不全零而不自证）：I-8.14 有它就报部分没判。judge_the_pool_from_its_system_configuration_slots 逐盘扫之前清零
var<private> w_some_journal_slot_does_not_read_back: bool;

fn checker_scan_journal_of_device(device: u32, fsid_low: vec2<u32>, ring_slots: vec2<u32>) {
    record_count = 0u;
    let location_count = hdr(6u);
    for (var l = 0u; l < location_count; l = l + 1u) {
        if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {
            continue;
        }
        let s = read_location(l);
        if (!checker_record_valid_past_the_named_entry_flags(s, fsid_low)) {
            // I-8.14：不全零而不自证的槽读不出（walk.rs scanned_journal_ring_of_device 的 slots_that_do_not_read_back；没有版本的切片是从没写过的槽）
            if (s.version != NONE && !all_zero(s.base, 0u, s.length)) {
                w_some_journal_slot_does_not_read_back = true;
            }
            continue;
        }
        let offset = vec2<u32>(location_word(l, 1u), location_word(l, 2u));
        let ring_slot = shr64(sub64(offset, u64_of(JOURNAL_RING_START_BYTES)), 12u);
        let counter = rd48(s.base, 20u);
        if (!is_zero64(counter)) {
            judge(INVARIANT_I_8_13, journal_record_sits_in_the_ring_slot_its_counter_names(counter, ring_slot, ring_slots));
        }
        // 只因点名项 flags 非 0 被池级 checker 的记录读者拒的：I-8.13 照判它的落点（上一行），判不变量那一份表照旧不收
        // （同盘同键先到时不挤掉后到的那条）
        if (!journal_record_named_entry_flags_are_zero(s.base)) {
            continue;
        }
        checker_insert_record(s.version, counter, rd32(s.base, 16u), ring_slot);
    }
}

// I-8.6：反向链 = 本实例内逻辑前一条头的链值；计数器 1 ⇒ 0；计数器 0 违例。本实例内逻辑前一条按键 (计数器 − 1, 本实例) 找
// （walk.rs expected_back_chain_of）：在 ⇒ 它的链值；不在而这块盘上计数器 − 1 坐着别的实例的记录 ⇒ 0；都没有（前一条不在这块盘上）不判
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
            let previous_counter = sub64(r.counter, u64_of(1u));
            let previous_of_the_same_instance = checker_find_record_by_key(previous_counter, r.instance);
            if (previous_of_the_same_instance != NONE) {
                expected = checker_record_at(previous_of_the_same_instance).chain_value;
            } else if (checker_some_record_sits_at_counter(previous_counter)) {
                expected = 0u;
            } else {
                continue;
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

// I-8.7「从 1 起」那半句（walk.rs judge_transaction_numbers_start_from_one_per_instance，C469 ②逐盘判）：
// 每个在这块盘上写出过非 0 记录的实例，它计数器最小的那条是它的第一条（计数器 0 或 1，或前一个计数器上坐着别的实例的记录）、从那一条到
// 它第一条非 0 记录之间每个计数器都坐着记录时，判第一条非 0 记录的事务号是 1。前一个计数器那一格坐着环转过一圈之后的记录（计数器大一个环长）时
// 这个实例的第一条被盖掉了，射程外、什么都不记（CPU 同；判决 research/prompts/m3-close-b19-checker-r1-main-verification.md 第二节 K3 ①：
// 记成射程里判不了会把 ① ② 已判成立的整条压掉）；那一格读不出、中间有一格读不出（CPU 报部分没判），GPU 只记射程里判不了：不置判到位
fn judge_transaction_numbers_start_from_one_of_device(ring_slots: vec2<u32>) {
    for (var i = 0u; i < record_count; i = i + 1u) {
        let r = checker_record_at(i);
        // 只从每个实例计数器最小的那条起判：表按 (计数器, 实例) 升序，前面有同实例的就不是它（紧挨着的前一条先看，常见的连号一步就出）
        var earlier_of_the_same_instance = i > 0u && checker_record_at(i - 1u).instance == r.instance;
        for (var j = 0u; j < i && !earlier_of_the_same_instance; j = j + 1u) {
            if (checker_record_at(j).instance == r.instance) {
                earlier_of_the_same_instance = true;
            }
        }
        if (earlier_of_the_same_instance) {
            continue;
        }
        var non_empty_index = NONE;
        for (var j = i; j < record_count; j = j + 1u) {
            let candidate = checker_record_at(j);
            if (candidate.instance == r.instance && !is_zero64(candidate.transaction)) {
                non_empty_index = j;
                break;
            }
        }
        if (non_empty_index == NONE) {
            continue;
        }
        if (!lt64(r.counter, u64_of(2u)) && !checker_some_record_sits_at_counter(sub64(r.counter, u64_of(1u)))) {
            if (!checker_some_record_sits_at_counter(add64(sub64(r.counter, u64_of(1u)), ring_slots))) {
                mark_part_of_the_range_not_judged(INVARIANT_I_8_7);
            }
            continue;
        }
        // 中间每个计数器都坐着记录：表按计数器升序，走到第一条非 0 记录为止相邻两条的计数器差不超过 1
        var some_counter_between_does_not_read_back = false;
        var previous_counter = r.counter;
        for (var k = i + 1u; k <= non_empty_index; k = k + 1u) {
            let counter = checker_record_at(k).counter;
            if (lt64(add64(previous_counter, u64_of(1u)), counter)) {
                some_counter_between_does_not_read_back = true;
                break;
            }
            previous_counter = counter;
        }
        if (some_counter_between_does_not_read_back) {
            mark_part_of_the_range_not_judged(INVARIANT_I_8_7);
            continue;
        }
        // 只在不成立时判（CPU 同：成立时不单独算一次评估，只有一条非 0 记录时照旧不置判到位）
        if (!eq64(checker_record_at(non_empty_index).transaction, u64_of(1u))) {
            judge(INVARIANT_I_8_7, false);
        }
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
        // 「不从 1 起」：组首计数器 − 1 上坐着记录（哪个实例的都算：同实例同 txg 的会在组里，坐在那里的一定不属于这一组）
        if (!is_zero64(r.counter)) {
            if (checker_some_record_sits_at_counter(sub64(r.counter, u64_of(1u))) && r.ordinal != 1u) {
                violate(INVARIANT_I_8_9);
            }
        }
        if (flagged == 0u) {
            // 「没有末条」：组末计数器 + 1 上坐着同一实例的记录（按键找，walk.rs 的 records.contains_key）
            let last = checker_record_at(last_index);
            if (checker_find_record_by_key(add64(last.counter, u64_of(1u)), r.instance) != NONE) {
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
// 同名七段（根记录 36、256、122、130、170、342，卸载记号对根记录 flags 20 那 4 字节）逐字节相同（从四段扩到七段）。
// 要 C_ROW_ORDER 里是按 (实例, checkpoint_txg) 排好的次序：一组里带末条标志的数一遍，恰一条才比。
// 往非末条扩的那一半（C738，walk.rs judge_new_root_segment_of_the_last_record_against_the_roots 开头那一段）：一组里每条记录的新根段 373 字节
// 与组首那一条逐字节相同，组里两条以上才有对象。walk.rs 拿组里计数器最小的那一条当比的基准，这里拿组首（表里下标最小的那一条）：
// 逐字节相同可传递，「每条都与某一条相同」与基准取哪一条无关，判到与违例都一样
fn judge_new_root_segment_of_the_last_record_against_the_roots_of_device() {
    var position = 0u;
    while (position < record_count) {
        let group_end = journal_group_end(position);
        let first_base = checker_record_base_at(sw(C_ROW_ORDER + position));
        for (var member_position = position + 1u; member_position < group_end; member_position = member_position + 1u) {
            judge(INVARIANT_I_8_10, bytes_equal(checker_record_base_at(sw(C_ROW_ORDER + member_position)), JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
                first_base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, JOURNAL_NEW_ROOT_SEGMENT_BYTES));
        }
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
                && bytes_equal(r.base, segment + JOURNAL_NEW_ROOT_SEGMENT_RECLAIM_THRESHOLD, root.base, ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET, 9u)
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

// walk::instance_carriers 的 some_slot_is_unreadable 在根环槽上的那一半（池级 checker 按择到的那一份几何读 R × S 个根槽，
// image::root_slot_positions 同一个算式）：有一槽 CPU 的读者交 None 就是读不全——那一槽的盘不在池里（卸载掉电之后只剩一块盘，
// 丢掉的那块住着根环区域），或越过盘尾（memory_pool 的 PoolReader::read）。GPU 判表只在池里的盘、盘内建根槽落点
// （crash_judge_tables::enumerate_locations），读那几槽交没有版本的切片，与 CPU 读不出同一个结局；这里照 CPU 的条件现判，不看切片
fn checker_some_root_slot_is_unreadable(g: CheckerSlot) -> bool {
    for (var region = 0u; region < g.regions; region = region + 1u) {
        let device = checker_region_device(g, region);
        if (!device_is_in_the_pool(device)) {
            return true;
        }
        for (var slot = 0u; slot < g.slots_per_region; slot = slot + 1u) {
            let offset = add64(slot_to_bytes(g.base_slot), u64_of(region * ROOT_RING_REGION_STRIDE + slot * g.spacing));
            let root_slot_bytes = u64_of(g.physical_block_size);
            if (add64_overflows(offset, root_slot_bytes) || lt64(device_bytes(), add64(offset, root_slot_bytes))) {
                return true;
            }
        }
    }
    return false;
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

// I-9.7（walk.rs judge_reserved_bytes_and_the_truncation_field_of_an_inode_record，判决 `research/prompts/m4-trunc-r2-main-verification.md`
// 第三节 R1）：一条记录各判三次——偏移 108–119 恒 0、136–139 恒 0，截断字段判「T = 0 ⇒ N_w = 0」
// 打包表 base 起 [o, o + byte_count) 那几个字节全是 0：与逐字节 all_zero 判的同一件事，按字读、每个字只读一次（首尾两个字只看落在这一段里的字节），
// 走读内核里每条 inode 记录的两段保留字节都过它
fn w_bytes_are_all_zero_reading_each_word_once(base: u32, o: u32, byte_count: u32) -> bool {
    if (byte_count == 0u) {
        return true;
    }
    let first_word = o >> 2u;
    let last_word = (o + byte_count - 1u) >> 2u;
    // 迭代上界：这一段跨的字数 ⌈byte_count ÷ 4⌉ + 1
    for (var word = first_word; word <= last_word; word = word + 1u) {
        var mask_of_the_bytes_in_the_range = 0xFFFFFFFFu;
        if (word == first_word) {
            mask_of_the_bytes_in_the_range = mask_of_the_bytes_in_the_range << ((o & 3u) << 3u);
        }
        if (word == last_word) {
            mask_of_the_bytes_in_the_range = mask_of_the_bytes_in_the_range & (0xFFFFFFFFu >> ((3u - ((o + byte_count - 1u) & 3u)) << 3u));
        }
        if ((tables[base + word] & mask_of_the_bytes_in_the_range) != 0u) {
            return false;
        }
    }
    return true;
}

fn w_judge_reserved_bytes_and_the_truncation_field_of_an_inode_record(base: u32, record_offset: u32) {
    w_judge(INVARIANT_I_9_7, w_bytes_are_all_zero_reading_each_word_once(base,
        record_offset + INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD_OFFSET, INODE_RECORD_RESERVED_BYTES_BEFORE_THE_TRUNCATION_FIELD_LENGTH));
    w_judge(INVARIANT_I_9_7, w_bytes_are_all_zero_reading_each_word_once(base,
        record_offset + INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD_OFFSET, INODE_RECORD_RESERVED_BYTES_AFTER_THE_TRUNCATION_FIELD_LENGTH));
    let truncation_field_txg = rd64(base, record_offset + INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET);
    let truncation_field_size = rd64(base, record_offset + INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET);
    w_judge(INVARIANT_I_9_7, !is_zero64(truncation_field_txg) || is_zero64(truncation_field_size));
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

// 正在走的这一版的实例表指针非全零、出生 txg 等于这一版的 txg（w_walk_one 读指针时置，这一版走完记进版本表那一行）
var<private> appearance_instance_table_pointer_is_born_with_the_version: bool;

// ② 版本表一行里第 C_APPEARANCE_VERSION_INSTANCE_TABLE_POINTER_IS_BORN_WITH_THE_VERSION_OFFSET 字：这一版的实例表指针是不是这一版新建的（1 / 0，
// walk.rs VersionReferencesForTheAppearanceRule::instance_table_pointer_birth_txg 等于这一版的 txg）。用的是用户可见树的根那几棵之后留空的最后一字
const C_APPEARANCE_VERSION_INSTANCE_TABLE_POINTER_IS_BORN_WITH_THE_VERSION_OFFSET: u32 = 47u;

const_assert C_APPEARANCE_VERSION_INSTANCE_TABLE_POINTER_IS_BORN_WITH_THE_VERSION_OFFSET == C_APPEARANCE_VERSION_ROOTS_OFFSET + C_APPEARANCE_MAX_USER_VISIBLE_ROOTS * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;

const_assert C_APPEARANCE_VERSION_INSTANCE_TABLE_POINTER_IS_BORN_WITH_THE_VERSION_OFFSET < C_APPEARANCE_VERSION_WORDS;

// 正在走的这一版的标签戳（walk.rs 每走一版之前清空 user_visible_tree_of_each_placement_in_this_version）：w_start(0) 清零，
// 主走读每开一版加一；落点表一行第 6 字等于它，才是这一版打的标签
var<private> appearance_label_stamp: u32;

// 正在走的这一版树表里用户可见树的根记了几棵：记在版本表第 appearance_version_count 行（这一版走完才算进那一行）
var<private> appearance_root_count_of_this_version: u32;

// 走读此刻在哪棵用户可见树里往下走（walk.rs 的 user_visible_tree_being_walked）：在不在、种类、树 ID
var<private> appearance_user_visible_tree_is_being_walked: bool;

var<private> appearance_user_visible_tree_kind: u32;

var<private> appearance_user_visible_tree: vec2<u32>;

// 落点表一行第 7 字：低 16 位是这一版给它打的标签那棵树的种类，最高位记这个落点第一次记下时是不是按中央映射 key 折的
// （walk.rs 的 placements_first_noted_through_the_central_mapping，C748（最终折叠表漏判引用区间的落点））：I-3.5 比相邻两版时
// 置了这一位的按第 3–5 字（折成的单元）认，没置的按第 0–2 字（落点自己）认；取诞生代号一律按第 3–5 字
const APPEARANCE_PLACEMENT_TREE_KIND_MASK: u32 = 0xFFFFu;
const APPEARANCE_PLACEMENT_NOTED_THROUGH_THE_CENTRAL_MAPPING: u32 = 0x80000000u;

// 用户可见树的种类（walk.rs 的 USER_VISIBLE_TREE_KINDS：extent、inode、deleted_inodes）
fn appearance_tree_kind_is_user_visible(kind: u32) -> bool {
    return kind == TREE_KIND_EXTENT || kind == TREE_KIND_INODE || kind == TREE_KIND_DELETED_INODES;
}

// 落点表里 (盘, 槽) 那一行的下标，不在就是 NONE（按桶表 W_PLACEMENT_BUCKETS 找）
fn appearance_placement_index(device: u32, slot: vec2<u32>) -> u32 {
    return hashed_row_of(W_PLACEMENT_BUCKETS, W_PLACEMENT_BUCKET_COUNT, C_APPEARANCE_PLACEMENTS, C_APPEARANCE_PLACEMENT_WORDS, device, slot, 0u, false).row;
}

// walk.rs note_the_user_visible_tree_of_a_placement_in_this_version：走读此刻在这一版的一棵用户可见树里，就把落点表这一行记成这一版里落在
// 那棵树里（戳写这一版的，种类、树 ID 写那棵树）；这一版先打过的不换，别的版本打的（戳不同）照换
fn appearance_label_the_placement_row_in_this_version(row: u32) {
    if (appearance_user_visible_tree_is_being_walked && sw(row + 6u) != appearance_label_stamp) {
        sws(row + 6u, appearance_label_stamp);
        sws(row + 7u, (sw(row + 7u) & APPEARANCE_PLACEMENT_NOTED_THROUGH_THE_CENTRAL_MAPPING) | appearance_user_visible_tree_kind);
        sws(row + 8u, appearance_user_visible_tree.x);
        sws(row + 9u, appearance_user_visible_tree.y);
    }
}

// walk.rs merge_placements_below_a_unit_walked_by_an_earlier_version 里那一句：别的版本先走过的单元下面并进这一版的落点，按这一版此刻
// 在哪棵用户可见树里打标签（与先走那一版从哪棵树、或从快照那两棵树走到它无关）。落点表里没有这一行时不打（这一版走完冻 ③ 时那一处记判不了）
fn appearance_label_a_merged_placement_in_this_version(device: u32, slot: vec2<u32>) {
    if (!appearance_user_visible_tree_is_being_walked) {
        return;
    }
    let index = appearance_placement_index(device, slot);
    if (index != NONE) {
        appearance_label_the_placement_row_in_this_version(C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS);
    }
}

// walk.rs note_references_of_a_pointer_followed_by_the_walk 里记 I-3.5 那两张表的几句：落点第一次记下时记它是哪个单元
// （unit_of_each_placement.entry(..).or_insert）、是不是按中央映射 key 折的（placements_first_noted_through_the_central_mapping），
// 走读此刻在这一版的一棵用户可见树里时给它打这一版的标签（note_the_user_visible_tree_of_a_placement_in_this_version）
fn appearance_note_placement(device: u32, slot: vec2<u32>, unit_device: u32, unit_slot: vec2<u32>, noted_through_the_central_mapping: bool) {
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
        sws(new_row + 7u, select(0u, APPEARANCE_PLACEMENT_NOTED_THROUGH_THE_CENTRAL_MAPPING, noted_through_the_central_mapping));
        sws(new_row + 8u, 0u);
        sws(new_row + 9u, 0u);
        if (found.empty_bucket != NONE) {
            sws(W_PLACEMENT_BUCKETS + found.empty_bucket, index + 1u);
        }
        appearance_placement_count = appearance_placement_count + 1u;
    }
    appearance_label_the_placement_row_in_this_version(C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS);
}

// 实例表一片的诞生代号（walk.rs 的 birth_txg_of_walked_instance_table_pages）：记进走过的单元那一行，标记写 2——
// I-4.9 只认标记 1（实例表单元按 I-1.2 那一行的例外不进 I-4.9），I-3.5 两种都认
fn appearance_note_the_birth_txg_of_an_instance_table_page(device: u32, slot: vec2<u32>, birth_txg: vec2<u32>) {
    let visited_index = w_visited_index_in_the_main_walk(device, slot);
    if (visited_index == NONE || w_birth_mark_of_a_visited_row(visited_index) != 0u) {
        return;
    }
    w_note_the_birth_txg_of_a_visited_row(visited_index, 2u, birth_txg);
}

// 正在走的这一版树表里一棵用户可见树的根记进版本表的待记那一行（版本表满了不记：这一版走完记它时判不了）
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

// walk.rs tree_table_entry_is_a_shape_without_a_clause：树表的一条条目是不是 I-3.5 条款说的「走到没有条款的形状」（主走读走不下去、这一版的引用集合数不全）。
// 种类码登记到 TREE_KIND_SNAPSHOT_EXTENT_ROOT 为止，没登记的（0 与更大的码）一律算；livelist、稀疏旁表根指针全零是第一版的合法形状，非全零才算；别的走读走得下去
fn appearance_tree_table_entry_is_a_shape_without_a_clause(kind: u32, root_pointer_is_all_zero: bool) -> bool {
    if (kind == TREE_KIND_LIVELIST || kind == TREE_KIND_SPARSE_SIDE_TABLE) {
        return !root_pointer_is_all_zero;
    }
    return kind == 0u || kind > TREE_KIND_SNAPSHOT_EXTENT_ROOT;
}

// 这一版第一个读得出的树表（walk_the_entries_of_a_tree_table_walked_first_by_this_version）：树表里每棵根指针非全零的用户可见树
// （条目窄于 200 字节的不算，tree_table_entry_of_the_walk 交 None）记成这一版的根，连同树表落点、这一版在 ② 里的下标与有没有「走到没有条款的形状」的条目记进走过的树表那张表
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
            if (appearance_tree_table_entry_is_a_shape_without_a_clause(kind, root_pointer.all_zero)) {
                appearance_tree_table_of_this_version_has_an_entry_without_a_clause = true;
            }
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
    sws(table_row + 3u, roots | (appearance_version_count << APPEARANCE_FIRST_WALKER_SHIFT)
        | select(0u, APPEARANCE_WALKED_TREE_TABLE_HAS_AN_ENTRY_WITHOUT_A_CLAUSE, appearance_tree_table_of_this_version_has_an_entry_without_a_clause));
    appearance_walked_tree_table_count = appearance_walked_tree_table_count + 1u;
}

// 这一版的树表别的版本先走过（judge_the_tree_table_entries_an_earlier_version_walked）：这一版的用户可见树的根与有没有「走到没有条款的形状」的条目照先走那一版记下的取，
// 标签照先走那一版冻进 ③ 的接上（take_the_user_visible_tree_labels_below_a_tree_table_walked_earlier）；先走那一版头用不了、没记下的，这里没有对象
fn appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_device: u32, tree_table_slot: vec2<u32>) {
    for (var t = 0u; t < appearance_walked_tree_table_count; t = t + 1u) {
        let table_row = C_APPEARANCE_WALKED_TREE_TABLES + t * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
        if (sw(table_row) != tree_table_device || sw(table_row + 1u) != tree_table_slot.x || sw(table_row + 2u) != tree_table_slot.y) {
            continue;
        }
        let walked_tree_table_word = sw(table_row + 3u);
        appearance_tree_table_of_this_version_has_an_entry_without_a_clause = (walked_tree_table_word & APPEARANCE_WALKED_TREE_TABLE_HAS_AN_ENTRY_WITHOUT_A_CLAUSE) != 0u;
        let roots = walked_tree_table_word & ((1u << APPEARANCE_FIRST_WALKER_SHIFT) - 1u);
        for (var r = 0u; r < roots; r = r + 1u) {
            let root_row = table_row + C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET + r * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
            appearance_note_a_user_visible_root_of_this_version(sw(root_row), vec2<u32>(sw(root_row + 1u), sw(root_row + 2u)), vec2<u32>(sw(root_row + 3u), sw(root_row + 4u)));
        }
        appearance_take_the_labels_of_the_version_that_walked_the_tree_table_first((walked_tree_table_word & ~APPEARANCE_WALKED_TREE_TABLE_HAS_AN_ENTRY_WITHOUT_A_CLAUSE) >> APPEARANCE_FIRST_WALKER_SHIFT);
        return;
    }
}

// walk.rs take_the_user_visible_tree_labels_below_a_tree_table_walked_earlier：树表下面的落点并进这一版的那一刻走读不在任何一棵树里、没打标签，
// 照第一个走到这片树表的那一版（② 里第 first_walker 行，已经走完、冻过 ③）记下的接上：那一版每个打了标签的落点，这一版还没打的打上同一棵树
// （种类、树 ID 取那一版 ② 那一行里标签码指的那棵根）。first_walker 不是走完冻过的那几版（冻的时候记了判不了）时不接
fn appearance_take_the_labels_of_the_version_that_walked_the_tree_table_first(first_walker: u32) {
    if (first_walker >= appearance_version_count) {
        return;
    }
    let walker_row = appearance_version_row(first_walker);
    let first = sw(walker_row + 4u);
    let count = sw(walker_row + 5u);
    for (var k = 0u; k < count; k = k + 1u) {
        let word = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k);
        let label = word >> APPEARANCE_LABEL_SHIFT;
        let placement_row = C_APPEARANCE_PLACEMENTS + (word & APPEARANCE_PLACEMENT_INDEX_MASK) * C_APPEARANCE_PLACEMENT_WORDS;
        if (label == 0u || sw(placement_row + 6u) == appearance_label_stamp) {
            continue;
        }
        let root_row = walker_row + C_APPEARANCE_VERSION_ROOTS_OFFSET + (label - 1u) * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
        sws(placement_row + 6u, appearance_label_stamp);
        sws(placement_row + 7u, (sw(placement_row + 7u) & APPEARANCE_PLACEMENT_NOTED_THROUGH_THE_CENTRAL_MAPPING) | sw(root_row));
        sws(placement_row + 8u, sw(root_row + 1u));
        sws(placement_row + 9u, sw(root_row + 2u));
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

// 落点表第 placement 行这一版打的标签换成 ③ 的标签码：这一版没打是 0，打了是那棵树在这一版 ② 那一行（version_row）根里的序号加一
// （按种类与树 ID 找第一棵对得上的）；打了而这一版的根里找不到那棵树交 NONE——标签只在走这一版树表里的用户可见树时打、或照共享树表的
// 先走那一版接上，那几棵都记进了这一版的根，找不到只在根没记全时出现（根多于 C_APPEARANCE_MAX_USER_VISIBLE_ROOTS 棵、走过的树表那张表满了，那几处已记判不了）
fn appearance_label_code_in_this_version(version_row: u32, placement: u32) -> u32 {
    let placement_row = C_APPEARANCE_PLACEMENTS + placement * C_APPEARANCE_PLACEMENT_WORDS;
    if (sw(placement_row + 6u) != appearance_label_stamp) {
        return 0u;
    }
    for (var r = 0u; r < appearance_root_count_of_this_version; r = r + 1u) {
        let root_row = version_row + C_APPEARANCE_VERSION_ROOTS_OFFSET + r * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
        if (sw(root_row) == (sw(placement_row + 7u) & APPEARANCE_PLACEMENT_TREE_KIND_MASK) && sw(root_row + 1u) == sw(placement_row + 8u) && sw(root_row + 2u) == sw(placement_row + 9u)) {
            return r + 1u;
        }
    }
    return NONE;
}

// ⑤ 每一版谱系树里的谱系记录（walk.rs VersionPlacementsForTheAppearanceRule::lineage_records，读法 lineage_records_of_a_version_for_the_appearance_rule）：
//    版本表 ② 第 v 行那一版占一行——第 0 字读得全是 1、判不了是 0（树表指针全零、树表读不出、条目比字段表窄、谱系树条目不是恰好一条、谱系树读不全），
//    第 1 字谱系容器几片，之后每片 3 个字：容器在打包表里的字节基、记录数、第一条记录的偏移。记录不抄，判的时候从容器字节里读
//    （同扫描内核的 D_LINEAGE_CONTAINERS）。一版的容器多于 C_APPEARANCE_MAX_LINEAGE_CONTAINERS 片判不了（站点 120）。
//    放在版本表 ② 之后那一段（crash_judge_checker_common.wgsl 里 C_APPEARANCE_VERSION_PLACEMENTS 上面那一句说的前 7808 个字）
const C_APPEARANCE_MAX_LINEAGE_CONTAINERS: u32 = 40u;

const C_APPEARANCE_LINEAGE_CONTAINER_WORDS: u32 = 3u;

const C_APPEARANCE_LINEAGE_WORDS: u32 = 2u + C_APPEARANCE_LINEAGE_CONTAINER_WORDS * C_APPEARANCE_MAX_LINEAGE_CONTAINERS;

const C_APPEARANCE_LINEAGES: u32 = C_APPEARANCE_VERSIONS + C_APPEARANCE_MAX_VERSIONS * C_APPEARANCE_VERSION_WORDS;

const_assert C_APPEARANCE_LINEAGES + C_APPEARANCE_MAX_VERSIONS * C_APPEARANCE_LINEAGE_WORDS <= C_APPEARANCE_WALKED_TREE_TABLES;

// 谱系记录 24 字节（打包记录类型 5 登记的宽）：T_rb 在偏移 0、T_old 在偏移 8，读作「T_rb 那次回退回到了 T_old」
const APPEARANCE_LINEAGE_RECORD_BYTES: u32 = 24u;

const APPEARANCE_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: u32 = 0u;

const APPEARANCE_LINEAGE_RECORD_PARENT_TXG_OFFSET: u32 = 8u;

// 一版的谱系容器多于 ⑤ 一行装得下的片数、谱系树往下走的显式栈满了：判不了（交 CPU）
const UNDECIDABLE_APPEARANCE_LINEAGE_CONTAINERS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (120u << 8u);

const UNDECIDABLE_APPEARANCE_LINEAGE_TREE_PAST_THE_STACK: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (121u << 8u);

// 正在走的这一版的树表里有没有「走到没有条款的形状」的条目（appearance_tree_table_entry_is_a_shape_without_a_clause）：这一版走完记进 ②
var<private> appearance_tree_table_of_this_version_has_an_entry_without_a_clause: bool;

// ④ 第 3 字的最高位：这片树表里有「走到没有条款的形状」的条目（第一个走到它的那一版的下标不到 C_APPEARANCE_MAX_VERSIONS，碰不到这一位）。
// 共享这片树表的版本不重走条目，照这一位认
const APPEARANCE_WALKED_TREE_TABLE_HAS_AN_ENTRY_WITHOUT_A_CLAUSE: u32 = 1u << 31u;

const_assert C_APPEARANCE_MAX_VERSIONS < (1u << (31u - APPEARANCE_FIRST_WALKER_SHIFT));

// 正在走的这一版的树表指针（w_walk_one 读指针时置，这一版走完读它的谱系）
var<private> appearance_tree_table_pointer_of_the_version: Ptr;

fn appearance_lineage_row(version_index: u32) -> u32 {
    return C_APPEARANCE_LINEAGES + version_index * C_APPEARANCE_LINEAGE_WORDS;
}

// walk.rs read_located_unit_without_judging：读一个豁免三类之外的单元、不判（同扫描内核的 r_read_located_unit）。按查找路径给的类在装着的那一版映射里
// 认位置；认到的位置与提示（指针自己的两条位置条目）不同时先按提示读，校验和对得上、头里出生身份与指针相同才用；读不出再按映射位置两条位置条目读、
// 校验和过滤；两者相同时照位置条目读。提示两份与位置两份在一个循环里读（一个 read_at 调用点）
fn appearance_read_located_unit_without_judging(p: Ptr, mapping_class: u32, unit_bytes: u32) -> Slice {
    let located = located_by_the_central_mapping(p, mapping_class);
    let hints_differ = located.device0 != p.device0 || !eq64(located.slot0, p.slot0) || located.device1 != p.device1 || !eq64(located.slot1, p.slot1);
    for (var copy = 0u; copy < 4u; copy = copy + 1u) {
        let is_a_hint = copy < 2u;
        if (is_a_hint && !hints_differ) {
            continue;
        }
        let is_the_second = (copy % 2u) == 1u;
        var device = select(located.device0, located.device1, is_the_second);
        var slot = select(located.slot0, located.slot1, is_the_second);
        if (is_a_hint) {
            device = select(p.device0, p.device1, is_the_second);
            slot = select(p.slot0, p.slot1, is_the_second);
        }
        let s = read_at(device, slot_to_bytes(slot), unit_bytes);
        if (s.version != NONE && slice_crc_matches(s, unit_bytes, select(p.sum0, p.sum1, is_the_second))
            && (!is_a_hint || unit_birth_identity_matches_the_pointer(s, p))) {
            return s;
        }
    }
    return no_slice();
}

// read_index_node_without_judging 按这一版中央映射认位置的那一支（谱系树的码 2 节点）
fn appearance_read_located_index_node_without_judging(p: Ptr) -> Node {
    let s = appearance_read_located_unit_without_judging(p, MAPPING_CLASS_INDEX_NODE, NODE_BYTES);
    if (s.version == NONE) {
        return bad_node();
    }
    return parse_index_node(s);
}

// 谱系树一个码 2 节点的条目倒着压进 C_PENDING（每条字节基、条目偏移、这个节点的层级），弹出来是叶序。C_PENDING 这一段一版中央映射的只读读法
// 在走树表条目之前借、I-3.5 摊落点在全部版本走完之后借，这里借在一版走完那一刻，三处不重叠
fn appearance_push_lineage_tree_entries_in_reverse(node: Node, pending: ptr<function, u32>) -> bool {
    for (var k = 0u; k < node.entry_count; k = k + 1u) {
        if (*pending >= C_MAX_PENDING) {
            undecidable = UNDECIDABLE_APPEARANCE_LINEAGE_TREE_PAST_THE_STACK;
            return false;
        }
        let row = C_PENDING + *pending * 3u;
        sws(row, node.base);
        sws(row + 1u, entry_offset(node, node.entry_count - 1u - k));
        sws(row + 2u, node.level);
        *pending = *pending + 1u;
    }
    return true;
}

// walk.rs lineage_records_of_a_version_for_the_appearance_rule：一版走完（装着的还是这一版的中央映射），读这一版树表里谱系树的每片容器，记进 ⑤ 第 version_index 行。
// 树表按指针本身读（自举豁免）：指针全零、读不出、有条目而条目宽不到 200 判不了；0 条是没有谱系树、读得全（空）；谱系树条目不是恰好一条判不了。
// 谱系树（walk.rs containers_of_a_container_tree_without_judging，同扫描内核的 d_read_containers_of_a_container_tree）：根指针全零读得全（空）；
// 码 2 节点要读得出、层级 ≥ 1、条目宽 ≥ 120，类型段 0 的条目指下一层码 2 节点（子指针非零、层级 = 父层级 − 1），类型段 5 的指码 3 容器
// （打包记录类型 5、记录宽 24），别的类型段、读不出、形状不对判不了
fn appearance_note_the_lineage_of_the_version_just_walked(version_index: u32) {
    let row = appearance_lineage_row(version_index);
    sws(row, 0u);
    sws(row + 1u, 0u);
    let tree_table_pointer = appearance_tree_table_pointer_of_the_version;
    if (tree_table_pointer.all_zero) {
        return;
    }
    let tt = parse_index_node(read_unit_via_locations(tree_table_pointer, NODE_BYTES));
    if (!tt.ok || (tt.entry_count > 0u && tt.entry_width < 200u)) {
        return;
    }
    var lineage_entry_count = 0u;
    var lineage_entry = 0u;
    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
        let o = entry_offset(tt, e);
        if (rd16(tt.base, o + 10u) == TREE_KIND_LINEAGE) {
            lineage_entry_count = lineage_entry_count + 1u;
            lineage_entry = o;
        }
    }
    if (tt.entry_count == 0u) {
        sws(row, 1u);
        return;
    }
    if (lineage_entry_count != 1u) {
        return;
    }
    let root_pointer = parse_node_pointer(tt.base, lineage_entry + 14u);
    if (root_pointer.all_zero) {
        sws(row, 1u);
        return;
    }
    let root = appearance_read_located_index_node_without_judging(root_pointer);
    if (!root.ok || root.level == 0u || root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
        return;
    }
    var pending = 0u;
    if (!appearance_push_lineage_tree_entries_in_reverse(root, &pending)) {
        return;
    }
    var container_count = 0u;
    while (pending > 0u) {
        pending = pending - 1u;
        let pending_row = C_PENDING + pending * 3u;
        let base = sw(pending_row);
        let o = sw(pending_row + 1u);
        let parent_level = sw(pending_row + 2u);
        let child_pointer = parse_node_pointer(base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
        let entry_type = rd16(base, o + 16u);
        if (entry_type == 0u) {
            if (child_pointer.all_zero) {
                return;
            }
            let child = appearance_read_located_index_node_without_judging(child_pointer);
            if (!child.ok || child.level + 1u != parent_level || child.level == 0u || child.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
                return;
            }
            if (!appearance_push_lineage_tree_entries_in_reverse(child, &pending)) {
                return;
            }
        } else if (entry_type == PACKED_TYPE_LINEAGE) {
            if (child_pointer.all_zero) {
                return;
            }
            let container = parse_packed_unit(appearance_read_located_unit_without_judging(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
            if (!container.ok || container.record_type != PACKED_TYPE_LINEAGE || container.record_width != APPEARANCE_LINEAGE_RECORD_BYTES) {
                return;
            }
            if (container_count >= C_APPEARANCE_MAX_LINEAGE_CONTAINERS) {
                undecidable = UNDECIDABLE_APPEARANCE_LINEAGE_CONTAINERS_PAST_THE_TABLE;
                return;
            }
            let container_row = row + 2u + container_count * C_APPEARANCE_LINEAGE_CONTAINER_WORDS;
            sws(container_row, container.base);
            sws(container_row + 1u, container.record_count);
            sws(container_row + 2u, packed_record_offset(container, 0u));
            container_count = container_count + 1u;
        } else {
            return;
        }
    }
    sws(row + 1u, container_count);
    sws(row, 1u);
}

// walk.rs lineage_records_name_a_rollback_between_the_two_versions_to_a_target_not_older_than_the_root：⑤ 第 version_index 行那一版的谱系记录里
// 有没有一条 (T_rb, T_old) 满足 previous_txg < T_rb ≤ version_txg、root_pointer_birth_txg ≤ T_old。只在那一行读得全时调
fn appearance_lineage_names_a_rollback_between_the_two_versions_to_a_target_not_older_than_the_root(version_index: u32, previous_txg: vec2<u32>,
    version_txg: vec2<u32>, root_pointer_birth_txg: vec2<u32>) -> bool {
    let row = appearance_lineage_row(version_index);
    let container_count = sw(row + 1u);
    for (var container_index = 0u; container_index < container_count; container_index = container_index + 1u) {
        let container_row = row + 2u + container_index * C_APPEARANCE_LINEAGE_CONTAINER_WORDS;
        let base = sw(container_row);
        let record_count = sw(container_row + 1u);
        let first_record = sw(container_row + 2u);
        for (var record_index = 0u; record_index < record_count; record_index = record_index + 1u) {
            let record = first_record + record_index * APPEARANCE_LINEAGE_RECORD_BYTES;
            let rollback_txg = rd64(base, record + APPEARANCE_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
            let parent_txg = rd64(base, record + APPEARANCE_LINEAGE_RECORD_PARENT_TXG_OFFSET);
            if (lt64(previous_txg, rollback_txg) && le64(rollback_txg, version_txg) && le64(root_pointer_birth_txg, parent_txg)) {
                return true;
            }
        }
    }
    return false;
}

// note_the_version_just_walked_for_the_appearance_rule：一版走完，把它此刻引用过的落点（这一版的集合，w_judge_placement_in_version 记的）
// 换成落点表的下标、连同这一版打的标签码记进 ③（标签当场冻下，后走的版本不改它），连同实例、txg、这一版的引用收全没有（走读没记失败、树表里没有「走到没有条款的形状」的条目）、实例表指针是不是这一版新建的，
// 再读这一版的谱系记进 ⑤
fn appearance_note_the_version_just_walked(instance: u32, txg: vec2<u32>, references_are_complete: bool, instance_table_pointer_is_born_with_the_version: bool) {
    if (appearance_version_count >= C_APPEARANCE_MAX_VERSIONS) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (43u << 8u);
        return;
    }
    let row = C_APPEARANCE_VERSIONS + appearance_version_count * C_APPEARANCE_VERSION_WORDS;
    sws(row, instance);
    sws(row + 1u, txg.x);
    sws(row + 2u, txg.y);
    sws(row + 3u, select(0u, 1u, references_are_complete));
    sws(row + C_APPEARANCE_VERSION_INSTANCE_TABLE_POINTER_IS_BORN_WITH_THE_VERSION_OFFSET, select(0u, 1u, instance_table_pointer_is_born_with_the_version));
    sws(row + 4u, appearance_version_placement_count);
    let count = w_in_version_count[0];
    for (var i = 0u; i < count; i = i + 1u) {
        let placement = appearance_placement_index(sw(C_IN_VERSION0 + i * 3u), vec2<u32>(sw(C_IN_VERSION0 + i * 3u + 1u), sw(C_IN_VERSION0 + i * 3u + 2u)));
        // 这一版引用过的每个落点都经 w_note_pointer_references 记过（这一版或先走的那一版）：不在说明落点表与这一版的集合脱节了。
        // 每一版引用的落点那张表满了的那一半走不到（它取版本上限乘每版上限，那两处先判不了，见 C_APPEARANCE_MAX_VERSION_PLACEMENTS），留着守写入边界
        if (placement == NONE || appearance_version_placement_count >= C_APPEARANCE_MAX_VERSION_PLACEMENTS) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (44u << 8u);
            return;
        }
        let label = appearance_label_code_in_this_version(row, placement);
        if (label == NONE) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (41u << 8u);
            return;
        }
        sws(C_APPEARANCE_VERSION_PLACEMENTS + appearance_version_placement_count, placement | (label << APPEARANCE_LABEL_SHIFT));
        appearance_version_placement_count = appearance_version_placement_count + 1u;
    }
    sws(row + 5u, count);
    sws(row + 6u, appearance_root_count_of_this_version);
    appearance_note_the_lineage_of_the_version_just_walked(appearance_version_count);
    appearance_version_count = appearance_version_count + 1u;
}

// 主走读里 (盘, 槽) 在走过的单元表里的下标，不在就是 NONE（按桶表 W_VISITED_BUCKETS 找）
fn w_visited_index_in_the_main_walk(device: u32, slot: vec2<u32>) -> u32 {
    return hashed_row_of(W_VISITED_BUCKETS, W_VISITED_BUCKET_COUNT, C_VISITED0, 3u, device, slot, 0u, false).row;
}

// 主走读走过的单元表第 visited_index 行记的诞生代号标记（C_VISITED_BIRTH_MARK_MASK 那两位）
fn w_birth_mark_of_a_visited_row(visited_index: u32) -> u32 {
    return sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS) & C_VISITED_BIRTH_MARK_MASK;
}

// 主走读走过的单元表第 visited_index 行记下诞生代号与它的标记；先走那一遍记的两位（C_VISITED_HEADER_THE_WALK_COULD_USE、C_VISITED_INDEX_NODE_PARSES）照留
fn w_note_the_birth_txg_of_a_visited_row(visited_index: u32, mark: u32, birth_txg: vec2<u32>) {
    let row = C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS;
    sws(row, (sw(row) & ~C_VISITED_BIRTH_MARK_MASK) | mark);
    sws(row + 1u, birth_txg.x);
    sws(row + 2u, birth_txg.y);
}

// 主走读里先走这个单元的那一遍认下的一样东西（C_VISITED_HEADER_THE_WALK_COULD_USE：walk.rs 的 walked_units_whose_header_the_walk_could_use.insert；
// C_VISITED_INDEX_NODE_PARSES），记进它在走过的单元表那一行：那一行是刚才那一次 w_visit 记进去的（w_visited_row_of_the_unit_asked_last），
// 调用方在那一次 w_visit 交回 true 之后、下一次 w_visit 之前调。另起的那几遍（上下文 1）那个下标是 NONE，不记：那几遍的判定丢掉，
// 别的版本再引用时照判的那几格也丢掉
fn w_note_on_the_visited_row_what_the_first_walk_found(found: u32) {
    if (w_visited_row_of_the_unit_asked_last == NONE) {
        return;
    }
    let row = C_VISITED_BIRTHS0 + w_visited_row_of_the_unit_asked_last * C_VISITED_BIRTH_WORDS;
    sws(row, sw(row) | found);
}

fn w_note_the_header_the_walk_could_use() {
    w_note_on_the_visited_row_what_the_first_walk_found(C_VISITED_HEADER_THE_WALK_COULD_USE);
}

// 走读跟着的一条引用要被指单元是什么（walk.rs 的 ReferenceTheWalkFollowsToAUnit）：种类定这条引用要的类标签，其余几段按种类用
const W_REFERENCE_TO_AN_INDEX_NODE_OF_A_TREE: u32 = 0u;          // 码 2 节点：头里的树 ID 是 tree
const W_REFERENCE_TO_A_PACKED_CONTAINER_OF_A_TREE: u32 = 1u;     // 码 3 容器（inode 叶容器，墓碑 / 谱系 / 退役树的容器）：偏移 43 的出生树是 tree
const W_REFERENCE_TO_AN_INSTANCE_TABLE_PAGE: u32 = 2u;           // 实例表链上第 page_index 片（码 3 打包记录类型 4，I-1.3 豁免）
const W_REFERENCE_TO_A_DATA_UNIT_OF_AN_EXTENT_KEY: u32 = 3u;     // extent 树 tree 里 key (extent_inode, extent_offset) 指着的数据单元（码 1）

struct ReferenceTheWalkFollowsToAUnit {
    kind: u32,
    tree: vec2<u32>,
    extent_inode: vec2<u32>,
    extent_offset: vec2<u32>,
    page_index: vec2<u32>,
};

fn reference_to_an_index_node_of_a_tree(tree: vec2<u32>) -> ReferenceTheWalkFollowsToAUnit {
    return ReferenceTheWalkFollowsToAUnit(W_REFERENCE_TO_AN_INDEX_NODE_OF_A_TREE, tree, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u), vec2<u32>(0u, 0u));
}

fn reference_to_a_packed_container_of_a_tree(tree: vec2<u32>) -> ReferenceTheWalkFollowsToAUnit {
    return ReferenceTheWalkFollowsToAUnit(W_REFERENCE_TO_A_PACKED_CONTAINER_OF_A_TREE, tree, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u), vec2<u32>(0u, 0u));
}

fn reference_to_an_instance_table_page(page_index: vec2<u32>) -> ReferenceTheWalkFollowsToAUnit {
    return ReferenceTheWalkFollowsToAUnit(W_REFERENCE_TO_AN_INSTANCE_TABLE_PAGE, vec2<u32>(0u, 0u), vec2<u32>(0u, 0u), vec2<u32>(0u, 0u), page_index);
}

fn reference_to_a_data_unit_of_an_extent_key(tree: vec2<u32>, extent_inode: vec2<u32>, extent_offset: vec2<u32>) -> ReferenceTheWalkFollowsToAUnit {
    return ReferenceTheWalkFollowsToAUnit(W_REFERENCE_TO_A_DATA_UNIT_OF_AN_EXTENT_KEY, tree, extent_inode, extent_offset, vec2<u32>(0u, 0u));
}

// walk.rs judge_the_cells_the_reference_decides_on_a_unit_walked_earlier：别的版本先走过的单元被这一条引用再走到时，照判按这一条引用定的那几格，
// 判的是这一次读回的那一份（与先走那一遍同一个落点）：
// I-1.6 类那一半：头里的类标签是这条引用要的那一类；不是就不往下判（字段表按类不同）；
// 先走那一遍头用得了时（C_VISITED_HEADER_THE_WALK_COULD_USE）：I-1.2 遍历方向那一格（头里的出生身份与这条指针相同）；I-1.3（码 2 节点的树 ID、
// 码 3 容器与数据单元的出生树是这条引用要的树，实例表豁免）；I-1.1（数据单元五元组是这条 extent key、实例表的身份四元组是这一片的片号）；
// I-1.12（只在先走那一遍判它的两处：实例表、数据单元）。
// 只按单元自己定的那几格不重判（头校验和、格式版本与预留位、I-4.2、容器格式、诞生代号）；I-4.9 照旧在 w_merge_below 里判；
// 按父条目定的层级、key 区间与子树覆盖那几格（`.claude/kb/checks-owed.md` C896）不在这里，由读码 2 节点的调用方经
// w_index_node_an_earlier_walk_parsed 照判。另起的那几遍（上下文 1）判定丢掉，不判。
// 先走那一遍认下的东西按刚才那一次交回 false 的 w_visit 找到的那一行读（w_visited_row_of_the_unit_asked_last）
fn w_judge_the_cells_the_reference_decides_on_a_unit_walked_earlier(s: Slice, p: Ptr, reference: ReferenceTheWalkFollowsToAUnit) {
    if (w_context != 0u) {
        return;
    }
    var expected_class = 3u;
    if (reference.kind == W_REFERENCE_TO_AN_INDEX_NODE_OF_A_TREE) {
        expected_class = 2u;
    } else if (reference.kind == W_REFERENCE_TO_A_DATA_UNIT_OF_AN_EXTENT_KEY) {
        expected_class = 1u;
    }
    let unit_class = rd8(s.base, 6u);
    w_judge(INVARIANT_I_1_6, unit_class == expected_class);
    if (unit_class != expected_class) {
        return;
    }
    if (w_visited_row_of_the_unit_asked_last == NONE) {
        return;
    }
    let found_by_the_first_walk = sw(C_VISITED_BIRTHS0 + w_visited_row_of_the_unit_asked_last * C_VISITED_BIRTH_WORDS);
    if ((found_by_the_first_walk & C_VISITED_HEADER_THE_WALK_COULD_USE) == 0u) {
        return;
    }
    w_judge(INVARIANT_I_1_2, unit_birth_identity_matches_the_pointer(s, p));
    let birth_tree_in_the_header = rd64(s.base, 43u);
    if (reference.kind == W_REFERENCE_TO_AN_INDEX_NODE_OF_A_TREE) {
        // 码 2 节点头里的树 ID 在偏移 42（parse_index_node 的 n.tree）
        if ((found_by_the_first_walk & C_VISITED_INDEX_NODE_PARSES) != 0u) {
            w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 42u), reference.tree));
        }
    } else if (reference.kind == W_REFERENCE_TO_A_PACKED_CONTAINER_OF_A_TREE) {
        w_judge(INVARIANT_I_1_3, eq64(birth_tree_in_the_header, reference.tree));
    } else if (reference.kind == W_REFERENCE_TO_AN_INSTANCE_TABLE_PAGE) {
        w_judge(INVARIANT_I_1_12, eq64(p.birth_tree, birth_tree_in_the_header));
        w_judge(INVARIANT_I_1_1, is_zero64(birth_tree_in_the_header) && rd16(s.base, 51u) == 4u && eq64(rd64(s.base, 53u), reference.page_index)
            && is_zero64(rd64(s.base, 61u)));
    } else {
        w_judge(INVARIANT_I_1_12, eq64(p.birth_tree, birth_tree_in_the_header));
        w_judge(INVARIANT_I_1_3, eq64(birth_tree_in_the_header, reference.tree));
        w_judge(INVARIANT_I_1_1, eq64(rd64(s.base, 51u), reference.extent_inode) && eq64(rd64(s.base, 67u), reference.extent_offset)
            && eq64(birth_tree_in_the_header, reference.tree));
    }
}

// walk.rs note_a_unit_an_earlier_version_walked_referenced_again：别的版本先走过的单元（w_visit 交回 false）又被这一版跟着一条引用走到：
// 这一条引用自己定的那几格照判，不往下展开，它下面的落点照 w_merge_below 并进这一版。五处命中都走这里
fn w_note_a_unit_an_earlier_version_walked_referenced_again(s: Slice, p: Ptr, unit_device: u32, unit_slot: vec2<u32>,
    reference: ReferenceTheWalkFollowsToAUnit) {
    w_judge_the_cells_the_reference_decides_on_a_unit_walked_earlier(s, p, reference);
    w_merge_below(unit_device, unit_slot);
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

// 这一份 Walk 的挂父单元那张表的散列索引起点（桶表、下一行、最后一行三段，crash_judge_checker_common.wgsl 定义 W_BELOW_INDEX0 那一处）
fn w_below_index_base() -> u32 {
    if (w_context == 0u) { return W_BELOW_INDEX0; }
    return W_BELOW_INDEX1;
}

// 这一份 Walk 的挂父单元那张表（键是父盘、父槽 lo/hi）
fn w_below_table() -> KeyedTable {
    return KeyedTable(c_below_base(), C_BELOW_WORDS, 3u, w_below_index_base(), W_BELOW_BUCKET_COUNT);
}

// 开一份新的 Walk（上下文 c）：全部集合清空；判定只在主走读那一份进违例位
fn w_start(context: u32, judging: bool) {
    w_context = context;
    w_judging = judging;
    w_reference_count[context] = 0u;
    w_visited_count[context] = 0u;
    w_in_version_count[context] = 0u;
    w_below_count[context] = 0u;
    w_below_rows_are_indexed[context] = false;
    w_failures = 0u;
    w_mismatches = 0u;
    w_parent_valid = false;
    if (context == 0u) {
        appearance_placement_count = 0u;
        appearance_version_count = 0u;
        appearance_version_placement_count = 0u;
        appearance_walked_tree_table_count = 0u;
        appearance_root_count_of_this_version = 0u;
        appearance_label_stamp = 0u;
        w_leaf_count = 0u;
        w_tree_id_count = 0u;
        w_inode_birth_count = 0u;
        w_inode_births_are_indexed = false;
        w_data_object_count = 0u;
        w_accounting_row_count = 0u;
        w_accounting_seen = false;
        w_instance_row_count = 0u;
        w_instance_rows_are_indexed = false;
        w_inode_tree_walked = false;
        w_have_largest_inode = false;
        w_largest_inode = vec2<u32>(0u, 0u);
        w_mapping_key_unit_count = 0u;
        w_mapping_key_units_are_indexed = false;
        for (var bucket = W_REFERENCE_BUCKETS; bucket < W_BUCKETS_END; bucket = bucket + 1u) {
            sws(bucket, 0u);
        }
    }
}

fn w_begin_version() {
    w_in_version_count[w_context] = 0u;
    w_parent_valid = false;
    w_walking_the_trees_of_a_snapshot = false;
    // 这一版的中央映射在走它的树表条目之前读（w_walk_tree_table_and_mapping）；之前（实例表链、根记录持有的分配记录树）照提示走
    version_mapping_count = 0u;
    version_mapping_is_indexed = false;
    w_hint_is_noted = false;
    appearance_user_visible_tree_is_being_walked = false;
    if (w_context == 0u) {
        appearance_root_count_of_this_version = 0u;
        appearance_tree_table_of_this_version_has_an_entry_without_a_clause = false;
        appearance_label_stamp = appearance_label_stamp + 1u;
    }
}

// 引用集合：(盘, 起点槽, 跨度) 去重记进（这个函数在走读内核里展开二十来份）。主走读那一张按桶表找（W_REFERENCE_BUCKETS），
// 另起的那几遍（上下文 1）的引用集合照旧从头比：那几遍各走一条根、表小。满了记站点 1。
// 位置项校验和不再收：池级 checker 只认隔离记号、不读副本（D19（块指针的结构与宽度预算） 已定项 5），walk.rs 同删
fn w_note_reference(device: u32, slot: vec2<u32>, span: u32) {
    let rows = c_references_base();
    let count = w_reference_count[w_context];
    var present = false;
    var empty_bucket = NONE;
    if (w_context == 0u) {
        let found = hashed_row_of(W_REFERENCE_BUCKETS, W_REFERENCE_BUCKET_COUNT, rows, C_REFERENCE_WORDS, device, slot, span, true);
        present = found.row != NONE;
        empty_bucket = found.empty_bucket;
    } else {
        for (var i = 0u; i < count; i = i + 1u) {
            if (sw(rows + i * C_REFERENCE_WORDS) == device && sw(rows + i * C_REFERENCE_WORDS + 1u) == slot.x
                && sw(rows + i * C_REFERENCE_WORDS + 2u) == slot.y && sw(rows + i * C_REFERENCE_WORDS + 3u) == span) {
                present = true;
                break;
            }
        }
    }
    if (present) {
        return;
    }
    if (count >= C_MAX_REFERENCES) {
        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (1u << 8u);
        return;
    }
    sws(rows + count * C_REFERENCE_WORDS, device);
    sws(rows + count * C_REFERENCE_WORDS + 1u, slot.x);
    sws(rows + count * C_REFERENCE_WORDS + 2u, slot.y);
    sws(rows + count * C_REFERENCE_WORDS + 3u, span);
    if (empty_bucket != NONE) {
        sws(W_REFERENCE_BUCKETS + empty_bucket, count + 1u);
    }
    sws(rows + count * C_REFERENCE_WORDS + 4u, 0u);
    w_reference_count[w_context] = count + 1u;
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

// 最近一次 w_visit 问的那个单元在主走读走过的单元表里的下标：先走过（交回 false）是它原来那一行，这一次记进去是新的那一行；
// 另起的那几遍（上下文 1）与表满了的那一次是 NONE。w_visit 之后、下一次 w_visit 之前读它的两处：命中时照判那几格
// （w_judge_the_cells_the_reference_decides_on_a_unit_walked_earlier）、先走那一遍记认下的东西（w_note_on_the_visited_row_what_the_first_walk_found）；
// 记一个下标，那两处不再各展开一份按桶表找
var<private> w_visited_row_of_the_unit_asked_last: u32;

// visited_units.insert：已有交回 false。主走读那一张按桶表 W_VISITED_BUCKETS 找，另起的那几遍照旧从头比
fn w_visit(device: u32, slot: vec2<u32>) -> bool {
    var visited_bucket = NONE;
    w_visited_row_of_the_unit_asked_last = NONE;
    if (w_context == 0u) {
        let found = hashed_row_of(W_VISITED_BUCKETS, W_VISITED_BUCKET_COUNT, C_VISITED0, 3u, device, slot, 0u, false);
        if (found.row != NONE) {
            w_visited_row_of_the_unit_asked_last = found.row;
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
    if (w_context == 0u) {
        w_visited_row_of_the_unit_asked_last = count;
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

// 走读此刻在不在走活快照的两棵树（walk.rs 的 TreeSharingInTheWalk::SnapshotTreesSharingUnitsWithTheHead，判决
// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3）：它们与活头、与别的活快照共享没改过的单元是合法的 COW——同一版里再遇到同一个落点
// 不判第二次引用（落点照样挂到父单元下面，这一版已经从活头记过的也挂）；活头那几样累计（inode 号最大值、对象出生代、数据单元的对象、走没走到 inode 树）不收。
// w_walk_tree_table_entry 走码 14、15 那两种条目时设、每条条目开头与一版的树表条目走完时清
var<private> w_walking_the_trees_of_a_snapshot: bool;

// I-5.1「同一版第二次引用同一个落点」：这一版引用过就红，没有就记下（walk.rs note_a_placement_referenced_in_this_version）。
// 走活快照那两棵树时不判，这一版还没记过才记下
fn w_judge_placement_in_version(device: u32, slot: vec2<u32>) {
    let base = c_in_version_base();
    let count = w_in_version_count[w_context];
    for (var i = 0u; i < count; i = i + 1u) {
        if (sw(base + i * 3u) == device && sw(base + i * 3u + 1u) == slot.x && sw(base + i * 3u + 2u) == slot.y) {
            if (!w_walking_the_trees_of_a_snapshot) {
                w_judge(INVARIANT_I_5_1, false);
            }
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

// 一个别的版本已经走过的单元又被这一版引用：把它下面的落点（孩子、孩子的孩子……）逐个并进这一版的集合。显式栈与展开过的单元两张表
// （C_MERGE_PENDING、C_MERGE_EXPANDED）各 C_MAX_BELOW + 1 条，为什么够见 crash_judge_checker_common.wgsl 定义它们那一处。
// 去重按展开过的单元那张表的散列索引找，孩子按挂父单元那张表的散列索引从父单元的第一行沿着下一行走（同一处之后那一段）：
// 两样都与从表头比到表尾找到的行与次序相同，判法、次序、站点码不变
fn w_merge_below(device: u32, slot: vec2<u32>) {
    var pending_count = 1u;
    sws(C_MERGE_PENDING, device);
    sws(C_MERGE_PENDING + 1u, slot.x);
    sws(C_MERGE_PENDING + 2u, slot.y);
    var expanded_count = 0u;
    var expanded_units_are_indexed = false;
    let expanded_table = KeyedTable(C_MERGE_EXPANDED, C_MERGE_UNIT_WORDS, 3u, W_MERGE_EXPANDED_BUCKETS, W_MERGE_EXPANDED_BUCKET_COUNT);
    let below = c_below_base();
    let below_count = w_below_count[w_context];
    let below_table = w_below_table();
    let below_rows_are_indexed = w_below_rows_are_indexed[w_context];
    let below_next = w_below_index_base() + W_BELOW_NEXT_OFFSET;
    while (pending_count > 0u) {
        pending_count = pending_count - 1u;
        let unit_device = sw(C_MERGE_PENDING + pending_count * C_MERGE_UNIT_WORDS);
        let unit_slot = vec2<u32>(sw(C_MERGE_PENDING + pending_count * C_MERGE_UNIT_WORDS + 1u), sw(C_MERGE_PENDING + pending_count * C_MERGE_UNIT_WORDS + 2u));
        let unit_key = vec4<u32>(unit_device, unit_slot.x, unit_slot.y, 0u);
        if (keyed_first_row_of(expanded_table, expanded_count, expanded_units_are_indexed, unit_key) != NONE) {
            continue;
        }
        if (expanded_count >= C_MAX_MERGE_EXPANDED) {
            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (5u << 8u);
            return;
        }
        sws(C_MERGE_EXPANDED + expanded_count * C_MERGE_UNIT_WORDS, unit_device);
        sws(C_MERGE_EXPANDED + expanded_count * C_MERGE_UNIT_WORDS + 1u, unit_slot.x);
        sws(C_MERGE_EXPANDED + expanded_count * C_MERGE_UNIT_WORDS + 2u, unit_slot.y);
        expanded_count = expanded_count + 1u;
        expanded_units_are_indexed = keyed_table_after_a_row_was_appended(expanded_table, expanded_count, expanded_units_are_indexed,
            expanded_count - 1u);
        // I-4.9：别的版本先走过的单元按记下的诞生代号判（merge_placements_below_a_unit_walked_by_an_earlier_version）
        if (w_version_is_an_older_candidate_root && w_context == 0u) {
            let visited_index = w_visited_index_in_the_main_walk(unit_device, unit_slot);
            if (visited_index != NONE && w_birth_mark_of_a_visited_row(visited_index) == 1u) {
                w_judge_unit_birth_against_an_older_candidate_root(vec2<u32>(
                    sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u),
                    sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u)));
            }
        }
        // 下一行只指向更靠后的行，合并当中不记新行：至多走 below_count 步；NONE 大于任何条数，也在这一句里收尾
        for (var below_row = keyed_first_row_of(below_table, below_count, below_rows_are_indexed, unit_key); below_row < below_count;
            below_row = sw(below_next + below_row) - 1u) {
            let child_device = sw(below + below_row * C_BELOW_WORDS + 3u);
            let child_slot = vec2<u32>(sw(below + below_row * C_BELOW_WORDS + 4u), sw(below + below_row * C_BELOW_WORDS + 5u));
            w_judge_placement_in_version(child_device, child_slot);
            if (w_context == 0u) {
                appearance_label_a_merged_placement_in_this_version(child_device, child_slot);
            }
            if (pending_count >= C_MAX_MERGE_PENDING) {
                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (6u << 8u);
                return;
            }
            sws(C_MERGE_PENDING + pending_count * C_MERGE_UNIT_WORDS, child_device);
            sws(C_MERGE_PENDING + pending_count * C_MERGE_UNIT_WORDS + 1u, child_slot.x);
            sws(C_MERGE_PENDING + pending_count * C_MERGE_UNIT_WORDS + 2u, child_slot.y);
            pending_count = pending_count + 1u;
        }
    }
}

// 走读跟着一条指针往下：两条位置条目记进引用集合，判同一版第二次引用，记到父单元下面。
// 八步照原来的次序走：两条位置条目的落点（I-3.5，主走读）、第 0 条的引用、同一版第二次引用、父单元，第 1 条的同样三样（与第 0 条同落点时后两样不做）。
// 落点一律挂到父单元下面，走活快照那两棵树时这一版已经从活头记过的也挂（walk.rs note_references_of_a_pointer_followed_by_the_walk）：
// 父单元在这一版只从快照那两棵树走得到，在先建快照的那几版里却是活头自己的单元，那几版不重走它、只按这张表并进它下面的落点，
// 少挂一个，那几版的引用集合就少一个落点、走读又不记失败，I-3.5 把它当成后一版新出现的（判决
// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9「种子下标 1032 那一形」）。
// 次序不改（几张表同时满时 undecidable 留最后写的那个站点）；每样只有一个调用点：这个函数在走读内核里展开二十来份，原来每份里每样两份拷贝
// 自举豁免的三类、类型段不认得的孩子、孩子自证成另一类的那几处：落点不按中央映射 key 折（walk.rs 的 PositionAuthority::ThePointerItself），
// 这两个落点折成这条指针自己的第 0 条位置
fn w_note_pointer_references(p: Ptr, span: u32) {
    w_note_pointer_references_of_a_located_pointer(p, span, MappingKeyUnit(p.device0, p.slot0), false);
}

// 按查找路径给了类的一条指针（已经 w_locate 认过位置）：这两个落点折成哪个单元（walk.rs note_references_of_a_pointer_followed_by_the_walk 的 unit）——
// 按中央映射 key 折成第一次记下这把 key 时那条指针认的第 0 条位置（搬前、搬后两个落点是同一个单元）；只在主走读折（I-3.5 只看主走读），
// 类是 0 的是这条指针自己的第 0 条位置
fn w_unit_of_a_located_pointer(p: Ptr, mapping_class: u32) -> MappingKeyUnit {
    let first = MappingKeyUnit(p.device0, p.slot0);
    if (w_context != 0u || mapping_class == MAPPING_CLASS_THE_POINTER_ITSELF) {
        return first;
    }
    return w_unit_of_the_mapping_key(mapping_key_of_a_pointer(p, mapping_class), first);
}

// p 已按 w_locate 认过位置（豁免三类之外是映射位置）：引用集合记的是映射位置、不是提示（判决 G3「引用集合记映射位置」）；unit 是两个落点折成的单元，
// folded_by_the_mapping_key 是不是按中央映射 key 折的（walk.rs 的 PositionAuthority::TheCentralMapping）
fn w_note_pointer_references_of_a_located_pointer(p: Ptr, span: u32, unit: MappingKeyUnit, folded_by_the_mapping_key: bool) {
    let second_location_is_the_first = p.device1 == p.device0 && eq64(p.slot1, p.slot0);
    for (var step = 0u; step < 8u; step = step + 1u) {
        let on_the_second_location = step == 1u || step >= 5u;
        let device = select(p.device0, p.device1, on_the_second_location);
        let slot = select(p.slot0, p.slot1, on_the_second_location);
        if (step < 2u) {
            if (w_context == 0u) {
                appearance_note_placement(device, slot, unit.device, unit.slot, folded_by_the_mapping_key);
            }
        } else if (step == 2u || step == 5u) {
            w_note_reference(device, slot, span);
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
    // 这个父单元在记这一行之前的第一行：有就把这一行接到它那几行的最后一行后面（w_merge_below 沿着下一行走，次序同记进的次序）
    let index = w_below_index_base();
    let first_row_of_the_parent = keyed_first_row_of(w_below_table(), count, w_below_rows_are_indexed[w_context],
        vec4<u32>(w_parent_device, w_parent_slot.x, w_parent_slot.y, 0u));
    let base = c_below_base();
    sws(base + count * C_BELOW_WORDS, w_parent_device);
    sws(base + count * C_BELOW_WORDS + 1u, w_parent_slot.x);
    sws(base + count * C_BELOW_WORDS + 2u, w_parent_slot.y);
    sws(base + count * C_BELOW_WORDS + 3u, device);
    sws(base + count * C_BELOW_WORDS + 4u, slot.x);
    sws(base + count * C_BELOW_WORDS + 5u, slot.y);
    sws(index + W_BELOW_NEXT_OFFSET + count, 0u);
    if (first_row_of_the_parent == NONE) {
        sws(index + W_BELOW_LAST_OFFSET + count, count);
    } else {
        sws(index + W_BELOW_NEXT_OFFSET + sw(index + W_BELOW_LAST_OFFSET + first_row_of_the_parent), count + 1u);
        sws(index + W_BELOW_LAST_OFFSET + first_row_of_the_parent, count);
    }
    w_below_count[w_context] = count + 1u;
    w_below_rows_are_indexed[w_context] = keyed_table_after_a_row_was_appended(w_below_table(), count + 1u,
        w_below_rows_are_indexed[w_context], count);
}

// w_read_referenced_unit 最近一次交回的是第二条位置条目那一份（第一条那一份对不上、第二条对得上）：「走过」按读回那一份的落点认的几处读它
// （w_visit_the_copy_read_back）
var<private> w_read_back_from_the_second_location: bool;

// w_read_unit_to_walk 最近一次读回、自证过的那一份的落点：这个单元在走读里的身份（walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from
// 交回的落点；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，攻方 allrbmap）。走过、诞生代号、并进前一版、
// 父单元、叶序与 I-3.5 的折叠表都按它认。两份都读不出时是第一条位置条目的落点（调用方不用它）
var<private> w_unit_device_read_back: u32;
var<private> w_unit_slot_read_back: vec2<u32>;
