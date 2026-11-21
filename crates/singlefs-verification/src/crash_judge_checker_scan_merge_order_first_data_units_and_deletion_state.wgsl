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

// 码 1 的归并键不换次序（[42, 101) 原样）：对象 key（五元组）是前 33 个字节；全序键 (诞生代号, 实例代号, 事务号) 是诞生代号 [75, 83) 与写序
// [91, 101) 两段，中间隔着 fsid [83, 91)，不是一段前缀，逐段比
const C_OBJECT_KEY_BYTES_OF_A_DATA_UNIT: u32 = 33u;

// 扫描项按归并键排好的次序借 C_ROW_ORDER（至多 C_MAX_SCANNED 个，那张表 C_MAX_REFERENCES 个字）；每一项读出来那一份切片的检查字（c2）
// 借 C_JOURNAL_GROUP_KEYS 的前 C_MAX_SCANNED 个字（那一段 C_JOURNAL_GROUP_KEY_WORDS × MAX_RECORDS 个字，crash_judge_checker_common.wgsl ⑯）。I-1.8 是扫描内核第一判，
// 之后墓碑两两比那一处借 C_ROW_ORDER、各自从头写（I-5.4、I-3.16 排分配记录与 I-3.9、I-3.10 的两张桶表借 crash_judge_checker_common.wgsl ⑮ 的两段）
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

// 扫描项的对象 key 在换过次序的归并键里占前几个字节
fn c_object_key_bytes(unit_class: u32) -> u32 {
    return select(C_OBJECT_KEY_BYTES, C_OBJECT_KEY_BYTES_OF_A_DATA_UNIT, unit_class == 1u);
}

// 两个同类、同对象 key 的扫描项（字节基）全序键 (诞生代号, 实例代号, 事务号) 相同（walk.rs TotalOrderKey，码 1 与码 3 同形，C464 定 ①）。
// 码 3 的全序键在换过次序的归并键里紧接对象 key，公共前缀够长就是相同；码 1 的隔着 fsid，逐段比诞生代号与写序
fn c_total_order_keys_are_equal(base_a: u32, base_b: u32, unit_class: u32, common_merge_key_prefix: u32) -> bool {
    if (unit_class == 3u) {
        return common_merge_key_prefix >= C_OBJECT_KEY_AND_TOTAL_ORDER_KEY_BYTES;
    }
    return eq64(rd64(base_a, 75u), rd64(base_b, 75u)) && rd32(base_a, 91u) == rd32(base_b, 91u)
        && eq64(rd48(base_a, 95u), rd48(base_b, 95u));
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
    // ② 那一串（同对象 key 的各组，码 1 与码 3 都判）：上一组的代表、这一串几组、串里有没有两组全序键相同
    var previous_representative = NONE;
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
        // ② 同对象 key 的各组两两全序键不等（码 1 与码 3 都判，码 1 排在码 3 前面，两类之间公共前缀是 0、串在类的边界上断开）：
        // 同对象 key 的组连成一串、串里全序键相同的组挨在一起（码 3 的全序键紧接对象 key；码 1 的归并键里对象 key 之后是诞生代号、fsid、写序，
        // 扫描项的 fsid 都等于本池的，全序键相同的两组归并键也相同、是同一组），比相邻两组的代表就够；一串两组以上判一次
        var same_object_key = false;
        if (previous_representative != NONE) {
            let prefix = c_scanned_units_common_merge_key_prefix(previous_representative, representative);
            same_object_key = prefix >= c_object_key_bytes(unit_class);
            if (same_object_key && c_total_order_keys_are_equal(sw(C_SCANNED + previous_representative),
                sw(C_SCANNED + representative), unit_class, prefix)) {
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
        previous_representative = representative;
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
// mapping_root_offset 起是这一版的中央映射根指针（候选根取根记录偏移 256，由记录施加出来的那一版取记录新根段偏移 86）：extent 树、
// 分配记录树的节点与数据单元按它认位置
fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32, txg: vec2<u32>, mapping_root_offset: u32) {
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
    s_use_the_central_mapping_named_at(base, mapping_root_offset);
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
        var mapping_root_offset = G_ROOT_RECORD_MAPPING_ROOT_POINTER;
        var txg = vec2<u32>(0u, 0u);
        if (version_index < c_candidate_count) {
            let root = checker_root_of(sw(C_CANDIDATES + version_index));
            base = root.base;
            txg = root.txg;
        } else {
            let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
            base = record.base;
            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
            mapping_root_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + G_NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER;
            txg = record.txg;
        }
        judge_first_data_unit_of_a_version(base, tree_table_offset, txg, mapping_root_offset);
    }
}

// ───────────────────────── 删除态扫描（walk.rs judge_deletion_state_of_the_versions）与 I-9.6 墓碑那一半 ─────────────────────────
// I-9.5、I-3.4、I-4.7、I-4.6、I-4.10、I-9.38、I-9.24 在回退候选集里每条根与由记录施加出来的那几版上各重读一遍树表与几棵树（只读）。CPU 记成射程里判不了的那几格
// （树表读不出、条目宽不到 200、deleted_inodes 的头 ID 配不上恰好一条 inode 树、队列根读不出或形状不对、inode 树 / extent 树 / 记账树读不全、
// 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
// I-4.4（logged_ops 条目，整理与截断两种）在引用与快照那一截判（g_judge_the_compaction_intents_of_the_version）。

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

// 这一版的一条 inode 记录记进 D_INODE_RECORDS：同一个号已有一行就盖掉（walk.rs 是按号插入的 BTreeMap，后读到的盖掉先读到的）；表满交回 false。
// 截断字段 T 与 N_w 两段照记（walk.rs InodeRecordOfTheDeletionScan 的 truncation_field_txg、truncation_field_size_in_bytes，I-9.39 读）
fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>, object_birth: vec2<u32>,
    truncation_field_txg: vec2<u32>, truncation_field_size: vec2<u32>) -> bool {
    let existing = d_inode_record_row_of(inode);
    if (existing != NONE) {
        sws(existing + 2u, nlink);
        sws(existing + 3u, size_in_bytes.x);
        sws(existing + 4u, size_in_bytes.y);
        sws(existing + 5u, object_birth.x);
        sws(existing + 6u, object_birth.y);
        sws(existing + 7u, truncation_field_txg.x);
        sws(existing + 8u, truncation_field_txg.y);
        sws(existing + 9u, truncation_field_size.x);
        sws(existing + 10u, truncation_field_size.y);
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
    sws(row + 7u, truncation_field_txg.x);
    sws(row + 8u, truncation_field_txg.y);
    sws(row + 9u, truncation_field_size.x);
    sws(row + 10u, truncation_field_size.y);
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

// D_INODE_RECORDS 里一行的截断字段 T 与 N_w
fn d_inode_record_truncation_field_txg(row: u32) -> vec2<u32> {
    return vec2<u32>(sw(row + 7u), sw(row + 8u));
}

fn d_inode_record_truncation_field_size(row: u32) -> vec2<u32> {
    return vec2<u32>(sw(row + 9u), sw(row + 10u));
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

// 这一版 extent 树里每个号的数据单元那两张表（r_note_data_unit_of_inode 记）。借引用与快照那一截的 I-1.14 跨版本那张 key 表（G_KEY_GROUPS 起）那一段：
// 扫描内核主入口里删除态扫描在 I-3.16 之前跑，I-3.16 借 S_KEYED_UNITS 起的那一段也从头写，引用与快照那一截的几张表都从头写、写过才读，
// 不读这里留下的字；主入口里 G_KEY_GROUPS 之前那几张（logged_ops 叶条目、读树的栈）截断条目那一判照它们自己的用法借。
// ① 每个号一行：号 lo/hi、单元数、上段叶条目的声明（D_DECLARED_*）、它最后记进的那个单元在 D_DATA_UNITS 里的下标 + 1（0 是没有单元）
const D_DATA_UNITS_OF_AN_INODE_WORDS: u32 = 5u;
const D_DATA_UNITS_OF_EACH_INODE: u32 = G_KEY_GROUPS;
// ② 每个单元一行：文件字节偏移 lo/hi、数据指针所在的打包表字节基与偏移、同号前一个记进的单元的下标 + 1（0 是没有）
const D_DATA_UNIT_WORDS: u32 = 5u;
const D_MAX_DATA_UNITS: u32 = 8192u;
const D_DATA_UNITS: u32 = D_DATA_UNITS_OF_EACH_INODE + D_DATA_UNITS_OF_AN_INODE_WORDS * D_MAX_DATA_UNIT_INODES;
// ③ I-4.6 ① 判一个号的单元连号时，单元序号 k（偏移 ÷ 净荷）见过没有的位表：位 k 在第 k / 32 字的第 k % 32 位，一个号一个号地清
const D_DATA_UNIT_POSITIONS_SEEN: u32 = D_DATA_UNITS + D_DATA_UNIT_WORDS * D_MAX_DATA_UNITS;
// 上界是 ⑯ 的起点（journal 记录表挪到末尾之前的草稿区末端）：⑯ 那三张扫描内核要读，借的这一段不越进去
const_assert D_DATA_UNIT_POSITIONS_SEEN + D_MAX_DATA_UNITS / 32u <= R_RECORDS;
const_assert G_PENDING + G_PENDING_WORDS * G_MAX_PENDING <= D_DATA_UNITS_OF_EACH_INODE;
// 单元多于 D_MAX_DATA_UNITS 个判不了（「条目太多」一类的站点号，与别处不撞）
const D_SITE_DATA_UNITS_PAST_THE_TABLE: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (105u << 8u);

// 上段叶条目对一个号的声明（walk.rs DataUnitsDeclaredByTheUpperLeafEntry 加「没有上段叶条目」）
const D_DECLARED_BY_NO_UPPER_LEAF_ENTRY: u32 = 0u;
const D_DECLARED_NO_DATA_UNIT: u32 = 1u;
const D_DECLARED_DATA_UNITS_WITHOUT_A_HOLE_AFTER_THEM: u32 = 2u;
const D_DECLARED_A_HOLE_UP_TO_THE_SIZE_AFTER_THE_LAST_DATA_UNIT: u32 = 3u;

// 截断条目的种类码与 value 里各段的偏移（walk.rs LOGGED_OPERATION_KIND_TRUNCATION 与 TRUNCATION_INTENT_*_OFFSET_IN_THE_VALUE；判决 m4-trunc-r1 K1：
// value 34 = inode 号 8、对象出生代 8、目标 size 8、补零 10）
const G_LOGGED_OPERATION_KIND_TRUNCATION: u32 = 2u;
const G_TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE: u32 = 0u;
const G_TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE: u32 = 8u;
const G_TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE: u32 = 16u;
const G_TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE: u32 = 24u;
const G_TRUNCATION_INTENT_PADDING_BYTES: u32 = 10u;

// 数据单元头里声明长度那一段的偏移（walk.rs DATA_UNIT_DECLARED_LENGTH_OFFSET，D18（块里携带什么信息） 已定项 7 字段表：码 1 偏移 8，2 字节）
const D_DATA_UNIT_DECLARED_LENGTH_OFFSET: u32 = 8u;

// 数据单元头里诞生代号那 8 字节的偏移（walk.rs 经 ContentUnitClass::Data.header_offsets().birth_txg，同一张字段表：码 1 偏移 75）
const D_DATA_UNIT_BIRTH_TXG_OFFSET: u32 = 75u;

// 一条记录的 size 要几个数据单元：⌈size ÷ 净荷容量⌉.max(1)（walk.rs data_units_the_size_asks_for）
fn d_data_units_the_size_asks_for(size_in_bytes: vec2<u32>) -> vec2<u32> {
    let division = divmod64(size_in_bytes, u64_of(DATA_UNIT_PAYLOAD_CAPACITY));
    var asked_for = division.quotient;
    if (!is_zero64(division.remainder)) {
        asked_for = add64(asked_for, u64_of(1u));
    }
    if (is_zero64(asked_for)) {
        asked_for = u64_of(1u);
    }
    return asked_for;
}

// g_read_the_logged_operation_entries 收进 G_LOGGED_OPERATION_ENTRIES 的条目里有没有一条截断条目对得上这条记录（walk.rs truncation_intent_names_the_record）：
// inode 号、对象出生代相同，目标 = 记录的 size
fn d_a_truncation_intent_names_the_record(inode: vec2<u32>, object_birth: vec2<u32>, size_in_bytes: vec2<u32>) -> bool {
    for (var entry_index = 0u; entry_index < g_logged_operation_entry_count; entry_index = entry_index + 1u) {
        let base = sw(G_LOGGED_OPERATION_ENTRIES + entry_index * 2u);
        let o = sw(G_LOGGED_OPERATION_ENTRIES + entry_index * 2u + 1u);
        let value = o + LOGGED_OPERATIONS_KEY_BYTES;
        if (rd16(base, o) == G_LOGGED_OPERATION_KIND_TRUNCATION
            && eq64(rd64(base, value + G_TRUNCATION_INTENT_INODE_OFFSET_IN_THE_VALUE), inode)
            && eq64(rd64(base, value + G_TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE), object_birth)
            && eq64(rd64(base, value + G_TRUNCATION_INTENT_TARGET_SIZE_OFFSET_IN_THE_VALUE), size_in_bytes)) {
            return true;
        }
    }
    return false;
}

// 一个号存在的单元从 0 起连号（walk.rs judge_data_unit_counts_of_the_inodes_against_their_size ①：按偏移升序第 i 个的偏移是 i × 净荷容量）：
// 等价于每个单元的偏移都是净荷容量的整数倍 k、k 小于单元数、两两不同——单元的偏移集合恰是 {0, 净荷, …, (单元数 − 1) × 净荷}
fn d_data_units_of_the_inode_are_in_sequence_from_zero(first_unit_plus_one: u32, data_unit_count: u32) -> bool {
    for (var word = 0u; word < (data_unit_count + 31u) / 32u; word = word + 1u) {
        sws(D_DATA_UNIT_POSITIONS_SEEN + word, 0u);
    }
    var unit_plus_one = first_unit_plus_one;
    // 迭代上界：链表每个单元只记进一次（r_note_data_unit_of_inode 只往表头接），至多 D_MAX_DATA_UNITS 步
    for (var step = 0u; step < D_MAX_DATA_UNITS && unit_plus_one != 0u; step = step + 1u) {
        let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
        let division = divmod64(vec2<u32>(sw(unit), sw(unit + 1u)), u64_of(DATA_UNIT_PAYLOAD_CAPACITY));
        if (!is_zero64(division.remainder) || !lt64(division.quotient, u64_of(data_unit_count))) {
            return false;
        }
        let position = division.quotient.x;
        let word = D_DATA_UNIT_POSITIONS_SEEN + position / 32u;
        let bit = 1u << (position % 32u);
        if ((sw(word) & bit) != 0u) {
            return false;
        }
        sws(word, sw(word) | bit);
        unit_plus_one = sw(unit + 4u);
    }
    return true;
}

// I-4.6（walk.rs judge_data_unit_counts_of_the_inodes_against_their_size，判决 m4-trunc-r1 K7 T-3 与第六节 S1）、I-4.10（judge_data_units_beyond_the_size_against_the_truncation_intents，
// T-2，只判「多出来就要有对得上的截断条目」那一个方向，第六节 S2）、I-9.38（judge_declared_lengths_of_the_data_units_within_the_size，T-4）、
// I-9.39（judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field，判决 m4-trunc-r2 第三节 R6）：
// 这一版 inode 树的每条记录（nlink = 0 与 ≥ 1 都判）。tt 是这一版的树表，读截断条目用；读不全的 I-4.6、I-4.10 判不了，I-9.38、I-9.39 照判
fn d_judge_the_data_units_of_each_record(tt: Node) {
    // 这一版 logged_ops 里的条目（walk.rs truncation_intents_of_a_version_without_judging：树表里种类 12 那几条指着的根逐棵读到叶）
    g_logged_operation_entry_count = 0u;
    var truncation_intents_read_in_full = true;
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        let root = parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LOGGED_OPERATIONS && !root.all_zero
            && !g_read_the_logged_operation_entries(root)) {
            truncation_intents_read_in_full = false;
        }
    }
    if (!truncation_intents_read_in_full) {
        mark_part_of_the_range_not_judged(INVARIANT_I_4_6);
        mark_part_of_the_range_not_judged(INVARIANT_I_4_10);
    }
    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
    for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
        let record = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
        let inode = vec2<u32>(sw(record), sw(record + 1u));
        let size_in_bytes = vec2<u32>(sw(record + 3u), sw(record + 4u));
        let asked_for = d_data_units_the_size_asks_for(size_in_bytes);
        let row = d_data_units_row_of_inode(inode);
        var data_unit_count = 0u;
        var declaration = D_DECLARED_BY_NO_UPPER_LEAF_ENTRY;
        var first_unit_plus_one = 0u;
        if (row != NONE) {
            data_unit_count = sw(row + 2u);
            declaration = sw(row + 3u);
            first_unit_plus_one = sw(row + 4u);
        }
        if (truncation_intents_read_in_full) {
            judge(INVARIANT_I_4_6, d_data_units_of_the_inode_are_in_sequence_from_zero(first_unit_plus_one, data_unit_count));
            let names_a_truncation_intent = d_a_truncation_intent_names_the_record(inode, d_inode_record_object_birth(record), size_in_bytes);
            let existing = u64_of(data_unit_count);
            // 单元数对标签：标签 1、2 等于 size 要的个数，有一条对得上的截断条目时 size 之外可以多；标签 3、4 至少一个、少于它（至少一个洞单元）；
            // 标签 0 的号一个单元都没有；没有上段叶条目的号一个单元都没有、而且 size 是 0（判决 m4-multi-inode-size-r1 第二节 S2 与第三节第 4 句：
            // 「有记录、没有条目、size > 0」与 CPU 一侧同红，walk.rs 那一判的 None 那一支）
            var count_matches_the_declaration = data_unit_count == 0u;
            if (declaration == D_DECLARED_BY_NO_UPPER_LEAF_ENTRY) {
                count_matches_the_declaration = data_unit_count == 0u && is_zero64(size_in_bytes);
            } else if (declaration == D_DECLARED_DATA_UNITS_WITHOUT_A_HOLE_AFTER_THEM) {
                count_matches_the_declaration = eq64(existing, asked_for) || (names_a_truncation_intent && le64(asked_for, existing));
            } else if (declaration == D_DECLARED_A_HOLE_UP_TO_THE_SIZE_AFTER_THE_LAST_DATA_UNIT) {
                count_matches_the_declaration = data_unit_count >= 1u && lt64(existing, asked_for);
            }
            judge(INVARIANT_I_4_6, count_matches_the_declaration);
            judge(INVARIANT_I_4_10, le64(existing, asked_for) || names_a_truncation_intent);
        }
        // I-9.38：size 之内（偏移 ÷ 净荷 < size 要的个数）的每个单元，单元头偏移 8 的声明长度 = min(size − 起点, 净荷容量)；单元读不出的判不了
        var unit_plus_one = first_unit_plus_one;
        for (var step = 0u; step < D_MAX_DATA_UNITS && unit_plus_one != 0u; step = step + 1u) {
            let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
            unit_plus_one = sw(unit + 4u);
            let offset_in_bytes = vec2<u32>(sw(unit), sw(unit + 1u));
            if (!lt64(divmod64(offset_in_bytes, payload).quotient, asked_for)) {
                continue;
            }
            let s = r_read_located_unit(parse_data_pointer(sw(unit + 2u), sw(unit + 3u)), MAPPING_CLASS_DATA_UNIT, DATA_UNIT_BYTES);
            if (s.version == NONE) {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_38);
                continue;
            }
            var expected_length = sub64_saturating(size_in_bytes, offset_in_bytes);
            if (lt64(payload, expected_length)) {
                expected_length = payload;
            }
            judge(INVARIANT_I_9_38, eq64(u64_of(rd16(s.base, D_DATA_UNIT_DECLARED_LENGTH_OFFSET)), expected_length));
        }
        d_judge_the_birth_txgs_of_the_live_data_units_beyond_the_truncation_field(record, first_unit_plus_one, asked_for);
    }
}

// I-9.39（walk.rs judge_birth_txgs_of_the_live_data_units_beyond_the_truncation_field）：一条记录判一次。截断字段 T = 0 的照判成立；
// T ≠ 0 时，extent 树里存在、在 size 之内（单元号 < size 要的个数 asked_for）、单元号不小于 ⌈N_w ÷ 净荷⌉.max(1) 的数据单元，单元头诞生代号
// 都不早于 T。单元两份都读不出的记射程里判不了、不拿它判（读不出归 I-2.1）
fn d_judge_the_birth_txgs_of_the_live_data_units_beyond_the_truncation_field(record: u32, first_unit_plus_one: u32, asked_for: vec2<u32>) {
    let truncation_field_txg = d_inode_record_truncation_field_txg(record);
    var every_live_unit_is_born_no_earlier = true;
    if (!is_zero64(truncation_field_txg)) {
        let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
        let kept_by_the_truncation_field = d_data_units_the_size_asks_for(d_inode_record_truncation_field_size(record));
        var unit_plus_one = first_unit_plus_one;
        // 迭代上界：同 I-9.38 那一圈，链表每个单元只记进一次，至多 D_MAX_DATA_UNITS 步
        for (var step = 0u; step < D_MAX_DATA_UNITS && unit_plus_one != 0u; step = step + 1u) {
            let unit = D_DATA_UNITS + (unit_plus_one - 1u) * D_DATA_UNIT_WORDS;
            unit_plus_one = sw(unit + 4u);
            let unit_number = divmod64(vec2<u32>(sw(unit), sw(unit + 1u)), payload).quotient;
            if (lt64(unit_number, kept_by_the_truncation_field) || !lt64(unit_number, asked_for)) {
                continue;
            }
            let s = r_read_located_unit(parse_data_pointer(sw(unit + 2u), sw(unit + 3u)), MAPPING_CLASS_DATA_UNIT, DATA_UNIT_BYTES);
            if (s.version == NONE) {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_39);
                continue;
            }
            if (lt64(rd64(s.base, D_DATA_UNIT_BIRTH_TXG_OFFSET), truncation_field_txg)) {
                every_live_unit_is_born_no_earlier = false;
            }
        }
    }
    judge(INVARIANT_I_9_39, every_live_unit_is_born_no_earlier);
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

// D_INODE_RECORDS（连同条数、桶表）此刻装的是不是一次读全了的读、读的是哪一条根指针：那一张只由 d_read_inode_records 写，
// 读全了的读只看根指针（逐段相同）、装着的那一版中央映射与镜像，所以同一条根指针在同一份映射下再读一次，读出的与表里此刻装的逐字相同，不重读
// （墓碑与谱系那一遍一版里先逐棵 inode 树读一遍记 I-9.29，再给每棵墓碑树配的那棵、I-9.18 每棵队列不空的那棵各读一遍，配的多半是同一棵）。
// 映射换了（s_use_the_central_mapping_of 重读、r_references_of_root 清空）、删除态扫描与墓碑、谱系那一遍每进一版，都忘掉
var<private> d_inode_records_hold_a_whole_read: bool;
var<private> d_inode_records_read_from: Ptr;

fn d_forget_the_inode_records_read_in_full() {
    d_inode_records_hold_a_whole_read = false;
}

// 一版 inode 树的全部记录读进 D_INODE_RECORDS（d_read_inode_records_from_the_nodes_of_the_tree）；表里此刻装的就是同一条根指针在同一份映射下读全了的那一次时不重读，照交回读全了
fn d_read_inode_records(root_pointer: Ptr) -> bool {
    if (d_inode_records_hold_a_whole_read && root_pointer.all_zero == d_inode_records_read_from.all_zero
        && same_pointer(root_pointer, d_inode_records_read_from)) {
        return true;
    }
    let read_in_full = d_read_inode_records_from_the_nodes_of_the_tree(root_pointer);
    d_inode_records_hold_a_whole_read = read_in_full;
    d_inode_records_read_from = root_pointer;
    return read_in_full;
}

// walk.rs inode_records_without_judging：一版 inode 树的全部记录读进 D_INODE_RECORDS。根指针全零是零条记录；根层级 0 只许是零条目根；
// 层级 ≥ 1 的往下走：类型段 2 的叶容器（码 3、记录类型 2、记录宽 ≥ 140）读每条记录，类型段 0 的内部节点（子指针非零、层级 = 父层级 − 1 且 ≥ 1、
// 条目宽 ≥ 120）往下走，别的类型段、读不出、形状不对都交回 false（判不了）
fn d_read_inode_records_from_the_nodes_of_the_tree(root_pointer: Ptr) -> bool {
    d_inode_record_count = 0u;
    d_inode_records_are_indexed = false;
    if (root_pointer.all_zero) {
        return true;
    }
    let root = r_read_located_index_node(root_pointer);
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
            let container = parse_packed_unit(r_read_located_unit(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
            if (!container.ok || container.record_type != DELETION_PACKED_TYPE_INODE
                || container.record_width < DELETION_INODE_RECORD_BYTES) {
                return false;
            }
            for (var r = 0u; r < container.record_count; r = r + 1u) {
                let record = packed_record_offset(container, r);
                if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + INODE_RECORD_NLINK_OFFSET),
                    rd64(container.base, record + INODE_RECORD_SIZE_OFFSET), rd64(container.base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET),
                    rd64(container.base, record + INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET),
                    rd64(container.base, record + INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET))) {
                    return false;
                }
            }
        } else if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
            if (child_pointer.all_zero) {
                return false;
            }
            let child = r_read_located_index_node(child_pointer);
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
        let n = r_read_located_index_node(p);
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
        read.queue = r_read_located_index_node(queue_pointer);
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
// 读每个号的数据单元判 I-4.7（对象取丙）与 I-4.6、I-4.10、I-9.38（每条记录，d_judge_the_data_units_of_each_record）
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
    // 配不上恰好一棵 inode 树、队列读不出、inode 树的记录读不全：I-9.5、I-4.7、I-4.6、I-4.10、I-9.38 记射程里有一部分没判（walk.rs DELETION_QUEUE_SCAN_INVARIANTS）
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
    // 每个号的数据单元（walk.rs data_units_of_each_inode_without_judging）：根指针全零的树一个单元都没有
    d_data_unit_inode_count = 0u;
    d_data_unit_count = 0u;
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
        let count_row = D_DATA_UNITS_OF_EACH_INODE + i * D_DATA_UNITS_OF_AN_INODE_WORDS;
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
    // I-4.6、I-4.10、I-9.38、I-9.39：每条记录（截断那一批起 I-4.6 不只判 nlink = 0 的记录）
    d_judge_the_data_units_of_each_record(tt);
}

// 删除态扫描判不全时记射程里有一部分没判的三组（walk.rs judge_deletion_state_of_the_versions 与 judge_deletion_queue_of_a_version）：
// 整张树表扫不了是 DELETION_QUEUE_SCAN_INVARIANTS 那六条加 I-3.4、I-9.24；队列配不上 inode 树、读不全是 DELETION_QUEUE_SCAN_INVARIANTS 那六条；
// extent 树配不上、读不全是 DELETION_SCAN_INVARIANTS_READING_THE_EXTENT_TREE 那五条（I-4.7、I-4.6、I-4.10、I-9.38、I-9.39）
fn d_mark_the_inodes_being_deleted_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_4_7);
    mark_part_of_the_range_not_judged(INVARIANT_I_4_6);
    mark_part_of_the_range_not_judged(INVARIANT_I_4_10);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_38);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_39);
}

fn d_mark_the_deletion_queue_scan_not_judged() {
    mark_part_of_the_range_not_judged(INVARIANT_I_9_5);
    d_mark_the_inodes_being_deleted_not_judged();
}

fn d_mark_the_deletion_scan_not_judged() {
    d_mark_the_deletion_queue_scan_not_judged();
    mark_part_of_the_range_not_judged(INVARIANT_I_3_4);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_24);
    mark_part_of_the_range_not_judged(INVARIANT_I_9_40);
}

// ── I-9.40：extent 上段的规范形（walk.rs 的 extent_upper_segment_shape_without_judging、canonical_extent_upper_segment_of、
// departure_of_the_extent_upper_segment_from_its_canonical_shape、judge_extent_upper_segment_shape_of_a_version）
//
// CPU 读一版上段（不进下段）记三样：根层级、读到的节点位置集合 P、叶条目的 inode 号集合 S，再与 S 定的规范形比：根层级是罩得住 max(S) 的最低一层 R
// （S 空时是 0），节点是 S 里每个号在 0..=R 每一层上罩着它的那一格。GPU 不把这两个集合抄进表（按块分批读、不设上限），按下面两条比：
// 读到的位置都是从根 (根层级, 0) 沿条目一层层往下定的，每个孩子的序号整除扇出就是父的序号，P 对祖先封闭；所以根层级等于 R 时
// ① P ⊆ 规范形 ⟺ 每个读到的节点罩着 S 里至少一个号；② 规范形 ⊆ P ⟺ S 里每个号那片叶的位置 (0, 号 ÷ 143) 读到了（它的祖先跟着都在）。
// 一遍深度优先走全上段，每个节点按后序记「子树里有没有落在它那片叶自己区间里的条目」：有的节点 ① 成立，落在自己那片叶区间里的号 ② 成立；
// 剩下的节点与号（写路径写不出，改过字节的镜像才有）逐个再走上段：找罩着那个节点的号、找那个号的那片叶。每一遍都是同一个读法、同一份字节。
const UPPER_SHAPE_WALK_TAKES_THE_SHAPE: u32 = 0u;
const UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES: u32 = 1u;
const UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION: u32 = 2u;
const UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF: u32 = 3u;
const UPPER_SHAPE_WALK_FINDS_THE_LEAF: u32 = 4u;
// 走上段的节点栈越过 C_R_PENDING 那张表（层级至多 255；写路径写出的上段不过十来层）
const UPPER_SHAPE_SITE_FRAMES_PAST_THE_STACK: u32 = UNDECIDABLE_TOO_DEEP | (116u << 8u);

// 走一遍上段记下的（u_walk_the_extent_upper_segment 写）：根层级、有没有叶条目、最大的号、子树里没有落在自己那片叶区间里的条目的节点数、
// 落在所在那片叶区间之外的条目数，与这一遍要找的那一个找到了没有、是哪一格、哪个号
var<private> u_root_level: u32;
var<private> u_has_an_entry: bool;
var<private> u_largest_inode: vec2<u32>;
var<private> u_nodes_without_an_entry_of_their_own_leaves: u32;
var<private> u_entries_outside_their_leaf: u32;
var<private> u_found: bool;
var<private> u_found_level: u32;
var<private> u_found_index: vec2<u32>;
var<private> u_found_inode: vec2<u32>;

// 上段一个节点走完（叶读完条目、内部节点的孩子都走完）：frames_left 是走完之后栈里还有几帧（父是最上面那一帧）。子树里有没有落在自己那片叶区间里的
// 条目并进父那一帧、父往下一条条目；没有的数一个，这一遍找第 wanted_ordinal 个这样的节点时记下它的位置
fn u_finish_a_node(frames_left: u32, level: u32, index: vec2<u32>, has_an_entry_of_its_own_leaves: bool, mode: u32, wanted_ordinal: u32) {
    if (!has_an_entry_of_its_own_leaves) {
        if (mode == UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES && u_nodes_without_an_entry_of_their_own_leaves == wanted_ordinal) {
            u_found = true;
            u_found_level = level;
            u_found_index = index;
        }
        u_nodes_without_an_entry_of_their_own_leaves = u_nodes_without_an_entry_of_their_own_leaves + 1u;
    }
    if (frames_left > 0u) {
        let parent = C_R_PENDING + (frames_left - 1u) * C_R_PENDING_WORDS;
        sws(parent + 4u, sw(parent + 4u) + 1u);
        if (has_an_entry_of_its_own_leaves) {
            sws(parent + 5u, 1u);
        }
    }
}

// 走一版 extent 树的上段（根指针在 base 起 root_offset 处），照 walk.rs 的 extent_upper_segment_shape_without_judging 只读不判：读不出、层级或 key 宽
// 对不上、叶条目解不开（extent_upper_leaf_entry_view）、内部条目的 key 不在父那一格里（extent_upper_child_of_entry_key）、同一个落点读到两次，交回 false
// （CPU 交 None）。mode 说这一遍另找什么（UPPER_SHAPE_WALK_*）：找第 wanted_ordinal 个；罩着哪一格的号按 wanted_span 除、比 wanted_index；
// 找那片叶时比 wanted_index
fn u_walk_the_extent_upper_segment(base: u32, root_offset: u32, mode: u32, wanted_ordinal: u32, wanted_span: vec2<u32>, wanted_index: vec2<u32>) -> bool {
    u_root_level = 0u;
    u_has_an_entry = false;
    u_largest_inode = vec2<u32>(0u, 0u);
    u_nodes_without_an_entry_of_their_own_leaves = 0u;
    u_entries_outside_their_leaf = 0u;
    u_found = false;
    r_seen_count = 0u;
    var frames = 0u;
    var pending_base = base;
    var pending_offset = root_offset;
    var pending_level = NONE;
    var pending_index = vec2<u32>(0u, 0u);
    var enter = true;
    // 迭代上界：每个节点进一次（同一落点读两次交 false，见过的表 C_R_MAX_SEEN 个落点）、每条内部条目往下一次；跨轮带着的是节点栈
    // （C_R_PENDING 的前 frames 行：字节基、层级、序号 lo/hi、下一条条目、子树里有没有落在自己那片叶区间里的条目）
    loop {
        if (enter) {
            let p = parse_node_pointer(pending_base, pending_offset);
            if (p.all_zero || !r_seen_insert(p.device0, p.slot0)) {
                return false;
            }
            let n = r_read_located_index_node(p);
            if (!n.ok) {
                return false;
            }
            var level = pending_level;
            if (level == NONE) {
                level = n.level;
                u_root_level = n.level;
            }
            if (n.level != level || n.key_width != 24u) {
                return false;
            }
            if (level == 0u) {
                if (n.entry_width < EXTENT_UPPER_LEAF_ENTRY_BYTES) {
                    return false;
                }
                var has_an_entry_of_its_own_range = false;
                for (var e = 0u; e < n.entry_count; e = e + 1u) {
                    let o = entry_offset(n, e);
                    if (!is_zero64(extent_key_locality(n.base, o)) || !is_zero64(extent_key_offset(n.base, o))) {
                        return false;
                    }
                    let tag = rd8(n.base, o + 24u);
                    // 标签 3 同 1（指针之后两字节要是 0）、4 同 2（判决 m4-trunc-r1 K5）
                    let names_a_lower_segment_root = tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE
                        || tag == EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT;
                    if (tag == EXTENT_UPPER_LEAF_TAG_NO_DATA_UNIT) {
                        if (!all_zero(n.base, o + 25u, 88u)) {
                            return false;
                        }
                    } else if (names_a_lower_segment_root) {
                        if (!all_zero(n.base, o + 25u + 86u, 2u)) {
                            return false;
                        }
                    } else if (tag != EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT && tag != EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT_FOLLOWED_BY_A_HOLE) {
                        return false;
                    }
                    let inode = extent_key_inode(n.base, o);
                    if (!u_has_an_entry || lt64(u_largest_inode, inode)) {
                        u_largest_inode = inode;
                    }
                    u_has_an_entry = true;
                    if (eq64(divide_by_a_divisor_below_65536(inode, EXTENT_UPPER_LEAF_INODES).quotient, pending_index)) {
                        has_an_entry_of_its_own_range = true;
                    } else {
                        if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF && u_entries_outside_their_leaf == wanted_ordinal) {
                            u_found = true;
                            u_found_inode = inode;
                        }
                        u_entries_outside_their_leaf = u_entries_outside_their_leaf + 1u;
                    }
                    if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION && eq64(divmod64(inode, wanted_span).quotient, wanted_index)) {
                        u_found = true;
                    }
                }
                if (mode == UPPER_SHAPE_WALK_FINDS_THE_LEAF && eq64(pending_index, wanted_index)) {
                    u_found = true;
                }
                u_finish_a_node(frames, level, pending_index, has_an_entry_of_its_own_range, mode, wanted_ordinal);
                if (frames == 0u) {
                    break;
                }
                enter = false;
                continue;
            }
            if (n.entry_width < 110u) {
                return false;
            }
            if (frames >= C_R_MAX_PENDING) {
                undecidable = UPPER_SHAPE_SITE_FRAMES_PAST_THE_STACK;
                return false;
            }
            let pushed = C_R_PENDING + frames * C_R_PENDING_WORDS;
            sws(pushed, n.base);
            sws(pushed + 1u, level);
            sws(pushed + 2u, pending_index.x);
            sws(pushed + 3u, pending_index.y);
            sws(pushed + 4u, 0u);
            sws(pushed + 5u, 0u);
            frames = frames + 1u;
            enter = false;
            continue;
        }
        let top = C_R_PENDING + (frames - 1u) * C_R_PENDING_WORDS;
        let node_base = sw(top);
        let level = sw(top + 1u);
        let index = vec2<u32>(sw(top + 2u), sw(top + 3u));
        let entries = index_node_entries_of_the_header(node_base);
        let next_entry = sw(top + 4u);
        if (next_entry >= entries.entry_count) {
            frames = frames - 1u;
            u_finish_a_node(frames, level, index, sw(top + 5u) == 1u, mode, wanted_ordinal);
            if (frames == 0u) {
                break;
            }
            continue;
        }
        // position_addressed::extent_upper_child_of_entry_key：key 是 (0, 孩子那一格的第一个号, 0)，号整除孩子那一层的跨度，孩子的序号整除扇出是父的序号
        let o = entries.entries_start + next_entry * entries.entry_width;
        let child_level = level - 1u;
        let child = divmod64(extent_key_inode(node_base, o), extent_upper_span(child_level));
        if (!is_zero64(extent_key_locality(node_base, o)) || !is_zero64(extent_key_offset(node_base, o)) || !is_zero64(child.remainder)) {
            return false;
        }
        if (!eq64(divide_by_a_divisor_below_65536(child.quotient, EXTENT_INTERNAL_FANOUT).quotient, index)) {
            return false;
        }
        pending_base = node_base;
        pending_offset = o + 24u;
        pending_level = child_level;
        pending_index = child.quotient;
        enter = true;
    }
    return true;
}

// I-9.40 判一棵 extent 树（根指针在 base 起 root_offset 处）：先走一遍取形状，读不全报射程里判不了（walk.rs 同）；根层级不是 R 判违例；
// 再逐个核没有落在自己那片叶区间里的条目的节点（找罩着它的号）与落在所在那片叶区间之外的号（找它那片叶）。几遍走法在一个循环里、一个调用点
fn u_judge_the_extent_upper_segment_shape(base: u32, root_offset: u32) {
    var mode = UPPER_SHAPE_WALK_TAKES_THE_SHAPE;
    var ordinal = 0u;
    var wanted_span = vec2<u32>(0u, 0u);
    var wanted_index = vec2<u32>(0u, 0u);
    var nodes_without_an_entry_of_their_own_leaves = 0u;
    var entries_outside_their_leaf = 0u;
    var holds = true;
    // 迭代上界：取形状一遍，加上那几个节点与那几个号各两遍；跨轮带着的是这一遍找什么（mode）与第几个（ordinal）
    loop {
        let readable = u_walk_the_extent_upper_segment(base, root_offset, mode, ordinal, wanted_span, wanted_index);
        if (!readable) {
            // 取形状那一遍读不全是 CPU 的 None；之后几遍读同一份字节、同一个读法，只有见过的表或节点栈满了（判不了已记）才走不完
            if (mode == UPPER_SHAPE_WALK_TAKES_THE_SHAPE) {
                mark_part_of_the_range_not_judged(INVARIANT_I_9_40);
            }
            return;
        }
        if (mode == UPPER_SHAPE_WALK_TAKES_THE_SHAPE) {
            // canonical_extent_upper_segment_of：罩得住最大号的最低一层（罩的号数大于它），层级至多 255；没有叶条目是层级 0
            var canonical_root_level = 0u;
            if (u_has_an_entry) {
                var span = u64_of(EXTENT_UPPER_LEAF_INODES);
                loop {
                    if (lt64(u_largest_inode, span) || canonical_root_level >= 255u) {
                        break;
                    }
                    canonical_root_level = canonical_root_level + 1u;
                    span = mul64_32_saturating(span, EXTENT_INTERNAL_FANOUT);
                }
            }
            holds = u_root_level == canonical_root_level;
            if (!holds || !u_has_an_entry) {
                break;
            }
            nodes_without_an_entry_of_their_own_leaves = u_nodes_without_an_entry_of_their_own_leaves;
            entries_outside_their_leaf = u_entries_outside_their_leaf;
            ordinal = 0u;
            if (nodes_without_an_entry_of_their_own_leaves > 0u) {
                mode = UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES;
                continue;
            }
            if (entries_outside_their_leaf > 0u) {
                mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF;
                continue;
            }
            break;
        }
        if (mode == UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES) {
            wanted_span = extent_upper_span(u_found_level);
            wanted_index = u_found_index;
            mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION;
            continue;
        }
        if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_UNDER_THE_POSITION) {
            // ①：那个节点的位置不罩 S 里任何一个号，是规范形之外多出来的节点
            if (!u_found) {
                holds = false;
                break;
            }
            ordinal = ordinal + 1u;
            if (ordinal < nodes_without_an_entry_of_their_own_leaves) {
                mode = UPPER_SHAPE_WALK_FINDS_A_NODE_WITHOUT_AN_ENTRY_OF_ITS_OWN_LEAVES;
                continue;
            }
            ordinal = 0u;
            if (entries_outside_their_leaf > 0u) {
                mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF;
                continue;
            }
            break;
        }
        if (mode == UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF) {
            wanted_index = divide_by_a_divisor_below_65536(u_found_inode, EXTENT_UPPER_LEAF_INODES).quotient;
            mode = UPPER_SHAPE_WALK_FINDS_THE_LEAF;
            continue;
        }
        // ②：那个号那片叶的位置没读到，规范形里的这一格少了
        if (!u_found) {
            holds = false;
            break;
        }
        ordinal = ordinal + 1u;
        if (ordinal < entries_outside_their_leaf) {
            mode = UPPER_SHAPE_WALK_FINDS_AN_ENTRY_OUTSIDE_ITS_LEAF;
            continue;
        }
        break;
    }
    judge(INVARIANT_I_9_40, holds);
}

// walk.rs judge_deletion_state_of_the_versions 的一版：树表指针在 (base, offset)。树表 0 条的不判；指针全零、树表读不出、条目宽不到 200 的
// 判不了（记射程里有一部分没判）
// mapping_root_offset 起是这一版的中央映射根指针（候选根取根记录偏移 256，由记录施加出来的那一版取记录新根段偏移 86）：几棵树里豁免三类之外的
// 节点、码 3 容器与数据单元按它认位置（walk.rs judge_deletion_state_of_the_versions 每一版的 central_mapping）
fn d_judge_deletion_state_of_a_version(base: u32, tree_table_pointer_offset: u32, mapping_root_offset: u32) {
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
    s_use_the_central_mapping_named_at(base, mapping_root_offset);
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
    // I-9.40（walk.rs judge_extent_upper_segment_shape_of_a_version）：每一棵根指针非零的 extent 树（种类 1），按这一版的中央映射认位置
    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
        let entry = entry_offset(tt, entry_index);
        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_EXTENT
            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
            continue;
        }
        u_judge_the_extent_upper_segment_shape(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
    }
}

// 删除态扫描：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针），同 I-5.5 那一份清单
fn judge_deletion_state_of_the_versions() {
    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
        let version = d_version_of_the_scan(version_index);
        d_forget_the_inode_records_read_in_full();
        d_judge_deletion_state_of_a_version(version.base, version.tree_table_offset, version.mapping_root_offset);
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
    // 墓碑树的节点与墓碑容器按所选根自己的中央映射认位置（walk.rs 同处的 central_mapping）
    s_use_the_central_mapping_named_at(newest_root.base, G_ROOT_RECORD_MAPPING_ROOT_POINTER);
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
        let n = r_read_located_index_node(parse_node_pointer(sw(row), sw(row + 1u)));
        if (!n.ok || n.level == 0u || n.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
            mark_part_of_the_range_not_judged(INVARIANT_I_9_6);
            return;
        }
        for (var e = 0u; e < n.entry_count; e = e + 1u) {
            let o = entry_offset(n, e);
            let child_pointer = parse_node_pointer(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
            let entry_type = rd16(n.base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
            if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
                let child = r_read_located_index_node(child_pointer);
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
                let container = parse_packed_unit(r_read_located_unit(child_pointer, MAPPING_CLASS_PACKED_RECORD_UNIT, DATA_UNIT_BYTES));
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
